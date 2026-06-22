---
id: doctrine-enforcement-adoption
title: ANVIL adopts the portable doctrine-enforcement architecture — every doctrine paired with a deterministic check, run from one registry+driver, gated by the git hook (E3) and CI (E4)
answers:
  - "how are ANVIL's doctrines mechanically enforced"
  - "what is the doctrine-enforcement architecture"
  - "where is the doctrine registry and driver"
  - "what does scripts/check_doctrines.sh do"
  - "how do I add a new enforced doctrine to ANVIL"
  - "what doctrines does ANVIL mechanically check"
  - "is task-tree ownership of code changes mechanically enforced"
  - "what is TOOLBOX.md for"
  - "what are ANVIL's own diagnostic tools"
  - "which checks run in the pre-commit hook and CI"
  - "what is the fourth portable architecture"
  - "how is CODE-CHANGE-EVIDENCE enforced"
date: 2026-06-22
status: accepted
tags: [process, doctrine, enforcement, ci, hooks, task-tree, memory-architecture, knowledge-map, toolbox, portability, north-star]
evidence: DOCTRINE_ENFORCEMENT.md (the adopted standard); scripts/check_doctrines.sh (the registry+driver); scripts/check_diagnosis_evidence.sh + scripts/check_task_tree_ownership.sh (the new scope-aware checks); scripts/check_memory_architecture.sh + knowledge-map/scripts/check_knowledge_map.sh (the pre-existing structural checks now registered); TOOLBOX.md (ANVIL's own diagnostic toolbox + the acceptance-checklist template); .githooks/pre-commit + .github/workflows/ci.yml (E3 + E4 run the driver); docs/tasks/DOCTRINE-ENFORCEMENT-ADOPTION.md (the owning tree); MEMORY_ARCHITECTURE.md §9 (the E1→E4 model this generalizes)
---

# 0026 - DOCTRINE-ENFORCEMENT-ADOPTION: adopt the portable doctrine-enforcement architecture

- Date: 2026-06-22
- Status: accepted
- Tree: `DOCTRINE-ENFORCEMENT-ADOPTION` (workflow tree; owns the adoption)
- Activated by: owner directive (`2026-06-22`) — "when the ramp up is complete then
  adopt this doctrine enforcement system" (the donor project's
  `DOCTRINE_ENFORCEMENT.md`), with the explicit steer that `TOOLBOX.md` catalog
  ANVIL's **own** tools for pinpointing issues ANVIL may have.

## Context

ANVIL already runs **three** of the four portable, harness-agnostic architectures:

1. **Task-trees** (`docs/TASK_TREE.md`) — per-unit work memory.
2. **Memory-architecture** (`MEMORY_ARCHITECTURE.md`) — durable 4-layer agent memory,
   with `scripts/check_memory_architecture.sh` wired into `.githooks/pre-commit` + CI.
3. **Knowledge-map** (`knowledge-map/`) — a retrieval layer over fact cards, with
   `knowledge-map/scripts/check_knowledge_map.sh` wired into the same hook + CI.

The **fourth** — doctrine-enforcement — was missing. Its thesis: *a doctrine that is
not mechanically checked is not enforced — it is a suggestion.* ANVIL's most
load-bearing rules lived as prose only: the 2026-05-17 **task-tree ownership** doctrine
("no code change without a task-tree leaf owning it first") and `COMMIT.md`'s
**mandatory live-doc evidence** ("`CHANGES.md` and `MEMORY.md` MUST be amended before
every commit"). Both were discoverable but not gated. The two existing checks were also
wired *directly* into the hook/CI rather than behind one uniform driver, so adding a
fourth check meant editing the hook again, and there was no meta-check that a registered
check actually exists and is executable.

## Decision

**Adopt the doctrine-enforcement architecture as portable architecture #4**, by
replaying its standard into ANVIL:

- Land `DOCTRINE_ENFORCEMENT.md` (the standard, §8 replay manifest) at the repo root.
- Add `scripts/check_doctrines.sh` — the **registry + driver**. It runs every
  registered check, **collects all results** (does not stop at the first failure),
  **meta-checks** that each registered check exists and is executable, prints a
  per-doctrine PASS/FAIL report, and exits nonzero iff any check failed. The registry
  array is the single source of truth for "which doctrines are enforced by what".
- Register the first four ANVIL doctrines:
  - `MEMORY-ARCH` (structural) → existing `scripts/check_memory_architecture.sh`.
  - `KNOWLEDGE-MAP` (structural) → existing `knowledge-map/scripts/check_knowledge_map.sh`.
  - `CODE-CHANGE-EVIDENCE` (evidence, scope-aware) → new
    `scripts/check_diagnosis_evidence.sh`: a staged code change must co-stage the
    mandatory live-doc evidence (`CHANGES.md` + `MEMORY.md`); pure non-code commits are
    exempt.
  - `TASK-TREE-OWNERSHIP` (structural, scope-aware) → new
    `scripts/check_task_tree_ownership.sh`: a staged code change must co-stage an owning
    `docs/tasks/*.md` task file; pure non-code commits are exempt.
- Rewire `.githooks/pre-commit` (E3) and `.github/workflows/ci.yml` (E4) to run the
  **driver** (preserving the Knowledge Map derive-and-stage step that runs before it).
- Add `TOOLBOX.md` — **ANVIL's own diagnostic toolbox** (per the owner steer): the
  catalog of the instruments ANVIL ships to pinpoint issues it may have — `--trace`,
  `--metrics`, `--dump-config`, `--introspect`, the MCP `analyze` / `coverage` /
  `coverage_gaps` queries, `validate` / `minimize` / `hunt`, `divergence`, `--diff-sim`,
  the `tool_matrix --<surface>-gate` / `--phase*-gate` runs, `tests/snapshots.rs`, and
  `scripts/ram_guard.sh` — plus the **acceptance-checklist template** a code change must
  satisfy, each box citing a named re-runnable oracle.
- Route every harness bootstrap pointer (`CLAUDE.md`, `AGENTS.md`, `.cursorrules`,
  `.github/copilot-instructions.md`, new `GEMINI.md` + `.windsurfrules`, and the
  `README.md` ramp-up list) to `DOCTRINE_ENFORCEMENT.md` + `TOOLBOX.md`, preserving the
  `README.md` + `MEMORY_ARCHITECTURE.md` references the memory-arch check requires.

### Construction discipline

- **The existing checks are registered, not rewritten.** No behaviour change to
  `MEMORY-ARCH` / `KNOWLEDGE-MAP`; the driver wraps them.
- **Scope-aware new checks (`DOCTRINE_ENFORCEMENT.md` §4(5)).** The code-scoped checks
  govern only commits that stage code (`src/`, `tests/`, `examples/`, behaviour-altering
  `Cargo` manifests / build logic); pure docs / workflow commits — including this tree's
  own leaves — are **exempt**, so the doctrine never blocks legitimate non-code work.
- **Structural co-staging proxy at pre-commit; oracle leg deferred (§4(7), §6.1, §9).**
  The new checks prove the mandatory files are *co-staged*; the un-fakeable proof
  (the change actually validates) is the `cargo` + `tool_matrix` oracle re-run at
  `COMMIT.md` / CI. The standard's §9 states this honest limit openly.
- **DUT byte-identical.** Every artifact is a workflow-config / live-doc file; no `src/`,
  no generated-RTL change, `tests/snapshots.rs` untouched.

## Decisive test applied

"Is every load-bearing ANVIL doctrine paired with a deterministic check that the git
hook (E3) and CI (E4) run from one registry+driver, with a meta-check against dangling
registry entries?" After adoption: yes for `MEMORY-ARCH`, `KNOWLEDGE-MAP`,
`CODE-CHANGE-EVIDENCE`, `TASK-TREE-OWNERSHIP`. Heavier oracle doctrines (byte-identical
snapshots, the downstream `tool_matrix` gates) remain where they already re-execute the
real generator + tools and are *referenced* in `DOCTRINE_ENFORCEMENT.md` §10, not
duplicated into the fast pre-commit path.

## Rejected alternatives

- **Leave the doctrines as prose.** Rejected — the thesis of the architecture is that
  unchecked prose is a suggestion; ANVIL's two most-emphasized doctrines were ungated.
- **Keep wiring each check directly into the hook/CI.** Rejected — no single registry,
  no meta-check, and the hook grows per check. The driver centralizes it.
- **Make the code-scoped checks block *all* commits (not scope-aware).** Rejected — it
  would block legitimate docs / workflow commits (including this adoption), violating
  §4(5); the doctrine must exempt what it does not govern.
- **Duplicate the heavy oracle gates (snapshots / `tool_matrix`) into pre-commit.**
  Rejected — too slow for the local gate (§4(7)); they run under `cargo test` and the
  local matrix per `COMMIT.md`, and are referenced in the registry table.
- **A generic debug toolbox in `TOOLBOX.md`.** Rejected per owner steer — `TOOLBOX.md`
  catalogs **ANVIL's own** diagnostic instruments, the ones that pinpoint issues ANVIL
  may have.
- **Claim "impossible to violate".** Rejected — local hooks are bypassable; the honest
  claim (§9) is "expensive, visible, and blocked at every active gate; CI is the
  backstop".

## Consequences

- ANVIL now runs **all four** portable architectures; non-compliant work lands only by
  defeating all four enforcement layers, and CI (E4) cannot be defeated from a clone.
- Adding a future doctrine is one `check_*.sh` (the §4 contract) + one registry line; the
  driver's meta-check rejects a dangling entry.
- The default `anvil` build and `--artifact dut` are byte-identical (workflow/docs only).
- `TOOLBOX.md` becomes the single place an agent or contributor learns which ANVIL
  instrument to reach for when a generated artifact misbehaves or a downstream tool
  rejects it, and what a code change must prove before it commits.

## Links

- Standard replayed: `DOCTRINE_ENFORCEMENT.md` (this repo's copy) — sibling of
  `MEMORY_ARCHITECTURE.md` §9 (the E1→E4 model generalized to every doctrine).
- Owner doctrine: `feedback_dont_ask_just_do` (announce + act),
  `feedback_task_tree_available` (task-tree ownership — the flagship doctrine now
  mechanized), `project_anvil_north_star` (the toolbox serves the bug-hunt mission).
- Owning tree: `docs/tasks/DOCTRINE-ENFORCEMENT-ADOPTION.md` (leaves `.1`–`.6`).
- Registry: `scripts/check_doctrines.sh`; acceptance checklist + ANVIL diagnostic
  toolbox: `TOOLBOX.md`.
