//! `SV-VERSION-TARGETING.3b.2` — post-construction annotation that marks
//! selected *proper low-bits* `Slice` gates for the IEEE-1800-2023
//! `union soft` overlay rendering (decision `0010` + the `.3b.1`
//! design-detail in `DEVELOPMENT_NOTES.md`).
//!
//! The *up-opt*: when the emission target permits IEEE 1800-2023
//! (`SvVersion::permits(Sv2023)`), a marked gate is rendered as an internal
//! heterogeneous-width `union soft` overlay
//!
//! ```systemverilog
//! union soft { logic [W-1:0] w; logic [SW-1:0] n; } <gate>__u;
//! assign <gate>__u.w = <src>;
//! assign <gate>      = <gate>__u.n;
//! ```
//!
//! instead of the plain `assign <gate> = <src>[hi:0];`. This is genuinely a
//! 2023 construct (heterogeneous-width packed-union members are legal only as
//! `union soft`, IEEE 1800-2023 §7.3.1) and **behaviour-preserving**:
//! packed-union members are LSB-aligned, so `<gate>__u.n == <src>[SW-1:0]`
//! (verified by the `.3a`/`.3b.1` `--binary` probes).
//!
//! **Down-gating.** The marker is consulted by the emitter *only* under
//! `SvVersion::permits(Sv2023)`; below 2023 a marked gate down-gates to the
//! plain `src[hi:0]` slice — the standard-validity guarantee.
//!
//! **Non-rolling annotation, rolled at the call site like every other knob.**
//! The per-gate decision is a seeded `gen_bool(prob)` here (reproducible;
//! never `thread_rng`). The generator guards the call on
//! `Config::soft_union_slice_prob > 0.0`, so the default (`0.0`) draws nothing
//! from the RNG and marks nothing ⇒ byte-identical stream + output. The
//! annotation is an emitter-surface marker only: the flat IR body, validators,
//! CSE keys and `canonical_module_signature` are all untouched.

use crate::ir::{GateOp, Module, Node, NodeId};
use rand::Rng;

/// True iff `node` is a *proper low-bits* slice the emitter can faithfully
/// render as a heterogeneous-width `union soft` overlay: a
/// `GateOp::Slice { hi, lo: 0 }` over a **non-constant**, multi-bit source
/// whose width strictly exceeds the slice width (so the union's two members
/// — `w` at the source width and `n` at the slice width — genuinely differ,
/// which is exactly the 2023 *soft* requirement; an equal-width member set is
/// a plain `union packed`, not version-distinctive). A constant source is
/// excluded because the emitter folds it to a literal.
fn slice_qualifies(m: &Module, node: &Node) -> bool {
    let Node::Gate {
        op: GateOp::Slice { hi, lo },
        operands,
        width,
        ..
    } = node
    else {
        return false;
    };
    if *lo != 0 {
        return false;
    }
    let Some(src) = operands.first() else {
        return false;
    };
    let Some(src_node) = m.nodes.get(*src as usize) else {
        return false;
    };
    if matches!(src_node, Node::Constant { .. }) {
        return false;
    }
    let src_width = src_node.width();
    // hi+1 == width (lo==0), and a real narrowing into a *narrower* member.
    *width >= 1 && *hi + 1 == *width && *width < src_width
}

/// Mark qualifying low-bits `Slice` gates for the `union soft` overlay by
/// rolling `prob` per qualifying gate on the seeded generator RNG. Returns
/// the number newly marked. Callers must gate on `prob > 0.0` so the default
/// path is byte-identical (draws nothing). Single-call per module (mirrors the
/// `aggregate_prob` call-site roll).
pub fn annotate_soft_union_slices(m: &mut Module, rng: &mut impl Rng, prob: f64) -> usize {
    // Scope: leave Phase 5 parameterized modules out (the param/up-opt
    // cross-product is out of scope; their emitted widths are symbolic). The
    // param/aggregate cross-product is excluded for the same reason.
    if m.param_env.is_some() {
        return 0;
    }
    let p = prob.clamp(0.0, 1.0);
    // Collect candidates first so the immutable scan over `m.nodes` does not
    // overlap the mutable insert into `m.soft_union_slice_gates`.
    let candidates: Vec<NodeId> = m
        .nodes
        .iter()
        .enumerate()
        .filter(|(_, n)| slice_qualifies(m, n))
        .map(|(i, _)| i as NodeId)
        .collect();
    let mut marked = 0usize;
    for id in candidates {
        if rng.gen_bool(p) && m.soft_union_slice_gates.insert(id) {
            marked += 1;
        }
    }
    marked
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{DepSet, Direction, GateOp, Module, Node, Port};
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    fn rng() -> ChaCha8Rng {
        ChaCha8Rng::seed_from_u64(0)
    }

    /// `a` (8-bit input) sliced to its low `sw` bits → node 1 is a proper
    /// low-bits slice over a non-constant source.
    fn module_with_low_bits_slice(sw: u32) -> Module {
        let mut m = Module {
            name: "m".into(),
            ..Module::default()
        };
        m.inputs.push(Port {
            id: 0,
            name: "a".into(),
            width: 8,
            dir: Direction::In,
        });
        m.nodes.push(Node::PrimaryInput { port: 0, width: 8 }); // id 0
        m.nodes.push(Node::Gate {
            op: GateOp::Slice { hi: sw - 1, lo: 0 },
            operands: vec![0],
            width: sw,
            deps: DepSet::new(),
        }); // id 1
        m
    }

    #[test]
    fn prob_one_marks_a_proper_low_bits_slice() {
        let mut m = module_with_low_bits_slice(4);
        let n = annotate_soft_union_slices(&mut m, &mut rng(), 1.0);
        assert_eq!(n, 1);
        assert!(m.soft_union_slice_gates.contains(&1));
    }

    #[test]
    fn prob_zero_marks_nothing_byte_identical() {
        let mut m = module_with_low_bits_slice(4);
        let n = annotate_soft_union_slices(&mut m, &mut rng(), 0.0);
        assert_eq!(n, 0);
        assert!(m.soft_union_slice_gates.is_empty());
    }

    #[test]
    fn full_width_slice_does_not_qualify() {
        // hi+1 == src_width (8) → no narrowing → no heterogeneous members.
        let mut m = module_with_low_bits_slice(8);
        let n = annotate_soft_union_slices(&mut m, &mut rng(), 1.0);
        assert_eq!(n, 0);
    }

    #[test]
    fn high_slice_lo_nonzero_does_not_qualify() {
        // a[7:4] — lo != 0, union members are LSB-aligned so this is not
        // faithfully an overlay.
        let mut m = Module {
            name: "m".into(),
            ..Module::default()
        };
        m.inputs.push(Port {
            id: 0,
            name: "a".into(),
            width: 8,
            dir: Direction::In,
        });
        m.nodes.push(Node::PrimaryInput { port: 0, width: 8 });
        m.nodes.push(Node::Gate {
            op: GateOp::Slice { hi: 7, lo: 4 },
            operands: vec![0],
            width: 4,
            deps: DepSet::new(),
        });
        let n = annotate_soft_union_slices(&mut m, &mut rng(), 1.0);
        assert_eq!(n, 0);
    }

    #[test]
    fn constant_source_slice_does_not_qualify() {
        // A constant source is folded to a literal by the emitter; never overlay.
        let mut m = Module {
            name: "m".into(),
            ..Module::default()
        };
        m.nodes.push(Node::Constant {
            value: 0xA5,
            width: 8,
        });
        m.nodes.push(Node::Gate {
            op: GateOp::Slice { hi: 3, lo: 0 },
            operands: vec![0],
            width: 4,
            deps: DepSet::new(),
        });
        let n = annotate_soft_union_slices(&mut m, &mut rng(), 1.0);
        assert_eq!(n, 0);
    }

    #[test]
    fn param_env_module_is_skipped() {
        use crate::ir::ParamEnv;
        let mut m = module_with_low_bits_slice(4);
        m.param_env = Some(ParamEnv {
            name: "W".into(),
            min: 2,
            max: 8,
            design_value: 8,
        });
        let n = annotate_soft_union_slices(&mut m, &mut rng(), 1.0);
        assert_eq!(n, 0, "parameterized modules are out of scope");
    }

    /// The end-to-end up-opt + down-gating proof through the real emitter:
    /// a marked low-bits slice renders the IEEE 1800-2023 `union soft` overlay
    /// only when the target permits 2023; below 2023 it down-gates to the plain
    /// `a[3:0]` slice, byte-identical to the unmarked emission.
    #[test]
    fn overlay_renders_only_at_2023_and_down_gates_below() {
        use crate::config::SvVersion;
        use crate::emit::to_sv_versioned;

        // y = a[3:0] — a proper low-bits slice over an 8-bit input.
        let build = || -> Module {
            let mut m = Module {
                name: "uo".into(),
                ..Module::default()
            };
            m.inputs.push(Port {
                id: 0,
                name: "a".into(),
                width: 8,
                dir: Direction::In,
            });
            m.outputs.push(Port {
                id: 1,
                name: "y".into(),
                width: 4,
                dir: Direction::Out,
            });
            m.nodes.push(Node::PrimaryInput { port: 0, width: 8 });
            m.nodes.push(Node::Gate {
                op: GateOp::Slice { hi: 3, lo: 0 },
                operands: vec![0],
                width: 4,
                deps: DepSet::new(),
            });
            m.drives.push((1, 1));
            m
        };

        // Unmarked: the plain slice is a 2012/2017/2023 common floor.
        let unmarked = build();
        let base = to_sv_versioned(&unmarked, SvVersion::Sv2012);
        assert!(!base.contains("union soft"));
        for v in [SvVersion::Sv2012, SvVersion::Sv2017, SvVersion::Sv2023] {
            assert_eq!(to_sv_versioned(&unmarked, v), base);
        }

        // Marked: consulted only under a 2023 target.
        let mut marked = build();
        marked.soft_union_slice_gates.insert(1);
        let at2012 = to_sv_versioned(&marked, SvVersion::Sv2012);
        let at2017 = to_sv_versioned(&marked, SvVersion::Sv2017);
        let at2023 = to_sv_versioned(&marked, SvVersion::Sv2023);

        // Down-gating below 2023 = exactly the plain slice.
        assert_eq!(at2012, base, "Sv2012 must down-gate the marked slice");
        assert_eq!(at2017, base, "Sv2017 must down-gate the marked slice");

        // Up-opt at 2023 = the `union soft` overlay (divergence).
        assert_ne!(at2023, base);
        assert!(
            at2023.contains("union soft"),
            "2023 must emit `union soft`:\n{at2023}"
        );
        assert!(
            at2023.contains("__u.w = "),
            "overlay drives the wide member"
        );
        assert!(at2023.contains("__u.n;"), "overlay reads the narrow member");
    }
}
