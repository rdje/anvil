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
│                    # for Interleaved). FactorizationLevel enum along
│                    # the chain none → cse → operand-unique →
│                    # commutative → associative → constant-fold →
│                    # peephole → e-graph (default; effective() clamps
│                    # aspirational levels to the highest implemented).
├── ir/
│   ├── mod.rs       # re-exports.
│   ├── types.rs     # Module, Port, Node, GateOp (with Hash derive),
│   │                # Flop, FlopKind, FlopMux, MuxArm, DepSet,
│   │                # KnobId, KnobRollCounters, Design. Module
│   │                # carries construction-time dedup tables
│   │                # (gate_instances, const_instances), per-module
│   │                # knob mirrors (max_ast_instances,
│   │                # mux_arm_duplication_rate,
│   │                # operand_duplication_rate, factorization_level),
│   │                # and live counters (fold_identities_applied,
│   │                # peephole_rewrites_applied,
│   │                # flatten_associative_applied, nodes_compacted,
│   │                # block-build counters, knob_rolls).
│   │                # API: intern_gate() runs the full factorization
│   │                # ladder (flatten_associative → commutative sort →
│   │                # fold_constants → apply_peephole → CSE dedup)
│   │                # and returns (NodeId, is_new). intern_constant()
│   │                # is the constant analogue. Inline unit tests
│   │                # pin each layer's contract.
│   ├── compact.rs   # Post-construction compact_node_ids pass: BFS
│   │                # from roots, drops unreachable gates, remaps
│   │                # NodeIds across m.nodes / m.drives / m.flops /
│   │                # dedup tables. Enables orphan-producing
│   │                # rewrites (Not(Not), Associative flattening,
│   │                # Not(cmp) inversion) to stay Rule-18-clean at
│   │                # module finalisation. Inline unit tests.
│   └── validate.rs  # invariant + per-gate shape checker; inline unit tests.
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

This means `ir` can be tested in isolation, `emit` can be tested with
hand-constructed IRs (no need to invoke the generator), and `gen` can
be tested by inspecting the IR it produces without ever emitting SV.

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
    pub factorization_level:      FactorizationLevel,
    // Block-build live counters:
    pub priority_encoder_built:  u32,
    pub comb_mux_one_hot_built:  u32,
    pub comb_mux_encoded_built:  u32,
    // Factorization-layer live counters:
    pub fold_identities_applied:     u64,
    pub peephole_rewrites_applied:   u64,
    pub flatten_associative_applied: u64,
    pub nodes_compacted:             u32,
    // Per-knob probability-roll counters:
    pub knob_rolls:                  KnobRollCounters,
}
impl Module {
    /// Single chokepoint for gate creation. Runs the full
    /// factorization ladder in order: associative flattening →
    /// commutative sort → constant fold → peephole → CSE. See
    /// `book/src/factorization.md` for the layer-by-layer view.
    pub fn intern_gate(&mut self, op, operands, width, deps) -> (NodeId, bool);
    pub fn intern_constant(&mut self, width, value) -> (NodeId, bool);

    // (Layer helpers, `pub(crate)`):
    //   fn flatten_associative(&mut self, op, operands, width) -> Option<(NodeId, bool)>;
    //   fn fold_constants     (&mut self, op, operands, width) -> Option<(NodeId, bool)>;
    //   fn apply_peephole     (&mut self, op, operands, width) -> Option<(NodeId, bool)>;
}

// ir/compact.rs
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

- `src/ir/types.rs` — 2 tests (commutative normalization +
  non-commutative preservation).
- `src/ir/validate.rs` — 8 tests (valid modules + each rejection
  class).
- `src/gen/cone.rs` — 13 tests (`ceil_log2`, `pick_mux_arm_count`,
  `make_width_adapter` edge cases, DAG-sharing sanity, four
  flop-assembler shapes, N-arity anti-collapse, dep-bearing
  terminal picker, coefficient-width clamping).
- `src/emit/sv.rs` — 6 tests (module header, clk/rst_n omission,
  `always_ff` shape, operator + constant rendering, Slice/Concat,
  Mux ternary).
- `src/metrics.rs` — 3 tests (empty module, per-kind gate
  counting, per-shape flop counting).
- Other unit tests total a few more; actual unit count fluctuates
  as slices land. Run `cargo test --lib` for the current number.

**Integration tests** in `tests/pipeline.rs` — 15 tests covering
cross-seed generation + validation across all four strategy
values, byte-identical reproducibility, coefficient / shift /
comparand motifs at boundary rates, and priority-encoder shape.

**Total (current HEAD): 39 unit + 15 integration = 54 tests, all passing.**

**External smoke tests** (not wired up yet) — will invoke Verilator
and Yosys against generated output. These are the remaining Phase 1
and Phase 2 exit gates.

## Error handling

`anvil` should not fail silently or on valid configurations. The
error taxonomy:

- `ConfigError` — invalid knobs (e.g., `min_width > max_width`,
  `min_mux_arms > max_mux_arms`, out-of-range probability). Caught
  at `Config::validate()` before any generation begins.
- `ValidateError` — IR invariant violation (per-gate arity, per-gate
  width, missing flop D, empty-dep-set output, etc.). Treated as a
  generator bug — if real generator output produces this, the
  generator is wrong.
- `IoError` — failed to write output file. Surfaced to the user.

The generator never produces invalid IR. If it does, that's a
generator bug, not a recoverable error.

## CLI

Every motif knob has a dedicated flag (44 total across structure /
sequential / sharing / operator-arity / coefficient / shift /
comparand / blocks / construction-strategy / factorization-ladder,
plus the run-control flags `--seed`, `--count`, `--out`, `--config`,
`--dump-config`, `--trace`, `--trace-file`, `--metrics`).

The full categorised list lives in
[Knobs and Reproducibility — CLI coverage](knobs.md#cli-coverage);
`anvil --help` is the canonical source of truth.

Piping stdout is valid for `count = 1` (no directory required). For
`count > 1`, `--out` is required so that per-module files and the
manifest have somewhere to go.
