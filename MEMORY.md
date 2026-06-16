# MEMORY - resume pointer (layer A; overwrite-only, keep <= 50 lines)

## How to resume
- Read `README.md`, then `MEMORY_ARCHITECTURE.md`.
- Work is task-tree tracked under `docs/tasks/`; index: `docs/TASK_TREE.md`.
- Durable cross-task facts live in `docs/decisions/`.
- Question-keyed retrieval facts are indexed in `KNOWLEDGE_MAP.md`.
- Commit completed leaves per `COMMIT.md`; include the leaf id in the subject.

## Current state (overwrite this block; do not append history)
- latest_commit: this `IDENTITY-DEEPENING.3b.2b.2a` commit (hash backfills next update; prior: `314664c` = `.3b.2b.1`; `b873b40` = `SV-VERSION-TARGETING.3b.2b`; `e7fa265` = `SEMANTIC-INTROSPECTION-EXPANSION.2b.2`; `cc2b5bc` = `.2b.1`; `63d2622` = `.2a`). Working tree clean after this commit. Push cadence: `origin/main` at `63d2622` → **5 commits ahead after this commit; push at ~30** (`feedback_push_cadence`).
- active_work_unit: **`IDENTITY-DEEPENING.3b.2b.2a` is `done`** — the metric + schema + downstream-bank half of the cross-module sequential merge closeout (default-off / DUT byte-identical). `.3b.2b` → `.3b.2b.1` (mechanism, done) + `.3b.2b.2` (closeout); `.3b.2b.2` → `.3b.2b.2a` (metric/schema/bank, done) + `.3b.2b.2b` (book/USER_GUIDE/ROADMAP/KM narrative, next). Tree `IDENTITY-DEEPENING` stays `active`. Index: `docs/TASK_TREE.md`.
- `.3b.2b.2a` delivered (CODE, default-off / RTL-invisible): factored the non-mutating `group_sequentially_equivalent_modules(&Design)` helper (`src/ir/dedup.rs`) shared by the pass + the metric (counted pairs = what the pass collapses). New `DesignMetrics::sequential_module_proof_signatures` (`Vec<Option<u64>>`, class-id = FNV of class lex-min name) + `num_sequentially_duplicate_module_pairs`, computed in `compute_design`, pre-filtered (zero proof work on default designs). Additive introspection schema MINOR bump **1.3→1.4** (DesignMetrics is in the `--introspect` payload; both fields `#[serde(default)]`) — `SCHEMA_VERSION`, schema doc §4/§7+checklist, `introspect`/`mcp` schema_version assertions, and README/USER_GUIDE/`book/agent-mcp.md` example numbers all synced. Downstream bank `/tmp/anvil-seq-bank/` (merged 2-module delay-line design, one `.sv`/module): Verilator `-Wall` clean, Yosys both modes (non-empty `$_DFF_` netlist), Icarus `-g2012` clean. `--lib` 435/0/2; snapshots 6/6 byte-identical; clippy/fmt clean; mdbook clean.
- next_action: continue PNT (no self-pause). Frontier = **`IDENTITY-DEEPENING.3b.2b.2b`** (`proposed`) — the user-facing narrative closeout: book `factorization.md` + `hierarchy.md` (whole-module sequential equivalence), USER_GUIDE knob row for `hierarchy_sequential_module_dedup`, ROADMAP gap 2 (record the cross-module sequential merge as delivered), and a Knowledge Map card. Resume from `docs/tasks/IDENTITY-DEEPENING.md`. Sibling lanes still open: `STRUCTURED-EMISSION-EXPANSION` (`proposed`); `SEMANTIC-INTROSPECTION-EXPANSION` (`active`, no active frontier — `.3+` open-ended).
- lane invariants (all lanes): rules-first / no generate-then-filter (valid-by-construction); default-off / byte-identical where output could change; `tests/snapshots.rs` untouched by default; **no retirement** (`feedback_never_retire_strategies`); every capability is an opt-in knob; decision/KM fact per durable capability or boundary.
- in_flight_uncommitted: none after this commit. Repo handoff-ready: tree clean, self-checks green, resume pointer current.
- blockers: none. Validation policy: focused checks + `scripts/ram_guard.sh --threshold 90 -- <cmd>` for heavy builds (note the `--` separator); monitor RAM, stop >90% per `0003-resource-safe-validation`.

## Validation policy
- For workflow-doc memory/retrieval architecture leaves, use focused functional checks; full `cargo test` is not required per owner instruction.
- If a future full suite is run, monitor RAM; stop immediately above 90% RAM and record it as an environment/resource stop.
