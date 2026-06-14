# LIVE-DOC-BOOK-ALIGNMENT: Realign mdBook With Delivered Motifs

## Metadata

- Tree ID: `LIVE-DOC-BOOK-ALIGNMENT`
- Status: `done`
- Roadmap lane: `Live docs / mdBook ↔ codebase alignment`
- Created: `2026-06-14`
- Last updated: `2026-06-14`
- Owner: repo-local workflow

## Goal

Remove user-facing mdBook drift where delivered Phase 5–9 capabilities
were still described as "future" / "not yet implemented", so the book
again reflects the codebase per the no-drift mandate.

## Non-Goals

- No code, test, IR, knob, or generated-output change.
- No new Motif-Catalogue chapters in this leaf (that broader book
  expansion, if pursued, is a separate task-tree-exempt follow-up).
- No rewrite of historical verification logs or roadmap promotions
  (all phases are already `done`).

## Acceptance Criteria

- No mdBook chapter labels a delivered capability "future" / "not yet
  implemented".
- `book/src/synthesizability.md` memories section reads as delivered
  (Phase 6, `memory_prob`).
- `book/src/ir.md` extensions section header no longer claims "not yet
  implemented"; the Phase 5 parameters subsection carries a Delivered
  tag.
- `book/src/faq.md` and `book/src/core-idea.md` no longer call the
  delivered FSM/memory motifs "future" (core-idea is a protected file;
  the tense-only edit is justified in `DEVELOPMENT_NOTES.md`).
- `CHANGES.md`, `MEMORY.md`, `DEVELOPMENT_NOTES.md` refreshed.
- Focused documentation checks pass; the byte-identical book-runnable
  contract is preserved (no runnable bash block touched).

## Task Tree

- ID: `LIVE-DOC-BOOK-ALIGNMENT`
  Status: `done`
  Goal: `Realign the mdBook with the delivered codebase.`
  Children: `LIVE-DOC-BOOK-ALIGNMENT.1`

- ID: `LIVE-DOC-BOOK-ALIGNMENT.1`
  Status: `done`
  Goal: `Correct mdBook chapters that label delivered Phase 5-9 motifs as future.`
  Acceptance: `synthesizability.md / ir.md / faq.md / core-idea.md no longer describe delivered memories, parameterization, or Phase 7-9 lanes as future; live docs refreshed; mdbook build + self-checks clean.`
  Verification: `mdbook build book`; residual-staleness grep; `scripts/check_memory_architecture.sh`; `knowledge-map/scripts/check_knowledge_map.sh`; `git diff --check`.
  Commit: `LIVE-DOC-BOOK-ALIGNMENT.1 - realign mdBook with delivered motifs`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `LIVE-DOC-BOOK-ALIGNMENT.1` | `done` | Completed and closed the tree. |

## Decisions

- `2026-06-14`: Scope this leaf to drift correction (false "future"
  labels) only. Adding dedicated Motif-Catalogue chapters for the
  delivered Phase 5/5b/6/7-9 motifs is a larger, separate effort and is
  not bundled here.
- `2026-06-14`: The `core-idea.md` edit is tense-only and preserves the
  load-bearing "extend the recursion, never iterate" decision verbatim;
  justified in `DEVELOPMENT_NOTES.md` per the COMMIT.md protected-file
  rule.

## Open Questions

- None.

## Blockers

- None.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-06-14` | `LIVE-DOC-BOOK-ALIGNMENT.1` | `mdbook build book`; residual-staleness grep; `scripts/check_memory_architecture.sh`; `knowledge-map/scripts/check_knowledge_map.sh`; `git diff --check` | passed (full `cargo test` intentionally skipped — no code changed; full-suite RAM risk per `docs/decisions/0003-resource-safe-validation.md`) |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `LIVE-DOC-BOOK-ALIGNMENT.1` | `LIVE-DOC-BOOK-ALIGNMENT.1 - realign mdBook with delivered motifs` | Pending hash; closes tree. |

## Changelog

- `2026-06-14`: Created, completed, and closed the mdBook alignment tree.
