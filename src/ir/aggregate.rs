//! Phase 5b packed-aggregate emitter projection — post-construction
//! annotation pass (`PHASE-5B-AGGREGATES.2.1`).
//!
//! Architecture **(P)** (see `DEVELOPMENT_NOTES.md` "Phase 5b
//! packed-aggregate emitter projection design"): a contiguous,
//! same-direction group of *data* ports is recorded as a packed
//! aggregate the emitter renders as one aggregate port plus boundary
//! alias wires. The flat IR body, validators, CSE keys and
//! `canonical_module_signature` are all untouched — a packed `struct`
//! is LRM-defined to be bit-equivalent to the concatenation of its
//! members, so the projection is a bijective, semantically-empty
//! regrouping that is valid by construction.
//!
//! **Non-rolling.** The opt-in *decision* (whether to project a given
//! module) is taken once at the call site in `crate::gen` via the
//! seeded generator RNG under the `Config::aggregate_prob` knob
//! (reproducible; never `thread_rng`). This function only performs the
//! post-construction annotation, so it never draws from the RNG and is
//! safe to call on any module: one that has no eligible group is left
//! untouched, and the default-off (`aggregate_prob == 0.0`) caller
//! guard means it is never invoked at all when the feature is off
//! (byte-identical).
//!
//! **Scaffold scoping (`.2.1`).** Only `AggregateKind::StructPacked`
//! is selected (the general, always-sound case for differing-width
//! groups). Modules that already carry a Phase 5 `param_env` are
//! skipped so the param/aggregate cross-product is out of scope for
//! the scaffold (recorded as a `.2.1` decision); `aggregate_prob == 0`
//! keeps both features off and byte-identical regardless.

use crate::ir::{AggregateGroup, AggregateKind, AggregateLayout, Module};

/// Minimum number of same-direction data ports for a group to be worth
/// projecting as an aggregate (a 1-field struct adds no parser stress).
const MIN_AGGREGATE_FIELDS: usize = 2;

/// Record a packed-aggregate emitter projection on `module` when an
/// eligible same-direction data-port group exists. Idempotent; returns
/// `true` iff a layout was set. Never mutates the flat IR body.
pub fn annotate_aggregate(module: &mut Module) -> bool {
    // Idempotent / never double-annotate.
    if module.aggregate_layout.is_some() {
        return false;
    }
    // `.2.1` scaffold scoping: leave Phase 5 parameterized modules to
    // the param projection; the param/aggregate cross-product is a
    // later sub-slice.
    if module.param_env.is_some() {
        return false;
    }

    let input_fields: Vec<(String, u32)> = module
        .emitted_data_input_ports()
        .map(|p| (p.name.clone(), p.id))
        .collect();
    let output_fields: Vec<(String, u32)> = module
        .outputs
        .iter()
        .map(|p| (p.name.clone(), p.id))
        .collect();

    let inputs = (input_fields.len() >= MIN_AGGREGATE_FIELDS).then(|| AggregateGroup {
        type_name: format!("{}_in_t", module.name),
        port_name: format!("{}_in", module.name),
        fields: input_fields,
    });
    let outputs = (output_fields.len() >= MIN_AGGREGATE_FIELDS).then(|| AggregateGroup {
        type_name: format!("{}_out_t", module.name),
        port_name: format!("{}_out", module.name),
        fields: output_fields,
    });

    if inputs.is_none() && outputs.is_none() {
        return false;
    }

    module.aggregate_layout = Some(AggregateLayout {
        kind: AggregateKind::StructPacked,
        inputs,
        outputs,
    });
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{Direction, ParamEnv, Port};

    fn port(id: u32, name: &str, width: u32, dir: Direction) -> Port {
        Port {
            id,
            name: name.to_string(),
            width,
            dir,
        }
    }

    fn comb_module(n_in: u32, n_out: u32) -> Module {
        let mut m = Module {
            name: "m".into(),
            ..Module::default()
        };
        for i in 0..n_in {
            m.inputs.push(port(i, &format!("a{i}"), 8, Direction::In));
        }
        for i in 0..n_out {
            m.outputs
                .push(port(100 + i, &format!("o{i}"), 8, Direction::Out));
        }
        m
    }

    #[test]
    fn two_in_two_out_is_annotated_struct_packed() {
        let mut m = comb_module(2, 2);
        assert!(annotate_aggregate(&mut m));
        let layout = m.aggregate_layout.as_ref().expect("annotated");
        assert_eq!(layout.kind, AggregateKind::StructPacked);
        let i = layout.inputs.as_ref().expect("input group");
        assert_eq!(i.type_name, "m_in_t");
        assert_eq!(i.port_name, "m_in");
        assert_eq!(i.fields, vec![("a0".to_string(), 0), ("a1".to_string(), 1)]);
        let o = layout.outputs.as_ref().expect("output group");
        assert_eq!(o.type_name, "m_out_t");
        assert_eq!(
            o.fields,
            vec![("o0".to_string(), 100), ("o1".to_string(), 101)]
        );
    }

    #[test]
    fn single_port_side_forms_no_group() {
        // 1 input (< MIN), 3 outputs → only the output group forms.
        let mut m = comb_module(1, 3);
        assert!(annotate_aggregate(&mut m));
        let layout = m.aggregate_layout.as_ref().unwrap();
        assert!(layout.inputs.is_none());
        assert!(layout.outputs.is_some());
    }

    #[test]
    fn no_eligible_group_is_not_annotated() {
        let mut m = comb_module(1, 1);
        assert!(!annotate_aggregate(&mut m));
        assert!(m.aggregate_layout.is_none());
    }

    #[test]
    fn idempotent() {
        let mut m = comb_module(2, 2);
        assert!(annotate_aggregate(&mut m));
        assert!(!annotate_aggregate(&mut m));
        assert!(m.aggregate_layout.is_some());
    }

    #[test]
    fn parameterized_modules_are_skipped() {
        let mut m = comb_module(2, 2);
        m.param_env = Some(ParamEnv {
            name: "W".into(),
            min: 2,
            max: 8,
            design_value: 8,
        });
        assert!(!annotate_aggregate(&mut m));
        assert!(m.aggregate_layout.is_none());
    }

    #[test]
    fn clk_rst_excluded_from_input_group() {
        // Sequential-style interface: clk, rst_n are control inputs and
        // must never enter the data aggregate.
        let mut m = Module {
            name: "s".into(),
            ..Module::default()
        };
        m.inputs.push(port(0, "clk", 1, Direction::In));
        m.inputs.push(port(1, "rst_n", 1, Direction::In));
        m.inputs.push(port(2, "a", 8, Direction::In));
        m.inputs.push(port(3, "b", 8, Direction::In));
        m.outputs.push(port(100, "o0", 8, Direction::Out));
        m.outputs.push(port(101, "o1", 8, Direction::Out));
        m.clock = Some(0);
        m.reset = Some(1);
        // Give it local sequential state so clk/rst_n pass the control-
        // port emission filter and are then explicitly excluded as
        // control (not merely "not emitted"). The flop body is a
        // placeholder — `annotate_aggregate` never reads or validates
        // it, only ports/param_env/aggregate_layout.
        m.flops.push(crate::ir::Flop {
            id: 0,
            width: 8,
            d: None,
            q: 0,
            reset_val: 0,
            reset_kind: crate::ir::ResetKind::Async,
            kind: crate::ir::FlopKind::ZeroDefault,
            mux: crate::ir::FlopMux::None,
        });
        assert!(annotate_aggregate(&mut m));
        let i = m
            .aggregate_layout
            .as_ref()
            .unwrap()
            .inputs
            .as_ref()
            .expect("data input group");
        assert_eq!(
            i.fields,
            vec![("a".to_string(), 2), ("b".to_string(), 3)],
            "clk/rst_n must be excluded from the data aggregate"
        );
    }

    // ---- PHASE-5B-AGGREGATES.2.2(b): identity-invariance ----

    fn named_comb(name: &str, n_in: u32, n_out: u32) -> Module {
        let mut m = Module {
            name: name.into(),
            ..Module::default()
        };
        for i in 0..n_in {
            m.inputs.push(port(i, &format!("a{i}"), 8, Direction::In));
        }
        for i in 0..n_out {
            m.outputs
                .push(port(100 + i, &format!("o{i}"), 8, Direction::Out));
            // Drive each output from input 0 so the signature reflects
            // real structure, not just an empty port list.
            m.nodes
                .push(crate::ir::Node::PrimaryInput { port: 0, width: 8 });
            m.drives.push((100 + i, (i) as crate::ir::NodeId));
        }
        m
    }

    #[test]
    fn canonical_signature_is_invariant_under_projection() {
        // The projection is an emitter-surface annotation only; the
        // flat IR (ports/nodes/drives) is untouched, so the module's
        // canonical signature must be identical before and after
        // `annotate_aggregate` — the annotation is deliberately NOT
        // hashed into identity (opposite of Phase 5's `param_env`).
        use crate::metrics::canonical_module_signature;
        let mut m = named_comb("m", 3, 2);
        let key = |ps: &[crate::ir::Port]| -> Vec<(u32, u32, String)> {
            ps.iter().map(|p| (p.id, p.width, p.name.clone())).collect()
        };
        let sig_before = canonical_module_signature(&m);
        let ins_before = key(&m.inputs);
        let outs_before = key(&m.outputs);

        assert!(annotate_aggregate(&mut m));
        assert!(m.aggregate_layout.is_some());

        assert_eq!(
            canonical_module_signature(&m),
            sig_before,
            "aggregate annotation must not change the canonical signature"
        );
        // Flat IR is genuinely unchanged.
        assert_eq!(key(&m.inputs), ins_before);
        assert_eq!(key(&m.outputs), outs_before);
    }

    #[test]
    fn aggregate_projected_twin_dedup_collapses() {
        // A concrete module and a structurally-identical module that
        // has been aggregate-projected are the SAME circuit and must
        // dedup-collapse (the projection changes nothing semantic).
        use crate::ir::dedup::dedup_modules;
        use crate::ir::Design;
        use crate::metrics::canonical_module_signature;

        let a = named_comb("a", 3, 2);
        let mut b = named_comb("b", 3, 2);
        assert!(annotate_aggregate(&mut b));

        assert_eq!(
            canonical_module_signature(&a),
            canonical_module_signature(&b),
            "a module and its aggregate-projected twin must share a signature"
        );

        let top = Module {
            name: "top".into(),
            ..Module::default()
        };
        let mut design = Design {
            top: "top".into(),
            modules: vec![a, b, top],
        };
        let removed = dedup_modules(&mut design);
        assert_eq!(
            removed, 1,
            "the projected twin collapses into its concrete equal"
        );
        assert_eq!(design.modules.len(), 2, "survivor + top");
        assert!(design.modules.iter().any(|m| m.name == "top"));
    }
}
