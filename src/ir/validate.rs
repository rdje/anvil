//! IR invariant checker. A development-time safety net — if this rejects
//! generator output in production, that's a generator bug to fix.

use super::types::*;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ValidateError {
    #[error("node {0} references undefined operand {1}")]
    UndefinedOperand(NodeId, NodeId),
    #[error("gate {0:?} operand width mismatch: expected {1}, got {2}")]
    OperandWidth(GateOp, u32, u32),
    #[error("output port {0} is driven {1} times (expected 1)")]
    DriveCount(PortId, usize),
    #[error("flop {0} has no D input set")]
    FlopMissingD(FlopId),
    #[error("output cone for port {0} has empty dep-set (trivially constant)")]
    TrivialOutput(PortId),
    #[error("node {0} width {1} disagrees with declared {2}")]
    WidthMismatch(NodeId, u32, u32),
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
        // PrimaryInput/FlopQ/Constant as direct output: constant would be
        // caught here too, but for Phase 1 constants-as-outputs are guarded
        // during generation, so we do not flag them here.
    }

    // 5. Gate operand widths agree with declared output width rules.
    // TODO(phase-1): enumerate GateOp rules and check.

    Ok(())
}
