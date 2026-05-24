//! `MULTI-CLOCK-CDC.3a` — 2-flop synchronizer construction
//! primitive for cross-clock-domain signals.
//!
//! Per `MULTI-CLOCK-CDC.1`'s design (`DEVELOPMENT_NOTES.md`
//! "Multi-clock + CDC primitives design (2026-05-24,
//! MULTI-CLOCK-CDC.1)"): when ANVIL emits a flop in domain B
//! whose D-cone references a flop output in domain A, the cone
//! must dereference a **2-flop synchronizer in domain B**, not
//! the bare cross-domain Q. The synchronizer is two newly-minted
//! flops, both in `dst_domain`. This module exposes the
//! construction primitive in isolation; `.3b` will wire it into
//! the per-module generator's domain-crossing decision path.
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
//!   the synchronizer is two distinct flops; their Q nodes are
//!   real `NodeId`s in the Module's node table, participating
//!   in the existing identity discipline (Relaxed mode forces
//!   fresh nodes; NodeId mode CSEs would be a future
//!   consideration for synchronizer reuse — but the canonical
//!   rule is "one synchronizer per (src, dst_domain) pair", not
//!   structural CSE, so today's identity discipline already
//!   gives the right answer).

use crate::ir::{Flop, FlopId, FlopKind, FlopMux, Module, Node, NodeId, ResetKind};

/// Result of constructing a 2-flop synchronizer chain.
#[derive(Debug, Clone, Copy)]
pub struct SynchronizerChain {
    /// FlopId of the first synchronizer flop (D = src_q;
    /// captures the metastable transition).
    pub first_flop: FlopId,
    /// FlopId of the second synchronizer flop (D = first.Q;
    /// resolves the metastable transition).
    pub second_flop: FlopId,
    /// NodeId of the second flop's Q — the synchronized signal
    /// in `dst_domain`. This is what downstream-in-dst-domain
    /// cones should reference.
    pub synced_q: NodeId,
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
    // Look up the source's width — synchronizer flops inherit it.
    let width = node_width(module, src_q)?;

    // Allocate the first synchronizer flop. D = src_q.
    let first_flop_id = module.flops.len() as FlopId;
    let first_q_node = module.nodes.len() as NodeId;
    module.nodes.push(Node::FlopQ {
        flop: first_flop_id,
        width,
    });
    module.flops.push(Flop {
        id: first_flop_id,
        width,
        d: Some(src_q),
        q: first_q_node,
        reset_val: 0,
        reset_kind: ResetKind::Async,
        kind: FlopKind::ZeroDefault,
        mux: FlopMux::None,
    });
    module.flop_domains.insert(first_flop_id, dst_domain);

    // Allocate the second synchronizer flop. D = first_q_node.
    let second_flop_id = module.flops.len() as FlopId;
    let second_q_node = module.nodes.len() as NodeId;
    module.nodes.push(Node::FlopQ {
        flop: second_flop_id,
        width,
    });
    module.flops.push(Flop {
        id: second_flop_id,
        width,
        d: Some(first_q_node),
        q: second_q_node,
        reset_val: 0,
        reset_kind: ResetKind::Async,
        kind: FlopKind::ZeroDefault,
        mux: FlopMux::None,
    });
    module.flop_domains.insert(second_flop_id, dst_domain);

    Some(SynchronizerChain {
        first_flop: first_flop_id,
        second_flop: second_flop_id,
        synced_q: second_q_node,
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
}
