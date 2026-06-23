# LIVE-DOC-HYGIENE-BACKFILL: close accumulated live-doc / task-tree drift

## Metadata

- Tree ID: `LIVE-DOC-HYGIENE-BACKFILL`
- Status: `active` (`.1` book schema-drift **done**; `.2` task-tree log-integrity backfill pending)
- Roadmap lane: `Workflow / live-doc + task-tree hygiene`
- Created: `2026-06-24`
- Last updated: `2026-06-24` (**`.1` landed** — `book/src/api-introspection.md` was stale at schema `1.14` [predating the `.15`–`.22` bumps]; rewritten to `1.22` with the nine `analyze` queries + the metric-count bumps enumerated and the canonical §7 changelog cross-linked; also fixed a `1.21 → 1.22` JSON example in `book/src/api-tools.md` that `SEMANTIC-INTROSPECTION-EXPANSION.10b.2` missed. mdBook builds. Docs-only ⇒ DUT byte-identical. Frontier → `.2`.)
- Owner: repo-local workflow

## Goal

Close the live-doc and task-tree drift that accumulated across the
`SEMANTIC-INTROSPECTION-EXPANSION` query lane and was flagged for a dedicated
backfill (the resume pointer's "pre-existing doc-hygiene gaps", deliberately not
bundled into the `.8*`/`.9*`/`.10*` slices to keep each leaf↔commit mapping
clean). The mdBook is the user's only window into the project, so a stale schema
chapter is a real doctrine breach; the task-tree logs are the session-recovery
backbone, so a missing leaf↔commit row weakens continuity.

## Non-Goals

- No source-code or generated-RTL behaviour change (docs-only ⇒ DUT byte-identical).
- No roadmap phase reclassification.
- No rewrite of `/tmp` banked-evidence paths (external evidence, not project refs).
- No retroactive migration of closed `rN` slices.

## Acceptance Criteria

- `book/src/api-introspection.md` reflects the current introspection schema
  (`1.22`) and enumerates the delivered `analyze` query kinds; no stale "current
  schema" claim remains in the book.
- The `SEMANTIC-INTROSPECTION-EXPANSION` task tree's Verification Log + Commit Log
  carry the missing `.8a`/`.8b.1`/`.8b.2` rows, and the stale `.5b.2` (`pending`)
  + `.9` (`active`) Status labels are corrected to `done` so the file is
  self-consistent (Result/Verification/Commit all say done).
- `mdbook build book` passes; standard `COMMIT.md` precommit checks run; each leaf
  committed through `COMMIT.md` with its leaf id.

## Task Tree

- ID: `LIVE-DOC-HYGIENE-BACKFILL`
  Status: `active`
  Goal: `Close the live-doc + task-tree drift flagged across the SEMANTIC-INTROSPECTION-EXPANSION lane (book schema chapter + task-tree log integrity), docs-only / DUT byte-identical.`
  Children: `LIVE-DOC-HYGIENE-BACKFILL.1`, `LIVE-DOC-HYGIENE-BACKFILL.2`

- ID: `LIVE-DOC-HYGIENE-BACKFILL.1`
  Status: `done`
  Goal: `Book schema-drift: bring book/src/api-introspection.md from its stale schema 1.14 to the current 1.22 (the JSON envelope example, the field table "currently 1.X", and the stability-contract section), enumerating the nine delivered analyze query kinds + the metric-count bumps and cross-linking the canonical §7 changelog in docs/AGENT_INTROSPECTION_SCHEMA.md; sweep the rest of the book for any other stale "current schema" / schema_version JSON example and fix it (caught + fixed a 1.21 → 1.22 output_support JSON example in book/src/api-tools.md missed by .10b.2). Docs-only / DUT byte-identical.`
  Acceptance: `book/src/api-introspection.md shows schema 1.22 in all three places + the nine-query/metric-count history; no remaining non-1.22 schema_version JSON example or stale "current schema" claim in book/src; mdbook build book clean; COMMIT.md precommit checks run.`
  Result: `Done. book/src/api-introspection.md: the envelope JSON example (1.14 → 1.22), the field-table "currently 1.14" → "1.22", and the "## schema_version stability contract" section ("The document schema is 1.14" → "1.22") — the MINOR-bump narrative rewritten to enumerate the nine analyze query kinds (output_support/input_reach/flop_reset_provenance/module_reachability + flop_dependencies 1.17→1.18 + memory_provenance 1.18→1.19 + fsm_provenance 1.19→1.20 + node_drivers 1.20→1.21 + node_readers 1.21→1.22) and the metric-count bumps (coverage_readout 1.11→1.12, num_mealy_fsm_modules 1.12→1.13, num_emitted_multi_output_tasks 1.13→1.14, procedural/case/casez mux counts 1.14→1.17), with the canonical §7 of docs/AGENT_INTROSPECTION_SCHEMA.md named as the full changelog. Sweep caught + fixed a stale "schema_version": "1.21" output_support JSON example at book/src/api-tools.md:165 (missed by .10b.2, which fixed the other two). Post-fix sweep: no remaining non-1.22 schema_version JSON example or "current schema" claim in book/src. mdbook build book clean. Docs-only ⇒ DUT byte-identical.`
  Verification: `grep over book/src finds no "schema_version": "1.<non-22>" JSON example and no stale "currently 1.X" / "document schema is 1.X" claim; mdbook build book clean; bash scripts/check_doctrines.sh green (docs commit ⇒ code-scoped checks exempt). DUT byte-identical (no src/ touched).`
  Commit: `this LIVE-DOC-HYGIENE-BACKFILL.1 commit`

- ID: `LIVE-DOC-HYGIENE-BACKFILL.2`
  Status: `pending`
  Goal: `Task-tree log integrity in docs/tasks/SEMANTIC-INTROSPECTION-EXPANSION.md: backfill the missing .8a/.8b.1/.8b.2 rows in the Verification Log + Commit Log (reconstructed from each leaf's recorded Result/Verification/Commit + CHANGES.md), and correct the stale leaf Status labels .5b.2 (pending → done) and .9 (active → done) so the file is internally consistent (Result/Verification/Commit already say done). Docs-only / DUT byte-identical; no other tree content changed.`
  Acceptance: `The .8a/.8b.1/.8b.2 leaves appear in both the Verification Log and the Commit Log of SEMANTIC-INTROSPECTION-EXPANSION.md; .5b.2 + .9 Status read done; a grep of that file shows no leaf whose Status disagrees with its Result; bash scripts/check_doctrines.sh green; committed through COMMIT.md.`