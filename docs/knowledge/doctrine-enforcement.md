---
id: doctrine-enforcement
title: ANVIL's doctrines are mechanically enforced by scripts/check_doctrines.sh (the registry+driver)
answers:
  - "how are ANVIL's doctrines enforced"
  - "what is scripts/check_doctrines.sh"
  - "where is the doctrine registry and driver"
  - "how do I add a new enforced doctrine"
  - "what doctrines does the driver check"
  - "is task-tree ownership of code mechanically gated"
  - "what runs in the pre-commit hook and CI"
  - "what is TOOLBOX.md"
  - "where are ANVIL's own diagnostic tools listed"
  - "how do I run the doctrine checks"
  - "what is CODE-CHANGE-EVIDENCE / TASK-TREE-OWNERSHIP"
  - "what is the fourth portable architecture"
date: 2026-06-22
status: current
tags: [doctrine, enforcement, ci, hooks, task-tree, toolbox, process, portability]
reverify: bash scripts/check_doctrines.sh
evidence: DOCTRINE_ENFORCEMENT.md; scripts/check_doctrines.sh; scripts/check_diagnosis_evidence.sh; scripts/check_task_tree_ownership.sh; scripts/check_memory_architecture.sh; scripts/check_no_boot_volume_refs.sh; scripts/check_evidence_citations.sh; scripts/check_enumeration_parity.sh; knowledge-map/scripts/check_knowledge_map.sh; TOOLBOX.md; .githooks/pre-commit; .github/workflows/ci.yml; docs/decisions/0026-doctrine-enforcement-adoption.md; docs/decisions/0033-shadow-enumeration-classification.md; docs/tasks/DOCTRINE-ENFORCEMENT-ADOPTION.md
---

ANVIL enforces every load-bearing doctrine with a deterministic check run from
one registry+driver, `scripts/check_doctrines.sh` (the fourth portable
architecture; standard `DOCTRINE_ENFORCEMENT.md`, decision `0026`). The driver
collects all results, meta-checks each registered check exists+executable
(`REGISTRY ERROR` on a dangling entry), prints a per-doctrine PASS/FAIL report,
and exits nonzero on any breach. The live registry (kept in parity with the
driver by `ENUMERATION-PARITY` itself; no count is stated here because a number
beside a list is a second copy of it):

- <!--enum:doctrine-ids-->`MEMORY-ARCH` → `scripts/check_memory_architecture.sh` (structural).
- `KNOWLEDGE-MAP` → `knowledge-map/scripts/check_knowledge_map.sh` (structural).
- `CODE-CHANGE-EVIDENCE` → `scripts/check_diagnosis_evidence.sh` (evidence,
  scope-aware: a staged code change must co-stage `CHANGES.md` + `MEMORY.md`).
- `TASK-TREE-OWNERSHIP` → `scripts/check_task_tree_ownership.sh` (structural,
  scope-aware: a staged code change must co-stage an owning `docs/tasks/*.md`).
- `NO-BOOT-VOLUME-REFS` → `scripts/check_no_boot_volume_refs.sh` (structural:
  no tracked file points at the boot volume; `temp_dir()` confined to
  `src/paths.rs`; decision `0031`).
- `EVIDENCE-CITATIONS` → `scripts/check_evidence_citations.sh` (structural,
  fail-closed: every cited evidence bank is digest-backed or classified;
  decision `0030`).
- `ENUMERATION-PARITY` → `scripts/check_enumeration_parity.sh` (structural:
  every declared docs/script enumeration pair matches the set it mirrors —
  including this list; decision `0033`).
- `README-GROWTH` → `scripts/check_readme_growth.sh` (structural: `README.md`
  stays a landing page within its reviewed line **and** byte caps, with the
  project-owned `README_POLICY.md` beside it; decision `0036`, owner directive
  `2026-07-30`. Not scope-aware — landing-page size is a property of the
  tree, not of a change).
- `TABLE-RENDER-FIDELITY` → `scripts/check_markdown_tables.sh` (structural: no
  tracked `*.md` table row over-splits past its header, because GFM drops the
  excess cells and that content never reaches a rendered page —
  `OVERFLOW-DESTINATION-INSTRUMENTATION.6` measured **36** such rows dropping
  **57,283** characters. Escape-aware and fence-aware; deliberately silent on
  rows with FEWER cells, which the spec pads without losing anything).<!--/enum:doctrine-ids-->

The two code-scoped checks exempt pure docs / workflow commits (they govern only
`src/`/`tests/`/`examples/`/`build.rs`/`Cargo.toml`/`Cargo.lock`). `.githooks/pre-commit`
(E3) and `.github/workflows/ci.yml` (E4) both run the driver. They are structural
co-staging proxies; the un-fakeable oracle leg is the `cargo test` + `tool_matrix`
re-run at `COMMIT.md`/CI and the `commit-msg` leaf-id gate (`DOCTRINE_ENFORCEMENT.md`
§6.1/§9). Add a doctrine = write `scripts/check_<id>.sh` (the §4 contract) + one
registry line. ANVIL's own bug-pinpointing instruments + the acceptance-checklist a
code change must satisfy are in the companion `TOOLBOX.md`.
