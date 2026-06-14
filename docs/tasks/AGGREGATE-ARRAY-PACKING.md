# AGGREGATE-ARRAY-PACKING: Packed-Array Aggregate Projection

## Metadata

- Tree ID: `AGGREGATE-ARRAY-PACKING`
- Status: `active`
- Roadmap lane: `Phase 5b follow-on — synthesizable aggregates (packed array)`
- Created: `2026-06-14`
- Last updated: `2026-06-14`
- Owner: repo-local workflow

## Goal

Add `AggregateKind::ArrayPacked` as a second, uniform-width
packed-aggregate emitter projection alongside the delivered
`StructPacked` (Phase 5b). A same-direction, **same-width** data-port
group is rendered as one packed-array port
(`typedef logic [N-1:0][W-1:0] t;` + `port[i]` boundary aliases),
bit-equivalent to the flat concatenation — a faithful, valid-by-
construction projection that stresses a different parser/elaboration
surface than `struct packed`. Opt-in behind a new default-`0.0`
`aggregate_array_prob` knob; off ⇒ byte-identical.

This is the recorded post-Phase-5b `ArrayPacked` sub-slice (deferred
boundary in `src/ir/types.rs` `AggregateKind` doc + `ROADMAP.md`
Phase 5b scope note), now task-tree-owned. It does **not** reopen
Phase 5b.

## Non-Goals

- No `UnionPacked` — a union overlays/aliases distinct ports, so it is
  NOT a faithful projection of distinct data ports; it stays deferred.
- No parent-side aggregate connections (instantiated children stay
  flat, exactly as the `StructPacked` scaffold).
- No param/aggregate cross-product (Phase 5 `param_env` modules stay
  skipped).
- No change to the flat IR body, validators, CSE keys, or
  `canonical_module_signature` (the projection is semantically empty).

## Acceptance Criteria

- `AggregateKind::ArrayPacked` exists; the IR shape is additive.
- A new `Config::aggregate_array_prob` (default `0.0`, validated,
  in `--dump-config`) gates a second per-module roll; when it fires and
  every projected group is internally uniform-width, the layout is
  `ArrayPacked`, else `StructPacked`.
- The emitter renders `ArrayPacked` as a packed-array typedef + one
  aggregate port/side + `port[i]` boundary aliases; `StructPacked`
  output is unchanged.
- Default-off byte-identical: `aggregate_array_prob == 0.0` ⇒
  snapshots + book_examples unchanged.
- A `tool_matrix` scenario produces a downstream-clean array-packed
  design and lights a `saw_array_packed_aggregate_design` coverage
  fact (Verilator + both Yosys), proven under `scripts/ram_guard.sh`.
- Book (progressive prose + runnable `--config` example), USER_GUIDE,
  knobs.md, ir.md aggregates subsection, and ROADMAP are synced in the
  same slices as the code; `DEVELOPMENT_NOTES.md` records the design.

## Task Tree

- ID: `AGGREGATE-ARRAY-PACKING`
  Status: `active`
  Goal: `Land packed-array aggregate projection, default-off, downstream-clean, fully documented.`
  Children: `.1`, `.2`, `.3`, `.4`, `.5`

- ID: `AGGREGATE-ARRAY-PACKING.1`
  Status: `done`
  Goal: `Add AggregateKind::ArrayPacked variant (additive IR shape).`
  Acceptance: `Variant compiles; deferral doc comment updated; any AggregateKind match stays exhaustive; existing aggregate tests green.`
  Verification: `cargo check --all-targets clean (variant additive — no non-exhaustive match broke); cargo test --lib aggregate 10/10; cargo fmt --all --check + cargo clippy --lib -D warnings clean (all under scripts/ram_guard.sh). Behavioral ArrayPacked tests land in .2/.3.`
  Commit: `AGGREGATE-ARRAY-PACKING.1 - add ArrayPacked variant`

- ID: `AGGREGATE-ARRAY-PACKING.2`
  Status: `done`
  Goal: `Emitter renders ArrayPacked (packed-array typedef + port[i] boundary aliases).`
  Acceptance: `A hand-built ArrayPacked layout emits logic [N-1:0][W-1:0] typedef, one aggregate port/side, and indexed alias wires/assigns; StructPacked output byte-identical; focused emitter test.`
  Verification: `cargo test --lib emit::sv 26/26 (incl. new emits_array_packed_aggregate_typedef_and_indexed_aliases + array_packed_single_bit_elements_emit_vector_typedef); cargo test --test snapshots 6/6 byte-identical; cargo test --test pipeline packed_aggregate 2/2 (StructPacked path unchanged); fmt + clippy clean. All under scripts/ram_guard.sh --threshold 88.`
  Commit: `AGGREGATE-ARRAY-PACKING.2 - emit ArrayPacked`

- ID: `AGGREGATE-ARRAY-PACKING.3`
  Status: `pending`
  Goal: `aggregate_array_prob knob + uniform-width selection in annotate + call-site second roll.`
  Acceptance: `Default 0.0 byte-identical (snapshots + book_examples); prob 1.0 with uniform widths yields ArrayPacked end-to-end; non-uniform falls back to StructPacked; validated + dump-config.`
  Verification: `pending`
  Commit: `pending`

- ID: `AGGREGATE-ARRAY-PACKING.4`
  Status: `pending`
  Goal: `Metrics coverage fact + tool_matrix scenario + downstream-clean proof.`
  Acceptance: `saw_array_packed_aggregate_design lit by a uniform-width scenario; focused matrix smoke clean in Verilator + both Yosys, run under scripts/ram_guard.sh.`
  Verification: `pending`
  Commit: `pending`

- ID: `AGGREGATE-ARRAY-PACKING.5`
  Status: `pending`
  Goal: `Book + USER_GUIDE + knobs + ir.md + ROADMAP sync; close tree.`
  Acceptance: `Progressive aggregate prose with a runnable --config example (struct→array); knobs.md aggregate_array_prob; ir.md marks ArrayPacked delivered; ROADMAP scope note updated; mdbook build + book_examples clean; tree closed with empty frontier.`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `AGGREGATE-ARRAY-PACKING.3` | `pending` | Wire selection once emission is correct. |
| 2 | `AGGREGATE-ARRAY-PACKING.4` | `pending` | Prove downstream-clean + coverage. |
| 3 | `AGGREGATE-ARRAY-PACKING.5` | `pending` | Sync docs/book + close. |

`.1` done — `AggregateKind::ArrayPacked` variant. `.2` done — emitter
renders the packed-array typedef + positional `[i]` boundary aliases;
`StructPacked` output byte-identical.

## Decisions

- `2026-06-14`: `ArrayPacked` chosen over `UnionPacked` as the first
  follow-on because a packed array is LRM-bit-equivalent to the field
  concatenation (faithful projection), whereas a union aliases distinct
  ports (not faithful). `UnionPacked` stays deferred.
- `2026-06-14`: `AggregateLayout.kind` stays per-layout (no per-group
  kind). `ArrayPacked` is chosen only when **every** present projected
  group is internally uniform-width; otherwise the layout falls back to
  `StructPacked`. Conservative, avoids an IR shape change.
- `2026-06-14`: Selection is a second independent seeded roll
  (`aggregate_array_prob`) at the existing `gen/mod.rs` call site;
  `annotate_aggregate` stays non-rolling (gains a `prefer_array` param,
  with the existing 1-arg form preserved for byte-identical callers).
- `2026-06-14`: Emitter/test-only validation per the resource policy;
  heavy matrix proof runs under `scripts/ram_guard.sh`.

## Open Questions

- Whether `.4` wires `saw_array_packed_aggregate_design` into the hard
  `Phase4Hierarchy` gate or keeps it a focused-smoke coverage fact
  (decide at `.4` based on scenario cost).

## Blockers

- None.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-06-14` | `AGGREGATE-ARRAY-PACKING.1` | `cargo check --all-targets`; `cargo test --lib aggregate`; `cargo fmt --all --check`; `cargo clippy --lib -D warnings` (all under `scripts/ram_guard.sh --threshold 88`) | passed (check clean; aggregate 10/10; fmt+clippy clean; guard never tripped) |
| `2026-06-14` | `AGGREGATE-ARRAY-PACKING.2` | `cargo test --lib emit::sv` (26/26); `cargo test --test snapshots` (6/6 byte-identical); `cargo test --test pipeline packed_aggregate` (2/2, StructPacked unchanged); fmt + clippy (all under `scripts/ram_guard.sh --threshold 88`) | passed (guard never tripped) |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `AGGREGATE-ARRAY-PACKING.1` | `AGGREGATE-ARRAY-PACKING.1 - add ArrayPacked variant` | Opens the tree (file + index row + ROADMAP pointer) in the same commit; pending hash. |
| `AGGREGATE-ARRAY-PACKING.2` | `AGGREGATE-ARRAY-PACKING.2 - emit ArrayPacked` | Emitter typedef + `[i]` aliases; StructPacked byte-identical; pending hash. |

## Changelog

- `2026-06-14`: Created tree; designed the 5-leaf decomposition; landed
  `.1` (`AggregateKind::ArrayPacked` additive variant); frontier moves
  to `.2`.
- `2026-06-14`: Landed `.2` (emitter ArrayPacked rendering); frontier
  moves to `.3`.
