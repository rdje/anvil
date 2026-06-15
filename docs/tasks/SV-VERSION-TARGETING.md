# SV-VERSION-TARGETING: target a chosen IEEE 1800 standard valid-by-construction

## Metadata

- Tree ID: `SV-VERSION-TARGETING`
- Status: `active`
- Roadmap lane: `Capability / breadth — version-targeted synthesizable RTL (ROADMAP steering gaps 1 + 3)`
- Created: `2026-06-15`
- Last updated: `2026-06-15`
- Owner: repo-local workflow
- Note: opened `2026-06-15` by owner roadmap steering as the recommended
  highest-leverage capability lane (over the two registered-`proposed` siblings
  `STRUCTURED-EMISSION-EXPANSION` and `SEMANTIC-INTROSPECTION-EXPANSION`).

## Goal

Give ANVIL a `--sv-version <2012|2017|2023>` capability gate
(`Config::sv_version`) that makes the generator/emitter target a chosen IEEE
1800 SystemVerilog standard **valid-by-construction**, serving the north star
(`project_anvil_north_star`): expose version-specific downstream-tool bugs via
legal, standard-valid, downstream-acceptance-quality output. Two effects, both
rules-first: **down-gating** (never emit a construct newer than the target — a
standard-validity guarantee) and **up-opting** (deliberately emit a higher
standard's distinctive synthesizable constructs, each proven downstream-clean in
the matching tool standard mode). Default reproduces today's output
byte-identical.

## Non-Goals

- No generate-then-filter: the version is a construction-time capability bound,
  not a post-hoc reject (`feedback_rules_first_generation`).
- No default output change: the default `--sv-version` is byte-identical to
  today (`tests/snapshots.rs` untouched); the gate is opt-in
  (`feedback_never_retire_strategies`).
- No aspirational constructs: an up-opted construct lands only once proven
  accepted by a downstream tool in its matching standard mode.
- Not classic Verilog / SV-2005: ANVIL emits SystemVerilog; the floor is the
  2012-era synthesizable SV subset.

## Acceptance Criteria

- A `Config::sv_version` enum + `--sv-version` CLI flag + `--dump-config` /
  introspection field; the default value is byte-identical to current emission.
- The emitter (and any version-relevant generator choice) honours the target as
  a read-only capability bound; down-gating is a guarantee.
- A per-version downstream acceptance axis proves the targeted corpus is accepted
  in the matching tool standard mode, with retained seed + `sv_version` + knobs
  counterexamples.
- Each up-opted version-distinctive construct is design-first and proven
  downstream-clean before default-on for that version.
- Live docs (book chapter, USER_GUIDE, README CLI truth, ROADMAP, knobs) updated
  where the surface changes; a Knowledge Map fact per durable capability/boundary.
- Every leaf committed through `COMMIT.md` with its leaf id.

## Task Tree

- ID: `SV-VERSION-TARGETING`
  Status: `active`
  Goal: `Version-targeted valid-by-construction SystemVerilog emission.`
  Children: `SV-VERSION-TARGETING.1`, `SV-VERSION-TARGETING.2`, `SV-VERSION-TARGETING.3`

- ID: `SV-VERSION-TARGETING.1`
  Status: `done`
  Goal: `Design/decision leaf: fix the gate semantics (down-gating guarantee + up-opting), the default (byte-identical) value, the valid-by-construction discipline, the per-version downstream acceptance gate, the first-increment scope, and rejected alternatives — before any code.`
  Acceptance: `A decision record naming the gate, its two construction-time effects, its byte-identical default, its downstream proof, its first-increment scope, and its rejected alternatives; no source change; docs/workflow self-checks clean.`
  Result: `Decision 0009 — opt-in --sv-version <2012|2017|2023> gate (Config::sv_version). Down-gating = never emit a construct newer than the target (standard-validity guarantee); up-opting = deliberately emit a higher standard's distinctive synthesizable constructs, each proven downstream-clean in the matching tool mode. Default = the floor value byte-identical to today's emission (tests/snapshots untouched). Rules-first (construction-time bound, no generate-then-filter). Per-version downstream acceptance axis (verilator --language 1800-20xx, yosys -sv, iverilog -g2012 gated/no-op beyond its newest generation). First increment (.2) = plumbing + down-gating + per-version acceptance over the existing subset; first up-opted construct = .3 (design-first). Tree split into .1 (done) + .2 (impl) + .3 (future up-opt).`
  Verification: `done`
  Commit: `done`

- ID: `SV-VERSION-TARGETING.2`
  Status: `proposed`
  Goal: `(Future) Implement the plumbing + down-gating + per-version acceptance axis over the existing subset: Config::sv_version enum + --sv-version CLI + dump-config/introspection field; thread the target into the emitter as a capability bound; the per-version downstream acceptance column; floor target byte-identical (snapshots 6/6); 2017/2023 targets downstream-clean over the current subset in the matching tool standard mode.`
  Acceptance: `cargo fmt/check/clippy clean; default --sv-version byte-identical (snapshots 6/6); --dump-config + introspection expose the field (schema MINOR bump); per-version acceptance proven; book/USER_GUIDE/README/ROADMAP/knobs + KM updated; committed through COMMIT.md with the leaf id. Split into .2a design-detail + .2b impl if broad.`
  Verification: `pending`
  Commit: `pending`

- ID: `SV-VERSION-TARGETING.3`
  Status: `proposed`
  Goal: `(Future) The first version-distinctive up-opted synthesizable construct (a construct introduced by 2017 or 2023 that a downstream tool accepts in its version mode), design-first, gated on sv_version >= that_standard, proven downstream-clean in the matching tool standard mode.`
  Acceptance: `Design leaf first (which construct, why synthesizable + tool-accepted, the gate); then impl with a downstream-clean bank in the matching tool mode; default-off for lower versions / byte-identical; book + KM updated.`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `SV-VERSION-TARGETING.2` | `proposed` | Now eligible — `.1` design landed (decision `0009`). Implement the gate plumbing + down-gating + per-version acceptance over the existing subset, default byte-identical. |
| — | `SV-VERSION-TARGETING.1` | `done` | Landed decision `0009` — gate semantics, byte-identical default, valid-by-construction discipline, per-version downstream proof, first-increment scope, rejected alternatives. No source change. |

## Decisions

- `2026-06-15` (`.1`, decision [`0009`](../decisions/0009-sv-version-targeting.md)):
  Opened the lane `active` by owner roadmap steering. First leaf designs the
  `--sv-version <2012|2017|2023>` gate: down-gating guarantee + up-opting stress,
  byte-identical default, rules-first construction-time bound, per-version
  downstream acceptance proof. Rejected: generate-then-filter, single-newest no
  selector, unproven up-opted constructs, non-byte-identical default, classic
  Verilog targets. Tree split into `.2` (plumbing impl) + `.3` (first up-opt).

## Open Questions

- `.2` finalizes the enum spelling + the exact byte-identical floor default value,
  the introspection field name, the Verilator language-selector spelling the
  installed tool accepts, and whether the per-version axis is a new `tool_matrix`
  gate or a `--sv-version`-parameterized run of the existing columns.

## Blockers

- None.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-06-15` | `SV-VERSION-TARGETING.1` | Design/decision leaf, no source change (grounded in `src/emit/sv.rs` current subset + `src/downstream/mod.rs` fixed tool standards + confirming no existing `sv_version` knob). Decision `0009` with KM `answers:`; `KNOWLEDGE_MAP.md` regenerated; `bash scripts/check_memory_architecture.sh` + `bash knowledge-map/scripts/check_knowledge_map.sh` clean. | `done` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `SV-VERSION-TARGETING.1` | `SV-VERSION-TARGETING.1 — open SV-version lane + decision 0009` | Decision record `0009`; opened the lane + registered the two sibling `proposed` lanes. No source change. |

## Changelog

- `2026-06-15`: Created task tree (owner-directed capability lane), opened
  `active`, landed `.1` (decision `0009`); split into `.2` (plumbing impl) +
  `.3` (first up-opted construct). Frontier advances to `.2`.
