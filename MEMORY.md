# MEMORY - resume pointer (layer A; overwrite-only, keep <= 50 lines)

## How to resume
- Read `README.md`, then `MEMORY_ARCHITECTURE.md`.
- Work is task-tree tracked under `docs/tasks/`; index: `docs/TASK_TREE.md`.
- Durable cross-task facts live in `docs/decisions/`.
- Question-keyed retrieval facts are indexed in `KNOWLEDGE_MAP.md`.
- Commit completed leaves per `COMMIT.md`; include the leaf id in the subject.

## Current state (overwrite this block; do not append history)
- latest_commit: this `SV-VERSION-TARGETING.1` commit (hash backfills next update; prior: `edae1b0` = `IDENTITY-DEEPENING.3b.2a`; `762cf46` = `.3b.1`). Working tree clean after this commit.
- active_work_unit: **`SV-VERSION-TARGETING`** (owner-directed capability lane, **`active`**, opened `2026-06-15`). `.1` (decision `0009`) **done**. Frontier leaf: **`.2`** (`proposed`, plumbing impl). Two sibling lanes registered `proposed`: `STRUCTURED-EMISSION-EXPANSION`, `SEMANTIC-INTROSPECTION-EXPANSION`. Index: `docs/TASK_TREE.md`.
- next_action: **`SV-VERSION-TARGETING.2`** — implement decision `0009`'s first increment: `Config::sv_version` enum (`2012|2017|2023`) + `--sv-version` CLI + `--dump-config`/introspection field (schema MINOR bump); thread the target into `src/emit/sv.rs` as a read-only capability bound (**down-gating** guarantee); add the per-version downstream acceptance axis (`src/downstream/mod.rs` + `tool_matrix`: verilator `--language 1800-20xx`, yosys `-sv`, iverilog `-g2012` gated/no-op beyond its newest gen). Default value = the floor byte-identical to today (snapshots 6/6 untouched). Rules-first / valid-by-construction (no generate-then-filter). Book/USER_GUIDE/README/ROADMAP/knobs + KM. Split into `.2a` design-detail + `.2b` impl if broad. Continue PNT (no self-pause — `feedback_no_self_pause_until_trees_closed`).
- `.1` delivered (docs-only, no source change): decision `0009` — opt-in `--sv-version <2012|2017|2023>` gate; down-gating (never emit newer-than-target → standard-validity guarantee) + up-opting (emit a higher standard's distinctive synthesizable constructs, each proven downstream-clean in the matching tool mode); default byte-identical; per-version downstream acceptance axis. Grounded in `src/emit/sv.rs` (current 2012/2017 floor subset) + `src/downstream/mod.rs` (fixed tool standards; no existing knob). Opened the lane + registered the 2 sibling proposed lanes.
- **PARKED (not abandoned): `IDENTITY-DEEPENING`** (`active`, frontier `.3b.2b`) — cross-module whole-module sequential equivalence, **fully designed** (decision `0008` + `.3b.1` detail + `.3b.2a` `bisimulation_partition` helper landed `edae1b0`). Resume from `docs/tasks/IDENTITY-DEEPENING.md` `.3b.2b` (combined-module `modules_bisimilar` + `dedup_sequential_modules` + `hierarchy_sequential_module_dedup` knob + metric + gate). Owner redirected priority to `SV-VERSION-TARGETING`.
- lane invariants (both lanes): rules-first / no generate-then-filter (valid-by-construction); default-off / byte-identical where output could change; `tests/snapshots.rs` untouched by default; **no retirement** (`feedback_never_retire_strategies`); every capability is an opt-in knob; decision/KM fact per durable capability or boundary.
- in_flight_uncommitted: none after this commit. Repo handoff-ready: tree clean, self-checks green, resume pointer current.
- blockers: none. Validation policy: focused checks + `scripts/ram_guard.sh --threshold 88` for heavy builds; for the `.1` docs/design leaf only `check_memory_architecture.sh` + `check_knowledge_map.sh` are load-bearing (no source change ⇒ DUT byte-identical) per `0003-resource-safe-validation`; monitor RAM, stop >90%. Push cadence: `origin/main` at `381ec01`; **21 commits ahead after this commit — under the ~30 threshold, no push yet** (`feedback_push_cadence`).

## Validation policy
- For workflow-doc memory/retrieval architecture leaves, use focused functional checks; full `cargo test` is not required per owner instruction.
- If a future full suite is run, monitor RAM; stop immediately above 90% RAM and record it as an environment/resource stop.
