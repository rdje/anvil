# IR-TYPES-DECOMPOSITION: split `src/ir/types.rs` by ownership, not by size

## Metadata

- Tree ID: `IR-TYPES-DECOMPOSITION`
- Status: `done`
- Roadmap lane: Codebase hygiene / module ownership — owner-directed
- Created: `2026-07-31`
- Last updated: `2026-08-01` (**CLOSED at `.4`** — re-measured and decided: `types.rs` **4,069 → 1,473**, and what remains is one coherent data model. The candidate third split did not survive measurement)
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
  Status: `done`
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
  Status: `done`
  Goal: `Extract the canonicalization engine into src/ir/intern.rs: flatten_associative (539 lines) + intern_gate (141) + intern_constant, as an impl Module block in the new module (Rust permits inherent impls in any module of the defining crate, so no field visibility changes), together with the ~38 fold_*/peephole_*/flatten_associative_*/intern_gate_* tests that exercise them. This is the single largest tenant: ~1830 lines with its tests, 45% of the file.`
  Acceptance: `Same as .2. Additionally: the moved tests keep their fixtures (fold_fixture, port, comb_child, seq_child) — duplicate a fixture only if BOTH files still need it, otherwise move it, and never leave a copy behind (feedback_full_factorization).`
  Verification: `done — VERBATIM MOVE, PROVEN RATHER THAN ASSERTED. src/ir/types.rs 3,607 -> 1,485 lines; new src/ir/intern.rs 2,173. FIVE functions moved, not the three the goal named: measuring the block found it CONTIGUOUS at lines 742-1904 and containing fold_constants (326 lines, pub(crate), ZERO callers outside types.rs) and apply_peephole (422 lines, pub(crate), ONE caller — intern_gate itself) between the named three. Moving intern_gate/intern_constant/flatten_associative while leaving those two behind would have split one engine across two files and left types.rs holding code only the other file calls, so the goal's list was widened on measurement and the reason is recorded here rather than silently. SEVERABILITY PROVEN IN BOTH DIRECTIONS BEFORE CUTTING, as .2 did: (a) the two fields the engine touches, gate_instances and const_instances, are already pub(crate), so a SIBLING module of crate::ir reaches them and NO visibility widened — Rust permits an inherent impl in any module of the defining crate, which is what makes this a move rather than a redesign; (b) the engine's only outward calls are self.effective_factorization_level() (pub, stays in types.rs) and the four engine fns themselves. VERBATIM PROVEN BY WHOLE-FILE LINE CENSUS against a pre-image copy, not by reading the diff: every non-blank line of the original 3,608-line types.rs appears byte-identical in types.rs or intern.rs afterwards — RESIDUE 0 — and the only lines in intern.rs absent from the original are the 17-line module doc plus one use. The import list was derived from the COMPILER, not guessed: the first build named DepSet as missing and HashMap as unused, both fixed, giving use super::types::{DepSet, GateOp, Module, Node, NodeId}. TEST COUNT EXACTLY CONSERVED: 40 -> 3 + 37. FIXTURE SPLIT MEASURED PER SIDE rather than assumed, which is what the acceptance demanded: fold_fixture has 7 uses, ALL from engine tests, 0 from stayers => MOVED; port (12 uses), comb_child (1) and seq_child (1) are used ONLY by the three staying tests => STAYED. No fixture is needed by both, so NOTHING was duplicated and no copy was left behind. BLAST RADIUS exactly src/ir/ — 3 files (types.rs, mod.rs, new intern.rs); ZERO call sites changed anywhere, because intern_gate (91 refs) and intern_constant (15 refs) are inherent methods on Module and resolve identically. Checks: cargo test 1,058 passed / 0 failed across 17 suites (lib 749, downstream 133, pipeline 113) with a REAL captured exit status of 0; tests/snapshots.rs 6/6 and NO .snap rewritten => DUT BYTE-IDENTICAL; cargo clippy --all-targets -D warnings clean; cargo fmt --all --check clean; cargo check --all-targets clean. METHOD CORRECTION RECORDED: the first test run was invoked as `cargo test | tail -40`, whose exit status is TAIL'S, not cargo's — the repo's own gated-workflow-shell-gotchas card warns of exactly this, and it still happened; the run was redone unpiped with $? captured to a file before any green claim was made.`
  Commit: `1b1793e`

- ID: `IR-TYPES-DECOMPOSITION.4`
  Status: `done`
  Goal: `Re-measure src/ir/types.rs and decide, on evidence, whether a third split is warranted (candidate: the ~649 lines of port/clock-domain/emitted-port predicates) or whether what remains is one coherent data model and the tree should close. Record the decision either way — "it is one thing now" is a legitimate and preferred outcome.`
  Acceptance: `A measured per-region table like the one above, and an explicit close-or-continue decision recorded in this tree.`
  Verification: `done — RE-MEASURED AT 1b1793e; DECISION: CLOSE. src/ir/types.rs is 1,473 lines (from 4,069), and the per-region table is in the Decisions section below. THE CANDIDATE THIRD SPLIT DID NOT SURVIVE RE-MEASUREMENT, which is the whole reason this leaf exists. .1 estimated "~649 lines of port/clock-domain/emitted-port predicates"; measured today that region is 137 declaration lines + a 165-line impl Module block = 302 lines, 20.5 % of the file. The two numbers I can reproduce exactly are these: impl Module was 1,327 lines / 21 methods at .1 and is 165 lines / 16 methods now, the difference being precisely the five engine functions .3 moved (21 - 16 = 5). I could NOT reproduce .1's exact ~649 basis and am recording that rather than reverse-engineering a justification for it: the estimate was taken when the impl block still had the interning engine mixed into it, and it did not survive contact with a measurement. WHY CLOSE RATHER THAN SPLIT AGAIN: every remaining region answers the ONE question a types file exists to answer — "what is a <thing>?" (Module 29.7 %, nodes 24.6 %, tests 12.1 %, impl Module accessors 11.2 %, blocks 10.3 %, ports/params/aggregates/domains 9.3 %, Design 0.3 %). The 16 surviving impl Module methods are small accessors and predicates OVER THE STRUCT DECLARED 170 LINES ABOVE THEM (has_local_flops, flop_domain, effective_clock_domains, is_emitted_input_port, emitted_data_input_ports, input_port/output_port, ...); separating a struct from its own accessors is not an ownership boundary, it is a line-count boundary — the exact thing this tree's Goal rejects ("the bar is OWNERSHIP, not line count"). The foreign-tenant probe agrees: types.rs's only non-std imports are CaseQualifier (a field type) and a re-export of KnobRollCounters (a field type), i.e. nothing that answers a different question. "It is one thing now" is the outcome the leaf named as legitimate and preferred, and it is the measured one.`
  Commit: `IR-TYPES-DECOMPOSITION.4`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `IR-TYPES-DECOMPOSITION.1` | `done` | Audit + register. The measurement is the work product: two tenants foreign to a types file are **55 %** of it, the blast radius of a split is **zero call sites** (`pub use types::*`, only 2 explicit `types::` sites repo-wide), and one doctrine check hard-codes the path. Ordering fixed against `CSG.6`. |
| 2 | `IR-TYPES-DECOMPOSITION.2` | `done` | `types.rs` **4069 → 3607**; `src/ir/knob_id.rs` is 483 lines; tests conserved exactly (42 → 40 + 2). Severability proven in **both** directions before cutting. Blast radius exactly as `.1` predicted: **one** import line, inside `src/ir/`. Surfaced two pre-existing defects, both now tree-owned. |
| — | **`COVERAGE-STEERED-GENERATION.6`** | `done` | Landed `f335926` in its own tree, as this ordering required. `KnobId` had already moved out at `.2`, so the rung-R1 macro table landed in a file that owns one thing — `knob_id.rs` 483 → 344 lines. |
| 3 | `IR-TYPES-DECOMPOSITION.3` | `done` | `1b1793e`. `types.rs` **3,607 → 1,473**; new `intern.rs` 2,193. **Five** functions moved, not the three the goal named — measuring the block found it contiguous and found `fold_constants` / `apply_peephole` sitting between the named three, so leaving them would have split one engine across two files. Verbatim proven by whole-file line census (residue 0), tests conserved 40 → 3 + 37, zero call sites changed. |
| 4 | `IR-TYPES-DECOMPOSITION.4` | `done` | **Re-measured, and the answer was CLOSE.** The candidate third split evaporated under measurement: `.1`'s *"~649 lines of port/clock-domain predicates"* is **302** today (137 declarations + a 165-line `impl Module`), and those 165 lines are 16 small accessors over a struct declared 170 lines above them. Separating a struct from its own accessors is a line-count boundary, not an ownership one — the exact thing this tree's Goal rejects. **The tree is closed.** |

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

- `2026-08-01` (`.4`): **CLOSE — what remains is one coherent data model.** Measured
  at `1b1793e`, `src/ir/types.rs` is **1,473** lines (from **4,069**), and every
  region answers the one question a types file exists to answer:

  | region | lines | share | answers the question |
  | --- | ---: | ---: | --- |
  | `struct Module` declaration | 438 | 29.7 % | *"what is a module?"* |
  | `Node` / `ForFoldKind` / `GateOp` / `ResetKind` / `FlopKind` / `MuxArm` / `FlopMux` / `Flop` / `DepSet` | 363 | 24.6 % | *"what is a node?"* |
  | `#[cfg(test)] mod tests` | 178 | 12.1 % | — (the engine's ~37 tests left with it at `.3`) |
  | `impl Module` — accessors + port/domain predicates | 165 | 11.2 % | *"what does this module contain or expose?"* |
  | `Instance` / `Memory` / `Fsm` + impls | 152 | 10.3 % | *"what is a block?"* |
  | `Port` / `Direction` / `WidthExpr` / `ParamEnv` / `Aggregate*` / `ModuleInterfaceProfile` / `ClockDomain` | 137 | 9.3 % | *"what is a port / a parameter / a domain?"* |
  | `struct Design` | 5 | 0.3 % | *"what is a design?"* |

  **The candidate third split evaporated under measurement**, which is precisely why
  this leaf was written as *re-measure and decide* rather than *split again*. `.1`
  estimated *"~649 lines of port / clock-domain / emitted-port predicates"*; the
  region is **302** lines today. Two numbers reproduce exactly and are worth keeping:
  `impl Module` was **1,327 lines / 21 methods** at `.1` and is **165 lines / 16
  methods** now — a difference of exactly the five engine functions `.3` moved. The
  `~649` basis could **not** be reproduced and is recorded as unreproducible rather
  than reverse-engineered into a justification: it was taken while the interning
  engine was still mixed into that block.

  Those 165 lines are 16 small accessors and predicates over the struct declared
  **170 lines above them** (`has_local_flops`, `flop_domain`,
  `effective_clock_domains`, `is_emitted_input_port`, `emitted_data_input_ports`,
  `input_port` / `output_port`, …). **Separating a struct from its own accessors is a
  line-count boundary, not an ownership one** — the exact split this tree's Goal
  refuses. The foreign-tenant probe agrees: the file's only non-`std` imports are
  `CaseQualifier` and a re-export of `KnobRollCounters`, both *field types*, so
  nothing in the file answers a different question.

  Recorded as the preferred outcome the leaf itself named: **"it is one thing now."**

- `2026-08-01` (`.4`, observed not acted on): `src/ir/compact.rs` is now **5,453**
  lines — the largest file under `src/ir/` and the third-largest in the crate. Under
  this tree's own bar that is **not by itself a defect**, and no measurement of its
  tenancy exists, so the honest status is *unmeasured, therefore unknown*. Deliberately
  **not** folded into this tree: doing so would be splitting by line count, which is
  the one thing the Goal forbids. If it is ever picked up it needs its own tree and
  its own `.1` audit — the same shape that made this one work.

## Open Questions

- Whether `.2`'s new module should be `src/ir/intern.rs` or `src/ir/canonical.rs`.
  Leaning `intern.rs`: the entry points are `intern_gate` / `intern_constant`, and
  `src/ir/dedup.rs` already owns the *design*-level dedup, so `canonical` would be
  ambiguous between the two.
- ~~Whether the `~649` lines of port / clock-domain / emitted-port predicates are a
  third tenant or part of the data model.~~ **ANSWERED at `.4`: part of the data
  model, and the `~649` did not survive measurement** — the region is **302** lines
  today (137 declarations + a 165-line `impl Module` of 16 accessors). Holding this
  question open until the remainder could be *measured rather than guessed* is
  exactly what stopped a third split from happening on a stale estimate.

## Blockers

- None.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-08-01` | `IR-TYPES-DECOMPOSITION.4` | `re-measured src/ir/types.rs at 1b1793e = 1,473 lines with a per-region table (Module 438 / nodes 363 / tests 178 / impl Module 165 / blocks 152 / ports+params+aggregates+domains 137 / Design 5); impl Module measured 1,327 lines + 21 methods at .1 vs 165 lines + 16 methods now, the delta being exactly the five engine fns .3 moved; the .1 ~649 estimate NOT reproducible and recorded as such; foreign-tenant probe = only CaseQualifier + KnobRollCounters, both field types; no code touched` | `CLOSE — one coherent data model; docs-only, DUT byte-identical` |
| `2026-07-31` | `IR-TYPES-DECOMPOSITION.2` | `severability proven both ways (no KnobId reference in types.rs outside the block; no data-model reference inside it); types.rs 4069 -> 3607, knob_id.rs 483, #[test] 42 -> 40+2 conserved; exactly ONE call site changed (an import in src/ir/knob_roll.rs), zero outside src/ir/; extractor repointed + negative-controlled (wrong path -> 0 categories -> floor trip, exit 1); cargo fmt/check/clippy/test all PASS with tests/snapshots.rs untouched; check_doctrines.sh 8/8` | `pure move; DUT byte-identical` |
| `2026-07-31` | `IR-TYPES-DECOMPOSITION.1` | `tree registered (docs-only); measured src/ir/types.rs = 4069 lines at bd7dba2 with the per-region table above; confirmed src/ir/mod.rs already does pub use types::*, and that only 2 sites repo-wide write types:: explicitly; confirmed scripts/check_enumeration_parity.sh hard-codes src/ir/types.rs in extract_steering_categories; no code touched` | `registered` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `IR-TYPES-DECOMPOSITION.1` | `30e731b` — `IR-TYPES-DECOMPOSITION.1 — audit + register the ownership-split tree` | Docs-only; no code moved. |
| `IR-TYPES-DECOMPOSITION.2` | `218277d` — `IR-TYPES-DECOMPOSITION.2 — extract KnobId into src/ir/knob_id.rs` | Pure move ⇒ DUT byte-identical. |
| `IR-TYPES-DECOMPOSITION.3` | `1b1793e` — `IR-TYPES-DECOMPOSITION.3 — extract the canonicalization engine` | Pure move ⇒ DUT byte-identical. `types.rs` 3,607 → 1,473; new `intern.rs` 2,193. **Five** functions moved, not the three the goal named — the block was contiguous and `fold_constants` / `apply_peephole` sat between them. Verbatim proven by whole-file line census (residue 0); tests conserved 40 → 3 + 37; zero call sites changed. |
| `IR-TYPES-DECOMPOSITION.4` | `IR-TYPES-DECOMPOSITION.4 — re-measure and close: it is one thing now` | Docs-only; no code touched. Re-measured `types.rs` at **1,473** lines with a per-region table and decided **CLOSE**: every region answers *"what is a ⟨thing⟩?"*. The candidate third split evaporated — `.1`'s `~649` is **302** today, and its 165 `impl Module` lines are 16 accessors over a struct declared 170 lines above them. **Closes the tree** (`4,069 → 1,473`, two new single-purpose modules, zero call sites changed across all three moves). |

## Changelog

- `2026-07-31`: Created on owner directive. The framing *"by ownership, I mean by
  who does what"* is load-bearing and is written into the Non-Goals: this tree
  will not split by line count, and `.4` may legitimately conclude that what
  remains is one coherent thing and close without splitting further.
- `2026-08-01`: **`.4` done — and it concluded exactly that. The tree is CLOSED.**
  `src/ir/types.rs` went **4,069 → 1,473** across three moves that changed **zero
  call sites**, producing two single-purpose modules (`knob_id.rs`, `intern.rs`).
  `.4` re-measured rather than split: the candidate third tenant — `.1`'s *"~649
  lines of port / clock-domain predicates"* — is **302** lines today, and its
  `impl Module` half is 16 small accessors over a struct declared 170 lines above
  them, which is a line-count boundary rather than an ownership one. The `~649`
  basis could not be reproduced and is recorded as unreproducible rather than
  rationalised. **The escape hatch the tree wrote for itself on day one is the one
  it took**, which is the point worth keeping: a decomposition tree needs a leaf
  whose licensed answer is *stop*, or it will keep splitting until the criterion
  is size. Also observed and deliberately not acted on: `src/ir/compact.rs` is now
  **5,453** lines, unmeasured — it would need its own tree and its own `.1` audit,
  never a silent widening of this one.
