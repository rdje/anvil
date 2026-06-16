# MEMORY - resume pointer (layer A; overwrite-only, keep <= 50 lines)

## How to resume
- Read `README.md`, then `MEMORY_ARCHITECTURE.md`.
- Work is task-tree tracked under `docs/tasks/`; index: `docs/TASK_TREE.md`.
- Durable cross-task facts live in `docs/decisions/`.
- Question-keyed retrieval facts are indexed in `KNOWLEDGE_MAP.md`.
- Commit completed leaves per `COMMIT.md`; include the leaf id in the subject.

## Current state (overwrite this block; do not append history)
- latest_commit: this `LIVE-DOC-CODEBASE-ALIGNMENT.2` commit (bootstrap-surfaced live-doc fix — `CODEBASE_ANALYSIS.md` integration-test count `6 → 8`, adding the `sv_version` + `sv_version_downstream` test files; **docs-only / no `src/` / DUT byte-identical**). Prior: `7163d72` = `STRUCTURED-EMISSION-EXPANSION.6b.3`; `1ea5a82` = `.6b.2b`; `5f6822b` = `.6b.2a`; `55761f9` = `.6b.1`; `7b6a500` = `.6a`; `b90bdff` = `.5` (**pushed** — `origin/main` is at `b90bdff`). Working tree clean after this commit. Push cadence: `origin/main` at `b90bdff` → **6 commits ahead (`.6a`,`.6b.1`,`.6b.2a`,`.6b.2b`,`.6b.3`,this `LIVE-DOC-CODEBASE-ALIGNMENT.2`); push at ~30** (`feedback_push_cadence`).
- active_work_unit: **`STRUCTURED-EMISSION-EXPANSION` tree `active`, NO current frontier** (open-ended lane). Three structured surfaces shipped: `function automatic` (`.1`+`.2`), `generate for` loop (`.3`+`.4`), `task automatic` (`.5`+`.6`). Next: open `.7` (decision `0015`) picking the FOURTH surface. `LIVE-DOC-CODEBASE-ALIGNMENT` re-closed. Index: `docs/TASK_TREE.md`.
- next_action: **PNT — open `STRUCTURED-EMISSION-EXPANSION.7`** (design leaf, docs-only): write decision `0015` picking the FOURTH structured surface = **the wider-lane `generate for` part-select** (a default-off broadening of the `generate for` surface to `{N{x}}` replications with a W>1 lane, rendered `assign <wire>[gi*W +: W] = <x>;`, closing the recorded wider-lane follow-up). **Empirically probed this session** (Verilator 5.046 -Wall + Yosys 0.64 both modes + Icarus 13.0): wider-lane part-select universally CLEAN + sim-proven `== {4{b}}`; `interface`/`modport` DISQUALIFIED (Icarus syntax-fail on the modport port + Yosys implicit-`.data`-decl warnings — confirms the recorded weak-support claim); nested-generate clean but bigger blast radius + harder by-construction source. Split `.7` (design) + `.8` (impl, pre-split `.8a` design-detail + `.8b`). Then PNT → `.8`. Doctrine: NO code change without a task-tree leaf owning it first.
- handoff: repo handoff-ready — `cargo check --all-targets` clean; tree clean. The `.7` design is fully scoped (decision + probe done); `.8` is a small/localized emitter change (relax the `generate_loop` predicate to W>=1 + part-select body; reuses `generate_loop_emit_prob` + `num_emitted_generate_loops`, so NO new knob / NO schema bump).
- lane invariants (all lanes): rules-first / no generate-then-filter (valid-by-construction); default-off / byte-identical where output could change; `tests/snapshots.rs` untouched by default; **no retirement** (`feedback_never_retire_strategies`); every capability is an opt-in knob; decision/KM fact per durable capability or boundary.
- in_flight_uncommitted: none after this commit. Repo handoff-ready: tree clean, self-checks green, resume pointer current.
- blockers: none. Validation policy: focused checks + `scripts/ram_guard.sh --threshold 90 -- <cmd>` for heavy builds (note the `--` separator); monitor RAM, stop >90% per `0003-resource-safe-validation`.

## Validation policy
- For workflow-doc memory/retrieval architecture leaves, use focused functional checks; full `cargo test` is not required per owner instruction.
- If a future full suite is run, monitor RAM; stop immediately above 90% RAM and record it as an environment/resource stop.
