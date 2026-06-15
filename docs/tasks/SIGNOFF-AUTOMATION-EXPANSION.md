# SIGNOFF-AUTOMATION-EXPANSION: Broaden Downstream Signoff Automation

## Metadata

- Tree ID: `SIGNOFF-AUTOMATION-EXPANSION`
- Status: `active`
- Roadmap lane: `Quality — downstream signoff automation breadth`
- Created: `2026-06-15`
- Last updated: `2026-06-15` (`.2` split into `.2a` design + `.2b` impl per the `.3a`/`.3b` precedent; `.2a` done — exact knob batch / scenario shapes / fact names / gate fixed in `DEVELOPMENT_NOTES.md`; frontier → `.2b`)
- Owner: repo-local workflow

## Goal

Broaden the downstream signoff-acceptance automation beyond the current
per-phase gates, in service of the `project_anvil_north_star` purpose
(surface downstream-tool bugs via valid-by-construction,
downstream-acceptance-quality output). Candidate directions, to be
prioritized in `.1`:

- richer **tool/knob sweep coverage** — exercise the adversarial axis
  matrix (construction strategy, identity mode, factorization level,
  motif/category mix, sequential density, width/depth ranges, the
  probability knobs) without hidden bias toward whichever path is
  currently easiest;
- additional **simulator/frontend acceptance columns** where available
  (extending the Verilator / Yosys / Icarus / `--diff-sim` surface);
- additional **valid-by-construction synthesizable artifact families**
  beyond today's DUT / microdesign / frontend lanes.

This lane is Lane 3 of the three owner-directed post-phase capability
lanes; it is opened `proposed` and promoted to `active` after
`AGENT-MCP-EXPANSION` reaches handoff.

## Non-Goals

- No weakening of the warning-as-failure discipline; warnings stay
  failures, counterexamples are retained with exact seed + effective
  knobs and fed back into invariants/rewrites, never hidden behind
  suppressions.
- No generate-then-filter (rules-first only, per
  `feedback_rules_first_generation`).
- No whole-module spec/oracle or shadow simulator (per `ROADMAP.md`
  non-goal 4 / `book/src/non-goals.md`).
- No retirement of any existing gate or scenario axis (per
  `feedback_never_retire_strategies`).

## Acceptance Criteria

- Each landed leaf adds proven downstream-acceptance coverage (new
  sweep, column, or artifact family) with repo-owned, banked evidence
  (a `tool_matrix`/parity report or equivalent), not narrative claims.
- Default-off / byte-identical wherever a knob could change emitted RTL.
- Live docs (`USER_GUIDE.md`, `ROADMAP.md`, the relevant book chapter)
  updated for any new user-visible gate or column.
- Every leaf committed through `COMMIT.md` with its leaf ID in the
  subject.

## Task Tree

- ID: `SIGNOFF-AUTOMATION-EXPANSION`
  Status: `active`
  Goal: `Broaden downstream signoff automation (sweeps, columns, artifact families).`
  Children: `SIGNOFF-AUTOMATION-EXPANSION.1`, `SIGNOFF-AUTOMATION-EXPANSION.2`

- ID: `SIGNOFF-AUTOMATION-EXPANSION.1`
  Status: `done`
  Goal: `Design/decision leaf: inventory the current gates/columns/axes, identify the single highest-value next signoff-automation increment, and scope the first implementation leaf (and split the tree).`
  Acceptance: `A decision record naming the chosen first increment with rationale; no source change; docs/workflow validation clean.`
  Result: `Inventoried the current signoff surface (4 acceptance columns Verilator/Yosys-without-abc/Yosys-with-abc/iverilog-compile + opt-in --diff-sim, all DUT-lane only; ~112 saw_* coverage facts; phase1-4 + share/structured/hierarchy gates; non-DUT lanes only in separate parity gates). Recorded decision 0006: the first increment is "richer adversarial knob-sweep coverage" — promote currently-unswept-but-existing generator knobs (operand/mux-arm duplication, width_parameterization_prob, aggregate/aggregate_array_prob, memory×fsm interplay) into explicit first-class tool_matrix scenario axes + saw_* coverage facts (+ a focused gate), so they fire by construction not by chance, closing ROADMAP steering gap 3's hidden-bias hole. Chosen on the deciding-factor test "a day-one failure must be a real downstream signal, not noise": a newly-swept LEGAL knob combo that trips Verilator/Yosys is the north-star signal, whereas a new aggressive-synth/techmap column produces non-actionable noise (the with-abc softening precedent), formal/SVA has no spec to prove (structure-first non-goal), and frontend stub-child corpora are not end-to-end synthesizable. Higher-ceiling paths (new tool columns; non-DUT lanes under acceptance) preserved as future leaves (nothing retired). Split the tree forward: added .2 (implement the first knob-sweep batch). Decision 0006 carries Knowledge Map answers front-matter. No source change.`
  Verification: `scripts/check_memory_architecture.sh (incl. 0006 indexed) + knowledge-map regen/check; docs/decision + task-tree edits; no source change`
  Commit: `SIGNOFF-AUTOMATION-EXPANSION.1 — design/decision leaf + decision 0006`

- ID: `SIGNOFF-AUTOMATION-EXPANSION.2`
  Status: `active`
  Goal: `Implement the first richer-knob-sweep batch (per decision 0006): promote the highest-bias unswept knobs into explicit tool_matrix scenario axes + saw_* coverage facts + a focused gate; default-off / byte-identical where a knob changes RTL; banked clean across Verilator + both Yosys modes.`
  Children: `SIGNOFF-AUTOMATION-EXPANSION.2a`, `SIGNOFF-AUTOMATION-EXPANSION.2b`

- ID: `SIGNOFF-AUTOMATION-EXPANSION.2a`
  Status: `done`
  Goal: `Design leaf (docs-only): fix the exact first knob batch, the per-knob scenario shapes, the saw_* fact names + the metric each is proved from, the focused gate, and the gap wiring — resolving the open question 0006 left to .2.`
  Acceptance: `A DEVELOPMENT_NOTES.md design entry naming the four genuinely-unswept knobs (operand_duplication_rate, mux_arm_duplication_rate, aggregate_array_prob, memory×fsm interplay), one focused scenario each, the four saw_* facts + their proving metric (incl. the new num_operator_gates_with_duplicate_operands metric and the memory-vs-fsm mutual-exclusivity gotcha), and the dedicated --signoff-knob-sweep-gate; no source change; docs/workflow validation clean.`
  Result: `Refined 0006's inventory: width_parameterization_prob / aggregate_prob / memory_prob / fsm_prob already have dedicated default-set axes + gated facts; the genuinely unswept knobs are exactly four — mux_arm_duplication_rate (prove via existing num_muxes_degenerate), operand_duplication_rate (needs a new RTL-byte-identical metric num_operator_gates_with_duplicate_operands), aggregate_array_prob (the deferred AGGREGATE-ARRAY-PACKING.4b; prove via num_array_packed_aggregate_modules, needs uniform widths), and memory×fsm interplay (prove via num_memory_modules>0 && num_fsm_modules>0; needs memory_prob in (0,1) + fsm_prob=1.0 + calibrated seed because src/gen/module.rs:368-386 rolls memory before fsm and returns early). Four focused scenarios (int_operand_duplication, int_mux_arm_duplication, phase5b_array_packed_aggregate, memory_fsm_interplay) under a dedicated opt-in --signoff-knob-sweep-gate / ScenarioSet::SignoffKnobSweep (modeled on --phase2/3-gate), with the four saw_* facts required in compute_coverage_gaps for that set. Nothing retired; default-off / byte-identical. Full design in DEVELOPMENT_NOTES.md.`
  Verification: `scripts/check_memory_architecture.sh; knowledge-map regen/check; docs-only (no source change)`
  Commit: `SIGNOFF-AUTOMATION-EXPANSION.2a — first signoff knob-sweep batch design`

- ID: `SIGNOFF-AUTOMATION-EXPANSION.2b`
  Status: `pending`
  Goal: `Implement the .2a design: add the new num_operator_gates_with_duplicate_operands metric (src/metrics.rs), the four focused scenarios + four saw_* facts + the --signoff-knob-sweep-gate / ScenarioSet::SignoffKnobSweep + gap wiring (src/bin/tool_matrix.rs), new cargo-portable proofs, live-doc/book sync, and bank a clean repo-owned report.`
  Acceptance: `The four saw_* facts (saw_operand_duplication, saw_mux_arm_duplication, saw_array_packed_aggregate_design, saw_memory_fsm_interplay_design) land + are required by the new gate; a repo-owned tool_matrix --signoff-knob-sweep-gate report proves all four true with clean Verilator + both-Yosys results and coverage_gaps = []; snapshots 6/6 byte-identical; fmt/check/clippy/focused-tests clean; USER_GUIDE/ROADMAP/book/README synced for the new gate + knob coverage.`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `SIGNOFF-AUTOMATION-EXPANSION.2b` | `pending` | Implement the `.2a` design: four focused unswept-knob scenarios + four `saw_*` facts + the dedicated `--signoff-knob-sweep-gate` + the new operand-duplication metric, banked clean. |

## Decisions

- `2026-06-15`: Opened `proposed` as Lane 3 (execution order `2 → 3 →
  1`). The first leaf is a design/decision leaf because "broaden signoff
  automation" is open-ended; `.1` must pick one concrete, evidenced
  increment before any code edit.
- `2026-06-15` (`.1`): Promoted `proposed`→`active` (Lane 2
  `AGENT-MCP-EXPANSION` closed) and recorded decision
  [`0006`](../decisions/0006-signoff-automation-first-increment.md). The first
  increment is **richer adversarial knob-sweep coverage** — promote
  currently-unswept-but-existing generator knobs (operand/mux-arm duplication,
  `width_parameterization_prob`, `aggregate_prob`/`aggregate_array_prob`,
  memory×fsm interplay) into **explicit first-class `tool_matrix` scenario axes
  + `saw_*` coverage facts** (+ a focused gate), so they fire by construction
  rather than by chance — closing ROADMAP steering gap 3's hidden-bias hole.
  Chosen on the test "a day-one failure must be a real downstream signal, not
  noise": a newly-swept **legal** knob combo that trips Verilator/Yosys is the
  north-star signal; a new aggressive-synth/techmap column produces
  non-actionable noise (the `with-abc` softening precedent); formal/SVA has no
  spec to prove (structure-first non-goal); and the frontend stub-child corpora
  are not end-to-end synthesizable. The higher-ceiling paths (new tool columns;
  non-DUT lanes under the acceptance columns) are **preserved as future
  leaves** (nothing retired). Split the tree: added `.2` (implement the first
  knob-sweep batch).
- `2026-06-15` (`.2a`): Split `.2` into `.2a` (design, docs-only) + `.2b`
  (impl) per the `.3a`/`.3b` precedent. Resolved 0006's open question: the
  genuinely-unswept knobs are exactly four (`mux_arm_duplication_rate`,
  `operand_duplication_rate`, `aggregate_array_prob`, memory×fsm interplay) —
  the other 0006 candidates (`width_parameterization_prob`, `aggregate_prob`,
  `memory_prob`, `fsm_prob`) already have dedicated default-set axes + gated
  facts. One focused scenario per knob (not a combined-stress scenario) so each
  fact is provable from one realized metric; a dedicated opt-in
  `--signoff-knob-sweep-gate` (modeled on `--phase2/3-gate`) keeps the blast
  radius minimal and the bank self-contained. Full design (incl. the new
  `num_operator_gates_with_duplicate_operands` metric and the memory-vs-fsm
  mutual-exclusivity gotcha) in `DEVELOPMENT_NOTES.md`.

## Open Questions

- (`.1` resolved) The leading increment is the richer adversarial knob sweep
  (decision `0006`), not a new acceptance column or a new artifact family —
  those are preserved as future leaves.
- (`.2a` resolved) The first knob batch is the four genuinely-unswept knobs with
  one focused scenario each, the four `saw_*` facts named, and a dedicated
  `--signoff-knob-sweep-gate` (see `DEVELOPMENT_NOTES.md` `.2a` entry).
- (`.2b` open) Seed calibration for `memory_fsm_interplay` (which `memory_prob`
  in `(0,1)` + leaf count + seed deterministically realizes ≥1 memory leaf and
  ≥1 FSM leaf), and the exact arithmetic/mux gate-weight shaping for the two
  duplication scenarios — to be fixed empirically during impl.

## Blockers

- None. (Sequenced after Lane 2 by choice, not by dependency.)

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-06-15` | `SIGNOFF-AUTOMATION-EXPANSION.1` | `scripts/check_memory_architecture.sh` (incl. `0006` indexed); `knowledge-map/scripts/gen_knowledge_map.sh` regen + `knowledge-map/scripts/check_knowledge_map.sh`; docs/decision + task-tree edits; no source change (design/decision leaf) | `clean` |
| `2026-06-15` | `SIGNOFF-AUTOMATION-EXPANSION.2a` | `scripts/check_memory_architecture.sh`; `knowledge-map/scripts/gen_knowledge_map.sh` regen + `knowledge-map/scripts/check_knowledge_map.sh`; `DEVELOPMENT_NOTES.md` design entry + task-tree edits; no source change (design leaf) | `clean` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `SIGNOFF-AUTOMATION-EXPANSION.1` | `SIGNOFF-AUTOMATION-EXPANSION.1 — design/decision leaf + decision 0006` | Decision `0006`; first increment = richer knob-sweep coverage; `.2` added; frontier → `.2`. |
| `SIGNOFF-AUTOMATION-EXPANSION.2a` | `SIGNOFF-AUTOMATION-EXPANSION.2a — first signoff knob-sweep batch design` | `.2` split into `.2a`+`.2b`; four unswept knobs + four `saw_*` facts + dedicated `--signoff-knob-sweep-gate` fixed; frontier → `.2b`. |

## Changelog

- `2026-06-15`: Created task tree (Lane 3), opened `proposed`, via
  `CAPABILITY-LANE-OWNERSHIP.1`.
- `2026-06-15`: `.1` done — promoted `proposed`→`active` (Lane 2 closed);
  recorded decision `0006` (first increment = richer adversarial knob-sweep
  coverage: promote unswept knobs into explicit matrix axes + coverage facts,
  closing ROADMAP gap 3's hidden bias; new-tool-column and non-DUT-acceptance
  paths preserved as future leaves). Split the tree: added `.2` (implement the
  first knob-sweep batch). No source change; frontier advanced to `.2`.
- `2026-06-15`: `.2` split into `.2a` (design, docs-only) + `.2b` (impl) per
  the `.3a`/`.3b` precedent. `.2a` done — fixed the four genuinely-unswept
  knobs, one focused scenario each, the four `saw_*` facts + their proving
  metric (incl. the new `num_operator_gates_with_duplicate_operands` metric and
  the memory-vs-fsm mutual-exclusivity gotcha), and the dedicated
  `--signoff-knob-sweep-gate`; design recorded in `DEVELOPMENT_NOTES.md`. No
  source change; frontier advanced to `.2b`.
