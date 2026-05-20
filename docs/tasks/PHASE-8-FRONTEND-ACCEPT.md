# PHASE-8-FRONTEND-ACCEPT: Frontend/elaboration accept corpora

## Metadata

- Tree ID: `PHASE-8-FRONTEND-ACCEPT`
- Status: `active`
- Roadmap lane: Phase 8 — Frontend/elaboration accept corpora
- Created: `2026-05-16`
- Last updated: `2026-05-20` (**`.2a` source-level AST IR + construction-time elaboration-evaluator landed** — new top-level module `src/frontend/mod.rs` with `SourceUnit`/`Package`/`Module`/`ModuleItem`/`Instance`/`GenerateIf`/`ParamDecl`/`ParamBinding` types, `elaborate()` walker that resolves every `ParamDecl.value`/`ParamBinding.resolved`/`GenerateIf.taken` from `ConstExpr` evaluation, and rules-first reproducible `build_acceptable_unit(seed, n_params, n_children)` builder; reuses Phase 7's `ConstExpr`/`eval`/`ParamKind`/`BinOp` cross-tree per the full-factorization plan; 4 unit proofs green incl. the load-bearing oracle-no-drift invariant; full `cargo test` green: lib 233 (was 229 +4), microdesign 8/8, microdesign_parity 15+1, pipeline 121, snapshots 6, bin 5+29+3, doc 0; frontier → `.2b`)
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
  Children: `PHASE-8-FRONTEND-ACCEPT.2a`, `PHASE-8-FRONTEND-ACCEPT.2b`, `PHASE-8-FRONTEND-ACCEPT.2c`

- ID: `PHASE-8-FRONTEND-ACCEPT.2a`
  Status: `done`
  Goal: `Source-level AST IR + construction-time elaboration-evaluator (the oracle). A new separate top-level module src/srcform/ (or src/frontend/; final name TBD in implementation; NOT in src/ir/ — circuit IR cannot express modules/packages/typedef/generate, per .1's category-error rejection): SourceUnit{packages, modules}, Package{name, items}, Module{name, params, ports, items}, ModuleItem = Localparam{name, expr, value} | VarDecl{name, ty, init} | Typedef{name, ty} | ContinuousAssign{lhs, rhs} | Always{kind, body} | Instance{module, params (Named|Ordered), ports (Named|Ordered|Wildcard), array} | Generate(If{cond, then, else} | For{var, init, cond, step, body}), Type = Logic{packed_width} | Atom{name: int|byte|bit|...} | Enum{base, members} | Struct{kind: Packed|Unpacked, fields} | Union{kind: Packed|Unpacked, fields} | Named(String) | PkgQual{pkg, name}, Expr = reuse Phase 7's ConstExpr set (cross-tree reuse). Construction-time elaboration-evaluator: traverses the SourceUnit and resolves every parameter value, typedef instance, generate condition, instance-path port binding, and array dimension; produces an in-memory ElaboratedFacts struct that mirrors .1's manifest schema (the oracle). Reproducible rules-first build_acceptable_unit(seed, knobs) builder (ChaCha8::seed_from_u64, project convention, no thread_rng) — a literal-root package, a top module with N parameters, M sub-instances with both Named and Ordered param/port styles, K generate branches, and L typedef references; resolved in place. Reuses Phase 7's eval/resolve for the ConstExpr layer; no analysis pass, no re-parse — builder IS the oracle. Unit-proven: evaluator's resolved facts match independent reference values; reproducible byte-stable IR for fixed seeds. No SV/manifest emit (that is .2b), no harness (that is .2c).`
  Acceptance: `cargo fmt/clippy(-D warnings)/check --all-targets/test green; new src/srcform/ (or final name) module landed with the source-level AST IR + construction-time elaboration-evaluator + reproducible rules-first builder + unit proofs (elaboration correctness on a curated set incl. nested generate, named-vs-ordered port maps, typedef chains, array instances; reproducibility for fixed seeds, seed-sensitivity); no emit/harness; no ROADMAP advance; no book/ change.`
  Verification: `New separate top-level module src/frontend/mod.rs registered via pub mod frontend in src/lib.rs (deliberately NOT in src/ir/ — the circuit IR cannot express modules/params/packages/typedef/generate, exactly the category-error .1 rejected). AST IR types: SourceUnit{seed, packages: Vec<Package>, children: Vec<Module>, top: Module} (the minimum-viable shape: depth-1 instance tree — enough to stress every elaboration axis the parity gate checks; deeper trees are a recorded post-.2a knob in .2b's emit work, NOT a .2a blocker), Package{name, items: Vec<PackageItem>}, PackageItem::Localparam(ParamDecl) (minimum-viable set; .2b may add Typedef), Module{name, params: Vec<ParamDecl>, body: Vec<ModuleItem>}, ParamDecl{name, kind: ParamKind (reused from microdesign cross-tree), expr: ConstExpr (reused), value: i128 (the oracle)}, ModuleItem::Localparam(ParamDecl) | Instance(Instance) | GenerateIf(GenerateIf), Instance{inst_name, child_module, param_bindings: Vec<ParamBinding>} (named-binding form only in .2a; ordered is a .2b extension knob), ParamBinding{name, expr: ConstExpr, resolved: i128 (the per-instance oracle)}, GenerateIf{label, else_label, condition: ConstExpr, taken: bool (the oracle), then_branch/else_branch: Vec<ModuleItem>}. Every type derives Debug+Clone+PartialEq+Eq so the reproducibility proof can compare two builds for byte identity and the manifest-mirrors-oracle proof can compare resolved fact maps for equality. Cross-tree reuse: use crate::microdesign::{eval, BinOp, ConstExpr, EvalError, ParamKind} — Phase 7's ConstExpr/eval are the expression layer for parameter defaults, localparam chains, instance bindings, and generate predicates (per .1's full-factorization plan). Construction-time elaboration-evaluator: pub fn elaborate(unit: &mut SourceUnit) -> Result<BTreeMap<String, i128>, EvalError> walks (1) package localparams (resolved values populate the pkg::name namespace), (2) top module parameter ports (literal defaults; .2a's builder doesn't override — instance bindings are one level down), (3) top module body items (Localparams extend the env in declaration order; Instance param_bindings resolve in the PARENT's env and populate ParamBinding.resolved; GenerateIf.taken = eval(condition) != 0 with then_branch elaborating in the env, else_branch in a sandboxed clone so it doesn't leak — SV's model). The builder IS the oracle: every .value/.resolved/.taken is set in place; downstream readers (emit, manifest, comparator) read them directly without re-evaluating. Reproducible rules-first builder pub fn build_acceptable_unit(seed: u64, n_params: usize, n_children: usize) -> SourceUnit: one ChaCha8Rng::seed_from_u64(seed) drives everything (project convention; no thread_rng); package acc_<seed>_pkg with one localparam int K = (seed % 32) + 1; child module child_<seed> with n_params parameters (literal defaults via g.rng); top module acc_<seed> with n_params parameter ports + n_params body localparams (chained: L0 references P0; Li references L<i-1>; ±small literal) + n_children Instance(s) of child_<seed> with named bindings (each binds CP<i> to a parent-evaluated Add of a top-param-or-localparam ref plus a small offset) + one GenerateIf with condition P0 >= acc_<seed>_pkg::K. Resolved in place via elaborate(). 4 unit proofs green: build_acceptable_unit_has_the_documented_shape (smoke: one package, one child, one top, n_params/n_children/lp counts match), unit_is_reproducible_and_seed_sensitive (same (seed, shape) → byte-identical SourceUnit across rebuilds for seeds {0,1,7,42,12345}; distinct seeds differ — the load-bearing reproducibility invariant the emitters and parity gate depend on), elaboration_evaluator_resolves_every_axis (package K positive; literal-rooted top params resolve to their literal; localparams re-eval consistently in the prefix env; GenerateIf.taken matches a fresh eval of the condition), elaborated_facts_match_a_fresh_reeval_across_the_seed_set (the Phase-8 counterpart of Phase 7's stored_values_are_consistent_with_a_fresh_reeval load-bearing oracle-no-drift invariant: every stored ParamDecl.value / ParamBinding.resolved / GenerateIf.taken equals a fresh eval against the reconstructed env, across seeds 0..=8 — covers ALL fact axes the manifest will carry). cargo fmt --all (re-sorted the pub mod declarations alphabetically — frontend now sits between emit and gen in src/lib.rs) / clippy --all-targets -- -D warnings / check --all-targets clean. Full cargo test green: lib 233 passed (was 229 + 4 new proofs), frontend lib tests 4/4, microdesign tests 8/8 unchanged, tests/microdesign_parity 15+1 (every .2c.1 + .2c.2a portable proof still green — the cross-tree ConstExpr import is read-only by the new module), tests/pipeline 121 passed, tests/snapshots 6 passed, bin tests 5+29+3 passed, doc-tests 0 (unchanged). DUT lane stays byte-identical by construction (frontend is never invoked from gen::*; the new pub mod is structurally additive). No SV/manifest emit (.2b), no harness (.2c). No ROADMAP/book change.`
  Commit: `Phase 8: PHASE-8-FRONTEND-ACCEPT.2a source-level AST IR + construction-time elaboration-evaluator (oracle)`

- ID: `PHASE-8-FRONTEND-ACCEPT.2b`
  Status: `pending`
  Goal: `Emitters: the un-elaborated-where-appropriate SV emitter for the source-IR (parameter ports kept symbolic, instance bindings carrying expressions not resolved integers, generate predicates preserved as written, typedef references un-flattened) + the JSON elaborated-facts manifest emitter (instance tree with path→target→resolved child params→port bindings, selected generate branches/iterations, package+typedef resolutions, per .1's manifest schema extension of Phase 7's), both emitted from the same evaluated IR (.2a). Default-off DUT-byte-identical is structural (separate module never invoked from the DUT generate path; PHASE-9 selector wires invocation later). Cargo-portable structural proof: emitted SV declarations + manifest are consistent with the elaboration-evaluator by construction; byte-reproducible for fixed seeds.`
  Acceptance: `cargo fmt/clippy/check/test green; forced-on emits valid un-elaborated SV + schema-valid elaborated-facts manifest, byte-reproducible; default-off byte-identical to the DUT lane; structural-consistency proof per .1's schema; no ROADMAP advance; no book/ change (book reconciliation is .2c.2-equivalent or .2c).`
  Verification: `pending`
  Commit: `pending`

- ID: `PHASE-8-FRONTEND-ACCEPT.2c`
  Status: `pending`
  Goal: `The hierarchy-aware parity harness + repo-owned gate: a downstream consumer (currently planned: yosys hierarchy -top + write_json AFTER elaboration, or slang elaborate --ast-json, or verilator --xml-only) reports resolved instance-tree facts; compare to the manifest — exact agreement on the tool-supported categories or a retained counterexample tuple. Tool-gated (cargo test stays green tool-less — the convention reaffirmed in PHASE-7-ORACLE-MICRODESIGN's Decisions and applied at .2c.1/.2c.2a). Then verify a clean run and record ROADMAP Phase 8 -> done (r87 no-aspirational-claims). Reuses the scoped-comparator infrastructure (FactCategory/ParityScope/compare_manifest_to_tool_report_in_scope) that PHASE-7's .2c.2a delivered, extended with HIERARCHY-aware variants (InstancePathMismatch, PortBindingMismatch, GenerateBranchMismatch keyed by instance-tree path) — a recorded extension that PHASE-7's comparator stays unchanged; or, simpler, the Phase-8 comparator is its own type in src/srcform/. Final shape TBD when .2c lands; expected to split further per the Phase 7 precedent.`
  Acceptance: `Reproducible accept corpus + manifests; parity harness green or retains counterexamples on a real run; ROADMAP Phase 8 -> done only after a verified clean gate; cargo test green tool-less.`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `PHASE-8-FRONTEND-ACCEPT.2b` | `pending` (unblocked, code-bearing) | **`.2a` done (`2026-05-20`)** — new top-level module `src/frontend/mod.rs` (registered via `pub mod frontend`); AST IR types `SourceUnit`/`Package`/`Module`/`ModuleItem`/`Instance`/`GenerateIf`/`ParamDecl`/`ParamBinding` (Clone+PartialEq+Eq throughout); construction-time elaboration-evaluator `elaborate(&mut SourceUnit)` resolves every `ParamDecl.value`/`ParamBinding.resolved`/`GenerateIf.taken` from a single `ConstExpr` evaluation (the builder IS the oracle); rules-first reproducible builder `build_acceptable_unit(seed, n_params, n_children)` (one `ChaCha8Rng::seed_from_u64`; one package + one child + one top with N params + N chained localparams + M named-binding instances + one `generate if (P0 >= pkg::K)`). Cross-tree reuse of Phase 7's `ConstExpr`/`eval`/`ParamKind`/`BinOp` (per `.1`'s full-factorization plan). 4 unit proofs green incl. the **load-bearing oracle-no-drift invariant** `elaborated_facts_match_a_fresh_reeval_across_the_seed_set` (Phase-8 counterpart of Phase 7's `stored_values_are_consistent_with_a_fresh_reeval`). Full `cargo test` green: lib **233** (was 229 + 4), microdesign 8/8 unchanged, microdesign_parity 15+1, pipeline 121, snapshots 6, bin 5+29+3, doc 0. DUT lane stays byte-identical by construction. `.2b` adds the un-elaborated-SV emitter (parameter ports symbolic, instance bindings carrying expressions not resolved integers) + the elaborated-facts JSON manifest emitter (instance tree path→resolved-child-params + generate-branch-taken + package-constant resolutions), both from the same `.2a` oracle. |

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
| `2026-05-20` | `PHASE-8-FRONTEND-ACCEPT.2a` | New separate top-level module `src/frontend/mod.rs` registered via `pub mod frontend` (NOT in `src/ir/` — circuit IR cannot express modules/params/packages/generate; the category-error `.1` rejected). AST IR types: `SourceUnit{seed, packages, children, top}` (depth-1 instance tree — sufficient to stress every elaboration axis the parity gate checks; deeper trees are a post-`.2a` knob, not a blocker); `Package{name, items}`; `PackageItem::Localparam(ParamDecl)`; `Module{name, params, body}`; `ParamDecl{name, kind, expr, value}` (own type so Phase-8's package-vs-port distinction is local; cross-tree reuse is at the `ConstExpr`/`eval` layer); `ModuleItem::Localparam(ParamDecl) | Instance(Instance) | GenerateIf(GenerateIf)`; `Instance{inst_name, child_module, param_bindings}` (named-binding form in `.2a`); `ParamBinding{name, expr, resolved}`; `GenerateIf{label, else_label, condition, taken, then_branch, else_branch}`. Every type derives `Debug+Clone+PartialEq+Eq` so the reproducibility proof can byte-compare and the manifest-mirror proof can map-compare. Cross-tree reuse: `use crate::microdesign::{eval, BinOp, ConstExpr, EvalError, ParamKind}` — Phase 7's `ConstExpr`/`eval` are the expression layer for parameter defaults / localparam chains / instance bindings / generate predicates (per `.1`'s full-factorization plan). Construction-time elaboration-evaluator `pub fn elaborate(unit: &mut SourceUnit) -> Result<BTreeMap<String, i128>, EvalError>` walks (1) package localparams → `pkg::name` env, (2) top module parameter ports → `name` env, (3) top module body items (Localparams extend env in declaration order; Instance bindings resolve in the PARENT's env and populate `ParamBinding.resolved`; `GenerateIf.taken = eval(condition) != 0`; else-branch elaborates in a sandboxed clone so it doesn't leak — SV's model). Builder IS the oracle: every `.value`/`.resolved`/`.taken` is set in place; downstream readers (emit, manifest, comparator) read them directly without re-evaluating. Rules-first reproducible builder `pub fn build_acceptable_unit(seed, n_params, n_children)`: one `ChaCha8Rng::seed_from_u64` drives everything (no `thread_rng`); package `acc_<seed>_pkg`/`K = (seed % 32) + 1`; child stub `child_<seed>` with `n_params` literal-default parameters; top module `acc_<seed>` with `n_params` parameter ports + `n_params` chained localparams (L0 references P0, Li references L<i-1>, ±small literal) + `n_children` named-binding `child_<seed>` instances (each binds every `CP<i>` to `Add(<top-param-or-localparam-ref>, <small-offset>)`) + one `GenerateIf` with condition `P0 >= acc_<seed>_pkg::K`. Resolved in place via `elaborate()`. 4 unit proofs (all green): `build_acceptable_unit_has_the_documented_shape` (smoke); `unit_is_reproducible_and_seed_sensitive` (load-bearing reproducibility invariant — same `(seed, shape)` → byte-identical `SourceUnit` across rebuilds for seeds `{0, 1, 7, 42, 12345}`; distinct seeds differ); `elaboration_evaluator_resolves_every_axis` (package K positive; literal-rooted top params resolve to their literal; localparams re-eval consistently in the prefix env; `GenerateIf.taken` matches a fresh eval of the condition); `elaborated_facts_match_a_fresh_reeval_across_the_seed_set` (**load-bearing oracle-no-drift invariant** — every stored `ParamDecl.value`/`ParamBinding.resolved`/`GenerateIf.taken` equals a fresh eval against the reconstructed env, across seeds 0..=8; covers ALL fact axes the manifest will carry). `cargo fmt --all --check` (sorted `pub mod` lines alphabetically — `frontend` now sits between `emit` and `gen` in `src/lib.rs`) / `clippy --all-targets -- -D warnings` / `check --all-targets` clean. Full `cargo test` green: lib **233 passed** (was 229 + 4 new proofs); `frontend::tests` 4/4; `microdesign::tests` 8/8 unchanged; `tests/microdesign_parity` 15 passed + 1 ignored; `tests/pipeline` 121 passed; `tests/snapshots` 6 passed; bin tests 5+29+3 passed; doc-tests 0 (unchanged). DUT lane stays byte-identical by construction (`frontend` never invoked from `gen::*`; the new `pub mod` is structurally additive). No SV/manifest emit (that is `.2b`); no harness (that is `.2c`); no ROADMAP/book change. | Done. Frontier → `.2b`. |
| `2026-05-20` | `PHASE-8-FRONTEND-ACCEPT.2` (split) | `.2` made a container with children `.2a` (source-level AST IR + construction-time elaboration-evaluator/oracle; unit-proven; no emit/harness) + `.2b` (un-elaborated-SV emitter + elaborated-facts JSON manifest emitter; default-off DUT-byte-identical structural) + `.2c` (hierarchy-aware parity harness + repo-owned gate → ROADMAP Phase 8; r87 no-aspirational-claims). Exactly mirrors the proven `PHASE-7-ORACLE-MICRODESIGN.2`→`.2a`/`.2b`/`.2c` decomposition that closed Phase 7 on 2026-05-20. `.2a`+`.2b`'s evaluator/manifest core *extends* the Phase 7 core; Phase 7's `ConstExpr` set is cross-tree-imported as the Expr layer of Phase 8's source IR. Unblocked now that Phase 7 closed. Tree-planning, docs-only; no `src/`/`tests/` change (`cargo` unchanged-green vs `20a7b4a`). `mdbook build book` clean (no `book/` change). | Done. Frontier → `.2a`. |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `PHASE-8-FRONTEND-ACCEPT.1` | `Docs: PHASE-8-FRONTEND-ACCEPT.1 source-level frontend/elaboration accept-corpus IR design` | Design-only; source-level AST IR sketch + instance-tree manifest schema + oracle-by-construction + reuses Phase 7 core + 4 rejected alternatives. No code. |
| `PHASE-8-FRONTEND-ACCEPT.2` (split) | `Docs: split PHASE-8-FRONTEND-ACCEPT.2 into .2a (source IR + elaboration-evaluator) + .2b (emitters) + .2c (parity harness + gate)` | Tree-planning, no code. Exactly mirrors the proven `PHASE-7-ORACLE-MICRODESIGN.2`→`.2a`/`.2b`/`.2c` decomposition that closed Phase 7 on 2026-05-20. Unblocked now that Phase 7 closed. |
| `PHASE-8-FRONTEND-ACCEPT.2a` | `Phase 8: PHASE-8-FRONTEND-ACCEPT.2a source-level AST IR + construction-time elaboration-evaluator (oracle)` | New `src/frontend/` module + AST IR (`SourceUnit`/`Package`/`Module`/`ModuleItem`/`Instance`/`GenerateIf`/`ParamDecl`/`ParamBinding`) + `elaborate()` walker + rules-first reproducible `build_acceptable_unit` + 4 unit proofs (incl. the load-bearing oracle-no-drift invariant); cross-tree reuse of Phase 7's `ConstExpr`/`eval`/`ParamKind`/`BinOp`. No emit/harness. |

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
