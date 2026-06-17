---
id: bug-hunt-cli
title: How to run ANVIL's turnkey bug-hunt — the `anvil hunt` CLI + the MCP `hunt` tool
answers:
  - "how do I run anvil hunt from the command line"
  - "how do I fuzz a downstream tool with the anvil CLI"
  - "what does anvil hunt print"
  - "what flags does anvil hunt take"
  - "how do I get a reproducer bundle from anvil hunt"
  - "how do I make anvil hunt write reproducer bundles to a directory"
  - "how do I drive the bug-hunt loop without writing my own script"
  - "how do I run a downstream-tool bug hunt over MCP vs the CLI"
  - "what does a clean anvil hunt sweep look like"
  - "how do I confirm the anvil hunt loop works end to end"
date: 2026-06-17
status: current
tags: [bug-hunt, cli, mcp, downstream, reproducer, usability, north-star, turnkey]
evidence: src/main.rs (the `hunt` subcommand — `Commands::Hunt(HuntCommand)`, `run_hunt_command`, `build_hunt_request`); src/mcp/mod.rs (the controlled `hunt` MCP tool — `run_hunt`, `cache_hunt_failures`); src/hunt/mod.rs (`hunt::run` — the shared loop both shim over); tests/hunt_e2e.rs (the real-tool e2e gate); USER_GUIDE.md ("`anvil hunt` (turnkey CLI bug-hunt)"); book/src/agent-mcp.md ("The bug-hunting loop, end to end" → "One command: the `hunt` loop"); docs/decisions/0018-bug-hunt-orchestration-loop.md
reverify: 'cargo test --test hunt_e2e -- --ignored   (tool-gated: with Verilator on $PATH, asserts a clean real-tool sweep + a byte-identical reproducer recipe; tool-less ⇒ skips green)'
---

# `BUG-HUNT-ORCHESTRATION` — running the turnkey downstream bug-hunt

ANVIL is directly usable as a **downstream-tool bug-finder**: one turnkey loop
(fuzz a seed sweep → run the vetted tools → detect any reject/warning [and,
opt-in, a cross-simulator trace mismatch] → auto-minimize each failure → emit a
reproducer), surfaced two ways — both thin shims over the **same** `hunt::run`
(decision [`0018`](../decisions/0018-bug-hunt-orchestration-loop.md), API parity
decision [`0017`](../decisions/0017-api-first-everything-mcp-accessible.md)):

- **CLI — `anvil hunt`** (ANVIL's first subcommand): the loop from the shell,
  printing a `HuntReport` JSON to stdout.
  ```
  anvil hunt --seed 1 --seeds 16 --tools verilator,yosys [--yosys-mode <m>]
             [--config knobs.json] [--no-minimize] [--budget 200] [--diff-sim]
             [--out ./hunt-bundles]
  ```
  `--out <dir>` additionally drops a self-contained reproducer **bundle
  directory** per finding (`repro.sv` / `knobs.json` / `introspection.json` /
  `hunt-verdict.json` / `tool-logs/` / a one-command `repro.sh`). Full flag
  reference: the User Guide's *`anvil hunt`* section.
- **MCP — the `hunt` controlled tool**: an agent calls it with the same controls
  and reads each failing reproducer back as an
  `anvil://artifact/<run_id>/{sv,introspection}` resource (the MCP path writes no
  on-disk bundle — the agent never supplies a filesystem path; the cache is
  populated for every finding); the sweep is recorded in `anvil://audit/log`.

**A clean sweep (`n_failures = 0`) is the expected result** — ANVIL output is
valid by construction, so a finding is a candidate **downstream-tool** bug (file
it with the seed + knobs), never an ANVIL bug. The whole sweep is reproducible
from its arguments (seeded, no wall-clock); every tool runs through the hardened
`verilator`/`yosys`/`iverilog` allow-list in an auto-removed sandbox.

The loop **composes** the existing `validate` / `minimize` surfaces — it adds no
detector and no minimizer of its own; the reproducer **recipe** (`repro.sh`
regenerates the `.sv` from `(seed, knobs.json)`, then re-runs the failing tool)
is byte-identical-faithful by the reproducibility contract.

See [[bug-hunt-orchestration-loop]] for the design (why a thin orchestrator, the
bundle format, the detection policy, the sandbox discipline), and
`book/src/agent-mcp.md` for the end-to-end walk-through.
