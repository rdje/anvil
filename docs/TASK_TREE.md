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
| `PHASE-8-FRONTEND-ACCEPT` | `done` | Phase 8 — Frontend/elaboration accept corpora | (complete — Phase 8 closed `2026-05-20`; verified-clean banked artifact `/tmp/anvil-frontend-parity-phase8-yosys-p1/` — `cargo test -- --ignored parity_against_real_yosys_hierarchy_write_json` against yosys 0.64 exits 0 with "parity gate clean across 5 seeds" on **first try**; per-seed fact agreement verified incl. both generate branches exercised AND the load-bearing hierarchy-aware Phase-8 axis (every seed has 2 instances × 4 per-instance per-binding values matched); explicit yosys-supported-categories scope caveat — richer-AST coverage via slang/verilator-with-debug is a recorded post-Phase-8 follow-up that does NOT retract closure since ANVIL's by-construction oracle already covers all 7 manifest categories; cross-tree reuse of Phase 7's `expr_to_sv` carried `.2c.2b.1`'s non-negative-modulo-idiom fix forward at zero incremental cost — full-factorization doctrine vindicated) | [docs/tasks/PHASE-8-FRONTEND-ACCEPT.md](tasks/PHASE-8-FRONTEND-ACCEPT.md) |
| `PHASE-9-MULTI-ARTIFACT-UMBRELLA` | `done` | Phase 9 — Multi-artifact ANVIL umbrella | (complete — Phase 9 closed `2026-05-20`; `src/umbrella/` carries the `ArtifactLane` trait + all 3 lane impls + 8 cargo-portable proofs incl. per-lane byte-identical regression + cross-lane heterogeneous `dyn` dispatch; `src/main.rs` carries the `--artifact <lane>` CLI flag with default `dut`; load-bearing byte-identical default-`dut` contract verified by `tests/book_examples::every_runnable_book_bash_block_succeeds` passing 3/3 in 80s AFTER the CLI change. **All 9 numbered roadmap phases now delivered.** Remaining open follow-up trees: multi-clock CDC [optional deferral from Phase 6]; `DIFFERENTIAL-SIMULATION.3`/`.4` [quality lane — `.2b` closed `2026-05-24`]) | [docs/tasks/PHASE-9-MULTI-ARTIFACT-UMBRELLA.md](tasks/PHASE-9-MULTI-ARTIFACT-UMBRELLA.md) |
| `INSTA-SNAPSHOTS` | `done` | Quality — reproducibility regressions | (complete — closed `2026-05-18`; `.1` insta `=1.47.2` pin + baseline / `.2` 6 byte-stable shapes spanning every reachable axis incl. dedup-canonical-signatures / `.3` COMMIT.md non-negotiable snapshot-acceptance protocol + book "Snapshot guard-rails") | [docs/tasks/INSTA-SNAPSHOTS.md](tasks/INSTA-SNAPSHOTS.md) |
| `DIFFERENTIAL-SIMULATION` | `active` | Quality — signoff-level downstream consistency | `DIFFERENTIAL-SIMULATION.4` (docs — describe the contract) — **`.3b.2` done + closes `.3b` + `.3` container (`2026-05-24`)**: `src/bin/tool_matrix.rs` (~600 lines added) gains the `--diff-sim` opt-in CLI flag + `DiffSimReport` per-module struct + `saw_design_with_cross_simulator_agreement` coverage fact + per-axis subset selector (combinational/sequential-flop/hierarchy/memory/fsm; K=5 cap; deterministic) + per-module pipeline (after Verilator+Yosys clean; sentinel-gated) + `parse_dut_ports`/`emit_testbench_for_ports` matrix-side helpers + 8 cargo-portable proofs + 1 tool-gated `#[ignore]` end-to-end gate. Real-tool gate clean: `DiffSimReport { ran: true, success: true, n_samples: 8 }` (24.15s wall against iverilog 13.0 + verilator 5.046). `cargo fmt`/clippy(-D warnings)/check all clean; `cargo test --bin tool_matrix` 37 (+8); all other suites unchanged (247 lib, 121 pipeline, 6 snapshots, 15+1 microdesign_parity, 12+2 frontend_parity, **3 book_examples — byte-identical default-`dut` contract preserved**, 2+2 diff_sim). FOUND-AND-FIXED spec-vs-reality bug during e2e gate (ANVIL emits `"input  logic"` with TWO spaces — `src/emit/sv.rs:124`; replaced `strip_prefix` with `split_whitespace`). **First gate to wire downstream-tool *semantic* agreement into the repo-owned `tool_matrix` flow.** `.3b.1` done `2026-05-24` (pure refactor → `src/diff_sim/mod.rs`); `.3a` design landed `2026-05-24` (docs-only); `.2b.2` closed `.2b` + `.2` container `2026-05-24` — first gate to assert downstream-tool *semantic* agreement on ANVIL output (`project_anvil_north_star.md`). | [docs/tasks/DIFFERENTIAL-SIMULATION.md](tasks/DIFFERENTIAL-SIMULATION.md) |
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
