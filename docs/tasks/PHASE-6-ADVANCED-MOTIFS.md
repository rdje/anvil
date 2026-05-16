# PHASE-6-ADVANCED-MOTIFS: Memories, FSMs, optional multi-clock

## Metadata

- Tree ID: `PHASE-6-ADVANCED-MOTIFS`
- Status: `active`
- Roadmap lane: Phase 6 — Advanced motifs
- Created: `2026-05-16`
- Last updated: `2026-05-16`
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
  Children: `PHASE-6-ADVANCED-MOTIFS.1`, `PHASE-6-ADVANCED-MOTIFS.2`, `PHASE-6-ADVANCED-MOTIFS.3`

- ID: `PHASE-6-ADVANCED-MOTIFS.1`
  Status: `pending`
  Goal: `Design the inferrable-memory motif (IR/emit shape, single vs dual port, write/read patterns Yosys infers as $mem, knob surface, proof shape, rejected alternatives) in DEVELOPMENT_NOTES.md. Design-only.`
  Acceptance: `DEVELOPMENT_NOTES.md Phase 6 memory design entry with >=1 rejected alternative; mdbook clean; no code change.`
  Verification: `pending`
  Commit: `pending`

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
| 1 | `PHASE-6-ADVANCED-MOTIFS.1` | `pending` | Memory inference is the highest-value downstream-stress motif; design first. Independent of Phase 4/5. |

## Decisions

- `2026-05-16`: Multi-clock CDC is held as an optional, possibly-deferred
  sub-objective (not yet a leaf) per its roadmap "optional, expensive"
  framing; it will be added as a leaf only if/when prioritised, with the
  single-clock invariant explicitly preserved until then.

## Open Questions

- Which exact write/read templates Yosys reliably infers as `$mem`
  across both Yosys modes. Owner: `.1` design (empirical probe).

## Blockers

- None. Independent of Phase 4/5.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-05-16` | `PHASE-6-ADVANCED-MOTIFS.1` | `pending` | `pending` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `PHASE-6-ADVANCED-MOTIFS.1` | `pending` | `pending` |

## Changelog

- `2026-05-16`: Created task tree as part of the directive to task-tree
  every remaining roadmap phase.
