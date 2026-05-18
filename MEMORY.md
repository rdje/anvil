# Memory
Compact, operational continuity snapshot. Read on session bootstrap. Keep only what is actionable.

## Current state
- **DOCTRINE (2026-05-17, non-negotiable, owner directive):** no code change may be made without a task-tree leaf owning it **first** (`src/`/`tests/`/`examples/`/build-codegen = code; pure-docs/mdBook/workflow/doctrine edits exempt). `rN` survives only as the optional within-leaf cadence. Recorded in `docs/TASK_TREE.md` "ANVIL Adoption Scope", `COMMIT.md`, `SESSION_BOOTSTRAP.md`, `DEVELOPMENT_NOTES.md`, `README.md`, `book/src/architecture.md`. Supersedes the earlier "task trees opt-in / `rN` for linear coverage" scope.
- **Session recovery:** a `PostCompact` hook in `.claude/settings.json` auto-re-injects the full `SESSION_BOOTSTRAP.md` as context after every compaction. After an auto/manual compact, re-run the SESSION_BOOTSTRAP recovery protocol — do not assume prior in-context state survived. (`.claude/settings.local.json` carries the local `Write(.claude/settings.json)` allow rule; it stays uncommitted.)
- **Phase:** Phase 0 done. Phase 1 (Single-module MVP) is done. Phase 2 (Signal sharing / DAG cones) is done. Phase 3 (structured combinational ops) is done. **Phase 4 (hierarchy) is done** (2026-05-16, closed by `PHASE-4-HIERARCHY.3` as a deliberate evidence-backed scope cut against explicit ROADMAP exit criteria; closing artifact r87, `coverage_gaps=[]`, 840/0). **Phase 5 (parameterization) is done** (2026-05-17, `PHASE-5-PARAMETERIZATION` tree complete: rules-first width-parameterizable leaves + `#(.W(v))` instantiation + parameter-aware identity, closed against explicit ROADMAP exit criteria; closing artifact `/tmp/anvil-tool-matrix-phase5-p1` — 213 scenarios / 852 designs, `coverage_gaps=[]`, Verilator+both-Yosys 852/0, `saw_width_parameterized_design=true`). **Phase 5b (synthesizable aggregates) is done** (2026-05-18, `PHASE-5B-AGGREGATES` tree complete: opt-in `aggregate_prob` packed-`struct` emitter projection over the flat IR — annotation-only, default-off byte-identical; closed against explicit ROADMAP exit criteria; closing artifact `/tmp/anvil-tool-matrix-phase5b-p1` — 216 scenarios / 864 designs, `coverage_gaps=[]`, Verilator+both-Yosys 864/0, `saw_packed_aggregate_design=true`, Phase 4/5 regressions clean). **Phase 6 (advanced motifs) is the next numbered phase, not started.**
- **Active task trees:**
  - `HIERARCHY-AWARE-IDENTITY` — **tree complete** (all five leaves `done`): `H-A-I.1` (canonical signatures, r85), `H-A-I.2` (existence proof, r86), `H-A-I.3` (design sketch), `H-A-I.4` (dedup-pass implementation, r87), `H-A-I.5` (matrix gate proof, r87 same commit). The doctrine "NodeId = identity of an expression" now extends to "ModuleId = identity of a hierarchical module template" under the opt-in `Config::hierarchy_module_dedup` knob.
  - **Every remaining roadmap phase is now task-tree-tracked (2026-05-16 owner directive):**
    - `PHASE-4-HIERARCHY` — **tree complete** (`.1` done — surface landed-proven; `.2` superseded; `.3` done — Phase 4 closed via deliberate evidence-backed scope cut against explicit ROADMAP exit criteria, closing artifact r87). **ROADMAP Phase 4 = done.**
    - `PHASE-5-PARAMETERIZATION` — **tree complete** (`2026-05-17`, all leaves `done`; ROADMAP Phase 5 = `done`). `.1` design (architecture (C); (A)/(B)/(C') rejected). `.2.1` scaffold (`WidthExpr{Lit,Param}`+`ParamEnv`, additive `Module` fields, opt-in `Config::width_parameterization_prob` default 0.0, `src/ir/param.rs` annotation-only post-pass, emitter `#(parameter int W=D)`+`[W-1:0]`). `.2.2.1` soundness primitives + **rules-first pivot** (post-hoc filter inert/generate-then-filter). `.2.2.2` `src/gen/module.rs::build_parameterizable_leaf` (width-homogeneous combinational leaf by rule; param.rs rolling→non-rolling `annotate_parameterized`) — feature **fires by construction**. `.2.2.3a/b` `Instance.param_bindings`+emitter `#(.W(v))` + hierarchy override pick + resolved-width validate (**end-to-end functional**; soundness-scoped to the planned-child loop). `.2.3` param-aware `canonical_module_signature` (dedup unchanged; r87/H-A-I regression-clean). `.2.4a` `phase5_width_parameterized` tool_matrix scenario + metrics + `saw_width_parameterized_design` fact/gap (213/852; phase4 bin 3/3). **`.2.4b` closed Phase 5**: real repo-owned `Phase4Hierarchy` gate verified CLEAN on `/tmp/anvil-tool-matrix-phase5-p1` (213 scenarios / 852 designs, `coverage_gaps=[]`, Verilator+both-Yosys 852/0, `saw_width_parameterized_design=true`); promotion strictly followed the verified artifact (r87 no-aspirational-claims) — ROADMAP Phase 5 `(not started)`→`(done)` + explicit 4-point "Exit criteria (met)" block + scope note (parameter-aware child selection / parameter-driven parent generation are open-ended post-phase, scope-cut, not a blocker); README/CODEBASE_ANALYSIS/`book/src/hierarchy.md` synced. **Next numbered phase: Phase 5b — Synthesizable aggregates** (`PHASE-5B-AGGREGATES`, frontier `.1`).
    - `PHASE-5B-AGGREGATES` — **tree complete** (`2026-05-18`, all leaves `done`; ROADMAP Phase 5b = `done`). `.1` design (architecture **(P)** emitter-only packed-`struct` projection; (A)/(B)/(C) rejected; identity not hashed). `.2.1` scaffold: additive `Module.aggregate_layout` + `AggregateKind{StructPacked}`/`AggregateGroup`, `Config::aggregate_prob` serde-default 0.0, non-rolling `src/ir/aggregate.rs::annotate_aggregate`, seeded `gen/mod.rs` roll scoped to **non-instantiated** modules, boundary-alias emitter (flat IR byte-identical). `.2.2` organic existence ~85% → **no rules-first pivot**; identity-invariance proven (projected twin dedup-collapses). `.2.3` `num_packed_aggregate_modules` + `phase5b_packed_aggregate` scenario (dedup-anchor shape) + `saw_packed_aggregate_design` fact/gap (bin 213→216/852→864) + non-vacuity test. **`.2.4` closed Phase 5b**: real repo-owned `Phase4Hierarchy` gate verified CLEAN on `/tmp/anvil-tool-matrix-phase5b-p1` (216 scenarios / 864 designs, `coverage_gaps=[]`, Verilator+both-Yosys 864/0, `saw_packed_aggregate_design=true`, P4/P5 regressions clean); promotion strictly followed the verified artifact (r87) — ROADMAP Phase 5b `(not started)`→`(done)` + 3-point exit criteria + scope note; `book/src/ir.md`+`knobs.md`/README/CODEBASE_ANALYSIS synced. Scaffold scope (recorded, not blockers): StructPacked-only / non-instantiated / `param_env`-skipped. **Next numbered phase: Phase 6 — Advanced motifs** (`PHASE-6-ADVANCED-MOTIFS`, frontier `.1`).
    - `PHASE-6-ADVANCED-MOTIFS` — **active**, frontier `.2`. `.1` memory design **done** (`2026-05-18`, design-only): `DEVELOPMENT_NOTES.md` "Phase 6 inferrable-memory motif design" — empirical Yosys probe (single-port + simple-dual-port → `1 $mem_v2`, clean both repo modes + Verilator); architecture **(M)** first-class `Memory` block (additive `Vec<Memory>` on `Module`, Default-empty) + opaque `Node::MemRead` leaf (sibling to `FlopQ`, never CSE'd) + emitter renders the validated inferrable template on shared `clk` + opt-in `Config::memory_prob` serde-default 0.0; rejected (A) flop-array+mux (not `$mem`-inferred) / (B) emitter-only string template / (C) generic unpacked-array datatype. `.2` split (`2026-05-18`) per Splitting Rules + r87 into `.2.1` scaffold / `.2.2` Yosys-inference proof + `MemRead` CSE-opacity / `.2.3` matrix scenario+metric+gap no-advance / `.2.4` real-gate verify → ROADMAP **memory delivered** note (Phase 6 stays open for `.3` FSM — no tree closure). `.2.1` further split → `.2.1a`/`.2.1b` on a discovered compaction-reachability dependency (opaque stateful leaf ≠ mechanical FlopQ-mirroring). **`.2.1` container complete (`2026-05-18`)**: `.2.1a` IR core (`MemId`/`MemKind`/`Memory`/`Node::MemRead`/`DepAtom::MemVirtual`/`has_local_memories`) + `MemRead` through ~21 matches + load-bearing `compact.rs` reachability + emitter inferrable template + validator step-5b + 3 unit proofs; `.2.1b` `Config::memory_prob` (serde-default 0.0) + rules-first `build_memory_leaf` + single opt-in roll (after the param lane, mutually exclusive) + default-off/forced-on focused proof; real spot-check of generated memory → `1 $mem_v2`, verilator + both-yosys clean. **`.2.2` done** (`2026-05-18`, proof only): `inferrable_memory_matches_yosys_template_and_is_factorization_opaque` — 64 combos (4 strategies × 4 FactorizationLevel incl. EGraph × 4 seeds) prove the generated SV is exactly the `.1` Yosys-`$mem_v2` template + `MemRead`/array never enter the NodeId graph (CSE/factorization-opaque); tool-level `$mem_v2`/Verilator proof scoped to `.2.1b` spot-check (interim) + `.2.4` real gate (cargo can't shell yosys — Decisions). **`.2.3` done** (`2026-05-18`): `DesignMetrics.num_memory_modules` + `phase6_inferrable_memory_focus_config`/scenario (clone of phase5b/dedup anchor → shape sets unperturbed; library leaves = memory blocks) + `saw_inferrable_memory_design` set/merge + Phase4Hierarchy gap; bin 216→219 / 864→876; `phase6_inferrable_memory_scenario_is_non_vacuous` proves ≥1 memory module/strategy (coverage fact reachable); phase4 bin 3/3; ROADMAP unchanged. **Frontier `.2.4`** = run the real repo-owned `Phase4Hierarchy` gate, verify `coverage_gaps=[]` + Verilator/both-Yosys all-pass + `saw_inferrable_memory_design=true` (+ P4/P5/P5b regressions clean), THEN record memory **delivered** in ROADMAP Phase 6 + reconcile `book/src/ir.md`/`knobs.md` (Phase 6 stays open for `.3` FSM — no tree closure; r87 no-aspirational-claims). `.3` (generated-encoding FSM motif) still pending.
    - `PHASE-7-ORACLE-MICRODESIGN` — frontier `.1` (expected-facts design, unblocked).
    - `PHASE-8-FRONTEND-ACCEPT` — frontier `.1` (source-level IR design, unblocked).
    - `PHASE-9-MULTI-ARTIFACT-UMBRELLA` — frontier `.1` (selector/plumbing design, unblocked); `.2` blocked until ≥2 lanes exist.
  - `INSTA-SNAPSHOTS` — current frontier `INSTA-SNAPSHOTS.1` (baseline insta wire-up). Quality lane: reproducibility regressions.
  - `DIFFERENTIAL-SIMULATION` — current frontier `DIFFERENTIAL-SIMULATION.1` (scope iverilog or alternative as the second simulator; Verilator is already a fait accompli via the matrix gate). Quality lane: signoff-level downstream consistency.
  - `COVERAGE-INSTRUMENTATION` — current frontier `COVERAGE-INSTRUMENTATION.2` (triage of top-5 under-covered files). Quality lane: test-discipline visibility. Baseline landed at `docs/coverage-baseline.md` (85.26% lines / 91.95% functions / 87.61% regions; planner core 88-99% covered by focused proofs alone).
  See [docs/TASK_TREE.md](docs/TASK_TREE.md) for the workflow and the full active-tree index. The whole roadmap is now trackable via task trees; `rN` is **not** retired — it remains the within-leaf slice cadence (each phase tree owns the decomposition; linear coverage slices inside a leaf still land under `rN` + `CHANGES.md`, exactly as the closed `HIERARCHY-AWARE-IDENTITY` leaves landed as r85/r86/r87).
- Current README execution found and fixed a source-tree command
  contract break: with both `anvil` and `tool_matrix` binaries present,
  plain `cargo run -- ...` was ambiguous until `Cargo.toml` restored
  `default-run = "anvil"`. README, USER_GUIDE, mdBook getting-started,
  architecture, and developer notes now describe the split:
  `cargo run -- ...` for the generator, `cargo run --bin tool_matrix -- ...`
  for the harness.
- Current mdBook audit also fixed a stale structural-rules sentence
  about M-to-1 muxes. General M-to-1 muxes are live both as
  combinational blocks (Rule 15) and as flop-D mux motifs; the Rule 14
  operator-arity note no longer implies they exist only inside flop
  D-inputs.
- Latest full downstream-clean Phase 4 hierarchy bank is:
  `/tmp/anvil-tool-matrix-phase4-hierarchy-r87/tool_matrix_report.json`
  covers the live `210`-scenario policy at `4` designs/scenario
  (`840` total designs), with `artifact_kind = "design"`,
  `coverage_gaps = []`, and `840/0` pass-fail in Verilator plus both
  repo-owned Yosys modes. It includes the direct sibling helper route,
  direct registered sibling helper route, multi-stage registered sibling
  route, stateful parent-output helper route, multi-stage direct
  registered sibling helper route, multi-stage registered
  parent-composed helper route, stateful parent-composed helper
  child-input routing, and recursive exact-depth-2 non-top
  parent-composed helper child-input routing through parent-local flops,
  plus recursive exact-depth-2 non-top direct sibling helper routing and
  recursive exact-depth-2 non-top direct registered sibling helper
  routing, plus recursive exact-depth-2 non-top registered
  parent-composed helper D-cone routing, plus recursive exact-depth-2
  non-top multi-stage direct registered sibling helper routing, plus
  recursive exact-depth-2 non-top multi-stage registered
  parent-composed helper routing, plus recursive exact-depth-2 non-top
  parent-output helper routing, plus recursive exact-depth-2 non-top
  stateful parent-output helper routing, plus recursive exact-depth-2
  non-top multi-helper budget evidence for parent-output composition,
  plus recursive exact-depth-2 non-top multi-helper budget evidence for
  parent-composed child-input bindings,
  plus recursive exact-depth-2 non-top stateful multi-helper budget
  evidence for parent outputs through parent-local flops, plus
  recursive exact-depth-2 non-top registered mixed-support child-input
  routing without helper instances, plus recursive exact-depth-2
  non-top multi-stage registered parent-composed child-input routing
  through earlier parent-local Qs without helper instances, plus
  recursive exact-depth-2 non-top multi-stage registered sibling-routed
  child-input routing through earlier parent-local Qs without helper
  instances, plus recursive exact-depth-2 non-top multi-stage
  registered mixed-support child-input routing that combines parent
  ports, child outputs, and earlier parent-local Qs without helper
  instances, plus recursive exact-depth-2 non-top registered
  parent-composed helper child-input routing that also mixes parent-port
  support into the helper-sourced D cone below the top parent, plus
  recursive exact-depth-2 non-top parent-output helper routing that also
  mixes parent-port support in the same helper-backed output cone, plus
  stateful helper-backed parent outputs that also mix parent-port
  support, unregistered parent-composed helper child-input bindings that
  mix helper and parent-port support, and stateful helper-through-flop
  unregistered child-input bindings that also mix parent-port support,
  plus direct registered sibling child-input D paths that mix sibling or
  helper instance-output support with parent data-port support without
  becoming registered parent-composed logic, plus recursive exact-depth-2
  non-top direct registered sibling mixed-support child-input D paths
  below the top parent, plus recursive exact-depth-2 non-top
  unregistered parent-composed mixed-support child-input routing without
  helper instances, plus recursive exact-depth-2 non-top stateful
  unregistered parent-composed mixed-support child-input routing through
  parent-local Qs without helper instances.
  Focused regressions:
  `cargo test recursive_hierarchy_sibling_routes_can_use_helper_instances_below_top`,
  `cargo test recursive_hierarchy_registered_sibling_routes_can_use_helper_instances_below_top`,
  `cargo test recursive_hierarchy_registered_sibling_routes_can_chain_helper_instances_below_top`,
  `cargo test recursive_hierarchy_registered_parent_composed_routes_can_chain_helper_instances_below_top`,
  `cargo test recursive_hierarchy_registered_child_input_cones_can_use_helper_instances_below_top`,
  `cargo test recursive_hierarchy_parent_outputs_can_depend_on_helper_instances_below_top`,
  `cargo test recursive_hierarchy_parent_outputs_can_spend_helper_budget_below_top`,
  `cargo test recursive_hierarchy_parent_cone_helper_budget_allows_multiple_helpers_below_top`,
  `cargo test recursive_hierarchy_parent_outputs_can_spend_stateful_helper_budget_below_top`,
  `cargo test recursive_hierarchy_parent_outputs_can_route_helper_instances_through_parent_flops_below_top`,
  `cargo test recursive_hierarchy_registered_mixed_support_routes_below_top`,
  `cargo test recursive_hierarchy_registered_parent_composed_routes_can_chain_without_helpers_below_top`,
  `cargo test recursive_hierarchy_registered_sibling_routes_can_chain_without_helpers_below_top`,
  `cargo test recursive_hierarchy_registered_multistage_mixed_support_routes_below_top`,
  `cargo test recursive_hierarchy_registered_helper_routes_mix_parent_ports_below_top`,
  `cargo test recursive_hierarchy_parent_outputs_mix_helper_instances_with_parent_ports_below_top`,
  `cargo test hierarchy_registered_sibling_routes_can_mix_parent_port_support`,
  `cargo test recursive_hierarchy_registered_sibling_routes_can_mix_parent_port_support_below_top`,
  `cargo test recursive_hierarchy_parent_composed_routes_mix_parent_ports_below_top_without_helpers`,
  `cargo test recursive_hierarchy_parent_outputs_mix_parent_ports_below_top_without_helpers`,
  `cargo test recursive_hierarchy_stateful_parent_outputs_mix_parent_ports_below_top_without_helpers`,
  `cargo test recursive_hierarchy_stateful_parent_composed_routes_mix_parent_ports_below_top_without_helpers`,
  `cargo test recursive_hierarchy_parents_can_emit_local_flops_below_top`,
  `cargo test recursive_hierarchy_parents_can_emit_local_flops_at_depth_3`,
  `cargo test recursive_hierarchy_parent_composed_routes_mix_parent_ports_at_depth_3_without_helpers`,
  `cargo test recursive_hierarchy_parent_outputs_mix_parent_ports_at_depth_3_without_helpers`,
  `cargo test recursive_hierarchy_stateful_parent_outputs_mix_parent_ports_at_depth_3_without_helpers`,
  `cargo test recursive_hierarchy_stateful_parent_composed_routes_mix_parent_ports_at_depth_3_without_helpers`,
  `cargo test recursive_hierarchy_parents_can_emit_local_flops_at_depth_4`,
  `cargo test recursive_hierarchy_parent_composed_routes_mix_parent_ports_at_depth_4_without_helpers`,
  `cargo test recursive_hierarchy_parent_outputs_mix_parent_ports_at_depth_4_without_helpers`,
  `cargo test recursive_hierarchy_stateful_parent_outputs_mix_parent_ports_at_depth_4_without_helpers`,
  `cargo test recursive_hierarchy_stateful_parent_composed_routes_mix_parent_ports_at_depth_4_without_helpers`,
  `cargo test recursive_hierarchy_parents_can_emit_local_flops_at_depth_5`,
  `cargo test recursive_hierarchy_parent_composed_routes_mix_parent_ports_at_depth_5_without_helpers`,
  `cargo test registered_sibling_mixed_support`,
  and
  `cargo test recursive_hierarchy_parent_composed_helper_routes_can_use_parent_flops_below_top`.
  The earlier coverage-only proofs at
  `/tmp/anvil-tool-matrix-phase4-recursive-direct-helper-r32/tool_matrix_report.json`
  and
  `/tmp/anvil-tool-matrix-phase4-recursive-helper-state-r31/tool_matrix_report.json`
  are now policy breadcrumbs; `r31` remains the previous full bank for
  the 66-scenario recursive helper-state policy. The failed
  `/tmp/anvil-tool-matrix-phase4-hierarchy-r32/tool_matrix_report.json`
  is useful root-cause evidence for the CaseMux/Casez exact-selector
  shift-cleanup warning fixed in `src/gen/cone.rs`.
  The older `r23` full bank, `r24` coverage-only direct-helper proof,
  `r25` direct-helper full bank, `r26` multi-stage registered sibling
  bank, `r27` stateful parent-output helper bank, and `r28`
  multi-stage direct registered sibling helper bank, and `r29`
  multi-stage registered parent-composed helper bank, `r36` recursive
  registered parent-composed helper bank, `r37` recursive multi-stage
  direct registered helper bank, and `r38` recursive multi-stage
  registered parent-composed helper bank, and `r39` recursive non-top
  parent-output helper bank, and `r40` recursive non-top stateful
  parent-output helper bank, `r41` recursive non-top parent-output
  multi-helper budget bank, and `r42` recursive non-top stateful
  multi-helper budget bank, and `r43` recursive non-top child-input
  multi-helper budget bank, and `r44` recursive non-top registered
  mixed-support bank, and `r45` recursive non-top registered
  parent-composed multistage no-helper bank, and `r46` recursive
  non-top registered sibling multistage no-helper bank, and `r47`
  recursive non-top registered multistage mixed-support no-helper bank,
  and `r48` recursive non-top registered parent-composed helper
  mixed-support bank, `r49` recursive non-top parent-output helper
  mixed-support bank, and `r50` accumulated mixed-support hierarchy
  bank are now historical breadcrumbs.
- Current Phase 4 hierarchy r87 batch closes `HIERARCHY-AWARE-IDENTITY.4`
  AND `.5`: implements `src/ir/dedup.rs`, wires the opt-in
  `Config::hierarchy_module_dedup: bool` knob, and adds the matrix
  scenario `phase4_hier1_module_dedup_active` per construction
  strategy. New `saw_recursive_hierarchy_module_dedup_active` fact
  requires the toggle is on, ≥2 surviving Modules, 0 duplicate
  pairs, and `num_distinct == num_modules`. Focused proof
  `module_dedup_pass_collapses_structurally_duplicate_modules`
  asserts (a) baseline duplicates remain when dedup off,
  (b) `num_modules` strictly decreases with dedup on,
  (c) 0 duplicate pairs after dedup, (d) `validate_design` still
  passes post-rewrite. Validation: 3 unit tests in
  `src/ir/dedup.rs`, focused regression, `cargo test --bin tool_matrix`,
  and the full r87 gate with `210` scenarios / `840` designs,
  `coverage_gaps = []`,
  `saw_recursive_hierarchy_module_dedup_active = true`, and `840/0`
  pass-fail in Verilator plus both repo-owned Yosys modes. The
  HIERARCHY-AWARE-IDENTITY tree is complete.
- Previous Phase 4 hierarchy r86 batch closed `HIERARCHY-AWARE-IDENTITY.2`:
  proves the planner can emit structurally-duplicate Module
  definitions under tight 1-in/1-out/width-1 leaf constraints. New
  `saw_design_with_structurally_duplicate_modules` fact requires
  `num_structurally_duplicate_module_pairs > 0` and
  `num_distinct_module_signatures < num_modules`. Focused proof
  `planner_can_emit_structurally_duplicate_modules` sweeps the four
  ConstructionStrategy values. Matrix scenario
  `phase4_hier1_structurally_duplicate_modules` per construction
  strategy gates the surface. Validation: focused regression,
  `cargo test --bin tool_matrix`, and the full r86 gate at
  `/tmp/anvil-tool-matrix-phase4-hierarchy-r86/tool_matrix_report.json`
  with `207` scenarios / `828` designs, `coverage_gaps = []`,
  `saw_design_with_structurally_duplicate_modules = true`, and
  `828/0` pass-fail in Verilator plus both repo-owned Yosys modes.
- Previous Phase 4 hierarchy r85 batch lands the first slice of
  hierarchy-aware identity: a deterministic canonical signature per
  Module recorded as `DesignMetrics.canonical_module_signatures`. The
  new `saw_recursive_hierarchy_canonical_module_signature_diversity`
  fact requires `realized_max_leaf_depth > 1`, one nonzero signature
  per module, and `num_distinct_module_signatures >= 2`. Focused proof
  `canonical_module_signatures_are_stable_and_isomorphism_aware`
  asserts stability across re-generation. Matrix scenario
  `phase4_recur_d2_canonical_module_signatures` per construction
  strategy gates the surface. Validation: focused regression,
  `cargo test --bin tool_matrix`, and the full r85 gate at
  `/tmp/anvil-tool-matrix-phase4-hierarchy-r86/tool_matrix_report.json`
  with `204` scenarios / `816` designs, `coverage_gaps = []`,
  `saw_recursive_hierarchy_canonical_module_signature_diversity = true`,
  and `816/0` pass-fail in Verilator plus both repo-owned Yosys modes.
  PNT-3 of the autonomous-PNT chain (final slice).
- Previous Phase 4 hierarchy r84 batch extended the helper-budget axis
  above r83's chain-depth axis. The new
  `saw_recursive_parent_cone_helper_budget_5` fact requires
  `realized_max_leaf_depth > 1`,
  `max_parent_cone_instances_per_module >= 5`, planner-realized
  `max_parent_cone_instances_per_internal_module >= 5`, and non-top
  helper count strictly exceeding the top-only baseline. Focused proof
  `recursive_hierarchy_parent_cone_helper_budget_5_below_top` isolates
  the surface at exact hierarchy depth 2 with 4,4 child instances.
  Matrix scenario `phase4_recur_d2_parent_cone_instance_budget5` per
  construction strategy gates the surface in the full bank. Validation:
  focused regression, `cargo test --bin tool_matrix`, and the full r84
  gate at
  `/tmp/anvil-tool-matrix-phase4-hierarchy-r84/tool_matrix_report.json`
  with `201` scenarios / `804` designs, `coverage_gaps = []`,
  `saw_recursive_parent_cone_helper_budget_5 = true`, and `804/0`
  pass-fail in Verilator plus both repo-owned Yosys modes. PNT-2 of the
  autonomous-PNT chain.
- Previous Phase 4 hierarchy r83 batch opened a chain-depth axis above
  the closed depth-3..7 sweeps. The new
  `saw_recursive_hierarchy_three_stage_registered_parent_composed_chain`
  fact requires `realized_max_leaf_depth > 1`, hierarchy-wide registered
  parent-composed child-input bindings whose D chain through three or
  more parent-local flop stages exceeding top-only, and
  `hierarchy_parent_cone_instances == 0`. Focused proof
  `recursive_hierarchy_registered_parent_composed_routes_can_chain_three_stages_below_top`
  isolates the surface at exact hierarchy depth 3 with 4,4 child
  instances, `max_flops_per_module = 128`, and `max_depth = 8`. Matrix
  scenario `phase4_recur_d3_registered_three_stage_parent_composed_chain`
  per construction strategy gates the surface in the full bank.
  Validation: focused regression, `cargo test --bin tool_matrix`, and
  the full r83 gate at
  `/tmp/anvil-tool-matrix-phase4-hierarchy-r83/tool_matrix_report.json`
  with `198` scenarios / `792` designs, `coverage_gaps = []`,
  `saw_recursive_hierarchy_three_stage_registered_parent_composed_chain = true`,
  and `792/0` pass-fail in Verilator plus both repo-owned Yosys modes.
  PNT-1 of the autonomous-PNT chain.
- Previous Phase 4 hierarchy r82 batch closed the depth-7 sweep by
  pushing the recursive stateful unregistered parent-composed
  mixed-support child-input surface (r77's depth-6, r72's depth-5,
  r67's depth-4, r62's depth-3 territory) to exact hierarchy depth 7.
  The new
  `saw_recursive_hierarchy_depth_7_stateful_parent_composed_mixed_support_child_inputs`
  fact requires `realized_max_leaf_depth >= 7`, hierarchy-wide
  stateful parent-composed mixed-support child-input bindings
  exceeding top-only, hierarchy-wide unregistered parent-composed
  child-input bindings exceeding top-only, hierarchy-wide parent-local
  flops exceeding top-only, and `hierarchy_parent_cone_instances == 0`.
  Focused proof
  `recursive_hierarchy_stateful_parent_composed_routes_mix_parent_ports_at_depth_7_without_helpers`
  isolates the surface across six intermediate parent layers. Matrix
  scenario
  `phase4_recur_d7_stateful_parent_composed_mixed_support_child_input`
  per construction strategy uses 2,2 child-instance bounds (same 2,2
  calibration as r77/r79). Validation: focused regression,
  `cargo test --bin tool_matrix`, and the full r82 gate at
  `/tmp/anvil-tool-matrix-phase4-hierarchy-r82/tool_matrix_report.json`
  with `195` scenarios / `780` designs, `coverage_gaps = []`,
  `saw_recursive_hierarchy_depth_7_stateful_parent_composed_mixed_support_child_inputs = true`,
  and `780/0` pass-fail in Verilator plus both repo-owned Yosys modes.
- Previous Phase 4 hierarchy r81 batch extended the depth-7 axis by
  pushing the recursive stateful parent-port-composed parent-output
  surface (r76's depth-6 territory and earlier) to exact hierarchy
  depth 7. New `saw_recursive_hierarchy_depth_7_stateful_parent_port_composed_outputs`
  fact requires `realized_max_leaf_depth >= 7`, hierarchy-wide
  parent-port-composed parent outputs through parent-local flops
  exceeding top-only, hierarchy-wide parent-local flops exceeding
  top-only, and `hierarchy_parent_cone_instances == 0`. Focused proof
  `recursive_hierarchy_stateful_parent_outputs_mix_parent_ports_at_depth_7_without_helpers`
  isolates the surface across six intermediate parent layers. Matrix
  scenario `phase4_recur_d7_stateful_parent_port_composed_output` per
  construction strategy uses 2,2 child-instance bounds with
  `hierarchy_parent_flop_prob = 1.0` and `max_flops_per_module = 64`.
  Validation: focused regression, `cargo test --bin tool_matrix`, and
  the full r81 gate at
  `/tmp/anvil-tool-matrix-phase4-hierarchy-r81/tool_matrix_report.json`
  with `192` scenarios / `768` designs, `coverage_gaps = []`,
  `saw_recursive_hierarchy_depth_7_stateful_parent_port_composed_outputs = true`,
  and `768/0` pass-fail in Verilator plus both repo-owned Yosys modes.
- Previous Phase 4 hierarchy r80 batch extended the depth-7 axis by
  pushing the recursive unregistered parent-port-composed parent-output
  surface (r75's depth-6, r70's depth-5, r65's depth-4, r60's depth-3
  territory) to exact hierarchy depth 7. The new
  `saw_recursive_hierarchy_depth_7_parent_port_composed_outputs` fact
  requires `realized_max_leaf_depth >= 7`, hierarchy-wide
  parent-port-composed and parent-composed parent outputs exceeding
  top-only, `hierarchy_parent_cone_instances == 0`, and
  `hierarchy_parent_local_flops == 0`. Focused proof
  `recursive_hierarchy_parent_outputs_mix_parent_ports_at_depth_7_without_helpers`
  isolates the surface across six intermediate parent layers (no
  helpers, no sibling routing, no registered routing, no
  parent-composed child-input cones, no parent-local state). The new
  matrix scenario `phase4_recur_d7_parent_port_composed_output` per
  construction strategy uses `2,2` child-instance bounds (same as
  earlier depths). Validation: focused pipeline regression,
  `cargo test --bin tool_matrix`, and the full downstream-clean r80
  gate at `/tmp/anvil-tool-matrix-phase4-hierarchy-r80/tool_matrix_report.json`
  with `189` scenarios / `756` designs, `coverage_gaps = []`,
  `saw_recursive_hierarchy_depth_7_parent_port_composed_outputs = true`,
  and `756/0` pass-fail in Verilator plus both repo-owned Yosys modes.
- Previous Phase 4 hierarchy r79 batch extended the depth-7 axis by
  pushing the recursive unregistered parent-composed mixed-support
  child-input surface (r74's depth-6 territory, r69's depth-5
  territory, r64's depth-4 territory, r59's depth-3 territory) to
  exact hierarchy depth 7. The new
  `saw_recursive_hierarchy_depth_7_mixed_support_child_inputs` fact
  requires `realized_max_leaf_depth >= 7`, hierarchy-wide unregistered
  parent-composed and mixed-support child-input bindings exceeding
  top-only, and `hierarchy_parent_cone_instances == 0`. A focused
  exact-depth-7 proof
  `recursive_hierarchy_parent_composed_routes_mix_parent_ports_at_depth_7_without_helpers`
  isolates the surface across six intermediate parent layers (no
  helpers, no sibling routing, no registered routing, no parent-local
  flops). The new matrix scenario
  `phase4_recur_d7_parent_composed_mixed_support_child_input` per
  construction strategy uses `2,2` child-instance bounds — same 2,2
  calibration as r74/r77 (mixed-support cells at depths >= 6 use 2,2).
  The slice does not change the generator. Validation: focused pipeline
  regression, `cargo test --bin tool_matrix`, and the full
  downstream-clean r79 gate at
  `/tmp/anvil-tool-matrix-phase4-hierarchy-r79/tool_matrix_report.json`
  with `186` scenarios / `744` designs, `coverage_gaps = []`,
  `saw_recursive_hierarchy_depth_7_mixed_support_child_inputs = true`,
  and `744/0` pass-fail in Verilator plus both repo-owned Yosys modes.
- Previous Phase 4 hierarchy r78 batch opened the depth-7 axis by
  pushing the recursive parent-flop surface (r73's depth-6 territory,
  r68's depth-5 territory, r63's depth-4 territory, r58's depth-3
  territory) to exact hierarchy depth 7. The new
  `saw_recursive_hierarchy_depth_7_parent_local_flops` fact requires
  `realized_max_leaf_depth >= 7`, hierarchy-wide parent-local flops
  exceeding top-only, and at least one internal parent module
  occurrence with local flops. A focused exact-depth-7 proof
  `recursive_hierarchy_parents_can_emit_local_flops_at_depth_7`
  isolates the surface across six intermediate parent layers (no
  helpers, no sibling routing, no registered routing, no
  parent-composed child-input cones). The new matrix scenario
  `phase4_recur_d7_parent_state` per construction strategy uses `2,2`
  child-instance bounds. The depth-6 sweep closed in r77; r78 now opens
  the depth-7 axis with the simplest surface as a foothold. Validation:
  focused pipeline regression, `cargo test --bin tool_matrix`, and the
  full downstream-clean r78 gate at
  `/tmp/anvil-tool-matrix-phase4-hierarchy-r78/tool_matrix_report.json`
  with `183` scenarios / `732` designs, `coverage_gaps = []`,
  `saw_recursive_hierarchy_depth_7_parent_local_flops = true`,
  and `732/0` pass-fail in Verilator plus both repo-owned Yosys modes.
- Previous Phase 4 hierarchy r77 batch closed the depth-6 sweep by
  pushing the recursive stateful unregistered parent-composed
  mixed-support child-input surface (r72's depth-5 territory, r67's
  depth-4 territory, r62's depth-3 territory) to exact hierarchy
  depth 6. The new
  `saw_recursive_hierarchy_depth_6_stateful_parent_composed_mixed_support_child_inputs`
  fact requires `realized_max_leaf_depth >= 6`, hierarchy-wide stateful
  parent-composed mixed-support child-input bindings exceeding top-only,
  hierarchy-wide unregistered parent-composed child-input bindings
  exceeding top-only, hierarchy-wide parent-local flops exceeding
  top-only, and `hierarchy_parent_cone_instances == 0`. A focused
  exact-depth-6 proof
  `recursive_hierarchy_stateful_parent_composed_routes_mix_parent_ports_at_depth_6_without_helpers`
  isolates the surface across five intermediate parent layers (no
  helpers, no sibling routing, no registered routing, with
  parent-composed child-input cones and parent-local flops). The new
  matrix scenario
  `phase4_recur_d6_stateful_parent_composed_mixed_support_child_input`
  per construction strategy uses `2,2` child-instance bounds with
  `hierarchy_child_input_cone_prob = 1.0`,
  `hierarchy_parent_flop_prob = 1.0`, and `max_flops_per_module = 64`
  — same 2,2 calibration as r74 (mixed-support cells at depth 6 use
  2,2 instead of 4,4). The depth-6 sweep is now structurally complete:
  all five mixed-support cells gated at exact depth 6. Validation:
  focused pipeline regression, `cargo test --bin tool_matrix`, and the
  full downstream-clean r77 gate at
  `/tmp/anvil-tool-matrix-phase4-hierarchy-r77/tool_matrix_report.json`
  with `180` scenarios / `720` designs, `coverage_gaps = []`,
  `saw_recursive_hierarchy_depth_6_stateful_parent_composed_mixed_support_child_inputs = true`,
  and `720/0` pass-fail in Verilator plus both repo-owned Yosys modes.
- Previous Phase 4 hierarchy r76 batch extended the depth-6 axis by
  pushing the recursive stateful parent-port-composed parent-output
  surface (r71's depth-5 territory, r66's depth-4 territory, r61's
  depth-3 territory) to exact hierarchy depth 6. The new
  `saw_recursive_hierarchy_depth_6_stateful_parent_port_composed_outputs`
  fact requires `realized_max_leaf_depth >= 6`, hierarchy-wide
  parent-port-composed parent outputs exceeding top-only,
  hierarchy-wide parent-port-composed outputs through parent-local
  flops exceeding top-only, hierarchy-wide parent-local flops exceeding
  top-only, and `hierarchy_parent_cone_instances == 0`. A focused
  exact-depth-6 proof
  `recursive_hierarchy_stateful_parent_outputs_mix_parent_ports_at_depth_6_without_helpers`
  isolates the surface across five intermediate parent layers (no
  helpers, no sibling routing, no registered routing, no
  parent-composed child-input cones, with parent-local flops at prob
  1.0 and 64 flops/module). The new matrix scenario
  `phase4_recur_d6_stateful_parent_port_composed_output` per
  construction strategy uses `2,2` child-instance bounds. The slice
  does not change the generator. Validation: focused pipeline
  regression, `cargo test --bin tool_matrix`, and the full
  downstream-clean r76 gate at
  `/tmp/anvil-tool-matrix-phase4-hierarchy-r76/tool_matrix_report.json`
  with `177` scenarios / `708` designs, `coverage_gaps = []`,
  `saw_recursive_hierarchy_depth_6_stateful_parent_port_composed_outputs = true`,
  and `708/0` pass-fail in Verilator plus both repo-owned Yosys modes.
- Previous Phase 4 hierarchy r75 batch extended the depth-6 axis by
  pushing the recursive unregistered parent-port-composed parent-output
  surface (r70's depth-5 territory, r65's depth-4 territory, r60's
  depth-3 territory) to exact hierarchy depth 6. The new
  `saw_recursive_hierarchy_depth_6_parent_port_composed_outputs` fact
  requires `realized_max_leaf_depth >= 6`, hierarchy-wide
  parent-port-composed and parent-composed parent outputs exceeding
  top-only, `hierarchy_parent_cone_instances == 0`, and
  `hierarchy_parent_local_flops == 0`. A focused exact-depth-6 proof
  `recursive_hierarchy_parent_outputs_mix_parent_ports_at_depth_6_without_helpers`
  isolates the surface across five intermediate parent layers (no
  helpers, no sibling routing, no registered routing, no
  parent-composed child-input cones, no parent-local state). The new
  matrix scenario `phase4_recur_d6_parent_port_composed_output` per
  construction strategy uses `2,2` child-instance bounds (consistent
  with depths 3-5 for parent-port-composed cells). The slice does not
  change the generator. Validation: focused pipeline regression,
  `cargo test --bin tool_matrix`, and the full downstream-clean r75
  gate at `/tmp/anvil-tool-matrix-phase4-hierarchy-r75/tool_matrix_report.json`
  with `174` scenarios / `696` designs, `coverage_gaps = []`,
  `saw_recursive_hierarchy_depth_6_parent_port_composed_outputs = true`,
  and `696/0` pass-fail in Verilator plus both repo-owned Yosys modes.
- Previous Phase 4 hierarchy r74 batch extended the depth-6 axis by
  pushing the recursive unregistered parent-composed mixed-support
  child-input surface (r69's depth-5 territory, r64's depth-4
  territory, r59's depth-3 territory) to exact hierarchy depth 6. The
  new `saw_recursive_hierarchy_depth_6_mixed_support_child_inputs` fact
  requires `realized_max_leaf_depth >= 6`, hierarchy-wide unregistered
  parent-composed and mixed-support child-input bindings exceeding
  top-only, and `hierarchy_parent_cone_instances == 0`. A focused
  exact-depth-6 proof
  `recursive_hierarchy_parent_composed_routes_mix_parent_ports_at_depth_6_without_helpers`
  isolates the surface across five intermediate parent layers (no
  helpers, no sibling routing, no registered routing, no parent-local
  flops). The new matrix scenario
  `phase4_recur_d6_parent_composed_mixed_support_child_input` per
  construction strategy uses `2,2` child-instance bounds — a calibration
  choice (depths 3-5 used 4,4 for mixed-support cells; at depth 6 the
  4,4 case grew the design to 1365 internal module occurrences and
  pushed the downstream-clean gate beyond a safe slice). The slice
  does not change the generator — it tightens the gate around an
  already-supported capability. Validation includes the focused
  pipeline regression, `cargo test --bin tool_matrix`, and the full
  downstream-clean r74 gate at
  `/tmp/anvil-tool-matrix-phase4-hierarchy-r74/tool_matrix_report.json`
  with `171` scenarios / `684` designs, `coverage_gaps = []`,
  `saw_recursive_hierarchy_depth_6_mixed_support_child_inputs = true`,
  and `684/0` pass-fail in Verilator plus both repo-owned Yosys modes.
- Previous Phase 4 hierarchy r73 batch opened the depth-6 axis by
  pushing the recursive parent-flop surface (r68's depth-5 territory,
  r63's depth-4 territory, r58's depth-3 territory) to exact hierarchy
  depth 6. The new `saw_recursive_hierarchy_depth_6_parent_local_flops`
  fact requires `realized_max_leaf_depth >= 6`, hierarchy-wide
  parent-local flops exceeding top-only, and at least one internal
  parent module occurrence with local flops. A focused exact-depth-6
  proof `recursive_hierarchy_parents_can_emit_local_flops_at_depth_6`
  isolates the surface across five intermediate parent layers (no
  helpers, no sibling routing, no registered routing, no
  parent-composed child-input cones). The new matrix scenario
  `phase4_recur_d6_parent_state` per construction strategy uses `2,2`
  child-instance bounds. The depth-5 sweep closed in r72; r73 now opens
  the depth-6 axis with the simplest surface as a foothold. The slice
  does not change the generator — it tightens the gate around an
  already-supported capability. Validation includes the focused pipeline
  regression, `cargo test --bin tool_matrix`, and the full
  downstream-clean r73 gate at
  `/tmp/anvil-tool-matrix-phase4-hierarchy-r73/tool_matrix_report.json`
  with `168` scenarios / `672` designs, `coverage_gaps = []`,
  `saw_recursive_hierarchy_depth_6_parent_local_flops = true`,
  and `672/0` pass-fail in Verilator plus both repo-owned Yosys modes.
- Previous Phase 4 hierarchy r72 batch closed the depth-5 sweep by
  pushing the recursive stateful unregistered parent-composed
  mixed-support child-input surface (r67's depth-4 territory, r62's
  depth-3 territory) to exact hierarchy depth 5. The new
  `saw_recursive_hierarchy_depth_5_stateful_parent_composed_mixed_support_child_inputs`
  fact requires `realized_max_leaf_depth >= 5`, hierarchy-wide stateful
  parent-composed mixed-support child-input bindings exceeding top-only,
  hierarchy-wide unregistered parent-composed child-input bindings
  exceeding top-only, hierarchy-wide parent-local flops exceeding
  top-only, and `hierarchy_parent_cone_instances == 0`. A focused
  exact-depth-5 proof
  `recursive_hierarchy_stateful_parent_composed_routes_mix_parent_ports_at_depth_5_without_helpers`
  isolates the surface across four intermediate parent layers (no
  helpers, no sibling routing, no registered routing, with
  parent-composed child-input cones and parent-local flops). The new
  matrix scenario
  `phase4_recur_d5_stateful_parent_composed_mixed_support_child_input`
  per construction strategy uses `4,4` child-instance bounds with
  `hierarchy_child_input_cone_prob = 1.0`,
  `hierarchy_parent_flop_prob = 1.0`, and `max_flops_per_module = 64`.
  The depth-5 sweep is now structurally complete: all five mixed-support
  cells (parent-flops, unregistered parent-composed mixed-support child
  inputs, unregistered parent-port-composed parent outputs, stateful
  parent-port-composed parent outputs, stateful unregistered
  parent-composed mixed-support child inputs) are gated as first-class
  facts at exact hierarchy depth 5. The slice does not change the
  generator — it tightens the gate around an already-supported
  capability. Validation includes the focused pipeline regression,
  `cargo test --bin tool_matrix`, and the full downstream-clean r72 gate
  at `/tmp/anvil-tool-matrix-phase4-hierarchy-r72/tool_matrix_report.json`
  with `165` scenarios / `660` designs, `coverage_gaps = []`,
  `saw_recursive_hierarchy_depth_5_stateful_parent_composed_mixed_support_child_inputs = true`,
  and `660/0` pass-fail in Verilator plus both repo-owned Yosys modes.
- Previous Phase 4 hierarchy r71 batch extended the depth-5 axis by
  pushing the recursive stateful parent-port-composed parent-output
  surface (r66's depth-4 territory, r61's depth-3 territory) to exact
  hierarchy depth 5. The
  `saw_recursive_hierarchy_depth_5_stateful_parent_port_composed_outputs`
  fact requires `realized_max_leaf_depth >= 5`, hierarchy-wide
  parent-port-composed parent outputs exceeding top-only,
  hierarchy-wide parent-port-composed parent outputs through
  parent-local flops exceeding top-only, hierarchy-wide parent-local
  flops exceeding top-only, and `hierarchy_parent_cone_instances == 0`.
  A focused exact-depth-5 proof
  `recursive_hierarchy_stateful_parent_outputs_mix_parent_ports_at_depth_5_without_helpers`
  isolates the surface across four intermediate parent layers (no
  helpers, no sibling routing, no registered routing, no
  parent-composed child-input cones, with parent-local flops). The
  matrix scenario `phase4_recur_d5_stateful_parent_port_composed_output`
  per construction strategy uses `2,2` child-instance bounds with
  `hierarchy_parent_flop_prob = 1.0` and `max_flops_per_module = 64`.
  The slice does not change the generator — it tightens the gate around
  an already-supported capability. Validation includes the focused
  pipeline regression, `cargo test --bin tool_matrix`, and the full
  downstream-clean r71 gate at
  `/tmp/anvil-tool-matrix-phase4-hierarchy-r71/tool_matrix_report.json`
  with `162` scenarios / `648` designs, `coverage_gaps = []`,
  `saw_recursive_hierarchy_depth_5_stateful_parent_port_composed_outputs = true`,
  and `648/0` pass-fail in Verilator plus both repo-owned Yosys modes.
- Previous Phase 4 hierarchy r70 batch extended the depth-5 axis by
  pushing the recursive unregistered parent-port-composed parent-output
  surface (r65's depth-4 territory, r60's depth-3 territory) to exact
  hierarchy depth 5. The
  `saw_recursive_hierarchy_depth_5_parent_port_composed_outputs` fact
  requires `realized_max_leaf_depth >= 5`, hierarchy-wide
  parent-port-composed and parent-composed parent outputs exceeding
  top-only, `hierarchy_parent_cone_instances == 0`, and
  `hierarchy_parent_local_flops == 0`. A focused exact-depth-5 proof
  `recursive_hierarchy_parent_outputs_mix_parent_ports_at_depth_5_without_helpers`
  isolates the surface across four intermediate parent layers (no
  helpers, no sibling routing, no registered routing, no
  parent-composed child-input cones, no parent-local state). The matrix
  scenario `phase4_recur_d5_parent_port_composed_output` per
  construction strategy uses `2,2` child-instance bounds. The slice
  does not change the generator — it tightens the gate around an
  already-supported capability. Validation includes the focused
  pipeline regression, `cargo test --bin tool_matrix`, and the full
  downstream-clean r70 gate at
  `/tmp/anvil-tool-matrix-phase4-hierarchy-r70/tool_matrix_report.json`
  with `159` scenarios / `636` designs, `coverage_gaps = []`,
  `saw_recursive_hierarchy_depth_5_parent_port_composed_outputs = true`,
  and `636/0` pass-fail in Verilator plus both repo-owned Yosys modes.
- Previous Phase 4 hierarchy r69 batch extended the depth-5 axis by
  pushing the recursive unregistered parent-composed mixed-support
  child-input surface (r64's depth-4 territory, r59's depth-3
  territory) to exact hierarchy depth 5. The
  `saw_recursive_hierarchy_depth_5_mixed_support_child_inputs` fact
  requires `realized_max_leaf_depth >= 5`, hierarchy-wide unregistered
  parent-composed and mixed-support child-input bindings exceeding
  top-only, and `hierarchy_parent_cone_instances == 0`. A focused
  exact-depth-5 proof
  `recursive_hierarchy_parent_composed_routes_mix_parent_ports_at_depth_5_without_helpers`
  isolates the surface across four intermediate parent layers (no
  helpers, no sibling routing, no registered routing, no parent-local
  flops). The matrix scenario
  `phase4_recur_d5_parent_composed_mixed_support_child_input` per
  construction strategy uses `4,4` child-instance bounds. The slice
  does not change the generator — it tightens the gate around an
  already-supported capability. Validation includes the focused
  pipeline regression, `cargo test --bin tool_matrix`, and the full
  downstream-clean r69 gate at
  `/tmp/anvil-tool-matrix-phase4-hierarchy-r69/tool_matrix_report.json`
  with `156` scenarios / `624` designs, `coverage_gaps = []`,
  `saw_recursive_hierarchy_depth_5_mixed_support_child_inputs = true`,
  and `624/0` pass-fail in Verilator plus both repo-owned Yosys modes.
- Previous Phase 4 hierarchy r68 batch opened the depth-5 axis by
  pushing the recursive parent-flop surface (r63's depth-4 territory,
  r58's depth-3 territory, r57's depth-2 territory) to exact hierarchy
  depth 5. The new
  `saw_recursive_hierarchy_depth_5_parent_local_flops` fact requires
  `realized_max_leaf_depth >= 5`, hierarchy-wide parent-local flops
  exceeding top-only, and at least one internal parent module
  occurrence with local flops. A focused exact-depth-5 proof
  `recursive_hierarchy_parents_can_emit_local_flops_at_depth_5`
  isolates the surface across four intermediate parent layers (no
  helpers, no sibling routing, no registered routing, no
  parent-composed child-input cones). The new matrix scenario
  `phase4_recur_d5_parent_state` per construction strategy uses `2,2`
  child-instance bounds. The depth-4 sweep had completed all five
  mixed-support cells; r68 now opens the depth-5 axis with the
  simplest surface as a foothold. The slice does not change the
  generator — it tightens the gate around an already-supported
  capability. Validation includes the focused pipeline regression,
  `cargo test --bin tool_matrix`, and the full downstream-clean r68
  gate at
  `/tmp/anvil-tool-matrix-phase4-hierarchy-r68/tool_matrix_report.json`
  with `153` scenarios / `612` designs, `coverage_gaps = []`,
  `saw_recursive_hierarchy_depth_5_parent_local_flops = true`, and
  `612/0` pass-fail in Verilator plus both repo-owned Yosys modes.
- Previous Phase 4 hierarchy r67 batch closed the depth-4 sweep by
  pushing the recursive stateful unregistered parent-composed
  mixed-support child-input surface (r62's depth-3 territory) to exact
  hierarchy depth 4 without helpers. The new
  `saw_recursive_hierarchy_depth_4_stateful_parent_composed_mixed_support_child_inputs`
  fact requires `realized_max_leaf_depth >= 4`, hierarchy-wide
  stateful-parent-composed-mixed-support and unregistered
  parent-composed counters exceeding top-only, parent-local flops below
  the top, and `hierarchy_parent_cone_instances == 0`. A focused
  exact-depth-4 proof
  `recursive_hierarchy_stateful_parent_composed_routes_mix_parent_ports_at_depth_4_without_helpers`
  isolates the surface across three intermediate parent layers (no
  helpers, no sibling routing, no registered routing,
  parent-composed child-input cones on, parent-local flops on). The new
  matrix scenario `phase4_recur_d4_stateful_parent_composed_mixed_support_child_input`
  per construction strategy uses `4,4` child-instance bounds. The slice
  does not change the generator — it tightens the gate around an
  already-supported capability. Validation includes the focused
  pipeline regression, `cargo test --bin tool_matrix`, and the full
  downstream-clean r67 gate at
  `/tmp/anvil-tool-matrix-phase4-hierarchy-r67/tool_matrix_report.json`
  with `150` scenarios / `600` designs, `coverage_gaps = []`,
  `saw_recursive_hierarchy_depth_4_stateful_parent_composed_mixed_support_child_inputs = true`,
  and `600/0` pass-fail in Verilator plus both repo-owned Yosys modes.
  The depth-4 sweep is now structurally complete across all five cells:
  parent-flops (r63), no-state mixed-support child inputs (r64),
  no-state parent-port-composed outputs (r65), stateful
  parent-port-composed outputs (r66), and stateful child-input
  mixed-support (r67).
- Previous Phase 4 hierarchy r66 batch extended the depth-4 axis by
  pushing the recursive stateful parent-port-composed parent-output
  surface (r61's depth-3 territory) to exact hierarchy depth 4 without
  helpers. The new
  `saw_recursive_hierarchy_depth_4_stateful_parent_port_composed_outputs`
  fact requires `realized_max_leaf_depth >= 4`, hierarchy-wide
  parent-port-composed outputs and through-parent-flop variants
  exceeding top-only, parent-local flops below the top, and
  `hierarchy_parent_cone_instances == 0`. A focused exact-depth-4 proof
  `recursive_hierarchy_stateful_parent_outputs_mix_parent_ports_at_depth_4_without_helpers`
  isolates the stateful parent-output cone surface across three
  intermediate parent layers (no helpers/sibling/registered/parent-composed
  child-input cones, parent-local flops on). The new matrix scenario
  `phase4_recur_d4_stateful_parent_port_composed_output` per
  construction strategy uses `2,2` child-instance bounds. The slice
  does not change the generator — it tightens the gate around an
  already-supported capability. Validation includes the focused
  pipeline regression, `cargo test --bin tool_matrix`, and the full
  downstream-clean r66 gate at
  `/tmp/anvil-tool-matrix-phase4-hierarchy-r66/tool_matrix_report.json`
  with `147` scenarios / `588` designs, `coverage_gaps = []`,
  `saw_recursive_hierarchy_depth_4_stateful_parent_port_composed_outputs = true`,
  and `588/0` pass-fail in Verilator plus both repo-owned Yosys modes.
- Previous Phase 4 hierarchy r65 batch extended the depth-4 axis by
  pushing the recursive parent-port-composed parent-output surface
  (r60's depth-3 territory) to exact hierarchy depth 4 without helpers
  or state. The new
  `saw_recursive_hierarchy_depth_4_parent_port_composed_outputs` fact
  requires `realized_max_leaf_depth >= 4`, hierarchy-wide
  parent-composed and parent-port-composed output counters exceeding
  top-only, `hierarchy_parent_cone_instances == 0`, and
  `hierarchy_parent_local_flops == 0`. A focused exact-depth-4 proof
  `recursive_hierarchy_parent_outputs_mix_parent_ports_at_depth_4_without_helpers`
  isolates the parent-output cone surface across three intermediate
  parent layers (no helpers/sibling/registered/parent-composed
  child-input cones, no parent-local flops). The new matrix scenario
  `phase4_recur_d4_parent_port_composed_output` per construction
  strategy uses `2,2` child-instance bounds. The slice does not change
  the generator — it tightens the gate around an already-supported
  capability. Validation includes the focused pipeline regression,
  `cargo test --bin tool_matrix`, and the full downstream-clean r65
  gate at `/tmp/anvil-tool-matrix-phase4-hierarchy-r65/tool_matrix_report.json`
  with `144` scenarios / `576` designs, `coverage_gaps = []`,
  `saw_recursive_hierarchy_depth_4_parent_port_composed_outputs = true`,
  and `576/0` pass-fail in Verilator plus both repo-owned Yosys modes.
- Previous Phase 4 hierarchy r64 batch extended the depth-4 axis by
  pushing the recursive unregistered parent-composed mixed-support
  child-input surface (r59's depth-3 territory) to exact hierarchy
  depth 4. The new
  `saw_recursive_hierarchy_depth_4_mixed_support_child_inputs` fact
  requires `realized_max_leaf_depth >= 4`, hierarchy-wide
  unregistered parent-composed and mixed-support child-input bindings
  exceeding top-only, and `hierarchy_parent_cone_instances == 0`. A
  focused exact-depth-4 proof
  `recursive_hierarchy_parent_composed_routes_mix_parent_ports_at_depth_4_without_helpers`
  isolates the surface across three intermediate parent layers (no
  helpers, no sibling routing, no registered routing, no parent-local
  flops). The new matrix scenario
  `phase4_recur_d4_parent_composed_mixed_support_child_input` per
  construction strategy uses `4,4` child-instance bounds. The slice
  does not change the generator — it tightens the gate around an
  already-supported capability. Validation includes the focused
  pipeline regression, `cargo test --bin tool_matrix`, and the full
  downstream-clean r64 gate at
  `/tmp/anvil-tool-matrix-phase4-hierarchy-r64/tool_matrix_report.json`
  with `141` scenarios / `564` designs, `coverage_gaps = []`,
  `saw_recursive_hierarchy_depth_4_mixed_support_child_inputs = true`,
  and `564/0` pass-fail in Verilator plus both repo-owned Yosys modes.
- Previous Phase 4 hierarchy r63 batch opened the depth-4 axis by
  pushing the recursive parent-flop surface (r58's depth-3 territory,
  r57's depth-2 territory) to exact hierarchy depth 4. The new
  `saw_recursive_hierarchy_depth_4_parent_local_flops` fact requires
  `realized_max_leaf_depth >= 4`, hierarchy-wide parent-local flops
  exceeding top-only, and at least one internal parent module
  occurrence with local flops. A focused exact-depth-4 proof
  `recursive_hierarchy_parents_can_emit_local_flops_at_depth_4`
  isolates the surface across three intermediate parent layers (no
  helpers, no sibling routing, no registered routing, no
  parent-composed child-input cones). The new matrix scenario
  `phase4_recur_d4_parent_state` per construction strategy uses `2,2`
  child-instance bounds. The depth-3 push had completed all four
  mixed-support cells (parent-flops, no-state child-input mixed-support,
  no-state parent-output mixed-support, stateful parent-output
  mixed-support, stateful child-input mixed-support); r63 now opens the
  depth-4 axis with the simplest surface as a foothold. The slice does
  not change the generator — it tightens the gate around an
  already-supported capability. Validation includes the focused
  pipeline regression, `cargo test --bin tool_matrix`, and the full
  downstream-clean r63 gate at
  `/tmp/anvil-tool-matrix-phase4-hierarchy-r63/tool_matrix_report.json`
  with `138` scenarios / `552` designs, `coverage_gaps = []`,
  `saw_recursive_hierarchy_depth_4_parent_local_flops = true`, and
  `552/0` pass-fail in Verilator plus both repo-owned Yosys modes.
- Previous Phase 4 hierarchy r62 batch closed the depth-3 push by
  pushing the recursive stateful unregistered parent-composed
  mixed-support child-input surface (r56's depth-2 territory) to exact
  hierarchy depth 3 without helpers. The new
  `saw_recursive_hierarchy_depth_3_stateful_parent_composed_mixed_support_child_inputs`
  fact requires `realized_max_leaf_depth >= 3`, hierarchy-wide
  stateful-parent-composed-mixed-support and unregistered
  parent-composed child-input counters exceeding top-only,
  parent-local flops below the top, and
  `hierarchy_parent_cone_instances == 0`. A focused exact-depth-3 proof
  `recursive_hierarchy_stateful_parent_composed_routes_mix_parent_ports_at_depth_3_without_helpers`
  isolates the surface across two intermediate parent layers (no
  helpers, no sibling routing, no registered routing, parent-composed
  child-input cones on, parent-local flops on). The new matrix scenario
  `phase4_recur_d3_stateful_parent_composed_mixed_support_child_input`
  per construction strategy uses `4,4` child-instance bounds. The slice
  does not change the generator — it tightens the gate around an
  already-supported capability. Validation includes the focused
  pipeline regression, `cargo test --bin tool_matrix`, and the full
  downstream-clean r62 gate at
  `/tmp/anvil-tool-matrix-phase4-hierarchy-r62/tool_matrix_report.json`
  with `135` scenarios / `540` designs, `coverage_gaps = []`,
  `saw_recursive_hierarchy_depth_3_stateful_parent_composed_mixed_support_child_inputs = true`,
  and `540/0` pass-fail in Verilator plus both repo-owned Yosys modes.
  The depth-3 push is now complete across all four mixed-support cells:
  parent-flops (r58), no-state child-input mixed-support (r59),
  no-state parent-output mixed-support (r60), stateful parent-output
  mixed-support (r61), and stateful child-input mixed-support (r62).
- Previous Phase 4 hierarchy r61 batch pushed the recursive stateful
  parent-port-composed parent-output surface (r55's depth-2 territory)
  to exact hierarchy depth 3 without helpers. The new
  `saw_recursive_hierarchy_depth_3_stateful_parent_port_composed_outputs`
  fact requires `realized_max_leaf_depth >= 3`, hierarchy-wide
  parent-port-composed outputs and through-parent-flop variants
  exceeding top-only, parent-local flops below the top, and
  `hierarchy_parent_cone_instances == 0`. A focused exact-depth-3 proof
  `recursive_hierarchy_stateful_parent_outputs_mix_parent_ports_at_depth_3_without_helpers`
  isolates the stateful parent-output cone surface across two
  intermediate parent layers (no helpers, no sibling routing, no
  registered routing, no parent-composed child-input cones,
  parent-local flops on). The new matrix scenario
  `phase4_recur_d3_stateful_parent_port_composed_output` per construction
  strategy uses `2,2` child-instance bounds. The slice does not change
  the generator — it tightens the gate around an already-supported
  capability. Validation includes the focused pipeline regression,
  `cargo test --bin tool_matrix`, and the full downstream-clean r61 gate
  at `/tmp/anvil-tool-matrix-phase4-hierarchy-r61/tool_matrix_report.json`
  with `132` scenarios / `528` designs, `coverage_gaps = []`,
  `saw_recursive_hierarchy_depth_3_stateful_parent_port_composed_outputs = true`,
  and `528/0` pass-fail in Verilator plus both repo-owned Yosys modes.
- Previous Phase 4 hierarchy r60 batch pushed the recursive
  parent-port-composed parent-output surface (r54's depth-2 territory)
  to exact hierarchy depth 3 without helpers or state. The new
  `saw_recursive_hierarchy_depth_3_parent_port_composed_outputs` fact
  requires `realized_max_leaf_depth >= 3`, hierarchy-wide parent-composed
  and parent-port-composed output counters exceeding top-only,
  `hierarchy_parent_cone_instances == 0`, and `hierarchy_parent_local_flops
  == 0`. A focused exact-depth-3 proof
  `recursive_hierarchy_parent_outputs_mix_parent_ports_at_depth_3_without_helpers`
  isolates the parent-output cone surface across two intermediate parent
  layers (no helpers, no sibling routing, no registered routing, no
  parent-composed child-input cones, no parent-local flops). The new
  matrix scenario `phase4_recur_d3_parent_port_composed_output` per
  construction strategy uses `2,2` child-instance bounds. The slice does
  not change the generator — it tightens the gate around an
  already-supported capability. Validation includes the focused pipeline
  regression, `cargo test --bin tool_matrix`, and the full
  downstream-clean r60 gate at
  `/tmp/anvil-tool-matrix-phase4-hierarchy-r60/tool_matrix_report.json`
  with `129` scenarios / `516` designs, `coverage_gaps = []`,
  `saw_recursive_hierarchy_depth_3_parent_port_composed_outputs = true`,
  and `516/0` pass-fail in Verilator plus both repo-owned Yosys modes.
- Previous Phase 4 hierarchy r59 batch pushed the recursive unregistered
  parent-composed mixed-support child-input surface from exact depth 2
  (r53) to exact depth 3 without helpers. The new
  `saw_recursive_hierarchy_depth_3_mixed_support_child_inputs` fact
  requires `realized_max_leaf_depth >= 3`, hierarchy-wide unregistered
  parent-composed and mixed-support child-input bindings exceeding
  top-only, and `hierarchy_parent_cone_instances == 0`. A focused
  exact-depth-3 proof
  `recursive_hierarchy_parent_composed_routes_mix_parent_ports_at_depth_3_without_helpers`
  isolates the surface across two intermediate parent layers (no helpers,
  no sibling routing, no registered routing, no parent-local flops). The
  new matrix scenario `phase4_recur_d3_parent_composed_mixed_support_child_input`
  per construction strategy uses `4,4` child-instance bounds (distinct
  from r58's depth-3 / `2,2` parent-state shape). The slice does not
  change the generator — it tightens the gate around an already-supported
  capability. Validation includes the focused pipeline regression,
  `cargo test --bin tool_matrix`, and the full downstream-clean r59 gate
  at `/tmp/anvil-tool-matrix-phase4-hierarchy-r59/tool_matrix_report.json`
  with `126` scenarios / `504` designs, `coverage_gaps = []`,
  `saw_recursive_hierarchy_depth_3_mixed_support_child_inputs = true`,
  and `504/0` pass-fail in Verilator plus both repo-owned Yosys modes.
- Previous Phase 4 hierarchy r58 batch pushed the recursive parent-state
  surface from exact depth 2 to exact depth 3. The new
  `saw_recursive_hierarchy_depth_3_parent_local_flops` fact requires
  `realized_max_leaf_depth >= 3`, `hierarchy_parent_local_flops >
  top_local_flops`, and `internal_module_occurrences_with_local_flops > 0`.
  A focused exact-depth-3 proof
  `recursive_hierarchy_parents_can_emit_local_flops_at_depth_3` isolates
  the parent-flop surface across two intermediate hierarchy layers (no
  helpers, no sibling routing, no registered routing, no parent-composed
  child-input cones). The new matrix scenario
  `phase4_recur_d3_parent_state` per construction strategy uses `2,2`
  child-instance bounds (distinct from r57's depth-2 / `4,4` shape). The
  slice does not change the generator — it tightens the gate around an
  already-supported capability. Validation includes the focused pipeline
  regression, `cargo test --bin tool_matrix`, and the full
  downstream-clean r58 gate at
  `/tmp/anvil-tool-matrix-phase4-hierarchy-r58/tool_matrix_report.json`
  with `123` scenarios / `492` designs, `coverage_gaps = []`,
  `saw_recursive_hierarchy_depth_3_parent_local_flops = true`, and
  `492/0` pass-fail in Verilator plus both repo-owned Yosys modes.
- Previous Phase 4 hierarchy r57 batch promoted recursive non-top
  parent-local flops to first-class gated coverage. The new
  `saw_recursive_hierarchy_parent_local_flops` fact requires
  `realized_max_leaf_depth > 1`, `hierarchy_parent_local_flops >
  top_local_flops`, and `internal_module_occurrences_with_local_flops > 0`.
  A focused exact-depth-2 proof
  `recursive_hierarchy_parents_can_emit_local_flops_below_top` isolates
  the parent-flop surface by disabling helpers, sibling routing,
  registered routing, and parent-composed child-input cones. The new
  matrix scenario `phase4_recur_d2_parent_state` per construction
  strategy uses `4,4` child-instance bounds (distinct from r55's `2,2`)
  so the parent-state surface has its own labeled focus point in the
  matrix. Validation includes the focused pipeline regression, `cargo
  test --bin tool_matrix`, and the full downstream-clean r57 gate at
  `/tmp/anvil-tool-matrix-phase4-hierarchy-r57/tool_matrix_report.json`
  with `120` scenarios / `480` designs, `coverage_gaps = []`,
  `saw_recursive_hierarchy_parent_local_flops = true`, and `480/0`
  pass-fail in Verilator plus both repo-owned Yosys modes.
- Previous Phase 4 hierarchy r56 batch proves recursive non-top stateful
  unregistered parent-composed mixed-support child-input routing without
  helper instances. A new `child_input_bindings_from_stateful_parent_composed_mixed_support`
  metric counts unregistered parent-composed (Gate-node) child-input
  bindings whose dep set spans parent ports, child instance outputs, and
  parent-local flops simultaneously. The focused exact-depth-2 proof
  asserts hierarchy-wide unregistered parent-composed mixed-support
  counters exceed their top-only counterparts and that parent-local flops
  are present below the top parent, while helper-instance, registered
  sibling, and registered parent-composed counters remain zero. The
  Phase 4 matrix adds
  `phase4_recur_d2_stateful_parent_composed_mixed_support_child_input` per
  construction strategy and now requires
  `saw_recursive_hierarchy_stateful_parent_composed_mixed_support_child_inputs`.
  Validation includes the focused pipeline regression, `cargo test --bin
  tool_matrix`, and the full downstream-clean r56 gate at
  `/tmp/anvil-tool-matrix-phase4-hierarchy-r56/tool_matrix_report.json` with
  `117` scenarios / `468` designs, `coverage_gaps = []`,
  `saw_recursive_hierarchy_stateful_parent_port_composed_outputs = true`,
  `saw_recursive_hierarchy_stateful_parent_composed_mixed_support_child_inputs = true`,
  and `468/0` pass-fail in Verilator plus both repo-owned Yosys modes.
- Previous Phase 4 hierarchy r55 batch proves recursive non-top stateful
  parent outputs that mix parent data ports, child outputs, and
  parent-local Qs without helper instances. The focused exact-depth-2
  proof asserts hierarchy-wide parent-composed and parent-port-composed
  output counters exceed their top-only counterparts, that
  parent-port-composed outputs through parent-local flops are present
  below the top parent, and that helper-instance output counters remain
  zero. The Phase 4 matrix adds
  `phase4_recur_d2_stateful_parent_port_composed_output` per construction
  strategy and now requires
  `saw_recursive_hierarchy_stateful_parent_port_composed_outputs`.
  Validation includes the focused pipeline regression, `cargo test --bin
  tool_matrix`, and the full downstream-clean r55 gate at
  `/tmp/anvil-tool-matrix-phase4-hierarchy-r55/tool_matrix_report.json` with
  `114` scenarios / `456` designs, `coverage_gaps = []`,
  `saw_hierarchy_parent_port_composed_outputs = true`,
  `saw_recursive_hierarchy_parent_port_composed_outputs = true`,
  `saw_recursive_hierarchy_stateful_parent_port_composed_outputs = true`,
  and `456/0` pass-fail in Verilator plus both repo-owned Yosys modes.
- Previous Phase 4 hierarchy r54 batch proves recursive non-top parent outputs
  that mix parent data ports with child outputs without helper instances or
  parent-local state. The focused exact-depth-2 proof asserts hierarchy-wide
  parent-composed and parent-port-composed output counters exceed their
  top-only counterparts while helper-instance, parent-local-flop, and
  helper-output counters stay zero. The Phase 4 matrix adds
  `phase4_recur_d2_parent_port_composed_output` per construction strategy and
  now requires `saw_recursive_hierarchy_parent_port_composed_outputs`.
  Validation includes the focused pipeline regression, `cargo test --bin
  tool_matrix`, and the full downstream-clean r54 gate at
  `/tmp/anvil-tool-matrix-phase4-hierarchy-r54/tool_matrix_report.json` with
  `111` scenarios / `444` designs, `coverage_gaps = []`,
  `saw_hierarchy_parent_port_composed_outputs = true`,
  `saw_recursive_hierarchy_parent_port_composed_outputs = true`, and `444/0`
  pass-fail in Verilator plus both repo-owned Yosys modes. The commit-workflow
  hygiene gate is also clean: `cargo check --all-targets`, `cargo test` (`197`
  lib, `5` main, `26` tool-matrix, `79` integration, `0` doctests),
  `cargo clippy --all-targets -- -D warnings`, `cargo fmt --all --check`,
  `mdbook build book`, and `git --no-pager diff --check`.
- Previous Phase 4 hierarchy r53 batch proves recursive non-top unregistered
  parent-composed mixed-support child-input routing without helper
  instances. The generator now promotes no-helper parent-composed
  child-input cones to include both parent-port and sibling-output
  support when possible, while helper-required cones keep the helper
  preservation path. The focused exact-depth-2 proof asserts
  hierarchy-wide parent-composed and mixed-support child-input counters
  exceed their top-only counterparts while helper, registered sibling,
  and registered parent-composed counters stay zero. The Phase 4 matrix
  adds `phase4_recur_d2_parent_composed_mixed_support_child_input` per
  construction strategy and now requires
  `saw_recursive_hierarchy_mixed_support_child_inputs`. Validation
  includes the focused pipeline regression, `cargo test --bin
  tool_matrix`, and the full downstream-clean r53 gate at
  `/tmp/anvil-tool-matrix-phase4-hierarchy-r53/tool_matrix_report.json`
  with `108` scenarios / `432` designs, `coverage_gaps = []`,
  `saw_hierarchy_mixed_support_child_inputs = true`,
  `saw_recursive_hierarchy_mixed_support_child_inputs = true`, and
  `432/0` pass-fail in Verilator plus both repo-owned Yosys modes. The
  commit-workflow hygiene gate is also clean: `cargo check --all-targets`,
  `cargo test` (`197` lib, `5` main, `26` tool-matrix, `78` integration,
  `0` doctests), `cargo clippy --all-targets -- -D warnings`,
  `cargo fmt --all --check`, `mdbook build book`, and
  `git --no-pager diff --check`.
- Previous Phase 4 hierarchy r52 batch proves recursive non-top direct
  registered sibling mixed-support routing. No generator path change was
  needed: the r51 direct registered sibling mixed-support route uses the
  same parent-generation logic below the top parent. The new focused
  exact-depth-2 proof asserts hierarchy-wide direct registered sibling
  and registered sibling mixed-support counters exceed their top-only
  counterparts while helper-instance and registered parent-composed
  counters stay zero. The Phase 4 matrix adds
  `phase4_recur_d2_registered_sibling_mixed_support_state` per
  construction strategy and now requires
  `saw_recursive_hierarchy_registered_sibling_mixed_support_routing`.
  Validation so far includes the focused pipeline regression,
  `cargo test --bin tool_matrix`, and the full downstream-clean r52 gate
  at `/tmp/anvil-tool-matrix-phase4-hierarchy-r52/tool_matrix_report.json`
  with `105` scenarios / `420` designs, `coverage_gaps = []`,
  `saw_hierarchy_registered_sibling_mixed_support_routing = true`,
  `saw_recursive_hierarchy_registered_sibling_mixed_support_routing = true`,
  and `420/0` pass-fail in Verilator plus both repo-owned Yosys modes.
- Earlier Phase 4 hierarchy batch includes stateful
  parent-composed helper child-input mixed-support metrics and
  coverage. It distinguishes unregistered parent-composed child-input
  bindings that consume helper-sourced parent-local Qs from the
  stricter case where the same final binding also carries parent-port
  support. New metrics include
  `top_child_input_bindings_from_parent_cone_instance_flop_mixed_support`,
  `child_input_bindings_from_parent_cone_instance_flop_mixed_support`,
  `top_parent_cone_instance_flop_mixed_support_child_input_binding_fraction`,
  and
  `parent_cone_instance_flop_mixed_support_child_input_binding_fraction`.
  Validation so far includes the focused metrics regression,
  `cargo test --bin tool_matrix`, and a coverage-only Phase 4 dry run at
  `/tmp/anvil-tool-matrix-phase4-stateful-helper-child-input-mixed-check/tool_matrix_report.json`
  with `99` scenarios / `396` designs, `coverage_gaps = []`,
  `saw_hierarchy_parent_composed_parent_cone_instance_flop_mixed_support_routing = true`,
  and
  `saw_recursive_hierarchy_parent_composed_parent_cone_instance_flop_mixed_support_routing = true`.
  The full downstream-clean `r50` bank now carries these facts through
  Verilator and both repo-owned Yosys modes; the coverage-only dry run
  remains a focused breadcrumb.
- Earlier Phase 4 hierarchy batch also includes
  unregistered parent-composed helper child-input mixed-support metrics
  and coverage.
  It distinguishes child-input bindings that reach parent-cone helper
  outputs from the stricter case where the same unregistered
  parent-composed binding also carries parent-port support. The
  generator now repairs required helper-backed child-input cones by
  adding a parent-port companion when the helper route would otherwise
  lack ports. New metrics include
  `top_child_input_bindings_from_parent_cone_instance_mixed_support`,
  `child_input_bindings_from_parent_cone_instance_mixed_support`,
  `top_parent_cone_instance_mixed_support_child_input_binding_fraction`,
  and
  `parent_cone_instance_mixed_support_child_input_binding_fraction`.
  Validation so far includes the focused metrics regression,
  `cargo test --bin tool_matrix`, and a coverage-only Phase 4 dry run at
  `/tmp/anvil-tool-matrix-phase4-parent-helper-child-input-mixed-check/tool_matrix_report.json`
  with `99` scenarios / `396` designs, `coverage_gaps = []`,
  `saw_hierarchy_parent_cone_instance_mixed_support_routing = true`, and
  `saw_recursive_hierarchy_parent_cone_instance_mixed_support_routing = true`.
  The full downstream-clean `r50` bank now carries these facts through
  Verilator and both repo-owned Yosys modes; the coverage-only dry run
  remains a focused breadcrumb.
- Earlier Phase 4 hierarchy batch also includes stateful
  parent-output helper mixed-support metrics and coverage. It distinguishes parent
  outputs that reach parent-cone helper instances through parent-local
  flops from the stricter case where the same output cone also carries
  parent-port support. New metrics include
  `top_outputs_reaching_parent_cone_instances_through_parent_flops_with_mixed_support`,
  `hierarchy_outputs_reaching_parent_cone_instances_through_parent_flops_with_mixed_support`,
  `top_parent_cone_instance_flop_mixed_support_output_fraction`, and
  `hierarchy_parent_cone_instance_flop_mixed_support_output_fraction`.
  The tool-matrix gate now requires nonrecursive and recursive coverage
  facts for that overlap and also requires decision-site attempts for
  the plain `hierarchy_sibling_route_prob` knob. Validation included
  the focused metrics regression, `cargo test --bin tool_matrix`, a
  coverage-only Phase 4 dry run at
  `/tmp/anvil-tool-matrix-phase4-mixed-helper-check/tool_matrix_report.json`
  with `99` scenarios / `396` designs and no coverage gaps,
  `cargo check --all-targets`, and full `cargo test` with 302 passing tests.
  The full downstream-clean `r50` bank now carries these facts through
  Verilator and both repo-owned Yosys modes; the coverage-only dry run
  remains a focused breadcrumb.
- Prior Phase 4 hierarchy slice lets parent-output helper routing below
  the top parent mix parent-port support into the same helper-backed
  output cone in exact-depth-2 recursive hierarchy. With
  `hierarchy_parent_cone_instance_prob = 1.0`, helper budget `3`, two
  child instances per recursive parent, terminal reuse forced on, and
  sibling, registered sibling, child-input cone, registered child-input
  cone, and parent-flop route probabilities disabled, a non-top parent
  can instantiate helper children for output cones and still pull in
  parent data ports. Key metrics are
  `hierarchy_parent_cone_instances > top_parent_cone_instances`,
  `hierarchy_outputs_reaching_parent_cone_instances >
  top_outputs_reaching_parent_cone_instances`,
  `hierarchy_outputs_reaching_parent_cone_instance_mixed_support >
  top_outputs_reaching_parent_cone_instance_mixed_support`,
  `hierarchy_parent_cone_instance_mixed_support_output_fraction > 0.0`,
  and
  `hierarchy_outputs_reaching_parent_cone_instances_through_parent_flops = 0`.
- Prior Phase 4 hierarchy slice lets registered parent-composed helper
  child-input routing below the top parent mix parent-port support into
  the helper-sourced D cone in exact-depth-2 recursive hierarchy. With
  `hierarchy_registered_child_input_cone_prob = 1.0`,
  `hierarchy_parent_cone_instance_prob = 1.0`, a helper budget of `3`,
  two child instances per recursive parent, and sibling, unregistered
  child-input, and parent-flop route probabilities disabled, a non-top
  parent can instantiate a helper child, use the helper output in
  registered parent-composed D logic, mix in parent data ports, and bind
  a later child input through the resulting parent-local Q. Key metrics
  are `hierarchy_parent_cone_instances > top_parent_cone_instances`,
  `hierarchy_parent_local_flops > top_local_flops`,
  `child_input_bindings_from_registered_parent_composed_logic >
  top_child_input_bindings_from_registered_parent_composed_logic`,
  `child_input_bindings_from_registered_parent_cone_instances >
  top_child_input_bindings_from_registered_parent_cone_instances`,
  `child_input_bindings_from_registered_parent_cone_instance_mixed_support >
  top_child_input_bindings_from_registered_parent_cone_instance_mixed_support`,
  and
  `registered_parent_cone_instance_mixed_support_child_input_binding_fraction > 0.0`.
- Prior Phase 4 hierarchy slice lets registered parent-composed
  child-input routing below the top parent combine mixed support and
  multi-stage parent-local Q reuse without helper instances in
  exact-depth-2 recursive hierarchy. With
  `hierarchy_registered_child_input_cone_prob = 1.0`, four child
  instances per recursive parent, and sibling, unregistered
  child-input, parent-cone-helper, and parent-flop route probabilities
  disabled, a non-top parent can build a registered D cone from parent
  ports, child outputs, and an earlier parent-local Q, then bind a
  later child input through that Q. Key metrics are
  `hierarchy_parent_local_flops > top_local_flops`,
  `child_input_bindings_from_registered_parent_composed_logic >
  top_child_input_bindings_from_registered_parent_composed_logic`,
  `child_input_bindings_from_registered_mixed_support >
  top_child_input_bindings_from_registered_mixed_support`,
  `child_input_bindings_from_registered_multistage_parent_composed_logic >
  top_child_input_bindings_from_registered_multistage_parent_composed_logic`,
  `child_input_bindings_from_registered_multistage_mixed_support >
  top_child_input_bindings_from_registered_multistage_mixed_support`,
  `registered_multistage_mixed_support_child_input_binding_fraction > 0.0`,
  `child_input_bindings_from_registered_parent_cone_instances = 0`,
  `child_input_bindings_from_registered_multistage_parent_cone_instances = 0`,
  and
  `child_input_bindings_from_registered_multistage_parent_composed_parent_cone_instances = 0`.
- Prior Phase 4 hierarchy slice lets direct registered sibling-routed
  child-input routing below the top parent chain through earlier
  parent-local Qs without helper instances or parent-composed D logic
  in exact-depth-2 recursive hierarchy. With
  `hierarchy_registered_sibling_route_prob = 1.0`, four child
  instances per recursive parent, and sibling, registered
  parent-composed, unregistered parent-composed, helper, and
  parent-flop route probabilities disabled, a non-top parent can bind
  one child input through an earlier child output captured in
  parent-local state, then reuse that Q as the D source for a later
  registered sibling route. Key metrics are
  `hierarchy_parent_local_flops > top_local_flops`,
  `child_input_bindings_from_registered_instance_outputs >
  top_child_input_bindings_from_registered_instance_outputs`,
  `child_input_bindings_from_registered_multistage_instance_outputs >
  top_child_input_bindings_from_registered_multistage_instance_outputs`,
  `registered_multistage_instance_output_child_input_binding_fraction > 0.0`,
  `child_input_bindings_from_registered_parent_composed_logic = 0`,
  `child_input_bindings_from_registered_multistage_parent_composed_logic = 0`,
  `child_input_bindings_from_registered_parent_cone_instances = 0`,
  and
  `child_input_bindings_from_registered_multistage_parent_cone_instances = 0`.
- Prior Phase 4 hierarchy slice lets registered parent-composed
  child-input routing below the top parent chain through earlier
  parent-local Qs without helper instances in exact-depth-2 recursive
  hierarchy. With `hierarchy_registered_child_input_cone_prob = 1.0`,
  four child instances per recursive parent, and sibling, child-input,
  parent-cone-helper, and parent-flop route probabilities disabled, a
  non-top parent can bind one child input through parent-local state and
  reuse that earlier parent-local Q in later registered parent-composed
  child-input logic. Key metrics are
  `hierarchy_parent_local_flops > top_local_flops`,
  `child_input_bindings_from_registered_parent_composed_logic >
  top_child_input_bindings_from_registered_parent_composed_logic`,
  `child_input_bindings_from_registered_multistage_parent_composed_logic >
  top_child_input_bindings_from_registered_multistage_parent_composed_logic`,
  `registered_multistage_parent_composed_child_input_binding_fraction > 0.0`,
  `child_input_bindings_from_registered_parent_cone_instances = 0`,
  `child_input_bindings_from_registered_multistage_parent_cone_instances = 0`,
  and
  `child_input_bindings_from_registered_multistage_parent_composed_parent_cone_instances = 0`.
- Prior Phase 4 hierarchy slice lets registered parent-composed
  child-input routing below the top parent mix parent data ports with
  child outputs without helper instances in exact-depth-2 recursive
  hierarchy. With `hierarchy_registered_child_input_cone_prob = 1.0`,
  sibling, child-input, parent-cone-helper, and parent-flop route
  probabilities disabled, a non-top parent can build D-side
  parent-composed logic from both parent ports and child outputs, then
  bind later child inputs through parent-local flops. Key metrics are
  `hierarchy_parent_local_flops > top_local_flops`,
  `child_input_bindings_from_registered_parent_composed_logic >
  top_child_input_bindings_from_registered_parent_composed_logic`,
  `child_input_bindings_from_registered_instance_outputs >
  top_child_input_bindings_from_registered_instance_outputs`,
  `child_input_bindings_from_registered_mixed_support >
  top_child_input_bindings_from_registered_mixed_support`, and
  `child_input_bindings_from_registered_parent_cone_instances = 0`.
- Prior Phase 4 hierarchy slice lets parent-composed child-input
  routing below the top parent spend a multi-helper parent-cone instance
  budget in exact-depth-2 recursive hierarchy. With
  `hierarchy_child_input_cone_prob = 1.0`,
  `hierarchy_parent_cone_instance_prob = 1.0`,
  `max_parent_cone_instances_per_module = 3`, and registered/stateful
  child-input route probabilities disabled, a non-top parent can
  instantiate multiple helper children and use those helper outputs in
  parent-composed child-input bindings without proving a
  helper-through-flop route or registered child-input helper D cone.
  Key metrics are
  `max_parent_cone_instances_per_internal_module = 3`,
  `hierarchy_parent_cone_instances > top_parent_cone_instances`,
  `child_input_bindings_from_parent_composed_logic >
  top_child_input_bindings_from_parent_composed_logic`,
  `child_input_bindings_from_parent_cone_instances >
  top_child_input_bindings_from_parent_cone_instances`,
  `child_input_bindings_from_parent_cone_instances_through_parent_flops = 0`, and
  `child_input_bindings_from_registered_parent_cone_instances = 0`.
- Prior Phase 4 hierarchy slice lets parent outputs below the top
  parent depend on parent-cone helper instance outputs through
  parent-local flops in exact-depth-2 recursive hierarchy. With
  `hierarchy_parent_cone_instance_prob` active and child-input route
  probabilities disabled, a non-top parent can instantiate helper
  children, register helper outputs into parent-local state, and use
  those helper-sourced Qs in parent-output cones without proving a
  child-input helper binding or a registered child-input helper D cone.
  Key metrics are
  `hierarchy_parent_cone_instances > top_parent_cone_instances`,
  `hierarchy_parent_local_flops > top_local_flops`,
  `hierarchy_outputs_reaching_parent_cone_instances_through_parent_flops >
  top_outputs_reaching_parent_cone_instances_through_parent_flops`,
  `child_input_bindings_from_parent_cone_instances = 0`, and
  `child_input_bindings_from_registered_parent_cone_instances = 0`.
- Latest cleanup slice also fixed a warning-only downstream failure found
  by the first full `r32` attempt: exact-selector `CaseMux` / `CasezMux`
  arms now participate in bounds/exact-value reasoning, allowing
  provably constant dynamic shifts to fold before Yosys warns.
- Prior Phase 4 hierarchy slice lets parent-composed helper
  child-input routes pass through parent-local state without becoming
  registered child-input routes. With `hierarchy_child_input_cone_prob`,
  `hierarchy_parent_cone_instance_prob`, and
  `hierarchy_parent_flop_prob` active, a helper output can seed a
  parent-local Q and unregistered parent-composed child-input logic can
  consume that helper Q before binding a later child input.
  The focused regression is
  `cargo test hierarchy_parent_composed_helper_routes_can_use_parent_flops`;
  key metrics are
  `child_input_bindings_from_parent_cone_instances_through_parent_flops`,
  `top_child_input_bindings_from_parent_cone_instances_through_parent_flops`,
  `parent_cone_instance_flop_child_input_binding_fraction`, and
  `top_parent_cone_instance_flop_child_input_binding_fraction`, with
  `child_input_bindings_from_registered_parent_cone_instances` kept at
  zero in the focused proof.
- Prior Phase 4 hierarchy slice lets registered parent-composed helper
  routes chain through parent-local state. With a one-helper budget, a
  helper output seeds an earlier parent Q and later
  `hierarchy_registered_child_input_cone_prob` routes can reuse that Q
  inside parent-composed D logic before registering the next child
  input.
  The focused regression is
  `cargo test hierarchy_registered_parent_composed_routes_can_chain_helper_instances_through_parent_flops`;
  key metrics are
  `child_input_bindings_from_registered_multistage_parent_composed_parent_cone_instances`,
  `top_child_input_bindings_from_registered_multistage_parent_composed_parent_cone_instances`,
  `registered_multistage_parent_composed_parent_cone_instance_child_input_binding_fraction`,
  and
  `top_registered_multistage_parent_composed_parent_cone_instance_child_input_binding_fraction`,
  with the direct registered sibling helper multistage counter kept at
  zero in the focused proof.
- Prior Phase 4 hierarchy slice lets direct registered sibling helper
  routes chain through parent-local state. With a one-helper budget, a
  helper output seeds the first parent Q and later
  `hierarchy_registered_sibling_route_prob` routes can choose that Q as
  a later flop D source without using registered parent-composed logic.
  The focused regression is
  `cargo test hierarchy_registered_sibling_routes_can_chain_helper_instances_through_parent_flops`;
  key metrics are
  `child_input_bindings_from_registered_multistage_parent_cone_instances`,
  `top_child_input_bindings_from_registered_multistage_parent_cone_instances`,
  `registered_multistage_parent_cone_instance_child_input_binding_fraction`,
  and
  `top_registered_multistage_parent_cone_instance_child_input_binding_fraction`,
  with registered parent-composed counters kept at zero.
- Prior Phase 4 hierarchy slice lets parent-output helper sources
  route through parent-local state when both
  `hierarchy_parent_cone_instance_prob` and `hierarchy_parent_flop_prob`
  are active. The focused regression is
  `cargo test hierarchy_parent_outputs_can_route_helper_instances_through_parent_flops`;
  key metrics are
  `top_outputs_reaching_parent_cone_instances_through_parent_flops`,
  `hierarchy_outputs_reaching_parent_cone_instances_through_parent_flops`,
  `top_parent_cone_instance_flop_output_fraction`, and
  `hierarchy_parent_cone_instance_flop_output_fraction`, with
  child-input helper bindings kept at zero for the output-only proof.
- Prior Phase 4 hierarchy slice broadened direct registered sibling
  routing so later `hierarchy_registered_sibling_route_prob` routes can
  choose earlier parent-local Qs as D sources and create multi-stage
  registered child-to-child chains without parent-composed logic. The
  focused regression is
  `cargo test hierarchy_registered_sibling_routes_can_chain_through_parent_flops`;
  key metrics are
  `child_input_bindings_from_registered_multistage_instance_outputs`,
  the top-level counterpart, and the two
  `registered_multistage_instance_output_*` fractions, with registered
  parent-composed counters kept at zero.
- Prior Phase 4 hierarchy slice broadened direct sibling routing so
  `hierarchy_sibling_route_prob` can allocate a parent-cone helper
  instance when `hierarchy_parent_cone_instance_prob` fires and bind a
  later child input directly from that helper output. The focused
  regression proves helper instances are present beyond planned child
  slots, direct sibling bindings exist,
  `child_input_bindings_from_parent_cone_instances > 0`, and the
  registered helper counters stay zero.
- Prior Phase 4 hierarchy slice broadened direct registered sibling
  routing so `hierarchy_registered_sibling_route_prob` can allocate a
  parent-cone helper instance when `hierarchy_parent_cone_instance_prob`
  fires and use that helper output as the parent-flop D source. The
  focused regression proves helper instances are present beyond planned
  child slots, registered sibling bindings exist,
  `child_input_bindings_from_registered_parent_composed_logic = 0`, and
  `child_input_bindings_from_registered_parent_cone_instances > 0`.
- Prior Phase 4 hierarchy slice broadened parent-output helper
  placement so parent-output composition can allocate multiple
  parent-cone helper instances up to the configured per-parent budget.
  The focused regression forces child-input helper routes off and proves
  `top_parent_cone_instances = 3`,
  `max_parent_cone_instances_per_internal_module = 3`,
  `child_input_bindings_from_parent_cone_instances = 0`, and parent
  outputs reaching helper outputs.
- Prior package-metadata cleanup fixes the remaining stale
  `constrained-random` purpose wording in `Cargo.toml`. The crate
  package description now says ANVIL is a random by-construction
  generator of synthesizable SystemVerilog RTL, matching the accepted
  README/Rustdoc/mdBook terminology.
- Prior Phase 4 hierarchy gate refresh fixed the gate budget so the
  repo-owned hierarchy matrix preserves four designs per scenario. The
  corrected pre-direct-helper full downstream-clean bank was
  `/tmp/anvil-tool-matrix-phase4-hierarchy-r23/tool_matrix_report.json`:
  42 scenarios, 4 designs/scenario, 168 total designs,
  `coverage_gaps = []`, and 168/0 pass-fail in Verilator plus both
  repo-owned Yosys modes. The current `r31` bank supersedes it for the
  expanded hierarchy policy. The clean pre-fix `r22` run is
  root-cause evidence only: the stale 120-design total floor produced
  42 scenarios at 3 designs/scenario, or 126 total designs.
- Prior documentation-continuity slice combines the README/session
  bootstrap drift fix with the purpose-terminology clarification. The
  live docs now align `src/metrics.rs` probability-roll telemetry with
  `Module::knob_rolls`, refresh relevant test-count/source-state
  references, and consistently describe ANVIL as a random
  by-construction synthesizable SystemVerilog RTL generator whose
  legal HDL artifacts target downstream HDL consumers while
  Verilator/Yosys remain repository validation tools.
- Latest purpose-terminology clarification: describe ANVIL as a
  **random by-construction synthesizable SystemVerilog RTL generator**,
  not as SV/UVM-style constrained-random generation. Verilator and Yosys
  are repository validation tools for syntax/elaboration/lint/synthesis
  acceptance; the generated artifacts target the broader class of
  downstream HDL consumers that accept synthesizable HDL. ANVIL corpora
  can still be used to stress parsers, elaborators, RTL compilers,
  linters, simulators, synthesizers, and related tools, but that is a use
  of legal generated RTL rather than the product being a malformed-input
  fuzzer or generic toolchain stress tester.
- Latest README / bootstrap execution rechecked the authority docs, the
  mdBook map, the Rust/test/example inventory, and the current hygiene
  gates. It found only small hierarchy-parent wording drift after the
  local parent-state slice: a few summaries still said "wrappers" or
  "combinational top-output cones" where the current surface is
  broader. The docs now say hierarchy parents keep `clk` / `rst_n`
  visible iff they carry local state or sequential descendants, and
  parent-side output cones are combinational by default but optionally
  stateful under `hierarchy_parent_flop_prob`.
- Latest mdBook audit compared `anvil --help` and `tool_matrix --help`
  against `book/src`: no current help flags are missing from the book.
  `book/src/recipes.md` now has concrete hierarchy recipes for
  multi-file output, reuse/under-instantiation, recursive hierarchy,
  on-demand children, parent-cone helper instances, and registered
  hierarchy routes.
- Latest live-doc alignment pass removed stale current-state wording
  that still treated hierarchy as pre-Phase-4 or only wrapper-shaped.
  Active README / USER_GUIDE / CODEBASE_ANALYSIS / ROADMAP / mdBook
  text now distinguishes the current Phase 4 hierarchy surface from the
  still-open future work: parameterization, aggregates, memories, FSMs,
  broader helper placement, broader hierarchy-aware identity, and
  future artifact-family plumbing.
- The current live Phase 4 slice is no longer just a depth-1 wrapper. `hierarchy_depth = 1` plus `num_leaf_modules >= 1` still generates a library of leaf modules plus a real top module; `num_child_instances = 0` preserves the legacy exact-once behavior, `num_child_instances < num_leaf_modules` under-instantiates the library, and `num_child_instances > num_leaf_modules` reuses child definitions.
- Current HEAD now also has an explicit hierarchy child-sourcing axis: `hierarchy_child_source_mode = library | on-demand`. Both the legacy wrapper lane and the bounded recursive lane can choose between reusable child-definition pools and fresh child-definition synthesis per planned instance slot.
- Current HEAD now makes `on-demand` stronger than "fresh per slot":
  every planned on-demand child slot carries a parent-planned exact
  data-interface profile, and the realized child module is required to
  emit that exact data-input/output shape. Control ports remain
  structural (`clk` / `rst_n` propagate only when sequential state is
  present), but data-interface demand is now explicit and validated.
- Current HEAD now also has an explicit sibling-routing surface in the
  hierarchy planner. Both wrapper and recursive parents may bind later
  child data inputs from earlier sibling instance outputs via
  `hierarchy_sibling_route_prob`, and when
  `hierarchy_parent_cone_instance_prob` fires that direct route can
  allocate a helper child and bind from the helper output instead. The
  resulting provenance is measurable directly from design metrics.
- Current HEAD now also has an explicit parent-composed child-input
  surface. Both wrapper and recursive parents may bind child data inputs
  through parent-local combinational cones over already-available
  parent sources via `hierarchy_child_input_cone_prob`; source
  candidates are parent data inputs, earlier sibling instance outputs,
  and earlier parent-side route gates.
- Current HEAD now also has the first parent-cone helper-instance slice.
  When `hierarchy_parent_cone_instance_prob` fires, parent-composed
  child-input cones and direct sibling routes can instantiate a helper
  child as an internal parent-cone source; that helper is tagged
  separately from planned child slots, and its outputs can feed later
  child inputs either directly or through parent logic.
- Current HEAD now also broadens that helper-instance slice to
  parent-output composition. Parent-output cones can allocate helper
  children as internal parent-cone sources even when
  `hierarchy_child_input_cone_prob = 0.0`; the parent-output-only path
  can now spend multiple helpers up to
  `max_parent_cone_instances_per_module`. Metrics expose this through
  `top_outputs_reaching_parent_cone_instances`,
  `hierarchy_outputs_reaching_parent_cone_instances`, matching output
  fractions, `top_parent_cone_instances`, and
  `max_parent_cone_instances_per_internal_module`.
- Current HEAD now also has explicit parent-cone helper budgeting.
  `max_parent_cone_instances_per_module` defaults to `1` for backward
  compatibility, can be set to `0` to suppress helper allocation, and
  can be raised to let one hierarchy parent instantiate multiple
  helper children. The budget is now proven through both child-input
  helper routing and parent-output-only helper composition. Metrics
  expose the realized local budget through
  `max_parent_cone_instances_per_internal_module`.
- Current HEAD now also lets registered parent-composed child-input D
  cones use parent-cone helper instance outputs when
  `hierarchy_registered_child_input_cone_prob` and
  `hierarchy_parent_cone_instance_prob` are both active. Metrics expose
  this through
  `child_input_bindings_from_registered_parent_cone_instances`,
  `top_child_input_bindings_from_registered_parent_cone_instances`, and
  the matching registered helper-route fractions.
- Current HEAD now also lets direct registered sibling routes use
  parent-cone helper instance outputs when
  `hierarchy_registered_sibling_route_prob` and
  `hierarchy_parent_cone_instance_prob` are both active. The route still
  binds the later child input through one local parent flop, but the D
  source can be a helper output instead of only a planned sibling output.
  The registered helper metric therefore keys off the flop-D dependency
  set, not only registered parent-composed D logic.
- Current HEAD now treats module names as a generator-global hierarchy
  resource. Leaf modules, hierarchy parents, and repeated hierarchical
  designs in one `--count N --out DIR` run reserve names from the same
  sequence, so one `.sv` file per module definition cannot collide with
  a later generated design.
- Current HEAD now also has explicit local parent state. Both wrapper
  and recursive parents may emit local parent flops in parent output
  cones and parent-composed child-input cones via
  `hierarchy_parent_flop_prob`; default `0.0` preserves the
  combinational parent layer unless state is explicitly requested.
- Current HEAD now also has explicit registered sibling routing. Both
  wrapper and recursive parents may bind a later child data input from
  an earlier sibling instance output through one local parent flop via
  `hierarchy_registered_sibling_route_prob`; default `0.0` keeps this
  stateful child-to-child route opt-in and distinct from the direct
  combinational sibling route.
- Current HEAD now also has explicit registered parent-composed
  child-input routing. Both wrapper and recursive parents may bind a
  later child data input through parent-local combinational logic over
  already-available parent sources and then one local parent flop via
  `hierarchy_registered_child_input_cone_prob`; when parent data ports
  and sibling outputs are both live, that registered route can mix both
  supports, and later registered routes can chain through earlier
  parent-local Qs when those are available. Default `0.0` keeps this
  stateful parent-composed route opt-in and distinct from direct
  registered sibling routing.
- Current HEAD now also deepens parent output composition: hierarchy
  parent output cones are built over the full parent source pool, then
  repaired after finalization so every output keeps child-output
  support and, when live parent data inputs exist, can also carry
  parent-port support. Metrics expose this as
  `top_parent_port_composed_outputs`,
  `hierarchy_parent_port_composed_outputs`, and matching fractions.
- Current HEAD is no longer wrapper-only. The top module now treats child `InstanceOutput` nodes as real dep-bearing leaf variables and builds parent-side output cones over them, and bounded recursive hierarchy can now mix shallow and deep branches inside one legal tree. The parent layer still stays intentionally narrow in the remaining open ways: the first one-flop registered sibling route, the first registered parent-composed child-input route, registered mixed-support routing, the first multi-stage registered parent-composed chain, and parent-cone helper-instance routes for parent-composed child-input cones, direct sibling routes, direct registered sibling routes, registered child-input D cones, and parent-output cones with an explicit per-parent budget are live; additional helper placement, broader multi-stage registered hierarchy patterns, and hierarchy-aware identity remain open.
- The remaining intentional narrowness in the parent layer is now
  clearer: local parent flops, the first one-flop registered sibling
  route, the first registered parent-composed child-input route, the
  mixed-support registered subcase, and the first multi-stage
  registered parent-composed chain, plus the first parent-cone
  helper-instance routes for parent-composed child-input cones, direct
  sibling routes, direct registered sibling routes, registered
  child-input D cones, and parent-output cones plus explicit helper
  budgeting, are live, but
  additional helper placement, broader multi-stage registered hierarchy
  patterns, and hierarchy-aware identity are not.
- Current HEAD now also has a bounded recursive hierarchy lane driven by `min_hierarchy_depth..=max_hierarchy_depth` plus `min_child_instances_per_module..=max_child_instances_per_module`, with optional repeated `--child-instances-per-depth` overrides keyed by parent depth (`0` = top, `1` = its direct children, ...). The recursive planner now keeps subtree-local depth intervals live, so leaves stay inside the requested global range and the tree can realize both shallow and deep branches when the interval is open and the structure allows it.
- The hierarchy surface is now measurable from trustworthy numbers rather than SV inspection. `manifest.json` and `tool_matrix` design reports carry per-design `DesignMetrics` such as library coverage, unused-library fraction, instance reuse, top-interface shape, control fanout, weighted child load/complexity, instantiation histograms, direct-vs-composed top-output counts, parent-port-composed output counts/fractions, child-output dependency fractions, and instance-output support depth per top output.
- Those design metrics now also distinguish single-use vs reused child-definition structure directly:
  - `avg_instances_per_unique_instantiated_module`
  - `num_single_use_instantiated_modules`
  - `num_multiuse_instantiated_modules`
  - `single_use_instantiated_module_fraction`
- They now also quantify exact profiled child-interface quality
  directly:
  - `num_profiled_module_definitions`
  - `num_profiled_instantiated_modules`
  - `num_profiled_instance_slots`
  - `profiled_instantiated_module_fraction`
  - `profiled_instance_fraction`
  - `dep_bearing_child_input_bindings`
  - `dep_bearing_child_input_binding_fraction`
- Recursive-hierarchy metrics now also include per-parent-depth branching summaries (`avg/min/max_child_instances_by_parent_depth`) plus `leaf_module_occurrences_by_depth` on top of the existing depth histograms and raw instance-slot totals, so both branching shape and mixed-depth realization can be trusted from numbers alone.
- The control-port doctrine is explicit and design-aware: pure comb-only modules do **not** emit `clk` / `rst_n`; sequential leaves do; and hierarchy parents keep those ports visible all the way up the instantiated ancestor chain iff they carry local state or sequential descendants.
- The focused proof artifact for the metrics/control doctrine remains `/tmp/anvil-hier-metrics-smoke-r1`, which is clean in Verilator plus both repo-owned Yosys modes and records correct values such as `top_clock_inputs = 1`, `top_reset_inputs = 1`, `clock_fanout_instances = 5`, `reset_fanout_instances = 5`, `instance_reuse_fraction = 0.4`, and `library_coverage_fraction = 1.0`.
- The focused proof artifact for the new parent-composition slice is `/tmp/anvil-hier-parent-compose-smoke-r1/manifest.json`, also clean in Verilator plus both repo-owned Yosys modes. Its key design metrics are:
  - `top_parent_composed_outputs = 10`
  - `top_direct_instance_output_drives = 0`
  - `top_outputs_reaching_instance_outputs = 10`
  - `top_outputs_without_instance_outputs = 0`
  - `top_instance_output_dependency_fraction = 1.0`
  - `avg_instance_output_support_per_top_output = 2.5`
- The focused proof artifact for the new depth-specific recursive branching slice is `/tmp/anvil-hier-depth-profile-smoke-r1/manifest.json`, also clean in Verilator plus both repo-owned Yosys modes. Its key design metrics are:
  - `realized_min_leaf_depth = 2`
  - `realized_max_leaf_depth = 2`
  - `avg_child_instances_by_parent_depth = {"0": 4.0, "1": 2.0}`
  - `min_child_instances_by_parent_depth = {"0": 4, "1": 2}`
  - `max_child_instances_by_parent_depth = {"0": 4, "1": 2}`
  - `hierarchy_parent_composed_outputs = 36`
  - `top_parent_composed_outputs = 18`
- The focused proof artifact for the new mixed-depth recursive slice is `/tmp/anvil-hier-mixed-depth-smoke-r1/manifest.json`, also clean in Verilator plus both repo-owned Yosys modes. Its key design metrics are:
  - `realized_min_leaf_depth = 2`
  - `realized_max_leaf_depth = 3`
  - `leaf_module_occurrences_by_depth = {"2": 2, "3": 4}`
  - `avg_child_instances_by_parent_depth = {"0": 2.0, "1": 2.0, "2": 2.0}`
  - `hierarchy_parent_composed_outputs = 40`
  - `top_parent_composed_outputs = 14`
- The older focused proof artifact for the first explicit on-demand
  child-sourcing slice is `/tmp/anvil-hier-ondemand-wrapper-smoke-r1/manifest.json`,
  also clean in Verilator plus both repo-owned Yosys modes. Its key
  design metrics are:
  - `num_instances = 3`
  - `num_unique_instantiated_modules = 3`
  - `num_single_use_instantiated_modules = 3`
  - `single_use_instantiated_module_fraction = 1.0`
  - `instance_reuse_fraction = 0.0`
  - `unused_library_fraction = 0.0`
- The focused proof artifact for the stronger profiled on-demand slice
  is `/tmp/anvil-hier-profiled-ondemand-smoke-r1/manifest.json`, also
  clean in Verilator plus both repo-owned Yosys modes. Its key design
  metrics are:
  - `num_profiled_instance_slots = 3`
  - `profiled_instance_fraction = 1.0`
  - `profiled_instantiated_module_fraction = 1.0`
  - `dep_bearing_child_input_binding_fraction = 1.0`
- The focused proof artifact for the new sibling-routing slice is
  `/tmp/anvil-hier-sibling-routing-smoke-r1/manifest.json`, also clean
  in Verilator plus both repo-owned Yosys modes. Its key design metrics
  are:
  - `child_input_bindings_from_instance_outputs = 6`
  - `top_child_input_bindings_from_instance_outputs = 6`
  - `instance_output_child_input_binding_fraction = 0.75`
  - `top_instance_output_child_input_binding_fraction = 0.75`
- The focused proof for direct sibling helper routing is
  `cargo test hierarchy_sibling_routes_can_use_helper_instances`. Its
  key design metrics are:
  - `top_parent_cone_instances > 0`
  - `child_input_bindings_from_instance_outputs > 0`
  - `child_input_bindings_from_registered_instance_outputs = 0`
  - `child_input_bindings_from_registered_parent_cone_instances = 0`
  - `child_input_bindings_from_parent_cone_instances > 0`
  - `parent_cone_instance_child_input_binding_fraction > 0.0`
  - `top_parent_cone_instance_child_input_binding_fraction > 0.0`
  - `num_instances > planned_child_instances`
  This route is now banked in the full downstream-clean `r28` Phase 4
  matrix through the dedicated direct sibling helper scenario.
- The focused proof artifact for the new parent-composed child-input
  slice is `/tmp/anvil-hier-child-input-cone-smoke-r1/manifest.json`,
  also clean in Verilator plus both repo-owned Yosys modes. Its key
  design metrics are:
  - `child_input_bindings_from_parent_composed_logic = 13`
  - `top_child_input_bindings_from_parent_composed_logic = 13`
  - `parent_composed_child_input_binding_fraction = 0.9285714285714286`
  - `top_parent_composed_child_input_binding_fraction = 0.9285714285714286`
- The focused proof artifact for the mixed parent-port /
  child-output parent-output slice is
  `/tmp/anvil-hier-parent-output-mix-smoke-r1/manifest.json`, clean in
  Verilator plus both repo-owned Yosys modes. Its key design metrics
  are:
  - `top_parent_port_composed_outputs = 8`
  - `hierarchy_parent_port_composed_outputs = 8`
  - `top_outputs_reaching_instance_outputs = 8`
  - `top_outputs_without_instance_outputs = 0`
- The focused proof artifact for the new local-parent-state slice is
  `/tmp/anvil-hier-parent-state-smoke-r1/manifest.json`, also clean in
  Verilator plus both repo-owned Yosys modes. Its key design metrics
  are:
  - `hierarchy_parent_local_flops = 8`
  - `top_local_flops = 8`
  - `top_clock_inputs = 1`
  - `top_reset_inputs = 1`
  - `child_input_bindings_from_parent_flops = 1`
- The focused proof artifact for the new registered sibling-route slice
  is `/tmp/anvil-hier-registered-sibling-smoke-r1/manifest.json`, also
  clean in Verilator plus both repo-owned Yosys modes. Its key design
  metrics are:
  - `child_input_bindings_from_registered_instance_outputs = 4`
  - `top_child_input_bindings_from_registered_instance_outputs = 4`
  - `registered_instance_output_child_input_binding_fraction = 0.8`
  - `top_registered_instance_output_child_input_binding_fraction = 0.8`
  - `hierarchy_parent_local_flops = 3`
  - `top_clock_inputs = 1`
  - `top_reset_inputs = 1`
- The focused proof artifact for the new registered parent-composed
  child-input route slice is
  `/tmp/anvil-hier-registered-child-input-cone-smoke-r2/manifest.json`,
  also clean in Verilator plus both repo-owned Yosys modes. Its key
  design metrics are:
  - `child_input_bindings_from_registered_parent_composed_logic = 3`
  - `top_child_input_bindings_from_registered_parent_composed_logic = 3`
  - `registered_parent_composed_child_input_binding_fraction = 0.75`
  - `top_registered_parent_composed_child_input_binding_fraction = 0.75`
  - `child_input_bindings_from_registered_instance_outputs = 3`
  - `hierarchy_parent_local_flops = 3`
  - `top_clock_inputs = 1`
  - `top_reset_inputs = 1`
- The focused proof artifact for the new registered mixed-support
  child-input route slice is
  `/tmp/anvil-hier-registered-mixed-child-input-smoke-r1/manifest.json`,
  clean in Verilator plus both repo-owned Yosys modes. Its key design
  metrics are:
  - `child_input_bindings_from_registered_mixed_support = 3`
  - `top_child_input_bindings_from_registered_mixed_support = 3`
  - `registered_mixed_support_child_input_binding_fraction = 0.75`
  - `top_registered_mixed_support_child_input_binding_fraction = 0.75`
- The focused proof artifact for the new multi-stage registered
  parent-composed child-input route slice is
  `/tmp/anvil-hier-registered-multistage-child-input-smoke-r1/manifest.json`,
  clean in Verilator plus both repo-owned Yosys modes. Its key design
  metrics are:
  - `child_input_bindings_from_registered_multistage_parent_composed_logic = 2`
  - `top_child_input_bindings_from_registered_multistage_parent_composed_logic = 2`
  - `registered_multistage_parent_composed_child_input_binding_fraction = 0.5`
- The focused proof artifact for the new parent-cone helper-instance
  child-input route slice is
  `/tmp/anvil-parent-cone-instance-smoke-r1/manifest.json`, clean in
  Verilator plus both repo-owned Yosys modes. Its key design metrics
  are:
  - `top_parent_cone_instances = 1`
  - `hierarchy_parent_cone_instances = 1`
  - `child_input_bindings_from_parent_cone_instances = 4`
  - `top_child_input_bindings_from_parent_cone_instances = 4`
  - `parent_cone_instance_child_input_binding_fraction = 0.4444444444444444`
  - `top_parent_cone_instance_child_input_binding_fraction = 0.4444444444444444`
- The focused proof for the new parent-cone helper-instance
  parent-output route slice is
  `cargo test hierarchy_parent_outputs_can_depend_on_helper_instance_outputs`.
  Its key design metrics are:
  - `top_parent_cone_instances > 0`
  - `top_outputs_reaching_parent_cone_instances > 0`
  - `hierarchy_outputs_reaching_parent_cone_instances > 0`
  - `top_parent_cone_instance_output_fraction > 0.0`
  This route is now banked in the full `r30` Phase 4 matrix through the
  dedicated `phase4_hier2_inst4_parent_output_cone_instance` axis.
- The focused proof for stateful parent-output helper routing is
  `cargo test hierarchy_parent_outputs_can_route_helper_instances_through_parent_flops`.
  Its key design metrics are:
  - `top_outputs_reaching_parent_cone_instances_through_parent_flops > 0`
  - `hierarchy_outputs_reaching_parent_cone_instances_through_parent_flops > 0`
  - `top_parent_cone_instance_flop_output_fraction > 0.0`
  - `hierarchy_parent_cone_instance_flop_output_fraction > 0.0`
  - `child_input_bindings_from_parent_cone_instances = 0`
  This route is now banked in the full `r30` Phase 4 matrix through the
  dedicated `phase4_hier2_inst4_parent_output_cone_instance_state`
  axis.
- The focused proof for the new parent-cone helper budget slice is
  `cargo test hierarchy_parent_cone_helper_budget_allows_multiple_helpers`.
  Its key design metrics are:
  - `top_parent_cone_instances = 3`
  - `max_parent_cone_instances_per_internal_module = 3`
  - `child_input_bindings_from_parent_cone_instances > 0`
  This route is now banked in the full `r30` Phase 4 matrix through the
  dedicated `phase4_hier2_inst4_parent_cone_instance_budget3` axis.
- The focused proof for budgeted parent-output helper composition is
  `cargo test hierarchy_parent_outputs_can_spend_helper_budget`. Its
  key design metrics are:
  - `top_parent_cone_instances = 3`
  - `max_parent_cone_instances_per_internal_module = 3`
  - `child_input_bindings_from_parent_cone_instances = 0`
  - `top_outputs_reaching_parent_cone_instances >= 3`
  This focused regression proves the output-helper-only path can spend
  the helper budget without relying on child-input helper bindings.
- The focused proof for the registered parent-cone helper route slice is
  `cargo test hierarchy_registered_child_input_cones_can_use_helper_instances`.
  Its key design metrics are:
  - `child_input_bindings_from_registered_parent_cone_instances > 0`
  - `top_child_input_bindings_from_registered_parent_cone_instances > 0`
  - `registered_parent_cone_instance_child_input_binding_fraction > 0.0`
  - `top_registered_parent_cone_instance_child_input_binding_fraction > 0.0`
  This route is now banked in the full `r30` Phase 4 matrix through the
  dedicated `phase4_hier2_inst4_registered_parent_cone_instance_state`
  axis.
- The focused proof for direct registered sibling helper routing is
  `cargo test hierarchy_registered_sibling_routes_can_use_helper_instances`.
  Its key design metrics are:
  - `top_parent_cone_instances > 0`
  - `child_input_bindings_from_registered_instance_outputs > 0`
  - `child_input_bindings_from_registered_parent_composed_logic = 0`
  - `child_input_bindings_from_registered_parent_cone_instances > 0`
  - `registered_parent_cone_instance_child_input_binding_fraction > 0.0`
  - `num_instances > planned_child_instances`
  This route is now banked in the full downstream-clean `r28` Phase 4
  matrix through the dedicated direct registered sibling helper
  scenario.
- The focused proof for multi-stage direct registered sibling routing
  is
  `cargo test hierarchy_registered_sibling_routes_can_chain_through_parent_flops`.
  Its key design metrics are:
  - `child_input_bindings_from_registered_instance_outputs > 0`
  - `child_input_bindings_from_registered_multistage_instance_outputs > 0`
  - `top_child_input_bindings_from_registered_multistage_instance_outputs > 0`
  - `child_input_bindings_from_registered_parent_composed_logic = 0`
  - `registered_multistage_instance_output_child_input_binding_fraction > 0.0`
  This route is banked in the full downstream-clean `r30` Phase 4
  matrix through the dedicated
  `phase4_hier2_inst4_registered_sibling_multistage_state` scenario.
- The focused proof for multi-stage direct registered sibling helper
  routing is
  `cargo test hierarchy_registered_sibling_routes_can_chain_helper_instances_through_parent_flops`.
  Its key design metrics are:
  - `child_input_bindings_from_registered_multistage_parent_cone_instances > 0`
  - `top_child_input_bindings_from_registered_multistage_parent_cone_instances > 0`
  - `registered_multistage_parent_cone_instance_child_input_binding_fraction > 0.0`
  - `top_registered_multistage_parent_cone_instance_child_input_binding_fraction > 0.0`
  - `child_input_bindings_from_registered_parent_composed_logic = 0`
  - `child_input_bindings_from_registered_multistage_parent_composed_logic = 0`
  This route is banked in the full downstream-clean `r28` Phase 4
  matrix through the dedicated
  `phase4_hier2_inst4_registered_sibling_parent_cone_instance_multistage_state`
  scenario.
- The refreshed repo-owned Phase 4 hierarchy closure report `/tmp/anvil-tool-matrix-phase4-hierarchy-r52/tool_matrix_report.json` is the latest full downstream-clean bank: **105 scenarios**, **4 designs/scenario**, **420 total designs**, `artifact_kind = "design"`, `coverage_gaps = []`, **420/0** pass-fail in Verilator plus both repo-owned Yosys modes, `saw_recursive_hierarchy_parent_cone_instance_mixed_support_outputs = true`, `saw_recursive_hierarchy_registered_parent_cone_instance_mixed_support_routing = true`, `saw_recursive_hierarchy_registered_multistage_mixed_support_routing = true`, `saw_recursive_hierarchy_registered_multistage_sibling_routing = true`, `saw_recursive_hierarchy_registered_multistage_routing = true`, `saw_recursive_hierarchy_registered_mixed_support_routing = true`, `saw_hierarchy_registered_mixed_support_routing = true`, `saw_hierarchy_registered_sibling_mixed_support_routing = true`, `saw_recursive_hierarchy_registered_sibling_mixed_support_routing = true`, `saw_recursive_multiple_parent_cone_instances_per_parent_child_inputs = true`, `saw_recursive_multiple_parent_cone_instances_per_parent_through_flops = true`, `saw_recursive_multiple_parent_cone_instances_per_parent = true`, `saw_multiple_parent_cone_instances_per_parent = true`, `saw_recursive_hierarchy_parent_cone_instance_flop_outputs = true`, `saw_recursive_hierarchy_parent_cone_instance_outputs = true`, `saw_recursive_hierarchy_direct_sibling_parent_cone_instance_routing = true`, `saw_recursive_hierarchy_direct_registered_sibling_parent_cone_instance_routing = true`, `saw_recursive_hierarchy_registered_multistage_parent_cone_instance_routing = true`, `saw_recursive_hierarchy_registered_multistage_parent_composed_parent_cone_instance_routing = true`, `saw_recursive_hierarchy_registered_parent_composed_parent_cone_instance_routing = true`, and `saw_recursive_hierarchy_parent_composed_parent_cone_instance_flop_routing = true`. The failed `r32` full attempt is root-cause evidence for the CaseMux/Casez exact-selector shift-cleanup warning; `r33` is the pre-compact-normalization direct-helper bank, `r34` is the previous recursive direct-helper full bank, `r35` is the previous recursive direct registered-helper full bank, `r36` is the previous recursive registered parent-composed helper full bank, `r37` is the previous recursive multi-stage direct registered-helper full bank, `r38` is the previous recursive multi-stage registered parent-composed helper full bank, `r39` is the previous recursive non-top parent-output helper full bank, `r40` is the previous recursive non-top stateful parent-output helper full bank, `r41` is the previous recursive non-top parent-output multi-helper budget full bank, `r42` is the previous recursive non-top stateful multi-helper budget full bank, `r43` is the previous recursive non-top child-input multi-helper budget full bank, `r44` is the previous recursive non-top registered mixed-support full bank, `r45` is the previous recursive non-top registered parent-composed multistage no-helper full bank, `r46` is the previous recursive non-top registered sibling multistage no-helper full bank, `r47` is the previous recursive non-top registered multistage mixed-support no-helper full bank, `r48` is the previous recursive non-top registered parent-composed helper mixed-support full bank, `r49` is the previous recursive non-top parent-output helper mixed-support full bank, `r50` is the previous accumulated mixed-support hierarchy full bank, and `r51` is the previous direct registered sibling mixed-support full bank.
- The clean pre-fix `/tmp/anvil-tool-matrix-phase4-hierarchy-r22/tool_matrix_report.json` is root-cause evidence only: the stale total-design budget ran 42 scenarios at 3 designs/scenario, or 126 total designs. The live gate now uses a direct four-designs-per-scenario Phase 4 floor.
- That refreshed report covers the current representative hierarchy surface rather than only the older wrapper baseline. Its saved coverage facts include:
  - `hierarchy_depths = ["1", "2", "2:3"]`
  - `hierarchy_leaf_module_counts = ["0", "2", "4"]`
  - `hierarchy_child_instance_counts = ["1:3", "2", "2:3", "4"]`
  - `hierarchy_child_instance_override_profiles = ["0=4:4,1=2:2"]`
  - `hierarchy_child_source_modes = ["library", "on-demand"]`
  - `saw_recursive_hierarchy = true`
  - `saw_per_depth_branching_metrics = true`
  - `saw_hierarchy_registered_sibling_routing = true`
  - `saw_hierarchy_registered_parent_composed_routing = true`
  - `saw_mixed_leaf_depth_hierarchy = true`
  - `saw_hierarchy_parent_composition = true`
  - `saw_hierarchy_sibling_routing = true`
  - `saw_hierarchy_parent_composed_child_inputs = true`
  - `saw_hierarchy_parent_local_flops = true`
  - `saw_hierarchy_parent_port_composed_outputs = true`
  - `saw_hierarchy_registered_mixed_support_routing = true`
  - `saw_hierarchy_registered_sibling_mixed_support_routing = true`
  - `saw_recursive_hierarchy_registered_sibling_mixed_support_routing = true`
  - `saw_hierarchy_registered_multistage_routing = true`
  - `saw_hierarchy_registered_multistage_sibling_routing = true`
  - `saw_recursive_hierarchy_registered_multistage_sibling_routing = true`
  - `saw_hierarchy_registered_multistage_parent_cone_instance_routing = true`
  - `saw_recursive_hierarchy_registered_multistage_parent_cone_instance_routing = true`
  - `saw_hierarchy_registered_multistage_parent_composed_parent_cone_instance_routing = true`
  - `saw_recursive_hierarchy_registered_multistage_parent_composed_parent_cone_instance_routing = true`
  - `saw_hierarchy_parent_composed_parent_cone_instance_flop_routing = true`
  - `saw_hierarchy_parent_cone_instance_routing = true`
  - `saw_hierarchy_parent_cone_instance_outputs = true`
  - `saw_recursive_hierarchy_parent_cone_instance_outputs = true`
  - `saw_recursive_hierarchy_parent_cone_instance_mixed_support_outputs = true`
  - `saw_hierarchy_parent_cone_instance_flop_outputs = true`
  - `saw_recursive_hierarchy_parent_cone_instance_flop_outputs = true`
  - `saw_recursive_multiple_parent_cone_instances_per_parent = true`
  - `saw_recursive_multiple_parent_cone_instances_per_parent_child_inputs = true`
  - `saw_recursive_multiple_parent_cone_instances_per_parent_through_flops = true`
  - `saw_multiple_parent_cone_instances_per_parent = true`
  - `saw_hierarchy_registered_parent_cone_instance_routing = true`
  - `saw_hierarchy_direct_sibling_parent_cone_instance_routing = true`
  - `saw_hierarchy_direct_registered_sibling_parent_cone_instance_routing = true`
  - `saw_on_demand_child_sourcing = true`
  - `saw_profiled_child_interface_synthesis = true`
  - `saw_reused_child_definition = true`
  - `saw_underinstantiated_library = true`
- Earlier current-code coverage-only Phase 4 matrix probes at
  `/tmp/anvil-tool-matrix-phase4-parent-port-coverage-r1/tool_matrix_report.json`
  first proved the refreshed gate policy also requires
  `saw_hierarchy_parent_port_composed_outputs = true`, with
  `coverage_gaps = []`.
  `/tmp/anvil-tool-matrix-phase4-registered-mixed-r1/tool_matrix_report.json`
  first proved the refreshed gate policy also requires
  `saw_hierarchy_registered_mixed_support_routing = true`, with
  `coverage_gaps = []`.
  `/tmp/anvil-tool-matrix-phase4-registered-multistage-r1/tool_matrix_report.json`
  first proved the refreshed gate policy also requires
  `saw_hierarchy_registered_multistage_routing = true`, with
  `coverage_gaps = []`.
  `/tmp/anvil-tool-matrix-phase4-parent-cone-instance-r1/tool_matrix_report.json`
  first proved the refreshed gate policy also requires
  `saw_hierarchy_parent_cone_instance_routing = true`, with
  `coverage_gaps = []`.
  `/tmp/anvil-tool-matrix-phase4-parent-output-helper-state-r3/tool_matrix_report.json`
  first proved the refreshed gate policy also requires
  `saw_hierarchy_parent_cone_instance_flop_outputs = true`, with
  `coverage_gaps = []`. Those probes were run with `--skip-verilator
  --skip-yosys`; the full downstream-clean `r30` bank now carries those
  facts plus the newer helper-output, budgeted-helper, registered
  helper-route, stateful parent-output helper, direct registered helper
  chain, and registered parent-composed helper chain facts with real
  tool validation.
- The older `/tmp/anvil-tool-matrix-phase4-hierarchy-r18` report now
  remains useful historical evidence for the first
  registered-parent-composed route bank,
  `/tmp/anvil-tool-matrix-phase4-hierarchy-r19` remains useful
  historical evidence for the pre-full parent-port / registered-mixed /
  multi-stage bank,
  `/tmp/anvil-tool-matrix-phase4-hierarchy-r17` remains useful
  historical evidence for the pre-registered-parent-composed route
  hierarchy bank, `/tmp/anvil-tool-matrix-phase4-hierarchy-r16`
  remains useful historical evidence for the first
  registered sibling-route bank, `/tmp/anvil-tool-matrix-phase4-hierarchy-r15` report now
  remains useful historical evidence for the pre-parent-state
  hierarchy bank, `/tmp/anvil-tool-matrix-phase4-hierarchy-r13` report now
  remains useful historical evidence for the first sibling-routing
  bank, `/tmp/anvil-tool-matrix-phase4-hierarchy-r12` report now
  remains useful historical evidence for the first exact profiled
  child-interface bank, `/tmp/anvil-tool-matrix-phase4-hierarchy-r11` now
  remains useful historical evidence for the first explicit
  child-sourcing bank, `/tmp/anvil-tool-matrix-phase4-hierarchy-r10`
  is the pre-on-demand mixed-depth bank, `/tmp/anvil-tool-matrix-phase4-hierarchy-r9`
  is the first mixed-depth recursive bank, `/tmp/anvil-tool-matrix-phase4-hierarchy-r21`
  is the historical pre-parent-output-helper full bank,
  `/tmp/anvil-tool-matrix-phase4-hierarchy-r22` is the clean but
  insufficient 126-design pre-fix budget-mismatch run, and
  `/tmp/anvil-tool-matrix-phase4-hierarchy-r6` remains historical
  debugging evidence only.
- The durable runtime lesson from the broadened rerun is now explicit: the hierarchy gate should prove hierarchy structure with a hierarchy-focused sequential leaf profile, not silently borrow the fattest Phase 1 sequential leaf stress shape. The aborted `/tmp/anvil-tool-matrix-phase4-hierarchy-r8` rerun showed the cost of that coupling; the banked `r10` report closes the same hierarchy surface cleanly after decoupling it, and the gate budget was also explicitly raised from 48 to 60 total designs to preserve 4 designs/scenario after the scenario set grew from 15 to 18 entries. The newer `r22` mismatch repeated the same lesson at 42 scenarios: a stale 120-design total floor silently reduced Phase 4 to 3 designs/scenario. `src/bin/tool_matrix.rs` now encodes the policy as a direct 4-designs/scenario Phase 4 floor.
- The repo-owned `tool_matrix` smoke lane is still green (15/15 clean in Verilator and 15/15 clean in Yosys with warnings treated as failures), the Yosys axis is explicit and clean in both repo-owned sub-modes (`without-abc = 15/15 pass`, `with-abc = 15/15 pass` on the small `--yosys-mode both` probe), the 1000-module Phase 1 gate is closed for real on current code, the dedicated Phase 2 sharing gate is closed, and the dedicated Phase 3 structured-surface gate is closed at `/tmp/anvil-tool-matrix-phase3-structured-r4` with **210** completed checkpoints / **210** emitted `.sv` files, `coverage_gaps = []`, and `210/0` pass-fail in Verilator plus both repo-owned Yosys modes.
- The strongest no-ABC real frontier still stands at **365** generated modules with **0** Verilator warning logs and **0** Yosys warning lines. The older current-code both-mode `r18` frontier still stands as historical evidence at **372** completed checkpoints / **373** emitted `.sv` files, and the later `/tmp/anvil-tool-matrix-phase1-real-r20` frontier still stands as historical evidence at **570** completed checkpoints / **571** emitted `.sv` files spanning full clean closure through `peephole` plus 34 `e-graph` modules.
- The completed current-code both-mode Phase 1 tree is `/tmp/anvil-tool-matrix-phase1-real-r21`, fully closed at **1005** completed checkpoints / **1005** emitted `.sv` files with zero warning artifacts and `1005/0` pass/fail in Verilator plus both repo-owned Yosys modes.
- The completed Phase 2 share-sweep tree is `/tmp/anvil-tool-matrix-phase2-share-r1`, fully closed at **216** completed checkpoints / **216** emitted `.sv` files with `coverage_gaps = []` and `216/0` pass/fail in Verilator plus both repo-owned Yosys modes. Its `share_sweep` summary proves the sharing knob is controlling the landed graphs in the right direction: `shared_node_fraction` rises monotonically across the representative sweep (`0.4122 @ share_prob=0.0`, `0.4232 @ 0.3`, `0.4386 @ 0.9`) while `avg_nodes/module` drops (`4727.56`, `3525.01`, `2117.76`).
- The crate MSRV is explicitly pinned to **Rust 1.95** via `Cargo.toml` `rust-version = "1.95"`.
- `tool_matrix` now writes both per-module and per-design checkpoint sidecars and supports `--resume`, including legacy bootstrap paths for older output trees that only have emitted `.sv` artifacts. Fresh checkpoints carry generator state, emitted-file hashes, and a runtime fingerprint, so same-binary resume can skip replaying already-proven artifacts while still checking file integrity.
- The completed `r21` tree has already gone through that one-time replay-and-upgrade pass too: all **1005** saved checkpoints carry the new metadata, and the completed Phase 2 `r1` tree was also successfully re-walked under `--resume` to refresh its report under the corrected normalized share metric.
- `src/gen/cone.rs` still bounds exact small-value-set reasoning with a shared work budget, memoized unknown results, and a small-support gate (current cap: **3** canonical leaf endpoints), which keeps generator-side exact proofs live on the intended narrow cones without letting larger shared structures soak generator time.
- `src/ir/compact.rs` now applies the same "small support is not enough by itself" lesson to post-construction semantic merging too: large settled cones with tiny leaf support no longer trigger an unbounded semantic truth-table proof in `merge_equivalent_gates`; once the reachable cone exceeds the merge budget, compaction falls back cleanly to the structural proof path. Cleanup remains stricter still (width <= 8, support <= 10 bits, <= 3 canonical leaf endpoints), while its cheap warning-oriented revisit paths for unsigned compares and bounds-provable shifts stay live.
- The docs and book still say the NodeId doctrine plainly and consistently: `identity_mode = node-id` means full factorization by definition, `relaxed` is the only intentional semantic off-switch, and `factorization_level` is the current-build enforcement/proof-depth dial inside `node-id`, not an alternate definition of it.
- The roadmap still carries new not-started artifact-family phases beyond the current RTL lanes: parameterization, aggregates, advanced motifs, oracle-backed micro-designs, frontend/elaboration accept corpora, and a future multi-artifact umbrella.
- **Last completed slice:** Proved recursive non-top direct registered sibling mixed-support routing and banked it through full
  downstream tools at
  `/tmp/anvil-tool-matrix-phase4-hierarchy-r52/tool_matrix_report.json`
  (`420` designs, `coverage_gaps = []`, `420/0` in Verilator plus both
  repo-owned Yosys modes). This covers the same direct registered sibling D path below the top parent in exact-depth-2 hierarchy by requiring hierarchy-wide mixed-support counters to exceed top-only counters while helper and registered parent-composed counters remain zero, and carries forward the old `r23`/`r24` evidence
  split, the `r25` direct-helper full bank, the `r26` multi-stage
  registered sibling bank, the `r27` stateful parent-output helper bank,
  the `r28` multi-stage direct registered sibling helper bank, the
  `r29` multi-stage registered parent-composed helper bank, the `r31`
  recursive helper-state bank, the `r36` recursive registered
  parent-composed helper bank, and the `r37` recursive multi-stage
  direct registered helper bank, the `r38` recursive multi-stage
  registered parent-composed helper bank, the `r39` recursive non-top
  parent-output helper bank, the `r40` recursive non-top stateful
  parent-output helper bank, the `r41` recursive non-top parent-output
  multi-helper budget bank, and the `r42` recursive non-top stateful
  multi-helper budget bank, and the `r43` recursive non-top
  child-input multi-helper budget bank, the `r44` recursive registered
  mixed-support bank, and the `r45` recursive registered
  parent-composed multistage no-helper bank, and the `r46` recursive
  registered sibling multistage no-helper bank, and the `r47`
  recursive registered multistage mixed-support no-helper bank, and the
  `r48` recursive registered parent-composed helper mixed-support bank.
- **Prior slice:** Broadened direct registered sibling routing so later
  child-input routes can chain through earlier parent-local Qs without
  becoming registered parent-composed logic.
- **Prior slice:** Banked the direct-helper Phase 4 hierarchy matrix
  through full downstream tools at
  `/tmp/anvil-tool-matrix-phase4-hierarchy-r25/tool_matrix_report.json`
  (48 scenarios / 192 designs, `coverage_gaps = []`, `192/0` in
  Verilator plus both repo-owned Yosys modes).
- **Prior slice:** Broadened direct sibling routing so a
  helper instance can source a later child input directly when
  `hierarchy_sibling_route_prob` and
  `hierarchy_parent_cone_instance_prob` are both active. Focused proof:
  `cargo test hierarchy_sibling_routes_can_use_helper_instances`.
- **Prior slice:** Broadened direct registered sibling routing so a
  helper instance can source the parent-flop D side when
  `hierarchy_registered_sibling_route_prob` and
  `hierarchy_parent_cone_instance_prob` are both active. Metrics now
  count registered helper use by inspecting the registered flop-D
  dependencies, so the direct route is measured without pretending it is
  registered parent-composed D logic. Focused proof:
  `cargo test hierarchy_registered_sibling_routes_can_use_helper_instances`.
- **Prior slice:** Broadened parent-output helper placement so
  parent-output-only composition can allocate and spend multiple
  parent-cone helper instances up to the configured per-parent budget.
  Focused proof: `cargo test hierarchy_parent_outputs_can_spend_helper_budget`.
- **Earlier slice:** Aligned the crate package metadata with the
  accepted ANVIL purpose terminology. `Cargo.toml` no longer describes
  ANVIL as constrained-random; it now says ANVIL is a random
  by-construction generator of synthesizable SystemVerilog RTL. The
  terminology doctrine in `DEVELOPMENT_NOTES.md`, the workspace summary
  in `CODEBASE_ANALYSIS.md`, and this handoff now treat package
  metadata as part of the live terminology surface.
- **Prior slice:** Refreshed the Phase 4 hierarchy gate at full
  per-scenario depth. `src/bin/tool_matrix.rs` now uses a direct
  four-designs-per-scenario Phase 4 floor. The previous full bank at
  `/tmp/anvil-tool-matrix-phase4-hierarchy-r26/tool_matrix_report.json`
  proves 51 scenarios / 204 designs with `coverage_gaps = []` and
  204/0 Verilator/Yosys pass-fail in all repo-owned lanes; the older
  `r23` report is the historical pre-direct-helper full bank.
- **Earlier slice:** Added explicit parent-cone helper budgeting.
  `max_parent_cone_instances_per_module` defaults to `1`, `0` disables
  helper allocation, higher values allow multiple helper children in one
  hierarchy parent, `DesignMetrics` records
  `max_parent_cone_instances_per_internal_module`, `tool_matrix` has a
  dedicated budget-3 Phase 4 scenario/coverage fact, and the mdBook/live
  docs describe the new knob, metrics, and recipe.
- **Earlier slice:** Broadened parent-cone helper instances beyond
  child-input cones into parent-output composition. The helper
  insertion path can now source parent outputs independently of
  `hierarchy_child_input_cone_prob`, `DesignMetrics` records
  helper-output support for top and hierarchy outputs, and
  `tool_matrix` has a dedicated Phase 4 scenario/coverage fact now
  banked in the full `r27` gate.
- **Earlier slice:** Landed multi-stage registered parent-composed
  hierarchy child-input routing. Later
  `hierarchy_registered_child_input_cone_prob` routes now add an earlier
  parent-flop companion when one is available, giving the
  parent-composed D cone a prior-Q leg before the final parent flop.
  Metrics now expose
  `child_input_bindings_from_registered_multistage_parent_composed_logic`
  and matching top / fraction fields. Focused proof:
  `/tmp/anvil-hier-registered-multistage-child-input-smoke-r1/manifest.json`.
  The earlier coverage-only proof remains
  `/tmp/anvil-tool-matrix-phase4-registered-multistage-r1/tool_matrix_report.json`
  with `coverage_gaps = []`, now superseded by the full `r27` bank for
  downstream-clean evidence.
- **Prior slice:** Landed registered mixed-support hierarchy
  child-input routing. `hierarchy_registered_child_input_cone_prob`
  now builds D cones from the full parent source pool, then repairs the
  route so it keeps sibling-output support and can add parent-port
  support. Metrics now expose
  `child_input_bindings_from_registered_mixed_support` and matching top
  / fraction fields. Focused proof:
  `/tmp/anvil-hier-registered-mixed-child-input-smoke-r1/manifest.json`.
  Coverage proof:
  `/tmp/anvil-tool-matrix-phase4-registered-mixed-r1/tool_matrix_report.json`
  with `coverage_gaps = []`.
- **Prior slice:** Banked mixed parent-port / child-output
  parent-output coverage in the Phase 4 hierarchy matrix. `tool_matrix`
  now tracks `saw_hierarchy_parent_port_composed_outputs` and the
  Phase 4 coverage gate rejects representative matrices that never emit
  parent outputs mixing parent ports with child outputs. The
  coverage-only proof is
  `/tmp/anvil-tool-matrix-phase4-parent-port-coverage-r1/tool_matrix_report.json`
  with `coverage_gaps = []`.
- **Prior slice:** Landed mixed parent-port / child-output
  hierarchy parent outputs. Parent output cones now build from the full
  parent source pool and receive a post-finalization repair that keeps
  child-output support live while adding parent-port support when live
  parent data inputs exist. Metrics now distinguish
  `top_parent_port_composed_outputs` /
  `hierarchy_parent_port_composed_outputs` and matching fractions. The
  focused regression is
  `cargo test --test pipeline hierarchy_parent_outputs_can_mix_parent_ports_with_child_outputs`,
  and the focused downstream proof is
  `/tmp/anvil-hier-parent-output-mix-smoke-r1/manifest.json`.
- **Prior slice:** Landed registered parent-composed child-input
  routing and refreshed the repo-owned Phase 4 gate. See `CHANGES.md`
  entry `2026-04-24-boot4`. The current hierarchy planner now has a
  real route via `hierarchy_registered_child_input_cone_prob` where a
  later child input is driven by parent-local logic and then one local
  parent flop. That route initially used sibling-output-derived
  sources; the newer registered mixed-support slice broadens it to the
  full parent source pool and repairs in parent-port and sibling-output
  support when both are live.
  Metrics now distinguish this from direct registered sibling routing,
  the focused proof artifact is
  `/tmp/anvil-hier-registered-child-input-cone-smoke-r2/manifest.json`,
  and the refreshed repo-owned Phase 4 hierarchy report that first
  banked the route was
  `/tmp/anvil-tool-matrix-phase4-hierarchy-r21/tool_matrix_report.json`.
  The current full `r55` gate supersedes that historical report with
  `coverage_gaps = []`, `456/0` pass-fail in Verilator plus both
  repo-owned Yosys modes, and saved coverage facts including
  `saw_hierarchy_registered_parent_composed_routing = true`.
- **Prior slice:** Landed registered sibling routing through
  parent-local state and refreshed the repo-owned Phase 4 gate. See
  `CHANGES.md` entry `2026-04-24-boot3`. The current hierarchy planner
  now has a real registered child-to-child route via
  `hierarchy_registered_sibling_route_prob`, metrics for registered
  instance-output child-input bindings, a focused proof artifact
  `/tmp/anvil-hier-registered-sibling-smoke-r1/manifest.json`, and the
  refreshed repo-owned Phase 4 hierarchy report at
  `/tmp/anvil-tool-matrix-phase4-hierarchy-r17/tool_matrix_report.json`
  with `coverage_gaps = []`, `108/0` pass-fail in Verilator plus both
  repo-owned Yosys modes, and saved coverage facts including
  `saw_hierarchy_registered_sibling_routing = true`.
- **Prior slice:** Clarified hierarchy parent wording after
  executing the README/bootstrap path. See `CHANGES.md` entry
  `2026-04-24-boot2`. No source behavior changed; the docs now avoid
  wrapper-only and combinational-only phrasing for parent-side hierarchy
  cones and control-port propagation.
- **Prior slice:** Landed local parent state in hierarchy
  parent-side cones and refreshed the repo-owned Phase 4 gate. See
  `CHANGES.md` entry `2026-04-24-boot1`. The current hierarchy planner
  now has a real parent-state surface via
  `hierarchy_parent_flop_prob`, trustworthy parent-flop metrics, a
  focused proof artifact
  `/tmp/anvil-hier-parent-state-smoke-r1/manifest.json`, and the
  refreshed repo-owned Phase 4 hierarchy report at
  `/tmp/anvil-tool-matrix-phase4-hierarchy-r16/tool_matrix_report.json`
  with `coverage_gaps = []`, `96/0` pass-fail in Verilator plus both
  repo-owned Yosys modes, and saved coverage facts including
  `saw_hierarchy_parent_local_flops = true`.
- **Prior slice:** Landed parent-composed hierarchy child-input
  bindings and refreshed the repo-owned Phase 4 gate. See `CHANGES.md`
  entry `2026-04-23-boot9`.
- **Prior slice:** Landed sibling-routed hierarchy child inputs and
  refreshed the repo-owned Phase 4 gate. See `CHANGES.md` entry
  `2026-04-23-boot7`.
- **Prior slice:** Landed exact profiled on-demand child synthesis and
  refreshed the repo-owned Phase 4 gate. See `CHANGES.md` entry
  `2026-04-23-boot6`.
- **Prior slice:** Landed explicit hierarchy child sourcing and
  refreshed the repo-owned Phase 4 gate. See `CHANGES.md` entry
  `2026-04-23-boot5`.
- **Prior slice:** Closed the mixed-depth Phase 4 hierarchy gate cleanly. See
  `CHANGES.md` entry `2026-04-23-boot4`. `tool_matrix` now banks the mixed-depth
  recursive hierarchy axis in the repo-owned Phase 4 matrix instead of leaving
  it as focused-smoke-only evidence. The refreshed report at
  `/tmp/anvil-tool-matrix-phase4-hierarchy-r10/tool_matrix_report.json` closes
  at 18 scenarios / 72 designs with `coverage_gaps = []`, `72/0` pass-fail in
  Verilator plus both repo-owned Yosys modes, and saved coverage facts
  including `hierarchy_depths = ["1", "2", "2:3"]` and
  `saw_mixed_leaf_depth_hierarchy = true`.
- **Prior slice:** Landed mixed-depth recursive hierarchy planning. See
  `CHANGES.md` entry `2026-04-23-boot3`. Bounded recursive hierarchy no longer
  collapses the requested depth interval to one exact realized scalar for the
  whole design. The planner now carries subtree-local depth intervals, can
  realize both shallow and deep branches inside one legal tree, and exposes
  that fact numerically via `leaf_module_occurrences_by_depth`. Validation was
  the focused smoke at `/tmp/anvil-hier-mixed-depth-smoke-r1/manifest.json`,
  new unit/integration regressions, and downstream-clean runs in Verilator plus
  both repo-owned Yosys modes.
- **Prior slice:** Completed a literal `SESSION_BOOTSTRAP.md` rerun and
  corrected the one stale live-doc claim it exposed. See `CHANGES.md` entry
  `2026-04-23-boot2`. The bootstrap now genuinely re-reads the live docs, the
  full mdBook, and the Rust workspace against current HEAD; the only drift
  found was stale README wording that still implied the refreshed recursive
  Phase 4 closure refresh was pending. That sentence now matches repo reality:
  the recursive Phase 4 matrix is already banked, and the next work is deeper
  hierarchy capability rather than closure refresh.
- **Prior slice:** Landed bounded recursive hierarchy depth profiles and per-depth metrics. See `CHANGES.md` entry `2026-04-23-1735`. Recursive hierarchy now keeps the global child-instance fallback range and adds repeated `child_instances_per_depth` overrides keyed by parent depth; the recursive planner now consults those overrides while building the tree; and `DesignMetrics` now report `avg/min/max_child_instances_by_parent_depth` so the realized shape can be trusted from manifests. Validation was the focused smoke at `/tmp/anvil-hier-depth-profile-smoke-r1/manifest.json`, new unit / integration regressions, and downstream clean runs in Verilator plus both repo-owned Yosys modes.
- **Prior slice:** Landed parent-composed hierarchy tops and trustworthy composition metrics. See `CHANGES.md` entry `2026-04-23-1557`. `DepSet` now tracks typed leaf identities including child instance outputs; `Node::InstanceOutput` is now a real dep-bearing parent-side leaf; the depth-1 top now builds a first combinational parent-output layer over child outputs; `compact_node_ids` now treats and remaps instance input bindings correctly; validation now allows genuinely unused child outputs; the SV emitter now renders unused child outputs as `.port()`; and `DesignMetrics` now quantify direct-vs-composed top outputs plus child-output dependency/support. Validation was the focused smoke at `/tmp/anvil-hier-parent-compose-smoke-r1/manifest.json`, new unit/integration regressions, and the full hygiene gate.
- **Prior slice:** Landed trustworthy hierarchy design metrics and tightened the control-port doctrine. See `CHANGES.md` entry `2026-04-23-0210`. `src/metrics.rs` now exposes `DesignMetrics`; `src/main.rs` and `src/bin/tool_matrix.rs` now embed those per-design metrics in hierarchy manifests and design reports; `src/gen/hierarchy.rs` now tags wrapper shared `clk` / `rst_n` as `Module.clock` / `Module.reset`; and `src/ir/types.rs`, `src/ir/validate.rs`, `src/metrics.rs`, and `src/emit/sv.rs` now agree on the exact boundary rule: pure comb-only modules omit `clk` / `rst_n`, while wrappers keep them visible iff they carry sequential descendants. Validation was the focused `/tmp/anvil-hier-metrics-smoke-r1` downstream proof, new IR/emitter regression tests for both the comb-only and grandparent-wrapper cases, plus the full hygiene gate.
- **Prior slice:** Landed the bounded procedural for-fold surface. See `CHANGES.md` entry `2026-04-22-2219`. The leaf lane now has a real statically bounded unrolled-logic surface via a new `for_fold_prob` knob, `GateOp::ForFold { kind, trip_count, chunk_width }`, emitter support, validator support, exact-evaluator support, metrics / matrix coverage plumbing, and strategy-spanning tests. The proof harness now also pins a real smoke emission at `/tmp/anvil-forfold-smoke-r1/mod_1_0000.sv`, which contains live `always_comb` `for (int i = 0; i < ...)` blocks on current HEAD. Landing this surface also exposed and fixed a latent width-domain bug in `pick_priority_encoder_n`: recursive wider packed-source builds could ask that helper about widths above 32, so it now rejects `target_width > 32` explicitly instead of overflowing. Validation was the full hygiene gate.
- **Prior slice:** Landed the procedural combinational casez-mux block. See `CHANGES.md` entry `2026-04-22-2315`. The leaf lane now has a real `always_comb casez (sel)` surface via a new `casez_mux_prob` knob, `GateOp::CasezMux`, emitter support, validator support, exact-evaluator support, metrics / matrix coverage plumbing, and strategy-spanning tests. The generated wildcard patterns are non-overlapping by construction, so the new surface stays a wildcarded mux motif rather than becoming an accidental priority chain. `ROADMAP.md`, `CODEBASE_ANALYSIS.md`, and this handoff file now treat both `case` and `casez` as landed; the remaining obvious Phase 3 breadth gap was statically bounded unrolled logic, which is now also landed in the newer slice above.
- **Prior slice:** Landed the procedural combinational case-mux block and tightened the late settled-graph cleanup. See `CHANGES.md` entry `2026-04-22-2210`. The leaf lane now has a real `always_comb case (sel)` surface via a new `case_mux_prob` knob, `GateOp::CaseMux`, emitter support, validator support, metrics/case coverage plumbing, and strategy-spanning tests; current HEAD also has a dedicated post-construction `fold_mixed_associative_constants` pass so strict duplicate-free `Add` / `Mul` output survives the remap-heavy cleanup stages. The variable-shift regression was also tightened from one fixed seed to a 32-seed sweep so it proves the surface rather than one particular RNG path. Validation was the full hygiene gate.
- **Prior slice:** Pinned the crate MSRV to Rust 1.95. See `CHANGES.md` entry `2026-04-22-2048`. `Cargo.toml` now declares `rust-version = "1.95"`, and the stale "MSRV not yet pinned" note in `CODEBASE_ANALYSIS.md` is gone. Validation was the full hygiene gate.
- **Prior slice:** Banked 40 clean modules in the sequential motif-heavy `e-graph` lane on `r21`. See `CHANGES.md` entry `2026-04-22-1458`. No Rust source changed in this slice; the work was the real resumed frontier run at `/tmp/anvil-tool-matrix-phase1-real-r21`. The tree is now at **710** completed checkpoints / **710** emitted `.sv` files with zero warning artifacts, including **40** clean checkpoints in `seq_nodeid_egraph_motif_heavy_seq`. Validation was that resumed `tool_matrix --phase1-gate --yosys-mode both --resume` run plus the full hygiene gate.
- **Prior slice:** Silenced the associative Yosys `shiftadd` warning by strengthening generator-side overshift proofs. See `CHANGES.md` entry `2026-04-21-0110`. `src/gen/cone.rs` now lets shifts use a tiny-domain rhs fallback for narrow boolean-mask arithmetic, so "always overshift" can still be proven even when the whole cone is too large for the general exact small-set engine. Validation was the new unit test, the focused current-code associative repro (`seed=5 / interleaved / node-id / associative / count=12`) that is now clean in Verilator plus both repo-owned Yosys modes, and a fresh current-code real both-mode frontier at `/tmp/anvil-tool-matrix-phase1-real-r18` intentionally interrupted after 372 completed checkpoints / 373 emitted `.sv` files with zero Verilator warning logs and zero Yosys warning lines.
- **Prior slice:** Capped the post-construction cleanup semantic exact prover to tiny endpoint sets. See `CHANGES.md` entry `2026-04-21-0109`. Sampling the stalled `tool_matrix --phase1-gate --yosys-mode both --resume` run showed the hotspot in `ir::compact::fold_proven_gates` / `semantic_exact_value`, not in Yosys or Verilator. `src/ir/compact.rs` now refuses cleanup-time brute-force proofs unless the cone is <= 8 bits wide, carries at most 10 support bits, and uses no more than 3 canonical leaf endpoints; the negative result is memoized immediately. Validation was the new unit test plus a focused current-code repro (`seed=2 / interleaved / node-id / cse / count=2`) that now emits promptly and stays warning-clean in Verilator, Yosys `synth -noabc`, and the repo-owned ABC path. Because the emitted `.sv` changed, `/tmp/anvil-tool-matrix-phase1-real-r16` is now byte-stale and should not be resumed in place.
- **Prior slice:** Recorded a fresh current-code both-mode frontier through `cse`. See `CHANGES.md` entry `2026-04-21-0104`. This is a docs/evidence slice only: `cargo run --bin tool_matrix -- --out /tmp/anvil-tool-matrix-phase1-real-r12 --phase1-gate --yosys-mode both` was run from a fresh tree and deliberately stopped after 221 completed checkpoints with zero Verilator warning logs and zero Yosys warning lines. The saved frontier covers all 67 relaxed modules, all 67 nodeid-none modules, all 67 nodeid-cse modules, and 20 nodeid-operand-unique modules.
- **Prior slice:** Bounded the exact small-set proof engine to unblock the CSE frontier. See `CHANGES.md` entry `2026-04-21-0103`. `src/gen/cone.rs` now gives exact finite-set reasoning a shared work budget and memoizes both known and unknown results, plus a regression test (`small_value_set_bails_out_before_cartesian_blow_up`). Validation reached the full test suite plus a focused current-code repro: `cargo run --bin anvil -- --seed 2 --count 10 --out /tmp/anvil-cse-seed2-repro-r1 --construction-strategy interleaved --identity-mode node-id --factorization-level cse`, followed by clean Verilator, Yosys `-noabc`, and Yosys `with-abc` sweeps on those 10 modules.
- **Prior slice:** Upgraded the legacy `r11` both-mode frontier into resumable state. See `CHANGES.md` entry `2026-04-21-0102`. This is a docs/evidence slice only: `tool_matrix --phase1-gate --yosys-mode both --resume` was run against `/tmp/anvil-tool-matrix-phase1-real-r11` and deliberately stopped after writing 143 module-report sidecars with zero Verilator warning logs and zero Yosys warning lines. The tree is now resumable in place through all 67 relaxed modules, all 67 nodeid-none modules, and 9 cse modules.
- **Prior slice:** Added resumable per-module checkpoints to `tool_matrix`. See `CHANGES.md` entry `2026-04-21-0101`. `tool_matrix` now writes `<stem>.module-report.json` after each completed module, validates checkpoint reuse against the current tool surface, refreshes metrics locally on resume, and can upgrade older partial trees by validating the saved `.sv` and rerunning the current tool surface once. A real partial both-mode smoke run was interrupted at 14/15 scenarios and then completed successfully on the same output tree under `--resume`.
- **Prior slice:** Advanced the real both-mode frontier to 368 clean modules. See `CHANGES.md` entry `2026-04-21-0100`. This is a docs/evidence slice only: a real `tool_matrix --phase1-gate --yosys-mode both` run was pushed to 368 generated modules with zero Verilator warning logs and zero Yosys warning lines, then deliberately stopped after the full `commutative` rung and 33 clean modules into `associative`. The repo now carries the strongest repo-owned Phase 1 frontier in the stricter both-mode lane, slightly ahead of the older 365-module no-ABC baseline.
- **Prior slice:** Advanced the real both-mode frontier to 288 clean modules. See `CHANGES.md` entry `2026-04-21-0099`. This is a docs/evidence slice only: a real `tool_matrix --phase1-gate --yosys-mode both` run was pushed to 288 generated modules with zero Verilator warning logs and zero Yosys warning lines, then deliberately stopped after the full `operand-unique` rung and 20 clean modules into `commutative`. The repo now carries an actually substantial both-mode industrial checkpoint instead of just the earlier 144-module toe-hold.
- **Prior slice:** Recorded the first clean both-mode Phase 1 frontier. See `CHANGES.md` entry `2026-04-21-0098`. This is a docs/evidence slice only: a real `tool_matrix --phase1-gate --yosys-mode both` run was pushed to 144 generated modules with zero Verilator warning logs and zero Yosys warning lines, then deliberately stopped at the boundary after two full scenarios plus the beginning of the third. The repo now carries two explicit industrial frontiers instead of one: 365 clean modules on the baseline gate and 144 clean modules on the stronger both-mode gate.
- **Prior slice:** Made the ABC-enabled Yosys harness warning-clean. See `CHANGES.md` entry `2026-04-21-0097`. `tool_matrix --yosys-mode with-abc` no longer replays the raw default `synth` script; it now uses the explicit repo-owned path `synth -noabc; abc -fast; opt -fast; stat; check`, which keeps ABC in the loop without reproducing the old `ABC: Warning: The network is combinational` bucket. The report now also records the actual warning line when a tool emits one. Validation reached the full hygiene quartet plus `mdbook build book`, and a fresh `--yosys-mode both` probe is now clean in both Yosys sub-modes.
- **Prior slice:** Advanced the real Phase 1 gate frontier to 365 clean modules. See `CHANGES.md` entry `2026-04-21-0096`. This slice is mostly an evidence checkpoint, plus one tiny hygiene-only source cleanup in `src/gen/cone.rs` to satisfy the repo's mandatory `cargo clippy --all-targets -- -D warnings` bar. The durable result is that the repaired warning bucket now stays clean through five full scenarios and into the associative rung of the real `--phase1-gate` run.
- **Prior slice:** Advanced the real Phase 1 gate frontier to 246 clean modules. See `CHANGES.md` entry `2026-04-20-0095`. This was a docs/evidence slice: no source changes, but the repo now has a durable checkpoint showing that the repaired warning bucket stays clean across multiple identity/factorization lanes in the real `--phase1-gate` run rather than only in the earliest relaxed/default scenario.
- **Prior slice:** Closed wide-slice overshift proof gaps and pruned dead state at finalisation. See `CHANGES.md` entry `2026-04-20-0094`. `src/gen/cone.rs` now keeps narrow `Slice` outputs in the exact-value proof path even when the source cone is wider than the small-set engine's direct domain, `src/ir/compact.rs::compact_node_ids` now removes dead flops whose `Q` is never observed by the live graph, and late proof / semantic-merge remaps are now pruned if they would introduce duplicate operands into strict `Add` / `Mul` gates. Validation reached `cargo test`, `cargo fmt --all --check`, `mdbook build book`, `cargo test --test pipeline zero_duplicate_operands_at_default_knobs`, and a fresh relaxed/none seed-0 repro where `mod_0_0006.sv` is clean in both Verilator and `yosys ... synth -noabc`.
- **Prior slice:** Extended exact proof shortcuts and restored associative normal form after remaps. See `CHANGES.md` entry `2026-04-20-0093`. `src/gen/cone.rs` now short-circuits exact proofs once an absorbing or saturating prefix has already forced the result (reflexive comparisons, duplicate-XOR parity, `Or(..., all_ones, tail)`, `Mul(..., 0, tail)`, etc.), and `src/ir/compact.rs` now has `flatten_posthoc_associative_gates` so remap-producing post-construction passes cannot leave behind legal nested associative shapes. Validation reached `cargo test` + `cargo fmt --all --check`, a fresh relaxed/none seed-0 repro where `mod_0_0013`, `0016`, `0018`, `0026`, and `0030` all lint clean in Verilator, and a real `tool_matrix --phase1-gate` rerun that was manually stopped after 76 generated modules with zero Verilator warning logs in the output tree.
- **Prior slice:** Made the Phase 1 gate first-class in `tool_matrix`. See `CHANGES.md` entry `2026-04-20-0092`. `src/bin/tool_matrix.rs` now has `--phase1-gate`, which auto-enables coverage-gap failure and raises `modules_per_scenario` high enough to generate at least 1000 modules total across the built-in scenario set. The report now records `total_modules` and `phase1_gate`, and the docs now point at the real command instead of hand-waving the arithmetic.
- **Prior slice:** Closed the downstream-warning bucket. See `CHANGES.md` entry `2026-04-20-0091`. `src/ir/compact.rs` now has `fold_proven_gates`, a post-construction proof-cleanup pass that revisits the settled graph, folds provably-exact gates to constants, and rewires constant-selector muxes. `src/gen/cone.rs` now exposes a reusable exact-value proof helper and has stronger shift-range reasoning (`overshift by proved-large range -> 0`). `src/bin/tool_matrix.rs` now treats warnings as failures and uses `yosys ... synth -noabc; stat` so repo-owned green runs mean "no errors, no warnings." On the smoke matrix this moved the repo-owned result from 13/15 Verilator-clean + 15/15 Yosys-clean to 15/15 clean in both tools.
- **Prior slice:** The roadmap, live docs, and book now capture the broadened artifact-family mandate without weakening the existing validity contract. See `CHANGES.md` entry `2026-04-20-0088`. ANVIL is now documented as growing from its current leaf-module typed-circuit lane into multiple families of pseudo-random, valid-by-construction, synthesizable HDL artifacts. The new durable distinctions are: (1) the current signoff-grade RTL lane stays intact, (2) future artifact families such as oracle-backed micro-design corpora and frontend/elaboration accept corpora are additive lanes, and (3) explicit expected-facts manifests are in scope while a bundled shadow simulator is still out of scope. The stale "reject corpus" drift and stale "only through peephole" wording were also cleaned up.
- **Prior slice:** The `e-graph` rung is now a live bounded semantic-sharing fragment for combinational cones. See `CHANGES.md` entry `2026-04-20-0087`. `FactorizationLevel::EGraph` now remains live under `identity_mode = node-id`, `generate_leaf_module` now runs `merge_equivalent_gates` before the post-drain flop merge, and `Module` / `Metrics` now expose `semantic_gates_merged`. Small-support same-endpoint cones can now collapse even when they were built with different graph shapes. The docs also now capture the new durable steering rule from the latest user exchange: adversarial generation must be modeled as an orthogonal axis matrix with no hidden implementation bias.
- **Prior slice:** Stateful identity now has a bounded semantic proof path for small-support cones. See `CHANGES.md` entry `2026-04-20-0086`. `merge_equivalent_flops` still falls back to the leaf-aware normalized structural proof, but for cones whose canonical endpoint support is small enough (`<= 10` bits today) it now enumerates all endpoint assignments and keys the cone by its actual truth table. That means some genuinely different-shape cones over the same endpoint set now merge, while different-endpoint cones like `q0 + 1` / `q1 + 1` still stay distinct. Live docs + the factorization chapter now say this explicitly instead of stopping at structure-only proof forms.
- **Prior slice:** Endpoint-aware functional state identity replaced both the old self-relative shortcut and the too-strict exact-`d` fallback. See `CHANGES.md` entry `2026-04-20-0085`. `merge_equivalent_flops` moved to endpoint-aware normalized proof forms over canonical leaf endpoints.
- **Prior slice:** The roadmap and durable docs now carry an explicit four-gap suitability map for steering future implementation. This slice answers the question "is the current codebase suited to the goal?" with "yes, as a foundation" and then spells out the four concrete gaps that still govern PNT choices: (1) feature breadth beyond the leaf kernel, (2) `NodeId`-as-identity beyond normalized combinational forms plus exact-signature flop merge, (3) industrialized Verilator/Yosys clean-run evidence, and (4) structure-first implementation rather than whole-module specification chasing. `ROADMAP.md`, `DEVELOPMENT_NOTES.md`, `CODEBASE_ANALYSIS.md`, `book/src/architecture.md`, and `book/src/factorization.md` now all tell the same story, and `src/ir/types.rs` / `src/gen/module.rs` Rustdoc was aligned so the code comments match the current factorization/foundation story. See `CHANGES.md` entry `2026-04-20-0083`.
- **Prior slice:** The structure-over-functionality doctrine is now captured explicitly, and verbatim, in both the book and the live contributor docs. See `CHANGES.md` entry `2026-04-20-0082`. `book/src/core-idea.md` now records the user's clarification word-for-word: recursively generating fanin cones mechanically yields arbitrary or gibberish whole-module behavior, and that is acceptable because ANVIL is targeting legitimate structure and downstream-tool ingestibility, not intended top-level functionality. `README.md`, `ROADMAP.md`, `DEVELOPMENT_NOTES.md`, `book/src/introduction.md`, and `book/src/faq.md` were aligned to the same distinction: whole modules are usually arbitrary in behavior, while some local motifs can still be functionally correct blocks. This slice is docs-only but highly load-bearing because it corrects a wrong premise that had survived in `core-idea.md`.
- **Prior slice:** The signoff-grade bug-finder direction is now captured explicitly in the durable docs. See `CHANGES.md` entry `2026-04-20-0081`. `README.md`, `ROADMAP.md`, `DEVELOPMENT_NOTES.md`, and the book now say the stronger version plainly: `anvil` should become a signoff-level quality random synthesizable RTL generator whose outputs are clean in downstream tools by default and still adversarial enough to expose real bugs in them. `book/src/non-goals.md` now clarifies that "no oracle / no reference simulator" means "no bundled shadow simulator", not "downstream-tool stress is somebody else's concern". This slice is docs-only but load-bearing for future PNT choices because it makes "legal, clean, adversarial RTL" a durable implementation filter rather than a transient conversation.
- **Prior slice:** NodeId identity now reaches exact-signature flops. See `CHANGES.md` entry `2026-04-20-0078`. Added `ir::compact::merge_equivalent_flops(&mut Module) -> u32`, called from `generate_leaf_module` after flop drain and mux-metadata summarization. Under `identity_mode = node-id` with effective level `>= cse`, flops now merge when they have identical exact emitted-state signatures: `width`, `reset_kind`, `reset_val`, and exact same `d: NodeId`. The pass rewires duplicate Q consumers, remaps virtual flop deps, renumbers surviving flops, rebuilds dedup tables, and leaves dead duplicate `FlopQ` nodes for `compact_node_ids` to remove. New telemetry: `Module::flops_merged` + `Metrics::flops_merged`. 3 new compact unit tests landed; `cargo test` is now 86 unit + 24 integration = 110 passing tests. Live docs + book updated to say the combinational ladder is mainly intern-time, while this first stateful identity step is a conservative post-drain finalisation pass.
- **Prior slice:** Identity mode is now a first-class typed axis. See `CHANGES.md` entry `2026-04-20-0077`. Added `IdentityMode` (`node-id` default, `relaxed`) to `Config`, `Cli`, and `Module`, plus `effective_factorization_level()` helpers so the coarse NodeId semantics switch is applied before every factorization/anti-collapse gate. New CLI flag `--identity-mode <node-id|relaxed>` landed, and the convenience aliases were tightened to expand to the explicit pair: `--full-factorization` → `node-id + e-graph`, `--no-full-factorization` → `relaxed + none`. Proof tests now show the same requested `factorization_level = e-graph` dedupes under `node-id` but allocates fresh nodes under `relaxed`; `cargo test` reached 83 unit + 24 integration = 107 passing tests. Live docs + book refreshed so identity mode is consistently described as orthogonal to construction strategy and finer-grained than the ladder.
- **Prior slice:** Peak-sharing control surface + live category/knob exercise sweep. See `CHANGES.md` entry `2026-04-20-0076`. Added coarse CLI aliases `--full-factorization` / `--no-full-factorization` on top of the existing factorization ladder, exposed live config-only knobs (`terminal_reuse_prob`, `constant_prob`, and the gate-category weights) on the CLI, and made the documented leaf knobs real by teaching `pick_terminal` to consult both `terminal_reuse_prob` and `constant_prob` through `roll_knob`. `pick_gate` now exercises the full comparison family plus the reduction bucket, so `gate_reduce_weight` is live instead of dead. Tests now cover CLI aliases, the two duplication-rate probability validators, direct gate-category coverage, leaf-knob edge behavior, and an end-to-end category sweep; `cargo test` reached 80 unit + 24 integration = 104 passing tests. Docs refreshed across live docs + book to align on the fact that the factorization ladder is live through `peephole`, and to make explicit that construction strategy is orthogonal to identity/sharing mode.
- **Prior slice:** Finalise the live-signal surface for lint-clean output. See `CHANGES.md` entry `2026-04-19-0075`. Root-cause fix for the reported "unused bits / unused signal" issue: non-multiple width adapters now build exact-width Concats instead of oversized Concat+Slice shapes; finalisation now summarizes `Flop.mux` metadata before compaction, shrinks surviving primary inputs to their highest live bit, and prunes dead data-input ports; `nested_associative_operand_count` now respects the strict Add/Mul duplicate-preservation policy instead of counting semantically-illegal flattening as residual opportunity. Added 2 module unit tests + 1 integration test, refreshed stale graph-first-alias docs/tests, and restored full hygiene (`cargo check`, `cargo test`, `cargo clippy -D warnings`, `cargo fmt --check`) to green. Verilator unused-signal sweep clean for seeds 0..4 on both the default path and the `graph-first` alias; seed-42 Yosys synth clean.
- **Prior slice:** All-constant evaluation completes the constant-fold surface. See `CHANGES.md` entry `2026-04-17-0074`. Extended `fold_constants` with three new all-const paths: (1) associative ops (`And/Or/Xor/Add/Mul`) — bitwise/arithmetic eval over values with mod-2^width wrapping; supersedes existing absorbing + identity-drop for the all-const subcase; (2) 2-arity Sub/Shl/Shr — arithmetic eval with over-shift → 0 clamp and mod-2^width wrap for Sub/Shl. Extended `apply_peephole` Concat arm with MSB-first bit-assembly for all-const operand lists (matches SV emit `{c1, c2, c3}` convention; widths must sum to gate width). 8 new unit tests. Docs: `factorization.md` constant-fold table split into associative + non-commutative sub-tables with All-const columns; Rule 21c rows extended; `fold_constants` and `apply_peephole` Rustdoc rule tables refreshed. 70 unit + 22 integration = 92 tests pass. After this slice the "constants flow through pure operators" story is complete for every operator class; remaining gap is cross-gate symbolic rewrites over non-constants (e-graph problem).
- **Prior slice:** Thorough docs pass on factorization pipeline (docs only). See `CHANGES.md` entry `2026-04-17-0073`. In-code Rustdoc upgrades on `Module::intern_gate` (rewrote into full pipeline-orchestrator spec with numbered steps and orphan-safety contract), `intern_constant`, `fold_constants` (added rule table + absorbing orphan-safety restriction + non-commutative position-sensitivity note), `flatten_associative` (Layer-4 framing), `apply_peephole` (rewrote the stale doc that was missing Not(cmp) inversions + all-const Not/Slice/reduction evaluations — grouped by outer operator), `KnobRollCounters::record`. Fixed stale field docs on `Module::factorization_level` (was "Default Full" → now "Default EGraph") and `peephole_rewrites_applied` (extended rule list). New book chapter `book/src/factorization.md` under "How It Works" in SUMMARY.md: doctrinal anchor, ladder, pipeline in execution order with per-layer tables, orphan safety / compaction, empirical counters with seed-42 baseline, knob-sweep recipe. `ir.md` Module struct listing refreshed + "Node construction" rewritten with three subsections (CSE, full pipeline, orphan safety). `architecture.md` crate layout adds `compact.rs` entry (was missing) and factorization counters / KnobId / KnobRollCounters in Key Types block. 62 unit + 22 integration = 84 tests unchanged.
- **Prior slice:** Peephole all-const evaluation: Not, Slice, reductions. See `CHANGES.md` entry `2026-04-17-0072`. Extended `apply_peephole` with four new constant-folding rules: `Not(c) → ~c & mask`, `Slice(hi, lo)(c) → (c >> lo) & mask`, `RedAnd(c)`, `RedOr(c)`, `RedXor(c)` all evaluated to 1-bit constants. Closes the gap noted in previous slice (`Not(Eq(c1, c2))` now folds end-to-end to a 1-bit const; was previously stuck at the Neq fold). Fires share `peephole_rewrites_applied`; orphan-safe because constants don't count as gate orphans for Rule 18. Known gaps remaining: `Concat(all-const)` and `Shl/Shr(const, const)` — both need minor width/shift-amount accounting. 3 new + 1 upgraded unit tests. 62 unit + 22 integration = 84 tests pass. Peephole layer now handles all-const evaluation for every operator class except Concat and shifts.
- **Prior slice:** Cross-gate peephole — `Not(comparison) → inverted comparison`. See `CHANGES.md` entry `2026-04-17-0071`. Extended the `Not` arm of `Module::apply_peephole` to cover `Not(Eq) → Neq`, `Not(Neq) → Eq`, `Not(Lt) → Ge`, `Not(Gt) → Le`, `Not(Le) → Gt`, `Not(Ge) → Lt`. Recursive `self.intern_gate` call creates the inverted comparison; the original inner comparison becomes orphaned and is cleaned up by `compact_node_ids` at module finalisation. Implementation pattern: extract inner gate fields into owned values before touching `self.intern_gate` recursively. Seed-42 empirical: peephole_rewrites 9 → 31 (+22 Not(cmp) fires); nodes_compacted 94 → 96 (most cmps shared via CSE and remain reachable). 3 new unit tests. 59 unit + 22 integration = 81 tests pass. Establishes infrastructure pattern for future cross-gate peephole work toward the EGraph ceiling.
- **Prior slice:** Associative flattening factorization layer goes live (Rule 21c layer 4). See `CHANGES.md` entry `2026-04-17-0070`. `FactorizationLevel::Associative` promoted to implemented. `Module::flatten_associative` dispatched from `intern_gate` BEFORE commutative sort: for `And`/`Or`/`Xor`/`Add`/`Mul`, splice any same-op same-width inner gate operand into the outer operand list. Per-op semantic normalisation: And/Or dedup (idempotent), Xor pair-cancel (self-inverse), Add/Mul skip the flatten when duplicates would result under strict `operand_duplication_rate` (preserves `x + x = 2x` / `x * x = x²`). Counter `flatten_associative_applied` via Metrics. Canary `nested_associative_opportunities_exist_today` (which asserted > 0 pre-layer) renamed to `nested_associative_opportunities_flatten_to_zero` and flipped to `== 0`. Seed-42 empirical: nested count 373 → 0; flatten fires 268; nodes_compacted 7 → 94 (inner gates orphaned by splice cleaned up by compaction pass). ConstantFold reach expanded (91 vs 28) as flattening exposes identity operands. 4 new unit tests. 56 unit + 22 integration = 78 tests pass. **Only `EGraph` (theoretical ceiling, cross-gate identities) remains aspirational.** The syntactic-identity contract is now complete: every expression built any way sharing the same AST-after-normalisation shares one NodeId.
- **Prior slice:** NodeId compaction pass + Not(Not(x)) peephole unlock. See `CHANGES.md` entry `2026-04-17-0069`. New `src/ir/compact.rs` adds post-construction `compact_node_ids(&mut Module) -> u32`: BFS from all roots (drives, flop fields), remap NodeIds, drop unreachable nodes, preserve topological order, rebuild dedup tables. Called from `generate_leaf_module` after flop drain. `Not(Not(x)) → x` peephole re-enabled — the inner Not becomes an orphan at intern time, compaction cleans it up at finalisation. New `Module::nodes_compacted: u32` + `Metrics::nodes_compacted`. Seed-42 default knobs: 7 nodes compacted per module, 9 peephole fires (up from 2 pre-Not(Not)). 3 new compact unit tests + 1 restored peephole test + 1 new integration test (`compaction_preserves_rule_18_and_records_removals`: 40-seed sweep asserts zero orphans post-compaction + validator accept + total compacted > 0). 51 unit + 22 integration = 73 tests pass. **Infrastructure in place for Associative flattening** — the intern-time merge logic (`Add(a, Add(b, c))` → `Add(a, b, c)`) is the natural next slice; it would orphan the inner Add which compaction now handles.
- **Prior slice:** Per-knob probability-roll counters (attempts / fires) live. See `CHANGES.md` entry `2026-04-17-0068`. New `KnobId` enum (10 variants, one per `gen_bool(cfg.<prob>)` site) and `KnobRollCounters` struct on `Module`. New `roll_knob` helper in `src/gen/cone.rs` replaces all 25 `gen_bool(cfg.<prob>)` sites with a roll-and-record call. Surfaced via new `Metrics::knob_roll_attempts` and `knob_roll_fires` (BTreeMap<String, u64>). Knobs instrumented: flop_prob, comb_mux_prob, priority_encoder_prob, coefficient_prob, const_shift_amount_prob, const_comparand_prob, comb_mux_encoding_prob, flop_mux_encoding_prob, share_prob, flop_qfeedback_prob. Seed-42 empirical ratios all track configured defaults within sampling noise (e.g. `share_prob: 607/1999 ≈ 0.304` for default `0.3`, `coefficient_prob: 51/256 ≈ 0.199` for default `0.2`). New integration test `knob_rolls_recorded_across_seeds` (attempts > 0 per knob, fires ≤ attempts). knobs.md gets a "Per-knob roll-rate validation" subsection. 47 unit + 21 integration = 68 tests pass. Measurability doctrine now at its strongest form for probability dials. **Remaining aspirational factorization rungs (`Associative`, deeper peephole): still blocked on NodeId compaction** — the natural next architectural slice, deferred here to keep this slice well-scoped.
- **Prior slice:** Peephole factorization layer goes live (Rule 21c layer 6), orphan-safe subset. See `CHANGES.md` entry `2026-04-17-0067`. `FactorizationLevel::Peephole` promoted to implemented; default `EGraph` walks down to `Peephole` as effective. `Module::apply_peephole` wired to `intern_gate` with three narrow orphan-safe rules: fully-constant comparison evaluation (`Eq/Neq/Lt/Gt/Le/Ge` with both operands constants → 1-bit const); full-width `Slice(hi, 0)` with `hi+1 == src_width` → src; single-operand `Concat([x]) → x`. Counter `peephole_rewrites_applied` via Metrics. `Not(Not(x)) → x` deliberately NOT implemented (would orphan the inner Not — violates Rule 18 without NodeId compaction; deferred to compaction-equipped e-graph). Same slice hardens ConstantFold's absorbing rule: now fires only when no operand is a Gate (the "evaluate all-constant expression" subset); dynamic absorbing on Gate operands was a latent Rule 18 orphan hazard exposed by peephole's RNG path shift. 4 new peephole unit tests + 1 new integration test (`peephole_layer_fires_at_default_knobs`). 47 unit + 20 integration = 67 tests pass. Remaining aspirational rungs: `Associative` and deeper peephole (cross-gate) — both blocked on NodeId compaction.
- **Prior slice:** ConstantFold factorization layer goes live (Rule 21c layer 5). See `CHANGES.md` entry `2026-04-17-0066`. `FactorizationLevel::ConstantFold` promoted to implemented; `is_implemented()` + `effective()` rewritten to walk the enum top-down and cleanly skip unimplemented middle rungs (e.g. `Associative` drops to `Commutative`, default `EGraph` activates `ConstantFold`). `Module::fold_constants` dispatches at `intern_gate` time for: absorbing (`x & 0`, `x | all_ones`, `x * 0`), identity-drop (drops `0`/`1`/`all_ones` operands per op), and 2-arity Sub/Shl/Shr rhs-zero short-circuit. Counter `fold_identities_applied` surfaced via Metrics. Three latent bugs exposed and fixed in the same slice: (1) `assemble_mul_linear_combination` now dedupes the coef constant against its signal list (closes a post-CSE collision path); (2) `make_mul` / `make_sub` gain degeneracy guards mirroring `make_and`; (3) `deliver`'s interleaved anti-collapse fallback returns a width-correct constant for comparisons (`Eq(a,a) → 1`, `Neq(a,a) → 0`) instead of `operands[0]` — previously `operands[0]` was the comparand width K, not the 1-bit comparison output, so the delivered fallback landed at the wrong width in the parent slot. 6 new unit tests + 1 new integration test (`constant_fold_layer_fires_at_default_knobs`). 49 unit + 19 integration = 68 tests pass. Remaining aspirational rung: `Associative` (canary `nested_associative_opportunities_exist_today` still in place; flips to `== 0` when it lands).
- **Prior slice:** Syntactic-vs-semantic-identity framing in the factorization-ladder narrative (docs only). See `CHANGES.md` entry `2026-04-17-0065`. Added a durable one-paragraph framing to Rule 21b's "Position in the factorization ladder" (structural-rules.md) and to non-triviality.md's "Factorization ladder" subsection: *today's layers guarantee syntactic identity; the goal is semantic identity; the latter is a strictly harder problem that synthesis tools themselves solve incompletely.* Sets reader expectations without overclaiming. 57 tests unchanged.
- **Prior slice:** Regression tests pinning three doctrine-level invariants. See `CHANGES.md` entry `2026-04-17-0064`. Added three integration tests in `tests/pipeline.rs`: (1) `zero_orphans_at_default_knobs` (Rule 18 guard across 4 strategies × 6 seeds); (2) `zero_duplicate_operands_at_default_knobs` (Rule 8 guard across 5 seeds, `And`/`Or`/`Xor`/`Add`/`Mul` operand uniqueness); (3) `nested_associative_opportunities_exist_today` (informational canary — flips direction when Associative layer lands). Test count 54 → 57.
- **Prior slice:** Associative-flattening opportunity metric (informational). See `CHANGES.md` entry `2026-04-17-0063`. New `Metrics::nested_associative_operand_count`: post-hoc walk counts operand slots on associative gates whose operand is itself a same-op same-width gate — i.e., slots the not-yet-implemented `Associative` factorization layer would absorb. Seed sweep at default knobs shows 10–16% flattening opportunity → justifies queuing the full Associative implementation. knobs.md effectiveness map gains `operand_duplication_rate` entry + extended `factorization_level` entry. 54 tests unchanged.
- **Prior slice:** FAQ chapter refresh — strategies + full-factorization Q (docs only). See `CHANGES.md` entry `2026-04-17-0062`. `book/src/faq.md` refreshed: "four strategies" → "three" (graph-first retired, silent-alias note); cross-cone-sharing entry updated to reflect interleaved + CSE; new entry "What does 'full factorization' mean? Does anvil dedupe?" covering the three implemented layers (CSE / operand-unique / commutative) + four aspirational layers + `--factorization-level` dial. **Book audit complete.** Every authored chapter reflects shipping code; only `hierarchy.md` (Phase 4+ placeholder) and doctrine chapters (core-idea / non-goals / why-not-grammar) remain un-audited, by design. 54 tests unchanged.
- **Prior slice:** Sharing chapter refresh — Rule 2 + Rule 18 + Rule 21 CSE (docs only). See `CHANGES.md` entry `2026-04-17-0061`. `book/src/sharing.md` refreshed: `try_share` no longer references Q-exclusion (Rule 2 pointer instead); forbidden-patterns list matches current Rule 8 extended rule set with knob gating; **new "Construction-time CSE" section** replaces the old "sharing does not CSE" paragraph (which contradicted the shipping code); cross-output section corrected (interleaved default, graph-first retired); "No cycles" retitled "No combinational cycles" with Rule 1 + Rule 2 cross-links. 54 tests unchanged.
- **Prior slice:** Non-triviality chapter — anti-collapse rule table + factorization-ladder framing (docs only). See `CHANGES.md` entry `2026-04-17-0060`. Anti-collapse table in `book/src/non-triviality.md` rewritten to match `violates_anti_collapse` reality (removed never-implemented entries like `a & 0`, `a | all_ones`, shift-by-0; added N-arity operand-multiset distinctness, Add/Mul knob-gated, Mux knob-gated). New snapshot-restore note connecting to Rule 18 α. "Algebraic residue" section reframed to cite the implemented factorization layers (CSE, operand-unique, commutative) and the four aspirational ones (associative, constant-fold, peephole, e-graph) with Rule 21c cross-link. 54 tests unchanged.
- **Prior slice:** By-construction chapter — validator tense + Rule 18 exemplar + retry grandfather clause (docs only). See `CHANGES.md` entry `2026-04-17-0059`. `book/src/by-construction.md` gets (1) tense fix on validator (shipped, not "will include"); (2) new "Exemplar: Rule 18" sub-section recording the α-vs-β decision, mechanism (build_cone snapshot + process_signal_frame existing-operand fallback), empirical result (0 orphans × 4 strategies × 6 seeds); (3) new "Grandfather clause: bounded retry" sub-section making explicit that `build_cone_with_retry`'s snapshot-per-attempt empty-dep retry is the one permitted retry-and-discard pattern, and any additional one would be a design regression. 54 tests unchanged.
- **Prior slice:** Architecture chapter refresh — align with current workspace reality (docs only). See `CHANGES.md` entry `2026-04-17-0058`. `book/src/architecture.md` refreshed: crate layout adds metrics.rs and extends per-file descriptions (TRACE_DEBUG + trace_verbose!, ConstructionStrategy + FactorizationLevel enums, intern_gate API + dedup tables + knob mirrors + block-build counters, motif-dispatch, snapshot/rollback, orphan audit, dumb-emitter doctrine). "Key types" rewritten: Module struct with dedup tables + knob mirrors + intern_gate/intern_constant sigs; GateOp expanded; ConstructionStrategy + FactorizationLevel enums added; metrics::Metrics + lib.rs trace signatures shown. Testing-strategy counts updated 23→54. CLI section collapsed to a pointer at `knobs.md` + `anvil --help` (drift-reduction). 54 tests unchanged.
- **Prior slice:** Algorithm chapter refresh — strategies, Rule 2, Rule 18, CSE, motif dispatch (docs only). See `CHANGES.md` entry `2026-04-17-0057`. `book/src/algorithm.md` refreshed to match current `src/gen/cone.rs`: strategy note (interleaved default, graph-first retired); `build_cone` pseudocode now includes priority-encoder dispatch + three motif branches (linear-combination, const-shift, const-comparand) + snapshot/rollback around operand construction + intern_gate for CSE; flop-drain fixed from `exclude = Some(q_node)` to `exclude = None` with Rule 2 pointer; retry-loop section mentions dedup-table snapshot; anti-collapse section rewritten with full rule set gated on factorization_level + operand/mux-arm duplication rates. 54 tests unchanged.
- **Prior slice:** Book audit — last `w_N`/`r_N` naming remnants (docs only). See `CHANGES.md` entry `2026-04-17-0056`. Three final chapters refreshed: `introduction.md` (5-minute pitch with a real seed-20 `flop_0` hold-register — 23 lines showing Rule 12 naming in action); `sequential.md` (clock-and-reset snippet `r_0 → flop_0`, added Rule 12 pointer); `synthesizability.md` (flop template `r_0 → flop_0`, also corrected an aspirational footnote about sync-reset/no-reset variants — aligned with Rule 5). Remaining `w_N`/`r_N` in structural-rules.md Rule 12 motivation is deliberate (contrasts new vs retired scheme). 54 tests unchanged.
- **Prior slice:** Construction-strategies chapter refresh (docs only). See `CHANGES.md` entry `2026-04-17-0055`. `book/src/construction-strategies.md` rewritten: lede 4→3 strategies, default now interleaved, new "Retired: graph-first" section explaining Rule 18 orphan-rate rationale + silent-alias behaviour + "use interleaved" migration, comparison table updated, implementation-status section updated, new Rule 18 bullet in the "interaction with existing rules" list making the zero-orphan construction contract explicit. 54 tests unchanged.
- **Prior slice:** Tutorial chapter refresh — naming scheme + re-captured examples (docs only). See `CHANGES.md` entry `2026-04-17-0054`. Added lede paragraph introducing the `<kind>_<N>` / `flop_<N>` naming scheme with Rule 12 pointer. Re-captured Example 4 (direct-D flop → `flop_0`/`shl_0`), Example 5 (one-hot mask with canonical `{W{sel}} & data` pattern + replication-syntax note), Example 6 (encoded D mux with `slice_0`/`eq_0`/`mux_0`/`eq_1`/`mux_1` verbatim), Example 9 (comb-mux encoded, 3-arm chained ternary with bottom-up read). Prose in Examples 2 and 8 updated from `w_N` to `<kind>_N`. Every SV excerpt re-captured by running its `cargo run` command against HEAD. 54 tests unchanged.
- **Prior slice:** Knobs chapter alignment with actual config + CLI surface (docs only). See `CHANGES.md` entry `2026-04-16-0053`. Audit-and-fix of `book/src/knobs.md`: added missing `operand_duplication_rate` entry in catalog; refreshed defaults block (was missing ~20 knobs); rewrote CLI coverage section as 11 categories covering all 44 flags; updated Quick Reference table (construction-strategy default, trace-default alias). Validation script confirms book lists every CLI flag shown by `anvil --help` except `--version`. 54 tests unchanged.
- **Prior slice:** Block counters — priority_encoder + comb_mux_encoding (closes the last pending entries). See `CHANGES.md` entry `2026-04-16-0052`. New Module fields `priority_encoder_built`, `comb_mux_one_hot_built`, `comb_mux_encoded_built` (u32 live counters); incremented at block-build sites (recursive + pool for priority encoder; comb_mux & comb_mux_pool_only for the two comb-mux shapes). Exposed as Metrics `num_priority_encoder_blocks`, `num_comb_muxes_one_hot`, `num_comb_muxes_encoded`. Seed-42 sweeps: priority-encoder counter clean monotone (0→49→221→454 at probs 0→0.05→0.2→0.5); comb-mux encoding hits 0/all at 0.0/1.0, splits 887/859 at 0.5. **No *pending* entries remain in the knob-effectiveness map.** 54 tests pass.
- **Prior slice:** Combinational-depth metrics (closes another pending entry). See `CHANGES.md` entry `2026-04-16-0051`. New `Metrics` fields `max_gate_depth` + `gate_depth_histogram`; single forward walk over `m.nodes` (topological) assigns `depth = max(operand depth) + 1`. Knob-effectiveness map for `max_depth` moves from *pending* → concrete. Seed-42 sweep shows clean monotone 54→115→154→206→354 as `--max-depth` goes 2→10 (knob-to-metric ratio ≈ 10–100× due to block-assembly gate-chain expansion). Pending entries shrink from 3 → 2: `priority_encoder_prob` + `comb_mux_encoding_prob` still to cover.
- **Prior slice:** Live-doc catch-up (docs only). See `CHANGES.md` entry `2026-04-16-0050`. Refreshed four stale live docs: CODEBASE_ANALYSIS.md (module map, phase coverage, invariants, 54-test count); USER_GUIDE.md (new metrics fields, updated knob effects list); DEVELOPMENT_NOTES.md (5 new design-decision sections for the last 15 src-touching commits: construction-time CSE via intern_gate, Rule 18 α enforcement + GraphFirst retirement, full factorization doctrine, dumb-emitter doctrine, rejected without-replacement default); ROADMAP.md (Phase 1 → mostly done; Phase 3 item statuses; smoke tests blocked locally). No code, no test count change. Commit workflow audit surfaced the drift; enforcing strictly going forward.
- **Prior slice:** Operand-arity metrics (closes a pending effectiveness-map entry). See `CHANGES.md` entry `2026-04-16-0049`. New `Metrics` fields `gate_operand_count_histogram`, `max_gate_operand_count`, `max_operand_count_by_kind`. Knobs `min_gate_arity` / `max_gate_arity` now have concrete metric coverage: `max_operand_count_by_kind["add"]` tracks the knob exactly; `["mul"]` is knob+1 (coefficient prepends); `concat` unbounded by the knob (replicate-to-width). Effectiveness map updated — still pending: `max_depth`, `priority_encoder_prob`, `comb_mux_encoding_prob`. 54 tests pass.
- **Prior slice:** Close residual Add/Mul/And duplicate operands at default knobs. See `CHANGES.md` entry `2026-04-16-0048`. Three fixes: (a) `assemble_mul_linear_combination` dedupes signals when `operand_duplication_rate < 1.0`; (b) `assemble_add_linear_combination` dedupes the post-Mul `terms` list; (c) `make_and` short-circuits `x & x = x` when factorization level ≥ OperandUnique (closes the one-hot-mux mask escape path). Syntactic factorization now COMPLETE at default knobs: 0 duplicate operands across 4633 gates × 5 seeds (was 0.09%). Config::FactorizationLevel::Default via `#[default]` derive; doc comments reworded to avoid clippy lint. 54 tests pass.
- **Prior slice:** Recipe for the factorization dial (docs only). See `CHANGES.md` entry `2026-04-16-0047`. `book/src/recipes.md` gains a paste-and-run sweep over `--factorization-level none..e-graph` with real seed-42 gate counts and a layer-by-layer reading of the deltas. Addresses the "littered with examples" book doctrine.
- **Prior slice:** Commutative normalization + factorization-level dial. See `CHANGES.md` entry `2026-04-16-0046`. Layer 3 of the factorization chain lands: commutative ops (`And`/`Or`/`Xor`/`Add`/`Mul`) sort operands before intern, so `a+b` and `b+a` dedupe. New `FactorizationLevel` enum with 8 positions (`none → cse → operand-unique → commutative → associative → constant-fold → peephole → e-graph`), default `e-graph` (theoretical ceiling; clamps to highest implemented layer via `effective()`). CLI flag `--factorization-level`. 39 unit + 15 integration = 54 tests. Book Rule 21b + 21c landed. Aspirational levels (associative/constant-fold/peephole/e-graph) compile without behavioural surprise — future slices will activate them for users already at those levels.
- **Prior slice:** Operand-uniqueness knob (`--operand-duplication-rate`). See `CHANGES.md` entry `2026-04-16-0045`. New knob `operand_duplication_rate: f64 ∈ [0.0, 1.0]`, default 0.0 → strict Add/Mul operand uniqueness. `violates_anti_collapse` now checks Add/Mul duplicates when knob < 1.0; And/Or/Xor always strict (algebraic). `pick_signals_with_dup_rate` helper for pool-mode linear-combination. **User coined "full factorization"** = CSE (NodeId uniqueness across AST) + operand-uniqueness (no NodeId twice inside one gate) — both now enforced at default. 49 tests pass. Residual 0.09% duplicates in recursive linear-combination path (CSE-collapse of sub-cones); follow-up to reach 0%.
- **User doctrine (logged in memory):** NodeId = identity of an expression. Full factorization = no expression / sub-expression / sub-sub-expression ever duplicated — every expression has a unique NodeId. Beyond today's syntactic CSE; reaches algebraic equivalence (next-level work: commutative normalization, associative flattening, constant folding). Target for future slices.
- **Prior slice:** `--trace debug` is now strictly more verbose than `high`; `off` aliased as `none`. See `CHANGES.md` entry `2026-04-16-0044`. New `trace_verbose!` macro + `TRACE_DEBUG` atomic guard on `src/lib.rs`. `Module::intern_gate` / `intern_constant` emit `🔗 new` and `♻️ reuse` events — every node entering the IR is traceable. `pick_gate` return traced in both recursive and interleaved paths with depth + width. CLI: `--trace none` default (was `off`, kept as alias). Empirical line counts at seed 42: none=0, low=5, medium=141, high=3779, debug=8241 (+4462 strict super-set). 49 tests pass.
- **Prior slice:** Zero orphans: Rule 18 enforced construction-time. See `CHANGES.md` entry `2026-04-16-0043`. build_cone snapshots m.nodes/flops/pool/worklist/dedup-tables before operand construction; anti-collapse rejection rolls back. process_signal_frame (interleaved) can't snapshot per-gate so it uses an existing operand as anti-collapse fallback (no new node). GraphFirst retired as default; silently aliased to Interleaved. Safety-net audit warns if any orphan survives. Emitter reverts to dumb serialiser per doctrine. 49 tests, 0 orphans across 4 strategies × 6 seeds. **Known gap (next slice):** trace doesn't show "who requested this new gate" — build_cone and process_signal_frame need op-pick trace events with requester context.
- **Prior slice:** IR chapter refresh + future-extensions roadmap (docs only). See `CHANGES.md` entry `2026-04-16-0042`. `book/src/ir.md` gets (1) refreshed `Module` struct showing `gate_instances`, `const_instances`, `max_ast_instances`, `mux_arm_duplication_rate`; (2) new "Node construction" section documenting `intern_gate` / `intern_constant` signatures, cap semantics, snapshot/rollback contract; (3) naming section updated for Rule 12 (no more `w_N`/`r_N`); (4) new "Future extensions" section for parameters (Phase 5, Phase-4-dependent), synthesizable aggregates (four sub-paths with cost/payoff — packed cheap/emitter-only, unpacked arrays = Phase 6 memories, unpacked datapath + enums deprioritised), blocks as first-class IR. `ROADMAP.md` gains a Phase 5b aggregates entry. mdbook builds cleanly.
- **Prior slice:** Friendly docs — quick ref, naming refresh, recipe examples (docs only). See `CHANGES.md` entry `2026-04-16-0041`. `getting-started.md` sample output refreshed to match typed-per-kind naming (`slice_0`, `add_0`, `mul_0`) with a naming explanation. `knobs.md` gains a reassuring intro ("you don't need to read this top-to-bottom") and a Quick reference table of the ~13 most-touched knobs. `recipes.md` gets 6 new recipes: strict CSE (default); duplicated expressions (`--max-ast-instances`); pathological mux shapes (`--mux-arm-duplication-rate`); verify-a-knob via metrics grep; sweep-a-knob workflow with real `--flop-prob` values; trace levels with sample output. mdbook builds cleanly. 50 tests unchanged.
- **Prior slice:** Knob measurement doctrine + effectiveness map (docs only). See `CHANGES.md` entry `2026-04-16-0040`. `book/src/knobs.md` gains (1) a "Measurement doctrine" opening: no knob is privileged, every knob's effect must be empirically measurable via `Metrics` and/or `--trace`, with three landing requirements; (2) a dedicated "AST uniqueness / duplication" sub-section covering `max_ast_instances` and `mux_arm_duplication_rate`; (3) a knob-to-metric effectiveness map at the bottom listing which metric measures each knob, with *pending* entries flagging known gaps. No code changed. mdbook builds cleanly. 50 tests unchanged.
- **Prior slice:** Structural metrics (per-module observability). See `CHANGES.md` entry `2026-04-16-0039`. New module `src/metrics.rs` with `Metrics` struct + `compute(&Module)` post-hoc walker. Captures size, per-kind gate distribution, constant width/value distribution, mux shape (2-to-1 count + degenerate count), concat shape (replication vs heterogeneous), fanout (shared nodes + max + avg), flop kind/mux-shape distribution, AST-instance saturation. CLI flag `--metrics` → stderr JSON for single-module; multi-module runs always embed metrics in `manifest.json`. 3 new unit tests. 50 tests total. Knob effectiveness now empirically observable (seed 42 demonstration: `num_muxes_degenerate = 0` at default; flips to 1 at `--mux-arm-duplication-rate 1.0`). Live counters for attempt/miss/retry signals deliberately deferred to a future slice (most are already in `--trace high` events).
- **Prior slice:** Mux arm-duplication rate (Rule 22). See `CHANGES.md` entry `2026-04-16-0038`. New knob `mux_arm_duplication_rate: f64 ∈ [0.0, 1.0]`, default 0.0 (all arms distinct). Probabilistic uniqueness: at each arm pick, a candidate that duplicates an already-picked arm is kept with probability `rate` and rejected otherwise (8-try budget). 2-to-1 `make_mux` collapses `(s)?(x):(x) = x` when rate = 0.0; at any rate > 0.0 the upstream picker's decision stands. Applied at all pool-mode N-to-1 mux sites. Verified seed 42: 0 degenerate ternaries at default, 1 at rate 1.0. Book Rule 22 added. 47 tests pass.
- **Prior slice:** Construction-time CSE with tunable AST-instance cap (Rule 21). See `CHANGES.md` entry `2026-04-16-0037`. `Module::intern_gate` / `intern_constant` enforce a per-AST instance cap; default `max_ast_instances = 1` gives strict uniqueness (one RHS = one signal = one node). `GateOp` gains `Hash` derive. Every gate/constant creation in cone.rs routed through intern. Critical: `build_cone_with_retry` snapshots/restores `gate_instances` + `const_instances` alongside `m.nodes` — otherwise stale dedup entries would return wrong-kind nodes after rollback. CLI flag `--max-ast-instances`. Book Rule 21 added. 47 tests pass. Spot-check seed 42 confirms `slice_17 == 2'h2` now exists once (`eq_0`); at N=3 Eq count doubles.
- **Prior slice:** Emit `{N{expr}}` replication for same-operand Concat. See `CHANGES.md` entry `2026-04-16-0036`. `render_gate` for `Concat` detects all-operands-identical and emits the canonical SV replication form instead of the flat list. Clean-up triggered by user seeing `{eq_0, eq_0, … × 22}` in seed-42 output; now reads `{22{eq_0}}`. Semantics unchanged. Emitter unit test updated. 47 tests pass.
- **Prior slice:** UVM-style tracing (`--trace` / `--trace-file`). See `CHANGES.md` entry `2026-04-16-0035`. New deps: `tracing` + `tracing-subscriber`. CLI: `--trace <off|low|medium|high|debug>` default off, `--trace-file <path>`. Level mapping: low=INFO, medium=DEBUG, high/debug=TRACE. `#[instrument]` + explicit trace calls across `gen/module.rs`, `gen/cone.rs`, `emit/sv.rs` at the named control points (module start/done, strategy dispatch, motif forks, anti-collapse retry/exhausted, terminal tier picks 1-4, leaf-vs-recurse, emitter summary). Emojis at milestones only. Deterministic output — no timestamps/thread-ids/ANSI. Stdout stays byte-clean for SV. Release build compiles out below info. 47 tests pass; reproducibility holds byte-identical across trace levels. Block-level naming (`priority_encoder_0` flatten/hierarchical modes) still deferred.
- **Prior slice:** Typed per-kind naming in emitted SV (Rule 12 revised). See `CHANGES.md` entry `2026-04-16-0034`.
- **Doctrinal anchor:** user reinforced that generation must be rule-based (construction-time rules only, no post-hoc filters). Tree-shake / validator-as-gate are off the table. See `feedback_rules_first_generation.md` in session memory. This slice is the template: rule in catalog, invariant in picker.
- **Doctrinal anchor:** user clarified the tool-quality bar: generated
  output should be accepted by downstream HDL consumers by default.
  Verilator and Yosys sweeps are repository validation evidence toward
  that target, not the product target itself.
- **Doctrinal anchor:** user further clarified the product direction:
  `anvil` should become a signoff-level quality random
  by-construction synthesizable RTL generator. Generated legal RTL can
  be used to expose downstream tool bugs, but the way to do that is not
  malformed junk; it is legal, reproducible, structurally rich RTL that
  downstream HDL consumers should accept.
- **Doctrinal anchor:** user then clarified the structure-vs-function boundary and asked for it to be logged verbatim. The durable anchors are now `book/src/core-idea.md` and `DEVELOPMENT_NOTES.md`: whole-module intended functionality is generally absent and not a construction target; structure is the target; some local motifs may still be functionally correct blocks.
- **Doctrinal anchor:** user then asked whether the current codebase is suited to the goal and requested that the answer be logged durably. The answer now captured across `ROADMAP.md`, `DEVELOPMENT_NOTES.md`, `CODEBASE_ANALYSIS.md`, `book/src/architecture.md`, and `book/src/factorization.md` is: yes as a foundation, with four explicit steering gaps still open (feature breadth, stronger NodeId identity, industrialized clean-run evidence, structure-first implementation doctrine).
- **Doctrinal anchor:** user then clarified the meaning of cone identity more sharply: two fanin cones may not have the same `NodeId` if they do not have the same variable endpoints, and the real target is equality by proven same functionality with respect to those same endpoints. Shape alone is not the doctrine.
- **Doctrinal anchor:** user then clarified the generation-space doctrine more sharply: ANVIL must model all adversarial-generation axes explicitly and exercise them efficiently during real runs, with no hidden bias toward whichever implementation path is easiest. Construction strategy, identity mode, factorization level, category weights, and probability knobs are all separate axes.
- **Doctrinal anchor:** user then broadened the product scope again, but without relaxing the quality bar: the current valid-by-construction synthesizable lane stays valid and stays in force; ANVIL is instead meant to grow into multiple families of valid-by-construction synthesizable HDL artifacts. The first newly-requested families are oracle-backed micro-design corpora and frontend/elaboration accept corpora with explicit expected-facts manifests.
- **Prior slice:** Rule 18 proposal + sample-output defect catalogue (docs only). See `CHANGES.md` entry `2026-04-15-0030`. New `priority_encoder_prob` knob (default 0.05) + CLI flag. `pick_priority_encoder_n` finds an N ∈ `[min_mux_arms, max_mux_arms]` with `ceil_log2(N) == target_width`, returns None if none fits. `assemble_priority_encoder` emits a chained ternary `req_0 ? 0 : req_1 ? 1 : ... : 0`. `build_priority_encoder_recursive` and `build_priority_encoder_pool` dispatch helpers. Three dispatch sites (build_cone / process_signal_frame / grow_pool_one_unit) with applicability-check-then-fall-through semantics. Book Rule 17 added. 1 new integration test. 29 unit + 15 integration = 44 tests. See `CHANGES.md` entry `2026-04-15-0029`.
- **Doctrinal note (deferred):** the motif-trait refactor is explicitly deferred per user direction. After landing several more block motifs, revisit to factor the copy-paste pattern into a `Motif` trait + registry.
- **Conceptual advance this session:** the operators-vs-blocks distinction is now load-bearing doctrine. Operators (associative primitives) generalize by arity; blocks (mux, flop, future memory/FSM) generalize by structural parameters (port counts, encoding choices, feedback topology). Subsequent slices use this framework.
- **Next up (ordered by the four-gap steering map):**
  0. **Deepen Phase 4 hierarchy beyond the current banked gate.** Mixed parent-port / child-output parent composition, the first registered sibling route plus its multi-stage parent-Q chain, first registered parent-composed child-input route, registered mixed-support child-input routing, recursive non-top registered mixed-support child-input routing, the first multi-stage registered parent-composed chain, recursive non-top multi-stage registered parent-composed routing without helpers, recursive non-top multi-stage registered sibling routing without helpers, recursive non-top multi-stage registered mixed-support routing without helpers, recursive non-top registered helper mixed-support routing, recursive non-top parent-output helper mixed-support routing, parent-cone helper-instance parent-composed child-input, stateful parent-composed helper child-input routing, recursive non-top stateful parent-composed helper child-input routing, recursive non-top direct sibling helper routing, recursive non-top direct registered sibling helper routing, recursive non-top registered parent-composed helper routing, recursive non-top multi-stage direct registered sibling helper routing, recursive non-top multi-stage registered parent-composed helper routing, recursive non-top parent-output helper routing, recursive non-top stateful parent-output helper routing, recursive non-top parent-output multi-helper budgets, recursive non-top child-input multi-helper budgets, recursive non-top stateful multi-helper budgets, direct sibling, direct registered sibling, multi-stage direct registered sibling helper, multi-stage registered parent-composed helper, registered child-input D-cone, budgeted parent-output routes, stateful parent-output helper routes, budgeted parent-cone helper allocation, generator-global module-name allocation, the 66-scenario / 264-design full `r31` bank, the 69-scenario / 276-design full `r34` bank, the 72-scenario / 288-design full `r35` bank, the 75-scenario / 300-design full `r36` bank, the 78-scenario / 312-design full `r37` bank, the 81-scenario / 324-design full `r38` bank, the 84-scenario / 336-design full `r39` bank, the 90-scenario / 360-design full `r43` bank, the 93-scenario / 372-design full `r44` bank, the 96-scenario / 384-design full `r45` bank, the 99-scenario / 396-design full `r46` bank, the 99-scenario / 396-design full `r47` bank, the 99-scenario / 396-design full `r48` bank, the 99-scenario / 396-design full `r49` bank, the 99-scenario / 396-design full `r50` bank, the 102-scenario / 408-design full `r51` bank, the 105-scenario / 420-design full `r52` bank, the 108-scenario / 432-design full `r53` bank, the 111-scenario / 444-design full `r54` bank, and the 114-scenario / 456-design full `r55` bank are live; the next structural work is broader registered hierarchy routing/composition and hierarchy-aware identity.
  1. **Keep the hierarchy gate representative without letting it drift back into leaf-stress cost or stale total-budget arithmetic.** The banked `r53` result closes cleanly because the Phase 4 sequential profiles are hierarchy-focused rather than borrowing the heaviest Phase 1 leaf stress, because helper-through-state metrics are dependency/memo based instead of recursive-cone expensive, because CaseMux/Casez exact-selector bounds clean warning-prone shifts, because post-remap idempotent duplicate cleanup preserves the strict operand doctrine, because mixed helper-support is measured directly instead of inferred from separate counters, because direct registered sibling mixed-support is measured separately from registered parent-composed routing, and because the gate budget directly preserves four designs/scenario as the scenario set grows.
  2. **Broaden semantic identity beyond the current bounded fragment.** `merge_equivalent_gates` now covers small-support combinational cones at `e-graph`, and `merge_equivalent_flops` now covers both the endpoint-aware normalized-proof subset and a bounded small-support semantic proof. The next factorization question is stronger equivalence across larger supports, richer D-cone graphs, and future state/hierarchy motifs, but only when it can preserve the same canonical leaf endpoints and supply a real proof of equal functionality.
  3. **Turn the new artifact-family mandate into executable architecture.** The next docs-to-code bridge is deciding how ANVIL selects artifact families above the current leaf-module lane, how expected-facts manifests are represented, and what minimum source-level parameter / hierarchy / package IR is needed for the first oracle-backed micro-design and frontend/elaboration accept corpora.
  4. **Memories (medium).** Inferrable single-port / simple-dual-port memory patterns (`reg [W-1:0] mem [0:DEPTH-1]` with an always_ff block driving read/write). Knob for depth range.
  5. **FSMs (medium-large).** Explicit state encoding (binary / one-hot / gray), transition logic, optional output logic. The first real multi-part block motif.
  6. **Hierarchy + parameterization.** Keep `generate_leaf_module` as the leaf kernel and add the higher layer rather than smearing inter-module behavior into the existing path.
  7. After the above, revisit the motif-trait refactor (the copy-paste pattern will then cover ~7-8 block motifs, enough to extract the right abstraction).

## Recent commits
- `<pending>` — Docs: BOOK-EXAMPLES-RUNNABLE.2.1 migrate book examples to `cargo run --release --` (45 bash heads across factorization/knobs/recipes; missed=0; 9 rust sketches→rust,ignore; shorthand note; spot-runs incl. full multi-line recipe→50 .sv exit 0). Book/docs only, no code; published book now copy-paste-correct.
- `5cb6fb1` — Docs: split BOOK-EXAMPLES-RUNNABLE.2 → `.2.1` (convention migration + rust-sketch annotation, docs) / `.2.2` (extraction harness + mdbook-test + CI wiring); dependency order (correct before enforced). Tree-planning, no code.
- `38c49fb` — Docs: BOOK-EXAMPLES-RUNNABLE.1 — new quality tree + design-only DEVELOPMENT_NOTES.md entry (62-bash/8-rust/9-sv/4-text inventory; `cargo run --release --` convention; `tests/book_examples.rs` integration harness + `mdbook test` + skip-sentinel; 3 rejected alternatives; CI-wired). No code; mdbook clean.
- `8076e25` — pushed; **repo confirmed PUBLIC**, CI workflow active (first run in progress), Pages enabled (source=GitHub Actions) and the mdBook is **live at https://rdje.github.io/anvil/** (deploy run succeeded, HTTP 200). [earlier `{owner}/{repo}`-resolved `gh` query returned a stale PRIVATE; explicit `rdje/anvil` = PUBLIC.]
- `a612a5f` — Add GitHub Actions CI (.github/workflows/ci.yml — fmt/clippy/test/mdbook) + Pages (pages.yml — mdBook deploy) workflows; none existed. Workflow-config, no code.
- `ac18cd5` — Phase 6: PHASE-6-ADVANCED-MOTIFS.2.3 phase6_inferrable_memory matrix scenario + num_memory_modules metric + saw_inferrable_memory_design fact/gap (bin 216→219/864→876; non-vacuity test; no ROADMAP advance).
- `f4ee02f` — Phase 6: PHASE-6-ADVANCED-MOTIFS.2.2 memory inference structural-contract + factorization-opacity proof (64 combos: 4 strategies × 4 FactorizationLevel × 4 seeds; template equivalence + MemRead/array never in NodeId graph incl. EGraph; tool-level $mem_v2 proof = .2.1b spot-check + .2.4 gate). Proof only, no code.
- `aa9abf0` — Phase 6: PHASE-6-ADVANCED-MOTIFS.2.1b memory_prob knob + rules-first build_memory_leaf (opt-in roll after the param lane, mutually exclusive; default-off byte-identical; forced-on memory leaf validates; spot-check generated SV → 1 $mem_v2 + verilator/both-yosys clean). Closes the .2.1 container.
- `244cabd` — Phase 6: PHASE-6-ADVANCED-MOTIFS.2.1a memory IR core + opaque-stateful-leaf pipeline integration (MemId/MemKind/Memory/Node::MemRead/DepAtom::MemVirtual; ~21 match sites; load-bearing compact.rs reachability; emitter inferrable template; validator; 3 unit proofs; no generator/knob → default-off trivially byte-identical).
- `4ad089b` — Docs: split PHASE-6-ADVANCED-MOTIFS.2.1 → `.2.1a`/`.2.1b` on a discovered compaction-reachability dependency (opaque stateful leaf ≠ mechanical FlopQ-mirroring); in-flight IR edits reverted to clean `.2`-split base; tree-planning, no code.
- `c96b433` — Docs: PHASE-6-ADVANCED-MOTIFS.2 split into `.2.1`–`.2.4` signoff-sized leaves (Splitting Rules + r87; tree-planning, no code; `.3` FSM unchanged).
- `ab491a8` — Docs: PHASE-6-ADVANCED-MOTIFS.1 inferrable-memory motif design (architecture (M) `Memory` block + `MemRead` leaf; empirical Yosys `$mem_v2` probe both modes; 3 rejected alternatives; design-only, no code; frontier → `.2`).
- `957b1aa` — Phase 5b: PHASE-5B-AGGREGATES.2.4 real-gate verify + ROADMAP Phase 5b (not started)->(done) + tree closure (Phase 5b closed; next is Phase 6).
- `6fabd7e` — Phase 5b: PHASE-5B-AGGREGATES.2.3 packed_aggregate matrix scenario + `num_packed_aggregate_modules` metric + `saw_packed_aggregate_design` fact/gap (bin 213→216/852→864; non-vacuity test; no ROADMAP promotion).
- `d0d7ad6` — Phase 5b: PHASE-5B-AGGREGATES.2.2 organic-existence proof (68/80 ~85% → no rules-first pivot) + identity-invariance (signature-invariant + projected-twin dedup-collapses); proofs only, no feature code change.
- `67e909d` — Phase 5b: PHASE-5B-AGGREGATES.2.1 packed-aggregate IR annotation + `aggregate_prob` knob + boundary-alias emitter projection (default-off byte-identical; projected design verilator-clean; StructPacked/non-instantiated/non-param scoped).
- `3fbbc79` — Docs: PHASE-5B-AGGREGATES.2 split into `.2.1`–`.2.4` signoff-sized leaves (Splitting Rules + r87 no-aspirational-claims; tree-planning, no code).
- `6976346` — Docs: PHASE-5B-AGGREGATES.1 packed-aggregate emitter-projection design (architecture (P); 3 rejected alternatives; identity-invariance resolved; design-only, no code; frontier → `.2`).
- `80516ca` — Ignore Claude Code harness runtime artifacts (`.claude/scheduled_tasks.lock`, `.claude/worktrees/`, `.claude/settings.local.json`); `git status` now spotless. `.claude/settings.json` stays tracked.
- `b3c1906` — Doctrine: task-tree ownership mandatory for all code changes (live-docs + book + memory; supersedes opt-in/`rN` scope).
- `53e4c7f` — Phase 5: PHASE-5-PARAMETERIZATION.2.4b real-gate verify + ROADMAP Phase 5 (not started)->(done) + tree closure (Phase 5 closed; next is Phase 5b).
- `04b13ec` — Config: PostCompact hook re-injects SESSION_BOOTSTRAP.md after compaction.
- `1ab78f0` — Phase 5: record PHASE-5-PARAMETERIZATION.2.4a commit hash 6f87d7a.
- `6f87d7a` — Phase 5: PHASE-5-PARAMETERIZATION.2.4a phase5 matrix scenario + metrics + gap.
- `99be245` — Phase 5: record PHASE-5-PARAMETERIZATION.2.3 commit hash 2e99d6d.
- `2e99d6d` — Phase 5: PHASE-5-PARAMETERIZATION.2.3 parameter-aware identity.
- `55f83d6` — Phase 5: record PHASE-5-PARAMETERIZATION.2.2.3b commit hash 1fd53bd.
- `1fd53bd` — Phase 5: PHASE-5-PARAMETERIZATION.2.2.3b hierarchy instantiation + resolved-width validate (Phase 5 end-to-end functional).
- `ef8e095` — Phase 5: record PHASE-5-PARAMETERIZATION.2.2.3a commit hash 7950e37.
- `7950e37` — Phase 5: PHASE-5-PARAMETERIZATION.2.2.3a Instance.param_bindings + emitter #(.W(v)).
- `c5fe28d` — Phase 5: record PHASE-5-PARAMETERIZATION.2.2.2 commit hash b3c7f0c.
- `b3c7f0c` — Phase 5: PHASE-5-PARAMETERIZATION.2.2.2 rules-first parameterizable-leaf constructor.
- `862bf67` — Phase 5: record PHASE-5-PARAMETERIZATION.2.2.1 commit hash 8cc4fc4.
- `8cc4fc4` — Phase 5: PHASE-5-PARAMETERIZATION.2.2.1 soundness gate + width-generic emitter.
- `f8875c8` — Docs: record PHASE-5-PARAMETERIZATION.2.2-scope hash 8c28eae.
- `8c28eae` — Docs: PHASE-5-PARAMETERIZATION.2.2 scope refinement (soundness).
- `6646c97` — Phase 5: record PHASE-5-PARAMETERIZATION.2.1 commit hash 4cedad2.
- `4cedad2` — Phase 5: PHASE-5-PARAMETERIZATION.2.1 width-parameterization scaffold.
- `786e468` — Docs: PHASE-5-PARAMETERIZATION.1 parameterization design (architecture C chosen).
- `9f07576` — Phase 4: record Phase-4-closure commit hash 21ec427.
- `21ec427` — Phase 4: close Phase 4 done (PHASE-4-HIERARCHY.3, evidence-backed scope cut).
- `0219ac7` — Docs: PHASE-4-HIERARCHY.1 Surface Inventory (hierarchy surface landed-proven).
- `0924c8c` — Docs: register every remaining roadmap phase as a task tree (Phases 4-9).
- `750ef8b` — Phase 4: implement and gate module-dedup pass (r87, HIERARCHY-AWARE-IDENTITY.4 + .5; tree complete).
- `e4f0f04` — Phase 4: record H-A-I.3 commit hash e83efd8.
- `e83efd8` — Phase 4: dedup-pass design sketch (HIERARCHY-AWARE-IDENTITY.3).
- `555058d` — Phase 4: prove planner emits structurally-duplicate Modules (r86, HIERARCHY-AWARE-IDENTITY.2).
- `21174d8` — Docs: narrow DIFFERENTIAL-SIMULATION.1 scope to second simulator.
- `d19d427` — Quality: add cargo-llvm-cov baseline (COVERAGE-INSTRUMENTATION.1).
- `f3ee1f3` — Phase 4: add canonical module signatures (r85, HIERARCHY-AWARE-IDENTITY.1).
- `65ca372` — Docs: register three quality-improvement task trees (INSTA-SNAPSHOTS, DIFFERENTIAL-SIMULATION, COVERAGE-INSTRUMENTATION).
- `f2b95f7` — Docs: adopt FSMGen task-tree workflow (scoped to multi-slice work).
- `ed4988b` — Phase 4: prove parent-cone helper budget of 5 saturates below the top (r84).
- `da6a900` — Phase 4: prove recursive non-top registered parent-composed three-stage chain (r83).
- `69b2173` — Phase 4: close depth-7 sweep with r82 stateful mixed-support child inputs (2,2 calibrated).
- `89dadfe` — Phase 4: extend depth-7 axis with r81 stateful parent-port-composed parent outputs.
- `e8eb1a8` — Phase 4: extend depth-7 axis with r80 parent-port-composed parent outputs.
- `7bd3235` — Phase 4: extend depth-7 axis with r79 mixed-support child inputs (2,2 calibrated).
- `6f2bab0` — Phase 4: open depth-7 axis with r78 recursive parent-local flops.
- `ed4b9f3` — Phase 4: close depth-6 sweep with r77 stateful mixed-support child inputs.
- `89a406d` — Phase 4: extend depth-6 axis with r76 stateful parent-port-composed parent outputs.
- `96fdd4f` — Phase 4: extend depth-6 axis with r75 parent-port-composed parent outputs.
- `4834f98` — Phase 4: extend depth-6 axis with r74 mixed-support child inputs (2,2 calibrated).
- `88854fd` — Phase 4: open depth-6 axis with r73 recursive parent-local flops.
- `c646f50` — Phase 4: close depth-5 sweep with r72 stateful mixed-support child inputs.
- `ef6e5bd` — Phase 4: extend depth-5 axis with r71 stateful parent-port-composed parent outputs.
- `b5219b7` — Phase 4: extend depth-5 axis with r70 parent-port-composed parent outputs.
- `fa05fb5` — Phase 4: extend depth-5 axis with r69 mixed-support child inputs.
- `063f196` — Phase 4: open depth-5 axis with r68 recursive parent-local flops.
- `28144ac` — Phase 4: close depth-4 sweep with r67 stateful child-input mixed-support.
- `df49f55` — Phase 4: extend depth-4 axis with r66 stateful parent-port-composed outputs.
- `42e9678` — Phase 4: extend depth-4 axis with r65 parent-port-composed outputs.
- `cded654` — Phase 4: extend depth-4 axis with r64 mixed-support child inputs.
- `bb4d738` — Phase 4: open depth-4 axis with r63 recursive parent-local flops.
- `709bff6` — Phase 4: close depth-3 push with r62 stateful child-input mixed-support.
- `dd0940c` — Phase 4: push r61 stateful parent-port-composed outputs to depth 3.
- `fa08ccd` — Phase 4: push r60 parent-port-composed outputs to depth 3.
- `dc4fbf3` — Phase 4: push r59 mixed-support child inputs to depth 3.
- `3bc6a71` — Phase 4: push r58 recursive parent-local flops to depth 3.
- `5cdca4a` — Phase 4: gate r57 recursive non-top parent-local flops.
- `8590e43` — Phase 4: add r56 recursive stateful parent-composed mixed-support child inputs.
- `1606b08` — Phase 4: add r55 recursive stateful parent-port-composed outputs.
- `b12d732` — Phase 4: add r54 recursive parent-output coverage.
- `a11d0ec` — Phase 4: add r53 parent-composed mixed support.
- `fc373c1` — Phase 4: add r52 recursive sibling mixed support.
- `9e45d09` — Phase 4: add r51 registered sibling mixed support.
- `140c962` — Phase 4: bank registered helper mixed support.
- `a225c21` — Phase 4: bank registered multistage mixed support.
- `dbe328a` — Phase 4: bank recursive registered sibling multistage.
- `d9a5f72` — Phase 4: bank recursive registered multistage.
- `18b5a78` — Phase 4: bank recursive registered mixed support.
- `d79f69c` — Phase 4: bank recursive child-input helper budgets.
- `702ad66` — Phase 4: bank stateful helper budgets.
- `52e2004` — Phase 4: bank recursive helper budgets.
- `e107c49` — Phase 4: bank recursive stateful output helpers.
- `df9a71e` — Phase 4: bank recursive hierarchy helper routes.
- `b0b9fc8` — Phase 4: route helper child inputs through state.
- `1f57cea` — Chain registered sibling routes through parent state.
- `25abd72` — Bank Phase 4 hierarchy r25.
- `d4cb9c1` — Restore README cargo-run entrypoint.
- `c2b4118` — Bank direct helper routes in Phase 4 matrix.
- `d6ccd22` — Route direct sibling inputs via helper instances.
- `0e3e833` — Route registered sibling flops via helper instances.
- `c348884` — Spend parent-output helper budget.
- `785a143` — Align package metadata terminology.
- `f9f0288` — Refresh Phase 4 hierarchy gate budget.
- `34a420e` — Docs: align ANVIL purpose and continuity notes.
- `1f8364e` — Route registered child-input cones through helper instances.
- `7909b30` — Budget parent-cone helper instances.
- `05a6dfa` — Route parent-output cones through helper instances.
- `619f775` — Land registered hierarchy sibling routing.
- `cf3dc3c` — Clarify hierarchy parent wording.
- `87d4940` — Land hierarchy parent-local state.
- `30b1846` — Land parent-composed hierarchy child inputs.
- `8944c14` — Refresh bootstrap doc drift.
- `28c5474` — Land sibling-routed hierarchy child inputs.
- `0fc7ae7` — Land explicit Phase 4 child sourcing.
- `57eef7e` — Land exact profiled on-demand child synthesis.
- `f706232` — Refresh Phase 4 gate for mixed-depth hierarchy.
- `8f6abfc` — Land mixed-depth recursive hierarchy planning.
- `ce4327d` — Complete literal bootstrap and fix README drift.
- `1bda5c7` — Refresh Phase 4 hierarchy gate.
- `134e889` — Add per-depth recursive hierarchy controls.
- `28713a0` — Land parent-composed hierarchy tops cleanly.
- `13ef73e` — Close refreshed Phase 4 hierarchy matrix cleanly.
- `8d7795d` — Land hierarchy design metrics and control ports.
- `2eebe58` — Decouple Phase 4 child-instance planning.
- `7dae70a` — Close Phase 4 wrapper hierarchy gate cleanly.
- `747a3b3` — Start Phase 4 with a real depth-1 hierarchy slice.
- `f759403` — Close Phase 3 structured gate cleanly.
- `d8b1556` — Land selectable Slice/Concat surfaces cleanly.
- `f8aa35d` — Land casez structured mux surface.
- `e97349b` — Land case mux surface and late constant cleanup.
- `87c37d0` — Prove variable-shift surface and refresh Phase 3 docs.
- `9fcd782` — Close r20 constant-fold frontier and enter peephole.
- `4fb5761` — Close r20 associative frontier and enter constant-fold.
- `dfe3285` — Advance r20 both-mode frontier through commutative.
- `3ddcfbd` — Fold large-endpoint overshift cleanup and bank r20 frontier.
- `f0567ff` — Cap cleanup semantic proofs to tiny endpoint sets.
- `4023050` — Record fresh current-code nodeid-none frontier.
- `23fece6` — Clarify NodeId full-factorization doctrine.
- `3a4f7c9` — Fold reflexive subtraction before compare emit.
- `b41b367` — Cap exact proofs to small-support cones.
- `94bdf24` — Record fresh current-code both-mode CSE frontier.
- `248d5f2` — Budget exact small-set proofs for CSE frontier.
- `878eb4f` — Checkpoint r11 resume upgrade progress.
- `0c9b3f0` — Add resumable checkpoints to tool_matrix.
- `cd25e8e` — Advance both-mode Phase 1 frontier to 368.
- `148ee8d` — Advance both-mode Phase 1 frontier to 288.
- `e532cc9` — Record clean both-mode Phase 1 frontier.
- `bbfca1d` — Stabilize ABC-enabled Yosys harness lane.
- `f708d8d` — Advance Phase 1 gate frontier to 365 clean modules.
- `60d9883` — Record 246-module clean Phase 1 frontier.
- `739f9fe` — Tighten final proof cleanup and sequential liveness.
- `cda8bd1` — Extend exact proof shortcuts and restore associative normal form after remaps.
- `fe4dd0e` — Make Phase 1 gate explicit in tool_matrix.
- `07536df` — Close downstream warning bucket.
- `1ed22db` — Strengthen generator-side proof for constant comparisons.
- `5eba379` — Add repo-owned Verilator/Yosys tool matrix harness.
- `ca2947b` — Document broader synthesizable artifact-family roadmap.
- `58c31cc` — Activate bounded semantic gate merging at `e-graph`.
- `3cac9b6` — Add bounded semantic proofs to state identity.
- `ac243cd` — Align state identity with endpoint-aware proofs.
- `92c9ef7` — Extend state identity through self-feedback.
- `cb090be` — Map the four ANVIL steering gaps.
- `fc7ae3e` — Docs: capture structure-over-function doctrine.
- `3281e53` — Docs: capture signoff-grade bug-finder doctrine.
- `559a8be` — Fold Verilator tautology residues.
- `0a6cc89` — Validate canonical flop identity invariants.
- `420fbd4` — State identity: merge exact-signature flops.
- `033e03d` — Identity mode: make NodeId semantics first-class.
- `dd28086` — Expose peak-sharing controls and exercise live categories.
- `e973d30` — Finalise the live-signal surface for lint-clean output.
- `30753c8` — All-constant evaluation completes the constant-fold surface.
- `a75f678` — Regression tests pinning three doctrine-level invariants.
- `99084a8` — Associative-flattening opportunity metric (informational).
- `594fb51` — FAQ chapter refresh: strategies + full-factorization Q (docs only).
- `77e06af` — Sharing chapter refresh: Rule 2 + Rule 18 + Rule 21 CSE (docs only).
- `d3c0269` — Non-triviality chapter: anti-collapse rule table + factorization-ladder framing (docs only).
- `98b994f` — By-construction chapter: validator tense + Rule 18 exemplar + retry grandfather clause (docs only).
- `d6299fa` — Architecture chapter refresh: align with current workspace reality (docs only).
- `a9a96e4` — Algorithm chapter refresh: strategies, Rule 2, Rule 18, CSE, motif dispatch (docs only).
- `5847b83` — Book audit: last w_N/r_N naming remnants (docs only).
- `ea8ea39` — Construction-strategies chapter: graph-first retirement + interleaved-as-default (docs only).
- `bd27846` — Tutorial chapter refresh: naming scheme + re-captured examples (docs only).
- `0cde85f` — Knobs chapter alignment with actual config + CLI surface (docs only).
- `ee4321a` — Block counters: priority_encoder + comb_mux_encoding (last pending entries).
- `c943a30` — Combinational-depth metrics (closes another pending effectiveness-map entry).
- `c0ba963` — Live-doc catch-up: CODEBASE_ANALYSIS, USER_GUIDE, DEVELOPMENT_NOTES, ROADMAP (docs only).
- `64850da` — Operand-arity metrics (closes a pending effectiveness-map entry).
- `9e18c89` — Close residual Add/Mul/And duplicate operands at default knobs.
- `52be449` — Recipe for the factorization dial (docs only).
- `c9c2f98` — Commutative normalization + factorization-level dial.
- `5a9b477` — Operand-uniqueness knob (--operand-duplication-rate).
- `2ec33b7` — --trace debug strictly more verbose than high; off→none alias.
- `b78550d` — Zero orphans: Rule 18 enforced construction-time.
- `186db2b` — IR chapter refresh + future-extensions roadmap (docs only).
- `3af6001` — Friendly docs: quick ref, naming refresh, recipe examples (docs only).
- `7c8fa2f` — Knob measurement doctrine + effectiveness map (docs only).
- `6fb5b9b` — Structural metrics (per-module observability).
- `d2aefba` — Mux arm-duplication rate (Rule 22).
- `f425657` — Construction-time CSE with tunable AST-instance cap (Rule 21).
- `88212f7` — Emit {N{expr}} replication for same-operand Concat.
- `b533288` — UVM-style tracing (--trace / --trace-file).
- `26f90a3` — Typed per-kind naming in emitted SV (Rule 12 revised).
- `3544a0c` — N-arity anti-collapse + OR-reduce dedup (Rule 8 extended).
- `6a9daf5` — Dep-bearing source at elaboration-sensitive positions (Rule 20).
- `92d43f8` — Coefficient fits operand width (Rule 19).
- `e6850fc` — Rule 18 proposal + sample-output defect catalogue (docs only).
- `b4c489a` — Priority-encoder block (Rule 17).
- `06b5a52` — Flop-assembler unit tests + FAQ chapter.
- `1211120` — Constant comparand motif: third and final constant-role motif.
- `2da9d3d` — Constant shift-amount motif + Shl/Shr added to pick_gate.
- `7290e3d` — Linear-combination coefficient motif for Add / Sub / Mul.
- `b0f84fd` — Sub coefficient constraint: ck > 0 for all k.
- `4085401` — graph-first strategy landed; becomes the new default.
- `6d2da98` — Interleaved construction strategy: frame state machine.
- `2d038a9` — Construction-strategy machinery + shuffled strategy landed.
- `8eb03f0` — Construction-strategies chapter: 4 named strategies, graph-first default.
- `126411d` — Rule 16: cross-output sharing via the module-wide signal pool.
- `8ff1d84` — Log constants-roles clarification in the book + two corrections.
- `dde27a2` — Doctrinal fix: coefficient / shift amount / comparand are distinct motifs.
- `0564a49` — M-to-1 combinational mux as a first-class block.
- `b91188d` — N-arity for associative operators + operators-vs-blocks doctrine.
- `6cbcbff` — Q-feedback rule relaxation + structural-rules catalog.
- `bac6060` — mdBook becomes user-facing: Getting Started, Tutorial, Recipes.
- `62fdeaa` — mdBook staleness refresh: knobs, IR, algorithm, architecture.
- `c9ec12c` — CLI coverage for all Phase 1/2 motif knobs.
- `6ba646b` — Phase 2 start: per-operand DAG-cone sharing.
- `c8043c3` — Inline unit tests for cone helpers and SV emitter.
- `4eb5daa` — Per-gate width/arity validator + inline unit tests.
- `f2a3d81` — Elevate mdBook to equal-standing live doc in session recovery.
- `a1a9ea9` — Live-doc catch-up + tighten commit workflow (12-item checklist).
- `10090c2` — Encoded-select flop mux (chained ternary) alongside one-hot.
- `47675df` — M-to-1 one-hot mux flops with two motifs (ZeroDefault, QFeedback).
- `4317c82` — Fold flops into the cone recursion (single-clock synchronous design).
- `c4668a2` — Elevate "recursion is the core principle" to load-bearing status.
- `5f6022f` — Initial scaffold + Phase 1 cone-adapter hardening.

## Open questions / deferred decisions
- Exact semantic scope of NodeId-as-identity for stateful / hierarchical objects. Exact duplicate flops merge today. Broader sequential equivalence and future module-instance identity are still open, but any future strengthening must preserve the same canonical leaf endpoints rather than abstract them away.
- How to operationalize the signoff-quality bar in broader sweeps:
  exact tool versions, seed coverage, knob matrices, and what counts as
  an acceptable clean-run gate for Phase 1 / Phase 2 evidence.
- Constant-probability value — current default `0.1` is a guess; tune after Phase 1 seed sweeps.
- Whether the IR should use `typed-arena` or stay on `Vec<Node>` with `u32` indices. Current choice: plain `Vec`, because it's simple, cache-friendly, and `serde`-friendly.
- The lazy adapter currently picks the *widest* pool entry with deps. Random-among-eligible may give better motif coverage; revisit after Phase 1 metrics.
- `flop_prob` default `0.15` is a guess; calibrate after the first synthesis smoke run that reports flop counts vs gate counts.
- `max_flops_per_module` cap of `32` is conservative. May raise once metrics show generation time is not bottlenecked by D-cone draining.
- `flop_mux_encoding_prob` default `0.5` is equal-motif; no empirical data yet. Bias once synthesis metrics show which style catches more bugs.
- Ternary-over-`case` for the Encoded mux SV form — see `DEVELOPMENT_NOTES.md` rejected-alternatives; revisit when/if FSM motifs force procedural block emission.

## Known gaps vs `ROADMAP.md`
- Phase 1 exit criterion (1000 modules through Verilator + Yosys) is met locally via `/tmp/anvil-tool-matrix-phase1-real-r21/tool_matrix_report.json`, the Phase 2 sharing exit criterion is met locally via `/tmp/anvil-tool-matrix-phase2-share-r1/tool_matrix_report.json`, and the Phase 3 structured-surface gate is met locally via `/tmp/anvil-tool-matrix-phase3-structured-r4/tool_matrix_report.json`. The next real roadmap gap is therefore deeper Phase 4 hierarchy, not leaf-lane closure.
- Phase 4 hierarchy is started and the latest full downstream-clean bank is `/tmp/anvil-tool-matrix-phase4-hierarchy-r55/tool_matrix_report.json`, covering wrapper, recursive, mixed-depth recursive, explicit library-vs-on-demand child-sourcing profiles, exact profiled child-interface synthesis, sibling-routed child-input binding, registered sibling-routed child-input binding, direct registered sibling mixed-support child-input binding, recursive non-top direct registered sibling mixed-support child-input binding, recursive non-top unregistered parent-composed mixed-support child-input binding without helper instances, multi-stage registered sibling-routed child-input binding, recursive non-top multi-stage registered sibling-routed child-input binding without helpers, multi-stage direct registered sibling helper binding, multi-stage registered parent-composed helper binding, stateful parent-composed helper child-input binding, recursive non-top stateful parent-composed helper child-input binding, recursive non-top direct sibling helper binding, recursive non-top direct registered sibling helper binding, recursive non-top multi-stage direct registered sibling helper binding, recursive non-top multi-stage registered parent-composed helper binding, recursive non-top registered parent-composed helper binding, recursive non-top registered parent-composed helper mixed-support binding, recursive non-top parent-output helper binding, recursive non-top parent-output helper mixed-support binding, recursive non-top stateful parent-output helper binding, recursive non-top parent-output multi-helper budget evidence, recursive non-top child-input multi-helper budget evidence, recursive non-top stateful multi-helper budget evidence, registered parent-composed child-input binding, registered mixed-support child-input binding, recursive non-top registered mixed-support child-input binding, multi-stage registered parent-composed child-input binding, recursive non-top multi-stage registered parent-composed child-input binding without helpers, recursive non-top multi-stage registered mixed-support child-input binding without helpers, mixed parent-port / child-output parent outputs, parent-composed child-input binding, parent-cone helper-instance child-input binding, direct sibling helper binding, direct registered sibling helper binding, parent-cone helper-instance parent-output routing, stateful parent-output helper routing, budgeted parent-cone helper allocation, registered helper-sourced child-input D cones, generator-global module-name allocation, and local parent state, recursive non-top parent-port-composed parent-output routing without helpers or parent-local state, and recursive non-top stateful parent-port-composed parent-output routing without helpers at 114 scenarios / 456 designs. The roadmap gap is broader registered hierarchy routing/composition beyond the current helper-route slices and future hierarchy-aware identity.
- Parameterization is still not started.

## Session handoff notes
- All design decisions discussed so far are captured in `book/src/core-idea.md`, `book/src/why-not-grammar.md`, `book/src/non-triviality.md`, and `book/src/non-goals.md`. Read those before proposing structural changes.
- `COMMIT.md` is strict. Follow it exactly. `git_message_brief.txt` must stay untracked.
- The generator's "by construction" contract is load-bearing. Any PR that adds a generate-then-filter step (aside from the bounded retry in `cone::build_cone_with_retry`) is a design regression.
