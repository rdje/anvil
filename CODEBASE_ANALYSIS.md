# Code Base Analysis
Live analysis of the Rust workspace as it currently stands. Updated whenever a slice materially changes the workspace.

## Snapshot
- **Workspace:** single crate `anvil` (no Cargo workspace; flat layout).
- **Edition:** 2021.
- **Targets:** three binaries (`anvil` as Cargo's default run target; the auxiliary `tool_matrix` harness; and `anvil-mcp` — the read-only agent MCP server, `AGENT-INTROSPECTION-MCP.4`, an explicit `[[bin]]`), one library (`anvil`), one example (`generate_one`), six integration tests (`pipeline`; `book_examples` — the mdBook copy-paste-runnable gate; `snapshots` — the `insta` byte-identical-reproducibility guard, `INSTA-SNAPSHOTS.1`; `diff_sim` — the `DIFFERENTIAL-SIMULATION` cross-simulator harness proofs; `microdesign_parity` — the Phase-7 oracle-vs-tool parity proofs; `frontend_parity` — the Phase-8 frontend/elaboration parity proofs). The last three each also carry a tool-gated `#[ignore]` end-to-end gate that runs only when the real downstream tool is present.
- **External deps:** `rand`, `rand_chacha`, `clap`, `serde`, `serde_json`, `thiserror`, `anyhow`, `tracing`, `tracing-subscriber`. `insta` (dev) reserved for snapshot tests. `tracing` carries `release_max_level_info` so trace-level calls compile out in release.
- **MSRV:** pinned to Rust 1.95 via `Cargo.toml` `rust-version = "1.95"`.
- **Package description:** `Cargo.toml` describes ANVIL as a random by-construction generator of synthesizable SystemVerilog RTL; do not use SV/UVM-style constrained-random terminology for the crate purpose.

## Suitability assessment against the product goal

Short answer: **yes as a foundation, not yet as a completed generator**.

The current architecture is well matched to ANVIL's direction:

- typed IR construction instead of grammar/text emission;
- one combinational identity chokepoint in `ir/types.rs`;
- post-drain state and reachability finalisation in `ir/compact.rs`;
- validator-owned invariants in `ir/validate.rs`;
- explicit knob/control plumbing in `config.rs`; and
- a deliberately dumb SV emitter in `emit/sv.rs`.

That is the right base for a random by-construction, signoff-grade
synthesizable RTL generator. In this terminology, Verilator and Yosys
are validation tools used by this repository to check generated HDL
acceptance; the generated artifacts themselves target the broader class
of downstream HDL consumers such as parsers, elaborators, RTL compilers,
linters, simulators, and synthesis tools. The work still required falls
into four explicit gaps:

1. **Feature breadth / legal surface area / artifact-family breadth**
   The active generator is still grounded in the Phase 1/2/3
   leaf-module kernel, but it is no longer leaf-module-only. The
   previously explicit Phase 3 breadth gaps (`case`, `casez`, variable
   shifts, generic selectable `Slice` / `Concat`, bounded unrolled
   logic) are now landed, the dedicated Phase 3 structured-surface
   closure gate is landed, and Phase 4 hierarchy now has real depth-1
   and bounded recursive lanes with child sourcing, parent-side
   composition, registered routing, helper instances, and measurable
   design metrics. Parameterization, aggregates, memories, FSMs, and
   the multi-artifact lanes are real delivered surfaces. Beyond that,
   the newer user direction broadens the target beyond one output
   family: ANVIL now has separate DUT, oracle-backed micro-design, and
   frontend/elaboration accept artifact lanes. Broader
   identity/factorization strengthening has now been audited through
   closed post-phase task trees in `docs/TASK_TREE.md`:
   `COMBINATIONAL-SEMANTIC-IDENTITY`,
   `SEQUENTIAL-COINDUCTIVE-IDENTITY`, `MEMORY-STATE-IDENTITY`, and
   `HIERARCHY-SEMANTIC-IDENTITY`.
2. **`NodeId`-as-identity is only partially realized**
   `Module::intern_gate` gives a strong combinational canonicalization
   chokepoint. The bounded `e-graph` fragment now merges
   different-shape same-endpoint gate cones and can fold a proven gate
   to an earlier endpoint/constant when irrelevant helper endpoints
   cancel out. `merge_equivalent_flops` handles endpoint-preserving
   duplicate flop state, and `merge_equivalent_fsms` now handles
   deterministic duplicate generated FSM blocks. But "same expression
   anywhere in the cone forest means same `NodeId`" is still not fully
   true for broader sequential equivalence, memory-state merging beyond
   the current instance-local memory boundary, or future hierarchical
   objects.
3. **Downstream-acceptance confidence still needs broader automation beyond current phase gates**
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
   `share_prob ∈ {0.0, 0.3, 0.9}`. The Phase 3 structured-surface gate
   is also closed at `/tmp/anvil-tool-matrix-phase3-structured-r4`,
   and the latest full downstream-clean Phase 4 hierarchy gate is
   closed at `/tmp/anvil-tool-matrix-phase4-hierarchy-r87` with 840/0
   in Verilator plus both repo-owned Yosys modes. `SIGNOFF-SURFACE-EXPANSION.3`
   also adds an opt-in Icarus Verilog compile/elaboration column to
   `tool_matrix`; the focused current-code smoke at
   `/tmp/anvil-signoff-surface-iverilog-r1/tool_matrix_report.json`
   is clean at 17/0 in Verilator, 17/0 in Yosys without ABC, 17/0 in
   Yosys with ABC, and 17/0 in Icarus compile. So closure evidence
   now exists for the current Phase 1-4 surfaces; the remaining confidence
   gap is broader validation automation for future phases, richer knob
   sweeps, and the larger artifact-family space implied by the
   signoff-grade goal. The first richer-knob-sweep increment landed as
   `SIGNOFF-AUTOMATION-EXPANSION.2b`: `tool_matrix` gains a new
   `ScenarioSet::SignoffKnobSweep` + the opt-in `--signoff-knob-sweep-gate`
   flag + `build_signoff_knob_sweep_scenarios` (four focused
   single-knob scenarios × three construction strategies) + four `saw_*`
   coverage facts, backed by a new post-hoc `Metrics` field
   `num_operator_gates_with_duplicate_operands` in `src/metrics.rs`
   (RTL byte-identical — metrics are never emitted). It promotes four
   previously-unswept knobs (`operand_duplication_rate`,
   `mux_arm_duplication_rate`, `aggregate_array_prob`, memory×fsm
   interplay) into explicit axes, banked downstream-clean at
   `/tmp/anvil-signoff-knob-sweep-r1` (12 scenarios, 48 modules,
   `coverage_gaps = []`, `48/0` Verilator + both Yosys). The focused
   gate's `compute_coverage_gaps` arm early-returns after the four facts,
   so it does not inherit the broad-motif richness the phase gates
   require. The per-version acceptance gate landed the same way as
   `SV-VERSION-TARGETING.2b.2b`: `tool_matrix` gains a new
   `ScenarioSet::SvVersionSweep` + the opt-in `--sv-version-gate` flag +
   `build_sv_version_sweep_scenarios` (per IEEE 1800 target ×
   {comb leaf, seq leaf, recursive hierarchy design} = 9 `Interleaved`
   scenarios), each carrying `Config::sv_version` and emitted via the
   versioned `to_sv_versioned` / `to_sv_in_design_versioned` entry points;
   `verilator_language_for` runs Verilator in the matching
   `--language 1800-20xx` mode (via the `.2b.2a` `run_verilator(_design)`
   selector) only under the gate; four `saw_sv_version_*_targeted_acceptance`
   coverage facts (lit by `light_sv_version_acceptance` from
   `summarize_{coverage,design_coverage}` when Verilator ran-and-succeeded
   + a non-empty, clean Yosys vec) are enforced by an early-return arm in
   `compute_coverage_gaps` *before* the construction-strategy loop (so an
   Interleaved-only sweep is valid). `SV-VERSION-TARGETING.3b.2b` adds a
   tenth scenario, `soft_union_upopt_config` (slice-heavy, 2023-targeted,
   `soft_union_slice_prob = 1.0`), that genuinely emits the IEEE 1800-2023
   `union soft` overlay. Because Yosys/Icarus reject the syntax, such a
   scenario is detected by `scenario_emits_soft_union_overlay` and threaded
   as `verilator_only` through `run_module_scenario` →
   `resume_existing_module` / `materialize_prepared_module` →
   `run_module_tools` (Yosys vec empty + Icarus `None` — a recorded
   no-op); `ModuleReport.emitted_soft_union_overlay` (from the emitted SV
   text) gives `summarize_coverage` honest evidence to light the dedicated
   `saw_sv_version_2023_soft_union_upopt` fact (Verilator-only — never
   Yosys), enforced by `compute_coverage_gaps`. `MatrixReport.sv_version_gate`
   records the run. Banked downstream-clean at
   `/tmp/anvil-sv-version-gate-upopt-r1` (10 scenarios, 20 units,
   `coverage_gaps = []`, Verilator `20/0`, Yosys `18/0` both modes — the
   up-opt scenario is the Yosys no-op). Default matrix runs stay
   byte-identical (selector `None`, `Sv2012`-floor emits).
   The combinational `function automatic` emit gate landed the same way as
   `STRUCTURED-EMISSION-EXPANSION.2b.2b`: `tool_matrix` gains a new
   `ScenarioSet::FunctionEmitSweep` + the opt-in `--function-emit-gate`
   flag + `build_function_emit_sweep_scenarios` (one comb-only
   `function_emit_focus_config` DUT × three construction strategies, all
   forcing `Config::function_emit_prob = 1.0`) + the
   `saw_combinational_function_emit` coverage fact + a new
   `ModuleReport.emitted_combinational_function` field. Detection mirrors
   the `union soft` precedent — `materialize_prepared_module` sets the
   field from `prepared.sv_text.contains("function automatic")`, and
   `summarize_coverage` lights the fact when an emitted module is accepted
   by Verilator **and** Yosys (a synthesizable function is universally
   accepted, so — unlike the Verilator-only `union soft` up-opt — the gate
   runs the full tool plan; Icarus, when `--iverilog-compile` is set, is
   enforced via the `ToolSummary::any_failed` bail). `compute_coverage_gaps`
   early-returns after the one fact (plus the universal
   construction-strategy coverage), so it does not inherit the broad-motif
   richness the phase gates require. `MatrixReport.function_emit_gate`
   records the run. Banked downstream-clean at
   `/tmp/anvil-function-emit-gate-r1` (3 scenarios, 12 modules, 608
   emitted functions, `coverage_gaps = []`, `12/0` Verilator + both Yosys
   modes + Icarus compile). Default `function_emit_prob = 0.0` emission
   stays byte-identical.
   The `generate for` loop emit gate landed the same way as
   `STRUCTURED-EMISSION-EXPANSION.4b.2b`: a new
   `ScenarioSet::GenerateLoopSweep` + the opt-in `--generate-loop-gate` flag
   + `build_generate_loop_sweep_scenarios` (one comb-only
   `generate_loop_focus_config` DUT × three construction strategies, all
   forcing `Config::generate_loop_emit_prob = 1.0`) + the
   `saw_generate_loop_emit` coverage fact + a new
   `ModuleReport.emitted_generate_loop` field (set from
   `prepared.sv_text.contains("generate")`); `summarize_coverage` lights the
   fact on Verilator + Yosys acceptance (a `generate for` is universally
   synthesizable, like a function), and `compute_coverage_gaps` early-returns
   after the one fact. `MatrixReport.generate_loop_gate` records the run.
   Banked downstream-clean at `/tmp/anvil-generate-loop-gate-r1` (3
   scenarios, 12 modules, 8 emitting a loop, `coverage_gaps = []`, `12/0`
   Verilator + both Yosys modes + Icarus compile). Default
   `generate_loop_emit_prob = 0.0` emission stays byte-identical.
4. **The IR is optimized for structural legitimacy more than semantic
   richness today**
   That matches the project doctrine: whole-module intended behavior is
   usually arbitrary. The missing work is therefore not "add a
   bundled spec/oracle layer", but "add more legal, synthesizable,
   interaction-rich motifs, composition surfaces, and explicit
   expected-facts manifests where a particular artifact family needs
   them".

Taken literally against the user's `rtl_const_expr` / `rtl_frontend`
style request, the repo is now ready for the first delivered slices of
those artifact families. Phase 7 provides the oracle-backed
const-expression micro-design lane, Phase 8 provides the source-level
frontend/elaboration accept lane, and Phase 9 exposes all three lanes
through `--artifact <dut|microdesign|frontend>`. The immediate
five-tree post-phase follow-up batch is now exhausted to its current
proof/tool boundaries: combinational, sequential, memory, and hierarchy
identity are either landed or explicitly bounded, and the signoff
surface has richer CDC, Verilator JSON frontend parity, and Icarus
compile acceptance. Future work is not "create those lanes" anymore;
it must open new task-tree leaves for deeper signoff sweeps, broader
source-language constructs, or new proof domains.

## Module map

```
src/
├── main.rs           CLI entry point. Parses `Cli`, loads/merges
│                     `Config`, validates it, runs `Generator`, and
│                     writes stdout or per-file output with
│                     `manifest.json`. The directory-output manifest is
│                     written by `manifest::write_streamed_manifest`
│                     element-by-element (generate → emit `.sv` → build
│                     one metadata Value → stream → drop), so peak
│                     metadata memory is O(1) in `--count` rather than
│                     O(`--count`) (`WORKLOAD-MEMORY-SAFETY.2`). CLI surface covers structure,
│                     sequential motifs, comb-mux / priority-encoder /
│                     coefficient / constant-shift / const-comparand
│                     motifs, construction strategy, factorization,
│                     tracing, metrics, and the opt-in `--introspect`
│                     agent-introspection dump (`AGENT-INTROSPECTION-MCP.3`,
│                     `src/introspect/`; single-artifact stdout only,
│                     default-off ⇒ byte-identical). Tracing:
│                     `init_tracing` wires a deterministic subscriber
│                     from `--trace <level>` + `--trace-file`;
│                     `TraceLevel` maps `low=INFO`, `medium=DEBUG`,
│                     `high/debug=TRACE`, with `debug` additionally
│                     enabling `trace_verbose!`.
│
├── lib.rs            Public surface: re-exports Config, Generator, Module.
│                     Also exposes the `metrics` and `manifest` modules. Trace
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
│                     per-definition instantiation histograms, and
│                     parent-output helper routes through parent-local
│                     flops, plus stateful parent-port-composed
│                     parent-output support.
│                     Serde-serializable; embedded in `manifest.json`
│                     and printed with the `--metrics` flag.
│
├── manifest.rs       Streaming `manifest.json` writer
│                     (`WORKLOAD-MEMORY-SAFETY.2`).
│                     `write_streamed_manifest(w, scalars, array_key,
│                     elements)` writes a top-level
│                     `{ <scalars…>, "<array_key>": [ … ] }` object,
│                     streaming the array element-by-element so only one
│                     element Value is live at a time (peak metadata
│                     memory O(1) in artifact count). Byte-identical to
│                     the previous accumulate-then-`to_string_pretty`
│                     path: the surrounding framing is derived from serde
│                     via a placeholder split and each element is
│                     serde-pretty-printed then re-indented by its base
│                     depth. `streamed_matches_reference` proves the
│                     byte-identity against serde itself.
│
├── mem_guard.rs      Opt-in internal RAM/RSS self-governor
│                     (`WORKLOAD-MEMORY-SAFETY.4`). Pure decision
│                     `evaluate(&MemLimits, &MemSample) → Option<AbortReason>`
│                     (RSS before host-%; disabled/`None` never trips)
│                     + best-effort dep-free OS reads mirroring
│                     `scripts/ram_guard.sh`: `read_process_rss_mb`
│                     (Linux `/proc/self/status` VmRSS; macOS `ps`) and
│                     `read_host_used_pct` (Linux `/proc/meminfo`; macOS
│                     `memory_pressure`). `MemGuard::from_config` (or
│                     `from_limits`, used by `downstream::validate`,
│                     `AGENT-INTROSPECTION-MCP.5.2`) / `check()`
│                     short-circuits to `None` when both knobs are off, so
│                     the default `--out` loop is byte-identical and draws
│                     RNG identically. `main.rs` checks it BETWEEN units in
│                     the streaming closures (decline-to-start-more, never
│                     mid-cone) and exits `99` with a seed+knobs message on a
│                     trip. Process-safety governor, not a generation knob:
│                     never alters emitted RTL.
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
│                     SvVersion enum (clap::ValueEnum + serde, derives
│                     PartialOrd/Ord; bare-year value spelling
│                     "2012"/"2017"/"2023"): the opt-in --sv-version
│                     emission-target capability gate (sv_version knob,
│                     #[serde(default)] = Sv2012 floor). permits() is the
│                     down-gating capability bound; ieee_standard() →
│                     "1800-20xx" for the per-version downstream axis
│                     (SV-VERSION-TARGETING.2b, decision 0009). The first
│                     up-opt (.3b.2a, decision 0010) adds the default-off
│                     `soft_union_slice_prob` knob (serde-only, validated
│                     0.0..=1.0): when > 0.0 AND sv_version permits 2023,
│                     proper low-bits Slice gates render as an internal
│                     `union soft` overlay (see `ir/soft_union.rs`).
│
├── ir/soft_union.rs  SV-VERSION-TARGETING.3b.2a first up-opt. Gen-time
│                     `annotate_soft_union_slices(m, rng, prob)` pass
│                     (rolled at the `gen/mod.rs` call site like
│                     `aggregate_prob`; param-env modules skipped) marks
│                     proper low-bits `GateOp::Slice{lo:0}` gates (over a
│                     non-constant, strictly-wider source) into the new
│                     emitter-surface `Module.soft_union_slice_gates`
│                     (BTreeSet<NodeId>, not hashed into identity). The
│                     emitter (`emit/sv.rs::soft_union_slice_overlay`)
│                     realizes the IEEE-1800-2023 `union soft` overlay
│                     (`u.w = src; gate = u.n`) ONLY under
│                     `SvVersion::permits(Sv2023)`; below 2023 it
│                     down-gates to the plain `src[hi:0]`. Behaviour-
│                     preserving (LSB-aligned members), default-off
│                     byte-identical (snapshots 6/6). Banked Verilator
│                     `--language 1800-2023` clean
│                     (`tests/sv_version_downstream.rs`); Yosys/Icarus
│                     reject the syntax → recorded no-op.
│
├── ir/function_emit.rs  STRUCTURED-EMISSION-EXPANSION.2b.1 — the first
│                     richer-structured emit surface (decision 0012).
│                     Gen-time `annotate_function_emit_gates(m, rng, prob)`
│                     pass (rolled at the `gen/mod.rs` call site like
│                     `soft_union`, after it; param-env modules skipped)
│                     marks *computational* `Node::Gate`s — not structured
│                     (CaseMux/CasezMux/ForFold), **not `Slice`**, ≥1
│                     operand, not already `union soft`-marked — into the
│                     new emitter-surface `Module.function_emit_gates`
│                     (BTreeSet<NodeId>, not hashed into identity, disjoint
│                     from `soft_union_slice_gates`). The emitter
│                     (`emit/sv.rs::function_emit_gate` +
│                     `render_gate_function_decl`/`_body`/`_call`) renders
│                     each marked gate as a behaviour-preserving
│                     `function automatic logic[W-1:0] <wire>__f(positional
│                     params)` decl + a `assign <wire> = <wire>__f(...)`
│                     call. Default-off (`function_emit_prob == 0.0`)
│                     byte-identical (snapshots 6/6). `Slice` excluded
│                     because a bit-select uses only a sub-range of its
│                     operand (a full-width param trips `-Wall
│                     UNUSEDSIGNAL`); still emitted inline (nothing
│                     retired), slice-aware projection = follow-up. Forced
│                     `function_emit_prob=1.0` sweep clean across Verilator
│                     `--lint-only` + Yosys both modes + Icarus
│                     (`/tmp/anvil-fe-r2/`). Repo-owned gate + coverage
│                     fact = `.2b.2`.
│
├── ir/generate_loop.rs  STRUCTURED-EMISSION-EXPANSION.4b.1 — the second
│                     richer-structured emit surface (decision 0013).
│                     Gen-time `annotate_generate_loop_gates(m, rng, prob)`
│                     pass (rolled at the `gen/mod.rs` call site like
│                     `function_emit`, after it; param-env modules skipped)
│                     marks `{N{x}}` replication `Node::Gate`s — a
│                     `GateOp::Concat` of ≥2 operands all the same `NodeId`,
│                     **1-bit lane** (⇒ result width == N), not already
│                     function-emit-marked (nor `union soft`-marked) — into
│                     the new emitter-surface `Module.generate_loop_gates`
│                     (BTreeSet<NodeId>, not hashed into identity, disjoint
│                     from `function_emit_gates`). The emitter
│                     (`emit/sv.rs::generate_loop_gate` +
│                     `render_generate_loop_block`) renders each marked gate
│                     as a behaviour-preserving single-level `genvar
│                     <wire>__gi; generate for (gi=0; gi<N; gi=gi+1) begin :
│                     <wire>__gen assign <wire>[gi] = <x>; end endgenerate`
│                     and suppresses the inline `assign <wire> = {N{x}};`.
│                     The unrolled loop is byte-equivalent to the inline
│                     replication. Default-off (`generate_loop_emit_prob ==
│                     0.0`) byte-identical (snapshots 6/6). Wider-lane
│                     part-select = recorded follow-up (still emitted inline,
│                     nothing retired). Forced `generate_loop_emit_prob=1.0`
│                     sweep clean across Verilator `--lint-only` (+`-Wall`
│                     Δ=0 vs OFF) + Yosys both modes + Icarus
│                     (`/tmp/anvil-gl-r1/`). Repo-owned gate + coverage fact
│                     = `.4b.2`.
│
├── ir/task_emit.rs  STRUCTURED-EMISSION-EXPANSION.6b.1 — the third
│                     richer-structured emit surface (decision 0014).
│                     Gen-time `annotate_task_emit_gates(m, rng, prob)`
│                     pass (rolled at the `gen/mod.rs` call site like
│                     `function_emit`/`generate_loop`, after both;
│                     param-env modules skipped) marks combinational
│                     `Node::Gate`s — the same candidate set as
│                     `function_emit` (non-structured, non-`Slice`, ≥1
│                     operand) that is not already marked for the
│                     `function_emit` / `generate_loop` / `soft_union`
│                     projections — into the new emitter-surface
│                     `Module.task_emit_gates` (BTreeSet<NodeId>, not hashed
│                     into identity, disjoint from the sibling gate sets).
│                     The emitter (`emit/sv.rs::task_emit_gate` +
│                     `render_gate_task_decl` + `render_gate_task_call`,
│                     reusing `render_gate_function_body` for the body)
│                     renders each marked gate as a behaviour-preserving
│                     combinational `task automatic <wire>__t(output logic
│                     [W-1:0] o, input ...); o = <op over params>; endtask`
│                     + `logic [W-1:0] <wire>__tv; always_comb
│                     <wire>__t(<wire>__tv, <refs>);` and rewrites the gate's
│                     assign to the passthrough `assign <wire> =
│                     <wire>__tv;`. The decision 0012 single-gate
│                     `function automatic` parallel, but a *procedural* task
│                     called from `always_comb`. Default-off
│                     (`task_emit_prob == 0.0`) byte-identical (snapshots
│                     6/6). Forced `task_emit_prob=1.0` sweep clean across
│                     Verilator `--lint-only` (+`-Wall` Δ=0 vs OFF) + Yosys
│                     both modes + Icarus (`/tmp/anvil-te-r1/`). Metric +
│                     repo-owned gate + coverage fact = `.6b.2`.
│
├── microdesign/      Phase 7 oracle-backed micro-design lane
│   └── mod.rs        (`PHASE-7-ORACLE-MICRODESIGN`). A **separate
│                     generator path** from the DUT lane, NOT threaded
│                     through `ir`: a source-level const-expr /
│                     parameter dependency-DAG IR (`ConstExpr`,
│                     `ParamDecl`, `ConstExprUnit`) + the
│                     construction-time `eval`/`resolve` evaluator (the
│                     oracle: every `ParamDecl.value` resolved once at
│                     build time) + the reproducible rules-first
│                     `build_constexpr_unit(seed,n)` (ChaCha8). Plus
│                     (`.2b`) the un-resolved SV emitter (`emit_sv` —
│                     `rtl_const_expr` family) + the JSON
│                     expected-facts manifest emitter
│                     (`emit_manifest`/`Manifest`), both from the same
│                     oracle. Parity harness + repo-owned gate are
│                     `.2c`. Never invoked by the DUT path ⇒ DUT lane
│                     byte-identical (Phase 9 wires the selector).
├── frontend/        Phase 8 source-level frontend/elaboration accept
│   └── mod.rs        lane (`PHASE-8-FRONTEND-ACCEPT`). A **separate
│                     generator path** from the DUT lane: a source-level
│                     AST IR (`SourceUnit`/`Package`/`Module`/`Instance`/
│                     `GenerateIf`/`ParamDecl`/`ParamBinding`) +
│                     construction-time `elaborate()` evaluator (the
│                     oracle) + un-elaborated `emit_sv` + elaborated-facts
│                     `emit_manifest` + a hierarchy-aware parity comparator.
│                     Reuses Phase 7's `ConstExpr`/`eval`/`expr_to_sv`.
│                     Never invoked by the DUT path ⇒ DUT lane
│                     byte-identical.
├── umbrella/        Phase 9 multi-artifact selector + shared plumbing
│   └── mod.rs        (`PHASE-9-MULTI-ARTIFACT-UMBRELLA`). `pub trait
│                     ArtifactLane` + `LaneArtifact` carrier + `CheckPlan`
│                     enum + `LaneError` + `DutLane`/`MicrodesignLane`/
│                     `FrontendLane` impls. Only the plumbing unifies
│                     (seed→artifact, byte-stable output, optional
│                     manifest, downstream check plan); the three lanes'
│                     rules-first generators stay decoupled. `main.rs`
│                     selects via `--artifact <dut|microdesign|frontend>`
│                     (default `dut`, byte-identical to the historical
│                     no-flag path).
├── diff_sim/        Cross-simulator semantic-agreement harness core
│   └── mod.rs        (`DIFFERENTIAL-SIMULATION`). Used by
│                     `bin/tool_matrix.rs`'s opt-in `--diff-sim` column:
│                     normalizes fixed-width-hex traces from iverilog +
│                     verilator and byte-compares them, proving emitted SV
│                     is *semantically equivalent* across two independent
│                     simulators, not merely accepted.
├── downstream/      Hardened downstream-tool invocation surface
│   └── mod.rs        (`AGENT-INTROSPECTION-MCP.5.1`). The single source of
│                     truth for the acceptance-tool command lines:
│                     `run_verilator(_design)` (`--lint-only`, plus an
│                     optional `--language 1800-20xx` per-version selector —
│                     `None` = today's exact argv, byte-identical;
│                     SV-VERSION-TARGETING.2b.2a),
│                     `run_yosys(_design)` + `yosys_invocations(_design)`
│                     (`synth` scripts, `YosysMode` without/with-abc/both),
│                     `run_iverilog_compile(_design)` + the `*_argv` builders
│                     (`-g2012`), the spawn core `run_tool` +
│                     `first_tool_warning` (warning-as-failure), the
│                     `ToolInvocation` report row, `yosys_mode_slug`, and the
│                     double-quote escapers. Extracted verbatim from
│                     `bin/tool_matrix.rs` (which now `use`s them) so the
│                     `.5.2`/`.5.3` agent `validate`/`minimize` tools reuse the
│                     existing hardened invocations instead of forking a second
│                     source of truth (the `diff_sim` full-factorization
│                     pattern). Behavior-preserving: serialized
│                     `ToolInvocation` shape unchanged ⇒ banked matrix reports +
│                     `--resume` checkpoints stay valid.
│                     `.5.2` adds the controlled `validate(seed, &Config,
│                     &ValidateOptions) -> ValidateReport` orchestration here:
│                     generate the DUT artifact deterministically into a fresh
│                     per-run sandbox (`<root>/anvil-validate-<run_id>/`), run
│                     the selected vetted tools (the `AcceptanceTool` allow-list
│                     — fixed binary names), ram-guard via `MemGuard` before each
│                     spawn (decline-to-start-more), and return per-tool
│                     `ToolInvocation`s + an overall verdict. No arbitrary shell,
│                     no agent-supplied path. Reuses `introspect::content_run_id`
│                     for the shared content address.
│                     `.5.3` adds the controlled `minimize(seed, &Config,
│                     &MinimizeOptions) -> MinimizeReport` delta-debugger here: a
│                     deterministic coordinate-descent (`search_minimal`, generic
│                     over the failure oracle) that bisects integer size bounds
│                     toward their floors and drives optional-motif probabilities
│                     to `0.0`, to a fixpoint, using `validate` as a pure failure
│                     oracle (seed held fixed; every candidate re-checked with
│                     `Config::validate` before the generator; hard-capped by
│                     `max_oracle_calls`). Reports `reduced` knobs +
│                     `final_validation` (the surviving failure).
├── introspect/      Agent-introspection emission surface
│   ├── mod.rs        (`AGENT-INTROSPECTION-MCP.3`). Builds the versioned
│                     introspection document specified in
│                     `docs/AGENT_INTROSPECTION_SCHEMA.md` out of facts ANVIL
│                     already records (`Config` / `Metrics` /
│                     `DesignMetrics`). Invariant SCHEMA-DERIVED: zero new
│                     computed truth — every payload field is a serde
│                     projection of an existing struct; the new fields are
│                     only the envelope metadata (`schema_version` `"1.9"`
│                     — additive MINOR bumps: 1.2→1.3 derived-relation
│                     analyze surface, 1.3→1.4 `DesignMetrics` sequential
│                     proof-signature fields, 1.4→1.5/1.6/1.7 the
│                     `input_reach` / `flop_reset_provenance` /
│                     `module_reachability` analyze query kinds,
│                     1.7→1.8 the `Metrics::num_emitted_combinational_functions`
│                     count (`STRUCTURED-EMISSION-EXPANSION.2b.2a`), and
│                     1.8→1.9 the `Metrics::num_emitted_generate_loops`
│                     count (`STRUCTURED-EMISSION-EXPANSION.4b.2a`);
│                     the default introspection-document shape is unchanged.
│                     The sibling `DerivedAnalysisDocument` +
│                     `derived_analysis_document` builder reuse this envelope
│                     with an `analysis` payload),
│                     `anvil_version`, `lane`, the `request` echo with a
│                     content-addressed FNV-1a `run_id`, the `artifact`
│                     `ResourceRef`s, `warnings`). Pure `module_document` /
│                     `design_document` builders; read-only and additive,
│                     reached only via the default-off `--introspect` CLI flag
│                     on a single-artifact stdout run (rejects `--out` /
│                     `--count > 1`), so the default build + the streamed
│                     `--out` path stay byte-identical. `coverage` + the
│                     `microdesign`/`frontend` lane-manifest sections are
│                     deferred (matrix-only / `.4`+). `content_run_id` is `pub`
│                     (`.5.2`) so `validate` shares the one content address.
│   └── analyze.rs    (`SEMANTIC-INTROSPECTION-EXPANSION.2b.1`). The pure
│                     derived-RELATION analysis core (decision `0011`): the
│                     output **support cone**. `DerivedAnalysis { query,
│                     results: Vec<SupportCone> }` + `SupportCone { target,
│                     support_inputs[], support_flops[],
│                     support_instance_outputs[], cone_nodes, cone_depth }`
│                     (serde + `Default`; `BTreeSet` → sorted `Vec` ⇒
│                     deterministic). `module_support_cones(&Module, …)` /
│                     `design_support_cones(&Design, …)` do a memoized
│                     **combinational** fan-in DFS over the existing IR graph
│                     (`nodes` operands + `drives`): `Gate` recurses; `FlopQ`
│                     is a register-boundary support leaf (cone feeding `D` is
│                     the separate target `"flop:<id>"`); `InstanceOutput`
│                     stops at the instance boundary; `Constant` counts but is
│                     no support source; opaque `MemRead`/`FsmOut` terminate
│                     the cone. `target=None` ⇒ all outputs; unknown target ⇒
│                     no cone (the `.2b.2` MCP `analyze` tool maps that to
│                     `-32602`). Invariant SCHEMA-DERIVED: no IR field, no
│                     generator change — DUT byte-identical (not wired to any
│                     emit path). 9 in-crate cone-correctness proofs.
│                     `SEMANTIC-INTROSPECTION-EXPANSION.3b.1` adds the dual
│                     `input_reach` core: `QUERY_INPUT_REACH` + `ReachResult {
│                     target, reaches_outputs[], reaches_flops[], fanout_targets
│                     }` + a **second** `DerivedAnalysis.reach_results:
│                     Vec<ReachResult>` field (`#[serde(default,
│                     skip_serializing_if)]` ⇒ `output_support` docs stay
│                     byte-identical) + `module_input_reach`/`design_input_reach`,
│                     which **invert** the support relation (build every target's
│                     cone, bucket `T` under each `X ∈ support(T)`) ⇒ dual-consistency
│                     by construction. `target=None` ⇒ all sources (inputs, flop
│                     Qs, instance outputs); `"flop:<id>"` source = the Q's
│                     fan-out (direction set by the query kind); unknown source ⇒
│                     no result. `.3b.2` registers `input_reach` in
│                     `supported_query_kinds()` together with the `run_analyze`
│                     dispatch and bumps the schema `1.4 → 1.5`. 7 in-crate reach
│                     proofs (the transpose of the cone proofs). DUT byte-identical.
│                     `.4b.1` adds the third query core `flop_reset_provenance`:
│                     `QUERY_FLOP_RESET_PROVENANCE` + `FlopProvenance { flop,
│                     width, has_reset, reset_kind, reset_value (decimal string),
│                     default_behavior, mux_kind, mux_arms, has_d }` + a **third**
│                     `DerivedAnalysis.flop_provenance` vec (`skip_serializing_if`)
│                     + `module_flop_provenance`/`design_flop_provenance` — a
│                     direct projection of `Module.flops` (ascending id; enums →
│                     strings), no graph walk. `target=None` ⇒ all flops;
│                     `"flop:<id>"` ⇒ one; unknown ⇒ none. `.4b.2` registers the
│                     kind in `supported_query_kinds()` with the `run_analyze`
│                     dispatch and bumps the schema `1.5 → 1.6`. 5 in-crate proofs.
│                     DUT byte-identical. `.5b.1` adds the fourth query core
│                     `module_reachability`: `QUERY_MODULE_REACHABILITY` +
│                     `ModuleReachability { module, reachable, depth:
│                     Option<usize> (skip-if-None), instantiates[], instance_count
│                     }` + a **fourth** `DerivedAnalysis.module_reachability` vec
│                     (`skip_serializing_if`) + `design_module_reachability` (a BFS
│                     from `design.top` over the `Module.instances[].module`
│                     instance-graph edges of a name→`Module` index; min-depth;
│                     one entry per module sorted by name; absent top ⇒ all
│                     unreachable) / `module_module_reachability` (the bare-module
│                     degenerate one-node case). `target` = a **module name**
│                     (`None` ⇒ all; unknown ⇒ none). `.5b.2` registers the kind
│                     in `supported_query_kinds()` with the `run_analyze` dispatch
│                     and bumps the schema `1.6 → 1.7`; 2 MCP proofs (+ e2e smoke).
│                     6 in-crate proofs. DUT byte-identical. The parallel-vec
│                     pattern now carries four query kinds
│                     (`results`/`reach_results`/`flop_provenance`/`module_reachability`),
│                     each a `skip_serializing_if` vec the `query` discriminates.
├── mcp/             Read-only in-process MCP server
│   ├── mod.rs        (`AGENT-INTROSPECTION-MCP.4`). A dependency-light
│                     JSON-RPC 2.0 dispatcher (`McpServer::handle`, a pure
│                     `Value → Option<Value>`) over the MCP stdio transport
│                     (newline-delimited JSON). Exposes the deterministic
│                     generator as pure/safe **tools** (`generate`,
│                     `introspect`, `dump_config`) + **resources** (the cached
│                     `.sv` / introspection document, addressed by the
│                     content-addressed `run_id`, plus static `knobs`/`lanes`
│                     catalogs). `.5.2` adds the controlled `validate` tool (a
│                     thin adapter over `downstream::validate`: parses the
│                     tool allow-list + `yosys_mode`, fixes the sandbox to the OS
│                     temp dir, audit-logs each call) and the read-only
│                     `anvil://audit/log` resource; the three original tools stay
│                     pure (no FS/exec). `.5.3` adds the controlled `minimize`
│                     tool (a thin adapter over `downstream::minimize` with the
│                     same guardrails + an optional `max_oracle_calls`,
│                     audit-logged); the shared `tools`/`yosys_mode` parsing is
│                     factored into `parse_validate_tools`/`parse_yosys_mode_arg`
│                     so `validate`/`minimize` cannot drift. `.6` adds the third
│                     MCP primitive — **prompts** (`prompts/list`/`prompts/get`,
│                     advertised in `initialize`): a fixed `PROMPTS` registry of
│                     `PromptSpec { name, description, args, render: PromptRender }`
│                     whose pure renderers instantiate the five agent-workflow
│                     chains (`find_downstream_bug`, `close_coverage_gap`,
│                     `minimize_reproducer`, `triage_tool_failures`,
│                     `explain_artifact`) over the *existing* tools/resources with
│                     sample-arg substitution; `prompts/get` validates name +
│                     string-arg type + required args (clean `-32602`). Prompts add
│                     no capability and no new truth. `AGENT-MCP-EXPANSION.2` adds
│                     the pure `coverage_gaps` tool (`project_coverage_gaps` /
│                     `load_coverage_report` / `coverage_gaps_projection`): it
│                     relays the already-computed `coverage_gaps` out of a recorded
│                     `tool_matrix_report.json` (inline `report` OR `report_path`),
│                     plus run metadata, tool pass/fail, and the dark `saw_*` facts
│                     — a `serde_json::Value` key projection (never mirrors the
│                     bin-private `CoverageSummary`), read-only (no generate / no
│                     spawn / no recompute), so the single gap computation stays in
│                     `tool_matrix` (decision `0005`). `AGENT-MCP-EXPANSION.3b`
│                     routes `generate`/`introspect` over the non-DUT lanes via a
│                     `lane` arg (default `dut`): `build_and_cache_lane` drives the
│                     umbrella `MicrodesignLane`/`FrontendLane`, `CachedArtifact`
│                     gains a `manifest`, `resources_read`/`resources_list` serve
│                     `anvil://artifact/<run_id>/manifest`, and
│                     `introspect::manifest_lane_document` builds the non-DUT
│                     envelope as a `Value` (keeping the typed DUT path
│                     byte-identical) that inlines the lane manifest under the
│                     schema's `microdesign_manifest`/`frontend_manifest` payload
│                     key (§5/§6.5) plus the `artifact.manifest` ResourceRef (§4);
│                     `content_run_id` was refactored to `content_run_id_for_knobs`
│                     so non-DUT scoped knobs (`n_params`/`n_children`) feed the
│                     content address (DUT output unchanged). Driven by the
│                     `anvil-mcp` bin; the whole protocol surface is unit-tested
│                     in-process. Separate target ⇒ default `anvil` build /
│                     `--artifact dut` unaffected.
│                     `SEMANTIC-INTROSPECTION-EXPANSION.2b.2` adds the pure
│                     `analyze` tool (`run_analyze`): a DUT-only derived-RELATION
│                     query over the IR graph (regenerates the `Module`/`Design`,
│                     dispatches by `query` kind, wraps the relation in a
│                     `DerivedAnalysisDocument`). `.3b.2` adds the `input_reach`
│                     branch (`module_input_reach`/`design_input_reach`) and
│                     `.4b.2` the `flop_reset_provenance` branch
│                     (`module_flop_provenance`/`design_flop_provenance`), and
│                     `.5b.2` the `module_reachability` branch
│                     (`module_module_reachability`/`design_module_reachability`,
│                     schema `1.6 → 1.7`) beside
│                     `output_support` (`{module,design}_support_cones`); the
│                     unknown-target test checks the result vec the query
│                     populates. Unknown `query`/`target` ⇒ `-32602`
│                     (JSON-RPC error, like `prompts/get`); the result is cached
│                     in a `CachedArtifact.analyses` map and served as
│                     `anvil://artifact/<run_id>/analysis/<query>` (added to
│                     `resources_list`/`resources_read`). No FS/spawn — pure.
│   └── http.rs      (`AGENT-MCP-EXPANSION.4b`). The optional hand-rolled
│                     HTTP/1.1 POST transport beside stdio, re-exported as
│                     `mcp::serve_http` + `mcp::resolve_http_addr`. Pure framing
│                     helpers (`read_http_request` over `BufRead`,
│                     `write_http_response` over `Write`), the `Request` enum,
│                     `handle_http_connection` (one request per connection,
│                     read-timeout, dispatch through the SAME
│                     `McpServer::handle_line`, `Some`→`200`/`None`→`204`,
│                     framing errors → `400`/`405`/`411`/`413`), and `serve_http`
│                     (a single-threaded sequential accept loop over ONE shared
│                     `McpServer`, so cache+audit persist across calls with no
│                     lock). `resolve_http_addr` applies the loopback default
│                     (bare port ⇒ `127.0.0.1:port`; `IP:port` honored +
│                     non-loopback flag). NO new Cargo dependency (`std::net` /
│                     `std::io` / `std::time`); 15 in-process tests incl. 2
│                     real-socket round-trips. Default stdio path byte-identical.
├── bin/
│   ├── anvil_mcp.rs Thin transport shell over `mcp::McpServer` (the `anvil-mcp`
│   │                target, `AGENT-INTROSPECTION-MCP.4`). Default: the stdio
│   │                loop — reads JSON-RPC lines from stdin, writes one response
│   │                line per request, flushes per message. With the opt-in
│   │                `--http <addr>` flag (`AGENT-MCP-EXPANSION.4b`, hand-parsed,
│   │                no clap) it dispatches to `mcp::serve_http` instead
│   │                (loopback default + a non-loopback stderr warning). All
│   │                logic lives in `mcp`; this is transport selection only.
│   └── tool_matrix.rs
│                     Repo-owned downstream-tool matrix harness. Its
│                     Verilator/Yosys/iverilog invocation primitives now live
│                     in `anvil::downstream` (`AGENT-INTROSPECTION-MCP.5.1`);
│                     the binary `use`s them and keeps the scenario/coverage/
│                     resume/report orchestration.
│                     Builds a curated scenario set over
│                     construction strategy, identity mode,
│                     factorization level, and two stress profiles;
│                     generates per-scenario corpora, runs Verilator
│                     and Yosys, optionally compiles/elaborates with
│                     Icarus Verilog, writes per-module
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
│                     file. `--iverilog-compile` shells
│                     `iverilog -g2012` and records a warning-clean
│                     compile/elaboration result without running a
│                     testbench. `--phase2-share-gate` now adds the
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
│   │                 cones (`width`, reset, and endpoint-aware proof;
│   │                 up to 12 endpoint-support bits only inside the
│   │                 current node/work budget).
│   │                 Different endpoint variables do not merge.
│   │                 `merge_equivalent_gates(&mut Module)` is the
│   │                 first live bounded `e-graph` fragment:
│   │                 under `identity_mode = node-id` and effective
│   │                 `EGraph`, small-support combinational cones
│   │                 proven equal over the same canonical leaf
│   │                 variables collapse to one gate; tiny 12-bit
│   │                 support cones are admitted only when
│   │                 assignment-count × cone-node-count stays within
│   │                 the old 10-bit work envelope. Then
│   │                 `merge_equivalent_flops(&mut Module)` applies
│   │                 the analogous endpoint-aware proof discipline
│   │                 to flop state elements. The opt-in
│   │                 `merge_bisimilar_flops(&mut Module)`
│   │                 (`IDENTITY-DEEPENING.2b`, gated on
│   │                 `Module::bisimulation_flop_merge` +
│   │                 node-id / e-graph) then runs a bounded
│   │                 greatest-fixpoint partition refinement that merges
│   │                 flops proven sequentially equivalent up to a state
│   │                 correspondence (e.g. mutually-recursive registers):
│   │                 it threads a `FlopQ -> class rep` quotient through
│   │                 the same proof functions and reuses the shared
│   │                 `finalize_flop_merge` rewrite the exact pass now
│   │                 also calls; resetless flops are excluded (no reset
│   │                 base case); default-off / byte-identical. Its
│   │                 bucket -> refinable-partition -> greatest-fixpoint
│   │                 refinement core is factored into the non-mutating
│   │                 `bisimulation_partition(&Module) -> Option<Vec<Vec<FlopId>>>`
│   │                 helper (`IDENTITY-DEEPENING.3b.2a`); `merge_bisimilar_flops`
│   │                 keeps its collapse + `finalize_flop_merge` tail and stays
│   │                 byte-identical. The cross-module whole-leaf-module
│   │                 sequential-equivalence proof
│   │                 `modules_sequentially_equivalent(&Module, &Module) -> bool`
│   │                 (`IDENTITY-DEEPENING.3b.2b.1`) reuses that helper on a
│   │                 temporary combined module (`build_combined_module`, A's and
│   │                 B's primary inputs unified by `(PortId, width)`): it runs the
│   │                 bisimulation partition on the union state, then proves every
│   │                 output drive cone equal under the final quotient. Pure /
│   │                 non-mutating; consumed by `dedup_sequential_modules`.
│   │                 `merge_equivalent_fsms(&mut Module)` applies it
│   │                 to deterministic generated FSM blocks with
│   │                 matching selector proof, encoding, transition
│   │                 table, Moore-output table, and output width.
│   │                 `fold_proven_gates(&mut Module)`
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
│   │                 surviving NodeIds / FlopIds and virtual flop/FSM
│   │                 deps, and rebuilds dedup tables. Called from
│   │                 `gen::module::generate_leaf_module`; counts are
│   │                 surfaced as `Metrics::semantic_gates_merged`,
│   │                 `Metrics::flops_merged`,
│   │                 `Metrics::bisimulation_flops_merged`,
│   │                 `Metrics::fsms_merged`, and
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
│   ├── dedup.rs      Opt-in hierarchy module identity passes:
│                     `dedup_modules` collapses structurally-identical
│                     Module definitions by canonical signature;
│                     `dedup_semantic_modules` collapses non-top
│                     pure-combinational, state-free concrete modules
│                     by a bounded whole-module truth-table proof
│                     (same PortId/width interface, <=12 input-support
│                     bits, <=128 reachable nodes). The semantic proof
│                     covers instance-free modules plus bounded
│                     pure-combinational wrappers whose children are
│                     also inside the proof boundary; it keeps leaves
│                     and wrappers in separate proof classes and skips
│                     ancestor/descendant wrapper merge groups.
│                     `dedup_sequential_modules` (`IDENTITY-DEEPENING.3b.2b.1`,
│                     opt-in `Config::hierarchy_sequential_module_dedup`,
│                     node-id / e-graph) is the sequential generalization: it
│                     collapses stateful flops-only leaf modules proven
│                     observationally equivalent by the cross-module
│                     bisimulation `compact::modules_sequentially_equivalent`,
│                     grouping candidates with a cheap structural pre-filter
│                     (interface + flop multiset) and greedy-by-representative
│                     grouping (sound because sequential equivalence is
│                     transitive). That grouping is factored into a
│                     non-mutating `group_sequentially_equivalent_modules`
│                     (`IDENTITY-DEEPENING.3b.2b.2a`) shared by the pass and
│                     the `DesignMetrics` sequential proof metric, so the
│                     metric's counted duplicate pairs are exactly what the
│                     pass collapses. All three rewrite Instance.module
│                     references to the survivor and, after a real merge, prune
│                     definitions that were reachable before dedup but
│                     are no longer reachable from the design top.
│                     No-merge calls and pre-existing
│                     under-instantiation are not reachability-pruned.
│   ├── param.rs      Phase 5 parameterization helpers
│   │                 (`PHASE-5-PARAMETERIZATION`): `WidthExpr` /
│   │                 `ParamEnv` resolution behind the IR's
│   │                 `Instance.param_bindings`, supporting rules-first
│   │                 width-generic parameterizable leaves and per-instance
│   │                 `#(.W(v))` overrides with resolved-width validation.
│   └── aggregate.rs  Phase 5b synthesizable-aggregate layout
│                     (`PHASE-5B-AGGREGATES`): the `Default`-able
│                     `Module.aggregate_layout` annotation
│                     (`AggregateLayout`/`AggregateKind`/`AggregateGroup`)
│                     consulted only by the emitter for the packed-`struct`
│                     (and `ArrayPacked`) boundary projection; flat IR /
│                     validators / dedup untouched, so the projected twin
│                     dedup-collapses and the default path stays
│                     byte-identical.
│
├── gen/
│   ├── mod.rs        Generator struct (rng + cfg + next_module_index),
│   │                 generate_module(), generate_design(). Depth 0
│   │                 still routes into the mature leaf-module lane;
│   │                 hierarchy dispatches to either the legacy exact
│   │                 depth-1 wrapper lane or the newer bounded
│   │                 recursive lane. `generate_design` runs opt-in
│   │                 structural module dedup, then opt-in bounded
│   │                 semantic module dedup only under node-id/e-graph,
│   │                 before parameter/aggregate/multi-clock projection.
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
│   │                 1/2/3 leaf kernel; hierarchy composes above it
│   │                 rather than collapsing inter-module generation
│   │                 into it. `m.semantic_gates_merged`,
│   │                 `m.flops_merged`, `m.fsms_merged`, and `m.nodes_compacted`
│   │                 record the removal counts.
│   ├── hierarchy.rs  Current Phase 4 planner. Keeps the legacy exact
│   │                 depth-1 wrapper lane alive, and also lands a
│   │                 bounded recursive lane driven by
│   │                 `min_hierarchy_depth..=max_hierarchy_depth` and
│   │                 `min_child_instances_per_module..=max_child_instances_per_module`.
│   │                 The recursive lane now keeps every leaf depth
│   │                 inside the requested interval, can mix
│   │                 shallow/deep branches when the interval is open
│   │                 and the structure allows it, chooses each
│   │                 non-leaf module's child count inside the
│   │                 requested interval. Both hierarchy
│   │                 lanes now also expose explicit child sourcing
│   │                 (`library` vs `on-demand`), and both build real
│   │                 parent-side logic over child `InstanceOutput`
│   │                 leaves and parent data ports, including mixed
│   │                 parent-port / child-output parent outputs,
│   │                 sibling-routed child-input binding,
│   │                 parent-composed child-input cones, registered
│   │                 child-input routes, and optional local parent
│   │                 flops. Exact-depth recursive profiles now also
│   │                 prove helper-through-state parent-composed,
│   │                 direct sibling, direct registered sibling,
│   │                 multi-stage direct registered sibling, registered
│   │                 parent-composed, and multi-stage registered
│   │                 parent-composed helper routes below the top parent.
│   │                 First-class helper
│   │                 instantiation inside
│   │                 parent cone choice is now live for parent-composed
│   │                 child-input cones, direct sibling routes, direct
│   │                 registered sibling-route D inputs, registered
│   │                 child-input D cones, and parent-output cones,
│   │                 with an explicit per-parent budget. Opt-in
│   │                 module-dedup identity is live through
│   │                 `ir/dedup.rs`; broader helper placement beyond
│   │                 those routes and deeper hierarchy equivalence
│   │                 remain open.
│   ├── cone.rs       Fanin-cone recursion + interleaved frame machine
│   │                 (strategy core). `CONE-DECOMPOSITION` split the former
│   │                 5551-line file into a `cone/` submodule dir; `cone.rs`
│   │                 is now ~2446 lines holding only the recursion strategy
│   │                 (build_cone_with_retry, build_graph_first,
│   │                 grow_pool_one_unit, build_outputs_interleaved,
│   │                 process_signal_frame, deliver, build_cone,
│   │                 drain_flop_worklist_pool_only, roll_knob,
│   │                 node_budget_reached, the SignalFrame/GateFrame frames,
│   │                 the FlopWorklist alias) plus the inline tests. Each
│   │                 submodule is re-exported via `pub(crate) use <sub>::*`
│   │                 so every `crate::gen::cone::<symbol>` path stays stable.
│   ├── cone/semantic.rs   value-set / unsigned-bounds / exact-value proofs
│   │                 (pure `&Module` analysis; ~1360 lines).
│   ├── cone/primitives.rs IR gate makers (make_constant/_eq_const/_mux/
│   │                 _and/_mul/_sub/_nary_add/_nary_mul, build_comparison_gate,
│   │                 replicate_to_width).
│   ├── cone/terminals.rs  terminal/pool selection + gate-shape policy
│   │                 (pick_terminal[_dep_bearing], make_width_adapter,
│   │                 pick_gate, input_widths_for, anti-collapse, try_share,
│   │                 node_deps).
│   ├── cone/flops.rs      flop D-cone drains + assemblers (drain_flop_worklist
│   │                 + one_hot/encoded, assemble_flop_d_*, build_flop_leaf,
│   │                 pick_reset_value, ceil_log2, pick_mux_arm_count).
│   ├── cone/motifs.rs     structured block/motif builders (comb-mux / case /
│   │                 casez / for-fold recursive + pool-only, priority encoder,
│   │                 linear-combination, shift, comparand, or_reduce_terms).
│   ├── cone/snapshot.rs   construction-snapshot rollback machinery.
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
    ├── mod.rs        Re-exports to_sv, to_sv_in_design, to_sv_design and
    │                 their `*_versioned(.., SvVersion)` counterparts.
    └── sv.rs         IR → String pretty-printer. Assumes invariants hold.
                      `to_sv*` entry points delegate to `to_sv*_versioned`
                      with `SvVersion::default()` (Sv2012), so existing
                      callers are byte-identical; `sv_version` threads to
                      `to_sv_with_modules` as the down-gating capability
                      bound, consulted via `SvVersion::permits` at
                      version-distinctive construct sites (none yet — the
                      whole subset is 1800-2012-valid, so every target is
                      byte-identical today; SV-VERSION-TARGETING.2b.1).
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
| 1 — Single-module MVP        | done         | `gen/cone.rs`, `gen/module.rs`, `emit/sv.rs`, `gen/pool.rs`, `ir/types.rs`, `ir/compact.rs`, `metrics.rs` | Combinational + sequential cone recursion functional; flop worklist drained; `always_ff` emitted; single CLK + single RST_N (async). 22 structural rules enforced (Rules 1-22). Zero orphans restored at module finalisation via Rule-18 construction discipline plus `compact_node_ids`; final compaction now also drops dead flops whose `Q` is never observed, and the emitted input surface is trimmed to live ports/bits. Factorization ladder is live through a bounded `EGraph` fragment, with post-construction semantic gate merging for small-support cones, post-remap associative re-normalisation on the settled graph, a late mixed-associative-constant cleanup pass on that same settled graph, endpoint-preserving post-drain flop merging, deterministic generated-FSM block merging under `identity_mode = node-id`, strict Add/Mul remap-pruning under `operand_duplication_rate < 1.0`, a final exact-value cleanup pass (`fold_proven_gates`) for downstream-tool cleanliness that keeps the general exact prover tiny-only (width <= 8, support <= 12 bits, <= 3 canonical leaf endpoints, and the cleanup node/work budget) while still revisiting compare gates with the bounded unsigned-compare proof and shift gates with a bounds-only exact check, plus a tiny-domain rhs fallback for shift overshift proofs when narrow boolean-mask arithmetic keeps the rhs domain small even though the whole cone is large. Exit gate now closed locally via `/tmp/anvil-tool-matrix-phase1-real-r21/tool_matrix_report.json` (1005 modules, `coverage_gaps = []`, 1005/0 in Verilator and both repo-owned Yosys modes). |
| 2 — Sharing                  | done         | `gen/cone.rs`, `ir/types.rs`, `ir/compact.rs` | Per-operand `share_prob` hook wired; internal gates enter the pool as they are built. Construction-time CSE (Rule 21) + operand-uniqueness (Rule 8 extended) + commutative normalization (Rule 21b) + associative flattening + constant folding + peephole rewrites all enforced via `intern_gate`; the live bounded `EGraph` fragment now merges small-support combinational cones post-construction under `identity_mode = node-id`, duplicate flops and deterministic generated FSM blocks merge post-drain when they are proven equal over the same canonical leaf endpoints by the same proof discipline, and late remaps are pruned when they would violate the strict Add/Mul duplicate policy. Final compaction cleans orphaned intermediates and dead state from these rewrites. Exit gate now closed locally via `/tmp/anvil-tool-matrix-phase2-share-r1/tool_matrix_report.json` (216 modules, `coverage_gaps = []`, 216/0 in Verilator and both repo-owned Yosys modes). The representative sweep proves controllability with normalized `shared_node_fraction` rather than raw shared-node count, because stronger reuse collapses total node count. |
| 3 — Structured combinational | done         | `gen/cone.rs`, `ir/types.rs`, `emit/sv.rs`, `ir/validate.rs`, `metrics.rs`, `bin/tool_matrix.rs`, `ir/compact.rs` | Priority-encoder block (Rule 17), combinational mux block (Rule 15), procedural case-mux block (`always_comb case` for dynamic selectors, continuous `assign` for constant selectors), procedural casez-mux block (`always_comb casez` with non-overlapping wildcard patterns for dynamic selectors, continuous `assign` for constant selectors), structured bounded `for`-fold blocks (`always_comb` + `for (int i = 0; i < N; i++)` over packed chunks for dynamic sources, continuous `assign` for constant sources), generic selectable `Slice` / variadic `Concat`, coefficient motif, both shift-amount paths (`const_shift_amount_prob` plus the ordinary variable-amount path), const-comparand motif, and reduction-category gate picking are all landed. The dedicated structured-surface closure gate now exists in `tool_matrix` as `--phase3-structured-gate`, and it is closed locally via `/tmp/anvil-tool-matrix-phase3-structured-r4/tool_matrix_report.json` (210 modules, `coverage_gaps = []`, 210/0 in Verilator and both repo-owned Yosys modes). The runtime hotspot that surfaced while proving that gate was addressed at the real seam: large settled cones with tiny support now skip semantic merge proofs and fall back to structural proof instead of stalling in `semantic_cone_proof`. |
| 4 — Hierarchy                | done         | `gen/hierarchy.rs`, `ir/types.rs`, `ir/compact.rs`, `ir/validate.rs`, `ir/dedup.rs`, `emit/sv.rs`, `main.rs`, `metrics.rs`, `bin/tool_matrix.rs` | Closed `2026-05-16` (`PHASE-4-HIERARCHY.3` scope-cut against explicit ROADMAP exit criteria; closing artifact r87). Has the legacy exact depth-1 wrapper planner and bounded recursive planner. The wrapper lane exercises exact, reuse, and under-instantiation profiles via `num_child_instances`; the recursive lane uses `min_hierarchy_depth..=max_hierarchy_depth`, `min_child_instances_per_module..=max_child_instances_per_module`, and optional per-depth child-instance overrides. Both lanes expose explicit `hierarchy_child_source_mode = library | on-demand`, parent-side output cones over child instance outputs plus parent data ports, sibling-routed and parent-composed child-input binding, registered sibling and registered parent-composed binding through parent-local flops, parent-local child-input cones, optional local parent flops, and parent-cone helper instances for child-input, sibling-route, registered-D, and parent-output sources. The recursive lane now proves helper-through-state parent-composed routing, direct sibling helper routing, direct registered sibling helper routing, multi-stage direct registered sibling helper routing, registered parent-composed helper D-cone routing, registered parent-composed helper D-cone routing with mixed parent-port support, multi-stage registered parent-composed helper routing, parent-output helper routing, parent-output helper routing with mixed parent-port support, stateful parent-output helper routing, stateful parent-output helper routing with mixed parent-port support, unregistered parent-composed helper child-input routing with mixed parent-port support, stateful parent-composed helper child-input routing with mixed parent-port support, direct registered sibling mixed-support routing, recursive non-top direct registered sibling mixed-support routing, recursive non-top no-helper parent-port-composed output routing, recursive non-top stateful no-helper parent-port-composed output routing, recursive non-top stateful no-helper unregistered parent-composed mixed-support child-input routing, recursive non-top parent-local flops as a first-class gated coverage fact, recursive parent-local flops gated at exact hierarchy depth 3, recursive non-top unregistered parent-composed mixed-support child inputs gated at exact hierarchy depth 3 without helpers, recursive non-top parent-port-composed parent outputs gated at exact hierarchy depth 3 without helpers or state, recursive non-top stateful parent-port-composed parent outputs gated at exact hierarchy depth 3 without helpers, recursive non-top stateful parent-composed mixed-support child inputs gated at exact hierarchy depth 3 without helpers, recursive non-top parent-local flops gated at exact hierarchy depth 4, recursive non-top mixed-support child inputs gated at exact hierarchy depth 4 without helpers, recursive non-top parent-port-composed parent outputs gated at exact hierarchy depth 4 without helpers or state, recursive non-top stateful parent-port-composed parent outputs gated at exact hierarchy depth 4 without helpers, recursive non-top stateful parent-composed mixed-support child inputs gated at exact hierarchy depth 4 without helpers, recursive non-top parent-local flops gated at exact hierarchy depth 5, recursive non-top mixed-support child inputs gated at exact hierarchy depth 5 without helpers, recursive non-top parent-port-composed parent outputs gated at exact hierarchy depth 5 without helpers or state, recursive non-top stateful parent-port-composed parent outputs gated at exact hierarchy depth 5 without helpers, recursive non-top stateful unregistered parent-composed mixed-support child inputs gated at exact hierarchy depth 5 without helpers, recursive non-top parent-local flops gated at exact hierarchy depth 6, recursive non-top mixed-support child inputs gated at exact hierarchy depth 6 without helpers, recursive non-top parent-port-composed parent outputs gated at exact hierarchy depth 6 without helpers or state, recursive non-top stateful parent-port-composed parent outputs gated at exact hierarchy depth 6 without helpers, recursive non-top stateful unregistered parent-composed mixed-support child inputs gated at exact hierarchy depth 6 without helpers (2,2 calibrated), recursive non-top parent-local flops gated at exact hierarchy depth 7, recursive non-top mixed-support child inputs gated at exact hierarchy depth 7 without helpers (2,2 calibrated), recursive non-top parent-port-composed parent outputs gated at exact hierarchy depth 7 without helpers or state, recursive non-top stateful parent-port-composed parent outputs gated at exact hierarchy depth 7 without helpers, registered mixed-support routing, no-helper multi-stage registered parent-composed routing, no-helper multi-stage registered sibling routing, and no-helper multi-stage registered mixed-support routing below the top parent in exact-depth-2 trees. Hierarchy manifests/reports carry exact per-design `DesignMetrics`, including child-input provenance, registered/multistage/helper fractions, helper-instance budgets, parent-output helper support, helper-through-flop support, direct and stateful helper mixed-support output fractions, unregistered helper child-input mixed-support fractions, stateful helper-through-flop mixed-support child-input fractions, direct registered sibling mixed-support fractions, local parent-state counts, top-interface shape, depth histograms, per-depth branching summaries, and weighted child load. Module names are reserved from one generator-global sequence. The latest full downstream-clean repo-owned Phase 4 bank is `/tmp/anvil-tool-matrix-phase4-hierarchy-r87/tool_matrix_report.json` (840 designs, `coverage_gaps = []`, 840/0 in Verilator and both repo-owned Yosys modes), and it covers wrapper exact/reuse/under-instantiation, recursive depth `2`, mixed recursive depth range `2:3`, child-source modes, child-instance profiles `2`, `4`, `2:3`, `1:3`, per-depth override `0=4:4,1=2:2`, registered mixed-support routing, recursive non-top registered mixed-support routing, multi-stage registered parent-composed routing, recursive non-top multi-stage registered parent-composed no-helper routing, multi-stage registered sibling routing, recursive non-top multi-stage registered sibling no-helper routing, recursive non-top multi-stage registered mixed-support no-helper routing, recursive non-top registered parent-composed helper mixed-support routing, recursive non-top parent-output helper mixed-support routing, registered sibling routing, direct registered sibling mixed-support routing, helper-backed child-input and parent-output routing, recursive non-top helper routes, recursive non-top multi-helper budgets, local parent flops, parent-side composition, and mixed parent-port / child-output parent outputs, recursive non-top stateful parent-port-composed parent outputs without helpers, recursive non-top stateful unregistered parent-composed mixed-support child-input routing through parent-local Qs without helpers, recursive non-top parent-local flops gated as a first-class coverage fact, recursive parent-local flops gated at exact hierarchy depth 3, recursive non-top unregistered parent-composed mixed-support child inputs gated at exact hierarchy depth 3 without helpers, recursive non-top parent-port-composed parent outputs gated at exact hierarchy depth 3 without helpers or state, and recursive non-top stateful parent-port-composed parent outputs gated at exact hierarchy depth 3 without helpers, and recursive non-top stateful parent-composed mixed-support child inputs gated at exact hierarchy depth 3 without helpers, and recursive non-top parent-local flops gated at exact hierarchy depth 4, and recursive non-top mixed-support child inputs gated at exact hierarchy depth 4 without helpers, and recursive non-top parent-port-composed parent outputs gated at exact hierarchy depth 4 without helpers or state, and recursive non-top stateful parent-port-composed parent outputs gated at exact hierarchy depth 4 without helpers, and recursive non-top stateful parent-composed mixed-support child inputs gated at exact hierarchy depth 4 without helpers, and recursive non-top parent-local flops gated at exact hierarchy depth 5, and recursive non-top mixed-support child inputs gated at exact hierarchy depth 5 without helpers, and recursive non-top parent-port-composed parent outputs gated at exact hierarchy depth 5 without helpers or state, and recursive non-top stateful parent-port-composed parent outputs gated at exact hierarchy depth 5 without helpers, and recursive non-top stateful unregistered parent-composed mixed-support child inputs gated at exact hierarchy depth 5 without helpers — closing the depth-5 sweep, and recursive non-top parent-local flops gated at exact hierarchy depth 6 — opening the depth-6 axis, and recursive non-top mixed-support child inputs gated at exact hierarchy depth 6 without helpers, and recursive non-top parent-port-composed parent outputs gated at exact hierarchy depth 6 without helpers or state, and recursive non-top stateful parent-port-composed parent outputs gated at exact hierarchy depth 6 without helpers, and recursive non-top stateful unregistered parent-composed mixed-support child inputs gated at exact hierarchy depth 6 without helpers (2,2 calibrated) — closing the depth-6 sweep, and recursive non-top parent-local flops gated at exact hierarchy depth 7 — opening the depth-7 axis, and recursive non-top mixed-support child inputs gated at exact hierarchy depth 7 without helpers (2,2 calibrated), and recursive non-top parent-port-composed parent outputs gated at exact hierarchy depth 7 without helpers or state, and recursive non-top stateful parent-port-composed parent outputs gated at exact hierarchy depth 7 without helpers, recursive non-top stateful unregistered parent-composed mixed-support child inputs gated at exact hierarchy depth 7 without helpers (2,2 calibrated) — closing the depth-7 sweep, recursive non-top registered parent-composed child-input bindings that chain through three or more parent-local flop stages without helpers — opening a chain-depth axis above the closed depth-3..7 sweeps, a recursive non-top internal parent saturating a parent-cone helper budget of 5 helpers — extending the helper-budget axis above the previous budget-3 baseline, and per-module canonical signatures as the first slice of hierarchy-aware identity instrumentation, plus a depth-1 wrapper-lane scenario proving the planner can emit structurally-duplicate Module definitions under tight constraints (HIERARCHY-AWARE-IDENTITY.2). The current mixed-support batch includes stateful parent-composed helper child-input mixed-support metrics and Phase 4 coverage facts, with coverage-only dry-run evidence at `/tmp/anvil-tool-matrix-phase4-stateful-helper-child-input-mixed-check/tool_matrix_report.json`. The same mixed-support batch also includes unregistered parent-composed helper child-input mixed-support metrics and Phase 4 coverage facts, with coverage-only dry-run evidence at `/tmp/anvil-tool-matrix-phase4-parent-helper-child-input-mixed-check/tool_matrix_report.json`. The same mixed-support batch also includes stateful parent-output helper mixed-support metrics and Phase 4 coverage facts, plus required decision-site attempts for the plain `hierarchy_sibling_route_prob` knob, with coverage-only dry-run evidence at `/tmp/anvil-tool-matrix-phase4-mixed-helper-check/tool_matrix_report.json`. `r50` superseded those coverage-only dry runs with full downstream-clean evidence, `r51` added direct registered sibling mixed-support evidence, `r52` added recursive non-top direct registered sibling mixed-support evidence, and `r53` carries them forward while adding recursive non-top unregistered parent-composed mixed-support child-input evidence, `r54` adds recursive no-state parent-port-composed parent-output evidence, `r55` adds recursive stateful parent-port-composed parent-output evidence, `r56` adds recursive stateful unregistered parent-composed mixed-support child-input evidence, `r57` gates recursive non-top parent-local flops as a first-class coverage fact, `r58` extends parent-local-flop gating to exact hierarchy depth 3, `r59` extends mixed-support child-input gating to exact hierarchy depth 3, `r60` extends parent-port-composed parent-output gating to exact hierarchy depth 3, `r61` extends stateful parent-port-composed parent-output gating to exact hierarchy depth 3, `r62` extends stateful parent-composed mixed-support child-input gating to exact hierarchy depth 3 — completing the depth-3 push, `r63` opens the depth-4 axis with parent-local flops at exact hierarchy depth 4, `r64` extends the depth-4 axis to mixed-support child inputs, `r65` extends the depth-4 axis to parent-port-composed parent outputs, `r66` extends the depth-4 axis to stateful parent-port-composed parent outputs, `r67` closes the depth-4 sweep with stateful parent-composed mixed-support child inputs, `r68` opens the depth-5 axis with parent-local flops, `r69` extends the depth-5 axis with mixed-support child inputs, `r70` extends the depth-5 axis with parent-port-composed parent outputs, `r71` extends the depth-5 axis with stateful parent-port-composed parent outputs, `r72` closes the depth-5 sweep with stateful unregistered parent-composed mixed-support child inputs, `r73` opens the depth-6 axis with parent-local flops, `r74` extends the depth-6 axis with mixed-support child inputs (2,2 calibrated), `r75` extends the depth-6 axis with parent-port-composed parent outputs, `r76` extends the depth-6 axis with stateful parent-port-composed parent outputs, `r77` closes the depth-6 sweep with stateful unregistered parent-composed mixed-support child inputs (2,2 calibrated), `r78` opens the depth-7 axis with parent-local flops, `r79` extends the depth-7 axis with mixed-support child inputs (2,2 calibrated), `r80` extends the depth-7 axis with parent-port-composed parent outputs, `r81` extends the depth-7 axis with stateful parent-port-composed parent outputs, `r82` closes the depth-7 sweep with stateful unregistered parent-composed mixed-support child inputs (2,2 calibrated), `r83` opens a chain-depth axis above the closed depth-3..7 sweeps with three-stage registered parent-composed chain coverage, `r84` extends the helper-budget axis above the previous budget-3 baseline with parent-cone helper budget 5 coverage, `r85` adds canonical module signatures as the first slice of hierarchy-aware identity instrumentation, `r86` closes HIERARCHY-AWARE-IDENTITY.2 by proving the planner can emit structurally-duplicate Module definitions under tight constraints, and `r87` closes HIERARCHY-AWARE-IDENTITY.4 + .5 by implementing the post-finalisation module-dedup pass under the opt-in `Config::hierarchy_module_dedup` knob (tree complete). `r48` is now the previous recursive non-top registered parent-composed helper mixed-support full bank; `r49` is the previous recursive non-top parent-output helper mixed-support full bank; `r50` is the previous accumulated mixed-support hierarchy full bank; `r51` is the previous direct registered sibling mixed-support hierarchy full bank; `r52` is the previous recursive direct registered sibling mixed-support hierarchy full bank; `r53` is the previous recursive parent-composed mixed-support child-input hierarchy full bank, `r54` is the previous recursive parent-port-composed parent-output hierarchy full bank, `r55` is the previous recursive stateful parent-port-composed parent-output hierarchy full bank, `r56` is the previous recursive stateful unregistered parent-composed mixed-support child-input hierarchy full bank, `r57` is the previous hierarchy full bank that gated recursive non-top parent-local flops as a first-class coverage fact, `r58` is the previous hierarchy full bank that pushed recursive parent-local flops to exact hierarchy depth 3, `r59` is the previous hierarchy full bank that pushed recursive non-top unregistered parent-composed mixed-support child inputs to exact hierarchy depth 3 without helpers, `r60` is the previous hierarchy full bank that pushed recursive non-top parent-port-composed parent outputs to exact hierarchy depth 3 without helpers or state, `r61` is the previous hierarchy full bank that pushed recursive non-top stateful parent-port-composed parent outputs to exact hierarchy depth 3 without helpers, `r62` is the previous hierarchy full bank that closed the depth-3 push with recursive non-top stateful parent-composed mixed-support child inputs at exact hierarchy depth 3 without helpers, `r63` is the previous hierarchy full bank that opened the depth-4 axis with recursive non-top parent-local flops at exact hierarchy depth 4, `r64` is the previous hierarchy full bank that extended the depth-4 axis with recursive non-top mixed-support child inputs at exact hierarchy depth 4 without helpers, `r65` is the previous hierarchy full bank that extended the depth-4 axis with recursive non-top parent-port-composed parent outputs at exact hierarchy depth 4 without helpers or state, `r66` is the previous hierarchy full bank that extended the depth-4 axis with recursive non-top stateful parent-port-composed parent outputs at exact hierarchy depth 4 without helpers, `r67` is the previous hierarchy full bank that closed the depth-4 sweep with recursive non-top stateful parent-composed mixed-support child inputs at exact hierarchy depth 4 without helpers, `r68` is the previous hierarchy full bank that opened the depth-5 axis with recursive non-top parent-local flops at exact hierarchy depth 5, `r69` is the previous hierarchy full bank that extended the depth-5 axis with recursive non-top mixed-support child inputs at exact hierarchy depth 5 without helpers, `r70` is the previous hierarchy full bank that extended the depth-5 axis with recursive non-top parent-port-composed parent outputs at exact hierarchy depth 5 without helpers or state, `r71` is the previous hierarchy full bank that extended the depth-5 axis with recursive non-top stateful parent-port-composed parent outputs at exact hierarchy depth 5 without helpers, `r72` is the previous hierarchy full bank that closed the depth-5 sweep with recursive non-top stateful unregistered parent-composed mixed-support child inputs at exact hierarchy depth 5 without helpers, `r73` is the previous hierarchy full bank that opened the depth-6 axis with recursive non-top parent-local flops at exact hierarchy depth 6, `r74` is the previous hierarchy full bank that extended the depth-6 axis with recursive non-top mixed-support child inputs at exact hierarchy depth 6 without helpers (2,2 calibrated), `r75` is the previous hierarchy full bank that extended the depth-6 axis with recursive non-top parent-port-composed parent outputs at exact hierarchy depth 6 without helpers or state, `r76` is the previous hierarchy full bank that extended the depth-6 axis with recursive non-top stateful parent-port-composed parent outputs at exact hierarchy depth 6 without helpers, `r77` is the previous hierarchy full bank that closed the depth-6 sweep with recursive non-top stateful unregistered parent-composed mixed-support child inputs at exact hierarchy depth 6 without helpers (2,2 calibrated), `r78` is the previous hierarchy full bank that opened the depth-7 axis with recursive non-top parent-local flops at exact hierarchy depth 7, `r79` is the previous hierarchy full bank that extended the depth-7 axis with recursive non-top mixed-support child inputs at exact hierarchy depth 7 without helpers (2,2 calibrated), `r80` is the previous hierarchy full bank that extended the depth-7 axis with recursive non-top parent-port-composed parent outputs at exact hierarchy depth 7 without helpers or state, `r81` is the previous hierarchy full bank that extended the depth-7 axis with recursive non-top stateful parent-port-composed parent outputs at exact hierarchy depth 7 without helpers, `r82` is the previous hierarchy full bank that closed the depth-7 sweep with recursive non-top stateful unregistered parent-composed mixed-support child inputs at exact hierarchy depth 7 without helpers (2,2 calibrated), `r83` is the previous hierarchy full bank that opened a chain-depth axis above the closed depth-3..7 sweeps with recursive non-top registered parent-composed three-stage chain coverage, `r84` is the previous hierarchy full bank that extended the helper-budget axis above the previous budget-3 baseline with recursive non-top parent-cone helper budget 5 coverage, `r85` is the previous hierarchy full bank that added canonical module signatures as the first slice of hierarchy-aware identity instrumentation, `r86` is the previous hierarchy full bank that closed HIERARCHY-AWARE-IDENTITY.2 by proving the planner can emit structurally-duplicate Module definitions under tight constraints, and `r87` is the current hierarchy full bank that closes HIERARCHY-AWARE-IDENTITY.4 + .5 by implementing the post-finalisation module-dedup pass under the opt-in `Config::hierarchy_module_dedup` knob (tree complete). Focused targeted evidence includes `cargo test recursive_hierarchy_parent_outputs_mix_helper_instances_with_parent_ports_below_top`, `cargo test metrics::tests::design_metrics_capture_stateful_parent_cone_instance_mixed_output_support`, `cargo test metrics::tests::design_metrics_capture_multiple_parent_cone_instance_budget`, `cargo test metrics::tests::design_metrics_capture_parent_composed_parent_cone_instance_flop_routes`, `cargo test registered_sibling_mixed_support`, `cargo test hierarchy_registered_sibling_routes_can_mix_parent_port_support`, and `cargo test recursive_hierarchy_registered_sibling_routes_can_mix_parent_port_support_below_top`, `cargo test recursive_hierarchy_parent_outputs_mix_parent_ports_below_top_without_helpers`, `cargo test recursive_hierarchy_stateful_parent_outputs_mix_parent_ports_below_top_without_helpers`, `cargo test recursive_hierarchy_stateful_parent_composed_routes_mix_parent_ports_below_top_without_helpers`, `cargo test recursive_hierarchy_parents_can_emit_local_flops_below_top`, `cargo test recursive_hierarchy_parents_can_emit_local_flops_at_depth_3`, `cargo test recursive_hierarchy_parent_composed_routes_mix_parent_ports_at_depth_3_without_helpers`, `cargo test recursive_hierarchy_parent_outputs_mix_parent_ports_at_depth_3_without_helpers`, `cargo test recursive_hierarchy_stateful_parent_outputs_mix_parent_ports_at_depth_3_without_helpers`, `cargo test recursive_hierarchy_stateful_parent_composed_routes_mix_parent_ports_at_depth_3_without_helpers`, `cargo test recursive_hierarchy_parents_can_emit_local_flops_at_depth_4`, `cargo test recursive_hierarchy_parent_composed_routes_mix_parent_ports_at_depth_4_without_helpers`, `cargo test recursive_hierarchy_parent_outputs_mix_parent_ports_at_depth_4_without_helpers`, `cargo test recursive_hierarchy_stateful_parent_outputs_mix_parent_ports_at_depth_4_without_helpers`, `cargo test recursive_hierarchy_stateful_parent_composed_routes_mix_parent_ports_at_depth_4_without_helpers`, `cargo test recursive_hierarchy_parents_can_emit_local_flops_at_depth_5`, `cargo test recursive_hierarchy_parent_composed_routes_mix_parent_ports_at_depth_5_without_helpers`, `cargo test recursive_hierarchy_parent_outputs_mix_parent_ports_at_depth_5_without_helpers`, `cargo test recursive_hierarchy_stateful_parent_outputs_mix_parent_ports_at_depth_5_without_helpers`, `cargo test recursive_hierarchy_stateful_parent_composed_routes_mix_parent_ports_at_depth_5_without_helpers`, `cargo test recursive_hierarchy_parents_can_emit_local_flops_at_depth_6`, `cargo test recursive_hierarchy_parent_composed_routes_mix_parent_ports_at_depth_6_without_helpers`, `cargo test recursive_hierarchy_parent_outputs_mix_parent_ports_at_depth_6_without_helpers`, `cargo test recursive_hierarchy_stateful_parent_outputs_mix_parent_ports_at_depth_6_without_helpers`, `cargo test recursive_hierarchy_stateful_parent_composed_routes_mix_parent_ports_at_depth_6_without_helpers`, `cargo test recursive_hierarchy_parents_can_emit_local_flops_at_depth_7`, `cargo test recursive_hierarchy_parent_composed_routes_mix_parent_ports_at_depth_7_without_helpers`, `cargo test recursive_hierarchy_parent_outputs_mix_parent_ports_at_depth_7_without_helpers`, and `cargo test recursive_hierarchy_stateful_parent_outputs_mix_parent_ports_at_depth_7_without_helpers` alongside the earlier recursive helper and registered-route tests. Hierarchy-aware identity is delivered (HIERARCHY-AWARE-IDENTITY tree, r85–r87). Broader registered hierarchy routing/composition is open-ended capability-deepening explicitly scope-cut out of the Phase 4 bar (no mode retired; optional post-Phase-4 `rN` work). Phase 4 is closed; Phase 5 (parameterization) is next. |
| 5 — Parameterization         | done         | `ir/types.rs` (`WidthExpr`/`ParamEnv`/`Instance.param_bindings`), `ir/param.rs`, `config.rs`, `gen/module.rs` (`build_parameterizable_leaf`), `gen/hierarchy.rs`, `ir/validate.rs`, `emit/sv.rs`, `metrics.rs`, `bin/tool_matrix.rs` | Closed `2026-05-17` (`PHASE-5-PARAMETERIZATION` tree). Rules-first width-homogeneous parameterizable leaves (`is_width_generic` soundness gate), opt-in `width_parameterization_prob` (default-off byte-identical), per-instance `#(.W(v))` overrides with resolved-width validate, parameter-aware `canonical_module_signature`. Closing artifact `/tmp/anvil-tool-matrix-phase5-p1` (213 scenarios / 852 designs, `coverage_gaps=[]`, 852/0 Verilator + both Yosys). Parameter-aware child selection / parameter-driven parent generation are open-ended post-phase work (scope-cut, not a blocker). |
| 5b — Synthesizable aggregates | done        | `ir/types.rs` (`AggregateLayout`/`AggregateKind`/`AggregateGroup`), `ir/aggregate.rs`, `ir/mod.rs`, `config.rs` (`aggregate_prob`), `gen/mod.rs`, `emit/sv.rs`, `metrics.rs`, `bin/tool_matrix.rs` | Closed `2026-05-18` (`PHASE-5B-AGGREGATES` tree). Architecture (P) emitter-only packed-`struct` projection: additive `Default`-able `Module.aggregate_layout` annotation consulted only by the emitter; flat IR / validators / CSE / `canonical_module_signature` untouched (projected twin dedup-collapses). Non-rolling `annotate_aggregate` pass + seeded per-module roll at the `gen/mod.rs` post-pass scoped to **non-instantiated** modules; boundary-alias emitter (`typedef struct packed` + one aggregate port/side + alias wires/assigns); opt-in `aggregate_prob` (default-off byte-identical). Organic existence ~85% (no rules-first pivot). Closing artifact `/tmp/anvil-tool-matrix-phase5b-p1` (216 scenarios / 864 designs, `coverage_gaps=[]`, 864/0 Verilator + both Yosys, `saw_packed_aggregate_design=true`). Scaffold scope: `StructPacked` only / non-instantiated only / skips Phase 5 `param_env` modules — `union`/`array`, parent-side aggregate connections, param×aggregate cross-product are open-ended post-phase sub-slices (scope-cut, not a blocker). |
| 6 — Advanced motifs          | **done (2026-05-20)** | `ir/types.rs` (`Memory`/`MemKind`/`Node::MemRead`/`Fsm`/`FsmEncoding`/`Node::FsmOut`/`DepAtom::{MemVirtual,FsmVirtual}`), `ir/compact.rs` (load-bearing reachability), `ir/validate.rs` (steps 5b/5c), `config.rs` (`memory_prob`/`fsm_prob`), `gen/module.rs` (`build_memory_leaf`/`build_fsm_block`), `emit/sv.rs`, `metrics.rs`, `bin/tool_matrix.rs`, `tests/pipeline.rs` | **Phase 6 closed (2026-05-20, `PHASE-6-ADVANCED-MOTIFS` tree done).** Both substantive motifs landed and are verified downstream-clean against the banked `Phase4Hierarchy` gate. **Memory motif (delivered 2026-05-18, `.2` container done):** first-class `Memory` block (additive `Default`-empty `Module.memories`) + opaque `Node::MemRead` leaf (sibling to `FlopQ`, never CSE'd; load-bearing `compact.rs` reachability keeps `we`/`waddr`/`wdata`/`raddr` cones alive) + reset-less emitter `$mem_v2`-inferrable synchronous template + opt-in `memory_prob` (default-off byte-identical); closing artifact `/tmp/anvil-tool-matrix-phase6-p1` (219/876, `coverage_gaps=[]`, 876/0 Verilator + both Yosys, `saw_inferrable_memory_design=true`). `MEMORY-STATE-IDENTITY.1` confirmed the reset-defined boundary: a reset-all unpacked-array probe is Verilator-clean but Yosys warns and lowers it to registers, so the current memory-inference motif remains reset-less and memory state remains identity-by-instance. **FSM motif (delivered 2026-05-20, `.3.4b` done, closes Phase 6):** first-class `Fsm` block + opaque `Node::FsmOut` (sibling to `FlopQ`/`MemRead`, never CSE'd; same reachability obligation as `MemRead`) + encoding-derived emitter (binary / one-hot / gray) — async-reset state register + `always_comb` next-state / Moore-output `case`s on the shared `clk`/`rst_n` — behind opt-in `fsm_prob` (default-off byte-identical); closing artifact `/tmp/anvil-tool-matrix-phase6-fsm-p1` (222/888, `coverage_gaps=[]`, 888/0 Verilator + both Yosys, `saw_fsm_design=true` AND `saw_inferrable_memory_design=true`; P4/P5/P5b regressions still proven in the same banked report). Scaffold scope: memory `SinglePort`/`SimpleDualPort` only, `param_env`-skipped/non-instantiated; FSM Moore-only (Mealy is the recorded post-closure extension). The separately-prioritised multi-clock CDC follow-up is also closed (`MULTI-CLOCK-CDC`, 2026-05-24), adding opt-in multi-clock promotion plus a by-construction 2-flop synchronizer lane. |
| 7 — Oracle-backed micro-design artifacts | **done (2026-05-20)** | `src/microdesign/mod.rs` (own source-level const-expr/parameter IR + construction-time oracle + `expr_to_sv` + `emit_sv` + `Manifest` + `emit_manifest` + `ToolReport`/`Divergence`/`FactCategory`/`ParityScope`/`compare_manifest_to_tool_report_in_scope` parity comparator core); `tests/microdesign_parity.rs` (15 cargo-portable proofs + 1 tool-gated `#[ignore]` `parity_against_real_yosys_write_json` end-to-end harness with the yosys-specific `parse_yosys_binary_param` + `yosys_write_json_to_tool_report` extractor) | **Phase 7 closed (`PHASE-7-ORACLE-MICRODESIGN` tree done, 2026-05-20):** `rtl_const_expr`-family micro-designs delivered. Generator IS the oracle: every const-expr/parameter value is resolved at construction time (one `ChaCha8` stream per seed) and shipped in a JSON manifest while held symbolic in the emitted `.sv` (the gap = front-end elaboration). Parity gate against real yosys 0.64 verified clean on closing artifact `/tmp/anvil-microdesign-parity-phase7-yosys-p1/` (5 reproducibility seeds × {`.sv`, `.json`, `.yosys.json`}; `cargo test -- --ignored parity_against_real_yosys_write_json` exits 0 with "parity gate clean across 5 seeds"); per-seed fact agreement verified including the previously-divergent seed 7 (P4=-1; both sides bits=8 post-`.2c.2b.1` non-negative-modulo-idiom fix) and both generate branches (seed 12345 takes `g_else`, others take `g_taken`). The closing run found and fixed an ANVIL-self-consistency bug in `width_expr` (oracle used `rem_euclid`, SV used `%`; diverged for negative `last.value`) — exactly what `.1` designed the gate to surface. Scope caveat: yosys 0.64 `write_json` exposes 4 of 7 manifest fact categories (Seed/Top/Params/Widths/Generate); localparams + package-constants are folded — richer-AST coverage via a future microdesign-specific AST extractor is a recorded post-Phase-7 follow-up that does NOT retract closure (ANVIL's by-construction oracle already covers all 7 categories). DUT lane stays byte-identical by construction (microdesign is a separate top-level module never invoked from `src/gen/`). |
| 8 — Frontend/elaboration accept corpora | **done (2026-05-20)** | `src/frontend/mod.rs` (own source-level AST IR `SourceUnit`/`Package`/`Module`/`ModuleItem`/`Instance`/`GenerateIf`/`ParamDecl`/`ParamBinding` + `elaborate()` construction-time elaboration-evaluator + `emit_sv` + `emit_manifest` + the Phase-8-specific parity comparator `ToolReport`/`InstanceToolReport`/`Divergence` × 23 variants/`FactCategory`/`ParityScope`/`compare_manifest_to_tool_report_in_scope`/`synthetic_tool_report_from_manifest` with hierarchy-aware `Instance*` additions); `tests/frontend_parity.rs` (15 cargo-portable proofs + 3 tool-gated `#[ignore]` tests incl. `parity_against_real_yosys_hierarchy_write_json` and `parity_against_real_verilator_json_frontend_ast`; Yosys extractor reads `.cells[<inst>].{type, parameters}`, Verilator JSON extractor reads top/package param `VAR` values, specialized child-module GPARAMs reached through `CELL.modp`, and surviving `GENBLOCK`s) | **Phase 8 closed (`PHASE-8-FRONTEND-ACCEPT` tree done, 2026-05-20):** depth-1 elaboratable hierarchies delivered (one package + one top module + N child stub instances + chained body localparams + named-binding parameter overrides + generate-if). Generator IS the oracle: every `ParamDecl.value`/`ParamBinding.resolved`/`GenerateIf.taken` is resolved at construction time (one `ChaCha8` stream per seed) and shipped in a JSON manifest while held *symbolic* in the emitted `.sv`. Parity gate against real yosys 0.64 verified clean on closing artifact `/tmp/anvil-frontend-parity-phase8-yosys-p1/` (5 reproducibility seeds × {`.sv`, `.json`, `.yosys.json`}; `cargo test -- --ignored parity_against_real_yosys_hierarchy_write_json` exits 0 with "parity gate clean across 5 seeds" on **first try**); per-seed fact agreement verified including both generate branches exercised (seed 12345 takes `g_else`, others take `g_taken`) AND the load-bearing hierarchy-aware Phase-8 axis (every seed has 2 instances × 4 per-instance per-binding values matched against yosys's `.cells[<inst>].parameters`). **Cross-tree reuse of Phase 7's `ConstExpr`/`eval`/`expr_to_sv`** kept the full-factorization doctrine satisfied AND carried Phase 7's `.2c.2b.1` non-negative-modulo-idiom fix forward at zero incremental cost — Phase 8's gate came back clean on first try, contrast with Phase 7's needing a fix-and-retry. Scope caveat: yosys 0.64 `hierarchy + write_json` exposes 5 of 7 manifest fact categories (Seed/Top/TopParams/Instances/GenerateBranches); top_localparams + package_constants are folded. `SIGNOFF-SURFACE-EXPANSION.2` adds the optional Verilator JSON-AST gate for local builds supporting `--json-only`; it enforces all 7 categories via `ParityScope::all()` and is clean across the same 5 seeds with artifacts in `target/tmp/frontend-parity-signoff-verilator-json`. `slang` was absent locally and is not required for this path. An empirical-probe-driven discovery during `.2c.2`'s split — that yosys's `proc; opt` collapses empty-bodied child instances out of `.cells` — was the only Phase-8-specific Yosys capability dependency surfaced, and was folded into the `.2c.2a` extractor's invocation (`hierarchy -top` only, no `proc; opt`). DUT lane stays byte-identical by construction (`frontend` is a separate top-level module never invoked from `src/gen/`). |
| 9 — Multi-artifact umbrella  | **done (2026-05-20)** | `src/umbrella/mod.rs` (`ArtifactLane` trait + `LaneArtifact` carrier + `CheckPlan` enum + `LaneError` + `DutLane`/`MicrodesignLane`/`FrontendLane` impls + 8 cargo-portable proofs incl. per-lane byte-identical regression + cross-lane heterogeneous `dyn` dispatch); `src/main.rs` (`--artifact <lane>` CLI flag with `ArtifactKind::{Dut,Microdesign,Frontend}`; default `dut` falls through to historical code path UNCHANGED via early-return guard; `run_non_dut_lane` helper dispatches via `Box<dyn ArtifactLane>`); load-bearing byte-identical default-`dut` contract verified by `tests/book_examples::every_runnable_book_bash_block_succeeds` passing 3/3 in 80s AFTER the CLI change | **Phase 9 closed (`PHASE-9-MULTI-ARTIFACT-UMBRELLA` tree done, 2026-05-20):** the artifact-family selector + shared plumbing landed; ANVIL now ships THREE complementary lanes selectable via one tool (DUT RTL Phases 1–6 + microdesign Phase 7 + frontend Phase 8). The explicit anti-goal from `.1` is preserved: only the plumbing (seed→artifact, byte-stable output, optional manifest, downstream check plan) unifies; the three lanes' rules-first generators stay decoupled in their own modules. The default `--artifact dut` invocation is byte-identical to today's no-flag invocation — load-bearing for `BOOK-EXAMPLES-RUNNABLE` + every CI gate, enforced from `.2a` forward by `dut_lane_is_byte_identical_to_direct_generator_path` AND verified end-to-end at `.2c` by `every_runnable_book_bash_block_succeeds`. The cross-lane heterogeneous dispatch proof (landed in `.2b`) made the CLI dispatch correct-by-construction the moment it compiled. **All 9 numbered roadmap phases now delivered.** The post-phase `DIFFERENTIAL-SIMULATION` and `MULTI-CLOCK-CDC` trees are closed as of 2026-05-24; the five 2026-06-05 post-phase follow-up trees in `docs/TASK_TREE.md` are now closed or explicitly bounded at their current proof/tool limits. |

## Invariants currently enforced

In code (constructors / generator):
- `WORKLOAD-MEMORY-SAFETY.3` enforces a per-module construction-time node
  budget. `Config::max_nodes_per_module` (sentinel `0` = unlimited,
  default) is checked by `cone::node_budget_reached`, OR-ed into both
  `force_leaf` decisions (`process_signal_frame`, `build_cone`) and the
  `build_graph_first` growth loop: once `Module::nodes` reaches the budget,
  construction steers to existing terminals (rules-first; never truncates a
  finished cone), bounding peak per-module memory. Default `0` keeps the
  recursion (and RNG) byte-identical; effect measured by `Metrics::num_nodes`.
- `SIGNOFF-SURFACE-EXPANSION.1` extends the closed multi-clock CDC
  lane from exact 2-flop synchronizers to configurable N-flop 1-bit
  synchronizer chains. `Config::cdc_synchronizer_stages` defaults to
  `2` and validates `>= 2`; `src/gen/multi_clock.rs` builds the chain
  by construction in the destination domain; `Metrics` now separates
  exact-2 counts from stage-count-agnostic chain counts and maximum
  stage depth; `tool_matrix` has a dedicated
  `int_multi_clock_3flop_sync` scenario and
  `saw_cdc_nflop_synchronizer` coverage fact. General CDC fabrics
  (async FIFO, gray-code pointer transfer, req/ack word handshakes,
  pulse synchronizers, reset synchronizers) remain outside current
  ANVIL scope.
- `SIGNOFF-SURFACE-EXPANSION.2` extends the Phase-8 frontend parity
  harness with an optional Verilator JSON-AST extractor. It is
  test-harness only, not a DUT-generation path: cargo-portable tests
  prove the parser/extractor on synthetic JSON, and the ignored real
  gate `parity_against_real_verilator_json_frontend_ast` enforces all
  7 frontend manifest categories when Verilator supports `--json-only`.
- `SIGNOFF-SURFACE-EXPANSION.3` extends `tool_matrix` with an optional
  Icarus Verilog compile/elaboration column (`--iverilog-compile`).
  It shells `iverilog -g2012`, records `iverilog_compile` reports, and
  treats warnings as failures. The same slice changed static structured
  gate emission so constant-selector case/casez muxes and
  constant-source for-folds lower to continuous `assign` statements;
  dynamic selectors/sources still emit the procedural structured
  surfaces.
- `Module::intern_gate` / `intern_constant` enforce the currently-implemented combinational factorization ladder (Rule 21 / 21b / 21c): associative flattening, commutative sort on `And`/`Or`/`Xor`/`Add`/`Mul`, constant folding, peephole rewrites, then AST-cap CSE keyed by `(op, operands, width)` / `(width, value)`. `identity_mode = Relaxed` forces the effective level to `None`; `identity_mode = NodeId` uses `FactorizationLevel::effective()`, which now keeps the bounded `EGraph` fragment live at the top rung. Doctrinally, `node-id` still means full factorization (`NodeId` = expression identity); the ladder is the current build's enforcement/proof-depth dial inside that doctrine, not a competing definition of `node-id`.
- `Config::validate()` rejects out-of-range knobs.
- `Generator::new()` seeds RNG deterministically.
- `gen::module::generate_leaf_module` produces port counts within knob ranges.
- `gen::cone::build_cone_with_retry` retries up to 4× on empty-dep-set cone roots; snapshots `m.nodes`, `m.flops`, pool, worklist, `gate_instances`, `const_instances` before each attempt and restores on empty-dep retry.
- `gen::cone::build_cone` snapshots the same state before operand construction. On anti-collapse rejection, restores the snapshot and returns `pick_terminal` as fallback. No orphan leaks from rejected recursive gates.
- `gen::cone::process_signal_frame` (interleaved) uses an existing operand as anti-collapse fallback (not `pick_terminal`) because per-gate snapshot is infeasible once sibling frames have committed.
- `gen::module::summarize_flop_mux_metadata` clears construction-only mux operand references once `flop.d` exists, so metadata-only select/data cones do not survive liveness/compaction.
- `ir::compact::merge_equivalent_gates` is the first live post-construction combinational `EGraph` fragment. It runs only under `identity_mode = NodeId` with effective level `>= EGraph`, and merges gates by endpoint-preserving proof forms: same width, same canonical primary-input / flop-Q leaf endpoints, and same currently-proven functionality. For small-support cones the proof may be semantic (bounded truth table up to 12 endpoint-support bits, 128 cone nodes, and `assignment_count * cone_node_count <= 131072`); otherwise it falls back to the normalized structural proof. Different endpoint variables do not merge; `ENDPOINT-IDENTITY-BOUNDARY.1` proves same-shaped cones over disjoint primary-input endpoints stay distinct.
- `ir::compact::merge_equivalent_flops` is the first stateful extension of the NodeId-as-identity contract. It runs after D-cones exist, only under `identity_mode = NodeId` with effective level `>= Cse`, and merges flops by a reset/domain-safe proof subset: same `width`, `reset_kind`, `reset_val`, `Module::flop_domain`, and either the same D-cone proof over canonical primary-input / flop-Q endpoints or exact reset-defined self-hold (`D == own Q` on both flops). The ordinary D-cone proof is structural over the normalized IR by default, with the same bounded semantic truth-table signature used for small-support gate cones. The self-hold proof is the narrow coinductive exception: reset establishes equality and `D == Q` preserves it. The pass rewires duplicate Q consumers, remaps virtual flop deps, remaps explicit `flop_domains` entries, renumbers surviving flops, and rebuilds dedup tables. Different endpoint variables, different clock domains, reset-less self-hold, reset mismatches, and width mismatches do not merge.
- `gen::module::generate_leaf_module` now re-runs associative normalisation on the settled graph via `ir::compact::flatten_posthoc_associative_gates` after remap-producing passes (`fold_proven_gates`, `merge_equivalent_gates`). This keeps `nested_associative_operand_count` at zero for legal flattening opportunities and restores idempotent `And` / `Or` / `Xor` duplicate normal forms even when a later remap changes which already-built node an operand points at.
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
- `pick_terminal_dep_bearing(g, m, pool, width, exclude)` is a strict variant of `pick_terminal`: only admits (1) a random dep-bearing matching-width pool entry or (2) a width-adapter from the widest dep-bearing pool entry. Today it is used by the retired pool-only helpers (`build_graph_first`, comb-mux / priority-encoder / const-shift pool paths) and by active paths that must force an already-existing dep-bearing source, including constant-output repair in `src/gen/module.rs` and hierarchy child-input/helper/source-selection routes in `src/gen/hierarchy.rs`. The active recursive/interleaved leaf output-cone builders still construct most internal signals through `build_cone`. Panics if the pool has no dep-bearing entry (invariant). See `book/src/structural-rules.md` Rule 20.
- `pick_coefficient(g, width)` clamps the draw range to `[max(min_coefficient,1), min(max_coefficient, 2^W-1)]` so the emitted `width`-bit `Constant` can never overflow its declared width. Width=1 forces c=1; larger widths see the unclamped range up to `2^W-1`. See `book/src/structural-rules.md` Rule 19.
- Associative operators (`And`, `Or`, `Xor`, `Add`, `Mul`) are N-arity with N drawn from `[cfg.min_gate_arity, cfg.max_gate_arity]` each emission. `Sub` stays strictly 2-arity (not associative). Non-operators retain their natural operand counts. See `book/src/structural-rules.md` Rule 14 and the "Operators vs blocks" preamble.
- The full catalog of enforced invariants lives in `book/src/structural-rules.md`. This file's invariants lists above are a summary with pointers to the catalog.
- `pick_terminal` filters out the excluded `NodeId` from every candidate set (matching-width, dep-bearing, fallback adapter source).
- `build_cone`, `process_signal_frame`, `grow_pool_one_unit`, `pick_terminal`, and `drain_flop_worklist` route every leaf/cone probability choice through `roll_knob`, populating `m.knob_rolls` for measurability of `flop_prob`, `comb_mux_prob`, `priority_encoder_prob`, `coefficient_prob`, `const_shift_amount_prob`, `const_comparand_prob`, `constant_prob`, `terminal_reuse_prob`, `comb_mux_encoding_prob`, `flop_mux_encoding_prob`, `share_prob`, and `flop_qfeedback_prob`. Hierarchy binding helpers separately record the hierarchy probability knobs into the same `m.knob_rolls` sink: `hierarchy_sibling_route_prob`, `hierarchy_registered_sibling_route_prob`, `hierarchy_registered_child_input_cone_prob`, `hierarchy_child_input_cone_prob`, `hierarchy_parent_cone_instance_prob`, and `hierarchy_parent_flop_prob`.
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
- The generator now reserves module names through one global sequence,
  so hierarchy output also avoids cross-design filename collisions when
  `--count N --out DIR` emits multiple designs into the same directory.
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
- `src/gen/cone.rs` — 42 inline unit tests covering flop assemblers, `ceil_log2`, `pick_mux_arm_count`, width-adapter cases, comb-mux generation, DAG-sharing sanity, anti-collapse, dep-bearing terminal picking, coefficient-width clamping, dynamic overshift proofs, exact-selector `CaseMux` / `CasezMux` bounds cleanup, exact small-set budgeting, support caps, priority-encoder width-domain guards, selectable Slice/Concat shape guards, CLI alias behavior, and category / leaf-knob exercise coverage.
- `src/gen/mod.rs` — 1 inline unit test proving that a saved generator checkpoint reproduces the exact next module after restore.
- `src/gen/hierarchy.rs` — 6 inline unit tests covering control-port propagation, exact-profiled parent module shaping, recursive depth ranges, per-depth branching overrides, and current recursive hierarchy invariants.
- `src/gen/module.rs` — 4 inline unit tests covering primary-input width shrinking, the "do not shrink full-width non-slice uses" guard, instance-input binding width preservation, and the orphan-gate consumer audit for instance inputs.
- `src/emit/sv.rs` — 17 inline unit tests pinning emitter output on hand-built IRs: module header + endmodule + port declarations + passthrough assign, conditional omission of clk/rst_n when zero flops, canonical `always_ff @(posedge clk or negedge rst_n)` header with active-low reset branch, operator and constant rendering, Slice / Concat rendering, scalar-slice emission without illegal `[0:0]` on scalar `logic`, constant-slice folding to legal literals, Mux ternary form, both procedural case surfaces, the procedural bounded `for` surface, explicit unconnected child-output emission (`.port()`), and the exact hierarchy control-port doctrine for comb-only wrappers, direct sequential wrappers, and grandparent wrappers.
- `src/metrics.rs` — 20 inline unit tests for empty-module, per-kind gate, flop-shape metrics, constant-vs-variable shift-rhs classification, and hierarchy design metrics for reuse, under-instantiation, parent-side composition, direct sibling helper routes, parent-cone helper-instance output support, stateful parent-output helper mixed-support output metrics, budgeted parent-cone helper allocation, unregistered helper child-input mixed-support metrics, registered helper-sourced child-input D cones, direct registered sibling helper routes, stateful parent-composed helper child-input routes, stateful parent-composed helper child-input mixed-support metrics, direct registered sibling mixed-support metrics, bounded recursive tree shape, per-depth branching profiles, and profiled on-demand interface realization.
- `src/manifest.rs` — 3 inline unit tests (`WORKLOAD-MEMORY-SAFETY.2`): `streamed_matches_reference` and `streamed_matches_reference_for_designs` prove the streamed manifest is byte-identical to `serde_json::to_string_pretty` of the fully-assembled object across element counts (0/1/2/5/17 and nested designs), and `propagates_element_error` proves a per-element error aborts the write.
- `src/microdesign/mod.rs` — 7 inline unit tests. `.2a`:
  `eval_matches_known_values` (operator precedence, shift/bitwise,
  comparisons/logicals→1/0, truncating div/mod toward zero,
  ternary+unary, a localparam dependency chain),
  `eval_reports_div_by_zero_and_undefined_param` (defensive
  `EvalError` paths), `build_is_reproducible_and_seed_sensitive`
  (byte-identical IR+values per seed; distinct seeds differ),
  `stored_values_are_consistent_with_a_fresh_reeval` (the
  load-bearing oracle-no-drift invariant). `.2b`:
  `emit_sv_is_valid_unresolved_shape` (package/module/symbolic
  parameter+localparam/`PKG_REF`/`W_SIG`+`sig`/`generate
  if-else`/`endmodule`; chained decls render their symbolic expr),
  `manifest_mirrors_the_oracle` (valid JSON; every
  params/localparams/widths/generate/package_constants/const_exprs
  fact equals the `.2a` oracle), `sv_and_manifest_are_byte_reproducible`
  (same seed → identical `.sv`+`.json` across rebuilds; distinct
  seeds differ).
- `src/ir/compact.rs` — inline unit tests for bounded semantic gate merge, gate-to-endpoint semantic folding, endpoint-aware state merge, relaxed-mode bypass, reset-signature separation, self-feedback non-merge, cleanup exact-proof eligibility caps, 12-bit shallow semantic proof admission, merge/cleanup work-budget skips, the landed `ForFold` exact evaluator, late mixed-constant cleanup on the settled graph, post-remap idempotent duplicate cleanup, no-op compaction, orphan removal, dead-flop removal, strict post-remap duplicate protection, instance-input remapping during compaction, topological-order preservation, and the large-low-support semantic-merge budget guard.
- `src/bin/tool_matrix.rs` — 26 inline unit tests covering scenario-name uniqueness, full factorization-rung coverage, full construction-strategy coverage, coverage-gap detection, the Phase-1 / Phase-2 / Phase-3 / Phase-4 gate run-plan math, representative `share_prob`-sweep coverage, Phase-3 structured-surface coverage, the refreshed Phase-4 hierarchy coverage facts (wrapper and recursive depths, child-instance profiles, per-depth override profiles, reuse, under-instantiation, mixed parent-output coverage, parent-cone helper-output coverage, parent-output helper mixed-support coverage, stateful helper-through-flop mixed-support output coverage, unregistered helper child-input mixed-support coverage, stateful helper-through-flop child-input mixed-support coverage, registered helper-sourced child-input coverage, registered helper mixed-support coverage, registered mixed-support routing coverage, recursive non-top registered mixed-support coverage, multi-stage registered routing coverage, recursive non-top multi-stage registered no-helper routing coverage, recursive non-top direct-helper coverage, recursive non-top direct-registered-helper coverage, recursive non-top multi-stage direct-registered-helper coverage, recursive non-top helper-through-state coverage, recursive fact derivation from `DesignMetrics`, and required knob-attempt coverage including the plain `hierarchy_sibling_route_prob` route axis, the registered sibling mixed-support route axis, and recursive non-top registered sibling mixed-support coverage, recursive non-top unregistered parent-composed mixed-support child-input coverage, and recursive non-top parent-port-composed output coverage), design-level metrics/report embedding, design-level Yosys invocation shaping, legacy `.sv` bootstrap resume, same-binary generator-checkpoint resume for both module and design artifacts, `sv`-hash mismatch rejection, and legacy-checkpoint upgrade.
- `tests/pipeline.rs` — 79 integration tests covering cross-seed validity, reproducibility across strategies, motif sweeps, both constant- and variable-shift surfaces, the landed procedural case/casez/for-fold surfaces, the landed selectable `Slice` / `Concat` surface, the hierarchy surface (legacy depth-1 wrapper exact/reuse/under-instantiation plus bounded recursive tree shape, per-depth branching profiles, exact profiled on-demand child interfaces, sibling-routed child inputs, parent-composed child-input bindings, parent-cone helper-instance child-input bindings, direct sibling helper routes, recursive non-top direct sibling helper routes, recursive non-top direct registered sibling helper routes, recursive non-top multi-stage direct registered sibling helper routes, recursive non-top multi-stage registered sibling routes without helpers, recursive non-top multi-stage registered mixed-support routes without helpers, recursive non-top multi-stage registered parent-composed helper routes, recursive non-top registered parent-composed helper routes, recursive non-top registered parent-composed helper mixed-support routes, unregistered parent-composed helper child-input mixed-support routes, parent-cone helper-instance parent-output composition, recursive non-top parent-output helper mixed-support composition, budgeted parent-cone helper allocation, budgeted parent-output helper composition, recursive non-top parent-output helper budget composition, stateful parent-output helper routing through parent-local flops, recursive non-top stateful parent-output helper routing through parent-local flops, recursive non-top stateful parent-output helper budget composition, registered helper-sourced child-input D cones, direct registered sibling helper routes, multi-stage direct registered sibling routes through earlier parent-local Qs, multi-stage direct registered sibling helper routes through helper-sourced parent Qs, multi-stage registered parent-composed helper routes through helper-sourced parent Qs, stateful parent-composed helper child-input routes through parent-local flops, recursive non-top stateful parent-composed helper child-input routes through parent-local flops, local parent flops, registered sibling-routed child-input bindings, direct registered sibling mixed-support child-input bindings, recursive non-top direct registered sibling mixed-support child-input bindings, recursive non-top unregistered parent-composed mixed-support child-input bindings without helper instances, recursive non-top parent-port-composed parent-output bindings without helper instances or parent-local state, registered parent-composed child-input bindings, registered mixed-support child-input bindings, recursive non-top registered mixed-support child-input bindings, multi-stage registered parent-composed child-input bindings, recursive non-top multi-stage registered parent-composed child-input bindings without helpers, mixed parent-port / child-output parent outputs, and module-name uniqueness across batched hierarchy designs), the first parent-side composition surface over child outputs, all live gate categories, zero-orphan / zero-duplicate-operand doctrine guards, input-surface finalisation, associative / constant-fold / peephole / compaction counters, and knob-roll telemetry.
- `tests/book_examples.rs` — std-only mdBook copy-paste-runnable gate (`BOOK-EXAMPLES-RUNNABLE.2.2`, 2026-05-18). 3 tests: `every_runnable_book_bash_block_succeeds` (builds release `anvil` once; parses every ```bash fence in `book/src/*.md`; honours the `<!-- book-test: skip — <reason> -->` sentinel; substitutes `cargo run --release --`→`"$ANVIL"`; **panics** on any unclassified residual `cargo`/bare-`anvil`/external-tool so a gap can never be silent; runs each non-skipped block via `bash -eu -o pipefail` in a fresh temp CWD, offline, child stdio→temp files (not pipes — a default module is ≈86 KB > the ≈64 KB OS pipe buffer; a piped+undrained wait deadlocks), defensive 600 s timeout, asserts exit 0 — 54 runnable / 9 skip-sentineled), `harness_detects_a_broken_command` (negative control — a broken flag must fail, so green is non-vacuous), `skip_sentinels_have_reasons`. CI gates this via `cargo test` + the `.github/workflows/ci.yml` `mdbook test book` step.
- Current executed counts (`cargo test`, 2026-05-02): **228 unit-target tests + 79 integration tests = 307 passing tests** (+ `tests/book_examples.rs`: 3 tests, 54 runnable book blocks). Doc-tests: 0.
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
  latest full downstream-clean Phase 4 hierarchy gate is closed at
  `/tmp/anvil-tool-matrix-phase4-hierarchy-r87/tool_matrix_report.json`
  (840 designs, `coverage_gaps = []`, and 840/0 pass-fail in Verilator
  plus both repo-owned Yosys modes). That refreshed report now covers
  wrapper exact / reuse / under-instantiation plus recursive depth `2`,
  mixed recursive depth range `2:3`, explicit child-sourcing modes
  `library` and `on-demand`, child-instance profiles `2`, `4`, `2:3`,
  and `1:3`, the per-depth override profile `0=4:4,1=2:2`, real mixed
  shallow/deep recursive realization, real on-demand child sourcing,
  exact profiled child-interface synthesis, real parent-side composition above instance outputs, real sibling-routed hierarchy child inputs, real registered sibling-routed child inputs, direct registered sibling mixed-support child-input bindings, real registered parent-composed child-input bindings, registered mixed-support child-input bindings, recursive non-top registered mixed-support child-input bindings, multi-stage registered parent-composed child-input bindings, recursive non-top multi-stage registered parent-composed child-input bindings without helpers, multi-stage registered sibling-routed child-input bindings, recursive non-top multi-stage registered sibling-routed child-input bindings without helpers, recursive non-top multi-stage registered mixed-support child-input bindings without helpers, multi-stage direct registered sibling helper bindings, recursive non-top multi-stage direct registered sibling helper bindings, recursive non-top multi-stage registered parent-composed helper bindings, real parent-composed child-input bindings, parent-cone helper-instance child-input bindings, parent-output helper-instance composition, recursive non-top parent-output helper routing, recursive non-top stateful parent-output helper routing, recursive non-top parent-output multi-helper budget evidence, recursive non-top child-input multi-helper budget evidence, recursive non-top stateful multi-helper budget evidence, stateful parent-output helper routing through parent-local flops, recursive non-top stateful parent-output helper routing through parent-local flops, stateful parent-composed helper child-input routing through parent-local flops, recursive non-top stateful parent-composed helper child-input routing through parent-local flops, recursive non-top direct sibling helper routing, recursive non-top direct registered sibling helper routing, recursive non-top multi-stage direct registered sibling helper routing, recursive non-top multi-stage registered parent-composed helper routing, recursive non-top registered parent-composed helper routing, recursive non-top registered parent-composed helper mixed-support routing, recursive non-top parent-output helper mixed-support routing, budgeted multi-helper allocation, registered parent-composed helper-sourced child-input D cones, real mixed parent-port / child-output parent outputs, and real local parent flops, stateful helper-backed parent-output mixed-support routing, unregistered parent-composed helper child-input mixed-support routing, stateful helper-through-flop child-input mixed-support routing, direct registered sibling mixed-support routing, and recursive non-top direct registered sibling mixed-support routing, and recursive non-top unregistered parent-composed mixed-support child-input routing without helper instances, and recursive non-top parent-port-composed parent-output routing without helper instances or parent-local state, recursive non-top parent-port-composed parent-output routing that mixes parent data ports, child outputs, and parent-local Qs without helper instances, recursive non-top stateful unregistered parent-composed mixed-support child-input routing through parent-local Qs without helper instances, recursive non-top parent-local flops gated as a first-class coverage fact, recursive parent-local flops gated at exact hierarchy depth 3, recursive non-top unregistered parent-composed mixed-support child inputs gated at exact hierarchy depth 3 without helpers, recursive non-top parent-port-composed parent outputs gated at exact hierarchy depth 3 without helpers or state, and recursive non-top stateful parent-port-composed parent outputs gated at exact hierarchy depth 3 without helpers, and recursive non-top stateful parent-composed mixed-support child inputs gated at exact hierarchy depth 3 without helpers, and recursive non-top parent-local flops gated at exact hierarchy depth 4, and recursive non-top mixed-support child inputs gated at exact hierarchy depth 4 without helpers, and recursive non-top parent-port-composed parent outputs gated at exact hierarchy depth 4 without helpers or state, and recursive non-top stateful parent-port-composed parent outputs gated at exact hierarchy depth 4 without helpers, and recursive non-top stateful parent-composed mixed-support child inputs gated at exact hierarchy depth 4 without helpers, and recursive non-top parent-local flops gated at exact hierarchy depth 5, and recursive non-top mixed-support child inputs gated at exact hierarchy depth 5 without helpers, and recursive non-top parent-port-composed parent outputs gated at exact hierarchy depth 5 without helpers or state, and recursive non-top stateful parent-port-composed parent outputs gated at exact hierarchy depth 5 without helpers, and recursive non-top stateful unregistered parent-composed mixed-support child inputs gated at exact hierarchy depth 5 without helpers — closing the depth-5 sweep, and recursive non-top parent-local flops gated at exact hierarchy depth 6 — opening the depth-6 axis, and recursive non-top mixed-support child inputs gated at exact hierarchy depth 6 without helpers, and recursive non-top parent-port-composed parent outputs gated at exact hierarchy depth 6 without helpers or state, and recursive non-top stateful parent-port-composed parent outputs gated at exact hierarchy depth 6 without helpers, and recursive non-top stateful unregistered parent-composed mixed-support child inputs gated at exact hierarchy depth 6 without helpers (2,2 calibrated) — closing the depth-6 sweep, and recursive non-top parent-local flops gated at exact hierarchy depth 7 — opening the depth-7 axis, and recursive non-top mixed-support child inputs gated at exact hierarchy depth 7 without helpers (2,2 calibrated), and recursive non-top parent-port-composed parent outputs gated at exact hierarchy depth 7 without helpers or state, and recursive non-top stateful parent-port-composed parent outputs gated at exact hierarchy depth 7 without helpers, recursive non-top stateful unregistered parent-composed mixed-support child inputs gated at exact hierarchy depth 7 without helpers (2,2 calibrated) — closing the depth-7 sweep, recursive non-top registered parent-composed child-input bindings that chain through three or more parent-local flop stages without helpers — opening a chain-depth axis above the closed depth-3..7 sweeps, a recursive non-top internal parent saturating a parent-cone helper budget of 5 helpers — extending the helper-budget axis above the previous budget-3 baseline, and per-module canonical signatures as the first slice of hierarchy-aware identity instrumentation, plus a depth-1 wrapper-lane scenario proving the planner can emit structurally-duplicate Module definitions under tight constraints (HIERARCHY-AWARE-IDENTITY.2). The `r61` full downstream-clean report also records the direct sibling helper route, direct registered sibling helper route, stateful parent-output helper route, multi-stage direct registered sibling helper route, multi-stage registered parent-composed helper route, recursive non-top parent-output helper route, recursive non-top parent-output helper mixed-support route, recursive non-top stateful parent-output helper route, recursive non-top parent-output multi-helper budget evidence, recursive non-top child-input multi-helper budget evidence, recursive non-top stateful multi-helper budget evidence, recursive non-top registered mixed-support routing, recursive non-top multi-stage registered parent-composed no-helper routing, recursive non-top multi-stage registered sibling no-helper routing, recursive non-top multi-stage registered mixed-support no-helper routing, and recursive non-top registered parent-composed helper mixed-support routing. The stale-total-budget `r22` run is clean but insufficient root-cause evidence at 126 designs; `r23` is the historical pre-direct-helper full bank; `r24` is the historical coverage-only direct-helper policy proof; `r25` is the previous direct-helper full bank, `r26` is the previous multi-stage registered sibling bank, `r27` is the previous stateful parent-output helper bank, `r28` is the previous multi-stage direct registered sibling helper bank, `r29` is the previous multi-stage registered parent-composed helper bank, `r30` is the previous stateful parent-composed helper full bank, `r31` is the previous recursive helper-state full bank, `r32` is root-cause evidence for the exact-selector `CaseMux` / `CasezMux` shift-cleanup fix, `r33` is the pre-compact-normalization recursive direct-helper bank, `r34` is the previous recursive direct-helper full bank, `r35` is the previous recursive direct registered-helper full bank, `r36` is the previous recursive registered parent-composed helper full bank, `r37` is the previous recursive non-top multi-stage direct registered helper full bank, `r38` is the previous recursive non-top multi-stage registered parent-composed helper full bank, `r39` is the previous recursive non-top parent-output helper full bank, `r40` is the previous recursive non-top stateful parent-output helper full bank, `r41` is the previous recursive non-top parent-output multi-helper budget full bank, `r42` is the previous recursive non-top stateful multi-helper budget full bank, `r43` is the previous recursive non-top child-input multi-helper budget full bank, `r44` is the previous recursive non-top registered mixed-support routing full bank, `r45` is the previous recursive non-top registered parent-composed multistage no-helper full bank, `r46` is the previous recursive non-top registered sibling multistage no-helper full bank, `r47` is the previous recursive non-top registered mixed-support multistage no-helper full bank, `r48` is the previous recursive non-top registered parent-composed helper mixed-support full bank, `r49` is the previous recursive non-top parent-output helper mixed-support full bank, `r50` is the previous accumulated mixed-support hierarchy full bank, `r51` is the previous direct registered sibling mixed-support hierarchy full bank, and `r52` is the previous recursive direct registered sibling mixed-support hierarchy full bank, and `r53` is the previous recursive parent-composed mixed-support child-input hierarchy full bank, and `r54` is the previous recursive parent-port-composed parent-output hierarchy full bank, `r55` is the previous recursive stateful parent-port-composed parent-output hierarchy full bank, `r56` is the previous recursive stateful unregistered parent-composed mixed-support child-input hierarchy full bank, and `r57` is the previous hierarchy full bank that gated recursive non-top parent-local flops as a first-class coverage fact, `r58` is the previous hierarchy full bank that pushed recursive parent-local flops to exact hierarchy depth 3, `r59` is the previous hierarchy full bank that pushed recursive non-top unregistered parent-composed mixed-support child inputs to exact hierarchy depth 3, `r60` is the previous hierarchy full bank that pushed recursive non-top parent-port-composed parent outputs to exact hierarchy depth 3, `r61` is the previous hierarchy full bank that pushed recursive non-top stateful parent-port-composed parent outputs to exact hierarchy depth 3, and `r62` is the previous hierarchy full bank that closed the depth-3 push, `r63` is the previous hierarchy full bank that opened the depth-4 axis, and `r64` is the previous hierarchy full bank that extended the depth-4 axis to mixed-support child inputs, `r65` is the previous hierarchy full bank that extended the depth-4 axis to parent-port-composed parent outputs, and `r66` is the previous hierarchy full bank that extended the depth-4 axis to stateful parent-port-composed parent outputs, `r67` is the previous hierarchy full bank that closed the depth-4 sweep, `r68` is the previous hierarchy full bank that opened the depth-5 axis, `r69` is the previous hierarchy full bank that extended the depth-5 axis with mixed-support child inputs, `r70` is the previous hierarchy full bank that extended the depth-5 axis with parent-port-composed parent outputs, `r71` is the previous hierarchy full bank that extended the depth-5 axis with stateful parent-port-composed parent outputs, `r72` is the previous hierarchy full bank that closed the depth-5 sweep with stateful unregistered parent-composed mixed-support child inputs, `r73` is the previous hierarchy full bank that opened the depth-6 axis with parent-local flops, and `r74` is the previous hierarchy full bank that extended the depth-6 axis with mixed-support child inputs, `r75` is the previous hierarchy full bank that extended the depth-6 axis with parent-port-composed parent outputs, and `r76` is the previous hierarchy full bank that extended the depth-6 axis with stateful parent-port-composed parent outputs, `r77` is the previous hierarchy full bank that closed the depth-6 sweep with stateful unregistered parent-composed mixed-support child inputs, `r78` is the previous hierarchy full bank that opened the depth-7 axis, and `r79` is the previous hierarchy full bank that extended the depth-7 axis with mixed-support child inputs, and `r80` is the previous hierarchy full bank that extended the depth-7 axis with parent-port-composed parent outputs, and `r81` is the previous hierarchy full bank that extended the depth-7 axis with stateful parent-port-composed parent outputs. The `r82` full downstream-clean report records
  `saw_hierarchy_parent_port_composed_outputs = true`,
  `saw_hierarchy_registered_mixed_support_routing = true`,
  `saw_hierarchy_registered_sibling_mixed_support_routing = true`,
  `saw_recursive_hierarchy_registered_sibling_mixed_support_routing = true`,
  `saw_hierarchy_mixed_support_child_inputs = true`,
  `saw_recursive_hierarchy_mixed_support_child_inputs = true`,
  `saw_recursive_hierarchy_parent_port_composed_outputs = true`,
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
  `saw_hierarchy_parent_cone_instance_routing = true`,
  `saw_hierarchy_parent_cone_instance_outputs = true`,
  `saw_recursive_hierarchy_parent_cone_instance_outputs = true`,
  `saw_recursive_hierarchy_parent_cone_instance_mixed_support_outputs = true`,
  `saw_recursive_hierarchy_parent_cone_instance_flop_outputs = true`,
  `saw_recursive_multiple_parent_cone_instances_per_parent = true`,
  `saw_recursive_multiple_parent_cone_instances_per_parent_child_inputs = true`,
  `saw_recursive_multiple_parent_cone_instances_per_parent_through_flops = true`,
  `saw_multiple_parent_cone_instances_per_parent = true`,
  `saw_hierarchy_registered_parent_cone_instance_routing = true`,
  `saw_recursive_hierarchy_registered_parent_cone_instance_mixed_support_routing = true`,
  `saw_hierarchy_direct_sibling_parent_cone_instance_routing = true`,
  `saw_hierarchy_direct_registered_sibling_parent_cone_instance_routing = true`,
  `saw_recursive_hierarchy_direct_sibling_parent_cone_instance_routing = true`,
  `saw_recursive_hierarchy_parent_composed_parent_cone_instance_flop_routing = true`,
  `saw_hierarchy_parent_cone_instance_flop_mixed_support_outputs = true`,
  `saw_recursive_hierarchy_parent_cone_instance_flop_mixed_support_outputs = true`,
  `saw_hierarchy_parent_cone_instance_mixed_support_routing = true`,
  `saw_recursive_hierarchy_parent_cone_instance_mixed_support_routing = true`,
  `saw_hierarchy_parent_composed_parent_cone_instance_flop_mixed_support_routing = true`,
  and
  `saw_recursive_hierarchy_parent_composed_parent_cone_instance_flop_mixed_support_routing = true`, so the earlier
  coverage-only probes at
  `/tmp/anvil-tool-matrix-phase4-parent-port-coverage-r1/tool_matrix_report.json`,
  `/tmp/anvil-tool-matrix-phase4-registered-mixed-r1/tool_matrix_report.json`,
  and `/tmp/anvil-tool-matrix-phase4-registered-multistage-r1/tool_matrix_report.json`
  are now historical policy breadcrumbs rather than the strongest
  current evidence. The earlier coverage-only proof at
  `/tmp/anvil-tool-matrix-phase4-recursive-direct-helper-r32/tool_matrix_report.json`
  and
  `/tmp/anvil-tool-matrix-phase4-recursive-helper-state-r31/tool_matrix_report.json`
  are now historical policy breadcrumbs because the full `r87` bank
  carries the recursive non-top helper routes through Verilator and both
  repo-owned Yosys modes.
  The focused smokes at
  `/tmp/anvil-hier-reuse-smoke-r1`,
  `/tmp/anvil-hier-under-smoke-r2`,
  `/tmp/anvil-hier-range-smoke-r1`, and
  `/tmp/anvil-hier-depth-profile-smoke-r1`, and
  `/tmp/anvil-hier-mixed-depth-smoke-r1`, and
  `/tmp/anvil-hier-profiled-ondemand-smoke-r1`, and
  `/tmp/anvil-hier-sibling-routing-smoke-r1`,
  `/tmp/anvil-hier-child-input-cone-smoke-r1`,
  `/tmp/anvil-hier-parent-state-smoke-r1`,
  `/tmp/anvil-hier-registered-sibling-smoke-r1`,
  `/tmp/anvil-hier-registered-child-input-cone-smoke-r2`,
  `/tmp/anvil-parent-cone-instance-smoke-r1`, and
  `cargo test recursive_hierarchy_sibling_routes_can_use_helper_instances_below_top`, and
  `cargo test recursive_hierarchy_registered_sibling_routes_can_use_helper_instances_below_top`,
  `cargo test recursive_hierarchy_registered_sibling_routes_can_chain_helper_instances_below_top`,
  `cargo test recursive_hierarchy_registered_sibling_routes_can_chain_without_helpers_below_top`, and
  `cargo test hierarchy_sibling_routes_can_use_helper_instances`, and
  `cargo test hierarchy_registered_sibling_routes_can_use_helper_instances` remain useful targeted
  proofs, while the old `/tmp/anvil-tool-matrix-phase4-hierarchy-r7`
  report is now the historical wrapper-baseline artifact, `r9` is the
  pre-mixed recursive bank, `r10` is the pre-on-demand mixed-depth
  bank, `r11` is the first explicit child-sourcing bank, `r15` is the
  pre-parent-state bank, `r16` is the pre-registered-sibling-route
  bank, `r17` is the pre-registered-parent-composed-route bank, `r18`
  is the first registered-parent-composed bank, `r20` is the
  pre-parent-cone helper-instance bank, `r31` is the previous recursive
  helper-state bank, `r32` is the failed direct-helper run that exposed
  the CaseMux/Casez warning-cleanup gap, `r33` is the
  pre-compact-normalization recursive direct-helper bank, `r34` is the
  previous recursive direct-helper bank, `r35` is the previous recursive
  direct registered-helper bank, `r36` is the previous recursive registered parent-composed helper bank, `r37` is the previous recursive non-top multi-stage direct registered helper bank, `r38` is the previous recursive non-top multi-stage registered parent-composed helper bank, `r39` is the previous recursive non-top parent-output helper bank, `r40` is the previous recursive non-top stateful parent-output helper bank, `r41` is the previous recursive non-top parent-output multi-helper budget bank, `r42` is the previous recursive non-top stateful multi-helper budget bank, `r43` is the previous recursive non-top child-input multi-helper budget bank, `r44` is the previous recursive non-top registered mixed-support routing bank, `r45` is the previous recursive non-top registered parent-composed multistage no-helper bank, `r46` is the previous recursive non-top registered sibling multistage no-helper bank, `r47` is the previous recursive non-top registered mixed-support multistage no-helper bank, `r48` is the previous recursive non-top registered parent-composed helper mixed-support bank, `r49` is the previous recursive non-top parent-output helper mixed-support bank, `r50` is the previous accumulated mixed-support hierarchy full bank, `r51` is the previous direct registered sibling mixed-support hierarchy full bank, `r52` is the previous recursive direct registered sibling mixed-support hierarchy full bank, `r53` is the previous recursive parent-composed mixed-support child-input hierarchy full bank, `r54` is the previous recursive parent-port-composed parent-output hierarchy full bank, `r55` is the previous recursive stateful parent-port-composed parent-output hierarchy full bank, `r56` is the previous recursive stateful unregistered parent-composed mixed-support child-input hierarchy full bank, `r57` is the previous hierarchy full bank that gated recursive non-top parent-local flops as a first-class coverage fact, `r58` is the previous hierarchy full bank that pushed recursive parent-local flops to exact hierarchy depth 3, `r59` is the previous hierarchy full bank that pushed recursive non-top unregistered parent-composed mixed-support child inputs to exact hierarchy depth 3 without helpers, `r60` is the previous hierarchy full bank that pushed recursive non-top parent-port-composed parent outputs to exact hierarchy depth 3 without helpers or state, `r61` is the previous hierarchy full bank that pushed recursive non-top stateful parent-port-composed parent outputs to exact hierarchy depth 3 without helpers, `r62` is the previous hierarchy full bank that closed the depth-3 push with recursive non-top stateful parent-composed mixed-support child inputs at exact hierarchy depth 3 without helpers, `r63` is the previous hierarchy full bank that opened the depth-4 axis with recursive non-top parent-local flops at exact hierarchy depth 4, `r64` is the previous hierarchy full bank that extended the depth-4 axis with recursive non-top mixed-support child inputs at exact hierarchy depth 4 without helpers, `r65` is the previous hierarchy full bank that extended the depth-4 axis with recursive non-top parent-port-composed parent outputs at exact hierarchy depth 4 without helpers or state, `r66` is the previous hierarchy full bank that extended the depth-4 axis with recursive non-top stateful parent-port-composed parent outputs at exact hierarchy depth 4 without helpers, `r67` is the previous hierarchy full bank that closed the depth-4 sweep with recursive non-top stateful parent-composed mixed-support child inputs at exact hierarchy depth 4 without helpers, `r68` is the previous hierarchy full bank that opened the depth-5 axis with recursive non-top parent-local flops at exact hierarchy depth 5, `r69` is the previous hierarchy full bank that extended the depth-5 axis with recursive non-top mixed-support child inputs at exact hierarchy depth 5 without helpers, `r70` is the previous hierarchy full bank that extended the depth-5 axis with recursive non-top parent-port-composed parent outputs at exact hierarchy depth 5 without helpers or state, `r71` is the previous hierarchy full bank that extended the depth-5 axis with recursive non-top stateful parent-port-composed parent outputs at exact hierarchy depth 5 without helpers, `r72` is the previous hierarchy full bank that closed the depth-5 sweep with recursive non-top stateful unregistered parent-composed mixed-support child inputs at exact hierarchy depth 5 without helpers, `r73` is the previous hierarchy full bank that opened the depth-6 axis with recursive non-top parent-local flops at exact hierarchy depth 6, `r74` is the previous hierarchy full bank that extended the depth-6 axis with recursive non-top mixed-support child inputs at exact hierarchy depth 6 without helpers (2,2 calibrated), `r75` is the previous hierarchy full bank that extended the depth-6 axis with recursive non-top parent-port-composed parent outputs at exact hierarchy depth 6 without helpers or state, `r76` is the previous hierarchy full bank that extended the depth-6 axis with recursive non-top stateful parent-port-composed parent outputs at exact hierarchy depth 6 without helpers, `r77` is the previous hierarchy full bank that closed the depth-6 sweep with recursive non-top stateful unregistered parent-composed mixed-support child inputs at exact hierarchy depth 6 without helpers (2,2 calibrated), `r78` is the previous hierarchy full bank that opened the depth-7 axis with recursive non-top parent-local flops at exact hierarchy depth 7, `r79` is the previous hierarchy full bank that extended the depth-7 axis with recursive non-top mixed-support child inputs at exact hierarchy depth 7 without helpers (2,2 calibrated), `r80` is the previous hierarchy full bank that extended the depth-7 axis with recursive non-top parent-port-composed parent outputs at exact hierarchy depth 7 without helpers or state, `r81` is the previous hierarchy full bank that extended the depth-7 axis with recursive non-top stateful parent-port-composed parent outputs at exact hierarchy depth 7 without helpers, `r82` is the previous hierarchy full bank that closed the depth-7 sweep with recursive non-top stateful unregistered parent-composed mixed-support child inputs at exact hierarchy depth 7 without helpers (2,2 calibrated), `r83` is the previous hierarchy full bank that opened a chain-depth axis above the closed depth-3..7 sweeps with recursive non-top registered parent-composed three-stage chain coverage, `r84` is the previous hierarchy full bank that extended the helper-budget axis above the previous budget-3 baseline with recursive non-top parent-cone helper budget 5 coverage, `r85` is the previous hierarchy full bank that added canonical module signatures as the first slice of hierarchy-aware identity instrumentation, `r86` is the previous hierarchy full bank that proved the planner can emit structurally-duplicate Module definitions under tight constraints (HIERARCHY-AWARE-IDENTITY.2), `r87` is the current hierarchy full bank that implements the post-finalisation module-dedup pass under the opt-in `Config::hierarchy_module_dedup` knob and proves it downstream-clean (HIERARCHY-AWARE-IDENTITY.4 + .5; tree complete), and the aborted `r8`
  rerun is historical
  runtime evidence that the Phase 4 gate should use a
  hierarchy-focused sequential leaf profile instead of silently
  borrowing the fattest Phase 1 leaf-stress shape.

## Known weaknesses (visible in code today)

- The broader signoff-grade cleanliness matrix described in
  `ROADMAP.md` now has a repo-owned implementation in
  `src/bin/tool_matrix.rs`, and the focused smoke matrix is currently
  green after `SIGNOFF-SURFACE-EXPANSION.1`: 17/17 clean in Verilator
  and 17/17 clean in Yosys under `--yosys-mode without-abc`, with
  `coverage_gaps = []` and both CDC facts lit. The harness now treats
  warnings as failures, so "green" here means no errors and no
  warnings, not merely zero non-zero exits. The repo-owned gate surface
  now also includes the dedicated `--phase2-share-gate`, whose
  normalized `share_sweep` summary proves that stronger `share_prob`
  increases the *fraction* of shared nodes even though the raw shared
  node count falls as the graph collapses.
- `NodeId`-as-identity is still conservative for state, but it is no
  longer flop-only: endpoint-preserving duplicate flops and
  deterministic generated FSM blocks merge under the live proof
  discipline. Opt-in module-dedup identity exists for hierarchy
  templates; current memories remain state-by-instance under a focused
  full-factorization regression because their stored contents are not
  reset-defined; opt-in hierarchy module dedup remains structural-only
  under a focused regression; broader sequential equivalence,
  memory-state merging beyond that boundary, and deeper hierarchical
  equivalence remain open work.
- Phase 4 is no longer only the first depth-1 slice. The legacy exact
  wrapper lane is still real, and the repo-owned Phase 4 gate now also
  banks the current representative bounded-recursive hierarchy surface,
  including the mixed-depth recursive axis and explicit `library` vs
  `on-demand` child sourcing. Current HEAD now also has a real
  combinational sibling-routing surface, parent-composed child-input
  cone surface, parent-cone helper-instance child-input route,
  parent-cone helper-instance parent-output route,
  budgeted parent-cone helper allocation,
  registered helper-sourced child-input D cones,
  direct sibling helper routing,
  direct registered sibling helper routing,
  multi-stage direct registered sibling helper routing,
  optional local parent flops, a registered sibling-route surface that
  can now chain through earlier parent-local Qs, a registered parent-composed
  child-input route surface, and mixed parent-port / child-output
  parent outputs. Current HEAD also lets direct registered sibling routes
  mix parent-port support into the sibling/helper-backed D path without
  registered parent-composed classification. Current HEAD also lets the registered
  parent-composed child-input route mix parent data ports with sibling
  outputs and chain through earlier parent-local Qs, and lets
  parent-composed helper child-input routes consume helper-sourced
  parent-local Qs without becoming registered child-input bindings,
  including below the top parent in an exact-depth-2 recursive
  hierarchy. Direct registered sibling routing can also chain through
  earlier parent-local Qs below the top parent without helper instances
  or parent-composed D logic, and the registered mixed-support route now
  has a direct metric/proof for combining parent ports, child outputs,
  and earlier parent-local Qs below the top parent without helpers.
  Direct sibling helper routing, direct
  registered sibling
  helper routing, multi-stage direct registered sibling helper routing,
  registered parent-composed helper D-cone routing, registered
  parent-composed helper mixed-support routing, parent-output helper
  routing, parent-output helper mixed-support routing, stateful parent-output helper routing, and multi-helper budget evidence are also proved below the top parent in the recursive
  exact-depth-2 lane. These surfaces are proved
  numerically in focused smokes and the full downstream-clean `r87`
  hierarchy bank. The `r87` bank requires
  `saw_hierarchy_parent_port_composed_outputs`,
  `saw_hierarchy_registered_mixed_support_routing`,
  `saw_hierarchy_registered_sibling_mixed_support_routing`,
  `saw_recursive_hierarchy_registered_sibling_mixed_support_routing`,
  `saw_recursive_hierarchy_registered_mixed_support_routing`,
  `saw_hierarchy_registered_multistage_routing`,
  `saw_recursive_hierarchy_registered_multistage_routing`,
  `saw_recursive_hierarchy_registered_multistage_mixed_support_routing`,
  `saw_hierarchy_registered_multistage_sibling_routing`,
  `saw_recursive_hierarchy_registered_multistage_sibling_routing`, and
  `saw_hierarchy_registered_multistage_parent_cone_instance_routing`,
  `saw_hierarchy_parent_composed_parent_cone_instance_flop_routing`,
  `saw_hierarchy_parent_cone_instance_routing`,
  `saw_hierarchy_parent_cone_instance_outputs`,
  `saw_recursive_multiple_parent_cone_instances_per_parent`,
  `saw_multiple_parent_cone_instances_per_parent`,
  `saw_hierarchy_registered_parent_cone_instance_routing`,
  `saw_hierarchy_direct_sibling_parent_cone_instance_routing`,
  `saw_hierarchy_direct_registered_sibling_parent_cone_instance_routing`,
  `saw_recursive_hierarchy_direct_sibling_parent_cone_instance_routing`,
  `saw_recursive_hierarchy_direct_registered_sibling_parent_cone_instance_routing`,
  `saw_recursive_hierarchy_registered_multistage_parent_cone_instance_routing`,
  `saw_recursive_hierarchy_registered_multistage_parent_composed_parent_cone_instance_routing`,
  `saw_recursive_hierarchy_registered_parent_composed_parent_cone_instance_routing`,
  `saw_recursive_hierarchy_registered_parent_cone_instance_mixed_support_routing`,
  `saw_recursive_hierarchy_parent_cone_instance_mixed_support_outputs`,
  `saw_recursive_hierarchy_parent_composed_parent_cone_instance_flop_routing`,
  `saw_hierarchy_parent_cone_instance_flop_mixed_support_outputs`,
  `saw_recursive_hierarchy_parent_cone_instance_flop_mixed_support_outputs`,
  `saw_hierarchy_parent_cone_instance_mixed_support_routing`,
  `saw_recursive_hierarchy_parent_cone_instance_mixed_support_routing`,
  `saw_hierarchy_parent_composed_parent_cone_instance_flop_mixed_support_routing`, and
  `saw_recursive_hierarchy_parent_composed_parent_cone_instance_flop_mixed_support_routing`.
  The next honest work is broader helper-instance placement beyond the current
  parent-composed child-input, stateful parent-composed child-input,
  recursive non-top stateful parent-composed child-input,
  recursive non-top direct sibling, recursive non-top direct registered
  sibling, recursive non-top multi-stage direct registered sibling,
  recursive non-top multi-stage registered parent-composed helper,
  recursive non-top registered parent-composed helper, recursive non-top parent-output helper, recursive non-top stateful parent-output helper, recursive non-top parent-output multi-helper budget, recursive non-top child-input multi-helper budget, recursive non-top stateful multi-helper budget, direct sibling,
  direct registered sibling, registered child-input,
  budgeted parent-output helper, stateful parent-output helper, and
  multi-stage direct registered helper slices,
  broader registered
  hierarchy routing/composition where it is structurally warranted, and
  future hierarchy-aware identity.
- `emit::sv::render_gate` for `Concat` joins operand names with commas (correct SV); the IR does not currently distinguish per-operand widths in storage because every current producer of `Concat` either replicates a single source or concatenates uniform-width bits. When variadic `Concat` with mixed widths becomes a real motif, the IR shape is still adequate (widths are a property of each operand node, not of the `Concat` itself), but a generator-side helper will need to compose such shapes carefully.

## Build hygiene
- `cargo check --all-targets` — clean.
- `cargo test` — monitored full-suite attempt stopped at 90.7% RAM per
  the resource-safety rule; not a completed full-suite result. Focused
  cargo tests for the new CDC/config/matrix paths are clean.
- `cargo test --test snapshots` — clean (6/6 byte-identical snapshot
  guard).
- `cargo test --test book_examples` — clean (3/3).
- `cargo clippy --all-targets -- -D warnings` — clean.
- `cargo fmt --all --check` — clean.
- `mdbook build book` — clean.
- `knowledge-map/scripts/check_knowledge_map.sh` and
  `scripts/check_memory_architecture.sh` — clean.
- Generator-output smoke: focused current default `tool_matrix`
  (`cargo run --bin tool_matrix -- --out
  /tmp/anvil-signoff-surface-nflop-r1 --fail-on-coverage-gap
  --yosys-mode without-abc`) is 17/17 clean in Verilator and 17/17
  clean in Yosys, `coverage_gaps = []`, with
  `saw_multi_clock_design`, `saw_cdc_2_flop_synchronizer`, and
  `saw_cdc_nflop_synchronizer` all true. Historical larger banks remain
  useful evidence for the pre-`SIGNOFF-SURFACE-EXPANSION.1` surface,
  including `/tmp/anvil-tool-matrix-phase1-real-r21` (1005/0 in
  Verilator and both repo-owned Yosys modes), Phase 2 share r1, Phase 3
  structured r4, and Phase 4 hierarchy r87.
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
  cone. The cheap layer also follows exact `CaseMux` and `CasezMux`
  selector arms, and falls back to conservative arm unions when the
  selector is not exact, so procedural case shapes feed the same
  warning-clean shift bounds as ternary muxes. This is an enforced
  output-cleanliness invariant, not a user
  knob.
