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
//! Every map is a `BTreeMap`, every rate is `fires as f64 / attempts as f64`
//! (exact IEEE-754, attempts `>= 1` for any present key), so a
//! [`CoverageReadout`] is a byte-stable function of its input [`Metrics`].

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
}
