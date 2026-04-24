# Structural Rules

This chapter is the definitive catalog of structural rules that `anvil`
enforces. Every rule here is:

- **Load-bearing** — correctness of generated output depends on it.
- **Enforced by construction** — the generator cannot violate it by
  accident; the IR shape or the generator logic makes violation
  structurally impossible.
- **Cross-checked by the validator** (`src/ir/validate.rs`) as a
  development-time safety net.

This catalog grows as `anvil` matures. New rules land here the moment
they become invariants of the generator, not a release cycle later.

## Operators vs blocks

A distinction that runs through the whole catalog. The two words are
not interchangeable:

- An **operator** is an associative primitive function. Arity only
  makes sense for operators. Arity generalization = same-width
  operands combined by the same op; grouping does not matter
  algebraically. `And`, `Or`, `Xor`, `Add`, `Mul` are operators.
  `Sub` is *not* associative and therefore stays strictly 2-arity.
  `Not` is unary.
- A **block** is a functional unit with internal structure. Blocks
  have **ports** (typed named inputs/outputs), not "arity". Their
  generalization is structural — port counts (M data + K select
  lines), encoding choices (one-hot vs encoded), feedback topology
  (Q→D via the mux vs Q free in fanin), reset kind. A mux is a
  block. A flop is a block. A memory is a block. Generalizing a
  block means enumerating *motif shapes*, not incrementing an arity
  counter.

**Vocabulary discipline:** say "arity" only when referring to an
operator's operand count. Say "ports" / "port count" / "arms" when
referring to a block's structural parameters. The distinction keeps
generalization strategies from getting conflated — N-arity is one
concrete pattern; block-motif generalization is a different,
structurally richer activity.

## Roles of constants in RTL

Integer literals appear as operands in many places in real RTL, but
the *semantic role* of the constant differs — and the vocabulary
should follow the role, not the syntax. Three distinct roles, each
with its own motif family, constraints, and (future) knobs. Do not
collapse them under a single "constant_prob" knob.

### Coefficient — arithmetic weight

- **Operators:** `Add`, `Sub`, `Mul`.
- **Role:** multiplicative weight in a linear combination. Each term
  of the form `signal * coefficient`.
- **Shapes:** compound motif.
  - **Add:** `y = (s1*c1) + (s2*c2) + ... + (sn*cn)`. N `Mul` nodes +
    one N-arity `Add`. Per-term coefficients.
  - **Sub (left-associative):** `y = (s1*c1) - (s2*c2) - ... - (sn*cn)`.
    N `Mul` nodes + N-1 chained 2-arity `Sub` nodes. Per-term
    coefficients.
  - **Mul:** `y = c * s1 * s2 * ... * sn`. A *single* coefficient c
    multiplying all N signals. One N+1-arity `Mul` node whose
    operands are `[const, s1, s2, ..., sn]`. All operands at the
    same width W (`bits(s1) == bits(s2) == ... == W`).
- **Per-op constraints on coefficients:**
  - **Add:** `ci ≠ 0` for all i (non-zero; a zero weight kills its
    term and makes it structurally dead). Anvil's implementation
    draws `ci` from `[min_coefficient, max_coefficient]` (strictly
    positive); signed-negative coefficients would be expressible as a
    future extension but today the generator uses positive values.
  - **Sub:** `ci > 0` for all i — strictly positive. A negative `ci`
    on a `- sk*ck` term would flip the sign to `+ sk*|ck|`, which is
    an Add contribution masquerading as a Sub term. Zero kills the
    term. Strictly positive preserves the subtractive character of
    the chain.
  - **Mul:** `c >= 1`. If `c == 1`, at least 2 signals are required
    (otherwise `1 * s1 = s1` is structurally dead). When `c >= 2`,
    a single signal is fine (`c * s1` is a scaled signal).
- **Knobs:** `coefficient_prob` (per-op probability of emitting the
  compound form instead of the standard operator; default 0.2),
  `min_coefficient` (default 1), `max_coefficient` (default 15).
- **Scope note:** "coefficient" is arithmetic vocabulary. It does
  not apply to shifts, comparisons, or any other op family.

### Shift amount — structural parameter

- **Operators:** `Shl`, `Shr`.
- **Role:** how far to shift. A structural parameter of the shift
  op, not a weight. Even though `a << 2` is arithmetically `a * 4`,
  the RTL representation and the synthesis cost are distinct —
  constant-amount shifts are a wire reroute; variable-amount shifts
  synthesize to a barrel shifter.
- **Two modes, both legal:**
  - **Constant shift amount:** `a << 2`. Dominant pattern in real
    designs. Cheap in hardware.
  - **Variable shift amount:** `a << count`, where `count` is a
    signal. Legal SV; synthesizes to a barrel shifter; expensive.
- **Knobs:** `const_shift_amount_prob` (per-shift probability the
  amount is a constant instead of a variable-amount signal; default
  `0.8` biasing toward real-design prevalence), `min_shift_amount`
  (default 0), `max_shift_amount` (default 7, clamped to `W-1` for
  a W-bit value to keep the shift semantically meaningful).
- **Where enforced:** `src/gen/cone.rs` — `build_shift_const_amount`
  emits `value_signal OP const` when the coin fires; otherwise the
  existing `input_widths_for(Shl|Shr, ...)` variable-amount path
  runs. Dispatched from `build_cone`, `process_signal_frame`, and
  `grow_pool_one_unit` right after `pick_gate`.
- Shifts are now pickable by `pick_gate` via a new `gate_shift_weight`
  bucket (default weight 1). They are disabled at `target_width == 1`
  (a 1-bit shift is always either the value or zero, both trivial).
- **Scope note:** "shift amount" is shift-op vocabulary. Not a
  coefficient; not a comparand.

### Comparand — threshold / sentinel

- **Operators:** `Eq`, `Neq`, `Lt`, `Gt`, `Le`, `Ge`.
- **Role:** the value being compared against. A threshold, a target,
  a sentinel — `a == 7`, `x < LIMIT`.
- **Two sources for a comparison's RHS, both legal:**
  - **Another signal:** `a == b`, `x < y`. Signal-vs-signal
    comparison. The default today — both operands come from
    recursive `build_cone`.
  - **A comparand (constant):** `a == 7`, `x >= LIMIT`. Threshold /
    sentinel pattern. Emitted per the `const_comparand_prob` knob
    (default 0.3). LHS is a recursive / pool signal cone of the
    chosen internal operand width K; RHS is a constant drawn from
    `[min_comparand, max_comparand]` clamped to `[0, 2^K - 1]`.
- **No zero-exclusion:** comparing to zero (`a == 0`, `a < 0`) is
  common and meaningful. Unlike coefficients, a zero comparand does
  not kill the operation.
- **Scope note:** "comparand" is comparison vocabulary. It is one of
  two sources for the comparison's RHS; the other source is another
  signal. The comparand motif *adds to* signal-vs-signal
  comparisons; it does not replace them.

### Why the distinction matters

Flattening all three into a single "constant-as-operand" mechanism
would:

- Apply Add's `ci ≠ 0` constraint to shift amounts and comparands
  where zero is fine.
- Use the same integer range for all three, even though coefficients
  are typically small, shift amounts are bounded by the operand
  width, and comparands span the full operand width.
- Lose the shift-amount "structural parameter vs signal operand"
  distinction, so there would be no way to bias toward the
  realistic constant-amount case.
- Lose the comparison "RHS is signal or constant" distinction, so
  the default (signal-vs-signal) would compete with the motif
  (constant comparand) instead of coexisting.

Three distinct motif families keep the semantic structure explicit.

The rules below are grouped by what they govern:

- **Combinational integrity:** Rules 1, 8, 10, 14 — no combinational
  loops; anti-collapse; per-gate width; operator N-arity.
- **Block: flop:** Rules 2, 3, 4, 5, 6, 7 — Q-feedback, mux-term,
  clk/rst_n visibility, single-clock discipline, single-drive, M
  arity exclusions.
- **Block: mux (combinational):** Rule 15 — M-to-1 mux with
  OneHot or Encoded select; no Q-feedback axis (combinational muxes
  have no state). Built as a compound gate tree like the flop D-mux
  helpers, minus the Q-feedback terms.
- **Block: priority encoder:** Rule 17 — N 1-bit request signals →
  `ceil(log2(N))`-bit index of the highest-priority asserted bit.
  Chained-ternary emission.
- **Module-wide sharing:** Rule 16 — the signal pool is
  module-scoped, not per-output. Gates built for output A's cone are
  freely available as operands / shared leaves in output B's cone
  and in every flop's D-cone.
- **Correctness guarantees:** Rules 9, 11, 12, 13 — non-triviality;
  synthesizable subset; deterministic naming; reproducibility.

---

## 1 — Combinational no-loop

**Rule:** The output of a combinational (non-flop) node cannot appear
upstream in its own fanin cone. There is no purely-combinational path
from any gate's output back to one of its own inputs.

**Why it holds by construction:** the `SignalPool` contains only
signals that *already exist* when a recursion step is made. A new
`Node::Gate` is pushed to `m.nodes` and `pool` only **after** its
operands are resolved. Therefore no gate can reference itself or any
later-created gate as an operand. Arena-index monotonicity
(`NodeId`s are assigned in construction order, and operands always
refer to lower `NodeId`s) is the structural invariant.

**Where enforced:** `src/gen/cone.rs` — `build_cone`, `try_share`.

---

## 2 — Flop Q-feedback freedom

**Rule:** A flop's own Q output may appear **any number of times** as
a leaf in **any** of its data, select, or direct-D sub-cones. Q→D
feedback through arbitrary combinational logic is legal.

**Why it is safe:** the clock edge breaks the Q→D loop temporally.
`Q[n+1]` depends on `Q[n]` plus possibly other inputs — a standard
synchronous feedback pattern (counters, accumulators, toggles, state
machines). Combinational self-reference within that Q→D logic is
still forbidden by Rule 1 above.

**Where enforced:** `src/gen/cone.rs` — `drain_flop_worklist`,
`drain_flop_one_hot`, `drain_flop_encoded` all pass `exclude = None`
to `build_cone_with_retry` for the D sub-cones, allowing the flop's
Q (already in the pool) to be picked as a leaf.

---

## 3 — Explicit Q-feedback mux term (QFeedback kind)

**Rule:** Orthogonal to Rule 2: when a flop has `FlopKind::QFeedback`,
the mux driving D adds an explicit Q fall-through term that fires
when no data select is asserted:

- **OneHot + QFeedback:** `D = OR_i({W{sel_i}} & data_i) | ({W{~(OR sel_i)}} & Q)`.
- **Encoded + QFeedback:** the index-0 slot is replaced with Q;
  the chained-ternary fall-through routes Q instead of `0`.

The ZeroDefault kind has no such term — D = 0 when no select fires.

**Where enforced:** `src/gen/cone.rs` — `assemble_flop_d_one_hot`
(conditional `make_none_selected` term), `assemble_flop_d_encoded`
(conditional fall-through selection).

---

## 4 — Clock and reset never appear as cone leaves

**Rule:** The module's `clk` and `rst_n` input ports exist in
`Module.inputs` but are **not** added to the `SignalPool`. No cone
can terminate at them; no gate can reference them as an operand.
Their sole use is the `always_ff` header and reset branch.

**Why:** referencing `clk` or `rst_n` in combinational logic would
produce patterns synthesis tools reject (`clk` driving combinational
data; `rst_n` appearing in a datapath). The structural exclusion
makes such misuse impossible.

**Where enforced:** `src/gen/module.rs` — `generate_leaf_module`
iterates `m.inputs` and seeds the pool only with entries whose id
differs from `m.clock` and `m.reset`.

---

## 5 — Single-clock / single-reset synchronous discipline

**Rule:** Every module is fully synchronous to exactly one clock
domain. Every flop uses the module's single `clk` (posedge) and
single `rst_n` (async, active-low). There is no per-flop clock
choice, no per-flop reset polarity choice, no mixed-edge flops.

**Why:** this matches real production synchronous-design practice
and keeps generated modules within the scope that synthesis and
formal tools handle without additional configuration.

**Where enforced:** the IR has no field for per-flop clock or per-flop
reset polarity. `Flop.reset_kind` is populated with `ResetKind::Async`
unconditionally by `build_flop_leaf`. Multi-clock would require new
IR fields and is an explicit future-phase item.

**Boundary rule:** `clk` / `rst_n` are emitted at a module boundary iff
that module carries sequential state locally or through instantiated
descendants. Pure comb-only modules stay control-free. Stateful
hierarchy parents keep the ports visible all the way up the
instantiated ancestor chain. Child output ports are different: a parent
may either name them explicitly through `Node::InstanceOutput` and use
them in parent logic, or leave them unconnected at the instance site
when they are genuinely unused.

---

## 6 — Single-drive on every output port

**Rule:** Each output port appears in `Module.drives` exactly once.
Multi-driver situations (two `assign` statements for the same output,
or an `assign` competing with an `always_ff` block) are structurally
impossible.

**Where enforced:** `src/gen/module.rs` — one `module.drives.push`
per output. Validator: `ir::validate::validate` rejects any port
whose drive count differs from 1.

---

## 7 — M = 1 mux arm structurally impossible

**Rule:** A flop's D-input mux arm count M is drawn from
`{0, 2, 3, ..., max_mux_arms}`. M = 1 is never drawn.

**Why:** a 1-arm mux is algebraically `sel ? data_0 : 0` (or `: Q`)
— a shape already covered by `M = 0` (direct cone) or `M = 2` (real
mux). Allowing M = 1 would bloat the generator's decision space
without expanding the output distribution.

**Where enforced:** `src/gen/cone.rs` — `pick_mux_arm_count`.

---

## 8 — Structural anti-collapse rules

**Rule:** The following gate shapes are forbidden at construction
time. The check is on `NodeId` equality, which catches both pure-tree
self-reference and sharing-induced self-reference (same pool entry
picked for multiple operands).

- **Idempotent / self-inverse N-arity operators** — `And`, `Or`,
  `Xor` at any arity: any `NodeId` may appear at most once in the
  operand list. Duplicate operands collapse the gate
  (`x & x = x`, `x | x = x`, `x ^ x = 0`, `x ^ x ^ x ^ x = 0`).
  The check is operand-multiset distinctness, not just pairwise.
- **Sub (2-arity):** `x - x = 0` — operands must differ.
- **Eq, Neq (2-arity):** `x == x = 1`, `x != x = 0` — operands
  must differ.
- **Mux:** `mux(s, a, a) = a` — data_true and data_false must
  differ.

Add and Mul are deliberately **not** on this list: `x + x = 2x`
and `x * x = x²` are algebraically meaningful, not collapses.

When a forbidden shape is detected, the generator falls back to
`pick_terminal` rather than emitting the degenerate gate.

**Where enforced:** `src/gen/cone.rs` — `violates_anti_collapse` in
`build_cone` (recursive path), `process_signal_frame` (interleaved
path), and `grow_pool_one_unit` (graph-first path). The helper
`has_duplicate_operand` performs the N-arity distinctness check
(O(N²) in operand count, N bounded by `max_gate_arity`).

**Downstream dedup:** `or_reduce_terms` (used by one-hot mux
assembly) deduplicates its input terms before building the
`Or`-chain, because identical per-arm product terms are
structurally possible when arms share the same sel+data. The
`make_none_selected` helper routes through `or_reduce_terms`, so
the `~(sel_0 | sel_1 | … | sel_{M-1})` collapse is caught the
same way.

**Not caught:** algebraic identities deeper in the tree (`(a+b) - b`,
`(a & b) | (a & ~b)`, etc.). A real synthesizer will fold those
away; the surrounding cone retains its non-trivial structure.

---

## 9 — Non-triviality: every output reaches an input

**Rule:** Every output cone root has a non-empty dependency set
(`DepSet`). An output whose cone collapses to purely constants is
rejected and regenerated (bounded retry, 4 attempts).

**Why:** a trivially-constant output is useless — synthesis reduces
it to a wire tied to the constant, and no downstream tooling is
meaningfully exercised.

**Where enforced:** `src/gen/cone.rs` — `build_cone_with_retry`
checks `node_deps(root).is_empty()` and rewinds the module state if
so. `ir::validate::validate` rejects the IR entirely if an output
cone root has empty deps after retries are exhausted.

**Flop-virtual contribution:** a `FlopQ` reference contributes a
virtual dep (`{virtual_id(flop)}`), so cones consisting entirely of
flop references are considered non-trivial. The flop's D-cone is
recursively required to be non-trivial too, propagating the
requirement until primary inputs are eventually reached (or until
the module is entirely self-contained state, which is legal).

---

## 10 — Per-gate width rules

**Rule:** Every `Node::Gate` operand and output width satisfies the
per-op width contract. Violations are a generator bug.

| GateOp                            | Output width W | Operand widths                                |
|-----------------------------------|----------------|-----------------------------------------------|
| `And / Or / Xor / Add / Mul`      | W              | [W, W, ...] (N ≥ 2; associative, see Rule 14) |
| `Sub`                             | W              | [W, W] (strictly 2-arity; not associative)    |
| `Not`                             | W              | [W]                                           |
| `Mux`                             | W              | [1, W, W]                                     |
| `Eq / Neq / Lt / Gt / Le / Ge`    | W = 1          | [K, K] for chosen K                           |
| `RedAnd / RedOr / RedXor`         | W = 1          | [K] for chosen K                              |
| `Shl / Shr`                       | W              | [W, any]                                      |
| `Slice { hi, lo }`                | W = hi-lo+1    | [K] with K > hi, hi≥lo                        |
| `Concat`                          | W = sum(Wᵢ)    | [W₁, W₂, ...], ≥ 1 op                         |

**Where enforced:** `src/gen/cone.rs` — gate constructors
(`make_and`, `make_mux`, `make_eq_const`, `replicate_to_width`,
`make_width_adapter`, `build_cone`'s `input_widths_for`). Validator:
`ir::validate::check_gate_shape` with dedicated `ValidateError`
variants (`GateArity`, `GateOperandWidth`, `GateOutputWidth`,
`GateOperandsMustMatch`).

---

## 11 — Synthesizable subset

**Rule:** The current leaf-RTL lane's gate set, flop pattern, and
emitter cover only the synthesizable subset of SystemVerilog. No
`initial`, no delays (`#N`), no `$display` / `$finish`, no dynamic
arrays, no classes, no events, no tasks, no fork/join, no `wait`, and
no latch-emitting datapath form in this lane (`always_comb` is not
emitted for datapaths — `assign` is used instead and always fully
defines its target). Future artifact families may broaden the source
surface, but they must remain valid-by-construction and synthesizable
too.

**Where enforced:** the IR has no node kinds for these constructs;
the emitter has no code path that produces them. Absence is
structural.

---

## 12 — Deterministic naming

**Rule:** Generated names are deterministic, uniform, and collision-free
within a module.

| Pattern             | Meaning                                                                    |
|---------------------|----------------------------------------------------------------------------|
| `clk`               | Clock input (emitted when module carries sequential state locally or through descendants) |
| `rst_n`             | Async active-low reset (same visibility rule as `clk`)                     |
| `i_N`               | Primary data input, `N` counts from 0                                      |
| `o_N`               | Primary output, `N` counts from 0                                          |
| `<gate_kind>_N`     | Internal gate wire. `<gate_kind>` is the lowercase `GateOp` name; `N` counts per-kind from 0. |
| `flop_N`            | Flop register, `N` is the `FlopId`                                         |

`<gate_kind>` values: `and`, `or`, `xor`, `not`, `add`, `sub`, `mul`,
`eq`, `neq`, `lt`, `gt`, `le`, `ge`, `mux`, `slice`, `concat`,
`red_and`, `red_or`, `red_xor`, `shl`, `shr`. Each kind maintains its
own counter within a module, so `and_0`, `or_0`, `mux_0` can coexist
in the same module without collision. The per-kind counter is
assigned during emission by a one-pass walk (`build_names`) over
`m.nodes` in declaration order, so names are a pure function of
declaration order — reproducible from the same `(seed, knobs)`.

**Why per-kind and not global:** reading generated SV should make the
structural shape of the module visible at a glance. A declaration
block full of `and_0 … and_12; mux_0 … mux_3; flop_0 … flop_9;`
tells you immediately what kind of logic dominates. An opaque
`w_0 … w_47` hides it.

**SV identifier legality:** although `and`, `or`, `xor`, `not` are
reserved gate-primitive keywords in SystemVerilog, identifiers like
`and_0` are lexed as a single identifier (longest-match) and are
distinct from the reserved words. The names are legal, synthesizable,
and unambiguous.

Same `(seed, knobs)` → byte-identical names.

**Where enforced:** `src/emit/sv.rs` — `build_names` (per-kind
counter assignment), `gate_kind_name` (kind → prefix mapping),
`flop_name` (flop id → `flop_N`), `node_ref` (single source of truth
for cross-reference), wire-declaration loop, flop-declaration loop.

---

## 13 — Reproducibility

**Rule:** Given the same `(seed, knobs)`, `anvil` produces
byte-identical output on any platform, any time.

**Why it holds:** a single `ChaCha8Rng` seeded from the user seed
drives every random choice. No `thread_rng`, no wall-clock, no
floating-point, no hash-map iteration in any code path that affects
output.

**Where enforced:** `src/gen/mod.rs` — `Generator::new` is the sole
RNG construction point. Tests: `tests/pipeline.rs::reproducibility`.

---

---

## 16 — Cross-output sharing via the module-wide signal pool

**Rule:** Internal signals created while building output A's fanin
cone (or any flop's D-cone) are freely available as leaves and
DAG-sharing candidates inside output B's cone (or any later flop's
D-cone). There is **no per-output isolation**. The signal pool is
module-scoped, not cone-scoped.

**Why it holds by construction:** `generate_leaf_module` constructs a
single `SignalPool` before building any cones, seeded with primary
data inputs. That pool is passed by `&mut` to every
`build_cone_with_retry` call (each output cone, each flop D-cone).
Every `Node::Gate` is added to the pool when constructed. So every
subsequent `pick_terminal` / `try_share` call sees the full history
of gates built so far in the module.

**Practical effect:**

- Realistic RTL shapes. Real designs routinely share intermediate
  signals across multiple outputs — an ALU's operand-decode logic
  feeds multiple result paths; a state encoder feeds multiple
  downstream datapaths.
- No multi-driver risk. Cross-output sharing means *multiple
  consumers* of the same signal — that is fine. The rule forbidden
  is multiple drivers (Rule 6 — one `drives` entry per output port).
- Asymmetric ordering: outputs are built in declaration order
  (`0, 1, ..., n_out-1`). Output `0` sees only primary inputs as
  sharing candidates; output `n_out-1` sees primary inputs plus every
  gate built during outputs `0..n_out-1` plus every gate built in
  flop D-cones drained so far. This is a by-product of the
  implementation, not a rule; it matches real-design patterns where
  later-declared logic often references earlier-computed
  intermediates.

**Where enforced:** `src/gen/module.rs` — one `SignalPool` for the
whole module, threaded through every cone build. `src/gen/cone.rs` —
`pick_terminal` and `try_share` iterate the pool with no cone-identity
filter.

**Combinational no-loop still holds cross-cone:** Rule 1 (arena-index
monotonicity) applies across the whole module, not per cone. A gate
in output A's cone can be an operand of a gate in output B's cone
because A's gate has a lower `NodeId`. The reverse cannot happen — B
cannot reference an A-gate not yet constructed, nor can A reference a
B-gate that comes later.

---

## 22 — Mux arm-duplication rate

**Rule:** The arms of an N-to-1 mux are picked under a
probabilistic uniqueness constraint governed by
`mux_arm_duplication_rate` ∈ `[0.0, 1.0]`. At each arm pick, if
the candidate signal duplicates one already picked for this mux,
it is kept with probability `mux_arm_duplication_rate` and
rejected (re-pick) otherwise. Rate `0.0` (default) → every arm
is a distinct signal; rate `1.0` → no constraint (arms may all
be connected to the same data).

**Motivation:** a mux with multiple arms connected to the same
data is structurally redundant — at best synthesizer-folded, at
worst confusing to readers. The degenerate 2-to-1 form
`(s)?(x):(x) = x` is the Rule 8 case; the N-to-1 generalisation
is "m of the M arms share the same signal." The knob keeps the
pathological form reachable for stress-testing downstream tools
without making it the default.

**Construction-time enforcement:**
`pick_datas_with_dup_cap(g, m, pool, width, count, exclude)` is
the picker used at every N-to-1 mux assembly site (pool-mode
and, in future, recursive-mode). It tracks the set of already-
picked NodeIds; on a duplicate candidate it flips
`gen_bool(rate)` — keep-or-retry — with a bounded 8-try budget.
After the budget, the candidate is accepted (best-effort)
rather than looping forever when the pool is too small to
satisfy uniqueness. The 2-to-1 `make_mux` special-cases
`a == b`: at rate `0.0` it collapses the layer (returns `a`
directly); at any rate `> 0.0` the caller's probabilistic
decision stands and the mux is emitted.

**Knob semantics:**
- `mux_arm_duplication_rate = 0.0` (default): all mux arms
  distinct (best-effort). `make_mux(s, a, a)` skips the layer.
- `mux_arm_duplication_rate = 1.0`: arm duplication
  unconstrained. `make_mux(s, a, a)` emits the degenerate
  ternary verbatim.
- Intermediate: probabilistic — expected duplication rate
  approximates the knob value over large seed sweeps.

**Where enforced:** `src/gen/cone.rs` —
`pick_datas_with_dup_cap` (pool-mode N-to-1 paths),
`make_mux` (2-to-1 gate + chained-ternary tail layers).

---

## 21b — Commutative normalization

**Rule:** For commutative operators (`And`, `Or`, `Xor`, `Add`,
`Mul`), operand lists are sorted in ascending `NodeId` order
before being used as the `intern_gate` key. Consequently,
`a + b` and `b + a` share a single `NodeId` — they compute the
same expression and so share identity per the full-factorization
doctrine.

**Why:** CSE keys on `(op, operands, width)`. Without
normalization, `Add([a, b], 8)` and `Add([b, a], 8)` are
distinct keys and would produce two NodeIds. Semantically
they're the same expression. The rule closes that gap.

**Scope:** strictly commutative ops only. Explicitly excluded:
- `Sub` — `a - b ≠ b - a`.
- `Mux` — operands have positional roles (sel, data_true,
  data_false).
- `Lt` / `Gt` / `Le` / `Ge` — order-sensitive. `Eq` / `Neq`
  are commutative but kept un-normalized for uniformity with
  the other comparison ops; could be split out if a future
  defect demands it.
- `Slice` / `Concat` / `Shl` / `Shr` — positional.
- `Not` / reductions — unary.

**Where enforced:** `src/ir/types.rs` `Module::intern_gate` —
`operands.sort_unstable()` before key construction, gated by a
`matches!` on the commutative set.

**Position in the factorization ladder:** CSE (21) → operand-
uniqueness (Rule 8 + knob) → commutative normalization (21b) →
**associative flattening (live)** → **constant folding (live)** →
**peephole rules (live)** → e-graph equivalence (theoretical
ceiling). Each layer tightens the NodeId-identity contract;
we land them incrementally as defects demand.

**The syntactic-vs-semantic boundary.** What the currently-
implemented layers (CSE, operand-uniqueness, commutative,
associative, constant-fold, peephole, and the bounded `e-graph`
fragment) guarantee is that **two expressions share one node**
when ANVIL can prove that with today's available tactics, plus a
curated set of structural and well-known algebraic identities
(`Add(a, Add(b, c)) = Add(a, b, c)`, `a ^ a = 0`, `x + 0`,
`x * 1`, `x & 0`, `x | all_ones`, `Eq(c1, c2)`, full-width
`Slice`, single-operand `Concat`, …) get collapsed at intern
time. The aspirational layer above extends the contract toward
**two semantically equivalent expressions share one node** — a
strictly harder problem that synthesis tools themselves solve
incompletely. Full factorization in the semantic sense is an
asymptote: we climb toward it one layer at a time, confident
that each layer tightens the identity contract without
sacrificing reproducibility or construction-time correctness.

## 21c — Identity mode + factorization level (user-controllable dial)

**Rule:** the coarse switch is `Config::identity_mode`, and the
fine-grained ladder within `identity_mode = node-id` is
`Config::factorization_level`.

`identity_mode` values:

`relaxed → node-id` (default)

- `relaxed`: disable the identity/factorization ladder entirely.
  Every `intern_gate` / `intern_constant` call allocates a fresh
  `NodeId`.
- `node-id`: NodeId means expression identity, which implies full
  factorization by definition. The factorization ladder below does not
  change that meaning; it is the current build's implementation /
  proof-depth dial inside that doctrine.

Within `identity_mode = node-id`, `factorization_level` selects
how far along the factorization chain the current build enforces or
proves that doctrine. Values in increasing order:

`none → cse → operand-unique → commutative → associative →
constant-fold → peephole → e-graph` (default).

Each level implies all lower ones. Levels above a fully-implemented
semantic engine activate every implemented layer today; at present the
top rung is a bounded live `e-graph` fragment rather than the full
theoretical ceiling.

CLI convenience aliases: `--full-factorization` requests
`--identity-mode node-id --factorization-level e-graph`, while
`--no-full-factorization` requests
`--identity-mode relaxed --factorization-level none`.

**Doctrinal anchor:** the user's "full factorization" doctrine
states that `NodeId` is the identity of an expression — two
expressions that are the same in the mathematical or logical
sense must share one `NodeId`; different expressions must have
different `NodeId`s. Interpreted strictly, that already implies full
factorization by definition. `relaxed` is the only intentional mode
where equivalent expressions may keep different `NodeId`s. `e-graph` is
the theoretical ceiling where the current build proves this for all
semantic equivalences. Today we approximate it with syntactic CSE +
operand-uniqueness + commutative normalization + associative
flattening + constant folding + a narrow set of peephole rewrites plus
a bounded semantic merge fragment at the `e-graph` rung. Future slices
will close the gap further via deeper peephole rewrites (e.g.
cross-gate identities like `(a + b) - b → a`) and a stronger e-graph
engine.

**How the ladder behaves inside `identity_mode = node-id`:**

| Level          | Enables                                                            |
|----------------|--------------------------------------------------------------------|
| `none`         | Current-build diagnostic/stress rung only. Every `intern_gate` / `intern_constant` creates a fresh `NodeId`. Useful for matrix coverage, but not the doctrinal meaning of `node-id`. |
| `cse`          | + Syntactic CSE: `(op, operands, width)` / `(width, value)` dedupe. Also enables the post-drain endpoint-preserving flop merge: under `identity_mode = node-id`, flops with the same `width`, reset, canonical leaf endpoints, and currently-proven D-cone functionality collapse to one state element. Construction-only provenance (`FlopKind`, cleared mux operand metadata) is ignored once `d` exists. Fires counted in `Metrics::flops_merged`. |
| `operand-unique` | + Rule 8 operand uniqueness for And/Or/Xor/Add/Mul (Add/Mul also gated by `operand_duplication_rate`). |
| `commutative`  | + Commutative-operand sort at intern time (Rule 21b).              |
| `associative`  | + Associative flattening at intern time: any `And`/`Or`/`Xor`/`Add`/`Mul` operand that is itself a same-op same-width gate is spliced into the outer operand list (`Add(a, Add(b, c)) → Add(a, b, c)`). Per-op semantic normalisation after the splice: `And`/`Or` dedup (`a & a = a`), `Xor` pair-cancel (`a ^ a = 0`), `Add`/`Mul` skip the flatten when it would produce duplicates (preserves `x + x = 2x` / `x * x = x²` under strict `operand_duplication_rate`). Inner gates orphaned by the splice are cleaned up by `compact_node_ids` at module finalisation. Fires counted in `Metrics::flatten_associative_applied`; residual nesting in `Metrics::nested_associative_operand_count` is zero at default knobs. |
| `constant-fold` | + Constant folding at intern time. Identity drops (`x + 0 → x`, `x * 1 → x`, `x & all_ones → x`, `x \| 0 → x`, `x ^ 0 → x`, `x - 0 → x`, `x << 0 → x`, `x >> 0 → x`); absorbing constants now fire on mixed operands too (`x * 0 → 0`, `x & 0 → 0`, `x \| all_ones → all_ones`) because post-construction compaction removes dead gate operands; mixed associative constants are aggregated (`1 + x + 1 → 2 + x`, `3 * x * 5 → 15 * x`); full all-constant evaluation for associative ops (bitwise AND/OR/XOR over values, sum/product mod 2^width) and 2-arity non-commutative ops (`Sub(c1, c2)`, `Shl(c1, c2)`, `Shr(c1, c2)` all with mod-2^width semantics and over-shift → 0). The settled graph also gets a late mixed-constant cleanup pass after remap-heavy normalization for the same doctrine. Fires counted in `Metrics::fold_identities_applied`. |
| `peephole`    | + Local rewrites at intern time. Single-gate involutions: `Not(Not(x)) → x`. Cross-gate comparison inversions: `Not(Eq) → Neq`, `Not(Neq) → Eq`, `Not(Lt) → Ge`, `Not(Gt) → Le`, `Not(Le) → Gt`, `Not(Ge) → Lt`. Unsigned comparison-boundary tautologies: `x < 0`, `x >= 0`, `x <= all_ones`, `x > all_ones`, plus the symmetric lhs-constant forms. Constant-selector mux collapse: `Mux(0, a, b) → b`, `Mux(1, a, b) → a`. Constant evaluation (all-operand-constants → evaluated constant): comparisons (`Eq`/`Neq`/`Lt`/`Gt`/`Le`/`Ge`), `Not(c) → ~c & mask`, `Slice(hi, lo)(c) → (c >> lo) & mask`, `Concat([c1, c2, ...]) → MSB-first bit assembly`, `RedAnd(c) → (c == all_ones)`, `RedOr(c) → (c != 0)`, `RedXor(c) → popcount(c) & 1`. Structural identities: full-width `Slice(hi, 0)` where `hi+1 == src_width` returns the source, single-operand `Concat([x]) → x`. In every case the inner gate may be orphaned by the rewrite; the post-construction `compact_node_ids` pass cleans it up. Fires counted in `Metrics::peephole_rewrites_applied`; removed nodes in `Metrics::nodes_compacted`. Broader cross-gate rewrites like `(a + b) - b → a` still await the e-graph layer. |
| `e-graph`    | Bounded semantic equivalence fragment. Under `identity_mode = node-id`, small-support combinational cones proven equal over the same canonical leaf endpoints collapse post-construction. Full semantic equivalence remains open. |

**Important scope note:** ANVIL also has an **always-on
generator-side proof** for obviously-constant unsigned comparisons.
That proof runs before interning and is used even under
`identity_mode = relaxed` and low factorization rungs. It exists to
keep emitted RTL cleaner in downstream tools, not to change the meaning
of the user-visible factorization ladder.

**Effective level:** `Config::effective_factorization_level()` /
`Module::effective_factorization_level()` apply the coarse mode
first, then the ladder:

- `identity_mode = relaxed` forces the effective level to `none`
  regardless of the requested rung.
- `identity_mode = node-id` uses
  `FactorizationLevel::effective()`, which returns the highest
  *implemented* layer at or below the requested one, walking the
  enum order top-down.

Every current rung is implemented. `FactorizationLevel::effective()`
still walks the ladder defensively so future aspirational rungs can be
added without lying to the generator: a request for an unimplemented
future rung will drop to the nearest implemented one below it, while
`e-graph` activates everything currently live — today the bounded
semantic gate-sharing fragment plus every lower rung. When deeper
layers land, the same `--factorization-level e-graph` invocation
automatically gains them — no config change required.

**Where enforced:** `src/ir/types.rs` `intern_gate` /
`intern_constant` gate commutative sort and dedup-bypass on
`self.effective_factorization_level()`. `src/gen/cone.rs`
`violates_anti_collapse` gates operand-uniqueness checks the
same way.

**Knob interactions:** `max_ast_instances`,
`operand_duplication_rate`, and `mux_arm_duplication_rate`
remain fine-grained overrides at their active levels. For
example, at level `operand-unique` with
`operand_duplication_rate = 1.0`, Add/Mul duplicates pass
anyway (the rate overrides the level's default). The level is
the coarse dial; the rates are fine tuning.

---

## 21 — AST-instance cap (construction-time CSE)

**Rule:** Each unique AST — `(op, operands, width)` for gates, or
`(width, value)` for constants — may be materialised as a named
node at most `max_ast_instances` times per module (default **1**,
strict uniqueness). Callers that would create an `N+1`th instance
of the same AST are routed to the most-recent existing node
instead. Knob: `Config::max_ast_instances` /
`--max-ast-instances`.

**Motivation:** without dedup, a sub-expression like
`slice_17 == 2'h2` can be computed dozens of times under different
wire names (`eq_4`, `eq_9`, `eq_10`, …) — every copy describes the
same circuit but uses wire capacity and obscures the module's
structure. Under strict uniqueness (N = 1), **one RHS drives one
signal**; downstream consumers reference that single node by name.
Higher N is useful for driving synthesis tools with duplicated
nets (cover more patterns) without abandoning the rule entirely.

**Construction-time enforcement:** `Module::intern_gate` and
`Module::intern_constant` own the cap. They maintain a per-key
instance vector; on call, create a new node when
`vec.len() < max_ast_instances`, otherwise return the last-created
instance. All generator helpers (`make_constant`, `make_eq_const`,
`make_mux`, `make_and`, `make_mul`, `make_nary_add`,
`replicate_to_width`, `or_reduce_terms`, `make_width_adapter`, the
`deliver` path in the interleaved frame machine, the gate-creation
block in `grow_pool_one_unit`) route through `intern_*` rather
than pushing `Node::Gate` / `Node::Constant` directly.

**Snapshot / rollback interaction:** `build_cone_with_retry`
rolls back `m.nodes`, `m.flops`, pool, and worklist on an empty-dep
retry. The dedup tables (`gate_instances`, `const_instances`)
must be snapshotted and restored alongside, or a stale entry would
point at a truncated `NodeId` and a later call would silently
return a now-different node (e.g. a Constant that was overwritten
by an Eq at the same `NodeId`).

**Knob semantics:**
- `max_ast_instances = 1` (default): strict CSE, no duplicates.
- `max_ast_instances = K` (K > 1): up to K copies of the same AST.
- `max_ast_instances = u32::MAX`: effectively disables dedup.

---

## 20 — Dep-bearing source required at elaboration-sensitive positions

**Rule:** At positions where a dep-empty (constant) source would
cause the surrounding logic to fold at elaboration time, the
terminal picker must return a node whose dep-set is non-empty —
i.e., the node is transitively driven by a primary input or a flop
Q. The positions covered today are:

- **Mux select.** The `sel` operand of a one-hot or encoded mux
  (comb or flop). A constant select makes the mux degenerate to
  one fixed arm.
- **Priority-encoder request bits.** Each `req_i` in
  `req_0 ? 0 : req_1 ? 1 : … : 0`. Constant request bits degrade
  the block to a fixed value.
- **Const-comparand LHS.** The signal side of `lhs == K` /
  `lhs < K` / …. A constant LHS folds the entire comparison to a
  single bit at elaboration.
- **Const-shift-amount value.** The value side of `v << K`. A
  constant value folds the shift to a literal.

**Motivation:** a signal generated "without a proper reason" is,
in practice, a signal that does not carry a dependency on any
primary input or flop Q. Every downstream consumer of such a
signal reduces to a literal at elaboration, and the generator has
spent work producing nothing observable. The rule is a concrete
restatement of the "signals must have a reason to exist" principle
applied to the specific positions where elaboration-time folding
is the failure mode.

**Construction-time enforcement:**
`pick_terminal_dep_bearing(g, m, pool, width, exclude)` is the
picker used at these positions. It admits only:
1. A randomly-chosen dep-bearing matching-width pool entry, or
2. A width-adapter (Slice / replicating Concat) from the widest
   dep-bearing pool entry of any width.

Tiers 2 (any matching-width) and 4 (fresh constant) of the general
`pick_terminal` are excluded. The picker panics if the pool has no
dep-bearing entry at all — an invariant violation, since primary
inputs are always seeded with non-empty deps at module start.

**Scope (today):** the pool-mode dispatch paths
(`graph-first` / `pool_only` helpers) — where the defect was
directly observed. The recursive and interleaved paths construct
selects via `build_cone` and rely on depth budget to spawn
real logic; they do not currently terminate select sub-cones on a
constant except at depth-exhaustion edge cases. If those paths
start producing folded selects, the fix is to thread a
`require_deps` flag through `build_cone` recursion.

---

## 19 — Coefficient fits operand width

**Rule:** Every constant coefficient drawn by the linear-combination
motif (`y = Σ sᵢ·cᵢ` / `y = s₁·c₁ − …` / `y = c · s₁ · s₂ · …`)
must satisfy `1 ≤ c ≤ 2^W − 1`, where `W` is the operand width at
which the `Constant` node is emitted. The coefficient range
`[min_coefficient, max_coefficient]` is clamped down to fit `W` at
the point of generation, not at validation time.

**Motivation:** a literal whose value does not fit its declared
width is either truncated by the emitter (`1'h6` → `1'h0`, the real
observed bug in sample output) or expanded silently — either way
the generated constant no longer means what the coefficient motif
says it means. A constant that does not fit is a signal created
for no reason: it has no semantic value beyond "something the RNG
happened to draw." The rule forces coefficients to be drawn
*knowing* the target width.

**Construction-time enforcement:** `pick_coefficient(g, width)`
owns the clamping. It narrows the draw range to
`[max(min_coefficient, 1), min(max_coefficient, 2^W − 1)]` and
never returns a value outside it. The validator does not need a
corresponding check because the picker cannot emit an ill-sized
coefficient.

**Edge case — width = 1.** The only legal coefficient is 1.
Add/Sub collapse to `y = s₁ ± s₂ ± …` (coefficient-free); Mul at
`c = 1` already forces `n ≥ 2` via
`pick_mul_coefficient_and_arity`, so `y = s₁ · s₂ · …` is the only
shape.

---

## 18 — No orphan gates

**Rule:** Every `Node::Gate` in a finalised `Module` has at least
one consumer: an output drive-root, a flop's D input, a flop's Q
read by a live gate, or an operand of another live gate. Gates
that would be orphaned by a rejection path must never make it
into `m.nodes`.

**Motivation:** a signal with no reader serves no purpose. It is
an artifact of speculative construction, not a real piece of the
circuit. Orphaned gates bloat the emitted output, confuse readers,
and indicate the generator did work that wasn't driven by demand.

**Construction-time enforcement (α — adopted):**

1. **`build_cone` snapshot / rollback.** The recursive path
   (`sequential` / `shuffled`) snapshots `m.nodes.len()`,
   `m.flops.len()`, `pool`, `worklist`, `gate_instances`, and
   `const_instances` before building a gate's operand sub-trees.
   On anti-collapse rejection the snapshot is restored and
   `pick_terminal` provides a safe fallback. The operand sub-trees
   built for the rejected gate vanish from the IR — no orphans
   leak.

2. **`process_signal_frame` existing-operand fallback.** The
   interleaved frame machine cannot snapshot per-gate because
   sibling sub-frames have already committed. On anti-collapse
   rejection it delivers one of the gate's *existing operand
   NodeIds* as the fallback instead of creating a new node via
   `pick_terminal`. Idempotent / self-inverse / comparison
   collapses have all operands sharing a NodeId; `mux(s, a, a)`
   uses `operands[1]`. No new node is created; existing operands
   remain consumed.

3. **`GraphFirst` retired.** The strategy's phase-1 speculative
   pool growth was the root cause — units were created before any
   consumer existed. The variant is retained as a silent alias
   for `Interleaved` for CLI / config backward compatibility;
   the speculative code is unreachable.

**Safety-net audit.** `generate_leaf_module` runs
`count_orphan_gates` after flop drain and emits
`tracing::warn!` if any Gate lacks a consumer. In the current
implementation across 4 strategies × many seeds the audit
count is consistently zero; the warning exists to catch future
regressions.

**Emitter is dumb.** The emitter does not filter. Per doctrine,
by the time the emitter runs it is too late to roll back —
rules must be enforced at IR construction.

---

## 17 — Priority-encoder block

**Rule:** A priority-encoder block takes N 1-bit request signals and
emits a `ceil(log2(N))`-bit output that is the index of the highest-
priority asserted request (lowest-indexed by convention). Emitted as
a chained ternary:
`y = req_0 ? 0 : req_1 ? 1 : ... : req_{N-1} ? N-1 : 0`.

- **N** is drawn from `[min_mux_arms, max_mux_arms]` *constrained* to
  values where `ceil_log2(N) == target_width`. For target width 1:
  N = 2. For target width W >= 2: N ∈ [2^(W-1)+1, 2^W].
- If no valid N exists in the arity range for the current target
  width, the block dispatch is skipped and the generator falls
  through to the usual operator-gate path.
- Fall-through (no request asserted): output = 0. This is a design
  convention — in real RTL, priority encoders typically also emit a
  "valid" flag; `anvil` omits that today.

**Block, not operator:** ports are N request inputs + one
log-width output. N is a port count, not arity. Emitted as a
compound gate tree — each priority level is a Mux node, chained
left-to-right from highest-index fall-through up to index-0-wins.

**Where enforced:** `src/gen/cone.rs` —
`pick_priority_encoder_n` (applicability check),
`assemble_priority_encoder` (chained-ternary assembly),
`build_priority_encoder_recursive` / `build_priority_encoder_pool`
(dispatch for the three construction strategies).

---

## 16 — Procedural combinational case-mux block

**Rule:** A procedural case mux is a *block* with one encoded select
bus plus M data inputs (all width W). It emits a synthesizable
`always_comb` block:

- `case (sel)`
- one arm per explicit index `0 .. M-1`
- explicit `default: out = 0`

So when `sel` is out of range (for non-power-of-two M), the output is
0 by construction.

- **M** is drawn from `[max(2, min_mux_arms), max_mux_arms]`.
- **Select width** is `ceil(log2(M))`.
- **Output width** is the caller's target width W.

This is intentionally a syntax-surface motif distinct from the
expression-level encoded comb mux. The downstream logic function is the
same kind of indexed mux, but the emitted RTL goes through a different
frontend/elaboration path because it uses `always_comb case`.

**Where enforced:** `src/gen/cone.rs` —
`build_case_mux_recursive`, `build_case_mux_pool_only`,
`make_case_mux`. Validator: `check_gate_shape`'s `CaseMux` arm.
Emitter: `src/emit/sv.rs` declares the target as `logic` and emits one
`always_comb begin ... case (...) ... endcase end` block per case mux.

---

## 16a — Procedural combinational casez-mux block

**Rule:** A procedural casez mux is a *block* with one encoded select
bus plus M data inputs (all width W). It emits a synthesizable
`always_comb` block:

- `casez (sel)`
- one arm per generated wildcard pattern
- explicit `default: out = 0`

The wildcard patterns are generated **non-overlapping by construction**,
so the block behaves as a wildcarded indexed mux rather than an
accidental priority chain.

- **M** is drawn from `[max(2, min_mux_arms), max_mux_arms]`.
- **Select width** is `ceil(log2(M)) + 1`.
- **Pattern form** today is a unique encoded prefix plus one wildcard
  low bit, which renders as literals like `3'b01?`.
- **Output width** is the caller's target width W.

This is intentionally a syntax-surface motif distinct from both the
expression-level encoded comb mux and the plain indexed `case` block.
The downstream logic family is still "indexed mux with a default", but
the emitted RTL goes through the `casez` frontend/elaboration path and
therefore exercises a different parser / elaborator surface.

**Where enforced:** `src/gen/cone.rs` —
`build_casez_mux_recursive`, `build_casez_mux_pool_only`,
`make_casez_mux`, `build_casez_patterns`. Validator:
`check_gate_shape`'s `CasezMux` arm. Emitter:
`src/emit/sv.rs` declares the target as `logic` and emits one
`always_comb begin ... casez (...) ... endcase end` block per casez
mux.

---

## 16b — Procedural bounded for-fold block

**Rule:** A bounded `for` fold is a *block* with one packed source bus
plus a fixed fold kind. It emits a synthesizable `always_comb` block:

- initialize an accumulator of width `W`
- `for (int i = 0; i < N; i++)`
- fold `src[(i * W) +: W]` into the accumulator

The block is intentionally procedural. The goal is to exercise the
frontend/elaboration surface for statically bounded loops, not merely to
construct an equivalent expression tree and hope the emitter happens to
print it that way.

- **Fold kind** today is one of `xor`, `or`, `and`, `add`.
- **Trip count N** is drawn from `[max(2, min_gate_arity), max_gate_arity]`.
- **Chunk width W** is the caller's target width.
- **Packed source width** is exactly `N * W`.
- The generated trip count is further constrained so the packed source
  stays in the current small exact-evaluation comfort zone during
  generation (`N * W <= 128` today).

This is a structured combinational block distinct from both the
expression-level operator family and the case/casez surfaces.

**Where enforced:** `src/gen/cone.rs` —
`build_for_fold_recursive`, `build_for_fold_pool_only`,
`make_for_fold`, `pick_for_fold_trip_count`, `pick_for_fold_kind`.
Validator: `check_gate_shape`'s `ForFold` arm. Emitter:
`src/emit/sv.rs` declares the target as `logic` and emits one
`always_comb begin ... for (int i = 0; i < N; i++) ... end end` block
per for-fold node. Exact evaluator: `src/ir/compact.rs` evaluates the
same packed-chunk fold semantics under assignment.

---

## 16c — Selectable Slice and Concat surfaces

**Rule:** `Slice` and `Concat` are real selectable structured operators,
not helper-only shapes. They must also be emitted in forms that survive
the settled graph as genuine surface area rather than collapsing
immediately into peephole identities.

### Selectable `Slice`

- `Slice { hi, lo }` remains 1-operand.
- The output width is `hi - lo + 1`.
- The generated source width must be **strictly greater than `hi`**.
  This keeps the selectable shape from degenerating into the full-width
  slice identity.

### Selectable `Concat`

- `Concat` remains variadic.
- The generated operand widths must sum exactly to the output width.
- The selectable form must use **at least 2 operands** so it cannot
  collapse into the single-operand concat identity.

The old width-adapter / block-assembly helpers still use `Slice` and
`Concat` too, but the important doctrinal change is that the generator
can now pick them directly as surface-carrying gates.

**Where enforced:** `src/gen/cone.rs` —
`pick_structured_gate`, `pick_slice_gate`, `pick_concat_operand_widths`,
and `input_widths_for`. Validator: existing `Slice` / `Concat` shape
checks in `check_gate_shape`. Emitter: existing `render_gate` paths in
`src/emit/sv.rs`.

---

## 15 — M-to-1 combinational mux block

**Rule:** A combinational mux is a *block* (not an operator) with
ports: M data inputs (width W) plus a select. Two encoding styles
are supported, chosen per-mux via `comb_mux_encoding_prob`:

- **OneHot:** M independent 1-bit select signals, one per data arm.
  D = `OR_i({W{sel_i}} & data_i)`. When no select asserts, D = 0.
- **Encoded:** a single `ceil(log2(M))`-bit select bus.
  D = `(sel==0)? data_0 : (sel==1)? data_1 : ... : (sel==M-1)? data_{M-1} : 0`.
  When M is not a power of 2 and sel is out of range, D = 0.

M is drawn from `[max(2, min_mux_arms), max_mux_arms]`. M = 1 is
excluded (Rule 7 rationale). M = 0 is excluded for combinational
muxes — unlike flops where M = 0 means "direct-D with no mux", a
combinational mux has no fall-back semantic without state.

**No Q-feedback axis:** combinational muxes have no state, so there
is no `QFeedback` kind. Any Q-feedback-style pattern arises only
when a flop's D-cone contains a comb mux whose inputs reference the
flop's Q — which is permitted freely by Rule 2.

**Block, not operator:** the Mux has ports (data_0..data_{M-1}, sel),
not arity. M is a port count parameter, not an N-arity. The per-mux
encoding choice (OneHot vs Encoded) and the per-mux M are structural
parameters enumerating *motif shapes* — see the "Operators vs blocks"
preamble.

**Where enforced:** `src/gen/cone.rs` — `build_comb_mux`,
`build_comb_mux_one_hot`, `build_comb_mux_encoded`. The same helpers
`replicate_to_width`, `make_and`, `or_reduce_terms`, `make_eq_const`,
`make_mux`, `make_constant` as the flop D-mux path, minus the
Q-feedback terms. Validator: `check_gate_shape` verifies each
emitted primitive (And, Or, Eq, Mux, Concat) independently.

---

## 14 — Operator N-arity for associative operators

**Rule:** Operators that are algebraically associative
(`And`, `Or`, `Xor`, `Add`, `Mul`) are emitted with a random arity
N drawn from `[cfg.min_gate_arity, cfg.max_gate_arity]` (inclusive),
N ≥ 2. All N operands share the same width as the output.

- `And`, `Or`, `Xor` — bitwise logic, associative and commutative.
  `a & b & c` is a 3-input AND that the synthesizer interprets as a
  single 3-input gate (or a 2-gate tree — the choice is the
  synthesizer's, not the generator's).
- `Add`, `Mul` — associative arithmetic. `a + b + c` is a legal SV
  expression; operands truncate to the declared result width.

**Sub is excluded:** subtraction is not associative
(`(a - b) - c ≠ a - (b - c)`). Sub stays strictly 2-arity in the IR.
Chains like `a - b - c` in emitted output come from cascading
separate 2-arity Sub nodes, not from a single N-arity Sub node.

**Other operators stay at their natural arity:**

- `Not` — 1 operand (unary inversion).
- `Mux` — a *block*, not an operator. The IR's `GateOp::Mux` is a
  2:1 mux (3 operands: `[sel, a, b]`), the smallest mux primitive.
  Arity is not the right word for a mux; a mux has **ports** (1
  select + M data). The general M-to-1 mux is a block with
  structural parameters (M data ports + select encoding). It is
  not yet a first-class combinational motif in the IR; M-to-1 muxes
  today exist only as compound gate trees inside flop D-inputs (see
  Rules 2–3).
- `Eq / Neq / Lt / Gt / Le / Ge` — 2 operands. Comparisons are not
  associative (`a == b == c` means `(a == b) == c`, which compares a
  1-bit result against `c` — not a meaningful generalization).
- `RedAnd / RedOr / RedXor` — 1 operand by definition (unary
  reduction over all bits).
- `Shl / Shr` — 2 operands (value + shift amount).
- `Slice` — 1 operand (one bit range on one source).
- `Concat` — variadic (already N-arity by construction; each operand
  contributes its own width, unlike the N-arity associative ops
  where all operands share the output width).

**Where enforced:** `src/gen/cone.rs` — `input_widths_for` picks
`N = rand(min_gate_arity..=max_gate_arity)` for the associative
ops and returns `vec![out_w; N]`. `src/emit/sv.rs` — `render_gate`
joins operands with the infix symbol for associative ops. Validator:
`ir::validate::check_gate_shape` accepts `operands.len() >= 2` for
the associative ops, exactly 2 for `Sub`.

**Why this is an operator rule, not a block rule:** N-arity for
associative operators is a width-preserving, algebra-driven
generalization: adding more operands of the same width is trivially
equivalent to a 2-input tree. Blocks (mux, flop, memory) have
structural parameters that are not reducible to "how many operands";
they get their own rules (Rules 2, 3, 5, 7, and future additions).

---

## Future rules

As `anvil` grows, this catalog will too. Expected additions:

- **Phase 3 (structured ops):** any later structured combinational
  motifs beyond the already-landed case/casez/for-fold/selectable-
  Slice/selectable-Concat surfaces.
- **Phase 4 (hierarchy):** deeper helper-instance placement rules and
  future hierarchy-aware identity/factorization beyond the already-live
  design-level rules for unique module names, instance-boundary port
  matching, and acyclic hierarchy.
- **Phase 5 (parameterization):** parameter-dependent width
  propagation, parameter range enforcement.
- **Phase 6 (advanced):** memory inferrable patterns, multi-clock
  CDC-safety.

Every new rule lands here as it becomes a generator invariant.

## Cross-reference

Live docs that point to this catalog:

- `DEVELOPMENT_NOTES.md` — core design decisions link here instead of
  restating the rules inline.
- `CODEBASE_ANALYSIS.md` — the invariants-enforced section links here
  for the full list.
- `book/src/sequential.md` — rules 2, 3, 5, 7 apply.
- `book/src/sharing.md` — rule 1 applies.
- `book/src/by-construction.md` — the philosophy; this chapter is
  the concrete embodiment.
