# Memory
Compact, operational continuity snapshot. Read on session bootstrap. Keep only what is actionable.

## Current state
- **Phase:** Phase 0 done. Phase 1 (Single-module MVP) effectively feature-complete pending Verilator-lint smoke. Phase 2 (Signal sharing / DAG cones) in progress with default-on.
- **Last completed slice:** 4 new inline unit tests for the flop-mux assemblers (`assemble_flop_d_one_hot_zero_default_top_is_or`, `assemble_flop_d_one_hot_qfeedback_includes_q_term`, `assemble_flop_d_encoded_zero_default_top_is_mux`, `assemble_flop_d_encoded_qfeedback_fallthrough_is_q`) + test fixture helpers `fixture_with_inputs` and `alloc_flop`. New `book/src/faq.md` chapter with 12 Q&A entries covering the vocabulary/doctrine questions that have come up in the design (operators-vs-blocks, coefficient-vs-shift-amount-vs-comparand, Q-feedback, cross-output sharing, reproducibility, non-goals, synthesizability). Added to SUMMARY.md under Reference. 29 unit + 14 integration = 43 tests. See `CHANGES.md` entry `2026-04-15-0028`.
- **Status:** all three constant-role motifs implemented (coefficients ✅, shift amounts ✅, comparands ✅). Verilator-lint smoke is blocked (no Verilator available). Phase 1/2 feature work done in practice.
- **Conceptual advance this session:** the operators-vs-blocks distinction is now load-bearing doctrine. Operators (associative primitives) generalize by arity; blocks (mux, flop, future memory/FSM) generalize by structural parameters (port counts, encoding choices, feedback topology). Subsequent slices use this framework.
- **Next up (per user direction: switch to Phase 3+ since Verilator is unavailable):**
  1. **Phase 3+ entry point.** The roadmap lists: structured combinational ops (case/casez, priority encoders, shifts-already-done, for-loop unrolled logic), hierarchy (module instantiation, library/on-demand sub-module sourcing), parameterization (parameter-dependent widths), memories (inferrable patterns), FSMs (explicit state encodings), optional multi-clock. User needs to scope the first Phase 3+ slice. Candidates ranked by independent value and complexity:
     a) **Hierarchy (Phase 4 per ROADMAP)** — single biggest expressiveness gain. Module instantiation means anvil can emit realistic multi-module designs. Largest slice among the candidates.
     b) **Case / casez structured combinational blocks (Phase 3)** — compound block motif, adds case-statement idiom to generated output. Medium slice.
     c) **Priority encoder block (Phase 3)** — specific motif. Small slice.
     d) **Memories (Phase 6)** — inferrable read/write patterns. Medium slice.
     e) **FSMs (Phase 6)** — explicit state-encoding block with transition logic. Medium-large slice.
     f) **Parameterization (Phase 5)** — parameter-dependent widths, generate-loops. Large slice touching the IR.
  2. Blocked on external tooling:
     - Verilator-lint smoke run (no Verilator available).
     - Yosys smoke run (not attempted).

## Recent commits
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
