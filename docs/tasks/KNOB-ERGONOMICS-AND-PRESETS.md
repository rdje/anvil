# KNOB-ERGONOMICS-AND-PRESETS: CLI flags, curated presets, and full API knob control

## Metadata

- Tree ID: `KNOB-ERGONOMICS-AND-PRESETS`
- Status: `active`
- Roadmap lane: `Usability — knob ergonomics + presets + full API steerability (north star, idea 4)`
- Created: `2026-06-17`
- Last updated: `2026-06-17`
- Owner: repo-local workflow

## Goal

Lower the barrier to driving ANVIL's large knob space. Two strands: (1) promote
high-value config-file-only knobs to first-class CLI flags, and add curated
`--profile` **presets** (e.g. `arithmetic-heavy`, `deep-hierarchy`,
`structured-emission-max`, `sv2023-upopts`) so a user gets a rich shape without
hand-authoring config JSON; and (2) — per the API-first mandate — guarantee that
**every** knob is fully settable/steerable via the MCP/config API and that the
full knob catalog + the preset registry are themselves **API-queryable** (today
`--dump-config` projects the effective config; this lane makes the knob catalog,
defaults, validation ranges, and presets first-class queryable facts).

## Non-Goals

- No new generator behaviour from the knobs themselves — this is ergonomics +
  surfacing over the existing knob set (a preset is a named bundle of existing
  knob values). Default DUT output stays byte-identical (a preset is opt-in).
- No removal/renaming of existing config-file knobs (no retirement); CLI flags
  and presets are additive over the serde config.

## Acceptance Criteria

- A `--profile <name>` preset mechanism exists with at least the curated presets
  above; selected high-value knobs gain CLI flags; CLI/flag/preset and config
  JSON resolve to the same `Config` deterministically.
- **API-completeness gate (decision `0017`):** every knob is settable via the
  MCP/config API; the **knob catalog** (name, default, validation range, group)
  and the **preset registry** are queryable via the MCP/introspection API
  (SCHEMA-DERIVED — projected from the `Config` definition + a preset table, the
  single source of truth); a preset is applicable via the MCP `generate`/`analyze`
  tool inputs, not just the CLI. The CLI is a shim over the same surface.
- Reproducible: a given `(seed, profile, knob overrides)` is byte-stable; presets
  do not introduce wall-clock/randomness.
- Documented in `book/src/knobs.md` + `book/src/agent-mcp.md` + USER_GUIDE +
  README; committed through `COMMIT.md`.

## Task Tree

- ID: `KNOB-ERGONOMICS-AND-PRESETS`
  Status: `active`
  Goal: `CLI flags for high-value knobs + a curated --profile preset registry + full MCP/config knob steerability + an API-queryable knob catalog & preset registry.`
  Children: `KNOB-ERGONOMICS-AND-PRESETS.1`

- ID: `KNOB-ERGONOMICS-AND-PRESETS.1`
  Status: `pending`
  Goal: `Design/decision leaf (ADR, no code): audit the current knob surface (config-file-only vs CLI-exposed; the *_emit_prob / hierarchy / identity / aggregate / memory / fsm / sv-version families), pick the high-value knobs to promote to CLI flags + the initial curated preset set (arithmetic-heavy / deep-hierarchy / structured-emission-max / sv2023-upopts), pin the preset registry shape (a named bundle of Config field overrides, the single source of truth), the CLI/MCP/config resolution order + byte-stability contract, and the API-queryable knob-catalog + preset-registry surface (decision 0017). Record as the next decision record + pre-split .2 (impl).`
  Acceptance: `A decision record + a tree/DEVELOPMENT_NOTES entry pinning the promoted knobs, the preset set + registry shape, the resolution order, and the MCP knob-catalog/preset query surface; docs-only; INDEX + this tree + docs/TASK_TREE.md updated.`
  Verification: `pending`
  Commit: `pending`

- ID: `KNOB-ERGONOMICS-AND-PRESETS.2`
  Status: `pending`
  Goal: `Implement the .1 design: the preset registry + the promoted CLI flags + the API-queryable knob catalog/preset endpoints + proofs (byte-stability + resolution-order) + book/USER_GUIDE/README/KM. Default-off / DUT byte-identical (presets opt-in). Pre-split when picked.`
  Acceptance: `pending (set at .1)`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `KNOB-ERGONOMICS-AND-PRESETS.1` | `pending` | Design-first ADR audits the knob surface + pins presets and the API knob-catalog/preset query surface (decision `0017`) before any code; presets unblock cleaner profiles for `BUG-HUNT-ORCHESTRATION`. |

## Decisions

- `2026-06-17`: Registered as an owner-directed usability lane (idea 4). Binds
  decision [`0017`](../decisions/0017-api-first-everything-mcp-accessible.md):
  beyond CLI ergonomics, **every** knob must be MCP-steerable and the knob
  catalog + presets must be API-queryable. Design-first ADR before code.

## Open Questions

- Preset composition: can presets stack / be overridden by explicit knobs, and
  in what order. *(Resolved at `.1` — the byte-stability contract depends on it.)*
- How much of the knob catalog is auto-derived from `Config` (serde + a
  `dump-config` extension) vs. a hand-maintained table. *(Prefer derived;
  decided at `.1`.)*

## Blockers

- None. (Feeds `BUG-HUNT-ORCHESTRATION` profiles; not blocked by it.)

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-06-17` | `KNOB-ERGONOMICS-AND-PRESETS` | `tree registered (docs-only); no code` | `registered` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `KNOB-ERGONOMICS-AND-PRESETS` | `USABILITY-LANE-OWNERSHIP.1 — register 7 owner-directed usability/capability lanes + API-first decision 0017` | Tree registered (not yet started); frontier `.1` (design ADR) pending. |

## Changelog

- `2026-06-17`: Created task tree (registration via `USABILITY-LANE-OWNERSHIP.1`).
