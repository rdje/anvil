---
id: owner-standing-directives-recorded-in-layer-c
title: The owner's standing directives are **recorded here**, in layer C, not in the resume pointer — thirteen citations across nine decision records pointed at `MEMORY.md`, which is overwrite-only with a hard cap, so the directive that says *"if you just record it in `MEMORY.md` it will be lost there"* was itself parked in `MEMORY.md`
answers:
  - "what are the owner's standing directives for this project"
  - "must every defect be owned by a task tree"
  - "should I ask the owner or decide autonomously"
  - "what does DECIDE DON'T ASK mean and when do I escalate anyway"
  - "when am I allowed to stop and ask the owner a question"
  - "is it enough to record a finding in MEMORY.md"
  - "where is the owner directive that a defect is only handled if a task-tree owns it"
  - "may an agent delete or migrate ~/Documents/github"
  - "which directories are excluded from every audit"
  - "where do the standing directives live now that MEMORY.md points at them"
  - "why is a directive not recorded in the resume pointer"
  - "how often should I push to the remote"
  - "what is the push cadence for this project"
  - "how many unpushed commits are acceptable"
  - "should I push after every commit or after a batch"
  - "is the push cadence mechanically enforced"
date: 2026-07-31
status: accepted
tags: [owner-directive, workflow, task-tree, memory-architecture, provenance, autonomy, audit, north-star]
reverify: "git grep -lE \"DECIDE, DON'T ASK|DEFECT IS ONLY HANDLED\" -- docs/decisions/ | wc -l  -> the records that cite these directives; before this record they cited MEMORY.md, an overwrite-only file with a hard line cap"
evidence: MEMORY.md (the pre-move text, verbatim below; measured 2,696 B across 9 lines at e4b4fd5); docs/decisions/0032-emit-surface-interaction-gate.md:440 and 0035:324 (two of the four records citing "`MEMORY.md` standing directives, `2026-07-30`"); docs/decisions/0033-shadow-enumeration-classification.md:405 and 0034:373 (the "Owner doctrine" citations); MEMORY_ARCHITECTURE.md §3 (layer A is overwritten each update with a hard size cap) and §6; docs/decisions/0031-ssd-volume-exclusivity.md (the four directives that were ALREADY recorded in layer C and are pointed at rather than restated); docs/decisions/0040-overflow-destination-classification-and-the-unmeasured-axis.md §4 (the measurement that 65 % of MEMORY.md is layer-C content). Counts re-derived 2026-07-31 at 85e4b7f.
---

# 0041 - The owner's standing directives, recorded in layer C

- Date: 2026-07-31
- Status: accepted
- Tree: `OVERFLOW-DESTINATION-INSTRUMENTATION.5a`
- Provenance: **owner directives, dated below.** Cited by **owner + date**, never by a harness
  bootstrap file — a rule that binds every author must not appear to rest on what one harness was
  told (`README-POLICY-PROVENANCE.1`).
- Relocation authorised by the owner on `2026-07-31` (*"OK then to relocate"*), in response to the
  measurement in [`0040`](0040-overflow-destination-classification-and-the-unmeasured-axis.md) §4.

## Context

[`0040`](0040-overflow-destination-classification-and-the-unmeasured-axis.md) §4 measured that
**65 % of `MEMORY.md` is layer-C content living in layer A**, invisible to that file's line cap
because it is written as 23 very long lines. `## Standing directives` is 2,696 bytes of it.

Measuring *why that matters* produced something sharper than a size problem.

**Thirteen citations across nine decision records point at these directives, and four of them
name their location literally as `(MEMORY.md standing directives, 2026-07-30)`.** `MEMORY.md` is
layer **A**: `MEMORY_ARCHITECTURE.md` §3 defines it as **overwritten** on each update with a hard
size cap, and §6 states the rule as *"Overwrite, don't append — it always describes now, never
the journey."* So nine decision records were citing, as durable provenance, a file the standard
guarantees will be overwritten.

The first directive below says exactly this, in the owner's own words — *"If you just record in
`MEMORY.md` it will be lost there."* **The directive warning against parking things in the resume
pointer was itself parked in the resume pointer**, and it is the most-cited one in the project.
That is not irony worth a sentence; it is the failure mode operating on its own statement, and it
is why this record exists rather than a size trim.

### What was already in layer C, and is therefore pointed at rather than restated

Four of the seven entries in that section were **already fully recorded** in
[`0031`](0031-ssd-volume-exclusivity.md) — not merely mentioned, but stated with the owner's
verbatim quotes:

| directive | recorded at |
| --- | --- |
| **Never rewrite history** — no `rebase` / `--amend` of landed commits / `reset --hard` / `filter-branch` / force-push; `CHANGES.md` + `DEVELOPMENT_NOTES.md` never retro-edited | `0031` (the owner quote *"Keep it raw, keep honest, so that people can follow the whole history"*, and the explicit prohibitions) |
| **The SSD is the only project volume** | `0031` — the whole record |
| **Shared means shared — use it, never duplicate** (`~/.cargo`, `~/.rustup`, `/opt/homebrew`; `CARGO_HOME`/`RUSTUP_HOME` at defaults) | `0031`, with the owner's verbatim clarification of `2026-07-29` |
| **Harness runtime files belong to the harness** — never depend on them; redirect real output to `.cache/scratch/` | `0031`, stated as an honest limit |

Their copies in `MEMORY.md` were therefore **shadows under [`0033`](0033-shadow-enumeration-classification.md)**
— derivable from `0031`, growth-coupled to it, and silent on divergence — and are repaired at
rung **R1** by a pointer, not reproduced here. Restating them would be the second copy this
project's own doctrine forbids.

The **three** below had no layer-C home at all. They are recorded now, **verbatim**.

## Decision

> **These are owner-set standing directives. Violating them is worse than not shipping.** They are
> recorded verbatim; an agent may relocate and point at them, and may **never** reword them.

### (a) A DEFECT IS ONLY HANDLED IF A TASK-TREE OWNS IT

**Owner-set, `2026-07-30`.** In the owner's words:

> *"If you just record in `MEMORY.md` it will be lost there."*

Correct, and mechanical: layer A is **overwrite-only with a hard cap**, so a finding parked there
is queued for deletion by the next state refresh. **Surfacing a finding is step one; opening the
tree is step two, in the same turn.** Never leave a defect in the resume pointer, a commit
message, or prose alone.

*Applies to itself:* a finding recorded in a decision record but not in a task tree is parked the
same way — nothing re-reads a landed record looking for outstanding work. `OVERFLOW-DESTINATION-INSTRUMENTATION.6`
was opened for exactly that reason.

### (b) DECIDE, DON'T ASK

**Owner-set, `2026-07-30`.** In the owner's words:

> *"You are an elite coder and you know the roadmap and the objectives of ANVIL, so take SOTA and
> signoff-level decisions whenever you can."*

Pick the next work unit, narrow or supersede a decision record, choose the mechanism —
autonomously, and **record the reasoning where it survives**. Escalate only when proceeding under
any assumption would be unsafe or would make the work useless if wrong.

**Surfacing a finding is still mandatory; turning it into a question is not.** The two halves are
independent: the directive removes the *question*, never the *disclosure*.

### (c) `~/Documents/github` is owner-owned

**Owner-set.** It holds the pre-move checkouts. **No agent deletes or migrates it; the owner
removes it themselves.** It is excluded from every audit.

This is the one directive of the three that constrains an *action* rather than a *judgement*, and
it is the one most likely to be violated by a well-meaning cleanup sweep — which is why it is
recorded beside the other two rather than left as a line in a file that gets overwritten.

### (d) The push cadence is every 200 commits

**Owner-set, `2026-08-01`, verbatim: *"The push cadence is every 200 commits."*** Stated in answer
to an agent that had surfaced 105 unpushed commits as a durability risk — so the recorded fact is
not merely the number but that **105 was explicitly judged acceptable**, and an agent below the
threshold should neither push nor re-raise it.

**Why this is layer C and not a line in `COMMIT.md`.** It is an *owner preference*, not a workflow
step, and it resolves a question the project's own documents leave open in both directions:
`MEMORY_ARCHITECTURE.md` §8 says *"push regularly"* without a number, and `COMMIT.md`'s nine
workflow steps **never mention pushing at all**. A cadence recorded only inside a workflow step
would also be re-derived every time a session asked *"is 105 a lot?"* — which is precisely the
archaeology layer C exists to eliminate. `COMMIT.md` gains a **pointer** here, not a copy
(`feedback_full_factorization`: one mechanism, never two).

**It is deliberately NOT mechanically gated, and the reason is stated rather than left implicit**
(`DOCTRINE_ENFORCEMENT.md` §9). A check is *technically* available and needs no network —
`git rev-list --count origin/main..HEAD` reads the last-known remote ref locally. It is not built
because a cadence is an owner **preference about risk appetite**, not an invariant of the
repository: the correct threshold is a judgement the owner has now made once and may revise, and a
gate that blocked a commit over it would convert a preference into a rule the owner never asked
for. Registering a doctrine is a deliberate act (`OVERFLOW-DESTINATION-INSTRUMENTATION.7` applied
the factorization test explicitly rather than waving it through); this does not clear that bar
today. **If the count is ever missed in practice, that is the evidence that changes the answer** —
and the check is one line when it is wanted.

## Decisive test applied

*"If `MEMORY.md` were overwritten tonight, would anything be lost?"*

Before this record: **yes** — the text, the owner's quotes, and the dates of three directives, and
with them the referent of thirteen citations in nine decision records. After it: **no**. The
resume pointer keeps a one-line pointer to this record and to `0031`; both are layer-C files that
are appended to and superseded, never overwritten.

## What this decision does NOT license

1. **It does not license rewording, softening, or "modernising" any directive above.** They are
   owner-set. An agent may add a pointer, a cross-link, or a dated Amendment recording a *new*
   owner statement — never an edit to what the owner said.
2. **It does not license removing the directives from the agent-facing path.** `MEMORY.md` keeps a
   pointer, and the bootstrap files keep routing through `README.md` + `MEMORY_ARCHITECTURE.md`.
   Relocation must not reduce discoverability; that is the whole reason it is a pointer and not a
   deletion.
3. **It does not license restating `0031`'s four directives here.** They are recorded there; a
   second copy is a `0033` shadow, and this record would be the one that rots.
4. **It does not license treating "DECIDE, DON'T ASK" as licence to skip disclosure.** §(b) — the
   directive removes the question, never the surfacing.
5. **It does not license auditing, cleaning, or migrating `~/Documents/github`** under any tree.

## Rejected alternatives

- **Leave the directives in `MEMORY.md` and drop the byte cap instead.** Rejected — and it was put
  to the owner, who chose relocation (`2026-07-31`). Independently, it fails on its own terms: the
  problem is not that the file is large, it is that **durable facts are stored in a file the
  standard guarantees will be overwritten**. A larger cap preserves the defect and grows it.
- **Restate all seven directives here for a single self-contained home.** Rejected — four are
  already recorded in `0031` with the owner's own quotes. A second copy is a `0033` shadow that
  can rot away from the original with nothing to catch it, which is the same reasoning
  [`0038`](0038-changes-md-position-repair-by-pointer.md) §(b) used to reject re-publication.
- **Write them as Knowledge Map fact cards instead of a decision record.** Rejected — a card is a
  *retrieval pointer to a canonical home*, and these needed the home itself. `docs/decisions/` is
  a Knowledge Map scan directory, so this record's `answers:` keys are indexed anyway and a card
  would add a hop and a copy (the same reasoning as
  [`0039`](0039-sweep-exemption-past-vs-present-and-recorded-recall.md) §(e)).
- **Fold them into `0031`** as an Amendment, since four of the seven live there. Rejected —
  `0031`'s subject is *volume and data locality*; "decide autonomously" and "a defect needs a task
  tree" are workflow directives with nothing to do with storage. Folding them in would bury three
  general rules inside a specific one, which is the same error
  [`0039`](0039-sweep-exemption-past-vs-present-and-recorded-recall.md) rejected for `0033`.
- **Also relocate `## Operating gotchas` in this leaf.** Rejected as scope — different content
  kind (operational lessons, not owner policy) and a different destination (Knowledge Map cards,
  which `KNOWLEDGE_MAP_ARCHITECTURE.md` §4 names explicitly for gotchas). It is `.5b`, and the
  byte cap is not reachable until both land.

## Consequences

- **Three owner directives move from a file that is overwritten to files that are appended to.**
  Thirteen citations across nine decision records now have a durable referent. No citation needed
  editing: they cite by **owner + date**, which is stable across the move — the provenance style
  `README-POLICY-PROVENANCE.1` mandated is what made the relocation cheap.
- **`MEMORY.md` loses 2,696 bytes of layer-C content and gains a two-line pointer block.** The
  derived 6,144-byte cap (`0040` §(c)) is *not* reached by this leaf alone; `.5b` carries the
  remainder, and the gap is reported rather than met by raising the cap (`0040` non-license 3).
- **Four directives are repaired at `0033` rung R1 by pointer** rather than being copied — the
  project's own anti-shadow rule applied to its own policy statements.
- The relocation is **lossless and proven so**: every token in the removed range was swept, not a
  hand-picked phrase list (`0036` §(iii)), and the residue reported in the owning task leaf.
- Docs-only leaf: no `src/` change ⇒ **DUT byte-identical**, `tests/snapshots.rs` untouched.

## Links

- Tree: [`docs/tasks/OVERFLOW-DESTINATION-INSTRUMENTATION.md`](../tasks/OVERFLOW-DESTINATION-INSTRUMENTATION.md)
  (`.2` = `0040` measured the 65 %; this leaf is `.5a`; `.5b` relocates the gotchas; `.3` then
  installs the byte cap).
- The four directives **already** recorded, pointed at rather than restated:
  [`0031`](0031-ssd-volume-exclusivity.md).
- Why a pointer and never a second copy: [`0033`](0033-shadow-enumeration-classification.md)
  (the three-question shadow rule and rung R1),
  [`0038`](0038-changes-md-position-repair-by-pointer.md) §(b) (re-publication rejected for the
  same reason).
- Why here rather than folded into an existing record:
  [`0039`](0039-sweep-exemption-past-vs-present-and-recorded-recall.md) §(e) and its rejected
  alternatives — a general rule is not buried inside a specific one.
- Provenance style: `README-POLICY-PROVENANCE.1` — cite an owner directive by **owner + date**,
  never by a harness bootstrap file.
- Standard: `MEMORY_ARCHITECTURE.md` §3 (layer A is overwritten; layer C is appended and
  superseded) and §6 (*"if it exceeds the cap, information is in the wrong layer"*).
