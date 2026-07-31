# CHANGES-ENTRY-PLACEMENT: the two newest entries are at the bottom of a newest-first file

## Metadata

- Tree ID: `CHANGES-ENTRY-PLACEMENT`
- Status: `active`
- Roadmap lane: Live-doc hygiene / evidence findability
- Created: `2026-07-31`
- Last updated: `2026-07-31` (`.1` **done** — audited + registered; frontier `.2`)
- Owner: repo-local workflow

## Goal

`CHANGES.md` declares its own ordering on line 2 — *"Newest entries at the top."*
`COMMIT.md` §2 restates it as a mandatory pre-commit step — *"`CHANGES.md` gets a new
entry at the top."* The file is newest-first throughout: its **last** heading before the
defect is `2026-04-15-0001 — Initial scaffold + Phase 1 cone-adapter hardening`, the
oldest entry in project history.

**The two most recent entries are appended after it, at the absolute bottom of the
file**, measured at `087ca7b`:

| line | heading | commit |
| ---: | --- | --- |
| 43642 | `## 2026-04-15-0001 — Initial scaffold + Phase 1 cone-adapter hardening` | the oldest entry |
| 43706 | `## 2026-07-31-book-test-count-shadows-2 — BOOK-TEST-COUNT-SHADOWS.2 …` | `715019b` |
| 43780 | `## 2026-07-31-live-doc-registry-shadows-1 — LIVE-DOC-REGISTRY-SHADOWS.1 …` | `abf7090` |

Two defects, not one:

1. **Placement.** They sit where a reader looks for the *oldest* content, 43,700 lines
   below where the file's own rule puts them.
2. **Heading convention.** They use `## YYYY-MM-DD-slug — TITLE`, the format retired
   after `2026-06-14`; every entry written since uses `## YYYY-MM-DD — LEAF — title`.
   Measured: 249 headings in the current convention, 245 in the retired one, and the
   boundary between the two regions is otherwise clean.

The content of both entries is complete and correct (65 and 74 lines). This is purely a
placement and format defect — which is what makes it a *findability* problem rather than
a data-loss one, and why the repair is delicate rather than obvious.

## Why it matters

`CHANGES.md` is not decoration. It is the **evidence artifact** the doctrine layer names:
`COMMIT.md` makes it mandatory on every commit, `CODE-CHANGE-EVIDENCE` gates code changes
on it, and `DOCTRINE_ENFORCEMENT.md` §6 builds the reasoned-from-evidence pattern on top
of it. A session recovering cold reads it top-down and concludes the last change was
`BOOK-TEST-COUNT-SHADOWS.1` — **two leaves stale**, with no signal that anything is
missing.

## Why no mechanism caught it

`scripts/check_diagnosis_evidence.sh:43` is the whole of it:

```sh
printf '%s\n' "${staged}" | grep -qx 'CHANGES.md' || { … FAIL … }
```

The check asks whether `CHANGES.md` is **staged**. It cannot ask **where** the entry
landed, or whether one was added at all. And it is scope-aware — it governs only commits
touching `src/`/`tests/`/`examples/` — so a **docs-only** commit is exempt entirely, which
is exactly what both offending commits were.

**This is the same defect shape decision [`0037`](../decisions/0037-enumeration-parity-declared-sites-and-list-scoped-coverage.md)
just measured in `covers_set`, one layer up:** a check scoped to a **file** when the
property it means to hold lives in a **region** of that file. `covers_set` greps a whole
chapter for ids that belong in one list; `CODE-CHANGE-EVIDENCE` greps a whole staged-file
list for a name whose *position inside the file* is the thing that matters. Both pass
while the property they exist to hold is false. Registering this separately rather than
folding it into `LIVE-DOC-REGISTRY-SHADOWS` because the *subject* differs (evidence
findability vs enumeration parity) and the repair has a doctrine conflict that tree does
not — but the two share a root cause worth naming once, in one place.

## Non-Goals

- **Not "move the entries and move on."** `CHANGES.md` is **append-only and never
  retro-edited** (decision `0031`, standing owner directive: *"Keep it raw, keep honest,
  so that people can follow the whole history."*). Relocating two landed entries is an
  edit to already-published content. Whether that is a permitted correction or a
  prohibited sweep is a **decision**, and it must be recorded before anything moves —
  the recorded gotcha about mass-rewriting documents whose *subject* is the thing being
  rewritten applies directly.
- **Not "rewrite the two headings to the current format."** Same objection, and the
  retired format is itself a historical fact about when they were written.
- **No code change** beyond, possibly, `scripts/check_diagnosis_evidence.sh` if `.4`
  concludes a mechanism is warranted.
- **Not a general `CHANGES.md` reformat.** The 245 pre-`2026-06-14` slug-style headings
  are correct history in the convention of their time and stay exactly as they are.

## Acceptance Criteria

- The placement question is answered by a **recorded decision**, not by an edit: does
  `0031`'s append-only rule forbid relocating a misplaced entry, permit it as a
  correction of *position* (not content), or require a third option — leaving them in
  place with a forward pointer from the top of the file?
- Whatever is chosen, **the top of `CHANGES.md` must stop lying about what the most
  recent change was**, since that is the failure a cold session actually hits.
- The mechanism question is answered explicitly, applying decision `0033`'s three-part
  test before proposing any gate: is "an entry was added at the top" derivable,
  growth-coupled, and silent? If a check is warranted it obeys the
  `DOCTRINE_ENFORCEMENT.md` §4 contract and is negative-controlled both ways **and**
  vacuity-probed per decision `0037`.
- `scripts/check_doctrines.sh` 8/8; docs-only ⇒ DUT byte-identical.

## Task Tree

- ID: `CHANGES-ENTRY-PLACEMENT`
  Status: `active`
  Goal: `Restore CHANGES.md's stated newest-first ordering for its two most recent entries without breaching the append-only doctrine, and decide whether entry placement warrants a mechanism.`
  Children: `.1` (audit + register), `.2` (decide the repair against 0031), `.3` (apply it), `.4` (the mechanism question)

- ID: `CHANGES-ENTRY-PLACEMENT.1`
  Status: `done`
  Goal: `Audit the defect precisely and register the tree before anything is touched, per the standing directive that a defect is only handled if a task-tree owns it.`
  Acceptance: `Placement and heading-convention counts measured from the file itself, not inferred; both offending commits checked to confirm CHANGES.md WAS staged (so the defect is placement, not a skipped amendment); the governing check read directly to establish why it cannot see the defect; no repair attempted in this leaf.`
  Verification: `done — MEASURED at 087ca7b. CHANGES.md is 43843 lines and newest-first (line 4 = 2026-07-31, descending). Its last heading before the defect is line 43642, "2026-04-15-0001 — Initial scaffold + Phase 1 cone-adapter hardening" — the OLDEST entry in project history. The two most recent entries sit BELOW it: line 43706 (BOOK-TEST-COUNT-SHADOWS.2, commit 715019b) and line 43780 (LIVE-DOC-REGISTRY-SHADOWS.1, commit abf7090), both in the "## YYYY-MM-DD-slug" heading convention retired after 2026-06-14 (measured 249 headings in the current convention vs 245 in the retired one, with an otherwise clean boundary). CONFIRMED NOT A SKIPPED AMENDMENT: git show --stat shows both commits staged CHANGES.md, +74 and +65 lines respectively, and both entry bodies are complete — so the mandatory-amendment rule was followed and only the PLACEMENT is wrong, making this a findability defect rather than data loss. WHY NO MECHANISM SEES IT, read directly at scripts/check_diagnosis_evidence.sh:43: the check is `grep -qx CHANGES.md` over the STAGED FILE LIST — presence only, never position, never even "an entry was added" — and it is scope-aware, so a docs-only commit is exempt outright, which is what both offending commits were. That is decision 0037's finding one layer up: a check scoped to a FILE when the property it holds lives in a REGION of that file. NO REPAIR ATTEMPTED, deliberately: CHANGES.md is append-only by absolute owner directive (0031), so whether relocating a landed entry is a permitted correction of position or a prohibited sweep is .2's decision to record BEFORE anything moves. Checks: check_doctrines.sh 8/8 after git add. Docs-only => DUT byte-identical.`
  Commit: `e37cec3` — `CHANGES-ENTRY-PLACEMENT.1 — audit + register: the two newest entries are at the bottom`

- ID: `CHANGES-ENTRY-PLACEMENT.2`
  Status: `pending`
  Goal: `Decide, and record as a decision, whether relocating a misplaced-but-landed CHANGES.md entry is permitted under decision 0031's append-only rule — distinguishing a correction of POSITION from a rewrite of CONTENT — and choose between relocation, an in-place forward pointer, or a re-published entry at the top that cites the misplaced original.`
  Acceptance: `The three options are stated with their costs, the owner's "keep it raw, keep honest" rationale is applied to each (a reader following the whole history must not be misled by the repair either), and the chosen option is recorded as a decision record before any content moves. The decision must state what it does NOT license, so it cannot be cited later to justify a general CHANGES.md sweep.`
  Verification: `pending`
  Commit: `pending`

- ID: `CHANGES-ENTRY-PLACEMENT.3`
  Status: `pending`
  Goal: `Apply .2's decision so the top of CHANGES.md truthfully reflects the two most recent changes, and verify that no content was altered in the process.`
  Acceptance: `Byte-level proof that the two entries' BODIES are unchanged (hash them before and after); the top-of-file reading order matches the file's own stated rule; git diff reviewed line by line, since the recorded gotcha is that a sweep over a large doc silently damages unrelated content. check_doctrines.sh 8/8.`
  Verification: `pending`
  Commit: `pending`

- ID: `CHANGES-ENTRY-PLACEMENT.4`
  Status: `pending`
  Goal: `Decide whether entry PLACEMENT warrants a mechanism, applying decision 0033's three-part test first, and either register one or record precisely why diligence is the right answer here.`
  Acceptance: `The three-part test is applied explicitly (derivable? growth-coupled? silent?) before any check is proposed — the answer may well be that a placement check cries wolf on legitimate edits to old entries and so is worse than nothing, which is a valid and recordable outcome. If a check IS written, it must survive the decision 0037 vacuity probe: delete the top entry and the check must fail. Note the scope trap: both offending commits were docs-only, so a code-scoped check would not have fired regardless.`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `CHANGES-ENTRY-PLACEMENT.1` | `done` | Audited and registered. Measured: the two newest entries sit at lines 43706/43780, *below* the oldest entry in project history at 43642, in a heading convention retired in June; both commits **did** stage `CHANGES.md`, so only the placement is wrong. `scripts/check_diagnosis_evidence.sh:43` checks presence in the staged list and is scope-aware ⇒ docs-only commits, which both of these were, are exempt outright. |
| 2 | `CHANGES-ENTRY-PLACEMENT.2` | `pending` | **Next.** Nothing may move until the append-only question is answered and recorded. The defect is findability, not loss, so there is no pressure to act before deciding — and acting first is precisely how this lane has previously damaged a document (the `/tmp` sweep that rewrote decision `0030`'s own `reverify` line). |
| 3 | `CHANGES-ENTRY-PLACEMENT.3` | `pending` | Apply the decision, with byte-level proof the bodies are untouched. |
| 4 | `CHANGES-ENTRY-PLACEMENT.4` | `pending` | The mechanism question, last — because whether a gate is warranted depends on what `.2` concludes the correct end state even is. |

## Decisions

- `2026-07-31`: Registered as its own tree rather than handled inline while
  `LIVE-DOC-REGISTRY-SHADOWS.2` was in flight. Two reasons, both binding: the **pivot
  rule** (no new tree while the tree is dirty — `.2` was mid-flight when this was found),
  and the standing directive that **a defect is only handled if a task-tree owns it**,
  which makes registration mandatory rather than optional. Found while placing `.2`'s own
  `CHANGES.md` entry.
- `2026-07-31`: The repair is **explicitly gated behind a decision** rather than being
  treated as an obvious tidy-up. `CHANGES.md` is append-only by absolute owner directive;
  a tree that "just fixes" it would be exercising the judgement `0031` reserves.

## Blockers

- None. `.2` is a decision leaf and needs nothing but the measurement `.1` recorded here.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-07-31` | `CHANGES-ENTRY-PLACEMENT.1` | `measured at 087ca7b: CHANGES.md is 43843 lines and newest-first (line 4 = 2026-07-31, descending); the last heading before the defect is line 43642, "2026-04-15-0001 — Initial scaffold", the OLDEST entry in project history; the two newest entries sit at lines 43706 (BOOK-TEST-COUNT-SHADOWS.2, commit 715019b) and 43780 (LIVE-DOC-REGISTRY-SHADOWS.1, commit abf7090), i.e. BELOW it, in the "## YYYY-MM-DD-slug" heading convention retired after 2026-06-14 (249 headings in the current convention vs 245 in the retired one). Confirmed both commits DID stage CHANGES.md (git show --stat: +74 and +65 lines), so the mandatory-amendment rule was followed and only the placement is wrong. Confirmed scripts/check_diagnosis_evidence.sh:43 checks presence in the staged-file list only (grep -qx CHANGES.md) and is scope-aware, so a docs-only commit — which both of these were — is exempt outright` | `defect confirmed, pre-existing, live` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `CHANGES-ENTRY-PLACEMENT.1` | `CHANGES-ENTRY-PLACEMENT.1 — audit + register: the two newest entries are at the bottom` | Registration only; no repair attempted, deliberately — relocating a landed entry needs `.2`'s decision against `0031` first. |

## Changelog

- `2026-07-31`: Created while writing `LIVE-DOC-REGISTRY-SHADOWS.2`'s own `CHANGES.md`
  entry — the top of the file did not name the two changes that had just landed. The
  finding is not that two entries are in the wrong place; it is that **the file's
  ordering rule, the commit workflow that mandates it, and the doctrine check that
  gates the file are three layers none of which can see position**, so the error was
  free to happen twice in a row and would have happened a third time.
