---
id: one-steering-aware-knob-roll-primitive
title: There is exactly **one** knob-roll primitive, and it is steering-aware; a second hand-rolled primitive in the hierarchy planner made the whole `hierarchy` steering category a measured silent no-op
answers:
  - "why does --steer hierarchy do nothing"
  - "why does --steer hierarchy_sibling_route_prob not change anvil output"
  - "which ANVIL knobs are actually reachable by coverage steering"
  - "is every KnobId roll steered by SteeringConfig"
  - "where is ANVIL's knob-roll primitive"
  - "how does anvil stop a second knob-roll primitive from being written"
  - "why did the coverage-steering survey miss the hierarchy rolls"
  - "how do you find every knob-roll site in anvil"
  - "is hierarchy_parent_flop_prob steerable"
  - "what does knob_rolls.record mean and who may call it"
date: 2026-07-30
status: accepted
tags: [steering, coverage, knob-roll, hierarchy, full-factorization, silent-no-op, defect, rules-first, api-completeness, north-star]
evidence: src/gen/cone.rs:42-54 (`roll_knob` — the steering-aware primitive); src/gen/hierarchy.rs:883-938 (the seven hand-rolled `roll_hierarchy_*` helpers that record the same telemetry without the prior); src/config.rs:496-507 (`SteeringConfig::effective_prob` — the multiplier the helpers never call); src/ir/types.rs:666-691 (`KnobId::all` — the 22-knob universe); docs/tasks/COVERAGE-STEERED-GENERATION.md "Implementation Notes" (the `.1` survey claim "All 31 steerable rolls funnel through one function … No call site changes" — false when written); measured probes reproducible from the commands in Context §2
---

# 0034 - COVERAGE-STEERED-GENERATION: one steering-aware knob-roll primitive, structurally enforced

- Date: 2026-07-30
- Status: accepted
- Tree: `COVERAGE-STEERED-GENERATION.3a` (design leaf; decides the fix shape, the
  structural guard, and the scope boundary against `.4`)
- Activated by: autonomous PNT selection (`2026-07-30`) under the owner's standing
  **DECIDE, DON'T ASK** directive, from a source-level observation while surveying the
  recorded decision-`0023` follow-up ("route the remaining raw `gen_bool` sites through
  `roll_knob`") that the hierarchy planner records knob telemetry through helpers of its
  own.

## Context

### 1. What was believed

`COVERAGE-STEERED-GENERATION.2a` (decision [`0023`](0023-coverage-steered-generation.md))
shipped construction-time coverage steering as a **probability-prior multiplier applied at
one place**. The tree's own pre-implementation survey states the premise in full:

> **Single integration point.** All 31 steerable rolls funnel through one function,
> `roll_knob(g, m, knob, prob)` at `src/gen/cone.rs:42` … `.2a` changes ONLY this function
> … **No call site changes.**

Everything downstream rests on that premise: `--steer <key>=<weight>`, the six documented
steering categories, `derive_steering_from_coverage`, the outer measure→derive→re-steer
loop, and the decision-`0017` API-completeness claim for the whole lane.

### 2. What is true (measured `2026-07-30`, commit `ff506e1`)

The premise was false when it was written. `src/gen/hierarchy.rs:883-938` defines **seven
more roll primitives**:

```rust
fn roll_hierarchy_sibling_route(m: &mut Module, rng: &mut impl Rng, prob: f64) -> bool {
    let fired = rng.gen_bool(prob);                                  // <-- no effective_prob
    m.knob_rolls.record(KnobId::HierarchySiblingRouteProb, fired);   // <-- same telemetry
    fired
}
```

They record **exactly the same telemetry** as `roll_knob` and omit **exactly one thing**:
`SteeringConfig::effective_prob`. Steering therefore never reaches them.

**Reachability of the 22-knob `KnobId` universe** (derived by walking every
`roll_knob(` invocation, including the `g.active_flop_knob` indirection):

| | count | knobs |
| --- | --- | --- |
| reached by `roll_knob` ⇒ steered | 16 | all `state` / `selectors` / `datapath` / `terminals` / `sharing` knobs, plus `HierarchyParentFlopProb` (partially — see below) |
| **never reached** ⇒ **unsteerable** | **6** | `HierarchySiblingRouteProb`, `HierarchyRegisteredSiblingRouteProb`, `HierarchyRegisteredSiblingMixedSupportProb`, `HierarchyRegisteredChildInputConeProb`, `HierarchyChildInputConeProb`, `HierarchyParentConeInstanceProb` |

The `hierarchy` category has exactly **seven** members. Six are wholly unsteerable and the
seventh is **half-steered**: `HierarchyParentFlopProb` *is* steered where the planner
temporarily swaps `cfg.flop_prob = cfg.hierarchy_parent_flop_prob` and builds a cone
through `roll_knob` (`hierarchy.rs:1533`, `:1673`), and *is not* at its own dedicated
helper (`hierarchy.rs:934`, called from `:1766`). So a single knob obeys the steer on one
of its two decision sites and ignores it on the other.

**The category is inert. Measured — the recorded fire counts are bit-identical under a
9× weight:**

```
$ B="--seed 42 --hierarchy-depth 1 --num-leaf-modules 2 --num-child-instances 6 \
     --flop-prob 0.0 --hierarchy-sibling-route-prob 0.3 --hierarchy-child-input-cone-prob 0.3"
$ anvil $B --introspect [--steer …] | <read introspection.coverage_readout.knob_fire_rates>

[unsteered]                                    sibling_route 2/5   child_input_cone 0/5
[--steer hierarchy_sibling_route_prob=9.0]     sibling_route 2/5   child_input_cone 0/5
[--steer hierarchy_child_input_cone_prob=9.0]  sibling_route 2/5   child_input_cone 0/5
[--steer hierarchy=9.0]                        sibling_route 2/5   child_input_cone 0/5
```

`hierarchy_child_input_cone_prob` stays at **0 fires in 5 attempts** at a 9× weight. Its
base probability is `0.3`; steered it should be `clamp01(0.3 × 9) = 1.0`, i.e. **5/5**.
Per-knob steer, sibling-knob steer, and whole-category steer are all indistinguishable
from unsteered — in the telemetry *and* in the emitted SV:

```
$ B2="--seed 42 --hierarchy-depth 1 --num-leaf-modules 2 --num-child-instances 4 \
      --hierarchy-{sibling-route,registered-sibling-route,registered-sibling-mixed-support,
                   registered-child-input-cone,child-input-cone,parent-cone-instance,
                   parent-flop}-prob 0.5"
$ for k in <the seven>; do md5(anvil $B2) vs md5(anvil $B2 --steer $k=9.0); done
  NO-OP     hierarchy_sibling_route_prob
  NO-OP     hierarchy_registered_sibling_route_prob
  NO-OP     hierarchy_registered_sibling_mixed_support_prob
  NO-OP     hierarchy_registered_child_input_cone_prob
  NO-OP     hierarchy_child_input_cone_prob
  NO-OP     hierarchy_parent_cone_instance_prob
  effective hierarchy_parent_flop_prob            <-- only via the roll_knob-mediated scopes
```

and `--steer hierarchy=8.0` / `--steer hierarchy=0.01` — an 800× spread — produce
**byte-identical** SV. The positive control works: `--steer state=8.0` changes the output.

The failure is **silent by every surface**. `--steer hierarchy=9.0` is accepted by
`SteeringConfig::set_weight` (`hierarchy` *is* a valid category — it is derived from
`KnobId::category()`, which the six knobs do declare), validated, stored in
`Config.steering`, echoed by `--dump-config` and `--introspect`, and documented in
`book/src/algorithm.md` + `USER_GUIDE.md`. Nothing anywhere reports that it changed
nothing.

### 3. Root cause: the survey searched by shape, not from the authoritative set

`roll_hierarchy_sibling_route` and its siblings landed `2026-04-23` (`28c5474`, "Land
sibling-routed hierarchy child inputs"). The steering multiplier landed `2026-06-21`
(`2530bfd`, `.2a`) — **two months later**. The `.1` survey enumerated the roll sites by
searching for the *shape* it already knew (`roll_knob(` call sites) and concluded "single
integration point".

The **authoritative set** is not `roll_knob(` — it is `knob_rolls.record(`, the site that
commits a roll to the telemetry the steering loop reads back. Searching from *that*
returns 8 sites in 2 files, and the second file is the whole finding. This is decision
[`0033`](0033-shadow-enumeration-classification.md) rule (2) — *"Search from the
AUTHORITATIVE SET, never from the shadow you found first"* — recurring in a second lane,
which is the argument for fixing it structurally rather than by patching seven functions.

It is also a plain `feedback_full_factorization` violation: **one runner, one classifier,
never two.** Two roll primitives existed; the newer capability was added to one of them.

### 4. Why the existing proofs could not catch it

`.2a`'s distribution-shift proof up-weights the `state` category and asserts `flop_prob`'s
fire rate rises. `flop_prob` is a `roll_knob` knob, so the proof passes on the steered half
of the codebase and says nothing about the other half. A proof that picks one member of a
set cannot detect that the set is partitioned.

## Decision

### (a) One primitive, shared — `cone` and `hierarchy` call the same function

Extract the body of `roll_knob` into a single steering-aware primitive whose signature does
not force the caller to hold a `&mut Generator` and a `&mut Module` simultaneously — the
borrow shape that is *why* the hierarchy planner wrote its own (its module is reached
through `ctx.top` while `g` is also live):

```rust
pub(crate) fn roll_knob_into(
    rolls: &mut KnobRollCounters,
    steering: &SteeringConfig,
    rng: &mut impl Rng,
    knob: KnobId,
    prob: f64,
) -> bool
```

`cone::roll_knob(g, m, knob, prob)` becomes a one-line wrapper over it (its 37 call sites
are untouched). The seven `roll_hierarchy_*` helpers are **deleted**, and each of their
seven call sites calls the primitive directly with its explicit `KnobId::…` — the `cone.rs`
convention, which also puts the knob identity at the decision point instead of one
indirection away. Seven near-identical named wrappers are themselves a shadow of the
`KnobId` set (decision `0033`); replacing them with seven direct calls removes the shape,
not just the symptom.

### (b) The guard is a **compile error**, not a doctrine check

`KnobRollCounters::record` moves, together with the primitive, into a new
`src/ir/knob_roll.rs`, and `record` becomes **private to that module**. After the move,
the only way to record a knob roll from anywhere else in the crate is to call
`roll_knob_into` — which applies the prior. A second hand-rolled primitive stops being a
review question and becomes a build failure.

This is the **R2 rung** of the repair ladder (`MEMORY.md`: R1 derive → R2 compile error →
R3 derived test → R4 registered doctrine). R1 is unavailable — a decision site cannot be
derived from data. R2 is available and costs one module move, so nothing weaker is
justified, and **no new registered doctrine is added**: a doctrine check here would be a
permanently-maintained mechanism guarding something the type system can guard for free.

`KnobRollCounters` is re-exported so every existing `crate::ir::…` path and
`Module.knob_rolls` field access keeps working; `metrics.rs` reads `.attempts` / `.fires`
and is unaffected (reading stays public — only *recording* is gated).

### (c) The fix is byte-identical when unsteered, and only then

`SteeringConfig::effective_prob` short-circuits to `prob.min(1.0)` when both weight maps
are empty. Every one of the seven hierarchy call sites already passes a `cfg` probability
validated into `[0, 1]`, so `prob.min(1.0) == prob` exactly and the RNG draw count is
unchanged (one `gen_bool` per roll, as before). **Unsteered output is byte-identical**
(`tests/snapshots.rs` 6/6 untouched); steered hierarchy output *changes*, which is the
entire point and is what the regression proof asserts.

### (d) Scope boundary: this tree node fixes the *reach* of steering, not its *width*

`.3` is scoped to the 22-knob `KnobId` universe: after it, every knob that declares a
`KnobId` is steered at every one of its roll sites. The separate, larger question — that
**16 further `Config` Bernoulli knobs have no `KnobId` at all**, so they are neither
steerable nor visible in `coverage_readout` — is the decision-`0023` follow-up proper and
is registered as `.4`. Measured, that set is:

- **7 module-level motif rolls** — `width_parameterization_prob`, `memory_prob`,
  `fsm_prob`, `fsm_mealy_prob`, `multi_clock_prob`, `aggregate_prob`,
  `aggregate_array_prob`
- **9 emit-projection rolls** — `soft_union_slice_prob`, `function_emit_prob`,
  `generate_loop_emit_prob`, `task_emit_prob`, `multi_output_task_emit_prob`,
  `cone_function_emit_prob`, `mux_if_emit_prob`, `case_mux_if_emit_prob`,
  `casez_mux_if_emit_prob`

Their user-visible symptom differs in kind and that is why they are a separate node:
`--steer memory_prob=2.0` **errors loudly** (`unknown steer key "memory_prob"; expected a
knob name or a category …`) because there is no such `KnobId`. Loudly-absent is a feature
gap; silently-inert is a defect. `.3` fixes the defect; `.4` closes the gap.

Two knobs measured in the same sweep are **excluded from `.4` by kind**:
`operand_duplication_rate` and `mux_arm_duplication_rate` are not Bernoulli decision rolls
at all — they are dedup *thresholds* compared against in `ir/compact.rs` / `metrics.rs`.
There is no roll to apply a prior to. `library_prob` has **no reader anywhere in `src/`**;
it is one of the documented-reserved orphan knobs recorded by `COVERAGE-INSTRUMENTATION.3`
and stays out of scope here.

## Correction (`2026-07-30`, at `.3b` — an explicit amendment, not a silent rewrite)

Two claims above were **wrong**, and are corrected here rather than edited away
(`NEVER REWRITE HISTORY`; a decision record shows its own reasoning, including where that
reasoning was mistaken). Everything else in this record stands, and the fix it pins is
unchanged.

**1. The stated reason for the fork was wrong.** §Decision (a) says the seven helpers
existed because `roll_knob` "forces the caller to hold a `&mut Generator` and a
`&mut Module` simultaneously — the borrow shape that is *why* the hierarchy planner wrote
its own". Measured at `.3b`: **it is not.** `Generator::roll_knob(&mut self, m: &mut Module,
…)` compiles unchanged at **all seven** hierarchy call sites — Rust's two-phase borrows
accept `g.roll_knob(ctx.top, KnobId::X, g.cfg.x)` — with zero warnings.

The real reason is duller and more instructive: the pre-`.3b` `roll_knob` was a
**module-private `fn` in `gen::cone`**. `gen::hierarchy` is a *sibling* module, so it could
not see the function at all, and wrote its own rather than widen the visibility. **A
private helper in one module is an invitation to fork it in the next.** That is the
transferable lesson, and it is a better one than the borrow theory it replaces.

The fix is unaffected — if anything it is simpler than designed. The ergonomic shim is
`Generator::roll_knob` in `src/gen/mod.rs` (crate-visible, so every generator module reaches
the same roll), `cone::roll_knob` survives only as a free-function alias keeping its 37 call
sites' spelling, and `gen/hierarchy.rs` ends up with **zero** roll helpers, calling
`g.roll_knob(m, KnobId::…, prob)` inline at each of its seven decision sites.

**2. "16 of 22 `KnobId`s reached by `roll_knob`" over-counted by one in effect.** The
reachability table counts `HierarchyParentFlopProb` in the reached column because *two* of
its three roll sites go through `roll_knob`. Its dedicated helper site did not, which the
table's footnote says but its arithmetic does not. The honest statement is: **15 knobs were
fully steered, 6 were not steered at all, and 1 was steered at 2 of its 3 sites.** Every
measurement quoted in §2 is unaffected (they are per-knob, not derived from that count).

**What this changes about the guard.** Nothing — it strengthens the case for it. The
mechanism that failed was *visibility*, and a visibility mistake is exactly what the R2
guard now catches: after `.3b`, a module that cannot see `Generator::roll_knob` still cannot
record a roll, because `KnobRollCounters::record` is private to `ir::knob_roll`.
Negative-controlled at `.3b` in both directions: re-introducing the pre-`.3b` helper shape
fails with `error[E0624]: method 'record' is private`, and removing the probe restores a
clean build.

## Decisive test applied

"Would a reviewer reading only the shipped docs be misled about what the product does?"
Yes — `book/src/algorithm.md`, `USER_GUIDE.md`, and decision `0023` all state that steering
biases the construction-time rolls, and one of the six advertised categories biases nothing.
That is the bar for a defect rather than a missing feature, and it is why this lands as a
fix with a regression proof before `.4`'s capability work.

## Rejected alternatives

- **Patch the seven helpers to call `effective_prob` and stop there.** Rejected: it fixes
  today's six knobs and leaves the shape that produced them — a second primitive that
  records the same telemetry — intact and copyable. The eighth helper is a code review away.
- **Keep the seven named helpers as thin wrappers over the shared primitive.** Rejected:
  seven one-line functions whose only content is a `KnobId` are a hand-maintained shadow of
  the `KnobId` set (decision `0033`), and they hide the knob identity from the decision
  site. `cone.rs` names its knob inline at 37 sites; hierarchy should read the same.
- **Enforce the single primitive with a `scripts/check_doctrines.sh` grep** (R4). Rejected:
  the type system can enforce it exactly (R2) for the cost of one module move, and every
  registered doctrine is a mechanism the project maintains forever. Reserve R4 for
  properties no compiler can see.
- **Give the hierarchy helpers a `&mut Generator` so they can call `roll_knob` unchanged.**
  Rejected: they cannot — the module is borrowed through `ctx.top` while `g` is live, which
  is the exact borrow conflict that caused the fork. The primitive must take the counters,
  not the generator.
- **Retro-edit `.2a`'s "single integration point" implementation note.** Rejected —
  `MEMORY_ARCHITECTURE.md` §3: task files are layer-B **history** and are not retro-edited
  (`NEVER REWRITE HISTORY`). The note records what was believed on `2026-06-21`; this
  decision records what is true, and the tree's `Decisions` section links them. A swept
  history would delete the most useful artifact here — the exact shape of the reasoning
  error.
- **Fold `.4`'s 16 knobs into this fix.** Rejected: different kind (feature gap vs defect),
  different blast radius (16 new `KnobId` variants + categories + a schema-visible
  `coverage_readout` widening vs a byte-identical-when-unsteered refactor), and the defect
  should not wait behind the feature.
- **Widening `KnobId::category()`'s taxonomy now.** Rejected: `.3` adds no knob, so the six
  existing categories are unchanged. `.4` decides whether the motif and emit-projection
  knobs join `state`/`datapath` or earn `motifs`/`emission` categories.
- **Changing any default.** Rejected: unsteered generation stays byte-identical; steering
  stays opt-in.

## Consequences

- `--steer hierarchy=<w>` and the six per-knob hierarchy steers start working. Every
  `KnobId` is steered at **every** one of its roll sites, so `SteeringConfig`'s contract —
  and the decision-`0017` API-completeness claim for this lane — becomes true as written
  rather than true for 16 of 22 knobs.
- The outer measure→derive→re-steer loop becomes sound over the hierarchy category:
  `derive_steering_from_coverage` already emits weights for it, and those weights were
  being computed, serialized, and discarded.
- A second knob-roll primitive becomes a compile error. The failure mode that took two
  months to surface cannot recur in this crate.
- Unsteered DUT emission is byte-identical; steered hierarchy emission changes (proven, and
  that change is the fix).
- The `.2a` distribution-shift proof gains a hierarchy-category sibling, so the next
  partition of the roll surface would fail a test rather than pass one.
- `.4` inherits a clean base: one primitive to route new knobs through, with the prior
  already applied.

## Open questions (to be resolved at `.3b` / `.3c` / `.4`)

- Whether the shared primitive lives in `src/ir/knob_roll.rs` beside `KnobRollCounters`
  (proposed — it is what makes `record` privatizable) or in `src/gen/roll.rs` with the
  counters staying in `ir::types` (which cannot give the R2 guard, since `pub(in path)`
  requires an ancestor module).
- Whether `roll_knob_into` takes `&mut KnobRollCounters` (proposed — minimal, and avoids an
  `ir::knob_roll` → `ir::types::Module` dependency) or `&mut Module`.
- Whether `HierarchyParentFlopProb`'s **two** meanings should be separated. It currently
  labels both the `cfg.flop_prob`-swap cone rolls and the dedicated helper roll, so its
  reported fire rate conflates two different decisions. Not a steering defect once `.3b`
  lands (both sites will be steered by the same weight) — but it is a telemetry-honesty
  question, and the answer may be a second `KnobId`. Recorded, not decided.
- `.4`: whether the 16 knobs join the existing six categories or introduce `motifs` /
  `emission`, and whether widening `coverage_readout`'s key set warrants an introspection
  schema MINOR bump (the map is additive, but the lane's convention has been to bump).

## Tree split

`COVERAGE-STEERED-GENERATION` reopens with a new `.3` node (the closed `.1`/`.2` scope is
untouched — the Phase-4 closure pattern, `feedback_never_retire_strategies`):

- **`.3a`** (this leaf, design) — decision `0034`: the measured defect, the root cause, the
  one-primitive fix, the R2 structural guard, and the `.3`/`.4` scope boundary. Docs-only.
- **`.3b`** (`pending`) — the fix: `roll_knob_into` + the privatized `record` + the seven
  deleted helpers + their seven re-pointed call sites; the hierarchy-category
  distribution-shift regression proof; neutral-weight and unsteered byte-identity proofs;
  full `COMMIT.md` cargo gate.
- **`.3c`** (`pending`) — docs/close: `book/src/algorithm.md` (the steering section's
  "one primitive" statement + the guard), `book/src/knobs.md`, `USER_GUIDE.md`, the KM
  card, and the tree/`docs/TASK_TREE.md` close of `.3`.
- **`.4`** (`pending`, new) — the decision-`0023` follow-up proper: give the 16 remaining
  Bernoulli knobs a `KnobId` and route them through the one primitive, so they gain
  telemetry **and** steerability together.

## Links

- Owner doctrine: **DECIDE, DON'T ASK** and **A DEFECT IS ONLY HANDLED IF A TASK-TREE OWNS
  IT** (`MEMORY.md` standing directives, `2026-07-30`).
- Supersedes-in-part: decision [`0023`](0023-coverage-steered-generation.md) §"First cut
  steers only the `roll_knob`-mediated knobs (the `KnobId` set)". That sentence is now
  literally true only after `.3b`; before it, the `KnobId` set and the `roll_knob`-mediated
  set were **not** the same set. `0023` is not retracted — its primitive, contract, and
  rejected alternatives all stand.
- Doctrine: `feedback_full_factorization` (one runner/classifier — the violation this fixes),
  `feedback_rules_first_generation` (the prior stays a construction-time multiplier; no
  filter, no extra draw), `feedback_never_retire_strategies` (nothing removed; the closed
  `.1`/`.2` scope stands), decision [`0017`](0017-api-first-everything-mcp-accessible.md)
  (API-completeness — the gate this defect was failing), decision
  [`0033`](0033-shadow-enumeration-classification.md) (search from the authoritative set;
  a list of near-identical wrappers is a shadow).
- Lane / ROADMAP: steering gap 3 — *"the probability knobs must be exercised without hidden
  bias from whichever implementation path is currently easiest"*. A steering category that
  silently cannot fire is exactly that bias.
- Reuse / touch points: `src/gen/cone.rs` (`roll_knob` → wrapper), `src/gen/hierarchy.rs`
  (delete 7 helpers, re-point 7 call sites), `src/ir/knob_roll.rs` (new — the primitive +
  the privatized `record`), `src/ir/types.rs` (`KnobRollCounters` moves out; `Module` field
  and `KnobId` unchanged), `tests/pipeline.rs` (the hierarchy distribution-shift proof),
  `book/src/algorithm.md` + `book/src/knobs.md` + `USER_GUIDE.md` (`.3c`).
