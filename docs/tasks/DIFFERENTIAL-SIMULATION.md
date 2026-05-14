# DIFFERENTIAL-SIMULATION: Cross-simulator semantic-equivalence gate for emitted RTL

## Metadata

- Tree ID: `DIFFERENTIAL-SIMULATION`
- Status: `active`
- Roadmap lane: Quality — signoff-level downstream consistency
- Created: `2026-05-14`
- Last updated: `2026-05-14`
- Owner: repo-local workflow

## Goal

Add a downstream check that proves emitted RTL behaves identically across
multiple independent simulators (Verilator and Icarus iverilog at
minimum), not just that each simulator parses and synthesises the
output cleanly. For every shared input vector pattern, every observable
output must match bit-for-bit.

This raises ANVIL's downstream contract from "parses and synthesises
on the curated matrix" to "all open-source simulators we test against
agree on semantics", which is the actual signoff-quality bar.

## Non-Goals

- Functional correctness of the emitted module against some intended
  spec. By construction ANVIL outputs are functionally arbitrary —
  the goal is *cross-simulator agreement*, not *correctness*.
- Coverage of every commercial simulator. Open-source first
  (Verilator, iverilog). Commercial parity is an explicit deferral.
- Replacing the existing Phase 4 hierarchy gate. This is additive:
  a new axis on the matrix, not a substitute for parse/synth checks.

## Acceptance Criteria

- A new test harness drives the same `(generated SV file, random
  input-vector seed)` through Verilator simulation and iverilog
  simulation and asserts byte-equal output traces.
- A focused proof covers at least one combinational and one sequential
  design from a canonical `(seed, config)` set.
- The matrix gate gains a new opt-in mode (e.g.,
  `--phase4-hierarchy-gate --diff-sim`) that runs the differential
  check across a representative subset; the full matrix is too
  expensive but a curated subset is gate-feasible.
- Coverage fact `saw_design_with_cross_simulator_agreement` fires when
  the differential pass succeeds; the matrix records mismatches
  explicitly rather than silently.
- README + USER_GUIDE + book/src/* describe the new contract.

## Task Tree

- ID: `DIFFERENTIAL-SIMULATION`
  Status: `active`
  Goal: `Prove cross-simulator semantic equivalence for emitted RTL across at least two independent simulators.`
  Children: `DIFFERENTIAL-SIMULATION.1`, `DIFFERENTIAL-SIMULATION.2`, `DIFFERENTIAL-SIMULATION.3`, `DIFFERENTIAL-SIMULATION.4`

- ID: `DIFFERENTIAL-SIMULATION.1`
  Status: `pending`
  Goal: `Investigate and document: which open-source simulators (Verilator, iverilog, sv2v + downstream) can ingest ANVIL output today without configuration? Which subset of ANVIL's emitted SV is supported by each? Output: DEVELOPMENT_NOTES.md entry with a compatibility matrix and a recommended pair of simulators for the differential check.`
  Acceptance: `DEVELOPMENT_NOTES.md entry exists; the recommended simulator pair is named; rejected alternatives are recorded.`
  Verification: `pending`
  Commit: `pending`

- ID: `DIFFERENTIAL-SIMULATION.2`
  Status: `pending`
  Goal: `Build a single-design differential harness: given (generated SV, input-vector seed, simulation cycle count), drive the design through both simulators and return aligned output traces. Pure CLI utility; no integration with tool_matrix yet.`
  Acceptance: `A focused test calls the harness on a hand-picked (seed, config) leaf design, gets two output traces, and asserts they agree byte-for-byte.`
  Verification: `pending`
  Commit: `pending`

- ID: `DIFFERENTIAL-SIMULATION.3`
  Status: `pending`
  Goal: `Wire the harness into tool_matrix as an opt-in --diff-sim mode covering a representative scenario subset (not the full matrix, which is computationally infeasible). Add saw_design_with_cross_simulator_agreement coverage fact.`
  Acceptance: `cargo run --bin tool_matrix -- --phase4-hierarchy-gate --diff-sim --out ... produces a report with cross-simulator agreement metrics; matrix has no spurious mismatches on the chosen subset.`
  Verification: `pending`
  Commit: `pending`

- ID: `DIFFERENTIAL-SIMULATION.4`
  Status: `pending`
  Goal: `Document the new downstream contract: README, USER_GUIDE, book/src/synthesizability.md or a new chapter describes that ANVIL output is now gated for cross-simulator agreement on a representative subset.`
  Acceptance: `Docs describe the contract and how to invoke the differential gate; mdbook build clean.`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `DIFFERENTIAL-SIMULATION.1` | `pending` | Cannot design the harness without first establishing which simulators can ingest current output. The compatibility investigation is a pure-research leaf and unblocks everything else. |

## Decisions

- `2026-05-14`: Open-source simulators only for the first pass. Commercial simulator parity (VCS, Xcelium, Questa) is explicitly deferred — those tools are not available in the project's local environment, and the open-source pair already gives independent corroboration. Revisit once Verilator+iverilog parity is solid.

## Open Questions

- Should input-vector generation be deterministic (seeded RNG) or
  pattern-based (zero, all-ones, random-walk, edge cases like sign
  boundaries)? Owner: `DIFFERENTIAL-SIMULATION.2` design.
- How do we handle simulation timing differences? Verilator is a
  cycle-accurate event-driven simulator; iverilog is event-driven.
  Output sampling needs a single canonical sample point. Owner:
  `DIFFERENTIAL-SIMULATION.2`.
- What is the gate-time budget for `--diff-sim`? Each simulator run
  takes wall-clock time per design; the full 204-scenario matrix
  is infeasible. Need a representative-subset selector. Owner:
  `DIFFERENTIAL-SIMULATION.3`.

## Blockers

- None on `DIFFERENTIAL-SIMULATION.1`. Once `.1` lands, `.2`–`.4`
  become eligible.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `pending` | `DIFFERENTIAL-SIMULATION.1` | `pending` | `pending` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `DIFFERENTIAL-SIMULATION.1` | `pending` | `pending` |

## Changelog

- `2026-05-14`: Created task tree as part of the quality-improvement initiative.
