# MEMORY - resume pointer (layer A; overwrite-only, keep <= 50 lines)

## How to resume
- Read `README.md`, then `MEMORY_ARCHITECTURE.md`.
- Work is task-tree tracked under `docs/tasks/`; index: `docs/TASK_TREE.md`.
- Durable cross-task facts live in `docs/decisions/`.
- Question-keyed retrieval facts are indexed in `KNOWLEDGE_MAP.md`.
- Commit completed leaves per `COMMIT.md`; include the leaf id in the subject.

## Current state (overwrite this block; do not append history)
- latest_commit: this `IDENTITY-DEEPENING.3b.2b.1` commit (hash backfills next update; prior: `b873b40` = `SV-VERSION-TARGETING.3b.2b`; `e7fa265` = `SEMANTIC-INTROSPECTION-EXPANSION.2b.2`; `cc2b5bc` = `.2b.1`; `63d2622` = `.2a`). Working tree clean after this commit. Push cadence: `origin/main` at `63d2622` → **4 commits ahead after this commit; push at ~30** (`feedback_push_cadence`).
- active_work_unit: **`IDENTITY-DEEPENING.3b.2b.1` is `done`** — the cross-module whole-leaf-module sequential-equivalence merge **mechanism + proof** landed (default-off / DUT byte-identical). I split the large `.3b.2b` leaf into `.3b.2b.1` (mechanism, done) + `.3b.2b.2` (metric/bank/docs closeout, next). Tree `IDENTITY-DEEPENING` stays `active`. Index: `docs/TASK_TREE.md`.
- `.3b.2b.1` delivered (CODE, default-off / byte-identical): `modules_sequentially_equivalent` (`src/ir/compact.rs`) materializes a combined module `a.nodes++b.nodes`/`a.flops++b.flops` (B's NodeId/FlopId offset; B's `PrimaryInput{port,width}` kept so A/B inputs unify for free), reuses `bisimulation_partition` on the union state, then proves per-output-cone equality under the final quotient (one shared structural interner + fixed-quotient memos). `dedup_sequential_modules` (`src/ir/dedup.rs`) buckets eligible stateful flops-only leaves by `SequentialPrefilterKey` + greedy-by-rep grouping (sound: equivalence is transitive), reusing the survivor/rewrite/prune tail. Default-off `Config::hierarchy_sequential_module_dedup` (config-only, no CLI flag — like siblings) + gated `generate_design` wire-in. `N_BISIM_MODULE_FLOPS=64`; reuses 12-bit/128-node/131072-work budget. 6 rules-first gate tests; `--lib` 433/0/2; snapshots 6/6; clippy/fmt clean; bisim regression intact; no new proof engine (`merge_bisimilar_flops` byte-identical).
- next_action: continue PNT (no self-pause). Frontier = **`IDENTITY-DEEPENING.3b.2b.2`** (`proposed`) — the closeout: `DesignMetrics` `sequential_module_proof_signatures` + `num_sequentially_duplicate_module_pairs` (design-level pairwise grouping, RTL-invisible), a downstream-clean merged-design bank (Verilator + both Yosys), and book (`factorization.md`/`hierarchy.md`) + USER_GUIDE/knobs + ROADMAP gap 2 + CODEBASE + KM card closeout. Resume from `docs/tasks/IDENTITY-DEEPENING.md`. Sibling lanes still open: `STRUCTURED-EMISSION-EXPANSION` (`proposed`); `SEMANTIC-INTROSPECTION-EXPANSION` (`active`, no active frontier — `.3+` open-ended).
- lane invariants (all lanes): rules-first / no generate-then-filter (valid-by-construction); default-off / byte-identical where output could change; `tests/snapshots.rs` untouched by default; **no retirement** (`feedback_never_retire_strategies`); every capability is an opt-in knob; decision/KM fact per durable capability or boundary.
- in_flight_uncommitted: none after this commit. Repo handoff-ready: tree clean, self-checks green, resume pointer current.
- blockers: none. Validation policy: focused checks + `scripts/ram_guard.sh --threshold 90 -- <cmd>` for heavy builds (note the `--` separator); monitor RAM, stop >90% per `0003-resource-safe-validation`.

## Validation policy
- For workflow-doc memory/retrieval architecture leaves, use focused functional checks; full `cargo test` is not required per owner instruction.
- If a future full suite is run, monitor RAM; stop immediately above 90% RAM and record it as an environment/resource stop.
