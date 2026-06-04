# KNOWLEDGE-MAP-DOC: Knowledge Map Retrieval Layer Adoption

## Metadata

- Tree ID: `KNOWLEDGE-MAP-DOC`
- Status: `active`
- Roadmap lane: `Workflow / retrieval architecture`
- Created: `2026-06-05`
- Last updated: `2026-06-05`
- Owner: repo-local workflow

## Goal

Adopt the project-agnostic Knowledge Map bundle in ANVIL so durable facts
that have been logged once can be found by question-keyed lookup instead
of re-derived from source, runtime, or chat history.

## Non-Goals

- No source-code or generated-RTL behavior changes.
- No conversion of existing docs, task trees, or mdBook prose into cards.
- No import of donor-project facts.
- No full `cargo test`; focused workflow checks are sufficient for this
  docs/workflow adoption per `docs/decisions/0003-resource-safe-validation.md`.

## Acceptance Criteria

- `knowledge-map/` is present as the project-agnostic bundle and is
  discoverable from README/bootstrap surfaces.
- `KNOWLEDGE_MAP.md` is derived from fact front-matter and never
  hand-authored.
- Knowledge-map validation is wired into local hooks and CI.
- Existing ANVIL layer-C decisions that are high-value retrieval facts
  receive `answers:` front-matter in place; no donor-specific facts are
  imported.
- Focused validation passes and the tree closes through `COMMIT.md`.

## Task Tree

- ID: `KNOWLEDGE-MAP-DOC`
  Status: `active`
  Goal: `Install the Knowledge Map retrieval layer as an additive memory-architecture extension.`
  Children: `KNOWLEDGE-MAP-DOC.1`, `KNOWLEDGE-MAP-DOC.2`, `KNOWLEDGE-MAP-DOC.3`

- ID: `KNOWLEDGE-MAP-DOC.1`
  Status: `done`
  Goal: `Copy the project-agnostic knowledge-map bundle and add README/bootstrap discovery pointers.`
  Acceptance: `knowledge-map/ is present; README and bootstrap pointers mention KNOWLEDGE_MAP.md / knowledge-map; donor-project residue search is clean; focused checks pass.`
  Verification: `knowledge-map/ bundle present; README and AGENTS.md/CLAUDE.md/.cursorrules/.github/copilot-instructions.md point at knowledge-map/KNOWLEDGE_MAP_ARCHITECTURE.md; donor-project residue rg search clean; scripts/check_memory_architecture.sh clean; git diff --check clean; mdbook build book clean; cargo check --all-targets clean. Full cargo test intentionally skipped per resource-safe workflow-doc policy.`
  Commit: `KNOWLEDGE-MAP-DOC.1 — add Knowledge Map bundle`

- ID: `KNOWLEDGE-MAP-DOC.2`
  Status: `done`
  Goal: `Install functional generation/enforcement: create docs/knowledge, generate KNOWLEDGE_MAP.md, wire pre-commit and CI gates.`
  Acceptance: `knowledge-map/install.sh or equivalent setup has produced the fact dir and map; pre-commit regenerates/stages/checks the map; CI runs check_knowledge_map.sh; focused validation proves drift checks work.`
  Verification: `bash knowledge-map/install.sh created docs/knowledge/ and generated KNOWLEDGE_MAP.md; knowledge-map/scripts/check_knowledge_map.sh clean; .githooks/pre-commit runs memory architecture check then regenerates/stages/checks KNOWLEDGE_MAP.md; .github/workflows/ci.yml runs knowledge-map/scripts/check_knowledge_map.sh after the memory architecture check; scripts/check_memory_architecture.sh clean; git diff --check clean; mdbook build book clean; cargo check --all-targets clean. Full cargo test intentionally skipped per resource-safe workflow-doc policy.`
  Commit: `KNOWLEDGE-MAP-DOC.2 — install Knowledge Map enforcement`

- ID: `KNOWLEDGE-MAP-DOC.3`
  Status: `pending`
  Goal: `Seed ANVIL-specific retrieval keys for existing layer-C decisions and close the tree.`
  Acceptance: `Only ANVIL decision records are folded into the map by adding front-matter answers; generated KNOWLEDGE_MAP.md has question keys; final focused validation passes; tree closes.`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `KNOWLEDGE-MAP-DOC.3` | `pending` | Seed ANVIL-specific retrieval keys now that generation/enforcement exists. |

## Decisions

- `2026-06-05`: Treat the donor repository as a source of the
  project-agnostic bundle
  only. ANVIL must not import donor-project facts.
- `2026-06-05`: Seed the map lazily with ANVIL's existing layer-C
  decisions only; do not sweep or convert live docs/book/task trees.

## Open Questions

- None.

## Blockers

- None.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-06-05` | `KNOWLEDGE-MAP-DOC.1` | donor-project residue `rg`; `scripts/check_memory_architecture.sh`; `git diff --check`; `mdbook build book`; `cargo check --all-targets` | Done — focused checks clean; full cargo test intentionally skipped per resource-safe workflow-doc policy. |
| `2026-06-05` | `KNOWLEDGE-MAP-DOC.2` | `bash knowledge-map/install.sh`; `knowledge-map/scripts/check_knowledge_map.sh`; `.githooks/pre-commit`; CI wiring review; `scripts/check_memory_architecture.sh`; `git diff --check`; `mdbook build book`; `cargo check --all-targets` | Done — functional generation/enforcement clean; full cargo test intentionally skipped per resource-safe workflow-doc policy. |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `KNOWLEDGE-MAP-DOC.1` | `cf16846` — `KNOWLEDGE-MAP-DOC.1 — add Knowledge Map bundle` | Bundle and discovery pointers landed. |
| `KNOWLEDGE-MAP-DOC.2` | `KNOWLEDGE-MAP-DOC.2 — install Knowledge Map enforcement` | Hash can be backfilled in a later live-doc update per `COMMIT.md`. |
| `KNOWLEDGE-MAP-DOC.3` | `pending` | `pending` |

## Changelog

- `2026-06-05`: Created task tree and opened `KNOWLEDGE-MAP-DOC.1`.
- `2026-06-05`: Completed `KNOWLEDGE-MAP-DOC.1`; frontier moves to `.2`.
- `2026-06-05`: Completed `KNOWLEDGE-MAP-DOC.2`; frontier moves to `.3`.
