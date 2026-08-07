# RESUME-POINTER-COMMIT-PATH-COUPLING: a fail-closed size cap sits on the mandatory commit path, its prescribed remedy is exhausted, and the remedy actually taken is the one it forbids

## Metadata

- Tree ID: `RESUME-POINTER-COMMIT-PATH-COUPLING`
- Status: `active`
- Roadmap lane: Workflow / gate quality — commit path
- Created: `2026-08-07`
- Last updated: `2026-08-07` (registered)
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
  Status: `active`
  Goal: `The resume pointer can always be updated to name the current frontier, without an unrelated editorial tax on the commit that must update it.`
  Children: `.1` (measure the candidates and decide), `.2` (implement whatever `.1` chooses)

- ID: `RESUME-POINTER-COMMIT-PATH-COUPLING.1`
  Status: `pending`
  Goal: `Decide between candidates A-D, measured rather than argued.`
  Acceptance: `Measures candidate A against the real question that kills it or not: how much of the current Current-state block is DERIVABLE from git log + tree frontier rows, and how much is irreducible editorial judgement. That ratio decides A, and it is measurable today by attempting the derivation over the last N commits and diffing against what was actually written. States what is lost by the chosen candidate. Must NOT raise the cap (0040 non-license 3) and must NOT resume OVERFLOW-DESTINATION-INSTRUMENTATION. Recorded acceptance is a legitimate outcome.`
  Verification: `pending`
  Commit: `pending`

- ID: `RESUME-POINTER-COMMIT-PATH-COUPLING.2`
  Status: `pending`
  Goal: `Implement the decision from .1.`
  Acceptance: `Per .1. If A: the generator is deterministic (no clocks, sorted) so derive-and-diff is a valid sync gate, matching the KNOWLEDGE_MAP.md precedent; the hook regenerates and stages it; and the irreducible editorial part stays hand-written and is NOT overwritten by the generator.`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `RESUME-POINTER-COMMIT-PATH-COUPLING.1` | `pending` | **Next.** The measurement that decides candidate A — the derivable-vs-editorial ratio of the current-state block — is cheap and has never been taken. |
| 2 | `RESUME-POINTER-COMMIT-PATH-COUPLING.2` | `pending` | After `.1`, and only if `.1` chooses a mechanism. |

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
| `2026-08-07` | `.0` | `Measured at 4925847: git rev-list --count HEAD = 840 total, 820 touching MEMORY.md = 97.6%; MEMORY.md 6,064 B of a 6,144 B cap = 98.7%, headroom 79 B; band over the last 40 commits touching it 5,955 -> 6,064 B = +2.7 B/commit; the ## Current state block grew 2,250 -> 2,359 B (37.8% -> 38.9%) over the same window, so eviction pressure lands on ballast not on the pointer. Already-demoted sections carrying no further demotion: Standing directives 953 B, Operating gotchas 610 B, Validation policy 421 B. The gate's routing hint reads "Move content down, do not trim prose"; the remedy taken at 4925847 was a three-word prose rewording recovering 17 B, and no check compares remedy-taken against remedy-prescribed. ODI .3/.5a/.5b confirmed done by reading the tree, so the prescribed remedy was exhausted rather than skipped.` | `registered` (docs-only; DUT byte-identical) |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `.0` (registration) | `this commit` — `RESUME-POINTER-COMMIT-PATH-COUPLING.0 — register the resume-pointer commit-path coupling` | Docs-only. `.0` is this repo's registration-commit convention, required because `.githooks/commit-msg` rejects a subject naming no leaf. The same commit demonstrates the **prescribed** remedy — the `saturated instrument` gotcha was demoted out of `MEMORY.md` to a fact card, freeing 44 B — so the registration is not itself another prose trim. |

## Changelog

- `2026-08-07`: Created from an owner challenge to an agent's own "working as designed" framing.
  Measured rather than conceded: a fail-closed byte cap with **79 B** of slack sits on the path
  **97.6 %** of commits must take, its prescribed remedy (*demote content*) was **exhausted** by
  `OVERFLOW-DESTINATION-INSTRUMENTATION.5a`/`.5b`, and the remedy actually taken when it fired was
  the **prose trim the hint forbids** — unobserved, and reported as compliance. Four candidates
  recorded at registration, of which **A** (derive the current-state block) is the one
  `MEMORY_ARCHITECTURE.md` §6/§11.7 already prescribes and this repo never implemented, with a
  working in-repo precedent in the hook that regenerates and stages `KNOWLEDGE_MAP.md`.
