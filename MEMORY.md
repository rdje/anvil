# MEMORY - resume pointer (layer A; overwrite-only, keep <= 50 lines)

## How to resume
- Read `README.md`, then `MEMORY_ARCHITECTURE.md`.
- Work is task-tree tracked under `docs/tasks/`; index: `docs/TASK_TREE.md`.
- Durable cross-task facts live in `docs/decisions/`.
- Question-keyed retrieval facts are indexed in `KNOWLEDGE_MAP.md`.
- Commit completed leaves per `COMMIT.md`; include the leaf id in the subject.

## Current state (overwrite this block; do not append history)
- latest_commit: this `SV-VERSION-TARGETING.3b.2b` commit (hash backfills next update; prior: `e7fa265` = `SEMANTIC-INTROSPECTION-EXPANSION.2b.2`; `cc2b5bc` = `.2b.1`; `63d2622` = `.2a`). Working tree clean after this commit. Push cadence: `origin/main` at `63d2622` → **3 commits ahead after this commit; push at ~30** (`feedback_push_cadence`).
- active_work_unit: **`SV-VERSION-TARGETING` is `done` (CLOSED `2026-06-16`)** — `.3b.2b` landed: the repo-owned matrix up-opt gate. The whole tree's scope (down-gating + up-opting + per-version downstream acceptance axis) is delivered; the first version-distinctive up-opt (IEEE 1800-2023 `union soft`) ships as both a generator capability and a matrix gate. Further up-opts are open-ended post-tree breadth (nothing retired). Index: `docs/TASK_TREE.md`.
- `.3b.2b` delivered (CODE+DOCS, all in `src/bin/tool_matrix.rs`; DUT byte-identical): a tenth `--sv-version-gate` scenario `sv2023_soft_union_upopt` (`soft_union_upopt_config`: slice-heavy, `soft_union_slice_prob=1.0`, 2023) emits the real `union soft` overlay; `scenario_emits_soft_union_overlay`→`verilator_only` runs it Verilator-only (Yosys/Icarus recorded no-op); `ModuleReport.emitted_soft_union_overlay` (from emitted SV text) lights the new `saw_sv_version_2023_soft_union_upopt` fact (never Yosys), enforced by `compute_coverage_gaps`; `!yosys.is_empty()` honesty guard added to the general per-version fact. Banked clean `/tmp/anvil-sv-version-gate-upopt-r1` (10 scenarios / 20 units / `coverage_gaps=[]` / Verilator 20/0 / Yosys 18/0). `cargo test --bin tool_matrix` 53/0; `--lib` 427/0/2; snapshots 6/6; clippy/fmt clean. Rejected a `Metrics` counter (would force schema 1.3→1.4) for the matrix-local `ModuleReport` bool.
- next_action: continue PNT (no self-pause). `SV-VERSION-TARGETING` closed; the un-parked frontier is **`IDENTITY-DEEPENING.3b.2b`** (`active`) — cross-module whole-module sequential equivalence, **fully designed** (decision `0008` + `.3b.1` detail + `.3b.2a` `bisimulation_partition` helper landed `edae1b0`); it was parked only pending `SV-VERSION-TARGETING`, now done. Resume from `docs/tasks/IDENTITY-DEEPENING.md` `.3b.2b` (it is a large leaf — split before implementing: the cross-module merge pass beside `dedup_semantic_modules` in `src/ir/dedup.rs` + knob + metric + gate; default-off / byte-identical). Sibling lanes still open: `STRUCTURED-EMISSION-EXPANSION` (`proposed`, design leaf `.1` when activated); `SEMANTIC-INTROSPECTION-EXPANSION` (`active`, no active frontier — `.3+` query kinds open-ended).
- lane invariants (all lanes): rules-first / no generate-then-filter (valid-by-construction); default-off / byte-identical where output could change; `tests/snapshots.rs` untouched by default; **no retirement** (`feedback_never_retire_strategies`); every capability is an opt-in knob; decision/KM fact per durable capability or boundary.
- in_flight_uncommitted: none after this commit. Repo handoff-ready: tree clean, self-checks green, resume pointer current.
- blockers: none. Validation policy: focused checks + `scripts/ram_guard.sh --threshold 90 -- <cmd>` for heavy builds (note the `--` separator); monitor RAM, stop >90% per `0003-resource-safe-validation`.

## Validation policy
- For workflow-doc memory/retrieval architecture leaves, use focused functional checks; full `cargo test` is not required per owner instruction.
- If a future full suite is run, monitor RAM; stop immediately above 90% RAM and record it as an environment/resource stop.
