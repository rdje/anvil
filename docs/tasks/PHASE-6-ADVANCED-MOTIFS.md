# PHASE-6-ADVANCED-MOTIFS: Memories, FSMs, optional multi-clock

## Metadata

- Tree ID: `PHASE-6-ADVANCED-MOTIFS`
- Status: `active`
- Roadmap lane: Phase 6 — Advanced motifs
- Created: `2026-05-16`
- Last updated: `2026-05-18` (`.2.3` matrix scenario+metric+gap landed; frontier → `.2.4`)
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
  Children: `PHASE-6-ADVANCED-MOTIFS.1` (done), `PHASE-6-ADVANCED-MOTIFS.2` (active container: `.2.1`–`.2.4`), `PHASE-6-ADVANCED-MOTIFS.3` (pending — FSM)

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
  Status: `pending`
  Goal: `Generated-state-encoding FSM motif (design + implementation + matrix scenario). May split into design/impl leaves when reached.`
  Acceptance: `FSM-encoding designs downstream-clean; encoding selectable; ROADMAP Phase 6 advances.`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `PHASE-6-ADVANCED-MOTIFS.2.4` | `pending` | `.2.3` landed the `phase6_inferrable_memory` matrix scenario + `num_memory_modules` metric + `saw_inferrable_memory_design` fact/gap (bin 216→219 / 864→876; scenario proven non-vacuous). `.2.4` runs the real repo-owned `Phase4Hierarchy` gate, verifies downstream-clean (`coverage_gaps=[]`, Verilator + both Yosys all-pass, `saw_inferrable_memory_design=true`, P4/P5/P5b regressions clean), then records memory **delivered** in ROADMAP Phase 6 (Phase 6 stays open for `.3` FSM — no tree closure) + reconciles the book — promotion strictly follows the verified artifact (r87 no-aspirational-claims). |

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

## Open Questions

- Resolved by `.1` (empirical probe): the **single-port sync-write /
  sync-read** template and the **simple dual-port** template (1 write
  port + 1 independent synchronous read port) are both reliably
  inferred as `$mem_v2` by Yosys `memory_collect`, and synth clean
  (`check -assert`, exit 0) in **both** repo Yosys modes
  (`synth -noabc`, `synth; abc -fast`); Verilator `--lint-only` exit
  0. Recorded in `DEVELOPMENT_NOTES.md` "Phase 6 inferrable-memory
  motif design". (No open questions remain for `.1`.)

## Blockers

- None. Independent of Phase 4/5.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-05-18` | `PHASE-6-ADVANCED-MOTIFS.1` | `DEVELOPMENT_NOTES.md` Phase 6 memory design entry landed (codebase-grounded; empirical Yosys probe → single-port + simple-dual-port both `1 $mem_v2`, clean both modes; architecture (M) `Memory` block + opaque `MemRead` leaf; 3 rejected alternatives; proof shape). Doc-only, no code; `mdbook build book` clean; `cargo fmt --check` clean; `cargo test` unchanged-green (no `src/`/`tests/` touched since Phase 5b `.2.3`). | Done. |
| `2026-05-18` | `PHASE-6-ADVANCED-MOTIFS.2.1a` | IR core (`MemId`/`MemKind`/`Memory`/additive `Module.memories`/opaque `Node::MemRead`/`DepAtom::MemVirtual`/`from_mem_virtual`/`has_local_memories`) + `MemRead` threaded through all ~21 exhaustive `Node` matches (compiler-as-oracle, mirrors `FlopQ`); **load-bearing `compact.rs` reachability** (a live `MemRead` keeps `mem.{we,waddr,wdata,raddr}` cones alive; `StructuralNodeShape`/`LeafEndpoint::MemRead`); emitter inferrable template (`mem_<id>` array + `memrd_<id>` + reset-less `always_ff @(posedge clk)`); validator memory step-5b. 3 unit proofs (roundtrip+validate+emit; **compaction-reachability**; structural-distinctness/CSE-opacity) — `ir::compact::tests` mem 3/3. `cargo fmt`/`clippy -D warnings`/`check --all-targets` clean; full `cargo test` (COMMIT.md gate). No generator/knob ⇒ default-off trivially byte-identical. No `book/` change. | Done. |
| `2026-05-18` | `PHASE-6-ADVANCED-MOTIFS.2.1b` | `Config::memory_prob` (serde-default 0.0 + validation, mirrors `aggregate_prob`); rules-first `build_memory_leaf` (`clk`/`rst_n`+`we`/`waddr`/`wdata`[+`raddr`] inputs, `rdata` out, one `Memory` kind rolled via `g.rng`, opaque `MemRead` drive; no gates/flops) + single opt-in roll after the Phase 5 param lane (mutually exclusive; default-off never enters). Focused proof `inferrable_memory_is_default_off_and_constructs_when_forced_on` (default-off byte-identical 4 strategies × 6 seeds; forced-on every single-module design is a 1-`Memory` leaf, validates, emits the inferrable template). Real spot-check (binary seed 3, prob 1.0): `verilator --lint-only` exit 0; yosys `memory_collect` → `1 $mem_v2`; `synth -noabc` & `synth;abc -fast` both `check -assert` clean. `cargo fmt`/`clippy -D warnings`/`check --all-targets` clean; full `cargo test` (COMMIT.md gate). No `book/` change. | Done (closes the `.2.1` container). |
| `2026-05-18` | `PHASE-6-ADVANCED-MOTIFS.2.2` | `tests/pipeline.rs::inferrable_memory_matches_yosys_template_and_is_factorization_opaque` — across 4 `ConstructionStrategy` × 4 `FactorizationLevel` (None/Cse/Commutative/EGraph) × 4 seeds (64 combos): `validate_design` clean; SV is exactly the `.1`-validated Yosys-inferrable form (concrete `mem_0 [0:depth]`, reset-less `always_ff @(posedge clk)`, `if (we) mem_0[..] <= wdata;`, `memrd_0 <= mem_0[..]`); exactly one `MemRead` survives every factorization level + zero expression-graph `Gate` nodes (array/`MemRead` never enter the NodeId graph — CSE/factorization-opaque incl. EGraph). Tool-level `$mem_v2`/Verilator proof is `.2.1b`'s spot-check (interim) + `.2.4`'s real gate (authoritative; cargo can't shell yosys — see Decisions). Default-off byte-identical reaffirmed by the `.2.1b` proof. `cargo fmt`/`clippy -D warnings`/`check --all-targets` clean; full `cargo test` (COMMIT.md gate). No `book/` change. | Done. |
| `2026-05-18` | `PHASE-6-ADVANCED-MOTIFS.2.3` | `DesignMetrics.num_memory_modules` + populate; `phase6_inferrable_memory_focus_config` (clone of the phase5b/dedup anchor — depth-1 wrapper, library, `memory_prob=1.0`, 4 leaves/4 instances → shape-coverage sets unperturbed) + `phase6_inferrable_memory` scenario tuple; `CoverageSummary.saw_inferrable_memory_design` set/merge + Phase4Hierarchy `compute_coverage_gaps` arm; bin counts 216→219 / 864→876 (observed) + exception-list entry; tool_matrix phase4 bin tests 3/3; new `phase6_inferrable_memory_scenario_is_non_vacuous` proves the scenario builds ≥1 memory module per strategy (coverage fact reachable). `cargo fmt`/`clippy -D warnings`/`check --all-targets` clean; full `cargo test` (COMMIT.md gate). ROADMAP unchanged (advance is `.2.4`). No `book/` change. | Done. |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `PHASE-6-ADVANCED-MOTIFS.1` | `Docs: PHASE-6-ADVANCED-MOTIFS.1 inferrable-memory motif design` | Design-only; architecture (M) `Memory` block + `MemRead` leaf; empirical Yosys probe; 3 rejected alternatives. No code. |
| `PHASE-6-ADVANCED-MOTIFS.2.1a` | `Phase 6: PHASE-6-ADVANCED-MOTIFS.2.1a memory IR core + opaque-stateful-leaf pipeline integration` | IR core + `MemRead` through ~21 matches + the load-bearing `compact.rs` reachability + emitter template + validator + 3 unit proofs. No generator/knob (default-off trivially byte-identical). |
| `PHASE-6-ADVANCED-MOTIFS.2.1b` | `Phase 6: PHASE-6-ADVANCED-MOTIFS.2.1b memory_prob knob + rules-first build_memory_leaf` | Knob + rules-first constructor + opt-in roll (mutually exclusive with the param lane) + focused proof; generated memory spot-checked `1 $mem_v2` + verilator/both-yosys clean. Closes the `.2.1` container. |
| `PHASE-6-ADVANCED-MOTIFS.2.2` | `Phase 6: PHASE-6-ADVANCED-MOTIFS.2.2 memory inference structural-contract + factorization-opacity proof` | Cargo-portable structural-template-equivalence + factorization/CSE-opacity (64 combos incl. EGraph); tool-level `$mem_v2` proof = `.2.1b` spot-check + `.2.4` gate. No code change (proof only). |
| `PHASE-6-ADVANCED-MOTIFS.2.3` | `Phase 6: PHASE-6-ADVANCED-MOTIFS.2.3 phase6_inferrable_memory matrix scenario + metric + gap` | `num_memory_modules` metric + `phase6_inferrable_memory` scenario + `saw_inferrable_memory_design` fact/gap; bin 216→219 / 864→876; non-vacuity test. No ROADMAP advance (that is `.2.4` on verified evidence). |

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
