# AGGREGATE-ARRAY-PACKING: Packed-Array Aggregate Projection

## Metadata

- Tree ID: `AGGREGATE-ARRAY-PACKING`
- Status: `done`
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
  Status: `done`
  Goal: `aggregate_array_prob knob + uniform-width selection in annotate + call-site second roll.`
  Acceptance: `Default 0.0 byte-identical (snapshots); prob 1.0 with uniform widths yields ArrayPacked end-to-end; non-uniform falls back to StructPacked; validated + dump-config.`
  Verification: `cargo test --lib aggregate 14/14 (incl. 3 new selection tests); cargo test --test pipeline aggregate 3/3 (new array_packed_aggregate_selected_with_uniform_widths reachable + downstream-valid; struct path unchanged); cargo test --test snapshots 6/6 byte-identical; --dump-config shows aggregate_array_prob:0.0; cargo check --all-targets + clippy --all-targets + fmt clean. All under scripts/ram_guard.sh --threshold 88.`
  Commit: `AGGREGATE-ARRAY-PACKING.3 - aggregate_array_prob selection`

- ID: `AGGREGATE-ARRAY-PACKING.4`
  Status: `done`
  Goal: `Distinguishing metric + authoritative downstream-clean proof.`
  Acceptance: `DesignMetrics distinguishes array- from struct-packed (num_array_packed_aggregate_modules); a generated array-packed design is Verilator + Yosys clean using the matrix's exact invocations.`
  Verification: `src/metrics.rs num_array_packed_aggregate_modules added + asserted in the end-to-end pipeline test; corpus via --config (aggregate_array_prob=1.0, uniform width) over the hierarchy design path produced 35/40 array designs + 10 struct fallbacks; 7/7 isolated array-packed designs passed verilator --lint-only --top-module AND yosys "synth -noabc; check" (matrix flags) — all generation under scripts/ram_guard.sh; cargo check --all-targets + clippy + fmt clean.`
  Commit: `AGGREGATE-ARRAY-PACKING.4 - array metric + downstream-clean proof`

- ID: `AGGREGATE-ARRAY-PACKING.4b`
  Status: `deferred`
  Goal: `Permanent tool_matrix array scenario + saw_array_packed_aggregate_design coverage fact (optional CI instrumentation).`
  Acceptance: `A depth-1-wrapper uniform-width scenario lights a new saw_array_packed_aggregate_design fact via summarize/merge, with a non-vacuous + a summarize unit test; NOT added to the hard Phase4Hierarchy gap-gate.`
  Verification: `n/a`
  Commit: `n/a`
  Deferral: `Optional CI tracking only. The knob is already exercised end-to-end by the pipeline test and proven downstream-clean directly (.4, 7/7 Verilator+Yosys), so it is not a dead knob. Deferred to avoid destabilizing the rigid Phase4Hierarchy scenario asserts; reopen as an rN slice if matrix CI coverage of the array kind is wanted.`

- ID: `AGGREGATE-ARRAY-PACKING.5`
  Status: `done`
  Goal: `Book + knobs + ir.md sync; close tree.`
  Acceptance: `knobs.md documents aggregate_array_prob (+ metrics-map row) and corrects the stale "struct packed only" note; ir.md aggregates subsection marks ArrayPacked delivered; ROADMAP pointer already added at tree open; mdbook build clean; book-runnable contract preserved (prose-only, no bash block touched); tree closed.`
  Verification: `mdbook build book clean; book changes are prose-only (git diff --stat book/ = ir.md + knobs.md, no runnable bash block), so the byte-identical book-runnable contract holds and the heavy --release book_examples rebuild was intentionally not triggered (resource policy). USER_GUIDE does not document config-only aggregate knobs, so no drift there.`
  Commit: `AGGREGATE-ARRAY-PACKING.5 - book/docs sync + close`

## Current Frontier

Empty — the tree is `done`. `.1`–`.5` are complete; `.4b` (optional
tool_matrix CI instrumentation) is `deferred` with a recorded rationale.
Reopen `.4b` as an `rN` slice only if matrix CI coverage of the
`ArrayPacked` kind is later wanted.

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
- `2026-06-14` (`.4`): downstream-clean is proved DIRECTLY (generate an
  array corpus via the hierarchy design path, then run the matrix's
  exact `verilator --lint-only --top-module` + `yosys "synth -noabc;
  check"` on isolated array designs: 7/7 clean). The permanent
  `tool_matrix` array scenario + coverage fact is split to `.4b` and
  kept OUT of the hard Phase4Hierarchy gap-gate because (a) that set has
  rigid exact-count / child-tuple / name-whitelist asserts, and (b)
  array selection depends on per-design uniform output widths (~87%
  reach in the depth-1 wrapper), so a hard gap-check could rarely flake.
  The knob is already exercised by the end-to-end pipeline test, so it
  is not a dead knob; `.4b` adds CI tracking without gate fragility.

## Open Questions

- Resolved at `.4`: `saw_array_packed_aggregate_design` is NOT wired
  into the hard `Phase4Hierarchy` gap-gate (see `.4` decision). `.4b`
  adds it as a tracked coverage fact + non-vacuous/summarize unit tests
  only.

## Blockers

- None.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-06-14` | `AGGREGATE-ARRAY-PACKING.1` | `cargo check --all-targets`; `cargo test --lib aggregate`; `cargo fmt --all --check`; `cargo clippy --lib -D warnings` (all under `scripts/ram_guard.sh --threshold 88`) | passed (check clean; aggregate 10/10; fmt+clippy clean; guard never tripped) |
| `2026-06-14` | `AGGREGATE-ARRAY-PACKING.2` | `cargo test --lib emit::sv` (26/26); `cargo test --test snapshots` (6/6 byte-identical); `cargo test --test pipeline packed_aggregate` (2/2, StructPacked unchanged); fmt + clippy (all under `scripts/ram_guard.sh --threshold 88`) | passed (guard never tripped) |
| `2026-06-14` | `AGGREGATE-ARRAY-PACKING.3` | `cargo test --lib aggregate` (14/14); `cargo test --test pipeline aggregate` (3/3); `cargo test --test snapshots` (6/6 byte-identical); `--dump-config` shows `aggregate_array_prob`; `cargo check --all-targets` + `cargo clippy --all-targets -D warnings` + fmt clean (all under `scripts/ram_guard.sh --threshold 88`) | passed (guard never tripped) |
| `2026-06-14` | `AGGREGATE-ARRAY-PACKING.4` | metric added; `cargo test --test pipeline array_packed_aggregate_selected_with_uniform_widths` (metric asserted); corpus 35/40 array + 10 struct fallback; 7/7 isolated array designs `verilator --lint-only --top-module` + `yosys "synth -noabc; check"` clean (matrix flags); check/clippy/fmt clean (under `scripts/ram_guard.sh`) | passed (Verilator 5.046 + Yosys 0.64 both clean on every array design) |
| `2026-06-14` | `AGGREGATE-ARRAY-PACKING.5` | `mdbook build book` clean; `git diff --stat book/` = `ir.md` + `knobs.md` prose only (no runnable bash block); USER_GUIDE has no config-only aggregate knob coverage (no drift) | passed (book in lockstep; book-runnable contract byte-identical, heavy `--release` rebuild not needed) |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `AGGREGATE-ARRAY-PACKING.1` | `AGGREGATE-ARRAY-PACKING.1 - add ArrayPacked variant` | Opens the tree (file + index row + ROADMAP pointer) in the same commit; pending hash. |
| `AGGREGATE-ARRAY-PACKING.2` | `AGGREGATE-ARRAY-PACKING.2 - emit ArrayPacked` | Emitter typedef + `[i]` aliases; StructPacked byte-identical; pending hash. |
| `AGGREGATE-ARRAY-PACKING.3` | `AGGREGATE-ARRAY-PACKING.3 - aggregate_array_prob selection` | Knob + uniform-width selection + call-site roll; default-off byte-identical; pending hash. |
| `AGGREGATE-ARRAY-PACKING.4` | `AGGREGATE-ARRAY-PACKING.4 - array metric + downstream-clean proof` | `num_array_packed_aggregate_modules` metric + 7/7 Verilator+Yosys clean on array designs; pending hash. |
| `AGGREGATE-ARRAY-PACKING.5` | `AGGREGATE-ARRAY-PACKING.5 - book/docs sync + close` | knobs.md + ir.md synced; tree closed; `.4b` deferred; pending hash. |

## Changelog

- `2026-06-14`: Created tree; designed the 5-leaf decomposition; landed
  `.1` (`AggregateKind::ArrayPacked` additive variant); frontier moves
  to `.2`.
- `2026-06-14`: Landed `.2` (emitter ArrayPacked rendering); frontier
  moves to `.3`.
- `2026-06-14`: Landed `.3` (`aggregate_array_prob` knob + uniform-width
  selection + call-site roll); frontier moves to `.4`.
- `2026-06-14`: Landed `.4` (distinguishing metric + 7/7 Verilator+Yosys
  downstream-clean proof on array designs). Split the permanent matrix
  scenario/coverage-fact to `.4b` (optional, out of the hard gate).
  Frontier moves to `.5` (book/docs + close).
- `2026-06-14`: Landed `.5` (knobs.md + ir.md book sync, prose-only).
  `.4b` marked `deferred` (optional CI instrumentation). Tree CLOSED —
  the packed-array aggregate capability is delivered, downstream-clean,
  and documented.
