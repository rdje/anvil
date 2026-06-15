# MEMORY - resume pointer (layer A; overwrite-only, keep <= 50 lines)

## How to resume
- Read `README.md`, then `MEMORY_ARCHITECTURE.md`.
- Work is task-tree tracked under `docs/tasks/`; index: `docs/TASK_TREE.md`.
- Durable cross-task facts live in `docs/decisions/`.
- Question-keyed retrieval facts are indexed in `KNOWLEDGE_MAP.md`.
- Commit completed leaves per `COMMIT.md`; include the leaf id in the subject.

## Current state (overwrite this block; do not append history)
- latest_commit: this `IDENTITY-DEEPENING.1` commit (hash backfills next update; prior: `5de8e91` = `SIGNOFF-AUTOMATION-EXPANSION.2b`; `92a04c8` = `.2a`; `edf79d6` = `.1`). Working tree clean after this commit.
- active_work_unit: **`IDENTITY-DEEPENING`** (Lane 1, **`active`**). Lanes 2 (`AGENT-MCP-EXPANSION`) and 3 (`SIGNOFF-AUTOMATION-EXPANSION`) at handoff/closed. `.1` (decision `0007`) **done** → first sound identity extension chosen + tree split. Frontier leaf: **`.2`** (`pending`, impl). `.3` (module-level sequential equivalence) `proposed`/future. Index: `docs/TASK_TREE.md`.
- next_action: **do `IDENTITY-DEEPENING.2`** — implement the **bounded bisimulation flop merge** designed in decision `0007`: greatest-fixpoint partition refinement over flops (bucket by width/reset_kind/reset_val/flop_domain) + bounded quotient D-cone proof reusing the existing combinational budget (rewrite each `FlopQ` endpoint to its class rep) + a **new default-off `Config` knob** (working name `bisimulation_flop_merge`, also requires node-id/e-graph) + a merge-count metric (`bisimulation_flops_merged`) + a focused **downstream-clean gate** (rules-first duplicated mutually-recursive registers → merge>0, Verilator + both Yosys clean). Knob-off byte-identical (snapshots untouched); existing exact self-hold / same-endpoint / FSM merges still fire. Lives in `src/ir/compact.rs` (beside `merge_equivalent_flops`). Split into `.2a` design-detail + `.2b` impl if broad. Continue PNT (no self-pause — `feedback_no_self_pause_until_trees_closed`).
- soundness (recorded, `0007`): reset base case + bisimulation step ⇒ corresponding Qs equal for all time (coinduction). Strictly generalizes the exact self-hold + same-endpoint classes; correctly excludes retimed state (not bisimilar), reset/domain/width mismatch, memory-state (`memory-identity-boundary`), and any cone over the proof budget (→ conservative split, no merge). BMC rejected (unsound merge proof). Whole stateful-module reachable-product equivalence deferred to `.3`.
- lane invariants: proof-not-resemblance bar; rules-first / no generate-then-filter; bounded by support/node/work budget (12-bit / 128-node / 131072-work) with structural fallback; **no retirement** of any landed merge class; default-off / byte-identical where a merge could change RTL; `--identity-mode relaxed` stays the real off-switch; single identity source stays `src/ir/compact.rs`. A Knowledge Map card captures each new identity fact/boundary.
- in_flight_uncommitted: none after this commit. Repo handoff-ready: tree clean, self-checks green (`check_memory_architecture.sh` + `check_knowledge_map.sh`), KM regenerated (23 facts / 124 keys), resume pointer current.
- blockers: none. Validation policy: focused checks + `scripts/ram_guard.sh --threshold 88` for heavy builds; full `cargo test` not required for docs/decision leaves per owner resource policy (monitor RAM; stop above 90%, >95% reboots). For `.2` (code) run fmt/check/clippy + focused tests + snapshot byte-identical guard. Push cadence: `origin/main` at `381ec01`; **15 commits ahead after this commit — under the ~30 threshold, no push yet** (`feedback_push_cadence`).

## Validation policy
- For workflow-doc memory/retrieval architecture leaves, use focused functional checks; full `cargo test` is not required per owner instruction.
- If a future full suite is run, monitor RAM; stop immediately above 90% RAM and record it as an environment/resource stop.
