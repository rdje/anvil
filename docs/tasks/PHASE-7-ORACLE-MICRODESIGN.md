# PHASE-7-ORACLE-MICRODESIGN: Oracle-backed micro-design artifacts

## Metadata

- Tree ID: `PHASE-7-ORACLE-MICRODESIGN`
- Status: `active`
- Roadmap lane: Phase 7 — Oracle-backed micro-design artifacts
- Created: `2026-05-16`
- Last updated: `2026-05-18` (`.1` design landed; frontier → `.2`)
- Owner: repo-local workflow

## Goal

Add a new artifact family: small, self-contained `.sv` files with a
**known expected-facts manifest** (e.g. `rtl_const_expr`-style) — param
/ localparam dependency chains, expression-derived widths and ranges,
generate conditions and loop bounds driven by expressions,
package-qualified constants, precedence-sensitive expressions — each
with a machine-checkable expected-facts contract and downstream parity
checks.

## Non-Goals

- Broad cone complexity / DUT RTL stress — that is the existing Phase
  1–4 lane; Phase 7 is the opposite (tiny, oracle-backed).
- A bundled reference simulator — facts are obviously-checkable
  elaboration facts, not full RTL semantics (project non-goal).
- The artifact-family selector that unifies lanes — that is Phase 9.

## Acceptance Criteria

- Reproducible micro-design corpus generator (seeded, byte-stable).
- Explicit expected-facts manifest per emitted file.
- Parity checks: downstream consumers either agree with the manifest or
  a counterexample is retained.

## Task Tree

- ID: `PHASE-7-ORACLE-MICRODESIGN`
  Status: `active`
  Goal: `Reproducible oracle-backed micro-design corpus with expected-facts contract and downstream parity checks.`
  Children: `PHASE-7-ORACLE-MICRODESIGN.1`, `PHASE-7-ORACLE-MICRODESIGN.2`

- ID: `PHASE-7-ORACLE-MICRODESIGN.1`
  Status: `done`
  Goal: `Design the micro-design artifact family in DEVELOPMENT_NOTES.md / book: expected-facts schema, generation strategy (param/expr chains), reproducibility contract, parity-check harness shape, relationship to the existing DUT lane, rejected alternatives. Design-only.`
  Acceptance: `Design entry with expected-facts schema sketch and >=1 rejected alternative; mdbook clean; no code change.`
  Verification: `DEVELOPMENT_NOTES.md "Phase 7 oracle-backed micro-design artifact family design (2026-05-18, PHASE-7-ORACLE-MICRODESIGN.1)" entry landed. Records: the conceptual shift (Phases 1-6 = random RTL, no semantic oracle; Phase 7 = tiny .sv whose elaboration facts are known by construction + a machine-checkable manifest — pressure point is front-end constant-expr/param/elaboration correctness). Codebase grounding (the scalar-u32 gate-level circuit IR has no parameter/localparam/generate/package/typed-constant concept; WidthExpr/ParamEnv is width-only; Phase 7 needs its own small source-level constant/parameter IR, a separate generator path, reusing seeding/CLI/reproducibility). rtl_const_expr artifact family per ROADMAP (param/localparam dependency chains; expr-derived widths/ranges; generate if/for; package-qualified constants; precedence-sensitive expressions). Expected-facts JSON manifest schema sketch (params/localparams/widths/generate/package_constants/const_exprs). Oracle-by-construction generation strategy (the generator evaluates every const-expr/param node as it builds it and emits both the .sv and the manifest from the same resolved values — no analysis pass, no re-parse; the generator IS the oracle; valid-by-construction/rules-first). Reproducibility contract (seed,knobs → byte-identical .sv + .json). Parity-check harness (separate from the tool_matrix lint/synth DUT gate; downstream consumer reports resolved facts → compared to manifest; exact agreement or retained counterexample; cargo-portable structural-equivalence formalization + repo-owned gate for the genuine tool parity, mirroring memory/FSM). Boundaries (Phase 8 = richer source-level hierarchy/package IR; Phase 9 = the family selector; Phase 7 lands behind an explicit family flag, no selector). 4 rejected alternatives (reuse circuit IR / generate-then-parse / bundle reference elaborator / facts-as-comments). .2 proof shape + split candidates. Design-only; no code; mdbook build book clean; cargo fmt --all --check clean; full cargo test green at base 5db4ac9 (no src/tests touched).`
  Commit: `Docs: PHASE-7-ORACLE-MICRODESIGN.1 oracle-backed micro-design artifact-family design`

- ID: `PHASE-7-ORACLE-MICRODESIGN.2`
  Status: `pending`
  Goal: `Implement the micro-design generator + manifest + parity harness per .1, behind an explicit artifact-family selector flag, with a matrix/parity gate.`
  Acceptance: `Reproducible corpus + manifests; parity harness green or retains counterexamples; ROADMAP Phase 7 -> done.`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `PHASE-7-ORACLE-MICRODESIGN.2` | `pending` | `.1` design **done** (`DEVELOPMENT_NOTES.md`: expected-facts JSON schema; oracle-by-construction generation; reproducibility; parity harness; Phase-8/9 boundaries; 4 rejected alternatives; `.2` split candidates). `.2` implements the const-expr/parameter IR + construction-time evaluator + SV/manifest emitters + parity harness behind an explicit artifact-family flag, with a repo-owned parity gate. Expected to split per the Splitting Rules (IR+evaluator / emitters / harness+gate). |

## Decisions

- `2026-05-16`: Phase 7 introduces a *second* artifact lane; it must not
  overload the existing DUT generator path (the doctrinal lane
  separation is preserved here and unified later in Phase 9).

## Open Questions

- Manifest format (JSON schema vs sidecar comments) — **resolved by
  `.1`**: a typed **JSON manifest** per `.sv` (params/localparams/
  widths/generate/package_constants/const_exprs). Sidecar comments
  rejected (not machine-checkable without re-parsing; couples the
  oracle to comment formatting).
- Whether the parity harness reuses `tool_matrix` or is new —
  **resolved by `.1`**: a **new, separate** parity harness (the
  `tool_matrix` gate proves lint/synth *acceptance*; Phase 7 proves
  *fact agreement* — a different contract). Cargo-portable
  structural-equivalence formalization + a repo-owned gate for the
  genuine downstream parity (cargo cannot shell yosys/verilator —
  the Phase-1 convention), mirroring memory/FSM.

## Blockers

- None for `.1`. `.2` benefits from but is not hard-blocked by Phase 5
  parameterization; `.1` will record whether `.2` should wait.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-05-18` | `PHASE-7-ORACLE-MICRODESIGN.1` | `DEVELOPMENT_NOTES.md` Phase 7 design entry landed (conceptual shift; codebase grounding — own source-level const/param IR, separate generator path; `rtl_const_expr` family; expected-facts JSON schema; oracle-by-construction generation; reproducibility; new parity harness; Phase-8/9 boundaries; 4 rejected alternatives; `.2` split). Design-only, no code; `mdbook build book` clean; `cargo fmt --all --check` clean; full `cargo test` green at base `5db4ac9` (no `src/`/`tests/` touched). | Done. |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `PHASE-7-ORACLE-MICRODESIGN.1` | `Docs: PHASE-7-ORACLE-MICRODESIGN.1 oracle-backed micro-design artifact-family design` | Design-only; expected-facts JSON schema + oracle-by-construction strategy + new parity harness + 4 rejected alternatives. No code. |

## Changelog

- `2026-05-16`: Created task tree as part of the directive to task-tree
  every remaining roadmap phase.
- `2026-05-18`: **`.1` design landed** (design-only, no code) —
  continuous-PNT while both remaining Phase 6 leaves (`.2.4`/`.3.4b`)
  are gate-blocked. `DEVELOPMENT_NOTES.md` "Phase 7 oracle-backed
  micro-design artifact family design": the oracle-by-construction
  shift (the generator evaluates every const-expr/param node as it
  builds it and emits the `.sv` + JSON manifest from the same
  resolved values — no analysis pass, no re-parse), its own
  source-level const/parameter IR (separate generator path; the
  circuit IR has no param/generate/package concept), the
  expected-facts JSON schema, the reproducibility contract, a new
  parity harness (distinct from the `tool_matrix` DUT gate;
  cargo-portable structural-equivalence + repo-owned gate), the
  Phase-8/9 boundaries, 4 rejected alternatives, and the `.2` proof
  shape + split candidates. Both Open Questions resolved (typed JSON
  manifest; new separate parity harness). `mdbook` clean. Frontier →
  `.2` (implement; expected to split IR+evaluator / emitters /
  harness+gate).
