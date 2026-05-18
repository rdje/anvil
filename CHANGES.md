# Changes
Fully detailed change history. Newest entries at the top. One entry per commit.
## 2026-05-18-phase8-1 — PHASE-8-FRONTEND-ACCEPT.1: source-level frontend/elaboration accept-corpus IR design

**Landed as:** this commit

**What changed**

- `DEVELOPMENT_NOTES.md`: new "Phase 8 frontend/elaboration
  accept-corpus source-IR design (2026-05-18,
  PHASE-8-FRONTEND-ACCEPT.1)" entry — the shift to *compact
  elaboratable hierarchies* emitted with **parameters unresolved in
  the SV text** + a manifest of what elaboration must resolve;
  codebase grounding (the post-elaboration scalar circuit IR cannot
  express modules / parameter ports / packages / typedef / generate
  ⇒ a dedicated source-level AST IR on a separate generator path);
  the source-IR sketch (`SourceUnit`/`Package`/`Module`/
  `ModuleItem`/`Type`/reused Phase-7 `Expr`); the
  expected-elaboration-facts manifest schema (extends Phase 7's with
  the instance tree / generate selections / package + typedef
  resolutions); oracle-by-construction generation reusing Phase 7's
  evaluator + manifest core; the hierarchy-aware parity harness;
  4 rejected alternatives; the `.2` proof shape + split candidates.
- `docs/tasks/PHASE-8-FRONTEND-ACCEPT.md`: `.1` done (Verification /
  Commit / Frontier→`.2` / Open Question resolved [reuse Phase 7
  core; `.2` sequences after `PHASE-7-ORACLE-MICRODESIGN.2`] /
  Verification-Log / Commit-Log / Changelog / Metadata).
  `docs/TASK_TREE.md` Phase 8 row updated.

**Why**

- Continuous-PNT while Phase 6 `.2.4`/`.3.4b` are gate-blocked.
  Design-only (no code, no CPU, no full-cargo-test) — clears a
  third tree's `.1` with zero contention on the in-flight gate, and
  the separate source-level IR is Phase 8's load-bearing decision
  that must be designed before any code.

**Validation**

- Design-only; **no code change** (diff = `DEVELOPMENT_NOTES.md` +
  the Phase 8 tree + `docs/TASK_TREE.md` + `CHANGES.md` +
  `MEMORY.md`). `mdbook build book` clean; `cargo fmt --all --check`
  clean; full `cargo test` was green at this slice's base `f0cff2c`
  and no `src/`/`tests/` were touched since.

**Impact**

- Phase 8 has a codebase-grounded target; `.2` (implement, expected
  to split; sequenced after `PHASE-7-ORACLE-MICRODESIGN.2` for the
  shared evaluator/manifest core) is unblocked. No ROADMAP advance.

**Files touched**

- `DEVELOPMENT_NOTES.md`; `docs/tasks/PHASE-8-FRONTEND-ACCEPT.md`;
  `docs/TASK_TREE.md`; `CHANGES.md`; `MEMORY.md`.

---

## 2026-05-18-phase7-1 — PHASE-7-ORACLE-MICRODESIGN.1: oracle-backed micro-design artifact-family design

**Landed as:** f0cff2c

**What changed**

- `DEVELOPMENT_NOTES.md`: new "Phase 7 oracle-backed micro-design
  artifact family design (2026-05-18, PHASE-7-ORACLE-MICRODESIGN.1)"
  entry — the conceptual shift (Phases 1–6 random RTL with no
  semantic oracle vs Phase 7 tiny `.sv` whose elaboration facts are
  known by construction + a machine-checkable manifest); codebase
  grounding (the scalar-`u32` circuit IR has no
  parameter/localparam/generate/package/typed-constant concept ⇒
  Phase 7 needs its own small source-level const/parameter IR and a
  separate generator path); the `rtl_const_expr` family; the
  expected-facts JSON manifest schema sketch; the
  **oracle-by-construction** generation strategy (the generator
  evaluates every const-expr/param node as it builds it and emits
  the `.sv` and the manifest from the same resolved values — no
  analysis pass, no re-parse); the reproducibility contract; a new
  parity harness distinct from the `tool_matrix` DUT gate; the
  Phase-8/9 boundaries; 4 rejected alternatives; the `.2` proof
  shape + split candidates.
- `docs/tasks/PHASE-7-ORACLE-MICRODESIGN.md`: `.1` done (Verification
  / Commit / Frontier→`.2` / both Open Questions resolved [typed JSON
  manifest; new separate parity harness] / Verification-Log /
  Commit-Log / Changelog / Metadata). `docs/TASK_TREE.md` Phase 7
  row updated.

**Why**

- Continuous-PNT: both remaining Phase 6 leaves (`.2.4`, `.3.4b`)
  are gate-blocked (the in-flight memory gate is alive but
  CPU-starved; a second concurrent gate would starve both). Phase 7
  `.1` is design-only — no code, no CPU contention, no
  full-cargo-test — exactly the right unblocked work while the
  gates drain, and it mirrors the proven Phase 5/5b/6 design-first
  method.

**Validation**

- Design-only; **no code change** (diff = `DEVELOPMENT_NOTES.md` +
  the Phase 7 tree + `docs/TASK_TREE.md` + `CHANGES.md` +
  `MEMORY.md`). `mdbook build book` clean; `cargo fmt --all --check`
  clean; full `cargo test` was green at this slice's base `5db4ac9`
  and no `src/`/`tests/` were touched since.

**Impact**

- Phase 7 has an unambiguous, codebase-grounded target; `.2`
  (implement, expected to split IR+evaluator / emitters /
  harness+gate) is unblocked. No ROADMAP advance (design-only). The
  Phase 6 gate situation is unchanged (still draining).

**Files touched**

- `DEVELOPMENT_NOTES.md`; `docs/tasks/PHASE-7-ORACLE-MICRODESIGN.md`;
  `docs/TASK_TREE.md`; `CHANGES.md`; `MEMORY.md`.

---

## 2026-05-18-phase6-3.4a — PHASE-6-ADVANCED-MOTIFS.3.4a: `phase6_fsm` matrix scenario + `num_fsm_modules` metric + gap

**Landed as:** 5db4ac9

**What changed**

- `src/metrics.rs`: `DesignMetrics.num_fsm_modules` (count of
  `design.modules` with `!fsms.is_empty()`) + computed + added to the
  struct literal — mirrors `num_memory_modules`.
- `src/bin/tool_matrix.rs`: `CoverageSummary.saw_fsm_design`; the
  `phase6_fsm` scenario tuple (registered after
  `phase6_inferrable_memory`, `next_seed + 73`);
  `phase6_fsm_focus_config` — an exact clone of
  `phase6_inferrable_memory_focus_config` (depth-1 wrapper, library,
  4 leaves / 4 instances, all hierarchy-routing probabilities 0.0,
  EGraph, width band 2..8) with the **single** change
  `fsm_prob = 1.0` instead of `memory_prob`, so the matrix's
  leaf/child/range/source shape-coverage sets are unperturbed;
  coverage set (`fsm_prob > 0 && num_fsm_modules > 0`); `merge_into`;
  the `Phase4Hierarchy` `compute_coverage_gaps` arm; bin counts
  **219→222 scenarios / 876→888 modules**; the scenario-name
  exception list `+= phase6_fsm`; a new
  `phase6_fsm_scenario_is_non_vacuous` test.

**Why**

- `PHASE-6-ADVANCED-MOTIFS.3.4a` — the matrix scenario + metric +
  coverage gap for the FSM motif, mirroring memory `.2.3` exactly.
  `.3.4` was split (`.3.4a` scenario code, no advance / `.3.4b`
  real-gate verify + Phase-6/tree closure) per the proven memory
  `.2.3`/`.2.4` decomposition + r87: the scenario lands before any
  advance; the gate-verify is a separate gated step. The non-vacuity
  test guarantees the `saw_fsm_design` coverage fact is reachable so
  `.3.4b`'s gate cannot carry a permanent gap.

**Validation**

- `cargo check --all-targets` clean (`CoverageSummary` uses
  `..Default::default()` ⇒ no struct-literal breakage);
  `cargo test --bin tool_matrix` **29/29** (the phase4 bin test +
  the 222-scenario `covers_wrapper` test confirm the **+3 scenarios
  / +12 modules** delta matches `phase6_inferrable_memory` exactly);
  `phase6_fsm_scenario_is_non_vacuous` green; `cargo fmt --all
  --check` / `cargo clippy --all-targets -- -D warnings` clean; full
  `cargo test` green (COMMIT.md gate). **No ROADMAP advance** —
  promotion is `.3.4b` on a verified gate (r87 no-aspirational-
  claims). No `book/` change (book reconciliation is `.3.4b`).
  `CODEBASE_ANALYSIS.md` / `DEVELOPMENT_NOTES.md` intentionally
  unchanged — consistent with the memory `.2.3` sibling precedent
  (mechanical scenario mirror, no new design decision/gotcha; the
  bootstrap-refreshed snapshot is not amended per scenario slice).

**Impact**

- The FSM coverage scenario + metric + gap are wired; `.3.4b` (a
  fresh real repo-owned gate — one run covers memory + fsm —
  verified clean → records FSM delivered + closes ROADMAP Phase 6 +
  the `PHASE-6-ADVANCED-MOTIFS` tree) is the last remaining leaf,
  gate-blocked. Frontier: `.2.4` (gate-blocked) ‖ `.3.4b`.

**Files touched**

- `src/metrics.rs`; `src/bin/tool_matrix.rs`;
  `docs/tasks/PHASE-6-ADVANCED-MOTIFS.md`; `docs/TASK_TREE.md`;
  `CHANGES.md`; `MEMORY.md`.

---

## 2026-05-18-phase6-3.3 — PHASE-6-ADVANCED-MOTIFS.3.3: FSM structural-contract + factorization-opacity proof

**Landed as:** 725e7fc

**What changed**

- `tests/pipeline.rs`: new proof
  `fsm_block_matches_probed_template_and_is_factorization_opaque` —
  across 4 `ConstructionStrategy` × 4 `FactorizationLevel`
  (None/Cse/Commutative/EGraph) × 6 seeds (96 designs), with
  `fsm_prob = 1.0`: `validate_design` clean; exactly 1 module / 1
  `Fsm`; **exactly one `FsmOut` survives every factorization level
  (incl. EGraph) and the FSM leaf has zero `Gate` nodes** (the state
  machine never enters the NodeId expression graph); the emitted
  state constants are **exactly** the chosen `FsmEncoding`'s
  (`state_width`/`state_const`: Binary `s` / OneHot `1<<s` / Gray
  `s^(s>>1)`) plus the exact async-low-reset state register and the
  `sel`-selected next-state / Moore `case`s; all three encodings
  reachable across the deterministic seeds-0..6 sweep.

**Why**

- `PHASE-6-ADVANCED-MOTIFS.3.3` — the cargo-portable formalization
  of the Phase 6 FSM contract (the cargo gate cannot shell
  Yosys/Verilator; the tool-level proof is `.3.4`'s real repo gate).
  The template-equivalence + factorization/CSE-opacity *is* the
  contract, mirroring how memory `.2.2` was proved. Keying the
  structural assertion on the exact `FsmEncoding` constant formula
  is robust where Binary/Gray coincide at `N = 2` (a weaker
  "pairwise-distinct SV" claim would be false there).

**Validation**

- Proof-only — `git diff` = `tests/pipeline.rs` (+ tree/live-docs);
  **no `src/` change**. New proof green; `cargo fmt --all --check` /
  `cargo clippy --all-targets -- -D warnings` / `cargo check
  --all-targets` clean; full `cargo test` green (COMMIT.md gate).
  Default-off byte-identical is reaffirmed by `.3.2b`'s focused proof
  (unchanged). No `book/` change (book reconciliation is `.3.4`).
  `CODEBASE_ANALYSIS.md` intentionally unchanged — consistent with
  the memory `.2.2` proof-only sibling precedent (the
  bootstrap-refreshed snapshot is not amended per proof slice; the
  live record is the tree + `CHANGES.md` + `MEMORY.md`).

**Impact**

- The FSM motif's structural correctness + factorization-opacity is
  cargo-proven. `.3.4` (the last Phase 6 leaf — `phase6_fsm` matrix
  scenario + metric + gap → real-gate verify → closes ROADMAP Phase 6
  + the tree) is unblocked for its scenario-landing part. No ROADMAP
  advance. Frontier: `.2.4` (gate-blocked) ‖ `.3.4`.

**Files touched**

- `tests/pipeline.rs`; `docs/tasks/PHASE-6-ADVANCED-MOTIFS.md`;
  `docs/TASK_TREE.md`; `CHANGES.md`; `MEMORY.md`.

---

## 2026-05-18-phase6-3.2b — PHASE-6-ADVANCED-MOTIFS.3.2b: `fsm_prob` knob + rules-first `build_fsm_block` (`.3.2` container done)

**Landed as:** 9ee5285

**What changed**

- `src/config.rs`: new calibration knob `Config::fsm_prob` (`f64`,
  `#[serde(default = "default_fsm_prob")]` → `0.0`, Default-impl line,
  probability-range `[0.0,1.0]` validation tuple entry) — mirrors
  `memory_prob`/`aggregate_prob`/`width_parameterization_prob`.
- `src/gen/module.rs`: rules-first `build_fsm_block` (clk/rst_n + one
  `sel` input, `q` output; `num_states` `g.rng` 2..=6; `encoding`
  `g.rng` Binary|OneHot|Gray; `sel_width` `g.rng` 1..=2; `out_width`
  from the configured width band; `transitions[s][j]=(s+1+j)%N` by
  rule; distinct masked per-state Moore outputs; opaque `FsmOut`
  drives `q`; no gates/flops — all rolls via `g.rng`, reproducible)
  + a single opt-in roll in
  `generate_leaf_module_with_interface_profile` placed **after** the
  Phase 5 param lane and the Phase 6 memory lane (mutually exclusive;
  `interface_profile.is_none()` only; default-off `fsm_prob == 0.0`
  never enters → byte-identical).
- `tests/pipeline.rs`: focused proof
  `fsm_block_is_default_off_and_constructs_when_forced_on`.

**Why**

- `PHASE-6-ADVANCED-MOTIFS.3.2b` — the FSM scaffold's knob + rules-
  first constructor, closing the `.3.2` container. Construction-time
  rules, never generate-then-filter; one exclusive motif per
  free-standing single-module design (the proven Phase-5/5b/6-memory
  opt-in-lane discipline).

**Validation**

- Focused proof green: (a) default-off byte-identical (no `Fsm`, no
  `fsm_state_0`/` fsm_0;`) across 4 `ConstructionStrategy` × 6 seeds;
  (b) forced-on (`fsm_prob = 1.0`) every single-module design is a
  1-`Fsm` leaf that `validate_design`-passes, exposes a `FsmOut`, and
  emits the `.3.1`-probed-clean template (`fsm_state_0` +
  `FSM0_S0 =` constants + `always_ff @(posedge clk or negedge rst_n)`
  with `if (!rst_n) fsm_state_0 <= FSM0_S0` + `case (fsm_state_0)`);
  **all three generated encodings reachable** across the sweep.
  `cargo fmt --all --check` / `cargo clippy --all-targets --
  -D warnings` / `cargo check --all-targets` clean; full `cargo test`
  green (COMMIT.md gate). No `book/` change (book reconciliation is
  `.3.4`). `CODEBASE_ANALYSIS.md` intentionally unchanged — consistent
  with the memory `.2.1b` sibling precedent (the bootstrap-refreshed
  snapshot is not amended per knob slice; the live record is the tree
  + `DEVELOPMENT_NOTES.md` + `CHANGES.md` + `MEMORY.md`).

**Impact**

- The `.3.2` FSM-scaffold container is **done**. `.3.3` (cargo-
  portable structural + CSE/EGraph-opacity proof, mirrors memory
  `.2.2`) is unblocked. No ROADMAP advance (promotion is `.3.4` on a
  verified gate). Frontier: `.2.4` (gate-blocked) ‖ `.3.3`.

**Files touched**

- `src/config.rs`; `src/gen/module.rs`; `tests/pipeline.rs`;
  `docs/tasks/PHASE-6-ADVANCED-MOTIFS.md`; `docs/TASK_TREE.md`;
  `DEVELOPMENT_NOTES.md`; `CHANGES.md`; `MEMORY.md`.

---

## 2026-05-18-phase6-3.2a — PHASE-6-ADVANCED-MOTIFS.3.2a: FSM IR core + opaque `FsmOut` leaf + `compact.rs` reachability

**Landed as:** ef0eef7

**What changed**

- `src/ir/types.rs`: `FsmId`; `FsmEncoding{Binary,OneHot,Gray}` with
  `state_width()` (Binary/Gray = `ceil(log2 N)`, OneHot = `N`) and
  `state_const()` (Binary `s`, OneHot `1<<s`, Gray `s^(s>>1)`); the
  `Fsm` struct (`num_states`, `encoding`, `sel:NodeId`, `sel_width`,
  `transitions[N][1<<sel_width]`, `outputs[N]`, `out_width`); additive
  `Default`-empty `Module.fsms`; opaque `Node::FsmOut{fsm,width}` +
  the `Node::width` arm; `DepAtom::FsmVirtual` +
  `DepSet::from_fsm_virtual`; `has_local_fsms()` OR'd into both
  `carries_sequential_state` predicates.
- `FsmOut` threaded — via the compiler as a completeness oracle —
  through **every** exhaustive `Node` match: `src/ir/compact.rs`
  (`StructuralNodeShape::FsmOut`, `LeafEndpoint::FsmOut` + `width()`,
  intern, leaf-endpoint set, cone-eval offset, flop-remap no-op,
  the **load-bearing reachability arm** that marks `fsm.sel` alive —
  sibling to `MemRead` keeping its address/data cones, byte-identical
  rebuild, instance-table no-op, `node_deps`); `src/gen/cone.rs` ×5;
  `src/gen/hierarchy.rs`; `src/gen/module.rs`; `src/ir/param.rs`
  (`FsmOut` ⇒ not width-generic); `src/metrics.rs` ×3 (kind-count
  no-op, `node_deps`, structural-hash tag 7).
- `src/ir/validate.rs`: 5 `ValidateError` variants + step 5c (slot-id,
  `num_states>=1`, `sel_width`/`out_width>=1`, `sel` node defined +
  width, transitions shape + range, outputs length + width mask;
  every `FsmOut` resolves at `out_width`).
- `src/emit/sv.rs`: `fsm_out_name`/`fsm_state_name`/`fsm_next_name`/
  `fsm_state_lit`; per-FSM declarations; the `.3.1`-probed-clean
  template (per-FSM `FSM<id>_S<k>` encoding-derived `localparam`s,
  `always_comb` next-state `case` selected by `sel`, async-low-reset
  state `always_ff` on the shared `clk`/`rst_n`, `always_comb` Moore
  output `case`); `Node::FsmOut → fsm_out_name`.
- `src/ir/compact.rs` tests: 3 FSM unit proofs
  (`fsm_leaf_roundtrips_validate_and_emit`,
  `fsmout_keeps_sel_cone_through_compaction`,
  `fsmout_is_structurally_distinct_and_not_cse_merged`).

**Why**

- `PHASE-6-ADVANCED-MOTIFS.3.2a` (the FSM scaffold's IR core),
  mirroring the proven memory `.2.1a` opaque-stateful-leaf pattern.
  The load-bearing piece is the `compact.rs` reachability: a
  reachable `FsmOut` must transitively keep the FSM's `sel`
  transition cone alive, exactly as a reachable `MemRead` keeps the
  memory's address/data cones — that is correctness-critical pipeline
  code, which is why `.3.2` was split (`.3.2a` IR core / `.3.2b`
  knob+generator).

**Validation**

- `cargo check --all-targets` clean (the additive `Module.fsms` field
  is covered by `Module`'s `Default`, so no struct-literal breakage);
  `cargo fmt --all --check` clean; `cargo clippy --all-targets --
  -D warnings` clean; full `cargo test` green (COMMIT.md gate; FSM
  proofs 3/3, no regression). **Default-off is trivially
  byte-identical**: the emitter blocks are gated on
  `!m.fsms.is_empty()`, the predicates only OR when `fsms` is
  non-empty, and the `FsmOut` arms only fire when a `FsmOut` node
  exists — none occur without the (`.3.2b`) generator. No `book/`
  change (book reconciliation is `.3.4`). `CODEBASE_ANALYSIS.md`
  intentionally unchanged — consistent with the immediate sibling
  precedent (memory `.2.1a` did not amend the bootstrap-refreshed
  snapshot per IR-core slice); the live record is the task tree +
  `DEVELOPMENT_NOTES.md` + `CHANGES.md` + `MEMORY.md`.

**Impact**

- The FSM motif's IR + opaque leaf + pipeline integration are in
  place and proven; `.3.2b` (knob + rules-first `build_fsm_block`)
  is unblocked. No ROADMAP advance (promotion is `.3.4` on a verified
  gate). Frontier: `.2.4` (gate-blocked) ‖ `.3.2b`.

**Files touched**

- `src/ir/types.rs`; `src/ir/compact.rs`; `src/ir/validate.rs`;
  `src/ir/param.rs`; `src/gen/cone.rs`; `src/gen/hierarchy.rs`;
  `src/gen/module.rs`; `src/metrics.rs`; `src/emit/sv.rs`;
  `docs/tasks/PHASE-6-ADVANCED-MOTIFS.md`; `docs/TASK_TREE.md`;
  `DEVELOPMENT_NOTES.md`; `CHANGES.md`; `MEMORY.md`.

---

## 2026-05-18-phase6-3.2-split — PHASE-6-ADVANCED-MOTIFS.3.2 split into `.3.2a` (IR core) + `.3.2b` (knob)

**Landed as:** f39f66b

**What changed**

- `docs/tasks/PHASE-6-ADVANCED-MOTIFS.md`: `.3.2` becomes a container
  with children `.3.2a` (FSM IR core + opaque `Node::FsmOut`
  pipeline integration incl. the load-bearing `compact.rs`
  reachability + 3 unit proofs; no generator/knob ⇒ default-off
  trivially byte-identical) and `.3.2b` (`Config::fsm_prob` +
  rules-first `build_fsm_block` + default-off/forced-on focused
  proof). Decisions / Changelog / Current Frontier / Metadata
  updated; `docs/TASK_TREE.md` Phase 6 row updated.

**Why**

- Tree-planning, no code. `.3.2` (IR core + reachability + emitter +
  validator + ~21 exhaustive `Node` match sites + knob + rules-first
  generator + focused proof + full gate) cannot reach signoff in one
  reviewable slice — exactly why memory's `.2.1` split. Unlike `.2.1`
  (split *after* implementation surfaced the dependency), `.3.1`'s
  design already established that `Node::FsmOut` carries the
  *identical* correctness-critical compaction-reachability obligation
  as the landed `Node::MemRead` (a reachable `FsmOut` must keep the
  FSM's `sel` condition cone alive). The lower-level dependency is
  known concretely from the landed `.2.1a`, not speculative, so the
  split is "decided when reached" with the dependency in hand —
  Splitting Rules + the proven `.2.1` precedent.

**Validation**

- Tree/docs-only — **no code change** (diff = the two tree docs +
  `CHANGES.md` + `MEMORY.md`). `mdbook build book` clean (no `book/`
  touched); `cargo` untouched (unchanged-green at base `4dddf0e`).

**Impact**

- `.3.2a` is the next continuous-PNT leaf (unblocked, independent of
  the running `.2.4` memory gate). No ROADMAP change.

**Files touched**

- `docs/tasks/PHASE-6-ADVANCED-MOTIFS.md`; `docs/TASK_TREE.md`;
  `CHANGES.md`; `MEMORY.md`.

---

## 2026-05-18-phase6-3.1 — PHASE-6-ADVANCED-MOTIFS.3.1: generated-encoding FSM motif design (`.3` split)

**Landed as:** 4dddf0e

**What changed**

- `DEVELOPMENT_NOTES.md`: new "Phase 6 generated-encoding FSM motif
  design (2026-05-18, PHASE-6-ADVANCED-MOTIFS.3.1)" entry. Codebase
  grounding (IR has no FSM concept; `Flop` is the only stateful
  element; the operators-vs-blocks doctrine → an FSM is a **block**).
  An empirical downstream probe of the *exact* SV ANVIL would emit for
  a 4-state Moore FSM in **all three generated encodings** — binary
  (2-bit `state_q`), one-hot (4-bit), gray (2-bit): every one is
  `verilator --lint-only -Wall` exit 0, `yosys synth -noabc;
  check -assert` clean, and `yosys synth; abc -fast; check -assert`
  clean (**both** repo Yosys modes). State width/constants differ by
  encoding ⇒ "encoding selectable" is a structural fact. Chosen
  architecture **(F)**: additive `Vec<Fsm>` on `Module`
  (Default-empty) + opaque `Node::FsmOut` leaf (sibling to
  `FlopQ`/`MemRead`, never CSE'd, same `compact.rs` reachability
  obligation discovered in `.2.1a`) + encoding-derived-constant
  emitter on the shared `clk` + opt-in `Config::fsm_prob` serde-default
  0.0. Four rejected/deferred alternatives ((A) primitives-only,
  (B) emitter-only string, (C) generic enum/typedef datatype,
  (D) Mealy — deferred, not a `.3` blocker).
- `docs/tasks/PHASE-6-ADVANCED-MOTIFS.md`: `.3` split into a container
  `.3.1`(done)/`.3.2`/`.3.3`/`.3.4` mirroring the proven memory
  `.2.1`–`.2.4`; `.3.1` Verification/Commit/Decisions/Open-Questions/
  Verification-Log/Commit-Log/Changelog/Frontier/Metadata updated.
  FSM is recorded as the **last** Phase 6 motif: a verified-clean
  `.3.4` gate closes ROADMAP Phase 6 + the tree (memory delivered at
  `.2.4`; multi-clock CDC stays the optional separately-prioritised
  deferral). `docs/TASK_TREE.md` Phase 6 row updated.

**Why**

- Continuous-PNT doctrine: the active Phase 6 tree's `.2.4` memory
  leaf is verify-only and gate-blocked (the real `tool_matrix` gate is
  running); `.3` (FSM) is the substantive remaining Phase 6 work and
  is unblocked. Design-first (no code, no CPU) is exactly the right
  work while the gate saturates the machine, and mirrors the proven
  Phase 5 / 5b / memory-`.1` design-first method.

**Validation**

- Design-only; **no code change** (diff = `DEVELOPMENT_NOTES.md` +
  the two tree docs + `CHANGES.md` + `MEMORY.md`). The empirical probe
  used local `verilator`/`yosys` at design time (not the cargo gate —
  project convention since Phase 1). `mdbook build book` clean;
  `cargo fmt --all --check` clean; full `cargo test` was green at this
  slice's base `0b799b6` and no `src/`/`tests/` were touched since.

**Impact**

- Phase 6 FSM motif has an unambiguous, evidence-backed target. No
  ROADMAP advance (design-only; promotion is `.3.4` on a verified
  artifact, r87 no-aspirational-claims). Frontier: `.2.4`
  (gate-blocked) ‖ `.3.2` (next).

**Files touched**

- `DEVELOPMENT_NOTES.md`; `docs/tasks/PHASE-6-ADVANCED-MOTIFS.md`;
  `docs/TASK_TREE.md`; `CHANGES.md`; `MEMORY.md`.

---

## 2026-05-18-book-examples-2.2 — BOOK-EXAMPLES-RUNNABLE.2.2: book-examples harness + embedded migration + mdbook-test CI (tree CLOSED)

**Landed as:** 0b799b6

**What changed**

- **`tests/book_examples.rs` (NEW, leaf-owned code).** A std-only cargo
  integration test that makes every runnable mdBook `bash` example a
  CI gate. It builds the release `anvil` once, parses every ```bash
  fence in `book/src/*.md`, honours an HTML-comment skip sentinel
  (`<!-- book-test: skip — <reason> -->`, mandatory reason), substitutes
  `cargo run --release --` → `"$ANVIL"`, **panics** on any unclassified
  residual (`cargo` / bare `anvil` / `verilator` / `yosys` / `jq` /
  `git clone`) so a gap can never be silent, then runs each non-skipped
  block via `bash -eu -o pipefail` in a fresh temp CWD, offline
  (`CARGO_NET_OFFLINE`), asserting exit 0. Plus a **negative-control**
  test (a deliberately broken flag must fail — so a green run is
  meaningful, not vacuous) and a skip-sentinel-reason test.
- **Embedded-position migration completed** (the honest `.2.1`
  correction): the one `$()`-embedded bare `anvil` → `cargo run`;
  9 reasoned skip sentinels on the install / `verilator` / `yosys` /
  `jq` blocks; 32 bare ``` fences → ```text so `mdbook test` stops
  trying to compile prose as Rust. One real broken example fixed in a
  prior slice (tutorial `min_inputs > max_inputs`) plus two
  dependency-chain blocks made self-contained / sentineled.
- **`.github/workflows/ci.yml`**: added the `mdbook test book` step
  (alongside the existing fmt / clippy / test / `mdbook build`).
- Tree docs (`docs/tasks/BOOK-EXAMPLES-RUNNABLE.md`,
  `docs/TASK_TREE.md`) closed: `.2.2`, the `.2` container, and the
  `BOOK-EXAMPLES-RUNNABLE` tree are all `done`.

**Why**

- The repo is public with live Pages; the owner mandate is that every
  book example works for copy-paste users and can never silently rot.
  `.2.1` made the book *correct*; `.2.2` makes it *enforced* — a broken
  or drifted example now fails CI.
- **Root-caused a harness pipe-buffer deadlock (a harness bug, not a
  book defect).** The first full runs reported 12 blocks "TIMED OUT
  after 600s". A default `--seed 42` module is ≈86 KB of stdout (a
  5-level `factorization` sweep ≈525 KB) but the macOS OS pipe buffer
  is ≈64 KB; `run_script` used `Stdio::piped()` with a `try_wait()`
  loop that never drained the pipe, so `anvil` blocked on `write()`,
  never exited, and the loop spun to the timeout (12 × 600 s ≈ the
  observed 7273 s first-run total). Direct invocation proved the
  examples are correct — each runs in 0.03–0.15 s. Fix: capture child
  stdout/stderr to temp **files** (no buffer limit, std-only, no
  reader-thread plumbing) and reap the child after a timeout kill.

**Validation**

- `cargo test --test book_examples` = **3 passed / 0 failed**: ran
  **54** runnable blocks (all exit 0), **9** skip-sentineled, **76.4 s**
  (was 7273 s before the deadlock fix). Negative control fails on a
  broken command as required.
- `mdbook build book` clean; `mdbook test book` exit 0; `ci.yml`
  carries the `mdbook test book` step.
- `cargo fmt --all --check` clean; `cargo check --all-targets` clean;
  `cargo clippy --all-targets -- -D warnings` clean; full `cargo test`
  green (the only new code is `tests/book_examples.rs`, a separate
  integration binary that cannot regress the lib/unit suite).

**Impact**

- Every runnable book example is now CI-gated against drift. New code
  is one self-contained, leaf-owned integration test; the generator,
  library, and other tests are untouched. `BOOK-EXAMPLES-RUNNABLE` tree
  CLOSED.

**Files touched**

- `tests/book_examples.rs` (new); `.github/workflows/ci.yml`;
  `book/src/*.md` (embedded migration + skip sentinels + bare→text:
  algorithm, architecture, factorization, faq, getting-started, knobs,
  non-triviality, recipes, sequential, sharing, tutorial);
  `docs/tasks/BOOK-EXAMPLES-RUNNABLE.md`; `docs/TASK_TREE.md`;
  `DEVELOPMENT_NOTES.md`; `CODEBASE_ANALYSIS.md`; `CHANGES.md`;
  `MEMORY.md`.

---

## 2026-05-18-book-examples-2.2-scope — BOOK-EXAMPLES-RUNNABLE.2.2 scope: record a discovered `.2.1` migration gap

**Landed as:** 481d9fc

**What changed**

- `docs/tasks/BOOK-EXAMPLES-RUNNABLE.md`: `.2.2` goal expanded + an honest Open-Questions correction recorded. **Discovered during `.2.2` recon:** the `.2.1` migration and its `missed_runnable_bare_anvil = 0` audit were **line-leading only** — bare `anvil` *embedded* in shell command-substitution (`gates=$(anvil …)`) and inside `for … do anvil … done` loops was not migrated and is still not paste-runnable. Full bash-block classification recorded: 48 pure `cargo run`; 8 `# comment` + `cargo run` (runnable); the remainder `$()`/loop (runnable via the harness shell-script model once embedded `anvil` is migrated) or external-tool (`verilator`/`yosys`/`jq`) / `git clone` install blocks (genuine skips). `.2.2` now owns: (a) completing the embedded-position migration; (b) skip sentinels on the ~6 non-runnable blocks; (c) the `tests/book_examples.rs` harness (shell-script model, `cargo run`/`anvil` both shimmed to `CARGO_BIN_EXE_anvil`, offline, timeout, exit-0, fail-on-unclassified, negative control); (d) the `mdbook test book` CI step.

**Why**

- Faithful reporting: `.2.1`'s audit overclaimed (line-leading only). Surfacing it immediately + folding the corrective into `.2.2` (whose harness makes this gap class impossible to reintroduce) is the honest fix, rather than silently patching or leaving a false "0 missed" claim.

**Validation**

- Tree/docs-only — **no code change**. `cargo`/`mdbook` unaffected. (`.2.1`'s book migration remains correct for the common line-leading case; the published book is still strictly better than before — the residual is `$()`/loop embeddings, fixed in `.2.2`.)

**Impact**

- `.2.2` scope is now precise and honest. No behavioural change.

**Files touched**

- Updated: docs/tasks/BOOK-EXAMPLES-RUNNABLE.md, CHANGES.md, MEMORY.md.

## 2026-05-18-book-examples-runnable-2.1 — BOOK-EXAMPLES-RUNNABLE.2.1: migrate book examples to `cargo run --release --`

**Landed as:** 415818c

**What changed**

- A surgical migrator rewrote every line-leading `anvil ` command head **inside ```bash fences** to `cargo run --release -- `, preserving `\`-continuations, `|` pipes, `$`-prompts and indentation: **45** heads across `book/src/factorization.md` (3), `knobs.md` (3), `recipes.md` (39). `getting-started.md`/`introduction.md`/`tutorial.md` already used `cargo run`. Prose mentions of "anvil" and `systemverilog`/`text` output blocks were left untouched; `faq.md`'s `verilator --lint-only anvil_output.sv` was correctly **not** matched (not an anvil invocation).
- All **9** illustrative ```rust IR/struct sketches annotated ```rust,ignore (rendered identically; `mdbook test` will skip them — `.2.2`).
- `getting-started.md` Install: one optional `cargo install --path .` → `anvil` shorthand note (the book otherwise uses `cargo run --release --` everywhere so it works zero-setup from a fresh clone).

**Why**

- `BOOK-EXAMPLES-RUNNABLE.2.1`: the repo is public and the book is live (https://rdje.github.io/anvil/); bare `anvil …` examples are not runnable from a fresh clone (no binary on PATH). Owner decision: standardize on `cargo run --release --`. This makes the published book *correct* before `.2.2` *enforces* it.

**Validation**

- Audit `missed_runnable_bare_anvil = 0` (no runnable ```bash block left bare-`anvil`); `bare_rust_blocks = 0` / `rust,ignore = 9`. `mdbook build book` clean. Three paste-and-run spot-runs against the release binary: reproduce-style → real SV, exit 0; `--dump-config` → valid JSON, exit 0; first full **multi-line** `recipes.md` block run verbatim → **50 `.sv` files, exit 0**. **Docs/book only — no code change** (`git diff` = `book/src/*.md`, 7 files); `cargo`/tests untouched (unchanged-green).

**Impact**

- Every published book example is now copy-paste runnable from a fresh clone. Frontier → `BOOK-EXAMPLES-RUNNABLE.2.2` (the `tests/book_examples.rs` harness + `mdbook test` + CI step — drift enforcement). Independent of the running Phase 6 `.2.4` gate.

**Files touched**

- Updated: book/src/{architecture,factorization,getting-started,hierarchy,ir,knobs,recipes}.md, docs/tasks/BOOK-EXAMPLES-RUNNABLE.md, docs/TASK_TREE.md, CHANGES.md, MEMORY.md.

## 2026-05-18-book-examples-runnable-2-split — BOOK-EXAMPLES-RUNNABLE.2 split into migration + enforcement

**Landed as:** 5cb6fb1

**What changed**

- `docs/tasks/BOOK-EXAMPLES-RUNNABLE.md`: `.2` split into `.2.1` (convention migration + rust-sketch annotation — docs across 7 chapters: `anvil ` → `cargo run --release -- `, one optional `cargo install` shorthand note, 8 `rust` blocks → `rust,ignore`, skip-sentinels where genuinely illustrative) and `.2.2` (`tests/book_examples.rs` extraction harness + `mdbook test` + `ci.yml` wiring + negative control). Dependency order: `.2.1` makes the book *correct* (every example runnable from a fresh clone), `.2.2` *enforces* it. `.2` is now a container; no node renumbered. Frontier `.2` → `.2.1`. Decisions/Changelog/Metadata + `docs/TASK_TREE.md` row updated.

**Why**

- Splitting Rules: docs migration vs test-harness code vs CI workflow-config are independently reviewable, and the book must be correct before it is enforced (a harness landed first would fail on the 62 un-migrated blocks).

**Validation**

- Tree-planning doc change only — **no code change**. `cargo`/`mdbook` unaffected.

**Impact**

- Frontier → `BOOK-EXAMPLES-RUNNABLE.2.1` (the ~62-block `cargo run --release --` migration — highest user value: makes every published example paste-runnable now). No behavioural change.

**Files touched**

- Updated: docs/tasks/BOOK-EXAMPLES-RUNNABLE.md, docs/TASK_TREE.md, CHANGES.md, MEMORY.md.

## 2026-05-18-book-examples-runnable-1 — BOOK-EXAMPLES-RUNNABLE.1: book-examples-runnable design + tree

**Landed as:** 38c49fb

**What changed**

- New quality task tree `docs/tasks/BOOK-EXAMPLES-RUNNABLE.md` (registered in `docs/TASK_TREE.md` "Active Task Trees"): every mdBook example must be copy-paste runnable from a fresh public clone and CI-enforced against drift.
- `DEVELOPMENT_NOTES.md`: design-only entry "Book-examples-runnable design (2026-05-18, BOOK-EXAMPLES-RUNNABLE.1)" — audited fenced-block inventory (62 `bash` / 8 `rust` / 9 `systemverilog` / 4 `text`; `recipes.md` 41 + `tutorial.md` 10 dominate the runnable surface; ~58 bare-`anvil` invocations to migrate; `getting-started.md` already uses `cargo run --release --`, the rest don't — the core defect). Owner decisions recorded: standardize on `cargo run --release --` (+ one optional `cargo install` shorthand note); CI-gated extraction harness + `mdbook test`. Chosen architecture: a `tests/book_examples.rs` cargo integration test (build-once, run every non-skipped `bash` block offline against the fresh binary with timeout + tagged sample-output match) + `mdbook test` with the 8 `rust` sketches annotated `rust,ignore`, both wired into `ci.yml`; an invisible HTML-comment skip sentinel (mandatory reason; default = run). Three rejected alternatives (doctest-only; CI-only `.sh`; golden-doc generation); `.2` proof shape incl. negative control + split candidates.
- Infra activation outcome (this session, commits `a612a5f`/`8076e25`): repo confirmed **PUBLIC** (an earlier `{owner}/{repo}`-resolved `gh` query returned a stale `PRIVATE`; explicit `gh repo view rdje/anvil` = `PUBLIC`); workflows pushed; **CI** workflow active (first run in progress — same commands as the local COMMIT.md gate); **Pages enabled** (source = GitHub Actions) and the mdBook is **live at https://rdje.github.io/anvil/** (deploy run succeeded; site returns HTTP 200).

**Why**

- Owner mandate: now that the repo is public and the book is the live user-facing surface, every example is a copy-paste contract; `BOOK-EXAMPLES-RUNNABLE.1` designs the convention + drift-proof enforcement. Continuous-PNT: advanced in parallel while the Phase 6 `.2.4` gate runs (fully independent).

**Validation**

- Design/docs only — **no code change** (leaf is design-only; task-tree owned by `BOOK-EXAMPLES-RUNNABLE.1`). `git diff` = `DEVELOPMENT_NOTES.md` + `docs/TASK_TREE.md` (+ new tree file); `mdbook build book` clean; `cargo fmt --all --check` clean; `cargo test` unchanged-green (no `src/`/`tests/` touched).

**Impact**

- A new tracked quality lane. Frontier → `BOOK-EXAMPLES-RUNNABLE.2` (implement: harness + ~62-block migration + CI wiring; expected to split). The mdBook is published and reachable; example-runnability enforcement lands in `.2`.

**Files touched**

- New: docs/tasks/BOOK-EXAMPLES-RUNNABLE.md. Updated: DEVELOPMENT_NOTES.md, docs/TASK_TREE.md, CHANGES.md, MEMORY.md.

## 2026-05-18-ci-pages-workflows — Add GitHub Actions CI + Pages workflows (repo public; none existed)

**Landed as:** a612a5f

**What changed**

- New `.github/` (first CI in this repo): `.github/workflows/ci.yml` — repo-owned fast hygiene gate mirroring the COMMIT.md checklist (`cargo fmt --all --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, `mdbook build book`) on push/PR to `main`, with cargo caching + `cancel-in-progress`. `.github/workflows/pages.yml` — builds the mdBook and deploys `book/book-out` to GitHub Pages (`upload-pages-artifact@v3` + `deploy-pages@v4`, `github-pages` environment), on `book/**` changes + `workflow_dispatch`.

**Why**

- Owner re-enabled Actions + Pages and made the repo public, asking to "re-enable whatever needs re-enabling". Investigation found **no `.github/` at all** — CI/Pages had to be created, not re-enabled. CI is deliberately scoped to the fast gates; the multi-hour Phase-4 hierarchy downstream gate (Verilator + both Yosys, ~9h) stays a local/manual repo-owned gate, never in CI.

**Validation**

- YAML authored to the documented Actions schema; `mdbook build book` verified clean locally; CI runs the exact COMMIT.md gate commands. Workflow-config only — **no code change**. **Not yet executing on GitHub**: `gh repo view` still reports `PRIVATE` and no Pages site exists — surfaced to the owner (the files are correct and inert until the repo is public + Pages source = "GitHub Actions").

**Impact**

- Once the repo is public with Actions + Pages source = GitHub Actions, every push to `main` runs the hygiene gate and republishes the user-facing mdBook. No generator behaviour change.

**Files touched**

- New: .github/workflows/ci.yml, .github/workflows/pages.yml. Updated: CHANGES.md, MEMORY.md.

## 2026-05-18-phase6-2.3 — PHASE-6-ADVANCED-MOTIFS.2.3: phase6_inferrable_memory matrix scenario + metric + gap

**Landed as:** ac18cd5

**What changed**

- `src/metrics.rs`: `DesignMetrics.num_memory_modules` (count of `Design::modules` with `!memories.is_empty()`), populated in `compute_design`.
- `src/bin/tool_matrix.rs`: new `phase6_inferrable_memory_focus_config` (a clone of `phase5b_packed_aggregate_focus_config` — depth-1 wrapper, library mode, **exact** dedup/phase5/5b anchor shape: 4 leaves / 4 instances, all hierarchy-routing probs 0.0 — so leaf/child/range/source shape-coverage sets are unperturbed; sole difference `memory_prob = 1.0`, so the rules-first library leaves are inferrable-memory blocks instantiated by the wrapper) + a `phase6_inferrable_memory` scenario tuple; `CoverageSummary.saw_inferrable_memory_design` (set when `config.memory_prob > 0 && num_memory_modules > 0`) + `merge_coverage` + a `Phase4Hierarchy` `compute_coverage_gaps` arm.
- Bin tests: `scenario_count` 216 → **219**, `total_modules` 864 → **876** (observed deterministically from the run, not guessed); exception-list entry `phase6_inferrable_memory`; `tool_matrix` phase4 bin tests 3/3. New `phase6_inferrable_memory_scenario_is_non_vacuous` bin test proves every `phase6_inferrable_memory` scenario builds ≥1 memory module across all strategies, so `saw_inferrable_memory_design` is reachable and `.2.4`'s gate cannot carry a permanent coverage gap.

**Why**

- `PHASE-6-ADVANCED-MOTIFS.2.3` (r87 no-aspirational-claims): land the gate scenario, metric, coverage fact and gap **before** any ROADMAP advance — that happens only at `.2.4` on a verified downstream-clean artifact.

**Validation**

- `cargo fmt --all --check` clean; `cargo clippy --all-targets -- -D warnings` clean; `tool_matrix` phase4 bin tests 3/3; non-vacuity test green (3/3 strategies build a memory module); full `cargo test` (COMMIT.md gate — see Verification Log). **ROADMAP unchanged** (no phase label change — that is `.2.4`). No `book/` change (reconciliation is `.2.4`).

**Impact**

- The Phase 4 hierarchy matrix now carries a `phase6_inferrable_memory` scenario that exercises inferrable memory downstream; `.2.4` will run the real gate and, only on a clean verified artifact, record memory **delivered** in ROADMAP Phase 6 (Phase 6 stays open for the `.3` FSM motif — no tree closure). Frontier → `PHASE-6-ADVANCED-MOTIFS.2.4`.

**Files touched**

- Updated: src/metrics.rs, src/bin/tool_matrix.rs, docs/tasks/PHASE-6-ADVANCED-MOTIFS.md, docs/TASK_TREE.md, CHANGES.md, MEMORY.md.

## 2026-05-18-phase6-2.2 — PHASE-6-ADVANCED-MOTIFS.2.2: memory inference structural-contract + factorization-opacity proof

**Landed as:** f4ee02f

**What changed**

- `tests/pipeline.rs`: new `inferrable_memory_matches_yosys_template_and_is_factorization_opaque`. Across 4 `ConstructionStrategy` × 4 `FactorizationLevel` (`None`/`Cse`/`Commutative`/`EGraph`) × 4 seeds (64 combinations) with `memory_prob = 1.0`: `validate_design` clean; the emitted SV is **exactly** the `.1`-validated Yosys-`$mem_v2` template (concrete-depth `mem_0 [0:depth]` array, reset-less `always_ff @(posedge clk)`, `if (we) mem_0[..] <= wdata;`, `memrd_0 <= mem_0[..]`); exactly one `MemRead` survives every factorization level and the memory leaf has **zero** expression-graph `Gate` nodes (the array / `MemRead` never enter the NodeId graph — CSE/factorization-opaque, including the strongest `EGraph`).

**Why**

- `PHASE-6-ADVANCED-MOTIFS.2.2`: formalize the inference contract + identity-invariance. **Scoping (recorded in the tree Decisions):** the cargo gate cannot shell out to Yosys/Verilator — this repo proves downstream-tool cleanliness only via the repo-owned `tool_matrix` gate (`.2.3` scenario + `.2.4` real run), never `cargo test`. The cargo-portable formalization of the contract is the structural-template equivalence (the generator emits *exactly* the `.1`-proven inferrable template, which *is* the contract); the tool-level `$mem_v2`/Verilator proof is `.2.1b`'s real binary spot-check (interim) + `.2.4`'s authoritative real gate (r87 no-aspirational-claims).

**Validation**

- `cargo fmt --all --check` clean; `cargo clippy --all-targets -- -D warnings` clean; the new proof green (64 combos); full `cargo test` (COMMIT.md gate — see Verification Log). **Proof only — no feature code change**; default-off byte-identical reaffirmed by the unchanged `.2.1b` proof. No `book/` change.

**Impact**

- The Phase 6 memory-inference contract + `MemRead` factorization-opacity are now reproducibly asserted in the cargo gate across all strategies/factorization levels. Phase 6 frontier → `PHASE-6-ADVANCED-MOTIFS.2.3` (matrix scenario + metric + gap; no ROADMAP advance until the verified gate at `.2.4`).

**Files touched**

- Updated: tests/pipeline.rs, docs/tasks/PHASE-6-ADVANCED-MOTIFS.md, docs/TASK_TREE.md, CHANGES.md, MEMORY.md.

## 2026-05-18-phase6-2.1b — PHASE-6-ADVANCED-MOTIFS.2.1b: memory_prob knob + rules-first build_memory_leaf

**Landed as:** aa9abf0

**What changed**

- `src/config.rs`: `Config::memory_prob: f64` (`#[serde(default = "default_memory_prob")]` = 0.0 + Default-impl line + probability-range validation tuple entry), mirroring `aggregate_prob` / `width_parameterization_prob`.
- `src/gen/module.rs`: rules-first `build_memory_leaf` — a single-`Memory` leaf with shared `clk`/`rst_n`, `we`/`waddr`/`wdata` (+ independent `raddr` for `SimpleDualPort`) inputs, one `rdata` output driven by the opaque `Node::MemRead`; `MemKind`, `addr_width` (2..=4) and `data_width` (configured width band) rolled via `g.rng`; no gates/flops (the memory is the only state). A single opt-in roll in `generate_leaf_module_with_interface_profile`, placed **after** the Phase 5 parameterization lane (mutually exclusive; `interface_profile` None only; default-off never enters → byte-identical).
- `tests/pipeline.rs`: `inferrable_memory_is_default_off_and_constructs_when_forced_on`.

**Why**

- `PHASE-6-ADVANCED-MOTIFS.2.1b`: make the memory motif actually fire by rule (valid-by-construction, not generate-then-filter), opt-in and default-off byte-identical — completing the `.2.1` scaffold container. Continuous-PNT.

**Validation**

- `cargo fmt --all --check` clean; `cargo clippy --all-targets -- -D warnings` clean; focused proof green (default-off byte-identical across 4 `ConstructionStrategy` × 6 seeds; forced-on every single-module design is a 1-`Memory` leaf that `validate_design`-passes, exposes a `MemRead`, and emits the inferrable array + reset-less `always_ff` write/read); full `cargo test` (COMMIT.md gate — see Verification Log). **Real generated-output spot-check** (binary, seed 3, `memory_prob = 1.0`): emitted SV `verilator --lint-only` exit 0; yosys `proc;opt;memory_collect` → **`1 $mem_v2`**; `synth -noabc` and `synth; abc -fast` both `check -assert` clean — the Phase 6 inference contract holds on real generated output (formally proven in `.2.2`). No `book/` change.

**Impact**

- Opt-in `memory_prob > 0` makes the single-module lane emit inferrable memory; default-off ⇒ byte-identical. `.2.1` container closed. Phase 6 frontier → `PHASE-6-ADVANCED-MOTIFS.2.2`.

**Files touched**

- Updated: src/config.rs, src/gen/module.rs, tests/pipeline.rs, docs/tasks/PHASE-6-ADVANCED-MOTIFS.md, docs/TASK_TREE.md, CHANGES.md, MEMORY.md.

## 2026-05-18-phase6-2.1a — PHASE-6-ADVANCED-MOTIFS.2.1a: memory IR core + opaque-stateful-leaf pipeline integration

**Landed as:** 244cabd

**What changed**

- **IR core** (`src/ir/types.rs`): `MemId`; `MemKind{SinglePort,SimpleDualPort}`; `Memory{id,addr_width,data_width,kind,we,waddr,wdata,raddr}`; additive `Default`-empty `Module.memories: Vec<Memory>`; new **opaque leaf** `Node::MemRead{mem,width}`; `DepAtom::MemVirtual(MemId)` + `DepSet::from_mem_virtual`; `Module::has_local_memories()` OR'd into the sequential-state predicates so a memory-only module exposes `clk` (`has_local_flops` left untouched — flop emission gates unchanged).
- **Pipeline integration**: `MemRead` threaded through all ~21 exhaustive `Node` matches (compiler-as-completeness-oracle) mirroring `FlopQ` as an opaque, identity-by-instance leaf — `gen/cone.rs`, `gen/hierarchy.rs`, `gen/module.rs`, `ir/param.rs`, `metrics.rs` (incl. `canonical_module_signature` tag 6), `ir/compact.rs`. **Load-bearing correctness**: the `compact.rs` reachability walk gains a `Node::MemRead` arm that marks `mem.{we,waddr,wdata,raddr}` reachable (like `FlopQ` keeps its flop's D cone), so memory input cones survive dead-elimination; `StructuralNodeShape::MemRead` + `LeafEndpoint::MemRead` (+ `width()`). Memories are never dead-eliminated in 6.2.1 ⇒ stable `MemId`, no remap machinery.
- **Emitter** (`src/emit/sv.rs`): `memrd_<id>` helper + `node_ref` arm; per-memory `logic [DW-1:0] mem_<id> [0:2^AW-1];` + `logic [DW-1:0] memrd_<id>;` declarations + a reset-less `always_ff @(posedge clk)` synchronous write/read block — the `.1`-validated Yosys-`$mem_v2` template.
- **Validator** (`src/ir/validate.rs`): new `BadMemory`/`UndefinedMemoryNode`/`MemoryNodeWidthMismatch`/`DanglingMemRead`/`MemReadWidthMismatch` + a step-5b that checks every `Memory`'s widths + `SinglePort` shared-address invariant + every `MemRead` resolves at the right width.
- **Proofs** (`src/ir/compact.rs` tests): `memory_leaf_roundtrips_validate_and_emit`, `memread_keeps_memory_source_cones_through_compaction` (the reachability proof — dead gate stripped, `wdata` XOR cone survives, validate+emit clean), `memread_is_structurally_distinct_and_not_cse_merged`.

**Why**

- `PHASE-6-ADVANCED-MOTIFS.2.1a`: land the IR core + the correctness-critical opaque-stateful-leaf pipeline integration (the discovered compaction-reachability dependency) atomically, with unit proofs, *before* the knob/generator (`.2.1b`). Continuous-PNT.

**Validation**

- `cargo fmt --all --check` clean; `cargo clippy --all-targets -- -D warnings` clean; `ir::compact::tests` mem 3/3; full `cargo test` (COMMIT.md gate — see Verification Log). **No generator/knob** ⇒ no `Memory` is ever constructed ⇒ default-off is *trivially* byte-identical (emit unchanged for all real generation). No `book/` change (reconciliation is `.2.4`).

**Impact**

- The IR can now represent inferrable memory and the pipeline handles the opaque stateful leaf correctly; nothing constructs a `Memory` yet. Phase 6 frontier → `PHASE-6-ADVANCED-MOTIFS.2.1b` (knob + rules-first `build_memory_leaf` + focused proof).

**Files touched**

- Updated: src/ir/types.rs, src/ir/compact.rs, src/ir/validate.rs, src/emit/sv.rs, src/gen/cone.rs, src/gen/hierarchy.rs, src/gen/module.rs, src/ir/param.rs, src/metrics.rs, docs/tasks/PHASE-6-ADVANCED-MOTIFS.md, docs/TASK_TREE.md, CHANGES.md, MEMORY.md.

## 2026-05-18-phase6-2.1-split — PHASE-6-ADVANCED-MOTIFS.2.1 split on a discovered compaction-reachability dependency

**Landed as:** 4ad089b

**What changed**

- `docs/tasks/PHASE-6-ADVANCED-MOTIFS.md`: `.2.1` (IR+leaf+knob+emitter+validator scaffold) split into `.2.1a` (IR core: `MemId`/`MemKind`/`Memory`/`Node::MemRead`/`Module.memories`/`DepAtom::MemVirtual`; thread `MemRead` through all ~20 exhaustive `Node` matches; **the load-bearing `compact.rs` reachability integration so a reachable `MemRead` transitively keeps the memory's `we`/`waddr`/`wdata`/`raddr` cones alive, like `FlopQ` keeps its flop's D cone**; emitter inferrable template + validator + control-port clk; unit proofs incl. a compaction-reachability test and CSE-opacity; no generator/knob → default-off trivially byte-identical) and `.2.1b` (`Config::memory_prob` knob + rules-first `build_memory_leaf` + opt-in roll + default-off/forced-on focused proof). `.2.1` is now a container; `.2.2`/`.2.3`/`.2.4` and `.3` unchanged; no renumbering. Frontier `.2.1` → `.2.1a`. Decisions/Changelog/Metadata + `docs/TASK_TREE.md` row updated.

**Why**

- Implementing `.2.1` surfaced a genuine **discovered lower-level dependency**: a new opaque *stateful* leaf is not mechanical `FlopQ`-mirroring — `src/ir/compact.rs` reachability/dead-elimination must transitively keep a memory's input cones alive, which is correctness-critical and must not be rushed into one slice alongside the knob/generator/proof. Per the Splitting Rules ("cannot be completed to signoff in one slice"; "discovers a lower-level dependency that should be solved first"). In-flight IR-core edits (`src/ir/types.rs` only) were reverted to the clean `.2`-split base (`c96b433`) so `.2.1a` lands atomically from a clean tree.

**Validation**

- Tree-planning doc change only — **no code change** (in-flight edits reverted; `git status` clean of `src/` before this commit). `cargo` gates unaffected/green; `mdbook` unaffected.

**Impact**

- Phase 6 frontier is now `PHASE-6-ADVANCED-MOTIFS.2.1a` (IR core + the correctness-critical compaction-reachability integration). No behavioural change.

**Files touched**

- Updated: docs/tasks/PHASE-6-ADVANCED-MOTIFS.md, docs/TASK_TREE.md, CHANGES.md, MEMORY.md.

## 2026-05-18-phase6-2-split — PHASE-6-ADVANCED-MOTIFS.2 split into signoff-sized leaves

**Landed as:** c96b433

**What changed**

- `docs/tasks/PHASE-6-ADVANCED-MOTIFS.md`: `.2` ("implement the memory motif per `.1`") was a single oversized leaf (new IR `Memory` block + opaque `MemRead` leaf + `memory_prob` knob + emitter inferrable-template + validator + matrix gate). Split into a container with `.2.1` (IR+leaf+knob+emitter+validator scaffold, default-off byte-identical), `.2.2` (Yosys-inference proof on *generated* output — `memory_collect` ≥1 `$mem_v2` both modes + Verilator-clean — plus `MemRead` CSE-opacity), `.2.3` (tool_matrix `phase6_inferrable_memory` scenario + `num_memory_modules` metric + `saw_inferrable_memory_design` fact/gap + non-vacuity, **no** ROADMAP advance), `.2.4` (run the real gate, verify clean, then record memory **delivered** in ROADMAP Phase 6 — Phase 6 stays open until the `.3` FSM motif lands, so no tree closure). `.2` is now a container; `.3` (FSM) unchanged; no node renumbered. Frontier `.2` → `.2.1`.
- `docs/TASK_TREE.md`: `PHASE-6-ADVANCED-MOTIFS` frontier row `.1`→`.2.1`.

**Why**

- Splitting Rules: `.2` cannot reach signoff in one slice and mixes independently-reviewable concerns; r87 no-aspirational-claims requires the matrix-gate scenario (`.2.3`) before any ROADMAP advance (`.2.4` on a verified artifact). Decomposition mirrors the proven Phase 5/5b `.2.x` shape.

**Validation**

- Tree-planning doc change only — **no code change**. `cargo` gates unaffected/green (no `src/`/`tests/` touched); `mdbook` unaffected (no `book/` change).

**Impact**

- Phase 6 memory-implementation frontier is now `PHASE-6-ADVANCED-MOTIFS.2.1` (the reviewable IR scaffold). No behavioural change.

**Files touched**

- Updated: docs/tasks/PHASE-6-ADVANCED-MOTIFS.md, docs/TASK_TREE.md, CHANGES.md, MEMORY.md.

## 2026-05-18-phase6-1 — PHASE-6-ADVANCED-MOTIFS.1: inferrable-memory motif design

**Landed as:** ab491a8

**What changed**

- `DEVELOPMENT_NOTES.md`: new design-notes entry "Phase 6 inferrable-memory motif design (2026-05-18, PHASE-6-ADVANCED-MOTIFS.1)". Codebase-grounded (the IR has no array/memory concept — scalar `u32` `Port`/`Node`/`Flop`; `Flop` is the only stateful element; operators-vs-blocks doctrine → a memory is a *block*). Includes an **empirical Yosys-inference probe**: single-port sync RAM and simple dual-port templates both yield exactly `1 $mem_v2` under `proc;opt;memory_collect`, Verilator `--lint-only` exit 0, and `synth -noabc` / `synth;abc -fast` both exit 0 with `check -assert` (clean in both repo Yosys modes). Chosen architecture **(M)**: a first-class `Memory` block (additive `Vec<Memory>` on `Module`, Default-empty) + an opaque `Node::MemRead` leaf (sibling to `FlopQ`, never CSE'd) + emitter renders the validated inferrable template on the shared `clk` + opt-in `Config::memory_prob` (serde-default `0.0`). Three rejected alternatives: (A) flop-array+mux (not `$mem`-inferred — defeats the purpose), (B) emitter-only string template (not valid-by-construction), (C) generic unpacked-array datatype threaded through width arithmetic (massive invasive change; memory is a block, not a datatype). Proof shape for `.2` specified (Phase 5/5b `.2.x` decomposition + r87 no-aspirational-claims).
- `docs/tasks/PHASE-6-ADVANCED-MOTIFS.md`: `.1` `pending`→`done`, Open Question resolved (the empirical probe), frontier `.1`→`.2`, Decisions/Verification-Log/Commit-Log/Changelog/Metadata. `docs/TASK_TREE.md`: `PHASE-6-ADVANCED-MOTIFS` frontier row `.1`→`.2`.

**Why**

- `PHASE-6-ADVANCED-MOTIFS.1` (design-only leaf): give `.2` an unambiguous, codebase-grounded, empirically-validated implementation target with the rejected-alternatives trail recorded durably. Continuous-PNT: Phase 5b closed, loop advanced to the next open tree.

**Validation**

- Design/docs only — **no code change** (leaf is design-only; task-tree owned by `PHASE-6-ADVANCED-MOTIFS.1`, doctrine-compliant). `mdbook build book` clean; `cargo fmt --all --check` clean; `cargo test` unchanged-green (no `src/`/`tests/` touched since Phase 5b `.2.3`'s green run).

**Impact**

- Phase 6 frontier advances to `PHASE-6-ADVANCED-MOTIFS.2` (implement architecture (M); expected to split `.2.x`). No behavioural change.

**Files touched**

- Updated: DEVELOPMENT_NOTES.md, docs/tasks/PHASE-6-ADVANCED-MOTIFS.md, docs/TASK_TREE.md, CHANGES.md, MEMORY.md.

## 2026-05-18-phase5b-2.4 — PHASE-5B-AGGREGATES.2.4: real-gate verify → ROADMAP Phase 5b (not started)→(done) + tree closure

**Landed as:** 957b1aa

**What changed**

- **Phase 5b — Synthesizable aggregates is promoted `done`** (phase-label change `(not started)` → `(done)`). `ROADMAP.md` Phase 5b: header `(not started)`→`(done)`, `**Status:** done as of 2026-05-18`, an explicit 3-point **"Exit criteria (met)"** block tied to the verified gate artifact, plus a scope note (StructPacked-only / non-instantiated / `param_env`-skipped are open-ended post-phase sub-slices — deliberate scope cut, not blockers; no mode retired).
- `docs/tasks/PHASE-5B-AGGREGATES.md`: `.2.4` `pending`→`done` (Verification + Commit filled), `.2.4`/`.2` container and the `PHASE-5B-AGGREGATES` root `active`→`done`, Current Frontier → none, Metadata, Verification/Commit-Log/Changelog `.2.4` rows.
- `docs/TASK_TREE.md`: `PHASE-5B-AGGREGATES` row `active`→`done`.
- Book reconciled: `book/src/ir.md` "Synthesizable aggregates" gains a **Delivered (Phase 5b)** note; `book/src/knobs.md` documents `aggregate_prob` (and the previously-undocumented `width_parameterization_prob`) in the config-only knob list + the knob-effectiveness map. `mdbook build book` clean.
- Synced live docs: `README.md` (Phase 5b done; next is Phase 6), `CODEBASE_ANALYSIS.md` (new `5b — Synthesizable aggregates` phase-coverage row → done), `MEMORY.md` (Phase line → Phase 5b done, Phase 6 next).

**Why**

- `.2.4`'s contract (r87 no-aspirational-claims): a phase is promoted **only** after a real repo-owned downstream gate is run and verified clean — promotion strictly follows the verified artifact.

**Validation**

- Real repo-owned `Phase4Hierarchy` gate (now including `phase5b_packed_aggregate`) ran to completion (background `bifczmcw7`, exit 0). Banked `/tmp/anvil-tool-matrix-phase5b-p1/tool_matrix_report.json` verified CLEAN: `scenario_count` 216, `total_modules` 864, `coverage_gaps` `[]`, Verilator 864/0, `yosys_without_abc` 864/0, `yosys_with_abc` 864/0, `saw_packed_aggregate_design` `true`, `saw_width_parameterized_design` `true` (Phase 5 regression), `saw_recursive_hierarchy_module_dedup_active` `true` (Phase 4 regression).
- Gate-run + docs/closure only — **no code change** in this leaf. `cargo` gates green from `.2.3` (`6fabd7e`); `mdbook build book` clean.

**Impact**

- Phase 5b is closed. The next numbered roadmap phase is **Phase 6 — Advanced motifs** (`docs/tasks/PHASE-6-ADVANCED-MOTIFS.md`, frontier `PHASE-6-ADVANCED-MOTIFS.1`). No behavioural change to generated RTL (`aggregate_prob` remains default-off / byte-identical).

**Files touched**

- Updated: ROADMAP.md, README.md, CODEBASE_ANALYSIS.md, MEMORY.md, book/src/ir.md, book/src/knobs.md, docs/tasks/PHASE-5B-AGGREGATES.md, docs/TASK_TREE.md, CHANGES.md.

## 2026-05-17-phase5b-2.3 — PHASE-5B-AGGREGATES.2.3: packed_aggregate matrix scenario + metrics + gap

**Landed as:** 6fabd7e

**What changed**

- `src/metrics.rs`: `DesignMetrics.num_packed_aggregate_modules` (count of `Design::modules` with `aggregate_layout.is_some()`), populated in `compute_design`.
- `src/bin/tool_matrix.rs`: new `phase5b_packed_aggregate_focus_config` (depth-1 wrapper, library mode, `aggregate_prob = 1.0`, shaped **exactly** like the phase5 / dedup anchor — 4 leaves / 4 instances, all hierarchy-routing probs 0.0 — so the matrix leaf/child/range/source shape-coverage sets are unperturbed; the never-instantiated top wrapper is projected, leaves stay flat per the `.2.1` scaffold scope) + a `phase5b_packed_aggregate` scenario tuple; `CoverageSummary.saw_packed_aggregate_design` (set when `config.aggregate_prob > 0 && num_packed_aggregate_modules > 0`) + `merge_coverage` + a `Phase4Hierarchy` `compute_coverage_gaps` arm.
- Bin tests: `scenario_count` 213 → **216**, `total_modules` 852 → **864** (observed deterministically from the run, not guessed); exception-list entry `phase5b_packed_aggregate`; `tool_matrix` phase4 bin tests 3/3. New `phase5b_packed_aggregate_scenario_is_non_vacuous` bin test proves every `phase5b_packed_aggregate` scenario projects ≥ 1 module (the top wrapper) across all strategies, so `saw_packed_aggregate_design` is reachable and `.2.4`'s gate cannot carry a permanent coverage gap.

**Why**

- `PHASE-5B-AGGREGATES.2.3` (r87 no-aspirational-claims discipline): land the gate scenario, metric, coverage fact and gap **before** any ROADMAP promotion. Promotion happens only at `.2.4` on a verified downstream-clean artifact.

**Validation**

- `cargo fmt --all --check` clean; `cargo clippy --all-targets -- -D warnings` clean; `tool_matrix` phase4 bin tests 3/3; non-vacuity test green (3/3 strategies project the wrapper); full `cargo test` (COMMIT.md gate — see Verification Log). **ROADMAP unchanged** (no phase label change — that is `.2.4`). No `book/` change (reconciliation is `.2.4`).

**Impact**

- The Phase 4 hierarchy matrix now carries a `phase5b_packed_aggregate` scenario that exercises the packed-aggregate projection downstream; `.2.4` will run the real gate and, only on a clean verified artifact, promote ROADMAP Phase 5b → done. Frontier → `PHASE-5B-AGGREGATES.2.4`.

**Files touched**

- Updated: src/metrics.rs, src/bin/tool_matrix.rs, docs/tasks/PHASE-5B-AGGREGATES.md, docs/TASK_TREE.md, CHANGES.md, MEMORY.md.

## 2026-05-17-phase5b-2.2 — PHASE-5B-AGGREGATES.2.2: organic-existence proof + identity-invariance

**Landed as:** d0d7ad6

**What changed**

- `tests/pipeline.rs`: new `packed_aggregate_organic_existence_is_not_inert` — with **DEFAULT** port ranges (no forcing) and `aggregate_prob = 1.0`, sweeps 4 `ConstructionStrategy` × 20 seeds and asserts the projection fires on a substantial fraction of *organic* single-module designs. Observed **68/80 ≈ 85 %**; durable threshold pinned at ≥ 50 %.
- `src/ir/aggregate.rs`: two new unit tests — `canonical_signature_is_invariant_under_projection` (signature + flat ports identical before/after `annotate_aggregate`) and `aggregate_projected_twin_dedup_collapses` (a concrete module and its aggregate-projected twin share `canonical_module_signature` and `dedup_modules` collapses them under a top, `removed == 1`). `aggregate::` 8 → 10.

**Why**

- `PHASE-5B-AGGREGATES.2.2`: (a) prove the `.2.1` post-construction pass is **not inert** on organic output (the Phase-5 rules-first-pivot guard) — and it is robustly not, unlike Phase 5 width-homogeneity, so **no rules-first eligible-interface constructor is added** (decision recorded in the tree). (b) prove in code the identity Open Question: the annotation is not hashed into module identity, so projection is semantically a no-op.

**Validation**

- `cargo fmt --all --check` clean; `cargo clippy --all-targets -- -D warnings` clean; `aggregate::` 10/10; existence sweep 68/80; full `cargo test` (COMMIT.md gate — see Verification Log). Proofs only — **no change to the feature code**; default-off byte-identical unaffected.

**Impact**

- Phase 5b frontier → `PHASE-5B-AGGREGATES.2.3` (tool_matrix scenario + metrics + coverage gap; no ROADMAP promotion until the verified gate at `.2.4`). No behavioural change.

**Files touched**

- Updated: tests/pipeline.rs, src/ir/aggregate.rs, docs/tasks/PHASE-5B-AGGREGATES.md, docs/TASK_TREE.md, CHANGES.md, MEMORY.md.

## 2026-05-17-phase5b-2.1 — PHASE-5B-AGGREGATES.2.1: packed-aggregate IR annotation + knob + emitter projection

**Landed as:** 67e909d

**What changed**

- **IR** (`src/ir/types.rs`): additive `Default`-able `Module.aggregate_layout: Option<AggregateLayout>` + `AggregateKind { StructPacked }` + `AggregateGroup { type_name, port_name, fields: Vec<(String, PortId)> }`. Zero churn to `..Module::default()` sites (mirrors the Phase 5 `param_env` additive pattern). New `pub mod aggregate;` in `src/ir/mod.rs`.
- **Knob** (`src/config.rs`): `Config::aggregate_prob: f64`, `#[serde(default = "default_aggregate_prob")]` (= 0.0), Default-impl line, and a probability-range validation tuple entry — identical pattern to `width_parameterization_prob`.
- **Pass** (`src/ir/aggregate.rs`, new): `annotate_aggregate` — non-rolling, idempotent, skips `param_env` modules, builds an input group from `emitted_data_input_ports()` (clk/rst_n excluded) and an output group from `outputs`, each requiring ≥2 ports. 6 unit tests.
- **Generator** (`src/gen/mod.rs`): post-pass after dedup + the param pass; per-module Bernoulli roll via the seeded ChaCha8 RNG (reproducible; never `thread_rng`), scoped to **non-instantiated** modules so hierarchy child connections stay byte-identical.
- **Emitter** (`src/emit/sv.rs`): boundary-alias projection — `typedef struct packed { … } <name>;` before the module; grouped ports replaced by one aggregate port per side; input fields aliased to same-named `wire`s; grouped outputs become internal `logic` + a boundary `assign <agg>.<field> = <name>;`. The flat IR body emission is byte-for-byte unchanged.
- **Proof** (`tests/pipeline.rs`): `packed_aggregate_is_default_off_and_projects_when_forced_on`.

**Why**

- `PHASE-5B-AGGREGATES.2.1`: the reviewable scaffold for architecture (P) (design `.1`), default-off byte-identical, before the organic-existence/identity proof (`.2.2`), the matrix scenario (`.2.3`), and verified ROADMAP promotion (`.2.4`). Continuous-PNT mode.

**Validation**

- `cargo fmt --all --check` clean; `cargo clippy --all-targets -- -D warnings` clean; `aggregate::` 6/6; focused proof green (default-off byte-identical across 4 `ConstructionStrategy` × 6 seeds; forced-on projects every single-module design, `validate_design` clean, SV declares `typedef struct packed` + aggregate port + boundary alias/assign, IR shape unchanged); full `cargo test` (COMMIT.md gate — see Verification Log). Real generator-output spot-check: a projected hierarchy design (top `mod_5_0001`, packed-struct output port) `verilator --lint-only` → EXIT 0 (clean).
- Scaffold scoping (recorded in the tree Decisions, not silent): `AggregateKind::StructPacked` only (union/array need same-width groups — later sub-slice); non-instantiated modules only (parent-side aggregate connection deferred); `param_env` modules skipped (param/aggregate cross-product deferred). `aggregate_prob == 0` (default) keeps every feature off and output byte-identical.

**Impact**

- Opt-in `aggregate_prob > 0` projects eligible non-instantiated modules' data ports as a packed `struct`. Default-off ⇒ no behavioural change. Phase 5b frontier → `PHASE-5B-AGGREGATES.2.2`.

**Files touched**

- New: src/ir/aggregate.rs. Updated: src/ir/types.rs, src/ir/mod.rs, src/config.rs, src/gen/mod.rs, src/emit/sv.rs, tests/pipeline.rs, docs/tasks/PHASE-5B-AGGREGATES.md, docs/TASK_TREE.md, CHANGES.md, MEMORY.md.

## 2026-05-17-phase5b-2-split — PHASE-5B-AGGREGATES.2 split into signoff-sized leaves

**Landed as:** 3fbbc79

**What changed**

- `docs/tasks/PHASE-5B-AGGREGATES.md`: `.2` ("implement the projection per `.1`") was a single oversized leaf mixing IR + knob + emitter + tests + matrix gate + ROADMAP promotion. Split it into a container with `.2.1` (IR `AggregateLayout` annotation + `Config::aggregate_prob` knob + emitter `typedef … packed` projection, default-off byte-identical), `.2.2` (soundness / organic-existence proof — with the Phase-5 rules-first-pivot fallback if the unconstrained generator is inert — + identity-invariance unit test), `.2.3` (tool_matrix scenario + metrics + `saw_packed_aggregate_design` gap, **no** ROADMAP promotion), `.2.4` (run the real repo-owned gate, verify downstream-clean, then ROADMAP Phase 5b → done + book reconciliation + tree closure). `.2` is now a container; no node renumbered. Frontier `.2` → `.2.1`. Decisions + Changelog + Metadata updated.
- `docs/TASK_TREE.md`: `PHASE-5B-AGGREGATES` frontier row `.2` → `.2.1`.

**Why**

- Splitting Rules: `.2` cannot reach signoff in one slice and mixes independently-reviewable concerns. The r87 no-aspirational-claims precedent additionally requires the matrix-gate scenario to land (`.2.3`) before any ROADMAP promotion (`.2.4` on a verified artifact). Decomposition mirrors the proven Phase 5 `.2.1`–`.2.4` shape.

**Validation**

- Tree-planning doc change only — **no code change**. `cargo` gates unaffected/green (no `src/`/`tests/` touched); `mdbook` unaffected (no `book/` change).

**Impact**

- Phase 5b implementation frontier is now `PHASE-5B-AGGREGATES.2.1` (the reviewable scaffold). No behavioural change.

**Files touched**

- Updated: docs/tasks/PHASE-5B-AGGREGATES.md, docs/TASK_TREE.md, CHANGES.md, MEMORY.md.

## 2026-05-17-phase5b-1 — PHASE-5B-AGGREGATES.1: packed-aggregate emitter-projection design

**Landed as:** 6976346

**What changed**

- `DEVELOPMENT_NOTES.md`: new design-notes entry "Phase 5b packed-aggregate emitter projection design (2026-05-17, PHASE-5B-AGGREGATES.1)". Codebase-grounded (file-anchored audit of `src/emit/sv.rs` dumb-serialiser chokepoints, `src/ir/types.rs` flat `Port`/`u32`-width reality, the Phase 5 `param_env` annotation precedent). Chosen architecture **(P)**: emitter-only packed-aggregate projection driven by an additive `Default`-able per-module `AggregateLayout` annotation — construction/validate/CSE/dedup all unchanged, bijective bit-layout-preserving regrouping, opt-in `aggregate_*_prob` serde-default `0.0` (byte-identical). Three rejected alternatives: (A) first-class aggregate IR nodes, (B) post-hoc SV text rewrite, (C) unpacked aggregates/enums now. Resolves the tree's identity Open Question (annotation **not** hashed into `canonical_module_signature` — opposite of Phase 5, because aggregates change nothing semantic; projected twin dedup-collapses, correct). Proof shape for `.2` specified.
- `docs/tasks/PHASE-5B-AGGREGATES.md`: `.1` `pending`→`done` (Verification + Commit filled), frontier `.1`→`.2`, identity Open Question resolved, Decisions/Verification-Log/Commit-Log/Changelog/Metadata updated.
- `docs/TASK_TREE.md`: `PHASE-5B-AGGREGATES` frontier row `.1`→`.2`.

**Why**

- `PHASE-5B-AGGREGATES.1` (design-only leaf): give `.2` an unambiguous, codebase-grounded implementation target with the rejected-alternatives trail recorded durably (not just in a commit message). Continuous-PNT mode: Phase 5 closed, so the loop advanced to the next open tree.

**Validation**

- Design/docs only — **no code change** (leaf is design-only; task-tree owned by `PHASE-5B-AGGREGATES.1`, doctrine-compliant). `cargo fmt`/`clippy -D warnings`/`check` green; `cargo test` unchanged-green (no `src/`/`tests/` touched since the `b5cto7m8m` exit-0 run); `mdbook build book` clean.

**Impact**

- Phase 5b frontier advances to `PHASE-5B-AGGREGATES.2` (implementation: `AggregateLayout` annotation + `aggregate_*_prob` knob + emitter `typedef … packed` projection + matrix scenario + downstream-clean proof, then ROADMAP Phase 5b → done). No behavioural change.

**Files touched**

- Updated: DEVELOPMENT_NOTES.md, docs/tasks/PHASE-5B-AGGREGATES.md, docs/TASK_TREE.md, CHANGES.md, MEMORY.md.

## 2026-05-17-gitignore-claude-runtime — Ignore Claude Code harness runtime artifacts

**Landed as:** 80516ca

**What changed**

- `.gitignore`: added `.claude/scheduled_tasks.lock`, `.claude/worktrees/`, and `.claude/settings.local.json` under a comment clarifying that the committed team-wide config is `.claude/settings.json` (tracked) while those three are per-machine/per-session runtime or personal overrides that must never be committed.

**Why**

- After the Phase 5 / doctrine commits, `git status` was clean for all tracked files but still listed `.claude/scheduled_tasks.lock` and `.claude/worktrees/` as untracked harness runtime. These are Claude Code session artifacts, not project state. `.claude/settings.local.json` was previously excluded only via a non-shared mechanism (`.git/info/exclude`/global); making the ignore explicit in-repo is portable for teammates and prevents an accidental future commit of personal settings (the recommended hygiene when a project commits `.claude/settings.json`).

**Validation**

- `git check-ignore -v` confirms all three patterns match. `git ls-files .claude/` still lists `.claude/settings.json` (tracked config preserved). `git status --porcelain` now shows only the `.gitignore` edit; no other untracked entries. Workflow/config only — **no code change** (consistent with the task-tree doctrine's code/not-code boundary; cargo gates unaffected and green from `53e4c7f`/`b3c1906`, `cargo test` exit 0 at `b5cto7m8m` — zero `src/`/`tests/`/build files touched).

**Impact**

- `git status` is now fully spotless (no stray untracked harness files). No behavioural change.

**Files touched**

- Updated: .gitignore, CHANGES.md, MEMORY.md.

## 2026-05-17-task-tree-mandatory-doctrine — Task-tree ownership is mandatory for all code changes

**Landed as:** b3c1906

**What changed**

- New **non-negotiable doctrine (owner directive)**: it is strictly forbidden to make any code change without it being task-tree tracked or task-tree owned first. Recorded across the live-docs and the live-book:
  - `docs/TASK_TREE.md` "ANVIL Adoption Scope" rewritten from "opt-in per top-level task" to the mandate, with an explicit code/not-code boundary and the `rN`-survives-only-within-a-leaf rule.
  - `COMMIT.md` "Task-tree-managed commits": closing paragraph rewritten to state the mandate; non-task-tree commits are now limited to non-code (docs/mdBook/workflow/doctrine).
  - `SESSION_BOOTSTRAP.md`: new "Non-negotiable doctrine: task-tree ownership of code" section so every recovering session enforces it immediately.
  - `DEVELOPMENT_NOTES.md` "Workflow notes": new dated entry with the rationale (task-tree ownership *measurably* improved review and code quality; supersedes the earlier opt-in/`rN` scope).
  - `README.md`: the `docs/TASK_TREE.md` doc-index line updated from "Opt-in per top-level task" to the mandate.
  - `book/src/architecture.md`: new "Development doctrine: task-tree ownership of code" section (the contributor-facing Reference chapter).
- Session memory: `feedback_task_tree_available.md` rewritten from "adopt at your own pace" to the mandate (slug `task-tree-mandatory-for-code-changes`); memory index line updated.

**Why**

- Owner directive: task-tree ownership improved code review and code quality tremendously over the ad-hoc / linear-`rN` cadence. Doctrine going forward, no compromise, non-negotiable. Must be permanent (live-docs + book), not just a commit message.

**Validation**

- Docs/book/workflow only — **no code change** (consistent with the doctrine's own code/not-code boundary; this commit is itself an exempt non-code change). `mdbook build book` clean. `cargo fmt`/`clippy -D warnings`/`check`/`test` unaffected and green (unchanged from `53e4c7f`).

**Impact**

- Forward-going: every code change must have a task-tree leaf owning it before the edit, leaf ID in the commit subject. `rN` is not retired — it remains the optional within-leaf cadence. Closed `rN` work is not migrated retroactively.

**Files touched**

- Updated: docs/TASK_TREE.md, COMMIT.md, SESSION_BOOTSTRAP.md, DEVELOPMENT_NOTES.md, README.md, book/src/architecture.md, CHANGES.md, MEMORY.md.

## 2026-05-17-phase5-2.4b — PHASE-5-PARAMETERIZATION.2.4b: real-gate verify → ROADMAP Phase 5 (not started)→(done) + tree closure

**Landed as:** 53e4c7f

**What changed**

- **Phase 5 — Parameterization is promoted `done`** (phase-label change `(not started)` → `(done)`). `ROADMAP.md` Phase 5 section: header `(not started)`→`(done)`, added `**Status:** done as of 2026-05-17`, kept the original intent bullets, added an explicit 4-point **"Exit criteria (met)"** block tied to the verified gate artifact, plus a scope note (parameter-aware child selection / parameter-driven parent generation are open-ended post-phase work — deliberate evidence-backed scope cut, not a blocker; no mode retired).
- `docs/tasks/PHASE-5-PARAMETERIZATION.md`: `.2.4b` `pending`→`done` (Verification + Commit filled), `.2.4`/`.2` containers and the `PHASE-5-PARAMETERIZATION` root `active`→`done`, Current Frontier → none (Phase 5 closed), Metadata Status/Last updated, Verification Log + Commit Log + Changelog `.2.4b` rows; recorded the `.2.4a` commit hash `6f87d7a` and removed a stale duplicate `.2.3` Commit-Log row.
- `docs/TASK_TREE.md`: `PHASE-5-PARAMETERIZATION` row `active`→`done` (frontier "complete — Phase 5 closed").
- Synced live docs: `README.md` (Phase 5 done; next is Phase 5b), `CODEBASE_ANALYSIS.md` (phase-coverage Phase 5 row `not started`→`done` with the closing artifact + file anchors), `MEMORY.md` (Phase line → Phase 5 done, Phase 5b next), `book/src/hierarchy.md` (Phase 5 width parameterization delivered note); `mdbook build book` clean.

**Why**

- `.2.4b`'s contract (r87 no-aspirational-claims discipline): a phase is promoted **only** after a real repo-owned downstream gate is run and verified clean — promotion strictly follows the verified artifact.

**Validation**

- Real repo-owned `Phase4Hierarchy` gate (now including `phase5_width_parameterized`) ran to completion (background job, exit 0). Banked artifact `/tmp/anvil-tool-matrix-phase5-p1/tool_matrix_report.json` verified CLEAN: `scenario_count` 213, `total_modules` 852, `coverage_gaps` `[]`, Verilator 852/0, `yosys_without_abc` 852/0, `yosys_with_abc` 852/0, `saw_width_parameterized_design` `true`, `saw_recursive_hierarchy_module_dedup_active` `true`.
- Docs/closure only — **no code change** in this leaf. `cargo fmt`/`clippy -D warnings`/`check`/`test` green (unchanged from `.2.4a`); `mdbook build book` clean.

**Impact**

- Phase 5 is closed. The next numbered roadmap phase is **Phase 5b — Synthesizable aggregates** (`docs/tasks/PHASE-5B-AGGREGATES.md`, frontier `PHASE-5B-AGGREGATES.1`). No behavioural change to generated RTL (`width_parameterization_prob` remains default-off / byte-identical).

**Files touched**

- Updated: ROADMAP.md, README.md, CODEBASE_ANALYSIS.md, MEMORY.md, book/src/hierarchy.md, docs/tasks/PHASE-5-PARAMETERIZATION.md, docs/TASK_TREE.md, CHANGES.md.

## 2026-05-16-postcompact-bootstrap-hook — Re-inject SESSION_BOOTSTRAP.md after context compaction

**Landed as:** 04b13ec

**What changed**

- New project-scoped `.claude/settings.json` (first committed `.claude/` file in this repo) with a `PostCompact` hook: `jq -Rs '{hookSpecificOutput:{hookEventName:"PostCompact",additionalContext:.}}' SESSION_BOOTSTRAP.md 2>/dev/null || true`. After every compaction (auto or manual `/compact`) the harness re-injects the full `SESSION_BOOTSTRAP.md` as `additionalContext`, so a compacted session is handed the project's recovery protocol (which itself lists the full live-doc reading order + sanity checks) instead of relying only on the compaction summary.
- `SESSION_BOOTSTRAP.md`: added a note at the top documenting that it is auto-re-injected post-compaction by that hook, so a recovering session knows to re-run the protocol.

**Why**

- Audit finding: this repo had **no** post-compaction re-read mechanism (no `PostCompact`/`SessionStart` hook, no project `settings.json`, no `CLAUDE.md`). SESSION_BOOTSTRAP.md was not re-read after compaction because nothing was. The owner asked for it to be part of the post-compaction re-read set; the correct mechanism is a `PostCompact` hook. Owner explicitly authorized a committed, team-wide project-scope hook and granted the `Write(.claude/settings.json)` permission for it.

**Validation**

- `jq -e` confirms valid JSON + the command is present. Running the stored command emits valid `PostCompact` JSON whose `additionalContext` is the full SESSION_BOOTSTRAP.md (3120 B, first line `# Session Bootstrap`, includes the "What you must do at session start" protocol). Missing-file path exits 0 (graceful — never breaks compaction). Relative path = portable for teammates. Config/doc-only: no `.rs`/`Cargo` touched (cargo gates unaffected, green from prior); not part of the mdBook (mdbook unaffected).

**Impact**

- Every future compacted session in this repo is re-handed the recovery protocol automatically. `.claude/settings.local.json` (which now also carries the `Write(.claude/settings.json)` allow rule) stays local/uncommitted as before; only `.claude/settings.json` is tracked.

**Files touched**

- New: .claude/settings.json.
- Updated: SESSION_BOOTSTRAP.md, CHANGES.md, MEMORY.md.

## 2026-05-16-phase5-2.4a — PHASE-5-PARAMETERIZATION.2.4a: phase5 matrix scenario + metrics + gap

**Landed as:** 6f87d7a

**What changed**

- `src/metrics.rs`: `DesignMetrics` gains `num_width_parameterized_modules` (modules with `param_env.is_some()`) and `num_param_override_instances` (instances with non-empty `param_bindings`), populated in `compute_design`. Both 0 for default-off / pre-Phase-5 designs.
- `src/bin/tool_matrix.rs`: new `phase5_width_parameterization_focus_config` (legacy depth-1 wrapper, library child-sourcing, `width_parameterization_prob = 1.0`, 4 leaves / 4 instances — shaped exactly like the dedup anchor so the matrix's leaf/child shape-coverage sets are unperturbed, `min_width 2`/`max_width 8`, all hierarchy-routing probs 0.0) and a `phase5_width_parameterized` scenario tuple in the Phase 4 hierarchy set. New `CoverageSummary.saw_width_parameterized_design` (set when `config.width_parameterization_prob > 0 && num_width_parameterized_modules > 0 && num_param_override_instances > 0`), merged in `merge_coverage`, and a `ScenarioSet::Phase4Hierarchy` `compute_coverage_gaps` arm. Bin-test counts updated 210→213 scenarios / 840→852 designs, and `phase5_width_parameterized` added to the all-routing-probs-0.0 exception list.
- `.2.4` split per the r87 no-aspirational-claims precedent into `.2.4a` (this — scenario/metrics/gap, **no ROADMAP promotion**) and `.2.4b` (run the real gate, verify downstream-clean, then author ROADMAP Phase 5 exit criteria + promote Phase 5 `done` + close the tree).

**Why**

- Promotion of a roadmap phase must follow a verified downstream-clean artifact, never precede it (the r87 lesson). This slice lands only the instrumentation so the next slice can run the real gate and promote on evidence.

**Validation**

- `cargo fmt --all -- --check` clean; `cargo clippy --all-targets -- -D warnings` clean; `tool_matrix` phase4 bin tests 3/3 (counts/exception verified); full `cargo test` green (COMMIT.md gate). **No ROADMAP phase label changed** (that is `.2.4b`, gated on the real artifact). No `book/` change.

**Impact**

- The repo-owned hierarchy gate now includes a parameterized-design scenario with a coverage gap that fails if no downstream-clean parameterized design is proven. Frontier → `.2.4b` (run gate → verify → promote).

**Files touched**

- Updated: src/metrics.rs, src/bin/tool_matrix.rs, docs/tasks/PHASE-5-PARAMETERIZATION.md, docs/TASK_TREE.md, CHANGES.md, MEMORY.md.

## 2026-05-16-phase5-2.3 — PHASE-5-PARAMETERIZATION.2.3: parameter-aware identity

**Landed as:** 2e99d6d

**What changed**

- `src/metrics.rs` `canonical_module_signature`: now parameter-aware. A one-time `module.param_env.is_some()` marker is hashed, and a `wsig(w)` helper replaces any width equal to `design_value` with a `u32::MAX` sentinel **for parameterized modules only** (sound: the `is_width_generic` gate guarantees a parameterized module is width-homogeneous, so every width is the parameter). Applied at every width hash site (ports, `PrimaryInput`/`Constant`/`FlopQ`/`InstanceOutput`/`Gate` node widths, `flop.width`).
- Effect: two structurally-identical parameterizable templates that differ **only** in their concrete `design_value` now hash to the **same** signature — they are the same template (instances override the width via `#(.W(v))`), so `dedup_modules` legitimately collapses them. A genuinely concrete module hashes its real widths and marker `0`, so it can **never** alias a structurally-identical parameterized one. `dedup_modules` is **unchanged** (the rule lives entirely in the signature).
- Non-parameterized modules (every default-off / pre-Phase-5 module, including the whole r87 hierarchy bank): only a constant marker-`0` prefix is added, so *relative* signature equality, determinism, and isomorphism-awareness are preserved. Verified: `canonical_module_signatures_are_stable_and_isomorphism_aware`, `planner_can_emit_structurally_duplicate_modules` (H-A-I.2), and `module_dedup_pass_collapses_structurally_duplicate_modules` (H-A-I.4) all still pass.
- New unit test `parameter_aware_identity_collapses_templates_differing_only_in_design_width` (`src/ir/dedup.rs`): param template @8 sig == param template @16 sig; concrete @8 sig ≠ param @8 sig (marker disambiguates); structurally-different param templates stay distinct; `dedup_modules` collapses the equal-signature pair under a preserved top.

**Why**

- Completes the doctrine extension from `.1`: "NodeId = identity of an expression" / "ModuleId = identity of a hierarchical module template" → "a parameterized template is one identity across its legal width range". The marker is the refinement that keeps parameterized and concrete modules from ever aliasing.

**Validation**

- `cargo fmt --all -- --check` clean; `cargo clippy --all-targets -- -D warnings` clean; `dedup::` 4/0, `metrics::` 22/0, the three H-A-I regression proofs pass; full `cargo test` green (COMMIT.md gate). `dedup_modules` unchanged; non-parameterized dedup/stability preserved; no `book/` change.

**Impact**

- Parameter-aware identity is in place. Frontier → `.2.4` (Phase 5 matrix gate + explicit ROADMAP Phase 5 exit criteria + Phase 5 → done). No phase label changed yet.

**Files touched**

- Updated: src/metrics.rs, src/ir/dedup.rs, docs/tasks/PHASE-5-PARAMETERIZATION.md, docs/TASK_TREE.md, CHANGES.md, MEMORY.md.

## 2026-05-16-phase5-2.2.3b — PHASE-5-PARAMETERIZATION.2.2.3b: hierarchy instantiation + resolved-width validate

**Landed as:** 1fd53bd

**What changed**

- `src/gen/hierarchy.rs`: new `resolved_child_port_width` helper. In `generate_parent_module`, each instantiated child that carries a `ParamEnv` gets a per-instance override value picked reproducibly via `g.rng.gen_range(env.min..=env.max)`, recorded in `Instance.param_bindings`. The resolved width (override for parameterized ports, template width otherwise) is threaded through child-input binding, the `InstanceOutput` node, the instance/parent source pools, and the synthesized top output ports. A non-parameterized child draws nothing → byte-identical.
- `src/ir/validate.rs`: a `resolved_child_width` closure makes `ChildInputWidthMismatch` / `ChildOutputWidthMismatch` compare against the instance's resolved override for parameterized child ports (template width otherwise). Default-off / pre-Phase-5 designs unaffected.
- **Soundness scoping** (recorded): only the planned-child loop picks an override. Helper / default instantiations leave `param_bindings` empty → the child elaborates at its default `W = design_value`, which is exactly its concrete monomorphic template → already valid by construction, no change needed there. This bounds the slice to one instantiation site + validate and stays sound.
- Focused proof `width_parameterization_instances_override_at_multiple_values` (legacy depth-1 wrapper, library mode, 1 parameterizable leaf × 6 instances, 4 `ConstructionStrategy` × 4 seeds): every parameterized-child instance carries a `W` binding in `[2,8]`; emitted SV carries `#(.W(v))` per instance; ≥2 *distinct* override values across the sweep (genuine multi-width reuse of one template); `validate_design` passes; default-off hierarchy emits no instance `#(` and validates.
- Closes leaf `.2.2.3b`, the `.2.2.3` container, and (with `.2.2.1`/`.2.2.2`) the `.2.2` container. **Phase 5 width parameterization is now end-to-end functional**: rules-first parameterizable leaves, multi-width `#(.W(v))` instantiation, valid by construction, downstream-shaped.

**Why**

- Completes instantiation substitution so a parameterized template is genuinely reused at multiple widths. The soundness scoping (helpers/defaults need no change because a parameterized module at its default == its concrete template) keeps the change minimal and provably correct.

**Validation**

- `cargo fmt --all -- --check` clean; `cargo clippy --all-targets -- -D warnings` clean; phase5 focused proofs 2/2; full `cargo test` green (COMMIT.md gate). Default-off byte-identical; no `book/` change.

**Impact**

- Phase 5's core capability is delivered. Frontier → `.2.3` (parameter-aware identity: a parameterized template must be one identity across its legal range), then `.2.4` (matrix gate + Phase 5 closure). No phase label changed.

**Files touched**

- Updated: src/gen/hierarchy.rs, src/ir/validate.rs, tests/pipeline.rs, docs/tasks/PHASE-5-PARAMETERIZATION.md, docs/TASK_TREE.md, CHANGES.md, MEMORY.md.

## 2026-05-16-phase5-2.2.3a — PHASE-5-PARAMETERIZATION.2.2.3a: Instance.param_bindings + emitter #(.W(v))

**Landed as:** 7950e37

**What changed**

- `src/ir/types.rs`: `Instance` gains `param_bindings: Vec<(String, u32)>` — `(parameter_name, resolved_value)` overrides for this instance. Empty for every non-parameterized instance (default-off / pre-Phase-5) → emission byte-identical.
- All **19** `Instance { … }` literal construction sites updated with `param_bindings: Vec::new()` (Instance has no `Default`). Completeness was driven by the compiler: the field was added first, then every `missing field` site fixed, and `cargo build --all-targets` clean is the completeness oracle.
- `src/emit/sv.rs`: instance emission now renders `child #(.NAME(v), …) inst (` when `param_bindings` is non-empty, and the unchanged `child inst (` when empty (byte-identical).
- Focused unit test `instance_with_param_bindings_emits_parameter_override_list`: a hand-built `Design` with one instance carrying `[("W",8)]` emits `child #(.W(8)) u_0 (`, and a sibling instance with empty bindings emits the byte-identical `child u_1 (`.
- `.2.2.3` split per the Splitting Rules into `.2.2.3a` (this — IR field + emitter) and `.2.2.3b` (hierarchy instantiation pick + resolved-width validate). Tree updated; no node renumbered (`.2.2.3` became a container).

**Why**

- Mechanically separating the IR/emit surface (no semantics) from the hierarchy/validate semantics keeps each slice signoff-reviewable. `param_bindings` defaulting to empty everywhere makes this slice provably byte-identical for all existing output; the override-emission path is exercised only by the focused unit test until `.2.2.3b` produces real bindings.

**Validation**

- `cargo fmt --all -- --check` clean; `cargo clippy --all-targets -- -D warnings` clean; `cargo build --all-targets` clean (19/19 sites); `emit::` suite 18/0 incl. the new test; full `cargo test` green (COMMIT.md gate). No behaviour change when `param_bindings` empty; no `book/` change.

**Impact**

- The IR can now carry per-instance parameter overrides and the emitter renders them. Frontier → `.2.2.3b` (hierarchy picks in-range values + resolved-width validate). No phase label changed.

**Files touched**

- Updated: src/ir/types.rs, src/emit/sv.rs, src/ir/compact.rs, src/ir/validate.rs, src/ir/dedup.rs, src/gen/hierarchy.rs, src/gen/module.rs, docs/tasks/PHASE-5-PARAMETERIZATION.md, docs/TASK_TREE.md, CHANGES.md, MEMORY.md.

## 2026-05-16-phase5-2.2.2 — PHASE-5-PARAMETERIZATION.2.2.2: rules-first parameterizable-leaf constructor

**Landed as:** b3c7f0c

**What changed**

- New `src/gen/module.rs::build_parameterizable_leaf(g, index)`: a rules-first, valid-by-construction width-homogeneous combinational leaf — design width `W >= 2` from the configured band; 2..4 inputs and 1..3 outputs all width `W`; no `clk`/`rst_n`/flops/instances; each output driven by one N-arity width-preserving gate (`Xor`/`And`/`Or`/`Add`) over all input nodes via `m.intern_gate` (so dep-sets / identity stay correct). No `Constant`/`Slice`/`Concat`/`ForFold`/`Mux`/compare → `is_width_generic` always accepts it and the single emitted body is correct for every `W`.
- Single opt-in roll added at the top of `generate_leaf_module_with_interface_profile` (free-standing single-module lane only, `interface_profile.is_none()`): on a `g.rng.gen_bool(width_parameterization_prob)` hit it builds the parameterizable leaf and annotates it. Default-off (`prob == 0.0`) never enters → byte-identical.
- `src/ir/param.rs` refactored: rolling `parameterize_module(module, rng, cfg)` → **non-rolling** `annotate_parameterized(module, cfg)`. The opt-in decision now lives solely in the constructor lane, so there is no double-roll. `src/gen/mod.rs` `generate_design` post-pass updated to the non-rolling call (still guarded by `width_parameterization_prob > 0.0`, so default-off skips it entirely). `param.rs` unit tests updated to the non-rolling API (5/0); rng imports dropped.
- Focused proof rewritten: at `prob = 1.0`, **every** forced-on single-module design across all four `ConstructionStrategy` values is a parameterized width-generic leaf, `validate_design` passes, the emitted body is fully `[W-1:0]` with no concrete `[D-1:0]` leak, and the default-off path stays byte-identical. Organic existence now holds **by construction**.

**Why**

- Resolves the `.2.2.1` rules-first pivot: a post-hoc homogeneity filter was inert (the unconstrained generator ~never emits a homogeneous module) and was the generate-then-filter anti-pattern. Constructing the parameterizable leaf by rule makes Phase 5 a real, firing capability while preserving valid-by-construction and the full-factorization doctrine. The non-rolling refactor removes the double-roll and keeps reproducibility clean.

**Validation**

- `cargo fmt --all -- --check` clean; `cargo clippy --all-targets -- -D warnings` clean; `cargo test --lib` param 5/0; focused proof passes (4 strategies × 6 seeds, all parameterized + valid + width-generic); full `cargo test` green (COMMIT.md gate). Default-off byte-identical; no `book/` change.

**Impact**

- Phase 5 width parameterization now fires by construction and is downstream-shaped. Frontier → `.2.2.3` (instantiation substitution: `Instance.param_bindings` + reproducible in-range pick + `#(.W(v))` + resolved-width validate). No phase label changed.

**Files touched**

- Updated: src/gen/module.rs, src/gen/mod.rs, src/ir/param.rs, tests/pipeline.rs, docs/tasks/PHASE-5-PARAMETERIZATION.md, docs/TASK_TREE.md, CHANGES.md, MEMORY.md.

## 2026-05-16-phase5-2.2.1 — PHASE-5-PARAMETERIZATION.2.2.1: soundness gate + width-generic emitter

**Landed as:** 8cc4fc4

**What changed**

- `src/ir/param.rs`: added `is_width_generic(module, design)` soundness gate — only a width-homogeneous combinational leaf qualifies (no flops/instances; every input/output port and every node width == design; no `Constant`; no `Slice`/`Concat`/`ForFold`; `Mux`/compare auto-excluded because their select/result nodes are width-1 ≠ design ≥ 2). `parameterize_module` now declines anything that does not pass. Unit tests updated/added: homogeneous accepted, mixed-width declined, **module-with-Constant declined**, idempotent (6/0).
- `src/emit/sv.rs`: new `param_width_decl_w(m, w)` so a parameterized module renders **every** width-homogeneous site — internal gate/instance-output wires and flop registers, not just ports — as `[W-1:0]`. The emitted body is now fully width-generic; `param_env == None` stays byte-identical.
- `tests/pipeline.rs`: focused proof renamed to `width_parameterization_is_default_off_and_emits_width_generic_bodies` — proves (a) default-off byte-identical, (b) for any parameterized module the soundness invariant holds and the SV leaks no concrete `[D-1:0]`. Organic-existence is deliberately *not* asserted here (see pivot below).
- **Rules-first pivot (recorded in `DEVELOPMENT_NOTES.md` + tree Decisions):** a 64-seed forced-on sweep produced zero organically width-homogeneous modules. A post-hoc homogeneity filter is therefore inert *and* the generate-then-filter anti-pattern the project forbids. `.2.2` was split into `.2.2.1` (these soundness primitives — done), `.2.2.2` (a rules-first parameterizable-leaf *constructor* that makes the feature actually fire — new frontier), `.2.2.3` (instantiation substitution). No node renumbered; `.2.2` became a container.

**Why**

- The soundness primitives are correct and committable on their own (default-off byte-identical, proven by unit tests + the focused proof). Splitting out the rules-first constructor keeps each slice signoff-reviewable and records *why* the original post-filter plan was wrong (durable, per the crash-recovery + rules-first doctrines).

**Validation**

- `cargo fmt --all -- --check` clean; `cargo clippy --all-targets -- -D warnings` clean; `cargo test --lib` (param.rs 6/0); focused proof passes; full `cargo test` green (COMMIT.md gate). No `.sv`-affecting change when default-off; no `book/` change so `mdbook` unaffected.

**Impact**

- Phase 5 soundness is locked in; the next slice (`.2.2.2`) makes parameterization non-inert by construction. No phase label changed.

**Files touched**

- Updated: src/ir/param.rs, src/emit/sv.rs, tests/pipeline.rs, DEVELOPMENT_NOTES.md, docs/tasks/PHASE-5-PARAMETERIZATION.md, docs/TASK_TREE.md, CHANGES.md, MEMORY.md.

## 2026-05-16-phase5-2.2-scope — PHASE-5-PARAMETERIZATION.2.2 scope refinement (soundness)

**Landed as:** 8c28eae

**What changed**

- `docs/tasks/PHASE-5-PARAMETERIZATION.md`: refined the `.2.2` leaf goal/acceptance and recorded a Decisions entry capturing a soundness finding made on entering `.2.2`. With monomorphic bodies, instantiating a parameterized module at a value ≠ `design_value` is unsound unless the emitted body is genuinely width-generic. Architecture (C) is kept sound — without (B)'s symbolic width arithmetic — by restricting parameterization to **width-homogeneous** modules (every port/node/flop width == design value, no `Constant`/`Slice`/`Concat`/`ForFold`/mixed-width site) and rendering *all* those sites (not just ports) as `[W-1:0]`. `.2.2` now also owns tightening `.2.1`'s port-anchored pass to this rule, the `Instance.param_bindings` field (+19 literal sites), the reproducible in-range value pick, and the resolved-width validation path.

**Why**

- Pure-doc tree refinement committed as the leaf's honest pre-implementation outcome (per the `docs/TASK_TREE.md` splitting/refine rule) so the soundness finding is crash-durable and the working tree stays clean before the `.2.2` implementation. No code change.

**Validation**

- Doc-only (only `docs/` + mandatory `CHANGES.md`/`MEMORY.md`); no `.rs`/`Cargo`/`book/` touched, so `cargo` + `mdbook` unaffected (green from `4cedad2`).

**Impact**

- `.2.2` is now correctly scoped to deliver *sound* multi-value instantiation. No phase label changed.

**Files touched**

- Updated: docs/tasks/PHASE-5-PARAMETERIZATION.md, CHANGES.md, MEMORY.md.

## 2026-05-16-phase5-scaffold — PHASE-5-PARAMETERIZATION.2.1: width-parameterization scaffold

**Landed as:** 4cedad2

**What changed**

- New `src/ir/param.rs`: `parameterize_module(&mut Module, &mut ChaCha8Rng, &Config) -> bool`, an opt-in **annotation-only** post-construction pass. When `Config::width_parameterization_prob` rolls true it picks a design width from an existing output port (≥2 so `[W-1:0]` is well-formed), records `ParamEnv { name:"W", min, max, design_value }`, and marks every interface port sharing that width. The module body is never touched. 5 unit tests (default-off, forced-on, mixed-width, width-1 decline, idempotence).
- `src/ir/types.rs`: new `WidthExpr { Lit(u32), Param }` (the minimal seed of the recorded architecture-(B) follow-on, not a different design) and `ParamEnv`. `Module` gains three **additive, `Default`** fields (`param_env: Option<ParamEnv>`, `parameterized_input_ports`, `parameterized_output_ports`) — zero churn to `..Module::default()` sites and no change to any load-bearing `width: u32` field.
- `src/config.rs`: opt-in `width_parameterization_prob: f64` (serde-default `0.0`, included in the probability-range validation).
- `src/gen/mod.rs`: pass wired into `generate_design` after dedup, gated by the knob — `prob == 0.0` skips it entirely (default-off byte-identical).
- `src/emit/sv.rs`: module header emits `#( parameter int W = <design_value> )` when `param_env` is set; new `param_width_decl` renders `[W-1:0]` for parameterized ports, falling back to the concrete `width_decl` everywhere else (so `param_env == None` is byte-identical to before).
- Focused proof `width_parameterization_round_trips_and_is_default_off` in `tests/pipeline.rs` (8 seeds): default-off path emits no parameter header and never sets `param_env`; forced-on path round-trips IR → `validate_design` → emit with `module X #(`, `parameter int W = D`, and `[W-1:0]`.
- Slice-boundary refinement: `Instance.param_bindings` moved from `.2.1` to `.2.2` (it is produced/consumed at instantiation; adding a non-`Default` `Instance` field in `.2.1` would churn 19 literal sites for a field unused until `.2.2`). Recorded in the task tree Decisions; no ID renumbering.

**Why**

- Implements architecture (C) from the `.1` design as the first Phase 5 code slice: a post-construction annotation pass + emitter feature that preserves valid-by-construction with **zero** changes to the invasive width-arithmetic / fold / validate code, and keeps the full-factorization doctrine intact. Default-off keeps every existing artifact byte-identical (no mode retired).

**Validation**

- `cargo fmt --all -- --check` clean; `cargo clippy --all-targets -- -D warnings` clean; `cargo test --lib` 205/0 (200 prior + 5 new); focused proof passes (8 seeds); full `cargo test` green (COMMIT.md gate). No `.sv`-affecting behaviour change when default-off; no `book/` change so `mdbook` unaffected.

**Impact**

- Phase 5 has a landed IR/emit/knob scaffold. Frontier → `.2.2` (instantiation substitution + `Instance.param_bindings` + reproducible in-range value pick + `#(.W(v))`). No phase label changed.

**Files touched**

- New: src/ir/param.rs.
- Updated: src/ir/types.rs, src/ir/mod.rs, src/config.rs, src/gen/mod.rs, src/emit/sv.rs, tests/pipeline.rs, docs/tasks/PHASE-5-PARAMETERIZATION.md, docs/TASK_TREE.md, CHANGES.md, MEMORY.md.

## 2026-05-16-phase5-design — PHASE-5-PARAMETERIZATION.1: parameterization design

**Landed as:** 786e468

**What changed**

- `DEVELOPMENT_NOTES.md`: new "Phase 5 parameterization design (2026-05-16, PHASE-5-PARAMETERIZATION.1)" entry. Codebase-grounded (file-anchored audit of the bare-`u32` width surface in `ir/types.rs`, the invasive width-arithmetic in `gen/cone.rs`/`ir/validate.rs`, the single `width_decl`/header/instance emission chokepoints in `emit/sv.rs`, the identity hashing in `metrics.rs` + `ir/dedup.rs`, the instantiation attach point in `gen/hierarchy.rs`, and the config/RNG/tool_matrix patterns). Chosen architecture **(C)**: a post-construction parameterization pass + monomorphic instantiation that preserves valid-by-construction with zero changes to the invasive width math and keeps the full-factorization doctrine intact. Three rejected alternatives recorded: (A) monomorphize-only with a lying symbolic header, (B) full symbolic `WidthExpr` threaded through construction (recorded as the strict follow-on, not a rework), (C') factorization-disable for parameterized modules. Explicit parameter-aware identity rule (hash the normalized symbolic form at parameterized sites in `canonical_module_signature`; `dedup_modules` unchanged), proof shape, and open questions.
- `docs/tasks/PHASE-5-PARAMETERIZATION.md`: `.1` → `done`; Phase 4 reaching `done` unblocked `.2`, which is split into signoff-sized leaves `.2.1` (IR+emit scaffold) → `.2.2` (instantiation substitution) → `.2.3` (parameter-aware identity) → `.2.4` (matrix gate + Phase 5 closure). Frontier → `.2.1`. Decision/Open-Questions/Blockers/Verification/Commit/Changelog updated. `docs/TASK_TREE.md` frontier row updated.

**Why**

- PNT directive: drive Phase 5 to exhaustion. The honest entry point is the design decision between monomorphize-only, full-symbolic, and a post-construction pass — recorded with rejected alternatives so `.2.x` has an unambiguous, doctrine-aligned target and a future reviewer sees why (B) was deferred rather than missed.

**Validation**

- Doc-only slice: no `.rs`/`Cargo` touched (cargo `check/test/clippy/fmt` unaffected, green from r87); `DEVELOPMENT_NOTES.md` and `docs/` are not in the mdBook so `mdbook` is unaffected. Design grounded in a file-anchored read-only audit of the live code.

**Impact**

- Phase 5 has a concrete, signoff-grade implementation plan and a split, unblocked `.2.x` frontier. No code/behaviour change; no phase label changed.

**Files touched**

- Updated: DEVELOPMENT_NOTES.md, docs/tasks/PHASE-5-PARAMETERIZATION.md, docs/TASK_TREE.md, CHANGES.md, MEMORY.md.

## 2026-05-16-phase4-closed — Phase 4 closed (PHASE-4-HIERARCHY.3, evidence-backed scope cut)

**Landed as:** 21ec427

**What changed**

- `ROADMAP.md`: Phase 4 header `(in progress)` → `(done)`; added an explicit 5-point **"Exit criteria (met)"** block (each point tied to a concrete r87 fact), and reframed the Phase 4 intro to record both Phase 4 task trees complete and the deliberate scope-cut rationale.
- Phase-4-status flipped to `done` across all live docs: `README.md` (Current-scope / next-honest-work paragraph), `CODEBASE_ANALYSIS.md` (phase-coverage row `in progress` → `done` + residual reframed), `MEMORY.md` (Phase line + `PHASE-4-HIERARCHY` tree line), `book/src/hierarchy.md` ("Phase 4 status: done" rewrite of the closing section), `book/src/architecture.md` (lead clause).
- `docs/tasks/PHASE-4-HIERARCHY.md`: `.3` → `done`, container + tree Status → `done`, frontier = complete, Verification/Commit/Changelog finalised. `docs/TASK_TREE.md` row → `done`.
- Backfilled the `PHASE-4-HIERARCHY.1` entry's `Landed as:` with its hash `0219ac7`; recorded `0219ac7` and this commit in `MEMORY.md` recent-commits.

**Why**

- The `.1` audit proved the instrumented hierarchy surface is fully landed-proven (r87, `coverage_gaps=[]`) with no missing surface, and the roadmap residual ("broader registered hierarchy patterns") is open-ended capability-deepening with no finite completion point. "Complete exhaustion" of Phase 4 is therefore necessarily a deliberate, documented, evidence-backed scope cut against explicit exit criteria — not a gap drain (already empty) nor an unbounded permutation grind. No mode/strategy retired (never-retire-strategies upheld). This satisfies the COMMIT.md rule that a phase is not promoted to `done` from narrative alone: the r87 repo-owned artifact is the visible evidence and was re-verified directly.

**Validation**

- r87 re-verified directly: `scenario_count=210`, `total_modules=840`, `coverage_gaps=[]`, Verilator/Yosys-no-abc/Yosys-abc all `840/0`, all 105 `saw_*` `true`, zero `false`.
- Doc-only slice: no `.rs`/`Cargo` touched, so `cargo check/test/clippy/fmt` unaffected (green from r87). `book/src/*` changed → `mdbook build book` run and clean.

**Impact**

- ROADMAP Phase 4 = `done`; Phase 5 (Parameterization) is the next numbered phase and is now unblocked. The `PHASE-4-HIERARCHY` task tree is complete, fulfilling the PNT-to-exhaustion directive for Phase 4. No code/behaviour change.

**Files touched**

- Updated: ROADMAP.md, README.md, CODEBASE_ANALYSIS.md, MEMORY.md, book/src/hierarchy.md, book/src/architecture.md, docs/tasks/PHASE-4-HIERARCHY.md, docs/TASK_TREE.md, CHANGES.md.

## 2026-05-16-phase4-hierarchy-surface-inventory — PHASE-4-HIERARCHY.1: hierarchy surface is landed-proven

**Landed as:** 0219ac7

**What changed**

- `docs/tasks/PHASE-4-HIERARCHY.md`: appended a definitive **Surface Inventory** (read-only audit of `src/config.rs`, `src/gen/hierarchy.rs`, `src/bin/tool_matrix.rs` `CoverageSummary`/`summarize_design_coverage`/`compute_coverage_gaps`, cross-referenced to the banked r87 report). Conclusion: of 105 `saw_*` facts, 92 hierarchy-specific facts are Phase4-gated and **all true** in r87, with `coverage_gaps = []` and 840/0 downstream-clean — **no hierarchy surface is MISSING or AMBIGUOUS**; the roadmap's "broader registered hierarchy patterns" is genuinely open-ended capability-deepening, not a finite gap set.
- Tree restructured from the audit: `.1` → `done`; `.2` ("implement next missing surface") → `superseded` (premise invalidated — no missing surface, and an open-ended permutation grind is not "completion"); frontier rotated to `.3` (deliberate evidence-backed exit-criteria scope-cut closure). Decision, Open-Questions resolution, Verification Log, Commit Log, Changelog updated.
- `docs/TASK_TREE.md`: `PHASE-4-HIERARCHY` row frontier updated `.1` → `.3` (`.1` done, `.2` superseded).

**Why**

- The directive is to PNT Phase 4 to "complete exhaustion". Honest bounding required first enumerating the finite remaining surface. The audit shows the instrumented surface is already fully landed-proven (gap list empty in r87), so exhaustion is necessarily a documented scope cut, not a gap drain or an unbounded permutation grind. Recorded so the `done` promotion in `.3` is defensible and not narrative-only.

**Validation**

- Doc-only slice (only `docs/` + mandatory `CHANGES.md`/`MEMORY.md`); no `.rs`/`Cargo`/`book/` touched, so `cargo check/test/clippy/fmt` unaffected (green from r87) and `mdbook` unaffected. Audit conclusion triangulated against the independent r87 verification done during session-recovery finalization (17 checks incl. `coverage_gaps=[]`).

**Impact**

- Phase 4's remaining work is now bounded and the path to `done` is explicit: `.3` authors ROADMAP exit criteria and promotes Phase 4 on the existing r87 evidence. No code/behaviour change; no phase label changed yet (that is `.3`).

**Files touched**

- Updated: docs/tasks/PHASE-4-HIERARCHY.md, docs/TASK_TREE.md, CHANGES.md, MEMORY.md.

## 2026-05-16-register-remaining-phase-task-trees — Register every remaining roadmap phase as a task tree

**Landed as:** 0924c8c

**What changed**

- New task-tree files under `docs/tasks/` for every not-yet-tracked remaining roadmap phase: `PHASE-4-HIERARCHY.md`, `PHASE-5-PARAMETERIZATION.md`, `PHASE-5B-AGGREGATES.md`, `PHASE-6-ADVANCED-MOTIFS.md`, `PHASE-7-ORACLE-MICRODESIGN.md`, `PHASE-8-FRONTEND-ACCEPT.md`, `PHASE-9-MULTI-ARTIFACT-UMBRELLA.md`. Each follows the `docs/tasks/TEMPLATE.md` Required Task File Sections (Metadata, Goal, Non-Goals, Acceptance Criteria, Task Tree, Current Frontier, Decisions, Open Questions, Blockers, Verification Log, Commit Log, Changelog), with Goal/Non-Goals/Acceptance derived from the corresponding `ROADMAP.md` phase text.
- `docs/TASK_TREE.md`: the Active Task Trees table gains the seven phase rows; a dated "Project-wide tracking directive" note reconciles the Adoption-Scope section with the new reality — every remaining phase is now task-tree-tracked while `rN` is explicitly preserved as the within-leaf slice cadence (mirrors the closed `HIERARCHY-AWARE-IDENTITY` leaves landing as r85/r86/r87).
- `PHASE-4-HIERARCHY` is the only tree with an executable frontier now (`.1` = a no-risk Surface-Inventory audit that enumerates the finite remaining hierarchy surface so "complete exhaustion" is bounded). The not-started phase trees open with an unblocked design-only frontier leaf and record their real blockers (e.g., Phase 5 implementation blocked by Phase 4).

**Why**

- Owner directive: the whole roadmap must be trackable through task trees. Prior state only had the closed `HIERARCHY-AWARE-IDENTITY` sub-tree plus three quality trees; Phases 4 (remaining), 5, 5b, 6, 7, 8, 9 had no tree. Registering them makes the frontier explicit and PNT-drivable without retiring the `rN` cadence (never-retire-strategies).
- Precedent: commit `65ca372` registered three quality task trees in one coherent doc commit; this slice mirrors that shape for the phase trees.

**Validation**

- Doc-only slice: no Rust touched, so `cargo check/test/clippy/fmt` are unaffected and remain green from r87. `docs/tasks/*.md` and `docs/TASK_TREE.md` are not part of the mdBook (`book/src/`), so `mdbook build` is unaffected.
- `docs/TASK_TREE.md` internal consistency re-checked: Active Task Trees table, Adoption-Scope directive note, and the seven new files agree on tree IDs, frontiers, and the rN-preservation decision.

**Impact**

- The entire remaining roadmap (Phases 4–9) is now decomposed into trackable task trees. `PHASE-4-HIERARCHY.1` is the next executable leaf; the rest carry honest blockers. No phase label changed; no code/behaviour change.

**Files touched**

- New: docs/tasks/PHASE-4-HIERARCHY.md, docs/tasks/PHASE-5-PARAMETERIZATION.md, docs/tasks/PHASE-5B-AGGREGATES.md, docs/tasks/PHASE-6-ADVANCED-MOTIFS.md, docs/tasks/PHASE-7-ORACLE-MICRODESIGN.md, docs/tasks/PHASE-8-FRONTEND-ACCEPT.md, docs/tasks/PHASE-9-MULTI-ARTIFACT-UMBRELLA.md.
- Updated: docs/TASK_TREE.md, CHANGES.md, MEMORY.md.

## 2026-05-15-phase4-module-dedup-pass — Implement and gate module-dedup pass (r87, HIERARCHY-AWARE-IDENTITY.4 + HIERARCHY-AWARE-IDENTITY.5)

**Landed as:** 750ef8b

**What changed**

- New `src/ir/dedup.rs` module implementing `dedup_modules(&mut Design) -> usize`. Fixed-point iteration: each pass groups Modules by canonical FNV-1a signature (the same signature used by `DesignMetrics.canonical_module_signatures`), picks the lexicographically-smallest-name survivor per group (with the top always preserved by explicit name match), builds a name-remap, rewrites every `Instance.module` reference in the surviving Modules, and drops merged-away entries. Iteration terminates because each non-trivial pass strictly decreases `design.modules.len()`. Three unit tests cover the typical case, the no-op case, and the top-preservation invariant.
- New `Config::hierarchy_module_dedup: bool` knob (default `false`). When `true`, `Generator::generate_design` runs `dedup_modules` after the existing per-Module finalisation passes (compact, flop-merge, gate-merge). Default-off preserves all existing behaviour.
- `pub(crate)` exposure of `canonical_module_signature` in `src/metrics.rs` so `src/ir/dedup.rs` reuses the same hash function. Single source of truth.
- Focused proof `module_dedup_pass_collapses_structurally_duplicate_modules` (depth-1 wrapper lane, 4 leaves, 4 instances, 1-in/1-out/width-1 leaves, max_depth=1, terminal_reuse_prob=1.0). For each of the four ConstructionStrategy values: runs the baseline (dedup off) and verifies `num_structurally_duplicate_module_pairs > 0`; runs the same config with dedup on and verifies (a) `num_modules < baseline`, (b) `num_structurally_duplicate_module_pairs == 0`, (c) `num_distinct == num_modules` (every survivor is unique), and (d) `validate_design` still passes (instance references resolvable post-rewrite).
- Matrix scenario `phase4_hier1_module_dedup_active` per ConstructionStrategy via `phase4_hierarchy_module_dedup_active_focus_config` (thin wrapper that toggles `hierarchy_module_dedup = true` on top of `phase4_hierarchy_structurally_duplicate_modules_focus_config`). Both scenarios stay in the bank so the before/after comparison is visible in the matrix output.
- New `saw_recursive_hierarchy_module_dedup_active` coverage fact requires the scenario's `hierarchy_module_dedup` is true, the design has ≥2 surviving Modules, 0 duplicate pairs, and `num_distinct == num_modules`. Matching gap message references `HIERARCHY-AWARE-IDENTITY.4`.
- Bank counts: 210 scenarios / 840 designs.
- Doc-sync (session-recovery finalization): added the `hierarchy_module_dedup` knob to `book/src/knobs.md` — summary-list bullet, a detailed entry with a worked before/after Rust example, and the `Config::default()` snippet — closing a knob-reference sync gap; the entry states explicitly that it is config/library-only with **no CLI flag**. Fixed the stale ROADMAP.md Phase 4 task-tree line (was "current frontier: HIERARCHY-AWARE-IDENTITY.1"; now reflects tree completion, consistent with MEMORY.md / docs/TASK_TREE.md / the task file). Removed a crash-left spurious `mut` in a `src/ir/dedup.rs` unit test that would have failed `clippy -D warnings`.
- Ran full Phase 4 hierarchy gate at `/tmp/anvil-tool-matrix-phase4-hierarchy-r87/tool_matrix_report.json`.
- Closes leaves `HIERARCHY-AWARE-IDENTITY.4` AND `HIERARCHY-AWARE-IDENTITY.5` in `docs/tasks/HIERARCHY-AWARE-IDENTITY.md` (the matrix-gate proof from `.5` is delivered by the same r87 gate run that proves `.4` is downstream-clean). Tree status moves to `done` since every leaf has landed.

**Why**

- `H-A-I.3` design sketch fixed the dedup pass's shape; this slice implements it exactly as specified. Pipeline placement (post-finalisation, in `Generator::generate_design` after the existing per-Module passes), instance-rewrite policy (fixed-point with lex-smallest-name survivor, top always preserved), and toggle/API choice (new orthogonal `Config` bool, not an extension of `IdentityMode`) all match the sketch. The proof shape also matches: baseline-vs-dedup comparison, both scenarios coexist in the bank.
- Closing `.4` and `.5` together is the natural consequence of having the matrix scenario in the same commit: the r87 gate IS the `.5` deliverable. Splitting would be artificial.

**Validation**

- 3 unit tests in `src/ir/dedup.rs`: typical case, no-op case, top-preservation.
- Focused proof `module_dedup_pass_collapses_structurally_duplicate_modules` passes (~0.05s release) for all four ConstructionStrategy values.
- Full gate: 210 scenarios, 840 designs, `coverage_gaps = []`, `Verilator 840/0`, both Yosys modes 840/0, `saw_recursive_hierarchy_module_dedup_active = true`. The earlier `saw_design_with_structurally_duplicate_modules` fact still fires because the baseline `H-A-I.2` scenario remains in the bank with dedup off.
- `cargo fmt --all -- --check`, `mdbook build book` clean.

**Impact**

- `HIERARCHY-AWARE-IDENTITY` tree complete: `.1` (canonical signatures, r85) -> `.2` (existence proof, r86) -> `.3` (design sketch) -> `.4`+`.5` (implementation + matrix gate, r87). The doctrine "NodeId = identity of an expression" now extends to "ModuleId = identity of a hierarchical module template" under the opt-in `Config::hierarchy_module_dedup` knob.
- Default-off behaviour is preserved end-to-end. The existing `H-A-I.2` scenario still produces structurally-duplicate Modules and the `saw_design_with_structurally_duplicate_modules` fact still fires on it.

**Files touched**

- New: src/ir/dedup.rs.
- Updated: src/config.rs, src/gen/mod.rs, src/ir/mod.rs, src/metrics.rs (canonical_module_signature visibility), src/bin/tool_matrix.rs, tests/pipeline.rs, CHANGES.md, DEVELOPMENT_NOTES.md, MEMORY.md, CODEBASE_ANALYSIS.md, USER_GUIDE.md, README.md, ROADMAP.md, book/src/hierarchy.md, book/src/architecture.md, book/src/knobs.md, docs/TASK_TREE.md, docs/tasks/HIERARCHY-AWARE-IDENTITY.md.

## 2026-05-15-phase4-dedup-pass-design-sketch — Module-dedup pass design sketch (HIERARCHY-AWARE-IDENTITY.3)

**Landed as:** e83efd8

**What changed**

- Added a "Design notes / Module-dedup pass design sketch (2026-05-15, HIERARCHY-AWARE-IDENTITY.3)" entry in `DEVELOPMENT_NOTES.md` covering pipeline placement, instance-rewrite policy, toggle/API choice, edge cases, proof shape, slice budget for `H-A-I.4`, and open questions. Three rejected alternatives recorded: (a) incremental dedup during construction, (b) dedup as emitter pass, (c) extending `IdentityMode` with a `HierarchicalNodeId` variant.
- Closed leaf `HIERARCHY-AWARE-IDENTITY.3` in `docs/tasks/HIERARCHY-AWARE-IDENTITY.md`; tree frontier rotates to `H-A-I.4` (implement the dedup pass per this sketch).
- Pure design slice. No code change.

**Why**

- The dedup pass touches three independent concerns — IR transformation, Config API surface, and existing-doctrine compatibility (`IdentityMode`, "never retire strategies"). Sketching the design first surfaces those concerns where the user can weigh in before code lands, and records rejected alternatives durably so future contributors know why the chosen shape was chosen.
- `H-A-I.4` now has a concrete plan to follow: ~50 lines in a new `src/ir/dedup.rs`, opt-in via a new `Config::hierarchy_module_dedup: bool`, fixed-point iteration with lexicographic-smallest-name survivor, default-off to preserve existing behaviour.

**Validation**

- `mdbook build book` clean.
- No code change; no `cargo test` impact.

**Impact**

- `HIERARCHY-AWARE-IDENTITY.3` complete. Frontier rotates to `H-A-I.4` (implement the dedup pass).
- The "never retire discussed strategies" feedback memory is honoured: the dedup pass does NOT redefine `IdentityMode::NodeId` and does NOT overload `FactorizationLevel`; it adds a new orthogonal Config knob.

**Files touched**

- Updated: DEVELOPMENT_NOTES.md, docs/TASK_TREE.md, docs/tasks/HIERARCHY-AWARE-IDENTITY.md, CHANGES.md, MEMORY.md.

## 2026-05-15-phase4-structural-duplicate-modules — Prove the planner can emit structurally-duplicate Modules (r86, HIERARCHY-AWARE-IDENTITY.2)

**Landed as:** 555058d

**What changed**

- Added focused proof `planner_can_emit_structurally_duplicate_modules` (depth-1 wrapper lane, 4 leaves, 4 instances, 1-in/1-out/width-1 leaves, max_depth=1, terminal_reuse_prob=1.0). Under these tight constraints the leaf generator's RNG-driven choices collapse to a single canonical structure, so every library leaf shares one canonical signature; `num_structurally_duplicate_module_pairs > 0` for all four ConstructionStrategy values.
- Added matrix scenario `phase4_hier1_structurally_duplicate_modules` per ConstructionStrategy via `phase4_hierarchy_structurally_duplicate_modules_focus_config`. This is the one Phase 4 hierarchy scenario whose hierarchy-routing probabilities are all 0.0 by design (the test exception list in the matrix unit test was extended to recognise its suffix).
- Added `saw_design_with_structurally_duplicate_modules` coverage fact and matching gap message. Requires `num_structurally_duplicate_module_pairs > 0` and `num_distinct_module_signatures < num_modules` so the gate explicitly proves the planner produces the dedup-pass exercise material at least once per bank.
- Bank counts: 207 scenarios / 828 designs.
- Ran full Phase 4 hierarchy gate at `/tmp/anvil-tool-matrix-phase4-hierarchy-r86/tool_matrix_report.json`.
- Closes leaf `HIERARCHY-AWARE-IDENTITY.2` in [docs/tasks/HIERARCHY-AWARE-IDENTITY.md](docs/tasks/HIERARCHY-AWARE-IDENTITY.md); tree frontier rotates to `H-A-I.3` (dedup-pass design sketch).

**Why**

- `HIERARCHY-AWARE-IDENTITY.1` (r85) gave every Module a canonical signature. Without a proof that the planner currently emits at least one structurally-duplicate Module pair, the future dedup pass (`H-A-I.4`) would have no live exercise — and we would not know whether dedup is even applicable to ANVIL's planner. This slice establishes the existence proof: the planner CAN produce duplicates under the right knob settings. That removes the only structural blocker on the path from canonical signatures to dedup.

**Validation**

- Focused proof passes (~0.08s release) for all four ConstructionStrategy values.
- Full gate: 207 scenarios, 828 designs, `coverage_gaps = []`, `Verilator 828/0`, both Yosys modes 828/0, `saw_design_with_structurally_duplicate_modules = true`.
- `cargo fmt --all -- --check`, `mdbook build book` clean.

**Impact**

- `HIERARCHY-AWARE-IDENTITY.2` complete; frontier rotates to `.3` (design sketch for the dedup pass, recorded in DEVELOPMENT_NOTES.md as required by the tree).
- The new scenario `phase4_hier1_structurally_duplicate_modules` is the first Phase 4 hierarchy scenario in the bank whose hierarchy-routing probabilities are all 0.0 by design. It exists specifically to exercise the per-Module canonical-signature path, not any of the existing route surfaces.

**Files touched**

- src/bin/tool_matrix.rs, tests/pipeline.rs, CHANGES.md, DEVELOPMENT_NOTES.md, MEMORY.md, CODEBASE_ANALYSIS.md, USER_GUIDE.md, README.md, ROADMAP.md, book/src/hierarchy.md, book/src/architecture.md, docs/TASK_TREE.md, docs/tasks/HIERARCHY-AWARE-IDENTITY.md.

## 2026-05-14-differential-simulation-scope-narrowed — Narrow DIFFERENTIAL-SIMULATION.1 scope to "scope the second simulator"

**Landed as:** 21174d8

**What changed**

- Updated `docs/tasks/DIFFERENTIAL-SIMULATION.md`: `.1`'s goal now reflects that Verilator-side compatibility is already proven by the matrix gate (Verilator is wired into every focused proof and every scenario in the 204-scenario bank); the leaf's real question is **which second simulator** to pair with it for the differential check, and where iverilog (the default candidate) diverges from Verilator on ANVIL's output. Yosys is explicitly clarified as a synthesizer rather than a differential-simulation peer.
- Recorded the decision in the tree's `Decisions` section.

**Why**

- User-supplied correction: Verilator and Yosys are already part of the ANVIL flow. The original `.1` scope ("which open-source simulators ingest ANVIL output") asked too broad a question — we already know Verilator does, and Yosys is the wrong axis. The leaf now matches the actual question the next session will work on.

**Validation**

- Pure doc edit on `docs/tasks/DIFFERENTIAL-SIMULATION.md`; no code change.
- `cargo fmt --all -- --check`, `mdbook build book` clean.

**Impact**

- `DIFFERENTIAL-SIMULATION.1` is now the most-focused single-investigation leaf among the four open frontiers, since the question shrank from "survey simulators" to "scope iverilog (or alternative) against Verilator's already-clean output". Frontier ordering in `docs/TASK_TREE.md` unchanged.

**Files touched**

- Updated: docs/tasks/DIFFERENTIAL-SIMULATION.md, CHANGES.md, MEMORY.md.

## 2026-05-14-coverage-instrumentation-baseline — Add cargo-llvm-cov baseline (COVERAGE-INSTRUMENTATION.1)

**Landed as:** d19d427

**What changed**

- Confirmed cargo-llvm-cov 0.8.7 and `llvm-tools-aarch64-apple-darwin` are already installed locally — no install step required.
- Ran `cargo llvm-cov --release` over the existing unit + `tests/pipeline.rs` test suite (110 tests, ~295s wall-clock). The 75-min Phase 4 hierarchy matrix gate was intentionally excluded so the baseline stays reproducible in minutes.
- Wrote `docs/coverage-baseline.md` with per-file line/function/region coverage numbers across all 14 crate files, a top-5 under-covered-files triage target list, and explicit caveats about matrix-gate-excluded paths and defensive-panic unreachability.
- Closes leaf `COVERAGE-INSTRUMENTATION.1` in [docs/tasks/COVERAGE-INSTRUMENTATION.md](docs/tasks/COVERAGE-INSTRUMENTATION.md). Tree frontier rotates to `.2` (triage of the top-5 under-covered files).

**Why**

- ANVIL's "the matrix is comprehensive by design intent" claim was previously unverified. The baseline gives a concrete measurement: 85.26% lines / 91.95% functions / 87.61% regions covered without the matrix gate. The matrix gate is where the planner core gets its strongest exercise (Phase 4 hierarchy paths in `gen/hierarchy.rs` and `bin/tool_matrix.rs`) — but the baseline still shows `gen/hierarchy.rs` at 95.61% lines and `metrics.rs` at 99.66% from the unit + pipeline suite alone, which is a strong signal that the focused proofs already exercise the planner core comprehensively.
- The top-5 under-covered files are exactly the ones expected: matrix harness, CLI overlay, defensive validators, clap boilerplate. `.2` triage will decide which uncovered regions are dead code (remove), rarely-fired paths (add focused proofs), or intentionally-unreachable defensive panics (leave and document). The hot-spot is `gen/cone.rs` at 88.65% lines / 454 missed — the only planner-core file outside the 95%+ band — which suggests anti-collapse rollback paths or block-assembly variants worth examining.

**Validation**

- `cargo llvm-cov --release` ran to completion in ~295s, all 110 tests passing.
- `cargo llvm-cov report --release` produced the per-file breakdown that anchors `docs/coverage-baseline.md`.
- `cargo fmt --all -- --check`, `mdbook build book` clean.

**Impact**

- ANVIL now has a reproducible, dependency-free coverage baseline; `.2` triage is the next eligible leaf in `COVERAGE-INSTRUMENTATION`.
- First task-tree-managed slice on `COVERAGE-INSTRUMENTATION`. The leaf-ID convention from `docs/TASK_TREE.md` is followed: commit subject identifies the leaf; task file is updated in the same commit.

**Files touched**

- New: docs/coverage-baseline.md.
- Updated: docs/TASK_TREE.md, docs/tasks/COVERAGE-INSTRUMENTATION.md, CHANGES.md, MEMORY.md, DEVELOPMENT_NOTES.md.

## 2026-05-13-phase4-canonical-module-signature-instrumentation — Add canonical module signatures (first slice of hierarchy-aware identity, HIERARCHY-AWARE-IDENTITY.1)

**Landed as:** f3ee1f3

**What changed**

- Added `canonical_module_signatures: Vec<u64>` to `DesignMetrics`, one deterministic 64-bit FNV-1a hash per module in `design.modules`. The hash covers port direction+width sequence, clock/reset presence, every node's kind+width+operand-id sequence, every drive, every flop's width+D, and every instance's role+input wiring — but intentionally excludes `instance.module` and `instance.name`, so structurally-identical Modules that instantiate distinctly-named-but-identically-shaped children share a signature.
- Added `num_distinct_module_signatures` and `num_structurally_duplicate_module_pairs` as derived counters.
- Added focused proof `canonical_module_signatures_are_stable_and_isomorphism_aware` asserting (a) same config + same seed produces the same signature vec across two independent generator runs, (b) one signature per module, (c) every signature is nonzero, (d) `num_distinct >= 1`, and (e) `num_structurally_duplicate_module_pairs` agrees with the re-computed combinatorial value.
- Added matrix scenario `phase4_recur_d2_canonical_module_signatures` per ConstructionStrategy as a gate-time anchor for the new instrumentation.
- Added `saw_recursive_hierarchy_canonical_module_signature_diversity` coverage fact (requires `realized_max_leaf_depth > 1`, `canonical_module_signatures.len() == num_modules`, all signatures nonzero, and `num_distinct_module_signatures >= 2`) and matching gap message.
- Bank counts: 204 scenarios / 816 designs.
- Ran full Phase 4 hierarchy gate at `/tmp/anvil-tool-matrix-phase4-hierarchy-r85/tool_matrix_report.json`.
- Closes leaf `HIERARCHY-AWARE-IDENTITY.1` in [docs/tasks/HIERARCHY-AWARE-IDENTITY.md](docs/tasks/HIERARCHY-AWARE-IDENTITY.md); updates the tree's verification log and commit log in the same commit.

**Why**

- PNT-3 of the autonomous-PNT chain. First slice of hierarchy-aware `NodeId` identity. The metric is purely observational — it doesn't change the planner — but it gives every Module a deterministic structural identity that future slices can use to dedupe `Design::modules` when `IdentityMode::NodeId` is active, extending the doctrine "NodeId = identity of an expression" up to "ModuleId = identity of a hierarchical module template". The signature deliberately excludes child-module *names* so structural isomorphism is detected even when the planner emitted distinctly-named-but-identical sub-hierarchies (which is the entire point of the future dedup work).

**Validation**

- Focused proof passes (~0.35s release) for all four construction strategies.
- Full gate: 204 scenarios, 816 designs, `coverage_gaps = []`, `Verilator 816/0`, both Yosys modes 816/0, `saw_recursive_hierarchy_canonical_module_signature_diversity = true`.
- `cargo fmt --all -- --check`, `mdbook build book` clean.

**Impact**

- Every design in the live bank now carries a stable, dependency-free canonical signature per module. Follow-up slices in `HIERARCHY-AWARE-IDENTITY` (leaves `.2`–`.5`) can: (i) flag when the planner emits structural duplicates, (ii) sketch the dedup pass, (iii) implement opt-in dedup, and (iv) prove the dedup path downstream-clean in the matrix gate.
- Autonomous PNT-3 of 3. First task-tree-managed code slice on ANVIL.

**Files touched**

- src/metrics.rs, src/bin/tool_matrix.rs, tests/pipeline.rs, CHANGES.md, DEVELOPMENT_NOTES.md, MEMORY.md, CODEBASE_ANALYSIS.md, USER_GUIDE.md, README.md, ROADMAP.md, book/src/hierarchy.md, book/src/architecture.md, docs/tasks/HIERARCHY-AWARE-IDENTITY.md.

## 2026-05-14-quality-improvement-task-trees — Register three quality-improvement task trees

**Landed as:** 65ca372

**What changed**

- Added three new active task trees under `docs/tasks/`:
  - `INSTA-SNAPSHOTS` (`docs/tasks/INSTA-SNAPSHOTS.md`): enforce byte-identical reproducibility by adding `insta`-backed snapshot tests of generator output for canonical `(seed, config)` tuples covering ANVIL's reachable generation modes (leaf, depth-1 wrapper, recursive lane, library/on-demand sourcing, helper-instance routes). 3 leaves: baseline wire-up → axis coverage → COMMIT.md integration + protocol docs.
  - `DIFFERENTIAL-SIMULATION` (`docs/tasks/DIFFERENTIAL-SIMULATION.md`): raise the downstream contract from "parses and synthesises" to "all open-source simulators agree on semantics" by driving the same SV through Verilator and iverilog and asserting byte-equal output traces. 4 leaves: simulator-compatibility investigation → single-design harness → tool_matrix `--diff-sim` integration → contract documentation.
  - `COVERAGE-INSTRUMENTATION` (`docs/tasks/COVERAGE-INSTRUMENTATION.md`): convert "the matrix is comprehensive by design intent" into "by measurement" using `cargo-llvm-cov`. 3 leaves: baseline report → top-5 under-covered triage → act on triage (remove dead code or add focused proofs).
- Registered all three in `docs/TASK_TREE.md`'s `Active Task Trees` table alongside `HIERARCHY-AWARE-IDENTITY`.
- No code change. Pure documentation slice. The trees are forward-looking decomposition artifacts; the first eligible leaf in each is `pending` until selected for implementation.

**Why**

- ANVIL's existing test discipline already does the rarest hard thing right: validity by construction. The remaining quality dial is *consistency across observers* — different simulators, different runs, different platforms, different code paths. The three new trees decompose that dial along its three orthogonal axes (byte-identical across runs, semantic agreement across simulators, planner-branch comprehensiveness across the matrix). Each is multi-slice and fits the task-tree adoption rule cleanly.
- Registering all three as `active` immediately makes the planned scope visible in `docs/TASK_TREE.md` and discoverable in `SESSION_BOOTSTRAP.md`'s reading order. Future sessions see the decomposition without re-deriving it from chat history.

**Validation**

- Pure documentation slice; no code change.
- `mdbook build book` clean (the new task-tree files live in `docs/tasks/`, not `book/src/`).
- No new cargo dependencies.

**Impact**

- Three new active task trees register quality-improvement scope. None of the leaves are `in_progress`; selection of the first leaf happens in a future session unless the user picks an explicit ordering.
- Does not change the frontier of `HIERARCHY-AWARE-IDENTITY` — that tree's `H-A-I.1` slice is still in-flight via the r85 gate at the time this commit lands.

**Files touched**

- New: docs/tasks/INSTA-SNAPSHOTS.md, docs/tasks/DIFFERENTIAL-SIMULATION.md, docs/tasks/COVERAGE-INSTRUMENTATION.md.
- Updated: docs/TASK_TREE.md, CHANGES.md, MEMORY.md, DEVELOPMENT_NOTES.md.

## 2026-05-14-adopt-task-tree-workflow — Adopt FSMGen task-tree tracking workflow

**Landed as:** f2b95f7

**What changed**

- Added `docs/TASK_TREE_README.md` (portable setup guide, lifted verbatim from FSMGen) and `docs/TASK_TREE.md` (ANVIL-adapted operating spec, active-tree index, PNT selection rules).
- Added `docs/tasks/TEMPLATE.md` (copyable skeleton, lifted verbatim from FSMGen).
- Added `docs/tasks/HIERARCHY-AWARE-IDENTITY.md` as the first active tree, covering the multi-slice hierarchy-aware identity work that r85 opens (canonical signatures → structural-duplicate metric → dedup-pass design sketch → opt-in dedup implementation → matrix gate proof).
- Added a "Task-tree-managed commits" section to `COMMIT.md` requiring leaf-ID traceability and same-commit task-file updates for task-tree-managed work. Non-task-tree commits follow the standard checklist unchanged.
- Linked the task-tree docs from `README.md`'s fast-ramp-up reading order and `SESSION_BOOTSTRAP.md`'s session-start reading list.
- Linked the active `HIERARCHY-AWARE-IDENTITY` tree from `ROADMAP.md`'s Phase 4 section.
- Recorded the adoption decision and scope in `DEVELOPMENT_NOTES.md` under a new "Workflow notes" section, including the rejected alternative (full FSMGen-style mandate).

**Why**

- Two recent pause/resume cycles (mid-r81 and mid-r85) were patched with bespoke `project_*_pause.md` memory entries. That works once or twice but does not scale to the multi-slice hierarchy-aware identity refactor that r85 opens. Task trees give multi-slice work durable leaf IDs, an explicit current frontier, and dedicated places to record blockers, decisions, and verification. FSMGen ships a portable workflow already proven there; lifting it (verbatim where possible, adapted only where ANVIL's live-doc names differ) is cheaper and more correct than re-inventing it locally.
- Adoption is deliberately scoped: linear `rN` coverage extensions stay on the existing convention; only multi-slice tasks open a tree. This avoids the "two parallel ledgers" failure mode where `CHANGES.md` and `docs/tasks/*.md` drift apart.

**Validation**

- No code change; pure documentation/workflow slice.
- `mdbook build book` clean (new files live in `docs/`, not `book/src/`, so the book SUMMARY does not need to change).
- No new cargo dependencies, no new generator behaviour, no new IR shape.

**Impact**

- Future multi-slice work (starting with the dedup follow-ups to r85's canonical signatures) gets a durable execution ledger with stable leaf IDs in commit subjects.
- The `r85` slice (the very next commit) is the first task-tree-managed commit: its subject identifies `HIERARCHY-AWARE-IDENTITY.1`.

**Files touched**

- New: docs/TASK_TREE_README.md, docs/TASK_TREE.md, docs/tasks/TEMPLATE.md, docs/tasks/HIERARCHY-AWARE-IDENTITY.md.
- Updated: COMMIT.md, README.md, SESSION_BOOTSTRAP.md, ROADMAP.md, MEMORY.md, DEVELOPMENT_NOTES.md, CHANGES.md.

## 2026-05-12-phase4-recursive-parent-cone-helper-budget-5 — Prove a recursive non-top internal parent can saturate a parent-cone helper budget of 5 helpers

**Landed as:** ed4988b

**What changed**

- Added focused proof `recursive_hierarchy_parent_cone_helper_budget_5_below_top` (depth 2, 4,4 children, hierarchy_child_input_cone_prob=1.0, hierarchy_parent_cone_instance_prob=1.0, max_parent_cone_instances_per_module=5).
- Added matrix scenario `phase4_recur_d2_parent_cone_instance_budget5` per ConstructionStrategy via `phase4_recursive_parent_cone_instance_budget_5_focus_config`.
- Added `saw_recursive_parent_cone_helper_budget_5` coverage fact and matching gap message. Required condition: `max_parent_cone_instances_per_module >= 5`, planner-realized `max_parent_cone_instances_per_internal_module >= 5`, and non-top helper count strictly exceeding top-only baseline.
- Bank counts: 201 scenarios / 804 designs.
- Ran full Phase 4 hierarchy gate at `/tmp/anvil-tool-matrix-phase4-hierarchy-r84/tool_matrix_report.json`.

**Why**

- PNT-2 of the autonomous-PNT chain. Extends the helper-budget axis from `3` (the previous saturating proof) to `5`, demonstrating the planner can spend non-trivial helper budgets per internal parent. Uses 4,4 child instances so each parent has ~8 child-input decision sites — enough demand to saturate budget 5.

**Validation**

- Focused proof passes (~0.1s release) for all four construction strategies.
- Full gate: 201 scenarios, 804 designs, `coverage_gaps = []`, `Verilator 804/0`, both Yosys modes 804/0, `saw_recursive_parent_cone_helper_budget_5 = true`.
- `cargo fmt --all -- --check`, `mdbook build book` clean.

**Impact**

- Helper-budget axis is now gated at two thresholds (>=2 via the existing `saw_recursive_multiple_parent_cone_instances_per_parent` family, and >=5 via the new fact). Future slices can push budget further if demand-shape work warrants it.
- Autonomous PNT-2 of 3. No push until PNT-3 lands.

**Files touched**

- src/bin/tool_matrix.rs, tests/pipeline.rs, CHANGES.md, DEVELOPMENT_NOTES.md, MEMORY.md, CODEBASE_ANALYSIS.md, USER_GUIDE.md, README.md, ROADMAP.md, book/src/hierarchy.md, book/src/architecture.md.

## 2026-05-12-phase4-recursive-three-stage-registered-parent-composed-chain — Prove recursive non-top registered parent-composed child-input bindings can chain through three parent-local flop stages without helpers

**Landed as:** da6a900

**What changed**

- Added focused proof `recursive_hierarchy_registered_parent_composed_routes_can_chain_three_stages_below_top` (depth 3, 4,4 children, registered_child_input_cone_prob=1.0, max_flops=128, max_depth=8).
- Added new metric pair `child_input_bindings_from_registered_three_stage_parent_composed_logic` (top + non-top) plus `registered_three_stage_parent_composed_child_input_binding_fraction` (top + non-top), detected by walking the FlopQ -> D -> FlopQ chain three deep with each stage's D being non-slice/non-concat parent-composed logic that depends on both instance outputs and flop virtuals.
- Added matrix scenario `phase4_recur_d3_registered_three_stage_parent_composed_chain` per ConstructionStrategy via `phase4_recursive_registered_three_stage_parent_composed_chain_focus_config`.
- Added `saw_recursive_hierarchy_three_stage_registered_parent_composed_chain` coverage fact and matching gap message.
- Bank counts: 198 scenarios / 792 designs.
- Ran full Phase 4 hierarchy gate at `/tmp/anvil-tool-matrix-phase4-hierarchy-r83/tool_matrix_report.json`.

**Why**

- First slice of the broader-Phase-4 work after the depth-7 sweep closed in r82. Extends the registered-parent-composed multi-stage subcase from chain length 2 (proven since the earliest multistage banks) to chain length >= 3. The new metric distinguishes chain depth 3 from any chain depth >= 2, so the matrix gate can keep enforcing both as separate first-class facts. PNT-1 in the autonomous-PNT chain.

**Validation**

- Focused proof passes (~1.2s release) for all four construction strategies.
- Full gate: 198 scenarios, 792 designs, `coverage_gaps = []`, `Verilator 792/0`, both Yosys modes 792/0, `saw_recursive_hierarchy_three_stage_registered_parent_composed_chain = true`.
- `cargo fmt --all -- --check`, `mdbook build book` clean.

**Impact**

- Registered parent-composed multi-stage chain is now gated at exactly two distinct chain-depth thresholds (>=2 via the existing `saw_recursive_hierarchy_registered_multistage_routing` family, and >=3 via the new fact). Future slices can extend along chain depth (length 4, 5, ...) the same way the recent slices extended along hierarchy depth.
- Autonomous PNT-1 of 3. No push until PNT-3 lands.

**Files touched**

- src/metrics.rs, src/bin/tool_matrix.rs, tests/pipeline.rs, CHANGES.md, DEVELOPMENT_NOTES.md, MEMORY.md, CODEBASE_ANALYSIS.md, USER_GUIDE.md, README.md, ROADMAP.md, book/src/hierarchy.md, book/src/architecture.md.

## 2026-05-11-phase4-recursive-depth-7-stateful-mixed-support-child-inputs — Close depth-7 sweep with recursive non-top stateful unregistered parent-composed mixed-support child inputs at exact hierarchy depth 7 without helpers (2,2 calibrated)

**Landed as:** 69b2173

**What changed**

- Added focused proof `recursive_hierarchy_stateful_parent_composed_routes_mix_parent_ports_at_depth_7_without_helpers` (depth 7, 2,2 children, child_input_cone_prob=1.0, parent_flop_prob=1.0, max_flops=64).
- Added matrix scenario `phase4_recur_d7_stateful_parent_composed_mixed_support_child_input` per ConstructionStrategy via `phase4_recursive_d7_stateful_parent_composed_mixed_support_focus_config` (2,2 calibrated, mirrors r77).
- Added `saw_recursive_hierarchy_depth_7_stateful_parent_composed_mixed_support_child_inputs` coverage fact and matching gap message.
- Bank counts: 195 scenarios / 780 designs.
- Ran full Phase 4 hierarchy gate at `/tmp/anvil-tool-matrix-phase4-hierarchy-r82/tool_matrix_report.json`.

**Why**

- Fifth and final slice of the depth-7 sweep, mirroring r77 (depth 6), r72 (depth 5), r67 (depth 4), r62 (depth 3) — closes the depth-7 axis. Calibration: depth-7 stateful mixed-support cells use 2,2 child-instance bounds (same as r77/r79); at 4,4/depth-7 the gate would explode well beyond a safe-slice budget.

**Validation**

- Focused proof passes (~14s release).
- Full gate: 195 scenarios, 780 designs, `coverage_gaps = []`, `Verilator 780/0`, both Yosys modes 780/0, `saw_recursive_hierarchy_depth_7_stateful_parent_composed_mixed_support_child_inputs = true`.
- `cargo fmt --all -- --check`, `mdbook build book` clean.

**Impact**

- Depth-7 axis fully closed: all five cells (parent-flops r78, mixed-support child inputs r79, parent-port-composed outputs r80, stateful parent-port-composed outputs r81, stateful mixed-support child inputs r82) are first-class downstream-clean coverage facts.
- Batch step 10/10. Push follows the hash record.

**Files touched**

- src/bin/tool_matrix.rs, tests/pipeline.rs, CHANGES.md, DEVELOPMENT_NOTES.md, MEMORY.md, CODEBASE_ANALYSIS.md, USER_GUIDE.md, README.md, ROADMAP.md, book/src/hierarchy.md, book/src/architecture.md.

## 2026-05-08-phase4-recursive-depth-7-stateful-parent-port-composed-outputs — Push recursive non-top stateful parent-port-composed parent outputs to exact hierarchy depth 7 without helpers

**Landed as:** 89dadfe

**What changed**

- Added focused proof `recursive_hierarchy_stateful_parent_outputs_mix_parent_ports_at_depth_7_without_helpers` (depth 7, 2,2 children, parent_flop_prob=1.0).
- Added matrix scenario `phase4_recur_d7_stateful_parent_port_composed_output` per ConstructionStrategy via `phase4_recursive_d7_stateful_parent_port_composed_output_focus_config`.
- Added `saw_recursive_hierarchy_depth_7_stateful_parent_port_composed_outputs` coverage fact and matching gap message.
- Bank counts: 192 scenarios / 768 designs.
- Ran full Phase 4 hierarchy gate at `/tmp/anvil-tool-matrix-phase4-hierarchy-r81/tool_matrix_report.json`.

**Why**

- Fourth slice of the depth-7 sweep, mirroring r76 (depth 6), r71 (depth 5), r66 (depth 4), r61 (depth 3). Stateful parent-port-composed cells already use 2,2 children at all depths, so no calibration drift.

**Validation**

- Focused proof passes (~5s release).
- Full gate: 192 scenarios, 768 designs, `coverage_gaps = []`, `Verilator 768/0`, both Yosys modes 768/0, `saw_recursive_hierarchy_depth_7_stateful_parent_port_composed_outputs = true`.
- `cargo fmt`, `cargo clippy`, `mdbook build` all clean.

**Impact**

- Depth-7 axis now has four first-class coverage facts. Only one cell remains: stateful mixed-support child inputs (r82).
- Batch step 9/10. No push.

**Files touched**

- src/bin/tool_matrix.rs, tests/pipeline.rs, CHANGES.md, DEVELOPMENT_NOTES.md, MEMORY.md, CODEBASE_ANALYSIS.md, USER_GUIDE.md, README.md, ROADMAP.md, book/src/hierarchy.md, book/src/architecture.md.

## 2026-05-08-phase4-recursive-depth-7-parent-port-composed-outputs — Push recursive non-top parent-port-composed parent outputs to exact hierarchy depth 7 without helpers or parent-local state

**Landed as:** e8eb1a8

**What changed**

- Added focused proof `recursive_hierarchy_parent_outputs_mix_parent_ports_at_depth_7_without_helpers` (depth 7, 2,2 children, no flops, no helpers, no child-input cones).
- Added matrix scenario `phase4_recur_d7_parent_port_composed_output` per ConstructionStrategy via `phase4_recursive_d7_parent_port_composed_output_focus_config`.
- Added `saw_recursive_hierarchy_depth_7_parent_port_composed_outputs` coverage fact and matching gap message.
- Bank counts: 189 scenarios / 756 designs.
- Ran full Phase 4 hierarchy gate at `/tmp/anvil-tool-matrix-phase4-hierarchy-r80/tool_matrix_report.json`.

**Why**

- Third slice of the depth-7 sweep, mirroring r75 (depth 6), r70 (depth 5), r65 (depth 4), r60 (depth 3). Parent-port-composed cells already use 2,2 children at all depths, so no calibration drift.

**Validation**

- Focused proof passes (~1.3s release).
- Full gate: 189 scenarios, 756 total designs, `coverage_gaps = []`, `Verilator 756/0`, `Yosys without-abc 756/0`, `Yosys with-abc 756/0`, `saw_recursive_hierarchy_depth_7_parent_port_composed_outputs = true`.
- `cargo fmt`, `cargo clippy`, `mdbook build` all clean.

**Impact**

- Depth-7 axis now has three first-class coverage facts (parent-flops r78, mixed-support child inputs r79, parent-port-composed outputs r80). Future r81/r82 close the depth-7 sweep.
- Batch step 8/10. No push.

**Files touched**

- src/bin/tool_matrix.rs, tests/pipeline.rs, CHANGES.md, DEVELOPMENT_NOTES.md, MEMORY.md, CODEBASE_ANALYSIS.md, USER_GUIDE.md, README.md, ROADMAP.md, book/src/hierarchy.md, book/src/architecture.md.

## 2026-05-08-phase4-recursive-depth-7-mixed-support-child-inputs — Push recursive non-top mixed-support child inputs to exact hierarchy depth 7 without helpers (2,2 calibrated)

**Landed as:** 7bd3235

**What changed**

- Added focused proof `recursive_hierarchy_parent_composed_routes_mix_parent_ports_at_depth_7_without_helpers` (depth 7, 2,2 children, child_input_cone_prob=1.0, no helpers, no flops).
- Added matrix scenario `phase4_recur_d7_parent_composed_mixed_support_child_input` per ConstructionStrategy via `phase4_recursive_d7_parent_composed_mixed_support_focus_config` (2,2 calibrated, mirrors r74/r77).
- Added `saw_recursive_hierarchy_depth_7_mixed_support_child_inputs` coverage fact and matching gap message.
- Bank counts: 186 scenarios / 744 designs.
- Ran full Phase 4 hierarchy gate at `/tmp/anvil-tool-matrix-phase4-hierarchy-r79/tool_matrix_report.json`.

**Why**

- Second slice of the depth-7 sweep, mirroring r74 (depth 6), r69 (depth 5), r64 (depth 4), r59 (depth 3). Calibration: depth-7 mixed-support cells use 2,2 child-instance bounds (same as depth 6); the 4,4 tree at depth 7 would balloon to ~5461 internal occurrences, exceeding safe-slice budget. Smoke at depth 7 with 2,2 confirmed 127 internal-parent occurrences with child_input_bindings_from_parent_composed_logic 219 vs top 1, child_input_bindings_from_mixed_support 173 vs top 1.

**Validation**

- `cargo test --test pipeline recursive_hierarchy_parent_composed_routes_mix_parent_ports_at_depth_7_without_helpers`
- `cargo test --bin tool_matrix`
- `cargo run --bin tool_matrix -- --phase4-hierarchy-gate --out /tmp/anvil-tool-matrix-phase4-hierarchy-r79 --yosys-mode both`
  - 186 scenarios, 4 designs/scenario, 744 total designs
  - `coverage_gaps = []`, `Verilator 744/0`, `Yosys without-abc 744/0`, `Yosys with-abc 744/0`
  - `saw_recursive_hierarchy_depth_7_mixed_support_child_inputs = true`
- `cargo fmt`, `cargo clippy`, `mdbook build` all clean.

**Impact**

- Depth-7 axis now has two first-class coverage facts (parent-flops r78, mixed-support child inputs r79). Future r80..r82 close the depth-7 sweep.
- Batch step 7/10. No push.

**Files touched**

- src/bin/tool_matrix.rs, tests/pipeline.rs, CHANGES.md, DEVELOPMENT_NOTES.md, MEMORY.md, CODEBASE_ANALYSIS.md, USER_GUIDE.md, README.md, ROADMAP.md, book/src/hierarchy.md, book/src/architecture.md.

## 2026-05-07-phase4-recursive-depth-7-parent-local-flops — Open depth-7 axis with recursive non-top parent-local flops at exact hierarchy depth 7

**Landed as:** 6f2bab0

**What changed**

- Added focused proof `recursive_hierarchy_parents_can_emit_local_flops_at_depth_7` (depth 7, 2,2 children, parent_flop_prob=1.0, max_flops=64).
- Added matrix scenario `phase4_recur_d7_parent_state` per ConstructionStrategy via `phase4_recursive_d7_parent_state_focus_config`.
- Added `saw_recursive_hierarchy_depth_7_parent_local_flops` coverage fact and matching gap message.
- Added `(7, 7)` to recognized `range_depths` set.
- Bank counts: 183 scenarios / 732 designs.
- Ran full Phase 4 hierarchy gate at `/tmp/anvil-tool-matrix-phase4-hierarchy-r78/tool_matrix_report.json`.

**Why**

- First slice of the depth-7 sweep, opening the axis above the closed depth-6 sweep (r73..r77), mirroring how r73 opened depth-6 above r68..r72, r68 opened depth-5 above r63..r67, and r63 opened depth-4 above r58..r62. Smoke at depth 7 with 2,2 children confirmed 127 internal-parent occurrences with parent_local_flops 8122 vs top 64 and 127 internal occurrences carrying flops.

**Validation**

- `cargo test --test pipeline recursive_hierarchy_parents_can_emit_local_flops_at_depth_7`
- `cargo test --bin tool_matrix`
- `cargo run --bin tool_matrix -- --phase4-hierarchy-gate --out /tmp/anvil-tool-matrix-phase4-hierarchy-r78 --yosys-mode both`
  - 183 scenarios, 4 designs/scenario, 732 total designs
  - `coverage_gaps = []`, `Verilator 732/0`, `Yosys without-abc 732/0`, `Yosys with-abc 732/0`
  - `saw_recursive_hierarchy_depth_7_parent_local_flops = true`
- `cargo fmt`, `cargo clippy`, `mdbook build` all clean.

**Impact**

- Depth-7 axis now open. Future r79..r82 close the depth-7 sweep with mixed-support child inputs, parent-port-composed parent outputs, stateful parent-port-composed parent outputs, and stateful mixed-support child inputs (mirroring depth-3..6 sweeps; mixed-support cells will adopt the same 2,2 calibration as depth 6).
- Batch step 6/10. No push.

**Files touched**

- src/bin/tool_matrix.rs, tests/pipeline.rs, CHANGES.md, DEVELOPMENT_NOTES.md, MEMORY.md, CODEBASE_ANALYSIS.md, USER_GUIDE.md, README.md, ROADMAP.md, book/src/hierarchy.md, book/src/architecture.md.

## 2026-05-07-phase4-recursive-depth-6-stateful-mixed-support-child-inputs — Close depth-6 sweep with recursive non-top stateful unregistered parent-composed mixed-support child inputs to exact hierarchy depth 6 without helpers (2,2 calibrated)

**Landed as:** ed4b9f3

**What changed**

- Added focused proof `recursive_hierarchy_stateful_parent_composed_routes_mix_parent_ports_at_depth_6_without_helpers` (depth 6, 2,2 children, child_input_cone_prob=1.0, parent_flop_prob=1.0, max_flops=64).
- Added matrix scenario `phase4_recur_d6_stateful_parent_composed_mixed_support_child_input` per ConstructionStrategy via `phase4_recursive_d6_stateful_parent_composed_mixed_support_focus_config` (built atop r74's 2,2 helper by promoting parent-flop probability and budget).
- Added `saw_recursive_hierarchy_depth_6_stateful_parent_composed_mixed_support_child_inputs` coverage fact and matching gap message.
- Bank counts: 180 scenarios / 720 designs.
- Ran full Phase 4 hierarchy gate at `/tmp/anvil-tool-matrix-phase4-hierarchy-r77/tool_matrix_report.json`.

**Why**

- Closes the depth-6 sweep. r73 opened depth-6 with parent flops, r74 extended with mixed-support child inputs (2,2 calibrated), r75 with parent-port-composed parent outputs, r76 with stateful parent-port-composed parent outputs. r77 closes the sweep with stateful mixed-support child inputs, mirroring r72 (depth 5), r67 (depth 4), and r62 (depth 3).
- Calibration choice mirrors r74 — depth-6 mixed-support cells use 2,2 child instances (not 4,4 like depths 3-5) to keep each gate within a safe slice. Smoke at depth 6 with 2,2 + flops + child-input cone confirmed 63 internal occurrences with parent_local_flops 4032 vs top 64, child_input_bindings_from_stateful_parent_composed_mixed_support 74 vs top 1, fraction 0.454.

**Validation**

- `cargo test --test pipeline recursive_hierarchy_stateful_parent_composed_routes_mix_parent_ports_at_depth_6_without_helpers`
- `cargo test --bin tool_matrix`
- `cargo run --bin tool_matrix -- --phase4-hierarchy-gate --out /tmp/anvil-tool-matrix-phase4-hierarchy-r77 --yosys-mode both`
  - 180 scenarios, 4 designs/scenario, 720 total designs
  - `coverage_gaps = []`, `Verilator 720/0`, `Yosys without-abc 720/0`, `Yosys with-abc 720/0`
  - `saw_recursive_hierarchy_depth_6_stateful_parent_composed_mixed_support_child_inputs = true`
- `cargo fmt`, `cargo clippy`, `mdbook build` all clean.

**Impact**

- The depth-6 axis is now structurally complete: all five mixed-support cells (parent-flops, mixed-support child inputs, parent-port-composed outputs, stateful parent-port-composed outputs, stateful mixed-support child inputs) are gated as first-class facts at exact hierarchy depth 6. Mirrors closed depth-3 (r58..r62), depth-4 (r63..r67), and depth-5 (r68..r72) sweeps.
- Batch step 5/10 — half of the BMSC=10 batch complete. No push (per BATCH MODE policy).

**Files touched**

- src/bin/tool_matrix.rs, tests/pipeline.rs, CHANGES.md, DEVELOPMENT_NOTES.md, MEMORY.md, CODEBASE_ANALYSIS.md, USER_GUIDE.md, README.md, ROADMAP.md, book/src/hierarchy.md, book/src/architecture.md.

## 2026-05-06-phase4-recursive-depth-6-stateful-parent-port-composed-outputs — Push recursive non-top stateful parent-port-composed parent outputs to exact hierarchy depth 6 without helpers

**Landed as:** 89a406d

**What changed**

- Added focused proof `recursive_hierarchy_stateful_parent_outputs_mix_parent_ports_at_depth_6_without_helpers` (depth 6, 2,2 children, parent_flop_prob=1.0, max_flops=64).
- Added matrix scenario `phase4_recur_d6_stateful_parent_port_composed_output` per ConstructionStrategy via `phase4_recursive_d6_stateful_parent_port_composed_output_focus_config`.
- Added `saw_recursive_hierarchy_depth_6_stateful_parent_port_composed_outputs` coverage fact and matching gap message.
- Bank counts: 177 scenarios / 708 designs.
- Ran full Phase 4 hierarchy gate at `/tmp/anvil-tool-matrix-phase4-hierarchy-r76/tool_matrix_report.json`.

**Why**

- Fourth slice of the depth-6 sweep, mirroring r71 (depth 5), r66 (depth 4), r61 (depth 3). Smoke at depth 6 confirmed 63 occurrences with parent_local_flops 4028 vs top 64, parent_port_composed_outputs_through_parent_flops 890 vs 109, fraction 0.927.

**Validation**

- `cargo test --test pipeline recursive_hierarchy_stateful_parent_outputs_mix_parent_ports_at_depth_6_without_helpers`
- `cargo test --bin tool_matrix`
- `cargo run --bin tool_matrix -- --phase4-hierarchy-gate --out /tmp/anvil-tool-matrix-phase4-hierarchy-r76 --yosys-mode both`
  - 177 scenarios, 4 designs/scenario, 708 total designs
  - `coverage_gaps = []`, `Verilator 708/0`, `Yosys without-abc 708/0`, `Yosys with-abc 708/0`
  - `saw_recursive_hierarchy_depth_6_stateful_parent_port_composed_outputs = true`
- `cargo fmt`, `cargo clippy`, `mdbook build` all clean.

**Impact**

- Depth-6 axis now has four first-class coverage facts. Only one cell remains to close depth-6: stateful unregistered parent-composed mixed-support child inputs (r77).
- Batch step 4/10. No push.

**Files touched**

- src/bin/tool_matrix.rs, tests/pipeline.rs, CHANGES.md, DEVELOPMENT_NOTES.md, MEMORY.md, CODEBASE_ANALYSIS.md, USER_GUIDE.md, README.md, ROADMAP.md, book/src/hierarchy.md, book/src/architecture.md.

## 2026-05-06-phase4-recursive-depth-6-parent-port-composed-outputs — Push recursive non-top parent-port-composed parent outputs to exact hierarchy depth 6 without helpers or parent-local state

**Landed as:** 96fdd4f

**What changed**

- Added focused proof `recursive_hierarchy_parent_outputs_mix_parent_ports_at_depth_6_without_helpers` (depth 6, 2,2 children, no helpers, no flops, no child-input cones).
- Added matrix scenario `phase4_recur_d6_parent_port_composed_output` per ConstructionStrategy via `phase4_recursive_d6_parent_port_composed_output_focus_config` (2,2 children at depth 6).
- Added `saw_recursive_hierarchy_depth_6_parent_port_composed_outputs` coverage fact and matching gap message.
- Updated bank counts: 174 scenarios / 696 designs.
- Ran full Phase 4 hierarchy gate at `/tmp/anvil-tool-matrix-phase4-hierarchy-r75/tool_matrix_report.json`.

**Why**

- Third slice of the depth-6 sweep, mirroring r70 (depth 5), r65 (depth 4), r60 (depth 3). The parent-port-composed parent-output cells already use 2,2 children at all depths, so no calibration drift here. Smoke at depth 6 with 2,2 confirmed 63 internal-parent occurrences with `hierarchy_parent_port_composed_outputs = 1008` versus 168 top-only and `hierarchy_parent_port_composed_output_fraction = 1.0`.

**Validation**

- `cargo test --test pipeline recursive_hierarchy_parent_outputs_mix_parent_ports_at_depth_6_without_helpers`
- `cargo test --bin tool_matrix`
- `cargo run --bin tool_matrix -- --phase4-hierarchy-gate --out /tmp/anvil-tool-matrix-phase4-hierarchy-r75 --yosys-mode both`
  - 174 scenarios, 4 designs/scenario, 696 total designs
  - `coverage_gaps = []`
  - `Verilator 696/0`, `Yosys without-abc 696/0`, `Yosys with-abc 696/0`
  - `saw_recursive_hierarchy_depth_6_parent_port_composed_outputs = true`
- `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings`, `mdbook build book` all clean.

**Impact**

- Depth-6 axis now has three first-class coverage facts: parent-flops (r73), mixed-support child inputs (r74, 2,2 calibrated), parent-port-composed outputs (r75). Future r76..r77 close the depth-6 sweep.
- Batch step 3/10. No push.

**Files touched**

- src/bin/tool_matrix.rs, tests/pipeline.rs, CHANGES.md, DEVELOPMENT_NOTES.md, MEMORY.md, CODEBASE_ANALYSIS.md, USER_GUIDE.md, README.md, ROADMAP.md, book/src/hierarchy.md, book/src/architecture.md.

## 2026-05-06-phase4-recursive-depth-6-mixed-support-child-inputs — Push recursive non-top mixed-support child inputs to exact hierarchy depth 6 without helpers

**Landed as:** 4834f98

**What changed**

- Added the focused recursive integration proof `recursive_hierarchy_parent_composed_routes_mix_parent_ports_at_depth_6_without_helpers` across all four `ConstructionStrategy` variants. It uses `min/max_hierarchy_depth = 6`, `2,2` child-instance bounds (calibration choice — see Why), and `hierarchy_child_input_cone_prob = 1.0`, isolating the unregistered parent-composed mixed-support child-input surface across five intermediate parent layers below the top.
- Added the new Phase 4 matrix scenario `phase4_recur_d6_parent_composed_mixed_support_child_input` per `ConstructionStrategy` via the new `phase4_recursive_d6_parent_composed_mixed_support_focus_config` helper.
- Added the `saw_recursive_hierarchy_depth_6_mixed_support_child_inputs` coverage fact and matching gap message.
- Updated the Phase 4 hierarchy run-plan and coverage tests from `168` scenarios / `672` designs to `171` scenarios / `684` designs.
- Ran the full Phase 4 hierarchy gate at `/tmp/anvil-tool-matrix-phase4-hierarchy-r74/tool_matrix_report.json`.

**Why**

- Second slice of the depth-6 sweep, mirroring how r69 followed r68 at depth 5, r64 followed r63 at depth 4, and r59 followed r58 at depth 3.
- **Calibration choice — bounds reduced from 4,4 (d3-d5 pattern) to 2,2 at d6.** Smoke at depth 6 with 4,4 child instances initially confirmed 1365 non-top internal-parent occurrences with `child_input_bindings_from_parent_composed_logic = 6553` versus 3 top-only — but the downstream-clean gate (Verilator + Yosys without-ABC + Yosys with-ABC) ran 22+ minutes on a single design, projecting to ~10h per gate. The 2,2 calibration at depth 6 yields 63 internal occurrences (matching r73's parent-flop scenario) while still proving the mixed-support surface at exact depth 6 cleanly. This is a slice-time calibration choice, not a generator change. Future depth-6 mixed-support cells (r77 stateful) will adopt the same 2,2 calibration.

**Validation**

- `cargo test --test pipeline recursive_hierarchy_parent_composed_routes_mix_parent_ports_at_depth_6_without_helpers`
- `cargo test --bin tool_matrix`
- `cargo run --bin tool_matrix -- --phase4-hierarchy-gate --out /tmp/anvil-tool-matrix-phase4-hierarchy-r74 --yosys-mode both`
  - `171` scenarios
  - `4` designs/scenario
  - `684` total designs
  - `artifact_kind = "design"`
  - `coverage_gaps = []`
  - `Verilator pass/fail = 684/0`
  - `Yosys without-abc pass/fail = 684/0`
  - `Yosys with-abc pass/fail = 684/0`
  - `saw_recursive_hierarchy_depth_6_mixed_support_child_inputs = true`

**Impact**

- The Phase 4 hierarchy gate now explicitly proves recursive non-top unregistered parent-composed mixed-support child inputs at exact hierarchy depth 6 without helpers. `r73` becomes the previous bank (opening the depth-6 axis with parent-flops); `r74` is the current full downstream-clean Phase 4 hierarchy bank, extending the depth-6 axis with the second cell. Future r75..r77 slices will close the depth-6 sweep.
- Batch step 2/10.

**Files touched**

- `src/bin/tool_matrix.rs`
- `tests/pipeline.rs`
- `CHANGES.md`
- `DEVELOPMENT_NOTES.md`
- `MEMORY.md`
- `CODEBASE_ANALYSIS.md`
- `USER_GUIDE.md`
- `README.md`
- `ROADMAP.md`
- `book/src/hierarchy.md`
- `book/src/architecture.md`

## 2026-05-05-phase4-recursive-depth-6-parent-local-flops — Open depth-6 axis with recursive non-top parent-local flops at exact hierarchy depth 6

**Landed as:** 88854fd

**What changed**

- Added the focused recursive integration proof `recursive_hierarchy_parents_can_emit_local_flops_at_depth_6` across all four `ConstructionStrategy` variants. It uses `min/max_hierarchy_depth = 6` and `2,2` child-instance bounds, isolating the parent-flop surface across five intermediate parent layers below the top.
- Added the new Phase 4 matrix scenario `phase4_recur_d6_parent_state` per `ConstructionStrategy` via the new `phase4_recursive_d6_parent_state_focus_config` helper.
- Added the `saw_recursive_hierarchy_depth_6_parent_local_flops` coverage fact and matching gap message.
- Updated the Phase 4 hierarchy run-plan and coverage tests from `165` scenarios / `660` designs to `168` scenarios / `672` designs, and added `(6, 6)` to the recognized `range_depths` set.
- Ran the full Phase 4 hierarchy gate at `/tmp/anvil-tool-matrix-phase4-hierarchy-r73/tool_matrix_report.json`.

**Why**

- First slice of the depth-6 sweep, opening the axis above the closed depth-5 sweep (r68..r72) just as r68 opened depth-5 above the closed depth-4 sweep (r63..r67) and r63 opened depth-4 above the closed depth-3 sweep (r58..r62). Smoke at depth 6 with 2,2 child instances confirmed 63 non-top internal-parent occurrences with `hierarchy_parent_local_flops = 4028` versus `top_local_flops = 64` and 63 internal occurrences carrying parent-local flops.

**Validation**

- `cargo test --test pipeline recursive_hierarchy_parents_can_emit_local_flops_at_depth_6`
- `cargo test --bin tool_matrix`
- `cargo run --bin tool_matrix -- --phase4-hierarchy-gate --out /tmp/anvil-tool-matrix-phase4-hierarchy-r73 --yosys-mode both`
  - `168` scenarios
  - `4` designs/scenario
  - `672` total designs
  - `artifact_kind = "design"`
  - `coverage_gaps = []`
  - `Verilator pass/fail = 672/0`
  - `Yosys without-abc pass/fail = 672/0`
  - `Yosys with-abc pass/fail = 672/0`
  - `saw_recursive_hierarchy_depth_6_parent_local_flops = true`

**Impact**

- The Phase 4 hierarchy gate now explicitly proves recursive non-top parent-local flops at exact hierarchy depth 6. `r72` becomes the previous bank (closing the depth-5 sweep); `r73` is the current full downstream-clean Phase 4 hierarchy bank, opening the depth-6 axis. Future r74..r77 slices will close the depth-6 sweep with mixed-support child inputs, parent-port-composed parent outputs, stateful parent-port-composed parent outputs, and stateful unregistered parent-composed mixed-support child inputs — mirroring the depth-3 (r58..r62), depth-4 (r63..r67), and depth-5 (r68..r72) sweeps.

**Files touched**

- `src/bin/tool_matrix.rs`
- `tests/pipeline.rs`
- `CHANGES.md`
- `DEVELOPMENT_NOTES.md`
- `MEMORY.md`
- `CODEBASE_ANALYSIS.md`
- `USER_GUIDE.md`
- `README.md`
- `ROADMAP.md`
- `book/src/hierarchy.md`
- `book/src/architecture.md`

## 2026-05-05-phase4-recursive-depth-5-stateful-mixed-support-child-inputs — Close depth-5 sweep with recursive non-top stateful unregistered parent-composed mixed-support child inputs to exact hierarchy depth 5 without helpers

**Landed as:** c646f50

**What changed**

- Added the focused recursive integration proof `recursive_hierarchy_stateful_parent_composed_routes_mix_parent_ports_at_depth_5_without_helpers` across all four `ConstructionStrategy` variants. It uses `min/max_hierarchy_depth = 5`, `4,4` child-instance bounds, `hierarchy_child_input_cone_prob = 1.0`, and `hierarchy_parent_flop_prob = 1.0` (max 64 flops/module), isolating the stateful unregistered parent-composed mixed-support child-input surface across four intermediate parent layers below the top.
- Added the new Phase 4 matrix scenario `phase4_recur_d5_stateful_parent_composed_mixed_support_child_input` per `ConstructionStrategy` via the new `phase4_recursive_d5_stateful_parent_composed_mixed_support_focus_config` helper (built atop the r69 helper by promoting parent-flop probability and budget).
- Added the `saw_recursive_hierarchy_depth_5_stateful_parent_composed_mixed_support_child_inputs` coverage fact and matching gap message.
- Updated the Phase 4 hierarchy run-plan and coverage tests from `162` scenarios / `648` designs to `165` scenarios / `660` designs.
- Ran the full Phase 4 hierarchy gate at `/tmp/anvil-tool-matrix-phase4-hierarchy-r72/tool_matrix_report.json`.

**Why**

- Closes the depth-5 sweep. r68 opened the depth-5 axis with parent-local flops, r69 extended with mixed-support child inputs, r70 with parent-port-composed parent outputs, r71 with stateful parent-port-composed parent outputs. r72 closes the sweep with stateful mixed-support child inputs, mirroring how r67 closed depth 4 and r62 closed depth 3. Smoke at depth 5 with 4,4 child instances confirmed 341 non-top internal-parent occurrences with `hierarchy_parent_local_flops = 21820` versus `top_local_flops = 64`, `child_input_bindings_from_parent_composed_logic = 1777` versus 3 top-only, `child_input_bindings_from_stateful_parent_composed_mixed_support = 1460` versus 2 top-only, and `stateful_parent_composed_mixed_support_child_input_binding_fraction = 0.642`.

**Validation**

- `cargo test --test pipeline recursive_hierarchy_stateful_parent_composed_routes_mix_parent_ports_at_depth_5_without_helpers`
- `cargo test --bin tool_matrix`
- `cargo run --bin tool_matrix -- --phase4-hierarchy-gate --out /tmp/anvil-tool-matrix-phase4-hierarchy-r72 --yosys-mode both`
  - `165` scenarios
  - `4` designs/scenario
  - `660` total designs
  - `artifact_kind = "design"`
  - `coverage_gaps = []`
  - `Verilator pass/fail = 660/0`
  - `Yosys without-abc pass/fail = 660/0`
  - `Yosys with-abc pass/fail = 660/0`
  - `saw_recursive_hierarchy_depth_5_stateful_parent_composed_mixed_support_child_inputs = true`
- Commit-workflow hygiene: pending final gate in this commit.

**Impact**

- The Phase 4 hierarchy gate now explicitly proves all five mixed-support cells at exact hierarchy depth 5: parent-local flops, unregistered parent-composed mixed-support child inputs, unregistered parent-port-composed parent outputs, stateful parent-port-composed parent outputs, and stateful unregistered parent-composed mixed-support child inputs. The depth-5 sweep is now structurally complete, mirroring the closed depth-3 sweep (r58..r62) and depth-4 sweep (r63..r67). `r71` becomes the previous bank (depth-5 stateful parent-port-composed outputs); `r72` is the current full downstream-clean Phase 4 hierarchy bank closing the depth-5 sweep.

**Files touched**

- `src/bin/tool_matrix.rs`
- `tests/pipeline.rs`
- `CHANGES.md`
- `DEVELOPMENT_NOTES.md`
- `MEMORY.md`
- `CODEBASE_ANALYSIS.md`
- `USER_GUIDE.md`
- `README.md`
- `ROADMAP.md`
- `book/src/hierarchy.md`
- `book/src/architecture.md`

## 2026-05-05-phase4-recursive-depth-5-stateful-parent-port-composed-outputs — Push recursive non-top stateful parent-port-composed parent outputs to exact hierarchy depth 5 without helpers

**Landed as:** ef6e5bd

**What changed**

- Added the focused recursive integration proof `recursive_hierarchy_stateful_parent_outputs_mix_parent_ports_at_depth_5_without_helpers` across all four `ConstructionStrategy` variants. It uses `min/max_hierarchy_depth = 5`, `2,2` child-instance bounds, and `hierarchy_parent_flop_prob = 1.0` (max 64 flops/module), isolating the stateful parent-port-composed parent-output surface across four intermediate parent layers below the top.
- Added the new Phase 4 matrix scenario `phase4_recur_d5_stateful_parent_port_composed_output` per `ConstructionStrategy` via the new `phase4_recursive_d5_stateful_parent_port_composed_output_focus_config` helper (built atop the r70 helper by promoting parent-flop probability and budget).
- Added the `saw_recursive_hierarchy_depth_5_stateful_parent_port_composed_outputs` coverage fact and matching gap message.
- Updated the Phase 4 hierarchy run-plan and coverage tests from `159` scenarios / `636` designs to `162` scenarios / `648` designs.
- Ran the full Phase 4 hierarchy gate at `/tmp/anvil-tool-matrix-phase4-hierarchy-r71/tool_matrix_report.json`.

**Why**

- Fourth slice of the depth-5 sweep: r68 opened the depth-5 axis with parent-local flops, r69 extended it with mixed-support child inputs, r70 extended it with parent-port-composed parent outputs. r71 extends it with stateful parent-port-composed parent outputs, mirroring how r66 followed r65 at depth 4 and r61 followed r60 at depth 3. Smoke at depth 5 with 2,2 child instances confirmed 31 non-top internal-parent occurrences with `hierarchy_parent_local_flops = 1980` versus `top_local_flops = 64`, `hierarchy_parent_port_composed_outputs = 340` versus 68 top-only, `hierarchy_parent_port_composed_outputs_through_parent_flops = 336` versus 64 top-only, and `hierarchy_parent_port_composed_parent_flop_output_fraction = 0.988`.

**Validation**

- `cargo test --test pipeline recursive_hierarchy_stateful_parent_outputs_mix_parent_ports_at_depth_5_without_helpers`
- `cargo test --bin tool_matrix`
- `cargo run --bin tool_matrix -- --phase4-hierarchy-gate --out /tmp/anvil-tool-matrix-phase4-hierarchy-r71 --yosys-mode both`
  - `162` scenarios
  - `4` designs/scenario
  - `648` total designs
  - `artifact_kind = "design"`
  - `coverage_gaps = []`
  - `Verilator pass/fail = 648/0`
  - `Yosys without-abc pass/fail = 648/0`
  - `Yosys with-abc pass/fail = 648/0`
  - `saw_recursive_hierarchy_depth_5_stateful_parent_port_composed_outputs = true`
- Commit-workflow hygiene: pending final gate in this commit.

**Impact**

- The Phase 4 hierarchy gate now explicitly proves recursive non-top stateful parent-port-composed parent outputs at exact hierarchy depth 5 without helpers. `r70` becomes the previous bank (depth-5 unregistered parent-port-composed outputs); `r71` is the current full downstream-clean Phase 4 hierarchy bank. Only one cell remains to close the depth-5 sweep: stateful unregistered parent-composed mixed-support child inputs (depth-3 territory r62 / depth-4 territory r67).

**Files touched**

- `src/bin/tool_matrix.rs`
- `tests/pipeline.rs`
- `CHANGES.md`
- `DEVELOPMENT_NOTES.md`
- `MEMORY.md`
- `CODEBASE_ANALYSIS.md`
- `USER_GUIDE.md`
- `README.md`
- `ROADMAP.md`
- `book/src/hierarchy.md`
- `book/src/architecture.md`

## 2026-05-05-phase4-recursive-depth-5-parent-port-composed-outputs — Push recursive non-top parent-port-composed parent outputs to exact hierarchy depth 5 without helpers or parent-local state

**Landed as:** b5219b7

**What changed**

- Added the focused recursive integration proof `recursive_hierarchy_parent_outputs_mix_parent_ports_at_depth_5_without_helpers` across all four `ConstructionStrategy` variants. It uses `min/max_hierarchy_depth = 5` and `2,2` child-instance bounds, isolating the unregistered parent-port-composed parent-output surface across four intermediate parent layers below the top.
- Added the new Phase 4 matrix scenario `phase4_recur_d5_parent_port_composed_output` per `ConstructionStrategy` via the new `phase4_recursive_d5_parent_port_composed_output_focus_config` helper.
- Added the `saw_recursive_hierarchy_depth_5_parent_port_composed_outputs` coverage fact and matching gap message.
- Updated the Phase 4 hierarchy run-plan and coverage tests from `156` scenarios / `624` designs to `159` scenarios / `636` designs.
- Ran the full Phase 4 hierarchy gate at `/tmp/anvil-tool-matrix-phase4-hierarchy-r70/tool_matrix_report.json`.

**Why**

- The depth-5 axis is being banked one focused proof at a time. r68 opened it with parent-local flops, r69 extended it with mixed-support child inputs. r70 extends it again with parent-port-composed parent outputs, mirroring how r65 followed r64 at depth 4 and r60 followed r59 at depth 3. Smoke at depth 5 with 2,2 child instances confirmed 31 non-top internal-parent occurrences with `hierarchy_parent_port_composed_outputs = 390` vs `top_parent_port_composed_outputs = 78` and a `hierarchy_parent_port_composed_output_fraction = 1.0`.

**Validation**

- `cargo test --test pipeline recursive_hierarchy_parent_outputs_mix_parent_ports_at_depth_5_without_helpers`
- `cargo test --bin tool_matrix`
- `cargo run --bin tool_matrix -- --phase4-hierarchy-gate --out /tmp/anvil-tool-matrix-phase4-hierarchy-r70 --yosys-mode both`
  - `159` scenarios
  - `4` designs/scenario
  - `636` total designs
  - `artifact_kind = "design"`
  - `coverage_gaps = []`
  - `Verilator pass/fail = 636/0`
  - `Yosys without-abc pass/fail = 636/0`
  - `Yosys with-abc pass/fail = 636/0`
  - `saw_recursive_hierarchy_depth_5_parent_port_composed_outputs = true`
- Commit-workflow hygiene: pending final gate in this commit.

**Impact**

- The Phase 4 hierarchy gate now explicitly proves recursive non-top unregistered parent-port-composed parent outputs at exact hierarchy depth 5 without helpers or parent-local state. `r69` becomes the previous bank (depth-5 mixed-support child inputs); `r70` is the current full downstream-clean Phase 4 hierarchy bank.

**Files touched**

- `src/bin/tool_matrix.rs`
- `tests/pipeline.rs`
- `CHANGES.md`
- `DEVELOPMENT_NOTES.md`
- `MEMORY.md`
- `CODEBASE_ANALYSIS.md`
- `USER_GUIDE.md`
- `README.md`
- `ROADMAP.md`
- `book/src/hierarchy.md`
- `book/src/architecture.md`

## 2026-05-04-phase4-recursive-depth-5-mixed-support-child-inputs — Push recursive non-top mixed-support child inputs to exact hierarchy depth 5 without helpers

**Landed as:** fa05fb5

**What changed**

- Added the focused recursive integration proof `recursive_hierarchy_parent_composed_routes_mix_parent_ports_at_depth_5_without_helpers` across all four `ConstructionStrategy` variants. It uses `min/max_hierarchy_depth = 5` and `4,4` child-instance bounds, isolating the unregistered parent-composed mixed-support surface across four intermediate parent layers below the top.
- Added the new Phase 4 matrix scenario `phase4_recur_d5_parent_composed_mixed_support_child_input` per `ConstructionStrategy` via the new `phase4_recursive_d5_parent_composed_mixed_support_focus_config` helper.
- Added the `saw_recursive_hierarchy_depth_5_mixed_support_child_inputs` coverage fact and matching gap message.
- Updated the Phase 4 hierarchy run-plan and coverage tests from `153` scenarios / `612` designs to `156` scenarios / `624` designs.
- Ran the full Phase 4 hierarchy gate at `/tmp/anvil-tool-matrix-phase4-hierarchy-r69/tool_matrix_report.json`.

**Why**

- The depth-5 axis was opened in r68 with parent flops. r69 extends the depth-5 coverage to the unregistered parent-composed mixed-support child-input surface, mirroring how r64 followed r63 at depth 4 and r59 followed r58 at depth 3. Smoke at depth 5 confirmed 341 internal module occurrences with 1457 hierarchy-wide vs 3 top-only mixed-support bindings and 1599 vs 3 parent-composed bindings.

**Validation**

- `cargo test --test pipeline recursive_hierarchy_parent_composed_routes_mix_parent_ports_at_depth_5_without_helpers`
- `cargo test --bin tool_matrix`
- `cargo run --bin tool_matrix -- --phase4-hierarchy-gate --out /tmp/anvil-tool-matrix-phase4-hierarchy-r69 --yosys-mode both`
  - `156` scenarios
  - `4` designs/scenario
  - `624` total designs
  - `artifact_kind = "design"`
  - `coverage_gaps = []`
  - `Verilator pass/fail = 624/0`
  - `Yosys without-abc pass/fail = 624/0`
  - `Yosys with-abc pass/fail = 624/0`
  - `saw_recursive_hierarchy_depth_5_mixed_support_child_inputs = true`
- Commit-workflow hygiene: pending final gate in this commit.

**Impact**

- The Phase 4 hierarchy gate now explicitly proves recursive non-top unregistered parent-composed mixed-support child inputs at exact hierarchy depth 5. `r68` becomes the previous bank (depth-5 parent-flops); `r69` is the current full downstream-clean Phase 4 hierarchy bank.

**Files touched**

- `src/bin/tool_matrix.rs`
- `tests/pipeline.rs`
- `CHANGES.md`
- `DEVELOPMENT_NOTES.md`
- `MEMORY.md`
- `CODEBASE_ANALYSIS.md`
- `USER_GUIDE.md`
- `README.md`
- `ROADMAP.md`
- `book/src/hierarchy.md`
- `book/src/architecture.md`

## 2026-05-04-phase4-recursive-depth-5-parent-local-flops — Open depth-5 axis with recursive non-top parent-local flops

**Landed as:** 063f196

**What changed**

- Added the focused recursive integration proof `recursive_hierarchy_parents_can_emit_local_flops_at_depth_5` across all four `ConstructionStrategy` variants. It uses `min/max_hierarchy_depth = 5` and `2,2` child-instance bounds, isolating the parent-flop surface across four intermediate parent layers below the top.
- Added the new Phase 4 matrix scenario `phase4_recur_d5_parent_state` per `ConstructionStrategy` via the new `phase4_recursive_d5_parent_state_focus_config` helper.
- Added the `saw_recursive_hierarchy_depth_5_parent_local_flops` coverage fact and matching gap message.
- Updated the Phase 4 hierarchy run-plan and coverage tests from `150` scenarios / `600` designs to `153` scenarios / `612` designs. Extended the `range_depths` assertion to include `(5, 5)`.
- Ran the full Phase 4 hierarchy gate at `/tmp/anvil-tool-matrix-phase4-hierarchy-r68/tool_matrix_report.json`.

**Why**

- The depth-4 sweep is structurally complete (r63–r67 covered all five mixed-support cells at depth 4). r68 opens the depth-5 axis with the simplest surface — parent flops at depth 5 — to prove the recursive generator handles five hierarchy layers cleanly. Smoke at depth 5 confirmed 31 internal modules with 1984 hierarchy-wide vs 64 top-only parent-local flops across 4 intermediate parent layers.

**Validation**

- `cargo test --test pipeline recursive_hierarchy_parents_can_emit_local_flops_at_depth_5`
- `cargo test --bin tool_matrix`
- `cargo run --bin tool_matrix -- --phase4-hierarchy-gate --out /tmp/anvil-tool-matrix-phase4-hierarchy-r68 --yosys-mode both`
  - `153` scenarios
  - `4` designs/scenario
  - `612` total designs
  - `artifact_kind = "design"`
  - `coverage_gaps = []`
  - `Verilator pass/fail = 612/0`
  - `Yosys without-abc pass/fail = 612/0`
  - `Yosys with-abc pass/fail = 612/0`
  - `saw_recursive_hierarchy_depth_5_parent_local_flops = true`
- Commit-workflow hygiene: pending final gate in this commit.

**Impact**

- The Phase 4 hierarchy gate now explicitly proves recursive non-top parent-local flops at exact hierarchy depth 5, opening the depth-5 axis on top of the completed depth-4 sweep. `r67` becomes the previous bank (closed depth-4 sweep); `r68` is the current full downstream-clean Phase 4 hierarchy bank with depth-5 parent-flop evidence.

**Files touched**

- `src/bin/tool_matrix.rs`
- `tests/pipeline.rs`
- `CHANGES.md`
- `DEVELOPMENT_NOTES.md`
- `MEMORY.md`
- `CODEBASE_ANALYSIS.md`
- `USER_GUIDE.md`
- `README.md`
- `ROADMAP.md`
- `book/src/hierarchy.md`
- `book/src/architecture.md`

## 2026-05-04-phase4-recursive-depth-4-stateful-parent-composed-mixed-support-child-inputs — Close depth-4 sweep with stateful parent-composed mixed-support child inputs

**Landed as:** 28144ac

**What changed**

- Added the focused recursive integration proof `recursive_hierarchy_stateful_parent_composed_routes_mix_parent_ports_at_depth_4_without_helpers` across all four `ConstructionStrategy` variants. It uses `min/max_hierarchy_depth = 4`, `4,4` child-instance bounds, `hierarchy_child_input_cone_prob = 1.0`, and `hierarchy_parent_flop_prob = 1.0`, isolating unregistered parent-composed child-input cones that mix parent ports + child outputs + parent-local Qs across three intermediate parent layers below the top.
- Added the new Phase 4 matrix scenario `phase4_recur_d4_stateful_parent_composed_mixed_support_child_input` per `ConstructionStrategy` via the new `phase4_recursive_d4_stateful_parent_composed_mixed_support_focus_config` helper.
- Added the `saw_recursive_hierarchy_depth_4_stateful_parent_composed_mixed_support_child_inputs` coverage fact and matching gap message.
- Updated the Phase 4 hierarchy run-plan and coverage tests from `147` scenarios / `588` designs to `150` scenarios / `600` designs.
- Ran the full Phase 4 hierarchy gate at `/tmp/anvil-tool-matrix-phase4-hierarchy-r67/tool_matrix_report.json`.

**Why**

- The depth-4 sweep is now structurally complete — covers parent-flops (r63), no-state mixed-support child inputs (r64), no-state parent-port-composed outputs (r65), stateful parent-port-composed outputs (r66), and stateful child-input mixed-support (r67), mirroring the depth-3 sweep r58→r62. Smoke at depth 4 confirmed 85 internal modules with 471 hierarchy-wide vs 3 top-only stateful-parent-composed-mixed-support bindings and 5438 vs 64 parent-local flops.

**Validation**

- `cargo test --test pipeline recursive_hierarchy_stateful_parent_composed_routes_mix_parent_ports_at_depth_4_without_helpers`
- `cargo test --bin tool_matrix`
- `cargo run --bin tool_matrix -- --phase4-hierarchy-gate --out /tmp/anvil-tool-matrix-phase4-hierarchy-r67 --yosys-mode both`
  - `150` scenarios
  - `4` designs/scenario
  - `600` total designs
  - `artifact_kind = "design"`
  - `coverage_gaps = []`
  - `Verilator pass/fail = 600/0`
  - `Yosys without-abc pass/fail = 600/0`
  - `Yosys with-abc pass/fail = 600/0`
  - `saw_recursive_hierarchy_depth_4_stateful_parent_composed_mixed_support_child_inputs = true`
- Commit-workflow hygiene: pending final gate in this commit.

**Impact**

- The Phase 4 hierarchy gate now explicitly proves recursive non-top stateful parent-composed mixed-support child inputs at exact hierarchy depth 4, completing the depth-4 sweep. `r66` becomes the previous bank (depth-4 stateful parent-port-composed outputs); `r67` is the current full downstream-clean Phase 4 hierarchy bank.

**Files touched**

- `src/bin/tool_matrix.rs`
- `tests/pipeline.rs`
- `CHANGES.md`
- `DEVELOPMENT_NOTES.md`
- `MEMORY.md`
- `CODEBASE_ANALYSIS.md`
- `USER_GUIDE.md`
- `README.md`
- `ROADMAP.md`
- `book/src/hierarchy.md`
- `book/src/architecture.md`

## 2026-05-04-phase4-recursive-depth-4-stateful-parent-port-composed-outputs — Push recursive non-top stateful parent-port-composed parent outputs to exact hierarchy depth 4 without helpers

**Landed as:** df49f55

**What changed**

- Added the focused recursive integration proof `recursive_hierarchy_stateful_parent_outputs_mix_parent_ports_at_depth_4_without_helpers` across all four `ConstructionStrategy` variants. It uses `min/max_hierarchy_depth = 4`, `2,2` child-instance bounds, and `hierarchy_parent_flop_prob = 1.0`, isolating parent-output cones that mix parent ports + child outputs + parent-local Qs across three intermediate parent layers below the top.
- Added the new Phase 4 matrix scenario `phase4_recur_d4_stateful_parent_port_composed_output` per `ConstructionStrategy` via the new `phase4_recursive_d4_stateful_parent_port_composed_output_focus_config` helper.
- Added the `saw_recursive_hierarchy_depth_4_stateful_parent_port_composed_outputs` coverage fact and matching gap message.
- Updated the Phase 4 hierarchy run-plan and coverage tests from `144` scenarios / `576` designs to `147` scenarios / `588` designs.
- Ran the full Phase 4 hierarchy gate at `/tmp/anvil-tool-matrix-phase4-hierarchy-r66/tool_matrix_report.json`.

**Why**

- The depth-4 axis now covers parent-flops (r63), unregistered parent-composed mixed-support child inputs (r64), and no-state parent-port-composed parent outputs (r65). r66 extends it to the stateful parent-port-composed parent-output surface (r61's depth-3 territory), mirroring how r61 followed r60 at depth 3. Smoke at depth 4 confirmed 15 internal modules with 128 hierarchy-wide vs 32 top-only parent-port-composed-through-flops outputs and 960 vs 64 parent-local flops.

**Validation**

- `cargo test --test pipeline recursive_hierarchy_stateful_parent_outputs_mix_parent_ports_at_depth_4_without_helpers`
- `cargo test --bin tool_matrix`
- `cargo run --bin tool_matrix -- --phase4-hierarchy-gate --out /tmp/anvil-tool-matrix-phase4-hierarchy-r66 --yosys-mode both`
  - `147` scenarios
  - `4` designs/scenario
  - `588` total designs
  - `artifact_kind = "design"`
  - `coverage_gaps = []`
  - `Verilator pass/fail = 588/0`
  - `Yosys without-abc pass/fail = 588/0`
  - `Yosys with-abc pass/fail = 588/0`
  - `saw_recursive_hierarchy_depth_4_stateful_parent_port_composed_outputs = true`
- Commit-workflow hygiene: pending final gate in this commit.

**Impact**

- The Phase 4 hierarchy gate now explicitly proves recursive non-top stateful parent-port-composed parent outputs at exact hierarchy depth 4. `r65` becomes the previous bank (depth-4 no-state parent-port-composed); `r66` is the current full downstream-clean Phase 4 hierarchy bank.

**Files touched**

- `src/bin/tool_matrix.rs`
- `tests/pipeline.rs`
- `CHANGES.md`
- `DEVELOPMENT_NOTES.md`
- `MEMORY.md`
- `CODEBASE_ANALYSIS.md`
- `USER_GUIDE.md`
- `README.md`
- `ROADMAP.md`
- `book/src/hierarchy.md`
- `book/src/architecture.md`

## 2026-05-03-phase4-recursive-depth-4-parent-port-composed-outputs — Push recursive non-top parent-port-composed parent outputs to exact hierarchy depth 4 without helpers or state

**Landed as:** 42e9678

**What changed**

- Added the focused recursive integration proof `recursive_hierarchy_parent_outputs_mix_parent_ports_at_depth_4_without_helpers` across all four `ConstructionStrategy` variants. It uses `min/max_hierarchy_depth = 4` and `2,2` child-instance bounds, isolating parent-output cones across three intermediate parent layers below the top.
- Added the new Phase 4 matrix scenario `phase4_recur_d4_parent_port_composed_output` per `ConstructionStrategy` via the new `phase4_recursive_d4_parent_port_composed_output_focus_config` helper.
- Added the `saw_recursive_hierarchy_depth_4_parent_port_composed_outputs` coverage fact and matching gap message.
- Updated the Phase 4 hierarchy run-plan and coverage tests from `141` scenarios / `564` designs to `144` scenarios / `576` designs.
- Ran the full Phase 4 hierarchy gate at `/tmp/anvil-tool-matrix-phase4-hierarchy-r65/tool_matrix_report.json`.

**Why**

- The depth-4 axis now covers parent-flops (r63) and unregistered parent-composed mixed-support child inputs (r64). r65 extends it to the parent-port-composed parent-output surface (r60's depth-3 territory), mirroring how r60 followed r59 at depth 3. Smoke at depth 4 confirmed 15 internal modules with 176 hierarchy-wide vs 44 top-only parent-port-composed outputs.

**Validation**

- `cargo test --test pipeline recursive_hierarchy_parent_outputs_mix_parent_ports_at_depth_4_without_helpers`
- `cargo test --bin tool_matrix`
- `cargo run --bin tool_matrix -- --phase4-hierarchy-gate --out /tmp/anvil-tool-matrix-phase4-hierarchy-r65 --yosys-mode both`
  - `144` scenarios
  - `4` designs/scenario
  - `576` total designs
  - `artifact_kind = "design"`
  - `coverage_gaps = []`
  - `Verilator pass/fail = 576/0`
  - `Yosys without-abc pass/fail = 576/0`
  - `Yosys with-abc pass/fail = 576/0`
  - `saw_recursive_hierarchy_depth_4_parent_port_composed_outputs = true`
- Commit-workflow hygiene: pending final gate in this commit.

**Impact**

- The Phase 4 hierarchy gate now explicitly proves recursive non-top parent-port-composed parent outputs at exact hierarchy depth 4. `r64` becomes the previous bank (depth-4 mixed-support child inputs); `r65` is the current full downstream-clean Phase 4 hierarchy bank.

**Files touched**

- `src/bin/tool_matrix.rs`
- `tests/pipeline.rs`
- `CHANGES.md`
- `DEVELOPMENT_NOTES.md`
- `MEMORY.md`
- `CODEBASE_ANALYSIS.md`
- `USER_GUIDE.md`
- `README.md`
- `ROADMAP.md`
- `book/src/hierarchy.md`
- `book/src/architecture.md`

## 2026-05-03-phase4-recursive-depth-4-mixed-support-child-inputs — Push recursive non-top mixed-support child inputs to exact hierarchy depth 4 without helpers

**Landed as:** cded654

**What changed**

- Added the focused recursive integration proof `recursive_hierarchy_parent_composed_routes_mix_parent_ports_at_depth_4_without_helpers` across all four `ConstructionStrategy` variants. It uses `min/max_hierarchy_depth = 4` and `4,4` child-instance bounds, isolating the unregistered parent-composed mixed-support surface across three intermediate parent layers below the top.
- Added the new Phase 4 matrix scenario `phase4_recur_d4_parent_composed_mixed_support_child_input` per `ConstructionStrategy` via the new `phase4_recursive_d4_parent_composed_mixed_support_focus_config` helper.
- Added the `saw_recursive_hierarchy_depth_4_mixed_support_child_inputs` coverage fact and matching gap message.
- Updated the Phase 4 hierarchy run-plan and coverage tests from `138` scenarios / `552` designs to `141` scenarios / `564` designs.
- Ran the full Phase 4 hierarchy gate at `/tmp/anvil-tool-matrix-phase4-hierarchy-r64/tool_matrix_report.json`.

**Why**

- The depth-4 axis was opened in r63 with parent flops. r64 extends the depth-4 coverage to the unregistered parent-composed mixed-support child-input surface — mirroring how r59 followed r58 at depth 3. The slice does not change the generator; it tightens the gate around an already-supported capability. Smoke at depth 4 confirmed 85 internal module occurrences with 315 hierarchy-wide vs 3 top-only mixed-support bindings and 355 vs 3 parent-composed bindings.

**Validation**

- `cargo test --test pipeline recursive_hierarchy_parent_composed_routes_mix_parent_ports_at_depth_4_without_helpers`
- `cargo test --bin tool_matrix`
- `cargo run --bin tool_matrix -- --phase4-hierarchy-gate --out /tmp/anvil-tool-matrix-phase4-hierarchy-r64 --yosys-mode both`
  - `141` scenarios
  - `4` designs/scenario
  - `564` total designs
  - `artifact_kind = "design"`
  - `coverage_gaps = []`
  - `Verilator pass/fail = 564/0`
  - `Yosys without-abc pass/fail = 564/0`
  - `Yosys with-abc pass/fail = 564/0`
  - `saw_recursive_hierarchy_depth_4_mixed_support_child_inputs = true`
- Commit-workflow hygiene: pending final gate in this commit.

**Impact**

- The Phase 4 hierarchy gate now explicitly proves recursive non-top unregistered parent-composed mixed-support child inputs at exact hierarchy depth 4. `r63` becomes the previous bank that opened the depth-4 axis; `r64` is the current full downstream-clean Phase 4 hierarchy bank.

**Files touched**

- `src/bin/tool_matrix.rs`
- `tests/pipeline.rs`
- `CHANGES.md`
- `DEVELOPMENT_NOTES.md`
- `MEMORY.md`
- `CODEBASE_ANALYSIS.md`
- `USER_GUIDE.md`
- `README.md`
- `ROADMAP.md`
- `book/src/hierarchy.md`
- `book/src/architecture.md`

## 2026-05-03-phase4-recursive-depth-4-parent-local-flops — Push recursive non-top parent-local flops to exact hierarchy depth 4

**Landed as:** bb4d738

**What changed**

- Added the focused recursive integration proof `recursive_hierarchy_parents_can_emit_local_flops_at_depth_4` across all four `ConstructionStrategy` variants. It uses `min/max_hierarchy_depth = 4` and `2,2` child-instance bounds, isolating the parent-flop surface across three intermediate parent layers below the top.
- Added the new Phase 4 matrix scenario `phase4_recur_d4_parent_state` per `ConstructionStrategy` via the new `phase4_recursive_d4_parent_state_focus_config` helper.
- Added the `saw_recursive_hierarchy_depth_4_parent_local_flops` coverage fact and matching gap message.
- Updated the Phase 4 hierarchy run-plan and coverage tests from `135` scenarios / `540` designs to `138` scenarios / `552` designs. Extended the `range_depths` assertion to include `(4, 4)`.
- Ran the full Phase 4 hierarchy gate at `/tmp/anvil-tool-matrix-phase4-hierarchy-r63/tool_matrix_report.json`.

**Why**

- The depth-3 push is structurally complete (r58 through r62 covered all four mixed-support cells at depth 3). r63 starts the depth-4 axis with the simplest surface — parent flops at depth 4 — to prove the recursive generator handles four hierarchy layers cleanly. The slice does not change the generator; it tightens the gate around an already-supported capability. Smoke confirmed 15 internal module occurrences and 960 hierarchy-wide parent-local flops vs 64 top-only at depth 4.

**Validation**

- `cargo test --test pipeline recursive_hierarchy_parents_can_emit_local_flops_at_depth_4`
- `cargo test --bin tool_matrix`
- `cargo run --bin tool_matrix -- --phase4-hierarchy-gate --out /tmp/anvil-tool-matrix-phase4-hierarchy-r63 --yosys-mode both`
  - `138` scenarios
  - `4` designs/scenario
  - `552` total designs
  - `artifact_kind = "design"`
  - `coverage_gaps = []`
  - `Verilator pass/fail = 552/0`
  - `Yosys without-abc pass/fail = 552/0`
  - `Yosys with-abc pass/fail = 552/0`
  - `saw_recursive_hierarchy_depth_4_parent_local_flops = true`
- Commit-workflow hygiene: pending final gate in this commit.

**Impact**

- The Phase 4 hierarchy gate now explicitly proves recursive non-top parent-local flops at exact hierarchy depth 4, opening the depth-4 axis on top of the completed depth-3 sweep. `r62` becomes the previous bank that closed the depth-3 push; `r63` is the current full downstream-clean Phase 4 hierarchy bank with depth-4 parent-flop evidence.

**Files touched**

- `src/bin/tool_matrix.rs`
- `tests/pipeline.rs`
- `CHANGES.md`
- `DEVELOPMENT_NOTES.md`
- `MEMORY.md`
- `CODEBASE_ANALYSIS.md`
- `USER_GUIDE.md`
- `README.md`
- `ROADMAP.md`
- `book/src/hierarchy.md`
- `book/src/architecture.md`

## 2026-05-03-phase4-recursive-depth-3-stateful-parent-composed-mixed-support-child-inputs — Push recursive non-top stateful parent-composed mixed-support child inputs to exact hierarchy depth 3 without helpers

**Landed as:** 709bff6

**What changed**

- Added the focused recursive integration proof `recursive_hierarchy_stateful_parent_composed_routes_mix_parent_ports_at_depth_3_without_helpers` across all four `ConstructionStrategy` variants. It uses `min/max_hierarchy_depth = 3`, `4,4` child-instance bounds, `hierarchy_child_input_cone_prob = 1.0`, and `hierarchy_parent_flop_prob = 1.0`, isolating unregistered parent-composed child-input cones that mix parent data ports + child outputs + parent-local Qs across two intermediate parent layers below the top.
- Added the new Phase 4 matrix scenario `phase4_recur_d3_stateful_parent_composed_mixed_support_child_input` per `ConstructionStrategy` via the new `phase4_recursive_d3_stateful_parent_composed_mixed_support_focus_config` helper.
- Added the `saw_recursive_hierarchy_depth_3_stateful_parent_composed_mixed_support_child_inputs` coverage fact and matching gap message. The fact requires `realized_max_leaf_depth >= 3`, hierarchy-wide stateful-parent-composed-mixed-support and unregistered parent-composed counters exceeding top-only, parent-local flops below the top, and `hierarchy_parent_cone_instances == 0`.
- Updated the Phase 4 hierarchy run-plan and coverage tests from `132` scenarios / `528` designs to `135` scenarios / `540` designs.
- Ran the full Phase 4 hierarchy gate at `/tmp/anvil-tool-matrix-phase4-hierarchy-r62/tool_matrix_report.json`.

**Why**

- The depth-3 push has covered parent-flops (r58), no-state mixed-support child inputs (r59), no-state parent-port-composed outputs (r60), and stateful parent-port-composed outputs (r61). r62 closes the final symmetric cell of the depth-3 sweep: the stateful child-input mixed-support surface (r56's depth-2 territory) at exact hierarchy depth 3. The slice does not change the generator — it tightens the gate around an already-supported capability.

**Validation**

- `cargo test --test pipeline recursive_hierarchy_stateful_parent_composed_routes_mix_parent_ports_at_depth_3_without_helpers`
- `cargo test --bin tool_matrix`
- `cargo run --bin tool_matrix -- --phase4-hierarchy-gate --out /tmp/anvil-tool-matrix-phase4-hierarchy-r62 --yosys-mode both`
  - `135` scenarios
  - `4` designs/scenario
  - `540` total designs
  - `artifact_kind = "design"`
  - `coverage_gaps = []`
  - `Verilator pass/fail = 540/0`
  - `Yosys without-abc pass/fail = 540/0`
  - `Yosys with-abc pass/fail = 540/0`
  - `saw_recursive_hierarchy_depth_3_stateful_parent_composed_mixed_support_child_inputs = true`
- Commit-workflow hygiene: pending final gate in this commit.

**Impact**

- The Phase 4 hierarchy gate now explicitly proves recursive non-top stateful parent-composed mixed-support child inputs at exact hierarchy depth 3, completing the depth-3 push across all four mixed-support cells. `r61` becomes the previous bank that pushed stateful parent-port-composed outputs to depth 3; `r62` is the current full downstream-clean Phase 4 hierarchy bank.

**Files touched**

- `src/bin/tool_matrix.rs`
- `tests/pipeline.rs`
- `CHANGES.md`
- `DEVELOPMENT_NOTES.md`
- `MEMORY.md`
- `CODEBASE_ANALYSIS.md`
- `USER_GUIDE.md`
- `README.md`
- `ROADMAP.md`
- `book/src/hierarchy.md`
- `book/src/architecture.md`

## 2026-05-03-phase4-recursive-depth-3-stateful-parent-port-composed-outputs — Push recursive non-top stateful parent-port-composed parent outputs to exact hierarchy depth 3 without helpers

**Landed as:** dd0940c

**What changed**

- Added the focused recursive integration proof `recursive_hierarchy_stateful_parent_outputs_mix_parent_ports_at_depth_3_without_helpers` across all four `ConstructionStrategy` variants. It uses `min/max_hierarchy_depth = 3`, `2,2` child-instance bounds, and `hierarchy_parent_flop_prob = 1.0`, isolating parent-output cones that mix parent data ports + child outputs + parent-local Qs across two intermediate parent layers below the top.
- Added the new Phase 4 matrix scenario `phase4_recur_d3_stateful_parent_port_composed_output` per `ConstructionStrategy` via the new `phase4_recursive_d3_stateful_parent_port_composed_output_focus_config` helper. The scenario forces depth 3 with `2,2` child-instance bounds and parent flops enabled, distinct from r60's depth-3 / no-state shape.
- Added the `saw_recursive_hierarchy_depth_3_stateful_parent_port_composed_outputs` coverage fact and matching gap message. The fact requires `realized_max_leaf_depth >= 3`, hierarchy-wide parent-port-composed outputs and through-flop variants exceeding top-only, parent-local flops below the top, and `hierarchy_parent_cone_instances == 0`.
- Updated the Phase 4 hierarchy run-plan and coverage tests from `129` scenarios / `516` designs to `132` scenarios / `528` designs.
- Ran the full Phase 4 hierarchy gate with Verilator plus both repo-owned Yosys modes at `/tmp/anvil-tool-matrix-phase4-hierarchy-r61/tool_matrix_report.json`.

**Why**

- The depth-3 push has now covered parent-flops (r58), unregistered parent-composed mixed-support child inputs (r59), and parent-port-composed parent outputs without state (r60). r61 closes the remaining symmetric gap by pushing the stateful version of the parent-output surface (r55's depth-2 territory) to the same exact hierarchy depth. A depth-3 regression specifically in stateful parent-output composition would now show up as a coverage gap. The slice does not change the generator — it tightens the gate around an already-supported capability.

**Validation**

- `cargo test --test pipeline recursive_hierarchy_stateful_parent_outputs_mix_parent_ports_at_depth_3_without_helpers`
- `cargo test --bin tool_matrix`
- `cargo run --bin tool_matrix -- --phase4-hierarchy-gate --out /tmp/anvil-tool-matrix-phase4-hierarchy-r61 --yosys-mode both`
  - `132` scenarios
  - `4` designs/scenario
  - `528` total designs
  - `artifact_kind = "design"`
  - `coverage_gaps = []`
  - `Verilator pass/fail = 528/0`
  - `Yosys without-abc pass/fail = 528/0`
  - `Yosys with-abc pass/fail = 528/0`
  - `saw_recursive_hierarchy_depth_3_stateful_parent_port_composed_outputs = true`
- Commit-workflow hygiene: pending final gate in this commit.

**Impact**

- The Phase 4 hierarchy gate now explicitly proves recursive non-top stateful parent-port-composed parent outputs at exact hierarchy depth 3, alongside the existing depth-3 parent-flop (r58), depth-3 mixed-support child-input (r59), and depth-3 no-state parent-port-composed output (r60) gating. `r60` becomes the previous bank that pushed parent-port-composed outputs to depth 3 without state; `r61` is the current full downstream-clean Phase 4 hierarchy bank with depth-3 stateful parent-output evidence.

**Files touched**

- `src/bin/tool_matrix.rs`
- `tests/pipeline.rs`
- `CHANGES.md`
- `DEVELOPMENT_NOTES.md`
- `MEMORY.md`
- `CODEBASE_ANALYSIS.md`
- `USER_GUIDE.md`
- `README.md`
- `ROADMAP.md`
- `book/src/hierarchy.md`
- `book/src/architecture.md`

## 2026-05-03-phase4-recursive-depth-3-parent-port-composed-outputs — Push recursive non-top parent-port-composed parent outputs to exact hierarchy depth 3 without helpers or state

**Landed as:** fa08ccd

**What changed**

- Added the focused recursive integration proof `recursive_hierarchy_parent_outputs_mix_parent_ports_at_depth_3_without_helpers` across all four `ConstructionStrategy` variants. It uses `min/max_hierarchy_depth = 3` and `2,2` child-instance bounds, isolates the parent-output cone surface (no helpers, no sibling routing, no registered routing, no parent-composed child-input cones, no parent-local flops), then asserts hierarchy-wide parent-composed and parent-port-composed output counters exceed top-only across two intermediate parent layers below the top.
- Added the new Phase 4 matrix scenario `phase4_recur_d3_parent_port_composed_output` per `ConstructionStrategy` via the new `phase4_recursive_d3_parent_port_composed_output_focus_config` helper. The scenario forces depth 3 with `2,2` child-instance bounds, distinct from r59's depth-3 / `4,4` mixed-support shape and r58's depth-3 / `2,2` parent-state shape.
- Added the `saw_recursive_hierarchy_depth_3_parent_port_composed_outputs` coverage fact and matching gap message. The fact requires `realized_max_leaf_depth >= 3`, hierarchy-wide parent-composed and parent-port-composed output counters exceeding top-only, `hierarchy_parent_cone_instances == 0`, and `hierarchy_parent_local_flops == 0`.
- Updated the Phase 4 hierarchy run-plan and coverage tests from `126` scenarios / `504` designs to `129` scenarios / `516` designs.
- Ran the full Phase 4 hierarchy gate with Verilator plus both repo-owned Yosys modes at `/tmp/anvil-tool-matrix-phase4-hierarchy-r60/tool_matrix_report.json`.

**Why**

- The depth-3 push has now covered parent-flops (r58) and unregistered parent-composed mixed-support child inputs (r59). r60 closes the remaining symmetry gap by pushing parent-port-composed parent outputs (r54's surface) to the same exact hierarchy depth. A depth-3 regression specifically in parent-output cone composition would now show up as a coverage gap. The slice does not change the generator — it tightens the gate around an already-supported capability.

**Validation**

- `cargo test --test pipeline recursive_hierarchy_parent_outputs_mix_parent_ports_at_depth_3_without_helpers`
- `cargo test --bin tool_matrix`
- `cargo run --bin tool_matrix -- --phase4-hierarchy-gate --out /tmp/anvil-tool-matrix-phase4-hierarchy-r60 --yosys-mode both`
  - `129` scenarios
  - `4` designs/scenario
  - `516` total designs
  - `artifact_kind = "design"`
  - `coverage_gaps = []`
  - `Verilator pass/fail = 516/0`
  - `Yosys without-abc pass/fail = 516/0`
  - `Yosys with-abc pass/fail = 516/0`
  - `saw_recursive_hierarchy_depth_3_parent_port_composed_outputs = true`
- Commit-workflow hygiene: pending final gate in this commit.

**Impact**

- The Phase 4 hierarchy gate now explicitly proves recursive non-top parent-port-composed parent outputs at exact hierarchy depth 3, alongside the existing depth-3 parent-flop (r58) and depth-3 mixed-support child-input (r59) gating. `r59` becomes the previous bank that pushed mixed-support child inputs to depth 3; `r60` is the current full downstream-clean Phase 4 hierarchy bank with depth-3 parent-port-composed output evidence.

**Files touched**

- `src/bin/tool_matrix.rs`
- `tests/pipeline.rs`
- `CHANGES.md`
- `DEVELOPMENT_NOTES.md`
- `MEMORY.md`
- `CODEBASE_ANALYSIS.md`
- `USER_GUIDE.md`
- `README.md`
- `ROADMAP.md`
- `book/src/hierarchy.md`
- `book/src/architecture.md`

## 2026-05-03-phase4-recursive-depth-3-mixed-support-child-inputs — Push recursive non-top mixed-support child inputs to exact hierarchy depth 3 without helpers

**Landed as:** dc4fbf3

**What changed**

- Added the focused recursive integration proof `recursive_hierarchy_parent_composed_routes_mix_parent_ports_at_depth_3_without_helpers` across all four `ConstructionStrategy` variants. It uses `min/max_hierarchy_depth = 3` and `4,4` child-instance bounds, isolates the unregistered parent-composed mixed-support surface (no helpers, no sibling routing, no registered routing, no parent-local flops), then asserts hierarchy-wide unregistered parent-composed mixed-support counters exceed top-only across two intermediate parent layers below the top.
- Added the new Phase 4 matrix scenario `phase4_recur_d3_parent_composed_mixed_support_child_input` per `ConstructionStrategy` via the new `phase4_recursive_d3_parent_composed_mixed_support_focus_config` helper. The scenario forces depth 3 with `4,4` child-instance bounds — distinct from r58's depth-3 / `2,2` parent-state shape.
- Added the `saw_recursive_hierarchy_depth_3_mixed_support_child_inputs` coverage fact and matching gap message. The fact requires `realized_max_leaf_depth >= 3`, hierarchy-wide unregistered parent-composed and mixed-support child-input bindings exceeding top-only, and `hierarchy_parent_cone_instances == 0`.
- Updated the Phase 4 hierarchy run-plan and coverage tests from `123` scenarios / `492` designs to `126` scenarios / `504` designs.
- Ran the full Phase 4 hierarchy gate with Verilator plus both repo-owned Yosys modes at `/tmp/anvil-tool-matrix-phase4-hierarchy-r59/tool_matrix_report.json`.

**Why**

- r58 pushed only the parent-flop surface to exact hierarchy depth 3. The richer mixed-support child-input surface — proved at depth 2 in r53, with state added in r56 — had no depth-3 focused proof. r59 closes that asymmetry: a depth-3 regression specifically in the parent-composed mixed-support generator path would now show up as a coverage gap. The slice does not change the generator — it tightens the gate around an already-supported capability.

**Validation**

- `cargo test --test pipeline recursive_hierarchy_parent_composed_routes_mix_parent_ports_at_depth_3_without_helpers`
- `cargo test --bin tool_matrix`
- `cargo run --bin tool_matrix -- --phase4-hierarchy-gate --out /tmp/anvil-tool-matrix-phase4-hierarchy-r59 --yosys-mode both`
  - `126` scenarios
  - `4` designs/scenario
  - `504` total designs
  - `artifact_kind = "design"`
  - `coverage_gaps = []`
  - `Verilator pass/fail = 504/0`
  - `Yosys without-abc pass/fail = 504/0`
  - `Yosys with-abc pass/fail = 504/0`
  - `saw_recursive_hierarchy_depth_3_mixed_support_child_inputs = true`
- Commit-workflow hygiene: pending final gate in this commit.

**Impact**

- The Phase 4 hierarchy gate now explicitly proves recursive non-top unregistered parent-composed mixed-support child inputs at exact hierarchy depth 3, in addition to the existing depth-2 evidence and the r58 parent-flop depth-3 evidence. `r58` becomes the previous bank that gated recursive parent-local flops at depth 3; `r59` is the current full downstream-clean Phase 4 hierarchy bank with depth-3 mixed-support evidence.

**Files touched**

- `src/bin/tool_matrix.rs`
- `tests/pipeline.rs`
- `CHANGES.md`
- `DEVELOPMENT_NOTES.md`
- `MEMORY.md`
- `CODEBASE_ANALYSIS.md`
- `USER_GUIDE.md`
- `README.md`
- `ROADMAP.md`
- `book/src/hierarchy.md`
- `book/src/architecture.md`

## 2026-05-03-phase4-recursive-depth-3-parent-local-flops — Push recursive non-top parent-local flops to exact hierarchy depth 3

**Landed as:** 3bc6a71

**What changed**

- Added the focused recursive integration proof `recursive_hierarchy_parents_can_emit_local_flops_at_depth_3` across all four `ConstructionStrategy` variants. It uses `min/max_hierarchy_depth = 3` and isolates the parent-flop surface (no helpers, no sibling routing, no registered routing, no parent-composed child-input cones), then asserts `realized_min/max_leaf_depth = 3`, hierarchy-wide parent-local flops exceed top-only, and at least one internal parent module occurrence carries local flops.
- Added the new Phase 4 matrix scenario `phase4_recur_d3_parent_state` per `ConstructionStrategy` via the new `phase4_recursive_d3_parent_state_focus_config` helper. The scenario forces depth 3 with `2,2` child-instance bounds — distinct from r57's depth-2 / `4,4` shape.
- Added the `saw_recursive_hierarchy_depth_3_parent_local_flops` coverage fact and matching gap message. The fact requires `realized_max_leaf_depth >= 3`, `hierarchy_parent_local_flops > top_local_flops`, and `internal_module_occurrences_with_local_flops > 0`.
- Updated the Phase 4 hierarchy run-plan and coverage tests from `120` scenarios / `480` designs to `123` scenarios / `492` designs. Extended the `range_depths` assertion to include `(3, 3)`.
- Ran the full Phase 4 hierarchy gate with Verilator plus both repo-owned Yosys modes at `/tmp/anvil-tool-matrix-phase4-hierarchy-r58/tool_matrix_report.json`.

**Why**

- All r51-r57 focused proofs use exact hierarchy depth 2 (one layer of internal parents below the top). The mixed-range `2:3` scenario produces some depth-3 designs as a side-effect, but no focused proof asserts that the parent-state surface fires AT depth 3 specifically. r58 closes that asymmetry: a depth-3 regression that broke the parent-flop mechanism specifically at greater nesting depth would now show up as a coverage gap. The slice does not change the generator — it tightens the gate around an already-supported capability.

**Validation**

- `cargo test --test pipeline recursive_hierarchy_parents_can_emit_local_flops_at_depth_3`
- `cargo test --bin tool_matrix`
- `cargo run --bin tool_matrix -- --phase4-hierarchy-gate --out /tmp/anvil-tool-matrix-phase4-hierarchy-r58 --yosys-mode both`
  - `123` scenarios
  - `4` designs/scenario
  - `492` total designs
  - `artifact_kind = "design"`
  - `coverage_gaps = []`
  - `Verilator pass/fail = 492/0`
  - `Yosys without-abc pass/fail = 492/0`
  - `Yosys with-abc pass/fail = 492/0`
  - `saw_recursive_hierarchy_parent_local_flops = true`
  - `saw_recursive_hierarchy_depth_3_parent_local_flops = true`
- Commit-workflow hygiene: pending final gate in this commit.

**Impact**

- The Phase 4 hierarchy gate now explicitly proves the parent-flop surface at exact hierarchy depth 3 below the top parent, in addition to the depth-2 evidence. `r57` becomes the previous bank that gated recursive non-top parent-local flops; `r58` is the current full downstream-clean Phase 4 hierarchy bank with depth-3 parent-state evidence.

**Files touched**

- `src/bin/tool_matrix.rs`
- `tests/pipeline.rs`
- `CHANGES.md`
- `DEVELOPMENT_NOTES.md`
- `MEMORY.md`
- `CODEBASE_ANALYSIS.md`
- `USER_GUIDE.md`
- `README.md`
- `ROADMAP.md`
- `book/src/hierarchy.md`
- `book/src/architecture.md`

## 2026-05-03-phase4-recursive-parent-local-flops — Gate recursive non-top parent-local flops as a first-class coverage fact

**Landed as:** 5cdca4a

**What changed**

- Added the `saw_recursive_hierarchy_parent_local_flops` coverage fact in `src/bin/tool_matrix.rs`. It fires when a design has `realized_max_leaf_depth > 1` AND `hierarchy_parent_local_flops > top_local_flops` AND `internal_module_occurrences_with_local_flops > 0`.
- Added the focused recursive integration proof `recursive_hierarchy_parents_can_emit_local_flops_below_top` across all four `ConstructionStrategy` variants. It isolates the parent-flop surface by disabling helpers, sibling routing, registered routing, and parent-composed child-input cones, then asserts hierarchy-wide parent-local flops exceed top-only and that at least one internal parent module occurrence carries local flops.
- Added the new Phase 4 matrix scenario `phase4_recur_d2_parent_state` per `ConstructionStrategy`. It uses `min_child_instances_per_module = max_child_instances_per_module = 4` (distinct from r55's 2,2) so the parent-state surface gets a dedicated focus point in the matrix rather than being only implicit in the r55 stateful parent-port-composed scenario.
- Added a coverage gap message that fires when the new saw fact is missing from a Phase 4 hierarchy run.
- Updated the Phase 4 hierarchy run-plan and coverage tests from `117` scenarios / `468` designs to `120` scenarios / `480` designs.
- Ran the full Phase 4 hierarchy gate with Verilator plus both repo-owned Yosys modes at `/tmp/anvil-tool-matrix-phase4-hierarchy-r57/tool_matrix_report.json`.

**Why**

- `saw_hierarchy_parent_local_flops` (top-only) had no recursive-non-top twin, so the gate did not enforce the parent-flop surface below the top parent. r55 and r56 implicitly evidenced non-top parent-local flops via stronger mixed-support assertions, but a regression that broke the parent-flop mechanism specifically for non-top parents could have slipped past the gate. r57 promotes the fact to first-class gated evidence with a dedicated focused proof and a focused recursive matrix scenario that exercises the parent-flop surface in isolation.

**Validation**

- `cargo test --test pipeline recursive_hierarchy_parents_can_emit_local_flops_below_top`
- `cargo test --bin tool_matrix`
- `cargo run --bin tool_matrix -- --phase4-hierarchy-gate --out /tmp/anvil-tool-matrix-phase4-hierarchy-r57 --yosys-mode both`
  - `120` scenarios
  - `4` designs/scenario
  - `480` total designs
  - `artifact_kind = "design"`
  - `coverage_gaps = []`
  - `Verilator pass/fail = 480/0`
  - `Yosys without-abc pass/fail = 480/0`
  - `Yosys with-abc pass/fail = 480/0`
  - `saw_recursive_hierarchy_parent_local_flops = true`
- Commit-workflow hygiene: pending final gate in this commit.

**Impact**

- The Phase 4 hierarchy gate now explicitly proves recursive non-top parent-local flops as a stand-alone fact rather than only as a side-channel of richer mixed-support proofs. `r56` becomes the previous recursive stateful parent-composed mixed-support child-input bank; `r57` is the current full downstream-clean Phase 4 hierarchy bank.

**Files touched**

- `src/bin/tool_matrix.rs`
- `tests/pipeline.rs`
- `CHANGES.md`
- `DEVELOPMENT_NOTES.md`
- `MEMORY.md`
- `CODEBASE_ANALYSIS.md`
- `USER_GUIDE.md`
- `README.md`
- `ROADMAP.md`
- `book/src/hierarchy.md`
- `book/src/architecture.md`

## 2026-05-03-phase4-recursive-stateful-parent-composed-mixed-support-child-inputs — Prove recursive stateful parent-composed mixed-support child inputs without helpers

**Landed as:** 8590e43

**What changed**

- Added the `child_input_bindings_from_stateful_parent_composed_mixed_support` counter (top-level twin and fractions included) — fires when an unregistered parent-composed (Gate-node) child-input binding's dep set contains parent ports, child instance outputs, and parent-local flops simultaneously.
- Added the focused recursive integration proof `recursive_hierarchy_stateful_parent_composed_routes_mix_parent_ports_below_top_without_helpers` across all four construction strategies. It asserts the new stateful-mixed-support counter exceeds top-only while helper-instance and registered-route counters remain zero.
- Extended the Phase 4 hierarchy matrix with `phase4_recur_d2_stateful_parent_composed_mixed_support_child_input` per construction strategy, sharing the no-helper parent-composed shape from r53 with `hierarchy_parent_flop_prob = 1.0` and `max_flops_per_module = 64`.
- Added the coverage fact `saw_recursive_hierarchy_stateful_parent_composed_mixed_support_child_inputs`, with a new Phase 4 coverage gap if the recursive non-top stateful no-helper parent-composed mixed-support child-input fact is absent.
- Updated the Phase 4 hierarchy run-plan and coverage tests from `114` scenarios / `456` designs to `117` scenarios / `468` designs.
- Ran the full Phase 4 hierarchy gate with Verilator plus both repo-owned Yosys modes at `/tmp/anvil-tool-matrix-phase4-hierarchy-r56/tool_matrix_report.json`.

**Why**

- r53 proved recursive non-top unregistered parent-composed mixed-support child inputs in a stateless setup. r55 added parent-local flop state on top of r54's parent-output cone proof. This slice closes the symmetric child-input + state cell of that matrix: prove the unregistered parent-composed child-input cone can simultaneously mix parent ports, child outputs, and parent-local Qs below the top parent without spending helper instances.

**Validation**

- `cargo test --test pipeline recursive_hierarchy_stateful_parent_composed_routes_mix_parent_ports_below_top_without_helpers`
- `cargo test --bin tool_matrix`
- `cargo run --bin tool_matrix -- --phase4-hierarchy-gate --out /tmp/anvil-tool-matrix-phase4-hierarchy-r56 --yosys-mode both`
  - `117` scenarios
  - `4` designs/scenario
  - `468` total designs
  - `artifact_kind = "design"`
  - `coverage_gaps = []`
  - `Verilator pass/fail = 468/0`
  - `Yosys without-abc pass/fail = 468/0`
  - `Yosys with-abc pass/fail = 468/0`
  - `saw_recursive_hierarchy_stateful_parent_port_composed_outputs = true`
  - `saw_recursive_hierarchy_stateful_parent_composed_mixed_support_child_inputs = true`
- Commit-workflow hygiene: pending final gate in this commit.

**Impact**

- The Phase 4 hierarchy gate now explicitly proves recursive non-top parent-composed unregistered child inputs that mix parent data ports, child outputs, and parent-local Qs without helper instances. `r55` becomes the previous recursive stateful parent-port-composed parent-output bank; `r56` is the current full downstream-clean Phase 4 hierarchy bank.

**Files touched**

- `src/metrics.rs`
- `src/bin/tool_matrix.rs`
- `tests/pipeline.rs`
- `CHANGES.md`
- `DEVELOPMENT_NOTES.md`
- `MEMORY.md`
- `CODEBASE_ANALYSIS.md`
- `USER_GUIDE.md`
- `README.md`
- `ROADMAP.md`
- `book/src/hierarchy.md`
- `book/src/architecture.md`

## 2026-05-02-phase4-recursive-stateful-parent-port-composed-outputs — Prove recursive stateful parent-port-composed outputs

**Landed as:** 1606b08

**What changed**

- Added hierarchy metrics for parent-port-composed parent outputs that also include parent-local Q support.
- Added the focused recursive integration proof `recursive_hierarchy_stateful_parent_outputs_mix_parent_ports_below_top_without_helpers`.
- Extended the Phase 4 hierarchy matrix with `phase4_recur_d2_stateful_parent_port_composed_output` for each construction strategy.
- Added the coverage fact `saw_recursive_hierarchy_stateful_parent_port_composed_outputs`, with a new Phase 4 coverage gap if the recursive non-top stateful no-helper parent-output fact is absent.
- Updated the Phase 4 hierarchy run-plan and coverage tests from `111` scenarios / `444` designs to `114` scenarios / `456` designs.
- Ran the full Phase 4 hierarchy gate with Verilator plus both repo-owned Yosys modes at `/tmp/anvil-tool-matrix-phase4-hierarchy-r55/tool_matrix_report.json`.

**Why**

- The previous recursive parent-port-composed parent-output proof intentionally excluded parent-local state. This slice proves the adjacent recursive non-top no-helper case where parent outputs mix parent data ports, child outputs, and parent-local Qs in the same parent-output cone.

**Validation**

- `cargo test --test pipeline recursive_hierarchy_stateful_parent_outputs_mix_parent_ports_below_top_without_helpers`
- `cargo test --bin tool_matrix`
- `cargo run --bin tool_matrix -- --phase4-hierarchy-gate --out /tmp/anvil-tool-matrix-phase4-hierarchy-r55 --yosys-mode both`
  - `114` scenarios
  - `4` designs/scenario
  - `456` total designs
  - `artifact_kind = "design"`
  - `coverage_gaps = []`
  - `Verilator pass/fail = 456/0`
  - `Yosys without-abc pass/fail = 456/0`
  - `Yosys with-abc pass/fail = 456/0`
  - `saw_hierarchy_parent_port_composed_outputs = true`
  - `saw_recursive_hierarchy_parent_port_composed_outputs = true`
  - `saw_recursive_hierarchy_stateful_parent_port_composed_outputs = true`
- Commit-workflow hygiene: pending final gate in this commit.

**Impact**

- The Phase 4 hierarchy gate now explicitly proves recursive non-top parent outputs that mix parent data ports, child outputs, and parent-local Qs without helper instances. `r54` becomes the previous recursive no-helper/no-state parent-port-composed parent-output bank; `r55` is the current full downstream-clean Phase 4 hierarchy bank.

**Files touched**

- `src/metrics.rs`
- `src/bin/tool_matrix.rs`
- `tests/pipeline.rs`
- `CHANGES.md`
- `DEVELOPMENT_NOTES.md`
- `MEMORY.md`
- `CODEBASE_ANALYSIS.md`
- `USER_GUIDE.md`
- `README.md`
- `ROADMAP.md`
- `book/src/hierarchy.md`
- `book/src/architecture.md`

## 2026-05-02-phase4-recursive-parent-port-composed-outputs — Prove recursive parent-port-composed outputs

**Landed as:** b12d732

**What changed**

- Added the focused recursive integration proof `recursive_hierarchy_parent_outputs_mix_parent_ports_below_top_without_helpers`.
- Extended the Phase 4 hierarchy matrix with `phase4_recur_d2_parent_port_composed_output` for each construction strategy.
- Added the coverage fact `saw_recursive_hierarchy_parent_port_composed_outputs`, with a new Phase 4 coverage gap if the recursive non-top parent-output fact is absent.
- Updated the Phase 4 hierarchy run-plan and coverage tests from `108` scenarios / `432` designs to `111` scenarios / `444` designs.
- Ran the full Phase 4 hierarchy gate with Verilator plus both repo-owned Yosys modes at `/tmp/anvil-tool-matrix-phase4-hierarchy-r54/tool_matrix_report.json`.

**Why**

- The existing parent-port-composed parent-output policy proved hierarchy-wide mixed parent-port / child-output output cones, but the gate did not explicitly distinguish the recursive non-top no-helper/no-state case from top-level evidence. This slice makes that exact recursive path a first-class Phase 4 coverage fact.

**Validation**

- `cargo test recursive_hierarchy_parent_outputs_mix_parent_ports_below_top_without_helpers`
- `cargo test --bin tool_matrix`
- `cargo run --bin tool_matrix -- --phase4-hierarchy-gate --out /tmp/anvil-tool-matrix-phase4-hierarchy-r54 --yosys-mode both`
  - `111` scenarios
  - `4` designs/scenario
  - `444` total designs
  - `artifact_kind = "design"`
  - `coverage_gaps = []`
  - `Verilator pass/fail = 444/0`
  - `Yosys without-abc pass/fail = 444/0`
  - `Yosys with-abc pass/fail = 444/0`
  - `saw_hierarchy_parent_port_composed_outputs = true`
  - `saw_recursive_hierarchy_parent_port_composed_outputs = true`
- Commit-workflow hygiene:
  - `cargo check --all-targets`
  - `cargo test` (`197` lib tests, `5` main tests, `26` tool-matrix tests, `79` integration tests, and `0` doctests passed)
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo fmt --all --check`
  - `mdbook build book`
  - `git --no-pager diff --check`

**Impact**

- The Phase 4 hierarchy gate now explicitly proves recursive non-top parent outputs that mix parent data ports with child outputs without helper instances or parent-local state. `r53` becomes the previous recursive parent-composed mixed-support child-input bank; `r54` is the current full downstream-clean Phase 4 hierarchy bank.

**Files touched**

- `src/bin/tool_matrix.rs`
- `tests/pipeline.rs`
- `CHANGES.md`
- `DEVELOPMENT_NOTES.md`
- `MEMORY.md`
- `CODEBASE_ANALYSIS.md`
- `USER_GUIDE.md`
- `README.md`
- `ROADMAP.md`
- `book/src/hierarchy.md`
- `book/src/architecture.md`

## 2026-05-02-phase4-recursive-parent-composed-mixed-support — Prove recursive parent-composed mixed-support routing
**Landed as:** a11d0ec

**What changed**

- Extended no-helper parent-composed child-input cones so, when both supports are available, the generated parent logic deliberately mixes parent data ports with sibling child outputs.
- Kept helper-required parent-composed routes on their existing helper-preservation path, while no-helper routes now prove mixed support without adding parent-cone helper instances or registered child-input state.
- Added the focused recursive integration proof `recursive_hierarchy_parent_composed_routes_mix_parent_ports_below_top_without_helpers`.
- Extended the Phase 4 hierarchy matrix with `phase4_recur_d2_parent_composed_mixed_support_child_input` for each construction strategy.
- Added the coverage facts `saw_hierarchy_mixed_support_child_inputs` and `saw_recursive_hierarchy_mixed_support_child_inputs`, with a new Phase 4 gap if the recursive non-top fact is absent.
- Updated the Phase 4 hierarchy run-plan and coverage tests from `105` scenarios / `420` designs to `108` scenarios / `432` designs.
- Ran the full Phase 4 hierarchy gate with Verilator plus both repo-owned Yosys modes at `/tmp/anvil-tool-matrix-phase4-hierarchy-r53/tool_matrix_report.json`.

**Why**

- Existing mixed-support child-input coverage either used registered parent-composed D cones, direct registered sibling routing, or helper-backed parent-cone paths. This slice proves the ordinary unregistered parent-composed child-input path below the top parent can mix parent-port and sibling-output support without relying on helper instances or parent-local flops.

**Validation**

- `cargo test recursive_hierarchy_parent_composed_routes_mix_parent_ports_below_top_without_helpers`
- `cargo test --bin tool_matrix`
- `cargo run --bin tool_matrix -- --phase4-hierarchy-gate --out /tmp/anvil-tool-matrix-phase4-hierarchy-r53 --yosys-mode both`
  - `108` scenarios
  - `4` designs/scenario
  - `432` total designs
  - `artifact_kind = "design"`
  - `coverage_gaps = []`
  - `Verilator pass/fail = 432/0`
  - `Yosys without-abc pass/fail = 432/0`
  - `Yosys with-abc pass/fail = 432/0`
  - `saw_hierarchy_mixed_support_child_inputs = true`
  - `saw_recursive_hierarchy_mixed_support_child_inputs = true`
- Commit-workflow hygiene:
  - `cargo check --all-targets`
  - `cargo test` (`197` lib tests, `5` main tests, `26` tool-matrix tests, `78` integration tests, and `0` doctests passed)
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo fmt --all --check`
  - `mdbook build book`
  - `git --no-pager diff --check`

**Impact**

- The Phase 4 hierarchy gate now proves no-helper unregistered parent-composed mixed-support child-input routing both at the hierarchy level and below the top parent. `r52` becomes the previous recursive direct registered sibling mixed-support bank; `r53` is the current full downstream-clean Phase 4 hierarchy bank.

**Files touched**

- `src/gen/hierarchy.rs`
- `src/bin/tool_matrix.rs`
- `tests/pipeline.rs`
- `CHANGES.md`
- `DEVELOPMENT_NOTES.md`
- `MEMORY.md`
- `CODEBASE_ANALYSIS.md`
- `USER_GUIDE.md`
- `README.md`
- `ROADMAP.md`
- `book/src/hierarchy.md`
- `book/src/architecture.md`

## 2026-05-02-phase4-recursive-registered-sibling-mixed-support — Prove recursive registered sibling mixed-support routing

**Landed as:** fc373c1

**What changed**

- Added a focused recursive integration proof for non-top direct
  registered sibling mixed-support child-input routing in exact-depth-2
  hierarchy designs.
- Extended the Phase 4 hierarchy matrix with
  `phase4_recur_d2_registered_sibling_mixed_support_state` for each
  construction strategy.
- Added the coverage fact
  `saw_recursive_hierarchy_registered_sibling_mixed_support_routing` and
  made the Phase 4 hierarchy gate fail if it is absent.
- Updated the Phase 4 hierarchy run-plan and coverage tests from
  `102` scenarios / `408` designs to `105` scenarios / `420` designs.
- Ran the full Phase 4 hierarchy gate with Verilator plus both
  repo-owned Yosys modes at
  `/tmp/anvil-tool-matrix-phase4-hierarchy-r52/tool_matrix_report.json`.

**Why**

- The direct registered sibling mixed-support policy was already proven
  at the top hierarchy parent in `r51`. This slice proves the same
  direct registered sibling route below the top parent in recursive
  hierarchy, using hierarchy-wide counters exceeding their top-only
  counterparts while registered parent-composed and helper-source
  counters remain zero.

**Validation**

- `cargo test --manifest-path /Users/richarddje/Documents/github/anvil/Cargo.toml recursive_hierarchy_registered_sibling_routes_can_mix_parent_port_support_below_top`
- `cargo test --manifest-path /Users/richarddje/Documents/github/anvil/Cargo.toml --bin tool_matrix`
- `cargo run --manifest-path /Users/richarddje/Documents/github/anvil/Cargo.toml --bin tool_matrix -- --phase4-hierarchy-gate --yosys-mode both --fail-on-coverage-gap --out /tmp/anvil-tool-matrix-phase4-hierarchy-r52`
  - `105` scenarios
  - `4` designs/scenario
  - `420` total designs
  - `artifact_kind = "design"`
  - `coverage_gaps = []`
  - `Verilator pass/fail = 420/0`
  - `Yosys without-abc pass/fail = 420/0`
  - `Yosys with-abc pass/fail = 420/0`
  - `saw_hierarchy_registered_sibling_mixed_support_routing = true`
  - `saw_recursive_hierarchy_registered_sibling_mixed_support_routing = true`
- Commit-workflow hygiene:
  - `cargo check --all-targets`
  - `cargo test`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo fmt --all --check`

**Impact**

- The Phase 4 hierarchy gate now proves direct registered sibling
  mixed-support routing both at the top parent and below the top parent
  in recursive hierarchy. `r51` becomes the previous direct registered
  sibling mixed-support bank; `r52` is the current full downstream-clean
  Phase 4 hierarchy bank.

**Files touched**

- `src/bin/tool_matrix.rs`
- `tests/pipeline.rs`
- `CHANGES.md`
- `DEVELOPMENT_NOTES.md`
- `MEMORY.md`
- `CODEBASE_ANALYSIS.md`
- `USER_GUIDE.md`
- `README.md`
- `ROADMAP.md`
- `book/src/hierarchy.md`
- `book/src/architecture.md`

## 2026-05-02-phase4-registered-sibling-mixed-support — Add direct registered sibling mixed-support routing

**Landed as:** 9e45d09

**What changed**

- Added the default-off `hierarchy_registered_sibling_mixed_support_prob`
  knob across config, CLI override parsing, knob telemetry, and the
  tool-matrix required-attempt surface.
- Extended direct registered sibling child-input routing so the flop D
  side can optionally mix an available parent data-port companion with
  the sibling/helper source before the parent-local flop.
- Kept the new route classified as direct registered sibling routing by
  wrapping the mixed D expression before registration, so it does not
  become registered parent-composed child-input evidence.
- Added metrics and fractions for direct registered sibling
  mixed-support child-input bindings:
  `child_input_bindings_from_registered_sibling_mixed_support`,
  `top_child_input_bindings_from_registered_sibling_mixed_support`,
  `registered_sibling_mixed_support_child_input_binding_fraction`, and
  `top_registered_sibling_mixed_support_child_input_binding_fraction`.
- Added focused unit and pipeline coverage, plus a Phase 4 hierarchy
  matrix scenario and coverage gate for the new policy fact.
- Ran the full Phase 4 hierarchy gate with Verilator plus both repo-owned
  Yosys modes at
  `/tmp/anvil-tool-matrix-phase4-hierarchy-r51/tool_matrix_report.json`.

**Why**

- Registered mixed-support hierarchy coverage existed for
  parent-composed D cones, but direct registered sibling routes were
  still purely sibling/helper sourced. This slice proves the stricter
  direct child-to-child route can carry parent-port support in the D
  path while staying out of the registered parent-composed bucket.

**Validation**

- `cargo fmt --all --manifest-path /Users/richarddje/Documents/github/anvil/Cargo.toml`
- `cargo test --manifest-path /Users/richarddje/Documents/github/anvil/Cargo.toml registered_sibling_mixed_support`
- `cargo test --manifest-path /Users/richarddje/Documents/github/anvil/Cargo.toml hierarchy_registered_sibling_routes_can_mix_parent_port_support`
- `cargo test --manifest-path /Users/richarddje/Documents/github/anvil/Cargo.toml --bin tool_matrix`
- `cargo run --manifest-path /Users/richarddje/Documents/github/anvil/Cargo.toml --bin tool_matrix -- --phase4-hierarchy-gate --yosys-mode both --fail-on-coverage-gap --out /tmp/anvil-tool-matrix-phase4-hierarchy-r51`
  - `102` scenarios
  - `4` designs/scenario
  - `408` total designs
  - `artifact_kind = "design"`
  - `coverage_gaps = []`
  - `Verilator pass/fail = 408/0`
  - `Yosys without-abc pass/fail = 408/0`
  - `Yosys with-abc pass/fail = 408/0`
  - `saw_hierarchy_registered_sibling_mixed_support_routing = true`
- `cargo check --all-targets --manifest-path /Users/richarddje/Documents/github/anvil/Cargo.toml`
- `cargo test --manifest-path /Users/richarddje/Documents/github/anvil/Cargo.toml`
  - `197` lib tests, `5` main tests, `26` tool-matrix tests, and `76` integration tests passed.
- `mdbook build /Users/richarddje/Documents/github/anvil/book`
- `git --no-pager diff --check`

**Impact**

- The Phase 4 hierarchy gate now proves direct registered sibling
  child-input bindings that mix sibling/helper instance-output support
  with parent data-port support without counting as registered
  parent-composed logic. `r50` becomes the previous accumulated
  mixed-support hierarchy bank; `r51` is the direct registered sibling
  mixed-support Phase 4 hierarchy bank.

**Files touched**

- `src/config.rs`
- `src/main.rs`
- `src/ir/types.rs`
- `src/gen/hierarchy.rs`
- `src/metrics.rs`
- `src/bin/tool_matrix.rs`
- `tests/pipeline.rs`
- `CHANGES.md`
- `DEVELOPMENT_NOTES.md`
- `MEMORY.md`
- `CODEBASE_ANALYSIS.md`
- `USER_GUIDE.md`
- `README.md`
- `ROADMAP.md`
- `book/src/hierarchy.md`
- `book/src/architecture.md`
- `git_message_brief.txt`

## 2026-05-01-phase4-hierarchy-r50-downstream-clean — Bank accumulated mixed-support hierarchy coverage downstream-clean

**Landed as:** this commit

**What changed**

- Ran the full Phase 4 hierarchy gate with Verilator plus both repo-owned
  Yosys modes at
  `/tmp/anvil-tool-matrix-phase4-hierarchy-r50/tool_matrix_report.json`.
- Promoted the current mixed-support coverage-only hierarchy slices into
  a full downstream-clean bank:
  - stateful helper-backed parent outputs that also carry parent-port
    support;
  - unregistered parent-composed helper child-input bindings that also
    carry parent-port support; and
  - unregistered stateful helper-through-parent-flop child-input
    bindings that also carry parent-port support.
- Refreshed continuity docs so `r50` is the latest full
  downstream-clean Phase 4 hierarchy evidence anchor, while the prior
  coverage-only probes remain focused policy breadcrumbs.

**Why**

- The previous mixed-support slices had focused tests and coverage-only
  Phase 4 report evidence, but the latest full downstream-clean bank was
  still `r49`. Before stacking more hierarchy routing work, the current
  policy surface needed to be proven through Verilator, Yosys
  without-ABC, and Yosys with-ABC.

**Validation**

- `cargo run --manifest-path /Users/richarddje/Documents/github/anvil/Cargo.toml --bin tool_matrix -- --phase4-hierarchy-gate --yosys-mode both --fail-on-coverage-gap --out /tmp/anvil-tool-matrix-phase4-hierarchy-r50`
  - `99` scenarios
  - `4` designs/scenario
  - `396` total designs
  - `artifact_kind = "design"`
  - `coverage_gaps = []`
  - `Verilator pass/fail = 396/0`
  - `Yosys without-abc pass/fail = 396/0`
  - `Yosys with-abc pass/fail = 396/0`
  - `saw_hierarchy_parent_cone_instance_flop_mixed_support_outputs = true`
  - `saw_recursive_hierarchy_parent_cone_instance_flop_mixed_support_outputs = true`
  - `saw_hierarchy_parent_cone_instance_mixed_support_routing = true`
  - `saw_recursive_hierarchy_parent_cone_instance_mixed_support_routing = true`
  - `saw_hierarchy_parent_composed_parent_cone_instance_flop_mixed_support_routing = true`
  - `saw_recursive_hierarchy_parent_composed_parent_cone_instance_flop_mixed_support_routing = true`

**Impact**

- The accumulated Phase 4 mixed-support hierarchy metrics are now banked
  in a full downstream-clean report instead of only coverage-only dry
  runs. `r49` becomes the previous recursive non-top parent-output
  helper mixed-support full bank; `r50` is the current full
  downstream-clean Phase 4 hierarchy bank.

**Files touched**

- `CHANGES.md`
- `DEVELOPMENT_NOTES.md`
- `MEMORY.md`
- `CODEBASE_ANALYSIS.md`
- `USER_GUIDE.md`
- `README.md`
- `ROADMAP.md`
- `book/src/hierarchy.md`
- `book/src/architecture.md`

## 2026-05-01-phase4-stateful-helper-child-input-mixed-support — Gate stateful helper child-input mixed-support metrics

**Landed as:** this commit

**What changed**

- Added `DesignMetrics` counters and fractions for unregistered
  parent-composed child-input bindings that consume parent-cone helper
  outputs through parent-local flops while also carrying parent-port
  support in the same final binding:
  `child_input_bindings_from_parent_cone_instance_flop_mixed_support`,
  `top_child_input_bindings_from_parent_cone_instance_flop_mixed_support`,
  `parent_cone_instance_flop_mixed_support_child_input_binding_fraction`,
  and
  `top_parent_cone_instance_flop_mixed_support_child_input_binding_fraction`.
- Extended the focused stateful parent-composed helper child-input
  metrics regression so it proves the new mixed-support counters and
  fractions are positive while registered helper and registered sibling
  route counters remain zero.
- Added Phase 4 hierarchy coverage facts for nonrecursive and recursive
  stateful parent-composed helper child-input mixed support:
  `saw_hierarchy_parent_composed_parent_cone_instance_flop_mixed_support_routing`
  and
  `saw_recursive_hierarchy_parent_composed_parent_cone_instance_flop_mixed_support_routing`.

**Why**

- The matrix already proved stateful parent-composed helper child-input
  routes through parent-local helper Qs and separately proved
  unregistered helper child-input mixed parent-port support. It did not
  prove the stricter overlap where the same unregistered final
  child-input binding both consumes a helper-sourced parent-local Q and
  includes parent-port support. This slice makes that overlap explicit
  in metrics and the Phase 4 coverage gate without adding a new
  scenario.

**Validation**

- `cargo fmt --all --manifest-path /Users/richarddje/Documents/github/anvil/Cargo.toml`
- `cargo test --manifest-path /Users/richarddje/Documents/github/anvil/Cargo.toml metrics::tests::design_metrics_capture_parent_composed_parent_cone_instance_flop_routes -- --nocapture`
- `cargo test --manifest-path /Users/richarddje/Documents/github/anvil/Cargo.toml --bin tool_matrix`
- `cargo run --manifest-path /Users/richarddje/Documents/github/anvil/Cargo.toml --bin tool_matrix -- --phase4-hierarchy-gate --skip-verilator --skip-yosys --fail-on-coverage-gap --out /tmp/anvil-tool-matrix-phase4-stateful-helper-child-input-mixed-check`
  - `99` scenarios
  - `4` designs/scenario
  - `396` total designs
  - `coverage_gaps = []`
  - `saw_hierarchy_parent_composed_parent_cone_instance_flop_mixed_support_routing = true`
  - `saw_recursive_hierarchy_parent_composed_parent_cone_instance_flop_mixed_support_routing = true`

**Impact**

- Phase 4 gate-quality metrics now distinguish ordinary stateful
  parent-composed helper child-input routes from the stricter
  helper-through-parent-flop routes that also mix parent-port support in
  the same unregistered child-input binding. The full downstream-clean
  `r50` bank now carries this fact through Verilator and both repo-owned
  Yosys modes; the earlier coverage-only dry run remains a focused
  breadcrumb.

**Files touched**

- `src/metrics.rs`
- `src/bin/tool_matrix.rs`
- `USER_GUIDE.md`
- `book/src/hierarchy.md`
- `CHANGES.md`
- `DEVELOPMENT_NOTES.md`
- `MEMORY.md`
- `CODEBASE_ANALYSIS.md`
- `git_message_brief.txt`

## 2026-05-01-phase4-unregistered-helper-child-input-mixed-support — Gate unregistered helper child-input mixed-support metrics

**Landed as:** this commit

**What changed**

- Updated `src/gen/hierarchy.rs` so parent-composed child-input cones
  that are forced to include a parent-cone helper output also try to add
  parent data-port support into the same unregistered child-input
  binding when the original cone lacks ports.
- Added `DesignMetrics` counters and fractions for unregistered
  parent-composed helper child-input bindings that mix helper output
  support with parent-port support:
  `child_input_bindings_from_parent_cone_instance_mixed_support`,
  `top_child_input_bindings_from_parent_cone_instance_mixed_support`,
  `parent_cone_instance_mixed_support_child_input_binding_fraction`, and
  `top_parent_cone_instance_mixed_support_child_input_binding_fraction`.
- Extended the existing budgeted helper metrics regression so it proves
  the new mixed-support counters are positive without relying on the
  registered helper route or helper-through-parent-flop route.
- Added Phase 4 hierarchy coverage facts for nonrecursive and recursive
  unregistered parent-composed helper child-input mixed support:
  `saw_hierarchy_parent_cone_instance_mixed_support_routing` and
  `saw_recursive_hierarchy_parent_cone_instance_mixed_support_routing`.

**Why**

- The matrix already proved unregistered parent-composed helper
  child-input routes and separately proved parent-port mixed support in
  other hierarchy routes. It did not prove that the same unregistered
  helper-backed child-input binding carried both helper-output support
  and parent-port support. This slice makes that overlap explicit in
  generation, metrics, and the Phase 4 coverage gate without adding a
  new scenario.

**Validation**

- `cargo fmt --all --manifest-path /Users/richarddje/Documents/github/anvil/Cargo.toml`
- `cargo test --manifest-path /Users/richarddje/Documents/github/anvil/Cargo.toml metrics::tests::design_metrics_capture_multiple_parent_cone_instance_budget -- --nocapture`
- `cargo test --manifest-path /Users/richarddje/Documents/github/anvil/Cargo.toml --bin tool_matrix`
- `cargo run --manifest-path /Users/richarddje/Documents/github/anvil/Cargo.toml --bin tool_matrix -- --phase4-hierarchy-gate --skip-verilator --skip-yosys --fail-on-coverage-gap --out /tmp/anvil-tool-matrix-phase4-parent-helper-child-input-mixed-check`
  - `99` scenarios
  - `4` designs/scenario
  - `396` total designs
  - `coverage_gaps = []`
  - `saw_hierarchy_parent_cone_instance_mixed_support_routing = true`
  - `saw_recursive_hierarchy_parent_cone_instance_mixed_support_routing = true`

**Impact**

- Phase 4 gate-quality metrics now distinguish ordinary unregistered
  parent-composed helper child-input bindings from helper-backed
  child-input bindings that also mix parent-port support. The full
  downstream-clean `r50` bank now carries this fact through Verilator
  and both repo-owned Yosys modes; the earlier coverage-only dry run
  remains a focused breadcrumb.

**Files touched**

- `src/gen/hierarchy.rs`
- `src/metrics.rs`
- `src/bin/tool_matrix.rs`
- `USER_GUIDE.md`
- `book/src/hierarchy.md`
- `CHANGES.md`
- `DEVELOPMENT_NOTES.md`
- `MEMORY.md`
- `CODEBASE_ANALYSIS.md`
- `git_message_brief.txt`

## 2026-05-01-phase4-stateful-parent-output-helper-mixed-support-metrics — Gate stateful parent-output helper mixed-support metrics

**Landed as:** this commit

**What changed**

- Added `DesignMetrics` counters and fractions for parent outputs that
  reach parent-cone helper instance outputs through parent-local flops
  while also carrying parent-port support in the same output cone:
  `top_outputs_reaching_parent_cone_instances_through_parent_flops_with_mixed_support`,
  `hierarchy_outputs_reaching_parent_cone_instances_through_parent_flops_with_mixed_support`,
  `top_parent_cone_instance_flop_mixed_support_output_fraction`, and
  `hierarchy_parent_cone_instance_flop_mixed_support_output_fraction`.
- Added a focused metrics regression proving the deterministic
  stateful parent-output helper route records mixed parent-port support
  through the new counters.
- Added Phase 4 hierarchy coverage facts for stateful helper-backed
  parent outputs with mixed parent-port support:
  `saw_hierarchy_parent_cone_instance_flop_mixed_support_outputs` and
  `saw_recursive_hierarchy_parent_cone_instance_flop_mixed_support_outputs`.
- Tightened the Phase 4 hierarchy coverage gate to require decision-site
  attempts for the plain `hierarchy_sibling_route_prob` knob, alongside
  the existing registered and parent-composed hierarchy knobs.

**Why**

- The previous `r49` policy proved recursive parent-output helper
  mixed support and separately proved helper-through-parent-flop output
  routes. It did not distinguish the overlap where the same parent
  output cone reaches a helper through parent-local state and also
  includes parent-port support. This slice makes that overlap explicit
  in design metrics and in the Phase 4 coverage gate.

**Validation**

- `cargo fmt --all --manifest-path /Users/richarddje/Documents/github/anvil/Cargo.toml`
- `cargo test --manifest-path /Users/richarddje/Documents/github/anvil/Cargo.toml metrics::tests::design_metrics_capture_stateful_parent_cone_instance_mixed_output_support -- --nocapture`
- `cargo test --manifest-path /Users/richarddje/Documents/github/anvil/Cargo.toml --bin tool_matrix`
- `cargo run --manifest-path /Users/richarddje/Documents/github/anvil/Cargo.toml --bin tool_matrix -- --phase4-hierarchy-gate --skip-verilator --skip-yosys --out /tmp/anvil-tool-matrix-phase4-mixed-helper-check`
  - `99` scenarios
  - `4` designs/scenario
  - `396` total designs
  - no coverage gaps reported
- `cargo check --all-targets --manifest-path /Users/richarddje/Documents/github/anvil/Cargo.toml`
- `cargo test --manifest-path /Users/richarddje/Documents/github/anvil/Cargo.toml`
  - 227 unit-target tests + 75 integration tests = 302 passing tests

**Impact**

- Phase 4 gate-quality metrics now distinguish ordinary stateful
  parent-output helper routes from stateful helper-backed parent-output
  cones that also mix parent-port support. The full downstream-clean
  `r50` bank now carries this fact through Verilator and both repo-owned
  Yosys modes; the earlier coverage-only dry run remains a focused
  breadcrumb.

**Files touched**

- `src/metrics.rs`
- `src/bin/tool_matrix.rs`
- `CHANGES.md`
- `DEVELOPMENT_NOTES.md`
- `MEMORY.md`
- `CODEBASE_ANALYSIS.md`
- `git_message_brief.txt`

## 2026-04-30-phase4-recursive-parent-output-helper-mixed-support-policy — Bank recursive parent-output helper mixed-support routing below the top parent

**Landed as:** this commit

**What changed**

- Added `DesignMetrics` counters and fractions for parent outputs that
  reach parent-cone helper instance outputs and also carry parent-port
  support in the same output cone:
  `top_outputs_reaching_parent_cone_instance_mixed_support`,
  `hierarchy_outputs_reaching_parent_cone_instance_mixed_support`,
  `top_parent_cone_instance_mixed_support_output_fraction`, and
  `hierarchy_parent_cone_instance_mixed_support_output_fraction`.
- Added
  `recursive_hierarchy_parent_outputs_mix_helper_instances_with_parent_ports_below_top`,
  a focused integration regression proving exact-depth-2 recursive
  parent-output helper cones below the top parent mix parent-port
  support with helper outputs.
- Added the Phase 4 hierarchy coverage fact
  `saw_recursive_hierarchy_parent_cone_instance_mixed_support_outputs`.
  The fact requires a recursive design, parent-output helper instances
  below the top parent, no child-input helper route, no registered
  child-input helper route, no helper-through-parent-flop output route,
  and non-top helper mixed-support parent outputs beyond the top parent.
- Banked the refreshed full downstream-clean Phase 4 hierarchy matrix at
  `/tmp/anvil-tool-matrix-phase4-hierarchy-r49/tool_matrix_report.json`
  with the same `99` scenarios / `396` designs as `r48`, empty
  coverage gaps, and the new recursive parent-output helper
  mixed-support fact.
- Refreshed live docs to present `r49` as the latest full
  downstream-clean bank, while keeping `r48` as the previous recursive
  non-top registered parent-composed helper mixed-support full bank.

**Why**

- The previous matrix proved recursive parent-output helper routing and
  separately proved mixed parent-port / child-output parent outputs. It
  did not have a first-class counter proving that the same non-top
  parent-output cone carries parent-port support and a parent-cone
  helper output. This slice tightens the parent-output composition
  policy without adding another scenario to the matrix.

**Validation**

- `cargo fmt --all`
- `cargo test recursive_hierarchy_parent_outputs_mix_helper_instances_with_parent_ports_below_top`
- `cargo test --bin tool_matrix phase4_hierarchy`
- `cargo run --bin tool_matrix -- --out /tmp/anvil-tool-matrix-phase4-hierarchy-r49-coverage --phase4-hierarchy-gate --skip-verilator --skip-yosys --fail-on-coverage-gap`
  - `99` scenarios
  - `4` designs/scenario
  - `396` total designs
  - `coverage_gaps = []`
  - `saw_recursive_hierarchy_parent_cone_instance_mixed_support_outputs = true`
- `cargo run --bin tool_matrix -- --out /tmp/anvil-tool-matrix-phase4-hierarchy-r49 --phase4-hierarchy-gate --yosys-mode both --fail-on-coverage-gap`
  - `99` scenarios
  - `4` designs/scenario
  - `396` total designs
  - `artifact_kind = "design"`
  - `coverage_gaps = []`
  - `saw_recursive_hierarchy_parent_cone_instance_mixed_support_outputs = true`
  - `saw_recursive_hierarchy_parent_cone_instance_outputs = true`
  - `saw_hierarchy_parent_port_composed_outputs = true`
  - Verilator `396/0`
  - Yosys without-ABC `396/0`
  - Yosys with-ABC `396/0`
- `cargo test design_metrics_capture_parent_cone_instance_output_support`
- `cargo check --all-targets`
- `cargo test`
  - 226 unit-target tests + 75 integration tests = 301 passing tests
- `cargo build`
- `cargo clippy --all-targets -- -D warnings`
- `cargo fmt --all --check`
- `mdbook build book`
- `git diff --check`

**Impact**

- The live Phase 4 hierarchy policy now distinguishes ordinary
  parent-output helper support from helper-supported parent outputs that
  also mix parent-port support below the top parent. The latest full
  downstream-clean bank is now `r49`; `r48` remains the previous full
  recursive non-top registered parent-composed helper mixed-support
  bank.

**Files touched**

- `src/metrics.rs`
- `tests/pipeline.rs`
- `src/bin/tool_matrix.rs`
- `README.md`
- `USER_GUIDE.md`
- `ROADMAP.md`
- `DEVELOPMENT_NOTES.md`
- `MEMORY.md`
- `CODEBASE_ANALYSIS.md`
- `book/src/architecture.md`
- `book/src/hierarchy.md`
- `CHANGES.md`

## 2026-04-30-phase4-recursive-registered-helper-mixed-support-policy — Bank recursive registered helper mixed-support routing below the top parent

**Landed as:** `140c962`

**What changed**

- Added `DesignMetrics` counters and fractions for registered
  parent-cone-helper child-input bindings that also carry parent-port
  support in the same registered parent-composed D cone:
  `child_input_bindings_from_registered_parent_cone_instance_mixed_support`,
  `top_child_input_bindings_from_registered_parent_cone_instance_mixed_support`,
  `registered_parent_cone_instance_mixed_support_child_input_binding_fraction`,
  and
  `top_registered_parent_cone_instance_mixed_support_child_input_binding_fraction`.
- Added
  `recursive_hierarchy_registered_helper_routes_mix_parent_ports_below_top`,
  a focused integration regression proving exact-depth-2 recursive
  registered parent-composed helper routes below the top parent mix
  parent-port support into the helper-sourced D cone.
- Added the Phase 4 hierarchy coverage fact
  `saw_recursive_hierarchy_registered_parent_cone_instance_mixed_support_routing`.
  The fact requires a recursive design, registered parent-composed
  routing, parent-cone helper instances below the top parent, no
  registered sibling route, no unregistered child-input cone route, and
  non-top registered helper mixed-support bindings beyond the top
  parent.
- Banked the refreshed full downstream-clean Phase 4 hierarchy matrix at
  `/tmp/anvil-tool-matrix-phase4-hierarchy-r48/tool_matrix_report.json`
  with the same `99` scenarios / `396` designs as `r47`, empty
  coverage gaps, and the new recursive registered helper mixed-support
  fact.
- Refreshed live docs to present `r48` as the latest full
  downstream-clean bank, while keeping `r47` as the previous recursive
  non-top registered multistage mixed-support no-helper full bank.

**Why**

- The previous matrix proved recursive registered parent-composed
  helper routes below the top parent and separately proved registered
  mixed-support routes. It did not have a first-class counter proving
  that the same registered helper-sourced D cone also mixes parent-port
  support. This slice tightens that helper-backed registered route
  policy without adding another scenario to the matrix.

**Validation**

- `cargo fmt --all`
- `cargo test recursive_hierarchy_registered_helper_routes_mix_parent_ports_below_top`
- `cargo test --bin tool_matrix phase4_hierarchy`
- `cargo run --bin tool_matrix -- --out /tmp/anvil-tool-matrix-phase4-hierarchy-r48-coverage --phase4-hierarchy-gate --skip-verilator --skip-yosys --fail-on-coverage-gap`
  - `99` scenarios
  - `4` designs/scenario
  - `396` total designs
  - `coverage_gaps = []`
  - `saw_recursive_hierarchy_registered_parent_cone_instance_mixed_support_routing = true`
- `cargo run --bin tool_matrix -- --out /tmp/anvil-tool-matrix-phase4-hierarchy-r48 --phase4-hierarchy-gate --yosys-mode both --fail-on-coverage-gap`
  - `99` scenarios
  - `4` designs/scenario
  - `396` total designs
  - `artifact_kind = "design"`
  - `coverage_gaps = []`
  - `saw_recursive_hierarchy_registered_parent_cone_instance_mixed_support_routing = true`
  - `saw_recursive_hierarchy_registered_parent_composed_parent_cone_instance_routing = true`
  - `saw_recursive_hierarchy_registered_multistage_mixed_support_routing = true`
  - Verilator `396/0`
  - Yosys without-ABC `396/0`
  - Yosys with-ABC `396/0`
- `cargo check --all-targets`
- `cargo test`
  - 226 unit-target tests + 74 integration tests = 300 passing tests
- `cargo build`
- `cargo clippy --all-targets -- -D warnings`
- `cargo fmt --all --check`
- `mdbook build book`
- `git diff --check`

**Impact**

- The live Phase 4 hierarchy policy now distinguishes ordinary
  registered helper-sourced child-input D cones from helper-sourced D
  cones that also mix parent-port support in the same registered
  parent-composed route below the top parent. The latest full
  downstream-clean bank is now `r48`; `r47` remains the previous full
  recursive non-top registered multistage mixed-support no-helper bank.

**Files touched**

- `src/metrics.rs`
- `tests/pipeline.rs`
- `src/bin/tool_matrix.rs`
- `README.md`
- `USER_GUIDE.md`
- `ROADMAP.md`
- `DEVELOPMENT_NOTES.md`
- `MEMORY.md`
- `CODEBASE_ANALYSIS.md`
- `book/src/architecture.md`
- `book/src/hierarchy.md`
- `CHANGES.md`

## 2026-04-30-phase4-recursive-registered-multistage-mixed-support-policy — Bank recursive registered multistage mixed-support routing below the top parent

**Landed as:** `a225c21`

**What changed**

- Added the `DesignMetrics` counters and fractions for registered
  multi-stage mixed-support child-input bindings:
  `child_input_bindings_from_registered_multistage_mixed_support`,
  `top_child_input_bindings_from_registered_multistage_mixed_support`,
  `registered_multistage_mixed_support_child_input_binding_fraction`,
  and
  `top_registered_multistage_mixed_support_child_input_binding_fraction`.
  These counters fire when a registered child-input binding is driven by
  parent-local Q, and that earlier Q's registered D cone is
  parent-composed logic that simultaneously depends on parent data
  ports, child instance outputs, and an even earlier parent-local Q.
- Added
  `recursive_hierarchy_registered_multistage_mixed_support_routes_below_top`,
  a focused integration regression proving exact-depth-2 recursive
  registered parent-composed child-input routing can combine mixed
  support and multi-stage Q reuse below the top parent without helper
  instances.
- Added the Phase 4 hierarchy coverage fact
  `saw_recursive_hierarchy_registered_multistage_mixed_support_routing`.
  The fact requires a recursive design, registered parent-composed
  routing, no registered sibling route, no unregistered child-input cone
  route, no parent-cone helper instances, non-top registered
  multi-stage mixed-support bindings beyond the top parent, and zero
  registered helper-chain counters.
- Banked the refreshed full downstream-clean Phase 4 hierarchy matrix at
  `/tmp/anvil-tool-matrix-phase4-hierarchy-r47/tool_matrix_report.json`
  with the same `99` scenarios / `396` designs as `r46`, empty
  coverage gaps, and the new recursive registered multi-stage
  mixed-support no-helper fact.
- Refreshed live docs to present `r47` as the latest full
  downstream-clean bank, while keeping `r46` as the previous recursive
  non-top registered sibling multistage no-helper full bank.

**Why**

- The previous bank separately proved recursive registered mixed-support
  routing below the top parent, recursive multi-stage registered
  parent-composed routing without helpers, and recursive multi-stage
  registered sibling routing without helpers. It did not have a
  first-class counter or policy fact for the stricter overlap: a single
  registered parent-composed D cone below the top parent that mixes
  parent ports, child outputs, and an earlier parent-local Q before a
  later child-input binding reuses that Q. This slice records that
  overlap explicitly without adding another matrix scenario.

**Validation**

- `cargo fmt --all`
- `cargo test recursive_hierarchy_registered_multistage_mixed_support_routes_below_top`
- `cargo test --bin tool_matrix phase4_hierarchy`
- `cargo run --bin tool_matrix -- --out /tmp/anvil-tool-matrix-phase4-hierarchy-r47-coverage --phase4-hierarchy-gate --skip-verilator --skip-yosys --fail-on-coverage-gap`
  - `99` scenarios
  - `4` designs/scenario
  - `396` total designs
  - `coverage_gaps = []`
  - `saw_recursive_hierarchy_registered_multistage_mixed_support_routing = true`
- `cargo run --bin tool_matrix -- --out /tmp/anvil-tool-matrix-phase4-hierarchy-r47 --phase4-hierarchy-gate --yosys-mode both --fail-on-coverage-gap`
  - `99` scenarios
  - `4` designs/scenario
  - `396` total designs
  - `artifact_kind = "design"`
  - `coverage_gaps = []`
  - `saw_recursive_hierarchy_registered_multistage_mixed_support_routing = true`
  - `saw_recursive_hierarchy_registered_multistage_routing = true`
  - `saw_recursive_hierarchy_registered_multistage_sibling_routing = true`
  - Verilator `396/0`
  - Yosys without-ABC `396/0`
  - Yosys with-ABC `396/0`
- `cargo check --all-targets`
- `cargo test`
  - 226 unit-target tests + 73 integration tests = 299 passing tests
- `cargo build`
- `cargo clippy --all-targets -- -D warnings`
- `cargo fmt --all --check`
- `mdbook build book`
- `git diff --check`

**Impact**

- The live Phase 4 hierarchy policy now distinguishes ordinary
  registered mixed-support, ordinary multi-stage registered
  parent-composed routing, and the stricter recursive non-top overlap
  where the same no-helper route has mixed support and earlier-Q reuse.
  The latest full downstream-clean bank is now `r47`; `r46` remains the
  previous full recursive non-top registered sibling multistage
  no-helper bank.

**Files touched**

- `src/metrics.rs`
- `tests/pipeline.rs`
- `src/bin/tool_matrix.rs`
- `README.md`
- `USER_GUIDE.md`
- `ROADMAP.md`
- `DEVELOPMENT_NOTES.md`
- `MEMORY.md`
- `CODEBASE_ANALYSIS.md`
- `book/src/architecture.md`
- `book/src/hierarchy.md`
- `CHANGES.md`

## 2026-04-30-phase4-recursive-registered-sibling-multistage-policy — Bank recursive registered sibling multistage routing below the top parent

**Landed as:** `dbe328a`

**What changed**

- Added
  `recursive_hierarchy_registered_sibling_routes_can_chain_without_helpers_below_top`,
  a focused integration regression proving exact-depth-2 recursive
  direct registered sibling-routed child-input routing can chain through
  earlier parent-local Qs below the top parent without using
  parent-cone helper instances or parent-composed D logic.
- Added the Phase 4 hierarchy coverage fact
  `saw_recursive_hierarchy_registered_multistage_sibling_routing`. The
  fact requires a recursive design, registered sibling routing, non-top
  first-stage registered sibling bindings beyond the top parent,
  non-top multi-stage registered sibling bindings, zero registered
  parent-composed D-cone bindings, and zero registered helper-sourced
  D-cone bindings.
- Added the `phase4_recur_d2_registered_sibling_multistage_state`
  scenario to the Phase 4 hierarchy matrix for each construction
  strategy, raising the representative policy from `96` scenarios /
  `384` designs to `99` scenarios / `396` designs at the same
  four-design gate floor.
- Banked the refreshed full downstream-clean Phase 4 hierarchy matrix at
  `/tmp/anvil-tool-matrix-phase4-hierarchy-r46/tool_matrix_report.json`
  with empty coverage gaps and the new recursive registered sibling
  multistage no-helper fact.
- Refreshed live docs to present `r46` as the latest full
  downstream-clean bank, while keeping `r45` as the previous recursive
  non-top registered parent-composed multistage no-helper full bank.

**Why**

- The previous matrix proved multi-stage registered sibling routing at
  the wrapper/top-parent level, and proved recursive helper-backed
  multi-stage registered sibling routes below the top parent. This slice
  closes the no-helper recursive sibling-routing cross product: a
  non-top recursive hierarchy parent can drive one child input from a
  child output through parent-local state, then reuse that earlier
  parent-local Q to drive a later child input directly through another
  parent-local flop.

**Validation**

- `cargo test recursive_hierarchy_registered_sibling_routes_can_chain_without_helpers_below_top`
- `cargo test --bin tool_matrix phase4_hierarchy`
- `cargo run --bin tool_matrix -- --out /tmp/anvil-tool-matrix-phase4-hierarchy-r46-coverage --phase4-hierarchy-gate --skip-verilator --skip-yosys --fail-on-coverage-gap`
  - `99` scenarios
  - `4` designs/scenario
  - `396` total designs
  - `coverage_gaps = []`
  - `saw_recursive_hierarchy_registered_multistage_sibling_routing = true`
- `cargo run --bin tool_matrix -- --out /tmp/anvil-tool-matrix-phase4-hierarchy-r46 --phase4-hierarchy-gate --yosys-mode both --fail-on-coverage-gap`
  - `99` scenarios
  - `4` designs/scenario
  - `396` total designs
  - `artifact_kind = "design"`
  - `coverage_gaps = []`
  - `saw_recursive_hierarchy_registered_multistage_sibling_routing = true`
  - `saw_recursive_hierarchy_registered_multistage_routing = true`
  - Verilator `396/0`
  - Yosys without-ABC `396/0`
  - Yosys with-ABC `396/0`
- `cargo check --all-targets`
- `cargo test`
  - 226 unit-target tests + 72 integration tests = 298 passing tests
- `cargo build`
- `cargo clippy --all-targets -- -D warnings`
- `cargo fmt --all --check`
- `mdbook build book`
- `git diff --check`

**Impact**

- The live Phase 4 hierarchy policy now requires recursive non-top
  multi-stage registered sibling-routed child-input routing without
  helper instances in addition to the existing top-level sibling
  multistage fact and the helper-backed recursive sibling multistage
  routes. The latest full downstream-clean bank is now `r46`; `r45`
  remains the previous full recursive non-top registered
  parent-composed multistage no-helper bank.

**Files touched**

- `tests/pipeline.rs`
- `src/bin/tool_matrix.rs`
- `README.md`
- `USER_GUIDE.md`
- `ROADMAP.md`
- `DEVELOPMENT_NOTES.md`
- `MEMORY.md`
- `CODEBASE_ANALYSIS.md`
- `book/src/architecture.md`
- `book/src/hierarchy.md`
- `CHANGES.md`

## 2026-04-30-phase4-recursive-registered-multistage-policy — Bank recursive registered multistage routing below the top parent

**Landed as:** `d9a5f72`

**What changed**

- Added
  `recursive_hierarchy_registered_parent_composed_routes_can_chain_without_helpers_below_top`,
  a focused integration regression proving exact-depth-2 recursive
  registered parent-composed child-input routing can chain through
  earlier parent-local Qs below the top parent without using
  parent-cone helper instances.
- Added the Phase 4 hierarchy coverage fact
  `saw_recursive_hierarchy_registered_multistage_routing`. The fact
  requires a recursive design, registered parent-composed routing,
  non-top registered parent-composed child-input bindings beyond the
  top parent, non-top multi-stage registered parent-composed bindings,
  zero registered helper-sourced D cones, zero direct registered
  sibling helper chains, and zero parent-composed helper chains.
- Added the
  `phase4_recur_d2_registered_multistage_child_input_state` scenario to
  the Phase 4 hierarchy matrix for each construction strategy, raising
  the representative policy from `93` scenarios / `372` designs to
  `96` scenarios / `384` designs at the same four-design gate floor.
- Banked the refreshed full downstream-clean Phase 4 hierarchy matrix at
  `/tmp/anvil-tool-matrix-phase4-hierarchy-r45/tool_matrix_report.json`
  with empty coverage gaps and the new recursive registered
  multistage no-helper fact.
- Refreshed live docs to present `r45` as the latest full
  downstream-clean bank, while keeping `r44` as the previous recursive
  non-top registered mixed-support full bank.

**Why**

- The previous matrix proved multi-stage registered parent-composed
  routing at the wrapper/top-parent level, and proved recursive
  helper-backed multi-stage registered routes below the top parent.
  This slice closes the no-helper recursive cross product: a non-top
  recursive hierarchy parent can drive one child input through
  parent-local state, then reuse that earlier parent-local Q in later
  registered parent-composed child-input logic without relying on
  helper instances.

**Validation**

- `cargo test recursive_hierarchy_registered_parent_composed_routes_can_chain_without_helpers_below_top`
- `cargo test --bin tool_matrix phase4_hierarchy`
- `cargo run --bin tool_matrix -- --out /tmp/anvil-tool-matrix-phase4-hierarchy-r45-coverage --phase4-hierarchy-gate --skip-verilator --skip-yosys --fail-on-coverage-gap`
  - `96` scenarios
  - `4` designs/scenario
  - `384` total designs
  - `coverage_gaps = []`
  - `saw_recursive_hierarchy_registered_multistage_routing = true`
- `cargo run --bin tool_matrix -- --out /tmp/anvil-tool-matrix-phase4-hierarchy-r45 --phase4-hierarchy-gate --yosys-mode both --fail-on-coverage-gap`
  - `96` scenarios
  - `4` designs/scenario
  - `384` total designs
  - `artifact_kind = "design"`
  - `coverage_gaps = []`
  - `saw_recursive_hierarchy_registered_multistage_routing = true`
  - `saw_recursive_hierarchy_registered_mixed_support_routing = true`
  - Verilator `384/0`
  - Yosys without-ABC `384/0`
  - Yosys with-ABC `384/0`
- `cargo check --all-targets`
- `cargo test`
  - 226 unit-target tests + 71 integration tests = 297 passing tests
- `cargo build`
- `cargo clippy --all-targets -- -D warnings`
- `cargo fmt --all --check`
- `mdbook build book`
- `git diff --check`

**Impact**

- The live Phase 4 hierarchy policy now requires recursive non-top
  multi-stage registered parent-composed child-input routing without
  helper instances in addition to the existing top-level multistage fact
  and the helper-backed recursive multistage routes. The latest full
  downstream-clean bank is now `r45`; `r44` remains the previous full
  recursive non-top registered mixed-support bank.

**Files touched**

- `tests/pipeline.rs`
- `src/bin/tool_matrix.rs`
- `README.md`
- `USER_GUIDE.md`
- `ROADMAP.md`
- `DEVELOPMENT_NOTES.md`
- `MEMORY.md`
- `CODEBASE_ANALYSIS.md`
- `book/src/architecture.md`
- `book/src/hierarchy.md`
- `CHANGES.md`

## 2026-04-30-phase4-recursive-registered-mixed-support-policy — Bank recursive registered mixed-support routing below the top parent

**Landed as:** `18b5a78`

**What changed**

- Added
  `recursive_hierarchy_registered_mixed_support_routes_below_top`, a
  focused integration regression proving exact-depth-2 recursive
  registered parent-composed child-input routing can mix parent data
  ports with child outputs below the top parent, without using
  parent-cone helper instances.
- Added the Phase 4 hierarchy coverage fact
  `saw_recursive_hierarchy_registered_mixed_support_routing`. The fact
  requires a recursive design, registered parent-composed routing,
  non-top registered parent-composed child-input bindings beyond the
  top parent, non-top registered child-output support, non-top
  registered mixed-support bindings, and zero registered
  parent-cone-helper child-input bindings in the focused lane.
- Added the
  `phase4_recur_d2_registered_mixed_child_input_state` scenario to the
  Phase 4 hierarchy matrix for each construction strategy, raising the
  representative policy from `90` scenarios / `360` designs to `93`
  scenarios / `372` designs at the same four-design gate floor.
- Banked the refreshed full downstream-clean Phase 4 hierarchy matrix at
  `/tmp/anvil-tool-matrix-phase4-hierarchy-r44/tool_matrix_report.json`
  with empty coverage gaps and the new recursive registered
  mixed-support fact.
- Refreshed live docs to present `r44` as the latest full
  downstream-clean bank, while keeping `r43` as the previous recursive
  non-top child-input multi-helper budget full bank.

**Why**

- The previous matrix proved registered mixed-support routing at the
  top parent and proved several helper-backed registered recursive
  routes below the top parent. This slice closes the no-helper
  registered mixed-support recursive cross product: a non-top recursive
  hierarchy parent can build registered D logic from both its own data
  ports and child outputs, then drive later child inputs through
  parent-local state.

**Validation**

- `cargo test recursive_hierarchy_registered_mixed_support_routes_below_top`
- `cargo test --bin tool_matrix phase4_hierarchy`
- `cargo run --bin tool_matrix -- --out /tmp/anvil-tool-matrix-phase4-hierarchy-r44-coverage --phase4-hierarchy-gate --skip-verilator --skip-yosys --fail-on-coverage-gap`
  - `93` scenarios
  - `4` designs/scenario
  - `372` total designs
  - `coverage_gaps = []`
  - `saw_recursive_hierarchy_registered_mixed_support_routing = true`
- `cargo run --bin tool_matrix -- --out /tmp/anvil-tool-matrix-phase4-hierarchy-r44 --phase4-hierarchy-gate --yosys-mode both --fail-on-coverage-gap`
  - `93` scenarios
  - `4` designs/scenario
  - `372` total designs
  - `artifact_kind = "design"`
  - `coverage_gaps = []`
  - `saw_recursive_hierarchy_registered_mixed_support_routing = true`
  - `saw_hierarchy_registered_mixed_support_routing = true`
  - Verilator `372/0`
  - Yosys without-ABC `372/0`
  - Yosys with-ABC `372/0`
- `cargo check --all-targets`
- `cargo test`
  - 226 unit-target tests + 70 integration tests = 296 passing tests
- `cargo build`
- `cargo clippy --all-targets -- -D warnings`
- `cargo fmt --all --check`
- `mdbook build book`
- `git diff --check`

**Impact**

- The live Phase 4 hierarchy policy now requires recursive non-top
  registered mixed-support child-input routing in addition to the
  existing top-level registered mixed-support fact and the
  helper-backed recursive registered routes. The latest full
  downstream-clean bank is now `r44`; `r43` remains the previous full
  recursive non-top child-input multi-helper budget bank.

**Files touched**

- `tests/pipeline.rs`
- `src/bin/tool_matrix.rs`
- `README.md`
- `USER_GUIDE.md`
- `ROADMAP.md`
- `DEVELOPMENT_NOTES.md`
- `MEMORY.md`
- `CODEBASE_ANALYSIS.md`
- `book/src/architecture.md`
- `book/src/hierarchy.md`
- `CHANGES.md`

## 2026-04-30-phase4-recursive-child-input-helper-budget-policy — Bank recursive child-input helper budget below the top parent

**Landed as:** `d79f69c`

**What changed**

- Added
  `recursive_hierarchy_parent_cone_helper_budget_allows_multiple_helpers_below_top`,
  a focused integration regression proving exact-depth-2 recursive
  parent-composed child-input helper routing can spend the configured
  `max_parent_cone_instances_per_module = 3` budget below the top
  parent without routing through parent-local helper flops or
  registered child-input helper D cones.
- Added the Phase 4 hierarchy coverage fact
  `saw_recursive_multiple_parent_cone_instances_per_parent_child_inputs`,
  which requires a recursive design, a multi-helper budget, non-top
  helper instances beyond the top parent, non-top parent-composed
  child-input bindings sourced from helper outputs, and zero
  helper-through-flop or registered helper child-input bindings in the
  focused combinational route.
- Added the `phase4_recur_d2_parent_cone_instance_budget3` scenario to
  the Phase 4 hierarchy matrix for each construction strategy, raising
  the representative policy from `87` scenarios / `348` designs to
  `90` scenarios / `360` designs at the same four-design gate floor.
- Banked the refreshed full downstream-clean Phase 4 hierarchy matrix at
  `/tmp/anvil-tool-matrix-phase4-hierarchy-r43/tool_matrix_report.json`
  with empty coverage gaps and the new recursive child-input helper
  budget fact.
- Refreshed live docs to present `r43` as the latest full
  downstream-clean bank, while keeping `r42` as the previous recursive
  non-top stateful multi-helper budget full bank.

**Why**

- The matrix already proved recursive non-top multi-helper budgets for
  parent-output composition (`r41`) and stateful parent-output
  composition (`r42`). This slice closes the child-input side of the
  same local-budget story: a non-top recursive parent can spend
  multiple helper instances while driving parent-composed child-input
  bindings directly.

**Validation**

- `cargo test recursive_hierarchy_parent_cone_helper_budget_allows_multiple_helpers_below_top`
- `cargo test --bin tool_matrix phase4_hierarchy_matrix_covers_wrapper_and_recursive_profiles`
- `cargo test --bin tool_matrix phase4_hierarchy_coverage_requires_design_facts`
- `cargo run --bin tool_matrix -- --out /tmp/anvil-tool-matrix-phase4-hierarchy-r43-coverage --phase4-hierarchy-gate --skip-verilator --skip-yosys --fail-on-coverage-gap`
  - `90` scenarios
  - `4` designs/scenario
  - `360` total designs
  - `coverage_gaps = []`
  - `saw_recursive_multiple_parent_cone_instances_per_parent_child_inputs = true`
- `cargo run --bin tool_matrix -- --out /tmp/anvil-tool-matrix-phase4-hierarchy-r43 --phase4-hierarchy-gate --yosys-mode both --fail-on-coverage-gap`
  - `90` scenarios
  - `4` designs/scenario
  - `360` total designs
  - `artifact_kind = "design"`
  - `coverage_gaps = []`
  - `saw_recursive_multiple_parent_cone_instances_per_parent_child_inputs = true`
  - `saw_recursive_multiple_parent_cone_instances_per_parent_through_flops = true`
  - `saw_recursive_multiple_parent_cone_instances_per_parent = true`
  - Verilator `360/0`
  - Yosys without-ABC `360/0`
  - Yosys with-ABC `360/0`
- `cargo check --all-targets`
- `cargo test`
  - 226 unit-target tests + 69 integration tests = 295 passing tests
- `cargo build`
- `cargo clippy --all-targets -- -D warnings`
- `cargo fmt --all --check`
- `mdbook build book`
- `git diff --check`

**Impact**

- The live Phase 4 hierarchy policy now requires recursive non-top
  child-input multi-helper budget evidence in addition to recursive
  non-top parent-output and stateful parent-output budget evidence. The
  latest full downstream-clean bank is now `r43`; `r42` remains the
  previous full recursive non-top stateful multi-helper budget bank.

**Files touched**

- `tests/pipeline.rs`
- `src/bin/tool_matrix.rs`
- `README.md`
- `USER_GUIDE.md`
- `ROADMAP.md`
- `DEVELOPMENT_NOTES.md`
- `MEMORY.md`
- `CODEBASE_ANALYSIS.md`
- `book/src/architecture.md`
- `book/src/hierarchy.md`
- `CHANGES.md`

## 2026-04-30-phase4-recursive-stateful-helper-budget-policy — Bank recursive stateful helper budget below the top parent

**Landed as:** `702ad66`

**What changed**

- Added
  `recursive_hierarchy_parent_outputs_can_spend_stateful_helper_budget_below_top`,
  a focused integration regression proving exact-depth-2 recursive
  parent-output helper composition can spend the configured
  `max_parent_cone_instances_per_module = 3` budget below the top
  parent while routing those helper outputs through parent-local flops.
- Added the Phase 4 hierarchy coverage fact
  `saw_recursive_multiple_parent_cone_instances_per_parent_through_flops`,
  which requires a recursive design, a multi-helper budget, non-top
  helper instances beyond the top parent, parent-output support from
  helper-sourced parent-local flops, and zero direct, stateful, or
  registered child-input helper bindings in the focused budget route.
- Banked the refreshed full downstream-clean Phase 4 hierarchy matrix at
  `/tmp/anvil-tool-matrix-phase4-hierarchy-r42/tool_matrix_report.json`
  with the same 87-scenario policy and the stricter recursive stateful
  budget fact.
- Refreshed live docs to present `r42` as the latest full
  downstream-clean bank, while keeping `r41` as the previous recursive
  non-top multi-helper budget full bank.

**Why**

- The matrix already proved recursive non-top parent-output helper
  routing, recursive non-top stateful parent-output helper routing, and
  recursive non-top multi-helper budget spending as separate facts.
  This slice closes their cross product: a non-top recursive parent can
  spend multiple helper instances and route the resulting helper support
  through parent-local state before driving parent outputs.

**Validation**

- `cargo check --all-targets` before edits
- `cargo test` before edits
- `cargo test --test pipeline recursive_hierarchy_parent_outputs_can_spend_stateful_helper_budget_below_top`
- `cargo test --bin tool_matrix phase4_hierarchy`
- `cargo run --bin tool_matrix -- --out /tmp/anvil-tool-matrix-phase4-hierarchy-r42 --phase4-hierarchy-gate --yosys-mode both --fail-on-coverage-gap`
  - `87` scenarios
  - `4` designs/scenario
  - `348` total designs
  - `artifact_kind = "design"`
  - `coverage_gaps = []`
  - `saw_recursive_multiple_parent_cone_instances_per_parent_through_flops = true`
  - `saw_recursive_multiple_parent_cone_instances_per_parent = true`
  - `saw_recursive_hierarchy_parent_cone_instance_flop_outputs = true`
  - Verilator `348/0`
  - Yosys without-ABC `348/0`
  - Yosys with-ABC `348/0`
- `cargo check --all-targets`
- `cargo test`
  - 226 unit-target tests + 68 integration tests = 294 passing tests
- `cargo build`
- `cargo clippy --all-targets -- -D warnings`
- `cargo fmt --all --check`
- `mdbook build book`
- `git diff --check`

**Impact**

- The live Phase 4 hierarchy policy now requires recursive non-top
  stateful multi-helper budget evidence in addition to recursive
  non-top parent-output, stateful parent-output, and direct
  multi-helper budget routing. The latest full downstream-clean bank is
  now `r42`; `r41` remains the previous full recursive non-top
  multi-helper budget bank.

**Files touched**

- `tests/pipeline.rs`
- `src/bin/tool_matrix.rs`
- `README.md`
- `USER_GUIDE.md`
- `ROADMAP.md`
- `DEVELOPMENT_NOTES.md`
- `MEMORY.md`
- `CODEBASE_ANALYSIS.md`
- `book/src/architecture.md`
- `book/src/hierarchy.md`
- `CHANGES.md`

## 2026-04-29-phase4-recursive-helper-budget-policy — Bank recursive helper budget below the top parent

**Landed as:** `52e2004`

**What changed**

- Added
  `recursive_hierarchy_parent_outputs_can_spend_helper_budget_below_top`,
  a focused integration regression proving exact-depth-2 recursive
  parent-output helper composition can spend the configured
  `max_parent_cone_instances_per_module = 3` budget below the top
  parent.
- Added the Phase 4 hierarchy coverage fact
  `saw_recursive_multiple_parent_cone_instances_per_parent`, which
  requires a recursive design, a multi-helper budget, non-top helper
  instances beyond the top parent, parent-output support from those
  helper outputs, and zero child-input helper bindings in the focused
  budget route.
- Banked the refreshed full downstream-clean Phase 4 hierarchy matrix at
  `/tmp/anvil-tool-matrix-phase4-hierarchy-r41/tool_matrix_report.json`
  with the same 87-scenario policy and the stricter recursive budget
  fact.
- Refreshed live docs to present `r41` as the latest full
  downstream-clean bank, while keeping `r40` as the previous recursive
  non-top stateful parent-output helper full bank.

**Why**

- The matrix already proved depth-1 multi-helper allocation and
  recursive non-top parent-output helper routing. This slice closes the
  local-budget cross product: a non-top recursive parent can spend more
  than one helper instance for parent-output composition, not merely
  accumulate one helper per parent across the hierarchy.

**Validation**

- `cargo test --test pipeline recursive_hierarchy_parent_outputs_can_spend_helper_budget_below_top`
- `cargo test --bin tool_matrix phase4_hierarchy`
- `cargo run --bin tool_matrix -- --out /tmp/anvil-tool-matrix-phase4-hierarchy-r41 --phase4-hierarchy-gate --yosys-mode both --fail-on-coverage-gap`
  - `87` scenarios
  - `4` designs/scenario
  - `348` total designs
  - `artifact_kind = "design"`
  - `coverage_gaps = []`
  - `saw_recursive_multiple_parent_cone_instances_per_parent = true`
  - `saw_multiple_parent_cone_instances_per_parent = true`
  - `saw_recursive_hierarchy_parent_cone_instance_flop_outputs = true`
  - `saw_recursive_hierarchy_parent_cone_instance_outputs = true`
  - Verilator `348/0`
  - Yosys without-ABC `348/0`
  - Yosys with-ABC `348/0`
- `cargo check --all-targets`
- `cargo test`
  - 226 unit-target tests + 67 integration tests = 293 passing tests
- `cargo build`
- `cargo clippy --all-targets -- -D warnings`
- `cargo fmt --all --check`
- `mdbook build book`
- `git diff --check`

**Impact**

- The live Phase 4 hierarchy policy now requires recursive non-top
  multi-helper budget evidence in addition to recursive non-top
  parent-output and stateful parent-output helper routing. The latest
  full downstream-clean bank is now `r41`; `r40` remains the previous
  full recursive non-top stateful parent-output helper bank.

**Files touched**

- `tests/pipeline.rs`
- `src/bin/tool_matrix.rs`
- `README.md`
- `USER_GUIDE.md`
- `ROADMAP.md`
- `DEVELOPMENT_NOTES.md`
- `MEMORY.md`
- `CODEBASE_ANALYSIS.md`
- `book/src/architecture.md`
- `book/src/hierarchy.md`
- `CHANGES.md`

## 2026-04-29-phase4-recursive-parent-output-helper-state-policy — Bank recursive stateful parent-output helper routes

**Landed as:** `e107c49`

**What changed**

- Added
  `recursive_hierarchy_parent_outputs_can_route_helper_instances_through_parent_flops_below_top`,
  a focused integration regression proving an exact-depth-2 recursive
  hierarchy can source non-top parent outputs from parent-cone helper
  instance outputs through parent-local flops.
- Expanded the Phase 4 hierarchy matrix policy from `84` to `87`
  scenarios by adding `phase4_recur_d2_parent_output_cone_instance_state`
  for each construction strategy.
- Added the coverage fact
  `saw_recursive_hierarchy_parent_cone_instance_flop_outputs`, which
  requires stateful parent-output helper support below the top parent,
  not only in the depth-1 wrapper lane.
- Banked the refreshed full downstream-clean Phase 4 hierarchy matrix at
  `/tmp/anvil-tool-matrix-phase4-hierarchy-r40/tool_matrix_report.json`
  with the new 87-scenario policy.
- Refreshed live docs to present `r40` as the latest full
  downstream-clean bank, while keeping `r39` as the previous recursive
  non-top parent-output helper full bank.

**Why**

- The matrix already required depth-1 stateful parent-output helper
  routing and recursive non-top direct parent-output helper routing.
  This slice closes the cross product: recursive non-top parent outputs
  can now be required to route helper outputs through parent-local state
  while staying distinct from child-input helper bindings and registered
  child-input helper D cones.

**Validation**

- `cargo test --test pipeline recursive_hierarchy_parent_outputs_can_route_helper_instances_through_parent_flops_below_top`
- `cargo test --bin tool_matrix phase4_hierarchy`
- `cargo run --bin tool_matrix -- --out /tmp/anvil-tool-matrix-phase4-hierarchy-r40 --phase4-hierarchy-gate --yosys-mode both --fail-on-coverage-gap`
  - `87` scenarios
  - `4` designs/scenario
  - `348` total designs
  - `artifact_kind = "design"`
  - `coverage_gaps = []`
  - `saw_recursive_hierarchy_parent_cone_instance_flop_outputs = true`
  - `saw_recursive_hierarchy_parent_cone_instance_outputs = true`
  - `saw_recursive_hierarchy_direct_sibling_parent_cone_instance_routing = true`
  - `saw_recursive_hierarchy_direct_registered_sibling_parent_cone_instance_routing = true`
  - `saw_recursive_hierarchy_registered_multistage_parent_cone_instance_routing = true`
  - `saw_recursive_hierarchy_registered_multistage_parent_composed_parent_cone_instance_routing = true`
  - `saw_recursive_hierarchy_registered_parent_composed_parent_cone_instance_routing = true`
  - `saw_recursive_hierarchy_parent_composed_parent_cone_instance_flop_routing = true`
  - Verilator `348/0`
  - Yosys without-ABC `348/0`
  - Yosys with-ABC `348/0`
- `cargo check --all-targets`
- `cargo test`
  - 226 unit-target tests + 66 integration tests = 292 passing tests
- `cargo build`
- `cargo clippy --all-targets -- -D warnings`
- `cargo fmt --all --check`
- `mdbook build book`
- `git diff --check`

**Impact**

- The live Phase 4 hierarchy policy now requires recursive non-top
  stateful parent-output helper routing in addition to direct recursive
  non-top parent-output helper routing and the existing recursive
  child-input helper routes. The latest full downstream-clean bank is
  now `r40`; `r39` remains the previous full recursive non-top
  parent-output helper bank.

**Files touched**

- `tests/pipeline.rs`
- `src/bin/tool_matrix.rs`
- `README.md`
- `USER_GUIDE.md`
- `ROADMAP.md`
- `DEVELOPMENT_NOTES.md`
- `MEMORY.md`
- `CODEBASE_ANALYSIS.md`
- `book/src/architecture.md`
- `book/src/hierarchy.md`
- `CHANGES.md`

## 2026-04-29-phase4-recursive-parent-output-helper-policy — Bank recursive parent-output helper routes

**Landed as:** `df9a71e`

**What changed**

- Added
  `recursive_hierarchy_parent_outputs_can_depend_on_helper_instances_below_top`,
  a focused integration regression proving an exact-depth-2 recursive
  hierarchy can source non-top parent outputs from parent-cone helper
  instance outputs without using child-input helper bindings.
- Expanded the Phase 4 hierarchy matrix policy from `81` to `84`
  scenarios by adding `phase4_recur_d2_parent_output_cone_instance` for
  each construction strategy.
- Added the coverage fact
  `saw_recursive_hierarchy_parent_cone_instance_outputs`, which requires
  parent-output helper support below the top parent rather than only in
  the depth-1 wrapper lane.
- Banked the refreshed full downstream-clean Phase 4 hierarchy matrix at
  `/tmp/anvil-tool-matrix-phase4-hierarchy-r39/tool_matrix_report.json`
  with the new 84-scenario policy.
- Refreshed live docs to present `r39` as the latest full
  downstream-clean bank, while keeping `r38` as the previous recursive
  multi-stage registered parent-composed helper full bank.

**Why**

- Parent-output helper routing was banked in the depth-1 wrapper lane,
  while recursive non-top helper routing had been focused on child-input
  paths. This slice makes recursive non-top output composition a matrix
  requirement and keeps it distinct from child-input helper routing and
  helper-through-flop output routing.

**Validation**

- `cargo test --test pipeline recursive_hierarchy_parent_outputs_can_depend_on_helper_instances_below_top`
- `cargo test --bin tool_matrix phase4_hierarchy`
- `cargo run --bin tool_matrix -- --out /tmp/anvil-tool-matrix-phase4-hierarchy-r39 --phase4-hierarchy-gate --yosys-mode both --fail-on-coverage-gap`
  - `84` scenarios
  - `4` designs/scenario
  - `336` total designs
  - `artifact_kind = "design"`
  - `coverage_gaps = []`
  - `saw_recursive_hierarchy_parent_cone_instance_outputs = true`
  - `saw_recursive_hierarchy_direct_sibling_parent_cone_instance_routing = true`
  - `saw_recursive_hierarchy_direct_registered_sibling_parent_cone_instance_routing = true`
  - `saw_recursive_hierarchy_registered_multistage_parent_cone_instance_routing = true`
  - `saw_recursive_hierarchy_registered_multistage_parent_composed_parent_cone_instance_routing = true`
  - `saw_recursive_hierarchy_registered_parent_composed_parent_cone_instance_routing = true`
  - `saw_recursive_hierarchy_parent_composed_parent_cone_instance_flop_routing = true`
  - Verilator `336/0`
  - Yosys without-ABC `336/0`
  - Yosys with-ABC `336/0`
- `cargo check --all-targets`
- `cargo test`
  - 226 unit-target tests + 65 integration tests = 291 passing tests
- `cargo build`
- `cargo clippy --all-targets -- -D warnings`
- `cargo fmt --all --check`
- `mdbook build book`
- `git diff --check`

**Impact**

- The live Phase 4 hierarchy policy now requires recursive non-top
  parent-output helper routing in addition to the recursive non-top
  child-input helper routes. The latest full downstream-clean bank is
  now `r39`; `r38` remains the previous full recursive multi-stage
  registered parent-composed helper bank.

**Files touched**

- `tests/pipeline.rs`
- `src/bin/tool_matrix.rs`
- `README.md`
- `USER_GUIDE.md`
- `ROADMAP.md`
- `DEVELOPMENT_NOTES.md`
- `MEMORY.md`
- `CODEBASE_ANALYSIS.md`
- `book/src/architecture.md`
- `book/src/hierarchy.md`
- `CHANGES.md`

## 2026-04-29-phase4-recursive-multistage-parent-composed-helper-policy — Bank recursive multi-stage parent-composed helper chains

**Landed as:** this commit

**What changed**

- Added
  `recursive_hierarchy_registered_parent_composed_routes_can_chain_helper_instances_below_top`,
  a focused integration regression proving an exact-depth-2 recursive
  hierarchy can chain non-top registered parent-composed helper routes
  through helper-sourced parent-local Qs.
- Expanded the Phase 4 hierarchy matrix policy from `78` to `81`
  scenarios by adding
  `phase4_recur_d2_registered_parent_cone_instance_multistage_state`
  for each construction strategy.
- Added the coverage fact
  `saw_recursive_hierarchy_registered_multistage_parent_composed_parent_cone_instance_routing`,
  which requires multi-stage helper-sourced registered parent-composed
  routing below the top parent rather than only in the depth-1 wrapper
  lane.
- Banked the refreshed full downstream-clean Phase 4 hierarchy matrix at
  `/tmp/anvil-tool-matrix-phase4-hierarchy-r38/tool_matrix_report.json`
  with the new 81-scenario policy.
- Refreshed live docs to present `r38` as the latest full
  downstream-clean bank, while keeping `r37` as the previous recursive
  multi-stage direct registered helper full bank.

**Why**

- The previous recursive multi-stage helper proof covered direct
  registered sibling routes below the top parent. The registered
  parent-composed version needed its own recursive non-top proof so the
  matrix can distinguish helper-sourced parent-Q chains that feed later
  parent-composed D logic from direct sibling helper chains.

**Validation**

- `cargo test --test pipeline recursive_hierarchy_registered_parent_composed_routes_can_chain_helper_instances_below_top`
- `cargo test --bin tool_matrix phase4_hierarchy`
- `cargo run --bin tool_matrix -- --out /tmp/anvil-tool-matrix-phase4-hierarchy-r38 --phase4-hierarchy-gate --yosys-mode both --fail-on-coverage-gap`
  - `81` scenarios
  - `4` designs/scenario
  - `324` total designs
  - `artifact_kind = "design"`
  - `coverage_gaps = []`
  - `saw_recursive_hierarchy_direct_sibling_parent_cone_instance_routing = true`
  - `saw_recursive_hierarchy_direct_registered_sibling_parent_cone_instance_routing = true`
  - `saw_recursive_hierarchy_registered_multistage_parent_cone_instance_routing = true`
  - `saw_recursive_hierarchy_registered_multistage_parent_composed_parent_cone_instance_routing = true`
  - `saw_recursive_hierarchy_registered_parent_composed_parent_cone_instance_routing = true`
  - `saw_recursive_hierarchy_parent_composed_parent_cone_instance_flop_routing = true`
  - Verilator `324/0`
  - Yosys without-ABC `324/0`
  - Yosys with-ABC `324/0`
- `cargo check --all-targets`
- `cargo test`
  - 226 unit-target tests + 64 integration tests = 290 passing tests
- `cargo build`
- `cargo clippy --all-targets -- -D warnings`
- `cargo fmt --all --check`
- `mdbook build book`
- `git diff --check`

**Impact**

- The live Phase 4 hierarchy policy now requires recursive non-top
  multi-stage registered parent-composed helper routing in addition to
  the recursive non-top direct sibling, direct registered sibling,
  multi-stage direct registered sibling, registered parent-composed, and
  helper-through-state routes. The latest full downstream-clean bank is
  now `r38`; `r37` remains the previous full recursive multi-stage
  direct registered helper bank.

**Files touched**

- `tests/pipeline.rs`
- `src/bin/tool_matrix.rs`
- `README.md`
- `USER_GUIDE.md`
- `ROADMAP.md`
- `DEVELOPMENT_NOTES.md`
- `MEMORY.md`
- `CODEBASE_ANALYSIS.md`
- `book/src/architecture.md`
- `book/src/hierarchy.md`
- `CHANGES.md`

## 2026-04-29-phase4-recursive-multistage-registered-helper-policy — Bank recursive multi-stage direct registered helper chains

**Landed as:** this commit

**What changed**

- Added
  `recursive_hierarchy_registered_sibling_routes_can_chain_helper_instances_below_top`,
  a focused integration regression proving an exact-depth-2 recursive
  hierarchy can chain non-top direct registered sibling helper routes
  through helper-sourced parent-local Qs.
- Expanded the Phase 4 hierarchy matrix policy from `75` to `78`
  scenarios by adding
  `phase4_recur_d2_registered_sibling_parent_cone_instance_multistage_state`
  for each construction strategy.
- Added the coverage fact
  `saw_recursive_hierarchy_registered_multistage_parent_cone_instance_routing`,
  which requires multi-stage helper-sourced registered sibling routing
  below the top parent rather than only in the depth-1 wrapper lane.
- Banked the refreshed full downstream-clean Phase 4 hierarchy matrix at
  `/tmp/anvil-tool-matrix-phase4-hierarchy-r37/tool_matrix_report.json`
  with the new 78-scenario policy.
- Refreshed live docs to present `r37` as the latest full
  downstream-clean bank, while keeping `r36` as the previous recursive
  registered parent-composed helper full bank.

**Why**

- The previous recursive registered helper proofs covered immediate
  direct registered D paths and registered parent-composed D cones below
  the top parent. The multi-stage route still needed its own recursive
  proof so the matrix could distinguish helper-sourced parent-Q chains
  from both immediate registered helper routes and parent-composed
  registered logic.

**Validation**

- `cargo test --test pipeline recursive_hierarchy_registered_sibling_routes_can_chain_helper_instances_below_top`
- `cargo test --bin tool_matrix phase4_hierarchy`
- `cargo run --bin tool_matrix -- --out /tmp/anvil-tool-matrix-phase4-hierarchy-r37 --phase4-hierarchy-gate --yosys-mode both --fail-on-coverage-gap`
  - `78` scenarios
  - `4` designs/scenario
  - `312` total designs
  - `artifact_kind = "design"`
  - `coverage_gaps = []`
  - `saw_recursive_hierarchy_direct_sibling_parent_cone_instance_routing = true`
  - `saw_recursive_hierarchy_direct_registered_sibling_parent_cone_instance_routing = true`
  - `saw_recursive_hierarchy_registered_multistage_parent_cone_instance_routing = true`
  - `saw_recursive_hierarchy_registered_parent_composed_parent_cone_instance_routing = true`
  - `saw_recursive_hierarchy_parent_composed_parent_cone_instance_flop_routing = true`
  - Verilator `312/0`
  - Yosys without-ABC `312/0`
  - Yosys with-ABC `312/0`
- `cargo check --all-targets`
- `cargo test`
  - 226 unit-target tests + 63 integration tests = 289 passing tests
- `cargo build`
- `cargo clippy --all-targets -- -D warnings`
- `cargo fmt --all --check`
- `mdbook build book`
- `git diff --check`

**Impact**

- The live Phase 4 hierarchy policy now requires recursive non-top
  multi-stage direct registered sibling helper routing in addition to
  the recursive non-top direct sibling, direct registered sibling,
  registered parent-composed, and helper-through-state routes. The
  latest full downstream-clean bank is now `r37`; `r36` remains the
  previous full recursive registered parent-composed helper bank.

**Files touched**

- `tests/pipeline.rs`
- `src/bin/tool_matrix.rs`
- `README.md`
- `USER_GUIDE.md`
- `ROADMAP.md`
- `DEVELOPMENT_NOTES.md`
- `MEMORY.md`
- `CODEBASE_ANALYSIS.md`
- `book/src/architecture.md`
- `book/src/hierarchy.md`
- `CHANGES.md`

## 2026-04-29-phase4-recursive-registered-parent-helper-policy — Bank recursive registered parent-composed helper D cones

**Landed as:** this commit

**What changed**

- Added
  `recursive_hierarchy_registered_child_input_cones_can_use_helper_instances_below_top`,
  a focused integration regression proving an exact-depth-2 recursive
  hierarchy can route non-top registered parent-composed child-input D
  cones through parent-cone helper instances.
- Expanded the Phase 4 hierarchy matrix policy from `72` to `75`
  scenarios by adding
  `phase4_recur_d2_registered_parent_cone_instance_state` for each
  construction strategy.
- Added the coverage fact
  `saw_recursive_hierarchy_registered_parent_composed_parent_cone_instance_routing`,
  which requires registered parent-composed helper routing below the top
  parent rather than only in the depth-1 wrapper lane.
- Banked the refreshed full downstream-clean Phase 4 hierarchy matrix at
  `/tmp/anvil-tool-matrix-phase4-hierarchy-r36/tool_matrix_report.json`
  with the new 75-scenario policy.
- Refreshed live docs to present `r36` as the latest full
  downstream-clean bank, while keeping `r35` as the previous recursive
  direct registered-helper full bank.

**Why**

- The previous recursive registered helper proof covered direct
  registered sibling D paths below the top parent. The registered
  parent-composed D-cone helper route still needed its own recursive
  non-top proof so the matrix could distinguish parent-composed
  registered helper placement from direct registered child-to-child
  routing.

**Validation**

- `cargo test --test pipeline recursive_hierarchy_registered_child_input_cones_can_use_helper_instances_below_top`
- `cargo test --bin tool_matrix phase4_hierarchy`
- `cargo run --bin tool_matrix -- --out /tmp/anvil-tool-matrix-phase4-recursive-registered-parent-helper-r36 --phase4-hierarchy-gate --yosys-mode both --skip-verilator --skip-yosys --fail-on-coverage-gap`
  - `75` scenarios
  - `4` designs/scenario
  - `300` total designs
  - `coverage_gaps = []`
  - `saw_recursive_hierarchy_registered_parent_composed_parent_cone_instance_routing = true`
- `cargo run --bin tool_matrix -- --out /tmp/anvil-tool-matrix-phase4-hierarchy-r36 --phase4-hierarchy-gate --yosys-mode both --fail-on-coverage-gap`
  - `75` scenarios
  - `4` designs/scenario
  - `300` total designs
  - `artifact_kind = "design"`
  - `coverage_gaps = []`
  - `saw_recursive_hierarchy_direct_sibling_parent_cone_instance_routing = true`
  - `saw_recursive_hierarchy_direct_registered_sibling_parent_cone_instance_routing = true`
  - `saw_recursive_hierarchy_registered_parent_composed_parent_cone_instance_routing = true`
  - `saw_recursive_hierarchy_parent_composed_parent_cone_instance_flop_routing = true`
  - Verilator `300/0`
  - Yosys without-ABC `300/0`
  - Yosys with-ABC `300/0`
- `cargo check --all-targets`
- `cargo test`
  - 226 unit-target tests + 62 integration tests = 288 passing tests
- `cargo build`
- `cargo clippy --all-targets -- -D warnings`
- `cargo fmt --all --check`
- `mdbook build book`
- `git diff --check`

**Impact**

- The live Phase 4 hierarchy policy now requires recursive non-top
  registered parent-composed helper D-cone routing in addition to the
  recursive non-top direct sibling, direct registered sibling, and
  helper-through-state routes. The latest full downstream-clean bank is
  now `r36`; `r35` remains the previous full recursive direct
  registered-helper bank.

**Files touched**

- [CHANGES.md](/Users/richarddje/Documents/github/anvil/CHANGES.md)
- [README.md](/Users/richarddje/Documents/github/anvil/README.md)
- [USER_GUIDE.md](/Users/richarddje/Documents/github/anvil/USER_GUIDE.md)
- [ROADMAP.md](/Users/richarddje/Documents/github/anvil/ROADMAP.md)
- [CODEBASE_ANALYSIS.md](/Users/richarddje/Documents/github/anvil/CODEBASE_ANALYSIS.md)
- [DEVELOPMENT_NOTES.md](/Users/richarddje/Documents/github/anvil/DEVELOPMENT_NOTES.md)
- [MEMORY.md](/Users/richarddje/Documents/github/anvil/MEMORY.md)
- [book/src/architecture.md](/Users/richarddje/Documents/github/anvil/book/src/architecture.md)
- [book/src/hierarchy.md](/Users/richarddje/Documents/github/anvil/book/src/hierarchy.md)
- [src/bin/tool_matrix.rs](/Users/richarddje/Documents/github/anvil/src/bin/tool_matrix.rs)
- [tests/pipeline.rs](/Users/richarddje/Documents/github/anvil/tests/pipeline.rs)

## 2026-04-29-phase4-recursive-direct-registered-helper-policy — Bank recursive registered direct-helper routing below the top parent

**Landed as:** this commit

**What changed**

- Added
  `recursive_hierarchy_registered_sibling_routes_can_use_helper_instances_below_top`,
  a focused integration regression proving an exact-depth-2 recursive
  hierarchy can route non-top direct registered sibling child-input D
  paths through parent-cone helper instances.
- Expanded the Phase 4 hierarchy matrix policy from `69` to `72`
  scenarios by adding
  `phase4_recur_d2_direct_registered_sibling_parent_cone_instance_state`
  for each construction strategy.
- Added the coverage fact
  `saw_recursive_hierarchy_direct_registered_sibling_parent_cone_instance_routing`,
  which requires registered sibling helper routing below the top parent
  while keeping registered parent-composed D-cone counters out of the
  proof.
- Banked the refreshed full downstream-clean Phase 4 hierarchy matrix at
  `/tmp/anvil-tool-matrix-phase4-hierarchy-r35/tool_matrix_report.json`
  with the new 72-scenario policy.
- Refreshed live docs to present `r35` as the latest full
  downstream-clean bank, while keeping `r34` as the previous recursive
  direct-helper full bank.

**Why**

- The previous recursive helper proof covered direct sibling routes below
  top, but the direct registered sibling helper route was still proven
  only in the depth-1 wrapper lane. The next honest Phase 4 step was to
  prove that the helper-sourced registered D path also occurs below the
  top parent in recursive exact-depth-2 hierarchies.

**Validation**

- `cargo test --test pipeline recursive_hierarchy_registered_sibling_routes_can_use_helper_instances_below_top`
- `cargo test --bin tool_matrix phase4_hierarchy`
- `cargo run --bin tool_matrix -- --out /tmp/anvil-tool-matrix-phase4-recursive-direct-registered-helper-r35 --phase4-hierarchy-gate --yosys-mode both --skip-verilator --skip-yosys --fail-on-coverage-gap`
  - `72` scenarios
  - `4` designs/scenario
  - `288` total designs
  - `coverage_gaps = []`
  - `saw_recursive_hierarchy_direct_registered_sibling_parent_cone_instance_routing = true`
- `cargo run --bin tool_matrix -- --out /tmp/anvil-tool-matrix-phase4-hierarchy-r35 --phase4-hierarchy-gate --yosys-mode both --fail-on-coverage-gap`
  - `72` scenarios
  - `4` designs/scenario
  - `288` total designs
  - `artifact_kind = "design"`
  - `coverage_gaps = []`
  - `saw_recursive_hierarchy_direct_sibling_parent_cone_instance_routing = true`
  - `saw_recursive_hierarchy_direct_registered_sibling_parent_cone_instance_routing = true`
  - `saw_recursive_hierarchy_parent_composed_parent_cone_instance_flop_routing = true`
  - Verilator `288/0`
  - Yosys without-ABC `288/0`
  - Yosys with-ABC `288/0`
- `cargo check --all-targets`
- `cargo test`
  - 226 unit-target tests + 61 integration tests = 287 passing tests
- `cargo build`
- `cargo clippy --all-targets -- -D warnings`
- `cargo fmt --all --check`
- `mdbook build book`
- `git diff --check`

**Impact**

- The live Phase 4 hierarchy policy now requires recursive non-top direct
  registered sibling helper routing in addition to recursive non-top
  direct sibling helper routing and recursive non-top helper-through-state
  routing. The latest full downstream-clean bank is now `r35`; `r34`
  remains the previous full recursive direct-helper bank.

**Files touched**

- [CHANGES.md](/Users/richarddje/Documents/github/anvil/CHANGES.md)
- [README.md](/Users/richarddje/Documents/github/anvil/README.md)
- [USER_GUIDE.md](/Users/richarddje/Documents/github/anvil/USER_GUIDE.md)
- [ROADMAP.md](/Users/richarddje/Documents/github/anvil/ROADMAP.md)
- [CODEBASE_ANALYSIS.md](/Users/richarddje/Documents/github/anvil/CODEBASE_ANALYSIS.md)
- [DEVELOPMENT_NOTES.md](/Users/richarddje/Documents/github/anvil/DEVELOPMENT_NOTES.md)
- [MEMORY.md](/Users/richarddje/Documents/github/anvil/MEMORY.md)
- [book/src/architecture.md](/Users/richarddje/Documents/github/anvil/book/src/architecture.md)
- [book/src/hierarchy.md](/Users/richarddje/Documents/github/anvil/book/src/hierarchy.md)
- [src/bin/tool_matrix.rs](/Users/richarddje/Documents/github/anvil/src/bin/tool_matrix.rs)
- [tests/pipeline.rs](/Users/richarddje/Documents/github/anvil/tests/pipeline.rs)

## 2026-04-29-phase4-recursive-direct-helper-policy — Bank recursive direct-helper routing below the top parent

**Landed as:** this commit

**What changed**

- Added
  `recursive_hierarchy_sibling_routes_can_use_helper_instances_below_top`,
  a focused integration regression proving an exact-depth-2 recursive
  hierarchy can route non-top direct sibling child inputs through
  parent-cone helper instances.
- Expanded the Phase 4 hierarchy matrix policy from `66` to `69`
  scenarios by adding
  `phase4_recur_d2_direct_sibling_parent_cone_instance` for each
  construction strategy.
- Added the coverage fact
  `saw_recursive_hierarchy_direct_sibling_parent_cone_instance_routing`,
  which requires hierarchy-wide direct sibling helper routing below the
  top parent while keeping the registered helper-route counters at zero.
- The first full `r32` attempt exposed a Yosys warning-only failure on
  an exact-selector `CaseMux` feeding a provably constant overshift.
  `src/gen/cone.rs` now lets `exact_gate_value` and
  `node_unsigned_bounds` follow exact `CaseMux` / `CasezMux` selector
  arms, with conservative unions for non-exact selectors.
- The full hygiene rerun also exposed a post-remap `And` duplicate in
  the default strict operand-uniqueness lane. `flatten_posthoc_associative_gates`
  now normalizes idempotent `And` / `Or` duplicates and `Xor` pair
  cancellations even when no same-op child splice is involved.
- Banked the refreshed full downstream-clean Phase 4 hierarchy matrix at
  `/tmp/anvil-tool-matrix-phase4-hierarchy-r34/tool_matrix_report.json`
  with the new 69-scenario policy.
- Refreshed live docs to present `r34` as the latest full
  downstream-clean bank, while keeping `r31` as the previous recursive
  helper-state full bank, `r32` as useful root-cause evidence, and
  `r33` as the pre-compact-normalization direct-helper bank.

**Why**

- The previous direct sibling helper proof was depth-1. The next honest
  Phase 4 step was to prove that the same helper route can occur inside
  a recursive hierarchy below the top parent, without accidentally
  counting registered helper routes.
- The warning from the first full run was real cleanup feedback: exact
  case selectors need to be visible to the generator-side bounds logic
  so warning-prone dynamic shifts can be folded before downstream tools
  complain.
- Remap-driven cleanup has to preserve the same duplicate-operand
  doctrine as intern-time construction; otherwise later semantic folds
  can reintroduce illegal-looking associative operands after the
  generator already did the right thing up front.

**Validation**

- `cargo test --test pipeline recursive_hierarchy_sibling_routes_can_use_helper_instances_below_top`
- `cargo test --bin tool_matrix phase4_hierarchy`
- `cargo test case`
- `cargo test flatten_posthoc_associative_gates_dedups_idempotent_duplicates`
- `cargo test --test pipeline zero_duplicate_operands_at_default_knobs`
- `cargo run --bin tool_matrix -- --out /tmp/anvil-tool-matrix-phase4-recursive-direct-helper-r32 --phase4-hierarchy-gate --skip-verilator --skip-yosys --fail-on-coverage-gap`
  - `69` scenarios
  - `4` designs/scenario
  - `276` total designs
  - `coverage_gaps = []`
  - `saw_recursive_hierarchy_direct_sibling_parent_cone_instance_routing = true`
  - `saw_recursive_hierarchy_parent_composed_parent_cone_instance_flop_routing = true`
- `cargo run --bin tool_matrix -- --out /tmp/anvil-tool-matrix-phase4-hierarchy-r34 --phase4-hierarchy-gate --yosys-mode both --fail-on-coverage-gap`
  - `69` scenarios
  - `4` designs/scenario
  - `276` total designs
  - `artifact_kind = "design"`
  - `coverage_gaps = []`
  - `saw_recursive_hierarchy_direct_sibling_parent_cone_instance_routing = true`
  - `saw_recursive_hierarchy_parent_composed_parent_cone_instance_flop_routing = true`
  - Verilator `276/0`
  - Yosys without-ABC `276/0`
  - Yosys with-ABC `276/0`
- `cargo check --all-targets`
- `cargo test`
  - 226 unit-target tests + 60 integration tests = 286 passing tests
- `cargo clippy --all-targets -- -D warnings`
- `cargo fmt --all --check`
- `mdbook build book`
- `git diff --check`

**Impact**

- The live Phase 4 hierarchy policy now requires recursive non-top
  direct sibling helper routing and recursive non-top helper-through-state
  routing. The latest full downstream-clean bank is now `r34`; `r31`
  remains the previous full recursive helper-state bank.
- Case/casez exact-selector bounds now participate in shift cleanup, so
  exact procedural case shapes are less likely to leak deterministic
  overshift warnings into downstream tools.

**Files touched**

- [CHANGES.md](/Users/richarddje/Documents/github/anvil/CHANGES.md)
- [README.md](/Users/richarddje/Documents/github/anvil/README.md)
- [USER_GUIDE.md](/Users/richarddje/Documents/github/anvil/USER_GUIDE.md)
- [ROADMAP.md](/Users/richarddje/Documents/github/anvil/ROADMAP.md)
- [CODEBASE_ANALYSIS.md](/Users/richarddje/Documents/github/anvil/CODEBASE_ANALYSIS.md)
- [DEVELOPMENT_NOTES.md](/Users/richarddje/Documents/github/anvil/DEVELOPMENT_NOTES.md)
- [MEMORY.md](/Users/richarddje/Documents/github/anvil/MEMORY.md)
- [book/src/architecture.md](/Users/richarddje/Documents/github/anvil/book/src/architecture.md)
- [book/src/hierarchy.md](/Users/richarddje/Documents/github/anvil/book/src/hierarchy.md)
- [src/bin/tool_matrix.rs](/Users/richarddje/Documents/github/anvil/src/bin/tool_matrix.rs)
- [src/gen/cone.rs](/Users/richarddje/Documents/github/anvil/src/gen/cone.rs)
- [src/ir/compact.rs](/Users/richarddje/Documents/github/anvil/src/ir/compact.rs)
- [tests/pipeline.rs](/Users/richarddje/Documents/github/anvil/tests/pipeline.rs)

## 2026-04-29-phase4-recursive-helper-state-policy — Prove and bank helper-state routing below the top parent

**Landed as:** this commit

**What changed**

- Added
  `recursive_hierarchy_parent_composed_helper_routes_can_use_parent_flops_below_top`,
  a focused integration regression proving an exact-depth-2 recursive
  hierarchy can route non-top parent-composed child inputs from
  parent-cone helper instances through parent-local flops.
- Expanded the Phase 4 hierarchy matrix policy from `63` to `66`
  scenarios by adding `phase4_recur_d2_parent_cone_instance_state` for
  each construction strategy.
- Added the coverage fact
  `saw_recursive_hierarchy_parent_composed_parent_cone_instance_flop_routing`,
  which requires the helper instance, parent-local flop, and
  helper-through-flop child-input binding counts to exceed their
  top-only counterparts.
- Banked the refreshed full downstream-clean Phase 4 hierarchy matrix at
  `/tmp/anvil-tool-matrix-phase4-hierarchy-r31/tool_matrix_report.json`
  with the new 66-scenario policy.
- Refreshed live docs to present `r31` as the latest full
  downstream-clean bank and `r30` as the previous full bank for the
  63-scenario policy.

**Why**

- The previous stateful parent-composed helper proof was depth-1. The
  next honest Phase 4 step was to prove that the same route can happen
  inside a recursive hierarchy below the top parent, not merely in the
  top wrapper lane.

**Validation**

- `cargo test --test pipeline recursive_hierarchy_parent_composed_helper_routes_can_use_parent_flops_below_top`
- `cargo test --bin tool_matrix phase4_hierarchy`
- `cargo run --bin tool_matrix -- --out /tmp/anvil-tool-matrix-phase4-recursive-helper-state-r31 --phase4-hierarchy-gate --skip-verilator --skip-yosys --fail-on-coverage-gap`
  - `66` scenarios
  - `4` designs/scenario
  - `264` total designs
  - `coverage_gaps = []`
  - `saw_recursive_hierarchy_parent_composed_parent_cone_instance_flop_routing = true`
- `cargo run --bin tool_matrix -- --out /tmp/anvil-tool-matrix-phase4-hierarchy-r31 --phase4-hierarchy-gate --yosys-mode both --fail-on-coverage-gap`
  - `66` scenarios
  - `4` designs/scenario
  - `264` total designs
  - `artifact_kind = "design"`
  - `coverage_gaps = []`
  - `saw_recursive_hierarchy_parent_composed_parent_cone_instance_flop_routing = true`
  - Verilator `264/0`
  - Yosys without-ABC `264/0`
  - Yosys with-ABC `264/0`
- `cargo check --all-targets`
- `cargo test`
  - 223 unit-target tests + 59 integration tests = 282 passing tests
- `cargo clippy --all-targets -- -D warnings`
- `cargo fmt --all --check`
- `mdbook build book`
- `git diff --check`

**Impact**

- The live Phase 4 hierarchy policy now requires recursive non-top
  helper-through-parent-flop child-input routing. The latest full
  downstream-clean bank is now `r31`; `r30` remains the previous full
  bank for the 63-scenario policy.

**Files touched**

- [CHANGES.md](/Users/richarddje/Documents/github/anvil/CHANGES.md)
- [README.md](/Users/richarddje/Documents/github/anvil/README.md)
- [USER_GUIDE.md](/Users/richarddje/Documents/github/anvil/USER_GUIDE.md)
- [ROADMAP.md](/Users/richarddje/Documents/github/anvil/ROADMAP.md)
- [CODEBASE_ANALYSIS.md](/Users/richarddje/Documents/github/anvil/CODEBASE_ANALYSIS.md)
- [DEVELOPMENT_NOTES.md](/Users/richarddje/Documents/github/anvil/DEVELOPMENT_NOTES.md)
- [MEMORY.md](/Users/richarddje/Documents/github/anvil/MEMORY.md)
- [book/src/architecture.md](/Users/richarddje/Documents/github/anvil/book/src/architecture.md)
- [book/src/hierarchy.md](/Users/richarddje/Documents/github/anvil/book/src/hierarchy.md)
- [src/bin/tool_matrix.rs](/Users/richarddje/Documents/github/anvil/src/bin/tool_matrix.rs)
- [tests/pipeline.rs](/Users/richarddje/Documents/github/anvil/tests/pipeline.rs)

## 2026-04-29-phase4-parent-composed-helper-state — Route helper child inputs through parent state

**Landed as:** `b0b9fc8`

**What changed**

- Extended parent-composed child-input helper routing so a required
  parent-cone helper output can first be registered into parent-local
  state and then consumed by unregistered parent-composed child-input
  logic.
- Added dedicated design metrics for this shape:
  `child_input_bindings_from_parent_cone_instances_through_parent_flops`,
  `top_child_input_bindings_from_parent_cone_instances_through_parent_flops`,
  `parent_cone_instance_flop_child_input_binding_fraction`, and
  `top_parent_cone_instance_flop_child_input_binding_fraction`.
- Tightened the existing registered child-input helper classifiers so
  they only count genuinely registered child-input bindings whose final
  binding is a parent-local flop Q. The new stateful parent-composed
  helper route stays separate from
  `child_input_bindings_from_registered_parent_cone_instances`.
- Added focused regressions:
  `design_metrics_capture_parent_composed_parent_cone_instance_flop_routes`
  and
  `hierarchy_parent_composed_helper_routes_can_use_parent_flops`.
- Expanded the Phase 4 hierarchy matrix from 60 to 63 scenarios by
  adding `phase4_hier2_inst4_parent_cone_instance_state` for each
  construction strategy. The matrix coverage contract now requires
  `saw_hierarchy_parent_composed_parent_cone_instance_flop_routing = true`.
- Banked the refreshed full downstream-clean Phase 4 hierarchy matrix
  at `/tmp/anvil-tool-matrix-phase4-hierarchy-r30/tool_matrix_report.json`:
  `63` scenarios, `4` designs/scenario, `252` total designs,
  `artifact_kind = "design"`, `coverage_gaps = []`, Verilator
  `252/0`, Yosys without-ABC `252/0`, and Yosys with-ABC `252/0`.
- README, USER_GUIDE, ROADMAP, CODEBASE_ANALYSIS, DEVELOPMENT_NOTES,
  MEMORY, and the mdBook hierarchy / architecture / knobs / recipes /
  IR chapters now document the new route, knobs, metrics, examples, and
  `r30` evidence anchor.

**Why**

- The previous helper-through-state proof covered registered
  parent-composed D logic. This closes the sibling case where the final
  child binding remains unregistered parent logic, but one operand comes
  from a helper-sourced parent-local Q.
- Users can now distinguish three nearby helper-state shapes in
  manifests: stateful parent outputs, registered helper child-input D
  cones, and unregistered parent-composed helper child inputs that read
  helper state.

**Validation**

- `cargo test design_metrics_capture_parent_composed_parent_cone_instance_flop_routes`
- `cargo test hierarchy_parent_composed_helper_routes_can_use_parent_flops`
- `cargo test phase4_hierarchy_matrix_covers_wrapper_and_recursive_profiles`
- `cargo test phase4_hierarchy_gate_raises_designs_per_scenario_for_matrix`
- `cargo test phase4_hierarchy_coverage_requires_design_facts`
- `cargo run --bin tool_matrix -- --out /tmp/anvil-tool-matrix-phase4-hierarchy-r30-coverage --phase4-hierarchy-gate --skip-verilator --skip-yosys`
  - `63` scenarios
  - `252` generated designs
  - `coverage_gaps = []`
  - `saw_hierarchy_parent_composed_parent_cone_instance_flop_routing = true`
- `cargo run --bin tool_matrix -- --out /tmp/anvil-tool-matrix-phase4-hierarchy-r30 --phase4-hierarchy-gate --yosys-mode both`
  - `63` scenarios
  - `4` designs/scenario
  - `252` total designs
  - `coverage_gaps = []`
  - Verilator `252/0`
  - Yosys without-ABC `252/0`
  - Yosys with-ABC `252/0`
- `cargo check --all-targets`
- `cargo test`
  - 223 unit-target tests + 58 integration tests = 281 passing tests
- `cargo clippy --all-targets -- -D warnings`
- `cargo fmt --all --check`
- `mdbook build book`
- `git diff --check`

**Impact**

- `r30` is now the current Phase 4 hierarchy closure artifact. `r29`
  remains the previous full downstream-clean multi-stage registered
  parent-composed helper bank.
- Existing defaults are unchanged. The new route is opt-in through the
  existing `hierarchy_child_input_cone_prob`,
  `hierarchy_parent_cone_instance_prob`, `hierarchy_parent_flop_prob`,
  `max_parent_cone_instances_per_module`, and
  `max_flops_per_module` knobs.

**Files touched**

- [CHANGES.md](/Users/richarddje/Documents/github/anvil/CHANGES.md)
- [README.md](/Users/richarddje/Documents/github/anvil/README.md)
- [USER_GUIDE.md](/Users/richarddje/Documents/github/anvil/USER_GUIDE.md)
- [ROADMAP.md](/Users/richarddje/Documents/github/anvil/ROADMAP.md)
- [CODEBASE_ANALYSIS.md](/Users/richarddje/Documents/github/anvil/CODEBASE_ANALYSIS.md)
- [DEVELOPMENT_NOTES.md](/Users/richarddje/Documents/github/anvil/DEVELOPMENT_NOTES.md)
- [MEMORY.md](/Users/richarddje/Documents/github/anvil/MEMORY.md)
- [book/src/architecture.md](/Users/richarddje/Documents/github/anvil/book/src/architecture.md)
- [book/src/hierarchy.md](/Users/richarddje/Documents/github/anvil/book/src/hierarchy.md)
- [book/src/ir.md](/Users/richarddje/Documents/github/anvil/book/src/ir.md)
- [book/src/knobs.md](/Users/richarddje/Documents/github/anvil/book/src/knobs.md)
- [book/src/recipes.md](/Users/richarddje/Documents/github/anvil/book/src/recipes.md)
- [src/bin/tool_matrix.rs](/Users/richarddje/Documents/github/anvil/src/bin/tool_matrix.rs)
- [src/gen/hierarchy.rs](/Users/richarddje/Documents/github/anvil/src/gen/hierarchy.rs)
- [src/metrics.rs](/Users/richarddje/Documents/github/anvil/src/metrics.rs)
- [tests/pipeline.rs](/Users/richarddje/Documents/github/anvil/tests/pipeline.rs)

## 2026-04-29-phase4-parent-composed-helper-multistage — Prove parent-composed helper chains through parent state

**Landed as:** `cd079c4`

**What changed**

- Added structural metrics for multi-stage registered
  parent-composed helper routing:
  `child_input_bindings_from_registered_multistage_parent_composed_parent_cone_instances`,
  `top_child_input_bindings_from_registered_multistage_parent_composed_parent_cone_instances`,
  `registered_multistage_parent_composed_parent_cone_instance_child_input_binding_fraction`,
  and
  `top_registered_multistage_parent_composed_parent_cone_instance_child_input_binding_fraction`.
- The new metric distinguishes the parent-composed helper chain from
  the existing direct registered sibling helper chain: it requires the
  final registered D side to be parent-composed logic whose flop
  dependency reaches a parent-cone helper output.
- Added focused regressions:
  `design_metrics_capture_multistage_registered_parent_composed_parent_cone_instance_routes`
  and
  `hierarchy_registered_parent_composed_routes_can_chain_helper_instances_through_parent_flops`.
- Expanded the Phase 4 hierarchy matrix from 57 to 60 scenarios by
  adding
  `phase4_hier2_inst4_registered_parent_cone_instance_multistage_state`
  for each construction strategy. The matrix coverage contract now
  requires
  `saw_hierarchy_registered_multistage_parent_composed_parent_cone_instance_routing = true`.
- Banked the refreshed full downstream-clean Phase 4 hierarchy matrix
  at `/tmp/anvil-tool-matrix-phase4-hierarchy-r29/tool_matrix_report.json`:
  `60` scenarios, `4` designs/scenario, `240` total designs,
  `artifact_kind = "design"`, `coverage_gaps = []`, Verilator
  `240/0`, Yosys without-ABC `240/0`, and Yosys with-ABC `240/0`.
- README, USER_GUIDE, ROADMAP, CODEBASE_ANALYSIS, DEVELOPMENT_NOTES,
  MEMORY, and the mdBook hierarchy / architecture / knobs / recipes
  chapters now document the new route, metrics, examples, and `r29`
  evidence anchor.

**Why**

- `r28` proved the direct registered sibling helper cross-product: a
  helper output can seed one parent Q and a later direct registered
  sibling route can reuse that Q. The parent-composed route needed its
  own proof because the final D side is parent logic, not the direct
  sibling path.
- Users reading `manifest.json` or `tool_matrix_report.json` can now
  tell whether helper-through-state support came from direct sibling
  routing or registered parent-composed routing.

**Validation**

- `cargo test design_metrics_capture_multistage_registered_parent_composed_parent_cone_instance_routes`
- `cargo test hierarchy_registered_parent_composed_routes_can_chain_helper_instances_through_parent_flops`
- `cargo test phase4_hierarchy_matrix_covers_wrapper_and_recursive_profiles`
- `cargo test phase4_hierarchy_gate_raises_designs_per_scenario_for_matrix`
- `cargo test phase4_hierarchy_coverage_requires_design_facts`
- `cargo run --bin tool_matrix -- --out /tmp/anvil-tool-matrix-phase4-hierarchy-r29-coverage --phase4-hierarchy-gate --skip-verilator --skip-yosys`
  - `60` scenarios
  - `240` generated designs
  - `coverage_gaps = []`
  - `saw_hierarchy_registered_multistage_parent_composed_parent_cone_instance_routing = true`
- `cargo run --bin tool_matrix -- --out /tmp/anvil-tool-matrix-phase4-hierarchy-r29 --phase4-hierarchy-gate --yosys-mode both`
  - `60` scenarios
  - `4` designs/scenario
  - `240` total designs
  - `coverage_gaps = []`
  - Verilator `240/0`
  - Yosys without-ABC `240/0`
  - Yosys with-ABC `240/0`
- `cargo check --all-targets`
- `cargo test`
  - 222 unit-target tests + 57 integration tests = 279 passing tests
- `cargo clippy --all-targets -- -D warnings`
- `cargo fmt --all --check`
- `mdbook build book`
- `git diff --check`

**Impact**

- `r29` is now the current Phase 4 hierarchy closure artifact. `r28`
  remains the previous full downstream-clean direct registered sibling
  helper-chain bank.
- Existing defaults are unchanged. The route uses existing knobs:
  `hierarchy_registered_child_input_cone_prob`,
  `hierarchy_parent_cone_instance_prob`,
  `max_parent_cone_instances_per_module`, and
  `max_flops_per_module`.

**Files touched**

- [CHANGES.md](/Users/richarddje/Documents/github/anvil/CHANGES.md)
- [README.md](/Users/richarddje/Documents/github/anvil/README.md)
- [USER_GUIDE.md](/Users/richarddje/Documents/github/anvil/USER_GUIDE.md)
- [ROADMAP.md](/Users/richarddje/Documents/github/anvil/ROADMAP.md)
- [CODEBASE_ANALYSIS.md](/Users/richarddje/Documents/github/anvil/CODEBASE_ANALYSIS.md)
- [DEVELOPMENT_NOTES.md](/Users/richarddje/Documents/github/anvil/DEVELOPMENT_NOTES.md)
- [MEMORY.md](/Users/richarddje/Documents/github/anvil/MEMORY.md)
- [book/src/architecture.md](/Users/richarddje/Documents/github/anvil/book/src/architecture.md)
- [book/src/hierarchy.md](/Users/richarddje/Documents/github/anvil/book/src/hierarchy.md)
- [book/src/knobs.md](/Users/richarddje/Documents/github/anvil/book/src/knobs.md)
- [book/src/recipes.md](/Users/richarddje/Documents/github/anvil/book/src/recipes.md)
- [src/metrics.rs](/Users/richarddje/Documents/github/anvil/src/metrics.rs)
- [src/bin/tool_matrix.rs](/Users/richarddje/Documents/github/anvil/src/bin/tool_matrix.rs)
- [tests/pipeline.rs](/Users/richarddje/Documents/github/anvil/tests/pipeline.rs)

## 2026-04-29-phase4-registered-helper-multistage — Chain registered helper routes through parent state

**Landed as:** `85866c3`

**What changed**

- Added structural metrics for multi-stage direct registered sibling
  helper routing:
  `child_input_bindings_from_registered_multistage_parent_cone_instances`,
  `top_child_input_bindings_from_registered_multistage_parent_cone_instances`,
  `registered_multistage_parent_cone_instance_child_input_binding_fraction`,
  and
  `top_registered_multistage_parent_cone_instance_child_input_binding_fraction`.
- The metric walks flop virtual dependencies with the existing
  parent-cone helper reachability memo, so it proves
  helper output -> parent flop -> later parent flop -> later child input
  without classifying the route as registered parent-composed logic.
- Added focused regressions for the metric and the generated route:
  `design_metrics_capture_multistage_registered_parent_cone_instance_routes`
  and
  `hierarchy_registered_sibling_routes_can_chain_helper_instances_through_parent_flops`.
- Expanded the Phase 4 hierarchy matrix from 54 to 57 scenarios by
  adding
  `phase4_hier2_inst4_registered_sibling_parent_cone_instance_multistage_state`
  for each construction strategy. The matrix coverage contract now
  requires
  `saw_hierarchy_registered_multistage_parent_cone_instance_routing = true`.
- Banked the refreshed full downstream-clean Phase 4 hierarchy matrix
  at `/tmp/anvil-tool-matrix-phase4-hierarchy-r28/tool_matrix_report.json`:
  `57` scenarios, `4` designs/scenario, `228` total designs,
  `artifact_kind = "design"`, `coverage_gaps = []`, Verilator
  `228/0`, Yosys without-ABC `228/0`, and Yosys with-ABC `228/0`.
- README, USER_GUIDE, ROADMAP, CODEBASE_ANALYSIS, DEVELOPMENT_NOTES,
  MEMORY, and the mdBook hierarchy / architecture / knobs / recipes
  chapters now document the new route, metrics, examples, and `r28`
  evidence anchor.

**Why**

- The previous slices proved direct registered helper routing and
  direct registered sibling multi-stage chaining separately. This slice
  lands their cross-product: a helper-sourced parent Q can be reused by
  a later direct registered sibling route without parent-composed
  logic.
- Users reading the book or a saved matrix report can now distinguish
  first-stage helper D paths from helper-origin signals carried through
  multiple parent-local flop stages.

**Validation**

- `cargo test hierarchy_registered_sibling_routes_can_chain_helper_instances_through_parent_flops`
- `cargo test design_metrics_capture_multistage_registered_parent_cone_instance_routes`
- `cargo test phase4_hierarchy_matrix_covers_wrapper_and_recursive_profiles`
- `cargo test phase4_hierarchy_gate_raises_designs_per_scenario_for_matrix`
- `cargo test phase4_hierarchy_coverage_requires_design_facts`
- `cargo run --bin tool_matrix -- --out /tmp/anvil-tool-matrix-phase4-hierarchy-r28-coverage --phase4-hierarchy-gate --skip-verilator --skip-yosys`
  - `57` scenarios
  - `228` generated designs
  - `coverage_gaps = []`
  - `saw_hierarchy_registered_multistage_parent_cone_instance_routing = true`
- `cargo run --bin tool_matrix -- --out /tmp/anvil-tool-matrix-phase4-hierarchy-r28 --phase4-hierarchy-gate --yosys-mode both`
  - `57` scenarios
  - `4` designs/scenario
  - `228` total designs
  - `coverage_gaps = []`
  - Verilator `228/0`
  - Yosys without-ABC `228/0`
  - Yosys with-ABC `228/0`
- `cargo check --all-targets`
- `cargo test`
  - 221 unit-target tests + 56 integration tests = 277 passing tests
- `cargo clippy --all-targets -- -D warnings`
- `cargo fmt --all --check`
- `mdbook build book`
- `git diff --check`

**Impact**

- `r28` is now the current Phase 4 hierarchy closure artifact. `r27`
  remains the previous full downstream-clean stateful parent-output
  helper bank.
- The route uses existing knobs:
  `hierarchy_registered_sibling_route_prob`,
  `hierarchy_parent_cone_instance_prob`, and
  `max_parent_cone_instances_per_module`. Existing defaults are
  unchanged.

**Files touched**

- [CHANGES.md](/Users/richarddje/Documents/github/anvil/CHANGES.md)
- [README.md](/Users/richarddje/Documents/github/anvil/README.md)
- [USER_GUIDE.md](/Users/richarddje/Documents/github/anvil/USER_GUIDE.md)
- [ROADMAP.md](/Users/richarddje/Documents/github/anvil/ROADMAP.md)
- [CODEBASE_ANALYSIS.md](/Users/richarddje/Documents/github/anvil/CODEBASE_ANALYSIS.md)
- [DEVELOPMENT_NOTES.md](/Users/richarddje/Documents/github/anvil/DEVELOPMENT_NOTES.md)
- [MEMORY.md](/Users/richarddje/Documents/github/anvil/MEMORY.md)
- [book/src/hierarchy.md](/Users/richarddje/Documents/github/anvil/book/src/hierarchy.md)
- [book/src/architecture.md](/Users/richarddje/Documents/github/anvil/book/src/architecture.md)
- [book/src/knobs.md](/Users/richarddje/Documents/github/anvil/book/src/knobs.md)
- [book/src/recipes.md](/Users/richarddje/Documents/github/anvil/book/src/recipes.md)
- [src/metrics.rs](/Users/richarddje/Documents/github/anvil/src/metrics.rs)
- [src/bin/tool_matrix.rs](/Users/richarddje/Documents/github/anvil/src/bin/tool_matrix.rs)
- [tests/pipeline.rs](/Users/richarddje/Documents/github/anvil/tests/pipeline.rs)

## 2026-04-29-phase4-parent-output-helper-state — Route parent-output helpers through parent state

**Landed as:** `ad3e7d9`

**What changed**

- Parent-output helper composition can now optionally route the required
  parent-cone helper source through a parent-local flop before it feeds
  the parent output, controlled by the existing
  `hierarchy_parent_flop_prob` knob and bounded by
  `max_flops_per_module`.
- `DesignMetrics` now reports the new stateful output route through:
  `top_outputs_reaching_parent_cone_instances_through_parent_flops`,
  `hierarchy_outputs_reaching_parent_cone_instances_through_parent_flops`,
  `top_parent_cone_instance_flop_output_fraction`, and
  `hierarchy_parent_cone_instance_flop_output_fraction`.
- The metric implementation is dependency/memo based: it skips the
  expensive helper-through-flop walk entirely for modules with no
  parent-cone helper instances, then uses flop virtual dependencies and
  the existing instance-output-support memo for modules where the route
  is possible.
- Added the focused regression
  `hierarchy_parent_outputs_can_route_helper_instances_through_parent_flops`
  to prove the route across construction strategies while keeping
  child-input helper bindings at zero for the output-only proof.
- Expanded the Phase 4 hierarchy matrix from 51 to 54 scenarios by
  adding the dedicated
  `phase4_hier2_inst4_parent_output_cone_instance_state` scenario for
  each construction strategy. The matrix coverage contract now requires
  `saw_hierarchy_parent_cone_instance_flop_outputs = true`.
- Banked the refreshed full downstream-clean Phase 4 hierarchy matrix
  at `/tmp/anvil-tool-matrix-phase4-hierarchy-r27/tool_matrix_report.json`:
  `54` scenarios, `4` designs/scenario, `216` total designs,
  `artifact_kind = "design"`, `coverage_gaps = []`, Verilator
  `216/0`, Yosys without-ABC `216/0`, and Yosys with-ABC `216/0`.
- README, USER_GUIDE, ROADMAP, CODEBASE_ANALYSIS, DEVELOPMENT_NOTES,
  MEMORY, and the mdBook hierarchy / architecture / knobs / recipes
  chapters now document the new route, metrics, examples, and `r27`
  evidence anchor.

**Why**

- The previous parent-output helper route proved direct combinational
  helper-to-output support, and the previous parent-state routes proved
  stateful child-input composition. The next useful hierarchy slice was
  the cross-product: parent outputs sourced from helper children through
  local parent state, without relying on child-input helper bindings.
- Users and developers need the Phase 4 gate to prove this route from a
  saved report fact, not only from a focused regression.

**Validation**

- `cargo test hierarchy_parent_outputs_can_route_helper_instances_through_parent_flops`
- `cargo test phase4_hierarchy`
- `cargo run --bin tool_matrix -- --out /tmp/anvil-tool-matrix-phase4-parent-output-helper-state-r3 --phase4-hierarchy-gate --skip-verilator --skip-yosys`
  - `54` scenarios
  - `216` generated designs
  - `coverage_gaps = []`
  - `saw_hierarchy_parent_cone_instance_flop_outputs = true`
  - max
    `hierarchy_outputs_reaching_parent_cone_instances_through_parent_flops = 16`
  - max
    `top_outputs_reaching_parent_cone_instances_through_parent_flops = 16`
- `cargo run --bin tool_matrix -- --out /tmp/anvil-tool-matrix-phase4-hierarchy-r27 --phase4-hierarchy-gate --yosys-mode both`
  - `54` scenarios
  - `4` designs/scenario
  - `216` total designs
  - `coverage_gaps = []`
  - Verilator `216/0`
  - Yosys without-ABC `216/0`
  - Yosys with-ABC `216/0`
- `cargo check --all-targets`
- `cargo test`
  - 220 unit-target tests + 55 integration tests = 275 passing tests
- `cargo clippy --all-targets -- -D warnings`
- `cargo fmt --all --check`
- `mdbook build book`
- `git diff --check`

**Impact**

- `r27` is now the current Phase 4 hierarchy closure artifact. `r26`
  remains the previous full downstream-clean multi-stage registered
  sibling bank.
- The route is opt-in through the existing
  `hierarchy_parent_cone_instance_prob` and `hierarchy_parent_flop_prob`
  knobs. Existing defaults are unchanged.

**Files touched**

- [CHANGES.md](/Users/richarddje/Documents/github/anvil/CHANGES.md)
- [README.md](/Users/richarddje/Documents/github/anvil/README.md)
- [USER_GUIDE.md](/Users/richarddje/Documents/github/anvil/USER_GUIDE.md)
- [ROADMAP.md](/Users/richarddje/Documents/github/anvil/ROADMAP.md)
- [CODEBASE_ANALYSIS.md](/Users/richarddje/Documents/github/anvil/CODEBASE_ANALYSIS.md)
- [DEVELOPMENT_NOTES.md](/Users/richarddje/Documents/github/anvil/DEVELOPMENT_NOTES.md)
- [MEMORY.md](/Users/richarddje/Documents/github/anvil/MEMORY.md)
- [book/src/architecture.md](/Users/richarddje/Documents/github/anvil/book/src/architecture.md)
- [book/src/hierarchy.md](/Users/richarddje/Documents/github/anvil/book/src/hierarchy.md)
- [book/src/knobs.md](/Users/richarddje/Documents/github/anvil/book/src/knobs.md)
- [book/src/recipes.md](/Users/richarddje/Documents/github/anvil/book/src/recipes.md)
- [src/bin/tool_matrix.rs](/Users/richarddje/Documents/github/anvil/src/bin/tool_matrix.rs)
- [src/gen/hierarchy.rs](/Users/richarddje/Documents/github/anvil/src/gen/hierarchy.rs)
- [src/metrics.rs](/Users/richarddje/Documents/github/anvil/src/metrics.rs)
- [tests/pipeline.rs](/Users/richarddje/Documents/github/anvil/tests/pipeline.rs)

## 2026-04-29-phase4-registered-sibling-multistage — Chain registered sibling routes through parent state

**Landed as:** `1f57cea`

**What changed**

- Direct registered sibling routing can now choose an earlier
  parent-local Q as a later route's parent-flop D source when no helper
  source is required. This creates a multi-stage registered
  child-to-child chain without reclassifying the route as registered
  parent-composed logic.
- `DesignMetrics` now reports the new surface through:
  `child_input_bindings_from_registered_multistage_instance_outputs`,
  `top_child_input_bindings_from_registered_multistage_instance_outputs`,
  `registered_multistage_instance_output_child_input_binding_fraction`,
  and
  `top_registered_multistage_instance_output_child_input_binding_fraction`.
- Added the focused regression
  `hierarchy_registered_sibling_routes_can_chain_through_parent_flops`
  to prove the route across construction strategies while keeping the
  registered parent-composed counters at zero.
- Expanded the Phase 4 hierarchy matrix from 48 to 51 scenarios by
  adding the dedicated
  `phase4_hier2_inst4_registered_sibling_multistage_state` scenario
  for each construction strategy. The matrix coverage contract now
  requires
  `saw_hierarchy_registered_multistage_sibling_routing = true`.
- Banked the refreshed full downstream-clean Phase 4 hierarchy matrix
  at `/tmp/anvil-tool-matrix-phase4-hierarchy-r26/tool_matrix_report.json`:
  `51` scenarios, `4` designs/scenario, `204` total designs,
  `artifact_kind = "design"`, `coverage_gaps = []`, Verilator
  `204/0`, Yosys without-ABC `204/0`, and Yosys with-ABC `204/0`.
- README, USER_GUIDE, ROADMAP, CODEBASE_ANALYSIS, DEVELOPMENT_NOTES,
  MEMORY, and the mdBook hierarchy / architecture / knobs / recipes
  chapters now document the new route, knobs, metrics, examples, and
  `r26` evidence anchor.

**Why**

- The previous registered sibling route proved a one-flop stateful
  handoff, and the previous multi-stage route was specific to
  registered parent-composed logic. The roadmap's next useful hierarchy
  slice was the narrower child-to-child chain: earlier child output to
  parent state, then that parent state to a later registered child-input
  binding, with metrics proving the shape directly.
- Users and developers need one current Phase 4 closure artifact for
  the live hierarchy surface. The new scenario makes the matrix prove
  this route instead of relying on the focused regression alone.

**Validation**

- `cargo test hierarchy_registered_sibling_routes_can_chain_through_parent_flops`
- `cargo test phase4_hierarchy`
- `cargo run --bin tool_matrix -- --phase4-hierarchy-gate --list-scenarios`
  - listed 51 scenarios, including
    `phase4_hier2_inst4_registered_sibling_multistage_state` for
    sequential, shuffled, and interleaved strategies
- `cargo run --bin tool_matrix -- --out /tmp/anvil-tool-matrix-phase4-registered-sibling-multistage-r1 --phase4-hierarchy-gate --skip-verilator --skip-yosys`
  - `51` scenarios
  - `204` generated designs
  - `coverage_gaps = []`
  - `saw_hierarchy_registered_multistage_sibling_routing = true`
  - max
    `child_input_bindings_from_registered_multistage_instance_outputs = 6`
- `cargo run --bin tool_matrix -- --out /tmp/anvil-tool-matrix-phase4-hierarchy-r26 --phase4-hierarchy-gate --yosys-mode both`
  - `51` scenarios
  - `4` designs/scenario
  - `204` total designs
  - `coverage_gaps = []`
  - Verilator `204/0`
  - Yosys without-ABC `204/0`
  - Yosys with-ABC `204/0`
- `cargo check --all-targets`
- `cargo test`
  - 220 unit-target tests + 54 integration tests = 274 passing tests
- `cargo clippy --all-targets -- -D warnings`
- `cargo fmt --all --check`
- `mdbook build book`
- `git diff --check`

**Impact**

- `r26` is now the current Phase 4 hierarchy closure artifact. `r25`
  remains the previous full downstream-clean direct-helper bank, `r24`
  remains the coverage-only direct-helper policy proof, and `r23`
  remains the pre-direct-helper full bank.
- The route is opt-in through the existing
  `hierarchy_registered_sibling_route_prob` knob. Existing defaults are
  unchanged.

**Files touched**

- [CHANGES.md](/Users/richarddje/Documents/github/anvil/CHANGES.md)
- [README.md](/Users/richarddje/Documents/github/anvil/README.md)
- [USER_GUIDE.md](/Users/richarddje/Documents/github/anvil/USER_GUIDE.md)
- [ROADMAP.md](/Users/richarddje/Documents/github/anvil/ROADMAP.md)
- [CODEBASE_ANALYSIS.md](/Users/richarddje/Documents/github/anvil/CODEBASE_ANALYSIS.md)
- [DEVELOPMENT_NOTES.md](/Users/richarddje/Documents/github/anvil/DEVELOPMENT_NOTES.md)
- [MEMORY.md](/Users/richarddje/Documents/github/anvil/MEMORY.md)
- [book/src/architecture.md](/Users/richarddje/Documents/github/anvil/book/src/architecture.md)
- [book/src/hierarchy.md](/Users/richarddje/Documents/github/anvil/book/src/hierarchy.md)
- [book/src/knobs.md](/Users/richarddje/Documents/github/anvil/book/src/knobs.md)
- [book/src/recipes.md](/Users/richarddje/Documents/github/anvil/book/src/recipes.md)
- [src/bin/tool_matrix.rs](/Users/richarddje/Documents/github/anvil/src/bin/tool_matrix.rs)
- [src/config.rs](/Users/richarddje/Documents/github/anvil/src/config.rs)
- [src/gen/hierarchy.rs](/Users/richarddje/Documents/github/anvil/src/gen/hierarchy.rs)
- [src/metrics.rs](/Users/richarddje/Documents/github/anvil/src/metrics.rs)
- [tests/pipeline.rs](/Users/richarddje/Documents/github/anvil/tests/pipeline.rs)

## 2026-04-29-phase4-hierarchy-r25-full-bank — Bank the current Phase 4 hierarchy matrix

**Landed as:** `25abd72`

**What changed**

- Ran the full repo-owned Phase 4 hierarchy gate with downstream tools:
  `cargo run --bin tool_matrix -- --out /tmp/anvil-tool-matrix-phase4-hierarchy-r25 --phase4-hierarchy-gate --yosys-mode both`.
- The current 48-scenario hierarchy policy is now banked at
  `/tmp/anvil-tool-matrix-phase4-hierarchy-r25/tool_matrix_report.json`:
  `4` designs/scenario, `192` total designs,
  `artifact_kind = "design"`, `coverage_gaps = []`, Verilator
  `192/0`, Yosys without-ABC `192/0`, and Yosys with-ABC `192/0`.
- That `r25` report includes the two helper routes that previously only
  had coverage-only policy evidence:
  `saw_hierarchy_direct_sibling_parent_cone_instance_routing = true`
  and
  `saw_hierarchy_direct_registered_sibling_parent_cone_instance_routing = true`.
- README, USER_GUIDE, ROADMAP, CODEBASE_ANALYSIS, DEVELOPMENT_NOTES,
  MEMORY, and the mdBook hierarchy/architecture chapters now point to
  `r25` as the current Phase 4 closure artifact. The old `r23` full
  bank and `r24` coverage-only direct-helper proof are documented as
  historical breadcrumbs instead of current limitations.

**Why**

- The prior full downstream-clean Phase 4 bank (`r23`) had 42 scenarios
  / 168 designs and predated the direct sibling helper and direct
  registered sibling helper routes.
- The expanded 48-scenario policy was covered by the skip-tools `r24`
  report, but users and developers need one current full-bank artifact
  for the live hierarchy surface.

**Validation**

- `cargo run --bin tool_matrix -- --out /tmp/anvil-tool-matrix-phase4-hierarchy-r25 --phase4-hierarchy-gate --yosys-mode both`
  - `48` scenarios
  - `4` designs/scenario
  - `192` total designs
  - `coverage_gaps = []`
  - Verilator `192/0`
  - Yosys without-ABC `192/0`
  - Yosys with-ABC `192/0`
- `cargo check --all-targets`
- `cargo test`
- `cargo clippy --all-targets -- -D warnings`
- `cargo fmt --all --check`
- `mdbook build book`
- `git diff --check`

**Impact**

- Current Phase 4 hierarchy evidence is no longer split across `r23`
  and `r24`; the live closure anchor is `r25`.
- No generator semantics changed in this slice. This commit banks
  evidence and aligns live documentation, especially the user-facing
  mdBook.

**Files touched**

- [CHANGES.md](/Users/richarddje/Documents/github/anvil/CHANGES.md)
- [README.md](/Users/richarddje/Documents/github/anvil/README.md)
- [USER_GUIDE.md](/Users/richarddje/Documents/github/anvil/USER_GUIDE.md)
- [ROADMAP.md](/Users/richarddje/Documents/github/anvil/ROADMAP.md)
- [CODEBASE_ANALYSIS.md](/Users/richarddje/Documents/github/anvil/CODEBASE_ANALYSIS.md)
- [DEVELOPMENT_NOTES.md](/Users/richarddje/Documents/github/anvil/DEVELOPMENT_NOTES.md)
- [MEMORY.md](/Users/richarddje/Documents/github/anvil/MEMORY.md)
- [book/src/hierarchy.md](/Users/richarddje/Documents/github/anvil/book/src/hierarchy.md)
- [book/src/architecture.md](/Users/richarddje/Documents/github/anvil/book/src/architecture.md)

## 2026-04-28-readme-bootstrap-default-run — Restore the README cargo-run entrypoint

**Landed as:** `d4cb9c1`

**What changed**

- `Cargo.toml` now sets `default-run = "anvil"`, so the README and
  mdBook examples using plain `cargo run -- ...` invoke the generator
  even though the repository also has the auxiliary `tool_matrix`
  binary.
- README, USER_GUIDE, mdBook Getting Started, mdBook Architecture, and
  DEVELOPMENT_NOTES now document the command split explicitly:
  `cargo run -- ...` for `anvil`, and
  `cargo run --bin tool_matrix -- ...` for the matrix harness.
- `CODEBASE_ANALYSIS.md` now describes the actual target set as two
  binaries, one library, one example, and one integration test.
- `book/src/knobs.md` now includes
  `--max-parent-cone-instances-per-module` in the CLI hierarchy
  snapshot, matching `anvil --help`.
- `book/src/structural-rules.md` no longer says M-to-1 muxes only
  exist inside flop D-inputs. It now reflects the current generator:
  general M-to-1 muxes are live as combinational blocks (Rule 15) and
  as flop-D mux motifs.
- MEMORY captures both bootstrapping findings for future sessions.

**Why**

- Meticulously executing README surfaced a real command failure:
  `cargo run -- --help` was ambiguous because Cargo saw both `anvil`
  and `tool_matrix`. The docs already taught the correct UX; the crate
  metadata needed to preserve it.
- The same README/bootstrap pass found mdBook drift in the structural
  rules and hierarchy CLI snapshot. The book is user-facing, so these
  details need to be current instead of left as archaeology.

**Validation**

- `cargo check --all-targets`
- `cargo test`
- `cargo clippy --all-targets -- -D warnings`
- `cargo fmt --all --check`
- `mdbook build book`
- `git diff --check`
- `cargo run -- --help`
- Help-to-book CLI audit: all `74` `anvil` long flags and all `17`
  `tool_matrix` long flags are present in `book/src/knobs.md`.
- README generator command sweep, writing generated artifacts under
  `/tmp/anvil-readme-cmd-*-20260428-r1`, covering the documented leaf,
  batch, depth-1 hierarchy, child-definition reuse, bounded recursive
  hierarchy, per-depth branching, sibling routing, helper routing,
  parent-composed child inputs, parent-local flops, registered sibling,
  registered helper, registered parent-composed, parent-output-helper,
  hierarchy batch, `--dump-config`, and `--config` replay examples.
- `cargo run --example generate_one`
- `cargo run --bin tool_matrix -- --list-scenarios`
- `cargo run --bin tool_matrix -- --phase4-hierarchy-gate --list-scenarios`
- `cargo run --bin tool_matrix -- --out /tmp/anvil-readme-tool-matrix-smoke-20260428-r1 --modules-per-scenario 1 --yosys-mode both`
  - `15` scenarios, `15` modules total
  - Verilator `15/0`
  - Yosys without-abc `15/0`
  - Yosys with-abc `15/0`
- Optional README downstream smoke on
  `/tmp/anvil-readme-default-run-20260428-r1/mod_42_0000.sv`:
  - `verilator --lint-only ...`
  - `yosys -p "read_verilog -sv ...; synth -noabc; stat"` with
    `Found and reported 0 problems`.

**Impact**

- Cold-start source-tree commands from README and the mdBook work as
  written again.
- No generator semantics changed; this is executable metadata plus live
  documentation alignment.

**Files touched**

- [Cargo.toml](/Users/richarddje/Documents/github/anvil/Cargo.toml)
- [CHANGES.md](/Users/richarddje/Documents/github/anvil/CHANGES.md)
- [CODEBASE_ANALYSIS.md](/Users/richarddje/Documents/github/anvil/CODEBASE_ANALYSIS.md)
- [DEVELOPMENT_NOTES.md](/Users/richarddje/Documents/github/anvil/DEVELOPMENT_NOTES.md)
- [MEMORY.md](/Users/richarddje/Documents/github/anvil/MEMORY.md)
- [README.md](/Users/richarddje/Documents/github/anvil/README.md)
- [USER_GUIDE.md](/Users/richarddje/Documents/github/anvil/USER_GUIDE.md)
- [book/src/architecture.md](/Users/richarddje/Documents/github/anvil/book/src/architecture.md)
- [book/src/getting-started.md](/Users/richarddje/Documents/github/anvil/book/src/getting-started.md)
- [book/src/knobs.md](/Users/richarddje/Documents/github/anvil/book/src/knobs.md)
- [book/src/structural-rules.md](/Users/richarddje/Documents/github/anvil/book/src/structural-rules.md)

## 2026-04-27-phase4-matrix-direct-helper-routes — Bank direct sibling helper routes in Phase 4 matrix

**Landed as:** `c2b4118`

**What changed**

- [src/bin/tool_matrix.rs](/Users/richarddje/Documents/github/anvil/src/bin/tool_matrix.rs)
  now expands the repo-owned Phase 4 hierarchy scenario set from `42`
  to `48` scenarios by adding two focused helper-instance routes for
  each construction strategy:
  `phase4_hier2_inst4_direct_sibling_parent_cone_instance` and
  `phase4_hier2_inst4_direct_registered_sibling_parent_cone_instance_state`.
- The Phase 4 gate still preserves the four-design per-scenario floor,
  so the current code plan is now `48` scenarios × `4` designs/scenario
  = `192` total designs.
- `CoverageSummary` now records separate facts for direct unregistered
  sibling helper routing and direct registered sibling helper routing,
  instead of relying only on the older generic parent-cone helper and
  registered-helper coverage booleans.
- The Phase 4 coverage-gap policy now has distinct gap messages for the
  two direct helper routes, preventing a future matrix run from passing
  by proving only parent-composed helper paths.
- Live docs now distinguish the current 48-scenario matrix policy and
  coverage-only `r24` proof from the latest full downstream-clean `r23`
  bank, which predates both direct helper routes.

**Why**

- The direct registered sibling helper route (`0e3e833`) and the direct
  unregistered sibling helper route (`d6ccd22`) landed after the last
  full downstream-clean Phase 4 hierarchy bank. The matrix policy still
  represented the older 42-scenario plan and did not require those new
  routes explicitly.
- The existing generic helper facts could be satisfied by
  parent-composed helper paths. The direct routes need their own focused
  scenarios and their own coverage facts so regressions cannot hide
  behind broader helper-instance coverage.

**Validation**

- `cargo fmt --all --manifest-path /Users/richarddje/Documents/github/anvil/Cargo.toml`
- `cargo test --manifest-path /Users/richarddje/Documents/github/anvil/Cargo.toml --bin tool_matrix phase4_hierarchy`
- `cargo test --manifest-path /Users/richarddje/Documents/github/anvil/Cargo.toml --test pipeline hierarchy_sibling_routes_can_use_helper_instances`
- `cargo test --manifest-path /Users/richarddje/Documents/github/anvil/Cargo.toml --test pipeline hierarchy_registered_sibling_routes_can_use_helper_instances`
- `cargo test --manifest-path /Users/richarddje/Documents/github/anvil/Cargo.toml design_metrics_capture_direct`
- `cargo run --manifest-path /Users/richarddje/Documents/github/anvil/Cargo.toml --bin tool_matrix -- --out /tmp/anvil-tool-matrix-phase4-direct-helper-r24 --phase4-hierarchy-gate --skip-verilator --skip-yosys --fail-on-coverage-gap`
- `cargo check --manifest-path /Users/richarddje/Documents/github/anvil/Cargo.toml --all-targets`
- `cargo test --manifest-path /Users/richarddje/Documents/github/anvil/Cargo.toml`
- `cargo clippy --manifest-path /Users/richarddje/Documents/github/anvil/Cargo.toml --all-targets -- -D warnings`
- `cargo fmt --all --check`
- `mdbook build book`
- `git diff --check`

**Impact**

- The live Phase 4 hierarchy matrix policy now requires both direct
  sibling helper routes in addition to the previously banked
  parent-composed, registered parent-composed, parent-output, and
  budgeted helper-instance routes.
- The coverage-only `r24` report at
  `/tmp/anvil-tool-matrix-phase4-direct-helper-r24/tool_matrix_report.json`
  closes the current 48-scenario policy with `coverage_gaps = []` and
  `192` generated designs while Verilator and Yosys are intentionally
  skipped.
- The latest full downstream-clean Phase 4 hierarchy evidence remains
  `/tmp/anvil-tool-matrix-phase4-hierarchy-r23/tool_matrix_report.json`:
  `42` scenarios, `168` designs, `coverage_gaps = []`, and `168/0`
  pass-fail in Verilator plus both repo-owned Yosys modes. It should not
  be described as proving the two direct helper routes until a new full
  downstream matrix is run.
- Generator behavior is unchanged; this is a matrix policy, coverage,
  and documentation slice.

**Files touched**

- [CHANGES.md](/Users/richarddje/Documents/github/anvil/CHANGES.md)
- [CODEBASE_ANALYSIS.md](/Users/richarddje/Documents/github/anvil/CODEBASE_ANALYSIS.md)
- [DEVELOPMENT_NOTES.md](/Users/richarddje/Documents/github/anvil/DEVELOPMENT_NOTES.md)
- [MEMORY.md](/Users/richarddje/Documents/github/anvil/MEMORY.md)
- [README.md](/Users/richarddje/Documents/github/anvil/README.md)
- [ROADMAP.md](/Users/richarddje/Documents/github/anvil/ROADMAP.md)
- [USER_GUIDE.md](/Users/richarddje/Documents/github/anvil/USER_GUIDE.md)
- [book/src/hierarchy.md](/Users/richarddje/Documents/github/anvil/book/src/hierarchy.md)
- [src/bin/tool_matrix.rs](/Users/richarddje/Documents/github/anvil/src/bin/tool_matrix.rs)

## 2026-04-27-phase4-direct-sibling-helper-route — Route direct sibling inputs from helper instances

**Landed as:** `d6ccd22`

**What changed**

- [src/gen/hierarchy.rs](/Users/richarddje/Documents/github/anvil/src/gen/hierarchy.rs)
  now lets the `hierarchy_sibling_route_prob` branch request a
  parent-cone helper instance source when
  `hierarchy_parent_cone_instance_prob` fires. The direct child-input
  route can bind from that helper output, adapting width through the
  existing parent-cone adapter path when needed.
- The old direct sibling behavior is preserved when no helper source is
  requested: the route still falls back to a dep-bearing earlier sibling
  instance output from the existing sibling pool.
- The parent-cone helper width adapter was renamed to describe its
  generic role because both unregistered and registered routes now use
  it.
- [src/metrics.rs](/Users/richarddje/Documents/github/anvil/src/metrics.rs)
  adds focused design-metrics coverage for direct sibling helper
  bindings. The existing parent-cone helper child-input metrics already
  count this route by dependency, while the registered helper counters
  stay zero for the unregistered path.
- [tests/pipeline.rs](/Users/richarddje/Documents/github/anvil/tests/pipeline.rs)
  adds `hierarchy_sibling_routes_can_use_helper_instances`, which
  forces direct sibling routing and helper placement on while forcing
  registered sibling routing, registered parent-composed routing, and
  unregistered parent-composed cones off.

**Why**

- The previous helper-instance slices covered parent-composed
  child-input cones, direct registered sibling-route D sources,
  registered parent-composed D cones, and parent-output cones. The plain
  direct sibling branch still chose only from existing planned sibling
  outputs and could not allocate a helper instance as a direct
  child-input source.
- Keeping this as an unregistered route matters: it exercises helper
  instance outputs as direct acyclic child-input sources without
  conflating the result with the registered sibling or registered
  parent-composed D-cone paths.

**Validation**

- `cargo test --test pipeline hierarchy_sibling_routes_can_use_helper_instances`
- `cargo test design_metrics_capture_direct_sibling_parent_cone_instance_routes`
- `cargo check --all-targets`
- `cargo test`
- `cargo clippy --all-targets -- -D warnings`
- `cargo fmt --all --check`
- `mdbook build book`
- `git diff --check`

**Impact**

- When both `hierarchy_sibling_route_prob` and
  `hierarchy_parent_cone_instance_prob` are active, a later child input
  can now be bound directly from a helper instance output instead of
  only from a planned earlier sibling output.
- Focused tests prove the route with registered helper counters at zero:
  `child_input_bindings_from_registered_instance_outputs = 0`,
  `child_input_bindings_from_registered_parent_cone_instances = 0`,
  `child_input_bindings_from_parent_cone_instances > 0`, and helper
  binding fractions nonzero at both hierarchy and top level, with
  helper instances present beyond the planned child slots.
- Existing registered sibling helper routing and parent-composed helper
  routing remain covered by their focused regressions.

**Files touched**

- [CHANGES.md](/Users/richarddje/Documents/github/anvil/CHANGES.md)
- [CODEBASE_ANALYSIS.md](/Users/richarddje/Documents/github/anvil/CODEBASE_ANALYSIS.md)
- [DEVELOPMENT_NOTES.md](/Users/richarddje/Documents/github/anvil/DEVELOPMENT_NOTES.md)
- [MEMORY.md](/Users/richarddje/Documents/github/anvil/MEMORY.md)
- [README.md](/Users/richarddje/Documents/github/anvil/README.md)
- [ROADMAP.md](/Users/richarddje/Documents/github/anvil/ROADMAP.md)
- [USER_GUIDE.md](/Users/richarddje/Documents/github/anvil/USER_GUIDE.md)
- [book/src/architecture.md](/Users/richarddje/Documents/github/anvil/book/src/architecture.md)
- [book/src/hierarchy.md](/Users/richarddje/Documents/github/anvil/book/src/hierarchy.md)
- [book/src/ir.md](/Users/richarddje/Documents/github/anvil/book/src/ir.md)
- [book/src/knobs.md](/Users/richarddje/Documents/github/anvil/book/src/knobs.md)
- [src/gen/hierarchy.rs](/Users/richarddje/Documents/github/anvil/src/gen/hierarchy.rs)
- [src/metrics.rs](/Users/richarddje/Documents/github/anvil/src/metrics.rs)
- [tests/pipeline.rs](/Users/richarddje/Documents/github/anvil/tests/pipeline.rs)

## 2026-04-27-phase4-registered-sibling-helper-route — Route registered sibling flops from helper instances

**Landed as:** `0e3e833`

**What changed**

- [src/gen/hierarchy.rs](/Users/richarddje/Documents/github/anvil/src/gen/hierarchy.rs)
  now lets the `hierarchy_registered_sibling_route_prob` branch request
  a parent-cone helper instance source when
  `hierarchy_parent_cone_instance_prob` fires. The registered sibling
  route can use that helper output as the parent-flop D source, adapting
  width through the existing parent-cone adapter path when needed.
- The old registered sibling behavior is preserved when no helper source
  is requested: the route still falls back to a dep-bearing sibling /
  parent-source terminal and then inserts one local parent flop.
- [src/metrics.rs](/Users/richarddje/Documents/github/anvil/src/metrics.rs)
  now counts registered parent-cone helper bindings by checking whether
  the registered flop D dependencies include a parent-cone helper
  instance output. The metric no longer requires the D node itself to be
  registered parent-composed logic, which keeps direct registered sibling
  helper routes measurable.
- [tests/pipeline.rs](/Users/richarddje/Documents/github/anvil/tests/pipeline.rs)
  adds `hierarchy_registered_sibling_routes_can_use_helper_instances`,
  which forces registered sibling routing and helper placement on while
  forcing the registered parent-composed D-cone route off.

**Why**

- The previous helper-instance slices covered unregistered
  parent-composed child-input cones, registered parent-composed D cones,
  and parent-output cones. Direct registered sibling routing still chose
  only from the existing parent source pool and could not allocate a
  helper instance for its parent-flop D source.
- The metric needed to distinguish the new direct route from the older
  registered parent-composed helper route. A helper-backed registered
  sibling route may have a helper output or adapter gate on the D side,
  not a parent-composed D-cone root.

**Validation**

- `cargo test --test pipeline hierarchy_registered`
- `cargo test design_metrics_capture`
- `cargo check --all-targets`
- `cargo test`
- `cargo clippy --all-targets -- -D warnings`
- `cargo fmt --all --check`
- `mdbook build book`
- `git diff --check`

**Impact**

- When both `hierarchy_registered_sibling_route_prob` and
  `hierarchy_parent_cone_instance_prob` are active, a later child input
  can now be bound through a parent-local flop whose D side is sourced
  directly from a helper instance output.
- Focused tests prove this is not the older registered parent-composed
  D-cone path by requiring
  `child_input_bindings_from_registered_parent_composed_logic = 0` while
  `child_input_bindings_from_registered_parent_cone_instances > 0`,
  `registered_parent_cone_instance_child_input_binding_fraction > 0.0`,
  and helper instances are present beyond the planned child slots.
- Existing registered parent-composed helper routing and registered
  sibling routing remain covered by the focused hierarchy regression
  filters.

**Files touched**

- [CHANGES.md](/Users/richarddje/Documents/github/anvil/CHANGES.md)
- [CODEBASE_ANALYSIS.md](/Users/richarddje/Documents/github/anvil/CODEBASE_ANALYSIS.md)
- [DEVELOPMENT_NOTES.md](/Users/richarddje/Documents/github/anvil/DEVELOPMENT_NOTES.md)
- [MEMORY.md](/Users/richarddje/Documents/github/anvil/MEMORY.md)
- [README.md](/Users/richarddje/Documents/github/anvil/README.md)
- [ROADMAP.md](/Users/richarddje/Documents/github/anvil/ROADMAP.md)
- [USER_GUIDE.md](/Users/richarddje/Documents/github/anvil/USER_GUIDE.md)
- [book/src/architecture.md](/Users/richarddje/Documents/github/anvil/book/src/architecture.md)
- [book/src/hierarchy.md](/Users/richarddje/Documents/github/anvil/book/src/hierarchy.md)
- [book/src/knobs.md](/Users/richarddje/Documents/github/anvil/book/src/knobs.md)
- [src/gen/hierarchy.rs](/Users/richarddje/Documents/github/anvil/src/gen/hierarchy.rs)
- [src/metrics.rs](/Users/richarddje/Documents/github/anvil/src/metrics.rs)
- [tests/pipeline.rs](/Users/richarddje/Documents/github/anvil/tests/pipeline.rs)

## 2026-04-27-phase4-parent-output-helper-budget — Spend parent-output helper budget directly
**Landed as:** `c348884`

**What changed**

- [src/gen/hierarchy.rs](/Users/richarddje/Documents/github/anvil/src/gen/hierarchy.rs)
  now pre-allocates a vector of parent-cone helper instance outputs for
  parent-output composition instead of choosing one optional helper
  source for the whole parent-output set. Parent-output promotion picks
  a helper source per output, cycling through the collected sources
  when there are more outputs than helpers.
- Parent-output helper collection now uses the existing
  `max_parent_cone_instances_per_module` budget and can spend multiple
  helper instances in the parent-output-only path.
- Helper instances allocated for parent-output composition bind their
  own child inputs from non-helper parent sources, so the focused proof
  does not accidentally satisfy itself through helper-to-helper
  child-input routing.
- [tests/pipeline.rs](/Users/richarddje/Documents/github/anvil/tests/pipeline.rs)
  adds `hierarchy_parent_outputs_can_spend_helper_budget`, which forces
  `hierarchy_child_input_cone_prob = 0.0`,
  `hierarchy_registered_child_input_cone_prob = 0.0`,
  `hierarchy_parent_cone_instance_prob = 1.0`, and
  `max_parent_cone_instances_per_module = 3` across all construction
  strategies.

**Why**

- The previous parent-output helper route proved that a parent output
  could depend on a helper instance output, but it chose one helper
  source before building all parent outputs.
- The separate helper-budget proof spent multiple helpers through
  child-input routing, not through the parent-output-only route. That
  left a gap: raising the helper budget did not directly prove that
  parent-output composition itself could allocate and use more than one
  helper instance.

**Validation**

- `cargo test --test pipeline hierarchy_`
- `cargo check --all-targets`
- `cargo test`
- `cargo clippy --all-targets -- -D warnings`
- `cargo fmt --all --check`
- `mdbook build book`
- `git diff --check`

**Impact**

- A parent-output-only helper config can now spend the per-parent helper
  budget without relying on child-input helper bindings. The focused
  regression proves `top_parent_cone_instances = 3`,
  `max_parent_cone_instances_per_internal_module = 3`,
  `child_input_bindings_from_parent_cone_instances = 0`, and
  parent outputs reaching helper outputs.
- Existing helper routes remain covered by the hierarchy regression
  sweep: unregistered child-input helpers, budgeted child-input
  helpers, registered helper-sourced child-input D cones, and the
  original parent-output helper route.

**Files touched**

- [CHANGES.md](/Users/richarddje/Documents/github/anvil/CHANGES.md)
- [CODEBASE_ANALYSIS.md](/Users/richarddje/Documents/github/anvil/CODEBASE_ANALYSIS.md)
- [DEVELOPMENT_NOTES.md](/Users/richarddje/Documents/github/anvil/DEVELOPMENT_NOTES.md)
- [MEMORY.md](/Users/richarddje/Documents/github/anvil/MEMORY.md)
- [README.md](/Users/richarddje/Documents/github/anvil/README.md)
- [ROADMAP.md](/Users/richarddje/Documents/github/anvil/ROADMAP.md)
- [USER_GUIDE.md](/Users/richarddje/Documents/github/anvil/USER_GUIDE.md)
- [book/src/architecture.md](/Users/richarddje/Documents/github/anvil/book/src/architecture.md)
- [book/src/hierarchy.md](/Users/richarddje/Documents/github/anvil/book/src/hierarchy.md)
- [src/gen/hierarchy.rs](/Users/richarddje/Documents/github/anvil/src/gen/hierarchy.rs)
- [tests/pipeline.rs](/Users/richarddje/Documents/github/anvil/tests/pipeline.rs)
## 2026-04-27-package-description-terminology — Align package metadata with ANVIL terminology

**Landed as:** `785a143`

**What changed**

- [Cargo.toml](/Users/richarddje/Documents/github/anvil/Cargo.toml)
  now describes ANVIL as a random by-construction generator of
  synthesizable SystemVerilog RTL instead of using the stale
  `constrained-random` package description.
- [CODEBASE_ANALYSIS.md](/Users/richarddje/Documents/github/anvil/CODEBASE_ANALYSIS.md),
  [DEVELOPMENT_NOTES.md](/Users/richarddje/Documents/github/anvil/DEVELOPMENT_NOTES.md),
  and [MEMORY.md](/Users/richarddje/Documents/github/anvil/MEMORY.md)
  now record that package metadata is part of the terminology surface,
  not just README/book prose.

**Why**

- The earlier purpose-terminology cleanup correctly removed
  constrained-random wording from the user-facing docs and Rustdoc, but
  the package description in `Cargo.toml` still contradicted the accepted
  project wording.
- Keeping package metadata aligned matters because it is visible to
  Cargo tooling, generated package metadata, and cold-start readers who
  inspect the crate before opening the full docs.

**Validation**

- `cargo check --all-targets`
- `cargo test`
- `cargo clippy --all-targets -- -D warnings`
- `cargo fmt --all --check`
- `mdbook build book`
- `git diff --check`
- Searched active repo text for remaining non-historical
  `constrained-random` package-purpose wording.

**Impact**

- The crate metadata now matches the accepted ANVIL purpose wording:
  random by-construction synthesizable SystemVerilog RTL generation.
- No generator behavior changed.

**Files touched**

- [Cargo.toml](/Users/richarddje/Documents/github/anvil/Cargo.toml)
- [CHANGES.md](/Users/richarddje/Documents/github/anvil/CHANGES.md)
- [CODEBASE_ANALYSIS.md](/Users/richarddje/Documents/github/anvil/CODEBASE_ANALYSIS.md)
- [DEVELOPMENT_NOTES.md](/Users/richarddje/Documents/github/anvil/DEVELOPMENT_NOTES.md)
- [MEMORY.md](/Users/richarddje/Documents/github/anvil/MEMORY.md)
## 2026-04-27-phase4-hierarchy-gate-budget — Refresh Phase 4 hierarchy gate at full per-scenario depth
**Landed as:** `f9f0288`

**What changed**

- [src/bin/tool_matrix.rs](/Users/richarddje/Documents/github/anvil/src/bin/tool_matrix.rs)
  now treats the Phase 4 hierarchy gate budget as a per-scenario floor:
  `PHASE4_HIERARCHY_MIN_DESIGNS_PER_SCENARIO = 4`. Adding new hierarchy
  scenarios can no longer silently reduce each scenario from four
  designs to three by dividing a stale total-design floor across the
  larger matrix.
- The Phase 4 run-plan regression now builds the live Phase 4 hierarchy
  scenario set and asserts `4` designs/scenario and `168` total designs
  for the current `42`-scenario matrix.
- The corrected full downstream-clean bank is
  `/tmp/anvil-tool-matrix-phase4-hierarchy-r23/tool_matrix_report.json`:
  `42` scenarios, `4` designs/scenario, `168` total designs,
  `artifact_kind = "design"`, `coverage_gaps = []`, and `168/0`
  pass-fail in Verilator plus both repo-owned Yosys modes.
- The live docs and mdBook now present `r23` as the latest fully banked
  Phase 4 hierarchy closure artifact, keep `r21` as historical
  pre-parent-output-helper evidence, and record the pre-fix `r22` run
  as a clean but insufficient 126-design mismatch.

**Why**

- The current Phase 4 hierarchy matrix grew to `42` scenarios after the
  parent-output helper, budgeted-helper, and registered helper-sourced
  child-input axes were added. The old constant
  `PHASE4_HIERARCHY_MIN_TOTAL_DESIGNS = 120` made the refreshed gate run
  only `3` designs/scenario (`126` total designs), even though the
  project policy was to preserve four designs per representative
  hierarchy scenario.
- The corrected rule encodes that policy directly as a per-scenario
  floor, so future scenario-count growth cannot accidentally weaken
  each Phase 4 scenario's sampling depth.

**Validation**

- `cargo test --bin tool_matrix phase4_hierarchy`
- `cargo run --quiet --bin tool_matrix -- --out /tmp/anvil-tool-matrix-phase4-hierarchy-r23 --phase4-hierarchy-gate --yosys-mode both`
- `cargo check --all-targets`
- `cargo test`
- `cargo clippy --all-targets -- -D warnings`
- `cargo fmt --all --check`
- `mdbook build book`
- `git diff --check`

**Impact**

- The repo-owned Phase 4 hierarchy gate now proves the full current
  helper-instance hierarchy surface without lowering evidence density as
  the matrix grows.
- Cold-start recovery docs point to the latest `r23` bank for current
  downstream-clean evidence and to `r22` only for the root-cause audit.
- No generator output semantics changed; this slice changes the matrix
  gate's run-plan arithmetic plus the documentation/evidence state.

**Files touched**

- [CHANGES.md](/Users/richarddje/Documents/github/anvil/CHANGES.md)
- [CODEBASE_ANALYSIS.md](/Users/richarddje/Documents/github/anvil/CODEBASE_ANALYSIS.md)
- [DEVELOPMENT_NOTES.md](/Users/richarddje/Documents/github/anvil/DEVELOPMENT_NOTES.md)
- [MEMORY.md](/Users/richarddje/Documents/github/anvil/MEMORY.md)
- [README.md](/Users/richarddje/Documents/github/anvil/README.md)
- [ROADMAP.md](/Users/richarddje/Documents/github/anvil/ROADMAP.md)
- [USER_GUIDE.md](/Users/richarddje/Documents/github/anvil/USER_GUIDE.md)
- [book/src/architecture.md](/Users/richarddje/Documents/github/anvil/book/src/architecture.md)
- [book/src/hierarchy.md](/Users/richarddje/Documents/github/anvil/book/src/hierarchy.md)
- [src/bin/tool_matrix.rs](/Users/richarddje/Documents/github/anvil/src/bin/tool_matrix.rs)

## 2026-04-26-doc-continuity-and-purpose — Align docs with live source and ANVIL terminology

**Landed as:** `34a420e`

**What changed**

- [src/metrics.rs](/Users/richarddje/Documents/github/anvil/src/metrics.rs)
  now describes live probability-roll telemetry as sourced from
  `Module::knob_rolls` and surfaced through `knob_roll_attempts` /
  `knob_roll_fires`, rather than calling knob-attempt signals a future
  extension.
- [CODEBASE_ANALYSIS.md](/Users/richarddje/Documents/github/anvil/CODEBASE_ANALYSIS.md)
  now records the hierarchy probability knobs that flow into
  `m.knob_rolls`, corrects the current per-file unit-test counts for
  `src/gen/module.rs` and `src/metrics.rs`, and updates the current
  `cargo test` evidence date.
- [book/src/architecture.md](/Users/richarddje/Documents/github/anvil/book/src/architecture.md)
  now includes the live `ConstantProb`, `TerminalReuseProb`, and
  `HierarchyParentConeInstanceProb` variants in its `KnobId` overview
  and reflects the current `src/metrics.rs` test count.
- [README.md](/Users/richarddje/Documents/github/anvil/README.md),
  [book/src/introduction.md](/Users/richarddje/Documents/github/anvil/book/src/introduction.md),
  [book/src/faq.md](/Users/richarddje/Documents/github/anvil/book/src/faq.md),
  and [src/lib.rs](/Users/richarddje/Documents/github/anvil/src/lib.rs)
  now describe ANVIL as a random by-construction generator of
  synthesizable SystemVerilog RTL rather than as constrained-random
  generation.
- [ROADMAP.md](/Users/richarddje/Documents/github/anvil/ROADMAP.md),
  [CODEBASE_ANALYSIS.md](/Users/richarddje/Documents/github/anvil/CODEBASE_ANALYSIS.md),
  [USER_GUIDE.md](/Users/richarddje/Documents/github/anvil/USER_GUIDE.md),
  [DEVELOPMENT_NOTES.md](/Users/richarddje/Documents/github/anvil/DEVELOPMENT_NOTES.md),
  [MEMORY.md](/Users/richarddje/Documents/github/anvil/MEMORY.md), and
  mdBook purpose/non-goal chapters now state the distinction between
  ANVIL's target outputs and its validation tools: Verilator and Yosys
  check generated HDL acceptance, while the generated artifacts target
  the broader set of downstream HDL parsers, elaborators, RTL
  compilers, linters, simulators, synthesis tools, and related
  consumers.
- The docs now say ANVIL-generated legal HDL corpora can be used to
  stress downstream tools, without describing ANVIL itself as primarily
  a malformed-input fuzzer or generic toolchain stress tester.

**Why**

- The README/session bootstrap deep dive found small factual drift
  between the live source and the authority documentation. The edits
  keep the source walk, test-count evidence, and knob telemetry
  description aligned with current HEAD.
- The previous wording blurred two separate ideas: ANVIL generates legal
  synthesizable HDL artifacts, and those artifacts can be used to
  exercise downstream tools. The user clarified that Verilator/Yosys are
  validation tools in the repo, not the project target itself, and that
  "constrained-random" is misleading without user-authored constraints.

**Validation**

- `cargo check --all-targets`
- `cargo test`
- `cargo clippy --all-targets -- -D warnings`
- `cargo fmt --all --check`
- `mdbook build book`
- `git diff --check`

**Impact**

- Recovery documentation now represents the completed source-bootstrap
  and terminology work as one truthful commit, with the previous
  hierarchy slice anchored to its real commit hash.
- Cold-start readers get current wording for ANVIL's purpose,
  downstream-tool target class, validation-tool role, knob telemetry,
  and test-count/source-state notes before the next Phase 4 lane
  continues.
- No generator behavior changed; this is a documentation, Rustdoc, and
  source-comment alignment slice.

**Files touched**

- [CHANGES.md](/Users/richarddje/Documents/github/anvil/CHANGES.md)
- [CODEBASE_ANALYSIS.md](/Users/richarddje/Documents/github/anvil/CODEBASE_ANALYSIS.md)
- [DEVELOPMENT_NOTES.md](/Users/richarddje/Documents/github/anvil/DEVELOPMENT_NOTES.md)
- [MEMORY.md](/Users/richarddje/Documents/github/anvil/MEMORY.md)
- [README.md](/Users/richarddje/Documents/github/anvil/README.md)
- [ROADMAP.md](/Users/richarddje/Documents/github/anvil/ROADMAP.md)
- [USER_GUIDE.md](/Users/richarddje/Documents/github/anvil/USER_GUIDE.md)
- [book/src/architecture.md](/Users/richarddje/Documents/github/anvil/book/src/architecture.md)
- [book/src/core-idea.md](/Users/richarddje/Documents/github/anvil/book/src/core-idea.md)
- [book/src/faq.md](/Users/richarddje/Documents/github/anvil/book/src/faq.md)
- [book/src/introduction.md](/Users/richarddje/Documents/github/anvil/book/src/introduction.md)
- [book/src/non-goals.md](/Users/richarddje/Documents/github/anvil/book/src/non-goals.md)
- [src/lib.rs](/Users/richarddje/Documents/github/anvil/src/lib.rs)
- [src/metrics.rs](/Users/richarddje/Documents/github/anvil/src/metrics.rs)

## 2026-04-24-boot17 — Route registered child-input cones through helper instances
**Landed as:** `1f8364e`

**What changed**

- [src/gen/hierarchy.rs](/Users/richarddje/Documents/github/anvil/src/gen/hierarchy.rs)
  now lets registered parent-composed child-input D cones request a
  parent-cone helper instance when
  `hierarchy_registered_child_input_cone_prob` and
  `hierarchy_parent_cone_instance_prob` are both active.
- [src/metrics.rs](/Users/richarddje/Documents/github/anvil/src/metrics.rs)
  now reports registered helper-sourced child-input bindings through
  `child_input_bindings_from_registered_parent_cone_instances`,
  `top_child_input_bindings_from_registered_parent_cone_instances`,
  `registered_parent_cone_instance_child_input_binding_fraction`, and
  `top_registered_parent_cone_instance_child_input_binding_fraction`.
- [tests/pipeline.rs](/Users/richarddje/Documents/github/anvil/tests/pipeline.rs)
  adds `hierarchy_registered_child_input_cones_can_use_helper_instances`,
  proving the route across all construction strategies.
- [src/bin/tool_matrix.rs](/Users/richarddje/Documents/github/anvil/src/bin/tool_matrix.rs)
  adds the Phase 4
  `phase4_hier2_inst4_registered_parent_cone_instance_state` scenario
  and a coverage gap for registered parent-composed child inputs sourced
  from parent-cone helper instances. The next planned Phase 4 bank is
  now 42 scenarios / 168 designs.
- The mdBook and live docs now describe the combined knob behavior,
  new metrics, focused proof, recipe, and updated matrix-plan counts.

**Why**

- The helper-instance surface had reached unregistered child-input
  cones, parent-output cones, and budgeting. This broadens helper
  placement into the registered parent-composed path while keeping the
  route opt-in and measurable.

**Validation**

- `cargo test hierarchy_registered_child_input_cones_can_use_helper_instances`
- `cargo test design_metrics_capture_registered_parent_cone_instance_routes`
- `cargo test --bin tool_matrix phase4_hierarchy_matrix_covers_wrapper_and_recursive_profiles`
- `cargo test --bin tool_matrix phase4_hierarchy_coverage_requires_design_facts`
- `cargo fmt --all --check`
- `cargo check --all-targets`
- `cargo test`
- `cargo clippy --all-targets -- -D warnings`
- `mdbook build book`
- `git diff --check`

## 2026-04-24-boot16 — Budget parent-cone helper instances per parent

**Landed as:** this commit

**What changed**

- [src/config.rs](/Users/richarddje/Documents/github/anvil/src/config.rs)
  and [src/main.rs](/Users/richarddje/Documents/github/anvil/src/main.rs)
  add `max_parent_cone_instances_per_module`, exposed on the CLI as
  `--max-parent-cone-instances-per-module`. The default is `1`, so old
  configs keep the first helper-instance behavior; `0` disables helper
  allocation; higher values let one hierarchy parent instantiate
  multiple helper children when
  `hierarchy_parent_cone_instance_prob` fires.
- [src/gen/hierarchy.rs](/Users/richarddje/Documents/github/anvil/src/gen/hierarchy.rs)
  replaces the old per-parent boolean helper guard with a per-parent
  helper count checked against the configured budget.
- [src/metrics.rs](/Users/richarddje/Documents/github/anvil/src/metrics.rs)
  now reports `max_parent_cone_instances_per_internal_module`, so
  manifests and matrix reports can prove that a local helper budget was
  actually used.
- [tests/pipeline.rs](/Users/richarddje/Documents/github/anvil/tests/pipeline.rs)
  adds `hierarchy_parent_cone_helper_budget_allows_multiple_helpers`,
  proving a budget of `3` across all construction strategies.
- [src/bin/tool_matrix.rs](/Users/richarddje/Documents/github/anvil/src/bin/tool_matrix.rs)
  adds the Phase 4
  `phase4_hier2_inst4_parent_cone_instance_budget3` scenario and a
  coverage gap for multiple parent-cone helper instances in one
  hierarchy parent. At this commit's point in history, the next planned
  Phase 4 bank grew; the current plan is tracked by the newest
  changelog entry.
- The mdBook and live docs now describe the new knob, metrics, recipe,
  focused proof, and updated matrix-plan counts.

**Why**

- The previous helper-instance slice was intentionally limited to one
  helper per parent. This lands the first explicit budgeting control
  while keeping the default behavior stable.

**Validation**

- `cargo test hierarchy_parent_cone_helper_budget_allows_multiple_helpers`
- `cargo test design_metrics_capture_multiple_parent_cone_instance_budget`
- `cargo test --bin tool_matrix phase4_hierarchy_matrix_covers_wrapper_and_recursive_profiles`
- `cargo test newly_exposed_cli_knobs_round_trip_into_overrides`
- `cargo fmt --all --check`
- `cargo check --all-targets`
- `cargo test`
- `cargo clippy --all-targets -- -D warnings`
- `mdbook build book`
- `git diff --check`

## 2026-04-24-boot15 — Route parent-output cones through helper instances

**Landed as:** this commit

**What changed**

- [src/gen/hierarchy.rs](/Users/richarddje/Documents/github/anvil/src/gen/hierarchy.rs)
  now lets `hierarchy_parent_cone_instance_prob` instantiate one
  parent-cone helper child for parent-output composition, not only for
  parent-composed child-input bindings.
- [src/metrics.rs](/Users/richarddje/Documents/github/anvil/src/metrics.rs)
  now reports helper-instance support on parent outputs through
  `top_outputs_reaching_parent_cone_instances`,
  `hierarchy_outputs_reaching_parent_cone_instances`,
  `top_parent_cone_instance_output_fraction`, and
  `hierarchy_parent_cone_instance_output_fraction`.
- [src/bin/tool_matrix.rs](/Users/richarddje/Documents/github/anvil/src/bin/tool_matrix.rs)
  adds a dedicated Phase 4 scenario,
  `phase4_hier2_inst4_parent_output_cone_instance`, and a coverage gap
  for parent outputs sourced from parent-cone helper instances. The
  then-current scenario plan was 36 scenarios / 144 designs; the latest
  full downstream-clean bank remained the earlier `r21` report.
- [tests/pipeline.rs](/Users/richarddje/Documents/github/anvil/tests/pipeline.rs)
  proves the route across all construction strategies while keeping
  child-input helper bindings off, so the new surface is independent of
  the earlier child-input helper slice.
- The mdBook and live docs now describe the new helper-output metrics,
  recipe, and roadmap status.

**Why**

- Phase 4's next open item was broader helper-instance placement beyond
  child-input cones. Parent-output composition is the smallest useful
  widening: it reuses the existing helper-instance machinery while
  proving module instantiation can participate in another parent-side
  cone surface.

**Validation**

- `cargo test hierarchy_parent_outputs_can_depend_on_helper_instance_outputs`
- `cargo test design_metrics_capture_parent_cone_instance_output_support`
- `cargo test --bin tool_matrix`
- `cargo fmt --all --check`
- `cargo check --all-targets`
- `cargo test`
- `cargo clippy --all-targets -- -D warnings`
- `mdbook build book`
- `git diff --check`

## 2026-04-24-boot14 — Align live docs with current hierarchy state

**Landed as:** this commit

**What changed**

- [USER_GUIDE.md](/Users/richarddje/Documents/github/anvil/USER_GUIDE.md)
  now points the Phase 4 closure evidence at the current `r21` bank
  with 33 scenarios / 132 designs and 132/0 pass-fail in Verilator plus
  both repo-owned Yosys modes.
- [CODEBASE_ANALYSIS.md](/Users/richarddje/Documents/github/anvil/CODEBASE_ANALYSIS.md)
  and [ROADMAP.md](/Users/richarddje/Documents/github/anvil/ROADMAP.md)
  now describe hierarchy as a landed Phase 4 surface while
  distinguishing it from still-open parameterization, aggregate,
  memory, FSM, artifact-family, and hierarchy-aware identity work.
- [README.md](/Users/richarddje/Documents/github/anvil/README.md)
  now summarizes the hierarchy generator path with the current child
  sourcing, parent-composition, registered-routing, helper-instance,
  and parent-state surfaces.
- [book/src/ir.md](/Users/richarddje/Documents/github/anvil/book/src/ir.md),
  [book/src/hierarchy.md](/Users/richarddje/Documents/github/anvil/book/src/hierarchy.md),
  and [book/src/architecture.md](/Users/richarddje/Documents/github/anvil/book/src/architecture.md)
  now avoid current-wrapper-only wording and describe hierarchy as a
  live planner above the leaf kernel.
- [CODEBASE_ANALYSIS.md](/Users/richarddje/Documents/github/anvil/CODEBASE_ANALYSIS.md)
  also recorded the then-current executed test count: 262 passing
  tests.

**Why**

- The book and live docs are ANVIL's user-facing surface. Advanced
  users and developers should not have to infer from source code that
  the current hierarchy implementation is broader than the older
  wrapper-only slice.

**Validation**

- Searched active docs for stale hierarchy closure artifacts and
  wrapper-only/pre-Phase-4 wording.
- Compared `anvil --help` and `tool_matrix --help` flags against
  `book/src`; no help flags are missing from the mdBook.
- `cargo fmt --all --check`
- `cargo check --all-targets`
- `cargo test`
- `cargo clippy --all-targets -- -D warnings`
- `mdbook build book`
- `git diff --check`

## 2026-04-24-boot13 — Audit mdBook CLI and hierarchy recipes

**Landed as:** this commit

**What changed**

- [book/src/knobs.md](/Users/richarddje/Documents/github/anvil/book/src/knobs.md)
  now lists every current `anvil --help` flag and every current
  `tool_matrix --help` flag in the CLI coverage section.
- [book/src/recipes.md](/Users/richarddje/Documents/github/anvil/book/src/recipes.md)
  now has hierarchy recipes for depth-1 multi-file output,
  child-definition reuse, under-instantiated libraries, bounded
  recursive hierarchy, per-depth branching overrides, on-demand child
  synthesis, sibling routes, parent-composed child-input cones,
  parent-cone helper instances, and registered hierarchy routes.
- [book/src/hierarchy.md](/Users/richarddje/Documents/github/anvil/book/src/hierarchy.md)
  now lists the parent-cone helper-instance counters with the other
  design metrics exposed through hierarchy manifests.
- [src/main.rs](/Users/richarddje/Documents/github/anvil/src/main.rs)
  fixes the `--max-child-instances-per-module` help text so it points
  users at the required minimum-pair flag.

**Why**

- The mdBook is the user-facing surface. The hierarchy implementation
  had outpaced the reference and cookbook examples in a few places, so
  casual users could discover the feature in README but miss it in the
  book.

**Validation**

- Compared `anvil --help` and `tool_matrix --help` flags against
  `book/src`; no help flags are missing from the mdBook.
- `cargo fmt --all --check`
- `cargo check --all-targets`
- `cargo test`
- `cargo clippy --all-targets -- -D warnings`
- `mdbook build book`
- `git diff --check`

## 2026-04-24-boot12 — Centralize hierarchy module-name allocation

**Landed as:** this commit

**What changed**

- [src/gen/mod.rs](/Users/richarddje/Documents/github/anvil/src/gen/mod.rs)
  now owns module-index reservation through `reserve_module_index()`
  and module-name formatting through `module_name()`.
- [src/gen/module.rs](/Users/richarddje/Documents/github/anvil/src/gen/module.rs)
  and [src/gen/hierarchy.rs](/Users/richarddje/Documents/github/anvil/src/gen/hierarchy.rs)
  now use that allocator for leaf modules and hierarchy parent modules
  instead of hand-rolling parent-name reservation.
- [tests/pipeline.rs](/Users/richarddje/Documents/github/anvil/tests/pipeline.rs)
  now proves recursive/helper-heavy hierarchy generation keeps module
  names unique both within each design and across multiple hierarchical
  designs emitted by one generator run.
- README, roadmap, memory, development notes, codebase analysis, and the
  mdBook hierarchy/structural-rules chapters were updated to reflect
  that module-name uniqueness is now an explicit Phase 4 construction
  invariant.

**Why**

- Directory hierarchy output writes one `.sv` file per module
  definition. Name reuse across generated designs would silently
  overwrite files or make downstream tool inputs ambiguous. Module
  naming is now treated as a generator-global resource rather than an
  incidental formatting detail.

**Validation**

- `cargo fmt --all --check`
- `cargo check --all-targets`
- `cargo test --test pipeline hierarchy_module_names_are_unique_across_batch_generation`
- `cargo test`
- `cargo clippy --all-targets -- -D warnings`
- `mdbook build book`
- `git diff --check`

## 2026-04-24-boot11 — Land parent-cone helper hierarchy instances

**Landed as:** this commit

**What changed**

- [src/gen/hierarchy.rs](/Users/richarddje/Documents/github/anvil/src/gen/hierarchy.rs)
  now lets parent-composed child-input cones instantiate one helper
  child as an internal parent-cone source under
  `hierarchy_parent_cone_instance_prob`.
- [src/ir/types.rs](/Users/richarddje/Documents/github/anvil/src/ir/types.rs)
  now distinguishes planned child instances from parent-cone helper
  instances through `InstanceRole`, and dependency sets can expose
  referenced instance-output virtuals directly.
- [src/config.rs](/Users/richarddje/Documents/github/anvil/src/config.rs)
  and [src/main.rs](/Users/richarddje/Documents/github/anvil/src/main.rs)
  expose the new config/CLI knob with a default of `0.0`.
- [src/metrics.rs](/Users/richarddje/Documents/github/anvil/src/metrics.rs)
  now reports parent-cone helper instances and child-input bindings
  sourced through them with top-level and hierarchy-wide counters plus
  fractions.
- [src/bin/tool_matrix.rs](/Users/richarddje/Documents/github/anvil/src/bin/tool_matrix.rs)
  now includes a Phase 4 parent-cone helper-instance scenario per
  strategy and rejects Phase 4 matrices that never prove the route.
- [tests/pipeline.rs](/Users/richarddje/Documents/github/anvil/tests/pipeline.rs)
  now has a strategy sweep proving helper child instantiation can feed
  parent-composed child-input bindings.
- README, roadmap, development notes, memory, codebase analysis, and
  the mdBook hierarchy/architecture/knobs chapters were refreshed to
  make the new user-facing behavior and metrics discoverable.

**Why**

- Phase 4 already had parent-side logic around child instances, but
  module instantiation itself was still only a wrapper/planning action.
  This lands the first narrow slice where a parent cone may choose a
  module instance as an internal source, while keeping the behavior
  measurable and opt-in.

**Validation**

- `cargo fmt --all --check`
- `cargo check --all-targets`
- `cargo test --test pipeline hierarchy_child_input_cones_can_instantiate_helper_children`
- `cargo test --bin tool_matrix phase4_hierarchy_coverage_requires_design_facts`
- `cargo test --bin tool_matrix phase4_hierarchy_matrix_covers_wrapper_and_recursive_profiles`
- `cargo test`
- `cargo clippy --all-targets -- -D warnings`
- Focused smoke:
  `/tmp/anvil-parent-cone-instance-smoke-r1/manifest.json` records
  `top_parent_cone_instances = 1`,
  `hierarchy_parent_cone_instances = 1`,
  `child_input_bindings_from_parent_cone_instances = 4`, and
  `top_child_input_bindings_from_parent_cone_instances = 4`.
- The focused smoke is clean in Verilator, Yosys `synth -noabc`, and
  the repo-owned Yosys with-ABC path.
- Full downstream-clean Phase 4 matrix:
  `/tmp/anvil-tool-matrix-phase4-hierarchy-r21/tool_matrix_report.json`
  records `33` scenarios, `132` total designs, `coverage_gaps = []`,
  `Verilator pass/fail = 132/0`,
  `Yosys without-abc pass/fail = 132/0`, and
  `Yosys with-abc pass/fail = 132/0`.
- The same report records
  `saw_hierarchy_parent_cone_instance_routing = true`.
- `mdbook build book`
- `git diff --check`

## 2026-04-24-boot10 — Refresh Phase 4 hierarchy gate for multi-stage routing

**Landed as:** this commit

**What changed**

- The full repo-owned Phase 4 hierarchy matrix was rerun at
  `/tmp/anvil-tool-matrix-phase4-hierarchy-r20/tool_matrix_report.json`,
  promoting the current downstream-clean bank past the older `r19`
  artifact.
- README, roadmap, development notes, memory, codebase analysis, and the
  mdBook hierarchy/architecture chapters now point at `r20` as the
  authoritative current hierarchy proof.
- The docs now distinguish the older coverage-only parent-port,
  registered mixed-support, and registered multi-stage probes as focused
  policy breadcrumbs; `r20` now carries those same facts with Verilator
  and both repo-owned Yosys modes enabled.

**Why**

- The multi-stage registered routing slice had a focused downstream
  smoke and a coverage-only matrix proof, but the user-facing docs should
  cite the strongest current artifact. This keeps the book and live docs
  aligned with the actual full Phase 4 gate.

**Validation**

- `cargo run --quiet --bin tool_matrix -- --phase4-hierarchy-gate --yosys-mode both --out /tmp/anvil-tool-matrix-phase4-hierarchy-r20`
- `/tmp/anvil-tool-matrix-phase4-hierarchy-r20/tool_matrix_report.json`
  records `coverage_gaps = []`, `120` total designs,
  `Verilator pass/fail = 120/0`,
  `Yosys without-abc pass/fail = 120/0`, and
  `Yosys with-abc pass/fail = 120/0`.
- The same report records
  `saw_hierarchy_parent_port_composed_outputs = true`,
  `saw_hierarchy_registered_mixed_support_routing = true`, and
  `saw_hierarchy_registered_multistage_routing = true`.
- `mdbook build book`
- `git diff --check`

## 2026-04-24-boot9 — Land multi-stage registered hierarchy routing

**Landed as:** this commit

**What changed**

- [src/gen/hierarchy.rs](/Users/richarddje/Documents/github/anvil/src/gen/hierarchy.rs)
  now promotes registered parent-composed child-input D cones with an
  earlier parent-flop companion when one is available, so later
  registered routes can form a real multi-stage parent-local chain.
- [src/metrics.rs](/Users/richarddje/Documents/github/anvil/src/metrics.rs)
  now reports multi-stage registered parent-composed child-input
  bindings through
  `child_input_bindings_from_registered_multistage_parent_composed_logic`,
  the matching top-level counter, and matching hierarchy/top fractions.
- [src/bin/tool_matrix.rs](/Users/richarddje/Documents/github/anvil/src/bin/tool_matrix.rs)
  now tracks `saw_hierarchy_registered_multistage_routing`, and the
  Phase 4 hierarchy coverage gate rejects representative matrices that
  never prove the multi-stage registered route.
- CLI/config/Rustdoc wording and live project docs were refreshed so
  `hierarchy_registered_child_input_cone_prob` describes full parent
  source support, mixed parent-port / child-output support, and
  earlier parent-flop chaining.

**Why**

- The previous registered hierarchy slices proved one-flop registered
  sibling routing and one-flop registered parent-composed routing. The
  next small hierarchy-deepening step was to let the parent-composed
  route reuse earlier parent-local Qs as part of the next D cone,
  creating a measured multi-stage registered pattern without adding a
  new knob.

**Validation**

- `cargo fmt --all --check`
- `cargo check --all-targets`
- `cargo test --test pipeline hierarchy_child_inputs_can_be_registered_from_parent_composed_logic`
- `cargo test --bin tool_matrix phase4_hierarchy_coverage_requires_design_facts`
- Focused smoke:
  `/tmp/anvil-hier-registered-multistage-child-input-smoke-r1/manifest.json`
  records
  `child_input_bindings_from_registered_multistage_parent_composed_logic = 2`,
  `top_child_input_bindings_from_registered_multistage_parent_composed_logic = 2`,
  and
  `registered_multistage_parent_composed_child_input_binding_fraction = 0.5`.
- The focused smoke is clean in Verilator, Yosys `synth -noabc`, and
  the repo-owned Yosys with-ABC path.
- Coverage-only Phase 4 matrix:
  `/tmp/anvil-tool-matrix-phase4-registered-multistage-r1/tool_matrix_report.json`
  records `coverage_gaps = []` and
  `saw_hierarchy_registered_multistage_routing = true`.
- `cargo test`
- `cargo clippy --all-targets -- -D warnings`
- `mdbook build book`
- `git diff --check`

## 2026-04-24-boot8 — Strengthen hierarchy book guidance

**Landed as:** this commit

**What changed**

- [book/src/hierarchy.md](/Users/richarddje/Documents/github/anvil/book/src/hierarchy.md)
  now has a reader-facing hierarchy routing section that maps common
  goals to the right hierarchy knobs, produced structure, and metrics
  to inspect.
- [book/src/knobs.md](/Users/richarddje/Documents/github/anvil/book/src/knobs.md)
  now describes `hierarchy_registered_child_input_cone_prob` as a full
  parent-source route that can mix parent data inputs with sibling
  outputs when both supports are live.
- [book/src/architecture.md](/Users/richarddje/Documents/github/anvil/book/src/architecture.md)
  now lists the focused mixed parent-output and registered
  mixed-support hierarchy proof artifacts with the rest of the targeted
  Phase 4 evidence.

**Why**

- The book is the user-facing surface for ANVIL. The previous code
  slice updated the book, but the hierarchy chapter still needed a
  clearer "which knob should I use?" section for casual users and a
  tighter metric map for advanced users and developers.

**Validation**

- `mdbook build book`
- `git diff --check`

## 2026-04-24-boot7 — Land registered mixed-support hierarchy routing

**Landed as:** this commit

**What changed**

- [src/gen/hierarchy.rs](/Users/richarddje/Documents/github/anvil/src/gen/hierarchy.rs)
  now builds registered parent-composed child-input D cones from the
  full parent source pool, then repairs them so they keep sibling
  child-output support and add parent-port support when live parent
  data inputs are available.
- [src/metrics.rs](/Users/richarddje/Documents/github/anvil/src/metrics.rs)
  now reports registered mixed-support child-input bindings:
  `child_input_bindings_from_registered_mixed_support`,
  `top_child_input_bindings_from_registered_mixed_support`,
  `registered_mixed_support_child_input_binding_fraction`, and
  `top_registered_mixed_support_child_input_binding_fraction`.
- [src/bin/tool_matrix.rs](/Users/richarddje/Documents/github/anvil/src/bin/tool_matrix.rs)
  now tracks `saw_hierarchy_registered_mixed_support_routing` and the
  Phase 4 hierarchy coverage gate rejects matrices that never prove a
  registered child-input route mixing parent ports with child outputs.
- The existing registered parent-composed integration regression now
  asserts the mixed-support metrics too.
- CLI/config docs and live project docs were refreshed for the new
  registered route shape.

**Why**

- The first registered parent-composed route was intentionally
  sibling-output-derived only. The next small hierarchy step was to
  let the registered route use the same richer parent source pool as
  the combinational route: parent data ports, earlier sibling outputs,
  and earlier parent-side route gates.
- This makes the registered hierarchy surface less one-note while
  keeping it under the existing
  `hierarchy_registered_child_input_cone_prob` knob.

**Validation**

- `cargo fmt --all --check`
- `cargo check --all-targets`
- `cargo test --test pipeline hierarchy_child_inputs_can_be_registered_from_parent_composed_logic`
- `cargo test --bin tool_matrix phase4_hierarchy_coverage_requires_design_facts`
- Focused smoke:
  `/tmp/anvil-hier-registered-mixed-child-input-smoke-r1/manifest.json`
  records `child_input_bindings_from_registered_mixed_support = 3`,
  `top_child_input_bindings_from_registered_mixed_support = 3`, and
  `registered_mixed_support_child_input_binding_fraction = 0.75`.
- The focused smoke is clean in Verilator, Yosys `synth -noabc`, and
  the repo-owned Yosys with-ABC path.
- Coverage-only Phase 4 matrix:
  `/tmp/anvil-tool-matrix-phase4-registered-mixed-r1/tool_matrix_report.json`
  records `coverage_gaps = []` and
  `saw_hierarchy_registered_mixed_support_routing = true`.
- `cargo test`
- `cargo clippy --all-targets -- -D warnings`
- `mdbook build book`
- `git diff --check`

## 2026-04-24-boot6 — Bank mixed parent-output hierarchy coverage

**Landed as:** this commit

**What changed**

- [src/bin/tool_matrix.rs](/Users/richarddje/Documents/github/anvil/src/bin/tool_matrix.rs)
  now tracks `saw_hierarchy_parent_port_composed_outputs` in
  `CoverageSummary`.
- The Phase 4 hierarchy coverage gate now fails if the representative
  matrix never emits hierarchy outputs that mix parent data ports with
  child instance outputs.
- The existing Phase 4 tool-matrix coverage unit test now asserts that
  missing this fact produces a coverage gap.
- Live docs were refreshed:
  [README.md](/Users/richarddje/Documents/github/anvil/README.md),
  [ROADMAP.md](/Users/richarddje/Documents/github/anvil/ROADMAP.md),
  [CODEBASE_ANALYSIS.md](/Users/richarddje/Documents/github/anvil/CODEBASE_ANALYSIS.md),
  [MEMORY.md](/Users/richarddje/Documents/github/anvil/MEMORY.md),
  [book/src/hierarchy.md](/Users/richarddje/Documents/github/anvil/book/src/hierarchy.md),
  and [book/src/architecture.md](/Users/richarddje/Documents/github/anvil/book/src/architecture.md).

**Why**

- The previous slice landed mixed parent-port / child-output parent
  outputs and a focused smoke. The repo-owned Phase 4 matrix should
  also treat that behavior as a required representative coverage fact.
- This keeps the hierarchy gate from drifting back to child-output-only
  parent composition if future scenario profiles change.

**Validation**

- `cargo fmt --all --check`
- `cargo check --all-targets`
- `cargo test`
- `cargo clippy --all-targets -- -D warnings`
- `mdbook build book`
- `git diff --check`
- `cargo test --bin tool_matrix phase4_hierarchy`
- Coverage-only Phase 4 matrix:
  `/tmp/anvil-tool-matrix-phase4-parent-port-coverage-r1/tool_matrix_report.json`
  records `coverage_gaps = []` and
  `saw_hierarchy_parent_port_composed_outputs = true`.

## 2026-04-24-boot5 — Land mixed parent-output hierarchy composition

**Landed as:** this commit

**What changed**

- [src/gen/hierarchy.rs](/Users/richarddje/Documents/github/anvil/src/gen/hierarchy.rs)
  now builds hierarchy parent output cones from the full parent source
  pool rather than only child instance outputs. After finalization, it
  rebuilds live parent-source pools and repairs parent outputs so they
  retain child-output support and, when live parent data inputs exist,
  can also carry parent-port support.
- [src/metrics.rs](/Users/richarddje/Documents/github/anvil/src/metrics.rs)
  now reports mixed parent-output composition through
  `top_parent_port_composed_outputs`,
  `hierarchy_parent_port_composed_outputs`,
  `top_parent_port_composed_output_fraction`, and
  `hierarchy_parent_port_composed_output_fraction`.
- [tests/pipeline.rs](/Users/richarddje/Documents/github/anvil/tests/pipeline.rs)
  adds an integration regression proving mixed parent-port /
  child-output parent outputs across the live construction strategies.
- Live docs and the mdBook were refreshed:
  [README.md](/Users/richarddje/Documents/github/anvil/README.md),
  [ROADMAP.md](/Users/richarddje/Documents/github/anvil/ROADMAP.md),
  [CODEBASE_ANALYSIS.md](/Users/richarddje/Documents/github/anvil/CODEBASE_ANALYSIS.md),
  [DEVELOPMENT_NOTES.md](/Users/richarddje/Documents/github/anvil/DEVELOPMENT_NOTES.md),
  [MEMORY.md](/Users/richarddje/Documents/github/anvil/MEMORY.md),
  [book/src/hierarchy.md](/Users/richarddje/Documents/github/anvil/book/src/hierarchy.md),
  [book/src/knobs.md](/Users/richarddje/Documents/github/anvil/book/src/knobs.md),
  and [book/src/architecture.md](/Users/richarddje/Documents/github/anvil/book/src/architecture.md).

**Why**

- The previous parent-output slice proved composition above child
  instance outputs, but parent data inputs did not participate in the
  parent output surface.
- Building from the full parent source pool gives parent outputs a
  richer legal support set while the post-final repair preserves the
  key hierarchy invariant: parent outputs must still reach child
  instance outputs.
- Measuring parent-port-composed outputs keeps the feature auditable
  from manifests and reports instead of relying on SV inspection.

**Validation**

- Focused regression:
  - `cargo test --test pipeline hierarchy_parent_outputs_can_mix_parent_ports_with_child_outputs`
  - `cargo test --test pipeline hierarchy_`
- Full hygiene:
  - `cargo check --all-targets`
  - `cargo test`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo fmt --all --check`
  - `mdbook build book`
- Focused downstream smoke:
  - `/tmp/anvil-hier-parent-output-mix-smoke-r1/manifest.json`
    records `top_parent_port_composed_outputs = 8`,
    `hierarchy_parent_port_composed_outputs = 8`,
    `top_outputs_reaching_instance_outputs = 8`, and
    `top_outputs_without_instance_outputs = 0`.
  - The emitted three-module design is clean in Verilator, Yosys
    `synth -noabc`, and the repo-owned ABC-enabled Yosys path.

## 2026-04-24-boot4 — Land registered parent-composed hierarchy routing

**Landed as:** this commit

**What changed**

- [src/config.rs](/Users/richarddje/Documents/github/anvil/src/config.rs)
  now carries `hierarchy_registered_child_input_cone_prob`,
  defaulting to `0.0`, as an explicit hierarchy routing knob.
- [src/main.rs](/Users/richarddje/Documents/github/anvil/src/main.rs)
  exposes the knob as
  `--hierarchy-registered-child-input-cone-prob <float>`.
- [src/ir/types.rs](/Users/richarddje/Documents/github/anvil/src/ir/types.rs)
  adds `KnobId::HierarchyRegisteredChildInputConeProb` for
  telemetry and matrix coverage.
- [src/gen/hierarchy.rs](/Users/richarddje/Documents/github/anvil/src/gen/hierarchy.rs)
  now lets a later child data input bind from earlier
  sibling-output-derived parent sources through parent-local
  combinational logic and then one local parent flop. The generated
  route is structurally:
  child output / parent route gate -> parent logic -> parent flop ->
  later child input.
- [src/metrics.rs](/Users/richarddje/Documents/github/anvil/src/metrics.rs)
  now reports registered parent-composed child-input provenance:
  `child_input_bindings_from_registered_parent_composed_logic`,
  `top_child_input_bindings_from_registered_parent_composed_logic`,
  `registered_parent_composed_child_input_binding_fraction`, and
  `top_registered_parent_composed_child_input_binding_fraction`.
- [src/bin/tool_matrix.rs](/Users/richarddje/Documents/github/anvil/src/bin/tool_matrix.rs)
  adds a Phase 4 registered child-input-cone scenario per construction
  strategy, raises the Phase 4 hierarchy gate to 30 scenarios / 120
  designs, and requires
  `saw_hierarchy_registered_parent_composed_routing = true`.
- [tests/pipeline.rs](/Users/richarddje/Documents/github/anvil/tests/pipeline.rs)
  now proves registered parent-composed child-input binding across the
  live construction strategies.
- Live docs and the mdBook were refreshed:
  [README.md](/Users/richarddje/Documents/github/anvil/README.md),
  [USER_GUIDE.md](/Users/richarddje/Documents/github/anvil/USER_GUIDE.md),
  [ROADMAP.md](/Users/richarddje/Documents/github/anvil/ROADMAP.md),
  [CODEBASE_ANALYSIS.md](/Users/richarddje/Documents/github/anvil/CODEBASE_ANALYSIS.md),
  [DEVELOPMENT_NOTES.md](/Users/richarddje/Documents/github/anvil/DEVELOPMENT_NOTES.md),
  [MEMORY.md](/Users/richarddje/Documents/github/anvil/MEMORY.md),
  [book/src/hierarchy.md](/Users/richarddje/Documents/github/anvil/book/src/hierarchy.md),
  [book/src/knobs.md](/Users/richarddje/Documents/github/anvil/book/src/knobs.md),
  and [book/src/architecture.md](/Users/richarddje/Documents/github/anvil/book/src/architecture.md).

**Why**

- Direct registered sibling routing proved the route
  child output -> parent flop -> later child input. The next
  composition-level axis is richer: parent combinational logic must sit
  before the register, and the metrics must prove that structure.
- Keeping this as its own knob avoids hiding the behavior under either
  `hierarchy_registered_sibling_route_prob` or
  `hierarchy_parent_flop_prob`.
- The route is intentionally generated by a signoff-clean construction
  rule, not a post-hoc SV rewrite: the D side is built from
  sibling-output-derived parent sources and wrapped in non-collapsing
  parent logic when needed before the local flop is allocated.

**Validation**

- Full hygiene:
  - `cargo check --all-targets`
  - `cargo test`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo fmt --all --check`
  - `mdbook build book`
- Focused regressions:
  - `cargo test --test pipeline hierarchy_child_inputs_can_be_registered_from_parent_composed_logic`
  - `cargo test --bin tool_matrix phase4_hierarchy`
- Focused registered parent-composed smoke:
  `/tmp/anvil-hier-registered-child-input-cone-smoke-r2/manifest.json`
  records
  `child_input_bindings_from_registered_parent_composed_logic = 3`,
  `top_child_input_bindings_from_registered_parent_composed_logic = 3`,
  `registered_parent_composed_child_input_binding_fraction = 0.75`,
  `top_registered_parent_composed_child_input_binding_fraction = 0.75`,
  `child_input_bindings_from_registered_instance_outputs = 3`,
  `hierarchy_parent_local_flops = 3`, `top_clock_inputs = 1`, and
  `top_reset_inputs = 1`. The emitted design is clean in Verilator,
  Yosys `synth -noabc`, and the repo-owned Yosys with-ABC path.
- Refreshed repo-owned Phase 4 gate:
  `/tmp/anvil-tool-matrix-phase4-hierarchy-r19/tool_matrix_report.json`
  closes at 30 scenarios, 4 designs/scenario, 120 total designs,
  `coverage_gaps = []`, and `120/0` pass-fail in Verilator plus both
  repo-owned Yosys modes. Its coverage facts include
  `saw_hierarchy_registered_parent_composed_routing = true` and
  `hierarchy_registered_child_input_cone_prob` in both knob-attempts
  and knob-fires.

**Impact**

- Phase 4 now has a measured registered parent-composed child-input
  route, not only direct registered sibling routing.
- Hierarchy quality metrics can distinguish a register fed directly by
  a sibling output from a register whose D input is parent-local logic
  over sibling-output-derived sources.
- Phase 4 remains `in progress`: richer multi-stage registered
  hierarchy composition patterns and hierarchy-aware identity are still
  open.

## 2026-04-24-boot3 — Land registered hierarchy sibling routing

**Landed as:** `619f775`

**What changed**

- [src/config.rs](/Users/richarddje/Documents/github/anvil/src/config.rs)
  now carries `hierarchy_registered_sibling_route_prob`, defaulting to
  `0.0`, as an explicit hierarchy routing knob.
- [src/main.rs](/Users/richarddje/Documents/github/anvil/src/main.rs)
  exposes the knob as `--hierarchy-registered-sibling-route-prob <float>`.
- [src/ir/types.rs](/Users/richarddje/Documents/github/anvil/src/ir/types.rs)
  adds `KnobId::HierarchyRegisteredSiblingRouteProb` and a `DepSet`
  iterator for local flop virtual endpoints.
- [src/gen/hierarchy.rs](/Users/richarddje/Documents/github/anvil/src/gen/hierarchy.rs)
  now lets a later child data input bind from an earlier sibling
  instance output through one local parent flop. The route is
  deliberately separate from direct combinational sibling routing and
  from generic parent-composed child-input cones.
- [src/metrics.rs](/Users/richarddje/Documents/github/anvil/src/metrics.rs)
  now reports registered sibling-route provenance:
  `child_input_bindings_from_registered_instance_outputs`,
  `top_child_input_bindings_from_registered_instance_outputs`,
  `registered_instance_output_child_input_binding_fraction`, and
  `top_registered_instance_output_child_input_binding_fraction`.
- [src/ir/compact.rs](/Users/richarddje/Documents/github/anvil/src/ir/compact.rs)
  now rewrites child instance input bindings during partial NodeId
  remaps. This fixes the root cause exposed by registered hierarchy
  routing: instance inputs are live NodeId consumers just like output
  drives and flop fields.
- [src/bin/tool_matrix.rs](/Users/richarddje/Documents/github/anvil/src/bin/tool_matrix.rs)
  adds a Phase 4 registered-sibling-state scenario per construction
  strategy, raises the Phase 4 gate to 27 scenarios / 108 designs, and
  requires `saw_hierarchy_registered_sibling_routing = true`.
- [tests/pipeline.rs](/Users/richarddje/Documents/github/anvil/tests/pipeline.rs)
  now proves registered sibling-routed child-input binding across the
  live construction strategies.
- Live docs and the mdBook were refreshed:
  [README.md](/Users/richarddje/Documents/github/anvil/README.md),
  [USER_GUIDE.md](/Users/richarddje/Documents/github/anvil/USER_GUIDE.md),
  [ROADMAP.md](/Users/richarddje/Documents/github/anvil/ROADMAP.md),
  [CODEBASE_ANALYSIS.md](/Users/richarddje/Documents/github/anvil/CODEBASE_ANALYSIS.md),
  [DEVELOPMENT_NOTES.md](/Users/richarddje/Documents/github/anvil/DEVELOPMENT_NOTES.md),
  [MEMORY.md](/Users/richarddje/Documents/github/anvil/MEMORY.md),
  [book/src/hierarchy.md](/Users/richarddje/Documents/github/anvil/book/src/hierarchy.md),
  [book/src/knobs.md](/Users/richarddje/Documents/github/anvil/book/src/knobs.md),
  and [book/src/architecture.md](/Users/richarddje/Documents/github/anvil/book/src/architecture.md).

**Why**

- Direct sibling routing and parent-local state were already live, but
  there was no explicit way to ask for the composition-level analogue
  of a registered handoff: child output -> parent flop -> later child
  input.
- Keeping this as its own knob avoids hidden coupling to
  `hierarchy_parent_flop_prob` and makes the adversarial-generation
  axis measurable in `tool_matrix`.
- The implementation uncovered a real finalization invariant:
  hierarchy child inputs are NodeId consumers, so every post-construction
  remap pass must rewrite instance input bindings too.

**Validation**

- Full hygiene:
  - `cargo check --all-targets`
  - `cargo test`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo fmt --all --check`
  - `mdbook build book`
- Focused regressions:
  - `cargo test ir::compact::tests::merge_equivalent_flops_rewrites_consumers_and_deps`
  - `cargo test --test pipeline hierarchy_child_inputs_can_be_registered_from_sibling_instance_outputs`
  - `cargo test --bin tool_matrix phase4_hierarchy`
- Focused registered-sibling smoke:
  `/tmp/anvil-hier-registered-sibling-smoke-r1/manifest.json` records
  `child_input_bindings_from_registered_instance_outputs = 4`,
  `top_child_input_bindings_from_registered_instance_outputs = 4`,
  `registered_instance_output_child_input_binding_fraction = 0.8`,
  `top_registered_instance_output_child_input_binding_fraction = 0.8`,
  `hierarchy_parent_local_flops = 3`, `top_clock_inputs = 1`, and
  `top_reset_inputs = 1`. The emitted design is clean in Verilator,
  Yosys `synth -noabc`, and the repo-owned Yosys with-ABC path.
- Refreshed repo-owned Phase 4 gate:
  `/tmp/anvil-tool-matrix-phase4-hierarchy-r17/tool_matrix_report.json`
  closes at 27 scenarios, 4 designs/scenario, 108 total designs,
  `coverage_gaps = []`, and `108/0` pass-fail in Verilator plus both
  repo-owned Yosys modes. Its coverage facts include
  `saw_hierarchy_registered_sibling_routing = true` and
  `hierarchy_registered_sibling_route_prob` in both knob-attempts and
  knob-fires.

**Impact**

- Phase 4 now has a measured registered child-to-child routing surface,
  not only direct combinational sibling routing.
- Parent-local state can now act as an explicit inter-child route
  element.
- The compact/remap infrastructure is stronger for all hierarchy
  child-input bindings, not only this new route.
- Phase 4 remains `in progress`: richer registered child-to-child
  patterns, deeper hierarchy composition, and hierarchy-aware identity
  are still open.

## 2026-04-24-boot2 — Clarify hierarchy parent wording after bootstrap

**Landed as:** `cf3dc3c164b0f8bb908d23d15b8248c275b683fb`

**What changed**

- [README.md](/Users/richarddje/Documents/github/anvil/README.md)
  now describes hierarchy parent outputs as parent-side cones over
  child instance outputs, combinational by default and optionally
  stateful when parent-local state is requested.
- [USER_GUIDE.md](/Users/richarddje/Documents/github/anvil/USER_GUIDE.md)
  now says the same thing in the hierarchy metrics/trust-surface
  section instead of implying top outputs are only combinational cones.
- [CODEBASE_ANALYSIS.md](/Users/richarddje/Documents/github/anvil/CODEBASE_ANALYSIS.md)
  now uses "hierarchy parents" instead of "wrappers" for the
  control-port boundary doctrine, matching both wrapper and bounded
  recursive parent modules.
- [book/src/structural-rules.md](/Users/richarddje/Documents/github/anvil/book/src/structural-rules.md)
  now phrases Rule 5's boundary rule in hierarchy-parent terms instead
  of wrapper-only terms.
- [MEMORY.md](/Users/richarddje/Documents/github/anvil/MEMORY.md)
  was refreshed with this bootstrap result and the previous commit hash.

**Why**

- Executing `README.md` led through `SESSION_BOOTSTRAP.md`, which says
  to amend live docs when the bootstrap pass finds facts that are
  missed or misstated.
- The code and most docs were already correct after the local
  parent-state slice, but a few summaries still carried pre-state or
  wrapper-only wording. That is exactly the kind of small drift that can
  mislead a cold recovery session.

**Validation**

- `cargo check --all-targets`
- `cargo test`
- `cargo clippy --all-targets -- -D warnings`
- `cargo fmt --all --check`
- `mdbook build book`

**Impact**

- The live docs now consistently describe parent-side hierarchy cones
  as combinational by default, optionally stateful under
  `hierarchy_parent_flop_prob`.
- The control-port doctrine is described in terms of all hierarchy
  parents, not only depth-1 wrappers.

## 2026-04-24-boot1 — Land local parent state in hierarchy cones

**Landed as:** `87d49406ab15ee771bffa9f9c75493f009e9da8f`

**What changed**

- [src/config.rs](/Users/richarddje/Documents/github/anvil/src/config.rs)
  now carries `hierarchy_parent_flop_prob`, defaulting to `0.0`, as a
  hierarchy-specific state knob.
- [src/main.rs](/Users/richarddje/Documents/github/anvil/src/main.rs)
  exposes the knob as `--hierarchy-parent-flop-prob <float>`.
- [src/gen/mod.rs](/Users/richarddje/Documents/github/anvil/src/gen/mod.rs)
  now tracks the active flop-roll telemetry key, so parent-side flops
  can be measured separately from ordinary leaf `flop_prob`.
- [src/gen/cone.rs](/Users/richarddje/Documents/github/anvil/src/gen/cone.rs)
  records flop-roll attempts through that active key instead of always
  charging them to `flop_prob`.
- [src/gen/hierarchy.rs](/Users/richarddje/Documents/github/anvil/src/gen/hierarchy.rs)
  now lets parent output cones and parent-composed child-input cones
  emit local parent flops when `hierarchy_parent_flop_prob` is nonzero.
  Parent flop worklists are drained before finalization, and control
  ports are reserved only when local parent state is possible or a
  sequential child requires them.
- [src/ir/types.rs](/Users/richarddje/Documents/github/anvil/src/ir/types.rs)
  adds `KnobId::HierarchyParentFlopProb` and a `DepSet` helper for
  detecting local flop endpoints.
- [src/metrics.rs](/Users/richarddje/Documents/github/anvil/src/metrics.rs)
  now reports local parent-state facts:
  `hierarchy_parent_local_flops`,
  `internal_module_occurrences_with_local_flops`, `top_local_flops`,
  `child_input_bindings_from_parent_flops`,
  `parent_flop_child_input_binding_fraction`, and
  `top_parent_flop_child_input_binding_fraction`.
- [src/bin/tool_matrix.rs](/Users/richarddje/Documents/github/anvil/src/bin/tool_matrix.rs)
  adds a Phase 4 parent-state scenario per construction strategy,
  raises the Phase 4 gate to 24 scenarios / 96 designs, and requires
  `saw_hierarchy_parent_local_flops = true`.
- [tests/pipeline.rs](/Users/richarddje/Documents/github/anvil/tests/pipeline.rs)
  now proves local parent flops across the live construction strategies.
- Live docs and the mdBook were refreshed:
  [README.md](/Users/richarddje/Documents/github/anvil/README.md),
  [USER_GUIDE.md](/Users/richarddje/Documents/github/anvil/USER_GUIDE.md),
  [ROADMAP.md](/Users/richarddje/Documents/github/anvil/ROADMAP.md),
  [CODEBASE_ANALYSIS.md](/Users/richarddje/Documents/github/anvil/CODEBASE_ANALYSIS.md),
  [DEVELOPMENT_NOTES.md](/Users/richarddje/Documents/github/anvil/DEVELOPMENT_NOTES.md),
  [MEMORY.md](/Users/richarddje/Documents/github/anvil/MEMORY.md),
  [book/src/hierarchy.md](/Users/richarddje/Documents/github/anvil/book/src/hierarchy.md),
  [book/src/ir.md](/Users/richarddje/Documents/github/anvil/book/src/ir.md),
  [book/src/knobs.md](/Users/richarddje/Documents/github/anvil/book/src/knobs.md),
  and [book/src/architecture.md](/Users/richarddje/Documents/github/anvil/book/src/architecture.md).

**Why**

- Phase 4 had already landed parent-composed outputs, sibling-routed
  child inputs, and parent-composed child-input cones. The next honest
  structural step was parent-local state, but as its own hierarchy axis
  rather than a hidden reuse of leaf `flop_prob`.
- Keeping the default at `0.0` preserves the clean combinational parent
  layer unless the user explicitly asks for state, while still allowing
  stress runs to exercise registered parent routing.
- The control-port doctrine remains intact: pure comb-only modules
  stay free of `clk` / `rst_n`; modules with local flops or sequential
  descendants emit them.

**Validation**

- Full hygiene:
  - `cargo check --all-targets`
  - `cargo test`
    - 184 lib tests
    - 5 main tests
    - 26 `tool_matrix` tests
    - 42 integration tests
    - 0 doctests
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo fmt --all --check`
  - `mdbook build book`
- Focused regressions:
  - `cargo test --test pipeline hierarchy_parents_can_emit_local_flops`
  - `cargo test --bin tool_matrix phase4_hierarchy`
- Focused parent-state smoke:
  `/tmp/anvil-hier-parent-state-smoke-r1/manifest.json` records
  `hierarchy_parent_local_flops = 8`, `top_local_flops = 8`,
  `top_clock_inputs = 1`, `top_reset_inputs = 1`, and
  `child_input_bindings_from_parent_flops = 1`. The emitted design is
  clean in Verilator, Yosys `synth -noabc`, and the repo-owned Yosys
  with-ABC path.
- Refreshed repo-owned Phase 4 gate:
  `/tmp/anvil-tool-matrix-phase4-hierarchy-r16/tool_matrix_report.json`
  closes at 24 scenarios, 4 designs/scenario, 96 total designs,
  `coverage_gaps = []`, and `96/0` pass-fail in Verilator plus both
  repo-owned Yosys modes. Its coverage facts include
  `saw_hierarchy_parent_local_flops = true` and
  `hierarchy_parent_flop_prob` in both knob-attempts and knob-fires.

**Impact**

- Hierarchy parents can now be sequential modules because of their own
  local state, not only because they instantiate sequential children.
- Parent-side registered routing is now represented numerically in
  manifests and matrix reports.
- Remaining Phase 4 work moves to richer registered child-to-child
  patterns, deeper hierarchy composition, and future hierarchy-aware
  identity/factorization.

## 2026-04-23-boot9 — Land parent-composed hierarchy child inputs

**Landed as:** `30b1846a0bc2e611f97f864d6fa67cd4ffffced7`

**What changed**

- [src/config.rs](/Users/richarddje/Documents/github/anvil/src/config.rs)
  now carries `hierarchy_child_input_cone_prob`, defaulting to `0.35`,
  as a first-class hierarchy routing knob.
- [src/main.rs](/Users/richarddje/Documents/github/anvil/src/main.rs)
  exposes that knob as `--hierarchy-child-input-cone-prob <float>`.
- [src/ir/types.rs](/Users/richarddje/Documents/github/anvil/src/ir/types.rs)
  records `KnobId::HierarchyChildInputConeProb`, so the new surface is
  visible in knob telemetry instead of being an undocumented branch.
- [src/gen/hierarchy.rs](/Users/richarddje/Documents/github/anvil/src/gen/hierarchy.rs)
  now lets both wrapper and recursive hierarchy parents bind child data
  inputs through parent-local combinational cones over already-available
  parent sources: parent data inputs, earlier sibling instance outputs,
  and earlier parent-side route gates. The cone builder is reused here
  with local flops disabled, keeping the current Phase 4 surface purely
  combinational while still mirroring the leaf generator's "drive this
  sink from the live source pool" discipline.
- [src/metrics.rs](/Users/richarddje/Documents/github/anvil/src/metrics.rs)
  now reports parent-composed child-input provenance through
  `child_input_bindings_from_parent_composed_logic`,
  `parent_composed_child_input_binding_fraction`, and their top-level
  counterparts.
- [src/bin/tool_matrix.rs](/Users/richarddje/Documents/github/anvil/src/bin/tool_matrix.rs)
  now treats parent-composed child-input bindings as required Phase 4
  coverage, forces the knob in representative hierarchy scenarios, and
  rejects a hierarchy gate that never proves the surface.
- [src/gen/module.rs](/Users/richarddje/Documents/github/anvil/src/gen/module.rs)
  fixes a real width-finalization bug exposed by the broadened Phase 4
  matrix: `Instance.inputs` are now treated as live consumers when
  shrinking primary input widths and counting orphan gates, so a parent
  primary input directly bound to a child port cannot be narrowed based
  only on unrelated low-bit slice uses.
- [tests/pipeline.rs](/Users/richarddje/Documents/github/anvil/tests/pipeline.rs)
  now proves parent-composed child-input bindings across the live
  construction strategies and keeps the direct sibling-routing
  regression isolated by disabling the new preempting cone knob.
- Live docs and the mdBook were refreshed:
  [README.md](/Users/richarddje/Documents/github/anvil/README.md),
  [USER_GUIDE.md](/Users/richarddje/Documents/github/anvil/USER_GUIDE.md),
  [ROADMAP.md](/Users/richarddje/Documents/github/anvil/ROADMAP.md),
  [CODEBASE_ANALYSIS.md](/Users/richarddje/Documents/github/anvil/CODEBASE_ANALYSIS.md),
  [DEVELOPMENT_NOTES.md](/Users/richarddje/Documents/github/anvil/DEVELOPMENT_NOTES.md),
  [MEMORY.md](/Users/richarddje/Documents/github/anvil/MEMORY.md),
  [book/src/hierarchy.md](/Users/richarddje/Documents/github/anvil/book/src/hierarchy.md),
  [book/src/knobs.md](/Users/richarddje/Documents/github/anvil/book/src/knobs.md),
  and [book/src/architecture.md](/Users/richarddje/Documents/github/anvil/book/src/architecture.md).

**Why**

- The hierarchy planner had already learned to compose top outputs over
  child outputs and to route child inputs directly from earlier
  siblings. The next structural step was to replace direct-only child
  input routing with a real parent-composition layer: child inputs can
  now be driven by parent-local logic, not merely by raw top inputs or
  raw sibling outputs.
- The user explicitly asked for hierarchy routing to mirror the leaf
  generator, with child modules playing the role gates play in the leaf
  graph. This slice moves in that direction without adding local parent
  state prematurely.
- The Phase 4 matrix found a real generator defect while exercising the
  new surface. Treating instance input bindings as live consumers fixes
  the root cause rather than papering over the emitted width mismatch.

**Validation**

- Full hygiene:
  - `cargo check --all-targets`
  - `cargo test`
    - 184 lib tests
    - 5 main tests
    - 26 `tool_matrix` tests
    - 41 integration tests
    - 0 doctests
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo fmt --all --check`
  - `mdbook build book`
- Focused regressions:
  - `cargo test --test pipeline hierarchy_child_inputs`
  - `cargo test --bin tool_matrix phase4_hierarchy`
  - `cargo test --lib instance_input_binding`
  - `cargo test --lib design_metrics_capture_sibling_routed_child_inputs`
- Focused parent-composed child-input smoke:
  `/tmp/anvil-hier-child-input-cone-smoke-r1/manifest.json` records
  `child_input_bindings_from_parent_composed_logic = 13`,
  `top_child_input_bindings_from_parent_composed_logic = 13`,
  `parent_composed_child_input_binding_fraction = 0.9285714285714286`,
  and
  `top_parent_composed_child_input_binding_fraction = 0.9285714285714286`.
  The emitted design is clean in Verilator, Yosys `synth -noabc`, and
  the repo-owned Yosys with-ABC path.
- Width-bug repro after the fix:
  `/tmp/anvil-hier-child-cone-width-repro-r3` is clean in Verilator,
  Yosys `synth -noabc`, and the repo-owned Yosys with-ABC path.
- Refreshed repo-owned Phase 4 gate:
  `/tmp/anvil-tool-matrix-phase4-hierarchy-r15/tool_matrix_report.json`
  closes at 21 scenarios, 4 designs/scenario, 84 total designs,
  `coverage_gaps = []`, and `84/0` pass-fail in Verilator plus both
  repo-owned Yosys modes.

**Impact**

- Phase 4 hierarchy now has a measurable parent-composed child-input
  lane in addition to direct sibling routing and parent-composed top
  outputs.
- Hierarchy quality can be judged from metrics instead of visually
  inspecting emitted `.sv` files for this surface.
- Remaining Phase 4 work stays explicit: richer parent-side
  composition, local parent state where structurally warranted, and
  future hierarchy-aware identity/factorization.

## 2026-04-23-boot8 — Refresh bootstrap doc drift

**Landed as:** `8944c1401bb3e66c87d3978d596bacb3adab0f26`

**What changed**

- [book/src/ir.md](/Users/richarddje/Documents/github/anvil/book/src/ir.md)
  now matches the live IR: `GateOp::CaseMux`, `GateOp::CasezMux`,
  `GateOp::ForFold`, `ForFoldKind`, the structured block counters, and
  the current hierarchy status are documented instead of the older
  wrapper-only/future-parent-composition wording.
- [book/src/architecture.md](/Users/richarddje/Documents/github/anvil/book/src/architecture.md)
  refreshed the key-type snippets for `Node::InstanceOutput`,
  procedural structured gates, the newer knob telemetry variants, and
  the current emitter/metrics test counts.
- [book/src/algorithm.md](/Users/richarddje/Documents/github/anvil/book/src/algorithm.md)
  now describes flop-Q endpoints and anti-collapse gating the way the
  generator actually enforces them: lower factorization rungs relax the
  operand-uniqueness checks, while base `Sub` / `Eq` / `Neq` and
  mux-arm cleanup still guard obvious degeneracies.
- [book/src/non-triviality.md](/Users/richarddje/Documents/github/anvil/book/src/non-triviality.md)
  now states the endpoint rule accurately: a cone may be a function of
  primary inputs and/or flop Q endpoints, including a flop's own Q under
  Rule 2, but not a pure constant.
- [book/src/sequential.md](/Users/richarddje/Documents/github/anvil/book/src/sequential.md)
  now explains worklist termination via finite recursion and
  `max_flops_per_module`, not the stale assumption that `flop_prob < 1`.
- [book/src/factorization.md](/Users/richarddje/Documents/github/anvil/book/src/factorization.md)
  now warns that per-seed gate counts are not a strict monotonic proof
  of factorization strength because enabling rungs can change retry
  paths and legal shapes.
- [book/src/structural-rules.md](/Users/richarddje/Documents/github/anvil/book/src/structural-rules.md)
  now says every current factorization rung is implemented and that the
  `e-graph` rung is the bounded semantic fragment plus lower layers,
  while still leaving room for future stronger semantic engines.
- [ROADMAP.md](/Users/richarddje/Documents/github/anvil/ROADMAP.md)
  now consistently identifies `r13` as the current fully banked Phase 4
  hierarchy closure artifact instead of leaving an older `r10`
  reference in the wrapper-planning paragraph.
- [src/config.rs](/Users/richarddje/Documents/github/anvil/src/config.rs)
  and [src/gen/cone.rs](/Users/richarddje/Documents/github/anvil/src/gen/cone.rs)
  had stale comments corrected for the implemented factorization
  ladder, `effective()` behavior, `GraphFirst` aliasing, and the
  deliberate `exclude = None` Q-feedback doctrine for flop D-cones.

**Why**

- `README.md` / `SESSION_BOOTSTRAP.md` require a literal live-doc,
  mdBook, and source walk before continuing work.
- That walk found several stale descriptions left by earlier slices.
  They were documentation/comment drift only, but they were exactly the
  sort of drift that can steer later implementation in the wrong
  direction if left behind.

**Validation**

- `cargo check --all-targets`
- `cargo test`
  - 182 lib tests
  - 5 main tests
  - 26 `tool_matrix` tests
  - 40 integration tests
  - 0 doctests
- `cargo clippy --all-targets -- -D warnings`
- `cargo fmt --all --check`
- `mdbook build book`

## 2026-04-23-boot7 — Land sibling-routed hierarchy child inputs

**Landed as:** this commit

**What changed**

- [src/config.rs](/Users/richarddje/Documents/github/anvil/src/config.rs)
  now carries `hierarchy_sibling_route_prob`, a real Phase 4 knob for
  whether later sibling child inputs may bind from earlier sibling
  instance outputs instead of always coming from parent-boundary
  inputs.
- [src/main.rs](/Users/richarddje/Documents/github/anvil/src/main.rs)
  now exposes that knob directly as
  `--hierarchy-sibling-route-prob <float>`.
- [src/gen/hierarchy.rs](/Users/richarddje/Documents/github/anvil/src/gen/hierarchy.rs)
  now gives both the wrapper and recursive hierarchy lanes a genuine
  sibling-routing composition surface: later child instances may now
  bind data inputs from dep-bearing earlier sibling outputs through the
  same width-adaptation machinery used elsewhere in the generator,
  while staying acyclic by construction. This slice stays intentionally
  combinational at the parent level; local parent flops remain future
  work.
- [src/ir/types.rs](/Users/richarddje/Documents/github/anvil/src/ir/types.rs)
  now records `KnobId::HierarchySiblingRouteProb`, and `DepSet`
  exposes `has_ports()` so hierarchy metrics can distinguish pure
  parent-port support from sibling-instance-output support and mixed
  support.
- [src/metrics.rs](/Users/richarddje/Documents/github/anvil/src/metrics.rs)
  now reports trustworthy child-input provenance metrics:
  `child_input_bindings_from_parent_ports`,
  `child_input_bindings_from_instance_outputs`,
  `child_input_bindings_from_mixed_support`,
  `child_input_bindings_from_constants`,
  `instance_output_child_input_binding_fraction`,
  plus the corresponding top-level variants.
- [src/bin/tool_matrix.rs](/Users/richarddje/Documents/github/anvil/src/bin/tool_matrix.rs)
  now treats sibling-routed hierarchy child inputs as a required Phase 4
  coverage fact, and the banked Phase 4 matrix explicitly exercises the
  new knob.
- [tests/pipeline.rs](/Users/richarddje/Documents/github/anvil/tests/pipeline.rs)
  now proves the sibling-routing surface across all live construction
  strategies via a seed sweep, while the existing metrics unit tests pin
  the direct counts.

**Why**

- The hierarchy planner had already grown real parent-side composition
  over child outputs, but child input binding was still too close to a
  pure wrapper discipline.
- The next honest step was to let the parent route between siblings the
  way the leaf generator routes between live dep-bearing signals, while
  still keeping the current phase intentionally simpler than the later
  local-parent-state surface.
- The user also asked to trust the numbers without reading emitted SV,
  so sibling routing had to land together with explicit provenance
  metrics and a refreshed repo-owned Phase 4 gate.

**Validation**

- Full hygiene:
  - `cargo check --all-targets`
  - `cargo test`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo fmt --all --check`
  - `mdbook build book`
- Focused regressions:
  - `cargo test hierarchy_child_inputs_can_be_routed_from_sibling_instance_outputs --test pipeline`
  - `cargo test design_metrics_capture_sibling_routed_child_inputs --lib`
  - `cargo test phase4_hierarchy_coverage_requires_design_facts --bin tool_matrix`
- Focused sibling-routing smoke:
  - `cargo run --bin anvil -- --seed 29 --hierarchy-depth 1 --num-leaf-modules 2 --num-child-instances 4 --hierarchy-sibling-route-prob 1.0 --count 1 --out /tmp/anvil-hier-sibling-routing-smoke-r1`
  - report:
    `/tmp/anvil-hier-sibling-routing-smoke-r1/manifest.json`
  - key facts:
    - `child_input_bindings_from_instance_outputs = 6`
    - `top_child_input_bindings_from_instance_outputs = 6`
    - `instance_output_child_input_binding_fraction = 0.75`
    - `top_instance_output_child_input_binding_fraction = 0.75`
- Refreshed repo-owned Phase 4 rerun:
  - `cargo run --bin tool_matrix -- --out /tmp/anvil-tool-matrix-phase4-hierarchy-r13 --phase4-hierarchy-gate --yosys-mode both`
  - report:
    `/tmp/anvil-tool-matrix-phase4-hierarchy-r13/tool_matrix_report.json`
  - key facts:
    - `scenario_count = 21`
    - `total_modules = 84`
    - `coverage_gaps = []`
    - `tool_summary.verilator_passed = 84`
    - `tool_summary.yosys_without_abc_passed = 84`
    - `tool_summary.yosys_with_abc_passed = 84`
    - hierarchy coverage facts:
      - `hierarchy_child_source_modes = ["library", "on-demand"]`
      - `saw_profiled_child_interface_synthesis = true`
      - `saw_hierarchy_parent_composition = true`
      - `saw_hierarchy_sibling_routing = true`

**Impact**

- Phase 4 parent-side composition is no longer only "child output up to
  top output". Later sibling instances may now consume earlier sibling
  instance outputs directly, and the result is measurable numerically.
- The current hierarchy doctrine is sharper now too: routing is richer,
  but the slice still stays **combinational**. Local parent flops remain
  future work rather than being smuggled into this step.
- The banked Phase 4 proof artifact is now `r13`, not `r12`.
- Phase 4 remains `in progress`. The next honest work narrows to local
  parent state, deeper hierarchy composition beyond the current
  sibling-binding slice, and later hierarchy-aware identity.

**Files touched**

- [src/config.rs](/Users/richarddje/Documents/github/anvil/src/config.rs)
- [src/main.rs](/Users/richarddje/Documents/github/anvil/src/main.rs)
- [src/ir/types.rs](/Users/richarddje/Documents/github/anvil/src/ir/types.rs)
- [src/gen/hierarchy.rs](/Users/richarddje/Documents/github/anvil/src/gen/hierarchy.rs)
- [src/metrics.rs](/Users/richarddje/Documents/github/anvil/src/metrics.rs)
- [src/bin/tool_matrix.rs](/Users/richarddje/Documents/github/anvil/src/bin/tool_matrix.rs)
- [tests/pipeline.rs](/Users/richarddje/Documents/github/anvil/tests/pipeline.rs)

## 2026-04-23-boot6 — Land exact profiled on-demand child synthesis

**Landed as:** `57eef7e`

**What changed**

- [src/ir/types.rs](/Users/richarddje/Documents/github/anvil/src/ir/types.rs)
  now carries `ModuleInterfaceProfile` plus
  `Module::planned_interface_profile`, so exact data-interface demand
  is part of the live IR contract rather than an implicit planner
  side-channel.
- [src/gen/module.rs](/Users/richarddje/Documents/github/anvil/src/gen/module.rs)
  now supports exact interface-profiled leaf synthesis, reusable
  interface sampling, and a final exact-profile enforcement pass so a
  planned data boundary survives compaction / shrink / pruning cleanly.
- [src/gen/mod.rs](/Users/richarddje/Documents/github/anvil/src/gen/mod.rs)
  now exposes `generate_module_with_interface_profile(...)`.
- [src/gen/hierarchy.rs](/Users/richarddje/Documents/github/anvil/src/gen/hierarchy.rs)
  now plans exact data-interface profiles for `on-demand` child slots
  and threads those profiles through both wrapper and recursive
  hierarchy generation. Internal parent roots can now honor an exact
  demanded external data interface too.
- [src/ir/validate.rs](/Users/richarddje/Documents/github/anvil/src/ir/validate.rs)
  now rejects any profiled module whose emitted data-input or output
  widths drift from its planned profile.
- [src/metrics.rs](/Users/richarddje/Documents/github/anvil/src/metrics.rs)
  now reports profile-quality facts directly:
  `num_profiled_module_definitions`,
  `num_profiled_instantiated_modules`,
  `num_profiled_instance_slots`,
  `profiled_instantiated_module_fraction`,
  `profiled_instance_fraction`,
  `dep_bearing_child_input_bindings`, and
  `dep_bearing_child_input_binding_fraction`.
- [src/bin/tool_matrix.rs](/Users/richarddje/Documents/github/anvil/src/bin/tool_matrix.rs)
  now requires the Phase 4 hierarchy gate to prove not only structural
  on-demand sourcing, but exact profiled child-interface synthesis too
  via `saw_profiled_child_interface_synthesis`.

**Why**

- The old `on-demand` truth was still too weak for the direction we had
  agreed on. "Fresh child per slot" was useful, but it still left the
  child's data boundary as a child-local random choice rather than a
  parent-planned exact contract.
- The repo already had the right architectural seam for a stronger
  solution: parent modules bind child inputs through typed width
  adaptation and already build composed outputs over child
  `InstanceOutput` leaves. That meant we could land exact profiled
  data-interface synthesis without faking it.
- The user also asked to trust the numbers without opening `.sv`, so
  the stronger on-demand slice had to land together with validator
  enforcement, new hierarchy metrics, and a refreshed repo-owned gate.

**Validation**

- Full hygiene:
  - `cargo check --all-targets`
  - `cargo test`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo fmt --all --check`
  - `mdbook build book`
- Focused hierarchy regressions:
  - `cargo test profiled_parent_module_honors_exact_data_interface_shape --lib`
  - `cargo test design_metrics_capture_on_demand_single_use_child_sourcing --lib`
  - `cargo test generates_valid_depth1_ondemand_wrapper_designs --test pipeline`
  - `cargo test generates_valid_recursive_hierarchy_designs_with_ondemand_child_sourcing --test pipeline`
  - `cargo test on_demand_recursive_hierarchy_exactly_realizes_profiled_child_interfaces --test pipeline`
- Focused profiled on-demand smoke:
  - `cargo run --bin anvil -- --seed 2 --out /tmp/anvil-hier-profiled-ondemand-smoke-r1 --hierarchy-depth 1 --num-child-instances 3 --hierarchy-child-source-mode on-demand`
  - report:
    `/tmp/anvil-hier-profiled-ondemand-smoke-r1/manifest.json`
  - key facts:
    - `num_profiled_instance_slots = 3`
    - `profiled_instance_fraction = 1.0`
    - `profiled_instantiated_module_fraction = 1.0`
    - `dep_bearing_child_input_binding_fraction = 1.0`
- Refreshed repo-owned Phase 4 rerun:
  - `cargo run --bin tool_matrix -- --out /tmp/anvil-tool-matrix-phase4-hierarchy-r12 --phase4-hierarchy-gate --yosys-mode both`
  - report:
    `/tmp/anvil-tool-matrix-phase4-hierarchy-r12/tool_matrix_report.json`
  - key facts:
    - `scenario_count = 21`
    - `total_modules = 84`
    - `coverage_gaps = []`
    - `tool_summary.verilator_passed = 84`
    - `tool_summary.yosys_without_abc_passed = 84`
    - `tool_summary.yosys_with_abc_passed = 84`
    - hierarchy coverage facts:
      - `hierarchy_child_source_modes = ["library", "on-demand"]`
      - `saw_on_demand_child_sourcing = true`
      - `saw_profiled_child_interface_synthesis = true`

**Impact**

- `on-demand` in Phase 4 now means more than "fresh child per slot":
  it means parent-planned exact data-interface synthesis for child
  definitions, validated at the IR boundary and measurable in reports.
- Hierarchy manifests can now tell the difference between merely
  single-use child definitions and exact profiled child-interface
  realization, without emitted-SV inspection.
- The banked Phase 4 proof artifact is now `r12`, not `r11`.
- Phase 4 remains `in progress`. The next honest work narrows to local
  parent state, richer hierarchy composition/routing surfaces, and
  later hierarchy-aware identity.

**Files touched**

- [src/ir/types.rs](/Users/richarddje/Documents/github/anvil/src/ir/types.rs)
- [src/gen/module.rs](/Users/richarddje/Documents/github/anvil/src/gen/module.rs)
- [src/gen/mod.rs](/Users/richarddje/Documents/github/anvil/src/gen/mod.rs)
- [src/gen/hierarchy.rs](/Users/richarddje/Documents/github/anvil/src/gen/hierarchy.rs)
- [src/gen/cone.rs](/Users/richarddje/Documents/github/anvil/src/gen/cone.rs)
- [src/ir/validate.rs](/Users/richarddje/Documents/github/anvil/src/ir/validate.rs)
- [src/metrics.rs](/Users/richarddje/Documents/github/anvil/src/metrics.rs)
- [src/bin/tool_matrix.rs](/Users/richarddje/Documents/github/anvil/src/bin/tool_matrix.rs)
- [tests/pipeline.rs](/Users/richarddje/Documents/github/anvil/tests/pipeline.rs)

## 2026-04-23-boot5 — Land explicit hierarchy child sourcing

**Landed as:** `0fc7ae7`

**What changed**

- [src/config.rs](/Users/richarddje/Documents/github/anvil/src/config.rs)
  now makes hierarchy child sourcing a first-class, typed Phase 4 axis:
  `hierarchy_child_source_mode = library | on-demand`.
- [src/main.rs](/Users/richarddje/Documents/github/anvil/src/main.rs)
  now exposes that axis directly on the CLI as
  `--hierarchy-child-source-mode <library|on-demand>`.
- [src/gen/hierarchy.rs](/Users/richarddje/Documents/github/anvil/src/gen/hierarchy.rs)
  now keeps the old reusable-library planner as the `library` path and
  adds the first live `on-demand` path, where each planned instance
  slot gets a fresh child definition instead of picking from a reusable
  child pool.
- [src/metrics.rs](/Users/richarddje/Documents/github/anvil/src/metrics.rs)
  now reports explicit single-use vs reused child-definition facts:
  `avg_instances_per_unique_instantiated_module`,
  `num_single_use_instantiated_modules`,
  `num_multiuse_instantiated_modules`, and
  `single_use_instantiated_module_fraction`.
- [src/bin/tool_matrix.rs](/Users/richarddje/Documents/github/anvil/src/bin/tool_matrix.rs)
  now folds the child-sourcing axis into the repo-owned Phase 4 gate,
  requiring both `library` and `on-demand` scenarios plus structural
  proof that on-demand really emitted one fresh child definition per
  instance slot.
- The repo-owned Phase 4 hierarchy artifact is now refreshed from
  `r10` to `r11` and covers the widened surface.

**Why**

- The current hierarchy story had reached the point where "library
  child reuse vs fresh child synthesis" needed to be an explicit,
  measurable design axis, not just a future note in the roadmap.
- The user also asked for hierarchy quality to be judgeable from
  trusted numbers rather than by opening emitted `.sv`, so the first
  live on-demand slice had to land together with structural metrics and
  representative gate evidence.
- The honest first slice is "fresh child definition per instance slot";
  it is deliberately narrower than the later stronger target of
  width-demand-driven child synthesis with required port widths.

**Validation**

- Focused config / planner / metrics / pipeline regressions:
  - `cargo test validate_rejects_on_demand_hierarchy_knob_without_hierarchy --lib`
  - `cargo test validate_rejects_on_demand_wrapper_without_explicit_child_instances --lib`
  - `cargo test validate_rejects_leaf_library_knob_in_on_demand_wrapper_mode --lib`
  - `cargo test legacy_wrapper_on_demand_synthesizes_one_child_definition_per_instance --lib`
  - `cargo test design_metrics_capture_on_demand_single_use_child_sourcing --lib`
  - `cargo test generates_valid_depth1_ondemand_wrapper_designs --test pipeline`
  - `cargo test generates_valid_recursive_hierarchy_designs_with_ondemand_child_sourcing --test pipeline`
- Focused `tool_matrix` regressions:
  - `cargo test phase4_hierarchy_gate_raises_designs_per_scenario_for_matrix --bin tool_matrix`
  - `cargo test phase4_hierarchy_matrix_covers_wrapper_and_recursive_profiles --bin tool_matrix`
  - `cargo test phase4_hierarchy_coverage_requires_design_facts --bin tool_matrix`
  - `cargo test recursive_hierarchy_facts_follow_design_metrics --bin tool_matrix`
- Focused on-demand wrapper smoke:
  - `cargo run --bin anvil -- --seed 23 --out /tmp/anvil-hier-ondemand-wrapper-smoke-r1 --hierarchy-depth 1 --num-child-instances 3 --hierarchy-child-source-mode on-demand`
  - report:
    `/tmp/anvil-hier-ondemand-wrapper-smoke-r1/manifest.json`
  - key facts:
    - `num_instances = 3`
    - `num_unique_instantiated_modules = 3`
    - `num_single_use_instantiated_modules = 3`
    - `single_use_instantiated_module_fraction = 1.0`
    - `instance_reuse_fraction = 0.0`
    - `unused_library_fraction = 0.0`
  - downstream proof:
    clean in Verilator, Yosys `synth -noabc`, and the repo-owned
    Yosys with-ABC path.
- Full refreshed repo-owned Phase 4 rerun:
  - `cargo run --bin tool_matrix -- --out /tmp/anvil-tool-matrix-phase4-hierarchy-r11 --phase4-hierarchy-gate --yosys-mode both`
  - report:
    `/tmp/anvil-tool-matrix-phase4-hierarchy-r11/tool_matrix_report.json`
  - key facts:
    - `scenario_count = 21`
    - `modules_per_scenario = 4`
    - `total_modules = 84`
    - `artifact_kind = "design"`
    - `coverage_gaps = []`
    - `tool_summary.verilator_passed = 84`
    - `tool_summary.verilator_failed = 0`
    - `tool_summary.yosys_without_abc_passed = 84`
    - `tool_summary.yosys_without_abc_failed = 0`
    - `tool_summary.yosys_with_abc_passed = 84`
    - `tool_summary.yosys_with_abc_failed = 0`
    - hierarchy coverage facts:
      - `hierarchy_child_source_modes = ["library", "on-demand"]`
      - `saw_on_demand_child_sourcing = true`
      - `saw_recursive_hierarchy = true`
      - `saw_mixed_leaf_depth_hierarchy = true`
      - `saw_hierarchy_parent_composition = true`

**Impact**

- Phase 4 now has an explicit, user-facing child-sourcing axis instead
  of only the reusable-library story.
- Hierarchy quality numbers now distinguish "everything reused" from
  "every child fresh" directly, without requiring emitted-SV
  inspection.
- The repo-owned Phase 4 gate now proves that both sourcing modes are
  exercised cleanly on current HEAD.
- Phase 4 remains `in progress`. The remaining work is now narrower:
  width-demand-driven on-demand child synthesis, local parent state,
  and later hierarchy-aware identity.

**Files touched**

- [src/config.rs](/Users/richarddje/Documents/github/anvil/src/config.rs)
- [src/main.rs](/Users/richarddje/Documents/github/anvil/src/main.rs)
- [src/gen/hierarchy.rs](/Users/richarddje/Documents/github/anvil/src/gen/hierarchy.rs)
- [src/metrics.rs](/Users/richarddje/Documents/github/anvil/src/metrics.rs)
- [src/bin/tool_matrix.rs](/Users/richarddje/Documents/github/anvil/src/bin/tool_matrix.rs)
- [tests/pipeline.rs](/Users/richarddje/Documents/github/anvil/tests/pipeline.rs)
- [README.md](/Users/richarddje/Documents/github/anvil/README.md)
- [ROADMAP.md](/Users/richarddje/Documents/github/anvil/ROADMAP.md)
- [USER_GUIDE.md](/Users/richarddje/Documents/github/anvil/USER_GUIDE.md)
- [DEVELOPMENT_NOTES.md](/Users/richarddje/Documents/github/anvil/DEVELOPMENT_NOTES.md)
- [CODEBASE_ANALYSIS.md](/Users/richarddje/Documents/github/anvil/CODEBASE_ANALYSIS.md)
- [book/src/hierarchy.md](/Users/richarddje/Documents/github/anvil/book/src/hierarchy.md)
- [book/src/architecture.md](/Users/richarddje/Documents/github/anvil/book/src/architecture.md)
- [book/src/knobs.md](/Users/richarddje/Documents/github/anvil/book/src/knobs.md)
- [CHANGES.md](/Users/richarddje/Documents/github/anvil/CHANGES.md)
- [MEMORY.md](/Users/richarddje/Documents/github/anvil/MEMORY.md)

## 2026-04-23-boot4 — Close mixed-depth Phase 4 hierarchy gate cleanly

**Landed as:** `f706232`

**What changed**

- [src/bin/tool_matrix.rs](/Users/richarddje/Documents/github/anvil/src/bin/tool_matrix.rs)
  now folds the mixed-depth recursive hierarchy axis into the
  repo-owned `--phase4-hierarchy-gate` matrix instead of leaving it as
  focused-smoke-only evidence.
- The Phase 4 hierarchy scenario set now includes a dedicated mixed
  recursive profile per construction strategy:
  bounded hierarchy depth range `[2:3]` with exact child-instance count
  `2`, chosen specifically so the realized tree contains both shallow
  and deep branches.
- Phase 4 coverage facts are now stronger and more honest:
  the report records whether the matrix actually realized mixed
  shallow/deep leaf depths (`saw_mixed_leaf_depth_hierarchy`), and the
  coverage gate now requires both the requested open interval
  `hierarchy_depths = ["2:3"]` and an actual mixed-depth realization.
- The Phase 4 gate's minimum total design budget was raised from `48` to
  `60` so that after the matrix grew from `15` to `18` scenarios it
  still proves `4` designs per scenario instead of silently dropping to
  `3`.
- The repo-owned Phase 4 closure artifact is now refreshed from `r9` to
  `r10`, and the live docs/book now point at the new report instead of
  still describing mixed-depth recursion as a pending closure step.

**Why**

- The previous slice landed real mixed-depth recursive hierarchy in the
  generator and proved it with the focused smoke at
  `/tmp/anvil-hier-mixed-depth-smoke-r1/manifest.json`.
- That left one honest gap: the repo-owned Phase 4 gate was still
  banking the older representative hierarchy surface and therefore
  could not prove the new mixed-depth capability from a closure report.
- The right next task was to close that gap at the matrix layer, not to
  leave the repo split between "current code" and "current banked gate."

**Proof**

- Focused `tool_matrix` regressions:
  - `cargo test --bin tool_matrix phase4_hierarchy_gate_raises_designs_per_scenario_for_matrix`
  - `cargo test --bin tool_matrix phase4_hierarchy_matrix_covers_wrapper_and_recursive_profiles`
  - `cargo test --bin tool_matrix phase4_hierarchy_coverage_requires_design_facts`
- Full refreshed repo-owned Phase 4 rerun:
  - `cargo run --bin tool_matrix -- --out /tmp/anvil-tool-matrix-phase4-hierarchy-r10 --phase4-hierarchy-gate --yosys-mode both`
  - report:
    `/tmp/anvil-tool-matrix-phase4-hierarchy-r10/tool_matrix_report.json`
  - key facts:
    - `scenario_count = 18`
    - `modules_per_scenario = 4`
    - `total_modules = 72`
    - `artifact_kind = "design"`
    - `coverage_gaps = []`
    - `tool_summary.verilator_passed = 72`
    - `tool_summary.verilator_failed = 0`
    - `tool_summary.yosys_without_abc_passed = 72`
    - `tool_summary.yosys_without_abc_failed = 0`
    - `tool_summary.yosys_with_abc_passed = 72`
    - `tool_summary.yosys_with_abc_failed = 0`
    - hierarchy coverage facts:
      - `hierarchy_depths = ["1", "2", "2:3"]`
      - `hierarchy_leaf_module_counts = ["0", "2", "4"]`
      - `hierarchy_child_instance_counts = ["1:3", "2", "2:3", "4"]`
      - `hierarchy_child_instance_override_profiles = ["0=4:4,1=2:2"]`
      - `saw_recursive_hierarchy = true`
      - `saw_per_depth_branching_metrics = true`
      - `saw_mixed_leaf_depth_hierarchy = true`
      - `saw_hierarchy_parent_composition = true`

**Impact**

- The repo-owned Phase 4 closure artifact now matches current hierarchy
  reality instead of lagging behind it.
- Mixed-depth recursion is no longer merely "focused clean proof"; it is
  a banked, gated part of the representative Phase 4 matrix.
- Phase 4 remains `in progress`. The next honest work is deeper
  hierarchy capability beyond the current bank: on-demand child
  sourcing, local parent state, and later hierarchy-aware identity.

**Files touched**

- [src/bin/tool_matrix.rs](/Users/richarddje/Documents/github/anvil/src/bin/tool_matrix.rs)
- [README.md](/Users/richarddje/Documents/github/anvil/README.md)
- [ROADMAP.md](/Users/richarddje/Documents/github/anvil/ROADMAP.md)
- [USER_GUIDE.md](/Users/richarddje/Documents/github/anvil/USER_GUIDE.md)
- [DEVELOPMENT_NOTES.md](/Users/richarddje/Documents/github/anvil/DEVELOPMENT_NOTES.md)
- [CODEBASE_ANALYSIS.md](/Users/richarddje/Documents/github/anvil/CODEBASE_ANALYSIS.md)
- [book/src/hierarchy.md](/Users/richarddje/Documents/github/anvil/book/src/hierarchy.md)
- [book/src/architecture.md](/Users/richarddje/Documents/github/anvil/book/src/architecture.md)
- [CHANGES.md](/Users/richarddje/Documents/github/anvil/CHANGES.md)
- [MEMORY.md](/Users/richarddje/Documents/github/anvil/MEMORY.md)

## 2026-04-23-boot3 — Land mixed-depth recursive hierarchy planning

**Landed as:** `8f6abfc`

**What changed**

- [src/gen/hierarchy.rs](/Users/richarddje/Documents/github/anvil/src/gen/hierarchy.rs)
  no longer collapses bounded recursive hierarchy depth to one exact
  scalar for the whole design. The recursive planner now carries
  subtree-local `[min_depth, max_depth]` intervals and can realize both
  shallow and deep branches inside one legal tree when the requested
  range is open and the branching structure allows it.
- The recursive planner now deliberately forces mixed child depth
  realization at the right seam: when a parent has at least two child
  slots and the allowed child interval is genuinely open, one child can
  realize the shallowest still-legal descendant depth and another can
  realize the deepest still-legal descendant depth, while all leaves
  still stay inside the requested global bounds.
- [src/metrics.rs](/Users/richarddje/Documents/github/anvil/src/metrics.rs)
  now reports `leaf_module_occurrences_by_depth`, so mixed shallow/deep
  recursion can be trusted numerically from the manifest rather than by
  reading emitted `.sv`.
- [tests/pipeline.rs](/Users/richarddje/Documents/github/anvil/tests/pipeline.rs)
  and the new hierarchy unit tests now prove the mixed-depth contract
  directly: realized leaf depths stay inside the requested interval, and
  the depth histogram reflects the intended shallow/deep split.
- The live docs and mdBook now say the same thing plainly: bounded
  recursive hierarchy no longer means "pick one exact realized depth for
  the whole design"; it means "keep leaves inside the requested
  interval, and mix depths when the structure allows it."

**Why**

- The user asked for hierarchy depth to be controllable over a
  reasonable `[min:max]` interval and for the resulting composition
  quality to be measurable from trustworthy metrics rather than visual
  inspection.
- The previous recursive slice had only a foothold version of that:
  depth was bounded, but the planner still collapsed the whole design to
  one exact realized depth. That meant the interval was acting more like
  a lottery than a true structural range.
- The right fix was to keep the depth interval live all the way down the
  recursive planner and expose the realized result numerically.

**Proof**

- Focused hierarchy regressions:
  - `cargo test recursive_range_generation_can_mix_shallow_and_deep_branches --lib`
  - `cargo test design_metrics_capture_mixed_leaf_depths --lib`
  - `cargo test design_metrics_capture_recursive_depth_and_branching --lib`
  - `cargo test generates_valid_recursive_hierarchy_designs_with_bounded_shape --test pipeline`
  - `cargo test generates_valid_recursive_hierarchy_designs_with_mixed_leaf_depths --test pipeline`
- Focused emitted-design smoke:
  - `cargo run --bin anvil -- --seed 19 --count 1 --out /tmp/anvil-hier-mixed-depth-smoke-r1 --min-hierarchy-depth 2 --max-hierarchy-depth 3 --min-child-instances-per-module 2 --max-child-instances-per-module 2`
  - artifact:
    `/tmp/anvil-hier-mixed-depth-smoke-r1/manifest.json`
  - key facts:
    - `realized_min_leaf_depth = 2`
    - `realized_max_leaf_depth = 3`
    - `leaf_module_occurrences_by_depth = {"2": 2, "3": 4}`
    - `avg_child_instances_by_parent_depth = {"0": 2.0, "1": 2.0, "2": 2.0}`
    - `hierarchy_parent_composed_outputs = 40`
    - `top_parent_composed_outputs = 14`
  - downstream tools:
    - Verilator clean
    - Yosys `synth -noabc` clean
    - repo-owned Yosys ABC path clean

**Impact**

- Recursive hierarchy ranges now behave like real structural ranges
  rather than a single-depth chooser.
- The repo has a trustworthy metric for mixed-depth realization, so the
  user no longer has to inspect emitted RTL to verify that shallow and
  deep branches really appeared.
- Phase 4 remains `in progress`. The next honest closure step is to
  absorb this mixed-depth axis into the repo-owned Phase 4 hierarchy
  gate, then continue with on-demand child sourcing, local parent state,
  and hierarchy-aware identity.

**Files touched**

- [src/gen/hierarchy.rs](/Users/richarddje/Documents/github/anvil/src/gen/hierarchy.rs)
- [src/metrics.rs](/Users/richarddje/Documents/github/anvil/src/metrics.rs)
- [tests/pipeline.rs](/Users/richarddje/Documents/github/anvil/tests/pipeline.rs)
- [README.md](/Users/richarddje/Documents/github/anvil/README.md)
- [ROADMAP.md](/Users/richarddje/Documents/github/anvil/ROADMAP.md)
- [USER_GUIDE.md](/Users/richarddje/Documents/github/anvil/USER_GUIDE.md)
- [DEVELOPMENT_NOTES.md](/Users/richarddje/Documents/github/anvil/DEVELOPMENT_NOTES.md)
- [CODEBASE_ANALYSIS.md](/Users/richarddje/Documents/github/anvil/CODEBASE_ANALYSIS.md)
- [book/src/hierarchy.md](/Users/richarddje/Documents/github/anvil/book/src/hierarchy.md)
- [book/src/architecture.md](/Users/richarddje/Documents/github/anvil/book/src/architecture.md)
- [book/src/knobs.md](/Users/richarddje/Documents/github/anvil/book/src/knobs.md)
- [CHANGES.md](/Users/richarddje/Documents/github/anvil/CHANGES.md)
- [MEMORY.md](/Users/richarddje/Documents/github/anvil/MEMORY.md)

## 2026-04-23-boot2 — Complete literal bootstrap pass and fix stale README closure note

**Landed as:** `ce4327d`

**What changed**

- [README.md](/Users/richarddje/Documents/github/anvil/README.md) no
  longer says the refreshed recursive Phase 4 hierarchy matrix closure
  is "the next honest closure step". That wording was stale after the
  banked `r9` report landed. The README now says what the repo already
  proves elsewhere: the recursive Phase 4 matrix is fully banked, and
  the next honest work is deeper hierarchy capability rather than
  another closure refresh.
- [MEMORY.md](/Users/richarddje/Documents/github/anvil/MEMORY.md) now
  records that a literal `SESSION_BOOTSTRAP.md` pass was rerun against
  current HEAD and that the only live-doc drift found was the stale
  README sentence above.

**Why**

- The user correctly called out that the previous bootstrap response had
  not fully executed `SESSION_BOOTSTRAP.md`.
- Re-running the full bootstrap pass across the live docs, the full
  mdBook, and the Rust workspace showed one real drift: the README was
  still talking as if the Phase 4 closure refresh had not happened yet,
  even though `/tmp/anvil-tool-matrix-phase4-hierarchy-r9/` is already
  the banked repo-owned artifact.

**Validation**

- Re-read all required live docs, the full mdBook, and every Rust
  source / test / example per `SESSION_BOOTSTRAP.md`.
- `cargo check --all-targets`
- `cargo test`
- `git --no-pager log -5 --oneline`
- `git --no-pager status --short`

**Impact**

- Bootstrap recovery is now truthful again: a cold-start reader will not
  be told that the Phase 4 closure refresh is still pending when it has
  already been banked.
- No code or roadmap state changed; this is a docs-recovery slice only.

**Files touched**

- [README.md](/Users/richarddje/Documents/github/anvil/README.md)
- [MEMORY.md](/Users/richarddje/Documents/github/anvil/MEMORY.md)
- [CHANGES.md](/Users/richarddje/Documents/github/anvil/CHANGES.md)

## 2026-04-23-1913 — Close refreshed recursive Phase 4 hierarchy gate cleanly

**Landed as:** `1bda5c7`

**What changed**

- [src/bin/tool_matrix.rs](/Users/richarddje/Documents/github/anvil/src/bin/tool_matrix.rs)
  now treats the repo-owned Phase 4 gate as a real hierarchy gate, not
  only a depth-1 wrapper baseline. The Phase 4 scenario matrix now
  covers:
  - legacy exact wrapper comb
  - legacy reuse-heavy wrapper seq
  - legacy under-instantiated wrapper comb
  - bounded recursive comb at exact depth `2` with fallback child range
    `[2:3]`
  - bounded recursive seq at exact depth `2` with fallback child range
    `[1:3]` and per-depth override profile `0=4:4,1=2:2`
- The Phase 4 coverage summary and gap logic now require the new
  hierarchy facts explicitly:
  - depth set includes both `1` and `2`
  - child-instance profiles include exact and ranged cases
  - the per-depth override profile is present
  - the matrix really emits recursive designs
  - the matrix really reports per-depth branching metrics
  - the matrix really emits top outputs composed above instance outputs
- `HierarchyFacts` are now derived from trusted `DesignMetrics` rather
  than top-wrapper-only heuristics, so reuse / under-instantiation facts
  stay truthful in recursive designs too.
- The recursive focused regression in
  [src/bin/tool_matrix.rs](/Users/richarddje/Documents/github/anvil/src/bin/tool_matrix.rs)
  now uses a deliberately tiny recursive profile. It still proves that
  hierarchy facts mirror design metrics, but it no longer burns time in
  a heavyweight sequential leaf shape that the assertion did not need.
- The Phase 4 sequential hierarchy scenarios now use a
  **hierarchy-focused sequential leaf profile** instead of reusing the
  fattest Phase 1 motif-heavy sequential leaf stress configuration.
  That keeps the hierarchy gate aimed at hierarchy structure,
  control-port propagation, parent composition, and recursive shape
  rather than accidentally turning it into a second leaf-stress gate.

**Why**

- The repo already had clean focused proofs for parent composition,
  bounded recursion, and per-depth branching, but the last fully banked
  Phase 4 report was still the older wrapper-baseline `r7` artifact.
- The first broadened rerun (`r8`) made the real runtime issue clear:
  the new gate logic was fine, but the recursive sequential hierarchy
  scenarios were over-coupled to the heaviest leaf sequential profile,
  so the proof was paying for a huge amount of downstream Yosys work
  unrelated to the hierarchy contract it was supposed to prove.
- The fix was to stabilize the gate at the right seam: keep the
  recursive/per-depth hierarchy surfaces in the matrix, but use a
  sequential leaf profile sized for hierarchy proof instead of
  Phase-1-scale leaf stress.

**Proof**

- Focused `tool_matrix` regressions:
  - `cargo test --bin tool_matrix phase4_hierarchy`
  - `cargo test --bin tool_matrix recursive_hierarchy_facts_follow_design_metrics`
- Full refreshed repo-owned Phase 4 rerun:
  - `cargo run --bin tool_matrix -- --out /tmp/anvil-tool-matrix-phase4-hierarchy-r9 --phase4-hierarchy-gate --yosys-mode both`
  - report:
    `/tmp/anvil-tool-matrix-phase4-hierarchy-r9/tool_matrix_report.json`
  - key facts:
    - `scenario_count = 15`
    - `modules_per_scenario = 4`
    - `total_modules = 60`
    - `artifact_kind = "design"`
    - `coverage_gaps = []`
    - `tool_summary.verilator_passed = 60`
    - `tool_summary.verilator_failed = 0`
    - `tool_summary.yosys_without_abc_passed = 60`
    - `tool_summary.yosys_without_abc_failed = 0`
    - `tool_summary.yosys_with_abc_passed = 60`
    - `tool_summary.yosys_with_abc_failed = 0`
    - hierarchy coverage facts:
      - `hierarchy_depths = ["1", "2"]`
      - `hierarchy_leaf_module_counts = ["0", "2", "4"]`
      - `hierarchy_child_instance_counts = ["1:3", "2", "2:3", "4"]`
      - `hierarchy_child_instance_override_profiles = ["0=4:4,1=2:2"]`
      - `saw_recursive_hierarchy = true`
      - `saw_per_depth_branching_metrics = true`
      - `saw_hierarchy_parent_composition = true`
      - `saw_reused_child_definition = true`
      - `saw_underinstantiated_library = true`

**Impact**

- Phase 4 now had a fully banked repo-owned closure artifact for the
  then-current hierarchy surface, not only the older wrapper baseline.
- The hierarchy gate is materially more trustworthy and more stable:
  it proves wrapper exact/reuse/under-instantiation, recursive depth,
  per-depth branching, and parent composition without dragging the
  proof through oversized leaf sequential stress.
- Phase labels do **not** change in this slice. Phase 4 remains
  `in progress`; the next honest work is deeper mixed-depth recursion,
  on-demand child sourcing as a first-class axis, local parent state,
  and eventual hierarchy-aware identity.

**Files touched**

- `src/bin/tool_matrix.rs`
- `README.md`
- `USER_GUIDE.md`
- `ROADMAP.md`
- `DEVELOPMENT_NOTES.md`
- `CODEBASE_ANALYSIS.md`
- `book/src/hierarchy.md`
- `book/src/architecture.md`
- `CHANGES.md`
- `MEMORY.md`

## 2026-04-23-1735 — Land bounded recursive hierarchy depth profiles and per-depth metrics

**Landed as:** `134e889`

**What changed**

- [src/config.rs](/Users/richarddje/Documents/github/anvil/src/config.rs)
  now exposes two separate recursive-hierarchy control layers:
  - the existing global fallback bounds
    `min_hierarchy_depth..=max_hierarchy_depth` and
    `min_child_instances_per_module..=max_child_instances_per_module`
  - a new optional per-parent-depth override map
    `child_instances_per_module_by_depth`
  Validation is correspondingly stricter: legacy exact wrapper knobs and
  recursive-range knobs are still mutually exclusive; per-depth
  overrides must themselves satisfy `1 <= min <= max`; they require the
  global child-instance fallback range to be present; and they may only
  target realized internal parent depths inside
  `[0, max_hierarchy_depth - 1]`.
- [src/main.rs](/Users/richarddje/Documents/github/anvil/src/main.rs)
  now exposes the new CLI surface:
  `--child-instances-per-depth DEPTH=MIN:MAX`
  (repeatable, depth `0` = top, depth `1` = the top's direct children,
  and so on). CLI parsing now lifts that into the config override map.
- [src/gen/hierarchy.rs](/Users/richarddje/Documents/github/anvil/src/gen/hierarchy.rs)
  now consults the effective child-instance range **per parent depth**
  while recursively planning the hierarchy tree. The old global range is
  still the fallback; the new per-depth overrides win where present.
- [src/metrics.rs](/Users/richarddje/Documents/github/anvil/src/metrics.rs)
  now reports the realized branching profile per parent depth, not just
  global branching summaries. New design metrics:
  - `avg_child_instances_by_parent_depth`
  - `min_child_instances_by_parent_depth`
  - `max_child_instances_by_parent_depth`
- [tests/pipeline.rs](/Users/richarddje/Documents/github/anvil/tests/pipeline.rs)
  now proves the new surface end to end across multiple seeds: a
  recursive design with fallback range `[1:3]`, top override `0=4:4`,
  and depth-1 override `1=2:2` validates and lands the requested
  realized shape numerically.

**Why**

- The user asked for hierarchy depth to be controllable by a reasonable
  `[min:max]` interval and, separately, for the number of instances per
  module to be controllable **at a certain level** in the hierarchy.
- The first recursive slice already had the global bounded interval, but
  not the level-specific steering. That was the missing seam.
- Trustworthy hierarchy quality also requires metrics that answer
  "what did level 0 do?" and "what did level 1 do?" numerically, rather
  than asking the user to inspect emitted `.sv`.

**Proof**

- Focused unit / integration regressions:
  - `cargo test --lib`
  - `cargo test --bin anvil newly_exposed_cli_knobs_round_trip_into_overrides`
  - `cargo test --test pipeline generates_valid_recursive_hierarchy_designs_with_per_depth_branching_controls`
- Focused emitted-design smoke:
  - `cargo run --bin anvil -- --seed 23 --count 1 --out /tmp/anvil-hier-depth-profile-smoke-r1 --min-hierarchy-depth 2 --max-hierarchy-depth 2 --min-child-instances-per-module 1 --max-child-instances-per-module 3 --child-instances-per-depth 0=4:4 --child-instances-per-depth 1=2:2`
  - manifest:
    `/tmp/anvil-hier-depth-profile-smoke-r1/manifest.json`
  - key design metrics from that manifest:
    - `realized_min_leaf_depth = 2`
    - `realized_max_leaf_depth = 2`
    - `avg_child_instances_by_parent_depth = {"0": 4.0, "1": 2.0}`
    - `min_child_instances_by_parent_depth = {"0": 4, "1": 2}`
    - `max_child_instances_by_parent_depth = {"0": 4, "1": 2}`
    - `hierarchy_parent_composed_outputs = 36`
    - `top_parent_composed_outputs = 18`
- Downstream proof on that emitted design:
  - Verilator:
    `verilator --lint-only --top-module mod_23_0007 /tmp/anvil-hier-depth-profile-smoke-r1/*.sv`
  - Yosys without ABC:
    `yosys -q -p "read_verilog -sv /tmp/anvil-hier-depth-profile-smoke-r1/*.sv; hierarchy -top mod_23_0007; synth -noabc; check"`
  - Yosys with ABC:
    `yosys -q -p "read_verilog -sv /tmp/anvil-hier-depth-profile-smoke-r1/*.sv; hierarchy -top mod_23_0007; synth -noabc; abc -fast; opt -fast; stat; check"`

**Impact**

- Recursive hierarchy is no longer only "bounded depth + one global
  branching range." It now supports depth-specific branching control in
  a way that is explicit in both CLI/config and manifests.
- The hierarchy metrics surface is stronger: the user can now trust the
  realized per-level branching from the manifest itself without opening
  the emitted RTL.
- Phase labels do **not** change in this slice. Phase 4 remains
  `in progress`; the next honest work is still refreshing the full
  repo-owned Phase 4 matrix on the newer recursive code and then moving
  deeper into mixed-depth recursion / on-demand child sourcing.

**Files touched**

- `src/config.rs`
- `src/main.rs`
- `src/gen/mod.rs`
- `src/gen/hierarchy.rs`
- `src/metrics.rs`
- `tests/pipeline.rs`
- `README.md`
- `USER_GUIDE.md`
- `ROADMAP.md`
- `DEVELOPMENT_NOTES.md`
- `CODEBASE_ANALYSIS.md`
- `book/src/architecture.md`
- `book/src/hierarchy.md`
- `book/src/knobs.md`
- `CHANGES.md`
- `MEMORY.md`

## 2026-04-23-1557 — Land parent-composed hierarchy tops and trustworthy composition metrics

**Landed as:** `28713a0`

**What changed**

- [src/ir/types.rs](/Users/richarddje/Documents/github/anvil/src/ir/types.rs)
  upgrades `DepSet` from a flat tagged-integer trick to a typed leaf-set
  that now distinguishes:
  - primary-input leaves,
  - local flop-Q virtual leaves, and
  - instantiated child-output leaves.
  That makes `Node::InstanceOutput` a real dep-bearing leaf variable for
  generator-side cone construction instead of looking like an empty-dep
  pseudo-constant.
- [src/gen/cone.rs](/Users/richarddje/Documents/github/anvil/src/gen/cone.rs)
  now propagates that identity in `node_deps`, so parent-side cone
  construction can reuse the existing leaf-kernel builder without
  misclassifying child outputs as trivial roots.
- [src/gen/module.rs](/Users/richarddje/Documents/github/anvil/src/gen/module.rs)
  now exposes a shared `finalize_generated_module` path. That means the
  new hierarchy top goes through the same settled-graph cleanup,
  bounded merge, constant-root repair, compaction, shrink/prune, and
  liveness discipline as ordinary generated modules.
- [src/gen/hierarchy.rs](/Users/richarddje/Documents/github/anvil/src/gen/hierarchy.rs)
  is the real Phase 4 step: the depth-1 top is no longer only a
  pass-through wrapper. It now:
  - pre-generates the leaf library as before,
  - instantiates the planned child set as before,
  - seeds a parent signal pool from child `InstanceOutput` leaves,
  - builds top-output cones from those child outputs with the current
    construction strategy, and
  - promotes any surviving direct child-output drive to parent logic
    when a second child-output source exists.
  The current slice stays honest: parent-local flops are still disabled,
  so this is a first **combinational** parent-composition layer, not the
  full recursive hierarchy destination yet.
- The first real hierarchy-regression bug shaken loose by that work was
  in [src/ir/compact.rs](/Users/richarddje/Documents/github/anvil/src/ir/compact.rs):
  `compact_node_ids` was not treating instance input bindings as live
  holders and was not remapping them through the compacted `NodeId`
  space. That is now fixed and pinned by a regression.
- The second old wrapper-era assumption was in
  [src/ir/validate.rs](/Users/richarddje/Documents/github/anvil/src/ir/validate.rs):
  design validation no longer requires every child output to be exposed
  exactly once. The right rule is narrower: any referenced child output
  node must name a real child output port with the right width, while
  genuinely unused child outputs are allowed.
- The third old wrapper-era assumption was in
  [src/emit/sv.rs](/Users/richarddje/Documents/github/anvil/src/emit/sv.rs):
  unused child outputs are now emitted as explicit unconnected ports
  (`.port()`) instead of panicking on the assumption that every child
  output must have a corresponding `Node::InstanceOutput` wire.
- [src/metrics.rs](/Users/richarddje/Documents/github/anvil/src/metrics.rs)
  now quantifies hierarchy composition directly, without requiring any
  `.sv` inspection:
  - `top_direct_instance_output_drives`
  - `top_parent_composed_outputs`
  - `top_outputs_reaching_instance_outputs`
  - `top_outputs_without_instance_outputs`
  - `top_instance_output_dependency_fraction`
  - `top_parent_composed_output_fraction`
  - `avg_instance_output_support_per_top_output`
  - `max_instance_output_support_per_top_output`

**Why**

- The next honest Phase 4 step was to move beyond wrapper-only
  composition and let parent outputs become real functions of child
  outputs.
- That only becomes trustworthy if the repo can prove the result
  numerically rather than asking the user to visually inspect emitted
  RTL.
- The resulting work also had to fix the actual structural bugs that a
  richer hierarchy slice exposed, rather than hiding them behind local
  workarounds.

**Proof**

- Focused hierarchy composition smoke:
  - `cargo run --bin anvil -- --seed 19 --out /tmp/anvil-hier-parent-compose-smoke-r1 --hierarchy-depth 1 --num-leaf-modules 2 --num-child-instances 4`
  - manifest:
    `/tmp/anvil-hier-parent-compose-smoke-r1/manifest.json`
  - key design metrics from that manifest:
    - `top_parent_composed_outputs = 10`
    - `top_direct_instance_output_drives = 0`
    - `top_outputs_reaching_instance_outputs = 10`
    - `top_outputs_without_instance_outputs = 0`
    - `top_instance_output_dependency_fraction = 1.0`
    - `avg_instance_output_support_per_top_output = 2.5`
- Downstream proof on that emitted design:
  - Verilator: `verilator --lint-only --top-module mod_19_0002 /tmp/anvil-hier-parent-compose-smoke-r1/*.sv`
  - Yosys without ABC:
    `yosys -q -p "read_verilog -sv /tmp/anvil-hier-parent-compose-smoke-r1/*.sv; hierarchy -top mod_19_0002; synth -noabc; check"`
  - Yosys with ABC:
    `yosys -q -p "read_verilog -sv /tmp/anvil-hier-parent-compose-smoke-r1/*.sv; hierarchy -top mod_19_0002; synth -noabc; abc -fast; opt -fast; stat; check"`
- New focused regressions:
  - `cargo test depth1_parent_outputs_depend_on_child_instance_outputs --test pipeline`
  - `cargo test compact_remaps_instance_input_bindings --lib`
  - `cargo test hierarchy_emits_unconnected_child_outputs_when_unused --lib`
  - `cargo test accepts_design_with_unreferenced_child_output --lib`
  - `cargo test design_metrics_capture_parent_side_composition --lib`
- Full hygiene gate on the final tree:
  - `cargo check --all-targets`
  - `cargo test`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo fmt --all --check`
  - `mdbook build book`

**Impact**

- Phase 4 is no longer only a wrapper/pass-through story on current
  HEAD; the top can now build a real combinational parent layer over
  child outputs.
- The repo now has trustworthy metrics for that structure, so hierarchy
  quality can be judged from manifests and reports instead of eyeballing
  `.sv`.
- The older `/tmp/anvil-tool-matrix-phase4-hierarchy-r7` report remains
  valuable, but it is now a **wrapper-baseline** closure artifact. The
  next honest closure step is to rerun the full Phase 4 matrix on this
  newer parent-composition code.
- Roadmap phase labels do **not** change in this slice: Phase 4 stays
  `in progress`.

**Files touched**

- `src/ir/types.rs`
- `src/gen/cone.rs`
- `src/gen/module.rs`
- `src/gen/hierarchy.rs`
- `src/ir/compact.rs`
- `src/ir/validate.rs`
- `src/emit/sv.rs`
- `src/metrics.rs`
- `tests/pipeline.rs`
- `CHANGES.md`
- `MEMORY.md`
- `README.md`
- `USER_GUIDE.md`
- `ROADMAP.md`
- `DEVELOPMENT_NOTES.md`
- `CODEBASE_ANALYSIS.md`
- `book/src/hierarchy.md`
- `book/src/architecture.md`
- `book/src/structural-rules.md`

## 2026-04-23-1507 — Close refreshed Phase 4 hierarchy matrix cleanly

**Landed as:** `13ef73e`

**What changed**

- No Rust source changed in this slice. The work was a fresh full rerun
  of the broadened repo-owned Phase 4 hierarchy gate on current HEAD:
  - `cargo run --bin tool_matrix -- --out /tmp/anvil-tool-matrix-phase4-hierarchy-r7 --phase4-hierarchy-gate --yosys-mode both`
- The saved report at
  `/tmp/anvil-tool-matrix-phase4-hierarchy-r7/tool_matrix_report.json`
  is now the current repo-owned closure artifact for the live wrapper
  slice. It proves the widened exact / reuse / under-instantiation
  matrix directly:
  - `12` scenarios
  - `4` designs/scenario
  - `48` total designs
  - `artifact_kind = "design"`
  - `coverage_gaps = []`
  - `Verilator pass/fail = 48/0`
  - `Yosys without-abc pass/fail = 48/0`
  - `Yosys with-abc pass/fail = 48/0`
- The report also banks the broader wrapper-coverage facts that matter
  for trust:
  - `hierarchy_child_instance_counts = ["2", "4"]`
  - `saw_reused_child_definition = true`
  - `saw_underinstantiated_library = true`
  - `saw_hierarchy_design = true`
  - `saw_multifile_design = true`
- The old runtime suspicion is resolved more honestly now. The heavy
  `*_hier4_inst4_seq` corners are genuinely expensive because they
  elaborate/synthesize tiny wrapper tops over very large sequential
  child libraries, but they do close cleanly. The root cause was
  downstream cost, not malformed hierarchy output.

**Why**

- After `num_child_instances`, reuse, under-instantiation, and
  design-level hierarchy metrics landed, the repo truth still said that
  the broadened Phase 4 matrix was only proven by focused smokes. That
  was no longer acceptable; the widened matrix needed the same kind of
  repo-owned clean closure artifact as Phases 1-3.

**Proof**

- Fresh full gate:
  - `cargo run --bin tool_matrix -- --out /tmp/anvil-tool-matrix-phase4-hierarchy-r7 --phase4-hierarchy-gate --yosys-mode both`
  - final report:
    `/tmp/anvil-tool-matrix-phase4-hierarchy-r7/tool_matrix_report.json`
- Full hygiene gate on the final tree:
  - `cargo check --all-targets`
  - `cargo test`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo fmt --all --check`
  - `mdbook build book`

**Impact**

- The refreshed Phase 4 wrapper matrix is now banked as a real
  repo-owned closure artifact, not just a set of focused smokes plus an
  interrupted partial rerun.
- Phase 4 still stays `in progress`, because the phase is broader than
  the current wrapper slice: parent-side cone construction from
  instance outputs, depth > 1, on-demand child sourcing, and eventual
  hierarchy-aware identity are still open.

**Files touched**

- `CHANGES.md`
- `MEMORY.md`
- `DEVELOPMENT_NOTES.md`
- `ROADMAP.md`
- `README.md`
- `USER_GUIDE.md`
- `CODEBASE_ANALYSIS.md`
- `book/src/hierarchy.md`
- `book/src/architecture.md`

## 2026-04-23-0210 — Land trustworthy hierarchy design metrics

**Landed as:** `8d7795d`

**What changed**

- [src/metrics.rs](/Users/richarddje/Documents/github/anvil/src/metrics.rs)
  now exposes a real `DesignMetrics` walker alongside the existing
  per-module `Metrics` walker. For the current Phase 4 wrapper slice it
  records exact composition facts instead of forcing anyone to inspect
  emitted `.sv` by hand:
  - library size vs instantiated child count;
  - unique-instantiated-module count, unused-leaf count, and reused
    instance slots;
  - normalized composition ratios
    (`library_coverage_fraction`, `unused_library_fraction`,
    `instance_reuse_fraction`, `instance_to_library_ratio`);
  - top-interface shape (`top_inputs`, `top_data_inputs`,
    `top_clock_inputs`, `top_reset_inputs`, `top_outputs`);
  - control fanout to child instances;
  - weighted child-interface load and weighted child complexity; and
  - a per-definition instantiation histogram.
- [src/main.rs](/Users/richarddje/Documents/github/anvil/src/main.rs)
  now threads those design metrics through the user-facing hierarchy
  output path:
  - `--metrics` prints a design-level JSON block in hierarchy mode; and
  - hierarchy `manifest.json` entries now embed `metrics` per design,
    not only per module.
- [src/bin/tool_matrix.rs](/Users/richarddje/Documents/github/anvil/src/bin/tool_matrix.rs)
  now carries `DesignMetrics` in design reports, design checkpoints,
  resume upgrades, and hierarchy scenario manifests. Old hierarchy
  checkpoints stay readable via `#[serde(default)]`, and resume now
  refreshes both per-module and per-design metrics locally when the
  emitted `.sv` is still byte-stable.
- The proof run exposed two real Phase 4 source-of-truth bugs that
  would have made the new metrics untrustworthy if left alone:
  - [src/gen/hierarchy.rs](/Users/richarddje/Documents/github/anvil/src/gen/hierarchy.rs)
    now tags the wrapper's shared `clk` / `rst_n` ports as
    `Module.clock` / `Module.reset`;
  - [src/ir/validate.rs](/Users/richarddje/Documents/github/anvil/src/ir/validate.rs),
    [src/metrics.rs](/Users/richarddje/Documents/github/anvil/src/metrics.rs),
    and [src/emit/sv.rs](/Users/richarddje/Documents/github/anvil/src/emit/sv.rs)
    now all resolve control-port visibility from the same design-aware
    rule instead of each relying on a looser local proxy; and
  - [src/ir/types.rs](/Users/richarddje/Documents/github/anvil/src/ir/types.rs)
    now makes the control-port rule explicit: a module emits `clk` /
    `rst_n` iff it carries sequential state locally or through
    instantiated descendants. Pure comb-only modules stay control-free;
    sequential wrappers keep those ports visible all the way up the
    instantiated ancestor chain.
- Regression coverage now pins the whole story:
  - new `metrics.rs` tests for reuse and under-instantiation design
    metrics;
  - new `tool_matrix` tests for design-report and manifest embedding of
    hierarchy metrics;
  - a new hierarchy generator test proving wrapper tops tag shared
    control ports; and
  - new emitter + IR tests proving both sides of the boundary rule:
    pure comb-only wrappers omit `clk` / `rst_n`, while wrapper
    ancestors with sequential descendants keep them visible.

**Why**

- The user made the requirement explicit: hierarchy quality must be
  measurable from trusted numbers, not by visual inspection of emitted
  `.sv`.
- "Hierarchy facts" alone (`reused_child_definition`,
  `underinstantiated_library`) were too coarse. We needed exact,
  machine-checkable composition metrics in manifests and reports.
- The top-control tagging/emission bugs were root-cause issues. The
  design metrics smoke found them because the first trustworthy numbers
  must agree with the emitted hierarchy surface and downstream tools.

**Proof**

- Targeted tests:
  - `cargo test design_metrics_capture_reused_child_definitions --lib`
  - `cargo test design_metrics_capture_underinstantiated_library --lib`
  - `cargo test wrapper_top_tags_shared_clock_and_reset_ports --lib`
  - `cargo test hierarchy_wrapper_emits_clk_rst_n_without_local_flops --lib`
  - `cargo test hierarchy_comb_only_wrapper_omits_clk_rst_n_even_if_tagged --lib`
  - `cargo test hierarchy_grandparent_emits_clk_rst_n_for_sequential_descendants --lib`
  - `cargo test sequential_descendants_keep_control_ports_visible --lib`
  - `cargo test comb_only_descendants_keep_control_ports_hidden --lib`
  - `cargo test --bin tool_matrix run_design_tools_reports_design_metrics`
  - `cargo test --bin tool_matrix design_manifest_embeds_design_metrics`
- Focused hierarchy metrics smoke at
  `/tmp/anvil-hier-metrics-smoke-r1`:
  - `cargo run --bin anvil -- --seed 41 --count 1 --out /tmp/anvil-hier-metrics-smoke-r1 --hierarchy-depth 1 --num-leaf-modules 3 --num-child-instances 5 --construction-strategy interleaved`
  - emitted `manifest.json` now carries per-design `metrics` with
    correct `top_clock_inputs = 1`, `top_reset_inputs = 1`,
    `clock_fanout_instances = 5`, `reset_fanout_instances = 5`,
    `instance_reuse_fraction = 0.4`, and
    `library_coverage_fraction = 1.0`
  - clean in Verilator, Yosys `synth -noabc`, and the repo-owned
    ABC-enabled Yosys path
- Full hygiene gate on the final tree:
  - `cargo check --all-targets`
  - `cargo test`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo fmt --all --check`
  - `mdbook build book`

**Impact**

- Hierarchy quality is now reportable from exact design facts instead
  of ad hoc SV inspection.
- `manifest.json` and `tool_matrix` design reports now carry enough
  structure telemetry to reason about wrapper composition, reuse,
  under-instantiation, control distribution, and weighted child
  complexity directly.
- Phase labels do **not** change in this slice. Phase 4 stays
  `in progress`.

**Files touched**

- `src/metrics.rs`
- `src/main.rs`
- `src/bin/tool_matrix.rs`
- `src/gen/hierarchy.rs`
- `src/ir/types.rs`
- `src/ir/validate.rs`
- `src/emit/sv.rs`
- `CHANGES.md`
- `MEMORY.md`
- `DEVELOPMENT_NOTES.md`
- `README.md`
- `USER_GUIDE.md`
- `CODEBASE_ANALYSIS.md`
- `book/src/hierarchy.md`
- `book/src/architecture.md`
- `book/src/structural-rules.md`
- `book/src/sequential.md`

## 2026-04-23-0209 — Decouple hierarchy library size from instance count

**Landed as:** `2eebe58`

**What changed**

- The Phase 4 wrapper slice now separates **leaf-library size** from
  **instantiated child count**:
  - [src/config.rs](/Users/richarddje/Documents/github/anvil/src/config.rs)
    now has `num_child_instances`,
    `Config::effective_num_child_instances()`, and a validation error
    for `num_child_instances > 0` in leaf-only mode;
  - [src/main.rs](/Users/richarddje/Documents/github/anvil/src/main.rs)
    exposes the new `--num-child-instances` CLI flag;
  - `num_child_instances = 0` preserves the legacy wrapper behavior:
    instantiate every generated leaf definition exactly once.
- The hierarchy generator in
  [src/gen/hierarchy.rs](/Users/richarddje/Documents/github/anvil/src/gen/hierarchy.rs)
  now plans child instances separately from the pre-generated leaf
  library:
  - if the requested child count is smaller than the library, the top
    instantiates a shuffled subset without replacement;
  - if it matches, the top instantiates every definition once;
  - if it is larger, the top first covers every definition once, then
    fills the remaining slots by random reuse with replacement.
- The wrapper top still stays intentionally narrow and honest:
  shared `clk` / `rst_n` appear only if the actually-instantiated
  children need them, every child data input still becomes a top input,
  every instantiated child output still becomes a top output, and
  parent-side cone construction from instance outputs is still not live.
- [src/bin/tool_matrix.rs](/Users/richarddje/Documents/github/anvil/src/bin/tool_matrix.rs)
  now records hierarchy-specific design facts for the Phase 4 slice:
  child-instance counts, unique-instantiated-module counts, and boolean
  flags for both repeated child-definition reuse and
  under-instantiated-library cases. The Phase 4 scenario set now targets
  representative exact / reuse / under-instantiation profiles:
  `phase4_hier2_inst2_comb`, `phase4_hier2_inst4_seq`,
  `phase4_hier4_inst2_comb`, and `phase4_hier4_inst4_seq`.
- Focused hierarchy proof exposed a second real emitter bug in
  [src/emit/sv.rs](/Users/richarddje/Documents/github/anvil/src/emit/sv.rs):
  constant-backed slices were being rendered as illegal literal
  indexing, e.g. `20'h0[18:1]`. The emitter now folds constant slices
  directly to narrower legal literals instead.
- Regression coverage now pins the new surface:
  - config tests for `num_child_instances` validation and legacy `0`
    semantics;
  - CLI override coverage for `--num-child-instances`;
  - pipeline tests proving both repeated child-definition reuse and
    under-instantiation of the leaf library;
  - Phase 4 `tool_matrix` tests for the new representative profiles and
    hierarchy coverage facts; and
  - an emitter unit test proving constant slices render as folded
    literals instead of literal indexing.

**Why**

- The agreed Phase 4 direction is budget-driven wrapper planning, not a
  hard-coded "instantiate every generated leaf once" rule forever.
  Library size and instance count are different decisions and need to be
  exercised independently.
- Repeated instantiation of the same child definition and deliberate
  non-instantiation of some generated leaf definitions are both useful
  downstream stress surfaces for elaboration and pruning.
- The literal-slice bug was a real emitter defect surfaced by the new
  under-instantiation proof. The correct answer was to fix the emitted
  SV at the source, not weaken the proof or hide the shape.

**Proof**

- Targeted tests:
  - `cargo test depth1_wrapper --test pipeline`
  - `cargo test phase4_hierarchy --bin tool_matrix`
  - `cargo test newly_exposed_cli_knobs_round_trip_into_overrides --bin anvil`
  - `cargo test validate_rejects_child_instance_count_without_hierarchy --lib`
  - `cargo test constant_slice_renders_as_folded_literal_not_literal_indexing --lib`
- Focused downstream reuse smoke at `/tmp/anvil-hier-reuse-smoke-r1`:
  - `cargo run --bin anvil -- --seed 11 --count 1 --out /tmp/anvil-hier-reuse-smoke-r1 --hierarchy-depth 1 --num-leaf-modules 2 --num-child-instances 4 --construction-strategy interleaved`
  - clean in Verilator, Yosys `synth -noabc`, and the repo-owned
    ABC-enabled Yosys path
  - Yosys stats show real repeated instantiation of both generated leaf
    definitions under the top wrapper.
- Focused downstream under-instantiation smoke at
  `/tmp/anvil-hier-under-smoke-r2`:
  - `cargo run --bin anvil -- --seed 17 --count 1 --out /tmp/anvil-hier-under-smoke-r2 --hierarchy-depth 1 --num-leaf-modules 4 --num-child-instances 2 --construction-strategy interleaved`
  - clean in Verilator, Yosys `synth -noabc`, and the repo-owned
    ABC-enabled Yosys path
  - Yosys removes two unused generated leaf modules during hierarchy
    cleanup, proving that the wrapper can now under-instantiate the
    library cleanly.
- Full hygiene gate on the final tree:
  - `cargo check --all-targets`
  - `cargo test`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo fmt --all --check`
  - `mdbook build book`
- Fresh repo-owned Phase 4 rerun attempt at
  `/tmp/anvil-tool-matrix-phase4-hierarchy-r6` was intentionally
  stopped after **14** completed design checkpoints. The run stayed
  clean, but `seq_nodeid_egraph_phase4_hier4_inst4_seq` exposed the next
  runtime hotspot inside Yosys `synth -noabc`. So the broadened exact /
  reuse / under-instantiation surface is proven here by focused clean
  smokes plus tests, while the older
  `/tmp/anvil-tool-matrix-phase4-hierarchy-r3/tool_matrix_report.json`
  remains the last fully banked repo-owned Phase 4 closure artifact.

**Impact**

- Phase 4 wrapper hierarchy is materially more truthful now: the
  generator can represent exact library use, repeated child-definition
  reuse, and under-instantiation without smuggling those behaviors in
  through unrelated knobs.
- The Phase 4 hierarchy gate code now models the right wrapper planning
  axes directly, even though the fresh full rerun of that broadened
  matrix is still pending runtime closure.
- Phase labels do **not** change in this slice. Phase 4 stays
  `in progress`.

**Files touched**

- `src/config.rs`
- `src/main.rs`
- `src/gen/hierarchy.rs`
- `src/bin/tool_matrix.rs`
- `src/emit/sv.rs`
- `tests/pipeline.rs`
- `CHANGES.md`
- `MEMORY.md`
- `DEVELOPMENT_NOTES.md`
- `ROADMAP.md`
- `README.md`
- `USER_GUIDE.md`
- `CODEBASE_ANALYSIS.md`
- `book/src/architecture.md`
- `book/src/hierarchy.md`
- `book/src/knobs.md`

## 2026-04-23-0208 — Close Phase 4 wrapper hierarchy gate cleanly

**Landed as:** 7dae70a

**What changed**

- `tool_matrix` now has a first-class Phase 4 hierarchy gate in
  [src/bin/tool_matrix.rs](/Users/richarddje/Documents/github/anvil/src/bin/tool_matrix.rs):
  `--phase4-hierarchy-gate`, a dedicated hierarchy scenario set,
  design-aware execution/reporting/checkpointing, and hierarchy-specific
  coverage facts.
- The hierarchy gate runs a representative 12-scenario design matrix
  over the currently-live wrapper slice:
  - `construction_strategy ∈ {sequential, shuffled, interleaved}`
  - `num_leaf_modules ∈ {2, 4}`
  - child profile ∈ {comb-heavy, seq-heavy}
  - `identity_mode = node-id`
  - `factorization_level = e-graph`
- `tool_matrix` now treats hierarchy artifacts as real designs rather
  than pretending they are single-module runs with extra files around
  them. The new design-aware path validates `Design`s, writes
  `*.design-report.json` sidecars, records per-design file hashes and
  generator checkpoints, and runs Verilator/Yosys against the full file
  set with the declared top module.
- Hierarchy Yosys runs are now repo-owned and explicit the same way the
  leaf lanes are:
  - without ABC:
    `read_verilog -sv <files>; synth -top <top> -noabc; stat; check`
  - with ABC:
    `read_verilog -sv <files>; synth -top <top> -noabc; abc -fast; opt -fast; stat; check`
- The real emitter root-cause fix that made the gate close is in
  [src/emit/sv.rs](/Users/richarddje/Documents/github/anvil/src/emit/sv.rs):
  literal-backed procedural `for`-fold sources are now materialized
  through a packed temporary before indexed part-select use inside
  `always_comb`. This replaces both invalid forms that the hierarchy
  gate exposed: direct literal indexing
  (`24'h86899[(i * 12) +: 12]`) and the blanket parenthesized variant
  (`(signal)[(i * 12) +: 12]`).
- Regression coverage now proves both parts of the new surface:
  hierarchy-gate planning / coverage / resume tests in
  [src/bin/tool_matrix.rs](/Users/richarddje/Documents/github/anvil/src/bin/tool_matrix.rs)
  and a literal-backed `for`-fold emission regression in
  [src/emit/sv.rs](/Users/richarddje/Documents/github/anvil/src/emit/sv.rs).

**Why**

- A single hierarchy smoke was no longer enough. Phase 4 needed the
  same repo-owned closure discipline as Phases 1-3 so the wrapper slice
  could be recovered, replayed, and re-proved after interruptions.
- The gate had to operate on real designs, not just leaf modules,
  because the relevant downstream pressure here is multifile
  elaboration/synthesis with a declared top module and real child
  instances.
- The emitter bug was worth fixing at the source rather than weakening
  the gate: the hierarchy matrix exposed a real invalid-SV shape, and
  the correct answer was to emit legal procedural indexing for
  constant-backed fold sources.

**Proof**

- Dedicated hierarchy gate report at
  `/tmp/anvil-tool-matrix-phase4-hierarchy-r3/tool_matrix_report.json`:
  - `scenario_count = 12`
  - `modules_per_scenario = 4`
  - `total_modules = 48`
  - `artifact_kind = "design"`
  - `phase4_hierarchy_gate = true`
  - `coverage_gaps = []`
  - `Verilator pass/fail = 48/0`
  - `Yosys without-abc pass/fail = 48/0`
  - `Yosys with-abc pass/fail = 48/0`
- The saved coverage facts prove the intended wrapper-slice surface
  directly:
  - `hierarchy_depths = ["1"]`
  - `hierarchy_leaf_module_counts = ["2", "4"]`
  - `saw_hierarchy_design = true`
  - `saw_multifile_design = true`
  - `saw_instance_module = true`
  - `saw_instance_output_node = true`
- Focused emitter regression:
  `for_fold_materializes_literal_sources_before_part_select`

**Impact**

- The repo now has a real Phase 4 closure gate for the currently-live
  wrapper hierarchy slice.
- Resume / checkpoint infrastructure now extends cleanly from module
  artifacts to hierarchy designs.
- Phase 4 **does not** move to `done` yet. The wrapper slice is now
  closed with repo-owned evidence, but the phase still has real open
  work: parent-side cone construction from instance outputs, depth `> 1`
  recursion, on-demand child generation beside library sourcing, and
  future hierarchy-aware identity/factorization.

**Files touched**

- `src/bin/tool_matrix.rs`
- `src/emit/sv.rs`
- `CHANGES.md`
- `MEMORY.md`
- `DEVELOPMENT_NOTES.md`
- `ROADMAP.md`
- `README.md`
- `USER_GUIDE.md`
- `CODEBASE_ANALYSIS.md`
- `book/src/architecture.md`
- `book/src/hierarchy.md`

## 2026-04-23-0207 — Start Phase 4 with a real depth-1 hierarchy slice

**Landed as:** 747a3b3

**What changed**

- ANVIL now has a real hierarchy entry point in
  [src/gen/hierarchy.rs](/Users/richarddje/Documents/github/anvil/src/gen/hierarchy.rs)
  and a real `Generator::generate_design()` path wired from
  [src/gen/mod.rs](/Users/richarddje/Documents/github/anvil/src/gen/mod.rs).
  When `hierarchy_depth = 1`, generation now produces a `Design`
  containing:
  - a pre-generated library of leaf modules,
  - a real top wrapper module,
  - real `Instance` records in the top module, and
  - real `Node::InstanceOutput` nodes exposing child outputs.
- The first live Phase 4 slice is intentionally narrow and explicit:
  it is **depth-1 wrapper hierarchy only**. The top wrapper
  instantiates every generated leaf and exposes every child output as a
  top-level output. Parent-side cone construction from instance outputs
  is deliberately not live yet.
- The IR is now hierarchy-capable:
  [src/ir/types.rs](/Users/richarddje/Documents/github/anvil/src/ir/types.rs)
  adds `InstanceId`, `Module.instances`, `Instance`, and
  `Node::InstanceOutput`, plus helper methods that distinguish the
  emitted input surface from internal clock/reset bookkeeping.
- Validation now exists at both levels:
  [src/ir/validate.rs](/Users/richarddje/Documents/github/anvil/src/ir/validate.rs)
  still validates one module locally, and now also exposes
  `validate_design(&Design)`, which checks:
  - unique module names,
  - top-module presence,
  - local module validity,
  - child-module existence,
  - exact input-binding coverage and width matches,
  - exact output-exposure coverage and width matches, and
  - acyclic hierarchy.
- Emission is now design-aware:
  [src/emit/sv.rs](/Users/richarddje/Documents/github/anvil/src/emit/sv.rs)
  adds `to_sv_in_design()` and `to_sv_design()`, emits real child
  instances with named connections, and treats `Node::InstanceOutput`
  as a first-class emitted wire.
- The CLI now exposes the slice directly through
  [src/main.rs](/Users/richarddje/Documents/github/anvil/src/main.rs)
  and [src/config.rs](/Users/richarddje/Documents/github/anvil/src/config.rs):
  - `--hierarchy-depth`
  - `--num-leaf-modules`
  Config validation currently accepts only depth `0` or `1`, and
  `hierarchy_depth > 0` requires `num_leaf_modules >= 1`.
- Directory output in hierarchy mode now writes one `.sv` file per
  module in each generated design and records a `designs` array in
  `manifest.json` instead of the old flat `modules` list.
- Metrics now count hierarchy artifacts too:
  [src/metrics.rs](/Users/richarddje/Documents/github/anvil/src/metrics.rs)
  records `num_instances` and `num_instance_outputs`.
- Regression coverage now proves the new surface from code, not just
  from a smoke log:
  - depth-1 design generation / validation / emission in
    [tests/pipeline.rs](/Users/richarddje/Documents/github/anvil/tests/pipeline.rs)
  - design-level validator acceptance/rejection in
    [src/ir/validate.rs](/Users/richarddje/Documents/github/anvil/src/ir/validate.rs)
  - hierarchical emitter instance wiring in
    [src/emit/sv.rs](/Users/richarddje/Documents/github/anvil/src/emit/sv.rs)

**Why**

- Phase 3 is now closed with repo-owned evidence. The next honest step
  is to add hierarchy **above** the leaf kernel instead of smearing
  inter-module behavior into `generate_leaf_module`.
- The first slice is deliberately wrapper-only because it gives ANVIL
  real module composition, real design-level validation, real multi-file
  emission, and real downstream elaboration pressure without pretending
  recursive parent-side instance-driven cone construction is already
  solved.
- Treating `Node::InstanceOutput` as a leaf boundary in the current
  proof / compaction passes keeps the new hierarchy surface compatible
  with the existing leaf-kernel proof machinery while the deeper
  hierarchical identity story remains future work.

**Proof**

- New integration proof:
  `generates_valid_depth1_wrapper_designs` in
  [tests/pipeline.rs](/Users/richarddje/Documents/github/anvil/tests/pipeline.rs)
  sweeps seeds, validates the full design, checks instance count, and
  proves emitted SV contains multiple module declarations plus real
  instantiation syntax.
- New validator proofs:
  - `accepts_valid_depth1_design`
  - `rejects_missing_child_output_exposure_in_design`
- New emitter proof:
  `hierarchical_design_emits_real_instance_connections`
- Real downstream smoke on
  `/tmp/anvil-hierarchy-smoke-r1` is clean:
  - `cargo run --bin anvil -- --seed 7 --out /tmp/anvil-hierarchy-smoke-r1 --hierarchy-depth 1 --num-leaf-modules 3`
  - `verilator --lint-only /tmp/anvil-hierarchy-smoke-r1/*.sv`
  - `yosys -p "read_verilog -sv /tmp/anvil-hierarchy-smoke-r1/*.sv; synth -top mod_7_0003 -noabc; stat; check"`
  - `yosys -p "read_verilog -sv /tmp/anvil-hierarchy-smoke-r1/*.sv; synth -top mod_7_0003 -noabc; abc -fast; opt -fast; stat; check"`

**Impact**

- Phase 4 is no longer hypothetical. The roadmap state is now
  `in progress`, not `not started`.
- ANVIL can now emit a real multi-module synthesizable design rather
  than only unrelated leaf modules.
- The hierarchy story is still intentionally partial:
  - only `hierarchy_depth = 1` is live,
  - the top wrapper only instantiates and exposes leaves,
  - instance outputs are not yet used as parent cone inputs, and
  - there is no repo-owned Phase 4 closure gate yet.

**Files touched**

- `src/config.rs`
- `src/emit/mod.rs`
- `src/emit/sv.rs`
- `src/gen/cone.rs`
- `src/gen/hierarchy.rs`
- `src/gen/mod.rs`
- `src/gen/module.rs`
- `src/ir/compact.rs`
- `src/ir/types.rs`
- `src/ir/validate.rs`
- `src/lib.rs`
- `src/main.rs`
- `src/metrics.rs`
- `tests/pipeline.rs`
- `CHANGES.md`
- `MEMORY.md`
- `DEVELOPMENT_NOTES.md`
- `CODEBASE_ANALYSIS.md`
- `ROADMAP.md`
- `README.md`
- `USER_GUIDE.md`
- `book/src/architecture.md`
- `book/src/hierarchy.md`
- `book/src/ir.md`

**Validation**

- `cargo fmt --all --check`
- `cargo check --all-targets`
- `cargo test` = `203` passing (`150` lib + `5` main + `18` tool_matrix + `30` integration)
- `cargo clippy --all-targets -- -D warnings`
- `mdbook build book`

## 2026-04-23-0104 — Close the Phase 3 structured-surface gate cleanly

**Landed as:** `f759403`

**What changed**

- `tool_matrix` now has a repo-owned `--phase3-structured-gate` mode in
  [src/bin/tool_matrix.rs](/Users/richarddje/Documents/github/anvil/src/bin/tool_matrix.rs).
  It runs a dedicated 21-scenario structured-surface matrix across the
  three live construction strategies (`sequential`, `shuffled`,
  `interleaved`) under `identity_mode = node-id` and
  `factorization_level = e-graph`.
- The Phase 3 gate is coverage-owned rather than anecdote-owned. Its
  coverage contract requires real exercise of:
  - combinational-only and sequential modules
  - priority-encoder blocks
  - one-hot and encoded combinational muxes
  - procedural `case` and `casez` blocks
  - bounded procedural `for`-fold blocks
  - one-hot and encoded flop muxes
  - selectable `Slice` and `Concat`
  - variable shifts
- `Metrics` in
  [src/metrics.rs](/Users/richarddje/Documents/github/anvil/src/metrics.rs)
  now distinguish constant-rhs and variable-rhs shifts, so the gate can
  prove the variable-shift surface from emitted IR facts instead of from
  knob intent alone.
- The first real Phase 3 gate attempt surfaced the actual runtime
  hotspot in
  [src/ir/compact.rs](/Users/richarddje/Documents/github/anvil/src/ir/compact.rs):
  `merge_equivalent_gates` was still willing to run a full semantic
  truth-table proof on very large settled cones as long as their leaf
  support stayed small. That made compaction spend minutes in
  `semantic_cone_proof` / `evaluate_node_under_assignment`.
- The fix is now explicit and bounded:
  - cleanup-time exact proofs stay on their already-strict tiny-cone
    path
  - semantic merge proofs now also require the reachable cone to stay
    within a merge budget (`MAX_MERGE_SEMANTIC_CONE_NODES = 128`)
  - when that budget is exceeded, compaction falls back cleanly to the
    structural proof instead of turning semantic merge into a whole-cone
    evaluator
- New regression coverage:
  - `phase3_structured_gate_raises_modules_per_scenario_for_surface_gate`
  - `phase3_structured_matrix_covers_requested_surface_profiles`
  - `phase3_structured_coverage_requires_slice_concat_and_variable_shift`
  - `metrics_distinguish_constant_and_variable_shift_rhs`
  - `semantic_merge_proof_skips_large_low_support_cones`

**Proof**

- The completed current-code Phase 3 report now exists at
  `/tmp/anvil-tool-matrix-phase3-structured-r4/tool_matrix_report.json`.
- Final report summary:
  - `21` scenarios
  - `10` modules per scenario
  - `210` total modules
  - `coverage_gaps = []`
  - `Verilator pass/fail = 210/0`
  - `Yosys without-abc pass/fail = 210/0`
  - `Yosys with-abc pass/fail = 210/0`
- This closes the structured-surface evidence boundary. Phase 3 is no
  longer missing either feature breadth or repo-owned closure evidence.

**Validation**

- `cargo fmt --all --check`
- `cargo check --all-targets`
- `cargo test` = `197` passing (`145` lib + `5` main + `18` tool_matrix + `29` integration)
- `cargo clippy --all-targets -- -D warnings`
- `mdbook build book`

## 2026-04-22-2351 — Make Slice and Concat first-class selectable surfaces

**Landed as:** `d8b1556`

**What changed**

- `Slice` and `Concat` are no longer helper-only shapes. The generic
  structured gate picker in
  [src/gen/cone.rs](/Users/richarddje/Documents/github/anvil/src/gen/cone.rs)
  now selects real `Slice` and variadic `Concat` gates alongside `Mux`.
- [src/gen/module.rs](/Users/richarddje/Documents/github/anvil/src/gen/module.rs)
  now repairs any settled-graph output drive that has collapsed to an
  empty-dep constant by swapping in a dep-bearing exact-width source (or
  width-adapter) before final compaction. That keeps the generated
  output-cone invariant honest even after late proof-cleanup passes.
- The generator keeps the new surface non-degenerate by construction:
  - selectable `Slice` now always uses a wider source than its high bit,
    so it cannot collapse into the full-width slice identity
  - selectable `Concat` now always has at least 2 operands whose widths
    partition the output width, so it cannot collapse into the
    single-operand concat identity
- New proof tests:
  - structured-bucket category test now proves `Mux`, `Slice`, and
    `Concat` are all selectable
  - direct unit tests prove selectable `Slice` and `Concat` shapes are
    well-formed and non-degenerate
  - new pipeline integration test proves live `Slice` and `Concat`
    gates survive finalisation across all four construction strategies

- Docs/book now stop calling generic `Slice` / `Concat` helper-only.
  Phase 3 stays `in progress`, but the old feature-breadth gap has been
  narrowed again: the remaining honest closure work is the dedicated
  Phase 3 evidence gate.

**Validation**

- `cargo fmt --all --check`
- `cargo check --all-targets`
- `cargo test` = `191` passing (`142` lib + `5` main + `15` tool_matrix + `29` integration)
- `cargo clippy --all-targets -- -D warnings`
- `mdbook build book`

## 2026-04-22-2219 — Land the bounded procedural for-fold surface

**Landed as:** this commit

**What changed**

- The leaf kernel now has a real structured statically bounded
  unrolled-logic surface:
  - new knob `for_fold_prob` in
    [src/config.rs](/Users/richarddje/Documents/github/anvil/src/config.rs)
    and [src/main.rs](/Users/richarddje/Documents/github/anvil/src/main.rs)
  - new gate kind `GateOp::ForFold { kind, trip_count, chunk_width }`
    plus module / knob counters in
    [src/ir/types.rs](/Users/richarddje/Documents/github/anvil/src/ir/types.rs)
  - generator builders in
    [src/gen/cone.rs](/Users/richarddje/Documents/github/anvil/src/gen/cone.rs)
    that emit bounded `always_comb` for-loop folds over packed chunks
  - emitter support in
    [src/emit/sv.rs](/Users/richarddje/Documents/github/anvil/src/emit/sv.rs)
    that declares the target as `logic` and emits a procedural
    `for (int i = 0; i < N; i++)` block
  - validator support in
    [src/ir/validate.rs](/Users/richarddje/Documents/github/anvil/src/ir/validate.rs)
  - exact evaluator support in
    [src/ir/compact.rs](/Users/richarddje/Documents/github/anvil/src/ir/compact.rs)
  - metrics + matrix coverage plumbing in
    [src/metrics.rs](/Users/richarddje/Documents/github/anvil/src/metrics.rs)
    and [src/bin/tool_matrix.rs](/Users/richarddje/Documents/github/anvil/src/bin/tool_matrix.rs)

- A latent width-domain bug surfaced while landing the new surface:
  `pick_priority_encoder_n` could shift past the `u32` domain when a
  recursive sub-cone asked for widths above 32. That helper now rejects
  `target_width > 32` explicitly instead of overflowing.

- New proof tests:
  - emitter unit test for `ForFold`
  - validator unit test for `ForFold`
  - integration test proving `for_fold_prob=1.0` emits bounded
    `always_comb` for-loops across all strategies
  - knob-roll expectations updated so the new probability surface is
    required to be exercised in the default seed sweep
  - regression test proving priority-encoder compatibility rejects
    widths above the helper's `u32` domain

- Docs/book now treat the structured bounded `for` surface as landed
  Phase 3 breadth. Phase 3 stays `in progress`, but the old "for-loop
  unrolled logic is missing" statement is gone.

**Validation**

- `cargo fmt --all --check`
- `cargo check --all-targets`
- `cargo test` = `188` passing (`140` lib + `5` main + `15` tool_matrix + `28` integration)
- `cargo clippy --all-targets -- -D warnings`
- `mdbook build book`
- real smoke emission:
  [/tmp/anvil-forfold-smoke-r1/mod_1_0000.sv](/tmp/anvil-forfold-smoke-r1/mod_1_0000.sv)
  contains live `always_comb` `for (int i = 0; i < ...)` blocks on
  current HEAD

## 2026-04-22-2315 — Land the procedural combinational casez-mux block

**Landed as:** this commit

**What changed**

- The leaf kernel now has a real structured `casez` surface:
  - new knob `casez_mux_prob` in
    [src/config.rs](/Users/richarddje/Documents/github/anvil/src/config.rs)
    and [src/main.rs](/Users/richarddje/Documents/github/anvil/src/main.rs)
  - new gate kind `GateOp::CasezMux` plus module / knob counters in
    [src/ir/types.rs](/Users/richarddje/Documents/github/anvil/src/ir/types.rs)
  - generator builders in
    [src/gen/cone.rs](/Users/richarddje/Documents/github/anvil/src/gen/cone.rs)
    that emit non-overlapping wildcard patterns by construction
  - emitter support in
    [src/emit/sv.rs](/Users/richarddje/Documents/github/anvil/src/emit/sv.rs)
    that declares the target as `logic` and emits an
    `always_comb casez (sel)` block with explicit `default`
  - validator support in
    [src/ir/validate.rs](/Users/richarddje/Documents/github/anvil/src/ir/validate.rs)
  - exact evaluator support in
    [src/ir/compact.rs](/Users/richarddje/Documents/github/anvil/src/ir/compact.rs)
  - metrics + matrix coverage plumbing in
    [src/metrics.rs](/Users/richarddje/Documents/github/anvil/src/metrics.rs)
    and [src/bin/tool_matrix.rs](/Users/richarddje/Documents/github/anvil/src/bin/tool_matrix.rs)

- New proof tests:
  - emitter unit test for `CasezMux`
  - validator unit test for `CasezMux`
  - integration test proving `casez_mux_prob=1.0` emits
    `always_comb casez` across all strategies
  - knob-roll expectations updated so the new probability surface is
    required to be exercised in the default seed sweep

- Docs/book now treat `casez` as landed Phase 3 breadth rather than a
  future placeholder. At the time of this slice, the remaining obvious
  Phase 3 breadth gap was statically bounded unrolled logic; that gap
  has since been closed by the newer `for_fold` slice above.

**Validation**

- `cargo fmt --all --check`
- `cargo check --all-targets`
- `cargo test` = `184` passing (`137` lib + `5` main + `15` tool_matrix + `27` integration)
- `cargo clippy --all-targets -- -D warnings`
- `mdbook build book`

## 2026-04-22-2210 — Land the procedural combinational case-mux block

**Landed as:** this commit

**What changed**

- The leaf kernel now has a real structured `case` surface:
  - new knob `case_mux_prob` in
    [src/config.rs](/Users/richarddje/Documents/github/anvil/src/config.rs)
    and [src/main.rs](/Users/richarddje/Documents/github/anvil/src/main.rs)
  - new gate kind `GateOp::CaseMux` plus module / knob counters in
    [src/ir/types.rs](/Users/richarddje/Documents/github/anvil/src/ir/types.rs)
  - generator builders in
    [src/gen/cone.rs](/Users/richarddje/Documents/github/anvil/src/gen/cone.rs)
  - emitter support in
    [src/emit/sv.rs](/Users/richarddje/Documents/github/anvil/src/emit/sv.rs)
    that declares the target as `logic` and emits an
    `always_comb case (sel)` block with explicit `default`
  - validator support in
    [src/ir/validate.rs](/Users/richarddje/Documents/github/anvil/src/ir/validate.rs)
  - metrics + matrix coverage plumbing in
    [src/metrics.rs](/Users/richarddje/Documents/github/anvil/src/metrics.rs)
    and [src/bin/tool_matrix.rs](/Users/richarddje/Documents/github/anvil/src/bin/tool_matrix.rs)

- New proof tests:
  - emitter unit test for `CaseMux`
  - validator unit test for `CaseMux`
  - integration test proving `case_mux_prob=1.0` emits
    `always_comb case` across all strategies
  - variable-shift proof tightened from one deterministic seed to a
    32-seed sweep, so the regression now proves the surface instead of
    overfitting to a lucky RNG path

- Late settled-graph cleanup now folds mixed associative constants
  after remap-heavy passes:
  - new post-construction pass
    `fold_mixed_associative_constants` in
    [src/ir/compact.rs](/Users/richarddje/Documents/github/anvil/src/ir/compact.rs)
  - wired after the posthoc associative-normalisation points in
    [src/gen/module.rs](/Users/richarddje/Documents/github/anvil/src/gen/module.rs)
  - keeps strict duplicate-free `Add` / `Mul` output intact when later
    remaps expose shapes like `1 + x + 1`
  - pinned with new unit tests in
    [src/ir/types.rs](/Users/richarddje/Documents/github/anvil/src/ir/types.rs)
    and [src/ir/compact.rs](/Users/richarddje/Documents/github/anvil/src/ir/compact.rs)

- Live docs and book now say the narrower truth:
  - `case` is landed
  - `casez` is still open
  - `always_comb` is now part of the leaf lane specifically for the
    structured case-mux block
  - late mixed-constant cleanup is now part of the settled-graph
    normalization story

**Why**

Phase 1 and Phase 2 are closed, and the next leaf-lane gap was no
longer "can we emit valid RTL at all?" but "are we actually exercising
the structured frontend surfaces we said we wanted?" A procedural case
block is a meaningful new downstream path: same broad mux semantics,
different parser/elaboration route, and still fully synthesizable by
construction.

**Validation**

- `cargo check --all-targets`
- `cargo test`
- `cargo clippy --all-targets -- -D warnings`
- `cargo fmt --all --check`
- `mdbook build book`

## 2026-04-22-2115 — Prove and document the variable-shift surface

**Landed as:** this commit

**What changed**

- [tests/pipeline.rs](/Users/richarddje/Documents/github/anvil/tests/pipeline.rs)
  now has `variable_shift_amount_appears_in_output`, a deterministic
  regression that forces a shift-only module with:
  - `const_shift_amount_prob = 0.0`
  - `gate_shift_weight = 1`
  - every other gate bucket disabled
  - `terminal_reuse_prob = 1.0`
  - `constant_prob = 0.0`
  - fixed 8-bit ports and `max_depth = 1`

  That test proves two things at once:
  - the IR really does contain a `Shl`/`Shr` whose rhs is **not** a
    `Constant`
  - the emitted SV really does contain a variable shift
    (`value << signal` / `value >> signal`)

- [ROADMAP.md](/Users/richarddje/Documents/github/anvil/ROADMAP.md),
  [CODEBASE_ANALYSIS.md](/Users/richarddje/Documents/github/anvil/CODEBASE_ANALYSIS.md),
  and [MEMORY.md](/Users/richarddje/Documents/github/anvil/MEMORY.md)
  no longer describe variable shifts as "not started". The remaining
  obvious Phase 3 breadth gaps are now `case` / `casez` and
  statically bounded unrolled logic.

**Why**

The code and the book had already drifted apart in an interesting way:
the generator had a real variable-shift path whenever
`const_shift_amount_prob` missed, and `book/src/structural-rules.md`
already said so, but the roadmap/live-doc layer still described
variable shifts as missing. This slice pins the behavior down with a
proof test and brings the steering docs back to the narrower truth.

**Validation**

- `cargo check --all-targets`
- `cargo test`
- `cargo clippy --all-targets -- -D warnings`
- `cargo fmt --all --check`
- `mdbook build book`

## 2026-04-22-2048 — Pin the crate MSRV to Rust 1.95

**Landed as:** this commit

**What changed**

- [Cargo.toml](/Users/richarddje/Documents/github/anvil/Cargo.toml)
  now declares `rust-version = "1.95"`.
- [CODEBASE_ANALYSIS.md](/Users/richarddje/Documents/github/anvil/CODEBASE_ANALYSIS.md)
  no longer says the MSRV is unpinned; it now reflects the explicit
  Rust 1.95 baseline.

**Why**

The toolchain floor is no longer aspirational or implicit. The user has
already raised the local Rust baseline to 1.95, so the repo should say
that directly in the manifest and the live docs.

**Validation**

- `cargo check --all-targets`
- `cargo test`
- `cargo clippy --all-targets -- -D warnings`
- `cargo fmt --all --check`
- `mdbook build book`

## 2026-04-22-2038 — Close the Phase 2 sharing gate with a normalized share metric

**Landed as:** this commit

**What changed**

`src/bin/tool_matrix.rs` now has a repo-owned `--phase2-share-gate`
mode alongside the existing `--phase1-gate`.

The new gate builds a dedicated 18-scenario share sweep:

- 3 construction strategies (`sequential`, `shuffled`,
  `interleaved`)
- 2 profiles (combinational share-stress and sequential share-stress)
- 3 explicit `share_prob` points (`0.0`, `0.3`, `0.9`)

It also records a `share_sweep` summary in
`tool_matrix_report.json`, grouped by `share_prob`.

One subtle correction landed during the slice: the first version of the
gate tried to prove "controlled sharing factor" with raw
`total_shared_nodes`. The real run showed that proxy was backwards:
higher `share_prob` collapses the graph enough that the absolute count
of shared nodes falls even though the graph becomes *more shared*. The
gate now measures the right thing: normalized
`shared_node_fraction = total_shared_nodes / total_nodes`, while still
recording node-count collapse alongside it.

That fixed gate has now been proven on a real current-code run at:

- [/tmp/anvil-tool-matrix-phase2-share-r1/tool_matrix_report.json](/tmp/anvil-tool-matrix-phase2-share-r1/tool_matrix_report.json)

The saved report records:

- `18` scenarios
- `12` modules per scenario
- `216` total modules
- `coverage_gaps = []`
- `Verilator pass/fail = 216/0`
- `Yosys without-abc pass/fail = 216/0`
- `Yosys with-abc pass/fail = 216/0`
- monotone sharing sweep:
  - `share_prob = 0.0`: `shared_node_fraction = 0.4122`,
    `avg_nodes/module = 4727.56`
  - `share_prob = 0.3`: `shared_node_fraction = 0.4232`,
    `avg_nodes/module = 3525.01`
  - `share_prob = 0.9`: `shared_node_fraction = 0.4386`,
    `avg_nodes/module = 2117.76`

That closes the repo-owned Phase 2 exit criterion locally: clean
Verilator/Yosys on the representative `share_prob` sweep, with an
actual structural sharing metric that moves the right way.

**Why**

Phase 1 is already closed, so the next leaf-kernel question was no
longer "does sharing exist?" It was "can we prove, on current code,
that sharing remains tool-clean across a representative sweep and that
the sharing knob measurably changes the landed graphs?" Making that a
first-class gate in `tool_matrix` keeps the answer reproducible and
recoverable after a crash.

**Validation**

- targeted harness tests:
  - `cargo test --bin tool_matrix`
- real downstream proof:
  - `cargo run --bin tool_matrix -- --out /tmp/anvil-tool-matrix-phase2-share-r1 --phase2-share-gate --yosys-mode both`
  - first run completed with `216/0` in all three tool lanes and
    exposed the bad raw-shared-node proxy
  - corrected-gate rerun:
    `cargo run --bin tool_matrix -- --out /tmp/anvil-tool-matrix-phase2-share-r1 --phase2-share-gate --yosys-mode both --resume`
  - completed successfully with `coverage_gaps = []`
- full repo hygiene:
  - `cargo check --all-targets`
  - `cargo test`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo fmt --all --check`
  - `mdbook build book`

## 2026-04-22-1623 — Close the full current-code Phase 1 gate

**Landed as:** this commit

**What changed**

No Rust source changed in this slice. The work was another real
`tool_matrix --phase1-gate --yosys-mode both --resume` continuation on
the live current-code frontier at
`/tmp/anvil-tool-matrix-phase1-real-r21`.

This slice did not stop at the earlier `804/804` bank. The resumed run
kept going, closed the remaining shuffled motif-heavy lane, closed the
remaining interleaved share-heavy lane, and then closed the final
interleaved motif-heavy lane too.

The saved tree now stands at:

- **1005** completed module checkpoints / **1005** emitted `.sv` files
- full closure of:
  - all 15 built-in `tool_matrix` scenarios
  - every construction strategy (`interleaved`, `sequential`,
    `shuffled`)
  - both identity modes
  - every current factorization rung through `e-graph`
  - both stress profiles (`share_heavy_comb_only`,
    `motif_heavy_seq`)

That means the repo-owned Phase 1 exit gate is now closed on current
code, not merely partially banked. The completed report is:

- [/tmp/anvil-tool-matrix-phase1-real-r21/tool_matrix_report.json](/tmp/anvil-tool-matrix-phase1-real-r21/tool_matrix_report.json)

**Why**

The next useful question was no longer "can we push the frontier a bit
farther?" It was "can the actual Phase 1 gate finish cleanly on current
code?" Once the final three scenarios stayed warning-clean deep enough
into the run, the right move was to keep going and answer that question
for real instead of banking another intermediate checkpoint.

**Validation**

- real resumed frontier run:
  - `cargo run --bin tool_matrix -- --out /tmp/anvil-tool-matrix-phase1-real-r21 --phase1-gate --yosys-mode both --resume`
  - completed successfully
- resulting tree state:
  - **1005** completed module checkpoints / **1005** emitted `.sv` files
  - **0** Verilator warning logs
  - **0** Yosys `Warning:` lines across both Yosys modes
  - `coverage_gaps: []`
  - `Verilator pass/fail = 1005/0`
  - `Yosys without-abc pass/fail = 1005/0`
  - `Yosys with-abc pass/fail = 1005/0`
- full repo hygiene:
  - `cargo check --all-targets`
  - `cargo test`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo fmt --all --check`
  - `mdbook build book`

## 2026-04-22-1458 — Bank 40 clean motif-heavy sequential e-graph modules

**Landed as:** `c77d9f2`

**What changed**

No Rust source changed in this slice. The work was another real
`tool_matrix --phase1-gate --yosys-mode both --resume` continuation on
the live current-code frontier at
`/tmp/anvil-tool-matrix-phase1-real-r21`.

This slice stayed on the same hard sequential `e-graph` lane and pushed
the clean checkpoint bank inside
`seq_nodeid_egraph_motif_heavy_seq` from 32 modules to 40.

The saved tree now stands at:

- **710** completed module checkpoints / **710** emitted `.sv` files
- full closure of:
  - all interleaved `int_*` scenarios through `e-graph`
  - `seq_nodeid_egraph_share_heavy_comb_only`
- `seq_nodeid_egraph_motif_heavy_seq`: **40** clean checkpoints /
  **40** emitted `.sv` files

So the live `r21` tree now carries a stronger fully checkpointed bank in
the heavier sequential `e-graph` lane while staying warning-clean
throughout.

**Why**

Once the docs were back in sync, the next natural task was to resume the
live frontier immediately and keep pressure on the same sequential lane.
The fast-resume path is now working as intended on `r21`, so this was a
good moment to bank another real current-code recovery point rather than
leaving that clean path idle.

**Validation**

- real resumed frontier run:
  - `cargo run --bin tool_matrix -- --out /tmp/anvil-tool-matrix-phase1-real-r21 --phase1-gate --yosys-mode both --resume`
  - intentionally interrupted after `seq_nodeid_egraph_motif_heavy_seq`
    reached 40 clean checkpoints
- resulting tree state:
  - **710** completed module checkpoints / **710** emitted `.sv` files
  - **0** Verilator warning logs
  - **0** Yosys `Warning:` lines across both Yosys modes
- full repo hygiene:
  - `cargo check --all-targets`
  - `cargo test`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo fmt --all --check`
  - `mdbook build book`

## 2026-04-22-1454 — Sync codebase analysis with the 702-module r21 bank

**Landed as:** `4255c15`

**What changed**

No Rust source changed in this slice. The work was a documentation
recovery pass to bring `CODEBASE_ANALYSIS.md` back into sync with the
actual live `r21` frontier that had already been banked and recorded in
`CHANGES.md` and `MEMORY.md`.

The real current-code resumable tree at
`/tmp/anvil-tool-matrix-phase1-real-r21` remains unchanged at:

- **702** completed module checkpoints / **703** emitted `.sv` files
- full closure of:
  - all interleaved `int_*` scenarios through `e-graph`
  - `seq_nodeid_egraph_share_heavy_comb_only`
- `seq_nodeid_egraph_motif_heavy_seq`: **32** clean checkpoints /
  **33** emitted `.sv` files

What changed here is only the handoff truth: the stale `690/690`
references in `CODEBASE_ANALYSIS.md` are now aligned with the real saved
tree and the already-landed `2026-04-22-0129` checkpoint.

**Why**

The previous checkpoint commit correctly landed the stronger sequential
`e-graph` bank, but `CODEBASE_ANALYSIS.md` was left behind at the older
`690/690` state. That is exactly the kind of recovery drift the commit
workflow is supposed to prevent, so this slice closes that gap cleanly.

**Validation**

- `cargo check --all-targets`
- `cargo test`
- `cargo clippy --all-targets -- -D warnings`
- `cargo fmt --all --check`
- `mdbook build book`

## 2026-04-22-0129 — Bank 32 clean motif-heavy sequential e-graph modules

**Landed as:** `7098a2f`

**What changed**

No Rust source changed in this slice. The work was another real
`tool_matrix --phase1-gate --yosys-mode both --resume` continuation on
the live current-code frontier at
`/tmp/anvil-tool-matrix-phase1-real-r21`.

This slice stayed on the same hard sequential `e-graph` lane and pushed
the clean checkpoint bank inside
`seq_nodeid_egraph_motif_heavy_seq` from 20 modules to 32.

The saved tree now stands at:

- **702** completed module checkpoints / **703** emitted `.sv` files
- full closure of:
  - all interleaved `int_*` scenarios through `e-graph`
  - `seq_nodeid_egraph_share_heavy_comb_only`
- `seq_nodeid_egraph_motif_heavy_seq`: **32** clean checkpoints /
  **33** emitted `.sv` files

So the live `r21` tree now carries a substantially stronger recovery
point in the heavier sequential motif lane rather than just a first
sampling.

**Why**

The previous slice had established that the sequential motif-heavy lane
could stay warning-clean for 20 modules. The sensible next move was to
keep pressure on that exact lane long enough to tell whether the clean
behavior was stable or just an early-run fluke.

This deeper bank gives a much better answer: the lane is still clean,
still resumable, and still worth pushing before we go hunting for some
other problem.

**Validation**

- real resumed frontier run:
  - `cargo run --bin tool_matrix -- --out /tmp/anvil-tool-matrix-phase1-real-r21 --phase1-gate --yosys-mode both --resume`
  - intentionally interrupted after `seq_nodeid_egraph_motif_heavy_seq`
    reached 32 clean checkpoints
- resulting tree state:
  - **702** completed module checkpoints / **703** emitted `.sv` files
  - **0** Verilator warning logs
  - **0** Yosys `Warning:` lines across both Yosys modes
- full repo hygiene:
  - `cargo check --all-targets`
  - `cargo test`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo fmt --all --check`
  - `mdbook build book`

## 2026-04-22-0128 — Deepen the sequential motif-heavy e-graph frontier

**Landed as:** this commit

**What changed**

No Rust source changed in this slice. The work was another real
`tool_matrix --phase1-gate --yosys-mode both --resume` continuation on
the live current-code frontier at
`/tmp/anvil-tool-matrix-phase1-real-r21`.

This slice stayed on the same sequential `e-graph` lane and banked a
deeper clean checkpoint inside
`seq_nodeid_egraph_motif_heavy_seq`.

The saved tree now stands at:

- **690** completed module checkpoints / **690** emitted `.sv` files
- full closure of:
  - all interleaved `int_*` scenarios through `e-graph`
  - `seq_nodeid_egraph_share_heavy_comb_only`
- `seq_nodeid_egraph_motif_heavy_seq`: **20** clean checkpoints /
  **20** emitted `.sv` files

So the live `r21` tree now carries a more durable sequential
`e-graph` bank instead of only a small toe-hold in that heavier lane.

**Why**

The previous slice had just entered the motif-heavy sequential lane. The
most useful next move was not to switch context, but to stay on that
same lane long enough to see whether it remained warning-clean under a
more meaningful sample.

It did, and that gives the next continuation a stronger recovery point
right where the frontier is currently hardest.

**Validation**

- real resumed frontier run:
  - `cargo run --bin tool_matrix -- --out /tmp/anvil-tool-matrix-phase1-real-r21 --phase1-gate --yosys-mode both --resume`
  - intentionally interrupted after the motif-heavy sequential lane was
    pushed to 20 clean checkpoints
- resulting tree state:
  - **690** completed module checkpoints / **690** emitted `.sv` files
  - **0** Verilator warning logs
  - **0** Yosys `Warning:` lines across both Yosys modes
- full repo hygiene:
  - `cargo check --all-targets`
  - `cargo test`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo fmt --all --check`
  - `mdbook build book`

## 2026-04-22-0127 — Close interleaved e-graph and enter sequential e-graph lanes

**Landed as:** this commit

**What changed**

No Rust source changed in this slice. The work was another real
`tool_matrix --phase1-gate --yosys-mode both --resume` continuation on
the live current-code frontier at
`/tmp/anvil-tool-matrix-phase1-real-r21`.

This push cleared three useful boundaries:

- full clean closure of `int_nodeid_e-graph_default`
- full clean closure of `seq_nodeid_egraph_share_heavy_comb_only`
- first clean bank into `seq_nodeid_egraph_motif_heavy_seq`

The saved tree now stands at:

- **675** completed module checkpoints / **676** emitted `.sv` files
- full closure of:
  - all interleaved `int_*` scenarios through `e-graph`
  - `seq_nodeid_egraph_share_heavy_comb_only`
- `seq_nodeid_egraph_motif_heavy_seq`: **5** clean checkpoints /
  **6** emitted `.sv` files

So `r21` is no longer just a fully-closed interleaved frontier; it now
extends cleanly into the sequential `e-graph` half of the Phase 1 gate.

**Why**

Once the previous slice had closed the interleaved ladder through
`peephole` and banked the first `e-graph` foothold, the natural next
step was to keep cashing in on the fast-resume path and see how far the
current code could go before the next real boundary appeared.

This slice pushed well past the old `r20` `e-graph` bank, closed the
entire interleaved matrix, and carried that momentum into the first
sequential `e-graph` lanes on the same live tree.

**Validation**

- real resumed frontier run:
  - `cargo run --bin tool_matrix -- --out /tmp/anvil-tool-matrix-phase1-real-r21 --phase1-gate --yosys-mode both --resume`
  - intentionally interrupted after full interleaved `e-graph` closure,
    full `seq_nodeid_egraph_share_heavy_comb_only`, and a first clean
    `seq_nodeid_egraph_motif_heavy_seq` bank
- resulting tree state:
  - **675** completed module checkpoints / **676** emitted `.sv` files
  - **0** Verilator warning logs
  - **0** Yosys `Warning:` lines across both Yosys modes
- full repo hygiene:
  - `cargo check --all-targets`
  - `cargo test`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo fmt --all --check`
  - `mdbook build book`

## 2026-04-22-0126 — Close r21 peephole and enter e-graph

**Landed as:** this commit

**What changed**

No Rust source changed in this slice. The work was another real
`tool_matrix --phase1-gate --yosys-mode both --resume` continuation on
the live current-code frontier at
`/tmp/anvil-tool-matrix-phase1-real-r21`.

The frontier now crossed two useful boundaries:

- full clean closure of `int_nodeid_peephole_default`
- first clean bank into `int_nodeid_e-graph_default`

The saved tree now stands at:

- **559** completed module checkpoints / **559** emitted `.sv` files
- full closure of:
  - `int_relaxed_none_default`
  - `int_nodeid_none_default`
  - `int_nodeid_cse_default`
  - `int_nodeid_operand-unique_default`
  - `int_nodeid_commutative_default`
  - `int_nodeid_associative_default`
  - `int_nodeid_constant-fold_default`
  - `int_nodeid_peephole_default`
- `int_nodeid_e-graph_default`: **23** clean checkpoints / **23**
  emitted `.sv` files

The upgraded fast-resume metadata remains intact across the saved bank,
so the next same-binary resume on `r21` can continue directly from this
state.

**Why**

After the previous slice finished the one-time replay-and-upgrade pass,
the natural next step was to cash in on that cheaper resume path and
push the live frontier farther on current code.

This slice does exactly that: it closes the remaining `peephole` work,
establishes a clean `e-graph` bank on the same tree, and leaves the
repo with a stronger live recovery point for the next continuation.

**Validation**

- real resumed frontier run:
  - `cargo run --bin tool_matrix -- --out /tmp/anvil-tool-matrix-phase1-real-r21 --phase1-gate --yosys-mode both --resume`
  - intentionally interrupted after full `peephole` closure and 23 clean
    `e-graph` checkpoints were banked
- resulting tree state:
  - **559** completed module checkpoints / **559** emitted `.sv` files
  - **0** Verilator warning logs
  - **0** Yosys `Warning:` lines across both Yosys modes
- full repo hygiene:
  - `cargo check --all-targets`
  - `cargo test`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo fmt --all --check`
  - `mdbook build book`

## 2026-04-22-0125 — Upgrade the live r21 frontier to fast-resume checkpoints

**Landed as:** this commit

**What changed**

No Rust source changed in this slice. The work was a real
`tool_matrix --phase1-gate --yosys-mode both --resume` continuation on
the live current-code frontier at
`/tmp/anvil-tool-matrix-phase1-real-r21`.

The important state change is operational:

- all existing `r21` checkpoint sidecars were replayed once and upgraded
  in place to the new fast-resume format;
- the tree now has **508** completed module checkpoints, and all **508**
  carry:
  - `runtime_fingerprint`
  - `sv_hash`
  - `generator_checkpoint`
- the live frontier also advanced from the older `peephole` bank to
  **39** completed `int_nodeid_peephole_default` checkpoints
  (**40** emitted `.sv` files).

So `r21` is no longer just "resumable after one more replay"; it is now
the first live current-code frontier whose whole saved bank is already
in the cheap same-binary resume format.

**Why**

The previous slice added the fast-resume machinery, but the live
frontier was still carrying pre-upgrade sidecars. That meant the next
real frontier run would still pay the old replay tax once before the new
path could help.

This slice paid that one-time upgrade cost on the real tree and banked
the result, so future same-binary resumes can start from the upgraded
state directly instead of rebuilding it again.

**Validation**

- real resumed frontier run:
  - `cargo run --bin tool_matrix -- --out /tmp/anvil-tool-matrix-phase1-real-r21 --phase1-gate --yosys-mode both --resume`
  - intentionally interrupted after the upgrade pass had completed and
    new `peephole` progress was banked
- resulting tree state:
  - **508** completed module checkpoints / **509** emitted `.sv` files
  - full closure of:
    - `int_relaxed_none_default`
    - `int_nodeid_none_default`
    - `int_nodeid_cse_default`
    - `int_nodeid_operand-unique_default`
    - `int_nodeid_commutative_default`
    - `int_nodeid_associative_default`
    - `int_nodeid_constant-fold_default`
  - `int_nodeid_peephole_default`: **39** checkpoints / **40** emitted
    `.sv` files
  - **0** Verilator warning logs
  - **0** Yosys `Warning:` lines across both Yosys modes
- full repo hygiene:
  - `cargo check --all-targets`
  - `cargo test`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo fmt --all --check`
  - `mdbook build book`

## 2026-04-22-0124 — Add same-binary fast resume checkpoints to tool_matrix

**Landed as:** this commit

**What changed**

`tool_matrix --resume` no longer has to replay every already-proven
module on the same binary just to reconstruct generator state.

This slice added a real same-binary fast-resume path:

- `Generator` now exposes a serializable checkpoint carrying its
  `ChaCha8Rng` stream position plus `next_module_index`.
- `tool_matrix` checkpoints now record:
  - the generator checkpoint,
  - a hash of the emitted `.sv`, and
  - a fingerprint of the current `tool_matrix` binary.
- On resume, when the saved tool surface still matches and those
  checkpoint fields still match the current binary and saved `.sv`,
  `tool_matrix` restores the generator directly and reuses the saved
  report without regenerating that module.
- Older checkpoints still work: if the fast-resume metadata is missing
  or stale, the old strict replay-and-validate path stays in force and
  upgrades the checkpoint in place.

**Why**

Sampling the live `r21` frontier showed that the expensive part was no
longer downstream tools; it was replaying generator work to rebuild RNG
state on resume. That strict replay is still the right fallback across
binary or generator-semantic changes, but it was unnecessary drag for
same-binary reruns of already-proven trees.

So the target here was narrow and deliberate:

- keep resume byte-stable,
- keep old trees resumable,
- keep file-integrity checks, and
- remove the same-build replay tax.

**Validation**

- focused unit coverage:
  - `cargo test --bin tool_matrix`
  - new tests:
    - same-binary fast resume restores generator state for the next
      module exactly
    - fast resume rejects `sv` hash mismatches
    - older checkpoint shape still upgrades and resumes correctly
- real same-binary smoke:
  - `cargo run --bin tool_matrix -- --out /tmp/anvil-tool-matrix-resume-fast-smoke-r1 --modules-per-scenario 1 --skip-verilator --skip-yosys --yosys-mode both`
  - `cargo run --bin tool_matrix -- --out /tmp/anvil-tool-matrix-resume-fast-smoke-r1 --modules-per-scenario 1 --skip-verilator --skip-yosys --yosys-mode both --resume`
  - rerun completed successfully on the same tree
  - saved checkpoints now contain:
    - `runtime_fingerprint`
    - `sv_hash`
    - `generator_checkpoint`
- full repo hygiene:
  - `cargo check --all-targets`
  - `cargo test`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo fmt --all --check`
  - `mdbook build book`

**Impact**

- Same-binary `tool_matrix --resume` is now much cheaper on fresh
  checkpoints written by the current binary.
- Resume remains intentionally byte-stable across generator changes:
  old trees still replay and validate instead of being trusted blindly.
- The live `r21` frontier remains the next real frontier task, but its
  older pre-upgrade checkpoints will still pay the one-time strict
  replay cost before future same-binary resumes become cheap.

## 2026-04-22-0123 — Advance the fresh r21 both-mode frontier through constant-fold

**Landed as:** this commit

**What changed**

No source changes in this slice. I resumed the live current-code
both-mode frontier at:

- `/tmp/anvil-tool-matrix-phase1-real-r21`

using:

- `cargo run --bin tool_matrix -- --out /tmp/anvil-tool-matrix-phase1-real-r21 --phase1-gate --yosys-mode both --resume`

and pushed it cleanly through the full `constant-fold` rung, then into
`peephole`, before intentionally interrupting at the next useful
checkpoint.

**Why**

The previous slice had already re-closed `associative` on the fresh
current-code tree. The next useful question was whether the repaired
code would keep holding through the whole `constant-fold` rung on that
same fresh both-mode tree.

It does.

**Validation**

- resumed current-code both-mode frontier:
  - `cargo run --bin tool_matrix -- --out /tmp/anvil-tool-matrix-phase1-real-r21 --phase1-gate --yosys-mode both --resume`
  - intentionally interrupted after **500** completed checkpoints /
    **501** emitted `.sv` files
  - scenario coverage at the checkpoint:
    - `int_relaxed_none_default`: 67 checkpoints / 67 `.sv`
    - `int_nodeid_none_default`: 67 checkpoints / 67 `.sv`
    - `int_nodeid_cse_default`: 67 checkpoints / 67 `.sv`
    - `int_nodeid_operand-unique_default`: 67 checkpoints / 67 `.sv`
    - `int_nodeid_commutative_default`: 67 checkpoints / 67 `.sv`
    - `int_nodeid_associative_default`: 67 checkpoints / 67 `.sv`
    - `int_nodeid_constant-fold_default`: 67 checkpoints / 67 `.sv`
    - `int_nodeid_peephole_default`: 31 checkpoints / 32 `.sv`
  - zero Verilator warning logs
  - zero Yosys `Warning:` lines
- full repo hygiene:
  - `cargo check --all-targets`
  - `cargo test`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo fmt --all --check`
  - `mdbook build book`

**Impact**

- `/tmp/anvil-tool-matrix-phase1-real-r21` remains the live resumable
  current-code both-mode frontier tree.
- The fresh-tree frontier has now fully re-closed `relaxed`,
  `nodeid-none`, `nodeid-cse`, `nodeid-operand-unique`,
  `nodeid-commutative`, `nodeid-associative`, and
  `nodeid-constant-fold` on current code, with `peephole` underway.
- The next frontier push should resume `r21` in place and keep climbing
  through `peephole`, then `e-graph`, toward full fresh-tree both-mode
  closure on current code.

## 2026-04-22-0122 — Advance the fresh r21 both-mode frontier through associative

**Landed as:** this commit

**What changed**

No source changes in this slice. I resumed the live current-code
both-mode frontier at:

- `/tmp/anvil-tool-matrix-phase1-real-r21`

using:

- `cargo run --bin tool_matrix -- --out /tmp/anvil-tool-matrix-phase1-real-r21 --phase1-gate --yosys-mode both --resume`

and pushed it cleanly through the full `associative` rung, then into
`constant-fold`, before intentionally interrupting at the next useful
checkpoint.

**Why**

The previous slice had already re-closed `commutative` on the fresh
current-code tree. The next useful question was whether the repaired
code would keep holding through the whole `associative` rung on that
same fresh both-mode tree.

It does.

**Validation**

- resumed current-code both-mode frontier:
  - `cargo run --bin tool_matrix -- --out /tmp/anvil-tool-matrix-phase1-real-r21 --phase1-gate --yosys-mode both --resume`
  - intentionally interrupted after **433** completed checkpoints /
    **434** emitted `.sv` files
  - scenario coverage at the checkpoint:
    - `int_relaxed_none_default`: 67 checkpoints / 67 `.sv`
    - `int_nodeid_none_default`: 67 checkpoints / 67 `.sv`
    - `int_nodeid_cse_default`: 67 checkpoints / 67 `.sv`
    - `int_nodeid_operand-unique_default`: 67 checkpoints / 67 `.sv`
    - `int_nodeid_commutative_default`: 67 checkpoints / 67 `.sv`
    - `int_nodeid_associative_default`: 67 checkpoints / 67 `.sv`
    - `int_nodeid_constant-fold_default`: 31 checkpoints / 32 `.sv`
  - zero Verilator warning logs
  - zero Yosys `Warning:` lines
- full repo hygiene:
  - `cargo check --all-targets`
  - `cargo test`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo fmt --all --check`
  - `mdbook build book`

**Impact**

- `/tmp/anvil-tool-matrix-phase1-real-r21` remains the live resumable
  current-code both-mode frontier tree.
- The fresh-tree frontier has now fully re-closed `relaxed`,
  `nodeid-none`, `nodeid-cse`, `nodeid-operand-unique`,
  `nodeid-commutative`, and `nodeid-associative` on current code, with
  `constant-fold` underway.
- The next frontier push should resume `r21` in place and keep climbing
  through `constant-fold`, then `peephole`, toward the repaired
  `e-graph` surface.

## 2026-04-22-0121 — Advance the fresh r21 both-mode frontier through commutative

**Landed as:** this commit

**What changed**

No source changes in this slice. I resumed the live current-code
both-mode frontier at:

- `/tmp/anvil-tool-matrix-phase1-real-r21`

using:

- `cargo run --bin tool_matrix -- --out /tmp/anvil-tool-matrix-phase1-real-r21 --phase1-gate --yosys-mode both --resume`

and pushed it cleanly through the full `commutative` rung, then into
`associative`, before intentionally interrupting at the next useful
checkpoint.

**Why**

The previous slice had already re-closed `operand-unique` on the fresh
current-code tree. The next useful question was whether the repaired
code would keep holding through the whole `commutative` rung on that
same fresh both-mode tree.

It does.

**Validation**

- resumed current-code both-mode frontier:
  - `cargo run --bin tool_matrix -- --out /tmp/anvil-tool-matrix-phase1-real-r21 --phase1-gate --yosys-mode both --resume`
  - intentionally interrupted after **355** completed checkpoints /
    **356** emitted `.sv` files
  - scenario coverage at the checkpoint:
    - `int_relaxed_none_default`: 67 checkpoints / 67 `.sv`
    - `int_nodeid_none_default`: 67 checkpoints / 67 `.sv`
    - `int_nodeid_cse_default`: 67 checkpoints / 67 `.sv`
    - `int_nodeid_operand-unique_default`: 67 checkpoints / 67 `.sv`
    - `int_nodeid_commutative_default`: 67 checkpoints / 67 `.sv`
    - `int_nodeid_associative_default`: 20 checkpoints / 21 `.sv`
  - zero Verilator warning logs
  - zero Yosys `Warning:` lines
- full repo hygiene:
  - `cargo check --all-targets`
  - `cargo test`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo fmt --all --check`
  - `mdbook build book`

**Impact**

- `/tmp/anvil-tool-matrix-phase1-real-r21` remains the live resumable
  current-code both-mode frontier tree.
- The fresh-tree frontier has now fully re-closed `relaxed`,
  `nodeid-none`, `nodeid-cse`, `nodeid-operand-unique`, and
  `nodeid-commutative` on current code, with `associative` underway.
- The next frontier push should resume `r21` in place and keep climbing
  through `associative`, then `constant-fold`, toward the repaired
  `e-graph` surface.

## 2026-04-22-0120 — Advance the fresh r21 both-mode frontier through operand-unique

**Landed as:** this commit

**What changed**

No source changes in this slice. I resumed the live current-code
both-mode frontier at:

- `/tmp/anvil-tool-matrix-phase1-real-r21`

using:

- `cargo run --bin tool_matrix -- --out /tmp/anvil-tool-matrix-phase1-real-r21 --phase1-gate --yosys-mode both --resume`

and pushed it cleanly through the full `operand-unique` rung, then into
`commutative`, before intentionally interrupting at the next useful
checkpoint.

**Why**

The previous slice had already re-closed `cse` on the fresh current-code
tree. The next useful question was whether the repaired code would keep
holding through the whole `operand-unique` rung on the same fresh
both-mode tree.

It does.

**Validation**

- resumed current-code both-mode frontier:
  - `cargo run --bin tool_matrix -- --out /tmp/anvil-tool-matrix-phase1-real-r21 --phase1-gate --yosys-mode both --resume`
  - intentionally interrupted after **289** completed checkpoints /
    **290** emitted `.sv` files
  - scenario coverage at the checkpoint:
    - `int_relaxed_none_default`: 67 checkpoints / 67 `.sv`
    - `int_nodeid_none_default`: 67 checkpoints / 67 `.sv`
    - `int_nodeid_cse_default`: 67 checkpoints / 67 `.sv`
    - `int_nodeid_operand-unique_default`: 67 checkpoints / 67 `.sv`
    - `int_nodeid_commutative_default`: 21 checkpoints / 22 `.sv`
  - zero Verilator warning logs
  - zero Yosys `Warning:` lines
- full repo hygiene:
  - `cargo check --all-targets`
  - `cargo test`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo fmt --all --check`
  - `mdbook build book`

**Impact**

- `/tmp/anvil-tool-matrix-phase1-real-r21` remains the live resumable
  current-code both-mode frontier tree.
- The fresh-tree frontier has now fully re-closed `relaxed`,
  `nodeid-none`, `nodeid-cse`, and `nodeid-operand-unique` on current
  code, with `commutative` underway.
- The next frontier push should resume `r21` in place and keep climbing
  through `commutative`, then `associative`, toward the repaired
  `e-graph` surface.

## 2026-04-22-0119 — Advance the fresh r21 both-mode frontier through cse

**Landed as:** this commit

**What changed**

No source changes in this slice. I resumed the live current-code
both-mode frontier at:

- `/tmp/anvil-tool-matrix-phase1-real-r21`

using:

- `cargo run --bin tool_matrix -- --out /tmp/anvil-tool-matrix-phase1-real-r21 --phase1-gate --yosys-mode both --resume`

and pushed it cleanly through the full `cse` rung, then into
`operand-unique`, before intentionally interrupting at the next useful
checkpoint.

**Why**

The previous slice only banked the fresh current-code tree through full
`relaxed` and full `nodeid-none`, with `cse` just beginning. The next
honest question was whether the repaired code would keep holding through
the whole `cse` rung on a fresh both-mode tree, not just on a short
prefix.

It does.

**Validation**

- resumed current-code both-mode frontier:
  - `cargo run --bin tool_matrix -- --out /tmp/anvil-tool-matrix-phase1-real-r21 --phase1-gate --yosys-mode both --resume`
  - intentionally interrupted after **211** completed checkpoints /
    **212** emitted `.sv` files
  - scenario coverage at the checkpoint:
    - `int_relaxed_none_default`: 67 checkpoints / 67 `.sv`
    - `int_nodeid_none_default`: 67 checkpoints / 67 `.sv`
    - `int_nodeid_cse_default`: 67 checkpoints / 67 `.sv`
    - `int_nodeid_operand-unique_default`: 10 checkpoints / 11 `.sv`
  - zero Verilator warning logs
  - zero Yosys `Warning:` lines
- full repo hygiene:
  - `cargo check --all-targets`
  - `cargo test`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo fmt --all --check`
  - `mdbook build book`

**Impact**

- `/tmp/anvil-tool-matrix-phase1-real-r21` remains the live resumable
  current-code both-mode frontier tree.
- The fresh-tree frontier has now fully re-closed `relaxed`,
  `nodeid-none`, and `nodeid-cse` on current code, with
  `operand-unique` underway.
- The next frontier push should resume `r21` in place and keep climbing
  through `operand-unique`, then `commutative`, toward the repaired
  `e-graph` surface.

## 2026-04-22-0118 — Bank a fresh current-code both-mode frontier tree

**Landed as:** this commit

**What changed**

No source changes in this slice. I started a fresh current-code
`tool_matrix --phase1-gate --yosys-mode both` tree at:

- `/tmp/anvil-tool-matrix-phase1-real-r21`

and pushed it cleanly past the initial repaired-code boundary instead of
resuming the now-historical `r20` tree.

I intentionally interrupted the run at the first useful fresh-tree
checkpoint after the current-code `nodeid-none` rung had fully closed
and `cse` had already started.

**Why**

The previous slice repaired a real `e-graph` warning by changing
generator semantics in `src/gen/cone.rs`. Because `tool_matrix --resume`
is intentionally byte-stable, that made `/tmp/anvil-tool-matrix-phase1-real-r20`
evidence only rather than a safe live resume target.

So the next honest move was not "pretend r20 is still live"; it was to
start a fresh tree on current code and bank the first new frontier
checkpoint there.

**Validation**

- fresh current-code both-mode frontier:
  - `cargo run --bin tool_matrix -- --out /tmp/anvil-tool-matrix-phase1-real-r21 --phase1-gate --yosys-mode both`
  - intentionally interrupted after **139** completed checkpoints /
    **140** emitted `.sv` files
  - scenario coverage at the checkpoint:
    - `int_relaxed_none_default`: 67 checkpoints / 67 `.sv`
    - `int_nodeid_none_default`: 67 checkpoints / 67 `.sv`
    - `int_nodeid_cse_default`: 5 checkpoints / 6 `.sv`
  - zero Verilator warning logs
  - zero Yosys `Warning:` lines
- full repo hygiene:
  - `cargo check --all-targets`
  - `cargo test`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo fmt --all --check`
  - `mdbook build book`

**Impact**

- `/tmp/anvil-tool-matrix-phase1-real-r21` is now the live resumable
  current-code both-mode frontier tree.
- The fresh-tree frontier has already re-closed the full relaxed and
  full `nodeid-none` rungs on current code, with `cse` underway.
- The next frontier push should resume `r21` in place and keep climbing
  through `cse`, then `operand-unique`, toward the repaired `e-graph`
  surface.

## 2026-04-22-0117 — Preserve wrapped-add lower bounds for overshift proofs

**Landed as:** this commit

**What changed**

Strengthened generator-side unsigned bounds in
`src/gen/cone.rs` for one specific but important case: when an `Add`
node has exactly one non-exact interval operand and the rest of its
operands are exact constants, ANVIL now keeps the shifted interval
instead of dropping straight to full-range as soon as wrapped addition
appears.

That repair is intentionally narrow. It does not reopen broad exact-set
enumeration or relax the small-support guards. It only preserves the
useful lower bound when a single live interval is translated by exact
wrapped constants and the translated interval stays linear in unsigned
space.

This slice also adds a regression test that exercises the real failure
shape without relying on the small-value-set engine:

- `add_bounds_preserve_shifted_single_interval_without_small_set_help`

**Why**

The live current-code both-mode `r20` frontier surfaced a real Yosys
warning in:

- `/tmp/anvil-tool-matrix-phase1-real-r20/int_nodeid_e-graph_default/mod_8_0053.sv`

The warning came from `PEEPOPT` and complained that a shift-add
candidate was ignored to avoid high resource usage. The underlying
generator issue was narrower than that warning text: ANVIL had emitted a
shift whose rhs was provably always oversized, but the proof was missed
because the rhs was a wrapped add of one non-exact interval plus exact
constants. The old `Add` bounds logic collapsed that case to full-range,
so the overshift proof never fired even though the rhs lower bound was
still available.

**Validation**

- `cargo test add_bounds_preserve_shifted_single_interval_without_small_set_help --lib`
- `cargo test prove_node_exact_value_detects_overshift_from_wrapped_small_rhs_set --lib`
- focused current-code repro:
  - `cargo run --bin anvil -- --seed 8 --count 54 --out /tmp/anvil-egraph-seed8-repro-r2 --construction-strategy interleaved --identity-mode node-id --factorization-level e-graph`
- direct culprit-file proof:
  - `verilator --lint-only /tmp/anvil-egraph-seed8-repro-r2/mod_8_0053.sv`
  - `yosys -p 'read_verilog -sv "/tmp/anvil-egraph-seed8-repro-r2/mod_8_0053.sv"; synth -noabc; stat'`
  - `yosys -p 'read_verilog -sv "/tmp/anvil-egraph-seed8-repro-r2/mod_8_0053.sv"; synth -noabc; abc -fast; opt -fast; stat; check'`
- full focused batch proof on the same seed-8 e-graph repro tree:
  - Verilator: `54/54`
  - Yosys `synth -noabc`: `54/54`
  - Yosys `synth -noabc; abc -fast; opt -fast; stat; check`: `54/54`
- full repo hygiene:
  - `cargo check --all-targets`
  - `cargo test`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo fmt --all --check`
  - `mdbook build book`

**Impact**

- The old `mod_8_0053.sv` `PEEPOPT` warning is gone on current code.
- The focused seed-8 `e-graph` repro batch is now clean across all 54
  modules in Verilator and both repo-owned Yosys modes.
- `/tmp/anvil-tool-matrix-phase1-real-r20` remains valuable historical
  evidence, but it should no longer be resumed on current code because
  this generator semantic change can alter emitted `.sv` and
  `tool_matrix --resume` is intentionally byte-stable.
- The next frontier push should therefore start from a fresh current-code
  tree, likely `/tmp/anvil-tool-matrix-phase1-real-r21`.

## 2026-04-22-0116 — Close the current-code both-mode peephole rung

**Landed as:** _to be filled in after this commit_

**What changed**

No source changes in this slice either. I resumed the live current-code
both-mode `tool_matrix` frontier in place again and pushed it past the
partial `peephole` checkpoint to full clean closure of that rung.

The run was resumed from:

- `/tmp/anvil-tool-matrix-phase1-real-r20`

using:

- `cargo run --bin tool_matrix -- --out /tmp/anvil-tool-matrix-phase1-real-r20 --phase1-gate --yosys-mode both --resume`

and then intentionally interrupted at the next natural checkpoint,
after `peephole` was fully banked and `e-graph` had already started
cleanly.

**Why**

The previous recorded `r20` checkpoint had only banked 13 clean
`peephole` modules. The next useful question was whether current code
could close the whole `peephole` rung cleanly in the stricter both-mode
lane, not just whether the earlier warning/runtime repairs held on a
prefix.

That closure now exists, and the run even stepped into `e-graph`, so
this slice is another durable recovery/evidence checkpoint.

**Validation**

- resumed current-code both-mode frontier:
  - `cargo run --bin tool_matrix -- --out /tmp/anvil-tool-matrix-phase1-real-r20 --phase1-gate --yosys-mode both --resume`
  - intentionally interrupted after **570** completed checkpoints /
    **571** emitted `.sv` files
  - scenario coverage at the checkpoint:
    - `int_relaxed_none_default`: 67
    - `int_nodeid_none_default`: 67
    - `int_nodeid_cse_default`: 67
    - `int_nodeid_operand-unique_default`: 67
    - `int_nodeid_commutative_default`: 67
    - `int_nodeid_associative_default`: 67
    - `int_nodeid_constant-fold_default`: 67
    - `int_nodeid_peephole_default`: 67
    - `int_nodeid_e-graph_default`: 34
  - zero Verilator warning logs
  - zero Yosys `Warning:` lines
- `cargo check --all-targets`
- `cargo test`
- `cargo clippy --all-targets -- -D warnings`
- `cargo fmt --all --check`
- `mdbook build book`

**Impact**

- The live current-code resumable both-mode frontier at
  `/tmp/anvil-tool-matrix-phase1-real-r20` now covers all clean
  scenarios through `peephole`, plus 34 clean `e-graph` checkpoints.
- The stronger both-mode current-code frontier now stands at **570**
  completed checkpoints / **571** emitted `.sv` files with zero
  warning artifacts.
- `r20` remains the live resumable tree on current code, and the next
  natural push is deeper into `int_nodeid_e-graph_default`.

## 2026-04-22-0115 — Close the current-code both-mode constant-fold rung

**Landed as:** `9fcd782`

**What changed**

No source changes in this slice either. I resumed the live current-code
both-mode `tool_matrix` frontier in place once more and pushed it past
the partial `constant-fold` checkpoint to full clean closure of that
rung.

The run was resumed from:

- `/tmp/anvil-tool-matrix-phase1-real-r20`

using:

- `cargo run --bin tool_matrix -- --out /tmp/anvil-tool-matrix-phase1-real-r20 --phase1-gate --yosys-mode both --resume`

and then intentionally interrupted at the next natural checkpoint,
after `constant-fold` was fully banked and `peephole` had already
started cleanly.

**Why**

The previous recorded `r20` checkpoint had only banked 22 clean
`constant-fold` modules. The next useful question was whether current
code could close the whole `constant-fold` rung cleanly in the stricter
both-mode lane, not just whether the earlier warning/runtime repairs
held on a prefix.

That closure now exists, and the run even stepped into `peephole`, so
this slice is another durable recovery/evidence checkpoint.

**Validation**

- resumed current-code both-mode frontier:
  - `cargo run --bin tool_matrix -- --out /tmp/anvil-tool-matrix-phase1-real-r20 --phase1-gate --yosys-mode both --resume`
  - intentionally interrupted after **482** completed checkpoints /
    **483** emitted `.sv` files
  - scenario coverage at the checkpoint:
    - `int_relaxed_none_default`: 67
    - `int_nodeid_none_default`: 67
    - `int_nodeid_cse_default`: 67
    - `int_nodeid_operand-unique_default`: 67
    - `int_nodeid_commutative_default`: 67
    - `int_nodeid_associative_default`: 67
    - `int_nodeid_constant-fold_default`: 67
    - `int_nodeid_peephole_default`: 13
  - zero Verilator warning logs
  - zero Yosys `Warning:` lines
- `cargo check --all-targets`
- `cargo test`
- `cargo clippy --all-targets -- -D warnings`
- `cargo fmt --all --check`
- `mdbook build book`

**Impact**

- The live current-code resumable both-mode frontier at
  `/tmp/anvil-tool-matrix-phase1-real-r20` now covers all clean
  scenarios through `constant-fold`, plus 13 clean `peephole`
  checkpoints.
- The stronger both-mode current-code frontier now stands at **482**
  completed checkpoints / **483** emitted `.sv` files with zero
  warning artifacts.
- `r20` remains the live resumable tree on current code, and the next
  natural push is deeper into `int_nodeid_peephole_default`.

## 2026-04-22-0114 — Close the current-code both-mode associative rung

**Landed as:** `4fb5761`

**What changed**

No source changes in this slice either. I resumed the live current-code
both-mode `tool_matrix` frontier in place again and pushed it past the
partial `associative` checkpoint to full clean closure of that rung.

The run was resumed from:

- `/tmp/anvil-tool-matrix-phase1-real-r20`

using:

- `cargo run --bin tool_matrix -- --out /tmp/anvil-tool-matrix-phase1-real-r20 --phase1-gate --yosys-mode both --resume`

and then intentionally interrupted at the next natural checkpoint,
after `associative` was fully banked and `constant-fold` had already
started cleanly.

**Why**

The previous recorded `r20` checkpoint had only banked 11 clean
`associative` modules. The next useful question was whether current
code could close the whole `associative` rung cleanly in the stricter
both-mode lane, not just whether the earlier warning fixes held on a
small prefix.

That closure now exists, and the run even stepped into
`constant-fold`, so this slice is another durable recovery/evidence
checkpoint.

**Validation**

- resumed current-code both-mode frontier:
  - `cargo run --bin tool_matrix -- --out /tmp/anvil-tool-matrix-phase1-real-r20 --phase1-gate --yosys-mode both --resume`
  - intentionally interrupted after **424** completed checkpoints /
    **425** emitted `.sv` files
  - scenario coverage at the checkpoint:
    - `int_relaxed_none_default`: 67
    - `int_nodeid_none_default`: 67
    - `int_nodeid_cse_default`: 67
    - `int_nodeid_operand-unique_default`: 67
    - `int_nodeid_commutative_default`: 67
    - `int_nodeid_associative_default`: 67
    - `int_nodeid_constant-fold_default`: 22
  - zero Verilator warning logs
  - zero Yosys `Warning:` lines
- focused current-code `associative` module repro:
  - `cargo run --bin anvil -- --seed 5 --count 16 --out /tmp/anvil-associative-seed5-repro-r8 --construction-strategy interleaved --identity-mode node-id --factorization-level associative`
  - direct checks on `mod_5_0015.sv` are clean in:
    - Verilator
    - Yosys `synth -noabc`
    - Yosys `synth -noabc; abc -fast; opt -fast; stat; check`
- `cargo check --all-targets`
- `cargo test`
- `cargo clippy --all-targets -- -D warnings`
- `cargo fmt --all --check`
- `mdbook build book`

**Impact**

- The live current-code resumable both-mode frontier at
  `/tmp/anvil-tool-matrix-phase1-real-r20` now covers all clean
  scenarios through `associative`, plus 22 clean `constant-fold`
  checkpoints.
- The stronger both-mode current-code frontier now stands at **424**
  completed checkpoints / **425** emitted `.sv` files with zero
  warning artifacts.
- `r20` remains the live resumable tree on current code, and the next
  natural push is deeper into `int_nodeid_constant-fold_default`.

## 2026-04-22-0113 — Advance the live `r20` both-mode frontier through commutative

**Landed as:** `dfe3285`

**What changed**

No source changes in this slice. I resumed the live current-code
both-mode `tool_matrix` frontier in place and banked a much deeper
warning-clean checkpoint.

The run was resumed from:

- `/tmp/anvil-tool-matrix-phase1-real-r20`

using:

- `cargo run --bin tool_matrix -- --out /tmp/anvil-tool-matrix-phase1-real-r20 --phase1-gate --yosys-mode both --resume`

and then intentionally interrupted again at the next good checkpoint.

**Why**

The previous `r20` checkpoint had only just entered
`int_nodeid_operand-unique_default` on current code. The next useful
move was not another local proof tweak; it was to push the same tree
forward and see whether the repaired code would stay clean through the
rest of `operand-unique`, the whole `commutative` rung, and into
`associative`.

That is exactly what happened, so this slice is evidence and recovery
state rather than a code change.

**Validation**

- resumed current-code both-mode frontier:
  - `cargo run --bin tool_matrix -- --out /tmp/anvil-tool-matrix-phase1-real-r20 --phase1-gate --yosys-mode both --resume`
  - intentionally interrupted after **346** completed checkpoints /
    **347** emitted `.sv` files
  - scenario coverage at the checkpoint:
    - `int_relaxed_none_default`: 67
    - `int_nodeid_none_default`: 67
    - `int_nodeid_cse_default`: 67
    - `int_nodeid_operand-unique_default`: 67
    - `int_nodeid_commutative_default`: 67
    - `int_nodeid_associative_default`: 11
  - zero Verilator warning logs
  - zero Yosys `Warning:` lines
- `cargo check --all-targets`
- `cargo test`
- `cargo clippy --all-targets -- -D warnings`
- `cargo fmt --all --check`
- `mdbook build book`

**Impact**

- The live current-code resumable both-mode frontier at
  `/tmp/anvil-tool-matrix-phase1-real-r20` now covers all clean
  scenarios through `commutative`, plus 11 clean `associative`
  checkpoints.
- The stronger both-mode current-code frontier now stands at **346**
  completed checkpoints / **347** emitted `.sv` files with zero
  warning artifacts.
- `r20` remains the live resumable tree on current code, and the next
  natural push is deeper into `int_nodeid_associative_default`.

## 2026-04-22-0112 — Fold large-endpoint overshifts and re-bank the fresh CSE frontier

**Landed as:** `3ddcfbd`

**What changed**

I tightened the post-construction cleanup pass one more step in a very
specific way.

`src/ir/compact.rs` already kept the general cleanup exact prover
tiny-only and already had a compare-specific escape hatch for bounded
unsigned tautologies. This slice adds the analogous narrow exception
for `Shl` / `Shr`: cleanup may now ask for the **bounds-only** exact
result of a shift gate even when the cone is too large for the general
cleanup exact gate.

To support that, `src/gen/cone.rs` now exposes
`prove_node_exact_value_from_bounds`, which skips the small-set
enumerator and returns only what `node_unsigned_bounds` can prove
exactly. `src/ir/compact.rs` uses that helper only for shift gates.

I also added a regression test for the exact missed shape:

- `fold_proven_gates_revisits_large_endpoint_overshift_shift`

That test forces a shift whose rhs depends on four endpoint variables,
so the general cleanup exact gate rejects it, but the bounds are still
enough to prove the shift result is always zero.

**Why**

The fresh current-code both-mode frontier at
`/tmp/anvil-tool-matrix-phase1-real-r19` got cleanly past the old
runtime/associative warning fixes and then stopped at the next real
downstream warning in `int_nodeid_cse_default/mod_2_0018.sv`:

- `Warning: ... result of shift operation is always constant ...`

The offending shape was:

- `shr_2 = mux_10 >> 3'h6`
- `add_5 = shr_2 + 8'h7`
- `shr_3 = 2'h1 >> add_5`

So the rhs was always at least `7`, while the lhs width was only `2`.
This was exactly the kind of cheap always-overshift proof we *want*
cleanup to catch, but the cone was large enough that the general
cleanup exact gate refused to revisit it.

So the load-bearing adjustment is now:

- keep the broad cleanup exact path tiny-only;
- still allow cheap, bounded shift exactness via the bounds engine.

That preserves runtime discipline while closing a real downstream-tool
warning class.

**Validation**

- targeted unit test:
  - `cargo test fold_proven_gates_revisits_large_endpoint_overshift_shift --lib`
- focused current-code repro:
  - `cargo run --bin anvil -- --seed 2 --count 19 --out /tmp/anvil-cse-seed2-repro-r2 --construction-strategy interleaved --identity-mode node-id --factorization-level cse`
  - all **19** emitted modules are now clean in:
    - Verilator
    - Yosys `synth -noabc`
    - Yosys `synth -noabc; abc -fast; opt -fast; stat; check`
- fresh current-code both-mode frontier:
  - `cargo run --bin tool_matrix -- --out /tmp/anvil-tool-matrix-phase1-real-r20 --phase1-gate --yosys-mode both`
  - intentionally interrupted after **211** completed checkpoints /
    **212** emitted `.sv` files
  - scenario coverage at the checkpoint:
    - `int_relaxed_none_default`: 67
    - `int_nodeid_none_default`: 67
    - `int_nodeid_cse_default`: 67
    - `int_nodeid_operand-unique_default`: 10
  - zero Verilator warning logs
  - zero Yosys `Warning:` lines
- `cargo check --all-targets`
- `cargo test`
- `cargo clippy --all-targets -- -D warnings`
- `cargo fmt --all --check`
- `mdbook build book`

**Impact**

- Large-endpoint always-overshift shift nodes are now cleaned up
  without reopening the broad expensive cleanup proof path.
- The old `r19` CSE warning on `mod_2_0018.sv` is fixed at the
  generator/finalization layer.
- There is now a fresh current-code both-mode frontier at
  `/tmp/anvil-tool-matrix-phase1-real-r20` with **211** completed
  checkpoints / **212** emitted `.sv` files, all warning-clean.
- `r20` is the new live resumable tree on current code.

## 2026-04-22-0111 — Cut clone-heavy rollback churn and scrub associative compare warnings

**Landed as:** `49286ef`

**What changed**

I tightened two generator/finalization hotspots that showed up while
trying to resume the fresh current-code both-mode associative frontier.

First, `src/gen/cone.rs` no longer snapshots whole construction state
by cloning the signal pool, worklist, and dedup tables on every
retry/anti-collapse rollback. It now snapshots lengths only, truncates
back to those markers, and prunes stale dedup-table entries whose
`NodeId`s were rewound. `src/gen/pool.rs` grew `len()` / `truncate()`
to support that rollback path, and a new regression test,
`rollback_snapshot_truncates_pool_and_prunes_stale_dedup_entries`,
pins the invariant.

Second, I tightened the exact-proof surface that cleanup is allowed to
use, but without giving up on warning cleanup:

- `src/ir/compact.rs` still keeps the general cleanup exact prover
  behind the existing tiny-cone eligibility gate, but it now runs the
  bounded unsigned-compare proof on compare gates even when the cone is
  too large for the general cleanup exact path.
- `src/gen/cone.rs` now gives `Shr` tighter bounds when the lhs is an
  exact constant and the rhs is only range-known. That lets ANVIL prove
  shapes like `&(4'h1 >> dynamic_rhs) == 0`, which in turn lets
  compare cleanup collapse warnings such as `sub_37 < red_and_8`.

I added two more regression tests for those exact gaps:

- `prove_node_exact_value_detects_reduction_zero_from_dynamic_single_bit_shr`
- `fold_proven_gates_revisits_large_endpoint_unsigned_compare`

**Why**

The first resume attempt against `/tmp/anvil-tool-matrix-phase1-real-r18`
was not actually stuck in Yosys. Sampling showed the first hotspot in
generator rollback churn (`build_cone_with_retry` / anti-collapse),
with lots of `SignalPool` / `DepSet` clone traffic. After fixing that,
the next hotspot moved to post-construction cleanup
(`fold_proven_gates` leaning too hard on exact-value proofs over large
associative cones).

Once those runtime issues were out of the way, a fresh focused
associative repro finally surfaced the next real downstream warning:

- Verilator `CMPCONST` on `mod_5_0030.sv`
- then Verilator `UNSIGNED` on `mod_5_0005.sv`

Those warnings were legitimate ANVIL cleanup gaps, not tool noise:

- `sub_37 < red_and_8` was constant because `red_and_8` was really the
  reduction-AND of `4'h1 >> dynamic_rhs`, which can never become all
  ones.
- `add_0 >= mux_27` was constant because the rhs mux was dead on a
  selector proven zero, so the comparison was really `unsigned_x >= 0`.

The fix therefore had to be a real proof-cleanup improvement, not just
more permissive harness logic.

**Validation**

- targeted unit tests:
  - `cargo test rollback_snapshot_truncates_pool_and_prunes_stale_dedup_entries --lib`
  - `cargo test prove_node_exact_value_detects_reduction_zero_from_dynamic_single_bit_shr --lib`
  - `cargo test cleanup_exact_proof_skips_four_endpoint_cones --lib`
  - `cargo test fold_proven_gates_revisits_large_endpoint_unsigned_compare --lib`
- focused fresh associative repro:
  - `cargo run --bin anvil -- --seed 5 --count 38 --out /tmp/anvil-associative-seed5-repro-r6 --construction-strategy interleaved --identity-mode node-id --factorization-level associative`
  - all **38** emitted modules are now clean in:
    - Verilator
    - Yosys `synth -noabc`
    - Yosys `synth -noabc; abc -fast; opt -fast; stat; check`
- attempted `tool_matrix --resume` against `/tmp/anvil-tool-matrix-phase1-real-r18`
  now correctly reports byte-stable mismatch (`existing SV differs from
  regenerated module`), so `r18` is historical evidence only on this
  code and should not be resumed in place.
- `cargo check --all-targets`
- `cargo test`
- `cargo clippy --all-targets -- -D warnings`
- `cargo fmt --all --check`
- `mdbook build book`

**Impact**

- Retry / anti-collapse rollback is materially cheaper on deep
  associative shapes because it truncates and prunes instead of cloning
  whole construction state.
- Post-construction cleanup no longer misses large-endpoint unsigned
  compare tautologies just because the general cleanup exact gate says
  "too big".
- The previously failing fresh associative focused repro is now clean
  through **38/38** modules under Verilator and both repo-owned Yosys
  modes.
- `/tmp/anvil-tool-matrix-phase1-real-r18` remains a valuable proof
  checkpoint at 372 completed modules / 373 emitted `.sv` files, but it
  is no longer resumable on the current code because the emitted `.sv`
  changed again.

## 2026-04-21-0110 — Silence associative shiftadd warnings and push the both-mode frontier to 372

**Landed as:** `ba6d69b`

**What changed**

I tightened the generator-side shift proof in `src/gen/cone.rs`.

The key change is that overshift detection no longer relies only on the
general small-support value-set engine. `small_value_set_min_at_least`
now has a tiny-domain fallback that can still reason about narrow
boolean-mask arithmetic even when the cone's full support is too large
for the normal exact enumerator.

That tiny-domain fallback is deliberately narrow:

- width must still be at most `8` bits;
- it only tracks very small result sets (current cap: `8` values);
- and it exists mainly to prove things like
  "replicated 1-bit mask plus constant is always >= source width".

I also added a regression test,
`prove_node_exact_value_detects_overshift_from_wrapped_small_rhs_set`,
which pins the exact warning pattern that showed up in the
`associative` frontier: a shift amount built from a large-support
1-bit control, replicated to an 8-bit mask and then offset by a
constant.

**Why**

The fresh `r17` both-mode frontier got deeper than any earlier
current-code run, then stopped on a real Yosys warning in
`int_nodeid_associative_default/mod_5_0011.sv`:

- `Warning: ... candiate for shiftadd optimization ... was ignored to avoid high resource usage`

The offending line was:

- `assign shr_15 = sub_25 >> add_44;`

and the shift amount simplified downstream to a boolean-controlled
mask-plus-constant expression that was **always** far above the source
width. So this was not "Yosys being noisy"; it was ANVIL missing a real
always-overshift proof and therefore emitting a pointless dynamic shift
that downstream tools still had to analyze.

The first fix attempt only helped when the rhs itself stayed inside the
main small-support enumerator. The live warning proved that was still
too conservative. The durable rule is now:

- shift overshift proofs may use a tiny-domain rhs fallback for narrow
  boolean-mask arithmetic, even when the whole cone is too large for
  the general exact small-set engine.

**Validation**

- `cargo test prove_node_exact_value_detects_overshift_from_wrapped_small_rhs_set --lib`
- focused current-code repro:
  - `cargo run --bin anvil -- --seed 5 --count 12 --out /tmp/anvil-associative-seed5-repro-r1 --construction-strategy interleaved --identity-mode node-id --factorization-level associative`
  - all 12 emitted modules are now clean in:
    - Verilator
    - Yosys `synth -noabc`
    - Yosys `synth -noabc; abc -fast; opt -fast; stat; check`
- fresh current-code real both-mode frontier:
  - `cargo run --bin tool_matrix -- --out /tmp/anvil-tool-matrix-phase1-real-r18 --phase1-gate --yosys-mode both`
  - intentionally interrupted after **372** completed module
    checkpoints / **373** emitted `.sv` files
  - zero Verilator warning logs
  - zero Yosys `Warning:` lines
- `cargo check --all-targets`
- `cargo test`
- `cargo clippy --all-targets -- -D warnings`
- `cargo fmt --all --check`
- `mdbook build book`

**Impact**

- The old associative Yosys `peepopt` / `shiftadd` warning is now
  fixed at the generator layer.
- The fresh current-code both-mode frontier is now parked at
  `/tmp/anvil-tool-matrix-phase1-real-r18` with **372** completed
  checkpoints:
  - `67` relaxed
  - `67` nodeid-none
  - `67` nodeid-cse
  - `67` nodeid-operand-unique
  - `67` nodeid-commutative
  - `37` nodeid-associative
- That overtakes the older 368-module both-mode checkpoint.
- Because the run was intentionally interrupted on a checkpoint
  boundary, there is no final `tool_matrix_report.json` yet; `r18`
  remains resumable in place on this code.

## 2026-04-21-0109 — Cap cleanup semantic proofs to tiny endpoint sets

**Landed as:** `f0567ff`

**What changed**

I tightened the post-construction exact-value cleanup path in
`src/ir/compact.rs`.

`fold_proven_gates` still revisits the settled graph to scrub obvious
constants for downstream-tool cleanliness, but its local semantic exact
fallback (`semantic_exact_value`) is now explicitly gated by a new
helper, `semantic_exact_cleanup_eligible`. That helper only admits
cleanup-time brute-force proofs when all three of these are true:

- node width is at most `8` bits;
- the canonical leaf-endpoint set is at most **3** endpoints; and
- the total endpoint support is at most
  `MAX_SEMANTIC_SUPPORT_BITS` (`10`).

When a cone fails that eligibility test, the cleanup prover now
memoizes `None` immediately instead of repeatedly re-entering the full
semantic walk.

I also added a regression test,
`semantic_exact_cleanup_skips_four_endpoint_cones`, which pins the new
cleanup-only cap on a 4-endpoint `Concat`.

**Why**

The fresh current-code both-mode frontier at
`/tmp/anvil-tool-matrix-phase1-real-r16` looked like a downstream-tool
stall at first glance, but sampling the live process showed the real
hotspot was internal:

- `anvil::ir::compact::fold_proven_gates`
- `semantic_exact_value`
- `semantic_cone_proof`
- `evaluate_node_under_assignment`

So this was not Yosys or Verilator getting stuck. The settled-graph
cleanup prover had become too eager on a `nodeid-cse` cone, and it was
burning time exploring a support set that is outside the narrow
"downstream cleanup" job of that pass.

The durable rule is now explicit in code: the cleanup-only exact prover
must stay **stricter** than the generator-side semantic-sharing passes.
It exists to scrub obvious constants for cleaner emitted RTL, not to
widen the main identity/factorization contract at arbitrary runtime
cost.

**Validation**

- `sample <tool_matrix_pid> 5`
  - showed the hot path in `ir::compact`, not in Yosys / Verilator
- `cargo test semantic_exact_cleanup_skips_four_endpoint_cones --lib`
- `cargo run --bin anvil -- --seed 2 --count 2 --out /tmp/anvil-cse-seed2-repro-r2 --construction-strategy interleaved --identity-mode node-id --factorization-level cse`
- clean downstream sweeps on both emitted repro modules:
  - `verilator --lint-only`
  - `yosys -q -p "read_verilog -sv ...; synth -noabc; stat"`
  - `yosys -q -p "read_verilog -sv ...; synth -noabc; abc -fast; opt -fast; stat; check"`
- `cargo check --all-targets`
- `cargo test`
- `cargo clippy --all-targets -- -D warnings`
- `cargo fmt --all --check`
- `mdbook build book`

**Impact**

- The old `nodeid-cse` choke point is now repaired in current code.
- `/tmp/anvil-tool-matrix-phase1-real-r16` is no longer a resumable
  work tree, because `tool_matrix --resume` is intentionally
  byte-stable and this semantic-cleanup change altered emitted `.sv`.
- The next frontier push must therefore start from a fresh output tree
  rather than trying to resume `r16` in place.

## 2026-04-21-0108 — Record fresh current-code nodeid-none frontier

**Landed as:** `4023050`

**What changed**

I pushed a fresh current-code real both-mode Phase 1 tree at
`/tmp/anvil-tool-matrix-phase1-real-r16` and stopped it on a clean
checkpoint boundary that is actually meaningful for the repaired proof
engine:

- `int_relaxed_none_default`: `67/67` completed module checkpoints
- `int_nodeid_none_default`: `67/67` completed module checkpoints
- `int_nodeid_cse_default`: `1/67` completed module checkpoints

At the stop point the tree contains:

- **135** completed `*.module-report.json` checkpoints
- **136** emitted `.sv` files
- **0** Verilator warning logs
- **0** Yosys `Warning:` lines across both Yosys modes

Because the harness was intentionally interrupted on a checkpointed
tree instead of finishing the full matrix, there is no final
`tool_matrix_report.json` yet. The saved tree remains resumable in
place via `--resume`.

I also refreshed the live recovery docs with that checkpoint and cleaned
up one stale doc drift in `CODEBASE_ANALYSIS.md`: the duplicated
"Generator-output smoke" bullet is now collapsed back to one bullet, and
the test count is corrected to the current `160` passing total.

**Why**

The previous fresh current-code frontier (`r12`) was useful, but it was
stale across the later proof-engine fixes. The next useful question was
not "does relaxed still work?" but "does the repaired current code now
carry the full `nodeid-none` rung cleanly under Verilator and both
repo-owned Yosys modes on a real matrix tree?"

The answer is now "yes": the fresh current-code tree has cleared the
full relaxed baseline and the full `nodeid-none` scenario with no
warning artifacts, then stepped one checkpoint into `cse` before the
intentional stop.

**Validation**

- `cargo run --bin tool_matrix -- --out /tmp/anvil-tool-matrix-phase1-real-r16 --phase1-gate --yosys-mode both`
  - intentionally interrupted after checkpoint counts reached the full
    `nodeid-none` boundary
- `find /tmp/anvil-tool-matrix-phase1-real-r16/int_relaxed_none_default -name '*.module-report.json' | wc -l` -> `67`
- `find /tmp/anvil-tool-matrix-phase1-real-r16/int_nodeid_none_default -name '*.module-report.json' | wc -l` -> `67`
- `find /tmp/anvil-tool-matrix-phase1-real-r16/int_nodeid_cse_default -name '*.module-report.json' | wc -l` -> `1`
- `find /tmp/anvil-tool-matrix-phase1-real-r16 -name '*.module-report.json' | wc -l` -> `135`
- `find /tmp/anvil-tool-matrix-phase1-real-r16 -name '*.sv' | wc -l` -> `136`
- `find /tmp/anvil-tool-matrix-phase1-real-r16 -name '*.verilator.stderr.log' | wc -l` -> `0`
- `find /tmp/anvil-tool-matrix-phase1-real-r16 -name '*.stdout.log' -path '*yosys*' -print0 | xargs -0 rg -n '^Warning:' | wc -l` -> `0`
- `cargo check --all-targets`
- `cargo test`
- `cargo clippy --all-targets -- -D warnings`
- `cargo fmt --all --check`
- `mdbook build book`

**Impact**

- There is now a fresh current-code both-mode frontier parked in
  `/tmp/anvil-tool-matrix-phase1-real-r16`, not only the older `r12`
  evidence tree.
- The repaired proof engine is now proven across the entire
  `nodeid-none` scenario in a real matrix run, not just on a focused
  seed-1 repro.
- The next PNT can resume `r16` directly and keep climbing through
  `cse`, `operand-unique`, and beyond instead of replaying the
  relaxed/none prefix again.

## 2026-04-21-0107 — Clarify NodeId doctrine across docs

**Landed as:** `23fece6`

**What changed**

I tightened the identity/factorization story across the live docs, the
book, and the key Rustdoc surfaces:

- `README.md`, `USER_GUIDE.md`, `ROADMAP.md`,
  `DEVELOPMENT_NOTES.md`, and `CODEBASE_ANALYSIS.md`
- `book/src/factorization.md`, `book/src/structural-rules.md`,
  `book/src/knobs.md`, `book/src/ir.md`, and `book/src/faq.md`
- `src/config.rs` and `src/ir/types.rs`

They now all say the same thing:

- `identity_mode = node-id` means `NodeId` is the identity of an
  expression, which implies full factorization by definition;
- `identity_mode = relaxed` is the only intentional mode where
  equivalent expressions may keep different `NodeId`s; and
- `factorization_level` is the current-build implementation /
  proof-depth dial inside `node-id`, not a competing definition of
  what `node-id` means.

I also corrected one smaller wording drift in the book:
`operand_duplication_rate = 0.0` now says what it really guarantees
(`Add`/`Mul` operand uniqueness), instead of overclaiming that it is
full factorization by itself.

**Why**

The docs had developed a real conceptual blur. Several core surfaces
still described `node-id` as merely "keeping the ladder live", which
made labels like `nodeid-none` sound as if they were alternate
definitions of `NodeId` identity.

That is not the doctrine the project is steering toward. The durable
interpretation is stricter:

- `node-id` = full-factorization doctrine
- `relaxed` = allow equivalent expressions to keep different names

The ladder still exists, but only as a staged implementation / proof
dial while the current build climbs toward that doctrine.

**Validation**

- repo-wide wording audit with `rg`
- `cargo check --all-targets`
- `cargo test`
- `cargo clippy --all-targets -- -D warnings`
- `cargo fmt --all --check`
- `mdbook build book`

**Impact**

- The live docs and the mdBook now steer future implementation with the
  same interpretation the user stated explicitly.
- `nodeid-none` is now documented as a matrix/debug rung in the current
  build, not as a different semantics for `node-id`.
- Future code changes have less room to smuggle the implementation
  ladder into the meaning of `NodeId` identity itself.

## 2026-04-21-0106 — Fold reflexive subtraction before unsigned compare emit

**Landed as:** `3a4f7c9`

**What changed**

`src/gen/cone.rs` now teaches the cheap exact/bounds proof layer the
missing reflexive subtraction identity:

- `x - x = 0` now folds immediately in `exact_gate_value`; and
- the bounds layer now also treats `Sub(lhs, lhs)` as the exact range
  `(0, 0)` even when `lhs` itself is not exact.

I also added a regression test,
`comparison_range_fold_proves_lt_against_reflexive_sub_zero`, which
pins the exact unsigned-warning shape down.

**Why**

The first fresh current-code both-mode frontier after the small-support
cap hit a real warning at
`/tmp/anvil-tool-matrix-phase1-real-r15/int_nodeid_none_default/mod_1_0022.sv:1019`:

`add_13 < and_49`

Verilator was right. In that file, `and_49` included:

- `sub_16 = mul_17 - mul_17`

so the RHS was provably zero and the unsigned `<` comparison was
constant false.

The hole was not in the new small-support gate. It was simpler:
the cheap proof layers already knew reflexive comparison tautologies
like `x < x = 0`, but they did **not** yet know the arithmetic identity
`x - x = 0` unless `x` itself had already become exact.

That let the zero stay hidden behind a non-exact producer and leak all
the way to emitted RTL.

**Validation**

- `cargo test comparison_range_fold --lib`
- Focused repro for the exact failing frontier scenario:
  - `cargo run --bin anvil -- --seed 1 --count 23 --out /tmp/anvil-nodeid-none-seed1-repro-r1 --construction-strategy interleaved --identity-mode node-id --factorization-level none`
  - now emits through `mod_1_0022.sv`
- Downstream tool proof on the exact offender:
  - `verilator --lint-only /tmp/anvil-nodeid-none-seed1-repro-r1/mod_1_0022.sv`
  - `yosys -p 'read_verilog -sv \".../mod_1_0022.sv\"; synth -noabc; stat'`
  - `yosys -p 'read_verilog -sv \".../mod_1_0022.sv\"; synth -noabc; abc -fast; opt -fast; stat; check'`
- Stronger scenario proof on the full 23-module batch:
  - Verilator: `23/23` pass, `0` warnings
  - Yosys without-abc: `23/23` pass, `0` warnings
  - Yosys with-abc: `23/23` pass, `0` warnings

**Impact**

- The first fresh current-code both-mode `nodeid-none` warning boundary
  is now clean.
- The generator-side comparison proof now covers another important
  unsigned-cleanliness identity without relying on the heavier exact
  finite-set engine.
- The next frontier push can restart from a fresh current-code tree
  instead of carrying this warning forward.

**Files touched**

- `src/gen/cone.rs`
- `CHANGES.md`
- `MEMORY.md`
- `DEVELOPMENT_NOTES.md`
- `CODEBASE_ANALYSIS.md`

## 2026-04-21-0105 — Cap exact finite-set proofs to small-support cones

**Landed as:** `b41b367`

**What changed**

`src/gen/cone.rs` now makes the generator-side exact finite-set proof
path explicitly about **small width and small endpoint support**, not
just small width plus a work budget.

The always-on exact proof helpers already had a shared work budget and
cached both exact and unknown results. That was enough to stop the
first `cse` hotspot, but it still left ANVIL repeatedly invoking the
same bounded proof on larger shared cones in the fresh
`operand-unique` frontier.

This slice adds a second guardrail:

- exact finite-set reasoning now only runs on cones up to 8 bits wide
  whose canonical leaf-endpoint support is at most **3** variables;
- comparison folding now uses the same support discipline on the
  combined endpoint set of both operands; and
- a new regression test,
  `small_value_set_skips_wide_support_cones`, pins that contract down.

The effect is simple: ANVIL still keeps the crisp narrow exact proofs it
needs for cleanliness, but it stops spending generator time proving
finite-set facts on larger shared structures that are outside the
intended "small-support" proof domain.

**Why**

The real fresh-current-code `r12` frontier had moved cleanly through
`relaxed`, `nodeid-none`, and `nodeid-cse`, then stalled again at the
start of `nodeid-operand-unique`.

Sampling showed the same broad family as before:

- generator CPU, not tool CPU;
- `build_comb_mux_encoded` in the construction path; and
- repeated time in `obvious_unsigned_compare_result ->
  node_small_value_set`.

The budget was preventing a single runaway proof, but it was not
preventing ANVIL from repeatedly entering exact finite-set reasoning on
cones whose endpoint support was already too large to be a good fit for
that proof technique.

The durable refinement is therefore:

> exact finite-set reasoning is for small-width, small-support cones.

Everything outside that stays on the cheaper proof layers (bounds,
local exact-value shortcuts, and downstream cleanup), which is exactly
where it belongs.

**Validation**

- `cargo test small_value_set --lib`
- `cargo test prove_node_exact_value --lib`
- Focused old-boundary repro:
  - `cargo run --bin anvil -- --seed 3 --count 21 --out /tmp/anvil-operand-unique-seed3-repro-r1 --construction-strategy interleaved --identity-mode node-id --factorization-level operand-unique`
  - now emits all 21 modules cleanly, including `mod_3_0020.sv`
- Downstream tool proof on that 21-module batch:
  - Verilator `--lint-only`: `21/21` pass, `0` warnings
  - Yosys `synth -noabc; stat`: `21/21` pass, `0` warnings
  - Yosys `synth -noabc; abc -fast; opt -fast; stat; check`: `21/21`
    pass, `0` warnings

**Impact**

- The fresh current-code `operand-unique` frontier no longer gets hung
  up re-entering exact finite-set proofs on larger shared cones.
- Generator-side comparison cleanliness remains intact, but the proof
  engine now has a clearer operational boundary.
- The old `/tmp/anvil-tool-matrix-phase1-real-r12` tree remains useful
  evidence, but it is now byte-stale across this further proof-semantics
  change; the next real frontier push should start from a fresh output
  tree.

**Files touched**

- `src/gen/cone.rs`
- `CHANGES.md`
- `MEMORY.md`
- `DEVELOPMENT_NOTES.md`
- `CODEBASE_ANALYSIS.md`

## 2026-04-21-0104 — Record a fresh current-code both-mode frontier through CSE

**Landed as:** `94bdf24`

**What changed**

This is an evidence slice only. No source files changed.

After fixing the exact small-set proof budget in `src/gen/cone.rs`, I
started a fresh real `tool_matrix` both-mode Phase 1 run instead of
trying to keep leaning on the older pre-fix frontiers:

- `cargo run --bin tool_matrix -- --out /tmp/anvil-tool-matrix-phase1-real-r12 --phase1-gate --yosys-mode both`

The run was deliberately stopped at the first meaningful fresh
current-code boundary: full relaxed coverage, full `nodeid-none`
coverage, full `nodeid-cse` coverage, and the beginning of
`nodeid-operand-unique`.

At the saved checkpoint, `/tmp/anvil-tool-matrix-phase1-real-r12`
contains **221 completed module checkpoints** and **221 emitted
modules**:

- `int_relaxed_none_default`: 67/67
- `int_nodeid_none_default`: 67/67
- `int_nodeid_cse_default`: 67/67
- `int_nodeid_operand-unique_default`: 20/67

And the whole saved tree remains clean:

- **0** `*.verilator.stderr.log` artifacts
- **0** Yosys `Warning:` lines across the saved
  `*.yosys-without-abc.stdout.log` and `*.yosys-with-abc.stdout.log`
  files

**Why**

The previous slice proved the proof-engine fix on a focused
`seed=2 / interleaved / node-id / cse / count=10` repro. That was a
good local proof, but it was still a local proof.

This slice answers the stronger question: does the fix actually hold up
inside the real repo-owned both-mode industrial lane?

The answer is now "yes, at least through the entire fresh current-code
`cse` rung". That matters because `cse` was exactly where the old
correlation-heavy proof hotspot had been dragging the run.

This slice also updates the recovery story. The immediate next move is
no longer "start a fresh current-code tree"; that work has now been
done. The next PNT can resume `r12` directly and continue the same lane
into `operand-unique`, `commutative`, and beyond.

**Validation**

- `cargo check --all-targets`
- `cargo test`
- `cargo clippy --all-targets -- -D warnings`
- `cargo fmt --all --check`
- Fresh current-code both-mode frontier:
  - `cargo run --bin tool_matrix -- --out /tmp/anvil-tool-matrix-phase1-real-r12 --phase1-gate --yosys-mode both`
  - manually stopped after the checkpoint
  - `find /tmp/anvil-tool-matrix-phase1-real-r12 -name '*.module-report.json' | wc -l` -> `221`
  - `find /tmp/anvil-tool-matrix-phase1-real-r12 -name 'mod_*.sv' | wc -l` -> `221`
  - `find /tmp/anvil-tool-matrix-phase1-real-r12 -name '*.verilator.stderr.log' | wc -l` -> `0`
  - `rg -n "Warning:" /tmp/anvil-tool-matrix-phase1-real-r12/*/*.yosys-*.stdout.log | wc -l` -> `0`

**Impact**

- There is now a fresh current-code real both-mode frontier parked in
  `/tmp/anvil-tool-matrix-phase1-real-r12`, not just older evidence
  trees from before the exact-proof budget fix.
- The fresh current-code lane has already cleared the full `cse` rung
  cleanly and is into `operand-unique`.
- The next PNT can resume `r12` cheaply from **221** saved checkpoints
  instead of replaying the relaxed/none/nodeid-none/cse prefix.

**Files touched**

- `CHANGES.md`
- `MEMORY.md`
- `CODEBASE_ANALYSIS.md`

## 2026-04-21-0103 — Bound exact small-set proofs to unblock the CSE frontier

**Landed as:** `248d5f2`

**What changed**

`src/gen/cone.rs` now treats exact finite-set reasoning as a bounded
generator-side proof engine rather than an unbounded search.

The small-width helper behind `prove_node_exact_value()` and
generator-side comparison folding now has two new guardrails:

- a shared work budget for exact finite-set exploration; and
- memoization of both **known** results and **unknown** results.

That means the proof engine still keeps all the useful narrow exact
facts on simple cones, but it can now bail out cleanly instead of
blowing up on correlation-heavy shared cones such as the one-hot-mux /
small-set search knot that was stalling the real `int_nodeid_cse`
frontier.

I also added a regression test that makes the new contract explicit:

- `small_value_set_bails_out_before_cartesian_blow_up`

which proves the helper now returns `None` rather than trying to fully
enumerate a clearly pathological 5-input 8-bit cartesian search.

**Why**

The real clue came from sampling the stuck `tool_matrix --resume` job
on the legacy `r11` both-mode frontier. It was not stuck in Verilator
or Yosys; it was burning CPU inside ANVIL itself, specifically in
`build_comb_mux_one_hot()` and `node_small_value_set()` while working
through an uncheckpointed `int_nodeid_cse_default` module.

This is exactly the kind of failure mode the generator-side proof
engine is supposed to avoid:

- exact narrow proofs are good because they keep emitted RTL cleaner;
- but exact proof machinery is not allowed to become the new source of
  pathological runtime.

So the right fix was not "disable the proofs" and not "let it spin".
The right fix was to make the proof engine explicitly budgeted.

One operational consequence also became explicit during validation:
`tool_matrix --resume` is intentionally byte-stable. Once generator
semantics change, older trees may fail resume validation because the
regenerated `.sv` no longer matches the saved artifact. That happened
here with the old `r11` tree, so the current-code proof was done on a
fresh reproduction tree instead of trying to cross that semantic
boundary in place.

**Validation**

- `cargo test --bin tool_matrix --lib --tests --quiet`
- `cargo test small_value_set_bails_out_before_cartesian_blow_up --lib --quiet`
- Focused current-code CSE reproduction:
  - `cargo run --bin anvil -- --seed 2 --count 10 --out /tmp/anvil-cse-seed2-repro-r1 --construction-strategy interleaved --identity-mode node-id --factorization-level cse`
  - all 10 modules emitted successfully (`mod_2_0000.sv` .. `mod_2_0009.sv`)
- Focused downstream cleanliness on that repro tree:
  - Verilator: `fails=0`, `warns=0`
  - Yosys `synth -noabc; check`: `fails=0`, `warns=0`
  - Yosys `synth -noabc; abc -fast; opt -fast; stat; check`: `fails=0`, `warns=0`

**Impact**

- The small-width exact-proof engine is now safe to leave on in the
  generator path even on correlation-heavy shared cones.
- The old "resume `r11` cheaply from 143 upgraded checkpoints" note is
  no longer the immediate cheap move after this semantic change; the
  next real frontier push should start from a fresh output tree under
  the new generator semantics.
- The former CSE choke point now has a durable regression test and a
  current-code proof run, not just a local hunch.

**Files touched**

- `src/gen/cone.rs`
- `CHANGES.md`
- `MEMORY.md`
- `README.md`
- `USER_GUIDE.md`
- `DEVELOPMENT_NOTES.md`
- `CODEBASE_ANALYSIS.md`

## 2026-04-21-0102 — Upgrade the legacy r11 both-mode frontier into resumable state

**Landed as:** `878eb4f`

**What changed**

This is an evidence slice only. No source files changed.

After landing `tool_matrix --resume`, I immediately used it on the real
legacy both-mode frontier that had been captured before checkpoint
sidecars existed:

- `cargo run --bin tool_matrix -- --out /tmp/anvil-tool-matrix-phase1-real-r11 --phase1-gate --yosys-mode both --resume`

The run was deliberately stopped at the first meaningful in-place
upgrade checkpoint rather than being left to churn indefinitely.

At the saved checkpoint, the legacy `r11` tree now contains
**143** per-module sidecars (`*.module-report.json`) covering:

- `int_relaxed_none_default`: 67/67 modules checkpointed
- `int_nodeid_none_default`: 67/67 modules checkpointed
- `int_nodeid_cse_default`: 9/67 modules checkpointed

The `.sv` frontier itself is unchanged at **368 generated modules**:

- `int_relaxed_none_default`: 67 modules
- `int_nodeid_none_default`: 67 modules
- `int_nodeid_cse_default`: 67 modules
- `int_nodeid_operand-unique_default`: 67 modules
- `int_nodeid_commutative_default`: 67 modules
- `int_nodeid_associative_default`: 33 modules

And the upgraded tree remained clean while doing that work:

- **0** `*.verilator.stderr.log` artifacts
- **0** Yosys `Warning:` lines across the saved
  `*.yosys-without-abc.stdout.log` and `*.yosys-with-abc.stdout.log`
  files

This is the first real proof that the new resume path is useful on a
large industrial frontier rather than only in tiny smoke examples.

**Why**

Adding `--resume` was only half the job. The repo already had a large
both-mode frontier parked in `/tmp/anvil-tool-matrix-phase1-real-r11`,
but that tree predated per-module checkpoints and therefore still
behaved like old dead evidence unless we actually upgraded it.

Stopping after two full scenarios plus the beginning of the third is a
good boundary because it proves three things at once:

- legacy `.sv` bootstrap works on a real frontier;
- the upgraded tree stays warning-clean while being converted; and
- future sessions can now resume `r11` cheaply from a real in-place
  checkpoint instead of replaying from zero.

**Validation**

- `cargo check --all-targets`
- `cargo test`
- `cargo clippy --all-targets -- -D warnings`
- `cargo fmt --all --check`
- `mdbook build book`
- `cargo run --bin tool_matrix -- --out /tmp/anvil-tool-matrix-phase1-real-r11 --phase1-gate --yosys-mode both --resume`
  - manually stopped after the checkpoint
  - `find /tmp/anvil-tool-matrix-phase1-real-r11 -name '*.module-report.json' | wc -l` -> `143`
  - `find /tmp/anvil-tool-matrix-phase1-real-r11 -name '*.sv' | wc -l` -> `368`
  - `find /tmp/anvil-tool-matrix-phase1-real-r11 -name '*.verilator.stderr.log' | wc -l` -> `0`
  - `rg -n "Warning:" /tmp/anvil-tool-matrix-phase1-real-r11/*/*.yosys-*.stdout.log | wc -l` -> `0`

**Impact**

- The legacy `r11` frontier is no longer just a docs checkpoint; it is
  now partially upgraded into resumable state.
- The next PNT can resume `/tmp/anvil-tool-matrix-phase1-real-r11`
  directly and cheaply from 143 verified module checkpoints.
- The stronger both-mode frontier remains at 368 generated modules, but
  the operational recovery state for that tree is now much better than
  before this slice.

**Files touched**

- `CHANGES.md`
- `MEMORY.md`
- `CODEBASE_ANALYSIS.md`

## 2026-04-21-0101 — Add resumable per-module checkpoints to tool_matrix

**Landed as:** `0c9b3f0`

**What changed**

`src/bin/tool_matrix.rs` now has real resume support instead of the old
"start a fresh output tree every time" workflow.

The harness now writes a per-module checkpoint sidecar:

- `<stem>.module-report.json`

after each fully processed module. Each sidecar stores the module
report plus the tool-surface settings that produced it
(`skip_verilator`, `skip_yosys`, and `yosys_mode`).

A new CLI flag is now live:

- `tool_matrix --resume`

When `--resume` is used, `tool_matrix` replays the deterministic
generator state, validates that the regenerated module still matches
the saved artifact, and then:

- reuses the saved tool results when the checkpoint matches the current
  tool surface; or
- bootstraps legacy / pre-checkpoint trees by validating the saved
  `.sv`, rerunning the current tool surface once, and writing the new
  sidecar.

One subtle design choice also landed: resume validation now keys on the
saved `.sv` text plus module identity, while metrics are refreshed from
the current code on resume. During the real smoke proof, metrics turned
out to be too strict a reuse key even when the emitted `.sv` matched
exactly, so the checkpoint logic now treats the emitted artifact as the
load-bearing truth and recomputes metrics locally.

`src/metrics.rs` now derives `Deserialize` and `PartialEq` to support
checkpoint I/O and the new harness tests.

The live docs now reflect the new truth: `tool_matrix` **does** resume
partial runs, older trees can be upgraded in place, and the stale
"no resume mode" guidance has been removed.

**Why**

The project had crossed the threshold where frontier runs were worth
resuming rather than replaying manually. The old workflow was wasting
time and leaving crash-recovery weaker than the user explicitly wanted.

Per-module checkpoints are the right grain here:

- scenario manifests only land at scenario boundaries, which is too
  coarse for interrupted industrial sweeps;
- top-level reports only land at the very end of the matrix; and
- partial output trees were otherwise just piles of `.sv` files with no
  trustworthy way to know what tool work had actually completed.

This slice turns `tool_matrix` into a recovery-friendly harness rather
than a one-shot batch runner.

**Validation**

- `cargo test --bin tool_matrix`
  - includes two new harness tests:
    - resume from checkpointed modules
    - bootstrap resume from legacy `.sv` files without checkpoints
- real smoke proof:
  - `cargo run --bin tool_matrix -- --out /tmp/anvil-tool-matrix-resume-smoke-r1 --modules-per-scenario 1 --yosys-mode both`
  - deliberately interrupted after 14 scenario module sidecars had landed
  - `cargo run --bin tool_matrix -- --out /tmp/anvil-tool-matrix-resume-smoke-r1 --modules-per-scenario 1 --yosys-mode both --resume`
  - final result:
    - `tool_matrix: 15 scenarios, 1 modules/scenario`
    - `Verilator pass/fail = 15/0`
    - `Yosys without-abc pass/fail = 15/0`
    - `Yosys with-abc pass/fail = 15/0`

**Impact**

- `tool_matrix` can now resume interrupted runs from the same `--out`
  tree instead of forcing fresh output roots every time.
- Future frontier pushes can be resumed exactly from new sidecars.
- Existing frontier trees can be upgraded in place by the legacy
  bootstrap path.
- The repo's recovery story now matches the project's crash-tolerant
  doctrine much better.

**Files touched**

- `src/bin/tool_matrix.rs`
- `src/metrics.rs`
- `CHANGES.md`
- `MEMORY.md`
- `DEVELOPMENT_NOTES.md`
- `README.md`
- `USER_GUIDE.md`
- `CODEBASE_ANALYSIS.md`

## 2026-04-21-0100 — Advance the real both-mode frontier to 368 clean modules

**Landed as:** `cd25e8e`

**What changed**

This is an evidence slice only. No source files changed.

I launched a fresh repo-owned both-mode gate from a new output tree:

- `cargo run --bin tool_matrix -- --out /tmp/anvil-tool-matrix-phase1-real-r11 --phase1-gate --yosys-mode both`

and deliberately stopped it only after the stricter both-mode frontier
had moved past both the previous 288-module both-mode checkpoint and
the older 365-module no-ABC baseline frontier.

The saved frontier is now **368 generated modules** with:

- **0** `*.verilator.stderr.log` artifacts
- **0** Yosys `Warning:` lines across the saved
  `*.yosys-without-abc.stdout.log` and `*.yosys-with-abc.stdout.log`
  files

Per-scenario progress at the checkpoint:

- `int_relaxed_none_default`: 67 modules clean
- `int_nodeid_none_default`: 67 modules clean
- `int_nodeid_cse_default`: 67 modules clean
- `int_nodeid_operand-unique_default`: 67 modules clean
- `int_nodeid_commutative_default`: 67 modules clean
- `int_nodeid_associative_default`: 33 modules clean

The docs now also say explicitly that `tool_matrix` still has **no
resume mode**: continuing these industrial sweeps currently means
starting a fresh output tree and pushing it to a stronger checkpoint,
not reusing an existing `--out` directory in place.

**Why**

The prior both-mode checkpoint at 288 modules was already useful, but
the next worthwhile move was to see whether the stricter Yosys surface
could stay warning-clean all the way through the full commutative rung
and into associative territory.

This checkpoint matters because it turns the both-mode lane from "real
but still smaller than the baseline gate" into the strongest repo-owned
Phase 1 frontier we have so far. The more demanding Yosys shape now
stands slightly ahead of the original no-ABC baseline instead of behind
it.

**Validation**

- `cargo check --all-targets`
- `cargo test`
- `cargo clippy --all-targets -- -D warnings`
- `cargo fmt --all --check`
- `mdbook build book`
- `cargo run --bin tool_matrix -- --out /tmp/anvil-tool-matrix-phase1-real-r11 --phase1-gate --yosys-mode both`
  - manually stopped after the checkpoint
  - `find /tmp/anvil-tool-matrix-phase1-real-r11 -name '*.sv' | wc -l` -> `368`
  - `find /tmp/anvil-tool-matrix-phase1-real-r11 -name '*.verilator.stderr.log' | wc -l` -> `0`
  - `rg -n "Warning:" /tmp/anvil-tool-matrix-phase1-real-r11/*/*.yosys-*.stdout.log | wc -l` -> `0`

**Impact**

- The real both-mode Phase 1 frontier advanced from 288 clean modules
  to 368 clean modules.
- The repo now has durable both-mode evidence through the full
  `commutative` rung and 33 clean modules into `associative`.
- The stronger both-mode gate has now edged past the older 365-module
  no-ABC baseline frontier.
- Until `tool_matrix` grows explicit resume support, the next PNT must
  fork a fresh output tree for any further frontier push.

**Files touched**

- `CHANGES.md`
- `MEMORY.md`
- `DEVELOPMENT_NOTES.md`
- `README.md`
- `USER_GUIDE.md`
- `ROADMAP.md`
- `CODEBASE_ANALYSIS.md`

## 2026-04-21-0099 — Advance the real both-mode frontier to 288 clean modules

**Landed as:** `148ee8d`

**What changed**

This is an evidence slice only. No source files changed.

I resumed the real repo-owned both-mode gate from a fresh output tree:

- `cargo run --bin tool_matrix -- --out /tmp/anvil-tool-matrix-phase1-real-r10 --phase1-gate --yosys-mode both`

and deliberately stopped it only after it had materially advanced past
the previous both-mode checkpoint.

The saved frontier is now **288 generated modules** with:

- **0** `*.verilator.stderr.log` artifacts
- **0** Yosys `Warning:` lines across the saved
  `*.yosys-without-abc.stdout.log` and `*.yosys-with-abc.stdout.log`
  files

Per-scenario progress at the checkpoint:

- `int_relaxed_none_default`: 67 modules clean
- `int_nodeid_none_default`: 67 modules clean
- `int_nodeid_cse_default`: 67 modules clean
- `int_nodeid_operand-unique_default`: 67 modules clean
- `int_nodeid_commutative_default`: 20 modules clean

Compared to the previous both-mode frontier (144 modules), this extends
the real zero-warning evidence across two additional full node-id
factorization rungs and materially into the commutative rung, while
keeping all three downstream surfaces clean:

- Verilator lint
- Yosys `synth -noabc`
- Yosys `synth -noabc; abc -fast; opt -fast; stat; check`

The live docs were updated accordingly. No roadmap phase labels changed.

**Why**

The prior both-mode checkpoint proved that the cleaned-up ABC harness
lane was not just a tiny 15-module smoke-case success. The next useful
step was to keep pushing that stricter gate shape until it had covered
more of the factorization ladder in real generated artifacts.

This new checkpoint matters because it now shows the both-mode tool
surface staying clean through:

- relaxed identity
- node-id / none
- node-id / cse
- node-id / operand-unique
- and into node-id / commutative

That is strong enough to be a real industrial checkpoint rather than a
mere "it still works on the first couple of lanes" reassurance.

**Validation**

- `cargo check --all-targets`
- `cargo test`
- `cargo clippy --all-targets -- -D warnings`
- `cargo fmt --all --check`
- `mdbook build book`
- `cargo run --bin tool_matrix -- --out /tmp/anvil-tool-matrix-phase1-real-r10 --phase1-gate --yosys-mode both`
  - manually stopped after the checkpoint
  - `find /tmp/anvil-tool-matrix-phase1-real-r10 -name '*.sv' | wc -l` -> `288`
  - `find /tmp/anvil-tool-matrix-phase1-real-r10 -name '*.verilator.stderr.log' | wc -l` -> `0`
  - `rg -n "Warning:" /tmp/anvil-tool-matrix-phase1-real-r10/*/*.yosys-*.stdout.log | wc -l` -> `0`

**Impact**

- The real both-mode Phase 1 frontier advanced from 144 clean modules
  to 288 clean modules.
- The repo now has durable both-mode evidence through the full
  `operand-unique` rung and into the `commutative` rung.
- Until `tool_matrix` grows explicit resume support, the next PNT must
  fork a fresh output tree rather than reusing the same `--out`
  directory in place.

**Files touched**

- `CHANGES.md`
- `MEMORY.md`
- `README.md`
- `USER_GUIDE.md`
- `ROADMAP.md`
- `CODEBASE_ANALYSIS.md`

## 2026-04-21-0098 — Record the first clean both-mode Phase 1 frontier

**Landed as:** `e532cc9`

**What changed**

This is an evidence slice only. No source files changed.

I resumed the real repo-owned Phase 1 gate from a fresh output tree,
but this time under the stronger Yosys surface:

- `cargo run --bin tool_matrix -- --out /tmp/anvil-tool-matrix-phase1-real-r9 --phase1-gate --yosys-mode both`

The run was deliberately stopped at the first meaningful both-mode
checkpoint after the harness had cleared multiple full scenarios.

The saved frontier is now **144 generated modules** with:

- **0** `*.verilator.stderr.log` artifacts
- **0** Yosys `Warning:` lines across the saved
  `*.yosys-without-abc.stdout.log` and `*.yosys-with-abc.stdout.log`
  files

Per-scenario progress at the checkpoint:

- `int_relaxed_none_default`: 67 modules clean
- `int_nodeid_none_default`: 67 modules clean
- `int_nodeid_cse_default`: 10 modules clean

This is the first real repo-owned frontier that keeps **both** Yosys
lanes clean, not just the `without-abc` baseline:

- Verilator lint
- Yosys `synth -noabc`
- Yosys `synth -noabc; abc -fast; opt -fast; stat; check`

The live docs were updated to record this stronger-but-smaller frontier
without overwriting the existing 365-module no-ABC frontier. Both
facts are now explicit in the repo:

- 365 clean modules under the original `tool_matrix --phase1-gate`
  baseline
- 144 clean modules under `tool_matrix --phase1-gate --yosys-mode both`

No roadmap phase labels changed.

**Why**

The previous slice made the ABC-enabled Yosys lane warning-clean in a
small 15-scenario smoke probe. The next responsible move was to test
that lane under the real industrial gate shape instead of assuming the
small probe would generalize.

Stopping at the two-scenario boundary plus the beginning of the third
gives a checkpoint that is both meaningful and recoverable:

- one full relaxed scenario under both Yosys modes
- one full node-id scenario under both Yosys modes
- the start of the next factorization rung already clean

That is enough evidence to say the cleaned-up ABC lane is not just a
toy-case success, while still keeping the slice scoped and commit-ready.

**Validation**

- `cargo check --all-targets`
- `cargo test`
- `cargo clippy --all-targets -- -D warnings`
- `cargo fmt --all --check`
- `mdbook build book`
- `cargo run --bin tool_matrix -- --out /tmp/anvil-tool-matrix-phase1-real-r9 --phase1-gate --yosys-mode both`
  - manually stopped after the checkpoint
  - `find /tmp/anvil-tool-matrix-phase1-real-r9 -name '*.sv' | wc -l` -> `144`
  - `find /tmp/anvil-tool-matrix-phase1-real-r9 -name '*.verilator.stderr.log' | wc -l` -> `0`
  - `rg -n "Warning:" /tmp/anvil-tool-matrix-phase1-real-r9/*/*.yosys-*.stdout.log | wc -l` -> `0`

**Impact**

- The repo now has a real both-mode Phase 1 frontier, not just a tiny
  both-mode smoke probe.
- The cleaned-up ABC harness lane is now proven through two full
  scenarios and into the next factorization rung.
- Future PNT work can start from a fresh output tree and target the
  next unseen boundary instead of settling for the 15-module smoke toy.

**Files touched**

- `CHANGES.md`
- `MEMORY.md`
- `README.md`
- `USER_GUIDE.md`
- `ROADMAP.md`
- `CODEBASE_ANALYSIS.md`

## 2026-04-21-0097 — Make the ABC-enabled Yosys harness warning-clean

**Landed as:** `bbfca1d`

**What changed**

This slice finishes the Yosys-mode work by turning `with-abc` into a
repo-owned, warning-clean ABC path instead of just replaying Yosys's
raw default `synth` script.

In `src/bin/tool_matrix.rs`:

- `--yosys-mode with-abc` now runs:
  `read_verilog -sv ...; synth -noabc; abc -fast; opt -fast; stat; check`
  instead of `read_verilog -sv ...; synth; stat`
- warning failures now retain the first actual warning line as the
  `error` string instead of the vague `tool emitted warning(s)` bucket
- the `yosys_mode` tests were updated to pin the new script shape

The driving evidence was the first repo-owned `--yosys-mode both`
probe: `without-abc` passed 15/15 while the old `with-abc` path failed
14/15 solely on:

- `ABC: Warning: The network is combinational (run "fraig" or "fraig_sweep").`

Direct Yosys probing showed that this warning came from the default
ABC script's `scorr` step, not from invalid RTL. Yosys's own `help abc`
text also confirms that ABC operates on extracted logic snippets rather
than necessarily on the whole module as one sequential network, which
explains why the warning appeared even on sequential modules with many
flops.

The harness now uses `abc -fast` for the repo-owned ABC lane, and the
follow-up repo-owned smoke run is clean:

- `without-abc = 15/15 pass`
- `with-abc = 15/15 pass`

The live docs and book were updated accordingly:

- `README.md`, `USER_GUIDE.md`, `ROADMAP.md`
- `CODEBASE_ANALYSIS.md`, `DEVELOPMENT_NOTES.md`
- `book/src/architecture.md`, `book/src/recipes.md`

No roadmap phase labels changed.

**Why**

The user's bar is explicit: warnings are not to be waved away, and the
project needs clean Verilator/Yosys runs by default. The previous
`with-abc` path was therefore useful as diagnosis, but not good enough
as the repo-owned ABC harness lane because it stayed red on a warning
bucket that current evidence points to as a tool-flow mismatch rather
than a generator defect.

The right move was not to hide warnings. It was to understand the exact
warning, choose an ABC-enabled script that stays meaningful for the
project, and keep the harness red only for genuinely actionable
problems.

That leaves `without-abc` as the stable baseline and `with-abc` as a
real second lane, not a knowingly noisy one.

**Validation**

- `cargo check --all-targets`
- `cargo test`
- `cargo clippy --all-targets -- -D warnings`
- `cargo fmt --all --check`
- `mdbook build book`
- `cargo run --bin tool_matrix -- --out /tmp/anvil-tool-matrix-yosys-both-smoke-r2 --modules-per-scenario 1 --skip-verilator --yosys-mode both`
  - `tool_matrix: Yosys without-abc pass/fail = 15/0`
  - `tool_matrix: Yosys with-abc pass/fail = 15/0`
- direct probes on the previous failing sequential and comb-only cases:
  - `yosys -Q -p '...; synth; stat'` reproduced the ABC warning
  - `yosys -Q -p '...; synth -noabc; abc -fast; opt -fast; stat; check'`
    stayed warning-clean and `check`-clean

**Impact**

- The repo-owned Yosys harness now has two clean lanes instead of one:
  `without-abc` and `with-abc`.
- `tool_matrix --yosys-mode both` is now a genuinely useful industrial
  smoke shape instead of a guaranteed warning bucket.
- Future warning regressions in tool runs are now more actionable
  because the report records the actual warning line.
- The docs now describe the Yosys axis accurately: `with-abc` is the
  explicit ABC-enabled harness path, not the raw default `synth`
  behavior.

**Files touched**

- `CHANGES.md`
- `MEMORY.md`
- `README.md`
- `USER_GUIDE.md`
- `ROADMAP.md`
- `CODEBASE_ANALYSIS.md`
- `DEVELOPMENT_NOTES.md`
- `book/src/architecture.md`
- `book/src/recipes.md`
- `src/bin/tool_matrix.rs`

## 2026-04-21-0096 — Advance the real Phase 1 gate frontier to 365 clean modules

**Landed as:** `f708d8d`

**What changed**

This is primarily an evidence slice, with one small hygiene-only source
cleanup discovered during the mandatory pre-commit checks.

I resumed the real repo-owned Phase 1 gate from a fresh output tree:

- `cargo run --bin tool_matrix -- --out /tmp/anvil-tool-matrix-phase1-real-r7 --phase1-gate`

and checkpointed it only after it had materially expanded the clean
frontier again.

The run reached **365 generated modules** with:

- **0** `*.verilator.stderr.log` artifacts
- **0** Yosys `Warning:` lines across the saved `*.yosys.stdout.log`
  files

Per-scenario progress at the checkpoint:

- `int_relaxed_none_default`: 67 modules clean
- `int_nodeid_none_default`: 67 modules clean
- `int_nodeid_cse_default`: 67 modules clean
- `int_nodeid_operand-unique_default`: 67 modules clean
- `int_nodeid_commutative_default`: 67 modules clean
- `int_nodeid_associative_default`: 30 modules clean

This extends the clean evidence from the first three node-id rungs into
the full commutative rung and materially into the associative rung,
without surfacing a single new Verilator warning or Yosys warning.

While running the required hygiene quartet for the checkpoint,
`cargo clippy --all-targets -- -D warnings` also surfaced a
pre-existing `clippy::filter_map_bool_then` lint in
`src/gen/cone.rs`. That site now uses the equivalent `filter(...).map(...)`
form. The behavior is unchanged; the slice just brings the repo back to
full hygiene-green status after recording the stronger frontier.

The live docs were updated to record this improved evidence frontier.
No roadmap phase labels changed.

**Why**

The user's standing direction is still the right one: warnings count,
tool cleanliness matters, and the adversarial axes need to be exercised
without hidden bias. Once the warning bucket stopped reproducing in the
earlier lanes, the next responsible move was to keep pushing the real
matrix forward across more identity/factorization scenarios instead of
pretending the early clean runs were enough.

The commit workflow is also explicit that the hygiene quartet must be
green before a commit proceeds. Folding the tiny clippy cleanup into
this slice keeps the checkpoint honest instead of landing an evidence
commit that immediately fails the repo's own pre-commit bar.

This slice matters because the new frontier now shows the repaired
generator behavior surviving:

- the full relaxed/default scenario,
- the full node-id/none scenario,
- the full node-id/cse scenario,
- the full node-id/operand-unique scenario,
- the full node-id/commutative scenario, and
- a substantial prefix of node-id/associative.

**Validation**

- `cargo check --all-targets`
- `cargo test`
- `cargo clippy --all-targets -- -D warnings`
- `cargo fmt --all --check`
- `cargo run --bin tool_matrix -- --out /tmp/anvil-tool-matrix-phase1-real-r7 --phase1-gate`
  - checkpointed after 365 generated modules
  - `find /tmp/anvil-tool-matrix-phase1-real-r7 -name '*.sv' | wc -l` -> `365`
  - `find /tmp/anvil-tool-matrix-phase1-real-r7 -name '*.verilator.stderr.log' | wc -l` -> `0`
  - `rg -n "Warning:" /tmp/anvil-tool-matrix-phase1-real-r7/*/*.yosys.stdout.log | wc -l` -> `0`

**Impact**

- The real repo-owned Phase 1 warning-clean frontier advanced from 246
  modules to 365 modules.
- Cleanliness evidence now spans the full commutative node-id scenario
  and 30 modules into the associative scenario.
- The repo is back to a fully green commit-workflow hygiene quartet
  after the tiny clippy-only cleanup in `src/gen/cone.rs`.
- The next PNT can resume from a stronger checkpoint and either finish
  the associative lane or investigate the first new warning/failure
  beyond it, instead of rerunning already-established clean ground.

**Files touched**

- `CHANGES.md`
- `MEMORY.md`
- `CODEBASE_ANALYSIS.md`
- `ROADMAP.md`
- `src/gen/cone.rs`

## 2026-04-20-0095 — Advance the real Phase 1 gate frontier to 246 clean modules

**Landed as:** `60d9883`

**What changed**

This is an evidence slice, not a code-change slice.

After landing the wide-slice proof / dead-state / strict post-remap
duplicate fixes, I reran the real repo-owned Phase 1 gate:

- `cargo run --bin tool_matrix -- --out /tmp/anvil-tool-matrix-phase1-real-r6 --phase1-gate`

and manually stopped it only after it had materially advanced beyond the
previous frontier.

The partial run reached **246 generated modules** with:

- **0** `*.verilator.stderr.log` artifacts
- **0** Yosys `Warning:` lines across the saved `*.yosys.stdout.log`
  files

Per-scenario progress at the checkpoint:

- `int_relaxed_none_default`: 67 modules clean
- `int_nodeid_none_default`: 67 modules clean
- `int_nodeid_cse_default`: 67 modules clean
- `int_nodeid_operand-unique_default`: 45 modules clean

This means the repaired warning bucket is no longer just a first-scenario
accident: the clean run now spans relaxed + node-id lanes and has pushed
well into the next factorization rung too.

The live docs were updated to record this improved evidence frontier.
No roadmap phase labels changed.

**Why**

The user explicitly set the bar as "no warning or error from Verilator
and Yosys" and asked that warnings always warrant investigation. Once a
warning bucket is fixed, the next responsible move is not to assume the
problem is gone forever; it is to push the real adversarial matrix
farther and retain the new clean frontier as durable evidence.

That matters here because the previous frontier had only reached the
first relaxed/default scenario. This slice shows that the same repaired
generator behavior now survives:

- the full relaxed/default scenario,
- the full node-id/none scenario,
- the full node-id/cse scenario, and
- a substantial prefix of node-id/operand-unique.

**Validation**

- `cargo run --bin tool_matrix -- --out /tmp/anvil-tool-matrix-phase1-real-r6 --phase1-gate`
  - manually stopped after 246 generated modules
  - `find /tmp/anvil-tool-matrix-phase1-real-r6 -name '*.sv' | wc -l` -> `246`
  - `find /tmp/anvil-tool-matrix-phase1-real-r6 -name '*.verilator.stderr.log' | wc -l` -> `0`
  - `rg -n "Warning:" /tmp/anvil-tool-matrix-phase1-real-r6/*/*.yosys.stdout.log | wc -l` -> `0`

**Impact**

- The real repo-owned Phase 1 warning-clean frontier advanced from the
  earlier 76-module checkpoint to 246 modules.
- Cleanliness evidence now spans multiple identity/factorization modes,
  not only the first relaxed/default lane.
- The next PNT can start from a stronger base: keep pushing the real
  gate until the next actual warning/failure, rather than re-litigating
  the already-fixed early scenarios.

**Files touched**

- `CHANGES.md`
- `MEMORY.md`
- `DEVELOPMENT_NOTES.md`
- `CODEBASE_ANALYSIS.md`
- `ROADMAP.md`

## 2026-04-20-0094 — Close wide-slice overshift proof gaps and prune dead state at finalisation

**Landed as:** `739f9fe`

**What changed**

This slice closes the next real downstream-cleanliness holes surfaced by
the live warning hunt.

### Exact-value proofs now survive narrow slices of wider cones

- `src/gen/cone.rs` no longer gives up on `Slice` outputs just because
  the source cone is wider than 8 bits.
- For a narrow slice whose source set is unavailable, the small-set
  helper now:
  - uses an exact source value when one is already provable, or
  - safely falls back to the full narrow output domain instead of
    returning `None`.
- That keeps exact-value reasoning alive for real shapes like:
  - wide-input slice -> OR with forcing constants -> SHL ->
    subtract-small-constant -> dynamic SHR
  - where the result is still provably zero even though the leaf source
    was wider than the small-set engine's direct domain.
- Added regressions in `src/gen/cone.rs` and `src/ir/compact.rs` for the
  real "dynamic overshift through a wide slice" pattern.

### Final compaction now removes dead flops, not just dead nodes

- `src/ir/compact.rs::compact_node_ids` now treats output drive-roots as
  the primary liveness roots.
- Live flops are discovered when a reachable `Node::FlopQ` is actually
  consumed; reaching `Q` then pulls the owning flop's `d` and mux-held
  nodes into the walk.
- Unobserved flops are dropped from `m.flops`, `Node::FlopQ` backrefs
  are renumbered into the compacted `FlopId` space, and gate dep-sets
  remap virtual flop ids alongside node ids.
- Added a regression proving that a flop whose `Q` is never observed is
  removed during compaction.

This closes the live Verilator `UNUSEDSIGNAL` leak where dead registers
could survive purely because the old compaction BFS rooted every
`flop.q` unconditionally.

### Post-construction remaps now respect the strict Add/Mul duplicate policy

- Added `prune_duplicate_introducing_add_mul_remaps(...)` in
  `src/ir/compact.rs`.
- `fold_proven_gates` and `merge_equivalent_gates` now prune candidate
  node remaps that would create duplicate `NodeId`s inside a strict
  `Add` or `Mul` operand list when `operand_duplication_rate < 1.0`.
- This preserves the default doctrine tested by
  `zero_duplicate_operands_at_default_knobs` even after late proof /
  sharing passes collapse two equivalent child cones to one canonical
  node.

**Why**

The real warning hunt exposed three important truths about the current
implementation:

1. a narrow output slice of a wider cone is still a narrow proof domain,
   so exact-value reasoning must not bail out just because the source is
   wide;
2. "reachable sequential state" means "reachable from outputs through
   actually-consumed `Q`s", not "every row in `m.flops` is live by
   definition"; and
3. late identity/remap passes are not allowed to silently violate the
   default strict duplicate-operand contract for `Add` / `Mul`.

All three were generator bugs, not downstream-tool quirks.

**Validation**

- `cargo test`
- `cargo fmt --all --check`
- `mdbook build book`
- `cargo test --test pipeline zero_duplicate_operands_at_default_knobs`
- `cargo run --bin anvil -- --seed 0 --count 7 --out /tmp/anvil-seed0-relaxed-none-repro --construction-strategy interleaved --identity-mode relaxed --factorization-level none`
  - `verilator --lint-only -Wall -Wno-DECLFILENAME /tmp/anvil-seed0-relaxed-none-repro/mod_0_0006.sv`
    - exit 0, no warnings
  - `yosys -p "read_verilog -sv /tmp/anvil-seed0-relaxed-none-repro/mod_0_0006.sv; synth -noabc; stat"`
    - exit 0, no warnings

**Impact**

- The exact-proof layer can now prove some real downstream-cleanliness
  facts through wide-source narrow slices that were previously invisible.
- Dead sequential state no longer survives into emitted SV just because
  it existed in the flop table.
- The default strict duplicate-operand doctrine remains true even after
  late proof and semantic-merge passes.
- The live `seed=0 / interleaved / relaxed / none` repro that had been
  carrying the overshift / unused-state investigation is now clean in
  both Verilator and the repo-owned Yosys `synth -noabc` check.

**Files touched**

- `CHANGES.md`
- `MEMORY.md`
- `ROADMAP.md`
- `DEVELOPMENT_NOTES.md`
- `CODEBASE_ANALYSIS.md`
- `book/src/factorization.md`
- `src/gen/cone.rs`
- `src/ir/compact.rs`

## 2026-04-20-0093 — Extend exact proof shortcuts and restore associative normal form after remaps

**Landed as:** `cda8bd1`

**What changed**

This slice keeps pushing on the real `--phase1-gate` warning bucket. It
does two related things: it teaches the exact-proof helpers to stop
giving up once the result is already forced, and it restores
associative normal form after later remap passes have changed which
already-built node an operand points at.

### Exact proof now short-circuits once the answer is already forced

- `src/gen/cone.rs` now folds reflexive unsigned comparisons exactly:
  - `Eq(x, x)`, `Le(x, x)`, `Ge(x, x)` -> `1`
  - `Neq(x, x)`, `Lt(x, x)`, `Gt(x, x)` -> `0`
- The exact finite-set helper and the unsigned-bounds helper now track
  duplicate-XOR parity by `NodeId`, so shapes like `x ^ x` collapse to
  zero even in `identity_mode = relaxed`.
- The exact finite-set helper now short-circuits on absorbing or
  saturating exact prefixes instead of recursing into irrelevant tails:
  - `And(..., 0, tail)` -> `{0}`
  - `Or(..., all_ones, tail)` -> `{all_ones}`
  - `Mul(..., 0, tail)` -> `{0}`
- The exact-value helper used by `node_unsigned_bounds` now applies the
  same idea on the settled graph, so proofs like `2'h2 * 2'h2 * tail`
  at width 2 -> `0`, or `6'h16 | 6'h39 | tail` at width 6 -> `6'h3f`,
  survive even when the tail itself is not exactly known.

Together these changes close the real warning shapes found in the live
Phase 1 run:

- `x <= x` / `x < x`
- `x >= (x ^ x)`
- `x > all_ones`
- `x > 1` after a product already wrapped to zero upstream

### Finalisation now re-runs associative flattening after remaps

- Added `flatten_posthoc_associative_gates(&mut Module)` in
  `src/ir/compact.rs`.
- The pass runs only when the effective factorization ladder includes
  `Associative`.
- It restores same-op same-width associative normal form after
  remap-producing post-construction passes (`fold_proven_gates`,
  `merge_equivalent_gates`) have changed which already-built node a
  gate operand points at.
- `Add` / `Mul` still respect the strict duplicate policy: if
  flattening would create duplicate operands under
  `operand_duplication_rate < 1.0`, the nested shape is preserved.
- `generate_leaf_module` now calls this post-remap associative pass
  after the first proof-cleanup pass, after semantic gate merge, and
  after the final proof-cleanup pass.

This closes the regression that showed up in
`nested_associative_opportunities_flatten_to_zero`: the live
Associative layer was fine at intern time, but later remaps could
reintroduce a legal nested `Add` shape unless we normalized again on
the settled graph.

**Why**

The real `tool_matrix --phase1-gate` run is doing exactly what it
should: surfacing concrete warning artifacts instead of letting us hand
wave about downstream cleanliness. The warnings we hit were not "tool
being picky"; they were proof blind spots:

- small exact expressions that were already forced by a constant
  prefix, but the helper still recursed into an irrelevant unknown
  tail; and
- post-construction remaps reintroducing a legal associative nesting
  after the intern-time Associative layer had already done its job.

Both needed to be fixed in ANVIL, not explained away.

**Validation**

- `cargo test`
- `cargo fmt --all --check`
- `cargo test --test pipeline nested_associative_opportunities_flatten_to_zero -- --exact`
- `cargo run --bin anvil -- --seed 0 --count 31 --out /tmp/anvil-relaxed-none-probe-r4 --identity-mode relaxed --factorization-level none`
  - `verilator --lint-only` clean on:
    - `mod_0_0013.sv`
    - `mod_0_0016.sv`
    - `mod_0_0018.sv`
    - `mod_0_0026.sv`
    - `mod_0_0030.sv`
- `cargo run --bin tool_matrix -- --out /tmp/anvil-tool-matrix-phase1-real-r4 --phase1-gate`
  - manually stopped after 76 generated modules
  - Verilator warning logs present: `0`

**Impact**

- The exact-proof layer is less correlation-blind in relaxed lanes.
- Small-width proofs no longer give up just because an irrelevant tail
  depends on a wider cone.
- Post-construction remaps no longer leak legal nested associative
  shapes into final metrics / emission.
- The real Phase 1 gate progressed materially farther before any
  warning investigation was needed.

**Files touched**

- `CHANGES.md`
- `MEMORY.md`
- `DEVELOPMENT_NOTES.md`
- `CODEBASE_ANALYSIS.md`
- `book/src/architecture.md`
- `book/src/factorization.md`
- `src/gen/cone.rs`
- `src/gen/module.rs`
- `src/ir/compact.rs`

## 2026-04-20-0092 — Make the Phase 1 gate first-class in tool_matrix

**Landed as:** `fe4dd0e`

**What changed**

This slice turns the "broaden the clean downstream sweep toward the
1000-module Phase 1 gate" step into a first-class repo-owned harness
mode instead of leaving it as manual arithmetic over the smoke matrix.

### `tool_matrix` now has an explicit Phase 1 gate mode

- Added `--phase1-gate` to `src/bin/tool_matrix.rs`.
- The flag:
  - auto-enables coverage-gap failure; and
  - raises `modules_per_scenario` high enough to generate at least
    1000 modules total across the built-in scenario set.
- The aggregated JSON report now records:
  - `total_modules`, and
  - whether the run was a `phase1_gate` run.
- Console output now prints the total module count so the scale of a
  run is explicit at a glance.

Today, with the built-in 15-scenario matrix, `--phase1-gate` lifts the
run to 67 modules/scenario, i.e. 1005 total modules.

### The run-plan math is now tested

- Added two inline unit tests in `src/bin/tool_matrix.rs`:
  - Phase 1 gate raises the default 1-module/scenario smoke shape to
    67 modules/scenario for 15 scenarios.
  - A larger explicit `--modules-per-scenario` still wins when the user
    intentionally asks for an even bigger run.

### Durable docs now point at the real command

- `README.md`, `USER_GUIDE.md`, `ROADMAP.md`, `CODEBASE_ANALYSIS.md`,
  `DEVELOPMENT_NOTES.md`, `MEMORY.md`, and `book/src/recipes.md` now
  all describe `tool_matrix --phase1-gate` as the repo-owned Phase 1
  gate shape instead of leaving that invocation implicit.

**Why**

Now that the smoke matrix is green, the next pressure point is scale.
The Phase 1 gate matters too much to live only as prose plus
"remember to do the math". If a quality gate is load-bearing, ANVIL
should be able to invoke it directly and deterministically.

**Validation**

- `cargo check --all-targets`
- `cargo test`
- `cargo clippy --all-targets -- -D warnings`
- `cargo fmt --all --check`
- `mdbook build book`
- `cargo run --bin tool_matrix -- --out /tmp/anvil-tool-matrix-smoke-post-phase1-flag --modules-per-scenario 1`
  - Verilator: 15 pass / 0 fail
  - Yosys: 15 pass / 0 fail

**Impact**

- The repo-owned clean smoke matrix remains easy to run.
- The broader Phase 1 gate is now an explicit harness mode instead of a
  remembered convention.
- This makes the next scale-up slice more mechanical: run the real gate,
  investigate any failures, retain fixtures if needed, repeat.

**Files touched**

- `CHANGES.md`
- `MEMORY.md`
- `DEVELOPMENT_NOTES.md`
- `CODEBASE_ANALYSIS.md`
- `README.md`
- `ROADMAP.md`
- `USER_GUIDE.md`
- `book/src/recipes.md`
- `src/bin/tool_matrix.rs`

## 2026-04-20-0091 — Close the downstream-warning bucket

**Landed as:** `07536df`

**What changed**

This slice closes the remaining repo-owned `tool_matrix`
warning-cleanliness gap and makes the warning policy itself explicit in
code.

### The final graph now gets a proof-cleanup pass

- Added `fold_proven_gates(&mut Module)` in `src/ir/compact.rs`.
- `generate_leaf_module` now runs that pass once after flop-mux
  metadata is summarized and again after the post-construction sharing
  passes settle the graph.
- The pass does two things on the settled graph:
  - rewrites any gate whose current cone is provably exact into a
    constant in place; and
  - rewires muxes whose selector is now provably constant.

This matters because some exact proofs are only visible after later
remaps / merges have changed what a gate actually sees.

### Exact-value proof got two important reinforcements

- Added `prove_node_exact_value` in `src/gen/cone.rs` so the
  post-construction pass can reuse the generator's exact-value proof
  rather than inventing a second local notion of "obvious constant".
- Strengthened shift bounds: `Shl` / `Shr` now fold to zero not only
  when the shift amount is an exact constant that overshifts, but also
  when the shift amount is merely proven to be **always** at least the
  source width.
- Added a bounded semantic-exact fallback in `fold_proven_gates` for
  small-support cones whose exact value is correlation-sensitive and is
  not recovered by interval / small-set local proof alone.

Together these close the three residual Verilator `UNSIGNED` cases that
had survived the earlier comparison-construction proof.

### `tool_matrix` now treats warnings as failures

- `src/bin/tool_matrix.rs` now scans tool output for warning markers and
  marks the invocation failed even when the process exit code is zero.
- The Yosys command used by the harness changed from `synth` to
  `synth -noabc` so the repo-owned matrix no longer tolerates an
  irrelevant ABC combinational-network warning and then pretends the run
  was clean.

This turns the user rule into executable policy: for repo-owned
Verilator/Yosys evidence, green means **no errors and no warnings**.

### New regression tests pin the late-proof path

- Added two `ir::compact` unit tests covering:
  - a constant-selector / masked-constant chain in the shape of the
    former `mod_6_0000.sv` warning; and
  - an overshift-to-zero compare chain in the shape of the former
    `mod_12_0000.sv` warning.

**Why**

The previous slice had narrowed the warning bucket enough that stopping
there would have been easy, but not acceptable under the project's
actual quality bar. The remaining warnings were already telling us
something precise: ANVIL still had exact facts that it could prove, but
was not proving late enough.

This slice fixes that architectural gap instead of teaching the project
to live with "known clean-ish" output.

**Validation**

- `cargo check --all-targets`
- `cargo test`
  - 140 passing tests: 116 unit + 24 integration
- `cargo clippy --all-targets -- -D warnings`
- `cargo fmt --all --check`
- `cargo run --bin tool_matrix -- --out /tmp/anvil-tool-matrix-current3 --modules-per-scenario 1`
  - Verilator: 15 pass / 0 fail
  - Yosys: 15 pass / 0 fail
  - warnings are now treated as failures

**Impact**

- The repo-owned downstream smoke matrix is now clean in both tools:
  15/15 Verilator-clean and 15/15 Yosys-clean.
- The warning policy is now executable rather than aspirational:
  `tool_matrix` rejects warning-bearing runs.
- The generator has a stronger architectural cleanliness story:
  construction-time proof, post-construction proof-cleanup, semantic
  gate merge, sequential merge, then compaction.

**Files touched**

- `CHANGES.md`
- `MEMORY.md`
- `DEVELOPMENT_NOTES.md`
- `CODEBASE_ANALYSIS.md`
- `ROADMAP.md`
- `README.md`
- `USER_GUIDE.md`
- `book/src/architecture.md`
- `book/src/faq.md`
- `src/bin/tool_matrix.rs`
- `src/gen/cone.rs`
- `src/gen/module.rs`
- `src/ir/compact.rs`

## 2026-04-20-0090 — Strengthen generator-side proof for constant comparisons

**Landed as:** `1ed22db`

**What changed**

This slice attacks the Verilator `CMPCONST` / `UNSIGNED`
warning-cleanliness bucket exposed by `tool_matrix`, but does it from
the generator side rather than by adding suppressions or by tying the
fix to one factorization rung.

### Comparison construction now has an always-on proof path

- Added a dedicated comparison-construction helper in
  `src/gen/cone.rs` that tries to prove a comparison is constant
  **before** interning the gate.
- Wired that helper through every comparison-emission path:
  - recursive `build_cone`,
  - interleaved `deliver`,
  - pool-only `grow_pool_one_unit`,
  - constant-comparand motif construction, and
  - equality-to-constant helpers used by encoded mux assembly.
- The proof is intentionally independent of
  `identity_mode` / `factorization_level`: this is treated as
  by-construction tool-cleanliness, not as optional semantic
  factorization.

### The proof got two layers of reasoning

- A conservative unsigned-bounds engine now recognizes easy local
  identities and range facts that matter for comparisons:
  - `x & 0 = 0`,
  - `x | all_ones = all_ones`,
  - `x * 0 = 0`,
  - `x * 1 = x`,
  - `x + 0 = x`,
  - `x - 0 = x`,
  - `~x` bound inversion,
  - exact / bounded `Shl` and `Shr`,
  - mux-arm union or exact branch selection when the select is known,
  - exact / bounded concat assembly for small-width shapes.
- For comparison operands up to 8 bits wide, ANVIL now also runs an
  exact finite-set proof instead of relying only on intervals. This is
  especially important because comparisons are already generated over
  1..8-bit operand widths.
- The finite-set path now preserves **replication correlation** for
  `Concat` shapes like `{N{bit}}`; repeated copies of the same node are
  no longer treated as independent free variables during the proof.

### New regression tests pin the warning-cleanliness logic

- Added unit tests for:
  - `x > all_ones` folding even below the peephole rung,
  - overshift-to-zero compare folding,
  - non-overlapping-range preservation of live comparisons, and
  - replicated-concat correlation (`{N{bit}}`) inside the small-set
    proof.

**Why**

The first repo-owned `tool_matrix` slice was useful precisely because
it turned a vague cleanliness goal into a concrete bucket:
Verilator-cleanliness failures concentrated in obviously-constant
unsigned comparisons.

Those warnings still matter even when the user deliberately asks for
lower factorization or `relaxed` identity. They are not useful
redundancy stress; they are locally-provable tautologies that muddy the
output and make the downstream-cleanliness lane harder to trust.

So this slice moves the proof earlier: if ANVIL can already prove that
an unsigned comparison is constant, it should emit the constant
directly regardless of sharing mode.

**Validation**

- `cargo test comparison_range_fold -- --nocapture`
- `cargo run --bin tool_matrix -- --out /tmp/anvil-tool-matrix-warning-fix --modules-per-scenario 1`
  - Verilator: 8 pass / 7 fail
  - Yosys: 15 pass / 0 fail
- `cargo run --bin tool_matrix -- --out /tmp/anvil-tool-matrix-warning-fix-2 --modules-per-scenario 1`
  - Verilator: 14 pass / 1 fail
  - Yosys: 15 pass / 0 fail
- `cargo run --bin tool_matrix -- --out /tmp/anvil-tool-matrix-warning-fix-4 --modules-per-scenario 1`
  - Verilator: 13 pass / 2 fail
  - Yosys: 15 pass / 0 fail
  - remaining Verilator warnings are now down to two correlation-heavy
    shapes:
    - `int_nodeid_constant-fold_default/mod_6_0000.sv`: `le_4`
    - `shuf_nodeid_egraph_motif_heavy_seq/mod_12_0000.sv`: `lt_0`,
      `ge_40`
- full hygiene rerun recorded in the commit workflow for this slice.

**Impact**

- Generator output is materially cleaner across the scenario matrix,
  especially below `peephole` and under `relaxed` identity, because
  provably-constant comparisons no longer depend on the factorization
  ladder to disappear.
- The repo-owned matrix improved from the original 7/15 Verilator-clean
  state after the harness landed to 13/15 Verilator-clean with Yosys
  still 15/15 clean.
- The remaining warning bucket is now much smaller and more specific:
  it has moved from easy boundary tautologies to correlation-heavy
  compare shapes that need a stronger local proof or a construction-time
  avoidance rule.

**Files touched**

- `CHANGES.md`
- `MEMORY.md`
- `DEVELOPMENT_NOTES.md`
- `CODEBASE_ANALYSIS.md`
- `ROADMAP.md`
- `README.md`
- `USER_GUIDE.md`
- `book/src/architecture.md`
- `book/src/structural-rules.md`
- `src/gen/cone.rs`

## 2026-04-20-0089 — Add a repo-owned Verilator/Yosys tool matrix harness

**Landed as:** `5eba379`

**What changed**

This slice turns the "exercise the full adversarial axis matrix"
doctrine into an executable repo-owned tool instead of leaving it as
documentation plus one-off shell commands.

### A new `tool_matrix` binary now drives curated downstream sweeps

- Added `src/bin/tool_matrix.rs`.
- The binary builds a curated scenario matrix that covers:
  - interleaved runs across `relaxed` plus every
    `factorization_level` rung;
  - all three live construction strategies
    (`sequential` / `shuffled` / `interleaved`);
  - a share-heavy comb-only stress profile; and
  - a motif-heavy sequential stress profile.
- For each scenario it:
  - generates a per-scenario corpus;
  - writes a scenario-local `manifest.json`;
  - runs Verilator and Yosys on every emitted file;
  - aggregates per-file metrics, tool outcomes, and coverage facts; and
  - writes a top-level `tool_matrix_report.json`.
- The harness exits non-zero on downstream-tool failures, because the
  point is to surface generator bugs rather than quietly accept red
  runs.

### The matrix reuses existing metrics as its coverage proof surface

- The report aggregates:
  - gate kinds / gate categories,
  - block counters,
  - per-knob roll attempts/fires,
  - comb-only vs sequential presence,
  - construction-strategy / identity-mode / factorization coverage.
- Added binary-level unit tests covering:
  - unique scenario naming,
  - full factorization-rung coverage,
  - all live construction strategies, and
  - coverage-gap detection.

### The first matrix run immediately found and helped fix a real bug

- The first `tool_matrix` smoke run found a hard Verilator error in the
  emitter: scalar `Slice { hi: 0, lo: 0 }` was serialized as
  `a[0:0]` even when `a` was already scalar `logic`.
- `src/emit/sv.rs` now renders that case as just `a`, and emits
  single-bit vector selections as `a[3]` instead of `a[3:3]`.
- Added an emitter unit test pinning the scalar-slice case.

### The docs now point at the new harness and record what it revealed

- `README.md`, `USER_GUIDE.md`, `ROADMAP.md`,
  `DEVELOPMENT_NOTES.md`, `CODEBASE_ANALYSIS.md`, and the book now
  document the `tool_matrix` binary, how to run it, and what role it
  plays in the Phase-1 / signoff-cleanliness lane.
- The phase/status docs now say the truthful current thing:
  - the repo-owned matrix exists;
  - Yosys is already green on the first smoke run; and
  - Verilator is not yet warning-clean across the matrix.

**Why**

The user has repeatedly pushed ANVIL toward two connected goals:

- model all adversarial-generation axes explicitly, with no hidden
  bias; and
- make clean downstream-tool behavior a real quality bar, not a vague
  aspiration.

The new harness is the first direct implementation of that doctrine.
It also paid for itself immediately: rather than just adding a test
surface, it found a real emitter defect and then narrowed the remaining
tool-clean gap to a concrete warning bucket (`CMPCONST` / `UNSIGNED`)
that the next bug-fix slice can attack directly.

**Validation**

- `cargo check --all-targets`
- `cargo test`
- `cargo clippy --all-targets -- -D warnings`
- `cargo fmt --all --check`
- `mdbook build book`
- `cargo run --bin tool_matrix -- --list-scenarios`
- `cargo run --bin tool_matrix -- --out /tmp/anvil-tool-matrix-smoke-2 --modules-per-scenario 1`
  - expected non-zero today because the matrix is not fully
    Verilator-clean yet
  - observed result after the scalar-slice fix:
    - Verilator: 7 pass / 8 fail
    - Yosys: 15 pass / 0 fail
    - coverage gaps: 0

**Impact**

- ANVIL now has a repo-owned executable proof surface for the
  adversarial axis matrix instead of relying only on prose and ad hoc
  commands.
- The first hard emitter bug surfaced by the matrix is fixed.
- The remaining tool-clean gap is now sharply characterized:
  Verilator warning-cleanliness (`CMPCONST` / `UNSIGNED`) is the next
  tactical lane, while Yosys is already green across the smoke matrix.

**Files touched**

- `CHANGES.md`
- `MEMORY.md`
- `README.md`
- `ROADMAP.md`
- `DEVELOPMENT_NOTES.md`
- `CODEBASE_ANALYSIS.md`
- `USER_GUIDE.md`
- `book/src/architecture.md`
- `book/src/recipes.md`
- `src/bin/tool_matrix.rs`
- `src/emit/sv.rs`

## 2026-04-20-0088 — Broaden ANVIL toward multiple synthesizable artifact families

**Landed as:** `ca2947b`

**What changed**

This slice captures a new steering conclusion across the roadmap, live
docs, and book: ANVIL is no longer documented as only a leaf-module RTL
generator. That lane stays intact, but it is now explicitly framed as
the first artifact family in a broader valid-by-construction
synthesizable HDL generator.

### The product scope is now documented as multiple valid artifact families

- `README.md`, `ROADMAP.md`, `DEVELOPMENT_NOTES.md`, and
  `CODEBASE_ANALYSIS.md` now say the same thing:
  - the current leaf-module typed-circuit lane remains valid and
    important;
  - the project is broadening to more artifact families, not relaxing
    the quality bar; and
  - every future family must stay valid-by-construction and
    synthesizable.
- The docs now explicitly capture the user's clarification that the
  "valid-by-construction synthesizable lane" remains in force while
  ANVIL grows into a broader pseudo-random HDL generator.

### The roadmap now names the next artifact-family layers

- `ROADMAP.md` now has a durable "broader artifact-family mandate"
  section near the top.
- Three new roadmap phases were added:
  - **Phase 7** — oracle-backed micro-design artifacts;
  - **Phase 8** — frontend/elaboration accept corpora; and
  - **Phase 9** — a multi-artifact ANVIL umbrella.
- These phases describe the requested direction more concretely:
  source-level parameter / hierarchy / package aware generation,
  expected-facts manifests, and compact synthesizable `.sv` corpora
  with known elaboration facts.

### Expected-facts manifests are now documented as in-scope

- The docs now distinguish carefully between two ideas:
  - a bundled shadow simulator or general oracle, which remains out of
    scope; and
  - explicit expected-facts manifests for artifact families that need
    them, which are in scope.
- This distinction is now reflected consistently in the roadmap, the
  contributor notes, the codebase analysis, and the book.

### The stale contradictions were cleaned up

- Removed the remaining stray "reject corpora" wording from the
  roadmap.
- Updated the book and live docs to stop claiming the live
  factorization ladder only reaches `peephole`; they now say the truth:
  ANVIL has a bounded live `e-graph` fragment, while fuller semantic
  equivalence remains future work.
- Clarified a few now-misleading phase/status labels in the README and
  structural rules.

**Why**

The user broadened the mandate in a very specific way: ANVIL should
become the go-to tool for pseudo-random HDL generation more broadly,
but it must do so by adding more **valid-by-construction synthesizable**
artifact families rather than by weakening the existing discipline.

That matters because the earlier draft of the docs still contained one
important wrong implication: broadening had started to drift toward
"reject corpus" language. The user's correction rules that out. The
durable documentation now reflects the intended direction precisely:
keep the current signoff-grade synthesizable lane, add more
synthesizable lanes, and give those new families explicit contracts.

**Validation**

- `cargo check --all-targets`
- `cargo test`
- `cargo clippy --all-targets -- -D warnings`
- `cargo fmt --all --check`
- `mdbook build book`

**Impact**

- Future implementation work now has a clearer steering target:
  broaden ANVIL by adding new synthesizable artifact families rather
  than by diluting the existing DUT lane.
- The roadmap now contains explicit phases for oracle-backed
  micro-designs, frontend/elaboration accept corpora, and the
  multi-artifact umbrella that will eventually coordinate them.
- Session recovery is safer because the roadmap, live docs, and book
  now tell the same story about what ANVIL is growing into.

**Files touched**

- `CHANGES.md`
- `MEMORY.md`
- `README.md`
- `ROADMAP.md`
- `DEVELOPMENT_NOTES.md`
- `CODEBASE_ANALYSIS.md`
- `book/src/introduction.md`
- `book/src/faq.md`
- `book/src/non-goals.md`
- `book/src/non-triviality.md`
- `book/src/synthesizability.md`
- `book/src/hierarchy.md`
- `book/src/algorithm.md`
- `book/src/structural-rules.md`

## 2026-04-20-0087 — Activate bounded semantic gate merging at `e-graph`

**Landed as:** `58c31cc`

**What changed**

This slice makes the top factorization rung honest in code, not just in
aspiration: `e-graph` is now a live bounded semantic-sharing fragment
under `identity_mode = node-id`.

### The `e-graph` rung is now partially implemented

- `src/config.rs` now treats `FactorizationLevel::EGraph` as an
  implemented rung instead of clamping it down to `Peephole`.
- The docs on the enum and the identity-mode tests now say the truthful
  thing: `e-graph` is still not the full semantic-equivalence engine,
  but it is no longer just an alias for the lower ladder.

### Post-construction combinational semantic sharing is now live

- `src/ir/compact.rs` already had the bounded proof machinery; this
  slice wires it into the real module-finalisation flow.
- `src/gen/module.rs` now runs
  `crate::ir::compact::merge_equivalent_gates(&mut m)` after
  `summarize_flop_mux_metadata` and before the post-drain flop merge.
- The pass is deliberately gated the strict way:
  - `identity_mode = node-id`
  - effective factorization level `>= e-graph`
  - same canonical leaf endpoints
  - same proven functionality (normalized structural proof first,
    bounded truth-table proof for small-support cones)
- `identity_mode = relaxed` remains a real off-switch; a new unit test
  pins that bypass directly.

### New telemetry makes the live `e-graph` fragment measurable

- `Module` and `Metrics` now expose `semantic_gates_merged: u32`.
- `Metrics::compute` now surfaces the count alongside
  `flops_merged` / `nodes_compacted`.
- The metrics test now asserts both post-construction merge counters.

### The docs now frame ANVIL as an axis matrix, not one randomness dial

- Live docs + book now say the durable steering rule explicitly:
  adversarial generation is the cross-product of orthogonal axes, not a
  single blended notion of "randomness".
- The named axes now captured in the durable docs are:
  - construction strategy (`sequential`, `shuffled`, `interleaved`,
    `graph-first` alias),
  - identity mode (`node-id` vs `relaxed`),
  - factorization level,
  - motif/category weights,
  - sequential density / widths / depths,
  - probability knobs.
- The new wording also captures the user's "no hidden bias" directive:
  these axes should be exercised efficiently during actual generation
  rather than only existing on paper.

**Why**

The previous slice made bounded semantic proof real for flop identity,
but the user then sharpened the control-surface doctrine again:

- `node-id as identity == full factorization` and `relaxed` are one
  axis;
- construction strategies are a separate axis;
- knobs and categories are another set of axes;
- and ANVIL should model the adversarial space without hidden bias.

Leaving `e-graph` clamped to `peephole` while claiming stronger
factorization would have kept the strongest mode misleading. This slice
lands the first honest combinational semantic fragment and updates the
docs so the project can steer from the real axis model instead of a
blurrier one.

**Validation**

- `cargo test merge_equivalent_gates -- --nocapture`
- `cargo test effective_factorization_level_respects_identity_mode -- --nocapture`
- `cargo test identity_mode_controls_whether_nodeid_means_expression_identity -- --nocapture`
- `cargo check --all-targets`
- `cargo test`
- `cargo clippy --all-targets -- -D warnings`
- `cargo fmt --all --check`
- `mdbook build book`
- `cargo run --quiet -- --seed 42 --count 1 --out /tmp/anvil-smoke-Fbucnm`
- `verilator --lint-only /tmp/anvil-smoke-Fbucnm/mod_42_0000.sv`
- `yosys -p "read_verilog -sv /tmp/anvil-smoke-Fbucnm/mod_42_0000.sv; synth"`

**Impact**

- `--identity-mode node-id --factorization-level e-graph` now does more
  than `peephole`: small-support same-endpoint combinational cones can
  collapse even when their graph shapes differ.
- `--identity-mode relaxed` still allocates fresh identities and skips
  both semantic merge passes.
- The docs now steer future work toward explicit axis coverage and away
  from accidental implementation bias.

**Files touched**

- `CHANGES.md`
- `MEMORY.md`
- `ROADMAP.md`
- `DEVELOPMENT_NOTES.md`
- `CODEBASE_ANALYSIS.md`
- `USER_GUIDE.md`
- `book/src/algorithm.md`
- `book/src/architecture.md`
- `book/src/factorization.md`
- `book/src/faq.md`
- `book/src/ir.md`
- `book/src/knobs.md`
- `book/src/recipes.md`
- `book/src/sequential.md`
- `book/src/structural-rules.md`
- `src/config.rs`
- `src/gen/module.rs`
- `src/ir/compact.rs`
- `src/ir/types.rs`
- `src/metrics.rs`

## 2026-04-20-0086 — Add bounded semantic proofs to state identity

**Landed as:** `3cac9b6`

**What changed**

This slice strengthens `merge_equivalent_flops` again, but in a way
that matches the doctrine more closely: when two flop D-cones depend on
the same canonical leaf endpoints and ANVIL can *prove* they implement
the same function, they now merge even if their graph shapes differ.

### Small-support semantic state proofs now complement normalized structure

- `src/ir/compact.rs` now computes two kinds of D-cone proof:
  - the existing leaf-aware structural proof over the already-normalized
    IR; and
  - a new bounded semantic proof for small-support cones.
- The semantic proof enumerates all assignments over the cone's
  canonical primary-input / flop-Q endpoints when the total support is
  small enough (`<= 10` bits today), evaluates the D-cone under each
  assignment, and interns the resulting truth table together with the
  endpoint list.
- If that semantic proof is available, it becomes the flop-merge key.
  Otherwise the pass falls back to the structural proof.

### Different-shape but same-function cones can now merge

- Added a new compact-unit test with two 1-bit D-cones over the same
  endpoints `{a, b}`:
  - `(a & b) | (a & !b)`
  - `a & (b | !b)`
- These cones have different graph shapes, but the new bounded semantic
  proof shows they implement the same function, so the duplicate flop
  now merges.
- The existing negative test still pins the endpoint doctrine directly:
  `q0 + 1` and `q1 + 1` do **not** merge, because the endpoint
  variables differ.

### Docs now say the proof surface more precisely

- `ROADMAP.md`, `DEVELOPMENT_NOTES.md`, `CODEBASE_ANALYSIS.md`,
  `book/src/factorization.md`, and `MEMORY.md` now describe the live
  state-identity story as:
  - normalized structural proof first;
  - bounded semantic proof for small-support state cones; and
  - larger semantic equivalence still open work.

**Why**

The user tightened the doctrine again: same endpoint set plus same
functionality is the real rule, not same endpoint set plus same shape.

The previous slice corrected the endpoint side of that doctrine. This
slice advances the functionality side by landing a real proof tactic for
some differently-shaped cones instead of only talking about the goal.

**Validation**

- `cargo test merge_equivalent_flops -- --nocapture`
- `cargo check --all-targets`
- `cargo test`
- `cargo clippy --all-targets -- -D warnings`
- `cargo fmt --all --check`
- `mdbook build book`

**Impact**

- Strict `NodeId as identity` mode now has one genuine
  different-shape / same-function proof path in state merging.
- The implementation is still deliberately bounded and conservative.
- The remaining gap is now sharper: broader semantic equivalence across
  larger cones, richer state motifs, and eventually general output-cone
  identity.

**Files touched**

- `CHANGES.md`
- `MEMORY.md`
- `ROADMAP.md`
- `DEVELOPMENT_NOTES.md`
- `CODEBASE_ANALYSIS.md`
- `book/src/factorization.md`
- `src/ir/compact.rs`

## 2026-04-20-0085 — Align state identity with endpoint-aware functional proofs

**Landed as:** `ac243cd`

**What changed**

This slice corrects the previous state-identity broadening, but it does
not fall all the way back to exact `d: NodeId` equality. The doctrine is
now stated the strong way: two cones may share identity only when ANVIL
can prove they implement the same functionality with respect to the same
canonical leaf endpoints. For this slice, `merge_equivalent_flops`
implements a conservative proof subset of that doctrine.

### Stateful identity now uses endpoint-aware proof forms

- `src/ir/compact.rs` no longer uses the self-relative
  "rename each owning `q` to SELF" shortcut.
- Flop merging now keys D-cones by a leaf-aware interned signature over
  the already-normalized IR: primary inputs and flop-Q leaves are kept
  distinct by their real endpoint identities, constants are explicit,
  and gates are interned by op/width/operand proof ids.
- This means duplicate D-cones such as two separately-built `i0 + 1`
  cones now merge even when their literal `NodeId`s differ, while
  `q0 + 1` and `q1 + 1` still stay distinct because the endpoint
  variables differ.

### The tests now pin the doctrine directly

- Replaced the previous positive self-feedback merge test with the
  opposite assertion: cones that depend on different `Q` endpoints must
  **not** merge.
- Flipped the non-self duplicate-D test into the positive case: two
  separately-built cones over the same input endpoint set now do merge,
  even though their literal `NodeId`s differ.

### Docs were corrected to match the stricter meaning

- `ROADMAP.md`, `DEVELOPMENT_NOTES.md`, `CODEBASE_ANALYSIS.md`,
  `book/src/factorization.md`, and `MEMORY.md` now state the doctrine
  precisely: identity is about proven same functionality over the same
  canonical leaf variables, while the current implementation is only a
  conservative proof subset of that goal.

**Why**

The user clarified the intended doctrine precisely:

- two fanin cones may not have the same `NodeId` if they do not have the
  same endpoints as variables; and
- the right target is equality as the same function with respect to
  those same endpoints, even when the shapes differ.

Under that doctrine, the self-relative `SELF + 1` shortcut was too
permissive, while exact `d: NodeId` equality was too weak. This slice
lands the conservative middle step that is actually faithful to the
current proof surface.

**Validation**

- `cargo check --all-targets`
- `cargo test`
- `cargo clippy --all-targets -- -D warnings`
- `cargo fmt --all --check`
- `mdbook build book`

**Impact**

- Strict `NodeId as identity` mode now preserves real leaf variables
  while still merging duplicate endpoint-equal D-cones that were built
  separately.
- The docs now stop equating "same shape" with the doctrine itself.
- Future sequential-identity work now has a sharper bar: stronger
  equivalence must preserve canonical leaf variables and must be backed
  by an explicit proof, not an ad hoc renaming trick.

**Files touched**

- `CHANGES.md`
- `MEMORY.md`
- `ROADMAP.md`
- `DEVELOPMENT_NOTES.md`
- `CODEBASE_ANALYSIS.md`
- `book/src/factorization.md`
- `src/gen/module.rs`
- `src/ir/compact.rs`
- `src/ir/types.rs`

## 2026-04-20-0084 — Extend state identity to self-feedback-isomorphic flops

**Landed as:** `92c9ef7`

**What changed**

This slice strengthens the post-drain sequential identity pass in
`src/ir/compact.rs`. `merge_equivalent_flops` no longer requires two
flops to share the same exact `d: NodeId` in every case; it now also
merges the common self-feedback case where the two D-cones are
identical after renaming each flop's own `q` leaf to a synthetic
"self" token.

### Stateful identity is now stronger where exact NodeId equality could never hold

- Added a self-relative D-cone signature for flop merging:
  - if a D-subtree does not depend on the owning `q`, it stays keyed by
    exact `NodeId`;
  - if it does depend on the owning `q`, the signature records the gate
    structure recursively with the owning `q` normalized to `SelfQ`.
- This lets ANVIL merge flops like:
  - `d0 = q0 + 1`
  - `d1 = q1 + 1`
  even though the two D-cones cannot share the same literal `NodeId`
  because each one contains its own distinct `FlopQ` leaf.

### The pass still respects intentional duplication outside self-feedback

- Non-self duplicate D-cones remain exact-`NodeId` keyed.
- That means this slice does **not** silently bulldoze
  `max_ast_instances` or other duplication controls for generic
  combinational subgraphs; it only strengthens state identity where
  self-reference was the blocker.

### Tests and docs were updated with the new contract

- `src/ir/compact.rs` gained 2 new unit tests:
  - positive: self-feedback-isomorphic flops merge and validate after
    compaction;
  - negative: duplicated non-self D-cones stay distinct.
- Live docs + book were updated so they no longer describe the pass as
  exact-signature-only.

**Why**

The freshly-added four-gap steering map named stronger state identity as
the first live code gap. The next safe rung was the self-feedback case:
it is common, structurally clear, and blocked only by each flop owning a
different `q` leaf.

Landing that case moves NodeId-as-identity in the right direction
without pretending to solve general sequential equivalence or trampling
the existing duplication knobs.

**Validation**

- `cargo check --all-targets`
- `cargo test`
- `cargo clippy --all-targets -- -D warnings`
- `cargo fmt --all --check`
- `mdbook build book`

**Impact**

- Stateful sharing is now stronger for common register-style feedback
  motifs.
- The state-identity story in the codebase is more honest: no longer
  "exact `d: NodeId` only", but still clearly bounded.
- The next sequential-identity questions are now narrower and cleaner:
  wider graph isomorphism, richer state motifs, and eventually
  hierarchical identity.

**Files touched**

- `CHANGES.md`
- `MEMORY.md`
- `ROADMAP.md`
- `DEVELOPMENT_NOTES.md`
- `CODEBASE_ANALYSIS.md`
- `book/src/factorization.md`
- `src/ir/compact.rs`

## 2026-04-20-0083 — Make the four suitability gaps explicit across roadmap, book, and live docs

**Landed as:** `cb090be`

**What changed**

This slice turns the earlier codebase-suitability assessment into
durable project guidance. The four missing-or-diffuse points are now
spelled out explicitly in the roadmap, the book, and the contributor
docs, and the one stale Rustdoc claim around absorbing folds was
corrected so the code comments match the actual implementation.

### The roadmap now carries the four-gap steering map directly

- `ROADMAP.md` gained a dedicated section that makes these four gaps
  explicit instead of leaving them spread across later phases:
  1. feature breadth beyond the current leaf-module core;
  2. `NodeId` as identity / full-factorization mode;
  3. industrialized Verilator/Yosys cleanliness evidence; and
  4. structure-first implementation rather than whole-module
     functionality chasing.
- The roadmap now also marks Phases 4, 5, 5b, and 6 as
  `not started`, fixes the malformed Phase-5b heading, and records the
  identity implications for hierarchy, parameters, and advanced motifs.

### The live docs now answer "is the codebase suited?" in the same words

- `DEVELOPMENT_NOTES.md` now says the codebase is suited **as a
  foundation** because the typed IR, factorization chokepoint,
  post-drain finalisation, validator-owned invariants, explicit config
  surface, and dumb emitter all match the problem.
- `CODEBASE_ANALYSIS.md` now has a dedicated suitability section that
  maps the four gaps onto the actual Rust modules and current testing
  reality.
- `MEMORY.md` now records this as the newest doctrinal steering slice
  and points future sessions at the new anchors.

### The book now explains the same thing in the right places

- `book/src/architecture.md` now answers the suitability question
  explicitly and ties the four gaps to the current crate layout.
- `book/src/factorization.md` now explains what "full factorization"
  still means in roadmap terms: combinational identity is strong,
  exact-signature flop merge is live, but stronger sequential and
  hierarchical identity are not finished yet.

### Code-adjacent documentation was aligned too

- `src/ir/types.rs` no longer claims that absorbing folds only fire when
  no gate operand would be orphaned; the Rustdoc now reflects the real
  compaction-backed behavior.
- `src/gen/module.rs` now documents `generate_leaf_module` as the
  leaf-kernel entry point that future hierarchy should wrap rather than
  distort.

**Why**

The roadmap already contained pieces of all four points, but they were
diffuse enough that a future contributor could miss their combined
force:

- the need to grow far beyond a leaf-module generator;
- the difference between today's NodeId identity and the user's stronger
  "equivalent expressions should share one id" doctrine;
- the fact that signoff-quality cleanliness needs a real sweep harness,
  not just a few clean seeds; and
- the structure-over-whole-module-function doctrine.

Making those explicit reduces the chance of future implementation drift
and gives PNT choices a clearer, repo-owned filter.

**Validation**

- `cargo check --all-targets`
- `cargo test`
- `cargo clippy --all-targets -- -D warnings`
- `cargo fmt --all --check`
- `mdbook build book`

**Impact**

- The roadmap now answers the user's question honestly: the codebase is
  suited as a foundation, but the missing work is concrete and named.
- Future sessions have a much better chance of extending the code along
  the intended axes rather than improvising a different product.
- The "NodeId as identity" story is now more precise in both the live
  docs and the book.

**Files touched**

- `CHANGES.md`
- `MEMORY.md`
- `ROADMAP.md`
- `DEVELOPMENT_NOTES.md`
- `CODEBASE_ANALYSIS.md`
- `book/src/architecture.md`
- `book/src/factorization.md`
- `src/ir/types.rs`
- `src/gen/module.rs`

## 2026-04-20-0082 — Capture the structure-over-functionality doctrine verbatim

**Landed as:** `fc7ae3e`

**What changed**

This docs-only slice records, verbatim, the user's doctrinal
clarification that ANVIL is not targeting meaningful top-level module
functionality. It is targeting structure: legal, synthesizable, complex
RTL that downstream tools can ingest.

### The book now preserves the doctrine in its strongest form

- `book/src/core-idea.md` no longer claims that RTL must be
  functionally correct "to make sense to use" for ANVIL's purpose.
- The same chapter now contains the user's clarification **verbatim**
  under a dedicated doctrinal-anchor section.
- `book/src/introduction.md` and `book/src/faq.md` now distinguish:
  - whole-module intended behavior is generally absent; but
  - local motifs may still be functionally correct blocks.

### The live docs now steer contributors the same way

- `README.md` now says explicitly that whole-module intended behavior is
  not the goal; legal structure and tool-ingestible complexity are.
- `ROADMAP.md` now states that whole-module intended functionality is
  not a roadmap goal.
- `DEVELOPMENT_NOTES.md` now preserves the user's clarification
  **verbatim** and translates it into an implementation rule of thumb.

### Continuity docs were refreshed too

- `MEMORY.md` now records this as the latest doctrinal slice and points
  future sessions at the new verbatim anchors.
- The previous docs-only slice now has its landed hash filled in.

**Why**

Some existing wording still implied that a generated RTL module had to
be "functionally correct" in a broad top-level sense to be useful.
That is not ANVIL's mission.

Without a specification, whole-module function correctness is not even
well-defined for most generated outputs. ANVIL's by-construction target
is instead: legitimate structure, synthesizability, complexity, and
downstream-tool ingestibility. This slice makes that distinction
durable.

**Validation**

- `cargo check --all-targets`
- `cargo test`
- `cargo clippy --all-targets -- -D warnings`
- `cargo fmt --all --check`
- `mdbook build book`

**Impact**

- Future contributors now have a sharper doctrinal answer when they ask
  whether ANVIL should chase meaningful whole-module behavior.
- The core idea chapter no longer undermines the project direction with
  the wrong premise.
- Session recovery now carries the user's exact words, not only
  paraphrases.

**Files touched**

- `CHANGES.md`
- `MEMORY.md`
- `README.md`
- `ROADMAP.md`
- `DEVELOPMENT_NOTES.md`
- `book/src/core-idea.md`
- `book/src/introduction.md`
- `book/src/faq.md`

## 2026-04-20-0081 — Capture the signoff-grade bug-finder doctrine

**Landed as:** `3281e53`

**What changed**

This docs-only slice captures the newly-explicit product direction:
`anvil` should become a signoff-level quality random synthesizable RTL
generator whose outputs are clean in downstream tools by default and
still adversarial enough to expose real bugs in them.

### The entry-point docs now say the strong version out loud

- `README.md` now describes the destination as "legal, reproducible,
  adversarial RTL" rather than merely "valid synthesizable output".
- `ROADMAP.md` now states that signoff-grade cleanliness and feature
  breadth are simultaneous goals, not separate tracks.

### The book now distinguishes "no bundled simulator" from "not a bug finder"

- `book/src/introduction.md` now frames `anvil` as aiming higher than a
  syntax fuzzer: it should fail downstream tools, when it does, with
  structurally solid synthesizable RTL.
- `book/src/non-goals.md` now clarifies that "no oracle / no reference
  simulator" is an implementation-boundary choice, not a retreat from
  downstream-tool stress or quality expectations.
- `book/src/faq.md` now has an explicit Q/A entry saying yes, the goal
  is a signoff-grade bug finder for downstream parsers / elaborators /
  synthesizers — but by way of legal RTL, not malformed junk.

### Contributor-facing doctrine now steers future PNT choices

- `DEVELOPMENT_NOTES.md` gained a new core design-decision statement
  and a dedicated note that signoff cleanliness and downstream
  adversarial power are complementary goals.
- `MEMORY.md` now records this doctrine as a fresh handoff anchor, and
  the stale open question about whether the old `UNSIGNED` residue was
  acceptable lint noise has been removed.

**Why**

Recent user guidance made the product bar sharper than some older docs
still reflected. A few passages still sounded like downstream-tool
robustness was merely external validation or someone else's concern.

That wording would steer implementation in the wrong direction. The
project's intended value is not "random RTL that is usually okay"; it
is "high-quality legal RTL that is strong enough to find real bugs in
consumer tools". This slice makes that durable.

**Validation**

- `cargo check --all-targets`
- `cargo test`
- `cargo clippy --all-targets -- -D warnings`
- `cargo fmt --all --check`
- `mdbook build book`

**Impact**

- Future roadmap / PNT choices now have a clearer doctrinal filter:
  grow legal feature richness and keep tightening the clean-run bar.
- The book no longer conflates "we do not ship a simulator" with "tool
  bug-finding is not our concern".
- Session recovery is safer because the product direction is now stated
  in the durable docs, not only in transient conversation.

**Files touched**

- `CHANGES.md`
- `MEMORY.md`
- `README.md`
- `ROADMAP.md`
- `DEVELOPMENT_NOTES.md`
- `book/src/introduction.md`
- `book/src/non-goals.md`
- `book/src/faq.md`

## 2026-04-20-0080 — Fold Verilator tautology residues in the rewrite ladder

**Landed as:** `559a8be`

**What changed**

This slice closes the remaining seed-42 Verilator `UNSIGNED` /
`CMPCONST` residue by strengthening the factorization ladder itself
instead of suppressing those warnings downstream.

### Dynamic absorbing folds now fire through gate operands

- `fold_constants` now accepts the three absorbing identities even
  when the other operand(s) are interned gates:
  - `x & 0 -> 0`
  - `x | all_ones -> all_ones`
  - `x * 0 -> 0`
- The old "all other operands must be non-gates" restriction is gone.
- Rationale: `compact_node_ids` already runs at module finalisation
  and now owns cleanup of the orphaned dynamic subgraph.

### Peephole now removes the local tautology shapes Verilator flagged

- Added unsigned boundary rewrites for same-width comparisons:
  - `x < 0 -> 0`
  - `x >= 0 -> 1`
  - `x <= all_ones -> 1`
  - `x > all_ones -> 0`
  - plus the symmetric constant-on-LHS cases (`0 <= x -> 1`,
    `all_ones < x -> 0`, etc.).
- Added constant-selector mux collapse:
  - `Mux(0, a, b) -> b`
  - `Mux(1, a, b) -> a`
- These are now handled as local IR identities before emission, so
  the generated SV no longer carries these tautological comparisons
  into Verilator.

### Tests and docs were extended with the stronger rewrite contract

- `src/ir/types.rs` gained 3 new unit tests covering:
  - dynamic `Or(all_ones, gate)` absorption;
  - unsigned min/max boundary comparison folds; and
  - constant-selector mux collapse.
- Live docs + book now explain that compaction legitimises these
  stronger absorbing rewrites, and that the seed-42 lint residue was
  fixed at the IR-identity layer rather than by tool-specific
  suppression.

**Why**

The user-reported "unused bits / signal" thread led into a broader
lint-cleaning audit. After the stateful identity work, seed 42 was
still producing a small set of Verilator warnings that were all
structural tautologies:

- unsigned comparisons against impossible bounds (`x < 0`, `x >= 0`);
- const-selector muxes; and
- dynamic expressions that should have been absorbed by `0` or
  `all_ones`.

Those are not desirable "noise we tolerate"; they are missed local
identities. Fixing them in the rewrite ladder keeps the output cleaner
and moves `node-id` identity closer to the "same function, same node"
goal.

**Validation**

- `cargo check --all-targets`
- `cargo test` (99 unit + 24 integration = 123 passing tests)
- `cargo clippy --all-targets -- -D warnings`
- `cargo fmt --all --check`
- `mdbook build book`
- `cargo run -- --seed 42 --count 1 --out /tmp/anvil_unsigned_probe2`
- `verilator --lint-only /tmp/anvil_unsigned_probe2/mod_42_0000.sv`
  (clean, no warning-specific suppressions)
- `yosys -p "read_verilog -sv /tmp/anvil_unsigned_probe2/mod_42_0000.sv; synth; stat"`
  (0 problems)

**Impact**

- Seed-42 Verilator lint is now warning-clean without relying on
  `-Wno-UNSIGNED`.
- The factorization ladder is stronger in a way that aligns with the
  NodeId-identity objective: equivalent local forms collapse sooner,
  before emission.
- The remaining Phase-1 tooling gate is now the breadth sweep
  (1000-module Verilator + Yosys), not this known lint residue.

**Files touched**

- `src/ir/types.rs`
- `CHANGES.md`
- `MEMORY.md`
- `DEVELOPMENT_NOTES.md`
- `CODEBASE_ANALYSIS.md`
- `ROADMAP.md`
- `book/src/architecture.md`
- `book/src/factorization.md`
- `book/src/ir.md`
- `book/src/synthesizability.md`
- `book/src/structural-rules.md`

## 2026-04-20-0079 — Harden canonical flop identity validation

**Landed as:** `0a6cc89`

**What changed**

This slice turns `ir::validate::validate` into a real
stateful-identity contract checker, not just a gate-shape checker.

### Drive roots and flop-held `NodeId`s are now validated before indexing

- Added three new structural error classes:
  - `UndefinedDriveRoot`
  - `FlopIdMismatch`
  - `UndefinedFlopNode`
- Every output drive root must exist before the validator inspects
  cone roots or gate shape.
- Every flop table slot must keep the dense canonical relation
  `m.flops[idx].id == idx`.
- `Flop.d`, `Flop.q`, and every `NodeId` stored inside `FlopMux`
  must point at a live node.

### Canonical `Flop.q <-> Node::FlopQ` backreferences are now enforced

- `Flop.q` must point at a `Node::FlopQ`.
- That `Node::FlopQ` must point back to the same flop and carry
  the same width as the owning `Flop`.
- Every `Node::FlopQ` in the arena must:
  - reference a real flop;
  - match that flop's width; and
  - be the canonical `q` node for that flop.
- This catches stale duplicate Q nodes, renumbering mistakes, and
  dangling post-merge state references before later passes or the
  emitter can trust a broken IR.

### Validator helpers and tests were expanded alongside the contract

- Added `node_exists` and `validate_flop_mux_refs`.
- The node-side `FlopQ` width check now runs before the canonical-q
  check so the dedicated width-mismatch error path is reachable on
  stale duplicate `FlopQ` nodes too.
- `src/ir/validate.rs` gained 10 new stateful-invariant unit tests
  covering:
  - undefined drive roots;
  - flop-id mismatch;
  - missing D;
  - non-`FlopQ` q node;
  - q backref mismatch;
  - q width mismatch;
  - dangling / noncanonical / wrong-width `FlopQ` nodes; and
  - undefined mux-held node references.
- Live docs + book were refreshed so the validator is now described
  as owning the canonical state-backreference contract.

**Why**

The previous slice introduced post-drain flop merging under
`identity_mode = node-id`. That made `Flop.id`, `Flop.q`, and
`Node::FlopQ { flop, .. }` part of the recovery-critical identity
fabric, not incidental metadata.

Per-gate arity/width checks were no longer enough: a bad
renumbering pass, stale duplicate Q node, or dangling drive root
could panic later passes or silently corrupt the "NodeId means
identity" story. This hardening moves those failures into one
explicit development-time safety net.

**Validation**

- `cargo check --all-targets`
- `cargo test` (96 unit + 24 integration = 120 passing tests)
- `cargo clippy --all-targets -- -D warnings`
- `cargo fmt --all --check`
- `mdbook build book`

**Impact**

- Stateful identity now has a validator-backed contract instead of a
  best-effort convention.
- Post-drain state-rewrite passes may still renumber and merge, but
  they now have to leave a provably self-consistent IR behind.
- Session recovery docs now point at the exact invariants protecting
  future deeper sequential-factorization work.

**Files touched**

- `src/ir/validate.rs`
- `CHANGES.md`
- `MEMORY.md`
- `DEVELOPMENT_NOTES.md`
- `CODEBASE_ANALYSIS.md`
- `book/src/ir.md`
- `book/src/architecture.md`
- `book/src/by-construction.md`

## 2026-04-20-0078 — Extend NodeId identity to exact-signature flops

**Landed as:** `420fbd4`

**What changed**

This slice extends the `node-id` identity story from purely
combinational nodes into the first conservative sequential case:
duplicate flops.

### Post-drain exact-signature flop merge now runs at module finalisation

- Added `crate::ir::compact::merge_equivalent_flops(&mut Module) ->
  u32`.
- The pass runs only when:
  - `identity_mode = node-id`; and
  - the effective factorization level is at least `cse`.
- It executes after `drain_flop_worklist` and after
  `summarize_flop_mux_metadata`, when every flop finally has a
  concrete `d`.
- Flops merge when they have the same exact emitted-state
  signature:
  - `width`
  - `reset_kind`
  - `reset_val`
  - exact same `d: NodeId`

### The merge rewires state consumers, not just the flop table

- Duplicate Q users are rewritten to the canonical Q.
- Virtual flop deps inside `DepSet` are remapped through the
  old-flop-id -> new-flop-id table, so dependency tracking stays
  truthful after the merge.
- Surviving flops are renumbered densely; their `Flop.id` and
  `Node::FlopQ { flop, .. }` references are kept in sync.
- Dedup tables are rebuilt after the rewrite so the final module
  metadata matches the post-merge IR.
- The later `compact_node_ids` pass then removes the now-dead
  duplicate `FlopQ` nodes.

### Construction-only flop provenance is intentionally ignored

- The signature does **not** include `FlopKind`.
- The signature does **not** include `FlopMux` operand metadata.
- Rationale: by the time this pass runs, emitted hardware
  semantics are determined by width/reset/D. `FlopKind` and the
  cleared mux operands are construction provenance / telemetry,
  not emitted behavior.
- This means a `ZeroDefault`-born flop and a `QFeedback`-born
  flop can merge if they ended up with the same actual `d`.

### New telemetry surfaced the state-sharing result

- Added `Module::flops_merged: u32`.
- Added `Metrics::flops_merged: u32`.
- `generate_leaf_module` now records the merge count and logs it
  alongside node/flop/compaction totals.

### Tests and docs were updated to match the new semantics

- `src/ir/compact.rs` gained three merge-specific unit tests:
  - merge rewrites gate operands and virtual deps correctly;
  - `identity_mode = relaxed` bypasses the pass;
  - different reset signatures do not merge.
- Existing compaction tests remain intact.
- The live docs and book now state the important nuance
  explicitly:
  combinational factorization is mainly intern-time, but stateful
  exact-signature sharing is a post-drain finalisation step.

**Why**

The previous slice made identity mode a real typed axis, but the
stateful side still had a visible hole: `build_flop_leaf` always
allocated a fresh flop, so "NodeId is the identity of an
expression" silently stopped at registers.

There is no honest way to solve that at allocation time because a
flop's semantics are not known when its Q is born; the D-cone only
exists after the worklist drains. So the right next step was a
conservative post-drain pass that merges the cases we can prove
exactly today without pretending to solve general sequential
equivalence.

**Validation**

- `cargo check --all-targets`
- `cargo test` (86 unit + 24 integration = 110 passing tests)
- `cargo clippy --all-targets -- -D warnings`
- `cargo fmt --all --check`
- `mdbook build book`

**Impact**

- `identity_mode = node-id` now reaches one level deeper into the
  sequential fabric: exact duplicate state elements can share one
  identity too.
- The project now has a clean architectural split:
  - combinational identity at intern time;
  - conservative state identity after drain.
- `flops_merged` makes the new behavior measurable, so future
  stateful-factorization work has a baseline.
- The remaining gap is clear and documented: deeper sequential
  equivalence still needs a stronger coinductive / e-graph-style
  story.

**Files touched**

- `src/ir/types.rs`
- `src/ir/compact.rs`
- `src/gen/module.rs`
- `src/metrics.rs`
- `README.md`
- `USER_GUIDE.md`
- `DEVELOPMENT_NOTES.md`
- `CODEBASE_ANALYSIS.md`
- `ROADMAP.md`
- `book/src/algorithm.md`
- `book/src/architecture.md`
- `book/src/factorization.md`
- `book/src/faq.md`
- `book/src/ir.md`
- `book/src/knobs.md`
- `book/src/non-triviality.md`
- `book/src/sequential.md`
- `book/src/sharing.md`
- `book/src/structural-rules.md`
- `CHANGES.md`
- `MEMORY.md`

## 2026-04-20-0077 — Make identity mode a first-class typed axis

**Landed as:** `033e03d`

**What changed**

This slice turns "NodeId as identity" from a documented doctrine
plus CLI sugar into a first-class typed axis in the codebase.

### Config / CLI / IR now model identity mode explicitly

- Added `IdentityMode` in `src/config.rs`:
  - `node-id` (default): NodeId means expression identity; the
    factorization ladder stays live.
  - `relaxed`: disable the identity/factorization ladder
    entirely and allocate fresh NodeIds for every AST.
- Added `identity_mode` to `Config`, `Overrides`, `Cli`, and
  `Module`.
- Added `Config::effective_factorization_level()` and
  `Module::effective_factorization_level()` so the coarse
  identity mode is applied before every factorization gate.
- Added the new CLI flag:
  `--identity-mode <node-id|relaxed>`.

### Convenience aliases now expand to the explicit coarse+fine pair

- `--full-factorization` now means:
  `--identity-mode node-id --factorization-level e-graph`.
- `--no-full-factorization` now means:
  `--identity-mode relaxed --factorization-level none`.
- The aliases now conflict with explicit `--identity-mode` /
  `--factorization-level` so the CLI no longer silently mixes
  sugar and direct control.

### All identity/factorization gating now consults the effective mode

- `Module::intern_gate` and `Module::intern_constant` no longer
  read the raw ladder directly; they consult
  `self.effective_factorization_level()`.
- `gen::cone::{make_and, make_mul, violates_anti_collapse}` do
  the same, so operand-uniqueness / anti-collapse behavior now
  tracks the coarse identity mode consistently.
- `generate_leaf_module` now copies `cfg.identity_mode` into the
  per-module IR mirror just like the other construction-time
  knobs.

### Proof tests landed for the new semantics

- Added a config unit test proving:
  - `identity_mode = node-id, factorization_level = e-graph`
    resolves to `peephole` today;
  - `identity_mode = relaxed` forces the effective level to
    `none`.
- Added CLI unit tests for:
  - direct `--identity-mode relaxed` parsing;
  - `--full-factorization` setting both `identity_mode` and
    `factorization_level`;
  - `--no-full-factorization` doing the inverse.
- Added an IR unit test proving the same requested
  `factorization_level = e-graph`:
  - dedupes under `IdentityMode::NodeId`;
  - allocates fresh NodeIds under `IdentityMode::Relaxed`.

### Docs now describe the same model the code implements

- README / USER_GUIDE now document `--identity-mode` directly.
- The book chapters on knobs, factorization, structural rules,
  IR, architecture, sharing, non-triviality, algorithm, and FAQ
  now distinguish:
  - coarse identity mode;
  - fine-grained factorization rung;
  - construction strategy as a separate axis.
- `DEVELOPMENT_NOTES.md` records the design consequence:
  the separation is now in types and gating sites, not just in
  prose.
- `CODEBASE_ANALYSIS.md` and `MEMORY.md` were refreshed for
  session recovery.

**Why**

The repo had drifted into an awkward in-between state:

- the docs correctly said "NodeId as identity" is orthogonal to
  cone-construction strategy;
- the CLI had sugar for "full factorization on/off";
- but the code still treated the identity story mostly as a raw
  `factorization_level` ladder.

That mismatch was survivable for users, but bad for recovery and
future work. The next deeper step toward full factorization must
reason about identity mode explicitly, especially once flops and
future hierarchy enter the question. This slice makes the
separation real without changing the already-working ladder.

**Validation**

- `cargo check --all-targets`
- `cargo test` (83 unit + 24 integration = 107 passing tests)
- `cargo clippy --all-targets -- -D warnings`
- `cargo fmt --all --check`
- `mdbook build book`

**Impact**

- The codebase now has an honest first-class place to talk about
  "NodeId as identity" without smuggling it through the ladder.
- CLI/config/IR all agree on the model:
  coarse identity mode + fine-grained ladder.
- Future identity work for stateful / hierarchical objects can
  build on a real axis instead of more aliases and prose.

**Files touched**

- `src/config.rs`
- `src/main.rs`
- `src/gen/module.rs`
- `src/gen/cone.rs`
- `src/ir/types.rs`
- `README.md`
- `USER_GUIDE.md`
- `DEVELOPMENT_NOTES.md`
- `CODEBASE_ANALYSIS.md`
- `MEMORY.md`
- `ROADMAP.md`
- `book/src/algorithm.md`
- `book/src/architecture.md`
- `book/src/factorization.md`
- `book/src/faq.md`
- `book/src/ir.md`
- `book/src/knobs.md`
- `book/src/non-triviality.md`
- `book/src/sharing.md`
- `book/src/structural-rules.md`
- `CHANGES.md`

---

## 2026-04-20-0076 — Expose peak-sharing controls and exercise live categories

**Landed as:** `dd28086`

**What changed**

This slice tightens the control surface around the user's
"NodeId as identity" doctrine without conflating it with cone
construction strategy.

### CLI surface: peak sharing is now an explicit mode

- Added `--full-factorization` as a convenience alias for
  `--factorization-level e-graph` (request the strongest
  currently-implemented identity mode).
- Added `--no-full-factorization` as the coarse off-switch
  (`--factorization-level none`).
- Kept the existing detailed ladder under
  `--factorization-level <none|cse|operand-unique|commutative|associative|constant-fold|peephole|e-graph>`.
- Exposed previously config-only *live* knobs on the CLI:
  `--terminal-reuse-prob`, `--constant-prob`,
  `--gate-bitwise-weight`, `--gate-arith-weight`,
  `--gate-struct-weight`, `--gate-compare-weight`,
  `--gate-reduce-weight`.

### Dead leaf knobs are live now

- `gen::cone::pick_terminal` now consults
  `terminal_reuse_prob` at forced leaves with an exact-width
  pool candidate:
  - `1.0` = always reuse the matching-width signal;
  - `0.0` = never reuse it, emit a fresh constant instead.
- `pick_terminal` now consults `constant_prob` when no
  matching-width signal exists but a dep-bearing width-adapter
  source does:
  - hit = emit a fresh constant;
  - miss = build the width adapter.
- Both decisions route through `roll_knob`, so they become
  measurable in `knob_roll_attempts` / `knob_roll_fires`.

### Every live gate category is now genuinely exercisable

- `pick_gate`'s compare bucket now contains the full comparison
  family: `Eq`, `Neq`, `Lt`, `Gt`, `Le`, `Ge`.
- The reduction bucket is now live in `pick_gate`:
  `RedAnd`, `RedOr`, `RedXor` can be selected at 1-bit target
  width.
- `gate_reduce_weight` is therefore no longer a dead config
  field.

### Config / test hardening

- `Config::validate()` now rejects out-of-range
  `mux_arm_duplication_rate` and `operand_duplication_rate`
  values, matching their documented `[0.0, 1.0]` contract.
- Added unit tests for:
  - the new CLI aliases and newly-exposed CLI knobs;
  - probability validation of the two duplication-rate knobs;
  - `pick_gate` coverage across every live category;
  - `pick_terminal` edge behavior for
    `terminal_reuse_prob` and `constant_prob`.
- Added an end-to-end integration test proving that each live
  gate category is reachable in generated modules and still
  IR-valid.
- Extended the per-knob roll telemetry test to cover
  `constant_prob` and `terminal_reuse_prob`.

### Docs synced to shipping reality

- Refreshed stale factorization passages that still claimed only
  the first three rungs were implemented.
- Added the load-bearing clarification that construction
  strategy (`sequential` / `shuffled` / `interleaved`) is a
  separate axis from identity / sharing mode.
- Updated knob docs to reflect that `constant_prob`,
  `terminal_reuse_prob`, and the gate-category weights are live
  CLI-controlled knobs today.

**Why**

The user clarified the doctrinal target precisely:

- output drives and flop D inputs are cone roots;
- primary inputs and flop Qs are the leaves;
- the entire fanin forest should collapse toward a maximally
  shared DAG when the "NodeId as identity" mode is enabled;
- this identity mode is **orthogonal** to cone-construction
  strategy.

Before this slice, the coarse factorization dial existed, but
the user-facing on/off control was awkward, one gate category
(`reduce`) was effectively dead, and two documented leaf knobs
(`constant_prob`, `terminal_reuse_prob`) were not actually
consulted. Several guide/book passages also understated the
current live factorization ladder. This slice cleans that up so
the next architectural step toward deeper semantic identity can
start from a coherent, fully-exercised surface.

**Validation**

- `cargo check --all-targets`
- `cargo test` (80 unit + 24 integration = 104 passing tests)
- `cargo clippy --all-targets -- -D warnings`
- `cargo fmt --all --check`
- `mdbook build book`

**Impact**

- Peak sharing / "full factorization" now has an obvious coarse
  CLI on/off control that sits alongside, not inside, the
  construction-strategy choice.
- Every *live* gate category is now reachable and tested.
- Leaf-level sharing-vs-constant choices are no longer dead
  docs-only knobs; they affect generation and are measurable.
- The book/live docs now consistently describe the current
  factorization ladder as live through `peephole`, with
  `e-graph` still aspirational.

**Files touched**

- `src/main.rs`
- `src/config.rs`
- `src/gen/cone.rs`
- `src/ir/types.rs`
- `tests/pipeline.rs`
- `book/src/algorithm.md`
- `book/src/faq.md`
- `book/src/factorization.md`
- `book/src/knobs.md`
- `book/src/recipes.md`
- `book/src/structural-rules.md`
- `USER_GUIDE.md`
- `README.md`
- `DEVELOPMENT_NOTES.md`
- `CODEBASE_ANALYSIS.md`
- `ROADMAP.md`
- `MEMORY.md`
- `CHANGES.md`

---

## 2026-04-19-0075 — Finalise the live-signal surface for lint-clean output

**Landed as:** `e973d30`

**What changed**

This slice closes the "unused bits / unused signal" defect at the
generator/finalisation layer instead of hiding it in the emitter.

### Exact-width width adapters

- `gen::cone::make_width_adapter` no longer builds a wider replicated
  `Concat` and slices it back down for non-multiple up-width
  expansions.
- The adapter now emits the exact-width shape directly:
  `{src[rem-1:0], src, src, ...}`.
- This removes dead high bits like the old seed-42
  `concat_0[41:27]` case that Verilator flagged as unused.

### Finalisation now matches emitted hardware

- `gen::module::generate_leaf_module` gained a proper
  post-construction clean-up sequence:
  1. summarize `Flop.mux` metadata so construction-only select/data
     operand NodeIds do not keep dead cones alive;
  2. orphan audit;
  3. `compact_node_ids`;
  4. shrink surviving primary inputs to the highest bit any live
     consumer touches;
  5. prune entirely unused primary data-input ports from the emitted
     interface.
- This fixes the mismatch where IR liveness used `Flop.mux` bookkeeping
  as if it were emitted hardware, so metadata-only gates survived and
  later triggered `%Warning-UNUSEDSIGNAL`.

### Metrics / test semantics aligned with duplicate-preserving flattening

- `Metrics::nested_associative_operand_count` now counts only same-op
  nested slots that are still flattenable under the current duplicate
  policy.
- This stops strict `operand_duplication_rate = 0.0` `Add`/`Mul`
  shapes like `x * (x * y)` from being misreported as "missed"
  associative opportunities when flattening them would change
  semantics.

### Tests and docs

- Added two `src/gen/module.rs` unit tests for primary-input shrinking.
- Updated the width-adapter non-multiple unit test to pin the new
  exact-width Concat shape.
- Added integration test
  `no_unused_primary_data_inputs_remain_after_finalisation`.
- Renamed stale pipeline tests/comments that still described
  `graph-first` as the default strategy.
- Refreshed `src/config.rs`, `src/main.rs`, `USER_GUIDE.md`,
  `ROADMAP.md`, `DEVELOPMENT_NOTES.md`, `CODEBASE_ANALYSIS.md`, and
  `book/src/knobs.md` to reflect:
  - `interleaved` is the live default;
  - `graph-first` is a deprecated alias;
  - `graph_first_pool_size` is legacy on the live path;
  - finalisation trims dead input ports / dead input high bits.

**Why**

The reported bug was "unused bits of signal", initially blamed on the
graph-first family. Reproduction showed two distinct root causes:

1. width-adapter expansion created intentionally oversized Concats,
   leaving dead high bits in otherwise-live internal wires; and
2. `Flop.mux` metadata was being treated as a liveness root even
   though the emitter only consumes `flop.d`.

Once those were fixed, a seed sweep still exposed unused *input* high
bits coming from low-slice-only consumers, so finalisation now shrinks
the live input surface as well. The graph-first diagnosis was partly a
doc/test mirage: the current CLI `graph-first` path is just the
interleaved builder under a deprecated alias, so the stale comments had
to be cleaned up in the same slice.

**Validation**

- `cargo check --all-targets`
- `cargo test`
- `cargo clippy --all-targets -- -D warnings`
- `cargo fmt --all --check`
- `mdbook build book`
- `verilator --lint-only -Wall -Wno-DECLFILENAME -Wno-UNSIGNED /tmp/anvil_seed42_final.sv`
- Verilator unused-signal sweep (seeds 0..4) clean for both the default
  path and the `graph-first` alias
- `yosys -p "read_verilog -sv /tmp/anvil_seed42_final.sv; synth"`

**Impact**

- Emitted modules no longer carry metadata-only orphan wires into SV.
- Non-multiple width adaptation no longer manufactures dead high bits.
- Final SV interfaces no longer include dead data-input ports or dead
  input high bits.
- The graph-first / interleaved story is now factually documented at
  the CLI, test, user-guide, roadmap, and book levels.
- The associative-opportunity metric now matches the live semantic
  policy for strict duplicate preservation.

**Files touched**

- `src/gen/cone.rs`
- `src/gen/module.rs`
- `src/metrics.rs`
- `src/main.rs`
- `src/config.rs`
- `src/ir/types.rs`
- `src/ir/compact.rs`
- `tests/pipeline.rs`
- `USER_GUIDE.md`
- `ROADMAP.md`
- `DEVELOPMENT_NOTES.md`
- `CODEBASE_ANALYSIS.md`
- `book/src/knobs.md`

---

## 2026-04-17-0074 — All-constant evaluation completes the constant-fold surface

**Landed as:** `30753c8`

**What changed**

Closes the remaining all-const evaluation gaps flagged in the
last two slices. The factorization pipeline now evaluates every
pure-function gate at intern time when every operand is a
constant of the expected width.

### `Module::fold_constants` — extended

- **All-const associative evaluation** (Layer 5): for
  `And`/`Or`/`Xor`/`Add`/`Mul` with every operand a same-width
  constant, compute the result directly (bitwise AND / OR / XOR
  over values, sum / product mod 2^width) and intern the
  resulting constant. Inserted before the existing absorbing
  and identity-drop branches so it supersedes them for the
  all-const subcase — `Add(3, 5)` folds to 8 directly instead
  of going through identity-drop (which wouldn't have worked
  anyway since 5 isn't the identity).
- **All-const Sub / Shl / Shr**: for the existing 2-arity
  non-commutative arm, added upfront all-const evaluation that
  handles `Sub(c1, c2) → (c1 - c2) mod 2^width`,
  `Shl(c1, c2) → (c1 << c2) mod 2^width` (over-shift → 0), and
  `Shr(c1, c2) → c1 >> c2` (over-shift → 0). For Shl/Shr the
  shift-amount constant can have its own narrower width — we
  read the value and only require the lhs to match the gate
  width.

The existing identity-drop + absorbing paths remain for mixed
operand lists (one constant + one primary input / flop Q). This
is intentional and desirable: those paths catch valuable
partial folds even when the expression isn't fully constant.

### `Module::apply_peephole` — extended

- **`Concat([c1, c2, ...]) → assembled const`** when every
  operand is a constant. MSB-first bit assembly matching the SV
  emit convention in `src/emit/sv.rs` — `{c1, c2, c3}` packs
  `c1` into the high bits. Widths must sum to the gate width;
  any mismatch defensively skips the fold rather than emit a
  wrong-width constant.

### Counters

Both helpers keep reusing the existing counters
(`Module::fold_identities_applied` and
`peephole_rewrites_applied`) — all-const evaluation fires in
the same helper, so the count aggregates with the existing
rules for that helper.

### Tests

Eight new unit tests in `src/ir/types.rs`:

- `fold_all_const_add_evaluates`
- `fold_all_const_mul_wraps_modulo_width` (verifies 8-bit
  `Mul(100, 3)` → 44, i.e. 300 mod 256)
- `fold_all_const_xor_evaluates`
- `fold_all_const_sub_evaluates` (both positive and
  wrap-around cases)
- `fold_all_const_shl_evaluates_and_clamps` (over-shift clamps
  to zero)
- `fold_all_const_shr_evaluates`
- `peephole_concat_of_constants_assembles_msb_first`
  (`{4'hA, 4'h5}` → 8'hA5)
- `peephole_concat_of_constants_variadic`
  (`{3'b101, 2'b01, 1'b1}` → 6'b101011)

### Docs

- `book/src/factorization.md`: Layer 3 (constant folding) rule
  table split into associative + non-commutative sub-tables
  and extended with the all-const columns. Peephole's Concat
  bullet updated with the bit-assembly rule.
- `book/src/structural-rules.md` Rule 21c: `constant-fold` and
  `peephole` level-table rows extended to list the new rules.
- `src/ir/types.rs`:
  [`Module::fold_constants`] Rustdoc rule tables split into
  associative (with All-const / Identity-drop / Absorbing
  columns) and non-commutative (with All-const / Rhs-zero
  identity columns). `apply_peephole` Rustdoc gains the
  Concat all-const bullet and the
  `peephole_rewrites_applied` field comment lists it.

**Why**

The previous slices (`2de8855` peephole Not/Slice/reductions,
`5f51c3b` associative flattening) flagged two known gaps:
`Concat(all-const)` and `Shl/Shr(const, const)`. Plus one gap
surfaced while writing unit tests for the Slice slice —
`Add(3, 5)` and similar fully-constant expressions weren't
evaluated (the existing absorbing + identity-drop paths only
fire when a specific absorbing/identity value is present).
This slice closes all three in one pass under the common
framing "evaluate the gate at intern time when every operand
is a constant".

After this slice, the `NodeId = expression identity` contract
holds for every syntactically-or-algebraically-evaluable
expression: every all-constant expression collapses to a
single constant node; every same-op-same-width nesting
flattens; every commutative-reordering is canonicalised;
every identity / absorbing / peephole identity collapses.
What remains genuinely unaddressed is **cross-gate symbolic
rewrites** over non-constant expressions (`(a + b) - b → a`,
`(a & b) | (a & ~b) → a`, etc.), which is the e-graph
problem — research-adjacent and still aspirational.

**Empirical (seed 42, default knobs):**

Counters unchanged at seed 42 (default construction path
rarely produces all-constant operand lists directly). The new
rules fire whenever earlier folds / peepholes produce constant
intermediates that flow into these shapes — same pattern as
the Not/Slice/reduction slice.

**Tests**
- 70 unit tests pass (was 62 — added 8).
- 22 integration tests pass (unchanged).
- Total test count: 92.
- `cargo build` clean; `mdbook build book` clean.

**Impact**
- The "constants flow through pure operators" story is now
  complete for every operator class. Pipelines that construct
  a constant-heavy shape via CSE / fold chains collapse all
  the way to a single constant.
- Rule 21c level table and `factorization.md` reflect every
  implemented rule — no known gaps remain in the
  documentation.

---

## 2026-04-17-0073 — Thorough docs pass on factorization pipeline (docs only)

**What changed**

User asked for the `apply_peephole` / `fold_constants` /
`flatten_associative` / `compact_node_ids` / knob-telemetry
surface to be thoroughly documented, both in-code and in the
book. This slice closes the gap.

### In-code Rustdoc upgrades (`src/ir/types.rs`)

- `Module::intern_gate` — rewrote from a one-paragraph note
  into a full pipeline-orchestrator spec: returns convention,
  numbered six-step pipeline (Associative → Commutative → Fold
  → Peephole → None bypass → CSE), orphan-safety contract,
  determinism guarantee.
- `Module::intern_constant` — expanded to cover the dedup key,
  cap semantics, why no factorization layers apply, and the
  `make_constant` wrapper pattern.
- `Module::fold_constants` — added a table of all rules by op
  class, the absorbing-rule orphan-safety restriction, the
  non-commutative position-sensitivity note, the explicit
  out-of-scope list.
- `Module::flatten_associative` — upgraded header to call out
  Layer-4 / `FactorizationLevel::Associative` framing and
  clarified the return convention.
- `Module::apply_peephole` — **rewrote the stale doc** (listed
  only the original four rules and missed every rule added
  since). New doc groups rules by outer operator: Not (const
  eval, involutive, comparison inversion), comparisons
  (all-const eval), Slice (full-width identity + const eval),
  Concat (single-operand identity), reductions (const eval).
  Counter mechanism documented.
- `KnobRollCounters::record` — gained a doc comment explaining
  when it's called (`roll_knob` helper in cone.rs) and how to
  interpret `fires / attempts`.
- `Module::factorization_level` field — stale "Default `Full`"
  replaced with the real default (`EGraph`) and the ladder
  listing.
- `Module::peephole_rewrites_applied` field — stale list
  (missed Not(cmp) inversions, all-const Not/Slice/reduction
  evaluations) replaced with the complete catalogue and a
  pointer to Rule 21c.

### New book chapter: `book/src/factorization.md`

A dedicated "How It Works" chapter walking through the full
`intern_gate` pipeline end-to-end. Sections:

1. Why factorize (doctrinal anchor: NodeId = expression identity)
2. The ladder (enum layers + selection)
3. Pipeline in execution order (layers 1–6 with per-layer
   tables, short-circuit semantics, and reasoning)
4. Orphan safety and the compaction pass
5. Empirical counters with a seed-42 baseline table
6. Turning layers off — paste-and-run knob-sweep recipe
7. Pointers to related chapters and source

Progressive disclosure per the book doctrine: Rule 21c in
`structural-rules.md` stays as the rule-catalogue entry;
`factorization.md` is the narrative walkthrough for readers
who want to understand what anvil does to every gate.

Added to `book/src/SUMMARY.md` under "How It Works".

### `book/src/ir.md` updates

- `Module` struct listing refreshed to include every current
  field: `operand_duplication_rate`, `factorization_level`,
  block counters, fold/peephole/flatten/compaction counters,
  `knob_rolls`.
- "Node construction" section rewritten with three subsections:
  CSE semantics (Rule 21), The full intern pipeline (Rule 21c
  with numbered layer list + pointer to `factorization.md`),
  Orphan safety via compaction. Snapshot contract kept as the
  final subsection.

### `book/src/architecture.md` updates

- Crate layout refreshed: `types.rs` description now mentions
  all current fields and helpers; new entry for
  `compact.rs` that was missing entirely.
- "Key types at a glance" `Module` block extended to show the
  factorization counters + `knob_rolls`. `intern_gate`
  signature gains a layered-pipeline doc comment. New entries
  for `compact_node_ids`, `KnobId`, `KnobRollCounters`.

### Doctrinal consistency

Every public / `pub(crate)` factorization function now has a
doc comment that (a) names its layer position, (b) lists its
rules, (c) documents its return convention, (d) cross-links
the book chapter where applicable. The book chapters
cross-link back to the source. No more "what does this do" for
a user reading `-- --help` output plus the book.

**Why**

User directive: "all these functions and every else shall be
thoroughly documented when they are part of the user facing
surface. Not only that, the book shall contain the accurate
description of all these internal algorithms and functions."
Prior slices landed the functionality but inherited stale or
terse doc strings; this slice catches up.

**Tests**
- 62 unit tests pass (unchanged — docs-only slice).
- 22 integration tests pass (unchanged).
- Total test count: 84.
- `cargo build` clean; `mdbook build book` clean.

**Impact**
- Readers of `cargo doc --open` now get a complete
  factorization-pipeline reference via the module-level
  summaries on `intern_gate` and the layer helpers.
- Readers of the book get a dedicated chapter that explains
  the pipeline rather than having to assemble the picture
  from Rule 21c, architecture.md, and ir.md.
- Future slices that add a layer (or promote an aspirational
  one) have a clear template: update the `intern_gate`
  pipeline list, Rule 21c table, `factorization.md` layer
  list, and the per-field counter doc.

---

## 2026-04-17-0072 — Peephole all-const evaluation: Not, Slice, reductions

**What changed**

Extended `Module::apply_peephole` with four new constant-folding
rules that extend the "evaluate at intern time when all operands
are constants" pattern (previously restricted to comparisons):

- **`Not(c)`** → `~c & mask(width)`. Handled first in the `Not`
  arm, before the `Not(Not)` and `Not(cmp)` cases.
- **`Slice(hi, lo)(c)`** → `(c >> lo) & mask(hi - lo + 1)`.
  Added to the existing `Slice` arm alongside the full-width
  identity rule.
- **`RedAnd(c)`** → `(c == all_ones(src_width)) as 1-bit`.
- **`RedOr(c)`** → `(c != 0) as 1-bit`.
- **`RedXor(c)`** → `popcount(c) & 1` as 1-bit.

All three reductions share a new arm matching
`GateOp::RedAnd | GateOp::RedOr | GateOp::RedXor` with
`operands.len() == 1`. Width invariants: reductions always
produce 1-bit output regardless of operand width.

Fires share the existing `peephole_rewrites_applied` counter.
Constants folded by these rules are orphan-safe (the outer
unary op never materialises; the inner constant operand may be
unreferenced but `compact_node_ids` only tracks Gate orphans,
not Constant orphans).

Tests:
- `src/ir/types.rs`: three new unit tests —
  `peephole_not_of_constant_folds`, `peephole_slice_of_constant_folds`,
  `peephole_reductions_of_constants_fold`. Plus the previous
  slice's `peephole_not_eq_of_constants_folds_to_bit` is
  upgraded to `peephole_not_eq_of_constants_folds_end_to_end`
  — it now asserts the full `Not(Eq(5, 7)) → 1'b1` collapse
  rather than stopping at the Eq fold (the boundary it noted
  as an outstanding gap).

Docs:
- `book/src/structural-rules.md` Rule 21c `peephole` row
  expanded to list all five new constant-evaluation rules
  alongside the existing ones. "Constant evaluation
  (all-operand-constants → evaluated constant)" framing
  groups them cleanly.

**Why**

Closes a gap noted in the previous slice: `Not(Eq(c1, c2))`
rewrites via the comparison-inversion rule to `Neq(c1, c2)`,
which folds to a 1-bit constant. But `Not(already_folded_const)`
was left as a real Not gate because `Not(const)` wasn't wired.
Same pattern for any path where ConstantFold or a peephole
produces a constant that flows into a Not / Slice / reduction.

The slice generalises the existing all-const-comparison-fold
pattern to the remaining unary and unary-like gates, completing
the "constants flow through pure unary operators" story at
intern time. `Concat(all-const)` and `Shl/Shr(const, const)`
remain as known gaps — they're slightly more involved (width
accounting for Concat, shift-amount clamping for shifts) and
deferred to follow-ups.

**Empirical (seed 42, default knobs):**
- `peephole_rewrites_applied`: unchanged at 31 (none of the
  new patterns arise at default knobs on this seed — it takes
  a constant to flow directly into a Not/Slice/reduction,
  which is rare in current construction). The rules still fire
  in targeted unit tests and will activate whenever ConstantFold
  produces a constant that flows into one of these unary gates.
- Other metrics unchanged.

**Tests**
- 62 unit tests pass (was 59 — added 3 new + 1 upgraded).
- 22 integration tests pass (unchanged).
- Total test count: 84.
- `cargo build` clean; `mdbook build book` clean.

**Impact**
- Default-config output is unchanged at seed 42, but any
  module where a constant reaches a unary op via some
  sequence of folds now avoids the literal `~`, `[3:0]`, or
  `&`/`|`/`^` reduction wrapper around the constant in SV.
- Conceptual gap closed: the peephole layer now handles
  all-const evaluation for every operator class except Concat
  and Shl/Shr.

---

## 2026-04-17-0071 — Cross-gate peephole: Not(comparison) → inverted comparison

**What changed**

Extended `Module::apply_peephole`'s `Not` arm from a single-gate
rewrite (`Not(Not(x)) → x`) to cover six cross-gate comparison
inversions:

| Inner op | Rewrite      |
|----------|--------------|
| `Eq`     | `→ Neq`      |
| `Neq`    | `→ Eq`       |
| `Lt`     | `→ Ge`       |
| `Gt`     | `→ Le`       |
| `Le`     | `→ Gt`       |
| `Ge`     | `→ Lt`       |

When `intern_gate(Not, [cmp_gate_id], 1, deps)` sees its single
operand is a 1-bit comparison gate, it interns the inverted
comparison through the normal pipeline (CSE, constant fold,
etc.) and returns that NodeId directly. The original inner
comparison becomes orphaned (its only referencing call was the
outer `Not`, which collapsed); the post-construction
`compact_node_ids` pass cleans it up at module finalisation.

No new counters — fires share the existing
`peephole_rewrites_applied` counter.

Implementation detail: the `Not` arm of `apply_peephole` now
extracts `(op, operands, width, deps)` from the inner gate into
owned values before touching `self.intern_gate` recursively,
because holding an immutable borrow of `self.nodes[...]` across
a `&mut self` call would alias `self`.

Tests:
- `src/ir/types.rs`: three new unit tests —
  `peephole_not_eq_becomes_neq` (happy path),
  `peephole_not_comparison_inversions` (sweep over the five
  remaining rewrites), `peephole_not_eq_of_constants_folds_to_bit`
  (boundary: Not of a folded-to-const comparison stays as a Not
  on a constant — we don't wire Not-of-const into the pipeline
  here, that's ConstantFold's domain).

Docs:
- `book/src/structural-rules.md` Rule 21c `peephole` row
  expanded with the six inversions, kept in the same
  "orphan-safe via compaction" framing as the other peephole
  rules. Broader cross-gate rewrites like `(a + b) - b → a`
  remain flagged as e-graph work.

**Why**

First concrete step toward the `EGraph` ceiling (cross-gate
semantic equivalence). The inversions are narrow, unambiguous,
and rely on the same compaction infrastructure that enabled
`Not(Not(x))` in slice `2cd8b7a`. Picking this slice over broader
e-graph work keeps the deliverable well-scoped and empirically
measurable.

**Empirical (seed 42, default knobs):**
- `peephole_rewrites_applied`: 9 → **31** (+22 Not(cmp) fires)
- `nodes_compacted`: 94 → **96** (only +2 new orphans because
  comparison gates are usually shared via CSE and remain
  reachable from other consumers post-inversion)

**Tests**
- 59 unit tests pass (was 56 — added 3 new cross-gate tests).
- 22 integration tests pass (unchanged).
- Total test count: 81.
- `cargo build` clean; `mdbook build book` clean.

**Impact**
- Default-config output emits `neq`/`ge`/`le`/`gt`/`lt`
  operators directly where the RTL semantically calls for a
  negated comparison, instead of `!(eq/lt/...)`. Downstream
  tools see the canonical operator form.
- Infrastructure pattern established for future cross-gate
  peepholes: read inner gate into owned values, call
  `intern_gate` recursively, trust compaction for orphan
  cleanup.

---

## 2026-04-17-0070 — Associative flattening factorization layer goes live (Rule 21c layer 4)

**What changed**

Factorization ladder:
- `src/config.rs` promotes `FactorizationLevel::Associative`
  from aspirational to implemented. `is_implemented()` now
  covers it. `effective()` walker still handles out-of-order
  activation correctly; default `EGraph` walks down to
  `Peephole` (the highest rung) which transitively enables
  `Associative` and all lower layers.
- `src/ir/types.rs` adds `Module::flatten_associative`,
  dispatched from `intern_gate` **before** commutative sort
  and constant fold. For associative ops
  (`And`/`Or`/`Xor`/`Add`/`Mul`), it scans operands for any
  same-op same-width inner gate and splices its operands into
  the outer operand list. Per-op semantic normalisation after
  the splice:
  - **`And` / `Or`**: dedup (idempotent — `a & a = a`).
  - **`Xor`**: pair-cancel (self-inverse — `a ^ a = 0`). Count
    occurrences, drop even-count operands entirely, keep one
    copy of each odd-count operand.
  - **`Add` / `Mul`**: skip the flatten when flattening would
    produce duplicates AND `operand_duplication_rate < 1.0`.
    Preserves both the Rule 8 uniqueness contract and the
    `x + x = 2x` / `x * x = x²` semantics (dropping duplicates
    here would silently change arithmetic).

Short-circuits match the other intern-time helpers: post-
normalisation an empty operand list returns the op's identity
constant (only reachable for `Xor`-all-cancel → zero); a
single survivor returns that operand's NodeId directly; ≥ 2
operands overwrite the caller's operand list and intern
proceeds normally.

Live counter:
- `Module::flatten_associative_applied: u64` increments on
  each fire. Surfaced via `Metrics::flatten_associative_applied`.

Canary flipped:
- `tests/pipeline.rs`
  `nested_associative_opportunities_exist_today` (which
  previously asserted `> 0` to verify the layer hadn't landed
  yet) renamed to `nested_associative_opportunities_flatten_to_zero`
  and now asserts `== 0` at default knobs. This is the direct
  doctrine check that every post-construction IR is free of
  remaining associative-flattening opportunities. Complements
  `flatten_associative_applied` — the former is the
  post-construction state, the latter is the event count.

Tests:
- `src/ir/types.rs`: four new unit tests
  (`flatten_associative_splices_same_op`,
  `flatten_associative_and_dedups`,
  `flatten_associative_xor_pair_cancels`,
  `flatten_associative_xor_all_cancel_to_zero`,
  `flatten_associative_add_skips_on_duplicates`) covering the
  splice mechanics and per-op normalisation.

Docs:
- `book/src/structural-rules.md` Rule 21b: ladder prose +
  syntactic-vs-semantic framing now include `associative` as
  live, citing structural identities like
  `Add(a, Add(b, c)) = Add(a, b, c)` and `a ^ a = 0`.
- `book/src/structural-rules.md` Rule 21c: level table entry
  for `associative` promoted to a concrete description of the
  splice + per-op normalisation; "Doctrinal anchor" paragraph
  lists `associative` alongside the other implemented layers.
  `highest_implemented` prose updated: no more "skipping the
  not-yet-live associative rung".
- `book/src/non-triviality.md`: rewritten "NodeId compaction"
  paragraph to acknowledge its new role as enabler for
  associative flattening, followed by a dedicated paragraph on
  the Associative layer with the per-op semantics and the
  `nested_associative_operand_count = 0` empirical validation.
  The aspirational-layer list narrows to "cross-gate
  identities → e-graph".

**Why**

Layer 4 of the factorization ladder, enabled by the NodeId
compaction pass from the previous slice. Previously-deferred
because of the orphan-safety problem: splicing `Add(b, c)` into
`Add(a, ...)` leaves the inner `Add(b, c)` unreferenced. Now
compaction removes it at finalisation, so the rewrite is legal.

With this slice, the `NodeId = expression identity` doctrine
holds for every case where **syntactic identity after associative
normalisation** is sufficient: `Add(a, Add(b, c))`,
`Add(Add(a, b), c)`, and `Add(a, b, c)` all produce the same
NodeId at default knobs. The only residual divergence is for
semantically-equivalent-but-structurally-different expressions
(`(a + b) - b = a`, `(a & b) | (a & ~b) = a`, `a + 2 - 1 = a + 1`),
which are the e-graph / deeper peephole domain.

**Empirical (seed 42, default knobs):**
- `nested_associative_operand_count`: 0 (was 373 pre-slice)
- `flatten_associative_applied`: 268
- `nodes_compacted`: 94 (was 7 — jump driven by Associative
  orphaning inner gates at splice time)
- `fold_identities_applied`: 91 (was 28 — more ConstantFold
  opportunities opened up by flattening)
- `peephole_rewrites_applied`: 9 (unchanged)

**Tests**
- 56 unit tests pass (was 51 — added 4 new associative tests +
  a minor rewrite of the existing test cluster).
- 22 integration tests pass (unchanged in count; the canary
  test was renamed and its assertion flipped).
- Total test count: 78.
- `cargo build` clean; `mdbook build book` clean.

**Impact**
- Default-config output contains zero nested associative
  shapes: every `Add`/`Or`/`And`/`Xor`/`Mul` tree is fully
  flattened into its maximum-arity operator form. Downstream
  synthesis / formal tools see the canonical shape instead of
  random nesting.
- ConstantFold's identity-drop reach expands: flattened lists
  that happen to contain an identity constant now collapse
  correctly (previously the constant was hidden inside an
  inner gate).
- `factorization_level` dial is consistent: `associative` no
  longer silently degrades to `commutative` via the walker.
  Only `e-graph` (the theoretical ceiling) remains aspirational.

---

## 2026-04-17-0069 — NodeId compaction pass + Not(Not(x)) peephole unlock

**What changed**

New post-construction compaction pass: `src/ir/compact.rs` adds
`compact_node_ids(&mut Module) -> u32`. BFS from all roots
(drives, flop.d, flop.q, flop.mux.sel/data, flop.mux.arms) marks
reachable nodes; unreachable ones are removed and `m.nodes` is
rewritten with an old→new `NodeId` map. Every NodeId holder is
remapped in place: `m.nodes[*].operands`, `m.drives`, `flop.d`,
`flop.q`, `FlopMux::OneHot(arms)`, `FlopMux::Encoded { sel,
data }`. Dedup tables (`gate_instances`, `const_instances`)
are rebuilt under the new NodeId space — entries whose targets
were unreachable are dropped; surviving ones are remapped.
Topological order is preserved by walking old indices in
ascending order.

Integration:
- `src/gen/module.rs` `generate_leaf_module` calls
  `compact_node_ids` after `drain_flop_worklist`. Removed count
  stored on `Module::nodes_compacted`. Orphan-audit warning now
  fires only if compaction left orphans (indicating a BFS or
  holder-enumeration bug).

Peephole unlock:
- `Not(Not(x)) → x` re-enabled in `Module::apply_peephole`. The
  previous slice (`88c268d`) disabled it because it orphaned the
  inner `Not`. Compaction makes the rewrite safe — the inner gate
  is removed at module finalisation.

Metrics:
- `Module::nodes_compacted: u32` and `Metrics::nodes_compacted`
  surface the removed count. Zero when every rewrite happens to
  be orphan-safe; non-zero (seed-42: 7) when Not(Not) fires.

Tests:
- `src/ir/compact.rs`: 3 unit tests — no-op on clean IR, removes
  injected orphan gate, preserves topological order.
- `src/ir/types.rs`: reinstated
  `peephole_double_not_collapses_with_inner_orphaned` (asserts
  the inner Not is left in place at intern time; compaction is
  a separate concern).
- `tests/pipeline.rs`: new
  `compaction_preserves_rule_18_and_records_removals` — across
  40 seeds at default knobs, asserts (a) zero orphan gates
  post-compaction, (b) validator accepts post-compaction IR,
  (c) total `nodes_compacted > 0` (i.e. Not(Not) actually fires).

Docs:
- `book/src/structural-rules.md` Rule 21c: peephole row updated
  to include `Not(Not(x)) → x` with a note about the compaction
  pass. Cross-gate rewrites (`(a + b) - b → a`) still flagged
  as deferred.
- `book/src/non-triviality.md`: new paragraph describing the
  compaction pass, its role in enabling orphan-tolerant
  rewrites, and the path to Associative (which needs
  intern-time merge logic on top of compaction).

**Why**

Compaction is the architectural prerequisite for Associative
flattening (Layer 4) and the deferred `Not(Not(x))` peephole
rule. Landing it now — together with re-enabling Not(Not(x)) as
the first concrete consumer — keeps the slice tied to observable
output rather than being pure infrastructure. Associative stays
deferred for a follow-up (the intern-time merge logic is
independent work).

**Empirical (seed 42, default knobs):**
- `peephole_rewrites_applied`: 9 (was 2 before Not(Not) re-enable)
- `nodes_compacted`: 7 (→ 7 of 9 peephole fires were Not(Not);
  the other 2 are constant-comparison / Slice / Concat, which
  don't orphan)

**Tests**
- 51 unit tests pass (was 47 — added 3 compact tests + 1 restored
  peephole test).
- 22 integration tests pass (was 21 — added
  `compaction_preserves_rule_18_and_records_removals`).
- Total test count: 73.
- `cargo build` clean; `mdbook build book` clean.

**Impact**
- Rule 18 (zero orphans) holds post-finalisation, as before —
  but now via a construction-plus-compaction pipeline rather
  than construction-alone. Orphan-tolerant rewrites become
  legal.
- `Not(Not(x))` now collapses at intern time everywhere it
  arises via CSE chains — downstream output is slightly smaller
  and cleaner.
- Infrastructure in place for Associative flattening (next
  factorization-layer slice) and future cross-gate peephole
  rewrites that would otherwise leave intermediate gates
  orphaned.

---

## 2026-04-17-0068 — Per-knob probability-roll counters (attempts / fires) live

**What changed**

Live per-knob telemetry for every probability-roll knob. New
`KnobId` enum in `src/ir/types.rs` with one variant per
`gen_bool(cfg.<prob>)` site (10 total). New
`KnobRollCounters` struct on `Module` tracking
`attempts: HashMap<KnobId, u64>` and
`fires: HashMap<KnobId, u64>`. New helper `roll_knob(g, m,
knob, prob)` in `src/gen/cone.rs` replaces all 25
`gen_bool(cfg.<prob>)` sites; it rolls, records the outcome,
and returns the bool.

Knobs instrumented:
- `flop_prob`
- `comb_mux_prob`
- `priority_encoder_prob`
- `coefficient_prob`
- `const_shift_amount_prob`
- `const_comparand_prob`
- `comb_mux_encoding_prob`
- `flop_mux_encoding_prob`
- `share_prob`
- `flop_qfeedback_prob`

Surfaced via new `Metrics` fields:
- `knob_roll_attempts: BTreeMap<String, u64>`
- `knob_roll_fires: BTreeMap<String, u64>`
(converted from `KnobId` → canonical string name via
`KnobId::name()` at `compute()` time).

Tests:
- New integration test
  `knob_rolls_recorded_across_seeds` in `tests/pipeline.rs`:
  across 20 seeds at default knobs, every one of the 10
  expected knobs must appear in `knob_roll_attempts` with
  `attempts > 0` and `fires <= attempts`. Catches regressions
  where a knob stops being consulted or its roll site becomes
  unreachable.

Docs:
- `book/src/knobs.md` "Knob effectiveness map" gains a new
  "Per-knob roll-rate validation" subsection explaining the
  empirical-fire-rate test (`fires / attempts` should track
  the configured probability), with concrete seed-42 numbers.

**Why**

Completes the measurability doctrine for every probability
knob: the effect is now a simple division away. Previously
only *shape* metrics (`num_flops`, `num_muxes_2to1`, etc.)
measured these knobs — useful but indirect (they conflate the
roll rate with the number of reachable roll sites). The new
ratio `knob_roll_fires[k] / knob_roll_attempts[k]` is a
direct check.

Picked as the next slice because `Associative` and the
deeper peephole rules are both blocked on NodeId compaction —
a larger architectural slice. This slice is a clean
well-scoped completion of the measurability goal, with no
risk of orphan cascades.

**Empirical spot-check (seed 42, default knobs):**

| Knob                      | Default | attempts | fires | ratio |
|---------------------------|---------|----------|-------|-------|
| `share_prob`              | 0.30    | 1999     | 607   | 0.304 |
| `comb_mux_encoding_prob`  | 0.50    | 94       | 49    | 0.521 |
| `coefficient_prob`        | 0.20    | 256      | 51    | 0.199 |
| `const_shift_amount_prob` | 0.75    | 55       | 40    | 0.727 |
| `flop_qfeedback_prob`     | 0.50    | 34       | 15    | 0.441 |
| `comb_mux_prob`           | 0.10    | 1010     | 94    | 0.093 |
| `flop_prob`               | 0.10    | 261      | 34    | 0.130 |

All ratios track their configured values within sampling
noise — the telemetry is faithful.

**Tests**
- 47 unit tests pass (unchanged).
- 21 integration tests pass (was 20 — added
  `knob_rolls_recorded_across_seeds`).
- Total test count: 68.
- `cargo build` clean; `mdbook build book` clean.

**Impact**
- Every probability knob now has a direct empirical check
  on its effective rate — the measurability doctrine's
  strongest form.
- `manifest.json` / `--metrics` dumps gain two new maps per
  module, keyed by knob name. Consumers can build sweep
  scripts that assert `|empirical - configured| < ε` across
  many seeds and flag regressions.
- No behavioural change to the generator: `roll_knob` is
  byte-identical to the previous `g.rng.gen_bool(...)` on
  the output path, only adds the counter record. Verified
  by reproducibility tests (byte-identical output across
  `--trace` levels, which includes this change).

---

## 2026-04-17-0067 — Peephole factorization layer goes live (Rule 21c layer 6), orphan-safe subset

**What changed**

Factorization ladder:
- `src/config.rs` promotes `FactorizationLevel::Peephole` from
  aspirational to implemented. `is_implemented()` now covers
  `Peephole`; default `EGraph` walks down to `Peephole` as the
  effective layer (was `ConstantFold`).
- `src/ir/types.rs` adds `Module::apply_peephole`, dispatched
  from `intern_gate` after `fold_constants`. Rules wired —
  each one is narrow and orphan-safe:
  - **Fully-constant comparisons**: `Eq`/`Neq`/`Lt`/`Gt`/`Le`/
    `Ge` with both operands same-width constants are evaluated
    at intern time to a 1-bit constant. Constants orphaned as
    a side effect don't count as gate orphans, so the rule is
    Rule-18-safe.
  - **Full-width `Slice`**: `Slice(hi, 0)` with
    `hi + 1 == src_width` returns the source NodeId. The
    source is used by the caller — no orphan.
  - **Single-operand `Concat`**: `Concat([x]) → x`. Same
    orphan-safety.

Orphan-safety hardening in ConstantFold's absorbing rule:
- Previous slice's absorbing rule (`x * 0 → 0`, `x & 0 → 0`,
  `x | all_ones → all_ones`) would orphan any Gate operand —
  the outer gate collapses to a constant and the Gate
  operand's only consumer (this call) disappears. The peephole
  slice's RNG-path shift exposed the latent orphan: a pre-
  existing build-Eq-gate, later consumed by an `And([eq, 0])`
  which absorbed to 0, left the Eq unreferenced.
- Fix: absorbing now fires only when **no operand is a Gate**
  — i.e. every operand is a Constant, PrimaryInput, or FlopQ.
  This restricts absorbing to the "evaluate all-constant
  expression" subset, which is strictly orphan-safe. Dynamic
  absorbing (`x & 0` where `x` is a Gate sub-tree) now waits
  for the compaction-equipped future layer.

Rule NOT implemented and why:
- **`Not(Not(x)) → x`** — would orphan the inner `Not` gate
  because the outer `Not` call's only reference to inner
  (the operand) disappears after the rewrite returns `x`
  directly. Without NodeId compaction this is a Rule 18
  violation. Documented in `apply_peephole` doc comment +
  Rule 21c table. Waits for the e-graph / compaction layer.

Live counter:
- `Module::peephole_rewrites_applied` (u64) increments on
  each fire. Surfaced via `Metrics::peephole_rewrites_applied`.

Tests:
- `src/ir/types.rs`: four new peephole unit tests
  (`peephole_constant_comparison_evaluates`,
  `peephole_full_width_slice_identity`,
  `peephole_single_operand_concat_identity`,
  `peephole_disabled_below_peephole_level`). The previously-
  written `peephole_double_not_collapses` test was dropped
  alongside its rule.
- `tests/pipeline.rs`: new integration test
  `peephole_layer_fires_at_default_knobs` — sums
  `peephole_rewrites_applied` over 40 seeds at default knobs
  and asserts > 0.

Docs:
- `book/src/structural-rules.md` Rule 21b: factorization-
  ladder prose updated to list peephole as live; syntactic-
  vs-semantic framing extended with the peephole identities.
- `book/src/structural-rules.md` Rule 21c: level table
  entry for `peephole` promoted from "Not implemented yet"
  to the concrete rule list, with a note about why
  `Not(Not(x)) → x` is deferred. ConstantFold entry
  updated to note the non-Gate-operand restriction on
  absorbing.
- `book/src/structural-rules.md` Rule 21c: `highest_implemented`
  prose updated `constant-fold` → `peephole`.
- `book/src/non-triviality.md`: "Factorization ladder"
  subsection updated; aspirational layer list now just
  `Associative` + `EGraph`, both noted as blocked on
  NodeId compaction.

**Why**

Layer 6 of 8 on the factorization ladder, picked over the
harder `Associative` layer (Layer 4) because peephole rules
land as construction-time short-circuits with no NodeId
compaction required — same architectural shape as ConstantFold.
The effective-level walker I added in slice `82b2213` already
handles the out-of-order activation (layer 6 live, layer 4
still aspirational). Peephole advances the ladder and
exposed+fixed the absorbing-rule orphan hazard that was latent
in ConstantFold.

The absorbing-rule restriction is the doctrinally-correct
call: Rule 18 (zero orphans) is a strict invariant, and
until we have a compaction pass, absorbing on Gate operands
can't be made orphan-safe. The restricted form (absorb only
when all operands are non-Gate) is a proper subset that still
legitimately fires — just less often than the unrestricted
form.

**Tests**
- 47 unit tests pass (was 49 after the slice; two Not(Not)
  tests + one `double_not` test removed; four new peephole
  tests added; net +3 from the prior slice's +6).
- 20 integration tests pass (was 19 — added
  `peephole_layer_fires_at_default_knobs`).
- Total test count: 67.
- `cargo build` clean; `mdbook build book` clean.

**Impact**
- Default-config output contains fewer trivial gates:
  fully-constant comparisons are evaluated at intern
  time, full-width slices disappear, single-operand
  concats disappear.
- `Metrics::peephole_rewrites_applied` gives empirical
  visibility into Peephole-layer activity.
- The absorbing-rule restriction tightens the Rule 18
  guarantee: no construction-time rewrite can orphan a
  Gate. This pushes `Not(Not(x)) → x` and
  dynamic-absorbing (`x & 0` with gate `x`) into the
  compaction-equipped future layer, preserving the
  strict-orphan-free doctrine today.

---

## 2026-04-17-0066 — ConstantFold factorization layer goes live (Rule 21c layer 5)

**What changed**

Factorization ladder:
- `src/config.rs` promotes `FactorizationLevel::ConstantFold`
  from aspirational to implemented. `is_implemented()` and
  `effective()` now walk the enum order top-down and skip
  any not-yet-live middle rungs: a request for `Associative`
  correctly drops to `Commutative`, while `EGraph` / default
  activates up to `ConstantFold`. The enum-order quirk that
  `Associative` sits between `Commutative` and `ConstantFold`
  is handled by the walker, not by reshuffling variants.
- `src/ir/types.rs` adds the fold dispatcher
  `Module::fold_constants` (called from `intern_gate` after
  commutative sort, before dedup). Rules wired:
  - **Absorbing**: `x & 0 → 0`, `x | all_ones → all_ones`,
    `x * 0 → 0` (returns a same-width constant via
    `intern_constant`).
  - **Identity drop**: `Add`/`Xor`/`Or` drop `0` operands,
    `Mul` drops `1` operands, `And` drops `all_ones`
    operands. Post-shrink: 0 operands → identity constant,
    1 operand → that operand's NodeId, ≥ 2 → caller
    continues with the shrunken list.
  - **2-arity Sub / Shl / Shr**: rhs-zero short-circuit
    (`a - 0 → a`, `a << 0 → a`, `a >> 0 → a`). The lhs-zero
    cases (`0 - a`, `0 << a`, `0 >> a`) are deliberately not
    folded — they're not algebraic identities.
  Comparison ops, reductions, `Not`, `Slice`, `Concat`, `Mux`
  are out of scope for this layer (they belong to `Peephole`).

Live counter:
- `Module::fold_identities_applied` (u64) increments on each
  fire. Surfaced via `Metrics::fold_identities_applied`,
  sourced directly from the per-module counter.

Pre-existing bugs exposed by fold and fixed in the same slice:

- `assemble_mul_linear_combination` didn't dedup the coefficient
  constant against its signal list — when coef == const_k and a
  signal happened to be the literal const_k (same NodeId via
  CSE), operands became `[c, c]`, tripping Rule 8 operand
  uniqueness. Fixed with a post-assembly dedup pass; single-
  operand residual returns directly.
- `make_mul` / `make_sub` lacked the `a == b` degeneracy guard
  that `make_and` already had. When CSE / fold collapsed two
  callers' ids into one, `make_mul(a, a)` hit the same
  duplicate-operand failure as above. Added guards mirroring
  `make_and`: `make_mul` short-circuits to `a` under strict
  operand-uniqueness; `make_sub` short-circuits to a zero
  constant (Sub is algebraically `x - x = 0`).
- `deliver`'s interleaved anti-collapse fallback used
  `operands[0]` as fallback for all ops, which works for
  gates whose operand width equals output width but BREAKS for
  comparisons: `Eq`/`Neq` output 1-bit but operand width is
  the comparand width K. When `violates_anti_collapse`
  flagged `Eq(a, a)` or `Neq(a, a)` during interleaved
  construction, delivering `operands[0]` (width K) into a
  slot expecting width-1 (the comparison output) yielded
  mismatched operand widths in the parent. Fixed with
  comparison-specific width-correct fallbacks:
  `Eq(a, a) → 1`, `Neq(a, a) → 0`. Mux, Sub, And/Or/Xor/Add/Mul
  cases unchanged since they already had the correct width.

Tests:
- `src/ir/types.rs`: five new unit tests covering fold
  identities (`fold_add_zero_collapses_to_x`,
  `fold_and_all_ones_collapses_to_x`,
  `fold_mul_zero_absorbs`, `fold_or_all_ones_absorbs`,
  `fold_miscellaneous_identities`) and a gating test
  (`fold_disabled_below_constant_fold_level`) that confirms
  the layer is inert at `FactorizationLevel::Commutative`.
- `tests/pipeline.rs`: new integration test
  `constant_fold_layer_fires_at_default_knobs` sums
  `fold_identities_applied` over 40 seeds at default knobs
  and asserts > 0 — a regression guard against the fold layer
  silently switching off (or `constant_prob` no longer
  producing identity-valued constants).

Docs:
- `book/src/structural-rules.md` Rule 21b: factorization-
  ladder prose updated to list constant-folding as live;
  syntactic-vs-semantic framing extended to cite the curated
  identities now caught at intern time.
- `book/src/structural-rules.md` Rule 21c: level table entry
  for `constant-fold` promoted from "Not implemented yet" to
  the concrete identity list, with a pointer to
  `Metrics::fold_identities_applied` for empirical
  measurement. Effective-level prose rewritten to document
  the walker semantics.
- `book/src/non-triviality.md` "Factorization ladder"
  paragraph: constant-folding added to the within-gate
  surface; aspirational layer list slimmed to
  `Associative`/`Peephole`/`EGraph`.

**Why**

Next rung on the factorization ladder (Layer 5 of 8). Picked
over `Associative` (Layer 4) because it's strictly simpler —
no NodeId compaction, no finalization pass, no dedup-table
rebuild — while still advancing the ladder and surfacing
latent bugs in adjacent code (the three fixed above). The
`Associative` rung stays aspirational for now with its
regression canary (`nested_associative_opportunities_exist_today`)
still in place; when that layer lands the canary flips to
`== 0`.

**Tests**
- 19 integration tests pass (was 18 — added
  `constant_fold_layer_fires_at_default_knobs`).
- 49 unit tests pass (was 43 — added 6 fold tests).
- Total test count: 68.
- `cargo build` clean; no warnings introduced.

**Impact**
- Default-config output contains fewer trivial-algebraic
  gates. Specifically: `x + 0`, `x * 1`, `x & all_ones`, and
  `x | 0` now disappear at intern time rather than landing
  as literal nodes. Downstream synthesis tools would fold
  these anyway; anvil now matches their view one step
  earlier.
- `Metrics::fold_identities_applied` exposes an empirical
  handle on how much work the fold layer does per seed /
  per module — useful for knob tuning (does
  `constant_prob` produce enough identity-valued literals
  to make fold meaningful? turns out yes at default).
- Three latent bugs in adjacent code paths (linear-comb
  dedup, make_mul degeneracy, comparison anti-collapse
  fallback width) landed fixes while I was there, each
  defensive against RNG-path shifts the fold layer
  introduced.

---

## 2026-04-17-0065 — Syntactic-vs-semantic-identity framing in the factorization-ladder narrative (docs only)

**What changed**
- `book/src/structural-rules.md` (Rule 21b, the "Position in the
  factorization ladder" paragraph): new follow-up paragraph
  making the syntactic-vs-semantic identity distinction
  explicit. What today's implemented layers guarantee is that
  **two syntactically identical expressions share one node**.
  The aspirational layers above extend the contract toward
  **two semantically equivalent expressions share one node** — a
  strictly harder problem that synthesis tools themselves solve
  incompletely.
- `book/src/non-triviality.md` (the "Factorization ladder"
  sub-section of "Algebraic residue"): same framing mirrored,
  tied to the local narrative about what anti-collapse rules
  catch and what they don't.

**Why**
A durable framing surfaced in the conversation: the contract
we actually ship today is *syntactic* identity; the goal is
*semantic* identity; the asymptote matters because synthesis
tools themselves solve semantic equivalence incompletely.
Recording the framing in the book makes it survive session loss
and sets reader expectations appropriately — neither overclaim
nor underclaim what the `NodeId = expression identity` doctrine
delivers in the current build.

**Tests**
- No code changed.
- 57 tests pass.
- `mdbook build book` succeeds.

**Impact**
- Readers learning about the factorization ladder now have a
  single-sentence summary of where we are vs where we aim, and
  an honest acknowledgment that the ceiling is an asymptote
  the synthesis industry itself hasn't closed.

---

## 2026-04-17-0064 — Regression tests pinning three doctrine-level invariants

**What changed**
- `tests/pipeline.rs` gains three integration tests:
  - **`zero_orphans_at_default_knobs`** — Rule 18 regression
    guard. Generates modules across all four strategy values ×
    6 seeds and asserts every `Node::Gate` has at least one
    consumer (gate operand, flop field, or output drive).
  - **`zero_duplicate_operands_at_default_knobs`** — Rule 8
    extended regression guard. At `operand_duplication_rate =
    0.0` (default), no `And`/`Or`/`Xor`/`Add`/`Mul` gate may
    have a duplicate `NodeId` in its operand list. Checked
    across 5 seeds.
  - **`nested_associative_opportunities_exist_today`** —
    informational guard. Asserts
    `nested_associative_operand_count > 0` at seed 42 today
    (Associative layer not implemented). When that layer lands,
    this test should flip to `== 0` as direct validation that
    flattening collapses the opportunity.
- `CODEBASE_ANALYSIS.md`: test count updated 54 → 57.

**Why**
Each of the three assertions captures a doctrine-level
invariant established in recent slices but not pinned by a
test:

- Rule 18 zero-orphans — enforced by build_cone
  snapshot/rollback + process_signal_frame existing-operand
  fallback. Slice `b78550d` validated manually across
  strategies × seeds; now a test catches regressions
  automatically.
- Rule 8 zero-duplicates at default — enforced by
  `violates_anti_collapse` + the post-assemble dedup in
  linear-combination + `make_and` idempotent short-circuit.
  Slice `9e18c89` drove the duplicate count to 0 at default;
  now regression-guarded.
- Associative-opportunity non-zero — direct complement to the
  metric added in `99084a8`. Serves as a canary: when the
  Associative layer lands and flips this to zero, the
  implementation is working.

**Tests**
- All four cargo gates green.
- **57 tests** pass (39 unit + 18 integration, +3 new).

**Impact**
- Future slices that break Rule 18 or the operand-uniqueness
  contract now fail CI instead of being spotted by manual
  `grep` audit.
- The associative-flattening regression test flipping direction
  is a simple, definite signal that the Associative layer has
  landed and works.

---

## 2026-04-17-0063 — Associative-flattening opportunity metric (informational, pre-implementation)

**What changed**
- `src/metrics.rs`: new `Metrics::nested_associative_operand_count:
  usize`. Post-hoc walk counts every operand slot on an associative
  gate (`And`/`Or`/`Xor`/`Add`/`Mul`) whose operand is itself a
  `Node::Gate` of the same op and width — i.e., a slot the
  not-yet-implemented `Associative` factorization layer would
  absorb.
- `book/src/knobs.md`: knob-effectiveness map gains an entry for
  `operand_duplication_rate` (previously missing) and extends the
  `factorization_level` entry with the new metric.
- `USER_GUIDE.md`: knob-effects bullet list gains an entry for
  the new metric.
- `CODEBASE_ANALYSIS.md`: `metrics.rs` one-liner extended.

**Why**
The factorization ladder has three implemented layers (CSE,
operand-uniqueness, commutative) and four aspirational ones
(Associative, ConstantFold, Peephole, EGraph). Before investing
in the full `Associative` implementation — which involves
non-trivial design (finalization pass vs construction-time;
NodeId compaction vs leaving orphans; pool coordination) — this
slice measures *how much flattening would actually happen*, so
the cost/benefit is data-driven rather than speculative.

**Tests**
- All four cargo gates green.
- 54 tests pass.
- `mdbook build book` succeeds.
- Seed sweep at default knobs:

  ```
  seed=1     num_gates=1999 nested_associative_operand_count=261 (13%)
  seed=42    num_gates=2368 nested_associative_operand_count=373 (16%)
  seed=100   num_gates=2311 nested_associative_operand_count=266 (12%)
  seed=777   num_gates=2861 nested_associative_operand_count=386 (13%)
  seed=9999  num_gates=20   nested_associative_operand_count=1   (5%)
  ```

  **10–16% of operand slots on associative gates would be
  absorbed by flattening.** Meaningful reduction target; the
  Associative slice is worth queuing.

**Impact**
- No behaviour change.
- Factorization-level effectiveness-map entry goes from
  qualitative ("`num_gates` shift across dial") to quantitative
  (concrete opportunity count).
- Data to justify (or postpone) the full Associative
  implementation.

---

## 2026-04-17-0062 — FAQ chapter refresh: strategies + full-factorization Q (docs only)

**What changed**
- `book/src/faq.md`:
  - "Why four construction strategies instead of just the default?"
    → "Why three". graph-first removed from the canonical list,
    retirement rationale + silent-alias behaviour noted with
    cross-link to the construction-strategies chapter. Interleaved
    described as the default.
  - "Can output J's cone reference a gate from output I's cone?"
    — stale `graph-first`-specific language replaced; added a
    mention that Rule 21 CSE makes the cross-cone identity
    automatic.
  - New entry: **"What does 'full factorization' mean in the
    book? Does `anvil` deduplicate expressions?"** Answers the
    user doctrine. Names the three implemented layers (CSE,
    operand uniqueness, commutative normalization) and the four
    aspirational layers (Associative, ConstantFold, Peephole,
    EGraph), with the `factorization_level` dial.

**Why**
FAQ chapter predated `graph-first` retirement (`b78550d`) and
the factorization-ladder work (`f425657`, `c9c2f98`, `d2aefba`,
`5a9b477`). A user landing on the FAQ first now sees the correct
strategy story and a direct answer to the "does anvil dedupe?"
question that the full-factorization doctrine prompts.

**Tests**
- No code changed.
- 54 tests pass.
- `mdbook build book` succeeds.

**Impact**
- Closes the book audit. Every authored chapter now reflects
  shipping code. A session that recovers cold from
  `SESSION_BOOTSTRAP.md`'s reading order won't find drift
  between the book's narrative and the generator's behaviour.

**Book audit completion status**

| Chapter | Status |
|---|---|
| introduction, getting-started, tutorial, recipes, knobs, construction-strategies, ir, algorithm, sequential, synthesizability, structural-rules, architecture, by-construction, non-triviality, sharing, faq | **Fresh** |
| hierarchy.md | Phase 4+ — intentionally placeholder |
| core-idea, non-goals, why-not-grammar | Doctrine — not casually edited |

---

## 2026-04-17-0061 — Sharing chapter refresh: Rule 2 + Rule 18 + Rule 21 CSE (docs only)

**What changed**
- `book/src/sharing.md`:
  - `try_share` description: removed the stale "Q-exclusion
    contract" reference; replaced with a pointer to Rule 2
    (Q-feedback freedom — Q is a free leaf inside its own
    D-cone; the clock edge breaks the Q→D loop temporally).
  - "Forbidden sharing patterns" section rewritten to match the
    current Rule 8 extended rule set: N-arity And/Or/Xor
    operand-multiset distinctness, 2-arity Sub/Eq/Neq
    degeneracy, Add/Mul gated on `operand_duplication_rate`,
    Mux gated on `mux_arm_duplication_rate`. Added a paragraph
    on the Rule 18 α snapshot-restore on rejection — rejected
    sub-trees don't orphan.
  - **New "Construction-time CSE (Rule 21)" section** replaces
    the old "What sharing does not do" paragraph. The old text
    said "does not deduplicate equivalent sub-expressions… CSE
    is the synthesizer's job" — this was reversed by slice
    `f425657`. New section explains that `intern_gate` dedupes
    by `(op, operands, width)`, with `max_ast_instances` cap
    knob, commutative sort at level ≥ `Commutative`, and
    articulates how per-operand `share_prob` and CSE compose
    (share_prob = early cut-off; CSE = identity of identical
    expressions).
  - Cross-output sharing section: "current sequential" +
    "graph-first will be the default" corrected. `interleaved`
    is default; `graph-first` retired as silent alias.
  - "No cycles possible" section retitled "No combinational
    cycles possible"; removed the stale Q-exclusion reference;
    added Rule 1 + Rule 2 cross-links and the explicit
    clock-edge-breaks-the-loop-temporally story.

**Why**
Per book doctrine. `sharing.md` predated four big changes:
- Rule 2 Q-feedback freedom (slice `6cbcbff`).
- Rule 8 extension (`3544a0c`).
- Rule 18 α enforcement (`b78550d`).
- Rule 21 CSE via intern_gate (`f425657`) + Rule 21b commutative
  normalization (`c9c2f98`).

The chapter's previous "sharing does not CSE" paragraph actively
contradicted the shipping code — the most misleading kind of
drift.

**Tests**
- No code changed.
- 54 tests pass.
- `mdbook build book` succeeds.

**Impact**
- Readers learning DAG sharing now see the full story:
  per-operand share/recurse coin **plus** construction-time CSE
  **plus** commutative normalization, and how the three compose.
- No more stale "Q-exclusion" references in the book (verified;
  this chapter was the last holdout).

---

## 2026-04-17-0060 — Non-triviality chapter: anti-collapse rule table + factorization-ladder framing (docs only)

**What changed**
- `book/src/non-triviality.md`:
  - Anti-collapse rules table (the heart of the chapter)
    rewritten to match actual `violates_anti_collapse` in
    `src/gen/cone.rs`. Old table listed rules that were never
    implemented (`a & 0`, `a | all_ones`, `a & all_ones`,
    `a | 0`, "minimum shift amount = 1") and missed the
    current reality (N-arity operand-multiset distinctness,
    `operand_duplication_rate` / `mux_arm_duplication_rate`
    gating). New table has five rows covering the actual
    implementation plus a paragraph on the factorization-level
    gating (rules relax at level `cse` / `none`).
  - New snapshot-restore note under the table: explains the
    Rule 18 α connection — on anti-collapse rejection,
    `build_cone` rolls back its pre-operand-construction
    snapshot so the rejected sub-tree doesn't orphan.
  - "Algebraic residue" section reframed. Old text: "the fix is
    to add a cheap canonicalizer". New text: "anvil has started
    climbing this ladder" — points at CSE / operand-uniqueness /
    commutative landed, and notes the four aspirational
    FactorizationLevel layers (Associative / ConstantFold /
    Peephole / EGraph) still to implement. Cross-links to
    Rule 21c and DEVELOPMENT_NOTES.

**Why**
Per book doctrine. `non-triviality.md` predated:
- Rule 8 extension (slice `3544a0c`) — N-arity duplicate check.
- Rule 18 α enforcement (`b78550d`) — snapshot/rollback.
- Rule 21 CSE (`f425657`).
- Rule 21b commutative normalization (`c9c2f98`).
- Rule 21c factorization dial (`c9c2f98`).
- Rule 22 mux-arm duplication knob (`d2aefba`).
- `operand_duplication_rate` knob (`5a9b477`).

Most of what the chapter described as "future canonicalizer" has
now landed — the framing was stale.

**Tests**
- No code changed.
- 54 tests pass.
- `mdbook build book` succeeds.

**Impact**
- Readers learning about anti-collapse see the current enforcement
  surface and its knob-driven relaxations, not the
  never-implemented aspirational table.
- The factorization-ladder narrative ties this chapter to
  `structural-rules.md` Rule 21c and the knobs chapter's dial,
  reinforcing the single doctrine across the book.

---

## 2026-04-17-0059 — By-construction chapter: validator tense + Rule 18 exemplar + retry grandfather clause (docs only)

**What changed**
- `book/src/by-construction.md`:
  - "Validator is a safety net" section: tense correction. Was
    "`anvil` will likely include an IR validator" — factually
    stale since `src/ir/validate.rs` has shipped with an inline
    test suite covering every rejection class. Now stated as
    present-tense reality with a note on the CI failure-conversion.
  - New sub-section "Exemplar: Rule 18 (no orphan gates)" — the
    cleanest illustration of by-construction discipline in action.
    Records the α vs β decision (β rejected as generate-then-filter),
    names the mechanism (build_cone snapshot/rollback +
    process_signal_frame existing-operand fallback), and cites the
    current empirical result (0 orphans across 4 strategies × 6
    seeds).
  - New sub-section "Grandfather clause: bounded retry" — makes
    explicit that the *one* retry-and-discard pattern in the
    generator (`build_cone_with_retry` on empty-dep cone roots) is
    bounded, snapshots state between attempts, and differs from
    "generate-then-filter" in that the rejected attempt leaves
    zero trace in the IR. Any other retry-and-filter pattern would
    be a design regression.

**Why**
by-construction.md is a doctrinal chapter (not on the don't-touch
list — that's core-idea / non-goals / why-not-grammar). Current
text predated Rule 18 α enforcement (slice `b78550d`), the CSE
snapshot-table fix (`f425657`), and the validator's actual
shipping; adding the Rule 18 exemplar strengthens the thesis
rather than changing it, and the tense/grandfather-clause edits
are factual corrections.

**Tests**
- No code changed.
- 54 tests pass.
- `mdbook build book` succeeds.

**Impact**
- A reader meeting the by-construction doctrine for the first
  time now sees both the principle and the freshest concrete
  example of the principle being applied.
- The grandfather-clause explicit statement makes the one
  "retry-and-discard" pattern in the codebase (bounded retry
  of empty-dep cone roots) unambiguously doctrine-compliant,
  which forestalls future arguments for relaxing the rule.

---

## 2026-04-17-0058 — Architecture chapter refresh: align with current workspace reality (docs only)

**What changed**
- `book/src/architecture.md`:
  - Crate layout: added `src/metrics.rs` (missing since slice
    `6fb5b9b`). Extended descriptions of `main.rs` (tracing wire-
    up), `lib.rs` (TRACE_DEBUG + trace_verbose! macro), `config.rs`
    (ConstructionStrategy + FactorizationLevel enums), `ir/types.rs`
    (intern_gate / intern_constant API + dedup tables + per-module
    knob mirrors + block-build counters), `gen/cone.rs` (motif
    dispatch, snapshot/rollback, terminal picker variants, dup-cap
    helpers), `gen/module.rs` (orphan audit), `emit/sv.rs`
    (dumb-serialiser doctrine + typed naming).
  - "Key types at a glance" rewritten:
    - `Module` struct showing dedup tables, per-module knob
      mirrors, block-build counters, and the `intern_gate` /
      `intern_constant` method signatures.
    - `GateOp` enum expanded with all variants (was
      `..., Mux, Slice{..}, ...`).
    - Added `ConstructionStrategy` and `FactorizationLevel` enums.
    - Added `metrics::Metrics` + `compute` signatures.
    - Added `lib.rs` trace infrastructure.
  - Testing strategy: per-file unit counts updated (cone.rs 7→13,
    types.rs 0→2, added metrics.rs 3 tests). Integration count
    2→15. Total 23→54.
  - CLI section: old 15-flag listing replaced with a pointer to
    `knobs.md`'s categorised CLI-coverage section and a note that
    `anvil --help` is canonical. Eliminates duplication and
    drift risk.

**Why**
Per book doctrine (up-to-date). `architecture.md` predated 10+
src-touching slices: metrics module, intern_gate API, dedup
tables, FactorizationLevel, ConstructionStrategy enum, trace
infrastructure, most cone.rs helpers, every new test, every new
CLI flag. CODEBASE_ANALYSIS.md was already updated for these
(slice `c0ba963`) but the book's mirror chapter had drifted.

**Tests**
- No code changed.
- 54 tests pass.
- `mdbook build book` succeeds.

**Impact**
- The architecture chapter now accurately describes the Rust
  workspace at HEAD. A cold reader gets the same picture from
  `book/src/architecture.md` as from `CODEBASE_ANALYSIS.md`.
- CLI is canonicalised to one place (`knobs.md`) instead of
  being duplicated in three (README, USER_GUIDE, knobs.md,
  architecture.md). Drift risk reduced.

---

## 2026-04-17-0057 — Algorithm chapter refresh: strategies, Rule 2, Rule 18, CSE, motif dispatch (docs only)

**What changed**
- `book/src/algorithm.md`:
  - Strategy note rewritten. Was: "current `sequential`; three
    *planned* alternatives." Now: all three strategies landed;
    default is `interleaved`; `graph-first` retired as a silent
    alias (pointer to the construction-strategies chapter's
    retirement rationale).
  - `build_cone` pseudocode expanded:
    - New branches for `priority_encoder_prob`, linear-combination
      motif (`coefficient_prob`), const-shift motif
      (`const_shift_amount_prob`), const-comparand motif
      (`const_comparand_prob`). These were all implemented but
      missing from the pseudocode.
    - Added the snapshot/rollback around operand construction
      (Rule 18 α enforcement). Rejected gates restore state so
      operand sub-trees don't orphan.
    - Final node creation goes through `intern_gate` (Rules 21 +
      21b — CSE + commutative normalization). Pool add gated on
      `is_new`.
  - Flop-drain pseudocode: corrected `exclude = Some(q_node)`
    (old "Q-exclusion contract") to `exclude = None` with a
    comment pointing to Rule 2 (Q-feedback freedom). The
    Q-exclusion was relaxed in slice `6cbcbff`.
  - Retry-loop section: now mentions that the snapshot also
    restores `gate_instances` / `const_instances` — the CSE
    dedup tables — and explains the failure mode when they are
    not (pointer to `DEVELOPMENT_NOTES.md`).
  - Anti-collapse section: old 5-line rule list replaced with
    the full current set:
    - Idempotent N-arity (And/Or/Xor) multiset-distinctness.
    - 2-arity algebraic degeneracy (Sub/Eq/Neq).
    - Mux duplicate-arm (gated on `mux_arm_duplication_rate`).
    - Add/Mul duplicate (gated on `operand_duplication_rate`).
    - `factorization_level`-dependent relaxation (cse / none).
    - Note that rejection restores snapshot (Rule 18).
    - Pointer to the factorization ladder for future layers.

**Why**
Audit found the algorithm chapter predated every major
2026-04-15 → 2026-04-17 slice touching `build_cone`:
snapshot/rollback (`b78550d`), CSE via intern_gate (`f425657`),
commutative normalization (`c9c2f98`), motif dispatch (already
landed when the chapter was written but only partially
described), and the Rule 2 Q-feedback relaxation (`6cbcbff`).

**Tests**
- No code changed.
- 54 tests pass.
- `mdbook build book` succeeds.

**Impact**
- The algorithm chapter is now a faithful pseudocode
  transcription of `src/gen/cone.rs` as it stands today. A
  reader can hold them side-by-side without discovering
  discrepancies.
- The anti-collapse section now reflects the
  `factorization_level` dial and the three duplication-rate
  knobs — ties the chapter back into the knob catalog.

---

## 2026-04-17-0056 — Book audit: last `w_N`/`r_N` naming remnants (docs only)

**What changed**
- `book/src/introduction.md`: five-minute pitch replaced. The old
  snippet was hand-written with `w_2 / w_3 / w_4` names that
  never matched current output even under the old opaque scheme.
  New snippet is a real seed-20 output: 23 lines, a single
  `flop_0` hold-register with canonical `always_ff` block,
  showing Rule 12 naming (`flop_<id>`) in action. Added a brief
  paragraph pointing at the `<kind>_<N>` / `flop_<id>`
  convention.
- `book/src/sequential.md`: clock-and-reset SV snippet refreshed
  from `r_0 <= 8'h0` / `r_0 <= w_42` to `flop_0 <= 8'h0` /
  `flop_0 <= add_3`. Added a parenthetical pointing to Rule 12
  for the naming scheme.
- `book/src/synthesizability.md`: canonical flop template's
  `r_0` → `flop_0`. Also corrected an aspirational footnote
  ("or the sync-reset variant, or the no-reset variant, chosen
  per flop at generation time") — this never shipped. Replaced
  with the actual discipline per Rule 5 (single-clock /
  single-reset, async active-low, one `always_ff` block per
  module).

**Why**
Grep of `book/src/` for `\bw_[0-9]+|\br_[0-9]+` (the retired
opaque-naming pattern) found three remaining files. After this
slice, the only remaining match is `w_0 … w_47` in Rule 12's
motivation paragraph where the old naming is deliberately
contrasted with the current scheme — intentional.

**Tests**
- No code changed.
- 54 tests pass.
- `mdbook build book` succeeds.

**Impact**
- Book's flop-related SV excerpts now all match current output.
- The `synthesizability.md` correction removes a misleading
  aspirational claim (sync-reset / no-reset variants) and
  aligns with the actual Rule 5 discipline.

---

## 2026-04-17-0055 — Construction-strategies chapter: graph-first retirement + interleaved-as-default (docs only)

**What changed**
- `book/src/construction-strategies.md`:
  - Lede rewritten: "three strategies" (was four); default is
    `interleaved` (was `graph-first`); graph-first noted as a
    silent alias.
  - `sequential` section: `*(current behavior)*` subtitle
    dropped (no longer true).
  - `interleaved` section: `*(default)*` subtitle added.
  - `graph-first` dedicated section deleted and replaced with a
    "Retired" section that explains *why* (Rule 18 orphan
    violation, pointer to slice `b78550d`), *why not just fix
    graph-first* (the demand-driven version IS interleaved),
    and *what to use instead*.
  - Comparison table: graph-first row removed; `interleaved`
    marked as default.
  - Interaction-with-rules section: updated Rule 9 bullet
    (unified path via `build_cone_with_retry`); Rule 16
    reworded ("strongest in interleaved"); **new Rule 18
    bullet** making the zero-orphan construction contract
    explicit and noting the snapshot/rollback + existing-
    operand-fallback mechanics.
  - Implementation status: graph-first marked retired; silent-
    alias behavior documented; historical-reproducibility note
    pointing to pre-`b78550d` commits.

**Why**
User doctrine: the book must be up-to-date. Audit found
`construction-strategies.md` still described a four-strategy
lineup with graph-first as default and lauded as "the most
realistic shared-DAG output" — but graph-first was retired for
producing 13–27 % orphan gates per module (slice `b78550d`)
and is now a silent alias for interleaved.

**Tests**
- No code changed.
- 54 tests pass.
- `mdbook build book` succeeds.

**Impact**
- Readers coming in cold see the actual strategy surface, the
  retirement rationale (which is also a user-memorable lesson
  about the "by construction" doctrine), and the clear
  migration guidance.
- `--construction-strategy graph-first` still works, so
  existing configs / scripts are unaffected.

---

## 2026-04-17-0054 — Tutorial chapter refresh: naming scheme + re-captured examples (docs only)

**What changed**
- `book/src/tutorial.md`:
  - New "Naming convention" lede paragraph introducing the
    `<kind>_<N>` / `flop_<N>` scheme up front with a pointer to
    Rule 12. New readers now know what `and_5`, `slice_2`,
    `flop_1` mean before encountering them.
  - Example 2's prose updated: `w_N` → `<kind>_N`.
  - Example 4 (direct-D flop): sample SV re-captured from
    current generator output — `r_0` → `flop_0`, `w_3 = a + a`
    → `shl_0 = flop_0 << 1'h0` (the seed-5 run now produces a
    shift, not an add; structural point unchanged). New bullet
    explaining the shift-by-zero is a structural-not-meaningful
    quirk.
  - Example 5 (one-hot D mux): sample lines re-captured with
    current typed naming and the canonical `{W{sel}} & data`
    pattern annotated. Note added that CSE + limited-pool can
    produce richer actual output than the illustrative excerpt.
    Replication-syntax callout (`{8{slice_0}}` vs expanded list)
    added.
  - Example 6 (encoded D mux): re-captured verbatim from seed 11
    output. Shows the full `slice_0` / `eq_0` / `mux_0` /
    `eq_1` / `mux_1` / `always_ff` structure; bottom-up read
    explanation updated.
  - Example 8 (sharing): `w_N` → `<kind>_N` in the prose.
  - Example 9 (comb-mux Encoded): re-captured. Shows the 3-arm
    chained ternary with `slice_0` / `slice_1` as sel / data
    and the `2'h0` fall-through; bottom-up read added.

**Why**
User directive: the book must be up-to-date with actual output.
Audit found every SV excerpt in the Tutorial chapter used the
retired `w_N` / `r_N` naming scheme (superseded by Rule 12
typed-per-kind naming in slice `26f90a3`). Prose in
Examples 2 and 8 also still used `w_N`.

**Tests**
- No code changed.
- 54 tests pass.
- `mdbook build book` succeeds.
- Every sample SV excerpt was re-captured by running the
  example's exact `cargo run` command against HEAD.

**Impact**
- Tutorial now faithful to shipping output. A reader following
  the chapter step-by-step sees the commands and their real
  output, not a historical snapshot.
- Book doctrine (up-to-date / example-heavy / not scary /
  progressive) reinforced.

---

## 2026-04-16-0053 — Knobs chapter alignment with actual config + CLI surface (docs only)

**What changed**
- `book/src/knobs.md`:
  - Quick Reference table: added missing `--operand-duplication-rate`
    row; fixed stale `construction-strategy` default
    (`graph-first` → `interleaved`); fixed `--trace` default
    (`off` → `none`).
  - Knob catalog body: added the missing `operand_duplication_rate`
    entry in the Factorization sub-section (it had landed in slice
    `5a9b477` but was never documented in the book).
  - Defaults block: refreshed to mirror `Config::default()`
    exactly — ~20 knobs were previously missing (`min_gate_arity`,
    `max_gate_arity`, `coefficient_prob`, `min/max_coefficient`,
    `const_shift_amount_prob`, `min/max_shift_amount`,
    `gate_shift_weight`, `const_comparand_prob`, `min/max_comparand`,
    `priority_encoder_prob`, `comb_mux_prob`,
    `comb_mux_encoding_prob`, `construction_strategy`,
    `graph_first_pool_size`, `factorization_level`,
    `max_ast_instances`, `mux_arm_duplication_rate`,
    `operand_duplication_rate`).
  - CLI coverage section: rewritten. Old list covered only
    structural + sequential + share knobs; now 44 flags
    organised by category (Run control / Structure /
    Sequential / Sharing / Operator arity / Coefficient /
    Shift / Comparand / Blocks / Construction strategy /
    Factorization). Explicit list of knobs NOT yet exposed
    via CLI.

**Why**
User directive: *"Make sure these knobs are thoroughly
documented in the book too."* Audit showed the
`operand_duplication_rate` knob (landed in `5a9b477`) was not
in the book catalog; the defaults block had ~20 missing
entries; the CLI coverage block listed only the original Phase
1 subset.

**Validation**
A script that grep-extracts every `--flag` mention from the
book's CLI-coverage section and compares against `anvil --help`
reports:
```
flags named in book:  44
flags in --help:      45
book-only (broken):   []
help-only (undoc'd):  ['--version']   ← clap boilerplate, expected
```

**Tests**
- No code changed.
- 54 tests pass.
- `mdbook build book` succeeds.

**Impact**
- Book's knob catalog now matches the shipping CLI 1:1.
- A reader opening `book/src/knobs.md` sees every flag they
  can pass on the command line, with defaults and intended
  effect. Session-recovery resilience restored for the knobs
  chapter.

---

## 2026-04-16-0052 — Block counters: priority_encoder + comb_mux_encoding (closes the last pending effectiveness-map entries)

**What changed**
- `src/ir/types.rs`: three new live-counter fields on `Module`:
  - `priority_encoder_built: u32`
  - `comb_mux_one_hot_built: u32`
  - `comb_mux_encoded_built: u32`
  Each is a per-module tally incremented at the block-build site.
- `src/gen/cone.rs`: increments at four sites:
  - `build_priority_encoder_recursive` — increments once the
    assemble succeeds.
  - `build_priority_encoder_pool` — same.
  - `build_comb_mux` — increments either the `one_hot` or
    `encoded` counter based on the `comb_mux_encoding_prob` coin
    before dispatching to the assembly helper.
  - `build_comb_mux_pool_only` — same counter pair inside its own
    encoded/one-hot branches.
- `src/metrics.rs`: three new `Metrics` fields
  (`num_priority_encoder_blocks`, `num_comb_muxes_one_hot`,
  `num_comb_muxes_encoded`) populated from the Module counters.
- `book/src/knobs.md` effectiveness map:
  - `priority_encoder_prob` → concrete metric.
  - `comb_mux_prob` → sum-of-two-counters (plus
    `num_muxes_2to1` still applicable).
  - `comb_mux_encoding_prob` → ratio of encoded / total comb muxes.
  - **No *pending* entries remain.** Closing paragraph rewritten.
- `USER_GUIDE.md`: knob-effects bullet list extended with the two
  new rate knobs' expected shifts.
- `CODEBASE_ANALYSIS.md`: `metrics.rs` one-liner extended.

**Why**
Two entries still marked *pending* on the effectiveness map:
`priority_encoder_prob` and `comb_mux_encoding_prob`. Both
required detecting block shape after the block had lowered to
a chain of gates, which is brittle post-hoc. The better fix is a
live counter at the block-build site — cheap, accurate, and
matches the "construction-time measurement" doctrine.

**Tests**
- All four cargo gates green.
- 54 tests pass.
- Demo sweeps at seed 42:

  ```
  priority-encoder-prob=0.0:  num_priority_encoder_blocks=0
  priority-encoder-prob=0.05: num_priority_encoder_blocks=49
  priority-encoder-prob=0.2:  num_priority_encoder_blocks=221
  priority-encoder-prob=0.5:  num_priority_encoder_blocks=454

  comb-mux-encoding-prob=0.0 (--comb-mux-prob 0.4): one_hot=2229, encoded=0
  comb-mux-encoding-prob=0.5 (--comb-mux-prob 0.4): one_hot=887,  encoded=859
  comb-mux-encoding-prob=1.0 (--comb-mux-prob 0.4): one_hot=0,    encoded=959
  ```

  Clean monotone for `priority_encoder_prob`; near-even split at
  `comb_mux_encoding_prob=0.5` (887/859 ≈ 50.8%/49.2%).

**Impact**
- Every knob in the catalog now has a concrete, measurable
  effect. The "every knob must be measurable" doctrine (Rule:
  knob measurement) is empirically satisfied across the full
  knob set.
- No behaviour change.

---

## 2026-04-16-0051 — Combinational-depth metrics (closes another pending effectiveness-map entry)

**What changed**
- `src/metrics.rs`: two new `Metrics` fields
  - `max_gate_depth: usize` — longest combinational fan-in path
    through any gate. Primary inputs, constants, and flop Q act
    as depth-0 leaves (clock edge breaks Q→D loop temporally).
  - `gate_depth_histogram: BTreeMap<usize, usize>` — count of
    gates at each depth.
  - `compute()` adds a single forward walk over `m.nodes` (which
    is always in topological order) that assigns depth =
    `max(operand depth) + 1`.
- `book/src/knobs.md` effectiveness map: `max_depth` moves from
  `pending` to `max_gate_depth` / `gate_depth_histogram`.
- `USER_GUIDE.md`: metrics section and knob-effects bullet list
  extended with the new fields.
- `CODEBASE_ANALYSIS.md`: metrics.rs one-liner updated.

**Why**
Two entries still listed as *pending* in the knob-effectiveness
map: `max_depth` and `priority_encoder_prob` / `comb_mux_encoding_prob`.
`max_depth` is the easier one — a single topological walk gives
both `max_gate_depth` and a full histogram. Closes one of the
three remaining pending entries.

**Relationship between `max_depth` knob and `max_gate_depth`
metric:** the knob bounds `build_cone` recursion depth; the
metric measures IR gate-chain depth. They are NOT 1:1 because
block-assembly helpers (chained-ternary muxes, OR-of-masked-arms
muxes, linear-combination trees) expand each recursion level
into many gate layers. The metric is typically 10–100× the knob
value but monotone in it — useful for verifying knob effect.

**Tests**
- All four cargo gates green.
- 54 tests pass (no new tests; single-walk metric is obvious
  from the existing `compute` structure).
- Demo sweep at seed 42:
  ```
  max-depth=2:  max_gate_depth=54
  max-depth=4:  max_gate_depth=115
  max-depth=6:  max_gate_depth=154
  max-depth=8:  max_gate_depth=206
  max-depth=10: max_gate_depth=354
  ```
  Clean monotone.

**Impact**
- Effectiveness-map *pending* list shrinks from 3 → 2
  (`priority_encoder_prob`, `comb_mux_encoding_prob` still
  pending).
- No behaviour change.

---

## 2026-04-16-0050 — Live-doc catch-up: CODEBASE_ANALYSIS, USER_GUIDE, DEVELOPMENT_NOTES, ROADMAP (docs only)

**What changed**
- `CODEBASE_ANALYSIS.md`:
  - Module map refreshed: `lib.rs` trace infrastructure
    (TRACE_DEBUG, set_trace_debug, trace_verbose! macro);
    `metrics.rs` new operand-arity fields; `config.rs`
    FactorizationLevel + the three duplication-rate knobs;
    `ir/types.rs` intern_gate / intern_constant API + dedup
    tables + per-module knob mirrors; `gen/module.rs` orphan
    audit; `gen/cone.rs` GraphFirst retirement + build_cone
    snapshot/rollback + process_signal_frame existing-operand
    fallback + pick_terminal_dep_bearing +
    pick_datas_with_dup_cap + pick_signals_with_dup_rate.
  - Phase coverage map: Phase 1 promoted in-progress → mostly
    done (22 structural rules enforced, 0 orphans, 0 default
    duplicate operands; blocked only on external Verilator/Yosys
    smoke). Phase 2 entry notes CSE + operand-uniqueness +
    commutative. Phase 3 entry notes priority-encoder landed.
  - Invariants list: intern_gate CSE + commutative contract;
    build_cone snapshot/rollback; process_signal_frame
    anti-collapse fallback; generate_leaf_module orphan audit.
  - Test count: 35 + 15 → 39 + 15 = **54**.
- `USER_GUIDE.md`:
  - Metrics section lists the operand-arity fields explicitly.
  - Knob-effects bullet list extended with
    `operand_duplication_rate`, `max_gate_arity`, and
    `factorization_level` entries.
- `DEVELOPMENT_NOTES.md` (first update since `e6850fc` —
  15 src-touching commits of drift):
  - "Construction-time CSE via Module::intern_gate" — method
    contract, dedup-table rationale, snapshot contract with
    build_cone_with_retry.
  - "Rule 18 α construction-time" — α vs β decision record +
    GraphFirst retirement rationale.
  - "Full factorization doctrine" — NodeId = expression identity,
    the 7-layer implementation ladder, FactorizationLevel::
    effective() clamping.
  - "Emitter is a dumb serialiser" — doctrinal anchor.
  - "Rejected: without-replacement operand picking as default"
    — why the anti-collapse + rollback path was chosen.
- `ROADMAP.md`:
  - Phase 1 label: in progress → mostly done; exit-criteria note
    that external smoke tests are blocked locally; internal
    validation (54 tests, 0 orphans, 0 default duplicates) is
    complete.
  - Phase 3: in progress; per-item status (priority encoder
    landed, constant-shift landed, linear-combination landed;
    case/casez / reductions / variable-shift / for-loop not
    started).

**Why**
Per user directive: *"Please strictly follow the commit workflow
w.r.t. which live docs shall be updated. It is not cosmetic, it
is of utmost importance to ensure the continuity of the project
following session loss or crash."*

Audit showed DEVELOPMENT_NOTES.md had not been touched in 15
src-changing commits (`92d43f8` through `64850da`). Several of
those slices embedded design decisions (Rule 18 α, intern_gate
contract, FactorizationLevel) that deserved permanence beyond
commit messages. CODEBASE_ANALYSIS.md was stale on module
ownership. USER_GUIDE.md missed the new knob surface. ROADMAP.md
had Phase 1 still at "in progress" despite Rule-18 enforcement
and the full factorization work.

**Tests**
- No code changed.
- All four cargo gates green.
- 54 tests unchanged.
- `mdbook build book` succeeds.

**Impact**
- A session that recovers cold from `git clone` → `SESSION_BOOTSTRAP.md`
  now sees the actual workspace reality in `CODEBASE_ANALYSIS.md`
  and the design rationale in `DEVELOPMENT_NOTES.md`.
- No behavioural change.

---

## 2026-04-16-0049 — Operand-arity metrics (closes a pending effectiveness-map entry)

**What changed**
- `src/metrics.rs`:
  - New `Metrics::gate_operand_count_histogram: BTreeMap<usize,
    usize>` — count of gates per operand-count value.
  - New `Metrics::max_gate_operand_count: usize` — the largest
    operand list observed on any single gate.
  - New `Metrics::max_operand_count_by_kind:
    BTreeMap<String, usize>` — per-`GateOp`-kind ceiling,
    distinguishing e.g. `add`'s arity (bounded by
    `max_gate_arity`) from `concat`'s arity (driven by
    replicate-to-width).
  - `compute()` populates all three during the existing single
    walk over `m.nodes`; no new passes.
- `book/src/knobs.md`: effectiveness-map entry for
  `min_gate_arity` / `max_gate_arity` moves from *pending* to
  concrete metric names.

**Why**
The knob-effectiveness map had `min_gate_arity` / `max_gate_arity`
marked *pending*. Per the measurement doctrine ("no knob is
privileged"), every knob needs a metric. This slice provides one.

**Tests**
- All four cargo gates green.
- 54 tests pass.
- Demo sweep at seed 42:
  ```
  max-gate-arity=2: max_gate_operand_count=8  (add max=2, mul max=3)
  max-gate-arity=4: max_gate_operand_count=27 (add max=4, mul max=5)
  max-gate-arity=6: max_gate_operand_count=11 (add max=6, mul max=7)
  max-gate-arity=8: max_gate_operand_count=27 (add max=8, mul max=9)
  ```
  - `add` max tracks the knob exactly.
  - `mul` max is `knob + 1` because the Mul linear-combination
    motif prepends a coefficient operand.
  - `max_gate_operand_count` top end is driven by `concat`
    (replicate-to-width can produce 27-operand concats
    regardless of the gate-arity knob) — exactly the reason a
    per-kind breakdown is useful.

**Impact**
- No behavioral change. Observability gain only.
- The effectiveness map moves one more knob off the *pending*
  list; remaining gaps: `max_depth`, `priority_encoder_prob`,
  `comb_mux_encoding_prob`.

---

## 2026-04-16-0048 — Close residual Add/Mul/And duplicate operands at default knobs

**What changed**
- `src/gen/cone.rs`:
  - `assemble_mul_linear_combination`: when
    `operand_duplication_rate < 1.0`, dedupes the `signals` list
    before building the N-ary `Mul`. `c * x * x * y` becomes
    `c * x * y` (loses the x² factor — that's the user's
    explicit no-duplicates contract). Preserves the `coef == 1 ⇒
    n >= 2` invariant via a degenerate passthrough when
    dedup produces a single signal.
  - `assemble_add_linear_combination`: same post-Mul
    dedup on the outer `Add`'s `terms` list. Two terms can
    coincide when signal + coefficient both match (CSE-collapse);
    we drop the duplicate.
  - `make_and`: idempotent short-circuit `x & x = x` when
    `factorization_level.effective() >= OperandUnique`. Closes
    the last escape path — the one-hot-mux mask assembly can
    produce `And(sel, data)` where `sel == data` via CSE at
    width=1, which bypassed anti-collapse because `make_and`
    calls `intern_gate` directly.
- `src/config.rs`: `FactorizationLevel` default now written
  via `#[default]` derive instead of hand-rolled `impl Default`
  (clippy hint). Doc comments for the enum variants reworded to
  avoid the `+ X` leading character that clippy parsed as a
  bullet list.

**Why**
Previous audit showed 0.09% duplicate-operand Add/Mul/And gates
at default `operand_duplication_rate = 0.0`. Per the user's full-
factorization doctrine these should be exactly zero.

**Tests**
- All four cargo gates green.
- 54 tests pass.
- Orphan audit: 0 across 4 strategies × 6 seeds (Rule 18 holds).
- Duplicate-operand audit:
  - `rate=0.0` (default): 4633 gates, **0 duplicates (0.000%)**
    — exactly zero, down from 0.09%.
  - `rate=1.0`: 5336 gates, 57 duplicates (1.07%) — knob
    still active.

**Impact**
- Syntactic factorization layer (CSE + operand-unique +
  commutative) is now *complete* at default knobs — no operand
  duplication anywhere across the tested seed range.
- Next layer (associative flattening) can now be designed
  without the noise of residual duplicates.

---

## 2026-04-16-0047 — Recipe: "I want to see how the factorization dial affects output" (docs only)

**What changed**
- `book/src/recipes.md`: new recipe walking a user through the
  `--factorization-level` dial with a real shell sweep, captured
  output at seed 42, and a layer-by-layer explanation of the
  deltas.

**Why**
Per user book doctrine — "littered with examples." The
factorization dial landed in the previous slice with catalog docs
in `knobs.md` and the rule text in `structural-rules.md`, but
there was no paste-and-run recipe for a user who wants to *see*
the knob. This slice provides one.

**Tests**
- No code changed.
- `mdbook build book` succeeds.
- 54 tests unchanged.

---

## 2026-04-16-0046 — Commutative normalization + factorization-level dial

**What changed**
- `src/ir/types.rs` `Module::intern_gate`:
  - Sorts operands of commutative ops (`And`/`Or`/`Xor`/`Add`/`Mul`)
    before building the dedup key, so `a + b` and `b + a` share
    a single NodeId. Gated on `factorization_level >=
    Commutative`.
  - New `None` fast path: every intern_gate / intern_constant
    call bypasses the dedup table and creates a fresh node.
    Useful for stress-testing downstream CSE.
  - Two unit tests covering commutative-vs-non-commutative
    interning.
- `src/config.rs`:
  - New enum `FactorizationLevel` with 8 levels:
    `None → Cse → OperandUnique → Commutative → Associative →
    ConstantFold → Peephole → EGraph` (default `EGraph`).
  - `Config::factorization_level` field, threaded through
    `Overrides` and `apply_cli_overrides`.
  - `FactorizationLevel::highest_implemented()` returns
    `Commutative` today; `effective()` clamps user requests
    down to that ceiling so aspirational levels don't error.
- `src/main.rs`: `--factorization-level <LEVEL>` CLI flag.
- `src/ir/types.rs` `Module.factorization_level` mirror field.
- `src/gen/module.rs`: wire from config.
- `src/gen/cone.rs` `violates_anti_collapse`: operand-uniqueness
  checks now gated on `factorization_level.effective() >=
  OperandUnique`.
- `book/src/structural-rules.md`:
  - New Rule 21b "Commutative normalization".
  - New Rule 21c "Factorization level" with the level table,
    doctrinal anchor ("NodeId = expression identity"), and the
    aspirational-anchor mechanism.
- `book/src/knobs.md`: new catalog entry + quick-reference row.

**Why**
User coined "full factorization" as the doctrine: NodeId is the
identity of an expression; no expression / sub-expression ever
duplicated. User directive: "we need a knob to control where on
the chain we want to be, default e-graph."

**Tests**
- All four cargo gates green.
- 39 unit + 15 integration = **54 tests, all passing**.
- Empirical dial at seed 42:

```
none             gates=1961     (no dedup — fresh node per call)
cse              gates=1776     (syntactic CSE only)
operand-unique   gates=2368     (+ Rule 8 operand uniqueness)
commutative      gates=2368     (+ commutative sort)
associative      gates=2368     (aspirational → commutative today)
constant-fold    gates=2368     (aspirational → commutative today)
peephole         gates=2368     (aspirational → commutative today)
e-graph          gates=2368     (aspirational → commutative today; DEFAULT)
```

**Impact**
- Default behaviour unchanged vs previous default (both land at
  `commutative`).
- Users can now dial *down* (for stress-testing) via
  `--factorization-level none` or `cse`.
- Aspirational levels (`associative`, `constant-fold`,
  `peephole`, `e-graph`) compile without behavioural surprise
  — when future slices implement them, users at those levels
  automatically gain the tighter factorization. No config
  migration needed.

---

## 2026-04-16-0045 — Operand-uniqueness knob (`--operand-duplication-rate`)

**What changed**
- `src/config.rs`: new knob `operand_duplication_rate: f64` in
  `[0.0, 1.0]`, default `0.0`. Threaded through `Overrides` +
  `apply_cli_overrides`. Applies to Add/Mul operand lists only —
  And/Or/Xor are always strict (algebraic collapses). Comparisons,
  Sub, Mux retain their 2-operand degenerate-shape rejection
  governed by Rule 8 / Rule 22.
- `src/main.rs`: CLI flag `--operand-duplication-rate <F>`.
- `src/ir/types.rs`: `Module.operand_duplication_rate` field.
- `src/gen/module.rs`: `generate_leaf_module` clamps + forwards
  the config value to the module.
- `src/gen/cone.rs`:
  - `violates_anti_collapse` signature change: `_m` → `m`
    (uses the module's knob fields).
  - Add/Mul operand-list duplicates are now flagged when
    `m.operand_duplication_rate < 1.0`.
  - Mux degenerate data-arm shape uses `m.mux_arm_duplication_rate
    < 1.0` (cleaner, same semantics).
  - New helper `pick_signals_with_dup_rate` mirrors
    `pick_datas_with_dup_cap` — used in
    `build_linear_combination_pool` so pool-mode Add/Sub/Mul
    signals are distinct.
  - Existing anti-collapse test updated to assert Add/Mul
    duplicates ARE flagged at default rate (was the inverse).
- USER_GUIDE / Knobs references updated via follow-up book slice.

**Why**
User directive — "we need to opt in to allow duplicates; by
default they are not allowed." Previously `Add` and `Mul` were
Rule-8 exempt (duplicates algebraically meaningful), so you could
get `mul = c * s * s * s * s` or `add = s + s + s` at default
knobs. Now default is strict uniqueness; the user explicitly
passes `--operand-duplication-rate 1.0` to exercise those shapes.

**Tests**
- All four cargo gates green.
- 49 tests pass.
- Empirical verification across 5 seeds:
  - `rate=0.0` (default): 4374 gates, 4 with duplicate operands
    (0.09%). Residuals come from the recursive linear-combination
    path where `build_cone` recursion + CSE can collapse two
    distinct sub-cones to the same NodeId; rewriting that path to
    enforce uniqueness without introducing orphans is a deferred
    follow-up.
  - `rate=1.0`: 5184 gates, 56 duplicates (1.08%). Knob clearly
    active.
- No orphans introduced (Rule 18 still holds).

**Impact**
- Default output has `x + y + z` instead of `x + x + y`.
- `x + x = 2x` / `x * x = x²` shapes reachable on demand.

---

## 2026-04-16-0044 — `--trace debug` is now strictly more verbose than `high`; `off` → `none`

**What changed**
- `src/lib.rs`:
  - New static `TRACE_DEBUG: AtomicBool` + public
    `set_trace_debug(bool)` / `trace_debug_enabled()` helpers.
  - New `trace_verbose!` macro: fires `tracing::trace!` only when
    `TRACE_DEBUG` is true. Used for super-verbose events that
    would flood `--trace high`.
- `src/main.rs`:
  - `TraceLevel` renamed `Off` → `None` with `#[value(alias = "off")]`
    so `--trace off` still works. Default remains silent.
  - `TraceLevel::debug_verbose()` returns true only for `Debug`.
  - `init_tracing` calls `anvil::set_trace_debug(cli.trace.debug_verbose())`.
- `src/ir/types.rs`: `intern_gate` and `intern_constant` emit
  `trace_verbose!` events on both creation (`🔗 new`) and
  reuse-on-cap-hit (`♻️ reuse`). Every node that enters the IR
  is now traceable, with span context showing which call path
  created it.
- `src/gen/cone.rs`: `pick_gate` return logged with
  `trace_verbose!(?op, depth, width, "🎲 pick_gate")` in both
  `build_cone` (recursive path) and `process_signal_frame`
  (interleaved path). Linear-combination motif dispatch also
  gets a `trace_verbose!` marker.
- `USER_GUIDE.md`: tracing level table updated with accurate
  descriptions and the `none` / `off` alias note.

**Why**
User directly tested `--trace debug` vs `--trace high` and found
them identical (both mapped to `LevelFilter::TRACE`). They also
expected `--trace none` to work, but the CLI only accepted `off`.
Both were real defects — the CLI advertised a level that did
nothing, and the naming didn't match user expectations. User
directive: "we should be able to see everything, start/end of
every function, every branch. Without this it is painful to
debug efficiently."

**Tests**
- All four cargo gates green.
- 49 tests pass.
- Empirical verification at seed 42:
  - `--trace none` → 0 lines
  - `--trace low` → 5 lines
  - `--trace medium` → 141 lines
  - `--trace high` → 3779 lines
  - `--trace debug` → 8241 lines (+4462 strict super-set)
  - `--trace off` still accepted as silent-alias.
- Sample `debug`-only events (not visible at `high`):
  - `🎲 interleaved pick_gate op=Mux depth=0 width=21`
  - `🔗 intern_gate new node=5 op=Not width=11 n_operands=1`
  - `🔗 intern_constant new node=6 width=1 value=0`
  - `♻️ intern_gate reuse (AST cap hit) node=N op=X width=W`

**Impact**
- `--trace debug` is now the tool for answering "who created this
  node?" — every `intern_*` call surfaces with its span context.
- Zero performance impact at `--trace none` (default) or at
  `high` (the `trace_verbose!` guard is an atomic load + `false`
  short-circuit).
- Trace output remains deterministic across runs with the same
  `(seed, knobs)`.

**Known residual trace gaps** (future slices):
- `pick_terminal` doesn't emit a `trace_verbose!` on every call
  (only tier-pick events at `high`). Already covered sufficiently
  for most debugging.
- Block-assembly helpers (`assemble_flop_d_encoded`,
  `build_comb_mux_encoded`, priority encoder) don't emit a
  `trace_verbose!` event for each assembly step. Adding them is
  a straightforward follow-up if block-debug is needed.

---

## 2026-04-16-0043 — Zero orphans: Rule 18 enforced (construction-time)

**What changed**
- `src/gen/cone.rs`:
  - `build_cone` (recursive path) snapshots `m.nodes.len()`,
    `m.flops.len()`, pool, worklist, `gate_instances`, and
    `const_instances` before building operand sub-trees. On
    anti-collapse rejection the snapshot is restored and
    `pick_terminal` provides a safe fallback. Sub-tree nodes
    that were built for the rejected gate are truncated — no
    orphan leaks.
  - `process_signal_frame` (interleaved path): the frame machine
    cannot snapshot per-gate because sibling sub-frames have
    already committed. On anti-collapse rejection it delivers
    one of the existing operand `NodeId`s as the fallback instead
    of creating a new `pick_terminal` node. For idempotent /
    self-inverse / comparison collapses the operands share a
    NodeId, so the fallback is semantically correct. For
    `mux(s, a, a)` it uses `operands[1]` (= operands[2]).
- `src/config.rs`:
  - Default `construction_strategy` switches from `GraphFirst` to
    `Interleaved`. GraphFirst was the only strategy that
    speculatively created pool units before knowing who would
    consume them (13–27% orphan rate).
  - `GraphFirst` enum variant retained as a silent alias for
    `Interleaved` so existing CLI invocations / config files
    continue to work; the speculative pool-growth code path is
    unreachable.
- `src/gen/module.rs`:
  - Match on `construction_strategy` routes both `Interleaved`
    and `GraphFirst` to `cone::build_outputs_interleaved`.
  - New safety-net audit `count_orphan_gates(m)` called after
    flop drain; warns via `tracing::warn!` if any Gate has no
    consumer.
- `src/emit/sv.rs`: emitter goes back to a dumb serialiser.
  Per user doctrine — "all thinking, checks, rules' enforcement
  ought to be done solely at the IR level; by the time you reach
  emission, it is too late to roll back." The brief live-set
  filter added in a previous iteration is removed; `to_sv`
  iterates `m.nodes` and dumps.
- Emitter test updated: `slice_and_concat_rendered` now chains
  the slice + concat through the drive-root so both are live.

**Why**
User directive: "A gate/module/block shall come into existence
solely when needed, not speculatively created beforehand in the
hope they will be picked and connected." — "Not acceptable!"

**Tests**
- All four cargo gates green.
- 34 unit (+0 net) + 15 integration = **49 tests, all passing**.
- Orphan audit across 4 strategies × 6 seeds (1, 42, 100, 777,
  9999, 12345): **0 orphans in every run.**
- Reproducibility holds: graph-first and interleaved now produce
  byte-identical output for the same seed (graph-first is an
  alias).
- No undeclared references in any emitted SV (verified 4 × 4
  strategy×seed matrix).

**Impact**
- Default output is smaller and cleaner. No declared wire goes
  unused; no referenced signal is undeclared.
- For users who explicitly selected `--construction-strategy
  graph-first`: behavior is now identical to `interleaved`. No
  CLI break.
- Generator's "by construction" contract is now honoured for
  Rule 18 too — no post-emission filter exists.

**Known trace-coverage gap (deferred)**
User flagged that the trace doesn't clearly show "which node
requested this new gate." `build_cone` and `process_signal_frame`
don't emit an op-pick trace event, so `--trace high` can't be
used to answer "who created `not_0`?" Follow-up slice will add
explicit trace events at every intern_gate call site with
requester context.

---

## 2026-04-16-0042 — IR chapter refresh + future-extensions roadmap (docs only)

**What changed**
- `book/src/ir.md`:
  - `Module` struct snippet refreshed: now shows `gate_instances`,
    `const_instances`, `max_ast_instances`,
    `mux_arm_duplication_rate` fields.
  - New section "Node construction: `intern_gate` /
    `intern_constant`" documenting the method signatures, cap
    semantics, why the dedup tables live on `Module`, and the
    snapshot/rollback contract with `build_cone_with_retry`.
  - Emitter naming section updated for Rule 12: no more `w_N`
    or `r_N`; current naming is `<gate_kind>_<N>` per-kind + `flop_<id>`.
  - New "Future extensions" section capturing the parameters /
    synthesizable-aggregates / first-class-blocks roadmap analysis
    in durable form. Parameters (Phase 5, hard-requires Phase 4).
    Aggregates split into four sub-paths with explicit
    cost/payoff per path (packed = cheap emitter-only; unpacked
    arrays = memories, already Phase 6; unpacked datapath
    aggregates + enums = deprioritised). Blocks as first-class
    IR cross-references the session memory on
    hierarchical-vs-flatten-with-mangling.
- `ROADMAP.md`:
  - Phase 5 entry adds cross-reference to IR chapter and names
    Phase 4 as hard prerequisite.
  - New Phase 5b entry for aggregates (scheduled alongside
    Phase 5, order not fixed), pointing to IR chapter for the
    four-sub-path breakdown.

**Why**
User direction: the book must thoroughly document the IR as it
evolves, and the parameters/aggregates discussion from the
preceding exchange must land in durable docs, not just commit
messages.

**Tests**
- No code changed.
- `mdbook build book` succeeds.
- 50 tests unchanged.

**Impact**
- Next session (or a cold reader) can open `book/src/ir.md` and
  see the full current IR plus the design record for the two
  roadmap axes, without losing context to session compaction.

---

## 2026-04-16-0041 — Friendly docs: quick ref, naming refresh, recipe examples (docs only)

**What changed**
- `book/src/getting-started.md`:
  - Refresh the sample module output to match current typed-per-
    kind naming (`slice_0`, `add_0`, `mul_0` — was `w_2`, `w_3`,
    `w_4`). Added `--construction-strategy sequential` to the
    starter command so the output stays small.
  - New paragraph explaining the naming scheme briefly with a
    pointer to Rule 12.
- `book/src/knobs.md`:
  - New reassuring opening: "you don't need to read this
    top-to-bottom". Points new readers at the Recipes chapter
    first.
  - New "Quick reference" table covering the ~13 knobs most users
    actually touch, with defaults and one-line descriptions.
- `book/src/recipes.md`: six new recipes covering the knobs and
  workflows that landed in this session:
  - "I want to see fewer redundant expressions" (strict CSE —
    the default).
  - "I want duplicated expressions anyway" (bounded duplication
    via `--max-ast-instances`).
  - "I want pathological mux shapes" (arm duplication via
    `--mux-arm-duplication-rate`).
  - "I want to verify a knob is doing something" (the metric-
    grep workflow).
  - "I want to sweep a knob and compare" (shell loop + jq,
    with a real `--flop-prob` sweep as the example).
  - "I want to trace what the generator is doing" (--trace
    levels with sample output).

**Why**
Per user direction: the knobs+metrics+tracing information that
landed in the last several slices needs to be user-facing and
*not scary*. The getting-started sample was out of date (old
`w_N` naming); the knobs reference didn't tell newcomers it's
a catalog they can skim, not a syllabus; the recipes chapter
didn't cover any of the new knobs.

**Tests**
- No code changed; no test impact.
- `mdbook build book` succeeds.
- 50 tests unchanged.
- The `--flop-prob` sweep values in the recipe were verified
  against real CLI output at seed 42.

**Impact**
- New reader's path: README → SESSION_BOOTSTRAP → book
  Getting Started → Tutorial → Recipes. All four now show
  current naming, current knobs, current workflows.
- Every landed knob now has a recipe or quick-reference entry
  somewhere in the book — no knob is orphaned in code only.

---

## 2026-04-16-0040 — Knob measurement doctrine + effectiveness map (docs only)

**What changed**
- `book/src/knobs.md`:
  - New opening section "Measurement doctrine": every knob is
    subject to the same rule — its effect must be empirically
    measurable via `Metrics` and/or `--trace`. No knob is
    privileged. Three landing requirements: (1) a metric captures
    the knob's effect; (2) the knob's section names the metric;
    (3) a CLI spot-check at boundary values shows the metric
    shifting.
  - New sub-section "AST uniqueness / duplication" covering the
    two recent knobs (`max_ast_instances`, `mux_arm_duplication_rate`)
    with cross-references to Rules 21 and 22 in the structural-
    rules catalog.
  - New table at the bottom, "Knob effectiveness map" — one row
    per knob listing the metric(s) that measure its effect.
    Entries marked *pending* flag knobs whose effect the current
    metric set does not yet capture (candidates for a follow-up
    slice).
- No code changed.

**Why**
Per user direction: the knobs + metrics design discussion from
this session must land in durable docs, not just commit
messages. The knobs chapter was already the canonical knob
reference but lacked (a) the doctrinal line that no knob is
privileged, (b) the two new knobs, (c) the explicit knob → metric
mapping.

**Tests**
- No code changed; no test impact.
- `mdbook build book` succeeds (HTML written to `book/book-out`).
- 50 tests unchanged.

**Impact**
- Durable design record for the next session.
- Explicit catalog of gaps (pending metrics) to address in
  follow-up slices.

---

## 2026-04-16-0039 — Structural metrics (per-module observability)

**What changed**
- New module `src/metrics.rs` with `Metrics` struct and
  `metrics::compute(m: &Module) -> Metrics`. Post-hoc walk over a
  generated module — no generator instrumentation required. Fields:
  - Size: `num_inputs`, `num_outputs`, `num_nodes`, `num_gates`,
    `num_constants`, `num_primary_inputs`, `num_flop_q_refs`,
    `num_flops`.
  - Per-kind distribution: `gates_by_kind` (BTreeMap<kind, count>),
    `constants_by_width`.
  - Constants: `constants_zero`, `constants_all_ones`,
    `constants_other`.
  - Mux shape: `num_muxes_2to1`, `num_muxes_degenerate`.
  - Concat shape: `num_concats_replication` (all operands
    identical → `{N{expr}}`) vs `num_concats_heterogeneous`.
  - Sharing / fanout: `num_shared_nodes` (fanout ≥ 2),
    `max_fanout`, `avg_fanout`.
  - Flops: `flops_zero_default`, `flops_qfeedback`,
    `flops_mux_none`, `flops_mux_one_hot`, `flops_mux_encoded`.
  - AST-instance saturation: `max_gate_ast_multiplicity`,
    `max_constant_ast_multiplicity`.
- `src/main.rs`: new CLI flag `--metrics`. For the single-module
  path it prints metrics JSON to stderr. For multi-module runs
  the metrics block is always embedded in `manifest.json` per
  entry (replacing the tiny `{file, name, inputs, outputs, nodes}`
  summary).
- 3 new unit tests in `metrics` module (empty, per-kind, flop
  shape).
- `USER_GUIDE.md`: new "Metrics" section with CLI examples and a
  list of typical sweep-verify workflows.

**Why**
User directive: "every aspect of what is generated, every knob
related generated shall be able to measure the effectiveness of
the knobs." Metrics give us empirical grounding — without them
we can't tell whether `mux_arm_duplication_rate = 0.0` actually
produces 0 degenerate muxes, or whether `max_ast_instances = 5`
lets expressions reach the cap, or whether `flop_prob = 0.15`
produces the expected flop-density. Now each is a grep away.

**Scope chosen (post-hoc, structural only)**
Live counters (probability rolls fired/missed, anti-collapse
retries, terminal-tier picks) are deliberately deferred — they
need instrumentation at every decision site, ~40 edit points.
Most are already surfaced as `--trace high` events; aggregating
them into counters is a future extension if the post-hoc
structural metrics aren't sufficient.

**Tests**
- All four cargo gates green.
- 35 unit (+3 new) + 15 integration = **50 tests, all passing**.
- Demonstrated observability: at seed 42 default,
  `num_muxes_degenerate = 0` (matches Rule 22 at rate 0.0);
  at `--mux-arm-duplication-rate 1.0`, it jumps to 1.
  `max_gate_ast_multiplicity = 1` at default; at
  `--max-ast-instances 5`, rises to 3 with 29 more nodes in
  the module.

**Impact**
- New public API: `anvil::metrics::{Metrics, compute}`.
- `manifest.json` shape changed: `inputs`/`outputs`/`nodes`
  summary replaced with a full `metrics` field. Consumers of the
  old shape need to update.

---

## 2026-04-16-0038 — Mux arm-duplication rate (Rule 22)

**What changed**
- `src/config.rs`: new knob `mux_arm_duplication_rate: f64` with
  range `[0.0, 1.0]`; default `0.0` = all arms of any N-to-1 mux
  must be distinct signals. Threaded through `Overrides` and
  `apply_cli_overrides`.
- `src/main.rs`: new CLI flag `--mux-arm-duplication-rate <F>`.
- `src/ir/types.rs`: `Module.mux_arm_duplication_rate` field.
  `generate_leaf_module` initialises from config (clamped to
  `[0.0, 1.0]`).
- `src/gen/cone.rs`:
  - New helper `pick_datas_with_dup_cap(g, m, pool, width, count,
    exclude)`: picks `count` arm signals; on a duplicate candidate,
    keeps with probability `mux_arm_duplication_rate`, otherwise
    re-picks (bounded 8-try budget). Used at all pool-mode mux
    assembly sites: encoded/one-hot comb-mux, encoded/one-hot
    flop-D drain.
  - `make_mux`: at rate `0.0`, `a == b` collapses the layer to
    return `a` directly (the 2-to-1 case). At any rate `> 0.0`,
    the mux is emitted as-is — the upstream caller has already
    decided whether duplication is permitted for this arm.
- `book/src/structural-rules.md`: new Rule 22 "Mux arm-duplication
  rate" with motivation, construction-time enforcement, and knob
  semantics.

**Why**
User flagged `mux_9 = (eq_0) ? (flop_0) : (flop_0)` as a
pathological form: a mux with both data arms connected to the
same signal is structurally redundant (equivalent to the data
signal alone). Rule 8 already forbade the 2-to-1 case at the
`Mux` gate level, but `make_mux` bypassed the anti-collapse
check when called from the chained-ternary assembly. The
broader N-to-1 generalisation — "m arms out of M share the same
data" — was uncontrolled until this slice.

The knob exists because the pathological form is genuine
downstream-tool input: we want to emit it *on request* (for
stress testing) but not by default.

**Tests**
- All four cargo gates green.
- 32 unit + 15 integration = **47 tests, all passing**.
- Verified knob behavior at seed 42:
  - Default (rate = 0.0): 17 ternary expressions, **0** with
    the degenerate `(X)?(Y):(Y)` shape.
  - `--mux-arm-duplication-rate 1.0`: 11 ternary expressions,
    **1** degenerate (chained-ternary layers collapse more
    often when arms repeat → fewer total mux nodes).

**Impact**
- Default output no longer contains any `(s)?(x):(x) = x`
  redundant-mux lines. Module semantics unchanged.
- At high rates, downstream synthesis tools see redundant-arm
  patterns for stress coverage.

---

## 2026-04-16-0037 — Construction-time CSE with tunable AST-instance cap (Rule 21)

**What changed**
- `src/ir/types.rs`:
  - `Module` gains `gate_instances: HashMap<(GateOp, Vec<NodeId>, u32), Vec<NodeId>>`,
    `const_instances: HashMap<(u32, u128), Vec<NodeId>>`, and
    `max_ast_instances: u32`.
  - New methods `Module::intern_gate(op, operands, width, deps) → (NodeId, is_new)`
    and `Module::intern_constant(width, value) → (NodeId, is_new)`.
    Cap behavior: create new if `vec.len() < max_ast_instances`,
    else return `*vec.last()`.
  - `GateOp` gains `Hash` derive.
- `src/config.rs`: new knob `max_ast_instances: u32` (default 1 = strict CSE).
  Threaded through `Overrides` and `apply_cli_overrides`.
- `src/main.rs`: new CLI flag `--max-ast-instances <N>`.
- `src/gen/module.rs`: `generate_leaf_module` sets
  `m.max_ast_instances = g.cfg.max_ast_instances.max(1)`.
- `src/gen/cone.rs`: every `m.nodes.push(Node::Gate|Constant)` site
  migrated to `intern_gate` / `intern_constant`. Callers only
  `pool.add` when `is_new = true`. Helpers: `make_constant`,
  `make_eq_const`, `make_mux`, `make_and`, `make_mul`, `make_sub`,
  `make_nary_add`, `make_nary_mul`, `replicate_to_width`,
  `or_reduce_terms`, `make_width_adapter`, the deliver-path in
  `process_signal_frame`, the operator-gate-creation block in
  `grow_pool_one_unit`, `build_cone`, and `pick_terminal`'s fresh-
  constant fallback.
- Critical snapshot fix: `build_cone_with_retry` now snapshots and
  restores `m.gate_instances` and `m.const_instances` alongside
  `m.nodes` / `m.flops` / pool / worklist. Without this, a retry
  rolls back the node vec but leaves stale dedup entries pointing
  at truncated NodeIds, causing `intern_gate` to return nodes of
  wrong kind/width on subsequent calls.
- `book/src/structural-rules.md`: new Rule 21 "AST-instance cap
  (construction-time CSE)" documenting the rule, motivation,
  enforcement, and snapshot/rollback interaction.

**Why**
User flagged observable RHS duplication:
`eq_4 = slice_17 == 2'h2; … eq_9 = slice_17 == 2'h2; …`.
Construction-time hash-consing is the right answer — one RHS =
one signal = one node. But blanket CSE is too opinionated for a
stress-test generator, so the cap is a knob: default 1 (strict
CSE), raise for bounded duplication, `u32::MAX` to disable.

**Tests**
- All four cargo gates green.
- 32 unit + 15 integration = **47 tests, all passing**.
- Spot-check seed 42: `slice_17 == 2'h2` now appears exactly once
  (`eq_0`). Downstream muxes reference `eq_0` instead of creating
  copies. Verified knob behavior: at `--max-ast-instances 3`, Eq
  count doubles from 6 to 12.

**Impact**
- **Structural change.** Modules generated under default knobs are
  smaller (fewer nodes) and more shared. The SV is semantically
  equivalent to the pre-CSE output for the same `(seed, knobs)`
  only when `max_ast_instances = u32::MAX`. Otherwise output
  differs and is denser.
- Integration tests needed to account for the interaction between
  dedup and retry rollback; the snapshot-restore of dedup tables
  is the load-bearing piece.

---

## 2026-04-16-0036 — Emit `{N{expr}}` replication for same-operand Concat

**What changed**
- `src/emit/sv.rs` `render_gate` for `GateOp::Concat`: when every
  operand points at the same `NodeId`, emit the canonical SV
  replication form `{N{expr}}` instead of the flat list
  `{expr, expr, …, expr}`. Covers the `replicate_to_width` helper's
  output in one-hot mux assembly (`{W{sel_i}} & data_i`).
- Emitter unit test updated to expect `{2{a}}` instead of `{a, a}`.

**Why**
User flagged lines like
`assign concat_15 = {eq_0, eq_0, …, eq_0};` (22 copies) as "uncontrolled."
The logic is intentional (one-hot mask broadcast) but the expanded
emission form hid the idiom. The replication form is synthesis-
equivalent and matches the SV convention every synthesizer already
recognizes.

**Tests**
- All four cargo gates green.
- 32 unit + 15 integration = **47 tests, all passing**.
- Spot-check seed 42: former 22-operand `concat_15 = {eq_0, eq_0, …}`
  now emits as `concat_15 = {22{eq_0}}`. Gate count and module
  semantics unchanged.

**Impact**
- No behavior change — only emission format. Byte-identical SV
  structure modulo the replication shortcut.
- Any downstream tool that parsed the flat-list form sees the
  replication form now, which is standard SV and synthesized
  identically.

---

## 2026-04-16-0035 — UVM-style tracing (`--trace` / `--trace-file`)

**What changed**
- `Cargo.toml`: adds `tracing` (with `release_max_level_info`) and
  `tracing-subscriber`.
- `src/main.rs`: new CLI flags `--trace <off|low|medium|high|debug>`
  (default `off`) and `--trace-file <path>`. `init_tracing`
  initialises a deterministic subscriber — no timestamps, no thread
  IDs, no ANSI — with output to stderr (default) or a file. Level
  mapping: `low = INFO`, `medium = DEBUG`, `high = TRACE`,
  `debug = TRACE`.
- `src/gen/module.rs`: `#[instrument(level="info")]` on
  `generate_leaf_module`; milestone logs for module start/done with
  n_in, n_out, strategy, final node/flop/drive counts.
- `src/gen/cone.rs`: `#[instrument]` on `build_cone_with_retry` (debug),
  `build_graph_first` (info), `grow_pool_one_unit` (trace),
  `build_outputs_interleaved` (info), `process_signal_frame` (trace),
  `build_cone` (trace), `drain_flop_worklist` (debug),
  `build_comb_mux` (trace), `build_flop_leaf` (trace),
  `pick_terminal` (trace), `pick_terminal_dep_bearing` (trace).
  Explicit `trace!` / `debug!` / `warn!` at named control points:
  motif dispatch forks (flop / comb-mux / priority-encoder / operator
  gate, linear-combination, const-shift, const-comparand), retry /
  fallback (cone retry, anti-collapse retry and exhaustion, terminal
  tier 1/2/3/4 picks), leaf vs recursion decision. Emoji tags at
  milestone / retry / fallback events only.
- `src/emit/sv.rs`: `#[instrument(level="info")]` on `to_sv`; info-
  level summary of gates/flops/inputs/outputs; debug-level dump of
  per-kind counter totals from `build_names`. `build_names` now uses
  `BTreeMap` instead of `HashMap` for deterministic counter-log
  ordering (no iteration-order leak into trace output).
- `USER_GUIDE.md`: new "Tracing and debugging" section with level
  table, CLI examples, and emoji legend.

**Why**
Per user direction: generator debugging needs UVM-style graduated
verbosity with broad coverage. The project's "by construction"
contract makes *silent* bugs the main debugging hazard (wrong motif
dispatch, unexpected retry / fallback paths, width-adapter surprises)
— tracing at the named control points is the cheapest way to surface
them without touching generator logic.

**Non-negotiables honored**
- Output goes to stderr (or file); stdout stays byte-clean for SV.
- No wall-clock, no thread IDs, no ANSI, no hash-map iteration in
  trace output. Verified: `diff` of `--trace off` vs `--trace high`
  generated SV is empty for the same `(seed, knobs)`.
- Release build compiles out below `info` via the
  `release_max_level_info` feature flag.

**Tests**
- All four cargo gates green.
- 32 unit + 15 integration = **47 tests, all passing** with default
  `--trace off`.
- Reproducibility spot-check: `--trace off` and `--trace high` on
  seed 42 produce byte-identical stdout.

**Impact**
- `--trace off` is the default — zero behavioral change for existing
  users or CI.
- Release builds compile out below `info`; `low` / default (off)
  have near-zero overhead. `high` / `debug` add measurable overhead
  and should not be used in seed sweeps.
- No CLI flag was renamed; only additions.

---

## 2026-04-16-0034 — Typed per-kind naming in emitted SV (Rule 12 revised)

**What changed**
- `src/emit/sv.rs`:
  - `build_names(m) -> Vec<Option<String>>`: single-pass walk that
    assigns each `Node::Gate` a name `<kind>_<counter>` with the
    counter maintained per `GateOp` kind. Non-gate nodes get
    `None`.
  - `gate_kind_name(op) -> &'static str`: canonical lowercase
    prefix for each `GateOp` variant (`and`, `or`, `xor`, `not`,
    `add`, `sub`, `mul`, `eq`, `neq`, `lt`, `gt`, `le`, `ge`,
    `mux`, `slice`, `concat`, `red_and`, `red_or`, `red_xor`,
    `shl`, `shr`).
  - `flop_name(id) -> String`: `flop_<id>`.
  - `node_ref` / `render_gate` threaded with the `&[Option<String>]`
    name table. Non-gate nodes resolve as before (primary input
    port name, literal constant, flop Q = `flop_<id>`).
- Emitter unit tests updated to the new naming: `flop_0` replaces
  `r_0`; gate references become `and_0`, `xor_0`, `mux_0`,
  `slice_0`, `concat_0`.
- `book/src/structural-rules.md`: Rule 12 rewritten — table now
  shows `<gate_kind>_N` and `flop_N`; lists all kind prefixes;
  explains per-kind counter rationale; documents SV identifier
  legality for gate-primitive-prefixed names (`and_0` is a legal
  identifier distinct from the `and` keyword).

**Why**
Per user direction — generated SV must be inspectable at a glance.
The opaque `w_<NodeId>` scheme collapsed all structural variety
into a uniform wire name; `<kind>_<counter>` makes the gate mix
visible and aligns emitted SV with the `GateOp` taxonomy already
used in the IR.

**Tests**
- All four cargo gates green.
- 32 unit + 15 integration = **47 tests, all passing**.
- Spot-check: `cargo run -- --seed 42` now emits
  `flop_0 … flop_9`, `and_0 … and_N`, `slice_0`, `mux_0`, `concat_0`
  and similar. No `w_<N>` / `r_<N>` identifiers remain.

**Impact**
- **Breaking for downstream tools that parsed the old `w_` / `r_`
  naming.** No users yet; the change is internal to a pre-release
  generator.
- Reproducibility contract holds: names are a deterministic
  function of declaration order, which is itself a deterministic
  function of `(seed, knobs)`.
- No IR or generator changes — naming is emission-time only.
- Block-level names (`priority_encoder_0`, `comb_mux_N`) are
  deferred: today blocks decompose into gate chains at
  construction time with no IR-level block identity to attach a
  name to. Follow-up slice if needed.

---

## 2026-04-16-0033 — N-arity anti-collapse + OR-reduce dedup (Rule 8 extended)

**What changed**
- `src/gen/cone.rs`:
  - `violates_anti_collapse` now catches duplicates at any arity
    for idempotent / self-inverse operators (`And`, `Or`, `Xor`).
    Helper `has_duplicate_operand` does the operand-multiset
    distinctness check (O(N²), N bounded by `max_gate_arity`).
    `Add` and `Mul` deliberately remain exempt (duplicates are
    algebraically meaningful).
  - `or_reduce_terms` deduplicates input terms before building
    the 2-arity `Or` chain, so identical per-arm products do not
    produce `x | x` gates.
  - `make_none_selected` (QFeedback one-hot fall-through) now
    routes through `or_reduce_terms`, inheriting the dedup.
  - New unit test `anti_collapse_catches_nary_duplicates` pins
    the broadened check on Xor/And/Or at arities 3 and 4 (with
    and without duplicates) and confirms Add/Mul are not
    flagged.
- `book/src/structural-rules.md`: Rule 8 rewritten to state the
  N-arity rule explicitly; lists the exempt ops (Add, Mul); notes
  the downstream dedup in `or_reduce_terms` / `make_none_selected`.

**Why**
Sample module `mod_1_0000` contained `w_21 = i_2 ^ i_2 ^ i_2 ^ i_2`
(constant 0) and multiple identical one-hot arms producing
downstream `w_A | w_A` gates. The pairwise `operands[0] ==
operands[1]` check in the old `violates_anti_collapse` did not cover
these. Root-cause fix per the rule-based-generation doctrine.

**Tests**
- All four cargo gates green.
- 32 unit (+1 new) + 15 integration = **47 tests, all passing**.
- Spot-check across 8 seeds (1, 2, 3, 42, 100, 777, 9999, 12345):
  zero self-operand chains (`x OP x`) in generated SV. Seed 100
  previously emitted `w_120 = w_104 | w_104` from
  `make_none_selected`; now clean.

**Impact**
- Default config paths produce strictly higher-entropy gate
  operand lists: `And`/`Or`/`Xor` never repeat an operand. The
  pick-retry path absorbs the rare case where the picker
  re-selects a duplicate; after retry exhaustion it falls back to
  `pick_terminal`.
- No CLI or config-surface change.

---

## 2026-04-16-0032 — Dep-bearing source at elaboration-sensitive positions (Rule 20)

**What changed**
- `src/gen/cone.rs`: new `pick_terminal_dep_bearing(g, m, pool,
  width, exclude)` helper. Two-tier picker: (1) random dep-bearing
  matching-width pool entry; (2) width-adapter from widest
  dep-bearing pool entry of any width. Panics if the pool has no
  dep-bearing entry at all (invariant — primary inputs are always
  seeded with non-empty deps).
- Seven pool-mode call sites migrated from `pick_terminal` to
  `pick_terminal_dep_bearing`:
  - `grow_pool_one_unit`: const-shift value operand, const-comparand
    LHS.
  - `build_comb_mux_pool_only`: encoded sel, one-hot per-arm sel.
  - `drain_flop_worklist_pool_only`: encoded sel, one-hot per-arm
    sel.
  - `build_priority_encoder_pool`: request bits.
- New unit test `pick_terminal_dep_bearing_rejects_constants` (100
  iterations against a pool polluted with a matching-width
  dep-empty constant).
- `book/src/structural-rules.md`: new Rule 20 "Dep-bearing source
  required at elaboration-sensitive positions" with the four
  positions covered, motivation, and enforcement.

**Why**
Sample module `mod_1_0000` contained `w_35 = 2'h2 == 2'h2` — a
comparison with both operands literal, folding to a constant 1 at
elaboration. Root cause: the comparison's LHS picker
(`pick_terminal`) permits dep-empty pool entries in its tier-2
fallback and, at tier 4, emits a fresh constant. The same hazard
applies to mux selects and priority-encoder request bits. Fixed at
the root per the user's rule-based-generation doctrine.

**Tests**
- All four cargo gates green.
- 31 unit (+1 new) + 15 integration = **46 tests, all passing**.
- Spot-check across six seeds (1, 2, 3, 42, 100, 777): zero
  literal-vs-literal comparison lines in generated SV (was several
  per module before).

**Impact**
- Default config paths produce muxes, priority encoders, comparisons,
  and shifts whose selects / LHS / value sides are now always
  dep-bearing signals (primary input or flop Q, possibly adapted by
  Slice/Concat).
- No CLI or config-surface change.

---

## 2026-04-15-0031 — Coefficient fits operand width (Rule 19)

**What changed**
- `src/gen/cone.rs`:
  - `pick_coefficient(g)` → `pick_coefficient(g, width)`. The picker
    now narrows the draw range to
    `[max(min_coefficient, 1), min(max_coefficient, 2^W − 1)]` so a
    coefficient can never overflow the `W`-bit `Constant` node it
    will be emitted as.
  - `pick_mul_coefficient_and_arity(g)` → `(g, width)`, threads
    through.
  - All three callers (`assemble_add_linear_combination`,
    `assemble_sub_linear_combination`, the `Mul` arms of
    `build_linear_combination_recursive` /
    `build_linear_combination_pool`) pass their local `width`.
  - New unit test `pick_coefficient_respects_target_width` pins the
    width-aware clamp across widths 1, 2, 4, 8 (200 iterations).
- `book/src/structural-rules.md`: new Rule 19 "Coefficient fits
  operand width" with motivation, enforcement, and edge case
  (`width = 1` → coefficient is always 1).

**Why**
Sample module `mod_1_0000` contained `1'h6`, a 6 in a 1-bit literal —
the emitter truncates it to `1'h0`. Root cause: `pick_coefficient`
drew from `[min_coefficient, max_coefficient]` without reference to
the operand width. This slice fixes the bug at the root (the
picker) rather than with a post-hoc filter, per the user's "rule-
based generation" doctrine.

**Tests**
- All four cargo gates green.
- 30 unit (+1 new) + 15 integration = **45 tests, all passing**.

**Impact**
- Default config paths unaffected for `width ≥ 4` (unclamped range
  fits). Width-1 paths now emit `1'h1` constants instead of
  truncating larger values. Width-2/3 paths see slightly tighter
  distributions.
- No CLI or config-surface change (range knobs still exist and
  still accept their original values; clamping is silent per-call).

---

## 2026-04-15-0030 — Rule 18 proposal + sample-output defect catalogue (docs only)

**What changed**
- `book/src/structural-rules.md`: add Rule 18 "No orphan gates"
  (proposed, not yet enforced). Documents the rule, motivation,
  status, and the two enforcement paths under consideration:
  (α) construction-time demand-driven vs (β) emission-time
  tree-shake. Decision deferred.
- `DEVELOPMENT_NOTES.md`: new section "Generation-time defects
  observed in sample output (pending fixes)" cataloguing six
  concrete defects seen in sample module `mod_1_0000`:
  constant-select muxes, N-arity self-cancellation
  (`i_2^i_2^i_2^i_2 = 0`), coefficient width overflow (`1'h6`),
  dead wires, stranded flop (`r_3<=r_3`), structurally-identical
  one-hot arms. Attributes each to a root cause and sketches a
  fix. Three categories: anti-collapse operand-multiset, position-
  dependent leaf rules, width-aware coefficient generation — plus
  the orthogonal orphan-gate axis covered by Rule 18.

**Why**
User flagged the anomalies in a generated sample module and framed
the issue philosophically: "when block or gate is created it is
before it needs to be used, connected somewhere... some of those
signals are created with no proper reason." This slice captures the
observations so the next session can fix the defects at the root
rather than rediscovering them.

**Tests**
No code changed. `cargo fmt / build / clippy / test` unchanged from
the previous commit.

---

## 2026-04-15-0029 — Priority-encoder block (Rule 17)

**What changed**
- `src/config.rs`: new `priority_encoder_prob` knob (default 0.05).
  New `CoefficientRange`-style error handling for the probability
  (via the existing probability-range loop). Threaded through
  `Overrides` and `apply_cli_overrides`.
- `src/main.rs`: new CLI flag `--priority-encoder-prob`.
- `src/gen/cone.rs`:
  - `pick_priority_encoder_n(g, target_width) -> Option<u32>`: finds an
    N ∈ `[min_mux_arms, max_mux_arms]` with
    `ceil_log2(N) == target_width`. Returns None if none fits the range.
  - `assemble_priority_encoder(m, pool, target_width, req_bits) -> NodeId`:
    emits the chained ternary `req_0 ? 0 : req_1 ? 1 : ... : 0`.
    Every priority level becomes one `Mux` node; the output width is
    `target_width`.
  - `build_priority_encoder_recursive` / `build_priority_encoder_pool`:
    dispatch helpers that source request bits via `build_cone` or
    `pick_terminal` respectively.
  - Three dispatch sites (`build_cone`, `process_signal_frame`,
    `grow_pool_one_unit`) call the appropriate build helper. Dispatch
    has applicability-check-then-fall-through semantics: if no N
    matches the target width, the block roll is wasted and the code
    continues to the usual operator gate path.
- `tests/pipeline.rs`: new
  `priority_encoder_block_across_all_strategies_is_valid` — all four
  strategies × 5 seeds × `priority_encoder_prob = 1.0` must produce
  IR-valid modules. Uses `max_depth = 3` to bound test runtime under
  heavy PE recursion.
- `book/src/structural-rules.md`:
  - New Rule 17 describing the priority-encoder block: shape,
    applicability constraint (`ceil_log2(N) == W`), fall-through
    convention, and the place it lives in the generator.
  - Operators-vs-blocks preamble gains an entry for the priority-
    encoder block.
- `book/src/knobs.md`: new "Priority-encoder block" subsection.
- `USER_GUIDE.md`: `--priority-encoder-prob` row added.
- `CODEBASE_ANALYSIS.md`: `cone.rs` module map extended.
- `MEMORY.md`: last-completed-slice refreshed; next-up list
  re-scoped per user direction ("close all small-to-medium first")
  into case/casez → memories → FSMs → motif-trait refactor, with
  hierarchy / parameterization deferred.

**Why**
Per user direction to "close all small to medium first" ahead of
the large Phase 3+ items (hierarchy, parameterization). Priority
encoder is the smallest self-contained block motif on the list —
clean shape, single-output-width applicability check, no Q-feedback
or mux-style variant axes. Also a classic synthesizer idiom
(arbitration, interrupt-level encoding, one-hot-to-binary
conversion) worth exercising.

**Doctrinal note.** User observed mid-slice that every new block
follows the same pattern (knob, assembly helper, three dispatch
sites, tests, docs) and asked whether a motif library is feasible.
Agreed in principle; deferred until we have 6-8 concrete block
motifs to factor from (currently mux, flop-mux-family, comb-mux,
priority-encoder, with case/casez/memories/FSMs planned next).

**Validation**
- `cargo check --all-targets`, `cargo test` (29 unit + 15
  integration = 44 tests), `cargo clippy --all-targets -- -D
  warnings`, `cargo fmt --all --check`: all clean.
- End-to-end at `--priority-encoder-prob 1.0`: emitted SV contains
  chained ternaries like
  `assign w_18 = (w_3) ? (3'h0) : (w_16);`
  `assign w_16 = (w_3) ? (3'h1) : (w_14);`
  `... assign w_6 = (w_3) ? (3'h6) : (3'h0);` — full 7-level PE.

**Impact**
- First Phase 3+ block motif landed.
- Pattern for the remaining small-to-medium motifs (case/casez,
  memories, FSMs) is now well-established.

**Files touched**
`src/config.rs`, `src/main.rs`, `src/gen/cone.rs`, `tests/pipeline.rs`, `book/src/structural-rules.md`, `book/src/knobs.md`, `USER_GUIDE.md`, `CODEBASE_ANALYSIS.md`, `MEMORY.md`, `CHANGES.md`.

**Commit hash:** `b4c489a`

---

## 2026-04-15-0028 — Flop-assembler unit tests + FAQ chapter

**Commit hash:** `06b5a52`

**What changed**
- `src/gen/cone.rs`: 4 new inline unit tests for the flop-D assembly helpers.
  - `assemble_flop_d_one_hot_zero_default_top_is_or` — verifies the OneHot/ZeroDefault emission produces an OR-rooted tree of width W.
  - `assemble_flop_d_one_hot_qfeedback_includes_q_term` — verifies the QFeedback variant adds a Not for `~(OR of sels)` and preserves the OR root.
  - `assemble_flop_d_encoded_zero_default_top_is_mux` — verifies the Encoded/ZeroDefault chained-ternary top-level is a Mux of width W.
  - `assemble_flop_d_encoded_qfeedback_fallthrough_is_q` — verifies QFeedback+Encoded with M=2 still builds a Mux-rooted tree when index 0 is Q.
  - New test-fixture helpers `fixture_with_inputs` (n wide + n 1-bit primary inputs seeded into pool) and `alloc_flop` (register a flop + FlopQ node). Reusable across future flop-assembler tests.
- `book/src/faq.md` (new): 12-entry FAQ chapter answering vocabulary/doctrine questions that have come up during design discussion. Topics: Sub non-associativity, operators-vs-blocks vocabulary, Q-feedback-vs-combinational-no-loop, coefficient vs shift-amount vs comparand roles, four construction strategies rationale, cross-output sharing, reproducibility, testbench non-goal, synthesizability guarantee, "meaningful logic" disclaimer, SV language standard targeting, clk/rst_n port emission, version-to-version reproducibility, Verilator/Yosys invocation.
- `book/src/SUMMARY.md`: FAQ added to Reference section.
- `CODEBASE_ANALYSIS.md`: testing surface updated — 11 cone unit tests (was 7), 43 total (was 39).
- `MEMORY.md`: last-completed slice refreshed; next-up list re-scoped per user direction ("switch to Phase 3+ since Verilator unavailable") with 6 ranked candidate scopes.

**Why**
Per `MEMORY.md` next-up item (2): the flop-D assembly helpers were previously covered only indirectly by the pipeline integration sweep. Direct unit tests give faster feedback on their top-level shape invariants and pin the expected emission forms (OR root for OneHot, Mux root for Encoded, extra Not for QFeedback+OneHot). Tests are shape-level rather than exact-node-count to stay robust under future refactor.

The FAQ chapter consolidates the doctrine questions that accumulated across ~15 slices of vocabulary / design / rule-catalog work. It's the user-facing entry point for "why is `anvil` shaped this way" without having to dig through the structural-rules catalog or commit history.

**Validation**
- `cargo check --all-targets`, `cargo test` (29 unit + 14 integration = 43 tests), `cargo clippy --all-targets -- -D warnings`, `cargo fmt --all --check`: all clean.
- `mdbook build book` succeeds with the new FAQ chapter rendered.

**Impact**
- Flop-assembler regressions now caught at the unit level, not just when the pipeline sweep happens to fail.
- Book gains a welcoming "why" entry point. Users arriving cold have a fast path to understanding anvil's doctrine without reading the full structural-rules catalog.

**Files touched**
`src/gen/cone.rs`, `book/src/faq.md` (new), `book/src/SUMMARY.md`, `CODEBASE_ANALYSIS.md`, `MEMORY.md`, `CHANGES.md`.

**Commit hash:** `06b5a52`

---

## 2026-04-15-0027 — Constant comparand motif: third and final constant-role motif

**Commit hash:** `1211120`

**What changed**
- `src/config.rs`: three new knobs.
  - `const_comparand_prob` (default 0.3): per-comparison probability the RHS is a constant literal instead of a recursive signal cone.
  - `min_comparand` (0), `max_comparand` (255): value range, clamped to `[0, 2^K - 1]` for the chosen internal operand width K.
  - Threaded through `Overrides`, `apply_cli_overrides`, and the probability-range validation loop.
- `src/main.rs`: three new CLI flags.
- `src/gen/cone.rs`:
  - New helpers: `pick_comparison_operand_width` (matches `input_widths_for`'s 1..=8 draw), `pick_comparand_value` (clamped range draw), `build_comparison_const_comparand` (emits `lhs_signal OP const` with 1-bit output), `is_comparison_op` (predicate).
  - Three dispatch sites after `pick_gate` returns a comparison op: `build_cone` (recursive LHS), `process_signal_frame` (interleaved recursive LHS), `grow_pool_one_unit` (graph-first pool-only LHS).
- `tests/pipeline.rs`: new `const_comparand_across_all_strategies_is_valid` — `const_comparand_prob = 1.0` × all four strategies × 5 seeds, all IR-valid.
- `book/src/structural-rules.md`: "Roles of constants in RTL" → Comparand subsection updated. Previous "Not yet emitted by anvil" note retired.
- `book/src/knobs.md`: new "Comparand motif" subsection.
- `USER_GUIDE.md`: three new CLI flag rows.
- `CODEBASE_ANALYSIS.md`: `cone.rs` module map extended.
- `MEMORY.md`: current-state refreshed; next-up list trimmed (all three constant-role motifs done). Recent commits list gains `2da9d3d`.

**Why**
Third and final constant-role motif from the catalog. Comparisons in real RTL frequently have constant RHS (`state == IDLE`, `counter >= LIMIT`, `error_code != 0`) — a threshold / sentinel / target pattern, not a coefficient. Per the vocabulary-discipline doctrine the motif has its own knob family, distinct from coefficients and shift amounts.

The motif is *additive* to signal-vs-signal comparisons: when the coin doesn't fire, the existing path emits both operands as recursive signals. Users who want purely-signal comparisons pin `--const-comparand-prob 0.0`; users who want threshold-stress pin `1.0`.

**Validation**
- `cargo check --all-targets`, `cargo test` (25 unit + 14 integration = 39 tests), `cargo clippy --all-targets -- -D warnings`, `cargo fmt --all --check`: all clean.
- End-to-end: on seed 1 with `--const-comparand-prob 1.0 --max-width 1`, emitted SV contains `assign w_10 = w_7 == 2'h3;`, `assign w_24 = w_22 == 8'h7a;`, etc.

**Impact**
- All three constant-role motifs implemented. The generator now emits realistic RTL idioms across the three semantic roles for integer literals.
- Phase 1/2 feature work is effectively done; remaining exit gate is the Verilator-lint smoke run across construction strategies and motif probabilities.

**Files touched**
`src/config.rs`, `src/main.rs`, `src/gen/cone.rs`, `tests/pipeline.rs`, `book/src/structural-rules.md`, `book/src/knobs.md`, `USER_GUIDE.md`, `CODEBASE_ANALYSIS.md`, `MEMORY.md`, `CHANGES.md`.

---

## 2026-04-15-0026 — Constant shift-amount motif + Shl/Shr added to pick_gate

**Commit hash:** `2da9d3d`

**What changed**
- `src/config.rs`: four new knobs.
  - `const_shift_amount_prob` (default 0.8): per-shift probability the amount operand is a constant literal instead of a variable-amount signal (barrel shifter).
  - `min_shift_amount` (default 0) / `max_shift_amount` (default 7): range for the drawn constant amount, clamped to `[0, W-1]` for a W-bit value.
  - `gate_shift_weight` (default 1): relative weight for the shifts bucket in `pick_gate`.
  - Threaded through `Overrides`, `apply_cli_overrides`, and the probability-range validation loop.
- `src/main.rs`: four new CLI flags.
- `src/gen/cone.rs`:
  - `pick_gate` now has a fifth bucket (`shifts: &[Shl, Shr]`) with weight `gate_shift_weight`. Shifts are disabled at `target_width == 1` (shift on a 1-bit value is trivial).
  - New helpers: `pick_shift_amount` (draws from `[min_shift_amount, max_shift_amount]` clamped to `[0, value_width-1]`), `build_shift_const_amount` (emits `value OP const` — a single 2-operand Shl/Shr node with a compact-width constant).
  - Three dispatch sites after `pick_gate` returns Shl/Shr:
    - `build_cone` (sequential / shuffled / interleaved-block-internal paths): value from recursive `build_cone`.
    - `process_signal_frame` (interleaved top-level): value from recursive `build_cone` at `depth+1`.
    - `grow_pool_one_unit` (graph-first): value from `pick_terminal`.
- `tests/pipeline.rs`: new `const_shift_amount_appears_in_output` — 32-seed sweep at `const_shift_amount_prob = 1.0, gate_shift_weight = 10` must produce at least one `<< N'hX` or `>> N'hX` emission.
- `book/src/structural-rules.md`: "Roles of constants in RTL" → Shift Amount subsection updated with the knob list and the implementation site; previous "today always variable-amount" note retired.
- `book/src/knobs.md`: new "Shift-amount motif" subsection.
- `USER_GUIDE.md`: four new CLI flag rows.
- `CODEBASE_ANALYSIS.md`: `cone.rs` module map extended.
- `MEMORY.md` / `CHANGES.md`: per workflow.

**Why**
Per MEMORY next-up item 1 and the roles-of-constants doctrine. Shifts in real RTL are predominantly constant-amount (wire reroutes, cheap) rather than variable-amount barrel shifters. The default probability is set high (0.8) to match that prevalence; users wanting to stress barrel-shifter synthesis can lower it to 0.0 for purely variable amounts.

Adding Shl/Shr to `pick_gate` fixes a longstanding absence — the shifts were defined in `GateOp` and `input_widths_for` but never selectable. Same pattern as the earlier Mul fix (slice `2026-04-15-0025`).

The knob set is its own family — distinct from `coefficient_prob` and (future) `const_comparand_prob`. Per the vocabulary-discipline doctrine, "shift amount" is a structural parameter, not a coefficient.

**Validation**
- `cargo check --all-targets`, `cargo test` (25 unit + 13 integration = 38 tests), `cargo clippy --all-targets -- -D warnings`, `cargo fmt --all --check`: all clean.
- End-to-end spot check at `--const-shift-amount-prob 1.0`: emitted SV contains `w_X >> 3'h5`, `w_X << 2'h3`, `w_X << 1'h0`, etc. Both operator directions and a range of amounts observed.

**Impact**
- Generated RTL now routinely includes constant-amount shifts — the dominant pattern in real datapaths (scaling by powers of two, alignment, field extraction).
- Barrel-shifter stress is still reachable by pinning `--const-shift-amount-prob 0.0`.
- Two of three constant-role motifs now implemented (coefficients ✅, shift amounts ✅); comparands remain.

**Files touched**
`src/config.rs`, `src/main.rs`, `src/gen/cone.rs`, `tests/pipeline.rs`, `book/src/structural-rules.md`, `book/src/knobs.md`, `USER_GUIDE.md`, `CODEBASE_ANALYSIS.md`, `MEMORY.md`, `CHANGES.md`.

---

## 2026-04-15-0025 — Linear-combination coefficient motif for Add / Sub / Mul

**Commit hash:** `7290e3d`

**What changed**
- `src/config.rs`: three new knobs —
  - `coefficient_prob` (default 0.2): per-op probability that Add/Sub/Mul emits the linear-combination compound form instead of a standard operator.
  - `min_coefficient` (default 1) and `max_coefficient` (default 15): strictly-positive integer coefficient range.
  - New `ConfigError::CoefficientRange`; validation enforces `1 <= min <= max`.
  - Overrides and `apply_cli_overrides` wired for all three.
- `src/main.rs`: CLI flags `--coefficient-prob`, `--min-coefficient`, `--max-coefficient`.
- `src/gen/cone.rs`:
  - Added `Mul` to `pick_gate`'s arith bucket (was absent — so the Mul compound could never previously fire).
  - New helpers: `make_mul`, `make_sub`, `make_nary_add`, `make_nary_mul`, `pick_coefficient`, `pick_linear_combination_arity`, `pick_mul_coefficient_and_arity`.
  - New assemblers: `assemble_add_linear_combination` (N Mul + one N-ary Add), `assemble_sub_linear_combination` (N Mul + N-1 chained 2-ary Sub), `assemble_mul_linear_combination` (one N+1-ary Mul with leading constant).
  - New dispatchers: `build_linear_combination_recursive` (signals via `build_cone`) and `build_linear_combination_pool` (signals via `pick_terminal`).
  - Three dispatch sites inserted:
    - `build_cone` (sequential/shuffled paths): after `pick_gate`, before operand loop.
    - `process_signal_frame` (interleaved): after `pick_gate`, before frame/operand enqueue. Compound tree built synchronously within the frame step (like blocks).
    - `grow_pool_one_unit` (graph-first): after `pick_gate`, before standard operand pool-pick loop.
- `tests/pipeline.rs`:
  - New `coefficient_motif_emits_compound_shapes`: 16-seed sweep at `coefficient_prob=1.0` must produce at least one front-constant Mul expression (`<W>'h<hex> * w_...`).
  - New `coefficient_motif_across_all_strategies`: all four construction strategies × 5 seeds × `coefficient_prob=1.0` must produce valid IR.
- `book/src/structural-rules.md` "Roles of constants in RTL" → Coefficient subsection updated with:
  - Mul shape `y = c * s1 * s2 * … * sn` spelled out.
  - `c >= 1`; `c == 1` forces `N >= 2`.
  - Knob list (`coefficient_prob`, `min_coefficient`, `max_coefficient`).
- `book/src/knobs.md`: new "Coefficient motif (linear combinations)" subsection.
- `USER_GUIDE.md`: three new CLI flag rows.
- `CODEBASE_ANALYSIS.md`: `cone.rs` module map extended with the new helpers, assemblers, and dispatchers.
- `MEMORY.md`: last-completed-slice refreshed; next-up list trimmed — coefficient motif done, shift-amount bias is now item 1.

**Why**
Per user direction: arithmetic operators benefit from a compound linear-combination motif that emits realistic RTL idioms (`3*a + 2*b + c` for Add, `a*5 - b*3` for Sub, `c * s1 * s2 * s3` for Mul). Constants in this role are **coefficients** (multiplicative weights), distinct from shift amounts or comparands. Per-op constraints:
- Add: `ci ≠ 0` (non-zero). Implementation uses positive-only.
- Sub: `ci > 0` (strictly positive). Negative would flip to Add contribution.
- Mul: single `c >= 1` scalar multiplier; `c == 1` forces `N >= 2` to avoid the dead `1 * s1 = s1` case.

This lands the first of three constant-role motifs (coefficients → shift amounts → comparands) the project committed to. Each has its own knob family per "do not collapse into a single `constant_prob` knob" doctrine.

**Validation**
- `cargo check --all-targets`, `cargo test` (25 unit + 12 integration = 37 tests), `cargo clippy --all-targets -- -D warnings`, `cargo fmt --all --check`: all clean.
- End-to-end spot check at `--coefficient-prob 1.0`:
  - `assign w_9 = 11'h2 * w_7 * w_7;` (Mul compound, c=2, 2 signals)
  - `assign w_12 = 26'hc * w_10 * w_10 * w_10 * w_10;` (Mul compound, c=12, 4 signals)
  - `assign w_49 = 5'hf * w_30 * w_32 * w_47;` (Mul compound, c=15, 3 distinct signals)
  - `assign w_32 = w_30 * 5'h8; w_34 = w_30 * 5'hc; ... w_44 = w_39 + w_41 + w_43;` (Add compound terms + N-ary sum)
  - `assign w_22 = w_15 - w_17; w_23 = w_22 - w_19; w_24 = w_23 - w_21;` (Sub compound chain)

**Impact**
- Generated RTL now routinely contains realistic arithmetic datapath idioms (scaled-sum accumulators, weighted differences, product chains with constant multipliers).
- `Mul` is now selectable by `pick_gate` (previously omitted from the menu). This also means the non-coefficient Mul path can now emit binary multipliers even when the coefficient motif doesn't fire.
- Three planned constant-role motifs: one done, two to go (shift amounts, comparands).

**Known simplification**
- Add's theoretical `ci ≠ 0` allows negative coefficients; the implementation draws positive-only from `[min_coefficient, max_coefficient]`. Signed-negative coefficients are a future extension. Sub's `ci > 0` and Mul's `c >= 1` are honored exactly.

**Files touched**
`src/config.rs`, `src/main.rs`, `src/gen/cone.rs`, `tests/pipeline.rs`, `book/src/structural-rules.md`, `book/src/knobs.md`, `USER_GUIDE.md`, `CODEBASE_ANALYSIS.md`, `MEMORY.md`, `CHANGES.md`.

---

## 2026-04-15-0024 — Sub coefficient constraint: `ck > 0` for all k

**Commit hash:** `b0f84fd`

**What changed**
- `book/src/structural-rules.md` "Roles of constants in RTL" → Coefficient subsection: expanded with per-op shapes and constraints.
  - Add: `y = s1*c1 + ... + sn*cn`, `ci ≠ 0` for all i (non-zero; positive or negative both legal).
  - Sub: `y = s1*c1 - s2*c2 - ... - sn*cn` (left-associative), **`ci > 0` for all i** (strictly positive). Rationale in-line: a negative `ci` on a `- sk*ck` term flips to `+ sk*|ck|` — an Add contribution disguised as a Sub term. Zero kills the term. Strictly positive preserves subtractive character.
  - Mul: shape + constraints TBD (pending user spec).
- `MEMORY.md` next-up item 1 rewritten to carry the per-op constraints, not just the Add shape.
- `DEVELOPMENT_NOTES.md` "Roles of constants in RTL" core-decision entry extended with the per-op constraint summary.

**Why**
User: "This 'Linear-combination ADD motif' shall also be true for SUB too. ck > 0 for all k." The distinction between Add's `ci ≠ 0` (non-zero) and Sub's `ci > 0` (strictly positive) is semantic, not arbitrary — negative coefficients inside a subtractive chain mean the term is an Add contribution rather than a Sub one, which defeats the purpose of generating a Sub-shaped motif.

Logging the clarification now so the next-up motif slice implements the correct per-op constraints without rediscovering them.

**Validation**
- Documentation-only slice; no source touched.
- `cargo check`, `cargo test` (35 tests), `cargo clippy --all-targets -- -D warnings`, `cargo fmt --all --check`: all still clean.

**Impact**
- Coefficient motif implementation now has precise per-op specs for Add and Sub ahead of implementation. Mul remains under-specified until you weigh in.
- Structural-rules catalog's coefficient section is now the durable reference for the per-op constraint set.

**Files touched**
`book/src/structural-rules.md`, `MEMORY.md`, `DEVELOPMENT_NOTES.md`, `CHANGES.md`.

---

## 2026-04-15-0023 — `graph-first` strategy landed, becomes the new default

**Commit hash:** `4085401`

**What changed**
- `src/config.rs`:
  - New `ConstructionStrategy::GraphFirst` variant.
  - New `graph_first_pool_size: u32` field on `Config` (default 32). Target number of top-level units (operator gate / flop / comb-mux block) to emit during pool growth. Does not count internal primitives generated by comb-mux or flop-mux assembly.
  - **Default flipped:** `Config::default().construction_strategy = GraphFirst`.
  - Overrides and `apply_cli_overrides` wired.
- `src/main.rs`: new CLI flags `--construction-strategy graph-first` (variant visible) and `--graph-first-pool-size`.
- `src/gen/cone.rs`: new `build_graph_first` entry point plus three helpers:
  - `grow_pool_one_unit`: emits one top-level unit. Picks flop (with deferred D) / comb-mux block / operator gate according to the usual probabilities. Operator-gate operands come from `pick_terminal` (no recursion). Anti-collapse retry up to 4× then skip. Returns a boolean indicating success; caller loops with an iteration budget of `8 × pool_size` to prevent pathological infinite loops.
  - `build_comb_mux_pool_only`: mirrors `build_comb_mux` but data/sel operands are pool picks. Reuses `replicate_to_width`, `make_and`, `or_reduce_terms`, `make_eq_const`, `make_mux`, `make_constant`.
  - `drain_flop_worklist_pool_only`: mirrors `drain_flop_worklist` but every data / select / direct-D sub-cone is a pool pick. Reuses `assemble_flop_d_one_hot` and `assemble_flop_d_encoded`.
- `src/gen/module.rs`: strategy dispatch updated — `GraphFirst` arm delegates to `cone::build_graph_first`. Subsequent `cone::drain_flop_worklist` is a no-op for GraphFirst (worklist already drained via the pool-only variant).
- `tests/pipeline.rs`:
  - `all_strategies_produce_valid_modules` extended to cover `GraphFirst`.
  - New `graph_first_is_default` — omitting `--construction-strategy` produces byte-identical output to `--construction-strategy graph-first`.
  - New `graph_first_reproducibility` — same seed + GraphFirst = byte-identical output twice.
  - New `graph_first_differs_from_sequential` — on a 3-output seed, GraphFirst produces different SV than Sequential.
- `book/src/construction-strategies.md`: implementation status table updated — all four strategies implemented; `graph-first` marked **default**. Top-of-chapter text updated to reflect that `graph-first` is current default. Implementation-sequence prose updated.
- `book/src/knobs.md`: construction-strategy subsection rewritten — all four values listed, `graph-first` marked default, `graph_first_pool_size` knob documented.
- `USER_GUIDE.md`: CLI table updated — `--construction-strategy` default flipped to `graph-first`, new `--graph-first-pool-size` row.
- `CODEBASE_ANALYSIS.md`: `config.rs` and `cone.rs` entries extended; `module.rs` dispatch documented to include GraphFirst.
- `MEMORY.md`: last-completed-slice refreshed; next-up reorganized — all construction-strategy work items removed (done); remaining items are motif slices (coefficients, shift-amount bias, comparands) + Verilator-lint smoke. Recent-commits list gains `6d2da98`.

**Why**
Per user direction: `graph-first` is the correct default because the user-visible output is a DAG, not a union of per-output cones. The cone-per-output construction idiom is a human-friendly fiction; `graph-first` drops it in favor of growing a gate pool with no output attribution and picking drive-roots from the pool. Sharing is truly symmetric including through block internals (flop D-cones, comb-mux sub-cones) — a property none of the prior strategies achieves.

Landing `graph-first` completes the four-strategy commitment. Users who want the old `sequential` behavior pin it explicitly via `--construction-strategy sequential`.

**Validation**
- `cargo check --all-targets`, `cargo test` (25 unit + 10 integration = 35 tests), `cargo clippy --all-targets -- -D warnings`, `cargo fmt --all --check`: all clean.
- End-to-end: `cargo run -- --seed 42 --min-outputs 3 --max-outputs 3` (default knobs) produces identical output to `cargo run -- ... --construction-strategy graph-first`, confirming the default flip. Diffing against `--construction-strategy sequential` on the same seed shows different module shape (different flop widths, different gate structure, different pool entries) — the strategy knob is load-bearing.

**Impact**
- Four construction strategies implemented; `graph-first` is now the default behavior of `anvil`.
- True module-wide symmetric sharing for the default strategy: every data / select / direct-D sub-cone — whether in an output cone, a flop D, or a comb-mux — is picked from the same fully-grown pool.
- Reproducibility preserved for prior-generated output by pinning the strategy at invocation time and recording effective knobs in the manifest.
- The construction-strategy work item from the last seven slices' next-up queues is complete.

**Files touched**
`src/config.rs`, `src/main.rs`, `src/gen/cone.rs`, `src/gen/module.rs`, `tests/pipeline.rs`, `book/src/construction-strategies.md`, `book/src/knobs.md`, `USER_GUIDE.md`, `CODEBASE_ANALYSIS.md`, `MEMORY.md`, `CHANGES.md`.

---

## 2026-04-15-0022 — `interleaved` construction strategy: frame state machine

**Commit hash:** `6d2da98`

**What changed**
- `src/config.rs`: new `ConstructionStrategy::Interleaved` variant.
- `src/gen/cone.rs`: new frame state machine at module level.
  - Types: `Dest` (Output(idx) | Slot { frame_id, slot }), `SignalFrame` (pending signal construction at given width/depth/exclude with a Dest), `GateFrame` (in-flight gate waiting on N operand slots).
  - Public entry: `build_outputs_interleaved(g, m, pool, worklist) -> Vec<NodeId>`. Seeds the queue with one `SignalFrame` per output, pops a random frame each step, processes it.
  - `process_signal_frame`: mirrors `build_cone`'s decision tree. force_leaf → `pick_terminal` → deliver. Flop → `build_flop_leaf` (synchronous block) → deliver. Comb-mux → `build_comb_mux` (synchronous block) → deliver. Operator gate → allocate a `GateFrame` in the in-flight table + enqueue N child `SignalFrame`s (with per-operand share check reusing `try_share`).
  - `deliver`: writes the resolved NodeId to the Dest. For `Slot`, decrements pending; when pending hits 0, the `GateFrame` fires (same anti-collapse check, same dep-set union, same pool.add), and its own Dest is then resolved (recursively).
- `src/gen/module.rs`: `generate_leaf_module` dispatches on strategy. For `Interleaved`, delegates to `cone::build_outputs_interleaved`. For `Sequential` / `Shuffled`, uses the existing recursive `build_cone_with_retry` path.
- `tests/pipeline.rs`:
  - `all_strategies_produce_valid_modules` extended to cover `Interleaved`.
  - New `interleaved_reproducibility` — same seed + Interleaved = byte-identical output twice.
  - New `interleaved_differs_from_sequential` — on a 3-output seed, the emitted SV differs between strategies.
- `book/src/construction-strategies.md` and `book/src/knobs.md`: implementation status flipped to "implemented" for `interleaved`; scope note clarifying that block internals are not interleaved in this slice (only output-cone frames).
- `USER_GUIDE.md`: `--construction-strategy` row updated to list `interleaved` as supported.
- `CODEBASE_ANALYSIS.md`: `cone.rs` module map expanded to document the frame machine; `config.rs` lists the three variants; `module.rs` describes the dispatch.
- `MEMORY.md` / `CHANGES.md`: per workflow.

**Why**
Per the user's direction that construction-order asymmetry is a construction artifact and not a design property, `interleaved` was the next milestone after `shuffled`. The frame state machine achieves near-symmetric per-module sharing for output-cone construction: by the time any given cone picks its deeper leaves, many other cones have already contributed gates to the pool. Declaration-order bias is gone; within-module ordering is still present for *block internals* (flop D-cones, comb-mux sub-cones built depth-first within one frame step) but much weaker than in `sequential` or `shuffled`.

Scope was deliberately kept to output-cone frames — block internals remain synchronous — because folding blocks into the frame machine adds meaningful complexity without buying proportional symmetry (flop Qs enter the pool when flops are allocated, so cross-flop sharing works regardless). Full symmetry awaits `graph-first`.

**Validation**
- `cargo check --all-targets`, `cargo test` (25 unit + 7 integration = 32 tests), `cargo clippy --all-targets -- -D warnings`, `cargo fmt --all --check`: all clean.
- End-to-end: `cargo run -- --seed 42 --min-outputs 3 --max-outputs 3 --construction-strategy interleaved` produces a valid ~4k-line module. Diffing against `sequential` on the same seed shows different internal shape; IR validator accepts both.

**Impact**
- Three of four construction strategies now implemented.
- Users can pick `interleaved` for realistic cross-output sharing without waiting for `graph-first`.
- `build_outputs_interleaved` is a self-contained alternative entry point; the recursive `build_cone` path is untouched.

**Known limitations (documented)**
- Block internals (flop D-cones, comb-mux sub-cones) still build depth-first. Full symmetry including blocks is the `graph-first` target.
- The `interleaved` path does not have a retry-on-trivial mechanism equivalent to `build_cone_with_retry`. If an output cone ends up with an empty dep-set it will fail validation. In practice this has not been observed under default knobs; the validator catches it if it happens.

**Files touched**
`src/config.rs`, `src/gen/cone.rs`, `src/gen/module.rs`, `tests/pipeline.rs`, `book/src/construction-strategies.md`, `book/src/knobs.md`, `USER_GUIDE.md`, `CODEBASE_ANALYSIS.md`, `MEMORY.md`, `CHANGES.md`.

---

## 2026-04-15-0021 — Construction-strategy machinery + `shuffled` strategy landed

**Commit hash:** `2d038a9`

**What changed**
- `src/config.rs`: new `ConstructionStrategy` enum with `Sequential` and `Shuffled` variants. Derives `Serialize`, `Deserialize`, `clap::ValueEnum`; uses kebab-case for both serde and clap. New `Config.construction_strategy` field (default `Sequential`). Threaded through `Overrides` and `apply_cli_overrides`.
- `src/main.rs`: new CLI flag `--construction-strategy <sequential|shuffled>` via `value_enum`. Imports the enum through the public `anvil::config::ConstructionStrategy` path.
- `src/gen/module.rs`: `generate_leaf_module` dispatches on `cfg.construction_strategy`.
  - `Sequential`: outputs built in declaration order (`0, 1, ..., n_out-1`). Current behavior preserved exactly.
  - `Shuffled`: a random permutation of the output indices is drawn from the seeded RNG and used as the build order.
  - Either way, drives are recorded in declaration order in `m.drives`, so SV port/assign emission is unaffected by build order. Only *which pool state each output's leaf-selection sees* changes.
- `tests/pipeline.rs`: three new integration tests.
  - `shuffled_reproducibility` — same seed + `Shuffled` strategy produces byte-identical output twice.
  - `shuffled_differs_from_sequential` — on a 4-output seed, `Shuffled` produces different SV than `Sequential`, confirming the knob actually changes output.
  - `all_strategies_produce_valid_modules` — both strategies × 10 seeds = 20 modules all pass `ir::validate`.
- `book/src/construction-strategies.md`: "Implementation status" section updated — `sequential` and `shuffled` now marked implemented; `interleaved` and `graph-first` still planned.
- `book/src/knobs.md`: new "Construction strategy" subsection documenting the knob and its values.
- `USER_GUIDE.md`: `--construction-strategy` added to the CLI flags table.
- `CODEBASE_ANALYSIS.md`: module-map entries for `config.rs` and `gen/module.rs` updated to reflect the new enum and the strategy-dispatching build-order logic.
- `MEMORY.md`: next-up list retires the "add the machinery" and "land shuffled" items; items 1 and 2 are now `interleaved` and `graph-first` respectively. Current-state snapshot refreshed. Recent commits list gains `8eb03f0`.

**Why**
User said the asymmetry of sequential declaration-order construction is a construction artifact, not a design property, and asked for all four strategies supported with `graph-first` as the eventual default. This slice lands the knob infrastructure plus the cheapest of the four alternative strategies (`Shuffled`), giving an immediate user-visible improvement (declaration-order bias removed) without the architectural rewrite that `Interleaved` / `GraphFirst` require.

Landing `Sequential` + `Shuffled` together in one slice is one coherent task — the knob has at least one non-trivial value from day one, rather than being a placeholder with only a single option. Future slices add `Interleaved` and then `GraphFirst` + default-flip.

**Validation**
- `cargo check --all-targets`, `cargo test` (25 unit + 5 integration = 30 tests), `cargo clippy --all-targets -- -D warnings`, `cargo fmt --all --check`: all clean.
- `cargo run -- --help` shows `--construction-strategy <CONSTRUCTION_STRATEGY>` with the clap-generated description.
- End-to-end: `cargo run -- --seed 42 --min-outputs 4 --max-outputs 4 --construction-strategy shuffled` produces a valid module; diffing against the `sequential` run on the same seed shows different internal shape (different gates, different sharing pattern) while the port list remains in declaration order.

**Impact**
- Users can now pick between `sequential` and `shuffled` at the CLI. The declaration-order bias is no longer mandatory.
- The knob scaffolding is in place for the two remaining strategies; adding them is a matter of extending the `ConstructionStrategy` enum and adding a match arm in `generate_leaf_module` (plus for `graph-first`, a new `build_module_graph_first` path).

**Files touched**
`src/config.rs`, `src/main.rs`, `src/gen/module.rs`, `tests/pipeline.rs`, `book/src/construction-strategies.md`, `book/src/knobs.md`, `USER_GUIDE.md`, `CODEBASE_ANALYSIS.md`, `MEMORY.md`, `CHANGES.md`.

---

## 2026-04-15-0020 — Construction-strategies chapter: 4 named strategies, graph-first planned default

**Commit hash:** `8eb03f0`

**What changed**
- **NEW `book/src/construction-strategies.md`**. Dedicated chapter under "How It Works" documenting four named strategies for module construction:
  - **`sequential`** — current behavior: per-output cone recursion in declaration order. Baseline; has declaration-order bias and within-module ordering asymmetry.
  - **`shuffled`** — per-output cone recursion in a random permutation of declaration order. Removes declaration-order bias; within-module asymmetry randomized per seed.
  - **`interleaved`** — frames from all cones interleaved via a random-pop work queue; cones grow in lockstep. Near-symmetric within-module sharing.
  - **`graph-first`** — no per-output cone recursion at all. Grow a gate pool with no output attribution; pick drive-roots from the pool at the end. True symmetric sharing. **Planned default** once implementation lands.
  Chapter covers: why this is a knob (it shapes the output distribution), per-strategy complexity and tradeoffs, a comparison table, rule-interaction summary (Rules 1, 9, 16 all preserved across strategies), and implementation status.
- `book/src/SUMMARY.md`: new chapter added under "How It Works" after `algorithm.md`.
- `book/src/algorithm.md`: strategy note near the top referencing the new chapter so readers know the pseudocode describes `sequential` specifically.
- `book/src/sharing.md`: cross-output sharing section updated to call out the sequential-order asymmetry as a construction artifact and point to the new chapter.
- `MEMORY.md`: next-up list reorganized. Construction-strategies machinery is now item 1 (land the knob and implement sequencing); the motif slices (coefficients / shift-amount bias / comparands) follow. Recent-commits list gains `126411d`.
- `DEVELOPMENT_NOTES.md`: new core design decision entry "Construction strategies" pointing to the book chapter. Captures the load-bearing framing: strategy is how-we-build, not what-we-emit; each strategy has its own output distribution properties.

**Why**
User flagged that declaration-order asymmetry is a construction artifact, not a design property, and asked for true symmetric sharing. The discussion surfaced three alternatives (shuffled / interleaved / graph-first). User then noted the current behavior deserves a name too — hence four strategies, not three.

The chapter codifies all four as a first-class design choice: what strategy the generator uses is a *per-run knob*, not a hidden implementation detail. Users who want reproducibility of prior outputs pin to `sequential`; users who want maximum realistic sharing use `graph-first` (the planned default). The knob stays unimplemented until the machinery lands, but the doctrine is now fixed.

User's choice of `graph-first` as the default is aligned with the project's overall framing (think in terms of the object — a DAG — not the construction order). `sequential` and `shuffled` keep a per-output-cone construction idiom that is a human-friendly fiction; `graph-first` drops the fiction in favor of the DAG.

**Validation**
- Documentation-only slice; no source touched.
- `mdbook build book` succeeds with the new chapter rendered.
- `cargo check`, `cargo test` (27 tests), `cargo clippy --all-targets -- -D warnings`, `cargo fmt --all --check`: all still clean.

**Impact**
- The book now documents an explicit four-way knob that governs a major axis of generator behavior. Future sessions have clear guidance on the implementation sequence and which strategy becomes default.
- The cone-per-output construction idiom remains valid for `sequential`/`shuffled`/`interleaved` but is explicitly retrospective (not construction-time) for `graph-first`. This is doctrine now, not just my preference.

**Files touched**
`book/src/construction-strategies.md` (new), `book/src/SUMMARY.md`, `book/src/algorithm.md`, `book/src/sharing.md`, `MEMORY.md`, `DEVELOPMENT_NOTES.md`, `CHANGES.md`.

---

## 2026-04-15-0019 — Rule 16: cross-output sharing via the module-wide signal pool

**Commit hash:** `126411d`

**What changed**
- `book/src/structural-rules.md`: new Rule 16 "Cross-output sharing via the module-wide signal pool". States that there is no per-output isolation — gates built while constructing output A's cone are immediately available as leaves / DAG-sharing candidates in output B's cone and in every flop's D-cone. Calls out the ordering asymmetry (outputs built in declaration order; later outputs see more sharing candidates) and the combinational-no-loop preservation (Rule 1 holds cross-cone because arena-index monotonicity is module-wide, not per cone).
- "Operators vs blocks" preamble's grouping list updated with a "Module-wide sharing: Rule 16" entry.
- `book/src/sharing.md`: new "Cross-output and cross-cone sharing" section that names the behavior and points to Rule 16.

**Why**
User flagged: "Nodes inside the fanin cone of one top level output can be used as inputs of gates/blocks in the fanin cone of another top level output. I guess you are already allowing that." The behavior was already in place (the `SignalPool` is constructed once per module and shared across all cone builds), but it was implicit — a reader would have to infer it from the code rather than find it in the rule catalog. Making it Rule 16 closes the gap.

The ordering asymmetry (output 0 sees fewer candidates than output N-1) is worth documenting explicitly so a reader isn't surprised when output 0 tends to have more standalone logic than later outputs.

**Validation**
- Documentation-only slice; no source touched.
- `cargo check`, `cargo test` (27 tests), `cargo clippy --all-targets -- -D warnings`, `cargo fmt --all --check`: all still clean.
- Behavior claim verified against code: `src/gen/module.rs::generate_leaf_module` constructs exactly one `SignalPool` and threads it by `&mut` through every `build_cone_with_retry` call; `src/gen/cone.rs::pick_terminal` and `try_share` iterate the pool with no cone-identity filter.

**Impact**
- The structural rules catalog is more complete. A reader coming cold can now see explicitly that the generator does not isolate output cones from each other.
- The book's sharing chapter now points to Rule 16 for the authoritative statement.

**Files touched**
`book/src/structural-rules.md`, `book/src/sharing.md`, `MEMORY.md`, `CHANGES.md`.

---

## 2026-04-15-0018 — Log the constants-roles clarification in the book + two corrections

**Commit hash:** `8ff1d84`

**What changed**
- `book/src/structural-rules.md`: added a new "Roles of constants in RTL" section to the preamble (right after "Operators vs blocks"). Three distinct roles — coefficient, shift amount, comparand — each with its own scope, constraints, and motif family. Explicitly lists why flattening them into a single mechanism would break the semantic structure.
- Within that new section, two corrections the user surfaced:
  - **Shifts:** both variable-amount (`a << count` with `count` a signal) and constant-amount (`a << 2`) are legal SV. `anvil` today always emits variable-amount; real designs overwhelmingly use constant. A bias knob is on the roadmap so defaults match prevalence. Both modes coexist.
  - **Comparisons:** the RHS of a comparison can be *another signal* (signal-vs-signal, the default today) OR a *constant comparand* (threshold/sentinel pattern). The comparand motif is *additive* — it does not replace signal-vs-signal comparisons.
- `MEMORY.md` next-up list rewritten to reflect both corrections precisely:
  - Shift-motif next-up is now framed as a constant-vs-variable bias (not "replace variable with constant").
  - Comparison-motif next-up is now framed as an additive constant-comparand option alongside the existing signal-vs-signal default.
- `DEVELOPMENT_NOTES.md`: added "Roles of constants in RTL" to the core design decisions recap, pointing to the new book section.

**Why**
The user asked that the coefficient/shift-amount/comparand clarification be logged in the book, not just in the CHANGES / MEMORY ledgers. They also caught two follow-on imprecisions in my prior framing:

1. I had implicitly suggested shifts should switch from variable-amount to constant-amount. The user correctly pointed out that we can (and do) do `a << b` with `b` a signal, and the question is bias — both modes have a place.
2. I had implicitly suggested all comparands are constants. The user correctly pointed out that the RHS of a comparison can be (and routinely is) another signal.

Both corrections are now in the doctrine alongside the original distinction. Future implementation of these motifs will follow the corrected framing.

**Validation**
- Documentation-only slice; no source touched.
- `cargo check`, `cargo test` (27 tests), `cargo clippy --all-targets -- -D warnings`, `cargo fmt --all --check`: all still clean.

**Impact**
- The book's `structural-rules.md` is now the durable reference for the three constant roles. Short-form docs point to it.
- A session recovering cold from `git log + live docs` has precise, corrected guidance for the next three motif slices (coefficients, shift-amount bias, constant comparands).

**Files touched**
`book/src/structural-rules.md`, `MEMORY.md`, `DEVELOPMENT_NOTES.md`, `CHANGES.md`.

---

## 2026-04-15-0017 — Doctrinal fix: coefficient / shift amount / comparand are distinct motifs

**Commit hash:** `dde27a2`

**What changed**
- `MEMORY.md` next-up list split the prior lumped "coefficient as general arithmetic motif" entry into three distinct motif families:
  1. **Coefficients** — multiplicative weights in arithmetic linear combinations (Add/Sub/Mul). `ci ≠ 0` for Add. Knob family `coefficient_*`.
  2. **Shift amounts** — structural parameters of shift ops. Typical range `[0, W-1]`. Knob family `shift_amount_*`.
  3. **Comparands** — thresholds / sentinels for comparisons. No zero-exclusion. Knob family `comparand_*`.
- Added an explicit reminder that the three are semantically distinct and should not be collapsed into a single `constant_prob` knob.

**Why**
In the prior slice's next-up list I wrote "Generalize coefficient-as-arithmetic-motif to Sub/Mul/Shift/Compare". User (rightly) pushed back: coefficient is arithmetic vocabulary (a multiplicative weight in a linear combination). It is not the correct word for:
- Shift amounts (`a << 2`): the `2` is a structural parameter of the shift op, not a weight. Yes, `a << 2` is arithmetically `a * 4`, but in representation and synthesis cost they are distinct.
- Comparands (`a == 7`): the `7` is a threshold / sentinel / target value, not a weight.

Lumping all three under "coefficient" conflates three distinct motifs. The correction preserves the vocabulary discipline the project has been accumulating (operators vs blocks, arity vs ports, etc.).

**Validation**
- Documentation-only slice; no source touched.
- `cargo check`, `cargo test` (27 tests), `cargo clippy --all-targets -- -D warnings`, `cargo fmt --all --check`: all still clean.

**Impact**
- The next-up list now correctly decomposes the work into three separate motif families with their own knobs and constraints.
- A session that crashes between here and the first motif-family implementation recovers with accurate guidance rather than the lumped-and-wrong original.
- Vocabulary discipline accumulates: "coefficient" joins "arity" and "port" as terms with restricted, precise meaning.

**Files touched**
`MEMORY.md`, `CHANGES.md`.

---

## 2026-04-15-0016 — M-to-1 combinational mux as a first-class block

**Commit hash:** `0564a49`

**What changed**
- `src/config.rs`: two new knobs.
  - `comb_mux_prob` (default `0.1`): probability that a non-leaf
    recursion point becomes an M-to-1 combinational mux block
    instead of an operator gate. Flop block takes priority; comb-mux
    block takes priority over operator gate.
  - `comb_mux_encoding_prob` (default `0.5`): per-mux probability of
    the Encoded style (chained ternary over `Eq(sel, k)` with a
    `ceil(log2(M))`-bit select bus) vs the OneHot style (M 1-bit
    select signals, OR of masked arms).
  - Both threaded into `Overrides`, `apply_cli_overrides`, and the
    probability-range validation loop.
- `src/main.rs`: two new CLI flags `--comb-mux-prob` and
  `--comb-mux-encoding-prob`.
- `src/gen/cone.rs`:
  - `build_cone` adds a new branch between the flop branch and the
    operator gate branch: if `rand() < comb_mux_prob`, dispatch to
    `build_comb_mux`.
  - New `build_comb_mux` — picks M from `[max(2, min_mux_arms),
    max_mux_arms]` (M=0 and M=1 excluded: no sensible fall-back for
    stateless muxes, 1-arm mux is a wire), picks encoding style via
    `comb_mux_encoding_prob`, dispatches to the style-specific helper.
  - New `build_comb_mux_one_hot` — recursively builds M (data, sel)
    arms, then assembles `D = OR_i({W{sel_i}} & data_i)` using the
    same `replicate_to_width` / `make_and` / `or_reduce_terms`
    primitives as the flop D-mux one-hot path. No Q-feedback term.
  - New `build_comb_mux_encoded` — recursively builds one
    `ceil(log2(M))`-bit select sub-cone + M data sub-cones, then
    assembles a chained ternary via `make_eq_const` / `make_mux`
    with a zero fall-through.
  - New inline unit test `comb_mux_block_produces_valid_output`:
    10 seeds × 2 encoding styles = 20 modules, all pass IR
    validation with `comb_mux_prob = 1.0`.
- `book/src/structural-rules.md`:
  - New Rule 15 "M-to-1 combinational mux block" codifying both
    shapes, the M range, the "no Q-feedback axis" constraint, and
    the block-vs-operator framing (muxes have ports, not arity).
  - "Operators vs blocks" preamble updated: the future-placeholder
    entry for "Block: mux (combinational)" is replaced with a
    pointer to Rule 15.
- `book/src/knobs.md`: new "Combinational mux block" subsection
  documenting the two knobs with cross-references to Rule 15.
- `book/src/algorithm.md`: `build_cone` pseudocode gains the comb-mux
  branch in its correct dispatch position (after flop, before operator).
- `book/src/tutorial.md`: new Example 9 "Combinational M-to-1 mux
  block" with actual captured SV excerpt showing the chained-ternary
  form; Example 10 (was 9) "Mixing everything" follows.
- `book/src/recipes.md`: new entry "I want combinational muxes, not
  just flop D-muxes" with a tuned knob combo.
- `USER_GUIDE.md`: two new CLI flags added to the knob table.
- `CODEBASE_ANALYSIS.md`: module map for `cone.rs` updated to list
  the three new build_comb_mux helpers and the new dispatch branch
  in `build_cone`.
- `MEMORY.md` / `CHANGES.md`: per workflow.

**Why**
Per user direction: promote the M-to-1 mux to a first-class
combinational motif. Prior to this slice, M-to-1 muxes existed only
as compound gate trees buried inside flop D-input construction;
combinational logic could only emit 2:1 muxes via `GateOp::Mux`.
Real designs use M-to-1 muxes extensively in combinational datapaths
(selectors, bus steering, priority encoders). Making them a
first-class block motif closes a large expressiveness gap.

This slice is also a direct application of the operators-vs-blocks
doctrine established in the prior slice: Mux is a block, so its
generalization is a *structural* motif (port counts, encoding
style), not an arity bump. No new `GateOp` variant — the mux is a
compound gate tree, same as the flop D-mux.

**Validation**
- `cargo check --all-targets`, `cargo test` (25 unit + 2 integration =
  27 tests, was 26), `cargo clippy --all-targets -- -D warnings`,
  `cargo fmt --all --check`: all clean.
- End-to-end: `cargo run -- --comb-mux-prob 1.0
  --comb-mux-encoding-prob 0.0 ...` emits the one-hot OR-of-masks
  shape; with `--comb-mux-encoding-prob 1.0` the same knobs produce
  the chained-ternary shape with a `20'h0` fall-through (no
  Q-feedback).

**Impact**
- M-to-1 combinational muxes are now routinely emitted. Generated SV
  shape distribution is closer to real-world datapath idioms.
- Phase 2 still in progress; Verilator-lint smoke now needs to
  also cover `comb_mux_prob` settings as well as `share_prob` and
  the flop styles.
- The prior conceptual plan "land M-to-1 combinational mux block"
  from the previous slice's next-up list is complete.

**Files touched**
`src/config.rs`, `src/main.rs`, `src/gen/cone.rs`, `book/src/structural-rules.md`, `book/src/knobs.md`, `book/src/algorithm.md`, `book/src/tutorial.md`, `book/src/recipes.md`, `USER_GUIDE.md`, `CODEBASE_ANALYSIS.md`, `MEMORY.md`, `CHANGES.md`.

---

## 2026-04-15-0015 — N-arity for associative operators + operators-vs-blocks doctrine

**What changed**
- `src/config.rs`: new knobs `min_gate_arity` (default 2) and
  `max_gate_arity` (default 4). `Config::validate` enforces `min >= 2`
  and `max >= min`. New `ConfigError::GateArityRange`. Overrides and
  `apply_cli_overrides` updated. Comment on the knob explicitly states
  that arity applies to operators only (And/Or/Xor/Add/Mul), not to
  blocks; Sub is excluded because it is not associative.
- `src/main.rs`: new CLI flags `--min-gate-arity` and
  `--max-gate-arity`, threaded into `Overrides`.
- `src/gen/cone.rs`: `input_widths_for` now returns N-wide operand
  lists for `And`, `Or`, `Xor`, `Add`, `Mul` (N drawn from the new
  knob range). `Sub` remains strictly 2-arity (documented inline with
  the reason: subtraction is not associative, so N-arity chains
  `a - b - c` come from cascaded 2-arity nodes, not a single N-arity
  Sub). Added `use crate::config::Config` so `input_widths_for` can
  read the new range.
- `src/emit/sv.rs`: `render_gate` uses a `joined(sep)` helper to emit
  any-arity infix expressions for the associative ops (`a & b & c`,
  `a + b + c + d`, etc.). `Sub` retained as the explicit 2-operand
  form.
- `src/ir/validate.rs`: `check_gate_shape` accepts `operands.len() >= 2`
  for the associative ops, exactly 2 for `Sub`. Added 3 tests:
  - `accepts_nary_and_with_three_operands`
  - `rejects_and_with_fewer_than_two_operands`
  - `rejects_nary_add_operand_width_mismatch` (4-way Add with one
    mismatched-width operand)
- `src/ir/types.rs`: header doc comment updated; "operand arity"
  replaced with "operand count", plus a vocabulary-discipline note
  pointing to the book's operators-vs-blocks preamble.
- `book/src/structural-rules.md`:
  - New "Operators vs blocks" preamble up front. Explicit vocabulary
    discipline: *arity* is operator vocabulary only; *ports / arms /
    port count* is block vocabulary. Rules grouped by what they
    govern (combinational integrity / flop block / future mux block
    / correctness guarantees).
  - New Rule 14 "Operator N-arity for associative operators". States
    which ops are associative (And/Or/Xor/Add/Mul), which are not
    (Sub, comparisons, shifts), and why operator arity is a
    different kind of generalization than block port-counts.
  - Rule 10 width table updated: associative ops show `[W, W, ...] (N ≥ 2)`;
    Sub shown separately as strictly 2-arity.
  - Mux entry in the unary/special-arity list rewritten to state
    explicitly that Mux is a block with *ports*, not arity.
- `book/src/algorithm.md`: width-rules table matches the catalog.
  Added a sentence explaining that the associative operators draw
  arity from `cfg.min_gate_arity..=cfg.max_gate_arity`.
- `book/src/knobs.md`: new "Operator N-arity" subsection documenting
  the two knobs with the operators-only framing.
- `USER_GUIDE.md`: two new CLI flags in the knobs table.
- `DEVELOPMENT_NOTES.md`: new "Operators vs blocks" entry in the core
  design decisions recap. Points to the book preamble + Rule 14.
- `CODEBASE_ANALYSIS.md`: invariants list gains the operator N-arity
  entry with a cross-reference.
- `MEMORY.md` / `CHANGES.md`: per workflow. Next-up list re-prioritized
  to queue up the M-to-1 combinational mux block and the linear-
  combination ADD coefficient motif that the user introduced during
  this slice's discussion.

**Why**
Per user direction: let logic and arithmetic operators have random
arity N ≥ 2 so the generator emits `a & b & c`, `w + x + y + z`, etc.
Not just 2-input trees. This is straightforward for associative ops
— grouping doesn't matter algebraically — but doesn't apply to Sub,
which the user flagged mid-slice. Sub was removed from the associative
set accordingly.

The deeper outcome of this slice is the operators-vs-blocks doctrine
that the user made explicit during discussion. Arity is the correct
word for operators; blocks have ports / arms / port count. Conflating
the two obscures the fact that operator generalization (N-arity) and
block generalization (enumerating motif shapes) are fundamentally
different activities. The book's rule catalog now opens with that
distinction so future rules land in the right category.

**Validation**
- `cargo check --all-targets`, `cargo test` (24 unit + 2 integration =
  26 tests), `cargo clippy --all-targets -- -D warnings`,
  `cargo fmt --all --check`: all clean.
- End-to-end: `cargo run -- --seed 3 --max-depth 3 --max-inputs 3
  --max-outputs 1 --flop-prob 0 --share-prob 0 --min-gate-arity 3
  --max-gate-arity 4` produces assign statements like
  `w_4 = w_2 + w_3 + w_3 + w_3` and `w_5 = w_2 + w_3 + w_2 + w_4`,
  confirming N-arity in emitted SV.

**Impact**
- Generated RTL now exhibits N-arity associative operators — closer
  to typical hand-written logic and arithmetic shapes.
- The operators-vs-blocks doctrine is now load-bearing and feeds
  straight into the next two slices' scope.

**Files touched**
`src/config.rs`, `src/main.rs`, `src/gen/cone.rs`, `src/emit/sv.rs`, `src/ir/validate.rs`, `src/ir/types.rs`, `book/src/structural-rules.md`, `book/src/algorithm.md`, `book/src/knobs.md`, `USER_GUIDE.md`, `DEVELOPMENT_NOTES.md`, `CODEBASE_ANALYSIS.md`, `MEMORY.md`, `CHANGES.md`.

---

## 2026-04-15-0014 — Q-feedback rule relaxation + structural-rules catalog

**Commit hash:** `6cbcbff`

**What changed**
- **Rule change (code):** `src/gen/cone.rs` — three sites in
  `drain_flop_worklist`, `drain_flop_one_hot`, `drain_flop_encoded`
  now pass `exclude = None` to `build_cone_with_retry` for the D
  sub-cones. Previously they passed `Some(q_node)`, forbidding the
  flop's own Q from being a leaf in its data/select/direct-D
  sub-cones. Q-feedback through arbitrary combinational logic in the
  D-cone is now freely permitted. The clock edge breaks the loop
  temporally; this matches standard synchronous feedback patterns
  (counters, toggles, accumulators, state machines).
- **Combinational no-loop preserved:** Rule 1 — a combinational gate
  output cannot appear upstream in its own fanin cone — is
  unchanged. It is enforced by arena-index monotonicity (pool entries
  pre-date each recursion step), not by the `exclude` parameter.
- **New durable artifact:** `book/src/structural-rules.md`. A
  catalog of 13 load-bearing generator invariants, each stated with
  its rationale, its "enforced where" location, and cross-references
  to the relevant code. Expected to grow as new rules become
  invariants (Phase 3+ placeholders already listed).
- **`book/src/SUMMARY.md`:** new chapter added to *Correctness
  Guarantees* section between "Generation by Construction" and
  "Synthesizability".
- **`book/src/sequential.md`:** retired the "No Q→D feedback through
  the mux datapath" section. Replaced with "Q-feedback in the D-cone
  is freely permitted" pointing to Structural Rules Rules 2 and 3.
  Pseudocode updated to drop the `exclude=Q` parameter.
- **`DEVELOPMENT_NOTES.md`:** the old "Q-exclusion contract" core
  design decision replaced with "Q-feedback freedom (revised)" that
  references the new catalog. Added a "Structural rules catalog"
  core decision establishing the book chapter as the durable source
  of truth — recaps point to it, do not duplicate rule text.
- **`CODEBASE_ANALYSIS.md`:** the `drain_flop_worklist` bullet
  updated to reflect `exclude = None` and to point to Rules 2 and 3.
  Added a pointer stating the full invariant catalog lives in the
  book.

**Why**
Per user direction: "Flop's Q output may be loopback to any input
and any number of times to inputs in the flop's D fanin cone."
Combined with the pre-existing QFeedback mux term (orthogonal), this
makes every legal synchronous feedback pattern expressible. The
previous Q-exclusion contract was an over-constraint I had inferred
from an earlier, tighter phrasing; the user has since clarified that
Q-in-sub-cones is intended.

Separately, the user asked that these kinds of rules make their way
into the book and into live docs, with an accumulating catalog as
the project matures. The `structural-rules.md` chapter is that
catalog. It is now the canonical location for every load-bearing
invariant. Inline rule restatements in short-form docs should point
to the catalog, not duplicate it — duplication leads to drift.

**Validation**
- Q-in-sub-cone working end-to-end: at `--seed 2 --max-depth 3
  --max-inputs 2 --max-outputs 1 --flop-prob 1.0 --max-flops 1
  --min-mux-arms 2 --max-mux-arms 2 --flop-mux-encoding-prob 0.0
  --share-prob 0.5`, the emitted SV contains `assign w_4 = r_0 + r_0`
  — the flop's Q (`r_0`) appears twice in a gate in its own D cone.
- `cargo check --all-targets`, `cargo test` (23 tests), `cargo
  clippy --all-targets -- -D warnings`, `cargo fmt --all --check`:
  all clean.
- Integration sweep of 20 seeds still passes with the relaxed rule.

**Impact**
- Generated RTL now exhibits real synchronous feedback patterns
  (counters, accumulators, state-returning logic) rather than only
  pass-through or clean-data registers.
- The book gains a durable, growing catalog of structural rules that
  a future session can scan to understand every invariant without
  archaeologizing commits.
- Future rule additions have a natural home. No more inline
  restatement and drift.

**Files touched**
`src/gen/cone.rs`, `book/src/structural-rules.md` (new), `book/src/SUMMARY.md`, `book/src/sequential.md`, `DEVELOPMENT_NOTES.md`, `CODEBASE_ANALYSIS.md`, `MEMORY.md`, `CHANGES.md`.

---

## 2026-04-15-0013 — mdBook becomes user-facing: Getting Started, Tutorial, Recipes

**Commit hash:** `bac6060`

**What changed**
- **`book/src/getting-started.md`** (new): installation, first module (with full annotated SV output), reading the output line-by-line, reproducibility explanation, batch generation via `--out`, dumping effective knobs. Ends with a pointer to Tutorial / Recipes / Knobs / Core Idea.
- **`book/src/tutorial.md`** (new): 9 progressive examples, each with the exact command and an excerpt of the generated SV. Progression: minimal combinational → deeper cones → multi-output → flops with direct D (M=0) → one-hot mux on D → encoded-select mux on D → Q-feedback variant → DAG-shaped cones → everything mixed. Opens with a "logic is deliberately nonsensical, that's the point" disclaimer so users aren't confused when the first `a + a + a` appears.
- **`book/src/recipes.md`** (new): 9 "I want to do X" cookbook entries — minimal smoke-test corpus, fanout stress, flop-heavy, encoded-mux stress, one-hot-mux stress, narrow/wide-data stress, reproduce a module, parser-only stress, formal-equivalence sizing. Each recipe states the goal, gives the CLI command, explains which knobs matter.
- **`book/src/introduction.md`** (rewritten): now leads with what anvil is (not with the "problem" section) and who it's for. Adds a five-minute pitch (command + output). Describes what makes anvil different (vs grammar fuzzers vs hand-written suites). Ends with a "what you'll find in this book" outline and a clear invitation to jump to Getting Started.
- **`book/src/SUMMARY.md`** (restructured): five parts —
  - *Using anvil* (Getting Started, Tutorial, Recipes) — leads the book.
  - *How It Works* (Core Idea, Why Not a Grammar?, Algorithm, IR).
  - *Correctness Guarantees* (By Construction, Synthesizability, Non-Triviality).
  - *Motif Catalogue* (Sequential, Sharing, Hierarchy).
  - *Reference* (Knobs, Architecture, Non-Goals).
  Users arrive at the welcoming part first; contributors find design content in the middle; everyone finds reference material at the end.
- **`book/book.toml`**: removed obsolete `multilingual = false` field that mdbook 0.4.51 now rejects. Updated book title and description to reflect the book's dual user/design role.

**Why**
Per user direction: "the book is the user facing surface to the project... documentation is key to attract and retain users... top-notch and littered with examples with increasing complexity. We should not scare users."

Prior to this slice the book was correct and thorough but relentlessly design-focused. A user arriving at the book's first page would land on "The Core Idea" — a philosophical argument about circuit-graph IRs vs EBNF — before ever seeing a single command. That is backward for a tool that people need to actually run. This slice fixes the on-ramp.

The user-facing chapters are copy-pasteable, progress by one concept per example, and show real generated SV at each step (not hypothetical snippets). The SV fragments in Tutorial were captured from actual `cargo run --` invocations during authoring.

**Validation**
- `mdbook build book` succeeds and produces `book/book-out/` with all chapters rendered.
- All code gates remain clean (no source touched): `cargo check`, `cargo test` (23 tests), `cargo clippy -- -D warnings`, `cargo fmt --check`.
- Cross-read new chapters against the code (`src/main.rs` CLI flags, `src/config.rs` defaults, `src/gen/cone.rs` flop motifs) to verify every command in the Tutorial and every recipe in Recipes actually works with the currently-implemented flags.

**Impact**
- The book is now the intended first-stop for users, not just contributors.
- Every user-exposed feature (`CLI flags`, flop motifs, DAG sharing, reproducibility) has at least one worked example.
- Design chapters remain for anyone who wants them — just accessible via a clearly-labeled "How It Works" section rather than as the book's opening.

**Files touched**
`book/src/getting-started.md` (new), `book/src/tutorial.md` (new), `book/src/recipes.md` (new), `book/src/introduction.md` (rewritten), `book/src/SUMMARY.md` (restructured), `book/book.toml` (obsolete field removed), `MEMORY.md`, `CHANGES.md`.

---

## 2026-04-15-0012 — mdBook staleness refresh: knobs, IR, algorithm, architecture

**Commit hash:** `62fdeaa`

**What changed**
- `book/src/knobs.md`:
  - Rewrote the knob taxonomy into four categories: Structural, Sequential, Sharing, Mix/Termination, Hierarchy.
  - Added every sequential knob that was missing: `max_flops_per_module`, `min_mux_arms`, `max_mux_arms`, `flop_qfeedback_prob`, `flop_mux_encoding_prob`, `use_async_reset`.
  - Updated defaults block to match current `Config::default()` (was showing Phase-0 defaults like `flop_prob: 0.0`, `share_prob: 0.0`).
  - Added a "CLI coverage" section listing every flag so users know what's reachable without a config file.
- `book/src/ir.md`:
  - Added `FlopKind`, `FlopMux`, `MuxArm` types to the core-types block.
  - Updated `Flop` to include `kind` and `mux` fields.
  - Clarified `Module.nodes` is `Vec<Node>` (not `Arena<Node>`) with arena *semantics* via `NodeId` indexing.
  - Removed "Phase 2+" annotations on clock/reset/flops (now live).
  - Fixed "Phase 3 (sharing)" reference (now Phase 2 after renumber).
  - Rewrote the Invariants section to distinguish what's enforced by construction vs by the per-gate width validator in `ir::validate::validate`.
  - Added the validator's rich error categories (per-gate arity, operand widths, output widths).
  - Updated name-generation section to cover clk/rst_n.
- `book/src/algorithm.md`:
  - Module-level pseudocode now shows clk/rst_n port reservation and the exclusion of those ports from the signal pool.
  - Cone recursion pseudocode shows the DAG-sharing fork (`rand() < share_prob` → `try_share`) and the `exclude` parameter for Q-isolation.
  - New "Flop worklist drain" section covering M ∈ {0, 2..=max}, per-flop FlopKind choice, per-flop mux style choice (one-hot vs encoded), with cross-references to `sequential.md`.
  - Terminal selection pseudocode rewritten to match current behavior: prefer dep-bearing matching-width entries, fall back to any matching-width, then lazy width-adapter, then constant as last resort.
  - Width-rules table: added `Shl/Shr` row.
  - Anti-collapse section: clarified `NodeId` equality catches sharing-induced self-reference.
- `book/src/architecture.md`:
  - Crate-layout comments updated: `main.rs` notes CLI coverage; `validate.rs` notes unit tests; `cone.rs` notes DAG sharing and flop-mux assembly; removed placeholder `hierarchy.rs # Phase 5+` (not in source yet; will land in Phase 4).
  - Renumbered "Phase 5" references to Phase 4.
  - Key-types block updated with `FlopKind`, `FlopMux`, and the `kind`/`mux` fields on `Flop`.
  - Testing-strategy section replaced placeholder with concrete counts: 8 validator tests, 7 cone tests, 6 emitter tests, 2 integration = 23 total.
  - CLI section replaced "..." placeholder with the full flag surface and defaults.
- `MEMORY.md` and `CHANGES.md` updated per the mandatory pre-commit docs.

**Why**
The user flagged that several book chapters had gone stale relative to recent code slices. Specifically:
- `knobs.md` still showed `flop_prob: 0.0` and `share_prob: 0.0` as defaults, which is contradicted by `Config::default()` (0.15 and 0.3 respectively) and would mislead anyone reading the book to understand tunable ranges.
- `ir.md` did not document the new `FlopKind`, `FlopMux`, `MuxArm` types at all, and still described clock/reset/flops as "Phase 2+" aspirations rather than live features.
- `algorithm.md` showed an outdated pseudocode with `pick_node_kind(gate | flop | terminal)` and a `terminal_reuse_prob` / `constant_prob` coin-flip that doesn't match the current `pick_terminal` implementation.
- `architecture.md` referenced Phase 5 for hierarchy (now Phase 4 after the renumbering in commit `4317c82`), had a `...` placeholder in the CLI section, and listed no test counts.

This slice closes those gaps. The book's design chapters now match the code at commit `c9ec12c`.

**Validation**
- Documentation-only slice; no source changes.
- `cargo check`, `cargo test` (23 tests), `cargo clippy -- -D warnings`, `cargo fmt --check`: all still clean (no code touched).
- Cross-read each updated chapter against the corresponding source file to verify no dangling references to removed/renamed types.

**Impact**
- A contributor reading the book to understand anvil's IR or algorithm now gets a faithful current-state picture.
- The knob defaults in `knobs.md` match what `cargo run -- --dump-config` actually prints.
- Phase numbering is consistent across the book, `ROADMAP.md`, and `CODEBASE_ANALYSIS.md`.

**Follow-up (flagged in next-up)**
The user additionally asked that the book serve as the user-facing surface — with progressive examples and a welcoming on-ramp, not just design reference. The existing chapters are correct but contributor-oriented. A follow-up slice will add Getting Started, Tutorial (progressive examples), and Recipes chapters, and restructure `SUMMARY.md` to lead with user material.

**Files touched**
`book/src/knobs.md`, `book/src/ir.md`, `book/src/algorithm.md`, `book/src/architecture.md`, `MEMORY.md`, `CHANGES.md`.

---

## 2026-04-15-0011 — CLI coverage for all Phase 1/2 motif knobs

**Commit hash:** `c9ec12c`

**What changed**
- `src/main.rs`:
  - New CLI flags on `Cli`: `--max-flops-per-module`, `--min-mux-arms`, `--max-mux-arms`, `--flop-qfeedback-prob`, `--flop-mux-encoding-prob`.
  - `cli_overrides` function threads the new flags into `anvil::config::Overrides`.
- `src/config.rs`:
  - `Overrides` struct gains five new `Option<_>` fields matching the new CLI flags.
  - `Config::apply_cli_overrides` handles each new override.

**Why**
Every Phase 1/2 motif knob now has a dedicated CLI flag. Previously, exercising flop motifs required editing a JSON config file and passing `--config`, which is enough friction to discourage casual experimentation and to make CLI-based reproducibility less pleasant. After this slice, a user can force any combination — e.g., encoded-mux-only QFeedback flops with M ≤ 3 — in a single command line.

This is the "Consider adding a `--share-prob` CLI flag" item from the prior `MEMORY.md` next-up list, broadened to include all the other Phase 1/2 motif knobs that were similarly JSON-only.

**Validation**
- `cargo check --all-targets`, `cargo test` (23 tests), `cargo clippy --all-targets -- -D warnings`, `cargo fmt --all --check`: all clean.
- `cargo run -- --help` surfaces all five new flags with their expected names.
- End-to-end check: `cargo run -- --seed 1 --max-depth 2 --max-inputs 2 --flop-prob 1.0 --flop-mux-encoding-prob 0.0 --max-mux-arms 2` produces the one-hot replicate-AND pattern (confirming `--flop-mux-encoding-prob 0.0` is actually honored).

**Impact**
- Phase 1/2 motif exploration is now CLI-native.
- Removes one friction point before the Verilator-lint smoke run: that smoke run will ultimately need to sweep both `share_prob` and the flop encoding probability to satisfy Phase 2's exit criterion, and CLI-driven sweeps are far easier to script than JSON-config-driven ones.

**Files touched**
`src/main.rs`, `src/config.rs`, `MEMORY.md`, `CODEBASE_ANALYSIS.md`, `CHANGES.md`.

---

## 2026-04-15-0010 — Phase 2 start: per-operand DAG-cone sharing

**Commit hash:** `6ba646b`

**What changed**
- `src/gen/cone.rs`:
  - `build_cone` operand loop now consults `cfg.share_prob` per operand. With that probability it calls the new `try_share` helper; on `Some(node)` the operand terminates at that existing pool entry, on `None` it falls back to normal recursion.
  - New `try_share(g, pool, width, exclude)` helper: returns a random matching-width pool entry with non-empty deps, honoring the `exclude` filter used for flop Q-exclusion.
  - New unit test `share_prob_high_shares_internal_gates`: a 32-seed sweep at `share_prob=0.9` must produce at least one Gate (not just a primary input) with fanout ≥ 2. This verifies the non-leaf DAG mechanism actually fires and is not masked by leaf-level reuse.
- `src/config.rs`: `share_prob` default raised from `0.0` to `0.3`, making DAG-ish cones the generator's default shape.
- `book/src/sharing.md` rewritten:
  - States that tree-and-DAG is a per-operand decision, not a global mode. The generator mixes both freely.
  - Explains the distinction between leaf-level reuse (always on) and non-leaf sharing (controlled by `share_prob`).
  - Includes the `try_share`/`build_cone` pseudocode.
  - Documents the anti-collapse guards still applying post-share.
- `ROADMAP.md`: Phase 2 status flipped to `in progress`. Exit criterion extended to cover Verilator-lint on `share_prob ∈ {0.0, 0.3, 0.9}`.
- `USER_GUIDE.md`: `--share-prob` default updated to 0.3; description rewritten as per-operand probability.
- `CODEBASE_ANALYSIS.md`:
  - Module map for `cone.rs` gains `try_share` and the DAG-sharing summary.
  - Phase coverage map: Phase 2 now `in progress`.
  - Invariants-enforced list gains the `share_prob` / `try_share` entry.
  - Testing surface: 7 cone unit tests (was 6), total 23 (was 22).
- `DEVELOPMENT_NOTES.md`: calibration section gains a `share_prob = 0.3` entry explaining the default and clarifying that `share_prob = 0.0` is not pure tree (leaf-level reuse via `pick_terminal` is always on).
- `MEMORY.md`: Current state, next-up, recent commits, known-gaps all refreshed.

**Why**
Phase 2 per user direction: enable DAG cones. User framing: "tree or DAG, randomly picked per recursion point" — exactly what a per-operand `share_prob` coin gives. For this slice we set `share_prob = 0.3` as the default so the generator produces DAG-shaped cones by default; users who want pure-tree or maximally-shared modes set `share_prob` explicitly to 0.0 or ~1.0.

The mechanism is intentionally minimal: two lines in `build_cone` plus one helper. The pool already contained every `Gate` node on creation from Phase 1 work, so the infrastructure was in place; what was missing was the non-leaf hook to consult it.

**Validation**
- `cargo check --all-targets`: clean.
- `cargo test`: 21 unit + 2 integration = 23 tests, all pass.
- `cargo clippy --all-targets -- -D warnings`: clean.
- `cargo fmt --all --check`: clean.
- Pipeline sweep of 20 seeds passes with DAG-sharing on by default — no multi-driver violations, no IR-validation failures, no empty dep-sets. The lazy-adapter path continues to operate when widths don't match any pool entry.
- New `share_prob_high_shares_internal_gates` unit test passes.

**Impact**
- Generated SV now routinely has internal gate fanout > 1: one wire drives multiple consumers. This is the first motif-diversity step that makes `anvil` output resemble real hand-written RTL rather than pure random trees.
- Phase 2 exit gate is now Verilator-lint on representative `share_prob` values, identical in form to the Phase 1 Verilator gate — both block on tooling availability.
- The `share_prob = 0.0` → pure tree framing in `book/src/sharing.md` is corrected: pure tree is impossible because leaf-level reuse is always on. The book now reflects that nuance.

**Files touched**
`src/gen/cone.rs`, `src/config.rs`, `book/src/sharing.md`, `ROADMAP.md`, `USER_GUIDE.md`, `CODEBASE_ANALYSIS.md`, `DEVELOPMENT_NOTES.md`, `MEMORY.md`, `CHANGES.md`.

---

## 2026-04-15-0009 — Inline unit tests for cone helpers and SV emitter

**Commit hash:** `c8043c3`

**What changed**
- `src/gen/cone.rs`: added `#[cfg(test)] mod tests` with 6 tests:
  - `ceil_log2_expected_values` — hand-picked values plus a 62-value sweep asserting the `2^ceil_log2(n) >= n` invariant.
  - `pick_mux_arm_count_never_returns_one` — 10K draws confirming the `M ∈ {0, 2..=max}` discipline is structurally enforced, not accidentally.
  - `width_adapter_identity` — passthrough when src == target, no IR nodes added.
  - `width_adapter_slice_shrinks` — src > target emits a `Slice{hi: target-1, lo: 0}` with correct operand.
  - `width_adapter_concat_expands_exact_multiple` — src < target and src divides target emits a single Concat with the right number of copies.
  - `width_adapter_concat_expands_non_multiple` — src < target and non-multiple emits Concat + Slice; outer node is a Slice of target width; a 9-bit Concat exists as its source (example: 3-bit src, 8-bit target, copies = 3, concat_width = 9, slice to 8).
- `src/emit/sv.rs`: added `#[cfg(test)] mod tests` with 6 tests on hand-built IRs:
  - `emits_module_header_and_endmodule` — module declaration shape + port typing + passthrough assign.
  - `omits_clk_rst_n_when_no_flops` — even when `Module.clock` and `Module.reset` are set, clk/rst_n are absent from the port list if `m.flops.is_empty()`.
  - `emits_always_ff_with_single_clk_and_async_rst_n` — canonical `always_ff @(posedge clk or negedge rst_n)` header, `if (!rst_n)` active-low reset branch, `r_0 <= 4'h0;` reset value, `r_0 <= a;` clocked assignment, output wired to Q.
  - `constant_and_operators_rendered` — `{W}'h{hex}` constant form, `a & b` for And, `w_3 ^ 8'h5a` for Xor with a constant operand.
  - `slice_and_concat_rendered` — `a[3:0]` for Slice, `{a, a}` for a 2-copy Concat.
  - `mux_rendered_with_ternary` — `(s) ? (a) : (b)` for Mux.
- `CODEBASE_ANALYSIS.md`: "Testing surface" section now enumerates all three inline test modules with counts; total is 22 tests.
- `MEMORY.md`: Current state, next-up, and recent commits refreshed. Phase 1's remaining exit gate is now just the Verilator-lint smoke run.

**Why**
The validator landed in the previous slice plus the 22-seed integration sweep cover "does the output validate?" — but the individual helpers (`make_width_adapter`, `ceil_log2`, `pick_mux_arm_count`) and the emitter's per-form rendering had no direct pin. A regression in, say, the `ceil_log2` function or the `always_ff` emitter shape would only be caught indirectly (or not at all, in the emitter's case, since a change to the `always_ff` header text would still validate). Direct unit tests convert those implicit regressions into visible test failures.

**Validation**
- `cargo test`: 20 unit + 2 integration = 22 tests, all pass.
- `cargo check --all-targets`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --all --check`: all clean.

**Impact**
- Phase 1 exit gate reduced to just "Verilator-lint pass on a representative seed range." All Rust-side checks are in place.
- Future refactors of cone helpers or the emitter will fail tests loudly rather than silently drift.

**Files touched**
`src/gen/cone.rs`, `src/emit/sv.rs`, `CODEBASE_ANALYSIS.md`, `MEMORY.md`, `CHANGES.md`.

---

## 2026-04-15-0008 — Per-gate width/arity validator + inline unit tests

**Commit hash:** `4eb5daa`

**What changed**
- `src/ir/validate.rs`:
  - Replaced the TODO `// 5. Gate operand widths agree with declared output width rules.` with a full per-gate shape checker. Every `GateOp` variant has explicit arity and width rules:
    - `And / Or / Xor / Add / Sub / Mul` — 2 operands, each width = output width.
    - `Not` — 1 operand, width = output width.
    - `Mux` — 3 operands, `[sel 1-bit, a out_w, b out_w]`.
    - `Eq / Neq / Lt / Gt / Le / Ge` — 2 operands, equal width, output = 1-bit.
    - `RedAnd / RedOr / RedXor` — 1 operand of any width, output = 1-bit.
    - `Shl / Shr` — 2 operands, value operand width = output width, shift amount unconstrained.
    - `Slice { hi, lo }` — 1 operand, `hi >= lo`, `out_w == hi - lo + 1`, source width > `hi`.
    - `Concat` — variadic (>= 1 operand), `out_w == sum(operand widths)`.
  - New richer `ValidateError` variants: `GateArity`, `GateOperandWidth`, `GateOutputWidth`, `GateOperandsMustMatch`. Old `OperandWidth` and `WidthMismatch` variants retired.
  - New inline `#[cfg(test)] mod tests` (8 tests):
    - `accepts_minimal_valid_module`
    - `rejects_and_operand_width_mismatch`
    - `rejects_mux_non_1bit_selector`
    - `rejects_eq_output_not_1bit`
    - `rejects_concat_sum_mismatch`
    - `rejects_slice_out_of_bounds`
    - `rejects_not_wrong_arity`
    - `accepts_concat_variadic_replicate` (the N-copy pattern used by the width adapter and flop-mux assembly).
- `CODEBASE_ANALYSIS.md`:
  - Module map for `validate.rs` updated to note the width-rule checker and inline unit tests.
  - "Invariants currently enforced" / `ir::validate::validate` section now enumerates the per-gate width contract.
  - "Testing surface" entry for `src/ir/validate.rs` added.
  - "Known weaknesses": removed the now-closed "validator does not check per-gate operand widths" item.
- `DEVELOPMENT_NOTES.md`:
  - Testing-strategy section gains a paragraph on the validator's new role: an active safety net specifically designed to catch width bugs in the hand-constructed flop-mux assembly code (where gate-building does not go through the recursion).
- `MEMORY.md`:
  - Next-up list updated to reflect the closed validator task.
  - Recent-commits list gains `f2a3d81` (the previous commit).
  - Known-gaps list retires the per-gate validator TODO.

**Why**
Phase 1's exit criteria call for a working, audited single-module generator. Without a per-gate width validator, generator bugs in the hand-constructed flop-mux assembly (where gates like `Mux`, `And`, `Eq`, `Concat` are built by hand rather than via the recursion's `input_widths_for`) could emit subtly malformed IR that happens to parse but violates SV semantics. The width validator catches these at the IR level, before the emitter or any downstream tool ever sees them.

The inline unit tests pin the validator's behavior: each rejection class has a dedicated test so future changes to the width rules cannot silently drop a case.

**Validation**
- `cargo check --all-targets`: clean.
- `cargo test`: 8 new unit tests + 2 pipeline integration tests = 10 total, all pass.
- `cargo clippy --all-targets -- -D warnings`: clean.
- `cargo fmt --all --check`: clean.
- Pipeline sweep of 20 seeds passes with the stricter validator active, confirming the generator is currently producing width-correct IR and the validator is an *active* (not drift-prone) safety net.

**Impact**
- Generator bugs that produce width-mismatched gates are now caught at validation time with specific, actionable error messages (node id, op, operand index, expected vs got widths).
- Phase 1 exit is one step closer: the remaining Phase 1 tasks are in-source unit tests for `cone.rs` / `sv.rs` and the Verilator/Yosys smoke run.

**Files touched**
`src/ir/validate.rs`, `CODEBASE_ANALYSIS.md`, `DEVELOPMENT_NOTES.md`, `MEMORY.md`, `CHANGES.md`.

---

## 2026-04-15-0007 — Elevate mdBook to equal-standing live doc in session recovery

**Commit hash:** `f2a3d81`

**What changed**
- `SESSION_BOOTSTRAP.md`: reworded the mdBook entry in the bootstrap reading order. The book is now described explicitly as a live doc, not reference material, with language stating that a session skipping the book will make locally-correct but globally-wrong decisions.
- `COMMIT.md`:
  - Reworded the `book/` files-involved section: the mdBook is "a live doc of equal standing" and is "load-bearing" for session recovery.
  - Item 9 of the 12-item pre-commit checklist now explicitly states the mdBook's role and mandates adding permanent design decisions there, not just in commit messages.
- `README.md`: the ramp-up reading list entry for `book/` now states equal standing and the recovery-requires-reading-it stance. Follow-up sentence clarifies the book is part of the status-authority set, not adjacent to it.

**Why**
The user pointed out that the mdBook is part of the context-rebuild surface for post-crash / post-session-loss recovery, not a separate reference tier. The short-form live docs (`README`, `ROADMAP`, `MEMORY`, `CHANGES`, `DEVELOPMENT_NOTES`, `CODEBASE_ANALYSIS`, `USER_GUIDE`, `COMMIT`) carry *operational* state; the mdBook carries *design* context — why the generator is shaped the way it is, what has been deliberately rejected, what the motif catalogue looks like. A session that reconstructs operational state without the design context will make decisions that are locally coherent but globally wrong.

This slice makes the mdBook's recovery role explicit in three places (`SESSION_BOOTSTRAP.md`, `COMMIT.md` preamble + checklist, `README.md` reading list) so no future session can miss it.

**Validation**
- Documentation-only slice; no source changes.
- `cargo check`, `cargo test`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --all --check`: all still clean (no code touched).

**Impact**
- The 12-item pre-commit checklist now has an explicitly strengthened item 9 that closes a gap where design decisions might have landed in commit messages and `DEVELOPMENT_NOTES.md` but not in the mdBook.
- New sessions reading `SESSION_BOOTSTRAP.md` will not mistake the mdBook for optional reading.

**Files touched**
`SESSION_BOOTSTRAP.md`, `COMMIT.md`, `README.md`, `MEMORY.md`, `CHANGES.md`.

---

## 2026-04-15-0006 — Live-doc catch-up: capture flop-mux rationale + tighten commit workflow

**Commit hash:** `a1a9ea9`

**What changed**
- `DEVELOPMENT_NOTES.md`:
  - Added "Flop-D mux motifs" and "Q-exclusion contract" to the Core design decisions recap.
  - Added rejected alternative: `always_comb` + `case` for Encoded-mux flop D (why chained ternary wins).
  - Added rejected alternative: M = 1 mux arm (why it's excluded by design).
  - Added gotcha: module-level `#![allow(clippy::too_many_arguments)]` in `src/gen/cone.rs` with rationale.
  - Added calibration notes for `flop_mux_encoding_prob = 0.5` and `flop_qfeedback_prob = 0.5`.
  - Documented the QFeedback-in-Encoded design choice (replace `data_0` with Q) and the rejected alternative (extra (M+1)th entry).
- `MEMORY.md`:
  - Recent-commits list updated with `10090c2`.
  - Open-questions list updated with the `flop_mux_encoding_prob` calibration entry and the ternary-vs-case revisit trigger.
- `COMMIT.md`:
  - Added a non-negotiable 12-item pre-commit checklist. Every item is listed explicitly. The checklist makes skipping any live-doc update a visible workflow violation rather than a silent drift.

**Why**
Prior to this slice, the last two commits (`47675df` and `10090c2`) landed load-bearing design rationale — why M=1 is excluded, why chained ternary over `case`, why the Q-exclusion contract — that was captured in `CHANGES.md` and `book/src/sequential.md` but not in `DEVELOPMENT_NOTES.md`, which is the contributor-facing design-decision ledger. `MEMORY.md`'s recent-commits list was also one commit behind. The user flagged the slippage.

The fix has two parts: (1) a factual catch-up of the missed content, and (2) a structural fix to the commit workflow itself — an explicit 12-item pre-commit checklist in `COMMIT.md` that makes every live-doc gate impossible to skip implicitly.

**Validation**
- Documentation-only slice; no source changes.
- `cargo check`, `cargo test`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --all --check`: all still clean (no code touched).

**Impact**
- Future sessions can reconstruct the full design rationale from `DEVELOPMENT_NOTES.md` alone, without having to archaeologize across commit messages.
- The pre-commit checklist makes workflow compliance auditable: each item is either affirmatively satisfied or the commit does not proceed.

**Files touched**
`DEVELOPMENT_NOTES.md`, `MEMORY.md`, `COMMIT.md`, `CHANGES.md`.

---

## 2026-04-15-0005 — Encoded-select flop mux (chained ternary) alongside one-hot

**Commit hash:** `10090c2`

**What changed**
- `src/ir/types.rs`:
  - Replaced `Flop.arms: Vec<MuxArm>` with `Flop.mux: FlopMux`.
  - `FlopMux` enum: `None` (M=0), `OneHot(Vec<MuxArm>)`, `Encoded { sel: NodeId, data: Vec<NodeId> }`.
- `src/config.rs`:
  - New knob `flop_mux_encoding_prob` (default `0.5`): per-flop probability of using the encoded-select style instead of one-hot.
- `src/gen/cone.rs`:
  - New `drain_flop_encoded`: builds one select sub-cone of width `ceil(log2(M))` and M (or M-1 for QFeedback) data sub-cones, assembles D as a chained ternary over `Eq(sel, k)` with a `0` or `Q` fall-through.
  - New `drain_flop_one_hot`: extracts the previous one-hot assembly into its own function.
  - New `assemble_flop_d_encoded`, `make_constant`, `make_eq_const`, `make_mux`, `ceil_log2` helpers.
  - Renamed `assemble_flop_d` → `assemble_flop_d_one_hot`.
  - Per-flop dispatch in `drain_flop_worklist`: picks encoded or one-hot via `cfg.flop_mux_encoding_prob`.
  - Module-level `#![allow(clippy::too_many_arguments)]` to silence the lint on helpers that legitimately thread many context refs.
- `book/src/sequential.md`: documents both encoding styles, the 2×2 style-kind matrix, and the QFeedback+Encoded special case where index 0 is replaced by Q.
- `USER_GUIDE.md`: documents `--flop-mux-encoding-prob`.
- `CODEBASE_ANALYSIS.md`: module map, helper list, and invariants updated for the new drain path.
- `MEMORY.md`: state, next-up, recent commits refreshed.

**Why**
The user asked for an encoded-select variant alongside the existing one-hot, with the Q-feedback case routing Q on `sel == 0` and on out-of-range values. Both styles correspond to real synchronous-design shapes (one-hot for arbitration-driven register banks, encoded for opcode/address/state-selected registers) and exercise different synthesis paths. Picking per-flop preserves motif diversity within a single generated module.

**Validation**
- `cargo check`, `cargo test` (2 tests pass, ~2s for 20-seed sweep), `cargo clippy --all-targets -- -D warnings`, `cargo fmt --all --check`: all clean.
- Visual inspection with `--seed 5 --max-depth 2 --flop-prob 1.0` shows chained ternaries in the output: `(eq_k) ? data_k : (eq_{k-1}) ? data_{k-1} : ... : fall_through`, confirming the encoded-mux assembly.

**Impact**
- Phase 1 now emits two distinct flop motifs. Motif diversity is no longer bound by encoding style.
- The `FlopMux` enum carries introspective information about each flop's mux shape, useful for future debugging/inspection tooling even though it is not load-bearing for emission today.

**Files touched**
`src/ir/types.rs`, `src/config.rs`, `src/gen/cone.rs`, `book/src/sequential.md`, `USER_GUIDE.md`, `CODEBASE_ANALYSIS.md`, `MEMORY.md`, `CHANGES.md`.

---

## 2026-04-15-0004 — M-to-1 one-hot mux flops with two motifs

**Commit hash:** `47675df`

**What changed**
- `src/ir/types.rs`:
  - New `FlopKind` enum: `ZeroDefault` (D = 0 when no select fires) and `QFeedback` (D = Q when no select fires).
  - New `MuxArm { data: NodeId, sel: NodeId }` representing one arm of a flop's input mux.
  - `Flop` gains `kind: FlopKind` and `arms: Vec<MuxArm>` fields.
- `src/gen/cone.rs`:
  - `build_cone_with_retry` and `build_cone` gain an `exclude: Option<NodeId>` parameter threaded into `pick_terminal`. Used to forbid this flop's own Q from being a leaf in any of its data or select sub-cones.
  - `pick_mux_arm_count` returns M from {0, 2, 3, ..., max_mux_arms}. M = 1 excluded by design (a 1-arm mux is a wire).
  - `drain_flop_worklist` rewritten:
    - For M = 0: D = recursive cone of width N (no mux).
    - For M >= 2: build M data sub-cones (width N) + M select sub-cones (1-bit), every one a recursion point. Assemble `D = OR_i({N{sel_i}} & data_i)`, plus `({N{~(OR sel_i)}} & Q)` for `QFeedback`.
  - New helpers: `assemble_flop_d`, `replicate_to_width` (N-fold Concat of a 1-bit signal), `make_and`, `make_none_selected`, `or_reduce_terms`.
  - `build_flop_leaf` picks a random `FlopKind` per flop (`flop_qfeedback_prob` knob).
- `src/config.rs`:
  - New knobs: `min_mux_arms` (default 1, becomes effective floor of 2 inside `pick_mux_arm_count`), `max_mux_arms` (default 4), `flop_qfeedback_prob` (default 0.5).
  - `Config::validate` checks the mux-arm range and the new probability.
  - New error variant `MuxArmsRange`.
- `src/gen/module.rs`: passes `None` exclusion for output cones.
- `book/src/sequential.md`: documents M=0 vs M>=2 cases, both flop kinds, and the Q-exclusion contract enforced via `exclude: Option<NodeId>`.
- `USER_GUIDE.md`: documents `--min-mux-arms`, `--max-mux-arms`, `--flop-qfeedback-prob` knobs.
- `CODEBASE_ANALYSIS.md`: module map updated for new helpers; invariants list updated.
- `MEMORY.md`: state, next-up, recent commits refreshed.

**Why**
The user specified the precise flop motif `anvil` should generate:
1. M ∈ {0, 2, 3, ...}. M = 0 means no mux, D recurses directly.
2. For M >= 2: each of the M data inputs (width N) is a recursion point; each of the M 1-bit select bits is a recursion point. Selects are one-hot (a design contract, not enforced).
3. Two kinds: `ZeroDefault` (D = 0 on no-select) and `QFeedback` (D = Q on no-select).
4. The flop's own Q is forbidden from feeding any of its data or select sub-cones — the *only* permitted Q→D path is the explicit Q-feedback term in `QFeedback`.

This produces RTL that resembles real synchronous datapath idioms (one-hot-controlled register banks, holding registers, etc.) rather than generic register-of-arbitrary-cone shapes.

**Validation**
- `cargo check`, `cargo test`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --all --check`: all clean.
- Visual inspection of `seed=3, max-depth=2, flop-prob=1.0` confirms:
  - `assign w_X = {bit, bit, ..., bit};` (replicate sel_i to N bits)
  - `assign w_Y = w_X & data_i;` (mask)
  - `assign w_Z = w_A | w_B;` (OR-reduce arm terms)
  - For `QFeedback`: extra `~(OR of sels)` term ANDed with Q.

**Impact**
- Generated flop motifs now match a real-world synchronous-design pattern.
- Tests run slower (~3-4s for the 20-seed sweep vs ~0.04s previously) due to the M+M sub-cone fan-out per flop. Tolerable; tunable via `max_mux_arms` and `max_flops_per_module`.

**Files touched**
`src/ir/types.rs`, `src/config.rs`, `src/gen/cone.rs`, `src/gen/module.rs`, `book/src/sequential.md`, `USER_GUIDE.md`, `CODEBASE_ANALYSIS.md`, `MEMORY.md`, `CHANGES.md`.

---

## 2026-04-15-0003 — Fold flops into the cone recursion (single-clock synchronous discipline)

**Commit hash:** `4317c82`

**What changed**
- `src/gen/cone.rs`:
  - New `FlopWorklist` type alias (`Vec<FlopId>`).
  - `build_cone` now decides between `Gate` and `Flop` at each non-leaf node, gated by `cfg.flop_prob` and `cfg.max_flops_per_module`.
  - New `build_flop_leaf`: allocates a `Flop`, pushes a `FlopQ` node, queues the flop for D-cone construction, returns Q as the leaf for the current cone.
  - New `drain_flop_worklist`: pops queued flops one at a time, recursively builds each D-cone with `build_cone_with_retry` (which itself may push more flops); loops to quiescence.
  - `build_cone_with_retry` now also snapshots/rewinds `m.flops` and the worklist.
  - All flops use `ResetKind::Async` unconditionally (single-CLK / single-RST_N discipline).
  - New `pick_reset_value` (50% zero, 25% all-ones, 25% random).
- `src/gen/module.rs`:
  - Reserves port id 0 for `clk` and 1 for `rst_n`. Sets `Module.clock` and `Module.reset`. Excludes them from the signal pool so cones cannot terminate at them.
  - Drains the flop worklist after building all output cones.
- `src/emit/sv.rs`:
  - Emits `logic [W-1:0] r_<id>;` for every flop.
  - Emits a single `always_ff @(posedge clk or negedge rst_n)` block containing all flops, with reset-branch initializing every flop and else-branch sequencing every flop's D.
  - Conditionally omits `clk`/`rst_n` from the port list when the module has no flops.
- `src/config.rs`:
  - `flop_prob` default raised to `0.15` (was `0.0`).
  - New knob `max_flops_per_module` (default `32`) capping flop count to bound generation time.
- `book/src/sequential.md`:
  - Reframed: flops are part of the same cone recursion, not a later phase.
  - New "Synchronous-design discipline" section spelling out the single-CLK / single-RST_N async constraint.
  - Updated example `always_ff` block.
- `ROADMAP.md`:
  - Phase 1 collapsed: combinational + sequential together. Old Phase 3/5/7 renumbered to new Phase 2/4/6.
- `USER_GUIDE.md`:
  - Updated `flop_prob` default.
  - Documented `max_flops_per_module` knob.
- `DEVELOPMENT_NOTES.md`:
  - Added "Synchronous-design discipline" as a core design decision.
- `CODEBASE_ANALYSIS.md`:
  - Updated module map for new cone helpers.
  - Updated phase coverage map (collapse + renumber).
  - Documented new construction-time invariants (flop allocation, single-clock, clk/rst_n exclusion from pool).
- `MEMORY.md`:
  - Recorded `c4668a2`.
  - Refreshed current state, next-up, open questions, known gaps.

**Why**
The user pointed out that artificially deferring flops to a later phase contradicts the recursion-as-core-principle stance: Q is just another leaf, D is just another sub-cone, the worklist is the same iterative shell that drives output cones. Folding sequential into Phase 1 also unlocks meaningful synthesis testing — purely combinational random RTL is far less representative of real designs than mixed sequential/combinational.

The single-CLK / single-RST_N (async, active-low) constraint matches real fully-synchronous design practice. Enforcing it by construction (no IR field for per-flop clock or polarity) means no random choice can violate it.

**Validation**
- `cargo check --all-targets`, `cargo test` (2 tests pass), `cargo clippy --all-targets -- -D warnings`, `cargo fmt --all --check`: all clean.
- `cargo run -- --seed 7`: produces a module with `always_ff @(posedge clk or negedge rst_n)`, all flops in one block, async-reset to per-flop reset values.
- IR validator passes across the 20-seed sweep with flops enabled.

**Impact**
- Phase 1 is now a meaningful single-module MVP rather than a combinational stub.
- Generated RTL now includes registered state, which is far more representative for downstream synthesis tooling.

**Files touched**
`src/config.rs`, `src/gen/cone.rs`, `src/gen/module.rs`, `src/emit/sv.rs`, `book/src/sequential.md`, `ROADMAP.md`, `USER_GUIDE.md`, `DEVELOPMENT_NOTES.md`, `CODEBASE_ANALYSIS.md`, `MEMORY.md`, `CHANGES.md`.

---

## 2026-04-15-0002 — Elevate "recursion is the core principle" to load-bearing status

**Commit hash:** `c4668a2`

**What changed**
- `README.md`: rewrote the project-objective section as **three** load-bearing principles, with recursion as the first. Recursion is now stated explicitly as the default algorithmic shape for any non-trivial generation step.
- `book/src/core-idea.md`: prepended a "The single guiding principle: recursion" section before the existing thesis. States that recursion is the default; iteration is the exception (flop worklist, per-output driver loop) and exists only to *kick off* recursive cone construction. Anchors the correctness argument: each recursive call carries its own constraints, which is what makes "valid by construction" hold.
- `DEVELOPMENT_NOTES.md`: added recursion as the first entry in the "Core design decisions" recap, with a pointer to the new book section.
- `MEMORY.md`: recorded `5f6022f` (the previous slice's commit hash).

**Why**
The user explicitly stated: "By design, anvil shall be heavily recursive — recursion is its core principle." The design as implemented already follows this, but the docs only hinted at it. Elevating it to first-class status ensures future contributors do not casually replace recursion with iteration in places where the recursion structure is what guarantees invariant preservation.

**Validation**
- Docs-only slice; no code changes.
- `cargo check`, `cargo test`: still clean (no source touched).

**Impact**
- Future PRs that introduce iterative scaffolding around generation logic should now expect to justify the choice against the "recursion is the default" principle.

**Files touched**
`README.md`, `book/src/core-idea.md`, `DEVELOPMENT_NOTES.md`, `MEMORY.md`, `CHANGES.md`.

---

## 2026-04-15-0001 — Initial scaffold + Phase 1 cone-adapter hardening

**Commit hash:** `5f6022f`

**What changed**
- Created Cargo project `anvil` with binary + library targets.
- Added `Cargo.toml` with deps: `rand`, `rand_chacha`, `clap` (derive), `serde`, `serde_json`, `thiserror`, `anyhow`.
- Added crate skeleton:
  - `src/lib.rs` — public re-exports (`Config`, `Generator`, `Module`).
  - `src/main.rs` — CLI (`--seed`, `--count`, `--out`, `--config`, `--dump-config`, knob overrides).
  - `src/config.rs` — `Config` struct, defaults, `validate()`, CLI overlay.
  - `src/ir/types.rs` — `Module`, `Port`, `Node`, `GateOp`, `Flop`, `DepSet`.
  - `src/ir/validate.rs` — IR invariant checker (safety net).
  - `src/gen/mod.rs` — `Generator` entry points, ChaCha8-seeded RNG.
  - `src/gen/module.rs` — leaf-module generator (N inputs, M outputs, cone per output).
  - `src/gen/cone.rs` — fanin-cone recursion with depth budget, anti-collapse rules, dep-set tracking, bounded retry on trivial cones.
  - `src/gen/pool.rs` — `SignalPool` for terminal selection.
  - `src/emit/sv.rs` — IR → SystemVerilog pretty-printer.
- Added `tests/pipeline.rs` — generates 20 seeds, asserts IR validation passes and SV output is non-empty; reproducibility test.
- Added `examples/generate_one.rs` — minimal library-usage example.
- Added live-doc set:
  - `README.md` — entry point.
  - `SESSION_BOOTSTRAP.md` — read-first on session recovery.
  - `ROADMAP.md` — 7-phase plan, exit criteria per phase.
  - `USER_GUIDE.md` — CLI, knobs, downstream verification.
  - `MEMORY.md` — operational continuity snapshot.
  - `CHANGES.md` (this file).
  - `DEVELOPMENT_NOTES.md` — engineering rationale.
  - `CODEBASE_ANALYSIS.md` — live workspace analysis.
  - `COMMIT.md` — commit workflow.
- Added mdBook design rationale at `book/`:
  - `core-idea.md`, `why-not-grammar.md`, `algorithm.md`, `ir.md`,
    `by-construction.md`, `synthesizability.md`, `non-triviality.md`,
    `sequential.md`, `sharing.md`, `hierarchy.md`, `knobs.md`,
    `architecture.md`, `non-goals.md`.
- Added `.gitignore` covering `/target`, `book-out`, `Cargo.lock`, swap files, and `git_message_brief.txt`.
- **Phase 1 hardening:** lazy width-adapter in `gen::cone::pick_terminal`. When the signal pool has no matching-width entry, build a Slice (or replicating Concat + Slice) from the widest available pool entry with non-empty deps, instead of falling back to a bare constant. Preserves dep-set propagation and resolves the seed-0 IR-validation failure where output cones were collapsing to constants.
- Added `gen::cone::make_width_adapter` helper.
- `gen::pool::SignalPool::iter()` exposed for adapter source selection.
- Clippy cleanups: `Config { seed, ..Default::default() }` patterns in tests/example; `u32::div_ceil` for adapter copy count.
- All `cargo fmt` corrections applied.

**Why**
Project bootstrap. The brainstorming session that preceded this slice converged on a circuit-graph-IR generator with by-construction validity, dep-set tracking for non-triviality, and explicit synthesizability-as-subset enforcement.

The lazy adapter fixes a Phase 1 bug surfaced on the first `cargo test` run: when randomly-chosen output port widths do not match any randomly-chosen input port width, the cone has no signal of the required width to terminate at, falls back to a constant, and the cone root's dep-set is empty. The validator correctly rejects this, but the bounded retry loop cannot recover because the pool composition does not change between attempts. The adapter resolves this structurally — any output width can now reach an input via Slice/Concat — without weakening the by-construction discipline.

**Validation**
- `cargo check --all-targets` clean.
- `cargo test`: 2 tests pass (`generates_valid_modules_across_seeds` over seeds 0..20, `reproducibility` byte-identical for seed 12345).
- `cargo clippy --all-targets -- -D warnings`: clean.
- `cargo fmt --all --check`: clean.
- `cargo run -- --seed 42`: produces a 4-output, 3-input module with a coherent assign net (visual spot-check).
- `cargo run -- --seed 7 --count 5 --out /tmp/anvil_out`: 5 .sv files + manifest.json written.
- External smoke tests (Verilator, Yosys): tools not installed locally; smoke runs are deferred until the dev environment provides them or CI is wired.

**Impact**
- Phase 0 (Scaffolding) exit criteria met: `cargo build` and `cargo test` pass.
- Phase 1 (Combinational MVP) is in progress: cone recursion functional and dep-set-correct across the seed sweep; remaining Phase 1 work is per-gate width-rule validation in `ir::validate`, unit tests inside source modules, and Verilator-lint smoke once available.
- `CODEBASE_ANALYSIS.md` "Known weaknesses" item #1 is resolved by this slice.

**Files touched**
All files in the repository (initial creation), plus subsequent edits to `src/gen/cone.rs`, `src/gen/pool.rs`, `tests/pipeline.rs`, `examples/generate_one.rs`.
