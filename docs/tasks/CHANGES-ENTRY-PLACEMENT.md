# CHANGES-ENTRY-PLACEMENT: the two newest entries are at the bottom of a newest-first file

## Metadata

- Tree ID: `CHANGES-ENTRY-PLACEMENT`
- Status: `active`
- Roadmap lane: Live-doc hygiene / evidence findability
- Created: `2026-07-31`
- Last updated: `2026-07-31` (`.2` **done** — decision [`0038`](../decisions/0038-changes-md-position-repair-by-pointer.md): repair by **pointer stub**, never relocation; frontier `.3`)
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
  Status: `done`
  Goal: `Decide, and record as a decision, whether relocating a misplaced-but-landed CHANGES.md entry is permitted under decision 0031's append-only rule — distinguishing a correction of POSITION from a rewrite of CONTENT — and choose between relocation, an in-place forward pointer, or a re-published entry at the top that cites the misplaced original.`
  Acceptance: `The three options are stated with their costs, the owner's "keep it raw, keep honest" rationale is applied to each (a reader following the whole history must not be misled by the repair either), and the chosen option is recorded as a decision record before any content moves. The decision must state what it does NOT license, so it cannot be cited later to justify a general CHANGES.md sweep.`
  Verification: `done — decision 0038 recorded; NOTHING MOVED in this leaf. RULING: position is itself a record, so a landed entry is never moved, re-dated or re-titled; the repair is ADDITIVE — append a dated POINTER STUB at the position the entry should have occupied, naming where the entry actually is and why. 0031's LETTER names content; its REASON is evidentiary ("keep it raw, keep honest"), and that these two entries sit at the bottom is the evidence their author appended them there — so relocation would leave a file in which the mistake never happened, which is decision 0030's reverify accident repeated: mechanically rewriting the one document whose SUBJECT is the thing being rewritten. Both other options rejected with reasons: re-publishing the full bodies at the top mints a second copy of a 65- and a 74-line entry (a 0033 shadow, silent on divergence); an in-place forward pointer inside the originals IS a retro-edit and never reaches the top-down reader. RE-MEASURED FROM THE AUTHORITATIVE SET (all 646 headings at c758c6c), which narrowed and sharpened the finding six ways: (i) against GIT — the real ordering oracle — 388 of 646 entries carry a resolvable hash and their commit indices descend with ZERO violations, so the file has EXACTLY ONE ordering defect, not an unknown number; (ii) a DATE-keyed scan reports 3, of which 2 are FALSE — mis-dated headings over correctly-ordered entries (lines 9428/9477 headed 2026-06-18 are successors of 2f17147, committed 2026-06-21T13:28, which the entry at 9477 itself names as `previous:`; line 26652 headed 2026-05-13 committed 2026-05-14T23:38, rev numbers descending 274/272/270/267/265/264/262) — so the obvious mechanism cries wolf on 2 of 3; (iii) a HASH-keyed scan is VACUOUS for this exact defect — both misplaced entries carry NO `Landed as:` line, so the check horizon stops at line 39567, 4,516 lines above them: decision 0037's delete-the-subject test firing WITHOUT deleting anything; (iv) the two entries deviate in THREE ways, not two — placement, retired heading convention, AND the missing provenance line (present in 571 entries, absent in 75 = the 73 oldest as one contiguous run at lines 39703-43882 PLUS these two, making them the only post-adoption entries lacking it) ⇒ root cause is a STALE TEMPLATE, not an ordering slip; (v) .1's retired-convention count of 245 is one of TWO sub-forms — the true partition is 253 current + 393 retired = 646, the retired region being 245 word-slug + 148 numeric-slug (## DATE-NNNN) — decision 0033 rule (2) recurring, recorded not quietly corrected; (vi) line 32289 cites cf3dc3c164b0f8bb908d23d15b8248c275b683fb, which resolves to no commit in this repository — history, left raw. .3's exact placement is therefore DETERMINED, not left to judgement: two stubs after the entry at line 244 (LIVE-DOC-REGISTRY-SHADOWS.2, e873a6e) and before line 380 (BOOK-TEST-COUNT-SHADOWS.1, 1a6f276), ordered LIVE-DOC-REGISTRY-SHADOWS.1 (abf7090) then BOOK-TEST-COUNT-SHADOWS.2 (715019b). Six explicit non-licenses recorded so 0038 cannot be cited for a sweep. 0031 APPLIED, NOT AMENDED. Checks: check_doctrines.sh 8/8 after git add; cargo check --all-targets clean. Docs-only ⇒ DUT byte-identical.`
  Commit: `pending`

- ID: `CHANGES-ENTRY-PLACEMENT.3`
  Status: `pending`
  Goal: `Apply decision 0038: insert two dated POINTER STUBS at the position the misplaced entries should have occupied, so the top of CHANGES.md truthfully reflects the two most recent changes, and prove that not one byte of existing content changed.`
  Acceptance: `Placement is the one 0038 determined — after the entry at line 244 (LIVE-DOC-REGISTRY-SHADOWS.2, e873a6e), before line 380 (BOOK-TEST-COUNT-SHADOWS.1, 1a6f276), ordered LIVE-DOC-REGISTRY-SHADOWS.1 (abf7090) then BOOK-TEST-COUNT-SHADOWS.2 (715019b). Each stub carries ONLY join keys (date, leaf id, title, commit hash) plus the reason it exists — never a copy of the body, which would be a 0033 shadow. INSERTION ONLY: hash the file tail from the insertion point downward before and after and assert equality, and confirm `git diff` shows additions and no deletions or modifications. The misplaced originals are NOT touched. Re-run the git-order oracle afterwards and confirm it still reports zero violations. check_doctrines.sh 8/8.`
  Verification: `pending`
  Commit: `pending`

- ID: `CHANGES-ENTRY-PLACEMENT.4`
  Status: `pending`
  Goal: `Decide whether entry PLACEMENT warrants a mechanism, applying decision 0033's three-part test first, and either register one or record precisely why diligence is the right answer here.`
  Acceptance: `The three-part test is applied explicitly (derivable? growth-coupled? silent?) before any check is proposed — the answer may well be that a placement check cries wolf on legitimate edits to old entries and so is worse than nothing, which is a valid and recordable outcome. If a check IS written, it must survive the decision 0037 vacuity probe: delete the top entry and the check must fail. Note the scope trap: both offending commits were docs-only, so a code-scoped check would not have fired regardless. TWO CANDIDATE MECHANISMS ARE ALREADY DISQUALIFIED ON MEASUREMENT by .2 / decision 0038 and must not be re-proposed without new evidence: a DATE-keyed ordering scan cries wolf (2 false of 3 findings — mis-dated headings over correctly-ordered entries), and a HASH-keyed ordering scan is VACUOUS for this defect (the offending entries carry no `Landed as:` line, so its horizon stops 4,516 lines above them). .4 must therefore either find a third design or record that diligence wins. The open third candidate 0038 names: key the check on the AUTHORING PATH rather than the file — "the staged CHANGES.md diff adds lines above the current first heading" — derivable from `git diff --cached`, needing neither date nor hash, and it WOULD have fired on both offending commits despite their being docs-only. Also weigh the reframing 0038 forces: the three deviations share ONE cause (a stale template), so a gate that caught placement would still have let the missing provenance line through — the leverage may be in COMMIT.md step 2, not in a post-hoc gate.`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `CHANGES-ENTRY-PLACEMENT.1` | `done` | Audited and registered. Measured: the two newest entries sit at lines 43706/43780, *below* the oldest entry in project history at 43642, in a heading convention retired in June; both commits **did** stage `CHANGES.md`, so only the placement is wrong. `scripts/check_diagnosis_evidence.sh:43` checks presence in the staged list and is scope-aware ⇒ docs-only commits, which both of these were, are exempt outright. |
| 2 | `CHANGES-ENTRY-PLACEMENT.2` | `done` | Decided and recorded as [`0038`](../decisions/0038-changes-md-position-repair-by-pointer.md); **nothing moved**. Ruling: **position is itself a record**, so a landed entry is never moved — the repair is a **dated pointer stub appended at the position the entry should have occupied**. Re-measuring from the authoritative set (all 646 headings) narrowed the finding to **exactly one** real ordering defect (git's commit order over 388 hash-bearing entries reports **zero** violations) and killed both obvious mechanisms in advance: the date-keyed scan **cries wolf** (2 false of 3), the hash-keyed scan is **vacuous** (blind to the only real defect). |
| 3 | `CHANGES-ENTRY-PLACEMENT.3` | `pending` | **Next.** Apply `0038`: insert two stubs at a placement the decision already **determined** (after line 244's `LIVE-DOC-REGISTRY-SHADOWS.2`, before line 380's `BOOK-TEST-COUNT-SHADOWS.1`; `LIVE-DOC-REGISTRY-SHADOWS.1` first), with insertion-only proof — hash the tail from the insertion point down, before and after, and assert equality. |
| 4 | `CHANGES-ENTRY-PLACEMENT.4` | `pending` | The mechanism question, last. It starts from a stronger position than planned: two candidate designs are **already disqualified on measurement**, and `0038` names a third to weigh (key the check on the **authoring path** — "the staged `CHANGES.md` diff adds lines above the current first heading" — which needs neither date nor hash and *would* have fired on both offending commits) plus the reframing that the three deviations share one cause, a **stale template**, so the leverage may be in `COMMIT.md` step 2 rather than in a gate. |

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
- `2026-07-31` (`.2`, decision [`0038`](../decisions/0038-changes-md-position-repair-by-pointer.md)):
  **Repair by pointer stub; never by relocation.** `0031`'s letter names *content*, but its
  reason is evidentiary — and **position is itself a record**. That these two entries sit at
  the bottom is the evidence their author appended them there; moving them would leave a file
  in which the mistake never happened. Since this tree's *subject* is the misplacement, a
  relocation would be decision `0030`'s `reverify` accident repeated exactly: mechanically
  rewriting the one document whose subject is the thing being rewritten.
- `2026-07-31` (`.2`): **`.1`'s framing is corrected on measurement, in three places** —
  recorded rather than quietly fixed, because the *reasons* are the reusable part.
  **(a)** The file has **exactly one** ordering defect, not "at least one": against git —
  the authoritative oracle — 388 of 646 entries carry a resolvable hash and descend with
  **zero** violations. **(b)** `.1`'s retired-convention count of **245** is one of **two**
  sub-forms; the true partition is 253 current + 393 retired (245 word-slug + 148
  numeric-slug). Decision `0033` rule (2) recurring — sweeping for the shape of the instance
  in hand rather than from the authoritative set. **(c)** The entries deviate in **three**
  ways, not two: the third is a **missing `Landed as:` line**, and it is the diagnostic
  signature — all three deviations come from one **stale template**, not from an ordering
  slip.
- `2026-07-31` (`.2`): **Both obvious mechanisms are disqualified before `.4` opens.** A
  date-keyed ordering scan **cries wolf** (3 findings, 2 false — mis-dated headings over
  correctly-ordered entries), and *a gate that cries wolf gets deleted, taking its real
  coverage with it*. A hash-keyed ordering scan is **vacuous for this exact defect** — the
  offending entries carry no hash, so its horizon stops 4,516 lines above them. Decision
  `0037`'s *delete-the-subject* test fires here without deleting anything.

## Blockers

- None. `.3` is fully determined by `0038` — placement, order and proof obligation are all
  fixed — and needs nothing further.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-07-31` | `CHANGES-ENTRY-PLACEMENT.2` | `re-measured at c758c6c over ALL 646 entry headings, not just the defect .1 found. (1) HEADING CONVENTIONS partition the file exactly: 253 current (## DATE — LEAF — title, lines 4-16842) + 393 retired (## DATE-slug — TITLE, lines 16893-43882 plus the two strays) = 646; the retired region itself splits 245 word-slug + 148 numeric-slug (## DATE-NNNN), so .1's "245" was ONE SUB-FORM of two. (2) ORDERING vs GIT, the authoritative oracle: 388 of 646 entries cite a git-resolvable hash; scanned top-to-bottom their commit indices descend MONOTONICALLY with ZERO violations => the file has EXACTLY ONE ordering defect. (3) ORDERING vs HEADING DATES: 3 apparent violations, 2 of them FALSE — lines 9428/9477 headed 2026-06-18 carry commits 4d1b8c4/e68e2d1 which are SUCCESSORS of 2f17147 (committed 2026-06-21T13:28) and the entry at 9477 names 2f17147 as its own `previous:`; line 26652 headed 2026-05-13 carries f3ee1f3, committed 2026-05-14T23:38, with rev numbers descending 274/272/270/267/265/264/262. Both are mis-dated HEADINGS over correctly-ordered entries. (4) VACUITY PROBE on the hash-keyed oracle: the two misplaced entries carry NO `Landed as:` line, so they are invisible to it — its last visible entry is line 39567, 4,516 lines above the defect; it reports a clean file. (5) THE `Landed as:` LINE is present in 571 entries and absent in 75 = the 73 oldest as ONE CONTIGUOUS RUN (lines 39703-43882, verified: zero entries WITH the line below 39703) PLUS the two strays => the strays are the only post-adoption entries lacking it, making a stale template the single root cause of all three deviations. (6) line 32289 cites cf3dc3c164b0f8bb908d23d15b8248c275b683fb, which git rev-parse resolves to no commit in this repository — recorded, left raw per 0031. (7) PLACEMENT DETERMINED for .3 from git: after line 244 (LIVE-DOC-REGISTRY-SHADOWS.2, e873a6e), before line 380 (BOOK-TEST-COUNT-SHADOWS.1, 1a6f276), ordered LIVE-DOC-REGISTRY-SHADOWS.1 (abf7090) then BOOK-TEST-COUNT-SHADOWS.2 (715019b). Checks: cargo check --all-targets clean; check_doctrines.sh 8/8 after git add; mdbook untouched. NOTHING MOVED — decision leaf only. Docs-only => DUT byte-identical` | `decision 0038 recorded; scope narrowed to one proven defect; two candidate mechanisms disqualified on measurement` |
| `2026-07-31` | `CHANGES-ENTRY-PLACEMENT.1` | `measured at 087ca7b: CHANGES.md is 43843 lines and newest-first (line 4 = 2026-07-31, descending); the last heading before the defect is line 43642, "2026-04-15-0001 — Initial scaffold", the OLDEST entry in project history; the two newest entries sit at lines 43706 (BOOK-TEST-COUNT-SHADOWS.2, commit 715019b) and 43780 (LIVE-DOC-REGISTRY-SHADOWS.1, commit abf7090), i.e. BELOW it, in the "## YYYY-MM-DD-slug" heading convention retired after 2026-06-14 (249 headings in the current convention vs 245 in the retired one). Confirmed both commits DID stage CHANGES.md (git show --stat: +74 and +65 lines), so the mandatory-amendment rule was followed and only the placement is wrong. Confirmed scripts/check_diagnosis_evidence.sh:43 checks presence in the staged-file list only (grep -qx CHANGES.md) and is scope-aware, so a docs-only commit — which both of these were — is exempt outright` | `defect confirmed, pre-existing, live` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `CHANGES-ENTRY-PLACEMENT.1` | `CHANGES-ENTRY-PLACEMENT.1 — audit + register: the two newest entries are at the bottom` | Registration only; no repair attempted, deliberately — relocating a landed entry needs `.2`'s decision against `0031` first. |
| `CHANGES-ENTRY-PLACEMENT.2` | `CHANGES-ENTRY-PLACEMENT.2 — position is a record: repair by pointer, not relocation` | Decision [`0038`](../decisions/0038-changes-md-position-repair-by-pointer.md). Decision leaf — **nothing moved**. Also re-measured the property from the authoritative set and corrected `.1` in three places (see Decisions). |

## Changelog

- `2026-07-31`: Created while writing `LIVE-DOC-REGISTRY-SHADOWS.2`'s own `CHANGES.md`
  entry — the top of the file did not name the two changes that had just landed. The
  finding is not that two entries are in the wrong place; it is that **the file's
  ordering rule, the commit workflow that mandates it, and the doctrine check that
  gates the file are three layers none of which can see position**, so the error was
  free to happen twice in a row and would have happened a third time.
- `2026-07-31` (`.2`): The decision landed as [`0038`](../decisions/0038-changes-md-position-repair-by-pointer.md)
  — **repair by pointer stub, never by relocation**, because *position is itself a record*.
  Re-measuring the property rather than the instance changed the tree in three ways worth
  noting for whoever picks up `.3`/`.4`. It **narrowed** the scope: against git the file has
  exactly **one** ordering defect, so `.3` is a bounded, provable two-stub insertion rather
  than an open-ended audit. It **pre-emptied** `.4`: both obvious mechanisms are already
  dead on measurement — the date-keyed scan cries wolf (2 false of 3), the hash-keyed scan
  is vacuous (blind to the only real defect) — so `.4` opens with evidence instead of
  intuition. And it **renamed the root cause**: the three co-occurring deviations come from
  a **stale authoring template**, not an ordering slip, which points `.4` at `COMMIT.md`
  step 2 as much as at any gate. The general rule the project had not yet written down:
  **when history is wrong about itself, add a record — do not edit one.**
