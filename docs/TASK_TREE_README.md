# Task-Tree Tracking Setup Guide

This guide explains how to add the repo-local task-tree tracking workflow to a
new project.

Use this document when a project already has, or wants to add, a roadmap and a
live roadmap-status file, but also needs a precise way to track task
decomposition over time without losing subtasks, blockers, decisions, or
completion evidence.

## What The Task Tree Is For

The roadmap answers:

- What broad lanes exist?
- Which lane is active?
- What is done, in progress, left, or deferred at the workstream level?

The task tree answers:

- Which exact top-level task is being decomposed?
- Which subtasks and sub-subtasks exist?
- Which leaf is eligible to be picked next?
- What decisions, blockers, and open questions belong to this task?
- What validation and commit evidence closed each executable leaf?

The task tree is therefore a companion to the roadmap and roadmap-status file.
It does not replace them.

## Files To Add

Minimum required files:

```text
docs/TASK_TREE.md
docs/tasks/TEMPLATE.md
docs/tasks/<FIRST-TREE>.md
```

Recommended project integration files:

```text
README.md
ROADMAP_STATUS.md
COMMIT.md
SESSION_BOOTSTRAP.md
MEMORY.md
CHANGES.md
DEVELOPMENT_NOTES.md
LIVE_ACHIEVEMENT_STATUS.md
```

The exact live-doc names can differ in another project. What matters is that
the project has:

- one front-door navigation file,
- one roadmap-status file,
- one commit-workflow file,
- one session-start/bootstrap file,
- and one or more continuity/history files.

## File Roles

| File | Role |
| --- | --- |
| `docs/TASK_TREE_README.md` | Setup guide for installing this workflow in another project. |
| `docs/TASK_TREE.md` | Local operating spec, active task-tree index, and PNT selection rules. |
| `docs/tasks/TEMPLATE.md` | Copyable skeleton for each new top-level task tree. |
| `docs/tasks/<TREE>.md` | One task tree for one top-level task. |
| `ROADMAP_STATUS.md` | High-level status board; links active lanes to active task trees. |
| `COMMIT.md` | Commit workflow; requires task-file updates and leaf-ID traceability. |
| `README.md` | Project entry point; links the task-tree docs. |
| `SESSION_BOOTSTRAP.md` | Session startup ritual; tells agents to read active task trees. |
| live history docs | Recovery and rationale logs; summarize task-tree state changes without duplicating the tree. |

## Minimum Setup

Use this when you want the workflow running with the fewest moving parts.

1. Create `docs/tasks/`.
2. Copy `docs/TASK_TREE.md` into the project.
3. Copy `docs/tasks/TEMPLATE.md` into the project.
4. Create the first task file from the template:

```text
docs/tasks/<FIRST-TREE>.md
```

5. Add the first task file to the `Active Task Trees` table in
   `docs/TASK_TREE.md`.
6. Add `docs/TASK_TREE.md` and `docs/tasks/TEMPLATE.md` to the project
   `README.md` or equivalent navigation file.
7. Add one rule to the commit workflow:

```text
If a completed activity belongs to a task-tree leaf, update the owning
docs/tasks/*.md file and identify the leaf ID in the commit subject or first
body line.
```

8. Add one line to the roadmap-status file for the active lane:

```text
Active task tree: docs/tasks/<FIRST-TREE>.md; current frontier: <TREE>.1.
```

At that point the workflow is usable.

## Recommended Full Setup

Use this for a project where agents need reliable crash recovery, handoff
continuity, and PNT-style execution.

1. Add `docs/TASK_TREE_README.md`.
2. Add `docs/TASK_TREE.md`.
3. Add `docs/tasks/TEMPLATE.md`.
4. Create `docs/tasks/<FIRST-TREE>.md` from the template.
5. Add the first tree to the `Active Task Trees` table in `docs/TASK_TREE.md`.
6. Update `README.md`:
   - Add `docs/TASK_TREE_README.md` to the documentation index.
   - Add `docs/TASK_TREE.md` to the fast ramp-up order.
   - Add `docs/tasks/TEMPLATE.md` to the documentation index.
   - Add any active task files to the documentation index if the project keeps
     a full Markdown index.
7. Update `SESSION_BOOTSTRAP.md` or equivalent:
   - Read `README.md`.
   - Read `COMMIT.md`.
   - Read `ROADMAP_STATUS.md`.
   - Read `docs/TASK_TREE.md`.
   - Read active task files listed in `docs/TASK_TREE.md`.
   - Pick work from the current frontier when the user asks for PNT.
8. Update `ROADMAP_STATUS.md`:
   - Keep roadmap lanes high-level.
   - For each active lane with task-tree-managed work, link the owning task
     file and name the current frontier leaf.
   - Do not copy the whole task tree into the roadmap.
9. Update `COMMIT.md`:
   - Require task-tree files to be updated when node status, frontier,
     blockers, decisions, validation, or completion evidence changes.
   - Require the commit subject or first body line to include the leaf ID for
     task-tree-managed work.
   - Require one commit per completed leaf before selecting another leaf.
10. Update continuity/history docs:
    - `MEMORY.md`: record the current active tree and frontier for recovery.
    - `CHANGES.md`: log creation of the workflow and any task-tree status
      transition that changes project state.
    - `DEVELOPMENT_NOTES.md`: record rationale and policy decisions.
    - `LIVE_ACHIEVEMENT_STATUS.md`: record the latest completed slice.
11. Commit the setup as one documentation/workflow slice.

## Adapting `docs/TASK_TREE.md`

Keep these sections:

- Purpose
- Active Task Trees
- Directory Layout
- Definitions
- ID Rules
- Status Vocabulary
- Required Task File Sections
- Node Rules
- Current Frontier Rules
- PNT Selection Rules
- Splitting Rules
- Completion Rules
- Blocker Rules
- Relationship To Live Docs

Customize these parts:

- Project name.
- Roadmap lane names.
- Live-doc filenames if the project uses different names.
- Commit-message policy if the project does not use `git_message_brief.txt`.
- Any project-specific default rule, such as "all ISF work is task-tree-managed"
  in FSMGen.

Remove project-specific sections that do not apply to the new project.

## Creating The First Task Tree

Create one file per top-level task. The file name should be stable and
descriptive:

```text
docs/tasks/API-STABILIZATION.md
docs/tasks/PARSER-DIAGNOSTICS.md
docs/tasks/UI-REDESIGN.md
docs/tasks/BUILD-CACHE.md
```

Choose a stable tree ID:

```text
API-STABILIZATION
PARSER-DIAGNOSTICS
UI-REDESIGN
BUILD-CACHE
```

Then define the first leaf:

```text
- ID: `API-STABILIZATION.1`
  Status: `pending`
  Goal: `Inventory current public API surfaces and name ownership boundaries.`
  Acceptance: `The task file lists public surfaces, owners, unknowns, and the next executable leaf.`
  Verification: `pending`
  Commit: `pending`
```

Add that leaf to the current frontier:

```text
| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `API-STABILIZATION.1` | `pending` | Policy and implementation need an accurate surface inventory first. |
```

## Operating Rules

Use these rules once the workflow is installed:

- PNT selects the first eligible leaf from the active tree's current frontier.
- Implement only one leaf at a time.
- Do not implement container nodes.
- If a leaf is too large, split it into child leaves before implementation.
- Keep node IDs stable forever.
- Do not renumber closed nodes.
- Record blockers with unblock conditions.
- Record decisions where they are made.
- Record validation in the owning task file.
- Update live docs only with summaries and links, not a duplicate of the whole
  task tree.
- Commit every completed leaf before selecting another leaf.

## Completion Evidence

A completed leaf should leave these traces:

- The task node status is `done`.
- The verification log names the checks run.
- The commit log names the commit subject or reference.
- The commit subject or first body line contains the leaf ID.
- Live docs summarize any project-state change.
- The roadmap-status file reflects any active-lane, done, left, or frontier
  change.

Commit hashes do not have to be written into the same task-file update. The
hash is only known after commit. The reliable join key is the leaf ID in the
task file and commit message. Hashes can be backfilled later if the project
wants that extra index.

## What Not To Do

- Do not use the roadmap as the detailed task ledger.
- Do not put broad container tasks in the current frontier.
- Do not create vague children that cannot be verified.
- Do not duplicate the whole task tree into `ROADMAP_STATUS.md`.
- Do not leave completed leaves uncommitted.
- Do not silently continue when a discovered subtask changes the scope; split
  the node and update the frontier.
- Do not renumber nodes after they have been referenced by commits or live
  docs.

## Setup Checklist

Use this checklist when enabling the workflow in a new project.

```text
[ ] docs/TASK_TREE_README.md exists.
[ ] docs/TASK_TREE.md exists and is customized for the project.
[ ] docs/tasks/TEMPLATE.md exists.
[ ] docs/tasks/<FIRST-TREE>.md exists.
[ ] docs/TASK_TREE.md lists the first active tree.
[ ] README.md links docs/TASK_TREE_README.md and docs/TASK_TREE.md.
[ ] ROADMAP_STATUS.md links active roadmap lane(s) to active task tree(s).
[ ] COMMIT.md requires task-file updates and leaf-ID commit traceability.
[ ] SESSION_BOOTSTRAP.md reads docs/TASK_TREE.md and active task files.
[ ] Continuity/history docs summarize the setup.
[ ] The setup is committed as one documentation/workflow slice.
```

## Minimal First Commit Message

```text
Docs: add task-tree tracking workflow

- Add task-tree setup guide, local workflow, and reusable task template
- Create the first active task tree and current-frontier leaf
- Wire roadmap, commit workflow, and startup docs to the task-tree ledger
```
