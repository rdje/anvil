---
id: knob-name-suffix-enrolls-a-preset
title: A new `Config` knob's **name** decides its catalog group, and one group is wired to a preset by a drift test — so ending a knob in `_emit_prob` silently enrols it in `--profile structured-emission-max`
answers:
  - "what do I have to do when I add a new Config knob"
  - "why did my new knob land in the other catalog group"
  - "what is knob_group and where does a knob's group come from"
  - "knob_catalog_classifies_every_field is failing"
  - "does a knob name ending in _emit_prob mean anything"
  - "how does a knob get added to a --profile automatically"
  - "why is structured-emission-max derived from the knob catalog"
  - "will naming my knob X put it in a preset"
date: 2026-08-01
status: current
tags: [config, knobs, naming, catalog, preset, profile, drift-test, gotcha, anti-drift]
reverify: "grep -n 'ends_with(\"_emit_prob\")' src/config.rs   # the suffix rule in knob_group; then: grep -n 'fn knob_catalog_classifies_every_field' src/config.rs (an unclassified field is a hard failure, so a new knob MUST get a group) and grep -n 'k.group == \"structured_emission\"' src/config.rs (structured_emission_max_preset_covers_every_non_version_gated_surface derives its expected set from that group, so membership in the group IS membership in the preset)"
evidence: src/config.rs `knob_group` (the `name.ends_with("_emit_prob") || name == "soft_union_slice_prob"` arm returning `"structured_emission"`, and the `"other"` fallthrough); `knob_catalog_classifies_every_field` (forbids `"other"`, so a new field forces a classification); `structured_emission_max_preset_covers_every_non_version_gated_surface` (builds its expected set by filtering `knob_catalog()` on `group == "structured_emission"`, minus the version-gated `soft_union_slice_prob`, and asserts the `structured-emission-max` preset covers all of it); decision `0032` (why that preset means maximal surface *diversity* at `0.25`, not every knob at `1.0`); `CAPABILITY-BREADTH-EXPANSION.4a` (where the trap was found, before the knobs were written)
---

Adding a `Config` field is not a neutral act. Two mechanisms read the field **name**, and the
second one can change what a shipped `--profile` does.

1. **`knob_group(name)` classifies by name**, partly by exact match and partly by *suffix rule* —
   including `name.ends_with("_emit_prob") ⇒ "structured_emission"`. An unrecognised name falls
   through to `"other"`, which `knob_catalog_classifies_every_field` **forbids**. So a new knob
   always forces a decision here; that part is the anti-drift gate working as intended.
2. **The `structured-emission-max` preset is derived from that group.** Its drift test builds the
   expected set by filtering `knob_catalog()` on `group == "structured_emission"` and asserts the
   preset covers every member. So **landing in the group *is* joining the preset** — enforced by a
   test that goes red if you don't.

That coupling is right for the nine emit-projections it was built for (decision
[`0032`](../decisions/0032-emit-surface-interaction-gate.md): a tenth projection must not be able
to omit itself from the preset). It is a **trap** for anything else that happens to be a per-gate
render-time knob without being a projection.

**The case that found it.** The `unique` / `priority` case qualifiers
([`0044`](../decisions/0044-capability-breadth-unique-priority-case-qualifiers.md)) roll once per
candidate gate at emit-annotation time, exactly like the nine projections — so `*_emit_prob` looks
like the natural suffix. It would have been wrong twice over: a qualifier *decorates* a rendering
rather than replacing one, so it adds nothing to the surface-diversity count the preset exists to
maximise; and `unique` makes Icarus print `vvp.tgt sorry: Case unique/unique0 qualities are
ignored.`, so enrolling it would have cost the preset its Icarus column. The knobs are named
`unique_case_prob` / `priority_case_prob` and take their own `case_qualifier` group instead.

**The rule to carry:** when you add a knob, choose the name and the `knob_group` arm **together**,
and ask what else is derived from that group. A suffix that matches an existing rule is a decision
you are making by accident. See also [[knob-presets-and-cli-flags]] for what the presets are, and
[[probability-is-priority-under-mutual-exclusion]] for why `structured-emission-max` sets `0.25`
rather than `1.0`.
