# DIFFERENTIAL-SIMULATION: Cross-simulator semantic-equivalence gate for emitted RTL

## Metadata

- Tree ID: `DIFFERENTIAL-SIMULATION`
- Status: `active`
- Roadmap lane: Quality — signoff-level downstream consistency
- Created: `2026-05-14`
- Last updated: `2026-05-18` (`.1` landed — iverilog zero-config-compatible all 4 categories; frontier → `.2`)
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
  Status: `done`
  Goal: `Verilator-side compatibility is already proven by the matrix gate. Scope the SECOND simulator (iverilog is the default candidate): does it ingest ANVIL's emitted SV without configuration? Where does its semantics diverge from Verilator's? Output: DEVELOPMENT_NOTES.md entry with a focused compatibility note (what iverilog accepts, what it rejects, what it warns on) and a chosen differential-pair recommendation.`
  Acceptance: `DEVELOPMENT_NOTES.md entry exists; iverilog (or alternative) compatibility status is named per ANVIL output category (combinational leaf, sequential leaf, hierarchy, helper-instance routes); rejected alternatives are recorded.`
  Verification: `DEVELOPMENT_NOTES.md "Second-simulator (iverilog) compatibility note (2026-05-18, DIFFERENTIAL-SIMULATION.1)" entry landed. Installed Icarus Verilog 13.0 (stable) and empirically probed iverilog -g2012 -o /dev/null (full parse+elaborate) vs verilator --lint-only on freshly-generated release output for ALL FOUR categories: combinational leaf (--seed 7 --flop-prob 0), sequential leaf with flops (--seed 5 --flop-prob 1.0), bounded recursive hierarchy (4 modules, --min/max-hierarchy-depth 2), helper-instance/sibling routes (3 modules, --hierarchy-sibling-route-prob 1.0). RESULT: iverilog exit 0 SILENT on every category; verilator clean on every category. Verdict: iverilog is a ZERO-CONFIGURATION second simulator (only the standard -g2012 SV-2012 select; no source edits/shims/per-category flags). Chosen differential pair = Verilator (compiled, 2-state-default, cycle-driven) ↔ iverilog (interpreted, 4-state, event-driven) — strong because the engines are semantically independent. Documented the single material divergence to design around (NOT an ingest blocker): pre-reset 4-state behaviour (iverilog flops x until async reset deasserts vs Verilator 2-state 0) ⇒ .2 harness must drive a deterministic reset, sample at one canonical post-reset point, compare defined bits only (these are exactly the tree's .2/.3 Open Questions — confirmed design problems, not feasibility blockers). 4 rejected/deferred alternatives (verilator self-vs-self / Yosys-as-sim / commercial / single-simulator). Research-only — no code change (diff = DEVELOPMENT_NOTES.md + tree/live-docs); cargo unchanged-green vs base da3a00d (no src/tests touched). iverilog now installed locally (icarus-verilog via brew).`
  Commit: `Docs: DIFFERENTIAL-SIMULATION.1 iverilog second-simulator compatibility note`

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
| 1 | `DIFFERENTIAL-SIMULATION.2` | `pending` | `.1` **done** — iverilog 13.0 installed; empirically proven a zero-config second simulator for all 4 ANVIL output categories (parse+elaborate exit 0 silent); pair = Verilator ↔ iverilog; the one divergence (pre-reset 4-state) documented as a `.2` design constraint, not a blocker. `.2` builds the single-design differential harness `(generated SV, input-vector seed, cycles) → aligned post-reset output traces` from both simulators, with a focused byte-equal proof on a combinational + a sequential design. Unblocked; code slice (one cargo-test). |

## Decisions

- `2026-05-14`: Open-source simulators only for the first pass. Commercial simulator parity (VCS, Xcelium, Questa) is explicitly deferred — those tools are not available in the project's local environment, and the open-source pair already gives independent corroboration. Revisit once Verilator+iverilog parity is solid.
- `2026-05-14`: Verilator is already a fait accompli — it's wired into the Phase 4 hierarchy matrix gate and every focused proof passes through it. Yosys (a *synthesizer*, not a simulator) is also in the flow but is not a differential-simulation peer. So `DIFFERENTIAL-SIMULATION.1` is really about scoping the **second** simulator: iverilog is the obvious candidate (mature, MIT-licensed, event-driven), but the leaf should also rule on whether `verilator --binary` simulation mode is sufficient or whether we need pure-event-driven semantics (iverilog) as the contrast. The compatibility investigation now has a much narrower question to answer than "which simulators ingest our output" — it's "what does iverilog (or alternative) require to ingest our existing Verilator-clean output, and where do the two diverge."

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
| `2026-05-18` | `DIFFERENTIAL-SIMULATION.1` | Installed iverilog 13.0; `iverilog -g2012 -o /dev/null` vs `verilator --lint-only` on freshly-generated release SV for all 4 categories (comb leaf / seq-flop leaf / 4-module recursive hierarchy / 3-module helper-sibling routes). | **All categories: iverilog exit 0 silent + verilator clean.** iverilog is a zero-config second simulator; pair = Verilator↔iverilog; pre-reset 4-state divergence documented as a `.2` design constraint; 4 rejected alternatives. Research-only, no code; `cargo` unchanged-green vs `da3a00d`. Done; frontier → `.2`. |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `DIFFERENTIAL-SIMULATION.1` | `Docs: DIFFERENTIAL-SIMULATION.1 iverilog second-simulator compatibility note` | Research-only; iverilog 13.0 zero-config-compatible all 4 categories; Verilator↔iverilog pair chosen; 4 rejected alternatives. No code. |

## Changelog

- `2026-05-14`: Created task tree as part of the quality-improvement initiative.
- `2026-05-18`: **`.1` landed** (research-only, no code) —
  continuous-PNT while Phase 6 `.2.4`/`.3.4b` gate-blocked.
  Installed Icarus Verilog 13.0 and empirically proved iverilog
  ingests **all four** ANVIL output categories (combinational leaf,
  sequential-flop leaf, bounded recursive hierarchy, helper-instance/
  sibling routes) with **zero configuration** beyond `-g2012`
  (`iverilog -o /dev/null` exit 0 silent; Verilator clean on the
  same). Differential pair chosen: **Verilator (compiled, 2-state,
  cycle-driven) ↔ iverilog (interpreted, 4-state, event-driven)** —
  engine-independent. The one material divergence (pre-reset
  4-state: iverilog `x` until async reset deasserts vs Verilator
  `0`) is documented as a `.2` harness design constraint (drive a
  reset, sample at one canonical post-reset point, compare defined
  bits), confirming the tree's `.2`/`.3` Open Questions are design
  problems, not feasibility blockers. 4 rejected/deferred
  alternatives recorded. `DEVELOPMENT_NOTES.md` entry landed; tree
  unblocked through `.4`. Frontier → `.2` (build the single-design
  differential harness).
