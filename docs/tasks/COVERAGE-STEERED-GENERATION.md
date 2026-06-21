# COVERAGE-STEERED-GENERATION: construction-time coverage-feedback steering

## Metadata

- Tree ID: `COVERAGE-STEERED-GENERATION`
- Status: `active`
- Roadmap lane: `Usability / effectiveness — coverage-steered generation (north star, idea 6)`
- Created: `2026-06-17`
- Last updated: `2026-06-21` (`.2c.1` derive helper + `--steer` landed; frontier `.2c.2`)
- Owner: repo-local workflow

## Goal

Find bugs faster by **biasing generation toward under-exercised constructs** —
but strictly at **construction time**, by adjusting the seeded construction-time
choices (the `roll_knob` decision sites), **never** by generate-then-filter. A
coverage target (which constructs/categories/surfaces to emphasize) and the
achieved coverage are both first-class, API-settable and API-queryable. This
turns ANVIL from uniform-random toward goal-directed exploration of the legal
design space while preserving every lane invariant.

## Non-Goals

- **No generate-then-filter / no post-hoc rejection** (`feedback_rules_first_generation`
  — the load-bearing doctrine). Steering biases the *construction-time* choice
  distribution; it never builds-then-discards.
- No behavioural oracle; coverage is over *structural* constructs (gate kinds,
  motifs, emission surfaces, hierarchy/identity features), not behaviour.
- No break to reproducibility: a given `(seed, knobs, steering-config)` stays
  byte-identical; default (no steering) is byte-identical to today.

## Acceptance Criteria

- A steering mechanism biases construction-time rolls toward a named coverage
  target and measurably shifts the achieved construct distribution vs. unsteered,
  on a seed sweep — while staying rules-first (no filtering) and reproducible.
- **API-completeness gate (decision `0017`):** the coverage **target** is
  settable via the MCP/config API and the **achieved coverage** is queryable via
  the MCP/introspection API (SCHEMA-DERIVED — projected from the existing
  metrics/knob-roll telemetry + the construct histograms). The CLI is a shim over
  the same surface.
- Default-off / DUT byte-identical (unsteered output unchanged); the byte-stable
  contract holds per `(seed, knobs, steering-config)`; downstream-clean.
- Documented in `book/src/algorithm.md` (or a steering subsection) +
  `book/src/agent-mcp.md` + USER_GUIDE; committed through `COMMIT.md`.

## Task Tree

- ID: `COVERAGE-STEERED-GENERATION`
  Status: `active`
  Goal: `Construction-time coverage-feedback steering (rules-first, reproducible) with an API-settable coverage target + an API-queryable achieved-coverage readout.`
  Children: `COVERAGE-STEERED-GENERATION.1`

- ID: `COVERAGE-STEERED-GENERATION.1`
  Status: `done`
  Goal: `Design/decision leaf (ADR, no code): pin HOW coverage feedback biases construction WITHOUT generate-then-filter (e.g. per-category/per-surface weight multipliers applied to the existing roll_knob decision sites; or a deterministic schedule across a --count run that nudges weights toward under-hit constructs) while keeping byte-stability per (seed, knobs, steering-config); define the coverage-target model + the achieved-coverage readout (reuse knob_roll_attempts/fires + gate/category/surface histograms in Metrics); pin the MCP target-set + coverage-query surface (decision 0017); and EXPLICITLY reconcile with feedback_rules_first_generation (steering is a construction-time prior, not a post-hoc filter). Record as the next decision record + pre-split .2 (impl).`
  Acceptance: `A decision record + a tree/DEVELOPMENT_NOTES entry pinning the rules-first steering model, the reproducibility contract, the coverage target/readout, and the MCP surface; docs-only; INDEX + this tree + docs/TASK_TREE.md updated.`
  Verification: `done — decision 0023: the steering primitive is a deterministic per-category probability-prior MULTIPLIER on prob at the roll_knob site (effective_prob = clamp01(prob * weight), one gen_bool draw preserved) — rules-first (a construction-time prior, NOT a filter; no rejection path) and byte-stable per (seed, knobs, steering-config), byte-identical when unset (weight=1.0). Coverage-target = a SteeringConfig (KnobId / category → emphasis weight); achieved-coverage readout = SCHEMA-DERIVED from knob_roll_attempts/fires + histograms (zero new truth, decision 0011); feedback = an OUTER measure→derive→re-steer loop (not in-generator); API target-set + coverage-query per decision 0017. In-generator adaptive schedule + raw gen_bool/weighted-choice sites + behavioural coverage explicitly rejected/deferred. Pre-split .2a/.2b/.2c. INDEX + tree + TASK_TREE + DEVELOPMENT_NOTES updated; KM regen; docs-only / DUT byte-identical.`
  Commit: `COVERAGE-STEERED-GENERATION.1 — design ADR (decision 0023)`

- ID: `COVERAGE-STEERED-GENERATION.2`
  Status: `pending`
  Goal: `Implement the .1 design (decision 0023). Pre-split: .2a (the SteeringConfig + weight() lookup + the roll_knob prior multiplier + byte-identical-when-unset + distribution-shift + no-filter proofs), .2b (the SCHEMA-DERIVED achieved-coverage readout in --introspect + the MCP coverage query), .2c (the outer measure→derive→re-steer helper + book/USER_GUIDE/KM; close).`
  Acceptance: `set at .1 (decision 0023): a per-category prior multiplier at roll_knob that measurably shifts the achieved construct distribution vs unsteered on a seed sweep while staying rules-first (no filter path) and byte-stable per (seed, knobs, steering-config); unsteered default byte-identical; the coverage target settable + the achieved coverage queryable over the MCP/config API (CLI a shim); downstream-clean.`
  Verification: `pending`
  Commit: `pending`

  Children: `COVERAGE-STEERED-GENERATION.2a` (steering core), `.2b` (coverage readout + MCP query), `.2c` (outer loop + docs + close).

- ID: `COVERAGE-STEERED-GENERATION.2a`
  Status: `done`
  Goal: `The steering CORE (code): a SteeringConfig type (per_knob/per_category emphasis weights) + the weight() lookup + the roll_knob prior multiplier (effective_prob = clamp01(prob * weight), one gen_bool draw preserved), with the three load-bearing proofs.`
  Acceptance: `(i) byte-identical when unset (tests/snapshots.rs 6/6 untouched); (ii) measurable distribution shift vs unsteered on a fixed seed sweep (up-weighted category's empirical fire-rate rises); (iii) no-filter (one gen_bool per roll, no rejection branch); weights validated finite & >= 0.0; full COMMIT.md cargo gate green.`
  Verification: `done — KnobId::category() (exhaustive 21-variant match → state/selectors/datapath/terminals/sharing/hierarchy); SteeringConfig in config.rs (per_knob/per_category BTreeMaps + weight()/effective_prob()/is_empty()/validate()); Config.steering field (the only skip_serializing_if ⇒ empty omitted ⇒ --dump-config/--introspect byte-identical when unset); ConfigError::SteeringWeight; roll_knob applies effective_prob before the single gen_bool. Proofs: snapshots 6/6 (byte-identical default); steering_shifts_achieved_construct_distribution (flop_prob fire-rate rises >0.1 over a 40-seed sweep when category "state" is up-weighted 4x); neutral_steering_weight_is_byte_identical_to_unsteered (explicit weight 1.0 = byte-identical SV across 16 seeds, proving the multiplier is exact at 1.0, not just the short-circuit); 6 config unit tests (weight resolution, neutral exactness, clamp, validation accept/reject, serde omission). Full gate green: cargo check --all-targets, cargo test, cargo clippy --all-targets -D warnings, cargo fmt --check. Rules-first / DUT byte-identical when unset.`
  Commit: `COVERAGE-STEERED-GENERATION.2a — steering core (SteeringConfig + roll_knob prior multiplier)`

- ID: `COVERAGE-STEERED-GENERATION.2b`
  Status: `done`
  Goal: `The achieved-coverage READOUT: a SCHEMA-DERIVED projection of knob_roll_attempts/fires + the gate/operand/depth histograms in --introspect (schema MINOR bump) + an MCP coverage query (decision 0017), with the byte-identical-elsewhere guarantee.`
  Acceptance: `set at .1 (decision 0023).`
  Verification: `done — src/introspect/coverage.rs: CoverageReadout (knob_fire_rates + category_fire_rates maps of KnobCoverage{attempts,fires,fire_rate} + gate_kind/operand/depth histograms) + module_coverage(&Metrics) / design_coverage(&[Metrics]) (cross-child aggregate). Pure projection of the Metrics already recorded — SCHEMA-DERIVED, zero new truth (decision 0011). KnobId::all() + category_of_name() in types.rs (the single name→category inversion). Embedded as IntrospectionPayload::coverage_readout (skip_serializing_if=Option::is_none) on DUT module/design docs; standalone CoverageDocument + coverage_document() for the MCP coverage tool (run_coverage reuses the embedded readout — one projection, not two). SCHEMA_VERSION 1.11→1.12; schema doc §5 row + §6.8 + changelog. Determinism: fire_rate computed as round-half-up integer-ppm then one exact u64→f64/1e6 (a raw f64 division diverged by 1 ULP between the MCP build path and a recompute, caught by the pre-existing introspect_tool_round_trips exact-equality test — fixed, test NOT weakened). Full COMMIT.md cargo gate green (check/test (snapshots 6/6 + new coverage unit + introspect/mcp coverage tests)/clippy -D warnings/fmt). DUT .sv byte-identical; --dump-config byte-identical (no Config change); only the --introspect/MCP-introspect docs gain coverage_readout.`
  Commit: `COVERAGE-STEERED-GENERATION.2b — achieved-coverage readout (--introspect section + MCP coverage query)`

- ID: `COVERAGE-STEERED-GENERATION.2c`
  Status: `pending`
  Goal: `The outer measure→derive→re-steer convenience (a deterministic derive_steering_from_coverage helper) + the --steer CLI shim + book (algorithm.md steering subsection + agent-mcp.md) + USER_GUIDE + a KM card; close .2.`
  Acceptance: `set at .1 (decision 0023).`
  Verification: `pending`
  Commit: `pending`

  Pre-split (introduced `2026-06-21` at the start of `.2c`, matching how `.2`
  itself was split, to keep the code slice and the load-bearing book slice each
  signoff-sized): `.2c.1` (CODE — the `derive_steering_from_coverage` helper + the
  `--steer` CLI shim, with proofs) then `.2c.2` (DOCS — book steering subsection +
  `agent-mcp.md` coverage/steering + USER_GUIDE + KM card; closes `.2c` and `.2`).
  Children: `COVERAGE-STEERED-GENERATION.2c.1`, `.2c.2`.

- ID: `COVERAGE-STEERED-GENERATION.2c.1`
  Status: `done`
  Goal: `The outer-loop CODE: (a) a pure deterministic derive_steering_from_coverage(&CoverageReadout, params) -> SteeringConfig helper (decision 0023 §4: per-category weight = clamp(target_share / max(observed_share, eps), 0, max_weight); emit only non-neutral weights to keep the SteeringConfig minimal); (b) the --steer <key>=<weight> repeatable CLI shim that fills Config.steering.per_category (known category) / per_knob (known KnobId name), validated. The MCP/--config target-set already exists (SteeringConfig is part of Config); this is the ergonomic shim + the derive convenience.`
  Acceptance: `derive helper is pure + deterministic (byte-identical for the same (readout, params)) and produces an up-weight for an under-hit category and ~neutral for an at-target one; --steer parses key=weight into the right map, errors on a bad weight / unknown key, and composes with --config/--profile in the documented resolution order; unsteered default byte-identical (no --steer ⇒ empty steering ⇒ DUT byte-identical); full COMMIT.md cargo gate green.`
  Verification: `done — src/introspect/coverage.rs: derive_steering_from_coverage(&CoverageReadout, &DeriveParams) -> SteeringConfig (per-category weight = clamp(target_share/max(observed,eps), 0, max_weight), milli-quantized for byte-stability, neutral weights omitted) + DeriveParams (target_share/max_weight/epsilon, neutral default). Pure read→config function — no generation, no filter (feedback in the orchestration, feedback_rules_first_generation). src/config.rs: SteeringConfig::set_weight (knob name → per_knob / category → per_category / else ConfigError::UnknownSteerKey, reusing KnobId::category_of_name + all — one classifier), validate() made pub, Overrides.steer (skip_serializing_if), resolve_config applies preset-then-explicit steer before validate. src/main.rs: --steer <key>=<weight> repeatable flag + parse_steer_arg + cli_overrides mapping. Proofs: 3 derive tests (up-weight under-hit + neutralize at-target + omit; clamp zero-fire to max_weight; deterministic + milli-quantized), 4 config tests (set_weight classify; resolve_config applies steer; unknown-key error; negative-weight error), 2 main tests (parse_steer_arg; --steer end-to-end resolves). CLI smoke: --steer state=8 ≠ unsteered SV; --steer state=1.0 byte-identical to unsteered; --steer bogus=2 errors naming categories. Full gate green: cargo check --all-targets, cargo test (snapshots 6/6), cargo clippy -D warnings, cargo fmt --check. Unsteered default DUT byte-identical.`
  Commit: `COVERAGE-STEERED-GENERATION.2c.1 — outer-loop derive helper + --steer CLI shim`

- ID: `COVERAGE-STEERED-GENERATION.2c.2`
  Status: `pending`
  Goal: `The DOCS + close: book steering subsection in algorithm.md + the coverage/steering surfaces in agent-mcp.md (incl. the schema 1.12 coverage_readout example refresh + the coverage MCP tool) + USER_GUIDE (the --steer shim + the measure→derive→re-steer recipe) + a KM card for the steering capability; mark .2c, .2, and re-evaluate the tree. Refreshes the owner-deferred steering-lane book/USER_GUIDE drift recorded at .2a/.2b.`
  Acceptance: `book + USER_GUIDE accurately document steering (the prior multiplier, the SteeringConfig target, the coverage readout, the coverage MCP tool, --steer, and the outer loop) with runnable examples; the schema-1.12 example refresh lands; KM regen+check green; mdbook build clean; .2c + .2 marked done; INDEX/tree/TASK_TREE updated.`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `COVERAGE-STEERED-GENERATION.1` | `done` | Design ADR (decision `0023`) pinned the rules-first steering primitive (a prior multiplier at `roll_knob`, not a filter), the byte-stability contract, the `SteeringConfig` target model, the SCHEMA-DERIVED achieved-coverage readout, the outer measure→derive→re-steer loop, and the decision-`0017` API surface. |
| 2 | `COVERAGE-STEERED-GENERATION.2a` | `done` | Steering core landed: `KnobId::category()`, `SteeringConfig` + `weight()`/`effective_prob()`, the `roll_knob` prior multiplier, `ConfigError::SteeringWeight`. All three proofs green (byte-identical default via snapshots 6/6; measurable distribution shift; no-filter architectural) + full cargo gate. |
| 3 | `COVERAGE-STEERED-GENERATION.2b` | `done` | Achieved-coverage readout landed: `src/introspect/coverage.rs` (`CoverageReadout` + per-knob/per-category fire rates + the gate/operand/depth histograms), embedded as `IntrospectionPayload::coverage_readout` (schema `1.11→1.12`) + the standalone MCP `coverage` query (`CoverageDocument`), one projection feeding both. SCHEMA-DERIVED / DUT `.sv` byte-identical; `fire_rate` integer-ppm for byte-stable determinism. Full cargo gate green. |
| 4 | `COVERAGE-STEERED-GENERATION.2c.1` | `done` | Outer-loop CODE landed: the pure `derive_steering_from_coverage(&CoverageReadout, &DeriveParams) -> SteeringConfig` helper (decision `0023` §4, milli-quantized weights) + `SteeringConfig::set_weight` + the repeatable `--steer <key>=<weight>` CLI shim into `Config.steering` (preset-then-explicit in `resolve_config`). Unsteered default byte-identical (snapshots 6/6); CLI smoke proves steered≠unsteered, neutral=unsteered, bad-key errors. Full cargo gate green. |
| 5 | `COVERAGE-STEERED-GENERATION.2c.2` | `pending` | **Next.** The DOCS + close: book steering subsection (`algorithm.md`) + `agent-mcp.md` (coverage/steering + the schema `1.12` example refresh) + USER_GUIDE + a KM card; mark `.2c` + `.2` done. Closes the owner-deferred steering-lane doc drift. Task-tree-owned. |

## Decisions

- `2026-06-17`: Registered as an owner-directed usability/effectiveness lane
  (idea 6). Binds decision [`0017`](../decisions/0017-api-first-everything-mcp-accessible.md)
  (API-settable target + API-queryable coverage) and is explicitly bounded by
  `feedback_rules_first_generation` (construction-time prior, never
  generate-then-filter). Design-first ADR before code.
- `2026-06-21` (`.1`): Design ADR landed as decision
  [`0023`](../decisions/0023-coverage-steered-generation.md): the steering
  primitive = a deterministic per-category probability-prior **multiplier** on
  `prob` at the `roll_knob` site (`effective_prob = clamp01(prob * weight)`, one
  `gen_bool` draw preserved) — rules-first (a construction-time prior, not a
  filter) and byte-stable per `(seed, knobs, steering-config)`, byte-identical
  when unset. Target = a `SteeringConfig` (per-`KnobId` / per-category emphasis
  weights); achieved-coverage readout = SCHEMA-DERIVED from
  `knob_roll_attempts`/`fires` + histograms (zero new truth); feedback = an
  **outer** measure→derive→re-steer loop. Pre-split `.2a`/`.2b`/`.2c`.

## Open Questions

- The steering primitive: per-roll weight multipliers vs. a deterministic
  per-`--count` schedule vs. a seeded distribution prior — which best biases
  construction while staying byte-stable per `(seed, knobs, steering-config)`.
  *(Resolved at `.1` / decision `0023`: a per-category probability-prior
  multiplier on `prob` at `roll_knob`, one draw preserved. The in-`--count`
  adaptive schedule is deferred to a follow-up `.N` — it couples units within a
  run; the outer measure→derive→re-steer loop gives the feedback benefit with a
  simpler reproducibility contract first.)*
- Whether steering targets categories, emission surfaces, or both, and how the
  target is expressed in the API. *(Resolved at `.1`: a `SteeringConfig` keyed by
  the existing `KnobId::name()` strings + a small fixed category taxonomy, settable
  via the `--config` JSON `steering` block + MCP + a `--steer` CLI shim.)*

## Implementation Notes (for `.2a` — captured during the `.1` design pass)

A pre-implementation code survey, recorded so `.2a` lands clean (continuity):

- **Single integration point.** All 31 steerable rolls funnel through one function,
  `roll_knob(g, m, knob, prob)` at `src/gen/cone.rs:42` (`g.rng.gen_bool(prob.min(1.0))`
  + `m.knob_rolls.record(knob, fired)`). `.2a` changes ONLY this function:
  `let w = g.cfg.steering.weight(knob); let eff = (prob * w).clamp(0.0, 1.0);` then
  `gen_bool(eff)`. No call site changes. For `prob ∈ [0,1]` and `w == 1.0`,
  `(prob*1.0).clamp(0,1) == prob` exactly (IEEE754) ⇒ byte-identical default
  (snapshots 6/6 prove it).
- **`SteeringConfig` type.** `per_knob: BTreeMap<String,f64>` (keyed by
  `KnobId::name()`) + `per_category: BTreeMap<String,f64>` + `weight(KnobId)->f64`
  (per-knob → per-category → `1.0`) + `is_empty()`. Add `KnobId::category()` next to
  `KnobId::name()` in `src/ir/types.rs` (suggested taxonomy: `state`, `selectors`,
  `datapath`, `terminals`, `sharing`, `hierarchy`).
- **Byte-identity of serialized outputs.** `config.rs` has **zero**
  `skip_serializing_if` today (every knob always serializes). Add the field as
  `#[serde(default, skip_serializing_if = "SteeringConfig::is_empty")]` so an empty
  steering block is OMITTED ⇒ `--dump-config` + `--introspect` stay byte-identical
  when unset, and the introspection schema version bump is deferred to `.2b` (the
  readout), per decision `0023`.
- **`Config::default`** is an explicit `impl Default for Config` at
  `src/config.rs:1012` — add the field there (default empty `SteeringConfig`).
- **Validation.** Add a non-negative-weight check (weights `>= 0.0`, finite) in the
  `Config` validation path (mirror the existing prob-range validation), returning a
  `ConfigError`.
- **Proofs.** (i) byte-identical-when-unset = existing `tests/snapshots.rs` 6/6
  untouched; (ii) distribution-shift = generate with a category up-weighted and
  assert `knob_roll_fires[knob]/attempts` rises vs unsteered on a fixed seed;
  (iii) no-filter = architectural (one `gen_bool` per roll, no rejection branch).
- **Gate.** `.2a` is a generator code change ⇒ run the full `COMMIT.md` gate
  (`cargo check --all-targets`, `cargo test`, `cargo clippy --all-targets -- -D
  warnings`, `cargo fmt --all --check`); watch RAM per `0003-resource-safe-validation`.

## Blockers

- None. (Reuses the existing `knob_roll_attempts`/`fires` + histogram telemetry;
  the rules-first boundary is a design constraint, not a blocker.)

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-06-17` | `COVERAGE-STEERED-GENERATION` | `tree registered (docs-only); no code` | `registered` |
| `2026-06-21` | `COVERAGE-STEERED-GENERATION.1` | `decision 0023 written; INDEX + tree + TASK_TREE + DEVELOPMENT_NOTES updated; KM regen+check green; mem-arch green; docs-only / DUT byte-identical` | `done` |
| `2026-06-21` | `COVERAGE-STEERED-GENERATION.2a` | `SteeringConfig + KnobId::category() + roll_knob prior multiplier + ConfigError::SteeringWeight; cargo check --all-targets, cargo test (snapshots 6/6 + new steering unit/integration tests), cargo clippy -D warnings, cargo fmt --check all green; rules-first / DUT byte-identical when unset` | `done` |
| `2026-06-21` | `COVERAGE-STEERED-GENERATION.2b` | `src/introspect/coverage.rs (CoverageReadout + module_coverage/design_coverage) + KnobId::all()/category_of_name() + IntrospectionPayload::coverage_readout + CoverageDocument + MCP coverage tool; schema 1.11→1.12 + schema doc §5/§6.8/changelog; fire_rate integer-ppm determinism fix (caught by introspect_tool_round_trips); cargo check --all-targets, cargo test (snapshots 6/6 + new coverage unit + introspect/mcp coverage tests), cargo clippy -D warnings, cargo fmt --check all green; SCHEMA-DERIVED / DUT .sv byte-identical` | `done` |
| `2026-06-21` | `COVERAGE-STEERED-GENERATION.2c.1` | `src/introspect/coverage.rs derive_steering_from_coverage + DeriveParams; src/config.rs SteeringConfig::set_weight + pub validate + Overrides.steer + resolve_config steer application + ConfigError::UnknownSteerKey; src/main.rs --steer flag + parse_steer_arg + cli_overrides; 9 new tests (3 derive + 4 config steer + 2 main CLI); cargo check --all-targets, cargo test (snapshots 6/6), cargo clippy -D warnings, cargo fmt --check all green; CLI smoke (steered≠unsteered, neutral=unsteered, bad-key error); unsteered default DUT byte-identical` | `done` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `COVERAGE-STEERED-GENERATION` | `USABILITY-LANE-OWNERSHIP.1 — register 7 owner-directed usability/capability lanes + API-first decision 0017` | Tree registered (not yet started); frontier `.1` (design ADR) pending. |
| `COVERAGE-STEERED-GENERATION.1` | `COVERAGE-STEERED-GENERATION.1 — design ADR (decision 0023)` | Design-only; pins the rules-first prior-multiplier steering primitive at `roll_knob`, the byte-stability contract, the `SteeringConfig` target, the SCHEMA-DERIVED coverage readout, the outer feedback loop, and the API surface; pre-splits `.2` into `.2a`/`.2b`/`.2c`. |
| `COVERAGE-STEERED-GENERATION.2a` | `COVERAGE-STEERED-GENERATION.2a — steering core (SteeringConfig + roll_knob prior multiplier)` | First code slice: `KnobId::category()` (exhaustive 21-variant taxonomy), `SteeringConfig` (`per_knob`/`per_category` weights + `weight()`/`effective_prob()`/`is_empty()`/`validate()`), `Config.steering` (only `skip_serializing_if`), `ConfigError::SteeringWeight`, the `roll_knob` prior multiplier. Three proofs green (byte-identical default; distribution shift; no-filter) + full cargo gate. Rules-first / DUT byte-identical when unset. |
| `COVERAGE-STEERED-GENERATION.2b` | `COVERAGE-STEERED-GENERATION.2b — achieved-coverage readout (--introspect section + MCP coverage query)` | Second code slice (the READ half): `src/introspect/coverage.rs` (`CoverageReadout` + `module_coverage`/`design_coverage`), `KnobId::all()`/`category_of_name()`, the `coverage_readout` payload section (schema `1.11→1.12`), the `CoverageDocument` envelope + the pure MCP `coverage` tool (one projection feeding both). Schema doc §5/§6.8/changelog. `fire_rate` integer-ppm for byte-stable determinism (1-ULP fix caught by the pre-existing round-trip test, not weakened). SCHEMA-DERIVED / DUT `.sv` byte-identical; full cargo gate green. |
| `COVERAGE-STEERED-GENERATION.2c.1` | `COVERAGE-STEERED-GENERATION.2c.1 — outer-loop derive helper + --steer CLI shim` | Third code slice (the steering-OUT half): the pure `derive_steering_from_coverage` helper (decision `0023` §4, milli-quantized weights) + `SteeringConfig::set_weight` (one classifier) + `pub validate` + `Overrides.steer` + the repeatable `--steer <key>=<weight>` CLI shim (preset-then-explicit in `resolve_config`). 9 new proofs + CLI smoke (steered≠unsteered, neutral=unsteered, bad-key error). Unsteered default DUT byte-identical; full cargo gate green. |

## Changelog

- `2026-06-17`: Created task tree (registration via `USABILITY-LANE-OWNERSHIP.1`).
- `2026-06-21`: `.1` design ADR landed (decision `0023`); frontier advances to
  `.2a` (the steering core). Docs-only / DUT byte-identical.
- `2026-06-21`: `.2a` steering core landed (code): `SteeringConfig` + the `roll_knob`
  prior multiplier + the three proofs + full cargo gate. Frontier advances to `.2b`
  (the SCHEMA-DERIVED achieved-coverage readout + MCP coverage query). Rules-first /
  DUT byte-identical when unset.
- `2026-06-21`: `.2b` achieved-coverage readout landed (code): `src/introspect/coverage.rs`
  (`CoverageReadout` + per-knob/per-category fire rates + the gate/operand/depth
  histograms; `module_coverage`/`design_coverage`) + `KnobId::all()`/`category_of_name()`
  + the `IntrospectionPayload::coverage_readout` section (schema `1.11→1.12`) + the
  standalone `CoverageDocument` returned by the new pure MCP `coverage` tool (one
  projection feeding both). Schema doc updated (§5 row, §6.8, changelog). `fire_rate`
  uses integer parts-per-million arithmetic for byte-stable determinism (a raw f64
  division diverged by 1 ULP between evaluation contexts; caught by the pre-existing
  exact-equality round-trip test, fixed without weakening it). SCHEMA-DERIVED / DUT
  `.sv` byte-identical; full cargo gate green. Frontier advances to `.2c` (the outer
  measure→derive→re-steer helper + `--steer` CLI shim + book/USER_GUIDE/KM; close `.2`).
- `2026-06-21`: `.2c` pre-split into `.2c.1` (code) + `.2c.2` (docs/close) — matching
  how `.2` itself was split — to keep the code slice and the load-bearing book slice
  each signoff-sized. `.2c.1` landed (code): `derive_steering_from_coverage` +
  `DeriveParams` (the outer-loop derive step, milli-quantized weights, rules-first —
  no generation/filter) + `SteeringConfig::set_weight`/`pub validate` + `Overrides.steer`
  + the `--steer <key>=<weight>` CLI shim (preset-then-explicit in `resolve_config`).
  9 proofs + CLI smoke; unsteered default DUT byte-identical; full cargo gate green.
  Frontier advances to `.2c.2` (book steering subsection + `agent-mcp.md` + USER_GUIDE
  + KM card + the schema-`1.12` book example refresh; closes `.2c` + `.2`).
