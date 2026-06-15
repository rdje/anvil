---
id: identity-deepening-first-extension
title: The first IDENTITY-DEEPENING extension is bounded bisimulation-based sequential flop equivalence (default-off, reusing the bounded combinational endpoint proof)
answers:
  - "what is the first IDENTITY-DEEPENING extension"
  - "does ANVIL merge mutually-recursive registers"
  - "can ANVIL merge non-exact feedback flops"
  - "what sequential equivalence does ANVIL prove beyond exact self-hold"
  - "how is the bisimulation flop merge proven sound"
  - "why not use bounded model checking for flop equivalence"
  - "what is the bisimulation flop merge budget"
  - "is the bisimulation flop merge default-off"
  - "does ANVIL merge retimed state"
  - "why did IDENTITY-DEEPENING pick sequential over module equivalence first"
date: 2026-06-15
status: current
tags: [identity, sequential, factorization, bisimulation, coinduction, flop-merge]
evidence: docs/decisions/0007-identity-deepening-first-extension.md; docs/tasks/IDENTITY-DEEPENING.md; src/ir/compact.rs; book/src/factorization.md; ROADMAP.md
---

# 0007 - IDENTITY-DEEPENING first extension: bounded bisimulation-based sequential flop equivalence

- Date: 2026-06-15
- Status: accepted
- Tags: identity, sequential, factorization, bisimulation, coinduction, flop-merge

## Context

`IDENTITY-DEEPENING` (Lane 1 of the three owner-directed post-phase capability
lanes) advances the `NodeId`-as-identity / full-factorization north star
(`ROADMAP.md` steering gap 2, `feedback_full_factorization`) into the territory
left explicitly bounded by the closed identity trees. Its `.1` leaf is a
design/decision leaf that must pick **one** concrete sound identity extension,
fix its proof discipline + budget + downstream gate, and split the tree, before
any merge code lands. The doctrinal bar is strong and unchanged: two structures
share one identity **only** when ANVIL can *prove* they implement the same
functionality with respect to the same canonical leaf endpoints — never
syntactic resemblance, never an unsound or unbounded merge
(`feedback_rules_first_generation`, `feedback_never_retire_strategies`).

### Current identity proof surface (inventory, current code)

- **Combinational** (`src/ir/types.rs` intern ladder + `src/ir/compact.rs`
  `merge_equivalent_gates`): normalized identity through the
  associative/commutative/constant-fold/peephole/CSE ladder, plus a bounded
  post-construction semantic gate merge over the same canonical leaf endpoints
  (truth-table proof; budget = ≤ 12 endpoint-support bits, ≤ 128 cone nodes, ≤
  128-bit width, `assignment_count * cone_node_count <= 131072`;
  `semantic-proof-budget`).
- **Sequential flops** (`merge_equivalent_flops`): conservative post-drain
  merge. Two flops merge only on same `width` / `reset_kind` / `reset_val` /
  `Module::flop_domain` **and** either (a) their D-cones prove combinationally
  equal over the **same** canonical `FlopQ`/`PrimaryInput` endpoints, or (b)
  exact reset-defined self-hold (`D == own Q`) on both sides
  (`reset-defined-self-hold-flop-identity`). Recorded no-merge boundary:
  *mutually-recursive registers, retimed state, and non-exact feedback forms*.
- **FSMs** (`merge_equivalent_fsms`): deterministic, table-defined,
  reset-to-state-0 FSM blocks merge on matching selector proof + encoding +
  tables (`fsm-identity-merge`).
- **Memories**: deliberately opaque / state-by-instance — array contents are not
  reset-defined; **explicitly not reopened** here (`memory-identity-boundary`).
- **Module / hierarchy** (`src/ir/dedup.rs`): `dedup_modules` is structural
  (canonical signature; `hierarchy-identity-boundary`); `dedup_semantic_modules`
  already proves **bounded whole-module truth-table equivalence for
  pure-combinational** leaves and bounded combinational wrappers
  (`bounded-semantic-module-identity`). Stateful / memory / FSM / parameterized /
  aggregate-projected modules are skipped.

### The gap chosen

The cleanest, soundest first step into new territory is **broader sequential
equivalence at the flop level**: merging flops whose D-cones reference *different
but provably-corresponding* state, which is exactly the recorded
*mutually-recursive-register / non-exact-feedback* no-merge boundary. Bounded
**module-level** semantic equivalence already exists for the pure-combinational
case; the genuinely open, high-value, soundly-bounded frontier is sequential.

## Decision

**The first `IDENTITY-DEEPENING` extension is a bounded
bisimulation-based sequential flop merge: a default-off, opt-in
greatest-fixpoint partition refinement over flops that reuses the existing
bounded combinational endpoint proof to compare D-cones *up to a state
correspondence*.** It captures the mutually-recursive-register and non-exact
feedback class the current exact self-hold rule cannot, while staying sound and
budget-bounded. It strictly generalizes — and does not retire — both existing
sound sequential classes.

### Proof discipline (soundness argument)

Treat each flop as a state element of a Mealy/Moore machine whose next-state
function is its D-cone over `(PrimaryInputs ∪ FlopQs)`.

1. **Bucketing (base case).** Partition flops by `(width, reset_kind,
   reset_val, flop_domain)`. Flops in different buckets are never identified
   (different reset value ⇒ the bisimulation base case fails; different clock
   domain ⇒ identifying them is unsound).
2. **Greatest-fixpoint refinement (step).** Within a bucket, keep two flops in
   the same class iff their D-cones — with **every `FlopQ` endpoint rewritten to
   its current class representative** — are proven combinationally equal by the
   existing endpoint-preserving bounded proof, taken over the quotient endpoint
   set `(PrimaryInputs ∪ class-ids)`. Refine until no class splits
   (Kanellakis–Smolka / Hopcroft-style coarsest stable partition).
3. **Soundness (coinduction).** At the fixpoint the partition is a bisimulation.
   By induction on cycle `t`: at `t = 0` every class member holds `Q =
   reset_val` (equal, by bucketing); if at cycle `t` every class's members hold
   equal `Q`, then for `f, g` in a class their quotient-substituted D-cones
   evaluate equally, so `Q_f(t+1) = Q_g(t+1)`. Hence corresponding Qs are equal
   for all time, so merging them is observationally sound.
4. **Generalization, not replacement.** The exact self-hold class (`D == Q`) and
   the same-endpoint D-cone merge are the special cases where the correspondence
   is the identity on endpoints; the *new* content is identifying flops whose
   D-cones reference different-but-corresponding Qs (mutual recursion, swapped
   feedback). Nothing existing is retired.
5. **Correctly excluded.** Retimed / latency-shifted state (observable cycle
   offset ⇒ not bisimilar), reset-value / domain / width mismatches (base case
   fails), and any cone exceeding the proof budget (see below) → the candidate
   conservatively **splits** to a singleton class ⇒ no merge.

### Budget

- **Per D-cone equivalence check:** reuse the existing bounded combinational
  proof and its budget verbatim — width ≤ 128 bits, canonical endpoint support ≤
  12 bits, ≤ 128 unique cone nodes, `assignment_count * cone_node_count <=
  131072`. A check that cannot be discharged within budget yields "not proven" ⇒
  the candidate splits (never a guess).
- **Refinement cost:** bounded by `O(k² · iterations)` with `iterations ≤ k`
  (each refinement step is monotone in class count, capped at `k` = bucket
  size). Buckets larger than a calibration cap `N_bisim_flops` fall back to the
  current exact merge only, so the pass cannot blow up on pathological modules.
  The concrete cap is set empirically at implementation.

### Control surface (default-off / byte-identical)

- Gate the **additional** merges behind a new default-off `Config` knob (working
  name `bisimulation_flop_merge`, finalized at `.2`), parallel to the existing
  opt-in `hierarchy_module_dedup` / `hierarchy_semantic_module_dedup` knobs.
  Additionally require `identity_mode = node-id` and effective
  `factorization_level = e-graph` (the rung that already gates semantic gate
  merge). When the knob is off (default), emitted RTL is **byte-identical** —
  `tests/snapshots.rs` is untouched — and `--identity-mode relaxed` stays the
  real off-switch.

### Downstream gate

- A merge-count metric (working name `bisimulation_flops_merged`) on
  `Module`/`Metrics`, RTL-invisible.
- A focused, rules-first scenario/test that constructs a design with
  deliberately-duplicated **mutually-recursive** register pairs whose
  equivalence the exact self-hold rule provably cannot prove; with the knob on,
  assert the merge count `> 0` and prove the merged output **clean across
  Verilator + both Yosys modes** (a banked `tool_matrix`/smoke report per the
  signoff discipline). Regression-protect: knob-off byte-identical, and the
  existing exact self-hold / same-endpoint / FSM classes still merge.

## Decisive test applied

"A merge must be a *proof*, not a heuristic, and a day-one gate must be clean by
construction." Bisimulation from a reset base case is a textbook *sound* proof of
sequential equivalence; the partition-refinement bound and the reused
combinational budget make it cheap and terminating; and the duplicated-state
scenario produces higher legal sharing (one register driving consumers that were
syntactically distinct) — exactly the kind of unusual-but-valid structure the
north star wants downstream tools to ingest.

## Rejected alternatives (as first)

- **Whole stateful-leaf-module bounded sequential equivalence via
  reachable-product-state exploration.** Sound but a larger jump; it naturally
  *builds on* the flop-level bisimulation primitive plus a state-correspondence
  search. **Kept as a named future leaf**, not retired.
- **Bounded model checking (k-step) equivalence.** Proves agreement only up to
  depth `k`, not for all time — **unsound as a merge proof**. Rejected outright;
  it violates the soundness bar.
- **Retimed / latency-shifted state equivalence.** Not bisimilar (observable
  cycle offset); needs retiming-aware reasoning. Correctly excluded from this
  class; a possible far-future lane, not now.
- **Memory-state merging.** Blocked by `memory-identity-boundary` (contents not
  reset-defined); reopening needs an explicit, downstream-clean reset-defined
  register-file motif first. Not this lane's first step.
- **Relaxing to structural / syntactic resemblance.** Forbidden by the lane
  non-goals, `feedback_rules_first_generation`, and the doctrinal bar.

No mode/strategy/gate is retired (`feedback_never_retire_strategies`); the
higher-ceiling sequential and module-level paths remain named future leaves.

## Tree split

`.1` (this leaf) splits the tree forward:

- **`.2`** — implement the bounded bisimulation flop merge: partition refinement
  + bounded quotient D-cone proof + the new default-off knob + the merge-count
  metric + the focused downstream-clean gate; default-off / byte-identical;
  banked clean across Verilator + both Yosys modes. To be split into `.2a`
  design-detail + `.2b` impl if it proves broad (the `.3a`/`.3b` precedent).
- **`.3` (future, `proposed`)** — whole stateful-leaf-module bounded sequential
  equivalence built on the `.2` primitive + a bounded state-correspondence
  search (extends `dedup_semantic_modules` past the pure-combinational
  boundary). Named, not active.

## Consequences

- ANVIL gains a **sound, bounded** sequential-equivalence merge class beyond
  exact reset-defined self-hold, lifting the recorded
  mutually-recursive-register / non-exact-feedback no-merge boundary at the flop
  level, with banked downstream-clean evidence rather than narrative.
- The single source of generator identity stays `src/ir/compact.rs`; the
  increment adds a pass + a default-off knob + a metric, not a second identity
  system.
- Soundness, bounded-budget, no-retirement, and default-off / byte-identical
  invariants are all preserved; `tests/snapshots.rs` stays byte-identical by
  default.
- The deeper module-level and retiming paths are explicitly preserved as future
  leaves of this lane.

## Open questions

- `.2` finalizes the knob name, the merge-count metric name, the bucket-size cap
  `N_bisim_flops`, and the exact scenario/gate shape (focused `cargo test` +
  smoke vs a dedicated `tool_matrix` scenario set).

## Links

- Task-tree: `IDENTITY-DEEPENING.1` (this leaf); frontier advances to `.2`
- Predecessor lanes: decisions [`0005`](0005-agent-mcp-expansion-surface.md)
  (`AGENT-MCP-EXPANSION`, closed) and
  [`0006`](0006-signoff-automation-first-increment.md)
  (`SIGNOFF-AUTOMATION-EXPANSION`, handoff)
- North star: `project_anvil_north_star` (auto-memory)
- Doctrine: `feedback_full_factorization`, `feedback_rules_first_generation`,
  `feedback_never_retire_strategies`
- Boundaries generalized / respected:
  [`reset-defined-self-hold-flop-identity`](../knowledge/reset-defined-self-hold-flop-identity.md),
  [`fsm-identity-merge`](../knowledge/fsm-identity-merge.md),
  [`semantic-proof-budget`](../knowledge/semantic-proof-budget.md),
  [`memory-identity-boundary`](../knowledge/memory-identity-boundary.md),
  [`bounded-semantic-module-identity`](../knowledge/bounded-semantic-module-identity.md)
- Reuse: `src/ir/compact.rs` (`merge_equivalent_flops`, the bounded
  combinational proof), `book/src/factorization.md`, `ROADMAP.md` steering gap 2
