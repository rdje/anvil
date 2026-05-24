# MULTI-CLOCK-CDC: Multi-clock domains + CDC primitives for ANVIL

## Metadata

- Tree ID: `MULTI-CLOCK-CDC`
- Status: `active`
- Roadmap lane: Quality / Capability — relax the single-clock-domain invariant
- Created: `2026-05-24`
- Last updated: `2026-05-24`
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
  Status: `pending`
  Goal: `Implement the IR extension + 2-flop synchronizer construction rule + emitter changes per .1's design. Backward-compatible: existing tests + book-runnable contract stay byte-identical with --multi-clock-prob defaulting to 0.0 (the single-domain K=1 special case). Cargo-portable proofs cover the synchronizer construction rule on a hand-built two-domain Module + per-(domain, polarity) always_ff emission.`
  Acceptance: `cargo fmt/clippy(-D warnings)/check/test all green; Module.clock_domains + Flop.domain extensions landed (backward-compatible); 2-flop synchronizer construction rule lands in src/gen; emitter handles per-(domain, polarity) always_ff blocks; --multi-clock-prob knob exists; default 0.0 ⇒ byte-identical existing book/snapshot/lib tests; cargo-portable proofs of the synchronizer construction + emission shape.`
  Verification: `pending`
  Commit: `pending`

- ID: `MULTI-CLOCK-CDC.3`
  Status: `pending`
  Goal: `Wire the downstream-tool gate per .1: Verilator --cdc=metastable on emitted multi-clock modules; --multi-clock-prob > 0 scenarios in tool_matrix; saw_multi_clock_design + saw_cdc_2_flop_synchronizer coverage facts; --diff-sim agreement on a synchronised-output trace.`
  Acceptance: `tool_matrix gains a multi-clock scenario; coverage facts on CoverageSummary; --diff-sim shows cross-simulator byte-equality on the synchronised trace; cargo-portable proofs of the coverage facts; #[ignore]-gated end-to-end proof against real Verilator --cdc=metastable.`
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
| 1 | `MULTI-CLOCK-CDC.2` | `pending` (implement IR extension + 2-flop synchronizer rule + emitter) | **`.1` done (`2026-05-24`)** docs-only — `DEVELOPMENT_NOTES.md` "Multi-clock + CDC primitives design" records: 7-tier CDC primitive catalogue (Tier 1 = 2-flop synchronizer first cut; tiers 2-7 deferred); minimum-viable IR shape (`Module.clock_domains: Vec<ClockDomain>` + per-flop `Flop.domain: usize`; K=1 backward-compatible); by-construction synchronizer rule (rules-first generation — never generate-then-filter); Verilator `--cdc=metastable` downstream gate (Yosys `-cdc` rejected; custom oracle deferred to `.4`); `--diff-sim` agreement on synchronised-output trace; 6 rejected alternatives; `.2`-`.4` leaf shape; `--multi-clock-prob: f64` knob (default 0.0 for byte-identical backward compatibility). `cargo` unchanged-green vs base `cfd5a72`. Frontier → `.2` (implement). |

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

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `MULTI-CLOCK-CDC.1` | `Docs: MULTI-CLOCK-CDC.1 multi-clock + CDC primitives design — opens the only remaining named follow-up tree` | Research-only design; 7-tier CDC primitive catalogue; min-viable IR shape; by-construction rule; downstream-tool gate; 6 rejected alternatives; `.2`-`.4` leaf shape. No code. |

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
