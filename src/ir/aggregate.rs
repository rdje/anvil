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
}
