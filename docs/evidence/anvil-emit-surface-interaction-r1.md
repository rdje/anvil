# anvil-emit-surface-interaction-r1

Evidence digest — decision [`0030`](../decisions/0030-durable-closure-evidence-citations.md).
Derived from the run's `tool_matrix_report.json` by `scripts/evidence_digest.sh`; the
corpus itself is not tracked (bulk), this digest is.

- bank: `anvil-emit-surface-interaction-r1`
- claim: The nine structured-emission surfaces are downstream-clean IN COMBINATION, and genuinely co-occur: 8 surfaces in every module of the three universal scenarios, 9 in the Verilator-only 2023 scenario, and exactly 3 under saturation — the first end-to-end exercise of the mutual-exclusion invariant that makes stacking nine projections sound (decision 0032).
- owning_leaf: `EMIT-SURFACE-INTERACTION-GATE.3`
- commit: `d73b1545a577`
- date: `2026-07-30`
- command: `cargo run --release --bin tool_matrix -- --emit-surface-interaction-gate --yosys-mode both --iverilog-compile --out .cache/anvil-sandbox/anvil-emit-surface-interaction-r1`
- report_sha256: `996fae2885e7e31e7700d65c64d120b29bff297e48c75c7790f14b929f58d386`
- coverage_gaps: `[]` — none

## Run shape

- scenario_set: `emit-surface-interaction`
- artifact_kind: `module`
- base_seed: `0`
- scenarios: `5` × `4` per scenario = **`20` units**
- yosys_mode: `both`

## Tool columns

| column | pass | fail |
| --- | --- | --- |
| Verilator | 20 | 0 |
| Yosys without-abc | 16 | 0 |
| Yosys with-abc | 16 | 0 |
| Icarus compile | 16 | 0 |
| sv2v | 0 | 0 |
| slang | 0 | 0 |

## Coverage facts lit

- `saw_all_emit_surfaces_in_one_module`
- `saw_all_nine_emit_surfaces_in_one_module`
- `saw_case_mux`
- `saw_case_mux_if_emit`
- `saw_casez_mux`
- `saw_casez_mux_if_emit`
- `saw_comb_mux_encoded`
- `saw_comb_mux_one_hot`
- `saw_comb_only_module`
- `saw_combinational_function_emit`
- `saw_combinational_task_emit`
- `saw_cone_function_emit`
- `saw_for_fold`
- `saw_generate_loop_emit`
- `saw_multi_output_task_emit`
- `saw_multi_surface_emit_interaction`
- `saw_mux_if_emit`
- `saw_priority_encoder`
- `saw_semantic_gate_merge`
- `saw_variable_shift`

## Re-verification

Re-run `command` above at commit `d73b1545a577`. The numbers here are what a clean
re-run must reproduce; a divergence is a finding, not a stale digest.
Later commits may legitimately produce different numbers — that is why the
commit is recorded alongside them.
