# PHASE-6-ADVANCED-MOTIFS: Memories, FSMs, optional multi-clock

## Metadata

- Tree ID: `PHASE-6-ADVANCED-MOTIFS`
- Status: `active`
- Roadmap lane: Phase 6 — Advanced motifs
- Created: `2026-05-16`
- Last updated: `2026-05-18` (`.1` memory design landed; frontier → `.2`)
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
  Children: `PHASE-6-ADVANCED-MOTIFS.1` (done), `PHASE-6-ADVANCED-MOTIFS.2`, `PHASE-6-ADVANCED-MOTIFS.3`

- ID: `PHASE-6-ADVANCED-MOTIFS.1`
  Status: `done`
  Goal: `Design the inferrable-memory motif (IR/emit shape, single vs dual port, write/read patterns Yosys infers as $mem, knob surface, proof shape, rejected alternatives) in DEVELOPMENT_NOTES.md. Design-only.`
  Acceptance: `DEVELOPMENT_NOTES.md Phase 6 memory design entry with >=1 rejected alternative; mdbook clean; no code change.`
  Verification: `DEVELOPMENT_NOTES.md "Phase 6 inferrable-memory motif design (2026-05-18, PHASE-6-ADVANCED-MOTIFS.1)" entry landed: codebase-grounded (IR has no array/memory concept — scalar u32 Port/Node/Flop; Flop is the only stateful element; operators-vs-blocks doctrine → memory is a block). Empirical Yosys probe (resolves the Open Question): single-port sync RAM and simple dual-port templates both yield exactly 1 $mem_v2 under proc;opt;memory_collect, verilator --lint-only exit 0, and synth -noabc / synth;abc -fast both exit 0 with check -assert (clean in both repo Yosys modes). Chosen architecture (M): first-class Memory block (additive Vec<Memory> on Module, Default-empty) + opaque Node::MemRead leaf (sibling to FlopQ, never CSE'd) + emitter renders the validated inferrable template on the shared clk + opt-in Config::memory_prob serde-default 0.0. Three rejected alternatives: (A) flop-array+mux (not $mem-inferred — defeats the purpose), (B) emitter-only string template (not valid-by-construction), (C) generic unpacked-array datatype threaded through width arithmetic (massive invasive change; memory is a block not a datatype). Proof shape for .2 specified (default-off byte-identical; forced-on memory_collect ≥1 $mem_v2 both modes; matrix scenario+metric+gap+non-vacuity, no promotion until verified gate — Phase 5/5b .2.x decomposition). Doc-only; no code. mdbook build clean; cargo fmt clean; cargo test unchanged-green (no src/tests touched since Phase 5b .2.3 green run).`
  Commit: `Docs: PHASE-6-ADVANCED-MOTIFS.1 inferrable-memory motif design`

- ID: `PHASE-6-ADVANCED-MOTIFS.2`
  Status: `pending`
  Goal: `Implement the inferrable-memory motif per .1, opt-in, with a matrix scenario and a Yosys memory-inference proof.`
  Acceptance: `Memory designs downstream-clean; Yosys infers memory; opt-in default preserves current output.`
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
| 1 | `PHASE-6-ADVANCED-MOTIFS.2` | `pending` | `.1` design landed (architecture (M) first-class `Memory` block + opaque `MemRead` leaf; empirical Yosys probe confirms single-port + simple-dual-port templates are `$mem_v2`-inferred clean in both modes). `.2` implements it opt-in with a Yosys memory-inference proof; expected to split into `.2.x` (scaffold / soundness / matrix scenario / verified-gate promote) per the Phase 5/5b precedent + r87 no-aspirational-claims. |

## Decisions

- `2026-05-16`: Multi-clock CDC is held as an optional, possibly-deferred
  sub-objective (not yet a leaf) per its roadmap "optional, expensive"
  framing; it will be added as a leaf only if/when prioritised, with the
  single-clock invariant explicitly preserved until then.

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
