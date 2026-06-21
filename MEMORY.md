# MEMORY - resume pointer (layer A; overwrite-only, keep <= 50 lines)

## How to resume
- Read `README.md`, then `MEMORY_ARCHITECTURE.md`.
- Work is task-tree tracked under `docs/tasks/`; index: `docs/TASK_TREE.md`.
- Durable cross-task facts live in `docs/decisions/`.
- Question-keyed retrieval facts are indexed in `KNOWLEDGE_MAP.md`.
- Commit completed leaves per `COMMIT.md`; include the leaf id in the subject.

## Current state (overwrite this block; do not append history)
- latest_commit: this `CI-PACKAGING-DISTRIBUTION.2a` commit — the **first impl slice** of decision `0022`: `.github/workflows/release.yml`, the hand-rolled `v*`-tag 5-target release matrix (`x86_64`/`aarch64` linux-gnu, `x86_64`/`aarch64` darwin, `x86_64` windows-msvc) → `cargo build --release --locked --bin anvil --bin anvil-mcp` → per-platform archives (`.tar.gz`/`.zip`, both binaries + README) + per-archive sha → a least-privilege `publish` job that aggregates one `SHA256SUMS` and creates/updates the GitHub Release via the runner-builtin **`gh` CLI** (no third-party release dep). Pin via `RUST_TOOLCHAIN` env (tracks `Cargo.toml` MSRV `1.95`) + `--locked`; aarch64-linux cross linker via `CARGO_TARGET_*_LINKER` env (no `.cargo/config`). **CI-infra only — no `src/`/`tests/`; DUT byte-identical.** Validated: pure-Python structural lint of the YAML clean (5 targets + all tokens; offline — no pyyaml/actionlint/yq) + mem-arch + KM(56) green. Prior: `51d97d9`=`CI-PACKAGING…1` (decision 0022 ADR); `d731087`=`KNOB-ERGONOMICS…2b.2b`. Push cadence: `origin/main` at `7142fd7` → **22 commits ahead** after this; **push at ~30** (`feedback_push_cadence`).
- active_work_unit: **`CI-PACKAGING-DISTRIBUTION` — `.1` (ADR) + `.2a` (release.yml) DONE; frontier = `.2b`** (the composite `action.yml` + POSIX entrypoint: download the pinned release tarball, run `anvil hunt` with inputs mapped 1:1 onto the CLI/MCP controls per decision `0017`, upload the reproducer bundle as a CI artifact, exit-on-finding, + a self-test job that skips clean when tools are absent) → then `.2c` (README/USER_GUIDE "Use ANVIL in your CI" + a KM card; close `.2`). Tree stays `active`.
- next_action: **PNT — continue.** Advance to `CI-PACKAGING-DISTRIBUTION.2b` (the composite Action — CI-infra, task-tree-owned; wraps the shipped `anvil hunt`/`0018`) **or** pick a design-first ADR `.1` (docs-only): `COVERAGE-STEERED-GENERATION.1` (construction-time coverage steering — rules-first, never generate-then-filter) or `CAPABILITY-BREADTH-EXPANSION.1`/`.2` (more SV up-opts / Mealy FSM). Pick-and-roll (`feedback_pick_and_roll_at_no_frontier`); refine acceptance at pick.
- handoff: repo fully handoff-ready & **in sync** after this commit — `cargo check`/`cargo test` unaffected (no Rust touched; full `cargo test` was green at `51d97d9`); snapshots 6/6; `check_memory_architecture` + KM gen/check green (**KM 56 facts**); mdbook clean; introspection schema `1.11`. No pending doc-drift.
- lane invariants (all lanes): rules-first / no generate-then-filter (valid-by-construction); default-off / byte-identical where output could change; `tests/snapshots.rs` untouched by default; **no retirement** (`feedback_never_retire_strategies`); one runner + one classifier not two (`feedback_full_factorization`); every capability opt-in + MCP-invocable + queryable + CLI-as-shim (decision `0017`); design the API for agents not humans (`feedback_api_for_agents_not_humans`); the book is the user-facing surface and must not drift (`feedback_book_doctrine`); decision/KM fact per durable capability or boundary.
- in_flight_uncommitted: none after this commit. Repo handoff-ready: tree clean, self-checks green, resume pointer current.
- blockers: none. Validation policy: focused checks + `scripts/ram_guard.sh --threshold 90 -- <cmd>` for heavy builds (note the `--` separator); monitor RAM, stop >90% per `0003-resource-safe-validation`.

## Validation policy
- For workflow-doc memory/retrieval architecture leaves, use focused functional checks; full `cargo test` is not required per owner instruction.
- If a future full suite is run, monitor RAM; stop immediately above 90% RAM and record it as an environment/resource stop.
