# Memory
Compact, operational continuity snapshot. Read on session bootstrap. Keep only what is actionable.

## Current state
- **Phase:** Phase 0 done. Phase 1 (Single-module MVP) effectively feature-complete pending Verilator-lint smoke. Phase 2 (Signal sharing / DAG cones) in progress with default-on.
- **Last completed slice:** Rule 18 proposal + sample-output defect catalogue (docs only). See `CHANGES.md` entry `2026-04-15-0030`. Rule 18 ("No orphan gates") is added to the structural-rules catalog *proposed, not yet enforced*. `DEVELOPMENT_NOTES.md` gained a concrete defect catalogue from sample module `mod_1_0000`: constant-select muxes, N-arity self-cancellation, coefficient width overflow (`1'h6`), dead wires, stranded flop, identical one-hot arms. Enforcement decision for Rule 18 (α construction-time vs β emission-time tree-shake) is deferred to the next session.
- **Prior slice:** priority-encoder block — first of the Phase 3 small-to-medium motifs. New `priority_encoder_prob` knob (default 0.05) + CLI flag. `pick_priority_encoder_n` finds an N ∈ `[min_mux_arms, max_mux_arms]` with `ceil_log2(N) == target_width`, returns None if none fits. `assemble_priority_encoder` emits a chained ternary `req_0 ? 0 : req_1 ? 1 : ... : 0`. `build_priority_encoder_recursive` and `build_priority_encoder_pool` dispatch helpers. Three dispatch sites (build_cone / process_signal_frame / grow_pool_one_unit) with applicability-check-then-fall-through semantics. Book Rule 17 added. 1 new integration test. 29 unit + 15 integration = 44 tests. See `CHANGES.md` entry `2026-04-15-0029`.
- **Doctrinal note (deferred):** the motif-trait refactor is explicitly deferred per user direction. After landing several more block motifs, revisit to factor the copy-paste pattern into a `Motif` trait + registry.
- **Conceptual advance this session:** the operators-vs-blocks distinction is now load-bearing doctrine. Operators (associative primitives) generalize by arity; blocks (mux, flop, future memory/FSM) generalize by structural parameters (port counts, encoding choices, feedback topology). Subsequent slices use this framework.
- **Next up (closing small-to-medium Phase 3+ motifs first, per user direction):**
  0. **Fix source-of-generation defects observed in sample output.** Six concrete items catalogued in `DEVELOPMENT_NOTES.md`. Three categories: (a) anti-collapse operand-multiset check (N-arity self-cancel, identical one-hot arms); (b) position-dependent leaf rules (no const in mux select); (c) width-aware coefficient generation (coefficient-bits ≤ operand-width). Plus the orthogonal Rule 18 enforcement decision (α construction-time vs β emission-time tree-shake; β is the low-friction first step).
  1. **case/casez structured combinational blocks (medium).** A block that takes a select signal (1..N bit wide) and emits `always_comb case (sel) ... endcase` (or equivalent chained-ternary if we stay in expression land). Similar to the encoded mux but the emitted SV uses a `case` statement with explicit branches. Distinct synthesizer code path from chained-ternary muxes.
  2. **Memories (medium).** Inferrable single-port / simple-dual-port memory patterns (`reg [W-1:0] mem [0:DEPTH-1]` with an always_ff block driving read/write). Knob for depth range.
  3. **FSMs (medium-large).** Explicit state encoding (binary / one-hot / gray), transition logic, optional output logic. The first real multi-part block motif.
  4. After the above, revisit the motif-trait refactor (the copy-paste pattern will then cover ~7-8 block motifs, enough to extract the right abstraction).
  5. Large-scope deferred: hierarchy (a), parameterization (f).
  6. Blocked on external tooling: Verilator-lint smoke, Yosys smoke.

## Recent commits
- `b4c489a` — Priority-encoder block (Rule 17).
- `06b5a52` — Flop-assembler unit tests + FAQ chapter.
- `1211120` — Constant comparand motif: third and final constant-role motif.
- `2da9d3d` — Constant shift-amount motif + Shl/Shr added to pick_gate.
- `7290e3d` — Linear-combination coefficient motif for Add / Sub / Mul.
- `b0f84fd` — Sub coefficient constraint: ck > 0 for all k.
- `4085401` — graph-first strategy landed; becomes the new default.
- `6d2da98` — Interleaved construction strategy: frame state machine.
- `2d038a9` — Construction-strategy machinery + shuffled strategy landed.
- `8eb03f0` — Construction-strategies chapter: 4 named strategies, graph-first default.
- `126411d` — Rule 16: cross-output sharing via the module-wide signal pool.
- `8ff1d84` — Log constants-roles clarification in the book + two corrections.
- `dde27a2` — Doctrinal fix: coefficient / shift amount / comparand are distinct motifs.
- `0564a49` — M-to-1 combinational mux as a first-class block.
- `b91188d` — N-arity for associative operators + operators-vs-blocks doctrine.
- `6cbcbff` — Q-feedback rule relaxation + structural-rules catalog.
- `bac6060` — mdBook becomes user-facing: Getting Started, Tutorial, Recipes.
- `62fdeaa` — mdBook staleness refresh: knobs, IR, algorithm, architecture.
- `c9ec12c` — CLI coverage for all Phase 1/2 motif knobs.
- `6ba646b` — Phase 2 start: per-operand DAG-cone sharing.
- `c8043c3` — Inline unit tests for cone helpers and SV emitter.
- `4eb5daa` — Per-gate width/arity validator + inline unit tests.
- `f2a3d81` — Elevate mdBook to equal-standing live doc in session recovery.
- `a1a9ea9` — Live-doc catch-up + tighten commit workflow (12-item checklist).
- `10090c2` — Encoded-select flop mux (chained ternary) alongside one-hot.
- `47675df` — M-to-1 one-hot mux flops with two motifs (ZeroDefault, QFeedback).
- `4317c82` — Fold flops into the cone recursion (single-clock synchronous design).
- `c4668a2` — Elevate "recursion is the core principle" to load-bearing status.
- `5f6022f` — Initial scaffold + Phase 1 cone-adapter hardening.

## Open questions / deferred decisions
- Constant-probability value — current default `0.1` is a guess; tune after Phase 1 seed sweeps.
- Whether the IR should use `typed-arena` or stay on `Vec<Node>` with `u32` indices. Current choice: plain `Vec`, because it's simple, cache-friendly, and `serde`-friendly.
- The lazy adapter currently picks the *widest* pool entry with deps. Random-among-eligible may give better motif coverage; revisit after Phase 1 metrics.
- `flop_prob` default `0.15` is a guess; calibrate after the first synthesis smoke run that reports flop counts vs gate counts.
- `max_flops_per_module` cap of `32` is conservative. May raise once metrics show generation time is not bottlenecked by D-cone draining.
- `flop_mux_encoding_prob` default `0.5` is equal-motif; no empirical data yet. Bias once synthesis metrics show which style catches more bugs.
- Ternary-over-`case` for the Encoded mux SV form — see `DEVELOPMENT_NOTES.md` rejected-alternatives; revisit when/if FSM motifs force procedural block emission.

## Known gaps vs `ROADMAP.md`
- Phase 1 exit criterion (1000 modules through Verilator + Yosys) not yet met locally; tools missing.
- Concat / Slice are used by the adapter and the flop emitter, but `input_widths_for(Concat|Slice, ...)` is still a placeholder. They are not selectable by `pick_gate` in Phase 1, so the placeholder is dead code today.
- Structured ops (case, for-loop), hierarchy, parameterization: not started.

## Session handoff notes
- All design decisions discussed so far are captured in `book/src/core-idea.md`, `book/src/why-not-grammar.md`, `book/src/non-triviality.md`, and `book/src/non-goals.md`. Read those before proposing structural changes.
- `COMMIT.md` is strict. Follow it exactly. `git_message_brief.txt` must stay untracked.
- The generator's "by construction" contract is load-bearing. Any PR that adds a generate-then-filter step (aside from the bounded retry in `cone::build_cone_with_retry`) is a design regression.
