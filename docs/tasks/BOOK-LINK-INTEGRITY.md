# BOOK-LINK-INTEGRITY: `mdbook build` is not a link checker, and the book's links to repo-root files render dead

## Metadata

- Tree ID: `BOOK-LINK-INTEGRITY`
- Status: `active`
- Roadmap lane: Live-doc hygiene / book fidelity
- Created: `2026-08-01`
- Last updated: `2026-08-01` (registered from a `USER-GUIDE-CLI-TABLE-SHADOW.4` finding; frontier `.1`)
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
  Status: `pending`
  Goal: `Measure the rendered book for BOTH dead file targets and dead in-page anchors, name every offender, and state the instrument precisely enough to re-run. No repair in this leaf.`
  Acceptance: `Offenders NAMED not counted, for both classes. The anchor class must actually be measured, not assumed empty — .4's scan did not cover it. The match count is recorded alongside the finds (decision 0039), so recall is auditable. No book edit.`
  Verification: `pending`
  Commit: `pending`

- ID: `BOOK-LINK-INTEGRITY.2`
  Status: `pending`
  Goal: `Decide the repair form for a book -> repo-root reference, record it, then repair every site .1 found to match.`
  Acceptance: `The decision is recorded BEFORE any edit, with each candidate's failure mode stated: plain-backtick prose (works everywhere, not clickable), an absolute GitHub URL (clickable, breaks on a fork or a rename, and pins a host), a book-visible copy of the target (clickable, and a new shadow under decision 0033 — likely disqualifying). Whichever lands, book/src/recipes.md:857 is fixed and the rendered book has zero dead local links.`
  Verification: `pending`
  Commit: `pending`

- ID: `BOOK-LINK-INTEGRITY.3`
  Status: `pending`
  Goal: `Decide whether the class warrants a mechanism, given that the honest obstacle is tooling: link-checking the rendered book needs mdbook, which ANVIL does not vendor and a fresh clone may not have.`
  Acceptance: `The decision follows .2's repair form. If a check is written it obeys DOCTRINE_ENFORCEMENT.md section 4, is negative-controlled both ways, and states plainly whether it checks SOURCE (portable, approximate — it must model mdBook's .md -> .html rewrite) or RENDERED output (exact, but skipped wherever mdbook is absent, which is exactly where a backstop is needed). If no mechanism is warranted, say so and say why, per section 9's honest-limits discipline — do not register a check that silently skips.`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `BOOK-LINK-INTEGRITY.1` | `pending` | **Next.** Measure before repairing. The file-target class is known (**2** dead at `ccfbc23`); the **anchor** class is *unmeasured*, and an anchor that no longer matches a renamed heading is the silent half of this defect — `USER-GUIDE-CLI-TABLE-SHADOW.4` itself renamed two headings in `book/src/knobs.md`. |
| 2 | `BOOK-LINK-INTEGRITY.2` | `pending` | Decide the repair form first, then repair. `.4` chose plain-backtick prose under time pressure for two sites; that choice should be made deliberately for all of them. |
| 3 | `BOOK-LINK-INTEGRITY.3` | `pending` | The mechanism question, answered against what a check can actually read. |

## Decisions

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

- **Are there dead in-page anchors too?** Unmeasured. `.1`'s first job. This is the likelier silent
  class: heading text changes far more often than file layout, and nothing checks it.
- **Can a doctrine check reach this at all?** A rendered-output check is exact but needs `mdbook`,
  which is not vendored — so it would be *skipped* on a fresh clone, the failure mode
  `USER-GUIDE-CLI-TABLE-SHADOW.3` rejected for `anvil --help`. A source-level check is portable but
  must model mdBook's rewrite rules to be right. `.3` decides; "no mechanism, and here is why" is a
  legitimate outcome under `DOCTRINE_ENFORCEMENT.md` §9.
- **Does `print.html` double-count?** mdBook emits a concatenated `print.html` containing every
  chapter, so one bad link in a chapter appears twice in a naive scan. `.1` should decide whether
  to report sites or occurrences, and say which.

## Blockers

- None.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-08-01` | registration | `found by USER-GUIDE-CLI-TABLE-SHADOW.4 nearly shipping a dead pointer; rendered-HTML link scan at ccfbc23 over book/book-out/*.html: 665 local hrefs, 2 dead (recipes.md:857 and its print.html copy), after .4 fixed its own two; mdbook build exits 0 on all of them` | `registered` (docs-only; DUT byte-identical) |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| registration | `USER-GUIDE-CLI-TABLE-SHADOW.4 — backfill the landed commit hash` | Tree registered alongside the hash backfill; no leaf executed yet. |

## Changelog

- `2026-08-01`: Created. Found while `USER-GUIDE-CLI-TABLE-SHADOW.4` replaced the book's deleted
  CLI-flag copy with a pointer at `USER_GUIDE.md` — and the pointer, written in the form the book
  already used at `recipes.md:857`, rendered **dead**. mdBook rewrites `.md` → `.html`, so a link
  to a repo-root file becomes `../../USER_GUIDE.html`, which does not exist; it works on GitHub
  and fails in the rendered book, which is the surface the owner reviews. `mdbook build` exits
  `0`. `.4` fixed its own two sites (6 dead → 2) and registered the class rather than absorbing
  it, because the **anchor** half is still unmeasured and nothing mechanical would catch a
  recurrence.
