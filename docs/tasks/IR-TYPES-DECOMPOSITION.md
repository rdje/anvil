# IR-TYPES-DECOMPOSITION: split `src/ir/types.rs` by ownership, not by size

## Metadata

- Tree ID: `IR-TYPES-DECOMPOSITION`
- Status: `active`
- Roadmap lane: Codebase hygiene / module ownership — owner-directed
- Created: `2026-07-31`
- Last updated: `2026-07-31` (`.2` landed — `KnobId` extracted, `types.rs` 4069 → 3607; frontier `COVERAGE-STEERED-GENERATION.6`, then `.3`)
- Owner: repo-local workflow

## Goal

`src/ir/types.rs` is **4069 lines** — the second-largest file in the crate. Split
it so each file owns **one job**, on the owner's directive (`2026-07-31`: *"why
not break it down into smaller .rs files by ownership, I mean by who does
what?"*).

The bar is **ownership, not line count**. A 4000-line file that declares one
coherent data model would be fine; this one is not that. It houses three tenants
that answer different questions, measured below.

## Non-Goals

- **No behaviour change.** Every leaf is a pure move: same items, same
  visibility, same order within each moved block. DUT byte-identical, and
  `tests/snapshots.rs` must stay untouched at every leaf.
- **No API churn.** `src/ir/mod.rs` already re-exports with `pub use types::*`,
  so callers name `crate::ir::X`. Every new module is re-exported the same way
  and **no call site changes**. Measured: only **2** sites in the whole repo
  write `types::` explicitly.
- **No merging of the emit-projection passes.** `src/ir/` is *already* decomposed
  by pass (`function_emit`, `task_emit`, `cone_function_emit`, `mux_if_emit`,
  `case_mux_if_emit`, `casez_mux_if_emit`, `generate_loop`, `soft_union`,
  `multi_output_task_emit`, `aggregate`, `dedup`, `validate`, `compact`,
  `knob_roll`). That decomposition is good and is not being revisited.
- **Not a rename sweep.** No type is renamed; a rename would make the move
  unreviewable by hiding it inside a diff of a thousand call sites.

## The measurement (at `bd7dba2`, `2026-07-31`)

| region | lines | share | answers the question |
| --- | ---: | ---: | --- |
| `#[cfg(test)] mod tests` | 1258 | 30.9 % | — (**~95 % of them test the interning engine**) |
| `flatten_associative` (539) + `intern_gate` (141) | 680 | 16.7 % | *"what is this expression's canonical form?"* |
| `struct Module` declaration | 417 | 10.2 % | *"what is a module?"* |
| `enum KnobId` + `impl KnobId` | 378 | 9.3 % | *"what can be steered, and in which family?"* |
| `Node` / `GateOp` / `Flop` / `MuxArm` / `DepSet` | 363 | 8.9 % | *"what is a node?"* |
| `Instance` / `Memory` / `Fsm` | 152 | 3.7 % | *"what is a block?"* |
| ports / domains / emitted-port predicates | ~649 | 16.0 % | *"what does this module expose?"* |

**Two tenants are foreign to a types file, and together they are 55 % of it:**

1. **The canonicalization engine.** `flatten_associative` + `intern_gate` are
   680 lines of the factorization ladder — *behaviour*, the entry point to
   CSE / commutative / associative / constant-fold / peephole / e-graph. Their
   tests are the bulk of the 1258-line test module (`fold_*`, `peephole_*`,
   `flatten_associative_*`, `intern_gate_*` — ~38 of the ~40 tests). Engine plus
   tests is **~1830 lines, 45 % of the file**, and it is self-contained.
2. **The steering taxonomy.** `KnobId` is not circuit IR. It enumerates the
   generator's probability-roll sites and their coverage categories. The
   giveaway is external: `scripts/check_enumeration_parity.sh` hard-codes
   `sed -n '/pub fn category(&self)/,/^    }$/p' src/ir/types.rs` — **a doctrine
   check reaches into a file called `types.rs` to find a steering taxonomy.**
   When a gate has to know that layout, the thing is in the wrong house.

`KnobId` also carries **five parallel tables of the same 38 variants** — the
`enum`, `all()`, `index()`, `name()`, `category()`. Collapsing those to one is
`COVERAGE-STEERED-GENERATION.6` (rung **R1**, decision `0033`), which is the
*next queued roadmap action*. Extracting `KnobId` first means that macro table
lands in a file that owns exactly one thing, and the ~340-line → ~40-line diff is
readable instead of buried in a 4069-line file.

**Projected result:** `types.rs` → **~1750 lines** of pure data model, with no
call-site change anywhere.

## Acceptance Criteria

- Each leaf is a **pure move**: `cargo test` green including `tests/snapshots.rs`
  untouched ⇒ DUT byte-identical; `cargo clippy --all-targets -- -D warnings`
  and `cargo fmt --all --check` clean.
- No call site outside `src/ir/` changes, because `src/ir/mod.rs` re-exports the
  new module the same way it re-exports `types`.
- Every doctrine check that names `src/ir/types.rs` is repointed in the **same**
  commit as the move (currently one: `ENUMERATION-PARITY`'s
  `extract_steering_categories`), and the driver stays 8/8.
- Each moved block keeps its doc comments verbatim — the rationale is the
  content, and a move that paraphrases is not a move.

## Task Tree

- ID: `IR-TYPES-DECOMPOSITION`
  Status: `active`
  Goal: `Split src/ir/types.rs so each file owns one job: the steering taxonomy, the canonicalization engine, and the circuit data model.`
  Children: `.1` (audit + register), `.2` (extract KnobId), `.3` (extract the interning engine + its tests), `.4` (re-measure and close)

- ID: `IR-TYPES-DECOMPOSITION.1`
  Status: `done`
  Goal: `Audit + register (docs-only). Measure src/ir/types.rs per region and classify each region by the QUESTION it answers, not by size; identify which regions are tenants foreign to a data-model file; measure the call-site blast radius of a split; find every doctrine check or script that hard-codes the file path; and fix the leaf ORDER against the queued roadmap work. No code moved — the owning leaf must exist before the edit.`
  Acceptance: `This tree registered with the per-region measurement table, the tenant classification, the measured blast radius, and an explicit ordering decision against COVERAGE-STEERED-GENERATION.6; docs-only.`
  Verification: `done. MEASURED at bd7dba2: src/ir/types.rs = 4069 lines, 2nd-largest in the crate after compact.rs (5453). Per-region table in "The measurement" above. TWO TENANTS FOREIGN TO A TYPES FILE, together 55%: (a) the canonicalization engine — flatten_associative (539 lines) + intern_gate (141), which is 51% of impl Module's 1329 lines while the other 17 methods average ~13 lines each — plus the ~38 fold_*/peephole_*/flatten_associative_*/intern_gate_* tests that are the bulk of the 1258-line test module, ~1830 lines together; (b) the steering taxonomy KnobId + impl (378 lines), which is not circuit IR at all. THE DECISIVE EVIDENCE for (b) is external, not aesthetic: scripts/check_enumeration_parity.sh::extract_steering_categories hard-codes `sed -n '/pub fn category(&self)/,/^    }$/p' src/ir/types.rs` — a doctrine check has to know that a file named types.rs contains a steering taxonomy. BLAST RADIUS MEASURED, not assumed: src/ir/mod.rs already does `pub use types::*`, and only 2 sites in the entire repo write `types::` explicitly, so a pure move changes ZERO call sites — which doubles as the review criterion (a changed call site proves the move was not pure). KnobId additionally carries FIVE PARALLEL TABLES of the same 38 variants (enum 38, all() 38, index() 38, name() 38, category() exhaustive) — exactly what COVERAGE-STEERED-GENERATION.6 (rung R1, decision 0033) exists to collapse. ORDERING DECIDED: .2 (extract KnobId) runs BEFORE CSG.6 so the macro table lands in a file owning one thing and the ~340 -> ~40-line diff is readable rather than buried in 4069 lines; .3 (the 1830-line interning move) runs AFTER, so a large mechanical move is never interleaved with a semantic change to a different type. Docs-only ⇒ DUT byte-identical.`
  Commit: `IR-TYPES-DECOMPOSITION.1 — audit + register the ownership-split tree`

- ID: `IR-TYPES-DECOMPOSITION.2`
  Status: `done`
  Goal: `Extract KnobId + impl KnobId + its two tests (all_is_complete_and_ordered, knob_names_and_categories_are_disjoint_and_total) into src/ir/knob_id.rs. Pure move, doc comments verbatim. Re-export from src/ir/mod.rs so no call site changes. Repoint scripts/check_enumeration_parity.sh's extract_steering_categories at the new path.`
  Acceptance: `cargo test green incl. tests/snapshots.rs untouched; clippy + fmt clean; scripts/check_doctrines.sh 8/8 with ENUMERATION-PARITY still extracting 8 steering categories (count-floored at 6, so a broken extractor fails loudly rather than passing vacuously); zero call-site changes outside src/ir/.`
  Verification: `done. SEVERABILITY PROVEN IN BOTH DIRECTIONS BEFORE CUTTING, which is what made this a pure move rather than a hopeful one: (a) no reference to KnobId anywhere in types.rs outside the block (lines 579-960) and its two tests (2815-2892); (b) no reference from that block to ANY data-model type (Module/Node/Port/Flop/GateOp/Design/DepSet/Instance/Memory/Fsm/WidthExpr/ParamEnv/ClockDomain/Direction). Zero coupling both ways. RESULT: src/ir/types.rs 4069 -> 3607 lines (-462); new src/ir/knob_id.rs 483 lines; #[test] count 42 -> 40 + 2, exactly conserved. src/ir/mod.rs gains `pub mod knob_id;` + `pub use knob_id::*;` so every crate::ir::KnobId path is unchanged. CALL-SITE BLAST RADIUS EXACTLY AS PREDICTED AT .1: the audit measured 2 sites writing `types::` explicitly and both are in src/ir/knob_roll.rs — one import (`use crate::ir::types::KnobId` -> `knob_id::KnobId`, the single line that had to change) and one doc-comment mention of ir::types::Module which remains correct. ZERO call sites outside src/ir/ changed, which was the acceptance criterion AND the review criterion: a changed call site would have proven the move impure. Doctrine extractor repointed at src/ir/knob_id.rs and negative-controlled — a deliberately wrong path yields 0 categories and trips floor_or_fail (exit 1), so a mis-repointed extractor fails loudly rather than passing vacuously. GATES: cargo fmt --all --check PASS (one rustfmt fixup for the blank line the cut left in the test module); cargo check --all-targets PASS; cargo clippy --all-targets -- -D warnings PASS; cargo test PASS incl. tests/snapshots.rs untouched => DUT byte-identical; scripts/check_doctrines.sh 8/8. FOUND WHILE DOING THIS, both pre-existing and both now task-tree-owned, neither caused by this move (each confirmed against `git show HEAD:`): (1) the ENUMERATION-PARITY steering extractor silently reads 7 of 8 categories -> PARITY-EXTRACTOR-ARM-SHAPE-GAP; (2) four of five per-file test counts in book/src/architecture.md are stale -> BOOK-TEST-COUNT-SHADOWS.`
  Commit: `IR-TYPES-DECOMPOSITION.2 — extract KnobId into src/ir/knob_id.rs`

- ID: `IR-TYPES-DECOMPOSITION.3`
  Status: `pending`
  Goal: `Extract the canonicalization engine into src/ir/intern.rs: flatten_associative (539 lines) + intern_gate (141) + intern_constant, as an impl Module block in the new module (Rust permits inherent impls in any module of the defining crate, so no field visibility changes), together with the ~38 fold_*/peephole_*/flatten_associative_*/intern_gate_* tests that exercise them. This is the single largest tenant: ~1830 lines with its tests, 45% of the file.`
  Acceptance: `Same as .2. Additionally: the moved tests keep their fixtures (fold_fixture, port, comb_child, seq_child) — duplicate a fixture only if BOTH files still need it, otherwise move it, and never leave a copy behind (feedback_full_factorization).`
  Verification: `pending`
  Commit: `pending`

- ID: `IR-TYPES-DECOMPOSITION.4`
  Status: `pending`
  Goal: `Re-measure src/ir/types.rs and decide, on evidence, whether a third split is warranted (candidate: the ~649 lines of port/clock-domain/emitted-port predicates) or whether what remains is one coherent data model and the tree should close. Record the decision either way — "it is one thing now" is a legitimate and preferred outcome.`
  Acceptance: `A measured per-region table like the one above, and an explicit close-or-continue decision recorded in this tree.`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `IR-TYPES-DECOMPOSITION.1` | `done` | Audit + register. The measurement is the work product: two tenants foreign to a types file are **55 %** of it, the blast radius of a split is **zero call sites** (`pub use types::*`, only 2 explicit `types::` sites repo-wide), and one doctrine check hard-codes the path. Ordering fixed against `CSG.6`. |
| 2 | `IR-TYPES-DECOMPOSITION.2` | `done` | `types.rs` **4069 → 3607**; `src/ir/knob_id.rs` is 483 lines; tests conserved exactly (42 → 40 + 2). Severability proven in **both** directions before cutting. Blast radius exactly as `.1` predicted: **one** import line, inside `src/ir/`. Surfaced two pre-existing defects, both now tree-owned. |
| — | **`COVERAGE-STEERED-GENERATION.6`** | `pending` | **Next, and it is in a different tree.** `KnobId` now lives alone, so the rung-R1 macro table lands in a file that owns one thing. |
| 3 | `IR-TYPES-DECOMPOSITION.3` | `pending` | The largest tenant (~1830 lines with tests, 45 %). Deliberately **after** `CSG.6`: it moves an `impl Module` block plus ~38 tests, and it should not be interleaved with a semantic change to a different type. |
| 4 | `IR-TYPES-DECOMPOSITION.4` | `pending` | Re-measure and close. Splitting further without re-measuring would be splitting by line count, which this tree explicitly rejects. |

## Decisions

- `2026-07-31`: Registered before any edit, per the task-tree ownership doctrine.
  Opened on the owner's `2026-07-31` directive to break the file down **by
  ownership** — the framing is the owner's and it is the right one, so this tree
  splits by *"who answers which question"* and refuses to split by line count.
- `2026-07-31`: **Ordered `.2` before `COVERAGE-STEERED-GENERATION.6`, and `.3`
  after it.** `.2` is a pure move that makes `.6`'s macro table land in its final
  home; doing `.6` first would rewrite ~340 lines in place and then move them,
  which is churn and an unreviewable diff. `.3` is held back because interleaving
  a 1830-line move with a semantic change to a different type would make both
  unreviewable.
- `2026-07-31`: **Registration is `.1`, not a bare-tree commit.** The `commit-msg`
  hook requires a task-tree **leaf** id (`^[A-Z][A-Z0-9-]+(\.[0-9A-Za-z]+)+`), so
  a tree cannot be registered under its own bare name — by design: every commit
  names a leaf. `.1` therefore carries the audit + registration, matching
  `README-POLICY-ADOPTION.1`'s established `.1 = audit + design` shape, and the
  execution leaves shift to `.2`/`.3`/`.4`. Caught by the hook on the first
  attempt, which is the layer working as intended.
- `2026-07-31`: **No `mod.rs` API change.** The split is invisible to callers by
  construction (`pub use types::*` is already the pattern). That is what makes a
  move of this size safe to review: if a call site changed, the move was not pure.

## Open Questions

- Whether `.2`'s new module should be `src/ir/intern.rs` or `src/ir/canonical.rs`.
  Leaning `intern.rs`: the entry points are `intern_gate` / `intern_constant`, and
  `src/ir/dedup.rs` already owns the *design*-level dedup, so `canonical` would be
  ambiguous between the two.
- Whether the `~649` lines of port / clock-domain / emitted-port predicates are a
  third tenant or part of the data model. **Deliberately unanswered until `.3`**,
  after the two clear tenants are gone and the remainder can be measured rather
  than guessed.

## Blockers

- None.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-07-31` | `IR-TYPES-DECOMPOSITION.2` | `severability proven both ways (no KnobId reference in types.rs outside the block; no data-model reference inside it); types.rs 4069 -> 3607, knob_id.rs 483, #[test] 42 -> 40+2 conserved; exactly ONE call site changed (an import in src/ir/knob_roll.rs), zero outside src/ir/; extractor repointed + negative-controlled (wrong path -> 0 categories -> floor trip, exit 1); cargo fmt/check/clippy/test all PASS with tests/snapshots.rs untouched; check_doctrines.sh 8/8` | `pure move; DUT byte-identical` |
| `2026-07-31` | `IR-TYPES-DECOMPOSITION.1` | `tree registered (docs-only); measured src/ir/types.rs = 4069 lines at bd7dba2 with the per-region table above; confirmed src/ir/mod.rs already does pub use types::*, and that only 2 sites repo-wide write types:: explicitly; confirmed scripts/check_enumeration_parity.sh hard-codes src/ir/types.rs in extract_steering_categories; no code touched` | `registered` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `IR-TYPES-DECOMPOSITION.1` | `30e731b` — `IR-TYPES-DECOMPOSITION.1 — audit + register the ownership-split tree` | Docs-only; no code moved. |
| `IR-TYPES-DECOMPOSITION.2` | `218277d` — `IR-TYPES-DECOMPOSITION.2 — extract KnobId into src/ir/knob_id.rs` | Pure move ⇒ DUT byte-identical. |

## Changelog

- `2026-07-31`: Created on owner directive. The framing *"by ownership, I mean by
  who does what"* is load-bearing and is written into the Non-Goals: this tree
  will not split by line count, and `.4` may legitimately conclude that what
  remains is one coherent thing and close without splitting further.
