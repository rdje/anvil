# RESUME-POINTER-COMMIT-PATH-COUPLING: a fail-closed size cap sits on the mandatory commit path, its prescribed remedy is exhausted, and the remedy actually taken is the one it forbids

## Metadata

- Tree ID: `RESUME-POINTER-COMMIT-PATH-COUPLING`
- Status: `done`
- Roadmap lane: Workflow / gate quality — commit path
- Created: `2026-08-07`
- Last updated: `2026-08-08` (`.2` **done** — coupling removed; **tree closed**, both children `done`)
- Owner: repo-local workflow — **opened on an owner challenge**, not on an agent's own noticing

## Goal

`MEMORY-ARCH`'s byte cap on `MEMORY.md` is **fail-closed on the mandatory commit path**. Measured at
`4925847` (`2026-08-07`):

| Fact | Value |
| --- | --- |
| `COMMIT.md` mandate | *"`CHANGES.md` and `MEMORY.md` **MUST** be amended before every git commit, without exception."* |
| commits that touch `MEMORY.md` | **820 of 840 — 97.6 %** |
| `MEMORY.md` against its cap | **6,064 of 6,144 B — 98.7 %**, headroom **79 B** |
| drift rate over the last 40 commits touching it | **+2.7 B/commit** (5,955 → 6,064) |

So a hard cap with **79 bytes** of slack sits astride the path **97.6 %** of commits are *required*
to take. Every commit is a potential block, and clearing the block is editorial work on content
unrelated to the change being landed.

**The finding that makes this a defect and not a forecast** is what happens when it fires. The
gate's own routing hint reads:

> `Layer A is a POINTER, not a summary. Move content down, do not trim prose:`

At `4925847` it fired, and the remedy taken was **a prose trim** — three words reworded to recover
17 bytes. Not through carelessness: the prescribed remedy was **unavailable**. `## Standing
directives` (953 B), `## Operating gotchas` (610 B) and `## Validation policy` (421 B) had *already*
been demoted to pure pointers — by `OVERFLOW-DESTINATION-INSTRUMENTATION.5a`/`.5b`, which are
**`done`** — so there was no third demotion left to perform on them.

**And nothing observed the substitution.** The hint is stderr prose; no check compares the remedy
taken against the remedy prescribed. The commit message recorded it as compliance (*"trimmed the
next_action line rather than raising the cap"*) and the gate went green. The author was, in that
same session, the maintainer of the gate — which is the strongest available evidence that the
substitution is a property of the mechanism rather than of one careless author.

## Non-Goals

- **Not a proposal to raise or remove the cap.** Decision [`0040`](../decisions/0040-overflow-destination-classification-and-the-unmeasured-axis.md)
  non-license 3 requires a new decision record stating the resume-pointer contract expanded, never an
  edit to the constant, and the cap is doing real work — the file it replaced was 20,311 B with 65 %
  of it in the wrong layer.
- **Not a resumption of [`OVERFLOW-DESTINATION-INSTRUMENTATION`](OVERFLOW-DESTINATION-INSTRUMENTATION.md)**,
  which is paused and must not be resumed without an owner nudge. The boundary is **temporal and
  therefore sharp**: that tree's `.3` (switch the cap on) and `.5a`/`.5b` (demote the layer-C
  content) are all **`done`**, and this tree's subject is the state that remains *after* its remedy
  was fully applied — the demotion completed and the pressure did not abate. Nothing here re-opens,
  re-measures or re-decides any ODI leaf.
- **Not a reopening of `RESUME-POINTER-CONTRACT`'s falsified hypothesis.** That tree closed on
  *"routed content returns in a loop"*, measured **false** (1 re-add in 185 phrases) and re-confirmed
  at `4925847`. That claim stays dead. This is a different claim with different evidence: not that
  routed content comes back, but that **there is no longer anything left to route**.
- **Not a claim the gate misfires.** [[gate-frequency-is-not-evidence]] stands: frequency alone
  proves nothing. The evidence here is not frequency — it is *coupling* plus *remedy exhaustion*
  plus *a substituted remedy nobody observed*.

## Acceptance Criteria

- `.1` decides between the candidates below, measuring each rather than arguing it, and states what
  is lost by the one chosen. **Recorded acceptance is a legitimate outcome** ([`0047`](../decisions/0047-negative-control-carrier-is-the-mutation.md)).
- Any repair must preserve the property the cap bought: `MEMORY.md` stays a **pointer**, not a
  summary, and layer-C content stays in layer C. A repair that reopens the 20 KB blob is worse than
  the coupling.
- The `MEMORY_ARCHITECTURE.md` §6 line *"prefer derived over hand-written — a small script can
  regenerate the current-state block … so it cannot drift"* must be treated as a **candidate the
  standard already prescribes and this repo never implemented**, not as a new idea.

## Candidates (recorded at registration so `.1` measures rather than re-derives)

| # | Candidate | Why it might win | Its failure mode |
| --- | --- | --- | --- |
| **A** | **Derive the `## Current state` block** from `git log` + each tree's frontier row, regenerated and staged by the pre-commit hook | `MEMORY_ARCHITECTURE.md` §6 **and** §11.7 already prescribe it; §12 lists hand-maintained current-state as an anti-pattern; and the repo already runs this exact pattern for `KNOWLEDGE_MAP.md`, where the hook regenerates + stages so the agent spends **zero** time indexing. It is the `0047` R1 move: **removes the need** rather than watching harder. The block is **38.9 %** of the file and the part that grows | the current-state block carries editorial judgement a generator cannot produce — *why* a leaf is next, the warnings a future session needs. A generator that drops those replaces a good pointer with a correct-but-useless one |
| **B** | **Relax `COMMIT.md`'s "every commit, without exception"** for commits that change no state — hash backfills, typo fixes | attacks the **coupling** rather than the cap, and the exempt class is already recognised elsewhere: `CHANGES-ENTRY-PLACEMENT` skips commits that add no entry, and **99 of 766** commits were correctly skipped on exactly that basis | an exemption is a hole an author can widen; the mandate exists because a stale pointer is how sessions get lost |
| **C** | **Make the prescribed remedy checkable** — a check that a cap-relieving edit demoted content rather than reworded prose | closes the observed gap directly: the substitution went unnoticed | likely a §6.1 self-tick over prose, and a diff cannot reliably tell a demotion from a rewording. May be structurally disqualified like `0047` candidate C |
| **D** | **Nothing — record acceptance** | legitimate per `DOCTRINE_ENFORCEMENT.md` §9 when no mechanism genuinely helps | the runway is **~29 commits** at the measured rate, after which the only compressible surface left *is* the pointer block — and that is the loss [[gate-frequency-is-not-evidence]] says to reopen on |

## Task Tree

- ID: `RESUME-POINTER-COMMIT-PATH-COUPLING`
  Status: `done`
  Goal: `The resume pointer can always be updated to name the current frontier, without an unrelated editorial tax on the commit that must update it.`
  Children: `.1` (measure the candidates and decide) — `done`, `.2` (implement whatever `.1` chooses) — `done`

- ID: `RESUME-POINTER-COMMIT-PATH-COUPLING.1`
  Status: `done`
  Goal: `Decide between candidates A-D, measured rather than argued.`
  Acceptance: `Measures candidate A against the real question that kills it or not: how much of the current Current-state block is DERIVABLE from git log + tree frontier rows, and how much is irreducible editorial judgement. That ratio decides A, and it is measurable today by attempting the derivation over the last N commits and diffing against what was actually written. States what is lost by the chosen candidate. Must NOT raise the cap (0040 non-license 3) and must NOT resume OVERFLOW-DESTINATION-INSTRUMENTATION. Recorded acceptance is a legitimate outcome.`
  Verification: `The question was reframed by measurement: not "how much is derivable" but "what term GROWS". One field answered it. MEMORY.md's active_work_unit named 7 trees; 16 carry Status: active on disk; 0 named-not-active, 9 active-not-named. It is a 0033 shadow of docs/TASK_TREE.md's Active Task Trees table, and test (3) silence is DEMONSTRATED not argued — it had drifted to 7 of 16 with nothing failing, because MEMORY-ARCH asserts the field NAME exists and has never had an opinion about its contents. Candidate A (derive the block) REJECTED: docs/TASK_TREE.md already is the derived roster, so generating a second one keeps the copy and automates its upkeep — R4 where R1 is available, and feedback_full_factorization forbids the second mechanism. Repaired by R1 deletion: one work unit + pointer, per MEMORY_ARCHITECTURE.md §6's own singular template. Effect: MEMORY.md 6,071 -> 5,235 B, headroom 73 -> 909 B (12.5x), growth term per new tree 40-60 B -> 0. PRECONDITION FOUND AND FIXED FIRST: the deleted text carried OVERFLOW-DESTINATION-INSTRUMENTATION's pause, which had NO other durable home — its own file said Status: active and said "paused" zero times — so deleting the shadow would have silently un-paused a tree the owner stopped. Recorded there as deferred (the vocabulary's own status) before anything was deleted. All 11 doctrines green.`
  Commit: `this commit`

- ID: `RESUME-POINTER-COMMIT-PATH-COUPLING.2`
  Status: `done`
  Goal: `Implement the decision from .1, and settle the residues .1 left open — next_action's queue and candidate B.`
  Acceptance: `Per .1. If A: the generator is deterministic (no clocks, sorted) so derive-and-diff is a valid sync gate, matching the KNOWLEDGE_MAP.md precedent; the hook regenerates and stages it; and the irreducible editorial part stays hand-written and is NOT overwritten by the generator.`
  Verification: `Scoped by owner directive 2026-08-08, which is candidate B plus three items .1 had not reached. FINDING 1 — .1's recorded claim that next_action's queue is "not a shadow" is FALSE, and measurably: docs/TASK_TREE.md §PNT Selection Rules already says "choose the first active tree in the table", so the Active Task Trees row order IS the cross-tree authority. 0033's three tests pass, and (1) is proven by RECONSTRUCTION rather than argument — the queue's entries appear in EXACT table order, so it was that order minus one tree, not an independent judgement. (3) is DEMONSTRATED: TASK-LEAF-COMMIT-SHADOW sits at table position 3 with a pending leaf marked "Next." and is absent from the queue, while UNGATED-PRACTICE-AUDIT at position 33 is present; nothing failed. R1 deletion therefore loses no judgement, only the drift. FINDING 2 — latest_commit cannot be correct even once: written before the commit containing it exists, it can only name that commit's predecessor; observed at bootstrap reading f9e1c61 while HEAD was 596e624 (3 commits stale), with COMMIT.md institutionalising a per-slice backfill to keep it half-right. FINDING 3 — the CODE-CHANGE-EVIDENCE assertion was evidence-shaped without being evidence: CHANGES.md is a function of the DIFF (one entry describing this commit, falsifiable), MEMORY.md is a function of the WORK, so a commit changing no resumable state could discharge it only with a no-op diff in an overwrite-only hard-capped file — a 6.1 self-tick that also SPENT the cap's headroom, which is how 4925847 came to answer the cap with the prose trim its own hint forbids. ORPHAN CHECK RUN BEFORE DELETING (the .1 precondition): every claim in the queue is durably recorded in its own tree — BOOTSTRAP-READ-CONTRACT.3's two-contracts/intersection-2-files rationale in that leaf's Acceptance + frontier row + 0049 §(c); the other three in their trees' frontier row 1. Nothing orphaned. IMPLEMENTED: MEMORY.md classified bounded_snapshot with BOTH caps unchanged, latest_commit + predecessor list + decision range + queue + a dated card count deleted, one work unit and one next action kept; COMMIT.md's mandate split (CHANGES.md unconditional, MEMORY.md when resumable state changes) and its hash-backfill step deleted; check_diagnosis_evidence.sh drops the MEMORY.md leg; check_memory_architecture.sh gains a field-shaped forbidden-latest_commit assertion; MEMORY_ARCHITECTURE.md §3/§5/§6 corrected AT SOURCE so the deleted field is not re-imported by the next reader of the portable template. NEGATIVE-CONTROLLED BOTH WAYS via scripts/negative_control.sh (which refuses a zero-match substitution, so each result is a known-run experiment): reintroducing the field FIRES, a mid-sentence prose mention stays SILENT — the check discriminates its subject from a mention of its subject. CODE-CHANGE-EVIDENCE re-tested across a 5-row staged-set matrix: code+CHANGES.md now passes, code-without-CHANGES.md still fails, so the surviving leg is intact rather than weakened. Effect: MEMORY.md 5,235 -> 5,175 B, headroom 909 -> 969 B, commits REQUIRED to touch the file 97.6% -> only those changing resumable state, hash-backfill follow-up commits per slice 1 -> 0. Recorded in 0051. All 11 doctrines green.`
  Commit: `this commit`

## Current Frontier

**None — the tree is closed.** Both children are `done` and the top-level goal is met: the resume
pointer can be updated to name the current frontier without an editorial tax, because it is no longer
required on commits that change nothing in it.

| Order | Leaf | Status | Outcome |
| --- | --- | --- | --- |
| — | `RESUME-POINTER-COMMIT-PATH-COUPLING.2` | `done` | Coupling removed. `MEMORY.md` is a `bounded_snapshot` holding only non-derivable facts; `CODE-CHANGE-EVIDENCE` no longer asserts it. Recorded in [`0051`](../decisions/0051-the-resume-pointer-is-updated-when-resumable-state-changes.md). |
| — | `RESUME-POINTER-COMMIT-PATH-COUPLING.1` | `done` | Root cause was one shadow field, not prose bloat. Recorded in [`0050`](../decisions/0050-resume-pointer-holds-one-work-unit-not-a-roster.md). |

## Decisions

- `2026-08-07` (registration): **Registered because an owner challenge survived two dismissals.** The
  agent reported the cap firing as *"working as designed"*, was asked *"isn't that a problem?"*,
  measured the direction of the squeeze (the pointer block **grows**; ballast is what gives) and
  reported *"not a problem today"* — and was asked again, specifically about the **blocking** itself.
  Only that second challenge produced the coupling measurement (**97.6 %** of commits, **79 B** of
  slack) and the observation that the remedy taken was the forbidden one. Recorded because the
  pattern is the finding: [[gate-frequency-is-not-evidence]]'s rule was **correctly** cited and
  **wrongly** applied — it answers *"does frequency prove the remedy wrong?"* (no) and was used to
  answer *"is this coupling safe?"*, which it never addressed. **A card that answers a neighbouring
  question is the most expensive kind of wrong answer, because it arrives with evidence attached.**
- `2026-08-07` (registration): **The boundary against the paused tree is temporal, not rhetorical.**
  Stated explicitly under Non-Goals because a new tree adjacent to a paused one is exactly how a
  pause gets laundered. ODI's remedy leaves are `done`; this tree begins where they ended.

## Blockers

- None.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-08-08` | `.2` | `Shadow test on next_action's queue: reconstructed as docs/TASK_TREE.md's active-row order EXACTLY, minus TASK-LEAF-COMMIT-SHADOW (table position 3, pending leaf marked "Next."), while UNGATED-PRACTICE-AUDIT (position 33) is present — 3 of 15 active trees named, drift silent. §PNT Selection Rules already elects that table ("choose the first active tree in the table"), so test (1) holds and .1's "derivable from no set" is falsified. latest_commit measured stale by 3 commits at bootstrap (f9e1c61 vs HEAD 596e624). Orphan check run BEFORE deleting: all four queue entries durably recorded in their own trees (BOOTSTRAP-READ-CONTRACT.3 Acceptance + frontier + 0049 §(c); others in frontier row 1) — nothing orphaned. scripts/negative_control.sh probe x2 on the new MEMORY-ARCH assertion: field reintroduced -> FIRES (exit 1), mid-sentence prose mention -> SILENT (exit 0); baseline confirmed non-saturated first, and negative_control refuses a zero-match substitution so both experiments are known to have run. CODE-CHANGE-EVIDENCE staged-set matrix (5 rows via DOCTRINE_STAGED_OVERRIDE): code+CHANGES.md passes, code alone still fails, code+MEMORY.md-only still fails, docs-only exempt. bash scripts/check_doctrines.sh green on all 11. MEMORY.md 5,235 -> 5,175 B (30 lines), headroom 909 -> 969 B.` | `.2 done — tree closed` (docs/workflow-only; no `src/` touched, DUT byte-identical) |
| `2026-08-07` | `.1` | `Shadow test at 339722b: 7 named in MEMORY.md vs 16 Status: active on disk, 9 silently missing, 561 B for that one line. 0033 three-part test passes with (3) demonstrated by the live miss rather than by grep. Repair R1 deletion, candidate A rejected as R4-where-R1-exists. Effect 6,071 -> 5,235 B, headroom 73 -> 909 B, growth-per-tree -> 0. Orphan check run BEFORE deleting: ODI pause had no home outside MEMORY.md (own file said Status: active, zero occurrences of "paused"), recorded as deferred first. bash scripts/check_doctrines.sh green on all 11.` | `.1 done` (docs-only; DUT byte-identical) |
| `2026-08-07` | `.0` | `Measured at 4925847: git rev-list --count HEAD = 840 total, 820 touching MEMORY.md = 97.6%; MEMORY.md 6,064 B of a 6,144 B cap = 98.7%, headroom 79 B; band over the last 40 commits touching it 5,955 -> 6,064 B = +2.7 B/commit; the ## Current state block grew 2,250 -> 2,359 B (37.8% -> 38.9%) over the same window, so eviction pressure lands on ballast not on the pointer. Already-demoted sections carrying no further demotion: Standing directives 953 B, Operating gotchas 610 B, Validation policy 421 B. The gate's routing hint reads "Move content down, do not trim prose"; the remedy taken at 4925847 was a three-word prose rewording recovering 17 B, and no check compares remedy-taken against remedy-prescribed. ODI .3/.5a/.5b confirmed done by reading the tree, so the prescribed remedy was exhausted rather than skipped.` | `registered` (docs-only; DUT byte-identical) |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `.2` | `this commit` — `RESUME-POINTER-COMMIT-PATH-COUPLING.2 — the pointer is updated when resumable state changes` | Docs + workflow-config only (`scripts/check_*.sh` are not `src/`). Closes the tree. |
| `.1` | `596e624` — `RESUME-POINTER-COMMIT-PATH-COUPLING.1 — the growth was one shadow field` | Docs-only. Deleted the roster field; recorded in `0050`. |
| `.0` (registration) | `this commit` — `RESUME-POINTER-COMMIT-PATH-COUPLING.0 — register the resume-pointer commit-path coupling` | Docs-only. `.0` is this repo's registration-commit convention, required because `.githooks/commit-msg` rejects a subject naming no leaf. The same commit demonstrates the **prescribed** remedy — the `saturated instrument` gotcha was demoted out of `MEMORY.md` to a fact card, freeing 44 B — so the registration is not itself another prose trim. |

## Changelog

- `2026-08-08`: **Closed at `.2`** on an owner directive that both scoped the leaf and settled
  candidate **B**. The leaf's own finding is that `.1`'s recorded *"the queue is not a shadow — a
  priority ordering is derivable from no set"* was **false**: `docs/TASK_TREE.md` §PNT Selection
  Rules had already elected the *Active Task Trees* row order as the cross-tree authority, and the
  queue reproduced it **exactly**, minus one tree it had silently dropped. Recorded in
  [`0051`](../decisions/0051-the-resume-pointer-is-updated-when-resumable-state-changes.md), whose
  transferable rule is about gates rather than about this file: **a gate a compliant author can
  satisfy with a no-op diff is not measuring compliance — it is charging rent for it.** A second
  lesson worth keeping: `.1` closed by *stating* a residue's classification rather than measuring
  it, and the statement was wrong — **a claim recorded as a by-product of finishing something else
  gets none of the scrutiny the main finding got.**
- `2026-08-07`: Created from an owner challenge to an agent's own "working as designed" framing.
  Measured rather than conceded: a fail-closed byte cap with **79 B** of slack sits on the path
  **97.6 %** of commits must take, its prescribed remedy (*demote content*) was **exhausted** by
  `OVERFLOW-DESTINATION-INSTRUMENTATION.5a`/`.5b`, and the remedy actually taken when it fired was
  the **prose trim the hint forbids** — unobserved, and reported as compliance. Four candidates
  recorded at registration, of which **A** (derive the current-state block) is the one
  `MEMORY_ARCHITECTURE.md` §6/§11.7 already prescribes and this repo never implemented, with a
  working in-repo precedent in the hook that regenerates and stages `KNOWLEDGE_MAP.md`.
