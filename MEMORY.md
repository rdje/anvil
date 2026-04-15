# Memory
Compact, operational continuity snapshot. Read on session bootstrap. Keep only what is actionable.

## Current state
- **Phase:** Phase 0 done locally. Phase 1 (Combinational MVP) in progress.
- **Last completed slice:** initial scaffold + Phase 1 cone-adapter hardening. `cargo check`, `cargo test`, `cargo clippy -D warnings`, `cargo fmt --check` all clean. See `CHANGES.md` entry `2026-04-15-0001`.
- **Next up:**
  1. Per-gate operand width validation in `src/ir/validate.rs` (currently TODO; this is Phase 1's most important missing safety net).
  2. Unit tests inside `src/ir/types.rs`, `src/gen/cone.rs`, `src/emit/sv.rs` (today only `tests/pipeline.rs` exercises the stack).
  3. Verilator-lint smoke when `verilator` is locally available, or wire CI to provide it.
  4. After above: declare Phase 1 done in `ROADMAP.md` and start Phase 2 (sequential).

## Recent commits
- `5f6022f` — Initial scaffold + Phase 1 cone-adapter hardening.

## Open questions / deferred decisions
- Async vs sync reset mix ratio — knob exists (`use_async_reset: bool`); may want a probability instead when Phase 2 lands.
- Constant-probability value — current default `0.1` is a guess; tune after Phase 1 seed sweeps.
- Whether the IR should use `typed-arena` or stay on `Vec<Node>` with `u32` indices. Current choice: plain `Vec`, because it's simple, cache-friendly, and `serde`-friendly.
- The lazy adapter currently picks the *widest* pool entry with deps. Random-among-eligible may give better motif coverage; revisit after Phase 1 metrics.

## Known gaps vs `ROADMAP.md`
- Phase 1 exit criterion (1000 modules through Verilator + Yosys) not yet met locally; tools missing.
- Per-gate operand-width validator: TODO in `src/ir/validate.rs`.
- Concat is now used by the adapter (variadic with same operand replicated). Width is always `copies * src_width`. The emitter handles variadic correctly; `input_widths_for(Concat, ...)` is still a placeholder used only when `Concat` is selected by `pick_gate`, which Phase 1 does not do.
- Slice is now used by the adapter. `input_widths_for(Slice, ...)` is still a placeholder used only when `Slice` is selected by `pick_gate`, which Phase 1 does not do.
- Flop worklist and `always_ff` emission are stubs; Phase 2 work.
- Hierarchy generator is absent (Phase 5).

## Session handoff notes
- All design decisions discussed so far are captured in `book/src/core-idea.md`, `book/src/why-not-grammar.md`, `book/src/non-triviality.md`, and `book/src/non-goals.md`. Read those before proposing structural changes.
- `COMMIT.md` is strict. Follow it exactly. `git_message_brief.txt` must stay untracked.
- The generator's "by construction" contract is load-bearing. Any PR that adds a generate-then-filter step (aside from the bounded retry in `cone::build_cone_with_retry`) is a design regression.
