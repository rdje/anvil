# MEMORY - resume pointer (layer A; overwrite-only, keep <= 50 lines)

## How to resume
- Read `README.md`, then `MEMORY_ARCHITECTURE.md`.
- Work is task-tree tracked under `docs/tasks/`; index: `docs/TASK_TREE.md`.
- Durable cross-task facts live in `docs/decisions/`.
- Question-keyed retrieval facts are indexed in `KNOWLEDGE_MAP.md`.
- Commit completed leaves per `COMMIT.md`; include the leaf id in the subject.

## Current state (overwrite this block; do not append history)
- latest_commit: this `STRUCTURED-EMISSION-EXPANSION.7` commit (design/decision leaf — decision `0015` picks the FOURTH structured surface = the **wider-lane `generate for` part-select**; **docs-only / no `src/` / DUT byte-identical**). Prior: `cf25642` = `LIVE-DOC-CODEBASE-ALIGNMENT.2` (test-count `6→8`); `7163d72` = `.6b.3`; … `b90bdff` = `.5` (**pushed** — `origin/main` is at `b90bdff`). Working tree clean after this commit. Push cadence: `origin/main` at `b90bdff` → **7 commits ahead (`.6a`,`.6b.1`,`.6b.2a`,`.6b.2b`,`.6b.3`,`LIVE-DOC-CODEBASE-ALIGNMENT.2`,this `.7`); push at ~30** (`feedback_push_cadence`).
- active_work_unit: **`STRUCTURED-EMISSION-EXPANSION` tree `active`, frontier = `.8a`**. Three structured surfaces shipped end-to-end (`function automatic` `.1`+`.2`, `generate for` loop `.3`+`.4`, `task automatic` `.5`+`.6`); the FOURTH (wider-lane `generate for` part-select, decision `0015`) is design-done (`.7`) and in implementation (`.8`, pre-split `.8a` design-detail + `.8b` impl). Index: `docs/TASK_TREE.md`.
- next_action: **PNT → `STRUCTURED-EMISSION-EXPANSION.8a`** (design-detail leaf, docs-only): ground decision `0015` in the real `src/ir/generate_loop.rs::gate_qualifies` (relax `lane.width()!=1` → `LW>=1`, `width==N*LW`) + `src/emit/sv.rs::generate_loop_gate`/`render_generate_loop_block` (the `[gi]` vs `[gi*LW +: LW]` render branch; keep `LW==1` byte-identical) in a `DEVELOPMENT_NOTES.md` design-detail entry; pin the byte-identity contract for the shipped 1-bit surface + the wider-lane downstream-proof shape. Then `.8b` impl (live surface + lib proofs + wider-lane `--generate-loop-gate` proof + book/USER_GUIDE closeout). `.8` is small/localized; **reuses `generate_loop_emit_prob` + `num_emitted_generate_loops` ⇒ NO new knob / NO schema bump**. Doctrine: NO code change without a task-tree leaf owning it first.
- handoff: repo handoff-ready — `cargo check --all-targets` clean; tree clean. `.7` design fully scoped (decision `0015` + empirical probe banked `/tmp/anvil-probe-se4/`: wider-lane part-select universally clean + sim-proven `== {N{x}}`; `interface`/`modport` empirically disqualified). `.8a` is docs-only; `.8b` is the first source change of the lane's fourth surface (two surgical edits: relax predicate + part-select render branch).
- lane invariants (all lanes): rules-first / no generate-then-filter (valid-by-construction); default-off / byte-identical where output could change; `tests/snapshots.rs` untouched by default; **no retirement** (`feedback_never_retire_strategies`); every capability is an opt-in knob; decision/KM fact per durable capability or boundary.
- in_flight_uncommitted: none after this commit. Repo handoff-ready: tree clean, self-checks green, resume pointer current.
- blockers: none. Validation policy: focused checks + `scripts/ram_guard.sh --threshold 90 -- <cmd>` for heavy builds (note the `--` separator); monitor RAM, stop >90% per `0003-resource-safe-validation`.

## Validation policy
- For workflow-doc memory/retrieval architecture leaves, use focused functional checks; full `cargo test` is not required per owner instruction.
- If a future full suite is run, monitor RAM; stop immediately above 90% RAM and record it as an environment/resource stop.
