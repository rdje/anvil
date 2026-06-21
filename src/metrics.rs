//! Post-hoc metrics computed by walking an emitted `Module`.
//!
//! Metrics are structural facts about *what landed* in a module —
//! not about the generator's internal decisions. They are cheap to
//! compute (one pass over `m.nodes`, one pass over `m.flops`,
//! plus a reverse-fanout pass). Probability-roll telemetry is
//! sourced from `Module::knob_rolls` and surfaced as
//! `knob_roll_attempts` / `knob_roll_fires`, so live generated
//! artifacts can report both structural facts and per-knob roll
//! attempts/fires.
//!
//! The goal is observability per the user's directive: every knob
//! must be measurable from the generated output so we can tell
//! whether it is doing its job, whether it is redundant with
//! another knob, or whether a new knob is needed.

use crate::ir::{Design, FlopId, GateOp, InstanceId, InstanceRole, Module, Node, NodeId, PortId};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, HashMap};

/// Structural summary of a single generated module. Serialisable as
/// JSON for inclusion in `manifest.json` or stderr dumps.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct Metrics {
    /// Module identifier (e.g. `mod_42_0000`).
    pub module: String,

    // --- Size ---------------------------------------------------
    pub num_inputs: usize,
    pub num_outputs: usize,
    pub num_nodes: usize,
    pub num_gates: usize,
    pub num_constants: usize,
    pub num_primary_inputs: usize,
    pub num_flop_q_refs: usize,
    pub num_instance_outputs: usize,
    pub num_flops: usize,
    pub num_instances: usize,
    /// `MULTI-CLOCK-CDC.3b.2` — declared clock domains
    /// (`Module.clock_domains.len()`). `0` for K=1 single-clock
    /// (the empty-default backward-compatible state) — see
    /// `Module::effective_clock_domains` for the synthesised
    /// K=1 fallback. `>= 2` once the `multi_clock_prob`
    /// promotion pass has fired on this module.
    #[serde(default)]
    pub num_clock_domains: usize,
    /// `MULTI-CLOCK-CDC.3b.2` — number of exact 2-flop synchronizer
    /// chains in this module. `SIGNOFF-SURFACE-EXPANSION.1` keeps
    /// this legacy metric exact-2 only; N-flop chains are counted by
    /// `num_cdc_synchronizer_chains` and summarized by
    /// `max_cdc_synchronizer_stages`.
    #[serde(default)]
    pub num_cdc_2_flop_synchronizers: usize,
    /// `SIGNOFF-SURFACE-EXPANSION.1` — number of CDC synchronizer
    /// chains with at least two destination-domain stages. This is the
    /// stage-count-agnostic companion to the legacy exact-2-flop
    /// metric above.
    #[serde(default)]
    pub num_cdc_synchronizer_chains: usize,
    /// `SIGNOFF-SURFACE-EXPANSION.1` — maximum number of destination-
    /// domain stages observed in any CDC synchronizer chain in this
    /// module. `2` is the default primitive; values `>= 3` prove the
    /// N-flop synchronizer path executed.
    #[serde(default)]
    pub max_cdc_synchronizer_stages: usize,

    // --- Per-gate-kind distribution -----------------------------
    /// Count of `Node::Gate` per `GateOp` kind (`"and"`, `"mux"`,
    /// etc.). Empty kinds omitted.
    pub gates_by_kind: BTreeMap<String, usize>,

    // --- Constants distribution ---------------------------------
    /// Count of `Node::Constant` by width. Reveals constant-width
    /// distribution (useful for the coefficient-width clamp).
    pub constants_by_width: BTreeMap<u32, usize>,
    /// Count of `Node::Constant` whose value is 0 vs all-ones vs
    /// other. Reveals the share of sentinel constants (zero fill,
    /// all-ones mask) vs meaningful literals.
    pub constants_zero: usize,
    pub constants_all_ones: usize,
    pub constants_other: usize,

    // --- Mux shape ----------------------------------------------
    /// Number of 2-to-1 `Mux` gates.
    pub num_muxes_2to1: usize,
    /// Number of 2-to-1 `Mux` gates whose two data arms are the
    /// same `NodeId` — the pathological `(s)?(x):(x)` form.
    /// Should be 0 at `mux_arm_duplication_rate = 0.0`.
    pub num_muxes_degenerate: usize,

    // --- Operator-gate operand duplication ----------------------
    /// Number of `Add`/`Mul` operator gates whose operand list
    /// repeats a `NodeId` (the same sub-expression occupies two or
    /// more operand slots). Should be 0 at
    /// `operand_duplication_rate = 0.0`, where operator-gate operand
    /// lists are strictly distinct; non-zero only when
    /// `operand_duplication_rate > 0.0` admits duplicates. Scoped to
    /// `Add`/`Mul` to match that knob — `And`/`Or`/`Xor`/`Mux` keep
    /// their 2-operand degenerate-shape rejection regardless of the
    /// rate. Computed post-hoc over the finalized IR and never
    /// emitted, so adding this metric is byte-identical for generated
    /// RTL (`SIGNOFF-AUTOMATION-EXPANSION.2b`).
    pub num_operator_gates_with_duplicate_operands: usize,

    // --- Concat shape -------------------------------------------
    /// Number of `Concat` gates whose operands are all the same
    /// `NodeId` — emitted as `{N{expr}}`.
    pub num_concats_replication: usize,
    pub num_concats_heterogeneous: usize,

    // --- Shift shape --------------------------------------------
    /// Number of `Shl`/`Shr` gates whose rhs is a literal constant.
    /// This is the constant-shift surface (`value << 3`,
    /// `value >> 1`).
    pub num_constant_shift_gates: usize,
    /// Number of `Shl`/`Shr` gates whose rhs is not a literal
    /// constant. This is the variable-shift surface
    /// (`value << signal`, `value >> signal`).
    pub num_variable_shift_gates: usize,

    // --- Sharing / fanout ---------------------------------------
    /// Number of internal nodes with fanout >= 2 (at least one
    /// other node references them). Measures sharing density
    /// after CSE.
    pub num_shared_nodes: usize,
    /// Maximum fanout observed on any single internal node.
    pub max_fanout: usize,
    /// Average fanout across all internal nodes (dep-bearing or
    /// not). `num_nodes == 0` → 0.0.
    pub avg_fanout: f64,

    // --- Flops --------------------------------------------------
    /// Per-kind flop count: how many `ZeroDefault` vs `QFeedback`.
    pub flops_zero_default: usize,
    pub flops_qfeedback: usize,
    /// Per-mux-shape flop count: `None` / `OneHot(M)` / `Encoded(M)`.
    pub flops_mux_none: usize,
    pub flops_mux_one_hot: usize,
    pub flops_mux_encoded: usize,

    // --- AST-instance saturation --------------------------------
    /// For each `(op, width)` pair, the maximum number of
    /// instances observed of any single AST of that kind. Should
    /// be `<= max_ast_instances` by construction. A value equal
    /// to the knob means the cap was hit — consumers are being
    /// routed to existing instances.
    pub max_gate_ast_multiplicity: usize,
    pub max_constant_ast_multiplicity: usize,

    // --- Operand-arity distribution -----------------------------
    /// Histogram of operator-gate arity (operand count) across all
    /// `Node::Gate`s. Keyed by operand count. Reveals the effective
    /// range of the `min_gate_arity` / `max_gate_arity` knobs.
    /// Non-operator nodes (comparisons, mux, slice, concat, reductions,
    /// shifts) with their fixed or variadic-positional arities are
    /// included too — all gate operand counts contribute.
    pub gate_operand_count_histogram: BTreeMap<usize, usize>,
    /// Maximum operand count observed on any single gate. For
    /// N-arity operators this is bounded above by `max_gate_arity`.
    pub max_gate_operand_count: usize,
    /// Per-op operand-count stats. Useful for distinguishing
    /// `Add`/`Mul` arity (bounded by `max_gate_arity`) from `Concat`
    /// arity (can be much larger, driven by mux-arm widths).
    pub max_operand_count_by_kind: BTreeMap<String, usize>,

    // --- Combinational depth ------------------------------------
    /// Combinational depth of each `Node::Gate`: longest path from
    /// the gate back to a leaf (primary input, constant, or flop Q).
    /// Computed by bottom-up walk over `m.nodes`, which is always
    /// in topological order (no forward references by construction).
    ///
    /// **Relationship to the `max_depth` knob:** the knob bounds
    /// the recursion depth of `build_cone`, not the IR gate-chain
    /// depth. Each `build_cone` recursion level can expand into
    /// many internal gate layers via block-assembly helpers
    /// (chained-ternary mux, OR-of-masked-arms mux, linear-
    /// combination adder trees). So `max_gate_depth` is typically
    /// 10–100× the knob value, but it is monotone in the knob —
    /// useful for verifying that raising `max_depth` produces
    /// deeper cones.
    pub max_gate_depth: usize,
    /// Histogram of per-gate combinational depth across all gates.
    /// Keyed by depth value.
    pub gate_depth_histogram: BTreeMap<usize, usize>,

    // --- Factorization-ladder telemetry -------------------------
    /// Count of operand slots on associative gates
    /// (`And`/`Or`/`Xor`/`Add`/`Mul`) whose operand is itself a
    /// same-op same-width gate *and is flattenable under the
    /// current duplicate policy*. `Add([a, Add(b, c), d])` counts
    /// 1 (the middle slot), flattening to `Add([a, b, c, d])`.
    /// `Add([a, Add(a, b)])` counts 0 at the default strict
    /// `operand_duplication_rate`, because flattening would
    /// introduce a duplicate `a` and the Associative layer
    /// intentionally preserves the nested shape in that case.
    ///
    /// The metric is post-hoc: it examines the finalized IR, not
    /// construction-time events. Running the generator over a
    /// seed sweep and summing this metric tells you how much
    /// flattening the current Associative layer still leaves on
    /// the table — justifying (or not) further work there.
    pub nested_associative_operand_count: usize,

    /// Number of times the `ConstantFold` factorization layer fired
    /// during construction. Each fire is one algebraic identity
    /// applied in `intern_gate` — either an operand dropped because
    /// it was an identity element (`x + 0`, `x & all_ones`, `x * 1`,
    /// …), an absorbing substitution (`x & 0 → 0`,
    /// `x | all_ones → all_ones`, `x * 0 → 0`), or a 2-arity rhs-
    /// zero short-circuit on `Sub` / `Shl` / `Shr`. Sourced from
    /// `Module::fold_identities_applied`. Zero at factorization
    /// levels below `ConstantFold`.
    pub fold_identities_applied: u64,

    /// Number of times the `Peephole` factorization layer fired
    /// during construction. Each fire is one local rewrite applied
    /// in `intern_gate` — double-negation collapse
    /// (`Not(Not(x)) → x`), fully-constant comparison evaluation
    /// (`Eq`/`Neq`/`Lt`/`Gt`/`Le`/`Ge` over two same-width
    /// constants), full-width `Slice(hi, 0)` identity, or
    /// single-operand `Concat` identity. Sourced from
    /// `Module::peephole_rewrites_applied`. Zero at factorization
    /// levels below `Peephole`.
    pub peephole_rewrites_applied: u64,

    /// Number of nodes removed by the post-construction
    /// `compact_node_ids` pass. Zero when every rewrite in
    /// `intern_gate` is orphan-safe by construction — non-zero
    /// when a rewrite like `Not(Not(x)) → x` leaves an inner
    /// gate unreferenced. Sourced from `Module::nodes_compacted`.
    pub nodes_compacted: u32,

    /// Number of duplicate flops merged away by the post-drain
    /// endpoint-preserving state-sharing pass. Sourced from
    /// `Module::flops_merged`.
    pub flops_merged: u32,

    /// Number of duplicate flops merged away by the opt-in bounded
    /// bisimulation flop-merge pass (`IDENTITY-DEEPENING`). Non-zero only
    /// when `bisimulation_flop_merge` is enabled under node-id / e-graph
    /// and a greatest-fixpoint state correspondence proved two flops
    /// sequentially equivalent beyond the exact reset-defined self-hold
    /// class. Sourced from `Module::bisimulation_flops_merged`.
    pub bisimulation_flops_merged: u32,

    /// Number of duplicate deterministic FSM blocks merged away by
    /// the post-construction endpoint-preserving state-sharing pass.
    /// Sourced from `Module::fsms_merged`.
    pub fsms_merged: u32,

    /// Number of duplicate combinational gates merged away by the
    /// post-construction bounded semantic-sharing pass. Sourced
    /// from `Module::semantic_gates_merged`.
    pub semantic_gates_merged: u32,

    /// Number of times the `Associative` factorization layer
    /// fired during construction. Each fire is one `intern_gate`
    /// call on an associative op where at least one same-op
    /// same-width inner gate was spliced into the outer operand
    /// list. Zero at factorization levels below `Associative`.
    /// Sourced from `Module::flatten_associative_applied`.
    pub flatten_associative_applied: u64,

    // --- Per-knob probability-roll counters --------------------
    /// Attempt count per probability knob. Keyed by the knob's
    /// canonical string name (matches `Config` field, e.g.
    /// `"flop_prob"`). Every `gen_bool(cfg.<prob>)` site during
    /// construction routes through the `roll_knob` helper which
    /// increments attempts (and fires below on success). Empty
    /// knobs (no attempts taken during this module) are omitted
    /// from the map to keep JSON dumps compact.
    ///
    /// Read this with `knob_roll_fires` to compute the empirical
    /// fire-rate per knob — should converge to the configured
    /// probability across large seed sweeps. Divergences indicate
    /// either low sample count or a latent gate that prevents
    /// the roll from reaching its decision site (e.g.
    /// `flop_prob` rolls are gated by `flop_allowed`, so a module
    /// that hits `max_flops_per_module` early will see fewer
    /// attempts than expected).
    pub knob_roll_attempts: BTreeMap<String, u64>,
    /// Fire count per probability knob — the subset of attempts
    /// that returned `true`. See `knob_roll_attempts` for keying
    /// and interpretation.
    pub knob_roll_fires: BTreeMap<String, u64>,

    // --- Block-build counters -----------------------------------
    /// Number of priority-encoder block instances built in this
    /// module. Measures the `priority_encoder_prob` knob directly.
    pub num_priority_encoder_blocks: u32,
    /// Number of combinational one-hot-style mux blocks built.
    /// Together with `num_comb_muxes_encoded` measures the
    /// `comb_mux_encoding_prob` knob (the ratio should converge
    /// to the knob value over large seed sweeps).
    pub num_comb_muxes_one_hot: u32,
    /// Number of combinational encoded-style (chained-ternary)
    /// mux blocks built.
    pub num_comb_muxes_encoded: u32,
    /// Number of procedural combinational `case` mux blocks built.
    pub num_case_mux_blocks: u32,
    /// Number of procedural combinational `casez` mux blocks built.
    pub num_casez_mux_blocks: u32,
    /// Number of procedural combinational statically bounded for-fold
    /// blocks built.
    pub num_for_fold_blocks: u32,

    /// `STRUCTURED-EMISSION-EXPANSION.2b.2a` — number of combinational
    /// gates this module emits as a `function automatic` projection
    /// (`Module.function_emit_gates.len()`; the `function_emit_prob`
    /// knob). Zero unless `function_emit_prob > 0.0` selected qualifying
    /// gates. A post-hoc structural count of an emitter-surface
    /// annotation — adding it changes no emitted RTL (default-off
    /// byte-identical). `#[serde(default)]` keeps the introspection
    /// projection additive (schema `1.8`).
    #[serde(default)]
    pub num_emitted_combinational_functions: usize,

    /// `STRUCTURED-EMISSION-EXPANSION.4b.2a` — number of `{N{x}}`
    /// replication gates this module emits as a single-level `generate for`
    /// loop projection (`Module.generate_loop_gates.len()`; the
    /// `generate_loop_emit_prob` knob). Zero unless
    /// `generate_loop_emit_prob > 0.0` selected qualifying replications. A
    /// post-hoc structural count of an emitter-surface annotation — adding
    /// it changes no emitted RTL (default-off byte-identical).
    /// `#[serde(default)]` keeps the introspection projection additive
    /// (schema `1.9`).
    #[serde(default)]
    pub num_emitted_generate_loops: usize,

    /// `STRUCTURED-EMISSION-EXPANSION.6b.2a` — number of combinational
    /// gates this module emits as a `task automatic` projection
    /// (`Module.task_emit_gates.len()`; the `task_emit_prob` knob). Zero
    /// unless `task_emit_prob > 0.0` selected qualifying gates. A post-hoc
    /// structural count of an emitter-surface annotation — adding it changes
    /// no emitted RTL (default-off byte-identical). `#[serde(default)]` keeps
    /// the introspection projection additive (schema `1.10`).
    #[serde(default)]
    pub num_emitted_combinational_tasks: usize,

    /// `STRUCTURED-EMISSION-EXPANSION.10b.2` — number of combinational
    /// **cones** this module emits as a multi-gate `function automatic`
    /// projection (`Module.cone_function_gates.len()`; the
    /// `cone_function_emit_prob` knob, decision `0016`). Each counted cone is
    /// a root gate plus its absorbed single-use interior gates rendered as one
    /// behaviour-preserving `function automatic` over the cone's boundary
    /// leaves. Zero unless `cone_function_emit_prob > 0.0` selected qualifying
    /// cones. Separate from `num_emitted_combinational_functions` (the
    /// single-gate `function_emit_prob` surface). A post-hoc structural count
    /// of an emitter-surface annotation — adding it changes no emitted RTL
    /// (default-off byte-identical). `#[serde(default)]` keeps the
    /// introspection projection additive (schema `1.11`).
    #[serde(default)]
    pub num_emitted_cone_functions: usize,
}

/// Structural summary of a generated multi-module `Design`.
/// These metrics quantify the current Phase 4 composition slice
/// directly: library size, wrapper usage, reuse, under-instantiation,
/// control fanout, and weighted child complexity.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct DesignMetrics {
    /// Design identifier (the top module name).
    pub design: String,

    // --- Hierarchy-aware identity instrumentation --------------
    /// One deterministic canonical signature per module in
    /// `design.modules`, in the same order. Two modules with the
    /// same signature have isomorphic ports, nodes, drives, flops,
    /// and instance interfaces (instance child-module names are
    /// excluded from the hash, so structurally-identical modules
    /// are detected even when their instance graphs reference
    /// distinctly-named children).
    ///
    /// This is the first slice of hierarchy-aware identity: pure
    /// observation, no behaviour change. Future slices will use
    /// these signatures to dedupe `Design::modules` at construction
    /// time when `IdentityMode::NodeId` is active.
    pub canonical_module_signatures: Vec<u64>,
    /// One deterministic bounded-semantic signature per module when
    /// the module is inside the current semantic proof boundary:
    /// pure combinational, concrete, same-port-id interface, input
    /// support <= 12 bits, reachable cone <= 128 nodes, output
    /// widths <= 128, and either instance-free or a bounded
    /// hierarchy wrapper whose children are also inside this proof
    /// boundary. `None` means the module is outside that boundary,
    /// not that it failed validation.
    pub semantic_module_signatures: Vec<Option<u64>>,
    /// Number of distinct values in `canonical_module_signatures`.
    /// Equal to `num_modules` when every module is structurally
    /// distinct; strictly less when the planner emitted duplicate
    /// structures.
    pub num_distinct_module_signatures: usize,
    /// Pairs of modules in `design.modules` that share the same
    /// canonical signature (sum of `count choose 2` over signatures
    /// with `count > 1`). Always 0 when every module is distinct.
    pub num_structurally_duplicate_module_pairs: usize,
    /// Pairs of modules that share the same bounded semantic module
    /// proof class. This can be non-zero even when structural
    /// signatures differ, for example `out = in` vs. `out = ~~in`
    /// inside the current pure-combinational proof boundary.
    pub num_semantically_duplicate_module_pairs: usize,
    /// `IDENTITY-DEEPENING.3b.2b.2a` — one sequential proof-class id per
    /// module. `Some(id)` for a module inside the bounded whole-leaf-module
    /// sequential-equivalence proof boundary (a stateful flops-only leaf, all
    /// flops settled + reset-defined, no memories/FSMs/instances/params/
    /// aggregates/multi-clock — and not the top); two modules sharing an id were
    /// proven sequentially (observationally) equivalent by the cross-module
    /// bisimulation. `None` means the module is outside that boundary. There is
    /// no per-module canonical sequential proof (equivalence is decided
    /// pairwise), so the id is the deterministic class id (a hash of the class's
    /// lex-smallest member name). RTL-invisible; default-off path unaffected.
    #[serde(default)]
    pub sequential_module_proof_signatures: Vec<Option<u64>>,
    /// `IDENTITY-DEEPENING.3b.2b.2a` — pairs of modules proven sequentially
    /// equivalent (sum of `count choose 2` over the cross-module proof classes).
    /// Reducible to 0 by `dedup_sequential_modules` on a supported design. 0 for
    /// every design without a distinct-but-sequentially-equivalent stateful leaf
    /// pair.
    #[serde(default)]
    pub num_sequentially_duplicate_module_pairs: usize,
    /// Phase 5: number of `Design::modules` carrying a width
    /// `parameter` (`Module::param_env.is_some()`). 0 for every
    /// default-off / pre-Phase-5 design.
    pub num_width_parameterized_modules: usize,
    /// Phase 5: number of `Instance`s across the design that carry a
    /// non-empty `param_bindings` (i.e. instantiate a parameterized
    /// child with an explicit `#(.W(v))` override). 0 when the feature
    /// is off.
    pub num_param_override_instances: usize,
    /// Phase 5b: number of `Design::modules` carrying a packed-aggregate
    /// emitter projection (`Module::aggregate_layout.is_some()`). 0 for
    /// every default-off / pre-Phase-5b design.
    pub num_packed_aggregate_modules: usize,
    /// AGGREGATE-ARRAY-PACKING: of the packed-aggregate modules above,
    /// how many use the `ArrayPacked` (packed-array) kind rather than
    /// `StructPacked`. 0 unless `aggregate_array_prob > 0.0` selected a
    /// uniform-width array projection.
    pub num_array_packed_aggregate_modules: usize,
    /// Phase 6: number of `Design::modules` carrying an inferrable
    /// `Memory` block (`!Module::memories.is_empty()`). 0 for every
    /// default-off / pre-Phase-6 design.
    pub num_memory_modules: usize,
    /// Phase 6: number of `Design::modules` carrying a generated-
    /// encoding `Fsm` block (`!Module::fsms.is_empty()`). 0 for every
    /// default-off / pre-`.3` design.
    pub num_fsm_modules: usize,
    /// `CAPABILITY-BREADTH-EXPANSION.2b` (decision `0024`): number of
    /// `Design::modules` carrying at least one **Mealy** FSM (a `Fsm`
    /// with `mealy_outputs.is_some()` — an output decoded over the
    /// current state *and* input). 0 for every default-off design
    /// (`fsm_mealy_prob == 0.0`). `<= num_fsm_modules`.
    pub num_mealy_fsm_modules: usize,

    // --- Overall size ------------------------------------------
    pub num_modules: usize,
    pub num_library_modules: usize,
    pub num_internal_modules: usize,
    pub num_leaf_modules: usize,
    pub num_instances: usize,
    pub num_unique_instantiated_modules: usize,
    pub num_unused_module_definitions: usize,
    pub num_unused_leaf_modules: usize,
    pub num_reused_instance_slots: usize,
    pub num_profiled_module_definitions: usize,
    pub num_profiled_instantiated_modules: usize,
    pub num_profiled_instance_slots: usize,
    pub num_internal_module_occurrences: usize,
    pub num_leaf_module_occurrences: usize,

    // --- Composition ratios ------------------------------------
    pub library_coverage_fraction: f64,
    pub unused_library_fraction: f64,
    pub instance_reuse_fraction: f64,
    pub instance_to_library_ratio: f64,
    pub avg_instances_per_unique_instantiated_module: f64,
    pub num_single_use_instantiated_modules: usize,
    pub num_multiuse_instantiated_modules: usize,
    pub single_use_instantiated_module_fraction: f64,
    pub profiled_instantiated_module_fraction: f64,
    pub profiled_instance_fraction: f64,

    // --- Hierarchy shape ---------------------------------------
    pub realized_min_leaf_depth: usize,
    pub realized_max_leaf_depth: usize,
    pub avg_leaf_depth: f64,
    pub max_module_depth: usize,
    pub avg_child_instances_per_internal_module: f64,
    pub min_child_instances_per_internal_module: usize,
    pub max_child_instances_per_internal_module: usize,
    pub module_defs_by_depth: BTreeMap<usize, usize>,
    pub module_occurrences_by_depth: BTreeMap<usize, usize>,
    pub leaf_module_occurrences_by_depth: BTreeMap<usize, usize>,
    pub instance_slots_by_parent_depth: BTreeMap<usize, usize>,
    pub avg_child_instances_by_parent_depth: BTreeMap<usize, f64>,
    pub min_child_instances_by_parent_depth: BTreeMap<usize, usize>,
    pub max_child_instances_by_parent_depth: BTreeMap<usize, usize>,
    pub child_instances_per_internal_module_histogram: BTreeMap<usize, usize>,

    // --- Top interface -----------------------------------------
    pub top_inputs: usize,
    pub top_data_inputs: usize,
    pub top_outputs: usize,
    pub top_clock_inputs: usize,
    pub top_reset_inputs: usize,
    pub top_local_flops: usize,
    pub clock_fanout_instances: usize,
    pub reset_fanout_instances: usize,
    pub top_total_child_data_input_bindings: usize,
    pub top_child_input_bindings_from_parent_ports: usize,
    pub top_child_input_bindings_from_instance_outputs: usize,
    pub top_child_input_bindings_from_mixed_support: usize,
    pub top_child_input_bindings_from_constants: usize,
    pub top_child_input_bindings_from_parent_composed_logic: usize,
    pub top_child_input_bindings_from_stateful_parent_composed_mixed_support: usize,
    pub top_child_input_bindings_from_parent_flops: usize,
    pub top_child_input_bindings_from_registered_instance_outputs: usize,
    pub top_child_input_bindings_from_registered_parent_composed_logic: usize,
    pub top_child_input_bindings_from_registered_mixed_support: usize,
    pub top_child_input_bindings_from_registered_sibling_mixed_support: usize,
    pub top_child_input_bindings_from_registered_multistage_parent_composed_logic: usize,
    pub top_child_input_bindings_from_registered_three_stage_parent_composed_logic: usize,
    pub top_child_input_bindings_from_registered_multistage_mixed_support: usize,
    pub top_child_input_bindings_from_registered_multistage_instance_outputs: usize,
    pub top_child_input_bindings_from_registered_multistage_parent_cone_instances: usize,
    pub top_child_input_bindings_from_registered_multistage_parent_composed_parent_cone_instances:
        usize,
    pub top_child_input_bindings_from_parent_cone_instances: usize,
    pub top_child_input_bindings_from_parent_cone_instance_mixed_support: usize,
    pub top_child_input_bindings_from_parent_cone_instances_through_parent_flops: usize,
    pub top_child_input_bindings_from_parent_cone_instance_flop_mixed_support: usize,
    pub top_child_input_bindings_from_registered_parent_cone_instances: usize,
    pub top_child_input_bindings_from_registered_parent_cone_instance_mixed_support: usize,
    pub top_parent_cone_instances: usize,
    pub top_outputs_reaching_parent_cone_instances: usize,
    pub top_outputs_reaching_parent_cone_instance_mixed_support: usize,
    pub top_outputs_reaching_parent_cone_instances_through_parent_flops: usize,
    pub top_outputs_reaching_parent_cone_instances_through_parent_flops_with_mixed_support: usize,
    pub top_direct_instance_output_drives: usize,
    pub top_parent_composed_outputs: usize,
    pub top_parent_port_composed_outputs: usize,
    pub top_parent_port_composed_outputs_through_parent_flops: usize,
    pub top_outputs_reaching_instance_outputs: usize,
    pub top_outputs_without_instance_outputs: usize,
    pub top_instance_output_dependency_fraction: f64,
    pub top_parent_composed_output_fraction: f64,
    pub top_parent_port_composed_output_fraction: f64,
    pub top_parent_port_composed_parent_flop_output_fraction: f64,
    pub top_instance_output_child_input_binding_fraction: f64,
    pub top_parent_composed_child_input_binding_fraction: f64,
    pub top_registered_instance_output_child_input_binding_fraction: f64,
    pub top_registered_parent_composed_child_input_binding_fraction: f64,
    pub top_registered_mixed_support_child_input_binding_fraction: f64,
    pub top_registered_sibling_mixed_support_child_input_binding_fraction: f64,
    pub top_registered_multistage_parent_composed_child_input_binding_fraction: f64,
    pub top_registered_three_stage_parent_composed_child_input_binding_fraction: f64,
    pub top_registered_multistage_mixed_support_child_input_binding_fraction: f64,
    pub top_registered_multistage_instance_output_child_input_binding_fraction: f64,
    pub top_registered_multistage_parent_cone_instance_child_input_binding_fraction: f64,
    pub top_registered_multistage_parent_composed_parent_cone_instance_child_input_binding_fraction:
        f64,
    pub top_registered_parent_cone_instance_child_input_binding_fraction: f64,
    pub top_registered_parent_cone_instance_mixed_support_child_input_binding_fraction: f64,
    pub top_parent_cone_instance_mixed_support_child_input_binding_fraction: f64,
    pub top_parent_cone_instance_flop_child_input_binding_fraction: f64,
    pub top_parent_cone_instance_flop_mixed_support_child_input_binding_fraction: f64,
    pub top_parent_cone_instance_output_fraction: f64,
    pub top_parent_cone_instance_mixed_support_output_fraction: f64,
    pub top_parent_cone_instance_flop_output_fraction: f64,
    pub top_parent_cone_instance_flop_mixed_support_output_fraction: f64,
    pub avg_instance_output_support_per_top_output: f64,
    pub max_instance_output_support_per_top_output: usize,

    // --- Composition across the whole hierarchy ----------------
    pub hierarchy_direct_instance_output_drives: usize,
    pub hierarchy_parent_composed_outputs: usize,
    pub hierarchy_parent_port_composed_outputs: usize,
    pub hierarchy_parent_port_composed_outputs_through_parent_flops: usize,
    pub module_occurrences_with_parent_composed_outputs: usize,
    pub hierarchy_parent_cone_instances: usize,
    pub max_parent_cone_instances_per_internal_module: usize,
    pub hierarchy_outputs_reaching_parent_cone_instances: usize,
    pub hierarchy_outputs_reaching_parent_cone_instance_mixed_support: usize,
    pub hierarchy_outputs_reaching_parent_cone_instances_through_parent_flops: usize,
    pub hierarchy_outputs_reaching_parent_cone_instances_through_parent_flops_with_mixed_support:
        usize,
    pub hierarchy_parent_local_flops: usize,
    pub internal_module_occurrences_with_local_flops: usize,
    pub avg_instance_output_support_per_hierarchy_output: f64,
    pub max_instance_output_support_per_hierarchy_output: usize,
    pub hierarchy_parent_port_composed_output_fraction: f64,
    pub hierarchy_parent_port_composed_parent_flop_output_fraction: f64,
    pub hierarchy_parent_cone_instance_output_fraction: f64,
    pub hierarchy_parent_cone_instance_mixed_support_output_fraction: f64,
    pub hierarchy_parent_cone_instance_flop_output_fraction: f64,
    pub hierarchy_parent_cone_instance_flop_mixed_support_output_fraction: f64,

    // --- Child interface load ----------------------------------
    pub total_child_data_input_bindings: usize,
    pub dep_bearing_child_input_bindings: usize,
    pub child_input_bindings_from_parent_ports: usize,
    pub child_input_bindings_from_instance_outputs: usize,
    pub child_input_bindings_from_mixed_support: usize,
    pub child_input_bindings_from_constants: usize,
    pub child_input_bindings_from_parent_composed_logic: usize,
    pub child_input_bindings_from_stateful_parent_composed_mixed_support: usize,
    pub child_input_bindings_from_parent_flops: usize,
    pub child_input_bindings_from_registered_instance_outputs: usize,
    pub child_input_bindings_from_registered_parent_composed_logic: usize,
    pub child_input_bindings_from_registered_mixed_support: usize,
    pub child_input_bindings_from_registered_sibling_mixed_support: usize,
    pub child_input_bindings_from_registered_multistage_parent_composed_logic: usize,
    pub child_input_bindings_from_registered_three_stage_parent_composed_logic: usize,
    pub child_input_bindings_from_registered_multistage_mixed_support: usize,
    pub child_input_bindings_from_registered_multistage_instance_outputs: usize,
    pub child_input_bindings_from_registered_multistage_parent_cone_instances: usize,
    pub child_input_bindings_from_registered_multistage_parent_composed_parent_cone_instances:
        usize,
    pub child_input_bindings_from_parent_cone_instances: usize,
    pub child_input_bindings_from_parent_cone_instance_mixed_support: usize,
    pub child_input_bindings_from_parent_cone_instances_through_parent_flops: usize,
    pub child_input_bindings_from_parent_cone_instance_flop_mixed_support: usize,
    pub child_input_bindings_from_registered_parent_cone_instances: usize,
    pub child_input_bindings_from_registered_parent_cone_instance_mixed_support: usize,
    /// Total child output-port slots across instantiated children.
    /// This counts the raw observable supply available from child
    /// modules, not necessarily the number of outputs that are still
    /// wired through directly at the top boundary.
    pub total_child_output_exposures: usize,
    pub avg_child_data_inputs_per_instance: f64,
    pub avg_child_outputs_per_instance: f64,
    pub dep_bearing_child_input_binding_fraction: f64,
    pub instance_output_child_input_binding_fraction: f64,
    pub parent_port_child_input_binding_fraction: f64,
    pub parent_composed_child_input_binding_fraction: f64,
    pub stateful_parent_composed_mixed_support_child_input_binding_fraction: f64,
    pub top_stateful_parent_composed_mixed_support_child_input_binding_fraction: f64,
    pub parent_flop_child_input_binding_fraction: f64,
    pub registered_instance_output_child_input_binding_fraction: f64,
    pub registered_parent_composed_child_input_binding_fraction: f64,
    pub registered_mixed_support_child_input_binding_fraction: f64,
    pub registered_sibling_mixed_support_child_input_binding_fraction: f64,
    pub registered_multistage_parent_composed_child_input_binding_fraction: f64,
    pub registered_three_stage_parent_composed_child_input_binding_fraction: f64,
    pub registered_multistage_mixed_support_child_input_binding_fraction: f64,
    pub registered_multistage_instance_output_child_input_binding_fraction: f64,
    pub registered_multistage_parent_cone_instance_child_input_binding_fraction: f64,
    pub registered_multistage_parent_composed_parent_cone_instance_child_input_binding_fraction:
        f64,
    pub registered_parent_cone_instance_child_input_binding_fraction: f64,
    pub registered_parent_cone_instance_mixed_support_child_input_binding_fraction: f64,
    pub parent_cone_instance_child_input_binding_fraction: f64,
    pub parent_cone_instance_mixed_support_child_input_binding_fraction: f64,
    pub parent_cone_instance_flop_child_input_binding_fraction: f64,
    pub parent_cone_instance_flop_mixed_support_child_input_binding_fraction: f64,
    pub top_parent_flop_child_input_binding_fraction: f64,
    pub top_parent_cone_instance_child_input_binding_fraction: f64,

    // --- Sequential / combinational mix ------------------------
    pub num_sequential_leaf_modules: usize,
    pub num_combinational_leaf_modules: usize,
    pub num_sequential_instances: usize,
    pub num_combinational_instances: usize,
    pub sequential_instance_fraction: f64,

    // --- Weighted child complexity -----------------------------
    pub total_instantiated_child_nodes: usize,
    pub total_instantiated_child_flops: usize,
    pub avg_nodes_per_instance: f64,
    pub avg_flops_per_instance: f64,
    pub max_nodes_per_instance: usize,
    pub max_flops_per_instance: usize,

    // --- Reuse histogram ---------------------------------------
    /// Instance count per instantiated child module definition.
    pub instantiated_module_histogram: BTreeMap<String, usize>,
}

/// Compute metrics from a generated `Module`. Pure function — does
/// not modify the module.
/// Structural CDC synchronizer chain lengths. A chain is two or more
/// flops in a non-zero domain where each later stage's D input is the
/// previous stage's Q. This recognizes both the default 2-flop
/// primitive and the N-flop extension.
fn cdc_synchronizer_chain_stage_lengths(m: &Module) -> Vec<usize> {
    let mut flops_by_domain: BTreeMap<u32, Vec<crate::ir::FlopId>> = BTreeMap::new();
    for (flop_id, domain) in &m.flop_domains {
        if *domain != 0 {
            flops_by_domain.entry(*domain).or_default().push(*flop_id);
        }
    }

    let mut lengths = Vec::new();
    for flops in flops_by_domain.values() {
        let domain_flops: BTreeSet<_> = flops.iter().copied().collect();
        let q_to_flop: BTreeMap<_, _> = flops
            .iter()
            .map(|flop_id| (m.flops[*flop_id as usize].q, *flop_id))
            .collect();
        let mut consumers_by_d: BTreeMap<crate::ir::NodeId, Vec<crate::ir::FlopId>> =
            BTreeMap::new();
        for flop_id in flops {
            if let Some(d) = m.flops[*flop_id as usize].d {
                consumers_by_d.entry(d).or_default().push(*flop_id);
            }
        }

        for start in flops {
            let start_d = m.flops[*start as usize].d;
            if start_d
                .and_then(|node| q_to_flop.get(&node).copied())
                .is_some()
            {
                continue;
            }

            let mut len = 1usize;
            let mut current = *start;
            let mut seen = BTreeSet::from([current]);
            loop {
                let q = m.flops[current as usize].q;
                let nexts: Vec<_> = consumers_by_d
                    .get(&q)
                    .into_iter()
                    .flat_map(|ids| ids.iter().copied())
                    .filter(|id| domain_flops.contains(id) && !seen.contains(id))
                    .collect();
                if nexts.len() != 1 {
                    break;
                }
                current = nexts[0];
                seen.insert(current);
                len += 1;
            }
            if len >= 2 {
                lengths.push(len);
            }
        }
    }
    lengths
}

pub fn compute(m: &Module) -> Metrics {
    let cdc_chain_lengths = cdc_synchronizer_chain_stage_lengths(m);
    let mut out = Metrics {
        module: m.name.clone(),
        num_inputs: m.inputs.len(),
        num_outputs: m.outputs.len(),
        num_nodes: m.nodes.len(),
        num_flops: m.flops.len(),
        num_instances: m.instances.len(),
        // `MULTI-CLOCK-CDC.3b.2` — surface multi-clock + CDC
        // facts into the per-module Metrics so `tool_matrix`'s
        // `summarize_coverage` can light the new coverage facts
        // without needing the typed Module IR in scope.
        num_clock_domains: m.clock_domains.len(),
        num_cdc_2_flop_synchronizers: cdc_chain_lengths
            .iter()
            .filter(|stages| **stages == 2)
            .count(),
        num_cdc_synchronizer_chains: cdc_chain_lengths.len(),
        max_cdc_synchronizer_stages: cdc_chain_lengths.iter().copied().max().unwrap_or(0),
        ..Default::default()
    };

    // One pass: count nodes by kind, constants by shape, muxes by
    // shape, concats by shape.
    for node in &m.nodes {
        match node {
            Node::PrimaryInput { .. } => out.num_primary_inputs += 1,
            Node::FlopQ { .. } => out.num_flop_q_refs += 1,
            Node::MemRead { .. } => {}
            // Phase 6 .3.2a: opaque FSM-output leaf; the
            // `num_fsm_modules` metric is .3.4, not this slice.
            Node::FsmOut { .. } => {}
            Node::InstanceOutput { .. } => out.num_instance_outputs += 1,
            Node::Constant { width, value } => {
                out.num_constants += 1;
                *out.constants_by_width.entry(*width).or_insert(0) += 1;
                let all_ones: u128 = if *width >= 128 {
                    u128::MAX
                } else {
                    (1u128 << width) - 1
                };
                if *value == 0 {
                    out.constants_zero += 1;
                } else if *value == all_ones {
                    out.constants_all_ones += 1;
                } else {
                    out.constants_other += 1;
                }
            }
            Node::Gate { op, operands, .. } => {
                out.num_gates += 1;
                let kind = gate_kind_name(*op).to_string();
                *out.gates_by_kind.entry(kind.clone()).or_insert(0) += 1;

                // Operand-arity histogram + per-kind max.
                let arity = operands.len();
                *out.gate_operand_count_histogram.entry(arity).or_insert(0) += 1;
                if arity > out.max_gate_operand_count {
                    out.max_gate_operand_count = arity;
                }
                let entry = out.max_operand_count_by_kind.entry(kind).or_insert(0);
                if arity > *entry {
                    *entry = arity;
                }

                if matches!(op, GateOp::Mux) && operands.len() == 3 {
                    out.num_muxes_2to1 += 1;
                    if operands[1] == operands[2] {
                        out.num_muxes_degenerate += 1;
                    }
                }
                // `operand_duplication_rate` (Add/Mul) telemetry: an
                // operand `NodeId` repeated across two or more slots.
                // Operand lists are small, so an O(n²) scan is cheaper
                // than allocating a set.
                if matches!(op, GateOp::Add | GateOp::Mul)
                    && operands
                        .iter()
                        .enumerate()
                        .any(|(i, slot)| operands[i + 1..].contains(slot))
                {
                    out.num_operator_gates_with_duplicate_operands += 1;
                }
                if matches!(op, GateOp::Concat) && !operands.is_empty() {
                    if operands.iter().all(|o| *o == operands[0]) {
                        out.num_concats_replication += 1;
                    } else {
                        out.num_concats_heterogeneous += 1;
                    }
                }
                if matches!(op, GateOp::Shl | GateOp::Shr) && operands.len() == 2 {
                    if matches!(m.nodes[operands[1] as usize], Node::Constant { .. }) {
                        out.num_constant_shift_gates += 1;
                    } else {
                        out.num_variable_shift_gates += 1;
                    }
                }
            }
        }
    }

    // Flops: per-kind and per-mux-shape counters.
    for f in &m.flops {
        match f.kind {
            crate::ir::FlopKind::ZeroDefault => out.flops_zero_default += 1,
            crate::ir::FlopKind::QFeedback => out.flops_qfeedback += 1,
        }
        match &f.mux {
            crate::ir::FlopMux::None => out.flops_mux_none += 1,
            crate::ir::FlopMux::OneHot(_) => out.flops_mux_one_hot += 1,
            crate::ir::FlopMux::Encoded { .. } => out.flops_mux_encoded += 1,
        }
    }

    // Combinational-depth pass. `m.nodes` is in topological order by
    // construction (Rule 1: combinational no-loop, arena-index
    // monotonicity). A single forward walk assigns each node its
    // depth as `max(operand depth) + 1`. Leaves (PrimaryInput,
    // Constant, FlopQ) are depth 0 — FlopQ acts as a leaf because
    // the clock edge breaks the Q→D loop temporally, so for
    // combinational depth reasoning the Q is a zero-depth source.
    let mut depth = vec![0usize; m.nodes.len()];
    for (idx, node) in m.nodes.iter().enumerate() {
        if let Node::Gate { operands, .. } = node {
            let max_operand = operands
                .iter()
                .map(|o| depth[*o as usize])
                .max()
                .unwrap_or(0);
            depth[idx] = max_operand + 1;
            *out.gate_depth_histogram.entry(depth[idx]).or_insert(0) += 1;
            if depth[idx] > out.max_gate_depth {
                out.max_gate_depth = depth[idx];
            }
        }
    }

    // Fanout pass: walk every Gate plus each emitted flop D-input to
    // build a use-count per NodeId. Primary inputs and constants are
    // included (they can have fanout like any other node). Output
    // drives also count as a use. Flop-mux operand metadata is
    // intentionally ignored here: after finalisation it is summary
    // shape information, not an emitted consumer.
    let mut fanout = vec![0usize; m.nodes.len()];
    for node in &m.nodes {
        if let Node::Gate { operands, .. } = node {
            for &op in operands {
                fanout[op as usize] += 1;
            }
        }
    }
    for f in &m.flops {
        if let Some(d) = f.d {
            fanout[d as usize] += 1;
        }
    }
    for (_, root) in &m.drives {
        fanout[*root as usize] += 1;
    }
    out.num_shared_nodes = fanout.iter().filter(|c| **c >= 2).count();
    out.max_fanout = fanout.iter().copied().max().unwrap_or(0);
    out.avg_fanout = if !fanout.is_empty() {
        fanout.iter().sum::<usize>() as f64 / fanout.len() as f64
    } else {
        0.0
    };

    // AST-instance saturation from the dedup tables.
    out.max_gate_ast_multiplicity = m
        .gate_instances
        .values()
        .map(|v| v.len())
        .max()
        .unwrap_or(0);
    out.max_constant_ast_multiplicity = m
        .const_instances
        .values()
        .map(|v| v.len())
        .max()
        .unwrap_or(0);

    // Block-build counters (populated live during construction).
    out.num_priority_encoder_blocks = m.priority_encoder_built;
    out.num_comb_muxes_one_hot = m.comb_mux_one_hot_built;
    out.num_comb_muxes_encoded = m.comb_mux_encoded_built;
    out.num_case_mux_blocks = m.case_mux_built;
    out.num_casez_mux_blocks = m.casez_mux_built;
    out.num_for_fold_blocks = m.for_fold_built;

    // `STRUCTURED-EMISSION-EXPANSION.2b.2a` — count of gates the emitter
    // projects as a `function automatic` (an emitter-surface annotation;
    // structural, post-hoc, RTL-invisible).
    out.num_emitted_combinational_functions = m.function_emit_gates.len();

    // `STRUCTURED-EMISSION-EXPANSION.4b.2a` — count of `{N{x}}` replication
    // gates the emitter projects as a `generate for` loop (an emitter-surface
    // annotation; structural, post-hoc, RTL-invisible).
    out.num_emitted_generate_loops = m.generate_loop_gates.len();

    // `STRUCTURED-EMISSION-EXPANSION.6b.2a` — count of gates the emitter
    // projects as a `task automatic` (an emitter-surface annotation;
    // structural, post-hoc, RTL-invisible).
    out.num_emitted_combinational_tasks = m.task_emit_gates.len();

    // `STRUCTURED-EMISSION-EXPANSION.10b.2` — count of combinational cones the
    // emitter projects as a multi-gate `function automatic` (an emitter-surface
    // annotation; structural, post-hoc, RTL-invisible).
    out.num_emitted_cone_functions = m.cone_function_gates.len();

    // ConstantFold factorization layer: counter sourced live from
    // `intern_gate`. Zero at levels below `ConstantFold`.
    out.fold_identities_applied = m.fold_identities_applied;
    out.peephole_rewrites_applied = m.peephole_rewrites_applied;
    out.nodes_compacted = m.nodes_compacted;
    out.flops_merged = m.flops_merged;
    out.bisimulation_flops_merged = m.bisimulation_flops_merged;
    out.fsms_merged = m.fsms_merged;
    out.semantic_gates_merged = m.semantic_gates_merged;
    out.flatten_associative_applied = m.flatten_associative_applied;

    // Per-knob attempt/fire counters. Convert enum keys to strings
    // for serialisation. Non-empty knobs only.
    for (knob, count) in &m.knob_rolls.attempts {
        out.knob_roll_attempts
            .insert(knob.name().to_string(), *count);
    }
    for (knob, count) in &m.knob_rolls.fires {
        out.knob_roll_fires.insert(knob.name().to_string(), *count);
    }

    // Associative-flattening-opportunities scan. For every
    // associative gate, count same-op same-width inner-gate slots
    // that the current duplicate policy would allow us to flatten.
    // Add/Mul are special: at strict `operand_duplication_rate`
    // the live Associative layer intentionally preserves nested
    // shapes that would become duplicate-bearing if flattened.
    for node in &m.nodes {
        if let Node::Gate {
            op,
            operands,
            width,
            ..
        } = node
        {
            if !matches!(
                op,
                GateOp::And | GateOp::Or | GateOp::Xor | GateOp::Add | GateOp::Mul
            ) {
                continue;
            }
            let nested_slots: Vec<_> = operands
                .iter()
                .copied()
                .filter(|operand_id| {
                    matches!(
                        &m.nodes[*operand_id as usize],
                        Node::Gate {
                            op: inner_op,
                            width: inner_w,
                            ..
                        } if inner_op == op && inner_w == width
                    )
                })
                .collect();
            if nested_slots.is_empty() {
                continue;
            }
            if matches!(op, GateOp::Add | GateOp::Mul) && m.operand_duplication_rate < 1.0 {
                use std::collections::HashSet;

                let mut flat: Vec<crate::ir::NodeId> = Vec::with_capacity(operands.len());
                for &operand_id in operands {
                    match &m.nodes[operand_id as usize] {
                        Node::Gate {
                            op: inner_op,
                            operands: inner_ops,
                            width: inner_w,
                            ..
                        } if inner_op == op && inner_w == width => {
                            flat.extend(inner_ops.iter().copied());
                        }
                        _ => flat.push(operand_id),
                    }
                }
                let mut seen = HashSet::new();
                if flat.iter().any(|id| !seen.insert(*id)) {
                    continue;
                }
            }
            out.nested_associative_operand_count += nested_slots.len();
        }
    }

    out
}

/// Compute design-level hierarchy metrics. For the current Phase 4
/// slice, these describe the quality of wrapper composition without
/// requiring manual SV inspection.
pub fn compute_design(design: &Design) -> DesignMetrics {
    let modules_by_name: BTreeMap<_, _> = design
        .modules
        .iter()
        .map(|module| (module.name.as_str(), module))
        .collect();
    let top = design
        .modules
        .iter()
        .find(|module| module.name == design.top)
        .expect("design top must exist");
    let library: Vec<_> = design
        .modules
        .iter()
        .filter(|module| module.name != design.top)
        .collect();
    let num_leaf_modules = library
        .iter()
        .filter(|module| module.instances.is_empty())
        .count();
    let num_internal_modules = design.modules.len().saturating_sub(num_leaf_modules);

    let canonical_module_signatures: Vec<u64> = design
        .modules
        .iter()
        .map(canonical_module_signature)
        .collect();
    let mut signature_counts: BTreeMap<u64, usize> = BTreeMap::new();
    for sig in &canonical_module_signatures {
        *signature_counts.entry(*sig).or_insert(0) += 1;
    }
    let num_distinct_module_signatures = signature_counts.len();
    let num_structurally_duplicate_module_pairs = signature_counts
        .values()
        .filter(|count| **count > 1)
        .map(|count| count * (count - 1) / 2)
        .sum();
    let semantic_module_proofs: Vec<_> = design
        .modules
        .iter()
        .map(|module| semantic_module_proof_with_modules(module, &modules_by_name))
        .collect();
    let semantic_module_signatures: Vec<_> = semantic_module_proofs
        .iter()
        .map(|proof| proof.as_ref().map(semantic_module_signature_hash))
        .collect();
    let mut semantic_proof_counts: BTreeMap<SemanticModuleProof, usize> = BTreeMap::new();
    for proof in semantic_module_proofs.into_iter().flatten() {
        *semantic_proof_counts.entry(proof).or_insert(0) += 1;
    }
    let num_semantically_duplicate_module_pairs = semantic_proof_counts
        .values()
        .filter(|count| **count > 1)
        .map(|count| count * (count - 1) / 2)
        .sum();
    // IDENTITY-DEEPENING.3b.2b.2a: sequential proof signatures + duplicate
    // pairs, derived from the SAME non-mutating grouping the dedup pass uses, so
    // the counted pairs are exactly the ones `dedup_sequential_modules` would
    // collapse. Pairwise (no per-module canonical proof), but pre-filtered so
    // the O(n^2) cross-module proof only runs inside a same-shape bucket — on a
    // default design (no equivalent stateful-leaf pair) it does no proof work.
    let sequential_classes = crate::ir::dedup::group_sequentially_equivalent_modules(design);
    let mut sequential_module_proof_signatures: Vec<Option<u64>> = vec![None; design.modules.len()];
    let mut num_sequentially_duplicate_module_pairs = 0usize;
    for class in &sequential_classes {
        let rep_name = class
            .iter()
            .map(|&idx| design.modules[idx].name.as_str())
            .min()
            .expect("sequential proof class is non-empty");
        let class_id = fnv1a_64_extend(fnv1a_64_init(), rep_name.as_bytes());
        for &idx in class {
            sequential_module_proof_signatures[idx] = Some(class_id);
        }
        num_sequentially_duplicate_module_pairs += class.len() * (class.len() - 1) / 2;
    }
    // Phase 5 (PHASE-5-PARAMETERIZATION.2.4) coverage inputs.
    let num_width_parameterized_modules = design
        .modules
        .iter()
        .filter(|m| m.param_env.is_some())
        .count();
    let num_param_override_instances = design
        .modules
        .iter()
        .flat_map(|m| m.instances.iter())
        .filter(|i| !i.param_bindings.is_empty())
        .count();
    // Phase 5b (PHASE-5B-AGGREGATES.2.3) coverage input.
    let num_packed_aggregate_modules = design
        .modules
        .iter()
        .filter(|m| m.aggregate_layout.is_some())
        .count();
    // AGGREGATE-ARRAY-PACKING: how many of those use the packed-array kind.
    let num_array_packed_aggregate_modules = design
        .modules
        .iter()
        .filter(|m| {
            m.aggregate_layout
                .as_ref()
                .is_some_and(|l| l.kind == crate::ir::AggregateKind::ArrayPacked)
        })
        .count();
    // Phase 6 (PHASE-6-ADVANCED-MOTIFS.2.3) coverage input.
    let num_memory_modules = design
        .modules
        .iter()
        .filter(|m| !m.memories.is_empty())
        .count();
    // Phase 6 (PHASE-6-ADVANCED-MOTIFS.3.4a) coverage input.
    let num_fsm_modules = design.modules.iter().filter(|m| !m.fsms.is_empty()).count();
    // CAPABILITY-BREADTH-EXPANSION.2b (decision 0024): modules carrying a Mealy FSM.
    let num_mealy_fsm_modules = design
        .modules
        .iter()
        .filter(|m| m.fsms.iter().any(|f| f.is_mealy()))
        .count();

    let mut out = DesignMetrics {
        design: design.top.clone(),
        canonical_module_signatures,
        semantic_module_signatures,
        num_distinct_module_signatures,
        num_structurally_duplicate_module_pairs,
        num_semantically_duplicate_module_pairs,
        sequential_module_proof_signatures,
        num_sequentially_duplicate_module_pairs,
        num_width_parameterized_modules,
        num_param_override_instances,
        num_packed_aggregate_modules,
        num_array_packed_aggregate_modules,
        num_memory_modules,
        num_fsm_modules,
        num_mealy_fsm_modules,
        num_modules: design.modules.len(),
        num_library_modules: design.modules.len().saturating_sub(1),
        num_internal_modules,
        num_leaf_modules,
        top_inputs: top.emitted_input_ports_in(Some(&modules_by_name)).count(),
        top_outputs: top.outputs.len(),
        ..Default::default()
    };

    let mut unique_instantiated = BTreeSet::new();
    let mut unique_instantiated_leafs = BTreeSet::new();
    let mut unique_profiled_instantiated = BTreeSet::new();
    let mut defs_by_depth_sets: BTreeMap<usize, BTreeSet<String>> = BTreeMap::new();
    let mut internal_module_occurrences_by_depth: BTreeMap<usize, usize> = BTreeMap::new();
    let mut leaf_depth_total = 0usize;
    let mut hierarchy_output_support_total = 0usize;

    out.num_profiled_module_definitions = design
        .modules
        .iter()
        .filter(|module| module.planned_interface_profile.is_some())
        .count();

    for port in top.emitted_input_ports_in(Some(&modules_by_name)) {
        if top.clock == Some(port.id) {
            out.top_clock_inputs += 1;
        } else if top.reset == Some(port.id) {
            out.top_reset_inputs += 1;
        } else {
            out.top_data_inputs += 1;
        }
    }

    let top_facts = module_composition_facts(top);
    out.top_direct_instance_output_drives = top_facts.direct_drives;
    out.top_parent_composed_outputs = top_facts.parent_composed_outputs;
    out.top_parent_port_composed_outputs = top_facts.parent_port_composed_outputs;
    out.top_parent_port_composed_outputs_through_parent_flops =
        top_facts.parent_port_composed_outputs_through_parent_flops;
    out.top_outputs_reaching_parent_cone_instances =
        top_facts.outputs_reaching_parent_cone_instances;
    out.top_outputs_reaching_parent_cone_instance_mixed_support =
        top_facts.outputs_reaching_parent_cone_instance_mixed_support;
    out.top_outputs_reaching_parent_cone_instances_through_parent_flops =
        top_facts.outputs_reaching_parent_cone_instances_through_parent_flops;
    out.top_outputs_reaching_parent_cone_instances_through_parent_flops_with_mixed_support =
        top_facts.outputs_reaching_parent_cone_instances_through_parent_flops_with_mixed_support;
    out.top_outputs_reaching_instance_outputs = top_facts.outputs_reaching_instance_outputs;
    out.top_outputs_without_instance_outputs = top_facts.outputs_without_instance_outputs;
    out.top_local_flops = top.flops.len();
    out.max_instance_output_support_per_top_output = top_facts.max_support;
    out.avg_instance_output_support_per_top_output =
        ratio(top_facts.total_support, top.outputs.len());
    out.top_instance_output_dependency_fraction =
        ratio(out.top_outputs_reaching_instance_outputs, out.top_outputs);
    out.top_parent_composed_output_fraction =
        ratio(out.top_parent_composed_outputs, out.top_outputs);
    out.top_parent_port_composed_output_fraction =
        ratio(out.top_parent_port_composed_outputs, out.top_outputs);
    out.top_parent_port_composed_parent_flop_output_fraction = ratio(
        out.top_parent_port_composed_outputs_through_parent_flops,
        out.top_outputs,
    );
    out.top_parent_cone_instance_output_fraction = ratio(
        out.top_outputs_reaching_parent_cone_instances,
        out.top_outputs,
    );
    out.top_parent_cone_instance_mixed_support_output_fraction = ratio(
        out.top_outputs_reaching_parent_cone_instance_mixed_support,
        out.top_outputs,
    );
    out.top_parent_cone_instance_flop_output_fraction = ratio(
        out.top_outputs_reaching_parent_cone_instances_through_parent_flops,
        out.top_outputs,
    );
    out.top_parent_cone_instance_flop_mixed_support_output_fraction = ratio(
        out.top_outputs_reaching_parent_cone_instances_through_parent_flops_with_mixed_support,
        out.top_outputs,
    );

    for module in library
        .iter()
        .copied()
        .filter(|module| module.instances.is_empty())
    {
        if module.has_local_flops() {
            out.num_sequential_leaf_modules += 1;
        } else {
            out.num_combinational_leaf_modules += 1;
        }
    }

    let mut walk = DesignWalkState {
        modules_by_name: &modules_by_name,
        out: &mut out,
        unique_instantiated: &mut unique_instantiated,
        unique_instantiated_leafs: &mut unique_instantiated_leafs,
        unique_profiled_instantiated: &mut unique_profiled_instantiated,
        defs_by_depth_sets: &mut defs_by_depth_sets,
        internal_module_occurrences_by_depth: &mut internal_module_occurrences_by_depth,
        leaf_depth_total: &mut leaf_depth_total,
        hierarchy_output_support_total: &mut hierarchy_output_support_total,
    };
    walk_module_occurrence(top, 0, &mut walk);

    out.num_unique_instantiated_modules = unique_instantiated.len();
    out.num_unused_module_definitions = out
        .num_library_modules
        .saturating_sub(out.num_unique_instantiated_modules);
    out.num_unused_leaf_modules = out
        .num_leaf_modules
        .saturating_sub(unique_instantiated_leafs.len());
    out.num_reused_instance_slots = out
        .num_instances
        .saturating_sub(out.num_unique_instantiated_modules);
    out.num_profiled_instantiated_modules = unique_profiled_instantiated.len();

    out.library_coverage_fraction =
        ratio(out.num_unique_instantiated_modules, out.num_library_modules);
    out.unused_library_fraction = ratio(out.num_unused_module_definitions, out.num_library_modules);
    out.instance_reuse_fraction = ratio(out.num_reused_instance_slots, out.num_instances);
    out.instance_to_library_ratio = ratio(out.num_instances, out.num_library_modules);
    out.avg_instances_per_unique_instantiated_module =
        ratio(out.num_instances, out.num_unique_instantiated_modules);
    out.num_single_use_instantiated_modules = out
        .instantiated_module_histogram
        .values()
        .filter(|&&count| count == 1)
        .count();
    out.num_multiuse_instantiated_modules = out
        .instantiated_module_histogram
        .values()
        .filter(|&&count| count > 1)
        .count();
    out.single_use_instantiated_module_fraction = ratio(
        out.num_single_use_instantiated_modules,
        out.num_unique_instantiated_modules,
    );
    out.profiled_instantiated_module_fraction = ratio(
        out.num_profiled_instantiated_modules,
        out.num_unique_instantiated_modules,
    );
    out.profiled_instance_fraction = ratio(out.num_profiled_instance_slots, out.num_instances);
    out.avg_child_data_inputs_per_instance =
        ratio(out.total_child_data_input_bindings, out.num_instances);
    out.avg_child_outputs_per_instance = ratio(out.total_child_output_exposures, out.num_instances);
    out.dep_bearing_child_input_binding_fraction = ratio(
        out.dep_bearing_child_input_bindings,
        out.total_child_data_input_bindings,
    );
    out.instance_output_child_input_binding_fraction = ratio(
        out.child_input_bindings_from_instance_outputs
            + out.child_input_bindings_from_mixed_support,
        out.total_child_data_input_bindings,
    );
    out.parent_port_child_input_binding_fraction = ratio(
        out.child_input_bindings_from_parent_ports + out.child_input_bindings_from_mixed_support,
        out.total_child_data_input_bindings,
    );
    out.parent_composed_child_input_binding_fraction = ratio(
        out.child_input_bindings_from_parent_composed_logic,
        out.total_child_data_input_bindings,
    );
    out.stateful_parent_composed_mixed_support_child_input_binding_fraction = ratio(
        out.child_input_bindings_from_stateful_parent_composed_mixed_support,
        out.total_child_data_input_bindings,
    );
    out.parent_flop_child_input_binding_fraction = ratio(
        out.child_input_bindings_from_parent_flops,
        out.total_child_data_input_bindings,
    );
    out.registered_instance_output_child_input_binding_fraction = ratio(
        out.child_input_bindings_from_registered_instance_outputs,
        out.total_child_data_input_bindings,
    );
    out.registered_parent_composed_child_input_binding_fraction = ratio(
        out.child_input_bindings_from_registered_parent_composed_logic,
        out.total_child_data_input_bindings,
    );
    out.registered_mixed_support_child_input_binding_fraction = ratio(
        out.child_input_bindings_from_registered_mixed_support,
        out.total_child_data_input_bindings,
    );
    out.registered_sibling_mixed_support_child_input_binding_fraction = ratio(
        out.child_input_bindings_from_registered_sibling_mixed_support,
        out.total_child_data_input_bindings,
    );
    out.registered_three_stage_parent_composed_child_input_binding_fraction = ratio(
        out.child_input_bindings_from_registered_three_stage_parent_composed_logic,
        out.total_child_data_input_bindings,
    );
    out.registered_multistage_parent_composed_child_input_binding_fraction = ratio(
        out.child_input_bindings_from_registered_multistage_parent_composed_logic,
        out.total_child_data_input_bindings,
    );
    out.registered_multistage_mixed_support_child_input_binding_fraction = ratio(
        out.child_input_bindings_from_registered_multistage_mixed_support,
        out.total_child_data_input_bindings,
    );
    out.registered_multistage_instance_output_child_input_binding_fraction = ratio(
        out.child_input_bindings_from_registered_multistage_instance_outputs,
        out.total_child_data_input_bindings,
    );
    out.registered_multistage_parent_cone_instance_child_input_binding_fraction = ratio(
        out.child_input_bindings_from_registered_multistage_parent_cone_instances,
        out.total_child_data_input_bindings,
    );
    out.registered_multistage_parent_composed_parent_cone_instance_child_input_binding_fraction =
        ratio(
            out.child_input_bindings_from_registered_multistage_parent_composed_parent_cone_instances,
            out.total_child_data_input_bindings,
        );
    out.registered_parent_cone_instance_child_input_binding_fraction = ratio(
        out.child_input_bindings_from_registered_parent_cone_instances,
        out.total_child_data_input_bindings,
    );
    out.registered_parent_cone_instance_mixed_support_child_input_binding_fraction = ratio(
        out.child_input_bindings_from_registered_parent_cone_instance_mixed_support,
        out.total_child_data_input_bindings,
    );
    out.parent_cone_instance_child_input_binding_fraction = ratio(
        out.child_input_bindings_from_parent_cone_instances,
        out.total_child_data_input_bindings,
    );
    out.parent_cone_instance_mixed_support_child_input_binding_fraction = ratio(
        out.child_input_bindings_from_parent_cone_instance_mixed_support,
        out.total_child_data_input_bindings,
    );
    out.parent_cone_instance_flop_child_input_binding_fraction = ratio(
        out.child_input_bindings_from_parent_cone_instances_through_parent_flops,
        out.total_child_data_input_bindings,
    );
    out.parent_cone_instance_flop_mixed_support_child_input_binding_fraction = ratio(
        out.child_input_bindings_from_parent_cone_instance_flop_mixed_support,
        out.total_child_data_input_bindings,
    );
    out.sequential_instance_fraction = ratio(out.num_sequential_instances, out.num_instances);
    out.avg_nodes_per_instance = ratio(out.total_instantiated_child_nodes, out.num_instances);
    out.avg_flops_per_instance = ratio(out.total_instantiated_child_flops, out.num_instances);
    out.avg_leaf_depth = ratio(leaf_depth_total, out.num_leaf_module_occurrences);
    out.avg_child_instances_per_internal_module =
        ratio(out.num_instances, out.num_internal_module_occurrences);
    out.avg_instance_output_support_per_hierarchy_output = ratio(
        hierarchy_output_support_total,
        out.hierarchy_direct_instance_output_drives + out.hierarchy_parent_composed_outputs,
    );
    out.hierarchy_parent_port_composed_output_fraction = ratio(
        out.hierarchy_parent_port_composed_outputs,
        out.hierarchy_direct_instance_output_drives + out.hierarchy_parent_composed_outputs,
    );
    out.hierarchy_parent_port_composed_parent_flop_output_fraction = ratio(
        out.hierarchy_parent_port_composed_outputs_through_parent_flops,
        out.hierarchy_direct_instance_output_drives + out.hierarchy_parent_composed_outputs,
    );
    out.hierarchy_parent_cone_instance_output_fraction = ratio(
        out.hierarchy_outputs_reaching_parent_cone_instances,
        out.hierarchy_direct_instance_output_drives + out.hierarchy_parent_composed_outputs,
    );
    out.hierarchy_parent_cone_instance_mixed_support_output_fraction = ratio(
        out.hierarchy_outputs_reaching_parent_cone_instance_mixed_support,
        out.hierarchy_direct_instance_output_drives + out.hierarchy_parent_composed_outputs,
    );
    out.hierarchy_parent_cone_instance_flop_output_fraction = ratio(
        out.hierarchy_outputs_reaching_parent_cone_instances_through_parent_flops,
        out.hierarchy_direct_instance_output_drives + out.hierarchy_parent_composed_outputs,
    );
    out.hierarchy_parent_cone_instance_flop_mixed_support_output_fraction = ratio(
        out.hierarchy_outputs_reaching_parent_cone_instances_through_parent_flops_with_mixed_support,
        out.hierarchy_direct_instance_output_drives + out.hierarchy_parent_composed_outputs,
    );
    out.top_instance_output_child_input_binding_fraction = ratio(
        out.top_child_input_bindings_from_instance_outputs
            + out.top_child_input_bindings_from_mixed_support,
        out.top_total_child_data_input_bindings,
    );
    out.top_parent_composed_child_input_binding_fraction = ratio(
        out.top_child_input_bindings_from_parent_composed_logic,
        out.top_total_child_data_input_bindings,
    );
    out.top_stateful_parent_composed_mixed_support_child_input_binding_fraction = ratio(
        out.top_child_input_bindings_from_stateful_parent_composed_mixed_support,
        out.top_total_child_data_input_bindings,
    );
    out.top_registered_instance_output_child_input_binding_fraction = ratio(
        out.top_child_input_bindings_from_registered_instance_outputs,
        out.top_total_child_data_input_bindings,
    );
    out.top_registered_parent_composed_child_input_binding_fraction = ratio(
        out.top_child_input_bindings_from_registered_parent_composed_logic,
        out.top_total_child_data_input_bindings,
    );
    out.top_registered_mixed_support_child_input_binding_fraction = ratio(
        out.top_child_input_bindings_from_registered_mixed_support,
        out.top_total_child_data_input_bindings,
    );
    out.top_registered_sibling_mixed_support_child_input_binding_fraction = ratio(
        out.top_child_input_bindings_from_registered_sibling_mixed_support,
        out.top_total_child_data_input_bindings,
    );
    out.top_registered_multistage_parent_composed_child_input_binding_fraction = ratio(
        out.top_child_input_bindings_from_registered_multistage_parent_composed_logic,
        out.top_total_child_data_input_bindings,
    );
    out.top_registered_three_stage_parent_composed_child_input_binding_fraction = ratio(
        out.top_child_input_bindings_from_registered_three_stage_parent_composed_logic,
        out.top_total_child_data_input_bindings,
    );
    out.top_registered_multistage_mixed_support_child_input_binding_fraction = ratio(
        out.top_child_input_bindings_from_registered_multistage_mixed_support,
        out.top_total_child_data_input_bindings,
    );
    out.top_registered_multistage_instance_output_child_input_binding_fraction = ratio(
        out.top_child_input_bindings_from_registered_multistage_instance_outputs,
        out.top_total_child_data_input_bindings,
    );
    out.top_registered_multistage_parent_cone_instance_child_input_binding_fraction = ratio(
        out.top_child_input_bindings_from_registered_multistage_parent_cone_instances,
        out.top_total_child_data_input_bindings,
    );
    out.top_registered_multistage_parent_composed_parent_cone_instance_child_input_binding_fraction =
        ratio(
            out.top_child_input_bindings_from_registered_multistage_parent_composed_parent_cone_instances,
            out.top_total_child_data_input_bindings,
        );
    out.top_registered_parent_cone_instance_child_input_binding_fraction = ratio(
        out.top_child_input_bindings_from_registered_parent_cone_instances,
        out.top_total_child_data_input_bindings,
    );
    out.top_registered_parent_cone_instance_mixed_support_child_input_binding_fraction = ratio(
        out.top_child_input_bindings_from_registered_parent_cone_instance_mixed_support,
        out.top_total_child_data_input_bindings,
    );
    out.top_parent_cone_instance_mixed_support_child_input_binding_fraction = ratio(
        out.top_child_input_bindings_from_parent_cone_instance_mixed_support,
        out.top_total_child_data_input_bindings,
    );
    out.top_parent_cone_instance_flop_child_input_binding_fraction = ratio(
        out.top_child_input_bindings_from_parent_cone_instances_through_parent_flops,
        out.top_total_child_data_input_bindings,
    );
    out.top_parent_cone_instance_flop_mixed_support_child_input_binding_fraction = ratio(
        out.top_child_input_bindings_from_parent_cone_instance_flop_mixed_support,
        out.top_total_child_data_input_bindings,
    );
    out.top_parent_flop_child_input_binding_fraction = ratio(
        out.top_child_input_bindings_from_parent_flops,
        out.top_total_child_data_input_bindings,
    );
    out.top_parent_cone_instance_child_input_binding_fraction = ratio(
        out.top_child_input_bindings_from_parent_cone_instances,
        out.top_total_child_data_input_bindings,
    );
    for (depth, count) in internal_module_occurrences_by_depth {
        out.avg_child_instances_by_parent_depth.insert(
            depth,
            ratio(
                *out.instance_slots_by_parent_depth.get(&depth).unwrap_or(&0),
                count,
            ),
        );
    }
    for (depth, names) in defs_by_depth_sets {
        out.module_defs_by_depth.insert(depth, names.len());
    }

    out
}

struct DesignWalkState<'a> {
    modules_by_name: &'a BTreeMap<&'a str, &'a Module>,
    out: &'a mut DesignMetrics,
    unique_instantiated: &'a mut BTreeSet<String>,
    unique_instantiated_leafs: &'a mut BTreeSet<String>,
    unique_profiled_instantiated: &'a mut BTreeSet<String>,
    defs_by_depth_sets: &'a mut BTreeMap<usize, BTreeSet<String>>,
    internal_module_occurrences_by_depth: &'a mut BTreeMap<usize, usize>,
    leaf_depth_total: &'a mut usize,
    hierarchy_output_support_total: &'a mut usize,
}

fn walk_module_occurrence(module: &Module, depth: usize, state: &mut DesignWalkState<'_>) {
    state
        .defs_by_depth_sets
        .entry(depth)
        .or_default()
        .insert(module.name.clone());
    *state
        .out
        .module_occurrences_by_depth
        .entry(depth)
        .or_insert(0) += 1;
    state.out.max_module_depth = state.out.max_module_depth.max(depth);

    if module.instances.is_empty() {
        state.out.num_leaf_module_occurrences += 1;
        *state
            .out
            .leaf_module_occurrences_by_depth
            .entry(depth)
            .or_insert(0) += 1;
        if state.out.num_leaf_module_occurrences == 1 {
            state.out.realized_min_leaf_depth = depth;
        } else {
            state.out.realized_min_leaf_depth = state.out.realized_min_leaf_depth.min(depth);
        }
        state.out.realized_max_leaf_depth = state.out.realized_max_leaf_depth.max(depth);
        *state.leaf_depth_total += depth;
        return;
    }

    state.out.num_internal_module_occurrences += 1;
    if module.has_local_flops() {
        state.out.hierarchy_parent_local_flops += module.flops.len();
        state.out.internal_module_occurrences_with_local_flops += 1;
    }
    *state
        .internal_module_occurrences_by_depth
        .entry(depth)
        .or_insert(0) += 1;
    let child_count = module.instances.len();
    *state
        .out
        .instance_slots_by_parent_depth
        .entry(depth)
        .or_insert(0) += child_count;
    state
        .out
        .min_child_instances_by_parent_depth
        .entry(depth)
        .and_modify(|min| *min = (*min).min(child_count))
        .or_insert(child_count);
    state
        .out
        .max_child_instances_by_parent_depth
        .entry(depth)
        .and_modify(|max| *max = (*max).max(child_count))
        .or_insert(child_count);
    *state
        .out
        .child_instances_per_internal_module_histogram
        .entry(child_count)
        .or_insert(0) += 1;
    if state.out.num_internal_module_occurrences == 1 {
        state.out.min_child_instances_per_internal_module = child_count;
    } else {
        state.out.min_child_instances_per_internal_module = state
            .out
            .min_child_instances_per_internal_module
            .min(child_count);
    }
    state.out.max_child_instances_per_internal_module = state
        .out
        .max_child_instances_per_internal_module
        .max(child_count);

    let facts = module_composition_facts(module);
    state.out.hierarchy_direct_instance_output_drives += facts.direct_drives;
    state.out.hierarchy_parent_composed_outputs += facts.parent_composed_outputs;
    state.out.hierarchy_parent_port_composed_outputs += facts.parent_port_composed_outputs;
    state
        .out
        .hierarchy_parent_port_composed_outputs_through_parent_flops +=
        facts.parent_port_composed_outputs_through_parent_flops;
    state.out.hierarchy_outputs_reaching_parent_cone_instances +=
        facts.outputs_reaching_parent_cone_instances;
    state
        .out
        .hierarchy_outputs_reaching_parent_cone_instance_mixed_support +=
        facts.outputs_reaching_parent_cone_instance_mixed_support;
    state
        .out
        .hierarchy_outputs_reaching_parent_cone_instances_through_parent_flops +=
        facts.outputs_reaching_parent_cone_instances_through_parent_flops;
    state
        .out
        .hierarchy_outputs_reaching_parent_cone_instances_through_parent_flops_with_mixed_support +=
        facts
            .outputs_reaching_parent_cone_instances_through_parent_flops_with_mixed_support;
    *state.hierarchy_output_support_total += facts.total_support;
    state.out.max_instance_output_support_per_hierarchy_output = state
        .out
        .max_instance_output_support_per_hierarchy_output
        .max(facts.max_support);
    if facts.parent_composed_outputs > 0 {
        state.out.module_occurrences_with_parent_composed_outputs += 1;
    }

    let parent_cone_instances_in_module = module
        .instances
        .iter()
        .filter(|instance| instance.role == InstanceRole::ParentCone)
        .count();
    state.out.max_parent_cone_instances_per_internal_module = state
        .out
        .max_parent_cone_instances_per_internal_module
        .max(parent_cone_instances_in_module);

    for instance in &module.instances {
        state.out.num_instances += 1;
        if instance.role == InstanceRole::ParentCone {
            state.out.hierarchy_parent_cone_instances += 1;
            if module.name == state.out.design {
                state.out.top_parent_cone_instances += 1;
            }
        }
        *state
            .out
            .instantiated_module_histogram
            .entry(instance.module.clone())
            .or_insert(0) += 1;
        state.unique_instantiated.insert(instance.module.clone());

        let child = state
            .modules_by_name
            .get(instance.module.as_str())
            .expect("instance child must exist in validated design");
        if child.instances.is_empty() {
            state.unique_instantiated_leafs.insert(child.name.clone());
        }
        if child.planned_interface_profile.is_some() {
            state
                .unique_profiled_instantiated
                .insert(child.name.clone());
            state.out.num_profiled_instance_slots += 1;
        }
        let child_data_inputs: BTreeSet<_> = child
            .emitted_data_input_ports_in(Some(state.modules_by_name))
            .map(|port| port.id)
            .collect();
        state.out.total_child_data_input_bindings += child_data_inputs.len();
        if module.name == state.out.design {
            state.out.top_total_child_data_input_bindings += child_data_inputs.len();
        }
        for (port_id, node_id) in &instance.inputs {
            if !child_data_inputs.contains(port_id) {
                continue;
            }
            let deps = node_deps(module, *node_id);
            if !deps.is_empty() {
                state.out.dep_bearing_child_input_bindings += 1;
            }
            let has_ports = deps.has_ports();
            let has_flops = deps.has_flop_virtuals();
            let has_instance_outputs = deps.has_instance_output_virtuals();
            if is_parent_composed_logic_node(module, *node_id) {
                state.out.child_input_bindings_from_parent_composed_logic += 1;
                if module.name == state.out.design {
                    state
                        .out
                        .top_child_input_bindings_from_parent_composed_logic += 1;
                }
                if has_ports && has_flops && has_instance_outputs {
                    state
                        .out
                        .child_input_bindings_from_stateful_parent_composed_mixed_support += 1;
                    if module.name == state.out.design {
                        state
                            .out
                            .top_child_input_bindings_from_stateful_parent_composed_mixed_support += 1;
                    }
                }
            }
            if has_flops {
                state.out.child_input_bindings_from_parent_flops += 1;
                if module.name == state.out.design {
                    state.out.top_child_input_bindings_from_parent_flops += 1;
                }
            }
            if binding_uses_registered_instance_output(module, *node_id) {
                state
                    .out
                    .child_input_bindings_from_registered_instance_outputs += 1;
                if module.name == state.out.design {
                    state
                        .out
                        .top_child_input_bindings_from_registered_instance_outputs += 1;
                }
            }
            if binding_uses_registered_parent_composed_logic(module, *node_id) {
                state
                    .out
                    .child_input_bindings_from_registered_parent_composed_logic += 1;
                if module.name == state.out.design {
                    state
                        .out
                        .top_child_input_bindings_from_registered_parent_composed_logic += 1;
                }
            }
            if binding_uses_registered_mixed_support(module, *node_id) {
                state.out.child_input_bindings_from_registered_mixed_support += 1;
                if module.name == state.out.design {
                    state
                        .out
                        .top_child_input_bindings_from_registered_mixed_support += 1;
                }
            }
            if binding_uses_registered_sibling_mixed_support(module, *node_id) {
                state
                    .out
                    .child_input_bindings_from_registered_sibling_mixed_support += 1;
                if module.name == state.out.design {
                    state
                        .out
                        .top_child_input_bindings_from_registered_sibling_mixed_support += 1;
                }
            }
            if binding_uses_registered_multistage_parent_composed_logic(module, *node_id) {
                state
                    .out
                    .child_input_bindings_from_registered_multistage_parent_composed_logic += 1;
                if module.name == state.out.design {
                    state
                        .out
                        .top_child_input_bindings_from_registered_multistage_parent_composed_logic += 1;
                }
            }
            if binding_uses_registered_three_stage_parent_composed_logic(module, *node_id) {
                state
                    .out
                    .child_input_bindings_from_registered_three_stage_parent_composed_logic += 1;
                if module.name == state.out.design {
                    state
                        .out
                        .top_child_input_bindings_from_registered_three_stage_parent_composed_logic += 1;
                }
            }
            if binding_uses_registered_multistage_mixed_support(module, *node_id) {
                state
                    .out
                    .child_input_bindings_from_registered_multistage_mixed_support += 1;
                if module.name == state.out.design {
                    state
                        .out
                        .top_child_input_bindings_from_registered_multistage_mixed_support += 1;
                }
            }
            if binding_uses_registered_multistage_instance_output(module, *node_id) {
                state
                    .out
                    .child_input_bindings_from_registered_multistage_instance_outputs += 1;
                if module.name == state.out.design {
                    state
                        .out
                        .top_child_input_bindings_from_registered_multistage_instance_outputs += 1;
                }
            }
            if binding_uses_registered_multistage_parent_cone_instance_output(module, *node_id) {
                state
                    .out
                    .child_input_bindings_from_registered_multistage_parent_cone_instances += 1;
                if module.name == state.out.design {
                    state.out.top_child_input_bindings_from_registered_multistage_parent_cone_instances += 1;
                }
            }
            if binding_uses_registered_multistage_parent_composed_parent_cone_instance_output(
                module, *node_id,
            ) {
                state.out.child_input_bindings_from_registered_multistage_parent_composed_parent_cone_instances += 1;
                if module.name == state.out.design {
                    state.out.top_child_input_bindings_from_registered_multistage_parent_composed_parent_cone_instances += 1;
                }
            }
            if binding_uses_parent_cone_instance_output(module, *node_id) {
                state.out.child_input_bindings_from_parent_cone_instances += 1;
                if module.name == state.out.design {
                    state
                        .out
                        .top_child_input_bindings_from_parent_cone_instances += 1;
                }
            }
            if binding_uses_parent_cone_instance_mixed_support(module, *node_id) {
                state
                    .out
                    .child_input_bindings_from_parent_cone_instance_mixed_support += 1;
                if module.name == state.out.design {
                    state
                        .out
                        .top_child_input_bindings_from_parent_cone_instance_mixed_support += 1;
                }
            }
            if binding_uses_parent_cone_instance_output_through_parent_flop(module, *node_id) {
                state
                    .out
                    .child_input_bindings_from_parent_cone_instances_through_parent_flops += 1;
                if module.name == state.out.design {
                    state
                        .out
                        .top_child_input_bindings_from_parent_cone_instances_through_parent_flops +=
                        1;
                }
            }
            if binding_uses_parent_cone_instance_flop_mixed_support(module, *node_id) {
                state
                    .out
                    .child_input_bindings_from_parent_cone_instance_flop_mixed_support += 1;
                if module.name == state.out.design {
                    state
                        .out
                        .top_child_input_bindings_from_parent_cone_instance_flop_mixed_support += 1;
                }
            }
            if binding_uses_registered_parent_cone_instance_output(module, *node_id) {
                state
                    .out
                    .child_input_bindings_from_registered_parent_cone_instances += 1;
                if module.name == state.out.design {
                    state
                        .out
                        .top_child_input_bindings_from_registered_parent_cone_instances += 1;
                }
            }
            if binding_uses_registered_parent_cone_instance_mixed_support(module, *node_id) {
                state
                    .out
                    .child_input_bindings_from_registered_parent_cone_instance_mixed_support += 1;
                if module.name == state.out.design {
                    state
                        .out
                        .top_child_input_bindings_from_registered_parent_cone_instance_mixed_support += 1;
                }
            }
            match (has_ports, has_instance_outputs, deps.is_empty()) {
                (_, _, true) => {
                    state.out.child_input_bindings_from_constants += 1;
                    if module.name == state.out.design {
                        state.out.top_child_input_bindings_from_constants += 1;
                    }
                }
                (true, true, false) => {
                    state.out.child_input_bindings_from_mixed_support += 1;
                    if module.name == state.out.design {
                        state.out.top_child_input_bindings_from_mixed_support += 1;
                    }
                }
                (true, false, false) => {
                    state.out.child_input_bindings_from_parent_ports += 1;
                    if module.name == state.out.design {
                        state.out.top_child_input_bindings_from_parent_ports += 1;
                    }
                }
                (false, true, false) => {
                    state.out.child_input_bindings_from_instance_outputs += 1;
                    if module.name == state.out.design {
                        state.out.top_child_input_bindings_from_instance_outputs += 1;
                    }
                }
                (false, false, false) => {}
            }
        }
        state.out.total_child_output_exposures += child.outputs.len();
        state.out.total_instantiated_child_nodes += child.nodes.len();
        state.out.total_instantiated_child_flops += child.flops.len();
        state.out.max_nodes_per_instance = state.out.max_nodes_per_instance.max(child.nodes.len());
        state.out.max_flops_per_instance = state.out.max_flops_per_instance.max(child.flops.len());
        if child.carries_sequential_state_in(Some(state.modules_by_name)) {
            state.out.num_sequential_instances += 1;
        } else {
            state.out.num_combinational_instances += 1;
        }

        if module.name == state.out.design {
            if child
                .emitted_input_ports_in(Some(state.modules_by_name))
                .any(|port| child.clock == Some(port.id))
            {
                state.out.clock_fanout_instances += 1;
            }
            if child
                .emitted_input_ports_in(Some(state.modules_by_name))
                .any(|port| child.reset == Some(port.id))
            {
                state.out.reset_fanout_instances += 1;
            }
        }

        walk_module_occurrence(child, depth + 1, state);
    }
}

#[derive(Default)]
struct ModuleCompositionFacts {
    direct_drives: usize,
    parent_composed_outputs: usize,
    parent_port_composed_outputs: usize,
    parent_port_composed_outputs_through_parent_flops: usize,
    outputs_reaching_parent_cone_instances: usize,
    outputs_reaching_parent_cone_instance_mixed_support: usize,
    outputs_reaching_parent_cone_instances_through_parent_flops: usize,
    outputs_reaching_parent_cone_instances_through_parent_flops_with_mixed_support: usize,
    outputs_reaching_instance_outputs: usize,
    outputs_without_instance_outputs: usize,
    total_support: usize,
    max_support: usize,
}

fn module_composition_facts(module: &Module) -> ModuleCompositionFacts {
    let mut memo = HashMap::new();
    let mut out = ModuleCompositionFacts::default();
    let has_parent_cone_instances = module
        .instances
        .iter()
        .any(|inst| inst.role == InstanceRole::ParentCone);
    for (_, root) in &module.drives {
        let support = collect_instance_output_support(module, *root, &mut memo);
        let support_len = support.len();
        let deps = node_deps(module, *root);
        out.total_support += support_len;
        out.max_support = out.max_support.max(support_len);
        if support_len > 0 {
            out.outputs_reaching_instance_outputs += 1;
            if support.iter().any(|(instance, _)| {
                module
                    .instances
                    .get(*instance as usize)
                    .is_some_and(|inst| inst.role == InstanceRole::ParentCone)
            }) {
                out.outputs_reaching_parent_cone_instances += 1;
                if deps.has_ports() {
                    out.outputs_reaching_parent_cone_instance_mixed_support += 1;
                }
            }
            if deps.has_ports() {
                out.parent_port_composed_outputs += 1;
                if deps.has_flop_virtuals() {
                    out.parent_port_composed_outputs_through_parent_flops += 1;
                }
            }
        } else {
            out.outputs_without_instance_outputs += 1;
        }
        if matches!(module.nodes[*root as usize], Node::InstanceOutput { .. }) {
            out.direct_drives += 1;
        } else if support_len > 0 {
            out.parent_composed_outputs += 1;
        }
        let reaches_parent_cone_instance_through_parent_flop = has_parent_cone_instances
            && output_reaches_parent_cone_instance_through_parent_flop(module, *root, &mut memo);
        if reaches_parent_cone_instance_through_parent_flop {
            out.outputs_reaching_parent_cone_instances_through_parent_flops += 1;
            if deps.has_ports() {
                out.outputs_reaching_parent_cone_instances_through_parent_flops_with_mixed_support +=
                    1;
            }
        }
    }
    out
}

fn output_reaches_parent_cone_instance_through_parent_flop(
    module: &Module,
    root: NodeId,
    support_memo: &mut HashMap<NodeId, BTreeSet<(InstanceId, PortId)>>,
) -> bool {
    let deps = node_deps(module, root);
    let mut flop_memo = HashMap::new();
    let mut visiting_flops = BTreeSet::new();
    let reaches = deps.flop_virtuals().any(|flop_id| {
        flop_d_reaches_parent_cone_instance_output(
            module,
            flop_id,
            support_memo,
            &mut flop_memo,
            &mut visiting_flops,
        )
    });
    reaches
}

fn flop_d_reaches_parent_cone_instance_output(
    module: &Module,
    flop_id: FlopId,
    support_memo: &mut HashMap<NodeId, BTreeSet<(InstanceId, PortId)>>,
    flop_memo: &mut HashMap<FlopId, bool>,
    visiting_flops: &mut BTreeSet<FlopId>,
) -> bool {
    if let Some(reaches) = flop_memo.get(&flop_id) {
        return *reaches;
    }
    if !visiting_flops.insert(flop_id) {
        return false;
    }

    let reaches = module
        .flops
        .get(flop_id as usize)
        .and_then(|flop| flop.d)
        .is_some_and(|d| {
            node_or_registered_source_reaches_parent_cone_instance_output(
                module,
                d,
                support_memo,
                flop_memo,
                visiting_flops,
            )
        });

    visiting_flops.remove(&flop_id);
    flop_memo.insert(flop_id, reaches);
    reaches
}

fn node_or_registered_source_reaches_parent_cone_instance_output(
    module: &Module,
    node_id: NodeId,
    support_memo: &mut HashMap<NodeId, BTreeSet<(InstanceId, PortId)>>,
    flop_memo: &mut HashMap<FlopId, bool>,
    visiting_flops: &mut BTreeSet<FlopId>,
) -> bool {
    collect_instance_output_support(module, node_id, support_memo)
        .iter()
        .any(|(instance, _)| {
            module
                .instances
                .get(*instance as usize)
                .is_some_and(|inst| inst.role == InstanceRole::ParentCone)
        })
        || node_deps(module, node_id).flop_virtuals().any(|flop_id| {
            flop_d_reaches_parent_cone_instance_output(
                module,
                flop_id,
                support_memo,
                flop_memo,
                visiting_flops,
            )
        })
}

fn collect_instance_output_support(
    module: &Module,
    node_id: NodeId,
    memo: &mut HashMap<NodeId, BTreeSet<(InstanceId, PortId)>>,
) -> BTreeSet<(InstanceId, PortId)> {
    if let Some(existing) = memo.get(&node_id) {
        return existing.clone();
    }

    let support = match &module.nodes[node_id as usize] {
        Node::InstanceOutput { instance, port, .. } => BTreeSet::from([(*instance, *port)]),
        Node::Gate { operands, .. } => operands.iter().fold(BTreeSet::new(), |mut acc, operand| {
            acc.extend(collect_instance_output_support(module, *operand, memo));
            acc
        }),
        _ => BTreeSet::new(),
    };
    memo.insert(node_id, support.clone());
    support
}

fn node_deps(module: &Module, node_id: NodeId) -> crate::ir::DepSet {
    match &module.nodes[node_id as usize] {
        Node::PrimaryInput { port, .. } => crate::ir::DepSet::from_port(*port),
        Node::FlopQ { flop, .. } => crate::ir::DepSet::from_flop_virtual(*flop),
        Node::MemRead { mem, .. } => crate::ir::DepSet::from_mem_virtual(*mem),
        Node::FsmOut { fsm, .. } => crate::ir::DepSet::from_fsm_virtual(*fsm),
        Node::InstanceOutput { instance, port, .. } => {
            crate::ir::DepSet::from_instance_output_virtual(*instance, *port)
        }
        Node::Constant { .. } => crate::ir::DepSet::new(),
        Node::Gate { deps, .. } => deps.clone(),
    }
}

fn is_parent_composed_logic_node(module: &Module, node_id: NodeId) -> bool {
    matches!(module.nodes[node_id as usize], Node::Gate { .. })
}

fn binding_is_registered_child_input_route(module: &Module, node_id: NodeId) -> bool {
    matches!(module.nodes[node_id as usize], Node::FlopQ { .. })
}

fn binding_uses_registered_instance_output(module: &Module, node_id: NodeId) -> bool {
    if !binding_is_registered_child_input_route(module, node_id) {
        return false;
    }
    let deps = node_deps(module, node_id);
    let uses_registered_instance_output = deps.flop_virtuals().any(|flop_id| {
        module
            .flops
            .get(flop_id as usize)
            .and_then(|flop| flop.d)
            .is_some_and(|d| node_deps(module, d).has_instance_output_virtuals())
    });
    uses_registered_instance_output
}

fn binding_uses_registered_parent_composed_logic(module: &Module, node_id: NodeId) -> bool {
    if !binding_is_registered_child_input_route(module, node_id) {
        return false;
    }
    let deps = node_deps(module, node_id);
    let uses_registered_parent_composed_logic = deps.flop_virtuals().any(|flop_id| {
        module
            .flops
            .get(flop_id as usize)
            .and_then(|flop| flop.d)
            .is_some_and(|d| {
                is_registered_parent_composed_logic_node(module, d)
                    && node_deps(module, d).has_instance_output_virtuals()
            })
    });
    uses_registered_parent_composed_logic
}

fn binding_uses_registered_mixed_support(module: &Module, node_id: NodeId) -> bool {
    if !binding_is_registered_child_input_route(module, node_id) {
        return false;
    }
    let deps = node_deps(module, node_id);
    let uses_registered_mixed_support = deps.flop_virtuals().any(|flop_id| {
        module
            .flops
            .get(flop_id as usize)
            .and_then(|flop| flop.d)
            .is_some_and(|d| {
                let d_deps = node_deps(module, d);
                is_registered_parent_composed_logic_node(module, d)
                    && d_deps.has_ports()
                    && d_deps.has_instance_output_virtuals()
            })
    });
    uses_registered_mixed_support
}

fn binding_uses_registered_sibling_mixed_support(module: &Module, node_id: NodeId) -> bool {
    if !binding_is_registered_child_input_route(module, node_id) {
        return false;
    }
    let deps = node_deps(module, node_id);
    let uses_registered_sibling_mixed_support = deps.flop_virtuals().any(|flop_id| {
        module
            .flops
            .get(flop_id as usize)
            .and_then(|flop| flop.d)
            .is_some_and(|d| {
                let d_deps = node_deps(module, d);
                !is_registered_parent_composed_logic_node(module, d)
                    && d_deps.has_ports()
                    && d_deps.has_instance_output_virtuals()
            })
    });
    uses_registered_sibling_mixed_support
}

fn binding_uses_registered_multistage_parent_composed_logic(
    module: &Module,
    node_id: NodeId,
) -> bool {
    if !binding_is_registered_child_input_route(module, node_id) {
        return false;
    }
    let deps = node_deps(module, node_id);
    let uses_registered_multistage_parent_composed_logic = deps.flop_virtuals().any(|flop_id| {
        module
            .flops
            .get(flop_id as usize)
            .and_then(|flop| flop.d)
            .is_some_and(|d| {
                let d_deps = node_deps(module, d);
                is_registered_parent_composed_logic_node(module, d)
                    && d_deps.has_instance_output_virtuals()
                    && d_deps.has_flop_virtuals()
            })
    });
    uses_registered_multistage_parent_composed_logic
}

fn binding_uses_registered_three_stage_parent_composed_logic(
    module: &Module,
    node_id: NodeId,
) -> bool {
    if !binding_is_registered_child_input_route(module, node_id) {
        return false;
    }
    let deps = node_deps(module, node_id);
    let uses_three_stage = deps.flop_virtuals().any(|flop_id| {
        module
            .flops
            .get(flop_id as usize)
            .and_then(|flop| flop.d)
            .is_some_and(|d_stage2| {
                let d2_deps = node_deps(module, d_stage2);
                let stage2_ok = is_registered_parent_composed_logic_node(module, d_stage2)
                    && d2_deps.has_instance_output_virtuals();
                if !stage2_ok {
                    return false;
                }
                let inner_any = d2_deps.flop_virtuals().any(|inner_flop_id| {
                    module
                        .flops
                        .get(inner_flop_id as usize)
                        .and_then(|flop| flop.d)
                        .is_some_and(|d_stage3| {
                            let d3_deps = node_deps(module, d_stage3);
                            is_registered_parent_composed_logic_node(module, d_stage3)
                                && d3_deps.has_instance_output_virtuals()
                                && d3_deps.has_flop_virtuals()
                        })
                });
                inner_any
            })
    });
    uses_three_stage
}

fn binding_uses_registered_multistage_mixed_support(module: &Module, node_id: NodeId) -> bool {
    if !binding_is_registered_child_input_route(module, node_id) {
        return false;
    }
    let deps = node_deps(module, node_id);
    let uses_registered_multistage_mixed_support = deps.flop_virtuals().any(|flop_id| {
        module
            .flops
            .get(flop_id as usize)
            .and_then(|flop| flop.d)
            .is_some_and(|d| {
                let d_deps = node_deps(module, d);
                is_registered_parent_composed_logic_node(module, d)
                    && d_deps.has_ports()
                    && d_deps.has_instance_output_virtuals()
                    && d_deps.has_flop_virtuals()
            })
    });
    uses_registered_multistage_mixed_support
}

fn binding_uses_registered_multistage_instance_output(module: &Module, node_id: NodeId) -> bool {
    if !binding_is_registered_child_input_route(module, node_id) {
        return false;
    }
    let deps = node_deps(module, node_id);
    let uses_registered_multistage_instance_output = deps.flop_virtuals().any(|flop_id| {
        module
            .flops
            .get(flop_id as usize)
            .and_then(|flop| flop.d)
            .is_some_and(|d| {
                let d_deps = node_deps(module, d);
                !is_registered_parent_composed_logic_node(module, d)
                    && d_deps.flop_virtuals().any(|prev_flop_id| {
                        module
                            .flops
                            .get(prev_flop_id as usize)
                            .and_then(|flop| flop.d)
                            .is_some_and(|prev_d| {
                                node_deps(module, prev_d).has_instance_output_virtuals()
                            })
                    })
            })
    });
    uses_registered_multistage_instance_output
}

fn binding_uses_registered_multistage_parent_cone_instance_output(
    module: &Module,
    node_id: NodeId,
) -> bool {
    if !binding_is_registered_child_input_route(module, node_id) {
        return false;
    }
    let deps = node_deps(module, node_id);
    let mut support_memo = HashMap::new();
    let mut flop_memo = HashMap::new();
    let mut visiting_flops = BTreeSet::new();

    let reaches_multistage_parent_cone_instance = deps.flop_virtuals().any(|flop_id| {
        module
            .flops
            .get(flop_id as usize)
            .and_then(|flop| flop.d)
            .is_some_and(|d| {
                let d_deps = node_deps(module, d);
                !is_registered_parent_composed_logic_node(module, d)
                    && d_deps.flop_virtuals().any(|prev_flop_id| {
                        flop_d_reaches_parent_cone_instance_output(
                            module,
                            prev_flop_id,
                            &mut support_memo,
                            &mut flop_memo,
                            &mut visiting_flops,
                        )
                    })
            })
    });
    reaches_multistage_parent_cone_instance
}

fn binding_uses_registered_multistage_parent_composed_parent_cone_instance_output(
    module: &Module,
    node_id: NodeId,
) -> bool {
    if !binding_is_registered_child_input_route(module, node_id) {
        return false;
    }
    let deps = node_deps(module, node_id);
    let mut support_memo = HashMap::new();
    let mut flop_memo = HashMap::new();
    let mut visiting_flops = BTreeSet::new();

    let reaches_multistage_parent_composed_parent_cone_instance =
        deps.flop_virtuals().any(|flop_id| {
            module
                .flops
                .get(flop_id as usize)
                .and_then(|flop| flop.d)
                .is_some_and(|d| {
                    let d_deps = node_deps(module, d);
                    is_registered_parent_composed_logic_node(module, d)
                        && d_deps.has_instance_output_virtuals()
                        && d_deps.flop_virtuals().any(|prev_flop_id| {
                            flop_d_reaches_parent_cone_instance_output(
                                module,
                                prev_flop_id,
                                &mut support_memo,
                                &mut flop_memo,
                                &mut visiting_flops,
                            )
                        })
                })
        });
    reaches_multistage_parent_composed_parent_cone_instance
}

fn binding_uses_parent_cone_instance_output(module: &Module, node_id: NodeId) -> bool {
    let deps = node_deps(module, node_id);
    deps_include_parent_cone_instance_output(module, &deps)
        || deps.flop_virtuals().any(|flop_id| {
            module
                .flops
                .get(flop_id as usize)
                .and_then(|flop| flop.d)
                .is_some_and(|d| {
                    let d_deps = node_deps(module, d);
                    deps_include_parent_cone_instance_output(module, &d_deps)
                })
        })
}

fn binding_uses_parent_cone_instance_mixed_support(module: &Module, node_id: NodeId) -> bool {
    if binding_is_registered_child_input_route(module, node_id) {
        return false;
    }
    if !is_parent_composed_logic_node(module, node_id) {
        return false;
    }

    let deps = node_deps(module, node_id);
    deps.has_ports() && binding_uses_parent_cone_instance_output(module, node_id)
}
fn binding_uses_parent_cone_instance_output_through_parent_flop(
    module: &Module,
    node_id: NodeId,
) -> bool {
    if !is_registered_parent_composed_logic_node(module, node_id) {
        return false;
    }

    let deps = node_deps(module, node_id);
    if !deps.has_flop_virtuals() {
        return false;
    }

    let mut support_memo = HashMap::new();
    let mut flop_memo = HashMap::new();
    let mut visiting_flops = BTreeSet::new();
    let reaches_parent_cone_instance_through_parent_flop = deps.flop_virtuals().any(|flop_id| {
        flop_d_reaches_parent_cone_instance_output(
            module,
            flop_id,
            &mut support_memo,
            &mut flop_memo,
            &mut visiting_flops,
        )
    });
    reaches_parent_cone_instance_through_parent_flop
}

fn binding_uses_parent_cone_instance_flop_mixed_support(module: &Module, node_id: NodeId) -> bool {
    let deps = node_deps(module, node_id);
    deps.has_ports()
        && binding_uses_parent_cone_instance_output_through_parent_flop(module, node_id)
}

fn binding_uses_registered_parent_cone_instance_output(module: &Module, node_id: NodeId) -> bool {
    if !binding_is_registered_child_input_route(module, node_id) {
        return false;
    }
    let deps = node_deps(module, node_id);
    let uses_registered_parent_cone_instance = deps.flop_virtuals().any(|flop_id| {
        module
            .flops
            .get(flop_id as usize)
            .and_then(|flop| flop.d)
            .is_some_and(|d| {
                let d_deps = node_deps(module, d);
                deps_include_parent_cone_instance_output(module, &d_deps)
            })
    });
    uses_registered_parent_cone_instance
}

fn binding_uses_registered_parent_cone_instance_mixed_support(
    module: &Module,
    node_id: NodeId,
) -> bool {
    if !binding_is_registered_child_input_route(module, node_id) {
        return false;
    }
    let deps = node_deps(module, node_id);
    let uses_registered_parent_cone_instance_mixed_support = deps.flop_virtuals().any(|flop_id| {
        module
            .flops
            .get(flop_id as usize)
            .and_then(|flop| flop.d)
            .is_some_and(|d| {
                let d_deps = node_deps(module, d);
                is_registered_parent_composed_logic_node(module, d)
                    && d_deps.has_ports()
                    && deps_include_parent_cone_instance_output(module, &d_deps)
            })
    });
    uses_registered_parent_cone_instance_mixed_support
}

fn deps_include_parent_cone_instance_output(module: &Module, deps: &crate::ir::DepSet) -> bool {
    deps.instance_output_virtuals().any(|(instance, _)| {
        module
            .instances
            .get(instance as usize)
            .is_some_and(|inst| inst.role == InstanceRole::ParentCone)
    })
}

fn is_registered_parent_composed_logic_node(module: &Module, node_id: NodeId) -> bool {
    matches!(
        module.nodes[node_id as usize],
        Node::Gate { op, .. }
            if !matches!(op, GateOp::Slice { .. } | GateOp::Concat)
    )
}

fn ratio(numer: usize, denom: usize) -> f64 {
    if denom == 0 {
        0.0
    } else {
        numer as f64 / denom as f64
    }
}

/// Deterministic, dependency-free 64-bit FNV-1a hash. Used as the
/// canonical-module-signature backbone so signatures are stable across
/// runs and across rust versions without pulling in a hashing crate.
fn fnv1a_64_init() -> u64 {
    0xcbf29ce484222325
}
fn fnv1a_64_extend(state: u64, bytes: &[u8]) -> u64 {
    let mut h = state;
    for b in bytes {
        h ^= *b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}
fn fnv1a_64_u64(state: u64, value: u64) -> u64 {
    fnv1a_64_extend(state, &value.to_le_bytes())
}
fn fnv1a_64_u32(state: u64, value: u32) -> u64 {
    fnv1a_64_extend(state, &value.to_le_bytes())
}

fn bitmask(width: u32) -> u128 {
    if width >= 128 {
        u128::MAX
    } else {
        (1u128 << width) - 1
    }
}

const MAX_SEMANTIC_MODULE_SUPPORT_BITS: u32 = 12;
const MAX_SEMANTIC_MODULE_NODES: usize = 128;
const MAX_SEMANTIC_MODULE_INSTANCES: usize = 8;
const BASELINE_SEMANTIC_MODULE_SUPPORT_BITS: usize = 10;
const MAX_SEMANTIC_MODULE_WORK_UNITS: usize =
    (1usize << BASELINE_SEMANTIC_MODULE_SUPPORT_BITS) * MAX_SEMANTIC_MODULE_NODES;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct SemanticModuleProof {
    contains_instances: bool,
    input_ports: Vec<(PortId, u32)>,
    output_ports: Vec<(PortId, u32)>,
    outputs_by_assignment: Vec<Vec<u128>>,
}

struct InstanceSemanticView {
    proof: SemanticModuleProof,
    input_binding_by_port: BTreeMap<PortId, NodeId>,
    output_index_by_port: BTreeMap<PortId, usize>,
}

/// Bounded whole-module semantic proof with a design environment for
/// resolving pure-combinational child instances. The proof deliberately
/// keys ports by `(PortId, width)`, not by width alone, because module
/// dedup rewrites `Instance.module` names while preserving parent-side
/// port-id bindings.
pub(crate) fn semantic_module_proof_with_modules(
    module: &Module,
    modules: &BTreeMap<&str, &Module>,
) -> Option<SemanticModuleProof> {
    semantic_module_proof_inner(module, Some(modules), &mut BTreeSet::new())
}

fn semantic_module_proof_inner(
    module: &Module,
    modules: Option<&BTreeMap<&str, &Module>>,
    visiting: &mut BTreeSet<String>,
) -> Option<SemanticModuleProof> {
    if module.has_local_flops()
        || module.has_local_memories()
        || module.has_local_fsms()
        || module.param_env.is_some()
        || module.aggregate_layout.is_some()
    {
        return None;
    }

    if !visiting.insert(module.name.clone()) {
        return None;
    }
    let proof = semantic_module_proof_body(module, modules, visiting);
    visiting.remove(&module.name);
    proof
}

fn semantic_module_proof_body(
    module: &Module,
    modules: Option<&BTreeMap<&str, &Module>>,
    visiting: &mut BTreeSet<String>,
) -> Option<SemanticModuleProof> {
    let instance_views = build_instance_semantic_views(module, modules, visiting)?;
    let input_ports: Vec<(PortId, u32)> = module
        .emitted_data_input_ports_in(modules)
        .map(|port| (port.id, port.width))
        .collect();
    let output_ports: Vec<(PortId, u32)> = module
        .outputs
        .iter()
        .map(|port| (port.id, port.width))
        .collect();
    if output_ports.is_empty() {
        return None;
    }
    if output_ports.iter().any(|(_, width)| *width > 128) {
        return None;
    }

    let support_bits: u32 = input_ports.iter().map(|(_, width)| *width).sum();
    if support_bits > MAX_SEMANTIC_MODULE_SUPPORT_BITS {
        return None;
    }
    let assignment_count = 1usize.checked_shl(support_bits)?;

    let drive_by_port: BTreeMap<PortId, NodeId> = module.drives.iter().copied().collect();
    let drive_nodes: Vec<NodeId> = output_ports
        .iter()
        .map(|(port, _)| drive_by_port.get(port).copied())
        .collect::<Option<Vec<_>>>()?;
    let reachable_nodes = reachable_nodes_from(module, &drive_nodes)?;
    if reachable_nodes.len() > MAX_SEMANTIC_MODULE_NODES {
        return None;
    }
    if assignment_count.saturating_mul(reachable_nodes.len()) > MAX_SEMANTIC_MODULE_WORK_UNITS {
        return None;
    }

    let mut input_offsets: BTreeMap<PortId, u32> = BTreeMap::new();
    let mut next_offset = 0u32;
    for (port, width) in &input_ports {
        input_offsets.insert(*port, next_offset);
        next_offset += *width;
    }

    let mut outputs_by_assignment = Vec::with_capacity(assignment_count);
    for assignment in 0..assignment_count {
        let mut memo = HashMap::new();
        let mut row = Vec::with_capacity(drive_nodes.len());
        for &drive in &drive_nodes {
            row.push(evaluate_semantic_module_node(
                module,
                drive,
                assignment as u128,
                &input_offsets,
                &instance_views,
                &mut memo,
            )?);
        }
        outputs_by_assignment.push(row);
    }

    Some(SemanticModuleProof {
        contains_instances: !module.instances.is_empty(),
        input_ports,
        output_ports,
        outputs_by_assignment,
    })
}

fn build_instance_semantic_views(
    module: &Module,
    modules: Option<&BTreeMap<&str, &Module>>,
    visiting: &mut BTreeSet<String>,
) -> Option<Vec<InstanceSemanticView>> {
    if module.instances.is_empty() {
        return Some(Vec::new());
    }
    if module.instances.len() > MAX_SEMANTIC_MODULE_INSTANCES {
        return None;
    }
    let modules = modules?;

    let mut views = Vec::with_capacity(module.instances.len());
    for instance in &module.instances {
        if instance.id as usize != views.len() {
            return None;
        }
        if !instance.param_bindings.is_empty() {
            return None;
        }
        let child = modules.get(instance.module.as_str()).copied()?;
        let proof = semantic_module_proof_inner(child, Some(modules), visiting)?;

        let input_binding_by_port: BTreeMap<PortId, NodeId> =
            instance.inputs.iter().copied().collect();
        if input_binding_by_port.len() != proof.input_ports.len() {
            return None;
        }
        for (port, width) in &proof.input_ports {
            let source = *input_binding_by_port.get(port)?;
            if module.nodes.get(source as usize)?.width() != *width {
                return None;
            }
        }

        let output_index_by_port: BTreeMap<PortId, usize> = proof
            .output_ports
            .iter()
            .enumerate()
            .map(|(idx, (port, _))| (*port, idx))
            .collect();
        views.push(InstanceSemanticView {
            proof,
            input_binding_by_port,
            output_index_by_port,
        });
    }

    Some(views)
}

pub(crate) fn semantic_module_signature_hash(proof: &SemanticModuleProof) -> u64 {
    let mut h = fnv1a_64_init();
    h = fnv1a_64_extend(h, b"semantic-module-v2");
    h = fnv1a_64_u64(h, proof.contains_instances as u64);
    h = fnv1a_64_u64(h, proof.input_ports.len() as u64);
    for (port, width) in &proof.input_ports {
        h = fnv1a_64_u32(h, *port);
        h = fnv1a_64_u32(h, *width);
    }
    h = fnv1a_64_u64(h, proof.output_ports.len() as u64);
    for (port, width) in &proof.output_ports {
        h = fnv1a_64_u32(h, *port);
        h = fnv1a_64_u32(h, *width);
    }
    h = fnv1a_64_u64(h, proof.outputs_by_assignment.len() as u64);
    for row in &proof.outputs_by_assignment {
        h = fnv1a_64_u64(h, row.len() as u64);
        for value in row {
            h = fnv1a_64_extend(h, &value.to_le_bytes());
        }
    }
    h
}

fn reachable_nodes_from(module: &Module, roots: &[NodeId]) -> Option<BTreeSet<NodeId>> {
    let mut seen = BTreeSet::new();
    let mut stack = roots.to_vec();
    while let Some(node_id) = stack.pop() {
        if node_id as usize >= module.nodes.len() {
            return None;
        }
        if !seen.insert(node_id) {
            continue;
        }
        match &module.nodes[node_id as usize] {
            Node::Gate { operands, .. } => stack.extend(operands.iter().copied()),
            Node::InstanceOutput { instance, .. } => {
                let instance = module.instances.get(*instance as usize)?;
                stack.extend(instance.inputs.iter().map(|(_, node)| *node));
            }
            _ => {}
        }
    }
    Some(seen)
}

fn evaluate_semantic_module_node(
    module: &Module,
    node_id: NodeId,
    assignment: u128,
    input_offsets: &BTreeMap<PortId, u32>,
    instance_views: &[InstanceSemanticView],
    memo: &mut HashMap<NodeId, u128>,
) -> Option<u128> {
    if let Some(&value) = memo.get(&node_id) {
        return Some(value);
    }
    if node_id as usize >= module.nodes.len() {
        return None;
    }

    let value = match &module.nodes[node_id as usize] {
        Node::PrimaryInput { port, width } => {
            if *width > 128 {
                return None;
            }
            let offset = *input_offsets.get(port)?;
            (assignment >> offset) & bitmask(*width)
        }
        Node::Constant { width, value } => {
            if *width > 128 {
                return None;
            }
            *value & bitmask(*width)
        }
        Node::Gate {
            op,
            operands,
            width,
            ..
        } => {
            if *width > 128 {
                return None;
            }
            let width_mask = bitmask(*width);
            let operand_values: Vec<u128> = operands
                .iter()
                .map(|&operand| {
                    evaluate_semantic_module_node(
                        module,
                        operand,
                        assignment,
                        input_offsets,
                        instance_views,
                        memo,
                    )
                })
                .collect::<Option<Vec<_>>>()?;
            match op {
                GateOp::And => operand_values
                    .iter()
                    .copied()
                    .fold(width_mask, |acc, v| acc & v),
                GateOp::Or => operand_values.iter().copied().fold(0u128, |acc, v| acc | v),
                GateOp::Xor => operand_values.iter().copied().fold(0u128, |acc, v| acc ^ v),
                GateOp::Not => (!operand_values[0]) & width_mask,
                GateOp::Add => operand_values
                    .iter()
                    .copied()
                    .fold(0u128, |acc, v| acc.wrapping_add(v) & width_mask),
                GateOp::Sub => operand_values[0].wrapping_sub(operand_values[1]) & width_mask,
                GateOp::Mul => operand_values
                    .iter()
                    .copied()
                    .fold(1u128, |acc, v| acc.wrapping_mul(v) & width_mask),
                GateOp::Eq => (operand_values[0] == operand_values[1]) as u128,
                GateOp::Neq => (operand_values[0] != operand_values[1]) as u128,
                GateOp::Lt => (operand_values[0] < operand_values[1]) as u128,
                GateOp::Gt => (operand_values[0] > operand_values[1]) as u128,
                GateOp::Le => (operand_values[0] <= operand_values[1]) as u128,
                GateOp::Ge => (operand_values[0] >= operand_values[1]) as u128,
                GateOp::Mux => {
                    if operand_values[0] == 0 {
                        operand_values[2] & width_mask
                    } else {
                        operand_values[1] & width_mask
                    }
                }
                GateOp::CaseMux => {
                    let sel = operand_values[0] as usize;
                    let data_arms = operand_values.len().saturating_sub(1);
                    if sel < data_arms {
                        operand_values[sel + 1] & width_mask
                    } else {
                        0
                    }
                }
                GateOp::CasezMux => {
                    let sel_width = module.nodes[operands[0] as usize].width();
                    if sel_width > 128 {
                        return None;
                    }
                    let sel_mask = bitmask(sel_width);
                    let sel = operand_values[0] & sel_mask;
                    let mut matched = None;
                    for arm in operand_values[1..].chunks_exact(3) {
                        let pattern = arm[0] & sel_mask;
                        let wildcard_mask = arm[1] & sel_mask;
                        let care_mask = (!wildcard_mask) & sel_mask;
                        if (sel & care_mask) == (pattern & care_mask) {
                            matched = Some(arm[2] & width_mask);
                            break;
                        }
                    }
                    matched.unwrap_or(0)
                }
                GateOp::ForFold {
                    kind,
                    trip_count,
                    chunk_width,
                } => {
                    if *chunk_width > 128 {
                        return None;
                    }
                    let mut acc = match kind {
                        crate::ir::ForFoldKind::And => bitmask(*chunk_width),
                        crate::ir::ForFoldKind::Xor
                        | crate::ir::ForFoldKind::Or
                        | crate::ir::ForFoldKind::Add => 0,
                    };
                    for idx in 0..*trip_count {
                        let shift = idx.saturating_mul(*chunk_width);
                        let chunk = if shift >= 128 {
                            0
                        } else {
                            (operand_values[0] >> shift) & bitmask(*chunk_width)
                        };
                        acc = match kind {
                            crate::ir::ForFoldKind::Xor => (acc ^ chunk) & width_mask,
                            crate::ir::ForFoldKind::Or => (acc | chunk) & width_mask,
                            crate::ir::ForFoldKind::And => (acc & chunk) & width_mask,
                            crate::ir::ForFoldKind::Add => acc.wrapping_add(chunk) & width_mask,
                        };
                    }
                    acc & width_mask
                }
                GateOp::Slice { hi, lo } => {
                    let slice_width = hi - lo + 1;
                    if slice_width > 128 {
                        return None;
                    }
                    (operand_values[0] >> lo) & bitmask(slice_width)
                }
                GateOp::Concat => {
                    let mut out = 0u128;
                    for (&operand, operand_value) in operands.iter().zip(operand_values.iter()) {
                        let operand_width = module.nodes[operand as usize].width();
                        if operand_width > 128 {
                            return None;
                        }
                        out = if operand_width >= 128 {
                            operand_value & bitmask(operand_width)
                        } else {
                            (out << operand_width) | (operand_value & bitmask(operand_width))
                        };
                    }
                    out & width_mask
                }
                GateOp::RedAnd => {
                    let src_width = module.nodes[operands[0] as usize].width();
                    if src_width > 128 {
                        return None;
                    }
                    (operand_values[0] == bitmask(src_width)) as u128
                }
                GateOp::RedOr => (operand_values[0] != 0) as u128,
                GateOp::RedXor => (operand_values[0].count_ones() & 1) as u128,
                GateOp::Shl => {
                    let amt = operand_values[1];
                    if amt >= u128::from(*width) {
                        0
                    } else {
                        operand_values[0].wrapping_shl(amt as u32) & width_mask
                    }
                }
                GateOp::Shr => {
                    let amt = operand_values[1];
                    if amt >= u128::from(*width) {
                        0
                    } else {
                        (operand_values[0] >> amt) & width_mask
                    }
                }
            }
        }
        Node::InstanceOutput {
            instance,
            port,
            width,
        } => {
            if *width > 128 {
                return None;
            }
            let view = instance_views.get(*instance as usize)?;
            let mut child_assignment = 0u128;
            let mut next_offset = 0u32;
            for (child_port, child_width) in &view.proof.input_ports {
                let source = *view.input_binding_by_port.get(child_port)?;
                let source_value = evaluate_semantic_module_node(
                    module,
                    source,
                    assignment,
                    input_offsets,
                    instance_views,
                    memo,
                )?;
                child_assignment |= (source_value & bitmask(*child_width)) << next_offset;
                next_offset += *child_width;
            }
            let output_idx = *view.output_index_by_port.get(port)?;
            let (_, expected_width) = view.proof.output_ports.get(output_idx)?;
            if expected_width != width {
                return None;
            }
            let row = view
                .proof
                .outputs_by_assignment
                .get(child_assignment as usize)?;
            *row.get(output_idx)? & bitmask(*width)
        }
        Node::FlopQ { .. } | Node::MemRead { .. } | Node::FsmOut { .. } => {
            return None;
        }
    };

    memo.insert(node_id, value);
    Some(value)
}

/// Canonical, deterministic signature of a `Module`'s structure.
///
/// Two modules with the same signature have isomorphic ports
/// (direction + width sequence), nodes (kind + width + operand
/// structure), drives, flops, and instance interfaces (instance
/// child-module names are intentionally excluded so that
/// structurally-identical parents that instantiate distinctly-named
/// but structurally-identical children share a signature).
///
/// First slice of hierarchy-aware identity (PNT-3). Future slices will
/// use this signature to drive `Design::modules` deduplication when
/// `IdentityMode::NodeId` is active and to extend the doctrine "NodeId
/// = identity of an expression" up to "ModuleId = identity of a
/// hierarchical module template".
pub(crate) fn canonical_module_signature(module: &Module) -> u64 {
    // Phase 5 parameter-aware identity (PHASE-5-PARAMETERIZATION.2.3).
    // A parameterized module is width-homogeneous by the
    // `crate::ir::param` soundness gate (every width == design_value),
    // and instantiations override the width via `#(.W(v))`. So two
    // structurally-identical parameterizable templates that differ
    // ONLY in their concrete `design_value` are the *same* template
    // and must share a signature: hash a normalized sentinel in place
    // of any width equal to `design_value`. A one-time
    // `param_env`-presence marker keeps a parameterized template from
    // ever aliasing a structurally-identical *concrete* module (which
    // hashes its real widths and marker 0). Non-parameterized modules
    // (every default-off / pre-Phase-5 module, including the whole r87
    // hierarchy bank) are byte-identical to the previous signature.
    const PARAM_WIDTH_SENTINEL: u32 = u32::MAX;
    let wsig = |w: u32| -> u32 {
        match &module.param_env {
            Some(env) if w == env.design_value => PARAM_WIDTH_SENTINEL,
            _ => w,
        }
    };

    let mut h = fnv1a_64_init();
    h = fnv1a_64_u32(h, module.param_env.is_some() as u32);
    h = fnv1a_64_u64(h, module.inputs.len() as u64);
    for port in &module.inputs {
        h = fnv1a_64_u32(h, wsig(port.width));
    }
    h = fnv1a_64_u64(h, module.outputs.len() as u64);
    for port in &module.outputs {
        h = fnv1a_64_u32(h, wsig(port.width));
    }
    h = fnv1a_64_u32(h, module.clock.is_some() as u32);
    h = fnv1a_64_u32(h, module.reset.is_some() as u32);
    h = fnv1a_64_u64(h, module.nodes.len() as u64);
    for node in &module.nodes {
        match node {
            Node::PrimaryInput { port, width } => {
                h = fnv1a_64_u32(h, 1);
                h = fnv1a_64_u32(h, *port);
                h = fnv1a_64_u32(h, wsig(*width));
            }
            Node::Constant { width, value } => {
                h = fnv1a_64_u32(h, 2);
                h = fnv1a_64_u32(h, wsig(*width));
                h = fnv1a_64_extend(h, &value.to_le_bytes());
            }
            Node::MemRead { mem, width } => {
                h = fnv1a_64_u32(h, 6);
                h = fnv1a_64_u32(h, *mem);
                h = fnv1a_64_u32(h, wsig(*width));
            }
            Node::FsmOut { fsm, width } => {
                h = fnv1a_64_u32(h, 7);
                h = fnv1a_64_u32(h, *fsm);
                h = fnv1a_64_u32(h, wsig(*width));
            }
            Node::FlopQ { flop, width } => {
                h = fnv1a_64_u32(h, 3);
                h = fnv1a_64_u32(h, *flop);
                h = fnv1a_64_u32(h, wsig(*width));
            }
            Node::InstanceOutput {
                instance,
                port,
                width,
            } => {
                h = fnv1a_64_u32(h, 4);
                h = fnv1a_64_u32(h, *instance);
                h = fnv1a_64_u32(h, *port);
                h = fnv1a_64_u32(h, wsig(*width));
            }
            Node::Gate {
                op,
                operands,
                width,
                ..
            } => {
                h = fnv1a_64_u32(h, 5);
                h = fnv1a_64_u32(h, gate_op_kind_tag(*op));
                h = fnv1a_64_u32(h, wsig(*width));
                h = fnv1a_64_u64(h, operands.len() as u64);
                for operand in operands {
                    h = fnv1a_64_u32(h, *operand);
                }
            }
        }
    }
    h = fnv1a_64_u64(h, module.drives.len() as u64);
    for (port, node_id) in &module.drives {
        h = fnv1a_64_u32(h, *port);
        h = fnv1a_64_u32(h, *node_id);
    }
    h = fnv1a_64_u64(h, module.flops.len() as u64);
    for flop in &module.flops {
        h = fnv1a_64_u32(h, wsig(flop.width));
        h = fnv1a_64_u32(h, flop.d.unwrap_or(u32::MAX));
    }
    h = fnv1a_64_u64(h, module.instances.len() as u64);
    for instance in &module.instances {
        // Intentionally exclude instance.module (child module name) and
        // instance.name (instance identifier) so that
        // structurally-identical parents that instantiate distinctly-named
        // children still share a signature.
        h = fnv1a_64_u32(h, instance.role as u32);
        h = fnv1a_64_u64(h, instance.inputs.len() as u64);
        for (port, node_id) in &instance.inputs {
            h = fnv1a_64_u32(h, *port);
            h = fnv1a_64_u32(h, *node_id);
        }
    }
    h
}

fn gate_op_kind_tag(op: GateOp) -> u32 {
    use GateOp::*;
    match op {
        And => 0,
        Or => 1,
        Xor => 2,
        Not => 3,
        Add => 4,
        Sub => 5,
        Mul => 6,
        Eq => 7,
        Neq => 8,
        Lt => 9,
        Gt => 10,
        Le => 11,
        Ge => 12,
        Mux => 13,
        CaseMux => 14,
        CasezMux => 15,
        ForFold { .. } => 16,
        Slice { .. } => 17,
        Concat => 18,
        RedAnd => 19,
        RedOr => 20,
        RedXor => 21,
        Shl => 22,
        Shr => 23,
    }
}

/// Canonical lowercase name per `GateOp`. Kept here (duplicated
/// from `emit::sv::gate_kind_name`) to avoid a cross-module
/// coupling — `metrics` must stay independent of `emit`.
fn gate_kind_name(op: GateOp) -> &'static str {
    use GateOp::*;
    match op {
        And => "and",
        Or => "or",
        Xor => "xor",
        Not => "not",
        Add => "add",
        Sub => "sub",
        Mul => "mul",
        Eq => "eq",
        Neq => "neq",
        Lt => "lt",
        Gt => "gt",
        Le => "le",
        Ge => "ge",
        Mux => "mux",
        CaseMux => "case_mux",
        CasezMux => "casez_mux",
        ForFold { kind, .. } => match kind {
            crate::ir::ForFoldKind::Xor => "for_fold_xor",
            crate::ir::ForFoldKind::Or => "for_fold_or",
            crate::ir::ForFoldKind::And => "for_fold_and",
            crate::ir::ForFoldKind::Add => "for_fold_add",
        },
        Slice { .. } => "slice",
        Concat => "concat",
        RedAnd => "red_and",
        RedOr => "red_or",
        RedXor => "red_xor",
        Shl => "shl",
        Shr => "shr",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{DepSet, Direction, FlopKind, FlopMux, Port};
    use crate::Config;
    use crate::Generator;

    #[test]
    fn metrics_on_empty_module() {
        let m = Module {
            name: "empty".into(),
            ..Module::default()
        };
        let met = compute(&m);
        assert_eq!(met.num_nodes, 0);
        assert_eq!(met.num_gates, 0);
        assert_eq!(met.num_flops, 0);
    }

    #[test]
    fn metrics_count_emitted_combinational_functions() {
        // `STRUCTURED-EMISSION-EXPANSION.2b.2a` — the metric is the count
        // of gates marked for the `function automatic` emit-projection.
        let mut m = Module {
            name: "fe".into(),
            ..Module::default()
        };
        m.inputs.push(Port {
            id: 0,
            name: "a".into(),
            width: 4,
            dir: Direction::In,
        });
        m.nodes.push(Node::PrimaryInput { port: 0, width: 4 }); // id 0
        let (g, _) = m.intern_gate(GateOp::And, vec![0, 0], 4, DepSet::from_port(0));
        // Unmarked ⇒ zero.
        assert_eq!(compute(&m).num_emitted_combinational_functions, 0);
        // Marked ⇒ counted.
        m.function_emit_gates.insert(g);
        assert_eq!(compute(&m).num_emitted_combinational_functions, 1);
    }

    #[test]
    fn metrics_count_emitted_generate_loops() {
        // `STRUCTURED-EMISSION-EXPANSION.4b.2a` — the metric is the count of
        // `{N{x}}` replication gates marked for the `generate for` loop
        // emit-projection.
        let mut m = Module {
            name: "gl".into(),
            ..Module::default()
        };
        m.inputs.push(Port {
            id: 0,
            name: "sel".into(),
            width: 1,
            dir: Direction::In,
        });
        m.nodes.push(Node::PrimaryInput { port: 0, width: 1 }); // id 0
        let (g, _) = m.intern_gate(GateOp::Concat, vec![0, 0, 0, 0], 4, DepSet::from_port(0));
        // Unmarked ⇒ zero.
        assert_eq!(compute(&m).num_emitted_generate_loops, 0);
        // Marked ⇒ counted.
        m.generate_loop_gates.insert(g);
        assert_eq!(compute(&m).num_emitted_generate_loops, 1);
    }

    #[test]
    fn metrics_count_emitted_combinational_tasks() {
        // `STRUCTURED-EMISSION-EXPANSION.6b.2a` — the metric is the count of
        // combinational gates marked for the `task automatic` emit-projection.
        let mut m = Module {
            name: "te".into(),
            ..Module::default()
        };
        m.inputs.push(Port {
            id: 0,
            name: "a".into(),
            width: 4,
            dir: Direction::In,
        });
        m.nodes.push(Node::PrimaryInput { port: 0, width: 4 }); // id 0
        let (g, _) = m.intern_gate(GateOp::And, vec![0, 0], 4, DepSet::from_port(0));
        // Unmarked ⇒ zero.
        assert_eq!(compute(&m).num_emitted_combinational_tasks, 0);
        // Marked ⇒ counted.
        m.task_emit_gates.insert(g);
        assert_eq!(compute(&m).num_emitted_combinational_tasks, 1);
    }

    #[test]
    fn metrics_count_emitted_cone_functions() {
        // `STRUCTURED-EMISSION-EXPANSION.10b.2` — the metric is the count of
        // combinational cones marked for the multi-gate `function automatic`
        // emit-projection (one entry per cone root → its absorbed interiors).
        let mut m = Module {
            name: "cf".into(),
            ..Module::default()
        };
        m.inputs.push(Port {
            id: 0,
            name: "a".into(),
            width: 4,
            dir: Direction::In,
        });
        m.nodes.push(Node::PrimaryInput { port: 0, width: 4 }); // id 0
        let (interior, _) = m.intern_gate(GateOp::And, vec![0, 0], 4, DepSet::from_port(0));
        let (root, _) = m.intern_gate(GateOp::Or, vec![interior, 0], 4, DepSet::from_port(0));
        // Unmarked ⇒ zero.
        assert_eq!(compute(&m).num_emitted_cone_functions, 0);
        // Marked ⇒ counted (one cone: root absorbs the single-use interior).
        m.cone_function_gates.insert(root, vec![interior]);
        assert_eq!(compute(&m).num_emitted_cone_functions, 1);
    }

    #[test]
    fn metrics_count_gates_by_kind() {
        let mut m = Module {
            name: "k".into(),
            ..Module::default()
        };
        m.inputs.push(Port {
            id: 0,
            name: "a".into(),
            width: 4,
            dir: Direction::In,
        });
        m.nodes.push(Node::PrimaryInput { port: 0, width: 4 });
        let (g1, _) = m.intern_gate(GateOp::And, vec![0, 0], 4, DepSet::from_port(0));
        let (g2, _) = m.intern_gate(GateOp::Mux, vec![0, g1, g1], 4, DepSet::from_port(0));
        let _ = g2;
        let met = compute(&m);
        assert_eq!(met.gates_by_kind.get("and").copied(), Some(1));
        assert_eq!(met.gates_by_kind.get("mux").copied(), Some(1));
        // Mux with equal data arms is the degenerate form.
        assert_eq!(met.num_muxes_2to1, 1);
        assert_eq!(met.num_muxes_degenerate, 1);
    }

    #[test]
    fn metrics_count_operator_gates_with_duplicate_operands() {
        // SIGNOFF-AUTOMATION-EXPANSION.2b: an Add whose two operand
        // slots are the same NodeId counts once; a distinct-operand
        // Mul does not. Mirrors the degenerate-mux metric for the
        // `operand_duplication_rate` knob.
        let mut m = Module {
            name: "dup".into(),
            ..Module::default()
        };
        m.inputs.push(Port {
            id: 0,
            name: "a".into(),
            width: 4,
            dir: Direction::In,
        });
        m.inputs.push(Port {
            id: 1,
            name: "b".into(),
            width: 4,
            dir: Direction::In,
        });
        m.nodes.push(Node::PrimaryInput { port: 0, width: 4 });
        m.nodes.push(Node::PrimaryInput { port: 1, width: 4 });
        // Add(a, a): a repeated operand slot.
        let (dup_add, _) = m.intern_gate(GateOp::Add, vec![0, 0], 4, DepSet::from_port(0));
        // Mul(a, b): distinct operands — must not count.
        let dep = DepSet::union(&[&DepSet::from_port(0), &DepSet::from_port(1)]);
        let (_, _) = m.intern_gate(GateOp::Mul, vec![0, 1], 4, dep);
        let _ = dup_add;
        let met = compute(&m);
        assert_eq!(met.num_operator_gates_with_duplicate_operands, 1);
    }

    #[test]
    fn metrics_count_flops_by_shape() {
        let mut m = Module {
            name: "f".into(),
            ..Module::default()
        };
        m.flops.push(crate::ir::Flop {
            id: 0,
            width: 4,
            d: Some(0),
            q: 0,
            reset_val: 0,
            reset_kind: crate::ir::ResetKind::Async,
            kind: FlopKind::ZeroDefault,
            mux: FlopMux::None,
        });
        m.flops.push(crate::ir::Flop {
            id: 1,
            width: 4,
            d: Some(0),
            q: 0,
            reset_val: 0,
            reset_kind: crate::ir::ResetKind::Async,
            kind: FlopKind::QFeedback,
            mux: FlopMux::OneHot(vec![]),
        });
        m.nodes.push(Node::Constant { width: 4, value: 0 });
        m.flops_merged = 1;
        m.bisimulation_flops_merged = 5;
        m.fsms_merged = 3;
        m.semantic_gates_merged = 2;
        let met = compute(&m);
        assert_eq!(met.num_flops, 2);
        assert_eq!(met.flops_zero_default, 1);
        assert_eq!(met.flops_qfeedback, 1);
        assert_eq!(met.flops_mux_none, 1);
        assert_eq!(met.flops_mux_one_hot, 1);
        assert_eq!(met.flops_merged, 1);
        assert_eq!(met.bisimulation_flops_merged, 5);
        assert_eq!(met.fsms_merged, 3);
        assert_eq!(met.semantic_gates_merged, 2);
    }

    #[test]
    fn metrics_distinguish_constant_and_variable_shift_rhs() {
        let mut m = Module {
            name: "shift_shapes".into(),
            ..Module::default()
        };
        m.inputs.push(Port {
            id: 0,
            name: "a".into(),
            width: 4,
            dir: Direction::In,
        });
        m.inputs.push(Port {
            id: 1,
            name: "s".into(),
            width: 4,
            dir: Direction::In,
        });
        m.nodes.push(Node::PrimaryInput { port: 0, width: 4 });
        m.nodes.push(Node::PrimaryInput { port: 1, width: 4 });
        m.nodes.push(Node::Constant { width: 4, value: 1 });
        let _ = m.intern_gate(GateOp::Shl, vec![0, 2], 4, DepSet::from_port(0));
        let _ = m.intern_gate(GateOp::Shr, vec![0, 1], 4, DepSet::from_port(0));

        let met = compute(&m);
        assert_eq!(met.num_constant_shift_gates, 1);
        assert_eq!(met.num_variable_shift_gates, 1);
    }

    #[test]
    fn design_metrics_capture_reused_child_definitions() {
        let cfg = Config {
            seed: 11,
            hierarchy_depth: 1,
            num_leaf_modules: 2,
            num_child_instances: 5,
            ..Config::default()
        };
        cfg.validate().expect("reuse config should be valid");

        let mut g = Generator::new(cfg);
        let design = g.generate_design();
        let met = compute_design(&design);

        assert_eq!(met.num_leaf_modules, 2);
        assert_eq!(met.num_instances, 5);
        assert_eq!(met.num_unique_instantiated_modules, 2);
        assert_eq!(met.num_unused_leaf_modules, 0);
        assert_eq!(met.num_reused_instance_slots, 3);
        assert_eq!(met.library_coverage_fraction, 1.0);
        assert_eq!(met.unused_library_fraction, 0.0);
        assert_eq!(met.instance_reuse_fraction, 3.0 / 5.0);
        assert_eq!(met.instance_to_library_ratio, 2.5);
        assert_eq!(met.avg_instances_per_unique_instantiated_module, 2.5);
        assert_eq!(met.num_single_use_instantiated_modules, 0);
        assert_eq!(met.num_multiuse_instantiated_modules, 2);
        assert_eq!(met.single_use_instantiated_module_fraction, 0.0);
        assert_eq!(
            met.instantiated_module_histogram.values().sum::<usize>(),
            5,
            "histogram should account for every instantiated child"
        );
    }

    #[test]
    fn design_metrics_capture_underinstantiated_library() {
        let cfg = Config {
            seed: 17,
            hierarchy_depth: 1,
            num_leaf_modules: 4,
            num_child_instances: 2,
            ..Config::default()
        };
        cfg.validate()
            .expect("under-instantiation config should be valid");

        let mut g = Generator::new(cfg);
        let design = g.generate_design();
        let met = compute_design(&design);

        assert_eq!(met.num_leaf_modules, 4);
        assert_eq!(met.num_instances, 2);
        assert_eq!(met.num_unique_instantiated_modules, 2);
        assert_eq!(met.num_unused_leaf_modules, 2);
        assert_eq!(met.num_reused_instance_slots, 0);
        assert_eq!(met.library_coverage_fraction, 0.5);
        assert_eq!(met.unused_library_fraction, 0.5);
        assert_eq!(met.instance_reuse_fraction, 0.0);
        assert_eq!(met.instance_to_library_ratio, 0.5);
        assert_eq!(met.avg_instances_per_unique_instantiated_module, 1.0);
        assert_eq!(met.num_single_use_instantiated_modules, 2);
        assert_eq!(met.num_multiuse_instantiated_modules, 0);
        assert_eq!(met.single_use_instantiated_module_fraction, 1.0);
    }

    #[test]
    fn design_metrics_capture_on_demand_single_use_child_sourcing() {
        let cfg = Config {
            seed: 23,
            hierarchy_depth: 1,
            num_child_instances: 4,
            hierarchy_child_source_mode: crate::config::HierarchyChildSourceMode::OnDemand,
            ..Config::default()
        };
        cfg.validate()
            .expect("on-demand depth-1 hierarchy config should be valid");

        let mut g = Generator::new(cfg);
        let design = g.generate_design();
        let met = compute_design(&design);

        assert_eq!(met.num_instances, 4);
        assert_eq!(met.num_unique_instantiated_modules, 4);
        assert_eq!(met.num_reused_instance_slots, 0);
        assert_eq!(met.library_coverage_fraction, 1.0);
        assert_eq!(met.unused_library_fraction, 0.0);
        assert_eq!(met.avg_instances_per_unique_instantiated_module, 1.0);
        assert_eq!(met.num_single_use_instantiated_modules, 4);
        assert_eq!(met.num_multiuse_instantiated_modules, 0);
        assert_eq!(met.single_use_instantiated_module_fraction, 1.0);
        assert_eq!(met.num_profiled_module_definitions, 4);
        assert_eq!(met.num_profiled_instantiated_modules, 4);
        assert_eq!(met.num_profiled_instance_slots, 4);
        assert_eq!(met.profiled_instantiated_module_fraction, 1.0);
        assert_eq!(met.profiled_instance_fraction, 1.0);
        assert_eq!(met.dep_bearing_child_input_binding_fraction, 1.0);
    }

    #[test]
    fn design_metrics_capture_parent_side_composition() {
        let cfg = Config {
            seed: 3,
            hierarchy_depth: 1,
            num_leaf_modules: 2,
            num_child_instances: 4,
            ..Config::default()
        };
        cfg.validate()
            .expect("hierarchy parent-composition config should be valid");

        let mut g = Generator::new(cfg);
        let design = g.generate_design();
        let met = compute_design(&design);

        assert_eq!(met.top_outputs_reaching_instance_outputs, met.top_outputs);
        assert_eq!(met.top_outputs_without_instance_outputs, 0);
        assert!(
            met.top_parent_composed_outputs > 0,
            "expected at least one top output to be driven by parent logic over child outputs"
        );
        assert_eq!(met.top_instance_output_dependency_fraction, 1.0);
        assert!(
            met.top_parent_composed_output_fraction > 0.0,
            "expected a non-zero composed-output fraction"
        );
        assert!(
            met.avg_instance_output_support_per_top_output >= 1.0,
            "every top output should depend on at least one child output"
        );
    }

    #[test]
    fn design_metrics_capture_parent_cone_instance_output_support() {
        let cfg = Config {
            seed: 42,
            hierarchy_depth: 1,
            num_leaf_modules: 2,
            num_child_instances: 4,
            hierarchy_sibling_route_prob: 0.0,
            hierarchy_registered_sibling_route_prob: 0.0,
            hierarchy_registered_child_input_cone_prob: 0.0,
            hierarchy_child_input_cone_prob: 0.0,
            hierarchy_parent_cone_instance_prob: 1.0,
            terminal_reuse_prob: 1.0,
            constant_prob: 0.0,
            ..Config::default()
        };
        cfg.validate()
            .expect("parent-output helper instance hierarchy config should be valid");

        let mut g = Generator::new(cfg);
        let design = g.generate_design();
        let met = compute_design(&design);

        assert!(
            met.top_parent_cone_instances > 0,
            "expected at least one top-level helper instance"
        );
        assert!(
            met.top_outputs_reaching_parent_cone_instances > 0,
            "top outputs should depend on parent-cone helper outputs"
        );
        assert!(
            met.hierarchy_outputs_reaching_parent_cone_instances > 0,
            "hierarchy-wide output metrics should record helper output support"
        );
        assert!(
            met.top_outputs_reaching_parent_cone_instance_mixed_support > 0,
            "top outputs should mix parent ports with helper outputs"
        );
        assert!(
            met.hierarchy_outputs_reaching_parent_cone_instance_mixed_support > 0,
            "hierarchy-wide output metrics should record mixed helper output support"
        );
        assert!(met.top_parent_cone_instance_output_fraction > 0.0);
        assert!(met.hierarchy_parent_cone_instance_output_fraction > 0.0);
        assert!(met.top_parent_cone_instance_mixed_support_output_fraction > 0.0);
        assert!(met.hierarchy_parent_cone_instance_mixed_support_output_fraction > 0.0);
    }
    #[test]
    fn design_metrics_capture_stateful_parent_cone_instance_mixed_output_support() {
        let cfg = Config {
            seed: 42,
            hierarchy_depth: 1,
            num_leaf_modules: 2,
            num_child_instances: 4,
            hierarchy_sibling_route_prob: 0.0,
            hierarchy_registered_sibling_route_prob: 0.0,
            hierarchy_registered_child_input_cone_prob: 0.0,
            hierarchy_child_input_cone_prob: 0.0,
            hierarchy_parent_cone_instance_prob: 1.0,
            max_parent_cone_instances_per_module: 3,
            hierarchy_parent_flop_prob: 1.0,
            max_flops_per_module: 64,
            min_width: 1,
            max_width: 8,
            max_depth: 1,
            terminal_reuse_prob: 1.0,
            constant_prob: 0.0,
            ..Config::default()
        };
        cfg.validate()
            .expect("stateful parent-output helper hierarchy config should be valid");

        let mut g = Generator::new(cfg);
        let design = g.generate_design();
        let met = compute_design(&design);

        assert!(met.top_parent_cone_instances > 0);
        assert!(met.top_local_flops > 0);
        assert!(met.top_outputs_reaching_parent_cone_instances_through_parent_flops > 0);
        assert!(
            met.top_outputs_reaching_parent_cone_instances_through_parent_flops_with_mixed_support
                > 0,
            "top outputs should mix parent ports with helper-through-parent-flop routes"
        );
        assert!(
            met.hierarchy_outputs_reaching_parent_cone_instances_through_parent_flops_with_mixed_support
                > 0,
            "hierarchy output metrics should record mixed helper-through-flop support"
        );
        assert!(met.top_parent_cone_instance_flop_mixed_support_output_fraction > 0.0);
        assert!(met.hierarchy_parent_cone_instance_flop_mixed_support_output_fraction > 0.0);
    }

    #[test]
    fn design_metrics_capture_multiple_parent_cone_instance_budget() {
        let cfg = Config {
            seed: 42,
            hierarchy_depth: 1,
            num_leaf_modules: 2,
            num_child_instances: 4,
            hierarchy_sibling_route_prob: 0.0,
            hierarchy_registered_sibling_route_prob: 0.0,
            hierarchy_registered_child_input_cone_prob: 0.0,
            hierarchy_child_input_cone_prob: 1.0,
            hierarchy_parent_cone_instance_prob: 1.0,
            max_parent_cone_instances_per_module: 3,
            terminal_reuse_prob: 1.0,
            constant_prob: 0.0,
            ..Config::default()
        };
        cfg.validate()
            .expect("budgeted parent-cone helper hierarchy config should be valid");

        let mut g = Generator::new(cfg);
        let design = g.generate_design();
        let met = compute_design(&design);

        assert_eq!(met.top_parent_cone_instances, 3);
        assert_eq!(met.hierarchy_parent_cone_instances, 3);
        assert_eq!(met.max_parent_cone_instances_per_internal_module, 3);
        assert!(
            met.child_input_bindings_from_parent_cone_instances > 0,
            "budgeted helpers should still source child-input bindings"
        );
        assert!(
            met.child_input_bindings_from_parent_cone_instance_mixed_support > 0,
            "budgeted helper child-input bindings should mix parent ports with helper outputs"
        );
        assert!(
            met.top_child_input_bindings_from_parent_cone_instance_mixed_support > 0,
            "top helper child-input bindings should record mixed parent-port support"
        );
        assert!(met.parent_cone_instance_mixed_support_child_input_binding_fraction > 0.0);
        assert!(met.top_parent_cone_instance_mixed_support_child_input_binding_fraction > 0.0);
    }

    #[test]
    fn design_metrics_capture_parent_composed_parent_cone_instance_flop_routes() {
        let cfg = Config {
            seed: 42,
            hierarchy_depth: 1,
            num_leaf_modules: 2,
            num_child_instances: 4,
            flop_prob: 0.0,
            hierarchy_sibling_route_prob: 0.0,
            hierarchy_registered_sibling_route_prob: 0.0,
            hierarchy_registered_child_input_cone_prob: 0.0,
            hierarchy_child_input_cone_prob: 1.0,
            hierarchy_parent_cone_instance_prob: 1.0,
            max_parent_cone_instances_per_module: 1,
            hierarchy_parent_flop_prob: 1.0,
            max_flops_per_module: 64,
            terminal_reuse_prob: 1.0,
            constant_prob: 0.0,
            min_width: 1,
            max_width: 8,
            max_depth: 1,
            ..Config::default()
        };
        cfg.validate()
            .expect("parent-composed helper-through-parent-flop hierarchy config should be valid");

        let mut g = Generator::new(cfg);
        let design = g.generate_design();
        let met = compute_design(&design);

        assert!(
            met.child_input_bindings_from_parent_composed_logic > 0,
            "expected parent-composed child-input bindings"
        );
        assert!(
            met.child_input_bindings_from_parent_cone_instances > 0,
            "expected parent-composed child-input bindings to depend on helper outputs"
        );
        assert!(
            met.child_input_bindings_from_parent_cone_instances_through_parent_flops > 0,
            "expected parent-composed child-input bindings to read helper outputs through parent-local flops"
        );
        assert!(
            met.child_input_bindings_from_parent_cone_instance_flop_mixed_support > 0,
            "expected helper-through-parent-flop child-input bindings to mix parent-port support"
        );
        assert!(met.top_child_input_bindings_from_parent_cone_instances_through_parent_flops > 0);
        assert!(met.top_child_input_bindings_from_parent_cone_instance_flop_mixed_support > 0);
        assert_eq!(
            met.child_input_bindings_from_registered_parent_cone_instances, 0,
            "stateful parent-composed helper bindings should not be counted as registered child-input routes"
        );
        assert_eq!(
            met.child_input_bindings_from_registered_instance_outputs, 0,
            "stateful parent-composed helper bindings should not be counted as registered sibling routes"
        );
        assert!(met.parent_cone_instance_flop_child_input_binding_fraction > 0.0);
        assert!(met.top_parent_cone_instance_flop_child_input_binding_fraction > 0.0);
        assert!(met.parent_cone_instance_flop_mixed_support_child_input_binding_fraction > 0.0);
        assert!(met.top_parent_cone_instance_flop_mixed_support_child_input_binding_fraction > 0.0);
    }

    #[test]
    fn design_metrics_capture_direct_sibling_parent_cone_instance_routes() {
        let cfg = Config {
            seed: 42,
            hierarchy_depth: 1,
            num_leaf_modules: 2,
            num_child_instances: 4,
            hierarchy_sibling_route_prob: 1.0,
            hierarchy_registered_sibling_route_prob: 0.0,
            hierarchy_registered_child_input_cone_prob: 0.0,
            hierarchy_child_input_cone_prob: 0.0,
            hierarchy_parent_cone_instance_prob: 1.0,
            max_parent_cone_instances_per_module: 3,
            hierarchy_parent_flop_prob: 0.0,
            terminal_reuse_prob: 1.0,
            constant_prob: 0.0,
            ..Config::default()
        };
        cfg.validate()
            .expect("direct sibling helper hierarchy config should be valid");

        let mut g = Generator::new(cfg);
        let design = g.generate_design();
        let met = compute_design(&design);

        assert!(
            met.child_input_bindings_from_instance_outputs > 0,
            "expected direct sibling child-input bindings"
        );
        assert_eq!(
            met.child_input_bindings_from_registered_instance_outputs, 0,
            "direct sibling helper routes should not use registered sibling flops"
        );
        assert_eq!(
            met.child_input_bindings_from_registered_parent_cone_instances, 0,
            "direct sibling helper routes should not use registered helper D paths"
        );
        assert!(
            met.child_input_bindings_from_parent_cone_instances > 0,
            "direct sibling bindings should depend on parent-cone helper outputs"
        );
        assert!(met.parent_cone_instance_child_input_binding_fraction > 0.0);
        assert!(met.top_parent_cone_instance_child_input_binding_fraction > 0.0);
    }

    #[test]
    fn design_metrics_capture_registered_parent_cone_instance_routes() {
        let cfg = Config {
            seed: 42,
            hierarchy_depth: 1,
            num_leaf_modules: 2,
            num_child_instances: 4,
            hierarchy_sibling_route_prob: 0.0,
            hierarchy_registered_sibling_route_prob: 0.0,
            hierarchy_registered_child_input_cone_prob: 1.0,
            hierarchy_child_input_cone_prob: 0.0,
            hierarchy_parent_cone_instance_prob: 1.0,
            max_parent_cone_instances_per_module: 3,
            max_flops_per_module: 8,
            terminal_reuse_prob: 1.0,
            constant_prob: 0.0,
            ..Config::default()
        };
        cfg.validate()
            .expect("registered parent-cone helper hierarchy config should be valid");

        let mut g = Generator::new(cfg);
        let design = g.generate_design();
        let met = compute_design(&design);

        assert!(
            met.child_input_bindings_from_registered_parent_composed_logic > 0,
            "expected registered parent-composed child-input bindings"
        );
        assert!(
            met.child_input_bindings_from_registered_parent_cone_instances > 0,
            "registered D cones should depend on parent-cone helper outputs"
        );
        assert!(
            met.child_input_bindings_from_parent_cone_instances
                >= met.child_input_bindings_from_registered_parent_cone_instances
        );
        assert!(met.registered_parent_cone_instance_child_input_binding_fraction > 0.0);
        assert!(met.top_registered_parent_cone_instance_child_input_binding_fraction > 0.0);
    }

    #[test]
    fn design_metrics_capture_direct_registered_sibling_parent_cone_instance_routes() {
        let cfg = Config {
            seed: 42,
            hierarchy_depth: 1,
            num_leaf_modules: 2,
            num_child_instances: 4,
            hierarchy_sibling_route_prob: 0.0,
            hierarchy_registered_sibling_route_prob: 1.0,
            hierarchy_registered_child_input_cone_prob: 0.0,
            hierarchy_child_input_cone_prob: 0.0,
            hierarchy_parent_cone_instance_prob: 1.0,
            max_parent_cone_instances_per_module: 3,
            max_flops_per_module: 8,
            terminal_reuse_prob: 1.0,
            constant_prob: 0.0,
            ..Config::default()
        };
        cfg.validate()
            .expect("direct registered sibling helper hierarchy config should be valid");

        let mut g = Generator::new(cfg);
        let design = g.generate_design();
        let met = compute_design(&design);

        assert!(
            met.child_input_bindings_from_registered_instance_outputs > 0,
            "expected registered sibling child-input bindings"
        );
        assert_eq!(
            met.child_input_bindings_from_registered_parent_composed_logic, 0,
            "direct registered sibling helper routes should not be counted as registered parent-composed D cones"
        );
        assert!(
            met.child_input_bindings_from_registered_parent_cone_instances > 0,
            "registered sibling D flops should depend on parent-cone helper outputs"
        );
        assert!(
            met.child_input_bindings_from_parent_cone_instances
                >= met.child_input_bindings_from_registered_parent_cone_instances
        );
        assert!(met.registered_parent_cone_instance_child_input_binding_fraction > 0.0);
        assert!(met.top_registered_parent_cone_instance_child_input_binding_fraction > 0.0);
    }

    #[test]
    fn design_metrics_capture_direct_registered_sibling_mixed_support_routes() {
        let cfg = Config {
            seed: 42,
            hierarchy_depth: 1,
            num_leaf_modules: 2,
            num_child_instances: 4,
            hierarchy_sibling_route_prob: 0.0,
            hierarchy_registered_sibling_route_prob: 1.0,
            hierarchy_registered_sibling_mixed_support_prob: 1.0,
            hierarchy_registered_child_input_cone_prob: 0.0,
            hierarchy_child_input_cone_prob: 0.0,
            hierarchy_parent_cone_instance_prob: 0.0,
            max_flops_per_module: 8,
            terminal_reuse_prob: 1.0,
            constant_prob: 0.0,
            ..Config::default()
        };
        cfg.validate()
            .expect("direct registered sibling mixed-support hierarchy config should be valid");

        let mut g = Generator::new(cfg);
        let design = g.generate_design();
        let met = compute_design(&design);

        assert!(
            met.child_input_bindings_from_registered_instance_outputs > 0,
            "expected registered sibling child-input bindings"
        );
        assert!(
            met.child_input_bindings_from_registered_sibling_mixed_support > 0,
            "expected direct registered sibling D paths to mix parent-port and child-output support"
        );
        assert!(
            met.top_child_input_bindings_from_registered_sibling_mixed_support > 0,
            "top metrics should record direct registered sibling mixed-support bindings"
        );
        assert_eq!(
            met.child_input_bindings_from_registered_parent_composed_logic, 0,
            "direct registered sibling mixed-support routes should not be counted as registered parent-composed D cones"
        );
        assert_eq!(
            met.child_input_bindings_from_registered_mixed_support, 0,
            "direct registered sibling mixed-support routes should stay separate from registered parent-composed mixed support"
        );
        assert!(met.registered_sibling_mixed_support_child_input_binding_fraction > 0.0);
        assert!(met.top_registered_sibling_mixed_support_child_input_binding_fraction > 0.0);
    }

    #[test]
    fn design_metrics_capture_multistage_registered_parent_cone_instance_routes() {
        let cfg = Config {
            seed: 42,
            hierarchy_depth: 1,
            num_leaf_modules: 2,
            num_child_instances: 4,
            flop_prob: 0.0,
            hierarchy_sibling_route_prob: 0.0,
            hierarchy_registered_sibling_route_prob: 1.0,
            hierarchy_registered_child_input_cone_prob: 0.0,
            hierarchy_child_input_cone_prob: 0.0,
            hierarchy_parent_cone_instance_prob: 1.0,
            max_parent_cone_instances_per_module: 1,
            hierarchy_parent_flop_prob: 0.0,
            max_flops_per_module: 8,
            terminal_reuse_prob: 1.0,
            constant_prob: 0.0,
            ..Config::default()
        };
        cfg.validate()
            .expect("multi-stage registered helper hierarchy config should be valid");

        let mut g = Generator::new(cfg);
        let design = g.generate_design();
        let met = compute_design(&design);

        assert!(
            met.child_input_bindings_from_registered_parent_cone_instances > 0,
            "expected first-stage registered helper D paths"
        );
        assert!(
            met.child_input_bindings_from_registered_multistage_instance_outputs > 0,
            "expected registered sibling routes to chain through parent-local Qs"
        );
        assert!(
            met.child_input_bindings_from_registered_multistage_parent_cone_instances > 0,
            "expected a later registered sibling route to chain from a helper-sourced parent Q"
        );
        assert_eq!(
            met.child_input_bindings_from_registered_parent_composed_logic, 0,
            "direct registered sibling helper routes should not be counted as registered parent-composed D cones"
        );
        assert!(met.registered_multistage_parent_cone_instance_child_input_binding_fraction > 0.0);
        assert!(
            met.top_registered_multistage_parent_cone_instance_child_input_binding_fraction > 0.0
        );
    }

    #[test]
    fn design_metrics_capture_multistage_registered_parent_composed_parent_cone_instance_routes() {
        let cfg = Config {
            seed: 42,
            hierarchy_depth: 1,
            num_leaf_modules: 2,
            num_child_instances: 4,
            flop_prob: 0.0,
            hierarchy_sibling_route_prob: 0.0,
            hierarchy_registered_sibling_route_prob: 0.0,
            hierarchy_registered_child_input_cone_prob: 1.0,
            hierarchy_child_input_cone_prob: 0.0,
            hierarchy_parent_cone_instance_prob: 1.0,
            max_parent_cone_instances_per_module: 1,
            hierarchy_parent_flop_prob: 0.0,
            max_flops_per_module: 8,
            terminal_reuse_prob: 1.0,
            constant_prob: 0.0,
            ..Config::default()
        };
        cfg.validate().expect(
            "multi-stage registered parent-composed helper hierarchy config should be valid",
        );

        let mut g = Generator::new(cfg);
        let design = g.generate_design();
        let met = compute_design(&design);

        assert!(
            met.child_input_bindings_from_registered_parent_composed_logic > 0,
            "expected first-stage registered parent-composed child-input bindings"
        );
        assert!(
            met.child_input_bindings_from_registered_multistage_parent_composed_logic > 0,
            "expected registered parent-composed routes to chain through parent-local Qs"
        );
        assert!(
            met.child_input_bindings_from_registered_parent_cone_instances > 0,
            "expected registered parent-composed D paths to depend on helper outputs"
        );
        assert_eq!(
            met.child_input_bindings_from_registered_multistage_parent_cone_instances, 0,
            "parent-composed helper chains should not be counted as direct sibling helper chains"
        );
        assert!(
            met.child_input_bindings_from_registered_multistage_parent_composed_parent_cone_instances
                > 0,
            "expected a later registered parent-composed route to chain from a helper-sourced parent Q"
        );
        assert!(
            met.top_child_input_bindings_from_registered_multistage_parent_composed_parent_cone_instances
                > 0
        );
        assert!(
            met.registered_multistage_parent_composed_parent_cone_instance_child_input_binding_fraction
                > 0.0
        );
        assert!(
            met.top_registered_multistage_parent_composed_parent_cone_instance_child_input_binding_fraction
                > 0.0
        );
    }

    #[test]
    fn design_metrics_capture_sibling_routed_child_inputs() {
        let cfg = Config {
            seed: 27,
            hierarchy_depth: 1,
            num_leaf_modules: 2,
            num_child_instances: 2,
            hierarchy_sibling_route_prob: 1.0,
            hierarchy_child_input_cone_prob: 0.0,
            ..Config::default()
        };
        cfg.validate()
            .expect("sibling-routing hierarchy config should be valid");

        let mut g = Generator::new(cfg);
        let design = g.generate_design();
        let met = compute_design(&design);

        assert!(
            met.child_input_bindings_from_instance_outputs > 0,
            "expected at least one child input to be sourced from a sibling output"
        );
        assert!(
            met.top_child_input_bindings_from_instance_outputs > 0,
            "top wrapper should expose sibling-routed child inputs directly"
        );
        assert!(met.instance_output_child_input_binding_fraction > 0.0);
        assert!(met.top_instance_output_child_input_binding_fraction > 0.0);
    }

    #[test]
    fn design_metrics_capture_recursive_depth_and_branching() {
        let cfg = Config {
            seed: 9,
            min_hierarchy_depth: 2,
            max_hierarchy_depth: 2,
            min_child_instances_per_module: 2,
            max_child_instances_per_module: 3,
            ..Config::default()
        };
        cfg.validate()
            .expect("bounded recursive hierarchy config should be valid");

        let mut g = Generator::new(cfg);
        let design = g.generate_design();
        let met = compute_design(&design);

        assert_eq!(met.realized_min_leaf_depth, 2);
        assert_eq!(met.realized_max_leaf_depth, 2);
        assert_eq!(met.max_module_depth, 2);
        assert_eq!(
            met.leaf_module_occurrences_by_depth.get(&2),
            Some(&met.num_leaf_module_occurrences)
        );
        assert!(met.num_internal_module_occurrences > 0);
        assert!(met.num_leaf_module_occurrences > 0);
        assert!(
            (2..=3).contains(&met.min_child_instances_per_internal_module),
            "min branching must respect the requested range"
        );
        assert!(
            (2..=3).contains(&met.max_child_instances_per_internal_module),
            "max branching must respect the requested range"
        );
        assert!(
            met.module_defs_by_depth.contains_key(&0)
                && met.module_defs_by_depth.contains_key(&1)
                && met.module_defs_by_depth.contains_key(&2),
            "depth histogram should record every realized level"
        );
        assert!(
            met.instance_slots_by_parent_depth.contains_key(&0)
                && met.instance_slots_by_parent_depth.contains_key(&1),
            "branching histogram should record internal parent depths"
        );
        assert!(
            met.avg_child_instances_by_parent_depth.contains_key(&0)
                && met.avg_child_instances_by_parent_depth.contains_key(&1),
            "per-depth branching averages should be recorded"
        );
    }

    #[test]
    fn design_metrics_capture_per_depth_branching_profile() {
        let cfg = Config {
            seed: 12,
            min_hierarchy_depth: 2,
            max_hierarchy_depth: 2,
            min_child_instances_per_module: 1,
            max_child_instances_per_module: 3,
            child_instances_per_module_by_depth: BTreeMap::from([
                (0, crate::config::CountRange { min: 4, max: 4 }),
                (1, crate::config::CountRange { min: 2, max: 2 }),
            ]),
            ..Config::default()
        };
        cfg.validate()
            .expect("per-depth recursive hierarchy config should be valid");

        let mut g = Generator::new(cfg);
        let design = g.generate_design();
        let met = compute_design(&design);

        assert_eq!(met.min_child_instances_by_parent_depth.get(&0), Some(&4));
        assert_eq!(met.max_child_instances_by_parent_depth.get(&0), Some(&4));
        assert_eq!(met.avg_child_instances_by_parent_depth.get(&0), Some(&4.0));
        assert_eq!(met.min_child_instances_by_parent_depth.get(&1), Some(&2));
        assert_eq!(met.max_child_instances_by_parent_depth.get(&1), Some(&2));
        assert_eq!(met.avg_child_instances_by_parent_depth.get(&1), Some(&2.0));
    }

    #[test]
    fn design_metrics_capture_mixed_leaf_depths() {
        let cfg = Config {
            seed: 19,
            min_hierarchy_depth: 2,
            max_hierarchy_depth: 3,
            min_child_instances_per_module: 2,
            max_child_instances_per_module: 2,
            ..Config::default()
        };
        cfg.validate()
            .expect("mixed-depth recursive hierarchy config should be valid");

        let mut g = Generator::new(cfg);
        let design = g.generate_design();
        let met = compute_design(&design);

        assert_eq!(met.realized_min_leaf_depth, 2);
        assert_eq!(met.realized_max_leaf_depth, 3);
        assert_eq!(met.max_module_depth, 3);
        assert_eq!(met.leaf_module_occurrences_by_depth.get(&2), Some(&2));
        assert_eq!(met.leaf_module_occurrences_by_depth.get(&3), Some(&4));
        assert_eq!(met.module_occurrences_by_depth.get(&0), Some(&1));
        assert_eq!(met.module_occurrences_by_depth.get(&1), Some(&2));
    }
}
