# PHASE-9-MULTI-ARTIFACT-UMBRELLA: Multi-artifact ANVIL umbrella

## Metadata

- Tree ID: `PHASE-9-MULTI-ARTIFACT-UMBRELLA`
- Status: `active`
- Roadmap lane: Phase 9 — Multi-artifact ANVIL umbrella
- Created: `2026-05-16`
- Last updated: `2026-05-18` (`.1` design landed; frontier → `.2`, blocked until ≥2 lanes)
- Owner: repo-local workflow

## Goal

Add an **artifact-family selector** so one tool drives every
valid-by-construction synthesizable family without overloading one
generator path with contradictory promises; unify reproducibility,
manifests, seed handling, knob plumbing, corpus output layout, and
downstream checking across families, while preserving the doctrinal
lane separation (synthesizable DUT RTL; oracle-backed positive
micro-design; frontend/elaboration accept; future valid synthesizable
lanes).

## Non-Goals

- Inventing new artifact families here — Phase 9 unifies the families
  delivered by Phases 1–8, it does not add new ones.
- Blurring the lanes into one "random SV files" notion (the explicit
  anti-goal this phase exists to prevent).

## Acceptance Criteria

- An explicit mode/lane selector covering the DUT RTL lane, the
  oracle-backed micro-design lane, and the frontend/elaboration accept
  lane.
- Unified reproducibility/manifest/seed/knob/output/downstream-check
  plumbing across lanes.
- ANVIL can honestly present itself as the go-to multi-lane tool.

## Task Tree

- ID: `PHASE-9-MULTI-ARTIFACT-UMBRELLA`
  Status: `active`
  Goal: `Unify all delivered artifact lanes under one explicit selector with shared plumbing.`
  Children: `PHASE-9-MULTI-ARTIFACT-UMBRELLA.1`, `PHASE-9-MULTI-ARTIFACT-UMBRELLA.2`

- ID: `PHASE-9-MULTI-ARTIFACT-UMBRELLA.1`
  Status: `done`
  Goal: `Design the artifact-family selector + shared plumbing abstraction in DEVELOPMENT_NOTES.md / book: lane interface, shared reproducibility/manifest/seed/output contract, CLI/selector surface, migration of existing lanes, rejected alternatives. Design-only.`
  Acceptance: `Design entry with selector/plumbing abstraction + lane-migration plan + >=1 rejected alternative; mdbook clean; no code change.`
  Verification: `DEVELOPMENT_NOTES.md "Phase 9 multi-artifact umbrella selector design (2026-05-18, PHASE-9-MULTI-ARTIFACT-UMBRELLA.1)" entry landed. The point + explicit anti-goal (unify plumbing, NOT the generators — never collapse into "one random-SV generator"). The lanes (L1 DUT RTL Phases 1-6, no semantic manifest; L2 oracle micro-design Phase 7; L3 frontend/elaboration accept Phase 8; future lanes plug in). ArtifactLane trait (name/validate_knobs[lane-scoped]/generate[byte-stable]/manifest[Option — L1 None is typed not a hack]/check_plan[SynthAccept|ParityVsManifest]); shared umbrella-owned plumbing (ChaCha8 seed→artifact + byte-stable cross-platform output; JSON manifest emitter + schema versioning; lane-scoped knob namespace rejecting cross-lane bleed; uniform on-disk layout; uniform CheckPlan the repo-owned gate dispatches). Open Question RESOLVED: a top-level --artifact <lane> flag on the existing anvil binary, default `dut` ⇒ every current invocation + the entire CI-gated book + CI keep working byte-identically (load-bearing vs BOOK-EXAMPLES-RUNNABLE); tool_matrix stays the L1 gate harness. Lane-migration plan (L1 wrapped as default lane, DutLane::generate IS today's generate_design, byte-identical hard regression gate in .2; L2/L3 built to the contract from the start — no retrofit; (lane,seed,lane_knobs)→byte-identical corpus+manifest is a strict superset of today's (seed,knobs) DUT contract with lane prepended + dut default). 4 rejected alternatives (separate binaries / one-generator-mode-flags = the anti-goal / subcommand-only CLI breaks book+CI / defer-until-2-lanes contradicts the standing Decision). .2 proof shape (DUT byte-identical regression incl. book/CI; ≥1 non-DUT lane via --artifact; uniform layout+manifest+lane-scoped knobs) + unblock condition (DUT + ≥1 of Phase 7/8) + split candidates. Design-only; no code; mdbook build book clean; cargo fmt --all --check clean; full cargo test green at base 5c9932c (no src/tests touched).`
  Commit: `Docs: PHASE-9-MULTI-ARTIFACT-UMBRELLA.1 artifact-family selector + shared-plumbing design`

- ID: `PHASE-9-MULTI-ARTIFACT-UMBRELLA.2`
  Status: `pending`
  Goal: `Implement the selector + shared plumbing per .1 and migrate the delivered lanes onto it. Blocked until at least two artifact lanes exist (Phase 1–4 DUT lane + one of Phase 7/8).`
  Acceptance: `One tool selects all delivered lanes; shared plumbing proven; ROADMAP Phase 9 -> done.`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `PHASE-9-MULTI-ARTIFACT-UMBRELLA.2` | `pending` (blocked) | `.1` design **done** (`DEVELOPMENT_NOTES.md`: `ArtifactLane` trait + shared plumbing; default-`dut` `--artifact` flag preserving book/CI byte-identically; lane-migration plan; 4 rejected alternatives). `.2` implements the selector + shared plumbing + migrates the delivered lanes. **Blocked until ≥2 delivered artifact lanes exist** (the DUT lane plus ≥1 of Phase 7/8 implemented — Phases 7.2/8.2 are themselves gated on the in-flight infra). Phase 7/8 will be built selector-compatible per this design, so `.2` is a wrap, not a retrofit. |

## Decisions

- `2026-05-16`: Designing the umbrella abstraction early (`.1`) is
  in-scope even though implementation (`.2`) is far off, so Phase 7/8
  artifact lanes are built selector-compatible rather than retrofitted.

## Open Questions

- Whether the selector is a CLI subcommand, a top-level `--artifact`
  flag, or separate binaries — **resolved by `.1`**: a top-level
  **`--artifact <lane>` flag on the existing `anvil` binary,
  default `dut`** (every current invocation + the CI-gated book +
  CI keep working byte-identically — load-bearing vs
  `BOOK-EXAMPLES-RUNNABLE`). Subcommand-only and separate-binaries
  forms rejected (break the flat CLI / book-CI surface; fragment
  shared plumbing).

## Blockers

- `PHASE-9-MULTI-ARTIFACT-UMBRELLA.2` is blocked by: fewer than two
  delivered artifact lanes. Unblock condition: the DUT lane plus at
  least one of Phase 7/8 lanes exist. Run `.1` (design) meanwhile.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-05-18` | `PHASE-9-MULTI-ARTIFACT-UMBRELLA.1` | `DEVELOPMENT_NOTES.md` Phase 9 umbrella design entry landed (anti-goal: unify plumbing not generators; `ArtifactLane` trait + shared seed/manifest/knob/output/check plumbing; default-`dut` `--artifact` flag preserving book/CI byte-identically; L1-wrap lane-migration plan; 4 rejected alternatives; Open Question resolved; `.2` proof shape + unblock condition). Design-only, no code; `mdbook build book` clean; `cargo fmt --all --check` clean; full `cargo test` green at base `5c9932c` (no `src/`/`tests/` touched). | Done. |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `PHASE-9-MULTI-ARTIFACT-UMBRELLA.1` | `Docs: PHASE-9-MULTI-ARTIFACT-UMBRELLA.1 artifact-family selector + shared-plumbing design` | Design-only; `ArtifactLane` trait + shared plumbing + default-`dut` `--artifact` flag + lane-migration + 4 rejected alternatives. No code. |

## Changelog

- `2026-05-16`: Created task tree as part of the directive to task-tree
  every remaining roadmap phase.
- `2026-05-18`: **`.1` design landed** (design-only, no code) —
  continuous-PNT while Phase 6 `.2.4`/`.3.4b` are gate-blocked.
  `DEVELOPMENT_NOTES.md` "Phase 9 multi-artifact umbrella selector
  design": the explicit anti-goal (unify plumbing, not generators),
  the `ArtifactLane` trait + umbrella-owned shared
  seed/manifest/knob/output/check plumbing, the **default-`dut`
  `--artifact` flag** chosen to keep the entire CI-gated book + CI
  byte-identical (load-bearing vs `BOOK-EXAMPLES-RUNNABLE`), the
  L1-wrap lane-migration plan (no retrofit; Phases 7/8 built to the
  contract), 4 rejected alternatives, and the `.2` proof shape +
  unblock condition (≥2 delivered lanes). Open Question resolved
  (`--artifact` flag, not subcommands/separate binaries). `mdbook`
  clean. Frontier → `.2` (blocked until ≥2 lanes).
