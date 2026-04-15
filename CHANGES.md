# Changes
Fully detailed change history. Newest entries at the top. One entry per commit.

---

## 2026-04-15-0002 — Elevate "recursion is the core principle" to load-bearing status

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

**Commit hash:** _to be filled in after this commit_

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
