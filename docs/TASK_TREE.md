# Repo-Local Task Tree Workflow

This document defines the repo-local task-tree workflow used by ANVIL. It is
intentionally portable: the workflow was lifted from FSMGen's
`docs/TASK_TREE.md` and adapted to ANVIL's existing live-doc set.

For the portable, project-agnostic setup guide, read
[docs/TASK_TREE_README.md](TASK_TREE_README.md).

## Purpose

Use a task tree when a top-level task is too broad to finish safely as one
signoff-level slice, or when a task is expected to discover subtasks and
sub-subtasks over time.

The goal is not to create a second roadmap. `ROADMAP.md` states the high-level
phase direction. A task tree owns the recursive breakdown, current frontier,
acceptance criteria, blockers, decisions, validation, and completion evidence
for one top-level task.

## ANVIL Adoption Scope

**Doctrine (2026-05-17, non-negotiable, owner directive):** it is
**strictly forbidden to make any code change without it being
task-tree tracked or task-tree owned first.** Task-tree ownership
demonstrably improved code review and code quality over the earlier
ad-hoc/linear cadence, so it is now the mandatory mode of work for all
code — no compromise, no exceptions.

- **Code change ⇒ a task-tree leaf must own it, *before* the edit.**
  "Code" means anything that changes program/generator behaviour or
  generated RTL: `src/`, `tests/`, `examples/`, build/codegen logic,
  `Cargo` manifests that alter behaviour. If no tree/leaf covers the
  change, create or extend one (`docs/tasks/<TREE>.md` + a
  `docs/TASK_TREE.md` row) and name the owning leaf first. The leaf ID
  goes in the commit subject / first body line (`COMMIT.md` task-tree
  rules).
- **Exempt (no tree required):** pure-docs / live-doc / mdBook edits,
  workflow-config tweaks, and recording doctrine itself. These are not
  code changes. They still follow the standard `COMMIT.md` checklist.
- **`rN` is *not* retired** — it survives only as the optional
  within-leaf slice cadence *inside* a task tree (as the closed
  `HIERARCHY-AWARE-IDENTITY` leaves landed as r85/r86/r87). A bare
  `rN` slice that no task-tree leaf owns is no longer a legal way to
  land a code change.
- **Do not migrate finished work** retroactively. Closed `rN` slices
  stay where they are; the mandate is forward-going.

**Project-wide tracking directive (2026-05-16):** by explicit owner
directive, *every remaining roadmap phase* now has a registered
top-level task tree (`PHASE-4-HIERARCHY`, `PHASE-5-PARAMETERIZATION`,
`PHASE-5B-AGGREGATES`, `PHASE-6-ADVANCED-MOTIFS`,
`PHASE-7-ORACLE-MICRODESIGN`, `PHASE-8-FRONTEND-ACCEPT`,
`PHASE-9-MULTI-ARTIFACT-UMBRELLA`) so the whole roadmap is trackable
through task trees. This **does not retire `rN`**: `rN` remains the
within-leaf slice cadence. Each phase tree owns the sub-objective
decomposition, frontier, blockers, and completion evidence; individual
linear coverage slices inside a leaf still land under the `rN` naming +
`CHANGES.md` + `MEMORY.md` combination, exactly as the closed
`HIERARCHY-AWARE-IDENTITY` tree's leaves landed as r85/r86/r87. Closed
`rN` slices are still not migrated retroactively.

## Active Task Trees

| Tree | Status | Roadmap lane | Current frontier | File |
| --- | --- | --- | --- | --- |
| `COMBINATIONAL-SEMANTIC-IDENTITY` | `done` | NodeId as identity / full-factorization mode | complete — `.1` gate-to-existing-endpoint / constant fold; `.2` shallow 12-bit semantic proof budget with fixed work envelope; `.3` closeout | [docs/tasks/COMBINATIONAL-SEMANTIC-IDENTITY.md](tasks/COMBINATIONAL-SEMANTIC-IDENTITY.md) |
| `SEQUENTIAL-COINDUCTIVE-IDENTITY` | `done` | NodeId as identity / full-factorization mode | complete — `.1` proof inventory; `.2.1` domain-aware flop identity; `.2.2` exact reset-defined self-hold merge; `.3` closeout | [docs/tasks/SEQUENTIAL-COINDUCTIVE-IDENTITY.md](tasks/SEQUENTIAL-COINDUCTIVE-IDENTITY.md) |
| `MEMORY-STATE-IDENTITY` | `done` | NodeId as identity / full-factorization mode | complete — `.1` reset-defined proof boundary; `.2` blocker record; `.3` closeout. Current reset-less memories remain state-by-instance; reset-defined memory sharing is blocked for the current warning-clean memory-inference lane. | [docs/tasks/MEMORY-STATE-IDENTITY.md](tasks/MEMORY-STATE-IDENTITY.md) |
| `HIERARCHY-SEMANTIC-IDENTITY` | `done` | NodeId as identity / hierarchical module identity | complete — `.1` pure-combinational leaves; `.2` bounded pure-combinational wrappers with recursively proven children; `.3` closeout and blockers | [docs/tasks/HIERARCHY-SEMANTIC-IDENTITY.md](tasks/HIERARCHY-SEMANTIC-IDENTITY.md) |
| `SIGNOFF-SURFACE-EXPANSION` | `done` | Quality / signoff-level downstream confidence | complete — `.1` N-flop CDC synchronizer; `.2` Verilator JSON frontend parity; `.3` Icarus compile axis; `.4` closeout and explicit deferred boundaries | [docs/tasks/SIGNOFF-SURFACE-EXPANSION.md](tasks/SIGNOFF-SURFACE-EXPANSION.md) |
| `ROADMAP-FOLLOWUP-OWNERSHIP` | `done` | Workflow / roadmap task-tree ownership | complete — `.1` registered the five post-phase follow-up trees before implementation resumed | [docs/tasks/ROADMAP-FOLLOWUP-OWNERSHIP.md](tasks/ROADMAP-FOLLOWUP-OWNERSHIP.md) |
| `HIERARCHY-AWARE-IDENTITY` | `done` | Phase 4 — Hierarchy | (complete — all leaves done) | [docs/tasks/HIERARCHY-AWARE-IDENTITY.md](tasks/HIERARCHY-AWARE-IDENTITY.md) |
| `PHASE-4-HIERARCHY` | `done` | Phase 4 — Hierarchy | (complete — `.1` done, `.2` superseded, `.3` done; Phase 4 closed) | [docs/tasks/PHASE-4-HIERARCHY.md](tasks/PHASE-4-HIERARCHY.md) |
| `PHASE-5-PARAMETERIZATION` | `done` | Phase 5 — Parameterization | (complete — Phase 5 closed `2026-05-17`; `.2.4b` verified `/tmp/anvil-tool-matrix-phase5-p1` clean → ROADMAP Phase 5 `done`) | [docs/tasks/PHASE-5-PARAMETERIZATION.md](tasks/PHASE-5-PARAMETERIZATION.md) |
| `PHASE-5B-AGGREGATES` | `done` | Phase 5b — Synthesizable aggregates | (complete — Phase 5b closed `2026-05-18`; `.2.4` verified `/tmp/anvil-tool-matrix-phase5b-p1` clean → ROADMAP Phase 5b `done`) | [docs/tasks/PHASE-5B-AGGREGATES.md](tasks/PHASE-5B-AGGREGATES.md) |
| `PHASE-6-ADVANCED-MOTIFS` | `done` | Phase 6 — Advanced motifs | (complete — Phase 6 closed `2026-05-20`; **memory** verified `/tmp/anvil-tool-matrix-phase6-p1` clean [`.2.4`, 219/876, `coverage_gaps=[]`, 876/0 Verilator+both-Yosys, `saw_inferrable_memory_design=true`] **and FSM** verified `/tmp/anvil-tool-matrix-phase6-fsm-p1` clean [`.3.4b`, 222/888, `coverage_gaps=[]`, 888/0 Verilator+both-Yosys, `saw_fsm_design=true` AND `saw_inferrable_memory_design=true`, P4/P5/P5b regressions proven in the same banked report] → ROADMAP Phase 6 `done`; the separately-prioritised `MULTI-CLOCK-CDC` follow-up is now closed too) | [docs/tasks/PHASE-6-ADVANCED-MOTIFS.md](tasks/PHASE-6-ADVANCED-MOTIFS.md) |
| `PHASE-7-ORACLE-MICRODESIGN` | `done` | Phase 7 — Oracle-backed micro-design artifacts | (complete — Phase 7 closed `2026-05-20`; verified-clean banked artifact `/tmp/anvil-microdesign-parity-phase7-yosys-p1/` — `cargo test -- --ignored parity_against_real_yosys_write_json` against yosys 0.64 exits 0 with "parity gate clean across 5 seeds"; per-seed fact agreement verified incl. seed 7 P4=-1 [bits=8 on both sides post-`.2c.2b.1` non-negative-modulo-idiom fix] and both generate branches exercised [seed 12345 takes `g_else`, others `g_taken`]; explicit yosys-supported-categories scope caveat — richer-AST coverage via a future microdesign-specific extractor is recorded as post-Phase-7 follow-up and does NOT retract closure since ANVIL's by-construction oracle already covers all 7 manifest categories) | [docs/tasks/PHASE-7-ORACLE-MICRODESIGN.md](tasks/PHASE-7-ORACLE-MICRODESIGN.md) |
| `PHASE-8-FRONTEND-ACCEPT` | `done` | Phase 8 — Frontend/elaboration accept corpora | (complete — Phase 8 closed `2026-05-20`; verified-clean banked artifact `/tmp/anvil-frontend-parity-phase8-yosys-p1/` — `cargo test -- --ignored parity_against_real_yosys_hierarchy_write_json` against yosys 0.64 exits 0 with "parity gate clean across 5 seeds" on **first try**; per-seed fact agreement verified incl. both generate branches exercised AND the load-bearing hierarchy-aware Phase-8 axis (every seed has 2 instances × 4 per-instance per-binding values matched); yosys-supported-categories scope caveat explicit — yosys folds top localparams + package constants; `SIGNOFF-SURFACE-EXPANSION.2` now adds the optional Verilator JSON-AST gate `parity_against_real_verilator_json_frontend_ast`, clean across the same 5 seeds and enforcing all 7 Phase-8 manifest categories when local Verilator supports `--json-only`; cross-tree reuse of Phase 7's `expr_to_sv` carried `.2c.2b.1`'s non-negative-modulo-idiom fix forward at zero incremental cost — full-factorization doctrine vindicated) | [docs/tasks/PHASE-8-FRONTEND-ACCEPT.md](tasks/PHASE-8-FRONTEND-ACCEPT.md) |
| `PHASE-9-MULTI-ARTIFACT-UMBRELLA` | `done` | Phase 9 — Multi-artifact ANVIL umbrella | (complete — Phase 9 closed `2026-05-20`; `src/umbrella/` carries the `ArtifactLane` trait + all 3 lane impls + 8 cargo-portable proofs incl. per-lane byte-identical regression + cross-lane heterogeneous `dyn` dispatch; `src/main.rs` carries the `--artifact <lane>` CLI flag with default `dut`; load-bearing byte-identical default-`dut` contract verified by `tests/book_examples::every_runnable_book_bash_block_succeeds` passing 3/3 in 80s AFTER the CLI change. **All 9 numbered roadmap phases now delivered.** Post-phase follow-up trees `DIFFERENTIAL-SIMULATION` and `MULTI-CLOCK-CDC` are also closed as of `2026-05-24`.) | [docs/tasks/PHASE-9-MULTI-ARTIFACT-UMBRELLA.md](tasks/PHASE-9-MULTI-ARTIFACT-UMBRELLA.md) |
| `INSTA-SNAPSHOTS` | `done` | Quality — reproducibility regressions | (complete — closed `2026-05-18`; `.1` insta `=1.47.2` pin + baseline / `.2` 6 byte-stable shapes spanning every reachable axis incl. dedup-canonical-signatures / `.3` COMMIT.md non-negotiable snapshot-acceptance protocol + book "Snapshot guard-rails") | [docs/tasks/INSTA-SNAPSHOTS.md](tasks/INSTA-SNAPSHOTS.md) |
| `DIFFERENTIAL-SIMULATION` | `done` | Quality — signoff-level downstream consistency | (**complete — entire tree closed `2026-05-24`**; all four `.1`/`.2`/`.3`/`.4` leaves done) **`.4` landed `2026-05-24`** (docs-only): README + USER_GUIDE + `book/src/synthesizability.md` describe the `--diff-sim` opt-in cross-simulator semantic-agreement contract — per-axis K=5 subset, gated AFTER Verilator+Yosys clean, `DiffSimReport` with retained `mismatch_excerpt`, `saw_design_with_cross_simulator_agreement` coverage fact, friendly no-op when simulators absent. `mdbook build` clean; `cargo test --test book_examples` 3/3 still green (new bash block carries `<!-- book-test: skip -->` sentinel preserving byte-identical book-runnable contract). **`.3b.2` closed `.3b` + `.3` container `2026-05-24`**: `src/bin/tool_matrix.rs` (~600 lines added) gains the `--diff-sim` opt-in CLI flag + `DiffSimReport` per-module struct + `saw_design_with_cross_simulator_agreement` coverage fact + per-axis subset selector + per-module pipeline + `parse_dut_ports`/`emit_testbench_for_ports` matrix-side helpers + 8 cargo-portable proofs + 1 tool-gated `#[ignore]` end-to-end gate; real-tool gate clean: `DiffSimReport { ran: true, success: true, n_samples: 8 }` (24.15s wall against iverilog 13.0 + verilator 5.046); FOUND-AND-FIXED spec-vs-reality bug during e2e gate (ANVIL emits `"input  logic"` with TWO spaces — `src/emit/sv.rs:124`; replaced `strip_prefix` with `split_whitespace`). **`.3b.1` done `2026-05-24`** (pure refactor → `src/diff_sim/mod.rs`); **`.3a` design landed `2026-05-24`** (docs-only); **`.2b.2` closed `.2b` + `.2` container `2026-05-24`** — first gate to assert downstream-tool *semantic* agreement on ANVIL output (`project_anvil_north_star.md`). | [docs/tasks/DIFFERENTIAL-SIMULATION.md](tasks/DIFFERENTIAL-SIMULATION.md) |
| `COVERAGE-INSTRUMENTATION` | `done` | Quality — test-discipline visibility | (complete — closed `2026-05-18`; `.1` llvm-cov baseline / `.2` top-5 triage [no dead code] / `.3` cone retry-exhaustion focused proof + config orphan-knob audit [3 documented-reserved knobs] + baseline refresh) | [docs/tasks/COVERAGE-INSTRUMENTATION.md](tasks/COVERAGE-INSTRUMENTATION.md) |
| `BOOK-EXAMPLES-RUNNABLE` | `done` | Quality — user-facing book correctness | (complete — closed `2026-05-18`; `.1`/`.2.1`/`.2.2` done: 45+1 examples migrated to `cargo run --release --`, `tests/book_examples.rs` harness + `mdbook test` CI gate, pipe-deadlock root-caused & fixed, `cargo test --test book_examples` 3/3 green, 54 runnable exit-0) | [docs/tasks/BOOK-EXAMPLES-RUNNABLE.md](tasks/BOOK-EXAMPLES-RUNNABLE.md) |
| `MULTI-CLOCK-CDC` | `done` | Capability / Quality — relax single-clock-domain invariant + emit by-construction CDC primitives | (**complete — entire tree closed `2026-05-24`**; all four `.1`/`.2`/`.3`/`.4` leaves done) **`.4` landed `2026-05-24`** (docs-only): `book/src/sequential.md` "Multi-clock and CDC" subsection describes the K=1 default + K=N multi-clock case + by-construction 2-flop synchronizer wrap + opt-in + 4-step pass flow + matrix scenario + coverage facts + first-cut MVP scope; README.md closure status updated. mdbook build clean; book_examples 3/3 byte-identical (no new bash blocks). **`.3b.2` closed `.3b` + `.3` container `2026-05-24`**: `Metrics.num_clock_domains` + `num_cdc_2_flop_synchronizers` in `src/metrics.rs` + `count_2flop_synchronizer_chains` helper (structural scan of `Module.flop_domains` for the synchronizer template `second.D == first.Q`); `CoverageSummary.saw_multi_clock_design` + `saw_cdc_2_flop_synchronizer` + merge + summarize updates; new `int_multi_clock_2flop_sync` default scenario; `Generator::generate_module` rewired to apply the promotion pass (single-module path matching tool_matrix's per-scenario flow); `promote_to_multi_clock` made idempotent (preserves byte-identical when both single-module + design-level paths fire). 4 new bin tool_matrix proofs. **End-to-end matrix gate clean**: Verilator 16/16 + Yosys 16/16; aggregate coverage facts both lit; multi-clock module recorded `num_clock_domains=2 num_cdc_2_flop_synchronizers=1`. **The first ANVIL multi-clock SV passed both downstream tools first try** — validates the entire `.2` + `.3a` + `.3b.1` + `.3b.2` chain end-to-end. K=1 default 0.0 byte-identical (snapshots + book_examples 3/3 in 84.10s preserved). **`.3b.1` done `2026-05-24`**: `Config.multi_clock_prob: f64` knob (default 0.0 backward-compatible); `multi_clock::promote_to_multi_clock` post-construction pass wired into `Generator::generate_design` per the `aggregate_prob` pattern (per-module Bernoulli roll on the seeded RNG; if eligible, adds clk_b/rst_n_b ports + 2 `ClockDomain` entries + wraps first 1-bit flop-driven output via `.3a` primitive + rewires output's drive to synced Q; declines cleanly on no-clock/no-output/wide-output modules). `PromotionOutcome { promoted, num_domains, num_synchronizers }` for `.3b.2` coverage. 8 new cargo-portable proofs incl. 2 end-to-end Generator integration tests. lib 256 → 264. K=1 default 0.0 byte-identical (snapshots + book_examples 3/3 in 84.28s preserved). `.3b` split per Phase-7 `.2c.2a`/`.2c.2b` discipline. **`.3a` done `2026-05-24`**: new `src/gen/multi_clock.rs` (~250 lines) carries `pub fn construct_2flop_synchronizer(module, src_q, dst_domain) -> Option<SynchronizerChain>` + `SynchronizerChain { first_flop, second_flop, synced_q }`. Both new flops land in `dst_domain` via `Module.flop_domains`; chain D=src_q → first → second → synced_q; width inherited via `Node::width()`. 5 cargo-portable proofs incl. end-to-end emit-shape integration. lib 251 → 256. K=1 byte-identical (snapshots + book_examples 3/3 in 72.71s preserved). `.3` split per Phase-7 `.2c.2a`/`.2c.2b` discipline. **`.2` done `2026-05-24`**: IR extension landed in `src/ir/types.rs` (new `ClockDomain { clk, rst_n, name }` struct + `Module.clock_domains: Vec<ClockDomain>` + `Module.flop_domains: BTreeMap<FlopId, u32>`, defaults empty for K=1 backward compat); emitter refactored in `src/emit/sv.rs` to per-domain `always_ff` loop via `Module::effective_clock_domains` + `Module::flop_domain` accessors. **K=1 byte-identical** verified by snapshots 6/6 + book_examples 3/3 in 85s (default-`dut` contract preserved across the IR + emit refactor). **K=2 proven** by hand-built `emits_one_always_ff_block_per_clock_domain_when_k_equals_two` lib unit proof. 4 new lib unit proofs (lib 247 → 251). All other suites unchanged. Minimum-blast-radius design (`Module.flop_domains` external BTreeMap, not `Flop.domain` field) kept 23 Flop construction sites at zero touches. **`.1` design landed `2026-05-24`** (docs-only): `DEVELOPMENT_NOTES.md` "Multi-clock + CDC primitives design" records 7-tier CDC primitive catalogue (Tier 1 = 2-flop synchronizer first cut; tiers 2-7 deferred to follow-up leaves or their own task trees); minimum-viable IR shape (`Module.clock_domains: Vec<ClockDomain>` + per-flop `Flop.domain: usize`; K=1 backward-compatible — existing tests + book-runnable contract stay byte-identical with `--multi-clock-prob` default 0.0); by-construction synchronizer rule (rules-first per `feedback_rules_first_generation.md` — never generate-then-filter); Verilator `--cdc=metastable` downstream gate (Yosys `-cdc` rejected — doesn't exist in stable 0.64; custom oracle deferred to `.4`); cross-simulator agreement via the just-landed `--diff-sim`; 6 rejected alternatives (single-flop synchronizer; clock-gating; latches; async-FIFO as min-viable; generate-then-filter; dynamic frequency); `.2`-`.4` leaf shape; `--multi-clock-prob: f64` knob. Opened as the only remaining named follow-up after `DIFFERENTIAL-SIMULATION` closed `2026-05-24`; the single-clock-domain invariant was the explicit Phase-6 deferral. | [docs/tasks/MULTI-CLOCK-CDC.md](tasks/MULTI-CLOCK-CDC.md) |
| `LIVE-DOC-PATH-HYGIENE` | `done` | Workflow / live-doc hygiene | (complete — `.1` rewrote local absolute repo paths to repo-root-relative references, aligned stale closed-tree status metadata, and passed full `COMMIT.md` validation) | [docs/tasks/LIVE-DOC-PATH-HYGIENE.md](tasks/LIVE-DOC-PATH-HYGIENE.md) |
| `MEMORY-ARCHITECTURE-DOC` | `done` | Workflow / memory architecture | (complete — `.1` standard + README pointer; `.2` ANVIL layer-C decisions; `.3` bounded `MEMORY.md`; `.4` self-check/hooks/CI/bootstrap enforcement; `.5` final focused validation clean; full cargo test intentionally out of scope per owner resource policy) | [docs/tasks/MEMORY-ARCHITECTURE-DOC.md](tasks/MEMORY-ARCHITECTURE-DOC.md) |
| `KNOWLEDGE-MAP-DOC` | `done` | Workflow / retrieval architecture | complete — `.1` project-agnostic bundle + discovery pointers; `.2` generated map + hook/CI enforcement; `.3` ANVIL decision-record retrieval keys + close. | [docs/tasks/KNOWLEDGE-MAP-DOC.md](tasks/KNOWLEDGE-MAP-DOC.md) |
| `SEQUENTIAL-IDENTITY` | `done` | NodeId as identity / full-factorization mode | complete — `.1` merges equivalent generated FSM blocks under node-id identity, surfaces `fsms_merged`, and documents the FSM-vs-memory proof boundary. | [docs/tasks/SEQUENTIAL-IDENTITY.md](tasks/SEQUENTIAL-IDENTITY.md) |
| `LIVE-DOC-IDENTITY-ALIGNMENT` | `done` | Live docs / NodeId identity status | complete — `.1` aligned stale CODEBASE identity-status prose after the FSM merge and existing hierarchy module-dedup layer. | [docs/tasks/LIVE-DOC-IDENTITY-ALIGNMENT.md](tasks/LIVE-DOC-IDENTITY-ALIGNMENT.md) |
| `LIVE-DOC-ROADMAP-ALIGNMENT` | `done` | Live docs / roadmap follow-up status | complete — `.1` aligned current roadmap/index/codebase follow-up status after `MULTI-CLOCK-CDC` and `DIFFERENTIAL-SIMULATION` closure. | [docs/tasks/LIVE-DOC-ROADMAP-ALIGNMENT.md](tasks/LIVE-DOC-ROADMAP-ALIGNMENT.md) |
| `LIVE-DOC-CODEBASE-ALIGNMENT` | `done` | Live docs / CODEBASE_ANALYSIS ↔ workspace alignment | complete — `.1` added the 5 omitted modules (`ir/param.rs`, `ir/aggregate.rs`, `frontend/`, `umbrella/`, `diff_sim/`) to the module map and corrected the integration-test count 3→6; surfaced by the session-bootstrap deep-dive, no other drift found. | [docs/tasks/LIVE-DOC-CODEBASE-ALIGNMENT.md](tasks/LIVE-DOC-CODEBASE-ALIGNMENT.md) |
| `HIERARCHY-DEDUP-PRUNE` | `done` | NodeId as identity / hierarchical module identity | complete — `.1` prunes modules made unreachable by opt-in hierarchy dedup merges while preserving no-merge under-instantiation and pre-existing top-unreachable modules from reachability cleanup. | [docs/tasks/HIERARCHY-DEDUP-PRUNE.md](tasks/HIERARCHY-DEDUP-PRUNE.md) |
| `MEMORY-IDENTITY-BOUNDARY` | `done` | NodeId as identity / full-factorization mode | complete — `.1` proves and documents that current inferrable memories remain state-by-instance under full-factorization passes because their stored contents are not reset-defined. | [docs/tasks/MEMORY-IDENTITY-BOUNDARY.md](tasks/MEMORY-IDENTITY-BOUNDARY.md) |
| `HIERARCHY-IDENTITY-BOUNDARY` | `done` | NodeId as identity / hierarchical module identity | complete — `.1` proves and documents that module dedup remains structural-only, not arbitrary semantic module equivalence. | [docs/tasks/HIERARCHY-IDENTITY-BOUNDARY.md](tasks/HIERARCHY-IDENTITY-BOUNDARY.md) |
| `ENDPOINT-IDENTITY-BOUNDARY` | `done` | NodeId as identity / full-factorization mode | complete — `.1` proves and documents that same-shape semantic cones over different leaf endpoints do not merge. | [docs/tasks/ENDPOINT-IDENTITY-BOUNDARY.md](tasks/ENDPOINT-IDENTITY-BOUNDARY.md) |
| `LIVE-DOC-BOOK-ALIGNMENT` | `done` | Live docs / mdBook ↔ codebase alignment | complete — `.1` corrected mdBook chapters that still labelled delivered Phase 5-9 motifs (memories, parameterization, Phase 7-9 lanes) as "future". | [docs/tasks/LIVE-DOC-BOOK-ALIGNMENT.md](tasks/LIVE-DOC-BOOK-ALIGNMENT.md) |
| `RESOURCE-SAFE-TOOLING` | `done` | Quality / workflow — resource-safe validation | complete — `.1` `scripts/ram_guard.sh` RAM watchdog; `.2` USER_GUIDE "Resource-safe runs" docs. | [docs/tasks/RESOURCE-SAFE-TOOLING.md](tasks/RESOURCE-SAFE-TOOLING.md) |
| `AGGREGATE-ARRAY-PACKING` | `done` | Phase 5b follow-on — synthesizable aggregates (packed array) | complete — `.1`–`.5` done (`AggregateKind::ArrayPacked` + emitter + `aggregate_array_prob` selection + metric + 7/7 Verilator/Yosys downstream-clean + book/docs sync); `.4b` (optional matrix CI instrumentation) `deferred`. Default-off byte-identical. | [docs/tasks/AGGREGATE-ARRAY-PACKING.md](tasks/AGGREGATE-ARRAY-PACKING.md) |
| `WORKLOAD-MEMORY-SAFETY` | `done` | Quality / signoff — resource-safe generation (ANVIL's own runtime) | complete — `.1` design; `.2` streamed directory-output manifest (peak metadata RAM O(1) in `--count`); `.3` real per-module node budget (`max_nodes_per_module`); `.4` opt-in internal RAM/RSS self-governor (`src/mem_guard.rs`, `--max-rss-mb`/`--ram-abort-pct`, between-unit `--out` checkpoint, clean exit 99); `.5` closeout (book safe-envelope narrative + deferred boundaries). All mechanisms default-off / byte-identical. | [docs/tasks/WORKLOAD-MEMORY-SAFETY.md](tasks/WORKLOAD-MEMORY-SAFETY.md) |
| `CONE-DECOMPOSITION` | `done` | Code quality / maintainability — generator core readability | complete — `.1` design + `.2`–`.7` extractions (`cone/{snapshot,semantic,primitives,terminals,flops,motifs}.rs`). `src/gen/cone.rs` 5551→2446 lines (56% reduction); root = recursion strategy, six cohesive submodules re-exported via `pub(crate) use <sub>::*`. Every leaf byte-identical (snapshots 6/6 throughout; full suite green at `.2` + `.7`). | [docs/tasks/CONE-DECOMPOSITION.md](tasks/CONE-DECOMPOSITION.md) |
| `AGENT-INTROSPECTION-MCP` | `done` | Capability — agent-drivable introspection + MCP interface | **closed `2026-06-15`** — all leaves `.1`–`.7` done. `.1` decision `0004`; `.2` `docs/AGENT_INTROSPECTION_SCHEMA.md` (versioned envelope, zero new computed truth); `.3` `src/introspect/` + default-off `--introspect` flag (DUT byte-identical); `.4` `src/mcp/` + `anvil-mcp` bin (stdio JSON-RPC; generate/introspect/dump_config + resources over a content-addressed cache); `.5` (`.5.1`/`.5.2`/`.5.3`) the controlled `validate`/`minimize` tools (`src/downstream/`: sandboxed + fixed `verilator`/`yosys`/`iverilog` allow-list + ram-guard + `anvil://audit/log`; minimize = deterministic coordinate-descent, seed fixed, budget-bounded; e2e clean vs real Verilator 5.046 + Yosys 0.64); `.6` the five agent-workflow prompts as first-class MCP prompts (`prompts/list`/`prompts/get`); `.7` the user-facing closeout — mdBook chapter `book/src/agent-mcp.md` (Reference) + USER_GUIDE section + README CLI-truth/key-paths sync (`book_examples` gate 3/3). DUT byte-identical throughout (snapshots 6/6). Optional future breadth (coverage_gaps MCP tool, HTTP transport, non-DUT lanes over MCP) does not reopen the tree. | [docs/tasks/AGENT-INTROSPECTION-MCP.md](tasks/AGENT-INTROSPECTION-MCP.md) |
| `CAPABILITY-LANE-OWNERSHIP` | `done` | Workflow / capability-lane task-tree ownership | complete — `.1` registered the three owner-directed post-phase capability lanes before implementation resumed (`2026-06-15`) | [docs/tasks/CAPABILITY-LANE-OWNERSHIP.md](tasks/CAPABILITY-LANE-OWNERSHIP.md) |
| `AGENT-MCP-EXPANSION` | `done` | Capability — agent/MCP interface breadth | complete — `.1` decision `0005`; `.2` coverage_gaps tool; `.3` (`.3a`+`.3b`) non-DUT lanes over MCP; `.4` (`.4a` framing design + `.4b` hand-rolled `--http <addr>` transport, loopback default, no new dep, DUT byte-identical); `.5` book/USER_GUIDE/README closeout. Tree CLOSED `2026-06-15` | [docs/tasks/AGENT-MCP-EXPANSION.md](tasks/AGENT-MCP-EXPANSION.md) |
| `SIGNOFF-AUTOMATION-EXPANSION` | `active` | Quality — downstream signoff automation breadth | `.1` done (decision `0006`); `.2` (= `.2a` design + `.2b` impl) **done** — first richer-knob-sweep increment delivered: new `num_operator_gates_with_duplicate_operands` metric + `ScenarioSet::SignoffKnobSweep` + `--signoff-knob-sweep-gate` + 4 `saw_*` facts (operand/mux-arm duplication, array-packed aggregate, memory×fsm interplay), banked clean at `/tmp/anvil-signoff-knob-sweep-r1` (12 scenarios / 48 modules / `coverage_gaps=[]` / 48/0 Verilator + both Yosys). **No active frontier** — higher-ceiling future leaves preserved (decision `0006`); next lane per order = `IDENTITY-DEEPENING` | [docs/tasks/SIGNOFF-AUTOMATION-EXPANSION.md](tasks/SIGNOFF-AUTOMATION-EXPANSION.md) |
| `IDENTITY-DEEPENING` | `active` | NodeId as identity / full-factorization deepening | `.1` **done** (decision `0007`). `.2` **done** (`.2a` design + `.2b` impl): the opt-in `merge_bisimilar_flops` bounded bisimulation pass is live — merges mutually-recursive / swapped-feedback registers via greatest-fixpoint partition refinement over the existing 12-bit/128-node/131072-work proof, default-off / byte-identical, resetless flops excluded; new `Config`/`Module` `bisimulation_flop_merge` knob + `Metrics::bisimulation_flops_merged`; shared `finalize_flop_merge` refactor keeps the exact pass byte-identical; introspection schema MINOR-bumped 1.0→1.1; banked downstream-clean (Verilator + both Yosys + Icarus). `.3` split into `.3a` (design — **done**, decision `0008`: bounded whole-leaf-module sequential equivalence via cross-module bisimulation beside `dedup_semantic_modules`) + `.3b` (impl). `.3b` split into `.3b.1` (design-detail — **done**: combined-module materialization so inputs unify by `(PortId,width)` for free, factored `bisimulation_partition` reuse, no-bijection coinduction soundness, pre-filter + union-find grouping) + `.3b.2` (impl). `.3b.2` split into `.3b.2a` (**done**: factored the byte-identical `bisimulation_partition` helper) + `.3b.2b` (cross-module feature — **done** `2026-06-16`). `.3b.2b` split into `.3b.2b.1` (**done**: `modules_sequentially_equivalent` via combined-module materialization + `dedup_sequential_modules` pass + default-off `hierarchy_sequential_module_dedup` knob + gated wire-in + rules-first gate; DUT byte-identical), `.3b.2b.2a` (**done**: shared `group_sequentially_equivalent_modules` helper + `DesignMetrics` `sequential_module_proof_signatures` + `num_sequentially_duplicate_module_pairs` + introspection schema 1.3→1.4 + downstream-clean bank), and `.3b.2b.2b` (**done**: book §9b + hierarchy.md + USER_GUIDE knob + ROADMAP gap 2 + KM card). `.3`/`.3b`/`.3b.2`/`.3b.2b` all **done**. **No current frontier** — the deeper module-equivalence boundaries (memory / FSM / wrapper / retimed-state) are named, not-started future leaves (open-ended, none retired). | [docs/tasks/IDENTITY-DEEPENING.md](tasks/IDENTITY-DEEPENING.md) |
| `SV-VERSION-TARGETING` | `done` | Capability / breadth — version-targeted SV emission | **CLOSED `2026-06-16`** — all leaves done; down-gating + up-opting + per-version downstream acceptance axis delivered; the first version-distinctive up-opt (the IEEE 1800-2023 `union soft` overlay) ships both as a generator capability (`.3b.2a`) and a repo-owned matrix gate (`.3b.2b`, banked `/tmp/anvil-sv-version-gate-upopt-r1`: 10 scenarios / 20 units / `coverage_gaps=[]` / Verilator 20/0 / Yosys 18/0); further up-opts are open-ended post-tree breadth (nothing retired). `.1` **done** (decision `0009`): opt-in `--sv-version <2012\|2017\|2023>` gate (`Config::sv_version`) — down-gating (never emit a construct newer than the target → standard-validity guarantee) + up-opting (deliberately emit a higher standard's distinctive synthesizable constructs, each proven downstream-clean in the matching tool mode), default byte-identical, rules-first, per-version downstream acceptance axis. Owner-directed highest-leverage lane (`2026-06-15`). `.2a` **done** (design detail). `.2b.1` **done**: `SvVersion {Sv2012<Sv2017<Sv2023}` enum + `Config::sv_version` + `--sv-version` CLI + versioned emitter entry points (`permits` down-gating bound) + introspection schema `1.1→1.2` + `tests/sv_version.rs`; default byte-identical. `.2b.2a` **done**: `run_verilator(_design)` `language: Option<&str>` selector (`--language 1800-20xx`; `None` = byte-identical) + `tests/sv_version_downstream.rs` (`#[ignore]`) banked clean. `.2b.2b` **done** (`2026-06-16`): repo-owned `tool_matrix --sv-version-gate` + `ScenarioSet::SvVersionSweep` (9 Interleaved scenarios = 3 targets × {comb leaf, seq leaf, recursive hierarchy design}) + per-version emit threading + matching-mode Verilator (`verilator_language_for`) + `saw_sv_version_*_targeted_acceptance` coverage facts + `MatrixReport.sv_version_gate`; banked clean `/tmp/anvil-sv-version-gate-r1` (9 scenarios / 18 units / `coverage_gaps=[]` / 18/0 Verilator + both Yosys). **`.2`/`.2b`/`.2b.2` all closed.** `.3` split into `.3a` (design — **done**, decision `0010`: first up-opt = a default-off heterogeneous-width packed `union soft` (IEEE 1800-2023 §7.3.1) gated on `sv_version >= Sv2023`, struct down-gate fallback ⇒ byte-identical; the installed tools don't enforce 1800-version acceptance so the teeth are LRM + construction-time down-gating + matching-mode acceptance; Yosys/Icarus recorded no-op) + `.3b` (impl). `.3b` split into `.3b.1` (design-detail — **done**) + `.3b.2` (impl). `.3b.2` split into `.3b.2a` (**done** — the live first up-opt: `Config::soft_union_slice_prob` + `src/ir/soft_union.rs` gen-time pass + the `permits(Sv2023)`-gated `emit/sv.rs` `union soft` overlay of a proper low-bits `Slice`; default-off / byte-identical, snapshots 6/6; banked Verilator `--language 1800-2023` clean, 159 overlays / 7 seeds, 2012 down-gate proven; Yosys/Icarus recorded no-op) + `.3b.2b` (**done** `2026-06-16`: the repo-owned matrix up-opt gate — a tenth `sv2023_soft_union_upopt` `--sv-version-gate` scenario, Verilator-only via `scenario_emits_soft_union_overlay`/`verilator_only`, `ModuleReport.emitted_soft_union_overlay` emission evidence, the `saw_sv_version_2023_soft_union_upopt` fact + a `!yosys.is_empty()` honesty guard; banked clean). **Tree CLOSED.** | [docs/tasks/SV-VERSION-TARGETING.md](tasks/SV-VERSION-TARGETING.md) |
| `STRUCTURED-EMISSION-EXPANSION` | `active` | Capability / breadth — richer structured emission | **Activated `2026-06-16` by explicit owner directive.** `.1` design **done** — decision `0012`: the first richer-structured surface is a default-off, valid-by-construction combinational `function automatic` emit-projection of an existing combinational cone (the `output_support` support-leaf boundary gives its parameter list), chosen over interface/modport (weak Yosys synth support) + nested generate (bigger blast radius); opt-in `function_emit_prob` (default `0.0`) ⇒ byte-identical; downstream gate = Verilator + both Yosys modes + Icarus. `.2a` design-detail **done**; `.2b.1` live surface **done** (`function_emit_prob` knob + `Module.function_emit_gates` + `src/ir/function_emit.rs` gen-time mark + two generator call-site rolls + `to_sv_with_modules` `<wire>__f` `function automatic` decl/positional-body/call rendering + 9 lib proofs; `Slice` excluded from the first cut — `-Wall UNUSEDSIGNAL` on a full-width param, still emitted inline, nothing retired; no schema bump; default-off / DUT byte-identical, snapshots 6/6; forced sweep clean across Verilator + both Yosys + Icarus, `/tmp/anvil-fe-r2/`). `.2b.2` pre-split (`2026-06-16`) → `.2b.2a` (metric + schema — **done**: `Metrics::num_emitted_combinational_functions` ⇒ introspection schema `1.7 → 1.8`; 468 lib tests / snapshots 6/6 / mdbook green) + `.2b.2b` (the `tool_matrix` gate — **done `2026-06-16`**: `--function-emit-gate` + `ScenarioSet::FunctionEmitSweep` + `build_function_emit_sweep_scenarios` [comb-only `function_emit_prob=1.0` × 3 strategies] + `ModuleReport.emitted_combinational_function` SV-text detection + `saw_combinational_function_emit` + early-return gap enforcement + 5 proofs; banked clean `/tmp/anvil-function-emit-gate-r1` [3 scenarios / 12 modules / 608 emitted functions / `coverage_gaps=[]` / `12/0` Verilator + both Yosys + Icarus compile]; default-off / DUT byte-identical, snapshots 6/6) + `.2b.2c` (book/knobs/USER_GUIDE/README/KM closeout — **done `2026-06-16`**: new `How It Works` chapter `book/src/structured-emission.md` [byte-verified seed-42 before/after; single-gate rule; `Slice`/structured exclusions; duplicate-operand positional params; combinational-only] + `function_emit_prob` knob entry in `book/src/knobs.md`/`USER_GUIDE.md`/README "Current CLI truth" [config-file-only knob] + KM how-to card `combinational-function-emit`; KM 36→37 facts/272→286 keys; `mdbook build` + `check_knowledge_map` + `check_memory_architecture` + `cargo test --test book_examples` 3/3 green; docs-only / DUT byte-identical). **The first structured surface (the combinational `function automatic` emit-projection) is delivered end-to-end — `.2b.2`/`.2b`/`.2` all close.** `.3` design **done `2026-06-16`** (decision `0013`, by owner steer *"structured emission: next surface"* → `generate`): the **second** surface is a default-off, valid-by-construction **`generate for` loop** emit-projection of an existing `{N{x}}` replication (index-regular by construction), over `task` [leading future, also clean for simple comb void tasks], `interface`/`modport` [weak Yosys synth], and constant-predicate `generate if` [dead untaken branch; frontend lane already has it]; empirically grounded clean across Verilator `-Wall` + both Yosys + Icarus; the DUT emitter has no `generate`/`genvar` today, the frontend lane has `generate if`. Rules-first / default-off `generate_loop_emit_prob` (proposed) ⇒ byte-identical; downstream gate `saw_generate_loop_emit`. ****`.4a` design-detail done `2026-06-16`** — grounded decision `0013` in the real emitter (`render_gate`'s `{N{x}}` `Concat` replication predicate `sv.rs:1159` is the index-regular source; `function_emit.rs`/`soft_union.rs` gen-time annotation is the mechanism) and resolved all five points: first-cut = a `{N{x}}` **1-bit-lane** replication `Concat` (excludes function-emit marks); gen-time `src/ir/generate_loop.rs` + `Module.generate_loop_gates`; `genvar`/`generate for` block + assign-loop `continue`; `Config::generate_loop_emit_prob` config-file-only default `0.0`; `tool_matrix --generate-loop-gate`/`saw_generate_loop_emit`. **`.4b.1` live surface done `2026-06-16`** — the second richer-structured emit surface goes live: `Config::generate_loop_emit_prob` (default `0.0`, config-file-only) + `Module.generate_loop_gates` + new `src/ir/generate_loop.rs` (`annotate_generate_loop_gates`, `{N{x}}` 1-bit-lane replication candidate excluding function-emit marks) + two guarded gen-time call-site rolls (after function_emit) + `to_sv_with_modules` `generate_loop_gate`/`render_generate_loop_block` (`genvar`/`generate for` block + assign-loop suppression) + 9 lib proofs; `gi = gi + 1` increment (`gi++` not retired); default-off / DUT byte-identical (snapshots 6/6, lib 477); forced sweep clean across Verilator `--lint-only` (`-Wall` Δ=0) + both Yosys + Icarus (`/tmp/anvil-gl-r1/`, 5 seeds). **`.4b.2a` metric + schema bump done `2026-06-16`** — `Metrics::num_emitted_generate_loops` (`= m.generate_loop_gates.len()`) surfaced in introspection `module_metrics` ⇒ `SCHEMA_VERSION` `1.8→1.9` (the metric bumps; the `.4b.1` knob rode the version); bumped all current-output schema refs (9 test assertions + schema doc + README + USER_GUIDE + 5 book example JSONs + the CODEBASE_ANALYSIS envelope line; historical landing attributions left intact); lib proof; default-off / DUT byte-identical (snapshots 6/6, lib 478); end-to-end introspect default `0` / forced `50`. **`.4`/`.4b`/`.4b.2` are `active` containers; **`.4b.2b` gate done `2026-06-16`** — the repo-owned `tool_matrix --generate-loop-gate` (`ScenarioSet::GenerateLoopSweep` + `build_generate_loop_sweep_scenarios` [comb-only `generate_loop_emit_prob=1.0` × 3 strategies] + `ModuleReport.emitted_generate_loop` + `saw_generate_loop_emit` + early-return gap enforcement + 5 proofs) banked clean `/tmp/anvil-generate-loop-gate-r1` (3 scenarios / 12 modules / 8 emitting a loop / `coverage_gaps=[]` / `12/0` Verilator + both Yosys + Icarus); README/USER_GUIDE/CODEBASE_ANALYSIS gate entries. `.4b.2` closes. **`.4`/`.4b` are `active` containers; **`.4b.3` user-facing closeout done `2026-06-16`** — `book/src/structured-emission.md` gains a `## The second surface: a generate for loop` section (byte-verified seed-12 before/after; the `{N{x}}` 1-bit-lane rule; wider-lane exclusion; `gi = gi + 1`) + the `generate_loop_emit_prob` knob entry in `book/src/knobs.md` / `USER_GUIDE.md` / README + the KM how-to card `generate-loop-emit` (KM 38→39 facts / 296→309 keys); `mdbook build` + `check_knowledge_map` + `check_memory_architecture` + `book_examples` 3/3 green. **`.4b.3` / `.4b` / `.4` all close — the second structured surface (the `generate for` loop emit-projection) is delivered end-to-end.** **`.5` design-leaf done `2026-06-16`** (decision `0014`, autonomously selected at the no-frontier boundary per the owner *"pick any tree and roll"* directive): the **THIRD** structured surface is a default-off, valid-by-construction combinational **`task automatic`** emit-projection of a single combinational gate (the decision `0012` single-gate parallel, but a procedural `task` with an `output` arg called from `always_comb`), over nested `generate` + `interface`/`modport`; empirically grounded clean across Verilator `-Wall` + both Yosys + Icarus (both the direct-output and the output-var passthrough forms); rules-first / default-off `task_emit_prob` / `saw_combinational_task_emit` gate. **`.6a` design-detail done `2026-06-16`** — grounded decision `0014` in the real emitter (the `to_sv_with_modules` function-decl + generate-block sections as the template; **`render_gate_function_body` reused verbatim** as the task body) and resolved all five points: the output-var + passthrough-`assign` integration; gen-time `src/ir/task_emit.rs` + `Module.task_emit_gates` (the function-emit predicate plus exclusion of the sibling projections, run after generate_loop); the `task automatic` decl + `always_comb` call + assign-RHS swap; `Config::task_emit_prob` config-file-only default `0.0`; `num_emitted_combinational_tasks` metric (schema `1.9→1.10`) + `tool_matrix --task-emit-gate` / `saw_combinational_task_emit`. **`.6`/`.6b` are `active` containers; **`.6b.1` live surface done `2026-06-16`** — `Config::task_emit_prob` (default `0.0`, config-file-only) + `Module.task_emit_gates` + new `src/ir/task_emit.rs` (`annotate_task_emit_gates`, the function-emit candidate predicate plus exclusion of the three sibling projections) + two guarded gen-time call-site rolls (after generate_loop) + the `to_sv_with_modules` `task_emit_gate` accessor + `render_gate_task_decl` (body via the reused `render_gate_function_body`) + `render_gate_task_call` (the `logic <wire>__tv` var + the `always_comb` call) + the gate-assign-loop passthrough + 11 lib proofs; output-var + passthrough integration; no schema bump (the `.6b.2` metric bumps `1.9→1.10`); default-off / DUT byte-identical (snapshots 6/6, lib 489); forced `task_emit_prob=1.0` sweep clean across Verilator `--lint-only` (`-Wall` Δ=0) + both Yosys + Icarus (`/tmp/anvil-te-r1/`, 5 seeds). **`.6b.3` user-docs closeout done `2026-06-16`** — the THIRD structured surface (the combinational `task automatic`, decision `0014`) is delivered end-to-end; `.6b.3` / `.6b` / `.6` all close. Docs-only: a `## The third surface: a combinational task automatic` section in `book/src/structured-emission.md` (byte-verified seed-1 before/after) + the `task_emit_prob` knob entry in `book/src/knobs.md` / `USER_GUIDE.md` / README + the KM how-to card `combinational-task-emit` (KM 40→41 facts / 318→331 keys); `mdbook build` + `check_knowledge_map` + `check_memory_architecture` + `book_examples` 3/3 green. Full surface: `.6a` design + `.6b.1` live (`task_emit_prob` + `src/ir/task_emit.rs` + emitter) + `.6b.2a` metric/schema-`1.10` + `.6b.2b` the `tool_matrix --task-emit-gate` (banked `/tmp/anvil-task-emit-gate-r1`, 12/12 tasks, `12/0` all tools). **`.7` design-leaf done `2026-06-17`** (decision `0015`, autonomously selected at the no-frontier boundary per `feedback_pick_and_roll_at_no_frontier`): the **FOURTH** structured surface is a default-off, valid-by-construction **wider-lane `generate for` part-select** — a behaviour-preserving broadening of the second surface from the 1-bit lane to `LW >= 1` (a `{N{x}}` replication whose lane is `LW` bits renders `assign <wire>[gi*LW +: LW] = <x>;`; `LW==1` stays the byte-identical `[gi]` form). Chosen over `interface`/`modport` (**empirically disqualified** this session: Icarus syntax-fails the modport port + both Yosys modes warn on the implicit interface-member decl) and nested/multi-level `generate` (clean but bigger blast radius + no routine by-construction 2D source); a fresh probe (Verilator 5.046 `-Wall` + Yosys 0.64 both modes + Icarus 13.0) accepts the wider-lane part-select warning-clean + iverilog-sim-proven `== {N{x}}`. **Reuses** the existing `generate_loop_emit_prob` knob + `num_emitted_generate_loops` metric — no new knob / no new metric / no schema bump. Split `.7` (design done) + `.8` (impl, pre-split `.8a` design-detail + `.8b` impl) + future `.9+` (nested/multi-level `generate`, `interface`/`modport`, richer tasks). **`.8b` impl done `2026-06-17` — the FOURTH structured surface (the wider-lane `generate for` part-select) is delivered end-to-end; the lane returns to no-current-frontier (open-ended).** Two surgical edits: `src/ir/generate_loop.rs` `gate_qualifies` relaxed to `LW >= 1` / `width == N*LW`, and `src/emit/sv.rs` `render_generate_loop_block` branches `LW==1` (verbatim `[gi]`, byte-identical) vs `LW>1` (`[gi*LW +: LW]`), with `generate_loop_gate`'s defensive re-check mirrored. 4 lib proofs + book §"The fourth surface" (byte-verified seed-74 before/after) + knobs/USER_GUIDE/README/CODEBASE_ANALYSIS/KM closeout. **Reuses** `generate_loop_emit_prob` + `num_emitted_generate_loops` — no new knob / no new metric / no schema bump. `cargo test --lib` 493 + snapshots 6/6 byte-identical; a per-seed ON-vs-OFF downstream sweep (8 seeds) emits 9 wider-lane part-selects with Verilator `-Wall` Δ=0 + Yosys both + Icarus, and `--generate-loop-gate` stays regression-clean (12/0). `.8a` (prior) resolved the design-detail + proved corpus-liveness (20 multi-bit-lane replications / 300-module sweep). FOUR structured surfaces delivered end-to-end (`function automatic` `.1`+`.2`, `generate for` loop `.3`+`.4`, `task automatic` `.5`+`.6`, wider-lane part-select `.7`+`.8`). **`.9` design-leaf done `2026-06-17`** (decision `0016`, autonomously selected at the no-frontier boundary per `feedback_pick_and_roll_at_no_frontier`): picked the **FIFTH** structured surface — a default-off, valid-by-construction **multi-gate-cone `function automatic`** (deepen the first surface from a single gate to a whole combinational cone: params = the `output_support` support leaves, body = topo-ordered function-local `logic` temps for the interior gates, return = root; behaviour-identical to the inline per-gate chain). Chosen on the by-construction-source axis (decision `0015`): any cone with `>= 2` interior gates qualifies (pervasive), vs the multi-output task's policy-laden co-supported-sink source + nested generate's absent 2D source. Fresh probe (`/tmp/anvil-se9-probe/`, Verilator 5.046 `-Wall` + Yosys 0.64 both modes + Icarus 13.0): zero `%Warning` across all tools + iverilog-sim-proven `==` the inline cone (4000 vec); the multi-output `task` is also clean+sim-equiv (deferred runner-up); nested `generate` clean-but-no-source; `interface`/`modport` stays disqualified (`0015`). Its **own** opt-in `cone_function_emit_prob` knob (the shipped single-gate `function_emit_prob` surface stays byte-identical; reusing it rejected) + a `num_emitted_cone_functions` metric (schema `1.10 → 1.11` at impl) + a `--cone-function-gate` / `saw_cone_function_emit` gate. Split `.9` (design done) + `.10` (impl, pre-split `.10a` design-detail + `.10b` impl) + future `.11+` (multi-output task, nested/multi-level `generate`, `interface`/`modport`). **Current frontier: `.10a`.** None retired | [docs/tasks/STRUCTURED-EMISSION-EXPANSION.md](tasks/STRUCTURED-EMISSION-EXPANSION.md) |
| `SEMANTIC-INTROSPECTION-EXPANSION` | `active` | Capability — deeper introspection surface | **Activated `2026-06-16` by explicit owner directive** (deep semantic introspection first-class + everything MCP-queryable via a top-notch API). `.1` design **done** (decision `0011`): a first-class, versioned, MCP-queryable, **SCHEMA-DERIVED derived-relation** API (a `DerivedAnalysis` introspection section, schema `1.3`, + a pure MCP `analyze` tool) answering *what depends on what* by pure IR-graph traversal — relations, **not** behaviour (the `0004` no-shadow-simulator / structure-first boundary is the permanent ceiling). First query = the output **support cone**. `.2` split into `.2a` (design-detail — **done**: the `DerivedAnalysis`/`SupportCone` shape in a pure `src/introspect/analyze.rs`, `output_support` kind + name-`target`, default-introspect-stays-lean + cone-stops-at-instance-boundary, schema `1.2→1.3`) + `.2b` (impl). `.2b` split into `.2b.1` (**done**: the pure `src/introspect/analyze.rs` support-cone analysis core) + `.2b.2` (**done**: schema `1.2→1.3` + the `DerivedAnalysisDocument` + the pure MCP `analyze` tool + dispatch/`tools/list`/`anvil://artifact/<run_id>/analysis/<query>` resource + book/USER_GUIDE/schema-doc + KM; unknown query/target ⇒ `-32602`; DUT byte-identical, snapshots 6/6). **`.2`/`.2b` done — the first query (output support cone) is delivered end-to-end.** Two queries delivered (`output_support` `.1`/`.2` + `input_reach` `.3`, schema `1.5`, e2e `anvil-mcp` clean). **`.4` (`flop_reset_provenance`, per-flop reset/data provenance) done** (`.4a`/`.4b.1`/`.4b.2`): the MCP `analyze` tool now answers `query=flop_reset_provenance` (a `FlopProvenance` per flop: reset kind/value, zero-vs-hold default, mux kind/arms, has_d) by projecting `Module.flops`; a third `flop_provenance` vec keeps prior docs byte-identical; schema `1.5 → 1.6`; e2e `anvil-mcp` clean. **all four named query kinds from decision `0011` delivered** (`output_support` `.1`/`.2`, `input_reach` `.3`, `flop_reset_provenance` `.4`, `module_reachability` `.5`), schema `1.7`, DUT byte-identical. `.5` closed `2026-06-16` (`.5a` design + `.5b.1` pure core + `.5b.2` surface): the MCP `analyze` tool answers `query=module_reachability` (which modules in a design are reachable from `design.top` via the instance graph) — `cargo test --lib` 458/0/2, snapshots 6/6, book_examples 3/3, e2e `anvil-mcp` smoke clean. **No active frontier** — further derived-query kinds are open-ended breadth; none retired | [docs/tasks/SEMANTIC-INTROSPECTION-EXPANSION.md](tasks/SEMANTIC-INTROSPECTION-EXPANSION.md) |
| `LOCAL-REFERENCE-CACHE` | `active` | Workflow / tooling — local LRM grounding | `.1` **done `2026-06-16`** — gitignore `.cache/` + land the owner-provided IEEE 1800-2017/2023 SystemVerilog LRM Markdown (untracked) under `.cache/local-references/sv/{2017,2023}/` with a provenance README; recorded as the `reference_sv_lrm_local_cache` auto-memory so future sessions grep the LRM before legality claims. No code / no RTL change. No active frontier (future reference caches land as new `.N` leaves). | [docs/tasks/LOCAL-REFERENCE-CACHE.md](tasks/LOCAL-REFERENCE-CACHE.md) |

## Directory Layout

```text
docs/TASK_TREE.md
docs/TASK_TREE_README.md
docs/tasks/
  TEMPLATE.md
  <TREE>.md
```

`docs/TASK_TREE.md` is the workflow and active-tree index.
Each top-level task owns one file in `docs/tasks/`.
`docs/tasks/TEMPLATE.md` is copied when creating a new top-level tree.

## Definitions

- Task tree: the recursive decomposition of one top-level task.
- Node: one item in that tree.
- Container node: a node with children. It is not directly executable.
- Leaf node: a node with no children. It is the only unit PNT may implement.
- Current frontier: the ordered set of leaf nodes that are eligible to be
  picked next.
- Slice: one completed leaf task plus its tests, docs, live-doc updates, and
  commit workflow.
- Evidence: the validation output, changed-doc summary, and git commit subject
  that prove a leaf was completed.

## ID Rules

Each task tree has a stable top-level ID.

```text
<TREE>
<TREE>.1
<TREE>.1.1
<TREE>.1.1.1
```

Rules:

- `<TREE>` uses uppercase letters, digits, and hyphens.
- Child IDs append dot-separated positive integers.
- IDs are permanent once published.
- Never renumber closed nodes.
- If a new ordering is needed, add new IDs and mark old nodes `superseded` or
  `deferred` with a reason.
- A commit that completes a task-tree leaf must identify the leaf ID in the
  commit subject or in the first body line.

## Status Vocabulary

Use only these statuses.

| Status | Meaning |
| --- | --- |
| `proposed` | Captured but not yet accepted into the active tree. |
| `active` | The top-level tree is open, or a container has unfinished children. |
| `pending` | Ready to be selected once it reaches the current frontier. |
| `in_progress` | Currently being implemented in the worktree. |
| `blocked` | Cannot proceed without a named blocker and unblock condition. |
| `done` | Completed, validated, documented, and committed. |
| `deferred` | Deliberately postponed with an explicit consequence. |
| `superseded` | Replaced by another node, with the replacement ID named. |

## Required Task File Sections

Every top-level task file must contain:

- Metadata: tree ID, status, roadmap lane, created date, last updated date.
- Goal: the user-visible or project-visible outcome.
- Non-goals: what this tree deliberately does not try to solve.
- Acceptance criteria: concrete conditions that close the top-level task.
- Task tree: all known nodes, with status and short result intent.
- Current frontier: ordered leaf nodes that PNT may select next.
- Decisions: accepted technical decisions and their rationale.
- Open questions: unresolved questions that do not block the whole tree yet.
- Blockers: blockers with unblock conditions.
- Verification log: checks run for completed leaves.
- Commit log: leaf IDs mapped to completion commit subjects.
- Changelog: dated edits to the tree itself.

## Node Rules

Every node must be one of these two shapes.

Container node:

```text
- ID: <TREE>.<n>
  Status: active
  Goal: ...
  Children: <TREE>.<n>.1, <TREE>.<n>.2
```

Leaf node:

```text
- ID: <TREE>.<n>
  Status: pending
  Goal: ...
  Acceptance: ...
  Verification: pending
  Commit: pending
```

A node with children must not be marked `done` until every child is `done`,
`deferred`, or `superseded`, and every non-`done` child has a recorded reason.

## Current Frontier Rules

The current frontier is the only list PNT uses when selecting work from a task
tree.

Rules:

- The frontier contains only leaf nodes.
- The frontier is ordered by intended priority.
- A container never appears in the frontier.
- A blocked node stays out of the frontier until unblocked.
- When a leaf is split, remove that leaf from the frontier, mark it `active`,
  add children, and place the first executable child or children in the
  frontier.
- When a leaf completes, remove it from the frontier and add the next eligible
  leaf or leaves.

## PNT Selection Rules

When PNT is asked to continue and at least one active task tree exists:

1. Read `docs/TASK_TREE.md`.
2. Read the active task file named in the `Active Task Trees` table.
3. Pick the first eligible leaf in that file's `Current Frontier`.
4. Implement only that leaf.
5. If the leaf is too broad, split it before implementation and commit the
   tree update as the leaf's honest outcome.
6. Run the required validation for the leaf.
7. Update the task file, live docs, and roadmap if status changed.
8. Run the full commit workflow before selecting another leaf.

If several active trees exist, choose the first active tree in the table unless
the user names another tree or the roadmap status names a different immediate
lane.

When the user asks for PNT and **no** active task tree is appropriate (the
work is a linear `rN` coverage extension), continue on the `rN` convention —
do not invent a task tree just to satisfy this section.

## Splitting Rules

Split a node when any of these are true:

- It cannot be completed to signoff quality in one slice.
- It mixes design, implementation, diagnostics, tests, and docs in ways that
  can be reviewed independently.
- It hides an unresolved policy choice behind implementation wording.
- It would require touching unrelated ownership areas in one commit.
- It discovers a lower-level dependency that should be solved first.

Do not split merely to create vague placeholders. Every child must have a
clear goal and a way to verify completion.

## Completion Rules

A leaf is complete only when all of the following are true:

- Implementation or documentation work for that leaf is finished.
- Focused checks passed, and broader checks ran when warranted (see
  `COMMIT.md` for the full pre-commit checklist).
- The owning task file records the result, validation, and commit subject.
- `CHANGES.md`, `MEMORY.md`, and the other live docs listed in `COMMIT.md`
  are updated when the leaf changes project state.
- The commit workflow in `COMMIT.md` has completed.
- `git_message_brief.txt` has been cleared after commit.

Commit hashes are intentionally not required inside the same task-file update:
the final hash cannot be known until after the commit exists. The stable
join key is the leaf ID in the commit subject or first body line. Later status
refreshes may backfill hashes if useful.

## Blocker Rules

A blocked node must record:

- the exact blocker,
- why it blocks the node,
- the unblock condition,
- and the next task that should run instead, if any.

Do not leave a node as `blocked` only because it is large or unclear. Large or
unclear work should be split until a real blocker is visible.

## Relationship To Live Docs

The task tree is the detailed execution ledger.

- `ROADMAP.md` remains the canonical high-level phase status.
- `MEMORY.md` remains the recovery/handoff continuity log.
- `CHANGES.md` remains the chronological technical history.
- `DEVELOPMENT_NOTES.md` remains design rationale.
- `CODEBASE_ANALYSIS.md` remains the live workspace analysis.
- `USER_GUIDE.md` remains user-facing CLI/workflow reference.
- The mdBook (`book/src/*.md`) remains user-facing product/algorithm
  documentation.

Do not duplicate the whole task tree into those files. Link to the task tree
and summarize only the part that changes live project state. ANVIL's
`rN`-named slices stay recorded in `CHANGES.md` and `MEMORY.md` as before —
task-tree adoption does not change how `rN` slices land.

## Commit Workflow Tie-In

When a commit completes a task-tree leaf, `COMMIT.md`'s checklist still
applies in full. The only additional rule is:

- The commit subject or first body line must include the leaf ID
  (e.g., `HIERARCHY-AWARE-IDENTITY.1`).
- The owning `docs/tasks/<TREE>.md` file must be updated in the same commit
  with the leaf's new status, verification log entry, and commit-log entry.

For commits that are **not** task-tree-managed (linear `rN` slices, isolated
doc edits, workflow tweaks), no leaf ID is required.

## Copying This Workflow To Another Project

The detailed project-adoption checklist lives in
[docs/TASK_TREE_README.md](TASK_TREE_README.md).
