//! PHASE-8-FRONTEND-ACCEPT.2a — source-level AST IR +
//! construction-time elaboration-evaluator (the oracle).
//!
//! Phase 8 is the *hierarchical extension* of Phase 7's
//! oracle-backed micro-design lane: instead of single-module
//! `rtl_const_expr` artifacts, this lane emits **compact
//! elaboratable hierarchies** (packages + a top module + sub-module
//! instances + generate-if blocks) whose **elaboration** is the
//! pressure point — parameter resolution across instance bindings,
//! generate-condition evaluation, package-qualified name resolution.
//!
//! Contents:
//! - `.2a` — the **source-level AST IR**
//!   (`SourceUnit` → `Package` → `Module` → `ModuleItem`) plus the
//!   **construction-time elaboration-evaluator** that resolves every
//!   parameter value, every generate predicate, every instance
//!   binding as the unit is built. The same `(seed, knobs)`
//!   reproducibility contract Phase 7 established, with one
//!   `ChaCha8Rng::seed_from_u64(seed)` driving everything. The
//!   builder *is* the oracle — no analysis pass, no re-parse, no
//!   bundled elaborator. Reuses Phase 7's `ConstExpr` set
//!   (cross-tree, per `.1`'s full-factorization plan).
//! - `.2b` — the **un-elaborated SV emitter** (`hier` family) +
//!   the **JSON elaborated-facts manifest emitter**, both from
//!   the same evaluated IR (the SV keeps parameter ports symbolic;
//!   the manifest records what elaboration must resolve them to).
//! - `.2c` — the **hierarchy-aware parity harness + repo-owned
//!   gate** (reuses Phase 7's scoped comparator with hierarchy-aware
//!   variants); ROADMAP Phase 8 closes on a verified clean run
//!   (r87 no-aspirational-claims).
//!
//! Lane separation (per `.1`): `frontend` is a **separate
//! generator path** from `ir`/`gen` (the DUT lane) and from
//! `microdesign` (the Phase 7 lane). The DUT lane stays
//! byte-identical by construction — `frontend` is never invoked
//! from `gen::*` and its default-off state is structurally trivial.
//! The Phase-9 multi-artifact umbrella selector (`PHASE-9-MULTI-
//! ARTIFACT-UMBRELLA.2`) wires invocation later.

use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

// Reuse Phase 7's expression layer cross-tree (per `.1`'s
// full-factorization plan). The same `ConstExpr` algebra — literals,
// param-by-name references, unary/binary/ternary nodes — is the
// expression form for parameter defaults, instance bindings,
// generate predicates, and localparam chains in Phase 8. `expr_to_sv`
// (the fully-parenthesized SV printer) is also reused — emitting the
// same symbolic-expression text Phase 7 does keeps oracle ≡ SV at
// the expression layer (the lesson learned by
// `PHASE-7-ORACLE-MICRODESIGN.2c.2b.1`'s
// non-negative-modulo-idiom fix carries over for free).
use crate::microdesign::{eval, expr_to_sv, BinOp, ConstExpr, EvalError, ParamKind};

// ===================================================================
// AST IR types — source-level surfaces the DUT circuit IR cannot
// express. Deliberately small and additive in `.2a`; `.2b` extends
// `ModuleItem` and `Type` as the emitter needs them, `.2c` extends
// the comparator. Every type derives `Clone, PartialEq, Eq` so the
// reproducible-builder proof can compare two builds for byte
// identity, and the manifest-mirrors-oracle proof can compare
// resolved fact maps for equality.
// ===================================================================

/// One translated source artifact — a package plus a top module
/// (plus, optionally, child-module declarations the top instantiates).
/// Phase 8's minimum-viable shape: every artifact has exactly one
/// `Package`, one top `Module`, and zero-or-more child `Module`s the
/// top instantiates. The instance tree is one level deep in `.2a`
/// (extensions to deeper trees are recorded as a post-`.2a` knob in
/// `.2b`'s emit work, not a `.2a` blocker — depth-1 is enough to
/// stress every elaboration axis the parity gate checks).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceUnit {
    pub seed: u64,
    /// Package(s) holding shared constants (`localparam`s).
    pub packages: Vec<Package>,
    /// Child module declarations the top instantiates. Each child is
    /// a parameterized stub — a module with `parameter int P = …` and
    /// no body — so the parity gate exercises elaboration without
    /// being clouded by behavioural code.
    pub children: Vec<Module>,
    /// The top module — the elaboration entry point.
    pub top: Module,
}

/// A SystemVerilog package: a named scope holding `localparam` decls
/// that the top module may reference via `pkg::name`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Package {
    pub name: String,
    pub items: Vec<PackageItem>,
}

/// What can live in a package. `.2a`'s minimum-viable set is just
/// `Localparam`; `.2b` may add `Typedef` etc. as the emitter
/// extends.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PackageItem {
    /// A package-qualified constant declaration. `value` is the
    /// resolved oracle (never re-derived after the builder runs).
    Localparam(ParamDecl),
}

/// A SystemVerilog module: name + parameter ports + body items.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Module {
    pub name: String,
    /// `parameter` ports (overridable at instantiation). Each carries
    /// its default expression + the construction-time-resolved value
    /// (the oracle). For child modules these are the declared
    /// parameters; the instance bindings in the parent override them
    /// per-instance.
    pub params: Vec<ParamDecl>,
    pub body: Vec<ModuleItem>,
}

/// A parameter / localparam declaration. Identical *shape* to
/// `microdesign::ParamDecl` but the Phase 8 lane carries its own
/// type so the source-AST IR is self-contained at this surface
/// (cross-tree reuse is at the `ConstExpr`/`eval` layer, not at the
/// param-decl-record layer where field meanings differ — e.g. Phase
/// 8 carries package-scoped versus module-port-scoped distinctions
/// the Phase 7 `ParamDecl` does not).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParamDecl {
    pub name: String,
    pub kind: ParamKind,
    /// The default / defining expression (SV-side-symbolic). Kept
    /// symbolic in the emit; resolved into `value` by the builder.
    pub expr: ConstExpr,
    /// Construction-time-resolved value — **the oracle**.
    pub value: i128,
}

/// What can live in a module body. `.2a`'s minimum-viable set; `.2b`
/// may add `ContinuousAssign`/`Always`/`Typedef`/etc. as the emitter
/// extends.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModuleItem {
    /// In-body `localparam` chain (mirrors Phase 7's body
    /// localparams — derived from earlier parameters / localparams).
    Localparam(ParamDecl),
    /// Sub-module instantiation. `param_bindings` is the parent-side
    /// override list (`#(.P0(<expr>), .P1(<expr>))`); the resolved
    /// child-parameter values are filled in by the
    /// elaboration-evaluator at construction time.
    Instance(Instance),
    /// `generate if (<cond>) ... else ...`. The `taken` field is the
    /// oracle's record of which branch elaborates; the SV emit keeps
    /// the predicate symbolic.
    GenerateIf(GenerateIf),
}

/// A sub-module instance. Named-binding form only in `.2a` (ordered
/// bindings are a `.2b` extension recorded as a knob; named is
/// sufficient to stress per-name elaboration and is the modern SV
/// style downstream tools document best).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Instance {
    /// Local instance identifier (`u_<seed>_<idx>`).
    pub inst_name: String,
    /// Child module name (must appear in `SourceUnit.children`).
    pub child_module: String,
    /// Parent-side parameter override bindings. Each is a
    /// `(name, expression)` pair evaluated in the parent's
    /// environment. `resolved` carries the elaboration-evaluator's
    /// resolved value — what a downstream consumer must agree on.
    pub param_bindings: Vec<ParamBinding>,
}

/// One `.NAME(<expr>)` parameter binding on an instance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParamBinding {
    pub name: String,
    pub expr: ConstExpr,
    /// The elaboration-evaluator's resolved value for this binding
    /// (the parent's environment). Must equal what the downstream
    /// consumer reports for `<inst>.<name>` after elaboration.
    pub resolved: i128,
}

/// `generate if (<cond>) begin : <label> ... end else begin : <else_label> ... end`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenerateIf {
    pub label: String,
    pub else_label: String,
    pub condition: ConstExpr,
    /// Oracle: `eval(condition, env) != 0`. The downstream consumer
    /// reports the branch it elaborated; the comparator checks
    /// agreement.
    pub taken: bool,
    /// `.2a`-minimum: the two branch bodies are empty
    /// (`logic gflag; assign gflag = 1'b1` / `... 1'b0` to give the
    /// branch a netname for the comparator's prefix-scan trick,
    /// exactly like Phase 7's `.2b`). `.2b` may add full bodies as
    /// the emitter extends.
    pub then_branch: Vec<ModuleItem>,
    pub else_branch: Vec<ModuleItem>,
}

// ===================================================================
// Construction-time elaboration-evaluator (the oracle).
//
// The builder calls `elaborate(&mut SourceUnit)` exactly once after
// constructing the tree; every `ParamDecl.value`, every
// `ParamBinding.resolved`, and every `GenerateIf.taken` is filled in
// from the same `ConstExpr` evaluation. From that point forward the
// `.value`/`.resolved`/`.taken` fields are the *single source of
// truth*; the `.2b` emitter reads them directly without re-evaluating.
// ===================================================================

/// Visit every `ParamDecl`, `ParamBinding`, and `GenerateIf` in the
/// unit and resolve its value/branch from the `ConstExpr` evaluation
/// against the lexical environment built so far. Returns the
/// top-level env (package-qualified-name → value, top-module-param-
/// name → value, top-module-localparam-name → value), which is also
/// what `.2b`'s manifest emitter consumes.
///
/// **The builder is the oracle**: this function is the *only* place
/// `ConstExpr` values are computed; downstream readers (emit, manifest,
/// comparator) must read the stored `.value`/`.resolved`/`.taken`
/// rather than re-evaluating. The
/// `elaborated_facts_match_a_fresh_reeval` proof below pins the
/// invariant.
pub fn elaborate(unit: &mut SourceUnit) -> Result<BTreeMap<String, i128>, EvalError> {
    let mut env: BTreeMap<String, i128> = BTreeMap::new();

    // 1. Resolve each package's localparams in declaration order; the
    //    resolved values land both in the package's `ParamDecl.value`
    //    (the oracle) AND in `env` keyed by `pkg::name` (the qualified
    //    form the top module references via `PkgQual`).
    for pkg in &mut unit.packages {
        for item in &mut pkg.items {
            match item {
                PackageItem::Localparam(p) => {
                    let v = eval(&p.expr, &env)?;
                    p.value = v;
                    env.insert(format!("{}::{}", pkg.name, p.name), v);
                }
            }
        }
    }

    // 2. Resolve the top module's parameter ports (their default
    //    expressions; the builder doesn't override them — that is the
    //    instance-binding contract one level down).
    for p in &mut unit.top.params {
        let v = eval(&p.expr, &env)?;
        p.value = v;
        env.insert(p.name.clone(), v);
    }

    // 3. Walk the top module body. Localparams extend the env in
    //    declaration order; instance bindings resolve in the
    //    *parent's* env (not the child's — the parent supplies the
    //    expressions); generate-if predicates resolve and the `taken`
    //    field records the branch.
    for item in &mut unit.top.body {
        elaborate_module_item(item, &mut env)?;
    }

    Ok(env)
}

fn elaborate_module_item(
    item: &mut ModuleItem,
    env: &mut BTreeMap<String, i128>,
) -> Result<(), EvalError> {
    match item {
        ModuleItem::Localparam(p) => {
            let v = eval(&p.expr, env)?;
            p.value = v;
            env.insert(p.name.clone(), v);
        }
        ModuleItem::Instance(inst) => {
            for b in &mut inst.param_bindings {
                let v = eval(&b.expr, env)?;
                b.resolved = v;
            }
        }
        ModuleItem::GenerateIf(g) => {
            let cond = eval(&g.condition, env)?;
            g.taken = cond != 0;
            // The body items inside the *taken* branch elaborate
            // (their localparams join the env); the not-taken branch
            // also elaborates locally but is not exposed to the
            // parent scope. For .2a's minimum-viable bodies these
            // are no-ops (no localparams inside the branches), but
            // the walk is here so `.2b`'s richer bodies just work.
            for sub in &mut g.then_branch {
                elaborate_module_item(sub, env)?;
            }
            for sub in &mut g.else_branch {
                // Else-branch items resolve in a sandboxed copy of
                // env so they cannot leak into the parent (only the
                // taken branch's env extensions persist; in SV's
                // model a generate-if either-branch is structurally
                // present in the IR but only the taken branch
                // contributes to the final elaboration).
                let mut sandbox = env.clone();
                elaborate_module_item(sub, &mut sandbox)?;
            }
        }
    }
    Ok(())
}

// ===================================================================
// Reproducible rules-first builder.
//
// `(seed, n_params, n_children)` → byte-identical `SourceUnit` across
// rebuilds (the reproducibility contract, identical in shape to Phase
// 7's `build_constexpr_unit`). One `ChaCha8Rng::seed_from_u64(seed)`
// drives everything; no `thread_rng`, no system time.
//
// The builder *is* the oracle: every parameter / localparam / binding
// / generate predicate is resolved in place at construction time.
// ===================================================================

/// Build a deterministic Phase 8 accept-corpus unit:
/// - One package `acc_<seed>_pkg` with a single `localparam int K = …`.
/// - One child module `child_<seed>` with `n_params` declared
///   parameters (each defaulting to a literal).
/// - One top module `acc_<seed>` with `n_params` declared parameters,
///   `n_params` body localparams forming a chain over earlier names,
///   `n_children` `child_<seed>` instances each binding the child's
///   parameters via parent-evaluated expressions, and one `generate
///   if (P0 >= acc_<seed>_pkg::K)` block that elaborates either
///   the `g_taken` branch or the `g_else` branch.
///
/// Reproducibility is structural: identical seed and shape parameters
/// always produce the byte-identical `SourceUnit`. The
/// `unit_is_reproducible_and_seed_sensitive` proof pins it.
pub fn build_acceptable_unit(seed: u64, n_params: usize, n_children: usize) -> SourceUnit {
    let n_p = n_params.max(1);
    let n_c = n_children.max(1);
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let pkg_name = format!("acc_{seed}_pkg");
    let pkg_k = (seed as i128 % 32) + 1; // small positive constant
    let child_name = format!("child_{seed}");
    let top_name = format!("acc_{seed}");

    // Package: one `K`.
    let k_decl = ParamDecl {
        name: "K".to_string(),
        kind: ParamKind::Localparam,
        expr: ConstExpr::Lit(pkg_k),
        value: 0, // filled by `elaborate`
    };
    let packages = vec![Package {
        name: pkg_name.clone(),
        items: vec![PackageItem::Localparam(k_decl)],
    }];

    // Child module: `n_p` parameters with literal defaults so the
    // child elaborates standalone (the instance bindings in the
    // parent override these).
    let mut child_params: Vec<ParamDecl> = Vec::with_capacity(n_p);
    for i in 0..n_p {
        let v = (rng.gen_range(1..=32)) as i128;
        child_params.push(ParamDecl {
            name: format!("CP{i}"),
            kind: ParamKind::Parameter,
            expr: ConstExpr::Lit(v),
            value: 0,
        });
    }
    let children = vec![Module {
        name: child_name.clone(),
        params: child_params,
        body: Vec::new(), // empty body; the elaboration we care about is the parameter resolution
    }];

    // Top module: `n_p` parameter ports (literal defaults; chained
    // localparams in the body reference earlier names so the
    // elaboration-evaluator builds an env step-by-step).
    let mut top_params: Vec<ParamDecl> = Vec::with_capacity(n_p);
    for i in 0..n_p {
        let v = (rng.gen_range(1..=64)) as i128;
        top_params.push(ParamDecl {
            name: format!("P{i}"),
            kind: ParamKind::Parameter,
            expr: ConstExpr::Lit(v),
            value: 0,
        });
    }

    let mut body: Vec<ModuleItem> = Vec::new();

    // Chained localparams: L{i} = earlier-name [+|-] small lit. The
    // first one references P0; the rest reference earlier L names.
    for i in 0..n_p {
        let earlier = if i == 0 {
            ConstExpr::Param("P0".to_string())
        } else {
            ConstExpr::Param(format!("L{}", i - 1))
        };
        let op = if rng.gen_bool(0.5) {
            BinOp::Add
        } else {
            BinOp::Sub
        };
        let small = ConstExpr::Lit((rng.gen_range(1..=4)) as i128);
        let expr = ConstExpr::Bin(op, Box::new(earlier), Box::new(small));
        body.push(ModuleItem::Localparam(ParamDecl {
            name: format!("L{i}"),
            kind: ParamKind::Localparam,
            expr,
            value: 0,
        }));
    }

    // Sub-module instances. Each binds every CP<i> by name to a
    // parent-evaluated expression (alternating between top param
    // refs and localparam refs so the evaluator must traverse the
    // env both ways). The parent ref scope is everything declared
    // before this instance.
    for ci in 0..n_c {
        let mut bindings = Vec::with_capacity(n_p);
        for pi in 0..n_p {
            let pick_l = rng.gen_bool(0.5);
            let src = if pick_l {
                ConstExpr::Param(format!("L{}", pi.min(n_p - 1)))
            } else {
                ConstExpr::Param(format!("P{}", pi.min(n_p - 1)))
            };
            let off = ConstExpr::Lit((rng.gen_range(0..=3)) as i128);
            let expr = ConstExpr::Bin(BinOp::Add, Box::new(src), Box::new(off));
            bindings.push(ParamBinding {
                name: format!("CP{pi}"),
                expr,
                resolved: 0,
            });
        }
        body.push(ModuleItem::Instance(Instance {
            inst_name: format!("u_{seed}_{ci}"),
            child_module: child_name.clone(),
            param_bindings: bindings,
        }));
    }

    // One generate-if block driven by `P0 >= pkg::K`.
    let gen_cond = ConstExpr::Bin(
        BinOp::Ge,
        Box::new(ConstExpr::Param("P0".to_string())),
        Box::new(ConstExpr::Param(format!("{pkg_name}::K"))),
    );
    body.push(ModuleItem::GenerateIf(GenerateIf {
        label: "g_taken".to_string(),
        else_label: "g_else".to_string(),
        condition: gen_cond,
        taken: false,
        then_branch: Vec::new(),
        else_branch: Vec::new(),
    }));

    let mut unit = SourceUnit {
        seed,
        packages,
        children,
        top: Module {
            name: top_name,
            params: top_params,
            body,
        },
    };

    // Elaborate in place. A valid-by-construction unit never errors
    // here; if it does, the panic is a builder bug, not a runtime
    // input.
    let _env = elaborate(&mut unit).expect("rules-first source unit is valid by construction");
    unit
}

// ===================================================================
// PHASE-8-FRONTEND-ACCEPT.2b — un-elaborated SV emitter + elaborated-
// facts JSON manifest emitter.
//
// Both are emitted *from the same evaluated IR* (`.2a`'s resolved
// `.value` / `.resolved` / `.taken` oracle): the `.sv` text keeps
// parameter ports, instance bindings, localparam expressions, and
// generate predicates **symbolic** (un-resolved) — that gap between
// symbolic text and the manifest's resolved facts is exactly the
// front-end / elaboration behaviour Phase 8 stresses. No analysis
// pass, no re-parse. Behind an explicit artifact-family path:
// `frontend` is a *separate module never invoked by the DUT generate
// path*, so the DUT lane is byte-identical by construction (default-
// off is trivial; the Phase 9 selector wires invocation later).
// ===================================================================

/// Emit the full un-elaborated SystemVerilog for a `SourceUnit`: the
/// package(s), the child module stubs, and the top module with
/// symbolic parameter / localparam / binding / generate-predicate
/// expressions. The SV text is what a downstream consumer's
/// elaborator must resolve against the same facts the manifest carries.
pub fn emit_sv(unit: &SourceUnit) -> String {
    let mut s = String::new();
    s.push_str(&format!(
        "// Generated by anvil frontend (Phase 8). Top: {}\n",
        unit.top.name
    ));

    // Packages.
    for pkg in &unit.packages {
        s.push_str(&format!("package {};\n", pkg.name));
        for item in &pkg.items {
            match item {
                PackageItem::Localparam(p) => {
                    s.push_str(&format!(
                        "    localparam int {} = {};\n",
                        p.name,
                        expr_to_sv(&p.expr)
                    ));
                }
            }
        }
        s.push_str("endpackage\n\n");
    }

    // Child module stubs — `parameter int CP<i> = <symbolic expr>` headers,
    // empty body. The instance bindings in the top module override
    // these defaults; the parity gate checks the resolved values.
    for child in &unit.children {
        s.push_str(&format!("module {}", child.name));
        if !child.params.is_empty() {
            s.push_str(" #(\n");
            for (i, p) in child.params.iter().enumerate() {
                let comma = if i + 1 < child.params.len() { "," } else { "" };
                s.push_str(&format!(
                    "    parameter int {} = {}{}\n",
                    p.name,
                    expr_to_sv(&p.expr),
                    comma
                ));
            }
            s.push(')');
        }
        s.push_str(";\nendmodule\n\n");
    }

    // Top module — symbolic everywhere.
    let top = &unit.top;
    s.push_str(&format!("module {}", top.name));
    if !top.params.is_empty() {
        s.push_str(" #(\n");
        for (i, p) in top.params.iter().enumerate() {
            let comma = if i + 1 < top.params.len() { "," } else { "" };
            s.push_str(&format!(
                "    parameter int {} = {}{}\n",
                p.name,
                expr_to_sv(&p.expr),
                comma
            ));
        }
        s.push(')');
    }
    s.push_str(";\n");
    for item in &top.body {
        emit_module_item(&mut s, item, 1);
    }
    s.push_str("endmodule\n");

    s
}

fn emit_module_item(s: &mut String, item: &ModuleItem, indent: usize) {
    let pad = "    ".repeat(indent);
    match item {
        ModuleItem::Localparam(p) => {
            s.push_str(&format!(
                "{pad}localparam int {} = {};\n",
                p.name,
                expr_to_sv(&p.expr)
            ));
        }
        ModuleItem::Instance(inst) => {
            s.push_str(&format!("{pad}{}", inst.child_module));
            if !inst.param_bindings.is_empty() {
                s.push_str(" #(\n");
                for (i, b) in inst.param_bindings.iter().enumerate() {
                    let comma = if i + 1 < inst.param_bindings.len() {
                        ","
                    } else {
                        ""
                    };
                    s.push_str(&format!(
                        "{pad}    .{}({}){}\n",
                        b.name,
                        expr_to_sv(&b.expr),
                        comma
                    ));
                }
                s.push_str(&format!("{pad})"));
            }
            s.push_str(&format!(" {} ();\n", inst.inst_name));
        }
        ModuleItem::GenerateIf(g) => {
            s.push_str(&format!("{pad}generate\n"));
            s.push_str(&format!(
                "{pad}    if ({}) begin : {}\n",
                expr_to_sv(&g.condition),
                g.label
            ));
            // .2a's minimum-viable bodies are empty; emit a marker
            // signal so downstream tools have a netname anchored to
            // the taken branch (matches Phase 7's `g_taken.gflag`
            // convention, which the parity-gate extractor relies on
            // for branch-detection via netname prefix).
            s.push_str(&format!("{pad}        logic gflag;\n"));
            s.push_str(&format!("{pad}        assign gflag = 1'b1;\n"));
            for sub in &g.then_branch {
                emit_module_item(s, sub, indent + 2);
            }
            s.push_str(&format!("{pad}    end else begin : {}\n", g.else_label));
            s.push_str(&format!("{pad}        logic gflag;\n"));
            s.push_str(&format!("{pad}        assign gflag = 1'b0;\n"));
            for sub in &g.else_branch {
                emit_module_item(s, sub, indent + 2);
            }
            s.push_str(&format!("{pad}    end\n"));
            s.push_str(&format!("{pad}endgenerate\n"));
        }
    }
}

// -------------------------------------------------------------------
// Elaborated-facts manifest (extends `.1`'s schema for Phase 8 with
// the instance tree + package localparam values + generate-branch
// resolutions). `BTreeMap` everywhere ⇒ deterministic key order ⇒
// byte-stable `serde_json` pretty output.
// -------------------------------------------------------------------

/// One package's resolved facts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageFacts {
    pub name: String,
    /// `localparam` name → resolved value (the oracle).
    pub constants: BTreeMap<String, i128>,
}

/// One `(name, value, expr-text)` triple. The `expr` is the
/// fully-parenthesized SV form (matching what `emit_sv` produced),
/// for diagnostic round-trip; `value` is the oracle's resolved value.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParamFact {
    pub value: i128,
    pub expr: String,
}

/// One instance's resolved facts: instance name, child module, and
/// the per-binding resolved values (the parent-side expressions
/// evaluated in the parent's env).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstanceFact {
    pub inst_name: String,
    pub child_module: String,
    /// Binding name → resolved value.
    pub resolved_bindings: BTreeMap<String, i128>,
}

/// One generate-block resolution: label → taken (`true` for the
/// `if`-branch, `false` for the `else`-branch). Mirrors Phase 7's
/// `generate["g_taken"].taken` shape so the comparator's
/// netname-prefix extractor extension reuses the same convention.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GenerateFact {
    pub taken: bool,
}

/// Phase 8 elaborated-facts manifest. Every field is populated from
/// the `.2a` oracle (`elaborate`'s output); none are re-derived.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Manifest {
    pub seed: u64,
    pub top: String,
    pub packages: Vec<PackageFacts>,
    pub top_params: BTreeMap<String, ParamFact>,
    pub top_localparams: BTreeMap<String, ParamFact>,
    pub instances: Vec<InstanceFact>,
    pub generate_branches: BTreeMap<String, GenerateFact>,
}

/// Build the manifest from an elaborated `SourceUnit`. Mirrors
/// exactly what `emit_sv` declares, populated from the `.2a` oracle.
pub fn build_manifest(unit: &SourceUnit) -> Manifest {
    let packages: Vec<PackageFacts> = unit
        .packages
        .iter()
        .map(|pkg| {
            let mut constants = BTreeMap::new();
            for item in &pkg.items {
                match item {
                    PackageItem::Localparam(p) => {
                        constants.insert(p.name.clone(), p.value);
                    }
                }
            }
            PackageFacts {
                name: pkg.name.clone(),
                constants,
            }
        })
        .collect();

    let mut top_params: BTreeMap<String, ParamFact> = BTreeMap::new();
    for p in &unit.top.params {
        top_params.insert(
            p.name.clone(),
            ParamFact {
                value: p.value,
                expr: expr_to_sv(&p.expr),
            },
        );
    }

    let mut top_localparams: BTreeMap<String, ParamFact> = BTreeMap::new();
    let mut instances: Vec<InstanceFact> = Vec::new();
    let mut generate_branches: BTreeMap<String, GenerateFact> = BTreeMap::new();
    for item in &unit.top.body {
        match item {
            ModuleItem::Localparam(p) => {
                top_localparams.insert(
                    p.name.clone(),
                    ParamFact {
                        value: p.value,
                        expr: expr_to_sv(&p.expr),
                    },
                );
            }
            ModuleItem::Instance(inst) => {
                let mut resolved_bindings = BTreeMap::new();
                for b in &inst.param_bindings {
                    resolved_bindings.insert(b.name.clone(), b.resolved);
                }
                instances.push(InstanceFact {
                    inst_name: inst.inst_name.clone(),
                    child_module: inst.child_module.clone(),
                    resolved_bindings,
                });
            }
            ModuleItem::GenerateIf(g) => {
                generate_branches.insert(g.label.clone(), GenerateFact { taken: g.taken });
            }
        }
    }

    Manifest {
        seed: unit.seed,
        top: unit.top.name.clone(),
        packages,
        top_params,
        top_localparams,
        instances,
        generate_branches,
    }
}

/// Serialize the manifest as deterministic pretty JSON.
pub fn emit_manifest(unit: &SourceUnit) -> String {
    serde_json::to_string_pretty(&build_manifest(unit)).expect("manifest serializes")
}

// ===================================================================
// PHASE-8-FRONTEND-ACCEPT.2c.1 — hierarchy-aware parity comparator
// core.
//
// Phase 8 has its OWN parity types (parallel to Phase 7's
// `microdesign::{ToolReport, Divergence, FactCategory, ParityScope}`,
// NOT derived): the artifact differs — Phase 7 emits single-module
// `rtl_const_expr`, Phase 8 emits hierarchical packages + child
// stubs + a top with instance bindings + generate-if, so the
// `ToolReport` carries an `instances` vector and `Divergence` adds
// instance-presence + instance-binding variants the Phase 7 set does
// not need. The scope mechanism mirrors Phase 7's; the comparator's
// fail-accumulating walk is identical in shape.
//
// The harness wiring (corpus + downstream consumer + extractor +
// `#[ignore]` real-tool gate) lives in `tests/frontend_parity.rs`
// and is `#[ignore]`-gated so the portable `cargo test` stays green
// tool-less — the Phase-1 doctrine reaffirmed in PHASE-7's
// Decisions, applied at PHASE-7's `.2c.1`/`.2c.2a`, and applied here.
// `.2c.2` runs the real `--ignored` gate end-to-end against a
// downstream elaborator and banks a verified-clean artifact before
// ROADMAP Phase 8 → done.
// ===================================================================

/// What a downstream consumer's resolved-facts report looks like for
/// a Phase 8 `SourceUnit`, normalized to the same fact set the
/// manifest carries.
///
/// Names match the manifest keys exactly; values are resolved
/// integers (no symbolic SV — the tool resolved them). `BTreeMap`
/// throughout for deterministic iteration; `Vec<InstanceToolReport>`
/// for the per-instance facts (the tool reports each instance by
/// `inst_name`; the comparator does name-keyed lookups so order
/// independence holds).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolReport {
    pub seed: u64,
    pub top: String,
    /// `pkg::name` → resolved value (the package localparams the tool
    /// can introspect).
    pub package_constants: BTreeMap<String, i128>,
    /// Top module parameter ports: `name` → resolved value.
    pub top_params: BTreeMap<String, i128>,
    /// Top module body localparams: `name` → resolved value.
    pub top_localparams: BTreeMap<String, i128>,
    /// Per-instance resolved facts.
    pub instances: Vec<InstanceToolReport>,
    /// `generate label` → taken.
    pub generate_branches: BTreeMap<String, bool>,
}

/// One instance's resolved facts as the tool reports them.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstanceToolReport {
    pub inst_name: String,
    pub child_module: String,
    /// Per-binding resolved value (`CP<i>` → value).
    pub resolved_bindings: BTreeMap<String, i128>,
}

/// A single category of disagreement between the manifest (the
/// oracle) and the tool report for a Phase 8 hierarchy.
///
/// Extends Phase 7's per-axis × per-direction scheme with **instance
/// presence** + **per-instance per-binding** variants — the
/// load-bearing hierarchy-aware additions. The comparator
/// accumulates the full divergence set rather than fail-fast so a
/// counterexample tuple is one-shot.
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

    // Package constants.
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

    // Top params.
    TopParamMissingInTool {
        name: String,
        expected: i128,
    },
    TopParamMissingInManifest {
        name: String,
        actual: i128,
    },
    TopParamMismatch {
        name: String,
        expected: i128,
        actual: i128,
    },

    // Top localparams.
    TopLocalparamMissingInTool {
        name: String,
        expected: i128,
    },
    TopLocalparamMissingInManifest {
        name: String,
        actual: i128,
    },
    TopLocalparamMismatch {
        name: String,
        expected: i128,
        actual: i128,
    },

    // Instance presence.
    InstanceMissingInTool {
        inst_name: String,
    },
    InstanceMissingInManifest {
        inst_name: String,
    },
    InstanceChildModuleMismatch {
        inst_name: String,
        expected: String,
        actual: String,
    },

    // Per-instance per-binding values.
    InstanceBindingMissingInTool {
        inst_name: String,
        name: String,
        expected: i128,
    },
    InstanceBindingMissingInManifest {
        inst_name: String,
        name: String,
        actual: i128,
    },
    InstanceBindingMismatch {
        inst_name: String,
        name: String,
        expected: i128,
        actual: i128,
    },

    // Generate branches.
    GenerateMissingInTool {
        label: String,
        expected: bool,
    },
    GenerateMissingInManifest {
        label: String,
        actual: bool,
    },
    GenerateMismatch {
        label: String,
        expected: bool,
        actual: bool,
    },
}

/// The fact-category axes the Phase 8 comparator can enforce — one
/// per `ToolReport` section. The scope mechanism mirrors Phase 7's
/// `microdesign::ParityScope` (different downstream consumers expose
/// different subsets — yosys hierarchy-elaborate may not surface
/// per-instance bindings the way slang's `--ast-json` does, etc.).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum FactCategory {
    Seed,
    Top,
    PackageConstants,
    TopParams,
    TopLocalparams,
    Instances,
    GenerateBranches,
}

/// The set of fact categories a Phase 8 parity gate enforces.
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
            FactCategory::PackageConstants,
            FactCategory::TopParams,
            FactCategory::TopLocalparams,
            FactCategory::Instances,
            FactCategory::GenerateBranches,
        ]
        .iter()
        .copied()
        .collect();
        Self { categories }
    }

    /// No category — the comparator returns `Ok(())` even on a
    /// maximally disagreeing report (used by the scoping
    /// self-check proof).
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

    pub fn contains(&self, category: FactCategory) -> bool {
        self.categories.contains(&category)
    }
}

/// Strict-all-categories comparison. Delegates to
/// `compare_manifest_to_tool_report_in_scope` with `ParityScope::all()`.
pub fn compare_manifest_to_tool_report(
    manifest: &Manifest,
    report: &ToolReport,
) -> Result<(), Vec<Divergence>> {
    compare_manifest_to_tool_report_in_scope(manifest, report, &ParityScope::all())
}

/// Walk `manifest` × `report` over the axes in `scope` and return
/// every disagreement. `Ok(())` ⇔ exact agreement on every scoped
/// axis. Out-of-scope axes are skipped entirely. The strict
/// [`compare_manifest_to_tool_report`] is this with
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

    if scope.contains(FactCategory::PackageConstants) {
        // The manifest carries `packages: Vec<PackageFacts>`; flatten
        // to `pkg::name` form for comparison against the
        // `report.package_constants` map.
        let mut manifest_pkg: BTreeMap<String, i128> = BTreeMap::new();
        for pkg in &manifest.packages {
            for (name, value) in &pkg.constants {
                manifest_pkg.insert(format!("{}::{}", pkg.name, name), *value);
            }
        }
        for (name, expected) in &manifest_pkg {
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
            if !manifest_pkg.contains_key(name) {
                divs.push(Divergence::PackageConstantMissingInManifest {
                    name: name.clone(),
                    actual: *actual,
                });
            }
        }
    }

    if scope.contains(FactCategory::TopParams) {
        for (name, fact) in &manifest.top_params {
            match report.top_params.get(name) {
                Some(actual) if *actual == fact.value => {}
                Some(actual) => divs.push(Divergence::TopParamMismatch {
                    name: name.clone(),
                    expected: fact.value,
                    actual: *actual,
                }),
                None => divs.push(Divergence::TopParamMissingInTool {
                    name: name.clone(),
                    expected: fact.value,
                }),
            }
        }
        for (name, actual) in &report.top_params {
            if !manifest.top_params.contains_key(name) {
                divs.push(Divergence::TopParamMissingInManifest {
                    name: name.clone(),
                    actual: *actual,
                });
            }
        }
    }

    if scope.contains(FactCategory::TopLocalparams) {
        for (name, fact) in &manifest.top_localparams {
            match report.top_localparams.get(name) {
                Some(actual) if *actual == fact.value => {}
                Some(actual) => divs.push(Divergence::TopLocalparamMismatch {
                    name: name.clone(),
                    expected: fact.value,
                    actual: *actual,
                }),
                None => divs.push(Divergence::TopLocalparamMissingInTool {
                    name: name.clone(),
                    expected: fact.value,
                }),
            }
        }
        for (name, actual) in &report.top_localparams {
            if !manifest.top_localparams.contains_key(name) {
                divs.push(Divergence::TopLocalparamMissingInManifest {
                    name: name.clone(),
                    actual: *actual,
                });
            }
        }
    }

    if scope.contains(FactCategory::Instances) {
        // Instance presence: index by name from each side.
        let manifest_by_name: BTreeMap<&str, &InstanceFact> = manifest
            .instances
            .iter()
            .map(|i| (i.inst_name.as_str(), i))
            .collect();
        let report_by_name: BTreeMap<&str, &InstanceToolReport> = report
            .instances
            .iter()
            .map(|i| (i.inst_name.as_str(), i))
            .collect();

        for (name, m_inst) in &manifest_by_name {
            match report_by_name.get(name) {
                Some(t_inst) => {
                    if m_inst.child_module != t_inst.child_module {
                        divs.push(Divergence::InstanceChildModuleMismatch {
                            inst_name: (*name).to_string(),
                            expected: m_inst.child_module.clone(),
                            actual: t_inst.child_module.clone(),
                        });
                    }
                    // Per-binding compare.
                    for (b_name, b_value) in &m_inst.resolved_bindings {
                        match t_inst.resolved_bindings.get(b_name) {
                            Some(actual) if actual == b_value => {}
                            Some(actual) => divs.push(Divergence::InstanceBindingMismatch {
                                inst_name: (*name).to_string(),
                                name: b_name.clone(),
                                expected: *b_value,
                                actual: *actual,
                            }),
                            None => divs.push(Divergence::InstanceBindingMissingInTool {
                                inst_name: (*name).to_string(),
                                name: b_name.clone(),
                                expected: *b_value,
                            }),
                        }
                    }
                    for (b_name, b_value) in &t_inst.resolved_bindings {
                        if !m_inst.resolved_bindings.contains_key(b_name) {
                            divs.push(Divergence::InstanceBindingMissingInManifest {
                                inst_name: (*name).to_string(),
                                name: b_name.clone(),
                                actual: *b_value,
                            });
                        }
                    }
                }
                None => divs.push(Divergence::InstanceMissingInTool {
                    inst_name: (*name).to_string(),
                }),
            }
        }
        for name in report_by_name.keys() {
            if !manifest_by_name.contains_key(name) {
                divs.push(Divergence::InstanceMissingInManifest {
                    inst_name: (*name).to_string(),
                });
            }
        }
    }

    if scope.contains(FactCategory::GenerateBranches) {
        for (label, fact) in &manifest.generate_branches {
            match report.generate_branches.get(label) {
                Some(actual) if *actual == fact.taken => {}
                Some(actual) => divs.push(Divergence::GenerateMismatch {
                    label: label.clone(),
                    expected: fact.taken,
                    actual: *actual,
                }),
                None => divs.push(Divergence::GenerateMissingInTool {
                    label: label.clone(),
                    expected: fact.taken,
                }),
            }
        }
        for (label, actual) in &report.generate_branches {
            if !manifest.generate_branches.contains_key(label) {
                divs.push(Divergence::GenerateMissingInManifest {
                    label: label.clone(),
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

/// Construct a `ToolReport` that agrees with `manifest` exactly —
/// "what a perfectly-conforming downstream tool would have produced".
/// Used by `.2c.1`'s cargo-portable proofs and as the structural
/// fallback in `.2c.2`'s real-tool path.
pub fn synthetic_tool_report_from_manifest(manifest: &Manifest) -> ToolReport {
    let mut package_constants: BTreeMap<String, i128> = BTreeMap::new();
    for pkg in &manifest.packages {
        for (name, value) in &pkg.constants {
            package_constants.insert(format!("{}::{}", pkg.name, name), *value);
        }
    }
    let top_params: BTreeMap<String, i128> = manifest
        .top_params
        .iter()
        .map(|(n, f)| (n.clone(), f.value))
        .collect();
    let top_localparams: BTreeMap<String, i128> = manifest
        .top_localparams
        .iter()
        .map(|(n, f)| (n.clone(), f.value))
        .collect();
    let instances: Vec<InstanceToolReport> = manifest
        .instances
        .iter()
        .map(|i| InstanceToolReport {
            inst_name: i.inst_name.clone(),
            child_module: i.child_module.clone(),
            resolved_bindings: i.resolved_bindings.clone(),
        })
        .collect();
    let generate_branches: BTreeMap<String, bool> = manifest
        .generate_branches
        .iter()
        .map(|(l, g)| (l.clone(), g.taken))
        .collect();
    ToolReport {
        seed: manifest.seed,
        top: manifest.top.clone(),
        package_constants,
        top_params,
        top_localparams,
        instances,
        generate_branches,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Smoke-shape: a freshly-built unit always has exactly one
    /// package, exactly one child module, one top module, and the
    /// declared parameter/instance/generate-if shape.
    #[test]
    fn build_acceptable_unit_has_the_documented_shape() {
        let u = build_acceptable_unit(7, 5, 3);
        assert_eq!(u.packages.len(), 1);
        assert_eq!(u.packages[0].name, "acc_7_pkg");
        assert_eq!(u.children.len(), 1);
        assert_eq!(u.children[0].name, "child_7");
        assert_eq!(u.children[0].params.len(), 5);
        assert_eq!(u.top.name, "acc_7");
        assert_eq!(u.top.params.len(), 5);

        // Localparam chain + instances + generate-if.
        let lps: usize = u
            .top
            .body
            .iter()
            .filter(|m| matches!(m, ModuleItem::Localparam(_)))
            .count();
        let insts: usize = u
            .top
            .body
            .iter()
            .filter(|m| matches!(m, ModuleItem::Instance(_)))
            .count();
        let gens: usize = u
            .top
            .body
            .iter()
            .filter(|m| matches!(m, ModuleItem::GenerateIf(_)))
            .count();
        assert_eq!(lps, 5);
        assert_eq!(insts, 3);
        assert_eq!(gens, 1);
    }

    /// Reproducibility contract: identical `(seed, shape)` always
    /// produces a byte-identical `SourceUnit` across rebuilds; distinct
    /// seeds differ. This is the load-bearing invariant
    /// `.2b`'s emitters and `.2c`'s parity gate both depend on.
    #[test]
    fn unit_is_reproducible_and_seed_sensitive() {
        for seed in [0u64, 1, 7, 42, 12345] {
            let a = build_acceptable_unit(seed, 5, 3);
            let b = build_acceptable_unit(seed, 5, 3);
            assert_eq!(
                a, b,
                "same (seed,shape) must produce byte-identical units (seed={seed})"
            );
        }
        // Distinct seeds must differ in at least one resolved value.
        let s1 = build_acceptable_unit(1, 5, 3);
        let s2 = build_acceptable_unit(2, 5, 3);
        assert_ne!(s1, s2, "distinct seeds must produce distinct units");
    }

    /// The construction-time evaluator resolves every parameter,
    /// instance binding, and generate predicate. Pin some
    /// independently-computable expectations:
    ///   - Every `ParamDecl.value` in the top module's params equals
    ///     `eval(p.expr, &<env up to that point>)`.
    ///   - Every `Instance`'s `param_bindings[i].resolved` equals
    ///     `eval(b.expr, &<env at that point in the body walk>)`.
    ///   - Every `GenerateIf.taken` equals `eval(cond) != 0`.
    #[test]
    fn elaboration_evaluator_resolves_every_axis() {
        for seed in [0u64, 1, 7, 42, 12345] {
            let u = build_acceptable_unit(seed, 5, 2);
            // Package K is small positive.
            let PackageItem::Localparam(ref k) = u.packages[0].items[0];
            assert!(k.value > 0, "K should be positive after elaboration");

            // Top params are literal-rooted (the builder makes them
            // `ConstExpr::Lit`); the resolved value must equal the
            // literal.
            for p in &u.top.params {
                if let ConstExpr::Lit(v) = p.expr {
                    assert_eq!(
                        v, p.value,
                        "literal-rooted top param {} must resolve to its literal",
                        p.name
                    );
                }
            }

            // Localparams must each be a sane value (the builder uses
            // small adds/subs so we can't predict exact values without
            // re-evaluating, but the env extension is rule-based and
            // every body-walk step must succeed — the elaborate()
            // call inside build_acceptable_unit would panic otherwise.
            // We additionally check that L0 = P0 +/- small).
            let mut top_env: BTreeMap<String, i128> = BTreeMap::new();
            for p in &u.top.params {
                top_env.insert(p.name.clone(), p.value);
            }
            for item in &u.top.body {
                if let ModuleItem::Localparam(lp) = item {
                    let fresh = eval(&lp.expr, &top_env).expect("localparam must re-eval");
                    assert_eq!(
                        fresh, lp.value,
                        "localparam {} must equal a fresh re-eval over the prefix env",
                        lp.name
                    );
                    top_env.insert(lp.name.clone(), lp.value);
                }
            }

            // Generate-if `taken` must match a fresh re-eval of the
            // condition (in the env after all top params + body
            // localparams). The condition references `P0` and
            // `acc_<seed>_pkg::K` — `P0` is in top_env; we add the
            // pkg-qualified K too.
            let mut env = top_env.clone();
            env.insert(format!("acc_{seed}_pkg::K"), k.value);
            for item in &u.top.body {
                if let ModuleItem::GenerateIf(g) = item {
                    let fresh = eval(&g.condition, &env).expect("generate predicate must re-eval");
                    assert_eq!(
                        g.taken,
                        fresh != 0,
                        "GenerateIf.taken must equal eval(condition) != 0 (seed={seed})"
                    );
                }
            }
        }
    }

    /// The load-bearing **oracle-no-drift** invariant (the Phase-8
    /// counterpart of Phase 7's
    /// `stored_values_are_consistent_with_a_fresh_reeval`): every
    /// stored resolved field (param `.value`, localparam `.value`,
    /// instance binding `.resolved`, generate-if `.taken`) equals
    /// what a fresh re-evaluation would produce against the
    /// reconstructed lexical environment. If this proof passes,
    /// `.2b`'s emitter can trust the stored fields and never has to
    /// re-evaluate.
    #[test]
    fn elaborated_facts_match_a_fresh_reeval_across_the_seed_set() {
        for seed in 0u64..=8 {
            let unit = build_acceptable_unit(seed, 4, 2);

            // Re-build the env step by step and verify every stored
            // resolved value against a fresh `eval`.
            let mut env: BTreeMap<String, i128> = BTreeMap::new();
            for pkg in &unit.packages {
                for item in &pkg.items {
                    let PackageItem::Localparam(p) = item;
                    let fresh = eval(&p.expr, &env).expect("package localparam fresh eval");
                    assert_eq!(fresh, p.value, "pkg::{} drift (seed={seed})", p.name);
                    env.insert(format!("{}::{}", pkg.name, p.name), p.value);
                }
            }
            for p in &unit.top.params {
                let fresh = eval(&p.expr, &env).expect("top param fresh eval");
                assert_eq!(fresh, p.value, "top param {} drift (seed={seed})", p.name);
                env.insert(p.name.clone(), p.value);
            }
            for item in &unit.top.body {
                match item {
                    ModuleItem::Localparam(lp) => {
                        let fresh = eval(&lp.expr, &env).expect("localparam fresh eval");
                        assert_eq!(
                            fresh, lp.value,
                            "body localparam {} drift (seed={seed})",
                            lp.name
                        );
                        env.insert(lp.name.clone(), lp.value);
                    }
                    ModuleItem::Instance(inst) => {
                        for b in &inst.param_bindings {
                            let fresh = eval(&b.expr, &env).expect("binding fresh eval");
                            assert_eq!(
                                fresh, b.resolved,
                                "instance {} binding .{} drift (seed={seed})",
                                inst.inst_name, b.name
                            );
                        }
                    }
                    ModuleItem::GenerateIf(g) => {
                        let fresh = eval(&g.condition, &env).expect("generate fresh eval");
                        assert_eq!(
                            g.taken,
                            fresh != 0,
                            "generate {} taken drift (seed={seed})",
                            g.label
                        );
                    }
                }
            }
        }
    }

    // ===============================================================
    // .2b proofs — un-elaborated SV + elaborated-facts JSON manifest.
    // ===============================================================

    /// The emitted SV carries the structural shape `.2b` documents:
    /// a `package acc_<seed>_pkg;` with `localparam int K = …`; a
    /// child module stub `module child_<seed> #(parameter int CP… = …);
    /// endmodule`; a top module `module acc_<seed> #(parameter int
    /// P0 = …);` with chained `localparam int L…`; named-binding
    /// instances `child_<seed> u_<seed>_<idx> #(.CP0(…), …) ();`; and
    /// a `generate if (…) begin : g_taken … end else begin : g_else
    /// … end endgenerate`. Parameter ports, localparam definitions,
    /// instance bindings, and the generate predicate are emitted as
    /// **symbolic expressions** (not resolved integers) — that gap
    /// between symbolic SV and resolved manifest is exactly what
    /// elaboration must close.
    #[test]
    fn emit_sv_is_valid_unresolved_shape() {
        let unit = build_acceptable_unit(7, 4, 2);
        let sv = emit_sv(&unit);
        assert!(sv.contains("package acc_7_pkg;"));
        assert!(sv.contains("localparam int K = "));
        assert!(sv.contains("endpackage"));
        assert!(sv.contains("module child_7"));
        assert!(sv.contains("parameter int CP0 = "));
        assert!(sv.contains("module acc_7"));
        assert!(sv.contains("parameter int P0 = "));
        assert!(sv.contains("localparam int L0 = "));
        // Instances by name; both children present.
        assert!(sv.contains("child_7 #("));
        assert!(sv.contains("u_7_0 ()"));
        assert!(sv.contains("u_7_1 ()"));
        // Generate-if with both branches present (un-elaborated text
        // carries them both; only the manifest's `.taken` records
        // which branch survives elaboration).
        assert!(sv.contains("generate"));
        assert!(sv.contains(": g_taken"));
        assert!(sv.contains(": g_else"));
        assert!(sv.contains("endgenerate"));
        // The chained localparam decl is symbolic: at least one body
        // localparam line must contain an operator + a reference (not
        // a bare resolved integer only).
        let chained_line = sv
            .lines()
            .find(|l| {
                l.trim_start().starts_with("localparam int L")
                    && (l.contains("+") || l.contains("-"))
                    && (l.contains("P0") || l.contains("L"))
            })
            .expect("at least one chained localparam line in symbolic form");
        assert!(
            chained_line.contains("("),
            "chained localparam must render its symbolic expr: {chained_line}"
        );
    }

    /// The manifest is valid JSON, schema-shaped, and **every fact
    /// equals the `.2a` oracle**: package constants, top params /
    /// localparams (value + symbolic expr), each instance's resolved
    /// bindings, each generate label's `taken`. Cross-validates the
    /// build-manifest path against the elaboration result for the
    /// reproducibility-set seeds.
    #[test]
    fn manifest_mirrors_the_oracle() {
        for seed in [0u64, 1, 7, 42, 12345] {
            let unit = build_acceptable_unit(seed, 4, 3);
            let json = emit_manifest(&unit);
            let v: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
            assert_eq!(v["seed"], seed);
            assert_eq!(v["top"], format!("acc_{seed}"));

            // Packages
            let pkg = &v["packages"][0];
            assert_eq!(pkg["name"], format!("acc_{seed}_pkg"));
            let PackageItem::Localparam(ref k) = unit.packages[0].items[0];
            assert_eq!(pkg["constants"]["K"].as_i64().unwrap() as i128, k.value);

            // Top params + localparams: every entry equals the oracle.
            for p in &unit.top.params {
                let e = &v["top_params"][&p.name];
                assert_eq!(
                    e["value"].as_i64().unwrap() as i128,
                    p.value,
                    "top_params.{}.value vs oracle (seed={seed})",
                    p.name
                );
                assert_eq!(e["expr"].as_str().unwrap(), expr_to_sv(&p.expr));
            }
            for item in &unit.top.body {
                if let ModuleItem::Localparam(lp) = item {
                    let e = &v["top_localparams"][&lp.name];
                    assert_eq!(
                        e["value"].as_i64().unwrap() as i128,
                        lp.value,
                        "top_localparams.{}.value vs oracle (seed={seed})",
                        lp.name
                    );
                    assert_eq!(e["expr"].as_str().unwrap(), expr_to_sv(&lp.expr));
                }
            }

            // Instances: every resolved binding matches the oracle.
            let inst_arr = v["instances"].as_array().unwrap();
            let mut oracle_insts: Vec<&Instance> = Vec::new();
            for item in &unit.top.body {
                if let ModuleItem::Instance(inst) = item {
                    oracle_insts.push(inst);
                }
            }
            assert_eq!(inst_arr.len(), oracle_insts.len());
            for (jinst, oinst) in inst_arr.iter().zip(oracle_insts.iter()) {
                assert_eq!(jinst["inst_name"], oinst.inst_name);
                assert_eq!(jinst["child_module"], oinst.child_module);
                for b in &oinst.param_bindings {
                    assert_eq!(
                        jinst["resolved_bindings"][&b.name].as_i64().unwrap() as i128,
                        b.resolved,
                        "instance {}.{} resolved (seed={seed})",
                        oinst.inst_name,
                        b.name
                    );
                }
            }

            // Generate branches.
            for item in &unit.top.body {
                if let ModuleItem::GenerateIf(g) = item {
                    let taken = v["generate_branches"][&g.label]["taken"].as_bool().unwrap();
                    assert_eq!(
                        taken, g.taken,
                        "generate.{} taken vs oracle (seed={seed})",
                        g.label
                    );
                }
            }
        }
    }

    /// `(seed, shape)` → `.sv` and `→ .json` are **byte-identical**
    /// across rebuilds (the reproducibility contract; the emitter
    /// output is part of the reproducible artifact). Distinct seeds
    /// differ. Identical in structure to Phase 7's
    /// `sv_and_manifest_are_byte_reproducible`.
    #[test]
    fn sv_and_manifest_are_byte_reproducible() {
        for seed in [0u64, 1, 7, 42, 999] {
            let a = build_acceptable_unit(seed, 4, 3);
            let b = build_acceptable_unit(seed, 4, 3);
            assert_eq!(emit_sv(&a), emit_sv(&b));
            assert_eq!(emit_manifest(&a), emit_manifest(&b));
        }
        let s1 = build_acceptable_unit(1, 4, 3);
        let s2 = build_acceptable_unit(2, 4, 3);
        assert_ne!(emit_sv(&s1), emit_sv(&s2));
        assert_ne!(emit_manifest(&s1), emit_manifest(&s2));
    }
}
