# MEMORY - resume pointer (layer A; overwrite-only, keep <= 50 lines)

## How to resume
- Read `README.md`, then `MEMORY_ARCHITECTURE.md`.
- Work is task-tree tracked under `docs/tasks/`; index: `docs/TASK_TREE.md`.
- Durable cross-task facts live in `docs/decisions/`.
- Question-keyed retrieval facts are indexed in `KNOWLEDGE_MAP.md`.
- Commit completed leaves per `COMMIT.md`; include the leaf id in the subject.

## Current state (overwrite this block; do not append history)
- latest_commit: this `IDENTITY-DEEPENING.3a` commit (hash backfills next update; prior: `be45819` = `IDENTITY-DEEPENING.2b`; `1d0a8d0` = `.2a`). Working tree clean after this commit.
- active_work_unit: **`IDENTITY-DEEPENING`** (Lane 1, **`active`**). `.1` (decision `0007`) **done**; `.2` (`.2a`+`.2b`) **done**; `.3` split into **`.3a` design (done, decision `0008`)** + **`.3b` impl (future)**. Frontier leaf: **`.3b`** (`proposed`, implement the cross-module whole-module sequential merge). Index: `docs/TASK_TREE.md`.
- next_action: **`IDENTITY-DEEPENING.3b`** — implement decision `0008`: a default-off `hierarchy_sequential_module_dedup` pass *beside* `dedup_semantic_modules` that merges two stateful (flops-only) leaf modules proven observationally equivalent via cross-module bisimulation (lift `.2b` partition refinement to the disjoint union of both modules' flops, primary inputs unified by `(PortId,width)`; prove a stable state correspondence, then output cones equal under the quotient; reuse 12-bit/128-node/131072-work budget). **Central impl challenge: a cross-module cone-proof signature** (shared `LeafEndpoint` vocab — `PrimaryInput` by `(PortId,width)`, `FlopQ` by global union class id). Default-off / byte-identical; rules-first cross-module merge gate + downstream-clean bank. Continue PNT (no self-pause — `feedback_no_self_pause_until_trees_closed`).
- `.3a` delivered (docs-only, no source change): decision `0008` — soundness (reset base case + cross-module bisimulation step, coinduction), budget, control surface, downstream gate, first-cut scope (flops-only; memory/FSM/instance/param/aggregate excluded; resetless excluded), and the `.3b` cross-module-proof challenge. Grounded in `dedup_semantic_modules` (`src/ir/dedup.rs`, `semantic_module_proof_inner` in `src/metrics.rs`) + `merge_bisimilar_flops`/`cone_proof`/`MERGE_SEMANTIC_LIMITS` (`src/ir/compact.rs`). Generalizes the pure-combinational module proof (zero-flop special case); retires nothing.
- lane invariants: proof-not-resemblance bar; rules-first / no generate-then-filter; bounded by support/node/work budget (12-bit / 128-node / 131072-work) with structural fallback; **no retirement** of any landed merge class; default-off / byte-identical where a merge could change RTL; `--identity-mode relaxed` stays the real off-switch; single identity source stays `src/ir/compact.rs` + `src/ir/dedup.rs`. Decision/KM fact per new identity step (`.3a` added decision `0008`'s `answers:`).
- in_flight_uncommitted: none after this commit. Repo handoff-ready: tree clean, self-checks green, resume pointer current.
- blockers: none. Validation policy: focused checks + `scripts/ram_guard.sh --threshold 88` for heavy builds; for the `.3a` docs/design leaf, only `check_memory_architecture.sh` + `check_knowledge_map.sh` are load-bearing (no source change; snapshots untouched ⇒ DUT byte-identical) per `0003-resource-safe-validation`. Push cadence: `origin/main` at `381ec01`; **18 commits ahead after this commit — under the ~30 threshold, no push yet** (`feedback_push_cadence`).

## Validation policy
- For workflow-doc memory/retrieval architecture leaves, use focused functional checks; full `cargo test` is not required per owner instruction.
- If a future full suite is run, monitor RAM; stop immediately above 90% RAM and record it as an environment/resource stop.
