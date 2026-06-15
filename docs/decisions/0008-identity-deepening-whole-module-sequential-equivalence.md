---
id: identity-deepening-whole-module-sequential-equivalence
title: The second IDENTITY-DEEPENING extension is bounded whole-leaf-module sequential equivalence via cross-module bisimulation (default-off, beside dedup_semantic_modules)
answers:
  - "can ANVIL merge whole stateful modules by sequential equivalence"
  - "does ANVIL merge sequentially-equivalent modules"
  - "what is the second IDENTITY-DEEPENING extension"
  - "how would ANVIL prove two stateful modules equivalent"
  - "what is whole-module sequential equivalence in ANVIL"
  - "how is whole-module sequential equivalence proven sound"
  - "why not use reachable-product-state exploration for module equivalence"
  - "does sequential module dedup merge modules with memories or FSMs"
  - "what is the cross-module bisimulation state correspondence"
  - "does whole-module sequential equivalence retire the combinational module dedup"
  - "is the sequential module dedup default-off"
  - "what budget bounds whole-module sequential equivalence"
date: 2026-06-15
status: current
tags: [identity, sequential, factorization, bisimulation, coinduction, module-dedup, hierarchy]
evidence: docs/decisions/0008-identity-deepening-whole-module-sequential-equivalence.md; docs/tasks/IDENTITY-DEEPENING.md; src/ir/dedup.rs; src/ir/compact.rs; src/metrics.rs; book/src/factorization.md; ROADMAP.md
---

# 0008 - IDENTITY-DEEPENING second extension: bounded whole-leaf-module sequential equivalence via cross-module bisimulation

- Date: 2026-06-15
- Status: accepted
- Tags: identity, sequential, factorization, bisimulation, coinduction, module-dedup, hierarchy

## Context

`IDENTITY-DEEPENING` (Lane 1 of the three owner-directed post-phase capability
lanes) advances the `NodeId`-as-identity / full-factorization north star
(`ROADMAP.md` steering gap 2, `feedback_full_factorization`) into the territory
left explicitly bounded by the closed identity trees. Its `.1` leaf picked the
**first** sound extension (bounded bisimulation *flop* merge, decision
[`0007`](0007-identity-deepening-first-extension.md), delivered as `.2`). `.1`
also named — and decision `0007` explicitly *deferred* as the lead rejected
alternative — the next, larger step:

> **Whole stateful-leaf-module bounded sequential equivalence via
> reachable-product-state exploration.** Sound but a larger jump; it naturally
> *builds on* the flop-level bisimulation primitive plus a state-correspondence
> search. **Kept as a named future leaf**, not retired.

This leaf (`IDENTITY-DEEPENING.3a`) is the design/decision leaf for that step:
it must fix the proof discipline + budget + downstream gate **before any merge
code lands** (`.3b`). The doctrinal bar is strong and unchanged: two structures
share one identity **only** when ANVIL can *prove* they implement the same
functionality with respect to the same canonical leaf endpoints — never
syntactic resemblance, never an unsound or unbounded merge
(`feedback_rules_first_generation`, `feedback_never_retire_strategies`).

### Current module-identity proof surface (inventory, current code)

- **Structural module dedup** (`src/ir/dedup.rs::dedup_modules`): groups module
  definitions by `canonical_module_signature` (`src/metrics.rs`, an FNV-1a hash
  over port/node/flop shape that deliberately omits child `Instance.module` /
  `Instance.name`), rewrites instances to the canonical survivor, and prunes
  modules made unreachable (`hierarchy-dedup-prune`). Structural only — it does
  **not** prove semantic equivalence (`hierarchy-identity-boundary`).
- **Bounded combinational module dedup** (`src/ir/dedup.rs::dedup_semantic_modules`
  → `src/metrics.rs::semantic_module_proof_inner`): a default-off
  (`hierarchy_semantic_module_dedup`, node-id / e-graph) pass that proves
  **bounded whole-module truth-table equivalence** for *pure-combinational*
  modules and bounded pure-combinational wrappers
  (`bounded-semantic-module-identity`). It enumerates every input assignment
  (`evaluate_semantic_module_node`) and compares the full output truth tables,
  keyed by `(PortId, width)` interface. Budget:
  `MAX_SEMANTIC_MODULE_SUPPORT_BITS = 12`, `MAX_SEMANTIC_MODULE_NODES = 128`,
  `MAX_SEMANTIC_MODULE_INSTANCES = 8`, output width `<= 128`,
  `MAX_SEMANTIC_MODULE_WORK_UNITS = 131072`. It **skips** any module where
  `has_local_flops() || has_local_memories() || has_local_fsms() ||
  param_env.is_some() || aggregate_layout.is_some()`. So **stateful modules are
  the explicit skipped boundary.**
- **Flop-level identity** (`src/ir/compact.rs`): exact reset-defined self-hold /
  same-endpoint D-cone merge (`merge_equivalent_flops`,
  `reset-defined-self-hold-flop-identity`), the opt-in bounded *bisimulation*
  flop merge (`merge_bisimilar_flops`, `0007`/`.2b`,
  `bisimulation-flop-merge`), and deterministic FSM-block merge
  (`merge_equivalent_fsms`, `fsm-identity-merge`). Memories stay
  state-by-instance (`memory-identity-boundary`).

### The gap chosen

The flop-level bisimulation primitive (`.2`) proves *individual flops within one
module* sequentially equivalent up to a state correspondence. The combinational
module dedup (`dedup_semantic_modules`) proves *whole modules* equivalent but
only when they are stateless. The open frontier exactly between them is the
named-future step: **prove two whole stateful leaf modules observationally
(sequentially) equivalent** — lifting the stateful-module skip in
`dedup_semantic_modules` soundly, by reusing the `.2` bisimulation idea across
*two* modules' state instead of within one.

## Decision

**The second `IDENTITY-DEEPENING` extension is a bounded whole-leaf-module
sequential-equivalence merge: a default-off, opt-in pass — added *beside*
`dedup_semantic_modules`, not a modification of it — that proves two stateful
leaf modules observationally equivalent via a *cross-module* bisimulation (the
`.2` greatest-fixpoint partition refinement lifted to the disjoint union of the
two modules' flops, with primary inputs unified by `(PortId, width)` interface)
plus bounded output-cone equality under the resulting quotient, reusing the same
12-bit / 128-node / 131072-work combinational proof budget.** When two stateful
leaf modules are proven equivalent, instances of one are rewritten to the other
exactly as the combinational pass already does, and unreachable definitions are
pruned. It strictly generalizes the pure-combinational `dedup_semantic_modules`
(the zero-flop special case) and the flop-level classes — and retires nothing.

### Scope of the first cut (`.3b` impl)

Eligible: **stateful leaf modules whose only state is local flops.** Excluded
(each an existing, separately-recorded boundary, named future work, none
retired):

- `has_local_memories()` — array contents are not reset-defined ⇒ no bisimulation
  base case (`memory-identity-boundary`).
- `has_local_fsms()` — FSM state is reset-defined and table-described; whole-module
  equivalence *including* FSM blocks is a larger correspondence problem than the
  first cut (intra-module duplicate FSMs already merge via
  `merge_equivalent_fsms`).
- `!instances.is_empty()` — composing child-instance state into the product is the
  sequential analogue of the bounded *wrapper* case; the combinational wrapper
  proof (`<= 8` instances) stays as-is, but the sequential wrapper case is
  deferred.
- `param_env.is_some()` / `aggregate_layout.is_some()` — existing
  `dedup_semantic_modules` skips (parameterized / aggregate-projected).

### Proof discipline (soundness argument)

Treat each module as a Moore/Mealy machine: its state is its flop vector over
`(PrimaryInputs ∪ FlopQs)`, its next-state function is the per-flop D-cone, its
output function is the per-output-port drive-cone. Given two candidate modules
`M_A`, `M_B`:

1. **Interface base case.** Require identical input-port and output-port sets
   keyed by `(PortId, width)` (the same interface match `dedup_semantic_modules`
   already enforces; port IDs are load-bearing because instance rewrites preserve
   parent-side port-id bindings). Unify each module's `PrimaryInput{port, width}`
   endpoints across the two modules by `(PortId, width)`. Mismatched interface ⇒
   not equivalent (skip).
2. **State base case.** Form the disjoint union `M_A.flops ⊎ M_B.flops`. Bucket
   by `(width, reset_kind, reset_val, flop_domain)` — the same key
   `merge_bisimilar_flops` uses. **Resetless flops are excluded** (and a module
   containing any resetless flop is conservatively skipped): with no reset there
   is no provable equal initial state, so a cross-module state correspondence has
   no base case (this carries the `0007`/`.2b` soundness fix forward — the
   resetless-self-hold boundary is preserved, not eroded).
3. **Greatest-fixpoint refinement (step).** Run the `.2` partition refinement on
   the union. Two flops (from either module) stay in one class iff their D-cones —
   with **every `FlopQ` endpoint rewritten to its current class representative**
   (the quotient) and **every `PrimaryInput` endpoint unified by `(PortId,
   width)`** — are proven combinationally equal by the existing bounded
   endpoint-preserving proof (`cone_proof`, 12-bit / 128-node / 131072-work). The
   cross-module endpoint unification is precisely what lets a single class hold
   flops from *both* modules. Refine until no class splits
   (Kanellakis–Smolka / Hopcroft-style coarsest stable partition).
4. **Output equality (observation).** For each output port `p` (matched by
   `(PortId, width)`), prove `M_A`'s drive-cone for `p` equals `M_B`'s drive-cone
   for `p` under the *final* quotient, by the same bounded combinational proof
   over the unified endpoint set.
5. **Equivalence verdict.** `M_A ≡ M_B` iff steps 1–4 all hold: matching
   interfaces, a stable cross-module bisimulation on the union state, and equal
   output cones under the quotient.
6. **Soundness (coinduction).** By induction on cycle `t`: at `t = 0` reset makes
   the members of every class (from both modules) hold equal `Q` (base case, by
   bucketing on `reset_val`); if at cycle `t` every class's members hold equal
   `Q`, then for any two flops in a class their quotient-substituted D-cones
   evaluate equally (step 3 fixpoint), so they hold equal `Q` at `t + 1`; and
   equal state ⇒ equal outputs by step 4. Hence for **every** input sequence
   `M_A` and `M_B` emit identical output sequences — observationally equivalent —
   so merging the two *definitions* (rewriting instances of one to the other) is
   sound for all time.
7. **Generalization, not replacement.** A pure-combinational module has zero
   flops: step 2's union is empty, step 3 is trivial, and the verdict reduces to
   "every output cone is equal over the input endpoints" — the same thing
   `dedup_semantic_modules` proves today. So this is a strict superset of the
   combinational case. Because the first cut is a **separate default-off pass**
   that runs only on flop-bearing modules, the existing combinational pass and
   its truth-table verdict are left **byte-identical** (no unification risk).
8. **Correctly excluded.** Retimed / latency-shifted whole-module state
   (observable cycle offset ⇒ not bisimilar), reset-value / domain / width
   mismatches (base case fails), memory/FSM/instance/parameterized/aggregate
   modules (scope), and any cone exceeding the proof budget → the candidate
   conservatively **fails to merge** (the offending pair splits to singleton
   classes / the module pair is rejected). Never a guess.

### Budget

- **Per cone equivalence check (D-cones in step 3, output cones in step 4):**
  reuse the existing bounded combinational proof and its budget verbatim — width
  `<= 128` bits, canonical endpoint support `<= 12` bits, `<= 128` unique cone
  nodes, `assignment_count * cone_node_count <= 131072`. A check that cannot be
  discharged within budget yields "not proven" ⇒ the candidate pair fails (never
  a guess).
- **Cross-module refinement cost:** bounded by `O(k² · iterations)` with
  `iterations <= k`, where `k` = total flops in the union of the two modules.
  Module pairs whose union flop count exceeds a calibration cap (working name
  `N_bisim_module_flops`, mirroring `N_bisim_flops = 64`) are skipped, so the
  pass cannot blow up on pathological modules. The concrete cap is set
  empirically at `.3b`.
- **Candidate-pair cost:** restrict candidate pairs the same way the
  combinational pass does — bucket flop-bearing leaf modules by a cheap pre-filter
  (matching `(PortId, width)` interface + matching flop multiset key
  `{(width, reset_kind, reset_val, domain)}` + matching output count) before any
  cone proof, so the `O(modules²)` comparison only runs on plausibly-equivalent
  groups.

### Control surface (default-off / byte-identical)

- Gate the merges behind a new default-off `Config` knob (working name
  `hierarchy_sequential_module_dedup`, finalized at `.3b`), parallel to the
  existing `hierarchy_module_dedup` (structural) and
  `hierarchy_semantic_module_dedup` (combinational) knobs. Additionally require
  `identity_mode = node-id` and effective `factorization_level = e-graph` (the
  rung that already gates semantic gate merge, `merge_bisimilar_flops`, and
  `dedup_semantic_modules`). When the knob is off (default), emitted RTL is
  **byte-identical** — `tests/snapshots.rs` is untouched — and
  `--identity-mode relaxed` stays the real off-switch.

### Downstream gate

- A design-level metric pair, RTL-invisible, parallel to the combinational
  `DesignMetrics.semantic_module_signatures` /
  `num_semantically_duplicate_module_pairs`: a `sequential_module_proof`
  signature for each eligible stateful leaf module and a
  `num_sequentially_duplicate_module_pairs` counter (working names; finalized at
  `.3b`), reducible to zero by the new pass on a supported design.
- A focused, rules-first scenario/test that constructs a design with two
  **stateful leaf modules that are sequentially equivalent up to a non-identity
  state correspondence** (e.g. two small flop networks whose registers are
  permuted / mutually cross-wired, same reset) yet structurally distinct enough
  that both `dedup_modules` (structural signatures differ) and the current
  `dedup_semantic_modules` (skips stateful) leave them as **two** modules. With
  the knob on, assert the design collapses to **one** module definition and the
  merged multi-module design is **clean across Verilator + both Yosys modes**
  (a banked `tool_matrix`/smoke report per the signoff discipline). Choose the
  lowest-cost shape (focused `cargo test` + manual smoke vs a dedicated
  `tool_matrix` scenario set) by whichever proves the cross-module merge by
  construction at lowest cost, mirroring the `0007`/`.2b` precedent.
  Regression-protect: knob-off byte-identical; the existing combinational module
  merge, structural dedup, exact / bisimulation flop merge, and FSM merge all
  still behave as before.

## Decisive test applied

"A merge must be a *proof*, not a heuristic, and a day-one gate must be clean by
construction." A bisimulation from a reset base case across the *union* of two
machines' state is a textbook *sound* proof of sequential equivalence; the
partition-refinement bound and the reused combinational budget make it cheap and
terminating without any reachable-state enumeration; and a design with two
genuinely-distinct-but-equivalent stateful modules collapsing to one definition
is exactly the unusual-but-valid structure the north star wants downstream tools
to ingest.

## Rejected alternatives (as the approach)

- **Bounded reachable-product-state exploration / bounded model checking
  (k-step).** Proves agreement only up to depth `k`, not for all time —
  **unsound as a merge proof**. Decision `0007` rejected it at the flop level for
  the same reason; it is rejected here at the module level too. The
  bisimulation / partition-refinement approach is the sound, bounded one. (Note:
  `0007`'s rejected-alternative wording named "reachable-product-state
  exploration" as the *future leaf*; this decision refines that to the **sound
  bisimulation form** of whole-module equivalence — the reachable-product framing
  is the intuition, the partition refinement is the sound mechanization.)
- **Unifying the new proof into `dedup_semantic_modules`** (one engine for both
  combinational and sequential). Rejected for the first cut: it risks changing the
  existing combinational verdict / byte-identical behavior and mixes two proof
  engines (whole-module input truth-table enumeration vs per-cone bounded proof).
  Added **beside** it instead — the `0007`/`.2b` precedent of placing
  `merge_bisimilar_flops` next to `merge_equivalent_flops`. Unification stays a
  possible later cleanup, not a first step.
- **Merging stateful modules containing memories.** Blocked by
  `memory-identity-boundary` (contents not reset-defined ⇒ no base case). Excluded;
  named future, needs an explicit reset-defined register-file motif first.
- **Merging stateful modules containing FSM blocks as part of this proof.**
  Larger correspondence problem (FSM state + flop state product). Excluded from the
  first cut; intra-module duplicate FSMs already merge via `merge_equivalent_fsms`.
- **Merging instance-bearing (wrapper) stateful modules.** The sequential analogue
  of the bounded combinational wrapper case (composing child state into the
  product). Excluded from the first cut; the combinational wrapper proof stays
  as-is.
- **Retimed / latency-shifted whole-module equivalence.** Not bisimilar
  (observable cycle offset); needs retiming-aware reasoning. Correctly excluded; a
  far-future lane.
- **Relaxing to structural / syntactic module resemblance.** Forbidden by the lane
  non-goals, `feedback_rules_first_generation`, and the doctrinal bar
  (`hierarchy-identity-boundary` is exactly the line this must not cross).

No mode/strategy/gate is retired (`feedback_never_retire_strategies`); the deeper
memory / FSM / wrapper / retiming module-equivalence paths remain named future
leaves.

## Central implementation challenge (handed to `.3b`)

`cone_proof` (`src/ir/compact.rs`) is **module-local**: it proves cones within a
single `Module` and keys endpoints by that module's `FlopId` / `PortId`. The new
proof compares cones **across two modules**. `.3b`'s core work is a *cross-module*
proof signature: a normalized cone proof whose `LeafEndpoint`s are expressed in a
**shared** vocabulary — `PrimaryInput` keyed by `(PortId, width)` and `FlopQ`
keyed by the **global union class id** (spanning both modules) rather than a
module-local `FlopId`. The `0007`/`.2b` quotient param
(`Option<&HashMap<FlopId, FlopId>>` threaded via `canonical_flop_endpoint`) is
the template; `.3b` generalizes it to a union-class map keyed across two modules
(e.g. `(ModuleTag, FlopId) -> ClassId`). This is the reason `.3` is split into a
design leaf (`.3a`, this record) and a dedicated impl leaf (`.3b`).

## Tree split

`.3a` (this leaf) splits `.3` forward (the `.2a`/`.2b` precedent):

- **`.3a`** (this leaf) — design/decision: this record (`0008`); no source change.
- **`.3b`** (future, `proposed`) — implement the bounded whole-leaf-module
  sequential-equivalence pass: cross-module bisimulation (union partition
  refinement + cross-module cone proof signature) + bounded output-cone equality +
  the new default-off `hierarchy_sequential_module_dedup` knob + the design-level
  metric pair + the focused downstream-clean gate; default-off / byte-identical;
  banked clean across Verilator + both Yosys modes.

## Consequences

- ANVIL gains a **sound, bounded** *whole-module* sequential-equivalence merge
  class beyond the pure-combinational module boundary, lifting the stateful-module
  skip in `dedup_semantic_modules` for flops-only leaf modules, with banked
  downstream-clean evidence rather than narrative.
- The single source of generator identity stays `src/ir/compact.rs` +
  `src/ir/dedup.rs` (no second identity system); the increment adds a pass + a
  default-off knob + a metric pair, all gated.
- Soundness, bounded-budget, no-retirement, and default-off / byte-identical
  invariants are all preserved; `tests/snapshots.rs` stays byte-identical by
  default; the existing combinational module proof is untouched.
- The deeper memory / FSM / wrapper / retiming module-equivalence paths are
  explicitly preserved as future leaves of this lane.

## Open questions

- `.3b` finalizes the knob name (`hierarchy_sequential_module_dedup`), the
  design-level metric names (`sequential_module_proof` signature +
  `num_sequentially_duplicate_module_pairs`), the union flop cap
  `N_bisim_module_flops`, the exact cross-module cone-proof signature
  representation, and the gate shape (focused `cargo test` + smoke vs a dedicated
  `tool_matrix` scenario set), by whichever proves the cross-module stateful merge
  by construction at lowest cost.
- Whether the candidate-pair pre-filter should additionally hash a cheap
  structural fingerprint of each module's flop network (beyond the flop multiset
  key) to keep the `O(modules²)` comparison tight on larger designs — a
  performance choice for `.3b`, not a soundness one.

## Links

- Task-tree: `IDENTITY-DEEPENING.3a` (this leaf); frontier advances to `.3b`
- Predecessor decision: [`0007`](0007-identity-deepening-first-extension.md)
  (`IDENTITY-DEEPENING.1`/`.2` — bounded bisimulation *flop* merge; this record is
  the deferred whole-module step it named)
- North star: `project_anvil_north_star` (auto-memory)
- Doctrine: `feedback_full_factorization`, `feedback_rules_first_generation`,
  `feedback_never_retire_strategies`
- Boundaries generalized / respected:
  [`bounded-semantic-module-identity`](../knowledge/bounded-semantic-module-identity.md),
  [`hierarchy-identity-boundary`](../knowledge/hierarchy-identity-boundary.md),
  [`bisimulation-flop-merge`](../knowledge/bisimulation-flop-merge.md),
  [`reset-defined-self-hold-flop-identity`](../knowledge/reset-defined-self-hold-flop-identity.md),
  [`fsm-identity-merge`](../knowledge/fsm-identity-merge.md),
  [`memory-identity-boundary`](../knowledge/memory-identity-boundary.md),
  [`semantic-proof-budget`](../knowledge/semantic-proof-budget.md)
- Reuse: `src/ir/dedup.rs` (`dedup_semantic_modules`,
  `prune_modules_made_unreachable`), `src/ir/compact.rs` (`merge_bisimilar_flops`,
  `canonical_flop_endpoint`, `cone_proof`, `MERGE_SEMANTIC_LIMITS`),
  `src/metrics.rs` (`semantic_module_proof_inner`, the module-proof budget
  constants), `book/src/factorization.md`, `ROADMAP.md` steering gap 2
