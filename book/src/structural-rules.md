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

The rules below are grouped by what they govern:

- **Combinational integrity:** Rules 1, 8, 10, 14 — no combinational
  loops; anti-collapse; per-gate width; operator N-arity.
- **Block: flop:** Rules 2, 3, 4, 5, 6, 7 — Q-feedback, mux-term,
  clk/rst_n visibility, single-clock discipline, single-drive, M
  arity exclusions.
- **Block: mux (combinational, future):** placeholder — will land
  as its own rule family when M-to-1 combinational muxes become a
  first-class motif (today, M-to-1 muxes exist only as compound gate
  trees inside flop D-inputs; see Rules 2–3).
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
