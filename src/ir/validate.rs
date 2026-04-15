//! IR invariant checker. A development-time safety net — if this rejects
//! generator output in production, that's a generator bug to fix.

use super::types::*;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ValidateError {
    #[error("node {0} references undefined operand {1}")]
    UndefinedOperand(NodeId, NodeId),
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
    #[error("flop {0} has no D input set")]
    FlopMissingD(FlopId),
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

    // 2. Each output port is driven exactly once.
    for out in &m.outputs {
        let count = m.drives.iter().filter(|(p, _)| *p == out.id).count();
        if count != 1 {
            return Err(ValidateError::DriveCount(out.id, count));
        }
    }

    // 3. Every flop has a D input.
    for flop in &m.flops {
        if flop.d.is_none() {
            return Err(ValidateError::FlopMissingD(flop.id));
        }
    }

    // 4. Cone roots have non-empty dep-sets.
    for (port_id, node_id) in &m.drives {
        let node = &m.nodes[*node_id as usize];
        if let Node::Gate { deps, .. } = node {
            if deps.is_empty() {
                return Err(ValidateError::TrivialOutput(*port_id));
            }
        }
    }

    // 5. Per-gate operand widths and arity.
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
        // Bitwise + arithmetic: 2 operands, all width = out_w.
        And | Or | Xor | Add | Sub | Mul => {
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
