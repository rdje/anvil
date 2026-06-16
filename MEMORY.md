# MEMORY - resume pointer (layer A; overwrite-only, keep <= 50 lines)

## How to resume
- Read `README.md`, then `MEMORY_ARCHITECTURE.md`.
- Work is task-tree tracked under `docs/tasks/`; index: `docs/TASK_TREE.md`.
- Durable cross-task facts live in `docs/decisions/`.
- Question-keyed retrieval facts are indexed in `KNOWLEDGE_MAP.md`.
- Commit completed leaves per `COMMIT.md`; include the leaf id in the subject.

## Current state (overwrite this block; do not append history)
- latest_commit: this `STRUCTURED-EMISSION-EXPANSION.8b` commit (impl — the FOURTH structured surface, the wider-lane `generate for` part-select, delivered end-to-end; `.8b`/`.8` close; **two surgical source edits + 4 lib proofs + book/docs; default-off / DUT byte-identical; no new knob / no schema bump**). Prior: `ab73083` = `.8a`; `3ec4f66` = `.7` (decision `0015`); `cf25642` = `LIVE-DOC-CODEBASE-ALIGNMENT.2`; `7163d72` = `.6b.3`; … `b90bdff` = `.5` (**pushed** — `origin/main` is at `b90bdff`). Working tree clean after this commit. Push cadence: `origin/main` at `b90bdff` → **9 commits ahead (`.6a`,`.6b.1`,`.6b.2a`,`.6b.2b`,`.6b.3`,`LIVE-DOC-CODEBASE-ALIGNMENT.2`,`.7`,`.8a`,this `.8b`); push at ~30** (`feedback_push_cadence`).
- active_work_unit: **`STRUCTURED-EMISSION-EXPANSION` tree `active`, NO current frontier** (open-ended lane). **FOUR** structured surfaces now delivered end-to-end: `function automatic` (`.1`+`.2`), `generate for` loop (`.3`+`.4`), `task automatic` (`.5`+`.6`), and the **wider-lane `generate for` part-select** (`.7`+`.8`, decision `0015`, closed `2026-06-17`). Future surfaces (nested/multi-level `generate`, `interface`/`modport`, richer tasks) are `.9+`, each its own decision. Index: `docs/TASK_TREE.md`.
- next_action: **PNT — pick-and-roll at the no-frontier boundary** (`feedback_pick_and_roll_at_no_frontier`: all active trees — `STRUCTURED-EMISSION-EXPANSION`, `SIGNOFF-AUTOMATION-EXPANSION`, `IDENTITY-DEEPENING`, `SEMANTIC-INTROSPECTION-EXPANSION`, `LOCAL-REFERENCE-CACHE` — are open-ended; self-select, do not steer-ask). Natural continuations: open `STRUCTURED-EMISSION-EXPANSION.9` (a design/decision leaf picking the FIFTH structured surface — nested/multi-level `generate` is the leading candidate; `interface`/`modport` is empirically disqualified per decision `0015`'s probe; re-confirm with a fresh tool probe + write decision `0016`), OR pick another open lane. Doctrine: NO code change without a task-tree leaf owning it first. **Consider a push soon** — 9 commits ahead (cadence ~30).
- handoff: repo handoff-ready — `cargo check --all-targets` clean; `cargo test --lib` 493 + snapshots 6/6; tree clean; all self-checks green. The fourth structured surface is a complete, self-contained deliverable (`.7`/`.8a`/`.8b`). A fresh session is reasonable before opening `.9` (a new sub-objective).
- lane invariants (all lanes): rules-first / no generate-then-filter (valid-by-construction); default-off / byte-identical where output could change; `tests/snapshots.rs` untouched by default; **no retirement** (`feedback_never_retire_strategies`); every capability is an opt-in knob; decision/KM fact per durable capability or boundary.
- in_flight_uncommitted: none after this commit. Repo handoff-ready: tree clean, self-checks green, resume pointer current.
- blockers: none. Validation policy: focused checks + `scripts/ram_guard.sh --threshold 90 -- <cmd>` for heavy builds (note the `--` separator); monitor RAM, stop >90% per `0003-resource-safe-validation`.

## Validation policy
- For workflow-doc memory/retrieval architecture leaves, use focused functional checks; full `cargo test` is not required per owner instruction.
- If a future full suite is run, monitor RAM; stop immediately above 90% RAM and record it as an environment/resource stop.
