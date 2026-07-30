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
//! `scripts/check_enumeration_parity.sh` reads [`KnobId::category`]'s match
//! arms from this file as the authoritative steering-category taxonomy; every
//! prose copy of that list is a shadow of it (decision `0033`).
//!
//! Re-exported by `crate::ir`, so every `crate::ir::KnobId` path is unchanged.

/// Identifier for each probability-roll knob. One variant per
/// `gen_bool(cfg.<prob>)` site across the generator. Keep
/// `Copy + Hash + Eq` cheap — the enum is keyed into a
/// `HashMap` on every roll.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KnobId {
    /// `Config::flop_prob` — per-depth chance that a leaf is
    /// a flop block (`build_flop_leaf`).
    FlopProb,
    /// `Config::comb_mux_prob` — per-depth chance of a comb-mux
    /// block (`build_comb_mux`).
    CombMuxProb,
    /// `Config::priority_encoder_prob` — per-depth chance of a
    /// priority-encoder block.
    PriorityEncoderProb,
    /// `Config::case_mux_prob` — per-depth chance of a procedural
    /// combinational case-mux block.
    CaseMuxProb,
    /// `Config::casez_mux_prob` — per-depth chance of a procedural
    /// combinational casez-mux block.
    CasezMuxProb,
    /// `Config::for_fold_prob` — per-depth chance of a procedural
    /// combinational statically bounded for-fold block.
    ForFoldProb,
    /// `Config::coefficient_prob` — chance that an Add/Sub/Mul
    /// becomes a linear-combination motif.
    CoefficientProb,
    /// `Config::const_shift_amount_prob` — chance that a Shl/Shr
    /// takes a constant shift amount.
    ConstShiftAmountProb,
    /// `Config::const_comparand_prob` — chance that a comparison's
    /// RHS is a constant.
    ConstComparandProb,
    /// `Config::constant_prob` — chance that a forced leaf without a
    /// matching-width signal becomes a fresh constant instead of a
    /// width-adapter from an existing dep-bearing source.
    ConstantProb,
    /// `Config::terminal_reuse_prob` — chance that a forced leaf with
    /// a matching-width signal reuses that signal instead of emitting
    /// a fresh constant.
    TerminalReuseProb,
    /// `Config::comb_mux_encoding_prob` — encoded vs one-hot
    /// comb-mux shape.
    CombMuxEncodingProb,
    /// `Config::flop_mux_encoding_prob` — encoded vs one-hot
    /// flop-D-mux shape.
    FlopMuxEncodingProb,
    /// `Config::share_prob` — chance of a DAG-sharing fork at
    /// operand slots.
    ShareProb,
    /// `Config::hierarchy_sibling_route_prob` — chance that a parent
    /// binds a child data input from a previously-instantiated
    /// sibling output when such a source is available.
    HierarchySiblingRouteProb,
    /// `Config::hierarchy_registered_sibling_route_prob` — chance
    /// that a parent binds a later child data input from an earlier
    /// sibling output through one local parent flop.
    HierarchyRegisteredSiblingRouteProb,
    /// `Config::hierarchy_registered_sibling_mixed_support_prob` —
    /// chance that a direct registered sibling route also mixes parent
    /// data-port support into the flop D side.
    HierarchyRegisteredSiblingMixedSupportProb,
    /// `Config::hierarchy_registered_child_input_cone_prob` — chance
    /// that a parent binds a later child data input through local
    /// parent logic over already-available parent sources and then one
    /// local parent flop, optionally chaining through earlier parent
    /// flops when available.
    HierarchyRegisteredChildInputConeProb,
    /// `Config::hierarchy_child_input_cone_prob` — chance that a
    /// parent binds a child data input through a local combinational
    /// cone over already-available parent sources.
    HierarchyChildInputConeProb,
    /// `Config::hierarchy_parent_cone_instance_prob` — chance that a
    /// parent-composed child-input cone instantiates an extra child module
    /// as an internal parent-cone source.
    HierarchyParentConeInstanceProb,
    /// `Config::hierarchy_parent_flop_prob` — chance that
    /// parent-side hierarchy cones may emit local parent flops.
    HierarchyParentFlopProb,
    /// `Config::flop_qfeedback_prob` — ZeroDefault vs QFeedback
    /// flop kind.
    FlopQFeedbackProb,

    // --- `motifs` (COVERAGE-STEERED-GENERATION.4b.1, decision 0035) ------
    // These seven select *what kind of module this is*. Each rolls at most
    // ONCE per module, so their `attempts` counts are low-resolution
    // compared with the per-gate knobs above — meaningful over a `--count`
    // sweep, not within one module.
    /// `Config::width_parameterization_prob` — chance that a free-standing
    /// leaf is built as a width-parameterizable module (Phase 5).
    WidthParameterizationProb,
    /// `Config::memory_prob` — chance that a free-standing leaf is built as
    /// an inferrable-memory module (Phase 6).
    MemoryProb,
    /// `Config::fsm_prob` — chance that a free-standing leaf is built as a
    /// generated-encoding FSM module (Phase 6).
    FsmProb,
    /// `Config::fsm_mealy_prob` — chance that an FSM block's output decode
    /// is Mealy (input-dependent) rather than Moore.
    FsmMealyProb,
    /// `Config::multi_clock_prob` — chance that a module is promoted to
    /// multiple clock domains with a by-construction CDC synchronizer.
    MultiClockProb,
    /// `Config::aggregate_prob` — chance that a module gains a packed
    /// aggregate emitter projection (Phase 5b).
    AggregateProb,
    /// `Config::aggregate_array_prob` — given an aggregate, the chance it
    /// is rendered array-packed rather than struct-packed.
    AggregateArrayProb,

    // --- `emission` (COVERAGE-STEERED-GENERATION.4b.2, decision 0035) ----
    // The nine richer-structured emit-projections. Each rolls once per
    // CANDIDATE GATE, so these are the highest-resolution knobs in the set —
    // hundreds of attempts per module, against the motif knobs' one. They are
    // a separate category from `motifs` for exactly that reason: a
    // per-category average over both would be dominated by these.
    /// `Config::soft_union_slice_prob` — per qualifying low-bits `Slice`, the
    /// chance it renders through an IEEE-1800-2023 `union soft` overlay.
    SoftUnionSliceProb,
    /// `Config::function_emit_prob` — per qualifying gate, the chance it is
    /// re-rendered as a `function automatic` over its direct operands.
    FunctionEmitProb,
    /// `Config::generate_loop_emit_prob` — per qualifying `{N{x}}`
    /// replication, the chance it is re-rendered as a `generate for` loop.
    GenerateLoopEmitProb,
    /// `Config::task_emit_prob` — per qualifying gate, the chance it is
    /// re-rendered as a combinational `task automatic`.
    TaskEmitProb,
    /// `Config::multi_output_task_emit_prob` — per ungrouped qualifying gate,
    /// the chance it leads a co-supported multi-output `task automatic` group.
    MultiOutputTaskEmitProb,
    /// `Config::cone_function_emit_prob` — per qualifying cone root, the
    /// chance the whole cone is re-rendered as one `function automatic`.
    ConeFunctionEmitProb,
    /// `Config::mux_if_emit_prob` — per qualifying 2:1 `Mux`, the chance it is
    /// re-expressed as a procedural `always_comb` `if`/`else` block.
    MuxIfEmitProb,
    /// `Config::case_mux_if_emit_prob` — per qualifying dynamic-selector
    /// `CaseMux`, the chance its `case` body becomes an `if`/`else if` chain.
    CaseMuxIfEmitProb,
    /// `Config::casez_mux_if_emit_prob` — per qualifying dynamic-selector
    /// `CasezMux`, the chance its `casez` body becomes a **masked**
    /// `if`/`else if` chain.
    CasezMuxIfEmitProb,
}

impl KnobId {
    /// Every steerable knob id, in declaration order. The single
    /// enumeration of the `KnobId` universe — reused wherever a caller
    /// needs to map a knob *name* (the string key in `Metrics`) back to
    /// its [`category`](KnobId::category) without re-deriving a second
    /// name→category table (`feedback_full_factorization`: one classifier,
    /// not two). Consumed by the coverage-readout per-category roll-up
    /// (`COVERAGE-STEERED-GENERATION.2b`). The list is exhaustive: a new
    /// variant must be added here too (the `name`/`category` matches below
    /// are exhaustive, so a missing entry here cannot silently mislabel a
    /// knob — it just omits it from the roll-up, which a reviewer catches).
    pub fn all() -> &'static [KnobId] {
        &[
            KnobId::FlopProb,
            KnobId::CombMuxProb,
            KnobId::PriorityEncoderProb,
            KnobId::CaseMuxProb,
            KnobId::CasezMuxProb,
            KnobId::ForFoldProb,
            KnobId::CoefficientProb,
            KnobId::ConstShiftAmountProb,
            KnobId::ConstComparandProb,
            KnobId::ConstantProb,
            KnobId::TerminalReuseProb,
            KnobId::CombMuxEncodingProb,
            KnobId::FlopMuxEncodingProb,
            KnobId::ShareProb,
            KnobId::HierarchySiblingRouteProb,
            KnobId::HierarchyRegisteredSiblingRouteProb,
            KnobId::HierarchyRegisteredSiblingMixedSupportProb,
            KnobId::HierarchyRegisteredChildInputConeProb,
            KnobId::HierarchyChildInputConeProb,
            KnobId::HierarchyParentConeInstanceProb,
            KnobId::HierarchyParentFlopProb,
            KnobId::FlopQFeedbackProb,
            KnobId::WidthParameterizationProb,
            KnobId::MemoryProb,
            KnobId::FsmProb,
            KnobId::FsmMealyProb,
            KnobId::MultiClockProb,
            KnobId::AggregateProb,
            KnobId::AggregateArrayProb,
            KnobId::SoftUnionSliceProb,
            KnobId::FunctionEmitProb,
            KnobId::GenerateLoopEmitProb,
            KnobId::TaskEmitProb,
            KnobId::MultiOutputTaskEmitProb,
            KnobId::ConeFunctionEmitProb,
            KnobId::MuxIfEmitProb,
            KnobId::CaseMuxIfEmitProb,
            KnobId::CasezMuxIfEmitProb,
        ]
    }

    /// Dense position of this knob in [`all`](KnobId::all).
    ///
    /// **The guard on `all()`** (`COVERAGE-STEERED-GENERATION.4b.1`, decision
    /// `0035`). `all()` is a hand-written list, and `name()`/`category()` are
    /// exhaustive matches — so a new variant is forced to declare a name and a
    /// category, but *not* to appear in `all()`. Omitting it there used to be
    /// silent: the knob would simply vanish from the per-category coverage
    /// roll-up and from `--steer`'s key classification.
    ///
    /// This match is **exhaustive**, so a new variant fails to compile until it
    /// is given an index; `all_is_complete_and_ordered` then fails until `all()`
    /// carries it at that index. The pair turns a silent omission into a red
    /// build plus a red test (repair rung **R2**, decision `0033`).
    ///
    /// `dead_code` is allowed deliberately: this function's value is the
    /// **exhaustiveness of its match**, not its call sites. It is called only
    /// by the guard test, and it must stay compiled in every profile so that a
    /// new variant is a hard error rather than a test-only one.
    #[allow(dead_code)]
    fn index(self) -> usize {
        match self {
            KnobId::FlopProb => 0,
            KnobId::CombMuxProb => 1,
            KnobId::PriorityEncoderProb => 2,
            KnobId::CaseMuxProb => 3,
            KnobId::CasezMuxProb => 4,
            KnobId::ForFoldProb => 5,
            KnobId::CoefficientProb => 6,
            KnobId::ConstShiftAmountProb => 7,
            KnobId::ConstComparandProb => 8,
            KnobId::ConstantProb => 9,
            KnobId::TerminalReuseProb => 10,
            KnobId::CombMuxEncodingProb => 11,
            KnobId::FlopMuxEncodingProb => 12,
            KnobId::ShareProb => 13,
            KnobId::HierarchySiblingRouteProb => 14,
            KnobId::HierarchyRegisteredSiblingRouteProb => 15,
            KnobId::HierarchyRegisteredSiblingMixedSupportProb => 16,
            KnobId::HierarchyRegisteredChildInputConeProb => 17,
            KnobId::HierarchyChildInputConeProb => 18,
            KnobId::HierarchyParentConeInstanceProb => 19,
            KnobId::HierarchyParentFlopProb => 20,
            KnobId::FlopQFeedbackProb => 21,
            KnobId::WidthParameterizationProb => 22,
            KnobId::MemoryProb => 23,
            KnobId::FsmProb => 24,
            KnobId::FsmMealyProb => 25,
            KnobId::MultiClockProb => 26,
            KnobId::AggregateProb => 27,
            KnobId::AggregateArrayProb => 28,
            KnobId::SoftUnionSliceProb => 29,
            KnobId::FunctionEmitProb => 30,
            KnobId::GenerateLoopEmitProb => 31,
            KnobId::TaskEmitProb => 32,
            KnobId::MultiOutputTaskEmitProb => 33,
            KnobId::ConeFunctionEmitProb => 34,
            KnobId::MuxIfEmitProb => 35,
            KnobId::CaseMuxIfEmitProb => 36,
            KnobId::CasezMuxIfEmitProb => 37,
        }
    }

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

    /// Stable lowercase string key for serialisation into
    /// `Metrics`. Matches the `Config` field name, minus the
    /// `_prob` suffix where present.
    pub fn name(&self) -> &'static str {
        match self {
            KnobId::FlopProb => "flop_prob",
            KnobId::CombMuxProb => "comb_mux_prob",
            KnobId::PriorityEncoderProb => "priority_encoder_prob",
            KnobId::CaseMuxProb => "case_mux_prob",
            KnobId::CasezMuxProb => "casez_mux_prob",
            KnobId::ForFoldProb => "for_fold_prob",
            KnobId::CoefficientProb => "coefficient_prob",
            KnobId::ConstShiftAmountProb => "const_shift_amount_prob",
            KnobId::ConstComparandProb => "const_comparand_prob",
            KnobId::ConstantProb => "constant_prob",
            KnobId::TerminalReuseProb => "terminal_reuse_prob",
            KnobId::CombMuxEncodingProb => "comb_mux_encoding_prob",
            KnobId::FlopMuxEncodingProb => "flop_mux_encoding_prob",
            KnobId::ShareProb => "share_prob",
            KnobId::HierarchySiblingRouteProb => "hierarchy_sibling_route_prob",
            KnobId::HierarchyRegisteredSiblingRouteProb => {
                "hierarchy_registered_sibling_route_prob"
            }
            KnobId::HierarchyRegisteredSiblingMixedSupportProb => {
                "hierarchy_registered_sibling_mixed_support_prob"
            }
            KnobId::HierarchyRegisteredChildInputConeProb => {
                "hierarchy_registered_child_input_cone_prob"
            }
            KnobId::HierarchyChildInputConeProb => "hierarchy_child_input_cone_prob",
            KnobId::HierarchyParentConeInstanceProb => "hierarchy_parent_cone_instance_prob",
            KnobId::HierarchyParentFlopProb => "hierarchy_parent_flop_prob",
            KnobId::FlopQFeedbackProb => "flop_qfeedback_prob",
            KnobId::WidthParameterizationProb => "width_parameterization_prob",
            KnobId::MemoryProb => "memory_prob",
            KnobId::FsmProb => "fsm_prob",
            KnobId::FsmMealyProb => "fsm_mealy_prob",
            KnobId::MultiClockProb => "multi_clock_prob",
            KnobId::AggregateProb => "aggregate_prob",
            KnobId::AggregateArrayProb => "aggregate_array_prob",
            KnobId::SoftUnionSliceProb => "soft_union_slice_prob",
            KnobId::FunctionEmitProb => "function_emit_prob",
            KnobId::GenerateLoopEmitProb => "generate_loop_emit_prob",
            KnobId::TaskEmitProb => "task_emit_prob",
            KnobId::MultiOutputTaskEmitProb => "multi_output_task_emit_prob",
            KnobId::ConeFunctionEmitProb => "cone_function_emit_prob",
            KnobId::MuxIfEmitProb => "mux_if_emit_prob",
            KnobId::CaseMuxIfEmitProb => "case_mux_if_emit_prob",
            KnobId::CasezMuxIfEmitProb => "casez_mux_if_emit_prob",
        }
    }

    /// Coarse coverage **category** for construction-time steering
    /// roll-ups (`COVERAGE-STEERED-GENERATION`, decision `0023`). A
    /// steering-config can up- or down-weight a whole family of related
    /// knobs with one `per_category` entry instead of naming each
    /// `KnobId` individually. The taxonomy is intentionally small and
    /// fixed: `state`, `selectors`, `datapath`, `terminals`, `sharing`,
    /// `hierarchy`, and — since `COVERAGE-STEERED-GENERATION.4b.1`
    /// (decision `0035`) — `motifs`, which selects *what kind of module
    /// this is*, and `emission`, which selects how an already-built cone is
    /// *rendered*. Every
    /// `KnobId` maps to exactly one category (the match is exhaustive, so
    /// a new knob must declare its category).
    ///
    /// The six original categories keep their **exact** membership, so a
    /// steering config written before `.4b.1` still means what it meant.
    pub fn category(&self) -> &'static str {
        match self {
            KnobId::FlopProb | KnobId::FlopQFeedbackProb => "state",
            KnobId::CombMuxProb
            | KnobId::PriorityEncoderProb
            | KnobId::CaseMuxProb
            | KnobId::CasezMuxProb
            | KnobId::ForFoldProb
            | KnobId::CombMuxEncodingProb
            | KnobId::FlopMuxEncodingProb => "selectors",
            KnobId::CoefficientProb | KnobId::ConstShiftAmountProb | KnobId::ConstComparandProb => {
                "datapath"
            }
            KnobId::ConstantProb | KnobId::TerminalReuseProb => "terminals",
            KnobId::ShareProb => "sharing",
            KnobId::HierarchySiblingRouteProb
            | KnobId::HierarchyRegisteredSiblingRouteProb
            | KnobId::HierarchyRegisteredSiblingMixedSupportProb
            | KnobId::HierarchyRegisteredChildInputConeProb
            | KnobId::HierarchyChildInputConeProb
            | KnobId::HierarchyParentConeInstanceProb
            | KnobId::HierarchyParentFlopProb => "hierarchy",
            KnobId::WidthParameterizationProb
            | KnobId::MemoryProb
            | KnobId::FsmProb
            | KnobId::FsmMealyProb
            | KnobId::MultiClockProb
            | KnobId::AggregateProb
            | KnobId::AggregateArrayProb => "motifs",
            KnobId::SoftUnionSliceProb
            | KnobId::FunctionEmitProb
            | KnobId::GenerateLoopEmitProb
            | KnobId::TaskEmitProb
            | KnobId::MultiOutputTaskEmitProb
            | KnobId::ConeFunctionEmitProb
            | KnobId::MuxIfEmitProb
            | KnobId::CaseMuxIfEmitProb
            | KnobId::CasezMuxIfEmitProb => "emission",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `COVERAGE-STEERED-GENERATION.4b.1` — the guard on `KnobId::all()`
    /// (decision `0035`, repair rung **R2** per decision `0033`).
    ///
    /// `all()` is a hand-written list, and it grew from 22 to 29 entries in
    /// this slice. `name()` and `category()` are exhaustive matches, so a new
    /// variant *is* forced to declare both — but nothing forced it into
    /// `all()`, and omitting it there is **silent**: the knob simply vanishes
    /// from the per-category coverage roll-up and from `--steer`'s key
    /// classification, which is precisely the class of bug this tree exists to
    /// eliminate.
    ///
    /// `index()` is exhaustive, so a new variant **cannot compile** without
    /// declaring an index; this test then fails unless `all()` lists every
    /// index it declares, exactly once, in order.
    ///
    /// **Known residual gap, stated rather than papered over.** This catches a
    /// misordering, a duplicate, and an omission *from the middle* (the entry
    /// at that position then declares the wrong index). It does **not** catch
    /// truncation at the **tail**: drop the highest-index variant from `all()`
    /// and the remaining `0..n-1` are still contiguous and in order. Verified
    /// by negative control — removing the last entry leaves this test green.
    ///
    /// A length assertion cannot close it either, because any "expected count"
    /// derived from `all()` shrinks with it, and a hand-written count would be
    /// a second copy of the very list being guarded (decision `0033`: a repair
    /// may not introduce a new hand-maintained list). Closing it properly means
    /// making `all()` **derived** — generating the enum, `all`, `name` and
    /// `category` from one macro table so the list stops existing (rung **R1**).
    /// Registered as `COVERAGE-STEERED-GENERATION.6`.
    #[test]
    fn all_is_complete_and_ordered() {
        let all = KnobId::all();
        for (position, knob) in all.iter().enumerate() {
            assert_eq!(
                knob.index(),
                position,
                "KnobId::all() is out of order at position {position}: {} declares \
                 index {} — all() must list every variant exactly once, in index order",
                knob.name(),
                knob.index()
            );
        }
    }

    /// Names and categories are both total and unambiguous over `all()`:
    /// every knob round-trips through `category_of_name`, no two knobs share a
    /// name, and no knob name collides with a category name (the `--steer` key
    /// classifier depends on that disjointness to decide `per_knob` vs
    /// `per_category`).
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
