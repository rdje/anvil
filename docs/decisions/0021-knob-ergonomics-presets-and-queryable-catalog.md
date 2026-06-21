---
id: knob-ergonomics-presets-and-queryable-catalog
title: Knob ergonomics — promote high-value config-only knobs to CLI flags, add a declarative --profile preset registry, and expose an API-queryable knob catalog + preset registry (SCHEMA-DERIVED, decision 0017)
answers:
  - "how do I set ANVIL's config-file-only knobs from the CLI"
  - "which ANVIL knobs have no CLI flag"
  - "does ANVIL have presets or profiles"
  - "what does --profile do in ANVIL"
  - "how do I query ANVIL's knob catalog over MCP"
  - "can I list ANVIL's presets over the API"
  - "what is the CLI vs config-file vs profile vs default resolution order"
  - "do explicit knobs override a preset"
  - "is the ANVIL knob catalog API-queryable"
  - "how are ANVIL presets defined"
  - "are ANVIL presets byte-stable and reproducible"
  - "how many ANVIL knobs are config-file-only"
date: 2026-06-18
status: accepted
tags: [usability, knobs, presets, profile, cli, mcp, api, catalog, schema-derived, byte-identical, reproducible, north-star]
evidence: docs/decisions/0021-knob-ergonomics-presets-and-queryable-catalog.md; docs/decisions/0017-api-first-everything-mcp-accessible.md; src/config.rs; src/main.rs; src/mcp/mod.rs; docs/tasks/KNOB-ERGONOMICS-AND-PRESETS.md; book/src/knobs.md
---

# 0021 - Knob ergonomics: CLI-flag promotion, a declarative `--profile` preset registry, and an API-queryable knob catalog + preset registry

- Date: 2026-06-18
- Status: accepted
- Tree: `KNOB-ERGONOMICS-AND-PRESETS.1` (the design/decision leaf; no code).
- Binds / extends: decision [`0017`](0017-api-first-everything-mcp-accessible.md)
  (the API-completeness gate — controllable/invocable/queryable/documented).
  Builds on `feedback_api_for_agents_not_humans` and the north star
  (`project_anvil_north_star`): a richer, easier-to-drive knob surface multiplies
  the downstream-bug-hunting loop.

## Context

ANVIL's knob space is large and grew feature-by-feature; the ergonomics did not.
Two friction points motivate the `KNOB-ERGONOMICS-AND-PRESETS` lane (owner idea 4):

1. **Config-file-only knobs.** A measurable slice of the knob surface is reachable
   only through `--config <json>`, not as a CLI flag — so a user who wants one
   capability on must hand-author config JSON.
2. **No presets and no rich catalog.** There is no curated bundle (a "profile")
   that turns on a coherent shape in one word, and the only machine-readable knob
   inventory is a raw `Config::default()` serialization (field → value), carrying
   no per-knob metadata (group, validation range, whether a CLI flag exists) and
   no preset listing.

This leaf is **design only**: it audits the surface and pins the decisions; the
implementation is `.2`.

### Audit of the current surface (verified `2026-06-18`)

Source of truth: `src/config.rs` (`struct Config`, `struct Overrides`,
`apply_cli_overrides`, `validate`), `src/main.rs` (the `Cli`/`cli_overrides`
mapping + the `--seed` stamp), `src/mcp/mod.rs` (`config_from_args`, the catalog
resources).

- **`Config` has 86 fields.** **66** are CLI-overridable through the `Overrides`
  struct + `apply_cli_overrides`; **`seed`** is CLI-settable too (`--seed`,
  stamped directly, not via `Overrides`) ⇒ **67 fields are CLI-reachable**.
- **19 knobs are genuinely config-file-only** (no CLI flag — settable only via
  `--config` JSON or the MCP `config` arg). Grouped by character:
  - **Capability "feature" knobs** (turning one on unlocks a visible
    synthesizable surface): `width_parameterization_prob`, `aggregate_prob`,
    `aggregate_array_prob`, `soft_union_slice_prob`, `function_emit_prob`,
    `generate_loop_emit_prob`, `task_emit_prob`, `cone_function_emit_prob`,
    `memory_prob`, `fsm_prob`, `multi_clock_prob`, `cdc_synchronizer_stages`.
    *(12)*
  - **Identity / dedup proof passes** (opt-in, expert, off-by-default bools):
    `hierarchy_module_dedup`, `hierarchy_semantic_module_dedup`,
    `hierarchy_sequential_module_dedup`, `bisimulation_flop_merge`. *(4)*
  - **Internal-tuning / guard-rail knobs**: `library_prob` (hierarchy
    library-reuse probability), `use_async_reset` (reset-style emit toggle),
    `max_nodes_per_module` (per-module construction budget; documented alongside
    the RAM governor). *(3)*
- **MCP steerability today is already complete for knobs.** The MCP
  `generate`/`introspect`/`dump_config` tools accept a **full effective `Config`
  JSON** via the `config` arg → `config_from_args` → `cfg.validate()`. So every
  knob — including the 19 above — is already settable over the API. The gap
  decision `0017` still leaves open is **queryability** (a rich catalog) and
  **ergonomics** (presets), not control.
- **The catalog today is a raw dump.** `--dump-config` and the
  `anvil://catalog/knobs` resource both emit a `Config::default()` /
  effective-config serialization — **field names + values only**. No group, no
  validation range, no "has a CLI flag" marker, and no preset listing. There is
  **no preset/profile mechanism** anywhere; a "profile" today is an ad-hoc
  `--config foo.json`.

## Decision

Four pinned decisions, all default-off / byte-identical, all API-first.

### 1. Promote 16 of the 19 config-only knobs to first-class CLI flags

Promote the **12 capability knobs** and the **4 identity/dedup bools** to
dedicated CLI flags (e.g. `--function-emit-prob`, `--generate-loop-emit-prob`,
`--task-emit-prob`, `--cone-function-emit-prob`, `--soft-union-slice-prob`,
`--width-parameterization-prob`, `--aggregate-prob`, `--aggregate-array-prob`,
`--memory-prob`, `--fsm-prob`, `--multi-clock-prob`, `--cdc-synchronizer-stages`;
`--hierarchy-module-dedup`, `--hierarchy-semantic-module-dedup`,
`--hierarchy-sequential-module-dedup`, `--bisimulation-flop-merge`). These are all
user-facing capability controls; uniform CLI reachability matches decision
`0017`'s "the CLI is a shim over the API."

**Keep config-file-only (3, with rationale, not retired):** `library_prob`
(internal hierarchy tuning), `use_async_reset` (niche structural toggle), and
`max_nodes_per_module` (a safety budget that pairs with the `--max-rss-mb` /
`--ram-abort-pct` governor, not a generation feature). Promoting them is a clean
additive future `.N` if demand appears — nothing is retired, and all three stay
fully MCP-settable via `config`.

**Hard impl rule (carried to `.2`): promoted flags MUST be `Option<T>` overrides**
(absent ⇒ "not set"), threaded through the `Overrides` struct exactly like the
existing flags — **never** a clap-defaulted concrete value. A defaulted value
would silently clobber a preset (Decision 3) for any knob the user did not pass;
the Option discipline is what makes "explicit beats preset" hold (Decision 3).

### 2. A curated, declarative `--profile <name>` preset registry

A **preset** is a named, documented bundle of explicit `Config` field overrides —
a *partial* config, not a whole one. Initial curated set (each a coherent,
downstream-clean shape over **existing** knobs only — no new generator behaviour):

- **`arithmetic-heavy`** — datapath bias: raise `gate_arith_weight`,
  `coefficient_prob`, `max_gate_arity`, `const_comparand_prob`; lower
  `gate_bitwise_weight` relatively.
- **`deep-hierarchy`** — bounded recursive hierarchy with routing: set
  `min_hierarchy_depth`/`max_hierarchy_depth` (e.g. `2:3`),
  `min/max_child_instances_per_module` (e.g. `2:3`), and enable
  `hierarchy_sibling_route_prob`, `hierarchy_child_input_cone_prob`,
  `hierarchy_parent_flop_prob`.
- **`structured-emission-max`** — turn on the emit-projection family:
  `function_emit_prob`, `generate_loop_emit_prob`, `task_emit_prob`,
  `cone_function_emit_prob` (the five emit-projections are mutually exclusive
  per gate, so all-on is safe and behaviour-preserving).
- **`sv2023-upopts`** — `sv_version = 2023` + `soft_union_slice_prob > 0` (the
  IEEE 1800-2023 `union soft` up-opt; Verilator-`--language 1800-2023`-clean,
  Yosys/Icarus no-op per decision `0010`).

**Registry shape — declarative data, the single source of truth.** One static,
compile-time table; each entry carries `{ name, description, overrides }` where
`overrides` is an **enumerable** set of `(Config field, value)` assignments (a
`PartialConfig` serde struct of `Option` fields, or an equivalent
field→json-value list) — explicitly **not** an opaque `fn(&mut Config)` closure,
**precisely so the preset's overrides are themselves API-queryable** (Decision 4).
Both the CLI `--profile`, the MCP `profile` input, and the
`anvil://catalog/presets` query read this one table.

### 3. CLI / MCP / config-file / default resolution order + byte-stability

Resolve to one canonical `Config` by this total precedence (lowest → highest):

1. `Config::default()` — the base.
2. `--config <json>` file (the existing full/merged base).
3. `--profile <name>` preset overrides — applied on top of the config-file base.
4. **Explicit** CLI flags / explicit MCP knob overrides — **always win** (the
   `Option`-based `Overrides` of Decision 1 carry only user-passed values, so
   "explicit beats preset" is a set operation, independent of CLI arg ordering).
5. `--seed` — stamped last (orthogonal).

Then `validate()`. This resolves the tree's open questions:

- *Composition / stacking:* first cut = **one `--profile`** (a single preset);
  repeated/stacked profiles are a deferred additive `.N` extension, kept out of
  the first cut to keep the byte-stability contract trivially total.
- *Explicit-vs-preset:* explicit user knobs always override the preset.

**Byte-stability contract:** a given `(seed, profile, explicit overrides)`
resolves deterministically to one `Config` and therefore one byte-identical
output; presets add no wall-clock / no `thread_rng`. **No `--profile` ⇒ default
DUT output stays byte-identical** (`tests/snapshots.rs` untouched). The identical
resolution must hold whether driven via the CLI or the MCP `profile` input (the
CLI is a shim over the same resolver).

### 4. An API-queryable, SCHEMA-DERIVED knob catalog + preset registry

Add a **rich knob catalog**: per `Config` field, project
`{ name, group, type, default, validation (min/max or enum variants),
cli_flag: Option<String>, config_only: bool }`.

- **Derivation:** field names + defaults are **derived** from `Config::default()`
  serde; group + validation range + cli-flag presence come from a small
  hand-maintained **metadata table keyed by field name**, guarded by a
  **completeness test** asserting exactly one catalog entry per `Config` field and
  no orphans (the KM's derive-and-diff anti-drift pattern). This keeps the catalog
  SCHEMA-DERIVED (a projection of the `Config` definition + a metadata table), not
  a recomputed second source of truth — honoring decision `0017`'s hard boundary.
- **Surfaces (additive, no retirement):** keep the existing
  `anvil://catalog/knobs` raw-default resource as-is; add the rich catalog as a
  new surface (a `anvil://catalog/knob-schema` resource and/or a pure
  `knob_catalog` MCP query) plus a new `anvil://catalog/presets` resource listing
  each preset `{ name, description, overrides }`. Add a `profile` input arg to the
  MCP `generate`/`introspect`/`analyze` tools (explicit `config` still wins). If
  `--introspect` surfaces the resolved profile name, bump the introspection schema
  additively-MINOR at `.2`.

## Pre-split of `.2` (implementation)

- **`.2a`** — design-detail: the exact `PartialConfig`/preset carrier type, the
  metadata-table shape + completeness-test contract, the resolver signature, and
  the `Option`-override threading for the promoted flags.
- **`.2b`** — impl: the preset table + the 16 promoted (`Option`) CLI flags + the
  resolver + the rich knob catalog + `anvil://catalog/presets` + the MCP `profile`
  input + proofs (default byte-identical; explicit-beats-preset; catalog
  completeness) + `book/src/knobs.md` / `book/src/agent-mcp.md` / USER_GUIDE /
  README / KM. Split further at pick if warranted.

## Decisive test applied (decision 0017)

*"Could an agent, with only the MCP API and no shell access, drive this?"* — Yes:
set any knob via `config`, apply a preset via `profile`, and read back the full
knob catalog + the preset registry (with each preset's concrete overrides) via the
catalog resources/query. Every added query is a pure projection of the `Config`
definition + the static preset/metadata tables (the SCHEMA-DERIVED test).

## Rejected alternatives

- **Promote all 19 config-only knobs to CLI flags.** Rejected: `library_prob`,
  `use_async_reset`, and `max_nodes_per_module` are internal-tuning / guard-rail
  knobs, not capability features; flag-promoting them adds CLI surface without
  ergonomic payoff. Kept config-only (still MCP-settable), additive later.
- **Opaque `fn(&mut Config)` preset closures.** Rejected: a closure's overrides
  cannot be enumerated, so the preset registry would not be API-queryable
  (violates decision `0017`). Presets must be declarative data.
- **Upgrade `anvil://catalog/knobs` in place to the rich schema.** Rejected for
  the first cut: changing an existing resource's content could break agents that
  parse the raw-default form. Add the rich catalog as a new surface; nothing
  retired.
- **Derive validation ranges automatically from `validate()`.** Rejected: the
  ranges live as imperative checks, not data; a metadata table + completeness test
  is the honest, drift-proof source. (Auto-derivation could be a later refinement
  if `validate` is ever refactored to data-driven bounds.)
- **clap-defaulted promoted flags (concrete defaults).** Rejected: a concrete
  default clobbers presets for un-passed knobs; promoted flags must be `Option`.
- **A free-form query language / a behavioural query over the catalog.** Out of
  scope by decision `0017` / `0011` / `0004` — the catalog is a vetted, named
  projection, never a shadow simulator.

## Consequences

- `.2` lands the preset registry, the 16 promoted `Option` CLI flags, the
  resolver, the rich SCHEMA-DERIVED knob catalog, the `anvil://catalog/presets`
  resource, and the MCP `profile` input — default-off / DUT byte-identical.
- Presets unblock cleaner, named shapes for `BUG-HUNT-ORCHESTRATION` /
  `CI-PACKAGING-DISTRIBUTION` (a maintainer fuzzes with `--profile sv2023-upopts`
  instead of hand-built JSON).
- The completeness test makes a new `Config` field's catalog entry mandatory, so
  the catalog cannot silently drift as the knob surface grows.
- The structure-first / no-shadow-simulator ceiling is unchanged; this lane is
  ergonomics + projection only.

## Links

- Owning tree: `docs/tasks/KNOB-ERGONOMICS-AND-PRESETS.md` (leaf `.1`).
- Parent decision: `0017` (API-first / API-completeness gate).
- Memory: `feedback_api_for_agents_not_humans`, `project_anvil_north_star`,
  `feedback_never_retire_strategies`, `feedback_rules_first_generation`.
