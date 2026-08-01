---
id: book-links-to-the-file-it-stopped-duplicating
title: When the book **stops duplicating** a repo file, it links to it with an **absolute GitHub URL** — a narrow supersession of `0046` candidate **B**, admitted only where the link is the *sole route* to content deliberately removed; `0046` form **A** still governs every incidental reference, and `{{#include}}` stays rejected
answers:
  - "may a book chapter link to a repo file with an absolute GitHub URL"
  - "how should the book reference content it used to duplicate"
  - "is decision 0046 still in force"
  - "why not use mdbook include to pull a repo file into the book"
  - "what happens to a relative ../../X.md link in an mdBook chapter"
  - "do book links work when browsing the rendered book locally"
  - "why is the book built host-agnostic and does an absolute URL contradict that"
  - "should a book link pin main or a commit SHA"
  - "what is the offline cost of linking the book to GitHub"
  - "when may an accepted decision record be superseded"
date: 2026-08-02
status: accepted
supersedes: 0046 (candidate B rejection only — narrowed, not reversed)
tags: [docs, book, mdbook, link-integrity, convention, owner-directive, supersession]
evidence: docs/tasks/BOOK-PARAGRAPH-BLOBS.md (`.1`'s census and the CONFLICT block; `.3`'s scope); docs/decisions/0046-book-never-links-outside-book-src.md (candidates A–D and their measured failure modes); book/book.toml:14-15 (`git-repository-url = ""`, `edit-url-template = ""`); scripts/check_book_link_targets.sh:149 (absolute schemes are skipped, hence ungated)
reverify: "grep -c 'github.com/rdje/anvil/blob/main' book/src/*.md   # the admitted sites; each must be a sole-route link, not an incidental reference. Relative escapes must stay at zero: grep -rhoE '\\]\\((\\.\\./)+[^)]*\\.md' book/src/*.md | wc -l"
---

# 0048 - BOOK-PARAGRAPH-BLOBS.3: the book links to the file it stopped duplicating

- Date: 2026-08-02
- Status: accepted
- Tree: `BOOK-PARAGRAPH-BLOBS.3`
- **Supersedes** [`0046`](0046-book-never-links-outside-book-src.md) **candidate B only**, and
  narrows rather than reverses it. `0046`'s accepted form **A**, its candidate **C** rejection and
  its candidate **D** rejection all stand unchanged.
- Owner directive `2026-08-02`, twice: *"there is no need to have the content of an already existing
  `.md` file in the book if you can just provide a link to that `.md` file from the book"* and
  *"I prefer having a link pointing the `.md` files than using `{{#include ...}}`."*

## Context

`BOOK-PARAGRAPH-BLOBS.1` measured four **run-on enumerations** in the book — single sentences of
dozens of clauses, up to 7,159 characters — that are a **lossy copy** of `ROADMAP.md`. Verified
before proposing anything: of the **111** distinct `saw_*` coverage flags the book names, **0 are
book-only**; every sampled bank id (`r51`, `r73`, `r78`, `r83`) appears **2–3×** in `ROADMAP.md`
against **1×** in the book, with strictly richer prose there (`ROADMAP.md` records `r83`'s
**198-scenario** count; the book omits it). By
[`0033`](0033-shadow-enumeration-classification.md) that is a shadow, and its repair is **R1 —
delete the copy**.

Deleting it raises a question `0046` never faced. `0046` decided how a chapter should make an
**incidental reference** to a repo-root file — *"see `USER_GUIDE.md` for the level table"* — where
the chapter's own content stands on its own and the reference is a courtesy. It measured form **A**
(name it in backticks, do not link) unanimous at **31 of 32** sites and rejected the absolute GitHub
URL as candidate **B**.

Here the reference is not a courtesy. It is the **sole route** to 7,000 characters being removed
from the page. A reference that cannot be followed is, in that position, a deletion.

## Decision

**Where the book removes duplicated content, it links to the owning file with an absolute GitHub
URL. Everywhere else, `0046` form A still governs.**

```markdown
The per-bank register lives in
[`ROADMAP.md`](https://github.com/rdje/anvil/blob/main/ROADMAP.md) — Phase 4.
```

Three constraints, all load-bearing:

1. **Absolute, never relative.** mdBook rewrites a `.md` target to `.html`, so
   `[ROADMAP.md](../../ROADMAP.md)` renders as `../../ROADMAP.html` — **dead in the rendered book,
   alive on GitHub, and `mdbook build` exits `0` on it**
   ([[mdbook-md-to-html-rewrite-trap]]). `BOOK-LINK-TARGETS` blocks that form, and must keep doing
   so. Verified on the current build: an absolute target survives rendering with its `.md` intact
   (`href="https://github.com/rdje/anvil/blob/main/docs/AGENT_INTROSPECTION_SCHEMA.md"`), and the
   URL returns **HTTP 200** on the public repo.
2. **Sole-route only.** This is not a licence to link. An incidental mention stays form **A**; the
   31/32 convention `0046` measured is not reopened.
3. **Pin `main`, not a SHA.** The target is a *live* document whose current state is the point; a
   SHA would freeze it stale on the day it landed.

## The owner's preferred option was tested, and it does not exist

Owner, `2026-08-02`: *"look if it is possible to have links in the books that can allow me to
successfully open `.md` files via links in the books on both GitHub and locally, I vote for it,
otherwise prioritize books browsing on GitHub if there is a choice to make."*

The **relative** form is the candidate that would satisfy both offline *and* host-agnosticism, and
the repo layout makes it look achievable: `book/src/` and `book/book-out/` sit at the **same depth**,
so `../../ROADMAP.md` resolves to the repository root from *either*. The only obstacle is mdBook's
rewrite — so the question is whether any authoring form escapes it.

**Measured, not assumed.** A throwaway probe chapter was built with three link forms and the
rendered `href`s read back:

| Authored in `book/src` | Rendered in `book/book-out` | Verdict |
| --- | --- | --- |
| `[ROADMAP.md](../../ROADMAP.md)` | `href="../../ROADMAP.html"` | rewritten ❌ |
| `<a href="../../ROADMAP.md">` (**raw HTML**) | `href="../../ROADMAP.html"` | **also rewritten** ❌ |
| `[Knobs](knobs.md)` | `href="knobs.html"` | correct, intra-book ✔ |

The raw-HTML escape hatch — the obvious way to bypass a markdown-level rewrite — **does not work**:
mdBook rewrites the attribute too. So there is **no** relative form that survives into the rendered
book, and the owner's first choice is unavailable. The probe was removed and the book rebuilt clean.

That settles it on the owner's own fallback rule, *prioritize browsing on GitHub*: the absolute URL
is the only form that works **on GitHub and in the locally-rendered book** — the sole cost being a
network connection for the local reader, which the relative form would have avoided only by being
dead there anyway.

## Why `0046`'s two objections do not carry in this narrow case

**(i) "`book.toml` builds the book host-agnostic."** Verified still true —
`git-repository-url = ""` and `edit-url-template = ""` at `book/book.toml:14-15`. But those settings
govern mdBook's **chrome**: the repository icon and the per-page *Edit this file* link. They express
"do not wire the theme to a forge", not "no prose may name one". The distinction is real, and a
handful of scoped links do not undo the setting. Reversing the *setting* is deliberately **not**
part of this decision.

**(ii) "It breaks on a fork, a rename, a mirror, or an offline reader."** Every clause is true and
**accepted as a stated cost**, because the alternative is worse in this position:

- **Offline** — the honest one, and the owner raised it themselves. A local reader of
  `book/book-out/` cannot follow the link without a network. They can, however, open the file
  directly: **anyone browsing the book locally has the repository on disk**, and the link text names
  the path. The link is the convenience; the named path is the fallback. Compare the alternative:
  form **A** alone, after deletion, leaves a reader who *does* have a network with no route at all.
- **Fork / rename / mirror** — accepted. These links **rot silently**, and that is stated below
  rather than discovered later.

The cost of being wrong here is small and reversible: a dead link on a page whose content still
exists in the repo. The cost of *not* linking is content the owner reviews simply vanishing.

## `{{#include}}` stays rejected — now on the owner's authority as well as `0046`'s

`0046` candidate **D** rejected it on four counts, and the owner independently rejected it on
`2026-08-02`. It is the only candidate that is clickable *and* offline *and* host-agnostic, so its
rejection must not read as an oversight: it would inject the file into the book as a chapter,
re-creating in the rendered output exactly the wall of text `BOOK-PARAGRAPH-BLOBS.1` spent a session
removing — a **7,159-character** paragraph would come straight back. It is the right tool for a
short, stable fragment and the wrong one for a live evidence register.

## Honest limits, stated rather than discovered

- **These links are gated by nothing.** `check_book_link_targets.sh:149` skips `https?:` targets by
  design, so a renamed or moved target **404s silently** and no gate fires. Whether that deserves a
  checker is `BOOK-PARAGRAPH-BLOBS.2`'s question; per
  [`0047`](0047-negative-control-carrier-is-the-mutation.md) a by-product route is preferred to a new
  gate, and an offline-capable one is not obvious.
- **The 6 absolute links already in the book are not precedent for this decision.** They were
  introduced by `AGENT-INTROSPECTION-MCP.7`, `ACCEPTANCE-DIVERGENCE-HUNTING.2f` and
  `BOOK-API-REFERENCE.1`, all of which **predate `0046`** (`9ad7385`, `2026-08-01`) — they are
  residue that predates the decision rejecting the form, not an endorsed convention.
  `BOOK-PARAGRAPH-BLOBS.md` recorded the opposite at `a5645c1` and is corrected there; this record
  states the corrected fact so the error is not re-derived from the count.
- **This narrows one candidate; it does not reopen `0046`.** Form **A** remains the rule for
  incidental references, and a future audit should expect the overwhelming majority of repo-file
  references in `book/src` to carry no link at all.
