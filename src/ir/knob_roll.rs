//! The **single** knob-roll primitive
//! (`COVERAGE-STEERED-GENERATION.3b`, decision `0034`).
//!
//! Every construction-time probability roll that is attributed to a
//! [`KnobId`] goes through [`roll_knob_into`]. There is exactly one such
//! function, and this module is the reason there can only be one:
//! [`KnobRollCounters::record`] is **private to this module**, so the only
//! way to commit a roll to the telemetry is to call the primitive — which
//! applies [`SteeringConfig::effective_prob`] first.
//!
//! ## Why the privacy matters (the defect this module exists to prevent)
//!
//! `COVERAGE-STEERED-GENERATION.2a` added the coverage-steering prior at
//! `gen::cone::roll_knob` on the premise that every steerable roll funnelled
//! through that one function. It did not. `src/gen/hierarchy.rs` had, two
//! months earlier, grown **seven roll primitives of its own** that recorded
//! the identical `knob_rolls` telemetry while omitting the prior. The
//! consequence went unnoticed for two months: 6 of the 22 `KnobId`s were
//! never steered, and `--steer hierarchy=<weight>` — one of the six
//! documented steering categories — was a **silent no-op**, accepted and
//! echoed by the CLI while changing nothing (a 9x weight left the recorded
//! fire counts bit-identical; an 800x category spread emitted byte-identical
//! SV).
//!
//! A second primitive is now a **compile error** rather than a review
//! question. That is repair rung **R2** of the ladder in `MEMORY.md`
//! (derive → compile error → derived test → registered doctrine); a doctrine
//! check was deliberately rejected as a mechanism maintained forever for
//! something the visibility rules enforce for free.
//!
//! ## Why the fork happened, and what stops the next one
//!
//! Measured at `.3b`, not assumed: the pre-`.3b` `roll_knob` was a
//! **module-private `fn` in `gen::cone`**, so `gen::hierarchy` — a sibling
//! module — simply could not call it, and wrote its own instead. The borrow
//! shape was never the obstacle; `Generator::roll_knob(&mut self, m, ..)`
//! compiles unchanged at all seven hierarchy call sites (two-phase borrows
//! cover the `g.roll_knob(m, k, g.cfg.x)` form). See decision `0034`'s
//! "Correction" section.
//!
//! So the durable fix is not a wider signature — it is that the *effect* is
//! now unreachable except through the primitive. A future module that cannot
//! see `Generator::roll_knob` still cannot record a roll: it gets `E0624:
//! method 'record' is private`, verified by negative control at `.3b`.
//!
//! `.3b` guarded only the *method*, which left the equivalent bypass — writing
//! `KnobRollCounters`' maps directly — compiling clean. `COVERAGE-STEERED-GENERATION.5`
//! closes that: the two fields are private too, readable through
//! [`KnobRollCounters::attempts`] / [`KnobRollCounters::fires`]. The rule the
//! two slices add up to: **when an invariant is made a compile error, the
//! protected thing is *state*, not a function — enumerate every syntactic route
//! that mutates it, not just the one you happened to write.**
//!
//! The primitive takes `&mut KnobRollCounters` rather than `&mut Module`
//! purely to keep this module free of a dependency on `ir::types::Module`
//! (the counters are all it needs).

use std::collections::HashMap;

use rand::Rng;

use crate::config::SteeringConfig;
use crate::ir::types::KnobId;

/// Live per-knob roll counters. Written only by [`roll_knob_into`]; the
/// empirical ratio `fires[knob] / attempts[knob]` should converge to the
/// knob's effective probability as the module grows.
///
/// Both maps are **private** and readable only through
/// [`attempts`](KnobRollCounters::attempts) / [`fires`](KnobRollCounters::fires)
/// (`metrics::compute` projects them into `Metrics::knob_roll_attempts` /
/// `knob_roll_fires`, which `introspect::coverage` in turn projects into the
/// `coverage_readout`).
///
/// The privacy is the guard, and it covers the **state**, not just the API
/// (`COVERAGE-STEERED-GENERATION.5`). `.3b` privatised `record` alone and left
/// these two fields `pub`, which left the equivalent bypass compiling clean:
///
/// ```ignore
/// let fired = rng.gen_bool(prob);                                  // no prior
/// *m.knob_rolls.attempts.entry(knob).or_insert(0) += 1;            // used to compile
/// if fired { *m.knob_rolls.fires.entry(knob).or_insert(0) += 1; }  // used to compile
/// ```
///
/// A guard on the method guards a habit; a guard on the state guards the
/// invariant.
#[derive(Debug, Clone, Default)]
pub struct KnobRollCounters {
    attempts: HashMap<KnobId, u64>,
    fires: HashMap<KnobId, u64>,
}

impl KnobRollCounters {
    /// Attempts per knob — every roll, fired or not. Read-only by design; the
    /// only writer in the crate is [`roll_knob_into`].
    pub fn attempts(&self) -> &HashMap<KnobId, u64> {
        &self.attempts
    }

    /// Fires per knob — the subset of [`attempts`](KnobRollCounters::attempts)
    /// that came up true. Read-only by design.
    pub fn fires(&self) -> &HashMap<KnobId, u64> {
        &self.fires
    }

    /// Record one probability-roll outcome.
    ///
    /// **Private to this module by design** — see the module docs. Callers
    /// reach it only through [`roll_knob_into`], which guarantees the
    /// steering prior was applied to the probability that produced `fired`.
    /// Making this — or either field — `pub` again re-opens the exact defect
    /// decision `0034` closed.
    fn record(&mut self, knob: KnobId, fired: bool) {
        *self.attempts.entry(knob).or_insert(0) += 1;
        if fired {
            *self.fires.entry(knob).or_insert(0) += 1;
        }
    }
}

/// Perform one probability roll against a named knob and record the attempt +
/// outcome. **The only knob-roll primitive in the crate.**
///
/// Two things happen here and nowhere else:
///
/// 1. **The steering prior** (`COVERAGE-STEERED-GENERATION.2a`, decision
///    `0023`): a construction-time probability *multiplier* applied before
///    the single draw. Rules-first — there is no rejection path and the draw
///    count is unchanged (exactly one `gen_bool` per roll), so output stays
///    byte-stable per `(seed, knobs, steering-config)`. When steering is
///    unset, [`SteeringConfig::effective_prob`] short-circuits to
///    `prob.min(1.0)`, so the unsteered path is byte-identical.
/// 2. **The telemetry** the outer measure→derive→re-steer loop reads back
///    (`--introspect`'s `coverage_readout`, the MCP `coverage` tool).
///
/// Keeping them in one function is what makes "steered" and "measured" the
/// same set of rolls (decision `0034`).
pub(crate) fn roll_knob_into(
    rolls: &mut KnobRollCounters,
    steering: &SteeringConfig,
    rng: &mut impl Rng,
    knob: KnobId,
    prob: f64,
) -> bool {
    let effective_prob = steering.effective_prob(knob, prob);
    let fired = rng.gen_bool(effective_prob);
    rolls.record(knob, fired);
    fired
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    fn rng() -> ChaCha8Rng {
        ChaCha8Rng::seed_from_u64(7)
    }

    /// The primitive records every attempt and only the fires, for the exact
    /// knob it was given.
    #[test]
    fn primitive_records_attempts_and_fires() {
        let mut rolls = KnobRollCounters::default();
        let steering = SteeringConfig::default();
        let mut r = rng();

        for _ in 0..64 {
            roll_knob_into(
                &mut rolls,
                &steering,
                &mut r,
                KnobId::HierarchySiblingRouteProb,
                0.5,
            );
        }

        let attempts = rolls.attempts()[&KnobId::HierarchySiblingRouteProb];
        let fires = rolls.fires()[&KnobId::HierarchySiblingRouteProb];
        assert_eq!(attempts, 64, "every roll is an attempt");
        assert!(fires > 0 && fires < attempts, "p=0.5 should mix outcomes");
        assert!(
            !rolls.attempts().contains_key(&KnobId::FlopProb),
            "a roll must not be attributed to any other knob"
        );
    }

    /// Decision `0034`, the regression at unit level: the prior reaches a
    /// **hierarchy** knob. Before `.3b` this knob's only roll site bypassed
    /// `effective_prob` entirely, so a category weight could not move it.
    #[test]
    fn primitive_applies_the_steering_prior_to_hierarchy_knobs() {
        fn fire_rate(steering: &SteeringConfig) -> f64 {
            let mut rolls = KnobRollCounters::default();
            let mut r = rng();
            for _ in 0..512 {
                roll_knob_into(
                    &mut rolls,
                    steering,
                    &mut r,
                    KnobId::HierarchyChildInputConeProb,
                    0.1,
                );
            }
            let knob = KnobId::HierarchyChildInputConeProb;
            rolls.fires().get(&knob).copied().unwrap_or(0) as f64 / rolls.attempts()[&knob] as f64
        }

        let baseline = fire_rate(&SteeringConfig::default());

        let mut per_category = std::collections::BTreeMap::new();
        per_category.insert("hierarchy".to_string(), 8.0);
        let steered = fire_rate(&SteeringConfig {
            per_category,
            ..Default::default()
        });

        assert!(
            steered > baseline + 0.3,
            "an 8x `hierarchy` category weight must raise a hierarchy knob's \
             fire rate (0.1 -> 0.8); got steered={steered:.4} \
             baseline={baseline:.4}"
        );
    }

    /// The prior is exact at weight `1.0` — not merely short-circuited — so a
    /// steering config that names a category but leaves it neutral consumes
    /// the same RNG draws and yields the same outcomes.
    #[test]
    fn neutral_weight_yields_the_identical_roll_sequence() {
        fn sequence(steering: &SteeringConfig) -> Vec<bool> {
            let mut rolls = KnobRollCounters::default();
            let mut r = rng();
            (0..128)
                .map(|_| {
                    roll_knob_into(
                        &mut rolls,
                        steering,
                        &mut r,
                        KnobId::HierarchySiblingRouteProb,
                        0.37,
                    )
                })
                .collect()
        }

        let mut per_category = std::collections::BTreeMap::new();
        per_category.insert("hierarchy".to_string(), 1.0);
        per_category.insert("state".to_string(), 1.0);
        let neutral = SteeringConfig {
            per_category,
            ..Default::default()
        };

        assert_eq!(
            sequence(&SteeringConfig::default()),
            sequence(&neutral),
            "an explicit neutral weight must be byte-exact, not approximate"
        );
    }
}
