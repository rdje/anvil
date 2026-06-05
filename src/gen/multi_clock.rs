//! `MULTI-CLOCK-CDC.3a` / `SIGNOFF-SURFACE-EXPANSION.1` —
//! synchronizer construction primitive for cross-clock-domain signals.
//!
//! Per `MULTI-CLOCK-CDC.1`'s design (`DEVELOPMENT_NOTES.md`
//! "Multi-clock + CDC primitives design (2026-05-24,
//! MULTI-CLOCK-CDC.1)"): when ANVIL emits a flop in domain B
//! whose D-cone references a flop output in domain A, the cone
//! must dereference a synchronizer in domain B, not the bare
//! cross-domain Q. The default is the original 2-flop chain; raising
//! `Config::cdc_synchronizer_stages` builds an N-flop chain with the
//! same by-construction shape.
//!
//! Why a primitive in isolation first: the generator-side
//! integration is sensitive (it touches the per-module
//! construction pipeline + risks breaking the byte-identical
//! default-`dut` book-runnable contract from Phase 9). Per the
//! proven Phase-7/8/9 + `DIFFERENTIAL-SIMULATION.2a`/`.3a`
//! design-first discipline, landing the construction primitive
//! with cargo-portable proofs FIRST gives `.3b` a stable
//! library surface to build against.
//!
//! By-construction discipline:
//!
//! - **Rules-first generation** (`feedback_rules_first_generation.md`):
//!   we never generate a cross-domain path then filter it. The
//!   caller (`.3b`'s Generator integration) decides "I have a
//!   `src_q` in domain A that some flop in B wants to use as a
//!   D-cone operand" — this primitive constructs the
//!   synchronizer in place, returning the synchronized NodeId
//!   that B's flop should reference.
//!
//! - **Full factorization** (`feedback_full_factorization.md`):
//!   the synchronizer flops are distinct state elements; their Q
//!   nodes are real `NodeId`s in the Module's node table,
//!   participating in the existing identity discipline (Relaxed mode
//!   forces fresh nodes; NodeId mode CSEs would be a future
//!   consideration for synchronizer reuse — but the canonical rule is
//!   "one synchronizer per (src, dst_domain) pair", not structural CSE,
//!   so today's identity discipline already gives the right answer).

use crate::ir::{
    ClockDomain, Direction, Flop, FlopId, FlopKind, FlopMux, Module, Node, NodeId, Port, PortId,
    ResetKind,
};

/// Result of constructing a synchronizer chain.
#[derive(Debug, Clone, Copy)]
pub struct SynchronizerChain {
    /// FlopId of the first synchronizer flop (D = src_q;
    /// captures the metastable transition).
    pub first_flop: FlopId,
    /// FlopId of the second synchronizer flop (D = first.Q;
    /// resolves the metastable transition).
    pub second_flop: FlopId,
    /// NodeId of the final stage's Q — the synchronized signal in
    /// `dst_domain`. This is what downstream-in-dst-domain cones
    /// should reference.
    pub synced_q: NodeId,
    /// Number of destination-domain flops in the synchronizer chain.
    pub num_stages: u32,
}

/// Construct a 2-flop synchronizer chain in `dst_domain` driven
/// by `src_q`. Mutates `module` to allocate two new flops + two
/// new `Node::FlopQ` entries, registers their domain in
/// `module.flop_domains`, and returns the chain handle. Both
/// flops carry `ResetKind::Async`, `FlopKind::ZeroDefault`,
/// `FlopMux::None`, `reset_val = 0` — the standard
/// flop-synchronizer template.
///
/// **Width must match.** `src_q`'s width is read from `module`'s
/// node table; the synchronizer flops inherit it. Multi-bit
/// signals should be transferred via async FIFO + handshake
/// (deferred to a follow-up tree per `.1`'s catalogue tier 3-5);
/// `.3a` accepts any width to keep the primitive general, but
/// the caller is responsible for choosing 1-bit signals only —
/// the **rule** that enforces 1-bit-only is part of `.3b`'s
/// knob-side decision logic, not this primitive's contract.
///
/// `dst_domain` is the domain index into
/// `module.clock_domains`; the caller is responsible for having
/// populated `clock_domains` such that `dst_domain` is a valid
/// index.
///
/// Returns `None` when `src_q` is not a known node in `module`
/// (defensive — the caller should always pass a valid NodeId).
pub fn construct_2flop_synchronizer(
    module: &mut Module,
    src_q: NodeId,
    dst_domain: u32,
) -> Option<SynchronizerChain> {
    construct_nflop_synchronizer(module, src_q, dst_domain, 2)
}

/// Construct an N-flop synchronizer chain in `dst_domain` driven by
/// `src_q`. `stages` must be >= 2; use
/// [`construct_2flop_synchronizer`] for the default compatibility
/// primitive.
pub fn construct_nflop_synchronizer(
    module: &mut Module,
    src_q: NodeId,
    dst_domain: u32,
    stages: u32,
) -> Option<SynchronizerChain> {
    if stages < 2 {
        return None;
    }

    let width = node_width(module, src_q)?;

    let mut previous_q = src_q;
    let mut first_flop = None;
    let mut second_flop = None;
    let mut synced_q = None;

    for stage in 0..stages {
        let flop_id = module.flops.len() as FlopId;
        let q_node = module.nodes.len() as NodeId;
        module.nodes.push(Node::FlopQ {
            flop: flop_id,
            width,
        });
        module.flops.push(Flop {
            id: flop_id,
            width,
            d: Some(previous_q),
            q: q_node,
            reset_val: 0,
            reset_kind: ResetKind::Async,
            kind: FlopKind::ZeroDefault,
            mux: FlopMux::None,
        });
        module.flop_domains.insert(flop_id, dst_domain);

        if stage == 0 {
            first_flop = Some(flop_id);
        } else if stage == 1 {
            second_flop = Some(flop_id);
        }
        previous_q = q_node;
        synced_q = Some(q_node);
    }

    Some(SynchronizerChain {
        first_flop: first_flop?,
        second_flop: second_flop?,
        synced_q: synced_q?,
        num_stages: stages,
    })
}

/// Internal: look up a node's width via the existing
/// `Node::width()` method (which exhaustively handles every
/// `Node` variant — PrimaryInput / Constant / FlopQ / MemRead /
/// FsmOut / InstanceOutput / Gate). Returns `None` only when
/// `node_id` is out of bounds.
fn node_width(module: &Module, node_id: NodeId) -> Option<u32> {
    module.nodes.get(node_id as usize).map(Node::width)
}

/// `MULTI-CLOCK-CDC.3b` — outcome of the per-module
/// `promote_to_multi_clock` pass. Reports what was added so the
/// caller can light coverage facts (`.3b.2`).
#[derive(Debug, Clone, Default)]
pub struct PromotionOutcome {
    /// `true` iff the pass actually transformed the module into
    /// K≥2 (a second domain was added + at least one flop-driven
    /// output was wrapped). `false` when the module had no
    /// eligible promotion target (no flops, no 1-bit outputs
    /// directly driven by a flop).
    pub promoted: bool,
    /// Number of clock domains in the module after the pass.
    /// `0` when the pass declined; `2` after a successful
    /// first-cut promotion.
    pub num_domains: u32,
    /// Number of synchronizer chains constructed by this pass (`0`
    /// or `1` in the current MVP — exactly one flop-driven output is
    /// promoted per call).
    pub num_synchronizers: u32,
    /// Number of destination-domain flops in the synchronizer chain.
    /// `0` when the pass declines.
    pub synchronizer_stages: u32,
}

/// `MULTI-CLOCK-CDC.3b` — promote a single-clock module to K=2
/// multi-clock by:
///
/// 1. **Allocating two new ports** for the secondary domain
///    (`clk_b` + `rst_n_b`), registered as
///    `Direction::In` in the IR.
/// 2. **Pushing two `ClockDomain` entries** into
///    `Module.clock_domains`: domain 0 = the existing
///    `Module.clock`/`reset` (named `"a"`); domain 1 = the new
///    `clk_b`/`rst_n_b` (named `"b"`). Index 0 still maps to
///    the original single-domain semantics — every existing
///    flop stays in domain 0 by default
///    (`Module.flop_domains` is empty initially; the
///    `flop_domain` accessor returns 0 by default).
/// 3. **Picking the first 1-bit output port directly driven by
///    a flop's Q**. If none exists, the pass declines and
///    returns `PromotionOutcome { promoted: false, .. }` (no
///    transformation; module remains K=1 — backward-compatible
///    decline).
/// 4. **Constructing a synchronizer chain in domain 1** driven by
///    that source flop's Q. The default is 2 stages; callers can
///    request N stages through `promote_to_multi_clock_with_stages`.
/// 5. **Rewiring the chosen output's drive** to dereference the
///    synced Q instead of the source Q. The output is now
///    semantically a domain-1 signal — synchronised in B and
///    sampled at B's clock.
///
/// This is still one synchronizer per call, on a 1-bit flop-driven
/// output. Wider promotion (multiple synchronizers per module,
/// arbitrary cross-domain data paths, fsm/memory in non-default
/// domains) remains a follow-up. The pass is **rules-first**: the
/// synchronizer is constructed in place at the moment of the
/// rewrite decision; there is no post-pass filter
/// (`feedback_rules_first_generation.md`).
///
/// **Backward-compatible.** Callers must gate this on a
/// per-module `Bernoulli(multi_clock_prob)` roll;
/// `Generator::generate_design` already does so when
/// `cfg.multi_clock_prob > 0.0` (default `0.0` ⇒ pass never
/// runs ⇒ byte-identical emit). When the pass DOES run on a
/// module with no eligible promotion target, it returns
/// `promoted: false` and leaves the module untouched — also
/// backward-compatible for downstream consumers.
pub fn promote_to_multi_clock(module: &mut Module) -> PromotionOutcome {
    promote_to_multi_clock_with_stages(module, 2)
}

pub fn promote_to_multi_clock_with_stages(
    module: &mut Module,
    synchronizer_stages: u32,
) -> PromotionOutcome {
    let synchronizer_stages = synchronizer_stages.max(2);
    // Idempotency: already-promoted modules
    // (`clock_domains.len() >= 2`) are returned untouched. This
    // is the load-bearing guard that lets `Generator::generate_module`
    // and `Generator::generate_design` both invoke the pass
    // without double-promoting modules that flow through both
    // paths (the design-level pass would otherwise pick the
    // synced Q as the next source and add a redundant chain,
    // breaking byte-identicality of the second-pass result).
    if module.clock_domains.len() >= 2 {
        return PromotionOutcome::default();
    }

    // Precondition: must have an existing single-clock domain.
    // Modules with no flops at all have no `clk`/`rst_n` and
    // can't be multi-clock-promoted.
    let (clk_a, rst_n_a) = match (module.clock, module.reset) {
        (Some(c), Some(r)) => (c, r),
        _ => return PromotionOutcome::default(),
    };

    // Find a 1-bit output port directly driven by a flop's Q.
    // (Multi-bit synchronization is out of scope per `.1`'s
    // tier-3-5 deferral — handshake / async FIFO / gray-code
    // pointer; the first-cut MVP only supports 1-bit signals.)
    let promotion_target = module.outputs.iter().find_map(|port| {
        if port.width != 1 {
            return None;
        }
        let (drive_idx, drive_node) =
            module.drives.iter().enumerate().find_map(|(idx, (p, n))| {
                if *p == port.id {
                    Some((idx, *n))
                } else {
                    None
                }
            })?;
        // The drive node must be a `Node::FlopQ` (the source
        // flop's output) — the first-cut MVP requires a
        // direct flop-driven output so the rewire is a
        // simple `drives[i].1 = synced_q`. Comb-driven
        // outputs would need cone rewriting which is a
        // follow-up.
        match &module.nodes[drive_node as usize] {
            Node::FlopQ { .. } => Some((port.id, drive_idx, drive_node)),
            _ => None,
        }
    });

    let (_output_port_id, drive_idx, src_q) = match promotion_target {
        Some(t) => t,
        None => return PromotionOutcome::default(),
    };

    // Allocate two new ports for domain B. PortId ids must be
    // unique — pick the next two unused ids by scanning.
    let next_port_id: PortId = module
        .inputs
        .iter()
        .chain(module.outputs.iter())
        .map(|p| p.id)
        .max()
        .map(|m| m + 1)
        .unwrap_or(0);
    let clk_b_id = next_port_id;
    let rst_n_b_id = next_port_id + 1;
    module.inputs.push(Port {
        id: clk_b_id,
        name: "clk_b".to_string(),
        width: 1,
        dir: Direction::In,
    });
    module.inputs.push(Port {
        id: rst_n_b_id,
        name: "rst_n_b".to_string(),
        width: 1,
        dir: Direction::In,
    });

    // Push the two ClockDomain entries. Domain 0 = the existing
    // single-clock (named "a"); domain 1 = the new (named "b").
    // Existing flops stay in domain 0 by default
    // (`flop_domains` empty ⇒ `flop_domain` returns 0).
    module.clock_domains.push(ClockDomain {
        clk: clk_a,
        rst_n: rst_n_a,
        name: "a".to_string(),
    });
    module.clock_domains.push(ClockDomain {
        clk: clk_b_id,
        rst_n: rst_n_b_id,
        name: "b".to_string(),
    });

    // Construct the synchronizer chain in domain 1.
    let chain = match construct_nflop_synchronizer(module, src_q, 1, synchronizer_stages) {
        Some(c) => c,
        None => return PromotionOutcome::default(),
    };

    // Rewire the chosen output's drive to the synced Q.
    module.drives[drive_idx].1 = chain.synced_q;

    PromotionOutcome {
        promoted: true,
        num_domains: 2,
        num_synchronizers: 1,
        synchronizer_stages: chain.num_stages,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{ClockDomain, Direction, Port};

    fn port(id: u32, name: &str, width: u32, dir: Direction) -> Port {
        Port {
            id,
            name: name.into(),
            width,
            dir,
        }
    }

    /// Helper: build a minimal K=2 module with one source flop
    /// in domain 0 (driven by an input), ready for the
    /// synchronizer chain to be added in domain 1.
    fn two_domain_module_with_source_flop() -> (Module, NodeId) {
        let mut m = Module {
            name: "src".into(),
            ..Module::default()
        };
        m.inputs.push(port(0, "clk_a", 1, Direction::In));
        m.inputs.push(port(1, "rst_n_a", 1, Direction::In));
        m.inputs.push(port(2, "clk_b", 1, Direction::In));
        m.inputs.push(port(3, "rst_n_b", 1, Direction::In));
        m.inputs.push(port(4, "i_a", 1, Direction::In));
        m.clock_domains.push(ClockDomain {
            clk: 0,
            rst_n: 1,
            name: "a".into(),
        });
        m.clock_domains.push(ClockDomain {
            clk: 2,
            rst_n: 3,
            name: "b".into(),
        });
        // Source flop in domain 0: D = i_a, Q = node 1.
        m.nodes.push(Node::PrimaryInput { port: 4, width: 1 }); // node 0
        m.nodes.push(Node::FlopQ { flop: 0, width: 1 }); // node 1
        m.flops.push(Flop {
            id: 0,
            width: 1,
            d: Some(0),
            q: 1,
            reset_val: 0,
            reset_kind: ResetKind::Async,
            kind: FlopKind::ZeroDefault,
            mux: FlopMux::None,
        });
        m.flop_domains.insert(0, 0);
        (m, 1) // src_q = node 1
    }

    #[test]
    fn construct_2flop_synchronizer_allocates_two_flops_in_target_domain() {
        let (mut m, src_q) = two_domain_module_with_source_flop();
        let initial_flop_count = m.flops.len();
        let initial_node_count = m.nodes.len();
        let chain = construct_2flop_synchronizer(&mut m, src_q, 1)
            .expect("synchronizer construction should succeed");
        // Exactly two new flops + two new FlopQ nodes.
        assert_eq!(m.flops.len(), initial_flop_count + 2);
        assert_eq!(m.nodes.len(), initial_node_count + 2);
        // Both new flops land in domain 1 (B).
        assert_eq!(m.flop_domain(chain.first_flop), 1);
        assert_eq!(m.flop_domain(chain.second_flop), 1);
        // The chain's synced_q is the second flop's Q.
        let second_flop = &m.flops[chain.second_flop as usize];
        assert_eq!(chain.synced_q, second_flop.q);
        // Source flop unchanged in domain 0.
        assert_eq!(m.flop_domain(0), 0);
    }

    #[test]
    fn construct_2flop_synchronizer_chains_d_to_q() {
        let (mut m, src_q) = two_domain_module_with_source_flop();
        let chain = construct_2flop_synchronizer(&mut m, src_q, 1).expect("ok");
        let first_flop = &m.flops[chain.first_flop as usize];
        let second_flop = &m.flops[chain.second_flop as usize];
        // first_flop.D = src_q (the cross-domain signal).
        assert_eq!(first_flop.d, Some(src_q));
        // second_flop.D = first_flop.q (the intermediate
        // sync-flop output — this is what makes it a 2-flop
        // chain rather than two independent flops).
        assert_eq!(second_flop.d, Some(first_flop.q));
    }

    #[test]
    fn construct_nflop_synchronizer_allocates_requested_stage_count() {
        let (mut m, src_q) = two_domain_module_with_source_flop();
        let chain = construct_nflop_synchronizer(&mut m, src_q, 1, 3).expect("ok");

        assert_eq!(chain.num_stages, 3);
        assert_eq!(m.flops.len(), 4, "source flop + three sync stages");
        assert_eq!(m.flop_domain(chain.first_flop), 1);
        assert_eq!(m.flop_domain(chain.second_flop), 1);
        assert_eq!(m.flop_domain(3), 1);
        let third_flop = &m.flops[3];
        assert_eq!(third_flop.d, Some(m.flops[chain.second_flop as usize].q));
        assert_eq!(chain.synced_q, third_flop.q);
    }

    #[test]
    fn construct_2flop_synchronizer_inherits_source_width() {
        // Build a wider source flop (width = 4) — multi-bit
        // sync is generally discouraged in real CDC (the caller
        // is expected to enforce 1-bit only via the knob-side
        // rule) but the primitive must inherit width correctly.
        let mut m = Module {
            name: "wide".into(),
            ..Module::default()
        };
        m.inputs.push(port(0, "clk_a", 1, Direction::In));
        m.inputs.push(port(1, "rst_n_a", 1, Direction::In));
        m.inputs.push(port(2, "clk_b", 1, Direction::In));
        m.inputs.push(port(3, "rst_n_b", 1, Direction::In));
        m.inputs.push(port(4, "i_a", 4, Direction::In));
        m.clock_domains.push(ClockDomain {
            clk: 0,
            rst_n: 1,
            name: "a".into(),
        });
        m.clock_domains.push(ClockDomain {
            clk: 2,
            rst_n: 3,
            name: "b".into(),
        });
        m.nodes.push(Node::PrimaryInput { port: 4, width: 4 }); // node 0
        m.nodes.push(Node::FlopQ { flop: 0, width: 4 }); // node 1
        m.flops.push(Flop {
            id: 0,
            width: 4,
            d: Some(0),
            q: 1,
            reset_val: 0,
            reset_kind: ResetKind::Async,
            kind: FlopKind::ZeroDefault,
            mux: FlopMux::None,
        });
        m.flop_domains.insert(0, 0);

        let chain = construct_2flop_synchronizer(&mut m, 1, 1).expect("ok");
        assert_eq!(m.flops[chain.first_flop as usize].width, 4);
        assert_eq!(m.flops[chain.second_flop as usize].width, 4);
        // The new FlopQ nodes also carry width=4.
        if let Node::FlopQ { width, .. } = &m.nodes[chain.synced_q as usize] {
            assert_eq!(*width, 4);
        } else {
            panic!("synced_q node should be FlopQ");
        }
    }

    #[test]
    fn construct_2flop_synchronizer_returns_none_for_invalid_src_q() {
        let (mut m, _) = two_domain_module_with_source_flop();
        // Pick a NodeId beyond the table.
        let bogus = m.nodes.len() as NodeId + 99;
        assert!(construct_2flop_synchronizer(&mut m, bogus, 1).is_none());
    }

    #[test]
    fn synchronizer_emit_shape_in_two_domain_module() {
        // End-to-end: construct + emit. The B-domain `always_ff`
        // block must contain both sync flops; the A-domain block
        // must contain only the source. This is the
        // .2-emitter-meets-.3a-construction integration check.
        let (mut m, src_q) = two_domain_module_with_source_flop();
        // Add an output port driven by the synchronized signal.
        m.outputs.push(port(5, "o", 1, Direction::Out));
        let chain = construct_2flop_synchronizer(&mut m, src_q, 1).expect("ok");
        m.drives.push((5, chain.synced_q));
        let sv = crate::emit::to_sv(&m);
        // Two always_ff blocks (one per domain).
        let n_blocks = sv.matches("always_ff @(").count();
        assert_eq!(
            n_blocks, 2,
            "expected exactly 2 always_ff blocks; got {n_blocks}:\n{sv}"
        );
        // Domain A's block contains only flop 0 (source).
        assert!(sv.contains("always_ff @(posedge clk_a or negedge rst_n_a)"));
        // Domain B's block contains both sync flops (1 and 2).
        assert!(sv.contains("always_ff @(posedge clk_b or negedge rst_n_b)"));
        // The synced signal is the output driver.
        let flop_name_synced = format!("flop_{}", chain.second_flop);
        assert!(
            sv.contains(&format!("assign o = {flop_name_synced};")),
            "output should be driven by the second-stage synced Q:\n{sv}"
        );
    }

    // ===========================================================
    // MULTI-CLOCK-CDC.3b — cargo-portable proofs of the
    // `promote_to_multi_clock` pass.
    // ===========================================================

    /// Build a minimal single-clock module suitable for the
    /// promotion pass: one 1-bit input + one 1-bit output, one
    /// flop driving the output. K=1, clock_domains empty.
    fn single_clock_module_with_flop_driven_1bit_output() -> Module {
        let mut m = Module {
            name: "promo_target".into(),
            ..Module::default()
        };
        m.inputs.push(port(0, "clk", 1, Direction::In));
        m.inputs.push(port(1, "rst_n", 1, Direction::In));
        m.inputs.push(port(2, "i_a", 1, Direction::In));
        m.outputs.push(port(3, "o", 1, Direction::Out));
        m.clock = Some(0);
        m.reset = Some(1);
        m.nodes.push(Node::PrimaryInput { port: 2, width: 1 }); // node 0
        m.nodes.push(Node::FlopQ { flop: 0, width: 1 }); // node 1
        m.flops.push(Flop {
            id: 0,
            width: 1,
            d: Some(0),
            q: 1,
            reset_val: 0,
            reset_kind: ResetKind::Async,
            kind: FlopKind::ZeroDefault,
            mux: FlopMux::None,
        });
        m.drives.push((3, 1)); // o := flop_0.Q
        m
    }

    #[test]
    fn promote_to_multi_clock_adds_two_domains_one_synchronizer() {
        let mut m = single_clock_module_with_flop_driven_1bit_output();
        let outcome = promote_to_multi_clock(&mut m);
        assert!(
            outcome.promoted,
            "promotion should succeed on a flop-driven 1-bit output"
        );
        assert_eq!(outcome.num_domains, 2);
        assert_eq!(outcome.num_synchronizers, 1);
        // Two ClockDomain entries, named "a" and "b".
        assert_eq!(m.clock_domains.len(), 2);
        assert_eq!(m.clock_domains[0].name, "a");
        assert_eq!(m.clock_domains[1].name, "b");
        // Two new input ports: clk_b + rst_n_b.
        assert!(m.inputs.iter().any(|p| p.name == "clk_b"));
        assert!(m.inputs.iter().any(|p| p.name == "rst_n_b"));
        // Three flops total: source (id=0) + first sync (id=1) +
        // second sync (id=2).
        assert_eq!(m.flops.len(), 3);
        // Source flop stays in domain 0; sync flops in domain 1.
        assert_eq!(m.flop_domain(0), 0);
        assert_eq!(m.flop_domain(1), 1);
        assert_eq!(m.flop_domain(2), 1);
    }

    #[test]
    fn promote_to_multi_clock_rewires_output_to_synced_q() {
        let mut m = single_clock_module_with_flop_driven_1bit_output();
        let _ = promote_to_multi_clock(&mut m);
        // The output `o` (port id 3) should now be driven by
        // flop_2.Q (the second-stage sync flop) — not flop_0.Q
        // any more.
        let (_, drive_node) = m.drives.iter().find(|(p, _)| *p == 3).expect("o drive");
        match &m.nodes[*drive_node as usize] {
            Node::FlopQ { flop, .. } => {
                assert_eq!(*flop, 2, "drive should be flop_2.Q (second sync stage)")
            }
            other => panic!("expected FlopQ drive after promotion; got {other:?}"),
        }
    }

    #[test]
    fn promote_to_multi_clock_with_stages_uses_configured_stage_count() {
        let mut m = single_clock_module_with_flop_driven_1bit_output();
        let outcome = promote_to_multi_clock_with_stages(&mut m, 3);

        assert!(outcome.promoted);
        assert_eq!(outcome.num_synchronizers, 1);
        assert_eq!(outcome.synchronizer_stages, 3);
        assert_eq!(m.flops.len(), 4, "source flop + three sync stages");
        assert_eq!(m.flop_domain(1), 1);
        assert_eq!(m.flop_domain(2), 1);
        assert_eq!(m.flop_domain(3), 1);
        let (_, drive_node) = m.drives.iter().find(|(p, _)| *p == 3).expect("o drive");
        assert_eq!(*drive_node, m.flops[3].q);
        let metrics = crate::metrics::compute(&m);
        assert_eq!(metrics.num_cdc_2_flop_synchronizers, 0);
        assert_eq!(metrics.num_cdc_synchronizer_chains, 1);
        assert_eq!(metrics.max_cdc_synchronizer_stages, 3);
    }

    #[test]
    fn promote_to_multi_clock_emit_shape_has_two_always_ff_blocks() {
        let mut m = single_clock_module_with_flop_driven_1bit_output();
        let _ = promote_to_multi_clock(&mut m);
        let sv = crate::emit::to_sv(&m);
        let n_blocks = sv.matches("always_ff @(").count();
        assert_eq!(
            n_blocks, 2,
            "expected 2 always_ff blocks after promotion:\n{sv}"
        );
        assert!(
            sv.contains("always_ff @(posedge clk or negedge rst_n)"),
            "domain A block missing"
        );
        assert!(
            sv.contains("always_ff @(posedge clk_b or negedge rst_n_b)"),
            "domain B block missing"
        );
        // Output driven by the synced (second-stage) Q.
        assert!(
            sv.contains("assign o = flop_2;"),
            "output should be rewired to flop_2:\n{sv}"
        );
    }

    #[test]
    fn promote_to_multi_clock_declines_on_module_with_no_outputs() {
        // Module with a flop but no output → no promotion target.
        let mut m = single_clock_module_with_flop_driven_1bit_output();
        m.outputs.clear();
        m.drives.clear();
        let outcome = promote_to_multi_clock(&mut m);
        assert!(!outcome.promoted);
        assert_eq!(outcome.num_domains, 0);
        assert_eq!(outcome.num_synchronizers, 0);
        // Module is unchanged.
        assert!(m.clock_domains.is_empty());
        assert!(m.flop_domains.is_empty());
        assert_eq!(m.flops.len(), 1);
    }

    #[test]
    fn promote_to_multi_clock_declines_on_module_with_no_clock() {
        // Pure-combinational module — no clock/reset to promote
        // from.
        let mut m = Module {
            name: "comb".into(),
            ..Module::default()
        };
        m.inputs.push(port(0, "i", 1, Direction::In));
        m.outputs.push(port(1, "o", 1, Direction::Out));
        m.nodes.push(Node::PrimaryInput { port: 0, width: 1 });
        m.drives.push((1, 0));
        let outcome = promote_to_multi_clock(&mut m);
        assert!(!outcome.promoted, "comb module has no clock to promote");
        assert!(m.clock_domains.is_empty());
    }

    #[test]
    fn promote_to_multi_clock_declines_on_wide_output() {
        // Width=8 output driven by flop — the first-cut MVP
        // only supports 1-bit signals (multi-bit needs handshake
        // or async-FIFO per `.1`'s tier 3-5 deferral).
        let mut m = Module {
            name: "wide_out".into(),
            ..Module::default()
        };
        m.inputs.push(port(0, "clk", 1, Direction::In));
        m.inputs.push(port(1, "rst_n", 1, Direction::In));
        m.inputs.push(port(2, "i_a", 8, Direction::In));
        m.outputs.push(port(3, "o", 8, Direction::Out));
        m.clock = Some(0);
        m.reset = Some(1);
        m.nodes.push(Node::PrimaryInput { port: 2, width: 8 });
        m.nodes.push(Node::FlopQ { flop: 0, width: 8 });
        m.flops.push(Flop {
            id: 0,
            width: 8,
            d: Some(0),
            q: 1,
            reset_val: 0,
            reset_kind: ResetKind::Async,
            kind: FlopKind::ZeroDefault,
            mux: FlopMux::None,
        });
        m.drives.push((3, 1));
        let outcome = promote_to_multi_clock(&mut m);
        assert!(
            !outcome.promoted,
            "wide outputs should be declined per .1's tier-3-5 deferral"
        );
    }
}
