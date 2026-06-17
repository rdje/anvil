---
id: sv2v-adapter
title: ANVIL has an sv2v transpile acceptance adapter (the first new downstream tool)
answers:
  - "does ANVIL have an sv2v column"
  - "does tool_matrix support sv2v"
  - "how do I run sv2v on ANVIL output"
  - "is sv2v a downstream tool in ANVIL"
  - "how do I add the sv2v transpile acceptance column"
  - "can I select sv2v over the MCP tools arg"
  - "what is the first new downstream adapter in ANVIL"
  - "what happens when sv2v is not installed"
date: 2026-06-18
status: current
tags: [tool-matrix, sv2v, downstream, adapter, registry, mcp, signoff]
evidence: 'cargo test --test sv2v_e2e   (portable: sv2v is a public selectable adapter; the real-tool gate is #[ignore], skips green when sv2v is absent). Also: cargo run --bin tool_matrix -- --out /tmp/x --skip-verilator --skip-yosys --sv2v  ⇒  exits 0 with "sv2v pass/fail = 0/0" and no sv2v invocations when sv2v is absent (the friendly no-op).'
---

`DOWNSTREAM-ADAPTER-EXPANSION.2b` (decision `0020`) lands **`sv2v`** as the
first downstream adapter beyond the original Verilator/Yosys/Icarus three —
an `sv2v` SystemVerilog→Verilog-2005 **transpile** accept/reject column. A
clean transpile accepts; a non-zero exit or a warning is a finding. It is an
acceptance gate, not a behavioural oracle (the transpiled Verilog is
discarded).

- **`.2b.1` — selectable + discoverable over the API.** `sv2v` is a fourth
  `AcceptanceTool` (`from_name("sv2v")`) + `run_sv2v`/`run_sv2v_design`
  primitives + an `Sv2vAdapter` in the closed `adapters()` registry, so it is
  selectable via the `validate`/`hunt`/`divergence`/`minimize` `tools` arg and
  the `anvil hunt --tools` CLI, and appears in `adapter_catalog()` (the
  `anvil://catalog/adapters` MCP resource).
- **`.2b.2` — the `tool_matrix` column.** `tool_matrix --sv2v` (override the
  binary with `--sv2v-bin`) records `ModuleReport.sv2v` / `DesignReport.sv2v`
  and tallies `sv2v pass/fail`. A `union soft` up-opt module skips it alongside
  Yosys/Icarus.

**Friendly absent-tool no-op.** `sv2v` is absent on most hosts. A presence
probe (`downstream::tool_version`) means a requested-but-missing `sv2v` records
*no* column and never fails the run; `brew install sv2v` lights it up. The
real-tool proof is the `#[ignore]` gate in `tests/sv2v_e2e.rs` (skips green when
absent). Default-off ⇒ DUT byte-identical; banked reports + `--resume` are
unchanged.

The richer adapter (`slang`, with the optional JSON-AST `extract_facts` hook) is
`.2c`. See `book/src/synthesizability.md` (the acceptance columns) and
`book/src/agent-mcp.md` (the adapter registry / catalog).
