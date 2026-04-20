# Architecture of the Rust Implementation

## Crate layout

```
src/
├── main.rs          # CLI entry point (clap-derived); covers every
│                    # motif knob as a dedicated flag; wires the
│                    # tracing-subscriber from --trace <level> and
│                    # --trace-file.
├── lib.rs           # public API, re-exports Config, Generator, Module.
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
│   │                # KnobId, KnobRollCounters, Design. Module
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
│   └── validate.rs  # invariant + canonical-state + per-gate shape checker; inline unit tests.
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
│   └── pool.rs      # SignalPool (width-indexed, cloneable for rewind).
└── emit/
    ├── mod.rs       # re-exports.
    └── sv.rs        # IR -> SystemVerilog. Dumb serialiser per doctrine —
                     # no filtering, no reachability checks. build_names
                     # assigns each gate a <kind>_<N> name (Rule 12);
                     # flops are flop_<id>. Inline unit tests.
```

Phase 4 (hierarchy) will add `src/gen/hierarchy.rs`; it does not exist
yet.

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
  Verilator/Yosys, and write an aggregated report.

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
   `src/gen/module.rs`. Richer structured ops, hierarchy,
   parameterization, aggregates, memories, and FSMs are future work on
   top of this base, not evidence against it.
2. **`NodeId` as identity**
   Full factorization is only partially realized today. Combinational
   identity flows through `Module::intern_gate`, a bounded semantic
   gate-merge fragment now lives at the `e-graph` rung after
   construction, and endpoint-aware duplicate flops merge after drain,
   but stronger state equivalence and future hierarchical identity are
   not finished.
3. **Tool-clean industrialization**
   Internal tests are strong, and a repo-owned `tool_matrix` harness
   now exists, and its current smoke matrix is green: 15/15 clean in
   Verilator and 15/15 clean in Yosys. The harness now treats warnings
   as failures, so "green" means no errors and no warnings. The path to
   that state needed both construction-time comparison proofs and a
   post-construction exact-value cleanup pass on the settled graph.
   That evidence layer must keep growing with each new motif family
   until the larger phase-exit sweeps are equally boring.
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
                  CoefficientProb, ConstShiftAmountProb,
                  ConstComparandProb, CombMuxEncodingProb,
                  FlopMuxEncodingProb, ShareProb,
                  FlopQFeedbackProb }
pub struct KnobRollCounters {
    pub attempts: HashMap<KnobId, u64>,
    pub fires:    HashMap<KnobId, u64>,
}

pub enum Node { PrimaryInput{..}, Constant{..}, FlopQ{..}, Gate{..} }
pub enum GateOp {
    And, Or, Xor, Not,              // bitwise (Not is unary)
    Add, Sub, Mul,                  // arithmetic
    Eq, Neq, Lt, Gt, Le, Ge,        // comparisons (1-bit output)
    Mux,                            // [sel, a, b]
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
    pub fn generate_design(&mut self) -> Design;   // Phase 4+ stub
}

// metrics.rs
pub struct Metrics { /* ~25 public fields; see module doc */ }
pub fn compute(m: &Module) -> Metrics;

// emit/sv.rs
pub fn to_sv(m: &Module) -> String;

// lib.rs
pub fn set_trace_debug(enabled: bool);
pub fn trace_debug_enabled() -> bool;
#[macro_export] macro_rules! trace_verbose { ... }
```

## Testing strategy

Three layers:

**Unit tests** live inline in each source module under
`#[cfg(test)] mod tests { ... }`. Current counts:

- `src/ir/types.rs` — 36 tests covering factorization,
  identity-mode, and rewrite-layer semantics.
- `src/ir/validate.rs` — 21 tests (valid modules, undefined drive
  roots, canonical flop/`FlopQ` backrefs, missing-D / mux-ref
  failures, and representative gate-shape rejection classes).
- `src/gen/cone.rs` — 18 tests covering picker, anti-collapse,
  width-adapter, and motif-edge cases.
- `src/emit/sv.rs` — 6 tests (module header, clk/rst_n omission,
  `always_ff` shape, operator + constant rendering, Slice/Concat,
  Mux ternary).
- `src/metrics.rs` — 3 tests (empty module, per-kind gate
  counting, per-shape flop counting).
- Other unit tests cover compaction, config validation, module
  finalisation, and CLI overrides.

**Integration tests** in `tests/pipeline.rs` cover cross-seed
generation + validation across all strategy values,
byte-identical reproducibility, motif boundary cases, the full
live gate-category surface, compaction/orphan guarantees, knob-roll
telemetry, and input-surface finalisation.

**Total (current HEAD, `cargo test` on 2026-04-20): 116 unit + 24 integration = 140 passing tests.**

**External smoke tests** — repo-owned downstream smoke now exists via
`src/bin/tool_matrix.rs`, which runs Verilator and Yosys across a
curated adversarial matrix and treats warnings as failures. Scaling
that green smoke matrix up is part of the remaining Phase 1 / Phase 2
exit work.

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
