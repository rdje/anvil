---
id: sv-version-targeting
title: ANVIL gains a --sv-version capability gate that targets a chosen IEEE 1800 standard valid-by-construction (default byte-identical)
answers:
  - "can ANVIL target a specific SystemVerilog version"
  - "what does --sv-version do"
  - "does ANVIL support IEEE 1800-2017 or 1800-2023"
  - "how does ANVIL target a chosen SystemVerilog standard"
  - "is --sv-version default-off / byte-identical"
  - "how does ANVIL prove version-targeted RTL is accepted by downstream tools"
  - "what SystemVerilog version does ANVIL emit"
  - "can ANVIL emit 2023-only or 2017-only constructs"
  - "why a SystemVerilog version capability gate"
date: 2026-06-15
status: accepted
tags: [capability, sv-version, emission, downstream, valid-by-construction, north-star, breadth]
evidence: docs/decisions/0009-sv-version-targeting.md; docs/tasks/SV-VERSION-TARGETING.md; src/emit/sv.rs; src/downstream/mod.rs; src/config.rs; ROADMAP.md
---

# 0009 - SV-VERSION-TARGETING: a `--sv-version` capability gate targeting a chosen IEEE 1800 standard

- Date: 2026-06-15
- Status: accepted
- Tags: capability, sv-version, emission, downstream, valid-by-construction, north-star

## Context

Owner-directed roadmap steering (`2026-06-15`): the highest-leverage next
capability lane is a **`--sv-version` targeting gate** that lets ANVIL generate
RTL aimed at a chosen SystemVerilog standard (IEEE 1800-**2017** / **2023**,
with a 2012-era floor), valid-by-construction. Two sibling lanes were also named
and registered `proposed` (`STRUCTURED-EMISSION-EXPANSION`,
`SEMANTIC-INTROSPECTION-EXPANSION`); this decision designs the recommended one.

It is directly aligned with the north star (`project_anvil_north_star`): the
product surfaces downstream-tool bugs via valid-by-construction,
downstream-acceptance-quality output. Parsers, elaborators, RTL compilers,
linters, and simulators have **version-specific** acceptance behaviour. Today
ANVIL emits a single conservative synthesizable subset and the downstream gates
run at fixed standards with no version axis:

- Emitter (`src/emit/sv.rs`): `module`/`logic`/`always_ff`/`always_comb`, packed
  arrays for memory, packed `struct` for aggregates — constructs valid across
  IEEE 1800-2012/2017/2023 (a common floor). There is no version selector.
- Downstream (`src/downstream/mod.rs`): `verilator --lint-only` (tool default
  language), `yosys read_verilog -sv`, `iverilog -g2012` — each pinned to one
  implicit standard, no per-version acceptance column.

So ANVIL cannot today (a) **guarantee** its output avoids constructs newer than a
chosen standard (down-targeting, for tools/flows pinned to an older standard),
nor (b) **deliberately exercise** a newer standard's distinctive synthesizable
constructs (up-targeting, to stress version-specific tool paths). Both are
north-star value the gate unlocks.

## Decision

**ANVIL gains an opt-in `--sv-version <2012|2017|2023>` capability gate (a
`Config::sv_version` enum, CLI flag + dump-config + introspection field) that
makes the generator/emitter target the chosen IEEE 1800 standard
valid-by-construction.** It has two construction-time effects, both rules-first
(no generate-then-filter):

1. **Down-gating (a guarantee).** When a lower standard is selected, the
   emitter/generator never emits a construct introduced after that standard.
   Output is guaranteed parseable/elaboratable by a tool or flow pinned to that
   standard. (Because today's subset is already a 2012-era floor, the floor
   target reproduces today's output.)
2. **Up-opting (the stress value).** When a higher standard is selected, the
   generator *may* opt into synthesizable constructs introduced by that standard,
   each gated at construction time on `sv_version >= that_standard`, so ANVIL
   emits legal RTL that exercises the newer standard's parser/elaborator paths.
   Every up-opted construct must be proven **downstream-clean in the matching
   tool standard mode** before it is enabled by default for that version
   (no aspirational claims; a construct whose tool support is absent is deferred,
   not emitted).

### Default (byte-identical)

`Config::sv_version` defaults to the value that reproduces **today's** emission
(working name `Sv2017`, finalized at `.2` — whichever floor value is byte-equal
to current output across the corpus; the current subset is a 2012/2017 common
floor). The default invocation, and every existing test / snapshot / book gate,
stays **byte-identical** — `tests/snapshots.rs` is untouched. The flag is the
real switch; selecting a *different* version is the only way to change output.

### Valid-by-construction discipline

The version is a construction/emission-time constraint, not a post-hoc filter
(`feedback_rules_first_generation`). A construct is chosen *because* the target
permits it; an out-of-version construct is never materialized then stripped.
This keeps every emitted artifact synthesizable and standard-valid by
construction, per the project's core principle 2.

### Downstream gate (per-version acceptance)

A per-version acceptance axis in the tool harness
(`src/downstream/mod.rs` + `tool_matrix`): the chosen-version corpus is run
through the downstream tools **in their matching standard mode** where the tool
exposes one —

- Verilator: `--language 1800-2017` / `--language 1800-2023` (Verilator exposes a
  language-standard selector); default-language for the floor.
- Icarus: `-g2012` remains the newest generation iverilog supports — so 2017/2023
  corpora are gated to the subset iverilog still accepts, or the iverilog column
  is a friendly no-op for constructs beyond `-g2012` (recorded, not failed).
- Yosys: `read_verilog -sv` (SystemVerilog mode); no finer standard selector,
  so it validates the synthesizable-subset acceptance.

Counterexamples are retained with the exact seed + `sv_version` + effective
knobs (the existing signoff discipline), and a coverage fact records that a
version-distinctive construct was exercised and accepted.

## First increment (the `.2` impl leaf, scope)

The first impl increment is the **plumbing + the down-gating guarantee + the
per-version acceptance axis** over the *existing* subset:

- `Config::sv_version` enum (+ CLI `--sv-version`, `--dump-config`,
  introspection field; introspection schema MINOR-bumped per its own policy);
- thread the target version into the emitter (and any version-relevant generator
  choice) as a read-only capability bound;
- the per-version downstream acceptance column;
- prove the floor target byte-identical (snapshots 6/6) and the
  2017/2023 targets downstream-clean over the current subset in the matching
  tool standard mode.

The *first version-distinctive up-opted construct* is then its own subsequent
leaf (design-first: pick one synthesizable construct introduced by 2017 or 2023
that a downstream tool accepts in its version mode, prove it clean, enable it
gated on `sv_version >= that_standard`). Nothing is emitted that a tool cannot
accept in the matching mode.

## Decisive test applied

"Every emitted module is valid by construction, and the gate is a *guarantee*,
not a hope." A version gate that constrains emission at construction time gives a
provable standard-validity guarantee (down-gating) and a deliberate, proven,
tool-accepted way to exercise newer standards (up-opting) — exactly the
legal-but-unusual, version-specific RTL the north star wants downstream tools to
ingest.

## Rejected alternatives

- **Generate freely, then filter/reject out-of-version constructs.** Forbidden by
  `feedback_rules_first_generation` and core principle 2 (no generate-then-filter;
  a construction-time rule IS the statement). The version is a construction-time
  capability bound.
- **A single "newest" emission with no selector.** Loses the down-gating
  guarantee (can't target an older flow) and couples ANVIL to one tool's default;
  the whole value is the *axis*.
- **Emit version-distinctive constructs without a matching-mode downstream
  proof.** Violates the signoff / no-aspirational-claims bar; an up-opted
  construct lands only once proven accepted in the matching tool mode.
- **Make `--sv-version` change the *default* output (non-byte-identical).**
  Rejected: the default must reproduce today's emission byte-for-byte
  (`tests/snapshots.rs` untouched); the gate is opt-in like every other ANVIL
  capability knob (`feedback_never_retire_strategies` — nothing existing is
  retired).
- **Treat SV-2005/Verilog-2001 as targets.** Out of scope: ANVIL emits
  SystemVerilog (`logic`, `always_ff`); the floor is the 2012-era synthesizable
  SV subset, not classic Verilog. A Verilog-target lane, if ever wanted, is a
  separate future tree.

No mode/strategy is retired; the default stays byte-identical and the other two
owner-named lanes (`STRUCTURED-EMISSION-EXPANSION`,
`SEMANTIC-INTROSPECTION-EXPANSION`) remain registered `proposed`.

## Tree split

`SV-VERSION-TARGETING.1` (this leaf) opens the lane and records this decision.
Forward:

- **`.2`** — implement the plumbing + down-gating + per-version acceptance axis
  over the existing subset (default byte-identical). Split into design-detail +
  impl if broad (the `.2a`/`.2b` precedent).
- **`.3` (future, `proposed`)** — the first version-distinctive up-opted
  synthesizable construct, design-first, proven downstream-clean in the matching
  tool standard mode.

## Consequences

- ANVIL gains a standard-targeting axis: a *guarantee* of standard-validity
  (down-gating) and a *proven* way to stress version-specific tool paths
  (up-opting) — both valid-by-construction, both default byte-identical.
- The adversarial matrix gains an explicit `sv_version` axis (ROADMAP steering
  gap 3 — model the adversarial space as an explicit axis matrix, not one vague
  "randomness").
- No existing output changes by default; `--identity-mode`, factorization, and
  every existing knob are orthogonal and untouched.

## Open questions

- `.2` finalizes the enum spelling + default value (the exact byte-identical
  floor), the introspection field name, and whether the per-version tool axis
  lives in `tool_matrix` as a new gate or as a `--sv-version`-parameterized run
  of the existing columns.
- Which Verilator language-selector spelling the installed version accepts
  (`--language 1800-2023` vs `--default-language`), probed at `.2`.

## Links

- Task-tree: `SV-VERSION-TARGETING.1` (this leaf); frontier advances to `.2`
- Sibling owner-directed lanes (registered `proposed`):
  `STRUCTURED-EMISSION-EXPANSION`, `SEMANTIC-INTROSPECTION-EXPANSION`
- North star: `project_anvil_north_star` (auto-memory); ROADMAP steering gaps 1
  (breadth) + 3 (explicit adversarial axis matrix)
- Doctrine: `feedback_rules_first_generation` (valid-by-construction, no
  generate-then-filter), `feedback_never_retire_strategies` (default
  byte-identical, opt-in), `feedback_book_doctrine` (the gate is user-facing →
  book chapter at impl)
- Reuse / touch points: `src/config.rs` (knob), `src/emit/sv.rs` (version-gated
  emission), `src/downstream/mod.rs` + `src/bin/tool_matrix.rs` (per-version
  acceptance), `src/introspect/` + `docs/AGENT_INTROSPECTION_SCHEMA.md`
  (introspection field)
