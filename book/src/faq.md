# FAQ

Common questions about how `anvil` works and why. For the
authoritative specifications, see the [Structural Rules
catalog](structural-rules.md). For the algorithmic details, see
[The Fanin Cone Algorithm](algorithm.md) and
[Construction Strategies](construction-strategies.md).

## Why is `Sub` not N-arity?

N-arity only makes sense for **associative** operators — grouping
doesn't matter, so `a ⊕ b ⊕ c` is well-defined regardless of how
you parenthesize. `Sub` is not associative: `(a − b) − c ≠ a − (b − c)`.
SV's left-associative parse makes `a - b - c` mean `((a - b) - c)`,
but that's a parse convention, not algebra. `anvil` keeps Sub
strictly 2-arity in the IR; if you want `a - b - c` in the output,
it comes from a cascade of two 2-arity Sub nodes.

`And`, `Or`, `Xor`, `Add`, `Mul` are associative and N-arity by the
configured `[min_gate_arity, max_gate_arity]` range.

## Why do operators have "arity" but blocks have "ports"?

Operators (associative primitives) generalize by **arity** — the
count of same-shape operands. Blocks (functional units with
structure: mux, flop, memory) generalize by **ports** — port
counts, encoding choices, feedback topology, reset kind. The two
are fundamentally different kinds of generalization.

Say "arity" only for operators. Say "ports" / "arms" for blocks.
The vocabulary discipline keeps generalization strategies from
getting conflated. See `structural-rules.md` "Operators vs blocks".

## How can a flop's Q appear inside its own D-cone without violating the no-loop rule?

Rule 1 (Combinational no-loop) concerns purely-combinational paths
— a gate's output transitively feeding back into its own input
through other gates only. Rule 2 permits Q→D feedback because the
flop breaks the loop **temporally**: `Q[n+1] = f(Q[n], …)` is the
definition of synchronous state, not a violation of anything. Arena-
index monotonicity still holds for the combinational pieces;
the flop is the only node type whose output logically feeds its
input, and it does so across a clock edge.

## What's the difference between "coefficient", "shift amount", and "comparand"?

All three are integer literals appearing as operands. They look the
same syntactically but have distinct **semantic roles**:

- **Coefficient** (arithmetic): a multiplicative weight in a linear
  combination. `3*a + 2*b + c`. Applies to `Add`, `Sub`, `Mul` with
  per-op constraints.
- **Shift amount** (shifts): a structural parameter of `Shl`/`Shr`
  telling you how far to shift. `a << 2`. Not a coefficient — even
  though `a << 2` is arithmetically `a * 4`, the RTL representation
  and synthesis cost are different (wire reroute vs multiplier).
- **Comparand** (comparisons): a threshold / sentinel value on the
  RHS of a comparison. `a == 7`, `x < LIMIT`. Not a coefficient —
  "what are we comparing against," not "how much are we scaling."

Each has its own knob family (`coefficient_*`, `const_shift_amount_*`,
`const_comparand_*`). Do not collapse them into a single
`constant_prob` knob — doing so loses the semantic distinctions.
See [Roles of constants in RTL](structural-rules.md).

## Why three construction strategies instead of just the default?

The three strategies (`sequential`, `shuffled`, `interleaved`)
differ in *when* gates are created relative to each other, and
therefore in *how symmetric* cross-output sharing is.

- `interleaved` (default) gives near-symmetric cross-output
  sharing via a single global frame queue driving all cones in
  lockstep. Each cone's leaf-level picks see the full
  module-wide pool.
- `sequential` builds cones one output at a time in declaration
  order — the original behavior, useful for reproducing output
  generated against older `anvil` versions and for exercising
  declaration-order-biased tooling.
- `shuffled` is `sequential` with a random output-build order per
  seed. Amortises the asymmetry across a seed sweep rather than
  eliminating it.

A fourth historical strategy, `graph-first`, was **retired** for
producing 10–30% orphan gates per module (Rule 18 violation).
`--construction-strategy graph-first` is accepted as a silent
alias for `interleaved` for backward compatibility. See the
[retirement rationale](construction-strategies.md#retired-graph-first).

## Can output J's cone reference a gate from output I's cone, regardless of declaration order?

Yes. The signal pool is module-scoped, not cone-scoped
([Rule 16](structural-rules.md)). In `interleaved`, cones grow in
lockstep so declaration order doesn't create asymmetry. In
`sequential` / `shuffled`, cones have construction-order
asymmetry, but any gate created before the current pick is
available regardless of which output's cone created it. And
construction-time CSE ([Rule 21](structural-rules.md)) means two
cones that independently build the same AST share a single
`NodeId` automatically.

## What does "full factorization" mean in the book? Does `anvil` deduplicate expressions?

Yes. `NodeId` is the **identity** of an expression in the IR: two
equivalent expressions should collapse to one `NodeId`, regardless
of which output cone first built them or how they were spelled
syntactically. For combinational nodes this is enforced at
construction time via `Module::intern_gate` / `intern_constant`.
For state, there is one conservative post-drain pass that merges
exact duplicate flop signatures once D-cones exist.

Today the live ladder reaches through **peephole**:

1. **Syntactic CSE** (Rule 21) — same `(op, operands, width)` ⇒
   same `NodeId`.
2. **Operand uniqueness** (Rule 8 extended) — no `NodeId` twice in
   one operand list (with the documented Add/Mul and Mux knobs).
3. **Commutative normalization** (Rule 21b) — `a + b` and `b + a`
   share identity.
4. **Associative flattening** — `Add(a, Add(b, c))` canonicalises to
   `Add(a, b, c)` when flattening is semantically safe.
5. **Constant folding** — identity/absorbing constants and all-const
   subexpressions collapse at intern time.
6. **Peephole rewrites** — local canonical rewrites like
   `Not(Not(x))`, constant comparison evaluation, full-width `Slice`,
   and single-operand `Concat`.
7. **Exact-signature flop merge** — after the flop worklist drains,
   flops with identical `width`, reset, and exact same `d: NodeId`
   share one state element.

Only the final **`e-graph`** rung remains aspirational. A user at
`--identity-mode node-id --factorization-level e-graph` (or the
shortcut `--full-factorization`) gets the strongest implemented
behaviour today, which currently means `peephole` plus every
lower layer plus that conservative flop merge. `--identity-mode relaxed` (or the shortcut
`--no-full-factorization`) is the coarse off-switch.

Construction strategy is a separate axis. `sequential`,
`shuffled`, and `interleaved` decide **how cones are built**;
factorization decides **when two built expressions share one
identity**.

Dial: `--identity-mode <node-id|relaxed>` plus
`--factorization-level <none|cse|operand-unique|commutative|
associative|constant-fold|peephole|e-graph>`, or the convenience
aliases `--full-factorization` / `--no-full-factorization`. See Rule 21c.

## How do I reproduce a specific generated module?

Every invocation is deterministic in `(seed, knobs)`. Run
`anvil --dump-config > knobs.json` to capture effective knobs, then
replay with `anvil --config knobs.json --seed <seed>`. The output
manifest (`manifest.json` in the `--out` directory) records both
the seed and the effective knobs per batch so any module can be
reproduced from its entry alone.

## Can `anvil` generate testbenches, assertions, or coverage?

No. `anvil` generates DUT code only. Testbenches require semantic
understanding of the DUT (what inputs are legal, what outputs
mean). A random testbench for a random DUT tests nothing. See
[What We Explicitly Do Not Do](non-goals.md) for the full list.

## Is the generated SystemVerilog synthesizable?

Yes, by construction. The gate set and the flop pattern are a
strict subset of synthesizable SV. There is no mode that emits
`initial` blocks, delays, dynamic arrays, or other
non-synthesizable constructs — those constructs don't exist in the
IR or the emitter. See
[Synthesizability as a Subset Constraint](synthesizability.md).

## Is the generated logic meaningful?

No, and that's the point. The circuits are *structurally* valid
and *functionally* non-trivial (every output depends on at least
one input), but the specific function is random — `a + (b ^ c) * 3`
or similar, with no design intent. `anvil` is for **stress-testing
tools** (parsers, elaborators, simulators, synthesizers, formal
equivalence checkers), not for generating real designs.

If you need RTL that *does* something meaningful, you hire an
engineer.

## Is `anvil` trying to generate functionally correct whole modules?

No. For most generated modules, whole-module function correctness is not
even a meaningful target because there is no specification to compare
against.

ANVIL is built by recursively generating fanin cones. That process is
great at producing legal, synthesizable, structurally rich RTL, but it
mechanically tends to produce arbitrary or gibberish overall behavior.
That is acceptable because ANVIL is optimizing for structure and
downstream-tool ingestibility, not top-level design intent.

The important distinction is:

- **whole modules** are usually arbitrary in behavior; but
- **local motifs / blocks** may still be functionally correct by
  construction (for example a mux as a mux, a flop as a flop, a
  priority encoder as a priority encoder, and future memories / FSM
  templates in their own local sense).

## Is `anvil` trying to be a signoff-grade bug finder for downstream tools?

Yes. That is the intended direction.

More precisely: `anvil` is trying to become a **signoff-level quality
random synthesizable RTL generator** whose outputs are clean in tools
like Verilator and Yosys by default, while still being rich enough to
expose real bugs in parsers, elaborators, synthesizers, and similar
consumers.

Those two goals are not in tension. The point is **not** to find bugs
by emitting malformed junk. The point is to find bugs with legal,
reproducible, structurally disciplined RTL that exercises hard corners
of the design space.

## What SystemVerilog language standard does `anvil` target?

The **synthesizable subset**. Emitted constructs are accepted by
Verilator, Yosys, Vivado, Design Compiler, and Synopsys VCS in
synthesis / elaboration / lint modes. `anvil` does not target a
specific IEEE standard version; the subset chosen is conservative
enough to work across the common tool landscape.

## Why does my module have `clk` and `rst_n` even though my outputs look purely combinational?

When the generator emits at least one flop, `clk` and `rst_n` are
declared as input ports (shared by every flop in the module — see
Rule 5, single-clock synchronous discipline). If you want purely
combinational output, pass `--flop-prob 0.0`. Then `clk` and
`rst_n` are omitted from the port list.

## The same seed produced different output after I upgraded `anvil`

`anvil` guarantees byte-identical output for a given `(seed, knobs)`
across platforms and time — but *not* across versions. A generator
change (new motif, changed default, bug fix) shifts the RNG
consumption pattern and produces different output for the same
seed. Record the `anvil` version alongside the seed and knobs when
you need to replay a specific module across a version boundary.

## Anything I should know about generated modules being fed back to Verilator / Yosys?

Nothing beyond the usual tool invocation. `anvil` output is
directly consumable:

```bash
verilator --lint-only anvil_output.sv
yosys -p "read_verilog -sv anvil_output.sv; synth; stat"
```

Both should succeed on every generated module. If one fails, it is
a generator bug — file an issue with the seed, effective knobs,
and the failing output.
