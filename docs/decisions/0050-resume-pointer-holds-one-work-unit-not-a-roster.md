---
id: resume-pointer-holds-one-work-unit-not-a-roster
title: The resume pointer holds **one** work unit, not a roster — `MEMORY.md`'s `active_work_unit` was a `0033` shadow of the active-tree set that had already silently drifted to **7 of 16**, and the repair is **R1 deletion**, never a generator
answers:
  - "should MEMORY.md list every active task tree"
  - "why does MEMORY.md name only one work unit"
  - "MEMORY.md keeps hitting its byte cap, what is actually growing"
  - "is the active_work_unit field a shadow enumeration"
  - "should I generate the MEMORY.md current-state block"
  - "where is the list of all active task trees"
  - "how do I free space in MEMORY.md without trimming prose"
  - "a tree is paused but its file says active, where should the pause live"
date: 2026-08-07
status: accepted
tags: [memory-architecture, shadow-list, resume-pointer, doctrine, task-tree, growth, gotcha, north-star]
evidence: docs/tasks/RESUME-POINTER-COMMIT-PATH-COUPLING.md (`.1`'s measurement — 7 named of 16 active, and the 6,071 -> 5,235 B effect); MEMORY_ARCHITECTURE.md §6 (the template this restores, which is singular) and §12 (hand-maintained current-state is a listed anti-pattern); docs/decisions/0033-shadow-enumeration-classification.md (the three-part test and the R1 repair ladder); docs/decisions/0041-owner-standing-directives-recorded-in-layer-c.md (a directive parked in layer A is queued for deletion — the lesson that recurred here)
reverify: "python3 -c \"import re,glob,os;m=[l for l in open('MEMORY.md') if l.startswith('- active_work_unit')][0];named=set(re.findall(r'\\`([A-Z][A-Z0-9-]{4,})\\`',m));auth={os.path.basename(f)[:-3] for f in glob.glob('docs/tasks/*.md') if re.search(r'^- Status: \\`active\\`',open(f).read(),re.M)};print('named in MEMORY.md:',len(named),'| active on disk:',len(auth),'| ONE work unit is correct, a roster is the shadow')\"   # before this decision: 7 named, 16 active, 9 silently missing and no gate noticed"
---

# 0050 - RESUME-POINTER-COMMIT-PATH-COUPLING.1: the resume pointer holds one work unit

- Date: 2026-08-07
- Status: accepted
- Tree: `RESUME-POINTER-COMMIT-PATH-COUPLING.1`
- Activated by: an owner challenge — *"are you going to address the `MEMORY.md` grow issue? Because
  if not addressed it will bite us again."*

## Context

`.0` registered the coupling: a fail-closed byte cap on the path **97.6 %** of commits must take,
with **79 B** of slack and its prescribed remedy exhausted. That framed the *symptom*. `.1` asked the
question `.0` had not: **what, specifically, is growing?**

Measured at `339722b`, and the answer is one field:

| | |
| --- | ---: |
| trees named in `MEMORY.md`'s `active_work_unit` | **7** |
| trees whose own file says `Status: active` | **16** |
| **named but not active / active but not named** | **0 / 9** |
| bytes of that one line | **561** |

**The field is a hand-maintained list mirroring an already-authoritative set.** `0033`'s three-part
test passes on all three counts — *derivable* (from each tree's `Status` field, and from
`docs/TASK_TREE.md`'s *Active Task Trees* table, which is literally that set), *growth-coupled*
(every new tree must be added by hand), and *silent* — and the third is **demonstrated rather than
argued**: it had already drifted to **7 of 16**, a 44 % recall, and nothing failed. The
`MEMORY-ARCH` check asserts the *field name* is present; it has never had an opinion about the
field's *contents*.

So the growth was never prose bloat to be trimmed. It was **one shadow enumeration with a growth term
proportional to the number of task trees**, inside a file with a hard cap.

## Decision

**`active_work_unit` names ONE work unit and its frontier leaf. The roster of every active tree lives
in `docs/TASK_TREE.md` and is reached by pointer.**

This is not a new rule — it is `MEMORY_ARCHITECTURE.md` §6's own template, restored:

> `- active_work_unit: <TASK-TREE-ID>  →  frontier leaf: <LEAF-ID> (<status>)`

**Singular.** The field had accreted from that into a seven-item roster, and §12 already lists
*"hand-maintained current-state that drifts from reality"* as an anti-pattern. The standard was right
and the file had drifted away from it; nothing needed inventing.

**The repair is `0033` R1 — delete the copy, leave a pointer — and explicitly NOT a generator.**
`.0` recorded candidate **A** (*derive the current-state block*, which `MEMORY_ARCHITECTURE.md` §6 and
§11.7 both suggest) as the front-runner. It is **rejected here**, and the reason is the same rule that
disqualifies most second mechanisms: `docs/TASK_TREE.md` **already is** the derived roster. Generating
a second one into `MEMORY.md` would keep the copy and merely automate its maintenance — R4 on the
ladder where R1 is available — and `feedback_full_factorization` forbids a second mechanism for a job
that has one. **A generated shadow is still a shadow; it just drifts on a schedule instead of by
neglect.**

### The precondition nobody would have predicted

The deleted text carried a **safety flag with no other durable home**:
`OVERFLOW-DESTINATION-INSTRUMENTATION`'s *"PAUSED, do not resume without a nudge"*. Measured before
deleting: its own tree file said **`Status: active`** and contained the word *paused* **zero** times.
The pause lived in `MEMORY.md` — overwrite-only, hard-capped, queued for deletion by design — and in
other trees' prose.

That is decision `0041`'s lesson recurring exactly: **a directive parked in layer A is queued for
deletion.** Deleting the shadow without noticing would have silently un-paused a tree the owner
stopped, and PNT selection reads `Status`.

Fixed first, using the vocabulary's own word rather than inventing one: the tree is now `deferred` —
*"deliberately postponed with an explicit consequence"* — with the consequence stated in its own
Metadata. **Recording a pause is not resuming it**: no leaf was executed, measured or re-decided.

**The transferable rule:** *before deleting a copy, check every claim it carries for an independent
home — a copy that has quietly become the only original is not a copy.* `0033`'s ladder assumes the
authoritative set still exists; this is the case where it silently did not.

## Effect, measured

| | before | after |
| --- | ---: | ---: |
| `MEMORY.md` | 6,071 B | **5,235 B** |
| headroom to the 6,144 B cap | 73 B | **909 B** (**12.5×**) |
| growth term per new task tree | ~40–60 B, forever | **0** |

The third row is the durable one. The first two buy time; **decoupling growth from the tree count is
what stops it recurring**, which is what the owner asked for.

## Honest limits (§9 — stated, not discovered later)

- **This does not fix the coupling itself.** `COMMIT.md` still requires `MEMORY.md` amended before
  every commit and the cap is still fail-closed on that path. What changed is that the file no longer
  *grows* with the project. Candidate **B** (relax the mandate for state-free commits) stays open and
  undecided under `.2`.
- **`next_action` still carries a priority queue** that grows with the number of live threads. It is
  *not* a shadow — a priority ordering is not derivable from any set, and each tree's frontier stays
  the tree's own — but it is accretion, and `.2` owns it.
- **Nothing checks the new invariant.** No gate asserts `active_work_unit` names exactly one tree, so
  it can re-accrete. Deliberate: a check here would be a second mechanism over a one-line field, and
  the failure is now *visible* (a roster reappearing is obvious on sight) rather than *silent* (the
  drift to 7 of 16 was not). Reconsider only if it recurs.
- **The 9 trees that went unnamed for an unknown period stay unmeasured.** Nothing here reconstructs
  when each fell out, and re-deriving it would not change what to do.

## Explicit non-licenses

1. **Not a licence to raise the cap.** `0040` non-license 3 stands; this decision *lowers demand*
   rather than raising supply, which is the only move that ever fixes a cap.
2. **Not a resumption of `OVERFLOW-DESTINATION-INSTRUMENTATION`.** Its status was *recorded*, which is
   the opposite of resuming; every leaf keeps the status it had.
3. **Not a general licence to delete lists.** The test is `0033`'s three-part one plus the new
   precondition above — check for orphaned claims *first*.
4. **Does not amend `MEMORY_ARCHITECTURE.md` §6** — it restores compliance with it.
5. **Does not settle candidate B or C.** `.2` owns what remains.

## Consequences

- `MEMORY.md`'s `active_work_unit` is one unit plus a pointer; the roster is `docs/TASK_TREE.md`'s.
- `OVERFLOW-DESTINATION-INSTRUMENTATION` is `deferred` with its pause finally durable in its own file.
- `.2` inherits the residual accretion (`next_action`'s queue) and the still-open candidate B.
- The rule that outlives the file: **when a capped file keeps filling, find the term that scales with
  the project before trimming the prose that does not.**
