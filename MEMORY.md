# MEMORY - resume pointer (layer A; overwrite-only, keep <= 50 lines)

## How to resume
- Read `README.md`, then `MEMORY_ARCHITECTURE.md`.
- Work is task-tree tracked under `docs/tasks/`; index: `docs/TASK_TREE.md`.
- Durable cross-task facts live in `docs/decisions/`.
- Question-keyed retrieval facts are indexed in `KNOWLEDGE_MAP.md`.
- Commit completed leaves per `COMMIT.md`; include the leaf id in the subject.

## Current state (overwrite this block; do not append history)
- latest_commit: this `IDENTITY-DEEPENING.3b.1` commit (hash backfills next update; prior: `ce46141` = `IDENTITY-DEEPENING.3a`; `be45819` = `.2b`). Working tree clean after this commit.
- active_work_unit: **`IDENTITY-DEEPENING`** (Lane 1, **`active`**). `.1` (decision `0007`) + `.2` **done**; `.3a` design (decision `0008`) **done**; `.3b` split into **`.3b.1` design-detail (done)** + **`.3b.2` impl (future)**. Frontier leaf: **`.3b.2`** (`proposed`, implement the cross-module sequential module merge). Index: `docs/TASK_TREE.md`.
- next_action: **`IDENTITY-DEEPENING.3b.2`** — implement decision `0008` per the `.3b.1` design detail. Steps: (1) factor `merge_bisimilar_flops`'s refinement core into a non-mutating `bisimulation_partition(m) -> rep_map` (collapse pass stays byte-identical, snapshots 6/6); (2) `modules_bisimilar(A,B)` via a throwaway **combined Module** (A.nodes++B.nodes offset; A.flops⊎B.flops) — inputs unify for free since `LeafEndpoint::PrimaryInput` keys by `(port,width)`; verdict = interfaces match by `(PortId,width)` AND every output port `p`: `cone_proof(combined, driveA(p), rep_map) == cone_proof(combined, driveB(p)+offset, rep_map)` (NO flop bijection needed); (3) `dedup_sequential_modules` beside `dedup_semantic_modules` — flops-only leaf candidates, pre-filter by (interface, flop multiset, output count), pairwise+union-find, reuse the lex-survivor/rewrite/prune tail; (4) default-off `hierarchy_sequential_module_dedup` knob (Config/Module/Design) + `DesignMetrics::sequential_module_proof_signatures` + `num_sequentially_duplicate_module_pairs`; (5) rules-first cross-module merge gate (permuted/cross-wired equal-reset stateful leaves collapse 2→1, downstream-clean) + knob-off byte-identical; (6) book/USER_GUIDE/CODEBASE/ROADMAP + KM card. Continue PNT (no self-pause — `feedback_no_self_pause_until_trees_closed`).
- `.3b.1` delivered (docs-only, no source change): grounded `.3b.2` in the real code; key findings — **no new proof engine** (combined-module materialization unifies inputs by `(PortId,width)` for free) and **no flop bijection** (interfaces + stable union partition + per-output-cone equality under the quotient is sound by coinduction); reuse via a factored `bisimulation_partition`; pre-filter + union-find grouping reusing `dedup_semantic_modules_once`'s tail. Decision `0008` left unmodified (resolution in the live ledger — `.2a` precedent).
- lane invariants: proof-not-resemblance bar; rules-first / no generate-then-filter; bounded by support/node/work budget (12-bit / 128-node / 131072-work) with structural fallback; **no retirement** of any landed merge class; default-off / byte-identical where a merge could change RTL; `--identity-mode relaxed` stays the real off-switch; single identity source stays `src/ir/compact.rs` + `src/ir/dedup.rs`.
- in_flight_uncommitted: none after this commit. Repo handoff-ready: tree clean, self-checks green, resume pointer current.
- blockers: none. Validation policy: focused checks + `scripts/ram_guard.sh --threshold 88` for heavy builds; for the `.3b.1` docs/design leaf, only `check_memory_architecture.sh` + `check_knowledge_map.sh` are load-bearing (no source change; snapshots untouched ⇒ DUT byte-identical) per `0003-resource-safe-validation`. Push cadence: `origin/main` at `381ec01`; **19 commits ahead after this commit — under the ~30 threshold, no push yet** (`feedback_push_cadence`).

## Validation policy
- For workflow-doc memory/retrieval architecture leaves, use focused functional checks; full `cargo test` is not required per owner instruction.
- If a future full suite is run, monitor RAM; stop immediately above 90% RAM and record it as an environment/resource stop.
