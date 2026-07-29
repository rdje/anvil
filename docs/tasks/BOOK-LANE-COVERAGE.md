# BOOK-LANE-COVERAGE: the mdBook has no chapters for the microdesign / frontend lanes

## Metadata

- Tree ID: `BOOK-LANE-COVERAGE`
- Status: `active`
- Roadmap lane: Documentation / book-sync (cross-cutting; no phase reopened)
- Created: `2026-07-29`
- Last updated: `2026-07-29`
- Owner: repo-local workflow (opened on an owner-relayed downstream-consumer report)

## Goal

Close the book-sync coverage gap: ANVIL ships **three** artifact lanes
(`--artifact dut|microdesign|frontend`), but the mdBook — the owner's
declared review surface — documents only the DUT lane with dedicated
chapters. The two non-DUT lanes exist in the book solely as passing
mentions (an `introduction.md` three-lane list, two `faq.md` bullets, and
the `lane` argument in the MCP/API reference chapters). The outcome: a
dedicated, user-facing book chapter per non-DUT lane, wired into
`SUMMARY.md`, with copy-paste-runnable examples that pass the
`tests/book_examples.rs` gate.

## Observation that opened this tree (`2026-07-29`, owner-relayed PGEN report)

PGEN — the downstream consumer the microdesign and frontend lanes were
built for — reported: *"ANVIL's mdBook documents the DUT lane thoroughly
but has no chapter for the microdesign or frontend lanes — the two built
for PGEN. Since the book is your review surface, two of three artifact
lanes are invisible there. That's ANVIL's to fix, not PGEN's."*

Audit of current book coverage (this session):

- `book/src/introduction.md` — names the three lanes (lines ~28–37); no
  links to lane documentation (none exists).
- `book/src/faq.md` — two one-line bullets for `--artifact microdesign` /
  `--artifact frontend`.
- `book/src/agent-mcp.md` + `api-*.md` — document the MCP `lane`
  argument (`generate`/`introspect`), the scoped knobs
  (`n_params`/`n_children`), and the `anvil://catalog/lanes` resource.
- **No chapter** describes what either lane emits, why (the oracle-backed
  expected-facts design), how to run it from the CLI, the manifest
  schema, or the parity gates that keep it honest — all of which ARE
  documented for the DUT lane and DO exist in code
  (`src/microdesign/`, `src/frontend/`, `src/umbrella/`,
  `tests/microdesign_parity.rs`, `tests/frontend_parity.rs`) and in
  `USER_GUIDE.md`.

This confirms the owner's suspicion (relayed the same day) that codebase
and book are not in lockstep: the drift is **coverage-shaped** — the
book-sync doctrine was honored where existing chapters touched changed
concepts, but landing two whole artifact lanes (Phases 7–8) never added
lane chapters.

Consumer-reported effectiveness context, recorded durably (KM card
`pgen-first-contact-parser-gap`): on first contact, ANVIL output exposed
a real, previously-unknown gap in PGEN's parser within minutes —
`case`/`endcase` absent from `grammars/rtl_frontend.ebnf`. That is the
product thesis (legal, unusual RTL exposing real consumer bugs) working
against a live consumer.

Discovered while grounding the chapters (this session): the non-DUT
`--out` filename bug — `--artifact frontend --out DIR` writes
`child_0.sv`/`child_0.json` (first `module` declaration) instead of the
top `acc_<seed>.sv` that `USER_GUIDE.md` documents. Owned and fixed by
the sibling tree `LANE-OUT-FILENAME` **before** `.3` documents `--out`
behavior, so the chapters describe corrected reality.

## Non-Goals

- No new generator capability; no lane behavior change (the sibling
  `LANE-OUT-FILENAME` fix is its own tree).
- No restructuring of existing DUT chapters.
- No naming of the external consumer in the public book text (the
  consumer-specific report stays in repo-internal records; the book gets
  the generic motivation).

## Acceptance Criteria

- A dedicated microdesign-lane chapter and a dedicated frontend-lane
  chapter exist under `book/src/`, wired into `SUMMARY.md` as a new
  "Artifact Lanes" part.
- Each chapter: what the lane emits (real pasted output), why it exists
  (oracle-backed expected-facts vs the DUT lane's acceptance-only bar),
  CLI usage incl. `--out` and the stdout/stderr split, the manifest
  fields, the reproducibility contract, the parity gates, and the MCP
  route (cross-linked to the API reference).
- Every `bash` block passes `tests/book_examples.rs` (runnable via
  `cargo run --release --`, or carries the skip sentinel).
- `mdbook build book` clean; cross-links from `introduction.md` to the
  new chapters.
- Live docs updated; each leaf lands through `COMMIT.md`.

## Task Tree

- ID: `BOOK-LANE-COVERAGE`
  Status: `active`
  Goal: dedicated book chapters for the microdesign + frontend lanes.
  Children: `BOOK-LANE-COVERAGE.1`, `BOOK-LANE-COVERAGE.2`,
        `BOOK-LANE-COVERAGE.3`

- ID: `BOOK-LANE-COVERAGE.1`
  Status: `done`
  Goal: register the tree before any edit (the
        `EVIDENCE-BANK-DURABILITY.1` precedent); record the PGEN report,
        the book-coverage audit, and the consumer-reported first-contact
        finding as a KM fact card (`pgen-first-contact-parser-gap`).
  Acceptance: tree file + `docs/TASK_TREE.md` row + KM card + live docs;
        docs-only ⇒ DUT byte-identical.
  Verification: book audit greps recorded above; KM regenerated with the
        new card; `scripts/check_doctrines.sh` 4/4 PASS.
  Commit: `BOOK-LANE-COVERAGE.1 — register: mdBook lane-chapter gap (PGEN report)`

- ID: `BOOK-LANE-COVERAGE.2`
  Status: `done`
  Goal: the microdesign-lane chapter (`book/src/microdesign-lane.md`) +
        the new "Artifact Lanes" `SUMMARY.md` part. Grounded in
        `src/microdesign/mod.rs`, `src/umbrella/mod.rs`,
        `USER_GUIDE.md`, and real seed-7 output captured this session.
  Acceptance: chapter per the criteria above; `mdbook build book` clean;
        `cargo test --test book_examples` green.
  Verification: `book/src/microdesign-lane.md` added + a new
        "Artifact Lanes" `SUMMARY.md` part (between Motif Catalogue and
        Reference) carrying the microdesign row only — the frontend row
        is deliberately deferred to `.3`, because an `SUMMARY.md` entry
        for a not-yet-written file makes mdBook generate an empty stub
        chapter. Every pasted artifact is REAL captured output (seed 7,
        `--lane-n-params 5`): the SV, the manifest fields, and the
        `--out` filenames (`mc_7.sv`/`mc_7.json`, post-`LANE-OUT-FILENAME.1`).
        Claims cross-checked against source: the `(((P4 % 8) + 8) % 8 + 1)`
        width idiom vs SV's sign-of-dividend `%` (`P4 = -1`); the
        `const_exprs` width rule (`bits_for` clamps negatives ⇒ width 1);
        and the parity-proof description against the real test names in
        `tests/microdesign_parity.rs` (agreement + per-category
        perturbation tests + the `#[ignore]`
        `parity_against_real_yosys_write_json` + the
        `yosys_scope_ignores_localparams_and_package_constants` scope
        pin). `mdbook build book` exit 0; `cargo test --test
        book_examples` 3/3 green (the harness EXECUTES both new bash
        blocks; `ran >= 40` assertion still satisfied).
  Commit: `BOOK-LANE-COVERAGE.2 — book: the microdesign artifact-lane chapter`

- ID: `BOOK-LANE-COVERAGE.3`
  Status: `pending`
  Goal: the frontend-lane chapter (`book/src/frontend-lane.md`) +
        `SUMMARY.md` row + cross-links from `introduction.md` (and
        `faq.md` if a pointer helps) to both new chapters. Documents the
        corrected `--out` naming (after `LANE-OUT-FILENAME`).
  Acceptance: chapter per the criteria above; `mdbook build book` clean;
        `cargo test --test book_examples` green; introduction links land.
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `BOOK-LANE-COVERAGE.3` | `pending` | The frontend chapter mirrors `.2`'s structure (the lane reuses the microdesign expression layer). It also adds the deferred `SUMMARY.md` frontend row and turns `.2`'s plain-text mention of the frontend lane into a link. Draft ready at `.cache/scratch/session-0cda78e8/frontend-lane.md`. |

## Decisions

- `2026-07-29`: Opened as a tracked tree on the owner-relayed PGEN
  report, not fixed silently — the registration-before-work precedent.
  Chapter placement: a new top-level "Artifact Lanes" part between
  "Motif Catalogue" and "Reference" (the lanes are user-facing artifact
  families with their own design story, not DUT motifs and not pure
  reference).
- `2026-07-29`: The external consumer is named in repo-internal records
  (this tree, the KM card) but not in the public book text.

## Open Questions

- None blocking. Whether `faq.md`'s two lane bullets should link the new
  chapters is decided in `.3` (cheap, likely yes).

## Blockers

- None. `.2`/`.3` are docs-only; the sibling `LANE-OUT-FILENAME` code
  fix is scheduled before `.2`.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-07-29` | `BOOK-LANE-COVERAGE.2` | `mdbook build book` exit 0; `cargo test --test book_examples` 3/3 (both new bash blocks executed by the harness, not merely rendered); every pasted SV/manifest/filename is real captured seed-7 output; `%`-semantics, `const_exprs` width rule, and parity-test names cross-checked against `src/`+`tests/` | `done` — microdesign chapter live; frontend row deferred to `.3` to avoid an mdBook stub |
| `2026-07-29` | `BOOK-LANE-COVERAGE.1` | Book-coverage audit greps (`grep -rln "microdesign\|frontend" book/src/*.md` → mentions only in introduction/faq/agent-mcp/api-* chapters; no lane chapter files exist); both lanes run live this session (seed 7 microdesign, seed 0 frontend) to ground the chapters; KM regenerated with `pgen-first-contact-parser-gap`; `scripts/check_doctrines.sh` 4/4 PASS | `done` — tree registered; docs-only ⇒ DUT byte-identical |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `BOOK-LANE-COVERAGE.1` | `BOOK-LANE-COVERAGE.1 — register: mdBook lane-chapter gap (PGEN report)` | docs-only |
| `BOOK-LANE-COVERAGE.2` | `BOOK-LANE-COVERAGE.2 — book: the microdesign artifact-lane chapter` | book-only ⇒ DUT byte-identical |

## Changelog

- `2026-07-29`: Created from the owner-relayed PGEN report; audit
  confirmed two of three artifact lanes have no book chapter.
