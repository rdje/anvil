//! PHASE-9-MULTI-ARTIFACT-UMBRELLA.2a — the `ArtifactLane` trait +
//! shared umbrella-owned plumbing + the L1 DUT lane wrap.
//!
//! Phase 9's job is to **unify the plumbing** across the three
//! delivered artifact lanes — the DUT RTL lane (Phases 1–6), the
//! oracle-backed micro-design lane (`src/microdesign/`, Phase 7), and
//! the source-level frontend / elaboration accept lane
//! (`src/frontend/`, Phase 8) — **without** blurring them into a
//! single "random SV generator". The explicit anti-goal recorded in
//! `PHASE-9-MULTI-ARTIFACT-UMBRELLA.1`'s design entry: never collapse
//! the three lanes' rules-first generators into one parametric
//! mode-flagged producer; only their plumbing (seed → reproducible
//! artifact, byte-stable output, optional manifest, downstream check
//! plan) unifies here.
//!
//! Contents:
//! - `.2a` (this slice) — the `ArtifactLane` trait, the `LaneArtifact`
//!   carrier, the `CheckPlan` enum, the `LaneError` placeholder, and
//!   the **L1 `DutLane`** impl wrapping today's `gen::Generator` +
//!   `emit::to_sv_design` path. The DUT lane wrap is *zero
//!   behavioural change* for the default `--artifact dut` case — the
//!   byte-identical regression test in `tests/lane_byte_identical.rs`
//!   pins it.
//! - `.2b` — L2 `MicrodesignLane` + L3 `FrontendLane` impls of the
//!   trait + cross-lane byte-identical proof.
//! - `.2c` — `--artifact <lane>` top-level CLI flag (default `dut`),
//!   book/CI byte-identical verification, **ROADMAP Phase 9 → done**.
//!
//! The load-bearing constraint throughout Phase 9 `.2`: the default
//! `--artifact dut` path stays byte-identical to today.
//! `BOOK-EXAMPLES-RUNNABLE` + every CI gate depend on this; Phase 9
//! `.2a` lands the regression test that enforces it from now on.

use crate::config::Config;
use crate::emit::to_sv_design;
use crate::gen::Generator;

/// What downstream-check shape a lane expects.
///
/// Per `.1`'s design: `SynthAccept` for the DUT RTL lane (lint + synth
/// acceptance; today's `tool_matrix` gate); `ParityVsManifest` for the
/// oracle-backed lanes (Phase 7 + Phase 8 — the parity gates already
/// land via the `microdesign_parity` + `frontend_parity` test files).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckPlan {
    /// Lint + synth acceptance against tools like Verilator / Yosys.
    /// L1 (DUT RTL) is the only `SynthAccept` lane today.
    SynthAccept,
    /// Parity against the lane's expected-facts manifest. L2
    /// (microdesign) and L3 (frontend) use this.
    ParityVsManifest,
}

/// What a lane produces: a name + an SV string + an optional
/// expected-facts manifest. `manifest` is `None` for lanes that don't
/// ship a semantic oracle (the DUT RTL lane — its check plan is
/// `SynthAccept` against real tools, not parity against a manifest).
/// `Some` for the Phase 7/8 lanes — typed-optional, not a hack.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LaneArtifact {
    /// The lane that produced this artifact (`"dut"` / `"microdesign"`
    /// / `"frontend"`; matches `ArtifactLane::name`).
    pub lane: String,
    /// The seed this artifact was built from.
    pub seed: u64,
    /// The emitted SystemVerilog.
    pub sv: String,
    /// The expected-facts JSON manifest, if the lane carries one.
    pub manifest: Option<String>,
}

/// A lane error. Placeholder enum — `.2a`'s `DutLane` doesn't
/// currently fail (the rules-first generators are valid by
/// construction), but the slot exists so `.2b`/`.2c`'s richer lane
/// impls + the eventual CLI dispatch can surface lane-scoped
/// validation failures (e.g. cross-lane knob bleed rejected by
/// `validate_knobs`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LaneError {
    /// A knob was set on a lane that doesn't accept it (cross-lane
    /// bleed). Carries the lane name + offending knob name(s).
    UnknownKnob { lane: String, knobs: Vec<String> },
    /// Lane-specific construction failure. Carries a free-form
    /// message; `.2b`/`.2c` may narrow into structured variants if
    /// new failure modes surface.
    Construction { lane: String, message: String },
}

/// The umbrella's lane abstraction. Each delivered artifact family
/// implements this; the `.2c` CLI dispatches over `dyn ArtifactLane`.
///
/// **By design** (`PHASE-9-MULTI-ARTIFACT-UMBRELLA.1`): the trait
/// only unifies the plumbing — `(seed, lane_knobs) → byte-identical
/// (artifact + optional manifest)` — never the *generators*. The
/// three lanes' rules-first generators remain decoupled in their
/// own modules (`src/gen/`, `src/microdesign/`, `src/frontend/`).
pub trait ArtifactLane {
    /// The lane name (`"dut"` / `"microdesign"` / `"frontend"`).
    /// Stable across versions — it appears in the manifest, in the
    /// on-disk layout, and on the `--artifact` CLI flag.
    fn name(&self) -> &'static str;

    /// Validate the lane-scoped knob bag. Rejects cross-lane bleed by
    /// returning the unknown-knob list. `.2a`'s `DutLane` accepts any
    /// `Config` (validation of DUT knobs lives in `Config` itself);
    /// `.2b`'s `MicrodesignLane` + `FrontendLane` impls will enforce
    /// their narrower scoped namespaces.
    fn validate_knobs(&self) -> Result<(), LaneError> {
        Ok(())
    }

    /// Build the artifact for `seed`. **Byte-stable** across rebuilds
    /// for fixed `(seed, lane_knobs)` (the load-bearing reproducibility
    /// contract, identical-in-shape to today's `(seed, knobs)` DUT
    /// contract with `lane` prepended).
    fn generate(&self, seed: u64) -> Result<LaneArtifact, LaneError>;

    /// Which downstream-check shape this lane expects. The
    /// `tool_matrix` gate dispatches to a different harness per
    /// lane's `CheckPlan` (`SynthAccept` for L1 vs `ParityVsManifest`
    /// for L2/L3).
    fn check_plan(&self) -> CheckPlan;
}

// ===================================================================
// L1 — the DUT RTL lane. Wraps today's `gen::Generator` path so the
// default `--artifact dut` invocation (and every book example +
// every CI gate that depends on it) stays byte-identical.
// ===================================================================

/// The DUT RTL lane (Phases 1–6). `DutLane::generate(seed)` IS today's
/// `Generator::new(cfg.with_seed(seed)).generate_design()` followed by
/// `to_sv_design(&design)` — zero behavioural change. The
/// load-bearing byte-identical regression proof lives in
/// `tests/lane_byte_identical.rs`.
///
/// `Config` doesn't impl `Eq` (it carries `f64` knobs like
/// `flop_prob`/`memory_prob`/`fsm_prob`/`aggregate_prob`/
/// `width_parameterization_prob` for which equality isn't a
/// meaningful operation on floats), so `DutLane` doesn't either.
/// Lane-equality checks in the test suite compare `LaneArtifact`s
/// (which are `Eq`) rather than `DutLane` values directly.
#[derive(Debug, Clone)]
pub struct DutLane {
    /// Base configuration. The lane's `generate(seed)` overrides
    /// `base_config.seed` per call so a single `DutLane` can serve
    /// many seeds without rebuilding (matches the existing
    /// `Generator::new(cfg)` per-seed construction pattern, just
    /// hoisted to the lane level).
    pub base_config: Config,
}

impl DutLane {
    /// Construct a `DutLane` from a base config. The lane's
    /// `generate(seed)` will override `base_config.seed` per call.
    pub fn new(base_config: Config) -> Self {
        Self { base_config }
    }
}

impl ArtifactLane for DutLane {
    fn name(&self) -> &'static str {
        "dut"
    }

    fn generate(&self, seed: u64) -> Result<LaneArtifact, LaneError> {
        // Zero behavioural change vs the direct call: clone the base
        // config, override the seed, run the existing generator, emit
        // SV. Any deviation here would break every book example.
        let mut cfg = self.base_config.clone();
        cfg.seed = seed;
        let mut gen = Generator::new(cfg);
        let design = gen.generate_design();
        let sv = to_sv_design(&design);
        Ok(LaneArtifact {
            lane: "dut".to_string(),
            seed,
            sv,
            manifest: None, // L1 has no semantic manifest (typed Option).
        })
    }

    fn check_plan(&self) -> CheckPlan {
        CheckPlan::SynthAccept
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::emit::to_sv_design;
    use crate::gen::Generator;

    /// `DutLane::name()` returns `"dut"` and `check_plan()` is
    /// `SynthAccept`. Smoke-shape; pins the lane identity.
    #[test]
    fn dut_lane_identity_and_check_plan() {
        let lane = DutLane::new(Config::default());
        assert_eq!(lane.name(), "dut");
        assert_eq!(lane.check_plan(), CheckPlan::SynthAccept);
    }

    /// **Load-bearing byte-identical regression proof.**
    ///
    /// `DutLane::generate(seed)` produces *byte-identical* SV to the
    /// direct call (`Generator::new(cfg)` with `cfg.seed = seed`,
    /// then `generate_design`, then `to_sv_design`), across the
    /// reproducibility-set seeds. If this proof breaks, every book
    /// example and every CI gate that depends on the default
    /// `--artifact dut` behaviour would regress. Mirrors the
    /// reproducibility proofs Phase 7's `.2a` and Phase 8's `.2a`
    /// introduced for their own lanes.
    #[test]
    fn dut_lane_is_byte_identical_to_direct_generator_path() {
        for &seed in &[0u64, 1, 7, 42, 12345] {
            // Direct legacy path.
            let direct_cfg = Config {
                seed,
                ..Config::default()
            };
            let mut direct_gen = Generator::new(direct_cfg);
            let direct_design = direct_gen.generate_design();
            let direct_sv = to_sv_design(&direct_design);

            // Trait-dispatched lane path.
            let lane = DutLane::new(Config::default());
            let artifact = lane
                .generate(seed)
                .expect("DutLane::generate must succeed on Config::default()");
            assert_eq!(artifact.lane, "dut");
            assert_eq!(artifact.seed, seed);
            assert_eq!(artifact.manifest, None);
            assert_eq!(
                artifact.sv, direct_sv,
                "DutLane::generate must be byte-identical to the direct \
                 Generator path (seed={seed})"
            );
        }
    }

    /// Trait-dispatched call via a `&dyn ArtifactLane` reference
    /// produces the same artifact as a direct concrete-type call —
    /// the proof that dynamic dispatch through the umbrella doesn't
    /// perturb the byte-stable contract. Important because `.2c`'s
    /// CLI dispatch will hand around `Box<dyn ArtifactLane>` values.
    #[test]
    fn dut_lane_is_byte_identical_through_dyn_artifact_lane() {
        let direct = DutLane::new(Config::default());
        let boxed: Box<dyn ArtifactLane> = Box::new(DutLane::new(Config::default()));
        for &seed in &[0u64, 7, 42] {
            let a = direct.generate(seed).unwrap();
            let b = boxed.generate(seed).unwrap();
            assert_eq!(a, b, "dyn dispatch must be byte-identical (seed={seed})");
        }
    }

    /// Reproducibility on a fixed seed: two successive
    /// `DutLane::generate(seed)` calls on the same `DutLane` produce
    /// the identical artifact. The lane shouldn't accumulate state
    /// across calls (the underlying `Generator` is reseeded from
    /// `cfg.seed` per call inside `generate`).
    #[test]
    fn dut_lane_is_reproducible_on_repeated_calls() {
        let lane = DutLane::new(Config::default());
        for &seed in &[1u64, 7, 42] {
            let a = lane.generate(seed).unwrap();
            let b = lane.generate(seed).unwrap();
            assert_eq!(
                a, b,
                "repeated DutLane::generate({seed}) must be byte-identical"
            );
        }
    }
}
