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
│   └── validate.rs   Module invariant checker: operand defined,
│                     drive count == 1, flop D filled, dep-set non-empty,
│                     per-gate arity + operand-width + output-width rules
│                     for every GateOp variant. Has inline unit tests
│                     covering valid and invalid hand-built IRs.
│
├── gen/
│   ├── mod.rs        Generator struct (rng + cfg), generate_module(),
│   │                 generate_design() (Phase 5+ stub).
│   ├── module.rs     Leaf-module top-level generator: pick port counts,
│   │                 pick widths, seed signal pool with primary inputs,
│   │                 build a cone per primary output.
│   ├── cone.rs       Fanin-cone recursion (combinational + sequential).
│   │                 Public: FlopWorklist alias, build_cone_with_retry,
│   │                 drain_flop_worklist, build_cone (all carry an
│   │                 `exclude: Option<NodeId>` for Q-feedback isolation).
│   │                 Per-flop drain: drain_flop_one_hot, drain_flop_encoded.
│   │                 Helpers: build_flop_leaf, pick_reset_value,
│   │                 pick_mux_arm_count (M ∈ {0, 2..=max}),
│   │                 ceil_log2, assemble_flop_d_one_hot, assemble_flop_d_encoded,
│   │                 make_constant, make_eq_const, make_mux,
│   │                 replicate_to_width, make_and,
│   │                 make_none_selected, or_reduce_terms, pick_terminal
│   │                 (with lazy width-adapter fallback and exclusion
│   │                 filter), make_width_adapter, pick_gate,
│   │                 input_widths_for, violates_anti_collapse, node_deps.
│   │                 Q is a leaf in the current cone; D opens either
│   │                 a direct cone (M=0), a one-hot OR-of-masks mux
│   │                 (M>=2, OneHot), or a chained-ternary encoded
│   │                 mux (M>=2, Encoded) via the worklist.
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
| 1 — Single-module MVP        | in progress  | `gen/cone.rs`, `gen/module.rs`, `emit/sv.rs`, `gen/pool.rs` | Combinational + sequential cone recursion functional; flop worklist drained; `always_ff` emitted; single CLK + single RST_N (async). Remaining: per-gate width validator, unit tests, Verilator/Yosys smoke. |
| 2 — Sharing                  | not started  | `gen/cone.rs`, `gen/pool.rs` | `share_prob` knob exists; recursion hook absent. |
| 3 — Structured combinational | not started  | `gen/cone.rs`, `emit/sv.rs` | New GateOp variants + emitter arms. |
| 4 — Hierarchy                | not started  | new `gen/hierarchy.rs`; `Design` already typed | Library + on-demand sourcing. |
| 5 — Parameterization         | not started  | new module | Significant extension to IR (parameter env). |
| 6 — Advanced motifs          | not started  | various | Memories, FSMs, optional multi-clock. |

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
- `gen::cone::build_flop_leaf` allocates `Flop` (with random `FlopKind`) and `FlopQ` together; `Flop.q` always points at the new `FlopQ` node; `Flop.d` and `Flop.mux` are filled later by `drain_flop_worklist`.
- All flops use `ResetKind::Async` unconditionally (single-CLK / single-RST_N synchronous discipline).
- `pick_mux_arm_count` returns M from {0, 2, 3, ..., max_mux_arms}. M = 1 is excluded by design.
- `drain_flop_worklist` constructs each flop's D as one of:
  - (a) a direct recursive cone when M=0;
  - (b) one-hot mux `OR_i({N{sel_i}} & data_i)` (+ `{N{none_selected}} & Q` for `QFeedback`) for the OneHot style;
  - (c) encoded-select chained-ternary mux over `Eq(sel, k)` with a fall-through of 0 (ZeroDefault) or Q (QFeedback) for the Encoded style. QFeedback+Encoded replaces `data_0` with Q.
  The style is picked per-flop via `cfg.flop_mux_encoding_prob`. All sub-cones forbid this flop's Q from being a leaf via the `exclude` parameter.
- `pick_terminal` filters out the excluded `NodeId` from every candidate set (matching-width, dep-bearing, fallback adapter source).
- `gen::cone::pick_node_kind` (inline in `build_cone`) gates flop selection on `m.flops.len() < cfg.max_flops_per_module` to bound generation cost.
- `gen::module::generate_leaf_module` reserves port id 0 for `clk` and 1 for `rst_n`. Neither is added to the signal pool, so cones cannot terminate at them.

In `ir::validate::validate`:
- Operand `NodeId`s in range.
- Each output port has exactly one drive.
- Every flop has a `d` set.
- Output-cone root has non-empty dep-set.
- Per-gate arity: each `GateOp` variant has a fixed or variadic-with-min operand count.
- Per-gate operand widths: `And/Or/Xor/Add/Sub/Mul` / `Not` require operand width == output width; `Mux` requires sel 1-bit + two data operands at output width; `Eq/Neq/Lt/Gt/Le/Ge` require equal-width operands + 1-bit output; `RedAnd/RedOr/RedXor` require 1-bit output; `Shl/Shr` require value operand at output width (shift amount unconstrained); `Slice{hi,lo}` requires `hi >= lo`, `out_w == hi-lo+1`, source width > `hi`; `Concat` requires sum of operand widths == output width.

## Testing surface

- `tests/pipeline.rs` — 20-seed cross-seed generation + validation + reproducibility.
- `src/ir/validate.rs` — 8 inline unit tests covering valid modules and each class of rejection (operand width mismatch, mux selector width, Eq output width, Concat sum, Slice out-of-bounds, wrong arity, variadic replicate Concat).
- `src/gen/cone.rs` — 6 inline unit tests covering `ceil_log2` correctness (incl. 62-value sweep), `pick_mux_arm_count` never returning 1 (10K draws), and `make_width_adapter` edge cases: identity (src == target), shrink (Slice), expand exact-multiple (single Concat), expand non-multiple (Concat + Slice).
- `src/emit/sv.rs` — 6 inline unit tests pinning emitter output on hand-built IRs: module header + endmodule + port declarations + passthrough assign, conditional omission of clk/rst_n when zero flops, canonical `always_ff @(posedge clk or negedge rst_n)` header with active-low reset branch, operator and constant rendering, Slice `[hi:lo]` and Concat `{a, b}` forms, Mux ternary form.
- Total: 20 unit tests + 2 integration = **22 tests, all passing**.
- No external smoke tests wired up yet. Phase 1 exit gate requires Verilator-lint pass on a representative seed range.

## Known weaknesses (visible in code today)

- `gen::cone::input_widths_for` for `Slice` and `Concat` returns placeholder widths. `Slice` and `Concat` are not currently selectable in `pick_gate` (only used by `make_width_adapter` and the flop-mux assembly, which construct them directly with correct widths). Properly wire `input_widths_for` when Phase 3 makes them pickable.
- `emit::sv::render_gate` for `Concat` joins operand names with commas (correct SV); the IR does not currently distinguish per-operand widths in storage because every current producer of `Concat` either replicates a single source or concatenates uniform-width bits. When variadic `Concat` with mixed widths becomes a real motif, the IR shape is still adequate (widths are a property of each operand node, not of the `Concat` itself), but a generator-side helper will need to compose such shapes carefully.

## Build hygiene
- `cargo fmt --all --check` — clean.
- `cargo clippy --all-targets -- -D warnings` — clean.
