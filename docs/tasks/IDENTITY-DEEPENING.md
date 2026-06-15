# IDENTITY-DEEPENING: Advance NodeId-as-Identity / Full-Factorization

## Metadata

- Tree ID: `IDENTITY-DEEPENING`
- Status: `active`
- Roadmap lane: `NodeId as identity / full-factorization deepening`
- Created: `2026-06-15`
- Last updated: `2026-06-15`
- Owner: repo-local workflow

## Goal

Advance the NodeId-as-identity / full-factorization north star (the
strong-form `ROADMAP.md` steering gap 2, and `feedback_full_factorization`)
into the currently-open territory left explicitly bounded by the closed
identity trees:

- **hierarchical / module semantic identity** under
  `identity_mode = node-id` beyond today's canonical *structural* module
  signatures (the boundary recorded by `hierarchy-identity-boundary`);
- **broader sequential equivalence** beyond the current exact
  reset-defined self-hold + deterministic-FSM merge classes (the
  boundary recorded by `reset-defined-self-hold-flop-identity` /
  `fsm-identity-merge`).

The doctrinal bar is unchanged and strong: two structures share one
identity **only** when ANVIL can *prove* they implement the same
functionality with respect to the same canonical leaf endpoints — never
mere syntactic resemblance. This lane is Lane 1 of the three
owner-directed post-phase capability lanes; it is opened `proposed` and
promoted to `active` after `SIGNOFF-AUTOMATION-EXPANSION` reaches
handoff.

## Non-Goals

- No relaxation of the proof discipline into syntactic resemblance, and
  no unbounded or unsound merges. Proofs stay bounded by the existing
  support / node / work budgets (`semantic-proof-budget`); larger cones
  fall back to structural identity rather than guessing.
- No generate-then-filter — identity is a construction/finalization-time
  property, never a post-hoc dedup of arbitrary text
  (`feedback_rules_first_generation`).
- No removal of the `--identity-mode relaxed` real off-switch, and no
  redefinition of what `node-id` means via `--factorization-level`
  (which stays a proof-depth dial).
- Does not merge instance-local memories whose contents are not
  reset-defined — that boundary (`memory-identity-boundary`) stays as
  proven, not reopened here unless a new sound proof class is
  established.
- Does not retire any landed identity/merge strategy
  (`feedback_never_retire_strategies`).

## Acceptance Criteria

- Each landed leaf either proves a new **sound** identity/merge class
  (with the proof discipline, budget, and a downstream-clean gate) or
  documents a new explicit boundary with a reproducible probe — both are
  legitimate outcomes for this lane.
- Default-off / byte-identical wherever a new merge could change emitted
  RTL under the relaxed default.
- A Knowledge Map card captures each new durable identity fact or
  boundary so it is never re-derived.
- Live docs (`book/src/factorization.md`, `DEVELOPMENT_NOTES.md`,
  `ROADMAP.md` steering gap 2, `CODEBASE_ANALYSIS.md`) updated where the
  proof surface changes.
- Every leaf committed through `COMMIT.md` with its leaf ID in the
  subject.

## Task Tree

- ID: `IDENTITY-DEEPENING`
  Status: `active`
  Goal: `Advance NodeId identity into hierarchical/module semantic equivalence and broader sequential equivalence.`
  Children: `IDENTITY-DEEPENING.1`, `IDENTITY-DEEPENING.2`, `IDENTITY-DEEPENING.3`

- ID: `IDENTITY-DEEPENING.1`
  Status: `done`
  Goal: `Design/decision leaf: pick the first concrete sound identity extension, define its proof discipline + budget + downstream gate, and split the tree.`
  Acceptance: `A decision record naming the chosen first extension, its soundness argument, and its budget; no source change; docs/workflow validation clean.`
  Result: `Decision 0007 — first extension = bounded bisimulation-based sequential flop equivalence (greatest-fixpoint partition refinement; reuses the bounded combinational endpoint proof up to a state correspondence; default-off knob + node-id/e-graph; captures the recorded mutually-recursive-register / non-exact-feedback no-merge boundary soundly via reset-base-case coinduction). Tree split into .2 (impl) + .3 (future module-level sequential equivalence).`
  Verification: `done`
  Commit: `done`

- ID: `IDENTITY-DEEPENING.2`
  Status: `pending`
  Goal: `Implement the bounded bisimulation flop merge: greatest-fixpoint partition refinement over flops (bucketed by width/reset_kind/reset_val/flop_domain) + bounded quotient D-cone equivalence proof (reusing the existing combinational budget) + a new default-off Config knob (working name bisimulation_flop_merge, requires node-id/e-graph) + a merge-count metric (working name bisimulation_flops_merged) + a focused downstream-clean gate.`
  Acceptance: `Knob-off byte-identical (snapshots untouched); existing exact self-hold / same-endpoint / FSM merges still fire; a rules-first scenario with deliberately-duplicated mutually-recursive register pairs proves merge-count > 0 with the knob on AND the merged output clean across Verilator + both Yosys modes (banked report); soundness regression-protected; live docs (book/src/factorization.md, DEVELOPMENT_NOTES.md, ROADMAP gap 2, CODEBASE_ANALYSIS.md) + a Knowledge Map card updated. Split into .2a design-detail + .2b impl if it proves broad.`
  Verification: `pending`
  Commit: `pending`

- ID: `IDENTITY-DEEPENING.3`
  Status: `proposed`
  Goal: `(Future) Whole stateful-leaf-module bounded sequential equivalence built on the .2 flop-bisimulation primitive + a bounded state-correspondence search, extending dedup_semantic_modules past today's pure-combinational boundary.`
  Acceptance: `Design leaf first (soundness + budget + gate) before any code; named future leaf, not on the active frontier until .2 reaches handoff.`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `IDENTITY-DEEPENING.2` | `pending` | First code leaf: implement the bounded bisimulation flop merge designed in `.1` (decision `0007`). Default-off / byte-identical; reuses the bounded combinational proof; banked downstream-clean gate. Split into `.2a`/`.2b` if broad. |
| — | `IDENTITY-DEEPENING.3` | `proposed` | Not on the active frontier yet; module-level sequential equivalence activates after `.2` reaches handoff. |

## Decisions

- `2026-06-15`: Opened `proposed` as Lane 1 (execution order `2 → 3 →
  1`), sequenced last because it is the deepest, most open-ended axis and
  benefits from the richer proof tooling that Lanes 2–3 build. The first
  leaf is a design/decision leaf: soundness and budget must be designed
  before any merge code lands.
- `2026-06-15` (`.1`, decision [`0007`](../decisions/0007-identity-deepening-first-extension.md)):
  Promoted to `active`. First extension = **bounded bisimulation-based
  sequential flop equivalence**. Rationale: bounded *module-level* semantic
  equivalence already exists for the pure-combinational case
  (`dedup_semantic_modules`); the genuinely open, high-value, soundly-bounded
  frontier is *sequential*. The pick lifts the recorded
  mutually-recursive-register / non-exact-feedback no-merge boundary at the flop
  level via a greatest-fixpoint partition refinement that reuses the existing
  bounded combinational endpoint proof up to a state correspondence. Soundness =
  reset base case + bisimulation step (coinduction); it strictly generalizes the
  exact self-hold and same-endpoint classes without retiring them. Rejected as
  first: whole stateful-module reachable-product equivalence (bigger jump → `.3`
  future), bounded model checking (unsound merge proof), retimed-state
  equivalence (not bisimilar), and memory-state merging
  (`memory-identity-boundary`, blocked).

## Open Questions

- `.2` finalizes the knob name (`bisimulation_flop_merge`), the merge-count
  metric name (`bisimulation_flops_merged`), the bucket-size cap
  `N_bisim_flops`, and the exact scenario/gate shape (focused `cargo test` +
  smoke vs a dedicated `tool_matrix` scenario set).

## Blockers

- None. (Sequenced after Lanes 2–3 by choice, not by dependency.)

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-06-15` | `IDENTITY-DEEPENING.1` | Design/decision leaf, no source change. `bash scripts/check_memory_architecture.sh` + `bash knowledge-map/scripts/check_knowledge_map.sh` clean; `KNOWLEDGE_MAP.md` regenerated to include decision `0007` answers. | `done` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `IDENTITY-DEEPENING.1` | `IDENTITY-DEEPENING.1 — promote lane + decision 0007 (bisimulation flop equivalence)` | Decision record `0007`; tree split into `.2`/`.3`. |

## Changelog

- `2026-06-15`: Created task tree (Lane 1), opened `proposed`, via
  `CAPABILITY-LANE-OWNERSHIP.1`.
- `2026-06-15`: `.1` done — promoted tree to `active`, landed decision `0007`
  (first extension = bounded bisimulation-based sequential flop equivalence),
  split the tree into `.2` (impl) + `.3` (future module-level sequential
  equivalence); frontier advances to `.2`.
