---
id: verilator-json-frontend-parity
title: Verilator JSON checks all frontend manifest categories
answers:
  - "can ANVIL check frontend manifests with Verilator JSON"
  - "what does parity_against_real_verilator_json_frontend_ast verify"
  - "does Verilator expose frontend top localparams and package constants"
  - "which frontend facts does the Verilator JSON gate check"
date: 2026-06-05
status: current
tags: [frontend, verilator, parity, signoff]
evidence: tests/frontend_parity.rs; book/src/ir.md; USER_GUIDE.md; ROADMAP.md; DEVELOPMENT_NOTES.md
---

`SIGNOFF-SURFACE-EXPANSION.2` adds an optional Verilator JSON-AST
frontend parity gate. When local Verilator supports `--json-only`,
`tests/frontend_parity.rs::parity_against_real_verilator_json_frontend_ast`
generates the 5 Phase-8 reproducibility seeds, asks Verilator for JSON,
builds a `ToolReport`, and compares it with `ParityScope::all()`.

The gate covers all 7 frontend manifest categories:
Seed/Top/PackageConstants/TopParams/TopLocalparams/Instances/
GenerateBranches. It reads top GPARAM/LPARAM `VAR.valuep[CONST]`
entries, package LPARAM constants, per-instance specialized child
module GPARAMs reached through each top `CELL.modp`, and surviving
`GENBLOCK` names. Local Verilator 5.046 rejects `--xml-only` but
supports `--json-only`; `slang` was not present and is not required for
this gate.
