# PHASE-9-MULTI-ARTIFACT-UMBRELLA: Multi-artifact ANVIL umbrella

## Metadata

- Tree ID: `PHASE-9-MULTI-ARTIFACT-UMBRELLA`
- Status: `active`
- Roadmap lane: Phase 9 — Multi-artifact ANVIL umbrella
- Created: `2026-05-16`
- Last updated: `2026-05-20` (**`.2` split** into `.2a` (`ArtifactLane` trait + shared plumbing + L1 DUT wrap + byte-identical regression proof) + `.2b` (L2 microdesign + L3 frontend lane impls under the trait) + `.2c` (`--artifact` CLI flag + book/CI byte-identical verification + ROADMAP Phase 9 → done). **Unblocked** now that **both** post-DUT lanes are delivered (PHASE-7 closed `2026-05-20` at `20a7b4a`; PHASE-8 closed `2026-05-20` at `997b0a6`); `.1`'s blocker recorded "DUT + ≥1 of Phase 7/8" — Phase 7 AND Phase 8 BOTH satisfy. Tree-planning, no code; frontier → `.2a`)
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
  Children: `PHASE-9-MULTI-ARTIFACT-UMBRELLA.1` (done), `PHASE-9-MULTI-ARTIFACT-UMBRELLA.2` (active container: `.2a`, `.2b`, `.2c`)

- ID: `PHASE-9-MULTI-ARTIFACT-UMBRELLA.1`
  Status: `done`
  Goal: `Design the artifact-family selector + shared plumbing abstraction in DEVELOPMENT_NOTES.md / book: lane interface, shared reproducibility/manifest/seed/output contract, CLI/selector surface, migration of existing lanes, rejected alternatives. Design-only.`
  Acceptance: `Design entry with selector/plumbing abstraction + lane-migration plan + >=1 rejected alternative; mdbook clean; no code change.`
  Verification: `DEVELOPMENT_NOTES.md "Phase 9 multi-artifact umbrella selector design (2026-05-18, PHASE-9-MULTI-ARTIFACT-UMBRELLA.1)" entry landed. The point + explicit anti-goal (unify plumbing, NOT the generators — never collapse into "one random-SV generator"). The lanes (L1 DUT RTL Phases 1-6, no semantic manifest; L2 oracle micro-design Phase 7; L3 frontend/elaboration accept Phase 8; future lanes plug in). ArtifactLane trait (name/validate_knobs[lane-scoped]/generate[byte-stable]/manifest[Option — L1 None is typed not a hack]/check_plan[SynthAccept|ParityVsManifest]); shared umbrella-owned plumbing (ChaCha8 seed→artifact + byte-stable cross-platform output; JSON manifest emitter + schema versioning; lane-scoped knob namespace rejecting cross-lane bleed; uniform on-disk layout; uniform CheckPlan the repo-owned gate dispatches). Open Question RESOLVED: a top-level --artifact <lane> flag on the existing anvil binary, default `dut` ⇒ every current invocation + the entire CI-gated book + CI keep working byte-identically (load-bearing vs BOOK-EXAMPLES-RUNNABLE); tool_matrix stays the L1 gate harness. Lane-migration plan (L1 wrapped as default lane, DutLane::generate IS today's generate_design, byte-identical hard regression gate in .2; L2/L3 built to the contract from the start — no retrofit; (lane,seed,lane_knobs)→byte-identical corpus+manifest is a strict superset of today's (seed,knobs) DUT contract with lane prepended + dut default). 4 rejected alternatives (separate binaries / one-generator-mode-flags = the anti-goal / subcommand-only CLI breaks book+CI / defer-until-2-lanes contradicts the standing Decision). .2 proof shape (DUT byte-identical regression incl. book/CI; ≥1 non-DUT lane via --artifact; uniform layout+manifest+lane-scoped knobs) + unblock condition (DUT + ≥1 of Phase 7/8) + split candidates. Design-only; no code; mdbook build book clean; cargo fmt --all --check clean; full cargo test green at base 5c9932c (no src/tests touched).`
  Commit: `Docs: PHASE-9-MULTI-ARTIFACT-UMBRELLA.1 artifact-family selector + shared-plumbing design`

- ID: `PHASE-9-MULTI-ARTIFACT-UMBRELLA.2`
  Status: `active`
  Goal: `Implement the selector + shared plumbing per .1 and migrate the delivered lanes onto it. UNBLOCKED 2026-05-20 — Phase 7 closed at 20a7b4a, Phase 8 closed at 997b0a6 (the .1 design's unblock condition "DUT + ≥1 of Phase 7/8" is satisfied with all 3 delivered lanes: DUT + microdesign + frontend). Split per the Splitting Rules + the proven Phase 7/8 .2→.2a/.2b/.2c precedent that closed both phases, sized to keep each leaf signoff-sized while preserving the load-bearing byte-identical default-dut contract (BOOK-EXAMPLES-RUNNABLE depends on it).`
  Children: `PHASE-9-MULTI-ARTIFACT-UMBRELLA.2a`, `PHASE-9-MULTI-ARTIFACT-UMBRELLA.2b`, `PHASE-9-MULTI-ARTIFACT-UMBRELLA.2c`

- ID: `PHASE-9-MULTI-ARTIFACT-UMBRELLA.2a`
  Status: `pending`
  Goal: `The umbrella trait + shared plumbing + L1 DUT wrap + byte-identical regression proof. Add a new src/umbrella/ (or src/artifact_lane/) module carrying pub trait ArtifactLane per .1's design: name() -> &'static str; validate_knobs(&self, knobs) -> Result<(), Vec<String>> (lane-scoped namespace; rejects cross-lane knob bleed by construction); generate(&self, seed, knobs) -> Result<LaneArtifact, LaneError> (byte-stable across rebuilds — the load-bearing reproducibility contract a strict superset of today's (seed, knobs) DUT contract); manifest(&self) -> Option<String> (typed Optional; L1 None is typed not a hack); check_plan(&self) -> CheckPlan (SynthAccept for L1, ParityVsManifest for L2/L3). Shared umbrella-owned plumbing: ChaCha8 seed→artifact path (already in place per the project convention; surfaced via the trait), byte-stable cross-platform output formatting (uses existing emit::emit_sv for L1; lane-specific for L2/L3 — that is .2b), JSON manifest emitter (typed Optional via the trait method; lane-specific impls in .2b), lane-scoped knob namespace (a wrapper Knobs<Lane> or a runtime check; final shape TBD when .2a lands), uniform on-disk layout (lane-named subdirectories under a single artifact output root), uniform CheckPlan enum the repo-owned gate dispatches. L1 DUT wrap: pub struct DutLane; impl ArtifactLane for DutLane where DutLane::generate IS today's gen::Generator path (zero behavioural change for the default --artifact dut case — the byte-identical hard regression gate is the load-bearing proof). Byte-identical regression proof: a new tests/lane_byte_identical.rs test (or a tests/pipeline.rs extension) that for a fixed seed set + a fixed Config exercises BOTH the trait-dispatched DutLane::generate path AND the direct legacy generate_design path and asserts emit_sv outputs are byte-identical — the proof that --artifact dut preserves every existing book example + every CI gate. No new lane impls for L2/L3 yet (that is .2b); no CLI flag yet (that is .2c); no ROADMAP advance.`
  Acceptance: `cargo fmt/clippy(-D warnings)/check --all-targets/test green; ArtifactLane trait lands + DutLane impl + byte-identical regression proof green for the reproducibility-set seeds; existing book examples + tool_matrix bin tests still green (load-bearing); no ROADMAP advance (that is .2c on the full selector + book/CI verification); no book/ change.`
  Verification: `pending`
  Commit: `pending`

- ID: `PHASE-9-MULTI-ARTIFACT-UMBRELLA.2b`
  Status: `pending`
  Goal: `L2 (microdesign) + L3 (frontend) lane impls of the ArtifactLane trait. pub struct MicrodesignLane; impl ArtifactLane for MicrodesignLane uses crate::microdesign::{build_constexpr_unit, emit_sv, emit_manifest} under the hood; check_plan returns ParityVsManifest. pub struct FrontendLane; impl ArtifactLane for FrontendLane uses crate::frontend::{build_acceptable_unit, emit_sv, emit_manifest} similarly. Each impl produces LaneArtifact carrying the .sv + Option<manifest_json> + the lane name. A trait-dispatched-per-lane proof: across the reproducibility-set seeds for all 3 lanes (Dut, Microdesign, Frontend), the trait-dispatched generate() output equals the direct module call's output byte-for-byte (the cross-lane byte-identical regression). No CLI flag yet (that is .2c); no ROADMAP advance.`
  Acceptance: `cargo fmt/clippy/check/test green; MicrodesignLane + FrontendLane impls land; trait-dispatched-per-lane byte-identical proof green; existing direct-call paths still produce identical output; no CLI flag; no ROADMAP advance; no book/ change.`
  Verification: `pending`
  Commit: `pending`

- ID: `PHASE-9-MULTI-ARTIFACT-UMBRELLA.2c`
  Status: `pending`
  Goal: `--artifact <lane> top-level CLI flag (default dut) on the existing anvil binary + book/CI byte-identical verification + ROADMAP Phase 9 → done. Add the flag to src/main.rs (or wherever the CLI lives); when --artifact dut (the default), invoke DutLane via the trait — byte-identical-to-today. When --artifact microdesign or --artifact frontend, invoke the corresponding lane; emit .sv to stdout + write manifest to a side path (or per .1's uniform-on-disk-layout). Verify book/CI byte-identical: tests/book_examples.rs (or the equivalent CI gate) still passes byte-identically under the new code path (load-bearing vs BOOK-EXAMPLES-RUNNABLE per .1's resolution of the Open Question — the entire CI-gated book + CI keep working byte-identically). Then promote ROADMAP Phase 9 → done with the explicit "all 3 lanes selectable via --artifact, default dut byte-identical" closure note + reconcile book (book/src/ir.md or a new book/src/lanes.md page describing the 3 lanes + the selector) + README phase narrative + CODEBASE_ANALYSIS phase-coverage-map Phase-9 row + MEMORY recent commits. Closes PHASE-9-MULTI-ARTIFACT-UMBRELLA.2 + .2 container + the tree. May split per the proven Phase 7/8 .2c → .2c.1/.2c.2 precedent if the book/CI byte-identical verification surfaces a discovered regression.`
  Acceptance: `--artifact <lane> CLI flag lands with default dut; book + CI byte-identical to today (load-bearing); microdesign + frontend artifacts emit correctly under the flag; ROADMAP Phase 9 → done only after the verified book/CI clean run; .2c + .2 container + tree all → done. No aspirational claims.`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `PHASE-9-MULTI-ARTIFACT-UMBRELLA.2a` | `pending` (unblocked, code-bearing) | **UNBLOCKED 2026-05-20**: Phase 7 closed at `20a7b4a` (oracle-backed micro-design lane delivered) + Phase 8 closed at `997b0a6` (source-level frontend / elaboration accept lane delivered) — the `.1` design's unblock condition "DUT + ≥1 of Phase 7/8" is satisfied with all 3 delivered lanes (DUT + microdesign + frontend). **`.2` split (`2026-05-20`)** per Splitting Rules + the proven Phase 7/8 `.2` → `.2a`/`.2b`/`.2c` decomposition into `.2a` (`ArtifactLane` trait + shared plumbing + L1 DUT wrap + byte-identical regression proof; no other lane impls, no CLI flag), `.2b` (L2 microdesign + L3 frontend lane impls under the trait + per-lane byte-identical regression proof), `.2c` (`--artifact` top-level CLI flag + book/CI byte-identical verification + ROADMAP Phase 9 → done). `.2a` is unblocked and is the next code-bearing slice. The load-bearing constraint throughout: `--artifact dut` (the default) MUST stay byte-identical to today — `BOOK-EXAMPLES-RUNNABLE` and every CI gate depend on it. |

## Decisions

- `2026-05-16`: Designing the umbrella abstraction early (`.1`) is
  in-scope even though implementation (`.2`) is far off, so Phase 7/8
  artifact lanes are built selector-compatible rather than retrofitted.
- `2026-05-20`: **`.2` UNBLOCKED + split**. Phase 7 closed at
  `20a7b4a` and Phase 8 closed at `997b0a6` (both on
  2026-05-20), so `.1`'s blocker "DUT + ≥1 of Phase 7/8" is
  satisfied with all 3 delivered lanes (DUT + microdesign +
  frontend). Per Splitting Rules + the proven Phase 7/8
  `.2` → `.2a`/`.2b`/`.2c` decomposition, `.2` split into
  `.2a` (`ArtifactLane` trait + shared plumbing + L1 DUT
  wrap + byte-identical regression proof — the load-bearing
  proof that `--artifact dut` preserves every existing book
  example + CI gate), `.2b` (L2 microdesign + L3 frontend
  lane impls of the trait + cross-lane byte-identical proof),
  `.2c` (`--artifact <lane>` top-level CLI flag with default
  `dut` + book/CI byte-identical verification + **ROADMAP
  Phase 9 → done**; may split per Phase 7/8 `.2c` →
  `.2c.1`/`.2c.2` precedent if a discovered regression
  surfaces). The load-bearing constraint throughout: the
  default-`dut` path stays byte-identical to today
  (`BOOK-EXAMPLES-RUNNABLE` + every CI gate depend on it).
  `.2` is now a container; no renumbering. Tree-planning,
  docs-only; no `src/`/`tests/` change (`cargo`
  unchanged-green vs `997b0a6`); `mdbook build book`
  clean. Frontier → `.2a`.

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

| `2026-05-20` | `PHASE-9-MULTI-ARTIFACT-UMBRELLA.2` (split) | `.2` made a container with children `.2a` (`ArtifactLane` trait + shared plumbing + L1 DUT wrap + byte-identical regression proof) + `.2b` (L2 microdesign + L3 frontend lane impls under the trait) + `.2c` (`--artifact` CLI flag + book/CI byte-identical verification + ROADMAP Phase 9 → done). UNBLOCKED 2026-05-20 by Phase 7 closure at `20a7b4a` + Phase 8 closure at `997b0a6` — `.1`'s blocker condition satisfied. Mirrors the proven Phase 7/8 `.2` → `.2a`/`.2b`/`.2c` decomposition. Tree-planning, docs-only; no `src/`/`tests/` change (`cargo` unchanged-green vs `997b0a6`); `mdbook build book` clean. | Done. Frontier → `.2a`. |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `PHASE-9-MULTI-ARTIFACT-UMBRELLA.1` | `Docs: PHASE-9-MULTI-ARTIFACT-UMBRELLA.1 artifact-family selector + shared-plumbing design` | Design-only; `ArtifactLane` trait + shared plumbing + default-`dut` `--artifact` flag + lane-migration + 4 rejected alternatives. No code. |

| `PHASE-9-MULTI-ARTIFACT-UMBRELLA.2` (split) | `Docs: split PHASE-9-MULTI-ARTIFACT-UMBRELLA.2 into .2a (trait + DUT wrap) + .2b (microdesign + frontend lane impls) + .2c (--artifact CLI + ROADMAP Phase 9)` | Tree-planning, no code. Unblocked by Phase 7 + Phase 8 closure 2026-05-20. Mirrors Phase 7/8 `.2` → `.2a`/`.2b`/`.2c`. |

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

- `2026-05-20`: **`.2` UNBLOCKED + split.** Phase 7 closed
  at `20a7b4a` and Phase 8 closed at `997b0a6` (both on
  2026-05-20), satisfying `.1`'s blocker condition "DUT +
  ≥1 of Phase 7/8" (all 3 delivered lanes now exist: DUT +
  microdesign + frontend). Per Splitting Rules + the
  proven Phase 7/8 `.2` → `.2a`/`.2b`/`.2c` decomposition,
  `.2` was split into `.2a` (`ArtifactLane` trait + shared
  umbrella-owned plumbing + L1 DUT wrap + byte-identical
  regression proof — the load-bearing proof that
  `--artifact dut` preserves every existing book example +
  CI gate; no other lane impls, no CLI flag yet), `.2b`
  (L2 microdesign + L3 frontend lane impls of the trait +
  cross-lane byte-identical proof; still no CLI flag),
  `.2c` (`--artifact <lane>` top-level CLI flag with
  default `dut` + book/CI byte-identical verification +
  **ROADMAP Phase 9 → done**; may further split per the
  proven Phase 7/8 `.2c` → `.2c.1`/`.2c.2` precedent if a
  discovered regression surfaces during book/CI
  verification). The load-bearing constraint throughout:
  the default-`dut` path stays byte-identical to today
  (`BOOK-EXAMPLES-RUNNABLE` + every CI gate depend on it
  per `.1`'s resolution of the Open Question on the
  selector form). `.2` is now a container; no
  renumbering. Tree-planning, docs-only; no `src/`/
  `tests/` change (`cargo` unchanged-green vs `997b0a6`);
  `mdbook build book` clean (no `book/` change).
  Continuous-PNT immediately after closing Phase 8 +
  the `PHASE-8-FRONTEND-ACCEPT` tree at `997b0a6`.
  Frontier → `.2a` (the trait + DUT wrap + byte-identical
  regression proof; unblocked, code-bearing).
