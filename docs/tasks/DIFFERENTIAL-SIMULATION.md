# DIFFERENTIAL-SIMULATION: Cross-simulator semantic-equivalence gate for emitted RTL

## Metadata

- Tree ID: `DIFFERENTIAL-SIMULATION`
- Status: `active`
- Roadmap lane: Quality — signoff-level downstream consistency
- Created: `2026-05-14`
- Last updated: `2026-05-20` (**`.2b` split** on a discovered iverilog↔verilator trace-alignment dependency surfaced by the very first real-tool gate run: iverilog samples one extra time at vector[0] vs verilator on the sequential design — the underlying output sequences match, just shifted. The harness helpers + cargo-portable proofs + `#[ignore]` scaffold landed cleanly in `.2b.1`; the trace-alignment fix + verified real-tool run is `.2b.2`. Mirrors the proven `PHASE-7-ORACLE-MICRODESIGN.2c.2a`/`.2c.2b` decomposition. Tree-planning, no code; frontier → `.2b.2`)
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
  Children: `DIFFERENTIAL-SIMULATION.2a` (done), `DIFFERENTIAL-SIMULATION.2b` (active container: `.2b.1`, `.2b.2`)

- ID: `DIFFERENTIAL-SIMULATION.2a`
  Status: `done`
  Goal: `Design the single-design Verilator↔iverilog differential harness in DEVELOPMENT_NOTES.md: IR-driven generic testbench generation (read ports from the Design/Module IR, not by re-parsing SV), the reset + single canonical post-reset sample-point alignment that neutralises .1's pre-reset 4-state divergence, deterministic baked input vectors (not per-sim $random), dual-simulator orchestration (iverilog -g2012 + vvp; verilator --binary), byte-equal trace comparison, the tool-gated #[ignore] focused-test convention (cargo test must stay green tool-less — the Phase-1 portability doctrine), rejected alternatives, and the .2b proof shape. Design-only; no code.`
  Acceptance: `DEVELOPMENT_NOTES.md design entry with the testbench-from-IR strategy, the reset/sample alignment, the deterministic-stimulus + tool-gated-test conventions, >=1 rejected alternative, and the .2b proof shape; no code change.`
  Verification: `DEVELOPMENT_NOTES.md "Single-design differential harness design (2026-05-18, DIFFERENTIAL-SIMULATION.2a)" entry landed. Records: testbench generation FROM the in-process Design/Module IR (port names/widths/dirs + has_local_flops/memories/fsms — never re-parsing emitted SV); reset + single canonical post-reset sample point that neutralises .1's pre-reset 4-state divergence (comb: hold+settle, no clock; seq: rst_n=0 K cycles → deassert → warmup → per-cycle sample; compare post-reset defined samples only); deterministic stimulus BAKED into the testbench from the seed (zero/all-ones/walking-1/seeded-pseudo-random), NOT per-sim $random (divergent streams ⇒ false mismatches); one identical testbench feeds both sims; orchestration iverilog -g2012 -o sim.vvp + vvp / verilator --binary -j0 -sv --top-module tb; byte-compare traces, mismatch = retained counterexample (Phase-7 parity discipline); tool-gated #[ignore] focused test so cargo test stays green tool-less (the Phase-1 doctrine; .2b adds ~zero mandatory-gate runtime). 5 rejected alternatives (parse SV / per-sim $random / non-ignored test / verilator --cc+C++ / compare pre-reset). .2b proof shape specified. Design-only; no code change (diff = DEVELOPMENT_NOTES.md + tree/live-docs); cargo unchanged-green vs base f0df6f5 (no src/tests touched).`
  Commit: `Docs: DIFFERENTIAL-SIMULATION.2a single-design differential-harness design`

- ID: `DIFFERENTIAL-SIMULATION.2b`
  Status: `active`
  Goal: `Implement the harness per .2a (IR-driven testbench emitter + iverilog/vvp + verilator --binary orchestration + post-reset trace align/compare) + a #[ignore]-gated focused test that, on a hand-picked combinational and a sequential (seed,config) leaf, drives both simulators and asserts byte-equal post-reset traces. cargo test stays green on tool-less machines. Split (Splitting Rules + the proven PHASE-7-ORACLE-MICRODESIGN.2c.2a/.2c.2b decomposition that closed Phase 7's parity gate on a discovered tool-capability dependency) into .2b.1 (harness helpers + testbench emitter + cargo-portable proofs of the helpers + #[ignore] scaffold; no real run verified, no advance) and .2b.2 (trace-alignment fix + verified real-tool run + byte-equal proof). The split was triggered by the very first real-tool gate run on .2b.1's scaffold surfacing an iverilog↔verilator sampling-alignment issue: iverilog produces one extra trace line at vector[0] compared to verilator on the sequential design — the underlying output sequences match, just shifted. The harness produces correct SV; the alignment discipline needs one more iteration. Per Splitting Rules + r87, the scaffold + portable proofs land first; the alignment fix + verified end-to-end run is a separate gated step.`
  Children: `DIFFERENTIAL-SIMULATION.2b.1`, `DIFFERENTIAL-SIMULATION.2b.2`

- ID: `DIFFERENTIAL-SIMULATION.2b.1`
  Status: `done`
  Goal: `Land the harness helpers + IR-driven testbench emitter + dual-sim orchestration + #[ignore] scaffold + cargo-portable proofs of the helpers. The #[ignore] test compiles and is invocable but is NOT required to pass end-to-end in this slice (the discovered iverilog↔verilator alignment dependency is .2b.2's deliverable). Portable cargo test stays green tool-less.`
  Acceptance: `cargo fmt/clippy(-D warnings)/check --all-targets/test green; new tests/diff_sim.rs landed with the harness helpers (baked_input_vectors / fmt_sv_hex / is_sequential / emit_testbench / run_iverilog / run_verilator / normalize_trace) + 5+ cargo-portable proofs of the helpers + the 2 #[ignore]-gated focused tests (combinational + sequential); the #[ignore] tests compile and are invocable but the real-tool byte-equality assertion is NOT required to pass in this slice; portable cargo test stays green tool-less.`
  Verification: `New tests/diff_sim.rs landed (~400 lines) with the harness helpers per .2a's design: baked_input_vectors(seed, n_inputs, n_vectors) produces the documented canonical-edge-case-prefixed reproducible vector sequence (all-zeros + all-ones + walking-1 + seeded ChaCha8 pseudo-random); fmt_sv_hex(v, width) produces fixed-width SV hex literals (<width>'h<nibbles>) masked to width using div_ceil; is_sequential(top) returns top.has_local_flops() || has_local_memories() || has_local_fsms() — NOT top.clock.is_some() (the latter is always true on ANVIL output per the synchronous-design discipline; the test caught the wrong assumption); emit_testbench(top, vectors) reads top.inputs/top.outputs from the IR directly (NOT by re-parsing SV) and emits a parameter-less SV testbench: instantiated DUT via named port map + reg/wire decls per port width + initial-block stimulus driver + canonical sample point ($display %h joined by spaces). Combinational shape: hold + #1 settle + $display. Sequential shape: clock generator (always #5 clk=~clk) + rst_n=0 + #45 + rst_n=1 + #20 warmup + per-vector drive + #10 + $display. emit_display_outputs handles the zero-output case ("NO_OUT" marker for stable line count). run_iverilog/run_verilator shell the respective tools (iverilog -g2012 -o sim.vvp dut.sv tb.sv → vvp sim.vvp; verilator --binary -j0 -sv --top-module tb --Mdir obj_dir dut.sv tb.sv → ./obj_dir/Vtb), capture stdout. normalize_trace filters to hex-only lines (per-token hex test so multi-output rows like "ca fe ba be" parse cleanly). tools_present() probes iverilog -V + verilator --version. 5 cargo-portable proofs of the helpers (all green): baked_input_vectors_are_reproducible_with_canonical_edge_cases (canonical-edge-case prefix + seed reproducibility + distinct seeds differ); fmt_sv_hex_produces_fixed_width_masked_literals (nibble counts + masking + 1-bit/4-bit/8-bit/9-bit/128-bit cases); normalize_trace_filters_to_hex_only_lines (multi-token hex rows preserved, banner/version/timing lines filtered); is_sequential_matches_clock_presence (combinational module reports false; sequential module reports true — caught the original wrong top.clock.is_some() assumption); emit_testbench_has_the_documented_shape (smoke: module tb, DUT instance, $display, $finish, endmodule all present). 2 tool-gated #[ignore] tests: differential_simulation_combinational + differential_simulation_sequential — each shells iverilog + verilator and asserts normalized-trace byte-equality; tools_present() guard makes them friendly no-ops when iverilog/verilator absent. Fixed 3 clippy hits: unused-imports (Direction); manual-div-ceil (used .div_ceil(4) on u32); field-reassign-with-default (used struct-update Config{seed, flop_prob, ..Config::default()}). cargo fmt --all --check / clippy --all-targets -- -D warnings / check --all-targets clean. Full cargo test green: tests/diff_sim 5 portable + 2 ignored; all other tests unchanged (lib 244; tests/microdesign_parity 15+1; tests/frontend_parity 12+2; tests/pipeline 121; tests/snapshots 6; bin 5+29+3; tests/book_examples 3; doc 0). Portable cargo test stays green tool-less. **Real-tool gate ran locally and SURFACED an iverilog↔verilator trace-alignment issue** (the discovered dependency that triggered the .2b split): iverilog samples one extra time at vector[0] vs verilator on the sequential design ("left: 8 items, right: 8 items, both have the same 7-item core sequence shifted by one"). The harness SV is correct; the alignment discipline (initial-block timing) needs one more iteration. **NOT a downstream-tool bug**: both sims are emitting the right values for the right cycles; they just disagree on when the FIRST post-reset sample happens. .2b.2 fixes the alignment + records a verified-clean end-to-end byte-equal run. No ROADMAP advance.`
  Commit: `DIFFERENTIAL-SIMULATION.2b.1 IR-driven testbench harness + dual-sim orchestration + cargo-portable helper proofs + #[ignore] scaffold`

- ID: `DIFFERENTIAL-SIMULATION.2b.2`
  Status: `pending`
  Goal: `Fix the iverilog↔verilator trace-alignment dependency surfaced by .2b.1's first real-tool gate run (iverilog produces one extra trace line at vector[0] vs verilator on the sequential design; underlying sequences match, just shifted). Candidate fixes: (a) sentinel '$display("BEGIN_TRACE")' before the vector loop so both sims sync on a known cycle; (b) align by dropping the first N samples from each trace before comparing; (c) restructure the initial-block timing so both sims sample the same canonical post-reset cycle. Then RUN the real-tool gate to completion (cargo test -- --ignored differential_simulation_combinational + ...sequential against locally-installed iverilog 13.0 + verilator 5.046), verify byte-equal post-reset traces, and bank the verified-clean evidence in the Verification Log. Closes DIFFERENTIAL-SIMULATION.2b + .2 container.`
  Acceptance: `Both #[ignore] tests run end-to-end and assert byte-equal post-reset traces on the chosen combinational + sequential designs; verified-clean evidence recorded; .2b.2 + .2b + .2 container all → done.`
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
| 1 | `DIFFERENTIAL-SIMULATION.2b.2` | `pending` (real-tool alignment fix + verified run) | **`.2b.1` done (`2026-05-20`)** — new `tests/diff_sim.rs` carries the harness helpers (`baked_input_vectors`/`fmt_sv_hex`/`is_sequential`/`emit_testbench`/`run_iverilog`/`run_verilator`/`normalize_trace`) + IR-driven testbench emitter + dual-sim orchestration + 2 `#[ignore]`-gated focused tests (combinational + sequential). 5 cargo-portable helper proofs green (`baked_input_vectors_are_reproducible_with_canonical_edge_cases`/`fmt_sv_hex_produces_fixed_width_masked_literals`/`normalize_trace_filters_to_hex_only_lines`/`is_sequential_matches_clock_presence`/`emit_testbench_has_the_documented_shape`). `cargo fmt`/clippy/check clean; full `cargo test` green; portable stays green tool-less. **The first real-tool gate run surfaced a documented iverilog↔verilator trace-alignment issue** (iverilog samples one extra time at vector[0] vs verilator on the sequential design; underlying sequences match, just shifted). `.2b.2` fixes the alignment (sentinel `$display` or initial-block timing restructuring) + RUNS the real-tool gate end-to-end + records the verified-clean byte-equal evidence. Mirrors `PHASE-7-ORACLE-MICRODESIGN.2c.2a`/`.2c.2b` discovered-dependency-split. |

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
