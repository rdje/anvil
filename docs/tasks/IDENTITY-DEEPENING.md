# IDENTITY-DEEPENING: Advance NodeId-as-Identity / Full-Factorization

## Metadata

- Tree ID: `IDENTITY-DEEPENING`
- Status: `active`
- Roadmap lane: `NodeId as identity / full-factorization deepening`
- Created: `2026-06-15`
- Last updated: `2026-06-15`
- Owner: repo-local workflow
- Note: `.2` (bisimulation flop merge) **delivered** (`.2a` design + `.2b`
  impl). `.3` (whole-module sequential equivalence) is now split into `.3a`
  (design/decision — **done**, decision `0008`) + `.3b` (impl — future). Tree
  stays `active` until `.3b` lands or is deferred.

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
  Status: `done`
  Goal: `Implement the bounded bisimulation flop merge designed in decision 0007.`
  Children: `IDENTITY-DEEPENING.2a`, `IDENTITY-DEEPENING.2b`
  Result: `Both children done. The opt-in merge_bisimilar_flops pass is live (default-off / byte-identical), banked downstream-clean across Verilator + both Yosys modes + Icarus.`

- ID: `IDENTITY-DEEPENING.2a`
  Status: `done`
  Goal: `Design-detail leaf: ground decision 0007 in the real merge machinery (src/ir/compact.rs merge_equivalent_flops + cone_proof/semantic_cone_proof + the FlopSignature path), and pin the exact algorithm, API reuse, knob/metric/field names, budget caps, refinement-memo gotcha, pass ordering, and gate scenario for .2b — no source change.`
  Acceptance: `DEVELOPMENT_NOTES.md records the grounded algorithm + the shared-finalize refactor + the quotient-signature mechanism + the bucket cap + the refinement-memo-clear gotcha + the rules-first gate scenario; no source change; docs/workflow self-checks clean.`
  Result: `New pass merge_bisimilar_flops beside merge_equivalent_flops (NOT a modification of it), gated on a new Module flag mirrored from Config::bisimulation_flop_merge + node-id/e-graph; runs AFTER the exact flop merge, BEFORE FSM merge/compaction in generate_leaf_module. Bucket by (width, reset_kind, reset_val, clock_domain); greatest-fixpoint partition refinement keyed on a QUOTIENT D-signature (the existing cone_proof but with every LeafEndpoint::FlopQ{flop} canonicalized to its current class representative); reset-defined self-hold and same-endpoint cones fall out as special cases. Reuse MERGE_SEMANTIC_LIMITS (12-bit/128-node/131072-work) per D-cone check + a bucket-size cap N_bisim_flops (default 64) to bound O(k²·iters); over-budget cones take the structural fallback (quotient-aware). Extract the post-old_to_canonical_old rewire/renumber/remap/remap_explicit_flop_domains_after_merge/rebuild_instance_tables tail of merge_equivalent_flops into a shared finalize_flop_merge helper reused by both passes (keeps merge_equivalent_flops byte-identical). New Module::bisimulation_flops_merged -> Metrics::bisimulation_flops_merged. Refinement-memo gotcha: structural_memo/semantic_memo/endpoint_memo are NodeId-keyed and assume fixed endpoints, so they MUST be rebuilt each refinement iteration (the class map changes between iterations). Gate = a rules-first compact.rs test with flops f,g where D_f=Q_g, D_g=Q_f (mutual swap, same width/reset/domain, each observed by an output): assert merge_equivalent_flops removes 0 (exact pass can't), then merge_bisimilar_flops removes 1; plus knob-off snapshot 6/6 byte-identical; .2b decides dedicated tool_matrix scenario vs focused test + manual Verilator/Yosys smoke for the downstream-clean bank.`
  Verification: `done`
  Commit: `done`

- ID: `IDENTITY-DEEPENING.2b`
  Status: `done`
  Goal: `Implement merge_bisimilar_flops per the .2a design: the shared finalize_flop_merge refactor (byte-identical), the quotient-signature partition refinement, the default-off Config::bisimulation_flop_merge knob threaded onto Module, the Metrics::bisimulation_flops_merged counter, the rules-first gate scenario, and the downstream-clean bank.`
  Acceptance: `cargo fmt/check/clippy clean; cargo test --lib + focused compact tests green incl. the mutual-swap proof and the knob-off byte-identical regression; cargo test --test snapshots 6/6 byte-identical (knob default off); merged output banked clean across Verilator + both Yosys modes; live docs (book/src/factorization.md "broader sequential equivalence" + sequential.md, DEVELOPMENT_NOTES.md, ROADMAP gap 2, CODEBASE_ANALYSIS.md, USER_GUIDE/knobs for the new flag) + a Knowledge Map card updated; committed through COMMIT.md with the leaf id.`
  Result: `Landed merge_bisimilar_flops + finalize_flop_merge refactor + quotient-aware proof threading (canonical_flop_endpoint) + default-off Config/Module bisimulation_flop_merge knob + Metrics::bisimulation_flops_merged. Discovered and fixed a soundness gap not in .2a: resetless flops have no base case and must be excluded from refinement (preserves the resetless-self-hold boundary). 6 rules-first gate tests (mutual-swap merge, exact-pass-cannot, default-off, resetless-excluded, relaxed off-switch, e-graph gate, non-bisimilar split). Schema MINOR bump 1.0->1.1 (new Metrics field). Banked downstream-clean: Verilator --lint-only -Wall 0 warnings + Yosys both modes + Icarus on the merged self-hold output.`
  Verification: `done`
  Commit: `done`

- ID: `IDENTITY-DEEPENING.3`
  Status: `active`
  Goal: `Whole stateful-leaf-module bounded sequential equivalence built on the .2 flop-bisimulation primitive + a bounded cross-module state-correspondence search, extending dedup_semantic_modules past today's pure-combinational boundary.`
  Children: `IDENTITY-DEEPENING.3a`, `IDENTITY-DEEPENING.3b`

- ID: `IDENTITY-DEEPENING.3a`
  Status: `done`
  Goal: `Design/decision leaf: fix the soundness discipline + budget + downstream gate for whole-leaf-module sequential equivalence, grounded in the real dedup_semantic_modules / merge_bisimilar_flops machinery, and split .3 — before any code.`
  Acceptance: `A decision record naming the approach, its soundness argument, its budget, its default-off control surface, and its downstream gate; the first-cut scope + excluded boundaries; the central .3b impl challenge; no source change; docs/workflow self-checks clean.`
  Result: `Decision 0008 — second extension = bounded whole-leaf-module sequential equivalence via CROSS-MODULE bisimulation (lift the .2 partition-refinement primitive to the disjoint union of two candidate modules' flops, primary inputs unified by (PortId,width); prove a stable cross-module state correspondence, then prove every output-port cone equal under the resulting quotient; reuse the 12-bit/128-node/131072-work combinational budget). Added BESIDE dedup_semantic_modules (not a modification), default-off / byte-identical, node-id/e-graph; first cut = flops-only leaf modules (memory/FSM/instance/param/aggregate modules excluded as named boundaries; resetless flops excluded — no base case, carrying the .2b fix forward). Strictly generalizes the pure-combinational dedup_semantic_modules (zero-flop special case) and the flop-level classes; retires nothing. Central .3b challenge recorded: a cross-module cone-proof signature (shared LeafEndpoint vocabulary — PrimaryInput by (PortId,width), FlopQ by global union class id). Tree split into .3a (this, done) + .3b (impl, future).`
  Verification: `done`
  Commit: `done`

- ID: `IDENTITY-DEEPENING.3b`
  Status: `proposed`
  Goal: `(Future) Implement the bounded whole-leaf-module sequential-equivalence pass per decision 0008: cross-module union partition refinement + cross-module cone-proof signature + bounded output-cone equality + the new default-off hierarchy_sequential_module_dedup knob + the design-level sequential_module_proof / num_sequentially_duplicate_module_pairs metric pair + the focused downstream-clean gate; default-off / byte-identical; banked clean across Verilator + both Yosys modes.`
  Acceptance: `Design leaf (0008) landed first; rules-first cross-module merge gate (two sequentially-equivalent stateful leaf modules collapse to one definition with the knob on, both structural + combinational dedup leave 2); knob-off byte-identical (snapshots 6/6); merged multi-module design downstream-clean; live docs + KM card updated; committed through COMMIT.md with the leaf id.`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `IDENTITY-DEEPENING.3b` | `proposed` | Now eligible — `.3a` design landed (decision `0008`). Implement the bounded whole-leaf-module sequential-equivalence pass (cross-module bisimulation + output-cone equality), default-off / byte-identical, downstream-clean bank. |
| — | `IDENTITY-DEEPENING.3a` | `done` | Landed decision `0008` — soundness discipline, budget, control surface, downstream gate, first-cut scope, and the central `.3b` cross-module-proof challenge; no source change. |
| — | `IDENTITY-DEEPENING.2b` | `done` | Delivered the opt-in `merge_bisimilar_flops` pass, default-off / byte-identical, downstream-clean bank. |

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
- `2026-06-15` (`.3a`, decision [`0008`](../decisions/0008-identity-deepening-whole-module-sequential-equivalence.md)):
  Split `.3` into `.3a` (design, done) + `.3b` (impl, future). Second extension
  = **bounded whole-leaf-module sequential equivalence via cross-module
  bisimulation**. Rationale: the `.2` primitive proves flops equivalent *within
  one module*; `dedup_semantic_modules` proves *whole modules* equivalent but
  only stateless ones — the open frontier exactly between them is proving two
  whole *stateful* leaf modules observationally equivalent. The pick lifts the
  flop-level partition refinement to the disjoint union of two modules' flops
  (primary inputs unified by `(PortId, width)`), finds a stable cross-module
  state correspondence, then proves every output cone equal under the resulting
  quotient. Soundness = reset base case + bisimulation step (coinduction), the
  same discipline as `.2`, now across two machines. It strictly generalizes the
  pure-combinational `dedup_semantic_modules` (zero-flop special case) without
  retiring it, and is added **beside** it (not a modification — the `.2b`
  precedent). First cut = flops-only leaf modules; memory/FSM/instance/param/
  aggregate modules stay excluded as named boundaries. Rejected as the approach:
  reachable-product / bounded model checking (unsound merge proof), unifying into
  `dedup_semantic_modules` (byte-identical risk), and any structural/syntactic
  module resemblance (`hierarchy-identity-boundary`).

## Open Questions

- `.3b` finalizes: the knob name (`hierarchy_sequential_module_dedup`), the
  design-level metric names (`sequential_module_proof` signature +
  `num_sequentially_duplicate_module_pairs`), the union flop cap
  `N_bisim_module_flops`, the exact cross-module cone-proof signature
  representation (the central impl challenge — a shared `LeafEndpoint` vocabulary
  keyed by `(PortId, width)` for inputs and a global union class id for `FlopQ`),
  and the downstream-clean gate shape (focused `cargo test` + smoke vs a
  dedicated `tool_matrix` scenario set), by whichever proves the cross-module
  stateful merge by construction at lowest cost (mirrors the `.2b` precedent).

## Blockers

- None. (Sequenced after Lanes 2–3 by choice, not by dependency.)

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-06-15` | `IDENTITY-DEEPENING.1` | Design/decision leaf, no source change. `bash scripts/check_memory_architecture.sh` + `bash knowledge-map/scripts/check_knowledge_map.sh` clean; `KNOWLEDGE_MAP.md` regenerated to include decision `0007` answers. | `done` |
| `2026-06-15` | `IDENTITY-DEEPENING.2a` | Design-detail leaf, no source change (grounded in a close read of `src/ir/compact.rs` `merge_equivalent_flops`/`flop_d_signature`/`cone_proof`/`semantic_cone_proof`, `src/config.rs` knob pattern, `src/metrics.rs` merge-count pattern). Self-checks clean. | `done` |
| `2026-06-15` | `IDENTITY-DEEPENING.2b` | `cargo build` + `cargo clippy --all-targets -- -D warnings` clean; `cargo fmt --all --check` clean; `cargo test --lib` 403 pass incl. 6 new bisim gate tests + the metrics-plumbing test + introspect/MCP schema_version tests; `cargo test --test snapshots` 6/6 byte-identical (knob default-off); representative `cargo test --test pipeline` reproducibility test green (full pipeline suite is heavy/slow and exercises only the byte-identical default path). Downstream-clean bank on the merged mutual-swap self-hold output: Verilator `--lint-only -Wall` 0 warnings, Yosys without-abc + with-abc, Icarus `iverilog -g2012` (re-bank: `ANVIL_DUMP_BISIM_SV=1 cargo test --lib merge_bisimilar_flops_merges_mutual_swap_registers`). KM + mem-arch self-checks clean. | `done` |
| `2026-06-15` | `IDENTITY-DEEPENING.3a` | Design/decision leaf, **no source change** (grounded in a close read of `src/ir/dedup.rs` `dedup_semantic_modules` + `prune_modules_made_unreachable`, `src/metrics.rs` `semantic_module_proof_inner` + the module-proof budget constants, and `src/ir/compact.rs` `merge_bisimilar_flops` / `canonical_flop_endpoint` / `cone_proof` / `MERGE_SEMANTIC_LIMITS`). Decision `0008` written with KM `answers:` front-matter; `KNOWLEDGE_MAP.md` regenerated; `bash scripts/check_memory_architecture.sh` + `bash knowledge-map/scripts/check_knowledge_map.sh` clean. | `done` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `IDENTITY-DEEPENING.1` | `IDENTITY-DEEPENING.1 — promote Lane 1 + decision 0007` | Landed `43e2a2d`. Decision record `0007`; tree split into `.2`/`.3`. |
| `IDENTITY-DEEPENING.2a` | `IDENTITY-DEEPENING.2a — bisimulation flop merge design detail` | Grounded `.2b` algorithm/API-reuse/names/budget/gate; `.2` split into `.2a`/`.2b`. |
| `IDENTITY-DEEPENING.2b` | `IDENTITY-DEEPENING.2b — implement bounded bisimulation flop merge` | Landed `merge_bisimilar_flops` + `finalize_flop_merge` + quotient proof threading + default-off knob + metric + 6 gate tests + schema 1.0→1.1; downstream-clean bank. Closes `.2`. |
| `IDENTITY-DEEPENING.3a` | `IDENTITY-DEEPENING.3a — whole-module sequential equivalence design` | Decision record `0008`; split `.3` into `.3a` (design, done) + `.3b` (impl, future). No source change. |

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
- `2026-06-15`: `.2b` done — implemented `merge_bisimilar_flops` (opt-in,
  default-off / byte-identical), the shared `finalize_flop_merge` refactor,
  the quotient-aware proof threading (`canonical_flop_endpoint`), the
  `Config`/`Module` `bisimulation_flop_merge` knob, and
  `Metrics::bisimulation_flops_merged`. Found + fixed a soundness gap beyond
  `.2a`: resetless flops have no base case and are excluded from refinement.
  6 rules-first gate tests; schema MINOR bump 1.0→1.1; banked downstream-clean
  (Verilator + both Yosys + Icarus). `.2` container closed. Frontier advances
  to `.3` (whole-module sequential equivalence, design leaf first).
- `2026-06-15`: `.3a` done — split `.3` into `.3a` (design/decision, done) +
  `.3b` (impl, future); `.3` is now an `active` container. Landed decision
  `0008` (second extension = bounded whole-leaf-module sequential equivalence
  via cross-module bisimulation: lift the `.2` union partition refinement across
  two candidate modules with primary inputs unified by `(PortId, width)`, prove
  a stable cross-module state correspondence, then prove every output cone equal
  under the quotient; reuse the 12-bit/128-node/131072-work combinational
  budget; added beside `dedup_semantic_modules`, default-off / byte-identical,
  node-id/e-graph; first cut flops-only — memory/FSM/instance/param/aggregate
  excluded; resetless excluded). Grounded in the real `dedup_semantic_modules` /
  `merge_bisimilar_flops` machinery; no source change. Frontier advances to
  `.3b`.
