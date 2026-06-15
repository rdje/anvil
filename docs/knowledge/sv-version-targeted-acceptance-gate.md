---
id: sv-version-targeted-acceptance-gate
title: tool_matrix --sv-version-gate proves each IEEE 1800 target is accepted in the matching Verilator --language standard mode
answers:
  - "what does tool_matrix --sv-version-gate do"
  - "how does ANVIL prove version-targeted RTL is accepted by downstream tools"
  - "how does ANVIL run Verilator in a specific SystemVerilog language mode"
  - "which saw_* facts does the sv-version gate require"
  - "what is saw_sv_version_targeted_acceptance"
  - "how does the matrix run Verilator --language 1800-2017 or 1800-2023"
  - "where is the banked sv-version gate report"
  - "does the sv-version matrix gate change emitted RTL"
  - "which scenarios does the sv-version sweep run"
date: 2026-06-16
status: current
tags: [sv-version, tool-matrix, coverage, downstream, verilator, language, acceptance, north-star]
evidence: src/bin/tool_matrix.rs (ScenarioSet::SvVersionSweep, build_sv_version_sweep_scenarios, verilator_language_for, light_sv_version_acceptance, compute_coverage_gaps); src/downstream/mod.rs (run_verilator(_design) language selector); /tmp/anvil-sv-version-gate-r1/tool_matrix_report.json; docs/decisions/0009-sv-version-targeting.md
reverify: cargo run --release --bin tool_matrix -- --sv-version-gate --yosys-mode both --out /tmp/anvil-sv-version-gate-check
---

# `tool_matrix --sv-version-gate` (SV-VERSION-TARGETING.2b.2b)

The repo-owned per-version downstream acceptance axis for the
`--sv-version` capability gate. It industrializes the focused
`.2b.2a` `#[ignore]` proof into a coverage-gated matrix run: each
targeted IEEE 1800 standard's corpus must be **accepted in the matching
tool standard mode**, not merely at the tool's default language.

- Opt-in `--sv-version-gate` ⇒ `ScenarioSet::SvVersionSweep`, mutually
  exclusive with the phase / signoff gates, auto-enables coverage-gap
  failure.
- Scenario set: for each of the three targets (2012 / 2017 / 2023), a
  combinational e-graph leaf (`sv<year>_comb_egraph`), a sequential
  motif leaf (`sv<year>_seq_motif`), and a recursive depth-2 hierarchy
  design (`sv<year>_hier_recursive`) — **9 scenarios**, all `Interleaved`
  (the gate's contract is per-version acceptance, not strategy breadth,
  so `compute_coverage_gaps` returns before the strategy/category checks).
- Each scenario's `Config::sv_version` is set to its target; the matrix
  `to_sv*` emits thread `cfg.sv_version` (`to_sv_versioned` /
  `to_sv_in_design_versioned`).
- Verilator runs in the matching `--language 1800-20xx` mode via
  `verilator_language_for` + the `.2b.2a` `run_verilator(_design)`
  `language` selector; Yosys runs `-sv`. The selector is `None` (today's
  byte-identical argv) for every non-gate run.
- Required coverage facts: `saw_sv_version_2012_targeted_acceptance`,
  `saw_sv_version_2017_targeted_acceptance`,
  `saw_sv_version_2023_targeted_acceptance`, and the umbrella
  `saw_sv_version_targeted_acceptance` (each lit by
  `light_sv_version_acceptance` only when Verilator actually ran and
  succeeded plus clean Yosys, gated on `version_targeted`).
- `MatrixReport.sv_version_gate` records the run mode.

**Emission is byte-identical across the three targets today** — the
current subset is a 2012/2017/2023 common floor — so the gate's value is
the per-version downstream acceptance axis, not output divergence (that
arrives with the future up-opting leaf `.3`). Banked downstream-clean at
`/tmp/anvil-sv-version-gate-r1` (9 scenarios, 18 units,
`coverage_gaps = []`, `18/0` Verilator + both Yosys modes; each
scenario's Verilator argv carries the matching `--language 1800-20xx`).
Default-off / byte-identical; nothing retired. See [[sv-version-targeting]]
(decision `0009`).
