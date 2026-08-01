# anvil-case-qualifier-gate-r1

Evidence digest — decision [`0030`](../decisions/0030-durable-closure-evidence-citations.md).
Derived from the run's `tool_matrix_report.json` by `scripts/evidence_digest.sh`; the
corpus itself is not tracked (bulk), this digest is.

- bank: `anvil-case-qualifier-gate-r1`
- claim: The unique/priority case qualifiers (decision 0044) are downstream-clean on both selector kinds across all three construction strategies, and the qualifier pass co-occurs with the eighth/ninth if-chain surfaces it excludes — the lane s only NON-VACUOUS sibling exclusion, exercised end-to-end. 359 qualified statements (173 unique / 186 priority) over 52 modules, Verilator 52/0 and BOTH Yosys modes 52/0; Icarus runs the 28 non-unique modules 28/0 and is a recorded accepting no-op for the 24 unique ones.
- owning_leaf: `CAPABILITY-BREADTH-EXPANSION.4b.2b`
- commit: `955e387`
- date: `2026-08-01`
- command: `cargo run --release --bin tool_matrix -- --case-qualifier-gate --yosys-mode both --iverilog-compile --out .cache/anvil-sandbox/anvil-case-qualifier-gate-r1`
- report_sha256: `6ea6468dd258642b40fdb2fadc9bb61ca04c3c9cb95a957f85207b31f95f9752`
- coverage_gaps: `[]` — none

## Run shape

- scenario_set: `case-qualifier-sweep`
- artifact_kind: `module`
- base_seed: `0`
- scenarios: `13` × `4` per scenario = **`52` units**
- yosys_mode: `both`

## Tool columns

| column | pass | fail |
| --- | --- | --- |
| Verilator | 52 | 0 |
| Yosys without-abc | 52 | 0 |
| Yosys with-abc | 52 | 0 |
| Icarus compile | 28 | 0 |
| sv2v | 0 | 0 |
| slang | 0 | 0 |

## Coverage facts lit

- `saw_case_mux`
- `saw_case_mux_if_emit`
- `saw_case_qualifier_beside_if_chain`
- `saw_casez_mux`
- `saw_casez_mux_if_emit`
- `saw_comb_only_module`
- `saw_for_fold`
- `saw_multi_surface_emit_interaction`
- `saw_priority_case_qualifier`
- `saw_priority_encoder`
- `saw_semantic_gate_merge`
- `saw_unique_case_qualifier`
- `saw_variable_shift`

## Re-verification

Re-run `command` above at commit `955e387`. The numbers here are what a clean
re-run must reproduce; a divergence is a finding, not a stale digest.
Later commits may legitimately produce different numbers — that is why the
commit is recorded alongside them.

> **`commit` was backfilled at `CAPABILITY-BREADTH-EXPANSION.4b.3`.** The generator
> produced `c1cc6a1ff1f9` — the commit the run executed *against* — because the gate was
> banked by the same commit that introduced it, so its hash did not yet exist.
> `scripts/evidence_digest.sh` warns about exactly this case and names backfill as the
> remedy. `955e387` is the commit that contains the gate, and is therefore the one a
> re-verification must check out.
