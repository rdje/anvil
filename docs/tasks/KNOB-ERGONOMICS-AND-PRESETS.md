# KNOB-ERGONOMICS-AND-PRESETS: CLI flags, curated presets, and full API knob control

## Metadata

- Tree ID: `KNOB-ERGONOMICS-AND-PRESETS`
- Status: `active`
- Roadmap lane: `Usability — knob ergonomics + presets + full API steerability (north star, idea 4)`
- Created: `2026-06-17`
- Last updated: `2026-06-18`
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
  Status: `done`
  Goal: `Design/decision leaf (ADR, no code): audit the current knob surface (config-file-only vs CLI-exposed; the *_emit_prob / hierarchy / identity / aggregate / memory / fsm / sv-version families), pick the high-value knobs to promote to CLI flags + the initial curated preset set (arithmetic-heavy / deep-hierarchy / structured-emission-max / sv2023-upopts), pin the preset registry shape (a named bundle of Config field overrides, the single source of truth), the CLI/MCP/config resolution order + byte-stability contract, and the API-queryable knob-catalog + preset-registry surface (decision 0017). Record as the next decision record + pre-split .2 (impl).`
  Acceptance: `A decision record + a tree/DEVELOPMENT_NOTES entry pinning the promoted knobs, the preset set + registry shape, the resolution order, and the MCP knob-catalog/preset query surface; docs-only; INDEX + this tree + docs/TASK_TREE.md updated.`
  Verification: `done — decision 0021 written (audit: 86 Config fields → 67 CLI-reachable, 19 config-file-only; promote 16, keep 3 config-only; 4 curated presets; declarative-data registry; default→config→profile→explicit→seed resolution; SCHEMA-DERIVED queryable catalog + presets). INDEX + this tree + docs/TASK_TREE.md + DEVELOPMENT_NOTES updated; KM regenerated (54 facts); docs-only / DUT byte-identical.`
  Commit: `KNOB-ERGONOMICS-AND-PRESETS.1 — design ADR (decision 0021): CLI-flag promotion + --profile preset registry + queryable knob catalog`

- ID: `KNOB-ERGONOMICS-AND-PRESETS.2`
  Status: `pending`
  Goal: `Implement the .1 design (decision 0021): the declarative preset registry (4 curated presets) + the 16 promoted Option-based CLI flags + the SCHEMA-DERIVED rich knob catalog + anvil://catalog/presets + the MCP profile input + proofs (default byte-identical + explicit-beats-preset resolution + catalog completeness) + book/USER_GUIDE/README/KM. Default-off / DUT byte-identical (presets opt-in). Pre-split into .2a (design-detail) + .2b (impl) per decision 0021.`
  Acceptance: `set at .1 (decision 0021): preset table is the single declarative source of truth; promoted flags are Option<T> (explicit beats preset); knob catalog projects {name,group,type,default,validation,cli_flag,config_only} with a completeness test; presets + catalog API-queryable; (seed,profile,overrides) byte-stable; no --profile ⇒ snapshots untouched.`
  Verification: `pending`
  Commit: `pending`
  Children: `KNOB-ERGONOMICS-AND-PRESETS.2a`, `.2b.1`, `.2b.2`, `.2b.3`

- ID: `KNOB-ERGONOMICS-AND-PRESETS.2a`
  Status: `done`
  Goal: `Impl design-detail: pin the exact Rust shapes — reuse Overrides as the preset carrier (+ Default/Serialize derives), the presets() registry + Preset struct, the shared resolve_config(base, profile, overrides, seed) used by both CLI and MCP, the config_from_args profile arg, and the SCHEMA-DERIVED knob_catalog (KnobInfo + metadata table + completeness test mirroring downstream::adapter_catalog()). Re-split .2b.`
  Acceptance: `A DEVELOPMENT_NOTES design-detail entry pinning the carrier type, registry shape, resolver signature, MCP profile threading, and the knob-catalog completeness-test contract; .2b re-split; docs-only.`
  Verification: `done — DEVELOPMENT_NOTES "Knob ergonomics impl design-detail" entry; .2b re-split into .2b.1/.2b.2/.2b.3; docs-only / DUT byte-identical.`
  Commit: `KNOB-ERGONOMICS-AND-PRESETS.2a — impl design-detail (Overrides-as-preset-carrier + shared resolver + knob-catalog contract)`

- ID: `KNOB-ERGONOMICS-AND-PRESETS.2b.1`
  Status: `done`
  Goal: `Code: the presets() registry (4 curated presets) + Preset struct + preset_overrides/preset_names; the shared resolve_config(base, profile, overrides, seed); the --profile CLI flag + the 16 promoted Option<T> CLI flags threaded through Overrides + apply_cli_overrides + cli_overrides; ConfigError::UnknownProfile. (Serialize on Overrides deferred to .2b.2 where the presets catalog needs it.) Proofs: default (no --profile) byte-identical; explicit flag beats preset; unknown profile errors.`
  Acceptance: `The 16 knobs are CLI-settable; --profile applies a preset; explicit flags beat the preset; default (no --profile) DUT output byte-identical (snapshots 6/6 untouched); unknown profile errors with the available names. cargo check/test/clippy/fmt green.`
  Verification: `done — config.rs presets()/resolve_config + ConfigError::UnknownProfile; main.rs --profile + 16 Option flags + cli_overrides mapping (SetTrue bools via .then_some(true)) + resolve_config wired into the DUT path. lib 553→558 (+5 preset/resolver tests), anvil bin 14 (+2 CLI-wiring tests), snapshots 6/6 byte-identical, clippy -D warnings clean, fmt clean, full cargo test green; --dump-config smoke confirms presets/explicit-beats-preset/unknown-error/bool-toggle. User-facing docs deferred to .2b.3.`
  Commit: `KNOB-ERGONOMICS-AND-PRESETS.2b.1 — promote 16 knobs to CLI flags + --profile preset registry + shared resolve_config`

- ID: `KNOB-ERGONOMICS-AND-PRESETS.2b.2`
  Status: `pending`
  Goal: `Code: the SCHEMA-DERIVED knob_catalog() (KnobInfo + metadata table) + the completeness test (catalog names == Config::default() serde keys) + the anvil://catalog/knob-schema resource + the anvil://catalog/presets resource + the MCP generate/introspect/analyze profile input. Pure projection; raw anvil://catalog/knobs kept.`
  Acceptance: `pending (set at pick)`
  Verification: `pending`
  Commit: `pending`

- ID: `KNOB-ERGONOMICS-AND-PRESETS.2b.3`
  Status: `pending`
  Goal: `Docs closeout: book/src/knobs.md (presets + CLI-flag promotion + catalog) + book/src/agent-mcp.md (profile input + catalog/presets resources) + USER_GUIDE + README CLI-truth + a Knowledge Map how-to card. Docs-only.`
  Acceptance: `pending (set at pick)`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `KNOB-ERGONOMICS-AND-PRESETS.1` | `done` | Design-first ADR (decision `0021`) audited the knob surface + pinned the promotion set, the 4 presets, the declarative registry shape, the resolution order, and the API-queryable knob-catalog/preset surface (decision `0017`). |
| 2 | `KNOB-ERGONOMICS-AND-PRESETS.2a` | `done` | Impl design-detail: `Overrides` reused as the preset carrier, the `presets()` registry, the shared `resolve_config` (CLI = shim over the same resolver), and the completeness-gated knob catalog; `.2b` re-split. |
| 3 | `KNOB-ERGONOMICS-AND-PRESETS.2b.1` | `done` | Code: preset registry + shared `resolve_config` + `--profile` + 16 promoted `Option` CLI flags + `ConfigError::UnknownProfile`; default byte-identical (snapshots 6/6), explicit-beats-preset + unknown-profile proofs; full `cargo test` green. |
| 4 | `KNOB-ERGONOMICS-AND-PRESETS.2b.2` | `pending` | Next: the SCHEMA-DERIVED queryable knob catalog + completeness test + `anvil://catalog/knob-schema`/`presets` resources + the MCP `profile` input (+ `Serialize` on `Overrides`). |

## Decisions

- `2026-06-17`: Registered as an owner-directed usability lane (idea 4). Binds
  decision [`0017`](../decisions/0017-api-first-everything-mcp-accessible.md):
  beyond CLI ergonomics, **every** knob must be MCP-steerable and the knob
  catalog + presets must be API-queryable. Design-first ADR before code.
- `2026-06-18` (`.1`): Design ADR landed as decision
  [`0021`](../decisions/0021-knob-ergonomics-presets-and-queryable-catalog.md).
  Audit (verified): 86 `Config` fields → 66 CLI via `Overrides` + `seed`
  special-cased = 67 CLI-reachable; **19 genuinely config-file-only**; MCP already
  takes a full `Config` (every knob steerable); the catalog is a raw
  `Config::default()` dump (no metadata, no presets). Decisions: (1) promote 16 of
  19 to `Option<T>` CLI flags (12 capability + 4 identity/dedup), keep 3
  config-only (`library_prob`, `use_async_reset`, `max_nodes_per_module`); (2) a
  declarative `--profile` registry (`arithmetic-heavy` / `deep-hierarchy` /
  `structured-emission-max` / `sv2023-upopts`) as enumerable data, not closures;
  (3) resolution `default → --config → --profile → explicit knobs → --seed`
  (explicit beats preset; one profile in the first cut), byte-stable, default-off
  byte-identical; (4) a SCHEMA-DERIVED rich knob catalog (`Config::default()`
  serde + a metadata table guarded by a completeness test) + an
  `anvil://catalog/presets` resource + a `profile` MCP input, all additive (the
  raw `anvil://catalog/knobs` resource is kept, nothing retired).

## Open Questions

- Preset composition: can presets stack / be overridden by explicit knobs, and
  in what order. *(Resolved at `.1` / decision `0021`: explicit knobs always
  override a preset; first cut = a single `--profile`; stacking is a deferred
  additive `.N`.)*
- How much of the knob catalog is auto-derived from `Config` (serde + a
  `dump-config` extension) vs. a hand-maintained table. *(Resolved at `.1` /
  decision `0021`: names + defaults derived from `Config::default()` serde;
  group + validation range + cli-flag presence from a metadata table guarded by a
  completeness test — SCHEMA-DERIVED + drift-proof.)*

## Blockers

- None. (Feeds `BUG-HUNT-ORCHESTRATION` profiles; not blocked by it.)

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-06-17` | `KNOB-ERGONOMICS-AND-PRESETS` | `tree registered (docs-only); no code` | `registered` |
| `2026-06-18` | `KNOB-ERGONOMICS-AND-PRESETS.1` | `decision 0021 written; knob-surface audit verified programmatically (86 fields → 67 CLI-reachable, 19 config-only); INDEX + tree + TASK_TREE + DEVELOPMENT_NOTES updated; KM regen+check green; mem-arch check green; docs-only / DUT byte-identical` | `done` |
| `2026-06-18` | `KNOB-ERGONOMICS-AND-PRESETS.2a` | `impl design-detail in DEVELOPMENT_NOTES (Overrides-as-preset-carrier, presets() registry, shared resolve_config, knob_catalog completeness test); .2b re-split into .2b.1/.2b.2/.2b.3; docs-only / DUT byte-identical` | `done` |
| `2026-06-18` | `KNOB-ERGONOMICS-AND-PRESETS.2b.1` | `lib 553→558 (+5 preset/resolver tests); anvil bin 14 (+2 CLI-wiring tests); snapshots 6/6 byte-identical; clippy -D warnings clean; fmt clean; full cargo test green; --dump-config smoke (presets/explicit-beats-preset/unknown-error/bool-toggle)` | `done` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `KNOB-ERGONOMICS-AND-PRESETS` | `USABILITY-LANE-OWNERSHIP.1 — register 7 owner-directed usability/capability lanes + API-first decision 0017` | Tree registered (not yet started); frontier `.1` (design ADR) pending. |
| `KNOB-ERGONOMICS-AND-PRESETS.1` | `KNOB-ERGONOMICS-AND-PRESETS.1 — design ADR (decision 0021): CLI-flag promotion + --profile preset registry + queryable knob catalog` | Design-only; pins promotion set (16/19), 4 presets, declarative registry, resolution order, SCHEMA-DERIVED queryable catalog; pre-splits `.2` into `.2a`/`.2b`. |
| `KNOB-ERGONOMICS-AND-PRESETS.2a` | `KNOB-ERGONOMICS-AND-PRESETS.2a — impl design-detail (Overrides-as-preset-carrier + shared resolver + knob-catalog contract)` | Docs-only; pins the Rust shapes + re-splits `.2b` into `.2b.1`/`.2b.2`/`.2b.3`. |
| `KNOB-ERGONOMICS-AND-PRESETS.2b.1` | `KNOB-ERGONOMICS-AND-PRESETS.2b.1 — promote 16 knobs to CLI flags + --profile preset registry + shared resolve_config` | First code slice; default DUT byte-identical (snapshots 6/6). User-facing docs deferred to `.2b.3`. |

## Changelog

- `2026-06-17`: Created task tree (registration via `USABILITY-LANE-OWNERSHIP.1`).
- `2026-06-18`: `.1` design ADR (decision `0021`); `.2a` impl design-detail (re-split
  `.2b`); `.2b.1` first code slice — 16 knobs promoted to CLI flags + `--profile`
  preset registry + shared `resolve_config`, default DUT byte-identical. Frontier
  advances to `.2b.2` (queryable catalog + MCP `profile` input).
