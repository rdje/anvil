# HIERARCHY-AWARE-IDENTITY: Extend NodeId identity doctrine to hierarchical modules

## Metadata

- Tree ID: `HIERARCHY-AWARE-IDENTITY`
- Status: `active`
- Roadmap lane: Phase 4 — Hierarchy
- Created: `2026-05-14`
- Last updated: `2026-05-15` (HIERARCHY-AWARE-IDENTITY.2 landed)
- Owner: repo-local workflow

## Goal

Extend ANVIL's "NodeId = identity of an expression" doctrine up one level to
**"ModuleId = identity of a hierarchical module template"** so that
structurally-identical `Module` definitions in a generated `Design` collapse
to a single entry, in the same spirit that structurally-identical expressions
collapse to a single `NodeId` today.

Concretely, this tree closes when the generator can be asked (via
`IdentityMode::NodeId` at the hierarchy level, or an equivalent knob) to
dedupe `Design::modules` by canonical structural signature, and the matrix
gate proves the dedup is downstream-clean.

## Non-Goals

- Across-design module sharing (one corpus of modules reused by many
  unrelated designs). Out of scope: each `Design` still owns its own module
  set.
- Semantic / behavioural equivalence beyond pure structural isomorphism.
  Two modules that compute the same function via different gate sequences
  remain distinct under this tree.
- A new emitter-level optimisation pass. Dedup happens at IR construction
  time, not after `emit::sv`.
- Replacing the existing `IdentityMode::NodeId` semantics for gate-level
  factorization. The hierarchy-level identity is a new layer that *extends*
  the doctrine; it does not change gate-level behaviour.

## Acceptance Criteria

- A canonical signature per `Module` is computed deterministically and is
  isomorphism-aware (instance child names do not perturb the hash).
  **`HIERARCHY-AWARE-IDENTITY.1` — landed in r85.**
- The generator can be asked to dedupe `Design::modules` by signature, with
  the toggle wired into `IdentityMode` (or an explicit hierarchy-identity
  knob if that is cleaner).
- The matrix gate proves dedup is downstream-clean: every Verilator/Yosys
  scenario stays green when dedup is active.
- A focused proof shows that the planner can produce structurally-identical
  module pairs and that the dedup pass collapses them.
- Live docs (`README.md`, `ROADMAP.md`, `USER_GUIDE.md`, `book/src/*.md`)
  describe the new identity layer alongside the existing gate-level doctrine.

## Task Tree

- ID: `HIERARCHY-AWARE-IDENTITY`
  Status: `active`
  Goal: `Extend NodeId-style identity doctrine to module-level structural identity, with dedup live under an opt-in knob and proven downstream-clean.`
  Children: `HIERARCHY-AWARE-IDENTITY.1`, `HIERARCHY-AWARE-IDENTITY.2`, `HIERARCHY-AWARE-IDENTITY.3`, `HIERARCHY-AWARE-IDENTITY.4`, `HIERARCHY-AWARE-IDENTITY.5`

- ID: `HIERARCHY-AWARE-IDENTITY.1`
  Status: `done`
  Goal: `Canonical per-module signature instrumentation as r85 (DesignMetrics.canonical_module_signatures + diversity coverage fact + focused stability/isomorphism proof + matrix scenario).`
  Acceptance: `r85 gate is downstream-clean at 204 scenarios / 816 designs; focused proof passes for all four ConstructionStrategy values; saw_recursive_hierarchy_canonical_module_signature_diversity fires.`
  Verification: `Full r85 gate downstream-clean — 204 scenarios, 816 designs, coverage_gaps = [], Verilator/Yosys all 816/0, saw_recursive_hierarchy_canonical_module_signature_diversity = true. Focused proof canonical_module_signatures_are_stable_and_isomorphism_aware passes ~0.35s release for all four ConstructionStrategy values. cargo fmt --all -- --check, mdbook build book clean.`
  Commit: `Phase 4: add canonical module signatures (r85, HIERARCHY-AWARE-IDENTITY.1)`

- ID: `HIERARCHY-AWARE-IDENTITY.2`
  Status: `done`
  Goal: `Add a structural-duplicate metric that flags when the planner currently emits duplicate Modules (num_structurally_duplicate_module_pairs > 0 in at least one matrix scenario, via a focused config that deliberately produces near-identical sub-trees).`
  Acceptance: `A focused proof produces a Design with num_structurally_duplicate_module_pairs > 0; the gate exercises that proof via a new scenario and a new saw fact saw_design_with_structurally_duplicate_modules.`
  Verification: `Full r86 gate downstream-clean — 207 scenarios, 828 designs, coverage_gaps = [], Verilator/Yosys all 828/0, saw_design_with_structurally_duplicate_modules = true. Focused proof planner_can_emit_structurally_duplicate_modules passes ~0.08s release for all four ConstructionStrategy values under tight 1-in/1-out/width-1 leaf constraints (max_depth=1, terminal_reuse_prob=1.0). cargo fmt --all -- --check, mdbook build book clean. Calibration discovery: default leaf-width / leaf-input ranges produce zero duplicates across a 500-config sweep, because the leaf generator's RNG advances between calls. The tight-leaf constraint collapses every library leaf to the same canonical structure.`
  Commit: `Phase 4: prove planner emits structurally-duplicate Modules (r86, HIERARCHY-AWARE-IDENTITY.2)`

- ID: `HIERARCHY-AWARE-IDENTITY.3`
  Status: `done`
  Goal: `Design sketch in DEVELOPMENT_NOTES.md for the dedup pass: where it sits in the pipeline (IR construction time vs. post-finalisation), how instance.module references are remapped after a Module is folded into a peer, how IdentityMode interacts with it, and what the proof shape looks like.`
  Acceptance: `A DEVELOPMENT_NOTES.md entry describes the pass with at least one rejected alternative recorded; no code change yet.`
  Verification: `DEVELOPMENT_NOTES.md "Design notes / Module-dedup pass design sketch (2026-05-15, HIERARCHY-AWARE-IDENTITY.3)" entry records pipeline placement, instance-rewrite policy, toggle/API choice, edge cases, proof shape, and open questions. Three rejected alternatives recorded (incremental dedup during construction; dedup as emitter pass; extending IdentityMode). mdbook build clean; no code change.`
  Commit: `Phase 4: dedup-pass design sketch (HIERARCHY-AWARE-IDENTITY.3)`

- ID: `HIERARCHY-AWARE-IDENTITY.4`
  Status: `pending`
  Goal: `Implement the dedup pass per the H-A-I.3 sketch, guarded by an opt-in toggle (IdentityMode::NodeId at the hierarchy level or a dedicated config knob). Default behaviour stays identical to today — never retire existing modes.`
  Acceptance: `cargo test all green; focused proof shows a Design with two structurally-identical Modules collapses to one when the toggle is on and remains two when off; cargo fmt / clippy / mdbook build clean.`
  Verification: `pending`
  Commit: `pending`

- ID: `HIERARCHY-AWARE-IDENTITY.5`
  Status: `pending`
  Goal: `Matrix gate proves dedup is downstream-clean: add a focused scenario with the dedup toggle on, run the full Phase 4 hierarchy gate, prove Verilator/Yosys all-green and a new saw fact saw_recursive_hierarchy_module_dedup_active fires.`
  Acceptance: `Full hierarchy gate green at the new scenario count, including both with-dedup and without-dedup configurations; matrix coverage_gaps stays [].`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `HIERARCHY-AWARE-IDENTITY.4` | `pending` | Implement the dedup pass per the design sketch in `DEVELOPMENT_NOTES.md`: new `src/ir/dedup.rs`, fixed-point iteration grouping Modules by canonical signature with lexicographic-smallest-name survivor, opt-in via a new `hierarchy_module_dedup: bool` Config knob. Default-off preserves current behaviour. |

`H-A-I.5` is NOT on the frontier yet — becomes eligible only after
`H-A-I.4` is `done`.
`H-A-I.1` (r85), `H-A-I.2` (r86), and `H-A-I.3` (design sketch) are all
`done`. Hashes are in this tree's Commit Log.

## Decisions

- `2026-05-14`: Adopted the FSMGen task-tree workflow on ANVIL, scoped to
  multi-slice tasks like this one. Linear `rN` slices keep their existing
  convention. Rationale: gate-style coverage extensions (r73–r82, r83, r84)
  already had clean handoff under the `rN` + `CHANGES.md` + `MEMORY.md`
  combination; the value of task-tree is highest where decomposition,
  blockers, and pause/resume cycles dominate, which describes this tree
  exactly.
- `2026-05-13`: Signature definition deliberately excludes
  `instance.module` and `instance.name`. The whole point of structural
  isomorphism for dedup is that two parents instantiating distinctly-named
  but identically-shaped children should be detected as identical.
  Including the child-module string would defeat the detector before dedup
  ever runs.
- `2026-05-13`: Signature uses a hand-rolled FNV-1a 64-bit hash instead of
  pulling in `ahash` / `seahash` / `rustc_hash`. The IR crate is
  dependency-light; reproducibility across Rust versions is required;
  FNV-1a satisfies both with negligible code.

## Open Questions

- Should the hierarchy-level identity toggle be a new variant on the
  existing `IdentityMode` enum (extending the existing two
  Relaxed / NodeId), a separate knob (e.g.,
  `hierarchy_module_dedup_mode`), or an extension of
  `FactorizationLevel`? Owner: user-facing API decision. Does not block
  `H-A-I.1`–`H-A-I.3`. Recorded for `H-A-I.4` design.
- Does dedup need to preserve the original module names anywhere
  (e.g., in `DesignMetrics` debug output) so that traces of the
  pre-dedup planner state are still inspectable? Owner: tooling/UX.
  Recorded for `H-A-I.4`.
- Will the dedup pass interact with the `library` vs `on-demand` child
  sourcing modes differently? The `library` mode already shares definitions;
  the `on-demand` mode emits fresh per slot. Owner: gen/hierarchy design.
  Recorded for `H-A-I.3` design sketch.

## Blockers

- None on `H-A-I.1`–`H-A-I.3`. Frontier is unblocked.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-05-14` | `HIERARCHY-AWARE-IDENTITY.1` | `cargo test --release --test pipeline canonical_module_signatures_are_stable_and_isomorphism_aware`; `cargo test --bin tool_matrix --release phase4_hierarchy` (3/3 unit tests); full r85 hierarchy gate. | All passing. Gate: 204 scenarios / 816 designs, `coverage_gaps = []`, Verilator/Yosys all 816/0, `saw_recursive_hierarchy_canonical_module_signature_diversity = true`. `cargo fmt --all -- --check`, `mdbook build book` clean. |
| `2026-05-15` | `HIERARCHY-AWARE-IDENTITY.2` | `cargo test --release --test pipeline planner_can_emit_structurally_duplicate_modules`; `cargo test --bin tool_matrix --release phase4_hierarchy` (3/3 unit tests); full r86 hierarchy gate. | All passing. Gate: 207 scenarios / 828 designs, `coverage_gaps = []`, Verilator/Yosys all 828/0, `saw_design_with_structurally_duplicate_modules = true`. Tight 1-in/1-out/width-1 leaf constraints collapse the leaf generator's RNG-driven choices to a single canonical structure. `cargo fmt --all -- --check`, `mdbook build book` clean. |
| `2026-05-15` | `HIERARCHY-AWARE-IDENTITY.3` | `mdbook build book`; design sketch reviewed for completeness against the leaf's acceptance criteria. | Design sketch landed in `DEVELOPMENT_NOTES.md` under "Module-dedup pass design sketch (2026-05-15, HIERARCHY-AWARE-IDENTITY.3)". Records pipeline placement (post-finalisation, new `src/ir/dedup.rs`), instance-rewrite policy (fixed-point iteration, lexicographic-smallest-name survivor), toggle/API choice (new `Config::hierarchy_module_dedup: bool`, default false), edge cases (top must survive, library-mode dedup is no-op, fixed-point termination via strict decrease), proof shape for `H-A-I.4`, and open questions. Three rejected alternatives recorded: incremental dedup during construction; dedup as emitter pass; extending `IdentityMode`. |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `HIERARCHY-AWARE-IDENTITY.1` | `Phase 4: add canonical module signatures (r85, HIERARCHY-AWARE-IDENTITY.1)` | First task-tree-managed code slice on ANVIL. |
| `HIERARCHY-AWARE-IDENTITY.2` | `Phase 4: prove planner emits structurally-duplicate Modules (r86, HIERARCHY-AWARE-IDENTITY.2)` | Existence proof: dedup is real and applicable to ANVIL's planner. |
| `HIERARCHY-AWARE-IDENTITY.3` | `Phase 4: dedup-pass design sketch (HIERARCHY-AWARE-IDENTITY.3)` | Pure design slice; design sketch landed in DEVELOPMENT_NOTES.md. |

## Changelog

- `2026-05-14`: Created task tree as part of FSMGen task-tree workflow adoption on ANVIL. `H-A-I.1` recorded as `in_progress` because r85's source/docs were already in the working tree.
- `2026-05-14`: `H-A-I.1` landed downstream-clean. Status -> `done`. Frontier rotated to `H-A-I.2`.
- `2026-05-15`: `H-A-I.2` landed downstream-clean as r86. Status -> `done`. Frontier rotated to `H-A-I.3`.
- `2026-05-15`: `H-A-I.3` design sketch landed in `DEVELOPMENT_NOTES.md`. Status -> `done`. Frontier rotated to `H-A-I.4`.
