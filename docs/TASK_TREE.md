# Repo-Local Task Tree Workflow

This document defines the repo-local task-tree workflow used by ANVIL. It is
intentionally portable: the workflow was lifted from FSMGen
(`/Users/richarddje/Documents/github/fsmgen/docs/TASK_TREE.md`) and adapted to
ANVIL's existing live-doc set.

For the portable, project-agnostic setup guide, read
[docs/TASK_TREE_README.md](TASK_TREE_README.md).

## Purpose

Use a task tree when a top-level task is too broad to finish safely as one
signoff-level slice, or when a task is expected to discover subtasks and
sub-subtasks over time.

The goal is not to create a second roadmap. `ROADMAP.md` states the high-level
phase direction. A task tree owns the recursive breakdown, current frontier,
acceptance criteria, blockers, decisions, validation, and completion evidence
for one top-level task.

## ANVIL Adoption Scope

**Doctrine (2026-05-17, non-negotiable, owner directive):** it is
**strictly forbidden to make any code change without it being
task-tree tracked or task-tree owned first.** Task-tree ownership
demonstrably improved code review and code quality over the earlier
ad-hoc/linear cadence, so it is now the mandatory mode of work for all
code — no compromise, no exceptions.

- **Code change ⇒ a task-tree leaf must own it, *before* the edit.**
  "Code" means anything that changes program/generator behaviour or
  generated RTL: `src/`, `tests/`, `examples/`, build/codegen logic,
  `Cargo` manifests that alter behaviour. If no tree/leaf covers the
  change, create or extend one (`docs/tasks/<TREE>.md` + a
  `docs/TASK_TREE.md` row) and name the owning leaf first. The leaf ID
  goes in the commit subject / first body line (`COMMIT.md` task-tree
  rules).
- **Exempt (no tree required):** pure-docs / live-doc / mdBook edits,
  workflow-config tweaks, and recording doctrine itself. These are not
  code changes. They still follow the standard `COMMIT.md` checklist.
- **`rN` is *not* retired** — it survives only as the optional
  within-leaf slice cadence *inside* a task tree (as the closed
  `HIERARCHY-AWARE-IDENTITY` leaves landed as r85/r86/r87). A bare
  `rN` slice that no task-tree leaf owns is no longer a legal way to
  land a code change.
- **Do not migrate finished work** retroactively. Closed `rN` slices
  stay where they are; the mandate is forward-going.

**Project-wide tracking directive (2026-05-16):** by explicit owner
directive, *every remaining roadmap phase* now has a registered
top-level task tree (`PHASE-4-HIERARCHY`, `PHASE-5-PARAMETERIZATION`,
`PHASE-5B-AGGREGATES`, `PHASE-6-ADVANCED-MOTIFS`,
`PHASE-7-ORACLE-MICRODESIGN`, `PHASE-8-FRONTEND-ACCEPT`,
`PHASE-9-MULTI-ARTIFACT-UMBRELLA`) so the whole roadmap is trackable
through task trees. This **does not retire `rN`**: `rN` remains the
within-leaf slice cadence. Each phase tree owns the sub-objective
decomposition, frontier, blockers, and completion evidence; individual
linear coverage slices inside a leaf still land under the `rN` naming +
`CHANGES.md` + `MEMORY.md` combination, exactly as the closed
`HIERARCHY-AWARE-IDENTITY` tree's leaves landed as r85/r86/r87. Closed
`rN` slices are still not migrated retroactively.

## Active Task Trees

| Tree | Status | Roadmap lane | Current frontier | File |
| --- | --- | --- | --- | --- |
| `HIERARCHY-AWARE-IDENTITY` | `done` | Phase 4 — Hierarchy | (complete — all leaves done) | [docs/tasks/HIERARCHY-AWARE-IDENTITY.md](tasks/HIERARCHY-AWARE-IDENTITY.md) |
| `PHASE-4-HIERARCHY` | `done` | Phase 4 — Hierarchy | (complete — `.1` done, `.2` superseded, `.3` done; Phase 4 closed) | [docs/tasks/PHASE-4-HIERARCHY.md](tasks/PHASE-4-HIERARCHY.md) |
| `PHASE-5-PARAMETERIZATION` | `done` | Phase 5 — Parameterization | (complete — Phase 5 closed `2026-05-17`; `.2.4b` verified `/tmp/anvil-tool-matrix-phase5-p1` clean → ROADMAP Phase 5 `done`) | [docs/tasks/PHASE-5-PARAMETERIZATION.md](tasks/PHASE-5-PARAMETERIZATION.md) |
| `PHASE-5B-AGGREGATES` | `done` | Phase 5b — Synthesizable aggregates | (complete — Phase 5b closed `2026-05-18`; `.2.4` verified `/tmp/anvil-tool-matrix-phase5b-p1` clean → ROADMAP Phase 5b `done`) | [docs/tasks/PHASE-5B-AGGREGATES.md](tasks/PHASE-5B-AGGREGATES.md) |
| `PHASE-6-ADVANCED-MOTIFS` | `done` | Phase 6 — Advanced motifs | (complete — Phase 6 closed `2026-05-20`; **memory** verified `/tmp/anvil-tool-matrix-phase6-p1` clean [`.2.4`, 219/876, `coverage_gaps=[]`, 876/0 Verilator+both-Yosys, `saw_inferrable_memory_design=true`] **and FSM** verified `/tmp/anvil-tool-matrix-phase6-fsm-p1` clean [`.3.4b`, 222/888, `coverage_gaps=[]`, 888/0 Verilator+both-Yosys, `saw_fsm_design=true` AND `saw_inferrable_memory_design=true`, P4/P5/P5b regressions proven in the same banked report] → ROADMAP Phase 6 `done`; multi-clock CDC remains the explicitly-optional, separately-prioritised deferral) | [docs/tasks/PHASE-6-ADVANCED-MOTIFS.md](tasks/PHASE-6-ADVANCED-MOTIFS.md) |
| `PHASE-7-ORACLE-MICRODESIGN` | `done` | Phase 7 — Oracle-backed micro-design artifacts | (complete — Phase 7 closed `2026-05-20`; verified-clean banked artifact `/tmp/anvil-microdesign-parity-phase7-yosys-p1/` — `cargo test -- --ignored parity_against_real_yosys_write_json` against yosys 0.64 exits 0 with "parity gate clean across 5 seeds"; per-seed fact agreement verified incl. seed 7 P4=-1 [bits=8 on both sides post-`.2c.2b.1` non-negative-modulo-idiom fix] and both generate branches exercised [seed 12345 takes `g_else`, others `g_taken`]; explicit yosys-supported-categories scope caveat — richer-AST coverage via slang/verilator-with-debug is a recorded post-Phase-7 follow-up that does NOT retract closure since ANVIL's by-construction oracle already covers all 7 manifest categories) | [docs/tasks/PHASE-7-ORACLE-MICRODESIGN.md](tasks/PHASE-7-ORACLE-MICRODESIGN.md) |
| `PHASE-8-FRONTEND-ACCEPT` | `active` | Phase 8 — Frontend/elaboration accept corpora | `PHASE-8-FRONTEND-ACCEPT.2b` (unblocked, code-bearing) — **`.2a` done (`2026-05-20`)**: new top-level module `src/frontend/mod.rs` (registered via `pub mod frontend`); AST IR types (`SourceUnit`/`Package`/`Module`/`ModuleItem`/`Instance`/`GenerateIf`/`ParamDecl`/`ParamBinding`); construction-time elaboration-evaluator `elaborate()` (builder IS the oracle — resolves every `.value`/`.resolved`/`.taken` in place); rules-first reproducible `build_acceptable_unit(seed, n_params, n_children)` (one ChaCha8Rng; package + child + top with chained localparams + named-binding instances + generate-if); cross-tree reuse of Phase 7's `ConstExpr`/`eval`/`ParamKind`/`BinOp` per full-factorization plan; 4 unit proofs green incl. load-bearing oracle-no-drift invariant `elaborated_facts_match_a_fresh_reeval_across_the_seed_set`; full `cargo test` green (lib 233 ← 229 + 4). `.2b` adds un-elaborated-SV + elaborated-facts JSON manifest emitters from the same `.2a` oracle | [docs/tasks/PHASE-8-FRONTEND-ACCEPT.md](tasks/PHASE-8-FRONTEND-ACCEPT.md) |
| `PHASE-9-MULTI-ARTIFACT-UMBRELLA` | `active` | Phase 9 — Multi-artifact ANVIL umbrella | `PHASE-9-MULTI-ARTIFACT-UMBRELLA.2` (`.1` design done — `ArtifactLane` trait + shared plumbing + default-`dut` `--artifact` flag preserving book/CI byte-identical + 4 rejected alternatives; `.2` blocked until ≥2 delivered lanes) | [docs/tasks/PHASE-9-MULTI-ARTIFACT-UMBRELLA.md](tasks/PHASE-9-MULTI-ARTIFACT-UMBRELLA.md) |
| `INSTA-SNAPSHOTS` | `done` | Quality — reproducibility regressions | (complete — closed `2026-05-18`; `.1` insta `=1.47.2` pin + baseline / `.2` 6 byte-stable shapes spanning every reachable axis incl. dedup-canonical-signatures / `.3` COMMIT.md non-negotiable snapshot-acceptance protocol + book "Snapshot guard-rails") | [docs/tasks/INSTA-SNAPSHOTS.md](tasks/INSTA-SNAPSHOTS.md) |
| `DIFFERENTIAL-SIMULATION` | `active` | Quality — signoff-level downstream consistency | `DIFFERENTIAL-SIMULATION.2b` (`.1` iverilog-compat + `.2a` harness design done; `.2` split; `.2b` = implement the IR-driven testbench + dual-sim orchestration + `#[ignore]` byte-equal proof) | [docs/tasks/DIFFERENTIAL-SIMULATION.md](tasks/DIFFERENTIAL-SIMULATION.md) |
| `COVERAGE-INSTRUMENTATION` | `done` | Quality — test-discipline visibility | (complete — closed `2026-05-18`; `.1` llvm-cov baseline / `.2` top-5 triage [no dead code] / `.3` cone retry-exhaustion focused proof + config orphan-knob audit [3 documented-reserved knobs] + baseline refresh) | [docs/tasks/COVERAGE-INSTRUMENTATION.md](tasks/COVERAGE-INSTRUMENTATION.md) |
| `BOOK-EXAMPLES-RUNNABLE` | `done` | Quality — user-facing book correctness | (complete — closed `2026-05-18`; `.1`/`.2.1`/`.2.2` done: 45+1 examples migrated to `cargo run --release --`, `tests/book_examples.rs` harness + `mdbook test` CI gate, pipe-deadlock root-caused & fixed, `cargo test --test book_examples` 3/3 green, 54 runnable exit-0) | [docs/tasks/BOOK-EXAMPLES-RUNNABLE.md](tasks/BOOK-EXAMPLES-RUNNABLE.md) |

## Directory Layout

```text
docs/TASK_TREE.md
docs/TASK_TREE_README.md
docs/tasks/
  TEMPLATE.md
  <TREE>.md
```

`docs/TASK_TREE.md` is the workflow and active-tree index.
Each top-level task owns one file in `docs/tasks/`.
`docs/tasks/TEMPLATE.md` is copied when creating a new top-level tree.

## Definitions

- Task tree: the recursive decomposition of one top-level task.
- Node: one item in that tree.
- Container node: a node with children. It is not directly executable.
- Leaf node: a node with no children. It is the only unit PNT may implement.
- Current frontier: the ordered set of leaf nodes that are eligible to be
  picked next.
- Slice: one completed leaf task plus its tests, docs, live-doc updates, and
  commit workflow.
- Evidence: the validation output, changed-doc summary, and git commit subject
  that prove a leaf was completed.

## ID Rules

Each task tree has a stable top-level ID.

```text
<TREE>
<TREE>.1
<TREE>.1.1
<TREE>.1.1.1
```

Rules:

- `<TREE>` uses uppercase letters, digits, and hyphens.
- Child IDs append dot-separated positive integers.
- IDs are permanent once published.
- Never renumber closed nodes.
- If a new ordering is needed, add new IDs and mark old nodes `superseded` or
  `deferred` with a reason.
- A commit that completes a task-tree leaf must identify the leaf ID in the
  commit subject or in the first body line.

## Status Vocabulary

Use only these statuses.

| Status | Meaning |
| --- | --- |
| `proposed` | Captured but not yet accepted into the active tree. |
| `active` | The top-level tree is open, or a container has unfinished children. |
| `pending` | Ready to be selected once it reaches the current frontier. |
| `in_progress` | Currently being implemented in the worktree. |
| `blocked` | Cannot proceed without a named blocker and unblock condition. |
| `done` | Completed, validated, documented, and committed. |
| `deferred` | Deliberately postponed with an explicit consequence. |
| `superseded` | Replaced by another node, with the replacement ID named. |

## Required Task File Sections

Every top-level task file must contain:

- Metadata: tree ID, status, roadmap lane, created date, last updated date.
- Goal: the user-visible or project-visible outcome.
- Non-goals: what this tree deliberately does not try to solve.
- Acceptance criteria: concrete conditions that close the top-level task.
- Task tree: all known nodes, with status and short result intent.
- Current frontier: ordered leaf nodes that PNT may select next.
- Decisions: accepted technical decisions and their rationale.
- Open questions: unresolved questions that do not block the whole tree yet.
- Blockers: blockers with unblock conditions.
- Verification log: checks run for completed leaves.
- Commit log: leaf IDs mapped to completion commit subjects.
- Changelog: dated edits to the tree itself.

## Node Rules

Every node must be one of these two shapes.

Container node:

```text
- ID: <TREE>.<n>
  Status: active
  Goal: ...
  Children: <TREE>.<n>.1, <TREE>.<n>.2
```

Leaf node:

```text
- ID: <TREE>.<n>
  Status: pending
  Goal: ...
  Acceptance: ...
  Verification: pending
  Commit: pending
```

A node with children must not be marked `done` until every child is `done`,
`deferred`, or `superseded`, and every non-`done` child has a recorded reason.

## Current Frontier Rules

The current frontier is the only list PNT uses when selecting work from a task
tree.

Rules:

- The frontier contains only leaf nodes.
- The frontier is ordered by intended priority.
- A container never appears in the frontier.
- A blocked node stays out of the frontier until unblocked.
- When a leaf is split, remove that leaf from the frontier, mark it `active`,
  add children, and place the first executable child or children in the
  frontier.
- When a leaf completes, remove it from the frontier and add the next eligible
  leaf or leaves.

## PNT Selection Rules

When PNT is asked to continue and at least one active task tree exists:

1. Read `docs/TASK_TREE.md`.
2. Read the active task file named in the `Active Task Trees` table.
3. Pick the first eligible leaf in that file's `Current Frontier`.
4. Implement only that leaf.
5. If the leaf is too broad, split it before implementation and commit the
   tree update as the leaf's honest outcome.
6. Run the required validation for the leaf.
7. Update the task file, live docs, and roadmap if status changed.
8. Run the full commit workflow before selecting another leaf.

If several active trees exist, choose the first active tree in the table unless
the user names another tree or the roadmap status names a different immediate
lane.

When the user asks for PNT and **no** active task tree is appropriate (the
work is a linear `rN` coverage extension), continue on the `rN` convention —
do not invent a task tree just to satisfy this section.

## Splitting Rules

Split a node when any of these are true:

- It cannot be completed to signoff quality in one slice.
- It mixes design, implementation, diagnostics, tests, and docs in ways that
  can be reviewed independently.
- It hides an unresolved policy choice behind implementation wording.
- It would require touching unrelated ownership areas in one commit.
- It discovers a lower-level dependency that should be solved first.

Do not split merely to create vague placeholders. Every child must have a
clear goal and a way to verify completion.

## Completion Rules

A leaf is complete only when all of the following are true:

- Implementation or documentation work for that leaf is finished.
- Focused checks passed, and broader checks ran when warranted (see
  `COMMIT.md` for the full pre-commit checklist).
- The owning task file records the result, validation, and commit subject.
- `CHANGES.md`, `MEMORY.md`, and the other live docs listed in `COMMIT.md`
  are updated when the leaf changes project state.
- The commit workflow in `COMMIT.md` has completed.
- `git_message_brief.txt` has been cleared after commit.

Commit hashes are intentionally not required inside the same task-file update:
the final hash cannot be known until after the commit exists. The stable
join key is the leaf ID in the commit subject or first body line. Later status
refreshes may backfill hashes if useful.

## Blocker Rules

A blocked node must record:

- the exact blocker,
- why it blocks the node,
- the unblock condition,
- and the next task that should run instead, if any.

Do not leave a node as `blocked` only because it is large or unclear. Large or
unclear work should be split until a real blocker is visible.

## Relationship To Live Docs

The task tree is the detailed execution ledger.

- `ROADMAP.md` remains the canonical high-level phase status.
- `MEMORY.md` remains the recovery/handoff continuity log.
- `CHANGES.md` remains the chronological technical history.
- `DEVELOPMENT_NOTES.md` remains design rationale.
- `CODEBASE_ANALYSIS.md` remains the live workspace analysis.
- `USER_GUIDE.md` remains user-facing CLI/workflow reference.
- The mdBook (`book/src/*.md`) remains user-facing product/algorithm
  documentation.

Do not duplicate the whole task tree into those files. Link to the task tree
and summarize only the part that changes live project state. ANVIL's
`rN`-named slices stay recorded in `CHANGES.md` and `MEMORY.md` as before —
task-tree adoption does not change how `rN` slices land.

## Commit Workflow Tie-In

When a commit completes a task-tree leaf, `COMMIT.md`'s checklist still
applies in full. The only additional rule is:

- The commit subject or first body line must include the leaf ID
  (e.g., `HIERARCHY-AWARE-IDENTITY.1`).
- The owning `docs/tasks/<TREE>.md` file must be updated in the same commit
  with the leaf's new status, verification log entry, and commit-log entry.

For commits that are **not** task-tree-managed (linear `rN` slices, isolated
doc edits, workflow tweaks), no leaf ID is required.

## Copying This Workflow To Another Project

The detailed project-adoption checklist lives in
[docs/TASK_TREE_README.md](TASK_TREE_README.md).
