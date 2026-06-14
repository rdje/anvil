//! IR-building gate primitives (`CONE-DECOMPOSITION.4`).
//!
//! The low-level "make a node" constructors — constants, equality
//! constants, comparison gates, muxes, bit replication, and the
//! `And`/`Mul`/`Sub`/n-ary `Add`/`Mul` builders. Each interns its gate
//! into the `Module` and registers the result in the `SignalPool`,
//! threading dep-sets via `node_deps`. Extracted verbatim from `cone.rs`;
//! behaviour is unchanged.

use super::{is_comparison_op, node_deps, obvious_unsigned_compare_result};
use crate::gen::pool::SignalPool;
use crate::ir::{DepSet, GateOp, Module, NodeId};

pub(crate) fn make_constant(
    m: &mut Module,
    pool: &mut SignalPool,
    width: u32,
    value: u128,
) -> NodeId {
    let (node_id, is_new) = m.intern_constant(width, value);
    if is_new {
        pool.add(node_id, width, DepSet::new());
    }
    node_id
}

pub(crate) fn make_eq_const(
    m: &mut Module,
    pool: &mut SignalPool,
    operand: NodeId,
    operand_width: u32,
    value: u128,
) -> NodeId {
    let const_node = make_constant(m, pool, operand_width, value);
    build_comparison_gate(m, pool, GateOp::Eq, operand, const_node)
}

pub(crate) fn build_comparison_gate(
    m: &mut Module,
    pool: &mut SignalPool,
    op: GateOp,
    lhs: NodeId,
    rhs: NodeId,
) -> NodeId {
    debug_assert!(is_comparison_op(op));
    if let Some(value) = obvious_unsigned_compare_result(m, op, lhs, rhs) {
        return make_constant(m, pool, 1, value);
    }
    let deps = DepSet::union(&[&node_deps(m, lhs), &node_deps(m, rhs)]);
    let (node_id, is_new) = m.intern_gate(op, vec![lhs, rhs], 1, deps.clone());
    if is_new {
        pool.add(node_id, 1, deps);
    }
    node_id
}

pub(crate) fn make_mux(
    m: &mut Module,
    pool: &mut SignalPool,
    sel: NodeId,
    a: NodeId,
    b: NodeId,
    width: u32,
) -> NodeId {
    // 2-to-1 arm-duplication guard. At the default rate 0.0, `a == b`
    // produces the degenerate `(sel)?(x):(x) = x`; skip the mux layer
    // and return `x` directly. A rate of 1.0 permits the pathological
    // form unconditionally. (Probabilistic middle-ground values are
    // enforced upstream at arm-pick time; by the time we reach here
    // the caller has already decided whether it's OK for a == b, so
    // the only case we still reject at this layer is rate == 0.0.)
    // See `book/src/structural-rules.md` Rule 8 + Rule 22.
    if a == b && m.mux_arm_duplication_rate <= 0.0 {
        return a;
    }
    let deps = DepSet::union(&[&node_deps(m, sel), &node_deps(m, a), &node_deps(m, b)]);
    let (node_id, is_new) = m.intern_gate(GateOp::Mux, vec![sel, a, b], width, deps.clone());
    if is_new {
        pool.add(node_id, width, deps);
    }
    node_id
}

/// `{N{bit}}` — replicate a 1-bit signal to N bits via Concat. If N == 1,
/// returns the bit unchanged.
pub(crate) fn replicate_to_width(
    m: &mut Module,
    pool: &mut SignalPool,
    bit: NodeId,
    width: u32,
) -> NodeId {
    if width == 1 {
        return bit;
    }
    let bit_deps = node_deps(m, bit);
    let (node_id, is_new) = m.intern_gate(
        GateOp::Concat,
        vec![bit; width as usize],
        width,
        bit_deps.clone(),
    );
    if is_new {
        pool.add(node_id, width, bit_deps);
    }
    node_id
}

pub(crate) fn make_and(
    m: &mut Module,
    pool: &mut SignalPool,
    a: NodeId,
    b: NodeId,
    width: u32,
) -> NodeId {
    // Idempotent: `x & x = x`. Skip the And layer at the default
    // factorization level (operand-unique and above). This closes
    // the make_and escape path that the one-hot-mux mask assembly
    // can hit when `replicate_to_width(sel, 1) == arm.data` via
    // CSE. At level `cse` / `none`, pass through — the user opted
    // out of operand uniqueness.
    use crate::config::FactorizationLevel;
    if a == b && m.effective_factorization_level() >= FactorizationLevel::OperandUnique {
        return a;
    }
    let deps = DepSet::union(&[&node_deps(m, a), &node_deps(m, b)]);
    let (node_id, is_new) = m.intern_gate(GateOp::And, vec![a, b], width, deps.clone());
    if is_new {
        pool.add(node_id, width, deps);
    }
    node_id
}

pub(crate) fn make_mul(
    m: &mut Module,
    pool: &mut SignalPool,
    a: NodeId,
    b: NodeId,
    width: u32,
) -> NodeId {
    // Degeneracy guard mirroring `make_and`: `x * x = x²` is a
    // duplicate-operand Mul, forbidden at the default strict
    // `operand_duplication_rate = 0.0`. Can arise when a signal `a`
    // happens to share NodeId with a coefficient constant `b` by
    // CSE (both are the same-width, same-value literal). At
    // operand-unique and above, collapse the degenerate shape to
    // `a` alone — matches Rule 8 (`operand-multiset distinctness`).
    use crate::config::FactorizationLevel;
    if a == b
        && m.effective_factorization_level() >= FactorizationLevel::OperandUnique
        && m.operand_duplication_rate < 1.0
    {
        return a;
    }
    let deps = DepSet::union(&[&node_deps(m, a), &node_deps(m, b)]);
    let (node_id, is_new) = m.intern_gate(GateOp::Mul, vec![a, b], width, deps.clone());
    if is_new {
        pool.add(node_id, width, deps);
    }
    node_id
}

pub(crate) fn make_sub(
    m: &mut Module,
    pool: &mut SignalPool,
    a: NodeId,
    b: NodeId,
    width: u32,
) -> NodeId {
    // Degeneracy guard: `x - x = 0` is a base Rule 8 rejection
    // regardless of factorization level. When the caller picks
    // colliding operands (via CSE or fold-induced re-use), short-
    // circuit to a same-width zero constant rather than interning a
    // Sub that the IR validator would reject.
    if a == b {
        return make_constant(m, pool, width, 0);
    }
    let deps = DepSet::union(&[&node_deps(m, a), &node_deps(m, b)]);
    let (node_id, is_new) = m.intern_gate(GateOp::Sub, vec![a, b], width, deps.clone());
    if is_new {
        pool.add(node_id, width, deps);
    }
    node_id
}

/// N-arity Add with all operands at `width`. N must be >= 2.
pub(crate) fn make_nary_add(
    m: &mut Module,
    pool: &mut SignalPool,
    operands: &[NodeId],
    width: u32,
) -> NodeId {
    debug_assert!(operands.len() >= 2);
    let effective_operands: Vec<NodeId> = if m.operand_duplication_rate < 1.0 {
        let mut seen = std::collections::HashSet::new();
        operands
            .iter()
            .copied()
            .filter(|o| seen.insert(*o))
            .collect()
    } else {
        operands.to_vec()
    };
    if effective_operands.len() == 1 {
        return effective_operands[0];
    }
    let deps_vec: Vec<DepSet> = effective_operands
        .iter()
        .map(|id| node_deps(m, *id))
        .collect();
    let deps = DepSet::union(&deps_vec.iter().collect::<Vec<_>>());
    let (node_id, is_new) = m.intern_gate(GateOp::Add, effective_operands, width, deps.clone());
    if is_new {
        pool.add(node_id, width, deps);
    }
    node_id
}

/// N-arity Mul with all operands at `width`. N must be >= 2.
pub(crate) fn make_nary_mul(
    m: &mut Module,
    pool: &mut SignalPool,
    operands: &[NodeId],
    width: u32,
) -> NodeId {
    debug_assert!(operands.len() >= 2);
    let deps_vec: Vec<DepSet> = operands.iter().map(|id| node_deps(m, *id)).collect();
    let deps = DepSet::union(&deps_vec.iter().collect::<Vec<_>>());
    let (node_id, is_new) = m.intern_gate(GateOp::Mul, operands.to_vec(), width, deps.clone());
    if is_new {
        pool.add(node_id, width, deps);
    }
    node_id
}
