# Development Notes
Engineering rationale behind design decisions. The "why" that does not belong in code comments and is too detailed for `MEMORY.md`.

For the canonical statement of the algorithm and load-bearing decisions, see `book/src/`. This file is the contributor-facing scratchpad: rejected alternatives, calibration notes, gotchas, and the reasoning behind small choices the book does not cover.

---

## Core design decisions (recap)

These are documented in detail in the mdBook. They are restated here only as anchors:

- **Recursion is the core principle.** Every non-trivial generation step is a recursive descent over the typed circuit graph. Iteration is the exception, used only where termination or ordering genuinely require it (e.g., the flop worklist drainer, the per-output driver loop). When in doubt, recurse. See `book/src/core-idea.md` "The single guiding principle".
- **Synchronous-design discipline.** Every module is fully synchronous to a single clock domain: one `clk` (posedge), one `rst_n` (async, active-low), every flop emitted into one `always_ff` block. Enforced by construction — there is no IR field for per-flop clock or per-flop reset polarity. See `book/src/sequential.md` "Synchronous-design discipline".
- **Flop-D mux motifs.** Every flop's D input is constructed from one of: M=0 (direct cone), M≥2 OneHot (OR-of-masked arms), M≥2 Encoded (chained ternary over `Eq(sel, k)`). M=1 is excluded by design; it collapses to a wire. The style (OneHot vs Encoded) and kind (ZeroDefault vs QFeedback) are chosen per-flop and orthogonal — four motif variants plus the M=0 plain register. See `book/src/sequential.md` "Flop motifs".
- **Q-feedback freedom (revised).** A flop's own Q may appear freely — any number of times — as a leaf in any of its data, select, or direct-D sub-cones. The clock edge breaks the Q→D loop temporally; this is the standard synchronous feedback pattern (counters, accumulators, state machines). Independently, `FlopKind::QFeedback` adds an explicit Q fall-through term in the mux when no select fires. Both are legal; both can be active at the same flop. Combinational self-reference (Rule 1) is still forbidden. See `book/src/structural-rules.md` Rules 2 and 3.
- **Structural rules catalog.** Every load-bearing generator invariant is documented in `book/src/structural-rules.md`. That chapter is the durable source of truth — new rules land there as they become invariants. Inline design-decision recaps in this file should *point* to the catalog, not duplicate rule text.
- **Operators vs blocks.** Load-bearing conceptual distinction. An operator is an associative primitive function; its generalization is **arity** (N same-width operands). A block is a functional unit with internal structure; its generalization is **ports / port counts / arms**, encoding choices, feedback topology. Arity is operator vocabulary only — blocks have ports, not arity. `And / Or / Xor / Add / Mul` are operators and got N-arity in `2026-04-15-0015`. `Sub` is not associative and stays 2-arity. `Mux` and `Flop` are blocks and are governed by block rules, not arity knobs. See `book/src/structural-rules.md` "Operators vs blocks" preamble and Rule 14.
- **Roles of constants in RTL.** Integer literals appear as operands with three *distinct* semantic roles: **coefficient** (multiplicative weight in arithmetic linear combinations; per-op constraints: Add `ci ≠ 0`, Sub `ci > 0` strictly positive, Mul TBD), **shift amount** (structural parameter of `Shl/Shr` — `a << 2`; constant-amount vs variable-amount are both legal, with real designs biased heavily toward constant), and **comparand** (threshold / sentinel on the RHS of a comparison — `a == 7`; additive to signal-vs-signal comparisons, not a replacement). These three are *not interchangeable*: each has its own motif family, its own constraints, and its own knob(s). Do not unify them under a single `constant_prob` knob — doing so loses the semantic distinctions. See `book/src/structural-rules.md` "Roles of constants in RTL".
- **Construction strategies.** Four named strategies for constructing a module's internal logic, selectable per-run: `sequential` (current; build cones per-output in declaration order), `shuffled` (per-output but in random permutation), `interleaved` (frames interleaved via random-pop work queue — cones grow in lockstep), `graph-first` (grow a gate pool with no per-output structure; pick drive-roots at the end — planned default). The strategy is a property of **how** the generator builds; the emitted SV is a DAG regardless. Different strategies produce different output *distributions* (declaration-order bias, within-module sharing symmetry). See `book/src/construction-strategies.md`. Planned implementation sequence: add the knob with only `sequential` accepted, then land `shuffled`, `interleaved`, `graph-first`; flip default to `graph-first` on landing.
- **Circuit IR over annotated EBNF.** The generator builds a typed circuit graph and emits SV from it. See `book/src/why-not-grammar.md`.
- **Generation by construction, not generate-then-filter.** Validity is structural; the validator is a safety net, not a gate. See `book/src/by-construction.md`.
- **Synthesizability is a subset constraint.** The gate set, flop pattern, and emitter cover only the synthesizable subset. There is no mode that emits non-synthesizable constructs. See `book/src/synthesizability.md`.
- **Non-triviality via dep-set tracking + structural anti-collapse rules.** No oracle. See `book/src/non-triviality.md`.
- **No oracle, no reference simulator.** `anvil` is a generator. Tool testing is downstream. See `book/src/non-goals.md`.

If you need to revise any of these, that is a deliberate task with its own commit and a `DEVELOPMENT_NOTES.md` entry.

---

## Calibration notes

### `constant_prob = 0.1`
Default chosen to prevent constants from dominating cone leaves. Real synthesis-stress workloads may want lower (≤ 0.05); aggressive pattern coverage may want higher. Revisit after first seed sweep with metrics on what fraction of generated cones survive non-triviality on the first attempt.

### `terminal_reuse_prob = 0.3`
Probability that, when a cone reaches a leaf decision and the signal pool has matching-width entries, it picks an existing pool entry rather than emitting a constant or recursing further. Higher = more sharing-like behavior even before Phase 3 explicitly turns on `share_prob`. Default is a guess; tune after Phase 1.

### `share_prob = 0.3` default
The non-leaf DAG-sharing fork is enabled by default at a modest rate. Every operand has a 30% chance of terminating at an existing pool entry rather than recursing. This is the Phase 2 guiding mode: cones are a mix of tree and DAG shapes, chosen per recursion point. Raise (0.5–0.9) for fanout-stress generation; lower (0.0–0.1) for wide-sprawling tree-ish cones. `share_prob = 0.0` does not produce *pure* trees — `pick_terminal` still reuses matching-width pool entries at forced leaves. The distinction is: `share_prob` controls *non-leaf* sharing; leaf-level reuse is always on.

### `gate_*_weight` defaults
3:2:1:1:1 (bitwise:arith:struct:compare:reduce). Bitwise dominates because bitwise gates are the most type-flexible and produce the widest cones. Comparisons are weighted lower because they collapse the width to 1, which limits downstream cone depth. These are gut-feel; replace with measurements when phase-1 sweeps land.

### `flop_mux_encoding_prob = 0.5`
Default chosen to give equal motif exposure to OneHot and Encoded styles across a random seed sweep. If post-synthesis metrics show that one style dominates as a bug-finding target, bias the default. The knob also allows users to run workloads stressing only one style for targeted testing.

### `flop_qfeedback_prob = 0.5`
Default 50/50. No empirical data yet. Real designs probably lean heavier on QFeedback (hold-on-no-write is far more common than zero-on-no-write), but generating the less-common pattern is precisely where random generation earns its keep. Revisit with data.

### QFeedback-in-Encoded: replace `data_0` with Q
Alternative considered: add Q as an extra (M+1)th entry encoded with the largest select value. **Rejected** because:
- It would require the sel bus to be one bit wider than `ceil(log2(M))` whenever M is a power of 2, breaking the clean "M mux entries ⇔ `ceil(log2(M))`-bit sel" invariant.
- The "slot 0 is Q" convention mirrors common RTL idioms where the zero-index / reset state is treated specially.
- It keeps M as the single knob for mux entry count across both styles.

---

## Rejected alternatives

### Annotated-EBNF runtime engine
Considered: a generic attribute-grammar interpreter that reads an annotated SV grammar at runtime and produces output. **Rejected** because:
- SV's grammar is enormous; encoding all of it is months of work for productions we will never emit.
- Threading mutable scope/driven-set/flop-worklist state through pure inherited/synthesized attributes is awkward; it really wants `&mut Context`.
- Extending the grammar engine for a new motif is comparable in effort to adding a Rust enum variant + emitter arm, with much worse error messages.

The grammar view is preserved as a *correctness argument* (every constructor preserves invariants ⇔ every production is valid under its attributes). Not as a runtime artifact.

### Oracle / reference simulator
Considered: a Rust evaluator that walks the IR with concrete input vectors and produces expected output values, used both for non-triviality filtering and for downstream tool testing. **Rejected** because:
- Doubles implementation effort.
- Introduces a second correctness question (is our interpreter LRM-correct?).
- The user's stated goal is *generation*, not *tool testing*.
- Non-triviality is cheaper to enforce by dep-set tracking + structural rules; multi-vector evaluation is overkill for that use case.

Users who want differential testing can run Verilator/Icarus/Yosys against the output; that is downstream work, not `anvil`'s job.

### `always_comb` + `case` for encoded-mux flop D

Considered for the Encoded-style flop D: emit an `always_comb` block with a `case (sel)` statement driving D. **Rejected** in favor of a chained ternary over `Eq(sel, k)` because:

- The emitter already handles `Mux` and `Eq` as ordinary `GateOp` variants; nothing new is required.
- `case` would require introducing procedural block emission (`always_comb`) and name-binding for the case target, which is a bigger scope than a uniform expression-level SV emitter.
- Synthesis tools produce the same netlist from both forms for well-formed one-cycle muxes; the readability difference only matters to a human reader.

If a future motif (e.g., FSM state encoding) genuinely requires `case`, revisit then.

### M = 1 mux arm

Excluded from `pick_mux_arm_count` by design. A 1-arm mux is algebraically `sel ? data_0 : 0` (ZeroDefault) or `sel ? data_0 : Q` (QFeedback) — in either case a trivially-simplified shape that adds no motif diversity over what a simple 2-arm mux or an M=0 direct cone already covers. Allowing M=1 would bloat the generator's decision space without expanding the generated-SV distribution meaningfully.

### `#![allow(clippy::too_many_arguments)]` in `src/gen/cone.rs`

The cone-recursion helpers legitimately thread 5–8 context references (`Generator`, `Module`, `SignalPool`, `FlopWorklist`, `width`, `depth`, `exclude`, sometimes more). Packaging them into a `Ctx` struct would help readability but also forces mutable-borrow juggling that fragments the code with no semantic benefit. The lint is silenced at the module level rather than per-function to avoid the ceremony of annotating every helper. Not recommended for modules outside `gen/cone.rs`.

### Generate-then-validate (filter loop)
Considered: emit random IR with looser invariants, then run the validator and discard rejected outputs. **Rejected** because:
- Untestable bound on generation time.
- Tempts contributors to weaken constructors and rely on the validator, leading to silent correctness drift.
- Complex invariants (dep-set non-emptiness) are far more expensive to check post-hoc than to maintain incrementally.

The bounded retry in `cone::build_cone_with_retry` is the *only* exception — it exists because dep-set non-emptiness depends on terminal selection in a way that cannot always be predicted at the gate level (e.g., when all available pool entries happen to be constants). Retry budget is small (4) and falls back to accepting the last attempt.

---

## Implementation gotchas

### Reproducibility hazards
- `HashMap` iteration order is *not* stable across builds. If iteration order ever affects output, switch to `BTreeMap` or sort the keys explicitly. The current code avoids this; new contributions must too.
- `f64` non-associativity is fine for probability comparisons but never use `f64` arithmetic to compute IR fields — only RNG-driven discrete choices.
- `rand::thread_rng()` is forbidden everywhere. All randomness flows from the seeded `ChaCha8Rng` in the `Generator`.

### IR arena indexing
`NodeId` is `u32`. We use `Vec<Node>` indexed by `u32`. This is fine for the foreseeable size range (modules of ≤ 10⁶ nodes). If we ever need more, the change is local to `ir/types.rs`.

Indices are stable for the lifetime of a `Module` because we only ever push, never remove. The bounded retry in `cone::build_cone_with_retry` rewinds by `Vec::truncate`, which is safe because no other code holds `NodeId`s referring to the rewound region.

### Width 0 is illegal
`Config::validate` requires `min_width >= 1`. Width-0 signals are not synthesizable and SV does not allow them. Do not relax this.

### 128-bit constant cap
Constants fit in `u128`. Modules with `max_width > 128` are technically allowed, but the constant generator emits `0` for any width ≥ 128. This is a deliberate simplification; widening the constant representation is straightforward when needed.

---

## Testing strategy notes

- **Unit tests** live in each module under `#[cfg(test)] mod tests`. Test IR constructors enforce invariants; test gate width rules; test dep-set propagation; test the emitter on hand-built IRs.
- **Integration tests** in `tests/`: cross-seed generation + IR validation + reproducibility.
- **External smoke tests** (Verilator lint, Yosys synth) are gated by env vars so they are skippable for developers without those tools. CI must enable them.

A failed external smoke test is always a generator bug. Do not "fix" by tweaking generator output — find the root invariant violation and fix it.

Same principle for the IR validator (`src/ir/validate.rs`): if it rejects real generator output, that's a generator bug. The validator is an active safety net, not a gate to be worked around. The per-gate arity + width checker added in slice `2026-04-15-0008` is specifically designed to catch width bugs in the new flop-mux assembly code, where gates are constructed by hand rather than by recursion — the most likely place for a width-arithmetic slip.

---

## File-level conventions

- Every Rust source file starts with a doc comment explaining its scope.
- Public types in `ir/types.rs` and `config.rs` get full doc comments. Internal helpers do not need them.
- No multi-paragraph docstrings. One short line; if more is needed, link to `book/`.
- No comments explaining *what* the code does; only *why* when non-obvious.
