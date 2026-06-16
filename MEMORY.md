# MEMORY - resume pointer (layer A; overwrite-only, keep <= 50 lines)

## How to resume
- Read `README.md`, then `MEMORY_ARCHITECTURE.md`.
- Work is task-tree tracked under `docs/tasks/`; index: `docs/TASK_TREE.md`.
- Durable cross-task facts live in `docs/decisions/`.
- Question-keyed retrieval facts are indexed in `KNOWLEDGE_MAP.md`.
- Commit completed leaves per `COMMIT.md`; include the leaf id in the subject.

## Current state (overwrite this block; do not append history)
- latest_commit: this `IDENTITY-DEEPENING.3b.2b.2b` commit (hash backfills next update; prior: `06bc2e1` = `.3b.2b.2a`; `314664c` = `.3b.2b.1`; `b873b40` = `SV-VERSION-TARGETING.3b.2b`; `e7fa265` = `SEMANTIC-INTROSPECTION-EXPANSION.2b.2`). Working tree clean after this commit. Push cadence: `origin/main` at `63d2622` → **6 commits ahead after this commit; push at ~30** (`feedback_push_cadence`).
- active_work_unit: **`IDENTITY-DEEPENING.3b.2b.2b` is `done` → the whole `.3b.2b` (cross-module whole-leaf-module sequential equivalence) sub-tree is CLOSED.** `.3`/`.3b`/`.3b.2`/`.3b.2b` all done. The `IDENTITY-DEEPENING` tree stays `active` with **no current frontier** — the deeper module-equivalence boundaries (memory / FSM / wrapper / retimed-state) are named, not-started, open-ended future leaves (none retired). Index: `docs/TASK_TREE.md`.
- `.3b.2b.2b` delivered (DOCS-ONLY, no behaviour change): book `factorization.md` §9b (whole-module sequential equivalence: combined-module materialization + §8b partition reuse + per-output-cone equality + coinduction + exclusions + worked delay-line example) + the "full factorization still means" list + the empirical-counters metric pair; `hierarchy.md` third module-identity layer; USER_GUIDE `hierarchy_sequential_module_dedup` config-knob bullet; ROADMAP gap 2 + capability-lanes "delivered" status; new KM card `docs/knowledge/sequential-module-dedup.md` (regenerated `KNOWLEDGE_MAP.md`, 32 facts). `mdbook build` clean; `cargo test --test book_examples` 3/3 (84.7s); KM + mem-arch self-checks clean. DUT byte-identical (no source change).
- next_action: continue PNT (no self-pause). `IDENTITY-DEEPENING` has no current frontier (deeper boundaries are open-ended future leaves). Pick the next active lane: **`SEMANTIC-INTROSPECTION-EXPANSION`** (`active`, no active frontier — `.3+` query kinds `input_reach`/`flop_reset_provenance`/`module_reachability` are open-ended; would need a design leaf to activate) and **`STRUCTURED-EMISSION-EXPANSION`** (`proposed` — needs activation + a `.1` design leaf: function/task, interface/modport, nested generate). Both are open-ended capability lanes with no pre-decided next slice; if no concrete eligible frontier exists, this may be a natural point to surface lane-direction choice to the owner. Resume from `docs/TASK_TREE.md`.
- lane invariants (all lanes): rules-first / no generate-then-filter (valid-by-construction); default-off / byte-identical where output could change; `tests/snapshots.rs` untouched by default; **no retirement** (`feedback_never_retire_strategies`); every capability is an opt-in knob; decision/KM fact per durable capability or boundary.
- in_flight_uncommitted: none after this commit. Repo handoff-ready: tree clean, self-checks green, resume pointer current.
- blockers: none. Validation policy: focused checks + `scripts/ram_guard.sh --threshold 90 -- <cmd>` for heavy builds (note the `--` separator); monitor RAM, stop >90% per `0003-resource-safe-validation`.

## Validation policy
- For workflow-doc memory/retrieval architecture leaves, use focused functional checks; full `cargo test` is not required per owner instruction.
- If a future full suite is run, monitor RAM; stop immediately above 90% RAM and record it as an environment/resource stop.
