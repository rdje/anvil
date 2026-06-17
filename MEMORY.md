# MEMORY - resume pointer (layer A; overwrite-only, keep <= 50 lines)

## How to resume
- Read `README.md`, then `MEMORY_ARCHITECTURE.md`.
- Work is task-tree tracked under `docs/tasks/`; index: `docs/TASK_TREE.md`.
- Durable cross-task facts live in `docs/decisions/`.
- Question-keyed retrieval facts are indexed in `KNOWLEDGE_MAP.md`.
- Commit completed leaves per `COMMIT.md`; include the leaf id in the subject.

## Current state (overwrite this block; do not append history)
- latest_commit: this `STRUCTURED-EMISSION-EXPANSION.9` commit (design/decision leaf — picked the FIFTH structured surface, the **multi-gate-cone `function automatic`**; decision `0016`; **docs-only, no source change, DUT byte-identical**; frontier opens → `.10a`). Prior: `afb5718` = `.8b`; `ab73083` = `.8a`; `3ec4f66` = `.7` (decision `0015`); `cf25642` = `LIVE-DOC-CODEBASE-ALIGNMENT.2`; … `b90bdff` = `.5` (**pushed** — `origin/main` is at `b90bdff`). Working tree clean after this commit. Push cadence: `origin/main` at `b90bdff` → **10 commits ahead (`.6a`,`.6b.1`,`.6b.2a`,`.6b.2b`,`.6b.3`,`LIVE-DOC-CODEBASE-ALIGNMENT.2`,`.7`,`.8a`,`.8b`,this `.9`); push at ~30** (`feedback_push_cadence`).
- active_work_unit: **`STRUCTURED-EMISSION-EXPANSION` tree `active`, frontier = `.10a`**. **FOUR** structured surfaces delivered end-to-end: `function automatic` (`.1`+`.2`), `generate for` loop (`.3`+`.4`), `task automatic` (`.5`+`.6`), wider-lane `generate for` part-select (`.7`+`.8`, decision `0015`). The **FIFTH** surface is picked: the **multi-gate-cone `function automatic`** (decision `0016`; `.9` design done; `.10` impl pending, pre-split `.10a` design-detail + `.10b` impl). Future surfaces (multi-output tasks / nested-multi-level `generate` / `interface`-`modport`) are `.11+`. Index: `docs/TASK_TREE.md`.
- next_action: **PNT — implement frontier leaf `STRUCTURED-EMISSION-EXPANSION.10a`** (the design-detail leaf): ground decision `0016` in the real `src/ir/function_emit.rs` (the single-gate predicate to fork), `src/introspect/analyze.rs` (`output_support` cone-walk to reuse), and `src/emit/sv.rs` (the `<wire>__f` decl/call path to extend to a multi-statement body); pin the `cone_function_emit_prob` knob, the interior-node admissibility set + fanout handling, the topo-order/local-naming scheme, the pass ordering + mutual exclusion vs the four sibling projections, and the `num_emitted_cone_functions` metric (schema `1.10 → 1.11`) + `--cone-function-gate` / `saw_cone_function_emit`. `.10a` is a `DEVELOPMENT_NOTES.md` design-detail leaf (no source); `.10b` then implements. Doctrine: NO code change without a task-tree leaf owning it first (`.10b` owns the impl). **Consider a push** — 10 commits ahead (cadence ~30).
- handoff: repo handoff-ready — `cargo check --all-targets` clean + `cargo test --lib` 493 + snapshots 6/6 per the `.8b` bank (this `.9` leaf touched no source); tree clean; all self-checks green; KM regenerated (42 → 43 facts). The fifth surface is *decided* (`.9`), not yet *delivered* — `.10` (pre-split `.10a`/`.10b`) implements it.
- lane invariants (all lanes): rules-first / no generate-then-filter (valid-by-construction); default-off / byte-identical where output could change; `tests/snapshots.rs` untouched by default; **no retirement** (`feedback_never_retire_strategies`); every capability is an opt-in knob (the fifth surface gets its OWN `cone_function_emit_prob` so the single-gate surface stays byte-identical); decision/KM fact per durable capability or boundary.
- in_flight_uncommitted: none after this commit. Repo handoff-ready: tree clean, self-checks green, resume pointer current.
- blockers: none. Validation policy: focused checks + `scripts/ram_guard.sh --threshold 90 -- <cmd>` for heavy builds (note the `--` separator); monitor RAM, stop >90% per `0003-resource-safe-validation`.

## Validation policy
- For workflow-doc memory/retrieval architecture leaves, use focused functional checks; full `cargo test` is not required per owner instruction.
- If a future full suite is run, monitor RAM; stop immediately above 90% RAM and record it as an environment/resource stop.
