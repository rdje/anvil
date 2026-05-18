//! PHASE-7-ORACLE-MICRODESIGN.2a — source-level const-expr / parameter
//! IR + construction-time evaluator (the oracle).
//!
//! Phase 7 is the *opposite* of the DUT lane (`src/ir/` + `src/gen/`):
//! instead of structurally-valid random RTL with no semantic oracle, it
//! emits tiny `.sv` whose elaboration facts are **known by
//! construction** and shipped in a machine-checkable manifest, so a
//! downstream tool can be checked against an oracle, not merely
//! "did it not error" (see `DEVELOPMENT_NOTES.md` "Phase 7 oracle-backed
//! micro-design artifact family design").
//!
//! This module is `.2a` only: the **source-level constant/parameter
//! IR** (a typed parameter+localparam dependency DAG of integer
//! constant expressions) and the **construction-time evaluator** that
//! resolves every node's value as the DAG is built — the *oracle*.
//! It is a **separate generator path**: it is deliberately *not*
//! threaded through the gate-level circuit IR (the circuit IR has no
//! `parameter`/`localparam`/expression concept; forcing them through
//! scalar `u32` node graphs is the category error `.1` rejected).
//! No SV/manifest emit, no parity harness yet — those are `.2b`/`.2c`.
//!
//! Reproducibility follows the project convention: one
//! `ChaCha8Rng::seed_from_u64(seed)`, no `thread_rng`, no system time
//! — `(seed, knobs)` ⇒ byte-identical IR + identical resolved values.

use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use std::collections::BTreeMap;

/// Integer constant-expression node. The bounded integer subset Phase 7
/// emits, evaluated in `i128` with SV-constant-expression-style
/// semantics. The rules-first builder keeps every intermediate value
/// well inside `i128`, so the oracle is *trivially exact* (width-sized
/// truncation against declared port/param widths is `.2b`'s concern;
/// `.2a` is the value DAG).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConstExpr {
    /// Integer literal.
    Lit(i128),
    /// Reference to an earlier `ParamDecl` by name (DAG edge).
    Param(String),
    Unary(UnOp, Box<ConstExpr>),
    Bin(BinOp, Box<ConstExpr>, Box<ConstExpr>),
    /// `cond ? a : b` (SV ternary; `cond` truthiness is `!= 0`).
    Ternary(Box<ConstExpr>, Box<ConstExpr>, Box<ConstExpr>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnOp {
    /// Arithmetic negation.
    Neg,
    /// Bitwise NOT (`~`).
    BitNot,
    /// Logical NOT (`!`) → 1/0.
    LogNot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Shl,
    Shr,
    BitAnd,
    BitOr,
    BitXor,
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
    LogAnd,
    LogOr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParamKind {
    /// `parameter` — overridable at instantiation.
    Parameter,
    /// `localparam` — derived, not overridable.
    Localparam,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParamDecl {
    pub name: String,
    pub kind: ParamKind,
    pub expr: ConstExpr,
    /// Construction-time-resolved value — **the oracle**. Filled by
    /// [`resolve`] in declaration order; never re-derived elsewhere
    /// (`.2b`'s SV text and JSON manifest will both read this single
    /// source of truth).
    pub value: i128,
}

/// An ordered parameter/localparam dependency DAG: decl `i` may
/// reference any decl `< i` by name (no forward refs, no cycles —
/// the rules-first builder guarantees this by construction).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConstExprUnit {
    pub params: Vec<ParamDecl>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvalError {
    /// Referenced a name that is not in the resolved environment
    /// (a forward reference or an undefined param — a malformed unit;
    /// the builder never produces one).
    UndefinedParam(String),
    DivByZero,
}

/// SV-style truthiness: nonzero is true.
#[inline]
fn truthy(v: i128) -> bool {
    v != 0
}

/// Evaluate one expression against the already-resolved environment.
/// Pure and total except for the two defensive `EvalError`s. Semantics
/// match SV constant expressions for the bounded integer subset:
/// truncating division/modulo, C-style shift, comparisons/logicals
/// yield `1`/`0`.
pub fn eval(e: &ConstExpr, env: &BTreeMap<String, i128>) -> Result<i128, EvalError> {
    Ok(match e {
        ConstExpr::Lit(v) => *v,
        ConstExpr::Param(name) => *env
            .get(name)
            .ok_or_else(|| EvalError::UndefinedParam(name.clone()))?,
        ConstExpr::Unary(op, a) => {
            let a = eval(a, env)?;
            match op {
                UnOp::Neg => a.wrapping_neg(),
                UnOp::BitNot => !a,
                UnOp::LogNot => i128::from(!truthy(a)),
            }
        }
        ConstExpr::Bin(op, a, b) => {
            let x = eval(a, env)?;
            let y = eval(b, env)?;
            match op {
                BinOp::Add => x.wrapping_add(y),
                BinOp::Sub => x.wrapping_sub(y),
                BinOp::Mul => x.wrapping_mul(y),
                BinOp::Div => {
                    if y == 0 {
                        return Err(EvalError::DivByZero);
                    }
                    x.wrapping_div(y)
                }
                BinOp::Mod => {
                    if y == 0 {
                        return Err(EvalError::DivByZero);
                    }
                    x.wrapping_rem(y)
                }
                // Shift amount is clamped to [0,127] so a malformed
                // (builder-impossible) huge amount cannot panic; the
                // builder only ever emits small non-negative amounts.
                BinOp::Shl => x.wrapping_shl((y.clamp(0, 127)) as u32),
                BinOp::Shr => x.wrapping_shr((y.clamp(0, 127)) as u32),
                BinOp::BitAnd => x & y,
                BinOp::BitOr => x | y,
                BinOp::BitXor => x ^ y,
                BinOp::Eq => i128::from(x == y),
                BinOp::Ne => i128::from(x != y),
                BinOp::Lt => i128::from(x < y),
                BinOp::Gt => i128::from(x > y),
                BinOp::Le => i128::from(x <= y),
                BinOp::Ge => i128::from(x >= y),
                BinOp::LogAnd => i128::from(truthy(x) && truthy(y)),
                BinOp::LogOr => i128::from(truthy(x) || truthy(y)),
            }
        }
        ConstExpr::Ternary(c, a, b) => {
            if truthy(eval(c, env)?) {
                eval(a, env)?
            } else {
                eval(b, env)?
            }
        }
    })
}

/// Resolve every decl in declaration order, filling each
/// [`ParamDecl::value`], and return the final name→value environment.
/// This *is* the oracle: it is run once at construction time; emitted
/// SV and the manifest (`.2b`) read the stored values, never
/// re-derive them.
pub fn resolve(unit: &mut ConstExprUnit) -> Result<BTreeMap<String, i128>, EvalError> {
    let mut env: BTreeMap<String, i128> = BTreeMap::new();
    for p in &mut unit.params {
        let v = eval(&p.expr, &env)?;
        p.value = v;
        env.insert(p.name.clone(), v);
    }
    Ok(env)
}

/// Rules-first, reproducible builder: from `seed`, construct a small
/// **valid** const-expr/parameter dependency DAG of `n_params` decls
/// (≥ 1) and resolve every value. Decl 0 is a literal; each later decl
/// is, by rule, an expression over *earlier* decl names + small
/// literals (parameter/localparam chains like `localparam B = A*2;
/// localparam C = B + A`), so the DAG is acyclic and forward-ref-free
/// by construction — `resolve` cannot error. Byte-stable per `seed`
/// (ChaCha8, no `thread_rng`).
pub fn build_constexpr_unit(seed: u64, n_params: usize) -> ConstExprUnit {
    let n = n_params.max(1);
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let mut params: Vec<ParamDecl> = Vec::with_capacity(n);

    for i in 0..n {
        let name = format!("P{i}");
        // Decl 0 (and, rarely, later decls) is a bare literal so the
        // DAG always has a defined root; later decls reference an
        // earlier decl by rule, forming dependency chains.
        let expr = if i == 0 || rng.gen_bool(0.15) {
            ConstExpr::Lit(rng.gen_range(0..=64) as i128)
        } else {
            let earlier = |rng: &mut ChaCha8Rng| -> ConstExpr {
                ConstExpr::Param(format!("P{}", rng.gen_range(0..i)))
            };
            let small = |rng: &mut ChaCha8Rng| ConstExpr::Lit(rng.gen_range(1..=8) as i128);
            match rng.gen_range(0u8..7) {
                0 => ConstExpr::Bin(
                    BinOp::Add,
                    Box::new(earlier(&mut rng)),
                    Box::new(small(&mut rng)),
                ),
                1 => ConstExpr::Bin(
                    BinOp::Mul,
                    Box::new(earlier(&mut rng)),
                    Box::new(small(&mut rng)),
                ),
                2 => ConstExpr::Bin(
                    BinOp::Shl,
                    Box::new(earlier(&mut rng)),
                    // small non-negative shift amount
                    Box::new(ConstExpr::Lit(rng.gen_range(0..=4) as i128)),
                ),
                3 => ConstExpr::Bin(
                    BinOp::Sub,
                    Box::new(earlier(&mut rng)),
                    Box::new(small(&mut rng)),
                ),
                4 => ConstExpr::Bin(
                    BinOp::BitOr,
                    Box::new(earlier(&mut rng)),
                    Box::new(small(&mut rng)),
                ),
                5 => {
                    // precedence-sensitive: earlier + earlier' * lit
                    let a = earlier(&mut rng);
                    let b = earlier(&mut rng);
                    ConstExpr::Bin(
                        BinOp::Add,
                        Box::new(a),
                        Box::new(ConstExpr::Bin(
                            BinOp::Mul,
                            Box::new(b),
                            Box::new(small(&mut rng)),
                        )),
                    )
                }
                _ => {
                    // ternary over a comparison of an earlier decl
                    let c = ConstExpr::Bin(
                        BinOp::Ge,
                        Box::new(earlier(&mut rng)),
                        Box::new(small(&mut rng)),
                    );
                    ConstExpr::Ternary(
                        Box::new(c),
                        Box::new(earlier(&mut rng)),
                        Box::new(small(&mut rng)),
                    )
                }
            }
        };
        // First decl is always a `parameter` (an override surface);
        // chained decls are mostly `localparam` (derived).
        let kind = if i == 0 || rng.gen_bool(0.3) {
            ParamKind::Parameter
        } else {
            ParamKind::Localparam
        };
        params.push(ParamDecl {
            name,
            kind,
            expr,
            value: 0,
        });
    }

    let mut unit = ConstExprUnit { params };
    // Resolve in-place: the builder *is* the oracle (no analysis pass,
    // no re-parse). A valid-by-construction unit never errors here.
    resolve(&mut unit).expect("rules-first const-expr unit is valid by construction");
    unit
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lit(v: i128) -> ConstExpr {
        ConstExpr::Lit(v)
    }
    fn p(n: &str) -> ConstExpr {
        ConstExpr::Param(n.into())
    }
    fn bin(o: BinOp, a: ConstExpr, b: ConstExpr) -> ConstExpr {
        ConstExpr::Bin(o, Box::new(a), Box::new(b))
    }

    /// The evaluator matches independently-computed values on a curated
    /// set: precedence, shift/width, comparisons→1/0, truncating
    /// div/mod, ternary, and a localparam dependency chain.
    #[test]
    fn eval_matches_known_values() {
        // Precedence: 2 + 3*4 == 14 (not 20).
        let e = bin(BinOp::Add, lit(2), bin(BinOp::Mul, lit(3), lit(4)));
        assert_eq!(eval(&e, &BTreeMap::new()).unwrap(), 14);

        // Shift + bitwise: (5 << 2) | 1 == 21.
        let e = bin(BinOp::BitOr, bin(BinOp::Shl, lit(5), lit(2)), lit(1));
        assert_eq!(eval(&e, &BTreeMap::new()).unwrap(), 21);

        // Comparisons yield 1/0; logical too.
        assert_eq!(
            eval(&bin(BinOp::Lt, lit(3), lit(7)), &BTreeMap::new()).unwrap(),
            1
        );
        assert_eq!(
            eval(&bin(BinOp::Ge, lit(3), lit(7)), &BTreeMap::new()).unwrap(),
            0
        );
        assert_eq!(
            eval(&bin(BinOp::LogAnd, lit(0), lit(9)), &BTreeMap::new()).unwrap(),
            0
        );

        // Truncating division / modulo toward zero.
        assert_eq!(
            eval(&bin(BinOp::Div, lit(-7), lit(2)), &BTreeMap::new()).unwrap(),
            -3
        );
        assert_eq!(
            eval(&bin(BinOp::Mod, lit(-7), lit(2)), &BTreeMap::new()).unwrap(),
            -1
        );

        // Ternary + unary.
        let e = ConstExpr::Ternary(
            Box::new(bin(BinOp::Eq, lit(1), lit(1))),
            Box::new(ConstExpr::Unary(UnOp::Neg, Box::new(lit(5)))),
            Box::new(lit(99)),
        );
        assert_eq!(eval(&e, &BTreeMap::new()).unwrap(), -5);

        // Localparam dependency chain: A=5; B=A*2; C=B+A → 5,10,15.
        let mut unit = ConstExprUnit {
            params: vec![
                ParamDecl {
                    name: "A".into(),
                    kind: ParamKind::Parameter,
                    expr: lit(5),
                    value: 0,
                },
                ParamDecl {
                    name: "B".into(),
                    kind: ParamKind::Localparam,
                    expr: bin(BinOp::Mul, p("A"), lit(2)),
                    value: 0,
                },
                ParamDecl {
                    name: "C".into(),
                    kind: ParamKind::Localparam,
                    expr: bin(BinOp::Add, p("B"), p("A")),
                    value: 0,
                },
            ],
        };
        let env = resolve(&mut unit).unwrap();
        assert_eq!(unit.params[0].value, 5);
        assert_eq!(unit.params[1].value, 10);
        assert_eq!(unit.params[2].value, 15);
        assert_eq!(env["C"], 15);
    }

    /// Defensive `EvalError` paths are reachable and correct
    /// (the rules-first builder never produces these, but `eval`
    /// must classify a malformed unit, not panic).
    #[test]
    fn eval_reports_div_by_zero_and_undefined_param() {
        assert_eq!(
            eval(&bin(BinOp::Div, lit(1), lit(0)), &BTreeMap::new()),
            Err(EvalError::DivByZero)
        );
        assert_eq!(
            eval(&bin(BinOp::Mod, lit(1), lit(0)), &BTreeMap::new()),
            Err(EvalError::DivByZero)
        );
        assert_eq!(
            eval(&p("missing"), &BTreeMap::new()),
            Err(EvalError::UndefinedParam("missing".into()))
        );
    }

    /// `build_constexpr_unit` is byte-stable per seed (the
    /// reproducibility contract) and seed-sensitive across seeds.
    #[test]
    fn build_is_reproducible_and_seed_sensitive() {
        for seed in [0u64, 1, 7, 42, 12345] {
            let a = build_constexpr_unit(seed, 8);
            let b = build_constexpr_unit(seed, 8);
            assert_eq!(a, b, "same seed must yield byte-identical IR + values");
        }
        // Different seeds give different units (sanity — not a strict
        // guarantee, but must hold for this fixed pair).
        assert_ne!(
            build_constexpr_unit(1, 8),
            build_constexpr_unit(2, 8),
            "distinct seeds should produce distinct units"
        );
    }

    /// The stored `value` oracle equals a *fresh* re-evaluation of
    /// every decl against the resolved prefix env — i.e. the
    /// construction-time oracle never drifts from the expressions it
    /// claims to resolve. This is the load-bearing `.2a` invariant.
    #[test]
    fn stored_values_are_consistent_with_a_fresh_reeval() {
        for seed in 0..16u64 {
            let unit = build_constexpr_unit(seed, 10);
            assert!(!unit.params.is_empty());
            let mut env: BTreeMap<String, i128> = BTreeMap::new();
            for d in &unit.params {
                let fresh = eval(&d.expr, &env).expect("valid-by-construction unit must evaluate");
                assert_eq!(
                    fresh, d.value,
                    "seed {seed}: stored oracle value for {} drifted from its expr",
                    d.name
                );
                env.insert(d.name.clone(), d.value);
            }
            // Decl 0 is always a parameter root (override surface).
            assert_eq!(unit.params[0].kind, ParamKind::Parameter);
        }
    }
}
