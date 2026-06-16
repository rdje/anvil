# MEMORY - resume pointer (layer A; overwrite-only, keep <= 50 lines)

## How to resume
- Read `README.md`, then `MEMORY_ARCHITECTURE.md`.
- Work is task-tree tracked under `docs/tasks/`; index: `docs/TASK_TREE.md`.
- Durable cross-task facts live in `docs/decisions/`.
- Question-keyed retrieval facts are indexed in `KNOWLEDGE_MAP.md`.
- Commit completed leaves per `COMMIT.md`; include the leaf id in the subject.

## Current state (overwrite this block; do not append history)
- latest_commit: this `STRUCTURED-EMISSION-EXPANSION.3` commit (design/decision leaf — decision `0013`: the SECOND structured surface is a default-off, valid-by-construction `generate for` loop emit-projection of an existing `{N{x}}` replication; owner steer *"structured emission: next surface"* → `generate`; **docs-only / no source change**). Prior: `d6a8517` = `.2b.2c` (function-emit user docs); `3c63f96` = `LOCAL-REFERENCE-CACHE.1`. Working tree clean after this commit. Push cadence: `origin/main` at `63d2622` → **24 commits ahead after this commit; push at ~30** (`feedback_push_cadence`).
- active_work_unit: **`STRUCTURED-EMISSION-EXPANSION` tree `active`; frontier = `.4`** (implement the `generate for` loop surface, decision `0013`; pre-split `.4a` design-detail + `.4b` impl). The first surface (combinational `function automatic`, `.1`+`.2`) is delivered end-to-end. Future surfaces (`task` [leading], nested/multi-level `generate`, `interface`/`modport`) are `.5+`, each its own decision. Index: `docs/TASK_TREE.md`.
- `.3` delivered (decision `0013`, docs-only): the second surface = a `generate for` loop projecting an existing `{N{x}}` replication (index-regular by construction) into `genvar gi; generate for (gi=0;gi<N;gi++) assign <wire>[gi] = <x>; endgenerate` (unrolls to exactly the inline replication). Chosen over `task` (ALSO clean for simple comb void tasks on this toolchain — the leading future candidate; `0012`'s "weak task synth" is precisely a multi-output/side-effecting caution), `interface`/`modport` (weak Yosys synth), constant-predicate `generate if` (dead untaken branch; frontend lane already has it). Empirically grounded clean across Verilator `-Wall` + both Yosys + Icarus; DUT emitter has no generate today; frontend lane has `generate if`. Rules-first / default-off `generate_loop_emit_prob` (proposed) ⇒ byte-identical; gate `saw_generate_loop_emit`.
- next_action: **PNT — execute `STRUCTURED-EMISSION-EXPANSION.4a`** (the design-detail leaf for the `generate for` surface, **docs-only, no source**): ground decision `0013` in the real `src/emit/sv.rs` `to_sv_with_modules` + the `{N{x}}` replication (`GateOp::Concat`) source + the `function_emit.rs`/`soft_union.rs` gen-time-annotation precedent. Pin: the exact replication-node selection rule (which `Concat`s qualify; index-regularity), gen-time annotation (`Module.generate_loop_gates`) vs emit-time, the `genvar`/`generate for` rendering + inline-`assign` suppression, the `generate_loop_emit_prob` knob semantics (default `0.0` byte-identical), and the `saw_generate_loop_emit` gate shape. Then `.4b` implements. Doctrine: NO code change without a task-tree leaf owning it first — `.4`/`.4a`/`.4b` now own it.
- lane invariants (all lanes): rules-first / no generate-then-filter (valid-by-construction); default-off / byte-identical where output could change; `tests/snapshots.rs` untouched by default; **no retirement** (`feedback_never_retire_strategies`); every capability is an opt-in knob; decision/KM fact per durable capability or boundary.
- in_flight_uncommitted: none after this commit. Repo handoff-ready: tree clean, self-checks green, resume pointer current.
- blockers: none. Validation policy: focused checks + `scripts/ram_guard.sh --threshold 90 -- <cmd>` for heavy builds (note the `--` separator); monitor RAM, stop >90% per `0003-resource-safe-validation`.

## Validation policy
- For workflow-doc memory/retrieval architecture leaves, use focused functional checks; full `cargo test` is not required per owner instruction.
- If a future full suite is run, monitor RAM; stop immediately above 90% RAM and record it as an environment/resource stop.
