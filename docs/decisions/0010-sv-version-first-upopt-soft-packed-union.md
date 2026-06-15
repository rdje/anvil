---
id: sv-version-first-upopt-soft-packed-union
title: ANVIL's first version-distinctive up-opt is a heterogeneous-width packed `union soft` (IEEE 1800-2023 §7.3.1), default-off / byte-identical
answers:
  - "what is ANVIL's first up-opted SystemVerilog construct"
  - "what is the first SV-VERSION-TARGETING up-opt"
  - "does ANVIL emit a SystemVerilog 2023 soft packed union"
  - "what construct does sv_version >= 2023 gate"
  - "is a heterogeneous-width packed union legal before SystemVerilog 2023"
  - "do downstream tools enforce IEEE 1800 version acceptance"
  - "does verilator --language reject newer SystemVerilog constructs"
  - "does verilator 5.046 differentiate 1800-2012 1800-2017 1800-2023"
  - "why is the first ANVIL up-opt a 2023 construct and not a 2017 construct"
  - "how does ANVIL prove an up-opted 2023 construct is downstream-accepted"
  - "why does the soft union up-opt record yosys and icarus as no-ops"
date: 2026-06-16
status: accepted
tags: [capability, sv-version, up-opt, emission, downstream, soft-union, 2023, valid-by-construction, north-star]
evidence: docs/decisions/0010-sv-version-first-upopt-soft-packed-union.md; docs/tasks/SV-VERSION-TARGETING.md; docs/decisions/0009-sv-version-targeting.md; src/emit/sv.rs; src/ir/aggregate.rs; src/bin/tool_matrix.rs
reverify: 'printf ''module v(input logic[7:0] a,input logic b,output logic[7:0] y);typedef union soft{logic[7:0] m0;logic m1;}u_t;u_t u;always_comb u=a;assign y=u.m0^{7''"''"''b0,u.m1}^{7''"''"''b0,b};endmodule\n'' > /tmp/us.sv && verilator --lint-only --language 1800-2023 /tmp/us.sv && echo accepts-2023; printf ''module v(input logic[7:0] a,output logic[7:0] y);typedef union packed{logic[7:0] m0;logic m1;}u_t;u_t u;always_comb u=a;assign y=u.m0;endmodule\n'' > /tmp/up.sv && (verilator --lint-only --language 1800-2012 /tmp/up.sv || echo hard-union-rejected-pre-2023)'
---

# 0010 - SV-VERSION-TARGETING: the first up-opted construct is a heterogeneous-width packed `union soft` (IEEE 1800-2023 §7.3.1)

- Date: 2026-06-16
- Status: accepted
- Tree: `SV-VERSION-TARGETING.3a` (design leaf; splits `.3` into `.3a` design + `.3b` impl)
- Supersedes/extends: decision [`0009`](0009-sv-version-targeting.md) (the `--sv-version` lane)

## Context

Decision `0009` opened the `--sv-version <2012|2017|2023>` capability lane with
two construction-time effects, both rules-first: **down-gating** (never emit a
construct newer than the target — a standard-validity guarantee) and
**up-opting** (deliberately emit a higher standard's distinctive synthesizable
constructs, each proven downstream-clean in the matching tool standard mode).
`SV-VERSION-TARGETING.2` is now fully delivered and closed: the `SvVersion`
knob, the emitter capability bound (`SvVersion::permits`), the versioned emitter
entry points, the introspection field (schema `1.2`), the downstream
`--language 1800-20xx` selector, and the repo-owned
`tool_matrix --sv-version-gate` per-version acceptance axis — **all default
byte-identical**, because today's emitted subset is a 1800-2012 floor that
down-gates to any target as a provable no-op.

`0009` deferred the *first version-distinctive up-opted construct* to `.3`,
explicitly design-first: "pick one synthesizable construct introduced by 2017 or
2023 that a downstream tool accepts in its version mode, prove it clean, enable
it gated on `sv_version >= that_standard`." This record is that design leaf
(`.3a`). It is grounded in a direct empirical probe of the **installed**
downstream tools — Verilator 5.046, Yosys 0.64, Icarus Verilog 13.0 — so the
choice is evidence-backed, never aspirational (`feedback_rules_first_generation`,
`project_anvil_north_star`).

## Empirical tool-reality finding (load-bearing, documented honestly)

A probe battery of candidate 2017/2023 synthesizable constructs across the three
installed tools at each IEEE 1800 mode established two facts that shape the whole
up-opt design:

1. **Verilator 5.046 does not differentiate `--language 1800-2012` /
   `1800-2017` / `1800-2023` for acceptance.** Every supported construct, and
   even keyword reservation, is identical across the three modes — e.g. `soft`
   (a 2023 keyword) and `implements` (a 2012 keyword) used as identifiers are
   rejected at *all three* modes, not just the standard that reserves them.
   **There is therefore no construct for which `--language 1800-2012` rejects and
   `--language 1800-2023` accepts.** The `--language` flag is accepted and
   recorded but does not gate acceptance.
2. **Yosys 0.64 and Icarus 13.0 expose no IEEE-1800 version selector and parse a
   fixed, conservative synthesizable subset** that rejects most genuinely-newer
   SV syntax (`union soft`, `let`, streaming concat, `case … inside`,
   `'{default:…}` assignment patterns, `parameter type` defaults, default
   subroutine arguments).

Consequence: with the installed tools, **the per-version axis cannot demonstrate
tool-side version *rejection*.** The up-opt's teeth are therefore the three that
*are* real and provable:

- **(a) LRM correctness** — the construct is genuinely introduced by the targeted
  standard, so ANVIL's down-gating guarantee (absence below the target) is a true
  standard-validity guarantee, not a tautology over a 2012-floor subset;
- **(b) ANVIL's construction-time down-gating guarantee** — the construct is
  emitted *because* the target permits it (`sv_version.permits(Sv2023)`), never
  materialized-then-stripped;
- **(c) matching-mode acceptance** — the construct is accepted/elaborated by a
  downstream tool under its matching-standard invocation
  (`verilator --language 1800-2023`, and `--binary` build).

This is the honest bar; `0009`'s "proven downstream-clean in the matching tool
standard mode" is satisfied by (c), and the gate must not claim version-rejection
the installed tools do not perform.

## Decision

**ANVIL's first up-opted construct is a heterogeneous-width packed union emitted
as `union soft` (IEEE 1800-2023 §7.3.1), as a new default-off aggregate
projection gated at construction time on `sv_version >= Sv2023`.**

It is the natural rules-first sibling of the existing packed-aggregate
projections (`AggregateKind::StructPacked` / `ArrayPacked` in
`src/ir/aggregate.rs`, rendered by `render_aggregate_typedef` in
`src/emit/sv.rs`): instead of folding selected same-direction ports into a packed
`struct` (concatenation), the up-opted projection overlays them in a packed
`union soft` (aliased storage with **heterogeneous member widths**, which is
exactly the 2023-only capability).

### Why this construct is the right first up-opt

- **Genuinely 2023 — unimpeachable LRM teeth.** A *non-soft* packed union with
  heterogeneous-width members is illegal before 2023, and all three tools reject
  it; Verilator's own diagnostic on the plain form cites the standard verbatim:
  `Hard packed union members must have equal size (IEEE 1800-2023 7.3.1)`.
  Heterogeneous-width members are legal *only* as `union soft`, a 1800-2023
  addition. So at `sv_version < 2023` ANVIL must not emit it (real down-gating
  teeth, unlike the vacuously-down-gated 2012-floor subset), and the existing
  down-gate fallback — the packed `struct` projection — keeps the default
  byte-identical.
- **Synthesizable, proven.** `verilator --binary --language 1800-2023` builds and
  simulates a soft-union overlay correctly (probe top produced `y=a5`); it is not
  merely lint-accepted.
- **Rules-first / by-construction.** It reuses the aggregate machinery's existing
  port-selection and boundary-alias discipline; the version is a construction
  bound, never a post-hoc filter (`feedback_rules_first_generation`, core
  principle 2).
- **Default-off / byte-identical.** It fires only when **both** `sv_version >=
  Sv2023` **and** a new default-off union-projection knob select it;
  `sv_version` defaults to `Sv2012`, so `tests/snapshots.rs` stays untouched
  (`feedback_never_retire_strategies` — nothing existing is retired; the struct
  projection remains).

### Per-version downstream proof handling

- **Verilator `--language 1800-2023` (accept + `--binary` build) is the primary
  matching-mode proof** — the only installed tool that elaborates `union soft`.
- **Yosys 0.64 and Icarus 13.0 reject the `union soft` syntax** and have no
  1800-version selector, so for the up-opt scenario they are a **recorded no-op,
  not a failure** — precisely the path `0009` already authorized for Icarus
  beyond `-g2012`, here extended to Yosys for a construct its frontend cannot
  parse. The existing `tool_matrix --sv-version-gate` facts
  (`saw_sv_version_2023_targeted_acceptance`) require *Yosys-clean*, so the union
  scenario **cannot** reuse them unchanged; `.3b` adds a **dedicated up-opt
  coverage fact** (working name `saw_sv_version_2023_soft_union_upopt`) that
  requires Verilator matching-mode acceptance and records Yosys/Icarus as no-ops.

## Rejected alternatives (with evidence)

- **`genvar`-in-generate-for header, unbased-unsized `'1`, signed/unsigned
  cast, default subroutine arguments, `parameter type` defaults.** Probed
  accepted by the tools at *all* `--language` modes (and several by Yosys/Icarus)
  because they are legal at the 2012 floor — **no down-gating teeth**, so not
  genuinely version-distinctive. Rejected.
- **A 1800-2017-distinctive construct.** 1800-2017 is a maintenance/clarification
  revision; the probe surfaced no synthesizable construct distinctive to 2017
  over 2012. The first up-opt is therefore necessarily a **2023** construct (2017
  remains a valid future target with no required distinctive emission).
- **`case … inside`, streaming concatenation, `let` declarations,
  `'{default:…}` assignment patterns.** Either not genuinely 2023 (`inside` is
  2012) or rejected by Yosys *and* Icarus while not offering a cleaner 2023 story
  than the soft union. Rejected as the *first* up-opt (some may return as later
  up-opts).
- **Claiming tool-side version rejection.** Rejected as aspirational: the
  installed Verilator does not reject newer constructs at lower `--language`
  modes; the design documents this honestly and rests the up-opt on LRM +
  construction-time down-gating + matching-mode acceptance.
- **Generate-then-filter; a non-byte-identical default.** Rejected by doctrine,
  as in `0009` (`feedback_rules_first_generation`, `feedback_never_retire_strategies`).

## Tree split

`SV-VERSION-TARGETING.3` becomes a container:

- **`.3a`** (this leaf, design) — names the first up-opt construct, the empirical
  tool-reality finding, the gate (`sv_version >= Sv2023`), the rules-first /
  default-off discipline, and the downstream-proof handling. Docs-only.
- **`.3b`** (impl, `proposed`) — implement the `union soft` projection gated on
  `sv_version >= Sv2023`, default-off / byte-identical, proven Verilator
  matching-mode-clean with Yosys/Icarus recorded no-op; extend
  `tests/sv_version.rs` (now showing **divergence** at 2023 when the knob fires)
  + `tests/sv_version_downstream.rs` + the matrix up-opt fact + book/KM. Pre-split
  into `.3b.1` (design-detail) + `.3b.2` (impl) when picked, per the `.2b`
  precedent, if it proves broad.

## Open questions (resolved at `.3b` / `.3b.1`)

- **Projection shape.** Input-port fold into one `union soft`-typed boundary port
  (changes the input bit-budget: union width = max member width) vs an
  internal-only `union soft` overlay over an existing wide signal (keeps the port
  boundary identical — lowest blast radius). First cut likely the internal/lower-
  risk shape.
- **`AggregateKind` variant + emitter site + the exact `permits(Sv2023)` gate**
  inside `render_aggregate_typedef` / the aggregate selection path.
- **The new union-projection knob** name + default-off value, and how it composes
  with the existing `aggregate_prob` / `aggregate_array_prob` selection.
- **Matrix wiring** for the dedicated up-opt scenario + the
  `saw_sv_version_2023_soft_union_upopt` fact + recording Yosys/Icarus as a no-op
  rather than a gate failure.

## Consequences

- ANVIL gains its first genuinely version-distinctive emission: `--sv-version
  2023` + the union knob produces standard-valid RTL that **no `< 2023` flow
  accepts** — real down-gating teeth and a real up-opt stress for 2023-aware
  parsers/elaborators (the north star).
- The default and every existing gate stay byte-identical (`sv_version` floor +
  default-off knob); `--identity-mode`, factorization, and every other knob are
  orthogonal and untouched.
- The adversarial matrix gains a dedicated 2023 up-opt acceptance axis beside the
  per-version `.2b.2b` facts (ROADMAP steering gap 3).

## Links

- Task-tree: `SV-VERSION-TARGETING.3a` (this leaf); frontier advances to `.3b`.
- Parent decision: `0009` (the `--sv-version` lane).
- North star: `project_anvil_north_star`; ROADMAP steering gaps 1 (breadth) + 3
  (explicit adversarial axis matrix).
- Doctrine: `feedback_rules_first_generation`, `feedback_never_retire_strategies`,
  `feedback_book_doctrine` (the construct is user-facing → book chapter at `.3b`).
- Reuse / touch points: `src/ir/aggregate.rs` (`AggregateKind`), `src/emit/sv.rs`
  (`render_aggregate_typedef`, `to_sv_with_modules` `sv_version` bound),
  `src/config.rs` (the new union knob), `src/bin/tool_matrix.rs`
  (`SvVersionSweep` up-opt scenario + fact), `tests/sv_version.rs`,
  `tests/sv_version_downstream.rs`.
