# Code Base Analysis
Live analysis of the Rust workspace as it currently stands. Updated whenever a slice materially changes the workspace.

## Snapshot
- **Workspace:** single crate `anvil` (no Cargo workspace; flat layout).
- **Edition:** 2021.
- **Targets:** one binary (`anvil`), one library (`anvil`), one example (`generate_one`), one integration test (`pipeline`).
- **External deps:** `rand`, `rand_chacha`, `clap`, `serde`, `serde_json`, `thiserror`, `anyhow`, `tracing`, `tracing-subscriber`. `insta` (dev) reserved for snapshot tests. `tracing` carries `release_max_level_info` so trace-level calls compile out in release.
- **MSRV:** pinned to Rust 1.95 via `Cargo.toml` `rust-version = "1.95"`.

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

1. **Feature breadth / legal surface area / artifact-family breadth**
   The active generator is still leaf-module-centric. The previously
   explicit Phase 3 breadth gaps (`case`, `casez`, variable shifts,
   generic selectable `Slice` / `Concat`, bounded unrolled logic) are
   now landed, and the dedicated Phase 3 structured-surface closure
   gate is landed too. Hierarchy, parameterization,
   aggregates, memories, and FSMs are not landed. Beyond that, the
   newer user direction broadens the target beyond one output family:
   ANVIL should eventually generate more kinds of
   valid-by-construction synthesizable artifacts such as oracle-backed
   micro-design corpora and frontend/elaboration accept corpora. The
   current codebase supports this as an architectural direction, but it
   does not yet implement the needed source-level IR or artifact-family
   plumbing.
2. **`NodeId`-as-identity is only partially realized**
   `Module::intern_gate` gives a strong combinational canonicalization
   chokepoint and `merge_equivalent_flops` adds the first stateful
   sharing pass, but "same expression anywhere in the cone forest means
   same `NodeId`" is not yet fully true for stronger sequential
   equivalence or future hierarchical objects.
3. **Tool-clean confidence still needs broader automation beyond Phases 1/2**
   The repo now has strong internal validation and strong local smoke
   evidence. That includes a real `tool_matrix --phase1-gate` frontier
   pushed to 365 warning-clean modules in the older no-ABC lane, a
   later historical both-mode frontier at 570 completed checkpoints /
   571 emitted files, and a completed current-code resumable both-mode
   tree at `/tmp/anvil-tool-matrix-phase1-real-r21` with a final
   `tool_matrix_report.json` at 1005 completed checkpoints / 1005
   emitted files, zero warning artifacts, full clean closure of all 15
   built-in scenarios, empty coverage gaps, and 1005/0 pass-fail in
   Verilator plus both repo-owned Yosys modes. `tool_matrix` also now
   has a same-binary fast-resume path that can skip replaying
   already-proven modules when the checkpoint fingerprint and saved-SV
   hash still match, and the live `r21` bank has already been upgraded
   in place to that checkpoint format. The latest focused current-code
   `e-graph` proof (`seed=8 / interleaved / node-id / e-graph /
   count=54`) is also clean 54/54 in Verilator and both repo-owned
   Yosys modes. On top of that, the new
   `/tmp/anvil-tool-matrix-phase2-share-r1/tool_matrix_report.json`
   closes the representative Phase 2 sharing sweep locally: 216/0 in
   Verilator plus both repo-owned Yosys modes, `coverage_gaps = []`,
   and a monotone normalized `share_sweep` summary across
   `share_prob ∈ {0.0, 0.3, 0.9}`. So the basic Phase 1 and Phase 2
   closure evidence now exists; the remaining confidence gap is broader
   automation for later phases, richer knob sweeps, and the larger
   artifact-family space implied by the signoff-grade goal.
4. **The IR is optimized for structural legitimacy more than semantic
   richness today**
   That matches the project doctrine: whole-module intended behavior is
   usually arbitrary. The missing work is therefore not "add a
   bundled spec/oracle layer", but "add more legal, synthesizable,
   interaction-rich motifs, composition surfaces, and explicit
   expected-facts manifests where a particular artifact family needs
   them".

Taken literally against the user's `rtl_const_expr` / `rtl_frontend`
style request, the repo is **not ready yet**. It currently lacks:

- a source-level parameter / package / typedef / instantiation IR;
- compact hierarchy generation beyond the planned future hierarchy lane;
- manifest infrastructure for expected elaboration facts; and
- an artifact-family selector above the current leaf-module generator.

So the answer remains "yes as a foundation", but now with a clearer
explanation of what is missing for the broader artifact-family goal.

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
│                     Metrics` plus `compute_design(&Design) →
│                     DesignMetrics` covering size, per-kind gate counts,
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
│                     Design metrics cover hierarchy composition
│                     directly: library coverage, unused-library
│                     fraction, instance reuse, top interface shape,
│                     control fanout, weighted child load/complexity,
│                     and per-definition instantiation histograms.
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
│                     (default request). effective() now keeps the
│                     bounded live `e-graph` fragment under
│                     `identity_mode = node-id` instead of clamping
│                     everything above `Peephole` downward.
│                     Fine-grained knobs:
│                     max_ast_instances, mux_arm_duplication_rate,
│                     operand_duplication_rate, identity_mode,
│                     factorization_level.
│
├── bin/
│   └── tool_matrix.rs
│                     Repo-owned downstream-tool matrix harness.
│                     Builds a curated scenario set over
│                     construction strategy, identity mode,
│                     factorization level, and two stress profiles;
│                     generates per-scenario corpora, runs Verilator
│                     and Yosys, writes per-module
│                     `.module-report.json` checkpoints plus the final
│                     `tool_matrix_report.json`, aggregates
│                     metrics/coverage facts, and exits non-zero on
│                     tool failures. `--phase1-gate` lifts the run to
│                     >=1000 total modules with coverage-gap failure
│                     enabled; `--resume` reuses compatible
│                     per-module checkpoints, now with a same-binary
│                     fast path that restores generator RNG state from
│                     saved generator checkpoints when the runtime
│                     fingerprint and emitted-`sv` hash still match,
│                     and bootstraps older trees from saved `.sv`
│                     artifacts otherwise. Also doubles as the first
│                     executable "axis matrix" proof surface.
│                     Yosys is now a first-class harness axis too:
│                     `--yosys-mode <without-abc|with-abc|both>`
│                     selects the current stable `synth -noabc`
│                     baseline, the explicit ABC-enabled
│                     `synth -noabc; abc -fast; opt -fast; check`
│                     harness path, or both as separate sub-runs per
│                     file. `--phase2-share-gate` now adds the
│                     repo-owned representative `share_prob`
│                     sweep, and its report records a normalized
│                     `share_sweep` summary so stronger sharing can be
│                     proven even when the raw shared-node count falls
│                     because the graph itself is collapsing.
│                     Current recorded real frontiers: 365 clean
│                     modules on the older no-ABC gate; a historical
│                     372-checkpoint / 373-emitted-file both-mode tree
│                     (`r18`) with zero warning artifacts; the later
│                     historical `r20` tree at 570 completed
│                     checkpoints / 571 emitted `.sv` files; and the
│                     completed current-code both-mode tree (`r21`) at
│                     1005 completed checkpoints / 1005 emitted `.sv`
│                     files, also warning-clean throughout. The
│                     completed current-code Phase 2 share-sweep tree
│                     (`/tmp/anvil-tool-matrix-phase2-share-r1`) adds
│                     216 warning-clean modules over the 18-scenario
│                     representative sharing matrix.
│                     Because `--resume` is intentionally byte-stable,
│                     later proof-driven `.sv` changes turn old trees
│                     into evidence only; `r18` and `r20` are
│                     historical while `r21` matches current code. The
│                     whole saved `r21` bank now carries the newer
│                     fast-resume metadata too, so future same-binary
│                     resumes on that tree can restore generator state
│                     directly instead of replaying the old checkpoints
│                     again.
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
│   │                 level `>= Cse`, flops collapse when their
│   │                 D-cones are proven equal over the same canonical
│   │                 leaf endpoints by the current proof subset:
│   │                 normalized structural signature first, plus a
│   │                 bounded semantic signature for small-support
│   │                 cones (`width`, reset, and endpoint-aware proof).
│   │                 Different endpoint variables do not merge.
│   │                 `merge_equivalent_gates(&mut Module)` is the
│   │                 first live bounded `e-graph` fragment:
│   │                 under `identity_mode = node-id` and effective
│   │                 `EGraph`, small-support combinational cones
│   │                 proven equal over the same canonical leaf
│   │                 variables collapse to one gate. Then
│   │                 `merge_equivalent_flops(&mut Module)` applies
│   │                 the analogous endpoint-aware proof discipline
│   │                 to state elements. `fold_proven_gates(&mut Module)`
│   │                 keeps the general cleanup exact prover tiny-only,
│   │                 but still revisits compare gates with the bounded
│   │                 unsigned-compare proof and shift gates with a
│   │                 bounds-only exact check, so large-endpoint
│   │                 `x >= 0` or `1 >> rhs` tautologies do not leak
│   │                 through.
│   │                 `compact_node_ids(&mut Module)` now
│   │                 BFSes from output drives, discovers live flops
│   │                 through actually-consumed `FlopQ` leaves, drops
│   │                 unreachable nodes plus dead flops, remaps
│   │                 surviving NodeIds / FlopIds and virtual flop
│   │                 deps, and rebuilds dedup tables. Called from
│   │                 `gen::module::generate_leaf_module`; counts are
│   │                 surfaced as `Metrics::semantic_gates_merged`,
│   │                 `Metrics::flops_merged`, and
│   │                 `Metrics::nodes_compacted`.
│   └── validate.rs   Module invariant checker: operands and drive
│                     roots defined, drive count == 1, flop ids dense,
│                     flop-held NodeIds live, canonical
│                     `Flop.q <-> Node::FlopQ` backrefs, dep-set
│                     non-empty, and per-gate arity + operand-width +
│                     output-width rules for every GateOp variant.
│                     Also exports `validate_design(&Design)` for
│                     design-level hierarchy checks (top exists,
│                     child modules exist, bindings are complete,
│                     output exposure is complete, acyclic module
│                     graph). Has inline unit tests covering valid
│                     and invalid hand-built IRs.
│
├── gen/
│   ├── mod.rs        Generator struct (rng + cfg + next_module_index),
│   │                 generate_module(), generate_design(). Depth 0
│   │                 still routes into the mature leaf-module lane;
│   │                 hierarchy dispatches to either the legacy exact
│   │                 depth-1 wrapper lane or the newer bounded
│   │                 recursive lane. No artifact-family selector
│   │                 exists yet.
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
│   │                 bounded semantic gate merge at the live
│   │                 `EGraph` fragment, endpoint-aware flop merge,
│   │                 orphan audit before compaction,
│   │                 `compact_node_ids`, post-compaction orphan audit,
│   │                 shrink surviving primary inputs to the highest
│   │                 live bit, then prune dead data-input ports from
│   │                 the emitted surface. This is still the Phase
│   │                 1/2/3 leaf kernel; future hierarchy should wrap
│   │                 it rather than collapse inter-module generation
│   │                 into it. `m.semantic_gates_merged`,
│   │                 `m.flops_merged`, and `m.nodes_compacted`
│   │                 record the removal counts.
│   ├── hierarchy.rs  Current Phase 4 planner. Keeps the legacy exact
│   │                 depth-1 wrapper lane alive, and also lands a
│   │                 bounded recursive lane driven by
│   │                 `min_hierarchy_depth..=max_hierarchy_depth` and
│   │                 `min_child_instances_per_module..=max_child_instances_per_module`.
│   │                 The recursive lane currently picks one exact
│   │                 realized depth inside the requested interval for
│   │                 the whole design, chooses each non-leaf module's
│   │                 child count inside the requested interval, and
│   │                 still keeps the parent-side layer combinational.
│   │                 Both lanes build real parent-side logic over child
│   │                 `InstanceOutput` leaves. Local parent flops,
│   │                 mixed shallow/deep branches, and hierarchy-aware
│   │                 identity are still open.
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
│   │                 Generator-side comparison cleanliness is also
│   │                 enforced here: `obvious_unsigned_compare_result`
│   │                 combines unsigned bounds with an exact
│   │                 finite-set proof engine that is now both
│   │                 budgeted and support-capped (up to 8 bits wide,
│   │                 current endpoint-support cap = 3).
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
    ├── mod.rs        Re-exports to_sv, to_sv_in_design, to_sv_design.
    └── sv.rs         IR → String pretty-printer. Assumes invariants hold.
                      No validation. Fixed 4-space indent. Naming:
                      build_names walks m.nodes once, assigns each
                      Gate node a `<kind>_<per-kind-counter>` name
                      (and_0, mux_3, etc.); flops are flop_<id>;
                      instance outputs are `instout_<instance>_<port>`;
                      non-gate nodes resolve via node_ref. Now emits
                      real child-module instantiations too when given
                      design context. See Rule 12 in
                      book/src/structural-rules.md.
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
| 1 — Single-module MVP        | done         | `gen/cone.rs`, `gen/module.rs`, `emit/sv.rs`, `gen/pool.rs`, `ir/types.rs`, `ir/compact.rs`, `metrics.rs` | Combinational + sequential cone recursion functional; flop worklist drained; `always_ff` emitted; single CLK + single RST_N (async). 22 structural rules enforced (Rules 1-22). Zero orphans restored at module finalisation via Rule-18 construction discipline plus `compact_node_ids`; final compaction now also drops dead flops whose `Q` is never observed, and the emitted input surface is trimmed to live ports/bits. Factorization ladder is live through a bounded `EGraph` fragment, with post-construction semantic gate merging for small-support cones, post-remap associative re-normalisation on the settled graph, a late mixed-associative-constant cleanup pass on that same settled graph, endpoint-preserving post-drain flop merging under `identity_mode = node-id`, strict Add/Mul remap-pruning under `operand_duplication_rate < 1.0`, a final exact-value cleanup pass (`fold_proven_gates`) for downstream-tool cleanliness that keeps the general exact prover tiny-only (width <= 8, support <= 10 bits, <= 3 canonical leaf endpoints) while still revisiting compare gates with the bounded unsigned-compare proof and shift gates with a bounds-only exact check, plus a tiny-domain rhs fallback for shift overshift proofs when narrow boolean-mask arithmetic keeps the rhs domain small even though the whole cone is large. Exit gate now closed locally via `/tmp/anvil-tool-matrix-phase1-real-r21/tool_matrix_report.json` (1005 modules, `coverage_gaps = []`, 1005/0 in Verilator and both repo-owned Yosys modes). |
| 2 — Sharing                  | done         | `gen/cone.rs`, `ir/types.rs`, `ir/compact.rs` | Per-operand `share_prob` hook wired; internal gates enter the pool as they are built. Construction-time CSE (Rule 21) + operand-uniqueness (Rule 8 extended) + commutative normalization (Rule 21b) + associative flattening + constant folding + peephole rewrites all enforced via `intern_gate`; the live bounded `EGraph` fragment now merges small-support combinational cones post-construction under `identity_mode = node-id`, duplicate flops merge post-drain when they are proven equal over the same canonical leaf endpoints by the same proof discipline, and late remaps are pruned when they would violate the strict Add/Mul duplicate policy. Final compaction cleans orphaned intermediates and dead state from these rewrites. Exit gate now closed locally via `/tmp/anvil-tool-matrix-phase2-share-r1/tool_matrix_report.json` (216 modules, `coverage_gaps = []`, 216/0 in Verilator and both repo-owned Yosys modes). The representative sweep proves controllability with normalized `shared_node_fraction` rather than raw shared-node count, because stronger reuse collapses total node count. |
| 3 — Structured combinational | done         | `gen/cone.rs`, `ir/types.rs`, `emit/sv.rs`, `ir/validate.rs`, `metrics.rs`, `bin/tool_matrix.rs`, `ir/compact.rs` | Priority-encoder block (Rule 17), combinational mux block (Rule 15), procedural case-mux block (`always_comb case`), procedural casez-mux block (`always_comb casez` with non-overlapping wildcard patterns), structured bounded `for`-fold blocks (`always_comb` + `for (int i = 0; i < N; i++)` over packed chunks), generic selectable `Slice` / variadic `Concat`, coefficient motif, both shift-amount paths (`const_shift_amount_prob` plus the ordinary variable-amount path), const-comparand motif, and reduction-category gate picking are all landed. The dedicated structured-surface closure gate now exists in `tool_matrix` as `--phase3-structured-gate`, and it is closed locally via `/tmp/anvil-tool-matrix-phase3-structured-r4/tool_matrix_report.json` (210 modules, `coverage_gaps = []`, 210/0 in Verilator and both repo-owned Yosys modes). The runtime hotspot that surfaced while proving that gate was addressed at the real seam: large settled cones with tiny support now skip semantic merge proofs and fall back to structural proof instead of stalling in `semantic_cone_proof`. |
| 4 — Hierarchy                | in progress  | `gen/hierarchy.rs`, `ir/types.rs`, `ir/validate.rs`, `emit/sv.rs`, `main.rs`, `metrics.rs`, `bin/tool_matrix.rs` | Current live slice now has two lanes: the legacy exact depth-1 wrapper planner and the newer bounded recursive planner. The wrapper lane still exercises exact, reuse, and under-instantiation profiles via `num_child_instances`; the recursive lane uses `min_hierarchy_depth..=max_hierarchy_depth` plus `min_child_instances_per_module..=max_child_instances_per_module`, currently choosing one exact realized depth inside the requested interval for the whole design. Repeated `child_instances_per_depth` overrides are now also live and let branching be steered by parent depth (`0` = top, `1` = its children, ...) on top of that global fallback range. Both lanes build a parent-side combinational output layer over child `InstanceOutput` leaves. Hierarchy manifests/reports now carry exact per-design `DesignMetrics`, so hierarchy quality is measurable without SV inspection (coverage, reuse, top-interface shape, direct-vs-composed outputs, child-output dependency fractions/support, realized depth, depth histograms, branching histograms, per-depth branching summaries, control fanout, weighted child load). The emitted-boundary doctrine is explicit too: pure comb-only modules omit `clk` / `rst_n`, while wrappers keep them visible iff they carry sequential descendants. The fully banked repo-owned Phase 4 matrix is now `/tmp/anvil-tool-matrix-phase4-hierarchy-r9/tool_matrix_report.json` (60 designs, `coverage_gaps = []`, 60/0 in Verilator and both repo-owned Yosys modes), and it covers wrapper exact/reuse/under-instantiation, recursive depth `2`, child-instance profiles `2`, `4`, `2:3`, `1:3`, the per-depth override profile `0=4:4,1=2:2`, and real parent-side composition. The focused clean smokes at `/tmp/anvil-hier-parent-compose-smoke-r1/manifest.json`, `/tmp/anvil-hier-range-smoke-r1/manifest.json`, and `/tmp/anvil-hier-depth-profile-smoke-r1/manifest.json` remain useful targeted evidence. Local parent state, mixed shallow/deep branches, on-demand child sourcing as a user-facing axis, and future hierarchy-aware identity remain open. |
| 5 — Parameterization         | not started  | new module | Significant extension to IR (parameter env). |
| 6 — Advanced motifs          | not started  | various | Memories, FSMs, optional multi-clock. |
| 7 — Oracle-backed micro-design artifacts | not started | new artifact-family layer; manifest extensions; likely source-level artifact builders | Small self-contained synthesizable `.sv` artifacts with expected-facts manifests (parameter values, ranges, generate decisions, similar elaboration facts). |
| 8 — Frontend/elaboration accept corpora | not started | source-level parameter / hierarchy / package / type IR; likely new emitter path | Compact 1–3 module synthesizable designs that stress frontend/elaboration surfaces rather than only the current gate-level leaf kernel. |
| 9 — Multi-artifact umbrella  | not started  | generator entrypoint / config / manifest plumbing | Artifact-family selector above the current leaf-module lane; preserves reproducibility and explicit contracts across all synthesizable families. |

## Invariants currently enforced

In code (constructors / generator):
- `Module::intern_gate` / `intern_constant` enforce the currently-implemented combinational factorization ladder (Rule 21 / 21b / 21c): associative flattening, commutative sort on `And`/`Or`/`Xor`/`Add`/`Mul`, constant folding, peephole rewrites, then AST-cap CSE keyed by `(op, operands, width)` / `(width, value)`. `identity_mode = Relaxed` forces the effective level to `None`; `identity_mode = NodeId` uses `FactorizationLevel::effective()`, which now keeps the bounded `EGraph` fragment live at the top rung. Doctrinally, `node-id` still means full factorization (`NodeId` = expression identity); the ladder is the current build's enforcement/proof-depth dial inside that doctrine, not a competing definition of `node-id`.
- `Config::validate()` rejects out-of-range knobs.
- `Generator::new()` seeds RNG deterministically.
- `gen::module::generate_leaf_module` produces port counts within knob ranges.
- `gen::cone::build_cone_with_retry` retries up to 4× on empty-dep-set cone roots; snapshots `m.nodes`, `m.flops`, pool, worklist, `gate_instances`, `const_instances` before each attempt and restores on empty-dep retry.
- `gen::cone::build_cone` snapshots the same state before operand construction. On anti-collapse rejection, restores the snapshot and returns `pick_terminal` as fallback. No orphan leaks from rejected recursive gates.
- `gen::cone::process_signal_frame` (interleaved) uses an existing operand as anti-collapse fallback (not `pick_terminal`) because per-gate snapshot is infeasible once sibling frames have committed.
- `gen::module::summarize_flop_mux_metadata` clears construction-only mux operand references once `flop.d` exists, so metadata-only select/data cones do not survive liveness/compaction.
- `ir::compact::merge_equivalent_gates` is the first live post-construction combinational `EGraph` fragment. It runs only under `identity_mode = NodeId` with effective level `>= EGraph`, and merges gates by endpoint-preserving proof forms: same width, same canonical primary-input / flop-Q leaf endpoints, and same currently-proven functionality. For small-support cones the proof may be semantic (bounded truth table); otherwise it falls back to the normalized structural proof. Different endpoint variables do not merge.
- `ir::compact::merge_equivalent_flops` is the first stateful extension of the NodeId-as-identity contract. It runs after D-cones exist, only under `identity_mode = NodeId` with effective level `>= Cse`, and merges flops by an endpoint-preserving proof subset: same `width`, `reset_kind`, `reset_val`, and the same D-cone proof over canonical primary-input / flop-Q endpoints. That proof is structural over the normalized IR by default, with a bounded semantic truth-table signature for small-support cones. The pass rewires duplicate Q consumers, remaps virtual flop deps, renumbers surviving flops, and rebuilds dedup tables. Different endpoint variables do not merge, even if the cone skeleton looks similar. It is still far short of coinductive sequential equivalence.
- `gen::module::generate_leaf_module` now re-runs associative normalisation on the settled graph via `ir::compact::flatten_posthoc_associative_gates` after remap-producing passes (`fold_proven_gates`, `merge_equivalent_gates`). This keeps `nested_associative_operand_count` at zero for legal flattening opportunities even when a later remap changes which already-built node an operand points at.
- `gen::module::generate_leaf_module` now repairs any settled-graph output drive root whose deps have collapsed to empty after the late proof-cleanup passes. The repair swaps in a dep-bearing exact-width source (or width-adapter) before final compaction so finalized outputs stay functions of primary inputs and/or flop-Q leaves rather than trivial constants.
- `gen::module::generate_leaf_module` runs `count_orphan_gates(m)` after the merge / before compaction as a Rule 18 safety-net audit, then `compact_node_ids`, then a second orphan audit; `m.semantic_gates_merged`, `m.flops_merged`, and `m.nodes_compacted` record the numbers of removed duplicates / unreachable nodes.
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
- `Config::validate()` still enforces the legacy exact wrapper lane
  (`hierarchy_depth ∈ {0,1}`, `num_leaf_modules >= 1` when exact
  hierarchy is enabled, `num_child_instances > 0` rejected in leaf-only
  mode), but current HEAD also validates the bounded recursive lane:
  exact legacy wrapper knobs and recursive range knobs are mutually
  exclusive, bounded ranges must satisfy `1 <= min <= max`, repeated
  `child_instances_per_depth` overrides must also satisfy
  `1 <= min <= max`, they may only target realized internal parent
  depths inside `[0, max_hierarchy_depth - 1]`, and `num_leaf_modules`
  is intentionally restricted to the legacy exact wrapper lane.

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

In `ir::validate::validate_design`:
- Every module name in `Design.modules` is unique.
- `Design.top` names a real module.
- Every instance references a real child module.
- Every child emitted input port is bound exactly once, at the right
  width.
- Every referenced child output port maps to a real
  `Node::InstanceOutput` of matching width in the parent; unused child
  outputs may stay unconnected.
- The module-instance graph is acyclic.

## Testing surface

- `src/ir/types.rs` — 40 inline unit tests covering commutative normalization, constant folding, mixed-constant aggregation, peephole rewrites, all-constant evaluation, associative flattening, identity-mode gates, unsigned-boundary tautologies, const-selector mux collapse, and the design-aware control-port visibility rule for sequential vs comb-only descendants.
- `src/ir/validate.rs` — 26 inline unit tests covering valid modules plus a broad rejection surface: undefined drive roots, dense flop-id enforcement, missing D, undefined mux-held refs, canonical `Flop.q` / `FlopQ` backrefs and widths, dangling / duplicate `FlopQ`s, representative gate-shape failures, the landed structured `case`, `casez`, and `for-fold` shapes, plus design-level hierarchy acceptance/rejection.
- `src/gen/cone.rs` — 40 inline unit tests covering flop assemblers, `ceil_log2`, `pick_mux_arm_count`, width-adapter cases, comb-mux generation, DAG-sharing sanity, anti-collapse, dep-bearing terminal picking, coefficient-width clamping, dynamic overshift proofs, exact small-set budgeting, support caps, priority-encoder width-domain guards, selectable Slice/Concat shape guards, CLI alias behavior, and category / leaf-knob exercise coverage.
- `src/gen/mod.rs` — 1 inline unit test proving that a saved generator checkpoint reproduces the exact next module after restore.
- `src/gen/module.rs` — 2 inline unit tests covering primary-input width shrinking and the "do not shrink full-width non-slice uses" guard.
- `src/emit/sv.rs` — 15 inline unit tests pinning emitter output on hand-built IRs: module header + endmodule + port declarations + passthrough assign, conditional omission of clk/rst_n when zero flops, canonical `always_ff @(posedge clk or negedge rst_n)` header with active-low reset branch, operator and constant rendering, Slice / Concat rendering, scalar-slice emission without illegal `[0:0]` on scalar `logic`, constant-slice folding to legal literals, Mux ternary form, both procedural case surfaces, the procedural bounded `for` surface, explicit unconnected child-output emission (`.port()`), and the exact hierarchy control-port doctrine for comb-only wrappers, direct sequential wrappers, and grandparent wrappers.
- `src/metrics.rs` — 9 inline unit tests for empty-module, per-kind gate, flop-shape metrics, constant-vs-variable shift-rhs classification, and hierarchy design metrics for reuse, under-instantiation, parent-side composition, bounded recursive tree shape, and per-depth branching profiles.
- `src/ir/compact.rs` — 24 inline unit tests for bounded semantic gate merge, endpoint-aware state merge, relaxed-mode bypass, reset-signature separation, self-feedback non-merge, cleanup exact-proof eligibility caps, the landed `ForFold` exact evaluator, late mixed-constant cleanup on the settled graph, no-op compaction, orphan removal, dead-flop removal, strict post-remap duplicate protection, instance-input remapping during compaction, topological-order preservation, and the large-low-support semantic-merge budget guard.
- `src/bin/tool_matrix.rs` — 26 inline unit tests covering scenario-name uniqueness, full factorization-rung coverage, full construction-strategy coverage, coverage-gap detection, the Phase-1 / Phase-2 / Phase-3 / Phase-4 gate run-plan math, representative `share_prob`-sweep coverage, Phase-3 structured-surface coverage, the refreshed Phase-4 hierarchy coverage facts (wrapper and recursive depths, child-instance profiles, per-depth override profiles, reuse, under-instantiation, recursive fact derivation from `DesignMetrics`), design-level metrics/report embedding, design-level Yosys invocation shaping, legacy `.sv` bootstrap resume, same-binary generator-checkpoint resume for both module and design artifacts, `sv`-hash mismatch rejection, and legacy-checkpoint upgrade.
- `tests/pipeline.rs` — 35 integration tests covering cross-seed validity, reproducibility across strategies, motif sweeps, both constant- and variable-shift surfaces, the landed procedural case/casez/for-fold surfaces, the landed selectable `Slice` / `Concat` surface, the hierarchy surface (legacy depth-1 wrapper exact/reuse/under-instantiation plus bounded recursive tree shape and per-depth branching profiles), the first parent-side composition surface over child outputs, all live gate categories, zero-orphan / zero-duplicate-operand doctrine guards, input-surface finalisation, associative / constant-fold / peephole / compaction counters, and knob-roll telemetry.
- Current executed counts (`cargo test`, 2026-04-23): **204 unit-target tests + 35 integration tests = 239 passing tests**. Doc-tests: 0.
- No external Verilator / Yosys smoke tests are wired into `cargo test`
  yet. A repo-owned `tool_matrix` harness now exists for broader
  sweeps; the smoke matrix is green, the full current-code Phase 1
  gate is now closed via
  `/tmp/anvil-tool-matrix-phase1-real-r21/tool_matrix_report.json`
  (1005 modules, `coverage_gaps = []`, and 1005/0 pass-fail in
  Verilator plus both repo-owned Yosys modes), and the representative
  Phase 2 sharing gate is now closed via
  `/tmp/anvil-tool-matrix-phase2-share-r1/tool_matrix_report.json`
  (216 modules, `coverage_gaps = []`, and 216/0 pass-fail in Verilator
  plus both repo-owned Yosys modes), and the new dedicated Phase 3
  structured gate is now closed via
  `/tmp/anvil-tool-matrix-phase3-structured-r4/tool_matrix_report.json`
  (210 modules, `coverage_gaps = []`, and 210/0 pass-fail in Verilator
  plus both repo-owned Yosys modes). A first real hierarchy smoke now
  exists too at `/tmp/anvil-hierarchy-smoke-r1`, clean in Verilator,
  Yosys `synth -noabc`, and the repo-owned ABC path for a depth-1
  wrapper design. A focused current-code parent-composition smoke now
  exists too at `/tmp/anvil-hier-parent-compose-smoke-r1`, clean in the
  same three lanes while its manifest proves `top_parent_composed_outputs > 0`
  and `top_instance_output_dependency_fraction = 1.0`. The dedicated
  Phase 4 hierarchy gate is now closed too at `/tmp/anvil-tool-matrix-phase4-hierarchy-r9/tool_matrix_report.json`
  (60 designs, `coverage_gaps = []`, and 60/0 pass-fail in Verilator
  plus both repo-owned Yosys modes). That refreshed report now covers
  wrapper exact / reuse / under-instantiation plus recursive depth `2`,
  child-instance profiles `2`, `4`, `2:3`, and `1:3`, the per-depth
  override profile `0=4:4,1=2:2`, and real parent-side composition
  above instance outputs. The focused smokes at
  `/tmp/anvil-hier-reuse-smoke-r1`,
  `/tmp/anvil-hier-under-smoke-r2`,
  `/tmp/anvil-hier-range-smoke-r1`, and
  `/tmp/anvil-hier-depth-profile-smoke-r1` remain useful targeted
  proofs, while the old `/tmp/anvil-tool-matrix-phase4-hierarchy-r7`
  report is now the historical wrapper-baseline artifact and the
  aborted `r8` rerun is historical runtime evidence that the Phase 4
  gate should use a hierarchy-focused sequential leaf profile instead
  of silently borrowing the fattest Phase 1 leaf-stress shape.

## Known weaknesses (visible in code today)

- The broader signoff-grade cleanliness matrix described in
  `ROADMAP.md` now has a repo-owned implementation in
  `src/bin/tool_matrix.rs`, and the smoke matrix is currently green:
  15/15 clean in Verilator and 15/15 clean in Yosys. The harness now
  treats warnings as failures, so "green" here means no errors and no
  warnings, not merely zero non-zero exits. The repo-owned gate surface
  now also includes the dedicated `--phase2-share-gate`, whose
  normalized `share_sweep` summary proves that stronger `share_prob`
  increases the *fraction* of shared nodes even though the raw shared
  node count falls as the graph collapses.
- `NodeId`-as-identity is still conservative for state and does not yet
  extend to future hierarchical objects. Exact-signature duplicate
  flops merge; stronger sequential/hierarchical equivalence remains open
  work.
- Phase 4 is no longer only the first depth-1 slice. The legacy exact
  wrapper lane is still real, and the repo-owned Phase 4 gate now also
  banks the current representative bounded-recursive hierarchy surface.
  The next honest work is mixed-depth recursion, local parent state,
  on-demand child sourcing, and future hierarchy-aware identity.
- `emit::sv::render_gate` for `Concat` joins operand names with commas (correct SV); the IR does not currently distinguish per-operand widths in storage because every current producer of `Concat` either replicates a single source or concatenates uniform-width bits. When variadic `Concat` with mixed widths becomes a real motif, the IR shape is still adequate (widths are a property of each operand node, not of the `Concat` itself), but a generator-side helper will need to compose such shapes carefully.

## Build hygiene
- `cargo check --all-targets` — clean.
- `cargo test` — clean (239 passing tests: 173 lib + 5 main + 26 tool_matrix + 35 integration).
- `cargo build` — clean.
- `cargo clippy --all-targets -- -D warnings` — clean.
- `cargo fmt --all --check` — clean.
- `mdbook build book` — clean.
- Generator-output smoke: Verilator lint on seed 42 is clean with no warning-specific suppressions beyond the usual filename noise; the previous `UNSIGNED` / `CMPCONST` tautology residue is now folded away in the IR; a default + graph-first-alias seed sweep (0..4) is clean for `UNUSEDSIGNAL`; the live `seed=0 / interleaved / relaxed / none` repro (`mod_0_0006.sv`) is now clean in both Verilator and `yosys ... synth -noabc`; the built-in `tool_matrix` smoke run is 15/15 clean in Verilator and 15/15 clean in Yosys under `--yosys-mode without-abc`; a small `--yosys-mode both` probe is now clean in both Yosys sub-modes too (`without-abc = 15/15 pass`, `with-abc = 15/15 pass`) after moving the ABC-enabled harness path to `synth -noabc; abc -fast; opt -fast; stat; check`; a same-binary fast-resume smoke on `/tmp/anvil-tool-matrix-resume-fast-smoke-r1` now completes cleanly both on the initial run and on the immediate `--resume` rerun, and saved checkpoints now include `runtime_fingerprint`, `sv_hash`, and `generator_checkpoint`; a real baseline `tool_matrix --phase1-gate` rerun has now been pushed to **365 generated modules** with **0 Verilator warning logs** and **0 Yosys warning lines** across the saved stdout logs (67 clean each in `int_relaxed_none_default`, `int_nodeid_none_default`, `int_nodeid_cse_default`, `int_nodeid_operand-unique_default`, and `int_nodeid_commutative_default`, plus 30 clean in `int_nodeid_associative_default` before checkpoint); the historical current-code real both-mode `tool_matrix --phase1-gate --yosys-mode both` tree at `/tmp/anvil-tool-matrix-phase1-real-r20` still stands at **570 completed module checkpoints / 571 emitted `.sv` files** with the same zero-warning bar (67 clean each in `int_relaxed_none_default`, `int_nodeid_none_default`, `int_nodeid_cse_default`, `int_nodeid_operand-unique_default`, `int_nodeid_commutative_default`, `int_nodeid_associative_default`, `int_nodeid_constant-fold_default`, and `int_nodeid_peephole_default`, plus 34 clean in `int_nodeid_e-graph_default` before intentional checkpoint); the completed current-code real both-mode `tool_matrix --phase1-gate --yosys-mode both --resume` tree at `/tmp/anvil-tool-matrix-phase1-real-r21` now has a final `tool_matrix_report.json` with **1005** completed module checkpoints / **1005** emitted `.sv` files, `coverage_gaps = []`, and the same zero-warning bar (`Verilator 1005/0`, `Yosys without-abc 1005/0`, `Yosys with-abc 1005/0`), with all **1005** saved checkpoints on `r21` already carrying the fast-resume metadata; the completed Phase 2 share-sweep tree at `/tmp/anvil-tool-matrix-phase2-share-r1` now has a final `tool_matrix_report.json` with **216** completed module checkpoints / **216** emitted `.sv` files, `coverage_gaps = []`, and the same zero-warning bar (`Verilator 216/0`, `Yosys without-abc 216/0`, `Yosys with-abc 216/0`), while the normalized `share_sweep` summary proves the representative sharing knob sweep directly (`shared_node_fraction = 0.4122`, `0.4232`, `0.4386` at `share_prob = 0.0`, `0.3`, `0.9` respectively, alongside the expected node-count collapse); the completed Phase 3 structured-surface tree at `/tmp/anvil-tool-matrix-phase3-structured-r4` now has a final `tool_matrix_report.json` with **210** completed module checkpoints / **210** emitted `.sv` files, `coverage_gaps = []`, and the same zero-warning bar (`Verilator 210/0`, `Yosys without-abc 210/0`, `Yosys with-abc 210/0`); the dedicated Phase 4 wrapper hierarchy gate is now also closed at `/tmp/anvil-tool-matrix-phase4-hierarchy-r7/tool_matrix_report.json` with **48** completed designs / **48** emitted design checkpoints, `artifact_kind = "design"`, `coverage_gaps = []`, and the same zero-warning bar (`Verilator 48/0`, `Yosys without-abc 48/0`, `Yosys with-abc 48/0`); the older depth-1 hierarchy smoke at `/tmp/anvil-hierarchy-smoke-r1` remains clean in Verilator, Yosys `synth -noabc`, and the repo-owned ABC path for a four-module emitted design (three leaves plus one top wrapper); and the newer hierarchy-metrics smoke at `/tmp/anvil-hier-metrics-smoke-r1` is also clean in the same three lanes while proving trustworthy design-level composition metrics plus the exact control-port propagation rule (`top_clock_inputs = 1`, `top_reset_inputs = 1`, `clock_fanout_instances = 5`, `reset_fanout_instances = 5`). A real partial both-mode smoke run interrupted at 14/15 scenarios was then completed successfully on the same output tree under `--resume`, ending at 15/15 clean in Verilator and both Yosys sub-modes; the legacy `r11` both-mode frontier has now been upgraded in place through **143** module checkpoints with the same zero-warning bar (67 relaxed, 67 nodeid-none, and 9 cse); after the exact-proof budget fix, a focused current-code repro (`cargo run --bin anvil -- --seed 2 --count 10 --out /tmp/anvil-cse-seed2-repro-r1 --construction-strategy interleaved --identity-mode node-id --factorization-level cse`) now emits all 10 modules cleanly, with `fails=0` / `warns=0` under Verilator, Yosys `synth -noabc`, and the repo-owned ABC-enabled Yosys path; the stricter cleanup-proof eligibility cap is now also proven on the old `nodeid-cse` stall boundary (`cargo run --bin anvil -- --seed 2 --count 2 --out /tmp/anvil-cse-seed2-repro-r2 --construction-strategy interleaved --identity-mode node-id --factorization-level cse`), which emits both modules cleanly under Verilator plus both repo-owned Yosys modes; the next support-cap refinement is also now proven on the old `operand-unique` stall boundary (`cargo run --bin anvil -- --seed 3 --count 21 --out /tmp/anvil-operand-unique-seed3-repro-r1 --construction-strategy interleaved --identity-mode node-id --factorization-level operand-unique`), which emits all 21 modules cleanly through `mod_3_0020.sv`, and all 21 are warning-clean in Verilator plus both repo-owned Yosys modes; the next warning fix is now also proven on the first fresh-current-code `nodeid-none` boundary (`cargo run --bin anvil -- --seed 1 --count 23 --out /tmp/anvil-nodeid-none-seed1-repro-r1 --construction-strategy interleaved --identity-mode node-id --factorization-level none`), which emits through `mod_1_0022.sv`, and all 23 modules are warning-clean in Verilator plus both repo-owned Yosys modes; the old associative warning boundary is now also proven clean on focused current-code repros (`cargo run --bin anvil -- --seed 5 --count 12 --out /tmp/anvil-associative-seed5-repro-r1 --construction-strategy interleaved --identity-mode node-id --factorization-level associative` and `cargo run --bin anvil -- --seed 5 --count 16 --out /tmp/anvil-associative-seed5-repro-r8 --construction-strategy interleaved --identity-mode node-id --factorization-level associative`), where the checked modules are warning-clean in Verilator plus both repo-owned Yosys modes; and the older `/tmp/anvil-tool-matrix-phase1-real-r16` and `/tmp/anvil-tool-matrix-phase1-real-r12` trees remain historical evidence only across older code.
- `src/gen/cone.rs` now owns an always-on generator-side comparison
  proof in addition to the factorization ladder. The proof combines a
  conservative unsigned-bounds engine with an exact finite-set engine
  for comparison operands up to 8 bits wide, and it is used in every
  comparison-emission path (recursive, interleaved, pool-only, and
  constant-comparand helpers). The exact-proof helpers now also
  short-circuit on absorbing / saturating exact prefixes and duplicate
  XOR parity, so small-width exact results are not lost just because an
  irrelevant tail depends on a wider cone. That exact finite-set engine
  is now explicitly budgeted, memoizes both exact and unknown results,
  and is further capped to small-support cones (current cap: 3
  canonical leaf endpoints), so the proof remains useful on narrow
  cones without turning into a runtime trap on correlation-heavy shared
  cartesian searches. The cheap exact/bounds layer also now carries the
  reflexive arithmetic identity `x - x = 0`, because that fact must
  stay available even when the heavier finite-set prover declines a
  cone. This is an enforced output-cleanliness invariant, not a user
  knob.
