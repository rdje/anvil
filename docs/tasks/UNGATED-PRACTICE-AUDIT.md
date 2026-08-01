# UNGATED-PRACTICE-AUDIT: a practice survives where a gate observes it — which practices here are unobserved?

## Metadata

- Tree ID: `UNGATED-PRACTICE-AUDIT`
- Status: `active`
- Roadmap lane: Workflow / gate quality
- Created: `2026-08-01`
- Last updated: `2026-08-02` (`.1` measured; the rule survives only in a **refined** form; frontier `.2`)
- Owner: repo-local workflow

## Goal

`NEGATIVE-CONTROL-HARNESS.1` measured one practice and found a general rule underneath it.

The practice was *"prove the mutation landed"*. The measurement: the **identical** primitive —
compare a file against a known baseline — is run **27 times across 6 trees** proving a control's
**revert** landed, and **twice** proving its **mutation** did. Same authors, same rule, same files.

The cause is not attention. A failed **revert** leaves a **dirty** tree, and three mechanisms punish
that (the pre-commit driver, `COMMIT.md` §10's `git status` review, the pivot rule). A failed
**mutation** leaves a **clean** tree, which all three read as success.

> **A practice survives where a gate observes it, and erodes where none does.**

That rule makes a **testable prediction**, and this tree exists to test it rather than to admire it:
*other* practices this project believes it follows are unobserved by any gate, and should therefore
be measurably eroding right now. The rule is either useful or it is a nice sentence; only a
measurement separates the two.

## The honest obstacle, stated up front

**This is a hypothesis, not a known defect.** No specific erosion has been measured. `.1` may find
that every unobserved practice here is in fact healthy — in which case the rule is *weaker* than
`NEGATIVE-CONTROL-HARNESS.1` implies, and saying so plainly is the deliverable. A tree that can only
confirm its own premise is not a measurement.

The second obstacle is **selection**: "practices this project believes it follows" is not a set
anyone has enumerated, and inventing one to fit the conclusion is the failure
`USER-GUIDE-CLI-TABLE-SHADOW.7` recorded as *do not manufacture a finding to justify a leaf*. `.1`
must derive the candidate set from something authoritative — `COMMIT.md`'s checklist, `TOOLBOX.md`
Part 2's boxes, the `DOCTRINE_ENFORCEMENT.md` §10 registry's complement — and state the derivation.

## Why a new tree rather than a leaf of an existing one

`feedback_full_factorization` applied, not waved through:

- `NEGATIVE-CONTROL-HARNESS` is **closed** and its subject was one practice, mechanised at `.3`.
  Re-opening it to hold a different, wider question would make its own record harder to read.
- `DOCTRINE-ENFORCEMENT-ADOPTION` owns *turning a doctrine into a check*. This tree asks the prior
  question — *which practices are not doctrines at all, and does that matter* — and its likely
  output is a **classification**, not a new check.
- `SHADOW-ENUMERATION-SWEEP` is the nearest shape (a defect **class** swept across the repo) and is
  closed; its subject was hand-maintained lists, not practices.

## Non-Goals

- **Not a mandate to gate everything.** `DOCTRINE_ENFORCEMENT.md` §9 is explicit that a gate which
  cries wolf gets deleted, and `0033` test (2) makes over-gating a defect in its own right. Several
  practices here are *correctly* ungated.
- **Not a re-statement of the rule.** It is already recorded in `DEVELOPMENT_NOTES.md` and decision
  `0047`. This tree measures whether it predicts anything.
- **No code change** in the generator sense.

## Acceptance Criteria

- `.1` **derives** its candidate set from an authoritative source rather than listing practices from
  memory, states the derivation and the denominator, and classifies each candidate as
  **gate-observed**, **unobserved-and-healthy**, or **unobserved-and-eroding** — with the evidence
  for each verdict, not an impression.
- *"The rule does not predict erosion here"* is a legitimate and fully acceptable result, and is
  stated plainly if that is what the measurement says.
- `.2` decides what, if anything, follows — for each eroding practice, whether the repair is a gate,
  a redesign that removes the need, or a recorded acceptance. Per `0047`'s precedent, *removing the
  need* outranks *watching harder*.
- `scripts/check_doctrines.sh` stays green; docs-only ⇒ DUT byte-identical.

## Task Tree

- ID: `UNGATED-PRACTICE-AUDIT`
  Status: `active`
  Goal: `Test whether "a practice survives where a gate observes it" predicts anything about THIS repo, and act only on what it predicts.`
  Children: `.1` (derive the candidate set and classify it), `.2` (decide what follows, per candidate)

- ID: `UNGATED-PRACTICE-AUDIT.1`
  Status: `done`
  Goal: `Derive the candidate set of practices from an authoritative source, state the denominator, and classify each as gate-observed, unobserved-and-healthy, or unobserved-and-eroding — on evidence.`
  Acceptance: `The derivation is stated and re-runnable, not a remembered list. Each verdict cites what was measured. "No erosion found" is a legitimate result and must be reported as such rather than reframed. Where a practice is unobserved AND healthy, say why it survives without a gate — that is the more interesting half.`
  Verification: `20 atomic obligations derived from COMMIT.md's checklist by a recorded command; classified 8 gate-observed / 1 gate-observed-but-blind / 5 unobserved-and-healthy / 1 unobserved-and-eroding / 5 not-measurable. The rule as stated FAILED its own test on the one eroding case and was refined; see Findings.`
  Commit: `pending`

- ID: `UNGATED-PRACTICE-AUDIT.2`
  Status: `pending`
  Goal: `Decide what follows for each eroding practice — a gate, a redesign that removes the need, or a recorded acceptance — before building anything.`
  Acceptance: `Per candidate, with its failure mode stated. Removing the need outranks adding a gate (decision 0047's R1-over-R2 precedent). Over-gating is a defect, not thoroughness.`
  Verification: `pending`
  Commit: `pending`

## Findings (`.1`, measured `2026-08-02` at `1dedbd8`)

### The derivation, and why this source

**Chosen:** `COMMIT.md` § *Non-negotiable pre-commit checklist*. Re-runnable, not remembered:

```bash
# the 12 numbered items
awk '/^## Non-negotiable pre-commit checklist$/{f=1;next} /^## /{f=0} f && /^[0-9]+\. \*\*/' COMMIT.md
# item 1's four explicit sub-boxes
awk '/^## Non-negotiable pre-commit checklist$/{f=1;next} /^## /{f=0} f && /^   - \[ \]/' COMMIT.md
```

**Denominator: 12 numbered items ⇒ 20 atomic obligations.** Items 1, 2, 3 and 11 each bundle more
than one obligation (item 1 carries four `[ ]` sub-boxes *plus* the snapshot-acceptance protocol;
item 2 carries *new entry at top* **and** *previous entry's hash backfilled*; item 3 carries *state
refreshed* **and** *previous hash recorded*; item 11 carries *trailer present* **and** *file stays
untracked*). The decomposition is stated so the denominator can be disputed rather than assumed.

**Why this source.** It is the only candidate that is normative for *every* commit — *"If any item
cannot be affirmatively answered, the commit does not proceed"* — which is exactly *"practices this
project believes it follows"*. It is also **self-ticked**, the property that makes the hypothesis
testable at all (`DOCTRINE_ENFORCEMENT.md` §6.1: a tick is a claim, not proof).

**What the other two would have given, reported rather than waved off:**

- `TOOLBOX.md` Part 2 — `awk '/^## Part 2/{f=1;next} /^---$/{if(f)f=0} f && /^- \[ \] \*\*/' TOOLBOX.md`
  yields **8 boxes**. It declares itself a *"Mirror of the `COMMIT.md` non-negotiable checklist"*, so
  using both measures one set twice (`feedback_full_factorization`). Worse for this question: every
  one of its boxes **already cites a named re-runnable oracle**, so its membership rule pre-selects
  the gate-observed subset — it would have answered the question with its own selection criterion.
- The **complement of the `DOCTRINE_ENFORCEMENT.md` §10 registry** — **not derivable, and that is a
  result rather than a shrug.** A complement needs a universe, and *"all practices this project
  follows"* is precisely the set nobody has enumerated; that is the tree's own stated obstacle.
  Inventing the universe to subtract from is the `USER-GUIDE-CLI-TABLE-SHADOW.7` failure
  *do not manufacture a finding to justify a leaf*.

### The classification (20 of 20 accounted for)

**Gate-observed — 8.** A registered mechanical check fails if the obligation is not met.

| # | Obligation | What observes it | Measured |
| --- | --- | --- | --- |
| 1–4 | `cargo check --all-targets` · `cargo test` · `cargo clippy --all-targets -- -D warnings` · `cargo fmt --all --check` | CI E4 (`.github/workflows/ci.yml` runs fmt, clippy, test) | green at `1dedbd8`: `fmt`/`check`/`clippy` exit `0`, `cargo test` **1,087 passed / 0 failed / 19 ignored across 17 targets**, exit `0`. **E4 observation lag: 159 of 811 commits unpushed** — `origin/main` is `ecda0e7`, confirmed against the remote, so the newest 159 commits have been seen by E3 only |
| 5 | snapshot-acceptance protocol (no unexplained `.snap`) | `cargo test` → `tests/snapshots.rs` | green in the same run |
| 6 | `CHANGES.md` new entry at the **top** | `CHANGES-ENTRY-PLACEMENT` (E3+E4) | driver 11/11 |
| 17 | only intended files staged | `.gitignore` — mechanical, not a doctrine | **0** of 811 commits ever staged `target/`, `.cache/`, `.claude/settings.local.json`, `.claude/worktrees/` or `book/book-out/` |
| 19 | `git_message_brief.txt` stays untracked | `.gitignore` | **0** commits in all of history tracked it |

**Gate-observed but blind — 1.** The third class the tree predicted: watched by something that
cannot see the thing.

| # | Obligation | What observes it | Measured |
| --- | --- | --- | --- |
| 9 | `MEMORY.md` current state / next-up / open questions **refreshed** | `MEMORY-ARCH` | the check asserts the **presence of four field names** (`active_work_unit`, `next_action`, `in_flight_uncommitted`, `blockers`) and the line/byte caps. It never asserts the content is *fresh*, and it does not mention `latest_commit` at all. **Presence is not refresh** — a file frozen for fifty commits passes every leg |

**Unobserved and healthy — 5.** Nothing would fail if these were dropped, and they were not dropped.

| # | Obligation | Measured | Why it survives |
| --- | --- | --- | --- |
| 8 | previous `CHANGES.md` entry's landed hash backfilled | **621 of 623 since adoption** (entry `2026-04-17-0074`); the 2 exceptions are `POINTER STUB` entries that by design carry no hash; the other 75 are one contiguous pre-adoption block at the file's oldest end | the hash is printed by the commit that just ran — the author is holding it |
| 10 | `MEMORY.md` recent-commits carries the **previous** commit's hash | over the newest 250 commits: **204 name the parent exactly**, **28 name the grandparent because the parent was a `backfill` commit** (a legitimate convention — the *previous slice*), 7 predate the hash format ⇒ **232/250 = 92.8 % compliant**. **11 genuinely stale, every one of them older than `2026-07-30`; 0 in the newest 108 commits.** Worst observed staleness: 3 consecutive commits frozen 28–30 behind (`c6a9037`/`4bd42d0`/`4564533`, `2026-06-22`) | same artifact as #8; **improving, not eroding** |
| 11 | `DEVELOPMENT_NOTES.md` when the slice adds rationale | scored by `COMMIT.md` item 4's **own** audit rule (a run of `src/`-touching commits with no `DEVELOPMENT_NOTES.md` update in that commit *or since*): **current run 0** (last touched at `4ad09a4`, `2026-08-01`), historical worst **15** | item 4 states its own detector, and it is cheap to run |
| 18 | commit message carries the co-author trailer | **756 of 811**; all 55 misses sit at history positions 29–192 (all `≤ 2026-04-30`); **619 consecutive compliant commits since** | emitted by the **authoring harness**, not by a repo gate — forced upstream of the author |
| 20 | post-commit `truncate -s 0 git_message_brief.txt` | **0 bytes now**. Historically unobservable: the file is gitignored, so no past state of it survives anywhere | see *Honest limits* — this verdict is a present-state check, not a history |

**Unobserved and eroding — 1.**

| # | Obligation | Measured |
| --- | --- | --- |
| 7 | the `CHANGES.md` entry carries **What / Why / Validation / Impact / Files touched** | **nothing reads any of the five** — verified, not assumed: `grep -lnE 'Validation\|Files touched\|\*\*Impact\|\*\*Why' scripts/*.sh knowledge-map/scripts/*.sh` matches only a **comment** in `check_enumeration_parity.sh`, and `check_diagnosis_evidence.sh` asserts `CHANGES.md` and `MEMORY.md` are **staged**, never what they contain. Label conformance over 698 entries fell from **100 % (entries 401–500) to 32 % (newest 50)**. Newest 50, per section: `Files touched.` **50/50**, `What.` **47/50**, `Validation.` **47/50**, `Why.` **32/50**, `Impact.` **25/50** |

**Not measurable from the tree — 5.** Stated rather than proxied.

| # | Obligation | Why not measurable | What *is* observed |
| --- | --- | --- | --- |
| 12–16 | `CODEBASE_ANALYSIS.md` · `ROADMAP.md` · `USER_GUIDE.md` · `README.md` · `book/src/*.md`, each **conditional** on *"if the slice changed X"* | the repo does not record whether a slice changed X, so any co-update-frequency proxy measures **frequency, not compliance**. Building one and calling it erosion is the failure this tree was told to avoid | named axes *are* gated: `ENUMERATION-PARITY` (three `USER_GUIDE.md` CLI tables, `SUMMARY.md` ↔ chapters, the downstream allow-list), `README-GROWTH` (size), `BOOK-LINK-TARGETS` (link escape), CI `mdbook build` + `mdbook test` |

### The control group (declared out-of-denominator)

The **leaf-id-in-subject** rule is gated by `.githooks/commit-msg`, which landed at `2d01e8e`
(`2026-06-05`). In the **410 commits since: 0** subjects fail its regex. Over the *same* window,
obligation #7 — same authors, same commits, same files, no gate — fell from 100 % to 32 %.

The related task-tree rule **one leaf per commit** is *ungated* and also measured clean: **0
violations in 811 commits.** (First pass reported 8; the detector was crying wolf on `MEMORY.md`,
`COMMIT.md` and friends, which match a leaf-id shape. Corrected by excluding file-extension tokens.)

### The rule did **not** survive its own test as stated — it survives refined

All five of obligation #7's sections are **equally ungated**, yet three held at 94–100 % and two
collapsed. Gate-presence therefore **cannot** be the discriminator. What discriminates is whether the
section's content is a **by-product of something the author is already forced to produce**:

| Section | Newest 50 | The by-product behind it |
| --- | --- | --- |
| `Files touched.` | 100 % | `git diff --stat`, forced by checklist item 10 |
| `What.` | 94 % | the diff itself |
| `Validation.` | 94 % | the four command runs, forced by item 1 |
| `Why.` | 64 % | **none** — recall and composition only |
| `Impact.` | 50 % | **none** — recall and composition only |

> **Refined rule: a practice survives where its output is a by-product of work the author is already
> forced to do. A gate is one way to force that — it is not the only way, and it is not the operative
> variable here.**

The refinement is strictly stronger, because it also explains the five healthy unobserved practices
(#8 and #10 fall out of the commit that just ran; #18 is emitted by the authoring harness; #19 by
`.gitignore`; #11 by item 4's own stated detector) **and** re-explains the `NEGATIVE-CONTROL-HARNESS.1`
observation that started this tree: a failed *revert* leaves a dirty tree as a by-product, a failed
*mutation* leaves nothing. The original rule predicts no difference among five equally-ungated
sections; this one predicts the observed split. This answers Open Question 3 on evidence.

### A correction to the erosion finding, from reading the entries

Six of the newest non-conforming entries were read in full rather than scored. In **all six**, the
*Why* content and *Impact* content are **present** — under bespoke headings (`**The finding.**`,
`**The claim under test.**`, `**The measurement.**`, `**Docs-only; no `src/` change** ⇒ **DUT
byte-identical**`). The substance did not erode; the **form** did. The honest statement is therefore:

> `COMMIT.md` item 2 prescribes a five-section template the repo stopped following roughly 150 entries
> ago. The practice moved on — arguably improved — and the specification did not follow, because
> nothing reads it. The concrete cost is not lost information: it is that **item 2 is now unverifiable
> by anything short of reading the prose**, so it cannot be gated as written, and an auditor scoring
> the project against its own document would record 32 %.

That is erosion of the **specification**, not of the practice — a fourth class this tree did not
predict, and the one `.2` has to decide about.

### The instrument's own control failed first — recorded, not buried

Control B (a constructed two-entry fixture — decision [`0047`](../decisions/0047-negative-control-carrier-is-the-mutation.md)
rung **R1**, no match step to silently no-op) was run against the `Landed as:` detector **before** its
number was reported, and **it failed**: the detector matched the marker inside a code span and
false-negatived. Anchored to line start, the count moved **75 → 77**; both additions are `POINTER STUB`
entries that legitimately carry no hash, so the verdict is unchanged and the instrument is now sound.
Recorded because `NEGATIVE-CONTROL-HARNESS.1`'s entire subject is that this is the step that gets skipped.

### One live violation, measured in this session, out of denominator

Decision [`0031`](../decisions/0031-ssd-volume-exclusivity.md) §1 puts **"agent scratch"** inside the
storage obligation. `scripts/check_no_boot_volume_refs.sh` scopes itself to **tracked files**, with the
comment *"untracked scratch is the author's business"* — narrower than the decision it enforces. This
session wrote three scratch files to a boot-volume path before re-reading `0031`; the driver was green
throughout, because it structurally cannot see them.

Control C, run on-volume so it does not itself violate the doctrine: the same banned string in an
**untracked** file ⇒ check exits `0`; the identical file **staged** ⇒ check fails, naming it. The gap is
structural, not an accident of this session.

This is the tree's clearest single instance — a doctrine the author had **already read**, violated
within twenty minutes, invisible to all four enforcement layers, caught only by re-reading the decision
record. It is not a `COMMIT.md` checklist item, so it is reported outside the denominator; `.2` owns it.

### Honest limits of this measurement

- **Obligation 20 has no history.** `git_message_brief.txt` is gitignored, so only its present state is
  knowable. "Healthy" here means *healthy now*, and no stronger claim is available.
- **The 32 % is label conformance, not information loss** — established by reading six entries, above.
- **Five obligations are unmeasured**, not silently passed. Their predicate is not recorded anywhere.
- **The 159-commit E4 lag is a stated honest limit** (`DOCTRINE_ENFORCEMENT.md` §9), not a finding; the
  push cadence is an owner-set risk appetite recorded in [`0041`](../decisions/0041-owner-standing-directives-recorded-in-layer-c.md) §(d).
- **`n = 5`** for the newest bucket of `src/`-touching commits — too small to read a trend from, and it
  is not read as one.

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `UNGATED-PRACTICE-AUDIT.2` | `pending` | **Next.** `.1` produced exactly one eroding candidate and it is a **stale specification**, not a lapsed practice — so the repair is almost certainly to fix `COMMIT.md` item 2, not to gate it. Two further items need a verdict: obligation 9's blind gate, and the `0031`-versus-`check_no_boot_volume_refs.sh` scope gap. Per `0047`, *removing the need* outranks *watching harder*. |
| — | `UNGATED-PRACTICE-AUDIT.1` | `done` | Measured `2026-08-02`. The rule failed as stated and was refined; see Findings. |

## Decisions

- `2026-08-01` (registration): **Registered rather than left as prose.** The rule is recorded in
  `DEVELOPMENT_NOTES.md` and decision `0047`, but a *prediction* recorded only as narrative is never
  tested — and the standing directive is that a finding is handled only when a tree owns it.
  Registered at a clean tree boundary, immediately after `NEGATIVE-CONTROL-HARNESS` closed, per the
  pivot rule.
- `2026-08-01` (registration): **Framed as a hypothesis under test, not a defect to repair.** No
  erosion has been measured. Framing it as a known problem would pre-load `.1` toward finding one,
  which is exactly how a confident wrong answer gets manufactured.
- `2026-08-02` (`.1`): **`COMMIT.md`'s checklist chosen as the denominator over `TOOLBOX.md` Part 2.**
  Part 2 declares itself a mirror of the same set, so using both measures one thing twice; and every
  Part-2 box already cites a named oracle, so its membership rule pre-selects the gate-observed half.
  A source whose selection criterion *is* the variable under test cannot measure that variable.
- `2026-08-02` (`.1`): **The `§10`-complement source is reported as not derivable rather than
  approximated.** A complement needs a universe; the universe here is the unenumerated set the tree
  was opened about. Approximating it would have manufactured the denominator.
- `2026-08-02` (`.1`): **Five obligations are reported unmeasured rather than proxied.** Their
  predicate (*"if the slice changed X"*) is not recorded anywhere in the repo, so a co-update-rate
  proxy would measure frequency and be reported as compliance. An unmeasured cell is honest; a
  confident wrong one is the failure mode this tree exists to avoid.
- `2026-08-02` (`.1`): **The rule is refined rather than confirmed.** Its literal form predicts no
  difference among five equally-ungated sections, and the measurement found a 50-point spread. The
  refinement (*by-product of forced work*, not *gate*) is adopted because it explains the spread, the
  five healthy unobserved practices, and the original `27`-to-`2` observation — and it demotes
  gating from *the* mechanism to *one* mechanism, which is what `0047`'s R1-over-R2 already implies.

## Open Questions

- **What is the authoritative source for "practices this project believes it follows"?**
  **Answered at `.1`:** `COMMIT.md`'s non-negotiable checklist, for the reasons in *Decisions*; the
  other two candidates are reported (8 boxes; not derivable).
- **Is "observed by a gate" binary?** **Answered at `.1`: no — measured.** Obligation 9 is watched by
  `MEMORY-ARCH`, which asserts four field *names* are present and never that their content is fresh.
  The predicted third class exists and has exactly one member in this denominator.
- **Does the rule survive its own test?** **Answered at `.1`: not as stated.** The discriminator is
  whether the output is a **by-product of forced work**, not whether a gate watches. See Findings.
- **New — does a stale specification count as erosion, and who repairs it?** The single eroding
  candidate is a `COMMIT.md` template the practice outgrew, not a lapsed practice. `.2` decides
  whether the repair is to rewrite item 2 to describe what entries actually do (and can then be
  gated), or to record that the five-section form was superseded.
- **New — should `check_no_boot_volume_refs.sh`'s tracked-only scope be widened, or `0031`'s wording
  narrowed?** They currently disagree about untracked agent scratch, and `.1` violated the decision
  under a green gate. Widening a gate to watch untracked files is expensive and noisy; the `0047`
  R1 move would be to remove the need — e.g. make the on-volume scratch path the one an agent
  reaches for first. `.2` decides; this leaf only measured it.

## Blockers

- None.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-08-01` | `.0` | `registered from NEGATIVE-CONTROL-HARNESS.1's measured finding — 27 recorded verifications that a control's REVERT landed against 2 that its MUTATION did, with the asymmetry explained by which side a gate watches. Ownership search run, not assumed: NEGATIVE-CONTROL-HARNESS is closed and held one practice; DOCTRINE-ENFORCEMENT-ADOPTION owns turning a doctrine into a check, not deciding whether a practice needs one; SHADOW-ENUMERATION-SWEEP is closed and held hand-maintained lists.` | `registered` (docs-only; DUT byte-identical) |
| `2026-08-02` | `.1` | `Denominator DERIVED by recorded command from COMMIT.md's checklist: 12 items ⇒ 20 atomic obligations; both alternative sources reported (TOOLBOX Part 2 = 8 boxes, pre-selected toward gate-observed; the §10 complement = not derivable). Classified 8 gate-observed / 1 gate-observed-but-blind / 5 unobserved-and-healthy / 1 unobserved-and-eroding / 5 not-measurable. Evidence: 811 commits, 698 CHANGES.md entries, 250-commit MEMORY.md hash trace. Control group (leaf-id-in-subject, gated at 2d01e8e): 0 failures in 410 commits. Instrument control B FAILED first and was fixed (75 → 77, verdict unchanged). Control C proved the boot-volume check is tracked-only by construction. scripts/check_doctrines.sh 11/11; cargo fmt/check/clippy exit 0; cargo test green.` | `measured — the rule does NOT hold as stated and is refined; one eroding candidate found, and it is a stale SPECIFICATION rather than a lapsed practice` (docs-only; DUT byte-identical) |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `.0` (registration) | `ef6413c` — `UNGATED-PRACTICE-AUDIT.0 — register the generalisation from NEGATIVE-CONTROL-HARNESS.1` | Docs-only; no work leaf executed yet. `.0` is this repo's registration-commit convention, required because `.githooks/commit-msg` rejects a subject naming no leaf. |
| `.1` | `UNGATED-PRACTICE-AUDIT.1 — 20 obligations classified; the rule holds only refined` | Docs-only ⇒ DUT byte-identical. |

## Changelog

- `2026-08-02`: `.1` measured. Denominator derived by recorded command from `COMMIT.md`'s
  non-negotiable checklist — **12 items ⇒ 20 atomic obligations** — with both rejected sources
  reported (`TOOLBOX.md` Part 2 gives **8** boxes but pre-selects the gate-observed half; the §10
  complement is **not derivable** and that is the result). Classified **8** gate-observed, **1**
  gate-observed-but-blind, **5** unobserved-and-healthy, **1** unobserved-and-eroding, **5**
  not-measurable-and-reported-as-such. **The rule does not hold as stated.** Its one eroding
  candidate — `CHANGES.md`'s five-section template, **100 % → 32 %** — has all five sections equally
  ungated, yet three held at 94–100 % and two collapsed, so gate-presence cannot be the
  discriminator. What discriminates is whether the content is a **by-product of work the author is
  already forced to do**; the refined rule additionally explains all five healthy unobserved
  practices and the original **27-to-2** observation. Reading six of the non-conforming entries
  corrected the finding further: the *substance* is present under bespoke headings, so what eroded
  is the **specification**, not the practice — a fourth class the tree did not predict. The gated
  control group (leaf-id-in-subject, gated at `2d01e8e`) measured **0 failures in 410 commits** over
  the same window. The leaf's own `Landed as:` detector **failed its R1 control** before its number
  was reported and was fixed (**75 → 77**, verdict unchanged). One live out-of-denominator violation
  was measured in-session: decision `0031` §1 covers **agent scratch**, but
  `check_no_boot_volume_refs.sh` is **tracked-only by construction** (proved by control C), so three
  boot-volume scratch files were written under a fully green driver by an author who had read the
  decision twenty minutes earlier.
- `2026-08-01`: Created. `NEGATIVE-CONTROL-HARNESS.1` measured one practice and found a general rule
  underneath it — **a practice survives where a gate observes it, and erodes where none does** —
  evidenced by a **27-to-2** split between proving a control's *revert* landed and proving its
  *mutation* did, explained entirely by which of the two leaves a dirty tree behind. The rule makes a
  testable prediction about *other* practices here, and a prediction recorded only as narrative is
  never tested. Registered as a hypothesis under test rather than as a defect: **no erosion has been
  measured**, and *"the rule does not predict anything here"* is an acceptable and fully reportable
  outcome.
