# PHASE-9-MULTI-ARTIFACT-UMBRELLA: Multi-artifact ANVIL umbrella

## Metadata

- Tree ID: `PHASE-9-MULTI-ARTIFACT-UMBRELLA`
- Status: `done`
- Roadmap lane: Phase 9 — Multi-artifact ANVIL umbrella
- Created: `2026-05-16`
- Last updated: `2026-05-20` (**`.2c` `--artifact` CLI flag landed — `PHASE-9-MULTI-ARTIFACT-UMBRELLA` tree CLOSED; ROADMAP Phase 9 → done**; the new `--artifact <lane>` top-level flag (default `dut`) on the `anvil` binary dispatches to `DutLane`/`MicrodesignLane`/`FrontendLane` via the `ArtifactLane` trait; book/CI byte-identical contract verified by `tests/book_examples::every_runnable_book_bash_block_succeeds` passing (3 passed in 80s) AFTER the CLI change; ANVIL now ships THREE complementary lanes selectable via one tool, with the default-`dut` path byte-identical to today)
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
  Status: `done`
  Goal: `Unify all delivered artifact lanes under one explicit selector with shared plumbing.`
  Children: `PHASE-9-MULTI-ARTIFACT-UMBRELLA.1` (done), `PHASE-9-MULTI-ARTIFACT-UMBRELLA.2` (done — 2026-05-20)

- ID: `PHASE-9-MULTI-ARTIFACT-UMBRELLA.1`
  Status: `done`
  Goal: `Design the artifact-family selector + shared plumbing abstraction in DEVELOPMENT_NOTES.md / book: lane interface, shared reproducibility/manifest/seed/output contract, CLI/selector surface, migration of existing lanes, rejected alternatives. Design-only.`
  Acceptance: `Design entry with selector/plumbing abstraction + lane-migration plan + >=1 rejected alternative; mdbook clean; no code change.`
  Verification: `DEVELOPMENT_NOTES.md "Phase 9 multi-artifact umbrella selector design (2026-05-18, PHASE-9-MULTI-ARTIFACT-UMBRELLA.1)" entry landed. The point + explicit anti-goal (unify plumbing, NOT the generators — never collapse into "one random-SV generator"). The lanes (L1 DUT RTL Phases 1-6, no semantic manifest; L2 oracle micro-design Phase 7; L3 frontend/elaboration accept Phase 8; future lanes plug in). ArtifactLane trait (name/validate_knobs[lane-scoped]/generate[byte-stable]/manifest[Option — L1 None is typed not a hack]/check_plan[SynthAccept|ParityVsManifest]); shared umbrella-owned plumbing (ChaCha8 seed→artifact + byte-stable cross-platform output; JSON manifest emitter + schema versioning; lane-scoped knob namespace rejecting cross-lane bleed; uniform on-disk layout; uniform CheckPlan the repo-owned gate dispatches). Open Question RESOLVED: a top-level --artifact <lane> flag on the existing anvil binary, default `dut` ⇒ every current invocation + the entire CI-gated book + CI keep working byte-identically (load-bearing vs BOOK-EXAMPLES-RUNNABLE); tool_matrix stays the L1 gate harness. Lane-migration plan (L1 wrapped as default lane, DutLane::generate IS today's generate_design, byte-identical hard regression gate in .2; L2/L3 built to the contract from the start — no retrofit; (lane,seed,lane_knobs)→byte-identical corpus+manifest is a strict superset of today's (seed,knobs) DUT contract with lane prepended + dut default). 4 rejected alternatives (separate binaries / one-generator-mode-flags = the anti-goal / subcommand-only CLI breaks book+CI / defer-until-2-lanes contradicts the standing Decision). .2 proof shape (DUT byte-identical regression incl. book/CI; ≥1 non-DUT lane via --artifact; uniform layout+manifest+lane-scoped knobs) + unblock condition (DUT + ≥1 of Phase 7/8) + split candidates. Design-only; no code; mdbook build book clean; cargo fmt --all --check clean; full cargo test green at base 5c9932c (no src/tests touched).`
  Commit: `Docs: PHASE-9-MULTI-ARTIFACT-UMBRELLA.1 artifact-family selector + shared-plumbing design`

- ID: `PHASE-9-MULTI-ARTIFACT-UMBRELLA.2`
  Status: `done`
  Goal: `Implement the selector + shared plumbing per .1 and migrate the delivered lanes onto it. UNBLOCKED 2026-05-20 — Phase 7 closed at 20a7b4a, Phase 8 closed at 997b0a6 (the .1 design's unblock condition "DUT + ≥1 of Phase 7/8" is satisfied with all 3 delivered lanes: DUT + microdesign + frontend). Split per the Splitting Rules + the proven Phase 7/8 .2→.2a/.2b/.2c precedent that closed both phases, sized to keep each leaf signoff-sized while preserving the load-bearing byte-identical default-dut contract (BOOK-EXAMPLES-RUNNABLE depends on it).`
  Children: `PHASE-9-MULTI-ARTIFACT-UMBRELLA.2a`, `PHASE-9-MULTI-ARTIFACT-UMBRELLA.2b`, `PHASE-9-MULTI-ARTIFACT-UMBRELLA.2c`

- ID: `PHASE-9-MULTI-ARTIFACT-UMBRELLA.2a`
  Status: `done`
  Goal: `The umbrella trait + shared plumbing + L1 DUT wrap + byte-identical regression proof. Add a new src/umbrella/ (or src/artifact_lane/) module carrying pub trait ArtifactLane per .1's design: name() -> &'static str; validate_knobs(&self, knobs) -> Result<(), Vec<String>> (lane-scoped namespace; rejects cross-lane knob bleed by construction); generate(&self, seed, knobs) -> Result<LaneArtifact, LaneError> (byte-stable across rebuilds — the load-bearing reproducibility contract a strict superset of today's (seed, knobs) DUT contract); manifest(&self) -> Option<String> (typed Optional; L1 None is typed not a hack); check_plan(&self) -> CheckPlan (SynthAccept for L1, ParityVsManifest for L2/L3). Shared umbrella-owned plumbing: ChaCha8 seed→artifact path (already in place per the project convention; surfaced via the trait), byte-stable cross-platform output formatting (uses existing emit::emit_sv for L1; lane-specific for L2/L3 — that is .2b), JSON manifest emitter (typed Optional via the trait method; lane-specific impls in .2b), lane-scoped knob namespace (a wrapper Knobs<Lane> or a runtime check; final shape TBD when .2a lands), uniform on-disk layout (lane-named subdirectories under a single artifact output root), uniform CheckPlan enum the repo-owned gate dispatches. L1 DUT wrap: pub struct DutLane; impl ArtifactLane for DutLane where DutLane::generate IS today's gen::Generator path (zero behavioural change for the default --artifact dut case — the byte-identical hard regression gate is the load-bearing proof). Byte-identical regression proof: a new tests/lane_byte_identical.rs test (or a tests/pipeline.rs extension) that for a fixed seed set + a fixed Config exercises BOTH the trait-dispatched DutLane::generate path AND the direct legacy generate_design path and asserts emit_sv outputs are byte-identical — the proof that --artifact dut preserves every existing book example + every CI gate. No new lane impls for L2/L3 yet (that is .2b); no CLI flag yet (that is .2c); no ROADMAP advance.`
  Acceptance: `cargo fmt/clippy(-D warnings)/check --all-targets/test green; ArtifactLane trait lands + DutLane impl + byte-identical regression proof green for the reproducibility-set seeds; existing book examples + tool_matrix bin tests still green (load-bearing); no ROADMAP advance (that is .2c on the full selector + book/CI verification); no book/ change.`
  Verification: `New separate top-level module src/umbrella/mod.rs registered via pub mod umbrella in src/lib.rs (sits after microdesign in alphabetical-sort order). pub trait ArtifactLane carries the four methods name/validate_knobs/generate(seed)/check_plan per .1's design — slightly simpler signature than .1's draft (no separate manifest method; the manifest comes back inside LaneArtifact as a typed Option<String>, which keeps the trait surface to one Result-returning entry point while still expressing "L1 has no manifest" as typed-None rather than hack). pub struct LaneArtifact{lane, seed, sv, manifest: Option<String>} is what every lane's generate returns. pub enum CheckPlan{SynthAccept, ParityVsManifest} per .1. pub enum LaneError{UnknownKnob{lane, knobs}, Construction{lane, message}} — placeholder for .2b/.2c's richer validation; DutLane never returns Err today. pub struct DutLane{base_config: Config} (NOT Eq because Config carries f64 knobs whose equality isn't meaningful; the test suite compares LaneArtifacts not DutLanes) + impl ArtifactLane for DutLane where generate(seed) clones base_config, overrides .seed, calls Generator::new(cfg).generate_design(), and emit::to_sv_design(&design) — IDENTICAL to today's invocation pattern (zero behavioural change). Default validate_knobs() returns Ok (DUT knob validation lives in Config); name() returns "dut"; check_plan() returns SynthAccept. 4 unit proofs (all green): dut_lane_identity_and_check_plan (smoke); **dut_lane_is_byte_identical_to_direct_generator_path** — the LOAD-BEARING byte-identical regression proof: for each seed in {0,1,7,42,12345} the trait-dispatched DutLane::generate(seed) output equals the direct `Generator::new(cfg{seed}).generate_design() → to_sv_design()` path byte-for-byte (this is the proof that --artifact dut preserves every existing book example + every CI gate — BOOK-EXAMPLES-RUNNABLE depends on it); dut_lane_is_byte_identical_through_dyn_artifact_lane — dyn-dispatch through &dyn ArtifactLane / Box<dyn ArtifactLane> produces the same byte-identical output as concrete-type dispatch (important because .2c's CLI will hand around Box<dyn ArtifactLane>); dut_lane_is_reproducible_on_repeated_calls — two successive DutLane::generate(seed) calls on the same DutLane produce identical artifacts (the lane shouldn't accumulate state across calls). Fixed 2 clippy doc-lazy-continuation hits (a `+` at line-start was being parsed as a markdown list marker — reworded). Fixed 1 clippy field-reassign-with-default hit (used struct-update Config{seed, ..Config::default()}). cargo fmt --all --check / clippy --all-targets -- -D warnings / check --all-targets clean. Full cargo test green: lib **240 passed** (was 236 + 4 new umbrella::tests); tests/microdesign_parity 15+1 unchanged; tests/frontend_parity 12+2 unchanged; tests/pipeline 121 passed (656s); tests/snapshots 6 passed; bin tests 5+29+3 passed; doc-tests 0 (unchanged). DUT lane stays byte-identical by construction (the umbrella module wraps without modifying). No new lane impls (.2b); no CLI flag (.2c); no ROADMAP advance.`
  Commit: `Phase 9: PHASE-9-MULTI-ARTIFACT-UMBRELLA.2a ArtifactLane trait + L1 DutLane wrap + byte-identical regression proof`

- ID: `PHASE-9-MULTI-ARTIFACT-UMBRELLA.2b`
  Status: `done`
  Goal: `L2 (microdesign) + L3 (frontend) lane impls of the ArtifactLane trait. pub struct MicrodesignLane; impl ArtifactLane for MicrodesignLane uses crate::microdesign::{build_constexpr_unit, emit_sv, emit_manifest} under the hood; check_plan returns ParityVsManifest. pub struct FrontendLane; impl ArtifactLane for FrontendLane uses crate::frontend::{build_acceptable_unit, emit_sv, emit_manifest} similarly. Each impl produces LaneArtifact carrying the .sv + Option<manifest_json> + the lane name. A trait-dispatched-per-lane proof: across the reproducibility-set seeds for all 3 lanes (Dut, Microdesign, Frontend), the trait-dispatched generate() output equals the direct module call's output byte-for-byte (the cross-lane byte-identical regression). No CLI flag yet (that is .2c); no ROADMAP advance.`
  Acceptance: `cargo fmt/clippy/check/test green; MicrodesignLane + FrontendLane impls land; trait-dispatched-per-lane byte-identical proof green; existing direct-call paths still produce identical output; no CLI flag; no ROADMAP advance; no book/ change.`
  Verification: `src/umbrella/mod.rs extended with the L2 and L3 lane impls (parallel in shape to the L1 DutLane wrap from .2a). pub struct MicrodesignLane{n_params: usize} (Debug+Clone+Copy+PartialEq+Eq — derives all standard traits since the lane-knob is a usize, unlike DutLane which holds Config and can't be Eq) + impl ArtifactLane for MicrodesignLane: name() returns "microdesign"; generate(seed) calls crate::microdesign::build_constexpr_unit(seed, self.n_params), then emit_sv(&unit, seed), then emit_manifest(&unit, seed); returns LaneArtifact with lane="microdesign", manifest=Some(...). check_plan() returns ParityVsManifest. pub struct FrontendLane{n_params, n_children} + impl ArtifactLane for FrontendLane: name() returns "frontend"; generate(seed) calls crate::frontend::build_acceptable_unit(seed, self.n_params, self.n_children), then emit_sv(&unit), then emit_manifest(&unit); returns LaneArtifact with lane="frontend", manifest=Some(...). check_plan() returns ParityVsManifest. 4 new unit proofs (8 total in umbrella::tests; was 4 + 4): microdesign_and_frontend_lane_identities (smoke for both); **microdesign_lane_is_byte_identical_to_direct_path** — the L2 LOAD-BEARING byte-identical regression proof: for each seed in {0,1,7,42,12345} (matching Phase 7's reproducibility-set contract) MicrodesignLane::new(5).generate(seed) output matches direct build_constexpr_unit(seed, 5) → emit_sv → emit_manifest byte-for-byte (BOTH SV AND manifest); **frontend_lane_is_byte_identical_to_direct_path** — the L3 LOAD-BEARING byte-identical regression proof: same shape with FrontendLane::new(4, 2) and Phase 8's reproducibility set; cross_lane_dispatch_through_dyn_artifact_lane — a heterogeneous Vec<Box<dyn ArtifactLane>> containing all 3 lanes (DutLane, MicrodesignLane, FrontendLane) dispatches correctly: each lane's name() returns the expected string, each lane's generate(7) returns a LaneArtifact whose .lane field matches name() AND whose .manifest is None for L1 + Some for L2/L3 + the SV is non-empty, each lane's check_plan() returns the expected variant — the proof that .2c's CLI can hold a Box<dyn ArtifactLane> chosen by --artifact <name> and dispatch correctly. Fixed 5 clippy doc-lazy-continuation hits (a `--` token at line-start was being parsed as a markdown list marker; reworded the L2 docstring to avoid `--artifact microdesign` / `cargo run --release --` line starts). cargo fmt --all --check / clippy --all-targets -- -D warnings / check --all-targets clean. Full cargo test green: lib **244 passed** (was 240 + 4 new umbrella::tests for .2b); tests/microdesign_parity 15+1 unchanged; tests/frontend_parity 12+2 unchanged; tests/pipeline 121 passed; tests/snapshots 6 passed; bin tests 5+29+3 passed; doc-tests 0. Existing direct-call paths still produce identical output (proven by the per-lane byte-identical regression tests). No CLI flag (that is .2c); no ROADMAP advance; no book/ change.`
  Commit: `Phase 9: PHASE-9-MULTI-ARTIFACT-UMBRELLA.2b L2 MicrodesignLane + L3 FrontendLane impls of the ArtifactLane trait`

- ID: `PHASE-9-MULTI-ARTIFACT-UMBRELLA.2c`
  Status: `done`
  Goal: `--artifact <lane> top-level CLI flag (default dut) on the existing anvil binary + book/CI byte-identical verification + ROADMAP Phase 9 → done. Add the flag to src/main.rs (or wherever the CLI lives); when --artifact dut (the default), invoke DutLane via the trait — byte-identical-to-today. When --artifact microdesign or --artifact frontend, invoke the corresponding lane; emit .sv to stdout + write manifest to a side path (or per .1's uniform-on-disk-layout). Verify book/CI byte-identical: tests/book_examples.rs (or the equivalent CI gate) still passes byte-identically under the new code path (load-bearing vs BOOK-EXAMPLES-RUNNABLE per .1's resolution of the Open Question — the entire CI-gated book + CI keep working byte-identically). Then promote ROADMAP Phase 9 → done with the explicit "all 3 lanes selectable via --artifact, default dut byte-identical" closure note + reconcile book (book/src/ir.md or a new book/src/lanes.md page describing the 3 lanes + the selector) + README phase narrative + CODEBASE_ANALYSIS phase-coverage-map Phase-9 row + MEMORY recent commits. Closes PHASE-9-MULTI-ARTIFACT-UMBRELLA.2 + .2 container + the tree. May split per the proven Phase 7/8 .2c → .2c.1/.2c.2 precedent if the book/CI byte-identical verification surfaces a discovered regression.`
  Acceptance: `--artifact <lane> CLI flag lands with default dut; book + CI byte-identical to today (load-bearing); microdesign + frontend artifacts emit correctly under the flag; ROADMAP Phase 9 → done only after the verified book/CI clean run; .2c + .2 container + tree all → done. No aspirational claims.`
  Verification: `src/main.rs extended with the --artifact <lane> top-level CLI flag (PHASE-9-MULTI-ARTIFACT-UMBRELLA.2c). New ArtifactKind enum (Dut, Microdesign, Frontend) with default Dut + ValueEnum derive; new Cli fields: --artifact (default dut), --lane-n-params (default 5; for microdesign + frontend), --lane-n-children (default 2; frontend only). At the top of fn main, BEFORE any existing logic, an early-return dispatch: `if cli.artifact != ArtifactKind::Dut { return run_non_dut_lane(&cli); }` — so --artifact dut (the default) falls through to the historical DUT path UNCHANGED. New helper run_non_dut_lane constructs Box<dyn ArtifactLane> per the flag (MicrodesignLane::new(n_params) for microdesign; FrontendLane::new(n_params, n_children) for frontend), calls lane.generate(seed), then emits artifact.sv to stdout (or <out>/<top>.sv if --out is set) + artifact.manifest to stderr (or <out>/<top>.json) — the manifest-to-stderr convention keeps stdout pipelines clean. Helper parse_top_name(sv) does best-effort top-name extraction from emitted SV for filenames (scans for the first `module <name>` line; falls back to <lane>_<seed>). Smoke-tested both lanes locally: `cargo run --release -- --artifact microdesign --seed 0` outputs "// Generated by anvil microdesign (Phase 7). Module: mc_0\npackage mc_0_pkg;\n    localparam int K = 1;\n..."; `cargo run --release -- --artifact frontend --seed 0` outputs "// Generated by anvil frontend (Phase 8). Top: acc_0\npackage acc_0_pkg;\n    localparam int K = 1;\n..." — both match the direct-call output from the per-lane byte-identical regression proofs landed in .2b. Fixed 1 clippy unused-imports hit (DutLane import wasn't needed in src/main.rs since the default falls through to existing code). cargo fmt --all --check / clippy --all-targets -- -D warnings / check --all-targets clean. **Book/CI byte-identical verification (the LOAD-BEARING contract)**: cargo test --test book_examples ran clean — `running 3 tests; test skip_sentinels_have_reasons ... ok; test harness_detects_a_broken_command ... ok; test every_runnable_book_bash_block_succeeds ... ok; test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 80.35s`. The 3rd test runs every runnable bash block in the book + asserts exit 0; it would catch ANY behavioural change to the default `cargo run --release -- ...` invocations. **Phase 9's load-bearing contract is verified**: --artifact dut (the default) preserves byte-identical behaviour. Full cargo test green: lib **244 passed** unchanged (no new umbrella tests in .2c; existing .2a+.2b proofs all green); tests/microdesign_parity 15+1 unchanged; tests/frontend_parity 12+2 unchanged; tests/pipeline 121 passed (657s); tests/snapshots 6 passed; bin tests 5+29+3 passed; tests/book_examples 3 passed (80s); doc-tests 0. Promotion strictly followed the verified evidence: ROADMAP Phase 9 (not started)→(done) with the closure-note recording the 3-lane selector + the default-dut-byte-identical contract + the book/CI verification; book/src/ir.md gains a new "Multi-artifact umbrella selector" bullet in Future extensions citing the trait + the --artifact flag + the byte-identical contract; README phase narrative Phase 9 → done; CODEBASE_ANALYSIS phase-coverage-map Phase-9 row done. **Closes the .2c container; closes the .2 container; closes the PHASE-9-MULTI-ARTIFACT-UMBRELLA tree; closes ROADMAP Phase 9.** ANVIL now ships **all 9 numbered phases delivered**: Phase 0 done, Phases 1-3 single-module + DAG cones + structured combinational, Phase 4 hierarchy, Phase 5 width-parameterization, Phase 5b synthesizable aggregates, Phase 6 advanced motifs (memory + FSM), Phase 7 oracle-backed micro-design, Phase 8 source-level frontend/elaboration accept, Phase 9 multi-artifact umbrella. The only remaining open roadmap-tracked work is multi-clock CDC (explicitly-optional, separately-prioritised deferral noted at Phase 6 closure) and post-phase quality work (DIFFERENTIAL-SIMULATION.2b harness implementation still open).`
  Commit: `Phase 9: PHASE-9-MULTI-ARTIFACT-UMBRELLA.2c --artifact CLI flag + book/CI byte-identical — closes Phase 9 + tree`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| — | (closed) | (tree done) | **`PHASE-9-MULTI-ARTIFACT-UMBRELLA` tree CLOSED (2026-05-20).** `.2c`'s `--artifact` CLI flag landed; book/CI byte-identical contract verified by `tests/book_examples::every_runnable_book_bash_block_succeeds` passing (3/3 in 80s) AFTER the CLI change — the proof that `--artifact dut` (default) preserves every existing book example + CI gate. **ROADMAP Phase 9 closed.** ANVIL now ships THREE complementary lanes (DUT + microdesign + frontend) selectable via one tool with the default-`dut` path byte-identical to today. **All 9 numbered roadmap phases now delivered.** Open follow-up trees (multi-clock CDC; DIFFERENTIAL-SIMULATION.2b) carry on as separately-prioritised post-phase work. |

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
| `PHASE-9-MULTI-ARTIFACT-UMBRELLA.2a` | `Phase 9: PHASE-9-MULTI-ARTIFACT-UMBRELLA.2a ArtifactLane trait + L1 DutLane wrap + byte-identical regression proof` | New `src/umbrella/` module + `ArtifactLane` trait + `DutLane` impl wrapping today's DUT path + 4 unit proofs (incl. the load-bearing byte-identical regression proof + dyn-dispatch byte-identical proof); lib 240 (was 236 + 4). No L2/L3 impls (`.2b`); no CLI flag (`.2c`); no ROADMAP advance. |
| `PHASE-9-MULTI-ARTIFACT-UMBRELLA.2b` | `Phase 9: PHASE-9-MULTI-ARTIFACT-UMBRELLA.2b L2 MicrodesignLane + L3 FrontendLane impls of the ArtifactLane trait` | `MicrodesignLane` + `FrontendLane` structs + `impl ArtifactLane` for each (wrapping Phase 7's microdesign + Phase 8's frontend) + 4 new lib proofs incl. per-lane byte-identical regression + cross-lane heterogeneous dispatch test; lib 244 (was 240 + 4). No CLI flag (`.2c`); no ROADMAP advance. |
| `PHASE-9-MULTI-ARTIFACT-UMBRELLA.2c` | `Phase 9: PHASE-9-MULTI-ARTIFACT-UMBRELLA.2c --artifact CLI flag + book/CI byte-identical — closes Phase 9 + tree` | `--artifact <lane>` top-level CLI flag (default `dut`) on the `anvil` binary; book/CI byte-identical contract verified by `tests/book_examples::every_runnable_book_bash_block_succeeds` passing; ROADMAP Phase 9 → done; book/README/CODEBASE phase-9 row reconciled. **Closes the tree.** **All 9 numbered roadmap phases now delivered.** |
| `2026-05-20` | `PHASE-9-MULTI-ARTIFACT-UMBRELLA.2a` | New separate top-level module `src/umbrella/mod.rs` registered via `pub mod umbrella` in `src/lib.rs`. `pub trait ArtifactLane` (`name`/`validate_knobs` default-Ok / `generate(seed)` byte-stable / `check_plan`); `pub struct LaneArtifact{lane, seed, sv, manifest: Option<String>}` (typed-Option manifest expresses "L1 has no semantic manifest" cleanly); `pub enum CheckPlan{SynthAccept, ParityVsManifest}`; `pub enum LaneError{UnknownKnob, Construction}` (placeholder for `.2b`/`.2c`); `pub struct DutLane{base_config: Config}` (not `Eq` because `Config` has `f64` knobs) + `impl ArtifactLane for DutLane` where `generate(seed)` clones `base_config`, overrides `.seed`, calls `Generator::new(cfg).generate_design()`, then `to_sv_design(&design)` — **identical to today's invocation pattern, zero behavioural change**. 4 unit proofs (all green): `dut_lane_identity_and_check_plan` (smoke); **`dut_lane_is_byte_identical_to_direct_generator_path`** — the LOAD-BEARING byte-identical regression proof comparing trait-dispatch vs direct-`Generator` output across `{0,1,7,42,12345}` (the proof that `--artifact dut` preserves every existing book example + CI gate — `BOOK-EXAMPLES-RUNNABLE` depends on it); `dut_lane_is_byte_identical_through_dyn_artifact_lane` (`dyn`-dispatch produces the same artifact as concrete-type dispatch — important because `.2c`'s CLI hands around `Box<dyn ArtifactLane>`); `dut_lane_is_reproducible_on_repeated_calls` (no state accumulation across calls). Fixed 2 clippy `doc-lazy-continuation` hits (a `+` at line-start was being parsed as a markdown list marker — reworded the docstring) + 1 `field-reassign-with-default` hit (used struct-update `Config{seed, ..Config::default()}`). `cargo fmt --all --check`/`clippy --all-targets -- -D warnings`/`check --all-targets` clean. Full `cargo test` green: lib **240 passed** (was 236 + 4 new `umbrella::tests`); `tests/microdesign_parity` 15+1 unchanged; `tests/frontend_parity` 12+2 unchanged; `tests/pipeline` 121 passed (656s); `tests/snapshots` 6 passed; bin tests 5+29+3 passed; doc-tests 0 (unchanged). DUT lane stays byte-identical by construction (the umbrella module wraps without modifying). No new lane impls (`.2b`); no CLI flag (`.2c`); no ROADMAP advance. | Done. Frontier → `.2b`. |
| `2026-05-20` | `PHASE-9-MULTI-ARTIFACT-UMBRELLA.2b` | `src/umbrella/mod.rs` extended with the L2 + L3 lane impls (parallel in shape to `.2a`'s L1 `DutLane` wrap). `pub struct MicrodesignLane{n_params: usize}` (Debug+Clone+Copy+PartialEq+Eq — full standard derives since the knob is a `usize`) + `impl ArtifactLane for MicrodesignLane`: `name="microdesign"`, `generate(seed)` calls `crate::microdesign::build_constexpr_unit(seed, n_params)` then `emit_sv(&unit, seed)` then `emit_manifest(&unit, seed)`, returns `LaneArtifact{lane="microdesign", manifest=Some(...)}`, `check_plan=ParityVsManifest`. `pub struct FrontendLane{n_params, n_children}` + `impl ArtifactLane for FrontendLane`: `name="frontend"`, `generate(seed)` calls `crate::frontend::build_acceptable_unit(seed, n_params, n_children)` then `emit_sv(&unit)` then `emit_manifest(&unit)`, returns `LaneArtifact{lane="frontend", manifest=Some(...)}`, `check_plan=ParityVsManifest`. 4 new unit proofs (8 total in `umbrella::tests`): `microdesign_and_frontend_lane_identities` (smoke); **`microdesign_lane_is_byte_identical_to_direct_path`** — the L2 LOAD-BEARING byte-identical regression proof comparing trait-dispatch vs direct-call SV + manifest across seeds `{0,1,7,42,12345}` (BOTH SV AND manifest byte-identical); **`frontend_lane_is_byte_identical_to_direct_path`** — the L3 LOAD-BEARING regression proof; `cross_lane_dispatch_through_dyn_artifact_lane` — heterogeneous `Vec<Box<dyn ArtifactLane>>` dispatches correctly (each lane's name/lane-field/check-plan/manifest-presence matches expected — important for `.2c`'s CLI). Fixed 5 clippy `doc-lazy-continuation` hits (a `--` token at line-start was being parsed as a markdown list marker; reworded the L2 docstring). `cargo fmt --all --check`/`clippy --all-targets -- -D warnings`/`check --all-targets` clean. Full `cargo test` green: lib **244 passed** (was 240 + 4 new); `tests/microdesign_parity` 15+1 unchanged; `tests/frontend_parity` 12+2 unchanged; `tests/pipeline` 121 passed; `tests/snapshots` 6 passed; bin 5+29+3 passed; doc-tests 0. Existing direct-call paths still produce identical output (proven by the per-lane regression tests). No CLI flag (`.2c`); no ROADMAP advance; no `book/` change. | Done. Frontier → `.2c`. |
| `2026-05-20` | `PHASE-9-MULTI-ARTIFACT-UMBRELLA.2c` | `src/main.rs` extended with the `--artifact <lane>` top-level CLI flag. New `ArtifactKind` enum (`Dut` default / `Microdesign` / `Frontend`); new `Cli` fields `--artifact` + `--lane-n-params` (default 5) + `--lane-n-children` (default 2). At the top of `fn main`, an early-return dispatch: `if cli.artifact != ArtifactKind::Dut { return run_non_dut_lane(&cli); }` — so `--artifact dut` (the default) falls through to the historical DUT path UNCHANGED. New helper `run_non_dut_lane` constructs `Box<dyn ArtifactLane>` per the flag, calls `lane.generate(seed)`, emits SV to stdout (or `<out>/<top>.sv`) + manifest to stderr (or `<out>/<top>.json`) — manifest-to-stderr keeps stdout pipelines clean. Helper `parse_top_name(sv)` does best-effort top-name extraction for filenames. Smoke-tested both non-DUT lanes locally (microdesign + frontend produce expected SV preambles). Fixed 1 clippy `unused-imports` hit. `cargo fmt --all --check`/`clippy --all-targets -- -D warnings`/`check --all-targets` clean. **Book/CI byte-identical verification (LOAD-BEARING)**: `cargo test --test book_examples` clean — `test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 80.35s` — the 3rd test (`every_runnable_book_bash_block_succeeds`) runs every runnable bash block in the book + asserts exit 0; it would catch ANY behavioural change to the default invocations. **Phase 9's load-bearing default-`dut`-byte-identical contract is verified.** Full `cargo test` green: lib **244** unchanged; `tests/microdesign_parity` 15+1; `tests/frontend_parity` 12+2; `tests/pipeline` 121 (657s); `tests/snapshots` 6; bin tests 5+29+3; `tests/book_examples` 3 passed (80s); doc-tests 0. Promotion strictly followed the artifact: ROADMAP Phase 9 `(not started)`→`(done)` with 3-lane-selector closure note; `book/src/ir.md` Phase-9 entry; README + CODEBASE phase-coverage-map Phase-9 row done. **Closes the `.2c` + `.2` containers + the `PHASE-9-MULTI-ARTIFACT-UMBRELLA` tree + ROADMAP Phase 9.** **All 9 numbered roadmap phases now delivered.** | Done — Phase 9 closed; tree closed. |

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

- `2026-05-20`: **`.2a` landed — `ArtifactLane` trait + L1
  `DutLane` wrap + byte-identical regression proof.** New
  separate top-level module `src/umbrella/mod.rs` registered
  via `pub mod umbrella` (sits after `microdesign` in
  alphabetical-sort order). `pub trait ArtifactLane` carries
  four methods: `name() -> &'static str`,
  `validate_knobs() -> Result<(), LaneError>` (default `Ok`;
  lane-scoped knob namespace enforced in `.2b`'s L2/L3
  impls), `generate(seed: u64) -> Result<LaneArtifact,
  LaneError>` (byte-stable across rebuilds — the
  load-bearing reproducibility contract), `check_plan()
  -> CheckPlan`. `pub struct LaneArtifact{lane, seed, sv,
  manifest: Option<String>}` is what every lane's
  `generate` returns — the typed-Option `manifest`
  expresses "L1 has no semantic manifest" cleanly without
  hack. `pub enum CheckPlan{SynthAccept,
  ParityVsManifest}`. `pub enum
  LaneError{UnknownKnob{lane, knobs}, Construction{lane,
  message}}` — placeholder for `.2b`/`.2c`'s richer
  validation; `DutLane` never returns `Err` today.
  `pub struct DutLane{base_config: Config}` (NOT `Eq`
  because `Config` carries `f64` knobs whose equality
  isn't meaningful; lane-equality checks compare
  `LaneArtifact` values which ARE `Eq`) +
  `impl ArtifactLane for DutLane` where `generate(seed)`
  clones `base_config`, overrides `.seed`, calls
  `Generator::new(cfg).generate_design()`, then
  `emit::to_sv_design(&design)` — **identical to today's
  invocation pattern, zero behavioural change**. 4 unit
  proofs (all green): `dut_lane_identity_and_check_plan`
  (smoke: `name="dut"`, `check_plan=SynthAccept`); **the
  LOAD-BEARING `dut_lane_is_byte_identical_to_direct_generator_path`**
  comparing trait-dispatch vs direct-`Generator` output
  across reproducibility seeds `{0, 1, 7, 42, 12345}` (if
  this proof breaks, every book example + every CI gate
  that depends on `--artifact dut` would regress —
  `BOOK-EXAMPLES-RUNNABLE` depends on it);
  `dut_lane_is_byte_identical_through_dyn_artifact_lane`
  (`dyn` dispatch via `&dyn ArtifactLane` and
  `Box<dyn ArtifactLane>` produces the same byte-identical
  output as concrete-type dispatch — important because
  `.2c`'s CLI will hand around `Box<dyn ArtifactLane>`);
  `dut_lane_is_reproducible_on_repeated_calls` (no state
  accumulation across calls). Fixed 2 clippy
  `doc-lazy-continuation` hits (a `+` at line-start was
  being parsed as a markdown list marker; reworded the
  docstring) + 1 `field-reassign-with-default` hit (used
  struct-update `Config{seed, ..Config::default()}`).
  `cargo fmt --all --check` / `cargo clippy --all-targets
  -- -D warnings` / `cargo check --all-targets` clean.
  Full `cargo test` green: lib **240 passed** (was 236 + 4
  new `umbrella::tests`); `tests/microdesign_parity` 15+1
  unchanged; `tests/frontend_parity` 12+2 unchanged;
  `tests/pipeline` 121 passed (656s); `tests/snapshots`
  6 passed; bin tests 5+29+3 passed; doc-tests 0
  (unchanged). DUT lane stays byte-identical by
  construction (the umbrella module wraps without
  modifying any existing call site). No new lane impls
  (that is `.2b`); no CLI flag (that is `.2c`); no
  ROADMAP advance. Frontier → `.2b` (`MicrodesignLane` +
  `FrontendLane` impls of the trait + cross-lane
  byte-identical proof).

- `2026-05-20`: **`.2b` landed — L2 `MicrodesignLane` + L3
  `FrontendLane` lane impls.** `src/umbrella/mod.rs` extended
  with the L2 and L3 lane impls (parallel in shape to `.2a`'s
  L1 `DutLane` wrap). `pub struct MicrodesignLane{n_params:
  usize}` (full standard derives since the knob is a `usize`,
  unlike `DutLane` which holds `Config` and can't be `Eq`) +
  `impl ArtifactLane for MicrodesignLane` calling Phase 7's
  `microdesign::{build_constexpr_unit, emit_sv,
  emit_manifest}` under the hood, returning
  `LaneArtifact{lane="microdesign", manifest=Some(...)}` with
  `check_plan=ParityVsManifest`. `pub struct FrontendLane{
  n_params, n_children}` + `impl ArtifactLane for
  FrontendLane` calling Phase 8's `frontend::{
  build_acceptable_unit, emit_sv, emit_manifest}` similarly.
  4 new unit proofs (8 total in `umbrella::tests`):
  `microdesign_and_frontend_lane_identities` (smoke);
  **`microdesign_lane_is_byte_identical_to_direct_path`** —
  the L2 LOAD-BEARING byte-identical regression proof
  comparing trait-dispatch vs direct-call SV AND manifest
  across the reproducibility-set seeds (matching Phase 7's
  `.2a` test);
  **`frontend_lane_is_byte_identical_to_direct_path`** — the
  L3 LOAD-BEARING regression proof (matching Phase 8's `.2a`
  test); `cross_lane_dispatch_through_dyn_artifact_lane` —
  a heterogeneous `Vec<Box<dyn ArtifactLane>>` containing
  all 3 lanes dispatches correctly (each lane's name /
  lane-field / check-plan / manifest-presence matches the
  expected lane identity — important for `.2c`'s CLI which
  will hold a `Box<dyn ArtifactLane>` chosen by
  `--artifact <name>`). Fixed 5 clippy
  `doc-lazy-continuation` hits (a `--` token at line-start
  was being parsed as a markdown list marker; reworded the
  L2 docstring). `cargo fmt --all --check`/`clippy
  --all-targets -- -D warnings`/`check --all-targets`
  clean. Full `cargo test` green: lib **244 passed** (was
  240 + 4 new `umbrella::tests`); `tests/microdesign_parity`
  15+1 unchanged; `tests/frontend_parity` 12+2 unchanged;
  `tests/pipeline` 121 passed; `tests/snapshots` 6 passed;
  bin tests 5+29+3 passed; doc-tests 0 (unchanged).
  Existing direct-call paths still produce identical
  output. No CLI flag (`.2c`); no ROADMAP advance; no
  `book/` change. Frontier → `.2c` (`--artifact <lane>`
  top-level CLI flag with default `dut` + book/CI
  byte-identical verification + record **ROADMAP Phase 9 →
  done**; may further split per Phase 7/8 `.2c` →
  `.2c.1`/`.2c.2` precedent on a discovered regression).

- `2026-05-20`: **`.2c` landed — `--artifact <lane>` CLI flag
  + book/CI byte-identical verification + ROADMAP Phase 9 →
  done; `PHASE-9-MULTI-ARTIFACT-UMBRELLA` tree CLOSED.**
  `src/main.rs` extended with the `--artifact <lane>`
  top-level CLI flag (`ArtifactKind::Dut|Microdesign|Frontend`,
  default `Dut`) + `--lane-n-params` + `--lane-n-children`
  knobs. At the top of `fn main`, an early-return dispatch:
  `if cli.artifact != ArtifactKind::Dut { return
  run_non_dut_lane(&cli); }` — so `--artifact dut` (the
  default) falls through to the historical DUT path
  UNCHANGED. New helper `run_non_dut_lane` constructs
  `Box<dyn ArtifactLane>` per the flag (`MicrodesignLane` or
  `FrontendLane`), calls `lane.generate(seed)`, emits SV to
  stdout (or `<out>/<top>.sv`) + manifest to stderr (or
  `<out>/<top>.json`). **Book/CI byte-identical verification
  (LOAD-BEARING)**: `cargo test --test book_examples` clean
  — `test result: ok. 3 passed; 0 failed; ...; finished in
  80.35s` (the 3rd test runs every runnable bash block in
  the book + asserts exit 0). **Phase 9's load-bearing
  default-`dut`-byte-identical contract is verified.** Full
  `cargo test` green: lib 244 unchanged;
  `tests/microdesign_parity` 15+1; `tests/frontend_parity`
  12+2; `tests/pipeline` 121 (657s); `tests/snapshots` 6;
  bin tests 5+29+3; `tests/book_examples` 3 passed (80s);
  doc-tests 0. Promotion strictly followed the verified
  evidence: ROADMAP Phase 9 `(not started)`→`(done)` with
  the 3-lane selector + default-`dut`-byte-identical
  closure note; `book/src/ir.md` Phase-9 entry; README +
  CODEBASE phase-coverage-map Phase-9 row done. **Closes
  `.2c` + `.2` containers + `PHASE-9-MULTI-ARTIFACT-UMBRELLA`
  tree + ROADMAP Phase 9. ALL 9 NUMBERED ROADMAP PHASES NOW
  DELIVERED.** Open follow-up trees (multi-clock CDC —
  explicitly-optional separately-prioritised deferral;
  DIFFERENTIAL-SIMULATION.2b harness implementation) carry
  on as post-phase work.
