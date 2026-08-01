# RESUME-POINTER-CONTRACT: the `MEMORY.md` byte cap fires every session, and the doctrine's prescribed remedy may be the wrong one

## Metadata

- Tree ID: `RESUME-POINTER-CONTRACT`
- Status: `closed` (`2026-08-01`) — **the hypothesis was measured and is FALSE. No defect. Do not reopen without new evidence.**
- Roadmap lane: Live-doc hygiene / memory-architecture
- Created: `2026-08-01`
- Last updated: `2026-08-01` (**closed same day**: the falsification test this file specified was run and killed the hypothesis)
- Owner: repo-local workflow

## The owner finding (verbatim, `2026-08-01`)

> *"the doctrine might be working, but it looks like it is still a problem, firing so often it
> not normal to me. Maybe the solution offered by the doctrine is not the most appropriate one."*

Raised after an agent reported the `MEMORY-ARCH` byte cap firing **three times in one session**
and framed it as *"the doctrine working as designed"*. The owner rejected that framing. This tree
exists because **the frequency of a gate firing is itself evidence about the gate's remedy**, and
nothing in the repo was treating it as such.

## Goal

Decide whether `MEMORY-ARCH`'s **remedy** — *"route content down to layer B/C and leave a
pointer"* — is the right response to a resume pointer that exceeds its byte cap, or whether the
cap is measuring a file whose real content model the four-layer standard does not have a home
for. **Not** to raise the cap: decision `0040` forbids that as a fix, and raising it would
re-saturate.

## The measurement (at `cf5deac`, `2026-08-01`)

| | value |
| --- | ---: |
| `MEMORY.md` actual | **30 lines / 6,043 bytes** = **201 B/line** |
| cap | 50 lines / 6,144 bytes = **122 B/line** as budgeted |
| the figure decision `0040` derived the cap from | **64 B/line** |
| line cap utilisation | **60 %** |
| byte cap utilisation | **98 %** |

**The line cap is not close to binding and the byte cap is at 98 %.** The file is written at
**1.7×** the density the cap was derived for, and **3.1×** the density `0040` measured as
demonstrated-achievable.

**The three largest lines are not "pointer" content:**

| line | bytes | what it actually is |
| ---: | ---: | --- |
| 26 | 995 | *Lane invariants* — eight cross-references with their rationale inlined |
| 15 | 812 | labelled `blockers: none.` — then ~800 bytes of **standing operating rules** |
| 12 | 570 | `active_work_unit` — a **summary of what three leaves did** (layer B/D content) |

Line 15 is the clearest exhibit: a field named **`blockers`** whose value is `none`, carrying
eight hundred bytes of unrelated doctrine.

## The diagnosis to test (this is a hypothesis, not a finding)

**The remedy is lossy in a way the standard does not acknowledge.** `MEMORY_ARCHITECTURE.md` §4
routes *"an operating gotcha"* to a fact card and *"a durable fact"* to a decision record, leaving
a **pointer** in layer A. But a pointer is not equivalent to the thing: it converts *"the next
session will see this"* into *"the next session may look this up"*.

So each session re-inlines the rules it most fears being ignored, layer A re-saturates, the cap
fires, the agent routes again — and the same content comes back next session under a different
heading. **Three firings in one session is that loop running three times, not three independent
overflows.**

If that is right, the missing piece is a **fourth category the four-layer model has no home
for**: *rules that must be re-read every session, not merely retrievable.* Note the repo already
has a guaranteed-read surface that is **not** layer A and **not** capped — `SESSION_BOOTSTRAP.md`,
which a `PostCompact` hook re-injects, and the harness bootstrap files. That is a candidate home,
and its existence is why this may be a **routing-destination** problem rather than a cap problem.

## Directions, none chosen

1. **Accept the tax.** The owner has already rejected this framing; recorded so the option is not
   silently dropped (`feedback_never_retire_strategies`).
2. **Re-home the "must be re-read" category** into the guaranteed-read bootstrap surface, leaving
   layer A genuinely a pointer. Cheapest if the diagnosis holds; needs the honest check that
   `SESSION_BOOTSTRAP.md` does not simply become the next uninstrumented overflow destination —
   **which is exactly `OVERFLOW-DESTINATION-INSTRUMENTATION`'s subject**, see Blockers.
3. **Scope the cap to the `Current state` block** rather than the whole file, so it measures the
   thing that is supposed to be overwrite-only. Risks fitting the measurement to the content.
4. **Re-derive the cap from a measured content model** rather than from `0040`'s 64 B/line
   assumption, which the file has never once achieved.

## Non-Goals

- **Not raising the cap as a fix.** `0040` and the check's own failure message both forbid it.
- **Not editing `MEMORY.md`'s content** before the direction is chosen — that is the loop above.
- **No code change.**

## Acceptance Criteria

- The diagnosis is **tested, not assumed**: count how many past commits repaired a cap firing and
  whether the routed content **returned** in a later session. If it did not return, the loop
  hypothesis is wrong and this tree closes.
- Whatever lands states which of the four categories in `MEMORY_ARCHITECTURE.md` §3 the
  re-inlined content belongs to, or states plainly that it belongs to none.
- Any cap change carries a **new decision record** stating that the resume-pointer contract
  itself expanded — never an edit to the constant.

## Blockers

- **Owner deferred the discussion (`2026-08-01`). Do not act without a nudge.**
- **Overlap to resolve before starting:** `OVERFLOW-DESTINATION-INSTRUMENTATION` owns *"a cap that
  redirects overflow must also check where the overflow lands"* and its `.2` **produced** the very
  byte cap in question (decision `0040`). That tree is itself **PAUSED by owner redirect**. Whether
  this is a new tree or a leaf of that one is the first thing to settle — `feedback_full_factorization`
  says one mechanism, and two trees over one subject would each hold half a picture.

## Changelog

- `2026-08-01`: Registered on an owner finding, with the measurement and no change made. The agent
  had reported the cap firing three times in one session as the doctrine working; the owner read
  the frequency as evidence against the remedy. Nothing is repaired here — this file exists so the
  finding survives a session boundary, per the standing rule that a defect is only handled once a
  task-tree owns it.

## Closure (`2026-08-01`) — the hypothesis is FALSE, measured

This tree specified its own falsification test in *Acceptance Criteria*: *"count how many past
commits repaired a cap firing and whether the routed content **returned** in a later session. If
it did not return, the loop hypothesis is wrong and this tree closes."* **The test was run. The
content does not return.**

| measurement (over the last **40** commits touching `MEMORY.md`) | result |
| --- | ---: |
| distinct phrases removed from the file at some point | **185** |
| phrases removed and later **re-added** | **1** |
| `MEMORY.md` byte range across those commits | **5,877 – 6,125 B** (95–99.7 % of the 6,144 cap) |

**The single re-add is not a loop:** it is *"a finding is not closed until something MECHANICAL
fails if it recurs"*, dropped at `ccfbc23` and deliberately restored at `6e95494` as a judgement
call, not a re-inlining reflex.

**So the diagnosis in this file is wrong, and the owner's reading was right.** The file is not
oscillating; it is in a **steady state** at the ceiling and has been for 40 commits across many
sessions, long predating the session that raised this. Routed content stays routed. The gate
blocks a commit, the author routes, the commit lands, nothing is lost. *That is a binding
constraint behaving normally at its limit* — high firing frequency is what a cap does to a file
that lives at 98 % of it, and frequency alone was never evidence about the remedy.

**The real defect was in the raising, not in the doctrine.** The agent observed three firings,
pattern-matched *"frequent ⇒ the remedy is wrong"*, and escalated it to an owner-facing finding
**without running the falsification test — which it had already written into this file's own
acceptance criteria.** That is this repo's recurring failure mode (a plausible narrative published
ahead of the measurement that would kill it), reproduced one level up: not a wrong *number* this
time, but a wrong *inference from a number*.

**Transferable rule:** *a gate firing often is not evidence that the gate's remedy is wrong.* It
is evidence that the constrained thing is at its limit — which is the constraint working. Before
escalating any "this mechanism misfires" claim, state what observation would prove it wrong and
**run that first**. Recorded as the fact card [[gate-frequency-is-not-evidence]].

**One honest residual, deliberately NOT inflated into a concern:** a blocked commit leaves no
trace in git, so "three firings" is knowable only from inside the session. That is an
observability gap in the *narration*, not a defect in the gate, and nothing depends on counting
them. Naming it so the next reader does not rediscover it and mistake it for a finding.

**Not reopened by:** the cap firing again. It will, and that is expected. Reopen only on evidence
that routed content **returns**, or that a firing caused something to be **lost** — neither of
which has ever been observed.
