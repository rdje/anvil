# SIGNOFF-AUTOMATION-EXPANSION: Broaden Downstream Signoff Automation

## Metadata

- Tree ID: `SIGNOFF-AUTOMATION-EXPANSION`
- Status: `active`
- Roadmap lane: `Quality — downstream signoff automation breadth`
- Created: `2026-06-15`
- Last updated: `2026-06-15` (promoted `proposed`→`active` after `AGENT-MCP-EXPANSION` closed; `.1` done — decision `0006`; frontier → `.2`)
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
  Status: `pending`
  Goal: `Implement the first richer-knob-sweep batch (per decision 0006): add explicit tool_matrix scenario(s) that force the highest-bias unswept knobs (lead candidates: operand/mux-arm duplication rates, width_parameterization_prob, aggregate_prob/aggregate_array_prob, memory×fsm interplay), plus the matching saw_* coverage facts and a focused gate; default-off / byte-identical where a knob changes RTL; banked clean across Verilator + both Yosys modes.`
  Acceptance: `New explicit scenario axis/axes + coverage facts land in src/bin/tool_matrix.rs; a repo-owned banked report proves the new saw_* facts true with clean Verilator + both-Yosys downstream results (coverage_gaps unaffected or extended intentionally); snapshots 6/6 byte-identical (no DUT generator-core change); fmt/check/clippy/focused-tests clean. Split into design + impl if it proves broad.`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `SIGNOFF-AUTOMATION-EXPANSION.2` | `pending` | Implement the first richer-knob-sweep batch (decision `0006`): promote the highest-bias unswept knobs into explicit matrix axes + coverage facts, banked clean. |

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

## Open Questions

- (`.1` resolved) The leading increment is the richer adversarial knob sweep
  (decision `0006`), not a new acceptance column or a new artifact family —
  those are preserved as future leaves.
- (`.2` open) The exact first knob batch and scenario shapes (one focused
  scenario per knob vs a small combined-stress scenario), and which `saw_*`
  facts + gate assertions to add.

## Blockers

- None. (Sequenced after Lane 2 by choice, not by dependency.)

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-06-15` | `SIGNOFF-AUTOMATION-EXPANSION.1` | `scripts/check_memory_architecture.sh` (incl. `0006` indexed); `knowledge-map/scripts/gen_knowledge_map.sh` regen + `knowledge-map/scripts/check_knowledge_map.sh`; docs/decision + task-tree edits; no source change (design/decision leaf) | `clean` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `SIGNOFF-AUTOMATION-EXPANSION.1` | `SIGNOFF-AUTOMATION-EXPANSION.1 — design/decision leaf + decision 0006` | Decision `0006`; first increment = richer knob-sweep coverage; `.2` added; frontier → `.2`. |

## Changelog

- `2026-06-15`: Created task tree (Lane 3), opened `proposed`, via
  `CAPABILITY-LANE-OWNERSHIP.1`.
- `2026-06-15`: `.1` done — promoted `proposed`→`active` (Lane 2 closed);
  recorded decision `0006` (first increment = richer adversarial knob-sweep
  coverage: promote unswept knobs into explicit matrix axes + coverage facts,
  closing ROADMAP gap 3's hidden bias; new-tool-column and non-DUT-acceptance
  paths preserved as future leaves). Split the tree: added `.2` (implement the
  first knob-sweep batch). No source change; frontier advanced to `.2`.
