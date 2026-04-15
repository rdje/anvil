# Memory
Compact, operational continuity snapshot. Read on session bootstrap. Keep only what is actionable.

## Current state
- **Phase:** Phase 0 done. Phase 1 (Single-module MVP) in progress with both one-hot and encoded-select flop mux motifs.
- **Last completed slice:** encoded-select flop mux variant added alongside one-hot. Chained ternary over `Eq(sel, k)`, `ceil(log2(M))`-bit select bus, per-flop choice via `flop_mux_encoding_prob`. `FlopMux` enum (None / OneHot / Encoded) replaces the old `arms` field. See `CHANGES.md` entry `2026-04-15-0005`.
- **Next up:**
  1. Per-gate operand width validation in `src/ir/validate.rs` (still TODO; the most important missing safety net).
  2. Unit tests inside `src/ir/types.rs`, `src/gen/cone.rs`, `src/emit/sv.rs` (today only `tests/pipeline.rs` exercises the stack).
  3. Verilator-lint smoke when `verilator` is locally available, or wire CI to provide it.
  4. After above: declare Phase 1 done and start Phase 2 (signal sharing / DAG cones).

## Recent commits
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
- Per-gate operand-width validator: TODO in `src/ir/validate.rs`.
- Concat / Slice are used by the adapter and the flop emitter, but `input_widths_for(Concat|Slice, ...)` is still a placeholder. They are not selectable by `pick_gate` in Phase 1, so the placeholder is dead code today.
- Sharing (DAG cones), structured ops (case, for-loop), hierarchy, parameterization: not started.

## Session handoff notes
- All design decisions discussed so far are captured in `book/src/core-idea.md`, `book/src/why-not-grammar.md`, `book/src/non-triviality.md`, and `book/src/non-goals.md`. Read those before proposing structural changes.
- `COMMIT.md` is strict. Follow it exactly. `git_message_brief.txt` must stay untracked.
- The generator's "by construction" contract is load-bearing. Any PR that adds a generate-then-filter step (aside from the bounded retry in `cone::build_cone_with_retry`) is a design regression.
