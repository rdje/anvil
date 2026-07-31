---
id: gfm-table-unescaped-pipe-drops-content
title: An unescaped `|` in a Markdown table cell — **including inside backticks** — splits the row, and every cell past the header's column count is **silently dropped from the rendered document**
answers:
  - "why did the end of my table row disappear when rendered"
  - "why is the link in my table's last column missing from the book"
  - "do backticks protect a pipe character inside a markdown table"
  - "how do I put a pipe character in a markdown table cell"
  - "what happens when a table row has more cells than the header"
  - "how do I find markdown table rows that silently lose content"
  - "why does a long table row render only partway"
date: 2026-07-31
status: current
tags: [markdown, mdbook, gfm, live-docs, rendering, gotcha]
evidence: 'docs/tasks/OVERFLOW-DESTINATION-INSTRUMENTATION.md leaf .6 — tree-wide sweep over 243 tracked *.md, 394 tables, 2,310 data rows: 36 malformed rows, 57,283 chars dropped, 5 link cells lost'
reverify: 'Put a 3-column table in a scratch mdbook whose second row carries an unescaped pipe inside a code span, run mdbook build, and count rendered <td> cells: the row splits and the trailing cell is absent from the HTML. Repeat with the pipe written as backslash-pipe: the row keeps 3 cells and the trailing cell returns. Both directions were run 2026-07-31.'
---

In GFM tables the row is split on `|` **before** inline parsing, so a code span does **not**
protect a pipe: `` | see `x | y` here | [L](y.md) | `` becomes four cells, not two. The spec
then says the excess is *ignored* — so with a 3-column header, `[L](y.md)` **never renders**.
The source still contains it; every reader of the rendered page loses it.

**Verified empirically, not assumed** (`OVERFLOW-DESTINATION-INSTRUMENTATION.6`, `2026-07-31`),
with the project's own renderer — `mdbook` / `pulldown-cmark`, against a 3-column header:

```markdown
| b1 | see `x | y` here | [LINKB](y.md) |     <- 4 cells; renders b1 / "see `x" / "y` here"
                                                 [LINKB](y.md) IS DROPPED
| c1 | see `x \| y` here | [LINKC](z.md) |    <- 3 cells; renders c1 / "see x | y here" / LINKC
```

**The fix is one backslash**: write `\|` inside the cell. The escape is consumed by the
table splitter, so the code span still renders a literal pipe and the row keeps its columns.

**Why it hides.** The failure is invisible from the source — the text is right there — and
invisible from the render — nothing marks the gap. It only appears when you compare the two, or
count separators against the header. A tree-wide sweep over 243 tracked `*.md` (394 tables,
2,310 data rows) found **36 malformed rows dropping 57,283 characters**, including **5 cells that
were the row's link to its own detail file**. The worst single case dropped **24,229 of 24,990
bytes — 97.0 % of the line**.

**Detect it by counting escape-aware separators per row against the header**, never with a naive
`gsub(/\|/)`: this project's first pass at that flagged 8 rows of which 3 were false positives
using the *correct* `\|` idiom — the standing lesson that an extractor which cannot tell the
escaped form from the bare one reports something plausible instead of dying. See
[[sweep-must-not-rewrite-its-own-subject]] and [[never-parse-a-formatter-for-a-semantic-set]].

**Corollary for long lines.** A table row **cannot be reflowed** — a newline terminates the row —
so "wrap the long line" is unavailable as a repair for any table row, however long it gets.
