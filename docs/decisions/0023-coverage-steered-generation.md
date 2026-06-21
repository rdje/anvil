---
id: coverage-steered-generation
title: Construction-time coverage steering — a deterministic per-category probability-prior multiplier at the roll_knob site (rules-first, never generate-then-filter), an outer measure→derive→re-steer feedback loop, with an API-settable target + API-queryable achieved coverage (decision 0017)
answers:
  - "can ANVIL bias generation toward under-exercised constructs"
  - "does ANVIL have coverage-steered or coverage-feedback generation"
  - "how does ANVIL steer generation without generate-then-filter"
  - "is coverage steering rules-first or post-hoc filtering"
  - "how do I make ANVIL emit more of a specific construct"
  - "can I set a coverage target over the ANVIL API"
  - "how do I query ANVIL's achieved construct coverage"
  - "does coverage steering stay reproducible and byte-stable"
  - "what is the ANVIL steering-config"
date: 2026-06-21
status: accepted
tags: [coverage, steering, generation, rules-first, reproducible, byte-identical, mcp, api, knobs, roll-knob, north-star, effectiveness, design]
evidence: docs/decisions/0023-coverage-steered-generation.md; docs/decisions/0017-api-first-everything-mcp-accessible.md; docs/decisions/0011-semantic-introspection-derived-query-surface.md; src/gen/cone.rs (the roll_knob site); src/ir/types.rs (KnobId + knob_rolls telemetry); src/metrics.rs (knob_roll_attempts/knob_roll_fires + gate/operand/depth histograms); docs/tasks/COVERAGE-STEERED-GENERATION.md
---

# 0023 - Coverage-steered generation: a construction-time prior, not a filter

- Date: 2026-06-21
- Status: accepted (design; implementation pending under the pre-split `.2`)
- Tree: `COVERAGE-STEERED-GENERATION.1` (design/decision leaf; no code yet).
- Binds: decision [`0017`](0017-api-first-everything-mcp-accessible.md) (the
  steering **target** is API-settable; the **achieved coverage** is
  API-queryable; the CLI is a shim over that surface).
- Reuses the readout precedent of decision
  [`0011`](0011-semantic-introspection-derived-query-surface.md) (a
  SCHEMA-DERIVED projection of existing metrics — **zero new computed truth**).
- Bounded by `feedback_rules_first_generation` (the load-bearing doctrine):
  steering is a **construction-time prior**, never a post-hoc filter.

## Context

ANVIL generates uniform-random-ish legal RTL: every construction-time choice is a
seeded roll against a fixed knob probability. That finds bugs, but it spends most
of its draws on common constructs and under-exercises rare ones (a deep `casez`
fold inside a registered hierarchy child input, say). The north star
(`project_anvil_north_star` — surface downstream-tool bugs) is served by
**goal-directed** exploration: bias the corpus toward under-hit constructs so a
fixed runtime budget probes more of the legal design space.

The hard constraint is the project's first doctrine: **rules-first, no
generate-then-filter** (`feedback_rules_first_generation`). A coverage-feedback
loop must therefore bias the *construction-time choice distribution* — it may
**never** build artifacts and discard the ones that miss the target. The other
hard constraint is reproducibility: a given `(seed, knobs, steering-config)` must
stay byte-identical forever, and the unsteered default must be byte-identical to
today.

The instrumented decision surface already exists. Every steerable choice flows
through one site:

```rust
// src/gen/cone.rs
fn roll_knob(g: &mut Generator, m: &mut Module, knob: KnobId, prob: f64) -> bool {
    let fired = g.rng.gen_bool(prob.min(1.0));   // exactly one RNG draw
    m.knob_rolls.record(knob, fired);            // telemetry: attempts + fires
    fired
}
```

…and the achieved distribution is already recorded as `Metrics.knob_roll_attempts`
/ `knob_roll_fires` (per `KnobId`) plus the gate-kind / operand-arity / depth
histograms. So both the steering hook and the coverage readout reuse machinery
that is already there.

## Decision

### 1. The steering primitive — a probability-prior multiplier at `roll_knob`

Steering multiplies the knob probability **before** the single `gen_bool` draw:

```
effective_prob = clamp01( prob * weight(knob) )      // weight defaults to 1.0
fired          = rng.gen_bool(effective_prob)         // still exactly ONE draw
```

`weight(knob)` is a deterministic lookup in the steering-config (per-`KnobId`,
falling back to a per-category default, falling back to `1.0`). This is the whole
mechanism, and it is **rules-first by construction**:

- It biases the **prior** of a decision; it does not build-then-discard. There is
  no rejection path, no second artifact, no filter. (Contrast: a filter would
  generate a module, measure it, and throw it away if off-target — forbidden.)
- The **RNG draw count is unchanged** — exactly one `gen_bool` per `roll_knob`,
  exactly as today — so output stays byte-stable per `(seed, knobs,
  steering-config)`.
- When no steering-config is supplied, every `weight` is `1.0` ⇒ `effective_prob
  == prob` **exactly** ⇒ the default is byte-identical to today (`tests/snapshots.rs`
  untouched).

First cut steers only the `roll_knob`-mediated knobs (the `KnobId` set — the
instrumented surface). Raw `gen_bool` sites (`src/gen/mod.rs`) and weighted-choice
sites (`gate_struct_weight`) are **out of first-cut scope**; routing them through
`roll_knob` (so they gain telemetry *and* steerability together) is a recorded
follow-up. This keeps the rules-first surface clean and the proof bounded.

### 2. The coverage-target model — a `SteeringConfig`

A declarative `SteeringConfig`: a map from a **coverage category** to a non-negative
**emphasis weight** (`> 1` up-weights, `< 1` down-weights, `1.0`/absent = neutral).
Categories are keyed by names that already exist:

- per-knob: the `KnobId::name()` strings (`flop_prob`, `casez_mux_prob`,
  `hierarchy_registered_child_input_cone_prob`, …);
- per-category roll-ups: a small fixed taxonomy (`structured-selectors`,
  `hierarchy-routing`, `sharing`, `datapath-motifs`, …) mapping to a set of
  `KnobId`s, so a user can up-weight a whole family in one line.

It rides alongside `Config` (its own block, default empty ⇒ neutral), so it is
already `--config`-settable and MCP-settable; a `--steer cat=weight` CLI shim and
a curated steering preset are ergonomics on top (the decision `0021` `--profile`
precedent). The steering-config is the **durable, reproducible artifact** a CI
finding or a sweep is pinned to.

### 3. The achieved-coverage readout — SCHEMA-DERIVED, zero new truth

A read-only projection of existing telemetry (the decision `0011` precedent):
per-knob empirical fire rate (`knob_roll_fires / knob_roll_attempts`), the
gate-kind / operand-arity / depth histograms, and the `CoverageSummary saw_*`
facts. Surfaced through `--introspect` and an MCP query (extend `analyze` or add a
`coverage` query), so an agent reads *what was actually exercised* and computes the
next steering-config. No new computed truth — it is the numbers the generator
already records.

### 4. The feedback loop — an OUTER measure→derive→re-steer loop

The "feedback" is **not** an in-generator loop (that would risk a filter or break
per-unit determinism). It is an outer, deterministic loop, exactly mirroring how
`coverage_gaps` already works (measure gaps → act):

1. **Measure** — run a sweep; read the achieved-coverage readout (§3).
2. **Derive** — a pure deterministic function maps under-hit categories to an
   up-weighted `SteeringConfig` (e.g. `weight = clamp(target_share /
   max(observed_share, eps))`).
3. **Re-steer** — re-run with that steering-config; byte-stable per `(seed, knobs,
   steering-config)`.

Each generation pass stays a pure, rules-first function of its inputs; the
feedback lives in the orchestration, not the generator. This is the cleanest
reconciliation with `feedback_rules_first_generation`.

### 5. API-completeness (decision 0017)

- **Target settable** via the config JSON `steering` block + MCP `generate`/
  `validate`/`hunt` config inputs + the `--steer` CLI shim — the same controls an
  MCP agent would set, no CLI-only path.
- **Achieved coverage queryable** via `--introspect` + the MCP coverage query —
  SCHEMA-DERIVED, so the catalog/introspection schema gains the readout, not new
  truth.

## Pre-split of `.2` (implementation)

- `.2a` — the **steering core**: the `SteeringConfig` type + the `weight()` lookup
  + the `roll_knob` prior multiplier, with the two load-bearing proofs — (i)
  **byte-identical when unset** (snapshots 6/6, `tests/snapshots.rs` untouched),
  and (ii) **measurable distribution shift** vs unsteered on a fixed seed sweep
  (the achieved fire-rate of an up-weighted category rises) — plus a **no-filter**
  proof (the construction path has no rejection branch; draw count per roll
  unchanged).
- `.2b` — the **achieved-coverage readout**: the SCHEMA-DERIVED projection in
  `--introspect` (schema MINOR bump) + the MCP coverage query (decision `0017`),
  with the byte-identical-elsewhere guarantee.
- `.2c` — the **outer measure→derive→re-steer** convenience (a deterministic
  `derive_steering_from_coverage` helper) + book (`algorithm.md` steering
  subsection + `agent-mcp.md`) + USER_GUIDE + a KM card; close `.2`.

## Rejected alternatives

- **Generate-then-filter / post-hoc rejection.** The forbidden mode
  (`feedback_rules_first_generation`): build a corpus, measure it, discard
  off-target artifacts. Rejected outright — steering is a construction-time prior,
  never a filter. This is the decision's whole point.
- **An in-generator adaptive schedule as the first cut** (recompute weights after
  each unit within one `--count` run, nudging toward under-hit constructs). More
  powerful "online" feedback, and still byte-stable per `(seed, knobs,
  steering-config, count)` because the schedule is a deterministic function of
  prior units — but it **couples units within a run** (unit N's output depends on
  the count and order), losing per-unit independence and complicating the
  reproducibility story. Deferred to a follow-up `.N`; the outer loop (§4) gets the
  same benefit with a simpler contract first.
- **Steering raw `gen_bool` / weighted-choice sites in the first cut.** They lack
  telemetry, so steering them blind is unprovable. Deferred until they are routed
  through `roll_knob` (gaining telemetry + steerability together).
- **A behavioural-coverage target.** Non-goal: coverage is over *structural*
  constructs (gate kinds, motifs, emission surfaces, hierarchy/identity features),
  never behaviour — ANVIL ships no behavioural oracle.
- **A new RNG stream or extra draws for steering.** Rejected: extra draws would
  break byte-stability vs today's unsteered path. The multiplier reuses the single
  existing draw.

## Consequences

- `.2` adds a `SteeringConfig` + a one-line prior multiplier at `roll_knob` + a
  SCHEMA-DERIVED coverage readout + an outer steering-derivation helper — all
  default-off / DUT byte-identical, all rules-first, all reproducible per `(seed,
  knobs, steering-config)`.
- ANVIL moves from uniform-random toward goal-directed exploration of the legal
  design space **without** weakening any lane invariant — directly serving the
  north star (find more downstream-tool bugs per unit of runtime), and composing
  with `anvil hunt` / the CI Action (steer the corpus a CI fuzz run explores via a
  `--profile`-like steering preset).

## Links

- Owning tree: `docs/tasks/COVERAGE-STEERED-GENERATION.md` (this is its `.1` leaf;
  pre-splits `.2a`/`.2b`/`.2c`).
- Bound/precedent decisions: `0017` (API-completeness), `0011` (SCHEMA-DERIVED
  readout), `0021` (preset ergonomics for the steering-config).
- Memory: `feedback_rules_first_generation` (the load-bearing boundary — steering
  is a prior, not a filter), `project_anvil_north_star`,
  `feedback_api_for_agents_not_humans`.
