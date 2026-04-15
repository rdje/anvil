# Changes
Fully detailed change history. Newest entries at the top. One entry per commit.

---

## 2026-04-15-0007 — Elevate mdBook to equal-standing live doc in session recovery

**What changed**
- `SESSION_BOOTSTRAP.md`: reworded the mdBook entry in the bootstrap reading order. The book is now described explicitly as a live doc, not reference material, with language stating that a session skipping the book will make locally-correct but globally-wrong decisions.
- `COMMIT.md`:
  - Reworded the `book/` files-involved section: the mdBook is "a live doc of equal standing" and is "load-bearing" for session recovery.
  - Item 9 of the 12-item pre-commit checklist now explicitly states the mdBook's role and mandates adding permanent design decisions there, not just in commit messages.
- `README.md`: the ramp-up reading list entry for `book/` now states equal standing and the recovery-requires-reading-it stance. Follow-up sentence clarifies the book is part of the status-authority set, not adjacent to it.

**Why**
The user pointed out that the mdBook is part of the context-rebuild surface for post-crash / post-session-loss recovery, not a separate reference tier. The short-form live docs (`README`, `ROADMAP`, `MEMORY`, `CHANGES`, `DEVELOPMENT_NOTES`, `CODEBASE_ANALYSIS`, `USER_GUIDE`, `COMMIT`) carry *operational* state; the mdBook carries *design* context — why the generator is shaped the way it is, what has been deliberately rejected, what the motif catalogue looks like. A session that reconstructs operational state without the design context will make decisions that are locally coherent but globally wrong.

This slice makes the mdBook's recovery role explicit in three places (`SESSION_BOOTSTRAP.md`, `COMMIT.md` preamble + checklist, `README.md` reading list) so no future session can miss it.

**Validation**
- Documentation-only slice; no source changes.
- `cargo check`, `cargo test`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --all --check`: all still clean (no code touched).

**Impact**
- The 12-item pre-commit checklist now has an explicitly strengthened item 9 that closes a gap where design decisions might have landed in commit messages and `DEVELOPMENT_NOTES.md` but not in the mdBook.
- New sessions reading `SESSION_BOOTSTRAP.md` will not mistake the mdBook for optional reading.

**Files touched**
`SESSION_BOOTSTRAP.md`, `COMMIT.md`, `README.md`, `MEMORY.md`, `CHANGES.md`.

**Commit hash:** _to be filled in after this commit_

---

## 2026-04-15-0006 — Live-doc catch-up: capture flop-mux rationale + tighten commit workflow

**Commit hash:** `a1a9ea9`

**What changed**
- `DEVELOPMENT_NOTES.md`:
  - Added "Flop-D mux motifs" and "Q-exclusion contract" to the Core design decisions recap.
  - Added rejected alternative: `always_comb` + `case` for Encoded-mux flop D (why chained ternary wins).
  - Added rejected alternative: M = 1 mux arm (why it's excluded by design).
  - Added gotcha: module-level `#![allow(clippy::too_many_arguments)]` in `src/gen/cone.rs` with rationale.
  - Added calibration notes for `flop_mux_encoding_prob = 0.5` and `flop_qfeedback_prob = 0.5`.
  - Documented the QFeedback-in-Encoded design choice (replace `data_0` with Q) and the rejected alternative (extra (M+1)th entry).
- `MEMORY.md`:
  - Recent-commits list updated with `10090c2`.
  - Open-questions list updated with the `flop_mux_encoding_prob` calibration entry and the ternary-vs-case revisit trigger.
- `COMMIT.md`:
  - Added a non-negotiable 12-item pre-commit checklist. Every item is listed explicitly. The checklist makes skipping any live-doc update a visible workflow violation rather than a silent drift.

**Why**
Prior to this slice, the last two commits (`47675df` and `10090c2`) landed load-bearing design rationale — why M=1 is excluded, why chained ternary over `case`, why the Q-exclusion contract — that was captured in `CHANGES.md` and `book/src/sequential.md` but not in `DEVELOPMENT_NOTES.md`, which is the contributor-facing design-decision ledger. `MEMORY.md`'s recent-commits list was also one commit behind. The user flagged the slippage.

The fix has two parts: (1) a factual catch-up of the missed content, and (2) a structural fix to the commit workflow itself — an explicit 12-item pre-commit checklist in `COMMIT.md` that makes every live-doc gate impossible to skip implicitly.

**Validation**
- Documentation-only slice; no source changes.
- `cargo check`, `cargo test`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --all --check`: all still clean (no code touched).

**Impact**
- Future sessions can reconstruct the full design rationale from `DEVELOPMENT_NOTES.md` alone, without having to archaeologize across commit messages.
- The pre-commit checklist makes workflow compliance auditable: each item is either affirmatively satisfied or the commit does not proceed.

**Files touched**
`DEVELOPMENT_NOTES.md`, `MEMORY.md`, `COMMIT.md`, `CHANGES.md`.

---

## 2026-04-15-0005 — Encoded-select flop mux (chained ternary) alongside one-hot

**Commit hash:** `10090c2`

**What changed**
- `src/ir/types.rs`:
  - Replaced `Flop.arms: Vec<MuxArm>` with `Flop.mux: FlopMux`.
  - `FlopMux` enum: `None` (M=0), `OneHot(Vec<MuxArm>)`, `Encoded { sel: NodeId, data: Vec<NodeId> }`.
- `src/config.rs`:
  - New knob `flop_mux_encoding_prob` (default `0.5`): per-flop probability of using the encoded-select style instead of one-hot.
- `src/gen/cone.rs`:
  - New `drain_flop_encoded`: builds one select sub-cone of width `ceil(log2(M))` and M (or M-1 for QFeedback) data sub-cones, assembles D as a chained ternary over `Eq(sel, k)` with a `0` or `Q` fall-through.
  - New `drain_flop_one_hot`: extracts the previous one-hot assembly into its own function.
  - New `assemble_flop_d_encoded`, `make_constant`, `make_eq_const`, `make_mux`, `ceil_log2` helpers.
  - Renamed `assemble_flop_d` → `assemble_flop_d_one_hot`.
  - Per-flop dispatch in `drain_flop_worklist`: picks encoded or one-hot via `cfg.flop_mux_encoding_prob`.
  - Module-level `#![allow(clippy::too_many_arguments)]` to silence the lint on helpers that legitimately thread many context refs.
- `book/src/sequential.md`: documents both encoding styles, the 2×2 style-kind matrix, and the QFeedback+Encoded special case where index 0 is replaced by Q.
- `USER_GUIDE.md`: documents `--flop-mux-encoding-prob`.
- `CODEBASE_ANALYSIS.md`: module map, helper list, and invariants updated for the new drain path.
- `MEMORY.md`: state, next-up, recent commits refreshed.

**Why**
The user asked for an encoded-select variant alongside the existing one-hot, with the Q-feedback case routing Q on `sel == 0` and on out-of-range values. Both styles correspond to real synchronous-design shapes (one-hot for arbitration-driven register banks, encoded for opcode/address/state-selected registers) and exercise different synthesis paths. Picking per-flop preserves motif diversity within a single generated module.

**Validation**
- `cargo check`, `cargo test` (2 tests pass, ~2s for 20-seed sweep), `cargo clippy --all-targets -- -D warnings`, `cargo fmt --all --check`: all clean.
- Visual inspection with `--seed 5 --max-depth 2 --flop-prob 1.0` shows chained ternaries in the output: `(eq_k) ? data_k : (eq_{k-1}) ? data_{k-1} : ... : fall_through`, confirming the encoded-mux assembly.

**Impact**
- Phase 1 now emits two distinct flop motifs. Motif diversity is no longer bound by encoding style.
- The `FlopMux` enum carries introspective information about each flop's mux shape, useful for future debugging/inspection tooling even though it is not load-bearing for emission today.

**Files touched**
`src/ir/types.rs`, `src/config.rs`, `src/gen/cone.rs`, `book/src/sequential.md`, `USER_GUIDE.md`, `CODEBASE_ANALYSIS.md`, `MEMORY.md`, `CHANGES.md`.

---

## 2026-04-15-0004 — M-to-1 one-hot mux flops with two motifs

**Commit hash:** `47675df`

**What changed**
- `src/ir/types.rs`:
  - New `FlopKind` enum: `ZeroDefault` (D = 0 when no select fires) and `QFeedback` (D = Q when no select fires).
  - New `MuxArm { data: NodeId, sel: NodeId }` representing one arm of a flop's input mux.
  - `Flop` gains `kind: FlopKind` and `arms: Vec<MuxArm>` fields.
- `src/gen/cone.rs`:
  - `build_cone_with_retry` and `build_cone` gain an `exclude: Option<NodeId>` parameter threaded into `pick_terminal`. Used to forbid this flop's own Q from being a leaf in any of its data or select sub-cones.
  - `pick_mux_arm_count` returns M from {0, 2, 3, ..., max_mux_arms}. M = 1 excluded by design (a 1-arm mux is a wire).
  - `drain_flop_worklist` rewritten:
    - For M = 0: D = recursive cone of width N (no mux).
    - For M >= 2: build M data sub-cones (width N) + M select sub-cones (1-bit), every one a recursion point. Assemble `D = OR_i({N{sel_i}} & data_i)`, plus `({N{~(OR sel_i)}} & Q)` for `QFeedback`.
  - New helpers: `assemble_flop_d`, `replicate_to_width` (N-fold Concat of a 1-bit signal), `make_and`, `make_none_selected`, `or_reduce_terms`.
  - `build_flop_leaf` picks a random `FlopKind` per flop (`flop_qfeedback_prob` knob).
- `src/config.rs`:
  - New knobs: `min_mux_arms` (default 1, becomes effective floor of 2 inside `pick_mux_arm_count`), `max_mux_arms` (default 4), `flop_qfeedback_prob` (default 0.5).
  - `Config::validate` checks the mux-arm range and the new probability.
  - New error variant `MuxArmsRange`.
- `src/gen/module.rs`: passes `None` exclusion for output cones.
- `book/src/sequential.md`: documents M=0 vs M>=2 cases, both flop kinds, and the Q-exclusion contract enforced via `exclude: Option<NodeId>`.
- `USER_GUIDE.md`: documents `--min-mux-arms`, `--max-mux-arms`, `--flop-qfeedback-prob` knobs.
- `CODEBASE_ANALYSIS.md`: module map updated for new helpers; invariants list updated.
- `MEMORY.md`: state, next-up, recent commits refreshed.

**Why**
The user specified the precise flop motif `anvil` should generate:
1. M ∈ {0, 2, 3, ...}. M = 0 means no mux, D recurses directly.
2. For M >= 2: each of the M data inputs (width N) is a recursion point; each of the M 1-bit select bits is a recursion point. Selects are one-hot (a design contract, not enforced).
3. Two kinds: `ZeroDefault` (D = 0 on no-select) and `QFeedback` (D = Q on no-select).
4. The flop's own Q is forbidden from feeding any of its data or select sub-cones — the *only* permitted Q→D path is the explicit Q-feedback term in `QFeedback`.

This produces RTL that resembles real synchronous datapath idioms (one-hot-controlled register banks, holding registers, etc.) rather than generic register-of-arbitrary-cone shapes.

**Validation**
- `cargo check`, `cargo test`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --all --check`: all clean.
- Visual inspection of `seed=3, max-depth=2, flop-prob=1.0` confirms:
  - `assign w_X = {bit, bit, ..., bit};` (replicate sel_i to N bits)
  - `assign w_Y = w_X & data_i;` (mask)
  - `assign w_Z = w_A | w_B;` (OR-reduce arm terms)
  - For `QFeedback`: extra `~(OR of sels)` term ANDed with Q.

**Impact**
- Generated flop motifs now match a real-world synchronous-design pattern.
- Tests run slower (~3-4s for the 20-seed sweep vs ~0.04s previously) due to the M+M sub-cone fan-out per flop. Tolerable; tunable via `max_mux_arms` and `max_flops_per_module`.

**Files touched**
`src/ir/types.rs`, `src/config.rs`, `src/gen/cone.rs`, `src/gen/module.rs`, `book/src/sequential.md`, `USER_GUIDE.md`, `CODEBASE_ANALYSIS.md`, `MEMORY.md`, `CHANGES.md`.

---

## 2026-04-15-0003 — Fold flops into the cone recursion (single-clock synchronous discipline)

**Commit hash:** `4317c82`

**What changed**
- `src/gen/cone.rs`:
  - New `FlopWorklist` type alias (`Vec<FlopId>`).
  - `build_cone` now decides between `Gate` and `Flop` at each non-leaf node, gated by `cfg.flop_prob` and `cfg.max_flops_per_module`.
  - New `build_flop_leaf`: allocates a `Flop`, pushes a `FlopQ` node, queues the flop for D-cone construction, returns Q as the leaf for the current cone.
  - New `drain_flop_worklist`: pops queued flops one at a time, recursively builds each D-cone with `build_cone_with_retry` (which itself may push more flops); loops to quiescence.
  - `build_cone_with_retry` now also snapshots/rewinds `m.flops` and the worklist.
  - All flops use `ResetKind::Async` unconditionally (single-CLK / single-RST_N discipline).
  - New `pick_reset_value` (50% zero, 25% all-ones, 25% random).
- `src/gen/module.rs`:
  - Reserves port id 0 for `clk` and 1 for `rst_n`. Sets `Module.clock` and `Module.reset`. Excludes them from the signal pool so cones cannot terminate at them.
  - Drains the flop worklist after building all output cones.
- `src/emit/sv.rs`:
  - Emits `logic [W-1:0] r_<id>;` for every flop.
  - Emits a single `always_ff @(posedge clk or negedge rst_n)` block containing all flops, with reset-branch initializing every flop and else-branch sequencing every flop's D.
  - Conditionally omits `clk`/`rst_n` from the port list when the module has no flops.
- `src/config.rs`:
  - `flop_prob` default raised to `0.15` (was `0.0`).
  - New knob `max_flops_per_module` (default `32`) capping flop count to bound generation time.
- `book/src/sequential.md`:
  - Reframed: flops are part of the same cone recursion, not a later phase.
  - New "Synchronous-design discipline" section spelling out the single-CLK / single-RST_N async constraint.
  - Updated example `always_ff` block.
- `ROADMAP.md`:
  - Phase 1 collapsed: combinational + sequential together. Old Phase 3/5/7 renumbered to new Phase 2/4/6.
- `USER_GUIDE.md`:
  - Updated `flop_prob` default.
  - Documented `max_flops_per_module` knob.
- `DEVELOPMENT_NOTES.md`:
  - Added "Synchronous-design discipline" as a core design decision.
- `CODEBASE_ANALYSIS.md`:
  - Updated module map for new cone helpers.
  - Updated phase coverage map (collapse + renumber).
  - Documented new construction-time invariants (flop allocation, single-clock, clk/rst_n exclusion from pool).
- `MEMORY.md`:
  - Recorded `c4668a2`.
  - Refreshed current state, next-up, open questions, known gaps.

**Why**
The user pointed out that artificially deferring flops to a later phase contradicts the recursion-as-core-principle stance: Q is just another leaf, D is just another sub-cone, the worklist is the same iterative shell that drives output cones. Folding sequential into Phase 1 also unlocks meaningful synthesis testing — purely combinational random RTL is far less representative of real designs than mixed sequential/combinational.

The single-CLK / single-RST_N (async, active-low) constraint matches real fully-synchronous design practice. Enforcing it by construction (no IR field for per-flop clock or polarity) means no random choice can violate it.

**Validation**
- `cargo check --all-targets`, `cargo test` (2 tests pass), `cargo clippy --all-targets -- -D warnings`, `cargo fmt --all --check`: all clean.
- `cargo run -- --seed 7`: produces a module with `always_ff @(posedge clk or negedge rst_n)`, all flops in one block, async-reset to per-flop reset values.
- IR validator passes across the 20-seed sweep with flops enabled.

**Impact**
- Phase 1 is now a meaningful single-module MVP rather than a combinational stub.
- Generated RTL now includes registered state, which is far more representative for downstream synthesis tooling.

**Files touched**
`src/config.rs`, `src/gen/cone.rs`, `src/gen/module.rs`, `src/emit/sv.rs`, `book/src/sequential.md`, `ROADMAP.md`, `USER_GUIDE.md`, `DEVELOPMENT_NOTES.md`, `CODEBASE_ANALYSIS.md`, `MEMORY.md`, `CHANGES.md`.

---

## 2026-04-15-0002 — Elevate "recursion is the core principle" to load-bearing status

**Commit hash:** `c4668a2`

**What changed**
- `README.md`: rewrote the project-objective section as **three** load-bearing principles, with recursion as the first. Recursion is now stated explicitly as the default algorithmic shape for any non-trivial generation step.
- `book/src/core-idea.md`: prepended a "The single guiding principle: recursion" section before the existing thesis. States that recursion is the default; iteration is the exception (flop worklist, per-output driver loop) and exists only to *kick off* recursive cone construction. Anchors the correctness argument: each recursive call carries its own constraints, which is what makes "valid by construction" hold.
- `DEVELOPMENT_NOTES.md`: added recursion as the first entry in the "Core design decisions" recap, with a pointer to the new book section.
- `MEMORY.md`: recorded `5f6022f` (the previous slice's commit hash).

**Why**
The user explicitly stated: "By design, anvil shall be heavily recursive — recursion is its core principle." The design as implemented already follows this, but the docs only hinted at it. Elevating it to first-class status ensures future contributors do not casually replace recursion with iteration in places where the recursion structure is what guarantees invariant preservation.

**Validation**
- Docs-only slice; no code changes.
- `cargo check`, `cargo test`: still clean (no source touched).

**Impact**
- Future PRs that introduce iterative scaffolding around generation logic should now expect to justify the choice against the "recursion is the default" principle.

**Files touched**
`README.md`, `book/src/core-idea.md`, `DEVELOPMENT_NOTES.md`, `MEMORY.md`, `CHANGES.md`.

---

## 2026-04-15-0001 — Initial scaffold + Phase 1 cone-adapter hardening

**Commit hash:** `5f6022f`

**What changed**
- Created Cargo project `anvil` with binary + library targets.
- Added `Cargo.toml` with deps: `rand`, `rand_chacha`, `clap` (derive), `serde`, `serde_json`, `thiserror`, `anyhow`.
- Added crate skeleton:
  - `src/lib.rs` — public re-exports (`Config`, `Generator`, `Module`).
  - `src/main.rs` — CLI (`--seed`, `--count`, `--out`, `--config`, `--dump-config`, knob overrides).
  - `src/config.rs` — `Config` struct, defaults, `validate()`, CLI overlay.
  - `src/ir/types.rs` — `Module`, `Port`, `Node`, `GateOp`, `Flop`, `DepSet`.
  - `src/ir/validate.rs` — IR invariant checker (safety net).
  - `src/gen/mod.rs` — `Generator` entry points, ChaCha8-seeded RNG.
  - `src/gen/module.rs` — leaf-module generator (N inputs, M outputs, cone per output).
  - `src/gen/cone.rs` — fanin-cone recursion with depth budget, anti-collapse rules, dep-set tracking, bounded retry on trivial cones.
  - `src/gen/pool.rs` — `SignalPool` for terminal selection.
  - `src/emit/sv.rs` — IR → SystemVerilog pretty-printer.
- Added `tests/pipeline.rs` — generates 20 seeds, asserts IR validation passes and SV output is non-empty; reproducibility test.
- Added `examples/generate_one.rs` — minimal library-usage example.
- Added live-doc set:
  - `README.md` — entry point.
  - `SESSION_BOOTSTRAP.md` — read-first on session recovery.
  - `ROADMAP.md` — 7-phase plan, exit criteria per phase.
  - `USER_GUIDE.md` — CLI, knobs, downstream verification.
  - `MEMORY.md` — operational continuity snapshot.
  - `CHANGES.md` (this file).
  - `DEVELOPMENT_NOTES.md` — engineering rationale.
  - `CODEBASE_ANALYSIS.md` — live workspace analysis.
  - `COMMIT.md` — commit workflow.
- Added mdBook design rationale at `book/`:
  - `core-idea.md`, `why-not-grammar.md`, `algorithm.md`, `ir.md`,
    `by-construction.md`, `synthesizability.md`, `non-triviality.md`,
    `sequential.md`, `sharing.md`, `hierarchy.md`, `knobs.md`,
    `architecture.md`, `non-goals.md`.
- Added `.gitignore` covering `/target`, `book-out`, `Cargo.lock`, swap files, and `git_message_brief.txt`.
- **Phase 1 hardening:** lazy width-adapter in `gen::cone::pick_terminal`. When the signal pool has no matching-width entry, build a Slice (or replicating Concat + Slice) from the widest available pool entry with non-empty deps, instead of falling back to a bare constant. Preserves dep-set propagation and resolves the seed-0 IR-validation failure where output cones were collapsing to constants.
- Added `gen::cone::make_width_adapter` helper.
- `gen::pool::SignalPool::iter()` exposed for adapter source selection.
- Clippy cleanups: `Config { seed, ..Default::default() }` patterns in tests/example; `u32::div_ceil` for adapter copy count.
- All `cargo fmt` corrections applied.

**Why**
Project bootstrap. The brainstorming session that preceded this slice converged on a circuit-graph-IR generator with by-construction validity, dep-set tracking for non-triviality, and explicit synthesizability-as-subset enforcement.

The lazy adapter fixes a Phase 1 bug surfaced on the first `cargo test` run: when randomly-chosen output port widths do not match any randomly-chosen input port width, the cone has no signal of the required width to terminate at, falls back to a constant, and the cone root's dep-set is empty. The validator correctly rejects this, but the bounded retry loop cannot recover because the pool composition does not change between attempts. The adapter resolves this structurally — any output width can now reach an input via Slice/Concat — without weakening the by-construction discipline.

**Validation**
- `cargo check --all-targets` clean.
- `cargo test`: 2 tests pass (`generates_valid_modules_across_seeds` over seeds 0..20, `reproducibility` byte-identical for seed 12345).
- `cargo clippy --all-targets -- -D warnings`: clean.
- `cargo fmt --all --check`: clean.
- `cargo run -- --seed 42`: produces a 4-output, 3-input module with a coherent assign net (visual spot-check).
- `cargo run -- --seed 7 --count 5 --out /tmp/anvil_out`: 5 .sv files + manifest.json written.
- External smoke tests (Verilator, Yosys): tools not installed locally; smoke runs are deferred until the dev environment provides them or CI is wired.

**Impact**
- Phase 0 (Scaffolding) exit criteria met: `cargo build` and `cargo test` pass.
- Phase 1 (Combinational MVP) is in progress: cone recursion functional and dep-set-correct across the seed sweep; remaining Phase 1 work is per-gate width-rule validation in `ir::validate`, unit tests inside source modules, and Verilator-lint smoke once available.
- `CODEBASE_ANALYSIS.md` "Known weaknesses" item #1 is resolved by this slice.

**Files touched**
All files in the repository (initial creation), plus subsequent edits to `src/gen/cone.rs`, `src/gen/pool.rs`, `tests/pipeline.rs`, `examples/generate_one.rs`.
