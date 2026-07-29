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
  - "how do I query the ANVIL knob catalog over MCP"
  - "can I discover ANVIL presets over the API"
  - "what is anvil://catalog/knob-schema"
  - "what is anvil://catalog/presets"
  - "can I apply an ANVIL preset over MCP"
date: 2026-06-18
status: current
tags: [knobs, presets, profile, cli, mcp, api, catalog, usability]
reverify: "anvil --profile structured-emission-max --dump-config  (all EIGHT non-version-gated *_emit_prob knobs = 0.25 and comb/case/casez_mux_prob = 0.35, NOT 1.0 — decision 0032; --profile nope errors listing the 4 names; explicit --function-emit-prob 1.0 overrides the preset)"
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
with sibling routing + parent-local flops), `structured-emission-max` (all
**eight** non-version-gated emit-projections at `0.25` + the three selector knobs
at `0.35`), and `sv2023-upopts` (`--sv-version 2023` + the `union soft` up-opt).

**`structured-emission-max` means maximal surface *diversity*, not every knob at
`1.0`** (`EMIT-SURFACE-INTERACTION-GATE.2`, decision
[`0032`](../decisions/0032-emit-surface-interaction-gate.md)). The nine
emit-projections are mutually exclusive per gate and run in a fixed order, so
under saturation **probability is priority, not intensity**: at `1.0`
`function_emit` runs second and claims every admissible gate, and the later
passes emit nothing. Measured — the pre-`0032` preset set four surfaces to `1.0`
and emitted **one** (796 combinational functions, everything else exactly `0`).
At `0.25` a single module carries all eight surfaces, downstream-clean across
Verilator + both Yosys modes + Icarus. A tenth surface joins at the same shared
value; a `structured_emission`-group drift test in `src/config.rs` fails if it
does not.

**Resolution order** (lowest → highest precedence): `Config::default()` →
`--config <json>` → `--profile <name>` → explicit CLI flags → `--seed`. So an
**explicit flag always overrides the preset**, and a preset overrides the
`--config`/default base. A given `(seed, profile, explicit overrides)` is
byte-stable; not passing `--profile` (with none of the promoted flags) is
byte-identical to before (default DUT output unchanged). An unknown profile name
errors and lists the valid names.

**Over MCP (decision `0017` queryability), all of this is API-discoverable +
steerable** (`KNOB-ERGONOMICS-AND-PRESETS.2b.2a`/`.2b.2b`): the `profile` tool
argument on `generate`/`introspect`/`analyze`/`dump_config` applies a preset; the
`anvil://catalog/presets` resource lists each preset's name/description/overrides;
and `anvil://catalog/knob-schema` is the per-knob catalog
(`name`/`group`/`ty`/`default`/`validation`/`cli_flag`/`config_only`, one row per
`Config` field — a SCHEMA-DERIVED projection, the richer companion to the bare
`anvil://catalog/knobs` default-`Config` dump, which is kept unchanged).

Full reference: `book/src/knobs.md` ("Knob presets and CLI-flag promotion") +
`book/src/agent-mcp.md`. The API-first mandate this serves is
[[api-first-everything-mcp-accessible]].
