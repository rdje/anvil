---
id: live-document-size-containment-adoption
title: ANVIL adopts the **Live-Document Size-Containment** doctrine as its fifth portable architecture — because `2` of ANVIL's **279** tracked documents are capped, they are the two *smallest*, and `MEMORY-ARCH`'s routing hint sends overflow into three **unbounded** neighbours
answers:
  - "why did ANVIL adopt LIVE_DOCUMENT_SIZE_CONTAINMENT.md"
  - "what is the difference between MEMORY_ARCHITECTURE.md and LIVE_DOCUMENT_SIZE_CONTAINMENT.md"
  - "does decision 0040 already cover document size containment"
  - "is it enough to cap README.md and MEMORY.md"
  - "where may an over-cap file route its overflow"
  - "what lifecycle class does a live document get"
  - "CHANGES.md is 2.6 MB and growing, is that a problem"
  - "why can docs/TASK_TREE.md not be opened by an agent read tool"
  - "may durable history grow without bound"
date: 2026-08-08
status: accepted
tags: [doctrine, live-docs, size-containment, memory-architecture, routing, lifecycle, adoption, north-star]
evidence: LIVE_DOCUMENT_SIZE_CONTAINMENT.md (the adopted standard, neutral body byte-identical to its template — SHA-256 verified at adoption); docs/tasks/LIVE-DOCUMENT-SIZE-CONTAINMENT-ADOPTION.md (the owning tree and its measured registration census); docs/decisions/0040-overflow-destination-classification-and-the-unmeasured-axis.md (the predecessor this subsumes and extends); scripts/check_memory_architecture.sh (the routing hint whose three destinations are all unbounded)
reverify: "bash -c \"n=\\$(git ls-files '*.md' | wc -l | tr -d ' '); b=\\$(git ls-files '*.md' | xargs cat | wc -c | tr -d ' '); echo \\\"tracked *.md: \\$n files, \\$b bytes\\\"; echo 'mechanically capped today: README.md + MEMORY.md = 2'; wc -lc CHANGES.md DEVELOPMENT_NOTES.md docs/TASK_TREE.md; awk '{if(length(\\$0)>m)m=length(\\$0)}END{print \\\"docs/TASK_TREE.md max line bytes: \\\" m}' docs/TASK_TREE.md\""
---

# 0052 - ANVIL adopts the Live-Document Size-Containment doctrine

- Date: 2026-08-08
- Status: accepted
- Tree: `LIVE-DOCUMENT-SIZE-CONTAINMENT-ADOPTION.0` (registration)
- Activated by: an owner directive — *"create a dedicated task-tree and decision to adopt
  `LIVE_DOCUMENT_SIZE_CONTAINMENT.md` … Do not solve any pressure by allowing a mandatory live file
  to grow indefinitely. Durable history may be logically unbounded, but it must leave the bounded
  working set through partitioning, sealing, rotation, or archival retrieval."*

## Context — what ANVIL already had, and the exact shape of the hole

ANVIL runs four portable architectures (`DOCTRINE_ENFORCEMENT.md`'s table). One of them,
`MEMORY_ARCHITECTURE.md`, is often mistaken for covering this ground. It does not, and the boundary
is clean:

> **`MEMORY_ARCHITECTURE.md` governs what memory *means* and where information *belongs*.
> `LIVE_DOCUMENT_SIZE_CONTAINMENT.md` governs how every long-lived document *remains usable as the
> project grows*.**

The nearest prior art is decision
[`0040`](0040-overflow-destination-classification-and-the-unmeasured-axis.md), which is genuinely
good work and genuinely insufficient. It distinguishes **bounded** from **unbounded** documents and
classifies overflow destinations. It has no lifecycle classes, no rolling ledger, no archive
terminal, no transition-debt control, no transitive route following, and no complete mechanical
inventory. It was also derived from **one** file's pressure, and a rule derived from one file is a
rule that has never been asked whether it scales.

### The measurement that settles it

Taken at `47ac5ce`, `2026-08-08`, over the tracked tree:

| Fact | Value |
| --- | ---: |
| tracked `*.md` files | **279** |
| their total size | **11,035,621 B ≈ 2.6×** a 4 MiB context window |
| of those, **mechanically capped** | **2** — `README.md` and `MEMORY.md` |
| `MEMORY.md` byte cap | 6,144 |
| `CHANGES.md` | 49,314 lines / **2,669,993 B** — **435×** that cap |
| `DEVELOPMENT_NOTES.md` | 18,124 lines / **1,138,689 B** |
| `docs/tasks/*.md` | 85 files / **3,285,728 B** |
| `docs/TASK_TREE.md` **longest single line** | **39,591 B** |
| `CODEBASE_ANALYSIS.md` longest single line | **24,991 B** |

**The two capped files are the two smallest live surfaces in the repository.** ANVIL bounded its
landing page and its resume pointer — the cheapest two — and left 11 MB unbounded, which is the
failure mode of a size rule derived from whichever file happened to hurt first.

### The finding that makes this a defect rather than a gap

The neutral doctrine states: *"Routing is transitive. A bounded file that sends overflow to an
unbounded neighbor has not contained anything."*

`MEMORY-ARCH`'s cap-failure routing hint — the text an author reads at the exact moment they must move
content — names **three** destinations:

```
  a durable cross-cutting fact  -> docs/decisions/  (layer C, append-only)
  an operating gotcha           -> docs/knowledge/  fact card
  per-unit work state           -> docs/tasks/      (layer B, append-only)
  history of what changed       -> git log + CHANGES.md
```

**Every one of them is unbounded**, and `0040` did not overlook this — it *argued* for it, recording
that the hint is safe *"because layer B and layer C are append-only records that are supposed to grow
and are never capped."* That reasoning treats unboundedness as a **justification** where this doctrine
treats it as a **deferral**. Under the core invariant, ANVIL's most-exercised containment mechanism
has been moving bytes from a capped file into uncapped ones and recording the move as compliance.

That is not a hypothetical. `docs/TASK_TREE.md` **could not be opened** by this session's file-read
tool — it exceeds the 256 KB limit — and its worst single line is **39,591 B**, on the
maximum-line-width axis `0040` itself identified as *"the axis nobody measures"* and then measured on
only one file. A mandatory bootstrap read that an ordinary reader cannot open has already failed,
whichever number one chooses to blame.

## Decision

**ANVIL adopts the Live-Document Size-Containment doctrine as its fifth portable architecture**, as
the repository-root, project-owned `LIVE_DOCUMENT_SIZE_CONTAINMENT.md`.

1. **The neutral body is copied verbatim** — SHA-256 verified byte-identical at adoption
   (`6c4e8a51dcd735dd…`, 16,673 B) — and the donor's fenced local-adoption note is **replaced
   entirely** by an ANVIL note. The origin is a **template, not an upstream**: there is no sync, and
   later revisions elsewhere bind ANVIL only through deliberate local review.
2. **No number, class, registry entry, migration outcome, or task identifier is copied from another
   adoption.** The neutral body forbids exactly that, and ANVIL derives its own from its own measured
   survivors. A donor-residue scan over the local note is part of the registration evidence.
3. **Nothing is classified by this decision.** Registration states the standard ANVIL is held to; it
   is *not* a claim about ANVIL's compliance. A lifecycle asserted before its measurement would be
   `DOCTRINE_ENFORCEMENT.md` §6.1's self-tick, and this doctrine's own checklist orders inventory →
   classification → measurement → targets for that reason.
4. **`0040` is subsumed and extended, never revoked.** Its two-axis insight (lines *and* bytes) and
   its classified-destination rule are this doctrine's requirements, arrived at early and on one
   file. Its non-licenses stand in full — in particular, **raising a cap still requires a new
   decision record stating the surface's contract expanded.** What changes is that "the destination
   is append-only" stops being an answer.
5. **Durable history stays logically unbounded, but leaves the bounded working set** by partitioning,
   sealing, rotation, or archival retrieval. No mandatory live file is permitted to grow indefinitely,
   and no pressure may be relieved by routing into a neighbour that has not itself been contained.
6. **Enforcement is git-level and unconditional**, like every other ANVIL doctrine: one registered
   check run by `.githooks/pre-commit` (E3) and CI (E4), reading the resulting tree regardless of
   which paths a commit touched — the choice `README-GROWTH` and `BOOK-LINK-TARGETS` already make,
   because a breach can arrive by revert or by merge.

## Scope of the owning tree

`LIVE-DOCUMENT-SIZE-CONTAINMENT-ADOPTION` owns the seven checklist steps and then the per-surface
migrations. Four surfaces are named by the owner as needing particular attention, and each is
recorded as a **question to be answered by measurement**, not as a conclusion:

- `MEMORY.md` → `bounded_snapshot`. *Already done* at
  [`0051`](0051-the-resume-pointer-is-updated-when-resumable-state-changes.md), which classified it,
  kept both caps, and removed its derivable content — so it enters this program as the one surface
  with its migration already landed.
- `CHANGES.md` → audit for `rolling_ledger`, archival replacement, or **retirement in favour of git
  and task history**. Retirement is genuinely on the table: 632 of 709 entries already carry a landed
  hash, which is what makes the git-history claim testable rather than rhetorical.
- `DEVELOPMENT_NOTES.md` → audit for partitioning, decision-record routing, or `frozen_legacy`.
- `docs/TASK_TREE.md` + `docs/tasks/*.md` → a bounded current index plus sealed or partitioned
  history. The index is a **mixed surface** by [`0042`](0042-task-tree-index-is-a-mixed-surface.md)
  and re-states 416 of 508 leaves, so duplicate-proof-before-deletion applies directly.
- the mdBook + `USER_GUIDE.md` → likely `maintained_reference`: product-sized prose where a fixed
  aggregate cap would be dishonest, requiring bounded parts and complete navigation instead.

## Honest limits (§9 — stated at adoption, not discovered later)

- **Registration changes no file's size and fixes no pressure.** Everything above is a plan; the
  measured state is unchanged until the leaves land. Saying otherwise would be the confidence
  manufacture §6.1 warns about.
- **This is the fifth doctrine standard in a repository that already carries four**, and the honest
  risk is meta-work crowding out product work. The mitigation is ordering, not enthusiasm: the tree
  runs inventory → measure → target → gate, and the gate lands *once*.
- **Adopting a size doctrine does not make ANVIL's documents shorter.** The transition-debt mechanism
  exists precisely because the first census will find surfaces far past any healthy target, and
  entering them as pinned debt with a named owner is the doctrine's answer to that — not a licence
  to keep appending.
- **The checker cannot prove currency**, only declared structural properties. Whether ANVIL opts into
  a currency contract for any surface is a later, separate decision.
- **`README.md` grows by one navigation row** to make this file discoverable. That is the one growth
  this commit causes, it is what `README_POLICY.md` explicitly permits (canonical navigation
  changed), and the file stays inside both caps.

## Consequences

- `LIVE_DOCUMENT_SIZE_CONTAINMENT.md` is ANVIL's fifth portable architecture, owner-authored copy,
  cited by owner + date.
- `LIVE-DOCUMENT-SIZE-CONTAINMENT-ADOPTION` is the owning tree; no other tree may claim its leaves.
- The rule that outlives this record: **a bounded file that routes its overflow into an unbounded one
  has not contained anything — it has renamed the problem and filed it under a different name.**
