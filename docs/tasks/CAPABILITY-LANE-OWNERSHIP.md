# CAPABILITY-LANE-OWNERSHIP: Register Post-Phase Capability Lanes

## Metadata

- Tree ID: `CAPABILITY-LANE-OWNERSHIP`
- Status: `done`
- Roadmap lane: `Workflow / capability-lane task-tree ownership`
- Created: `2026-06-15`
- Last updated: `2026-06-15`
- Owner: repo-local workflow

## Goal

Register task-tree ownership for the three owner-directed post-phase
capability lanes **before** any implementation source edit occurs, so
that every future code change has a tree/leaf to own it first (the
non-negotiable 2026-05-17 doctrine). The three lanes were authorized by
the owner on `2026-06-15` ("do these in any order"):

1. `AGENT-MCP-EXPANSION` — extend the read-mostly agent/MCP interface
   (Lane 2 — executed first).
2. `SIGNOFF-AUTOMATION-EXPANSION` — broaden downstream signoff
   automation (Lane 3 — executed second).
3. `IDENTITY-DEEPENING` — advance the NodeId-as-identity /
   full-factorization north star (Lane 1 — executed third).

The owner-chosen-by-agent execution order is `2 → 3 → 1`: the agent/MCP
lane has the most finite, independently verifiable sub-deliverables and
is the freshest code area; signoff automation strengthens the
bug-hunting north star and the proof tooling the identity lane will
lean on; identity deepening is the deepest, most open-ended axis and is
best tackled once the supporting automation is richest.

## Non-Goals

- No source code change. No generated RTL behavior change. No CLI,
  config, or user-facing feature change.
- This tree does **not** finalize the leaf decomposition of the three
  capability lanes; each lane's own `.1` design leaf does that.
- This tree does **not** start implementation; it only establishes
  ownership and alignment.

## Acceptance Criteria

- The three capability lanes each have a task-tree file under
  `docs/tasks/`, with goal, non-goals, acceptance criteria, an initial
  decomposition, and a first design leaf.
- `docs/TASK_TREE.md` lists all four trees (this ownership tree plus the
  three lanes) and their current frontiers/status.
- `ROADMAP.md` records the three lanes as task-tree-owned post-phase
  capability lanes so roadmap ↔ task-tree state stay aligned.
- `CHANGES.md` and `MEMORY.md` reflect the new active ownership state.
- Focused docs/workflow validation passes.
- The completed activity is committed through `COMMIT.md` with this leaf
  ID in the subject.

## Task Tree

- ID: `CAPABILITY-LANE-OWNERSHIP`
  Status: `done`
  Goal: `Register the three post-phase capability lanes as task trees.`
  Children: `CAPABILITY-LANE-OWNERSHIP.1`

- ID: `CAPABILITY-LANE-OWNERSHIP.1`
  Status: `done`
  Goal: `Create and index the three capability lane trees (Lane 2 active; Lanes 1 & 3 proposed) and align ROADMAP/live docs.`
  Acceptance: `All three lanes are task-tree tracked and linked from the active-tree index before implementation starts; ROADMAP/CHANGES/MEMORY agree.`
  Verification: `cargo check --all-targets`; `cargo fmt --all --check`; `scripts/check_memory_architecture.sh`; `knowledge-map/scripts/check_knowledge_map.sh`; `git diff --check`.
  Commit: `CAPABILITY-LANE-OWNERSHIP.1 - register post-phase capability lanes`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `CAPABILITY-LANE-OWNERSHIP.1` | `done` | Completed; implementation resumes at `AGENT-MCP-EXPANSION.1`. |

## Decisions

- `2026-06-15`: Mirror the `ROADMAP-FOLLOWUP-OWNERSHIP` precedent — use one
  completed workflow tree to own the registration activity itself, while
  the three newly opened capability trees own all future implementation
  leaves.
- `2026-06-15`: Execution order `AGENT-MCP-EXPANSION` →
  `SIGNOFF-AUTOMATION-EXPANSION` → `IDENTITY-DEEPENING`. Lane 2 is opened
  `active`; Lanes 3 and 1 are opened `proposed` and promoted to `active`
  when their turn arrives, so the frontier never lists leaves from a lane
  that is not yet being worked.

## Open Questions

- None. Each lane's `.1` design leaf resolves that lane's scope choices.

## Blockers

- None.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-06-15` | `CAPABILITY-LANE-OWNERSHIP.1` | `cargo check --all-targets`; `cargo fmt --all --check`; `scripts/check_memory_architecture.sh`; `knowledge-map/scripts/check_knowledge_map.sh`; `git diff --check` | pending fill-in at commit; full `cargo test` not run because no source code changed (resource policy `docs/decisions/0003`) |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `CAPABILITY-LANE-OWNERSHIP.1` | `CAPABILITY-LANE-OWNERSHIP.1 - register post-phase capability lanes` | `pending hash`; closes tree. |

## Changelog

- `2026-06-15`: Created task tree, completed
  `CAPABILITY-LANE-OWNERSHIP.1`, and closed the tree.
