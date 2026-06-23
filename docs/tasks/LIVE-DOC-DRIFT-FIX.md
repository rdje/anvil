# LIVE-DOC-DRIFT-FIX: close residual ROADMAP/TOOLBOX live-doc drift

## Metadata

- Tree ID: `LIVE-DOC-DRIFT-FIX`
- Status: `done` (`.1` ROADMAP ninth-surface frontier + TOOLBOX schema/analyze currency, `.2` mdBook drift; tree closed `2026-06-24`)
- Roadmap lane: `Workflow / live-doc hygiene`
- Created: `2026-06-24`
- Last updated: `2026-06-24` (**`.2` landed — tree closed.** A dedicated mdBook deep-read (the bootstrap "read the mdBook before code" gate) found six stale spots in `book/src/`, all pure-docs: `api-reference.md` stated the current introspection schema as `1.14` (→ `1.22`, with the MINOR-bump changelog through `node_readers`) and listed "9 tools" omitting `coverage` (→ 10 tools); `api-resources-prompts.md` listed only 4 of the 9 `analyze` queries in the `analysis/<query>` resource row (→ all nine); `synthesizability.md` claimed "No tasks or functions … only pure `function` if ever used" though `task_emit`/`multi_output_task` ship side-effect-free combinational tasks (→ corrected, cross-linked); `structured-emission.md`'s intro listed 8 surfaces (→ nine, adding the wider-lane part-select); `introduction.md`'s "What you'll find" omitted the Structural Rules chapter (→ added). `mdbook build` clean. Docs-only ⇒ DUT byte-identical. Prior — **`.1` landed**: bootstrap drift audit fixed three top-level-doc spots — `ROADMAP.md:409` `Frontier → .19 (impl)` (ninth surface `casez_mux_if` is delivered end-to-end `.19a`→`.19b.3`, code `src/ir/casez_mux_if_emit.rs`, schema `1.22`) → delivered/no-active-frontier; `TOOLBOX.md:36` schema `1.14` → `1.22`; `TOOLBOX.md:37` analyze query list 4 → 9. Docs-only ⇒ DUT byte-identical.)
- Owner: repo-local workflow

## Goal

Close the residual live-doc drift surfaced during the session-bootstrap pass: the
ROADMAP/codebase/mdBook must stay locked together with no drift (owner directive).
The drift the prior `LIVE-DOC-HYGIENE-BACKFILL` tree did not reach (it scoped one
mdBook schema chapter + the `SEMANTIC-INTROSPECTION-EXPANSION` task-tree logs) fell in
two buckets: top-level docs (`ROADMAP.md` + `TOOLBOX.md`, `.1`) and a second sweep of
`book/src/` chapters (`.2`) that a dedicated mdBook deep-read surfaced.

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
- `book/src/` carries no remaining stale "current schema" claim, tool-count
  miscount, 4-of-9 `analyze` query list, "no tasks/functions" claim, or
  structured-surface undercount; `mdbook build book` is clean and
  `tests/book_examples.rs` stays 3/3 (no runnable bash blocks changed).
- `bash scripts/check_doctrines.sh` green (docs commit ⇒ code-scoped checks exempt);
  each leaf committed through `COMMIT.md` with the leaf id in the subject.

## Task Tree

- ID: `LIVE-DOC-DRIFT-FIX`
  Status: `done`
  Goal: `Close the residual live-doc drift found at session bootstrap that LIVE-DOC-HYGIENE-BACKFILL did not reach: top-level docs (ROADMAP ninth-surface stale frontier; TOOLBOX stale current-schema 1.14 + 4-of-9 analyze query list) in .1, and book/src/ chapters (stale schema/tool-count/query-list/tasks-claim/surface-count/chapter-list) in .2. Docs-only / DUT byte-identical.`
  Children: `LIVE-DOC-DRIFT-FIX.1`, `LIVE-DOC-DRIFT-FIX.2`
  Result: `Done — both children done. .1: ROADMAP.md ninth-surface paragraph records casez_mux_if delivered end-to-end (.19a→.19b.3) + no-active-frontier; TOOLBOX.md current schema 1.14 → 1.22 + analyze query table 4 → 9. .2: six book/src/ chapters brought current (api-reference schema 1.14 → 1.22 + tools 9 → 10; api-resources-prompts analyze resource 4 → 9 queries; synthesizability tasks-claim corrected; structured-emission intro 8 → 9 surfaces; introduction adds the Structural Rules chapter). mdbook build clean. Docs-only ⇒ DUT byte-identical.`

- ID: `LIVE-DOC-DRIFT-FIX.1`
  Status: `done`
  Goal: `Fix the three stale spots: ROADMAP.md:409 ("Frontier → .19 (impl)" → ninth surface delivered end-to-end + no-active-frontier); TOOLBOX.md:36 ("schema 1.14" → "schema 1.22"); TOOLBOX.md:37 (analyze query list 4 → 9, adding flop_dependencies / memory_provenance / fsm_provenance / node_drivers / node_readers). Docs-only / DUT byte-identical; no other content changed.`
  Acceptance: `ROADMAP no longer lists .19 as the open frontier and states nine structured surfaces delivered + no-active-frontier; TOOLBOX shows schema 1.22 and all nine analyze queries; grep shows no remaining present-tense "schema 1.14" current claim or 4-query analyze list in TOOLBOX; bash scripts/check_doctrines.sh green; committed via COMMIT.md.`
  Result: `Done. ROADMAP.md: the casez/ninth-surface paragraph's "Frontier → .19 (impl, pre-split .19a/.19b)" sentence replaced with the delivered-end-to-end record (.19a design-detail + .19b.1 live src/ir/casez_mux_if_emit.rs + .19b.2a metric @ schema 1.17 + .19b.2b tool_matrix --casez-mux-if-gate banked /tmp/anvil-casez-mux-if-gate-r1 + .19b.3 docs) and "Nine structured surfaces delivered end-to-end; the lane returns to a no-active-frontier boundary". TOOLBOX.md: the --introspect row "(schema 1.14)" → "(schema 1.22)"; the MCP analyze row extended from the original four queries to all nine (added flop_dependencies, memory_provenance, fsm_provenance, node_drivers, node_readers with one-line descriptions), matching USER_GUIDE/README/AGENT_INTROSPECTION_SCHEMA. Verified the book + AGENT_INTROSPECTION_SCHEMA.md + USER_GUIDE were already current (schema 1.22, nine queries) so no further edits needed. No src/ touched ⇒ DUT byte-identical.`
  Verification: `grep over TOOLBOX.md shows "(schema 1.22)" and the nine-query analyze row, no remaining "(schema 1.14)" / 4-query list; ROADMAP.md shows no "Frontier → .19" and the "Nine structured surfaces delivered end-to-end" + "no-active-frontier" record; bash scripts/check_doctrines.sh green (4/4, docs commit ⇒ code-scoped checks exempt); DUT byte-identical (no src/ touched).`
  Commit: `d4fc14e`

- ID: `LIVE-DOC-DRIFT-FIX.2`
  Status: `done`
  Goal: `Fix the six mdBook drift spots surfaced by the bootstrap mdBook deep-read: book/src/api-reference.md (current schema 1.14 → 1.22 + MINOR-bump changelog through node_readers; "9 tools" → "10 tools" adding coverage); book/src/api-resources-prompts.md (analysis/<query> resource 4 → 9 analyze queries); book/src/synthesizability.md ("only pure function if ever used" → corrected, tasks/functions are behaviour-preserving emit-projections, cross-linked); book/src/structured-emission.md (intro 8 → 9 surfaces, adding the wider-lane part-select); book/src/introduction.md ("What you'll find" Correctness Guarantees parenthetical adds Structural Rules). Docs-only / DUT byte-identical; mdbook build clean; no runnable bash blocks changed.`
  Acceptance: `grep shows no remaining "currently **1.14**" / "the 9 tools" in api-reference.md, the analysis resource row lists nine queries, no "only pure function if ever used" in synthesizability.md, the structured-emission intro says nine surfaces, the introduction lists Structural Rules; mdbook build book clean; bash scripts/check_doctrines.sh green; committed via COMMIT.md.`
  Result: `Done. Six book/src/ chapters edited: api-reference.md (the schema_version field "currently 1.14" → "1.22" with the MINOR-bump changelog rewritten through 1.15–1.17 mux/case/casez counts + 1.18–1.22 flop_dependencies/memory_provenance/fsm_provenance/node_drivers/node_readers query sections + the §7 changelog cross-link; the reference-pages "9 tools" row → "10 tools" with coverage added in canonical order); api-resources-prompts.md (the analysis/<query> resource row 4 → all nine analyze queries); synthesizability.md (the "No tasks or functions … only pure function if ever used (Phase 4+)" bullet rewritten to state tasks/functions appear only as behaviour-preserving side-effect-free emit-projections, cross-linked to structured-emission.md); structured-emission.md (the intro surface list 8 → nine, inserting "a wider-lane generate for part-select"); introduction.md (the Correctness Guarantees parenthetical adds "Structural Rules"). mdbook build clean. No src/ touched ⇒ DUT byte-identical.`
  Verification: `grep over book/src finds none of: "currently **\`1.14\`**", "the 9 tools", "only pure \`function\` if ever", an analysis row without node_readers, a structured-emission intro without "across nine surfaces", an introduction Correctness-Guarantees parenthetical without "Structural Rules". mdbook build book clean; bash scripts/check_doctrines.sh green (4/4, docs commit ⇒ code-scoped checks exempt); DUT byte-identical (no src/ touched, tests/book_examples.rs unaffected — no runnable bash blocks changed).`
  Commit: `this LIVE-DOC-DRIFT-FIX.2 commit`

## Current Frontier

**Tree complete (closed `2026-06-24`).** Both leaves done. No remaining frontier.

| Order | Leaf | Status | Why |
| --- | --- | --- | --- |
| 1 | `LIVE-DOC-DRIFT-FIX.1` | `done` | ROADMAP ninth-surface frontier corrected; TOOLBOX schema `1.22` + nine-query `analyze` list. |
| 2 | `LIVE-DOC-DRIFT-FIX.2` | `done` | Six `book/src/` chapters brought current (schema `1.22`, 10 tools, nine-query analyze resource, tasks claim, nine surfaces, Structural Rules chapter); mdBook builds. |

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
| `2026-06-24` | `LIVE-DOC-DRIFT-FIX.2` | Six `book/src/` chapters edited (api-reference schema `1.14 → 1.22` + tools `9 → 10`; api-resources-prompts analyze resource `4 → 9`; synthesizability tasks-claim; structured-emission intro `8 → 9` surfaces; introduction `+ Structural Rules`). `mdbook build book` clean; post-fix `book/src` greps find no remaining stale spot; `bash scripts/check_doctrines.sh` green (4/4, docs commit ⇒ code-scoped checks exempt). DUT byte-identical (no `src/`; no runnable bash blocks changed ⇒ `tests/book_examples.rs` unaffected). | `done` |
| `2026-06-24` | `LIVE-DOC-DRIFT-FIX.1` | `ROADMAP.md` ninth-surface paragraph corrected (delivered end-to-end + no-active-frontier; no remaining `Frontier → .19`); `TOOLBOX.md` `--introspect` row `1.14 → 1.22` and `analyze` row extended 4 → 9 queries. Cross-checked `USER_GUIDE.md` / `README.md` / `docs/AGENT_INTROSPECTION_SCHEMA.md` already current (schema `1.22`, nine queries). `bash scripts/check_doctrines.sh` green (4/4, docs commit ⇒ code-scoped checks exempt). DUT byte-identical (no `src/` touched). | `done` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `LIVE-DOC-DRIFT-FIX.2` | `LIVE-DOC-DRIFT-FIX.2 — mdBook schema/tool-count/query-list/surface-count drift` | Six `book/src/` chapters brought current; `mdbook build` clean. Closes the tree. Docs-only ⇒ DUT byte-identical. |
| `LIVE-DOC-DRIFT-FIX.1` | `LIVE-DOC-DRIFT-FIX.1 — ROADMAP ninth-surface frontier + TOOLBOX schema/analyze currency` (`d4fc14e`) | ROADMAP `.19` frontier → delivered/no-active-frontier; TOOLBOX schema `1.22` + nine-query `analyze` list. Docs-only ⇒ DUT byte-identical. |

## Changelog

- `2026-06-24`: Created the tree and landed `.1` (ROADMAP ninth-surface frontier + TOOLBOX
  schema/analyze currency).
- `2026-06-24`: Landed `.2` (six `book/src/` mdBook chapters brought current — schema `1.22`,
  10 tools, nine-query analyze resource, tasks claim, nine surfaces, Structural Rules chapter);
  tree closed.
