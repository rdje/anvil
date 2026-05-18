# PHASE-8-FRONTEND-ACCEPT: Frontend/elaboration accept corpora

## Metadata

- Tree ID: `PHASE-8-FRONTEND-ACCEPT`
- Status: `active`
- Roadmap lane: Phase 8 — Frontend/elaboration accept corpora
- Created: `2026-05-16`
- Last updated: `2026-05-18` (`.1` design landed; frontier → `.2`)
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
  Children: `PHASE-8-FRONTEND-ACCEPT.1`, `PHASE-8-FRONTEND-ACCEPT.2`

- ID: `PHASE-8-FRONTEND-ACCEPT.1`
  Status: `done`
  Goal: `Design the source-level parameter/hierarchy/package IR and the accept-corpus expected-facts schema in DEVELOPMENT_NOTES.md / book: why a separate IR, what surfaces it must express, manifest schema, parity harness, rejected alternatives. Design-only.`
  Acceptance: `Design entry with source-IR sketch + manifest schema + >=1 rejected alternative; mdbook clean; no code change.`
  Verification: `DEVELOPMENT_NOTES.md "Phase 8 frontend/elaboration accept-corpus source-IR design (2026-05-18, PHASE-8-FRONTEND-ACCEPT.1)" entry landed. The shift (Phases 1-6 already-elaborated DUT RTL; Phase 7 single-module const-expr oracle; Phase 8 = compact elaboratable HIERARCHIES emitted with parameters UNRESOLVED in the SV text + a manifest of what elaboration must resolve — pressure point = downstream front-end/elaboration). Codebase grounding (post-elaboration scalar circuit IR cannot express modules/param-ports/packages/typedef/generate; Phase 5 ParamEnv & Phase 7 const-expr DAG are sub-models; Phase 8 = first-class source-level AST IR, separate generator path, reuses Phase 7 evaluator+manifest core + seeding/CLI). Source-IR sketch (SourceUnit/Package/Module{params,ports,items}; ModuleItem = Localparam|VarDecl|Typedef|ContinuousAssign|Always(kind)|Instance{params Named|Ordered, ports Named|Ordered|Wildcard, array}|Generate(If|For); Type = Logic|Atom|Enum|Struct|Union|Named|PkgQual; Expr = reused Phase 7 set; params carry construction-time-evaluated values). Manifest extends Phase 7's schema with the instance tree (path→target→resolved child params→port bindings), selected generate branches/iterations, package+typedef resolutions; byte-stable JSON. Oracle-by-construction (generator elaborates at construction time; emits un-elaborated SV + elaborated-facts manifest from the same knowledge; no analysis pass/re-parse/bundled elaborator). Open Question resolved (reuse Phase 7 evaluator+manifest core, extend schema; .2 depends on PHASE-7-ORACLE-MICRODESIGN.2's core; Phase 9 unifies the selector — Phase 8 behind an explicit family flag). Hierarchy-aware parity harness (repo-owned gate + cargo structural-consistency slice). 4 rejected alternatives (reuse circuit IR / emit already-elaborated SV / in-ANVIL SV elaborator / extend Phase 7 const-expr IR in place). .2 proof shape + split. Design-only; no code; mdbook build book clean; cargo fmt --all --check clean; full cargo test green at base f0cff2c (no src/tests touched).`
  Commit: `Docs: PHASE-8-FRONTEND-ACCEPT.1 source-level frontend/elaboration accept-corpus IR design`

- ID: `PHASE-8-FRONTEND-ACCEPT.2`
  Status: `pending`
  Goal: `Implement the source-level IR + accept-corpus generator + manifest + parity harness per .1, behind the artifact-family selector, with a parity gate.`
  Acceptance: `Reproducible accept corpora + manifests; parity green or retained counterexamples; ROADMAP Phase 8 -> done.`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `PHASE-8-FRONTEND-ACCEPT.2` | `pending` | `.1` design **done** (`DEVELOPMENT_NOTES.md`: source-level AST IR sketch; un-elaborated-SV + elaborated-facts-manifest oracle-by-construction; instance-tree manifest schema; hierarchy-aware parity harness; reuses Phase 7 evaluator/manifest core; 4 rejected alternatives; `.2` split candidates). `.2` implements the source IR + construction-time elaboration-evaluator + SV/manifest emitters + parity harness behind the artifact-family flag, with a repo-owned parity gate; **sequence after `PHASE-7-ORACLE-MICRODESIGN.2`** (reuses its evaluator+manifest core). Expected to split. |

## Decisions

- `2026-05-16`: Phase 8 uses a dedicated source-level IR by roadmap
  decree; reusing the gate-level circuit IR is a recorded rejected
  direction (it cannot express the required source surfaces).

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

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `PHASE-8-FRONTEND-ACCEPT.1` | `Docs: PHASE-8-FRONTEND-ACCEPT.1 source-level frontend/elaboration accept-corpus IR design` | Design-only; source-level AST IR sketch + instance-tree manifest schema + oracle-by-construction + reuses Phase 7 core + 4 rejected alternatives. No code. |

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
