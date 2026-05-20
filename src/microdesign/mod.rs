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
//! Contents:
//! - `.2a` — the **source-level constant/parameter IR** (a typed
//!   parameter+localparam dependency DAG of integer constant
//!   expressions) and the **construction-time evaluator** that
//!   resolves every node's value as the DAG is built — the *oracle*.
//! - `.2b` — the **un-resolved SV emitter** (`rtl_const_expr` family)
//!   and the **JSON expected-facts manifest emitter**, both emitted
//!   *from the same evaluated IR* (the SV keeps parameters symbolic;
//!   the manifest records what elaboration must resolve them to).
//! - `.2c.1` — the **parity comparator core**: `ToolReport` (the
//!   resolved-facts view a downstream consumer is expected to produce,
//!   normalized to the same fact set the manifest carries), `Divergence`
//!   (the comparator's per-category failure variants), and
//!   `compare_manifest_to_tool_report` (a cargo-portable walker; no
//!   tool invocation here), with `synthetic_tool_report_from_manifest`
//!   as the always-agreeing reference used by `.2c.1`'s proofs and as
//!   the structural fallback in `.2c.2`'s real-tool path.
//! - `.2c.2a` — the **scoped comparator + tool-capability scope**:
//!   `FactCategory` (one per fact axis), `ParityScope` (which axes
//!   to enforce — different downstream tools expose different
//!   subsets of the manifest's facts; yosys 0.64 `write_json` folds
//!   localparams + package_constants), and
//!   `compare_manifest_to_tool_report_in_scope` (the scoped walker;
//!   the strict comparator delegates to this with `ParityScope::all()`).
//!   The yosys-specific extractor and the actual tool-equipped
//!   real-corpus end-to-end run are `tests/microdesign_parity.rs`
//!   (`.2c.2a`'s harness) and `.2c.2b`'s gated `--ignored` run.
//!
//! It is a **separate generator path**: deliberately *not* threaded
//! through the gate-level circuit IR (the circuit IR has no
//! `parameter`/`localparam`/expression concept; forcing them through
//! scalar `u32` node graphs is the category error `.1` rejected) and
//! never invoked by the DUT generate path (so the DUT lane is
//! byte-identical by construction). The parity harness + repo-owned
//! gate are `.2c`.
//!
//! Reproducibility follows the project convention: one
//! `ChaCha8Rng::seed_from_u64(seed)`, no `thread_rng`, no system time
//! — `(seed, knobs)` ⇒ byte-identical IR + identical resolved values.

use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};
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

// ===================================================================
// PHASE-7-ORACLE-MICRODESIGN.2b — SV emitter + JSON manifest emitter.
//
// Both are emitted *from the same evaluated IR* (`.2a`'s resolved
// `ParamDecl.value` oracle): the `.sv` text keeps parameters
// **symbolic** (un-resolved) — that gap between symbolic text and the
// manifest's resolved facts is exactly the front-end/elaboration
// behaviour Phase 7 stresses. No analysis pass, no re-parse. Behind
// an explicit artifact-family path: `microdesign` is a *separate
// module never invoked by the DUT generate path*, so the DUT lane is
// byte-identical by construction (default-off is trivial; the Phase 9
// selector wires invocation later).
// ===================================================================

/// Minimum bit-width to hold non-negative `v` (≥ 1). Negative values
/// are clamped to 0 (the rules-first builder keeps widths' driving
/// values non-negative; this is a defensive floor).
fn bits_for(v: i128) -> u32 {
    let v = v.max(0) as u128;
    if v < 2 {
        1
    } else {
        128 - (v).leading_zeros()
    }
}

/// Pretty-print a `ConstExpr` to SystemVerilog source. **Fully
/// parenthesized**: the evaluator already fixed semantics; the
/// printer must not silently change them, and explicit parens make
/// the emitted text precedence-unambiguous for the downstream
/// front-end (the precedence-sensitivity axis is exercised by the
/// `.2a` builder's nested `a + b*c` / ternary shapes — round-tripped
/// here as written).
pub fn expr_to_sv(e: &ConstExpr) -> String {
    match e {
        ConstExpr::Lit(v) => v.to_string(),
        ConstExpr::Param(n) => n.clone(),
        ConstExpr::Unary(op, a) => {
            let s = expr_to_sv(a);
            let o = match op {
                UnOp::Neg => "-",
                UnOp::BitNot => "~",
                UnOp::LogNot => "!",
            };
            format!("({o}{s})")
        }
        ConstExpr::Bin(op, a, b) => {
            let o = match op {
                BinOp::Add => "+",
                BinOp::Sub => "-",
                BinOp::Mul => "*",
                BinOp::Div => "/",
                BinOp::Mod => "%",
                BinOp::Shl => "<<",
                BinOp::Shr => ">>",
                BinOp::BitAnd => "&",
                BinOp::BitOr => "|",
                BinOp::BitXor => "^",
                BinOp::Eq => "==",
                BinOp::Ne => "!=",
                BinOp::Lt => "<",
                BinOp::Gt => ">",
                BinOp::Le => "<=",
                BinOp::Ge => ">=",
                BinOp::LogAnd => "&&",
                BinOp::LogOr => "||",
            };
            format!("({} {} {})", expr_to_sv(a), o, expr_to_sv(b))
        }
        ConstExpr::Ternary(c, a, b) => format!(
            "({} ? {} : {})",
            expr_to_sv(c),
            expr_to_sv(a),
            expr_to_sv(b)
        ),
    }
}

/// The fixed package-constant for a unit (the package-qualified
/// constant axis). Derived deterministically from the seed.
fn pkg_const(seed: u64) -> i128 {
    (seed % 64) as i128 + 1
}

/// The signal-width localparam expression for a unit: a symbolic expr
/// over the last decl that always resolves to a positive width
/// (`(<last> % 8) + 1` ⇒ 1..=8). Returns `(sv_expr, resolved_bits)`.
fn width_expr(unit: &ConstExprUnit) -> (String, u32) {
    let last = unit.params.last().expect("unit has >=1 decl");
    let sv = format!("(({} % 8) + 1)", last.name);
    let bits = (last.value.rem_euclid(8) + 1) as u32;
    (sv, bits)
}

/// The `generate if` predicate for a unit: `<P0> >= <pkg_const>`.
/// Returns `(sv_predicate, taken)` where `taken` is resolved from the
/// oracle.
fn gen_predicate(unit: &ConstExprUnit, seed: u64) -> (String, bool) {
    let p0 = &unit.params[0];
    let k = pkg_const(seed);
    (format!("({} >= {})", p0.name, k), p0.value >= k)
}

/// Emit the `rtl_const_expr` micro-design as **un-resolved**
/// SystemVerilog: a tiny package + a module whose `parameter`s carry
/// their *symbolic* defining expressions (not the resolved values),
/// `localparam` chains, an expr-derived-width signal, a package-
/// qualified constant reference, and a `generate if` over a param
/// expression. Byte-stable per `(unit, seed)`.
pub fn emit_sv(unit: &ConstExprUnit, seed: u64) -> String {
    let mut s = String::new();
    let pkg = format!("mc_{seed}_pkg");
    let top = format!("mc_{seed}");
    s.push_str(&format!(
        "// Generated by anvil microdesign (Phase 7). Module: {top}\n"
    ));
    s.push_str(&format!("package {pkg};\n"));
    s.push_str(&format!("    localparam int K = {};\n", pkg_const(seed)));
    s.push_str("endpackage\n\n");

    let params: Vec<&ParamDecl> = unit
        .params
        .iter()
        .filter(|p| p.kind == ParamKind::Parameter)
        .collect();
    let localparams: Vec<&ParamDecl> = unit
        .params
        .iter()
        .filter(|p| p.kind == ParamKind::Localparam)
        .collect();

    s.push_str(&format!("module {top} #(\n"));
    for (i, p) in params.iter().enumerate() {
        let comma = if i + 1 < params.len() { "," } else { "" };
        s.push_str(&format!(
            "    parameter int {} = {}{}\n",
            p.name,
            expr_to_sv(&p.expr),
            comma
        ));
    }
    s.push_str(");\n");
    // localparam decls in body, in declaration order (chains).
    for p in &localparams {
        s.push_str(&format!(
            "    localparam int {} = {};\n",
            p.name,
            expr_to_sv(&p.expr)
        ));
    }
    s.push_str(&format!("    localparam int PKG_REF = {pkg}::K;\n"));
    let (wexpr, _bits) = width_expr(unit);
    s.push_str(&format!("    localparam int W_SIG = {wexpr};\n"));
    s.push_str("    logic [W_SIG-1:0] sig;\n");
    s.push_str("    assign sig = '0;\n");
    let (pred, _taken) = gen_predicate(unit, seed);
    s.push_str("    generate\n");
    s.push_str(&format!("        if {pred} begin : g_taken\n"));
    s.push_str("            logic gflag;\n");
    s.push_str("            assign gflag = 1'b1;\n");
    s.push_str("        end else begin : g_else\n");
    s.push_str("            logic gflag;\n");
    s.push_str("            assign gflag = 1'b0;\n");
    s.push_str("        end\n");
    s.push_str("    endgenerate\n");
    s.push_str("endmodule\n");
    s
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FactEntry {
    pub value: i128,
    pub expr: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WidthFact {
    pub msb: i64,
    pub lsb: i64,
    pub bits: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GenFact {
    pub taken: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConstExprFact {
    pub expr: String,
    pub value: i128,
    pub width: u32,
}

/// The expected-elaboration-facts manifest (`.1`'s schema). Every
/// value comes from `.2a`'s resolved oracle — never re-derived.
/// `BTreeMap` everywhere ⇒ deterministic key order ⇒ byte-stable.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Manifest {
    pub seed: u64,
    pub top: String,
    pub params: BTreeMap<String, FactEntry>,
    pub localparams: BTreeMap<String, FactEntry>,
    pub widths: BTreeMap<String, WidthFact>,
    pub generate: BTreeMap<String, GenFact>,
    pub package_constants: BTreeMap<String, i128>,
    pub const_exprs: Vec<ConstExprFact>,
}

/// Build the manifest from the evaluated unit (the oracle). Mirrors
/// exactly what `emit_sv` declares.
pub fn build_manifest(unit: &ConstExprUnit, seed: u64) -> Manifest {
    let mut params = BTreeMap::new();
    let mut localparams = BTreeMap::new();
    let mut const_exprs = Vec::new();
    for p in &unit.params {
        let entry = FactEntry {
            value: p.value,
            expr: expr_to_sv(&p.expr),
        };
        match p.kind {
            ParamKind::Parameter => {
                params.insert(p.name.clone(), entry);
            }
            ParamKind::Localparam => {
                localparams.insert(p.name.clone(), entry);
            }
        }
        const_exprs.push(ConstExprFact {
            expr: expr_to_sv(&p.expr),
            value: p.value,
            width: bits_for(p.value),
        });
    }
    let (_wexpr, bits) = width_expr(unit);
    let mut widths = BTreeMap::new();
    widths.insert(
        "sig".to_string(),
        WidthFact {
            msb: bits as i64 - 1,
            lsb: 0,
            bits,
        },
    );
    let (_pred, taken) = gen_predicate(unit, seed);
    let mut generate = BTreeMap::new();
    generate.insert("g_taken".to_string(), GenFact { taken });
    let mut package_constants = BTreeMap::new();
    package_constants.insert(format!("mc_{seed}_pkg::K"), pkg_const(seed));
    Manifest {
        seed,
        top: format!("mc_{seed}"),
        params,
        localparams,
        widths,
        generate,
        package_constants,
        const_exprs,
    }
}

/// Serialize the manifest as deterministic pretty JSON.
pub fn emit_manifest(unit: &ConstExprUnit, seed: u64) -> String {
    serde_json::to_string_pretty(&build_manifest(unit, seed)).expect("manifest serializes")
}

// ===================================================================
// PHASE-7-ORACLE-MICRODESIGN.2c.1 — parity comparator core.
//
// The harness wiring (which corpus, which downstream consumer, how the
// tool's output is parsed into a `ToolReport`) lives in
// `tests/microdesign_parity.rs` and is `#[ignore]`-gated so the
// portable `cargo test` stays green tool-less — the Phase-1 doctrine
// reaffirmed in this tree's Decisions (and matched by Phase 6 memory
// `.2.2` and DIFFERENTIAL-SIMULATION `.2b`). The pure-Rust comparator
// below operates on already-collected reports as input, so it is
// fully proven cargo-portably (`.2c.1`) and reused as-is by the
// real-tool gate (`.2c.2`).
// ===================================================================

/// What a downstream consumer's resolved-facts report looks like,
/// normalized to the same fact set the manifest carries (params,
/// localparams, widths, generate-branch taken, package constants).
///
/// Names match the manifest keys exactly; values are resolved integers
/// (no symbolic SV expressions — the tool resolved them, by definition).
/// `BTreeMap` everywhere ⇒ deterministic iteration. The
/// `serde::{Serialize,Deserialize}` derives let a tool's
/// post-extraction artifact round-trip through JSON next to the
/// manifest for diagnostics.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolReport {
    pub seed: u64,
    pub top: String,
    /// Resolved parameter values: `name → value`.
    pub params: BTreeMap<String, i128>,
    /// Resolved localparam values: `name → value`.
    pub localparams: BTreeMap<String, i128>,
    /// Declared signal widths: `name → WidthFact`.
    pub widths: BTreeMap<String, WidthFact>,
    /// Generate-block decisions: `name → taken`.
    pub generate: BTreeMap<String, bool>,
    /// Package-qualified constant values: `qualified_name → value`.
    pub package_constants: BTreeMap<String, i128>,
}

/// A single category of disagreement between the manifest (the oracle)
/// and the tool report. Listed independently per fact category and per
/// direction (missing-in-tool / missing-in-manifest / mismatch) so the
/// comparator surfaces *which* axis broke — `.1`'s rejected-alternatives
/// discussion noted a single "facts disagree" bit is not enough for
/// downstream triage. The comparator accumulates the full set so the
/// gate either reports `Ok(())` or retains the full counterexample
/// profile in one pass.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Divergence {
    SeedMismatch {
        expected: u64,
        actual: u64,
    },
    TopMismatch {
        expected: String,
        actual: String,
    },
    ParamMissingInTool {
        name: String,
        expected: i128,
    },
    ParamMissingInManifest {
        name: String,
        actual: i128,
    },
    ParamMismatch {
        name: String,
        expected: i128,
        actual: i128,
    },
    LocalparamMissingInTool {
        name: String,
        expected: i128,
    },
    LocalparamMissingInManifest {
        name: String,
        actual: i128,
    },
    LocalparamMismatch {
        name: String,
        expected: i128,
        actual: i128,
    },
    WidthMissingInTool {
        name: String,
        expected: WidthFact,
    },
    WidthMissingInManifest {
        name: String,
        actual: WidthFact,
    },
    WidthMismatch {
        name: String,
        expected: WidthFact,
        actual: WidthFact,
    },
    GenerateMissingInTool {
        name: String,
        expected: bool,
    },
    GenerateMissingInManifest {
        name: String,
        actual: bool,
    },
    GenerateMismatch {
        name: String,
        expected: bool,
        actual: bool,
    },
    PackageConstantMissingInTool {
        name: String,
        expected: i128,
    },
    PackageConstantMissingInManifest {
        name: String,
        actual: i128,
    },
    PackageConstantMismatch {
        name: String,
        expected: i128,
        actual: i128,
    },
}

/// Walk `manifest` × `report` and return every disagreement. `Ok(())`
/// ⇔ exact agreement across every fact category present in either.
///
/// Symbolic `expr` strings on the manifest side are deliberately
/// **not** compared — they are the un-resolved-SV documentation of
/// what was given to the tool, not something the tool re-emits. Only
/// resolved values, widths, generate-branch decisions, and
/// package-constant values are checked.
pub fn compare_manifest_to_tool_report(
    manifest: &Manifest,
    report: &ToolReport,
) -> Result<(), Vec<Divergence>> {
    compare_manifest_to_tool_report_in_scope(manifest, report, &ParityScope::all())
}

/// The fact-category axes the comparator can enforce — one per manifest
/// section. The granularity matches `Divergence`'s per-axis variants.
///
/// Different downstream tools expose different subsets: yosys 0.64's
/// `write_json` carries top-level parameters and elaborated-generate
/// branches and wire widths but **folds** localparams and
/// package-qualified constants; richer-AST consumers
/// (`slang --ast-json`, `verilator --xml-only`) carry them all. The
/// parity gate scopes the comparator to the categories the chosen
/// consumer actually introspects.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum FactCategory {
    Seed,
    Top,
    Params,
    Localparams,
    Widths,
    Generate,
    PackageConstants,
}

/// The set of fact categories a particular parity gate enforces. Built
/// from the downstream consumer's capabilities (`.2c.2a`'s
/// `yosys_write_json_to_tool_report` enumerates yosys 0.64's scope).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParityScope {
    pub categories: std::collections::BTreeSet<FactCategory>,
}

impl ParityScope {
    /// Every category — the strict-all-axes comparison
    /// `compare_manifest_to_tool_report` delegates to.
    pub fn all() -> Self {
        let categories = [
            FactCategory::Seed,
            FactCategory::Top,
            FactCategory::Params,
            FactCategory::Localparams,
            FactCategory::Widths,
            FactCategory::Generate,
            FactCategory::PackageConstants,
        ]
        .iter()
        .copied()
        .collect();
        Self { categories }
    }

    /// No category — the comparator returns `Ok(())` even on a maximally
    /// disagreeing report. Useful for unit-testing the scoping itself.
    pub fn none() -> Self {
        Self {
            categories: std::collections::BTreeSet::new(),
        }
    }

    /// Just the listed categories.
    pub fn only(categories: &[FactCategory]) -> Self {
        Self {
            categories: categories.iter().copied().collect(),
        }
    }

    /// Whether `category` is in this scope.
    pub fn contains(&self, category: FactCategory) -> bool {
        self.categories.contains(&category)
    }
}

/// Walk `manifest` × `report` over the axes in `scope` and return every
/// disagreement. `Ok(())` ⇔ exact agreement on every scoped axis.
/// Out-of-scope axes are skipped entirely — they do not contribute
/// `MissingIn*`/`Mismatch` variants. The strict
/// [`compare_manifest_to_tool_report`] is exactly this with
/// `ParityScope::all()`.
pub fn compare_manifest_to_tool_report_in_scope(
    manifest: &Manifest,
    report: &ToolReport,
    scope: &ParityScope,
) -> Result<(), Vec<Divergence>> {
    let mut divs = Vec::new();

    if scope.contains(FactCategory::Seed) && manifest.seed != report.seed {
        divs.push(Divergence::SeedMismatch {
            expected: manifest.seed,
            actual: report.seed,
        });
    }
    if scope.contains(FactCategory::Top) && manifest.top != report.top {
        divs.push(Divergence::TopMismatch {
            expected: manifest.top.clone(),
            actual: report.top.clone(),
        });
    }

    if scope.contains(FactCategory::Params) {
        // Each side checked against the other.
        for (name, entry) in &manifest.params {
            match report.params.get(name) {
                Some(actual) if *actual == entry.value => {}
                Some(actual) => divs.push(Divergence::ParamMismatch {
                    name: name.clone(),
                    expected: entry.value,
                    actual: *actual,
                }),
                None => divs.push(Divergence::ParamMissingInTool {
                    name: name.clone(),
                    expected: entry.value,
                }),
            }
        }
        for (name, actual) in &report.params {
            if !manifest.params.contains_key(name) {
                divs.push(Divergence::ParamMissingInManifest {
                    name: name.clone(),
                    actual: *actual,
                });
            }
        }
    }

    if scope.contains(FactCategory::Localparams) {
        for (name, entry) in &manifest.localparams {
            match report.localparams.get(name) {
                Some(actual) if *actual == entry.value => {}
                Some(actual) => divs.push(Divergence::LocalparamMismatch {
                    name: name.clone(),
                    expected: entry.value,
                    actual: *actual,
                }),
                None => divs.push(Divergence::LocalparamMissingInTool {
                    name: name.clone(),
                    expected: entry.value,
                }),
            }
        }
        for (name, actual) in &report.localparams {
            if !manifest.localparams.contains_key(name) {
                divs.push(Divergence::LocalparamMissingInManifest {
                    name: name.clone(),
                    actual: *actual,
                });
            }
        }
    }

    if scope.contains(FactCategory::Widths) {
        for (name, expected) in &manifest.widths {
            match report.widths.get(name) {
                Some(actual) if actual == expected => {}
                Some(actual) => divs.push(Divergence::WidthMismatch {
                    name: name.clone(),
                    expected: expected.clone(),
                    actual: actual.clone(),
                }),
                None => divs.push(Divergence::WidthMissingInTool {
                    name: name.clone(),
                    expected: expected.clone(),
                }),
            }
        }
        for (name, actual) in &report.widths {
            if !manifest.widths.contains_key(name) {
                divs.push(Divergence::WidthMissingInManifest {
                    name: name.clone(),
                    actual: actual.clone(),
                });
            }
        }
    }

    if scope.contains(FactCategory::Generate) {
        for (name, expected) in &manifest.generate {
            match report.generate.get(name) {
                Some(actual) if *actual == expected.taken => {}
                Some(actual) => divs.push(Divergence::GenerateMismatch {
                    name: name.clone(),
                    expected: expected.taken,
                    actual: *actual,
                }),
                None => divs.push(Divergence::GenerateMissingInTool {
                    name: name.clone(),
                    expected: expected.taken,
                }),
            }
        }
        for (name, actual) in &report.generate {
            if !manifest.generate.contains_key(name) {
                divs.push(Divergence::GenerateMissingInManifest {
                    name: name.clone(),
                    actual: *actual,
                });
            }
        }
    }

    if scope.contains(FactCategory::PackageConstants) {
        for (name, expected) in &manifest.package_constants {
            match report.package_constants.get(name) {
                Some(actual) if actual == expected => {}
                Some(actual) => divs.push(Divergence::PackageConstantMismatch {
                    name: name.clone(),
                    expected: *expected,
                    actual: *actual,
                }),
                None => divs.push(Divergence::PackageConstantMissingInTool {
                    name: name.clone(),
                    expected: *expected,
                }),
            }
        }
        for (name, actual) in &report.package_constants {
            if !manifest.package_constants.contains_key(name) {
                divs.push(Divergence::PackageConstantMissingInManifest {
                    name: name.clone(),
                    actual: *actual,
                });
            }
        }
    }

    if divs.is_empty() {
        Ok(())
    } else {
        Err(divs)
    }
}

/// Construct a `ToolReport` that agrees with `manifest` exactly — i.e.
/// "what a perfectly-conforming downstream tool would have produced".
/// Used by `.2c.1`'s cargo-portable proofs (agreement and
/// perturbed-divergence fixtures) and as the structural fallback in
/// `.2c.2`'s real-tool gate.
pub fn synthetic_tool_report_from_manifest(manifest: &Manifest) -> ToolReport {
    ToolReport {
        seed: manifest.seed,
        top: manifest.top.clone(),
        params: manifest
            .params
            .iter()
            .map(|(n, e)| (n.clone(), e.value))
            .collect(),
        localparams: manifest
            .localparams
            .iter()
            .map(|(n, e)| (n.clone(), e.value))
            .collect(),
        widths: manifest.widths.clone(),
        generate: manifest
            .generate
            .iter()
            .map(|(n, g)| (n.clone(), g.taken))
            .collect(),
        package_constants: manifest.package_constants.clone(),
    }
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

    // ---- .2b: SV + JSON manifest emitters ----

    /// The emitted SV has the expected un-resolved shape: a package
    /// with `K`, a module with `parameter`/`localparam` decls carrying
    /// **symbolic** expressions (not the resolved integers), a
    /// package-qualified ref, an expr-derived-width signal, and a
    /// `generate if/else`.
    #[test]
    fn emit_sv_is_valid_unresolved_shape() {
        let unit = build_constexpr_unit(7, 8);
        let sv = emit_sv(&unit, 7);
        assert!(sv.contains("package mc_7_pkg;"));
        assert!(sv.contains("localparam int K = "));
        assert!(sv.contains("module mc_7 #("));
        assert!(sv.contains("parameter int P0 = "));
        assert!(sv.contains("localparam int PKG_REF = mc_7_pkg::K;"));
        assert!(sv.contains("localparam int W_SIG = ((P"));
        assert!(sv.contains("logic [W_SIG-1:0] sig;"));
        assert!(sv.contains("generate") && sv.contains(": g_taken") && sv.contains(": g_else"));
        assert!(sv.trim_end().ends_with("endmodule"));
        // P0 is a literal root — its decl is `parameter int P0 = <lit>`
        // (symbolic-but-trivial); a *chained* decl must show an
        // operator (un-resolved), never a bare resolved integer only.
        let chained = unit
            .params
            .iter()
            .find(|p| matches!(p.expr, ConstExpr::Bin(..) | ConstExpr::Ternary(..)));
        if let Some(c) = chained {
            let rendered = expr_to_sv(&c.expr);
            assert!(
                rendered.contains('(') && rendered.contains(' '),
                "chained decl must render its symbolic expr, got {rendered}"
            );
            assert!(
                sv.contains(&format!("{} = {}", c.name, rendered)),
                "SV must carry the un-resolved expr for {}",
                c.name
            );
        }
    }

    /// The manifest is valid JSON, schema-shaped, and **every fact
    /// equals the `.2a` oracle** — params/localparams `value` ==
    /// `ParamDecl.value`, `expr` == the SV-printed expr, `widths`/
    /// `generate`/`package_constants` consistent with the emitter.
    #[test]
    fn manifest_mirrors_the_oracle() {
        let unit = build_constexpr_unit(42, 9);
        let json = emit_manifest(&unit, 42);
        let v: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
        assert_eq!(v["seed"], 42);
        assert_eq!(v["top"], "mc_42");
        for p in &unit.params {
            let bucket = match p.kind {
                ParamKind::Parameter => "params",
                ParamKind::Localparam => "localparams",
            };
            let e = &v[bucket][&p.name];
            assert_eq!(
                e["value"].as_i64().unwrap() as i128,
                p.value,
                "manifest {bucket}.{}.value must equal the oracle",
                p.name
            );
            assert_eq!(e["expr"].as_str().unwrap(), expr_to_sv(&p.expr));
        }
        // widths.sig.bits == (last % 8) + 1 (resolved from the oracle).
        let last = unit.params.last().unwrap();
        let want_bits = (last.value.rem_euclid(8) + 1) as i64;
        assert_eq!(v["widths"]["sig"]["bits"].as_i64().unwrap(), want_bits);
        assert_eq!(v["widths"]["sig"]["msb"].as_i64().unwrap(), want_bits - 1);
        // generate.g_taken.taken == (P0 >= pkg_const) from the oracle.
        let k = pkg_const(42);
        assert_eq!(
            v["generate"]["g_taken"]["taken"].as_bool().unwrap(),
            unit.params[0].value >= k
        );
        assert_eq!(
            v["package_constants"]["mc_42_pkg::K"].as_i64().unwrap() as i128,
            k
        );
        assert_eq!(
            v["const_exprs"].as_array().unwrap().len(),
            unit.params.len()
        );
    }

    /// `(seed) → .sv` and `→ .json` are byte-identical across rebuilds
    /// (the reproducibility contract; the manifest is part of the
    /// reproducible artifact). Distinct seeds differ.
    #[test]
    fn sv_and_manifest_are_byte_reproducible() {
        for seed in [0u64, 1, 7, 42, 999] {
            let u = build_constexpr_unit(seed, 8);
            assert_eq!(
                emit_sv(&u, seed),
                emit_sv(&build_constexpr_unit(seed, 8), seed)
            );
            assert_eq!(
                emit_manifest(&u, seed),
                emit_manifest(&build_constexpr_unit(seed, 8), seed)
            );
        }
        assert_ne!(
            emit_sv(&build_constexpr_unit(1, 8), 1),
            emit_sv(&build_constexpr_unit(2, 8), 2)
        );
    }
}
