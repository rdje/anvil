# Memory
Compact, operational continuity snapshot. Read on session bootstrap. Keep only what is actionable.

## Current state
- **Phase:** Phase 0 done. Phase 1 (Single-module MVP) effectively feature-complete pending Verilator-lint smoke. Phase 2 (Signal sharing / DAG cones) in progress with default-on.
- **Last completed slice:** Q-feedback rule relaxation and the birth of the structural-rules catalog. Per user direction, a flop's own Q is now freely reachable in its own D-cone sub-cones (any number of times). `exclude = Some(q_node)` dropped from the three flop drainer sites. New `book/src/structural-rules.md` chapter codifies this and every other load-bearing generator invariant (13 rules today). `DEVELOPMENT_NOTES.md` and `CODEBASE_ANALYSIS.md` now point to the catalog instead of inlining rules. See `CHANGES.md` entry `2026-04-15-0014`.
- **Next up:**
  1. Verilator-lint smoke run (still blocked on Verilator availability). Sweep `share_prob ∈ {0.0, 0.3, 0.9}` and both flop styles for Phase 2 exit.
  2. After Verilator-lint green: declare Phase 2 done and start Phase 3 (structured combinational ops: case/casez, priority encoders, shifts, for-loop unrolled logic).
  3. Optional pre-Phase-3 polish: unit tests for `assemble_flop_d_encoded` / `assemble_flop_d_one_hot`.
  4. Optional book polish: add a "Tools this project uses" section linking to Verilator/Yosys for downstream smoke tests; add a FAQ chapter as questions accumulate.

## Recent commits
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
