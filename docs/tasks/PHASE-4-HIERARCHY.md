# PHASE-4-HIERARCHY: Exhaust the Phase 4 hierarchy surface

## Metadata

- Tree ID: `PHASE-4-HIERARCHY`
- Status: `active`
- Roadmap lane: Phase 4 — Hierarchy
- Created: `2026-05-16`
- Last updated: `2026-05-16` (`.1` audit landed; `.2` superseded; frontier → `.3` closure)
- Owner: repo-local workflow

## Goal

Drive Phase 4 (Hierarchy) to honest, complete exhaustion: every remaining
hierarchy structural surface that ANVIL should support before Phase 5 is
landed, proven downstream-clean in the repo-owned Phase 4 matrix gate, and
documented — at which point `ROADMAP.md` Phase 4 can be promoted from
`in progress` to `done`.

"Exhaustion" here is finite and concrete: the set of remaining surfaces is
enumerated up front (leaf `.1`) so the tree has a bounded frontier rather
than an open-ended coverage grind. The completed sub-objective
`HIERARCHY-AWARE-IDENTITY` (r85→r87) is **not** re-tracked here; this tree
owns the remaining *breadth* of hierarchy structure.

## Non-Goals

- Parameterization of hierarchy (parameter-aware child selection,
  parameter-driven parent generation). That is Phase 5 by roadmap
  decree; this tree only ensures Phase 4 is a "real design/instance
  layer" so Phase 5 is unblocked.
- Hierarchy-aware identity. Already delivered by the closed
  `HIERARCHY-AWARE-IDENTITY` tree (`Config::hierarchy_module_dedup`).
- Multi-clock / CDC hierarchy. That is Phase 6.
- Retiring the `rN` cadence. `rN` remains the within-leaf slice cadence;
  this tree tracks the sub-objective decomposition, exactly as
  `HIERARCHY-AWARE-IDENTITY` did (its leaves landed as r85/r86/r87).

## Acceptance Criteria

- A definitive landed-vs-missing audit of the hierarchy surface exists,
  enumerating every remaining registered/helper/composition pattern as a
  concrete leaf (no open-ended "broader patterns" hand-wave).
- Each enumerated remaining surface is implemented as a construction-time
  rule (no generate-then-filter), default-off where it changes existing
  output, and proven downstream-clean in the Phase 4 hierarchy matrix
  gate (`coverage_gaps = []`, Verilator + both Yosys modes all-pass).
- `ROADMAP.md` gains an explicit Phase 4 exit-criteria block, and Phase 4
  is promoted to `done` only once those criteria are satisfied and
  visible in repo-owned artifacts.
- Live docs (`README.md`, `ROADMAP.md`, `USER_GUIDE.md`,
  `CODEBASE_ANALYSIS.md`, `book/src/hierarchy.md`, `book/src/knobs.md`)
  describe every new surface and knob.

## Task Tree

- ID: `PHASE-4-HIERARCHY`
  Status: `active`
  Goal: `Land every remaining Phase 4 hierarchy surface, prove it downstream-clean, and exit-criteria-gate Phase 4 to done.`
  Children: `PHASE-4-HIERARCHY.1` (done), `PHASE-4-HIERARCHY.2` (superseded by .3), `PHASE-4-HIERARCHY.3`

- ID: `PHASE-4-HIERARCHY.1`
  Status: `done`
  Goal: `Audit gen/hierarchy.rs + the Phase 4 coverage-fact set + the matrix scenario list and produce a definitive landed-vs-missing table of hierarchy surfaces. Output: a "Surface Inventory" section appended to this file. No behaviour change.`
  Acceptance: `This file gains a Surface Inventory table + conclusion; the tree's Children list and Current Frontier are rewritten from it; doc-only slice.`
  Verification: `Read-only audit of src/config.rs, src/gen/hierarchy.rs, src/bin/tool_matrix.rs (CoverageSummary + summarize_design_coverage + compute_coverage_gaps), README.md, ROADMAP.md, cross-referenced against the banked r87 report. Conclusion: 92 hierarchy saw_* facts are Phase4-gated and all true in r87, coverage_gaps=[], 840/0 downstream-clean — NO missing surface; "broader registered hierarchy patterns" is genuinely open-ended (capability-deepening, not a finite gap set). See Surface Inventory below.`
  Commit: `Docs(PHASE-4-HIERARCHY.1): Surface Inventory — Phase 4 hierarchy surface is landed-proven`

- ID: `PHASE-4-HIERARCHY.2`
  Status: `superseded`
  Goal: `(was) Implement the first missing hierarchy surface identified by .1.`
  Acceptance: `n/a`
  Verification: `Superseded by PHASE-4-HIERARCHY.3. Premise invalidated by .1: the audit found NO missing instrumented surface (gap list already empty in r87), and the residual roadmap text ("broader registered hierarchy patterns") is open-ended by construction, not a finite implementable leaf. Manufacturing permutation slices would be unbounded busywork, not completion. Replacement: .3 (exit-criteria scope-cut closure).`
  Commit: `n/a (superseded)`

- ID: `PHASE-4-HIERARCHY.3`
  Status: `pending`
  Goal: `Phase 4 exit-criteria closure as a deliberate, evidence-backed scope cut: add an explicit Phase 4 exit-criteria block to ROADMAP.md, show the repo-owned r87 artifact satisfies it (coverage_gaps=[], 840/0 Verilator+both Yosys, full hierarchy saw_* surface proven, real design/instance layer sufficient to unblock Phase 5), and promote ROADMAP Phase 4 in progress -> done.`
  Acceptance: `ROADMAP.md has a Phase 4 exit-criteria block; the closing artifact is the banked r87 gate; ROADMAP Phase 4 label = done; README/CODEBASE_ANALYSIS/MEMORY/book synced; PHASE-4-HIERARCHY tree -> done.`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `PHASE-4-HIERARCHY.3` | `pending` | `.1` proved the surface is landed-proven and the gap list empty; `.2` superseded. The only honest path to "complete exhaustion" is the deliberate exit-criteria scope cut + `done` promotion on the r87 evidence. |

## Decisions

- `2026-05-16`: Phase 4 "complete exhaustion" is bounded by an explicit
  up-front Surface Inventory (leaf `.1`) rather than treated as an
  open-ended coverage grind. Rationale: the roadmap's "broader registered
  hierarchy patterns remain future work" is not a finite exit condition;
  a concrete enumerated inventory makes the frontier finite and the
  `done` promotion defensible.
- `2026-05-16`: `rN` is preserved as the within-leaf slice cadence. This
  tree only owns the decomposition, mirroring the closed
  `HIERARCHY-AWARE-IDENTITY` precedent. Consistent with the
  never-retire-strategies rule and `docs/TASK_TREE.md` adoption scope.
- `2026-05-16`: `ROADMAP.md` currently has no Phase 4 exit-criteria
  block. Authoring one is in-scope for this tree (leaf `.3`); Phase 4
  must not be promoted to `done` from narrative progress alone.
- `2026-05-16` (`.1` audit outcome): the instrumented Phase 4 hierarchy
  surface is **fully landed-proven** — 92 hierarchy `saw_*` facts are
  Phase4-gated and all `true` in the banked r87 report, with
  `coverage_gaps = []` and 840/0 downstream-clean (Verilator + both
  Yosys modes). There is **no missing surface to implement**. The
  roadmap's "broader registered hierarchy patterns remain future work"
  is genuinely open-ended capability-deepening, not a finite gap set, so
  it cannot be "drained". **Decision:** Phase 4 "complete exhaustion" is
  therefore a *deliberate, evidence-backed scope cut* — declare the
  proven 92-fact gated surface a sufficient real design/instance layer
  (the Phase 5 prerequisite), author explicit ROADMAP exit criteria that
  the r87 artifact already satisfies, and promote Phase 4 to `done`.
  This is **not** retiring any mode/strategy (never-retire-strategies is
  untouched — every implemented route stays); it stops an unbounded
  permutation grind that has no completion point. `.2` is `superseded`
  accordingly; the frontier is `.3` (closure).

## Open Questions

- Resolved by the `.1` audit: there are no `blocking-phase5` vs
  `breadth` MISSING rows to classify — the instrumented surface is
  entirely landed-proven. The only residual ("broader patterns") is
  open-ended by construction and explicitly scope-cut by the decision
  above. No open question blocks `.3`.

## Blockers

- None. `.3` is a doc-only closure slice over already-banked repo
  evidence (r87); no code change, no new gate run required (the closing
  artifact already exists and was independently verified during the
  session-recovery finalization of r87).

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-05-16` | `PHASE-4-HIERARCHY.1` | Read-only audit of config.rs / gen/hierarchy.rs / bin/tool_matrix.rs (CoverageSummary, summarize_design_coverage, compute_coverage_gaps) vs README/ROADMAP, cross-referenced to the banked r87 report. | Done. 92 Phase4-gated hierarchy `saw_*` facts all true in r87; `coverage_gaps = []`; 840/0 Verilator + both Yosys. No MISSING/AMBIGUOUS surface. "Broader patterns" = open-ended. Doc-only; no code touched. |
| `2026-05-16` | `PHASE-4-HIERARCHY.3` | `pending` | `pending` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `PHASE-4-HIERARCHY.1` | `Docs(PHASE-4-HIERARCHY.1): Surface Inventory — Phase 4 hierarchy surface is landed-proven` | Audit slice; restructured the tree (`.2` superseded, frontier → `.3`). |
| `PHASE-4-HIERARCHY.3` | `pending` | `pending` |

## Changelog

- `2026-05-16`: Created task tree as part of the directive to task-tree
  every remaining roadmap phase. Frontier opened at `.1` (Surface
  Inventory audit).
- `2026-05-16`: `.1` landed. Audit conclusion: Phase 4 hierarchy surface
  is fully landed-proven (r87, gap list empty); no missing surface;
  "broader patterns" is open-ended. `.2` -> `superseded` (premise
  invalidated). Frontier rotated to `.3` (exit-criteria scope-cut
  closure). Decision recorded above.

## Surface Inventory (`PHASE-4-HIERARCHY.1`)

Source: read-only audit of `src/config.rs`, `src/gen/hierarchy.rs`,
`src/bin/tool_matrix.rs` (`CoverageSummary` + `summarize_design_coverage`
+ `compute_coverage_gaps`), `README.md` lines ~800-836, `ROADMAP.md`
Phase 4 (~254-485), cross-referenced against the banked r87 report
`/tmp/anvil-tool-matrix-phase4-hierarchy-r87/tool_matrix_report.json`
(210 scenarios, 840 designs, `coverage_gaps = []`, 840/0
Verilator + Yosys-no-abc + Yosys-with-abc).

**Headline:** of the 105 `CoverageSummary` `saw_*` fields, 92
hierarchy-specific facts are gated as Phase4 coverage gaps and **all are
`true` in r87**; 10 generic block facts are also gated and true; the
only non-gated facts are `saw_flop_merge` / `saw_semantic_gate_merge`
(intra-module identity, not a hierarchy surface) and `saw_variable_shift`
(a Phase 3 fact). **No hierarchy surface is MISSING or AMBIGUOUS.**

| Hierarchy surface family | Gating knob(s) | Status |
| --- | --- | --- |
| Hierarchy/multifile design, instance + instance-output nodes, declared top | `hierarchy_depth` / range | LANDED-PROVEN |
| Library reuse / under-instantiation; on-demand profiled child synthesis | `num_child_instances`, `hierarchy_child_source_mode` | LANDED-PROVEN |
| Combinational sibling routing; combinational parent-composed child-input cones (+mixed support, +recursive non-top) | `hierarchy_sibling_route_prob`, `hierarchy_child_input_cone_prob`, `hierarchy_parent_flop_prob` | LANDED-PROVEN |
| Registered sibling routing (+mixed support, +multi-stage, +recursive non-top) | `hierarchy_registered_sibling_route_prob`, `hierarchy_registered_sibling_mixed_support_prob` | LANDED-PROVEN |
| Registered parent-composed child-input (+mixed, +multi-stage, +3-stage chain, +recursive non-top) | `hierarchy_registered_child_input_cone_prob`, `hierarchy_parent_flop_prob` | LANDED-PROVEN |
| Parent-cone helper instances feeding child-input / sibling / registered-D / parent-output / through parent-local Q (+mixed, +recursive) | `hierarchy_parent_cone_instance_prob`, `max_parent_cone_instances_per_module` | LANDED-PROVEN |
| Multi-helper budgets (≥3, =5, child-input, through-flops, recursive non-top) | `max_parent_cone_instances_per_module` | LANDED-PROVEN |
| Parent-local flops; parent-port-composed parent outputs (stateful, recursive non-top, exact depths 3–7) | `hierarchy_parent_flop_prob` | LANDED-PROVEN |
| Recursive tree shape, mixed leaf depth, per-depth branching overrides | range knobs, `child_instances_per_module_by_depth` | LANDED-PROVEN |
| Hierarchy-aware identity (canonical signatures, structural-duplicate proof, opt-in module-dedup pass) | `hierarchy_module_dedup` | LANDED-PROVEN (HIERARCHY-AWARE-IDENTITY tree, r85–r87) |

**Conclusion.** Phase 4's implemented hierarchy surface is exhausted in
the only bounded sense available: every instrumented surface is landed
and proven downstream-clean in a repo-owned artifact, and the gap list
is empty. The residual roadmap phrase "broader registered hierarchy
patterns" is open-ended capability-deepening with no completion point;
treating it as a gap to drain would be an unbounded grind, not
completion. Phase 4 is therefore closed by `.3` as a deliberate,
documented, evidence-backed scope cut, not by manufacturing further
permutation slices.
