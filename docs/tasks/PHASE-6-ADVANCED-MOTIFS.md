# PHASE-6-ADVANCED-MOTIFS: Memories, FSMs, optional multi-clock

## Metadata

- Tree ID: `PHASE-6-ADVANCED-MOTIFS`
- Status: `active`
- Roadmap lane: Phase 6 — Advanced motifs
- Created: `2026-05-16`
- Last updated: `2026-05-18` (`.1` memory design landed; `.2` split into `.2.1`–`.2.4`; frontier → `.2.1`)
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
  Children: `PHASE-6-ADVANCED-MOTIFS.2.1`, `.2.2`, `.2.3`, `.2.4`

- ID: `PHASE-6-ADVANCED-MOTIFS.2.1`
  Status: `pending`
  Goal: `IR + emitter scaffold (architecture (M)). Additive Default-empty Module.memories: Vec<Memory> ({id, addr_width, data_width, kind: SinglePort|SimpleDualPort, write {we,addr,data NodeId src}, read {addr src}}); new opaque gate-graph leaf Node::MemRead{mem, addr,...} (sibling to FlopQ — never CSE'd, identity-by-instance); Config::memory_prob (f64, serde-default 0.0, probability-range validated); rules-first construction of a memory block + the emitter rendering the .1-validated inferrable template (logic [DW-1:0] mem_k [0:2**AW-1] + the synchronous write/read always_ff on the shared clk); validator: addr/data widths consistent, MemRead resolves to a declared memory. Default-off byte-identical.`
  Acceptance: `cargo fmt/clippy(-D warnings)/check/test green; focused proof: default-off byte-identical for fixed seeds across all ConstructionStrategy values; forced-on a memory module round-trips IR->validate->emit, SV declares the array + synchronous write/read; validate_design passes. No book/ change (book reconciliation is .2.4).`
  Verification: `pending`
  Commit: `pending`

- ID: `PHASE-6-ADVANCED-MOTIFS.2.2`
  Status: `pending`
  Goal: `Soundness + Yosys-inference proof. (a) Forced-on memory module: Yosys memory_collect reports >=1 $mem_v2 in BOTH repo modes (synth -noabc / synth;abc -fast) AND verilator --lint-only clean — the Phase 6 memory-inference contract, proven on real generated output (not a hand template). (b) Identity: a MemRead leaf is opaque to CSE / never merged; the memory array never enters the NodeId graph (regression-clean factorization).`
  Acceptance: `cargo gates green; Yosys-inference proof reproducible in both modes on generated output; default-off still byte-identical.`
  Verification: `pending`
  Commit: `pending`

- ID: `PHASE-6-ADVANCED-MOTIFS.2.3`
  Status: `pending`
  Goal: `tool_matrix scenario + metrics + gap (no ROADMAP advance). New phase6_inferrable_memory scenario (dedup/phase5/5b-anchor shape so shape-coverage sets are unperturbed); DesignMetrics.num_memory_modules; CoverageSummary.saw_inferrable_memory_design set + merged + a compute_coverage_gaps arm; bin-test scenario/design counts updated (observed, not guessed) + exception-list entry; non-vacuity test (scenario projects >=1 memory).`
  Acceptance: `cargo fmt/clippy(-D warnings)/check/test green incl. tool_matrix phase4 bin tests; NO ROADMAP phase label change yet.`
  Verification: `pending`
  Commit: `pending`

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
| 1 | `PHASE-6-ADVANCED-MOTIFS.2.1` | `pending` | `.1` design done; `.2` split (Splitting Rules + r87 no-aspirational-claims) into `.2.1`–`.2.4`. `.2.1` lands the IR `Memory` block + opaque `MemRead` leaf + `memory_prob` knob + emitter inferrable-template + validator, default-off byte-identical — the reviewable scaffold before the Yosys-inference proof (`.2.2`), gate scenario (`.2.3`), and verified-gate ROADMAP advance (`.2.4`). |

## Decisions

- `2026-05-16`: Multi-clock CDC is held as an optional, possibly-deferred
  sub-objective (not yet a leaf) per its roadmap "optional, expensive"
  framing; it will be added as a leaf only if/when prioritised, with the
  single-clock invariant explicitly preserved until then.
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

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `PHASE-6-ADVANCED-MOTIFS.1` | `Docs: PHASE-6-ADVANCED-MOTIFS.1 inferrable-memory motif design` | Design-only; architecture (M) `Memory` block + `MemRead` leaf; empirical Yosys probe; 3 rejected alternatives. No code. |

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
