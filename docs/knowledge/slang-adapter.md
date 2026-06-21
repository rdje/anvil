---
id: slang-adapter
title: ANVIL has a slang elaboration acceptance adapter (the first fact-bearing downstream tool)
answers:
  - "does ANVIL have a slang column"
  - "does tool_matrix support slang"
  - "how do I run slang on ANVIL output"
  - "is slang a downstream tool in ANVIL"
  - "how do I add the slang elaboration acceptance column"
  - "can I select slang over the MCP tools arg"
  - "what is the first fact-bearing downstream adapter in ANVIL"
  - "how does ANVIL extract facts from slang --ast-json"
  - "what happens when slang is not installed"
date: 2026-06-21
status: current
tags: [tool-matrix, slang, downstream, adapter, registry, mcp, facts, ast-json, signoff]
evidence: 'cargo test --test slang_e2e   (portable: slang is a public selectable, fact-bearing adapter — supports_facts=true; the real-tool gate is #[ignore], skips green when slang is absent). Also: cargo run --bin tool_matrix -- --out /tmp/x --skip-verilator --skip-yosys --slang  ⇒  exits 0 with "slang pass/fail = 0/0" and no slang invocations when slang is absent (the friendly no-op).'
---

`DOWNSTREAM-ADAPTER-EXPANSION.2c` (decision `0020`) lands **`slang`** as the
second downstream adapter beyond the original Verilator/Yosys/Icarus three, and
the **first fact-bearing one** — a strict, fast, independent SystemVerilog
**elaboration** accept/reject column that *also* projects structured facts. A
clean elaboration accepts; a non-zero exit or a warning is a finding. It is an
acceptance gate, not a behavioural oracle.

- **`.2c.1` — selectable + discoverable + the `extract_facts` hook.** `slang` is
  a fifth `AcceptanceTool` (`from_name("slang")`) + `run_slang`/`run_slang_design`
  primitives + a `SlangAdapter` in the closed `adapters()` registry, so it is
  selectable via the `validate`/`hunt`/`divergence`/`minimize` `tools` arg and the
  `anvil hunt --tools` CLI, and appears in `adapter_catalog()` (the
  `anvil://catalog/adapters` MCP resource) as the first entry with
  `supports_facts = true`. It is the trait's first `extract_facts` hook: the pure
  `parse_slang_ast_facts` projects `slang --ast-json` (run as
  `slang <sv> -q --ast-json <stem>.slang.json`) into `AdapterFacts` — the top, its
  ports (`name`/`direction`/`type`), and its child instances
  (`name`/`definition`). SCHEMA-DERIVED: a pure read of *slang's* AST, never an
  ANVIL behavioural oracle (decision `0004`).
- **`.2c.2a` — the `tool_matrix` column.** `tool_matrix --slang` (override the
  binary with `--slang-bin`) records `ModuleReport.slang` / `DesignReport.slang`
  and tallies `slang pass/fail`. Surfacing the extracted `AdapterFacts` into the
  matrix report is the follow-up `.2c.2b`.

**Friendly absent-tool no-op.** `slang` is absent on most hosts. A presence probe
(`downstream::tool_version`) means a requested-but-missing `slang` records *no*
column and never fails the run. The real-tool proof is the `#[ignore]` gate in
`tests/slang_e2e.rs` (skips green when absent); since slang was absent at landing,
its `--ast-json` schema + argv were verified against slang's published docs and the
parser is proven against a faithful synthetic fixture. Default-off ⇒ DUT
byte-identical; banked reports + `--resume` are unchanged.

See `book/src/synthesizability.md` (the acceptance columns) and
`book/src/agent-mcp.md` (the adapter registry / catalog). The accept/reject-only
predecessor is [[sv2v-adapter]]; the interface is
[[downstream-adapter-interface]].
