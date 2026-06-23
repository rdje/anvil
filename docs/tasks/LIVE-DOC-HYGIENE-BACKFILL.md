# LIVE-DOC-HYGIENE-BACKFILL: close accumulated live-doc / task-tree drift

## Metadata

- Tree ID: `LIVE-DOC-HYGIENE-BACKFILL`
- Status: `done` (`.1` book schema-drift + `.2` task-tree log-integrity backfill both **done**; tree closed `2026-06-24`)
- Roadmap lane: `Workflow / live-doc + task-tree hygiene`
- Created: `2026-06-24`
- Last updated: `2026-06-24` (**`.2` landed — tree closed.** `docs/tasks/SEMANTIC-INTROSPECTION-EXPANSION.md`: the `.8a`/`.8b.1`/`.8b.2` rows backfilled into the Verification Log + Commit Log (with the real hashes `cabe696`/`5499067`/`ce86560`); the stale Status labels corrected — `.5b.2` `pending → done`, `.9` + `.9b` `active → done`, and the `.3` frontier-table cell `active → done`; the file now has no leaf whose Status disagrees with its `done` Result. Prior — **`.1` landed**: `book/src/api-introspection.md` rewritten from its stale schema `1.14` to `1.22` + a missed `1.21 → 1.22` example in `book/src/api-tools.md`. Docs-only ⇒ DUT byte-identical.)
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
  Status: `done`
  Goal: `Close the live-doc + task-tree drift flagged across the SEMANTIC-INTROSPECTION-EXPANSION lane (book schema chapter + task-tree log integrity), docs-only / DUT byte-identical.`
  Children: `LIVE-DOC-HYGIENE-BACKFILL.1`, `LIVE-DOC-HYGIENE-BACKFILL.2`
  Result: `Done — both children done. .1 brought book/src/api-introspection.md from schema 1.14 to 1.22 (+ a missed example in api-tools.md); .2 backfilled the .8a/.8b.1/.8b.2 Verification/Commit-Log rows and corrected the stale .5b.2/.9/.9b/.3 Status labels in SEMANTIC-INTROSPECTION-EXPANSION.md. The book schema chapter is in sync and the SEMANTIC-INTROSPECTION-EXPANSION task file is internally self-consistent. Docs-only ⇒ DUT byte-identical.`

- ID: `LIVE-DOC-HYGIENE-BACKFILL.1`
  Status: `done`
  Goal: `Book schema-drift: bring book/src/api-introspection.md from its stale schema 1.14 to the current 1.22 (the JSON envelope example, the field table "currently 1.X", and the stability-contract section), enumerating the nine delivered analyze query kinds + the metric-count bumps and cross-linking the canonical §7 changelog in docs/AGENT_INTROSPECTION_SCHEMA.md; sweep the rest of the book for any other stale "current schema" / schema_version JSON example and fix it (caught + fixed a 1.21 → 1.22 output_support JSON example in book/src/api-tools.md missed by .10b.2). Docs-only / DUT byte-identical.`
  Acceptance: `book/src/api-introspection.md shows schema 1.22 in all three places + the nine-query/metric-count history; no remaining non-1.22 schema_version JSON example or stale "current schema" claim in book/src; mdbook build book clean; COMMIT.md precommit checks run.`
  Result: `Done. book/src/api-introspection.md: the envelope JSON example (1.14 → 1.22), the field-table "currently 1.14" → "1.22", and the "## schema_version stability contract" section ("The document schema is 1.14" → "1.22") — the MINOR-bump narrative rewritten to enumerate the nine analyze query kinds (output_support/input_reach/flop_reset_provenance/module_reachability + flop_dependencies 1.17→1.18 + memory_provenance 1.18→1.19 + fsm_provenance 1.19→1.20 + node_drivers 1.20→1.21 + node_readers 1.21→1.22) and the metric-count bumps (coverage_readout 1.11→1.12, num_mealy_fsm_modules 1.12→1.13, num_emitted_multi_output_tasks 1.13→1.14, procedural/case/casez mux counts 1.14→1.17), with the canonical §7 of docs/AGENT_INTROSPECTION_SCHEMA.md named as the full changelog. Sweep caught + fixed a stale "schema_version": "1.21" output_support JSON example at book/src/api-tools.md:165 (missed by .10b.2, which fixed the other two). Post-fix sweep: no remaining non-1.22 schema_version JSON example or "current schema" claim in book/src. mdbook build book clean. Docs-only ⇒ DUT byte-identical.`
  Verification: `grep over book/src finds no "schema_version": "1.<non-22>" JSON example and no stale "currently 1.X" / "document schema is 1.X" claim; mdbook build book clean; bash scripts/check_doctrines.sh green (docs commit ⇒ code-scoped checks exempt). DUT byte-identical (no src/ touched).`
  Commit: `this LIVE-DOC-HYGIENE-BACKFILL.1 commit`

- ID: `LIVE-DOC-HYGIENE-BACKFILL.2`
  Status: `done`
  Goal: `Task-tree log integrity in docs/tasks/SEMANTIC-INTROSPECTION-EXPANSION.md: backfill the missing .8a/.8b.1/.8b.2 rows in the Verification Log + Commit Log (reconstructed from each leaf's recorded Result/Verification/Commit + CHANGES.md), and correct the stale leaf Status labels .5b.2 (pending → done) and .9 (active → done) so the file is internally consistent (Result/Verification/Commit already say done). Docs-only / DUT byte-identical; no other tree content changed.`
  Acceptance: `The .8a/.8b.1/.8b.2 leaves appear in both the Verification Log and the Commit Log of SEMANTIC-INTROSPECTION-EXPANSION.md; .5b.2 + .9 Status read done; a grep of that file shows no leaf whose Status disagrees with its Result; bash scripts/check_doctrines.sh green; committed through COMMIT.md.`
  Result: `Done. docs/tasks/SEMANTIC-INTROSPECTION-EXPANSION.md: the .8a/.8b.1/.8b.2 leaves were backfilled into both the Verification Log and the Commit Log (each row reconstructed from the leaf's own recorded Result/Verification/Commit fields + the real commit hashes cabe696/.8a, 5499067/.8b.1, ce86560/.8b.2 from git log, marked "(backfilled by LIVE-DOC-HYGIENE-BACKFILL.2)"), inserted chronologically between the .9a and .7b.2 rows. Stale Status labels corrected: the .5b.2 leaf (pending → done), the .9 leaf (active → done), and — caught in the sweep, same stale-completed-container error class — the .9b leaf (active → done) and the .3 frontier-table summary cell (active → done). After the fix the file has no leaf whose Status disagrees with its done Result: the only remaining "active" labels are the legitimately-open tree Metadata Status and the root SEMANTIC-INTROSPECTION-EXPANSION node (the lane is at a no-frontier boundary but stays active by design — further query kinds are open-ended breadth). No src/ touched ⇒ DUT byte-identical.`
  Verification: `grep "Status: \`pending\`" docs/tasks/SEMANTIC-INTROSPECTION-EXPANSION.md ⇒ none; the 3 remaining "Status: \`active\`" are the tree Metadata + the root node (both legitimate) + nothing stale; the .8a/.8b.1/.8b.2 leaves appear in both logs (3 each); bash scripts/check_doctrines.sh green (docs commit ⇒ code-scoped checks exempt). DUT byte-identical (no src/ touched).`
  Commit: `this LIVE-DOC-HYGIENE-BACKFILL.2 commit`

## Current Frontier

**Tree complete (closed `2026-06-24`).** Both leaves done: `.1` (book schema chapter
`1.14 → 1.22`) and `.2` (task-tree log integrity in `SEMANTIC-INTROSPECTION-EXPANSION.md`).
No remaining frontier.

| Order | Leaf | Status | Why |
| --- | --- | --- | --- |
| 1 | `LIVE-DOC-HYGIENE-BACKFILL.1` | `done` | Book schema chapter brought to `1.22`; mdBook builds; book schema-example sweep clean. |
| 2 | `LIVE-DOC-HYGIENE-BACKFILL.2` | `done` | `.8*` log rows backfilled; stale `.5b.2`/`.9`/`.9b`/`.3` Status labels corrected; file self-consistent. |

## Decisions

- `2026-06-24`: Treat the accumulated `SEMANTIC-INTROSPECTION-EXPANSION` doc-drift (the
  stale book schema chapter + the missing `.8*` log rows + the stale completed-container
  Status labels) as a dedicated docs-only hygiene tree rather than bundling it into the
  per-query slices — preserving each query slice's clean leaf↔commit mapping while still
  closing the drift under task-tree ownership (the `LIVE-DOC-PATH-HYGIENE` precedent).
- `2026-06-24`: When backfilling historical log rows, reconstruct each from the leaf's own
  recorded `Result`/`Verification`/`Commit` fields + the real commit hash from `git log`,
  and mark it `(backfilled by …)` so the provenance of a late-added row is explicit (never
  fabricate a verification that was not run).

## Open Questions

- None.

## Blockers

- None.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-06-24` | `LIVE-DOC-HYGIENE-BACKFILL.2` | Backfilled the `.8a`/`.8b.1`/`.8b.2` rows into the Verification Log + Commit Log of `docs/tasks/SEMANTIC-INTROSPECTION-EXPANSION.md` (reconstructed from the leaf bodies + hashes `cabe696`/`5499067`/`ce86560`); corrected the stale Status labels `.5b.2` (`pending → done`), `.9` + `.9b` (`active → done`), and the `.3` frontier-table cell (`active → done`). `grep` confirms no `pending` leaf remains and the only `active` labels are the legitimately-open tree Metadata + root node; the `.8*` leaves now appear in both logs (3 each). `bash scripts/check_doctrines.sh` green (docs commit ⇒ code-scoped checks exempt). DUT byte-identical (no `src/` touched). | `done` |
| `2026-06-24` | `LIVE-DOC-HYGIENE-BACKFILL.1` | `book/src/api-introspection.md` rewritten from schema `1.14` to `1.22` (envelope example + field table + stability-contract section enumerating the nine `analyze` queries + the metric-count bumps + the canonical §7 cross-link); fixed a missed `1.21 → 1.22` example in `book/src/api-tools.md:165`. `book/src` schema-example sweep clean (all `1.22`); `mdbook build book` clean; `bash scripts/check_doctrines.sh` green. DUT byte-identical (no `src/` touched). | `done` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `LIVE-DOC-HYGIENE-BACKFILL.2` | `LIVE-DOC-HYGIENE-BACKFILL.2 — backfill SEMANTIC-INTROSPECTION task-tree logs` | `.8*` Verification/Commit-Log rows backfilled (hashes `cabe696`/`5499067`/`ce86560`); stale `.5b.2`/`.9`/`.9b`/`.3` Status labels corrected. Closes the tree. Docs-only ⇒ DUT byte-identical. |
| `LIVE-DOC-HYGIENE-BACKFILL.1` | `LIVE-DOC-HYGIENE-BACKFILL.1 — book schema chapter 1.14 -> 1.22` (`4092782`) | `book/src/api-introspection.md` + a missed `api-tools.md` example brought to schema `1.22`; new tree registered. Docs-only ⇒ DUT byte-identical. |

## Changelog

- `2026-06-24`: Created the tree and landed `.1` (book schema chapter `1.14 → 1.22`).
- `2026-06-24`: Landed `.2` (backfilled the `.8*` task-tree log rows + corrected the stale
  `.5b.2`/`.9`/`.9b`/`.3` Status labels); tree closed.