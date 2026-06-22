# DOCTRINE-ENFORCEMENT-ADOPTION: Adopt the Portable Doctrine-Enforcement Architecture

## Metadata

- Tree ID: `DOCTRINE-ENFORCEMENT-ADOPTION`
- Status: `active`
- Roadmap lane: `Workflow / mechanical doctrine enforcement`
- Created: `2026-06-22`
- Last updated: `2026-06-22`
- Owner: repo-local workflow

## Goal

Adopt the fourth portable architecture — **doctrine enforcement** — in ANVIL,
so every written doctrine is paired with a deterministic check, all checks run
from one registry+driver, and the local git hook (E3) plus CI (E4) gate on it.
ANVIL already runs the other three portable architectures (task-trees,
`MEMORY_ARCHITECTURE.md`, the Knowledge Map) with their own checks wired
directly into `.githooks/pre-commit` and CI; this tree unifies them behind one
driver and mechanizes ANVIL's most load-bearing prose doctrines
(task-tree ownership of code changes; the mandatory live-doc evidence).

The standard being replayed is `DOCTRINE_ENFORCEMENT.md` (copied/adapted from
the donor project's portable kit). Reference: decision `0026`.

## Non-Goals

- No source code change. No generated RTL behaviour change. No CLI / config /
  user-facing feature change. Every leaf is a workflow-config / live-doc edit
  and is DUT byte-identical (`tests/snapshots.rs` untouched).
- Not a replacement for the three existing checks
  (`scripts/check_memory_architecture.sh`,
  `knowledge-map/scripts/check_knowledge_map.sh`); they are *registered*, not
  rewritten.
- Not an attempt to mechanize every doctrine in one pass. The first deployment
  registers the doctrines with clean, deterministic, fast, scope-aware checks
  (`MEMORY-ARCH`, `KNOWLEDGE-MAP`, `CODE-CHANGE-EVIDENCE`,
  `TASK-TREE-OWNERSHIP`). Heavier oracle doctrines (byte-identical snapshots,
  the downstream `tool_matrix` gates) stay where they run today (`cargo test` /
  local matrix) and are referenced, not duplicated into pre-commit.
- No over-claiming: local hooks are bypassable; CI is the real backstop
  (`DOCTRINE_ENFORCEMENT.md` §9). Evidence/ownership checks are structural,
  scope-aware co-staging proxies — honest about what they do and do not prove.

## Acceptance Criteria

- `DOCTRINE_ENFORCEMENT.md` (the standard) is at the repo root and decision
  `0026` records the adoption rationale.
- `scripts/check_doctrines.sh` (the registry+driver) runs every registered
  check, collects all results, meta-checks each check exists + is executable,
  prints a per-doctrine PASS/FAIL report, and exits nonzero iff any fail.
- `.githooks/pre-commit` and `.github/workflows/ci.yml` run the driver; the
  Knowledge Map derive-and-stage step is preserved.
- `TOOLBOX.md` catalogs **ANVIL's own diagnostic instruments** (trace,
  metrics, dump-config, introspect, the MCP `analyze`/`coverage`/`coverage_gaps`
  queries, `validate`/`minimize`/`hunt`, `divergence`, `--diff-sim`, the
  `tool_matrix` gates, `tests/snapshots.rs`, `scripts/ram_guard.sh`) plus the
  acceptance-checklist template a code change must satisfy, each box citing a
  named re-runnable oracle.
- The flagship doctrine — no code change without a task-tree leaf owning it — is
  mechanized as a scope-aware structural check.
- Every harness bootstrap pointer routes a fresh agent to
  `DOCTRINE_ENFORCEMENT.md` + `TOOLBOX.md` (in addition to `README.md` +
  `MEMORY_ARCHITECTURE.md`).
- Focused validation passes; the full driver and the full COMMIT.md gate are
  green; each completed leaf is committed through `COMMIT.md`.

## Task Tree

- ID: `DOCTRINE-ENFORCEMENT-ADOPTION`
  Status: `active`
  Goal: `Adopt portable architecture #4 (doctrine enforcement) in ANVIL.`
  Children: `DOCTRINE-ENFORCEMENT-ADOPTION.1`, `.2`, `.3`, `.4`, `.5`, `.6`

- ID: `DOCTRINE-ENFORCEMENT-ADOPTION.1`
  Status: `done`
  Goal: `Register this tree and land the DOCTRINE_ENFORCEMENT.md standard + decision 0026.`
  Acceptance: `DOCTRINE_ENFORCEMENT.md is at the repo root; decision 0026 records the adoption; the tree is indexed in docs/TASK_TREE.md; CHANGES.md + MEMORY.md reflect the new state. No scripts wired yet (nothing can break).`
  Verification: `check_memory_architecture.sh OK (0026 indexed; MEMORY.md 21 lines <= cap); gen_knowledge_map.sh + check_knowledge_map.sh OK (62 -> 63 facts / 565 -> 577 keys); no src/ touched.`
  Commit: `DOCTRINE-ENFORCEMENT-ADOPTION.1 — register tree + land the doctrine-enforcement standard + decision 0026`

- ID: `DOCTRINE-ENFORCEMENT-ADOPTION.2`
  Status: `done`
  Goal: `Add scripts/check_doctrines.sh (registry+driver) over the two existing structural checks; rewire .githooks/pre-commit and CI through the driver.`
  Acceptance: `The driver runs MEMORY-ARCH + KNOWLEDGE-MAP, meta-checks each check exists/executable, reports per-doctrine PASS/FAIL, exits nonzero on any fail. Pre-commit preserves the KM derive-and-stage step then runs the driver. CI runs the driver. All green.`
  Verification: `bash scripts/check_doctrines.sh → PASS MEMORY-ARCH + PASS KNOWLEDGE-MAP, exit 0; meta-check proven on an in-repo copy with a dangling registry line (BOGUS-DOCTRINE META-FAIL → REGISTRY ERROR, exit 1, temp removed); rewired pre-commit ran on the commit itself.`
  Commit: `DOCTRINE-ENFORCEMENT-ADOPTION.2 — registry+driver over the existing checks; rewire pre-commit + CI`

- ID: `DOCTRINE-ENFORCEMENT-ADOPTION.3`
  Status: `done`
  Goal: `Add TOOLBOX.md (ANVIL's own diagnostic toolbox + acceptance-checklist) and scripts/check_diagnosis_evidence.sh; register CODE-CHANGE-EVIDENCE.`
  Acceptance: `TOOLBOX.md catalogs ANVIL's diagnostic instruments + the acceptance checklist (each box cites a named re-runnable oracle). The evidence check is scope-aware (code staged ⇒ CHANGES.md + MEMORY.md co-staged; pure non-code commits exempt) and is registered + green.`
  Verification: `evidence check proven on 4 staged-set cases (non-code → exempt exit 0; code+CHANGES+MEMORY → pass; code-only → FAIL exit 1; Cargo.lock-only → FAIL exit 1); bash scripts/check_doctrines.sh → PASS x3 (MEMORY-ARCH, KNOWLEDGE-MAP, CODE-CHANGE-EVIDENCE), exit 0.`
  Commit: `DOCTRINE-ENFORCEMENT-ADOPTION.3 — TOOLBOX.md (ANVIL's own diagnostic tools) + CODE-CHANGE-EVIDENCE check`

- ID: `DOCTRINE-ENFORCEMENT-ADOPTION.4`
  Status: `done`
  Goal: `Add scripts/check_task_tree_ownership.sh mechanizing the flagship code-ownership doctrine; register TASK-TREE-OWNERSHIP.`
  Acceptance: `The check is scope-aware (code staged ⇒ an owning docs/tasks/*.md file co-staged; pure non-code commits exempt), deterministic, mutates nothing, and is registered + green.`
  Verification: `ownership check proven on 4 staged-set cases (non-code → exempt; code + owning task file → pass; code with no task file → FAIL; code + only TEMPLATE.md → FAIL); bash scripts/check_doctrines.sh → PASS x4, exit 0.`
  Commit: `DOCTRINE-ENFORCEMENT-ADOPTION.4 — mechanize the flagship TASK-TREE-OWNERSHIP doctrine`

- ID: `DOCTRINE-ENFORCEMENT-ADOPTION.5`
  Status: `done`
  Goal: `Discovery layer — route every harness bootstrap pointer to DOCTRINE_ENFORCEMENT.md + TOOLBOX.md; add GEMINI.md + .windsurfrules.`
  Acceptance: `CLAUDE.md, AGENTS.md, .cursorrules, .github/copilot-instructions.md, README.md (and new GEMINI.md, .windsurfrules) point at DOCTRINE_ENFORCEMENT.md + TOOLBOX.md while preserving the README.md + MEMORY_ARCHITECTURE.md references the memory-arch check requires. Driver still green.`
  Verification: `bash scripts/check_doctrines.sh → all six bootstrap pointers ok (incl. GEMINI.md + .windsurfrules now in BOOTSTRAP_FILES), PASS x4, exit 0.`
  Commit: `DOCTRINE-ENFORCEMENT-ADOPTION.5 — discovery layer: route every harness pointer to the doctrine kit`

- ID: `DOCTRINE-ENFORCEMENT-ADOPTION.6`
  Status: `pending`
  Goal: `Closeout — align CODEBASE_ANALYSIS.md / DEVELOPMENT_NOTES.md / the mdBook / a KM card; verify the full driver + COMMIT.md gate green; close the tree.`
  Acceptance: `The new enforcement layer is reflected in the live workspace analysis, the design-rationale log, the book's architecture/enforcement narrative, and a Knowledge Map fact card with a working reverify. Full driver + cargo check/clippy/fmt/test + mdbook green. Tree marked done.`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `DOCTRINE-ENFORCEMENT-ADOPTION.1` | `done` | Registered ownership; landed the standard + decision 0026. |
| 2 | `DOCTRINE-ENFORCEMENT-ADOPTION.2` | `done` | Driver + hook/CI rewired over MEMORY-ARCH + KNOWLEDGE-MAP. |
| 3 | `DOCTRINE-ENFORCEMENT-ADOPTION.3` | `done` | TOOLBOX (ANVIL's diagnostic tools) + `CODE-CHANGE-EVIDENCE` registered. |
| 4 | `DOCTRINE-ENFORCEMENT-ADOPTION.4` | `done` | `TASK-TREE-OWNERSHIP` registered; flagship doctrine mechanically gated. |
| 5 | `DOCTRINE-ENFORCEMENT-ADOPTION.5` | `done` | Six harness pointers route to the doctrine kit. |
| 6 | `DOCTRINE-ENFORCEMENT-ADOPTION.6` | `in_progress` | Closeout + live-doc/book/KM alignment; verify and close. |

## Decisions

- `2026-06-22`: Adopt the portable doctrine-enforcement architecture as
  decision `0026`. ANVIL already runs the other three portable architectures;
  this one generalizes the same E1→E4 defense-in-depth to *every* doctrine via
  one driver. The existing checks are registered, not rewritten.
- `2026-06-22`: The first registry deploys four doctrines —
  `MEMORY-ARCH` + `KNOWLEDGE-MAP` (existing structural checks),
  `CODE-CHANGE-EVIDENCE` + `TASK-TREE-OWNERSHIP` (new scope-aware structural
  checks). Heavier oracle doctrines (snapshots, `tool_matrix` gates) are
  referenced where they already run, not duplicated into pre-commit (the §4(7)
  fast-or-deferred rule).
- `2026-06-22` (owner steer): `TOOLBOX.md` catalogs **ANVIL's own tools** for
  pinpointing issues ANVIL may have, not a generic debug toolbox — trace /
  metrics / introspect / `analyze` / `coverage` / `validate` / `minimize` /
  `hunt` / `divergence` / `--diff-sim` / `tool_matrix` gates / snapshots /
  `ram_guard`.
- `2026-06-22`: New code-scoped checks are *scope-aware co-staging proxies* at
  pre-commit (the un-fakeable leg lives in CI / commit-msg per
  `DOCTRINE_ENFORCEMENT.md` §6.1/§9). Pure non-code commits are exempt so the
  doctrine never blocks legitimate docs/workflow work — including this tree's
  own leaves.

## Open Questions

- Whether to extend `.githooks/commit-msg` to assert the subject's leaf id
  *exists* in a `docs/tasks/*.md` file for code commits (the un-fakeable
  ownership leg). Tracked as a possible deepening in `.4`; does not block the
  frontier.

## Blockers

- None.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-06-22` | `DOCTRINE-ENFORCEMENT-ADOPTION.1` | `scripts/check_memory_architecture.sh`; `knowledge-map/scripts/gen_knowledge_map.sh` + `check_knowledge_map.sh`; `wc -l MEMORY.md` (21 <= 50) | passed; full `cargo test` not run because no source code changed (`0003-resource-safe-validation`) |
| `2026-06-22` | `DOCTRINE-ENFORCEMENT-ADOPTION.2` | `bash scripts/check_doctrines.sh` (PASS MEMORY-ARCH + PASS KNOWLEDGE-MAP, exit 0); meta-check on a dangling-entry copy (META-FAIL → REGISTRY ERROR, exit 1); rewired pre-commit ran on the commit | passed; no source code changed (`0003-resource-safe-validation`) |
| `2026-06-22` | `DOCTRINE-ENFORCEMENT-ADOPTION.3` | evidence check on 4 staged-set cases (exempt / pass / code-only FAIL / Cargo.lock FAIL); `bash scripts/check_doctrines.sh` (PASS x3, exit 0) | passed; no source code changed (`0003-resource-safe-validation`) |
| `2026-06-22` | `DOCTRINE-ENFORCEMENT-ADOPTION.4` | ownership check on 4 staged-set cases (exempt / pass / no-task FAIL / template-only FAIL); `bash scripts/check_doctrines.sh` (PASS x4, exit 0) | passed; no source code changed (`0003-resource-safe-validation`) |
| `2026-06-22` | `DOCTRINE-ENFORCEMENT-ADOPTION.5` | `bash scripts/check_doctrines.sh` (6 bootstrap pointers ok incl. GEMINI.md + .windsurfrules, PASS x4, exit 0) | passed; no source code changed (`0003-resource-safe-validation`) |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `DOCTRINE-ENFORCEMENT-ADOPTION.1` | `DOCTRINE-ENFORCEMENT-ADOPTION.1 — register tree + land the doctrine-enforcement standard + decision 0026` | `1b433d9`; standard + decision + tree, no check wired yet. |
| `DOCTRINE-ENFORCEMENT-ADOPTION.2` | `DOCTRINE-ENFORCEMENT-ADOPTION.2 — registry+driver over the existing checks; rewire pre-commit + CI` | `fbe6849`; driver + hook/CI; the existing checks are registered, not rewritten. |
| `DOCTRINE-ENFORCEMENT-ADOPTION.3` | `DOCTRINE-ENFORCEMENT-ADOPTION.3 — TOOLBOX.md (ANVIL's own diagnostic tools) + CODE-CHANGE-EVIDENCE check` | `fb5ecac`; TOOLBOX + scope-aware evidence check; driver now 3 doctrines. |
| `DOCTRINE-ENFORCEMENT-ADOPTION.4` | `DOCTRINE-ENFORCEMENT-ADOPTION.4 — mechanize the flagship TASK-TREE-OWNERSHIP doctrine` | `4a49681`; scope-aware ownership check; driver now 4 doctrines. |
| `DOCTRINE-ENFORCEMENT-ADOPTION.5` | `DOCTRINE-ENFORCEMENT-ADOPTION.5 — discovery layer: route every harness pointer to the doctrine kit` | `pending hash`; 6 harness pointers + README ramp-up items 18+19. |

## Changelog

- `2026-06-22`: Created task tree; completed `DOCTRINE-ENFORCEMENT-ADOPTION.1`
  (standard `DOCTRINE_ENFORCEMENT.md` + decision `0026` + tree registration);
  frontier advanced to `.2`.
- `2026-06-22`: Completed `DOCTRINE-ENFORCEMENT-ADOPTION.2` (registry+driver
  `scripts/check_doctrines.sh` over `MEMORY-ARCH` + `KNOWLEDGE-MAP`; rewired
  `.githooks/pre-commit` + CI); frontier advanced to `.3`.
- `2026-06-22`: Completed `DOCTRINE-ENFORCEMENT-ADOPTION.3` (`TOOLBOX.md` cataloguing
  ANVIL's own diagnostic instruments + the acceptance-checklist; scope-aware
  `scripts/check_diagnosis_evidence.sh` registered as `CODE-CHANGE-EVIDENCE`); frontier
  advanced to `.4`.
- `2026-06-22`: Completed `DOCTRINE-ENFORCEMENT-ADOPTION.4` (scope-aware
  `scripts/check_task_tree_ownership.sh` registered as `TASK-TREE-OWNERSHIP`; the
  flagship code-ownership doctrine is now mechanically gated); frontier advanced to `.5`.
- `2026-06-22`: Completed `DOCTRINE-ENFORCEMENT-ADOPTION.5` (six harness bootstrap
  pointers — incl. new `GEMINI.md` + `.windsurfrules` — route to `DOCTRINE_ENFORCEMENT.md`
  + `TOOLBOX.md`; `BOOTSTRAP_FILES` + README ramp-up updated); frontier advanced to `.6`.
