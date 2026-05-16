//! Phase 5 width-parameterization: soundness gate + post-construction
//! annotation.
//!
//! Architecture (C) from `DEVELOPMENT_NOTES.md` "Phase 5
//! parameterization design": the module *body* is left exactly as
//! constructed — concrete `u32` at the design width — so every existing
//! fold / validate / CSE path is untouched and the design stays valid
//! by construction. The emitted `parameter` defaults to the design
//! width, so a default (non-overridden) instantiation elaborates
//! byte-identically to the pre-Phase-5 concrete module. Only the
//! emitter and the canonical identity signature consult the annotation.
//!
//! Rules-first (`DEVELOPMENT_NOTES.md` "Phase 5 rules-first pivot,
//! PHASE-5-PARAMETERIZATION.2.2.1"): the unconstrained cone generator
//! essentially never produces a width-homogeneous module, so a post-hoc
//! homogeneity *filter* would be inert and is the generate-then-filter
//! anti-pattern. The opt-in decision is therefore taken once at
//! construction time by `src/gen/module.rs`'s rules-first
//! `build_parameterizable_leaf` lane (`.2.2.2`), which *constructs* a
//! width-homogeneous combinational leaf by rule. [`annotate_parameterized`]
//! here is the **non-rolling** post-construction step: it annotates iff
//! [`is_width_generic`] holds (always true for a constructor-built
//! module; ~never for an organically-generated one). Instantiation
//! substitution with `#(.W(v))` overrides is `.2.2.3`; the
//! parameter-aware identity rule is `.2.3`; the matrix gate is `.2.4`.

use crate::config::Config;
use crate::ir::{GateOp, Module, Node, ParamEnv};

/// The fixed parameter name used by the first parameterization slice.
/// A single width parameter per module; multi-parameter modules are
/// explicitly out of the first Phase 5 slice.
const PARAM_NAME: &str = "W";

/// Minimum port width eligible for parameterization. Width-1 ports
/// emit no `[hi:lo]` range at all, so parameterizing them is
/// meaningless (and `[W-1:0]` with `W = 1` would render `[0:0]`); the
/// pass only parameterizes ports whose design width is at least 2 so
/// the symbolic `[W-1:0]` form is well-formed and meaningful.
const MIN_PARAMETERIZABLE_WIDTH: u32 = 2;

/// Soundness gate (PHASE-5-PARAMETERIZATION.2.2). The body is
/// monomorphic — it is constructed once at `design`. Emitting it with
/// a `parameter W` and instantiating at `W != design` is only valid if
/// the *identical* SystemVerilog body text is correct for every `W`.
/// That holds iff the module is **width-homogeneous**: a purely
/// combinational leaf (no flops, no instances) in which every port and
/// every node width equals `design`, built only from width-preserving
/// same-width gates, with no fixed-width `Constant` and no
/// width-changing `Slice` / `Concat` / `ForFold`. Comparison and `Mux`
/// modules are excluded automatically: their select / result nodes
/// have width 1 (≠ `design ≥ 2`) and fail the per-node check.
///
/// This keeps architecture (C) sound without (B)'s symbolic width
/// arithmetic, and is a construction-time rule (no generate-then-
/// filter): a module that does not qualify is simply left
/// un-parameterized.
fn is_width_generic(module: &Module, design: u32) -> bool {
    if !module.flops.is_empty() || !module.instances.is_empty() {
        return false;
    }
    if module.inputs.iter().any(|p| p.width != design)
        || module.outputs.iter().any(|p| p.width != design)
    {
        return false;
    }
    module.nodes.iter().all(|n| match n {
        Node::PrimaryInput { width, .. } => *width == design,
        Node::Gate { op, width, .. } => {
            *width == design
                && !matches!(
                    op,
                    GateOp::Slice { .. } | GateOp::Concat | GateOp::ForFold { .. }
                )
        }
        // Constant: fixed-width literal, not width-generic.
        // FlopQ / InstanceOutput: excluded by the no-flops /
        // no-instances guard above; listed for exhaustiveness.
        Node::Constant { .. } | Node::FlopQ { .. } | Node::InstanceOutput { .. } => false,
    })
}

/// Annotate `module` with a single width `parameter` iff it passes the
/// [`is_width_generic`] soundness gate. Returns `true` iff annotated.
///
/// **Non-rolling.** The opt-in *decision* (whether to produce a
/// parameterizable module at all) is taken once, at construction time,
/// by the rules-first `build_parameterizable_leaf` lane
/// (`PHASE-5-PARAMETERIZATION.2.2.2`) — see `src/gen/module.rs`. This
/// function only performs the post-construction annotation, so it never
/// draws from the RNG. It is therefore safe to call unconditionally on
/// every design module: a non-width-generic module (the overwhelming
/// majority) is simply left untouched, and the default-off
/// (`width_parameterization_prob == 0.0`) caller guard means it is
/// never invoked at all when the feature is off (byte-identical).
///
/// **Soundness.** The design value is an existing output port width;
/// the emitted `parameter` defaults to it, so default elaboration is
/// byte-identical to the un-parameterized module. The recorded
/// `[min, max]` range is the *intended* legal override range; only
/// `PHASE-5-PARAMETERIZATION.2.2.3` (instantiation substitution) may
/// pick override values from it. This function never mutates the body.
pub fn annotate_parameterized(module: &mut Module, cfg: &Config) -> bool {
    // Idempotent / never double-parameterize.
    if module.param_env.is_some() {
        return false;
    }

    // Choose the design width: the width of the first output port that
    // is wide enough to be meaningfully parameterized. Outputs are the
    // module's externally-observable boundary, so anchoring the
    // parameter on an output keeps the symbolic interface coherent.
    let Some(design_value) = module
        .outputs
        .iter()
        .map(|p| p.width)
        .find(|&w| w >= MIN_PARAMETERIZABLE_WIDTH)
    else {
        return false;
    };

    // Soundness gate: only parameterize a width-homogeneous module so
    // the single monomorphic body text is correct for every `W`.
    if !is_width_generic(module, design_value) {
        return false;
    }

    // Every interface port that shares exactly the design width is
    // parameterized together, yielding the canonical
    // `module m #(parameter int W = D) (input [W-1:0] ..., output
    // [W-1:0] ...)` shape. Ports of other widths stay concrete.
    let parameterized_input_ports: Vec<_> = module
        .inputs
        .iter()
        .filter(|p| p.width == design_value)
        .map(|p| p.id)
        .collect();
    let parameterized_output_ports: Vec<_> = module
        .outputs
        .iter()
        .filter(|p| p.width == design_value)
        .map(|p| p.id)
        .collect();

    // An output of `design_value` always exists (we just found one),
    // so the output set is non-empty; inputs may legitimately be
    // empty (a source-only module).
    debug_assert!(!parameterized_output_ports.is_empty());

    // Intended legal override range: the configured width band, always
    // containing the design value, floored at a well-formed width.
    let min = cfg
        .min_width
        .min(design_value)
        .max(MIN_PARAMETERIZABLE_WIDTH);
    let max = cfg.max_width.max(design_value);

    module.param_env = Some(ParamEnv {
        name: PARAM_NAME.to_string(),
        min,
        max,
        design_value,
    });
    module.parameterized_input_ports = parameterized_input_ports;
    module.parameterized_output_ports = parameterized_output_ports;
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{Direction, Module, Node, Port};

    fn two_port_module(in_w: u32, out_w: u32) -> Module {
        let mut m = Module {
            name: "m".into(),
            ..Module::default()
        };
        m.inputs.push(Port {
            id: 0,
            name: "a".into(),
            width: in_w,
            dir: Direction::In,
        });
        m.outputs.push(Port {
            id: 1,
            name: "y".into(),
            width: out_w,
            dir: Direction::Out,
        });
        m.nodes.push(Node::PrimaryInput {
            port: 0,
            width: in_w,
        });
        m.drives.push((1, 0));
        m
    }

    fn cfg_widths() -> Config {
        Config {
            min_width: 1,
            max_width: 16,
            ..Config::default()
        }
    }

    #[test]
    fn width_homogeneous_module_is_annotated() {
        let mut m = two_port_module(8, 8);
        assert!(annotate_parameterized(&mut m, &cfg_widths()));
        let env = m.param_env.expect("parameterized");
        assert_eq!(env.name, "W");
        assert_eq!(env.design_value, 8);
        assert!(env.min >= MIN_PARAMETERIZABLE_WIDTH && env.min <= env.design_value);
        assert!(env.max >= env.design_value);
        assert_eq!(m.parameterized_input_ports, vec![0]);
        assert_eq!(m.parameterized_output_ports, vec![1]);
    }

    #[test]
    fn mixed_width_module_is_not_parameterized() {
        // Input width 4, output width 8: not width-homogeneous, so the
        // single monomorphic body would not be correct for every `W`.
        // The soundness gate declines it entirely.
        let mut m = two_port_module(4, 8);
        assert!(!annotate_parameterized(&mut m, &cfg_widths()));
        assert!(m.param_env.is_none());
        assert!(m.parameterized_input_ports.is_empty());
        assert!(m.parameterized_output_ports.is_empty());
    }

    #[test]
    fn module_with_a_constant_is_not_parameterized() {
        // A fixed-width Constant is not width-generic even if its
        // width equals the design width; the gate must decline.
        let mut m = two_port_module(8, 8);
        m.nodes.push(Node::Constant { width: 8, value: 5 });
        assert!(!annotate_parameterized(&mut m, &cfg_widths()));
        assert!(m.param_env.is_none());
    }

    #[test]
    fn width_one_outputs_are_not_parameterized() {
        // Only a width-1 output: no meaningful `[W-1:0]` form.
        let mut m = two_port_module(1, 1);
        assert!(!annotate_parameterized(&mut m, &cfg_widths()));
        assert!(m.param_env.is_none());
    }

    #[test]
    fn annotation_is_idempotent() {
        let mut m = two_port_module(8, 8);
        assert!(annotate_parameterized(&mut m, &cfg_widths()));
        // A second call must not double-parameterize.
        assert!(!annotate_parameterized(&mut m, &cfg_widths()));
    }
}
