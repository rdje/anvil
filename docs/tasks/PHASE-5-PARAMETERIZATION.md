# PHASE-5-PARAMETERIZATION: Parameterized modules and instances

## Metadata

- Tree ID: `PHASE-5-PARAMETERIZATION`
- Status: `active`
- Roadmap lane: Phase 5 — Parameterization
- Created: `2026-05-16`
- Last updated: `2026-05-16` (`.1` design landed; Phase 4 done → `.2` unblocked & split into `.2.1`–`.2.4`; frontier → `.2.1`)
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
  Children: `PHASE-5-PARAMETERIZATION.2.1`, `.2.2`, `.2.3`, `.2.4`

- ID: `PHASE-5-PARAMETERIZATION.2.1`
  Status: `pending`
  Goal: `IR + emitter scaffold: WidthExpr{Lit,Param} + per-module ParamEnv{name,range,design_value} + Instance.param_bindings; post-construction parameterization pass marking the sound affine-in-W subset; opt-in knob width_parameterization_prob (f64, default 0.0, serde-default pattern); width_decl/module-header/instance #(.W()) emission. Non-parameterized output byte-identical.`
  Acceptance: `cargo fmt/clippy(-D warnings)/check/test green; focused unit test: a parameterized module round-trips IR->validate->emit with parameter W and [W-1:0]; default-off emits byte-identical SV to pre-change for a fixed seed; mdbook clean.`
  Verification: `pending`
  Commit: `pending`

- ID: `PHASE-5-PARAMETERIZATION.2.2`
  Status: `pending`
  Goal: `Instantiation substitution: reproducible param-value pick from range via g.rng in src/gen/hierarchy.rs (between child selection and input-binding); Instance.param_bindings recorded; child ports bound at resolved width so existing exact-equality child-width validation holds.`
  Acceptance: `Focused proof: a parent instantiates one template at >=2 in-range values, validate_design passes, emitted instances carry #(.W(v)); all four ConstructionStrategy values; cargo gates green.`
  Verification: `pending`
  Commit: `pending`

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
| 1 | `PHASE-5-PARAMETERIZATION.2.1` | `pending` | `.1` design landed; Phase 4 is `done` so `.2` is unblocked. `.2.1` is the IR/emit scaffold every later sub-leaf depends on. |
| 2 | `PHASE-5-PARAMETERIZATION.2.2` | `pending` | Instantiation substitution depends on the `.2.1` scaffold. |
| 3 | `PHASE-5-PARAMETERIZATION.2.3` | `pending` | Identity rule depends on the `WidthExpr` form from `.2.1`. |
| 4 | `PHASE-5-PARAMETERIZATION.2.4` | `pending` | Matrix gate + Phase 5 closure; depends on `.2.1`–`.2.3`. |

## Decisions

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
| `2026-05-16` | `PHASE-5-PARAMETERIZATION.2.1` | `pending` | `pending` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `PHASE-5-PARAMETERIZATION.1` | `Docs: PHASE-5-PARAMETERIZATION.1 parameterization design` | Design-only; DEVELOPMENT_NOTES.md entry. |
| `PHASE-5-PARAMETERIZATION.2.1` | `pending` | `pending` |

## Changelog

- `2026-05-16`: Created task tree as part of the directive to task-tree
  every remaining roadmap phase.
- `2026-05-16`: `.1` design landed (architecture (C) chosen; rejected
  (A)/(B)/(C')). Phase 4 reached `done`, unblocking `.2`; `.2` split
  into `.2.1`–`.2.4` (scaffold → instantiation → identity → gate).
  Frontier → `.2.1`.
