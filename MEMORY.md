# MEMORY - resume pointer (layer A; overwrite-only, keep <= 50 lines)

## How to resume
- Read `README.md`, then `MEMORY_ARCHITECTURE.md`.
- Work is task-tree tracked under `docs/tasks/`; index: `docs/TASK_TREE.md`.
- Durable cross-task facts live in `docs/decisions/`.
- Question-keyed retrieval facts are indexed in `KNOWLEDGE_MAP.md`.
- Commit completed leaves per `COMMIT.md`; include the leaf id in the subject.

## Current state (overwrite this block; do not append history)
- latest_commit: this `SIGNOFF-AUTOMATION-EXPANSION.2b` commit (hash backfills next update; prior: `92a04c8` = `.2a`; `edf79d6` = `.1`; `96d1203` = `AGENT-MCP-EXPANSION.5`). Working tree clean after this commit.
- active_work_unit: **`SIGNOFF-AUTOMATION-EXPANSION`** (Lane 3, `active`). `AGENT-MCP-EXPANSION` (Lane 2) CLOSED. `.1` (decision `0006`), `.2a` (design), `.2b` (impl) all **done** → first richer-knob-sweep increment delivered; **`.2` container done**. **No active frontier on this tree** (higher-ceiling future leaves preserved per decision `0006`: a new acceptance column; non-DUT lanes under acceptance; remaining unswept knobs/axes — each a future design+impl pair). Per lane order **`2 → 3 → 1`**, next lane = **`IDENTITY-DEEPENING`** (Lane 1, `proposed` → promote to `active`; `.1` design leaf pending). Index: `docs/TASK_TREE.md`.
- next_action: **promote `IDENTITY-DEEPENING` (Lane 1) to `active` and do its `.1` design leaf** (read `docs/tasks/IDENTITY-DEEPENING.md`; it's the NodeId-identity / full-factorization deepening lane per ROADMAP steering gap 2 — broader sequential equivalence, memory-state merging beyond the instance-local boundary, hierarchy equivalence beyond canonical signatures). `.1` is a design/decision leaf (pick one concrete deepening increment with evidence before any code edit), like the other lanes' `.1`. Continue PNT (no self-pause — `feedback_no_self_pause_until_trees_closed`). Optionally first scope a future `SIGNOFF-AUTOMATION-EXPANSION` leaf instead if the owner prefers more signoff breadth, but the lane order points to `IDENTITY-DEEPENING` next.
- knob-sweep gate (`.2b`, delivered): `tool_matrix --signoff-knob-sweep-gate` (`ScenarioSet::SignoffKnobSweep`, 12 scenarios, 4 facts `saw_operand_duplication`/`saw_mux_arm_duplication`/`saw_array_packed_aggregate_design`/`saw_memory_fsm_interplay_design`); new metric `num_operator_gates_with_duplicate_operands` (`src/metrics.rs`, RTL byte-identical). Banked clean: `/tmp/anvil-signoff-knob-sweep-r1` (48/0 Verilator + both Yosys, `coverage_gaps=[]`).
- lane invariants: warning-as-failure stays (counterexamples retained with exact seed+knobs); rules-first / no-generate-then-filter; no whole-module spec/oracle; **no retirement** of any gate/scenario/axis; default-off / byte-identical where a knob could change RTL; single downstream source of truth stays `tool_matrix`/`downstream` (add scenarios/facts, not a second path). MCP invariants (0004/0005) still hold for the closed Lane 2.
- in_flight_uncommitted: none after this commit. Repo is handoff-ready: tree clean, gates green, resume pointer current.
- blockers: none. Validation policy: focused checks + `scripts/ram_guard.sh --threshold 88` for heavy builds; full `cargo test` not required per owner resource policy (monitor RAM; stop above 90%, >95% reboots). For code leaves run fmt/check/clippy + focused tests + snapshot byte-identical guard. Push cadence: `origin/main` is at `381ec01` (`.5.3`); **14 commits ahead after this commit (…`.1`,`.2a`,`.2b`) — under the ~30 threshold, no push yet** (`feedback_push_cadence`).

## Validation policy
- For workflow-doc memory/retrieval architecture leaves, use focused functional checks; full `cargo test` is not required per owner instruction.
- If a future full suite is run, monitor RAM; stop immediately above 90% RAM and record it as an environment/resource stop.
