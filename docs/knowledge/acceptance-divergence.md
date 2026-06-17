---
id: acceptance-divergence
title: How to detect cross-tool acceptance divergence — the `tool_matrix --divergence` column, the `anvil hunt --divergence` axis, and the MCP `divergence` tool
answers:
  - "how do I detect when one tool accepts and another rejects ANVIL output"
  - "how do I run an acceptance-divergence check"
  - "how do I turn on the tool_matrix divergence column"
  - "how do I run anvil hunt with divergence detection"
  - "how do I find verilator-vs-yosys acceptance disagreements"
  - "what is a DivergenceReport and what fields does it have"
  - "how is acceptance divergence different from --diff-sim"
  - "how do I detect tool-version-vs-version disagreement"
  - "is the divergence version axis exposed over MCP or the CLI"
  - "how do I confirm the divergence detector works end to end"
  - "what does saw_acceptance_divergence mean"
date: 2026-06-17
status: current
tags: [divergence, acceptance, downstream, tool_matrix, hunt, mcp, reproducer, usability, north-star]
evidence: src/divergence/mod.rs (`divergence::run` / `classify_report` / `classify_version_mismatch` + `DivergenceReport`/`ToolDecision`/`Divergence`/`DivergenceOptions`); src/downstream/mod.rs (`tool_verdict` accept/warn/reject + `validate`/`validate_tool_specs`/`ToolSpec`/`ToolInvocation.version`/`tool_version`); src/bin/tool_matrix.rs (the `--divergence` column + `unit_divergence` + `saw_acceptance_divergence`); src/hunt/mod.rs (`HuntRequest.divergence` → `acceptance_divergence` finding); src/mcp/mod.rs (the controlled `divergence` tool `run_divergence`); src/main.rs (`anvil hunt --divergence`); tests/divergence_e2e.rs (the real-tool e2e gate); book/src/synthesizability.md ("Acceptance divergence across tools"); book/src/agent-mcp.md (the `divergence` tool); docs/decisions/0019-acceptance-divergence-hunting.md
reverify: 'cargo test --test divergence_e2e -- --ignored   (tool-gated: with Verilator [+ optionally Yosys] on $PATH, asserts an all-agree real-tool sweep records diverged=false and a synthetic accept/reject pair classifies accept_reject; tool-less ⇒ the portable synthetic test still passes, the real-tool tests skip green)'
---

# `ACCEPTANCE-DIVERGENCE-HUNTING` — detecting cross-tool acceptance divergence

ANVIL detects **acceptance divergence**: where one downstream tool *accepts* a
valid-by-construction artifact and another **warns or rejects** it. On legal RTL
every such disagreement is a real downstream-tool bug — the north star. It is the
complement of `--diff-sim`: that proves cross-*simulator* **trace** agreement
(behaviour, after both tools accept); this is the cross-*tool* **acceptance** axis
(legality). The detector lives in one shared place (`divergence::run`, decision
[[acceptance-divergence-hunting]]), is **default-off** (changes no emitted RTL),
and is surfaced **three ways over the one detector** (decision `0017` — one home,
no drift):

- **`tool_matrix --divergence`** — a per-unit `DivergenceReport` over the tools
  the matrix already ran (no extra spawn; no tool-clean precondition):
  ```
  cargo run --bin tool_matrix -- --divergence --out ./tool-matrix
  ```
- **`anvil hunt --divergence`** — a swept finding with
  `detection = "acceptance_divergence"` (not minimized — the `validate` oracle
  can't preserve a cross-tool *disagreement*):
  ```
  anvil hunt --seed 1 --seeds 16 --tools verilator,yosys --divergence
  ```
- **the MCP `divergence` controlled tool** — for an agent (single `(seed, config)`
  shim over `divergence::run`; caches each divergent `run_id`; audit-logged).

A verdict is a trinary `accept` / `warn` / `reject` projection of one
`ToolInvocation` (the shared `downstream::tool_verdict`, **not** a second
classifier). A divergence is "not all labelled-tool verdicts equal", classed
`accept_reject` | `accept_warn` | `warn_reject`. The unit of comparison is a
*labelled tool*, so `--yosys-mode both` contributes two labels and a
without-abc-vs-with-abc disagreement is itself a divergence.

`saw_acceptance_divergence` is **opportunistic, never a required gate** — on
valid-by-construction RTL the steady state is that all tools **agree**
(`diverged: false`), so a required-divergence gate would fail on clean output.

The **tool-version-vs-version** axis (one allow-listed kind, two caller-supplied
binaries — e.g. `verilator-5.046` vs `verilator-4.228` — classified
`version_mismatch`) is a **library surface only**
(`downstream::validate_tool_specs` + `DivergenceOptions.tool_specs`): an
allow-listed kind with an arbitrary caller-supplied binary path is a larger trust
surface than the fixed-binary tools, so it is deliberately **not** exposed over
MCP/CLI (decision `0019` `.2f` follow-up; nothing retired).

See [[acceptance-divergence-hunting]] for the design (the verdict/classifier, the
report shape, the three surfaces, the reproducer reuse, the honesty boundary) and
`book/src/synthesizability.md` → "Acceptance divergence across tools" for the
user-facing contract.
