# MEMORY - resume pointer (layer A; overwrite-only, keep <= 50 lines)

## How to resume
- Read `README.md`, then `MEMORY_ARCHITECTURE.md`.
- Work is task-tree tracked under `docs/tasks/`; index: `docs/TASK_TREE.md`.
- Durable cross-task facts live in `docs/decisions/`.
- Question-keyed retrieval facts are indexed in `KNOWLEDGE_MAP.md`.
- Commit completed leaves per `COMMIT.md`; include the leaf id in the subject.

## Current state (overwrite this block; do not append history)
- latest_commit: this `STRUCTURED-EMISSION-EXPANSION.6b.3` commit (the user-facing closeout — the THIRD structured surface, the combinational `task automatic`, is delivered end-to-end; `.6b.3`/`.6b`/`.6` all close; **docs-only / DUT byte-identical**). Prior: `1ea5a82` = `.6b.2b` (the `tool_matrix --task-emit-gate` gate); `5f6822b` = `.6b.2a` (metric + schema `1.10`); `55761f9` = `.6b.1` (live surface); `7b6a500` = `.6a`; `b90bdff` = `.5` (**pushed** — `origin/main` is at `b90bdff`). Working tree clean after this commit. Push cadence: `origin/main` at `b90bdff` → **5 commits ahead after this commit (`.6a` `7b6a500`, `.6b.1` `55761f9`, `.6b.2a` `5f6822b`, `.6b.2b` `1ea5a82`, this `.6b.3`); push at ~30** (`feedback_push_cadence`).
- active_work_unit: **`STRUCTURED-EMISSION-EXPANSION` tree `active`, NO current frontier** (open-ended lane). The THIRD structured surface (combinational `task automatic`, decision `0014`) is delivered end-to-end (`.5` design + `.6a` design-detail + `.6b.1` live + `.6b.2a` metric/schema-`1.10` + `.6b.2b` gate + `.6b.3` docs). All three structured surfaces shipped: `function automatic` (`.1`+`.2`), `generate for` loop (`.3`+`.4`), `task automatic` (`.5`+`.6`). Future surfaces (nested/multi-level `generate`, `interface`/`modport`, richer tasks) are `.7+`, each its own decision. Index: `docs/TASK_TREE.md`.
- `.6b.3` delivered (docs-only): `book/src/structured-emission.md` `## The third surface: a combinational task automatic` (byte-verified seed-1 before/after; the function-surface candidate parallel; the output-var passthrough form; the four-way mutual exclusion; metric @ schema `1.10` + gate) + intro update + the `task_emit_prob` knob in `book/src/knobs.md`/`USER_GUIDE.md`/README + the KM card `combinational-task-emit` (KM 40→41 facts / 318→331 keys). `mdbook build` + `check_knowledge_map` + `check_memory_architecture` + `book_examples` 3/3 green.
- next_action: **PNT — pick-and-roll at the no-frontier boundary** (`feedback_pick_and_roll_at_no_frontier`: all active trees — `STRUCTURED-EMISSION-EXPANSION`, `SIGNOFF-AUTOMATION-EXPANSION`, `IDENTITY-DEEPENING`, `SEMANTIC-INTROSPECTION-EXPANSION`, `LOCAL-REFERENCE-CACHE` — are open-ended; self-select, do not steer-ask). Natural continuation: **open `STRUCTURED-EMISSION-EXPANSION.7`** — a design/decision leaf picking the FOURTH structured surface (the recorded leading candidates from decision `0014` are nested/multi-level `generate` and `interface`/`modport`; re-confirm with a fresh empirical tool-acceptance probe, write decision `0015`, and split `.7` (design) + `.8` (impl) — no source change in `.7`). Alternatively pick another open lane. Doctrine: NO code change without a task-tree leaf owning it first.
- handoff: repo is handoff-ready — the third structured surface is a complete, self-contained deliverable (4 source/doc commits, all self-checks green). A fresh session is reasonable before opening `.7` (a new sub-objective).
- lane invariants (all lanes): rules-first / no generate-then-filter (valid-by-construction); default-off / byte-identical where output could change; `tests/snapshots.rs` untouched by default; **no retirement** (`feedback_never_retire_strategies`); every capability is an opt-in knob; decision/KM fact per durable capability or boundary.
- in_flight_uncommitted: none after this commit. Repo handoff-ready: tree clean, self-checks green, resume pointer current.
- blockers: none. Validation policy: focused checks + `scripts/ram_guard.sh --threshold 90 -- <cmd>` for heavy builds (note the `--` separator); monitor RAM, stop >90% per `0003-resource-safe-validation`.

## Validation policy
- For workflow-doc memory/retrieval architecture leaves, use focused functional checks; full `cargo test` is not required per owner instruction.
- If a future full suite is run, monitor RAM; stop immediately above 90% RAM and record it as an environment/resource stop.
