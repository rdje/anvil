# LIVE-DOC-DRIFT-FIX: close residual ROADMAP/TOOLBOX live-doc drift

## Metadata

- Tree ID: `LIVE-DOC-DRIFT-FIX`
- Status: `done` (`.1` ROADMAP ninth-surface frontier + TOOLBOX schema/analyze currency; tree closed `2026-06-24`)
- Roadmap lane: `Workflow / live-doc hygiene`
- Created: `2026-06-24`
- Last updated: `2026-06-24` (**`.1` landed — tree closed.** Bootstrap-pass drift audit found three stale spots that the prior `LIVE-DOC-HYGIENE-BACKFILL` sweep did not cover, all pure-docs: `ROADMAP.md:409` still read `Frontier → .19 (impl)` for the ninth structured surface (`casez_mux_if`) even though `.19a`→`.19b.3` all landed and the README/task-tree/code (`src/ir/casez_mux_if_emit.rs`, schema `1.22`) show it delivered end-to-end; `TOOLBOX.md:36` stated the current introspection schema as `1.14` (actual `1.22`); `TOOLBOX.md:37` listed only 4 of the 9 delivered `analyze` derived queries. Fixed all three. Docs-only ⇒ DUT byte-identical.)
- Owner: repo-local workflow

## Goal

Close the residual live-doc drift surfaced during the session-bootstrap pass: the
ROADMAP/codebase/mdBook must stay locked together with no drift (owner directive).
The drift was narrow and concentrated in two top-level live docs whose currency the
prior `LIVE-DOC-HYGIENE-BACKFILL` tree did not reach (it scoped the mdBook schema
chapter + the `SEMANTIC-INTROSPECTION-EXPANSION` task-tree logs).

## Non-Goals

- No source-code or generated-RTL behaviour change (docs-only ⇒ DUT byte-identical).
- No roadmap phase reclassification (every phase status is already correct).
- No rewrite of `/tmp` banked-evidence paths (external evidence, not project refs).
- No new feature, knob, or capability.

## Acceptance Criteria

- `ROADMAP.md` no longer claims `.19` is the open frontier of the ninth structured
  surface; it records the surface delivered end-to-end and the lane back at a
  no-active-frontier boundary, consistent with `docs/tasks/STRUCTURED-EMISSION-EXPANSION.md`
  and the README "Current CLI truth".
- `TOOLBOX.md` states the current introspection schema as `1.22` and enumerates all
  nine delivered `analyze` derived queries, consistent with `USER_GUIDE.md`,
  `README.md`, and `docs/AGENT_INTROSPECTION_SCHEMA.md`.
- `bash scripts/check_doctrines.sh` green (docs commit ⇒ code-scoped checks exempt);
  committed through `COMMIT.md` with the leaf id in the subject.

## Task Tree

- ID: `LIVE-DOC-DRIFT-FIX`
  Status: `done`
  Goal: `Close the residual ROADMAP + TOOLBOX live-doc drift found at session bootstrap (ROADMAP ninth-surface stale frontier; TOOLBOX stale current-schema 1.14 + 4-of-9 analyze query list), docs-only / DUT byte-identical.`
  Children: `LIVE-DOC-DRIFT-FIX.1`
  Result: `Done — .1 done. ROADMAP.md ninth-surface paragraph updated to record casez_mux_if delivered end-to-end (.19a→.19b.3) with the lane at a no-active-frontier boundary; TOOLBOX.md current schema 1.14 → 1.22 and the analyze query table extended from 4 to all 9 delivered queries. Docs-only ⇒ DUT byte-identical.`

- ID: `LIVE-DOC-DRIFT-FIX.1`
  Status: `done`
  Goal: `Fix the three stale spots: ROADMAP.md:409 ("Frontier → .19 (impl)" → ninth surface delivered end-to-end + no-active-frontier); TOOLBOX.md:36 ("schema 1.14" → "schema 1.22"); TOOLBOX.md:37 (analyze query list 4 → 9, adding flop_dependencies / memory_provenance / fsm_provenance / node_drivers / node_readers). Docs-only / DUT byte-identical; no other content changed.`
  Acceptance: `ROADMAP no longer lists .19 as the open frontier and states nine structured surfaces delivered + no-active-frontier; TOOLBOX shows schema 1.22 and all nine analyze queries; grep shows no remaining present-tense "schema 1.14" current claim or 4-query analyze list in TOOLBOX; bash scripts/check_doctrines.sh green; committed via COMMIT.md.`
  Result: `Done. ROADMAP.md: the casez/ninth-surface paragraph's "Frontier → .19 (impl, pre-split .19a/.19b)" sentence replaced with the delivered-end-to-end record (.19a design-detail + .19b.1 live src/ir/casez_mux_if_emit.rs + .19b.2a metric @ schema 1.17 + .19b.2b tool_matrix --casez-mux-if-gate banked /tmp/anvil-casez-mux-if-gate-r1 + .19b.3 docs) and "Nine structured surfaces delivered end-to-end; the lane returns to a no-active-frontier boundary". TOOLBOX.md: the --introspect row "(schema 1.14)" → "(schema 1.22)"; the MCP analyze row extended from the original four queries to all nine (added flop_dependencies, memory_provenance, fsm_provenance, node_drivers, node_readers with one-line descriptions), matching USER_GUIDE/README/AGENT_INTROSPECTION_SCHEMA. Verified the book + AGENT_INTROSPECTION_SCHEMA.md + USER_GUIDE were already current (schema 1.22, nine queries) so no further edits needed. No src/ touched ⇒ DUT byte-identical.`
  Verification: `grep over TOOLBOX.md shows "(schema 1.22)" and the nine-query analyze row, no remaining "(schema 1.14)" / 4-query list; ROADMAP.md shows no "Frontier → .19" and the "Nine structured surfaces delivered end-to-end" + "no-active-frontier" record; bash scripts/check_doctrines.sh green (4/4, docs commit ⇒ code-scoped checks exempt); DUT byte-identical (no src/ touched).`
  Commit: `this LIVE-DOC-DRIFT-FIX.1 commit`

## Current Frontier

**Tree complete (closed `2026-06-24`).** Single leaf `.1` done. No remaining frontier.

| Order | Leaf | Status | Why |
| --- | --- | --- | --- |
| 1 | `LIVE-DOC-DRIFT-FIX.1` | `done` | ROADMAP ninth-surface frontier corrected; TOOLBOX schema `1.22` + nine-query `analyze` list. |

## Decisions

- `2026-06-24`: Treat the residual ROADMAP/TOOLBOX drift found at bootstrap as a dedicated
  docs-only hygiene tree (the `LIVE-DOC-HYGIENE-BACKFILL` / `LIVE-DOC-PATH-HYGIENE`
  precedent) so the no-drift obligation is closed under task-tree ownership even though
  pure-docs edits are formally task-tree-exempt — maximizing session-continuity/auditability
  per the owner "track every activity" directive.

## Open Questions

- None.

## Blockers

- None.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-06-24` | `LIVE-DOC-DRIFT-FIX.1` | `ROADMAP.md` ninth-surface paragraph corrected (delivered end-to-end + no-active-frontier; no remaining `Frontier → .19`); `TOOLBOX.md` `--introspect` row `1.14 → 1.22` and `analyze` row extended 4 → 9 queries. Cross-checked `USER_GUIDE.md` / `README.md` / `docs/AGENT_INTROSPECTION_SCHEMA.md` / `book/src` already current (schema `1.22`, nine queries). `bash scripts/check_doctrines.sh` green (4/4, docs commit ⇒ code-scoped checks exempt). DUT byte-identical (no `src/` touched). | `done` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `LIVE-DOC-DRIFT-FIX.1` | `LIVE-DOC-DRIFT-FIX.1 — ROADMAP ninth-surface frontier + TOOLBOX schema/analyze currency` | ROADMAP `.19` frontier → delivered/no-active-frontier; TOOLBOX schema `1.22` + nine-query `analyze` list. Closes the tree. Docs-only ⇒ DUT byte-identical. |

## Changelog

- `2026-06-24`: Created the tree and landed `.1` (ROADMAP ninth-surface frontier + TOOLBOX
  schema/analyze currency); tree closed.
