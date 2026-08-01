//! `CAPABILITY-BREADTH-EXPANSION.4b.1` — post-construction annotation that
//! marks selected dynamic-selector `CaseMux` / `CasezMux` gates with an IEEE
//! 1800-2017 §12.5.3 **case qualifier** (decision `0044` + the `.4a`
//! design-detail in `DEVELOPMENT_NOTES.md`).
//!
//! A marked gate keeps the parallel `case` / `casez` statement it already emits
//! and gains a keyword prefix:
//!
//! ```systemverilog
//! always_comb begin
//!     unique case (sel)          // or: priority case (sel)
//!         2'd0: case_mux_0 = i_2;
//!         2'd1: case_mux_0 = i_2;
//!         default: case_mux_0 = 4'h0;
//!     endcase
//! end
//! ```
//!
//! # A qualifier is an ASSERTION, not a rendering
//!
//! This is what separates this pass from the nine emit-projections it runs
//! after. Each of those *replaces* how a gate renders and is judged by "does
//! the downstream tool ingest this shape?". A qualifier changes no shape: it
//! makes a **claim about the design** that a simulator checks at runtime and a
//! synthesizer acts on (full/parallel-case inference). Emitting one that can be
//! false would manufacture exactly the sim/synth divergence class ANVIL exists
//! to *find in tools*, never to inject into its own output.
//!
//! So the load-bearing question is *is the assertion true for every block the
//! generator can emit?*, and both halves fall out of the emitter with **no
//! analysis pass and no generate-then-filter**:
//!
//! - **FULL** (some `case_item` matches — asserted by both `priority` and
//!   `unique`). `crate::emit::sv` writes a `default:` arm for **every**
//!   `CaseMux`/`CasezMux` that renders as a statement. Because `default` is
//!   itself a `case_item`, a match always exists, so the "no case_item matches"
//!   violation cannot fire — regardless of how few arms the gate has.
//! - **PARALLEL** (no two `case_item`s match — asserted by `unique`). A
//!   `CaseMux` labels arm `i` as `SW'd{i}`, the sequential integers `0..N-1`.
//!   For `CasezMux`, `crate::gen::cone::motifs::build_casez_patterns` is the
//!   **sole** pattern source in the generator and gives arm `i` the care-value
//!   `i` with exactly one don't-care LSB, so two arms overlap iff two arm
//!   indices coincide — which they cannot.
//!
//! Both halves are asserted over real generated output by the property test at
//! the bottom of this file, each with its own firing negative control
//! (`DOCTRINE_ENFORCEMENT.md` §9). That test is the **durable** home for the
//! `.3` corpus measurement, whose checker was scratch under untracked `.cache/`.
//!
//! # The exclusion that actually bites
//!
//! Every sibling projection carries a sibling-mark exclusion that is *vacuous*
//! in practice — no other pass targets its gate kind. This one is real: a gate
//! claimed by `case_mux_if` / `casez_mux_if` renders as an `if`/`else if` chain
//! and has **no `case` keyword to prefix**. So this pass runs **last**, after
//! both, and the ordering is load-bearing rather than conventional.
//!
//! # Two knobs, one mechanism
//!
//! `unique` and `priority` are genuinely different assertions with different
//! downstream tool plans, so each gets its own knob, `KnobId` and metric. They
//! feed **one** pass and **one** carrier, rolled in a fixed order so a gate
//! receives at most one qualifier. Two knobs into one mechanism is not two
//! mechanisms (`feedback_full_factorization`).
//!
//! Both knobs default `0.0`, so the generator's guard skips the pass entirely,
//! nothing is drawn from the RNG, the carrier stays empty and the emitted
//! prefix is `""` ⇒ byte-identical stream *and* output. The mark is an
//! emitter-surface annotation only: the flat IR body, validators, CSE keys and
//! `canonical_module_signature` are untouched.

use crate::ir::{GateOp, Module, Node, NodeId};
use rand::Rng;

/// The IEEE 1800-2017 §12.5.3 qualifier a `case` / `casez` statement carries.
///
/// `unique0` is deliberately absent: because ANVIL always emits a `default:`
/// arm, `unique0` (parallel-only) asserts a strict subset of `unique`
/// (parallel + full) and adds nothing today. Deferred, not retired — decision
/// `0044`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum CaseQualifier {
    /// Asserts FULL **and** PARALLEL. The qualifier that adds a new downstream
    /// code path: the runtime uniqueness check and `parallel_case` inference.
    Unique,
    /// Asserts FULL only, first-match semantics.
    Priority,
}

impl CaseQualifier {
    /// The emitter's statement prefix — the keyword **plus its separating
    /// space** — so an unqualified block formats with `""` and its bytes are
    /// unchanged by construction rather than by test.
    pub fn prefix(self) -> &'static str {
        match self {
            CaseQualifier::Unique => "unique ",
            CaseQualifier::Priority => "priority ",
        }
    }
}

/// True iff the gate at `id` can carry a case qualifier: a `Node::Gate` whose
/// op is `GateOp::CaseMux` or `GateOp::CasezMux`, with a **non-constant
/// (dynamic) selector** and at least one complete arm, that is **not** already
/// claimed by a sibling emit-projection.
///
/// A constant-selector gate is statically collapsed by the emitter to a
/// continuous `assign` (`render_static_structured_gate`) and has no `case`
/// statement to qualify; excluding it is what keeps the
/// `num_emitted_{unique,priority}_cases` metrics exact.
fn gate_qualifies(m: &Module, id: NodeId, node: &Node) -> bool {
    let Node::Gate { op, operands, .. } = node else {
        return false;
    };
    // A selector plus at least one complete arm: `CaseMux` arms are single
    // data operands, `CasezMux` arms are `(value, mask, data)` triples.
    let min_operands = match op {
        GateOp::CaseMux => 2,
        GateOp::CasezMux => 4,
        // `ForFold` is a loop fold, not a selector, and every other op renders
        // as a continuous `assign`.
        _ => return false,
    };
    if operands.len() < min_operands {
        return false;
    }
    // Dynamic selector only — see the doc comment.
    if matches!(
        m.nodes.get(operands[0] as usize),
        Some(Node::Constant { .. })
    ) {
        return false;
    }
    // THE non-vacuous exclusion: a gate projected to an `if`/`else if` chain
    // by the eighth or ninth surface emits no `case` keyword to prefix. This
    // pass runs after both precisely so these two sets are populated here.
    if m.case_mux_if_gates.contains(&id) || m.casez_mux_if_gates.contains(&id) {
        return false;
    }
    // The remaining sibling exclusions are vacuous by target type — `function`
    // / `task` / `cone` reject `CaseMux`/`CasezMux` as inadmissible, `mux_if`
    // targets `Mux`, `generate_loop` targets `Concat`, `soft_union` targets
    // `Slice` — but kept for the established "later pass excludes earlier
    // marks" convention.
    if m.function_emit_gates.contains(&id)
        || m.generate_loop_gates.contains(&id)
        || m.task_emit_gates.contains(&id)
        || m.soft_union_slice_gates.contains(&id)
        || m.mux_if_gates.contains(&id)
    {
        return false;
    }
    if m.multi_output_task_groups.contains_key(&id)
        || m.multi_output_task_groups
            .values()
            .flatten()
            .any(|&x| x == id)
    {
        return false;
    }
    if m.cone_function_gates.contains_key(&id)
        || m.cone_function_gates.values().flatten().any(|&x| x == id)
    {
        return false;
    }
    true
}

/// Mark qualifying gates with a case qualifier by rolling the two knobs per
/// qualifying gate on the seeded generator RNG. Returns the number newly
/// marked. Must run **last** — after every sibling projection pass, and in
/// particular after `case_mux_if` and `casez_mux_if`, whose marks it excludes.
///
/// # Roll order, and why the second roll is skipped
///
/// `unique_prob` is rolled first; on a hit the gate takes `Unique` and
/// `priority_prob` is **not rolled for that gate**. Rolling both and letting
/// `unique` win would record a `priority` *fire* that emits no `priority`
/// qualifier — telemetry diverging from emitted reality, the exact defect class
/// decision `0034` was written to remove. With the skip,
/// `fires(knob) == the knob's metric` exactly, each knob's `coverage_readout`
/// rate stays directly interpretable, and the two metrics sum to
/// `case_qualifiers.len()`.
///
/// Each roll is additionally skipped when its probability is `0.0`. `gen_bool`
/// consumes a draw at `p == 0.0` (`rand`'s `Bernoulli::sample` short-circuits
/// only at `p == 1.0`) and the steering multiplier cannot lift `0.0`, so an
/// unguarded roll changes no outcome — it would only shift the RNG stream for
/// the *other* knob's runs and record `attempts` against a knob the user
/// disabled. Every sibling pass gets this from its single call-site guard; one
/// pass with two knobs must also guard each knob internally.
pub fn annotate_case_qualifiers(
    m: &mut Module,
    steering: &crate::config::SteeringConfig,
    rng: &mut impl Rng,
    unique_prob: f64,
    priority_prob: f64,
) -> usize {
    // Scope: leave Phase 5 parameterized modules out (their emitted widths are
    // symbolic; the param/structured cross-product is out of scope). Mirrors
    // every sibling projection pass.
    if m.param_env.is_some() {
        return 0;
    }
    let unique_p = unique_prob.clamp(0.0, 1.0);
    let priority_p = priority_prob.clamp(0.0, 1.0);
    if unique_p <= 0.0 && priority_p <= 0.0 {
        return 0;
    }
    // Collect candidates first so the immutable scan over `m.nodes` does not
    // overlap the mutable insert into `m.case_qualifiers`.
    let candidates: Vec<NodeId> = m
        .nodes
        .iter()
        .enumerate()
        .filter(|(i, n)| gate_qualifies(m, *i as NodeId, n))
        .map(|(i, _)| i as NodeId)
        .collect();
    let mut marked = 0usize;
    for id in candidates {
        let mut qualifier = None;
        if unique_p > 0.0
            && crate::ir::knob_roll::roll_knob_into(
                &mut m.knob_rolls,
                steering,
                rng,
                crate::ir::KnobId::UniqueCaseProb,
                unique_p,
            )
        {
            qualifier = Some(CaseQualifier::Unique);
        }
        if qualifier.is_none()
            && priority_p > 0.0
            && crate::ir::knob_roll::roll_knob_into(
                &mut m.knob_rolls,
                steering,
                rng,
                crate::ir::KnobId::PriorityCaseProb,
                priority_p,
            )
        {
            qualifier = Some(CaseQualifier::Priority);
        }
        if let Some(q) = qualifier {
            if m.case_qualifiers.insert(id, q).is_none() {
                marked += 1;
            }
        }
    }
    marked
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::emit::to_sv;
    use crate::gen::Generator;
    use crate::ir::{DepSet, Direction, Module, Node, Port};
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    fn rng() -> ChaCha8Rng {
        ChaCha8Rng::seed_from_u64(0)
    }

    /// `y = case (sel) 0:a 1:b 2:c` over a 2-bit dynamic selector — node 4 is
    /// the `CaseMux` candidate.
    fn module_case_mux_gate() -> Module {
        let mut m = Module {
            name: "cq".into(),
            ..Module::default()
        };
        for (id, name, width) in [(0u32, "sel", 2u32), (1, "a", 4), (2, "b", 4), (3, "c", 4)] {
            m.inputs.push(Port {
                id,
                name: name.into(),
                width,
                dir: Direction::In,
            });
        }
        m.outputs.push(Port {
            id: 4,
            name: "y".into(),
            width: 4,
            dir: Direction::Out,
        });
        m.nodes.push(Node::PrimaryInput { port: 0, width: 2 }); // 0 (sel)
        m.nodes.push(Node::PrimaryInput { port: 1, width: 4 }); // 1 (a)
        m.nodes.push(Node::PrimaryInput { port: 2, width: 4 }); // 2 (b)
        m.nodes.push(Node::PrimaryInput { port: 3, width: 4 }); // 3 (c)
        m.nodes.push(Node::Gate {
            op: GateOp::CaseMux,
            operands: vec![0, 1, 2, 3],
            width: 4,
            deps: DepSet::new(),
        }); // 4
        m.drives.push((4, 4));
        m
    }

    /// The same shape but with a **constant** selector — statically collapsed
    /// by the emitter, so never a candidate.
    fn module_static_case_mux_gate() -> Module {
        let mut m = module_case_mux_gate();
        m.nodes[0] = Node::Constant { value: 1, width: 2 };
        m
    }

    /// `y = casez (sel) 2'b0?:a 2'b1?:b` over a 2-bit dynamic selector, built
    /// the way `build_casez_patterns` builds them (arm `i` → care-value `i`
    /// with one don't-care LSB). Node 5 is the `CasezMux` candidate.
    fn module_casez_mux_gate() -> Module {
        let mut m = Module {
            name: "cqz".into(),
            ..Module::default()
        };
        for (id, name, width) in [(0u32, "sel", 2u32), (1, "a", 4), (2, "b", 4)] {
            m.inputs.push(Port {
                id,
                name: name.into(),
                width,
                dir: Direction::In,
            });
        }
        m.outputs.push(Port {
            id: 3,
            name: "y".into(),
            width: 4,
            dir: Direction::Out,
        });
        m.nodes.push(Node::PrimaryInput { port: 0, width: 2 }); // 0 (sel)
        m.nodes.push(Node::PrimaryInput { port: 1, width: 4 }); // 1 (a)
        m.nodes.push(Node::PrimaryInput { port: 2, width: 4 }); // 2 (b)
        m.nodes.push(Node::Constant { value: 0, width: 2 }); // 3 pattern 2'b0?
        m.nodes.push(Node::Constant { value: 1, width: 2 }); // 4 wildcard mask (LSB)
        m.nodes.push(Node::Gate {
            op: GateOp::CasezMux,
            // [sel, (value, mask, data), (value, mask, data)]
            operands: vec![0, 3, 4, 1, 6, 4, 2],
            width: 4,
            deps: DepSet::new(),
        }); // 5
        m.nodes.push(Node::Constant { value: 2, width: 2 }); // 6 pattern 2'b1?
        m.drives.push((3, 5));
        m
    }

    #[test]
    fn prob_one_marks_a_dynamic_case_mux() {
        let mut m = module_case_mux_gate();
        let n = annotate_case_qualifiers(&mut m, &Default::default(), &mut rng(), 1.0, 0.0);
        assert_eq!(n, 1);
        assert_eq!(m.case_qualifiers.get(&4), Some(&CaseQualifier::Unique));
    }

    #[test]
    fn prob_one_marks_a_dynamic_casez_mux() {
        let mut m = module_casez_mux_gate();
        let n = annotate_case_qualifiers(&mut m, &Default::default(), &mut rng(), 0.0, 1.0);
        assert_eq!(n, 1);
        assert_eq!(m.case_qualifiers.get(&5), Some(&CaseQualifier::Priority));
    }

    #[test]
    fn constant_selector_gate_is_excluded() {
        let mut m = module_static_case_mux_gate();
        let n = annotate_case_qualifiers(&mut m, &Default::default(), &mut rng(), 1.0, 1.0);
        assert_eq!(n, 0, "a constant-selector CaseMux is statically collapsed");
        assert!(m.case_qualifiers.is_empty());
    }

    #[test]
    fn both_probs_zero_marks_nothing_byte_identical() {
        let mut m = module_case_mux_gate();
        let n = annotate_case_qualifiers(&mut m, &Default::default(), &mut rng(), 0.0, 0.0);
        assert_eq!(n, 0);
        assert!(m.case_qualifiers.is_empty());
    }

    /// The **non-vacuous** exclusion, both directions: a gate the eighth or
    /// ninth surface already claimed renders as an `if`/`else if` chain and has
    /// no `case` keyword to prefix, so it is never qualified — while the same
    /// gate unclaimed is.
    #[test]
    fn chain_projected_gate_is_never_qualified() {
        let mut claimed = module_case_mux_gate();
        claimed.case_mux_if_gates.insert(4);
        assert_eq!(
            annotate_case_qualifiers(&mut claimed, &Default::default(), &mut rng(), 1.0, 1.0),
            0,
            "a CaseMux projected to an if/else-if chain has no `case` to qualify"
        );
        assert!(claimed.case_qualifiers.is_empty());

        let mut claimed_z = module_casez_mux_gate();
        claimed_z.casez_mux_if_gates.insert(5);
        assert_eq!(
            annotate_case_qualifiers(&mut claimed_z, &Default::default(), &mut rng(), 1.0, 1.0),
            0,
            "a CasezMux projected to a masked chain has no `casez` to qualify"
        );
        assert!(claimed_z.case_qualifiers.is_empty());

        // The control: unclaimed, the very same gates DO qualify — so the
        // assertions above are about the exclusion, not about an inert setup.
        let mut free = module_case_mux_gate();
        assert_eq!(
            annotate_case_qualifiers(&mut free, &Default::default(), &mut rng(), 1.0, 1.0),
            1
        );
        let mut free_z = module_casez_mux_gate();
        assert_eq!(
            annotate_case_qualifiers(&mut free_z, &Default::default(), &mut rng(), 1.0, 1.0),
            1
        );
    }

    /// Precedence: `unique` wins, and the `priority` roll is **not taken** for
    /// a gate `unique` claimed — so each knob's recorded fires equal the number
    /// of qualifiers it actually produced.
    #[test]
    fn unique_wins_and_priority_is_not_rolled_on_a_hit() {
        let mut m = module_case_mux_gate();
        annotate_case_qualifiers(&mut m, &Default::default(), &mut rng(), 1.0, 1.0);
        assert_eq!(m.case_qualifiers.get(&4), Some(&CaseQualifier::Unique));

        let attempts = m.knob_rolls.attempts();
        let fires = m.knob_rolls.fires();
        assert_eq!(attempts.get(&crate::ir::KnobId::UniqueCaseProb), Some(&1));
        assert_eq!(fires.get(&crate::ir::KnobId::UniqueCaseProb), Some(&1));
        assert_eq!(
            attempts.get(&crate::ir::KnobId::PriorityCaseProb),
            None,
            "the priority roll must be skipped once unique has claimed the gate — \
             a recorded fire that emits nothing is the decision-0034 defect"
        );
    }

    /// A knob at `0.0` is not rolled at all, so it records no attempts against
    /// a capability the user disabled (and does not shift the other knob's
    /// draw sequence).
    #[test]
    fn a_zero_probability_knob_is_not_rolled() {
        let mut m = module_case_mux_gate();
        annotate_case_qualifiers(&mut m, &Default::default(), &mut rng(), 0.0, 1.0);
        assert_eq!(
            m.knob_rolls
                .attempts()
                .get(&crate::ir::KnobId::UniqueCaseProb),
            None
        );
        assert_eq!(
            m.knob_rolls
                .attempts()
                .get(&crate::ir::KnobId::PriorityCaseProb),
            Some(&1)
        );
    }

    #[test]
    fn non_selector_gate_does_not_qualify() {
        let mut m = Module {
            name: "ng".into(),
            ..Module::default()
        };
        for (id, name, width) in [(0u32, "sel", 1u32), (1, "a", 4), (2, "b", 4)] {
            m.inputs.push(Port {
                id,
                name: name.into(),
                width,
                dir: Direction::In,
            });
        }
        m.nodes.push(Node::PrimaryInput { port: 0, width: 1 });
        m.nodes.push(Node::PrimaryInput { port: 1, width: 4 });
        m.nodes.push(Node::PrimaryInput { port: 2, width: 4 });
        m.nodes.push(Node::Gate {
            op: GateOp::Mux,
            operands: vec![0, 1, 2],
            width: 4,
            deps: DepSet::new(),
        });
        assert_eq!(
            annotate_case_qualifiers(&mut m, &Default::default(), &mut rng(), 1.0, 1.0),
            0
        );
        assert!(m.case_qualifiers.is_empty());
    }

    #[test]
    fn param_env_module_is_skipped() {
        use crate::ir::ParamEnv;
        let mut m = module_case_mux_gate();
        m.param_env = Some(ParamEnv {
            name: "W".into(),
            min: 2,
            max: 8,
            design_value: 4,
        });
        assert_eq!(
            annotate_case_qualifiers(&mut m, &Default::default(), &mut rng(), 1.0, 1.0),
            0,
            "parameterized modules are out of scope"
        );
    }

    #[test]
    fn marking_leaves_identity_and_node_count_untouched() {
        let mut m = module_case_mux_gate();
        let nodes_before = m.nodes.len();
        let sig_before = crate::metrics::canonical_module_signature(&m);
        annotate_case_qualifiers(&mut m, &Default::default(), &mut rng(), 1.0, 0.0);
        assert_eq!(m.nodes.len(), nodes_before, "no new IR node");
        assert_eq!(
            crate::metrics::canonical_module_signature(&m),
            sig_before,
            "identity is unaffected by the emitter-surface mark"
        );
    }

    /// End-to-end: a marked gate emits the qualifier keyword, an unmarked one
    /// emits the bare statement, and the `default:` arm — the thing that makes
    /// FULL true — survives in **both**.
    #[test]
    fn marked_gate_emits_the_qualifier_unmarked_is_bare() {
        let bare = to_sv(&module_case_mux_gate());
        assert!(bare.contains("        case (sel)"), "{bare}");
        assert!(!bare.contains("unique"), "{bare}");
        assert!(!bare.contains("priority"), "{bare}");
        assert!(bare.contains("default:"), "{bare}");

        let mut uniq = module_case_mux_gate();
        uniq.case_qualifiers.insert(4, CaseQualifier::Unique);
        let out = to_sv(&uniq);
        assert!(out.contains("        unique case (sel)"), "{out}");
        assert!(
            out.contains("default:"),
            "the default arm is what makes FULL true and must survive:\n{out}"
        );

        let mut prio = module_casez_mux_gate();
        prio.case_qualifiers.insert(5, CaseQualifier::Priority);
        let outz = to_sv(&prio);
        assert!(outz.contains("        priority casez (sel)"), "{outz}");
        assert!(outz.contains("default:"), "{outz}");
    }

    // ---------------------------------------------------------------------
    // The FULL + PARALLEL property, over real generated output.
    //
    // The durable replacement for the `.3` corpus measurement, whose checker
    // was scratch under untracked `.cache/`. Two predicates, because the two
    // properties have different sources: PARALLEL is a fact about the IR the
    // generator built, FULL is a fact about what the emitter wrote.
    // ---------------------------------------------------------------------

    fn bitmask(width: u32) -> u128 {
        if width >= 128 {
            u128::MAX
        } else {
            (1u128 << width) - 1
        }
    }

    fn constant_of(m: &Module, id: NodeId) -> Option<u128> {
        match m.nodes.get(id as usize) {
            Some(Node::Constant { value, .. }) => Some(*value),
            _ => None,
        }
    }

    /// Every dynamic-selector `CaseMux`/`CasezMux` that renders as a statement,
    /// as `(node id, is_casez)`.
    fn qualifiable_blocks(m: &Module) -> Vec<(NodeId, bool)> {
        m.nodes
            .iter()
            .enumerate()
            .filter_map(|(idx, node)| {
                let Node::Gate { op, operands, .. } = node else {
                    return None;
                };
                let is_casez = match op {
                    GateOp::CaseMux => false,
                    GateOp::CasezMux => true,
                    _ => return None,
                };
                let id = idx as NodeId;
                // Statically collapsed to a continuous assign, or projected to
                // an if/else-if chain — neither emits a `case` statement.
                if constant_of(m, operands[0]).is_some()
                    || m.case_mux_if_gates.contains(&id)
                    || m.casez_mux_if_gates.contains(&id)
                {
                    return None;
                }
                Some((id, is_casez))
            })
            .collect()
    }

    /// PARALLEL, read off the **IR**: no two arms of a qualifiable block can
    /// match the same selector value. Returns one message per violation.
    ///
    /// For `CaseMux` the arms are labelled `SW'd{i}`, so the only way two
    /// labels can collide is an arm index that does not fit the selector width
    /// — the literal would truncate. For `CasezMux` two arms overlap iff they
    /// agree on every bit both of them care about, i.e.
    /// `((v_i ^ v_j) & c_i & c_j) == 0` with `c = ~wildcard & sel_mask` and
    /// `v = pattern & c`. This reads the real arm constants rather than
    /// assuming `build_casez_patterns` produced them.
    fn parallel_violations(m: &Module) -> Vec<String> {
        let mut out = Vec::new();
        for (id, is_casez) in qualifiable_blocks(m) {
            let Node::Gate { operands, .. } = &m.nodes[id as usize] else {
                continue;
            };
            let sel_width = m.nodes[operands[0] as usize].width();
            let sel_mask = bitmask(sel_width);
            if is_casez {
                let mut arms: Vec<(usize, u128, u128)> = Vec::new();
                for (arm_idx, arm) in operands[1..].chunks_exact(3).enumerate() {
                    let (Some(pattern), Some(wildcard)) =
                        (constant_of(m, arm[0]), constant_of(m, arm[1]))
                    else {
                        out.push(format!(
                            "node {id}: casez arm {arm_idx} has a non-constant pattern or mask, \
                             so its match set is not statically known and PARALLEL cannot be \
                             established"
                        ));
                        continue;
                    };
                    let care = !wildcard & sel_mask;
                    if care == 0 {
                        out.push(format!(
                            "node {id}: casez arm {arm_idx} cares about no bit, so it matches \
                             every selector value and overlaps every other arm"
                        ));
                    }
                    arms.push((arm_idx, pattern & care, care));
                }
                for i in 0..arms.len() {
                    for j in (i + 1)..arms.len() {
                        let (ai, vi, ci) = arms[i];
                        let (aj, vj, cj) = arms[j];
                        if ((vi ^ vj) & ci & cj) == 0 {
                            out.push(format!(
                                "node {id}: casez arms {ai} and {aj} overlap — some selector \
                                 value matches both, so `unique` would be a FALSE assertion"
                            ));
                        }
                    }
                }
            } else {
                for arm_idx in 0..(operands.len() - 1) {
                    if arm_idx as u128 > sel_mask {
                        out.push(format!(
                            "node {id}: case arm {arm_idx} does not fit a {sel_width}-bit label \
                             (max {sel_mask}) — `{sel_width}'d{arm_idx}` truncates and collides \
                             with an earlier arm"
                        ));
                    }
                }
            }
        }
        out
    }

    /// FULL, read off the **emitted text**: every `case`/`casez`/`casex` block
    /// closes with a `default:` arm at its own nesting depth.
    ///
    /// Scoped to ANVIL-emitted SystemVerilog — this is not a parser, it is a
    /// line scan over an emitter that writes one statement per line. Two things
    /// it must get right, both learned at `.3`, where the first version of this
    /// check silently reported success about blocks it had never looked at:
    ///
    /// - **openers are matched anywhere in the line, not at its start**,
    ///   because the FSM emitter nests an inner block on the arm-label line
    ///   (`S0: case (sel)`) and a prefix match would miss every one of them;
    /// - **a qualifier prefix must not hide an opener**, which the substring
    ///   match gives for free (`unique case (` still contains `case (`).
    fn full_violations(sv: &str) -> Vec<String> {
        let mut out = Vec::new();
        // (1-based line where the block opened, whether a `default:` was seen)
        let mut open: Vec<(usize, bool)> = Vec::new();
        for (i, raw) in sv.lines().enumerate() {
            let lineno = i + 1;
            let line = raw.trim();
            if line.contains("endcase") {
                match open.pop() {
                    Some((opened_at, saw_default)) => {
                        if !saw_default {
                            out.push(format!(
                                "line {lineno}: the block opened at line {opened_at} closes with \
                                 no `default:` arm — FULL is not established, so a qualifier \
                                 could report a no-match violation"
                            ));
                        }
                    }
                    None => out.push(format!("line {lineno}: `endcase` with no open block")),
                }
                continue;
            }
            if line.contains("case (") || line.contains("casez (") || line.contains("casex (") {
                open.push((lineno, false));
                continue;
            }
            if line.starts_with("default:") {
                match open.last_mut() {
                    Some(top) => top.1 = true,
                    None => out.push(format!("line {lineno}: `default:` outside any block")),
                }
            }
        }
        for (opened_at, _) in open {
            out.push(format!(
                "line {opened_at}: block never closed with `endcase`"
            ));
        }
        out
    }

    /// A selector-rich comb-only shape so the corpus actually contains both
    /// block kinds. `comb_mux_prob` is zeroed because the structured-block
    /// rolls are a short-circuit chain in the cone builder — comb-mux rolls
    /// before case-mux, which rolls before casez-mux — so a live `comb_mux_prob`
    /// starves the two kinds under test.
    fn selector_rich_config(seed: u64) -> Config {
        Config {
            seed,
            flop_prob: 0.0,
            comb_mux_prob: 0.0,
            case_mux_prob: 0.5,
            casez_mux_prob: 0.5,
            ..Config::default()
        }
    }

    /// The property, over real generated output: every emitted `case`/`casez`
    /// block ANVIL can qualify is FULL and PARALLEL.
    #[test]
    fn generated_case_and_casez_blocks_are_full_and_parallel() {
        let mut total_case = 0usize;
        let mut total_casez = 0usize;
        for seed in 1..=8u64 {
            let mut g = Generator::new(selector_rich_config(seed));
            let m = g.generate_module();
            for (_, is_casez) in qualifiable_blocks(&m) {
                if is_casez {
                    total_casez += 1;
                } else {
                    total_case += 1;
                }
            }
            let parallel = parallel_violations(&m);
            assert!(parallel.is_empty(), "seed {seed} PARALLEL: {parallel:#?}");
            let full = full_violations(&to_sv(&m));
            assert!(full.is_empty(), "seed {seed} FULL: {full:#?}");
        }
        // Non-vacuity: a clean result means nothing if the corpus held no
        // blocks of either kind (the `.3` lesson — a checker that silently
        // looks at nothing reports success).
        assert!(
            total_case > 0,
            "vacuous: the corpus contained no dynamic CaseMux block"
        );
        assert!(
            total_casez > 0,
            "vacuous: the corpus contained no dynamic CasezMux block"
        );
    }

    /// Negative control for PARALLEL: a hand-built `CasezMux` whose two arms
    /// both match `2'b00`. Without this, "zero violations" is indistinguishable
    /// from "the checker never ran" (`DOCTRINE_ENFORCEMENT.md` §9).
    #[test]
    fn parallel_violations_fires_on_overlapping_casez_arms() {
        let mut m = module_casez_mux_gate();
        assert!(
            parallel_violations(&m).is_empty(),
            "the disjoint fixture is clean before the arms are broken"
        );
        // Widen the second arm's wildcard mask to `2'b11`: it now cares about
        // no bit and matches everything, overlapping the first arm.
        m.nodes[4] = Node::Constant { value: 3, width: 2 };
        let violations = parallel_violations(&m);
        assert!(
            violations.len() >= 2,
            "expected an all-wildcard report AND an overlap report, got {violations:#?}"
        );
        assert!(
            violations.iter().any(|v| v.contains("overlap")),
            "{violations:#?}"
        );
    }

    /// Negative control for PARALLEL on the `CaseMux` side: five arms cannot be
    /// labelled distinctly over a 2-bit selector.
    #[test]
    fn parallel_violations_fires_on_arm_labels_that_do_not_fit() {
        let mut m = module_case_mux_gate();
        assert!(parallel_violations(&m).is_empty(), "3 arms fit 2 bits");
        // Two more arms: indices 3 and 4, and `2'd4` truncates to `2'd0`.
        m.nodes.push(Node::Constant { value: 7, width: 4 }); // 5
        m.nodes.push(Node::Constant { value: 9, width: 4 }); // 6
        let Node::Gate { operands, .. } = &mut m.nodes[4] else {
            panic!("node 4 is the CaseMux");
        };
        operands.push(5);
        operands.push(6);
        let violations = parallel_violations(&m);
        assert!(
            violations.iter().any(|v| v.contains("truncates")),
            "{violations:#?}"
        );
    }

    /// Negative control for FULL: a `case` block with no `default:` arm, and a
    /// nested block whose *inner* default is missing while the outer one is
    /// present — the case the `.3` checker got wrong by being nesting-unaware.
    #[test]
    fn full_violations_fires_on_a_default_less_case() {
        let good = "\
    always_comb begin
        case (sel)
            2'd0: y = a;
            default: y = 4'h0;
        endcase
    end";
        assert!(full_violations(good).is_empty(), "the clean fixture passes");

        let bad = "\
    always_comb begin
        case (sel)
            2'd0: y = a;
        endcase
    end";
        assert_eq!(full_violations(bad).len(), 1, "{:#?}", full_violations(bad));

        // Nested: the outer block has a default, the inner one does not. A
        // nesting-unaware scan credits the outer default to the inner block and
        // reports clean.
        let nested_bad = "\
    always_comb begin
        case (state_q)
            S0: case (sel)
                1'h0: nx = S1;
            endcase
            default: nx = S0;
        endcase
    end";
        let violations = full_violations(nested_bad);
        assert_eq!(violations.len(), 1, "{violations:#?}");
        assert!(
            violations[0].contains("opened at line 3"),
            "the INNER block is the one reported: {violations:#?}"
        );
    }
}
