# TASK-LEAF-COMMIT-SHADOW: every task-tree leaf records its commit hash twice, and 7 of the copies have already diverged

## Metadata

- Tree ID: `TASK-LEAF-COMMIT-SHADOW`
- Status: `active`
- Roadmap lane: Live-doc hygiene / task-tree record integrity
- Created: `2026-08-02`
- Last updated: `2026-08-02` (registered from a measurement; frontier `.1`)
- Owner: repo-local task-tree workflow

## Goal

Each leaf in `docs/tasks/*.md` records the commit that completed it **in two places**: the leaf
record's `Commit:` field, and a row in the same file's `## Commit Log` table. Both are maintained by
hand, and they have drifted.

**Measured across all 82 trees before registering:**

| | count |
| --- | --- |
| leaf records | **625** |
| leaf records carrying a `Commit:` field | **424** |
| trees carrying a `## Commit Log` table | **81 of 82** |
| leaves marked `done` | **411** |
| leaves marked `done` **whose `Commit:` is still `pending`** | **7** |
| leaf field says `pending` while the tree's own Commit Log row names a hash | **1** (live) |

The 7 stale leaves are `COVERAGE-STEERED-GENERATION.4`, `LOCAL-REFERENCE-CACHE.3`,
`OVERFLOW-DESTINATION-INSTRUMENTATION.6/.7/.8/.10`, and `UNGATED-PRACTICE-AUDIT.1` — the last of
which is the live divergence: its leaf reads `pending` while its Commit Log row reads `0e4654f`.

**None of the seven were created by the session that found them.** They are pre-existing, spread
across four unrelated trees and many weeks, which is what makes this a class rather than an incident.

## Why this is a shadow, not a hygiene lapse

Decision [`0033`](../decisions/0033-shadow-enumeration-classification.md) classifies a hand-kept copy
of a fact that is derivable elsewhere as a **shadow**, whose repair is **R1 — delete the copy**. Two
independent derivations exist for a leaf's commit:

1. **The Commit Log table**, in 81 of 82 trees, which is *strictly richer* — it carries the hash, the
   commit subject, and a notes column the leaf field has no room for.
2. **Git history itself.** `.githooks/commit-msg` **rejects a subject that names no leaf**, so
   `git log --grep=<leaf id>` is a total, deterministic derivation, not a best-effort search. This is
   `COMMIT.md` task-tree rule 1's own statement: *"hashes can be backfilled into the tree later, but
   the leaf ID cannot"* — the leaf ID is the durable join key; the hash is a convenience cache.

`feedback_full_factorization` (**one mechanism, never two**) is decided against the second copy, and
the 7 stale leaves are the predicted consequence arriving on schedule.

## How it was found — and why the finding is not "be more careful"

A session backfilling a hash into `BOOK-PARAGRAPH-BLOBS.4` wrote it into `.1`'s field instead: a
scripted replace matched the **first** `` Commit: `pending` `` in the file, and `.1`'s field was
`pending` **even though its Commit Log row already named `df7bc6e`**. The mis-target was possible
*because the shadow was already stale*. Fixing the typing does not remove the class; removing the
second copy does.

That episode also produced [[matched-mutation-is-not-the-intended-mutation]] — an asserted match
proves an edit *happened*, not that it happened *where you meant*. That card is the operating rule.
This tree is the structural repair underneath it.

## Non-Goals

- **Not a new doctrine check, and this is a decision rather than an omission.** Registering a gate
  that a `done` leaf must not carry `Commit: pending` would be **watching harder instead of removing
  the need** — the precedence [`0047`](../decisions/0047-negative-control-carrier-is-the-mutation.md)
  sets and `DOCTRINE_ENFORCEMENT.md` §9 counts over-gating as a defect in its own right. A gate here
  would also **cry wolf**: `done` + `pending` is the *legitimate* state of a leaf between its work
  commit and its backfill commit, which is the false-alarm failure decisions `0038` and `0045`
  measured and disqualified. If `.1` concludes the field must stay, a gate becomes arguable again —
  and only then.
- **Not a change to the Commit Log table.** It is the surviving mechanism, not a subject of repair.
- **Not a history rewrite.** `0031`: stale fields are corrected forward, never amended away.
- **No `src/` change.** Docs-only ⇒ DUT byte-identical.

## Acceptance Criteria

- `.1` **decides** between R1 (delete the per-leaf `Commit:` field; the Commit Log table is the single
  mechanism) and the alternative it must state before choosing (keep the field and justify the second
  copy against `feedback_full_factorization`). The decision lands as a **decision record**, because it
  changes a repo-wide convention that 82 files and `COMMIT.md` task-tree rule 2 depend on.
- The **derivation is proven, not assumed**, before any field is deleted: for every leaf id, show that
  `git log --grep` resolves the completing commit, and state the exact residue — including leaves
  whose commits predate the `commit-msg` hook, and the rule that separates a work commit from its
  `backfill the landed commit hash` follow-up.
- If R1 lands, `docs/TASK_TREE_README.md` and `docs/tasks/TEMPLATE.md` are updated in the same slice —
  a convention that survives only in 82 already-written files is not a convention.
- The 7 stale leaves are resolved either way: repaired if the field stays, removed if it goes.

## Task Tree

- ID: `TASK-LEAF-COMMIT-SHADOW`
  Status: `active`
  Goal: `A leaf's completing commit is recorded once, and that record cannot go stale by hand.`
  Children: `.0` (register, **done**), `.1` (decide R1 vs keep, with the derivation proven), `.2` (execute the decision across 82 trees and the schema docs)

- ID: `TASK-LEAF-COMMIT-SHADOW.0`
  Status: `done`
  Goal: `Register the finding with a denominator, before proposing any repair.`
  Acceptance: `Measured across every tree, not sampled: the denominator, the stale count, and the divergence count. States why a gate is NOT the first move, so a later session does not re-derive that.`
  Verification: `625 leaf records / 424 with a Commit: field / 411 done / 7 done-but-pending / 1 live leaf-vs-row divergence, across 82 trees. Ownership search run: MEMORY-ARCH checks decision indexing, TASK-TREE-OWNERSHIP checks co-staging, CHANGES-ENTRY-PLACEMENT checks entry position — none owns task-leaf field integrity, so this is a first mechanism, not a second.`
  Commit: `pending`

- ID: `TASK-LEAF-COMMIT-SHADOW.1`
  Status: `pending`
  Goal: `Decide R1 (delete the per-leaf Commit: field) against the stated alternative, and prove the derivation before anything is deleted.`
  Acceptance: `A decision record. The derivation from git log --grep must be MEASURED over all 424 fields with the residue stated — commits predating the commit-msg hook, and the work-commit vs backfill-commit disambiguation rule. "Keep the field" is a legitimate outcome, but it must answer feedback_full_factorization rather than skip it.`
  Verification: `pending`
  Commit: `pending`

- ID: `TASK-LEAF-COMMIT-SHADOW.2`
  Status: `pending`
  Goal: `Execute .1's decision across all 82 trees plus docs/TASK_TREE_README.md, docs/tasks/TEMPLATE.md and COMMIT.md task-tree rule 2.`
  Acceptance: `Mechanical and reversible. Edits anchored STRUCTURALLY on the leaf id, never on a text match (the failure that opened this tree), with a substitution count asserted per file. The 7 stale leaves are resolved. scripts/check_doctrines.sh stays 11/11.`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `TASK-LEAF-COMMIT-SHADOW.1` | `pending` | **Next.** Decide before editing 82 files. The derivation must be proven first: R1 is only safe if `git log --grep` really is total. |
| 2 | `TASK-LEAF-COMMIT-SHADOW.2` | `pending` | Execute. Deliberately after `.1`, and deliberately mechanical. |
| — | `TASK-LEAF-COMMIT-SHADOW.0` | `done` | Registered `2026-08-02` from a measurement over all 82 trees. |

## Decisions

- `2026-08-02` (registration): **Measured before registering, with a denominator.** *"I mistyped a
  backfill twice"* is an incident; **7 stale leaves of 411 done, across four unrelated trees and many
  weeks, none authored by the session that found them** is a class. Without the denominator this
  would have been filed as carelessness and closed with a resolution to be careful.
- `2026-08-02` (registration): **A gate was considered first and rejected first.** The obvious repair
  — fail the commit when a `done` leaf carries `Commit: pending` — is rejected on two independent
  grounds: it watches a hand-maintained duplicate instead of removing it (`0047`), and it cries wolf
  on the legitimate window between a work commit and its backfill (`0038` / `0045`). Recorded so the
  option is visibly *decided*, not overlooked.
- `2026-08-02` (registration): **Registered rather than repaired in passing.** Repairing the 7 stale
  fields would have been a two-minute edit that left the mechanism that produces them intact — and
  the repo would have looked clean while the class stayed live.

## Open Questions

- **Is `git log --grep=<leaf id>` genuinely total?** The `commit-msg` hook makes it so *going
  forward*; leaves completed before the hook existed may not be reachable. `.1` must measure the
  residue rather than assume it is empty.
- **How is a work commit told from its backfill?** Both subjects name the same leaf, so a naive
  `--grep` returns 2–3 commits per leaf. The rule (oldest match, or "subject does not end in
  *backfill the landed commit hash*") must be stated and measured, not inferred at read time.
- **Does the Commit Log table need the same scrutiny?** It is the surviving mechanism under R1, so a
  divergence *inside* it would be unrecoverable. `.1` should count rows against completed leaves.

## Blockers

- None.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-08-02` | `.0` | `Measured over all 82 docs/tasks/*.md (TEMPLATE.md excluded): 625 leaf records, 424 carrying a Commit: field, 411 marked done, 7 done-but-pending, 1 live leaf-vs-Commit-Log divergence (UNGATED-PRACTICE-AUDIT.1: leaf pending, row 0e4654f). 81 of 82 trees carry a Commit Log table. Ownership search run against MEMORY-ARCH, TASK-TREE-OWNERSHIP and CHANGES-ENTRY-PLACEMENT: none owns task-leaf field integrity.` | `registered` (docs-only; DUT byte-identical) |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `.0` (registration) | `TASK-LEAF-COMMIT-SHADOW.0 — register the twice-recorded leaf commit hash` | Docs-only. `.0` is this repo's registration-commit convention, required because `.githooks/commit-msg` rejects a subject naming no leaf. |

## Changelog

- `2026-08-02`: Created from a measurement taken while backfilling a hash into the wrong leaf. Each
  leaf records its completing commit **twice** — in the leaf's `Commit:` field and in the tree's
  `## Commit Log` row — and **7 of 411 done leaves** carry a stale `pending` field, one of them
  contradicting its own Commit Log row. The duplicate is a
  [`0033`](../decisions/0033-shadow-enumeration-classification.md) **shadow**: the Commit Log table
  is strictly richer, and `git log --grep` is a total derivation because `.githooks/commit-msg`
  rejects a subject that names no leaf. A gate was **considered and rejected first** — it would watch
  a hand-maintained copy instead of removing it (`0047`), and would cry wolf on the legitimate window
  between a work commit and its backfill (`0038`/`0045`). The repair is therefore **R1 — delete the
  copy** — but only after `.1` proves the derivation is total and states its residue.
