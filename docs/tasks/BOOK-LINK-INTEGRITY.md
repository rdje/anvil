# BOOK-LINK-INTEGRITY: `mdbook build` is not a link checker, and the book's links to repo-root files render dead

## Metadata

- Tree ID: `BOOK-LINK-INTEGRITY`
- Status: `active`
- Roadmap lane: Live-doc hygiene / book fidelity
- Created: `2026-08-01`
- Last updated: `2026-08-01` (`.1` measured both classes; frontier `.2`)
- Owner: repo-local workflow

## Goal

The mdBook is, in the owner's words, the **only window into the project** — *"I review the book,
not the code."* A link in it that resolves to nothing is therefore not cosmetic: it is the review
surface failing at the one job a pointer has.

**mdBook rewrites every `.md` link to `.html` when it renders.** So the natural, correct-looking
Markdown link from a chapter to a repo-root file —

```markdown
[USER_GUIDE.md](../../USER_GUIDE.md#knobs)
```

— renders as `../../USER_GUIDE.html`, which **does not exist**. It resolves correctly when the
Markdown *source* is read on GitHub, and is dead in the rendered book. `mdbook build` exits **0**
on it, and nothing in `scripts/check_doctrines.sh` looks at links at all.

## How it was found (`2026-08-01`, at `ccfbc23`)

Not by a sweep — by `USER-GUIDE-CLI-TABLE-SHADOW.4` **nearly shipping one**. That leaf deleted the
book's copy of the CLI flag list and replaced it with a pointer at `USER_GUIDE.md`, using the form
`book/src/recipes.md:857` already used. Link-checking the rendered HTML (rather than trusting
`mdbook build`'s exit code) showed the new pointer was dead — i.e. the repair's entire payload was
a link to nowhere.

The two new pointers were switched to the plain-backtick form the book's other two references to
`USER_GUIDE.md` already use (`architecture.md:175`, `structured-emission.md:184`), taking the book
from **6 dead local links to 2**.

## The measurement (at `ccfbc23`, after `.4`'s own fix)

| bucket | count |
| ---: | --- |
| local (non-`http`) hrefs in the rendered book | **665** |
| dead | **2** |

Both survivors are one site: `book/src/recipes.md:857`'s `[USER_GUIDE.md](../../USER_GUIDE.md#tracing-and-debugging)`,
and the copy of it mdBook emits into `print.html`.

The instrument, stated so it can be re-run: render with `mdbook build book`, then for every
`href="…"` in `book/book-out/*.html` that is not `http`/`mailto`/`javascript`, resolve it relative
to the containing file and assert the target exists. Note `book-out` — the build dir is
`book-out`, not mdBook's default `book`, per `book/book.toml`.

**Superseded as the instrument of record by `.1` below.** That scan's `665` is reproducible and
is re-derived exactly in `.1` — but it is *not* "local hrefs". It is **file references including
`<link>`/`<base>` chrome** (339 `<a>` file refs + 326 stylesheet/font/base hrefs = 665), and it
**excluded all 1101 in-page anchors**, which is precisely why the anchor class was unmeasured.

## `.1` — the measurement (`2026-08-01`, at `614e977`)

Two **independent derivations**, because one alone is not trustworthy here: the rendered side is
dominated by generated chrome, and the authored side is the population that can actually rot.

### Derivation 1 — RENDERED

**Region:** `book/book-out/**/*.html` — **33** files, after `mdbook build book`.
**Denominator:** **1782** `href="…"` occurrences, classified so every candidate lands in exactly
one bucket:

| bucket | count |
| --- | ---: |
| `<a>` | **1456** |
| `<link>` (stylesheets, fonts) | 325 |
| `<base>` | 1 |
| — of the 1456 `<a>`: external (`http`/`mailto`/…) | 16 |
| — in-page anchor (`#…`) | **1101** |
| — file reference | **339** |
| — …of those, carrying a `#fragment` | 35 |
| **anchor-class denominator** (1101 + 35) | **1136** |

- **CLASS A — dead file target: 2 occurrences, 1 distinct href.**
  `../../USER_GUIDE.html#tracing-and-debugging`, in `book/book-out/recipes.html` **and**
  `book/book-out/print.html`.
- **CLASS B — dead anchor: 0 occurrences**, out of a **1136** anchor-class denominator.

### Derivation 2 — AUTHORED (`book/src/*.md`)

**Denominator:** **229** authored markdown links = **228** inline `[…](…)` + **1** reference
definition (`[mcp]:` in `agent-mcp.md`). Recall audited against the raw source: `grep -o '](' `
returns **228** — the extractor captured **228/228**, not a subset ⇒
[[extractor-charset-narrower-than-source]]. Also checked and accounted for: **0** image links,
**0** raw `<a href` in markdown, **0** autolinks, **5** `][` occurrences of which **2** are the
reference-style `[MCP][mcp]` usages resolving to that one *external* definition and **3** are
array-index syntax (`transitions[state][sel]`), not links.

| bucket | count |
| --- | ---: |
| external | 7 |
| in-page anchor (`#…`) | 35 |
| file reference | 187 |
| — …carrying a `#fragment` | 36 |
| **anchor-class denominator** (35 + 36) | **71** |

- **Escapes `book-out` — the `.md`→`.html` trap: 1 site.**
  `book/src/recipes.md:857` → `../../USER_GUIDE.md#tracing-and-debugging`.
  **The target `.md` exists.** It is dead only because the rewrite sends it outside the build dir.
- **Dead file target inside the book: 0.**
- **Dead anchor: 0.**

The two derivations reconcile: 1 authored site ⇒ 2 rendered occurrences (chapter + `print.html`).

### The negative control (both predicates, one sabotaged chapter)

`book/src/faq.md` was copied aside, appended with **four defects and two live controls**, rebuilt,
scanned by both derivations, then restored from the copy and re-scanned to confirm the baseline
returns. `git status` clean afterwards; **no book edit survives this leaf.**

| probe | caught? |
| --- | --- |
| dead intra-book file (`does-not-exist-xyz.md`) | **yes**, both derivations |
| dead in-page anchor (`#no-such-heading-xyz`) | **yes** |
| dead cross-chapter anchor (`knobs.md#no-such-anchor-xyz`) | **yes** |
| repo-root escape (`../../CHANGES.md`) | **yes**, both derivations |
| live control `knobs.md` | correctly **not** flagged |
| live control `ir.md#node` | **flagged** — and `ir.html` genuinely has no `id="node"`. The guess was wrong, the instrument right. |

- **`mdbook build` exited `0` with all four defects present** — re-confirmed directly, not assumed.
- The **positive** control for CLASS B is not one link but the **71** authored anchors that
  resolved: the predicate demonstrably returns *alive* as well as *dead* ⇒
  [[negative-control-must-be-able-to-fail]], [[coverage-check-vacuity]].

### Instrument of record

Kept at `.cache/book_link_scan.py` (rendered) and `.cache/book_link_source.py` (authored) —
untracked by design, since `.3` and not `.1` decides whether a mechanism is warranted, and
committing a script under `scripts/` would preempt that. Both are re-derivable from this
statement; a one-line approximation of the escape class alone is
`grep -rnE '\]\([^)]*\.\./[^)]*\.md' book/src/*.md`.

Two traps a re-implementation must avoid, both hit during this leaf:

1. **Match the element, not just the attribute.** A tag-agnostic `href="…"` regex reports
   mdBook's generated `<base href="/">` in `404.html` as a dead link. It is not a link.
2. **Check anchors per-file.** In `print.html` mdBook rewrites cross-chapter `knobs.html#x` to a
   bare `#x` resolved against the merged page — a *different* namespace from the per-chapter one.

## Why a new tree rather than a leaf of an existing one

`feedback_full_factorization` — one mechanism, never two — was applied, not waved through:

- `TABLE-RENDER-FIDELITY` owns markdown **table** well-formedness. Not links.
- `ENUMERATION-PARITY` owns a docs list mirroring an authoritative set. A link is not an
  enumeration.
- `USER-GUIDE-CLI-TABLE-SHADOW` owns shadows of clap's flag registry. Different subject.

**No registered doctrine and no active tree owns book link integrity.** The subject is unowned, so
it registers on its own — the same reasoning that gave `TABLE-RENDER-FIDELITY` its own doctrine
rather than an assertion inside another.

## Non-Goals

- **Not a link-style reformat of the book.** In-book chapter links (`knobs.md#anchor`) are correct
  and are what mdBook's rewrite is *for*; they are not in scope.
- **Not "ban links to repo-root files".** That is one candidate repair among several (see `.2`),
  and picking it before measuring is the mistake `USER-GUIDE-CLI-TABLE-SHADOW.1` made and `.2`
  corrected.
- **No code change.** This is docs + possibly one enforcement script.

## Acceptance Criteria

- `.1` measures **both** classes — dead *file* targets and dead *in-page anchors* — and names the
  offenders rather than counting them. `.4`'s scan covered file targets only; whether a
  `#fragment` resolves to a real heading was **not** checked, and is the more likely silent class
  given how many in-book anchors the docs carry.
- `.2` decides the repair **before** editing: plain-backtick prose, an absolute GitHub URL, or a
  mdBook-visible copy of the target — each with its failure mode stated.
- If a check is written it obeys `DOCTRINE_ENFORCEMENT.md` §4 and is negative-controlled both
  ways. **It must be decided honestly whether it can read the repository at all**: a doctrine
  check reads the repo (§4(4)), and link-checking the *rendered* book needs `mdbook build` — a
  tool ANVIL does not vendor. That is the same objection `USER-GUIDE-CLI-TABLE-SHADOW.3` raised
  against deriving from `anvil --help`, and it may force a source-level check (does the link
  target exist as a `.md`, and would the rewrite break it?) rather than a rendered-output one.
- `scripts/check_doctrines.sh` stays green; docs-only ⇒ DUT byte-identical.

## Task Tree

- ID: `BOOK-LINK-INTEGRITY`
  Status: `active`
  Goal: `Make the rendered book's links resolve, and decide whether the class warrants a mechanism.`
  Children: `.1` (measure both classes + register), `.2` (decide the repair, then repair), `.3` (the mechanism question)

- ID: `BOOK-LINK-INTEGRITY.1`
  Status: `done`
  Goal: `Measure the rendered book for BOTH dead file targets and dead in-page anchors, name every offender, and state the instrument precisely enough to re-run. No repair in this leaf.`
  Acceptance: `Offenders NAMED not counted, for both classes. The anchor class must actually be measured, not assumed empty — .4's scan did not cover it. The match count is recorded alongside the finds (decision 0039), so recall is auditable. No book edit.`
  Verification: `MET. Two independent derivations (rendered: 1782 hrefs over 33 files, anchor-class denominator 1136; authored: 229 links, anchor-class denominator 71), each denominator stated and each offender named. CLASS A = 1 authored site / 2 rendered occurrences, named. CLASS B = 0, and the zero is negative-controlled: four planted defects caught, 71 real anchors resolved. Authored-extractor recall audited at 228/228. No book edit survives — the sabotaged chapter was restored and the baseline re-confirmed.`
  Commit: `BOOK-LINK-INTEGRITY.1 — measure both dead-link classes; the anchor class is empty`

- ID: `BOOK-LINK-INTEGRITY.2`
  Status: `done`
  Goal: `Decide the repair form for a book -> repo-root reference, record it, then repair every site .1 found to match.`
  Acceptance: `The decision is recorded BEFORE any edit, with each candidate's failure mode stated: plain-backtick prose (works everywhere, not clickable), an absolute GitHub URL (clickable, breaks on a fork or a rename, and pins a host), a book-visible copy of the target (clickable, and a new shadow under decision 0033 — likely disqualifying). Whichever lands, book/src/recipes.md:857 is fixed and the rendered book has zero dead local links.`
  Verification: `MET. Decision 0046 written and committed-to BEFORE the edit, with four candidates' failure modes stated — the three named above plus mdBook {{#include}}, recorded because it is the only candidate that is both clickable and host-agnostic, so its rejection must not read as an oversight. Chosen: plain-backtick prose with the section named in parentheses, matching recipes.md:653 in the same chapter. The choice was decided by CENSUS, not taste: 31 of 32 existing book -> repo-root references already used it. Post-repair the rendered book has ZERO dead local links, confirmed by both derivations, and the convention is unanimous at 32/32.`
  Commit: `BOOK-LINK-INTEGRITY.2 — the book names repo-root files, it does not link to them`

- ID: `BOOK-LINK-INTEGRITY.3`
  Status: `pending`
  Goal: `Decide whether the class warrants a mechanism, given that the honest obstacle is tooling: link-checking the rendered book needs mdbook, which ANVIL does not vendor and a fresh clone may not have.`
  Acceptance: `The decision follows .2's repair form. If a check is written it obeys DOCTRINE_ENFORCEMENT.md section 4, is negative-controlled both ways, and states plainly whether it checks SOURCE (portable, approximate — it must model mdBook's .md -> .html rewrite) or RENDERED output (exact, but skipped wherever mdbook is absent, which is exactly where a backstop is needed). If no mechanism is warranted, say so and say why, per section 9's honest-limits discipline — do not register a check that silently skips.`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `BOOK-LINK-INTEGRITY.3` | `pending` | **Next, and last.** The mechanism question. `.2` changed its shape for the better: the rule to gate is no longer *"is this link dead?"* (which needs `mdbook`, unvendored) but *"does a `book/src` markdown link escape `book/src`?"* — greppable, portable, no build required. `.3` must still weigh what that predicate does **not** cover and decide honestly, per `DOCTRINE_ENFORCEMENT.md` §9; "no mechanism, and here is why" remains a legitimate outcome. |
| — | `BOOK-LINK-INTEGRITY.1` | `done` | Both classes measured, offenders named, negative-controlled. **The anchor class is empty** (0 of 71 authored / 1136 rendered). |
| — | `BOOK-LINK-INTEGRITY.2` | `done` | Decision [`0046`](../decisions/0046-book-never-links-outside-book-src.md) recorded before the edit; the single site repaired. **Rendered book: 0 dead local links.** Convention now unanimous, 32/32. |

## Decisions

- `2026-08-01` (`.2`): **The book *names* a repo-root file in backticks; it never *links* to
  one.** Recorded in full as [`0046`](../decisions/0046-book-never-links-outside-book-src.md),
  written **before** the edit as this leaf's acceptance required. Chosen by **census, not taste**:
  **31 of 32** existing book → repo-root references already used plain backticks, so the rule was
  already unanimous but for the one offender. The absolute-GitHub-URL candidate is disqualified by
  the book's *own* configuration — `book/book.toml` sets `git-repository-url = ""` and
  `edit-url-template = ""`, i.e. the project already decided the book is built host-agnostic. A
  fourth candidate not in the original acceptance list, mdBook `{{#include}}`, was evaluated and
  rejected on four counts and is recorded rather than dropped, because it is the **only** option
  that is both clickable *and* host-agnostic and its omission would otherwise look like an
  oversight.
- `2026-08-01` (registration): **A new tree, not a leaf of `USER-GUIDE-CLI-TABLE-SHADOW`.** That
  tree owns *shadows of clap's flag registry*; this is link integrity. `feedback_full_factorization`
  forbids a second mechanism for one job — and no existing doctrine or tree owns this job, so
  registering here creates the first, not a second.
- `2026-08-01` (registration): **Registered rather than fixed in passing.** `recipes.md:857` is a
  one-line fix and the temptation was to bundle it into `USER-GUIDE-CLI-TABLE-SHADOW.4`. The
  standing directive is that a defect is only handled if a tree owns it, `COMMIT.md` lands one
  leaf per commit, and the pivot rule requires a clean tree first. A one-line fix that skips all
  three would have left the *class* — the anchor half, and the missing mechanism — untracked.

## Open Questions

- ~~**Are there dead in-page anchors too?**~~ **ANSWERED by `.1`: no — zero.** Out of an anchor-class
  denominator of **71** authored (**1136** rendered), **none** is dead. The prediction that this was
  "the likelier silent class" was **wrong**, and it is worth saying so plainly rather than quietly
  dropping it: heading text does change often, but the book's cross-references are overwhelmingly
  whole-chapter links (`knobs.md`), not deep links into a heading. The zero is negative-controlled —
  four planted anchor/file defects were all caught — so it is a measured zero, not a vacuous one.
- ~~**Does `print.html` double-count?**~~ **ANSWERED by `.1`: yes, exactly ×2 per chapter site.**
  The decision is to **report sites, and say so** — the single authored site at `recipes.md:857`
  surfaces as 2 rendered occurrences. Both numbers are recorded above so neither reading is hidden.
  A second, sharper `print.html` fact fell out: it carries its **own anchor namespace** (cross-chapter
  `knobs.html#x` is rewritten to bare `#x`), so anchors must be resolved per-file.
- **Can a doctrine check reach this at all?** Still open — `.3`'s question, and `.1` has made it
  **harder, not easier**. The honest tooling obstacle stands (a rendered check needs `mdbook`, not
  vendored, so it would skip on a fresh clone — the failure `USER-GUIDE-CLI-TABLE-SHADOW.3`
  rejected for `anvil --help`). `.1` adds a second obstacle: the sole real defect is an **escape**,
  not a missing file — `../../USER_GUIDE.md` **exists**. So the obvious portable check ("does the
  link target exist?") **passes the one link the tree was opened for**, i.e. it is vacuous against
  its own subject. A source-level check must model the `.md`→`.html` rewrite *and* test whether the
  rewritten path still lands inside `book-out`. "No mechanism, and here is why" remains legitimate
  under `DOCTRINE_ENFORCEMENT.md` §9.
- **New, raised by `.1`:** should the subject be *links* or *link **targets***? The one defect is a
  link whose target is a **repo-root live doc**. `.2` may find the durable repair is a rule about
  crossing the `book/src` boundary at all, in which case the checkable predicate is much simpler
  than a link checker. Do not settle this inside `.2`; note it for `.3`.

## Blockers

- None.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-08-01` | registration | `found by USER-GUIDE-CLI-TABLE-SHADOW.4 nearly shipping a dead pointer; rendered-HTML link scan at ccfbc23 over book/book-out/*.html: 665 local hrefs, 2 dead (recipes.md:857 and its print.html copy), after .4 fixed its own two; mdbook build exits 0 on all of them` | `registered` (docs-only; DUT byte-identical) |
| `2026-08-01` | `.1` | `mdbook build book (exit 0); TWO derivations. RENDERED book/book-out/**/*.html, 33 files: 1782 href occurrences = 1456 <a> + 325 <link> + 1 <base>; of the <a>: 16 external, 1101 in-page anchors, 339 file refs (35 with #fragment); anchor-class denominator 1136. AUTHORED book/src/*.md: 229 links (228 inline + 1 refdef), 7 external, 35 in-page, 187 file refs (36 with #fragment); anchor-class denominator 71. Extractor recall audited: grep -o '](' = 228 vs 228 captured; 0 images, 0 raw <a href>, 0 autolinks, 5 '][' triaged (2 refstyle -> the one external refdef, 3 array-index syntax).` | **CLASS A = 2 occurrences / 1 site** (`book/src/recipes.md:857`, surfacing in `recipes.html` + `print.html`); **CLASS B = 0 of 1136** |
| `2026-08-01` | `.1` | `negative control: 4 defects + 2 live controls appended to a copy-aside book/src/faq.md, rebuilt, scanned, restored, baseline re-confirmed. Probes: dead intra-book file, dead in-page anchor, dead cross-chapter anchor, repo-root escape.` | `all 4 caught by both derivations; live control knobs.md correctly clean; ir.md#node flagged and CONFIRMED a true negative (ir.html has no id="node"); mdbook build STILL exit 0 with all 4 present; git clean after restore` |
| `2026-08-01` | `.1` | `earlier 665 reconciled, not waved through: 339 <a> file refs + 326 <link>/<base> chrome = 665 exactly; that denominator excluded all 1101 in-page anchors` | `prior scan reproduced; its blind spot identified` |
| `2026-08-01` | `.1` | `scripts/check_doctrines.sh; knowledge-map/scripts/check_knowledge_map.sh` | `10/10 doctrines PASS; knowledge map in sync (119 -> 120 facts)`. Docs-only ⇒ DUT byte-identical; no `src/` change, so `cargo` gates are not the discriminator for this leaf (decision `0003`). |
| `2026-08-01` | `.2` | `convention census before choosing: every repo-root live-doc filename occurrence in book/src/*.md — 33 total = 31 plain-backtick + 2 from the single markdown link (its TEXT and its TARGET both name the file). 31 of 32 distinct references already used the chosen form.` | `the repair form was already the de-facto convention; recorded as 0046 rather than invented` |
| `2026-08-01` | `.2` | `after the one-line repair: mdbook build book (exit 0), then BOTH derivations re-run` | **`0` dead-file + `0` dead-anchor**, rendered and authored alike (was 2 + 0). Census now **32/32** backticked, **0** markdown links to a repo-root file. `0046`'s reverify one-liner returns **no matches**. |
| `2026-08-01` | `.2` | `scripts/check_doctrines.sh; knowledge-map/scripts/check_knowledge_map.sh; cargo check --all-targets` | `10/10 doctrines PASS` — **after one true fire**: the first staged run failed `KNOWLEDGE-MAP`, because `docs/decisions/*.md` carry `answers:` frontmatter and so are **fact sources**; adding `0046` desynchronised the derived map (120 → **121** facts). Regenerated, green. `cargo check` clean; docs+book only ⇒ DUT byte-identical. |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| registration | `USER-GUIDE-CLI-TABLE-SHADOW.4 — backfill the landed commit hash` | Tree registered alongside the hash backfill; no leaf executed yet. |
| `.1` | `5e3e9a0` — `BOOK-LINK-INTEGRITY.1 — measure both dead-link classes; anchor class is empty` | Docs-only. Adds the fact card `docs/knowledge/mdbook-md-to-html-rewrite-trap.md` (map 119 → 120). No book edit. |
| `.1` | `8ff64cd` — `BOOK-LINK-INTEGRITY.1 — backfill the landed commit hash` | Hash-only follow-up. |
| `.2` | `BOOK-LINK-INTEGRITY.2 — the book names repo-root files, it does not link to them` | Adds decision `0046`; repairs `book/src/recipes.md:857` (one line). Rendered book reaches **0** dead local links. |

## Changelog

- `2026-08-01` (`.2`): **The rendered book now has zero dead local links.** Decision
  [`0046`](../decisions/0046-book-never-links-outside-book-src.md) recorded *before* the edit:
  a book chapter **names** a repo-root file in backticks, with any section named in parentheses
  as prose, and never links to one. Decided by **census** — 31 of 32 existing references already
  did it — rather than by preference, so the leaf codified a convention instead of imposing one.
  The absolute-URL candidate is refuted by `book/book.toml`'s own empty `git-repository-url`; a
  fourth candidate (`{{#include}}`) was evaluated and rejected on four counts. Payload: one line
  in `book/src/recipes.md`. **The rule left behind is more valuable than the fix**: *no markdown
  link in `book/src` may escape `book/src`* is greppable and needs no `mdbook`, which is exactly
  the portable predicate `.3` was blocked on.
- `2026-08-01` (`.1`): **Both classes measured; the anchor class is empty.** Two independent
  derivations — rendered (**1782** hrefs over 33 files) and authored (**229** links) — agree on
  **one** offending site, `book/src/recipes.md:857`, surfacing as **2** rendered occurrences. The
  tree's central prediction, that dead **anchors** were the likelier silent half, is **refuted**:
  **0** dead of an anchor-class denominator of **71** authored / **1136** rendered, and the zero
  is negative-controlled by four planted defects that were all caught. Two instrument traps were
  hit and recorded (`<base href="/">` is not a link; `print.html` has its own anchor namespace),
  and the registration scan's `665` was **reconciled** rather than discarded — it is `<a>` file
  refs **plus `<link>`/`<base>` chrome**, and it excluded all 1101 in-page anchors, which is
  exactly why the anchor class had never been measured. Sharpened for `.3`: the sole defect is an
  **escape**, not a missing file — `../../USER_GUIDE.md` exists — so the obvious portable check
  ("does the target exist?") is **vacuous against this tree's own subject**.
- `2026-08-01`: Created. Found while `USER-GUIDE-CLI-TABLE-SHADOW.4` replaced the book's deleted
  CLI-flag copy with a pointer at `USER_GUIDE.md` — and the pointer, written in the form the book
  already used at `recipes.md:857`, rendered **dead**. mdBook rewrites `.md` → `.html`, so a link
  to a repo-root file becomes `../../USER_GUIDE.html`, which does not exist; it works on GitHub
  and fails in the rendered book, which is the surface the owner reviews. `mdbook build` exits
  `0`. `.4` fixed its own two sites (6 dead → 2) and registered the class rather than absorbing
  it, because the **anchor** half is still unmeasured and nothing mechanical would catch a
  recurrence.
