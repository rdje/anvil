# MEMORY - resume pointer (layer A; overwrite-only, keep <= 50 lines)

## How to resume
- Read `README.md`, then `MEMORY_ARCHITECTURE.md`.
- Work is task-tree tracked under `docs/tasks/`; index: `docs/TASK_TREE.md`.
- Durable cross-task facts live in `docs/decisions/`.
- Question-keyed retrieval facts are indexed in `KNOWLEDGE_MAP.md`.
- Commit completed leaves per `COMMIT.md`; include the leaf id in the subject.

## Current state (overwrite this block; do not append history)
- latest_commit: this `SIGNOFF-AUTOMATION-EXPANSION.1` commit (hash backfills next update; prior: `96d1203` = `AGENT-MCP-EXPANSION.5`; `ce395de` = `.4b`; `6101c15` = `.4a`). Working tree clean after this commit.
- active_work_unit: **`SIGNOFF-AUTOMATION-EXPANSION`** (Lane 3, `active`). `AGENT-MCP-EXPANSION` (Lane 2) CLOSED. `.1` (design, decision `0006`) **done**. Frontier now **`.2`** (implement the first richer-knob-sweep batch, `pending`). Lane order **`2 → 3 → 1`**: then **`IDENTITY-DEEPENING`** (Lane 1, `proposed`). Index: `docs/TASK_TREE.md`.
- next_action: do **`SIGNOFF-AUTOMATION-EXPANSION.2`** (CODE — task-tree owned) per decision `0006`. In `src/bin/tool_matrix.rs`, promote the highest-bias currently-unswept generator knobs into **explicit first-class scenario axes + `saw_*` coverage facts** (lead candidates: operand/mux-arm duplication rates, `width_parameterization_prob`, `aggregate_prob`/`aggregate_array_prob`, memory×fsm interplay) + a focused gate; default-off / byte-identical where a knob changes RTL; bank a clean repo-owned report (new `saw_*` true; clean Verilator + both Yosys). First STUDY `src/bin/tool_matrix.rs` scenario construction (`CoverageSummary` ~`:286`, `compute_coverage_gaps` ~`:6552`, the built-in scenario set fns) + `src/config.rs` for the exact knob names/defaults. Validate fmt/check/clippy + focused tool_matrix/lib tests + `cargo test --test snapshots` (byte-identical). Consider splitting `.2` into `.2a` design (exact knob batch + scenario shapes + fact names) + `.2b` impl if it proves broad (the `.3a`/`.3b` precedent). Continue PNT through this tree, then `IDENTITY-DEEPENING` (no self-pause — `feedback_no_self_pause_until_trees_closed`).
- lane invariants: warning-as-failure stays (counterexamples retained with exact seed+knobs); rules-first / no-generate-then-filter; no whole-module spec/oracle; **no retirement** of any gate/scenario/axis; default-off / byte-identical where a knob could change RTL; single downstream source of truth stays `tool_matrix`/`downstream` (add scenarios/facts, not a second path). MCP invariants (0004/0005) still hold for the closed Lane 2: thin read-mostly adapter, SCHEMA-DERIVED, hardened `downstream`-only controlled tools, `--artifact dut` byte-identical, loopback-default `--http`, no new Cargo dep.
- in_flight_uncommitted: none after this commit. Repo is handoff-ready: tree clean, gates green, resume pointer current.
- blockers: none. Validation policy: focused checks + `scripts/ram_guard.sh --threshold 88` for heavy builds; full `cargo test` not required per owner resource policy (monitor RAM; stop above 90%, >95% reboots). For code leaves run fmt/check/clippy + focused tests + snapshot byte-identical guard. Push cadence: `origin/main` is at `381ec01` (`.5.3`); **12 commits ahead after this commit (…`.4a`,`.4b`,`.5`,`SIGNOFF-AUTOMATION-EXPANSION.1`) — under the ~30 threshold, no push yet** (`feedback_push_cadence`).

## Validation policy
- For workflow-doc memory/retrieval architecture leaves, use focused functional checks; full `cargo test` is not required per owner instruction.
- If a future full suite is run, monitor RAM; stop immediately above 90% RAM and record it as an environment/resource stop.
