# Architecture of the Rust Implementation

## Crate layout

```
src/
├── main.rs          # CLI entry point (clap-derived); Cargo's
│                    # default run target is `anvil`, so plain
│                    # cargo run -- ... invokes this generator even
│                    # with src/bin/tool_matrix.rs present. Covers every
│                    # motif knob as a dedicated flag; wires the
│                    # tracing-subscriber from --trace <level> and
│                    # --trace-file.
├── lib.rs           # public API, re-exports Config, Generator, Module,
│                    # Design.
│                    # Trace infrastructure: TRACE_DEBUG AtomicBool,
│                    # set_trace_debug(bool), trace_verbose! macro
│                    # (gates tracing::trace! behind the debug flag so
│                    # --trace debug is strictly more verbose than high).
├── metrics.rs       # Post-hoc structural metrics walker.
│                    # compute(&Module) → Metrics { size, per-kind counts,
│                    # fanout stats, depth histogram, block counters,
│                    # AST-instance saturation, operand-arity distribution,
│                    # ... }. Serde-serializable.
├── config.rs        # Config struct + serde + CLI overlay + validation.
│                    # ConstructionStrategy enum (Sequential / Shuffled /
│                    # Interleaved / GraphFirst — the last a silent alias
│                    # for Interleaved). IdentityMode enum (`relaxed`
│                    # vs `node-id`) plus FactorizationLevel along the
│                    # chain none → cse → operand-unique →
│                    # commutative → associative → constant-fold →
│                    # peephole → e-graph (default request; bounded
│                    # semantic fragment live at the top rung).
├── ir/
│   ├── mod.rs       # re-exports.
│   ├── types.rs     # Module, Port, Node, GateOp (with Hash derive),
│   │                # Flop, FlopKind, FlopMux, MuxArm, DepSet,
│   │                # KnobId, KnobRollCounters, Design, Instance.
│   │                # Module
│   │                # carries construction-time dedup tables
│   │                # (gate_instances, const_instances), per-module
│   │                # knob mirrors (max_ast_instances,
│   │                # mux_arm_duplication_rate,
│   │                # operand_duplication_rate, identity_mode,
│   │                # factorization_level),
│   │                # and live counters (fold_identities_applied,
│   │                # peephole_rewrites_applied,
│   │                # flatten_associative_applied, flops_merged,
│   │                # semantic_gates_merged,
│   │                # nodes_compacted,
│   │                # block-build counters, knob_rolls).
│   │                # API: intern_gate() runs the full factorization
│   │                # ladder (flatten_associative → commutative sort →
│   │                # fold_constants → apply_peephole → CSE dedup)
│   │                # and returns (NodeId, is_new). intern_constant()
│   │                # is the constant analogue. Inline unit tests
│   │                # pin each layer's contract.
│   ├── compact.rs   # Post-construction finalisation helpers:
│   │                # bounded semantic gate merge + endpoint-aware
│   │                # flop merge after D-cones exist,
│   │                # plus compact_node_ids BFS from roots dropping
│   │                # unreachable gates and remapping NodeIds across
│   │                # m.nodes / m.drives / m.flops / dedup tables.
│   │                # Keeps orphan-producing rewrites Rule-18-clean at
│   │                # module finalisation. Inline unit tests.
│   └── validate.rs  # invariant + canonical-state + per-gate shape
│                    # checker, plus design-level hierarchy
│                    # validation; inline unit tests.
├── gen/
│   ├── mod.rs       # Generator struct, public entry points.
│   ├── cone.rs      # fanin-cone recursion (combinational + sequential);
│   │                # DAG-sharing fork; flop-mux assembly (one-hot,
│   │                # encoded); priority-encoder, comb-mux, linear-
│   │                # combination, const-shift, const-comparand motifs;
│   │                # snapshot/rollback for Rule 18 α enforcement;
│   │                # interleaved frame machine with existing-operand
│   │                # anti-collapse fallback; pick_terminal tiers
│   │                # (+ pick_terminal_dep_bearing strict variant);
│   │                # pick_datas_with_dup_cap / pick_signals_with_dup_rate
│   │                # helpers; inline unit tests.
│   ├── module.rs    # leaf-module generator (clk/rst_n reservation,
│   │                # pool seeding, output cones, worklist drain,
│   │                # Rule 18 safety-net orphan audit).
│   ├── hierarchy.rs # current Phase 4 slice: legacy exact depth-1
│   │                # wrapper lane plus bounded recursive lane.
│   │                # Both now also expose explicit child sourcing
│   │                # (library vs on-demand), and both build
│   │                # parent-side layers over child InstanceOutput
│   │                # leaves, sibling-routed child-input binding,
│   │                # parent-composed child-input binding,
│   │                # helper-instance sources, and optional local
│   │                # parent flops.
│   └── pool.rs      # SignalPool (width-indexed, cloneable for rewind).
└── emit/
    ├── mod.rs       # re-exports.
    └── sv.rs        # IR -> SystemVerilog. Dumb serialiser per doctrine —
                     # no filtering, no reachability checks. build_names
                     # assigns each gate a <kind>_<N> name (Rule 12);
                     # flops are flop_<id>; also emits design-aware
                     # child-module instantiations. Inline unit tests.
```

Phase 4 is now in progress: `src/gen/hierarchy.rs` owns both the older
exact depth-1 wrapper lane and the newer bounded recursive hierarchy
lane. The older wrapper-baseline surface has a repo-owned closure gate
in `tool_matrix`, and current HEAD now extends hierarchy with both
parent-side composition, bounded recursive tree planning, mixed-depth
leaf shaping inside a requested depth interval, and depth-specific
branching overrides, plus real child-input routing surfaces,
helper-instance sources for parent-composed child-input cones, direct
sibling routes, direct registered sibling routes, registered child-input
D cones, and parent-output cones, and an explicit local-parent-state
axis.

## Dependency direction

```
main  ->  lib  ->  gen  ->  ir
                    |        ^
                    v        |
                   emit -----+
```

- `ir` has zero dependencies on other modules.
- `gen` depends on `ir` (builds IR).
- `emit` depends on `ir` (reads IR).
- `gen` and `emit` do not depend on each other.
- `main` wires it all together.
- `src/bin/tool_matrix.rs` is a repo-owned auxiliary binary: it uses
  the public crate API to generate a curated scenario matrix, run
  Verilator/Yosys, and write an aggregated report. Yosys is now an
  explicit harness axis too: the binary can run the current stable
  `synth -noabc` path, the explicit ABC-enabled
  `synth -noabc; abc -fast; opt -fast; check` path, or both as
  separate sub-runs per generated file.

This means `ir` can be tested in isolation, `emit` can be tested with
hand-constructed IRs (no need to invoke the generator), and `gen` can
be tested by inspecting the IR it produces without ever emitting SV.

## Is the current codebase suited to the goal?

Yes, as a foundation.

The present decomposition already matches ANVIL's problem:

- `gen` constructs typed IR rather than text;
- `ir/types.rs` provides one combinational identity chokepoint;
- `ir/compact.rs` owns post-drain state/reachability finalisation;
- `ir/validate.rs` owns the invariant contract; and
- `emit/sv.rs` stays dumb and therefore honest.

That is the correct architectural shape for a signoff-grade legal-RTL
generator. What remains is to keep four steering gaps explicit rather
than accidental:

1. **Feature breadth**
   The current engine is still centered on the leaf-module generator in
   `src/gen/module.rs`, with a live hierarchy planner above it in
   `src/gen/hierarchy.rs`. Richer structured ops and the current
   hierarchy slice are now real generator surfaces; parameterization,
   aggregates, memories, FSMs, and broader hierarchy-aware identity are
   future work on top of this base, not evidence against it.
2. **`NodeId` as identity**
   Full factorization is only partially realized today. Combinational
   identity flows through `Module::intern_gate`, a bounded semantic
   gate-merge fragment now lives at the `e-graph` rung after
   construction, and endpoint-aware duplicate flops merge after drain,
   but stronger state equivalence and future hierarchical identity are
   not finished.
3. **Tool-clean industrialization**
   Internal tests are strong, and a repo-owned `tool_matrix` harness
   now exists, and its current stable smoke matrix is green: 15/15
   clean in Verilator and 15/15 clean in Yosys under the explicit
   `without-abc` Yosys mode. The harness now treats warnings as
   failures, so "green" means no errors and no warnings. It also has a
   distinct `with-abc` mode (or `both`) so the ABC-enabled Yosys path
   can be measured separately too. The harness now uses an explicit
   `abc -fast` path there rather than Yosys's raw default `synth`
   script because the latter was emitting a non-actionable ABC warning
   bucket on extracted combinational subnetworks. The path to the
   current green state needed both construction-time comparison proofs,
   a post-construction exact-value cleanup pass on the settled graph,
   and a repo-owned ABC script choice that stays warning-clean on the
   generated corpus. That evidence layer must keep growing with each
   new motif family until the larger phase-exit sweeps are equally
   boring.
4. **Structure-first doctrine**
   The codebase is intentionally optimized for structural legitimacy and
   synthesizability, not for proving whole-module intended behavior.
   The missing work is more legal interaction richness, not a bundled
   oracle.

## Key types at a glance

```rust
// ir/types.rs
pub struct Module {
    pub name: String,
    pub inputs: Vec<Port>,
    pub outputs: Vec<Port>,
    pub clock: Option<PortId>,
    pub reset: Option<PortId>,
    pub nodes: Vec<Node>,
    pub flops: Vec<Flop>,
    pub drives: Vec<(PortId, NodeId)>,
    // Construction-time CSE tables:
    gate_instances:  HashMap<(GateOp, Vec<NodeId>, u32), Vec<NodeId>>,
    const_instances: HashMap<(u32, u128),              Vec<NodeId>>,
    // Per-module knob mirrors:
    pub max_ast_instances:        u32,
    pub mux_arm_duplication_rate: f64,
    pub operand_duplication_rate: f64,
    pub identity_mode:            IdentityMode,
    pub factorization_level:      FactorizationLevel,
    // Block-build live counters:
    pub priority_encoder_built:  u32,
    pub comb_mux_one_hot_built:  u32,
    pub comb_mux_encoded_built:  u32,
    pub case_mux_built:          u32,
    pub casez_mux_built:         u32,
    pub for_fold_built:          u32,
    // Factorization-layer live counters:
    pub fold_identities_applied:     u64,
    pub peephole_rewrites_applied:   u64,
    pub flatten_associative_applied: u64,
    pub flops_merged:                u32,
    pub semantic_gates_merged:       u32,
    pub nodes_compacted:             u32,
    // Per-knob probability-roll counters:
    pub knob_rolls:                  KnobRollCounters,
}
impl Module {
    /// Single chokepoint for gate creation. Runs the full
    /// effective identity mode in order: associative flattening →
    /// commutative sort → constant fold → peephole → CSE, with
    /// `identity_mode = Relaxed` forcing the effective level to
    /// `None`. See `book/src/factorization.md` for the
    /// layer-by-layer view.
    pub fn intern_gate(&mut self, op, operands, width, deps) -> (NodeId, bool);
    pub fn intern_constant(&mut self, width, value) -> (NodeId, bool);

    // (Layer helpers, `pub(crate)`):
    //   fn flatten_associative(&mut self, op, operands, width) -> Option<(NodeId, bool)>;
    //   fn fold_constants     (&mut self, op, operands, width) -> Option<(NodeId, bool)>;
    //   fn apply_peephole     (&mut self, op, operands, width) -> Option<(NodeId, bool)>;
}

// ir/compact.rs
/// Post-construction bounded semantic gate-sharing pass. Under
/// `identity_mode = node-id` with effective level `>= e-graph`,
/// merges same-width combinational cones proven equivalent over
/// the same canonical leaf endpoints.
pub fn merge_equivalent_gates(m: &mut Module) -> u32;

/// Post-remap associative normalization pass. Re-runs the live
/// Associative layer on the settled graph after remap-producing
/// cleanup passes have changed which already-built node an
/// operand points at.
pub fn flatten_posthoc_associative_gates(m: &mut Module) -> u32;

/// Post-drain endpoint-preserving state-sharing pass. Under
/// `identity_mode = node-id` with effective level `>= cse`,
/// merges flops with equal `width`, reset, and equal D-cone
/// meaning over the same canonical leaf endpoints.
/// Returns the number of duplicate flops removed
/// (`Metrics::flops_merged`).
pub fn merge_equivalent_flops(m: &mut Module) -> u32;

/// Post-construction BFS-reachability pass. Drops unreachable
/// gates, remaps every `NodeId` holder across `m.nodes` /
/// `m.drives` / `m.flops` / dedup tables. Called at the end of
/// `generate_leaf_module`. Returns the count of removed nodes
/// (surfaced via `Metrics::nodes_compacted`).
pub fn compact_node_ids(m: &mut Module) -> u32;

// Per-probability-roll telemetry:
pub enum KnobId { FlopProb, CombMuxProb, PriorityEncoderProb,
                  CaseMuxProb, CasezMuxProb, ForFoldProb,
                  CoefficientProb, ConstShiftAmountProb,
                  ConstComparandProb, ConstantProb,
                  TerminalReuseProb, CombMuxEncodingProb,
                  FlopMuxEncodingProb, ShareProb,
                  FlopQFeedbackProb, HierarchySiblingRouteProb,
                  HierarchyRegisteredSiblingRouteProb,
                  HierarchyRegisteredChildInputConeProb,
                  HierarchyChildInputConeProb,
                  HierarchyParentConeInstanceProb,
                  HierarchyParentFlopProb }
pub struct KnobRollCounters {
    pub attempts: HashMap<KnobId, u64>,
    pub fires:    HashMap<KnobId, u64>,
}

pub enum Node {
    PrimaryInput{..}, Constant{..}, FlopQ{..}, InstanceOutput{..}, Gate{..}
}
pub enum GateOp {
    And, Or, Xor, Not,              // bitwise (Not is unary)
    Add, Sub, Mul,                  // arithmetic
    Eq, Neq, Lt, Gt, Le, Ge,        // comparisons (1-bit output)
    Mux,                            // [sel, a, b]
    CaseMux, CasezMux,              // procedural combinational cases
    ForFold { kind, trip_count, chunk_width }, // bounded always_comb for
    Slice { hi: u32, lo: u32 },
    Concat,                         // variadic
    RedAnd, RedOr, RedXor,          // unary reductions (1-bit output)
    Shl, Shr,                       // [value, amount]
}
pub enum FlopKind { ZeroDefault, QFeedback }
pub enum FlopMux { None, OneHot(Vec<MuxArm>), Encoded { sel, data } }
pub struct Flop { id, width, d, q, reset_val, reset_kind, kind, mux }
pub struct DepSet(BTreeSet<u32>);

// config.rs
pub enum ConstructionStrategy { Sequential, Shuffled, Interleaved, GraphFirst }
pub enum IdentityMode { Relaxed, NodeId }
pub enum FactorizationLevel {
    None, Cse, OperandUnique, Commutative,
    Associative, ConstantFold, Peephole, EGraph,
}

// gen/mod.rs
pub struct Generator { rng: ChaCha8Rng, cfg: Config, ... }
impl Generator {
    pub fn new(cfg: Config) -> Self;
    pub fn generate_module(&mut self) -> Module;
    pub fn generate_design(&mut self) -> Design;   // depth-0 leaf, exact depth-1 wrapper, or bounded recursive hierarchy
}

// metrics.rs
pub struct Metrics { /* ~25 public fields; see module doc */ }
pub fn compute(m: &Module) -> Metrics;
pub struct DesignMetrics { /* hierarchy composition facts */ }
pub fn compute_design(d: &Design) -> DesignMetrics;

// emit/sv.rs
pub fn to_sv(m: &Module) -> String;
pub fn to_sv_in_design(m: &Module, design: &Design) -> String;
pub fn to_sv_design(design: &Design) -> String;

// lib.rs
pub fn set_trace_debug(enabled: bool);
pub fn trace_debug_enabled() -> bool;
#[macro_export] macro_rules! trace_verbose { ... }
```

## Testing strategy

Three layers:

**Unit tests** live inline in each source module under
`#[cfg(test)] mod tests { ... }`. Current counts:

- `src/ir/types.rs` — 40 tests covering factorization,
  identity-mode, rewrite-layer semantics, and design-aware
  control-port visibility.
- `src/ir/validate.rs` — 26 tests (valid modules, undefined drive
  roots, canonical flop/`FlopQ` backrefs, missing-D / mux-ref
  failures, representative gate-shape rejection classes, and
  design-level hierarchy acceptance/rejection).
- `src/gen/cone.rs` — 42 tests covering picker, anti-collapse,
  width-adapter, exact-selector `CaseMux` / `CasezMux` cleanup, and
  motif-edge cases.
- `src/emit/sv.rs` — 17 tests (module header, clk/rst_n omission,
  `always_ff` shape, operator + constant rendering, Slice/Concat,
  Mux ternary, procedural structured surfaces, and hierarchy
  control-port propagation across comb-only, direct-wrapper, and
  grandparent-wrapper cases).
- `src/metrics.rs` — 18 tests (empty module, per-kind gate
  counting, per-shape flop counting, variable-vs-constant shift-rhs,
  and hierarchy design metrics for reuse, under-instantiation,
  parent-side composition, sibling-routed child inputs,
  direct sibling helper routes, parent-cone helper output support,
  budgeted parent-cone helpers, registered helper-sourced child-input D
  cones, direct registered sibling helper routes, stateful
  parent-composed helper child-input routes, recursive tree shape,
  per-depth branching
  profiles, mixed-depth recursion, and profiled on-demand interface
  realization).
- Other unit tests cover compaction, config validation, module
  finalisation, hierarchy validation, and CLI overrides.

**Integration tests** in `tests/pipeline.rs` cover cross-seed
generation + validation across all strategy values,
byte-identical reproducibility, motif boundary cases, the full
live gate-category surface, the landed case/casez structured
surfaces, the landed bounded `for`-fold structured surface, the landed
selectable `Slice` / `Concat` surface, the hierarchy surface (legacy
depth-1 wrapper exact/reuse/under-instantiation plus the bounded
recursive tree planner, per-depth branching profiles, sibling-routed
child inputs, parent-composed child-input bindings, parent-cone
helper-instance child-input bindings, recursive non-top direct sibling
helper routes, parent-cone helper-instance
parent-output composition, stateful parent-output helper routing
through parent-local flops, stateful parent-composed helper child-input
routing through parent-local flops, local parent flops, registered sibling-routed
child-input bindings, registered parent-composed child-input bindings,
registered helper-sourced child-input D cones, direct sibling helper
routes, direct registered sibling helper routes,
budgeted parent-cone helper allocation, budgeted parent-output helper
composition,
mixed parent-port / child-output parent outputs, and module-name
uniqueness across batched hierarchy designs),
compaction/orphan guarantees, knob-roll telemetry, and input-surface
finalisation.

**Total (current HEAD, `cargo test` on 2026-04-30): 226 unit-target tests + 68 integration tests = 294 passing tests.**

**External smoke tests** — repo-owned downstream smoke now exists via
`src/bin/tool_matrix.rs`, which runs Verilator and Yosys across a
curated adversarial matrix and treats warnings as failures. The harness
now has an explicit Yosys mode axis too (`without-abc`, `with-abc`, or
`both`), so the stable no-ABC baseline and the explicit ABC-enabled
harness path can be tracked separately. The full current-code Phase 1
gate is now closed via
`/tmp/anvil-tool-matrix-phase1-real-r21/tool_matrix_report.json`
(1005 modules, `coverage_gaps = []`, and 1005/0 pass-fail in
Verilator plus both repo-owned Yosys modes). The explicit
`--phase1-gate` mode turned the old roadmap arithmetic into a real
repo-owned closure command. The dedicated Phase 2 sharing gate is now
closed too via `/tmp/anvil-tool-matrix-phase2-share-r1/tool_matrix_report.json`
(216 modules, `coverage_gaps = []`, and 216/0 pass-fail in Verilator
plus both repo-owned Yosys modes), with a normalized `share_sweep`
summary proving that `shared_node_fraction` rises monotonically across
`share_prob = 0.0`, `0.3`, and `0.9`. The dedicated Phase 3
structured-surface gate is now closed as well via
`/tmp/anvil-tool-matrix-phase3-structured-r4/tool_matrix_report.json`
(210 modules, `coverage_gaps = []`, and 210/0 pass-fail in Verilator
plus both repo-owned Yosys modes). The Phase 4 hierarchy slice now has
its latest full downstream-clean repo-owned gate via
`/tmp/anvil-tool-matrix-phase4-hierarchy-r83/tool_matrix_report.json`
(792 designs, `artifact_kind = "design"`, `coverage_gaps = []`, and
792/0 pass-fail in Verilator plus both repo-owned Yosys modes). That
report banks wrapper exact / reuse / under-instantiation, the current
representative recursive depth-2 profiles, the mixed recursive
depth-range profile `2:3`, the explicit child-sourcing modes
`library` and `on-demand`, exact profiled child-interface synthesis in
the on-demand lane, the per-depth override profile `0=4:4,1=2:2`, real
sibling-routed child inputs, real parent-side composition above
instance outputs, parent-composed child input bindings through
`hierarchy_child_input_cone_prob`, local parent flops through
`hierarchy_parent_flop_prob`, parent-cone helper-instance child-input
bindings through `hierarchy_parent_cone_instance_prob`, registered
sibling-routed child inputs through `hierarchy_registered_sibling_route_prob`,
direct registered sibling mixed-support bindings through
`hierarchy_registered_sibling_mixed_support_prob`, recursive non-top
direct registered sibling mixed-support bindings through the same parent
generation path,
registered parent-composed child-input bindings through
`hierarchy_registered_child_input_cone_prob`, registered mixed-support
child-input bindings, recursive non-top registered mixed-support
child-input bindings, multi-stage registered parent-composed
child-input bindings, recursive non-top multi-stage registered
parent-composed child-input bindings without helper instances,
multi-stage registered sibling-routed child-input
bindings, recursive non-top multi-stage registered sibling-routed
child-input bindings without helper instances, recursive non-top
multi-stage registered mixed-support child-input bindings without
helper instances, mixed parent-port / child-output parent outputs,
parent-output helper-instance composition, budgeted multi-helper
allocation, stateful parent-output helper routing through parent-local
flops, stateful parent-composed helper child-input routing through
parent-local flops, recursive non-top stateful parent-composed helper
child-input routing through parent-local flops, recursive non-top
direct sibling helper routing, recursive non-top direct registered
sibling helper routing, recursive non-top multi-stage direct registered
sibling helper routing, recursive non-top registered parent-composed
helper routing, recursive non-top multi-stage registered parent-composed
helper routing, recursive non-top parent-output helper routing,
recursive non-top stateful parent-output helper routing,
recursive non-top parent-output multi-helper budget evidence,
recursive non-top child-input multi-helper budget evidence,
recursive non-top stateful multi-helper budget evidence,
registered parent-composed helper-sourced child-input D cones,
recursive non-top registered parent-composed helper D-cone routing with
mixed parent-port support, direct sibling
helper routing, direct registered sibling helper routing, recursive
non-top direct registered sibling helper routing, multi-stage direct
registered sibling helper routing, and multi-stage registered
parent-composed helper routing and recursive non-top parent-output
helper routing that mixes parent-port support in the same output cone,
stateful helper-backed parent outputs that mix parent-port support,
unregistered helper-backed child-input bindings that mix parent-port
support, stateful helper-through-flop child-input bindings that mix
parent-port support, direct registered sibling mixed-support routes, recursive non-top
direct registered sibling mixed-support routes, and recursive non-top
unregistered parent-composed mixed-support child-input routes without
helper instances, and recursive non-top parent-port-composed parent-output
routes without helper instances or parent-local state, and recursive non-top stateful parent-port-composed parent-output routes without helper instances, and recursive non-top stateful unregistered parent-composed mixed-support child-input routes through parent-local Qs without helper instances at exact hierarchy depth 7 (2,2 calibrated) — closing the depth-7 sweep, and recursive non-top registered parent-composed child-input bindings that chain through at least three parent-local flop stages without helpers — opening a chain-depth axis above the closed depth-3..7 sweeps.
The `r83` report records
`saw_hierarchy_parent_composed_child_inputs = true`,
`saw_hierarchy_parent_local_flops = true`,
`saw_hierarchy_registered_sibling_routing = true`,
`saw_hierarchy_registered_sibling_mixed_support_routing = true`,
`saw_recursive_hierarchy_registered_sibling_mixed_support_routing = true`,
`saw_hierarchy_mixed_support_child_inputs = true`,
`saw_recursive_hierarchy_mixed_support_child_inputs = true`,
`saw_recursive_hierarchy_parent_port_composed_outputs = true`,
`saw_recursive_hierarchy_stateful_parent_port_composed_outputs = true`,
`saw_recursive_hierarchy_stateful_parent_composed_mixed_support_child_inputs = true`,
`saw_recursive_hierarchy_parent_local_flops = true`,
`saw_recursive_hierarchy_depth_3_parent_local_flops = true`,
`saw_recursive_hierarchy_depth_3_mixed_support_child_inputs = true`,
`saw_recursive_hierarchy_depth_3_parent_port_composed_outputs = true`,
`saw_recursive_hierarchy_depth_3_stateful_parent_port_composed_outputs = true`,
`saw_recursive_hierarchy_depth_3_stateful_parent_composed_mixed_support_child_inputs = true`,
`saw_recursive_hierarchy_depth_4_parent_local_flops = true`,
`saw_recursive_hierarchy_depth_4_mixed_support_child_inputs = true`,
`saw_recursive_hierarchy_depth_4_parent_port_composed_outputs = true`,
`saw_recursive_hierarchy_depth_4_stateful_parent_port_composed_outputs = true`,
`saw_recursive_hierarchy_depth_4_stateful_parent_composed_mixed_support_child_inputs = true`,
`saw_recursive_hierarchy_depth_5_parent_local_flops = true`,
`saw_recursive_hierarchy_depth_5_mixed_support_child_inputs = true`,
`saw_recursive_hierarchy_depth_5_parent_port_composed_outputs = true`,
`saw_recursive_hierarchy_depth_5_stateful_parent_port_composed_outputs = true`,
`saw_recursive_hierarchy_depth_5_stateful_parent_composed_mixed_support_child_inputs = true`,
`saw_recursive_hierarchy_depth_6_parent_local_flops = true`,
`saw_recursive_hierarchy_depth_6_mixed_support_child_inputs = true`,
`saw_recursive_hierarchy_depth_6_parent_port_composed_outputs = true`,
`saw_recursive_hierarchy_depth_6_stateful_parent_port_composed_outputs = true`,
`saw_recursive_hierarchy_depth_6_stateful_parent_composed_mixed_support_child_inputs = true`,
`saw_recursive_hierarchy_depth_7_parent_local_flops = true`,
`saw_recursive_hierarchy_depth_7_mixed_support_child_inputs = true`,
`saw_recursive_hierarchy_depth_7_parent_port_composed_outputs = true`,
`saw_recursive_hierarchy_depth_7_stateful_parent_port_composed_outputs = true`,
`saw_recursive_hierarchy_depth_7_stateful_parent_composed_mixed_support_child_inputs = true`,
`saw_recursive_hierarchy_three_stage_registered_parent_composed_chain = true`,
`saw_hierarchy_registered_parent_composed_routing = true`,
`saw_recursive_hierarchy_registered_parent_cone_instance_mixed_support_routing = true`,
`saw_hierarchy_registered_mixed_support_routing = true`,
`saw_recursive_hierarchy_registered_mixed_support_routing = true`,
`saw_hierarchy_registered_multistage_routing = true`,
`saw_recursive_hierarchy_registered_multistage_routing = true`,
`saw_recursive_hierarchy_registered_multistage_mixed_support_routing = true`,
`saw_hierarchy_registered_multistage_sibling_routing = true`,
`saw_recursive_hierarchy_registered_multistage_sibling_routing = true`,
`saw_hierarchy_registered_multistage_parent_cone_instance_routing = true`,
`saw_recursive_hierarchy_registered_multistage_parent_cone_instance_routing = true`,
`saw_hierarchy_registered_multistage_parent_composed_parent_cone_instance_routing = true`,
`saw_recursive_hierarchy_registered_multistage_parent_composed_parent_cone_instance_routing = true`,
`saw_hierarchy_parent_composed_parent_cone_instance_flop_routing = true`,
`saw_hierarchy_parent_port_composed_outputs = true`,
`saw_hierarchy_parent_cone_instance_routing = true`,
`saw_hierarchy_parent_cone_instance_outputs = true`,
`saw_recursive_hierarchy_parent_cone_instance_outputs = true`,
`saw_recursive_hierarchy_parent_cone_instance_mixed_support_outputs = true`,
`saw_recursive_hierarchy_parent_cone_instance_flop_outputs = true`,
`saw_hierarchy_parent_cone_instance_flop_outputs = true`,
`saw_recursive_multiple_parent_cone_instances_per_parent = true`,
`saw_recursive_multiple_parent_cone_instances_per_parent_child_inputs = true`,
`saw_recursive_multiple_parent_cone_instances_per_parent_through_flops = true`,
`saw_multiple_parent_cone_instances_per_parent = true`,
`saw_hierarchy_registered_parent_cone_instance_routing = true`,
`saw_hierarchy_direct_sibling_parent_cone_instance_routing = true`,
`saw_hierarchy_direct_registered_sibling_parent_cone_instance_routing = true`,
`saw_recursive_hierarchy_direct_sibling_parent_cone_instance_routing = true`,
`saw_recursive_hierarchy_direct_registered_sibling_parent_cone_instance_routing = true`,
`saw_recursive_hierarchy_registered_parent_composed_parent_cone_instance_routing = true`,
`saw_recursive_hierarchy_parent_composed_parent_cone_instance_flop_routing = true`,
`saw_hierarchy_parent_cone_instance_flop_mixed_support_outputs = true`,
`saw_recursive_hierarchy_parent_cone_instance_flop_mixed_support_outputs = true`,
`saw_hierarchy_parent_cone_instance_mixed_support_routing = true`,
`saw_recursive_hierarchy_parent_cone_instance_mixed_support_routing = true`,
`saw_hierarchy_parent_composed_parent_cone_instance_flop_mixed_support_routing = true`,
and
`saw_recursive_hierarchy_parent_composed_parent_cone_instance_flop_mixed_support_routing = true`.
It proves recursive non-top direct registered sibling mixed-support routing,
recursive non-top direct sibling helper routing, recursive
non-top direct registered sibling helper routing, recursive non-top
multi-stage direct registered sibling helper routing, recursive non-top
registered parent-composed helper routing, recursive non-top multi-stage
registered parent-composed helper routing, recursive non-top registered
parent-composed helper mixed-support routing, recursive non-top registered
mixed-support routing, and recursive non-top stateful
parent-composed helper child-input routing, plus recursive non-top
parent-output helper routing and recursive non-top stateful
parent-output helper routing, and recursive non-top parent-port-composed
parent-output routing without helpers or parent-local state, through the full
downstream tool bank. It
also proves recursive non-top parent-output multi-helper budget evidence,
recursive non-top child-input multi-helper budget evidence, and recursive
non-top stateful multi-helper budget evidence through the same full
downstream tool bank. It also proves recursive non-top multi-stage
registered parent-composed no-helper routing and recursive non-top
multi-stage registered sibling no-helper routing through the same full
downstream tool bank. The
earlier coverage-only proofs at
`/tmp/anvil-tool-matrix-phase4-recursive-direct-helper-r32/tool_matrix_report.json`
and
`/tmp/anvil-tool-matrix-phase4-recursive-helper-state-r31/tool_matrix_report.json`
are now historical policy breadcrumbs.
Earlier coverage-only probes at
`/tmp/anvil-tool-matrix-phase4-registered-mixed-r1/tool_matrix_report.json`,
`/tmp/anvil-tool-matrix-phase4-registered-multistage-r1/tool_matrix_report.json`,
`/tmp/anvil-tool-matrix-phase4-parent-port-coverage-r1/tool_matrix_report.json`,
and
`/tmp/anvil-tool-matrix-phase4-parent-cone-instance-r1/tool_matrix_report.json`
and
`/tmp/anvil-tool-matrix-phase4-parent-output-helper-state-r3/tool_matrix_report.json`
remain useful focused policy breadcrumbs, while the current full `r83`
bank carries those facts through Verilator and both repo-owned Yosys
modes. The old hierarchy smoke at
`/tmp/anvil-hierarchy-smoke-r1`
remains clean in Verilator, Yosys `synth -noabc`, and the repo-owned
ABC path. The focused clean proofs at `/tmp/anvil-hier-reuse-smoke-r1`,
`/tmp/anvil-hier-under-smoke-r2`,
`/tmp/anvil-hier-parent-compose-smoke-r1/manifest.json`,
`/tmp/anvil-hier-range-smoke-r1/manifest.json`, and
`/tmp/anvil-hier-depth-profile-smoke-r1/manifest.json`, and
`/tmp/anvil-hier-profiled-ondemand-smoke-r1/manifest.json`,
`/tmp/anvil-hier-registered-sibling-smoke-r1/manifest.json`, and
`/tmp/anvil-hier-registered-child-input-cone-smoke-r2/manifest.json`,
`/tmp/anvil-hier-parent-output-mix-smoke-r1/manifest.json`, and
`/tmp/anvil-hier-registered-mixed-child-input-smoke-r1/manifest.json`,
and
`/tmp/anvil-hier-registered-multistage-child-input-smoke-r1/manifest.json`,
and
`/tmp/anvil-parent-cone-instance-smoke-r1/manifest.json`, and
`cargo test hierarchy_sibling_routes_can_use_helper_instances`, and
`cargo test recursive_hierarchy_sibling_routes_can_use_helper_instances_below_top`, and
`cargo test hierarchy_registered_sibling_routes_can_use_helper_instances`, and
`cargo test hierarchy_registered_sibling_routes_can_chain_helper_instances_through_parent_flops`,
`cargo test recursive_hierarchy_registered_sibling_routes_can_chain_helper_instances_below_top`,
`cargo test recursive_hierarchy_registered_sibling_routes_can_mix_parent_port_support_below_top`,
`cargo test recursive_hierarchy_registered_sibling_routes_can_chain_without_helpers_below_top`,
`cargo test recursive_hierarchy_registered_parent_composed_routes_can_chain_helper_instances_below_top`,
`cargo test recursive_hierarchy_parent_outputs_can_depend_on_helper_instances_below_top`,
`cargo test recursive_hierarchy_parent_outputs_can_spend_helper_budget_below_top`,
`cargo test recursive_hierarchy_parent_cone_helper_budget_allows_multiple_helpers_below_top`,
`cargo test recursive_hierarchy_parent_outputs_can_spend_stateful_helper_budget_below_top`,
`cargo test recursive_hierarchy_parent_outputs_can_route_helper_instances_through_parent_flops_below_top`,
and
`cargo test hierarchy_parent_composed_helper_routes_can_use_parent_flops`
remain useful targeted evidence. The old `r7` report is now the historical
wrapper-baseline artifact, `r10` is the pre-on-demand mixed-depth bank,
`r11` is the first explicit child-sourcing bank, `r21` is historical
pre-parent-output-helper evidence, `r22` is the clean but insufficient
126-design budget-mismatch run, `r31` is the previous recursive
helper-state full bank, `r32` is root-cause evidence for exact-selector
`CaseMux` / `CasezMux` shift cleanup, `r37` is the previous recursive
non-top multi-stage direct registered helper bank, `r38` is the previous
recursive non-top multi-stage registered parent-composed helper bank,
`r39` is the previous recursive non-top parent-output helper bank, `r40`
is the previous recursive non-top stateful parent-output helper bank,
`r41` is the previous recursive non-top parent-output multi-helper budget bank, `r42`
is the previous recursive non-top stateful multi-helper budget bank, `r43` is the previous recursive non-top child-input multi-helper budget bank, `r44` is the previous recursive non-top registered mixed-support routing bank, `r45` is the previous recursive non-top multi-stage registered parent-composed no-helper bank, `r46` is the previous recursive non-top multi-stage registered sibling no-helper bank, `r47` is the previous recursive non-top multi-stage registered mixed-support no-helper bank, `r48` is the previous recursive non-top registered parent-composed helper mixed-support bank, `r49` is the previous recursive non-top parent-output helper mixed-support bank, `r50` is the previous accumulated mixed-support hierarchy full bank, `r51` is the previous direct registered sibling mixed-support hierarchy full bank, `r52` is the previous recursive direct registered sibling mixed-support hierarchy full bank, `r53` is the previous recursive parent-composed mixed-support child-input hierarchy full bank, `r54` is the previous recursive parent-port-composed parent-output hierarchy full bank, `r55` is the previous recursive stateful parent-port-composed parent-output hierarchy full bank, `r56` is the previous recursive stateful unregistered parent-composed mixed-support child-input hierarchy full bank, `r57` is the previous hierarchy full bank that gated recursive non-top parent-local flops as a first-class coverage fact, `r58` is the previous hierarchy full bank that pushed recursive parent-local flops to exact hierarchy depth 3, `r59` is the previous hierarchy full bank that pushed recursive non-top unregistered parent-composed mixed-support child inputs to exact hierarchy depth 3 without helpers, `r60` is the previous hierarchy full bank that pushed recursive non-top parent-port-composed parent outputs to exact hierarchy depth 3 without helpers or state, `r61` is the previous hierarchy full bank that pushed recursive non-top stateful parent-port-composed parent outputs to exact hierarchy depth 3 without helpers, `r62` is the previous hierarchy full bank that closed the depth-3 push with recursive non-top stateful parent-composed mixed-support child inputs at exact hierarchy depth 3 without helpers, `r63` is the previous hierarchy full bank that opened the depth-4 axis with recursive non-top parent-local flops at exact hierarchy depth 4, `r64` is the previous hierarchy full bank that extended the depth-4 axis with recursive non-top mixed-support child inputs at exact hierarchy depth 4 without helpers, `r65` is the previous hierarchy full bank that extended the depth-4 axis with recursive non-top parent-port-composed parent outputs at exact hierarchy depth 4 without helpers or state, `r66` is the previous hierarchy full bank that extended the depth-4 axis with recursive non-top stateful parent-port-composed parent outputs at exact hierarchy depth 4 without helpers, `r67` is the previous hierarchy full bank that closed the depth-4 sweep with recursive non-top stateful parent-composed mixed-support child inputs at exact hierarchy depth 4 without helpers, `r68` is the previous hierarchy full bank that opened the depth-5 axis with recursive non-top parent-local flops at exact hierarchy depth 5, `r69` is the previous hierarchy full bank that extended the depth-5 axis with recursive non-top mixed-support child inputs at exact hierarchy depth 5 without helpers, `r70` is the previous hierarchy full bank that extended the depth-5 axis with recursive non-top parent-port-composed parent outputs at exact hierarchy depth 5 without helpers or state, `r71` is the previous hierarchy full bank that extended the depth-5 axis with recursive non-top stateful parent-port-composed parent outputs at exact hierarchy depth 5 without helpers, `r72` is the previous hierarchy full bank that closed the depth-5 sweep with recursive non-top stateful unregistered parent-composed mixed-support child inputs at exact hierarchy depth 5 without helpers, `r73` is the previous hierarchy full bank that opened the depth-6 axis with recursive non-top parent-local flops at exact hierarchy depth 6, `r74` is the previous hierarchy full bank that extended the depth-6 axis with recursive non-top mixed-support child inputs at exact hierarchy depth 6 without helpers (2,2 calibrated), `r75` is the previous hierarchy full bank that extended the depth-6 axis with recursive non-top parent-port-composed parent outputs at exact hierarchy depth 6 without helpers or state, `r76` is the previous hierarchy full bank that extended the depth-6 axis with recursive non-top stateful parent-port-composed parent outputs at exact hierarchy depth 6 without helpers, `r77` is the previous hierarchy full bank that closed the depth-6 sweep with recursive non-top stateful unregistered parent-composed mixed-support child inputs at exact hierarchy depth 6 without helpers (2,2 calibrated), `r78` is the previous hierarchy full bank that opened the depth-7 axis with recursive non-top parent-local flops at exact hierarchy depth 7, `r79` is the previous hierarchy full bank that extended the depth-7 axis with recursive non-top mixed-support child inputs at exact hierarchy depth 7 without helpers (2,2 calibrated), `r80` is the previous hierarchy full bank that extended the depth-7 axis with recursive non-top parent-port-composed parent outputs at exact hierarchy depth 7 without helpers or state, `r81` is the previous hierarchy full bank that extended the depth-7 axis with recursive non-top stateful parent-port-composed parent outputs at exact hierarchy depth 7 without helpers, `r82` is the previous hierarchy full bank that closed the depth-7 sweep with recursive non-top stateful unregistered parent-composed mixed-support child inputs at exact hierarchy depth 7 without helpers (2,2 calibrated), `r83` is the current hierarchy full bank that opens a chain-depth axis above the closed depth-3..7 sweeps with recursive non-top registered parent-composed three-stage chain coverage, and
the aborted `r8` rerun remains
useful as evidence that the Phase 4 gate should use a hierarchy-focused
sequential leaf profile rather than silently borrowing the fattest
Phase 1 leaf-stress shape.

## Error handling

`anvil` should not fail silently or on valid configurations. The
error taxonomy:

- `ConfigError` — invalid knobs (e.g., `min_width > max_width`,
  `min_mux_arms > max_mux_arms`, out-of-range probability). Caught
  at `Config::validate()` before any generation begins.
- `ValidateError` — IR invariant violation (undefined drive root,
  canonical flop/`FlopQ` mismatch, per-gate arity/width, missing
  flop D, empty-dep-set output, etc.). Treated as a generator bug —
  if real generator output produces this, the generator is wrong.
- `IoError` — failed to write output file. Surfaced to the user.

The generator never produces invalid IR. If it does, that's a
generator bug, not a recoverable error.

## CLI

Every motif knob has a dedicated flag across structure /
sequential / sharing / operator-arity / coefficient / shift /
comparand / blocks / construction-strategy / identity/factorization,
plus the run-control flags `--seed`, `--count`, `--out`, `--config`,
`--dump-config`, `--trace`, `--trace-file`, `--metrics`.

The full categorised list lives in
[Knobs and Reproducibility — CLI coverage](knobs.md#cli-coverage);
`anvil --help` is the canonical source of truth.

Piping stdout is valid for `count = 1` (no directory required). For
`count > 1`, `--out` is required so that per-module files and the
manifest have somewhere to go.
