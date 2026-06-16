# IDENTITY-DEEPENING: Advance NodeId-as-Identity / Full-Factorization

## Metadata

- Tree ID: `IDENTITY-DEEPENING`
- Status: `active`
- Roadmap lane: `NodeId as identity / full-factorization deepening`
- Created: `2026-06-15`
- Last updated: `2026-06-16`
- Owner: repo-local workflow
- Note: `.2` (bisimulation flop merge) **delivered** (`.2a` design + `.2b`
  impl). `.3` (whole-module sequential equivalence): `.3a` (design/decision —
  **done**, decision `0008`); `.3b` (impl): `.3b.1` (design-detail — **done**);
  `.3b.2` (impl) split into `.3b.2a` (the byte-identical `bisimulation_partition`
  refactor — **done**) + `.3b.2b` (the cross-module feature). `.3b.2b` is now an
  container split into `.3b.2b.1` (the merge mechanism + proof — **done**) +
  `.3b.2b.2` (the closeout = `.3b.2b.2a` metric/schema/bank **done** +
  `.3b.2b.2b` book/USER_GUIDE/ROADMAP/KM docs **done**). **`.3b.2b`, `.3b.2`,
  `.3b`, and `.3` are now all `done`** — the cross-module whole-leaf-module
  sequential-equivalence sub-tree is closed. The tree stays `active` as an
  open-ended capability lane with **no current frontier**; the deeper
  module-equivalence boundaries (memory / FSM / wrapper / retimed-state) are
  named, not-started future leaves (none retired).

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
  Status: `done`
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
  Status: `done`
  Goal: `Implement the bounded whole-leaf-module sequential-equivalence pass per decision 0008.`
  Children: `IDENTITY-DEEPENING.3b.1`, `IDENTITY-DEEPENING.3b.2`

- ID: `IDENTITY-DEEPENING.3b.1`
  Status: `done`
  Goal: `Design-detail leaf: ground decision 0008's "central impl challenge" (the cross-module cone-proof signature) in the real dedup_semantic_modules / merge_bisimilar_flops / cone_proof code, and pin the exact algorithm, API reuse, knob/metric/field names, refactor, soundness conditions, and gate for .3b.2 — no source change.`
  Acceptance: `DEVELOPMENT_NOTES.md records the grounded algorithm (combined-module materialization so (PortId,width) endpoint identity unifies inputs for free; reuse of the merge_bisimilar_flops refinement via a factored bisimulation_partition helper; the no-bijection-needed soundness refinement: interfaces match + stable union partition + per-output-port cone equality under the quotient; the pre-filter + union-find grouping that fits dedup_semantic_modules_once's survivor/rewrite/prune tail) + the knob/metric names + the gate; no source change; docs/workflow self-checks clean.`
  Result: `Pinned: (1) NO new cross-module proof engine — materialize a temporary combined Module = A.nodes ++ B.nodes (B's NodeId/FlopId offset, operand/Q/d/flop_domain refs remapped, B's PrimaryInput{port,width} nodes keep their port so A/B inputs unify for free because LeafEndpoint::PrimaryInput keys by (port,width) and interface match requires equal input PortIds). (2) Reuse the merge_bisimilar_flops refinement by factoring its "bucket -> refinable partition -> greatest-fixpoint refine -> rep_map" core into a non-mutating bisimulation_partition(m) -> rep_map helper; merge_bisimilar_flops keeps its collapse+finalize_flop_merge tail and stays byte-identical. (3) NO flop bijection required — coinduction proof: define R on (A-state,B-state) = "every union class holds equal values across both modules"; reset (within-bucket equal reset_val) gives R at t=0; stable quotient D-cones give R(t)=>R(t+1); per-output-port drive cones equal under the final rep_map give equal outputs under R for all time => observably equivalent. So the verdict = interfaces match by (PortId,width) AND every output port p: cone_proof(combined, driveA(p), rep_map) == cone_proof(combined, driveB(p), rep_map). (4) Grouping: pre-filter stateful flops-only leaf modules by (sorted input (PortId,width), sorted output (PortId,width), flop multiset {(width,reset_kind,reset_val,domain)}, output count); within a bucket, pairwise modules_bisimilar checks + union-find -> equivalence classes; reuse dedup_semantic_modules_once's lex-smallest-survivor + rewrite_instance_module_names + prune_modules_made_unreachable tail. (5) Names finalized: Config/Module/Design knob hierarchy_sequential_module_dedup (default false); DesignMetrics.sequential_module_proof_signatures + num_sequentially_duplicate_module_pairs. (6) Gate: rules-first two permuted/cross-wired equal-reset stateful leaf modules that both dedup_modules + dedup_semantic_modules leave as 2, collapse to 1 with the knob on, downstream-clean Verilator + both Yosys; knob-off snapshots 6/6 byte-identical. Resetless / memory / FSM / instance / param / aggregate modules excluded (first cut). No source change.`
  Verification: `done`
  Commit: `done`

- ID: `IDENTITY-DEEPENING.3b.2`
  Status: `active`
  Goal: `Implement the .3b.1 design in code.`
  Children: `IDENTITY-DEEPENING.3b.2a`, `IDENTITY-DEEPENING.3b.2b`

- ID: `IDENTITY-DEEPENING.3b.2a`
  Status: `done`
  Goal: `Factor the merge_bisimilar_flops refinement core into a non-mutating bisimulation_partition(&Module) -> Option<Vec<Vec<FlopId>>> helper that the cross-module proof reuses, keeping merge_bisimilar_flops byte-identical.`
  Acceptance: `cargo build/fmt/clippy clean; snapshots 6/6 byte-identical; the 6 bisim lib tests still pass; no behaviour change (pure refactor); committed through COMMIT.md with the leaf id.`
  Result: `Extracted the "bucket -> refinable partition -> greatest-fixpoint refine" core of merge_bisimilar_flops (src/ir/compact.rs) into a new non-mutating bisimulation_partition(&Module) -> Option<Vec<Vec<FlopId>>> (None preserves the original !has_refinable early-return; merge_bisimilar_flops keeps its guards + collapse + finalize_flop_merge tail). Pure refactor: cargo build clean; cargo fmt --check + clippy -D warnings clean; tests/snapshots 6/6 byte-identical; the 6 merge_bisimilar_flops_* lib gate tests still pass (default-off, mutual-swap merge, exact-cannot, resetless-excluded, relaxed off, e-graph gate, non-bisimilar split). CODEBASE_ANALYSIS updated. The helper is the byte-identical foundation the .3b.2b combined-module check calls.`
  Verification: `done`
  Commit: `done`

- ID: `IDENTITY-DEEPENING.3b.2b`
  Status: `done`
  Goal: `The cross-module feature on top of .3b.2a, split into the merge mechanism + proof (.3b.2b.1) and the observability/evidence/docs closeout (.3b.2b.2).`
  Children: `IDENTITY-DEEPENING.3b.2b.1`, `IDENTITY-DEEPENING.3b.2b.2`

- ID: `IDENTITY-DEEPENING.3b.2b.1`
  Status: `done`
  Goal: `The cross-module sequential-equivalence merge MECHANISM + proof, fully wired (no dead code) and gated default-off: modules_sequentially_equivalent(A,B) in src/ir/compact.rs (combined-module materialization reusing bisimulation_partition + per-output-cone equality under the final quotient); the dedup_sequential_modules pass beside dedup_semantic_modules in src/ir/dedup.rs (pre-filter bucket + greedy-by-representative grouping reusing the lex-survivor/rewrite/prune tail); the default-off Config::hierarchy_sequential_module_dedup knob; the gated wire-in at the gen/mod.rs finalization site; and the rules-first cross-module merge gate (compact + dedup lib tests).`
  Acceptance: `cargo fmt/check/clippy clean; rules-first gate (two sequentially-equivalent stateful leaf modules collapse to 1 with the knob on, both structural dedup_modules + combinational dedup_semantic_modules leave 2); modules_sequentially_equivalent proven on a hand fixture (equivalent->true; non-equivalent->false; resetless->false; interface mismatch->false); knob-off byte-identical (snapshots 6/6); committed through COMMIT.md with the leaf id. Metric pair + downstream-clean bank + book/USER_GUIDE/ROADMAP/KM closeout deferred to .3b.2b.2.`
  Result: `Landed modules_sequentially_equivalent + build_combined_module + sequential_leaf_eligible + N_BISIM_MODULE_FLOPS cap (src/ir/compact.rs); the cross-module proof reuses bisimulation_partition on a combined module (A/B inputs unified by (PortId,width) for free) + per-output-cone equality under the final quotient with one shared structural interner. Landed dedup_sequential_modules + SequentialPrefilterKey + greedy-by-rep grouping reusing the survivor/rewrite/prune tail (src/ir/dedup.rs). Default-off Config::hierarchy_sequential_module_dedup knob + gated generate_design wire-in. 6 rules-first gate tests (4 proof + 2 dedup); cargo --lib 433 pass; snapshots 6/6 byte-identical; clippy/fmt clean; bisim regression intact. No new proof engine; merge_bisimilar_flops byte-identical.`
  Verification: `done`
  Commit: `done`

- ID: `IDENTITY-DEEPENING.3b.2b.2`
  Status: `done`
  Goal: `The closeout on top of .3b.2b.1, split into the metric + schema + downstream bank (.3b.2b.2a) and the book/USER_GUIDE/ROADMAP/KM docs (.3b.2b.2b).`
  Children: `IDENTITY-DEEPENING.3b.2b.2a`, `IDENTITY-DEEPENING.3b.2b.2b`

- ID: `IDENTITY-DEEPENING.3b.2b.2a`
  Status: `done`
  Goal: `The metric + schema + evidence: a non-mutating DRY group_sequentially_equivalent_modules helper (shared by the dedup pass and the metric); the DesignMetrics sequential_module_proof_signatures + num_sequentially_duplicate_module_pairs pair (design-level pairwise grouping, RTL-invisible) wired into compute_design; the additive introspection schema MINOR bump 1.3 -> 1.4 (SCHEMA-DERIVED: DesignMetrics is in the --introspect payload) + docs/AGENT_INTROSPECTION_SCHEMA.md changelog + schema_version test updates; and a downstream-clean bank of a merged multi-module stateful design (Verilator + both Yosys).`
  Acceptance: `cargo fmt/check/clippy clean; the metric pair detects the equivalent pair (num_sequentially_duplicate_module_pairs = 1 pre-dedup, 0 post-dedup) by a focused test; schema bump 1.3 -> 1.4 with all schema_version assertions + schema doc updated; merged multi-module design banked downstream-clean (Verilator + both Yosys); knob-off byte-identical (snapshots 6/6); committed through COMMIT.md with the leaf id.`
  Result: `Factored group_sequentially_equivalent_modules(&Design) (non-mutating, src/ir/dedup.rs) shared by the dedup pass + the metric (counted pairs == what the pass collapses). New DesignMetrics::sequential_module_proof_signatures (Vec<Option<u64>>, class-id = FNV of class lex-min name) + num_sequentially_duplicate_module_pairs in compute_design, pre-filtered (zero proof work on default designs). Schema MINOR bump 1.3->1.4 (SCHEMA_VERSION + schema doc §4/§7+checklist + introspect/mcp schema_version assertions + README/USER_GUIDE/book/agent-mcp.md example numbers); both fields #[serde(default)]. Downstream bank /tmp/anvil-seq-bank/ (merged 2-module delay-line design, one .sv/module): Verilator -Wall clean, Yosys both modes (non-empty $_DFF_ netlist), Icarus -g2012 clean. cargo --lib 435/0/2 incl. the metric counts-then-collapses test + bank validate test; snapshots 6/6 byte-identical; clippy/fmt clean; mdbook clean.`
  Verification: `done`
  Commit: `done`

- ID: `IDENTITY-DEEPENING.3b.2b.2b`
  Status: `done`
  Goal: `(Future) The user-facing docs closeout: book/src/factorization.md + hierarchy.md (the sequential whole-module equivalence narrative), USER_GUIDE/knobs (the hierarchy_sequential_module_dedup config knob), ROADMAP gap 2 (record the cross-module sequential-equivalence merge as delivered), and a Knowledge Map card for the new durable identity fact.`
  Acceptance: `mdbook build clean; book_examples gate still green; KM + memory-arch self-checks clean; ROADMAP gap 2 accurate (no premature done claims); committed through COMMIT.md with the leaf id.`
  Result: `book/src/factorization.md gains §9b (whole-module sequential equivalence) + updated "full factorization still means" list + empirical-counters metric pair; book/src/hierarchy.md gains the third (sequential) module-identity layer pointing at §9b; USER_GUIDE gains the hierarchy_sequential_module_dedup config-knob bullet; ROADMAP gap 2 + the capability-lanes section record the merge as delivered (lane active, no current frontier, deeper boundaries named/not-started); new KM card docs/knowledge/sequential-module-dedup.md (11 answer keys + reverify), KNOWLEDGE_MAP.md regenerated (32 facts). Docs-only / DUT byte-identical. mdbook build clean; cargo test --test book_examples 3/3 (84.7s); KM + memory-arch self-checks clean.`
  Verification: `done`
  Commit: `done`

## Current Frontier

**No current frontier.** `.3`/`.3b`/`.3b.2`/`.3b.2b` are all `done` (the
cross-module whole-leaf-module sequential-equivalence sub-tree is closed). The
`IDENTITY-DEEPENING` lane stays `active` as an open-ended capability lane; the
deeper module-equivalence boundaries (memory / FSM / wrapper / retimed-state
whole-module equivalence) are named, **not-started** future leaves — each would
open as a new `.4`/`.3c`-style design leaf when prioritized. None retired.

Most-recently-completed leaves:

| Order | Leaf | Status | Why |
| --- | --- | --- | --- |
| — | `IDENTITY-DEEPENING.3b.2b.2b` | `done` | The user-facing narrative closeout: book `factorization.md` §9b + `hierarchy.md` third module-identity layer, USER_GUIDE `hierarchy_sequential_module_dedup` knob bullet, ROADMAP gap 2 + capability-lanes "delivered" status, and the KM card `sequential-module-dedup`. Docs-only / DUT byte-identical; mdbook + book_examples green. Closes `.3b.2b`. |
| — | `IDENTITY-DEEPENING.3b.2b.2a` | `done` | Landed the metric + schema + evidence: the DRY `group_sequentially_equivalent_modules` helper, the `DesignMetrics` `sequential_module_proof_signatures` + `num_sequentially_duplicate_module_pairs` pair, the additive schema MINOR bump `1.3 → 1.4`, and the downstream-clean merged-design bank `/tmp/anvil-seq-bank/` (Verilator + both Yosys + Icarus). |
| — | `IDENTITY-DEEPENING.3b.2b.1` | `done` | Landed the cross-module merge **mechanism + proof**, fully wired and default-off: `modules_sequentially_equivalent` (combined-module materialization reusing `bisimulation_partition` + per-output-cone equality under the final quotient) + the `dedup_sequential_modules` pass + the default-off `hierarchy_sequential_module_dedup` knob + the gated `gen/mod.rs` wire-in + 6 rules-first gate tests; snapshots 6/6 byte-identical. |
| — | `IDENTITY-DEEPENING.3b.2a` | `done` | Factored the `merge_bisimilar_flops` refinement core into a non-mutating `bisimulation_partition(&Module) -> Option<Vec<Vec<FlopId>>>` helper; byte-identical (snapshots 6/6 + 6 bisim tests); the foundation `.3b.2b.1` reuses on a combined module. |
| — | `IDENTITY-DEEPENING.3b.1` | `done` | Ground decision `0008` in the real code: combined-module materialization (inputs unify by `(PortId,width)` for free), reuse the `merge_bisimilar_flops` refinement via a factored `bisimulation_partition`, the no-bijection soundness condition (interfaces + stable union partition + per-output-cone equality under the quotient), pre-filter + union-find grouping; pinned knob/metric names + gate. No source change. |
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

- `.3b.1` resolved decision `0008`'s open question on the cross-module proof
  representation: **no new proof engine** — materialize a temporary combined
  `Module` so `(PortId, width)` endpoint identity unifies A's and B's primary
  inputs for free, reuse the `merge_bisimilar_flops` refinement via a factored
  `bisimulation_partition` helper, and require **no flop bijection** (interfaces
  match + a stable union partition + per-output-port cone equality under the
  quotient is sufficient and sound by coinduction). Knob/metric names pinned:
  `Config/Module/Design::hierarchy_sequential_module_dedup` (default `false`);
  `DesignMetrics::sequential_module_proof_signatures` +
  `num_sequentially_duplicate_module_pairs`.
- Remaining for `.3b.2`: the union flop cap `N_bisim_module_flops` calibration,
  the precise combined-module remap helper, and the downstream-clean gate shape
  (focused `cargo test` + smoke vs a dedicated `tool_matrix` scenario set), by
  whichever proves the cross-module stateful merge by construction at lowest
  cost (mirrors the `.2b` precedent — the random generator rarely emits a
  distinct-but-equivalent stateful module pair, so a rules-first hand fixture is
  likely the lowest-cost proof).

## Blockers

- None. (Sequenced after Lanes 2–3 by choice, not by dependency.)

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-06-15` | `IDENTITY-DEEPENING.1` | Design/decision leaf, no source change. `bash scripts/check_memory_architecture.sh` + `bash knowledge-map/scripts/check_knowledge_map.sh` clean; `KNOWLEDGE_MAP.md` regenerated to include decision `0007` answers. | `done` |
| `2026-06-15` | `IDENTITY-DEEPENING.2a` | Design-detail leaf, no source change (grounded in a close read of `src/ir/compact.rs` `merge_equivalent_flops`/`flop_d_signature`/`cone_proof`/`semantic_cone_proof`, `src/config.rs` knob pattern, `src/metrics.rs` merge-count pattern). Self-checks clean. | `done` |
| `2026-06-15` | `IDENTITY-DEEPENING.2b` | `cargo build` + `cargo clippy --all-targets -- -D warnings` clean; `cargo fmt --all --check` clean; `cargo test --lib` 403 pass incl. 6 new bisim gate tests + the metrics-plumbing test + introspect/MCP schema_version tests; `cargo test --test snapshots` 6/6 byte-identical (knob default-off); representative `cargo test --test pipeline` reproducibility test green (full pipeline suite is heavy/slow and exercises only the byte-identical default path). Downstream-clean bank on the merged mutual-swap self-hold output: Verilator `--lint-only -Wall` 0 warnings, Yosys without-abc + with-abc, Icarus `iverilog -g2012` (re-bank: `ANVIL_DUMP_BISIM_SV=1 cargo test --lib merge_bisimilar_flops_merges_mutual_swap_registers`). KM + mem-arch self-checks clean. | `done` |
| `2026-06-15` | `IDENTITY-DEEPENING.3a` | Design/decision leaf, **no source change** (grounded in a close read of `src/ir/dedup.rs` `dedup_semantic_modules` + `prune_modules_made_unreachable`, `src/metrics.rs` `semantic_module_proof_inner` + the module-proof budget constants, and `src/ir/compact.rs` `merge_bisimilar_flops` / `canonical_flop_endpoint` / `cone_proof` / `MERGE_SEMANTIC_LIMITS`). Decision `0008` written with KM `answers:` front-matter; `KNOWLEDGE_MAP.md` regenerated; `bash scripts/check_memory_architecture.sh` + `bash knowledge-map/scripts/check_knowledge_map.sh` clean. | `done` |
| `2026-06-15` | `IDENTITY-DEEPENING.3b.1` | Design-detail leaf, **no source change** (grounded in the full source of `src/ir/dedup.rs` `dedup_semantic_modules_once` + survivor/rewrite/prune tail, `src/metrics.rs` `SemanticModuleProof` / `semantic_module_proof_inner` / `semantic_module_proof_body` / `build_instance_semantic_views`, and `src/ir/compact.rs` `merge_bisimilar_flops` refinement loop + `cone_proof` quotient threading + `LeafEndpoint::PrimaryInput{port,width}`). Pinned the combined-module materialization, the factored `bisimulation_partition` reuse, the no-bijection coinduction soundness condition, the pre-filter + union-find grouping, and the finalized knob/metric names + gate in `DEVELOPMENT_NOTES.md`. `bash scripts/check_memory_architecture.sh` + `bash knowledge-map/scripts/check_knowledge_map.sh` clean (no fact-file change ⇒ KM already in sync). | `done` |
| `2026-06-15` | `IDENTITY-DEEPENING.3b.2a` | Pure refactor (`src/ir/compact.rs`): `cargo build` clean; `cargo fmt --all --check` clean; `cargo clippy --all-targets -- -D warnings` clean; `cargo test --test snapshots` 6/6 byte-identical; `cargo test --lib bisim` 6/6 pass (default-off, mutual-swap merge, exact-cannot, resetless-excluded, relaxed off, e-graph gate, non-bisimilar split). No behaviour change ⇒ DUT byte-identical. CODEBASE_ANALYSIS updated. | `done` |
| `2026-06-16` | `IDENTITY-DEEPENING.3b.2b.1` | Code leaf (`src/ir/compact.rs`, `src/ir/dedup.rs`, `src/config.rs`, `src/gen/mod.rs`): `cargo check --all-targets` clean; `cargo clippy --all-targets -- -D warnings` clean; `cargo fmt --all --check` clean; `cargo test --lib` 433 pass incl. 6 new gate tests (4 `modules_sequentially_*` proof + 2 `sequential_dedup_*`) and the 6 `merge_bisimilar_flops_*` regressions; `cargo test --test snapshots` 6/6 byte-identical (knob default-off). DUT byte-identical (default-off). Live docs (CHANGES, DEVELOPMENT_NOTES, CODEBASE_ANALYSIS, MEMORY) updated. Landed `314664c`. | `done` |
| `2026-06-16` | `IDENTITY-DEEPENING.3b.2b.2b` | Docs-only leaf (`book/src/factorization.md`, `book/src/hierarchy.md`, `USER_GUIDE.md`, `ROADMAP.md`, `docs/knowledge/sequential-module-dedup.md`, `KNOWLEDGE_MAP.md`, `docs/TASK_TREE.md`): `mdbook build book` clean; `cargo test --test book_examples` 3/3 (84.7s); `bash knowledge-map/scripts/check_knowledge_map.sh` OK (32 facts, map in sync); `bash scripts/check_memory_architecture.sh` clean. No source change ⇒ DUT byte-identical. Closes `.3b.2b` / `.3b.2` / `.3b` / `.3`. | `done` |
| `2026-06-16` | `IDENTITY-DEEPENING.3b.2b.2a` | Code+schema leaf (`src/ir/dedup.rs`, `src/metrics.rs`, `src/introspect/mod.rs`, `src/mcp/mod.rs`, `docs/AGENT_INTROSPECTION_SCHEMA.md`, README/USER_GUIDE/`book/agent-mcp.md`): `cargo check --all-targets` + `cargo clippy --all-targets -- -D warnings` + `cargo fmt --all --check` clean; `cargo test --lib` 435 pass incl. `sequential_proof_metric_counts_then_collapses_pair` (1 pre-dedup, 0 post) + the bank validate test + the `introspect`/`mcp` schema_version `1.4` assertions; `cargo test --test snapshots` 6/6 byte-identical (RTL-invisible); `mdbook build book` clean. Downstream bank `/tmp/anvil-seq-bank/`: Verilator `--lint-only -Wall` clean (2 modules), Yosys without-abc + with-abc (non-empty `$_DFF_` netlist), Icarus `iverilog -g2012` clean (re-bank: `ANVIL_DUMP_SEQ_MODULE_SV=1 cargo test --lib sequential_dedup_merged_design_is_downstream_clean`). KM + memory-arch self-checks clean. | `done` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `IDENTITY-DEEPENING.1` | `IDENTITY-DEEPENING.1 — promote Lane 1 + decision 0007` | Landed `43e2a2d`. Decision record `0007`; tree split into `.2`/`.3`. |
| `IDENTITY-DEEPENING.2a` | `IDENTITY-DEEPENING.2a — bisimulation flop merge design detail` | Grounded `.2b` algorithm/API-reuse/names/budget/gate; `.2` split into `.2a`/`.2b`. |
| `IDENTITY-DEEPENING.2b` | `IDENTITY-DEEPENING.2b — implement bounded bisimulation flop merge` | Landed `merge_bisimilar_flops` + `finalize_flop_merge` + quotient proof threading + default-off knob + metric + 6 gate tests + schema 1.0→1.1; downstream-clean bank. Closes `.2`. |
| `IDENTITY-DEEPENING.3a` | `IDENTITY-DEEPENING.3a — whole-module sequential equivalence design` | Decision record `0008`; split `.3` into `.3a` (design, done) + `.3b` (impl, future). No source change. |
| `IDENTITY-DEEPENING.3b.1` | `IDENTITY-DEEPENING.3b.1 — cross-module sequential merge design detail` | Grounded `.3b.2` algorithm/API-reuse/names/soundness/gate in the real code; split `.3b` into `.3b.1` (design-detail, done) + `.3b.2` (impl). No source change. |
| `IDENTITY-DEEPENING.3b.2a` | `IDENTITY-DEEPENING.3b.2a — factor bisimulation_partition helper` | Factored the `merge_bisimilar_flops` refinement core into a non-mutating `bisimulation_partition` helper; byte-identical (snapshots 6/6 + 6 bisim tests); split `.3b.2` into `.3b.2a` (refactor, done) + `.3b.2b` (cross-module feature). |
| `IDENTITY-DEEPENING.3b.2b.1` | `IDENTITY-DEEPENING.3b.2b.1 — cross-module sequential-equivalence merge mechanism` | Landed `modules_sequentially_equivalent` (combined-module materialization reusing `bisimulation_partition` + per-output-cone equality under the final quotient) + `dedup_sequential_modules` (pre-filter + greedy-by-rep grouping + shared survivor/rewrite/prune tail) + default-off `hierarchy_sequential_module_dedup` knob + gated `generate_design` wire-in + 6 rules-first gate tests. Default-off / byte-identical (snapshots 6/6). Split `.3b.2b` into `.3b.2b.1` (mechanism, done) + `.3b.2b.2` (closeout). Landed `314664c`. |
| `IDENTITY-DEEPENING.3b.2b.2a` | `IDENTITY-DEEPENING.3b.2b.2a — sequential proof metric + schema 1.4 + downstream bank` | Factored the shared `group_sequentially_equivalent_modules` helper; added `DesignMetrics::sequential_module_proof_signatures` + `num_sequentially_duplicate_module_pairs`; bumped introspection schema `1.3 → 1.4` (DesignMetrics is in the `--introspect` payload); banked the merged 2-module stateful design downstream-clean (Verilator + both Yosys + Icarus). RTL-invisible / snapshots 6/6. Split `.3b.2b.2` into `.3b.2b.2a` (metric/schema/bank, done) + `.3b.2b.2b` (book/docs closeout). |
| `IDENTITY-DEEPENING.3b.2b.2b` | `IDENTITY-DEEPENING.3b.2b.2b — whole-module sequential equivalence docs closeout` | book `factorization.md` §9b + `hierarchy.md` third module-identity layer + USER_GUIDE knob bullet + ROADMAP gap 2 / capability-lanes "delivered" + KM card `sequential-module-dedup` (KNOWLEDGE_MAP regenerated, 32 facts) + `docs/TASK_TREE.md` index row. Docs-only / DUT byte-identical; mdbook + book_examples 3/3 green. Closes `.3b.2b` / `.3b.2` / `.3b` / `.3`. |

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
- `2026-06-15`: `.3b.2a` done — first code slice of `.3b.2`: factored the
  `merge_bisimilar_flops` refinement core (`src/ir/compact.rs`) into a
  non-mutating `bisimulation_partition(&Module) -> Option<Vec<Vec<FlopId>>>`
  helper that `.3b.2b` reuses on a combined module; `merge_bisimilar_flops`
  keeps its guards + collapse + `finalize_flop_merge` tail and stays
  byte-identical (snapshots 6/6, 6 bisim tests, clippy/fmt clean). Split
  `.3b.2` into `.3b.2a` (refactor, done) + `.3b.2b` (cross-module feature).
  Frontier advances to `.3b.2b`.
- `2026-06-16`: `.3b.2b` split into `.3b.2b.1` (cross-module merge mechanism +
  proof, `in_progress`) + `.3b.2b.2` (metric + downstream bank + book/docs
  closeout, future). Rationale: the full `.3b.2b` scope (cross-module proof core
  + dedup pass + knob + wire-in + DesignMetrics pair + downstream bank + 7 docs)
  is too broad for one signoff-quality slice; the merge mechanism and the
  observability/closeout are independently reviewable, and splitting keeps the
  mechanism leaf free of dead code (everything in `.3b.2b.1` is wired and used).
  Frontier advances to `.3b.2b.1`.
- `2026-06-16`: `.3b.2b.1` done — landed the cross-module sequential-equivalence
  merge mechanism + proof: `modules_sequentially_equivalent` reuses
  `bisimulation_partition` on a combined module (A/B inputs unified by
  `(PortId,width)` for free) then proves per-output-cone equality under the final
  quotient (`src/ir/compact.rs`); `dedup_sequential_modules` groups eligible
  stateful flops-only leaves by a structural pre-filter + greedy-by-representative
  proof, reusing the shared survivor/rewrite/prune tail (`src/ir/dedup.rs`);
  default-off `Config::hierarchy_sequential_module_dedup` knob + gated
  `generate_design` wire-in. 6 rules-first gate tests; `cargo --lib` 433 pass;
  snapshots 6/6 byte-identical; clippy/fmt clean; bisim regression intact; no new
  proof engine (`merge_bisimilar_flops` byte-identical). Frontier advances to
  `.3b.2b.2` (metric + downstream bank + book/docs closeout).
- `2026-06-16`: split `.3b.2b.2` into `.3b.2b.2a` (metric + schema bump +
  downstream bank, `in_progress`) + `.3b.2b.2b` (book/USER_GUIDE/ROADMAP/KM
  narrative, future); rationale: the metric requires an introspection schema MINOR
  bump (`DesignMetrics` is in the `--introspect` payload), making the
  code+schema+evidence half a distinct, independently-reviewable concern from the
  user-facing narrative docs.
- `2026-06-16`: `.3b.2b.2a` done — factored the non-mutating
  `group_sequentially_equivalent_modules(&Design)` helper shared by the dedup pass
  and the metric (counted pairs == what the pass collapses); added
  `DesignMetrics::sequential_module_proof_signatures` +
  `num_sequentially_duplicate_module_pairs` to `compute_design` (pre-filtered, zero
  proof work on default designs); bumped introspection schema `1.3 → 1.4`
  (additive MINOR, both fields `#[serde(default)]`) with the schema doc + all
  `schema_version` assertions + README/USER_GUIDE/book example numbers synced; and
  banked the merged 2-module stateful design downstream-clean
  (`/tmp/anvil-seq-bank/`: Verilator `-Wall` + Yosys both modes + Icarus). `cargo
  --lib` 435 pass; snapshots 6/6 byte-identical; mdbook clean. Frontier advances to
  `.3b.2b.2b` (book/USER_GUIDE/ROADMAP/KM narrative closeout).
- `2026-06-16`: `.3b.2b.2b` done — the user-facing narrative closeout: book
  `factorization.md` §9b (whole-module sequential equivalence) + the
  "full factorization still means" list + empirical-counters metric pair;
  `hierarchy.md` third (sequential) module-identity layer; USER_GUIDE
  `hierarchy_sequential_module_dedup` knob bullet; ROADMAP gap 2 + the
  capability-lanes section marked delivered; new KM card
  `sequential-module-dedup` (KNOWLEDGE_MAP regenerated, 32 facts); `docs/TASK_TREE.md`
  index row updated. Docs-only / DUT byte-identical; mdbook + `book_examples` 3/3
  green. **This closes `.3b.2b`, `.3b.2`, `.3b`, and `.3`** — the cross-module
  whole-leaf-module sequential-equivalence sub-tree is complete. The
  `IDENTITY-DEEPENING` lane stays `active` with no current frontier; the deeper
  module-equivalence boundaries (memory / FSM / wrapper / retimed-state) are named,
  not-started, open-ended future leaves (none retired).
- `2026-06-15`: `.3b.1` done — split `.3b` into `.3b.1` (design-detail, done) +
  `.3b.2` (impl, future); `.3b` is now an `active` container. Resolved decision
  `0008`'s central impl challenge by a full read of the real code: **no new
  cross-module proof engine** — materialize a combined `Module` so A/B primary
  inputs unify by `(PortId, width)` for free (`LeafEndpoint::PrimaryInput`
  already keys by `(port, width)`); reuse the `merge_bisimilar_flops` refinement
  via a factored non-mutating `bisimulation_partition` helper (collapse pass
  stays byte-identical); **no flop bijection needed** (interfaces match + stable
  union partition + per-output-port cone equality under the quotient is sound by
  coinduction); pre-filter (interface + flop-multiset + output-count) +
  pairwise + union-find grouping reusing `dedup_semantic_modules_once`'s
  survivor/rewrite/prune tail. Pinned knob `hierarchy_sequential_module_dedup`
  and metrics `sequential_module_proof_signatures` /
  `num_sequentially_duplicate_module_pairs`. No source change. Frontier advances
  to `.3b.2`.
