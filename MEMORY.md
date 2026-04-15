# Memory
Compact, operational continuity snapshot. Read on session bootstrap. Keep only what is actionable.

## Current state
- **Phase:** Phase 0 done. Phase 1 (Single-module MVP) effectively feature-complete pending Verilator-lint smoke. Phase 2 (Signal sharing / DAG cones) in progress with default-on.
- **Last completed slice:** M-to-1 combinational mux as a first-class block. `build_cone` gains a new branch (between flop and operator) that calls `build_comb_mux`. OneHot style = `OR_i({W{sel_i}} & data_i)`; Encoded style = chained ternary over `Eq(sel, k)` with a 0 fall-through. No Q-feedback axis (combinational muxes have no state). New knobs `comb_mux_prob` (default 0.1), `comb_mux_encoding_prob` (default 0.5), both with CLI flags. New book Rule 15. New unit test `comb_mux_block_produces_valid_output` across 10 seeds × 2 encodings = 20 modules, all IR-valid. Tutorial Example 9 + Recipe entry added. See `CHANGES.md` entry `2026-04-15-0016`.
- **Conceptual advance this session:** the operators-vs-blocks distinction is now load-bearing doctrine. Operators (associative primitives) generalize by arity; blocks (mux, flop, future memory/FSM) generalize by structural parameters (port counts, encoding choices, feedback topology). Subsequent slices use this framework.
- **Next up:**
  1. **Construction-strategy machinery:** four named strategies supported per-run via a `construction_strategy` knob — `sequential` (current), `shuffled`, `interleaved`, `graph-first` (planned default). See `book/src/construction-strategies.md`. Implementation sequence: (a) add the knob with only `sequential` accepted, (b) land `shuffled` (cheap), (c) land `interleaved` (moderate rewrite), (d) land `graph-first` and flip the default.
  2. **Linear-combination ADD motif (coefficients):** `y = s1*c1 + s2*c2 + ... + sn*cn` where `n` and each `ci` are randomized, `ci ≠ 0` (zero coefficient kills its term). Compound motif: each ADD term is itself a Mul(signal, non-zero constant). Similar shapes to follow for Sub and Mul with their own constraints per user guidance. Knob family: `coefficient_prob`, `min_coefficient`, `max_coefficient`. **Arithmetic only** — coefficients are multiplicative weights, not constants-in-general.
  2. **Shift amounts — constant-vs-variable bias:** shifts `Shl/Shr` today always emit variable-amount (`a << count` with `count` an 8-bit signal — synthesizes to a barrel shifter, expensive in hardware). Real designs overwhelmingly use constant shift amounts (`a << 2` — wire reroute, cheap). Add a bias: per-shift probability (`const_shift_amount_prob`) of emitting a constant shift amount in range `[0, W-1]` instead of a signal. Both modes coexist under a knob. Variable-amount remains legal and useful for barrel-shifter stress.
  3. **Comparands additive to signal-vs-signal comparisons:** today all comparisons are signal-vs-signal (`a == b`, `x < y`) because both operands come from `build_cone`. Add a motif: per-comparison probability (`const_comparand_prob`) that the RHS is a constant comparand (`a == 7`, `x >= LIMIT`) instead of another signal. The comparand motif is *additive* — signal-vs-signal remains the default and dominant shape; comparands add threshold/sentinel patterns on top. No zero-exclusion.
  4. Note: "coefficient" / "shift amount" / "comparand" are distinct vocabularies with distinct constraints — see `book/src/structural-rules.md` "Roles of constants in RTL". Do not collapse into a single `constant_prob` knob.
  3. Verilator-lint smoke run (still blocked on Verilator availability). Sweep `share_prob ∈ {0.0, 0.3, 0.9}` and both flop styles for Phase 2 exit.
  4. Optional pre-Phase-3 polish: unit tests for `assemble_flop_d_encoded` / `assemble_flop_d_one_hot`.
  5. Optional book polish: FAQ chapter as questions accumulate.

## Recent commits
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
