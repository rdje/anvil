# LIVE-DOCUMENT-SIZE-CONTAINMENT-ADOPTION: only 2 tracked documents are capped, they are the two smallest, and every one of the 10 destinations the size gates route overflow into is uncapped

## Metadata

- Tree ID: `LIVE-DOCUMENT-SIZE-CONTAINMENT-ADOPTION`
- Status: `active`
- Roadmap lane: Workflow / doctrine adoption — live-document containment
- Created: `2026-08-08`
- Last updated: `2026-08-08` (`.1` **done** — inventory derived; frontier `.2`)
- Owner: repo-local workflow — **opened on an explicit owner directive**, not on an agent's noticing

## Goal

Adopt and mechanically enforce the **Live-Document Size-Containment** doctrine
([`LIVE_DOCUMENT_SIZE_CONTAINMENT.md`](../../LIVE_DOCUMENT_SIZE_CONTAINMENT.md), decision
[`0052`](../decisions/0052-live-document-size-containment-adoption.md)), so that every long-lived
ANVIL document stays usable as a bounded working set while its durable history stays recoverable.

**The registration census** (at `47ac5ce`, `2026-08-08` — the *motivating* measurement; the governed
census is `.3`'s):

| Surface | Lines | Bytes | Longest line |
| --- | ---: | ---: | ---: |
| all tracked `*.md` (**279** files) | — | **11,035,621** | — |
| `CHANGES.md` | 49,314 | **2,669,993** | 1,307 |
| `docs/tasks/*.md` (85 files) | 25,801 | **3,285,728** | 33,890 |
| `DEVELOPMENT_NOTES.md` | 18,124 | **1,138,689** | 845 |
| `docs/decisions/*.md` (52 files) | 12,977 | 877,422 | 5,139 |
| `book/src/*.md` (30 files) | 15,174 | 752,754 | 2,007 |
| `KNOWLEDGE_MAP.md` (derived) | 2,422 | 729,941 | 2,348 |
| `docs/knowledge/*.md` (81 files) | 5,218 | 359,982 | 1,488 |
| `docs/TASK_TREE.md` | 386 | 304,714 | **39,591** |
| `CODEBASE_ANALYSIS.md` | 2,874 | 292,661 | **24,991** |
| `ROADMAP.md` | 2,904 | 185,517 | 3,653 |
| `USER_GUIDE.md` | 2,631 | 163,444 | 512 |
| **`README.md`** (capped 250 / 12,288) | 162 | 10,676 | 378 |
| **`MEMORY.md`** (capped 50 / 6,144) | 30 | 5,175 | 994 |

**Two of 279 are capped, and they are the two smallest.** ANVIL bounded its landing page and its
resume pointer and left ~11 MB unbounded — the signature of a size rule derived from whichever file
hurt first.

**The defect, not merely the gap.** `MEMORY-ARCH`'s cap-failure routing hint names four destinations
— `docs/decisions/`, `docs/knowledge/`, `docs/tasks/`, `CHANGES.md` — and **every one is unbounded**.
Decision [`0040`](../decisions/0040-overflow-destination-classification-and-the-unmeasured-axis.md)
did not overlook that; it *argued* for it, on the grounds that append-only records *"are supposed to
grow and are never capped."* The doctrine's core invariant answers directly: *"a bounded file that
sends overflow to an unbounded neighbor has not contained anything."* ANVIL's most-exercised
containment mechanism moves bytes from a capped file into uncapped ones and records it as compliance.

> **`.1` enlarged this figure and the correction is the interesting part.** Deriving both route kinds
> from the *whole* enforcement surface shows `README-GROWTH` emits an overflow hint as well — never
> examined, because `.0` read the one enforcer that was under discussion. The measured figure is
> **10 distinct live-document overflow destinations, 0 capped**, totalling **8,683,015 B** against
> the **16,115 B** the two caps actually bound: a **539×** ratio. *A hand-read sample of a mechanism
> reports the mechanism you were already looking at.*

**And a read path has already failed.** `docs/TASK_TREE.md` — a Tier-1 bootstrap read — **could not be
opened** by this session's file-read tool (296.3 KB against a 256 KB limit), and its worst single line
is **39,591 B**. That is the maximum-line-width axis `0040` named as *"the axis nobody measures"* and
then measured on exactly one file.

## Non-Goals

- **Not a rewrite of ANVIL's history.** [`0031`](../decisions/0031-ssd-volume-exclusivity.md) forbids
  it. Sealing, partitioning and archiving preserve record order and identity; they never edit landed
  entries. [`0038`](../decisions/0038-landed-changes-entries-are-immutable.md) stands.
- **Not a revocation of [`0040`](../decisions/0040-overflow-destination-classification-and-the-unmeasured-axis.md).**
  It is subsumed and extended. Its non-licenses — above all *raising a cap needs a new decision
  record* — remain in force for every cap this tree sets.
- **Not a licence to delete anything to hit a number.** The doctrine requires duplication proved
  before deletion and archive retrieval proved before a live copy is removed. Nothing is dropped on
  size grounds alone.
- **Not an import of another project's numbers.** No threshold, lifecycle, registry entry, migration
  outcome or task id is copied from any other adoption; the neutral body forbids it and ANVIL derives
  its own. The originating copy is a **template, not an upstream**.
- **Not a claim of compliance.** `.0` registers a standard; it classifies nothing and fixes nothing.

## Acceptance Criteria

- Every tracked live document, generated view, collection, route and historical terminal is
  inventoried, with routes followed **transitively** — including the route edges implied by
  enforcers' emitted failure hints, which the doctrine counts as real pressure edges.
- Every surface is classified by lifecycle and its actual canonical source named.
- Every surface is measured on **five** axes: lines, bytes, maximum content-line bytes, collection
  file-count/aggregate, and read path.
- Health targets are **derived from ANVIL's own reviewed survivors**, with separate inclusive
  enforcement ceilings and two ordered milestones; no number is copied from elsewhere and no current
  legacy size is called healthy merely because it was measured.
- Every surface already past warning is entered as **transition debt** with an exact pinned baseline
  and a **named** remediation owner; the baseline never widens.
- One ANVIL-local data registry plus one deterministic, **unconditional** checker, registered in
  `scripts/check_doctrines.sh`, `.githooks/pre-commit`, CI, the bootstrap documentation and
  `DOCTRINE_ENFORCEMENT.md` §10.
- The checker obeys `DOCTRINE_ENFORCEMENT.md` §4's contract, is **count-floored** so it cannot pass
  vacuously, and is **negative-controlled** — including the §9 acceptance test: *delete the subject
  and re-run it.*
- No mandatory live file is left growing indefinitely, and no pressure is relieved by routing into an
  uncontained neighbour.

## Task Tree

- ID: `LIVE-DOCUMENT-SIZE-CONTAINMENT-ADOPTION`
  Status: `active`
  Goal: `Every long-lived ANVIL document stays a bounded working set; durable history stays recoverable and leaves that working set by a declared operation.`
  Children: `.0`–`.7` (adopt + build the control plane), `.8`–`.12` (per-surface migrations)

- ID: `LIVE-DOCUMENT-SIZE-CONTAINMENT-ADOPTION.0`
  Status: `done`
  Goal: `Register the doctrine: the ANVIL-owned copy, the decision, this tree, and discoverability.`
  Acceptance: `LIVE_DOCUMENT_SIZE_CONTAINMENT.md exists at the repo root with the neutral body copied VERBATIM (proved, not asserted) and an ANVIL-specific local-adoption note replacing the donor's; a donor-residue scan over that note is clean; decision 0052 records why ANVIL needs a doctrine 0040 does not supply; README.md gains one navigation row and stays inside both caps.`
  Verification: `Neutral body proved byte-identical by SHA-256 over the region after the local-adoption END fence: donor 16,673 B / 6c4e8a51dcd735dd == ANVIL 16,673 B / 6c4e8a51dcd735dd. Local note proved fully replaced and scanned clean over 12 donor tokens (project names, decision numbers, task ids, registry filenames, migration counts). Registration census measured at 47ac5ce over 279 tracked *.md = 11,035,621 B, of which 2 files are capped. Routing-hint defect confirmed by reading scripts/check_memory_architecture.sh: all four hint destinations are uncapped — ENLARGED at .1 to 10 of 10, by deriving the same fact from the whole enforcement surface instead of the one enforcer under discussion. Read-path failure OBSERVED, not predicted: docs/TASK_TREE.md (296.3 KB) was refused by this session's file-read tool at its 256 KB limit during the Tier-1 bootstrap read. bash scripts/check_doctrines.sh green on all 11.`
  Commit: `d0426bd`

- ID: `LIVE-DOCUMENT-SIZE-CONTAINMENT-ADOPTION.1`
  Status: `done`
  Goal: `Inventory every live document, generated view, collection, route and historical terminal — completely, and mechanically.`
  Acceptance: `A complete enumeration derived from the tree (git ls-files), never hand-listed — a hand-listed inventory is precisely the 0033 shadow this project repairs by deletion. Routes are followed TRANSITIVELY and BOTH route kinds are separated per the doctrine: reader navigation vs author overflow, the latter derived from enforcers' emitted failure hints as well as hand-authored links, since an undeclared path-shaped hint is a real pressure edge. Output: one machine-readable inventory + the count of surfaces and route edges, with the residual (files present, files inventoried) proven zero.`
  Verification: `Delivered as scripts/live_doc_inventory.py — a DERIVATION, not a list: git ls-files is the authority and every later leaf reads the script rather than a copy of it. Census at d0426bd: 282 tracked *.md = 11,106,985 B across 32 surfaces (27 singletons + 5 collections), RESIDUAL 0 — the identity every *.md is either a singleton or a member of exactly one collection holds, so nothing is silently omitted. Routes: 843 reader-navigation edges, 22 author-overflow edges, the two kinds separated as the doctrine requires. Determinism proved: two --json runs byte-identical by SHA-256 (b69f2e7913effb12...). Extractors carry the lessons other ANVIL extractors paid for — whole-file not line-wise (a link text may wrap across a newline: BOOK-LINK-INTEGRITY.3), fence-masked (a link inside a fence is an example, not a route), and count-floored by the residual identity. THE FINDING, AND IT IS LARGER THAN .0 RECORDED: .0 measured MEMORY-ARCH's routing hint at 4-of-4 uncapped. The derivation finds README-GROWTH ALSO emits an overflow hint, which nobody had examined, so the true figure is 10 distinct live-document overflow destinations — CHANGES.md, ROADMAP.md, TOOLBOX.md, USER_GUIDE.md, book/src/, docs/TASK_TREE.md, docs/decisions/, docs/evidence/, docs/knowledge/, docs/tasks/ — of which 0 ARE CAPPED. Mechanically confirmed: grep over scripts/*.sh finds exactly two cap constants in the repository (README-GROWTH 250/12,288 and MEMORY-ARCH 50/6,144), and neither governs any of the ten. ANVIL therefore bounds 16,115 B of surface and routes its overflow into 8,683,015 B of unbounded surface — a 539x ratio. Extractor deliberately OVER-collects (7 further candidates: ENUMERATION-PARITY sync targets, NO-BOOT-VOLUME-REFS forbidden paths, 2 prose artifacts), because the doctrine makes an undeclared path-shaped hint fail closed; separating a genuine pressure edge from a sync target is a DECLARATION and belongs in .6's registry, not a hand-curated exclusion list here. One filter only, a parse fix not a judgement: a single-character path segment, since the hint text "layer B" yields a spurious "B/". bash scripts/check_doctrines.sh green on all 11.`
  Commit: `this commit`

- ID: `LIVE-DOCUMENT-SIZE-CONTAINMENT-ADOPTION.2`
  Status: `pending`
  Goal: `Classify each inventoried surface by lifecycle and name its actual canonical source.`
  Acceptance: `Every surface carries exactly one of the doctrine's eight classes (or a locally defined one that makes growth stop, partition under fixed bounds, or be governed by exact fresh authority). Each classification cites the information ROLE that produced it, per "choose the storage topology from the information role" — not a line count. MEMORY.md enters already classified bounded_snapshot by 0051. Renaming an append-only blob is not a new lifecycle, and classification alone never waives existing debt.`
  Verification: `pending`
  Commit: `pending`

- ID: `LIVE-DOCUMENT-SIZE-CONTAINMENT-ADOPTION.3`
  Status: `pending`
  Goal: `Measure every surface on all five axes, deterministically and reproducibly.`
  Acceptance: `Lines, bytes, maximum content-line bytes (raw content bytes, excluding LF and an optional preceding CR), collection file-count/aggregate, and read path. The maximum-line axis is measured on EVERY surface, not one: 0040 named it and measured it once, and docs/TASK_TREE.md's 39,591 B line plus CODEBASE_ANALYSIS.md's 24,991 B line are what that omission cost. Read path includes whether an ordinary reader/tool can open the file at all — a Tier-1 read that exceeds a 256 KB tool limit has already failed.`
  Verification: `pending`
  Commit: `pending`

- ID: `LIVE-DOCUMENT-SIZE-CONTAINMENT-ADOPTION.4`
  Status: `pending`
  Goal: `Derive ANVIL's own health targets and separate inclusive enforcement ceilings, with two ordered milestones.`
  Acceptance: `Targets derived from ANVIL's reviewed survivors, exactly as 0036 §(c) and 0040 §(c) derived README.md's and MEMORY.md's rather than fitting them to current size. Health target and inclusive ceiling are DIFFERENT declared values; the ceiling rejects only actual > ceiling and is a quarantine boundary, never evidence of health. Warning must leave capacity for the largest normal update plus the rollover transaction. A ceiling increase needs a separate reviewed authority record — a surface declaration cannot authorize itself. Lowering is free. Product-sized maintained reference gets per-part limits and exact per-change aggregate authority instead of a dishonest fixed aggregate cap.`
  Verification: `pending`
  Commit: `pending`

- ID: `LIVE-DOCUMENT-SIZE-CONTAINMENT-ADOPTION.5`
  Status: `pending`
  Goal: `Record transition debt for every surface already past warning, each with a named owner.`
  Acceptance: `Exact measured baseline, named remediation owner, deadline or ordered frontier, unchanged ceiling. The baseline is monotonic non-increasing across revisions; an atomic content reduction may lower it so the ceiling can ratchet down. Only records needed to complete the containment transition may extend a rollover-debt surface. The debt exception ends when the migration lands and can never excuse ceiling overflow.`
  Verification: `pending`
  Commit: `pending`

- ID: `LIVE-DOCUMENT-SIZE-CONTAINMENT-ADOPTION.6`
  Status: `pending`
  Goal: `Build the ANVIL-local data registry and its single deterministic checker.`
  Acceptance: `Data-only registry consumed by one checker. The registry is itself finite: schema-versioned metadata declaring positive maximum data-record count, total file bytes and raw bytes per record; finite array cardinalities; scalar byte limits; closed identifier domains; unknown fields fail closed. The checker obeys DOCTRINE_ENFORCEMENT.md §4 (exit code is the verdict, explains on breach, deterministic, mutates nothing, path-agnostic), is COUNT-FLOORED so it cannot pass vacuously, and fails on the doctrine's minimum list — undeclared surface or destination, forbidden persisted path, missing owner/lifecycle/limit, route cycle or route to an uncontrolled neighbour, stale projection, mutated sealed unit, warning without owned remediation, actual above ceiling, unauthorized ceiling increase or rewritten baseline.`
  Verification: `pending`
  Commit: `pending`

- ID: `LIVE-DOCUMENT-SIZE-CONTAINMENT-ADOPTION.7`
  Status: `pending`
  Goal: `Register the check in every enforcement layer, and prove it can fire.`
  Acceptance: `One line in scripts/check_doctrines.sh's DOCTRINES array, a §10 row, the README doctrine-id enumeration (itself gated by ENUMERATION-PARITY), .githooks/pre-commit and CI reached via the driver, and the bootstrap documentation. Runs UNCONDITIONALLY like README-GROWTH and BOOK-LINK-TARGETS — a breach can arrive by revert or merge. Negative-controlled via scripts/negative_control.sh, and subjected to §9's acceptance test for coverage-shaped checks: DELETE THE SUBJECT AND RE-RUN IT; a check that still passes with the thing it checks removed is checking nothing.`
  Verification: `pending`
  Commit: `pending`

- ID: `LIVE-DOCUMENT-SIZE-CONTAINMENT-ADOPTION.8`
  Status: `pending`
  Goal: `CHANGES.md — audit for rolling_ledger, archival replacement, or retirement in favour of git and task history.`
  Acceptance: `Decide by measurement, not preference. Retirement is genuinely open: 632 of 709 entries carry a Landed as: hash and 459 name a leaf id, so "git log already holds this" is TESTABLE — test it rather than assert it, and measure what is lost for the entries that carry neither. If it survives as a rolling ledger, declare the seal boundary, immutable segments, bounded index and the archive transition BEFORE aggregate growth becomes unbounded. Landed entries are immutable (0038) and history is never rewritten (0031), so any partition preserves record order and identity byte for byte.`
  Verification: `pending`
  Commit: `pending`

- ID: `LIVE-DOCUMENT-SIZE-CONTAINMENT-ADOPTION.9`
  Status: `pending`
  Goal: `DEVELOPMENT_NOTES.md — audit for partitioning, decision-record routing, or frozen_legacy.`
  Acceptance: `Measure the overlap with docs/decisions/ first: this file and layer C hold the same KIND of content, and the doctrine requires a duplicate PROVED before removal, never assumed. Then choose partitioning, routing new rationale to layer C, or freezing the existing record with a write prohibition. A frozen surface cannot accept new content or act as an overflow destination.`
  Verification: `pending`
  Commit: `pending`

- ID: `LIVE-DOCUMENT-SIZE-CONTAINMENT-ADOPTION.10`
  Status: `pending`
  Goal: `docs/TASK_TREE.md and docs/tasks/*.md — a bounded current index plus sealed or partitioned history.`
  Acceptance: `The index is a MIXED surface by 0042, which measured 416 of 508 leaves (81.9%) re-stated from the tree files it indexes — so duplicate-proof-before-deletion applies directly and the row contract (one frontier, not a per-leaf journal) is the bound to restore. Its 39,591 B maximum line and its 296 KB total both breach the read path today. Task files are layer-B history and are not retro-edited (docs/TASK_TREE.md's own note), so closed trees seal rather than shrink.`
  Verification: `pending`
  Commit: `pending`

- ID: `LIVE-DOCUMENT-SIZE-CONTAINMENT-ADOPTION.11`
  Status: `pending`
  Goal: `The mdBook and USER_GUIDE.md — classify, most likely maintained_reference.`
  Acceptance: `Product-sized prose where a fixed aggregate cap is dishonest because legitimate scope changes with the product. Requires auditable audience/role/rationale, stable semantic parts, a bounded COMPLETE mandatory index with its own limits, bounded navigation depth, per-part limits, and exact fresh authority for every aggregate change. The book already has SUMMARY.md gated against the chapter set by ENUMERATION-PARITY and its links gated by BOOK-LINK-TARGETS — reuse those rather than adding a second mechanism. Classification never waives existing debt: semantic partition and complete navigation land first.`
  Verification: `pending`
  Commit: `pending`

- ID: `LIVE-DOCUMENT-SIZE-CONTAINMENT-ADOPTION.12`
  Status: `pending`
  Goal: `Re-audit: close the routing defect and prove no bounded surface routes into an uncontained neighbour.`
  Acceptance: `MEMORY-ARCH's routing hint (and every other enforcer hint that names a destination) points only at surfaces this program has contained. Lower limits to the retained steady-state surface rather than preserving legacy headroom. Re-run the full census and diff it against .3's baseline.`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `LIVE-DOCUMENT-SIZE-CONTAINMENT-ADOPTION.2` | `pending` | **Next.** Classify only what `.1` proved exists — **32 surfaces**, now enumerable by derivation rather than by hand. |
| 2 | `LIVE-DOCUMENT-SIZE-CONTAINMENT-ADOPTION.3` | `pending` | Measure all five axes; targets without measurement are imported numbers by another name. `.1`'s script already emits four of them, so this leaf is the **read path** plus the governed census. |
| 3 | `LIVE-DOCUMENT-SIZE-CONTAINMENT-ADOPTION.4` | `pending` | Derive targets and ceilings from ANVIL's own survivors. |
| 4 | `LIVE-DOCUMENT-SIZE-CONTAINMENT-ADOPTION.5` | `pending` | Pin the debt before any migration moves a byte. |
| 5 | `LIVE-DOCUMENT-SIZE-CONTAINMENT-ADOPTION.6` | `pending` | Registry + checker, once the data they encode is real. |
| 6 | `LIVE-DOCUMENT-SIZE-CONTAINMENT-ADOPTION.7` | `pending` | Register in every layer; prove it fires. |
| 7+ | `.8` – `.12` | `pending` | Per-surface migrations, each atomic per the transition protocol. |
| — | `LIVE-DOCUMENT-SIZE-CONTAINMENT-ADOPTION.1` | `done` | Inventory derived, not listed. **32 surfaces, residual 0, 843 nav + 22 overflow edges** — and the overflow finding grew from `.0`'s 4-of-4 to **10 of 10 uncapped**. |
| — | `LIVE-DOCUMENT-SIZE-CONTAINMENT-ADOPTION.0` | `done` | Registered `2026-08-08` on an owner directive. |

## Decisions

- `2026-08-08` (registration): **The neutral body is copied verbatim and proved so, rather than
  paraphrased.** A doctrine restated in local words is a fork that will drift from its own template
  silently — the failure `0033` classifies. The SHA-256 equality is recorded as evidence so a later
  session can re-check the copy in one command.
- `2026-08-08` (registration): **Nothing is classified at registration.** The owner named five
  surfaces and their likely classes; those are recorded as **questions for `.2`**, with the likely
  answer attached, and not as decisions. `MEMORY.md` is the sole exception because
  [`0051`](../decisions/0051-the-resume-pointer-is-updated-when-resumable-state-changes.md) already
  classified it *after* measuring it.
- `2026-08-08` (registration): **`0040` is subsumed, not revoked**, and its non-licenses bind every
  cap this tree sets. Registering a superior standard is not a licence to re-open the settled numbers
  underneath it.

## Open Questions

- Does `CHANGES.md` survive at all, or is it retired into git + task history? `.8` decides by testing
  the claim rather than asserting it.
- Is `DEVELOPMENT_NOTES.md` distinguishable from `docs/decisions/` by content, or only by age? `.9`
  measures the overlap before choosing.
- Serialization format for the registry — the doctrine requires data-only and self-bounded, not a
  specific encoding. ANVIL already ships JSON-shaped artifacts and shell checks; `.6` picks the form
  that its checker can validate fail-closed without adding a dependency.
- Do `docs/knowledge/*.md` (81 files, 359,982 B) form a `partitioned_canonical` collection with
  `KNOWLEDGE_MAP.md` as its `generated_projection`? The shape strongly suggests it; `.2` decides.

## Blockers

- None.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-08-08` | `.1` | `scripts/live_doc_inventory.py at d0426bd: 282 tracked *.md = 11,106,985 B, 32 surfaces (27 singleton + 5 collection), RESIDUAL 0 by the every-file-is-exactly-one-surface identity. 843 reader-navigation + 22 author-overflow edges, the two kinds separated per the doctrine. Determinism: two --json runs byte-identical, SHA-256 b69f2e7913effb12... Overflow destinations of the two size gates: 10 distinct live documents, 0 capped, 8,683,015 B; the two capped surfaces total 16,115 B — a 539x ratio. Two cap constants exist in the whole repository (grep over scripts/*.sh), and neither governs any destination either gate names. bash scripts/check_doctrines.sh green on all 11.` | `.1 done` (workflow tooling + docs; no `src/`, DUT byte-identical) |
| `2026-08-08` | `.0` | `Neutral body SHA-256 equality proved over the post-fence region: donor 16,673 B 6c4e8a51dcd735dd == ANVIL 16,673 B 6c4e8a51dcd735dd (VERBATIM). Local-adoption note proved replaced and donor-residue-scanned clean across 12 tokens. Registration census at 47ac5ce: 279 tracked *.md, 11,035,621 B, 2 capped (README.md, MEMORY.md) — the two smallest. Routing-hint defect read directly from scripts/check_memory_architecture.sh: 4 of 4 hint destinations uncapped. Read-path failure OBSERVED this session: docs/TASK_TREE.md at 296.3 KB refused by a 256 KB file-read limit, worst line 39,591 B. README.md after its one navigation row: within both caps. bash scripts/check_doctrines.sh green on all 11.` | `.0 registered` (docs-only; DUT byte-identical) |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `.1` | `this commit` — `LIVE-DOCUMENT-SIZE-CONTAINMENT-ADOPTION.1 — derive the live-document inventory` | Adds `scripts/live_doc_inventory.py`. `scripts/` is not code by the `docs/TASK_TREE.md` boundary. |
| `.0` (registration) | `d0426bd` — `LIVE-DOCUMENT-SIZE-CONTAINMENT-ADOPTION.0 — adopt the containment doctrine` | Docs-only. `.0` is this repo's registration-commit convention, required because `.githooks/commit-msg` rejects a subject naming no leaf. |

## Changelog

- `2026-08-08` (`.1`): Inventory **derived** rather than listed, and the derivation immediately
  enlarged the registration finding. `.0` measured `MEMORY-ARCH`'s routing hint at **4 of 4**
  destinations uncapped; following *both* route kinds mechanically shows `README-GROWTH` emits an
  overflow hint too — one nobody had examined — so the real figure is **10 distinct live-document
  overflow destinations, 0 of them capped**. ANVIL bounds **16,115 B** and routes its overflow into
  **8,683,015 B**: a **539×** ratio. **The lesson is about the method, not the number:** `.0` read
  *one* enforcer's hint because that was the enforcer under discussion, and reported the result as
  the finding. Deriving the same fact from the whole enforcement surface found the rest in one run.
  *A hand-read sample of a mechanism reports the mechanism you were already looking at.*
- `2026-08-08`: Created on an explicit owner directive to adopt the doctrine as a project-owned copy,
  treating the originating repository as a **template, not an upstream**. Registration measured
  rather than assumed: **2 of 279** tracked documents are capped and they are the **two smallest**,
  while `MEMORY-ARCH`'s cap-relief routing hint points at **four** uncapped destinations — so the
  mechanism ANVIL uses to relieve size pressure has been moving bytes from a bounded file into
  unbounded ones and recording it as compliance. The doctrine's core invariant names that exactly:
  *a bounded file that sends overflow to an unbounded neighbour has not contained anything.*
