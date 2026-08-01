//! The **steering taxonomy**: which generator decisions are steerable, what
//! each is called on the wire, and which coverage family it belongs to.
//!
//! Split out of `types.rs` by `IR-TYPES-DECOMPOSITION.2`. `KnobId` is not
//! circuit IR — it enumerates the generator's probability-roll *sites*, so it
//! answers "what can be steered?", not "what is a module?". It is referenced
//! from `crate::ir::knob_roll` (the single steering-aware roll primitive,
//! decision `0034`), from `Config`/`--steer` key classification, and from the
//! coverage readout; it is referenced by **nothing** in the data model, and it
//! references nothing from it — verified in both directions before the move.
//!
//! # The table is the taxonomy
//!
//! There is exactly **one** hand-maintained list here: the [`knob_ids!`] table
//! below. The `KnobId` enum, [`KnobId::all`], [`KnobId::name`], and
//! [`KnobId::category`] are all generated from it, so they cannot disagree and
//! **a new knob is one row** (`COVERAGE-STEERED-GENERATION.6`, repair rung
//! **R1** of decision `0033`: retire a shadow list rather than gate it
//! forever). Until then the same 38 variants were written out five times, and
//! `.4b.1`'s guard could not catch a truncation of `all()`'s **tail** — a gap
//! that stops existing once the list does.
//!
//! `scripts/check_enumeration_parity.sh` parses the table's **third column**
//! as the authoritative steering-category taxonomy; every prose copy of that
//! list is a shadow of it (decision `0033`). Keep each row on one line in the
//! `Variant => "name", "category";` shape — the extractor matches the whole
//! row, so a reshaped row yields zero categories and trips its count floor
//! loudly rather than passing vacuously.
//!
//! Re-exported by `crate::ir`, so every `crate::ir::KnobId` path is unchanged.

/// Generates the `KnobId` enum and its three total projections from one table.
///
/// Row shape: `Variant => "wire name", "coverage category";`, preceded by the
/// variant's own doc comment. Everything the crate knows about a knob lives on
/// that row, which is the point — the five parallel tables this replaced were
/// five chances to add a knob to four of them.
macro_rules! knob_ids {
    (
        $(
            $(#[$variant_doc:meta])*
            $variant:ident => $name:literal, $category:literal;
        )+
    ) => {
        /// Identifier for each probability-roll knob. One variant per
        /// `gen_bool(cfg.<prob>)` site across the generator. Keep
        /// `Copy + Hash + Eq` cheap — the enum is keyed into a
        /// `HashMap` on every roll.
        ///
        /// Generated from the [`knob_ids!`] table; see the module docs.
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum KnobId {
            $(
                $(#[$variant_doc])*
                $variant,
            )+
        }

        impl KnobId {
            /// Every steerable knob id, in table order. The single
            /// enumeration of the `KnobId` universe — reused wherever a caller
            /// needs to map a knob *name* (the string key in `Metrics`) back to
            /// its [`category`](KnobId::category) without re-deriving a second
            /// name→category table (`feedback_full_factorization`: one
            /// classifier, not two). Consumed by the coverage-readout
            /// per-category roll-up (`COVERAGE-STEERED-GENERATION.2b`).
            ///
            /// Exhaustive **by construction**: this slice and the enum are
            /// expanded from the same table, so a knob that exists cannot be
            /// missing here (`COVERAGE-STEERED-GENERATION.6`).
            pub fn all() -> &'static [KnobId] {
                &[
                    $( KnobId::$variant, )+
                ]
            }

            /// Stable lowercase string key for serialisation into
            /// `Metrics`. Matches the `Config` field name, minus the
            /// `_prob` suffix where present.
            pub fn name(&self) -> &'static str {
                match self {
                    $( KnobId::$variant => $name, )+
                }
            }

            /// Coarse coverage **category** for construction-time steering
            /// roll-ups (`COVERAGE-STEERED-GENERATION`, decision `0023`). A
            /// steering-config can up- or down-weight a whole family of related
            /// knobs with one `per_category` entry instead of naming each
            /// `KnobId` individually. The taxonomy is intentionally small and
            /// fixed; the table's third column is its only definition, and a
            /// row must name a category, so a new knob cannot arrive
            /// unclassified.
            ///
            /// Two of the families are worth distinguishing explicitly, because
            /// they sit at opposite ends of the pipeline *and* of resolution
            /// (`COVERAGE-STEERED-GENERATION.4b.1`/`.4b.2`, decision `0035`):
            /// `motifs` selects *what kind of module this is* and rolls at most
            /// once per module, while `emission` selects how an already-built
            /// cone is *rendered* and rolls once per candidate gate. A category
            /// average over both would be dominated by the latter.
            ///
            /// The six categories that predate decision `0035` keep their
            /// **exact** membership, so a steering config written before
            /// `.4b.1` still means what it meant.
            pub fn category(&self) -> &'static str {
                match self {
                    $( KnobId::$variant => $category, )+
                }
            }
        }
    };
}

knob_ids! {
    /// `Config::flop_prob` — per-depth chance that a leaf is
    /// a flop block (`build_flop_leaf`).
    FlopProb => "flop_prob", "state";
    /// `Config::comb_mux_prob` — per-depth chance of a comb-mux
    /// block (`build_comb_mux`).
    CombMuxProb => "comb_mux_prob", "selectors";
    /// `Config::priority_encoder_prob` — per-depth chance of a
    /// priority-encoder block.
    PriorityEncoderProb => "priority_encoder_prob", "selectors";
    /// `Config::case_mux_prob` — per-depth chance of a procedural
    /// combinational case-mux block.
    CaseMuxProb => "case_mux_prob", "selectors";
    /// `Config::casez_mux_prob` — per-depth chance of a procedural
    /// combinational casez-mux block.
    CasezMuxProb => "casez_mux_prob", "selectors";
    /// `Config::for_fold_prob` — per-depth chance of a procedural
    /// combinational statically bounded for-fold block.
    ForFoldProb => "for_fold_prob", "selectors";
    /// `Config::coefficient_prob` — chance that an Add/Sub/Mul
    /// becomes a linear-combination motif.
    CoefficientProb => "coefficient_prob", "datapath";
    /// `Config::const_shift_amount_prob` — chance that a Shl/Shr
    /// takes a constant shift amount.
    ConstShiftAmountProb => "const_shift_amount_prob", "datapath";
    /// `Config::const_comparand_prob` — chance that a comparison's
    /// RHS is a constant.
    ConstComparandProb => "const_comparand_prob", "datapath";
    /// `Config::constant_prob` — chance that a forced leaf without a
    /// matching-width signal becomes a fresh constant instead of a
    /// width-adapter from an existing dep-bearing source.
    ConstantProb => "constant_prob", "terminals";
    /// `Config::terminal_reuse_prob` — chance that a forced leaf with
    /// a matching-width signal reuses that signal instead of emitting
    /// a fresh constant.
    TerminalReuseProb => "terminal_reuse_prob", "terminals";
    /// `Config::comb_mux_encoding_prob` — encoded vs one-hot
    /// comb-mux shape.
    CombMuxEncodingProb => "comb_mux_encoding_prob", "selectors";
    /// `Config::flop_mux_encoding_prob` — encoded vs one-hot
    /// flop-D-mux shape.
    FlopMuxEncodingProb => "flop_mux_encoding_prob", "selectors";
    /// `Config::share_prob` — chance of a DAG-sharing fork at
    /// operand slots.
    ShareProb => "share_prob", "sharing";
    /// `Config::hierarchy_sibling_route_prob` — chance that a parent
    /// binds a child data input from a previously-instantiated
    /// sibling output when such a source is available.
    HierarchySiblingRouteProb => "hierarchy_sibling_route_prob", "hierarchy";
    /// `Config::hierarchy_registered_sibling_route_prob` — chance
    /// that a parent binds a later child data input from an earlier
    /// sibling output through one local parent flop.
    HierarchyRegisteredSiblingRouteProb => "hierarchy_registered_sibling_route_prob", "hierarchy";
    /// `Config::hierarchy_registered_sibling_mixed_support_prob` —
    /// chance that a direct registered sibling route also mixes parent
    /// data-port support into the flop D side.
    HierarchyRegisteredSiblingMixedSupportProb => "hierarchy_registered_sibling_mixed_support_prob", "hierarchy";
    /// `Config::hierarchy_registered_child_input_cone_prob` — chance
    /// that a parent binds a later child data input through local
    /// parent logic over already-available parent sources and then one
    /// local parent flop, optionally chaining through earlier parent
    /// flops when available.
    HierarchyRegisteredChildInputConeProb => "hierarchy_registered_child_input_cone_prob", "hierarchy";
    /// `Config::hierarchy_child_input_cone_prob` — chance that a
    /// parent binds a child data input through a local combinational
    /// cone over already-available parent sources.
    HierarchyChildInputConeProb => "hierarchy_child_input_cone_prob", "hierarchy";
    /// `Config::hierarchy_parent_cone_instance_prob` — chance that a
    /// parent-composed child-input cone instantiates an extra child module
    /// as an internal parent-cone source.
    HierarchyParentConeInstanceProb => "hierarchy_parent_cone_instance_prob", "hierarchy";
    /// `Config::hierarchy_parent_flop_prob` — chance that
    /// parent-side hierarchy cones may emit local parent flops.
    HierarchyParentFlopProb => "hierarchy_parent_flop_prob", "hierarchy";
    /// `Config::flop_qfeedback_prob` — ZeroDefault vs QFeedback
    /// flop kind.
    FlopQFeedbackProb => "flop_qfeedback_prob", "state";

    // --- `motifs` (COVERAGE-STEERED-GENERATION.4b.1, decision 0035) ------
    // These seven select *what kind of module this is*. Each rolls at most
    // ONCE per module, so their `attempts` counts are low-resolution
    // compared with the per-gate knobs above — meaningful over a `--count`
    // sweep, not within one module.
    /// `Config::width_parameterization_prob` — chance that a free-standing
    /// leaf is built as a width-parameterizable module (Phase 5).
    WidthParameterizationProb => "width_parameterization_prob", "motifs";
    /// `Config::memory_prob` — chance that a free-standing leaf is built as
    /// an inferrable-memory module (Phase 6).
    MemoryProb => "memory_prob", "motifs";
    /// `Config::fsm_prob` — chance that a free-standing leaf is built as a
    /// generated-encoding FSM module (Phase 6).
    FsmProb => "fsm_prob", "motifs";
    /// `Config::fsm_mealy_prob` — chance that an FSM block's output decode
    /// is Mealy (input-dependent) rather than Moore.
    FsmMealyProb => "fsm_mealy_prob", "motifs";
    /// `Config::multi_clock_prob` — chance that a module is promoted to
    /// multiple clock domains with a by-construction CDC synchronizer.
    MultiClockProb => "multi_clock_prob", "motifs";
    /// `Config::aggregate_prob` — chance that a module gains a packed
    /// aggregate emitter projection (Phase 5b).
    AggregateProb => "aggregate_prob", "motifs";
    /// `Config::aggregate_array_prob` — given an aggregate, the chance it
    /// is rendered array-packed rather than struct-packed.
    AggregateArrayProb => "aggregate_array_prob", "motifs";

    // --- `emission` (COVERAGE-STEERED-GENERATION.4b.2, decision 0035) ----
    // The nine richer-structured emit-projections. Each rolls once per
    // CANDIDATE GATE, so these are the highest-resolution knobs in the set —
    // hundreds of attempts per module, against the motif knobs' one. They are
    // a separate category from `motifs` for exactly that reason: a
    // per-category average over both would be dominated by these.
    /// `Config::soft_union_slice_prob` — per qualifying low-bits `Slice`, the
    /// chance it renders through an IEEE-1800-2023 `union soft` overlay.
    SoftUnionSliceProb => "soft_union_slice_prob", "emission";
    /// `Config::function_emit_prob` — per qualifying gate, the chance it is
    /// re-rendered as a `function automatic` over its direct operands.
    FunctionEmitProb => "function_emit_prob", "emission";
    /// `Config::generate_loop_emit_prob` — per qualifying `{N{x}}`
    /// replication, the chance it is re-rendered as a `generate for` loop.
    GenerateLoopEmitProb => "generate_loop_emit_prob", "emission";
    /// `Config::task_emit_prob` — per qualifying gate, the chance it is
    /// re-rendered as a combinational `task automatic`.
    TaskEmitProb => "task_emit_prob", "emission";
    /// `Config::multi_output_task_emit_prob` — per ungrouped qualifying gate,
    /// the chance it leads a co-supported multi-output `task automatic` group.
    MultiOutputTaskEmitProb => "multi_output_task_emit_prob", "emission";
    /// `Config::cone_function_emit_prob` — per qualifying cone root, the
    /// chance the whole cone is re-rendered as one `function automatic`.
    ConeFunctionEmitProb => "cone_function_emit_prob", "emission";
    /// `Config::mux_if_emit_prob` — per qualifying 2:1 `Mux`, the chance it is
    /// re-expressed as a procedural `always_comb` `if`/`else` block.
    MuxIfEmitProb => "mux_if_emit_prob", "emission";
    /// `Config::case_mux_if_emit_prob` — per qualifying dynamic-selector
    /// `CaseMux`, the chance its `case` body becomes an `if`/`else if` chain.
    CaseMuxIfEmitProb => "case_mux_if_emit_prob", "emission";
    /// `Config::casez_mux_if_emit_prob` — per qualifying dynamic-selector
    /// `CasezMux`, the chance its `casez` body becomes a **masked**
    /// `if`/`else if` chain.
    CasezMuxIfEmitProb => "casez_mux_if_emit_prob", "emission";

    // --- `qualifiers` (CAPABILITY-BREADTH-EXPANSION.4b.1, decision 0044) ------
    // The IEEE 1800-2017 §12.5.3 case qualifiers. They roll once per candidate
    // gate and at the same pipeline point as `emission`, so sharing that
    // category was the obvious call — and it is wrong, for the same reason
    // decision 0035 split `motifs` off in the first place: **a steering
    // category is a family a user up-weights as a unit, so it must be
    // semantically homogeneous.**
    //
    // The nine `emission` knobs are PROJECTIONS competing for one gate graph
    // under mutual exclusion. A qualifier is not a projection — it decorates a
    // rendering rather than replacing one, and it claims only gates the
    // projections DECLINED. So the two are **anti-correlated**: one
    // `--steer emission=2.0` would push `case_mux_if` to claim more gates while
    // pushing the qualifier to claim more of a pool that is simultaneously
    // shrinking, and the category's aggregate achieved rate would be measuring
    // a self-cancelling mixture. Their own category keeps `--steer emission`
    // meaning exactly what it meant before this leaf, and makes the qualifiers
    // independently steerable (decision 0017's API-completeness gate).
    /// `Config::unique_case_prob` — per qualifying dynamic-selector `CaseMux` /
    /// `CasezMux` that still renders as a statement, the chance it is prefixed
    /// `unique` (asserting FULL and PARALLEL).
    UniqueCaseProb => "unique_case_prob", "qualifiers";
    /// `Config::priority_case_prob` — the same, for the `priority` prefix
    /// (asserting FULL only). Rolled only when the `unique` roll did not fire,
    /// so a gate carries at most one qualifier.
    PriorityCaseProb => "priority_case_prob", "qualifiers";
}

impl KnobId {
    /// The [`category`](KnobId::category) of the knob whose
    /// [`name`](KnobId::name) is `name`, or `None` if `name` is not a known
    /// knob. The reverse of `name` over the fixed [`all`](KnobId::all) set —
    /// the one place that inverts the name table, so the coverage roll-up
    /// never carries a private copy of the mapping.
    pub fn category_of_name(name: &str) -> Option<&'static str> {
        KnobId::all()
            .iter()
            .find(|knob| knob.name() == name)
            .map(|knob| knob.category())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Names and categories are both total and unambiguous over `all()`:
    /// every knob round-trips through `category_of_name`, no two knobs share a
    /// name, and no knob name collides with a category name (the `--steer` key
    /// classifier depends on that disjointness to decide `per_knob` vs
    /// `per_category`).
    ///
    /// This is the whole of what still needs asserting. `.4b.1`'s
    /// `all_is_complete_and_ordered` — and the exhaustive private `index()`
    /// that made it possible — retired at `COVERAGE-STEERED-GENERATION.6`:
    /// they existed only to catch a variant missing from a hand-written
    /// `all()`, and `all()` is now expanded from the same table as the enum,
    /// so the omission they guarded against has no way to be expressed. What
    /// *is* still expressible is a mistyped or duplicated string in a row,
    /// which is exactly what this test reads.
    #[test]
    fn knob_names_and_categories_are_disjoint_and_total() {
        let all = KnobId::all();

        let mut names: Vec<&str> = all.iter().map(|k| k.name()).collect();
        names.sort_unstable();
        let unique = names.len();
        names.dedup();
        assert_eq!(unique, names.len(), "two KnobId variants share a name");

        let mut categories: Vec<&str> = all.iter().map(|k| k.category()).collect();
        categories.sort_unstable();
        categories.dedup();

        for knob in all {
            assert_eq!(
                KnobId::category_of_name(knob.name()),
                Some(knob.category()),
                "{} does not round-trip through category_of_name",
                knob.name()
            );
            assert!(
                !categories.contains(&knob.name()),
                "knob name {} collides with a category name — --steer could not \
                 classify the key unambiguously",
                knob.name()
            );
        }
    }
}
