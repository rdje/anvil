//! Phase 5 width-parameterization pass.
//!
//! Optional, opt-in. When `Config::width_parameterization_prob` rolls
//! true for a finalized `Module`, this pass annotates the module with a
//! single width `parameter` (`ParamEnv`) and marks the interface ports
//! that share the chosen design width as parameterized.
//!
//! Architecture (C) from `DEVELOPMENT_NOTES.md` "Phase 5
//! parameterization design (2026-05-16, PHASE-5-PARAMETERIZATION.1)":
//! the module *body* is left exactly as constructed — concrete `u32`
//! at the design width — so every existing fold / validate / CSE path
//! is untouched and the design stays valid by construction. The
//! emitted `parameter` declaration defaults to the design width, so a
//! default (non-overridden) instantiation elaborates byte-identically
//! to the pre-Phase-5 concrete module. Only the emitter rendering and
//! the canonical identity signature consult the annotation.
//!
//! This is the `PHASE-5-PARAMETERIZATION.2.1` scaffold: it establishes
//! the annotation + the post-construction pass shape. Instantiation
//! substitution with `#(.W(v))` overrides and the soundness-restricted
//! override range are `PHASE-5-PARAMETERIZATION.2.2`; the
//! parameter-aware identity rule is `.2.3`; the matrix gate is `.2.4`.

use crate::config::Config;
use crate::ir::{GateOp, Module, Node, ParamEnv};
use rand::Rng;
use rand_chacha::ChaCha8Rng;

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

/// Annotate `module` with a single width `parameter` when the opt-in
/// `Config::width_parameterization_prob` knob rolls true and the
/// module passes the [`is_width_generic`] soundness gate. Returns
/// `true` iff the module was parameterized.
///
/// **Soundness.** The chosen design value is an existing port width;
/// the emitted `parameter` defaults to it, so default elaboration is
/// byte-identical to the un-parameterized module. The recorded
/// `[min, max]` range is the *intended* legal override range; only
/// `PHASE-5-PARAMETERIZATION.2.2` (instantiation substitution) may
/// pick override values from it, under the soundness restriction.
/// This pass never mutates the body and never picks an override.
pub fn parameterize_module(module: &mut Module, rng: &mut ChaCha8Rng, cfg: &Config) -> bool {
    // Idempotent / never double-parameterize.
    if module.param_env.is_some() {
        return false;
    }
    if cfg.width_parameterization_prob <= 0.0 {
        return false;
    }
    if !rng.gen_bool(cfg.width_parameterization_prob.clamp(0.0, 1.0)) {
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
    use rand::SeedableRng;

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

    fn cfg_with_prob(p: f64) -> Config {
        Config {
            width_parameterization_prob: p,
            min_width: 1,
            max_width: 16,
            ..Config::default()
        }
    }

    #[test]
    fn default_off_never_parameterizes() {
        let mut m = two_port_module(8, 8);
        let mut rng = ChaCha8Rng::seed_from_u64(1);
        let changed = parameterize_module(&mut m, &mut rng, &cfg_with_prob(0.0));
        assert!(!changed);
        assert!(m.param_env.is_none());
        assert!(m.parameterized_input_ports.is_empty());
        assert!(m.parameterized_output_ports.is_empty());
    }

    #[test]
    fn forced_prob_parameterizes_matching_width_ports() {
        let mut m = two_port_module(8, 8);
        let mut rng = ChaCha8Rng::seed_from_u64(1);
        let changed = parameterize_module(&mut m, &mut rng, &cfg_with_prob(1.0));
        assert!(changed);
        let env = m.param_env.expect("parameterized");
        assert_eq!(env.name, "W");
        assert_eq!(env.design_value, 8);
        assert!(env.min >= MIN_PARAMETERIZABLE_WIDTH && env.min <= env.design_value);
        assert!(env.max >= env.design_value);
        // Both ports share the design width 8 -> both parameterized.
        assert_eq!(m.parameterized_input_ports, vec![0]);
        assert_eq!(m.parameterized_output_ports, vec![1]);
    }

    #[test]
    fn mixed_width_module_is_not_parameterized() {
        // Input width 4, output width 8: not width-homogeneous, so the
        // single monomorphic body would not be correct for every `W`.
        // The soundness gate declines it entirely (no partial
        // parameterization).
        let mut m = two_port_module(4, 8);
        let mut rng = ChaCha8Rng::seed_from_u64(2);
        assert!(!parameterize_module(&mut m, &mut rng, &cfg_with_prob(1.0)));
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
        let mut rng = ChaCha8Rng::seed_from_u64(4);
        assert!(!parameterize_module(&mut m, &mut rng, &cfg_with_prob(1.0)));
        assert!(m.param_env.is_none());
    }

    #[test]
    fn width_one_outputs_are_not_parameterized() {
        // Only a width-1 output: no meaningful `[W-1:0]` form, so the
        // pass declines even at probability 1.0.
        let mut m = two_port_module(1, 1);
        let mut rng = ChaCha8Rng::seed_from_u64(3);
        assert!(!parameterize_module(&mut m, &mut rng, &cfg_with_prob(1.0)));
        assert!(m.param_env.is_none());
    }

    #[test]
    fn parameterization_is_idempotent() {
        let mut m = two_port_module(8, 8);
        let mut rng = ChaCha8Rng::seed_from_u64(1);
        assert!(parameterize_module(&mut m, &mut rng, &cfg_with_prob(1.0)));
        // A second pass must not double-parameterize.
        assert!(!parameterize_module(&mut m, &mut rng, &cfg_with_prob(1.0)));
    }
}
