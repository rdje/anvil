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

- `x ^ x` → 0
- `x - x` → 0
- `x == x` → 1
- `x != x` → 0
- `mux(s, a, a)` → a (trivial)

When a forbidden shape is detected, the generator falls back to
`pick_terminal` rather than emitting the degenerate gate.

**Where enforced:** `src/gen/cone.rs` — `violates_anti_collapse` in
`build_cone`.

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

**Rule:** The gate set, the flop pattern, and the emitter cover only
the synthesizable subset of SystemVerilog. No `initial`, no delays
(`#N`), no `$display` / `$finish`, no dynamic arrays, no classes, no
events, no tasks, no fork/join, no `wait`, no latches (`always_comb`
is not emitted for datapaths — `assign` is used instead and always
fully defines its target).

**Where enforced:** the IR has no node kinds for these constructs;
the emitter has no code path that produces them. Absence is
structural.

---

## 12 — Deterministic naming

**Rule:** Generated names are deterministic, uniform, and collision-free
within a module.

| Pattern   | Meaning                                        |
|-----------|------------------------------------------------|
| `clk`     | Clock input (emitted only when module has flops) |
| `rst_n`   | Async active-low reset (emitted only with flops) |
| `i_N`     | Primary data input, `N` counts from 0          |
| `o_N`     | Primary output, `N` counts from 0              |
| `w_N`     | Internal gate wire, `N` is the `NodeId`        |
| `r_N`     | Flop register, `N` is the `FlopId`             |

Same `(seed, knobs)` → byte-identical names.

**Where enforced:** `src/emit/sv.rs` — `node_ref`, wire-declaration
loop, flop-declaration loop.

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

- **Phase 3 (structured ops):** width rules for case/casez, priority
  encoders, for-loop unrolling.
- **Phase 4 (hierarchy):** naming uniqueness across sub-modules,
  port-width matching at instance boundaries, acyclic hierarchy.
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
