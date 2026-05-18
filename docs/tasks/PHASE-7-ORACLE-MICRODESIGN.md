# PHASE-7-ORACLE-MICRODESIGN: Oracle-backed micro-design artifacts

## Metadata

- Tree ID: `PHASE-7-ORACLE-MICRODESIGN`
- Status: `active`
- Roadmap lane: Phase 7 — Oracle-backed micro-design artifacts
- Created: `2026-05-16`
- Last updated: `2026-05-18` (`.2` split → `.2a`/`.2b`/`.2c`; frontier → `.2a`)
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
  Status: `active`
  Goal: `Implement the micro-design generator + manifest + parity harness per .1, behind an explicit artifact-family selector flag, with a matrix/parity gate. Split per the Splitting Rules along the exact independently-reviewable boundaries .1's design named (const-expr/parameter IR + construction-time evaluator / SV emitter + manifest emitter / parity harness + repo-owned gate).`
  Children: `PHASE-7-ORACLE-MICRODESIGN.2a`, `PHASE-7-ORACLE-MICRODESIGN.2b`, `PHASE-7-ORACLE-MICRODESIGN.2c`

- ID: `PHASE-7-ORACLE-MICRODESIGN.2a`
  Status: `pending`
  Goal: `The source-level const-expr/parameter IR + the construction-time evaluator (the oracle). A small typed parameter+localparam dependency DAG of integer constant expressions with their evaluated values (wide-int semantics matching SV constant-expression rules for the bounded integer subset), reproducible from (seed, knobs) via the existing ChaCha8 stream. Separate generator path; NOT threaded through the gate-level circuit IR. Unit-proven: the evaluator's resolved values match by construction; reproducible byte-stable IR for a fixed seed. No SV/manifest emit yet, no harness.`
  Acceptance: `cargo fmt/clippy(-D warnings)/check --all-targets/test green; new const-expr IR + evaluator with unit proofs (evaluation correctness on a curated expr set incl. precedence/width/localparam-chain cases; reproducibility); no emit/harness; no ROADMAP advance; no book/ change.`
  Verification: `pending`
  Commit: `pending`

- ID: `PHASE-7-ORACLE-MICRODESIGN.2b`
  Status: `pending`
  Goal: `Emitters: the un-resolved-where-appropriate SV emitter for the const-expr/parameter IR (rtl_const_expr family — param/localparam chains, expr-derived widths/ranges, generate if/for, package-qualified constants, precedence-sensitive expressions) + the JSON expected-facts manifest emitter (params/localparams/widths/generate/package_constants/const_exprs per .1's schema), both emitted from the same evaluated IR (.2a). Behind an explicit artifact-family flag (default off ⇒ DUT lane byte-identical). Cargo-portable structural proof: emitted declarations/manifest are consistent with the evaluator by construction; reproducible.`
  Acceptance: `cargo fmt/clippy/check/test green; forced-on emits valid SV + a schema-valid manifest, byte-reproducible; default-off byte-identical to the DUT lane; structural-consistency proof; no ROADMAP advance.`
  Verification: `pending`
  Commit: `pending`

- ID: `PHASE-7-ORACLE-MICRODESIGN.2c`
  Status: `pending`
  Goal: `The parity harness + repo-owned gate: a downstream consumer (Yosys write_json / slang|Verilator param introspection) reports resolved facts; compare to the manifest — exact agreement or a retained counterexample (SV+manifest+tool output). Tool-gated (cargo test stays green tool-less — Phase-1 doctrine, like memory/FSM .2.2 + DIFFERENTIAL .2b). Then verify a clean run and record ROADMAP Phase 7 -> done (r87 no-aspirational-claims: verified artifact precedes the promotion).`
  Acceptance: `Reproducible corpus + manifests; parity harness green or retains counterexamples on a real run; ROADMAP Phase 7 -> done only after a verified clean gate; cargo test green tool-less (the parity gate is tool-gated/repo-owned, not in the portable suite).`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `PHASE-7-ORACLE-MICRODESIGN.2a` | `pending` | `.1` design done; `.2` **split** into `.2a` (const-expr/parameter IR + construction-time evaluator/oracle), `.2b` (SV + JSON-manifest emitters behind an artifact-family flag), `.2c` (parity harness + repo-owned gate → ROADMAP Phase 7). `.2a` is the foundational, independently-reviewable IR+evaluator slice (unit-proven, no emit/harness). Unblocked code slice; PHASE-8/9 `.2` reuse this evaluator/manifest core (sequence after). |

## Decisions

- `2026-05-16`: Phase 7 introduces a *second* artifact lane; it must not
  overload the existing DUT generator path (the doctrinal lane
  separation is preserved here and unified later in Phase 9).
- `2026-05-18`: **`.2` split** into `.2a` (const-expr/parameter IR +
  construction-time evaluator/oracle), `.2b` (SV emitter + JSON
  manifest emitter, behind an artifact-family flag), `.2c` (parity
  harness + repo-owned gate → ROADMAP Phase 7). Splitting Rules
  along the exact independently-reviewable boundaries `.1`'s design
  named ("const-expr/parameter IR + construction-time evaluator /
  SV emitter + manifest emitter / parity harness + repo-owned
  gate"); each is separately reviewable and `.2a`'s evaluator +
  `.2b`'s manifest core are the dependency `PHASE-8-FRONTEND-ACCEPT.2`
  and the Phase-9 manifest plumbing reuse. Tree-planning, docs-only
  (~zero contention on the near-complete Phase 6 priority gate —
  the same contention-aware discipline applied all session). `.2`
  is now a container; no renumbering. Frontier → `.2a`.

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
| `2026-05-18` | `PHASE-7-ORACLE-MICRODESIGN.2` (split) | `.2` made a container with children `.2a` (const-expr/parameter IR + construction-time evaluator/oracle), `.2b` (SV + JSON-manifest emitters behind an artifact-family flag), `.2c` (parity harness + repo-owned gate → ROADMAP Phase 7) — the exact independently-reviewable boundaries `.1`'s design named. Tree-planning, docs-only; no `src/`/`tests/` (cargo unchanged-green vs base `e550db1`). | Done. Frontier → `.2a`. |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `PHASE-7-ORACLE-MICRODESIGN.1` | `Docs: PHASE-7-ORACLE-MICRODESIGN.1 oracle-backed micro-design artifact-family design` | Design-only; expected-facts JSON schema + oracle-by-construction strategy + new parity harness + 4 rejected alternatives. No code. |
| `PHASE-7-ORACLE-MICRODESIGN.2` (split) | `Docs: split PHASE-7-ORACLE-MICRODESIGN.2 into .2a (IR+evaluator) / .2b (emitters) / .2c (parity gate)` | Tree-planning, no code. Boundaries per `.1`'s named split candidates. |

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
- `2026-05-18`: **`.2` split** (tree-planning, docs-only, no code)
  — continuous-PNT while Phase 6 `.2.4`/`.3.4b` are gate-blocked
  and all design/research/triage leaves are exhausted; formalising
  the split is the remaining ~zero-contention advance (the heavy
  `.2a`/`.2b`/`.2c` implementation waits for the near-complete
  priority gate to free the machine — same contention-aware
  sequencing applied all session). `.2` → container `.2a`
  (const-expr/parameter IR + construction-time evaluator/oracle;
  unit-proven, no emit/harness), `.2b` (SV emitter + JSON manifest
  emitter from the same evaluated IR, behind an artifact-family
  flag, default-off DUT-byte-identical), `.2c` (parity harness +
  repo-owned gate, tool-gated so `cargo test` stays tool-less →
  ROADMAP Phase 7 only after a verified clean run, r87). Exactly
  the independently-reviewable boundaries `.1` named; `.2a`+`.2b`'s
  evaluator/manifest core is the reuse `PHASE-8-FRONTEND-ACCEPT.2`
  and the Phase-9 plumbing depend on. `cargo` unchanged-green vs
  `e550db1`. Frontier → `.2a`.
