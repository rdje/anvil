# PHASE-5-PARAMETERIZATION: Parameterized modules and instances

## Metadata

- Tree ID: `PHASE-5-PARAMETERIZATION`
- Status: `active`
- Roadmap lane: Phase 5 — Parameterization
- Created: `2026-05-16`
- Last updated: `2026-05-16` (`.2.2.3` complete — `.2.2.3a` IR/emit + `.2.2.3b` hierarchy instantiation + resolved-width validate; Phase 5 width parameterization end-to-end functional; frontier → `.2.3` parameter-aware identity)
- Owner: repo-local workflow

## Goal

Generated modules take `parameter` declarations for widths;
instantiation picks parameter values from allowed ranges;
parameter-dependent widths propagate correctly through cone generation;
and parameter-aware identity stays sound (different parameter values
never alias to one `NodeId` or one module instance unless genuinely
equivalent).

## Non-Goals

- Source-level package/typedef parameter flows for an accept corpus —
  that is Phase 8.
- Parameter-driven generate/`for` elaboration corpora with
  expected-facts manifests — that is Phase 7.
- Non-width parameters (e.g., behavioural-mode switches) beyond what
  width parameterization needs.

## Acceptance Criteria

- A concrete Phase 5 implementation plan derived from
  `book/src/ir.md` "Future extensions / Parameters and generics".
- `parameter`-bearing modules emitted, valid by construction,
  downstream-clean (Verilator + both Yosys modes).
- Parameter-aware identity proof: distinct parameter values do not
  collapse under `NodeId`/module dedup unless structurally equivalent.
- Live docs + a Phase 5 matrix gate shape.

## Task Tree

- ID: `PHASE-5-PARAMETERIZATION`
  Status: `active`
  Goal: `Deliver parameterized modules/instances with sound parameter-aware identity.`
  Children: `PHASE-5-PARAMETERIZATION.1` (done), `PHASE-5-PARAMETERIZATION.2` (active container)

- ID: `PHASE-5-PARAMETERIZATION.1`
  Status: `done`
  Goal: `Lift book/src/ir.md "Parameters and generics" into a concrete Phase 5 implementation + identity-soundness plan in DEVELOPMENT_NOTES.md (IR shape, propagation, identity rule, proof shape, rejected alternatives). Design-only.`
  Acceptance: `DEVELOPMENT_NOTES.md Phase 5 design entry with >=1 rejected alternative; no code change; mdbook clean.`
  Verification: `DEVELOPMENT_NOTES.md "Phase 5 parameterization design (2026-05-16, PHASE-5-PARAMETERIZATION.1)" entry landed: codebase-grounded (file-anchored audit of types.rs/cone.rs/validate.rs/emit/metrics/dedup/hierarchy/config), chosen architecture (C) post-construction parameterization pass + monomorphic instantiation, three rejected alternatives (A monomorphize-only, B full symbolic WidthExpr threaded, C' factorization-disable), explicit parameter-aware identity rule at canonical_module_signature, proof shape, open questions. Doc-only; no code.`
  Commit: `Docs: PHASE-5-PARAMETERIZATION.1 parameterization design`

- ID: `PHASE-5-PARAMETERIZATION.2`
  Status: `active`
  Goal: `Implement the .1 design (architecture C), default-off, downstream-clean, with parameter-aware identity. Split into signoff-sized leaves.`
  Children: `PHASE-5-PARAMETERIZATION.2.1` (done), `.2.2` (active container: `.2.2.1` done, `.2.2.2` done, `.2.2.3`), `.2.3`, `.2.4`

- ID: `PHASE-5-PARAMETERIZATION.2.1`
  Status: `done`
  Goal: `IR + emitter scaffold: WidthExpr{Lit,Param} + per-module ParamEnv{name,min,max,design_value}; post-construction parameterization pass; opt-in knob width_parameterization_prob (f64, default 0.0, serde-default pattern); module-header + parameterized-port emission. Non-parameterized output byte-identical. (Slice-boundary refinement: Instance.param_bindings moved to .2.2 — it is produced/consumed at instantiation, and adding an Instance field in .2.1 would churn 19 literal sites for a field unused until .2.2. Recorded in Decisions.)`
  Acceptance: `cargo fmt/clippy(-D warnings)/check/test green; focused proof: a parameterized module round-trips IR->validate->emit with parameter W and [W-1:0]; default-off byte-identical for fixed seeds; mdbook unaffected (no book/ change).`
  Verification: `New src/ir/param.rs::parameterize_module (annotation-only post-construction pass; 5 unit tests). WidthExpr{Lit,Param}+ParamEnv added to src/ir/types.rs (Module additive: param_env/parameterized_input_ports/parameterized_output_ports — all Default, zero churn to ..Module::default() sites). Config::width_parameterization_prob (serde-default 0.0 + probability-range validation). Wired default-off in generate_design after dedup. Emitter: #( parameter int W = D ) header + param_width_decl rendering [W-1:0] on parameterized ports. Focused proof width_parameterization_round_trips_and_is_default_off (8 seeds, default-off byte-identical + forced-on round-trip). cargo fmt/clippy -D warnings clean; lib 205/0 (200 + 5 new); full cargo test green (see Verification Log).`
  Commit: `Phase 5: PHASE-5-PARAMETERIZATION.2.1 width-parameterization scaffold`

- ID: `PHASE-5-PARAMETERIZATION.2.2`
  Status: `active`
  Goal: `Sound, *actually-firing* width parameterization. Split because the soundness primitives, the rules-first constructor that makes the feature non-inert, and instantiation substitution are independently reviewable.`
  Children: `PHASE-5-PARAMETERIZATION.2.2.1`, `.2.2.2`, `.2.2.3`

- ID: `PHASE-5-PARAMETERIZATION.2.2.1`
  Status: `done`
  Goal: `Soundness primitives (no behaviour change when default-off). (a) is_width_generic gate in src/ir/param.rs: only parameterize a width-homogeneous combinational leaf (no flops/instances; every port/node width == design; no Constant; no Slice/Concat/ForFold — Mux/compare auto-excluded via width-1 nodes). (b) Emitter renders ALL width-homogeneous sites ([W-1:0] for internal gate/instance-output wires + flops, not just ports) so a parameterized body is fully width-generic, never leaking a concrete [D-1:0].`
  Acceptance: `cargo fmt/clippy(-D warnings)/check/test green; param.rs unit tests prove the gate (homogeneous accepted, mixed-width / constant declined, idempotent); focused proof: default-off byte-identical AND any parameterized module is width-generic with no concrete [D-1:0] leak. No organic-existence claim (that is .2.2.2).`
  Verification: `is_width_generic gate + param_width_decl_w emitter helper landed; param.rs 6/0 unit tests; focused proof width_parameterization_is_default_off_and_emits_width_generic_bodies passes; cargo fmt/clippy -D warnings clean; full cargo test (see Verification Log). No book/ change.`
  Commit: `Phase 5: PHASE-5-PARAMETERIZATION.2.2.1 soundness gate + width-generic emitter`

- ID: `PHASE-5-PARAMETERIZATION.2.2.2`
  Status: `done`
  Goal: `Rules-first parameterizable-leaf constructor. The unconstrained cone generator essentially never produces a width-homogeneous module, so a post-hoc soundness filter is INERT and is the generate-then-filter anti-pattern the project forbids. Instead, when width_parameterization_prob fires for a module, *construct* it width-homogeneously by rule (single design width; only width-preserving same-width gates; no Constant/Slice/Concat/ForFold/Mux/compare), valid by construction. The .2.2.1 gate then always accepts it (cheap post-construction assertion, never a filter).`
  Acceptance: `Focused proof: forced-on generation reproducibly yields parameterized width-generic modules (organic existence now holds); validate_design passes; emitted body fully [W-1:0]; default-off still byte-identical; all four ConstructionStrategy values; cargo gates green.`
  Verification: `New src/gen/module.rs::build_parameterizable_leaf (rules-first valid-by-construction width-homogeneous combinational leaf: W>=2, 2..4 inputs / 1..3 outputs all width W, each output one N-arity Xor/And/Or/Add over all inputs via m.intern_gate, no clk/rst_n/flops/instances/constants). Single opt-in roll added at the top of generate_leaf_module_with_interface_profile (interface_profile None only). param.rs refactored: parameterize_module (rolling) -> annotate_parameterized (non-rolling); generate_design post-pass now non-rolling (no double-roll). Focused proof rewritten: at prob 1.0 EVERY single-module design is a parameterized width-generic leaf across all 4 ConstructionStrategy values, validate_design passes, body fully [W-1:0], no concrete [D-1:0]; default-off byte-identical. param.rs 5/0; cargo fmt/clippy -D warnings clean; full cargo test (Verification Log).`
  Commit: `Phase 5: PHASE-5-PARAMETERIZATION.2.2.2 rules-first parameterizable-leaf constructor`

- ID: `PHASE-5-PARAMETERIZATION.2.2.3`
  Status: `done`
  Goal: `Instantiation substitution. Split per the Splitting Rules (mixes an IR field + 19 literal sites, an emitter change, a generator change, and a validator change — independently reviewable).`
  Children: `PHASE-5-PARAMETERIZATION.2.2.3a` (done), `PHASE-5-PARAMETERIZATION.2.2.3b` (done)

- ID: `PHASE-5-PARAMETERIZATION.2.2.3a`
  Status: `done`
  Goal: `IR + emit: add Instance.param_bindings: Vec<(String,u32)> (Instance has no Default → update all ~19 literal sites with param_bindings: Vec::new(), driven by the compiler's missing-field errors for completeness). Emitter: when an instance's param_bindings is non-empty, emit child #(.NAME(v), ...) inst (...); empty bindings → byte-identical (no instance #(...)).`
  Acceptance: `cargo fmt/clippy(-D warnings)/check/test green; focused proof: a hand-built Design with an instance carrying param_bindings emits #(.W(v)); an instance with empty bindings emits no instance #(. Default-off byte-identical. No hierarchy/validate semantics yet.`
  Verification: `Instance.param_bindings: Vec<(String,u32)> added to src/ir/types.rs; all 19 literal sites updated with param_bindings: Vec::new() (compiler missing-field errors as the completeness oracle — cargo build --all-targets clean confirms all 19). src/emit/sv.rs instance emission: non-empty bindings → "child #(.NAME(v), ...) inst (", empty → byte-identical "child inst (". Focused unit test instance_with_param_bindings_emits_parameter_override_list (one instance with [("W",8)] → "child #(.W(8)) u_0 (", one empty → "child u_1 ("). cargo fmt/clippy -D warnings clean; emit:: suite 18/0; full cargo test (Verification Log). No book/ change.`
  Commit: `Phase 5: PHASE-5-PARAMETERIZATION.2.2.3a Instance.param_bindings + emitter #(.W(v))`

- ID: `PHASE-5-PARAMETERIZATION.2.2.3b`
  Status: `done`
  Goal: `Hierarchy instantiation + resolved-width validate. In src/gen/hierarchy.rs, when a selected child has param_env, pick an in-range value reproducibly via g.rng, record Instance.param_bindings, and bind/route child ports at the RESOLVED width. src/ir/validate.rs: parameterized child-port width checks use the instance's resolved width, not the template design_value.`
  Acceptance: `Focused proof: a parent instantiates one parameterizable template at >=2 distinct in-range values; validate_design passes; emitted SV carries #(.W(v)) per instance; all four ConstructionStrategy values; default-off byte-identical; cargo gates green.`
  Verification: `Soundness scoping: only the legacy-wrapper planned-child loop (generate_parent_module) picks an override; helper / default instantiations leave param_bindings empty → child elaborates at default W=design_value = its concrete template = already valid (no change needed there). resolved_child_port_width helper in src/gen/hierarchy.rs; per-instance g.rng pick from [env.min,env.max] (None / no draw when child not parameterized → byte-identical); resolved width threaded through child-input binding, InstanceOutput node, parent pools, top output ports. src/ir/validate.rs: resolved_child_width closure makes ChildInput/OutputWidthMismatch compare against the instance's override for parameterized ports. Focused proof width_parameterization_instances_override_at_multiple_values (legacy wrapper, library mode, 1 leaf × 6 instances, 4 ConstructionStrategy × 4 seeds): every parameterized-child instance carries a W binding in [2,8], emitted SV has #(.W(v)) per instance, >=2 distinct override values across the sweep (multi-width reuse), validate_design passes; default-off byte-identical (no instance #(). cargo fmt/clippy -D warnings clean; phase5 proofs 2/2; full cargo test (Verification Log). No book/ change.`
  Commit: `Phase 5: PHASE-5-PARAMETERIZATION.2.2.3b hierarchy instantiation + resolved-width validate`

- ID: `PHASE-5-PARAMETERIZATION.2.3`
  Status: `pending`
  Goal: `Parameter-aware identity: parameterized width sites hash a normalized symbolic form in canonical_module_signature (src/metrics.rs); non-parameterized sites unchanged. dedup_modules unchanged.`
  Acceptance: `Identity proof: same template at W=8 and W=16 -> one signature (dedup collapses them); a concrete width-8 module keeps a distinct signature (extends dedup_is_a_no_op_when_modules_are_structurally_distinct). cargo gates green.`
  Verification: `pending`
  Commit: `pending`

- ID: `PHASE-5-PARAMETERIZATION.2.4`
  Status: `pending`
  Goal: `Matrix gate + Phase 5 closure: opt-in phase5 focus config sweeping the param range, saw_width_parameterized_design coverage fact + gap, downstream-clean (Verilator + both Yosys); add explicit ROADMAP Phase 5 exit criteria; promote Phase 5 -> done; sync README/CODEBASE_ANALYSIS/MEMORY/book.`
  Acceptance: `Phase 5 matrix scenario coverage_gaps=[] and downstream-clean at >=2 swept param values; ROADMAP Phase 5 exit criteria authored + label done; tree -> done.`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `PHASE-5-PARAMETERIZATION.2.3` | `pending` | `.2.2.*` done — Phase 5 width parameterization is end-to-end functional (multi-width `#(.W(v))` reuse, valid by construction). Parameter-aware identity is the next correctness layer (a parameterized template must be one identity across its legal range). |
| 2 | `PHASE-5-PARAMETERIZATION.2.4` | `pending` | Matrix gate + Phase 5 closure; depends on `.2.3`. |

## Decisions

- `2026-05-16` (**rules-first pivot, found in `.2.2.1`**): a 64-seed
  forced-on sweep produced **zero** organically width-homogeneous
  modules — the unconstrained cone generator essentially never emits
  one. Gating parameterization on "did the random generator happen to
  produce a homogeneous module" is therefore both **inert** (the
  feature would never fire on real output) and the
  **generate-then-filter anti-pattern the project doctrine explicitly
  forbids** (rules-first construction, not post-hoc filtering).
  **Decision:** Phase 5 must *construct* width-homogeneous
  parameterizable modules **by rule** (new leaf `.2.2.2`), valid by
  construction; the `.2.2.1` `is_width_generic` gate is retained only
  as a cheap post-construction *assertion* (always satisfied by the
  constructor), never as a filter. `.2.2` was split into `.2.2.1`
  (soundness primitives, done), `.2.2.2` (rules-first constructor),
  `.2.2.3` (instantiation substitution). No node renumbered; `.2.2`
  became a container.
- `2026-05-16`: Split design (`.1`, unblocked) from implementation
  (`.2`, blocked by Phase 4) so Phase 5 thinking could advance in
  parallel without violating the roadmap's hard Phase 4 prerequisite.
- `2026-05-16` (`.1` outcome): chose architecture **(C)
  post-construction parameterization pass + monomorphic instantiation**
  over (A) monomorphize-only, (B) full symbolic `WidthExpr` threaded
  through construction, and (C') factorization-disable. Rationale and
  rejected alternatives in `DEVELOPMENT_NOTES.md` "Phase 5
  parameterization design". (C) preserves valid-by-construction with
  zero changes to the invasive width-arithmetic code and keeps the
  full-factorization doctrine intact; (B) is recorded as the strict
  follow-on extension (its algebra is a superset of (C)'s
  `WidthExpr{Lit,Param}` seed).
- `2026-05-16`: Phase 4 is `done`, so `.2` is unblocked. `.2` split
  into `.2.1`–`.2.4` (scaffold → instantiation → identity → gate)
  because parameterization cannot reach signoff in one slice.
- `2026-05-16` (`.2.1` outcome): the scaffold is **annotation-only** —
  `param_env` + parameterized-port-id lists are *additive* `Module`
  fields (all `Default`, so zero churn to the ~121 `..Module::default()`
  sites and no change to the load-bearing `width: u32` IR fields). The
  body stays concrete; only the emitter (`#( parameter int W = D )` +
  `param_width_decl` → `[W-1:0]`) and (later, `.2.3`) the identity
  signature consult the annotation. Confirms architecture (C) is
  implementable with zero changes to the invasive width-arithmetic
  code, exactly as the `.1` design predicted.
- `2026-05-16` (**soundness refinement, found entering `.2.2`**):
  monomorphic bodies make instantiating a parameterized module at a
  value ≠ `design_value` *unsound* unless the emitted body is genuinely
  width-generic. Architecture (C) is kept sound — without resorting to
  (B)'s full symbolic width arithmetic — by restricting parameterization
  to **width-homogeneous** modules: a module is parameterizable only if
  *every* port, node and flop width equals the design value, and the
  emitter renders *all* those sites (not just ports) as `[W-1:0]`. Then
  the single emitted body text is literally correct for every `W`
  (only width-preserving same-width logic; any constant / `Slice` /
  `Concat` / `ForFold` / mixed-width site disqualifies the module). This
  stays construction-time sound (a generator rule, no
  generate-then-filter) and does not weaken factorization. `.2.1`'s
  pass (port-anchored) is tightened to this rule in `.2.2`; `.2.1`'s
  default-off byte-identical guarantee is unaffected.
- `2026-05-16`: **Slice-boundary refinement.** `Instance.param_bindings`
  was moved from `.2.1` to `.2.2`. Reason: adding a non-`Default`
  field to `Instance` would churn 19 literal-construction sites for a
  field that is only produced/consumed at instantiation (`.2.2`).
  Keeping `.2.1` a Module-only additive scaffold is cleaner and the
  field lands where it is first used. IDs unchanged; no renumbering.

## Open Questions

- Resolved by `.1`: parameter-aware identity is implemented **as a
  modification of `canonical_module_signature`** (hash the normalized
  symbolic form at parameterized sites), not a separate guard;
  `dedup_modules` needs no change. Recorded in the design entry.
- Whether Phase 5 gets its own `ScenarioSet::Phase5` or rides the
  Phase 4 design harness for the first gate. Owner: `.2.4`. Lean: ride
  Phase 4 harness first. Does not block `.2.1`–`.2.3`.

## Blockers

- None. Phase 4 is `done`; the `.2.x` frontier is unblocked.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-05-16` | `PHASE-5-PARAMETERIZATION.1` | DEVELOPMENT_NOTES.md design entry landed (codebase-grounded; architecture C chosen; 3 rejected alternatives; identity rule; proof shape). Doc-only, no code; `mdbook build book` clean. | Done. |
| `2026-05-16` | `PHASE-5-PARAMETERIZATION.2.1` | `cargo fmt --all -- --check` clean; `cargo clippy --all-targets -- -D warnings` clean; `cargo test --lib` 205/0; focused proof (8 seeds); full `cargo test` green (CARGO_TEST_EXIT=0). No `book/` change. | Done (`4cedad2`). |
| `2026-05-16` | `PHASE-5-PARAMETERIZATION.2.2.1` | `is_width_generic` gate + `param_width_decl_w` emitter; `param.rs` 6/0; focused proof; full `cargo test` green. | Done (`8cc4fc4`). |
| `2026-05-16` | `PHASE-5-PARAMETERIZATION.2.2.2` | `build_parameterizable_leaf` rules-first constructor + non-rolling `param.rs` refactor; `param.rs` 5/0; focused proof (4 strategies); full `cargo test` green. | Done (`b3c7f0c`). |
| `2026-05-16` | `PHASE-5-PARAMETERIZATION.2.2.3a` | `Instance.param_bindings` field + 19 sites (compiler-driven completeness); emitter `#(.NAME(v), …)`; focused unit test. emit:: 18/0; full `cargo test` green. | Done (`7950e37`). |
| `2026-05-16` | `PHASE-5-PARAMETERIZATION.2.2.3b` | `resolved_child_port_width` helper + per-instance `g.rng` pick in `generate_parent_module` (None when not parameterized → byte-identical); resolved width threaded through child-input binding / InstanceOutput / pools / top output ports; `validate.rs` `resolved_child_width` closure for parameterized child-port checks. Soundness scoping: only the planned-child loop picks an override (helper/default → empty bindings → default elaboration = template = already valid). Focused proof `width_parameterization_instances_override_at_multiple_values` (legacy wrapper, library mode, 1 leaf × 6 instances, 4 strategies × 4 seeds): per-instance `#(.W(v))`, ≥2 distinct values, `validate_design` passes; default-off byte-identical. `cargo fmt`/`clippy -D warnings` clean; phase5 proofs 2/2; full `cargo test` (COMMIT.md gate). No `book/` change. | Done. |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `PHASE-5-PARAMETERIZATION.1` | `Docs: PHASE-5-PARAMETERIZATION.1 parameterization design` (`786e468`) | Design-only; DEVELOPMENT_NOTES.md entry. |
| `PHASE-5-PARAMETERIZATION.2.1` | `Phase 5: PHASE-5-PARAMETERIZATION.2.1 width-parameterization scaffold` (`4cedad2`) | IR+config+pass+emitter+focused proof; annotation-only, default-off byte-identical. |
| `PHASE-5-PARAMETERIZATION.2.2.1` | `Phase 5: PHASE-5-PARAMETERIZATION.2.2.1 soundness gate + width-generic emitter` (`8cc4fc4`) | Soundness primitives; rules-first pivot found here. |
| `PHASE-5-PARAMETERIZATION.2.2.2` | `Phase 5: PHASE-5-PARAMETERIZATION.2.2.2 rules-first parameterizable-leaf constructor` (`b3c7f0c`) | Constructor makes the feature fire by construction; param.rs refactored non-rolling. |
| `PHASE-5-PARAMETERIZATION.2.2.3a` | `Phase 5: PHASE-5-PARAMETERIZATION.2.2.3a Instance.param_bindings + emitter #(.W(v))` (`7950e37`) | IR field + 19 sites + instance override emission; no hierarchy/validate semantics yet. |
| `PHASE-5-PARAMETERIZATION.2.2.3b` | `Phase 5: PHASE-5-PARAMETERIZATION.2.2.3b hierarchy instantiation + resolved-width validate` | Closes `.2.2.3` and the `.2.2` container; Phase 5 width parameterization end-to-end functional. |

## Changelog

- `2026-05-16`: Created task tree as part of the directive to task-tree
  every remaining roadmap phase.
- `2026-05-16`: `.1` design landed (architecture (C) chosen; rejected
  (A)/(B)/(C')). Phase 4 reached `done`, unblocking `.2`; `.2` split
  into `.2.1`–`.2.4` (scaffold → instantiation → identity → gate).
  Frontier → `.2.1`.
- `2026-05-16`: `.2.1` scaffold landed (`4cedad2`) — `WidthExpr` +
  `ParamEnv` + additive `Module` annotation fields, opt-in
  `Config::width_parameterization_prob` (default 0.0), new
  `src/ir/param.rs` post-construction pass, emitter parameter header +
  `[W-1:0]`, focused round-trip / default-off proof.
- `2026-05-16`: `.2.2.1` soundness primitives landed —
  `is_width_generic` gate (width-homogeneous combinational leaf only)
  + `param_width_decl_w` so a parameterized body is fully
  width-generic. **Rules-first pivot found here:** a 64-seed sweep
  produced zero organically homogeneous modules, so post-hoc filtering
  is inert + generate-then-filter. `.2.2` split into `.2.2.1` (done),
  `.2.2.2` (rules-first constructor — makes the feature fire),
  `.2.2.3` (instantiation substitution). Decision recorded. Frontier →
  `.2.2.2`.
- `2026-05-16`: `.2.2.2` rules-first constructor landed —
  `src/gen/module.rs::build_parameterizable_leaf` builds a
  width-homogeneous combinational leaf by construction; single opt-in
  roll in `generate_leaf_module_with_interface_profile`; `param.rs`
  refactored from rolling `parameterize_module` to non-rolling
  `annotate_parameterized` (post-pass no longer double-rolls). Focused
  proof: at prob 1.0 every forced-on single-module design across all 4
  ConstructionStrategy values is a parameterized width-generic leaf,
  validates, emits a fully `[W-1:0]` body, default-off byte-identical.
  The feature now fires by construction. Frontier → `.2.2.3`
  (instantiation substitution).
- `2026-05-16`: `.2.2.3` split per the Splitting Rules into `.2.2.3a`
  (IR field + emitter) and `.2.2.3b` (hierarchy pick + resolved-width
  validate). `.2.2.3a` landed — `Instance.param_bindings:
  Vec<(String,u32)>` added; all 19 `Instance` literal sites updated
  (compiler missing-field errors as the completeness oracle, `cargo
  build --all-targets` clean); `src/emit/sv.rs` emits
  `child #(.NAME(v), …) inst (` for non-empty bindings and the
  byte-identical `child inst (` for empty; focused unit test. Frontier
  → `.2.2.3b`.
- `2026-05-16`: `.2.2.3b` landed — closes `.2.2.3` and the `.2.2`
  container. `src/gen/hierarchy.rs` `generate_parent_module` picks a
  per-instance in-range override via `g.rng` for parameterizable
  children and threads the resolved width through binding /
  InstanceOutput / pools / top output ports; `src/ir/validate.rs`
  resolves parameterized child-port widths from the instance's
  override. Soundness scoping: only the planned-child loop overrides
  (helper / default instances keep empty bindings → default
  elaboration = concrete template = already valid; no change needed).
  Focused proof `width_parameterization_instances_override_at_multiple_values`
  passes (multi-width `#(.W(v))` reuse, `validate_design` clean, ≥2
  distinct values, default-off byte-identical). **Phase 5 width
  parameterization is end-to-end functional.** Frontier → `.2.3`
  (parameter-aware identity).
