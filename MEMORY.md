# MEMORY - resume pointer (layer A; overwrite-only, keep <= 50 lines)

## How to resume
- Read `README.md`, then `MEMORY_ARCHITECTURE.md`.
- Work is task-tree tracked under `docs/tasks/`; index: `docs/TASK_TREE.md`.
- Durable cross-task facts live in `docs/decisions/`.
- Question-keyed retrieval facts are indexed in `KNOWLEDGE_MAP.md`.
- Commit completed leaves per `COMMIT.md`; include the leaf id in the subject.

## Current state (overwrite this block; do not append history)
- latest_commit: this `KNOB-ERGONOMICS-AND-PRESETS.2b.3` commit — **user-facing docs** for the shipped `.2b.1` CLI-flag promotion + `--profile` presets (decision `0021`). **Docs-only — README + USER_GUIDE + `book/src/{knobs,structured-emission,sequential}.md` + new KM card `knob-presets-and-cli-flags` + KM regen (54→55); no `src/` change, DUT byte-identical.** Closes the `.2b.1` **book-sync gap**: every now-false "no CLI flag"/"config-file-only" claim for the 16 promoted knobs corrected; a `--profile` presets section + the resolution order documented. mdbook build clean; KM gen/check + mem-arch green. Documents ONLY shipped behaviour (not the unbuilt `.2b.2` catalog). Prior: `c59bf39`=`.2b.1` (16 CLI flags + `--profile` + `resolve_config`, full `cargo test` green); `4d1b8c4`=`.2a`; `e68e2d1`=`.1`. Push cadence: `origin/main` at `7142fd7` → **18 commits ahead** after this; push at ~30 (`feedback_push_cadence`).
- active_work_unit: **`KNOB-ERGONOMICS-AND-PRESETS` — `.1`+`.2a`+`.2b.1`+`.2b.3` DONE; frontier = `.2b.2`** (the SCHEMA-DERIVED queryable knob catalog: `knob_catalog()` + `KnobInfo` + metadata table + completeness test [`catalog names == serde_json::to_value(Config::default()) keys`; group via prefix-classifier, cli_flag/config_only derived from the `Overrides` serde key set ∪ {seed}] mirroring `downstream::adapter_catalog()`; `Serialize`(skip-None) on `Overrides`; the new `anvil://catalog/knob-schema` + `anvil://catalog/presets` MCP resources [keep raw `anvil://catalog/knobs`]; the MCP `generate`/`introspect`/`analyze` `profile` input routed through `resolve_config` — **fold its own `agent-mcp.md` docs inline**). Tree stays `active`; book now in sync with shipped surface.
- next_action: **PNT — pick `KNOB-ERGONOMICS-AND-PRESETS.2b.2`** (code: the queryable catalog + the 2 new MCP resources + the MCP `profile` input + `agent-mcp.md` docs inline). SCHEMA-DERIVED only (project `Config::default()` + `Overrides` serde + a metadata table; no recomputed truth — decision `0017` ceiling). Full COMMIT.md workflow incl. snapshots 6/6 + a catalog completeness test; RAM-guard heavy builds (decision `0003`).
- handoff: repo fully handoff-ready & **in sync** after this commit — `.2b.1` code green (full `cargo test`), snapshots 6/6, clippy/fmt clean; book documents the shipped CLI surface (mdbook clean); `check_memory_architecture` + KM gen/check green (**KM 55 facts**); introspection schema `1.11`. No pending doc-drift.
- lane invariants (all lanes): rules-first / no generate-then-filter (valid-by-construction); default-off / byte-identical where output could change; `tests/snapshots.rs` untouched by default; **no retirement** (`feedback_never_retire_strategies`); one runner + one classifier not two (`feedback_full_factorization`); every capability opt-in + MCP-invocable + queryable + CLI-as-shim (decision `0017`); design the API for agents not humans (`feedback_api_for_agents_not_humans`); the book is the user-facing surface and must not drift (`feedback_book_doctrine`); decision/KM fact per durable capability or boundary.
- in_flight_uncommitted: none after this commit. Repo handoff-ready: tree clean, self-checks green, resume pointer current.
- blockers: none. Validation policy: focused checks + `scripts/ram_guard.sh --threshold 90 -- <cmd>` for heavy builds (note the `--` separator); monitor RAM, stop >90% per `0003-resource-safe-validation`.

## Validation policy
- For workflow-doc memory/retrieval architecture leaves, use focused functional checks; full `cargo test` is not required per owner instruction.
- If a future full suite is run, monitor RAM; stop immediately above 90% RAM and record it as an environment/resource stop.
