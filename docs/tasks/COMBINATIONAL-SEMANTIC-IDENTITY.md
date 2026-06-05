# COMBINATIONAL-SEMANTIC-IDENTITY: Broader Combinational Semantic Identity

## Metadata

- Tree ID: `COMBINATIONAL-SEMANTIC-IDENTITY`
- Status: `active`
- Roadmap lane: `NodeId as identity / full-factorization mode`
- Created: `2026-06-05`
- Last updated: `2026-06-05`
- Owner: repo-local workflow

## Goal

Exhaust the next sound, bounded expansions of combinational semantic
identity under `identity_mode = node-id` and `factorization_level =
e-graph`, while preserving the existing canonical-endpoint boundary.

## Non-Goals

- No cross-endpoint merging.
- No unbounded SAT/SMT engine.
- No semantic rewrite that can change emitted RTL under
  `identity_mode = relaxed`.
- No performance regression from proof-budget expansion without a
  focused guard.

## Acceptance Criteria

- Every source edit is owned by a leaf before it occurs.
- The bounded semantic layer can collapse at least one currently-open
  same-endpoint identity class that the lower ladder does not already
  catch, or the leaf records a real blocker with evidence.
- Endpoint preservation remains covered by regression tests.
- Metrics, live docs, and mdBook explain any new merge/fold behavior.
- Focused checks pass, broader gates run when the blast radius warrants
  them, and each completed leaf is committed through `COMMIT.md`.
- The tree closes only when its frontier is empty because all known
  sound bounded expansions have either landed or been explicitly
  deferred with a proof-boundary reason.

## Task Tree

- ID: `COMBINATIONAL-SEMANTIC-IDENTITY`
  Status: `active`
  Goal: `Broaden same-endpoint combinational semantic identity.`
  Children: `COMBINATIONAL-SEMANTIC-IDENTITY.1`, `COMBINATIONAL-SEMANTIC-IDENTITY.2`, `COMBINATIONAL-SEMANTIC-IDENTITY.3`

- ID: `COMBINATIONAL-SEMANTIC-IDENTITY.1`
  Status: `done`
  Goal: `Land the first safe same-endpoint semantic fold beyond gate-to-gate merging.`
  Acceptance: `A gate whose bounded semantic proof equals an existing endpoint or constant is rewired to that existing node at the e-graph rung; endpoint-distinct no-merge tests still pass; docs describe the new fold boundary.`
  Verification: `cargo test -q merge_equivalent_gates`; compact semantic/flop/FSM focused tests; `cargo test -q --test snapshots` after deliberate snapshot review/acceptance; focused Verilator/Yosys smoke; `cargo check --all-targets`; `cargo clippy --all-targets -- -D warnings`; `cargo fmt --all --check`; `cargo test -q --test book_examples`; `mdbook build book`; `mdbook test book`; memory/Knowledge Map checks; `git diff --check`.
  Commit: `COMBINATIONAL-SEMANTIC-IDENTITY.1 - fold gates to endpoints`

- ID: `COMBINATIONAL-SEMANTIC-IDENTITY.2`
  Status: `done`
  Goal: `Audit and extend bounded proof budgets only where focused tests prove runtime stays controlled.`
  Acceptance: `Current hard proof limits are either raised with bounded tests and metrics coverage or kept with an explicit measured blocker.`
  Verification: `focused semantic/cleanup budget tests with /usr/bin/time -l measurements; merge gate regression; snapshots after deliberate review/acceptance; generated seed-42 Verilator/Yosys smoke; check/clippy/fmt; book gates; memory/Knowledge Map checks; git diff check.`
  Commit: `COMBINATIONAL-SEMANTIC-IDENTITY.2 - widen semantic proof budget safely`

- ID: `COMBINATIONAL-SEMANTIC-IDENTITY.3`
  Status: `pending`
  Goal: `Close the combinational semantic frontier.`
  Acceptance: `The task file records all landed expansions, any deferred proof limits, validation, and an empty frontier.`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `COMBINATIONAL-SEMANTIC-IDENTITY.3` | `pending` | With the gate-to-endpoint fold and bounded proof-budget expansion landed, close the combinational frontier and record any remaining proof boundaries. |

## Decisions

- `2026-06-05`: Start with proven gate-to-existing-node folds before
  increasing support limits. This expands visible identity behavior
  while preserving the endpoint discipline protected by
  `ENDPOINT-IDENTITY-BOUNDARY.1`.
- `2026-06-05`: Permit shallow 12 endpoint-support-bit semantic proofs
  only under a combined work budget. The flat support cap alone is not
  the safety invariant; `assignment_count * cone_node_count` is.

## Open Questions

- None for the current frontier.

## Blockers

- None.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-06-05` | `COMBINATIONAL-SEMANTIC-IDENTITY.2` | `cargo test -q semantic_merge_proof`; `cargo test -q cleanup_exact_proof`; `/usr/bin/time -l cargo test -q semantic_merge_proof`; `/usr/bin/time -l cargo test -q cleanup_exact_proof`; `cargo test -q merge_equivalent_gates`; `cargo test -q ir::compact::tests::semantic`; `cargo test -q --test snapshots` after deliberate snapshot review/acceptance; generated seed-42 node-id/e-graph smoke through Verilator and Yosys; `cargo check --all-targets`; `cargo clippy --all-targets -- -D warnings`; `cargo fmt --all --check`; `cargo test -q --test book_examples`; `mdbook build book`; `mdbook test book`; `scripts/check_memory_architecture.sh`; `knowledge-map/scripts/check_knowledge_map.sh`; `git diff --check` | passed; measured focused budget tests stayed below 46 MB max RSS; Yosys smoke reported 0 problems and 67.75 MB peak; full `cargo test` not run because this resource-sensitive slice is covered by focused compact tests, snapshots, book gates, and downstream smoke |
| `2026-06-05` | `COMBINATIONAL-SEMANTIC-IDENTITY.1` | `cargo test -q merge_equivalent_gates`; `cargo test -q ir::compact::tests::semantic`; `cargo test -q ir::compact::tests::merge_equivalent_flops`; `cargo test -q ir::compact::tests::merge_equivalent_fsms`; `cargo test -q --test snapshots` after deliberate snapshot review/acceptance; generated seed-42 node-id/e-graph smoke through Verilator and Yosys; `cargo check --all-targets`; `cargo clippy --all-targets -- -D warnings`; `cargo fmt --all --check`; `cargo test -q --test book_examples`; `mdbook build book`; `mdbook test book`; `scripts/check_memory_architecture.sh`; `knowledge-map/scripts/check_knowledge_map.sh`; `git diff --check` | passed; full `cargo test` not run because focused gates and snapshot/book/downstream checks covered this slice without invoking the resource-sensitive full suite |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `COMBINATIONAL-SEMANTIC-IDENTITY.2` | `COMBINATIONAL-SEMANTIC-IDENTITY.2 - widen semantic proof budget safely` | `pending hash`; advances frontier to `.3`. |
| `COMBINATIONAL-SEMANTIC-IDENTITY.1` | `COMBINATIONAL-SEMANTIC-IDENTITY.1 - fold gates to endpoints` | `41948a6`; advanced frontier to `.2`. |

## Changelog

- `2026-06-05`: Created task tree and opened
  `COMBINATIONAL-SEMANTIC-IDENTITY.1`.
- `2026-06-05`: Marked `COMBINATIONAL-SEMANTIC-IDENTITY.1` in
  progress for the gate-to-existing-node semantic fold.
- `2026-06-05`: Completed `COMBINATIONAL-SEMANTIC-IDENTITY.1` and
  advanced the frontier to `COMBINATIONAL-SEMANTIC-IDENTITY.2`.
- `2026-06-05`: Marked `COMBINATIONAL-SEMANTIC-IDENTITY.2` in
  progress for the bounded semantic proof-budget audit.
- `2026-06-05`: Completed `COMBINATIONAL-SEMANTIC-IDENTITY.2` and
  advanced the frontier to `COMBINATIONAL-SEMANTIC-IDENTITY.3`.
