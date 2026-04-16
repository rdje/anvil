# Memory
Compact, operational continuity snapshot. Read on session bootstrap. Keep only what is actionable.

## Current state
- **Phase:** Phase 0 done. Phase 1 (Single-module MVP) effectively feature-complete pending Verilator-lint smoke. Phase 2 (Signal sharing / DAG cones) in progress with default-on.
- **Last completed slice:** Recipe for the factorization dial (docs only). See `CHANGES.md` entry `2026-04-16-0047`. `book/src/recipes.md` gains a paste-and-run sweep over `--factorization-level none..e-graph` with real seed-42 gate counts and a layer-by-layer reading of the deltas. Addresses the "littered with examples" book doctrine.
- **Prior slice:** Commutative normalization + factorization-level dial. See `CHANGES.md` entry `2026-04-16-0046`. Layer 3 of the factorization chain lands: commutative ops (`And`/`Or`/`Xor`/`Add`/`Mul`) sort operands before intern, so `a+b` and `b+a` dedupe. New `FactorizationLevel` enum with 8 positions (`none → cse → operand-unique → commutative → associative → constant-fold → peephole → e-graph`), default `e-graph` (theoretical ceiling; clamps to highest implemented layer via `effective()`). CLI flag `--factorization-level`. 39 unit + 15 integration = 54 tests. Book Rule 21b + 21c landed. Aspirational levels (associative/constant-fold/peephole/e-graph) compile without behavioural surprise — future slices will activate them for users already at those levels.
- **Prior slice:** Operand-uniqueness knob (`--operand-duplication-rate`). See `CHANGES.md` entry `2026-04-16-0045`. New knob `operand_duplication_rate: f64 ∈ [0.0, 1.0]`, default 0.0 → strict Add/Mul operand uniqueness. `violates_anti_collapse` now checks Add/Mul duplicates when knob < 1.0; And/Or/Xor always strict (algebraic). `pick_signals_with_dup_rate` helper for pool-mode linear-combination. **User coined "full factorization"** = CSE (NodeId uniqueness across AST) + operand-uniqueness (no NodeId twice inside one gate) — both now enforced at default. 49 tests pass. Residual 0.09% duplicates in recursive linear-combination path (CSE-collapse of sub-cones); follow-up to reach 0%.
- **User doctrine (logged in memory):** NodeId = identity of an expression. Full factorization = no expression / sub-expression / sub-sub-expression ever duplicated — every expression has a unique NodeId. Beyond today's syntactic CSE; reaches algebraic equivalence (next-level work: commutative normalization, associative flattening, constant folding). Target for future slices.
- **Prior slice:** `--trace debug` is now strictly more verbose than `high`; `off` aliased as `none`. See `CHANGES.md` entry `2026-04-16-0044`. New `trace_verbose!` macro + `TRACE_DEBUG` atomic guard on `src/lib.rs`. `Module::intern_gate` / `intern_constant` emit `🔗 new` and `♻️ reuse` events — every node entering the IR is traceable. `pick_gate` return traced in both recursive and interleaved paths with depth + width. CLI: `--trace none` default (was `off`, kept as alias). Empirical line counts at seed 42: none=0, low=5, medium=141, high=3779, debug=8241 (+4462 strict super-set). 49 tests pass.
- **Prior slice:** Zero orphans: Rule 18 enforced construction-time. See `CHANGES.md` entry `2026-04-16-0043`. build_cone snapshots m.nodes/flops/pool/worklist/dedup-tables before operand construction; anti-collapse rejection rolls back. process_signal_frame (interleaved) can't snapshot per-gate so it uses an existing operand as anti-collapse fallback (no new node). GraphFirst retired as default; silently aliased to Interleaved. Safety-net audit warns if any orphan survives. Emitter reverts to dumb serialiser per doctrine. 49 tests, 0 orphans across 4 strategies × 6 seeds. **Known gap (next slice):** trace doesn't show "who requested this new gate" — build_cone and process_signal_frame need op-pick trace events with requester context.
- **Prior slice:** IR chapter refresh + future-extensions roadmap (docs only). See `CHANGES.md` entry `2026-04-16-0042`. `book/src/ir.md` gets (1) refreshed `Module` struct showing `gate_instances`, `const_instances`, `max_ast_instances`, `mux_arm_duplication_rate`; (2) new "Node construction" section documenting `intern_gate` / `intern_constant` signatures, cap semantics, snapshot/rollback contract; (3) naming section updated for Rule 12 (no more `w_N`/`r_N`); (4) new "Future extensions" section for parameters (Phase 5, Phase-4-dependent), synthesizable aggregates (four sub-paths with cost/payoff — packed cheap/emitter-only, unpacked arrays = Phase 6 memories, unpacked datapath + enums deprioritised), blocks as first-class IR. `ROADMAP.md` gains a Phase 5b aggregates entry. mdbook builds cleanly.
- **Prior slice:** Friendly docs — quick ref, naming refresh, recipe examples (docs only). See `CHANGES.md` entry `2026-04-16-0041`. `getting-started.md` sample output refreshed to match typed-per-kind naming (`slice_0`, `add_0`, `mul_0`) with a naming explanation. `knobs.md` gains a reassuring intro ("you don't need to read this top-to-bottom") and a Quick reference table of the ~13 most-touched knobs. `recipes.md` gets 6 new recipes: strict CSE (default); duplicated expressions (`--max-ast-instances`); pathological mux shapes (`--mux-arm-duplication-rate`); verify-a-knob via metrics grep; sweep-a-knob workflow with real `--flop-prob` values; trace levels with sample output. mdbook builds cleanly. 50 tests unchanged.
- **Prior slice:** Knob measurement doctrine + effectiveness map (docs only). See `CHANGES.md` entry `2026-04-16-0040`. `book/src/knobs.md` gains (1) a "Measurement doctrine" opening: no knob is privileged, every knob's effect must be empirically measurable via `Metrics` and/or `--trace`, with three landing requirements; (2) a dedicated "AST uniqueness / duplication" sub-section covering `max_ast_instances` and `mux_arm_duplication_rate`; (3) a knob-to-metric effectiveness map at the bottom listing which metric measures each knob, with *pending* entries flagging known gaps. No code changed. mdbook builds cleanly. 50 tests unchanged.
- **Prior slice:** Structural metrics (per-module observability). See `CHANGES.md` entry `2026-04-16-0039`. New module `src/metrics.rs` with `Metrics` struct + `compute(&Module)` post-hoc walker. Captures size, per-kind gate distribution, constant width/value distribution, mux shape (2-to-1 count + degenerate count), concat shape (replication vs heterogeneous), fanout (shared nodes + max + avg), flop kind/mux-shape distribution, AST-instance saturation. CLI flag `--metrics` → stderr JSON for single-module; multi-module runs always embed metrics in `manifest.json`. 3 new unit tests. 50 tests total. Knob effectiveness now empirically observable (seed 42 demonstration: `num_muxes_degenerate = 0` at default; flips to 1 at `--mux-arm-duplication-rate 1.0`). Live counters for attempt/miss/retry signals deliberately deferred to a future slice (most are already in `--trace high` events).
- **Prior slice:** Mux arm-duplication rate (Rule 22). See `CHANGES.md` entry `2026-04-16-0038`. New knob `mux_arm_duplication_rate: f64 ∈ [0.0, 1.0]`, default 0.0 (all arms distinct). Probabilistic uniqueness: at each arm pick, a candidate that duplicates an already-picked arm is kept with probability `rate` and rejected otherwise (8-try budget). 2-to-1 `make_mux` collapses `(s)?(x):(x) = x` when rate = 0.0; at any rate > 0.0 the upstream picker's decision stands. Applied at all pool-mode N-to-1 mux sites. Verified seed 42: 0 degenerate ternaries at default, 1 at rate 1.0. Book Rule 22 added. 47 tests pass.
- **Prior slice:** Construction-time CSE with tunable AST-instance cap (Rule 21). See `CHANGES.md` entry `2026-04-16-0037`. `Module::intern_gate` / `intern_constant` enforce a per-AST instance cap; default `max_ast_instances = 1` gives strict uniqueness (one RHS = one signal = one node). `GateOp` gains `Hash` derive. Every gate/constant creation in cone.rs routed through intern. Critical: `build_cone_with_retry` snapshots/restores `gate_instances` + `const_instances` alongside `m.nodes` — otherwise stale dedup entries would return wrong-kind nodes after rollback. CLI flag `--max-ast-instances`. Book Rule 21 added. 47 tests pass. Spot-check seed 42 confirms `slice_17 == 2'h2` now exists once (`eq_0`); at N=3 Eq count doubles.
- **Prior slice:** Emit `{N{expr}}` replication for same-operand Concat. See `CHANGES.md` entry `2026-04-16-0036`. `render_gate` for `Concat` detects all-operands-identical and emits the canonical SV replication form instead of the flat list. Clean-up triggered by user seeing `{eq_0, eq_0, … × 22}` in seed-42 output; now reads `{22{eq_0}}`. Semantics unchanged. Emitter unit test updated. 47 tests pass.
- **Prior slice:** UVM-style tracing (`--trace` / `--trace-file`). See `CHANGES.md` entry `2026-04-16-0035`. New deps: `tracing` + `tracing-subscriber`. CLI: `--trace <off|low|medium|high|debug>` default off, `--trace-file <path>`. Level mapping: low=INFO, medium=DEBUG, high/debug=TRACE. `#[instrument]` + explicit trace calls across `gen/module.rs`, `gen/cone.rs`, `emit/sv.rs` at the named control points (module start/done, strategy dispatch, motif forks, anti-collapse retry/exhausted, terminal tier picks 1-4, leaf-vs-recurse, emitter summary). Emojis at milestones only. Deterministic output — no timestamps/thread-ids/ANSI. Stdout stays byte-clean for SV. Release build compiles out below info. 47 tests pass; reproducibility holds byte-identical across trace levels. Block-level naming (`priority_encoder_0` flatten/hierarchical modes) still deferred.
- **Prior slice:** Typed per-kind naming in emitted SV (Rule 12 revised). See `CHANGES.md` entry `2026-04-16-0034`.
- **Doctrinal anchor:** user reinforced that generation must be rule-based (construction-time rules only, no post-hoc filters). Tree-shake / validator-as-gate are off the table. See `feedback_rules_first_generation.md` in session memory. This slice is the template: rule in catalog, invariant in picker.
- **Prior slice:** Rule 18 proposal + sample-output defect catalogue (docs only). See `CHANGES.md` entry `2026-04-15-0030`. New `priority_encoder_prob` knob (default 0.05) + CLI flag. `pick_priority_encoder_n` finds an N ∈ `[min_mux_arms, max_mux_arms]` with `ceil_log2(N) == target_width`, returns None if none fits. `assemble_priority_encoder` emits a chained ternary `req_0 ? 0 : req_1 ? 1 : ... : 0`. `build_priority_encoder_recursive` and `build_priority_encoder_pool` dispatch helpers. Three dispatch sites (build_cone / process_signal_frame / grow_pool_one_unit) with applicability-check-then-fall-through semantics. Book Rule 17 added. 1 new integration test. 29 unit + 15 integration = 44 tests. See `CHANGES.md` entry `2026-04-15-0029`.
- **Doctrinal note (deferred):** the motif-trait refactor is explicitly deferred per user direction. After landing several more block motifs, revisit to factor the copy-paste pattern into a `Motif` trait + registry.
- **Conceptual advance this session:** the operators-vs-blocks distinction is now load-bearing doctrine. Operators (associative primitives) generalize by arity; blocks (mux, flop, future memory/FSM) generalize by structural parameters (port counts, encoding choices, feedback topology). Subsequent slices use this framework.
- **Next up (closing small-to-medium Phase 3+ motifs first, per user direction):**
  0. **Source-of-generation defect fixes — all landed.** (a) coefficient-width clamp — Rule 19 / `2026-04-15-0031`. (b) dep-bearing select/req/LHS/value — Rule 20 / `2026-04-16-0032`. (c) N-arity operand-multiset distinctness + OR-reduce dedup — Rule 8 extended / `2026-04-16-0033`. Next: (d) Rule 18 enforcement = α (construction-time demand-driven) — rework graph-first to demand-driven construction; accept that it may converge toward interleaved-with-symmetric-sharing. Identical one-hot arms at assembly level (distinct from the OR-reduce dedup that absorbs the downstream effect) remains a possible future slice if seen in output after (d).
  1. **case/casez structured combinational blocks (medium).** A block that takes a select signal (1..N bit wide) and emits `always_comb case (sel) ... endcase` (or equivalent chained-ternary if we stay in expression land). Similar to the encoded mux but the emitted SV uses a `case` statement with explicit branches. Distinct synthesizer code path from chained-ternary muxes.
  2. **Memories (medium).** Inferrable single-port / simple-dual-port memory patterns (`reg [W-1:0] mem [0:DEPTH-1]` with an always_ff block driving read/write). Knob for depth range.
  3. **FSMs (medium-large).** Explicit state encoding (binary / one-hot / gray), transition logic, optional output logic. The first real multi-part block motif.
  4. After the above, revisit the motif-trait refactor (the copy-paste pattern will then cover ~7-8 block motifs, enough to extract the right abstraction).
  5. Large-scope deferred: hierarchy (a), parameterization (f).
  6. Blocked on external tooling: Verilator-lint smoke, Yosys smoke.

## Recent commits
- `c9c2f98` — Commutative normalization + factorization-level dial.
- `5a9b477` — Operand-uniqueness knob (--operand-duplication-rate).
- `2ec33b7` — --trace debug strictly more verbose than high; off→none alias.
- `b78550d` — Zero orphans: Rule 18 enforced construction-time.
- `186db2b` — IR chapter refresh + future-extensions roadmap (docs only).
- `3af6001` — Friendly docs: quick ref, naming refresh, recipe examples (docs only).
- `7c8fa2f` — Knob measurement doctrine + effectiveness map (docs only).
- `6fb5b9b` — Structural metrics (per-module observability).
- `d2aefba` — Mux arm-duplication rate (Rule 22).
- `f425657` — Construction-time CSE with tunable AST-instance cap (Rule 21).
- `88212f7` — Emit {N{expr}} replication for same-operand Concat.
- `b533288` — UVM-style tracing (--trace / --trace-file).
- `26f90a3` — Typed per-kind naming in emitted SV (Rule 12 revised).
- `3544a0c` — N-arity anti-collapse + OR-reduce dedup (Rule 8 extended).
- `6a9daf5` — Dep-bearing source at elaboration-sensitive positions (Rule 20).
- `92d43f8` — Coefficient fits operand width (Rule 19).
- `e6850fc` — Rule 18 proposal + sample-output defect catalogue (docs only).
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
