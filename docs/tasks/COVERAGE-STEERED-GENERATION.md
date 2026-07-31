# COVERAGE-STEERED-GENERATION: construction-time coverage-feedback steering

## Metadata

- Tree ID: `COVERAGE-STEERED-GENERATION`
- Status: `done`
- Roadmap lane: `Usability / effectiveness — coverage-steered generation (north star, idea 6)`
- Created: `2026-06-17`
- Last updated: `2026-07-31` (**TREE CLOSED at `.6`** — `KnobId::all()` is derived from one macro table, so the list and `.4b.1`'s guard retire together; the `ENUMERATION-PARITY` extractor repointed in the same commit. Prior: `2026-07-30` (**`.3` CLOSED**, `.4a` design ADR landed (decision `0035`); frontier `.4b.1`. — `.3a` design ADR (decision `0034`, since amended with a dated Correction), `.3b` the fix (one steering-aware knob-roll primitive; a second one is now a compile error; `--steer hierarchy` works), `.3c` docs. Frontier: **`.4`** — steering's *width*. The `.1`/`.2` scope stays closed and is not revisited.))
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
  Status: `done`
  Goal: `Construction-time coverage-feedback steering (rules-first, reproducible) with an API-settable coverage target + an API-queryable achieved-coverage readout.`
  Closure: `The .1/.2 scope closed 2026-06-22 and is NOT reopened: every acceptance criterion of that scope was met (the roll_knob per-category/per-knob prior multiplier measurably shifts the distribution rules-first + reproducible (.2a); the achieved coverage is API-queryable (.2b) and the target API-settable (.2c.1) per decision 0017; unsteered DUT byte-identical; documented (.2c.2)). The tree returns to 'active' on 2026-07-30 for the new .3 node (decision 0034: a MEASURED SILENT NO-OP in the shipped surface — 6 of the 22 KnobIds, and therefore the whole documented 'hierarchy' steering category, are never reached by the prior because src/gen/hierarchy.rs defines seven roll primitives of its own) and .4 (the recorded decision-0023 follow-up: the 16 Bernoulli knobs that have no KnobId at all). Nothing retired (feedback_never_retire_strategies); the Phase-4 closure pattern — a closed scope stays closed, new work lands as new nodes.`
  Children: `COVERAGE-STEERED-GENERATION.1`, `.2`, `.3`, `.4`, `.5`, `.6`

- ID: `COVERAGE-STEERED-GENERATION.1`
  Status: `done`
  Goal: `Design/decision leaf (ADR, no code): pin HOW coverage feedback biases construction WITHOUT generate-then-filter (e.g. per-category/per-surface weight multipliers applied to the existing roll_knob decision sites; or a deterministic schedule across a --count run that nudges weights toward under-hit constructs) while keeping byte-stability per (seed, knobs, steering-config); define the coverage-target model + the achieved-coverage readout (reuse knob_roll_attempts/fires + gate/category/surface histograms in Metrics); pin the MCP target-set + coverage-query surface (decision 0017); and EXPLICITLY reconcile with feedback_rules_first_generation (steering is a construction-time prior, not a post-hoc filter). Record as the next decision record + pre-split .2 (impl).`
  Acceptance: `A decision record + a tree/DEVELOPMENT_NOTES entry pinning the rules-first steering model, the reproducibility contract, the coverage target/readout, and the MCP surface; docs-only; INDEX + this tree + docs/TASK_TREE.md updated.`
  Verification: `done — decision 0023: the steering primitive is a deterministic per-category probability-prior MULTIPLIER on prob at the roll_knob site (effective_prob = clamp01(prob * weight), one gen_bool draw preserved) — rules-first (a construction-time prior, NOT a filter; no rejection path) and byte-stable per (seed, knobs, steering-config), byte-identical when unset (weight=1.0). Coverage-target = a SteeringConfig (KnobId / category → emphasis weight); achieved-coverage readout = SCHEMA-DERIVED from knob_roll_attempts/fires + histograms (zero new truth, decision 0011); feedback = an OUTER measure→derive→re-steer loop (not in-generator); API target-set + coverage-query per decision 0017. In-generator adaptive schedule + raw gen_bool/weighted-choice sites + behavioural coverage explicitly rejected/deferred. Pre-split .2a/.2b/.2c. INDEX + tree + TASK_TREE + DEVELOPMENT_NOTES updated; KM regen; docs-only / DUT byte-identical.`
  Commit: `COVERAGE-STEERED-GENERATION.1 — design ADR (decision 0023)`

- ID: `COVERAGE-STEERED-GENERATION.2`
  Status: `done`
  Goal: `Implement the .1 design (decision 0023). Pre-split: .2a (the SteeringConfig + weight() lookup + the roll_knob prior multiplier + byte-identical-when-unset + distribution-shift + no-filter proofs), .2b (the SCHEMA-DERIVED achieved-coverage readout in --introspect + the MCP coverage query), .2c (the outer measure→derive→re-steer helper + book/USER_GUIDE/KM; close).`
  Acceptance: `set at .1 (decision 0023): a per-category prior multiplier at roll_knob that measurably shifts the achieved construct distribution vs unsteered on a seed sweep while staying rules-first (no filter path) and byte-stable per (seed, knobs, steering-config); unsteered default byte-identical; the coverage target settable + the achieved coverage queryable over the MCP/config API (CLI a shim); downstream-clean.`
  Verification: `done — delivered across .2a/.2b/.2c.1/.2c.2: the roll_knob per-category/per-knob prior multiplier (measurable distribution shift, no filter, byte-stable, byte-identical when unset); the SCHEMA-DERIVED coverage_readout in --introspect + the MCP coverage tool; the derive_steering_from_coverage helper + the --steer CLI shim (target settable via Config.steering over MCP/--config + the CLI shim — decision 0017 API-completeness); full book/USER_GUIDE/KM docs. Every acceptance criterion met; snapshots 6/6 byte-identical throughout; downstream-clean (no emitted RTL change by default).`
  Commit: `closed by .2c.2 (COVERAGE-STEERED-GENERATION.2c.2 — steering-lane docs + close .2)`

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
  Status: `done`
  Goal: `The outer measure→derive→re-steer convenience (a deterministic derive_steering_from_coverage helper) + the --steer CLI shim + book (algorithm.md steering subsection + agent-mcp.md) + USER_GUIDE + a KM card; close .2.`
  Acceptance: `set at .1 (decision 0023).`
  Verification: `done — delivered as .2c.1 (the derive_steering_from_coverage helper + the --steer CLI shim) + .2c.2 (book/USER_GUIDE/KM + close). Closes .2.`
  Commit: `.2c.1 (12416c1) + .2c.2 (this commit)`

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
  Status: `done`
  Goal: `The DOCS + close: book steering subsection in algorithm.md + the coverage/steering surfaces in agent-mcp.md (incl. the schema 1.12 coverage_readout example refresh + the coverage MCP tool) + USER_GUIDE (the --steer shim + the measure→derive→re-steer recipe) + a KM card for the steering capability; mark .2c, .2, and re-evaluate the tree. Refreshes the owner-deferred steering-lane book/USER_GUIDE drift recorded at .2a/.2b.`
  Acceptance: `book + USER_GUIDE accurately document steering (the prior multiplier, the SteeringConfig target, the coverage readout, the coverage MCP tool, --steer, and the outer loop) with runnable examples; the schema-1.12 example refresh lands; KM regen+check green; mdbook build clean; .2c + .2 marked done; INDEX/tree/TASK_TREE updated.`
  Verification: `done — book/src/algorithm.md gains the "Construction-time coverage steering" section (the roll_knob prior multiplier, rules-first, byte-stable, the follow-up note); book/src/agent-mcp.md gains the "Coverage-steered generation" section (the measure→derive→re-steer loop) + the coverage tool row + the coverage_readout in --introspect + schema 1.11→1.12 example refresh; book/src/api-tools.md (coverage tool section + 10-tools + schema refresh), api-introspection.md (coverage_readout schema section + envelope/contract 1.12), api-reference.md (version 1.12 + 10 tools), knobs.md (roll-rate → steering cross-ref); USER_GUIDE.md (--steer table row + "Coverage steering" subsection with the recipe + schema 1.12). KM: decision 0023 enriched with 4 shipped-surface answer keys + status → delivered. Verification: mdbook build clean; cargo test --test book_examples 3/3 (the runnable --steer example executes, exit 0); KM regen+check green. Docs-only (no src change beyond the already-shipped code) ⇒ DUT byte-identical. Closes .2c + .2.`
  Commit: `COVERAGE-STEERED-GENERATION.2c.2 — steering-lane docs + close .2 (book/USER_GUIDE/KM)`

- ID: `COVERAGE-STEERED-GENERATION.3`
  Status: `done`
  Goal: `Make the steering prior reach EVERY roll site of every KnobId. Opened 2026-07-30 on a measured defect: src/gen/hierarchy.rs:883-938 defines seven roll primitives of its own that record the same m.knob_rolls telemetry as roll_knob while omitting SteeringConfig::effective_prob, so 6 of the 22 KnobIds are never steered, HierarchyParentFlopProb is half-steered, and --steer hierarchy=<w> is a silent no-op that the CLI/config/--dump-config all accept and echo. Scoped to the existing 22-knob KnobId universe; widening that universe is .4.`
  Acceptance: `set at .3a (decision 0034): exactly one steering-aware knob-roll primitive exists and a second one is a COMPILE ERROR (KnobRollCounters::record privatized into the primitive's module — repair rung R2, no new registered doctrine); all 7 hierarchy call sites route through it with an explicit KnobId at the decision site; a hierarchy-category distribution-shift regression proof (the sibling of .2a's state-category proof) is green; a neutral weight (1.0) and no --steer are both byte-identical (tests/snapshots.rs 6/6 untouched); full COMMIT.md cargo gate.`
  Verification: `done — .3a (design ADR, decision 0034, since amended with a dated Correction), .3b (the fix + proofs), .3c (docs + close). --steer hierarchy measurably biases construction (child_input_cone 0/5 -> 5/5, exactly clamp01(0.3*9)); every KnobId is steered at every one of its decision sites; a second roll primitive is a compile error, negative-controlled both ways; unsteered emission byte-identical (3 pre-fix hashes + snapshots 6/6); book/USER_GUIDE/README/KM state the invariant accurately with a runnable example. Closes .3.`
  Children: `COVERAGE-STEERED-GENERATION.3a` (design ADR), `.3b` (the fix + proofs), `.3c` (docs + close `.3`).

- ID: `COVERAGE-STEERED-GENERATION.3a`
  Status: `done`
  Goal: `Design/decision leaf (ADR, no code): record the measured defect, root-cause it, pin the fix shape (one shared steering-aware primitive with a borrow signature the hierarchy planner can actually call), pin the structural guard rung, and pin the scope boundary between .3 (steering's REACH over the existing KnobId set) and .4 (its WIDTH — the knobs with no KnobId).`
  Acceptance: `A decision record + tree/INDEX/TASK_TREE/live-doc updates naming: the measurement, the root cause, the primitive's signature, the guard rung and why R2 rather than R4, the byte-identity argument, and the .3/.4 split. Docs-only ⇒ DUT byte-identical.`
  Verification: `done — decision 0034. MEASURED (commit ff506e1): 16 of 22 KnobIds reached by roll_knob; 6 never reached (HierarchySiblingRouteProb, HierarchyRegisteredSiblingRouteProb, HierarchyRegisteredSiblingMixedSupportProb, HierarchyRegisteredChildInputConeProb, HierarchyChildInputConeProb, HierarchyParentConeInstanceProb); HierarchyParentFlopProb steered only at the two cfg.flop_prob-swap cone scopes (hierarchy.rs:1533,:1673), not at its dedicated helper (:934). Evidence: a 9x per-knob / per-category steer leaves the recorded fire counts BIT-IDENTICAL (hierarchy_child_input_cone_prob 0/5 where clamp01(0.3*9)=1.0 demands 5/5); --steer hierarchy=8.0 vs =0.01 (800x spread) emits byte-identical SV; positive control --steer state=8.0 does change output. ROOT CAUSE: the .1 survey enumerated roll sites by the SHAPE it knew (roll_knob( call sites) rather than from the AUTHORITATIVE SET (knob_rolls.record(, 8 sites in 2 files) — decision 0033 rule (2) recurring in a second lane; the helpers predate the steering core by two months (28c5474 2026-04-23 vs 2530bfd 2026-06-21). FIX: one roll_knob_into(&mut KnobRollCounters, &SteeringConfig, rng, knob, prob) primitive (the borrow shape the hierarchy planner needs — its module is reached through ctx.top while g is live, which is WHY it forked); delete the 7 named helpers (7 one-line KnobId wrappers are themselves a shadow, decision 0033); privatize KnobRollCounters::record into the primitive's module so a second primitive cannot compile (R2 — R4 rejected: a registered doctrine is a mechanism maintained forever for something the type system enforces free). Retro-editing .2a's false "single integration point" note REJECTED (layer-B history, NEVER REWRITE HISTORY). Docs-only ⇒ DUT byte-identical.`
  Commit: `COVERAGE-STEERED-GENERATION.3a — design ADR (decision 0034)`

- ID: `COVERAGE-STEERED-GENERATION.3b`
  Status: `done`
  Goal: `The FIX (code): extract roll_knob_into into src/ir/knob_roll.rs beside KnobRollCounters and privatize record; make cone::roll_knob a one-line wrapper (its 37 call sites untouched); delete the seven roll_hierarchy_* helpers and re-point their seven call sites at the primitive with an explicit KnobId::… at each decision site.`
  Acceptance: `set at .3a (decision 0034). Proofs: (i) a hierarchy-category distribution-shift test — an up-weighted hierarchy steer raises a helper-rolled knob's recorded fire rate (the direct regression for this defect, which the sibling .2a state-category proof lacked); (ii) neutral weight 1.0 byte-identical; (iii) unsteered byte-identical (tests/snapshots.rs 6/6 untouched); (iv) architectural — one gen_bool per roll preserved (rules-first, no filter, no extra draw). Full COMMIT.md cargo gate (check --all-targets / test / clippy -D warnings / fmt --check).`
  Verification: `done — new src/ir/knob_roll.rs holds KnobRollCounters (moved from ir/types.rs, re-exported so Module.knob_rolls and every crate::ir::KnobRollCounters path is unchanged) + roll_knob_into(&mut KnobRollCounters, &SteeringConfig, &mut impl Rng, KnobId, f64); KnobRollCounters::record is PRIVATE to that module; Generator::roll_knob(&mut self, m, knob, prob) in gen/mod.rs is the crate-wide shim; cone::roll_knob survives as a free-fn alias (37 call sites untouched); gen/hierarchy.rs now has ZERO roll helpers — 7 direct g.roll_knob(m, KnobId::…, prob) calls. END-TO-END FIX (release, seed 42, depth-1 wrapper, 6 children, both routes 0.3): child_input_cone 0/5 -> 5/5 under --steer hierarchy_child_input_cone_prob=9.0 AND --steer hierarchy=9.0 (exactly clamp01(0.3*9)=1.0); sibling_route 2/5 -> 3/3. UNSTEERED BYTE-IDENTICAL vs pre-fix hashes measured the same session (default leaf fdaad15b, simple hierarchy d6a7961e, all-7-knob hierarchy f2ead3e2) + snapshots 6/6. NEW PROOFS: tests/pipeline.rs steering_shifts_hierarchy_category_construct_distribution (per-knob, BOTH directions — 3x must raise and 0.05x must lower each of 3 probed knobs over a 12-seed design sweep) + neutral_hierarchy_steering_is_byte_identical_on_designs (the .2a neutral proof only covers the single-module lane, which has no hierarchy roll); 3 unit proofs in knob_roll.rs. BOTH GUARDS NEGATIVE-CONTROLLED BOTH WAYS: reintroducing the pre-.3b helper shape fails with error[E0624] "method record is private" and removing the probe restores a clean build; rewiring one hierarchy call site to an unsteered roll makes the distribution test FAIL and restoring it makes it pass. FULL GATE: check --all-targets clean (0 warnings), clippy -D warnings clean, fmt --check clean, cargo test green (lib 748/0, book_examples 3/3, snapshots 6/6, pipeline green). CORRECTED .3a's root cause (decision 0034 gained a dated Correction section — not a silent edit): the fork was NOT a borrow conflict (g.roll_knob(ctx.top, KnobId::X, g.cfg.x) compiles at all 7 sites via two-phase borrows) but plain VISIBILITY — cone::roll_knob was a module-private fn invisible to sibling gen::hierarchy. Also corrected .3a's "16 of 22 reached" arithmetic: honestly 15 fully steered / 6 not at all / 1 (HierarchyParentFlopProb) steered at 2 of 3 sites.`
  Commit: `COVERAGE-STEERED-GENERATION.3b — one steering-aware knob-roll primitive`

- ID: `COVERAGE-STEERED-GENERATION.3c`
  Status: `done`
  Goal: `DOCS + close .3: book/src/algorithm.md's steering section states the one-primitive invariant and the compile-time guard (it currently says the prior applies "at the roll_knob site", which was true of 16 of 22 knobs); book/src/knobs.md + USER_GUIDE.md steering rows reflect that every KnobId is steerable at every roll site; a KM card; mark .3 done and refresh docs/TASK_TREE.md.`
  Acceptance: `book + USER_GUIDE state the invariant accurately with a runnable hierarchy-steer example; mdbook build clean; cargo test --test book_examples green; KM regen+check green; .3 marked done. Docs-only ⇒ DUT byte-identical.`
  Verification: `done — book/src/algorithm.md: the steering section now names ir::knob_roll::roll_knob_into as the primitive, adds the compile-time-guard bullet, states the scope precisely (every knob in the coverage readout is steerable at EVERY decision site; knobs outside the set error rather than being ignored), and gains a new "Why the guard is a compile error" subsection carrying the two transferable rules (guard the EFFECT not the wrapper; a private helper carrying a cross-cutting invariant is an invitation to fork it) plus the per-knob/two-sided test rule. book/src/knobs.md: the roll-rate section states that the counters cover EVERY site of each listed knob (an uninstrumented roll does not compile) and gains a RUNNABLE two-command example proving 0/5 -> 5/5 under --steer hierarchy=9.0, with the "identical rather than close" diagnostic. USER_GUIDE.md: a "Which knobs a steer reaches" paragraph with the real unknown-steer-key error text, plus an explicit callout that --steer hierarchy was inert before 2026-07-30 so a corpus tuned in that window is worth repeating. README.md: the steering bullet corrected NET-NEUTRAL in length (README policy, CLAUDE.md §14). Decision 0034 enriched as the KM card: status accepted -> delivered, 4 new answer keys, evidence rewritten to the shipped surface, and a reverify command. Checks: mdbook build clean; cargo test --test book_examples 3/3 (65 runnable blocks, up from 64 — the new example genuinely executes); KM regen 86 facts / 860 question keys + check green; scripts/check_doctrines.sh all 7 PASS. Docs-only ⇒ DUT byte-identical. ALSO REGISTERED two findings surfaced by this slice (ownership before the next edit, per the owner's standing directive): BOOK-EXAMPLES-RUNNABLE.3 (the no-silent-skips guard is defeatable — a reasonless sentinel parses to the reason ">") and the new README-POLICY-ADOPTION tree (CLAUDE.md §14 unimplemented; README is 1771 lines / 122767 bytes at HEAD vs the policy's 300 / 16384).`
  Commit: `COVERAGE-STEERED-GENERATION.3c — steering docs + close .3`

- ID: `COVERAGE-STEERED-GENERATION.4`
  Status: `done`
  Goal: `The decision-0023 follow-up proper — steering's WIDTH, not its reach. Give the 16 remaining Bernoulli knobs a KnobId and route them through the one .3b primitive so they gain construction-time telemetry AND steerability together: 7 module-level motif rolls (width_parameterization_prob, memory_prob, fsm_prob, fsm_mealy_prob, multi_clock_prob, aggregate_prob, aggregate_array_prob) + 9 emit-projection rolls (soft_union_slice_prob, function_emit_prob, generate_loop_emit_prob, task_emit_prob, multi_output_task_emit_prob, cone_function_emit_prob, mux_if_emit_prob, case_mux_if_emit_prob, casez_mux_if_emit_prob). Their symptom is LOUD (--steer memory_prob=2.0 errors "unknown steer key"), so this is a feature gap, not a defect — which is why it is separate from .3.`
  Acceptance: `to be set at .4a. Must decide: whether the 16 join the existing six categories or introduce motifs/emission; whether widening coverage_readout's key set warrants an introspection schema MINOR bump; and how the emit-projection passes obtain &SteeringConfig (they currently take only &mut Module + rng + p). EXCLUDED by kind: operand_duplication_rate / mux_arm_duplication_rate (dedup thresholds compared in ir/compact.rs + metrics.rs, not Bernoulli rolls — there is no roll to apply a prior to) and library_prob (no reader anywhere in src/; a documented-reserved orphan recorded by COVERAGE-INSTRUMENTATION.3). Default-off/unsteered must stay byte-identical.`
  Verification: `done — steering's WIDTH is delivered. .4a design (decision 0035), .4b.1 the 7 motif knobs, .4b.2 the 9 emission knobs, .4c the docs. All 38 KnobIds are now steered at every one of their roll sites and appear in the coverage readout when their knob is on; the decision-0017 API-completeness gate holds for every ANVIL capability that actually rolls. Three knobs stay excluded BY KIND and error loudly rather than no-op. Unsteered + default generation byte-identical throughout (33 configurations compared against HEAD binaries across the two code slices).`
  Commit: `pending`
  Children: `COVERAGE-STEERED-GENERATION.4a` (design ADR), `.4b.1` (7 motif knobs), `.4b.2` (9 emit-projection knobs), `.4c` (docs + close `.4`).

- ID: `COVERAGE-STEERED-GENERATION.4a`
  Status: `done`
  Goal: `Design/decision leaf (ADR, no code): pin the category taxonomy for the 16 knobs, how the emit-projection passes obtain a &SteeringConfig, the byte-identity argument, whether the introspection schema needs a MINOR bump, which knobs are excluded and why, and the .4b split.`
  Acceptance: `A decision record + tree/INDEX/TASK_TREE/live-doc updates naming the taxonomy, the signature change, the byte-identity argument, the schema call, the exclusions-by-kind, and the split. Docs-only ⇒ DUT byte-identical.`
  Verification: `done — decision 0035. MEASURED: Config carries 41 f64 probability knobs (the exact set Config::validate range-checks into [0,1]); 22 have a KnobId; of the remaining 19, 16 are in scope (7 module-level motif rolls + 9 per-gate emit-projection rolls) and 3 are excluded BY KIND (operand_duplication_rate / mux_arm_duplication_rate are dedup thresholds compared in ir/compact.rs + metrics.rs, not Bernoulli rolls — nothing for a prior to multiply; library_prob has no reader anywhere in src/ — a COVERAGE-INSTRUMENTATION.3 documented-reserved orphan). Blast radius counted: the 9 annotate_* passes have 99 in-crate call sites (mostly their own #[cfg(test)] tests) and the 7 motif rolls sit at 4 sites in gen/module.rs + 3 in gen/mod.rs (each duplicated across generate_module and generate_module_with_interface_profile). DECIDED: two NEW categories motifs + emission (folding into the existing six rejected — memory_prob is not `state`, rendering is not `selection`; one merged `capabilities` category rejected because per-module and per-gate roll granularities differ by ~2 orders of magnitude, the measured .3b calibration failure being the evidence); the existing six keep EXACT membership so every working steering config keeps its meaning; the 9 passes gain a &SteeringConfig param (not &Config — the exact dependency, and it keeps ir:: off the whole config surface); the `if cfg.<knob> > 0.0` guard is LOAD-BEARING for reproducibility (removing it consumes 16+ extra RNG draws on the default path) with its honest consequence stated — a default-off knob records attempts=0, so the readout cannot distinguish `off` from `never reached`; NO schema bump, with the general rule pinned (bump for a new payload key or query kind, not for more entries in an existing map). Byte-identity is exact because all 16 are range-checked into [0,1], so effective_prob's unset short-circuit prob.min(1.0) == prob bitwise. ALSO FLAGGED, not decided: src/gen/module.rs:143 rolls a hard-coded gen_bool(0.5) with no knob at all (steering gap 3 territory), and KnobId::all() reaching 38 hand-maintained entries needs an R2 guard (proposed: a private exhaustive index() match + a derived length test). Docs-only ⇒ DUT byte-identical.`
  Commit: `COVERAGE-STEERED-GENERATION.4a — design ADR (decision 0035)`

- ID: `COVERAGE-STEERED-GENERATION.4b.1`
  Status: `done`
  Goal: `The 7 MODULE-LEVEL MOTIF knobs (code): width_parameterization_prob, memory_prob, fsm_prob, fsm_mealy_prob, multi_clock_prob, aggregate_prob, aggregate_array_prob. 7 new KnobId variants + the `motifs` category + route their 4 gen/module.rs and 3 gen/mod.rs sites (each duplicated across generate_module and generate_module_with_interface_profile) through Generator::roll_knob, PRESERVING the `> 0.0` guard. Carries the KnobId::all() R2 guard from decision 0035's open questions, since this is the smaller of the two code slices.`
  Acceptance: `set at .4a (decision 0035). Unsteered + default-config byte-identical (tests/snapshots.rs 6/6 untouched; a default run records no new coverage_readout entries because the `> 0.0` guard means no roll); --steer motifs=<w> measurably shifts the achieved motif fire rate over a seed sweep, BOTH directions, per-knob (the .3b proof shape); the KnobId::all() guard is negative-controlled (adding a variant without extending all() must fail). Full COMMIT.md cargo gate.`
  Verification: `done — 7 new KnobId variants (WidthParameterization/Memory/Fsm/FsmMealy/MultiClock/Aggregate/AggregateArray) + the new `motifs` category; the 6 existing categories keep EXACT membership. All 7 roll sites routed: 4 in gen/module.rs, 3 in gen/mod.rs (multi_clock at both single-module entry points AND the design loop; aggregate + aggregate_array in the design loop). The 3 pre-module rolls use the new Generator::roll_knob_pending + pending_knob_rolls buffer, drained by KnobRollCounters::absorb INSIDE ir::knob_roll so the one-writer invariant survives the detour. FEATURE PROVEN: `--seed 7 --memory-prob 0.5 --introspect` went from an EMPTY coverage_readout (the exact symptom that opened this session) to `memory_prob {attempts:1, fires:1}` + `motifs` category; the unknown-steer-key error now lists 7 categories; over 24-30 seeds at base 0.25 the achieved memory_prob rate moves unsteered 0.267 -> motifs=3.0 0.833 -> motifs=0.05 0.033. BYTE-IDENTICAL: 17 configurations compared against a HEAD binary built in an isolated git worktree (default x2, every motif knob individually and combined, hierarchy x3, emit surfaces, a steered run) — ALL identical. BUG FOUND AND FIXED MID-SLICE: the first cut drained pending rolls only inside the 3 FIRING branches, so a non-firing motif roll recorded an attempt nobody saw — measured as a constant fire_rate of 1.000 (only firing modules reported). Fixed by wrapping the dispatcher so there is ONE drain outside every branch; negative-controlled (restoring the bug fails the new attempts assertion with "got 7" instead of 24). GUARD on KnobId::all(): exhaustive private index() (a new variant is E0004, negative-controlled) + all_is_complete_and_ordered (a middle omission fails with "out of order at position 20", negative-controlled). Its RESIDUAL GAP is documented rather than papered over — a TAIL truncation is NOT caught, and a length assertion cannot close it because any count derived from all() shrinks with it while a hand-written count is the second list decision 0033 forbids; the R1 fix (macro-derive all()) is registered as .6. 3 new pipeline proofs + 2 new types proofs. Full gate: fmt/clippy -D warnings/check all clean (0 warnings), cargo test green.`
  Commit: `COVERAGE-STEERED-GENERATION.4b.1 — the motifs category is a real dial`

- ID: `COVERAGE-STEERED-GENERATION.4b.2`
  Status: `done`
  Goal: `The 9 EMIT-PROJECTION knobs (code): add a `steering: &SteeringConfig` parameter to the nine annotate_* passes and route their per-gate rolls through roll_knob_into; 9 new KnobId variants + the `emission` category. ~99 in-crate call sites, mostly each pass's own #[cfg(test)] tests.`
  Acceptance: `set at .4a (decision 0035). Unsteered + default byte-identical; each of the nine knobs appears in coverage_readout with a per-gate attempts count when its prob > 0; --steer emission=<w> shifts the achieved rate both directions; the decision-0032 emit-surface interaction gate stays clean (the projections' mutual exclusion is unaffected — steering changes which gates are MARKED, never the exclusion order). Full COMMIT.md cargo gate.`
  Verification: `done — 9 new KnobId variants + the new `emission` category; all nine annotate_* passes gained a `steering: &SteeringConfig` parameter and route their single per-gate roll through crate::ir::knob_roll::roll_knob_into. All ~99 in-crate call sites updated (18 production sites in gen/mod.rs across both the single-module and design paths; the remainder are each pass's own #[cfg(test)] callers, which pass &Default::default()). FEATURE PROVEN: `--profile structured-emission-max --introspect` previously reported NO emission knob at all; it now reports all eight non-version-gated surfaces with their own per-gate counts — function 881/3420, task 626/2463, multi_output_task 401/1631, case_mux_if 120/425, casez_mux_if 77/285, cone_function 78/305, generate_loop 76/325, mux_if 61/253 — every achieved rate ~0.25, matching decision 0032's calibrated preset value, under a single `emission` category (2320/9107 = 0.2547). Steering works both directions: emission 0.397 unsteered -> 0.069 at --steer emission=0.2. The unknown-steer-key error now lists all 8 categories. BYTE-IDENTICAL across 16 configurations vs a HEAD binary built in an isolated git worktree — default x2, every one of the nine surfaces individually (incl. the 2023 soft_union up-opt), the structured-emission-max preset, all eight at 0.25 together, a hierarchy design with emission on, the motif knobs, and a steered run. 3 new pipeline proofs (both-direction category shift with a >100-attempts assertion proving the rolls are per-gate; per-surface measurability + steer-key reachability; default-off records nothing). Full gate: cargo fmt --all --check / clippy --all-targets -D warnings / check --all-targets all clean (0 warnings); cargo test green.`
  Commit: `COVERAGE-STEERED-GENERATION.4b.2 — nine emission surfaces, measurable per gate`

- ID: `COVERAGE-STEERED-GENERATION.4c`
  Status: `done`
  Goal: `DOCS + close .4: book/src/algorithm.md (six categories -> eight), book/src/knobs.md, book/src/structured-emission.md (the nine surfaces are now measurable per-gate — the missing input to the measure->derive->re-steer loop over ANVIL's densest artifacts), USER_GUIDE.md, a KM card; mark .4 done and refresh docs/TASK_TREE.md.`
  Acceptance: `book + USER_GUIDE name all eight categories and state which knobs are steerable and which are excluded by kind; mdbook build clean; cargo test --test book_examples green; KM regen+check green. Docs-only ⇒ DUT byte-identical.`
  Verification: `done — all SIX live-doc copies of the category taxonomy updated 6 -> 8 (book/src/algorithm.md, book/src/knobs.md, USER_GUIDE.md, README.md, CODEBASE_ANALYSIS.md, docs/AGENT_INTROSPECTION_SCHEMA.md). algorithm.md gains the motifs-vs-emission distinction (once per MODULE vs once per candidate GATE — opposite ends of the same pipeline AND of resolution, which is why they are separate categories). USER_GUIDE's "which knobs a steer reaches" paragraph is rewritten: the gap is closed for every knob that actually rolls, and the 3 exclusions-by-kind are named with the REAL error text (verified against the binary). book/src/structured-emission.md gains "Measuring the surfaces: per-gate fire rates" with the measured per-surface table and two cautions (a raised probability is not more surfaces — mutual exclusion plus fixed pass order means steering the CATEGORY up moves the lane toward saturation, so steer individual under-represented surfaces for diversity; and soft_union is in the category but version-gated). NEW ENUMERATION-PARITY PAIR (pair 4): the five live docs enumerating the taxonomy <-> KnobId::category's match arms, count-floored at 6, negative-controlled BOTH ways (a doc missing `emission` FAILs naming it; a new `probecat` category in the code FAILs across every doc site; restored clean). DOCTRINE_ENFORCEMENT.md's "Three today" hand-written count was REMOVED rather than incremented — decision 0033 repairs a count beside a list by DELETION, and the script's PAIRS table is the authority. Decision 0035 enriched as the KM card (status -> delivered, 4 new answer keys). mdbook build clean; cargo test --test book_examples 4/4 (65 runnable, 36 skip-sentineled); all 7 doctrines PASS. Docs-only ⇒ DUT byte-identical. Closes .4c and .4.`
  Commit: `COVERAGE-STEERED-GENERATION.4c — steering-width docs + close .4`

- ID: `COVERAGE-STEERED-GENERATION.5`
  Status: `done`
  Goal: `Complete the .3b R2 guard. MEASURED 2026-07-30 with a compile probe in src/gen/hierarchy.rs: privatising KnobRollCounters::record was not enough — `attempts` and `fires` are still `pub` fields, so a second roll primitive that skips the steering prior and writes the maps DIRECTLY (`*m.knob_rolls.attempts.entry(knob).or_insert(0) += 1`) compiles CLEAN (cargo check --all-targets exit 0). The guard blocks the obvious route and not the equivalent one.`
  Acceptance: `Make `attempts` / `fires` private with read-only accessors (the only external consumer is metrics::compute, which iterates them); the direct-write probe must then FAIL to compile, and the existing roll paths must still build and behave identically. NEGATIVE-CONTROL both ways (the probe fails; removing it restores a clean build), exactly as .3b did for `record`. Unsteered + default byte-identical (no generation path changes; tests/snapshots.rs 6/6 untouched). Full COMMIT.md cargo gate.`
  Verification: `done — KnobRollCounters.attempts / .fires are now PRIVATE, readable through the new read-only accessors attempts() / fires(). Only external consumer updated: metrics::compute (2 lines). NEGATIVE-CONTROLLED BOTH WAYS: the direct-field bypass probe now fails with error[E0616]: field `attempts` of struct `KnobRollCounters` is private (+ the same for `fires`), where before .5 it compiled CLEAN at exit 0; removing the probe restores a clean build. The .3b `record` route still fails with E0624, so BOTH syntactic writes to the protected state are now compile errors. No generation path touched (visibility + two accessors only) ⇒ unsteered and default emission byte-identical, snapshots 6/6. Full COMMIT.md gate: cargo fmt --all --check clean, cargo clippy --all-targets -- -D warnings clean, cargo check --all-targets clean, cargo test green.`
  Commit: `COVERAGE-STEERED-GENERATION.5 — the guard covers the state, not just the API`
  Ordering: `BEFORE .4b.1 — .4b.1 introduces a NEW writer path (the pending-counter drain for the three pre-module motif rolls) and that path should be designed against a complete guard, not an incomplete one.`

- ID: `COVERAGE-STEERED-GENERATION.6`
  Status: `done`
  Goal: `Retire the KnobId list by DERIVING it — rung R1 (decision 0033: repair a shadow by removing it, not by gating it forever). src/ir/knob_id.rs currently carries FIVE parallel tables of the same 38 variants: the enum, all(), index(), name(), category(). Generate all five from ONE macro_rules! table whose row is `Variant => "name", "category";`, so the enum and its three projections cannot disagree by construction and .4b.1's index()/all_is_complete_and_ordered guard pair retires with the list it guarded. Registered at .4b.1, whose own negative control proved that guard cannot catch a TAIL truncation of all() — and a length assertion cannot close that, because any count derived from all() shrinks with it while a hand-written count is the second list decision 0033 forbids as a repair.`
  Acceptance: `(i) exactly one hand-maintained table of the 38 knobs remains in the crate, and a new knob is ONE row — negative-controlled by adding a probe row and observing it reach all()/name()/category() with no other edit, and by the fact that no `all()` list exists to omit it from; (ii) index() and all_is_complete_and_ordered are DELETED, not kept alongside (two mechanisms for one job is feedback_full_factorization's anti-pattern); (iii) scripts/check_enumeration_parity.sh's extract_steering_categories is repointed at the macro table's category column IN THE SAME COMMIT, and the repointed extractor must read a source FACT, not a rustfmt-chosen layout (PARITY-EXTRACTOR-ARM-SHAPE-GAP.1's lesson); (iv) both parity directions still negative-controlled (a 9th category in the table FAILs at every doc site; a doc that drops one FAILs naming it); (v) unsteered + default generation byte-identical (tests/snapshots.rs 6/6 untouched) — this is a pure re-derivation of the same tables, so EVERY --introspect coverage readout and every emitted .sv must be bit-identical, not merely equivalent; (vi) full COMMIT.md cargo gate.`
  Verification: `done — src/ir/knob_id.rs 483 -> 317 lines. ONE macro table (`knob_ids! { Variant => "name", "category"; }`) expands to the enum + all() + name() + category(); category_of_name() stays hand-written beside it as the single inversion of name(). MEASURED with a per-variant occurrence count: at HEAD every one of the 38 variant names appears exactly 5 times in the file (enum/all/index/name/category) — `sort -u` over the 38 counts prints the single value `5`; it now prints `1`. index() and all_is_complete_and_ordered DELETED, not kept beside the macro; lib test total 752 -> 751, i.e. exactly one test retired and nothing else lost. NEGATIVE-CONTROLLED, every in-place edit restored byte-exact and verified with `cmp` (never `git checkout --`, per the recorded gotcha): (A) one probe ROW reaches all()/name()/category()/category_of_name() with NO other edit (a temporary tests/nc_probe.rs asserts it, green); (A2) THE DECISIVE ONE — at HEAD, deleting the LAST entry of all() while keeping the variant leaves both the build AND all_is_complete_and_ordered GREEN (reproduced live in the HEAD worktree: `2 passed`), which is the documented residual gap; now the same omission requires deleting the ROW, which deletes the VARIANT, so cargo check dies with `error[E0599]: no variant or associated item named CasezMuxIfEmitProb` — the gap has no syntactic form left; (B) a 9th category (`probecat`) in the table FAILs at all four doc sites naming it; (C) USER_GUIDE.md with `datapath` masked FAILs naming `datapath`; (D) a RESHAPED row (the sole `sharing` row split across 3 lines) yields 7 < floor 8 and reports `the extractor is broken, not the enumeration`. A CLAIM THIS SLICE MADE AND THEN DISPROVED, recorded rather than quietly amended: the acceptance criterion originally predicted that leaving the extractor on `pub fn category` would read ZERO categories and trip the floor loudly. MEASURED, it reads the correct 8 — worse than zero, and why the same-commit repoint is load-bearing rather than tidy. Its range terminator `/^    }$/` no longer exists where it used to (the macro definition closes `    };`, the invocation closes `}` at column 0), so the range OVER-RUNS 162 lines and swallows the table; `grep -oE '"[a-z]+"'` then skips the knob NAMES only because every one contains `_`. Right answer, wrong reason, one row deep: a probe knob named `"probe"` makes it emit a PHANTOM category, failing at every doc site for something that does not exist — the cry-wolf failure that gets a gate deleted. Rule recorded in DEVELOPMENT_NOTES + decision 0035: a sed line-range whose terminator stops existing does not fail, it runs on and returns something plausible. THREE of the first-cut negative controls were too weak to fail (a `datapathXX` mask that covers_set still substring-matches; a reshaped `CoefficientProb` row when `datapath` has three rows; and the unrun "reads zero" prediction) — all three redone. BYTE-IDENTICAL: 23 valid comparisons across 22 distinct configurations against a HEAD release binary built in an isolated git worktree — default seeds 42/7/2024, --introspect (incl. --memory-prob 0.5 and --profile structured-emission-max, the two coverage-readout paths), --dump-config x2, --metrics x2 (stderr captured), --trace high, four --steer runs (state/motifs/emission/neutral), legacy depth-1 hierarchy x3 and bounded recursive hierarchy x3 (incl. --steer hierarchy=9.0) — ALL identical, none empty. TWO EARLIER COMPARISONS WERE DISCARDED AS VACUOUS and re-run: `--hierarchy-depth 1` without --num-leaf-modules errors and prints nothing, and both sides hashed equal as e3b0c44298fc — the SHA-256 of the empty string. Caught by recognizing the hash; the harness gained an emptiness+exit-code guard and now reports byte counts (769 KB of hierarchy SV, 87 KB of introspection) so a vacuous pass cannot be scored again. FULL GATE: cargo fmt --all --check clean, cargo clippy --all-targets -- -D warnings clean, cargo check --all-targets clean, cargo test green (17 suites, 0 failures: lib 749, snapshots 6/6, book_examples 4/4, pipeline 133). mdbook build clean.`
  Commit: `f335926` — `COVERAGE-STEERED-GENERATION.6 — one table, not five`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `COVERAGE-STEERED-GENERATION.1` | `done` | Design ADR (decision `0023`) pinned the rules-first steering primitive (a prior multiplier at `roll_knob`, not a filter), the byte-stability contract, the `SteeringConfig` target model, the SCHEMA-DERIVED achieved-coverage readout, the outer measure→derive→re-steer loop, and the decision-`0017` API surface. |
| 2 | `COVERAGE-STEERED-GENERATION.2a` | `done` | Steering core landed: `KnobId::category()`, `SteeringConfig` + `weight()`/`effective_prob()`, the `roll_knob` prior multiplier, `ConfigError::SteeringWeight`. All three proofs green (byte-identical default via snapshots 6/6; measurable distribution shift; no-filter architectural) + full cargo gate. |
| 3 | `COVERAGE-STEERED-GENERATION.2b` | `done` | Achieved-coverage readout landed: `src/introspect/coverage.rs` (`CoverageReadout` + per-knob/per-category fire rates + the gate/operand/depth histograms), embedded as `IntrospectionPayload::coverage_readout` (schema `1.11→1.12`) + the standalone MCP `coverage` query (`CoverageDocument`), one projection feeding both. SCHEMA-DERIVED / DUT `.sv` byte-identical; `fire_rate` integer-ppm for byte-stable determinism. Full cargo gate green. |
| 4 | `COVERAGE-STEERED-GENERATION.2c.1` | `done` | Outer-loop CODE landed: the pure `derive_steering_from_coverage(&CoverageReadout, &DeriveParams) -> SteeringConfig` helper (decision `0023` §4, milli-quantized weights) + `SteeringConfig::set_weight` + the repeatable `--steer <key>=<weight>` CLI shim into `Config.steering` (preset-then-explicit in `resolve_config`). Unsteered default byte-identical (snapshots 6/6); CLI smoke proves steered≠unsteered, neutral=unsteered, bad-key errors. Full cargo gate green. |
| 5 | `COVERAGE-STEERED-GENERATION.2c.2` | `done` | The DOCS + close landed: book steering section (`algorithm.md`) + `agent-mcp.md` coverage-steering section + the schema-`1.12` book-example refresh (`agent-mcp`/`api-tools`/`api-introspection`/`api-reference`) + knobs.md cross-ref + USER_GUIDE `--steer`/recipe + KM (decision `0023` enriched). mdbook clean; book_examples 3/3 (runnable `--steer` example). **`.2c` + `.2` + the tree CLOSED.** |

| 6 | `COVERAGE-STEERED-GENERATION.3a` | `done` | Design ADR (decision `0034`) for the `.3` node: the measured silent no-op (6 of 22 `KnobId`s unreachable by the prior; a 9× steer leaves the recorded fire counts bit-identical; an 800× spread emits byte-identical SV), the root cause (a second roll primitive predating the steering core by two months, missed because the `.1` survey searched by shape instead of from `knob_rolls.record(`), the one-primitive fix, the **R2** compile-time guard, and the `.3`/`.4` scope boundary. Docs-only. |
| 7 | `COVERAGE-STEERED-GENERATION.4c` | `1b85589` — `COVERAGE-STEERED-GENERATION.4c — steering-width docs + close .4` | The docs/close slice: six live-doc taxonomy copies updated, the per-gate measurement chapter, a fourth `ENUMERATION-PARITY` pair so they cannot rot, and a hand-written count deleted. **Closes `.4`.** |
| `COVERAGE-STEERED-GENERATION.4b.2` | `4a15ecd` — `COVERAGE-STEERED-GENERATION.4b.2 — nine emission surfaces, measurable per gate` | The second feature slice of `.4`: the nine emit-projections gain a `KnobId`, the `emission` category, and per-gate telemetry. Byte-identical across 16 configurations. |
| `COVERAGE-STEERED-GENERATION.4b.1` | `af8bd9c` — `COVERAGE-STEERED-GENERATION.4b.1 — the motifs category is a real dial` | The first feature slice of `.4`: 7 motif knobs gain a `KnobId`, the `motifs` category, and a pending-roll buffer for the three that fire before their module exists. Byte-identical across 17 configurations. |
| `COVERAGE-STEERED-GENERATION.5` | `2d447c3` — `COVERAGE-STEERED-GENERATION.5 — the guard covers the state, not just the API` | Completes the `.3b` R2 guard by privatising the counter fields behind read-only accessors. Both write routes are now compile errors (`E0624` on `record`, `E0616` on the fields); negative-controlled both ways. Byte-identical. |
| `COVERAGE-STEERED-GENERATION.4a` | `040ebc3` — `COVERAGE-STEERED-GENERATION.4a — design ADR (decision 0035)` | Opens steering's **width** after `.3` closed its *reach*. Pins two new categories (`motifs`/`emission`), the `&SteeringConfig` signature change, the load-bearing `> 0.0` guard and its readout consequence, the no-schema-bump rule, the three exclusions by kind, and the `.4b.1`/`.4b.2`/`.4c` split. Docs-only ⇒ DUT byte-identical. |
| `COVERAGE-STEERED-GENERATION.3c` | `7a1fc50` — `COVERAGE-STEERED-GENERATION.3c — steering docs + close .3` | The docs/close slice: `algorithm.md` (one-primitive invariant + the new "Why the guard is a compile error" subsection), `knobs.md` (a **runnable** `0/5 → 5/5` example + the "identical rather than close" diagnostic), `USER_GUIDE.md` (which knobs a steer reaches + a callout that `--steer hierarchy` was inert before `2026-07-30`), `README.md` (net-neutral correction), decision `0034` enriched as the KM card (status → delivered, `reverify` added). Also registers `BOOK-EXAMPLES-RUNNABLE.3` and the `README-POLICY-ADOPTION` tree. **Closes `.3`.** |
| `COVERAGE-STEERED-GENERATION.3b` | `done` | The fix landed: `roll_knob_into` in the new `src/ir/knob_roll.rs` with `record` privatized (a second primitive is now `error[E0624]`), `Generator::roll_knob` as the crate-wide shim, `cone::roll_knob` reduced to an alias (37 call sites untouched), the seven `roll_hierarchy_*` helpers deleted. `child_input_cone` goes `0/5 → 5/5` under `--steer hierarchy=9.0`; unsteered byte-identical. Both guards negative-controlled both ways. Corrected `.3a`'s root cause: visibility, not borrows. |
| 8 | `COVERAGE-STEERED-GENERATION.3c` | `done` | Docs + close `.3`: `algorithm.md` restated as the one-primitive invariant + a new "Why the guard is a compile error" subsection; `knobs.md` gained a **runnable** `0/5 → 5/5` example; USER_GUIDE gained the reach paragraph + a callout that `--steer hierarchy` was inert before `2026-07-30`; README corrected net-neutral; decision `0034` enriched as the KM card (status → delivered, `reverify` added). mdbook clean; `book_examples` 3/3 (65 runnable). **`.3` CLOSED.** |
| 9 | `COVERAGE-STEERED-GENERATION.4a` | `done` | Design ADR (decision `0035`) for steering's **width**: two new categories (`motifs`, `emission`), the `&SteeringConfig` signature change across the nine emit passes, the `> 0.0` guard pinned as load-bearing for reproducibility, no schema bump (with the rule stated), three knobs excluded **by kind**, and the `.4b` split. Measured: 41 probability knobs, 22 already steerable, 16 in scope, 99 emit call sites. |
| 10 | `COVERAGE-STEERED-GENERATION.5` | `done` | The `.3b` R2 guard now covers the **state**: `attempts`/`fires` are private behind read-only accessors, so the direct-field bypass that compiled clean before `.5` now fails with `error[E0616]`. Both syntactic writes (`record` ⇒ `E0624`, fields ⇒ `E0616`) are compile errors; negative-controlled both ways. No generation path touched ⇒ byte-identical. |
| 11 | `COVERAGE-STEERED-GENERATION.4b.1` | `done` | The 7 module-level motif knobs are now steerable **and** measurable: 7 new `KnobId`s + the `motifs` category + a `Generator::pending_knob_rolls` buffer for the 3 that roll *before any `Module` exists*, drained at **one** point outside every branch. `--memory-prob 0.5 --introspect` went from an **empty** readout to `memory_prob 1/1`; a `motifs` steer moves the achieved rate `0.267 → 0.833 / 0.033` over 24 seeds. 17 configurations byte-identical vs `HEAD`. |
| 12 | `COVERAGE-STEERED-GENERATION.4b.2` | `done` | The 9 emit-projection knobs: a `&SteeringConfig` parameter across nine `annotate_*` passes and ~99 call sites, 9 new `KnobId`s, the `emission` category. `--profile structured-emission-max --introspect` went from reporting **no** emission knob to all eight non-version-gated surfaces with per-gate counts, every achieved rate ~`0.25` — decision `0032`'s calibrated value, now *observable*. Steers `0.397 → 0.069`. 16 configurations byte-identical. |
| 13 | `COVERAGE-STEERED-GENERATION.4c` | `pending` | **Next.** Docs + close `.4`: `book/src/algorithm.md` (six categories → eight), `knobs.md`, `structured-emission.md` (the nine surfaces are now measurable per-gate — the missing input to the measure→derive→re-steer loop), `USER_GUIDE.md`, a KM card. |
| 14 | `COVERAGE-STEERED-GENERATION.6` | `done` | `KnobId::all()` is now **derived**: one `knob_ids!` table expands to the enum + `all()` + `name()` + `category()`, so each of the 38 variant names went from appearing **5 times** in the file to **exactly once**, and `index()` + `all_is_complete_and_ordered` were **deleted** with the list they guarded. The tail-truncation gap they could not catch — reproduced live at `HEAD` as a green build *and* a green test — now has no syntactic form: omitting a knob means deleting its row, which deletes the variant (`error[E0599]`). The `ENUMERATION-PARITY` extractor was repointed in the same commit, and measuring *why* disproved this leaf's own prediction: the un-repointed extractor reads the right 8 by accident (a 162-line range over-run plus an underscore coincidence) and injects a **phantom** category one row later. 23 comparisons byte-identical. |

**Tree status: `done` (`2026-07-31`) — every registered node is closed.** The
`.1`/`.2` scope stays closed (`2026-06-22`); `.3` closed the same day it opened;
`.4` closed at `.4c`; `.5` completed the guard; and `.6` retired that guard along
with the list it protected. Steering is complete in all three dimensions —
**reach** (`.3` + `.5`: every `KnobId` is steered at every one of its roll sites,
and a second, prior-skipping roll primitive is a compile error), **width**
(`.4`: all 38 knobs that actually roll have a `KnobId`, a category, and per-roll
telemetry; three are excluded *by kind* and say so loudly), and **maintainability**
(`.6`: adding the 39th knob is one table row, and there is no second list to
forget).
The remaining decision-`0023` follow-up — the **in-generator adaptive schedule**,
where the steering weights are re-derived *during* a `--count` run rather than
between runs — was deliberately deferred at `.1` and is unaffected by any of this.
It stays a future node and would reopen this tree as `.7`; it is not a blocker and
nothing here depends on it.

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
- `2026-07-30` (`.3a`): Tree returns to `active` for a new `.3` node. Design ADR
  landed as decision [`0034`](../decisions/0034-one-steering-aware-knob-roll-primitive.md):
  there is to be **exactly one** knob-roll primitive and it is steering-aware; a
  second one becomes a **compile error** (`KnobRollCounters::record` privatized
  into the primitive's module — repair rung **R2**; a registered doctrine check
  was rejected as a mechanism maintained forever for something the type system
  enforces for free). The seven `roll_hierarchy_*` helpers are deleted rather than
  re-pointed, because seven one-line `KnobId` wrappers are themselves a shadow of
  the `KnobId` set (decision `0033`). `.3` is scoped to steering's **reach** over
  the existing 22-knob universe; its **width** (the 16 Bernoulli knobs with no
  `KnobId`) is `.4`, because that symptom is *loud* (`unknown steer key`) and is a
  feature gap rather than a defect.
- `2026-07-30` (`.3a`): **The `.2a` "Implementation Notes" below are preserved
  unedited even though their central claim is now known to be false.** They state
  *"All 31 steerable rolls funnel through one function … No call site changes"*;
  seven further roll primitives already existed in `src/gen/hierarchy.rs` when that
  was written. Task files are layer-B **history** (`MEMORY_ARCHITECTURE.md` §3) and
  the `NEVER REWRITE HISTORY` directive is absolute — and here the historical text
  is the most useful artifact in the file, because it records the exact shape of the
  reasoning error (searching by known shape rather than from the authoritative set).
  Decision `0034` carries the correction; the note stays raw.

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
- (`.3a`, open) Should `HierarchyParentFlopProb` be **split into two `KnobId`s**? It
  currently labels two different decisions: the `cfg.flop_prob`-swap cone rolls
  inside parent-side cone construction (`hierarchy.rs:1533`, `:1673`) and the
  dedicated parent-flop helper roll (`:934`). Its reported fire rate therefore
  conflates them. Once `.3b` lands, both sites obey the same steering weight, so
  this stops being a steering defect — but it remains a telemetry-honesty question.
  *(Recorded at `.3a`, deliberately not decided there: it is orthogonal to the fix
  and would change `coverage_readout` key membership, which belongs with `.4`'s
  `KnobId`-universe work.)*
- (`.3a`, open) Where the shared primitive lives — `src/ir/knob_roll.rs` beside
  `KnobRollCounters` (proposed; it is what makes `record` privatizable, since
  `pub(in path)` requires an ancestor module) vs `src/gen/roll.rs` with the counters
  left in `ir::types` (which cannot deliver the R2 guard). *(To be settled by `.3b`
  against the real borrow checker.)*

## Implementation Notes (for `.2a` — captured during the `.1` design pass)

> **Historical, and known-false in one load-bearing claim** (`.3a`, `2026-07-30`).
> The "Single integration point … No call site changes" bullet below was already
> wrong when written: `src/gen/hierarchy.rs` had defined seven further roll
> primitives since `2026-04-23`. Preserved unedited per `NEVER REWRITE HISTORY` /
> `MEMORY_ARCHITECTURE.md` §3 (task files are layer-B history). The correction lives
> in decision [`0034`](../decisions/0034-one-steering-aware-knob-roll-primitive.md).

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
| `2026-07-31` | `COVERAGE-STEERED-GENERATION.6` | `src/ir/knob_id.rs 483 -> 317 lines: one knob_ids! macro table expands to the enum + all() + name() + category(); index() + all_is_complete_and_ordered DELETED with the list they guarded (lib test total 752 -> 751, exactly one test retired). MEASURED per-variant occurrence count 5 -> 1 for all 38 names. NEGATIVE-CONTROLLED, all edits restored byte-exact via cmp (never git checkout --): a probe ROW reaches all four projections with no other edit; at HEAD, dropping the last all() entry keeps build AND guard test GREEN (the documented tail gap, reproduced live) while now the same omission deletes the VARIANT (error[E0599]); a 9th category FAILs at all 4 doc sites; a doc dropping datapath FAILs naming it; a reshaped row yields 7 < floor 8 with `the extractor is broken, not the enumeration`. THIS LEAF DISPROVED ITS OWN PREDICTION: the un-repointed extractor does NOT read zero — it reads the correct 8 via a 162-line sed range over-run plus an underscore coincidence, and emits a PHANTOM category one probe row later (cry-wolf, the failure that gets a gate deleted); recorded in DEVELOPMENT_NOTES + decision 0035 rather than quietly amended. 23 valid comparisons / 22 distinct configurations byte-identical vs a HEAD release binary in an isolated worktree; 2 earlier comparisons DISCARDED as vacuous (both sides hashed e3b0c44298fc, the empty-string SHA-256, because --hierarchy-depth 1 errors without --num-leaf-modules) and re-run under a new emptiness+exit-code guard. cargo fmt --check / clippy --all-targets -D warnings / check --all-targets all clean; cargo test 17 suites 0 failures (lib 749, snapshots 6/6, book_examples 4/4); mdbook build clean; scripts/check_doctrines.sh 8/8.` | `done` |
| `2026-06-17` | `COVERAGE-STEERED-GENERATION` | `tree registered (docs-only); no code` | `registered` |
| `2026-06-21` | `COVERAGE-STEERED-GENERATION.1` | `decision 0023 written; INDEX + tree + TASK_TREE + DEVELOPMENT_NOTES updated; KM regen+check green; mem-arch green; docs-only / DUT byte-identical` | `done` |
| `2026-06-21` | `COVERAGE-STEERED-GENERATION.2a` | `SteeringConfig + KnobId::category() + roll_knob prior multiplier + ConfigError::SteeringWeight; cargo check --all-targets, cargo test (snapshots 6/6 + new steering unit/integration tests), cargo clippy -D warnings, cargo fmt --check all green; rules-first / DUT byte-identical when unset` | `done` |
| `2026-06-21` | `COVERAGE-STEERED-GENERATION.2b` | `src/introspect/coverage.rs (CoverageReadout + module_coverage/design_coverage) + KnobId::all()/category_of_name() + IntrospectionPayload::coverage_readout + CoverageDocument + MCP coverage tool; schema 1.11→1.12 + schema doc §5/§6.8/changelog; fire_rate integer-ppm determinism fix (caught by introspect_tool_round_trips); cargo check --all-targets, cargo test (snapshots 6/6 + new coverage unit + introspect/mcp coverage tests), cargo clippy -D warnings, cargo fmt --check all green; SCHEMA-DERIVED / DUT .sv byte-identical` | `done` |
| `2026-06-21` | `COVERAGE-STEERED-GENERATION.2c.1` | `src/introspect/coverage.rs derive_steering_from_coverage + DeriveParams; src/config.rs SteeringConfig::set_weight + pub validate + Overrides.steer + resolve_config steer application + ConfigError::UnknownSteerKey; src/main.rs --steer flag + parse_steer_arg + cli_overrides; 9 new tests (3 derive + 4 config steer + 2 main CLI); cargo check --all-targets, cargo test (snapshots 6/6), cargo clippy -D warnings, cargo fmt --check all green; CLI smoke (steered≠unsteered, neutral=unsteered, bad-key error); unsteered default DUT byte-identical` | `done` |
| `2026-07-30` | `COVERAGE-STEERED-GENERATION.4c` | `All 6 live-doc copies of the steering category taxonomy updated 6 -> 8 (book/src/{algorithm,knobs,structured-emission}.md, USER_GUIDE.md, README.md, CODEBASE_ANALYSIS.md, docs/AGENT_INTROSPECTION_SCHEMA.md). NEW ENUMERATION-PARITY pair 4 (the 5 doc sites <-> KnobId::category match arms), count-floored at 6, NEGATIVE-CONTROLLED BOTH WAYS: a doc missing `emission` FAILs naming it; a new `probecat` category in the code FAILs across every doc site; restoring either is clean. DOCTRINE_ENFORCEMENT.md's hand-written "Three today" pair count DELETED rather than incremented (decision 0033: a count beside a list is one more copy of it; repair by deletion). Decision 0035 enriched as the KM card (status -> delivered, 4 new answer keys, a reverify reproducing both before/after measurements). mdbook build clean; cargo test --test book_examples 4/4 (65 runnable, 36 skip-sentineled); scripts/check_doctrines.sh all 7 PASS incl. the new pair; KM regen 87 facts / 877 question keys. Docs+one gate, no generator change ⇒ DUT byte-identical. Closes .4c and .4.` | `done` |
| `2026-07-30` | `COVERAGE-STEERED-GENERATION.4b.2` | `9 new KnobId variants + the emission category; all nine annotate_* passes gained a steering: &SteeringConfig param and route their per-gate roll through roll_knob_into; ~99 in-crate call sites updated (18 production in gen/mod.rs across both paths, the rest each pass's own #[cfg(test)] callers). FEATURE: --profile structured-emission-max --introspect went from reporting NO emission knob to all 8 non-version-gated surfaces with per-gate counts (function 881/3420, task 626/2463, multi_output 401/1631, case_mux_if 120/425, casez_mux_if 77/285, cone_function 78/305, generate_loop 76/325, mux_if 61/253), every achieved rate ~0.25 = decision 0032's calibrated preset value now OBSERVABLE; emission category 2320/9107 = 0.2547; steering moves 0.397 -> 0.069 at --steer emission=0.2; the unknown-steer-key error lists all 8 categories. BYTE-IDENTICAL across 16 configurations vs a HEAD binary built in an isolated git worktree, incl. every surface individually, the 2023 soft_union up-opt, the preset, all eight at 0.25 together, a hierarchy design, and a steered run. 3 new proofs. cargo fmt --check / clippy -D warnings / check --all-targets clean (0 warnings); cargo test green.` | `done` |
| `2026-07-30` | `COVERAGE-STEERED-GENERATION.4b.1` | `7 new KnobId variants + the motifs category; all 7 roll sites routed through the one primitive (4 in gen/module.rs, 3 in gen/mod.rs incl. the design loops); new Generator::roll_knob_pending + pending_knob_rolls + KnobRollCounters::absorb for the 3 pre-module rolls. FEATURE: --memory-prob 0.5 --introspect goes from an EMPTY coverage_readout to memory_prob{1/1} + the motifs category; achieved rate over 24-30 seeds at base 0.25 moves 0.267 unsteered -> 0.833 (motifs=3.0) -> 0.033 (motifs=0.05); the unknown-steer-key error now lists 7 categories. BYTE-IDENTICAL across 17 configurations vs a HEAD binary built in an isolated git worktree. Bug found+fixed mid-slice (drain only on fire -> constant 1.000 fire_rate; fixed with ONE drain outside every branch; negative-controlled at got 7 vs 24). KnobId::all() guard: exhaustive index() (E0004, negative-controlled) + ordering test (middle omission fails, negative-controlled); the drafted length assertion COULD NOT FAIL and was deleted rather than shipped, gap documented, R1 fix registered as .6. 5 new proofs. cargo fmt --check / clippy -D warnings / check --all-targets clean (0 warnings); cargo test green.` | `done` |
| `2026-07-30` | `COVERAGE-STEERED-GENERATION.5` | `KnobRollCounters.attempts/.fires made private + read-only attempts()/fires() accessors; metrics::compute updated (2 lines); knob_roll.rs unit tests updated. NEGATIVE-CONTROLLED BOTH WAYS: the direct-field bypass probe now fails with error[E0616] (it compiled CLEAN at exit 0 before .5) and removing it restores a clean build; .3b's E0624 on `record` still holds, so both syntactic writes to the protected state are compile errors. cargo fmt --all --check / cargo clippy --all-targets -- -D warnings / cargo check --all-targets all clean; cargo test green. No generation path touched ⇒ unsteered + default emission byte-identical (snapshots 6/6).` | `done` |
| `2026-07-30` | `COVERAGE-STEERED-GENERATION.4a` | `decision 0035 written + INDEX row; tree gains .4a/.4b.1/.4b.2/.4c; docs/TASK_TREE.md + ROADMAP + MEMORY/CHANGES/DEVELOPMENT_NOTES synced; KM regen+check green; scripts/check_doctrines.sh all 7 PASS. Measurements taken against the working tree at a42d3b5: 41 validated probability knobs / 22 with a KnobId / 16 in scope / 3 excluded by kind; 99 in-crate call sites across the nine annotate_* passes; 4 motif roll sites in gen/module.rs + 3 in gen/mod.rs. Docs-only ⇒ DUT byte-identical (no src/ touched).` | `done` |
| `2026-07-30` | `COVERAGE-STEERED-GENERATION.3c` | `mdbook build book` clean; `cargo test --test book_examples` 3/3 (**65** runnable blocks — up from 64, the new `knobs.md` steering example genuinely executes rather than carrying a skip sentinel); Knowledge Map regenerated (86 facts / 860 question keys) + check green; `scripts/check_doctrines.sh` all 7 PASS. README edit made net-neutral in length per `CLAUDE.md` §14. Docs-only ⇒ DUT byte-identical (no `src/`, `tests/`, or `examples/` file touched). Two findings surfaced by this slice were REGISTERED, not merely noted: `BOOK-EXAMPLES-RUNNABLE.3` and the new `README-POLICY-ADOPTION` tree. Closes `.3c` + `.3`. | `done` |
| `2026-07-30` | `COVERAGE-STEERED-GENERATION.3b` | `cargo check --all-targets` (clean, 0 warnings), `cargo clippy --all-targets -- -D warnings` (clean), `cargo fmt --all --check` (clean), `cargo test` (green: lib 748/0, snapshots 6/6, book_examples 3/3, pipeline green incl. the 2 new proofs). End-to-end fix verified on the release binary: `child_input_cone` 0/5 -> 5/5 under both `--steer hierarchy_child_input_cone_prob=9.0` and `--steer hierarchy=9.0`. Unsteered byte-identity confirmed against 3 pre-fix hashes measured earlier in the same session. Both guards negative-controlled BOTH ways (E0624 on reintroducing the helper shape; the distribution test FAILS when one call site is rewired unsteered). | `done` |
| `2026-07-30` | `COVERAGE-STEERED-GENERATION.3a` | `decision 0034 written + INDEX row; tree reopened (.3 container + .3a/.3b/.3c + .4) ; docs/TASK_TREE.md row refreshed; MEMORY/CHANGES/DEVELOPMENT_NOTES/ROADMAP synced; KM regen+check green; scripts/check_doctrines.sh all 7 PASS. Measurement re-run at ff506e1 against target/release/anvil: 16/22 KnobIds steered, 6 unreachable, hierarchy fire counts bit-identical under a 9x steer (0/5 vs the demanded 5/5), 800x category spread byte-identical, --steer state=8.0 positive control effective. Docs-only ⇒ DUT byte-identical (no src/ touched).` | `done` |
| `2026-06-22` | `COVERAGE-STEERED-GENERATION.2c.2` | `book/src/{algorithm.md steering section, agent-mcp.md coverage-steering section + coverage tool + --introspect coverage_readout, api-tools.md coverage tool, api-introspection.md coverage_readout schema, api-reference.md, knobs.md cross-ref} + schema 1.11→1.12 example refresh; USER_GUIDE.md (--steer row + Coverage steering subsection + recipe); decision 0023 enriched (4 shipped-surface answer keys + status delivered) = KM card. mdbook build clean; cargo test --test book_examples 3/3 (runnable --steer example exit 0); KM regen+check green. Docs-only / DUT byte-identical. Closes .2c + .2 + the tree.` | `done` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `COVERAGE-STEERED-GENERATION.6` | `COVERAGE-STEERED-GENERATION.6 — one table, not five` | Rung **R1**: the `KnobId` list is *derived*. One `knob_ids!` table generates the enum, `all()`, `name()` and `category()`; `index()` + `all_is_complete_and_ordered` deleted with it. `ENUMERATION-PARITY`'s extractor repointed at the table in the same commit — and measuring why disproved the leaf's own prediction (it read 8 by accident, not 0, and would have gone on to cry wolf). Book/`CODEBASE_ANALYSIS` `KnobId` copies repaired by deletion. 23 comparisons byte-identical. **Closes the tree.** |
| `COVERAGE-STEERED-GENERATION` | `USABILITY-LANE-OWNERSHIP.1 — register 7 owner-directed usability/capability lanes + API-first decision 0017` | Tree registered (not yet started); frontier `.1` (design ADR) pending. |
| `COVERAGE-STEERED-GENERATION.1` | `COVERAGE-STEERED-GENERATION.1 — design ADR (decision 0023)` | Design-only; pins the rules-first prior-multiplier steering primitive at `roll_knob`, the byte-stability contract, the `SteeringConfig` target, the SCHEMA-DERIVED coverage readout, the outer feedback loop, and the API surface; pre-splits `.2` into `.2a`/`.2b`/`.2c`. |
| `COVERAGE-STEERED-GENERATION.2a` | `COVERAGE-STEERED-GENERATION.2a — steering core (SteeringConfig + roll_knob prior multiplier)` | First code slice: `KnobId::category()` (exhaustive 21-variant taxonomy), `SteeringConfig` (`per_knob`/`per_category` weights + `weight()`/`effective_prob()`/`is_empty()`/`validate()`), `Config.steering` (only `skip_serializing_if`), `ConfigError::SteeringWeight`, the `roll_knob` prior multiplier. Three proofs green (byte-identical default; distribution shift; no-filter) + full cargo gate. Rules-first / DUT byte-identical when unset. |
| `COVERAGE-STEERED-GENERATION.2b` | `COVERAGE-STEERED-GENERATION.2b — achieved-coverage readout (--introspect section + MCP coverage query)` | Second code slice (the READ half): `src/introspect/coverage.rs` (`CoverageReadout` + `module_coverage`/`design_coverage`), `KnobId::all()`/`category_of_name()`, the `coverage_readout` payload section (schema `1.11→1.12`), the `CoverageDocument` envelope + the pure MCP `coverage` tool (one projection feeding both). Schema doc §5/§6.8/changelog. `fire_rate` integer-ppm for byte-stable determinism (1-ULP fix caught by the pre-existing round-trip test, not weakened). SCHEMA-DERIVED / DUT `.sv` byte-identical; full cargo gate green. |
| `COVERAGE-STEERED-GENERATION.2c.1` | `COVERAGE-STEERED-GENERATION.2c.1 — outer-loop derive helper + --steer CLI shim` | Third code slice (the steering-OUT half): the pure `derive_steering_from_coverage` helper (decision `0023` §4, milli-quantized weights) + `SteeringConfig::set_weight` (one classifier) + `pub validate` + `Overrides.steer` + the repeatable `--steer <key>=<weight>` CLI shim (preset-then-explicit in `resolve_config`). 9 new proofs + CLI smoke (steered≠unsteered, neutral=unsteered, bad-key error). Unsteered default DUT byte-identical; full cargo gate green. |
| `COVERAGE-STEERED-GENERATION.3b` | `c4c7843` — `COVERAGE-STEERED-GENERATION.3b — one steering-aware knob-roll primitive` | The fix. New `src/ir/knob_roll.rs` (the single primitive + the privatized `record`), `Generator::roll_knob` shim, `cone::roll_knob` reduced to an alias, the seven `roll_hierarchy_*` helpers deleted. `--steer hierarchy` starts working (`0/5 -> 5/5`); unsteered byte-identical. Corrects `.3a`'s root cause via a dated Correction section in decision `0034`: the fork was **visibility** (a module-private `fn` in `gen::cone`), not a borrow conflict. |
| `COVERAGE-STEERED-GENERATION.3a` | `3aabb1f` — `COVERAGE-STEERED-GENERATION.3a — design ADR (decision 0034)` | Reopens the tree with `.3`. Design-only: records the measured silent no-op (the `hierarchy` steering category biases nothing; 6 of 22 `KnobId`s never reach the prior), root-causes it to a second roll primitive that the `.1` shape-keyed survey missed (decision `0033` rule 2 recurring), and pins the one-primitive fix + the **R2** compile-time guard + the `.3`/`.4` scope boundary. Docs-only ⇒ DUT byte-identical. |
| `COVERAGE-STEERED-GENERATION.2c.2` | `COVERAGE-STEERED-GENERATION.2c.2 — steering-lane docs + close .2 (book/USER_GUIDE/KM)` | The docs/close slice: book steering section (`algorithm.md`) + `agent-mcp.md` coverage-steering section + the coverage tool + the schema-`1.12` book-example refresh across the API reference chapters + knobs.md cross-ref; USER_GUIDE `--steer` + the measure→derive→re-steer recipe; decision `0023` enriched (KM card) + marked delivered. mdbook clean; book_examples 3/3 (runnable `--steer` example). Docs-only / DUT byte-identical. **Closes `.2c`, `.2`, and the tree.** |

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
- `2026-06-22`: `.2c.2` docs/close landed: the steering-lane book section
  (`algorithm.md` mechanism + `agent-mcp.md` measure→derive→re-steer loop + the
  `coverage` tool + the `coverage_readout` in `--introspect`), the schema-`1.11→1.12`
  book-example refresh across `api-tools`/`api-introspection`/`api-reference`, the
  `knobs.md` roll-rate→steering cross-ref, the USER_GUIDE `--steer` row + the
  "Coverage steering" recipe, and decision `0023` enriched (4 shipped-surface answer
  keys + status → delivered) as the KM card. mdbook clean; `book_examples` 3/3 (the
  runnable `--steer` example executes). Docs-only / DUT byte-identical. **`.2c`, `.2`,
  and the whole `COVERAGE-STEERED-GENERATION` tree are now `done`** — every acceptance
  criterion met; optional open-ended follow-ups (in-generator adaptive schedule;
  routing raw `gen_bool` sites through `roll_knob`) are future `.N` leaves that do not
  reopen the closed scope.
- `2026-07-30`: **Tree reopened as `active`** with a new `.3` node; `.3a` design ADR
  landed (decision `0034`). The `.1`/`.2` scope stays closed and is not revisited.
  Opened on a measured defect in the shipped steering surface: `src/gen/hierarchy.rs`
  defines **seven roll primitives of its own** that record the same `m.knob_rolls`
  telemetry as `roll_knob` while omitting `SteeringConfig::effective_prob`. Measured
  at `ff506e1`: **16 of 22 `KnobId`s** are reached by `roll_knob` and **6 never are**
  (all hierarchy), with `HierarchyParentFlopProb` steered at its two
  `cfg.flop_prob`-swap cone scopes but not at its dedicated helper — so the whole
  documented `hierarchy` steering category is inert. A 9× per-knob or per-category
  steer leaves the recorded fire counts **bit-identical** (`hierarchy_child_input_cone_prob`
  stays `0/5` where `clamp01(0.3 × 9) = 1.0` demands `5/5`); `--steer hierarchy=8.0`
  vs `=0.01` — an 800× spread — emits **byte-identical** SV; the positive control
  `--steer state=8.0` does change output. Silent by every surface: `--steer` accepts
  the key, validates it, stores it in `Config.steering`, and echoes it in
  `--dump-config`/`--introspect`. Root cause: the `.1` survey enumerated roll sites by
  the **shape** it knew (`roll_knob(` call sites) rather than from the **authoritative
  set** (`knob_rolls.record(` — 8 sites in 2 files), decision `0033` rule (2) recurring;
  the helpers predate the steering core by two months (`28c5474` `2026-04-23` vs
  `2530bfd` `2026-06-21`). Fix pinned for `.3b`: one shared
  `roll_knob_into(&mut KnobRollCounters, &SteeringConfig, rng, knob, prob)` primitive
  (the borrow shape the hierarchy planner can actually call), the seven helpers deleted,
  and `KnobRollCounters::record` privatized into the primitive's module so a second
  primitive is a **compile error** (rung **R2**; no new registered doctrine). `.4`
  registered for the separate, louder gap: the 16 Bernoulli knobs with no `KnobId`.
  Docs-only ⇒ DUT byte-identical.
- `2026-07-30`: `.3b` **the fix landed** (code). `src/ir/knob_roll.rs` is new and holds
  the crate's single knob-roll primitive: `KnobRollCounters` (moved from `ir/types.rs`,
  re-exported so `Module.knob_rolls` and every `crate::ir::KnobRollCounters` path is
  unchanged) plus `roll_knob_into(&mut KnobRollCounters, &SteeringConfig, &mut impl Rng,
  KnobId, f64)`. **`KnobRollCounters::record` is private to that module**, so a second
  roll primitive is a compile error (`error[E0624]`) — repair rung **R2**, no registered
  doctrine added. `Generator::roll_knob(&mut self, m, knob, prob)` in `src/gen/mod.rs` is
  the crate-wide shim; `cone::roll_knob` survives as a free-function alias so its 37 call
  sites keep their spelling; `src/gen/hierarchy.rs` now holds **zero** roll helpers and
  names its `KnobId` inline at each of its seven decision sites. Verified end-to-end:
  `child_input_cone` `0/5 → 5/5` under `--steer hierarchy_child_input_cone_prob=9.0`
  *and* `--steer hierarchy=9.0` — exactly `clamp01(0.3 × 9) = 1.0`, where before the fix
  every steer left it at `0/5`. Unsteered output byte-identical against three pre-fix
  hashes measured earlier in the same session, plus `tests/snapshots.rs` 6/6. New proofs:
  `steering_shifts_hierarchy_category_construct_distribution` (per-knob, **both** weight
  directions over a 12-seed design sweep) and
  `neutral_hierarchy_steering_is_byte_identical_on_designs`, plus three unit proofs in
  `knob_roll.rs`. Both guards negative-controlled in both directions. Full `COMMIT.md`
  cargo gate green.
- `2026-07-30`: `.3b` **corrected two claims made at `.3a`**, via a dated *Correction*
  section appended to decision `0034` (an explicit amendment, never a silent rewrite).
  (1) The stated root cause was wrong: the fork was **not** forced by a borrow conflict —
  `g.roll_knob(ctx.top, KnobId::X, g.cfg.x)` compiles at all seven sites with zero
  warnings (two-phase borrows). It was plain **visibility**: the pre-`.3b` `roll_knob` was
  a module-private `fn` in `gen::cone`, invisible to its sibling `gen::hierarchy`. The
  replacement lesson is better than the one it replaces — *a private helper carrying a
  cross-cutting invariant is an invitation to fork it* — and it is exactly what the R2
  guard answers, since the guard is on the **effect** (`record`) rather than on the
  wrapper. (2) `.3a`'s "16 of 22 reached" over-counted: `HierarchyParentFlopProb` is
  steered at 2 of its 3 sites, so the honest split is **15 fully steered / 6 not at all /
  1 partial**. No measurement changes (all are per-knob).
- `2026-07-30`: `.3c` docs landed and **`.3` is closed**. `book/src/algorithm.md`'s
  steering section now names the primitive correctly, carries the compile-time-guard
  bullet, states the reach precisely (every knob in the coverage readout is steerable at
  *every* decision site; knobs outside the set error rather than being ignored), and gains
  a **"Why the guard is a compile error"** subsection with the two transferable rules —
  *guard the effect, not the wrapper* and *a private helper carrying a cross-cutting
  invariant is an invitation to fork it* — plus the per-knob/two-sided test rule.
  `book/src/knobs.md` gains a **runnable** two-command example proving `0/5 → 5/5` under
  `--steer hierarchy=9.0`, with the diagnostic that a rate coming back *identical* rather
  than merely close is the signature of a roll site that escaped the primitive.
  `USER_GUIDE.md` gains the reach paragraph (with the real `unknown steer key` error text)
  and an explicit callout that `--steer hierarchy` was inert before `2026-07-30`, so a
  corpus tuned in that window is worth repeating. `README.md`'s steering bullet was
  corrected **net-neutral in length** (`CLAUDE.md` §14). Decision `0034` was enriched as
  the KM card: `status: accepted → delivered`, four new answer keys, evidence rewritten to
  the shipped surface, and a `reverify` command. Frontier advances to `.4`.
- `2026-07-30`: `.3c` also **registered two findings it surfaced**, in the same turn,
  per the owner's standing *"a defect is only handled if a task-tree owns it"* directive.
  (1) `BOOK-EXAMPLES-RUNNABLE.3`: the book harness's "no silent skips" guard is itself
  defeatable — the reason extractor trims reason punctuation off the *front* before
  stripping the closing `-->` off the back, so a reasonless `<!-- book-test: skip -->`
  yields the non-empty reason `">"` and passes. Found by writing exactly that sentinel and
  watching the suite stay green; confirmed with a compiled probe over the real parse. The
  example was made genuinely runnable instead, so no reasonless sentinel is left in the
  tree. (2) The new `README-POLICY-ADOPTION` tree: `CLAUDE.md` §14 directs adoption of the
  README Stability Policy, and it has never been implemented — `README.md` measures **1771
  lines / 122,767 bytes** against the policy's `300` / `16,384`, with no repo-root
  `README_POLICY.md`. Neither is `COVERAGE-STEERED-GENERATION` work; both are now owned.
- `2026-07-30`: `.4a` design ADR landed (decision
  [`0035`](../decisions/0035-steering-width-motif-and-emission-knobs.md)), opening
  steering's **width** after `.3` closed its *reach*. Measured: `Config` carries **41**
  `f64` probability knobs (exactly the set `Config::validate` range-checks into
  `[0,1]`); **22** have a `KnobId`; of the remaining 19, **16** are in scope and **3**
  are excluded **by kind** — `operand_duplication_rate` / `mux_arm_duplication_rate` are
  dedup *thresholds* compared in `ir/compact.rs` + `metrics.rs`, not Bernoulli rolls
  (there is nothing for a prior to multiply), and `library_prob` has **no reader
  anywhere in `src/`**. Blast radius counted before splitting: the nine `annotate_*`
  passes have **99** in-crate call sites, mostly their own `#[cfg(test)]` tests. Decided:
  two **new** categories (`motifs`, `emission`) with the existing six keeping their exact
  membership; a `&SteeringConfig` parameter on the nine passes; the `if cfg.<knob> > 0.0`
  guard pinned as **load-bearing for reproducibility** (removing it would consume 16+
  extra RNG draws on the default path) together with its honest consequence — a
  default-off knob records `attempts = 0`, so the readout cannot distinguish *off* from
  *never reached*; and **no** introspection schema bump, with the general rule pinned for
  future contributors (bump for a new payload key or query kind, not for more entries in
  an existing map). Two things were **flagged rather than decided**:
  `src/gen/module.rs:143` rolls a hard-coded `gen_bool(0.5)` with no knob at all
  (steering-gap-3 territory), and `KnobId::all()` reaching **38** hand-maintained entries
  needs an R2 guard — proposed as a private exhaustive `index()` match plus a derived
  length test, to be confirmed against the real code at `.4b.1`. Docs-only ⇒ DUT
  byte-identical.
- `2026-07-30`: **`.4b.1` recon found two facts `.4a` did not account for**; both recorded
  as a dated *Correction* section on decision `0035` rather than edited into it silently.
  (1) **Three of the seven motif rolls have no `Module` to record into.**
  `width_parameterization_prob` / `memory_prob` / `fsm_prob` roll at
  `src/gen/module.rs:385/401/414` and each **`return`s a differently-built module** at
  `390/404/417`, while that function's own first `Module` binding is at line `432` — the
  roll *chooses which module to construct*. So `.4b.1` is design work, not mechanical
  routing: the proposed shape is a `Generator::pending_knob_rolls` buffer that pre-module
  rolls record into, drained into `m.knob_rolls` once the module exists, with the drain
  living **inside** `ir::knob_roll` so the one-writer property survives. Recording after
  the fact from the call site is rejected on sight — that is exactly the "roll here,
  record there" split decision `0034` exists to prevent. (2) **The `.3b` R2 guard is
  incomplete** — see `.5` below.
- `2026-07-30`: **`.5` registered** (`pending`, ordered *before* `.4b.1`). Measured with a
  compile probe: privatising `KnobRollCounters::record` at `.3b` left `attempts` and
  `fires` as `pub` fields, so a second roll primitive that skips the steering prior and
  writes the maps **directly** compiles **clean** (`cargo check --all-targets` exit `0`).
  The guard blocks the obvious route and not the equivalent one — which, by `.3b`'s own
  stated principle (*guard the effect, not the wrapper*), means it does not yet guard the
  effect. Fix: private fields + read-only accessors (the only external consumer is
  `metrics::compute`). Ordered first because `.4b.1` adds a **new** writer path and should
  be designed against a complete guard. The transferable lesson, recorded on `0035`: when
  building an R2 guard, **enumerate every way the protected state can be written, not just
  the intended one** — `0034`'s own "search the effect, not the shape" rule, turned on the
  guard instead of on the defect.
- `2026-07-30`: `.5` landed — **the guard now covers the state, not just the API**.
  `KnobRollCounters.attempts` / `.fires` are private, readable through new read-only
  `attempts()` / `fires()` accessors; the single external consumer (`metrics::compute`)
  is a two-line change. Negative-controlled both ways: the direct-field bypass probe,
  which compiled **clean** before `.5`, now fails with
  `error[E0616]: field 'attempts' of struct 'KnobRollCounters' is private` (and the same
  for `fires`), and removing it restores a clean build. Together with `.3b`'s `E0624` on
  `record`, **both** syntactic routes that write the protected state are compile errors.
  No generation path was touched — visibility plus two accessors — so unsteered and
  default emission stay byte-identical (snapshots 6/6). The rule this pair adds up to is
  now in the module docs: *when an invariant is made a compile error, the protected thing
  is state, not a function — enumerate every syntactic route that mutates it, not just
  the one you happened to write.*
- `2026-07-30`: `.4b.1` landed — **the `motifs` category is a real dial**, and the symptom
  that opened this session is closed: `--memory-prob 0.5 --introspect` returned an
  **empty** `coverage_readout` (a memory leaf is built entirely by rule, so no cone knob
  ever rolled) and now returns `memory_prob {attempts: 1, fires: 1}` under a `motifs`
  category. Over 24–30 seeds at base `0.25` the achieved rate moves `0.267` unsteered →
  `0.833` at `motifs=3.0` → `0.033` at `motifs=0.05`. **17 configurations byte-identical**
  against a `HEAD` binary built in an isolated git worktree, including every motif knob
  turned on individually and combined.
- `2026-07-30`: `.4b.1` **found and fixed a bug in its own first cut, by reading raw
  counts rather than the derived rate.** The pending-roll buffer was drained only inside
  the three *firing* branches, so a motif roll that came up false recorded an attempt
  nobody saw. Fire counts still moved correctly with the steer, so the feature looked
  right — but `attempts == fires`, making every reported `fire_rate` a constant `1.000`.
  Fixed structurally (one drain outside every exit path, so a missed branch is
  unconstructible) and negative-controlled. Durable lesson recorded in
  `DEVELOPMENT_NOTES.md`: *a ratio whose numerator and denominator share a failure mode
  cannot show you that failure — print the raw counts at least once.*
- `2026-07-30`: `.4b.1` also **deleted a guard of its own that could not fail**, and
  registered `.6` instead. The `KnobId::all()` guard was drafted as an exhaustive
  `index()` match *plus* a length assertion (`all().len() == max_index + 1`). Negative
  control: dropping the last entry from `all()` left the test **green** — the expected
  count was derived from `all()` itself and shrank with it. Shipping it would have put an
  S3 "gate that cannot fail" *inside the guard against S3 defects*. What ships is honest:
  the compile error (`E0004`) plus an ordering test that catches misordering and middle
  omissions, both negative-controlled, with the **tail-truncation gap written down at the
  test** together with the reason a length check cannot close it. The R1 repair — generate
  the enum, `all`, `name` and `category` from one macro table so the list stops existing —
  is `.6`.
- `2026-07-30`: `.4b.2` landed — **the nine structured-emission surfaces are measurable
  per-gate for the first time**, and the `emission` category steers them as one family.
  `--profile structured-emission-max --introspect` previously reported **no** emission
  knob at all; it now reports all eight non-version-gated surfaces with their own
  attempts counts, every achieved rate ≈ `0.25` — decision `0032` calibrated that value
  by external measurement, and it is now *observable from the artifact itself*. The
  category steers `0.397 → 0.069` at `--steer emission=0.2`. Nine `annotate_*` passes
  gained a `steering: &SteeringConfig` parameter; ~99 in-crate call sites updated (18
  production, the rest each pass's own `#[cfg(test)]` callers). **16 configurations
  byte-identical** against a `HEAD` binary built in an isolated worktree — every surface
  individually, the 2023 `soft_union` up-opt, the preset, all eight at `0.25` together,
  a hierarchy design, and a steered run.
- `2026-07-30`: with `.4b.2`, **steering's width is delivered**. All 38 `KnobId`s are
  steered at every one of their roll sites (`.3b` + `.5` make a second, unsteered
  primitive a compile error on both the method and the state) and every one of them
  appears in the coverage readout when its knob is on. The decision-`0017`
  API-completeness gate now holds for every ANVIL capability that actually rolls.
  Remaining: `.4c` (docs, closes `.4`) and `.6` (macro-derive `KnobId::all()`, rung R1).
- `2026-07-30`: `.4c` landed and **`.4` is closed — steering is now complete in both
  dimensions.** *Reach* (`.3` + `.5`): exactly one knob-roll primitive, with a second
  made a compile error on both the method (`E0624`) and the state (`E0616`). *Width*
  (`.4`): all **38** `KnobId`s are steered at every one of their roll sites and appear in
  the coverage readout when their knob is on. Decision `0017`'s API-completeness gate now
  holds for every ANVIL capability that actually rolls; the three probability-shaped
  knobs that are *not* rolls error loudly rather than silently no-op.
- `2026-07-30`: `.4c` found the category taxonomy copied into **six** live documents and
  bound them with a fourth `ENUMERATION-PARITY` pair. The failure mode is worse than a
  plain omission and worth recording: `--steer` **errors** on an unknown key, so a user
  reading a stale six-name list never learns the two new categories exist — the feature
  would be delivered and **invisible**. Negative-controlled both ways. Separately,
  `DOCTRINE_ENFORCEMENT.md`'s hand-written *"Three today"* pair count was **deleted rather
  than incremented**: a number beside a list is one more copy of it, and decision `0033`
  repairs that by deletion, never by gating the count too.
- `2026-07-31`: `.6` closed the tree by **deleting the list instead of fortifying its
  guard**. `.4b.1` had guarded `KnobId::all()` at rung **R2** and documented the gap it
  could not close — a *tail* truncation. The patch for that gap cannot exist: a count
  derived from `all()` shrinks with it and cannot fail, and a hand-written count is the
  second copy decision `0033` forbids as a repair. That impossibility is the diagnostic —
  **when a guard's residual gap can only be closed by adding another hand-written list,
  the guard is at the wrong rung** — so one `knob_ids!` table now expands to the enum,
  `all()`, `name()` and `category()`, each variant name goes from **5** occurrences to
  **1**, and both guards are deleted with the list. The omission has no syntactic form
  left: skipping a knob means deleting its row, which deletes the variant.
  The slice also **disproved its own written prediction**, which is recorded rather than
  quietly amended: leaving `ENUMERATION-PARITY`'s extractor on `pub fn category` does
  *not* read zero and trip the floor — it reads the correct **8**, because its range
  terminator stopped existing and the range over-ran 162 lines into the table, where a
  `"[a-z]+"` scan skips knob names only by the coincidence that all of them contain `_`.
  One probe row named `"probe"` turns that into a **phantom** category and the gate starts
  crying wolf. General rule now on file: *a `sed` line-range whose terminator stops
  existing does not fail — it runs on and returns something plausible.*
  Three first-cut negative controls were **too weak to fail** (a `datapathXX` mask that
  `covers_set` still substring-matches; a reshaped row for a category with three rows; and
  a prediction never actually run) and two byte-identity comparisons were **vacuous** —
  both sides hashed `e3b0c44298fc`, the SHA-256 of the empty string, because the run
  errored. Caught, re-run, and the harness now refuses to score an empty or failed run.
