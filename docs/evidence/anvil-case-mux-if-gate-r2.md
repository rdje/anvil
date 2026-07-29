# anvil-case-mux-if-gate-r2

Evidence digest — decision [`0030`](../decisions/0030-durable-closure-evidence-citations.md).
Derived from the run's `tool_matrix_report.json` by `scripts/evidence_digest.sh`; the
corpus itself is not tracked (bulk), this digest is.

- bank: `anvil-case-mux-if-gate-r2`
- claim: The eighth structured-emission surface (`case_mux_if`, decision `0028`) fires by construction and is downstream-clean — the first digest banked under decision `0030`.
- owning_leaf: `EVIDENCE-BANK-DURABILITY.5`
- commit: `45cb334010b8`
- date: `2026-07-30`
- command: `cargo run --release --bin tool_matrix -- --case-mux-if-gate --yosys-mode both --iverilog-compile --out .cache/anvil-sandbox/anvil-case-mux-if-gate-r2`
- report_sha256: `d789b736203df27cb2807728dd1a3de03a25056528caca755da277bf061137cf`
- coverage_gaps: `[]` — none

## Run shape

- scenario_set: `case-mux-if-sweep`
- artifact_kind: `module`
- base_seed: `0`
- scenarios: `3` × `4` per scenario = **`12` units**
- yosys_mode: `both`

## Tool columns

| column | pass | fail |
| --- | --- | --- |
| Verilator | 12 | 0 |
| Yosys without-abc | 12 | 0 |
| Yosys with-abc | 12 | 0 |
| Icarus compile | 12 | 0 |
| sv2v | 0 | 0 |
| slang | 0 | 0 |

## Coverage facts lit

- `saw_case_mux`
- `saw_case_mux_if_emit`
- `saw_comb_only_module`
- `saw_for_fold`
- `saw_priority_encoder`
- `saw_semantic_gate_merge`
- `saw_variable_shift`

## Re-verification

Re-run `command` above at commit `45cb334010b8`. The numbers here are what a clean
re-run must reproduce; a divergence is a finding, not a stale digest.
Later commits may legitimately produce different numbers — that is why the
commit is recorded alongside them.
