//! IR invariant checker. A development-time safety net — if this rejects
//! generator output in production, that's a generator bug to fix.

use super::types::*;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ValidateError {
    #[error("node {0} references undefined operand {1}")]
    UndefinedOperand(NodeId, NodeId),
    #[error("output port {port} is driven by undefined node {node}")]
    UndefinedDriveRoot { port: PortId, node: NodeId },
    #[error("gate {op:?} at node {node}: expected {expected} operands, got {got}")]
    GateArity {
        node: NodeId,
        op: GateOp,
        expected: String,
        got: usize,
    },
    #[error(
        "gate {op:?} at node {node}: operand {operand_idx} width {got} != expected {expected}"
    )]
    GateOperandWidth {
        node: NodeId,
        op: GateOp,
        operand_idx: usize,
        expected: u32,
        got: u32,
    },
    #[error("gate {op:?} at node {node}: output width {got} != expected {expected}")]
    GateOutputWidth {
        node: NodeId,
        op: GateOp,
        expected: u32,
        got: u32,
    },
    #[error(
        "gate {op:?} at node {node}: operand {a_idx} and {b_idx} must have equal width (got {a_w} vs {b_w})"
    )]
    GateOperandsMustMatch {
        node: NodeId,
        op: GateOp,
        a_idx: usize,
        b_idx: usize,
        a_w: u32,
        b_w: u32,
    },
    #[error("output port {0} is driven {1} times (expected 1)")]
    DriveCount(PortId, usize),
    #[error("flop table slot {index} stores id {id} (expected {expected})")]
    FlopIdMismatch {
        index: usize,
        id: FlopId,
        expected: FlopId,
    },
    #[error("flop {0} has no D input set")]
    FlopMissingD(FlopId),
    #[error("flop {flop} field `{field}` references undefined node {node}")]
    UndefinedFlopNode {
        flop: FlopId,
        field: &'static str,
        node: NodeId,
    },
    #[error("flop {flop} q node {q} is not a FlopQ node")]
    FlopQNotNode { flop: FlopId, q: NodeId },
    #[error("flop {flop} q node {q} points back to flop {got}")]
    FlopQBackrefMismatch {
        flop: FlopId,
        q: NodeId,
        got: FlopId,
    },
    #[error("flop {flop} width {flop_width} != FlopQ node {q} width {q_width}")]
    FlopQWidthMismatch {
        flop: FlopId,
        q: NodeId,
        flop_width: u32,
        q_width: u32,
    },
    #[error("FlopQ node {node} references undefined flop {flop}")]
    DanglingFlopQ { node: NodeId, flop: FlopId },
    #[error(
        "FlopQ node {node} is not the canonical q for flop {flop} (expected node {expected_q})"
    )]
    NonCanonicalFlopQ {
        node: NodeId,
        flop: FlopId,
        expected_q: NodeId,
    },
    #[error("FlopQ node {node} width {q_width} != flop {flop} width {flop_width}")]
    FlopNodeWidthMismatch {
        node: NodeId,
        flop: FlopId,
        q_width: u32,
        flop_width: u32,
    },
    #[error("output cone for port {0} has empty dep-set (trivially constant)")]
    TrivialOutput(PortId),
}

pub fn validate(m: &Module) -> Result<(), ValidateError> {
    // 1. Every NodeId referenced by an operand exists.
    for (idx, node) in m.nodes.iter().enumerate() {
        if let Node::Gate { operands, .. } = node {
            for op_id in operands {
                if (*op_id as usize) >= m.nodes.len() {
                    return Err(ValidateError::UndefinedOperand(idx as NodeId, *op_id));
                }
            }
        }
    }

    // 2. Every drive root exists.
    for (port, node) in &m.drives {
        if !node_exists(m, *node) {
            return Err(ValidateError::UndefinedDriveRoot {
                port: *port,
                node: *node,
            });
        }
    }

    // 3. Every flop's structural references are self-consistent.
    for (index, flop) in m.flops.iter().enumerate() {
        let expected = index as FlopId;
        if flop.id != expected {
            return Err(ValidateError::FlopIdMismatch {
                index,
                id: flop.id,
                expected,
            });
        }
        let Some(d) = flop.d else {
            return Err(ValidateError::FlopMissingD(flop.id));
        };
        if !node_exists(m, d) {
            return Err(ValidateError::UndefinedFlopNode {
                flop: flop.id,
                field: "d",
                node: d,
            });
        }
        if !node_exists(m, flop.q) {
            return Err(ValidateError::UndefinedFlopNode {
                flop: flop.id,
                field: "q",
                node: flop.q,
            });
        }
        match &m.nodes[flop.q as usize] {
            Node::FlopQ {
                flop: backref,
                width,
            } => {
                if *backref != flop.id {
                    return Err(ValidateError::FlopQBackrefMismatch {
                        flop: flop.id,
                        q: flop.q,
                        got: *backref,
                    });
                }
                if *width != flop.width {
                    return Err(ValidateError::FlopQWidthMismatch {
                        flop: flop.id,
                        q: flop.q,
                        flop_width: flop.width,
                        q_width: *width,
                    });
                }
            }
            _ => {
                return Err(ValidateError::FlopQNotNode {
                    flop: flop.id,
                    q: flop.q,
                });
            }
        }
        validate_flop_mux_refs(flop, m)?;
    }

    // 4. Every FlopQ node points at a real flop and is canonical.
    for (node_id, node) in m.nodes.iter().enumerate() {
        let Node::FlopQ { flop, width } = node else {
            continue;
        };
        let node_id = node_id as NodeId;
        let Some(owner) = m.flops.get(*flop as usize) else {
            return Err(ValidateError::DanglingFlopQ {
                node: node_id,
                flop: *flop,
            });
        };
        if owner.width != *width {
            return Err(ValidateError::FlopNodeWidthMismatch {
                node: node_id,
                flop: *flop,
                q_width: *width,
                flop_width: owner.width,
            });
        }
        if owner.q != node_id {
            return Err(ValidateError::NonCanonicalFlopQ {
                node: node_id,
                flop: *flop,
                expected_q: owner.q,
            });
        }
    }

    // 5. Each output port is driven exactly once.
    for out in &m.outputs {
        let count = m.drives.iter().filter(|(p, _)| *p == out.id).count();
        if count != 1 {
            return Err(ValidateError::DriveCount(out.id, count));
        }
    }

    // 6. Cone roots have non-empty dep-sets.
    for (port_id, node_id) in &m.drives {
        let node = &m.nodes[*node_id as usize];
        if let Node::Gate { deps, .. } = node {
            if deps.is_empty() {
                return Err(ValidateError::TrivialOutput(*port_id));
            }
        }
    }

    // 7. Per-gate operand widths and arity.
    for (idx, node) in m.nodes.iter().enumerate() {
        if let Node::Gate {
            op,
            operands,
            width,
            ..
        } = node
        {
            check_gate_shape(idx as NodeId, *op, operands, *width, m)?;
        }
    }

    Ok(())
}

fn node_exists(m: &Module, id: NodeId) -> bool {
    (id as usize) < m.nodes.len()
}

fn validate_flop_mux_refs(flop: &Flop, m: &Module) -> Result<(), ValidateError> {
    match &flop.mux {
        FlopMux::None => {}
        FlopMux::OneHot(arms) => {
            for arm in arms {
                for (field, node) in [("mux.data", arm.data), ("mux.sel", arm.sel)] {
                    if !node_exists(m, node) {
                        return Err(ValidateError::UndefinedFlopNode {
                            flop: flop.id,
                            field,
                            node,
                        });
                    }
                }
            }
        }
        FlopMux::Encoded { sel, data } => {
            if !node_exists(m, *sel) {
                return Err(ValidateError::UndefinedFlopNode {
                    flop: flop.id,
                    field: "mux.sel",
                    node: *sel,
                });
            }
            for node in data {
                if !node_exists(m, *node) {
                    return Err(ValidateError::UndefinedFlopNode {
                        flop: flop.id,
                        field: "mux.data",
                        node: *node,
                    });
                }
            }
        }
    }
    Ok(())
}

fn check_gate_shape(
    id: NodeId,
    op: GateOp,
    operands: &[NodeId],
    out_w: u32,
    m: &Module,
) -> Result<(), ValidateError> {
    use GateOp::*;
    let w = |i: usize| m.nodes[operands[i] as usize].width();
    let arity_err = |expected: &str| ValidateError::GateArity {
        node: id,
        op,
        expected: expected.to_string(),
        got: operands.len(),
    };

    match op {
        // N-arity associative operators (N >= 2). Every operand's width
        // matches the output width. N = 2 recovers the classic binary form.
        // Sub is handled separately because subtraction is not associative.
        And | Or | Xor | Add | Mul => {
            if operands.len() < 2 {
                return Err(arity_err(">= 2"));
            }
            for i in 0..operands.len() {
                if w(i) != out_w {
                    return Err(ValidateError::GateOperandWidth {
                        node: id,
                        op,
                        operand_idx: i,
                        expected: out_w,
                        got: w(i),
                    });
                }
            }
        }
        // Sub is strictly 2-arity (not associative).
        Sub => {
            if operands.len() != 2 {
                return Err(arity_err("2"));
            }
            for i in 0..2 {
                if w(i) != out_w {
                    return Err(ValidateError::GateOperandWidth {
                        node: id,
                        op,
                        operand_idx: i,
                        expected: out_w,
                        got: w(i),
                    });
                }
            }
        }
        // Unary bitwise: 1 operand, width = out_w.
        Not => {
            if operands.len() != 1 {
                return Err(arity_err("1"));
            }
            if w(0) != out_w {
                return Err(ValidateError::GateOperandWidth {
                    node: id,
                    op,
                    operand_idx: 0,
                    expected: out_w,
                    got: w(0),
                });
            }
        }
        // Mux: [sel (1-bit), a (out_w), b (out_w)].
        Mux => {
            if operands.len() != 3 {
                return Err(arity_err("3"));
            }
            if w(0) != 1 {
                return Err(ValidateError::GateOperandWidth {
                    node: id,
                    op,
                    operand_idx: 0,
                    expected: 1,
                    got: w(0),
                });
            }
            for i in 1..3 {
                if w(i) != out_w {
                    return Err(ValidateError::GateOperandWidth {
                        node: id,
                        op,
                        operand_idx: i,
                        expected: out_w,
                        got: w(i),
                    });
                }
            }
        }
        // CaseMux: [sel (K bits), data_0, data_1, ...]. At least 2
        // data arms, each data arm width == out_w, and the number of
        // data arms must fit in the select domain.
        CaseMux => {
            let data_arms = operands.len().saturating_sub(1);
            if data_arms < 2 {
                return Err(arity_err("sel + >= 2 data arms"));
            }
            let sel_w = w(0);
            if sel_w < 1 {
                return Err(ValidateError::GateOperandWidth {
                    node: id,
                    op,
                    operand_idx: 0,
                    expected: 1,
                    got: sel_w,
                });
            }
            if sel_w < 32 {
                let max_arms = 1usize << sel_w;
                if data_arms > max_arms {
                    return Err(ValidateError::GateArity {
                        node: id,
                        op,
                        expected: format!("sel + 2..={} data arms", max_arms),
                        got: operands.len(),
                    });
                }
            }
            for i in 1..operands.len() {
                if w(i) != out_w {
                    return Err(ValidateError::GateOperandWidth {
                        node: id,
                        op,
                        operand_idx: i,
                        expected: out_w,
                        got: w(i),
                    });
                }
            }
        }
        // CasezMux: [sel (K bits), value_0, wild_0, data_0, ...].
        // At least 2 arms, each value/wild constant width == sel_w,
        // and each data arm width == out_w.
        CasezMux => {
            let tail = operands.len().saturating_sub(1);
            if tail < 6 || !tail.is_multiple_of(3) {
                return Err(arity_err("sel + >= 2 (value, wild, data) arms"));
            }
            let sel_w = w(0);
            if sel_w < 1 {
                return Err(ValidateError::GateOperandWidth {
                    node: id,
                    op,
                    operand_idx: 0,
                    expected: 1,
                    got: sel_w,
                });
            }
            for arm_base in (1..operands.len()).step_by(3) {
                for operand_idx in [arm_base, arm_base + 1] {
                    if w(operand_idx) != sel_w {
                        return Err(ValidateError::GateOperandWidth {
                            node: id,
                            op,
                            operand_idx,
                            expected: sel_w,
                            got: w(operand_idx),
                        });
                    }
                    if !matches!(
                        m.nodes[operands[operand_idx] as usize],
                        Node::Constant { .. }
                    ) {
                        return Err(ValidateError::GateArity {
                            node: id,
                            op,
                            expected: "constant pattern and wildcard operands".to_string(),
                            got: operands.len(),
                        });
                    }
                }
                let data_idx = arm_base + 2;
                if w(data_idx) != out_w {
                    return Err(ValidateError::GateOperandWidth {
                        node: id,
                        op,
                        operand_idx: data_idx,
                        expected: out_w,
                        got: w(data_idx),
                    });
                }
            }
        }
        ForFold {
            trip_count,
            chunk_width,
            ..
        } => {
            if operands.len() != 1 {
                return Err(arity_err("1"));
            }
            if trip_count < 2 {
                return Err(ValidateError::GateArity {
                    node: id,
                    op,
                    expected: "trip_count >= 2".to_string(),
                    got: operands.len(),
                });
            }
            if out_w != chunk_width {
                return Err(ValidateError::GateOutputWidth {
                    node: id,
                    op,
                    expected: chunk_width,
                    got: out_w,
                });
            }
            let expected_src_w = trip_count.saturating_mul(chunk_width);
            if w(0) != expected_src_w {
                return Err(ValidateError::GateOperandWidth {
                    node: id,
                    op,
                    operand_idx: 0,
                    expected: expected_src_w,
                    got: w(0),
                });
            }
        }
        // Comparisons: out_w == 1, operands equal width.
        Eq | Neq | Lt | Gt | Le | Ge => {
            if operands.len() != 2 {
                return Err(arity_err("2"));
            }
            if out_w != 1 {
                return Err(ValidateError::GateOutputWidth {
                    node: id,
                    op,
                    expected: 1,
                    got: out_w,
                });
            }
            if w(0) != w(1) {
                return Err(ValidateError::GateOperandsMustMatch {
                    node: id,
                    op,
                    a_idx: 0,
                    b_idx: 1,
                    a_w: w(0),
                    b_w: w(1),
                });
            }
        }
        // Reductions: out_w == 1, 1 operand of any width.
        RedAnd | RedOr | RedXor => {
            if operands.len() != 1 {
                return Err(arity_err("1"));
            }
            if out_w != 1 {
                return Err(ValidateError::GateOutputWidth {
                    node: id,
                    op,
                    expected: 1,
                    got: out_w,
                });
            }
        }
        // Shifts: [value (out_w), amount (any)].
        Shl | Shr => {
            if operands.len() != 2 {
                return Err(arity_err("2"));
            }
            if w(0) != out_w {
                return Err(ValidateError::GateOperandWidth {
                    node: id,
                    op,
                    operand_idx: 0,
                    expected: out_w,
                    got: w(0),
                });
            }
        }
        // Slice: out_w == hi-lo+1, source wider than hi.
        Slice { hi, lo } => {
            if operands.len() != 1 {
                return Err(arity_err("1"));
            }
            if hi < lo {
                return Err(ValidateError::GateOutputWidth {
                    node: id,
                    op,
                    expected: 0,
                    got: out_w,
                });
            }
            let expected = hi - lo + 1;
            if out_w != expected {
                return Err(ValidateError::GateOutputWidth {
                    node: id,
                    op,
                    expected,
                    got: out_w,
                });
            }
            if w(0) <= hi {
                return Err(ValidateError::GateOperandWidth {
                    node: id,
                    op,
                    operand_idx: 0,
                    expected: hi + 1,
                    got: w(0),
                });
            }
        }
        // Concat: variadic, out_w == sum of operand widths.
        Concat => {
            if operands.is_empty() {
                return Err(arity_err(">= 1"));
            }
            let sum: u32 = (0..operands.len()).map(w).sum();
            if sum != out_w {
                return Err(ValidateError::GateOutputWidth {
                    node: id,
                    op,
                    expected: sum,
                    got: out_w,
                });
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_module() -> Module {
        Module {
            name: "m".into(),
            ..Module::default()
        }
    }

    fn add_input(m: &mut Module, name: &str, width: u32) -> (PortId, NodeId) {
        let port_id = m.inputs.len() as PortId + m.outputs.len() as PortId;
        m.inputs.push(Port {
            id: port_id,
            name: name.into(),
            width,
            dir: Direction::In,
        });
        let node_id = m.nodes.len() as NodeId;
        m.nodes.push(Node::PrimaryInput {
            port: port_id,
            width,
        });
        (port_id, node_id)
    }

    fn add_output(m: &mut Module, name: &str, width: u32, driver: NodeId) {
        let port_id = (m.inputs.len() + m.outputs.len()) as PortId;
        m.outputs.push(Port {
            id: port_id,
            name: name.into(),
            width,
            dir: Direction::Out,
        });
        m.drives.push((port_id, driver));
    }

    fn add_and(m: &mut Module, a: NodeId, b: NodeId, width: u32, deps: DepSet) -> NodeId {
        let id = m.nodes.len() as NodeId;
        m.nodes.push(Node::Gate {
            op: GateOp::And,
            operands: vec![a, b],
            width,
            deps,
        });
        id
    }

    fn add_flop(m: &mut Module, width: u32, q: NodeId, d: Option<NodeId>) -> FlopId {
        let flop_id = m.flops.len() as FlopId;
        m.flops.push(Flop {
            id: flop_id,
            width,
            d,
            q,
            reset_val: 0,
            reset_kind: ResetKind::Async,
            kind: FlopKind::ZeroDefault,
            mux: FlopMux::None,
        });
        flop_id
    }

    #[test]
    fn accepts_minimal_valid_module() {
        let mut m = empty_module();
        let (p_a, n_a) = add_input(&mut m, "a", 4);
        let (p_b, n_b) = add_input(&mut m, "b", 4);
        let deps = DepSet::union(&[&DepSet::from_port(p_a), &DepSet::from_port(p_b)]);
        let n_and = add_and(&mut m, n_a, n_b, 4, deps);
        add_output(&mut m, "o", 4, n_and);
        validate(&m).expect("valid module must pass");
    }

    #[test]
    fn rejects_undefined_drive_root() {
        let mut m = empty_module();
        add_output(&mut m, "o", 4, 99);
        let err = validate(&m).expect_err("undefined drive root must be rejected");
        assert!(matches!(err, ValidateError::UndefinedDriveRoot { .. }));
    }

    #[test]
    fn rejects_flop_id_mismatch() {
        let mut m = empty_module();
        m.nodes.push(Node::FlopQ { flop: 0, width: 4 });
        let flop_id = add_flop(&mut m, 4, 0, Some(0));
        m.flops[flop_id as usize].id = 7;
        add_output(&mut m, "o", 4, 0);
        let err = validate(&m).expect_err("flop ids must stay dense and canonical");
        assert!(matches!(err, ValidateError::FlopIdMismatch { .. }));
    }

    #[test]
    fn rejects_flop_missing_d() {
        let mut m = empty_module();
        m.nodes.push(Node::FlopQ { flop: 0, width: 4 });
        add_flop(&mut m, 4, 0, None);
        add_output(&mut m, "o", 4, 0);
        let err = validate(&m).expect_err("flop without D must be rejected");
        assert!(matches!(err, ValidateError::FlopMissingD(..)));
    }

    #[test]
    fn rejects_flop_q_that_is_not_a_flopq_node() {
        let mut m = empty_module();
        m.nodes.push(Node::Constant { width: 4, value: 0 }); // node 0
        add_flop(&mut m, 4, 0, Some(0));
        add_output(&mut m, "o", 4, 0);
        let err = validate(&m).expect_err("flop q must point at a FlopQ node");
        assert!(matches!(err, ValidateError::FlopQNotNode { .. }));
    }

    #[test]
    fn rejects_flop_q_backref_mismatch() {
        let mut m = empty_module();
        m.nodes.push(Node::FlopQ { flop: 1, width: 4 });
        add_flop(&mut m, 4, 0, Some(0));
        add_output(&mut m, "o", 4, 0);
        let err = validate(&m).expect_err("flop q backref must match owner flop");
        assert!(matches!(err, ValidateError::FlopQBackrefMismatch { .. }));
    }

    #[test]
    fn rejects_flop_q_width_mismatch() {
        let mut m = empty_module();
        m.nodes.push(Node::FlopQ { flop: 0, width: 8 });
        add_flop(&mut m, 4, 0, Some(0));
        add_output(&mut m, "o", 4, 0);
        let err = validate(&m).expect_err("flop q width must match flop width");
        assert!(matches!(err, ValidateError::FlopQWidthMismatch { .. }));
    }

    #[test]
    fn rejects_noncanonical_flopq_node() {
        let mut m = empty_module();
        m.nodes.push(Node::FlopQ { flop: 0, width: 4 }); // canonical q: node 0
        add_flop(&mut m, 4, 0, Some(0));
        m.nodes.push(Node::FlopQ { flop: 0, width: 4 }); // stale duplicate q: node 1
        add_output(&mut m, "o", 4, 1);
        let err = validate(&m).expect_err("duplicate stale FlopQ must be rejected");
        assert!(matches!(err, ValidateError::NonCanonicalFlopQ { .. }));
    }

    #[test]
    fn rejects_flopq_node_width_mismatch() {
        let mut m = empty_module();
        m.nodes.push(Node::FlopQ { flop: 0, width: 4 }); // canonical q
        add_flop(&mut m, 4, 0, Some(0));
        m.nodes.push(Node::FlopQ { flop: 0, width: 8 }); // stale duplicate with wrong width
        add_output(&mut m, "o", 8, 1);
        let err = validate(&m).expect_err("FlopQ node width must match owning flop width");
        assert!(matches!(err, ValidateError::FlopNodeWidthMismatch { .. }));
    }

    #[test]
    fn rejects_dangling_flopq_node() {
        let mut m = empty_module();
        m.nodes.push(Node::FlopQ { flop: 9, width: 4 });
        add_output(&mut m, "o", 4, 0);
        let err = validate(&m).expect_err("FlopQ must reference a real flop");
        assert!(matches!(err, ValidateError::DanglingFlopQ { .. }));
    }

    #[test]
    fn rejects_flop_mux_reference_to_undefined_node() {
        let mut m = empty_module();
        m.nodes.push(Node::FlopQ { flop: 0, width: 4 }); // q
        m.nodes.push(Node::Constant { width: 1, value: 1 }); // valid sel
        let flop_id = add_flop(&mut m, 4, 0, Some(0));
        m.flops[flop_id as usize].mux = FlopMux::OneHot(vec![MuxArm { data: 99, sel: 1 }]);
        add_output(&mut m, "o", 4, 0);
        let err = validate(&m).expect_err("flop mux refs must point at live nodes");
        assert!(matches!(err, ValidateError::UndefinedFlopNode { .. }));
    }

    #[test]
    fn rejects_and_operand_width_mismatch() {
        let mut m = empty_module();
        let (p_a, n_a) = add_input(&mut m, "a", 4);
        let (p_b, n_b) = add_input(&mut m, "b", 8); // wrong width
        let deps = DepSet::union(&[&DepSet::from_port(p_a), &DepSet::from_port(p_b)]);
        let n_and = add_and(&mut m, n_a, n_b, 4, deps);
        add_output(&mut m, "o", 4, n_and);
        let err = validate(&m).expect_err("width mismatch must be rejected");
        assert!(matches!(err, ValidateError::GateOperandWidth { .. }));
    }

    #[test]
    fn rejects_mux_non_1bit_selector() {
        let mut m = empty_module();
        let (p_a, n_a) = add_input(&mut m, "a", 4);
        let (p_b, n_b) = add_input(&mut m, "b", 4);
        let (p_s, n_s) = add_input(&mut m, "s", 4); // wrong: should be 1-bit
        let deps = DepSet::union(&[
            &DepSet::from_port(p_a),
            &DepSet::from_port(p_b),
            &DepSet::from_port(p_s),
        ]);
        let mux_id = m.nodes.len() as NodeId;
        m.nodes.push(Node::Gate {
            op: GateOp::Mux,
            operands: vec![n_s, n_a, n_b],
            width: 4,
            deps,
        });
        add_output(&mut m, "o", 4, mux_id);
        let err = validate(&m).expect_err("non-1-bit mux selector must be rejected");
        assert!(matches!(err, ValidateError::GateOperandWidth { .. }));
    }

    #[test]
    fn accepts_case_mux_with_explicit_default_domain() {
        let mut m = empty_module();
        let (_p_sel, n_sel) = add_input(&mut m, "sel", 2);
        let (_p_a, n_a) = add_input(&mut m, "a", 8);
        let (_p_b, n_b) = add_input(&mut m, "b", 8);
        let (_p_c, n_c) = add_input(&mut m, "c", 8);
        let case = m.nodes.len() as NodeId;
        m.nodes.push(Node::Gate {
            op: GateOp::CaseMux,
            operands: vec![n_sel, n_a, n_b, n_c],
            width: 8,
            deps: DepSet::from_port(0),
        });
        add_output(&mut m, "o", 8, case);
        validate(&m).expect("valid case mux must pass");
    }

    #[test]
    fn accepts_casez_mux_with_constant_patterns() {
        let mut m = empty_module();
        let (_p_sel, n_sel) = add_input(&mut m, "sel", 3);
        let (_p_a, n_a) = add_input(&mut m, "a", 8);
        let (_p_b, n_b) = add_input(&mut m, "b", 8);
        let pat0 = m.nodes.len() as NodeId;
        m.nodes.push(Node::Constant {
            width: 3,
            value: 0b000,
        });
        let wild0 = m.nodes.len() as NodeId;
        m.nodes.push(Node::Constant {
            width: 3,
            value: 0b001,
        });
        let pat1 = m.nodes.len() as NodeId;
        m.nodes.push(Node::Constant {
            width: 3,
            value: 0b010,
        });
        let wild1 = m.nodes.len() as NodeId;
        m.nodes.push(Node::Constant {
            width: 3,
            value: 0b001,
        });
        let casez = m.nodes.len() as NodeId;
        m.nodes.push(Node::Gate {
            op: GateOp::CasezMux,
            operands: vec![n_sel, pat0, wild0, n_a, pat1, wild1, n_b],
            width: 8,
            deps: DepSet::from_port(0),
        });
        add_output(&mut m, "o", 8, casez);
        validate(&m).expect("valid casez mux must pass");
    }

    #[test]
    fn accepts_for_fold_with_packed_source() {
        let mut m = empty_module();
        let (_p_src, n_src) = add_input(&mut m, "src", 8);
        let for_fold = m.nodes.len() as NodeId;
        m.nodes.push(Node::Gate {
            op: GateOp::ForFold {
                kind: ForFoldKind::Xor,
                trip_count: 4,
                chunk_width: 2,
            },
            operands: vec![n_src],
            width: 2,
            deps: DepSet::from_port(0),
        });
        add_output(&mut m, "o", 2, for_fold);
        validate(&m).expect("valid for-fold block must pass");
    }

    #[test]
    fn rejects_eq_output_not_1bit() {
        let mut m = empty_module();
        let (p_a, n_a) = add_input(&mut m, "a", 4);
        let (p_b, n_b) = add_input(&mut m, "b", 4);
        let deps = DepSet::union(&[&DepSet::from_port(p_a), &DepSet::from_port(p_b)]);
        let eq_id = m.nodes.len() as NodeId;
        m.nodes.push(Node::Gate {
            op: GateOp::Eq,
            operands: vec![n_a, n_b],
            width: 4, // wrong: must be 1
            deps,
        });
        add_output(&mut m, "o", 4, eq_id);
        let err = validate(&m).expect_err("non-1-bit Eq output must be rejected");
        assert!(matches!(err, ValidateError::GateOutputWidth { .. }));
    }

    #[test]
    fn rejects_concat_sum_mismatch() {
        let mut m = empty_module();
        let (p_a, n_a) = add_input(&mut m, "a", 4);
        let (p_b, n_b) = add_input(&mut m, "b", 4);
        let deps = DepSet::union(&[&DepSet::from_port(p_a), &DepSet::from_port(p_b)]);
        let concat_id = m.nodes.len() as NodeId;
        m.nodes.push(Node::Gate {
            op: GateOp::Concat,
            operands: vec![n_a, n_b],
            width: 16, // wrong: sum is 8
            deps,
        });
        add_output(&mut m, "o", 16, concat_id);
        let err = validate(&m).expect_err("concat sum mismatch must be rejected");
        assert!(matches!(err, ValidateError::GateOutputWidth { .. }));
    }

    #[test]
    fn rejects_slice_out_of_bounds() {
        let mut m = empty_module();
        let (p_a, n_a) = add_input(&mut m, "a", 4);
        let slice_id = m.nodes.len() as NodeId;
        m.nodes.push(Node::Gate {
            op: GateOp::Slice { hi: 7, lo: 0 }, // source is only 4 bits wide
            operands: vec![n_a],
            width: 8,
            deps: DepSet::from_port(p_a),
        });
        add_output(&mut m, "o", 8, slice_id);
        let err = validate(&m).expect_err("out-of-bounds slice must be rejected");
        assert!(matches!(err, ValidateError::GateOperandWidth { .. }));
    }

    #[test]
    fn rejects_not_wrong_arity() {
        let mut m = empty_module();
        let (p_a, n_a) = add_input(&mut m, "a", 4);
        let (p_b, n_b) = add_input(&mut m, "b", 4);
        let deps = DepSet::union(&[&DepSet::from_port(p_a), &DepSet::from_port(p_b)]);
        let not_id = m.nodes.len() as NodeId;
        m.nodes.push(Node::Gate {
            op: GateOp::Not,
            operands: vec![n_a, n_b], // wrong: Not takes 1 operand
            width: 4,
            deps,
        });
        add_output(&mut m, "o", 4, not_id);
        let err = validate(&m).expect_err("wrong-arity Not must be rejected");
        assert!(matches!(err, ValidateError::GateArity { .. }));
    }

    #[test]
    fn accepts_nary_and_with_three_operands() {
        // N-arity associative op: 3-way And with all operands at width 4.
        let mut m = empty_module();
        let (pa, na) = add_input(&mut m, "a", 4);
        let (pb, nb) = add_input(&mut m, "b", 4);
        let (pc, nc) = add_input(&mut m, "c", 4);
        let deps = DepSet::union(&[
            &DepSet::from_port(pa),
            &DepSet::from_port(pb),
            &DepSet::from_port(pc),
        ]);
        let and_id = m.nodes.len() as NodeId;
        m.nodes.push(Node::Gate {
            op: GateOp::And,
            operands: vec![na, nb, nc],
            width: 4,
            deps,
        });
        add_output(&mut m, "o", 4, and_id);
        validate(&m).expect("3-way And must validate");
    }

    #[test]
    fn rejects_and_with_fewer_than_two_operands() {
        // And with a single operand is below the N >= 2 floor.
        let mut m = empty_module();
        let (pa, na) = add_input(&mut m, "a", 4);
        let and_id = m.nodes.len() as NodeId;
        m.nodes.push(Node::Gate {
            op: GateOp::And,
            operands: vec![na],
            width: 4,
            deps: DepSet::from_port(pa),
        });
        add_output(&mut m, "o", 4, and_id);
        let err = validate(&m).expect_err("1-op And must be rejected");
        assert!(matches!(err, ValidateError::GateArity { .. }));
    }

    #[test]
    fn rejects_nary_add_operand_width_mismatch() {
        // 4-way Add where one operand has wrong width.
        let mut m = empty_module();
        let (pa, na) = add_input(&mut m, "a", 8);
        let (pb, nb) = add_input(&mut m, "b", 8);
        let (pc, nc) = add_input(&mut m, "c", 4); // wrong
        let (pd, nd) = add_input(&mut m, "d", 8);
        let deps = DepSet::union(&[
            &DepSet::from_port(pa),
            &DepSet::from_port(pb),
            &DepSet::from_port(pc),
            &DepSet::from_port(pd),
        ]);
        let add_id = m.nodes.len() as NodeId;
        m.nodes.push(Node::Gate {
            op: GateOp::Add,
            operands: vec![na, nb, nc, nd],
            width: 8,
            deps,
        });
        add_output(&mut m, "o", 8, add_id);
        let err = validate(&m).expect_err("4-way Add with width mismatch must be rejected");
        assert!(matches!(
            err,
            ValidateError::GateOperandWidth { operand_idx: 2, .. }
        ));
    }

    #[test]
    fn accepts_concat_variadic_replicate() {
        // The adapter and flop-mux code builds N-copy Concats.
        let mut m = empty_module();
        let (p_a, n_a) = add_input(&mut m, "a", 1);
        let concat_id = m.nodes.len() as NodeId;
        m.nodes.push(Node::Gate {
            op: GateOp::Concat,
            operands: vec![n_a; 8],
            width: 8,
            deps: DepSet::from_port(p_a),
        });
        add_output(&mut m, "o", 8, concat_id);
        validate(&m).expect("N-copy Concat must validate");
    }
}
