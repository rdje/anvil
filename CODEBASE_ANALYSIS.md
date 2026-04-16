# Code Base Analysis
Live analysis of the Rust workspace as it currently stands. Updated whenever a slice materially changes the workspace.

## Snapshot
- **Workspace:** single crate `anvil` (no Cargo workspace; flat layout).
- **Edition:** 2021.
- **Targets:** one binary (`anvil`), one library (`anvil`), one example (`generate_one`), one integration test (`pipeline`).
- **External deps:** `rand`, `rand_chacha`, `clap`, `serde`, `serde_json`, `thiserror`, `anyhow`, `tracing`, `tracing-subscriber`. `insta` (dev) reserved for snapshot tests. `tracing` carries `release_max_level_info` so trace-level calls compile out in release.
- **MSRV:** not yet pinned. Whatever stable Rust is current.

## Module map

```
src/
├── main.rs           CLI entry point. Parses Cli (clap), constructs Config,
│                     runs Generator, writes stdout or per-file output with
│                     manifest.json. Owns no domain logic.
│                     CLI flags cover every Phase 1/2 motif knob:
│                     structure (min/max-inputs/outputs/width, max-depth),
│                     sequential (flop-prob, max-flops-per-module,
│                     min/max-mux-arms, flop-qfeedback-prob,
│                     flop-mux-encoding-prob), sharing (share-prob).
│                     Tracing: init_tracing wires a deterministic
│                     tracing-subscriber from --trace <level> +
│                     --trace-file; TraceLevel maps low=INFO,
│                     medium=DEBUG, high/debug=TRACE. Output to
│                     stderr or file; stdout stays byte-clean.
│
├── lib.rs            Public surface: re-exports Config, Generator, Module.
│                     Also exposes the `metrics` module. Trace
│                     infrastructure: static TRACE_DEBUG: AtomicBool,
│                     set_trace_debug(bool), trace_debug_enabled(),
│                     and the `trace_verbose!` macro (exported) which
│                     gates tracing::trace! calls behind the debug
│                     flag so --trace debug is strictly more verbose
│                     than --trace high.
│
├── metrics.rs        Post-hoc structural metrics. compute(&Module) →
│                     Metrics { size, gates_by_kind, constants_by_width,
│                     mux/concat shape counts, fanout stats, flop
│                     distribution, AST-instance saturation,
│                     gate_operand_count_histogram +
│                     max_gate_operand_count +
│                     max_operand_count_by_kind, gate_depth_histogram
│                     + max_gate_depth, num_priority_encoder_blocks,
│                     num_comb_muxes_one_hot + num_comb_muxes_encoded,
│                     nested_associative_operand_count }.
│                     Serde-serializable; embedded in
│                     manifest.json and printed with --metrics flag.
│
├── config.rs         Config struct (knobs), Default impl, validate(),
│                     CLI Overrides struct, ConfigError taxonomy.
│                     ConstructionStrategy enum (clap::ValueEnum +
│                     serde): Sequential, Shuffled, Interleaved
│                     (default). GraphFirst variant retained as a
│                     silent alias for Interleaved — the original
│                     speculative pool-growth strategy was retired
│                     for producing Rule 18 violations.
│                     FactorizationLevel enum (derives
│                     PartialOrd/Ord): None, Cse, OperandUnique,
│                     Commutative, Associative, ConstantFold,
│                     Peephole, EGraph (default). effective()
│                     clamps to the highest implemented layer
│                     (currently Commutative). Fine-grained knobs:
│                     max_ast_instances, mux_arm_duplication_rate,
│                     operand_duplication_rate, factorization_level.
│
├── ir/
│   ├── mod.rs        Re-exports types::* and the validate module.
│   ├── types.rs      Core types: Module, Port, Direction, Node, GateOp,
│   │                 Flop, ResetKind, DepSet, Design. GateOp derives
│   │                 Hash (needed as dedup-table key).
│   │                 Node kinds: PrimaryInput/Constant/FlopQ/Gate.
│   │                 Module gains construction-time dedup tables:
│   │                 gate_instances: HashMap<(GateOp, Vec<NodeId>,
│   │                 u32), Vec<NodeId>>, const_instances:
│   │                 HashMap<(u32, u128), Vec<NodeId>>, plus per-
│   │                 module knob mirrors: max_ast_instances,
│   │                 mux_arm_duplication_rate,
│   │                 operand_duplication_rate, factorization_level.
│   │                 API: intern_gate(op, operands, width, deps) →
│   │                 (NodeId, is_new) and intern_constant(width,
│   │                 value) → (NodeId, is_new). intern_gate performs
│   │                 commutative sort on And/Or/Xor/Add/Mul when
│   │                 factorization_level ≥ Commutative; bypasses the
│   │                 dedup path entirely at level None; otherwise
│   │                 caps instances per AST key at max_ast_instances
│   │                 (default 1 = strict CSE). Both methods emit
│   │                 `trace_verbose!` 🔗 new / ♻️ reuse events.
│   └── validate.rs   Module invariant checker: operand defined,
│                     drive count == 1, flop D filled, dep-set non-empty,
│                     per-gate arity + operand-width + output-width rules
│                     for every GateOp variant. Has inline unit tests
│                     covering valid and invalid hand-built IRs.
│
├── gen/
│   ├── mod.rs        Generator struct (rng + cfg), generate_module(),
│   │                 generate_design() (Phase 5+ stub).
│   ├── module.rs     Leaf-module top-level generator: pick port counts,
│   │                 pick widths, seed signal pool with primary inputs,
│   │                 build a cone per primary output. Dispatches on
│   │                 cfg.construction_strategy: Sequential/Shuffled
│   │                 use the recursive build_cone_with_retry path;
│   │                 Interleaved (default) + the deprecated
│   │                 GraphFirst alias both delegate to
│   │                 cone::build_outputs_interleaved (frame machine).
│   │                 Drives recorded in declaration order regardless.
│   │                 After flop drain runs count_orphan_gates(m) as
│   │                 a Rule 18 safety-net audit — a non-zero count
│   │                 emits tracing::warn! (expected to be 0; current
│   │                 measurements confirm it).
│   ├── cone.rs       Fanin-cone recursion + interleaved frame machine.
│   │                 Public: FlopWorklist alias, build_cone_with_retry,
│   │                 drain_flop_worklist, build_cone.
│   │                 build_cone branches: flop block (build_flop_leaf),
│   │                 comb-mux block (build_comb_mux / *_one_hot /
│   │                 *_encoded), operator gate (pick_gate +
│   │                 input_widths_for). Both block branches pick
│   │                 style and arms via the shared min/max_mux_arms
│   │                 knob.
│   │                 Per-flop drain: drain_flop_one_hot, drain_flop_encoded.
│   │                 Helpers: build_flop_leaf, pick_reset_value,
│   │                 pick_mux_arm_count (M ∈ {0, 2..=max}),
│   │                 ceil_log2, assemble_flop_d_one_hot, assemble_flop_d_encoded,
│   │                 make_constant, make_eq_const, make_mux,
│   │                 replicate_to_width, make_and,
│   │                 make_none_selected, or_reduce_terms,
│   │                 try_share (DAG-sharing operand picker),
│   │                 pick_terminal (with lazy width-adapter fallback
│   │                 and exclusion filter), make_width_adapter, pick_gate,
│   │                 input_widths_for, violates_anti_collapse, node_deps.
│   │                 Q is a leaf in the current cone; D opens either
│   │                 a direct cone (M=0), a one-hot OR-of-masks mux
│   │                 (M>=2, OneHot), or a chained-ternary encoded
│   │                 mux (M>=2, Encoded) via the worklist. Comb muxes
│   │                 use the same two shapes minus any Q-feedback term.
│   │                 DAG sharing: per-operand `share_prob` decides
│   │                 share-vs-recurse; internal gates enter the pool
│   │                 as they are built.
│   │                 Interleaved strategy: build_outputs_interleaved
│   │                 + process_signal_frame + deliver with a
│   │                 SignalFrame queue and a GateFrame in-flight
│   │                 table. Gates finalize when their last operand
│   │                 resolves. Blocks (flop, comb-mux) still build
│   │                 synchronously within one frame step.
│   │                 GraphFirst strategy: retired. The CLI variant
│   │                 is routed to Interleaved. Original phase-1
│   │                 speculative pool growth produced 13–27 %
│   │                 orphan gates per module (Rule 18 violation);
│   │                 the dedicated code path (build_graph_first,
│   │                 grow_pool_one_unit, *_pool_only helpers) is
│   │                 currently dead and may be removed in a future
│   │                 cleanup slice.
│   │                 build_cone snapshot/rollback: before operand
│   │                 construction, build_cone snapshots m.nodes,
│   │                 m.flops, pool, worklist, gate_instances, and
│   │                 const_instances. On anti-collapse rejection the
│   │                 snapshot is fully restored — operand sub-trees
│   │                 built speculatively never become orphans.
│   │                 process_signal_frame anti-collapse fallback:
│   │                 the interleaved frame machine can't snapshot
│   │                 per-gate (siblings committed already) so it
│   │                 reuses one of the existing operands as the
│   │                 result NodeId instead of calling pick_terminal
│   │                 (which would create a fresh orphan-prone node).
│   │                 Dep-bearing terminal picker:
│   │                 pick_terminal_dep_bearing(g, m, pool, width,
│   │                 exclude) — returns only a dep-bearing matching-
│   │                 width pool entry or a dep-bearing width-adapter.
│   │                 Panics if the pool has no dep-bearing entry
│   │                 (invariant violation). Used at mux selects,
│   │                 priority-encoder request bits, const-comparand
│   │                 LHS, const-shift value operand.
│   │                 Signal-duplication helpers for N-to-1 mux arms
│   │                 (pick_datas_with_dup_cap, honours
│   │                 mux_arm_duplication_rate) and for linear-
│   │                 combination operand lists
│   │                 (pick_signals_with_dup_rate, honours
│   │                 operand_duplication_rate).
│   │                 Coefficient motif: when pick_gate returns
│   │                 Add/Sub/Mul and coefficient_prob fires,
│   │                 build_linear_combination_{recursive,pool}
│   │                 assembles a compound tree via
│   │                 assemble_add_linear_combination /
│   │                 assemble_sub_linear_combination /
│   │                 assemble_mul_linear_combination.
│   │                 Constant shift-amount motif: when pick_gate
│   │                 returns Shl/Shr and const_shift_amount_prob
│   │                 fires, build_shift_const_amount emits
│   │                 `value OP const` with a pick_shift_amount
│   │                 literal clamped to [0, W-1].
│   │                 Shl/Shr added to pick_gate's new shifts bucket
│   │                 (weight gate_shift_weight, default 1);
│   │                 disabled at target_width == 1.
│   │                 Constant comparand motif: when pick_gate returns
│   │                 a comparison op (Eq/Neq/Lt/Gt/Le/Ge) and
│   │                 const_comparand_prob fires,
│   │                 build_comparison_const_comparand emits
│   │                 `lhs_signal OP const` — LHS from the usual path,
│   │                 RHS a literal drawn from
│   │                 [min_comparand, max_comparand] clamped to
│   │                 [0, 2^K-1]. Additive to signal-vs-signal
│   │                 comparisons.
│   │                 Priority-encoder block: when
│   │                 priority_encoder_prob fires at a compatible
│   │                 target width (ceil_log2(N) == W for some N in
│   │                 [min_mux_arms, max_mux_arms]), emits a chained
│   │                 ternary over N 1-bit requests:
│   │                 req_0 ? 0 : req_1 ? 1 : ... : 0. Skipped (fall
│   │                 through to gate path) when no compatible N.
│   └── pool.rs       SignalPool: list of (node, width, deps) entries.
│                     Methods: add, of_width, iter, is_empty.
│                     Cloneable for snapshot/rewind during retry.
│
└── emit/
    ├── mod.rs        Re-exports to_sv.
    └── sv.rs         IR → String pretty-printer. Assumes invariants hold.
                      No validation. Fixed 4-space indent. Naming:
                      build_names walks m.nodes once, assigns each
                      Gate node a `<kind>_<per-kind-counter>` name
                      (and_0, mux_3, etc.); flops are flop_<id>;
                      non-gate nodes resolve via node_ref. See Rule 12
                      in book/src/structural-rules.md.
```

## Dependency direction
```
main  →  lib  →  gen  →  ir
                  │       ↑
                  ↓       │
                 emit ────┘
```

`ir` is a leaf. `gen` and `emit` both depend on `ir` but not on each other. This permits independent unit-testing of `emit` against hand-built IRs.

## Phase coverage map

| Phase | Status        | Code touched | Notes |
|-------|---------------|--------------|-------|
| 0 — Scaffolding              | done         | All files (initial) | `cargo check`, `cargo test`, `cargo clippy -D warnings`, `cargo fmt --check` all clean locally. |
| 1 — Single-module MVP        | mostly done  | `gen/cone.rs`, `gen/module.rs`, `emit/sv.rs`, `gen/pool.rs`, `ir/types.rs`, `metrics.rs` | Combinational + sequential cone recursion functional; flop worklist drained; `always_ff` emitted; single CLK + single RST_N (async). 22 structural rules enforced (Rules 1-22). Zero orphans (Rule 18 α). Zero duplicate operands at default knobs. Full factorization through Commutative layer. Remaining: Verilator/Yosys smoke (blocked — tools missing locally). |
| 2 — Sharing                  | in progress  | `gen/cone.rs`, `ir/types.rs` | Per-operand `share_prob` hook wired; internal gates enter the pool as they are built. Construction-time CSE (Rule 21) + operand-uniqueness (Rule 8 extended) + commutative normalization (Rule 21b) all enforced via intern_gate. factorization_level dial (default EGraph, clamps to Commutative today). |
| 3 — Structured combinational | in progress  | `gen/cone.rs` | Priority-encoder block (Rule 17) landed. case/casez, reductions, variable-shifts still not started. |
| 4 — Hierarchy                | not started  | new `gen/hierarchy.rs`; `Design` already typed | Library + on-demand sourcing. |
| 5 — Parameterization         | not started  | new module | Significant extension to IR (parameter env). |
| 6 — Advanced motifs          | not started  | various | Memories, FSMs, optional multi-clock. |

## Invariants currently enforced

In code (constructors / generator):
- `Module::intern_gate` / `intern_constant` enforce construction-time CSE (Rule 21): `(op, operands, width)` / `(width, value)` is the AST key; each key maps to a `Vec<NodeId>` capped at `max_ast_instances`. Commutative ops (`And`/`Or`/`Xor`/`Add`/`Mul`) have their operands sorted before key construction at factorization_level ≥ Commutative (Rule 21b). `FactorizationLevel::None` bypasses dedup entirely; `effective()` clamps aspirational levels to the highest implemented.
- `Config::validate()` rejects out-of-range knobs.
- `Generator::new()` seeds RNG deterministically.
- `gen::module::generate_leaf_module` produces port counts within knob ranges.
- `gen::cone::build_cone_with_retry` retries up to 4× on empty-dep-set cone roots; snapshots `m.nodes`, `m.flops`, pool, worklist, `gate_instances`, `const_instances` before each attempt and restores on empty-dep retry.
- `gen::cone::build_cone` snapshots the same state before operand construction. On anti-collapse rejection, restores the snapshot and returns `pick_terminal` as fallback. No orphan leaks from rejected recursive gates.
- `gen::cone::process_signal_frame` (interleaved) uses an existing operand as anti-collapse fallback (not `pick_terminal`) because per-gate snapshot is infeasible once sibling frames have committed.
- `gen::module::generate_leaf_module` runs `count_orphan_gates(m)` after flop drain as a Rule 18 safety-net audit; warns via `tracing::warn!` on non-zero orphan count.
- `gen::cone::pick_terminal` prefers matching-width pool entries with non-empty deps; on no width-match, builds a width-adapter (`make_width_adapter`) from the widest dep-bearing pool entry; only emits a constant when the entire pool has empty deps.
- `gen::cone::build_cone` consults `cfg.share_prob` per operand: with that probability it calls `try_share` to return an existing matching-width pool entry (with deps, honoring `exclude`); otherwise it recurses. Fresh `Gate` nodes enter the pool on creation, so later operand decisions in the same call chain can share them.
- `gen::cone::make_width_adapter` produces a Slice (when source > target), a single Concat (when source × N == target), or Concat-then-Slice (when source × N > target). Deps propagate from the source.
- `gen::cone::violates_anti_collapse` rejects (a) any duplicate operand in N-arity `And`/`Or`/`Xor` via `has_duplicate_operand` (O(N²), N bounded by `max_gate_arity`); (b) `x - x`, `x == x`, `x != x` at 2-arity; (c) `mux(s, a, a)`. `Add` and `Mul` are deliberately exempt. `or_reduce_terms` dedups input terms before chaining Ors; `make_none_selected` routes through it. See `book/src/structural-rules.md` Rule 8.
- `gen::cone::pick_gate` only offers comparison ops when the parent target width is 1.
- `gen::cone::build_flop_leaf` allocates `Flop` (with random `FlopKind`) and `FlopQ` together; `Flop.q` always points at the new `FlopQ` node; `Flop.d` and `Flop.mux` are filled later by `drain_flop_worklist`.
- All flops use `ResetKind::Async` unconditionally (single-CLK / single-RST_N synchronous discipline).
- `pick_mux_arm_count` returns M from {0, 2, 3, ..., max_mux_arms}. M = 1 is excluded by design.
- `drain_flop_worklist` constructs each flop's D as one of:
  - (a) a direct recursive cone when M=0;
  - (b) one-hot mux `OR_i({N{sel_i}} & data_i)` (+ `{N{none_selected}} & Q` for `QFeedback`) for the OneHot style;
  - (c) encoded-select chained-ternary mux over `Eq(sel, k)` with a fall-through of 0 (ZeroDefault) or Q (QFeedback) for the Encoded style. QFeedback+Encoded replaces `data_0` with Q.
  The style is picked per-flop via `cfg.flop_mux_encoding_prob`. Sub-cones pass `exclude = None`: this flop's Q is a freely-reachable leaf inside its own D-cone sub-cones. See `book/src/structural-rules.md` Rule 2 for the authoritative statement; Rule 3 covers the explicit QFeedback mux term.
- `pick_terminal_dep_bearing(g, m, pool, width, exclude)` is a strict variant of `pick_terminal`: only admits (1) a random dep-bearing matching-width pool entry or (2) a width-adapter from the widest dep-bearing pool entry. Used at mux selects (comb + flop, encoded + one-hot), priority-encoder request bits, const-comparand LHS, and const-shift value operand — all pool-mode sites. Panics if the pool has no dep-bearing entry (invariant). See `book/src/structural-rules.md` Rule 20.
- `pick_coefficient(g, width)` clamps the draw range to `[max(min_coefficient,1), min(max_coefficient, 2^W-1)]` so the emitted `width`-bit `Constant` can never overflow its declared width. Width=1 forces c=1; larger widths see the unclamped range up to `2^W-1`. See `book/src/structural-rules.md` Rule 19.
- Associative operators (`And`, `Or`, `Xor`, `Add`, `Mul`) are N-arity with N drawn from `[cfg.min_gate_arity, cfg.max_gate_arity]` each emission. `Sub` stays strictly 2-arity (not associative). Non-operators retain their natural operand counts. See `book/src/structural-rules.md` Rule 14 and the "Operators vs blocks" preamble.
- The full catalog of enforced invariants lives in `book/src/structural-rules.md`. This file's invariants lists above are a summary with pointers to the catalog.
- `pick_terminal` filters out the excluded `NodeId` from every candidate set (matching-width, dep-bearing, fallback adapter source).
- `gen::cone::pick_node_kind` (inline in `build_cone`) gates flop selection on `m.flops.len() < cfg.max_flops_per_module` to bound generation cost.
- `gen::module::generate_leaf_module` reserves port id 0 for `clk` and 1 for `rst_n`. Neither is added to the signal pool, so cones cannot terminate at them.

In `ir::validate::validate`:
- Operand `NodeId`s in range.
- Each output port has exactly one drive.
- Every flop has a `d` set.
- Output-cone root has non-empty dep-set.
- Per-gate arity: each `GateOp` variant has a fixed or variadic-with-min operand count.
- Per-gate operand widths: `And/Or/Xor/Add/Sub/Mul` / `Not` require operand width == output width; `Mux` requires sel 1-bit + two data operands at output width; `Eq/Neq/Lt/Gt/Le/Ge` require equal-width operands + 1-bit output; `RedAnd/RedOr/RedXor` require 1-bit output; `Shl/Shr` require value operand at output width (shift amount unconstrained); `Slice{hi,lo}` requires `hi >= lo`, `out_w == hi-lo+1`, source width > `hi`; `Concat` requires sum of operand widths == output width.

## Testing surface

- `tests/pipeline.rs` — 20-seed cross-seed generation + validation + reproducibility.
- `src/ir/validate.rs` — 8 inline unit tests covering valid modules and each class of rejection (operand width mismatch, mux selector width, Eq output width, Concat sum, Slice out-of-bounds, wrong arity, variadic replicate Concat).
- `src/gen/cone.rs` — 11 inline unit tests. Prior 7 (`ceil_log2`, `pick_mux_arm_count`, 4 width-adapter cases, DAG-sharing sanity, comb-mux-block) plus 4 new flop-assembler tests covering OneHot/ZeroDefault, OneHot/QFeedback, Encoded/ZeroDefault, Encoded/QFeedback — with `fixture_with_inputs` / `alloc_flop` shared helpers.
- `src/emit/sv.rs` — 6 inline unit tests pinning emitter output on hand-built IRs: module header + endmodule + port declarations + passthrough assign, conditional omission of clk/rst_n when zero flops, canonical `always_ff @(posedge clk or negedge rst_n)` header with active-low reset branch, operator and constant rendering, Slice `[hi:lo]` and Concat `{a, b}` forms, Mux ternary form.
- Total: 39 unit tests + 15 integration = **54 tests, all passing**.
- No external smoke tests wired up yet. Phase 1 exit gate requires Verilator-lint pass on a representative seed range.

## Known weaknesses (visible in code today)

- `gen::cone::input_widths_for` for `Slice` and `Concat` returns placeholder widths. `Slice` and `Concat` are not currently selectable in `pick_gate` (only used by `make_width_adapter` and the flop-mux assembly, which construct them directly with correct widths). Properly wire `input_widths_for` when Phase 3 makes them pickable.
- `emit::sv::render_gate` for `Concat` joins operand names with commas (correct SV); the IR does not currently distinguish per-operand widths in storage because every current producer of `Concat` either replicates a single source or concatenates uniform-width bits. When variadic `Concat` with mixed widths becomes a real motif, the IR shape is still adequate (widths are a property of each operand node, not of the `Concat` itself), but a generator-side helper will need to compose such shapes carefully.

## Build hygiene
- `cargo fmt --all --check` — clean.
- `cargo clippy --all-targets -- -D warnings` — clean.
