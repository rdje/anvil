//! Bounded semantic analysis over a `&Module` (`CONE-DECOMPOSITION.3`).
//!
//! The value-set / unsigned-bounds / exact-value proof machinery behind
//! constant folding, obvious-compare resolution, and the identity /
//! factorization proofs. Everything here is **pure analysis** over an
//! already-built `Module` — no `Generator`, no `SignalPool`, no RNG — so
//! the whole module reads top-to-bottom without construction concerns.
//! Extracted verbatim from `cone.rs`; behaviour is unchanged. Re-exported
//! from the cone root via `pub(crate) use semantic::*;`, so callers in the
//! cone root and in `crate::ir::compact`
//! (`obvious_unsigned_compare_result`, `prove_node_exact_value_from_bounds`)
//! keep their existing paths.

use super::node_deps;
use crate::ir::{DepSet, GateOp, Module, Node, NodeId};
use std::collections::HashMap;

pub(crate) fn width_mask(width: u32) -> u128 {
    if width >= 128 {
        u128::MAX
    } else {
        (1u128 << width) - 1
    }
}

pub(crate) fn exact_bound(bounds: (u128, u128)) -> Option<u128> {
    (bounds.0 == bounds.1).then_some(bounds.0)
}

pub(crate) fn casez_pattern_matches(
    sel: u128,
    pattern_value: u128,
    wildcard_mask: u128,
    width: u32,
) -> bool {
    let mask = width_mask(width);
    ((sel ^ pattern_value) & !wildcard_mask & mask) == 0
}

pub(crate) fn shift_interval_by_exact_addend(
    bounds: (u128, u128),
    addend: u128,
    width: u32,
) -> Option<(u128, u128)> {
    let mask = width_mask(width);
    let addend = addend & mask;
    let start = bounds.0.wrapping_add(addend) & mask;
    let end = bounds.1.wrapping_add(addend) & mask;
    (start <= end).then_some((start, end))
}

pub(crate) fn prove_node_exact_value(m: &Module, id: NodeId) -> Option<u128> {
    if can_enumerate_small_value_set(m, id) {
        let mut set_ctx = SmallValueSetContext::default();
        if let Some(values) = node_small_value_set(m, id, &mut set_ctx) {
            if let [value] = values.as_slice() {
                return Some(u128::from(*value));
            }
        }
    }

    let mut bound_memo = HashMap::new();
    exact_bound(node_unsigned_bounds(m, id, &mut bound_memo))
}

pub(crate) fn prove_node_exact_value_from_bounds(m: &Module, id: NodeId) -> Option<u128> {
    let mut bound_memo = HashMap::new();
    exact_bound(node_unsigned_bounds(m, id, &mut bound_memo))
}

pub(crate) fn obvious_unsigned_compare_from_bounds(
    op: GateOp,
    lhs: (u128, u128),
    rhs: (u128, u128),
) -> Option<u128> {
    let (lhs_min, lhs_max) = lhs;
    let (rhs_min, rhs_max) = rhs;
    match op {
        GateOp::Eq => {
            if lhs_max < rhs_min || rhs_max < lhs_min {
                Some(0)
            } else if lhs_min == lhs_max && lhs_min == rhs_min && rhs_min == rhs_max {
                Some(1)
            } else {
                None
            }
        }
        GateOp::Neq => {
            if lhs_max < rhs_min || rhs_max < lhs_min {
                Some(1)
            } else if lhs_min == lhs_max && lhs_min == rhs_min && rhs_min == rhs_max {
                Some(0)
            } else {
                None
            }
        }
        GateOp::Lt => {
            if lhs_max < rhs_min {
                Some(1)
            } else if lhs_min >= rhs_max {
                Some(0)
            } else {
                None
            }
        }
        GateOp::Le => {
            if lhs_max <= rhs_min {
                Some(1)
            } else if lhs_min > rhs_max {
                Some(0)
            } else {
                None
            }
        }
        GateOp::Gt => {
            if lhs_min > rhs_max {
                Some(1)
            } else if lhs_max <= rhs_min {
                Some(0)
            } else {
                None
            }
        }
        GateOp::Ge => {
            if lhs_min >= rhs_max {
                Some(1)
            } else if lhs_max < rhs_min {
                Some(0)
            } else {
                None
            }
        }
        _ => None,
    }
}

pub(crate) fn exact_gate_value(
    m: &Module,
    op: GateOp,
    operands: &[NodeId],
    width: u32,
    memo: &mut HashMap<NodeId, (u128, u128)>,
) -> Option<u128> {
    let exact_operand = |memo: &mut HashMap<NodeId, (u128, u128)>, id: NodeId| {
        exact_bound(node_unsigned_bounds(m, id, memo))
    };

    match op {
        GateOp::And => {
            let mut acc = width_mask(width);
            let mut saw_unknown = false;
            for &operand in operands {
                match exact_operand(memo, operand) {
                    Some(value) => {
                        acc &= value;
                        if acc == 0 {
                            return Some(0);
                        }
                    }
                    None => saw_unknown = true,
                }
            }
            (!saw_unknown).then_some(acc & width_mask(width))
        }
        GateOp::Or => {
            let mut acc = 0;
            let mut saw_unknown = false;
            for &operand in operands {
                match exact_operand(memo, operand) {
                    Some(value) => {
                        acc |= value;
                        if acc == width_mask(width) {
                            return Some(width_mask(width));
                        }
                    }
                    None => saw_unknown = true,
                }
            }
            (!saw_unknown).then_some(acc & width_mask(width))
        }
        GateOp::Xor => {
            let mut acc = 0;
            for &operand in operands {
                acc ^= exact_operand(memo, operand)?;
            }
            Some(acc & width_mask(width))
        }
        GateOp::Not if operands.len() == 1 => {
            Some((!exact_operand(memo, operands[0])?) & width_mask(width))
        }
        GateOp::Add => {
            let mut acc = 0u128;
            for &operand in operands {
                acc = acc.wrapping_add(exact_operand(memo, operand)?);
            }
            Some(acc & width_mask(width))
        }
        GateOp::Sub if operands.len() == 2 => {
            if operands[0] == operands[1] {
                return Some(0);
            }
            let lhs = exact_operand(memo, operands[0])?;
            let rhs = exact_operand(memo, operands[1])?;
            Some(lhs.wrapping_sub(rhs) & width_mask(width))
        }
        GateOp::Mul => {
            let mut acc = 1u128;
            let mut saw_unknown = false;
            for &operand in operands {
                match exact_operand(memo, operand) {
                    Some(value) => {
                        acc = acc.wrapping_mul(value) & width_mask(width);
                        if acc == 0 {
                            return Some(0);
                        }
                    }
                    None => saw_unknown = true,
                }
            }
            (!saw_unknown).then_some(acc & width_mask(width))
        }
        GateOp::Eq | GateOp::Neq | GateOp::Lt | GateOp::Gt | GateOp::Le | GateOp::Ge
            if operands.len() == 2 =>
        {
            let lhs = exact_operand(memo, operands[0])?;
            let rhs = exact_operand(memo, operands[1])?;
            let result = match op {
                GateOp::Eq => lhs == rhs,
                GateOp::Neq => lhs != rhs,
                GateOp::Lt => lhs < rhs,
                GateOp::Gt => lhs > rhs,
                GateOp::Le => lhs <= rhs,
                GateOp::Ge => lhs >= rhs,
                _ => unreachable!(),
            };
            Some(u128::from(result))
        }
        GateOp::Mux if operands.len() == 3 => {
            let sel = exact_operand(memo, operands[0])?;
            let branch = if sel == 0 { operands[2] } else { operands[1] };
            exact_operand(memo, branch)
        }
        GateOp::CaseMux if operands.len() >= 3 => {
            let sel = exact_operand(memo, operands[0])?;
            let arm_idx = usize::try_from(sel).ok();
            match arm_idx.and_then(|idx| operands.get(idx + 1)) {
                Some(&branch) => exact_operand(memo, branch),
                None => Some(0),
            }
        }
        GateOp::CasezMux if operands.len() >= 7 && (operands.len() - 1).is_multiple_of(3) => {
            let sel = exact_operand(memo, operands[0])?;
            let sel_width = m.nodes[operands[0] as usize].width();
            for arm in operands[1..].chunks_exact(3) {
                let pattern_value = exact_operand(memo, arm[0])?;
                let wildcard_mask = exact_operand(memo, arm[1])?;
                if casez_pattern_matches(sel, pattern_value, wildcard_mask, sel_width) {
                    return exact_operand(memo, arm[2]);
                }
            }
            Some(0)
        }
        GateOp::Slice { lo, .. } if operands.len() == 1 => {
            let src = exact_operand(memo, operands[0])?;
            Some((src >> lo) & width_mask(width))
        }
        GateOp::Concat => {
            let mut out = 0u128;
            for &operand in operands {
                let operand_width = m.nodes[operand as usize].width();
                let operand_value = exact_operand(memo, operand)?;
                if operand_width >= 128 {
                    out = operand_value;
                } else {
                    out = (out << operand_width) | (operand_value & width_mask(operand_width));
                }
            }
            Some(out & width_mask(width))
        }
        GateOp::RedAnd | GateOp::RedOr | GateOp::RedXor if operands.len() == 1 => {
            let operand = operands[0];
            let operand_width = m.nodes[operand as usize].width();
            let value = exact_operand(memo, operand)?;
            let result = match op {
                GateOp::RedAnd => value == width_mask(operand_width),
                GateOp::RedOr => value != 0,
                GateOp::RedXor => value.count_ones() % 2 == 1,
                _ => unreachable!(),
            };
            Some(u128::from(result))
        }
        GateOp::Shl | GateOp::Shr if operands.len() == 2 => {
            let lhs = exact_operand(memo, operands[0])?;
            let rhs = exact_operand(memo, operands[1])?;
            let src_width = m.nodes[operands[0] as usize].width();
            if rhs >= u128::from(src_width) {
                return Some(0);
            }
            let amount = rhs as u32;
            let shifted = match op {
                GateOp::Shl => lhs.wrapping_shl(amount),
                GateOp::Shr => lhs >> amount,
                _ => unreachable!(),
            };
            Some(shifted & width_mask(width))
        }
        _ => None,
    }
}

pub(crate) fn collect_small_set(seen: &[bool; 256], width: u32) -> Vec<u16> {
    let domain = if width >= 8 { 256 } else { 1usize << width };
    let mut out = Vec::new();
    for (value, present) in seen.iter().enumerate().take(domain) {
        if *present {
            out.push(value as u16);
        }
    }
    out
}

const SMALL_VALUE_SET_WORK_BUDGET: usize = 200_000;
const SMALL_VALUE_SET_MAX_SUPPORT: usize = 3;
const TINY_VALUE_SET_WORK_BUDGET: usize = 512;
const TINY_VALUE_SET_RESULT_LIMIT: usize = 8;

#[derive(Clone)]
pub(crate) enum SmallValueSetMemoEntry {
    Known(Vec<u16>),
    Unknown,
}

#[derive(Clone)]
pub(crate) struct SmallValueSetContext {
    memo: HashMap<NodeId, SmallValueSetMemoEntry>,
    remaining_work: usize,
}

impl Default for SmallValueSetContext {
    fn default() -> Self {
        Self {
            memo: HashMap::new(),
            remaining_work: SMALL_VALUE_SET_WORK_BUDGET,
        }
    }
}

impl SmallValueSetContext {
    fn spend(&mut self, amount: usize) -> bool {
        if amount > self.remaining_work {
            return false;
        }
        self.remaining_work -= amount;
        true
    }
}

#[derive(Clone)]
pub(crate) enum TinyValueSetMemoEntry {
    Known(Vec<u16>),
    Unknown,
}

#[derive(Clone)]
pub(crate) struct TinyValueSetContext {
    memo: HashMap<NodeId, TinyValueSetMemoEntry>,
    remaining_work: usize,
}

impl Default for TinyValueSetContext {
    fn default() -> Self {
        Self {
            memo: HashMap::new(),
            remaining_work: TINY_VALUE_SET_WORK_BUDGET,
        }
    }
}

impl TinyValueSetContext {
    fn spend(&mut self, amount: usize) -> bool {
        if amount > self.remaining_work {
            return false;
        }
        self.remaining_work -= amount;
        true
    }
}

pub(crate) fn remember_small_value_set(
    ctx: &mut SmallValueSetContext,
    id: NodeId,
    values: Vec<u16>,
) -> Option<Vec<u16>> {
    ctx.memo
        .insert(id, SmallValueSetMemoEntry::Known(values.clone()));
    Some(values)
}

pub(crate) fn mark_small_value_set_unknown(
    ctx: &mut SmallValueSetContext,
    id: NodeId,
) -> Option<Vec<u16>> {
    ctx.memo.insert(id, SmallValueSetMemoEntry::Unknown);
    None
}

pub(crate) fn remember_tiny_value_set(
    ctx: &mut TinyValueSetContext,
    id: NodeId,
    mut values: Vec<u16>,
) -> Option<Vec<u16>> {
    values.sort_unstable();
    values.dedup();
    if values.len() > TINY_VALUE_SET_RESULT_LIMIT {
        ctx.memo.insert(id, TinyValueSetMemoEntry::Unknown);
        return None;
    }
    ctx.memo
        .insert(id, TinyValueSetMemoEntry::Known(values.clone()));
    Some(values)
}

pub(crate) fn mark_tiny_value_set_unknown(
    ctx: &mut TinyValueSetContext,
    id: NodeId,
) -> Option<Vec<u16>> {
    ctx.memo.insert(id, TinyValueSetMemoEntry::Unknown);
    None
}

pub(crate) fn fold_small_binary_sets<F>(
    ctx: &mut SmallValueSetContext,
    lhs: &[u16],
    rhs: &[u16],
    width: u32,
    mut f: F,
) -> Option<Vec<u16>>
where
    F: FnMut(u16, u16) -> u16,
{
    let work = lhs.len().saturating_mul(rhs.len()).max(1);
    if !ctx.spend(work) {
        return None;
    }
    let mut seen = [false; 256];
    for &a in lhs {
        for &b in rhs {
            seen[f(a, b) as usize] = true;
        }
    }
    Some(collect_small_set(&seen, width))
}

pub(crate) fn node_small_value_set(
    m: &Module,
    id: NodeId,
    ctx: &mut SmallValueSetContext,
) -> Option<Vec<u16>> {
    if let Some(entry) = ctx.memo.get(&id) {
        return match entry {
            SmallValueSetMemoEntry::Known(values) => Some(values.clone()),
            SmallValueSetMemoEntry::Unknown => None,
        };
    }

    let width = m.nodes[id as usize].width();
    if !can_enumerate_small_value_set(m, id) {
        return mark_small_value_set_unknown(ctx, id);
    }
    if !ctx.spend(1) {
        return mark_small_value_set_unknown(ctx, id);
    }
    let mask = width_mask(width) as u16;

    let values = match &m.nodes[id as usize] {
        Node::PrimaryInput { .. }
        | Node::FlopQ { .. }
        | Node::MemRead { .. }
        | Node::FsmOut { .. } => (0..=mask).collect(),
        Node::Constant { value, .. } => vec![(*value & u128::from(mask)) as u16],
        Node::InstanceOutput { .. } => return mark_small_value_set_unknown(ctx, id),
        Node::Gate {
            op,
            operands,
            width,
            ..
        } => match *op {
            GateOp::And => {
                let mut exact_and = mask;
                let mut live = Vec::new();
                for &operand in operands {
                    let rhs = node_small_value_set(m, operand, ctx)?;
                    if rhs.len() == 1 {
                        exact_and &= rhs[0] & mask;
                        if exact_and == 0 {
                            return remember_small_value_set(ctx, id, vec![0]);
                        }
                    } else {
                        live.push(rhs);
                    }
                }

                if live.is_empty() {
                    vec![exact_and]
                } else {
                    let mut acc = vec![exact_and];
                    for rhs in live {
                        acc = fold_small_binary_sets(ctx, &acc, &rhs, *width, |a, b| a & b)?;
                        if acc == [0] {
                            break;
                        }
                    }
                    acc
                }
            }
            GateOp::Or => {
                let mut exact_or = 0u16;
                let mut live = Vec::new();
                for &operand in operands {
                    let rhs = node_small_value_set(m, operand, ctx)?;
                    if rhs.len() == 1 {
                        exact_or |= rhs[0] & mask;
                        if exact_or == mask {
                            return remember_small_value_set(ctx, id, vec![mask]);
                        }
                    } else {
                        live.push(rhs);
                    }
                }

                if live.is_empty() {
                    vec![exact_or]
                } else {
                    let mut acc = vec![exact_or];
                    for rhs in live {
                        acc = fold_small_binary_sets(ctx, &acc, &rhs, *width, |a, b| a | b)?;
                        if acc == [mask] {
                            break;
                        }
                    }
                    acc
                }
            }
            GateOp::Xor => {
                let mut exact_xor = 0u16;
                let mut live_parity = HashMap::<NodeId, bool>::new();
                let mut live_sets = HashMap::<NodeId, Vec<u16>>::new();
                for &operand in operands {
                    let rhs = node_small_value_set(m, operand, ctx)?;
                    if rhs.len() == 1 {
                        exact_xor ^= rhs[0] & mask;
                    } else {
                        let parity = live_parity.entry(operand).or_insert(false);
                        *parity = !*parity;
                        live_sets.entry(operand).or_insert(rhs);
                    }
                }

                let mut acc = vec![exact_xor & mask];
                for (operand, odd) in live_parity {
                    if !odd {
                        continue;
                    }
                    let rhs = live_sets.get(&operand)?;
                    acc = fold_small_binary_sets(ctx, &acc, rhs, *width, |a, b| (a ^ b) & mask)?;
                }
                acc
            }
            GateOp::Not if operands.len() == 1 => {
                let src = node_small_value_set(m, operands[0], ctx)?;
                if !ctx.spend(src.len().max(1)) {
                    return mark_small_value_set_unknown(ctx, id);
                }
                let mut seen = [false; 256];
                for value in src {
                    seen[((!value) & mask) as usize] = true;
                }
                collect_small_set(&seen, *width)
            }
            GateOp::Add => {
                let mut iter = operands.iter();
                let first = node_small_value_set(m, *iter.next()?, ctx)?;
                iter.try_fold(first, |acc, operand| {
                    let rhs = node_small_value_set(m, *operand, ctx)?;
                    fold_small_binary_sets(ctx, &acc, &rhs, *width, |a, b| a.wrapping_add(b) & mask)
                })?
            }
            GateOp::Sub if operands.len() == 2 => {
                let lhs = node_small_value_set(m, operands[0], ctx)?;
                let rhs = node_small_value_set(m, operands[1], ctx)?;
                fold_small_binary_sets(ctx, &lhs, &rhs, *width, |a, b| a.wrapping_sub(b) & mask)?
            }
            GateOp::Mul => {
                let mut exact_mul = 1u16;
                let mut live = Vec::new();
                for &operand in operands {
                    let rhs = node_small_value_set(m, operand, ctx)?;
                    if rhs.len() == 1 {
                        exact_mul = exact_mul.wrapping_mul(rhs[0]) & mask;
                        if exact_mul == 0 {
                            return remember_small_value_set(ctx, id, vec![0]);
                        }
                    } else {
                        live.push(rhs);
                    }
                }

                if live.is_empty() {
                    vec![exact_mul]
                } else {
                    let mut acc = vec![exact_mul];
                    for rhs in live {
                        acc = fold_small_binary_sets(ctx, &acc, &rhs, *width, |a, b| {
                            a.wrapping_mul(b) & mask
                        })?;
                        if acc == [0] {
                            break;
                        }
                    }
                    acc
                }
            }
            GateOp::Eq | GateOp::Neq | GateOp::Lt | GateOp::Gt | GateOp::Le | GateOp::Ge
                if operands.len() == 2 =>
            {
                let lhs = node_small_value_set(m, operands[0], ctx)?;
                let rhs = node_small_value_set(m, operands[1], ctx)?;
                fold_small_binary_sets(ctx, &lhs, &rhs, *width, |a, b| {
                    let result = match *op {
                        GateOp::Eq => a == b,
                        GateOp::Neq => a != b,
                        GateOp::Lt => a < b,
                        GateOp::Gt => a > b,
                        GateOp::Le => a <= b,
                        GateOp::Ge => a >= b,
                        _ => unreachable!(),
                    };
                    u16::from(result)
                })?
            }
            GateOp::Mux if operands.len() == 3 => {
                let sel = node_small_value_set(m, operands[0], ctx)?;
                let on_true = node_small_value_set(m, operands[1], ctx)?;
                let on_false = node_small_value_set(m, operands[2], ctx)?;
                let work = sel
                    .len()
                    .saturating_add(on_true.len())
                    .saturating_add(on_false.len())
                    .max(1);
                if !ctx.spend(work) {
                    return mark_small_value_set_unknown(ctx, id);
                }
                let mut seen = [false; 256];
                if sel.contains(&0) {
                    for &value in &on_false {
                        seen[value as usize] = true;
                    }
                }
                if sel.iter().any(|&v| v != 0) {
                    for &value in &on_true {
                        seen[value as usize] = true;
                    }
                }
                collect_small_set(&seen, *width)
            }
            GateOp::Slice { lo, .. } if operands.len() == 1 => {
                let src = match node_small_value_set(m, operands[0], ctx) {
                    Some(values) => values,
                    None => match prove_node_exact_value(m, operands[0]) {
                        Some(value) => vec![((value >> lo) & u128::from(mask)) as u16],
                        None => (0..=mask).collect(),
                    },
                };
                if !ctx.spend(src.len().max(1)) {
                    return mark_small_value_set_unknown(ctx, id);
                }
                let mut seen = [false; 256];
                for value in src {
                    seen[((value >> lo) & mask) as usize] = true;
                }
                collect_small_set(&seen, *width)
            }
            GateOp::Concat => {
                if !operands.is_empty() && operands.iter().all(|operand| *operand == operands[0]) {
                    let operand = operands[0];
                    let operand_width = m.nodes[operand as usize].width();
                    let src = node_small_value_set(m, operand, ctx)?;
                    let work = src.len().saturating_mul(operands.len()).max(1);
                    if !ctx.spend(work) {
                        return mark_small_value_set_unknown(ctx, id);
                    }
                    let mut seen = [false; 256];
                    for value in src {
                        let mut out = 0u16;
                        for _ in 0..operands.len() {
                            out = if operand_width >= 16 {
                                value & mask
                            } else {
                                (((out as u32) << operand_width) | u32::from(value)) as u16 & mask
                            };
                        }
                        seen[out as usize] = true;
                    }
                    collect_small_set(&seen, *width)
                } else {
                    let mut acc = vec![0u16];
                    for &operand in operands {
                        let operand_width = m.nodes[operand as usize].width();
                        let rhs = node_small_value_set(m, operand, ctx)?;
                        acc = fold_small_binary_sets(ctx, &acc, &rhs, *width, |a, b| {
                            if operand_width >= 16 {
                                b & mask
                            } else {
                                (((a as u32) << operand_width) | u32::from(b)) as u16 & mask
                            }
                        })?;
                    }
                    acc
                }
            }
            GateOp::RedAnd | GateOp::RedOr | GateOp::RedXor if operands.len() == 1 => {
                let src_width = m.nodes[operands[0] as usize].width();
                let all_ones = width_mask(src_width) as u16;
                let src = node_small_value_set(m, operands[0], ctx)?;
                if !ctx.spend(src.len().max(1)) {
                    return mark_small_value_set_unknown(ctx, id);
                }
                let mut seen = [false; 256];
                for value in src {
                    let result = match *op {
                        GateOp::RedAnd => value == all_ones,
                        GateOp::RedOr => value != 0,
                        GateOp::RedXor => value.count_ones() % 2 == 1,
                        _ => unreachable!(),
                    };
                    seen[usize::from(result)] = true;
                }
                collect_small_set(&seen, *width)
            }
            GateOp::Shl | GateOp::Shr if operands.len() == 2 => {
                let src_width = m.nodes[operands[0] as usize].width() as u16;
                let lhs = node_small_value_set(m, operands[0], ctx)?;
                let rhs = node_small_value_set(m, operands[1], ctx)?;
                fold_small_binary_sets(ctx, &lhs, &rhs, *width, |a, b| {
                    if b >= src_width {
                        0
                    } else {
                        match *op {
                            GateOp::Shl => a.wrapping_shl(u32::from(b)) & mask,
                            GateOp::Shr => a >> b,
                            _ => unreachable!(),
                        }
                    }
                })?
            }
            _ => return mark_small_value_set_unknown(ctx, id),
        },
    };

    remember_small_value_set(ctx, id, values)
}

pub(crate) fn fold_tiny_binary_sets<F>(
    ctx: &mut TinyValueSetContext,
    lhs: &[u16],
    rhs: &[u16],
    width: u32,
    mut f: F,
) -> Option<Vec<u16>>
where
    F: FnMut(u16, u16) -> u16,
{
    let work = lhs.len().saturating_mul(rhs.len()).max(1);
    if !ctx.spend(work) {
        return None;
    }

    let mut seen = [false; 256];
    for &a in lhs {
        for &b in rhs {
            seen[f(a, b) as usize] = true;
        }
    }

    let values = collect_small_set(&seen, width);
    (values.len() <= TINY_VALUE_SET_RESULT_LIMIT).then_some(values)
}

pub(crate) fn node_tiny_value_set(
    m: &Module,
    id: NodeId,
    ctx: &mut TinyValueSetContext,
) -> Option<Vec<u16>> {
    if let Some(entry) = ctx.memo.get(&id) {
        return match entry {
            TinyValueSetMemoEntry::Known(values) => Some(values.clone()),
            TinyValueSetMemoEntry::Unknown => None,
        };
    }

    let width = m.nodes[id as usize].width();
    if width > 8 || !ctx.spend(1) {
        return mark_tiny_value_set_unknown(ctx, id);
    }

    let mask = width_mask(width) as u16;
    let values = match &m.nodes[id as usize] {
        Node::PrimaryInput { width, .. }
        | Node::FlopQ { width, .. }
        | Node::MemRead { width, .. }
        | Node::FsmOut { width, .. } => {
            if *width == 1 {
                vec![0, 1]
            } else {
                return mark_tiny_value_set_unknown(ctx, id);
            }
        }
        Node::Constant { value, .. } => vec![(*value & u128::from(mask)) as u16],
        Node::InstanceOutput { .. } => return mark_tiny_value_set_unknown(ctx, id),
        Node::Gate {
            op,
            operands,
            width,
            ..
        } => {
            if *width == 1 {
                vec![0, 1]
            } else {
                match *op {
                    GateOp::Concat
                        if !operands.is_empty()
                            && operands.iter().all(|operand| *operand == operands[0])
                            && m.nodes[operands[0] as usize].width() == 1 =>
                    {
                        let src = node_tiny_value_set(m, operands[0], ctx)?;
                        if !ctx.spend(src.len().saturating_mul(operands.len()).max(1)) {
                            return mark_tiny_value_set_unknown(ctx, id);
                        }
                        let mut seen = [false; 256];
                        for value in src {
                            let mut out = 0u16;
                            for _ in 0..operands.len() {
                                out = ((out << 1) | (value & 1)) & mask;
                            }
                            seen[out as usize] = true;
                        }
                        let values = collect_small_set(&seen, *width);
                        if values.len() > TINY_VALUE_SET_RESULT_LIMIT {
                            return mark_tiny_value_set_unknown(ctx, id);
                        }
                        values
                    }
                    GateOp::Add => {
                        let mut iter = operands.iter();
                        let first = node_tiny_value_set(m, *iter.next()?, ctx)?;
                        iter.try_fold(first, |acc, operand| {
                            let rhs = node_tiny_value_set(m, *operand, ctx)?;
                            fold_tiny_binary_sets(ctx, &acc, &rhs, *width, |a, b| {
                                a.wrapping_add(b) & mask
                            })
                        })?
                    }
                    GateOp::Sub if operands.len() == 2 => {
                        let lhs = node_tiny_value_set(m, operands[0], ctx)?;
                        let rhs = node_tiny_value_set(m, operands[1], ctx)?;
                        fold_tiny_binary_sets(ctx, &lhs, &rhs, *width, |a, b| {
                            a.wrapping_sub(b) & mask
                        })?
                    }
                    _ => return mark_tiny_value_set_unknown(ctx, id),
                }
            }
        }
    };

    remember_tiny_value_set(ctx, id, values)
}

pub(crate) fn node_support_size(m: &Module, id: NodeId) -> usize {
    match &m.nodes[id as usize] {
        Node::PrimaryInput { .. }
        | Node::FlopQ { .. }
        | Node::MemRead { .. }
        | Node::FsmOut { .. } => 1,
        Node::Constant { .. } => 0,
        Node::InstanceOutput { .. } => SMALL_VALUE_SET_MAX_SUPPORT + 1,
        Node::Gate { deps, .. } => deps.len(),
    }
}

pub(crate) fn can_enumerate_small_value_set(m: &Module, id: NodeId) -> bool {
    m.nodes[id as usize].width() <= 8 && node_support_size(m, id) <= SMALL_VALUE_SET_MAX_SUPPORT
}

pub(crate) fn can_prove_compare_via_small_value_sets(m: &Module, lhs: NodeId, rhs: NodeId) -> bool {
    if !can_enumerate_small_value_set(m, lhs) || !can_enumerate_small_value_set(m, rhs) {
        return false;
    }

    let lhs_deps = node_deps(m, lhs);
    let rhs_deps = node_deps(m, rhs);
    DepSet::union(&[&lhs_deps, &rhs_deps]).len() <= SMALL_VALUE_SET_MAX_SUPPORT
}

pub(crate) fn small_value_set_min_at_least(m: &Module, id: NodeId, threshold: u128) -> bool {
    if can_enumerate_small_value_set(m, id) {
        let mut ctx = SmallValueSetContext::default();
        return node_small_value_set(m, id, &mut ctx)
            .map(|values| values.iter().all(|&value| u128::from(value) >= threshold))
            .unwrap_or(false);
    }

    let mut ctx = TinyValueSetContext::default();
    node_tiny_value_set(m, id, &mut ctx)
        .map(|values| values.iter().all(|&value| u128::from(value) >= threshold))
        .unwrap_or(false)
}

pub(crate) fn node_unsigned_bounds(
    m: &Module,
    id: NodeId,
    memo: &mut HashMap<NodeId, (u128, u128)>,
) -> (u128, u128) {
    if let Some(&bounds) = memo.get(&id) {
        return bounds;
    }

    let bounds = match &m.nodes[id as usize] {
        Node::PrimaryInput { width, .. }
        | Node::FlopQ { width, .. }
        | Node::MemRead { width, .. }
        | Node::FsmOut { width, .. } => (0, width_mask(*width)),
        Node::Constant { value, .. } => (*value, *value),
        Node::InstanceOutput { width, .. } => (0, width_mask(*width)),
        Node::Gate {
            op,
            operands,
            width,
            ..
        } => {
            if let Some(value) = exact_gate_value(m, *op, operands, *width, memo) {
                (value, value)
            } else {
                let default = (0, width_mask(*width));
                match *op {
                    GateOp::And => {
                        let all_ones = width_mask(*width);
                        let mut saw_zero = false;
                        let mut live = Vec::new();
                        for &operand in operands {
                            let bounds = node_unsigned_bounds(m, operand, memo);
                            match exact_bound(bounds) {
                                Some(0) => {
                                    saw_zero = true;
                                    break;
                                }
                                Some(v) if v == all_ones => {}
                                _ => live.push(bounds),
                            }
                        }
                        if saw_zero {
                            (0, 0)
                        } else if live.is_empty() {
                            (all_ones, all_ones)
                        } else if live.len() == 1 {
                            live[0]
                        } else {
                            let upper = live.iter().map(|(_, max)| *max).min().unwrap_or(all_ones);
                            (0, upper)
                        }
                    }
                    GateOp::Or => {
                        let all_ones = width_mask(*width);
                        let mut saw_all_ones = false;
                        let mut live = Vec::new();
                        for &operand in operands {
                            let bounds = node_unsigned_bounds(m, operand, memo);
                            match exact_bound(bounds) {
                                Some(v) if v == all_ones => {
                                    saw_all_ones = true;
                                    break;
                                }
                                Some(0) => {}
                                _ => live.push(bounds),
                            }
                        }
                        if saw_all_ones {
                            (all_ones, all_ones)
                        } else if live.is_empty() {
                            (0, 0)
                        } else if live.len() == 1 {
                            live[0]
                        } else {
                            let lower = live.iter().map(|(min, _)| *min).max().unwrap_or(0);
                            (lower, all_ones)
                        }
                    }
                    GateOp::Xor => {
                        let all_ones = width_mask(*width);
                        let mut exact_xor = 0u128;
                        let mut live_parity = HashMap::<NodeId, bool>::new();
                        let mut live_bounds = HashMap::<NodeId, (u128, u128)>::new();
                        for &operand in operands {
                            let bounds = node_unsigned_bounds(m, operand, memo);
                            if let Some(v) = exact_bound(bounds) {
                                exact_xor ^= v;
                            } else {
                                let parity = live_parity.entry(operand).or_insert(false);
                                *parity = !*parity;
                                live_bounds.entry(operand).or_insert(bounds);
                            }
                        }
                        let live: Vec<(u128, u128)> = live_parity
                            .into_iter()
                            .filter(|&(_, odd)| odd)
                            .map(|(operand, _)| live_bounds[&operand])
                            .collect();
                        if live.is_empty() {
                            (exact_xor & all_ones, exact_xor & all_ones)
                        } else if live.len() == 1 && exact_xor == 0 {
                            live[0]
                        } else if live.len() == 1 && exact_xor == all_ones {
                            let (src_min, src_max) = live[0];
                            (all_ones ^ src_max, all_ones ^ src_min)
                        } else {
                            default
                        }
                    }
                    GateOp::Not if operands.len() == 1 => {
                        let all_ones = width_mask(*width);
                        let (src_min, src_max) = node_unsigned_bounds(m, operands[0], memo);
                        (all_ones ^ src_max, all_ones ^ src_min)
                    }
                    GateOp::Add => {
                        let mask = width_mask(*width);
                        let mut exact_sum = 0u128;
                        let mut live = Vec::new();
                        for &operand in operands {
                            let bounds = node_unsigned_bounds(m, operand, memo);
                            match exact_bound(bounds) {
                                Some(0) => {}
                                Some(value) => exact_sum = exact_sum.wrapping_add(value) & mask,
                                None => live.push(bounds),
                            }
                        }
                        if live.is_empty() {
                            (exact_sum, exact_sum)
                        } else if live.len() == 1 {
                            if exact_sum == 0 {
                                live[0]
                            } else {
                                shift_interval_by_exact_addend(live[0], exact_sum, *width)
                                    .unwrap_or(default)
                            }
                        } else {
                            let mut min_sum = exact_sum;
                            let mut max_sum = exact_sum;
                            let mut overflow = false;
                            for (min, max) in live {
                                min_sum = min_sum.saturating_add(min);
                                max_sum = max_sum.saturating_add(max);
                                if min_sum > mask || max_sum > mask {
                                    overflow = true;
                                    break;
                                }
                            }
                            if overflow {
                                default
                            } else {
                                (min_sum, max_sum)
                            }
                        }
                    }
                    GateOp::Sub if operands.len() == 2 => {
                        if operands[0] == operands[1] {
                            (0, 0)
                        } else {
                            let lhs = node_unsigned_bounds(m, operands[0], memo);
                            let rhs = node_unsigned_bounds(m, operands[1], memo);
                            if exact_bound(rhs) == Some(0) {
                                lhs
                            } else if lhs.0 >= rhs.1 {
                                (lhs.0 - rhs.1, lhs.1 - rhs.0)
                            } else {
                                default
                            }
                        }
                    }
                    GateOp::Mul => {
                        let mut saw_zero = false;
                        let mut live = Vec::new();
                        for &operand in operands {
                            let bounds = node_unsigned_bounds(m, operand, memo);
                            match exact_bound(bounds) {
                                Some(0) => {
                                    saw_zero = true;
                                    break;
                                }
                                Some(1) => {}
                                _ => live.push(bounds),
                            }
                        }
                        if saw_zero {
                            (0, 0)
                        } else if live.is_empty() {
                            (1, 1)
                        } else if live.len() == 1 {
                            live[0]
                        } else {
                            let mut min_prod = 1u128;
                            let mut max_prod = 1u128;
                            let mut overflow = false;
                            for (min, max) in live {
                                min_prod = min_prod.saturating_mul(min);
                                max_prod = max_prod.saturating_mul(max);
                                if min_prod > width_mask(*width) || max_prod > width_mask(*width) {
                                    overflow = true;
                                    break;
                                }
                            }
                            if overflow {
                                default
                            } else {
                                (min_prod, max_prod)
                            }
                        }
                    }
                    GateOp::Eq
                    | GateOp::Neq
                    | GateOp::Lt
                    | GateOp::Gt
                    | GateOp::Le
                    | GateOp::Ge
                        if operands.len() == 2 =>
                    {
                        let lhs = node_unsigned_bounds(m, operands[0], memo);
                        let rhs = node_unsigned_bounds(m, operands[1], memo);
                        obvious_unsigned_compare_from_bounds(*op, lhs, rhs)
                            .map(|v| (v, v))
                            .unwrap_or((0, 1))
                    }
                    GateOp::RedAnd if operands.len() == 1 => {
                        let src = node_unsigned_bounds(m, operands[0], memo);
                        let all_ones = width_mask(m.nodes[operands[0] as usize].width());
                        if src.0 == all_ones {
                            (1, 1)
                        } else if src.1 < all_ones {
                            (0, 0)
                        } else {
                            (0, 1)
                        }
                    }
                    GateOp::RedOr if operands.len() == 1 => {
                        let src = node_unsigned_bounds(m, operands[0], memo);
                        if src.1 == 0 {
                            (0, 0)
                        } else if src.0 > 0 {
                            (1, 1)
                        } else {
                            (0, 1)
                        }
                    }
                    GateOp::RedXor => (0, 1),
                    GateOp::Mux if operands.len() == 3 => {
                        let sel = exact_bound(node_unsigned_bounds(m, operands[0], memo));
                        if let Some(sel) = sel {
                            let arm = if sel == 0 { operands[2] } else { operands[1] };
                            node_unsigned_bounds(m, arm, memo)
                        } else {
                            let on_true = node_unsigned_bounds(m, operands[1], memo);
                            let on_false = node_unsigned_bounds(m, operands[2], memo);
                            (on_true.0.min(on_false.0), on_true.1.max(on_false.1))
                        }
                    }
                    GateOp::CaseMux if operands.len() >= 3 => {
                        let sel_bounds = node_unsigned_bounds(m, operands[0], memo);
                        let data_arms = operands.len() - 1;
                        if let Some(sel) = exact_bound(sel_bounds) {
                            let arm_idx = usize::try_from(sel).ok();
                            match arm_idx.and_then(|idx| operands.get(idx + 1).copied()) {
                                Some(data) => node_unsigned_bounds(m, data, memo),
                                None => (0, 0),
                            }
                        } else {
                            let mut saw_value = false;
                            let mut min = u128::MAX;
                            let mut max = 0u128;
                            if sel_bounds.1 >= data_arms as u128 {
                                saw_value = true;
                                min = 0;
                                max = 0;
                            }
                            for (idx, &data) in operands[1..].iter().enumerate() {
                                let idx = idx as u128;
                                if idx < sel_bounds.0 || idx > sel_bounds.1 {
                                    continue;
                                }
                                let bounds = node_unsigned_bounds(m, data, memo);
                                saw_value = true;
                                min = min.min(bounds.0);
                                max = max.max(bounds.1);
                            }
                            if saw_value {
                                (min, max)
                            } else {
                                (0, 0)
                            }
                        }
                    }
                    GateOp::CasezMux
                        if operands.len() >= 7 && (operands.len() - 1).is_multiple_of(3) =>
                    {
                        let sel_bounds = node_unsigned_bounds(m, operands[0], memo);
                        let sel_width = m.nodes[operands[0] as usize].width();
                        if let Some(sel) = exact_bound(sel_bounds) {
                            let mut chosen = None;
                            for arm in operands[1..].chunks_exact(3) {
                                let pattern_value =
                                    exact_bound(node_unsigned_bounds(m, arm[0], memo));
                                let wildcard_mask =
                                    exact_bound(node_unsigned_bounds(m, arm[1], memo));
                                if let (Some(pattern_value), Some(wildcard_mask)) =
                                    (pattern_value, wildcard_mask)
                                {
                                    if casez_pattern_matches(
                                        sel,
                                        pattern_value,
                                        wildcard_mask,
                                        sel_width,
                                    ) {
                                        chosen = Some(arm[2]);
                                        break;
                                    }
                                }
                            }
                            chosen
                                .map(|data| node_unsigned_bounds(m, data, memo))
                                .unwrap_or((0, 0))
                        } else {
                            let mut min = 0u128;
                            let mut max = 0u128;
                            for arm in operands[1..].chunks_exact(3) {
                                let bounds = node_unsigned_bounds(m, arm[2], memo);
                                min = min.min(bounds.0);
                                max = max.max(bounds.1);
                            }
                            (min, max)
                        }
                    }
                    GateOp::Slice { .. } if operands.len() == 1 => default,
                    GateOp::Concat => {
                        let mut min = 0u128;
                        let mut max = 0u128;
                        let mut supported = true;
                        for &operand in operands {
                            let operand_width = m.nodes[operand as usize].width();
                            if operand_width >= 128 {
                                supported = false;
                                break;
                            }
                            let (op_min, op_max) = node_unsigned_bounds(m, operand, memo);
                            min = (min << operand_width) | (op_min & width_mask(operand_width));
                            max = (max << operand_width) | (op_max & width_mask(operand_width));
                        }
                        if supported {
                            (min & width_mask(*width), max & width_mask(*width))
                        } else {
                            default
                        }
                    }
                    GateOp::Shl if operands.len() == 2 => {
                        let lhs = node_unsigned_bounds(m, operands[0], memo);
                        let src_width = u128::from(m.nodes[operands[0] as usize].width());
                        let rhs_bounds = node_unsigned_bounds(m, operands[1], memo);
                        let rhs = exact_bound(rhs_bounds);
                        let rhs_all_overshift = rhs_bounds.0 >= src_width
                            || small_value_set_min_at_least(m, operands[1], src_width);
                        match rhs {
                            _ if lhs == (0, 0) => (0, 0),
                            _ if rhs_all_overshift => (0, 0),
                            Some(0) => lhs,
                            Some(amount) => {
                                let shift = amount as u32;
                                if lhs.1 <= (width_mask(*width) >> shift) {
                                    (
                                        (lhs.0 << shift) & width_mask(*width),
                                        (lhs.1 << shift) & width_mask(*width),
                                    )
                                } else {
                                    default
                                }
                            }
                            _ => default,
                        }
                    }
                    GateOp::Shr if operands.len() == 2 => {
                        let lhs = node_unsigned_bounds(m, operands[0], memo);
                        let src_width = u128::from(m.nodes[operands[0] as usize].width());
                        let rhs_bounds = node_unsigned_bounds(m, operands[1], memo);
                        let rhs = exact_bound(rhs_bounds);
                        let rhs_all_overshift = rhs_bounds.0 >= src_width
                            || small_value_set_min_at_least(m, operands[1], src_width);
                        match rhs {
                            _ if lhs == (0, 0) => (0, 0),
                            _ if rhs_all_overshift => (0, 0),
                            Some(0) => lhs,
                            Some(amount) => {
                                let shift = amount as u32;
                                (lhs.0 >> shift, lhs.1 >> shift)
                            }
                            _ if exact_bound(lhs).is_some() => {
                                let lhs_value = exact_bound(lhs).expect("guard checked");
                                let min_amount = rhs_bounds.0.min(src_width);
                                let max_amount = rhs_bounds.1.min(src_width);
                                let upper = if min_amount >= src_width {
                                    0
                                } else {
                                    lhs_value >> (min_amount as u32)
                                };
                                let lower = if max_amount >= src_width {
                                    0
                                } else {
                                    lhs_value >> (max_amount as u32)
                                };
                                (lower.min(upper), lower.max(upper))
                            }
                            None => default,
                        }
                    }
                    _ => default,
                }
            }
        }
    };

    memo.insert(id, bounds);
    bounds
}

pub(crate) fn obvious_unsigned_compare_result(
    m: &Module,
    op: GateOp,
    lhs: NodeId,
    rhs: NodeId,
) -> Option<u128> {
    if lhs == rhs {
        return match op {
            GateOp::Eq | GateOp::Le | GateOp::Ge => Some(1),
            GateOp::Neq | GateOp::Lt | GateOp::Gt => Some(0),
            _ => None,
        };
    }

    if can_prove_compare_via_small_value_sets(m, lhs, rhs) {
        let mut set_ctx = SmallValueSetContext::default();
        if let (Some(lhs_values), Some(rhs_values)) = (
            node_small_value_set(m, lhs, &mut set_ctx),
            node_small_value_set(m, rhs, &mut set_ctx),
        ) {
            let mut saw_true = false;
            let mut saw_false = false;
            for &a in &lhs_values {
                for &b in &rhs_values {
                    let result = match op {
                        GateOp::Eq => a == b,
                        GateOp::Neq => a != b,
                        GateOp::Lt => a < b,
                        GateOp::Gt => a > b,
                        GateOp::Le => a <= b,
                        GateOp::Ge => a >= b,
                        _ => return None,
                    };
                    saw_true |= result;
                    saw_false |= !result;
                    if saw_true && saw_false {
                        break;
                    }
                }
                if saw_true && saw_false {
                    break;
                }
            }
            if saw_true ^ saw_false {
                return Some(u128::from(saw_true));
            }
        }
    }

    let mut memo = HashMap::new();
    let lhs_bounds = node_unsigned_bounds(m, lhs, &mut memo);
    let rhs_bounds = node_unsigned_bounds(m, rhs, &mut memo);
    obvious_unsigned_compare_from_bounds(op, lhs_bounds, rhs_bounds)
}
