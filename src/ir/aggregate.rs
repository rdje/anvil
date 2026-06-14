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
//! **Kind selection.** `StructPacked` is the always-sound default (the
//! general case for differing-width groups). `ArrayPacked`
//! (AGGREGATE-ARRAY-PACKING) is selected via
//! `annotate_aggregate_with_kind(.., prefer_array = true)` when every
//! projected group is internally uniform-width (a packed array is
//! LRM-bit-equivalent to the field concatenation); a non-uniform group
//! falls back to `StructPacked`. The per-module array preference is
//! rolled at the `crate::gen` call site under the seeded
//! `aggregate_array_prob` knob (default `0.0` → always `StructPacked`,
//! byte-identical). Modules that already carry a Phase 5 `param_env`
//! are skipped (the param/aggregate cross-product stays out of scope);
//! `aggregate_prob == 0` keeps the whole feature off and byte-identical.

use crate::ir::{AggregateGroup, AggregateKind, AggregateLayout, Module, Port};

/// Minimum number of same-direction data ports for a group to be worth
/// projecting as an aggregate (a 1-field struct adds no parser stress).
const MIN_AGGREGATE_FIELDS: usize = 2;

/// Record a packed-aggregate emitter projection on `module` when an
/// eligible same-direction data-port group exists. Idempotent; returns
/// `true` iff a layout was set. Never mutates the flat IR body.
///
/// Back-compat wrapper: always selects `StructPacked` (byte-identical
/// to pre-AGGREGATE-ARRAY-PACKING callers). Use
/// [`annotate_aggregate_with_kind`] to request a packed array.
pub fn annotate_aggregate(module: &mut Module) -> bool {
    annotate_aggregate_with_kind(module, false)
}

/// As [`annotate_aggregate`], but `prefer_array` requests a packed-array
/// (`ArrayPacked`) projection when **every** projected group is
/// internally uniform-width; a non-uniform group (or
/// `prefer_array == false`) falls back to `StructPacked`. Non-rolling
/// and idempotent. (AGGREGATE-ARRAY-PACKING.3)
pub fn annotate_aggregate_with_kind(module: &mut Module, prefer_array: bool) -> bool {
    // Idempotent / never double-annotate.
    if module.aggregate_layout.is_some() {
        return false;
    }
    // Scaffold scoping: leave Phase 5 parameterized modules to the param
    // projection; the param/aggregate cross-product is out of scope.
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

    // `ArrayPacked` is faithful only over a uniform-width group; require
    // every present projected group to be internally same-width.
    let in_uniform = match &inputs {
        Some(g) => group_is_uniform_width(&module.inputs, g),
        None => true,
    };
    let out_uniform = match &outputs {
        Some(g) => group_is_uniform_width(&module.outputs, g),
        None => true,
    };
    let kind = if prefer_array && in_uniform && out_uniform {
        AggregateKind::ArrayPacked
    } else {
        AggregateKind::StructPacked
    };

    module.aggregate_layout = Some(AggregateLayout {
        kind,
        inputs,
        outputs,
    });
    true
}

/// True iff every field of `g` resolves to the same port width in
/// `ports` (the precondition for a faithful `ArrayPacked` projection).
fn group_is_uniform_width(ports: &[Port], g: &AggregateGroup) -> bool {
    let mut width: Option<u32> = None;
    for (_, pid) in &g.fields {
        let pw = ports.iter().find(|p| p.id == *pid).map(|p| p.width);
        match (width, pw) {
            (None, Some(x)) => width = Some(x),
            (Some(a), Some(b)) if a == b => {}
            _ => return false,
        }
    }
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

    // ---- AGGREGATE-ARRAY-PACKING.3: kind selection ----

    #[test]
    fn prefer_array_with_uniform_widths_selects_array_packed() {
        // comb_module ports are all width 8 → uniform → ArrayPacked.
        let mut m = comb_module(2, 2);
        assert!(annotate_aggregate_with_kind(&mut m, true));
        assert_eq!(
            m.aggregate_layout.as_ref().unwrap().kind,
            AggregateKind::ArrayPacked
        );
    }

    #[test]
    fn prefer_array_false_stays_struct_packed() {
        // The 1-arg wrapper / prefer_array=false path is byte-identical.
        let mut m = comb_module(2, 2);
        assert!(annotate_aggregate_with_kind(&mut m, false));
        assert_eq!(
            m.aggregate_layout.as_ref().unwrap().kind,
            AggregateKind::StructPacked
        );
    }

    #[test]
    fn prefer_array_non_uniform_group_falls_back_to_struct() {
        // A mixed-width input group is not a faithful array → the whole
        // layout falls back to StructPacked even though outputs are
        // uniform (kind is per-layout, all-or-nothing).
        let mut m = Module {
            name: "m".into(),
            ..Module::default()
        };
        m.inputs.push(port(0, "a", 8, Direction::In));
        m.inputs.push(port(1, "b", 4, Direction::In));
        m.outputs.push(port(100, "o0", 8, Direction::Out));
        m.outputs.push(port(101, "o1", 8, Direction::Out));
        assert!(annotate_aggregate_with_kind(&mut m, true));
        assert_eq!(
            m.aggregate_layout.as_ref().unwrap().kind,
            AggregateKind::StructPacked
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
