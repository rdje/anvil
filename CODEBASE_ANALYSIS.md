# Code Base Analysis
Live analysis of the Rust workspace as it currently stands. Updated whenever a slice materially changes the workspace.

## Snapshot
- **Workspace:** single crate `anvil` (no Cargo workspace; flat layout).
- **Edition:** 2021.
- **Targets:** one binary (`anvil`), one library (`anvil`), one example (`generate_one`), one integration test (`pipeline`).
- **External deps:** `rand`, `rand_chacha`, `clap`, `serde`, `serde_json`, `thiserror`, `anyhow`, `tracing`, `tracing-subscriber`. `insta` (dev) reserved for snapshot tests. `tracing` carries `release_max_level_info` so trace-level calls compile out in release.
- **MSRV:** not yet pinned. Whatever stable Rust is current.

## Suitability assessment against the product goal

Short answer: **yes as a foundation, not yet as a completed generator**.

The current architecture is well matched to ANVIL's direction:

- typed IR construction instead of grammar/text emission;
- one combinational identity chokepoint in `ir/types.rs`;
- post-drain state and reachability finalisation in `ir/compact.rs`;
- validator-owned invariants in `ir/validate.rs`;
- explicit knob/control plumbing in `config.rs`; and
- a deliberately dumb SV emitter in `emit/sv.rs`.

That is the right base for a signoff-grade random synthesizable RTL
generator. The work still required falls into four explicit gaps:

1. **Feature breadth / legal surface area**
   The active generator is still leaf-module-centric. Phase 3 motifs are
   only partially populated, while hierarchy, parameterization,
   aggregates, memories, and FSMs are not landed. The codebase supports
   these as extensions of the current architecture; it does not yet
   implement them.
2. **`NodeId`-as-identity is only partially realized**
   `Module::intern_gate` gives a strong combinational canonicalization
   chokepoint and `merge_equivalent_flops` adds the first stateful
   sharing pass, but "same expression anywhere in the cone forest means
   same `NodeId`" is not yet fully true for stronger sequential
   equivalence or future hierarchical objects.
3. **Tool-clean confidence is still under-automated**
   The repo has strong internal validation and local smoke evidence, but
   it does not yet have the broad Verilator/Yosys sweep matrix implied
   by the signoff-grade goal. That missing harness is now one of the
   main quality gaps, not an optional extra.
4. **The IR is optimized for structural legitimacy more than semantic
   richness today**
   That matches the project doctrine: whole-module intended behavior is
   usually arbitrary. The missing work is therefore not "add a
   spec/oracle layer", but "add more legal, synthesizable,
   interaction-rich motifs and composition surfaces".

## Module map

```
src/
├── main.rs           CLI entry point. Parses `Cli`, loads/merges
│                     `Config`, validates it, runs `Generator`, and
│                     writes stdout or per-file output with
│                     `manifest.json`. CLI surface covers structure,
│                     sequential motifs, comb-mux / priority-encoder /
│                     coefficient / constant-shift / const-comparand
│                     motifs, construction strategy, factorization,
│                     tracing, and metrics. Tracing:
│                     `init_tracing` wires a deterministic subscriber
│                     from `--trace <level>` + `--trace-file`;
│                     `TraceLevel` maps `low=INFO`, `medium=DEBUG`,
│                     `high/debug=TRACE`, with `debug` additionally
│                     enabling `trace_verbose!`.
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
├── metrics.rs        Post-hoc structural metrics. `compute(&Module) →
│                     Metrics` covering size, per-kind gate counts,
│                     constant distributions, mux/concat shape,
│                     fanout stats, flop distribution, AST-instance
│                     saturation, operand-arity distribution,
│                     combinational-depth histograms, factorization
│                     counters (`fold_identities_applied`,
│                     `peephole_rewrites_applied`,
│                     `flatten_associative_applied`,
│                     `nodes_compacted`), per-knob roll counters, and
│                     block-build counters. `nested_associative_
│                     operand_count` now counts only same-op nested
│                     slots that remain flattenable under the current
│                     duplicate policy, so preserved Add/Mul duplicate
│                     cases no longer read as missed flattening.
│                     Serde-serializable; embedded in `manifest.json`
│                     and printed with the `--metrics` flag.
│
├── config.rs         Config struct (knobs), Default impl, validate(),
│                     CLI Overrides struct, ConfigError taxonomy.
│                     ConstructionStrategy enum (clap::ValueEnum +
│                     serde): Sequential, Shuffled, Interleaved
│                     (default). GraphFirst variant retained as a
│                     silent alias for Interleaved — the original
│                     speculative pool-growth strategy was retired
│                     for producing Rule 18 violations.
│                     IdentityMode enum (`Relaxed`, `NodeId`) plus
│                     FactorizationLevel (derives PartialOrd/Ord):
│                     None, Cse, OperandUnique, Commutative,
│                     Associative, ConstantFold, Peephole, EGraph
│                     (default request). effective() clamps to the
│                     highest implemented layer (currently
│                     Peephole). Fine-grained knobs:
│                     max_ast_instances, mux_arm_duplication_rate,
│                     operand_duplication_rate, identity_mode,
│                     factorization_level.
│
├── ir/
│   ├── mod.rs        Re-exports `types::*`, `compact::*`, and validate.
│   ├── types.rs      Core types: Module, Port, Direction, Node, GateOp,
│   │                 Flop, ResetKind, DepSet, Design, KnobId,
│   │                 KnobRollCounters. GateOp derives Hash (dedup key).
│   │                 Node kinds: PrimaryInput / Constant / FlopQ / Gate.
│   │                 Module carries construction-time dedup tables:
│   │                 gate_instances: HashMap<(GateOp, Vec<NodeId>,
│   │                 u32), Vec<NodeId>>, const_instances:
│   │                 HashMap<(u32, u128), Vec<NodeId>>, per-module
│   │                 knob mirrors (`max_ast_instances`,
│   │                 `mux_arm_duplication_rate`,
│   │                 `operand_duplication_rate`,
│   │                 `identity_mode`,
│   │                 `factorization_level`), and live counters for
│   │                 block-builds / factorization / sequential-merge /
│   │                 compaction / knob rolls.
│   │                 API: intern_gate(op, operands, width, deps) →
│   │                 (NodeId, is_new) and intern_constant(width,
│   │                 value) → (NodeId, is_new). intern_gate runs the
│   │                 full currently-implemented factorization ladder:
│   │                 associative flattening → commutative sort →
│   │                 constant folding → peephole rewrites → AST-cap
│   │                 CSE, with `identity_mode = Relaxed` forcing the
│   │                 effective level to None and bypassing dedup
│   │                 entirely.
│   │                 Both methods emit `trace_verbose!` 🔗 new /
│   │                 ♻️ reuse events.
│   ├── compact.rs    Post-construction IR finalization helpers.
│   │                 `merge_equivalent_flops(&mut Module)` is a
│   │                 conservative post-drain state-sharing pass:
│   │                 under `identity_mode = NodeId` with effective
│   │                 level `>= Cse`, flops with identical exact
│   │                 signatures (`width`, reset, `d`) collapse to one
│   │                 state element and duplicate Q consumers are
│   │                 rewired. `compact_node_ids(&mut Module)` BFSes
│   │                 from output drives + flop fields, drops
│   │                 unreachable nodes, remaps every surviving NodeId
│   │                 holder, and rebuilds dedup tables. Called from
│   │                 `gen::module::generate_leaf_module`; counts are
│   │                 surfaced as `Metrics::flops_merged` and
│   │                 `Metrics::nodes_compacted`.
│   └── validate.rs   Module invariant checker: operands and drive
│                     roots defined, drive count == 1, flop ids dense,
│                     flop-held NodeIds live, canonical
│                     `Flop.q <-> Node::FlopQ` backrefs, dep-set
│                     non-empty, and per-gate arity + operand-width +
│                     output-width rules for every GateOp variant.
│                     Has inline unit tests covering valid and invalid
│                     hand-built IRs.
│
├── gen/
│   ├── mod.rs        Generator struct (rng + cfg + next_module_index),
│   │                 generate_module(), generate_design() (Phase 5+
│   │                 stub).
│   ├── module.rs     Leaf-module top-level generator: pick port counts,
│   │                 pick widths, seed signal pool with primary inputs,
│   │                 build a cone per primary output. Dispatches on
│   │                 cfg.construction_strategy: Sequential/Shuffled
│   │                 use the recursive build_cone_with_retry path;
│   │                 Interleaved (default) + the deprecated
│   │                 GraphFirst alias both delegate to
│   │                 `cone::build_outputs_interleaved` (frame machine).
│   │                 Drives recorded in declaration order regardless.
│   │                 Finalisation after flop drain: summarize
│   │                 `Flop.mux` metadata to drop dead operand refs,
│   │                 merge exact-signature duplicate flops,
│   │                 orphan audit before compaction,
│   │                 `compact_node_ids`, post-compaction orphan audit,
│   │                 shrink surviving primary inputs to the highest
│   │                 live bit, then prune dead data-input ports from
│   │                 the emitted surface. This is still the Phase
│   │                 1/2/3 leaf kernel; future hierarchy should wrap
│   │                 it rather than collapse inter-module generation
│   │                 into it. `m.flops_merged` and
│   │                 `m.nodes_compacted` record the removal counts.
│   ├── cone.rs       Fanin-cone recursion + interleaved frame machine.
│   │                 Public: FlopWorklist alias, build_cone_with_retry,
│   │                 build_outputs_interleaved, build_graph_first
│   │                 (legacy helper no longer selected by
│   │                 `generate_leaf_module`), drain_flop_worklist,
│   │                 build_cone.
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
│   │                 Width-adapter detail: non-multiple up-width
│   │                 adaptation now builds an exact-width Concat
│   │                 (`{src[rem-1:0], src, ...}`) instead of an
│   │                 oversized replicated Concat plus a low Slice,
│   │                 eliminating dead high bits in emitted SV.
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
│   │                 (invariant violation). Currently exercised by the
│   │                 legacy pool-only helpers (`build_graph_first`,
│   │                 pool-only comb mux / priority encoder / shift
│   │                 paths), not by the active output-cone builders.
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
| 0 — Scaffolding              | done         | All files (initial) | Historical scaffold landed; current HEAD builds/tests/lints/formats clean again (see Build hygiene). |
| 1 — Single-module MVP        | mostly done  | `gen/cone.rs`, `gen/module.rs`, `emit/sv.rs`, `gen/pool.rs`, `ir/types.rs`, `ir/compact.rs`, `metrics.rs` | Combinational + sequential cone recursion functional; flop worklist drained; `always_ff` emitted; single CLK + single RST_N (async). 22 structural rules enforced (Rules 1-22). Zero orphans restored at module finalisation via Rule-18 construction discipline plus `compact_node_ids`; the emitted input surface is now trimmed to live ports/bits. Factorization ladder is live through Peephole, with conservative exact-signature flop merging under `identity_mode = node-id`. Remaining: broader Verilator/Yosys sweeps for the Phase-1 exit gate. |
| 2 — Sharing                  | in progress  | `gen/cone.rs`, `ir/types.rs`, `ir/compact.rs` | Per-operand `share_prob` hook wired; internal gates enter the pool as they are built. Construction-time CSE (Rule 21) + operand-uniqueness (Rule 8 extended) + commutative normalization (Rule 21b) + associative flattening + constant folding + peephole rewrites all enforced via `intern_gate`; exact-signature duplicate flops now merge post-drain under `identity_mode = node-id`; final compaction cleans orphaned intermediates from these rewrites. |
| 3 — Structured combinational | in progress  | `gen/cone.rs`, `ir/types.rs`, `emit/sv.rs`, `ir/validate.rs` | Priority-encoder block (Rule 17), combinational mux block (Rule 15), coefficient motif, const-shift motif, const-comparand motif, and reduction-category gate picking landed. Generic Slice/Concat remain non-pickable helper shapes (width-adapter / block assembly only); case/casez, variable shifts, and loop-unrolled logic are not started. |
| 4 — Hierarchy                | not started  | new `gen/hierarchy.rs`; `Design` already typed | Library + on-demand sourcing. |
| 5 — Parameterization         | not started  | new module | Significant extension to IR (parameter env). |
| 6 — Advanced motifs          | not started  | various | Memories, FSMs, optional multi-clock. |

## Invariants currently enforced

In code (constructors / generator):
- `Module::intern_gate` / `intern_constant` enforce the currently-implemented combinational factorization ladder (Rule 21 / 21b / 21c): associative flattening, commutative sort on `And`/`Or`/`Xor`/`Add`/`Mul`, constant folding, peephole rewrites, then AST-cap CSE keyed by `(op, operands, width)` / `(width, value)`. `identity_mode = Relaxed` forces the effective level to `None`; `identity_mode = NodeId` uses `FactorizationLevel::effective()`, which currently clamps aspirational levels to `Peephole`.
- `Config::validate()` rejects out-of-range knobs.
- `Generator::new()` seeds RNG deterministically.
- `gen::module::generate_leaf_module` produces port counts within knob ranges.
- `gen::cone::build_cone_with_retry` retries up to 4× on empty-dep-set cone roots; snapshots `m.nodes`, `m.flops`, pool, worklist, `gate_instances`, `const_instances` before each attempt and restores on empty-dep retry.
- `gen::cone::build_cone` snapshots the same state before operand construction. On anti-collapse rejection, restores the snapshot and returns `pick_terminal` as fallback. No orphan leaks from rejected recursive gates.
- `gen::cone::process_signal_frame` (interleaved) uses an existing operand as anti-collapse fallback (not `pick_terminal`) because per-gate snapshot is infeasible once sibling frames have committed.
- `gen::module::summarize_flop_mux_metadata` clears construction-only mux operand references once `flop.d` exists, so metadata-only select/data cones do not survive liveness/compaction.
- `ir::compact::merge_equivalent_flops` is the first stateful extension of the NodeId-as-identity contract. It runs after D-cones exist, only under `identity_mode = NodeId` with effective level `>= Cse`, and merges flops by exact emitted-state signature: `width`, `reset_kind`, `reset_val`, `d`. It rewires duplicate Q consumers, remaps virtual flop deps, renumbers surviving flops, and rebuilds dedup tables. It is exact-signature only, not coinductive sequential equivalence.
- `gen::module::generate_leaf_module` runs `count_orphan_gates(m)` after the merge / before compaction as a Rule 18 safety-net audit, then `compact_node_ids`, then a second orphan audit; `m.flops_merged` and `m.nodes_compacted` record the numbers of removed duplicates / unreachable nodes.
- `gen::module::shrink_primary_inputs_to_live_width` reduces each surviving primary input to the highest bit any live consumer touches; `prune_unused_input_ports` removes data-input ports with no surviving `PrimaryInput` node.
- `gen::cone::pick_terminal` prefers matching-width pool entries with non-empty deps; on no width-match, builds a width-adapter (`make_width_adapter`) from the widest dep-bearing pool entry; only emits a constant when the entire pool has empty deps.
- `gen::cone::build_cone` consults `cfg.share_prob` per operand: with that probability it calls `try_share` to return an existing matching-width pool entry (with deps, honoring `exclude`); otherwise it recurses. Fresh `Gate` nodes enter the pool on creation, so later operand decisions in the same call chain can share them.
- `gen::cone::make_width_adapter` produces a Slice (when source > target) or an exact-width Concat (when source < target), using a leading low Slice only for the remainder chunk in non-multiple expansions. Deps propagate from the source.
- `gen::cone::violates_anti_collapse` rejects duplicate operands in `And`/`Or`/`Xor` whenever the effective factorization level is `>= OperandUnique`, rejects duplicate operands in `Add`/`Mul` at the same levels when `operand_duplication_rate < 1.0`, rejects `x - x`, `x == x`, `x != x` at 2-arity, and rejects `mux(s, a, a)` when `mux_arm_duplication_rate < 1.0`. `or_reduce_terms` dedups input terms before chaining Ors; `make_none_selected` routes through it. See `book/src/structural-rules.md` Rule 8.
- `gen::cone::pick_gate` only offers comparison ops when the parent target width is 1.
- `gen::cone::build_flop_leaf` allocates `Flop` (with random `FlopKind`) and `FlopQ` together; `Flop.q` always points at the new `FlopQ` node; `Flop.d` and `Flop.mux` are filled later by `drain_flop_worklist`.
- All flops use `ResetKind::Async` unconditionally (single-CLK / single-RST_N synchronous discipline).
- `pick_mux_arm_count` returns M from {0, 2, 3, ..., max_mux_arms}. M = 1 is excluded by design.
- `drain_flop_worklist` constructs each flop's D as one of:
  - (a) a direct recursive cone when M=0;
  - (b) one-hot mux `OR_i({N{sel_i}} & data_i)` (+ `{N{none_selected}} & Q` for `QFeedback`) for the OneHot style;
  - (c) encoded-select chained-ternary mux over `Eq(sel, k)` with a fall-through of 0 (ZeroDefault) or Q (QFeedback) for the Encoded style. QFeedback+Encoded replaces `data_0` with Q.
  The style is picked per-flop via `cfg.flop_mux_encoding_prob`. Sub-cones pass `exclude = None`: this flop's Q is a freely-reachable leaf inside its own D-cone sub-cones. See `book/src/structural-rules.md` Rule 2 for the authoritative statement; Rule 3 covers the explicit QFeedback mux term.
- `pick_terminal_dep_bearing(g, m, pool, width, exclude)` is a strict variant of `pick_terminal`: only admits (1) a random dep-bearing matching-width pool entry or (2) a width-adapter from the widest dep-bearing pool entry. Today it is used by the legacy pool-only helpers (`build_graph_first`, comb-mux / priority-encoder / const-shift pool paths); the active recursive/interleaved output-cone paths build those signals via `build_cone` instead. Panics if the pool has no dep-bearing entry (invariant). See `book/src/structural-rules.md` Rule 20.
- `pick_coefficient(g, width)` clamps the draw range to `[max(min_coefficient,1), min(max_coefficient, 2^W-1)]` so the emitted `width`-bit `Constant` can never overflow its declared width. Width=1 forces c=1; larger widths see the unclamped range up to `2^W-1`. See `book/src/structural-rules.md` Rule 19.
- Associative operators (`And`, `Or`, `Xor`, `Add`, `Mul`) are N-arity with N drawn from `[cfg.min_gate_arity, cfg.max_gate_arity]` each emission. `Sub` stays strictly 2-arity (not associative). Non-operators retain their natural operand counts. See `book/src/structural-rules.md` Rule 14 and the "Operators vs blocks" preamble.
- The full catalog of enforced invariants lives in `book/src/structural-rules.md`. This file's invariants lists above are a summary with pointers to the catalog.
- `pick_terminal` filters out the excluded `NodeId` from every candidate set (matching-width, dep-bearing, fallback adapter source).
- `build_cone`, `process_signal_frame`, `grow_pool_one_unit`, `pick_terminal`, and `drain_flop_worklist` route every probability choice through `roll_knob`, populating `m.knob_rolls` for measurability of `flop_prob`, `comb_mux_prob`, `priority_encoder_prob`, `coefficient_prob`, `const_shift_amount_prob`, `const_comparand_prob`, `constant_prob`, `terminal_reuse_prob`, `comb_mux_encoding_prob`, `flop_mux_encoding_prob`, `share_prob`, and `flop_qfeedback_prob`.
- `gen::module::generate_leaf_module` reserves port id 0 for `clk` and 1 for `rst_n`. Neither is added to the signal pool, so cones cannot terminate at them.

In `ir::validate::validate`:
- Operand `NodeId`s in range.
- Every drive root `NodeId` exists, and each output port has exactly
  one drive.
- Every flop table slot keeps the dense canonical relation
  `m.flops[idx].id == idx`.
- Every flop has a `d` set.
- `Flop.d`, `Flop.q`, and every `NodeId` held inside `FlopMux`
  point at live nodes.
- `Flop.q` points at `Node::FlopQ { flop: self.id, width:
  self.width }`.
- Every `Node::FlopQ` references a real flop, matches the owning
  flop's width, and is that flop's canonical `q` node.
- Output-cone root has non-empty dep-set.
- Per-gate arity: each `GateOp` variant has a fixed or variadic-with-min operand count.
- Per-gate operand widths: `And/Or/Xor/Add/Sub/Mul` / `Not` require operand width == output width; `Mux` requires sel 1-bit + two data operands at output width; `Eq/Neq/Lt/Gt/Le/Ge` require equal-width operands + 1-bit output; `RedAnd/RedOr/RedXor` require 1-bit output; `Shl/Shr` require value operand at output width (shift amount unconstrained); `Slice{hi,lo}` requires `hi >= lo`, `out_w == hi-lo+1`, source width > `hi`; `Concat` requires sum of operand widths == output width.

## Testing surface

- `src/ir/types.rs` — 36 inline unit tests covering commutative normalization, constant folding, peephole rewrites, all-constant evaluation, associative flattening, identity-mode gates, unsigned-boundary tautologies, and const-selector mux collapse.
- `src/ir/validate.rs` — 21 inline unit tests covering valid modules plus a broad rejection surface: undefined drive roots, dense flop-id enforcement, missing D, undefined mux-held refs, canonical `Flop.q` / `FlopQ` backrefs and widths, dangling / duplicate `FlopQ`s, and representative gate-shape failures.
- `src/gen/cone.rs` — 18 inline unit tests covering flop assemblers, `ceil_log2`, `pick_mux_arm_count`, width-adapter cases, comb-mux generation, DAG-sharing sanity, anti-collapse, dep-bearing terminal picking, coefficient-width clamping, CLI alias behavior, and category / leaf-knob exercise coverage.
- `src/gen/module.rs` — 2 inline unit tests covering primary-input width shrinking and the "do not shrink full-width non-slice uses" guard.
- `src/emit/sv.rs` — 6 inline unit tests pinning emitter output on hand-built IRs: module header + endmodule + port declarations + passthrough assign, conditional omission of clk/rst_n when zero flops, canonical `always_ff @(posedge clk or negedge rst_n)` header with active-low reset branch, operator and constant rendering, Slice / Concat rendering, and Mux ternary form.
- `src/metrics.rs` — 3 inline unit tests for empty-module, per-kind gate, and flop-shape metrics.
- `src/ir/compact.rs` — 6 inline unit tests for exact-signature flop merge (consumer + dep rewrite, relaxed-mode bypass, reset-signature separation) plus no-op compaction, orphan removal, and topological-order preservation.
- `tests/pipeline.rs` — 24 integration tests covering cross-seed validity, reproducibility across strategies, motif sweeps, all live gate categories, zero-orphan / zero-duplicate-operand doctrine guards, input-surface finalisation, associative / constant-fold / peephole / compaction counters, and knob-roll telemetry.
- Current executed counts (`cargo test`, 2026-04-20): **99 unit + 24 integration = 123 passing tests**. Doc-tests: 0.
- No external Verilator / Yosys smoke tests are wired into `cargo test` yet. Phase 1 exit gate remains blocked on running the larger sweeps, not on tool availability.

## Known weaknesses (visible in code today)

- The broader signoff-grade cleanliness matrix described in
  `ROADMAP.md` does not yet exist as repo-owned automation. Internal
  validation is strong; industrialized tool evidence is still an open
  engineering lane.
- `NodeId`-as-identity is still conservative for state and does not yet
  extend to future hierarchical objects. Exact-signature duplicate
  flops merge; stronger sequential/hierarchical equivalence remains open
  work.
- `gen::cone::input_widths_for` for `Slice` and `Concat` returns placeholder widths. `Slice` and `Concat` are not currently selectable in `pick_gate` (only used by `make_width_adapter` and the flop-mux assembly, which construct them directly with correct widths). Properly wire `input_widths_for` when Phase 3 makes them pickable.
- `emit::sv::render_gate` for `Concat` joins operand names with commas (correct SV); the IR does not currently distinguish per-operand widths in storage because every current producer of `Concat` either replicates a single source or concatenates uniform-width bits. When variadic `Concat` with mixed widths becomes a real motif, the IR shape is still adequate (widths are a property of each operand node, not of the `Concat` itself), but a generator-side helper will need to compose such shapes carefully.

## Build hygiene
- `cargo check --all-targets` — clean.
- `cargo test` — clean (123 passing tests: 99 unit + 24 integration).
- `cargo build` — clean.
- `cargo clippy --all-targets -- -D warnings` — clean.
- `cargo fmt --all --check` — clean.
- `mdbook build book` — clean.
- Generator-output smoke: Verilator lint on seed 42 is clean with no warning-specific suppressions beyond the usual filename noise; the previous `UNSIGNED` / `CMPCONST` tautology residue is now folded away in the IR; a default + graph-first-alias seed sweep (0..4) is clean for `UNUSEDSIGNAL`; Yosys `read_verilog -sv ...; synth` on seed 42 reports 0 problems.
