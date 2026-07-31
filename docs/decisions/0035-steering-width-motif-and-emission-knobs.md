---
id: steering-width-motif-and-emission-knobs
title: Steering's **width** — the 16 motif and emit-projection knobs join the `KnobId` universe under two new categories (`motifs`, `emission`), routed through the one primitive so they gain telemetry and steerability together
answers:
  - "why can I not steer memory_prob or function_emit_prob"
  - "how do I add a new steerable knob to ANVIL"
  - "which ANVIL knobs are missing from the coverage readout"
  - "what are the ANVIL steering categories"
  - "does adding a KnobId change the introspection schema version"
  - "why does a default-off knob record zero attempts instead of zero fires"
  - "are operand_duplication_rate and mux_arm_duplication_rate steerable"
  - "what is library_prob"
  - "how many roll sites does an emit-projection knob have"
  - "how do I measure how often each structured-emission surface fires"
  - "what are the motifs and emission steering categories"
  - "how do I steer all nine emit-projection surfaces at once"
  - "why is emission a separate steering category from motifs"
  - "why is KnobId::all generated from a macro table"
  - "where is the authoritative list of ANVIL steerable knobs"
  - "when should a guard be removed instead of reinforced"
  - "why was the KnobId index() guard deleted"
date: 2026-07-30
status: delivered
tags: [steering, coverage, knob-roll, motifs, structured-emission, api-completeness, rules-first, telemetry, north-star]
evidence: src/config.rs:1657-1729 (the 41-entry probability validation list — all 16 candidates are range-checked into [0,1], which is what makes the routing byte-identical); src/gen/module.rs:386-417 (the 4 module-level motif rolls) + src/gen/mod.rs:77-190,196-460 (the multi-clock / aggregate / 9 emit-projection call sites, each duplicated across `generate_module` and `generate_module_with_interface_profile`); src/ir/{soft_union,function_emit,generate_loop,task_emit,multi_output_task_emit,cone_function_emit,mux_if_emit,case_mux_if_emit,casez_mux_if_emit}.rs (the 9 `annotate_*(m, rng, prob)` signatures that must gain a `&SteeringConfig`); src/ir/knob_roll.rs (the one primitive, decision 0034); src/ir/types.rs (`KnobId::all` — the 22-entry list this widens to 38); docs/AGENT_INTROSPECTION_SCHEMA.md §6.8 (`coverage_readout` is a map, so widening its key set is data, not shape); DELIVERED by .4b.1 (7 motif knobs) + .4b.2 (9 emission knobs) + .4c (docs), on the guard .5 completed; SINCE .6 (2026-07-31, see the Resolution section) the knob universe lives in src/ir/knob_id.rs as ONE `knob_ids!` macro table that generates the enum + all() + name() + category(), so the `KnobId::all` list this decision widened no longer exists to be widened — a new knob is one row
reverify: cargo run --release -- --seed 7 --profile structured-emission-max --introspect   # introspection.coverage_readout.knob_fire_rates must carry all 8 non-version-gated emission knobs, each ~0.25, under a single `emission` category; before .4b.2 it carried NONE. Then: cargo run --release -- --seed 7 --memory-prob 0.5 --introspect  # memory_prob under `motifs`; the readout was EMPTY before .4b.1. Then, for the .6 Resolution: for v in $(sed -n '/^knob_ids! {$/,/^}$/p' src/ir/knob_id.rs | sed -nE 's/^[[:space:]]*([A-Za-z0-9_]+)[[:space:]]*=>.*;$/\1/p'); do grep -c "\b$v\b" src/ir/knob_id.rs; done | sort -u   # must print exactly `1` — every variant name appears once in the file; the same loop against `git show 218277d:src/ir/knob_id.rs` prints `5`
---

# 0035 - COVERAGE-STEERED-GENERATION.4: steering's width

- Date: 2026-07-30
- Status: accepted
- Tree: `COVERAGE-STEERED-GENERATION.4a` (design leaf; decides the category taxonomy,
  the signature change, the byte-identity argument, the schema call, and the `.4b` split)
- Activated by: autonomous PNT selection (`2026-07-30`) under the owner's standing
  **DECIDE, DON'T ASK** directive, continuing the tree whose `.3` node closed the same
  day.

## Context

Decision [`0034`](0034-one-steering-aware-knob-roll-primitive.md) fixed steering's
**reach**: every knob that has a `KnobId` is now steered at every one of its roll sites,
enforced by making a second roll primitive a compile error. This decision is about its
**width** — the knobs that have no `KnobId` at all.

### 1. The gap, measured

`Config` carries **41** `f64` probability knobs (the exact set `Config::validate`
range-checks). **22** have a `KnobId`. Of the remaining 19:

| group | count | knobs |
| --- | --- | --- |
| **module-level motif rolls** | 7 | `width_parameterization_prob`, `memory_prob`, `fsm_prob`, `fsm_mealy_prob`, `multi_clock_prob`, `aggregate_prob`, `aggregate_array_prob` |
| **emit-projection rolls** | 9 | `soft_union_slice_prob`, `function_emit_prob`, `generate_loop_emit_prob`, `task_emit_prob`, `multi_output_task_emit_prob`, `cone_function_emit_prob`, `mux_if_emit_prob`, `case_mux_if_emit_prob`, `casez_mux_if_emit_prob` |
| **excluded by kind** | 3 | `operand_duplication_rate`, `mux_arm_duplication_rate`, `library_prob` |

**16 knobs in scope.** Their symptom is the opposite of `0034`'s: *loud*.

```
$ anvil --seed 7 --steer memory_prob=2.0
Error: unknown steer key "memory_prob"; expected a knob name or a category
       (datapath, hierarchy, selectors, sharing, state, terminals)
```

Loudly-absent is a **feature gap**; silently-inert was a **defect**. That distinction is
why `0034` shipped first and alone.

The cost of the gap is not only that `--steer` cannot name them. It is that the whole
outer **measure → derive → re-steer** loop is blind to them: they contribute no
`knob_fire_rates` entry, so `derive_steering_from_coverage` cannot see that
`function_emit_prob` fired 12 times in 900 opportunities, and an agent hunting for
under-exercised constructs has no signal for **nine of ANVIL's most interesting
surfaces** — the entire structured-emission family that decision `0032` calls the DUT
lane's best bug-bait.

### 2. The three exclusions are exclusions of *kind*, not of effort

- **`operand_duplication_rate` / `mux_arm_duplication_rate`** are **not Bernoulli
  rolls**. They are thresholds compared against in `ir/compact.rs` and `metrics.rs`
  (`if m.operand_duplication_rate < 1.0`), reached through `Module` fields rather than
  an RNG draw. There is no roll, so there is nothing for a probability prior to
  multiply. Calling them "unsteered" would inflate the finding.
- **`library_prob`** has **no reader anywhere in `src/`** — one of the three
  documented-reserved orphan knobs recorded by `COVERAGE-INSTRUMENTATION.3`. A `KnobId`
  for a knob nothing rolls would be a paper category, which ROADMAP steering gap 1
  explicitly calls a regression.

A fourth thing the sweep found and this decision does **not** claim: `src/gen/module.rs:143`
rolls `g.rng.gen_bool(0.5)` with a hard-coded constant and no knob at all. It is a
construction-time choice with no dial, so it is out of scope here — but it is exactly the
kind of hidden bias ROADMAP steering gap 3 warns about, and it is recorded as an open
question rather than quietly passed over.

### 3. The two groups differ in roll granularity, and that matters

- A **motif** knob rolls **once per module** (`build_memory_leaf` or not). Its
  `attempts` count is therefore ~1 per module: a low-resolution but perfectly meaningful
  fire rate over a `--count` sweep.
- An **emit-projection** knob rolls **once per candidate gate** — hundreds of times per
  module. Its fire rate is a high-resolution measurement, and steering it is a genuinely
  fine-grained dial.

This is not a problem; it is the reason the readout is per-knob rather than a single
scalar. It does mean a per-category average over a mixed category would be dominated by
the high-attempt members — the exact trap that made `.3b`'s first regression-test draft
fail (a whole-`hierarchy` average was flooded by a zero-probability knob rolled thousands
of times). The taxonomy below keeps the two groups in **separate** categories partly for
that reason.

## Decision

### (a) Two new categories: `motifs` and `emission`

`KnobId::category()` grows from six values to eight:

| category | members | rationale |
| --- | --- | --- |
| `motifs` | the 7 module-level motif knobs | They select *what kind of module this is* — a memory, an FSM, a parameterized leaf, a multi-clock design. One dial for "make the corpus more motif-heavy" is a natural agent request. |
| `emission` | the 9 emit-projection knobs | They select *how an already-built cone is rendered*. Decision `0032` established these nine as one family with one shared invariant (mutual exclusion under a fixed pass order); they steer as one family too. |

Folding them into the existing six was rejected: `memory_prob` is not `state` (which
means flop-vs-gate leaf selection), and an emit projection is not `selectors` (which
means the mux/case/casez *construction* rolls, not their rendering). A category whose
members do unrelated things is a dial nobody can use.

The existing six categories keep exactly their current membership, so **every steering
config that works today keeps working and keeps meaning the same thing**.

### (b) The emit passes take `&SteeringConfig`

The nine `annotate_*` functions currently have the signature
`(m: &mut Module, rng: &mut impl Rng, prob: f64) -> usize`. They gain a
`steering: &SteeringConfig` parameter and call `roll_knob_into` instead of
`rng.gen_bool(prob)`.

`&SteeringConfig` rather than `&Config` (the `param::annotate_parameterized(&mut m, &g.cfg)`
precedent): it is exactly what the primitive needs, it keeps `ir::` from depending on the
whole config surface, and it makes the *call site* state which contract is being honoured.

Blast radius, counted: **99 in-crate call sites** across the nine, the large majority
being each pass's own `#[cfg(test)]` unit tests. This is mechanical but wide, which is why
`.4b` splits (below).

### (c) The `> 0.0` guard stays — and it is why the default stays byte-identical

Every one of the 16 sites is guarded by `if cfg.<knob> > 0.0` before the roll. That guard
is **load-bearing for reproducibility**, not an optimisation: with it, a default-off knob
consumes **no RNG draw**, so the whole default stream is unchanged. Remove it "for
cleanliness" and every default run would consume 16+ extra draws and every snapshot would
break.

It has a consequence worth stating plainly, because it will look like a bug to someone
reading the readout: **a default-off knob records `attempts = 0`, not "attempted, never
fired"**. The readout therefore cannot distinguish *"switched off"* from *"never
reached"*. That is the correct trade — byte-identity is a load-bearing project invariant
and readout resolution is not — and the ambiguity is resolvable by the reader, since the
effective config is in the same document.

Byte-identity of the routing itself is exact: `SteeringConfig::effective_prob`
short-circuits to `prob.min(1.0)` when unset, and **all 16 knobs are range-checked into
`[0.0, 1.0]`** by `Config::validate`'s probability list, so `prob.min(1.0) == prob`
bitwise. One `gen_bool` per roll is preserved (rules-first; no filter, no extra draw).

### (d) No introspection schema bump

`coverage_readout.knob_fire_rates` is documented as a **map**
(`docs/AGENT_INTROSPECTION_SCHEMA.md` §6.8); the schema does not enumerate the knob names,
and `KnobId` is not part of the schema surface. Widening the key set adds *data*, not
*shape*, so the contract is unchanged and the version stays put.

This is a deliberate narrowing of the lane's habit of MINOR-bumping for new facts. The bump
is for a new **payload key** or a new **query kind** — a consumer that has to learn
something. A consumer of `knob_fire_rates` already iterates a map whose membership depends
on the run's config; more entries is the same contract, honoured harder. Stating the rule
here means the next contributor does not have to re-litigate it.

Default `--introspect` output is unaffected regardless: with every one of the 16 knobs at
its `0.0` default the `> 0.0` guard means no roll, hence no map entry.

## Correction (`2026-07-30`, from the `.4b.1` recon — an explicit amendment, not a silent rewrite)

Implementation recon immediately after this record landed found **two facts it did not
account for**. Both are recorded here rather than edited away (`NEVER REWRITE HISTORY`);
the taxonomy, the exclusions, the schema call and the byte-identity argument all stand.

**1. Three of the seven motif rolls have no `Module` to record into.** §Decision (b) and
the `.4b.1` sketch assume every motif roll can be routed through
`Generator::roll_knob(&mut self, m: &mut Module, …)`. Measured in
`src/gen/module.rs::generate_leaf_module_with_interface_profile`: the
`width_parameterization_prob`, `memory_prob` and `fsm_prob` rolls happen at lines
385/401/414 and each **`return`s a differently-built module** (`build_parameterizable_leaf`
/ `build_memory_leaf` / `build_fsm_block`) at 390/404/417 — while the function's own first
`Module` binding is at **line 432**. The roll *chooses which module to construct*, so at
the moment of the roll there is nothing to record into. `fsm_mealy_prob`
(`build_fsm_block`, line 332) and the three `gen/mod.rs` rolls are fine — their module
already exists.

This is not a blocker, but it is design work `.4b.1` must do rather than mechanical
routing. The shape that fits the `0034` invariant: give `Generator` a small
`pending_knob_rolls: KnobRollCounters` that pre-module rolls record into, and drain it into
`m.knob_rolls` once the module is built — with the drain living **inside**
`ir::knob_roll`, so the "only this module writes the counters" property is preserved.
Rejected on sight: recording after the fact from the call site, which is precisely the
"roll here, record there" split decision `0034` exists to prevent (and which the guard
should make impossible — see below).

**2. The `.3b` R2 guard is incomplete.** `0034` privatised
`KnobRollCounters::record`, but left `attempts` and `fires` as `pub` fields. Measured
`2026-07-30` with a compile probe in `src/gen/hierarchy.rs`: a second roll primitive that
skips the steering prior and writes the maps **directly** —

```rust
let fired = rng.gen_bool(prob);                                  // no prior
*m.knob_rolls.attempts.entry(knob).or_insert(0) += 1;            // compiles
if fired { *m.knob_rolls.fires.entry(knob).or_insert(0) += 1; }  // compiles
```

— builds **clean** (`cargo check --all-targets` exit `0`). The guard blocks the obvious
route and not the equivalent one, which by its own stated principle (*guard the effect,
not the wrapper*) means it does not yet guard the effect. The fix is small — make the two
fields private with read-only accessors, since the only external consumer is
`metrics::compute` iterating them — and it is ordered **before** `.4b.1`, because `.4b.1`
introduces a *new* writer path (the pending-counter drain) and that path should be
designed against a complete guard rather than an incomplete one. Owned by a new leaf
`COVERAGE-STEERED-GENERATION.5`.

**What this says about the original guard.** `0034` chose R2 over R4 on the argument that
the type system enforces the property for free. That argument was right and the
*application* of it was half-done: privatising the method without privatising the fields
guards the API, not the invariant. A useful generalisation for the next R2 guard in this
repo: **enumerate every way the protected state can be written, not just the intended
one** — the same "search the effect, not the shape" rule `0034` itself established, turned
on the guard rather than on the defect.

## Decisive test applied

"Does an agent gain a capability it can act on, or only a longer list?" It gains one it can
act on: the nine emit-projection surfaces become **measurable** for the first time
(per-gate fire rates over hundreds of attempts) and **steerable** as one family, which is
exactly the loop decision `0032` says produces ANVIL's densest artifacts. The seven motif
knobs are lower-resolution but complete the API-completeness claim (decision `0017`) for
every capability that rolls.

## Rejected alternatives

- **Fold the 16 into the existing six categories.** Rejected — `memory_prob` is not
  `state`, and rendering is not selection. A category is a *dial a human asks for*; one
  with unrelated members cannot be asked for.
- **One new `capabilities` category for all 16.** Rejected — it merges roll granularities
  that differ by two orders of magnitude (per-module vs per-gate), so a per-category
  weight would be dominated by the emit knobs and a per-category *rate* would be
  meaningless. The `.3b` calibration failure is the direct evidence.
- **Pass `&Config` to the emit passes.** Rejected — `&SteeringConfig` is the exact
  dependency; widening it invites the next pass to reach for an unrelated knob and
  couples `ir::` to the config surface.
- **Drop the `> 0.0` guards so every knob always records an attempt.** Rejected — it
  consumes RNG draws on the default path and breaks byte-identity, which is a
  non-negotiable project invariant (`tests/snapshots.rs`). The readout ambiguity it would
  fix is cosmetic by comparison.
- **MINOR-bump the introspection schema.** Rejected, with the rule stated in (d): bump for
  a new key or query kind, not for more entries in an existing map.
- **Give `operand_duplication_rate` / `mux_arm_duplication_rate` a `KnobId`.** Rejected —
  there is no roll to apply a prior to. Recording them as "steerable" would be the
  paper-category regression ROADMAP steering gap 1 names.
- **Give `library_prob` a `KnobId`.** Rejected — nothing in `src/` reads it. A `KnobId`
  for an unrolled knob is a coverage entry that can never move.
- **Do all 16 in one slice.** Rejected — 99 call sites across two structurally different
  groups. Split below.

## Consequences

- `--steer` gains 16 knob names and 2 category names; the `unknown steer key` error's
  category list grows from six to eight, which is itself the discoverability surface.
- The nine structured-emission surfaces become measurable per-gate for the first time —
  the missing input to the measure→derive→re-steer loop over ANVIL's densest artifacts.
- `KnobId::all()` grows from 22 to **38** entries. That list is a hand-maintained shadow
  of the enum, and at 38 members it is well past the size where "a reviewer catches it"
  is a mechanism (decision `0033`). `.4b` must guard it — proposal in the open questions.
- Unsteered and default-config generation stay byte-identical; nothing is retired; no knob
  default changes.

## Open questions (to be resolved at `.4b`)

- **Guarding `KnobId::all()`.** Its doc comment argues an omission "just omits it from the
  roll-up, which a reviewer catches" — that is exactly the decision-`0033` *silent* test,
  and this decision doubles the list's length. Proposed repair (rung **R2**): add a
  private, **exhaustive** `fn index(self) -> usize` match — a new variant then fails to
  compile — plus a derived `#[test]` asserting `all()[k.index()] == k` for every `k` and
  `all().len() == max_index + 1`. A new variant cannot then be omitted from `all()`
  without a red test. To be confirmed against the real code at `.4b`.
- Whether the per-gate emit-projection knobs should record attempts for gates the pass
  *skips as ineligible* (a `Slice` under `function_emit`) or only for candidates it
  actually rolls on. Proposed: **only actual rolls**, matching every existing knob — a
  fire rate must stay `fires / rolls`, or it stops being comparable to the configured
  probability.
- `src/gen/module.rs:143`'s hard-coded `gen_bool(0.5)`: promote to a named knob (making it
  steerable and measurable) or leave it a structural constant? It is out of `.4`'s scope
  either way, but it should not stay unexamined — ROADMAP steering gap 3 is precisely
  about choices that fire by accident of implementation.
- Whether `--profile` presets should gain an emission-steering example now that the family
  is steerable as one dial (`structured-emission-max` sets the knobs directly; a steering
  weight is the orthogonal, coverage-driven route).

## Tree split

`COVERAGE-STEERED-GENERATION.4` splits, because the two groups differ in structure and the
emit group alone touches ~99 call sites:

- **`.4a`** (this leaf, design) — decision `0035`: the taxonomy, the signature change, the
  byte-identity argument, the schema call, the exclusions, and this split. Docs-only.
- **`.4b.1`** (`pending`) — the **7 motif knobs**. Self-contained: `src/gen/module.rs`
  (4 sites) + `src/gen/mod.rs` (3 sites, each duplicated across the two `generate_module*`
  entry points), 7 new `KnobId` variants, the `motifs` category, and the `KnobId::all()`
  guard from the open questions. Small enough to carry the guard work.
- **`.4b.2`** (`pending`) — the **9 emit-projection knobs**: the `&SteeringConfig`
  signature change across nine `annotate_*` functions and their ~99 call sites, 9 new
  `KnobId` variants, the `emission` category.
- **`.4c`** (`pending`) — docs/close: `book/src/algorithm.md` (eight categories),
  `book/src/knobs.md`, `book/src/structured-emission.md` (the family is now measurable),
  `USER_GUIDE.md`, the KM card; close `.4`.

## Links

- Owner doctrine: **DECIDE, DON'T ASK** (`MEMORY.md` standing directives, `2026-07-30`).
- Builds on: decision [`0034`](0034-one-steering-aware-knob-roll-primitive.md) (the one
  primitive `.4b` routes into — `.4` would have been unsafe before it, since 16 new knobs
  across two modules is exactly the pressure that produced the original fork), decision
  [`0023`](0023-coverage-steered-generation.md) (this is its recorded follow-up, *"routing
  the remaining raw `gen_bool` / weighted-choice sites through `roll_knob` so they gain
  telemetry **and** steerability together"*).
- Doctrine: `feedback_rules_first_generation` (a construction-time prior, one draw per
  roll, no filter), `feedback_full_factorization` (one primitive — `.4b` adds callers, not
  a second roll), `feedback_never_retire_strategies` (no knob default changes; the six
  existing categories keep their exact membership), decision
  [`0017`](0017-api-first-everything-mcp-accessible.md) (the API-completeness gate this
  closes for every capability that rolls), decision
  [`0033`](0033-shadow-enumeration-classification.md) (`KnobId::all()` at 38 entries).
- Lane / ROADMAP: steering gap 1 (dead knobs and paper-only categories are regressions —
  hence the three exclusions by kind) and gap 3 (the probability knobs must be exercised
  without hidden bias — hence flagging the unnamed `gen_bool(0.5)`).
- Reuse / touch points: `src/ir/types.rs` (`KnobId` + `all` + `category`),
  `src/ir/knob_roll.rs` (unchanged — it is the target), `src/gen/module.rs`,
  `src/gen/mod.rs`, the nine `src/ir/*_emit.rs` / `soft_union.rs` passes,
  `book/src/{algorithm,knobs,structured-emission}.md`, `USER_GUIDE.md`.

---

## Resolution — `2026-07-31` (`COVERAGE-STEERED-GENERATION.6`): the guard was at the wrong rung

The open question above proposed guarding `KnobId::all()` at rung **R2** — a private
exhaustive `index()` plus a derived ordering test. `.4b.1` shipped exactly that, and it
worked as designed: a new variant became `error[E0004]`, and a middle omission became a
red test. But its own negative control found the hole it could not close: **drop the
*last* entry from `all()` and the remaining indices are still contiguous and in order**,
so the test stays green.

The natural patch — assert a length — cannot work, and *why* it cannot is the durable
part. Any expected count **derived** from `all()` shrinks along with `all()`, so it
asserts nothing; and a **hand-written** count is a second hand-maintained copy of the
very list under guard, which decision [`0033`](0033-shadow-enumeration-classification.md)
forbids as a repair. Both routes fail, and that is the diagnostic:

> **When a guard's residual gap can only be closed by adding another hand-written list,
> the guard is at the wrong rung.** Stop reinforcing it and remove what it guards
> (rung **R1**).

`.6` does that. One `knob_ids!` macro table — `Variant => "name", "category";` — expands
to the `KnobId` enum, `all()`, `name()` and `category()`. Measured at
`git show HEAD:src/ir/knob_id.rs`, each of the 38 variant names appeared **5 times** in
the file, once per parallel table; it now appears **exactly once**. A knob that exists
cannot be missing from `all()`, because the same table emits both — so `index()` and
`all_is_complete_and_ordered` were **deleted**, not kept beside the macro (two mechanisms
for one job is `feedback_full_factorization`'s anti-pattern, and a retained guard would
imply the omission is still expressible).

Two consequences worth recording:

- **The authority moved, so the doctrine check moved with it, in the same commit — and
  the reason is not the one initially assumed.** `ENUMERATION-PARITY` pair 4 extracts the
  `--steer` taxonomy from `src/ir/knob_id.rs`. The expectation was that an extractor left
  on `pub fn category` would read **zero** categories (its body now holds `$category`,
  not literals) and trip the count floor loudly. **Measured, it reads the correct 8** —
  which is worse than zero, and is why the repoint is load-bearing rather than tidy. Two
  accidents stack: its range terminator `/^    }$/` no longer exists where it used to (the
  macro definition closes `    };`, the invocation closes `}` at column 0), so the range
  **over-runs 162 lines** and swallows the table on its way to `category_of_name`'s brace;
  and `grep -oE '"[a-z]+"'` over that over-run then skips the knob *names* only because
  every one of them happens to contain `_`. Right answer, wrong reason, one row deep: a
  knob named `"probe"` makes it emit a **phantom** category that fails at every doc site
  for something that does not exist — the cry-wolf failure that gets a gate deleted.
  Verified by probe. The general rule earned here: **a `sed` line-range whose terminator
  stops existing does not fail; it runs on and returns something plausible.**
  The check now parses the table's third column, matching a whole row
  (`Ident => "name", "category";`) rather than scanning for quoted words. That also
  retires
  [`PARITY-EXTRACTOR-ARM-SHAPE-GAP`](../tasks/PARITY-EXTRACTOR-ARM-SHAPE-GAP.md)'s hazard
  at its root instead of working around it: `rustfmt` does not format this macro
  invocation's body at all — measured, the longest row is **113 characters** and survives
  `cargo fmt` untouched, well past the 100-column `max_width` it would have wrapped if it
  owned the text. The layout is a source fact chosen by the author, which is exactly what
  that tree concluded an extractor must read.
- **The remaining test is the one that still has something to catch.**
  `knob_names_and_categories_are_disjoint_and_total` survives, because a *row* can still
  carry a duplicated or colliding string — and `--steer`'s key classifier depends on knob
  names and category names being disjoint. Structure is now enforced by construction;
  strings are not, so they are still asserted.
