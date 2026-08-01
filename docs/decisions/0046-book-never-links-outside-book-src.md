---
id: book-never-links-outside-book-src
title: A book chapter **names** a repo-root file in backticks; it never **links** to one — mdBook's `.md`→`.html` rewrite makes any such link dead in the rendered book, and the convention already held at **31 of 32** sites before it was written down
answers:
  - "how should a book chapter reference USER_GUIDE.md or another repo-root file"
  - "can I link from an mdBook chapter to a file outside book/src"
  - "why is a markdown link to ../../USER_GUIDE.md forbidden in the book"
  - "should the book link to GitHub with an absolute URL"
  - "should USER_GUIDE.md be included into the book as a chapter"
  - "how do I point at a section of a repo-root doc from the book"
  - "what is the checkable rule for book link integrity"
  - "why not use mdbook include to pull in a repo-root file"
date: 2026-08-01
status: accepted
tags: [docs, book, mdbook, link-integrity, convention, doctrine, shadow-list]
evidence: docs/tasks/BOOK-LINK-INTEGRITY.md (`.1`'s two-derivation measurement — the sole offender; `.2`'s convention census); book/book.toml (`git-repository-url = ""` — the book is deliberately built host-agnostic); book/src/recipes.md:653 (the sibling site in the same chapter already using the chosen form); docs/decisions/0033-shadow-enumeration-classification.md (the rule disqualifying candidate C)
reverify: "grep -rnE '\\]\\([^)]*\\.\\./[^)]*\\.md' book/src/*.md   -> expect 0 matches; the convention census is `grep -roE '(USER_GUIDE|README|ROADMAP|CHANGES|TOOLBOX|DEVELOPMENT_NOTES|CODEBASE_ANALYSIS|MEMORY|COMMIT)\\.md' book/src/*.md | wc -l`"
---

# 0046 - BOOK-LINK-INTEGRITY.2: the book names repo-root files, it does not link to them

- Date: 2026-08-01
- Status: accepted
- Tree: `BOOK-LINK-INTEGRITY.2` (the repair-form decision; `.1` supplied the measurement)
- Activated by: autonomous PNT selection under the owner's standing **DECIDE, DON'T ASK**
  directive ([`0041`](0041-owner-standing-directives-recorded-in-layer-c.md))

## Context

`.1` measured the whole book and found **exactly one** dead link:

```markdown
book/src/recipes.md:857
See [USER_GUIDE.md](../../USER_GUIDE.md#tracing-and-debugging)
```

mdBook rewrites every `.md` link target to `.html`, so this renders as `../../USER_GUIDE.html`
— a file that does not exist. It resolves when the *source* is read on GitHub and is **dead in
the rendered book**, which is the surface the owner reviews. `mdbook build` exits `0`.

### The census that decided it

Before choosing, the book was asked what it already does. Every occurrence of a repo-root
live-doc filename in `book/src/*.md`:

| form | occurrences |
| --- | ---: |
| plain backtick prose — `` `USER_GUIDE.md` `` | **31** |
| markdown link (`recipes.md:857`, counted twice: link **text** + link **target**) | **2** |
| **total** | **33** |

**31 of 32 distinct references already use plain backticks.** The convention was not invented
here; it was already unanimous but for one site. `USER-GUIDE-CLI-TABLE-SHADOW.4` independently
reached the same form under time pressure for its own two new pointers. This record makes the
de-facto rule explicit and checkable.

## Decision

**A book chapter refers to a file outside `book/src` by naming it in backticks, with any
section named in parentheses as prose — never as a markdown link.**

```markdown
See `USER_GUIDE.md` ("Tracing and debugging") for the level table and emoji legend.
```

This is exactly the form the sibling site `book/src/recipes.md:653` in the *same chapter*
already uses. In-book chapter links (`knobs.md#anchor`) are untouched and remain correct —
they are what mdBook's rewrite is *for*.

## The candidates, each with its failure mode

### A — plain-backtick prose ✅ **chosen**

- **Failure mode: not clickable.** The reader navigates manually. This is a real cost and it is
  accepted.
- **Second failure mode, stated rather than hidden:** the parenthesised section name is a
  *shadow* of the target's heading text and rots silently if that heading is renamed. **But the
  `#anchor` form rots on exactly the same event and rots worse** — a stale anchor silently lands
  the reader at the top of the page with no signal, whereas stale prose is at least legible as
  prose. So on the axis where A is weakest it is still no worse than the form it replaces.
- Strengths: renders correctly in the book, on GitHub, and in plain text; host-agnostic;
  survives fork, rename, and a move off GitHub; already unanimous at 31/32.

### B — absolute GitHub URL ❌

`https://github.com/rdje/anvil/blob/main/USER_GUIDE.md#tracing-and-debugging`

- **Disqualified by the book's own configuration.** `book/book.toml` sets
  `git-repository-url = ""` and `edit-url-template = ""` — the book is *deliberately* built
  without a host. Hard-coding one in prose contradicts a setting the project already made.
- Breaks on a fork, an org rename, a mirror, or an offline reader. Pins either `main` (a moving
  target, so the anchor may drift under it) or a SHA (stale the moment it lands).

### C — a book-visible copy of the target ❌

- **Disqualified by [`0033`](0033-shadow-enumeration-classification.md).** It creates a second
  copy of the CLI reference — precisely the shadow `USER-GUIDE-CLI-TABLE-SHADOW` was opened to
  delete. `.4` *removed* the book's CLI-flag copy; re-adding one to repair a link would undo
  that leaf to fix a one-line defect.

### D — mdBook `{{#include ../../USER_GUIDE.md}}` ❌

Recorded because it is the **only** candidate that would make the reference genuinely clickable
without pinning a host, so its rejection should not look like an oversight. It is a *derived*
copy produced at build time, not a hand-maintained one, so it escapes `0033` on a technicality.
It fails on four other counts:

1. it injects a **163 KB** file into the book as a single chapter, duplicating the entire CLI
   reference in the rendered output;
2. its headings collide with existing chapter anchors, and worse in `print.html`, whose merged
   single-page anchor namespace `.1` documented;
3. it makes the book build depend on a file outside `src`, which `mdbook serve`'s watch and any
   future `src`-scoped tooling will silently miss;
4. it restructures the table of contents — a structural change, to repair one line.

## Why this matters beyond one link

The rule is a **simpler predicate than a link checker**, and that is the point. `.1` showed the
obvious portable check — *"does the link target exist?"* — **passes this very defect**, because
`../../USER_GUIDE.md` does exist; the defect is a rendered target that *escapes* `book-out`. A
check for the convention above needs neither `mdbook` nor a rendered build:

> no markdown link in `book/src/*.md` may have a target that escapes `book/src`

That is greppable, portable, and exact for this class. Whether it is *registered* as a doctrine
is deliberately left to `BOOK-LINK-INTEGRITY.3`, which must also weigh the classes this
predicate does **not** cover (intra-book dead files, dead anchors — both measured at **zero** by
`.1`, which is an argument about *current* risk, not about permanent immunity).

## Outcome (added by `.3`, same day — this record is not otherwise amended)

`.3` answered **yes**: registered as the `BOOK-LINK-TARGETS` doctrine
(`scripts/check_book_link_targets.sh`), the eleventh in the registry. It enforces **both** halves
that are exact at source level — *escape* (tested first, because it is the one the obvious check
misses) and *existence* — and leaves `#fragments` out of scope for the reason stated above.
