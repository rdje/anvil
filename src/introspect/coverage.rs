//! Achieved-coverage readout surface (`COVERAGE-STEERED-GENERATION.2b`,
//! decision [`0023`](../../docs/decisions/0023-coverage-steered-generation.md)).
//!
//! The **read** half of construction-time coverage steering: *what did this
//! run actually exercise?* It is the data an outer
//! measure→derive→re-steer loop (decision `0023` §4) consumes to compute the
//! next [`SteeringConfig`](crate::config::SteeringConfig) — the achieved
//! coverage the steering **prior** (the `.2a` `roll_knob` multiplier) is meant
//! to bend.
//!
//! # Invariant SCHEMA-DERIVED (inherited from `0004`/`0011`)
//!
//! A [`CoverageReadout`] computes **zero new generator truth**. It is a pure
//! function of the [`Metrics`] the generator already records — the per-knob
//! roll counters (`knob_roll_attempts` / `knob_roll_fires`) and the
//! gate-kind / operand-arity / combinational-depth histograms. The single
//! genuinely-*derived* quantity is the per-knob empirical **fire rate**
//! (`fires / attempts`) — the division the agent would otherwise compute
//! itself — plus the per-**category** roll-up that mirrors the
//! [`SteeringConfig`](crate::config::SteeringConfig) target model (per-knob +
//! per-category). No IR field, no generator change: this is the
//! `coverage_gaps` / `analyze` project-don't-recompute precedent applied to
//! the roll telemetry.
//!
//! # Why the histograms ride along
//!
//! Decision `0023` §3 defines the readout as the fire rates **plus** the
//! gate-kind / operand-arity / depth histograms, so a single `coverage` query
//! is self-contained (decision `0017`: design the API for agents, not for
//! minimal duplication). For a single `module` the histograms also appear in
//! `module_metrics`; for a `design` the readout's histograms are a genuine
//! **aggregate** across the per-child metrics (no single-place equivalent in
//! `design_metrics`). The matrix-only `saw_*` coverage facts are *not* here: a
//! lone artifact cannot prove them (the existing `coverage` section, schema
//! §6.4, is matrix-only for the same reason).
//!
//! # Determinism
//!
//! Every map is a `BTreeMap`, and every `fire_rate` is a round-half-up integer
//! parts-per-million quotient (`KnobCoverage::new`), so a [`CoverageReadout`] is
//! a byte-stable function of its input [`Metrics`] — no raw `f64` division
//! reaches the document, which a bare `fires as f64 / attempts as f64` could let
//! diverge by 1 ULP between evaluation contexts.

use crate::config::SteeringConfig;
use crate::ir::KnobId;
use crate::metrics::Metrics;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// One coverage cell: how many times a knob (or a category roll-up of knobs)
/// was *rolled* (`attempts`), how many of those rolls *fired* (`fires`), and
/// the empirical `fire_rate ≈ fires / attempts`. `attempts` / `fires` are the
/// **exact** source of truth (a consumer wanting full precision divides them
/// itself); `fire_rate` is the convenience projection, rounded to **parts per
/// million** (6 decimal places — far finer than any steering decision needs).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct KnobCoverage {
    pub attempts: u64,
    pub fires: u64,
    pub fire_rate: f64,
}

impl KnobCoverage {
    fn new(attempts: u64, fires: u64) -> Self {
        // Determinism: compute the rate as a round-half-up integer
        // parts-per-million quotient, then one exact `u64 → f64 / 1e6`. This
        // avoids the cross-evaluation 1-ULP divergence a raw
        // `fires as f64 / attempts as f64` can show between two call sites
        // (the compiler may fold one site and run the other) — the
        // introspection document must be byte-identical for the same inputs.
        // `checked_div` is `None` exactly when `attempts == 0` (a knob that was
        // never rolled), which maps to a `0.0` rate.
        let fire_rate = (fires.saturating_mul(1_000_000) + attempts / 2)
            .checked_div(attempts)
            .map_or(0.0, |ppm| ppm as f64 / 1_000_000.0);
        KnobCoverage {
            attempts,
            fires,
            fire_rate,
        }
    }
}

/// The achieved-coverage readout for one artifact: a SCHEMA-DERIVED projection
/// of the run's roll telemetry + construct histograms (see the module docs).
/// The shape mirrors the [`SteeringConfig`](crate::config::SteeringConfig)
/// target model — `knob_fire_rates` (per-`KnobId::name`) and
/// `category_fire_rates` (per-`KnobId::category` roll-up) — so an agent reading
/// the readout can derive either a per-knob or a per-category steering weight
/// directly.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CoverageReadout {
    /// Per-knob empirical fire rate, keyed by `KnobId::name()`. Only knobs
    /// that were rolled at least once appear (the `Metrics` roll maps omit
    /// zero-attempt knobs to stay compact); a knob the run never reached is
    /// simply absent.
    pub knob_fire_rates: BTreeMap<String, KnobCoverage>,
    /// Per-category roll-up: the same cell summed over every knob in a coarse
    /// [`KnobId::category`] (`state` / `selectors` / `datapath` / `terminals` /
    /// `sharing` / `hierarchy`). The category's `fire_rate` is the pooled
    /// `sum(fires) / sum(attempts)`, not an average of per-knob rates, so it is
    /// attempt-weighted. Absent categories had no rolls.
    pub category_fire_rates: BTreeMap<String, KnobCoverage>,
    /// Count of `Node::Gate` per `GateOp` kind — the achieved construct mix.
    /// Echoes `Metrics::gates_by_kind` for a module; the design-level readout
    /// aggregates it across children.
    pub gate_kind_histogram: BTreeMap<String, usize>,
    /// Histogram of operator-gate operand counts (arity). Echoes
    /// `Metrics::gate_operand_count_histogram`; aggregated across children for a
    /// design.
    pub gate_operand_count_histogram: BTreeMap<usize, usize>,
    /// Histogram of per-gate combinational depth. Echoes
    /// `Metrics::gate_depth_histogram`; aggregated across children for a design.
    pub gate_depth_histogram: BTreeMap<usize, usize>,
}

/// Sum `delta` into `acc[key]` (the `BTreeMap` aggregation helper used to fold
/// per-child histograms into one design-level histogram).
fn add_into<K: Ord + Clone>(acc: &mut BTreeMap<K, usize>, src: &BTreeMap<K, usize>) {
    for (key, count) in src {
        *acc.entry(key.clone()).or_insert(0) += *count;
    }
}

/// Build the [`CoverageReadout`] for a single module's [`Metrics`]. Pure:
/// byte-identical for the same `m`.
pub fn module_coverage(m: &Metrics) -> CoverageReadout {
    readout_from_parts(
        &m.knob_roll_attempts,
        &m.knob_roll_fires,
        m.gates_by_kind.clone(),
        m.gate_operand_count_histogram.clone(),
        m.gate_depth_histogram.clone(),
    )
}

/// Build the design-level [`CoverageReadout`] by aggregating the per-child
/// [`Metrics`]. The roll counters and histograms sum across every module in the
/// design, so the readout reports the whole run's achieved coverage — exactly
/// what the outer steering loop measures. Pure: byte-identical for the same
/// `modules` (iteration is over the caller-supplied slice order, then every map
/// is `BTreeMap`-sorted).
pub fn design_coverage(modules: &[Metrics]) -> CoverageReadout {
    let mut attempts: BTreeMap<String, u64> = BTreeMap::new();
    let mut fires: BTreeMap<String, u64> = BTreeMap::new();
    let mut gates_by_kind: BTreeMap<String, usize> = BTreeMap::new();
    let mut operand_hist: BTreeMap<usize, usize> = BTreeMap::new();
    let mut depth_hist: BTreeMap<usize, usize> = BTreeMap::new();
    for m in modules {
        for (knob, count) in &m.knob_roll_attempts {
            *attempts.entry(knob.clone()).or_insert(0) += *count;
        }
        for (knob, count) in &m.knob_roll_fires {
            *fires.entry(knob.clone()).or_insert(0) += *count;
        }
        add_into(&mut gates_by_kind, &m.gates_by_kind);
        add_into(&mut operand_hist, &m.gate_operand_count_histogram);
        add_into(&mut depth_hist, &m.gate_depth_histogram);
    }
    readout_from_parts(&attempts, &fires, gates_by_kind, operand_hist, depth_hist)
}

/// Assemble a [`CoverageReadout`] from already-aggregated roll counters +
/// histograms. The per-knob cells iterate the `attempts` map (every present key
/// has `attempts >= 1`); the per-category roll-up pools each knob into its
/// `KnobId::category` via [`KnobId::category_of_name`] (the single name→category
/// inversion — no second table).
fn readout_from_parts(
    attempts: &BTreeMap<String, u64>,
    fires: &BTreeMap<String, u64>,
    gate_kind_histogram: BTreeMap<String, usize>,
    gate_operand_count_histogram: BTreeMap<usize, usize>,
    gate_depth_histogram: BTreeMap<usize, usize>,
) -> CoverageReadout {
    let mut knob_fire_rates: BTreeMap<String, KnobCoverage> = BTreeMap::new();
    let mut cat_attempts: BTreeMap<String, u64> = BTreeMap::new();
    let mut cat_fires: BTreeMap<String, u64> = BTreeMap::new();
    for (knob, attempt_count) in attempts {
        let fire_count = fires.get(knob).copied().unwrap_or(0);
        knob_fire_rates.insert(knob.clone(), KnobCoverage::new(*attempt_count, fire_count));
        if let Some(category) = KnobId::category_of_name(knob) {
            *cat_attempts.entry(category.to_string()).or_insert(0) += *attempt_count;
            *cat_fires.entry(category.to_string()).or_insert(0) += fire_count;
        }
    }
    let category_fire_rates = cat_attempts
        .iter()
        .map(|(category, attempt_count)| {
            let fire_count = cat_fires.get(category).copied().unwrap_or(0);
            (
                category.clone(),
                KnobCoverage::new(*attempt_count, fire_count),
            )
        })
        .collect();
    CoverageReadout {
        knob_fire_rates,
        category_fire_rates,
        gate_kind_histogram,
        gate_operand_count_histogram,
        gate_depth_histogram,
    }
}

/// Parameters for [`derive_steering_from_coverage`] — the **derive** step of the
/// outer measure→derive→re-steer loop (decision `0023` §4 step 2). Tunable so a
/// caller (a sweep, a CI job) can pick how aggressively to rebalance.
#[derive(Debug, Clone, PartialEq)]
pub struct DeriveParams {
    /// The fire rate to steer each category *toward*. A category whose achieved
    /// rate is below this gets up-weighted; one above gets down-weighted.
    pub target_share: f64,
    /// Clamp ceiling on the emitted weight, so a barely-exercised category cannot
    /// produce an unbounded multiplier. (Floor is `0.0`.)
    pub max_weight: f64,
    /// Floor on the observed share in the denominator, so a zero-fire category
    /// yields `target_share / epsilon` (a large but finite up-weight), never a
    /// division by zero.
    pub epsilon: f64,
}

impl Default for DeriveParams {
    fn default() -> Self {
        // A neutral midpoint target, a generous-but-bounded ceiling, and a
        // milli-floor on the denominator.
        DeriveParams {
            target_share: 0.5,
            max_weight: 8.0,
            epsilon: 1e-3,
        }
    }
}

/// Quantize a steering weight to milli-precision via integer rounding, so a
/// derived weight is **byte-stable across evaluation contexts** (the same
/// determinism discipline as `KnobCoverage::new`'s `fire_rate`: a raw
/// `f64`-division weight could differ by 1 ULP between machines, and the weight
/// is an *input* to a future generation run — `(seed, knobs, steering-config)`
/// must stay reproducible). Milli is far finer than any steering decision needs.
fn quantize_weight_milli(w: f64) -> f64 {
    // `w` is already clamped to `[0, max_weight]`, so `w * 1000.0` is a small
    // non-negative finite value; `.round()` collapses any sub-milli divergence.
    (w * 1000.0).round() / 1000.0
}

/// Derive a [`SteeringConfig`] that nudges each category's achieved fire rate
/// toward `params.target_share` — the pure, deterministic **derive** step of the
/// outer measure→derive→re-steer loop (decision `0023` §4). For each category in
/// the readout:
///
/// ```text
/// weight = clamp( target_share / max(observed_share, epsilon), 0.0, max_weight )
/// ```
///
/// so an **under-hit** category (low observed share) gets `weight > 1` (more
/// emphasis) and an over-hit one gets `weight < 1`. Only **non-neutral** weights
/// (more than milli away from `1.0`) are emitted, so a run already at target
/// yields an (almost) empty `SteeringConfig` ⇒ near-byte-identical re-runs. The
/// result is a per-category steering target; an agent can layer per-knob weights
/// on top.
///
/// This does **not** run the generator and is **not** a filter — it is a pure
/// `CoverageReadout → SteeringConfig` function (`feedback_rules_first_generation`:
/// the feedback lives in the orchestration, not the generator). Byte-identical
/// for the same `(readout, params)` (every weight is milli-quantized).
pub fn derive_steering_from_coverage(
    readout: &CoverageReadout,
    params: &DeriveParams,
) -> SteeringConfig {
    let mut per_category: BTreeMap<String, f64> = BTreeMap::new();
    for (category, cell) in &readout.category_fire_rates {
        let observed = cell.fire_rate.max(params.epsilon);
        let weight =
            quantize_weight_milli((params.target_share / observed).clamp(0.0, params.max_weight));
        // Omit a neutral weight to keep the steering-config minimal (and an
        // at-target re-run as close to byte-identical as possible).
        if (weight - 1.0).abs() > 1e-6 {
            per_category.insert(category.clone(), weight);
        }
    }
    SteeringConfig {
        per_knob: BTreeMap::new(),
        per_category,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::Generator;

    fn comb_cfg(seed: u64) -> Config {
        Config {
            seed,
            ..Config::default()
        }
    }

    #[test]
    fn fire_rate_is_fires_over_attempts() {
        assert_eq!(KnobCoverage::new(10, 4).fire_rate, 0.4);
        // Zero attempts is the total-constructor guard (no present key hits it,
        // but the cell stays well-defined).
        assert_eq!(KnobCoverage::new(0, 0).fire_rate, 0.0);
        // A fully-firing knob rounds to exactly 1.0.
        assert_eq!(KnobCoverage::new(7, 7).fire_rate, 1.0);
        // The ppm rounding stays within 1e-6 of the true ratio.
        let cell = KnobCoverage::new(1439, 161);
        assert!((cell.fire_rate - 161.0 / 1439.0).abs() < 1e-6);
    }

    #[test]
    fn fire_rate_is_deterministic_across_inputs() {
        // The same (attempts, fires) always yields a bit-identical rate — the
        // integer-ppm path has no cross-evaluation FP-division divergence.
        for (a, f) in [(1439u64, 161u64), (295, 36), (2134, 637), (61, 46)] {
            assert_eq!(KnobCoverage::new(a, f), KnobCoverage::new(a, f));
        }
    }

    #[test]
    fn module_readout_is_exact_projection_of_metrics() {
        // Every present knob's cell equals fires/attempts from the SAME Metrics
        // maps, and the histograms are byte-equal echoes — the SCHEMA-DERIVED
        // invariant asserted directly.
        let cfg = comb_cfg(7);
        let mut gen = Generator::new(cfg.clone());
        let m = gen.generate_module();
        let metrics = crate::metrics::compute(&m);
        let readout = module_coverage(&metrics);

        assert_eq!(readout.gate_kind_histogram, metrics.gates_by_kind);
        assert_eq!(
            readout.gate_operand_count_histogram,
            metrics.gate_operand_count_histogram
        );
        assert_eq!(readout.gate_depth_histogram, metrics.gate_depth_histogram);

        // Keyset of knob_fire_rates == keyset of knob_roll_attempts.
        let rate_keys: Vec<_> = readout.knob_fire_rates.keys().cloned().collect();
        let attempt_keys: Vec<_> = metrics.knob_roll_attempts.keys().cloned().collect();
        assert_eq!(rate_keys, attempt_keys);

        for (knob, cell) in &readout.knob_fire_rates {
            let attempts = metrics.knob_roll_attempts[knob];
            let fires = metrics.knob_roll_fires.get(knob).copied().unwrap_or(0);
            // The integers are exact; the rate is the ppm-rounded ratio.
            assert_eq!(cell.attempts, attempts);
            assert_eq!(cell.fires, fires);
            assert!((cell.fire_rate - fires as f64 / attempts as f64).abs() < 1e-6);
            assert!(attempts >= 1, "a present knob must have >= 1 attempt");
        }
    }

    #[test]
    fn category_rollup_pools_attempts_and_fires() {
        // Each category cell is the attempt-weighted pool of its member knobs,
        // and every per-knob category sums into exactly one category cell.
        let cfg = comb_cfg(3);
        let mut gen = Generator::new(cfg.clone());
        let m = gen.generate_module();
        let metrics = crate::metrics::compute(&m);
        let readout = module_coverage(&metrics);

        let mut expect_attempts: BTreeMap<String, u64> = BTreeMap::new();
        let mut expect_fires: BTreeMap<String, u64> = BTreeMap::new();
        for (knob, cell) in &readout.knob_fire_rates {
            let category = KnobId::category_of_name(knob)
                .expect("every metrics knob key is a known KnobId name");
            *expect_attempts.entry(category.to_string()).or_insert(0) += cell.attempts;
            *expect_fires.entry(category.to_string()).or_insert(0) += cell.fires;
        }
        for (category, cell) in &readout.category_fire_rates {
            assert_eq!(cell.attempts, expect_attempts[category]);
            assert_eq!(cell.fires, expect_fires[category]);
            assert!(
                (cell.fire_rate - cell.fires as f64 / cell.attempts as f64).abs() < 1e-6,
                "category {category} fire_rate must be the pooled ratio"
            );
        }
        assert_eq!(
            readout.category_fire_rates.len(),
            expect_attempts.len(),
            "every rolled category is present exactly once"
        );
    }

    #[test]
    fn design_readout_aggregates_children() {
        // A design readout sums the per-child roll counters and histograms.
        let cfg = Config {
            seed: 42,
            hierarchy_depth: 1,
            num_leaf_modules: 2,
            num_child_instances: 3,
            ..Config::default()
        };
        let mut gen = Generator::new(cfg.clone());
        let design = gen.generate_design();
        let per_child: Vec<_> = design.modules.iter().map(crate::metrics::compute).collect();
        let readout = design_coverage(&per_child);

        // Gate-kind histogram is the child-wise sum.
        let mut expect_gates: BTreeMap<String, usize> = BTreeMap::new();
        let mut expect_attempts: BTreeMap<String, u64> = BTreeMap::new();
        for m in &per_child {
            for (k, v) in &m.gates_by_kind {
                *expect_gates.entry(k.clone()).or_insert(0) += *v;
            }
            for (k, v) in &m.knob_roll_attempts {
                *expect_attempts.entry(k.clone()).or_insert(0) += *v;
            }
        }
        assert_eq!(readout.gate_kind_histogram, expect_gates);
        for (knob, cell) in &readout.knob_fire_rates {
            assert_eq!(cell.attempts, expect_attempts[knob]);
        }
    }

    #[test]
    fn readout_round_trips_through_json() {
        let cfg = comb_cfg(5);
        let mut gen = Generator::new(cfg.clone());
        let m = gen.generate_module();
        let readout = module_coverage(&crate::metrics::compute(&m));
        let s = serde_json::to_string(&readout).unwrap();
        let back: CoverageReadout = serde_json::from_str(&s).unwrap();
        assert_eq!(readout, back);
    }

    /// Build a readout carrying just the given per-category fire rates (the only
    /// field `derive_steering_from_coverage` reads).
    fn readout_with_category_rates(rates: &[(&str, f64)]) -> CoverageReadout {
        let mut category_fire_rates = BTreeMap::new();
        for (cat, rate) in rates {
            category_fire_rates.insert(
                (*cat).to_string(),
                KnobCoverage {
                    attempts: 1000,
                    fires: (rate * 1000.0).round() as u64,
                    fire_rate: *rate,
                },
            );
        }
        CoverageReadout {
            category_fire_rates,
            ..Default::default()
        }
    }

    #[test]
    fn derive_up_weights_under_hit_and_neutralizes_at_target() {
        // target 0.5: an under-hit category (0.1) is up-weighted (~5x), an
        // at-target one (0.5) is neutral and therefore omitted, and an over-hit
        // one (1.0) is down-weighted (~0.5x).
        let readout =
            readout_with_category_rates(&[("state", 0.1), ("selectors", 0.5), ("datapath", 1.0)]);
        let params = DeriveParams::default();
        let steering = derive_steering_from_coverage(&readout, &params);

        // Under-hit "state" up-weighted toward 0.5/0.1 = 5.0.
        assert!((steering.per_category["state"] - 5.0).abs() < 1e-6);
        // At-target "selectors" is neutral ⇒ omitted (keeps the config minimal).
        assert!(!steering.per_category.contains_key("selectors"));
        // Over-hit "datapath" down-weighted toward 0.5/1.0 = 0.5.
        assert!((steering.per_category["datapath"] - 0.5).abs() < 1e-6);
        // per_knob is untouched (the derive step targets categories).
        assert!(steering.per_knob.is_empty());
        // The derived config validates (weights finite, >= 0).
        assert!(steering.validate().is_ok());
    }

    #[test]
    fn derive_clamps_zero_fire_to_max_weight() {
        // A never-firing category (0.0) hits the epsilon floor ⇒ a large weight,
        // clamped to max_weight (not unbounded).
        let readout = readout_with_category_rates(&[("hierarchy", 0.0)]);
        let params = DeriveParams {
            target_share: 0.5,
            max_weight: 4.0,
            epsilon: 1e-3,
        };
        let steering = derive_steering_from_coverage(&readout, &params);
        assert_eq!(steering.per_category["hierarchy"], 4.0);
    }

    #[test]
    fn derive_is_deterministic() {
        // Same (readout, params) ⇒ byte-identical SteeringConfig (the weights are
        // milli-quantized, so no cross-evaluation drift).
        let readout = readout_with_category_rates(&[("state", 0.137), ("datapath", 0.291)]);
        let params = DeriveParams::default();
        let a = derive_steering_from_coverage(&readout, &params);
        let b = derive_steering_from_coverage(&readout, &params);
        assert_eq!(a, b);
        // And every weight is milli-quantized.
        for w in a.per_category.values() {
            assert_eq!(*w, (w * 1000.0).round() / 1000.0);
        }
    }
}
