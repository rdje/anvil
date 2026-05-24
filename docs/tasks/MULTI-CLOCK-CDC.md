# MULTI-CLOCK-CDC: Multi-clock domains + CDC primitives for ANVIL

## Metadata

- Tree ID: `MULTI-CLOCK-CDC`
- Status: `active`
- Roadmap lane: Quality / Capability — relax the single-clock-domain invariant
- Created: `2026-05-24`
- Last updated: `2026-05-24` (**`.3a` done** — `src/gen/multi_clock.rs` `construct_2flop_synchronizer` primitive landed with 5 cargo-portable proofs incl. emit-shape integration; K=1 byte-identical preserved; lib 251 → 256)
- Owner: repo-local workflow

## Goal

Relax ANVIL's single-clock-domain invariant — currently load-bearing
across every emit, gen, and IR construction path
(`book/src/sequential.md` "Synchronous-design discipline":
"Every module is fully synchronous to a single clock domain: one
`clk` (posedge), one `rst_n` (async, active-low)"; enforced
structurally by `Module.clock`/`Module.reset` being single reserved
slots, with no per-flop clock or reset polarity field) — so a
generated DUT may declare multiple clock domains and emit
**by-construction-correct CDC primitives** for signals crossing
between them. The valid-by-construction discipline must extend
across the relaxed invariant: every multi-clock module that ANVIL
emits must pass Verilator's `--cdc=metastable` check (or whatever
downstream-tool CDC check the matrix wires in) and must elaborate
+ synthesise + simulate cleanly on iverilog + verilator + yosys
without false-divergence in `--diff-sim`.

This is the explicitly-optional deferral named in `PHASE-6-
ADVANCED-MOTIFS`'s closing notes ("multi-clock CDC remains the
explicitly-optional, separately-prioritised deferral") and in
`book/src/sequential.md`'s synchronous-discipline section
("Multi-clock and CDC-safe handshakes are deferred to a much
later phase").

## Non-Goals

- **Not** asynchronous (non-clocked) logic. Every D-input is still
  flop-clocked; ANVIL does not generate latches or pure-async
  state machines. That stays the synchronous-design discipline's
  invariant.
- **Not** clock-gating motifs. Clock-gating is a power-optimisation
  concern; ANVIL's stance is "emit always-on flops; let downstream
  tools insert ICG". Tracked separately if ever opened.
- **Not** dynamic frequency / dynamic clock ratios. The clock
  generator inside the testbench may use any fixed ratio between
  domains, but the IR records a fixed declared frequency (or just
  a domain-name tag) per port, not a runtime-dynamic frequency.
- **Not** physical layout / DCT / fanout balancing concerns. ANVIL
  emits SV; downstream STA + P&R own those.
- **Not** commercial CDC checkers (SpyGlass, Conformal CDC). Open-
  source-first per `DIFFERENTIAL-SIMULATION.1`'s rejected-
  alternatives entry. Commercial parity is a separate optional
  deferral.

## Acceptance Criteria

- ANVIL can generate a module with N≥2 declared clock domains
  (e.g., `clk_a` + `rst_n_a` + `clk_b` + `rst_n_b`) whose
  inter-domain signals are wrapped in a chosen CDC primitive
  (2-flop synchronizer at minimum) by construction — the
  generator never emits a multi-clock flop-to-flop path without
  a synchronizer.
- A new `--multi-clock-prob` knob controls the per-module roll;
  when zero (the default for backward compatibility), every
  module is single-clock as today, and the existing tests +
  book-runnable contract are byte-identical.
- The IR is extended (per-flop clock + per-flop reset) without
  breaking the by-construction invariant: there's still one
  `always_ff @(posedge clk_X or negedge rst_n_X)` per
  (domain, polarity) tuple, never per-flop blocks.
- Verilator `--cdc=metastable` (or the project-chosen downstream
  CDC check) on every emitted multi-clock module exits clean.
- `tool_matrix --diff-sim` on a multi-clock scenario shows
  cross-simulator agreement on the post-reset post-handshake
  output trace (with stimulus synchronised to the receive domain).
- `saw_multi_clock_design` + `saw_cdc_2_flop_synchronizer`
  coverage facts on `CoverageSummary`.
- README + USER_GUIDE + `book/src/sequential.md` (which already
  says "Multi-clock … deferred to a much later phase") describe
  the new contract.

## Task Tree

- ID: `MULTI-CLOCK-CDC`
  Status: `active`
  Goal: `Relax single-clock-domain invariant + emit by-construction-correct CDC primitives.`
  Children: `MULTI-CLOCK-CDC.1`

- ID: `MULTI-CLOCK-CDC.1`
  Status: `done`
  Goal: `Research-only design entry (DEVELOPMENT_NOTES.md): catalogue the CDC primitives ANVIL must emit (2-flop synchronizer for 1-bit; deferred: async FIFO, gray-code pointer, handshake, pulse synchronizer, reset synchronizer); the minimum-viable IR extension (per-flop clock + per-flop reset slots; multi-domain Module shape); the by-construction rule that forbids multi-clock flop-to-flop paths without a registered synchronizer; the downstream-tool gate (Verilator --cdc=metastable + iverilog/verilator semantic agreement on a synchronised stimulus); rejected alternatives (single-flop synchronizer, clock-gating, dynamic frequency, latches); the .2/.3/.4 leaf shape. Docs-only, no code. Establishes the design before any IR/gen/emit change is touched.`
  Acceptance: `DEVELOPMENT_NOTES.md "Multi-clock + CDC primitives design (YYYY-MM-DD, MULTI-CLOCK-CDC.1)" entry landed: CDC primitive catalogue with first-cut scope (2-flop synchronizer for 1-bit signals; deferred: N-flop / handshake / FIFO / pulse / reset sync); minimum-viable IR shape (per-flop clock + per-flop reset slots, multi-domain Module); by-construction rule preventing unsynchronised multi-clock paths; downstream-tool gate; >=3 rejected alternatives; .2-.4 leaf shape. No code; cargo unchanged-green.`
  Verification: `DEVELOPMENT_NOTES.md "Multi-clock + CDC primitives design (2026-05-24, MULTI-CLOCK-CDC.1)" entry landed at the top of the "Design notes" section. Records: (1) CDC primitive catalogue with 7 tiers — Tier 1 (2-flop synchronizer for 1-bit signals) is the first-cut scope; tiers 2-7 (N-flop / async FIFO / gray-code pointer / req-ack handshake / pulse synchronizer / reset synchronizer) are explicitly deferred either to follow-up leaves (.5/.6/.7/.4) or to their own task trees (FIFO, gray code); (2) minimum-viable IR shape — Module.clock_domains: Vec<ClockDomain> (single-domain K=1 special case keeps Module.clock/Module.reset accessors backward-compatible) + per-flop Flop.domain: usize tag; emitter generates one always_ff per (domain, polarity) tuple — the Phase-1 doctrine "one always_ff per module" generalises to "one always_ff per domain" for K≥2; (3) by-construction rule (new structural rule for multi-clock) — when generator emits a flop in domain B whose D-cone references a domain-A flop output, the cone is rewritten to dereference a 2-flop synchronizer in B at *construction time*; never generate-then-filter (feedback_rules_first_generation.md); (4) downstream-tool gate — Verilator --cdc=metastable first-cut; Yosys -cdc rejected (doesn't exist in stable 0.64); custom oracle deferred to .4; (5) cross-simulator agreement via existing --diff-sim, sampling only synchronised post-handshake outputs to avoid the metastability-trace-divergence glass-jaw; (6) 6 rejected alternatives (single-flop synchronizer; clock-gating-instead-of-multi-clock; latches; async-FIFO as min-viable; generate-then-filter; dynamic frequency); (7) .2-.4 leaf shape; (8) knob shape (--multi-clock-prob: f64 per-module roll, defaults 0.0 for byte-identical backward compatibility). Docs-only; no code change (diff = DEVELOPMENT_NOTES.md + docs/tasks/MULTI-CLOCK-CDC.md + docs/TASK_TREE.md + CHANGES.md only). cargo unchanged-green vs base cfd5a72 (no src/tests touched).`
  Commit: `Docs: MULTI-CLOCK-CDC.1 multi-clock + CDC primitives design — opens the only remaining named follow-up tree`

- ID: `MULTI-CLOCK-CDC.2`
  Status: `done`
  Goal: `Implement the IR extension + emitter changes per .1's design. Backward-compatible: existing tests + book-runnable contract stay byte-identical with the K=1 special case (empty clock_domains + empty flop_domains). Cargo-portable proofs cover the per-(domain, polarity) always_ff emission. The 2-flop synchronizer construction rule + --multi-clock-prob knob + matrix wiring move to .3 (the generator-side change is bigger; .2 establishes the IR + emit shell so .3 has a stable surface to build on).`
  Acceptance: `cargo fmt/clippy(-D warnings)/check/test all green; Module.clock_domains + Module.flop_domains extensions landed (backward-compatible); emitter handles per-(domain, polarity) always_ff blocks via effective_clock_domains accessor; default empty clock_domains ⇒ byte-identical existing book/snapshot/lib tests; cargo-portable proofs of the K=2 emission shape + K=1 default-synthesis + flop_domain lookup.`
  Verification: `IR extension landed in src/ir/types.rs: new ClockDomain { clk: PortId, rst_n: PortId, name: String } struct + Module.clock_domains: Vec<ClockDomain> + Module.flop_domains: BTreeMap<FlopId, u32>, all defaults empty for K=1 backward compatibility. New accessors: Module::flop_domain(flop_id) returns 0 when absent from map; Module::effective_clock_domains() returns clock_domains if non-empty else synthesises [ClockDomain{ clk, rst_n, name: "default" }] from Module.clock/reset slots, or empty Vec for pure-combinational modules. Emitter refactored in src/emit/sv.rs: the single-block always_ff emission for flops is replaced by a per-domain loop over effective_clock_domains, with flops grouped via Module::flop_domain. For K=1 (the default — clock_domains empty), effective_clock_domains synthesises one ClockDomain and every flop has flop_domain == 0, producing byte-identical output to pre-.2 ANVIL. For K≥2, one always_ff @(posedge clk_X or negedge rst_n_X) block per domain. 4 new cargo-portable proofs in src/emit/sv.rs::tests: effective_clock_domains_synthesises_single_default_from_k1_module / flop_domain_defaults_to_zero_when_flop_domains_empty / emits_one_always_ff_block_per_clock_domain_when_k_equals_two (hand-built K=2 Module with two domains + two flops, asserts both always_ff blocks emitted + per-domain reset guards + correct flop-to-block grouping) / flop_domain_lookup_honors_populated_flop_domains_map. cargo fmt --all/clippy --all-targets -- -D warnings/check --all-targets all clean. cargo test --lib 251 passed (was 247 + 4 new). All other suites unchanged: bin tool_matrix 37+1 ignored, snapshots 6 (BYTE-IDENTICAL — proves the K=1 emit path is unchanged), book_examples 3/3 in 85s (BYTE-IDENTICAL default-dut contract from Phase 9 preserved across the IR + emit refactor), microdesign_parity 15+1, frontend_parity 12+2, diff_sim 2+2 ignored. Backward compatibility holds end-to-end. Frontier → .3 (2-flop synchronizer construction rule + --multi-clock-prob knob + tool_matrix wiring + coverage facts).`
  Commit: `MULTI-CLOCK-CDC.2 IR extension + emitter per-(domain, polarity) always_ff — backward-compatible K=1 byte-identical`

- ID: `MULTI-CLOCK-CDC.3`
  Status: `active`
  Goal: `Wire the downstream-tool gate per .1: Verilator --cdc=metastable on emitted multi-clock modules; --multi-clock-prob > 0 scenarios in tool_matrix; saw_multi_clock_design + saw_cdc_2_flop_synchronizer coverage facts; --diff-sim agreement on a synchronised-output trace. Split per the proven Phase-7 .2c.2a/.2c.2b + DIFFERENTIAL-SIMULATION.3b.1/.3b.2 discipline into .3a (synchronizer construction primitive in isolation + cargo-portable proofs; no Generator integration — landed first because the generator-side integration is sensitive and risks breaking the byte-identical default-dut book-runnable contract from Phase 9) and .3b (Generator integration — --multi-clock-prob knob + per-module rolls + automatic synchronizer wrap at domain-crossing decision points + tool_matrix scenario + saw_multi_clock_design / saw_cdc_2_flop_synchronizer coverage facts + Verilator --cdc=metastable downstream-tool gate + #[ignore] end-to-end proof).`
  Children: `MULTI-CLOCK-CDC.3a` (done), `MULTI-CLOCK-CDC.3b` (pending)

- ID: `MULTI-CLOCK-CDC.3a`
  Status: `done`
  Goal: `Land the 2-flop synchronizer construction primitive in src/gen/multi_clock.rs as a standalone library function with cargo-portable proofs. The primitive constructs two new flops (both in dst_domain) chained D=src_q → flop1 → flop2 → synced_q; registers their domain in Module.flop_domains; inherits source width. No Generator integration; no knob; no scenario wiring — those are .3b. The split mirrors the proven .2 (IR shell, byte-identical) → .3 (runtime integration) discipline at one finer granularity: landing the primitive first gives .3b a stable library surface to build against without back-pressure on the construction algorithm.`
  Acceptance: `cargo fmt/clippy(-D warnings)/check/test all green; new src/gen/multi_clock.rs landed with pub fn construct_2flop_synchronizer(module, src_q, dst_domain) -> Option<SynchronizerChain>; SynchronizerChain { first_flop, second_flop, synced_q } shape; >=4 cargo-portable proofs covering allocation, D-to-Q chaining, width inheritance, defensive None-on-invalid-src_q, and an end-to-end emit-shape integration; src/gen/mod.rs declares pub mod multi_clock; lib unit count +5; K=1 byte-identical (snapshots + book_examples).`
  Verification: `New src/gen/multi_clock.rs (~250 lines including 5 cargo-portable proofs) carries: pub struct SynchronizerChain { pub first_flop: FlopId, pub second_flop: FlopId, pub synced_q: NodeId }; pub fn construct_2flop_synchronizer(module: &mut Module, src_q: NodeId, dst_domain: u32) -> Option<SynchronizerChain> — allocates two new Flop entries (both in dst_domain via Module.flop_domains), two new Node::FlopQ entries, chains D=src_q → first_flop → second_flop → synced_q, inherits width from src_q via Node::width(). Both synchronizer flops carry ResetKind::Async + FlopKind::ZeroDefault + FlopMux::None + reset_val=0 (the standard flop-synchronizer template). Returns None for an out-of-bounds src_q (defensive). src/gen/mod.rs declares pub mod multi_clock with the rules-first generation doctrine docstring. 5 cargo-portable proofs in src/gen/multi_clock::tests: construct_2flop_synchronizer_allocates_two_flops_in_target_domain (both new flops land in domain 1; source unchanged in domain 0); construct_2flop_synchronizer_chains_d_to_q (first_flop.D = src_q; second_flop.D = first_flop.Q — the chain rather than two independent flops); construct_2flop_synchronizer_inherits_source_width (width=4 src → both sync flops width=4 + synced_q FlopQ node width=4); construct_2flop_synchronizer_returns_none_for_invalid_src_q (defensive); synchronizer_emit_shape_in_two_domain_module (end-to-end .2-emitter-meets-.3a-primitive integration: hand-built K=2 module + sync chain + output drive; asserts exactly 2 always_ff blocks emitted, domain A has only source flop, domain B has both sync flops, output is driven by second-stage synced Q). cargo fmt --all/clippy --all-targets -- -D warnings/check --all-targets all clean. cargo test --lib 256 passed (was 251 + 5 new). All other suites unchanged: snapshots 6 BYTE-IDENTICAL, book_examples 3/3 in 72.71s BYTE-IDENTICAL (default-dut contract from Phase 9 preserved), tool_matrix 37+1, microdesign_parity 15+1, frontend_parity 12+2, diff_sim 2+2 ignored. Frontier → .3b (Generator integration + --multi-clock-prob knob + tool_matrix scenario + coverage facts + #[ignore] end-to-end gate).`
  Commit: `MULTI-CLOCK-CDC.3a 2-flop synchronizer construction primitive in src/gen/multi_clock.rs — landed in isolation with 5 cargo-portable proofs`

- ID: `MULTI-CLOCK-CDC.3b`
  Status: `pending`
  Goal: `Generator integration: --multi-clock-prob knob (Config field + CLI flag), per-module roll, automatic 2-flop synchronizer wrap at domain-crossing decision points (rules-first; never generate-then-filter), tool_matrix multi-clock scenario, saw_multi_clock_design + saw_cdc_2_flop_synchronizer coverage facts, Verilator --cdc=metastable downstream-tool gate, #[ignore] end-to-end proof. Backward-compatible: --multi-clock-prob defaults to 0.0 ⇒ byte-identical book/snapshot/lib tests. Closes MULTI-CLOCK-CDC.3.`
  Acceptance: `cargo fmt/clippy(-D warnings)/check/test all green; --multi-clock-prob CLI flag + Config field; Generator integration that constructs K≥2 modules with automatic synchronizer wrap at cross-domain decisions; tool_matrix gains a multi-clock scenario; CoverageSummary gains saw_multi_clock_design + saw_cdc_2_flop_synchronizer + merge_coverage / summarize_coverage updates; cargo-portable proofs of the knob roll + synchronizer-wrap behavior + coverage facts; #[ignore] end-to-end proof shells Verilator --cdc=metastable on a generated multi-clock DUT and asserts no CDC violations; default --multi-clock-prob 0.0 ⇒ byte-identical existing book/snapshot/lib/diff_sim/microdesign_parity/frontend_parity tests.`
  Verification: `pending`
  Commit: `pending`

- ID: `MULTI-CLOCK-CDC.4`
  Status: `pending`
  Goal: `Documentation: README + USER_GUIDE + book/src/sequential.md describe the new multi-clock contract. Remove the "Multi-clock and CDC-safe handshakes are deferred to a much later phase" caveat from book/src/sequential.md; replace with the as-built contract description. Add the --multi-clock-prob knob to USER_GUIDE; describe the 2-flop synchronizer by-construction rule in book.`
  Acceptance: `Docs describe the new contract; mdbook build clean; cargo test --test book_examples 3/3 green AFTER the docs land (any new runnable bash blocks carry skip sentinels as needed).`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `MULTI-CLOCK-CDC.3b` | `pending` (Generator integration — `--multi-clock-prob` knob + per-module rolls + synchronizer wrap + `tool_matrix` scenario + coverage facts + `#[ignore]` end-to-end gate) | **`.3a` done (`2026-05-24`)** — new `src/gen/multi_clock.rs` (~250 lines) carries `pub fn construct_2flop_synchronizer(module, src_q, dst_domain) -> Option<SynchronizerChain>` + `SynchronizerChain { first_flop, second_flop, synced_q }` shape. Both new flops land in `dst_domain` via `Module.flop_domains`; chain is D=src_q → first_flop → second_flop → synced_q; width inherited from src_q via `Node::width()`. 5 cargo-portable proofs (allocation / D-to-Q chaining / width inheritance / defensive None-on-invalid-src_q / end-to-end emit-shape integration showing 2 `always_ff` blocks with correct flop-to-block grouping). `cargo fmt`/clippy(-D warnings)/check all clean. lib 251 → 256 (+5). All other suites unchanged — snapshots + book_examples **byte-identical**. **`.3` split per the proven Phase-7 `.2c.2a`/`.2c.2b` + DIFFERENTIAL-SIMULATION.3b.1/.3b.2 discipline** — primitive in isolation first; Generator integration follows. **`.2` done (`2026-05-24`)** — IR extension landed in `src/ir/types.rs` (new `ClockDomain` struct + `Module.clock_domains: Vec<ClockDomain>` + `Module.flop_domains: BTreeMap<FlopId, u32>`, both defaults empty for backward compat); emitter refactored in `src/emit/sv.rs` to per-domain `always_ff` blocks via `Module::effective_clock_domains` + `Module::flop_domain` accessors. K=1 default (empty `clock_domains`): the accessor synthesises a single-element `ClockDomain { clk, rst_n, name: "default" }` from existing `Module.clock`/`reset`, and every flop has `flop_domain == 0` → **byte-identical** emit verified by `cargo test --test snapshots` 6/6 + `cargo test --test book_examples` 3/3 in 85s (default-`dut` contract preserved). K=2: hand-built proof in `emits_one_always_ff_block_per_clock_domain_when_k_equals_two` asserts two `always_ff @(posedge clk_X or negedge rst_n_X)` blocks with per-domain reset guards and correct flop-to-block grouping. 4 new lib unit proofs (lib 247 → 251). `cargo fmt`/clippy(-D warnings)/check all clean. **`.1` done (`2026-05-24`)** docs-only — `DEVELOPMENT_NOTES.md` "Multi-clock + CDC primitives design" records: 7-tier CDC primitive catalogue (Tier 1 = 2-flop synchronizer first cut; tiers 2-7 deferred); minimum-viable IR shape (`Module.clock_domains: Vec<ClockDomain>` + per-flop `Flop.domain: usize`; K=1 backward-compatible); by-construction synchronizer rule (rules-first generation — never generate-then-filter); Verilator `--cdc=metastable` downstream gate (Yosys `-cdc` rejected; custom oracle deferred to `.4`); `--diff-sim` agreement on synchronised-output trace; 6 rejected alternatives; `.2`-`.4` leaf shape; `--multi-clock-prob: f64` knob (default 0.0 for byte-identical backward compatibility). `cargo` unchanged-green vs base `cfd5a72`. Frontier → `.2` (implement). |

## Decisions

- `2026-05-24`: Opened as the only remaining named follow-up tree after the `DIFFERENTIAL-SIMULATION` quality lane closed `2026-05-24`. Per `feedback_no_self_pause_until_trees_closed.md` continuous PNT, and the user's explicit "PNT into the next activity until full and complete exhaustion." Per the established design-first discipline (Phase 7/8/9 + `DIFFERENTIAL-SIMULATION.2a`/`.3a`), `.1` is research-only.

## Open Questions

- **Coverage-axis interactions.** How does multi-clock interact
  with the Phase-4 hierarchy axis (does every child have its own
  domain set?) and the Phase-6 FSM axis (one FSM per domain?
  cross-domain FSM via handshake?). Owner: `MULTI-CLOCK-CDC.1`
  decision section.
- **Reset-synchronizer scope.** The synchronous-design discipline
  uses async-active-low reset (`rst_n`). With multiple
  domains, each domain needs its own reset; should the
  synchronizer-on-deassertion pattern be mandatory or can we
  rely on the testbench to provide already-synchronised resets?
  Owner: `MULTI-CLOCK-CDC.1`.
- **Knob shape.** Single `--multi-clock-prob` (per-module roll)
  vs `--num-clock-domains-min`/`--num-clock-domains-max` range
  vs explicit `--cdc-pattern-prob` per primitive. Owner:
  `MULTI-CLOCK-CDC.1`.
- **Downstream-tool gate choice.** `verilator --cdc=metastable`
  is one option; `yosys + read_verilog -cdc` is another. The
  matrix gate to wire in. Owner: `MULTI-CLOCK-CDC.1` /
  `MULTI-CLOCK-CDC.4`.

## Blockers

- None. All preconditions are met:
  - `PHASE-6-ADVANCED-MOTIFS` (sequential / FSM motifs)
    closed `2026-05-20`.
  - `DIFFERENTIAL-SIMULATION` (cross-simulator gate) closed
    `2026-05-24` — provides the multi-sim agreement gate that
    `.3`/`.4` will reuse.
  - The IR/emit/gen are in a stable state (all 9 numbered
    phases done).

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-05-24` | `MULTI-CLOCK-CDC.1` | `DEVELOPMENT_NOTES.md` "Multi-clock + CDC primitives design (2026-05-24, MULTI-CLOCK-CDC.1)" entry landed: 7-tier CDC primitive catalogue (Tier 1 = 2-flop synchronizer first cut; tiers 2-7 deferred to follow-up leaves or their own task trees); minimum-viable IR shape (`Module.clock_domains: Vec<ClockDomain>` + per-flop `Flop.domain: usize`; K=1 backward-compatible); by-construction synchronizer rule (rules-first generation per `feedback_rules_first_generation.md`); Verilator `--cdc=metastable` downstream gate (Yosys `-cdc` rejected — doesn't exist in stable 0.64; custom oracle deferred to `.4`); cross-simulator agreement via existing `--diff-sim`; 6 rejected alternatives; `.2`-`.4` leaf shape; `--multi-clock-prob: f64` knob with default 0.0 for byte-identical backward compatibility. Docs-only; no code change (diff = `DEVELOPMENT_NOTES.md` + `docs/tasks/MULTI-CLOCK-CDC.md` + `docs/TASK_TREE.md` + `CHANGES.md` only); `cargo` unchanged-green vs base `cfd5a72`. | Done. Frontier → `.2`. |
| `2026-05-24` | `MULTI-CLOCK-CDC.3a` | New `src/gen/multi_clock.rs` (~250 lines) carries `pub struct SynchronizerChain { first_flop, second_flop, synced_q }` + `pub fn construct_2flop_synchronizer(module, src_q, dst_domain) -> Option<SynchronizerChain>` — allocates two new flops in `dst_domain` via `Module.flop_domains`, chains D=src_q → first_flop → second_flop → synced_q, inherits width via `Node::width()`. `src/gen/mod.rs` declares `pub mod multi_clock` with rules-first-generation doctrine docstring. 5 cargo-portable proofs in `src/gen/multi_clock::tests`: allocation / D-to-Q chaining / width inheritance / defensive None on invalid src_q / end-to-end `synchronizer_emit_shape_in_two_domain_module` (hand-built K=2 module + sync chain + output drive → asserts exactly 2 `always_ff` blocks emitted, domain A has only source flop, domain B has both sync flops, output driven by second-stage synced Q). `cargo fmt --all`/`clippy --all-targets -- -D warnings`/`check --all-targets` all clean. `cargo test --lib` 256 passed (was 251 + 5 new). All other suites unchanged: snapshots 6 BYTE-IDENTICAL, book_examples 3/3 in 72.71s BYTE-IDENTICAL (default-`dut` contract from Phase 9 preserved), tool_matrix 37+1, microdesign_parity 15+1, frontend_parity 12+2, diff_sim 2+2 ignored. | Done; `.3` split per Phase-7 `.2c.2a/.2c.2b` discipline. Frontier → `.3b`. |
| `2026-05-24` | `MULTI-CLOCK-CDC.2` | IR extension landed in `src/ir/types.rs`: new `ClockDomain { clk: PortId, rst_n: PortId, name: String }` struct + `Module.clock_domains: Vec<ClockDomain>` + `Module.flop_domains: BTreeMap<FlopId, u32>` (both defaults empty for K=1 backward compatibility) + accessors `Module::flop_domain(id)` (returns 0 when absent) + `Module::effective_clock_domains()` (returns `clock_domains` if non-empty, else synthesises single-element default from `Module.clock`/`reset`). Emitter refactored in `src/emit/sv.rs` to per-domain `always_ff` loop via `effective_clock_domains` + `flop_domain` grouping. **K=1 default byte-identical**: snapshots 6/6 + book_examples 3/3 in 85s (default-`dut` contract preserved). **K=2 proven**: hand-built two-domain Module emits two `always_ff @(posedge clk_X or negedge rst_n_X)` blocks with correct per-domain flop grouping. 4 new lib unit proofs (`effective_clock_domains_synthesises_single_default_from_k1_module` / `flop_domain_defaults_to_zero_when_flop_domains_empty` / `emits_one_always_ff_block_per_clock_domain_when_k_equals_two` / `flop_domain_lookup_honors_populated_flop_domains_map`). `cargo fmt`/clippy(-D warnings)/check all clean. lib 247 → 251 (+4). All other suites unchanged: bin tool_matrix 37+1 ignored, microdesign_parity 15+1, frontend_parity 12+2, diff_sim 2+2 ignored. | Done. Frontier → `.3` (synchronizer construction rule + `--multi-clock-prob` knob + tool_matrix wiring + coverage facts). |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `MULTI-CLOCK-CDC.1` | `Docs: MULTI-CLOCK-CDC.1 multi-clock + CDC primitives design — opens the only remaining named follow-up tree` | Research-only design; 7-tier CDC primitive catalogue; min-viable IR shape; by-construction rule; downstream-tool gate; 6 rejected alternatives; `.2`-`.4` leaf shape. No code. |
| `MULTI-CLOCK-CDC.2` | `MULTI-CLOCK-CDC.2 IR extension + emitter per-(domain, polarity) always_ff — backward-compatible K=1 byte-identical` | IR: new `ClockDomain` struct + `Module.clock_domains` + `Module.flop_domains` (defaults empty) + accessors. Emitter: per-domain `always_ff` loop via `effective_clock_domains` + `flop_domain` grouping. K=1 byte-identical (snapshots + book_examples). K=2 proven (4 lib unit proofs). lib 247 → 251. |
| `MULTI-CLOCK-CDC.3a` | `MULTI-CLOCK-CDC.3a 2-flop synchronizer construction primitive in src/gen/multi_clock.rs — landed in isolation with 5 cargo-portable proofs` | New `src/gen/multi_clock.rs`: `pub fn construct_2flop_synchronizer` + `SynchronizerChain` shape. Both new flops in `dst_domain` via `Module.flop_domains`; D=src_q → first → second → synced_q chain; width inherited via `Node::width()`. 5 cargo-portable proofs incl. end-to-end emit-shape integration. lib 251 → 256. K=1 byte-identical (snapshots + book_examples). `.3` split — Generator integration is `.3b`. |

## Changelog

- `2026-05-24`: Created task tree. Opened as the only remaining
  named follow-up after `DIFFERENTIAL-SIMULATION` closed
  `2026-05-24`. Per `feedback_no_self_pause_until_trees_closed.md`
  continuous PNT and the user's "PNT into the next activity
  until full and complete exhaustion" directive. The
  single-clock-domain invariant was explicitly deferred from
  Phase 6 (`PHASE-6-ADVANCED-MOTIFS` closing notes) and from
  `book/src/sequential.md` ("Multi-clock and CDC-safe
  handshakes are deferred to a much later phase").
- `2026-05-24`: **`.3a` landed + `.3` split** — new `src/gen/multi_clock.rs` (~250 lines) carries the 2-flop synchronizer construction primitive (`pub fn construct_2flop_synchronizer` + `SynchronizerChain` shape) in isolation. Both new flops land in `dst_domain` via `Module.flop_domains`; the chain is D=src_q → first_flop → second_flop → synced_q; width inherited from src_q via `Node::width()`. Both synchronizer flops carry the standard template (`ResetKind::Async` + `FlopKind::ZeroDefault` + `FlopMux::None` + `reset_val=0`). 5 cargo-portable proofs incl. end-to-end `synchronizer_emit_shape_in_two_domain_module` that integrates `.2`'s emitter with `.3a`'s primitive — asserts exactly 2 `always_ff` blocks emitted with correct flop-to-block grouping. `src/gen/mod.rs` declares `pub mod multi_clock` with rules-first-generation doctrine docstring. `cargo fmt`/clippy(-D warnings)/check all clean. lib 251 → 256 (+5). All other suites unchanged — snapshots 6 BYTE-IDENTICAL, book_examples 3/3 in 72.71s BYTE-IDENTICAL (default-`dut` contract from Phase 9 preserved). `.3` split per the proven Phase-7 `.2c.2a`/`.2c.2b` + `DIFFERENTIAL-SIMULATION.3b.1`/`.3b.2` discipline — the primitive landed in isolation FIRST because the Generator-side integration is sensitive (risks the byte-identical book-runnable contract); `.3b` will wire the knob + per-module rolls + automatic synchronizer wrap + matrix scenario + coverage facts + `#[ignore]` end-to-end gate. Frontier → `.3b`.
- `2026-05-24`: **`.2` landed** — IR extension + emitter K≥2 support, backward-compatible K=1 byte-identical. New `ClockDomain { clk: PortId, rst_n: PortId, name: String }` struct in `src/ir/types.rs`; `Module.clock_domains: Vec<ClockDomain>` + `Module.flop_domains: BTreeMap<FlopId, u32>` (both defaults empty for K=1 backward compat); `Module::flop_domain(id)` and `Module::effective_clock_domains()` accessors. Emitter refactored to per-domain `always_ff` loop via the new accessors — for K=1 the synthesised single-element default produces byte-identical output (snapshots 6/6 + book_examples 3/3 in 85s prove the default-`dut` contract is preserved across the IR + emit refactor); for K=2 the hand-built proof asserts two `always_ff` blocks with per-domain reset guards + correct flop-to-block grouping. 4 new lib unit proofs (`effective_clock_domains_synthesises_single_default_from_k1_module` / `flop_domain_defaults_to_zero_when_flop_domains_empty` / `emits_one_always_ff_block_per_clock_domain_when_k_equals_two` / `flop_domain_lookup_honors_populated_flop_domains_map`). `cargo fmt`/clippy(-D warnings)/check all clean. lib 247 → 251 (+4). All other suites unchanged: bin tool_matrix 37+1 ignored, microdesign_parity 15+1, frontend_parity 12+2, diff_sim 2+2 ignored. The minimum-blast-radius design — store the flop-domain mapping as an OPTIONAL external `BTreeMap` on `Module` rather than adding a field to `Flop` — kept the change to 23 Flop construction sites at zero touches. Frontier → `.3` (2-flop synchronizer construction rule + `--multi-clock-prob` knob + `tool_matrix` wiring + coverage facts).
- `2026-05-24`: **`.1` design landed** (research-only, no code).
  `DEVELOPMENT_NOTES.md` "Multi-clock + CDC primitives design"
  records the 7-tier CDC primitive catalogue (Tier 1 = 2-flop
  synchronizer first cut; tiers 2-7 deferred); minimum-viable
  IR shape (`Module.clock_domains` + per-flop `Flop.domain`;
  K=1 backward-compatible); by-construction synchronizer rule
  (rules-first per `feedback_rules_first_generation.md` —
  never generate-then-filter); Verilator `--cdc=metastable`
  downstream gate (Yosys `-cdc` rejected; custom oracle
  deferred to `.4`); cross-simulator agreement via existing
  `--diff-sim`; 6 rejected alternatives; `.2`-`.4` leaf shape;
  `--multi-clock-prob: f64` knob with default 0.0 for
  byte-identical backward compatibility. Mirrors the proven
  Phase-7/8/9 + `DIFFERENTIAL-SIMULATION.2a`/`.3a` design-first
  discipline. Frontier → `.2` (implement IR extension + 2-flop
  synchronizer rule + emitter).
