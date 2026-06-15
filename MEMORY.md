# MEMORY - resume pointer (layer A; overwrite-only, keep <= 50 lines)

## How to resume
- Read `README.md`, then `MEMORY_ARCHITECTURE.md`.
- Work is task-tree tracked under `docs/tasks/`; index: `docs/TASK_TREE.md`.
- Durable cross-task facts live in `docs/decisions/`.
- Question-keyed retrieval facts are indexed in `KNOWLEDGE_MAP.md`.
- Commit completed leaves per `COMMIT.md`; include the leaf id in the subject.

## Current state (overwrite this block; do not append history)
- latest_commit: this `IDENTITY-DEEPENING.2b` commit (hash backfills next update; prior: `1d0a8d0` = `IDENTITY-DEEPENING.2a`; `43e2a2d` = `.1`). Working tree clean after this commit.
- active_work_unit: **`IDENTITY-DEEPENING`** (Lane 1, **`active`**). `.1` (decision `0007`) **done**; `.2` (`.2a` design + `.2b` impl) **done** — `.2` container closed. Frontier leaf: **`.3`** (`proposed`, whole-module sequential equivalence — a DESIGN leaf first). Index: `docs/TASK_TREE.md`.
- next_action: **`IDENTITY-DEEPENING.3`** — whole stateful-leaf-module bounded sequential equivalence built on the `.2` flop-bisimulation primitive + a bounded state-correspondence search (extends `dedup_semantic_modules` past the pure-combinational boundary). Design leaf FIRST (soundness + budget + downstream gate) before any code. Continue PNT (no self-pause — `feedback_no_self_pause_until_trees_closed`).
- `.2b` delivered: opt-in `merge_bisimilar_flops` in `src/ir/compact.rs` (greatest-fixpoint partition refinement, quotient `FlopQ→class-rep` signature via the threaded `canonical_flop_endpoint`, fresh memos per iteration); shared `finalize_flop_merge` refactor (exact pass byte-identical); default-off `Config`/`Module` `bisimulation_flop_merge` (node-id/e-graph only); `Metrics::bisimulation_flops_merged`. **Soundness fix beyond `.2a`: resetless flops excluded (no base case).** Schema MINOR bump 1.0→1.1. 6 rules-first gate tests; snapshots 6/6 byte-identical; downstream-clean bank (Verilator `-Wall` 0 warn + both Yosys + Icarus on the merged self-hold output).
- lane invariants: proof-not-resemblance bar; rules-first / no generate-then-filter; bounded by support/node/work budget (12-bit / 128-node / 131072-work) with structural fallback; **no retirement** of any landed merge class; default-off / byte-identical where a merge could change RTL; `--identity-mode relaxed` stays the real off-switch; single identity source stays `src/ir/compact.rs`. KM card per new identity fact (`.2b` added `bisimulation-flop-merge`).
- in_flight_uncommitted: none after this commit. Repo handoff-ready: tree clean, self-checks green, resume pointer current.
- blockers: none. Validation policy: focused checks + `scripts/ram_guard.sh --threshold 88` for heavy builds; full `cargo test` (heavy/slow `pipeline` suite) not run end-to-end — it exercises only the byte-identical default path, proven by snapshots 6/6 + 403 lib tests; per owner resource policy (monitor RAM; stop above 90%, >95% reboots). Push cadence: `origin/main` at `381ec01`; **17 commits ahead after this commit — under the ~30 threshold, no push yet** (`feedback_push_cadence`).

## Validation policy
- For workflow-doc memory/retrieval architecture leaves, use focused functional checks; full `cargo test` is not required per owner instruction.
- If a future full suite is run, monitor RAM; stop immediately above 90% RAM and record it as an environment/resource stop.
