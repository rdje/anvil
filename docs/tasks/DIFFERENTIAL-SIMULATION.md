# DIFFERENTIAL-SIMULATION: Cross-simulator semantic-equivalence gate for emitted RTL

## Metadata

- Tree ID: `DIFFERENTIAL-SIMULATION`
- Status: `active`
- Roadmap lane: Quality — signoff-level downstream consistency
- Created: `2026-05-14`
- Last updated: `2026-05-18` (`.2` split + `.2a` harness design landed; frontier → `.2b`)
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
  Status: `active`
  Goal: `Single-design differential harness. Split (Splitting Rules + the proven design-first method: the testbench-generation strategy, reset/sample-point alignment, dual-simulator orchestration, and the tool-gated-test convention are load-bearing decisions to settle before code — and the design is docs-only, ~zero contention on the near-complete Phase 6 priority gate) into .2a (design, no code) and .2b (implement the harness + the focused tool-gated proof).`
  Children: `DIFFERENTIAL-SIMULATION.2a`, `DIFFERENTIAL-SIMULATION.2b`

- ID: `DIFFERENTIAL-SIMULATION.2a`
  Status: `done`
  Goal: `Design the single-design Verilator↔iverilog differential harness in DEVELOPMENT_NOTES.md: IR-driven generic testbench generation (read ports from the Design/Module IR, not by re-parsing SV), the reset + single canonical post-reset sample-point alignment that neutralises .1's pre-reset 4-state divergence, deterministic baked input vectors (not per-sim $random), dual-simulator orchestration (iverilog -g2012 + vvp; verilator --binary), byte-equal trace comparison, the tool-gated #[ignore] focused-test convention (cargo test must stay green tool-less — the Phase-1 portability doctrine), rejected alternatives, and the .2b proof shape. Design-only; no code.`
  Acceptance: `DEVELOPMENT_NOTES.md design entry with the testbench-from-IR strategy, the reset/sample alignment, the deterministic-stimulus + tool-gated-test conventions, >=1 rejected alternative, and the .2b proof shape; no code change.`
  Verification: `DEVELOPMENT_NOTES.md "Single-design differential harness design (2026-05-18, DIFFERENTIAL-SIMULATION.2a)" entry landed. Records: testbench generation FROM the in-process Design/Module IR (port names/widths/dirs + has_local_flops/memories/fsms — never re-parsing emitted SV); reset + single canonical post-reset sample point that neutralises .1's pre-reset 4-state divergence (comb: hold+settle, no clock; seq: rst_n=0 K cycles → deassert → warmup → per-cycle sample; compare post-reset defined samples only); deterministic stimulus BAKED into the testbench from the seed (zero/all-ones/walking-1/seeded-pseudo-random), NOT per-sim $random (divergent streams ⇒ false mismatches); one identical testbench feeds both sims; orchestration iverilog -g2012 -o sim.vvp + vvp / verilator --binary -j0 -sv --top-module tb; byte-compare traces, mismatch = retained counterexample (Phase-7 parity discipline); tool-gated #[ignore] focused test so cargo test stays green tool-less (the Phase-1 doctrine; .2b adds ~zero mandatory-gate runtime). 5 rejected alternatives (parse SV / per-sim $random / non-ignored test / verilator --cc+C++ / compare pre-reset). .2b proof shape specified. Design-only; no code change (diff = DEVELOPMENT_NOTES.md + tree/live-docs); cargo unchanged-green vs base f0df6f5 (no src/tests touched).`
  Commit: `Docs: DIFFERENTIAL-SIMULATION.2a single-design differential-harness design`

- ID: `DIFFERENTIAL-SIMULATION.2b`
  Status: `pending`
  Goal: `Implement the harness per .2a (IR-driven testbench emitter + iverilog/vvp + verilator --binary orchestration + post-reset trace align/compare) + a #[ignore]-gated focused test that, on a hand-picked combinational and a sequential (seed,config) leaf, drives both simulators and asserts byte-equal post-reset traces. cargo test stays green on tool-less machines. May sub-split if implementing surfaces a lower-level dependency.`
  Acceptance: `Focused #[ignore] test (run with both sims present) gets two post-reset traces and asserts byte-equality on >=1 combinational + >=1 sequential design; cargo fmt/clippy/check/test green (the diff-sim test ignored by default so the portable gate is unaffected).`
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
| 1 | `DIFFERENTIAL-SIMULATION.2b` | `pending` | `.1` done (iverilog zero-config-compatible) → `.2` split → `.2a` design **done** (DEVELOPMENT_NOTES.md: IR-driven testbench, post-reset canonical sampling neutralising the 4-state gap, baked deterministic stimulus, iverilog/`verilator --binary` orchestration, tool-gated `#[ignore]` test, 5 rejected alternatives). `.2b` implements the harness + the `#[ignore]` focused byte-equal proof (combinational + sequential). Unblocked; code slice — the diff-sim test is `#[ignore]` so the portable `cargo test` is unaffected (low marginal gate contention). May sub-split if implementing surfaces a lower-level dependency. |

## Decisions

- `2026-05-14`: Open-source simulators only for the first pass. Commercial simulator parity (VCS, Xcelium, Questa) is explicitly deferred — those tools are not available in the project's local environment, and the open-source pair already gives independent corroboration. Revisit once Verilator+iverilog parity is solid.
- `2026-05-14`: Verilator is already a fait accompli — it's wired into the Phase 4 hierarchy matrix gate and every focused proof passes through it. Yosys (a *synthesizer*, not a simulator) is also in the flow but is not a differential-simulation peer. So `DIFFERENTIAL-SIMULATION.1` is really about scoping the **second** simulator: iverilog is the obvious candidate (mature, MIT-licensed, event-driven), but the leaf should also rule on whether `verilator --binary` simulation mode is sufficient or whether we need pure-event-driven semantics (iverilog) as the contrast. The compatibility investigation now has a much narrower question to answer than "which simulators ingest our output" — it's "what does iverilog (or alternative) require to ingest our existing Verilator-clean output, and where do the two diverge."

- `2026-05-18`: **`.2` split** into `.2a` (design — IR-driven
  testbench, reset/post-reset-sample alignment, baked deterministic
  stimulus, dual-sim orchestration, tool-gated `#[ignore]` test
  convention, rejected alternatives; docs-only) and `.2b`
  (implement the harness + the focused tool-gated byte-equal
  proof). Splitting Rules + the proven design-first method (the
  testbench-from-IR strategy and the tool-gated-test convention are
  load-bearing decisions to settle before code) + a contention
  judgment: `.2a` is docs-only ⇒ ~zero contention on the
  near-complete Phase 6 priority gate, whereas a single bundled
  `.2` would start a sustained simulator-orchestration code
  campaign. Mirrors the Phase 7/8/9 design-first discipline. `.2`
  is now a container; `.3`/`.4` unchanged; no renumbering.
  Frontier → `.2a` → (done) → `.2b`.

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
| `2026-05-18` | `DIFFERENTIAL-SIMULATION.2a` | `DEVELOPMENT_NOTES.md` "Single-design differential harness design" entry landed: IR-driven generic testbench (ports from `Design`/`Module`, not SV re-parse); reset + single canonical post-reset sample point (neutralises `.1`'s pre-reset 4-state gap); baked deterministic stimulus (not per-sim `$random`); orchestration `iverilog -g2012`+`vvp` / `verilator --binary`; byte-equal trace compare with retained counterexamples; tool-gated `#[ignore]` focused test (cargo test stays green tool-less — Phase-1 doctrine); 5 rejected alternatives; `.2b` proof shape. Design-only, no code; `cargo` unchanged-green vs base `f0df6f5` (no `src/`/`tests/` touched). | Done. `.2` split; frontier → `.2b`. |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `DIFFERENTIAL-SIMULATION.1` | `Docs: DIFFERENTIAL-SIMULATION.1 iverilog second-simulator compatibility note` | Research-only; iverilog 13.0 zero-config-compatible all 4 categories; Verilator↔iverilog pair chosen; 4 rejected alternatives. No code. |
| `DIFFERENTIAL-SIMULATION.2a` | `Docs: DIFFERENTIAL-SIMULATION.2a single-design differential-harness design` | Design-only; IR-driven testbench + post-reset sampling + baked stimulus + tool-gated `#[ignore]` test + 5 rejected alternatives. `.2` split. No code. |

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
- `2026-05-18`: **`.2` split + `.2a` design landed** (design-only,
  no code) — continuous-PNT while Phase 6 `.2.4`/`.3.4b` are
  gate-blocked; `.2a` is docs-only so it does not start a sustained
  simulator-orchestration code campaign that would starve the
  near-complete Phase 6 priority gate (the same contention-aware
  design-first discipline applied to Phase 7/8/9).
  `DEVELOPMENT_NOTES.md` "Single-design differential harness
  design": testbench generated **from the IR** (not by re-parsing
  SV); reset + single canonical post-reset sample point that
  neutralises `.1`'s pre-reset 4-state divergence; **baked**
  deterministic stimulus (not per-sim `$random`); one testbench
  feeds both `iverilog -g2012`+`vvp` and `verilator --binary`;
  byte-equal trace compare with retained counterexamples;
  `#[ignore]` tool-gated focused test so the portable `cargo test`
  stays green tool-less (Phase-1 doctrine; `.2b` adds ~zero
  mandatory-gate runtime). 5 rejected alternatives; `.2b` proof
  shape. `.2` is now a container; `.3`/`.4` unchanged. Frontier →
  `.2b` (implement).
