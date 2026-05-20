# PHASE-8-FRONTEND-ACCEPT: Frontend/elaboration accept corpora

## Metadata

- Tree ID: `PHASE-8-FRONTEND-ACCEPT`
- Status: `active`
- Roadmap lane: Phase 8 — Frontend/elaboration accept corpora
- Created: `2026-05-16`
- Last updated: `2026-05-20` (**`.2c.2a` yosys extractor + end-to-end-runnable `#[ignore]` landed** — `tests/frontend_parity.rs` extended with `parse_yosys_binary_param` (signed-32-bit sign-extension; symmetric to Phase 7 even though Phase 8's builder doesn't currently emit negatives) + `yosys_hierarchy_write_json_to_tool_report(json, seed)` (reads `.parameter_default_values` → `top_params`; reads `.cells[<inst>].{type, parameters}` → `instances` with `child_module` + `resolved_bindings`; reads `.netnames` key prefix `g_taken.`/`g_else.` → `generate_branches["g_taken"]`; folded axes stay empty) + `yosys_hierarchy_scope()` (`only(&[Seed, Top, TopParams, Instances, GenerateBranches])`) + the rewritten end-to-end-runnable `parity_against_real_yosys_hierarchy_write_json` `#[ignore]` test (`hierarchy -top` only — NO `proc; opt`, per the .2c.2 Decisions probe). 2 new extractor proofs (12 portable + 2 ignored): `yosys_extractor_reads_a_synthetic_hierarchy_write_json_correctly`, `yosys_extractor_reports_g_else_when_else_branch_survives`. Full `cargo test` green; portable stays green tool-less; frontier → `.2c.2b`)
- Owner: repo-local workflow

## Goal

Add a source-level artifact family of **compact elaboratable
hierarchies** (not gate-level circuit-IR leaf modules): ANSI ports /
parameter lists, parameter/localparam flows, module instantiation
variants (named/ordered overrides, named/ordered/wildcard ports,
instance arrays), package imports and package-qualified constants/types,
typedef-backed types/structs/unions/enums/atoms, the full `assign` /
`always_comb` / `always @(*)` / `always_ff` / `always_latch` set, and
generate `if`/`for` — backed by a **source-level parameter/hierarchy/
package IR** and an expected-facts manifest.

## Non-Goals

- Forcing this family through the existing gate-level circuit IR; Phase
  8 explicitly introduces a *source-level* IR.
- Behavioural correctness of the elaborated design beyond the declared
  expected elaboration facts.
- The cross-lane selector — that is Phase 9.

## Acceptance Criteria

- A source-level parameter/hierarchy/package IR distinct from the
  circuit IR.
- Reproducible 1–3 module accept corpora with clear tops and
  expected-elaboration-fact manifests.
- Downstream parity checks against those facts.

## Task Tree

- ID: `PHASE-8-FRONTEND-ACCEPT`
  Status: `active`
  Goal: `Source-level elaboratable accept corpora with a dedicated source IR and expected-facts parity.`
  Children: `PHASE-8-FRONTEND-ACCEPT.1` (done), `PHASE-8-FRONTEND-ACCEPT.2` (active container: `.2a`, `.2b`, `.2c`)

- ID: `PHASE-8-FRONTEND-ACCEPT.1`
  Status: `done`
  Goal: `Design the source-level parameter/hierarchy/package IR and the accept-corpus expected-facts schema in DEVELOPMENT_NOTES.md / book: why a separate IR, what surfaces it must express, manifest schema, parity harness, rejected alternatives. Design-only.`
  Acceptance: `Design entry with source-IR sketch + manifest schema + >=1 rejected alternative; mdbook clean; no code change.`
  Verification: `DEVELOPMENT_NOTES.md "Phase 8 frontend/elaboration accept-corpus source-IR design (2026-05-18, PHASE-8-FRONTEND-ACCEPT.1)" entry landed. The shift (Phases 1-6 already-elaborated DUT RTL; Phase 7 single-module const-expr oracle; Phase 8 = compact elaboratable HIERARCHIES emitted with parameters UNRESOLVED in the SV text + a manifest of what elaboration must resolve — pressure point = downstream front-end/elaboration). Codebase grounding (post-elaboration scalar circuit IR cannot express modules/param-ports/packages/typedef/generate; Phase 5 ParamEnv & Phase 7 const-expr DAG are sub-models; Phase 8 = first-class source-level AST IR, separate generator path, reuses Phase 7 evaluator+manifest core + seeding/CLI). Source-IR sketch (SourceUnit/Package/Module{params,ports,items}; ModuleItem = Localparam|VarDecl|Typedef|ContinuousAssign|Always(kind)|Instance{params Named|Ordered, ports Named|Ordered|Wildcard, array}|Generate(If|For); Type = Logic|Atom|Enum|Struct|Union|Named|PkgQual; Expr = reused Phase 7 set; params carry construction-time-evaluated values). Manifest extends Phase 7's schema with the instance tree (path→target→resolved child params→port bindings), selected generate branches/iterations, package+typedef resolutions; byte-stable JSON. Oracle-by-construction (generator elaborates at construction time; emits un-elaborated SV + elaborated-facts manifest from the same knowledge; no analysis pass/re-parse/bundled elaborator). Open Question resolved (reuse Phase 7 evaluator+manifest core, extend schema; .2 depends on PHASE-7-ORACLE-MICRODESIGN.2's core; Phase 9 unifies the selector — Phase 8 behind an explicit family flag). Hierarchy-aware parity harness (repo-owned gate + cargo structural-consistency slice). 4 rejected alternatives (reuse circuit IR / emit already-elaborated SV / in-ANVIL SV elaborator / extend Phase 7 const-expr IR in place). .2 proof shape + split. Design-only; no code; mdbook build book clean; cargo fmt --all --check clean; full cargo test green at base f0cff2c (no src/tests touched).`
  Commit: `Docs: PHASE-8-FRONTEND-ACCEPT.1 source-level frontend/elaboration accept-corpus IR design`

- ID: `PHASE-8-FRONTEND-ACCEPT.2`
  Status: `active`
  Goal: `Implement the source-level IR + accept-corpus generator + manifest + parity harness per .1, behind the artifact-family selector, with a parity gate. Split per the Splitting Rules along the exact independently-reviewable boundaries .1's design named (source-level AST IR + construction-time elaboration-evaluator / un-elaborated-SV emitter + elaborated-facts manifest emitter / hierarchy-aware parity harness + repo-owned gate) — exactly mirroring the proven PHASE-7-ORACLE-MICRODESIGN.2 -> .2a/.2b/.2c decomposition that closed Phase 7 on 2026-05-20. Each child is separately reviewable and .2a's elaboration evaluator + .2b's manifest core extend the Phase 7 evaluator/manifest core (the reuse PHASE-9-MULTI-ARTIFACT-UMBRELLA's L1-wrap migration depends on).`
  Children: `PHASE-8-FRONTEND-ACCEPT.2a` (done), `PHASE-8-FRONTEND-ACCEPT.2b` (done), `PHASE-8-FRONTEND-ACCEPT.2c` (active container: `.2c.1` done, `.2c.2` active container: `.2c.2a`, `.2c.2b`)

- ID: `PHASE-8-FRONTEND-ACCEPT.2a`
  Status: `done`
  Goal: `Source-level AST IR + construction-time elaboration-evaluator (the oracle). A new separate top-level module src/srcform/ (or src/frontend/; final name TBD in implementation; NOT in src/ir/ — circuit IR cannot express modules/packages/typedef/generate, per .1's category-error rejection): SourceUnit{packages, modules}, Package{name, items}, Module{name, params, ports, items}, ModuleItem = Localparam{name, expr, value} | VarDecl{name, ty, init} | Typedef{name, ty} | ContinuousAssign{lhs, rhs} | Always{kind, body} | Instance{module, params (Named|Ordered), ports (Named|Ordered|Wildcard), array} | Generate(If{cond, then, else} | For{var, init, cond, step, body}), Type = Logic{packed_width} | Atom{name: int|byte|bit|...} | Enum{base, members} | Struct{kind: Packed|Unpacked, fields} | Union{kind: Packed|Unpacked, fields} | Named(String) | PkgQual{pkg, name}, Expr = reuse Phase 7's ConstExpr set (cross-tree reuse). Construction-time elaboration-evaluator: traverses the SourceUnit and resolves every parameter value, typedef instance, generate condition, instance-path port binding, and array dimension; produces an in-memory ElaboratedFacts struct that mirrors .1's manifest schema (the oracle). Reproducible rules-first build_acceptable_unit(seed, knobs) builder (ChaCha8::seed_from_u64, project convention, no thread_rng) — a literal-root package, a top module with N parameters, M sub-instances with both Named and Ordered param/port styles, K generate branches, and L typedef references; resolved in place. Reuses Phase 7's eval/resolve for the ConstExpr layer; no analysis pass, no re-parse — builder IS the oracle. Unit-proven: evaluator's resolved facts match independent reference values; reproducible byte-stable IR for fixed seeds. No SV/manifest emit (that is .2b), no harness (that is .2c).`
  Acceptance: `cargo fmt/clippy(-D warnings)/check --all-targets/test green; new src/srcform/ (or final name) module landed with the source-level AST IR + construction-time elaboration-evaluator + reproducible rules-first builder + unit proofs (elaboration correctness on a curated set incl. nested generate, named-vs-ordered port maps, typedef chains, array instances; reproducibility for fixed seeds, seed-sensitivity); no emit/harness; no ROADMAP advance; no book/ change.`
  Verification: `New separate top-level module src/frontend/mod.rs registered via pub mod frontend in src/lib.rs (deliberately NOT in src/ir/ — the circuit IR cannot express modules/params/packages/typedef/generate, exactly the category-error .1 rejected). AST IR types: SourceUnit{seed, packages: Vec<Package>, children: Vec<Module>, top: Module} (the minimum-viable shape: depth-1 instance tree — enough to stress every elaboration axis the parity gate checks; deeper trees are a recorded post-.2a knob in .2b's emit work, NOT a .2a blocker), Package{name, items: Vec<PackageItem>}, PackageItem::Localparam(ParamDecl) (minimum-viable set; .2b may add Typedef), Module{name, params: Vec<ParamDecl>, body: Vec<ModuleItem>}, ParamDecl{name, kind: ParamKind (reused from microdesign cross-tree), expr: ConstExpr (reused), value: i128 (the oracle)}, ModuleItem::Localparam(ParamDecl) | Instance(Instance) | GenerateIf(GenerateIf), Instance{inst_name, child_module, param_bindings: Vec<ParamBinding>} (named-binding form only in .2a; ordered is a .2b extension knob), ParamBinding{name, expr: ConstExpr, resolved: i128 (the per-instance oracle)}, GenerateIf{label, else_label, condition: ConstExpr, taken: bool (the oracle), then_branch/else_branch: Vec<ModuleItem>}. Every type derives Debug+Clone+PartialEq+Eq so the reproducibility proof can compare two builds for byte identity and the manifest-mirrors-oracle proof can compare resolved fact maps for equality. Cross-tree reuse: use crate::microdesign::{eval, BinOp, ConstExpr, EvalError, ParamKind} — Phase 7's ConstExpr/eval are the expression layer for parameter defaults, localparam chains, instance bindings, and generate predicates (per .1's full-factorization plan). Construction-time elaboration-evaluator: pub fn elaborate(unit: &mut SourceUnit) -> Result<BTreeMap<String, i128>, EvalError> walks (1) package localparams (resolved values populate the pkg::name namespace), (2) top module parameter ports (literal defaults; .2a's builder doesn't override — instance bindings are one level down), (3) top module body items (Localparams extend the env in declaration order; Instance param_bindings resolve in the PARENT's env and populate ParamBinding.resolved; GenerateIf.taken = eval(condition) != 0 with then_branch elaborating in the env, else_branch in a sandboxed clone so it doesn't leak — SV's model). The builder IS the oracle: every .value/.resolved/.taken is set in place; downstream readers (emit, manifest, comparator) read them directly without re-evaluating. Reproducible rules-first builder pub fn build_acceptable_unit(seed: u64, n_params: usize, n_children: usize) -> SourceUnit: one ChaCha8Rng::seed_from_u64(seed) drives everything (project convention; no thread_rng); package acc_<seed>_pkg with one localparam int K = (seed % 32) + 1; child module child_<seed> with n_params parameters (literal defaults via g.rng); top module acc_<seed> with n_params parameter ports + n_params body localparams (chained: L0 references P0; Li references L<i-1>; ±small literal) + n_children Instance(s) of child_<seed> with named bindings (each binds CP<i> to a parent-evaluated Add of a top-param-or-localparam ref plus a small offset) + one GenerateIf with condition P0 >= acc_<seed>_pkg::K. Resolved in place via elaborate(). 4 unit proofs green: build_acceptable_unit_has_the_documented_shape (smoke: one package, one child, one top, n_params/n_children/lp counts match), unit_is_reproducible_and_seed_sensitive (same (seed, shape) → byte-identical SourceUnit across rebuilds for seeds {0,1,7,42,12345}; distinct seeds differ — the load-bearing reproducibility invariant the emitters and parity gate depend on), elaboration_evaluator_resolves_every_axis (package K positive; literal-rooted top params resolve to their literal; localparams re-eval consistently in the prefix env; GenerateIf.taken matches a fresh eval of the condition), elaborated_facts_match_a_fresh_reeval_across_the_seed_set (the Phase-8 counterpart of Phase 7's stored_values_are_consistent_with_a_fresh_reeval load-bearing oracle-no-drift invariant: every stored ParamDecl.value / ParamBinding.resolved / GenerateIf.taken equals a fresh eval against the reconstructed env, across seeds 0..=8 — covers ALL fact axes the manifest will carry). cargo fmt --all (re-sorted the pub mod declarations alphabetically — frontend now sits between emit and gen in src/lib.rs) / clippy --all-targets -- -D warnings / check --all-targets clean. Full cargo test green: lib 233 passed (was 229 + 4 new proofs), frontend lib tests 4/4, microdesign tests 8/8 unchanged, tests/microdesign_parity 15+1 (every .2c.1 + .2c.2a portable proof still green — the cross-tree ConstExpr import is read-only by the new module), tests/pipeline 121 passed, tests/snapshots 6 passed, bin tests 5+29+3 passed, doc-tests 0 (unchanged). DUT lane stays byte-identical by construction (frontend is never invoked from gen::*; the new pub mod is structurally additive). No SV/manifest emit (.2b), no harness (.2c). No ROADMAP/book change.`
  Commit: `Phase 8: PHASE-8-FRONTEND-ACCEPT.2a source-level AST IR + construction-time elaboration-evaluator (oracle)`

- ID: `PHASE-8-FRONTEND-ACCEPT.2b`
  Status: `done`
  Goal: `Emitters: the un-elaborated-where-appropriate SV emitter for the source-IR (parameter ports kept symbolic, instance bindings carrying expressions not resolved integers, generate predicates preserved as written, typedef references un-flattened) + the JSON elaborated-facts manifest emitter (instance tree with path→target→resolved child params→port bindings, selected generate branches/iterations, package+typedef resolutions, per .1's manifest schema extension of Phase 7's), both emitted from the same evaluated IR (.2a). Default-off DUT-byte-identical is structural (separate module never invoked from the DUT generate path; PHASE-9 selector wires invocation later). Cargo-portable structural proof: emitted SV declarations + manifest are consistent with the elaboration-evaluator by construction; byte-reproducible for fixed seeds.`
  Acceptance: `cargo fmt/clippy/check/test green; forced-on emits valid un-elaborated SV + schema-valid elaborated-facts manifest, byte-reproducible; default-off byte-identical to the DUT lane; structural-consistency proof per .1's schema; no ROADMAP advance; no book/ change (book reconciliation is .2c.2-equivalent or .2c).`
  Verification: `src/frontend/mod.rs extended with the SV and manifest emitters and the supporting manifest types. emit_sv(unit: &SourceUnit) -> String emits: a one-line provenance comment; one package block per Package (`package <name>; localparam int K = <symbolic expr>; endpackage`); one child-stub block per child Module (`module child_<seed> #(parameter int CP<i> = <symbolic expr>); endmodule`); and the top module with `parameter int P<i> = <symbolic expr>` headers + `localparam int L<i> = <symbolic expr>;` chains + named-binding instance instantiations `child_<seed> #(.CP<i>(<symbolic expr>)) u_<seed>_<idx> ();` + `generate if (<symbolic condition>) begin : g_taken logic gflag; assign gflag = 1'b1; end else begin : g_else logic gflag; assign gflag = 1'b0; end endgenerate`. All expressions are emitted via the cross-tree-reused crate::microdesign::expr_to_sv (the fully-parenthesized SV printer); the gflag marker signals match Phase 7's g_taken/g_else convention so the same netname-prefix-scan trick from PHASE-7-ORACLE-MICRODESIGN.2c.2a's yosys extractor works on Phase-8 hierarchies. Manifest types (all pub, serde Serialize+Deserialize, BTreeMap throughout for deterministic key order): PackageFacts{name, constants: BTreeMap<String, i128>}; ParamFact{value: i128, expr: String}; InstanceFact{inst_name, child_module, resolved_bindings: BTreeMap<String, i128>}; GenerateFact{taken: bool}; Manifest{seed, top, packages: Vec<PackageFacts>, top_params: BTreeMap<String, ParamFact>, top_localparams: BTreeMap<String, ParamFact>, instances: Vec<InstanceFact>, generate_branches: BTreeMap<String, GenerateFact>}. build_manifest(unit: &SourceUnit) -> Manifest reads the .2a oracle's stored .value/.resolved/.taken fields directly (no re-evaluation; load-bearing on the elaborated_facts_match_a_fresh_reeval_across_the_seed_set proof from .2a); emit_manifest(unit) -> String serializes via serde_json::to_string_pretty (byte-stable thanks to BTreeMap ordering). DUT-byte-identical is structural+trivial: frontend is a separate top-level module never invoked from gen::*; the new emitters are extension-only. 3 new lib proofs (10 total in frontend::tests; was 7 + 3): emit_sv_is_valid_unresolved_shape (seed 7, n_params=4, n_children=2: package acc_7_pkg/K + endpackage; child_7 with parameter int CP0 = ; module acc_7 with parameter int P0 = ; localparam int L0 = ; both u_7_0 () and u_7_1 () instances; generate with : g_taken and : g_else labels + endgenerate; at least one chained localparam line is symbolic — contains an operator + a reference, not a bare integer), manifest_mirrors_the_oracle (across reproducibility-set seeds {0,1,7,42,12345}: parsed manifest is valid JSON; seed/top match; packages[0].name + constants.K = oracle; every top_params/top_localparams entry matches both .value and expr_to_sv(.expr); every InstanceFact matches inst_name+child_module + per-binding resolved_bindings vs the oracle's ParamBinding.resolved; every generate_branches[label].taken vs the oracle's GenerateIf.taken), sv_and_manifest_are_byte_reproducible (same (seed, shape) → byte-identical .sv + .json across rebuilds for seeds {0,1,7,42,999}; distinct seeds differ — the reproducibility contract). Fixed 2 single_char_add_str clippy hits (push_str(\")\") → push(')')); cargo fmt --all --check / clippy --all-targets -- -D warnings / check --all-targets clean. Full cargo test green: lib 236 passed (was 233 + 3 new proofs); frontend lib tests 10/10 (was 7); microdesign tests 8/8 unchanged; tests/microdesign_parity 15 passed + 1 ignored; tests/pipeline 121 passed (654s); tests/snapshots 6 passed; bin tests 5+29+3 passed; doc-tests 0 (unchanged). No ROADMAP advance (that is .2c). No book/ change (book reconciliation is .2c).`
  Commit: `Phase 8: PHASE-8-FRONTEND-ACCEPT.2b un-elaborated SV emitter + elaborated-facts JSON manifest emitter (from the .2a oracle)`

- ID: `PHASE-8-FRONTEND-ACCEPT.2c`
  Status: `active`
  Goal: `The hierarchy-aware parity harness + repo-owned gate: a downstream consumer (currently planned: yosys hierarchy -top + write_json AFTER elaboration, or slang elaborate --ast-json, or verilator --xml-only) reports resolved instance-tree facts; compare to the manifest — exact agreement on the tool-supported categories or a retained counterexample tuple. Tool-gated (cargo test stays green tool-less — the convention reaffirmed in PHASE-7-ORACLE-MICRODESIGN's Decisions and applied at .2c.1/.2c.2a). Then verify a clean run and record ROADMAP Phase 8 -> done (r87 no-aspirational-claims). Reuses the scoped-comparator infrastructure (FactCategory/ParityScope/compare_manifest_to_tool_report_in_scope) that PHASE-7's .2c.2a delivered, extended with HIERARCHY-aware variants (InstancePathMismatch, PortBindingMismatch, GenerateBranchMismatch keyed by instance-tree path) — a recorded extension that PHASE-7's comparator stays unchanged. Split per the Splitting Rules + the proven PHASE-7-ORACLE-MICRODESIGN.2c -> .2c.1/.2c.2 decomposition that closed Phase 7: .2c.1 builds the harness (cargo-portable hierarchy-aware comparator + scoped per-tool capability set + tool-gated #[ignore] real-tool scaffold; no real run, cargo stays green tool-less, no ROADMAP advance) and .2c.2 runs the real #[ignore] gate + verifies exact-agreement + banks the artifact + promotes ROADMAP Phase 8 -> done (gate-blocked, r87). .2c.2 may further split on a discovered tool-capability dependency, mirroring PHASE-7-ORACLE-MICRODESIGN.2c.2's split into .2c.2a (extractor + scoped comparator) + .2c.2b (real-tool run + ROADMAP).`
  Children: `PHASE-8-FRONTEND-ACCEPT.2c.1` (done), `PHASE-8-FRONTEND-ACCEPT.2c.2` (active container: `.2c.2a`, `.2c.2b`)

- ID: `PHASE-8-FRONTEND-ACCEPT.2c.1`
  Status: `done`
  Goal: `Build the hierarchy-aware parity harness — cargo-portable + tool-gated. A new top-level test file (e.g. tests/frontend_parity.rs, mirroring tests/microdesign_parity.rs) carrying: (a) a pure-Rust hierarchy-aware fact-extraction-and-comparison core operating on already-collected tool output (an in-test synthetic ToolReport representation, NOT a tool invocation) — testable cargo-portably without yosys/verilator/slang; (b) hierarchy-aware Divergence variants extending Phase 7's set (InstanceMissingInTool, InstanceMissingInManifest, InstanceBindingMismatch{inst_name, param_name, expected, actual}, plus the existing per-category MismatchInTool/InManifest/Mismatch for top params/localparams/generate-branches/package constants); (c) a tool-equipped #[ignore]-gated test that, when invoked with the tools available, drives a fixed deterministic corpus through emit_sv + emit_manifest, shells the chosen downstream consumer on each .sv, parses the resolved-facts report, and feeds it to the cargo-portable comparator. Cargo-portable proofs: deterministic seeds (matching .2a's reproducibility set) × build → manifest, then feed a hand-constructed-to-agree synthetic tool report to the comparator and prove exact-equality; AND feed a deliberately-perturbed synthetic report and prove the comparator surfaces the right divergence kind for each axis (top-param / top-localparam / package-constant / instance-binding / generate-branch / instance-presence). Tool-gated ⇒ portable cargo test stays green tool-less. No real cargo-test run of the #[ignore] (that is .2c.2); cargo stays green tool-less; ROADMAP unchanged.`
  Acceptance: `cargo fmt/clippy(-D warnings)/check --all-targets/test green; new tests/frontend_parity.rs landed with the cargo-portable hierarchy-aware comparator + the #[ignore]-gated tool harness; cargo-portable comparator proof exact-agrees on synthetic-agree fixtures and surfaces the right divergence kind on each perturbed fixture across every axis (top-params, top-localparams, package-constants, instance-bindings per instance, generate-branches, instance presence); the #[ignore] test compiles + is invocable but is NOT run in the portable suite; ROADMAP unchanged (advance is .2c.2 on a verified gate); no book/ change (book reconciliation is .2c.2).`
  Verification: `src/frontend/mod.rs extended with the Phase-8-specific parity comparator core (parallel to Phase 7's microdesign types, NOT derived — different artifact shape: Phase 8 carries an instances vector + per-instance bindings the Phase 7 single-module set does not). pub struct ToolReport{seed, top, package_constants: BTreeMap<String,i128>, top_params: BTreeMap<String,i128>, top_localparams: BTreeMap<String,i128>, instances: Vec<InstanceToolReport>, generate_branches: BTreeMap<String,bool>}; pub struct InstanceToolReport{inst_name, child_module, resolved_bindings: BTreeMap<String,i128>}; pub enum Divergence with 23 variants — SeedMismatch/TopMismatch + per-category {MissingInTool, MissingInManifest, Mismatch} × {PackageConstants, TopParams, TopLocalparams, GenerateBranches} + the load-bearing hierarchy-aware additions {InstanceMissingInTool, InstanceMissingInManifest, InstanceChildModuleMismatch, InstanceBindingMissingInTool, InstanceBindingMissingInManifest, InstanceBindingMismatch}; pub enum FactCategory{Seed, Top, PackageConstants, TopParams, TopLocalparams, Instances, GenerateBranches}; pub struct ParityScope{categories: BTreeSet<FactCategory>} with all()/none()/only(&[...]) constructors + .contains; pub fn compare_manifest_to_tool_report(manifest, report) → strict-all-categories case delegating to compare_manifest_to_tool_report_in_scope(manifest, report, &ParityScope::all()); pub fn compare_manifest_to_tool_report_in_scope(manifest, report, scope) — the scoped walker that skips out-of-scope axes entirely (no MissingIn*/Mismatch variants surface for skipped categories); the Instances arm builds name-keyed BTreeMap<&str, &InstanceFact|&InstanceToolReport> indices from each side to do order-independent presence checks + per-binding compares (deterministic since BTreeMap iteration is sorted); pub fn synthetic_tool_report_from_manifest(manifest) → ToolReport (constructs an always-agreeing report by flattening packages to pkg::name + projecting value/resolved/taken fields from the .2a oracle). New tests/frontend_parity.rs (mirrors tests/microdesign_parity.rs as a top-level integration test) carries 10 cargo-portable comparator proofs (all green): comparator_agrees_on_synthetic_tool_report_built_from_the_oracle (load-bearing baseline across reproducibility-set seeds {0,1,7,42,12345}); comparator_surfaces_top_param_mismatch_when_perturbed; comparator_surfaces_top_localparam_mismatch_when_perturbed; comparator_surfaces_package_constant_mismatch_when_perturbed; comparator_surfaces_instance_binding_mismatch_when_perturbed (the hierarchy-aware Phase-8 addition — perturbs ONE binding on ONE instance, asserts the right InstanceBindingMismatch surfaces AND no spurious divergence on the OTHER instance); comparator_surfaces_generate_branch_mismatch_when_flipped; comparator_surfaces_instance_presence_divergences (the other hierarchy-aware addition — drops an instance from the report → InstanceMissingInTool; adds a spurious one → InstanceMissingInManifest); comparator_surfaces_seed_and_top_mismatch_when_perturbed (defensive — both top-level variants in one fixture); scoped_comparator_only_enforces_scoped_categories (load-bearing scoping proof — TopParams-only scope ignores instance-binding perturbation but surfaces top-param perturbation); empty_scope_ignores_every_disagreement (self-check on the scoping). Plus 1 tool-gated #[ignore] scaffold parity_against_real_downstream_elaborator (any-of-yosys/slang/verilator presence guard at the head; corpus-driver loop wired against the same SEEDS/N_PARAMS/N_CHILDREN constants the portable proofs use; placeholder for the .2c.2-owned emit→shell→extract→compare end-to-end wiring). cargo fmt --all --check / cargo clippy --all-targets -- -D warnings / cargo check --all-targets clean. Full cargo test green: tests/frontend_parity 10 passed + 1 ignored; tests/microdesign_parity 15 passed + 1 ignored unchanged; tests/pipeline 121 passed (758s); tests/snapshots 6 passed; lib 236 passed (unchanged — the new code is in src/frontend/ as pub items; the new tests live in tests/frontend_parity.rs); bin tests 5+29+3 passed; doc-tests 0 (unchanged). Portable cargo test stays green tool-less. No ROADMAP advance (that is .2c.2 on a verified clean banked artifact, r87). No book/ change.`
  Commit: `Phase 8: PHASE-8-FRONTEND-ACCEPT.2c.1 hierarchy-aware parity harness — comparator core + cargo-portable proofs + tool-gated #[ignore] scaffold`

- ID: `PHASE-8-FRONTEND-ACCEPT.2c.2`
  Status: `active`
  Goal: `Real tool-equipped run of the .2c.1 #[ignore]-gated parity harness against a fixed deterministic corpus; VERIFY exact-agreement (or zero retained counterexamples) BEFORE any promotion (r87 no-aspirational-claims, mirroring Phase 6 .2.4/.3.4b and Phase 7 .2c.2b.2). Then record ROADMAP Phase 8 → done (with the explicit "tool-supported categories" scope caveat — the same kind of caveat Phase 7 closed under, recording what richer-AST tools would additionally cover); reconcile book (book/src/ir.md or a new "Phase 8 frontend-accept lane" page), README phase narrative, CODEBASE_ANALYSIS phase-coverage-map Phase-8 row, MEMORY recent commits. Closes PHASE-8-FRONTEND-ACCEPT.2c + .2 container + the tree. Split per the proven PHASE-7-ORACLE-MICRODESIGN.2c.2 → .2c.2a/.2c.2b discovered-dependency decomposition: empirical probe of yosys hierarchy + write_json on a Phase-8 acc_<seed>.sv confirmed yosys exposes 5 of the 7 manifest fact categories (top params via .parameter_default_values; instances + per-instance per-binding values via cells[<inst>].{type, parameters} — the load-bearing hierarchy-aware axis; generate-branch via netnames key prefix; localparams + package-constants folded). The FactCategory-scoped extractor + scoped comparator wiring is itself signoff-sized code that needs its own leaf BEFORE any real-tool run can be honest about what was checked (r87).`
  Children: `PHASE-8-FRONTEND-ACCEPT.2c.2a`, `PHASE-8-FRONTEND-ACCEPT.2c.2b`

- ID: `PHASE-8-FRONTEND-ACCEPT.2c.2a`
  Status: `done`
  Goal: `Land the yosys-specific extractor + end-to-end-runnable #[ignore] harness, exactly mirroring PHASE-7-ORACLE-MICRODESIGN.2c.2a. tests/frontend_parity.rs: yosys_hierarchy_write_json_to_tool_report(json, seed) extractor that reads (a) .modules.<top>.parameter_default_values into top_params (binary-string → SV int → i128 with sign-extension, reusing the parsing pattern from PHASE-7-ORACLE-MICRODESIGN.2c.2a's parse_yosys_binary_param); (b) .modules.<top>.cells[<inst>].{type, parameters} into instances (with the cell's type = child_module + per-binding resolved values from the cells parameters map); (c) .modules.<top>.netnames key prefix scan for g_taken./g_else. into generate_branches["g_taken"]. The yosys scope is only(&[Seed, Top, TopParams, Instances, GenerateBranches]) — top_localparams + package_constants are deliberately empty (folded by yosys; richer-AST tools see them). Then rewrite parity_against_real_downstream_elaborator end-to-end-runnable: for each seed in SEEDS, write emit_sv to a tmp file, shell yosys -q -p "read_verilog -sv <sv>; hierarchy -top acc_<seed>; write_json <out.json>" (NOT proc;opt — that strips empty-bodied child instances; verified by today's probe), parse, build a ToolReport, call compare_manifest_to_tool_report_in_scope with the yosys scope, assert Ok(()) or retain the counterexample. Plus a cargo-portable proof of the extractor that exercises the parse-yosys-binary path AND validates the cells-to-instances mapping on a hand-built synthetic JSON. No real cargo-test run of the #[ignore] (that is .2c.2b); cargo stays green tool-less.`
  Acceptance: `cargo fmt/clippy(-D warnings)/check --all-targets/test green; yosys-specific extractor lands + an extractor proof green; #[ignore] test is end-to-end runnable; portable cargo test stays green tool-less; ROADMAP unchanged (advance is .2c.2b on a verified run); no book/ change.`
  Verification: `tests/frontend_parity.rs extended with the Phase-8-specific yosys extractor + end-to-end-runnable #[ignore] test. parse_yosys_binary_param(s) -> Option<i128> parses yosys binary-strings as SV int (u32::from_str_radix(s, 2) then cast `as i32 as i128` for sign-extension; defensive on empty / non-binary / >32-bit); the sign-extension is kept symmetric with Phase 7 even though Phase 8's .2a builder doesn't currently emit negative values (forward-compatible for richer-builder future). yosys_hierarchy_write_json_to_tool_report(json, seed) reads .modules.<top>.parameter_default_values → top_params (parsing each binary-string); reads .modules.<top>.cells[<inst>].{type, parameters} → instances (each cell becomes an InstanceToolReport: type → child_module; parameters → resolved_bindings); reads .modules.<top>.netnames key prefix scan for g_taken./g_else. → generate_branches["g_taken"] (true iff g_taken-prefixed key present AND no g_else-prefixed key — matches the convention from Phase 7's yosys_write_json_to_tool_report); the folded axes (package_constants, top_localparams) are deliberately left empty since yosys doesn't expose them by name. yosys_hierarchy_scope() returns ParityScope::only(&[Seed, Top, TopParams, Instances, GenerateBranches]) — the 5 categories yosys's hierarchy + write_json actually covers per today's empirical probe. parity_against_real_yosys_hierarchy_write_json rewritten end-to-end: for each seed in SEEDS, emit_sv → CARGO_TARGET_TMPDIR/frontend-parity-phase8-yosys/acc_<seed>.sv; emit_manifest → .json; shell `yosys -q -p "read_verilog -sv <sv>; hierarchy -top acc_<seed>; write_json <out.json>"` (deliberately NO proc;opt — the probe confirmed it collapses empty-bodied child instances out of .cells); parse → ToolReport via the new extractor; call compare_manifest_to_tool_report_in_scope with yosys_hierarchy_scope; accumulate counterexamples; panic on any non-empty list (or eprintln "parity gate clean across N seeds"); yosys-presence guard at the head matches the iverilog-not-installed convention from DIFFERENTIAL-SIMULATION.1. parity_against_real_downstream_elaborator from .2c.1 preserved as a friendly no-op pointing at the named yosys test. 2 new cargo-portable extractor proofs (12 portable + 2 ignored total in tests/frontend_parity.rs; was 10 + 1): yosys_extractor_reads_a_synthetic_hierarchy_write_json_correctly (hand-built JSON for seed 0 with 2 instances × 2 bindings each: P0=57, P1=38; u_0_0 → child_0 with CP0=57, CP1=40; u_0_1 → child_0 with CP0=60, CP1=59; g_taken=true; folded axes empty — exercises every branch of the extractor); yosys_extractor_reports_g_else_when_else_branch_survives (g_else-survives case → generate.g_taken=false). cargo fmt --all --check / cargo clippy --all-targets -- -D warnings / cargo check --all-targets clean. Full cargo test green: tests/frontend_parity 12 passed + 2 ignored (was 10 + 1); rest unchanged. Portable cargo test stays green tool-less. NO real cargo-test run of the #[ignore] tests (that is .2c.2b). No ROADMAP advance; no book/ change.`
  Commit: `Phase 8: PHASE-8-FRONTEND-ACCEPT.2c.2a yosys hierarchy write_json extractor + end-to-end-runnable #[ignore] harness`

- ID: `PHASE-8-FRONTEND-ACCEPT.2c.2b`
  Status: `pending`
  Goal: `Run the .2c.2a #[ignore]-gated parity gate against real yosys, verify exact-agreement on the yosys-supported categories (Seed/Top/TopParams/Instances/GenerateBranches) across the full corpus, bank the verified-clean artifact (under /tmp/anvil-frontend-parity-phase8-yosys-p1/ per established convention), then promote ROADMAP Phase 8 → done with the explicit yosys-supported-categories scope caveat (top_localparams + package-constants remain visible only to richer-AST tools — slang/verilator-with-debug — and are recorded as a post-Phase-8 follow-up that does NOT block closure; ANVIL's by-construction oracle already covers all 7 categories). Reconcile book (book/src/ir.md "Phase 8 frontend-accept lane" delivered note), README phase narrative, CODEBASE_ANALYSIS phase-coverage-map Phase-8 row, MEMORY recent commits. Closes PHASE-8-FRONTEND-ACCEPT.2c.2 + .2c + .2 container + the tree. May further split per PHASE-7-ORACLE-MICRODESIGN.2c.2b → .2c.2b.1/.2c.2b.2 precedent if a discovered ANVIL self-consistency bug surfaces during the real run (the Phase 7 pattern).`
  Acceptance: `Banked artifact captures the gate's exact-agreement on the corpus (zero retained counterexamples); ROADMAP Phase 8 → done only after the verified clean run; .2c.2b + .2c.2 + .2c + .2 container + tree all → done. No aspirational claims (verified artifact precedes the ROADMAP promotion).`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `PHASE-8-FRONTEND-ACCEPT.2c.2b` | `pending` (gate-blocked, real-tool run) | **`.2c.2a` done (`2026-05-20`)** — `tests/frontend_parity.rs` extended with `parse_yosys_binary_param` + `yosys_hierarchy_write_json_to_tool_report` extractor + `yosys_hierarchy_scope()` (`only(&[Seed, Top, TopParams, Instances, GenerateBranches])`) + the rewritten end-to-end-runnable `parity_against_real_yosys_hierarchy_write_json` `#[ignore]` test (`hierarchy -top` only — no `proc; opt`); 2 new cargo-portable extractor proofs (12 portable + 2 ignored). Full `cargo test` green; portable stays green tool-less. `.2c.2b` runs `cargo test -- --ignored parity_against_real_yosys_hierarchy_write_json` against real yosys, verifies exact-agreement on the yosys-supported 5-of-7 categories across the corpus, banks the verified-clean artifact at `/tmp/anvil-frontend-parity-phase8-yosys-p1/`, then promotes **ROADMAP Phase 8 → done** with the explicit yosys-supported-categories scope caveat (richer-AST coverage via slang/verilator-with-debug is the recorded post-Phase-8 follow-up that does NOT block closure) + reconciles book/README/CODEBASE. r87 no-aspirational-claims. May further split per Phase 7 `.2c.2b` → `.2c.2b.1`/`.2c.2b.2` on a discovered ANVIL self-consistency bug. |

## Decisions

- `2026-05-16`: Phase 8 uses a dedicated source-level IR by roadmap
  decree; reusing the gate-level circuit IR is a recorded rejected
  direction (it cannot express the required source surfaces).
- `2026-05-20`: **`.2` split** into `.2a` (source-level AST IR +
  construction-time elaboration-evaluator), `.2b` (un-elaborated-
  SV emitter + elaborated-facts JSON manifest emitter), `.2c`
  (hierarchy-aware parity harness + repo-owned gate → ROADMAP
  Phase 8). Splitting Rules along the exact independently-
  reviewable boundaries `.1`'s design named, **exactly mirroring**
  the proven `PHASE-7-ORACLE-MICRODESIGN.2` → `.2a`/`.2b`/`.2c`
  decomposition that closed Phase 7 on 2026-05-20. Each child is
  separately reviewable; `.2a`'s elaboration evaluator + `.2b`'s
  manifest core *extend* the Phase 7 evaluator/manifest core
  (the reuse `PHASE-9-MULTI-ARTIFACT-UMBRELLA.2`'s L1-wrap
  migration depends on). Unblocked now that Phase 7 closed —
  `src/microdesign/` is in-tree, the Phase-7 `ConstExpr` set is
  ready to be cross-tree-imported as the Expr layer of Phase 8's
  source IR. `.2` is now a container; no renumbering. Tree-
  planning, docs-only; no `src/`/`tests/` change (`cargo`
  unchanged-green vs `20a7b4a`). Frontier → `.2a`.
- `2026-05-20`: **`.2c` split** into `.2c.1` (build the
  hierarchy-aware parity harness — cargo-portable comparator
  + tool-gated `#[ignore]` real-tool scaffold; no real run,
  no ROADMAP advance, cargo stays green tool-less) and
  `.2c.2` (real `--ignored` run + verify + bank +
  **ROADMAP Phase 8 → done**; gate-blocked, r87 no-aspirational-
  claims; may further split per the proven `PHASE-7-ORACLE-
  MICRODESIGN.2c.2` → `.2c.2a`/`.2c.2b` decomposition if a
  tool-capability dependency surfaces — the same shape that
  closed Phase 7). Exactly mirrors the proven `PHASE-7-
  ORACLE-MICRODESIGN.2c` → `.2c.1`/`.2c.2` decomposition
  that closed Phase 7 on 2026-05-20. `.2c` is now a
  container; no renumbering. Tree-planning, docs-only; no
  `src/`/`tests/` change (`cargo` unchanged-green vs
  `d67df0c`). Frontier → `.2c.1`.
- `2026-05-20` (**`.2c.2` split — discovered tool-capability
  dependency**): empirical probe of `yosys hierarchy -top
  acc_0; write_json` (locally-installed yosys 0.64) on a
  Phase-8 `acc_<seed>.sv` immediately after `.2c.1` landed
  at `977c632` confirmed yosys exposes **5 of the 7
  manifest fact categories** — `.parameter_default_values`
  carries the top parameters (binary-string → SV `int` →
  `i128` with sign-extension); **`.cells[<inst>].{type,
  parameters}` carries the per-instance per-binding
  resolved values (the load-bearing hierarchy-aware Phase-8
  axis the comparator gained `Instance*` variants for in
  `.2c.1`)**; `.netnames` key prefix `g_taken.`/`g_else.`
  carries the generate-branch decision. **Top_localparams
  and package-constants are folded** by yosys's elaborator
  and not name-introspectable from `write_json` alone —
  richer-AST tools (`slang --ast-json`,
  `verilator --xml-only`) see them. **Crucially**: the
  probe also discovered that `proc; opt` (the standard
  Phase 7 yosys pipeline) **collapses the empty-bodied
  child instances away**, dropping them from `.cells`. The
  fix is to invoke yosys with `hierarchy -top acc_<seed>;
  write_json` only — no `proc; opt`. This is the
  Phase-8-specific tool-capability dependency: the yosys
  invocation pattern from Phase 7 doesn't carry over
  unchanged, AND yosys covers a different (Phase-8-richer)
  set of axes. Per Splitting Rules + the proven
  `PHASE-7-ORACLE-MICRODESIGN.2c.2` → `.2c.2a`/`.2c.2b`
  discovered-dependency-split precedent, `.2c.2` was split
  into `.2c.2a` (yosys-specific extractor + the
  `hierarchy -top` invocation + end-to-end-runnable
  `#[ignore]` + cargo-portable extractor proof; no real
  run, no ROADMAP advance) and `.2c.2b` (real-tool run +
  verify exact-agreement on the yosys-supported categories
  + bank artifact + ROADMAP Phase 8 → done with the
  scope caveat; gate-blocked, r87 no-aspirational-claims).
  `.2c.2` is now a container; no renumbering. Tree-
  planning, docs-only; no `src/`/`tests/` change (`cargo`
  unchanged-green vs `977c632`); `mdbook build book` clean
  (no `book/` change). Frontier → `.2c.2a`.

## Open Questions

- Degree of reuse of Phase 7's expected-facts manifest machinery —
  **resolved by `.1`**: Phase 8 **reuses** Phase 7's construction-
  time evaluator + JSON-manifest emitter core and **extends** the
  schema with the instance tree / generate selections / package +
  typedef resolutions. Dependency direction:
  `PHASE-8-FRONTEND-ACCEPT.2` sequences **after**
  `PHASE-7-ORACLE-MICRODESIGN.2` (its evaluator/manifest core must
  land first).

## Blockers

- None for `.1`. `.2` coordinates with Phase 7's manifest/parity
  infrastructure; `.1` records the dependency direction.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-05-18` | `PHASE-8-FRONTEND-ACCEPT.1` | `DEVELOPMENT_NOTES.md` Phase 8 source-IR design entry landed (the shift to un-elaborated-hierarchy + manifest; codebase grounding — dedicated source-level AST IR, separate generator path, reuses Phase 7 evaluator/manifest core; source-IR sketch; instance-tree manifest schema; oracle-by-construction; hierarchy-aware parity harness; 4 rejected alternatives; Open Question resolved; `.2` split). Design-only, no code; `mdbook build book` clean; `cargo fmt --all --check` clean; full `cargo test` green at base `f0cff2c` (no `src/`/`tests/` touched). | Done. |
| `2026-05-20` | `PHASE-8-FRONTEND-ACCEPT.2b` | `src/frontend/mod.rs` extended with the SV emitter + manifest emitter + supporting types (all `pub`, all serde). `emit_sv(unit) -> String` emits: provenance comment; one `package <name>; localparam int K = <symbolic expr>; endpackage` per `Package`; one `module child_<seed> #(parameter int CP<i> = <symbolic expr>); endmodule` per child stub; the top module with symbolic `parameter int P<i>` headers + `localparam int L<i> = <symbolic expr>;` chains + named-binding `child_<seed> #(.CP<i>(<symbolic expr>)) u_<seed>_<idx> ();` instance instantiations + `generate if (<symbolic condition>) begin : g_taken logic gflag; assign gflag = 1'b1; end else begin : g_else logic gflag; assign gflag = 1'b0; end endgenerate`. All expressions render via the cross-tree-reused `crate::microdesign::expr_to_sv` (the fully-parenthesized SV printer); the `g_taken`/`g_else` `gflag` marker matches Phase 7's convention so the netname-prefix-scan trick from `.2c.2a`'s yosys extractor works on Phase-8 hierarchies. Manifest types: `PackageFacts{name, constants: BTreeMap<String, i128>}`; `ParamFact{value: i128, expr: String}`; `InstanceFact{inst_name, child_module, resolved_bindings: BTreeMap<String, i128>}`; `GenerateFact{taken: bool}`; `Manifest{seed, top, packages, top_params, top_localparams, instances, generate_branches}`. `build_manifest(unit) -> Manifest` reads the `.2a` oracle's stored `.value`/`.resolved`/`.taken` fields directly (no re-evaluation — load-bearing on `elaborated_facts_match_a_fresh_reeval_across_the_seed_set` from `.2a`). `emit_manifest(unit) -> String` serializes via `serde_json::to_string_pretty` (byte-stable thanks to `BTreeMap` ordering). DUT-byte-identical is structural+trivial. 3 new lib proofs (10 total): `emit_sv_is_valid_unresolved_shape` (seed 7: package + endpackage + child + both `u_7_0 ()`/`u_7_1 ()` instances + `: g_taken`/`: g_else` + at least one symbolic chained localparam line); `manifest_mirrors_the_oracle` (across seeds `{0,1,7,42,12345}`: parsed JSON matches the `.2a` oracle on every fact axis — `packages[0]` + every `top_params`/`top_localparams` entry value+expr + every `InstanceFact`'s resolved bindings + every `generate_branches[label].taken`); `sv_and_manifest_are_byte_reproducible` (seeds `{0,1,7,42,999}`: same `(seed, shape)` → byte-identical `.sv` + `.json`; distinct seeds differ). Fixed 2 single_char_add_str clippy hits (`push_str(")")` → `push(')')`). `cargo fmt --all --check`/`clippy --all-targets -- -D warnings`/`check --all-targets` clean. Full `cargo test` green: lib **236** passed (was 233 + 3); `frontend::tests` 10/10 (was 7); `microdesign::tests` 8/8 unchanged; `tests/microdesign_parity` 15 passed + 1 ignored; `tests/pipeline` 121 passed (654s); `tests/snapshots` 6 passed; bin tests 5+29+3 passed; doc-tests 0 (unchanged). DUT lane unchanged-byte-identical-by-construction. No ROADMAP advance (`.2c`); no `book/` change (`.2c`). | Done. Frontier → `.2c`. |
| `2026-05-20` | `PHASE-8-FRONTEND-ACCEPT.2a` | New separate top-level module `src/frontend/mod.rs` registered via `pub mod frontend` (NOT in `src/ir/` — circuit IR cannot express modules/params/packages/generate; the category-error `.1` rejected). AST IR types: `SourceUnit{seed, packages, children, top}` (depth-1 instance tree — sufficient to stress every elaboration axis the parity gate checks; deeper trees are a post-`.2a` knob, not a blocker); `Package{name, items}`; `PackageItem::Localparam(ParamDecl)`; `Module{name, params, body}`; `ParamDecl{name, kind, expr, value}` (own type so Phase-8's package-vs-port distinction is local; cross-tree reuse is at the `ConstExpr`/`eval` layer); `ModuleItem::Localparam(ParamDecl) | Instance(Instance) | GenerateIf(GenerateIf)`; `Instance{inst_name, child_module, param_bindings}` (named-binding form in `.2a`); `ParamBinding{name, expr, resolved}`; `GenerateIf{label, else_label, condition, taken, then_branch, else_branch}`. Every type derives `Debug+Clone+PartialEq+Eq` so the reproducibility proof can byte-compare and the manifest-mirror proof can map-compare. Cross-tree reuse: `use crate::microdesign::{eval, BinOp, ConstExpr, EvalError, ParamKind}` — Phase 7's `ConstExpr`/`eval` are the expression layer for parameter defaults / localparam chains / instance bindings / generate predicates (per `.1`'s full-factorization plan). Construction-time elaboration-evaluator `pub fn elaborate(unit: &mut SourceUnit) -> Result<BTreeMap<String, i128>, EvalError>` walks (1) package localparams → `pkg::name` env, (2) top module parameter ports → `name` env, (3) top module body items (Localparams extend env in declaration order; Instance bindings resolve in the PARENT's env and populate `ParamBinding.resolved`; `GenerateIf.taken = eval(condition) != 0`; else-branch elaborates in a sandboxed clone so it doesn't leak — SV's model). Builder IS the oracle: every `.value`/`.resolved`/`.taken` is set in place; downstream readers (emit, manifest, comparator) read them directly without re-evaluating. Rules-first reproducible builder `pub fn build_acceptable_unit(seed, n_params, n_children)`: one `ChaCha8Rng::seed_from_u64` drives everything (no `thread_rng`); package `acc_<seed>_pkg`/`K = (seed % 32) + 1`; child stub `child_<seed>` with `n_params` literal-default parameters; top module `acc_<seed>` with `n_params` parameter ports + `n_params` chained localparams (L0 references P0, Li references L<i-1>, ±small literal) + `n_children` named-binding `child_<seed>` instances (each binds every `CP<i>` to `Add(<top-param-or-localparam-ref>, <small-offset>)`) + one `GenerateIf` with condition `P0 >= acc_<seed>_pkg::K`. Resolved in place via `elaborate()`. 4 unit proofs (all green): `build_acceptable_unit_has_the_documented_shape` (smoke); `unit_is_reproducible_and_seed_sensitive` (load-bearing reproducibility invariant — same `(seed, shape)` → byte-identical `SourceUnit` across rebuilds for seeds `{0, 1, 7, 42, 12345}`; distinct seeds differ); `elaboration_evaluator_resolves_every_axis` (package K positive; literal-rooted top params resolve to their literal; localparams re-eval consistently in the prefix env; `GenerateIf.taken` matches a fresh eval of the condition); `elaborated_facts_match_a_fresh_reeval_across_the_seed_set` (**load-bearing oracle-no-drift invariant** — every stored `ParamDecl.value`/`ParamBinding.resolved`/`GenerateIf.taken` equals a fresh eval against the reconstructed env, across seeds 0..=8; covers ALL fact axes the manifest will carry). `cargo fmt --all --check` (sorted `pub mod` lines alphabetically — `frontend` now sits between `emit` and `gen` in `src/lib.rs`) / `clippy --all-targets -- -D warnings` / `check --all-targets` clean. Full `cargo test` green: lib **233 passed** (was 229 + 4 new proofs); `frontend::tests` 4/4; `microdesign::tests` 8/8 unchanged; `tests/microdesign_parity` 15 passed + 1 ignored; `tests/pipeline` 121 passed; `tests/snapshots` 6 passed; bin tests 5+29+3 passed; doc-tests 0 (unchanged). DUT lane stays byte-identical by construction (`frontend` never invoked from `gen::*`; the new `pub mod` is structurally additive). No SV/manifest emit (that is `.2b`); no harness (that is `.2c`); no ROADMAP/book change. | Done. Frontier → `.2b`. |
| `2026-05-20` | `PHASE-8-FRONTEND-ACCEPT.2` (split) | `.2` made a container with children `.2a` (source-level AST IR + construction-time elaboration-evaluator/oracle; unit-proven; no emit/harness) + `.2b` (un-elaborated-SV emitter + elaborated-facts JSON manifest emitter; default-off DUT-byte-identical structural) + `.2c` (hierarchy-aware parity harness + repo-owned gate → ROADMAP Phase 8; r87 no-aspirational-claims). Exactly mirrors the proven `PHASE-7-ORACLE-MICRODESIGN.2`→`.2a`/`.2b`/`.2c` decomposition that closed Phase 7 on 2026-05-20. `.2a`+`.2b`'s evaluator/manifest core *extends* the Phase 7 core; Phase 7's `ConstExpr` set is cross-tree-imported as the Expr layer of Phase 8's source IR. Unblocked now that Phase 7 closed. Tree-planning, docs-only; no `src/`/`tests/` change (`cargo` unchanged-green vs `20a7b4a`). `mdbook build book` clean (no `book/` change). | Done. Frontier → `.2a`. |
| `2026-05-20` | `PHASE-8-FRONTEND-ACCEPT.2c` (split) | `.2c` made a container with children `.2c.1` (build the hierarchy-aware parity harness — cargo-portable comparator + tool-gated `#[ignore]` real-tool scaffold; no real run, no ROADMAP advance, cargo stays green tool-less) + `.2c.2` (real `--ignored` run + verify + bank + **ROADMAP Phase 8 → done**; gate-blocked, r87; may further split per `PHASE-7-ORACLE-MICRODESIGN.2c.2` → `.2c.2a`/`.2c.2b` precedent on a discovered tool-capability dependency). Exactly mirrors the proven `PHASE-7-ORACLE-MICRODESIGN.2c` → `.2c.1`/`.2c.2` decomposition that closed Phase 7 on 2026-05-20. Tree-planning, docs-only; no `src/`/`tests/` change (`cargo` unchanged-green vs `d67df0c`); `mdbook build book` clean (no `book/` change). | Done. Frontier → `.2c.1`. |
| `2026-05-20` | `PHASE-8-FRONTEND-ACCEPT.2c.1` | `src/frontend/mod.rs` extended with the Phase-8-specific parity comparator core (parallel to Phase 7's microdesign types, NOT derived — different artifact shape: Phase 8 carries instances + per-instance bindings). `pub struct ToolReport{seed, top, package_constants, top_params, top_localparams, instances: Vec<InstanceToolReport>, generate_branches}`; `pub struct InstanceToolReport{inst_name, child_module, resolved_bindings}`; `pub enum Divergence` (23 variants — `SeedMismatch`/`TopMismatch` + per-category `{MissingInTool, MissingInManifest, Mismatch}` × `{PackageConstants, TopParams, TopLocalparams, GenerateBranches}` + the **load-bearing hierarchy-aware additions** `InstanceMissingInTool`/`InstanceMissingInManifest`/`InstanceChildModuleMismatch`/`InstanceBindingMissingInTool`/`InstanceBindingMissingInManifest`/`InstanceBindingMismatch`); `pub enum FactCategory` (7 axes — `Seed`/`Top`/`PackageConstants`/`TopParams`/`TopLocalparams`/`Instances`/`GenerateBranches`); `pub struct ParityScope{categories: BTreeSet<FactCategory>}` with `all()`/`none()`/`only(&[...])` + `.contains`; `pub fn compare_manifest_to_tool_report` strict delegates to `_in_scope` with `ParityScope::all()`; `pub fn compare_manifest_to_tool_report_in_scope` walks every scoped axis (the `Instances` arm builds name-keyed `BTreeMap` indices from each side for order-independent presence + per-binding compares); `pub fn synthetic_tool_report_from_manifest` flattens `packages` to `pkg::name` + projects the oracle fields. New `tests/frontend_parity.rs` carries 10 cargo-portable comparator proofs (all green): `comparator_agrees_on_synthetic_tool_report_built_from_the_oracle` (baseline across seeds `{0,1,7,42,12345}`); per-axis perturbation tests for top-param / top-localparam / package-constant / **instance-binding (the hierarchy-aware addition — perturbs ONE binding on ONE instance, asserts the right `InstanceBindingMismatch` surfaces AND no spurious divergence on the OTHER instance)** / generate-branch / **instance-presence (drop → `InstanceMissingInTool`; add → `InstanceMissingInManifest`)** / seed+top; `scoped_comparator_only_enforces_scoped_categories` (load-bearing scoping proof — `TopParams`-only scope ignores instance-binding perturbation but surfaces top-param); `empty_scope_ignores_every_disagreement` (self-check). Plus 1 tool-gated `#[ignore]` `parity_against_real_downstream_elaborator` scaffold (any-of-`yosys`/`slang`/`verilator` presence guard at the head; corpus-driver loop wired against the same `SEEDS`/`N_PARAMS`/`N_CHILDREN`; placeholder for `.2c.2`-owned `emit_sv`→shell→extract→compare end-to-end wiring). `cargo fmt --all --check`/`clippy --all-targets -- -D warnings`/`check --all-targets` clean. Full `cargo test` green: `tests/frontend_parity` 10 passed + 1 ignored; `tests/microdesign_parity` 15 passed + 1 ignored unchanged; `tests/pipeline` 121 passed (758s); `tests/snapshots` 6 passed; lib 236 passed (unchanged — new code is in `src/frontend/` `pub` items + tests in `tests/frontend_parity.rs`); bin tests 5+29+3 passed; doc-tests 0 (unchanged). Portable `cargo test` stays green tool-less. No ROADMAP advance (`.2c.2`); no `book/` change. | Done. Frontier → `.2c.2`. |
| `2026-05-20` | `PHASE-8-FRONTEND-ACCEPT.2c.2` (split) | `.2c.2` made a container with children `.2c.2a` (yosys-specific extractor + `hierarchy -top` invocation + end-to-end-runnable `#[ignore]` + cargo-portable extractor proof; no real run, no ROADMAP advance) + `.2c.2b` (real `--ignored` run + verify + bank + ROADMAP Phase 8 → done; gate-blocked). Triggered by today's empirical probe of `yosys hierarchy -top acc_0; write_json` confirming yosys exposes 5 of 7 manifest fact categories — top params + **instances + per-instance per-binding values via `.cells[<inst>].{type, parameters}`** + generate-branch — AND that `proc; opt` collapses the empty-bodied child instances (so the yosys invocation pattern omits `proc; opt`). Mirrors the proven `PHASE-7-ORACLE-MICRODESIGN.2c.2` → `.2c.2a`/`.2c.2b` discovered-dependency-split precedent. Tree-planning, docs-only; no `src/`/`tests/` change (`cargo` unchanged-green vs `977c632`); `mdbook build book` clean. | Done. Frontier → `.2c.2a`. |
| `PHASE-8-FRONTEND-ACCEPT.2c.2a` | `Phase 8: PHASE-8-FRONTEND-ACCEPT.2c.2a yosys hierarchy write_json extractor + end-to-end-runnable #[ignore] harness` | yosys-specific extractor + `hierarchy -top` invocation (NO `proc; opt`) + end-to-end-runnable `#[ignore]` test + 2 new cargo-portable extractor proofs (12 portable + 2 ignored); portable `cargo test` stays green tool-less. No ROADMAP advance. |
| `2026-05-20` | `PHASE-8-FRONTEND-ACCEPT.2c.2a` | `tests/frontend_parity.rs` extended with the Phase-8-specific yosys extractor + end-to-end-runnable `#[ignore]`. `parse_yosys_binary_param(s) -> Option<i128>` (signed-32-bit sign-extension, symmetric to Phase 7); `yosys_hierarchy_write_json_to_tool_report(json, seed)` (reads `.parameter_default_values` → `top_params`; reads **`.cells[<inst>].{type, parameters}` → `instances` with `child_module` + per-binding `resolved_bindings`** — the load-bearing hierarchy-aware Phase-8 axis; reads `.netnames` key prefix `g_taken.`/`g_else.` → `generate_branches["g_taken"]`; folded axes [package_constants, top_localparams] deliberately left empty since yosys doesn't expose them by name); `yosys_hierarchy_scope()` returns `ParityScope::only(&[Seed, Top, TopParams, Instances, GenerateBranches])` — the 5 categories yosys's `hierarchy + write_json` covers per today's probe. `parity_against_real_yosys_hierarchy_write_json` rewritten end-to-end: per-seed `emit_sv` → `CARGO_TARGET_TMPDIR/frontend-parity-phase8-yosys/acc_<seed>.sv` + `emit_manifest` → `.json` + shell `yosys -q -p "read_verilog -sv <sv>; hierarchy -top acc_<seed>; write_json <out.json>"` (**no `proc; opt`** — the probe confirmed it collapses empty-bodied child instances out of `.cells`); parse → `ToolReport` via the new extractor; scoped compare with `yosys_hierarchy_scope`; accumulate counterexamples + panic with full diagnostic on non-empty list (else `eprintln "parity gate clean across N seeds"`); yosys-presence guard. `parity_against_real_downstream_elaborator` from `.2c.1` preserved as a friendly no-op pointing at the named yosys test. 2 new cargo-portable extractor proofs (12 portable + 2 ignored total in `tests/frontend_parity.rs`; was 10 + 1): `yosys_extractor_reads_a_synthetic_hierarchy_write_json_correctly` (hand-built JSON for seed 0 with 2 instances × 2 bindings each — P0=57, P1=38; u_0_0 → child_0 with CP0=57, CP1=40; u_0_1 → child_0 with CP0=60, CP1=59; g_taken=true; folded axes empty — exercises every branch of the extractor); `yosys_extractor_reports_g_else_when_else_branch_survives` (g_else-survives case → `generate.g_taken=false`). `cargo fmt --all --check`/`clippy --all-targets -- -D warnings`/`check --all-targets` clean. Full `cargo test` green: `tests/frontend_parity` **12 passed + 2 ignored** (was 10 + 1); rest unchanged. Portable `cargo test` stays green tool-less. **NO real cargo-test run of the `#[ignore]` tests** (that is `.2c.2b`'s deliverable). No ROADMAP advance; no `book/` change. | Done. Frontier → `.2c.2b`. |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `PHASE-8-FRONTEND-ACCEPT.1` | `Docs: PHASE-8-FRONTEND-ACCEPT.1 source-level frontend/elaboration accept-corpus IR design` | Design-only; source-level AST IR sketch + instance-tree manifest schema + oracle-by-construction + reuses Phase 7 core + 4 rejected alternatives. No code. |
| `PHASE-8-FRONTEND-ACCEPT.2` (split) | `Docs: split PHASE-8-FRONTEND-ACCEPT.2 into .2a (source IR + elaboration-evaluator) + .2b (emitters) + .2c (parity harness + gate)` | Tree-planning, no code. Exactly mirrors the proven `PHASE-7-ORACLE-MICRODESIGN.2`→`.2a`/`.2b`/`.2c` decomposition that closed Phase 7 on 2026-05-20. Unblocked now that Phase 7 closed. |
| `PHASE-8-FRONTEND-ACCEPT.2a` | `Phase 8: PHASE-8-FRONTEND-ACCEPT.2a source-level AST IR + construction-time elaboration-evaluator (oracle)` | New `src/frontend/` module + AST IR + `elaborate()` walker + rules-first reproducible `build_acceptable_unit` + 4 unit proofs (incl. the load-bearing oracle-no-drift invariant); cross-tree reuse of Phase 7's `ConstExpr`/`eval`/`ParamKind`/`BinOp`. No emit/harness. |
| `PHASE-8-FRONTEND-ACCEPT.2b` | `Phase 8: PHASE-8-FRONTEND-ACCEPT.2b un-elaborated SV emitter + elaborated-facts JSON manifest emitter (from the .2a oracle)` | `emit_sv()` + manifest types + `build_manifest()` + `emit_manifest()`; cross-tree reuse of `microdesign::expr_to_sv`; 3 new lib proofs (10 total in `frontend::tests`); byte-reproducible. No harness. |
| `PHASE-8-FRONTEND-ACCEPT.2c` (split) | `Docs: split PHASE-8-FRONTEND-ACCEPT.2c into .2c.1 (build harness) + .2c.2 (real-tool gate + ROADMAP Phase 8)` | Tree-planning, no code. Exactly mirrors `PHASE-7-ORACLE-MICRODESIGN.2c` → `.2c.1`/`.2c.2`. |
| `PHASE-8-FRONTEND-ACCEPT.2c.1` | `Phase 8: PHASE-8-FRONTEND-ACCEPT.2c.1 hierarchy-aware parity harness — comparator core + cargo-portable proofs + tool-gated #[ignore] scaffold` | Phase-8 parity comparator core in `src/frontend/` (`ToolReport`/`InstanceToolReport`/`Divergence` × 23 variants incl. the hierarchy-aware `Instance*` additions/`FactCategory`/`ParityScope`/`compare_manifest_to_tool_report` + `_in_scope`/`synthetic_tool_report_from_manifest`); new `tests/frontend_parity.rs` (10 cargo-portable proofs + 1 tool-gated `#[ignore]` scaffold); portable `cargo test` stays green tool-less. No ROADMAP advance (`.2c.2`). |
| `PHASE-8-FRONTEND-ACCEPT.2c.2` (split) | `Docs: split PHASE-8-FRONTEND-ACCEPT.2c.2 into .2c.2a (yosys extractor + end-to-end-runnable #[ignore]) + .2c.2b (real-tool gate + ROADMAP Phase 8)` | Tree-planning, no code. Triggered by yosys-probe-discovered 5-of-7 coverage + the `proc; opt` cells-collapse caveat. Mirrors `PHASE-7-ORACLE-MICRODESIGN.2c.2` → `.2c.2a`/`.2c.2b`. |
| `PHASE-8-FRONTEND-ACCEPT.2a` | `Phase 8: PHASE-8-FRONTEND-ACCEPT.2a source-level AST IR + construction-time elaboration-evaluator (oracle)` | New `src/frontend/` module + AST IR (`SourceUnit`/`Package`/`Module`/`ModuleItem`/`Instance`/`GenerateIf`/`ParamDecl`/`ParamBinding`) + `elaborate()` walker + rules-first reproducible `build_acceptable_unit` + 4 unit proofs (incl. the load-bearing oracle-no-drift invariant); cross-tree reuse of Phase 7's `ConstExpr`/`eval`/`ParamKind`/`BinOp`. No emit/harness. |
| `PHASE-8-FRONTEND-ACCEPT.2b` | `Phase 8: PHASE-8-FRONTEND-ACCEPT.2b un-elaborated SV emitter + elaborated-facts JSON manifest emitter (from the .2a oracle)` | `emit_sv()` + manifest types (`PackageFacts`/`ParamFact`/`InstanceFact`/`GenerateFact`/`Manifest`) + `build_manifest()` + `emit_manifest()`; both from the same `.2a` oracle; cross-tree reuse of `microdesign::expr_to_sv`; 3 new lib proofs (10 total in `frontend::tests`); byte-reproducible. No harness (`.2c`). |
| `PHASE-8-FRONTEND-ACCEPT.2c` (split) | `Docs: split PHASE-8-FRONTEND-ACCEPT.2c into .2c.1 (build harness) + .2c.2 (real-tool gate + ROADMAP Phase 8)` | Tree-planning, no code. Exactly mirrors `PHASE-7-ORACLE-MICRODESIGN.2c` → `.2c.1`/`.2c.2`. |

## Changelog

- `2026-05-16`: Created task tree as part of the directive to task-tree
  every remaining roadmap phase.
- `2026-05-18`: **`.1` design landed** (design-only, no code) —
  continuous-PNT while Phase 6 `.2.4`/`.3.4b` are gate-blocked.
  `DEVELOPMENT_NOTES.md` "Phase 8 frontend/elaboration accept-corpus
  source-IR design": the un-elaborated-hierarchy-plus-manifest shift,
  a dedicated source-level AST IR (separate generator path; the
  post-elaboration circuit IR cannot express modules/params/packages/
  generate), the source-IR sketch, the instance-tree expected-facts
  manifest schema (extends Phase 7's), oracle-by-construction
  generation reusing Phase 7's evaluator/manifest core, the
  hierarchy-aware parity harness, 4 rejected alternatives, and the
  `.2` proof shape + split. Open Question resolved (reuse Phase 7
  core + extend schema; `.2` sequences after
  `PHASE-7-ORACLE-MICRODESIGN.2`). `mdbook` clean. Frontier → `.2`.
- `2026-05-20`: **`.2` split** into `.2a` (source-level AST IR +
  construction-time elaboration-evaluator), `.2b` (un-elaborated-
  SV + elaborated-facts JSON manifest emitters), `.2c`
  (hierarchy-aware parity harness + repo-owned gate → ROADMAP
  Phase 8). Splitting Rules along the exact independently-
  reviewable boundaries `.1`'s design named — **exactly the same
  shape** as the proven `PHASE-7-ORACLE-MICRODESIGN.2` →
  `.2a`/`.2b`/`.2c` decomposition that closed Phase 7 on
  2026-05-20 (one slice for IR + evaluator, one for emitters,
  one for the gated parity harness). Unblocked now that Phase 7
  closed: `src/microdesign/` is in-tree and the Phase-7
  `ConstExpr` set is ready to be cross-tree-imported as the Expr
  layer of Phase 8's source IR; the scoped comparator
  (`ToolReport`/`Divergence`/`FactCategory`/`ParityScope`/
  `compare_manifest_to_tool_report_in_scope`) is the shape
  `.2c` extends with hierarchy-aware variants. `.2` is now a
  container; no renumbering. Tree-planning, docs-only; no
  `src/`/`tests/` change (`cargo` unchanged-green vs `20a7b4a`);
  `mdbook build book` clean (no `book/` change). Continuous-PNT
  immediately after closing Phase 7 + the
  `PHASE-7-ORACLE-MICRODESIGN` tree at `20a7b4a`. Frontier →
  `.2a` (the source-IR-and-evaluator code-bearing slice;
  unblocked).
- `2026-05-20`: **`.2a` landed — source-level AST IR +
  construction-time elaboration-evaluator (the oracle).** New
  separate top-level module `src/frontend/mod.rs` registered
  via `pub mod frontend` (NOT in `src/ir/` — circuit IR cannot
  express modules/params/packages/generate, the category
  error `.1` rejected). AST IR types: `SourceUnit{seed,
  packages, children, top}` (depth-1 instance tree —
  sufficient to stress every elaboration axis the parity gate
  checks); `Package{name, items}` with `PackageItem::Localparam`;
  `Module{name, params, body}` with `ParamDecl{name, kind,
  expr, value}`; `ModuleItem::Localparam | Instance |
  GenerateIf`; `Instance{inst_name, child_module,
  param_bindings}` with named-binding `ParamBinding{name,
  expr, resolved}`; `GenerateIf{label, else_label, condition,
  taken, then_branch, else_branch}`. Every type derives
  `Debug+Clone+PartialEq+Eq`. **Cross-tree reuse** per `.1`'s
  full-factorization plan: `use crate::microdesign::{eval,
  BinOp, ConstExpr, EvalError, ParamKind}` — Phase 7's
  `ConstExpr`/`eval` are the expression layer for parameter
  defaults / localparam chains / instance bindings / generate
  predicates. Construction-time elaboration-evaluator
  `elaborate(&mut SourceUnit)` walks (1) package localparams →
  `pkg::name` env, (2) top-module parameter ports → `name`
  env, (3) top-module body items (Localparams extend env in
  declaration order; Instance bindings resolve in the PARENT's
  env and populate `ParamBinding.resolved`; `GenerateIf.taken
  = eval(condition) != 0`; else-branch elaborates in a
  sandboxed clone so it doesn't leak — SV's model). The
  builder IS the oracle: every `.value`/`.resolved`/`.taken`
  is set in place; downstream readers read them directly
  without re-evaluating. Rules-first reproducible builder
  `build_acceptable_unit(seed, n_params, n_children)`: one
  `ChaCha8Rng::seed_from_u64` drives everything (no
  `thread_rng`); one package `acc_<seed>_pkg`/`K = (seed % 32)
  + 1`; one child stub `child_<seed>` with `n_params`
  literal-default parameters; one top module `acc_<seed>` with
  `n_params` parameter ports + `n_params` chained localparams
  (L0 references P0, Li references L<i-1>, ±small literal) +
  `n_children` named-binding `child_<seed>` instances + one
  `GenerateIf` with condition `P0 >= acc_<seed>_pkg::K`.
  Resolved in place. **4 unit proofs (all green):**
  `build_acceptable_unit_has_the_documented_shape` (smoke);
  `unit_is_reproducible_and_seed_sensitive` (load-bearing
  reproducibility — same `(seed, shape)` → byte-identical
  `SourceUnit` across rebuilds for seeds `{0,1,7,42,12345}`;
  distinct seeds differ); `elaboration_evaluator_resolves_every_axis`
  (package K positive; literal-rooted top params resolve to
  their literal; localparams re-eval consistently in the
  prefix env; `GenerateIf.taken` matches a fresh eval of the
  condition); **`elaborated_facts_match_a_fresh_reeval_across_the_seed_set`
  — the load-bearing oracle-no-drift invariant** (Phase-8
  counterpart of Phase 7's
  `stored_values_are_consistent_with_a_fresh_reeval`): every
  stored `ParamDecl.value`/`ParamBinding.resolved`/
  `GenerateIf.taken` equals a fresh eval against the
  reconstructed env, across seeds 0..=8 — covers **all** fact
  axes the manifest will carry. `cargo fmt --all --check`
  (re-sorted the `pub mod` declarations alphabetically —
  `frontend` now sits between `emit` and `gen` in
  `src/lib.rs`) / `clippy --all-targets -- -D warnings` /
  `check --all-targets` clean. Full `cargo test` green: lib
  **233 passed** (was 229 + 4 new proofs); `frontend::tests`
  4/4; `microdesign::tests` 8/8 unchanged;
  `tests/microdesign_parity` 15 passed + 1 ignored; pipeline
  121; snapshots 6; bin 5+29+3; doc 0. DUT lane stays
  byte-identical by construction (`frontend` never invoked
  from `gen::*`). No SV/manifest emit (that is `.2b`); no
  harness (that is `.2c`); no ROADMAP/book change. Frontier
  → `.2b` (un-elaborated-SV emitter + elaborated-facts JSON
  manifest emitter, both from the same `.2a` oracle).
- `2026-05-20`: **`.2b` landed — un-elaborated SV emitter +
  elaborated-facts JSON manifest emitter, both from the same
  `.2a` oracle.** `src/frontend/mod.rs` extended with
  `emit_sv(unit) -> String` (one provenance comment, one
  `package <name>; localparam int K = <symbolic expr>;
  endpackage` per `Package`, one
  `module child_<seed> #(parameter int CP<i> = <symbolic
  expr>); endmodule` per child stub, and the top module with
  symbolic parameter ports + chained body localparams +
  named-binding instance instantiations
  `child_<seed> #(.CP<i>(<symbolic expr>)) u_<seed>_<idx> ();`
  + `generate if (<symbolic condition>) begin : g_taken …
  end else begin : g_else … end endgenerate`; all expressions
  via cross-tree-reused `crate::microdesign::expr_to_sv`; the
  `g_taken`/`g_else` `gflag` marker matches Phase 7's
  netname-prefix-scan convention). Manifest types (all `pub`,
  serde `Serialize`+`Deserialize`, `BTreeMap` throughout for
  deterministic key order): `PackageFacts{name, constants:
  BTreeMap<String, i128>}`; `ParamFact{value: i128, expr:
  String}`; `InstanceFact{inst_name, child_module,
  resolved_bindings: BTreeMap<String, i128>}`; `GenerateFact{
  taken: bool}`; `Manifest{seed, top, packages, top_params,
  top_localparams, instances, generate_branches}`.
  `build_manifest(unit)` reads `.2a`'s stored oracle fields
  directly (no re-evaluation — load-bearing on the
  oracle-no-drift proof). `emit_manifest(unit)` serializes via
  `serde_json::to_string_pretty` (byte-stable via `BTreeMap`).
  3 new lib proofs (10 total in `frontend::tests`):
  `emit_sv_is_valid_unresolved_shape` (full structural pin on
  the emitted SV — package/endpackage/child/two instances/
  generate-if labels/endgenerate + at least one symbolic
  chained localparam line); `manifest_mirrors_the_oracle`
  (across the reproducibility-set seeds: parsed JSON matches
  the `.2a` oracle on every fact axis — packages/top_params/
  top_localparams + per-binding resolved values per Instance +
  per-label generate.taken); `sv_and_manifest_are_byte_
  reproducible` (same `(seed, shape)` → byte-identical `.sv`
  + `.json`; distinct seeds differ). Fixed 2
  `single_char_add_str` clippy hits (`push_str(")")` →
  `push(')')`). `cargo fmt --all --check` / `clippy
  --all-targets -- -D warnings` / `check --all-targets`
  clean. Full `cargo test` green: lib **236 passed** (was 233
  + 3 new proofs); `frontend::tests` 10/10 (was 7);
  `microdesign::tests` 8/8 unchanged; `tests/microdesign_parity`
  15 passed + 1 ignored; `tests/pipeline` 121 passed (654s);
  `tests/snapshots` 6 passed; bin 5+29+3; doc 0. DUT lane
  stays byte-identical-by-construction. No ROADMAP advance
  (that is `.2c`); no `book/` change (that is `.2c`).
  Frontier → `.2c` (hierarchy-aware parity harness +
  repo-owned gate; reuses Phase 7's scoped comparator with
  hierarchy-aware variants; tool-gated `#[ignore]`; expected
  to split further per the proven `PHASE-7-ORACLE-MICRODESIGN.2c`
  → `.2c.1`/`.2c.2`/`.2c.2a`/`.2c.2b` decomposition).
- `2026-05-20`: **`.2c` split** into `.2c.1` (build the
  hierarchy-aware parity harness — cargo-portable comparator
  + tool-gated `#[ignore]` real-tool scaffold; no real run,
  no ROADMAP advance, cargo stays green tool-less) and
  `.2c.2` (real `--ignored` run + verify + bank +
  **ROADMAP Phase 8 → done**; gate-blocked, r87
  no-aspirational-claims; may further split per the proven
  `PHASE-7-ORACLE-MICRODESIGN.2c.2` → `.2c.2a`/`.2c.2b`
  decomposition if a tool-capability dependency surfaces).
  Splitting Rules along the exact independently-reviewable
  boundaries `.1`'s design implied — exactly the same shape
  as the proven `PHASE-7-ORACLE-MICRODESIGN.2c` →
  `.2c.1`/`.2c.2` decomposition that closed Phase 7 on
  2026-05-20. Tree-planning, docs-only; no `src/`/`tests/`
  change (`cargo` unchanged-green vs `d67df0c`); `mdbook
  build book` clean (no `book/` change). Continuous-PNT
  immediately after `.2b` landed. Frontier → `.2c.1` (the
  hierarchy-aware-parity-harness build leaf; unblocked).
- `2026-05-20`: **`.2c.1` landed — hierarchy-aware parity
  harness (comparator core + cargo-portable proofs +
  tool-gated `#[ignore]` scaffold).** `src/frontend/mod.rs`
  extended with the Phase-8-specific parity comparator core:
  `ToolReport`/`InstanceToolReport`/`Divergence` (23 variants
  — `SeedMismatch`/`TopMismatch` + per-category
  `{MissingInTool, MissingInManifest, Mismatch}` ×
  `{PackageConstants, TopParams, TopLocalparams,
  GenerateBranches}` + the **load-bearing hierarchy-aware
  additions** `InstanceMissingInTool`/`InstanceMissingInManifest`/
  `InstanceChildModuleMismatch`/`InstanceBindingMissingInTool`/
  `InstanceBindingMissingInManifest`/`InstanceBindingMismatch`)
  + `FactCategory` (7 axes) + `ParityScope` with
  `all()`/`none()`/`only(&[...])` + `.contains` +
  `compare_manifest_to_tool_report` (strict; delegates to
  `_in_scope` with `ParityScope::all()`) +
  `compare_manifest_to_tool_report_in_scope` (the scoped
  walker — the `Instances` arm builds name-keyed `BTreeMap`
  indices from each side for order-independent presence +
  per-binding compares) + `synthetic_tool_report_from_manifest`
  (flattens `packages` to `pkg::name` + projects the oracle
  fields). These types are **parallel to Phase 7's
  microdesign comparator, NOT derived** — the artifact
  differs (Phase 8 carries instances + per-instance bindings
  the Phase 7 single-module set does not). New
  `tests/frontend_parity.rs` carries **10 cargo-portable
  comparator proofs** (all green): baseline agreement across
  `{0,1,7,42,12345}` + per-axis perturbation for top-param /
  top-localparam / package-constant / **instance-binding
  (perturbs ONE binding on ONE instance + asserts no
  spurious divergence on the OTHER)** / generate-branch /
  **instance-presence (drop + add)** / seed+top +
  scoped-comparator + empty-scope self-check. Plus 1
  tool-gated `#[ignore]` `parity_against_real_downstream_elaborator`
  scaffold (any-of-`yosys`/`slang`/`verilator` presence
  guard; corpus-driver loop wired against the same
  `SEEDS`/`N_PARAMS`/`N_CHILDREN`; placeholder for
  `.2c.2`-owned `emit_sv`→shell→extract→compare wiring).
  `cargo fmt`/clippy/check clean. Full `cargo test` green:
  `tests/frontend_parity` **10 passed + 1 ignored**;
  `tests/microdesign_parity` 15+1 unchanged; `tests/pipeline`
  121 passed (758s); `tests/snapshots` 6 passed; lib **236
  passed** (unchanged); bin 5+29+3; doc 0. Portable `cargo
  test` stays green tool-less. No ROADMAP advance (`.2c.2`);
  no `book/` change. Frontier → `.2c.2` (real `--ignored`
  run against a downstream elaborator + verify
  exact-agreement + bank artifact + record ROADMAP Phase 8
  → done; may further split per the proven Phase 7 `.2c.2`
  → `.2c.2a`/`.2c.2b` precedent on a discovered
  tool-capability dependency).

- `2026-05-20`: **`.2c.2` split** on a discovered
  tool-capability dependency. Empirical probe of `yosys
  hierarchy -top acc_0; write_json` (locally-installed
  yosys 0.64) on a Phase-8 `acc_<seed>.sv` immediately
  after `.2c.1` landed at `977c632` showed:
    * yosys covers 5 of the 7 manifest fact categories —
      top params (`.parameter_default_values`), **instances
      + per-instance per-binding values
      (`.cells[<inst>].{type, parameters}` — the
      load-bearing hierarchy-aware axis)**, generate-branch
      (netname-prefix scan);
    * top_localparams + package-constants are folded
      (Phase 7-style; richer-AST tools see them);
    * `proc; opt` collapses the empty-bodied child instances
      away from `.cells` — the fix is to invoke yosys with
      `hierarchy -top` only.
  Per Splitting Rules + the proven `PHASE-7-ORACLE-MICRODESIGN.2c.2`
  → `.2c.2a`/`.2c.2b` discovered-dependency precedent,
  `.2c.2` was split into `.2c.2a` (yosys-specific extractor
  + end-to-end-runnable `#[ignore]` + cargo-portable
  extractor proof; no real run, no ROADMAP advance) +
  `.2c.2b` (real-tool run + verify + bank + **ROADMAP
  Phase 8 → done** with explicit scope caveat;
  gate-blocked, r87 no-aspirational-claims; may further
  split per Phase 7 `.2c.2b` → `.2c.2b.1`/`.2c.2b.2` on a
  discovered ANVIL self-consistency bug). `.2c.2` is now a
  container; no renumbering. Tree-planning, docs-only; no
  `src/`/`tests/` change (`cargo` unchanged-green vs
  `977c632`); `mdbook build book` clean (no `book/`
  change). Frontier → `.2c.2a`.

- `2026-05-20`: **`.2c.2a` landed — yosys hierarchy
  write_json extractor + end-to-end-runnable `#[ignore]`
  harness.** `tests/frontend_parity.rs` extended with
  `parse_yosys_binary_param` (signed-32-bit sign-extension,
  symmetric to Phase 7) + `yosys_hierarchy_write_json_to_tool_report(json, seed)`
  reading `.parameter_default_values` → `top_params`,
  **`.cells[<inst>].{type, parameters}` → `instances` with
  `child_module` + per-binding `resolved_bindings`** (the
  load-bearing hierarchy-aware Phase-8 axis), and
  `.netnames` key prefix scan for `g_taken.`/`g_else.` →
  `generate_branches["g_taken"]`. Folded axes
  (`package_constants`, `top_localparams`) deliberately
  left empty. `yosys_hierarchy_scope()` returns
  `ParityScope::only(&[Seed, Top, TopParams, Instances,
  GenerateBranches])`. `parity_against_real_yosys_hierarchy_write_json`
  rewritten end-to-end: per-seed `emit_sv` →
  `CARGO_TARGET_TMPDIR/frontend-parity-phase8-yosys/acc_<seed>.sv`
  + `emit_manifest` → `.json` + shell `yosys -q -p
  "read_verilog -sv <sv>; hierarchy -top acc_<seed>;
  write_json <out.json>"` (**deliberately no `proc; opt`**
  — the `.2c.2` probe confirmed it collapses
  empty-bodied child instances out of `.cells`); parse →
  `ToolReport` via the new extractor; scoped compare;
  panic on non-empty counterexample list (else `eprintln
  "parity gate clean across N seeds"`); yosys-presence
  guard. The `.2c.1` `parity_against_real_downstream_elaborator`
  is preserved as a friendly no-op pointing at the named
  yosys test. 2 new cargo-portable extractor proofs (12
  portable + 2 ignored total; was 10 + 1):
  `yosys_extractor_reads_a_synthetic_hierarchy_write_json_correctly`
  (hand-built JSON exercising every branch — top params +
  2 instances × 2 bindings each + g_taken alive);
  `yosys_extractor_reports_g_else_when_else_branch_survives`
  (the g_else-survives case → `generate.g_taken=false`).
  `cargo fmt --all --check`/`clippy --all-targets -- -D
  warnings`/`check --all-targets` clean. Full `cargo
  test` green; `tests/frontend_parity` 12 passed + 2
  ignored. Portable `cargo test` stays green tool-less.
  **NO real cargo-test run of the `#[ignore]`** (that is
  `.2c.2b`). No ROADMAP advance; no `book/` change.
  Frontier → `.2c.2b` (run the real `--ignored` gate +
  verify clean + bank artifact + record ROADMAP Phase 8
  → done; may further split per Phase 7 `.2c.2b` →
  `.2c.2b.1`/`.2c.2b.2` on a discovered ANVIL
  self-consistency bug).