---
id: knob-presets-and-cli-flags
title: ANVIL knob presets (--profile) and the CLI-flag promotion of 16 config-file-only knobs
answers:
  - "how do I set ANVIL's config-file-only knobs from the CLI"
  - "what does the --profile flag do in ANVIL"
  - "what ANVIL presets are there"
  - "does ANVIL have a --profile preset"
  - "how do I turn on ANVIL structured emission from the CLI"
  - "what is the ANVIL knob resolution order"
  - "do explicit ANVIL CLI flags override a --profile preset"
  - "which ANVIL knobs still have no CLI flag"
date: 2026-06-18
status: current
tags: [knobs, presets, profile, cli, usability]
reverify: "anvil --profile structured-emission-max --dump-config  (function/generate-loop/task/cone-function emit knobs all 1.0; --profile nope errors listing the 4 names; explicit --function-emit-prob 0.25 overrides the preset)"
---

`KNOB-ERGONOMICS-AND-PRESETS.2b.1` (decision
[`0021`](../decisions/0021-knob-ergonomics-presets-and-queryable-catalog.md))
made the knob space easier to drive.

**16 previously-config-file-only knobs are now first-class CLI flags**, each the
kebab-case of the field name: `--function-emit-prob`, `--generate-loop-emit-prob`,
`--task-emit-prob`, `--cone-function-emit-prob`, `--soft-union-slice-prob`,
`--width-parameterization-prob`, `--aggregate-prob`, `--aggregate-array-prob`,
`--memory-prob`, `--fsm-prob`, `--multi-clock-prob`, `--cdc-synchronizer-stages`,
plus the four on-only `SetTrue` toggles `--hierarchy-module-dedup`,
`--hierarchy-semantic-module-dedup`, `--hierarchy-sequential-module-dedup`, and
`--bisimulation-flop-merge`. Three knobs stay config-file-only (still settable via
`--config` JSON / MCP `config`): `library_prob`, `use_async_reset`, and
`max_nodes_per_module`.

**`--profile <name>` applies a curated bundle of knob overrides:**
`arithmetic-heavy` (datapath bias), `deep-hierarchy` (bounded recursive hierarchy
with sibling routing + parent-local flops), `structured-emission-max` (all four
emit-projections on), and `sv2023-upopts` (`--sv-version 2023` + the `union soft`
up-opt).

**Resolution order** (lowest → highest precedence): `Config::default()` →
`--config <json>` → `--profile <name>` → explicit CLI flags → `--seed`. So an
**explicit flag always overrides the preset**, and a preset overrides the
`--config`/default base. A given `(seed, profile, explicit overrides)` is
byte-stable; not passing `--profile` (with none of the promoted flags) is
byte-identical to before (default DUT output unchanged). An unknown profile name
errors and lists the valid names.

Full reference: `book/src/knobs.md` ("Knob presets and CLI-flag promotion"). The
API-first mandate this serves is [[api-first-everything-mcp-accessible]].
