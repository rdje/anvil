---
id: mdbook-md-to-html-rewrite-trap
title: mdBook rewrites every `.md` link target to `.html`, so a chapter link to a repo-root file renders **dead** while resolving fine on GitHub — and `mdbook build` exits `0` on it
answers:
  - "why does a book link work on GitHub but 404 in the rendered book"
  - "does mdbook build check links"
  - "does mdbook build fail on a dead link"
  - "how do I link from a book chapter to a repo-root file like USER_GUIDE.md"
  - "why is my link to ../../USER_GUIDE.md dead in the book"
  - "how do I link-check a rendered mdBook"
  - "are there dead anchors in the book"
  - "why would a source-level link check miss a dead book link"
  - "how many links does the anvil book have"
  - "does print.html double-count a bad book link"
date: 2026-08-01
status: current
tags: [mdbook, docs, book, link-integrity, gotcha, measurement, gate-quality]
evidence: docs/tasks/BOOK-LINK-INTEGRITY.md (`.1`'s two-derivation measurement — rendered and authored — with denominators, named offenders, and a four-predicate negative control on one sabotaged chapter)
reverify: "grep -rnE '\\]\\([^)]*\\.\\./[^)]*\\.md' book/src/*.md   # chapter links that escape book/src — each renders as a .html path outside book-out and is DEAD. Approximate: catches the escape class only, not intra-book dead files or dead anchors."
---

**mdBook rewrites a link target ending in `.md` to `.html` when it renders.** That is correct and
necessary *inside* the book (`knobs.md#anchor` → `knobs.html#anchor`). It is a trap the moment the
target leaves `book/src`: the natural-looking

```markdown
[USER_GUIDE.md](../../USER_GUIDE.md#tracing-and-debugging)
```

renders as `../../USER_GUIDE.html`, **which does not exist**. The link resolves when the Markdown
*source* is read on GitHub and is dead in the rendered book — the surface the owner actually
reviews. `mdbook build` exits **`0`**, and no doctrine looks at links at all.

## The part that defeats the obvious check

The target `.md` file **exists**. A source-level check that asks *"does the link target exist?"*
**passes this link.** The defect is not a missing file; it is a **rendered target that escapes
`book-out`**. Any check for this class must model the `.md`→`.html` rewrite *and* ask whether the
rewritten path still lands inside the build dir — otherwise it is vacuous against the one defect
the class is named for. See [[coverage-check-vacuity]].

## What was measured (`2026-08-01`, and how to redo it)

Two independent derivations, both required — the rendered side is dominated by generated chrome,
the authored side is the population that can actually rot:

- **rendered** — `mdbook build book`, then over `book/book-out/**/*.html` (build-dir is
  `book-out`, not mdBook's default `book`, per `book/book.toml`) resolve every `href` relative to
  its containing file. Separate `<a>` from `<link>`/`<base>` chrome or the denominator is
  meaningless.
- **authored** — over `book/src/*.md`, apply the `.md`→`.html` rewrite to every markdown link and
  ask both *does the rendered target exist* and *does it stay inside `book-out`*.

**Dead file targets: 1 authored site** (`book/src/recipes.md:857`). **Dead anchors: 0** — a
negative-controlled zero, not an unmeasured one. Denominators and every count live in the tree;
they are deliberately not copied here, per
[`0033`](../decisions/0033-shadow-enumeration-classification.md) — a metric copied out of its
home rots.

## Three things that will bite the next person

- **`print.html` double-counts.** mdBook concatenates every chapter into it, so one bad chapter
  link is **two** rendered occurrences. Report **sites**, and say so.
- **`print.html` has its own anchor namespace.** Cross-chapter links are rewritten there from
  `knobs.html#x` to bare `#x`, resolved against the merged page — so anchors must be checked
  per-file, never globally.
- **`<base href="/">` in mdBook's generated `404.html` is not a link.** A tag-agnostic `href="…"`
  regex reports it as dead. Match the element, not just the attribute.

## The negative control that makes the zero mean something

Four defects appended to one chapter — dead intra-book file, dead in-page anchor, dead
cross-chapter anchor, and the repo-root escape — plus two live controls; rebuild, scan, restore.
All four were caught by both derivations and **`mdbook build` still exited `0` with all four
present**. One "live" control (`ir.md#node`) flagged too, and checking it showed `ir.html` has no
such anchor: the guess was wrong, the instrument was right. The positive control is the 71
authored anchors that resolved — see [[negative-control-must-be-able-to-fail]].
