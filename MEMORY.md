# MEMORY - resume pointer (layer A; overwrite-only, keep <= 50 lines)

## How to resume
- Read `README.md`, then `MEMORY_ARCHITECTURE.md`.
- Work is task-tree tracked under `docs/tasks/`; index: `docs/TASK_TREE.md`.
- Durable cross-task facts live in `docs/decisions/`.
- Question-keyed retrieval facts are indexed in `KNOWLEDGE_MAP.md`.
- Commit completed leaves per `COMMIT.md`; include the leaf id in the subject.

## Current state (overwrite this block; do not append history)
- latest_commit: this `SV-VERSION-TARGETING.2a` commit (hash backfills next update; prior: `eab3f4f` = `SV-VERSION-TARGETING.1`; `edae1b0` = `IDENTITY-DEEPENING.3b.2a`). Working tree clean after this commit.
- active_work_unit: **`SV-VERSION-TARGETING`** (owner-directed capability lane, **`active`**, opened `2026-06-15`). `.1` (decision `0009`) + `.2a` (impl design detail) **done**. Frontier leaf: **`.2b.1`** (`proposed`). Two sibling lanes registered `proposed`: `STRUCTURED-EMISSION-EXPANSION`, `SEMANTIC-INTROSPECTION-EXPANSION`. Index: `docs/TASK_TREE.md`.
- next_action: **`SV-VERSION-TARGETING.2b.1`** — knob plumbing + emitter capability bound (per `.2a` design in `DEVELOPMENT_NOTES.md`): add `SvVersion {Sv2012<Sv2017<Sv2023}` enum + `Config::sv_version` (`#[serde(default)]` = `Sv2012`) + `--sv-version` CLI/Overrides/apply/validate in `src/config.rs`; new `to_sv_versioned`/versioned design entry points in `src/emit/sv.rs` (old ones delegate with `SvVersion::default()` ⇒ byte-identical) threading `SvVersion::permits()` (gates nothing yet, subset ≤2012); `src/main.rs` DUT path + umbrella DUT lane pass `cfg.sv_version`; introspection schema MINOR bump `1.1→1.2` (`src/introspect/mod.rs:43` + `docs/AGENT_INTROSPECTION_SCHEMA.md` + 5 `"1.1"` assertions: 2 introspect, 3 mcp); a cross-version byte-identity test; book(knobs)/USER_GUIDE/README/knobs + KM. **`tests/snapshots.rs` 6/6 untouched.** Then `.2b.2` = per-version downstream acceptance axis. Continue PNT (no self-pause — `feedback_no_self_pause_until_trees_closed`).
- `.2a` delivered (docs-only): resolved decision `0009`'s 5 open questions, split `.2`→`.2a`/`.2b`, pre-split `.2b`→`.2b.1`/`.2b.2`. Key calls: default `Sv2012` (honest floor, byte-identical, down-gating-to-floor = no-op); bound threads to emitter as a **parameter** (not the IR); introspection serde-automatic.
- **PARKED (not abandoned): `IDENTITY-DEEPENING`** (`active`, frontier `.3b.2b`) — cross-module whole-module sequential equivalence, **fully designed** (decision `0008` + `.3b.1` detail + `.3b.2a` `bisimulation_partition` helper landed `edae1b0`). Resume from `docs/tasks/IDENTITY-DEEPENING.md` `.3b.2b` (combined-module `modules_bisimilar` + `dedup_sequential_modules` + `hierarchy_sequential_module_dedup` knob + metric + gate). Owner redirected priority to `SV-VERSION-TARGETING`.
- lane invariants (both lanes): rules-first / no generate-then-filter (valid-by-construction); default-off / byte-identical where output could change; `tests/snapshots.rs` untouched by default; **no retirement** (`feedback_never_retire_strategies`); every capability is an opt-in knob; decision/KM fact per durable capability or boundary.
- in_flight_uncommitted: none after this commit. Repo handoff-ready: tree clean, self-checks green, resume pointer current.
- blockers: none. Validation policy: focused checks + `scripts/ram_guard.sh --threshold 90 -- <cmd>` for heavy builds (note the `--` separator); for the `.2a` docs/design leaf only `check_memory_architecture.sh` + `check_knowledge_map.sh` are load-bearing (no source change ⇒ DUT byte-identical) per `0003-resource-safe-validation`; monitor RAM, stop >90%. The `.2b.1`/`.2b.2` code leaves need full `cargo test` + clippy + fmt + snapshots 6/6. Push cadence: `origin/main` at `381ec01`; **22 commits ahead after this commit — under the ~30 threshold, no push yet** (`feedback_push_cadence`).

## Validation policy
- For workflow-doc memory/retrieval architecture leaves, use focused functional checks; full `cargo test` is not required per owner instruction.
- If a future full suite is run, monitor RAM; stop immediately above 90% RAM and record it as an environment/resource stop.
