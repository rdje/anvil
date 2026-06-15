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
  Status: `active`
  Goal: `Implement the bounded bisimulation flop merge designed in decision 0007.`
  Children: `IDENTITY-DEEPENING.2a`, `IDENTITY-DEEPENING.2b`

- ID: `IDENTITY-DEEPENING.2a`
  Status: `done`
  Goal: `Design-detail leaf: ground decision 0007 in the real merge machinery (src/ir/compact.rs merge_equivalent_flops + cone_proof/semantic_cone_proof + the FlopSignature path), and pin the exact algorithm, API reuse, knob/metric/field names, budget caps, refinement-memo gotcha, pass ordering, and gate scenario for .2b — no source change.`
  Acceptance: `DEVELOPMENT_NOTES.md records the grounded algorithm + the shared-finalize refactor + the quotient-signature mechanism + the bucket cap + the refinement-memo-clear gotcha + the rules-first gate scenario; no source change; docs/workflow self-checks clean.`
  Result: `New pass merge_bisimilar_flops beside merge_equivalent_flops (NOT a modification of it), gated on a new Module flag mirrored from Config::bisimulation_flop_merge + node-id/e-graph; runs AFTER the exact flop merge, BEFORE FSM merge/compaction in generate_leaf_module. Bucket by (width, reset_kind, reset_val, clock_domain); greatest-fixpoint partition refinement keyed on a QUOTIENT D-signature (the existing cone_proof but with every LeafEndpoint::FlopQ{flop} canonicalized to its current class representative); reset-defined self-hold and same-endpoint cones fall out as special cases. Reuse MERGE_SEMANTIC_LIMITS (12-bit/128-node/131072-work) per D-cone check + a bucket-size cap N_bisim_flops (default 64) to bound O(k²·iters); over-budget cones take the structural fallback (quotient-aware). Extract the post-old_to_canonical_old rewire/renumber/remap/remap_explicit_flop_domains_after_merge/rebuild_instance_tables tail of merge_equivalent_flops into a shared finalize_flop_merge helper reused by both passes (keeps merge_equivalent_flops byte-identical). New Module::bisimulation_flops_merged -> Metrics::bisimulation_flops_merged. Refinement-memo gotcha: structural_memo/semantic_memo/endpoint_memo are NodeId-keyed and assume fixed endpoints, so they MUST be rebuilt each refinement iteration (the class map changes between iterations). Gate = a rules-first compact.rs test with flops f,g where D_f=Q_g, D_g=Q_f (mutual swap, same width/reset/domain, each observed by an output): assert merge_equivalent_flops removes 0 (exact pass can't), then merge_bisimilar_flops removes 1; plus knob-off snapshot 6/6 byte-identical; .2b decides dedicated tool_matrix scenario vs focused test + manual Verilator/Yosys smoke for the downstream-clean bank.`
  Verification: `done`
  Commit: `done`

- ID: `IDENTITY-DEEPENING.2b`
  Status: `pending`
  Goal: `Implement merge_bisimilar_flops per the .2a design: the shared finalize_flop_merge refactor (byte-identical), the quotient-signature partition refinement, the default-off Config::bisimulation_flop_merge knob threaded onto Module, the Metrics::bisimulation_flops_merged counter, the rules-first gate scenario, and the downstream-clean bank.`
  Acceptance: `cargo fmt/check/clippy clean; cargo test --lib + focused compact tests green incl. the mutual-swap proof and the knob-off byte-identical regression; cargo test --test snapshots 6/6 byte-identical (knob default off); merged output banked clean across Verilator + both Yosys modes; live docs (book/src/factorization.md "broader sequential equivalence" + sequential.md, DEVELOPMENT_NOTES.md, ROADMAP gap 2, CODEBASE_ANALYSIS.md, USER_GUIDE/knobs for the new flag) + a Knowledge Map card updated; committed through COMMIT.md with the leaf id.`
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
| 1 | `IDENTITY-DEEPENING.2b` | `pending` | The code leaf: implement `merge_bisimilar_flops` per the `.2a` grounded design (shared `finalize_flop_merge` refactor + quotient-signature partition refinement + default-off knob + metric + rules-first gate + downstream-clean bank). |
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

- `.2a` pinned the names (`Config::bisimulation_flop_merge`,
  `Metrics::bisimulation_flops_merged`) and the bucket cap (`N_bisim_flops`
  default `64`). Remaining for `.2b`: whether the downstream-clean bank is a
  dedicated `tool_matrix` scenario set or a focused `cargo test` + a manual
  Verilator/Yosys smoke (decide by whichever proves the mutual-recursion merge
  by construction at lowest cost, mirroring the `signoff-knob-sweep` precedent),
  and the precise threading of the new `Module` flag from `Config` (alongside
  `identity_mode`).

## Blockers

- None. (Sequenced after Lanes 2–3 by choice, not by dependency.)

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-06-15` | `IDENTITY-DEEPENING.1` | Design/decision leaf, no source change. `bash scripts/check_memory_architecture.sh` + `bash knowledge-map/scripts/check_knowledge_map.sh` clean; `KNOWLEDGE_MAP.md` regenerated to include decision `0007` answers. | `done` |
| `2026-06-15` | `IDENTITY-DEEPENING.2a` | Design-detail leaf, no source change (grounded in a close read of `src/ir/compact.rs` `merge_equivalent_flops`/`flop_d_signature`/`cone_proof`/`semantic_cone_proof`, `src/config.rs` knob pattern, `src/metrics.rs` merge-count pattern). Self-checks clean. | `done` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `IDENTITY-DEEPENING.1` | `IDENTITY-DEEPENING.1 — promote Lane 1 + decision 0007` | Landed `43e2a2d`. Decision record `0007`; tree split into `.2`/`.3`. |
| `IDENTITY-DEEPENING.2a` | `IDENTITY-DEEPENING.2a — bisimulation flop merge design detail` | Grounded `.2b` algorithm/API-reuse/names/budget/gate; `.2` split into `.2a`/`.2b`. |

## Changelog

- `2026-06-15`: Created task tree (Lane 1), opened `proposed`, via
  `CAPABILITY-LANE-OWNERSHIP.1`.
- `2026-06-15`: `.1` done — promoted tree to `active`, landed decision `0007`
  (first extension = bounded bisimulation-based sequential flop equivalence),
  split the tree into `.2` (impl) + `.3` (future module-level sequential
  equivalence); frontier advances to `.2`.
- `2026-06-15`: `.2a` done — split `.2` into `.2a` (design-detail, done) +
  `.2b` (impl); grounded the bisimulation algorithm, the shared
  `finalize_flop_merge` refactor, the quotient D-signature mechanism, the
  bucket cap + refinement-memo-clear gotcha, the pass ordering, and the
  rules-first mutual-swap gate scenario in the real `src/ir/compact.rs` code;
  frontier advances to `.2b`.
