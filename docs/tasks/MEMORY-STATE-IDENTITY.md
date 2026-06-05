# MEMORY-STATE-IDENTITY: Reset-Defined Memory Identity

## Metadata

- Tree ID: `MEMORY-STATE-IDENTITY`
- Status: `done`
- Roadmap lane: `NodeId as identity / full-factorization mode`
- Created: `2026-06-05`
- Last updated: `2026-06-05`
- Owner: repo-local workflow

## Goal

Exhaust the safe path from the current instance-local memory boundary
to any reset-defined memory-state sharing ANVIL can prove.

## Non-Goals

- No merging of current reset-less inferrable memories.
- No assumption that equal read/write cones imply equal stored array
  contents.
- No generate-then-filter memory legality repair.
- No memory merge that changes emitted RTL under
  `identity_mode = relaxed`.

## Acceptance Criteria

- Every source edit is owned by a leaf before it occurs.
- Reset-less memories remain instance-local and covered by regression
  tests.
- If reset-defined memory identity is implemented, the reset/init
  semantics are explicit in the IR/emitter and downstream-clean.
- If reset-defined memory identity cannot be implemented safely in the
  current synthesizable subset, the blocker is recorded with evidence.
- User-facing docs explain the memory-state identity boundary and any
  new reset-defined memory behavior.
- Each completed leaf is committed through `COMMIT.md`.

## Task Tree

- ID: `MEMORY-STATE-IDENTITY`
  Status: `done`
  Goal: `Determine and implement safe reset-defined memory-state identity.`
  Children: `MEMORY-STATE-IDENTITY.1`, `MEMORY-STATE-IDENTITY.2`, `MEMORY-STATE-IDENTITY.3`

- ID: `MEMORY-STATE-IDENTITY.1`
  Status: `done`
  Goal: `Design the reset-defined memory proof boundary.`
  Acceptance: `The task tree and design notes record whether ANVIL can add a synthesizable reset-defined memory template suitable for sharing, plus the next executable implementation leaf or a blocker.`
  Verification: `current memory template audit; /tmp/anvil-reset-mem-probe.sv Verilator/Yosys reset-all probe; memory/knowledge-map checks`
  Commit: `f962e6e`

- ID: `MEMORY-STATE-IDENTITY.2`
  Status: `done`
  Goal: `Record the reset-defined memory identity blocker for the current memory-inference lane.`
  Acceptance: `No source behavior changes land; the task tree and user-facing docs retain the reset-less instance-local boundary and record the reset-all-array Yosys warning / register-lowering evidence.`
  Verification: `blocker-record audit; scripts/check_memory_architecture.sh; knowledge-map/scripts/check_knowledge_map.sh; git diff --check`
  Commit: `ca6a449`

- ID: `MEMORY-STATE-IDENTITY.3`
  Status: `done`
  Goal: `Close the memory-state identity frontier.`
  Acceptance: `The tree records landed memory identity behavior or explicit blocker evidence, and the current reset-less boundary remains documented.`
  Verification: `closeout audit; scripts/check_memory_architecture.sh; knowledge-map/scripts/check_knowledge_map.sh; git diff --check`
  Commit: `pending this commit`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| _none_ | _complete_ | _done_ | Reset-less memory identity remains instance-local; reset-defined memory sharing is blocked for the current warning-clean memory-inference lane. |

## Decisions

- `2026-06-05`: Keep the existing reset-less inferrable memory
  template instance-local until a reset-defined template exists and is
  downstream-clean.
- `2026-06-05`: A reset-all unpacked-array template is syntactically
  accepted by Verilator 5.046, but Yosys 0.64 warns
  `Replacing memory \mem with list of registers` and lowers the 16x8
  probe to flip-flop/register logic rather than preserving the current
  warning-clean `$mem_v2` memory-inference lane. That is not acceptable
  as a silent replacement for ANVIL's inferrable-memory motif.

## Open Questions

- None for the current frontier.

## Blockers

- Current `Memory` has no reset/init field, and the current emitter is
  intentionally a reset-less synchronous write/read template.
- Resetting all array contents in the memory `always_ff` makes Yosys
  replace the memory with registers and emit a warning, which violates
  ANVIL's warning-clean downstream contract for this lane.
- A future reset-defined register-file artifact may be possible, but it
  would be a distinct motif/knob with explicit docs and downstream
  evidence, not a merge of the current reset-less memory template.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-06-05` | `MEMORY-STATE-IDENTITY.1` | `current memory template audit; Verilator reset-all probe; Yosys reset-all probe; scripts/check_memory_architecture.sh; knowledge-map/scripts/check_knowledge_map.sh; mdbook build book; git diff --check` | `passed` |
| `2026-06-05` | `MEMORY-STATE-IDENTITY.2` | `blocker-record audit; scripts/check_memory_architecture.sh; knowledge-map/scripts/check_knowledge_map.sh; git diff --check` | `passed` |
| `2026-06-05` | `MEMORY-STATE-IDENTITY.3` | `closeout audit; scripts/check_memory_architecture.sh; knowledge-map/scripts/check_knowledge_map.sh; git diff --check` | `passed` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `MEMORY-STATE-IDENTITY.1` | `f962e6e MEMORY-STATE-IDENTITY.1 - design memory proof boundary` | `Reset-defined memory proof boundary and blocker evidence.` |
| `MEMORY-STATE-IDENTITY.2` | `ca6a449 MEMORY-STATE-IDENTITY.2 - record reset-memory blocker` | `Record blocker; no source behavior change.` |
| `MEMORY-STATE-IDENTITY.3` | `pending this commit` | `Close memory-state frontier.` |

## Changelog

- `2026-06-05`: Created task tree and opened
  `MEMORY-STATE-IDENTITY.1`.
- `2026-06-05`: Completed `MEMORY-STATE-IDENTITY.1` design boundary.
  Reset-all array contents are not warning-clean `$mem_v2` memory
  inference in Yosys; current reset-less memories remain instance-local.
- `2026-06-05`: Completed `MEMORY-STATE-IDENTITY.2` as a blocker
  record. No reset-defined memory merge is implemented for the current
  memory-inference lane.
- `2026-06-05`: Completed `MEMORY-STATE-IDENTITY.3` closeout. The
  tree is closed with current reset-less memories remaining
  instance-local and reset-defined memory sharing blocked for the
  current warning-clean memory-inference lane.
