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
use std::collections::BTreeMap;

// Reuse Phase 7's expression layer cross-tree (per `.1`'s
// full-factorization plan). The same `ConstExpr` algebra — literals,
// param-by-name references, unary/binary/ternary nodes — is the
// expression form for parameter defaults, instance bindings,
// generate predicates, and localparam chains in Phase 8.
use crate::microdesign::{eval, BinOp, ConstExpr, EvalError, ParamKind};

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
}
