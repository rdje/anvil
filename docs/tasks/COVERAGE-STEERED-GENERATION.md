# COVERAGE-STEERED-GENERATION: construction-time coverage-feedback steering

## Metadata

- Tree ID: `COVERAGE-STEERED-GENERATION`
- Status: `active`
- Roadmap lane: `Usability / effectiveness — coverage-steered generation (north star, idea 6)`
- Created: `2026-06-17`
- Last updated: `2026-07-30` (`.3b` the fix landed — one steering-aware knob-roll primitive, a second one now a compile error; `--steer hierarchy` works. `.3a` design ADR = decision `0034`, since amended with a dated Correction. Frontier: `.3c` docs/close, then `.4`. The `.1`/`.2` scope stays closed and is not revisited.)
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
  Closure: `The .1/.2 scope closed 2026-06-22 and is NOT reopened: every acceptance criterion of that scope was met (the roll_knob per-category/per-knob prior multiplier measurably shifts the distribution rules-first + reproducible (.2a); the achieved coverage is API-queryable (.2b) and the target API-settable (.2c.1) per decision 0017; unsteered DUT byte-identical; documented (.2c.2)). The tree returns to 'active' on 2026-07-30 for the new .3 node (decision 0034: a MEASURED SILENT NO-OP in the shipped surface — 6 of the 22 KnobIds, and therefore the whole documented 'hierarchy' steering category, are never reached by the prior because src/gen/hierarchy.rs defines seven roll primitives of its own) and .4 (the recorded decision-0023 follow-up: the 16 Bernoulli knobs that have no KnobId at all). Nothing retired (feedback_never_retire_strategies); the Phase-4 closure pattern — a closed scope stays closed, new work lands as new nodes.`
  Children: `COVERAGE-STEERED-GENERATION.1`, `.2`, `.3`, `.4`

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
  Status: `active`
  Goal: `Make the steering prior reach EVERY roll site of every KnobId. Opened 2026-07-30 on a measured defect: src/gen/hierarchy.rs:883-938 defines seven roll primitives of its own that record the same m.knob_rolls telemetry as roll_knob while omitting SteeringConfig::effective_prob, so 6 of the 22 KnobIds are never steered, HierarchyParentFlopProb is half-steered, and --steer hierarchy=<w> is a silent no-op that the CLI/config/--dump-config all accept and echo. Scoped to the existing 22-knob KnobId universe; widening that universe is .4.`
  Acceptance: `set at .3a (decision 0034): exactly one steering-aware knob-roll primitive exists and a second one is a COMPILE ERROR (KnobRollCounters::record privatized into the primitive's module — repair rung R2, no new registered doctrine); all 7 hierarchy call sites route through it with an explicit KnobId at the decision site; a hierarchy-category distribution-shift regression proof (the sibling of .2a's state-category proof) is green; a neutral weight (1.0) and no --steer are both byte-identical (tests/snapshots.rs 6/6 untouched); full COMMIT.md cargo gate.`
  Verification: `in progress — .3a (design ADR, decision 0034) and .3b (the fix + proofs) are done; --steer hierarchy now measurably biases construction (child_input_cone 0/5 -> 5/5) and a second roll primitive is a compile error. .3c (docs/close) remains.`
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
  Status: `pending`
  Goal: `DOCS + close .3: book/src/algorithm.md's steering section states the one-primitive invariant and the compile-time guard (it currently says the prior applies "at the roll_knob site", which was true of 16 of 22 knobs); book/src/knobs.md + USER_GUIDE.md steering rows reflect that every KnobId is steerable at every roll site; a KM card; mark .3 done and refresh docs/TASK_TREE.md.`
  Acceptance: `book + USER_GUIDE state the invariant accurately with a runnable hierarchy-steer example; mdbook build clean; cargo test --test book_examples green; KM regen+check green; .3 marked done. Docs-only ⇒ DUT byte-identical.`
  Verification: `pending`
  Commit: `pending`

- ID: `COVERAGE-STEERED-GENERATION.4`
  Status: `pending`
  Goal: `The decision-0023 follow-up proper — steering's WIDTH, not its reach. Give the 16 remaining Bernoulli knobs a KnobId and route them through the one .3b primitive so they gain construction-time telemetry AND steerability together: 7 module-level motif rolls (width_parameterization_prob, memory_prob, fsm_prob, fsm_mealy_prob, multi_clock_prob, aggregate_prob, aggregate_array_prob) + 9 emit-projection rolls (soft_union_slice_prob, function_emit_prob, generate_loop_emit_prob, task_emit_prob, multi_output_task_emit_prob, cone_function_emit_prob, mux_if_emit_prob, case_mux_if_emit_prob, casez_mux_if_emit_prob). Their symptom is LOUD (--steer memory_prob=2.0 errors "unknown steer key"), so this is a feature gap, not a defect — which is why it is separate from .3.`
  Acceptance: `to be set at .4a. Must decide: whether the 16 join the existing six categories or introduce motifs/emission; whether widening coverage_readout's key set warrants an introspection schema MINOR bump; and how the emit-projection passes obtain &SteeringConfig (they currently take only &mut Module + rng + p). EXCLUDED by kind: operand_duplication_rate / mux_arm_duplication_rate (dedup thresholds compared in ir/compact.rs + metrics.rs, not Bernoulli rolls — there is no roll to apply a prior to) and library_prob (no reader anywhere in src/; a documented-reserved orphan recorded by COVERAGE-INSTRUMENTATION.3). Default-off/unsteered must stay byte-identical.`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `COVERAGE-STEERED-GENERATION.1` | `done` | Design ADR (decision `0023`) pinned the rules-first steering primitive (a prior multiplier at `roll_knob`, not a filter), the byte-stability contract, the `SteeringConfig` target model, the SCHEMA-DERIVED achieved-coverage readout, the outer measure→derive→re-steer loop, and the decision-`0017` API surface. |
| 2 | `COVERAGE-STEERED-GENERATION.2a` | `done` | Steering core landed: `KnobId::category()`, `SteeringConfig` + `weight()`/`effective_prob()`, the `roll_knob` prior multiplier, `ConfigError::SteeringWeight`. All three proofs green (byte-identical default via snapshots 6/6; measurable distribution shift; no-filter architectural) + full cargo gate. |
| 3 | `COVERAGE-STEERED-GENERATION.2b` | `done` | Achieved-coverage readout landed: `src/introspect/coverage.rs` (`CoverageReadout` + per-knob/per-category fire rates + the gate/operand/depth histograms), embedded as `IntrospectionPayload::coverage_readout` (schema `1.11→1.12`) + the standalone MCP `coverage` query (`CoverageDocument`), one projection feeding both. SCHEMA-DERIVED / DUT `.sv` byte-identical; `fire_rate` integer-ppm for byte-stable determinism. Full cargo gate green. |
| 4 | `COVERAGE-STEERED-GENERATION.2c.1` | `done` | Outer-loop CODE landed: the pure `derive_steering_from_coverage(&CoverageReadout, &DeriveParams) -> SteeringConfig` helper (decision `0023` §4, milli-quantized weights) + `SteeringConfig::set_weight` + the repeatable `--steer <key>=<weight>` CLI shim into `Config.steering` (preset-then-explicit in `resolve_config`). Unsteered default byte-identical (snapshots 6/6); CLI smoke proves steered≠unsteered, neutral=unsteered, bad-key errors. Full cargo gate green. |
| 5 | `COVERAGE-STEERED-GENERATION.2c.2` | `done` | The DOCS + close landed: book steering section (`algorithm.md`) + `agent-mcp.md` coverage-steering section + the schema-`1.12` book-example refresh (`agent-mcp`/`api-tools`/`api-introspection`/`api-reference`) + knobs.md cross-ref + USER_GUIDE `--steer`/recipe + KM (decision `0023` enriched). mdbook clean; book_examples 3/3 (runnable `--steer` example). **`.2c` + `.2` + the tree CLOSED.** |

| 6 | `COVERAGE-STEERED-GENERATION.3a` | `done` | Design ADR (decision `0034`) for the `.3` node: the measured silent no-op (6 of 22 `KnobId`s unreachable by the prior; a 9× steer leaves the recorded fire counts bit-identical; an 800× spread emits byte-identical SV), the root cause (a second roll primitive predating the steering core by two months, missed because the `.1` survey searched by shape instead of from `knob_rolls.record(`), the one-primitive fix, the **R2** compile-time guard, and the `.3`/`.4` scope boundary. Docs-only. |
| 7 | `COVERAGE-STEERED-GENERATION.3b` | `done` | The fix landed: `roll_knob_into` in the new `src/ir/knob_roll.rs` with `record` privatized (a second primitive is now `error[E0624]`), `Generator::roll_knob` as the crate-wide shim, `cone::roll_knob` reduced to an alias (37 call sites untouched), the seven `roll_hierarchy_*` helpers deleted. `child_input_cone` goes `0/5 → 5/5` under `--steer hierarchy=9.0`; unsteered byte-identical. Both guards negative-controlled both ways. Corrected `.3a`'s root cause: visibility, not borrows. |
| 8 | `COVERAGE-STEERED-GENERATION.3c` | `pending` | **Next.** Docs + close `.3`: `book/src/algorithm.md`'s steering section says the prior applies "at the `roll_knob` site" — true of 15 of 22 knobs until `.3b`. Restate as the one-primitive invariant + the compile-time guard; `knobs.md` + USER_GUIDE + a KM card. |
| 9 | `COVERAGE-STEERED-GENERATION.4` | `pending` | Steering's **width**: give the 16 remaining Bernoulli knobs (7 motif + 9 emit-projection) a `KnobId` and route them through the `.3b` primitive so they gain telemetry and steerability together. Follows `.3` because it needs the single primitive to route into. |

**Tree status: `active` (`2026-07-30`).** The `.1`/`.2` scope stays closed
(`2026-06-22`) and is not revisited. Open frontier: `.3b`, then `.3c`, then `.4`.
The remaining decision-`0023` follow-up — the in-generator adaptive schedule — is
still a future `.N` and is unaffected by either.

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
| `2026-06-17` | `COVERAGE-STEERED-GENERATION` | `tree registered (docs-only); no code` | `registered` |
| `2026-06-21` | `COVERAGE-STEERED-GENERATION.1` | `decision 0023 written; INDEX + tree + TASK_TREE + DEVELOPMENT_NOTES updated; KM regen+check green; mem-arch green; docs-only / DUT byte-identical` | `done` |
| `2026-06-21` | `COVERAGE-STEERED-GENERATION.2a` | `SteeringConfig + KnobId::category() + roll_knob prior multiplier + ConfigError::SteeringWeight; cargo check --all-targets, cargo test (snapshots 6/6 + new steering unit/integration tests), cargo clippy -D warnings, cargo fmt --check all green; rules-first / DUT byte-identical when unset` | `done` |
| `2026-06-21` | `COVERAGE-STEERED-GENERATION.2b` | `src/introspect/coverage.rs (CoverageReadout + module_coverage/design_coverage) + KnobId::all()/category_of_name() + IntrospectionPayload::coverage_readout + CoverageDocument + MCP coverage tool; schema 1.11→1.12 + schema doc §5/§6.8/changelog; fire_rate integer-ppm determinism fix (caught by introspect_tool_round_trips); cargo check --all-targets, cargo test (snapshots 6/6 + new coverage unit + introspect/mcp coverage tests), cargo clippy -D warnings, cargo fmt --check all green; SCHEMA-DERIVED / DUT .sv byte-identical` | `done` |
| `2026-06-21` | `COVERAGE-STEERED-GENERATION.2c.1` | `src/introspect/coverage.rs derive_steering_from_coverage + DeriveParams; src/config.rs SteeringConfig::set_weight + pub validate + Overrides.steer + resolve_config steer application + ConfigError::UnknownSteerKey; src/main.rs --steer flag + parse_steer_arg + cli_overrides; 9 new tests (3 derive + 4 config steer + 2 main CLI); cargo check --all-targets, cargo test (snapshots 6/6), cargo clippy -D warnings, cargo fmt --check all green; CLI smoke (steered≠unsteered, neutral=unsteered, bad-key error); unsteered default DUT byte-identical` | `done` |
| `2026-07-30` | `COVERAGE-STEERED-GENERATION.3b` | `cargo check --all-targets` (clean, 0 warnings), `cargo clippy --all-targets -- -D warnings` (clean), `cargo fmt --all --check` (clean), `cargo test` (green: lib 748/0, snapshots 6/6, book_examples 3/3, pipeline green incl. the 2 new proofs). End-to-end fix verified on the release binary: `child_input_cone` 0/5 -> 5/5 under both `--steer hierarchy_child_input_cone_prob=9.0` and `--steer hierarchy=9.0`. Unsteered byte-identity confirmed against 3 pre-fix hashes measured earlier in the same session. Both guards negative-controlled BOTH ways (E0624 on reintroducing the helper shape; the distribution test FAILS when one call site is rewired unsteered). | `done` |
| `2026-07-30` | `COVERAGE-STEERED-GENERATION.3a` | `decision 0034 written + INDEX row; tree reopened (.3 container + .3a/.3b/.3c + .4) ; docs/TASK_TREE.md row refreshed; MEMORY/CHANGES/DEVELOPMENT_NOTES/ROADMAP synced; KM regen+check green; scripts/check_doctrines.sh all 7 PASS. Measurement re-run at ff506e1 against target/release/anvil: 16/22 KnobIds steered, 6 unreachable, hierarchy fire counts bit-identical under a 9x steer (0/5 vs the demanded 5/5), 800x category spread byte-identical, --steer state=8.0 positive control effective. Docs-only ⇒ DUT byte-identical (no src/ touched).` | `done` |
| `2026-06-22` | `COVERAGE-STEERED-GENERATION.2c.2` | `book/src/{algorithm.md steering section, agent-mcp.md coverage-steering section + coverage tool + --introspect coverage_readout, api-tools.md coverage tool, api-introspection.md coverage_readout schema, api-reference.md, knobs.md cross-ref} + schema 1.11→1.12 example refresh; USER_GUIDE.md (--steer row + Coverage steering subsection + recipe); decision 0023 enriched (4 shipped-surface answer keys + status delivered) = KM card. mdbook build clean; cargo test --test book_examples 3/3 (runnable --steer example exit 0); KM regen+check green. Docs-only / DUT byte-identical. Closes .2c + .2 + the tree.` | `done` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `COVERAGE-STEERED-GENERATION` | `USABILITY-LANE-OWNERSHIP.1 — register 7 owner-directed usability/capability lanes + API-first decision 0017` | Tree registered (not yet started); frontier `.1` (design ADR) pending. |
| `COVERAGE-STEERED-GENERATION.1` | `COVERAGE-STEERED-GENERATION.1 — design ADR (decision 0023)` | Design-only; pins the rules-first prior-multiplier steering primitive at `roll_knob`, the byte-stability contract, the `SteeringConfig` target, the SCHEMA-DERIVED coverage readout, the outer feedback loop, and the API surface; pre-splits `.2` into `.2a`/`.2b`/`.2c`. |
| `COVERAGE-STEERED-GENERATION.2a` | `COVERAGE-STEERED-GENERATION.2a — steering core (SteeringConfig + roll_knob prior multiplier)` | First code slice: `KnobId::category()` (exhaustive 21-variant taxonomy), `SteeringConfig` (`per_knob`/`per_category` weights + `weight()`/`effective_prob()`/`is_empty()`/`validate()`), `Config.steering` (only `skip_serializing_if`), `ConfigError::SteeringWeight`, the `roll_knob` prior multiplier. Three proofs green (byte-identical default; distribution shift; no-filter) + full cargo gate. Rules-first / DUT byte-identical when unset. |
| `COVERAGE-STEERED-GENERATION.2b` | `COVERAGE-STEERED-GENERATION.2b — achieved-coverage readout (--introspect section + MCP coverage query)` | Second code slice (the READ half): `src/introspect/coverage.rs` (`CoverageReadout` + `module_coverage`/`design_coverage`), `KnobId::all()`/`category_of_name()`, the `coverage_readout` payload section (schema `1.11→1.12`), the `CoverageDocument` envelope + the pure MCP `coverage` tool (one projection feeding both). Schema doc §5/§6.8/changelog. `fire_rate` integer-ppm for byte-stable determinism (1-ULP fix caught by the pre-existing round-trip test, not weakened). SCHEMA-DERIVED / DUT `.sv` byte-identical; full cargo gate green. |
| `COVERAGE-STEERED-GENERATION.2c.1` | `COVERAGE-STEERED-GENERATION.2c.1 — outer-loop derive helper + --steer CLI shim` | Third code slice (the steering-OUT half): the pure `derive_steering_from_coverage` helper (decision `0023` §4, milli-quantized weights) + `SteeringConfig::set_weight` (one classifier) + `pub validate` + `Overrides.steer` + the repeatable `--steer <key>=<weight>` CLI shim (preset-then-explicit in `resolve_config`). 9 new proofs + CLI smoke (steered≠unsteered, neutral=unsteered, bad-key error). Unsteered default DUT byte-identical; full cargo gate green. |
| `COVERAGE-STEERED-GENERATION.3b` | `COVERAGE-STEERED-GENERATION.3b — one steering-aware knob-roll primitive` | The fix. New `src/ir/knob_roll.rs` (the single primitive + the privatized `record`), `Generator::roll_knob` shim, `cone::roll_knob` reduced to an alias, the seven `roll_hierarchy_*` helpers deleted. `--steer hierarchy` starts working (`0/5 -> 5/5`); unsteered byte-identical. Corrects `.3a`'s root cause via a dated Correction section in decision `0034`: the fork was **visibility** (a module-private `fn` in `gen::cone`), not a borrow conflict. |
| `COVERAGE-STEERED-GENERATION.3a` | `COVERAGE-STEERED-GENERATION.3a — design ADR (decision 0034)` | Reopens the tree with `.3`. Design-only: records the measured silent no-op (the `hierarchy` steering category biases nothing; 6 of 22 `KnobId`s never reach the prior), root-causes it to a second roll primitive that the `.1` shape-keyed survey missed (decision `0033` rule 2 recurring), and pins the one-primitive fix + the **R2** compile-time guard + the `.3`/`.4` scope boundary. Docs-only ⇒ DUT byte-identical. |
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
