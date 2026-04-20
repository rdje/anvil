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
- **Construction strategies.** Three live strategies construct a
  module's internal logic: `sequential` (per-output cone recursion in
  declaration order), `shuffled` (same, randomised output order), and
  `interleaved` (frames interleaved via random-pop work queue — cones
  grow in lockstep). `graph-first` remains as a deprecated CLI/config
  alias for `interleaved`; the original speculative pool-growth
  implementation is retired. The strategy is a property of **how** the
  generator builds; the emitted SV is a DAG regardless. Different
  strategies produce different output *distributions*
  (declaration-order bias, within-module sharing symmetry). See
  `book/src/construction-strategies.md`.
- **Circuit IR over annotated EBNF.** The generator builds a typed circuit graph and emits SV from it. See `book/src/why-not-grammar.md`.
- **Generation by construction, not generate-then-filter.** Validity is structural; the validator is a safety net, not a gate. See `book/src/by-construction.md`.
- **Synthesizability is a subset constraint.** The gate set, flop pattern, and emitter cover only the synthesizable subset. There is no mode that emits non-synthesizable constructs. See `book/src/synthesizability.md`.
- **Non-triviality via dep-set tracking + structural anti-collapse rules.** No oracle. See `book/src/non-triviality.md`.
- **Signoff-grade adversarial RTL is the product goal.** `anvil` is not
  trying to be merely "valid enough". The target is a signoff-level
  quality random synthesizable RTL generator whose outputs are clean in
  mainstream downstream tools by default and still adversarial enough to
  expose real bugs in them. Feature growth and clean-run robustness are
  both first-class; neither is optional garnish for the other.
- **No oracle, no reference simulator.** `anvil` is still a generator,
  not a bundled shadow simulator. The way it stresses downstream tools
  is by emitting high-quality legal RTL, not by embedding a second
  implementation of RTL semantics. See `book/src/non-goals.md`.

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
- The user's stated goal is *generation*, not building a full shadow
  simulator or tool-oracle inside `anvil`.
- Non-triviality is cheaper to enforce by dep-set tracking + structural rules; multi-vector evaluation is overkill for that use case.

That does **not** lower the output-quality bar. The generator is still
expected to emit modules that run cleanly in downstream tools.
Verilator / Yosys are external validators, not the place where
`anvil` gets to finish the job.

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

### Canonical state backreferences are validator-owned (2026-04-20)

Once `merge_equivalent_flops` started rewriting state after drain,
`Flop.id`, `Flop.q`, and `Node::FlopQ { flop, .. }` stopped being
"born correct and forgotten" fields. They are now recovery-critical
identity links that a bad renumbering pass can corrupt.

`ir::validate::validate` now owns that contract:

- every output drive root exists before root inspection;
- `m.flops[idx].id == idx`;
- `Flop.d`, `Flop.q`, and every `NodeId` stored inside `FlopMux`
  exist;
- `Flop.q` points at a `Node::FlopQ` whose backref and width match
  the owning flop; and
- every `Node::FlopQ` points at a real flop and is that flop's
  canonical `q` node.

Keep the emitter dumb. If any of these invariants fail, fix the
producer or rewrite pass; do not add emitter-side repair logic.

### Compaction now legitimises dynamic absorbing folds (2026-04-20)

Before `compact_node_ids`, the cautious rule for absorbing constants
was "only fold if the other operand is not a gate", because
`x & 0 -> 0`, `x | all_ones -> all_ones`, and `x * 0 -> 0` would
otherwise orphan a dynamic subgraph immediately.

That restriction is now obsolete. Finalisation already performs a
reachability compaction from real roots and rebuilds the dedup tables,
so these local identities are safe to fire regardless of whether the
other operand is a gate. In other words: once compaction exists, the
correctness risk is no longer "did we orphan something?" but "did we
miss an identity we should have collapsed?"

The practical consequence showed up in tool smoke:

- the remaining seed-42 Verilator `UNSIGNED` / `CMPCONST` warnings
  were not tool quirks;
- they were missed IR-local tautologies; and
- the right fix was to strengthen the rewrite ladder
  (absorbing folds, unsigned boundary comparisons, const-selector
  muxes), not to suppress or special-case Verilator.

This is the pattern to keep following for the NodeId-identity roadmap:
when equivalent local forms are discovered in emitted SV, first ask
whether they should have already become the same node in the IR.

### Signoff-quality and bug-finding are not competing goals (2026-04-20)

The user clarified the product direction explicitly:

- `anvil` should become a signoff-level quality random synthesizable
  RTL generator;
- generated modules should be clean in tools like Verilator and Yosys
  by default; and
- `anvil` should still be strong enough to break downstream parsers,
  elaborators, synthesizers, and similar consumers.

Those statements are compatible. The project is **not** trying to find
tool bugs by emitting junk, malformed syntax, or semantically dubious
RTL. The intended bug-finding power comes from breadth, interaction
richness, factorization pressure, stateful motifs, hierarchy, memories,
and other legal-but-hard combinations that real tools should accept.

When choosing between slices, prefer work that strengthens one of these
two axes without regressing the other:

1. broader / harder legal design space; or
2. stronger confidence that generated output is clean and robust in
   downstream tools.

### Verbatim user doctrine: structure over intended functionality (2026-04-20)

The following user guidance is intentionally logged **verbatim** because
it is doctrinal and should steer future implementation choices:

> Let's be clear. Generating module by recursively generating fanin cones of its outputs, mechanically means that the resulting functionality will be gibberish but that's not the point. Having functioning behavior makes no sense here. For some modules, we might get some usable functionality but that's not the goal. The ultimate goal is to be able to generate synthesable legit RTL code that downstream tools (parser, synthesizer, linter, ...) can ingest.
>
> My construction we are not aiming at functionality but at structure, capiche.
>
> ANVIL will be able to create complex to very complex synthesizable RTL code.
>
> Any functionally correct synthesizable RTL code is undistinguishable from an functionally incorrect or even gibberish code at first sight, to ensure function correctioness one need functonal verification which needs to match a specification against a RTL module.
>
> So no one can tell at first glance whether a RTL is gibberish or functionally correct with a specification, meaning for most of what will be generated, function correctness is not the goal and can't be by construction.
>
> But they are features that will create functionally correct blocks.

Operational consequence: optimize ANVIL primarily for structural
legitimacy, synthesizability, complexity, and downstream-tool
ingestibility. Treat whole-module function correctness as out of scope
unless a feature introduces a local block motif whose own behavior is
well-defined by construction.

### Codebase suitability assessment: four steering gaps (2026-04-20)

The short answer to "is the existing codebase suited to the goal?" is:
**yes, as a foundation; no, not yet as a finished system**.

Why "yes": the architecture already matches the problem. `gen` builds a
typed IR instead of text, `Module::intern_gate` is a single
construction-time chokepoint for combinational identity,
`ir::compact` owns post-drain cleanup and state-finalisation work,
`validate` owns the invariant contract, `config` keeps the control
surface explicit, and the SV emitter stays deliberately dumb. That is
the right shape for a signoff-grade legal-RTL generator.

What still needs to stay explicit:

1. **Feature breadth grows above the leaf kernel, not by muddying it.**
   `src/gen/module.rs` is the leaf-module kernel. Hierarchy should land
   as a higher layer (planned `src/gen/hierarchy.rs`), not as ad hoc
   special cases in the leaf path. Likewise, memories/FSMs/aggregates
   should become first-class motifs or module-level generators, not
   emitter tricks.
2. **`NodeId`-as-identity must keep expanding through the IR, not via
   emitter magic.** Today's live coverage is normalized combinational
   identity plus a conservative endpoint-preserving state merge.
   Future work is stronger state identity across richer state graphs and
   later hierarchical/block identity, but it must stay faithful to the
   doctrine: same identity requires proven same functionality with
   respect to the same canonical leaf variables. Keep
   `--identity-mode` as the coarse on/off switch and
   `--factorization-level` as the finer dial; construction strategy
   must stay orthogonal.
3. **Tool cleanliness must be industrialized.** Seed 42 being clean is
   good news, not a stopping point. Each new motif/category/knob needs
   matrixed Verilator/Yosys evidence, retained seed+config
   counterexamples, and root-cause fixes at the IR/generator layer
   rather than warning suppressions.
4. **Structure-first doctrine remains load-bearing.** Absent a
   specification, whole-module functional intent is not the optimization
   target. Invest in legal interaction surfaces, factorization
   pressure, hierarchy, and stateful richness. Functionally correct
   local blocks are welcome; a bundled whole-module oracle is not the
   direction.

### Endpoint-preserving functional doctrine for state identity (2026-04-20)

The user clarified the intended meaning of state equality sharply:

- two fanin cones may **not** share one `NodeId` if they do not have the
  same leaf endpoints as variables;
- the relevant variables are the canonical leaf endpoints: primary
  inputs and/or flop `Q` outputs; and
- the goal is equality by proven same functionality with respect to
  those same endpoints, not equality by visual resemblance or by
  matching graph skeleton alone.

Operational consequence:

- `merge_equivalent_flops` now uses a conservative leaf-aware proof form
  over the already-normalized IR rather than exact `d: NodeId`;
- that proof form now includes a bounded semantic check for
  small-support cones, so some different-shape cones can merge when
  they evaluate identically over the same canonical endpoint set; and
- any future strengthening of sequential identity must preserve the
  canonical leaf namespace. "Rename each owning `q` to SELF" is **not**
  acceptable in strict `NodeId as identity` mode, and neither is
  equating cones solely because they happen to look structurally alike.

---

## Generation-time defects observed in sample output (pending fixes)

Cataloguing real defects observed in sample module `mod_1_0000`
(3 outputs, 10-level fanin, default knobs, graph-first strategy).
These are generator bugs — not SV-emitter or validator bugs.
Enumerated here so the next session can fix them at the root.

- **Constant-select muxes.** Every `wN = (2'h2 == 2'hK) ? ... : ...`
  in the sample is a mux whose select is a *literal* comparison of
  two literals. The select folds at elaboration. Root cause: the
  encoded-mux assembler feeds the select-side recursion through
  the same `pick_terminal` path that can terminate on a constant
  leaf, and the one-hot-mux assembler similarly accepts a constant
  for the per-arm select bit. Fix: in mux-select position, forbid
  constant termination — require a non-constant signal source.
- **N-arity self-cancellation.** `w_21 = i_2 ^ i_2 ^ i_2 ^ i_2 = 0`.
  The N-arity operator expansion re-picks the same pool entry for
  every operand, and `Xor` of even repetitions is zero. Fix: the
  anti-collapse check must look at operand *multiset equality* for
  idempotent / self-inverse operators, not just dep-set
  non-emptiness. (And for `And`/`Or` the same issue produces
  `x & x & x = x` which is a structural collapse, not a zero, but
  still a motif violation.)
- **Coefficient width overflow.** `1'h6` appears — a 6 encoded in a
  1-bit literal, which truncates to 0. Root cause: the linear-
  combination coefficient generator picks the coefficient value
  independently of the operand width. Fix: clamp the coefficient to
  `bits ≤ operand_width`, or widen the literal to the operand width
  and let the top bits be real.
- **Dead wires.** `w_17`, `w_26`, `w_27`, `w_29` are declared and
  assigned but never read. Graph-first speculative pool growth is
  the source; Rule 18 (proposed) addresses this.
- **Stranded flop.** `r_3 <= r_3` — a flop whose D is its own Q and
  whose Q is never read. A no-op. Rule 18 covers this too, as long
  as "consumer" is defined to exclude the flop's own Q feedback.
- **Structurally-identical one-hot arms.** `w_8`, `w_10`, `w_12`,
  `w_14` are all `{w_6,...} & w_5`, meaning four arms of the one-
  hot mux have the same per-arm product. OR-reducing identical
  arms collapses to just the arm value. Fix: in one-hot assembly,
  require per-arm *data* distinctness (or require the per-arm
  select to differ; the current issue is that all arms share the
  same broadcast select bit `w_6`).

All six share a theme the user articulated: signals are being
created without a *reason to exist*. The fixes are three-category:
(1) tighten anti-collapse (operand-multiset check); (2) position-
dependent leaf rules (no const in mux select); (3) width-aware
constant generation. Rule 18 addresses the orthogonal
"unconsumed output" axis.

---

## File-level conventions

- Every Rust source file starts with a doc comment explaining its scope.
- Public types in `ir/types.rs` and `config.rs` get full doc comments. Internal helpers do not need them.
- No multi-paragraph docstrings. One short line; if more is needed, link to `book/`.
- No comments explaining *what* the code does; only *why* when non-obvious.

---

## Construction-time CSE via `Module::intern_gate` (2026-04-15 → 2026-04-16)

Design decision: *all* `Node::Gate` and `Node::Constant` creation is routed through two inherent methods on `Module`:

```rust
pub fn intern_gate(&mut self, op, operands, width, deps) -> (NodeId, bool);
pub fn intern_constant(&mut self, width, value) -> (NodeId, bool);
```

The boolean return is `is_new`: callers that also maintain a `SignalPool` must call `pool.add` only when `is_new` is true, otherwise the pool accumulates duplicate entries for deduped nodes.

Rationale: we need CSE at *construction* time, not as a post-pass. Rule 21 ("AST-instance cap") uses the dedup tables on `Module` as the single source of truth for "which NodeIds represent which expressions."

Rejected alternative: decouple the dedup table from `Module`, keep it in the generator. Rejected because the dedup is an IR-level invariant — the emitter and validator may also want to reason about it, and the tables must survive a `Module::clone()`.

### Snapshot contract with `build_cone_with_retry`

`build_cone_with_retry` rewinds state on empty-dep retries. Before the snapshot fix, it rolled back `m.nodes.truncate(snap_len)` but *not* `gate_instances` / `const_instances`. Stale entries then pointed at truncated `NodeId`s; subsequent intern calls would return a different node than the key promised (witnessed by `const_comparand_across_all_strategies_is_valid` failing at seed 2 Interleaved during the migration).

Fix: snapshot and restore `gate_instances` and `const_instances` alongside `m.nodes`, `m.flops`, pool, and worklist. The `HashMap::clone` cost is bounded by module size — measured negligible on the default knob range.

## Rule 18 "No orphan gates": α construction-time (2026-04-16)

Two enforcement paths were considered:

- **(α) Construction-time:** only create a gate when a specific consumer is already waiting for it. `build_cone` snapshots state before operand construction; on anti-collapse rejection, the snapshot is restored — operand sub-trees vanish from the IR. `process_signal_frame` (interleaved) can't snapshot per-gate because sibling frames have committed, so it delivers one of the existing operand NodeIds as the fallback instead of calling `pick_terminal` (which would create a fresh orphan-prone node).
- **(β) Emission-time tree-shake:** post-generation, compute the live set from drive-roots + flop D/Q transitive fanin, emit only that set.

Rejected β: it's a generate-then-filter step, violating the "by construction" doctrine. User-memory feedback: *"Rule-based generation, not post-hoc filtering."* α is adopted.

Corollary: GraphFirst retired. Its phase-1 speculative pool growth produced 13–27 % orphan gates per module. The variant is kept as a silent CLI alias for Interleaved for backward compat; the dedicated code path (`build_graph_first`, `grow_pool_one_unit`, `*_pool_only` helpers) is unreachable at runtime and may be removed in a future cleanup slice.

## Full factorization doctrine (2026-04-16)

User framing: **`NodeId` is the identity of an expression**; two expressions that are the same mathematically must share one NodeId, different expressions must have different NodeIds.

Implementation ladder (see `book/src/structural-rules.md` Rule 21c):

1. Syntactic CSE (Rule 21) — `(op, operands, width)` key. **Implemented.**
2. Operand-uniqueness (Rule 8 extended) — no NodeId twice in one operand list. **Implemented.**
3. Commutative normalization (Rule 21b) — sort commutative operands before interning. **Implemented.**
4. Associative flattening — flatten `(a+b)+c` to `Add(a,b,c)` when semantically safe. **Implemented.**
5. Constant folding — `x+0 → x`, all-constant evaluation, etc. **Implemented.**
6. Peephole — local algebraic / structural rewrites. **Implemented.**
7. E-graph — full semantic equivalence. **Not implemented.** Default user-requested level.

`FactorizationLevel::effective()` clamps user requests down to the highest implemented layer so aspirational levels don't error. Today that means `e-graph` requests resolve to `peephole`. Construction strategy is orthogonal: `sequential` / `shuffled` / `interleaved` decide build order, while the factorization ladder decides identity/sharing strength.

## Identity mode is orthogonal to construction strategy (2026-04-20)

User clarification that should remain durable:
**"NodeId as identity" is a mode of operation, not a cone-builder.**

That means:
- `construction_strategy` answers *how fanin cones are walked/built*
  (`sequential`, `shuffled`, `interleaved`, graph-first alias);
- factorization / identity mode answers *when two built objects are
  considered the same thing* and therefore must share one NodeId.

Implementation consequence: expose the peak-sharing / no-sharing
switch as a separate CLI axis (`--full-factorization`,
`--no-full-factorization`) rather than pretending it is another
construction strategy value. Future work on the true NodeId-as-
identity engine must preserve this separation.

## Identity mode is now a first-class typed axis (2026-04-20)

The separation above now lives in the code, not just in the docs:

- `Config` owns a new `IdentityMode` enum with `node-id`
  (default) and `relaxed`.
- `Module` mirrors both `identity_mode` and the requested
  `factorization_level`.
- The actual gating sites consult
  `effective_factorization_level()` instead of reading the raw
  ladder directly.

Design consequence:
- `identity_mode = relaxed` is the coarse hard-off switch. It
  forces the effective level to `none`, so `intern_gate` and
  `intern_constant` always allocate fresh NodeIds.
- `identity_mode = node-id` means the ladder is live, and
  `factorization_level` becomes the fine-grained selector within
  that mode.

This is the minimum architectural move that makes the future
"NodeId as identity" engine honest: the repo can now talk about
identity mode without smuggling it through the ladder alone.

## Stateful identity must be decided post-drain (2026-04-20)

For gates and constants, identity is knowable at intern time: the
full key exists when `intern_gate` / `intern_constant` runs.

Flops are different. `build_flop_leaf` allocates a Q leaf
immediately, but the flop's semantics are not complete until the
worklist later constructs its D-cone. So the first honest stateful
extension of "NodeId as identity" cannot be an allocation-time guess;
it has to run after drain.

Current rule: after `summarize_flop_mux_metadata`, flops are merged
iff they have the same emitted-state signature over the same canonical
leaf variables: same `width`, `reset_kind`, `reset_val`, and the same
leaf-aware D-cone proof form. Today that proof form has two rungs:

1. normalized structural proof over the already-canonicalized IR; and
2. bounded semantic proof for small-support cones (enumerate every
   endpoint assignment, key by the resulting truth table).

Construction provenance (`FlopKind`, cleared mux operand metadata) is
deliberately ignored once D exists, because emitted hardware semantics
are carried by width/reset/D-cone meaning, not by how the generator
happened to assemble them.

This is intentionally narrower than full sequential equivalence. Two
cones that happen to compute the same function but are not reduced to
the same proof form by the current ladder, or whose endpoint support is
too large for the bounded semantic check, are not merged yet. That
deeper coinductive story remains a
future slice.

## Emitter is a dumb serialiser (2026-04-16)

User-memory feedback: *"All thinking, checks, rules' enforcement ought to be done solely at the IR level. By the time you reach emission it is too late to roll back."*

Consequence: `emit::to_sv` iterates `m.nodes` in order and writes. No filtering, no reachability check, no live-set computation. Any invariant worth enforcing must be enforced at IR construction or at a `generate_leaf_module` finalization step — never at the emitter.

The safety-net audit in `generate_leaf_module` (`count_orphan_gates`) is *at the IR level* and warns on Rule 18 violations; it does not modify the IR. The emitter trusts what it is given.

## Rejected: without-replacement operand picking as the default

For And/Or/Xor/Add/Mul operand lists, operand duplicates are caught by `violates_anti_collapse` after operands are picked. A natural alternative is to pick operands *without replacement* at the source — maintain a `HashSet<NodeId>` during the per-operand loop and exclude already-picked NodeIds.

Considered and not adopted as the default because:
1. Pool sizes at default knobs are often ≤ N (the requested arity). Without-replacement falls back to "partial arity" + distribution shift.
2. Anti-collapse + rollback already gives 0 duplicates at default. The without-replacement change would save RNG cycles at the cost of a distribution shift that has no empirically measured benefit.
3. `operand_duplication_rate` is the documented knob for users who want the alternative behaviour.

Retained for reference in case a future motif benefits from it.

## Finalisation trims metadata-only and unused-bit surface (2026-04-19)

This slice locked in a small but important finalisation doctrine:
**emit what the live hardware uses, not the generator's provisional
scratch structure.**

- **Width adapters now expand to the exact target width.** The old
  non-multiple up-width adapter built an oversized replicated `Concat`
  and then sliced it back down. Functionally fine, but it manufactured
  dead high bits that lint tools quite rightly flagged. The adapter now
  builds the exact-width shape directly (`{src[rem-1:0], src, ...}`).
- **`Flop.mux` operand NodeIds are construction-time metadata, not
  emitted hardware roots.** Once `flop.d` is assembled, keeping the
  original select/data operand references around lets metadata-only
  cones survive liveness/compaction even though the emitter never reads
  them. Finalisation now keeps only the variant shape and discards
  those operand references before compaction.
- **Primary inputs are shrunk/pruned to the live bit surface.** After
  compaction, each surviving primary input is reduced to the highest bit
  any live consumer touches, and entirely unused data inputs are
  dropped from the emitted interface. This keeps Verilator from
  reporting unused input bits or dead ports.
- **Residual associative-opportunity metrics now respect duplicate
  policy.** Nested `Add`/`Mul` slots that would introduce duplicates if
  flattened are intentionally preserved at strict
  `operand_duplication_rate`; the metric now matches that semantic
  policy instead of counting those slots as "missed" flattening.

Rejected alternative: paper over the issue in the emitter with
tool-specific lint pragmas. That would hide the symptom without fixing
the IR/finalisation mismatch.
