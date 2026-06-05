---
id: iverilog-compile-matrix-axis
title: tool_matrix has an optional Icarus compile axis
answers:
  - "does tool_matrix support Icarus Verilog compile checks"
  - "what does --iverilog-compile do"
  - "why do static case muxes lower to assign"
  - "why did Icarus warn always_comb process has no sensitivities"
  - "always_comb process has no sensitivities"
date: 2026-06-05
status: current
tags: [tool-matrix, iverilog, emitter, signoff]
evidence: src/bin/tool_matrix.rs; src/emit/sv.rs; book/src/synthesizability.md; DEVELOPMENT_NOTES.md
---

`SIGNOFF-SURFACE-EXPANSION.3` adds `tool_matrix --iverilog-compile`,
an optional Icarus Verilog acceptance column. The harness shells
`iverilog -g2012` for each emitted module/design, records the result in
`iverilog_compile`, and treats warnings as failures. It is compile /
elaboration evidence only; trace agreement remains `--diff-sim`.

The same slice changed `src/emit/sv.rs` so constant-selector
case/casez muxes and constant-source for-folds lower to continuous
`assign` statements. Dynamic selectors/sources still emit the
procedural `always_comb` surfaces. The static lowering removes Icarus
empty-sensitivity warnings while preserving the generated value.
