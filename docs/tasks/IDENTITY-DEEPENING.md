# IDENTITY-DEEPENING: Advance NodeId-as-Identity / Full-Factorization

## Metadata

- Tree ID: `IDENTITY-DEEPENING`
- Status: `proposed`
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
  Status: `proposed`
  Goal: `Advance NodeId identity into hierarchical/module semantic equivalence and broader sequential equivalence.`
  Children: `IDENTITY-DEEPENING.1`

- ID: `IDENTITY-DEEPENING.1`
  Status: `pending`
  Goal: `Design/decision leaf: pick the first concrete sound identity extension (e.g. bounded semantic module equivalence beyond structural signatures, or a broader bounded sequential class), define its proof discipline + budget + downstream gate, and split the tree.`
  Acceptance: `A decision record naming the chosen first extension, its soundness argument, and its budget; no source change; docs/workflow validation clean.`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| — | `IDENTITY-DEEPENING.1` | `pending` | Not on the active frontier yet; this lane activates after `SIGNOFF-AUTOMATION-EXPANSION` reaches handoff. |

## Decisions

- `2026-06-15`: Opened `proposed` as Lane 1 (execution order `2 → 3 →
  1`), sequenced last because it is the deepest, most open-ended axis and
  benefits from the richer proof tooling that Lanes 2–3 build. The first
  leaf is a design/decision leaf: soundness and budget must be designed
  before any merge code lands.

## Open Questions

- `.1` decides the first extension. Two strong candidates: (a) bounded
  semantic module equivalence (merge structurally-different but
  provably-equivalent bounded combinational module bodies, extending
  `bounded-semantic-module-identity`), and (b) a broader bounded
  sequential equivalence class beyond exact reset-defined self-hold. The
  deciding factor is which yields a sound, budget-bounded proof with a
  clean downstream gate at acceptable cost.

## Blockers

- None. (Sequenced after Lanes 2–3 by choice, not by dependency.)

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-06-15` | `IDENTITY-DEEPENING.1` | `pending` | `pending` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `IDENTITY-DEEPENING.1` | `pending` | `pending` |

## Changelog

- `2026-06-15`: Created task tree (Lane 1), opened `proposed`, via
  `CAPABILITY-LANE-OWNERSHIP.1`.
