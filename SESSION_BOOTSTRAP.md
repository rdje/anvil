# Session Bootstrap
Read this first when starting or recovering a session.

> This file is **re-injected verbatim after context compaction** by the `PostCompact` hook
> in `.claude/settings.json`. After an auto- or manual `/compact` the protocol below is
> freshly in force — re-run it, and do not assume prior in-context state survived. That
> re-injection is also why the contract below is *tiered*: a session pays Tier 1 once per
> compaction cycle, not once per session.

## The contract: read what the WORK needs

**Owner directive, `2026-08-07`, verbatim:** *"read whatever is necessary for the new
session. The idea is to be knowledgable about what need to be worked on, that is, because I
find it dangerous to fix something you don't understand."*

The read is therefore **scoped to the change in hand**, and its sufficiency test is
**understanding the subject of that change** — never coverage of a corpus. Read until you
can state what the code does and why *before* touching it; that is the whole test.

The **shape** of this read is not this file's to invent. `MEMORY_ARCHITECTURE.md` §5 already
prescribes it — *"a resume reads A + one unit of B + a few C records — never a monolith"* —
so that standard owns the read path and this file only **binds it to this repository's
filenames**, adding the project-specific tiers below and the sanity checks. One mechanism,
never two (`feedback_full_factorization`).

Why the previous *"read every live doc, in full"* text was replaced, and what each demotion
below costs, is measured in
[`docs/decisions/0049`](docs/decisions/0049-the-bootstrap-read-is-scoped-to-the-work.md):
that contract totalled **12.7 MB ≈ 3.0×** a full context window, and its live-doc section
alone exceeded one, so no session ever satisfied it.

## Tier 1 — fixed: read every time, including after every compaction

These are invariant of the work — they define how to work here at all, so a session cannot
know in advance whether it needs them. Measured `2026-08-07`: **120,178 B, ≈ 2.9 %** of a
4 MiB context window, against a derived budget of **256 KiB** it fills to **45.8 %**.

| Read | Why it is invariant of the work |
| --- | --- |
| `README.md` | the objective, the three load-bearing principles, and the canonical-navigation table that routes every read below |
| `MEMORY.md` | layer A — where the last session stopped, and the single next action. Byte-capped by the `MEMORY-ARCH` doctrine *precisely so* it is always affordable |
| `MEMORY_ARCHITECTURE.md` | the four memory layers and the read/write paths this file instantiates |
| `DOCTRINE_ENFORCEMENT.md` | the live doctrine registry — what will mechanically block the commit |
| `TOOLBOX.md` | the instruments to reach for *before* reading `src/`, plus the acceptance checklist a code change must satisfy |
| `COMMIT.md` | every commit executes it |
| `docs/TASK_TREE.md`, its **rule** sections | the workflow itself: ID rules, frontier rules, PNT selection, splitting, completion, and the code/not-code boundary. Measured `2026-08-07` at **11,466 B — 3.8 %** of that file; the rest is the index table, which Tier 2 reads one row of |

This table is **authoritative, not a shadow** ([`0033`](docs/decisions/0033-shadow-enumeration-classification.md)):
no other set in the repository enumerates the invariant read, so there is nothing it can
silently fall out of date against.

## Tier 2 — the subject: derived per leaf, and the tier the directive is about

There is no static list here by construction — what grounds a change varies with the
change, which is why the replaced §1/§2 enumeration was the wrong *shape* and not merely
the wrong *length*. Read, in this order:

1. **The active work unit.** `MEMORY.md`'s `next_action` names the leaf; open its owning
   `docs/tasks/<TREE>.md` in full, plus that tree's **single row** in `docs/TASK_TREE.md`'s
   *Active Task Trees* table (mean row **3,166 B**, measured `2026-08-07`). The remaining
   rows are a lossy re-statement of the tree files they index —
   [`0042`](docs/decisions/0042-task-tree-index-is-a-mixed-surface.md) measured **416 of
   508 leaves (81.9 %)** re-stated there — so reading them adds ~0 unique fact.
2. **The source the change touches**, at source level, in full.
   `CODEBASE_ANALYSIS.md` is the **map, never the substitute**: it names **55 of 55**
   `src/*.rs` files in **7.4 %** of the source mass (measured `2026-08-07`), and its
   per-file claims are known to go stale —
   [`0039`](docs/decisions/0039-sweep-exemption-past-vs-present-and-recorded-recall.md)
   measured **9 of 13** stale, every error an under-count.
3. **The book chapters that document the concept being changed**, reached through
   `book/src/SUMMARY.md`. The chapters this file forbids editing casually — `core-idea.md`,
   `non-goals.md`, `why-not-grammar.md` — carry the design decisions that cannot be
   reconstructed from code, and with `SUMMARY.md` cost **20,391 B, 2.7 %** of the book
   (measured `2026-08-07`). Read them whenever the change is a design change.
4. **The decision records the leaf cites**, plus whatever the Knowledge Map returns for the
   question actually in hand.

**If the subject genuinely will not fit, the slice is too big.** That is a scoping signal,
not licence to read less: split the leaf.

## Tier 3 — queried, never loaded

Reached by **lookup on the question you have**, not by reading through:

| Surface | How to reach it | Why never in full |
| --- | --- | --- |
| `KNOWLEDGE_MAP.md` + `docs/knowledge/` | scan *Questions → fact*, follow the one pointer, trust the dated fact or run its `reverify` | a derived index; `knowledge-map/KNOWLEDGE_MAP_ARCHITECTURE.md` §7 prescribes lookup and §1 says the map may be skipped entirely in favour of grepping the fact files |
| `CHANGES.md` | `grep -n '<LEAF-ID>' CHANGES.md`, or `git log --grep` | layer D, which `MEMORY_ARCHITECTURE.md` §3 already says is *queried, not loaded*. Measured `2026-08-07`: **632 of 709** entries carry a `**Landed as:**` hash and **459** name a leaf id in their heading, so the entry is reachable by key |
| `DEVELOPMENT_NOTES.md` | grep by topic | append-only rationale, same class. With `CHANGES.md` it was **68.6 %** of the replaced mandate's mass, so the old contract receded with every commit |
| `USER_GUIDE.md` | grep by flag, knob, or preset | a reference whose length tracks the product surface — [`0040`](docs/decisions/0040-overflow-destination-classification-and-the-unmeasured-axis.md) class **B2** |
| `ROADMAP.md` | read the lane when **choosing** the next tree | executing a leaf needs only its lane, and every tree file carries its own `- Roadmap lane:` field (**84 of 84**, measured `2026-08-07`) |
| `docs/decisions/`, `docs/evidence/` | by record number / bank name | records are cited, not swept |
| the remaining book chapters | via `book/src/SUMMARY.md` | subject-scoped by Tier 2 |
| every `.rs` under `src/`, `tests/`, `examples/` | via `CODEBASE_ANALYSIS.md`, then read the files the change touches | **3.9 MB ≈ 0.94×** a context window; walking it whole was never achievable, and Tier 2 already requires the part that matters |

## Keep `CODEBASE_ANALYSIS.md` true

Amend it whenever a slice changes workspace reality — module boundaries, helpers, enforced
invariants, knobs, phase coverage, testing surface (`COMMIT.md` step 5). The window is any
moment, not only bootstrap: at any commit point the file must describe the code as it now
is, so an interrupted project loses nothing. Do not rewrite cosmetically.

## Non-negotiable doctrine: task-tree ownership of code

**(2026-05-17, owner directive, no compromise.)** It is **strictly
forbidden to make any code change without it being task-tree tracked
or task-tree owned first.** Before touching `src/`, `tests/`,
`examples/`, or any build/codegen logic, confirm a task-tree leaf owns
the change; if none exists, create or extend a tree
(`docs/tasks/<TREE>.md` + a `docs/TASK_TREE.md` row) and name the
owning leaf **before** editing code. The leaf ID goes in the commit
subject. Pure-docs / live-doc / mdBook / workflow-config edits are not
"code changes" and are exempt. Full statement and the code/not-code
boundary: `docs/TASK_TREE.md` "ANVIL Adoption Scope" and `COMMIT.md`
"Task-tree-managed commits". A recovering session must treat this as
in force immediately.

## Sanity checks
```bash
cargo check --all-targets
cargo test
git --no-pager log -5 --oneline
git --no-pager status --short
```

Expected state:
- `cargo check` passes.
- `cargo test` passes.
- `git status` is clean, or shows an in-progress slice consistent with `MEMORY.md`.

Any deviation is a signal to stop and investigate before making changes.

## What not to do on bootstrap
- Do not spend the session's budget proving you read something. Nothing observes
  bootstrap-read completeness and nothing can — the only evidence a session can produce is
  its own claim, which is the self-ticked box `DOCTRINE_ENFORCEMENT.md` §6.1 disqualifies.
  What is observable is the work: a change grounded in its subject survives the gates.
- Do not edit `book/src/core-idea.md`, `book/src/non-goals.md`, or `book/src/why-not-grammar.md` as a warm-up. Those capture load-bearing design decisions; revising them requires a deliberate task.
- Do not reorganize the crate layout to match a mental model formed before reading `CODEBASE_ANALYSIS.md` and the source the change touches.
- Do not commit without running the full `COMMIT.md` workflow.

## When in doubt
Open `MEMORY.md`. It records what the last session was doing, what landed, and what was about to happen next. If `MEMORY.md` is stale or contradicts `git log`, trust `git log` and update `MEMORY.md` as part of the next commit per `COMMIT.md`.
