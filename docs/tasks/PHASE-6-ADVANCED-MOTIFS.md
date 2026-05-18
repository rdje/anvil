# PHASE-6-ADVANCED-MOTIFS: Memories, FSMs, optional multi-clock

## Metadata

- Tree ID: `PHASE-6-ADVANCED-MOTIFS`
- Status: `active`
- Roadmap lane: Phase 6 — Advanced motifs
- Created: `2026-05-16`
- Last updated: `2026-05-18` (`.3.4` split → `.3.4a` done [phase6_fsm scenario+metric+gap, 222/888] / `.3.4b` gate-blocked; frontier: `.2.4` gate-blocked ‖ `.3.4b` — last Phase 6 leaf)
- Owner: repo-local workflow

## Goal

Add the legal interaction richness needed to surface downstream tool
bugs without sacrificing downstream acceptance: inferrable memories
(single-port, dual-port, inferrable patterns only), FSMs with explicitly
generated state encodings, and — optional, expensive — CDC-safe
multi-clock handshakes.

## Non-Goals

- Non-inferrable / non-synthesizable memory patterns.
- Behavioural FSM intent or reachability guarantees (states may be
  functionally arbitrary; only the encoding/structure is generated).
- Making multi-clock mandatory: until/unless the multi-clock leaf lands,
  every module stays fully synchronous to a single clock.

## Acceptance Criteria

- Inferrable memory motifs emitted, valid by construction,
  downstream-clean and recognised as memory by Yosys where intended.
- Generated-state-encoding FSM motif, downstream-clean.
- Optional multi-clock CDC-safe handshake motif (may be deferred with a
  recorded consequence if cost outweighs value).
- Per-motif matrix scenarios + docs/knobs.

## Task Tree

- ID: `PHASE-6-ADVANCED-MOTIFS`
  Status: `active`
  Goal: `Land inferrable memories and generated-encoding FSMs (multi-clock optional), downstream-clean.`
  Children: `PHASE-6-ADVANCED-MOTIFS.1` (done), `PHASE-6-ADVANCED-MOTIFS.2` (active container: `.2.1`–`.2.4`), `PHASE-6-ADVANCED-MOTIFS.3` (active container — FSM: `.3.1` done, `.3.2`–`.3.4`)

- ID: `PHASE-6-ADVANCED-MOTIFS.1`
  Status: `done`
  Goal: `Design the inferrable-memory motif (IR/emit shape, single vs dual port, write/read patterns Yosys infers as $mem, knob surface, proof shape, rejected alternatives) in DEVELOPMENT_NOTES.md. Design-only.`
  Acceptance: `DEVELOPMENT_NOTES.md Phase 6 memory design entry with >=1 rejected alternative; mdbook clean; no code change.`
  Verification: `DEVELOPMENT_NOTES.md "Phase 6 inferrable-memory motif design (2026-05-18, PHASE-6-ADVANCED-MOTIFS.1)" entry landed: codebase-grounded (IR has no array/memory concept — scalar u32 Port/Node/Flop; Flop is the only stateful element; operators-vs-blocks doctrine → memory is a block). Empirical Yosys probe (resolves the Open Question): single-port sync RAM and simple dual-port templates both yield exactly 1 $mem_v2 under proc;opt;memory_collect, verilator --lint-only exit 0, and synth -noabc / synth;abc -fast both exit 0 with check -assert (clean in both repo Yosys modes). Chosen architecture (M): first-class Memory block (additive Vec<Memory> on Module, Default-empty) + opaque Node::MemRead leaf (sibling to FlopQ, never CSE'd) + emitter renders the validated inferrable template on the shared clk + opt-in Config::memory_prob serde-default 0.0. Three rejected alternatives: (A) flop-array+mux (not $mem-inferred — defeats the purpose), (B) emitter-only string template (not valid-by-construction), (C) generic unpacked-array datatype threaded through width arithmetic (massive invasive change; memory is a block not a datatype). Proof shape for .2 specified (default-off byte-identical; forced-on memory_collect ≥1 $mem_v2 both modes; matrix scenario+metric+gap+non-vacuity, no promotion until verified gate — Phase 5/5b .2.x decomposition). Doc-only; no code. mdbook build clean; cargo fmt clean; cargo test unchanged-green (no src/tests touched since Phase 5b .2.3 green run).`
  Commit: `Docs: PHASE-6-ADVANCED-MOTIFS.1 inferrable-memory motif design`

- ID: `PHASE-6-ADVANCED-MOTIFS.2`
  Status: `active`
  Goal: `Implement the inferrable-memory motif per .1 (architecture (M)), opt-in, with a matrix scenario and a Yosys memory-inference proof. Split per the Splitting Rules + the r87 no-aspirational-claims precedent (gate scenario lands before any ROADMAP advance); mirrors the proven Phase 5/5b .2.x decomposition.`
  Children: `PHASE-6-ADVANCED-MOTIFS.2.1` (container: `.2.1a`, `.2.1b`), `.2.2`, `.2.3`, `.2.4`

- ID: `PHASE-6-ADVANCED-MOTIFS.2.1`
  Status: `done`
  Goal: `IR + emitter scaffold (architecture (M)). Split into .2.1a (IR core + opaque-stateful-leaf pipeline integration incl. the load-bearing compaction-reachability correctness) and .2.1b (knob + rules-first construction + default-off/forced-on focused proof) — see the 2026-05-18 Decision.`
  Children: `PHASE-6-ADVANCED-MOTIFS.2.1a` (done), `PHASE-6-ADVANCED-MOTIFS.2.1b` (done)

- ID: `PHASE-6-ADVANCED-MOTIFS.2.1a`
  Status: `done`
  Goal: `IR core + opaque-stateful-leaf pipeline integration. Add MemId; MemKind{SinglePort,SimpleDualPort}; Memory{id,addr_width,data_width,kind,we,waddr,wdata,raddr:NodeId}; additive Default-empty Module.memories: Vec<Memory>; new opaque leaf Node::MemRead{mem,width}; DepAtom::MemVirtual(MemId) + DepSet::from_mem_virtual. Thread MemRead through ALL exhaustive Node matches (compiler-as-completeness-oracle, ~20 sites) mirroring FlopQ as an opaque identity-by-instance leaf. **Load-bearing correctness (the discovered dependency): src/ir/compact.rs reachability/dead-elimination must, like FlopQ keeps its flop's D cone, make a reachable MemRead transitively keep the memory's we/waddr/wdata/raddr source cones alive; canonical_module_signature/StructuralNodeShape/LeafEndpoint get distinct MemRead arms; MemRead is never CSE'd/merged.** Emitter renders the .1-validated inferrable template + a memrd_<id> read signal; validator: widths consistent + MemRead resolves to a declared memory + control-port logic emits clk for memory-bearing modules. NO knob, NO generator wiring (no Memory is ever constructed yet → default-off trivially byte-identical).`
  Acceptance: `cargo fmt/clippy(-D warnings)/check/test green; unit tests: a hand-built Memory module round-trips IR->validate->emit (SV declares the array + synchronous write/read on clk), survives compact_node_ids with we/waddr/wdata/raddr cones intact (the reachability proof), MemRead is opaque to CSE (two memories' reads never merge), and canonical signature distinguishes a MemRead node. No book/ change.`
  Verification: `src/ir/types.rs: MemId; MemKind{SinglePort,SimpleDualPort}; Memory{id,addr_width,data_width,kind,we,waddr,wdata,raddr}; additive Default-empty Module.memories; opaque leaf Node::MemRead{mem,width}; DepAtom::MemVirtual + DepSet::from_mem_virtual; has_local_memories() and the sequential-state predicates OR it in (clk exposed for memory-only modules; has_local_flops untouched so flop emission gates unchanged). MemRead threaded through all ~21 exhaustive Node matches via cargo-check completeness oracle, mirroring FlopQ as an opaque identity-by-instance leaf (gen/cone.rs, gen/hierarchy.rs, gen/module.rs, ir/param.rs, metrics.rs incl. canonical_module_signature tag 6, ir/compact.rs). Load-bearing reachability: compact.rs walk gains a Node::MemRead arm that marks mem.{we,waddr,wdata,raddr} reachable (memories never dead-eliminated in Phase 6.2.1 → MemId stable, no remap) + StructuralNodeShape::MemRead + LeafEndpoint::MemRead (+ width()). Emitter: memrd_<id> helper + node_ref arm + per-memory `logic [DW-1:0] mem_<id> [0:2^AW-1]` + `logic memrd_<id>` decls + a reset-less `always_ff @(posedge clk)` synchronous write/read block. Validator: BadMemory/UndefinedMemoryNode/MemoryNodeWidthMismatch/DanglingMemRead/MemReadWidthMismatch + a step-5b that checks every Memory's widths/SinglePort-shared-addr and every MemRead resolves. 3 unit proofs in ir/compact.rs: memory_leaf_roundtrips_validate_and_emit, memread_keeps_memory_source_cones_through_compaction (the reachability proof — dead gate stripped, wdata XOR cone survives, validate+emit clean), memread_is_structurally_distinct_and_not_cse_merged (distinct canonical signature vs PrimaryInput; two memories' reads never merged). cargo fmt/clippy -D warnings/check --all-targets clean; lib mem tests 3/3; full cargo test (COMMIT.md gate — Verification Log). No generator/knob ⇒ default-off trivially byte-identical (no Memory constructed). No book/ change.`
  Commit: `Phase 6: PHASE-6-ADVANCED-MOTIFS.2.1a memory IR core + opaque-stateful-leaf pipeline integration`

- ID: `PHASE-6-ADVANCED-MOTIFS.2.1b`
  Status: `done`
  Goal: `Knob + rules-first construction + focused proof. Config::memory_prob (f64, serde-default 0.0, probability-range validated); rules-first build_memory_leaf (a clk + we/waddr/wdata/raddr-input, rdata-output combinational-free memory leaf, single opt-in roll in generate_leaf_module_with_interface_profile, mutually exclusive with the param lane); default-off byte-identical + forced-on focused proof.`
  Acceptance: `cargo fmt/clippy(-D warnings)/check/test green; focused proof: default-off byte-identical for fixed seeds across all ConstructionStrategy values; forced-on every single-module design is a memory leaf that validates and whose SV declares the inferrable array + synchronous write/read. No book/ change (book reconciliation is .2.4).`
  Verification: `src/config.rs: Config::memory_prob (serde-default 0.0 via default_memory_prob + Default-impl line + probability-range validation tuple entry), mirroring aggregate_prob/width_parameterization_prob. src/gen/module.rs: rules-first build_memory_leaf (shared clk/rst_n + we/waddr/wdata[+raddr] inputs + rdata output; one Memory{kind rolled SinglePort|SimpleDualPort via g.rng, addr_width 2..=4, data_width in the configured width band}; opaque MemRead drives rdata; no gates/flops) + a single opt-in roll in generate_leaf_module_with_interface_profile placed AFTER the Phase 5 param lane (mutually exclusive; interface_profile None only; default-off never enters → byte-identical). Focused proof inferrable_memory_is_default_off_and_constructs_when_forced_on: default-off byte-identical (no Memory, no mem_0 array) across 4 ConstructionStrategy × 6 seeds; forced-on (prob 1.0) every single-module design is a 1-Memory leaf that validate_design-passes, exposes a MemRead node, and emits the inferrable array + reset-less always_ff write/read. Real spot-check (binary, seed 3, memory_prob 1.0): emitted SV verilator --lint-only exit 0; yosys memory_collect → 1 $mem_v2; synth -noabc and synth;abc -fast both check -assert clean — the Phase 6 inference contract holds on real generated output (formalised in .2.2). cargo fmt/clippy -D warnings/check --all-targets clean; focused proof green; full cargo test (COMMIT.md gate — Verification Log). No book/ change.`
  Commit: `Phase 6: PHASE-6-ADVANCED-MOTIFS.2.1b memory_prob knob + rules-first build_memory_leaf`

- ID: `PHASE-6-ADVANCED-MOTIFS.2.2`
  Status: `done`
  Goal: `Soundness + Yosys-inference proof. (a) Forced-on memory module: Yosys memory_collect reports >=1 $mem_v2 in BOTH repo modes (synth -noabc / synth;abc -fast) AND verilator --lint-only clean — the Phase 6 memory-inference contract, proven on real generated output (not a hand template). (b) Identity: a MemRead leaf is opaque to CSE / never merged; the memory array never enters the NodeId graph (regression-clean factorization).`
  Acceptance: `cargo gates green; Yosys-inference proof reproducible in both modes on generated output; default-off still byte-identical.`
  Verification: `Scoping (recorded in Decisions): the cargo gate cannot shell out to Yosys/Verilator — the repo proves downstream-tool cleanliness only via the repo-owned tool_matrix gate (.2.3 scenario + .2.4 real run), never cargo (tests must pass without yosys/verilator). So (a) is split: the tool-level "$mem_v2 in both modes + verilator clean on generated output" was empirically established by .1's probe and the .2.1b real spot-check (binary seed 3 → 1 $mem_v2, verilator --lint-only exit 0, synth -noabc & synth;abc -fast both check -assert clean) and is AUTHORITATIVELY re-verified end-to-end at .2.4's real gate (r87 no-aspirational-claims); the cargo-portable formalization is the structural-contract equivalence — the generator emits EXACTLY the .1-validated Yosys-inferrable template, which IS the inference contract. New tests/pipeline.rs::inferrable_memory_matches_yosys_template_and_is_factorization_opaque proves, across 4 ConstructionStrategy × 4 FactorizationLevel (None/Cse/Commutative/EGraph) × 4 seeds (64 combos): validate_design clean; the SV is exactly the inferrable form (concrete `mem_0 [0:depth]` array, reset-less `always_ff @(posedge clk)`, `if (we) mem_0[..] <= wdata;`, `memrd_0 <= mem_0[..]`); (b) exactly one MemRead survives every factorization level and the memory leaf has zero expression-graph Gate nodes (the array/MemRead never enter the NodeId graph — CSE/factorization opaque, incl. EGraph). Default-off byte-identical reaffirmed by the .2.1b proof (unchanged). cargo fmt/clippy -D warnings/check --all-targets clean; new proof green; full cargo test (COMMIT.md gate — Verification Log). No book/ change.`
  Commit: `Phase 6: PHASE-6-ADVANCED-MOTIFS.2.2 memory inference structural-contract + factorization-opacity proof`

- ID: `PHASE-6-ADVANCED-MOTIFS.2.3`
  Status: `done`
  Goal: `tool_matrix scenario + metrics + gap (no ROADMAP advance). New phase6_inferrable_memory scenario (dedup/phase5/5b-anchor shape so shape-coverage sets are unperturbed); DesignMetrics.num_memory_modules; CoverageSummary.saw_inferrable_memory_design set + merged + a compute_coverage_gaps arm; bin-test scenario/design counts updated (observed, not guessed) + exception-list entry; non-vacuity test (scenario projects >=1 memory).`
  Acceptance: `cargo fmt/clippy(-D warnings)/check/test green incl. tool_matrix phase4 bin tests; NO ROADMAP phase label change yet.`
  Verification: `src/metrics.rs: DesignMetrics.num_memory_modules (count of modules with !memories.is_empty()), populated in compute_design. src/bin/tool_matrix.rs: new phase6_inferrable_memory_focus_config (cloned from phase5b_packed_aggregate_focus_config — depth-1 wrapper, library, EXACT dedup/phase5/5b anchor shape (4 leaves / 4 instances, all routing 0.0) so leaf/child/range/source shape-coverage sets are unperturbed; sole diff memory_prob=1.0 → rules-first library leaves are inferrable-memory blocks instantiated by the wrapper) + phase6_inferrable_memory scenario tuple + CoverageSummary.saw_inferrable_memory_design (set when config.memory_prob>0 && num_memory_modules>0) + merge_coverage + Phase4Hierarchy compute_coverage_gaps arm. Bin tests: scenario_count 216→219, total_modules 864→876 (observed deterministically from the run, not guessed), exception-list entry; tool_matrix phase4 bin tests 3/3. New phase6_inferrable_memory_scenario_is_non_vacuous proves every phase6_inferrable_memory scenario builds ≥1 memory module (coverage fact reachable) — 3/3 strategies. cargo fmt/clippy -D warnings/check --all-targets clean; full cargo test (COMMIT.md gate — Verification Log). ROADMAP unchanged (advance is .2.4). No book/ change.`
  Commit: `Phase 6: PHASE-6-ADVANCED-MOTIFS.2.3 phase6_inferrable_memory matrix scenario + metric + gap`

- ID: `PHASE-6-ADVANCED-MOTIFS.2.4`
  Status: `pending`
  Goal: `Run the real repo-owned gate (now including phase6_inferrable_memory) and VERIFY downstream-clean (coverage_gaps=[], Verilator + both Yosys all-pass, saw_inferrable_memory_design=true, P4/P5/P5b regressions clean) BEFORE any promotion. Then record the memory motif as delivered in ROADMAP Phase 6 (Phase 6 stays open until the .3 FSM motif also lands — memory delivery ADVANCES Phase 6, does not close it), reconcile book/src/ir.md (memory delivered) + book/src/knobs.md (memory_prob), sync README/CODEBASE_ANALYSIS/MEMORY. No PHASE-6 tree closure (only .2 container closes; .3 FSM remains).`
  Acceptance: `A banked gate report shows coverage_gaps=[] + all-pass Verilator/Yosys + saw_inferrable_memory_design=true; ROADMAP Phase 6 notes memory delivered (not "done" — .3 pending); .2 container -> done. No aspirational claims (verified artifact precedes the ROADMAP note).`
  Verification: `pending`
  Commit: `pending`

- ID: `PHASE-6-ADVANCED-MOTIFS.3`
  Status: `active`
  Goal: `Generated-state-encoding FSM motif. Split (mirroring the proven memory .2.1–.2.4) into .3.1 design / .3.2 IR+leaf+emitter+knob scaffold / .3.3 cargo-portable structural+opacity proof / .3.4 matrix scenario+metric+gap then real-gate verify → ROADMAP Phase 6 (FSM is the last motif → closes Phase 6 + the tree on a verified clean gate).`
  Children: `PHASE-6-ADVANCED-MOTIFS.3.1` (done), `.3.2` (container: `.3.2a`, `.3.2b`), `.3.3`, `.3.4`

- ID: `PHASE-6-ADVANCED-MOTIFS.3.1`
  Status: `done`
  Goal: `Design (DEVELOPMENT_NOTES.md): codebase-grounded generated-encoding FSM plan — IR grounding (no FSM concept; Flop the only state; operators-vs-blocks → FSM is a block); empirical downstream probe of the exact emitted SV template across binary/one-hot/gray; chosen architecture; >=3 rejected alternatives; .3 proof shape + split. Design-only; no code; mdbook clean.`
  Acceptance: `DEVELOPMENT_NOTES.md "Phase 6 generated-encoding FSM motif design" entry with the codebase grounding, empirical probe results (all 3 encodings clean in Verilator + both repo Yosys modes), chosen architecture (F), >=3 rejected alternatives, the .3 proof shape + split; no code change; mdbook build clean.`
  Verification: `DEVELOPMENT_NOTES.md "Phase 6 generated-encoding FSM motif design (2026-05-18, PHASE-6-ADVANCED-MOTIFS.3.1)" entry landed. Codebase grounding: IR has no FSM concept; Flop is the only stateful element; operators-vs-blocks doctrine → FSM is a block; a generated-encoding FSM = state-flop + comb next-state case + comb Moore output case + encoding-derived localparam constants. Empirical probe (the exact SV ANVIL would emit, 4-state, all 3 encodings): binary (2-bit state), one-hot (4-bit), gray (2-bit) — ALL THREE: verilator --lint-only -Wall exit 0; yosys synth -noabc; check -assert clean; yosys synth; abc -fast; check -assert clean (both repo Yosys modes). State width/constants differ by encoding ⇒ "encoding selectable" is structural, not cosmetic — the ROADMAP Phase 6 requirement is met by construction. Chosen architecture (F): additive Vec<Fsm> on Module (Default-empty) + opaque Node::FsmOut leaf (sibling to FlopQ/MemRead, never CSE'd, same compact.rs reachability obligation as .2.1a) + encoding-derived-constant emitter on the shared clk + opt-in Config::fsm_prob serde-default 0.0 (rules-first build_fsm_block, mutually-exclusive opt-in lane). 4 rejected/deferred: (A) primitives-only (encoding implicit/not selectable — defeats ROADMAP), (B) emitter-only string (not valid-by-construction), (C) generic enum/typedef datatype (massive invasive scalar-IR change; FSM is a block), (D) Mealy outputs (deferred, not a .3 blocker — Moore-only matches the probed-clean template). .3 split specified: .3.1 design / .3.2 scaffold (may sub-split iff a lower-level dependency surfaces, like .2.1) / .3.3 cargo-portable structural+CSE-opacity proof / .3.4 matrix scenario+metric+gap then real-gate verify → closes ROADMAP Phase 6 + the tree (memory already delivered at .2.4; multi-clock stays the optional separately-prioritised deferral). Design-only; no code; mdbook build clean; cargo fmt --all --check clean; full cargo test green at this slice's base (0b799b6; no src/tests touched since).`
  Commit: `Phase 6: PHASE-6-ADVANCED-MOTIFS.3.1 generated-encoding FSM motif design`

- ID: `PHASE-6-ADVANCED-MOTIFS.3.2`
  Status: `done`
  Goal: `FSM scaffold. Split (Splitting Rules + the proven .2.1 precedent: the opaque-stateful-leaf compaction-reachability is correctness-critical pipeline code, not mechanical FlopQ-mirroring — known concretely from the landed .2.1a, not speculative) into .3.2a (IR core + opaque FsmOut leaf + load-bearing compact.rs reachability + emitter + validator + unit proofs; no generator/knob → default-off trivially byte-identical) and .3.2b (Config::fsm_prob + rules-first build_fsm_block + default-off/forced-on focused proof).`
  Children: `PHASE-6-ADVANCED-MOTIFS.3.2a`, `.3.2b`

- ID: `PHASE-6-ADVANCED-MOTIFS.3.2a`
  Status: `done`
  Goal: `FSM IR core + opaque-stateful-leaf pipeline integration (mirrors the landed memory .2.1a). types.rs: FsmId, FsmEncoding{Binary,OneHot,Gray}, Fsm struct (id, num_states, encoding, sel:NodeId condition cone + sel_width, transitions table, per-state Moore output values, out_width), additive Default-empty Module.fsms, opaque Node::FsmOut{fsm,width}, DepAtom::FsmVirtual + DepSet::from_fsm_virtual, has_local_fsms() OR'd into the sequential-state predicates, Node::width arm; FsmOut threaded through ALL exhaustive Node matches (compiler-as-oracle, mirroring MemRead). Load-bearing compact.rs: StructuralNodeShape::FsmOut, LeafEndpoint::FsmOut, reachability arm keeping fsm.sel cone alive (sibling to MemRead keeping we/waddr/wdata/raddr), byte-identical rebuild arm, DepSet derivation, canonical-signature tag. Emitter renders the .3.1-probed-clean encoding-derived template (localparam state constants per encoding, state_q flop async-low reset to state 0 on shared clk, always_comb next-state case, always_comb Moore output case driving the FsmOut wire). Validator FSM step. 3 unit proofs (roundtrip+validate+emit; compaction-reachability keeps sel cone; structural-distinctness/CSE-opacity incl. two distinct FSMs). No generator/knob ⇒ default-off trivially byte-identical.`
  Acceptance: `cargo fmt/clippy(-D warnings)/check --all-targets/test green; 3 FSM unit proofs green; no Module without an Fsm changes (no fsms ⇒ byte-identical); no book/ change (book reconciliation is .3.4).`
  Verification: `types.rs: FsmId; FsmEncoding{Binary,OneHot,Gray} with state_width() (Binary/Gray=ceil(log2 N), OneHot=N) + state_const() (Binary=s, OneHot=1<<s, Gray=s^(s>>1)); Fsm struct (id, num_states, encoding, sel:NodeId, sel_width, transitions [num_states][1<<sel_width], outputs[num_states], out_width); additive Default-empty Module.fsms; opaque Node::FsmOut{fsm,width} + Node::width arm; DepAtom::FsmVirtual + DepSet::from_fsm_virtual; has_local_fsms() OR'd into both carries_sequential_state predicates. FsmOut threaded via compiler-as-oracle through every exhaustive Node match: compact.rs (StructuralNodeShape::FsmOut, LeafEndpoint::FsmOut + width(), intern, leaf-endpoint set, cone-eval offset, flop-remap no-op group, the LOAD-BEARING reachability arm marking fsm.sel alive, byte-identical rebuild arm, instance-table no-op group, node_deps), cone.rs ×5 (value-set/tiny-set/support/bounds/node_deps — opaque leaf like MemRead), hierarchy.rs source-width, module.rs output_root_has_empty_deps, param.rs is_width_generic (FsmOut⇒not width-generic), metrics.rs ×3 (kind-count no-op, node_deps, structural-hash tag 7). validate.rs: ValidateError BadFsm/UndefinedFsmSel/FsmSelWidthMismatch/DanglingFsmOut/FsmOutWidthMismatch + step 5c (slot-id, num_states>=1, sel_width/out_width>=1, sel node defined+width, transitions shape+range, outputs len+mask; every FsmOut resolves at out_width). emit/sv.rs: fsm_out_name/fsm_state_name/fsm_next_name/fsm_state_lit; per-FSM decls (state reg/next/out); the .3.1-probed-clean template (per-FSM FSM<id>_S<k> encoding-derived localparams, always_comb next-state case selected by sel, async-low-reset state always_ff to state 0 on shared clk/rst_n, always_comb Moore output case); Node::FsmOut→fsm_out_name. 3 unit proofs green: fsm_leaf_roundtrips_validate_and_emit (Binary 4-state ⇒ [1:0] reg, FSM0_S0=2'h0, async-reset block, next-state+Moore cases), fsmout_keeps_sel_cone_through_compaction (OneHot; dead gate removed, sel Xor cone survives, still validates+emits), fsmout_is_structurally_distinct_and_not_cse_merged (canonical sig != PrimaryInput twin; two distinct FSMs' outputs both survive compaction, count==2). cargo check --all-targets clean (Module Default covers the additive fsms field ⇒ no struct-literal breakage); cargo fmt --all --check clean; cargo clippy --all-targets -- -D warnings clean; full cargo test green (COMMIT.md gate). No generator/knob ⇒ Modules without an Fsm byte-identical (emitter gated on !m.fsms.is_empty(); predicates only OR when fsms non-empty; FsmOut arms only fire when a FsmOut node exists). No book/ change.`
  Commit: `Phase 6: PHASE-6-ADVANCED-MOTIFS.3.2a FSM IR core + opaque FsmOut leaf + compact.rs reachability`

- ID: `PHASE-6-ADVANCED-MOTIFS.3.2b`
  Status: `done`
  Goal: `Config::fsm_prob (f64, serde-default 0.0, probability-range validated; mirrors memory_prob/aggregate_prob) + rules-first build_fsm_block (a clk/rst_n + sel-input, fsm-out-output combinational-free FSM leaf; num_states + FsmEncoding rolled via g.rng; transitions/outputs filled by rule; opaque FsmOut drives the output; no gates/flops) + single opt-in roll in generate_leaf_module_with_interface_profile (mutually exclusive with the memory + param lanes; default-off never enters) + default-off-byte-identical / forced-on focused proof. Closes the .3.2 container.`
  Acceptance: `cargo fmt/clippy(-D warnings)/check --all-targets/test green; focused proof: default fsm_prob=0.0 byte-identical across ConstructionStrategy×seeds; forced fsm_prob=1.0 every single-module design is a 1-Fsm leaf that validates and emits the probed-clean per-encoding template (all 3 encodings reachable across seeds); no ROADMAP advance; no book/ change.`
  Verification: `src/config.rs: Config::fsm_prob (serde-default default_fsm_prob → 0.0; Default-impl line; probability-range validation tuple entry), mirroring memory_prob/aggregate_prob/width_parameterization_prob. src/gen/module.rs: rules-first build_fsm_block (clk(0)/rst_n(1) + sel(2,sel_width) inputs, q(3,out_width) output; num_states g.rng 2..=6; encoding g.rng Binary|OneHot|Gray; sel_width g.rng 1..=2; out_width from the configured width band; transitions[s][j]=(s+1+j)%num_states by rule; distinct masked per-state Moore outputs; opaque FsmOut drives q; no gates/flops — all rolls via g.rng, reproducible) + a single opt-in roll in generate_leaf_module_with_interface_profile placed AFTER the Phase 5 param lane and the Phase 6 memory lane (interface_profile None only; mutually exclusive; default-off fsm_prob==0.0 never enters → byte-identical). Focused proof tests/pipeline.rs::fsm_block_is_default_off_and_constructs_when_forced_on: (a) default-off byte-identical (no Fsm, no fsm_state_0/ fsm_0 in SV) across 4 ConstructionStrategy × 6 seeds; (b) forced-on (prob 1.0) every single-module design is a 1-Fsm leaf that validate_design-passes, exposes a FsmOut node, and emits the .3.1-probed-clean template (fsm_state_0 + FSM0_S0= state constants + async-reset always_ff @(posedge clk or negedge rst_n) with if(!rst_n) fsm_state_0<=FSM0_S0 + case(fsm_state_0)); AND all three encodings (Binary/OneHot/Gray) are reachable across the 24-design sweep. cargo fmt --all --check / clippy --all-targets -- -D warnings / check --all-targets clean; focused proof green; full cargo test green (COMMIT.md gate). No book/ change (book reconciliation is .3.4). Closes the .3.2 container.`
  Commit: `Phase 6: PHASE-6-ADVANCED-MOTIFS.3.2b fsm_prob knob + rules-first build_fsm_block`

- ID: `PHASE-6-ADVANCED-MOTIFS.3.3`
  Status: `done`
  Goal: `Cargo-portable proof (tests/pipeline.rs): across ConstructionStrategy × FactorizationLevel (incl. EGraph) × seeds — emitted SV is exactly the probed-clean per-encoding FSM template; exactly one FsmOut survives every factorization level (the FSM/array never enters the NodeId graph — CSE/EGraph-opaque); all 3 encodings reachable + structurally distinct; validate_design clean; default-off byte-identical reaffirmed.`
  Acceptance: `New pipeline.rs FSM proof green; cargo fmt/clippy/check/test green; no book change; no ROADMAP advance.`
  Verification: `tests/pipeline.rs::fsm_block_matches_probed_template_and_is_factorization_opaque — across 4 ConstructionStrategy × 4 FactorizationLevel (None/Cse/Commutative/EGraph) × 6 seeds (96 designs; the .1-style cargo-portable formalization, since cargo cannot shell yosys/verilator — tool-level proof is .3.4's real gate): validate_design clean; exactly 1 module / 1 Fsm. (b) Factorization/CSE-opacity on generated output — exactly one FsmOut survives EVERY factorization level (incl. EGraph) AND the FSM leaf has ZERO Gate nodes (the state machine never enters the NodeId expression graph). (a) Structural correctness keyed on the exact encoding: for every state s the SV contains `localparam logic [sw-1:0] FSM0_S<s> = <sw>'h<FsmEncoding::state_const(s)>;` with sw = FsmEncoding::state_width(num_states) — i.e. the emitted constants are EXACTLY the chosen encoding's formula (Binary=s / OneHot=1<<s / Gray=s^(s>>1)), which both proves structural correctness and makes the encodings structurally distinct wherever their parameters differ (robust where Binary/Gray coincide at N=2); plus the exact async-low-reset state always_ff (if(!rst_n) fsm_state_0<=FSM0_S0; else fsm_state_0<=fsm_next_0;) and the sel-selected next-state + Moore case. All three encodings reachable across the seeds-0..6 sweep (matches .3.2b's reachability sweep; encoding is fixed by (strategy,seed) — FactorizationLevel is a post-construction pass — so deterministic + reproducible). Proof-only: git diff = tests/pipeline.rs (+ tree/live-docs); no src/ change. cargo fmt --all --check / clippy --all-targets -- -D warnings / check --all-targets clean; full cargo test green (COMMIT.md gate). Default-off byte-identical is reaffirmed by .3.2b's focused proof (unchanged). No book/ change (book reconciliation is .3.4).`
  Commit: `Phase 6: PHASE-6-ADVANCED-MOTIFS.3.3 FSM structural-contract + factorization-opacity proof`

- ID: `PHASE-6-ADVANCED-MOTIFS.3.4`
  Status: `active`
  Goal: `Scenario + real-gate verify → close Phase 6. Split (Splitting Rules + the proven memory .2.3/.2.4 decomposition + r87 no-aspirational-claims: the matrix scenario+metric+gap is code that lands before any advance; the real-gate verification + ROADMAP/tree closure + book reconcile is a separate gated step) into .3.4a (phase6_fsm scenario + num_fsm_modules metric + saw_fsm_design fact/Phase4Hierarchy gap + non-vacuity test; no advance — unblocked) and .3.4b (run the real repo-owned gate, verify downstream-clean, then close ROADMAP Phase 6 + the tree + reconcile the book — gate-blocked).`
  Children: `PHASE-6-ADVANCED-MOTIFS.3.4a`, `.3.4b`

- ID: `PHASE-6-ADVANCED-MOTIFS.3.4a`
  Status: `done`
  Goal: `Mirror memory .2.3 for FSM. src/metrics.rs: DesignMetrics.num_fsm_modules (count !Module::fsms.is_empty()) + populate. src/bin/tool_matrix.rs: phase6_fsm_focus_config (clone of phase6_inferrable_memory_focus_config — depth-1 wrapper, library, 4 leaves/4 instances, all hierarchy-routing probs 0.0, EGraph; the only change fsm_prob=1.0 instead of memory_prob) + phase6_fsm scenario tuple; CoverageSummary.saw_fsm_design field + set (fsm_prob>0 && num_fsm_modules>0) + merge + Phase4Hierarchy compute_coverage_gaps arm; bump scenario/module bin counts (219→222 / 876→888) + the scenario-name exception list; new phase6_fsm_scenario_is_non_vacuous proving every such scenario builds ≥1 Fsm module. No ROADMAP advance.`
  Acceptance: `cargo fmt/clippy(-D warnings)/check --all-targets/test green; phase4 bin tests updated + green; phase6_fsm_scenario_is_non_vacuous green; ROADMAP unchanged (advance is .3.4b); no book/ change.`
  Verification: `src/metrics.rs: DesignMetrics.num_fsm_modules (count of design.modules with !fsms.is_empty()) + computed + added to the struct literal, mirroring num_memory_modules. src/bin/tool_matrix.rs: CoverageSummary.saw_fsm_design field; phase6_fsm scenario tuple registered after phase6_inferrable_memory (next_seed + 73); phase6_fsm_focus_config (exact clone of phase6_inferrable_memory_focus_config — depth-1 wrapper, library, 4 leaves/4 instances, all hierarchy-routing probs 0.0, EGraph, min/max_width 2/8 — only fsm_prob=1.0 instead of memory_prob, so the leaf/child/range/source shape-coverage sets are unperturbed); coverage set (scenario.config.fsm_prob>0 && design.metrics.num_fsm_modules>0); merge_into; Phase4Hierarchy compute_coverage_gaps arm ("…generated-encoding FSM (PHASE-6-ADVANCED-MOTIFS.3.4)"); bin counts 219→222 scenarios / 876→888 modules (verified: phase4_hierarchy bin test + the 222-scenario covers_wrapper test green — the +3/+12 delta matches phase6_inferrable_memory exactly); scenario-name exception list += phase6_fsm; new phase6_fsm_scenario_is_non_vacuous (every phase6_fsm scenario builds ≥1 Fsm module so saw_fsm_design is reachable — .3.4b's gate cannot carry a permanent gap). cargo check --all-targets clean (CoverageSummary uses ..Default::default() ⇒ no literal breakage); cargo test --bin tool_matrix 29/29; cargo fmt --all --check / clippy --all-targets -- -D warnings clean; full cargo test green (COMMIT.md gate). ROADMAP unchanged (advance is .3.4b on a verified gate). No book/ change.`
  Commit: `Phase 6: PHASE-6-ADVANCED-MOTIFS.3.4a phase6_fsm matrix scenario + num_fsm_modules metric + gap`

- ID: `PHASE-6-ADVANCED-MOTIFS.3.4b`
  Status: `pending`
  Goal: `Run the real repo-owned Phase4Hierarchy gate (now including phase6_fsm) and VERIFY downstream-clean (coverage_gaps=[], Verilator + both Yosys all-pass, saw_fsm_design=true, saw_inferrable_memory_design=true, P4/P5/P5b regressions clean, 222 scenarios / 888 modules) BEFORE any promotion. FSM is the LAST Phase 6 motif: record FSM delivered + (memory delivered at .2.4) close ROADMAP Phase 6 + the PHASE-6-ADVANCED-MOTIFS tree; reconcile book/src/ir.md + knobs.md (fsm_prob); sync README/CODEBASE_ANALYSIS/MEMORY. Multi-clock CDC stays the explicitly-optional separately-prioritised deferral (not a Phase 6 blocker). A single fresh gate run covers BOTH memory and fsm (supersedes the in-flight memory-only .2.4 gate as the Phase-6-closing artifact; .2.4 may still record memory delivered from its own run).`
  Acceptance: `Banked gate report: coverage_gaps=[] + all-pass Verilator/Yosys + saw_fsm_design=true + saw_inferrable_memory_design=true; ROADMAP Phase 6 → done (exit criteria met, with the multi-clock optional-deferral note); .3.4 + .3 container + PHASE-6-ADVANCED-MOTIFS tree → done. No aspirational claims (verified artifact precedes the ROADMAP promotion).`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `PHASE-6-ADVANCED-MOTIFS.2.4` | `pending` (gate-blocked) | `.2.3` landed the `phase6_inferrable_memory` matrix scenario + `num_memory_modules` metric + `saw_inferrable_memory_design` fact/gap (bin 216→219 / 864→876; scenario proven non-vacuous). `.2.4` runs the real repo-owned `Phase4Hierarchy` gate, verifies downstream-clean (`coverage_gaps=[]`, Verilator + both Yosys all-pass, `saw_inferrable_memory_design=true`, P4/P5/P5b regressions clean), then records memory **delivered** in ROADMAP Phase 6 (Phase 6 stays open for `.3` FSM — no tree closure) + reconciles the book — promotion strictly follows the verified artifact (r87 no-aspirational-claims). The real gate is **currently running** (`/tmp/anvil-tool-matrix-phase6-p1`); `.2.4` is verification/recording only — actioned when the gate completes. |
| 2 | `PHASE-6-ADVANCED-MOTIFS.3.4b` | `pending` (gate-blocked) | `.3.4a` **done** — `phase6_fsm` matrix scenario + `num_fsm_modules` metric + `saw_fsm_design` fact/`Phase4Hierarchy` gap + `phase6_fsm_scenario_is_non_vacuous` (bin 219→222 / 876→888, verified; tool_matrix 29/29; full `cargo test` green). `.3.4b` (the **last** Phase 6 leaf) runs a fresh real repo-owned `Phase4Hierarchy` gate (now including `phase6_fsm`), verifies downstream-clean (`coverage_gaps=[]`, Verilator + both Yosys all-pass, `saw_fsm_design=true` + `saw_inferrable_memory_design=true`, P4/P5/P5b regressions clean, 222/888), then records FSM delivered + (memory delivered at `.2.4`) **closes ROADMAP Phase 6 + the `PHASE-6-ADVANCED-MOTIFS` tree** + reconciles the book. Gate-blocked: needs a fresh run with the new binary (the in-flight gate is the old memory-only `.2.4` one); a single fresh gate covers both memory + fsm. |

## Decisions

- `2026-05-16`: Multi-clock CDC is held as an optional, possibly-deferred
  sub-objective (not yet a leaf) per its roadmap "optional, expensive"
  framing; it will be added as a leaf only if/when prioritised, with the
  single-clock invariant explicitly preserved until then.
- `2026-05-18` (`.2.2` scoping — recorded so it is not misread as a
  gap): the cargo gate **cannot** shell out to Yosys/Verilator — this
  repo proves downstream-tool cleanliness **only** via the repo-owned
  `tool_matrix` gate (`.2.3` scenario + `.2.4` real run), never in
  `cargo test` (tests must pass on machines without yosys/verilator,
  and that has been the convention since Phase 1). So `.2.2`(a)
  ("`$mem_v2` in both modes + Verilator clean on generated output")
  is satisfied by: (i) `.1`'s empirical probe + `.2.1b`'s real
  binary spot-check (interim evidence — `1 $mem_v2`, verilator exit
  0, both synth modes `check -assert` clean), and (ii) the
  authoritative end-to-end re-verification at `.2.4`'s real gate
  (r87 no-aspirational-claims). The cargo-portable formalization of
  the contract is the **structural-template equivalence**: the
  generator emits *exactly* the `.1`-validated inferrable template,
  which *is* the inference contract. This mirrors how Phase 5/5b
  proved downstream-cleanliness via the gate, not cargo.
- `2026-05-18` (**`.2.1` split — discovered lower-level dependency**):
  implementing `.2.1` surfaced that a new opaque **stateful** leaf
  (`Node::MemRead`) is *not* mechanical `FlopQ`-mirroring. Beyond the
  ~20 exhaustive `Node` match sites, `src/ir/compact.rs`
  reachability/dead-elimination is **load-bearing**: a reachable
  `MemRead` must transitively keep the memory's `we`/`waddr`/`wdata`/
  `raddr` source cones alive (exactly as a reachable `FlopQ` keeps its
  flop's D cone), else those cones are dead-stripped and emission
  breaks. That is correctness-critical pipeline code that must not be
  rushed into one slice with the knob + generator + proof. Per the
  Splitting Rules ("cannot be completed to signoff in one slice";
  "discovers a lower-level dependency that should be solved first"),
  `.2.1` was split into **`.2.1a`** (IR core + opaque-stateful-leaf
  pipeline integration incl. the compaction-reachability correctness +
  unit proofs; no generator, default-off trivially byte-identical) and
  **`.2.1b`** (`memory_prob` knob + rules-first `build_memory_leaf` +
  default-off/forced-on focused proof). In-flight IR-core edits were
  reverted to the clean `.2`-split base (`c96b433`) so `.2.1a` lands
  atomically from a clean tree. `.2.1` is now a container; `.2.2`/
  `.2.3`/`.2.4` unchanged; no renumbering. Frontier → `.2.1a`.
- `2026-05-18`: **`.2` split** per the Splitting Rules (new IR element
  + leaf + knob + emitter + validator + matrix gate cannot reach
  signoff in one slice and review independently) and the r87
  no-aspirational-claims precedent (gate scenario lands before any
  ROADMAP advance). Children mirror the proven Phase 5/5b
  `.2.1`–`.2.4`: `.2.1` IR+leaf+knob+emitter+validator scaffold
  (default-off byte-identical), `.2.2` Yosys-inference proof on
  generated output + CSE-opacity, `.2.3` matrix scenario+metric+gap
  (no advance), `.2.4` real-gate verify → ROADMAP **memory delivered**
  note (Phase 6 stays open for `.3` FSM; no tree closure). `.2` is now
  a container; `.3` (FSM) unchanged; no renumbering. Frontier →
  `.2.1`.
- `2026-05-18`: **`.3` split** (FSM motif) per the Splitting Rules +
  the r87 no-aspirational-claims precedent + the proven memory `.2`
  decomposition. `.3.1` design landed this slice: codebase grounding
  (FSM is a block, not an operator/datatype) + an empirical
  downstream probe of the exact emitted SV across **all three
  generated encodings** (binary / one-hot / gray) — every one clean
  in Verilator `--lint-only` **and** both repo Yosys modes
  (`synth -noabc`, `synth; abc -fast`, `check -assert`); state
  width/constants differ by encoding so "encoding selectable" is
  structural. Architecture **(F)** chosen (additive `Vec<Fsm>` +
  opaque `Node::FsmOut` sibling-to-`FlopQ`/`MemRead` + encoding-
  derived emitter + opt-in `fsm_prob`), 4 rejected/deferred. `.3`
  became a container mirroring memory `.2.1`–`.2.4`: `.3.1` design /
  `.3.2` scaffold (may sub-split iff a lower-level dependency
  surfaces, exactly as `.2.1` split on the compaction-reachability
  discovery — decided when reached) / `.3.3` cargo-portable
  structural + CSE-opacity proof / `.3.4` matrix scenario+metric+gap
  then real-gate verify. **FSM is the last Phase 6 motif** — a clean
  `.3.4` gate closes ROADMAP Phase 6 + the tree (memory already
  delivered at `.2.4`; multi-clock CDC stays the explicitly-optional
  separately-prioritised deferral, not a blocker). No renumbering.
  Frontier: `.2.4` (gate-blocked, verify-only) ‖ `.3.2` (unblocked —
  next continuous-PNT leaf while the `.2.4` gate runs).
- `2026-05-18`: **`.3.2` split** into `.3.2a` (IR core +
  opaque-stateful-leaf pipeline integration incl. the load-bearing
  `compact.rs` reachability + unit proofs; no generator/knob →
  default-off trivially byte-identical) and `.3.2b` (`fsm_prob` knob
  + rules-first `build_fsm_block` + focused proof). Unlike `.2.1`
  (which split *after* implementation surfaced the dependency), the
  `.3.1` design already identified that `Node::FsmOut` carries the
  **identical** correctness-critical compaction-reachability
  obligation as the landed `Node::MemRead` (a reachable `FsmOut`
  must transitively keep the FSM's `sel` condition cone alive, or it
  is dead-stripped and emission breaks). The lower-level dependency
  is therefore **known concretely from the landed `.2.1a`, not
  speculative** — splitting up front is "decided when reached" with
  the dependency in hand, satisfying the Splitting Rules ("cannot be
  completed to signoff in one slice"; "a lower-level dependency that
  should be solved first") and matching the proven memory
  decomposition. `.3.2` is now a container; `.3.3`/`.3.4` unchanged;
  no renumbering. Frontier → `.3.2a` (‖ `.2.4` gate-blocked).
- `2026-05-18`: **`.3.4` split** into `.3.4a` (phase6_fsm matrix
  scenario + `num_fsm_modules` metric + `saw_fsm_design` fact/gap +
  non-vacuity — code, no advance, unblocked) and `.3.4b` (real-gate
  verify → close ROADMAP Phase 6 + the tree + reconcile the book —
  gate-blocked). Splitting Rules + the proven memory `.2.3`/`.2.4`
  decomposition + r87 no-aspirational-claims: the matrix
  scenario+metric+gap is code that lands *before* any advance; the
  real-gate verification + ROADMAP/tree closure is a separate gated
  step that must follow a verified clean artifact. Note: the
  in-flight gate (`/tmp/anvil-tool-matrix-phase6-p1`, alive at
  ~110/219, old memory-only binary) is the `.2.4` artifact; `.3.4b`
  needs a **fresh** gate with the post-`.3.4a` binary — that single
  fresh run covers BOTH memory + fsm (+ P4/P5/P5b) and is the
  Phase-6-closing artifact. `.3.4` is now a container; no
  renumbering. Frontier → `.3.4a` (‖ `.2.4` gate-blocked).

## Open Questions

- Resolved by `.1` (empirical probe): the **single-port sync-write /
  sync-read** template and the **simple dual-port** template (1 write
  port + 1 independent synchronous read port) are both reliably
  inferred as `$mem_v2` by Yosys `memory_collect`, and synth clean
  (`check -assert`, exit 0) in **both** repo Yosys modes
  (`synth -noabc`, `synth; abc -fast`); Verilator `--lint-only` exit
  0. Recorded in `DEVELOPMENT_NOTES.md` "Phase 6 inferrable-memory
  motif design". (No open questions remain for `.1`.)
- Resolved by `.3.1` (empirical probe): a **generated-encoding Moore
  FSM** emitted as encoding-derived `localparam` state constants +
  an async-low-reset `state_q` flop + `always_comb` next-state
  `case` + `always_comb` Moore output `case` is downstream-clean for
  **all three encodings** (binary / one-hot / gray): Verilator
  `--lint-only -Wall` exit 0; Yosys `synth -noabc; check -assert`
  clean; `synth; abc -fast; check -assert` clean (both repo modes).
  State width/constants differ by encoding ⇒ encoding selectability
  is a structural fact. Recorded in `DEVELOPMENT_NOTES.md` "Phase 6
  generated-encoding FSM motif design". (No open questions remain
  for `.3.1`; `.3.2`+ are implementation.)

## Blockers

- None. Independent of Phase 4/5.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-05-18` | `PHASE-6-ADVANCED-MOTIFS.1` | `DEVELOPMENT_NOTES.md` Phase 6 memory design entry landed (codebase-grounded; empirical Yosys probe → single-port + simple-dual-port both `1 $mem_v2`, clean both modes; architecture (M) `Memory` block + opaque `MemRead` leaf; 3 rejected alternatives; proof shape). Doc-only, no code; `mdbook build book` clean; `cargo fmt --check` clean; `cargo test` unchanged-green (no `src/`/`tests/` touched since Phase 5b `.2.3`). | Done. |
| `2026-05-18` | `PHASE-6-ADVANCED-MOTIFS.2.1a` | IR core (`MemId`/`MemKind`/`Memory`/additive `Module.memories`/opaque `Node::MemRead`/`DepAtom::MemVirtual`/`from_mem_virtual`/`has_local_memories`) + `MemRead` threaded through all ~21 exhaustive `Node` matches (compiler-as-oracle, mirrors `FlopQ`); **load-bearing `compact.rs` reachability** (a live `MemRead` keeps `mem.{we,waddr,wdata,raddr}` cones alive; `StructuralNodeShape`/`LeafEndpoint::MemRead`); emitter inferrable template (`mem_<id>` array + `memrd_<id>` + reset-less `always_ff @(posedge clk)`); validator memory step-5b. 3 unit proofs (roundtrip+validate+emit; **compaction-reachability**; structural-distinctness/CSE-opacity) — `ir::compact::tests` mem 3/3. `cargo fmt`/`clippy -D warnings`/`check --all-targets` clean; full `cargo test` (COMMIT.md gate). No generator/knob ⇒ default-off trivially byte-identical. No `book/` change. | Done. |
| `2026-05-18` | `PHASE-6-ADVANCED-MOTIFS.2.1b` | `Config::memory_prob` (serde-default 0.0 + validation, mirrors `aggregate_prob`); rules-first `build_memory_leaf` (`clk`/`rst_n`+`we`/`waddr`/`wdata`[+`raddr`] inputs, `rdata` out, one `Memory` kind rolled via `g.rng`, opaque `MemRead` drive; no gates/flops) + single opt-in roll after the Phase 5 param lane (mutually exclusive; default-off never enters). Focused proof `inferrable_memory_is_default_off_and_constructs_when_forced_on` (default-off byte-identical 4 strategies × 6 seeds; forced-on every single-module design is a 1-`Memory` leaf, validates, emits the inferrable template). Real spot-check (binary seed 3, prob 1.0): `verilator --lint-only` exit 0; yosys `memory_collect` → `1 $mem_v2`; `synth -noabc` & `synth;abc -fast` both `check -assert` clean. `cargo fmt`/`clippy -D warnings`/`check --all-targets` clean; full `cargo test` (COMMIT.md gate). No `book/` change. | Done (closes the `.2.1` container). |
| `2026-05-18` | `PHASE-6-ADVANCED-MOTIFS.2.2` | `tests/pipeline.rs::inferrable_memory_matches_yosys_template_and_is_factorization_opaque` — across 4 `ConstructionStrategy` × 4 `FactorizationLevel` (None/Cse/Commutative/EGraph) × 4 seeds (64 combos): `validate_design` clean; SV is exactly the `.1`-validated Yosys-inferrable form (concrete `mem_0 [0:depth]`, reset-less `always_ff @(posedge clk)`, `if (we) mem_0[..] <= wdata;`, `memrd_0 <= mem_0[..]`); exactly one `MemRead` survives every factorization level + zero expression-graph `Gate` nodes (array/`MemRead` never enter the NodeId graph — CSE/factorization-opaque incl. EGraph). Tool-level `$mem_v2`/Verilator proof is `.2.1b`'s spot-check (interim) + `.2.4`'s real gate (authoritative; cargo can't shell yosys — see Decisions). Default-off byte-identical reaffirmed by the `.2.1b` proof. `cargo fmt`/`clippy -D warnings`/`check --all-targets` clean; full `cargo test` (COMMIT.md gate). No `book/` change. | Done. |
| `2026-05-18` | `PHASE-6-ADVANCED-MOTIFS.3.1` | `DEVELOPMENT_NOTES.md` "Phase 6 generated-encoding FSM motif design" entry landed. Codebase grounding (no FSM IR; Flop the only state; operators-vs-blocks → FSM is a block). Empirical downstream probe of the exact emitted SV (4-state Moore, all 3 encodings): binary (2-bit), one-hot (4-bit), gray (2-bit) — every one: `verilator --lint-only -Wall` exit 0; `yosys synth -noabc; check -assert` clean; `yosys synth; abc -fast; check -assert` clean (both repo modes); state width/constants differ by encoding ⇒ selectability structural. Architecture (F) + 4 rejected/deferred; `.3` split (`.3.1`–`.3.4`, mirrors memory `.2.1`–`.2.4`; FSM = last Phase 6 motif → `.3.4` closes Phase 6 + the tree). Doc-only, no code; `mdbook build book` clean; `cargo fmt --all --check` clean; full `cargo test` green at base `0b799b6` (no `src/`/`tests/` touched since). | Done. |
| `2026-05-18` | `PHASE-6-ADVANCED-MOTIFS.2.3` | `DesignMetrics.num_memory_modules` + populate; `phase6_inferrable_memory_focus_config` (clone of the phase5b/dedup anchor — depth-1 wrapper, library, `memory_prob=1.0`, 4 leaves/4 instances → shape-coverage sets unperturbed) + `phase6_inferrable_memory` scenario tuple; `CoverageSummary.saw_inferrable_memory_design` set/merge + Phase4Hierarchy `compute_coverage_gaps` arm; bin counts 216→219 / 864→876 (observed) + exception-list entry; tool_matrix phase4 bin tests 3/3; new `phase6_inferrable_memory_scenario_is_non_vacuous` proves the scenario builds ≥1 memory module per strategy (coverage fact reachable). `cargo fmt`/`clippy -D warnings`/`check --all-targets` clean; full `cargo test` (COMMIT.md gate). ROADMAP unchanged (advance is `.2.4`). No `book/` change. | Done. |
| `2026-05-18` | `PHASE-6-ADVANCED-MOTIFS.3.2a` | FSM IR core (`FsmId`; `FsmEncoding{Binary,OneHot,Gray}` + `state_width`/`state_const`; `Fsm` struct; additive `Default`-empty `Module.fsms`; opaque `Node::FsmOut`; `DepAtom::FsmVirtual`/`from_fsm_virtual`; `has_local_fsms` OR'd into both `carries_sequential_state` predicates). `FsmOut` threaded by compiler-as-oracle through every exhaustive `Node` match (compact.rs incl. the **load-bearing reachability** marking `fsm.sel` alive + `StructuralNodeShape`/`LeafEndpoint`/cone-eval/rebuild/instance-table/`node_deps`; cone.rs ×5; hierarchy.rs; module.rs; param.rs; metrics.rs ×3 incl. structural-hash tag 7). `validate.rs` step 5c + 5 `ValidateError` variants. Emitter: per-FSM decls + the `.3.1`-probed-clean template (encoding-derived `FSM<id>_S<k>` localparams, `always_comb` next-state `case` on `sel`, async-low-reset state `always_ff` on shared `clk`/`rst_n`, `always_comb` Moore output `case`). 3 unit proofs green (roundtrip+validate+emit Binary 4-state; sel-cone-survives-compaction OneHot; structural-distinctness/CSE-opacity incl. two distinct FSMs). `cargo check --all-targets` (Module `Default` covers additive `fsms` → no struct-literal breakage) / `cargo fmt --all --check` / `cargo clippy --all-targets -- -D warnings` clean; full `cargo test` (COMMIT.md gate). No generator/knob ⇒ Modules without an `Fsm` byte-identical. No `book/` change. | Done. |
| `2026-05-18` | `PHASE-6-ADVANCED-MOTIFS.3.2b` | `src/config.rs`: `Config::fsm_prob` (serde-default `default_fsm_prob`→0.0; Default-impl line; probability-range validation tuple), mirroring `memory_prob`/`aggregate_prob`. `src/gen/module.rs`: rules-first `build_fsm_block` (clk(0)/rst_n(1)+sel(2,sel_width) inputs, q(3,out_width) output; `num_states` g.rng 2..=6; `encoding` g.rng Binary\|OneHot\|Gray; `sel_width` g.rng 1..=2; `out_width` from the configured width band; `transitions[s][j]=(s+1+j)%num_states` by rule; distinct masked Moore outputs; opaque `FsmOut` drives q; no gates/flops; all rolls via `g.rng`) + single opt-in roll in `generate_leaf_module_with_interface_profile` AFTER the Phase 5 param + Phase 6 memory lanes (interface_profile None only; mutually exclusive; default-off `fsm_prob==0.0` never enters → byte-identical). Focused proof `tests/pipeline.rs::fsm_block_is_default_off_and_constructs_when_forced_on`: (a) default-off byte-identical (no `Fsm`, no `fsm_state_0`/` fsm_0;`) across 4 `ConstructionStrategy` × 6 seeds; (b) forced-on (1.0) every single-module design is a 1-`Fsm` leaf that `validate_design`-passes, exposes a `FsmOut`, emits the `.3.1`-probed-clean template (`fsm_state_0`+`FSM0_S0=` constants + async-reset `always_ff @(posedge clk or negedge rst_n)` with `if (!rst_n) fsm_state_0 <= FSM0_S0` + `case (fsm_state_0)`); AND all 3 encodings reachable across the 24-design sweep. `cargo fmt --all --check`/`clippy --all-targets -- -D warnings`/`check --all-targets` clean; focused proof green; full `cargo test` (COMMIT.md gate). No ROADMAP advance (that is `.3.4`). No `book/` change. Closes the `.3.2` container. | Done. |
| `2026-05-18` | `PHASE-6-ADVANCED-MOTIFS.3.3` | `tests/pipeline.rs::fsm_block_matches_probed_template_and_is_factorization_opaque` — 4 `ConstructionStrategy` × 4 `FactorizationLevel` (None/Cse/Commutative/EGraph) × 6 seeds (96 designs; cargo-portable formalization, tool-level proof is `.3.4`'s real gate). validate_design clean; 1 module / 1 `Fsm`. (b) **Opacity**: exactly one `FsmOut` survives every factorization level (incl. EGraph) + the FSM leaf has ZERO `Gate` nodes (the state machine never enters the NodeId graph). (a) **Structural correctness keyed on the exact encoding**: every state `s` ⇒ SV contains `localparam logic [sw-1:0] FSM0_S<s> = <sw>'h<FsmEncoding::state_const(s)>;` with `sw = FsmEncoding::state_width(num_states)` (Binary=`s`/OneHot=`1<<s`/Gray=`s^(s>>1)`) — proves correctness + structural distinctness where params differ (robust where Binary/Gray coincide at N=2); + exact async-low-reset state `always_ff` (`if(!rst_n) fsm_state_0<=FSM0_S0; else fsm_state_0<=fsm_next_0;`) + sel-selected next-state/Moore cases. All 3 encodings reachable across seeds 0..6 (matches `.3.2b`; encoding fixed by `(strategy,seed)`, deterministic/reproducible). Proof-only — `git diff` = `tests/pipeline.rs` (+ tree/live-docs), no `src/`. `cargo fmt --all --check`/`clippy --all-targets -- -D warnings`/`check --all-targets` clean; full `cargo test` (COMMIT.md gate). Default-off byte-identical reaffirmed by `.3.2b` (unchanged). No `book/` change. | Done. |
| `2026-05-18` | `PHASE-6-ADVANCED-MOTIFS.3.4a` | `src/metrics.rs`: `DesignMetrics.num_fsm_modules` (count `!Module::fsms.is_empty()`) + computed + struct-literal, mirroring `num_memory_modules`. `src/bin/tool_matrix.rs`: `CoverageSummary.saw_fsm_design`; `phase6_fsm` scenario tuple (after `phase6_inferrable_memory`, `next_seed+73`); `phase6_fsm_focus_config` (exact clone of the memory anchor — depth-1 wrapper / library / 4 leaves / 4 instances / EGraph / routing-probs 0.0, only `fsm_prob=1.0` — shape-coverage sets unperturbed); coverage set (`fsm_prob>0 && num_fsm_modules>0`); `merge_into`; `Phase4Hierarchy` gap arm; bin counts **219→222 scenarios / 876→888 modules** (verified — phase4 bin test + 222-scenario covers_wrapper test green; +3/+12 matches `phase6_inferrable_memory`); scenario-name exception list `+= phase6_fsm`; new `phase6_fsm_scenario_is_non_vacuous` (every `phase6_fsm` scenario builds ≥1 `Fsm` module ⇒ `saw_fsm_design` reachable). `cargo check --all-targets` clean (`CoverageSummary` uses `..Default::default()`); `cargo test --bin tool_matrix` 29/29; `cargo fmt --all --check`/`clippy --all-targets -- -D warnings` clean; full `cargo test` (COMMIT.md gate). ROADMAP unchanged (advance is `.3.4b`). No `book/` change. | Done. |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `PHASE-6-ADVANCED-MOTIFS.1` | `Docs: PHASE-6-ADVANCED-MOTIFS.1 inferrable-memory motif design` | Design-only; architecture (M) `Memory` block + `MemRead` leaf; empirical Yosys probe; 3 rejected alternatives. No code. |
| `PHASE-6-ADVANCED-MOTIFS.2.1a` | `Phase 6: PHASE-6-ADVANCED-MOTIFS.2.1a memory IR core + opaque-stateful-leaf pipeline integration` | IR core + `MemRead` through ~21 matches + the load-bearing `compact.rs` reachability + emitter template + validator + 3 unit proofs. No generator/knob (default-off trivially byte-identical). |
| `PHASE-6-ADVANCED-MOTIFS.2.1b` | `Phase 6: PHASE-6-ADVANCED-MOTIFS.2.1b memory_prob knob + rules-first build_memory_leaf` | Knob + rules-first constructor + opt-in roll (mutually exclusive with the param lane) + focused proof; generated memory spot-checked `1 $mem_v2` + verilator/both-yosys clean. Closes the `.2.1` container. |
| `PHASE-6-ADVANCED-MOTIFS.2.2` | `Phase 6: PHASE-6-ADVANCED-MOTIFS.2.2 memory inference structural-contract + factorization-opacity proof` | Cargo-portable structural-template-equivalence + factorization/CSE-opacity (64 combos incl. EGraph); tool-level `$mem_v2` proof = `.2.1b` spot-check + `.2.4` gate. No code change (proof only). |
| `PHASE-6-ADVANCED-MOTIFS.2.3` | `Phase 6: PHASE-6-ADVANCED-MOTIFS.2.3 phase6_inferrable_memory matrix scenario + metric + gap` | `num_memory_modules` metric + `phase6_inferrable_memory` scenario + `saw_inferrable_memory_design` fact/gap; bin 216→219 / 864→876; non-vacuity test. No ROADMAP advance (that is `.2.4` on verified evidence). |
| `PHASE-6-ADVANCED-MOTIFS.3.1` | `Phase 6: PHASE-6-ADVANCED-MOTIFS.3.1 generated-encoding FSM motif design` | Design-only; architecture (F) `Fsm` block + opaque `FsmOut` leaf; empirical probe (all 3 encodings clean Verilator + both Yosys modes); 4 rejected/deferred; `.3` split into `.3.1`–`.3.4`. No code. |
| `PHASE-6-ADVANCED-MOTIFS.3.2` (split) | `Docs: split PHASE-6-ADVANCED-MOTIFS.3.2 into .3.2a (IR core) + .3.2b (knob)` | Tree-planning, no code. Dependency known concretely from `.2.1a`. |
| `PHASE-6-ADVANCED-MOTIFS.3.2a` | `Phase 6: PHASE-6-ADVANCED-MOTIFS.3.2a FSM IR core + opaque FsmOut leaf + compact.rs reachability` | IR core + `FsmOut` through every exhaustive `Node` match + load-bearing `compact.rs` reachability + emitter (.3.1 template) + validator 5c + 3 unit proofs. No generator/knob (default-off byte-identical). |
| `PHASE-6-ADVANCED-MOTIFS.3.2b` | `Phase 6: PHASE-6-ADVANCED-MOTIFS.3.2b fsm_prob knob + rules-first build_fsm_block` | `Config::fsm_prob` + rules-first `build_fsm_block` in the mutually-exclusive opt-in lane + focused proof (default-off byte-identical; forced-on 1-`Fsm` leaf, all 3 encodings reachable). Closes the `.3.2` container. |
| `PHASE-6-ADVANCED-MOTIFS.3.3` | `Phase 6: PHASE-6-ADVANCED-MOTIFS.3.3 FSM structural-contract + factorization-opacity proof` | Cargo-portable proof (96 designs): exact per-encoding `state_const`/`state_width` template + 1 `FsmOut`/0 `Gate` across every FactorizationLevel incl. EGraph + all 3 encodings reachable. Proof-only (no `src/`). |
| `PHASE-6-ADVANCED-MOTIFS.3.4a` | `Phase 6: PHASE-6-ADVANCED-MOTIFS.3.4a phase6_fsm matrix scenario + num_fsm_modules metric + gap` | `num_fsm_modules` metric + `phase6_fsm` scenario + `saw_fsm_design` fact/gap; bin 219→222 / 876→888; non-vacuity test. No ROADMAP advance (that is `.3.4b` on verified evidence). |

## Changelog

- `2026-05-16`: Created task tree as part of the directive to task-tree
  every remaining roadmap phase.
- `2026-05-18`: `.1` design landed (design-only, no code) —
  `DEVELOPMENT_NOTES.md` "Phase 6 inferrable-memory motif design".
  Empirical Yosys probe resolves the Open Question (single-port +
  simple-dual-port → `1 $mem_v2`, clean both repo modes + Verilator).
  Architecture **(M)**: first-class `Memory` block (additive
  `Vec<Memory>` on `Module`, Default-empty) + opaque `Node::MemRead`
  leaf (sibling to `FlopQ`, never CSE'd) + emitter renders the
  validated inferrable template on the shared `clk` + opt-in
  `Config::memory_prob` serde-default 0.0; rejected (A) flop-array+mux
  (not `$mem`-inferred), (B) emitter-only string template (not
  valid-by-construction), (C) generic unpacked-array datatype
  (massive invasive change; memory is a block not a datatype).
  `mdbook` clean. Frontier → `.2` (implement per (M); expected to
  split `.2.x` per the Phase 5/5b precedent + r87
  no-aspirational-claims).
- `2026-05-18`: `.2` split per the Splitting Rules + r87
  no-aspirational-claims into `.2.1` (IR+leaf+knob+emitter+validator
  scaffold, default-off byte-identical), `.2.2`
  (Yosys-inference proof on generated output + `MemRead` CSE-opacity),
  `.2.3` (matrix scenario+metric+gap, no advance), `.2.4` (real-gate
  verify → ROADMAP memory-delivered note; Phase 6 stays open for `.3`
  FSM — no tree closure). `.2` became a container; `.3` unchanged; no
  renumbering. Frontier → `.2.1`.
- `2026-05-18`: **`.2.1` split** on a discovered lower-level
  dependency (compaction-reachability for an opaque *stateful* leaf is
  correctness-critical, not mechanical `FlopQ`-mirroring): `.2.1a` (IR
  core + opaque-stateful-leaf pipeline integration incl. the
  reachability correctness + unit proofs; no generator/knob,
  default-off trivially byte-identical) and `.2.1b` (`memory_prob`
  knob + rules-first `build_memory_leaf` + default-off/forced-on
  focused proof). In-flight IR-core edits reverted to the clean
  `.2`-split base so `.2.1a` lands atomically. `.2.1` became a
  container; no renumbering. Frontier → `.2.1a`.
- `2026-05-18`: **`.2.1a` landed.** IR core: `MemId`,
  `MemKind{SinglePort,SimpleDualPort}`, `Memory`, additive
  `Default`-empty `Module.memories`, opaque leaf `Node::MemRead`,
  `DepAtom::MemVirtual` + `DepSet::from_mem_virtual`,
  `has_local_memories()` OR'd into the sequential-state predicates
  (clk exposed for memory-only modules; `has_local_flops` untouched).
  `MemRead` threaded through all ~21 exhaustive `Node` matches via the
  compiler-as-completeness-oracle, mirroring `FlopQ`. **Load-bearing
  correctness**: `compact.rs` reachability gains a `MemRead` arm that
  keeps `mem.{we,waddr,wdata,raddr}` cones alive (memories never
  dead-eliminated in 6.2.1 → stable `MemId`, no remap);
  `StructuralNodeShape`/`LeafEndpoint::MemRead`; canonical-signature
  tag 6. Emitter renders the `.1`-validated inferrable template
  (`mem_<id>` array + `memrd_<id>` + reset-less
  `always_ff @(posedge clk)`); validator step-5b checks memory widths
  / SinglePort-shared-addr / `MemRead` resolution. 3 unit proofs
  (roundtrip+validate+emit; compaction-reachability; structural
  distinctness + two-memory CSE-opacity) green; full `cargo` gate
  clean. No generator/knob ⇒ default-off trivially byte-identical; no
  `book/` change. Frontier → `.2.1b` (knob + rules-first
  `build_memory_leaf` + focused proof).
- `2026-05-18`: **`.2.1b` landed — closes the `.2.1` container.**
  `Config::memory_prob` (serde-default 0.0 + validation, mirrors
  `aggregate_prob`); rules-first `build_memory_leaf` (`clk`/`rst_n` +
  `we`/`waddr`/`wdata`[+`raddr`] inputs, `rdata` out, one `Memory`
  with kind rolled via `g.rng`, opaque `MemRead` drive — no
  gates/flops); a single opt-in roll in
  `generate_leaf_module_with_interface_profile` placed after the
  Phase 5 param lane (mutually exclusive; `interface_profile` None
  only; default-off never enters). Focused proof
  `inferrable_memory_is_default_off_and_constructs_when_forced_on`
  (default-off byte-identical 4 strategies × 6 seeds; forced-on every
  single-module design is a 1-`Memory` leaf that validates + emits the
  inferrable template). Real spot-check (binary seed 3, prob 1.0):
  `verilator --lint-only` exit 0, yosys `memory_collect` →
  `1 $mem_v2`, `synth -noabc` & `synth;abc -fast` both `check -assert`
  clean — the Phase 6 inference contract holds on real generated
  output (formalised in `.2.2`). Full `cargo` gate clean; no `book/`
  change. Frontier → `.2.2` (formal Yosys-inference proof + `MemRead`
  CSE-opacity; no ROADMAP advance — that is `.2.4`).
- `2026-05-18`: **`.2.2` landed (proof only — no feature code
  change).** `tests/pipeline.rs::inferrable_memory_matches_yosys_template_and_is_factorization_opaque`:
  across 4 `ConstructionStrategy` × 4 `FactorizationLevel`
  (None/Cse/Commutative/EGraph) × 4 seeds, every forced-on memory
  design `validate_design`-passes, emits *exactly* the `.1`-validated
  Yosys-`$mem_v2` template (concrete-depth array, reset-less
  `always_ff @(posedge clk)`, synchronous write + registered read),
  keeps exactly one `MemRead` through every factorization level, and
  has zero expression-graph `Gate` nodes (the array/`MemRead` never
  enter the NodeId graph — CSE/factorization-opaque incl. EGraph).
  Scoping recorded in Decisions: cargo can't shell yosys/verilator;
  the tool-level `$mem_v2`/Verilator proof is `.2.1b`'s real
  spot-check (interim) + `.2.4`'s real repo-owned gate
  (authoritative, r87). Default-off byte-identical reaffirmed by the
  `.2.1b` proof. Frontier → `.2.3` (matrix scenario + metric + gap,
  no ROADMAP advance).
- `2026-05-18`: **`.2.3` landed (matrix scenario + metric + gap; no
  ROADMAP advance).** `src/metrics.rs`
  `DesignMetrics.num_memory_modules` + populate. `src/bin/tool_matrix.rs`:
  `phase6_inferrable_memory_focus_config` (clone of the
  phase5b/dedup anchor — depth-1 wrapper, library, `memory_prob = 1.0`,
  4 leaves/4 instances so leaf/child/range/source shape sets are
  unperturbed; the rules-first library leaves are memory blocks
  instantiated by the wrapper) + `phase6_inferrable_memory` scenario
  tuple; `CoverageSummary.saw_inferrable_memory_design` set/merge +
  `Phase4Hierarchy` `compute_coverage_gaps` arm; bin counts
  216 → 219 / 864 → 876 (observed deterministically) + exception-list
  entry; tool_matrix phase4 bin tests 3/3. New
  `phase6_inferrable_memory_scenario_is_non_vacuous` proves every such
  scenario builds ≥1 memory module so `saw_inferrable_memory_design`
  is reachable — `.2.4`'s gate cannot carry a permanent coverage gap.
  ROADMAP unchanged. Frontier → `.2.4` (real-gate verify → ROADMAP
  memory-delivered note + book reconciliation; Phase 6 stays open for
  `.3` FSM — no tree closure).
- `2026-05-18`: **`.3` split + `.3.1` design landed** (design-only, no
  code) — continuous-PNT progress on the active Phase 6 tree while the
  `.2.4` memory gate runs. `DEVELOPMENT_NOTES.md` "Phase 6
  generated-encoding FSM motif design": codebase grounding (no FSM IR;
  Flop is the only state; operators-vs-blocks → FSM is a **block**);
  empirical downstream probe of the **exact emitted SV** for a 4-state
  Moore FSM in **all three encodings** — binary (2-bit `state_q`),
  one-hot (4-bit), gray (2-bit): each `verilator --lint-only -Wall`
  exit 0 + `yosys synth -noabc; check -assert` clean + `yosys synth;
  abc -fast; check -assert` clean (both repo Yosys modes); width and
  constants differ by encoding ⇒ "encoding selectable" is structural,
  satisfying the ROADMAP Phase 6 requirement by construction.
  Architecture **(F)**: additive `Vec<Fsm>` on `Module`
  (Default-empty) + opaque `Node::FsmOut` leaf (sibling to
  `FlopQ`/`MemRead`, never CSE'd, same `compact.rs` reachability
  obligation as `.2.1a`) + encoding-derived-constant emitter on the
  shared `clk` + opt-in `Config::fsm_prob` serde-default 0.0
  (rules-first `build_fsm_block`, mutually-exclusive opt-in lane);
  rejected/deferred (A) primitives-only (encoding implicit/not
  selectable), (B) emitter-only string (not valid-by-construction),
  (C) generic enum/typedef datatype (massive invasive scalar-IR
  change), (D) Mealy (deferred, not a `.3` blocker). `.3` became a
  container `.3.1`–`.3.4` mirroring memory `.2.1`–`.2.4`. **FSM is the
  last Phase 6 motif** → a verified-clean `.3.4` gate closes ROADMAP
  Phase 6 + the `PHASE-6-ADVANCED-MOTIFS` tree (memory delivered at
  `.2.4`; multi-clock CDC stays the explicitly-optional
  separately-prioritised deferral, not a blocker). `mdbook` clean.
  Frontier: `.2.4` (gate-blocked, verify-only) ‖ `.3.2` (unblocked —
  next continuous-PNT leaf while the `.2.4` gate runs).
- `2026-05-18`: **`.3.2` split** into `.3.2a` (IR core +
  opaque-`FsmOut`-leaf pipeline integration incl. the load-bearing
  `compact.rs` reachability that keeps `fsm.sel` alive + 3 unit
  proofs; no generator/knob → default-off trivially byte-identical)
  and `.3.2b` (`Config::fsm_prob` + rules-first `build_fsm_block` +
  focused proof). The lower-level dependency is **known concretely
  from the landed memory `.2.1a`** (`FsmOut` has the identical
  opaque-stateful-leaf reachability obligation as `MemRead`), so the
  split is decided up front with the dependency in hand — Splitting
  Rules + the proven `.2.1` precedent. `.3.2` is now a container;
  `.3.3`/`.3.4` unchanged; no renumbering. Frontier → `.3.2a`
  (‖ `.2.4` gate-blocked).
- `2026-05-18`: **`.3.2a` landed** (continuous-PNT while the `.2.4`
  memory gate runs). FSM IR core mirroring the proven memory
  `.2.1a`: `FsmId`; `FsmEncoding{Binary,OneHot,Gray}` with
  `state_width()`/`state_const()` (Binary=`s`, OneHot=`1<<s`,
  Gray=`s^(s>>1)`); `Fsm` struct (`num_states`, `encoding`,
  `sel:NodeId`+`sel_width`, `transitions[N][1<<sel_width]`,
  `outputs[N]`, `out_width`); additive `Default`-empty `Module.fsms`;
  opaque `Node::FsmOut`; `DepAtom::FsmVirtual`/`from_fsm_virtual`;
  `has_local_fsms()` OR'd into both `carries_sequential_state`
  predicates. `FsmOut` threaded by the compiler-as-oracle through
  **every** exhaustive `Node` match (the load-bearing `compact.rs`
  reachability marks `fsm.sel` alive — sibling to `MemRead` keeping
  its address/data cones; + cone.rs ×5, hierarchy.rs, module.rs,
  param.rs, metrics.rs ×3 incl. structural-hash tag 7).
  `validate.rs` step 5c + 5 error variants. Emitter renders the
  `.3.1`-probed-clean template (per-FSM `FSM<id>_S<k>`
  encoding-derived `localparam`s, `always_comb` next-state `case`
  on `sel`, async-low-reset state `always_ff` on the shared
  `clk`/`rst_n`, `always_comb` Moore output `case`). 3 unit proofs
  green (roundtrip+validate+emit; sel-cone survives compaction;
  structural-distinctness/CSE-opacity incl. two distinct FSMs). Full
  `cargo` gate green; no generator/knob ⇒ Modules without an `Fsm`
  byte-identical. Frontier → `.3.2b` (knob + rules-first
  `build_fsm_block` + focused proof, closes the `.3.2` container)
  (‖ `.2.4` gate-blocked).
- `2026-05-18`: **`.3.2b` landed → `.3.2` container done**
  (continuous-PNT while the `.2.4` memory gate runs). `Config::fsm_prob`
  (serde-default 0.0, probability-range validated; mirrors
  `memory_prob`/`aggregate_prob`). Rules-first `build_fsm_block`:
  clk/rst_n + a single `sel` input, `num_states`/`encoding`/
  `sel_width`/`out_width` rolled via `g.rng` (reproducible),
  transitions + distinct masked Moore outputs filled by rule, opaque
  `FsmOut` drives the sole output; no gates/flops. Single opt-in roll
  in `generate_leaf_module_with_interface_profile` placed after the
  Phase 5 param and Phase 6 memory lanes — **mutually exclusive**;
  default-off (`fsm_prob == 0.0`) never enters, so emission is
  byte-identical. Focused proof
  `fsm_block_is_default_off_and_constructs_when_forced_on`: default-off
  byte-identical (4 strategies × 6 seeds, no `Fsm`/no FSM SV);
  forced-on every single-module design is a 1-`Fsm` leaf that
  validates + emits the `.3.1`-probed-clean template; **all three
  generated encodings reachable** across the sweep. Full `cargo` gate
  green. No ROADMAP advance (promotion is `.3.4` on a verified gate).
  Frontier → `.3.3` (cargo-portable structural + CSE/EGraph-opacity
  proof, mirrors memory `.2.2`) (‖ `.2.4` gate-blocked).
- `2026-05-18`: **`.3.3` landed** (continuous-PNT while the `.2.4`
  memory gate runs). Cargo-portable proof
  `fsm_block_matches_probed_template_and_is_factorization_opaque`
  across 4 `ConstructionStrategy` × 4 `FactorizationLevel`
  (incl. EGraph) × 6 seeds (96 designs): exactly one `FsmOut`
  survives every factorization level and the FSM leaf has zero
  `Gate` nodes (the state machine never enters the NodeId expression
  graph — CSE/EGraph-opaque); the emitted state constants are
  *exactly* the chosen `FsmEncoding`'s
  (`state_width`/`state_const`), proving structural correctness +
  encoding distinctness (robust where Binary/Gray coincide at N=2),
  plus the exact async-low-reset state register and the
  sel-selected next-state/Moore cases; all three encodings
  reachable across the deterministic seeds-0..6 sweep. Proof-only
  (no `src/`); full `cargo` gate green. No ROADMAP advance
  (promotion is `.3.4`). Frontier → `.3.4` — the **last Phase 6
  leaf**: `phase6_fsm` matrix scenario + `num_fsm_modules` metric +
  `saw_fsm_design` fact/gap, then a real repo-owned gate verified
  clean → records FSM delivered + (memory delivered at `.2.4`)
  closes ROADMAP Phase 6 + the `PHASE-6-ADVANCED-MOTIFS` tree
  (‖ `.2.4` gate-blocked).
- `2026-05-18`: **`.3.4` split + `.3.4a` landed** (continuous-PNT
  while the `.2.4` memory gate runs). `.3.4` → `.3.4a` (scenario,
  code, unblocked) / `.3.4b` (real-gate verify + Phase-6/tree
  closure, gate-blocked) — Splitting Rules + the proven memory
  `.2.3`/`.2.4` decomposition + r87. `.3.4a` mirrors memory `.2.3`
  exactly: `DesignMetrics.num_fsm_modules`; `phase6_fsm` matrix
  scenario + `phase6_fsm_focus_config` (clone of the memory anchor,
  only `fsm_prob=1.0` ⇒ shape-coverage sets unperturbed);
  `CoverageSummary.saw_fsm_design` + set + merge +
  `Phase4Hierarchy` gap; bin **219→222 / 876→888** (verified, +3/+12
  matching `phase6_inferrable_memory`); scenario-name exception list;
  `phase6_fsm_scenario_is_non_vacuous`. `cargo test --bin
  tool_matrix` 29/29; full `cargo` gate green. No ROADMAP advance
  (promotion is `.3.4b` on a verified gate). Frontier → `.3.4b` —
  the **last Phase 6 leaf** (a fresh real repo-owned gate, now
  including `phase6_fsm`, verified clean → records FSM delivered +
  closes ROADMAP Phase 6 + the `PHASE-6-ADVANCED-MOTIFS` tree;
  one fresh run covers memory + fsm) (‖ `.2.4` gate-blocked).
