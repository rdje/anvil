# Development Notes
Engineering rationale behind design decisions. The "why" that does not belong in code comments and is too detailed for `MEMORY.md`.

For the canonical statement of the algorithm and load-bearing decisions, see `book/src/`. This file is the contributor-facing scratchpad: rejected alternatives, calibration notes, gotchas, and the reasoning behind small choices the book does not cover.

---

## Core design decisions (recap)

These are documented in detail in the mdBook. They are restated here only as anchors:

- **Recursion is the core principle.** Every non-trivial generation step is a recursive descent over the typed circuit graph. Iteration is the exception, used only where termination or ordering genuinely require it (e.g., the flop worklist drainer, the per-output driver loop). When in doubt, recurse. See `book/src/core-idea.md` "The single guiding principle".
- **Synchronous-design discipline.** Every module is fully synchronous to a single clock domain: one `clk` (posedge), one `rst_n` (async, active-low), every flop emitted into one `always_ff` block. Enforced by construction — there is no IR field for per-flop clock or per-flop reset polarity. See `book/src/sequential.md` "Synchronous-design discipline".
- **Flop-D mux motifs.** Every flop's D input is constructed from one of: M=0 (direct cone), M≥2 OneHot (OR-of-masked arms), M≥2 Encoded (chained ternary over `Eq(sel, k)`). M=1 is excluded by design; it collapses to a wire. The style (OneHot vs Encoded) and kind (ZeroDefault vs QFeedback) are chosen per-flop and orthogonal — four motif variants plus the M=0 plain register. See `book/src/sequential.md` "Flop motifs".
- **Q-feedback freedom (revised).** A flop's own Q may appear freely — any number of times — as a leaf in any of its data, select, or direct-D sub-cones. The clock edge breaks the Q→D loop temporally; this is the standard synchronous feedback pattern (counters, accumulators, state machines). Independently, `FlopKind::QFeedback` adds an explicit Q fall-through term in the mux when no select fires. Both are legal; both can be active at the same flop. Combinational self-reference (Rule 1) is still forbidden. See `book/src/structural-rules.md` Rules 2 and 3.
- **Structural rules catalog.** Every load-bearing generator invariant is documented in `book/src/structural-rules.md`. That chapter is the durable source of truth — new rules land there as they become invariants. Inline design-decision recaps in this file should *point* to the catalog, not duplicate rule text.
- **Operators vs blocks.** Load-bearing conceptual distinction. An operator is an associative primitive function; its generalization is **arity** (N same-width operands). A block is a functional unit with internal structure; its generalization is **ports / port counts / arms**, encoding choices, feedback topology. Arity is operator vocabulary only — blocks have ports, not arity. `And / Or / Xor / Add / Mul` are operators and got N-arity in `2026-04-15-0015`. `Sub` is not associative and stays 2-arity. `Mux` and `Flop` are blocks and are governed by block rules, not arity knobs. See `book/src/structural-rules.md` "Operators vs blocks" preamble and Rule 14.
- **Roles of constants in RTL.** Integer literals appear as operands with three *distinct* semantic roles: **coefficient** (multiplicative weight in arithmetic linear combinations; per-op constraints: Add `ci ≠ 0`, Sub `ci > 0` strictly positive, Mul TBD), **shift amount** (structural parameter of `Shl/Shr` — `a << 2`; constant-amount vs variable-amount are both legal, with real designs biased heavily toward constant), and **comparand** (threshold / sentinel on the RHS of a comparison — `a == 7`; additive to signal-vs-signal comparisons, not a replacement). These three are *not interchangeable*: each has its own motif family, its own constraints, and its own knob(s). Do not unify them under a single `constant_prob` knob — doing so loses the semantic distinctions. See `book/src/structural-rules.md` "Roles of constants in RTL".
- **Construction strategies.** Three live strategies construct a
  module's internal logic: `sequential` (per-output cone recursion in
  declaration order), `shuffled` (same, randomised output order), and
  `interleaved` (frames interleaved via random-pop work queue — cones
  grow in lockstep). `graph-first` remains as a deprecated CLI/config
  alias for `interleaved`; the original speculative pool-growth
  implementation is retired. The strategy is a property of **how** the
  generator builds; the emitted SV is a DAG regardless. Different
  strategies produce different output *distributions*
  (declaration-order bias, within-module sharing symmetry). See
  `book/src/construction-strategies.md`.
- **Circuit IR over annotated EBNF.** The generator builds a typed circuit graph and emits SV from it. See `book/src/why-not-grammar.md`.
- **Generation by construction, not generate-then-filter.** Validity is structural; the validator is a safety net, not a gate. See `book/src/by-construction.md`.
- **Synthesizability is a subset constraint.** The gate set, flop
  pattern, and emitter cover only the synthesizable subset. Broader
  artifact families must keep that contract too; the project is
  broadening to more kinds of valid-by-construction synthesizable
  artifacts, not abandoning synthesizability. See
  `book/src/synthesizability.md`.
- **Non-triviality via dep-set tracking + structural anti-collapse
  rules.** No bundled oracle. Expected-facts manifests for specific
  artifact families are acceptable; a shadow simulator used as a global
  filter is not. See `book/src/non-triviality.md`.
- **Random by-construction synthesizable RTL is the product goal.**
  `anvil` is not trying to be merely "valid enough". The target is a
  signoff-level quality random synthesizable RTL generator whose outputs
  are accepted by mainstream downstream HDL consumers by default and
  remain rich enough to expose real bugs in parsers, elaborators, RTL
  compilers, linters, simulators, synthesizers, and similar tools.
  Feature growth and downstream-acceptance robustness are both
  first-class; neither is optional garnish for the other.
- **No oracle, no reference simulator.** `anvil` is still a generator,
  not a bundled shadow simulator. It can stress downstream tools by
  emitting high-quality legal RTL and explicit expected-facts contracts
  where appropriate, not by embedding a second implementation of RTL
  semantics. See `book/src/non-goals.md`.

If you need to revise any of these, that is a deliberate task with its own commit and a `DEVELOPMENT_NOTES.md` entry.

---

## Calibration notes

### Phase 4 r48 banks recursive registered helper mixed-support routing downstream-clean
The live Phase 4 hierarchy policy now requires the recursive
registered parent-composed helper route to carry parent-port support in
the same D cone below the top parent. In an exact-depth-2 recursive
hierarchy, a parent-cone helper instance can feed registered
parent-composed child-input logic, that logic can also consume parent
data ports, and the resulting parent-local Q can bind a later child
input. The focused regression is
`cargo test recursive_hierarchy_registered_helper_routes_mix_parent_ports_below_top`.

This needed a dedicated helper-mixed metric instead of inferring the
fact from the older registered helper and registered mixed-support
counters. Those counters can both be true in a design without proving
that parent-port support and the parent-cone helper output occur in the
same registered D cone. The new
`registered_parent_cone_instance_mixed_support_*` counters make that
overlap explicit.

The current full downstream-clean evidence anchor is
`/tmp/anvil-tool-matrix-phase4-hierarchy-r48/tool_matrix_report.json`:
`99` scenarios, `4` designs/scenario, `396` total designs,
`coverage_gaps = []`,
`saw_recursive_hierarchy_registered_parent_cone_instance_mixed_support_routing = true`,
`saw_recursive_hierarchy_registered_parent_composed_parent_cone_instance_routing = true`,
`saw_recursive_hierarchy_registered_multistage_mixed_support_routing = true`,
`saw_recursive_hierarchy_registered_multistage_sibling_routing = true`,
`saw_recursive_hierarchy_registered_multistage_routing = true`,
`saw_recursive_hierarchy_registered_mixed_support_routing = true`,
`saw_hierarchy_registered_multistage_routing = true`,
`saw_hierarchy_registered_multistage_sibling_routing = true`,
`saw_recursive_hierarchy_registered_multistage_parent_cone_instance_routing = true`,
`saw_recursive_hierarchy_registered_multistage_parent_composed_parent_cone_instance_routing = true`,
and
`saw_recursive_hierarchy_registered_parent_composed_parent_cone_instance_routing = true`
with `396/0` pass-fail in Verilator plus both repo-owned Yosys modes.
The previous recursive non-top registered multistage mixed-support
no-helper full bank is `r47`.

### Phase 4 r47 banks recursive registered multistage mixed-support routing downstream-clean
The live Phase 4 hierarchy policy now requires the recursive no-helper
overlap between registered mixed support and multi-stage registered
parent-composed routing. Below the top parent, an exact-depth-2
recursive hierarchy can build a registered D cone that simultaneously
uses parent data ports, child instance outputs, and an earlier
parent-local Q, then bind a later child input through the resulting
parent-local state without relying on parent-cone helper instances. The
focused regression is
`cargo test recursive_hierarchy_registered_multistage_mixed_support_routes_below_top`.

This needed a dedicated metric instead of inferring the fact from the
existing mixed-support and multistage counters. Those older counters can
be true in the same design while describing different bindings; the new
`registered_multistage_mixed_support_*` counters only fire when one
registered route contains both kinds of support in the same D cone and
then participates in later Q reuse.

The `r47` full downstream-clean evidence anchor is
`/tmp/anvil-tool-matrix-phase4-hierarchy-r47/tool_matrix_report.json`:
`99` scenarios, `4` designs/scenario, `396` total designs,
`coverage_gaps = []`,
`saw_recursive_hierarchy_registered_multistage_mixed_support_routing = true`,
`saw_recursive_hierarchy_registered_multistage_sibling_routing = true`,
`saw_recursive_hierarchy_registered_multistage_routing = true`,
`saw_recursive_hierarchy_registered_mixed_support_routing = true`,
`saw_hierarchy_registered_multistage_routing = true`,
`saw_hierarchy_registered_multistage_sibling_routing = true`,
`saw_recursive_hierarchy_registered_multistage_parent_cone_instance_routing = true`,
`saw_recursive_hierarchy_registered_multistage_parent_composed_parent_cone_instance_routing = true`,
and
`saw_recursive_hierarchy_registered_parent_composed_parent_cone_instance_routing = true`
with `396/0` pass-fail in Verilator plus both repo-owned Yosys modes.
The previous recursive non-top registered sibling multistage no-helper
full bank is `r46`.

### Cargo default-run is part of the README contract
The repository has two binaries: the generator (`anvil`) and the
auxiliary `tool_matrix` harness. Cargo cannot infer which one plain
`cargo run -- ...` should execute unless `Cargo.toml` keeps
`default-run = "anvil"` in the `[package]` section.

This is a user-facing contract, not cosmetic metadata: README and the
mdBook intentionally teach `cargo run -- ...` for generator examples,
while `tool_matrix` is always selected explicitly with
`cargo run --bin tool_matrix -- ...`. Future auxiliary binaries must
preserve that default-run setting or update every source-tree command
surface in the live docs at the same time.

### Phase 4 r46 banks recursive registered sibling multistage routing downstream-clean
The live Phase 4 hierarchy policy now requires the recursive no-helper
multi-stage registered sibling-routed child-input cross product. Below
the top parent, an exact-depth-2 recursive hierarchy can bind one child
input from an earlier child output through parent-local state, then
reuse that earlier parent-local Q as the D source for a later direct
registered sibling route, without relying on parent-composed D logic or
parent-cone helper instances. The focused regression is
`cargo test recursive_hierarchy_registered_sibling_routes_can_chain_without_helpers_below_top`.
It uses four child instances per recursive parent so the sibling-output
route has enough earlier sources to force both the first registered
binding and the later Q-reuse binding across every construction
strategy.

The current full downstream-clean evidence anchor is
`/tmp/anvil-tool-matrix-phase4-hierarchy-r46/tool_matrix_report.json`:
`99` scenarios, `4` designs/scenario, `396` total designs,
`coverage_gaps = []`,
`saw_recursive_hierarchy_registered_multistage_sibling_routing = true`,
`saw_recursive_hierarchy_registered_multistage_routing = true`,
`saw_recursive_hierarchy_registered_mixed_support_routing = true`,
`saw_hierarchy_registered_multistage_routing = true`,
`saw_hierarchy_registered_multistage_sibling_routing = true`,
`saw_recursive_hierarchy_registered_multistage_parent_cone_instance_routing = true`,
`saw_recursive_hierarchy_registered_multistage_parent_composed_parent_cone_instance_routing = true`,
and
`saw_recursive_hierarchy_registered_parent_composed_parent_cone_instance_routing = true`
with `396/0` pass-fail in Verilator plus both repo-owned Yosys modes.
The previous recursive non-top registered parent-composed multistage
no-helper full bank is `r45`.

### Phase 4 r45 banks recursive registered multistage routing downstream-clean
The live Phase 4 hierarchy policy now requires the recursive no-helper
multi-stage registered parent-composed cross product. Below the top
parent, an exact-depth-2 recursive hierarchy can first bind a child input
through parent-local state, then reuse that earlier parent-local Q in a
later registered parent-composed child-input D cone, without relying on
parent-cone helper instances. The focused regression is
`cargo test recursive_hierarchy_registered_parent_composed_routes_can_chain_without_helpers_below_top`.
It uses four child instances per recursive parent because the two-child
registered mixed-support calibration is too sparse to force this
multi-stage subcase across every construction strategy.

The current full downstream-clean evidence anchor is
`/tmp/anvil-tool-matrix-phase4-hierarchy-r45/tool_matrix_report.json`:
`96` scenarios, `4` designs/scenario, `384` total designs,
`coverage_gaps = []`,
`saw_recursive_hierarchy_registered_multistage_routing = true`,
`saw_recursive_hierarchy_registered_mixed_support_routing = true`,
`saw_hierarchy_registered_multistage_routing = true`,
`saw_recursive_hierarchy_registered_multistage_parent_cone_instance_routing = true`,
`saw_recursive_hierarchy_registered_multistage_parent_composed_parent_cone_instance_routing = true`,
and
`saw_recursive_hierarchy_registered_parent_composed_parent_cone_instance_routing = true`
with `384/0` pass-fail in Verilator plus both repo-owned Yosys modes.
The previous recursive non-top registered mixed-support full bank is
`r44`.

### Phase 4 r44 banks recursive registered mixed-support routing downstream-clean
The live Phase 4 hierarchy policy now requires the recursive no-helper
registered mixed-support cross product. Below the top parent, an
exact-depth-2 recursive hierarchy can build registered parent-composed
child-input D logic from both parent data ports and child outputs, then
drive later child inputs through parent-local state without relying on
parent-cone helper instances. The focused regression is
`cargo test recursive_hierarchy_registered_mixed_support_routes_below_top`.
It requires the recursive tree shape, non-top parent-local flops,
non-top registered parent-composed child-input bindings, non-top
registered child-output support, non-top registered mixed-support
bindings, and zero registered helper-sourced D-cone bindings.

The previous full downstream-clean evidence anchor is
`/tmp/anvil-tool-matrix-phase4-hierarchy-r44/tool_matrix_report.json`:
`93` scenarios, `4` designs/scenario, `372` total designs,
`coverage_gaps = []`,
`saw_recursive_hierarchy_registered_mixed_support_routing = true`,
`saw_hierarchy_registered_mixed_support_routing = true`,
`saw_recursive_multiple_parent_cone_instances_per_parent_child_inputs = true`,
`saw_recursive_multiple_parent_cone_instances_per_parent_through_flops = true`,
`saw_recursive_multiple_parent_cone_instances_per_parent = true`,
`saw_recursive_hierarchy_registered_multistage_parent_cone_instance_routing = true`,
`saw_recursive_hierarchy_registered_multistage_parent_composed_parent_cone_instance_routing = true`,
and
`saw_recursive_hierarchy_registered_parent_composed_parent_cone_instance_routing = true`
with `372/0` pass-fail in Verilator plus both repo-owned Yosys modes.
The previous recursive non-top child-input multi-helper budget full bank
is `r43`.

### Phase 4 r43 banks recursive non-top child-input helper budgets downstream-clean
The live Phase 4 hierarchy policy now closes the child-input local-budget
cross product for recursive parent-cone helpers. Below the top parent,
an exact-depth-2 recursive hierarchy can spend a multi-helper
`max_parent_cone_instances_per_module = 3` budget while driving
parent-composed child-input bindings directly from helper outputs. The
focused regression is
`cargo test recursive_hierarchy_parent_cone_helper_budget_allows_multiple_helpers_below_top`.
It requires the recursive tree shape, the configured helper budget in
`max_parent_cone_instances_per_internal_module`, helper instances beyond
the top parent, non-top parent-composed child-input bindings, non-top
child-input bindings sourced from helper outputs, and zero
helper-through-flop or registered helper child-input bindings.

The current full downstream-clean evidence anchor is
`/tmp/anvil-tool-matrix-phase4-hierarchy-r43/tool_matrix_report.json`:
`90` scenarios, `4` designs/scenario, `360` total designs,
`coverage_gaps = []`,
`saw_recursive_multiple_parent_cone_instances_per_parent_child_inputs = true`,
`saw_recursive_multiple_parent_cone_instances_per_parent_through_flops = true`,
`saw_recursive_multiple_parent_cone_instances_per_parent = true`,
`saw_multiple_parent_cone_instances_per_parent = true`,
`saw_recursive_hierarchy_parent_cone_instance_flop_outputs = true`,
`saw_recursive_hierarchy_parent_cone_instance_outputs = true`,
`saw_recursive_hierarchy_direct_sibling_parent_cone_instance_routing = true`,
`saw_recursive_hierarchy_direct_registered_sibling_parent_cone_instance_routing = true`,
`saw_recursive_hierarchy_registered_multistage_parent_cone_instance_routing = true`,
`saw_recursive_hierarchy_registered_multistage_parent_composed_parent_cone_instance_routing = true`,
`saw_recursive_hierarchy_registered_parent_composed_parent_cone_instance_routing = true`,
and
`saw_recursive_hierarchy_parent_composed_parent_cone_instance_flop_routing = true`
with `360/0` pass-fail in Verilator plus both repo-owned Yosys modes.
The previous recursive non-top stateful multi-helper budget full bank is
`r42`.

### Phase 4 r42 banks recursive non-top stateful helper budgets downstream-clean
Phase 4 r42 closed the stateful local-budget
cross product for recursive parent-output helpers. Below the top parent,
an exact-depth-2 recursive hierarchy can spend a multi-helper
`max_parent_cone_instances_per_module = 3` budget, register the helper
outputs into parent-local flops, and drive parent outputs from those
helper-sourced Qs. The focused regression is
`cargo test recursive_hierarchy_parent_outputs_can_spend_stateful_helper_budget_below_top`.
It requires the recursive tree shape, the configured helper budget in
`max_parent_cone_instances_per_internal_module`, helper instances beyond
the top parent, parent-local flops below the top parent, parent outputs
that depend on helper outputs through those flops, and zero child-input
helper bindings through either direct, stateful, or registered helper
routes.

The full downstream-clean evidence anchor for that slice was
`/tmp/anvil-tool-matrix-phase4-hierarchy-r42/tool_matrix_report.json`:
`87` scenarios, `4` designs/scenario, `348` total designs,
`coverage_gaps = []`,
`saw_recursive_multiple_parent_cone_instances_per_parent_through_flops = true`,
`saw_recursive_multiple_parent_cone_instances_per_parent = true`,
`saw_multiple_parent_cone_instances_per_parent = true`,
`saw_recursive_hierarchy_parent_cone_instance_flop_outputs = true`,
`saw_recursive_hierarchy_parent_cone_instance_outputs = true`,
`saw_recursive_hierarchy_direct_sibling_parent_cone_instance_routing = true`,
`saw_recursive_hierarchy_direct_registered_sibling_parent_cone_instance_routing = true`,
`saw_recursive_hierarchy_registered_multistage_parent_cone_instance_routing = true`,
`saw_recursive_hierarchy_registered_multistage_parent_composed_parent_cone_instance_routing = true`,
`saw_recursive_hierarchy_registered_parent_composed_parent_cone_instance_routing = true`,
and
`saw_recursive_hierarchy_parent_composed_parent_cone_instance_flop_routing = true`
with `348/0` pass-fail in Verilator plus both repo-owned Yosys modes.
The previous recursive non-top multi-helper budget full bank is `r41`.

### Phase 4 r41 banks recursive non-top helper budgets downstream-clean
The live Phase 4 hierarchy policy now names the local-budget half of the
recursive parent-output helper surface. Below the top parent, an
exact-depth-2 recursive hierarchy can spend a multi-helper
`max_parent_cone_instances_per_module = 3` budget for parent-output
composition, not just accumulate one helper per parent across multiple
parents. The focused regression is
`cargo test recursive_hierarchy_parent_outputs_can_spend_helper_budget_below_top`.
It requires the recursive tree shape, the configured helper budget in
`max_parent_cone_instances_per_internal_module`, helper instances beyond
the top parent, parent outputs that depend on those helper outputs, no
child-input helper bindings, and no registered child-input helper D
cones.

The full downstream-clean evidence anchor for that slice was
`/tmp/anvil-tool-matrix-phase4-hierarchy-r41/tool_matrix_report.json`:
`87` scenarios, `4` designs/scenario, `348` total designs,
`coverage_gaps = []`,
`saw_recursive_multiple_parent_cone_instances_per_parent = true`,
`saw_multiple_parent_cone_instances_per_parent = true`,
`saw_recursive_hierarchy_parent_cone_instance_flop_outputs = true`,
`saw_recursive_hierarchy_parent_cone_instance_outputs = true`,
`saw_recursive_hierarchy_direct_sibling_parent_cone_instance_routing = true`,
`saw_recursive_hierarchy_direct_registered_sibling_parent_cone_instance_routing = true`,
`saw_recursive_hierarchy_registered_multistage_parent_cone_instance_routing = true`,
`saw_recursive_hierarchy_registered_multistage_parent_composed_parent_cone_instance_routing = true`,
`saw_recursive_hierarchy_registered_parent_composed_parent_cone_instance_routing = true`,
and
`saw_recursive_hierarchy_parent_composed_parent_cone_instance_flop_routing = true`
with `348/0` pass-fail in Verilator plus both repo-owned Yosys modes.
The previous recursive non-top stateful parent-output helper full bank
is `r40`.

### Phase 4 r40 banks recursive non-top stateful parent-output helper routing downstream-clean
The live Phase 4 hierarchy policy now includes the stateful
parent-output version of the recursive exact-depth-2 helper axis: below
the top parent, a non-top parent can instantiate helper children as
internal parent-cone sources, register those helper outputs into
parent-local flops, and drive parent outputs from the helper-sourced
state. The focused regression is
`cargo test recursive_hierarchy_parent_outputs_can_route_helper_instances_through_parent_flops_below_top`.
It requires the recursive tree shape, more helper instances below top
than at top, more parent-local flops below top than at top, more
helper-through-flop parent-output support across the hierarchy than at
top, no child-input helper bindings, and no registered child-input
helper D cones so the route stays distinct from both child-input helper
routing and direct recursive parent-output helper routing.

The full downstream-clean evidence anchor for that slice was
`/tmp/anvil-tool-matrix-phase4-hierarchy-r40/tool_matrix_report.json`:
`87` scenarios, `4` designs/scenario, `348` total designs,
`coverage_gaps = []`,
`saw_recursive_hierarchy_parent_cone_instance_flop_outputs = true`,
`saw_recursive_hierarchy_parent_cone_instance_outputs = true`,
`saw_recursive_hierarchy_direct_sibling_parent_cone_instance_routing = true`,
`saw_recursive_hierarchy_direct_registered_sibling_parent_cone_instance_routing = true`,
`saw_recursive_hierarchy_registered_multistage_parent_cone_instance_routing = true`,
`saw_recursive_hierarchy_registered_multistage_parent_composed_parent_cone_instance_routing = true`,
`saw_recursive_hierarchy_registered_parent_composed_parent_cone_instance_routing = true`,
and
`saw_recursive_hierarchy_parent_composed_parent_cone_instance_flop_routing = true`
with `348/0` pass-fail in Verilator plus both repo-owned Yosys modes.
The previous recursive non-top parent-output helper full bank is `r39`.

### Phase 4 r39 banks recursive non-top parent-output helper routing downstream-clean
The live Phase 4 hierarchy policy now includes the parent-output version
of the recursive exact-depth-2 helper axis: below the top parent, a
non-top parent can instantiate helper children as internal parent-cone
sources and drive its own parent outputs from those helper outputs. The
focused regression is
`cargo test recursive_hierarchy_parent_outputs_can_depend_on_helper_instances_below_top`.
It requires the recursive tree shape, more helper instances below top
than at top, more parent outputs reaching helper instances across the
hierarchy than at top, no child-input helper bindings, and no
helper-through-parent-flop output counts so the route stays distinct
from child-input helper routing and stateful parent-output helper
routing.

The full downstream-clean evidence anchor for that slice was
`/tmp/anvil-tool-matrix-phase4-hierarchy-r39/tool_matrix_report.json`:
`84` scenarios, `4` designs/scenario, `336` total designs,
`coverage_gaps = []`,
`saw_recursive_hierarchy_parent_cone_instance_outputs = true`,
`saw_recursive_hierarchy_direct_sibling_parent_cone_instance_routing = true`,
`saw_recursive_hierarchy_direct_registered_sibling_parent_cone_instance_routing = true`,
`saw_recursive_hierarchy_registered_multistage_parent_cone_instance_routing = true`,
`saw_recursive_hierarchy_registered_multistage_parent_composed_parent_cone_instance_routing = true`,
`saw_recursive_hierarchy_registered_parent_composed_parent_cone_instance_routing = true`,
and
`saw_recursive_hierarchy_parent_composed_parent_cone_instance_flop_routing = true`
with `336/0` pass-fail in Verilator plus both repo-owned Yosys modes.
The previous recursive multi-stage registered parent-composed helper
full bank is `r38`.

### Phase 4 r38 banks recursive non-top multi-stage registered parent-composed helper routing downstream-clean
The live Phase 4 hierarchy policy now includes the multi-stage
registered parent-composed version of the recursive exact-depth-2 helper
axis: below the top parent, a parent-cone helper output can seed a
parent-local Q, and later registered parent-composed D logic can reuse
that helper-sourced Q before driving a later child input. The focused
regression is
`cargo test recursive_hierarchy_registered_parent_composed_routes_can_chain_helper_instances_below_top`.
It requires the recursive tree shape, more helper instances below top
than at top, more multi-stage registered parent-composed bindings below
top than at top, more multi-stage helper-sourced parent-composed
bindings below top than at top, local parent flops below top, and zero
direct multi-stage registered helper counters so the route stays
distinct from the direct registered sibling helper-chain axis.

The full downstream-clean evidence anchor for that slice was
`/tmp/anvil-tool-matrix-phase4-hierarchy-r38/tool_matrix_report.json`:
`81` scenarios, `4` designs/scenario, `324` total designs,
`coverage_gaps = []`,
`saw_recursive_hierarchy_direct_sibling_parent_cone_instance_routing = true`,
`saw_recursive_hierarchy_direct_registered_sibling_parent_cone_instance_routing = true`,
`saw_recursive_hierarchy_registered_multistage_parent_cone_instance_routing = true`,
`saw_recursive_hierarchy_registered_multistage_parent_composed_parent_cone_instance_routing = true`,
`saw_recursive_hierarchy_registered_parent_composed_parent_cone_instance_routing = true`,
and
`saw_recursive_hierarchy_parent_composed_parent_cone_instance_flop_routing = true`
with `324/0` pass-fail in Verilator plus both repo-owned Yosys modes.
The previous recursive multi-stage direct registered-helper full bank is
`r37`.

### Phase 4 r37 banks recursive non-top multi-stage direct registered helper routing downstream-clean
The live Phase 4 hierarchy policy now includes the multi-stage version
of the recursive exact-depth-2 direct registered helper axis: below the
top parent, a direct registered sibling route can seed a parent-local Q
from a parent-cone helper instance, and a later direct registered sibling
route can reuse that helper-sourced Q as the next parent-flop D source.
The focused regression is
`cargo test recursive_hierarchy_registered_sibling_routes_can_chain_helper_instances_below_top`.
It requires the recursive tree shape, more helper instances below top
than at top, more multi-stage registered sibling bindings below top than
at top, more multi-stage helper-sourced registered sibling bindings below
top than at top, local parent flops below top, and zero registered
parent-composed counters so the route stays distinct from the
parent-composed helper-chain axis.

The full downstream-clean evidence anchor for that slice was
`/tmp/anvil-tool-matrix-phase4-hierarchy-r37/tool_matrix_report.json`:
`78` scenarios, `4` designs/scenario, `312` total designs,
`coverage_gaps = []`,
`saw_recursive_hierarchy_direct_sibling_parent_cone_instance_routing = true`,
`saw_recursive_hierarchy_direct_registered_sibling_parent_cone_instance_routing = true`,
`saw_recursive_hierarchy_registered_multistage_parent_cone_instance_routing = true`,
`saw_recursive_hierarchy_registered_parent_composed_parent_cone_instance_routing = true`,
and
`saw_recursive_hierarchy_parent_composed_parent_cone_instance_flop_routing = true`
with `312/0` pass-fail in Verilator plus both repo-owned Yosys modes.
The previous recursive registered parent-composed helper full bank is
`r36`.

### Phase 4 r36 banks recursive non-top registered parent-composed helper routing downstream-clean
The live Phase 4 hierarchy policy now includes the registered
parent-composed version of the recursive exact-depth-2 helper axis:
non-top registered parent-composed child-input D cones can source from
parent-cone helper instances below the top parent. The focused
regression is
`cargo test recursive_hierarchy_registered_child_input_cones_can_use_helper_instances_below_top`.
It requires the recursive tree shape, more helper instances below top
than at top, more registered parent-composed bindings below top than at
top, more registered helper bindings below top than at top, and local
parent flops below top.

The coverage-only policy anchor is
`/tmp/anvil-tool-matrix-phase4-recursive-registered-parent-helper-r36/tool_matrix_report.json`:
`75` scenarios, `4` designs/scenario, `300` total designs,
`coverage_gaps = []`, and
`saw_recursive_hierarchy_registered_parent_composed_parent_cone_instance_routing = true`.

The full downstream-clean evidence anchor for that slice was
`/tmp/anvil-tool-matrix-phase4-hierarchy-r36/tool_matrix_report.json`:
`75` scenarios, `4` designs/scenario, `300` total designs,
`coverage_gaps = []`,
`saw_recursive_hierarchy_direct_sibling_parent_cone_instance_routing = true`,
`saw_recursive_hierarchy_direct_registered_sibling_parent_cone_instance_routing = true`,
`saw_recursive_hierarchy_registered_parent_composed_parent_cone_instance_routing = true`,
and
`saw_recursive_hierarchy_parent_composed_parent_cone_instance_flop_routing = true`
with `300/0` pass-fail in Verilator plus both repo-owned Yosys modes.
The previous recursive direct registered-helper full bank is `r35`.

### Phase 4 r35 banks recursive non-top direct registered helper routing downstream-clean
The live Phase 4 hierarchy policy now includes the registered sibling
version of the recursive exact-depth-2 helper axis: non-top direct
registered sibling-routed child-input D paths can source from parent-cone
helper instances below the top parent. The focused regression is
`cargo test recursive_hierarchy_registered_sibling_routes_can_use_helper_instances_below_top`.
It requires the recursive tree shape, more helper instances below top
than at top, more registered sibling helper bindings below top than at
top, local parent flops below top, and zero registered parent-composed
D-cone counters so the route stays distinct from registered
parent-composed helper routing.

The coverage-only policy anchor is
`/tmp/anvil-tool-matrix-phase4-recursive-direct-registered-helper-r35/tool_matrix_report.json`:
`72` scenarios, `4` designs/scenario, `288` total designs,
`coverage_gaps = []`, and
`saw_recursive_hierarchy_direct_registered_sibling_parent_cone_instance_routing = true`.

The current full downstream-clean evidence anchor is
`/tmp/anvil-tool-matrix-phase4-hierarchy-r35/tool_matrix_report.json`:
`72` scenarios, `4` designs/scenario, `288` total designs,
`coverage_gaps = []`,
`saw_recursive_hierarchy_direct_sibling_parent_cone_instance_routing = true`,
`saw_recursive_hierarchy_direct_registered_sibling_parent_cone_instance_routing = true`,
and
`saw_recursive_hierarchy_parent_composed_parent_cone_instance_flop_routing = true`
with `288/0` pass-fail in Verilator plus both repo-owned Yosys modes.
The previous recursive direct-helper full bank is `r34`.

### Phase 4 r34 banks recursive non-top direct helper routing downstream-clean
The live Phase 4 hierarchy policy now includes a recursive exact-depth-2
axis that proves direct sibling-routed child inputs below the top parent
can source from parent-cone helper instances. The focused regression is
`cargo test recursive_hierarchy_sibling_routes_can_use_helper_instances_below_top`.
It requires the recursive tree shape, more helper instances and helper
bindings below top than at top, and zero registered helper counters so the
route stays distinct from registered sibling/helper D routing.

The first coverage-only policy anchor was
`/tmp/anvil-tool-matrix-phase4-recursive-direct-helper-r32/tool_matrix_report.json`;
the first full downstream attempt at
`/tmp/anvil-tool-matrix-phase4-hierarchy-r32/tool_matrix_report.json`
correctly failed because Yosys found one warning in both modes. The
repro was `int_nodeid_egraph_phase4_recur_profile_d2_top4_mid2_seq`,
`design_0002`, top `mod_50_0019`: a procedural `case` with an exact
selector chose an arm whose bounds made a later shift provably constant.
That exposed a real cleanup gap rather than a hierarchy bug: the cheap
bounds revisit handled shifts and ternary muxes, but not exact-selector
`CaseMux` / `CasezMux` arms. `src/gen/cone.rs` now teaches
`node_unsigned_bounds` and `exact_gate_value` to follow those procedural
mux arms conservatively, with regressions for the CaseMux overshift shape
and exact matching Casez patterns.

The current full downstream-clean evidence anchor is
`/tmp/anvil-tool-matrix-phase4-hierarchy-r34/tool_matrix_report.json`:
`69` scenarios, `4` designs/scenario, `276` total designs,
`coverage_gaps = []`,
`saw_recursive_hierarchy_direct_sibling_parent_cone_instance_routing = true`,
and
`saw_recursive_hierarchy_parent_composed_parent_cone_instance_flop_routing = true`
with `276/0` pass-fail in Verilator plus both repo-owned Yosys modes.
The previous recursive helper-state full bank is `r31` for the
66-scenario policy. The first clean direct-helper full bank was `r33`;
`r34` refreshes it after the post-remap idempotent duplicate cleanup.

### Phase 4 r31 banks recursive non-top helper state downstream-clean
The live Phase 4 hierarchy policy now includes a recursive exact-depth-2
axis that proves stateful parent-composed helper child-input routing
below the top parent. The focused regression is
`cargo test recursive_hierarchy_parent_composed_helper_routes_can_use_parent_flops_below_top`.
It is deliberately stronger than the depth-1 stateful helper proof:
it requires `hierarchy_parent_cone_instances > top_parent_cone_instances`,
`hierarchy_parent_local_flops > top_local_flops`, and
`child_input_bindings_from_parent_cone_instances_through_parent_flops >
top_child_input_bindings_from_parent_cone_instances_through_parent_flops`.

The first coverage-only policy anchor was
`/tmp/anvil-tool-matrix-phase4-recursive-helper-state-r31/tool_matrix_report.json`;
the current full downstream-clean evidence anchor is
`/tmp/anvil-tool-matrix-phase4-hierarchy-r31/tool_matrix_report.json`:
`66` scenarios, `4` designs/scenario, `264` total designs,
`coverage_gaps = []`, and
`saw_recursive_hierarchy_parent_composed_parent_cone_instance_flop_routing = true`
with `264/0` pass-fail in Verilator plus both repo-owned Yosys modes.
The previous full downstream-clean bank is `r30` for the 63-scenario
stateful parent-composed helper policy.

### Phase 4 r30 superseded the r29 registered parent-composed helper bank
Stateful parent-composed helper child-input routing now has its own
proof, distinct from registered child-input helper D cones. In the new
shape, a parent-cone helper output seeds a parent-local Q, and
unregistered parent-composed child-input logic consumes that helper Q
before binding the later child input. The focused proof should assert
`child_input_bindings_from_parent_cone_instances_through_parent_flops > 0`
and
`parent_cone_instance_flop_child_input_binding_fraction > 0.0` while
keeping `child_input_bindings_from_registered_parent_cone_instances = 0`.

The current evidence anchor is
`/tmp/anvil-tool-matrix-phase4-hierarchy-r30/tool_matrix_report.json`:
`63` scenarios, `4` designs/scenario, `252` total designs,
`coverage_gaps = []`, and `252/0` pass-fail in Verilator plus both
repo-owned Yosys modes. It fully banks the direct sibling helper,
direct registered sibling helper, multi-stage registered sibling,
stateful parent-output helper, multi-stage direct registered sibling
helper, multi-stage registered parent-composed helper, and stateful
parent-composed helper child-input routes. Keep `r23` as the
pre-direct-helper full-bank breadcrumb, `r24` as the coverage-only
direct-helper proof, `r25` as the direct-helper full bank, `r26` as the
previous multi-stage sibling full bank, `r27` as the previous stateful
parent-output helper bank, `r28` as the previous multi-stage direct
registered sibling helper bank, and `r29` as the previous
multi-stage registered parent-composed helper bank.

### Phase 4 r29 supersedes the r28 direct registered helper-chain bank
Registered parent-composed helper routing now has its own multi-stage
proof, distinct from the direct registered sibling helper proof. In the
new shape, a parent-cone helper output seeds an earlier parent-local Q,
and later registered parent-composed D logic reuses that Q before
driving a later child input. The focused proof should keep
`child_input_bindings_from_registered_multistage_parent_cone_instances = 0`
while asserting
`child_input_bindings_from_registered_multistage_parent_composed_parent_cone_instances > 0`,
so the direct sibling helper chain and parent-composed helper chain
stay observably separate.

The current evidence anchor is
`/tmp/anvil-tool-matrix-phase4-hierarchy-r29/tool_matrix_report.json`:
`60` scenarios, `4` designs/scenario, `240` total designs,
`coverage_gaps = []`, and `240/0` pass-fail in Verilator plus both
repo-owned Yosys modes. It fully banks the direct sibling helper,
direct registered sibling helper, multi-stage registered sibling,
stateful parent-output helper, multi-stage direct registered sibling
helper, and multi-stage registered parent-composed helper routes. Keep
`r23` as the pre-direct-helper full-bank breadcrumb, `r24` as the
coverage-only direct-helper proof, `r25` as the direct-helper full bank,
`r26` as the previous multi-stage sibling full bank, `r27` as the
previous stateful parent-output helper bank, and `r28` as the previous
multi-stage direct registered sibling helper bank.

### Phase 4 r28 superseded the r27 stateful parent-output helper bank
Direct registered sibling helper routing now has two distinct
child-input-proven forms: a helper output can feed the immediate
parent-local D path, and a helper output can first seed a parent-local Q
that a later registered sibling route reuses as the next flop's D
source. The second form is not registered parent-composed logic: the
focused proof should keep both
`child_input_bindings_from_registered_parent_composed_logic = 0` and
`child_input_bindings_from_registered_multistage_parent_composed_logic = 0`
while asserting
`child_input_bindings_from_registered_multistage_parent_cone_instances > 0`.

The evidence anchor was
`/tmp/anvil-tool-matrix-phase4-hierarchy-r28/tool_matrix_report.json`:
`57` scenarios, `4` designs/scenario, `228` total designs,
`coverage_gaps = []`, and `228/0` pass-fail in Verilator plus both
repo-owned Yosys modes. It fully banks the direct sibling helper,
direct registered sibling helper, multi-stage registered sibling,
stateful parent-output helper, and multi-stage direct registered
sibling helper routes. Keep `r23` as the pre-direct-helper full-bank
breadcrumb, `r24` as the coverage-only direct-helper proof, `r25` as
the direct-helper full bank, `r26` as the previous multi-stage sibling
full bank, and `r27` as the previous stateful parent-output helper
bank.

### Phase 4 r27 superseded the r26 multi-stage sibling bank
Parent-output helper routing now has two distinct output-proven forms:
direct helper-to-parent-output composition, and helper-to-parent-output
composition through parent-local state. The second form is not the same
as registered child-input routing: no child input needs to bind from a
helper output, and the proof should keep
`child_input_bindings_from_parent_cone_instances = 0` while asserting
that parent outputs reach helper instances through flop Qs.

That evidence anchor was
`/tmp/anvil-tool-matrix-phase4-hierarchy-r27/tool_matrix_report.json`:
`54` scenarios, `4` designs/scenario, `216` total designs,
`coverage_gaps = []`, and `216/0` pass-fail in Verilator plus both
repo-owned Yosys modes. It fully banks the direct sibling helper, direct
registered sibling helper, multi-stage registered sibling, and stateful
parent-output helper routes. Keep `r23` as the pre-direct-helper
full-bank breadcrumb, `r24` as the coverage-only direct-helper proof,
`r25` as the direct-helper full bank, and `r26` as the previous
multi-stage sibling full bank.

### Parent-output helper-through-flop metrics should stay dependency-based
The tempting implementation of the stateful parent-output helper metric
is to recursively walk every output cone looking for `FlopQ` nodes, then
recursively walk every flop's D cone looking for parent-cone helper
instance outputs. That is correct on small examples but too expensive
on parent-state-heavy scenarios that have many local flops and no
parent-cone helpers, and it re-walks the same D cones repeatedly.

The metric now first checks whether the module even has
`InstanceRole::ParentCone` instances. When it does, it uses the output
root's `DepSet` to find flop virtuals and the existing
`collect_instance_output_support` memo to ask whether each flop D side
reaches a parent-cone helper output. This preserves the structural fact
while keeping the Phase 4 matrix from turning a coverage metric into a
generation-time hotspot.

### Phase 4 r26 supersedes the r25 direct-helper bank
Direct sibling helper routes originally landed after the `r23`
full downstream-clean Phase 4 hierarchy bank, so `r24` was deliberately
coverage-only evidence for the expanded 48-scenario policy. `r25`
banked that direct-helper policy through the downstream tools. `r26`
adds the multi-stage registered sibling route and is now the current
historical 51-scenario evidence anchor.

That evidence anchor is
`/tmp/anvil-tool-matrix-phase4-hierarchy-r26/tool_matrix_report.json`:
`51` scenarios, `4` designs/scenario, `204` total designs,
`coverage_gaps = []`, and `204/0` pass-fail in Verilator plus both
repo-owned Yosys modes. It fully banks the direct sibling helper and
direct registered sibling helper routes, plus the registered sibling
route that chains through earlier parent-local Qs. Keep `r23` only as
the pre-direct-helper full-bank breadcrumb, `r24` only as the
coverage-only direct-helper proof, and `r25` only as the previous
direct-helper full bank.

### Direct sibling helper routes must stay unregistered in the metrics
Direct sibling routing can now request a parent-cone helper instance
source when `hierarchy_parent_cone_instance_prob` fires. That route
still binds the later child input directly from a dep-bearing parent
source; it does not allocate a parent-local flop and it must not be
counted as registered routing just because the source is a helper
instance output.

The metric contract is therefore split deliberately:
`child_input_bindings_from_parent_cone_instances` and the matching
plain helper fractions prove that a helper output reached a child input,
while `child_input_bindings_from_registered_parent_cone_instances`
stays zero unless the final binding goes through a registered route.
The focused direct sibling helper regression forces the registered
sibling and registered parent-composed axes off to preserve that
separation.

### Registered helper metrics must include direct registered sibling routes
The registered helper-instance metric originally grew out of the
registered parent-composed child-input route, where the child binding is
proved by a parent-composed D cone followed by one local parent flop.
That shape is still important, but it is not the only registered route
that can legitimately use a parent-cone helper instance.

Direct registered sibling routing can now request a helper instance
source when `hierarchy_parent_cone_instance_prob` fires. In that case
the route remains a registered child-input binding, but the flop D side
may be the helper `InstanceOutput` itself or a width-adapter gate over
that output, not a registered parent-composed D-cone root.

Therefore the metric contract is dependency-based: inspect the final
registered flop D dependencies and ask whether they include a
parent-cone helper instance output. Requiring the D node to also look
like registered parent-composed logic would undercount the direct
registered sibling helper route and would make the test prove the wrong
shape.

### Parent-output helper budgeting must be output-proven, not helper-to-helper-proven
The parent-output helper route originally proved only one fact: a parent
output could depend on a parent-cone helper instance output. The
separate helper-budget route proved a different fact through
child-input bindings: a parent could allocate multiple helper children
when `max_parent_cone_instances_per_module` was raised. Those two facts
did not prove that parent-output composition itself could spend the
budget.

The current implementation therefore collects parent-output helper
sources before parent-output root construction and lets promotion select
a required helper source per output. That makes the budget visible at
the output-composition seam instead of relying on later child-input
routes.

One gotcha is important: helper instances created specifically for
parent-output composition should not bind their own child inputs from
earlier helper outputs. If that helper-to-helper chaining is allowed,
`child_input_bindings_from_parent_cone_instances` becomes non-zero and
the test no longer proves an output-only path. The parent-output helper
collector therefore uses non-helper parent sources for helper child
inputs while still publishing the helper outputs to the real parent
source pool for output composition.

### Phase 4 starts as wrapper hierarchy on purpose
The first hierarchy slice is deliberately **not** "instances can appear
anywhere in any parent cone". That broader story is the destination,
but it is not the cheapest truthful first landing.

What landed instead is:

- generate a library of leaf modules with the already-proven leaf
  kernel,
- choose the wrapper's instantiated-child count separately from the
  library size,
- keep `num_child_instances = 0` as the legacy compatibility mode
  meaning "instantiate every generated leaf definition exactly once",
- if fewer instances than library entries are requested, instantiate a
  shuffled subset without replacement,
- if more instances than library entries are requested, cover every
  library entry once and then fill the remaining slots by reuse with
  replacement,
- build a real top wrapper module,
- treat child instance outputs as real parent-side leaf variables for
  top-output construction, and
- make emission / validation / manifest handling design-aware.

That buys several real things immediately:

- ANVIL now emits genuine multi-module SV, not just disconnected leaf
  files;
- downstream tools now see elaboration and inter-module port binding;
- the IR and validator now carry explicit instance structure; and
- the hierarchy layer stays above `generate_leaf_module` instead of
  smearing inter-module behavior into the leaf kernel.

Just as importantly, it keeps the open work honest. The first-landed
top layer was **combinational only**; since then, bounded recursive
sub-hierarchy growth, local parent flops, child-input routing, mixed
parent-port / child-output parent outputs, parent-cone helper instances
for parent-composed child-input cones, direct sibling child-input
routes, direct registered sibling D sources, registered child-input D
cones, parent-output cones, and explicit helper budgeting have landed
as separate slices. Broader helper-instance placement
beyond those seams, broader registered hierarchy-local routing, and
hierarchical identity remain future work.

Two narrow implementation choices are load-bearing in this slice:

- the wrapper top marks shared `clk` / `rst_n` as
  `Module.clock` / `Module.reset`, and control-port visibility is now
  design-aware instead of leaf-local: pure comb-only modules omit those
  ports, while hierarchy parents keep them visible iff they carry local
  state or sequential descendants;
- `Node::InstanceOutput` now carries a real dep-bearing leaf identity,
  so parent cones can use child outputs without being mistaken for
  empty-dep constants by later cleanup/finalisation passes.

That second point shook loose three old wrapper-era assumptions that had
to be fixed at the root instead of papered over:

- `compact_node_ids` was only treating output drives and flop holders as
  liveness roots, so instance input bindings could survive with stale
  `NodeId`s after compaction. The real fix was to mark instance-input
  bindings as holders and remap them just like drives and flops.
- `validate_design` was still enforcing "every child output is exposed
  exactly once", which was only true for the pass-through wrapper era.
  The right rule is narrower: any *referenced* child output node must
  name a real child port at the right width, but unreferenced child
  outputs are legal.

### Depth ranges must stay ranges in recursive hierarchy
The original bounded recursive planner was honest enough as a first
landing, but it was still throwing away too much information: it took
`min_hierarchy_depth..=max_hierarchy_depth` and collapsed that interval
to one exact realized depth for the whole design.

That was acceptable as a foothold. It is not the right long-term
algorithm, because the point of a bounded depth interval is to describe
allowed leaf-depth variation, not to sample one global scalar and ignore
the rest.

The strengthened planner now carries a remaining `[min,max]` depth
interval per subtree:

- `max == 0` still means a mandatory leaf;
- if a flexible subtree has only one child, it may sample one exact
  depth inside the still-allowed interval for that chain; and
- when a subtree is both depth-flexible and branching (`instances >= 2`)
  it now deliberately generates child definitions that realize both the
  shallowest and deepest still-legal descendants, instead of hoping RNG
  stumbles into both.

That last point is the load-bearing part. Mixed-depth recursion should
not be a rare accident of repeated runs; when the structure can support
it, the planner should intentionally exercise it.

The metrics contract grew with the planner: `DesignMetrics` now expose
`leaf_module_occurrences_by_depth`, so "did we really get both shallow
and deep leaves?" is answerable numerically from the manifest rather
than by reading emitted SV.

The focused artifact at `/tmp/anvil-hier-mixed-depth-smoke-r1/manifest.json`
was the first clean proof of that new mixed-depth recursive axis. The
current repo-owned Phase 4 gate at
`/tmp/anvil-tool-matrix-phase4-hierarchy-r27/tool_matrix_report.json`
also proves it, so the mixed-depth story is no longer "focused-only"
evidence.
- the emitter was still assuming every child output had a corresponding
  `Node::InstanceOutput`. That is no longer true once the parent may use
  only a subset of child outputs, so unused outputs are now rendered as
  explicit unconnected instance ports (`.port()`).

The separation between **library size** and **instance count** matters
enough to say plainly here: they are different planning decisions and
should not be welded together. Repeated instantiation of the same child
definition stresses elaboration and sharing pressure in a different way
from simply generating more unique children. Under-instantiating the
library is also useful because it exercises real unused-module cleanup
in downstream tools. The wrapper slice still is not the final hierarchy
algorithm, but this split is a real step toward a budget-driven one.

Module names are also a hierarchy resource now, not an incidental string
format at each construction site. Leaf modules, recursive parent
modules, and later designs in the same generator run all reserve names
from the same `Generator` sequence. That keeps `--count N --out DIR`
safe for hierarchy output: one module definition still maps to one
`.sv` file, and no later design can overwrite an earlier definition by
reusing the same `mod_<seed>_<index>` name.

### Bounded recursive hierarchy keeps the old wrapper lane, then adds a real tree planner
The next honest Phase 4 step was **not** to quietly overload the old
depth-1 wrapper knobs until they accidentally meant recursion. That
would have blurred the already-banked wrapper evidence and made the
meaning of the config surface harder to recover later.

So the deliberate rule now is:

- keep the legacy exact wrapper lane alive:
  `hierarchy_depth = 1`, `num_leaf_modules`, `num_child_instances`;
- add a separate bounded recursive lane:
  `min_hierarchy_depth..=max_hierarchy_depth` and
  `min_child_instances_per_module..=max_child_instances_per_module`; and
- make the two planning surfaces mutually exclusive.

That gives us a clean story:

- old repo-owned wrapper closure artifacts stay truthful;
- new recursive hierarchy is explicit in configs and manifests; and
- future recursive work can evolve without pretending the exact wrapper
  lane already solved arbitrary tree planning.

The current recursive planner is now intentionally exact about the
interval contract rather than about one sampled scalar:

- every realized leaf depth stays inside the requested `[min:max]`
  interval;
- when a subtree is both depth-flexible and branching, the planner
  deliberately exercises both the shallowest and deepest still-legal
  descendants instead of relying on luck; and
- each non-leaf module still picks its child-instance count uniformly
  inside the requested child-instance interval.

So the current guarantees are:

- realized leaf depths stay inside the requested interval;
- mixed shallow/deep trees are now intentional when the structure can
  support them;
- realized branching always stays inside the requested interval; and
- the metrics can prove all of that numerically.

One more planning layer is now live on top of that baseline:

- `min_child_instances_per_module..=max_child_instances_per_module`
  remains the global fallback range for recursive branching; and
- repeated `child_instances_per_depth` overrides can tighten or replace
  that range at specific parent depths (`0` = top, `1` = its direct
  children, ...).

That keeps the control surface honest: users can ask for "top is wide,
lower levels are narrower" without inventing a separate planner mode or
forcing the manifest reader to reverse-engineer the realized tree by
hand.

What it does **not** do yet is make every parent-side cone free to
instantiate arbitrary helper modules. The narrow helper-instance seams
are now live for parent-composed child-input cones, direct sibling
routes, direct registered sibling D sources, registered child-input D
cones, parent-output cones, multi-stage direct registered sibling
helper chains, and multi-stage registered parent-composed helper
chains, with explicit per-parent budgeting. Broader helper placement
beyond those seams remains future hierarchy work.

One more gate-level rule turned out to matter here: when a repo-owned
matrix grows new representative scenarios, its per-scenario evidence
budget must not shrink by accident. The Phase 4 gate moved from 15 to
18 scenarios once the mixed-depth recursive axis was added, so its
minimum total design budget was raised from 48 to 60 to preserve the
old 4 designs/scenario sampling depth instead of silently falling to 3.

That lesson is now encoded directly for Phase 4. After the
parent-output helper, budgeted-helper, and registered helper-sourced
child-input axes raised the scenario set to 42, the old
`PHASE4_HIERARCHY_MIN_TOTAL_DESIGNS = 120` rule silently produced only
3 designs/scenario (`126` total) in the clean pre-fix `r22` run. The
live gate now uses a per-scenario floor
`PHASE4_HIERARCHY_MIN_DESIGNS_PER_SCENARIO = 4`. After the direct
sibling helper and direct registered sibling helper axes raised the
scenario set to `48`, the live regression expected `192` total designs.
After the multi-stage registered sibling route raised the scenario set
to `51`, the live regression expects `204` total designs. The
stateful parent-output helper route raised the scenario set to `54`,
and the live regression expected `216` total designs. The multi-stage
direct registered sibling helper route raised it to `57` scenarios /
`228` total designs in `r28`, and the multi-stage registered
parent-composed helper route raises it to `60` scenarios / `240` total
designs in `r29`.

One more planner rule is load-bearing here: in recursive range mode,
child libraries are generated **on demand per parent**, and every
generated direct child definition is instantiated at least once. That
keeps reuse live without manufacturing dead unreachable subtrees just to
inflate counts. The legacy exact wrapper lane remains the place where
top-level under-instantiation of a pre-generated library is exercised.

### Explicit child sourcing must be a real axis, not a vague future promise
Once the wrapper/reuse/under-instantiation story and the recursive
mixed-depth story were both banked, the next honest Phase 4 question
was no longer "can hierarchy exist?" but "how do parents obtain child
definitions?"

That decision is too load-bearing to hide behind ad hoc planner
behavior. It needs to be a user-visible, measurable axis:

- `library` means pre-generate a reusable child-definition pool and let
  instance slots pick from it;
- `on-demand` means synthesize child definitions against exact
  parent-planned data-interface profiles per planned instance slot.

The current landed `on-demand` slice is now the stronger honest one:
each planned child slot carries an exact parent-planned data-interface
profile, and the realized child definition is validated against that
exact emitted data-input/output shape. Control ports stay structural,
which is also the right rule: `clk` / `rst_n` are propagated by
sequential-state presence, not by the data-profile planner.

That is also why the metrics contract grew again. The hierarchy reports
now need to distinguish:

- reused child definitions,
- single-use instantiated definitions, and
- the average instance count per unique instantiated module,
- exact profiled instance-slot coverage, and
- whether child data-input bindings stay dep-bearing instead of
  collapsing to constants.

Without those numbers, `library` vs `on-demand` would still force a
human to open the emitted `.sv`, which is exactly the trust failure we
want to avoid.

The repo-owned Phase 4 gate has now caught up here too. The current
artifact is `/tmp/anvil-tool-matrix-phase4-hierarchy-r27/tool_matrix_report.json`,
and it explicitly proves both child-sourcing modes (`library` and
`on-demand`) together with structural proof that the on-demand
scenarios really emitted fresh child definitions per planned instance
slot and exact profiled child-interface synthesis.

### Combinational sibling routing was the right next layer before local parent state
Once parent-composed outputs and exact profiled child sourcing were
real, the next honest hierarchy question for that slice was not
"should parents have local flops yet?" It was "can one child feed
another through the parent without us faking it as a top-level wrapper
input?"

The current answer is now yes, but intentionally only on the simpler
surface:

- later child data inputs may bind from earlier sibling instance
  outputs;
- the routing stays acyclic by construction because only already-built
  sibling outputs are eligible;
- the routing stayed purely combinational in that slice; and
- local parent flops deliberately remained future work instead of being
  smuggled into the same step.

That last point matters. Child-output -> child-input through local flop
layers is a valid future hierarchy surface, but it is a different
question from the one we needed to close here. This slice was about
making the parent behave more like the leaf generator's cone builder,
except with child-module outputs as additional dep-bearing leaves, while
keeping the phase boundary honest.

The metrics contract had to grow again for the same reason. A sibling
routing feature that can only be confirmed by opening `.sv` is not a
trustworthy feature. The design reports now distinguish:

- child inputs bound from parent ports,
- child inputs bound from sibling instance outputs,
- mixed-support child inputs, and
- the hierarchy-wide and top-level fractions of child inputs that come
  from sibling instance outputs.

The focused proof artifact is now
`/tmp/anvil-hier-sibling-routing-smoke-r1/manifest.json`, and the
repo-owned Phase 4 gate at
`/tmp/anvil-tool-matrix-phase4-hierarchy-r27/tool_matrix_report.json`
now requires `saw_hierarchy_sibling_routing = true`.

### Parent-composed child-input bindings are the cone-builder analogue of sibling routing
The next hierarchy routing step keeps the same phase boundary but
removes one more artificial flat-wrapper shape. Direct sibling routing
answers "can a later child consume an earlier child output?" Parent-
composed child-input binding answers "can the parent build a small
combinational cone for a child input, using the same generator machinery
as leaf cones?"

For that slice, the rule was deliberately narrow and structural:

- `hierarchy_child_input_cone_prob` controls the probability of this
  route;
- the cone's source pool contains only already-available parent sources:
  parent data inputs, earlier sibling instance outputs, and earlier
  parent-side route gates;
- local parent flops stayed disabled, so the parent-composed
  child-input route was a purely combinational composition surface; and
- the rule applies to both the legacy wrapper lane and the bounded
  recursive lane.

This is the shape the user suggested: at the composition level, replace
"gate" by "child module" where it makes sense, but keep that first
slice combinational. Local parent flops are now landed under
`hierarchy_parent_flop_prob`; the first one-flop registered sibling
route is now landed under `hierarchy_registered_sibling_route_prob`,
and the first registered parent-composed child-input route is now
landed under `hierarchy_registered_child_input_cone_prob`. The first
multi-stage registered parent-composed subcase is also live now, while
broader registered hierarchy routing remains a later, separate
hierarchy surface.

The metrics contract grew again with
`child_input_bindings_from_parent_composed_logic`,
`parent_composed_child_input_binding_fraction`, and
`top_parent_composed_child_input_binding_fraction`. The repo-owned
Phase 4 gate treats this as a required coverage fact via
`saw_hierarchy_parent_composed_child_inputs`; the current banked gate at
`/tmp/anvil-tool-matrix-phase4-hierarchy-r27/tool_matrix_report.json`
proves it together with local parent state, `coverage_gaps = []`, and
216/0 clean pass-fail in Verilator plus both repo-owned Yosys modes.
The focused targeted proof is
`/tmp/anvil-hier-child-input-cone-smoke-r1/manifest.json`.

### Parent-cone helper instances make module instantiation a parent source choice
The first helper-instantiation slice is intentionally small: when
`hierarchy_parent_cone_instance_prob` fires during a
parent-composed child-input route, the parent may instantiate one helper
child as an internal parent-cone source. That helper is not one of the
planned child slots. It is tagged with `InstanceRole::ParentCone`, bound
from the parent source pool, and its outputs can feed later child inputs
through ordinary parent combinational logic.

That gives the hierarchy planner a first real "module instance as cone
source" behavior without making every parent-side cone recursive all at
once. The metrics contract is explicit:
`top_parent_cone_instances`, `hierarchy_parent_cone_instances`,
`child_input_bindings_from_parent_cone_instances`,
`top_child_input_bindings_from_parent_cone_instances`,
`parent_cone_instance_child_input_binding_fraction`, and
`top_parent_cone_instance_child_input_binding_fraction`.

The focused proof is
`/tmp/anvil-parent-cone-instance-smoke-r1/manifest.json`
(`top_parent_cone_instances = 1`, `hierarchy_parent_cone_instances = 1`,
`child_input_bindings_from_parent_cone_instances = 4`, and
`top_child_input_bindings_from_parent_cone_instances = 4`), clean in
Verilator, Yosys `synth -noabc`, and the repo-owned Yosys with-ABC
path. The repo-owned Phase 4 gate now banks this as a required coverage
fact at `/tmp/anvil-tool-matrix-phase4-hierarchy-r27/tool_matrix_report.json`
with `coverage_gaps = []` and 216/0 pass-fail in Verilator plus both
repo-owned Yosys modes.

Current HEAD has broadened that helper source beyond the original
parent-composed child-input seam. Helper outputs can now feed direct
unregistered sibling routes, direct registered sibling-route D inputs,
registered parent-composed child-input D cones, and parent-output
composition. The direct sibling helper proof is
`cargo test hierarchy_sibling_routes_can_use_helper_instances`; it
requires registered helper counters to stay zero while
`child_input_bindings_from_parent_cone_instances > 0`,
`parent_cone_instance_child_input_binding_fraction > 0.0`,
`top_parent_cone_instance_child_input_binding_fraction > 0.0`, and
helper instances are present beyond the planned child slots.

### Local parent flops are a separate hierarchy state axis
The next Phase 4 step deliberately does not overload leaf `flop_prob`.
Hierarchy parent state is controlled by its own knob,
`hierarchy_parent_flop_prob`, because the parent layer is a different
structural axis from leaf-module sequential richness. The default is
`0.0`, which preserves the previously banked combinational hierarchy
surface; setting it non-zero lets parent output cones and
parent-composed child-input cones emit local parent flops.

The important invariant is the same one used for sequential
descendants: `clk` and `rst_n` are structural, not decorative. A parent
module reserves those control ports when local parent state is possible,
but the emitter only exposes them when the module actually carries
local flops or sequential descendants. Pure comb-only modules remain
free of control ports.

The implementation reuses the normal cone/flop worklist machinery.
While building a hierarchy parent cone, the generator temporarily maps
flop rolls to `KnobId::HierarchyParentFlopProb`, then drains the parent
flop worklist before finalization. That keeps telemetry honest: leaf
flop attempts still count as `flop_prob`, while parent-state attempts
count as `hierarchy_parent_flop_prob`.

Metrics now expose this state surface directly:
`hierarchy_parent_local_flops`,
`internal_module_occurrences_with_local_flops`, `top_local_flops`,
`child_input_bindings_from_parent_flops`,
`parent_flop_child_input_binding_fraction`, and
`top_parent_flop_child_input_binding_fraction`.
The focused proof is
`/tmp/anvil-hier-parent-state-smoke-r1/manifest.json`
(`hierarchy_parent_local_flops = 8`, `top_local_flops = 8`,
`top_clock_inputs = 1`, `top_reset_inputs = 1`,
`child_input_bindings_from_parent_flops = 1`), clean in Verilator,
Yosys `synth -noabc`, and the repo-owned Yosys with-ABC path. The
repo-owned Phase 4 gate now also banks this as a required coverage fact
at `/tmp/anvil-tool-matrix-phase4-hierarchy-r27/tool_matrix_report.json`
with `coverage_gaps = []` and 216/0 pass-fail in Verilator plus both
repo-owned Yosys modes.

### Registered sibling routing is a distinct hierarchy route axis
Direct sibling routing and registered sibling routing are deliberately
separate knobs. `hierarchy_sibling_route_prob` keeps the acyclic
combinational route live: earlier child output directly feeds a later
child input. `hierarchy_registered_sibling_route_prob` adds a
parent-local flop between those endpoints. A later registered sibling
route can now also choose an earlier parent-local Q as its D source,
creating a multi-stage registered sibling chain without parent-composed
logic. That route is still
acyclic at the module-instance level, but it introduces real state in
the parent, so it must be measured as both child-input provenance and
parent-local state.

The initial implementation intentionally used one flop and no extra mux
or cone around it. That is not "good enough"; it is the smallest
signoff-clean primitive for this axis. Richer registered
child-to-child patterns build from the same invariant:
earlier child output -> parent state -> later child input, with metrics
proving the route instead of requiring SV inspection. The multi-stage
direct sibling subcase is now reported separately through
`child_input_bindings_from_registered_multistage_instance_outputs`,
`top_child_input_bindings_from_registered_multistage_instance_outputs`,
`registered_multistage_instance_output_child_input_binding_fraction`,
and
`top_registered_multistage_instance_output_child_input_binding_fraction`.

This slice exposed a real finalization gotcha: post-construction remap
passes already rewrote output drives and flop fields, but instance
input bindings were also live NodeId consumers. Once a child input
could bind to a parent-local Q node, flop merging could leave an
instance input pointing at a stale duplicate FlopQ. The fix belongs in
`ir::compact`: every partial NodeId remap now rewrites instance input
bindings too. The focused unit test covers that root cause, not only
the hierarchy symptom.

### Registered parent-composed routing is not the same as registered sibling routing
The registered sibling route proves a minimal stateful handoff:
earlier child output -> parent flop -> later child input. The next
route axis deliberately adds parent logic before that flop:
earlier child output or earlier parent route gate -> parent-local
combinational logic -> parent flop -> later child input.

This is controlled by
`hierarchy_registered_child_input_cone_prob`, not by overloading
`hierarchy_registered_sibling_route_prob` or
`hierarchy_parent_flop_prob`. The distinction matters because the
metric has to prove a different structure: the binding must pass
through a parent-local flop whose D input is itself a parent-local gate
with instance-output support.

The implementation now builds the D cone from the full available parent
source pool, with spontaneous nested flop generation disabled for this
route. It then repairs the root before allocating the final flop: the D
path must keep sibling-output support, when parent data inputs are live
it can add parent-port support, and when earlier parent flops are live
it can add a prior-Q companion to create a multi-stage registered
chain. If the repaired root is not already a substantive parent gate,
the generator wraps it in a non-collapsing XOR-with-all-ones parent
gate. That keeps the route signoff-clean and construction-time
deterministic while preserving the structural proof obligation.

Metrics now expose the route directly through
`child_input_bindings_from_registered_parent_composed_logic`,
`top_child_input_bindings_from_registered_parent_composed_logic`,
`registered_parent_composed_child_input_binding_fraction`, and
`top_registered_parent_composed_child_input_binding_fraction`. Current
HEAD also exposes the mixed registered-support subcase through
`child_input_bindings_from_registered_mixed_support`,
`top_child_input_bindings_from_registered_mixed_support`,
`registered_mixed_support_child_input_binding_fraction`, and
`top_registered_mixed_support_child_input_binding_fraction`. Current
HEAD also exposes the first multi-stage registered subcase through
`child_input_bindings_from_registered_multistage_parent_composed_logic`,
`top_child_input_bindings_from_registered_multistage_parent_composed_logic`,
`registered_multistage_parent_composed_child_input_binding_fraction`,
and
`top_registered_multistage_parent_composed_child_input_binding_fraction`.
The original focused proof is
`/tmp/anvil-hier-registered-child-input-cone-smoke-r2/manifest.json`
(`child_input_bindings_from_registered_parent_composed_logic = 3`,
`top_child_input_bindings_from_registered_parent_composed_logic = 3`,
`registered_parent_composed_child_input_binding_fraction = 0.75`,
`top_registered_parent_composed_child_input_binding_fraction = 0.75`,
`hierarchy_parent_local_flops = 3`), clean in Verilator, Yosys
`synth -noabc`, and the repo-owned Yosys with-ABC path. The repo-owned
Phase 4 gate now banks this as a required coverage fact at
`/tmp/anvil-tool-matrix-phase4-hierarchy-r27/tool_matrix_report.json`
with `coverage_gaps = []` and 216/0 pass-fail in Verilator plus both
repo-owned Yosys modes.

The focused mixed-support proof is
`/tmp/anvil-hier-registered-mixed-child-input-smoke-r1/manifest.json`
(`child_input_bindings_from_registered_mixed_support = 3`,
`top_child_input_bindings_from_registered_mixed_support = 3`,
`registered_mixed_support_child_input_binding_fraction = 0.75`), clean
in Verilator, Yosys `synth -noabc`, and the repo-owned Yosys with-ABC
path. The current-code coverage-only Phase 4 matrix probe at
`/tmp/anvil-tool-matrix-phase4-registered-mixed-r1/tool_matrix_report.json`
first banked `saw_hierarchy_registered_mixed_support_routing = true`
with `coverage_gaps = []`; the full downstream-clean `r27` bank now
carries the same fact with Verilator and both repo-owned Yosys modes.

The focused multi-stage registered proof is
`/tmp/anvil-hier-registered-multistage-child-input-smoke-r1/manifest.json`
(`child_input_bindings_from_registered_multistage_parent_composed_logic = 2`,
`top_child_input_bindings_from_registered_multistage_parent_composed_logic = 2`,
`registered_multistage_parent_composed_child_input_binding_fraction = 0.5`),
clean in Verilator, Yosys `synth -noabc`, and the repo-owned Yosys
with-ABC path. The current-code coverage-only Phase 4 matrix probe at
`/tmp/anvil-tool-matrix-phase4-registered-multistage-r1/tool_matrix_report.json`
first banked `saw_hierarchy_registered_multistage_routing = true` with
`coverage_gaps = []`; the full downstream-clean `r27` bank now carries
the same fact with Verilator and both repo-owned Yosys modes.

The focused multi-stage registered sibling proof is
`cargo test hierarchy_registered_sibling_routes_can_chain_through_parent_flops`.
It proves the direct registered sibling route can chain through earlier
parent-local Qs while keeping registered parent-composed counters at
zero. The `r27` Phase 4 matrix banks that as
`saw_hierarchy_registered_multistage_sibling_routing = true` through
the dedicated
`phase4_hier2_inst4_registered_sibling_multistage_state` scenario.

### Parent outputs can mix parent ports with child outputs
The first parent-output composition slice built output cones from child
`InstanceOutput` leaves only. That proved real parent-side logic above
children, but it left parent data inputs out of the parent output
surface.

The current parent-output builder now starts from the full parent
source pool. After module finalization, it rebuilds live pools from the
settled parent module and repairs every parent output that lost
structural child-output support. When live parent data inputs exist,
the same repair path also adds parent-port support to outputs that
otherwise only reached child outputs.

The post-final repair point matters. Cleanup can fold or replace drive
roots, so the invariant has to be checked after compaction, input
pruning, and profile enforcement have settled. The repair adds ordinary
parent gates over live nodes; it does not patch emitted SV.

Metrics expose the result as
`top_parent_port_composed_outputs`,
`hierarchy_parent_port_composed_outputs`,
`top_parent_port_composed_output_fraction`, and
`hierarchy_parent_port_composed_output_fraction`. The focused
regression is
`cargo test --test pipeline hierarchy_parent_outputs_can_mix_parent_ports_with_child_outputs`.
The repo-owned Phase 4 coverage gate now tracks this as
`saw_hierarchy_parent_port_composed_outputs`; the current-code
coverage-only matrix probe at
`/tmp/anvil-tool-matrix-phase4-parent-port-coverage-r1/tool_matrix_report.json`
first recorded `coverage_gaps = []` with that fact true. It skipped
Verilator/Yosys; the full downstream-clean Phase 4 `r27` bank now
carries the same fact with real tool validation.

### Hierarchy quality has to be visible in the numbers
The user requirement here is the right one: for hierarchy, ANVIL should
not depend on someone opening the emitted `.sv` and eyeballing whether
the composition looks plausible. The reports and manifests need to
carry enough exact facts that the result can be trusted numerically.

That is why the current hierarchy slice now has a dedicated
`DesignMetrics` layer instead of only per-module metrics and a few
coarse booleans. The current trustworthy design facts are:

- library size vs instantiated child count,
- unique-instantiated-module count and unused-library count,
- reuse / coverage ratios,
- top interface shape, including `top_clock_inputs` and
  `top_reset_inputs`,
- direct-vs-composed outputs and parent-port-composed output counts,
- control fanout to child instances,
- weighted child interface / node / flop load, and
- per-definition instantiation histograms.

The smoke at `/tmp/anvil-hier-metrics-smoke-r1` mattered because it did
more than prove the metrics serializer. It exposed two real root-cause
bugs that would have made those numbers lie:

- wrapper tops were creating shared `clk` / `rst_n` ports without
  tagging them as `Module.clock` / `Module.reset`; and
- control-port emission was using a too-local rule, so wrappers with no
  local flops could hide `clk` / `rst_n` even when those ports were
  still required by sequential descendants.

The durable rule now is exact and inductive:

- pure comb-only modules do not emit `clk` / `rst_n`;
- sequential leaves do emit `clk` / `rst_n`; and
- hierarchy parents keep `clk` / `rst_n` visible iff they carry local
  state or sequential descendants, all the way up the instantiated
  chain.

That rule is now pinned in IR helpers, validation, metrics, and the SV
emitter, plus direct regression tests for both the comb-only and
grandparent-wrapper cases.

The recursive planner widened the metrics contract too. Wrapper-only
facts were no longer enough; the numbers now have to describe the
**tree**. So `DesignMetrics` now also carries:

- `realized_min_leaf_depth`, `realized_max_leaf_depth`,
  `avg_leaf_depth`, `max_module_depth`;
- `module_defs_by_depth`, `module_occurrences_by_depth`,
  `instance_slots_by_parent_depth`;
- `avg_child_instances_by_parent_depth`,
  `min_child_instances_by_parent_depth`,
  `max_child_instances_by_parent_depth`;
- `child_instances_per_internal_module_histogram`,
  `min/avg/max_child_instances_per_internal_module`; and
- hierarchy-wide composition counters in addition to the top-only ones.

That is the current trust surface for recursive hierarchy quality: the
user should not have to inspect the `.sv` to tell whether ANVIL built
the requested tree shape.

### Literal-backed for-fold sources must be materialized before procedural part-selects
The repo-owned Phase 4 hierarchy gate exposed a real emitter defect in
the bounded procedural `for` surface.

The bad shape was not subtle:

- direct literal indexing such as `24'h86899[(i * 12) +: 12]`, and then
- an attempted blanket fix that emitted `(signal)[(i * 12) +: 12]`.

Neither is a robust answer for the downstream tools we care about.
Verilator and Yosys both rejected those forms during the hierarchy
matrix.

The correct fix is narrower and more truthful:

- keep ordinary named packed sources as `src[(i * K) +: K]`;
- but when the fold source is a constant, materialize it through a
  packed procedural temporary inside the surrounding `always_comb`;
- then index that temporary.

That preserves the intended structured surface, keeps the emitted SV
legal, and fixes the root cause instead of weakening the gate or hiding
the fold behind different syntax.

### Constant-backed slices must fold to literals, not literal indexing
The new under-instantiation hierarchy smoke exposed another emitter bug
in a different surface:

- `assign slice_26 = 20'h0[18:1];`

That is just as wrong as the old procedural literal-indexing bug, but
the right fix is even narrower here. When a `Slice` operand is a
constant, there is no need to emit a slice at all. We already know the
answer exactly.

So the deliberate rule now is:

- if the slice source is non-constant, emit the normal `src[hi:lo]` or
  `src[bit]` form;
- if the slice source is a constant, compute the sliced value in the
  emitter and print the narrower constant literal directly.

That keeps the output legal and simple, and it fixes the real cause
instead of wrapping an invalid shape in more syntax.

### The broadened Phase 4 matrix found the next runtime cost shape
After landing `num_child_instances`, the Phase 4 `tool_matrix` planning
was widened from the old "leaf count x comb/seq" wrapper sweep to four
more truthful representative profiles:

- `phase4_hier2_inst2_comb`  — exact library/instance cardinality
- `phase4_hier2_inst4_seq`   — repeated child-definition reuse
- `phase4_hier4_inst2_comb`  — under-instantiated library
- `phase4_hier4_inst4_seq`   — exact cardinality at the heavier end

That is the right coverage model for the current wrapper slice, and the
full refreshed rerun now closes cleanly at
`/tmp/anvil-tool-matrix-phase4-hierarchy-r7/tool_matrix_report.json`.
But the reruns also made the runtime story obvious: the heavy
sequential `hier4_inst4_seq` cases spend real time inside Yosys because
they elaborate/synthesize tiny wrapper tops over very large sequential
child libraries.

So the durable lesson is:

- this is a downstream cost shape, not a malformed-output bug;
- the refreshed exact / reuse / under-instantiation matrix is now
  actually banked cleanly at `r7`; and
- future Phase 4 work should keep watching those heavy sequential
  corners, because they are the place where hierarchy cost surfaces
  first even when the emitted RTL is valid.

### The recursive hierarchy gate must prove hierarchy, not quietly re-run the fattest leaf stress lane
When the Phase 4 gate was widened again to cover the newer recursive
and per-depth-branching surfaces, the first full rerun (`r8`) exposed a
different version of the same problem. The new coverage logic itself
was fine, but the recursive sequential scenarios were still borrowing
the heaviest Phase 1 motif-heavy sequential leaf profile.

That made the hierarchy gate pay for a huge amount of downstream Yosys
work that belonged to leaf stress, not to hierarchy proof. The proof was
therefore answering the right structural question with the wrong leaf
payload.

The right fix was not to drop the recursive scenarios and not to weaken
the coverage facts. The right fix was to decouple concerns:

- keep the recursive depth-2 and per-depth override profiles in the
  repo-owned Phase 4 matrix;
- keep the clean-tool requirement exactly the same; but
- switch the Phase 4 sequential hierarchy scenarios to a
  hierarchy-focused sequential leaf profile sized for hierarchy proof
  rather than Phase-1-scale leaf stress.

That is why the banked `r9` report closes quickly and honestly:

- the gate still proves wrapper exact / reuse / under-instantiation;
- it still proves recursive depth `2`;
- it still proves the per-depth override profile `0=4:4,1=2:2`;
- it still proves parent-side composition above instance outputs; and
- it no longer burns runtime re-proving the fattest leaf-stress shape
  just to answer a hierarchy question.

### Wrapped-add bounds must preserve a shifted single interval when it stays linear
The `e-graph` warning in
`/tmp/anvil-tool-matrix-phase1-real-r20/int_nodeid_e-graph_default/mod_8_0053.sv`
turned out not to be a generic "Yosys got grumpy" case. It exposed a
specific gap in ANVIL's unsigned-bounds reasoning for `GateOp::Add`.

The old logic did this:

- collect operand bounds;
- if the sum might wrap the target width, fall back to full-range.

That is safe, but it was too blunt for the real rhs shape:

- one non-exact interval (`or_22` bounded to `[0xe7, 0xff]`);
- plus exact constants (`0x0c` and `0xc4`).

In that case, the exact constants are not adding uncertainty. They are
just translating one interval around the unsigned ring. If the
translated interval still lands as one linear interval in unsigned
space, we should keep it. For the real failing case, `[0xe7, 0xff] +
0xd0 (mod 256)` becomes `[183, 207]`, which is still linear and is more
than enough to prove a 3-bit shift is always an overshift.

So the deliberate rule now is:

- if an `Add` node has exactly one non-exact interval operand and the
  rest are exact constants, combine the exact constants first;
- translate the one live interval by that exact wrapped addend; and
- keep the translated interval only when it stays linear (`start <= end`
  after modular translation), otherwise fall back to full-range.

That rule is intentionally narrow. It improves downstream cleanliness on
the real `shift >> wrapped_add` warning shape without reopening the
broader exact-set proof surface that earlier slices had to cap for
runtime reasons.

### `tool_matrix` frontier runs now use per-module checkpoints
`tool_matrix` now writes `<stem>.module-report.json` after each fully
processed module and supports `--resume`.

The resume contract is intentionally narrow:

- checkpoint reuse is allowed only when the current tool surface matches
  the checkpoint (`skip_verilator`, `skip_yosys`, `yosys_mode`);
- same-binary fast resume is allowed only when the checkpoint also
  carries a matching runtime fingerprint, a matching saved-`sv` hash,
  and a saved generator checkpoint;
- otherwise the regenerated module must still match the saved `.sv`
  text and module identity; and
- metrics are refreshed locally on resume instead of being treated as
  the reuse key.

This means resume is intentionally **byte-stable**, not "best effort".
If generator semantics change and a regenerated module no longer matches
the saved `.sv`, that old tree is evidence only; use a fresh `--out`
tree for the new semantics instead of trying to cross that boundary in
place.

That last point is important. In the real smoke proof, the saved `.sv`
matched exactly while the checkpointed metrics did not, which means
metrics are too strict a resume key even when the emitted artifact is
unchanged. The load-bearing truth for reuse is therefore the emitted
module, not the old metric blob.

The newer fast path exists to avoid replaying hundreds of already-proven
modules on the **same binary** just to reconstruct RNG state. Each
fresh checkpoint now records:

- a generator checkpoint (ChaCha stream position + next module index),
- a hash of the emitted `.sv`, and
- a fingerprint of the current `tool_matrix` binary.

When all three match, resume can restore the generator directly and
reuse the saved report without regenerating that module. If any of them
do not match, the old strict replay path stays in force. That keeps the
same byte-stable correctness bar while removing the most painful
same-build resume cost.

Older output trees without sidecars are still resumable: `--resume`
will validate the saved `.sv`, rerun the current tool surface once for
that module, and then write the new checkpoint sidecar.

Likewise, older sidecars that predate the generator-checkpoint metadata
still resume correctly; they simply pay the strict replay cost once and
are upgraded in place to the newer, faster format.

One more operational detail now matters in practice: once a proof or
cleanup change alters emitted `.sv`, an older frontier tree becomes
historical evidence only even if it was the latest live checkpoint at
the time. That happened to `/tmp/anvil-tool-matrix-phase1-real-r18`
after the rollback / compare-cleanup repairs: the tree still records a
real 372-checkpoint both-mode frontier, but current code must continue
from a fresh output tree instead of trying to "upgrade" it in place.

### Cleanup exact proofs must stay compare-aware without becoming broad again
The post-construction `fold_proven_gates` pass now follows a deliberate
split:

- the **general** cleanup exact prover stays tiny-only (small width,
  small support, small endpoint count) so it cannot reintroduce the old
  large-cone runtime blowups; but
- compare gates still get the bounded unsigned-compare proof even when
  the cone is too large for the general cleanup exact gate; and
- shift gates (`Shl` / `Shr`) may still use the **bounds-only** exact
  result even when the cone is too large for the general cleanup exact
  gate.

That split exists because "large cone" and "cheap compare tautology"
are not the same thing. A dead-selector rhs can make `x >= 0` or
`1 < dead_rhs` obviously constant even when the whole cone's endpoint
set is wider than the general cleanup exact gate allows. Likewise, a
large-endpoint rhs range can still make `2'h1 >> rhs` or `x << rhs`
obviously zero. Those compare/shift revisit paths are therefore
downstream-cleanliness exceptions worth keeping separate from the
broader exact-value cleanup budget.

### `constant_prob = 0.1`
Default chosen to prevent constants from dominating cone leaves. Real synthesis-stress workloads may want lower (≤ 0.05); aggressive pattern coverage may want higher. Revisit after first seed sweep with metrics on what fraction of generated cones survive non-triviality on the first attempt.

### `terminal_reuse_prob = 0.3`
Probability that, when a cone reaches a leaf decision and the signal pool has matching-width entries, it picks an existing pool entry rather than emitting a constant or recursing further. Higher = more sharing-like behavior even before Phase 3 explicitly turns on `share_prob`. Default is a guess; tune after Phase 1.

### `share_prob = 0.3` default
The non-leaf DAG-sharing fork is enabled by default at a modest rate. Every operand has a 30% chance of terminating at an existing pool entry rather than recursing. This is the Phase 2 guiding mode: cones are a mix of tree and DAG shapes, chosen per recursion point. Raise (0.5–0.9) for fanout-stress generation; lower (0.0–0.1) for wide-sprawling tree-ish cones. `share_prob = 0.0` does not produce *pure* trees — `pick_terminal` still reuses matching-width pool entries at forced leaves. The distinction is: `share_prob` controls *non-leaf* sharing; leaf-level reuse is always on.

### Phase 2 share-gate metric: normalize by total nodes
The first repo-owned Phase 2 gate attempt tried to prove "controlled
sharing factor" with raw `total_shared_nodes`. The real run showed that
proxy was backwards: when `share_prob` rises, ANVIL often reuses enough
existing structure that the entire graph collapses, so the *absolute*
count of shared nodes can fall even while the graph becomes more
shared. The repo-owned `tool_matrix --phase2-share-gate` therefore uses
`shared_node_fraction = total_shared_nodes / total_nodes` as the
monotonic proof metric and records node-count collapse alongside it.
Current closure proof on `/tmp/anvil-tool-matrix-phase2-share-r1`:
`0.4122 @ share_prob=0.0`, `0.4232 @ 0.3`, `0.4386 @ 0.9`, while
`avg_nodes/module` drops from `4727.56` to `3525.01` to `2117.76`.

### Phase 3 should have its own structured-surface gate
Once the `case`, `casez`, bounded `for`-fold, selectable
`Slice` / `Concat`, and variable-shift surfaces were all landed, the
remaining honest Phase 3 blocker was no longer feature breadth. It was
evidence breadth.

That shape now lives in the harness itself as `tool_matrix
--phase3-structured-gate`. The dedicated matrix covers all three live
construction strategies under `identity_mode = node-id` +
`factorization_level = e-graph`, and the report is allowed to go green
only if it proves the landed Phase 3 surfaces directly:

- priority encoder
- one-hot and encoded comb mux
- procedural `case`
- procedural `casez`
- bounded procedural `for`-fold
- one-hot and encoded flop mux
- selectable `Slice`
- selectable `Concat`
- variable shifts

The closure proof now lives at
`/tmp/anvil-tool-matrix-phase3-structured-r4/tool_matrix_report.json`
with `21` scenarios, `210` total modules, `coverage_gaps = []`, and
`210/0` pass-fail in Verilator plus both repo-owned Yosys modes.

### Semantic merge proofs also need a cone-size budget
The first real Phase 3 gate run did not fail in Yosys or Verilator. It
stalled inside `merge_equivalent_gates`, specifically
`semantic_cone_proof -> evaluate_node_under_assignment`.

The root cause was subtle but real: *small endpoint support is not a
sufficient runtime guard by itself*. A settled cone can depend on only
2 or 3 canonical leaf endpoints and still contain a very large internal
graph. Brute-forcing every assignment through that whole graph turns
compaction into a whole-cone evaluator.

The durable fix is now explicit in `src/ir/compact.rs`:

- cleanup-time exact proofs stay on their already-strict tiny-cone path
- semantic merge proofs have their own reachable-cone budget
- once that budget is exceeded, compaction falls back to the
  structural proof path instead of chasing semantic equivalence at any
  cost

That keeps the semantic merge fragment live where it is valuable while
stopping large settled cones from becoming a runtime trap.

### `gate_*_weight` defaults
3:2:1:1:1 (bitwise:arith:struct:compare:reduce). Bitwise dominates because bitwise gates are the most type-flexible and produce the widest cones. Comparisons are weighted lower because they collapse the width to 1, which limits downstream cone depth. These are gut-feel; replace with measurements when phase-1 sweeps land.

### `flop_mux_encoding_prob = 0.5`
Default chosen to give equal motif exposure to OneHot and Encoded styles across a random seed sweep. If post-synthesis metrics show that one style dominates as a bug-finding target, bias the default. The knob also allows users to run workloads stressing only one style for targeted testing.

### `flop_qfeedback_prob = 0.5`
Default 50/50. No empirical data yet. Real designs probably lean heavier on QFeedback (hold-on-no-write is far more common than zero-on-no-write), but generating the less-common pattern is precisely where random generation earns its keep. Revisit with data.

### QFeedback-in-Encoded: replace `data_0` with Q
Alternative considered: add Q as an extra (M+1)th entry encoded with the largest select value. **Rejected** because:
- It would require the sel bus to be one bit wider than `ceil(log2(M))` whenever M is a power of 2, breaking the clean "M mux entries ⇔ `ceil(log2(M))`-bit sel" invariant.
- The "slot 0 is Q" convention mirrors common RTL idioms where the zero-index / reset state is treated specially.
- It keeps M as the single knob for mux entry count across both styles.

---

## Rejected alternatives

### Annotated-EBNF runtime engine
Considered: a generic attribute-grammar interpreter that reads an annotated SV grammar at runtime and produces output. **Rejected** because:
- SV's grammar is enormous; encoding all of it is months of work for productions we will never emit.
- Threading mutable scope/driven-set/flop-worklist state through pure inherited/synthesized attributes is awkward; it really wants `&mut Context`.
- Extending the grammar engine for a new motif is comparable in effort to adding a Rust enum variant + emitter arm, with much worse error messages.

The grammar view is preserved as a *correctness argument* (every constructor preserves invariants ⇔ every production is valid under its attributes). Not as a runtime artifact.

### Oracle / reference simulator
Considered: a Rust evaluator that walks the IR with concrete input vectors and produces expected output values, used both for non-triviality filtering and for downstream tool testing. **Rejected** because:
- Doubles implementation effort.
- Introduces a second correctness question (is our interpreter LRM-correct?).
- The user's stated goal is *generation*, not building a full shadow
  simulator or tool-oracle inside `anvil`.
- Non-triviality is cheaper to enforce by dep-set tracking + structural rules; multi-vector evaluation is overkill for that use case.

That does **not** lower the output-quality bar. The generator is still
expected to emit modules that run cleanly in downstream tools.
Verilator / Yosys are external validators, not the place where
`anvil` gets to finish the job.

### `always_comb` + `case` for encoded-mux flop D

Considered for the Encoded-style flop D: emit an `always_comb` block with a `case (sel)` statement driving D. **Rejected** in favor of a chained ternary over `Eq(sel, k)` because:

- The emitter already handles `Mux` and `Eq` as ordinary `GateOp` variants; nothing new is required.
- `case` would require introducing procedural block emission (`always_comb`) and name-binding for the case target, which is a bigger scope than a uniform expression-level SV emitter.
- Synthesis tools produce the same netlist from both forms for well-formed one-cycle muxes; the readability difference only matters to a human reader.

If a future motif (e.g., FSM state encoding) genuinely requires `case`, revisit then.

This remains the right decision for **flop D** muxes even after the
Phase 3 case-mux slice landed. The new case surface is a separate
combinational block motif with its own knob (`case_mux_prob`) and its
own structured gate kind; the flop path stays expression-based and
keeps its existing chained-ternary semantics.

### Casez muxes are a separate structured surface, not a decorated case-mux

The right shape for the `casez` slice was **not** to smuggle wildcard
syntax into the existing `CaseMux` gate or to make the emitter infer
question-mark patterns from ordinary indexed arms. `case` and `casez`
exercise different frontend/elaboration paths, so the IR should say so
explicitly.

That is why the slice introduced a distinct `GateOp::CasezMux` plus its
own knob (`casez_mux_prob`). Each arm stores a constant pattern, a
constant wildcard mask, and a data node. The emitter renders those as a
procedural `always_comb casez (sel)` block; the validator enforces the
constant-pattern contract; and the exact evaluator in `ir::compact`
understands the same first-match semantics.

Generation deliberately keeps the wildcard patterns **non-overlapping**
by construction. That preserves the intended "wildcarded mux" surface
without accidentally turning the new motif into a priority-case stressor
on top of the syntax stress we actually wanted.

### Bounded unrolled logic belongs in the IR as a block, not as emitter sugar

The right shape for the statically bounded `for` slice was to model it
as its own structured combinational block, not to hope that repeated
operator trees would "look enough like a loop" in emitted SV.

That is why the slice introduced a distinct
`GateOp::ForFold { kind, trip_count, chunk_width }` plus its own knob
(`for_fold_prob`). The IR carries the fold kind (`xor` / `or` / `and` /
`add`), the exact static trip count, and the chunk width. The single
operand is a packed source bus of width `trip_count * chunk_width`.

The emitter then has one honest job: declare the target as `logic`,
emit an `always_comb begin`, initialize the accumulator, and render a
bounded `for (int i = 0; i < N; i++)` loop over
`src[(i * chunk_width) +: chunk_width]`. The validator enforces that
shape directly, and the exact evaluator in `ir::compact` evaluates the
same chunk-fold semantics.

This keeps the syntax surface real. Downstream tools see an actual
procedural bounded loop, not just an expression tree that happens to
resemble one semantically.

### Selectable Slice/Concat must be non-degenerate by construction

Making generic `Slice` / `Concat` first-class selectable shapes was not
just a matter of adding them to `pick_gate`. The naive version would
have "landed" them and then immediately lost them again:

- selectable `Slice` would often degenerate to the full-width identity
  and disappear under the peephole layer
- selectable `Concat` would sometimes degenerate to the single-operand
  identity and disappear the same way

So the right design is to make the selectable forms intentionally
non-degenerate:

- selectable `Slice` always uses a source wider than its high bit
- selectable `Concat` always partitions the output width across at
  least 2 operands

That keeps the new surface honest. We are exercising real frontend
surface area, not just incrementing counters on gates that the settled
graph will erase as trivial identities.

### Late mixed-constant cleanup after remaps

Intern-time constant folding is not enough by itself once the
post-construction cleanup passes start remapping settled graphs. A gate
that was clean when originally interned can later become something like
`1 + x + inner`, where `inner` is subsequently proven/remapped to `1`.

The right place to address that is **not** to overcomplicate
associative flattening or to relax the strict duplicate doctrine; it is
to run a small late cleanup pass on the settled graph. That is now
`fold_mixed_associative_constants` in `src/ir/compact.rs`, wired after
the posthoc associative-normalisation points. It re-aggregates
associative constants (`1 + x + 1 -> x` at width 1, `1 + x + 1 -> 2 +
x` at width 8, `3 * x * 5 -> 15 * x`, etc.) after remaps expose those
opportunities.

### M = 1 mux arm

Excluded from `pick_mux_arm_count` by design. A 1-arm mux is algebraically `sel ? data_0 : 0` (ZeroDefault) or `sel ? data_0 : Q` (QFeedback) — in either case a trivially-simplified shape that adds no motif diversity over what a simple 2-arm mux or an M=0 direct cone already covers. Allowing M=1 would bloat the generator's decision space without expanding the generated-SV distribution meaningfully.

### `#![allow(clippy::too_many_arguments)]` in `src/gen/cone.rs`

The cone-recursion helpers legitimately thread 5–8 context references (`Generator`, `Module`, `SignalPool`, `FlopWorklist`, `width`, `depth`, `exclude`, sometimes more). Packaging them into a `Ctx` struct would help readability but also forces mutable-borrow juggling that fragments the code with no semantic benefit. The lint is silenced at the module level rather than per-function to avoid the ceremony of annotating every helper. Not recommended for modules outside `gen/cone.rs`.

### Generate-then-validate (filter loop)
Considered: emit random IR with looser invariants, then run the validator and discard rejected outputs. **Rejected** because:
- Untestable bound on generation time.
- Tempts contributors to weaken constructors and rely on the validator, leading to silent correctness drift.
- Complex invariants (dep-set non-emptiness) are far more expensive to check post-hoc than to maintain incrementally.

The bounded retry in `cone::build_cone_with_retry` is the *only* exception — it exists because dep-set non-emptiness depends on terminal selection in a way that cannot always be predicted at the gate level (e.g., when all available pool entries happen to be constants). Retry budget is small (4) and falls back to accepting the last attempt.

---

## Implementation gotchas

### Reproducibility hazards
- `HashMap` iteration order is *not* stable across builds. If iteration order ever affects output, switch to `BTreeMap` or sort the keys explicitly. The current code avoids this; new contributions must too.
- `f64` non-associativity is fine for probability comparisons but never use `f64` arithmetic to compute IR fields — only RNG-driven discrete choices.
- `rand::thread_rng()` is forbidden everywhere. All randomness flows from the seeded `ChaCha8Rng` in the `Generator`.

### IR arena indexing
`NodeId` is `u32`. We use `Vec<Node>` indexed by `u32`. This is fine for the foreseeable size range (modules of ≤ 10⁶ nodes). If we ever need more, the change is local to `ir/types.rs`.

Indices are stable for the lifetime of a `Module` because we only ever push, never remove. The bounded retry in `cone::build_cone_with_retry` rewinds by `Vec::truncate`, which is safe because no other code holds `NodeId`s referring to the rewound region.

### Width 0 is illegal
`Config::validate` requires `min_width >= 1`. Width-0 signals are not synthesizable and SV does not allow them. Do not relax this.

### 128-bit constant cap
Constants fit in `u128`. Modules with `max_width > 128` are technically allowed, but the constant generator emits `0` for any width ≥ 128. This is a deliberate simplification; widening the constant representation is straightforward when needed.

---

## Testing strategy notes

- **Unit tests** live in each module under `#[cfg(test)] mod tests`. Test IR constructors enforce invariants; test gate width rules; test dep-set propagation; test the emitter on hand-built IRs.
- **Integration tests** in `tests/`: cross-seed generation + IR validation + reproducibility.
- **External smoke tests** (Verilator lint, Yosys synth) are gated by env vars so they are skippable for developers without those tools. CI must enable them.

A failed external smoke test is always a generator bug. Do not "fix" by tweaking generator output — find the root invariant violation and fix it.

Same principle for the IR validator (`src/ir/validate.rs`): if it rejects real generator output, that's a generator bug. The validator is an active safety net, not a gate to be worked around. The per-gate arity + width checker added in slice `2026-04-15-0008` is specifically designed to catch width bugs in the new flop-mux assembly code, where gates are constructed by hand rather than by recursion — the most likely place for a width-arithmetic slip.

### Canonical state backreferences are validator-owned (2026-04-20)

Once `merge_equivalent_flops` started rewriting state after drain,
`Flop.id`, `Flop.q`, and `Node::FlopQ { flop, .. }` stopped being
"born correct and forgotten" fields. They are now recovery-critical
identity links that a bad renumbering pass can corrupt.

`ir::validate::validate` now owns that contract:

- every output drive root exists before root inspection;
- `m.flops[idx].id == idx`;
- `Flop.d`, `Flop.q`, and every `NodeId` stored inside `FlopMux`
  exist;
- `Flop.q` points at a `Node::FlopQ` whose backref and width match
  the owning flop; and
- every `Node::FlopQ` points at a real flop and is that flop's
  canonical `q` node.

Keep the emitter dumb. If any of these invariants fail, fix the
producer or rewrite pass; do not add emitter-side repair logic.

### Compaction now legitimises dynamic absorbing folds (2026-04-20)

Before `compact_node_ids`, the cautious rule for absorbing constants
was "only fold if the other operand is not a gate", because
`x & 0 -> 0`, `x | all_ones -> all_ones`, and `x * 0 -> 0` would
otherwise orphan a dynamic subgraph immediately.

That restriction is now obsolete. Finalisation already performs a
reachability compaction from real roots and rebuilds the dedup tables,
so these local identities are safe to fire regardless of whether the
other operand is a gate. In other words: once compaction exists, the
correctness risk is no longer "did we orphan something?" but "did we
miss an identity we should have collapsed?"

The practical consequence showed up in tool smoke:

- the remaining seed-42 Verilator `UNSIGNED` / `CMPCONST` warnings
  were not tool quirks;
- they were missed IR-local tautologies; and
- the right fix was to strengthen the rewrite ladder
  (absorbing folds, unsigned boundary comparisons, const-selector
  muxes), not to suppress or special-case Verilator.

This is the pattern to keep following for the NodeId-identity roadmap:
when equivalent local forms are discovered in emitted SV, first ask
whether they should have already become the same node in the IR.

### Signoff-quality and downstream-tool exercise are not competing goals (2026-04-20, refined 2026-04-26)

The user clarified the product direction explicitly, and later refined
the terminology around it:

- `anvil` should become a signoff-level quality random
  by-construction synthesizable RTL generator;
- generated HDL artifacts should be accepted by downstream HDL
  consumers by default; and
- `anvil` corpora should still be rich enough to exercise parsers,
  elaborators, RTL compilers, linters, simulators, synthesizers, and
  similar consumers.

Those statements are compatible. The project is **not** trying to expose
tool bugs by emitting junk, malformed syntax, or semantically dubious
RTL. The downstream-tool exercise value comes from breadth, interaction
richness, factorization pressure, stateful motifs, hierarchy, memories,
and other legal-but-hard combinations that downstream HDL consumers
should accept. Verilator and Yosys are repository validation tools for
that acceptance promise, not the only product targets.

When choosing between slices, prefer work that strengthens one of these
two axes without regressing the other:

1. broader / harder legal design space; or
2. stronger confidence that generated output is clean and robust in
   downstream HDL consumers.

### Purpose terminology clarification (2026-04-26)

The user clarified the wording around ANVIL's purpose:

- avoid calling ANVIL "constrained-random" unless that term is
  explicitly redefined away from SystemVerilog/UVM-style user-authored
  constraints or solver-driven randomization;
- the preferred short description is **random by-construction
  synthesizable SystemVerilog RTL generator**;
- ANVIL targets generated HDL artifacts that downstream consumers can
  accept: parsers, elaborators, RTL compilers, linters, simulators,
  synthesizers, and related tools;
- Verilator and Yosys are repository validation tools for syntax,
  elaboration/lint, and synthesis acceptability, not the only product
  targets; and
- ANVIL-generated corpora can still be used to stress downstream tools,
  but that is a use of the legal generated artifacts, not a license to
  describe ANVIL as primarily a malformed-input fuzzer or generic
  toolchain stress tester.

Follow-up gotcha (2026-04-27): package metadata is part of that same
terminology surface. `Cargo.toml` must not keep stale
`constrained-random` wording after README, Rustdoc, and mdBook text have
been corrected; Cargo metadata is visible to tooling and cold-start
readers before they open the longer docs.

### Verbatim user doctrine: structure over intended functionality (2026-04-20)

The following user guidance is intentionally logged **verbatim** because
it is doctrinal and should steer future implementation choices:

> Let's be clear. Generating module by recursively generating fanin cones of its outputs, mechanically means that the resulting functionality will be gibberish but that's not the point. Having functioning behavior makes no sense here. For some modules, we might get some usable functionality but that's not the goal. The ultimate goal is to be able to generate synthesable legit RTL code that downstream tools (parser, synthesizer, linter, ...) can ingest.
>
> My construction we are not aiming at functionality but at structure, capiche.
>
> ANVIL will be able to create complex to very complex synthesizable RTL code.
>
> Any functionally correct synthesizable RTL code is undistinguishable from an functionally incorrect or even gibberish code at first sight, to ensure function correctioness one need functonal verification which needs to match a specification against a RTL module.
>
> So no one can tell at first glance whether a RTL is gibberish or functionally correct with a specification, meaning for most of what will be generated, function correctness is not the goal and can't be by construction.
>
> But they are features that will create functionally correct blocks.

Operational consequence: optimize ANVIL primarily for structural
legitimacy, synthesizability, complexity, and downstream-tool
ingestibility. Treat whole-module function correctness as out of scope
unless a feature introduces a local block motif whose own behavior is
well-defined by construction.

### Broader artifact-family mandate (2026-04-20)

The user then broadened the scope again and explicitly corrected one
important boundary:

> It might sound contradictory but in addition to what's already
> described in the roadmap, book and live docs, I think it would good
> to include support for such things in the roadmap, book and live docs
> in order for ANVIL to be able address a lot more types of SV files
> formats as output. Being able to generate various types of pseudo
> random files for various types on downstream consumers would be a
> great plus, I think.
>
> In fine, I want ANVIL to be able accurately and precisely address the
> initial request, in full.
>
> ANVIL shall be the go to tool for everything (pseudo random) HDL
> generation related thing.
>
> I don't think this contradict the current roadmap of AMVIL that much,
> it is just that we are broadening the type HDL outputs we can target.
>
> As you wrote it clearly above, right now ANVIL is still a "leaf-module
> typed circuit generator", I agree.
>
> We need to start somewhere, but that is not the end goal.
>
> So we need to be able to embrass more output artifact types.
>
> This "valid-by-construction synthesizable lane” is still valid, and
> it will stay that way!
>
> We are just generating more types of valid-by-construction
> synthesizable artifacts.

Operational consequence:

- the current leaf-module typed circuit generator is now explicitly the
  **first artifact family**, not the whole product;
- future broadening still stays inside the
  **valid-by-construction synthesizable** contract;
- the first requested additions are oracle-backed micro-design corpora,
  source-level parameter / hierarchy / package IR, and explicit
  expected-facts manifests; and
- an earlier idea of broadening via invalid/reject corpora is **not**
  the adopted direction for ANVIL after the user's correction above.

This is a real scope change for planning, not a soft aspiration. The
roadmap now needs explicit phases for these broader synthesizable
artifact families.

### Repo-owned tool matrix harness (2026-04-20)

The "no hidden bias" / "exercise all axes" doctrine now has an
executable first form in the repo: `src/bin/tool_matrix.rs`.

The design choices for this harness are deliberate:

- it is a Rust binary in-repo, not an external shell script, so it can
  reuse `Config`, `Generator`, metrics, and manifest formats directly;
- it uses a **curated matrix**, not one giant Cartesian product, so the
  sweep stays fast enough to run routinely while still covering the
  load-bearing axes:
  - interleaved ladder sweep across `relaxed` plus every
    `factorization_level` rung,
  - strategy sweep across `sequential` / `shuffled` / `interleaved`,
  - a share-heavy comb-only profile,
  - a motif-heavy sequential profile;
- it reuses structural metrics as the coverage surface instead of
  inventing a second observability stack; gate kinds, block counters,
  and knob roll attempts/fires already tell us whether a scenario
  actually exercised what it claimed to stress; and
- it exits non-zero on downstream-tool failures because the point is to
  surface generator bugs, not to produce a pretty report while quietly
  accepting red runs.

The first smoke run after landing the harness was immediately useful:
it found one real emitter bug (`logic[0:0]`-style scalar slice
emission) and, after that fix, reduced the remaining failures to the
warning-cleanliness bucket (`CMPCONST` / `UNSIGNED` under Verilator).
That is exactly the intended feedback loop for the tool-clean
industrialization lane.

### Comparison warning-cleanliness is partly a generator concern, not only a factorization concern

The follow-up `tool_matrix` slice made an important distinction
explicit in code: obviously-constant unsigned comparisons are not just
"optional peephole opportunities". They are also by-construction
tool-cleanliness hazards.

That means ANVIL now has an **always-on generator-side proof path** for
comparisons in `src/gen/cone.rs`, independent of
`identity_mode` / `factorization_level`. If the generator can already
prove that a comparison is constant, it emits the constant directly
instead of relying on the factorization ladder to clean the shape up
later.

Current proof layers:

- conservative unsigned bounds for easy local identities (`x & 0 = 0`,
  `x | all_ones = all_ones`, `x * 0 = 0`, overshift-to-zero,
  select-known muxes, etc.);
- exact finite-set reasoning for comparison operands up to 8 bits
  wide; and
- replicated-concat correlation handling for shapes like `{N{bit}}`,
  so repeated copies of the same leaf are not treated as independent
  free variables during the proof.

This is intentionally narrower than full semantic factorization: it is
there to keep emitted RTL cleaner across *all* identity/factorization
modes, including `relaxed` and low rungs like `none` / `cse`.

Two implementation refinements became load-bearing once the real
`--phase1-gate` run started surfacing concrete warning files:

- **Exact proof must short-circuit once the result is already forced.**
  A small-width node can depend on a wider cone through `Slice`, so
  "walk every operand recursively until all are exact" is too blunt. If
  an exact prefix has already forced the result, the helper must stop:
  `6'h16 | 6'h39 | tail` at width 6 is already `6'h3f`; `2'h1 * 2'h2 *
  2'h2 * tail` at width 2 is already `0`; `x ^ x` is already `0`; and
  `x <= x` is already `1`. Letting the proof recurse into an irrelevant
  non-exact tail just turns an exact fact into an unnecessary `None`.
- **The small finite-set engine and the settled-graph exact-value
  engine need the same short-circuit doctrine.** The first catches
  narrow local cones directly; the second matters because
  `node_unsigned_bounds` asks "is this gate already exact?" before it
  falls back to interval reasoning. If only one engine gets the
  shortcut, the other can still miss exactly the same downstream
  warning.

Another refinement became necessary once the real `int_nodeid_cse`
frontier hit a correlation-heavy one-hot-mux cone: **exact finite-set
reasoning must also be budgeted.** The helper now carries a shared work
budget and memoizes both exact results and "unknown" results, so it can
still prove small exact facts on narrow cones without turning itself
into an exponential runtime trap on shared cartesian searches. The
durable contract is "prove what is cheap and crisp; otherwise return
`None` and fall back to the cheaper proof layers."

The next fresh-current-code `operand-unique` frontier made one more
refinement necessary: **budget alone is not a good enough admission
rule.** Even a budgeted proof can still waste generator time if ANVIL
keeps entering exact finite-set reasoning on larger shared cones whose
endpoint support is already beyond the intended proof domain.

So the contract is now sharper:

- exact finite-set reasoning is for **small width and small endpoint
  support**, not just small width; and
- the current support cap is **3 canonical leaf endpoints**.

That support cap applies both to `prove_node_exact_value` on one cone
and to the combined endpoint set used by comparison folding. This keeps
the proof useful where it is strongest, while making larger shared
cones stay on the cheaper proof layers instead of burning CPU proving
finite-set facts that are not load-bearing for cleanliness.

The first fresh-current-code both-mode rerun exposed a second, more
basic compare-cleanliness gap: **the cheap proof layer must know a few
arithmetic reflexive identities too, not only comparison tautologies.**

The concrete failing shape was:

- `sub_16 = mul_17 - mul_17`
- `and_49 = mul_18 & mul_18 & sub_16`
- `lt_0 = add_13 < and_49`

Verilator quite reasonably warned that the unsigned comparison was
constant. The missing fact was just `x - x = 0`.

The exact finite-set engine was not the right place to rely on for this
because it may legitimately decline a cone. The **cheap** layer has to
know it too. So `exact_gate_value` and `node_unsigned_bounds` now both
encode reflexive subtraction directly. Durable rule:

- local exact/bounds proofs should carry the cheapest algebraic facts
  that directly prevent mainstream tool warnings, even when those facts
  do not require the heavier finite-set prover at all.

### Downstream warnings are a generator bug, and the final graph gets a last proof pass

The follow-up slice closed the remaining `tool_matrix` warning bucket by
making two policy changes explicit in code.

First, ANVIL now runs a post-construction proof-cleanup pass in
`src/ir/compact.rs` (`fold_proven_gates`) after cone construction and
again after the sharing/remap passes settle. The key distinction is
timing: some exact proofs are not visible when a gate is first
constructed, but become visible later once remaps, merges, or other
local simplifications have changed the graph that the gate actually
sees. That pass:

- rewrites any gate whose current cone is provably exact into a
  constant in place; and
- rewires muxes whose selector is now provably constant.

One more settled-graph wrinkle showed up immediately afterwards:
remap-producing post-construction passes can reintroduce legal
associative nestings **after** the intern-time Associative layer has
already done its work. The live example was a width-1 `Add` whose
operand was later remapped to another width-1 `Add`, leaving
`nested_associative_operand_count = 1` at default knobs even though
flattening was still legal under the strict duplicate policy.

The durable rule is: **any pass that can change which already-built
node an operand points at may need to restore associative normal form
afterwards.** ANVIL now does that with
`flatten_posthoc_associative_gates(&mut Module)` in `src/ir/compact.rs`
after `fold_proven_gates` and after `merge_equivalent_gates`. The pass
uses the same duplicate policy as the intern-time Associative layer:
`And`/`Or` dedup, `Xor` pair-cancels, `Add`/`Mul` flatten only when the
flat list would still be legal at the current
`operand_duplication_rate`.

The proof stack now has three complementary layers:

- construction-time local proofs in `src/gen/cone.rs`,
- post-construction exact-value cleanup on the settled graph, and
- bounded semantic identity / sharing for the `e-graph` fragment.

One more durable constraint became explicit when the fresh current-code
`nodeid-cse` frontier stalled during resume: sampling the live process
showed the hotspot in `ir::compact::fold_proven_gates` /
`semantic_exact_value`, not in Yosys or Verilator. The settled-graph
cleanup prover is therefore intentionally **stricter** than the
generator-side semantic-sharing passes. Today it only brute-forces cones
that are all of:

- at most 8 bits wide;
- at most 10 total support bits; and
- at most 3 canonical leaf endpoints.

If a cone falls outside that tiny cleanup surface, the pass memoizes
`None` immediately and moves on. Durable rule: late proof-cleanup exists
to scrub obvious constants for downstream-tool cleanliness, not to widen
the main identity/factorization contract at arbitrary runtime cost.

### Narrow slices of wide cones are still narrow proof domains (2026-04-20)

The next live warning bucket made a subtle point painfully concrete:
the small finite-set engine is allowed to be width-bounded, but it is
not allowed to treat a narrow `Slice` result as "unprovable" just
because the source cone is wider than 8 bits. A 14-bit or 25-bit source
feeding an 8-bit slice still yields an 8-bit proof problem.

The durable implementation rule is:

- if a narrow slice's source is already exact, use that exact value;
- otherwise, if the source is too wide for direct enumeration, fall
  back to the full narrow output domain instead of returning `None`.

That fallback is conservative but still useful: it keeps later local
operations (`Or` with forcing constants, exact shifts, subtract-small,
dynamic overshift) in the proof path, which is enough to recover exact
facts like "this `Shr` is forced to zero". Returning `None` too early
throws away that whole proof chain.

One more shift-specific wrinkle showed up later in the fresh
`associative` frontier: some rhs cones are too large for the general
small-support exact enumerator **as whole cones**, but still have a
tiny value domain because they are really just boolean-mask arithmetic
(`{8{bit}} + constant`, similar patterns). The durable rule is:

- shift overshift proofs may use a tiny-domain rhs fallback for narrow
  boolean-mask arithmetic, even when the whole cone is too large for
  the main exact small-set engine.

That fallback stays intentionally narrow: width <= 8, tiny result-set
cap, and only a few structural forms. It exists to suppress pointless
dynamic shifts whose rhs is semantically always oversized, not to
replace the main semantic-sharing machinery.

### Finalisation liveness must be output-rooted, not flop-table-rooted (2026-04-20)

The dead-register Verilator warning exposed a mismatch between Rule-18
gate liveness and sequential liveness. The old compaction pass rooted
every `flop.q` unconditionally because the flop existed in `m.flops`.
That preserved dead state even when no output cone, live flop D-cone,
or other retained logic ever consumed that Q.

The durable rule is:

- start final liveness from output drive-roots;
- when the walk reaches a live `Node::FlopQ`, mark the owning flop
  live and pull in its `d` / mux-held nodes;
- drop any flop whose `Q` is never reached by the live graph.

That is the sequential analogue of Rule 18: state is live because it is
observed by retained logic, not because it once got allocated.

### Post-remap identity cannot violate strict Add/Mul duplicate policy (2026-04-20)

Late proof / sharing passes operate after construction, so they can
collapse two previously-distinct child cones to one canonical node.
That is fine in general, but under strict `operand_duplication_rate`
the final emitted IR is still not allowed to contain duplicate
`NodeId`s inside an `Add` or `Mul` operand list.

The durable rule is therefore stronger than "the remap is semantically
valid":

- a candidate remap is only acceptable if every strict `Add` / `Mul`
  consumer remains duplicate-free after the rewrite.

ANVIL now enforces that by pruning duplicate-introducing remaps before
they are applied in `fold_proven_gates` and `merge_equivalent_gates`.
This preserves the default "zero duplicate operands" doctrine without
backing away from late exact-value cleanup or bounded semantic sharing.

### Evidence slices are legitimate when the real gate frontier moves materially (2026-04-20)

Not every important slice changes code. Once the user set the quality
bar as "no warnings or errors from Verilator and Yosys", the real
`tool_matrix --phase1-gate` run became part of the implementation loop,
not just a nice-to-have afterthought.

That means there is a legitimate kind of slice whose output is:

- a materially advanced real downstream-clean frontier,
- recorded precisely (scenario names, module counts, command line), and
- committed into the live docs so the next session does not restart the
  same evidence climb from memory or vibes.

The key is that the checkpoint must be **material**, not cosmetic. In
this session, moving from the earlier 76-module clean frontier to 246
clean modules across multiple identity/factorization lanes cleared that
bar easily. It changed what we know about the repaired generator.

So the durable rule is:

- if a long real gate run advances the proven clean frontier
  substantially, it is acceptable to checkpoint that evidence as its own
  slice, even if no code changed in that commit.

That rule matters for crash recovery too, which is exactly why the
commit workflow is strict in the first place.

Second, the repo-owned downstream harness now treats warnings as
failures rather than as "successful but noisy" runs. `tool_matrix`
scans tool output for warning markers and marks the invocation failed
even if the process exit status is zero. The Yosys script was also
tightened from `synth` to `synth -noabc` so the matrix does not accept a
self-inflicted ABC combinational-network warning and then pretend the
run was clean.

This is a durable project rule now: for repo-owned Verilator/Yosys
evidence, "green" means no errors and no warnings.

### The 1000-module Phase 1 gate should be a first-class harness mode

Once the smoke matrix was green, the next missing piece was not more
doctrine. It was executable ergonomics. The Phase 1 exit criterion had
become "run the same harness, but remember to multiply the scenario
count, pick a large enough `--modules-per-scenario`, and also remember
that coverage gaps must fail."

That shape now lives in the harness itself as `tool_matrix
--phase1-gate`:

- it auto-enables coverage-gap failure; and
- it raises `modules_per_scenario` high enough to generate at least
  1000 modules total across the built-in scenario set.

The deliberate choice here is to encode the gate in the repo-owned tool
rather than leaving the phase-exit arithmetic in roadmap prose. When a
quality gate matters, the project should be able to invoke it directly.

### Codebase suitability assessment: four steering gaps (2026-04-20)

The short answer to "is the existing codebase suited to the goal?" is:
**yes, as a foundation; no, not yet as a finished system**.

Why "yes": the architecture already matches the problem. `gen` builds a
typed IR instead of text, `Module::intern_gate` is a single
construction-time chokepoint for combinational identity,
`ir::compact` owns post-drain cleanup and state-finalisation work,
`validate` owns the invariant contract, `config` keeps the control
surface explicit, and the SV emitter stays deliberately dumb. That is
the right shape for a signoff-grade legal-RTL generator.

What still needs to stay explicit:

1. **Feature breadth grows above the leaf kernel, not by muddying it.**
   `src/gen/module.rs` is the leaf-module kernel. Hierarchy should land
   as a higher layer (planned `src/gen/hierarchy.rs`), not as ad hoc
   special cases in the leaf path. Likewise, memories/FSMs/aggregates
   should become first-class motifs or module-level generators, not
   emitter tricks.
2. **`NodeId`-as-identity must keep expanding through the IR, not via
   emitter magic.** Today's live coverage is normalized combinational
   identity plus a conservative endpoint-preserving state merge.
   Future work is stronger state identity across richer state graphs and
   later hierarchical/block identity, but it must stay faithful to the
   doctrine: same identity requires proven same functionality with
   respect to the same canonical leaf variables. Keep
   `--identity-mode` as the coarse on/off switch and
   `--factorization-level` as the finer dial; construction strategy
   must stay orthogonal.
3. **Tool cleanliness must be industrialized.** Seed 42 being clean is
   good news, not a stopping point. Each new motif/category/knob needs
   matrixed Verilator/Yosys evidence, retained seed+config
   counterexamples, and root-cause fixes at the IR/generator layer
   rather than warning suppressions.
4. **Structure-first doctrine remains load-bearing.** Absent a
   specification, whole-module functional intent is not the optimization
   target. Invest in legal interaction surfaces, factorization
   pressure, hierarchy, and stateful richness. Functionally correct
   local blocks are welcome; a bundled whole-module oracle is not the
   direction.

### Endpoint-preserving functional doctrine for state identity (2026-04-20)

The user clarified the intended meaning of state equality sharply:

- two fanin cones may **not** share one `NodeId` if they do not have the
  same leaf endpoints as variables;
- the relevant variables are the canonical leaf endpoints: primary
  inputs and/or flop `Q` outputs; and
- the goal is equality by proven same functionality with respect to
  those same endpoints, not equality by visual resemblance or by
  matching graph skeleton alone.

Operational consequence:

- `merge_equivalent_flops` now uses a conservative leaf-aware proof form
  over the already-normalized IR rather than exact `d: NodeId`;
- that proof form now includes a bounded semantic check for
  small-support cones, so some different-shape cones can merge when
  they evaluate identically over the same canonical endpoint set; and
- any future strengthening of sequential identity must preserve the
  canonical leaf namespace. "Rename each owning `q` to SELF" is **not**
  acceptable in strict `NodeId as identity` mode, and neither is
  equating cones solely because they happen to look structurally alike.

---

## Generation-time defects observed in sample output (pending fixes)

Cataloguing real defects observed in sample module `mod_1_0000`
(3 outputs, 10-level fanin, default knobs, graph-first strategy).
These are generator bugs — not SV-emitter or validator bugs.
Enumerated here so the next session can fix them at the root.

- **Constant-select muxes.** Every `wN = (2'h2 == 2'hK) ? ... : ...`
  in the sample is a mux whose select is a *literal* comparison of
  two literals. The select folds at elaboration. Root cause: the
  encoded-mux assembler feeds the select-side recursion through
  the same `pick_terminal` path that can terminate on a constant
  leaf, and the one-hot-mux assembler similarly accepts a constant
  for the per-arm select bit. Fix: in mux-select position, forbid
  constant termination — require a non-constant signal source.
- **N-arity self-cancellation.** `w_21 = i_2 ^ i_2 ^ i_2 ^ i_2 = 0`.
  The N-arity operator expansion re-picks the same pool entry for
  every operand, and `Xor` of even repetitions is zero. Fix: the
  anti-collapse check must look at operand *multiset equality* for
  idempotent / self-inverse operators, not just dep-set
  non-emptiness. (And for `And`/`Or` the same issue produces
  `x & x & x = x` which is a structural collapse, not a zero, but
  still a motif violation.)
- **Coefficient width overflow.** `1'h6` appears — a 6 encoded in a
  1-bit literal, which truncates to 0. Root cause: the linear-
  combination coefficient generator picks the coefficient value
  independently of the operand width. Fix: clamp the coefficient to
  `bits ≤ operand_width`, or widen the literal to the operand width
  and let the top bits be real.
- **Dead wires.** `w_17`, `w_26`, `w_27`, `w_29` are declared and
  assigned but never read. Graph-first speculative pool growth is
  the source; Rule 18 (proposed) addresses this.
- **Stranded flop.** `r_3 <= r_3` — a flop whose D is its own Q and
  whose Q is never read. A no-op. Rule 18 covers this too, as long
  as "consumer" is defined to exclude the flop's own Q feedback.
- **Structurally-identical one-hot arms.** `w_8`, `w_10`, `w_12`,
  `w_14` are all `{w_6,...} & w_5`, meaning four arms of the one-
  hot mux have the same per-arm product. OR-reducing identical
  arms collapses to just the arm value. Fix: in one-hot assembly,
  require per-arm *data* distinctness (or require the per-arm
  select to differ; the current issue is that all arms share the
  same broadcast select bit `w_6`).

All six share a theme the user articulated: signals are being
created without a *reason to exist*. The fixes are three-category:
(1) tighten anti-collapse (operand-multiset check); (2) position-
dependent leaf rules (no const in mux select); (3) width-aware
constant generation. Rule 18 addresses the orthogonal
"unconsumed output" axis.

---

## File-level conventions

- Every Rust source file starts with a doc comment explaining its scope.
- Public types in `ir/types.rs` and `config.rs` get full doc comments. Internal helpers do not need them.
- No multi-paragraph docstrings. One short line; if more is needed, link to `book/`.
- No comments explaining *what* the code does; only *why* when non-obvious.

---

## Construction-time CSE via `Module::intern_gate` (2026-04-15 → 2026-04-16)

Design decision: *all* `Node::Gate` and `Node::Constant` creation is routed through two inherent methods on `Module`:

```rust
pub fn intern_gate(&mut self, op, operands, width, deps) -> (NodeId, bool);
pub fn intern_constant(&mut self, width, value) -> (NodeId, bool);
```

The boolean return is `is_new`: callers that also maintain a `SignalPool` must call `pool.add` only when `is_new` is true, otherwise the pool accumulates duplicate entries for deduped nodes.

Rationale: we need CSE at *construction* time, not as a post-pass. Rule 21 ("AST-instance cap") uses the dedup tables on `Module` as the single source of truth for "which NodeIds represent which expressions."

Rejected alternative: decouple the dedup table from `Module`, keep it in the generator. Rejected because the dedup is an IR-level invariant — the emitter and validator may also want to reason about it, and the tables must survive a `Module::clone()`.

### Snapshot contract with `build_cone_with_retry`

`build_cone_with_retry` rewinds state on empty-dep retries. Before the snapshot fix, it rolled back `m.nodes.truncate(snap_len)` but *not* `gate_instances` / `const_instances`. Stale entries then pointed at truncated `NodeId`s; subsequent intern calls would return a different node than the key promised (witnessed by `const_comparand_across_all_strategies_is_valid` failing at seed 2 Interleaved during the migration).

Fix: snapshot and restore `gate_instances` and `const_instances` alongside `m.nodes`, `m.flops`, pool, and worklist. The `HashMap::clone` cost is bounded by module size — measured negligible on the default knob range.

## Rule 18 "No orphan gates": α construction-time (2026-04-16)

Two enforcement paths were considered:

- **(α) Construction-time:** only create a gate when a specific consumer is already waiting for it. `build_cone` snapshots state before operand construction; on anti-collapse rejection, the snapshot is restored — operand sub-trees vanish from the IR. `process_signal_frame` (interleaved) can't snapshot per-gate because sibling frames have committed, so it delivers one of the existing operand NodeIds as the fallback instead of calling `pick_terminal` (which would create a fresh orphan-prone node).
- **(β) Emission-time tree-shake:** post-generation, compute the live set from drive-roots + flop D/Q transitive fanin, emit only that set.

Rejected β: it's a generate-then-filter step, violating the "by construction" doctrine. User-memory feedback: *"Rule-based generation, not post-hoc filtering."* α is adopted.

Corollary: GraphFirst retired. Its phase-1 speculative pool growth produced 13–27 % orphan gates per module. The variant is kept as a silent CLI alias for Interleaved for backward compat; the dedicated code path (`build_graph_first`, `grow_pool_one_unit`, `*_pool_only` helpers) is unreachable at runtime and may be removed in a future cleanup slice.

## Full factorization doctrine (2026-04-16)

User framing: **`NodeId` is the identity of an expression**; two expressions that are the same mathematically must share one NodeId, different expressions must have different NodeIds.

Implementation ladder (see `book/src/structural-rules.md` Rule 21c):

1. Syntactic CSE (Rule 21) — `(op, operands, width)` key. **Implemented.**
2. Operand-uniqueness (Rule 8 extended) — no NodeId twice in one operand list. **Implemented.**
3. Commutative normalization (Rule 21b) — sort commutative operands before interning. **Implemented.**
4. Associative flattening — flatten `(a+b)+c` to `Add(a,b,c)` when semantically safe. **Implemented.**
5. Constant folding — `x+0 → x`, all-constant evaluation, etc. **Implemented.**
6. Peephole — local algebraic / structural rewrites. **Implemented.**
7. E-graph — full semantic equivalence. **Partially implemented.**
   Default user-requested level. Today's live fragment is still bounded:
   small-support combinational cones can merge post-construction when
   they are proven equivalent over the same canonical leaf endpoints.

`FactorizationLevel::effective()` clamps user requests down to the highest implemented layer so aspirational levels don't error. Today `e-graph` remains the strongest implemented rung, but only as a bounded fragment rather than the full semantic-equivalence aspiration. Construction strategy is orthogonal: `sequential` / `shuffled` / `interleaved` decide build order, while the factorization ladder records how much of the `node-id` identity contract the current build can currently enforce/prove.

## Identity mode is orthogonal to construction strategy (2026-04-20)

User clarification that should remain durable:
**"NodeId as identity" is a mode of operation, not a cone-builder.**

That means:
- `construction_strategy` answers *how fanin cones are walked/built*
  (`sequential`, `shuffled`, `interleaved`, graph-first alias);
- factorization / identity mode answers *when two built objects are
  considered the same thing* and therefore must share one NodeId.

Implementation consequence: expose the peak-sharing / no-sharing
switch as a separate CLI axis (`--full-factorization`,
`--no-full-factorization`) rather than pretending it is another
construction strategy value. Future work on the true NodeId-as-
identity engine must preserve this separation.

## Identity mode is now a first-class typed axis (2026-04-20)

The separation above now lives in the code, not just in the docs:

- `Config` owns a new `IdentityMode` enum with `node-id`
  (default) and `relaxed`.
- `Module` mirrors both `identity_mode` and the requested
  `factorization_level`.
- The actual gating sites consult
  `effective_factorization_level()` instead of reading the raw
  ladder directly.

Design consequence:
- `identity_mode = relaxed` is the coarse hard-off switch. It
  forces the effective level to `none`, so `intern_gate` and
  `intern_constant` always allocate fresh NodeIds.
- `identity_mode = node-id` selects the full-factorization doctrine:
  `NodeId` is the identity of an expression.
- `factorization_level` is then the fine-grained implementation /
  proof-depth selector inside that doctrine. Lower rungs are useful
  diagnostic and stress modes, but they are not alternate semantics for
  `node-id`.

This is the minimum architectural move that makes the future
"NodeId as identity" engine honest: the repo can now talk about
identity mode without smuggling it through the ladder alone.

## Adversarial generation must be modeled as orthogonal axes (2026-04-20)

User clarification that should remain durable:
ANVIL must model all axes of adversarial generation explicitly and use
them efficiently during actual generation; there should be no hidden
bias toward whichever path the current implementation happens to favor.

Practically that means:
- construction strategy (`sequential`, `shuffled`, `interleaved`,
  graph-first alias) is one axis;
- identity mode (`node-id` vs `relaxed`) is another;
- factorization level is a third;
- motif/category weights, sequential density, width/depth ranges, and
  the probability knobs are additional orthogonal axes.

Implementation consequence: whenever a new generator feature lands, the
question is not only "does it work?" but also "which axis did it add,
how is that axis surfaced, how is it measured, and how do we avoid
silently under-sampling it during real workloads?"

## Stateful identity must be decided post-drain (2026-04-20)

For gates and constants, identity is knowable at intern time: the
full key exists when `intern_gate` / `intern_constant` runs.

Flops are different. `build_flop_leaf` allocates a Q leaf
immediately, but the flop's semantics are not complete until the
worklist later constructs its D-cone. So the first honest stateful
extension of "NodeId as identity" cannot be an allocation-time guess;
it has to run after drain.

Current rule: after `summarize_flop_mux_metadata`, flops are merged
iff they have the same emitted-state signature over the same canonical
leaf variables: same `width`, `reset_kind`, `reset_val`, and the same
leaf-aware D-cone proof form. Today that proof form has two rungs:

1. normalized structural proof over the already-canonicalized IR; and
2. bounded semantic proof for small-support cones (enumerate every
   endpoint assignment, key by the resulting truth table).

Construction provenance (`FlopKind`, cleared mux operand metadata) is
deliberately ignored once D exists, because emitted hardware semantics
are carried by width/reset/D-cone meaning, not by how the generator
happened to assemble them.

This is intentionally narrower than full sequential equivalence. Two
cones that happen to compute the same function but are not reduced to
the same proof form by the current ladder, or whose endpoint support is
too large for the bounded semantic check, are not merged yet. That
deeper coinductive story remains a
future slice.

## Bounded E-graph fragment for combinational identity (2026-04-20)

`merge_equivalent_gates(&mut Module)` is now the first live
post-construction combinational extension of the `e-graph` rung.

Current rule:
- gated by `identity_mode = node-id`;
- gated by effective factorization level `>= e-graph`;
- same canonical leaf endpoints are mandatory; and
- functionality may be proven either by the already-normalized
  structural proof form or by a bounded semantic truth table for
  small-support cones.

This is deliberately not the whole e-graph story. It is a bounded proof
fragment that makes the strongest mode honest today while preserving the
user's doctrine that `relaxed` remains a real no-sharing mode and that
construction strategy stays a separate axis.

## Emitter is a dumb serialiser (2026-04-16)

User-memory feedback: *"All thinking, checks, rules' enforcement ought to be done solely at the IR level. By the time you reach emission it is too late to roll back."*

Consequence: `emit::to_sv` iterates `m.nodes` in order and writes. No filtering, no reachability check, no live-set computation. Any invariant worth enforcing must be enforced at IR construction or at a `generate_leaf_module` finalization step — never at the emitter.

The safety-net audit in `generate_leaf_module` (`count_orphan_gates`) is *at the IR level* and warns on Rule 18 violations; it does not modify the IR. The emitter trusts what it is given.

## Rejected: without-replacement operand picking as the default

For And/Or/Xor/Add/Mul operand lists, operand duplicates are caught by `violates_anti_collapse` after operands are picked. A natural alternative is to pick operands *without replacement* at the source — maintain a `HashSet<NodeId>` during the per-operand loop and exclude already-picked NodeIds.

Considered and not adopted as the default because:
1. Pool sizes at default knobs are often ≤ N (the requested arity). Without-replacement falls back to "partial arity" + distribution shift.
2. Anti-collapse + rollback already gives 0 duplicates at default. The without-replacement change would save RNG cycles at the cost of a distribution shift that has no empirically measured benefit.
3. `operand_duplication_rate` is the documented knob for users who want the alternative behaviour.

Retained for reference in case a future motif benefits from it.

## Finalisation trims metadata-only and unused-bit surface (2026-04-19)

This slice locked in a small but important finalisation doctrine:
**emit what the live hardware uses, not the generator's provisional
scratch structure.**

- **Width adapters now expand to the exact target width.** The old
  non-multiple up-width adapter built an oversized replicated `Concat`
  and then sliced it back down. Functionally fine, but it manufactured
  dead high bits that lint tools quite rightly flagged. The adapter now
  builds the exact-width shape directly (`{src[rem-1:0], src, ...}`).
- **`Flop.mux` operand NodeIds are construction-time metadata, not
  emitted hardware roots.** Once `flop.d` is assembled, keeping the
  original select/data operand references around lets metadata-only
  cones survive liveness/compaction even though the emitter never reads
  them. Finalisation now keeps only the variant shape and discards
  those operand references before compaction.
- **Primary inputs are shrunk/pruned to the live bit surface.** After
  compaction, each surviving primary input is reduced to the highest bit
  any live consumer touches, and entirely unused data inputs are
  dropped from the emitted interface. This keeps Verilator from
  reporting unused input bits or dead ports.
- **Residual associative-opportunity metrics now respect duplicate
  policy.** Nested `Add`/`Mul` slots that would introduce duplicates if
  flattened are intentionally preserved at strict
  `operand_duplication_rate`; the metric now matches that semantic
  policy instead of counting those slots as "missed" flattening.

Rejected alternative: paper over the issue in the emitter with
tool-specific lint pragmas. That would hide the symptom without fixing
the IR/finalisation mismatch.

## Yosys ABC/no-ABC is now an explicit harness axis (2026-04-21)

Historically, the repo-owned Yosys smoke path settled on
`synth -noabc` because some runs with the default ABC-enabled `synth`
were reported to blow up or time out. That was useful operationally,
but it left the distinction implicit: future sessions could see one
hardcoded `-noabc` script and have no way to tell whether it was a
deliberate stability baseline, a temporary workaround, or stale cargo
cult.

`tool_matrix` now makes that choice explicit with a Yosys mode axis:

- `without-abc` — current stable baseline, still the default;
- `with-abc` — the repo-owned ABC-enabled harness path; and
- `both` — run both sub-modes per generated file and report them
  separately.

The default remains `without-abc` because that is the last known-good
repo-owned baseline. The point of adding `with-abc` and `both` is not
to silently relax warnings; it is to make the instability visible and
reproducible.

On the first small repo-owned probe, `without-abc` passed 15/15 while
the original `with-abc` path failed 14/15, not from a crash but from
ABC's `Warning: The network is combinational` line. Yosys's own `help abc`
text explains why that can happen even on sequential modules: ABC is
run on logic snippets extracted from the design, not necessarily on the
whole module as one sequential network.

The repo-owned harness now treats `with-abc` as the explicit
warning-clean script:

`synth -noabc; abc -fast; opt -fast; stat; check`

That keeps ABC in the loop while avoiding the default `scorr`-based ABC
script that was producing the non-actionable warning bucket. The
follow-up small `--yosys-mode both` probe is now clean in both
sub-modes: `without-abc = 15/15 pass`, `with-abc = 15/15 pass`.
