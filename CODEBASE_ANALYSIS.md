# Code Base Analysis
Live analysis of the Rust workspace as it currently stands. Updated whenever a slice materially changes the workspace.

## Snapshot
- **Workspace:** single crate `anvil` (no Cargo workspace; flat layout).
- **Edition:** 2021.
- **Targets:** one binary (`anvil`), one library (`anvil`), one example (`generate_one`), one integration test (`pipeline`).
- **External deps:** `rand`, `rand_chacha`, `clap`, `serde`, `serde_json`, `thiserror`, `anyhow`. `insta` (dev) reserved for snapshot tests.
- **MSRV:** not yet pinned. Whatever stable Rust is current.

## Module map

```
src/
├── main.rs           CLI entry point. Parses Cli (clap), constructs Config,
│                     runs Generator, writes stdout or per-file output with
│                     manifest.json. Owns no domain logic.
│
├── lib.rs            Public surface: re-exports Config, Generator, Module.
│
├── config.rs         Config struct (knobs), Default impl, validate(),
│                     CLI Overrides struct, ConfigError taxonomy.
│
├── ir/
│   ├── mod.rs        Re-exports types::* and the validate module.
│   ├── types.rs      Core types: Module, Port, Direction, Node, GateOp,
│   │                 Flop, ResetKind, DepSet, Design.
│   │                 Phase 1: PrimaryInput/Constant/FlopQ/Gate node kinds.
│   │                 Flop/FlopQ exist but are unused until Phase 2.
│   └── validate.rs   Module invariant checker (operand defined,
│                     drive count == 1, flop D filled, dep-set non-empty).
│                     Width-rule per-gate validation: TODO.
│
├── gen/
│   ├── mod.rs        Generator struct (rng + cfg), generate_module(),
│   │                 generate_design() (Phase 5+ stub).
│   ├── module.rs     Leaf-module top-level generator: pick port counts,
│   │                 pick widths, seed signal pool with primary inputs,
│   │                 build a cone per primary output.
│   ├── cone.rs       Fanin-cone recursion.
│   │                 Public: build_cone_with_retry, build_cone.
│   │                 Helpers: pick_terminal (with lazy width-adapter
│   │                 fallback), make_width_adapter, pick_gate,
│   │                 input_widths_for, violates_anti_collapse, node_deps.
│   │                 Phase 1 only: no flop branch (flop_prob defaults to 0).
│   └── pool.rs       SignalPool: list of (node, width, deps) entries.
│                     Methods: add, of_width, iter, is_empty.
│                     Cloneable for snapshot/rewind during retry.
│
└── emit/
    ├── mod.rs        Re-exports to_sv.
    └── sv.rs         IR → String pretty-printer. Assumes invariants hold.
                      No validation. No formatting choice beyond fixed
                      4-space indent and stable name scheme.
```

## Dependency direction
```
main  →  lib  →  gen  →  ir
                  │       ↑
                  ↓       │
                 emit ────┘
```

`ir` is a leaf. `gen` and `emit` both depend on `ir` but not on each other. This permits independent unit-testing of `emit` against hand-built IRs.

## Phase coverage map

| Phase | Status        | Code touched | Notes |
|-------|---------------|--------------|-------|
| 0 — Scaffolding              | done         | All files (initial) | `cargo check`, `cargo test`, `cargo clippy -D warnings`, `cargo fmt --check` all clean locally. |
| 1 — Combinational MVP        | in progress  | `gen/cone.rs`, `gen/module.rs`, `emit/sv.rs`, `gen/pool.rs` | Cone recursion functional with lazy width-adapter; remaining: per-gate width validator, unit tests, Verilator/Yosys smoke. |
| 2 — Sequential               | not started  | `gen/cone.rs`, `emit/sv.rs`, `ir/types.rs` (Flop already typed) | Flop worklist + `always_ff` emitter. |
| 3 — Sharing                  | not started  | `gen/cone.rs`, `gen/pool.rs` | `share_prob` knob exists; recursion hook absent. |
| 4 — Structured combinational | not started  | `gen/cone.rs`, `emit/sv.rs` | New GateOp variants + emitter arms. |
| 5 — Hierarchy                | not started  | new `gen/hierarchy.rs`; `Design` already typed | Library + on-demand sourcing. |
| 6 — Parameterization         | not started  | new module | Significant extension to IR (parameter env). |
| 7 — Advanced motifs          | not started  | various | Memories, FSMs, optional multi-clock. |

## Invariants currently enforced

In code (constructors / generator):
- `Config::validate()` rejects out-of-range knobs.
- `Generator::new()` seeds RNG deterministically.
- `gen::module::generate_leaf_module` produces port counts within knob ranges.
- `gen::cone::build_cone_with_retry` retries up to 4× on empty-dep-set cone roots.
- `gen::cone::pick_terminal` prefers matching-width pool entries with non-empty deps; on no width-match, builds a width-adapter (`make_width_adapter`) from the widest dep-bearing pool entry; only emits a constant when the entire pool has empty deps.
- `gen::cone::make_width_adapter` produces a Slice (when source > target), a single Concat (when source × N == target), or Concat-then-Slice (when source × N > target). Deps propagate from the source.
- `gen::cone::violates_anti_collapse` rejects `x ^ x`, `x - x`, `x == x`, `x != x`, `mux(s, a, a)`.
- `gen::cone::pick_gate` only offers comparison ops when the parent target width is 1.

In `ir::validate::validate`:
- Operand `NodeId`s in range.
- Each output port has exactly one drive.
- Every flop has a `d` set.
- Output-cone root has non-empty dep-set.
- **Missing:** per-gate operand-width validation (marked TODO; Phase 1 work).

## Testing surface

- `tests/pipeline.rs` — 20-seed cross-seed generation + validation + reproducibility.
- No unit tests yet inside source modules. Phase 1 work to add them.
- No external smoke tests wired up. Phase 1 exit gate requires Verilator-lint pass on a representative seed range.

## Known weaknesses (visible in code today)

- `gen::cone::input_widths_for` for `Slice` and `Concat` returns placeholder widths. `Slice` and `Concat` are not currently selectable in `pick_gate` (only used by `make_width_adapter`, which constructs them directly with correct widths). Properly wire `input_widths_for` when Phase 4 makes them pickable.
- `emit::sv::render_gate` for `Concat` joins operand names with commas (correct SV); the IR currently stores no per-operand width because the adapter only ever uses a single replicated source. When variadic Concat with mixed widths becomes a real motif (Phase 4), the IR will need per-operand width.
- `ir::validate::validate` does not check per-gate operand widths. This is the most important missing safety net and is the next Phase 1 task.

## Build hygiene
- `cargo fmt --all --check` — clean.
- `cargo clippy --all-targets -- -D warnings` — clean.
