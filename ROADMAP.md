# Roadmap

`anvil` grows in phases. Each phase delivers a working generator with a
larger expressive subset. No phase should land without end-to-end tests
and at least one `.sv` artifact run through Yosys or Verilator as a
synthesizability smoke check. Those sweeps are validation evidence, not
the product target: Verilator and Yosys check that generated HDL is
syntax/elaboration/synthesis acceptable, while the intended downstream
consumers are the broader class of tools that accept synthesizable HDL.

That quality bar coexists with breadth. `anvil` is meant to grow into a
signoff-grade random synthesizable RTL generator whose legal, unusual,
feature-rich designs can help expose bugs in parsers, elaborators, RTL
compilers, linters, simulators, synthesizers, and similar downstream
consumers, not by relying on malformed input or low-quality noise.

Whole-module intended functionality is not a roadmap goal. The roadmap
optimizes for structurally rich, legitimate, synthesizable RTL that
tools can ingest; local motifs may be functionally correct blocks, but
the top-level module usually has no meaningful specification.

## Broader artifact-family mandate (2026-04-20)

The user broadened the project direction explicitly: today's
"leaf-module typed circuit generator" is the starting point, not the
destination. ANVIL should grow into the go-to tool for pseudo-random
HDL artifact generation more broadly, which means the roadmap now needs
to cover more than one output family.

The important constraint is **separation of lanes, not dilution of the
current one**:

- the current signoff-grade synthesizable RTL lane remains
  valid-by-construction and tool-clean by default;
- future oracle-backed micro-design corpora are new artifact families,
  not a weakening of the existing lane; and
- broader source-level frontend/elaboration artifacts must also remain
  valid-by-construction and synthesizable, not a license to blur
  invalid files into the project scope.

The first explicitly-requested families beyond the current lane are:

- an **oracle-backed micro-design mode** for small self-contained `.sv`
  files with known expected facts;
- a **source-level parameter / hierarchy / package IR** for compact
  elaboration- and frontend-oriented artifacts;
- **explicit expected-facts manifests** for those corpora; and
- additional **valid-by-construction synthesizable artifact families**
  rather than one single leaf-module output style.

## Four steering gaps from the codebase suitability assessment (2026-04-20)

The current codebase is suited to the product goal as a **foundation**,
but these four gaps must stay explicit. They are already spread across
the phased plan below; this section makes them durable as a steering map
instead of leaving them implicit.

1. **Feature breadth / legal design-space width**
   The current lane is grounded in the Phase 1/2/3 leaf-module kernel
   and now has a live Phase 4 hierarchy planner above it. Reaching
   "complex to very complex synthesizable RTL" still requires broader
   hierarchy composition plus Phases 5, 5b, and 6 to land as real
   generator surfaces: parameterization, packed aggregates, memories,
   FSMs, and other legal interaction-heavy motifs. Beyond that, ANVIL
   also needs broader artifact families: oracle-backed micro-designs,
   frontend/elaboration accept corpora, and other valid-by-construction
   synthesizable artifact families. Every new category, knob, and
   artifact family must be exercised in generation paths, tests,
   metrics, and downstream tool sweeps; dead knobs or paper-only
   categories are regressions.

2. **`NodeId` as identity / full-factorization mode**
   The strong-form target is: under `identity_mode = node-id`,
   equivalent expressions anywhere in any output cone or flop-D cone
   should converge to one `NodeId`, so sharing of gates, blocks,
   modules, and flops is as high as the current build knows how to
   prove. The doctrinal bar is stronger than syntactic resemblance:
   two cones should share one identity only when ANVIL can prove they
   implement the same functionality with respect to the same canonical
   leaf endpoints. Today's implementation covers normalized
   combinational identity plus a live bounded semantic fragment at the
   `e-graph` rung for small-support gate cones, together with a
   conservative post-drain state merge over the same proof discipline;
   stronger sequential and hierarchical equivalence are still open
   work. This mode must remain user-controllable from the CLI:
   `--identity-mode relaxed` is the real semantic off-switch.
   Within `node-id`, `--factorization-level` remains an
   implementation/proof-depth and stress-coverage dial while the build
   climbs toward the doctrine; it must not be treated as redefining what
   `node-id` means.

3. **Signoff-quality downstream-acceptance industrialization**
   Seed-level cleanliness is not enough. The project needs automated
   Verilator/Yosys validation evidence across seeds, construction
   strategies, identity modes, factorization levels, category mixes,
   flop/no-flop cases, and deeper hierarchy/memory/FSM features.
   Counterexamples must be retained with exact seed+config evidence and
   fed back into IR invariants or rewrites, not hidden behind warning
   suppressions. The intended steady-state remains: generated RTL is
   boringly acceptable to mainstream downstream HDL consumers by default.

   The adversarial space must be modeled as an explicit axis matrix, not
   as one vague notion of "randomness". Construction strategy, identity
   mode, factorization level, motif/category selection, sequential
   density, width/depth ranges, and the probability knobs must be
   exercised without hidden bias from whichever implementation path is
   currently easiest.

4. **Structure-first, not whole-module specification-first**
   ANVIL optimizes for structural legitimacy, synthesizability,
   complexity, factorization pressure, and downstream-tool ingestibility
   rather than intended top-level behavior. Features that create locally
   meaningful or functionally correct blocks are welcome, but ANVIL is
   not turning into a bundled oracle or spec-driven synthesis engine.
   Expected-facts manifests for specific artifact families are fine; a
   full shadow simulator remains out of scope.
   When choosing between slices, prefer new legal interaction surfaces
   and stronger by-construction invariants over post-hoc whole-module
   "meaningfulness" scoring.

## Phase 0 — Scaffolding (done)

- Cargo project, module skeleton, CLI entry point.
- Design docs (`book/`) capturing the core algorithm.
- `Module`, `Port`, `Net`, `Node`, `Gate`, `Flop` IR types defined.
- CLI accepts `--seed`, `--count`, `--out`, `--config`, `--dump-config`.

**Exit criteria (met locally):** `cargo build`, `cargo test`,
`cargo clippy -D warnings`, `cargo fmt --check` all clean. Reproducibility
test passes byte-identical output for the same seed.

## Phase 1 — Single-module MVP (done)

One module, no hierarchy, no inter-module sharing. Combinational *and*
sequential logic from the start — flops are part of the same fanin-cone
recursion (Q is a leaf, D opens a new sub-cone, worklist drains).

- Random N inputs, M outputs, random widths per port.
- Per-output fanin cone recursion with depth budget.
- Gate set: bitwise (`and`, `or`, `xor`, `not`), arithmetic
  (`+`, `-`, `==`, `<`), `mux`, `slice`, `concat`, constants.
- **Sequential discipline:** single `clk` (posedge) and single `rst_n`
  (async, active-low) shared by every flop in the module. One
  `always_ff @(posedge clk or negedge rst_n)` block per module.
- Width propagates top-down; dependency set propagates bottom-up.
- Non-triviality: every output and every flop-D cone has dep-set ≥ 1,
  enforced at cone root.
- Tree-shaped cones only (each internal signal has one consumer).
- SV emitter produces `module` + `assign` + `always_ff`.

**Exit criteria:** 1000 modules generated from random seeds, all parse
and elaborate in Verilator without error, all Yosys-synthesize to
non-empty netlists, both with and without flops. **Met locally.** The
repo-owned `tool_matrix` harness now has a completed current-code Phase
1 report at
`/tmp/anvil-tool-matrix-phase1-real-r21/tool_matrix_report.json`:

- `scenario_count = 15`
- `modules_per_scenario = 67`
- `total_modules = 1005`
- `coverage_gaps = []`
- `Verilator pass/fail = 1005/0`
- `Yosys without-abc pass/fail = 1005/0`
- `Yosys with-abc pass/fail = 1005/0`

That completed run exercises the full built-in adversarial matrix across
construction strategies, identity modes, factorization levels, and the
share-heavy / motif-heavy stress profiles. The harness treats warnings
as failures, so this is a real zero-warning closure, not a noisy green
run.

## Phase 2 — Signal sharing (DAG cones) (done)

- Signal pool of already-created internal wires.
- Per-operand `share_prob` decision: recurse (tree) or reuse (DAG).
  Mixing is the default — a single gate's operands can freely combine
  shared and freshly-built sub-cones.
- Under `identity_mode = node-id` with effective factorization level
  `>= cse`, a conservative post-drain flop merge now extends sharing to
  state elements too: flops collapse when ANVIL can prove their D-cones
  implement the same currently-normalized functionality over the same
  canonical leaf endpoints, together with the same `width` and reset
  semantics. At effective level `e-graph`, a bounded semantic
  post-construction gate merge is also live for small-support
  combinational cones over the same canonical leaf variables.
- Dep-set propagation correctly handles shared fanout.
- Fanout stress: a single wire can drive many consumers.
- Anti-collapse rules still apply post-share (no `x ^ x` even when both
  operands come from pool reuse).

**Exit criteria (met locally):** generator produces cones with
controlled sharing factor; synthesis still succeeds; no multi-driver
violations; Verilator lint passes on a representative seed sweep with
`share_prob` ∈ {0.0, 0.3, 0.9}. The repo-owned `tool_matrix` harness
now has a completed current-code Phase 2 sharing report at
`/tmp/anvil-tool-matrix-phase2-share-r1/tool_matrix_report.json`:

- `scenario_count = 18`
- `modules_per_scenario = 12`
- `total_modules = 216`
- `coverage_gaps = []`
- `Verilator pass/fail = 216/0`
- `Yosys without-abc pass/fail = 216/0`
- `Yosys with-abc pass/fail = 216/0`
- normalized sharing sweep:
  - `share_prob = 0.0`: `shared_node_fraction = 0.4122`,
    `avg_nodes/module = 4727.56`
  - `share_prob = 0.3`: `shared_node_fraction = 0.4232`,
    `avg_nodes/module = 3525.01`
  - `share_prob = 0.9`: `shared_node_fraction = 0.4386`,
    `avg_nodes/module = 2117.76`

The normalized metric matters: raw `total_shared_nodes` falls as
`share_prob` rises because stronger reuse collapses the graph. The gate
therefore proves controllability with `shared_node_fraction`
(`total_shared_nodes / total_nodes`) while also recording the expected
node-count collapse.

## Phase 3 — Structured combinational ops (done)

- `case`/`casez` expressions. **Both landed as structured
  combinational case-style blocks (`always_comb case` and
  `always_comb casez`).**
- Priority encoders, one-hot decoders. **Priority encoder landed
  (Rule 17).**
- Reduction operators (`&`, `|`, `^` unary). **Selectable gate
  category landed.**
- Shift by variable amount. **Landed.** `Shl` / `Shr` now have both
  surfaces: constant-amount shifts via the Rule 19 motif
  (`const_shift_amount_prob`) and variable-amount shifts via the
  ordinary recursive operand path when that coin misses.
- Generic `Slice` / `Concat` as selectable surfaces. **Landed.**
  They are no longer helper-only width-adapter / block-assembly shapes;
  the structured picker now emits real non-degenerate `Slice` and
  variadic `Concat` gates directly.
- `for`-loop unrolled logic (statically bounded). **Landed.** The leaf
  kernel now has a structured bounded `always_comb` for-loop fold over
  packed chunks via `for_fold_prob` and `GateOp::ForFold`.
- Linear-combination compound motif (`Σ sᵢ·cᵢ`, etc.) **landed.**

Phase 3 is now **done**. The previously explicit breadth gaps are
landed (`case`, `casez`, variable shifts, generic selectable `Slice` /
`Concat`, bounded unrolled logic), and the repo-owned closure evidence
now exists too via
`/tmp/anvil-tool-matrix-phase3-structured-r4/tool_matrix_report.json`
(`21` scenarios, `10` modules/scenario, `210` total modules,
`coverage_gaps = []`, and `210/0` pass-fail in Verilator plus both
repo-owned Yosys modes).

**Exit criteria:** motif library covers common synthesizable idioms and
the structured surface has its own representative clean-run closure
evidence.

## Phase 4 — Hierarchy (in progress)

**Active task tree:** [`HIERARCHY-AWARE-IDENTITY`](docs/tasks/HIERARCHY-AWARE-IDENTITY.md) (current frontier: `HIERARCHY-AWARE-IDENTITY.1`). Phase 4's `rN`-named linear coverage slices (r73…r85) continue to land under the rN cadence; multi-slice sub-objectives like hierarchy-aware identity are now task-tree-managed per [docs/TASK_TREE.md](docs/TASK_TREE.md).

- **Landed slices so far:**
  - the legacy exact wrapper lane:
    `--hierarchy-depth 1 --num-leaf-modules N [--num-child-instances M]`
    generates a real `Design`: a pre-generated library of leaf modules
    plus a real top wrapper that instantiates them and builds a first
    parent-side output layer over child instance outputs. That layer is
    combinational by default and can now become locally stateful when
    `--hierarchy-parent-flop-prob` is explicitly requested.
    `M = 0` preserves the legacy exact-once behavior, `M < N`
    under-instantiates the library, and `M > N` reuses child
    definitions.
  - the newer bounded recursive lane:
    `--min-hierarchy-depth A --max-hierarchy-depth B
    --min-child-instances-per-module C
    --max-child-instances-per-module D`
    now builds a real recursive hierarchy tree. The current planner
    keeps every leaf depth inside `[A:B]`, can now mix shallow and deep
    branches inside one tree when the requested interval is open and
    the structure allows it, keeps each non-leaf module's child count
    inside `[C:D]`, and reports the realized tree shape numerically in
    `DesignMetrics`.
    Repeated `--child-instances-per-depth DEPTH=MIN:MAX` overrides are
    now also live and take priority over the fallback branching range at
    the matching parent depth.
  - the explicit child-sourcing axis:
    `--hierarchy-child-source-mode <library|on-demand>` now applies to
    both the wrapper and recursive hierarchy lanes. `library` keeps the
    reusable child-definition pool live; the currently-landed
    `on-demand` slice synthesizes children against parent-planned exact
    data-interface profiles for each planned instance slot.
  - the current parent-side routing surface:
    parent output cones are now built over the full parent source pool
    and post-finalized so they can mix parent data inputs with child
    instance outputs while every output still retains child-output
    support. `DesignMetrics` report this via
    `top_parent_port_composed_outputs`,
    `hierarchy_parent_port_composed_outputs`, and their fractions.
    both hierarchy lanes now expose
    `--hierarchy-sibling-route-prob <p>`, which lets later child data
    inputs bind from earlier sibling instance outputs while staying
    acyclic by construction. This routing is intentionally
    combinational-only in the current slice. The same slice now also
    exposes `--hierarchy-child-input-cone-prob <p>`, which lets child
    data inputs bind through parent-local combinational cones over
    already-available parent sources: parent data inputs, earlier
    sibling instance outputs, and earlier parent-side route gates.
    `--hierarchy-parent-flop-prob <p>` now separately controls whether
    those parent-side cones may emit local parent flops; default `0.0`
    preserves the combinational parent layer unless state is explicitly
    requested.
    `--hierarchy-registered-sibling-route-prob <p>` adds the first
    explicit registered child-to-child route: a later child input may
    bind from an earlier sibling output through one local parent flop.
    When `--hierarchy-parent-cone-instance-prob` also fires, that direct
    registered route can allocate a helper child and use the helper
    output as the parent-flop D source. The default-off
    `--hierarchy-registered-sibling-mixed-support-prob <p>` sub-route
    can mix a parent data-port companion into the direct registered
    sibling D path while keeping the route out of the registered
    parent-composed bucket.
    `--hierarchy-registered-child-input-cone-prob <p>` adds the
    registered parent-composed route: a later child input may bind
    through parent-local logic over already-available parent sources
    and then one local parent flop. Current HEAD can mix parent data
    ports with earlier sibling outputs in that registered route, and
    later registered parent-composed routes can chain through earlier
    parent-local flops when such Qs are already available. When
    `--hierarchy-parent-cone-instance-prob` also fires, the registered
    D cone can include a parent-cone helper instance output.
    `--hierarchy-parent-cone-instance-prob <p>` adds the first
    first-class module-instantiation route inside parent cone choice:
    parent-composed child-input cones, direct sibling routes, direct
    registered sibling routes, registered child-input D cones, and
    parent-output cones can instantiate one helper child as an internal
    parent-cone source, then route its output directly, through parent
    logic, or through a parent-local flop.
    Parent-output helper routing is now proven in both the direct
    combinational form and the stateful form where the helper output
    feeds parent-local state before reaching the parent output.
    `--max-parent-cone-instances-per-module <N>` controls the
    per-parent helper budget; default `1` preserves the first helper
    slice, and focused tests now prove budget `3` is reachable through
    both child-input helper routing and parent-output-only composition.
- Current slice constraints:
  - direct sibling routing is still combinational; the first one-flop
    registered sibling route and the first registered parent-composed
    child-input route are live, and the registered parent-composed
    route now has mixed parent-port / child-output support plus a first
    multi-stage parent-composed chaining subcase, and the direct
    registered sibling route can now chain through earlier parent-local
    Qs as a separate multi-stage child-to-child surface, and can now
    optionally mix parent-port support into the direct registered D path
    before the parent-local flop. The helper
    version can also seed one parent Q from a parent-cone helper and
    feed a later parent flop, and registered parent-composed helper
    routes can now reuse a helper-sourced parent Q in later
    parent-composed D logic. Parent-composed child-input helper routes
    can also register a helper output into parent-local state and feed
    that helper Q into unregistered parent-composed child-input logic.
    Broader
    registered hierarchy patterns remain future work
  - the latest full downstream-clean repo-owned Phase 4 matrix is banked at
    `/tmp/anvil-tool-matrix-phase4-hierarchy-r84/tool_matrix_report.json`.
    It covers both the wrapper lane and the representative recursive
    lane, including the mixed-depth recursive axis, the explicit
    child-sourcing axis, local parent state, registered sibling routing, direct registered sibling mixed-support
    routing, registered parent-composed child-input routing, registered
    mixed-support routing, multi-stage registered parent-composed
    routing, mixed parent-port / child-output parent outputs,
    parent-cone helper-instance child-input routing, parent-cone
    helper-instance parent-output composition, budgeted multi-helper
    allocation, registered parent-composed helper-sourced child-input D
    cones, direct sibling helper routing, direct registered sibling
    helper routing, multi-stage direct registered sibling helper
    routing, multi-stage registered parent-composed helper routing,
    and multi-stage registered sibling routing through earlier
    parent-local Qs, plus parent-output helper routing through
    parent-local flops, plus parent-composed helper child-input routing
    through parent-local flops, plus recursive non-top
    helper-through-parent-flop child-input routing, plus recursive
    non-top direct sibling helper routing, plus recursive non-top direct
    registered sibling helper routing, plus recursive non-top registered
    parent-composed helper routing, plus recursive non-top multi-stage
    direct registered sibling helper routing, plus recursive non-top
    multi-stage registered parent-composed helper routing, plus recursive
    non-top parent-output helper routing, plus recursive non-top
    stateful parent-output helper routing, plus recursive non-top
    parent-output multi-helper budget evidence, plus recursive non-top
    child-input multi-helper budget evidence, plus recursive non-top
    stateful multi-helper budget evidence, plus recursive non-top
    registered mixed-support child-input routing, plus recursive non-top
    multi-stage registered parent-composed child-input routing without
    helper instances, plus recursive non-top multi-stage registered
    sibling-routed child-input routing without helper instances, plus
    recursive non-top multi-stage registered mixed-support child-input
    routing without helper instances, plus recursive non-top registered
    parent-composed helper D-cone routing with mixed parent-port
    support, plus recursive non-top parent-output helper routing that
    mixes parent data-port support in the same helper-backed output
    cone, plus direct registered sibling mixed-support routing, plus
    recursive non-top direct registered sibling mixed-support routing, and recursive non-top unregistered parent-composed mixed-support child-input routing without helper instances, plus recursive non-top stateful parent-port-composed parent-output routing without helper instances, plus recursive non-top stateful unregistered parent-composed mixed-support child-input routing through parent-local Qs without helper instances, plus recursive non-top parent-local flops gated as a first-class coverage fact, plus recursive parent-local flops at exact hierarchy depth 3, plus recursive non-top unregistered parent-composed mixed-support child inputs at exact hierarchy depth 3 without helpers, plus recursive non-top parent-port-composed parent outputs at exact hierarchy depth 3 without helpers or state, plus recursive non-top stateful parent-port-composed parent outputs at exact hierarchy depth 3 without helpers, plus recursive non-top stateful parent-composed mixed-support child inputs at exact hierarchy depth 3 without helpers, plus recursive non-top parent-local flops at exact hierarchy depth 4, plus recursive non-top mixed-support child inputs at exact hierarchy depth 4 without helpers, plus recursive non-top parent-port-composed parent outputs at exact hierarchy depth 4 without helpers or state, plus recursive non-top stateful parent-port-composed parent outputs at exact hierarchy depth 4 without helpers, plus recursive non-top stateful parent-composed mixed-support child inputs at exact hierarchy depth 4 without helpers, plus recursive non-top parent-local flops at exact hierarchy depth 5, plus recursive non-top mixed-support child inputs at exact hierarchy depth 5 without helpers, plus recursive non-top parent-port-composed parent outputs at exact hierarchy depth 5 without helpers or state, plus recursive non-top stateful parent-port-composed parent outputs at exact hierarchy depth 5 without helpers, plus recursive non-top stateful unregistered parent-composed mixed-support child inputs at exact hierarchy depth 5 without helpers — closing the depth-5 sweep, plus recursive non-top parent-local flops at exact hierarchy depth 6 — opening the depth-6 axis, plus recursive non-top mixed-support child inputs at exact hierarchy depth 6 without helpers, plus recursive non-top parent-port-composed parent outputs at exact hierarchy depth 6 without helpers or state, plus recursive non-top stateful parent-port-composed parent outputs at exact hierarchy depth 6 without helpers, plus recursive non-top stateful unregistered parent-composed mixed-support child inputs at exact hierarchy depth 6 without helpers (2,2 calibrated) — closing the depth-6 sweep, plus recursive non-top parent-local flops at exact hierarchy depth 7 — opening the depth-7 axis, plus recursive non-top mixed-support child inputs at exact hierarchy depth 7 without helpers (2,2 calibrated), plus recursive non-top parent-port-composed parent outputs at exact hierarchy depth 7 without helpers or state, plus recursive non-top stateful parent-port-composed parent outputs at exact hierarchy depth 7 without helpers, plus recursive non-top stateful unregistered parent-composed mixed-support child inputs at exact hierarchy depth 7 without helpers (2,2 calibrated) — closing the depth-7 sweep, plus recursive non-top registered parent-composed child-input bindings that chain through at least three parent-local flop stages without helpers, plus a recursive non-top internal parent saturating a parent-cone helper budget of 5 helpers. The `r84`
    report records `201` scenarios, `4` designs/scenario, `804` total designs,
    `coverage_gaps = []`,
    `saw_recursive_hierarchy_parent_cone_instance_mixed_support_outputs = true`,
    `saw_recursive_hierarchy_registered_parent_cone_instance_mixed_support_routing = true`,
    `saw_recursive_hierarchy_registered_multistage_mixed_support_routing = true`,
    `saw_recursive_hierarchy_registered_multistage_sibling_routing = true`,
    `saw_recursive_hierarchy_registered_multistage_routing = true`,
    `saw_recursive_hierarchy_registered_mixed_support_routing = true`,
    `saw_recursive_multiple_parent_cone_instances_per_parent = true`,
    `saw_recursive_multiple_parent_cone_instances_per_parent_child_inputs = true`,
    `saw_recursive_multiple_parent_cone_instances_per_parent_through_flops = true`,
    `saw_recursive_hierarchy_parent_cone_instance_flop_outputs = true`,
    `saw_recursive_hierarchy_parent_cone_instance_outputs = true`,
    `saw_recursive_hierarchy_direct_sibling_parent_cone_instance_routing = true`,
    `saw_recursive_hierarchy_direct_registered_sibling_parent_cone_instance_routing = true`,
    `saw_hierarchy_registered_sibling_mixed_support_routing = true`,
    `saw_recursive_hierarchy_registered_sibling_mixed_support_routing = true`,
    `saw_hierarchy_mixed_support_child_inputs = true`,
    `saw_recursive_hierarchy_mixed_support_child_inputs = true`,
    `saw_recursive_hierarchy_parent_port_composed_outputs = true`,
    `saw_recursive_hierarchy_stateful_parent_port_composed_outputs = true`,
    `saw_recursive_hierarchy_registered_multistage_parent_cone_instance_routing = true`,
    `saw_recursive_hierarchy_registered_multistage_parent_composed_parent_cone_instance_routing = true`,
    `saw_recursive_hierarchy_registered_parent_composed_parent_cone_instance_routing = true`,
    `saw_recursive_hierarchy_parent_composed_parent_cone_instance_flop_routing = true`,
    `saw_hierarchy_parent_cone_instance_flop_mixed_support_outputs = true`,
    `saw_recursive_hierarchy_parent_cone_instance_flop_mixed_support_outputs = true`,
    `saw_hierarchy_parent_cone_instance_mixed_support_routing = true`,
    `saw_recursive_hierarchy_parent_cone_instance_mixed_support_routing = true`,
    `saw_hierarchy_parent_composed_parent_cone_instance_flop_mixed_support_routing = true`,
    `saw_recursive_hierarchy_parent_composed_parent_cone_instance_flop_mixed_support_routing = true`,
    `saw_recursive_hierarchy_stateful_parent_port_composed_outputs = true`,
    `saw_recursive_hierarchy_stateful_parent_composed_mixed_support_child_inputs = true`,
    `saw_recursive_hierarchy_parent_local_flops = true`,
    `saw_recursive_hierarchy_depth_3_parent_local_flops = true`,
    `saw_recursive_hierarchy_depth_3_mixed_support_child_inputs = true`,
    `saw_recursive_hierarchy_depth_3_parent_port_composed_outputs = true`,
    `saw_recursive_hierarchy_depth_3_stateful_parent_port_composed_outputs = true`,
    `saw_recursive_hierarchy_depth_3_stateful_parent_composed_mixed_support_child_inputs = true`,
    `saw_recursive_hierarchy_depth_4_parent_local_flops = true`,
    `saw_recursive_hierarchy_depth_4_mixed_support_child_inputs = true`,
    `saw_recursive_hierarchy_depth_4_parent_port_composed_outputs = true`,
    `saw_recursive_hierarchy_depth_4_stateful_parent_port_composed_outputs = true`,
    `saw_recursive_hierarchy_depth_4_stateful_parent_composed_mixed_support_child_inputs = true`,
    `saw_recursive_hierarchy_depth_5_parent_local_flops = true`,
    `saw_recursive_hierarchy_depth_5_mixed_support_child_inputs = true`,
    `saw_recursive_hierarchy_depth_5_parent_port_composed_outputs = true`,
    `saw_recursive_hierarchy_depth_5_stateful_parent_port_composed_outputs = true`,
    `saw_recursive_hierarchy_depth_5_stateful_parent_composed_mixed_support_child_inputs = true`,
    `saw_recursive_hierarchy_depth_6_parent_local_flops = true`,
    `saw_recursive_hierarchy_depth_6_mixed_support_child_inputs = true`,
    `saw_recursive_hierarchy_depth_6_parent_port_composed_outputs = true`,
    `saw_recursive_hierarchy_depth_6_stateful_parent_port_composed_outputs = true`,
    `saw_recursive_hierarchy_depth_6_stateful_parent_composed_mixed_support_child_inputs = true`,
    `saw_recursive_hierarchy_depth_7_parent_local_flops = true`,
    `saw_recursive_hierarchy_depth_7_mixed_support_child_inputs = true`,
    `saw_recursive_hierarchy_depth_7_parent_port_composed_outputs = true`,
    `saw_recursive_hierarchy_depth_7_stateful_parent_port_composed_outputs = true`,
    `saw_recursive_hierarchy_depth_7_stateful_parent_composed_mixed_support_child_inputs = true`,
    `saw_recursive_hierarchy_three_stage_registered_parent_composed_chain = true`,
    `saw_recursive_parent_cone_helper_budget_5 = true`,
    and `804/0` pass-fail in Verilator plus both repo-owned Yosys modes.
  - module names are now allocated from one generator-global sequence
    across leaf modules, recursive parent modules, and repeated
    hierarchical designs in one output run, so multi-file hierarchy
    output cannot collide or overwrite module definition files by
    reusing a name
- Open Phase 4 work:
  - broaden helper-instance placement beyond the current
    parent-composed child-input, direct sibling, direct registered
    sibling, registered child-input, budgeted parent-output helper,
    stateful parent-output helper, recursive non-top parent-output
    helper, recursive non-top stateful parent-output helper, recursive
    non-top parent-output multi-helper budget, recursive non-top
    child-input multi-helper budget, recursive non-top stateful
    multi-helper budget, multi-stage direct registered helper, and
    multi-stage registered parent-composed helper slices
  - deeper parent-side routing/composition beyond the current mixed
    parent-output, combinational sibling-binding, and parent-input-cone
    surfaces
  - richer registered child-to-child and parent-composed routing using
    the landed local parent-state surface
  - hierarchical identity as future required work: under
    `identity_mode = node-id`, equivalent instantiated structures
    should eventually participate in the same sharing story instead of
    creating a second identity system beside gates/flops

**Repo-owned Phase 4 hierarchy closure (latest full bank met locally):** the refreshed
hierarchy gate now exists at
`/tmp/anvil-tool-matrix-phase4-hierarchy-r84/tool_matrix_report.json`
with multi-file output, correct top declaration, design-level
validation, representative wrapper and recursive profiles,
`201` scenarios, `804` total designs, `coverage_gaps = []`, and clean Verilator + Yosys
elaboration/synthesis on the broadened hierarchy matrix
(`804/0` in Verilator plus both repo-owned Yosys modes). The `r84` report
proves all of the current representative hierarchy axes directly:
- wrapper exact / reuse / under-instantiation profiles
- recursive depth `2`
- mixed recursive depth range `2:3`
- explicit child-sourcing modes `library` and `on-demand`
- child-instance profiles `2`, `4`, `2:3`, and `1:3`
- registered sibling-routed hierarchy child inputs through parent-local
  state
- direct registered sibling mixed-support hierarchy child inputs that
  mix parent-port support into the direct registered D path without
  registered parent-composed classification
- recursive non-top direct registered sibling mixed-support hierarchy
  child inputs below the top parent, proved by hierarchy-wide counters
  exceeding top-only counters
- multi-stage registered sibling-routed hierarchy child inputs that
  chain through earlier parent-local Qs without parent-composed logic
- multi-stage direct registered sibling helper routes where a helper
  output seeds one parent Q and a later route reuses that Q
- recursive non-top multi-stage direct registered sibling helper routes
  where a helper output seeds one non-top parent Q and a later non-top
  route reuses that Q
- recursive non-top multi-stage registered parent-composed helper routes
  where a helper output seeds one non-top parent Q and later non-top
  parent-composed D logic reuses that Q
- recursive non-top direct registered sibling helper routes where a
  helper output feeds a direct registered sibling D path below the top
  parent
- recursive non-top registered parent-composed helper routes where a
  helper output feeds registered parent-composed D logic below the top
  parent
- recursive non-top registered parent-composed helper routes where that
  helper-sourced D cone also mixes parent data-port support below the
  top parent
- registered parent-composed hierarchy child-input bindings through
  parent logic plus parent-local state
- registered mixed-support child-input bindings that mix parent data
  ports with child outputs
- recursive non-top registered mixed-support child-input bindings that
  mix parent data ports with child outputs below the top parent
- multi-stage registered parent-composed child-input bindings that chain
  through earlier parent-local Qs
- recursive non-top multi-stage registered parent-composed child-input
  bindings that chain through earlier parent-local Qs below the top
  parent without helper instances
- recursive non-top multi-stage registered sibling-routed child-input
  bindings that chain through earlier parent-local Qs below the top
  parent without helper instances or parent-composed D logic
- recursive non-top multi-stage registered mixed-support child-input
  bindings that combine parent ports, child outputs, and earlier
  parent-local Qs below the top parent without helper instances
- multi-stage registered parent-composed helper routes where a helper
  output seeds one parent Q and later parent-composed D logic reuses
  that Q
- parent-cone helper instances that source parent-composed child-input
  bindings
- parent-cone helper instances that source parent outputs
- recursive non-top parent-cone helper instances that source parent
  outputs while also mixing parent data-port support below the top
  parent
- stateful parent-output helper routes that also mix parent-port support
- unregistered parent-composed helper child-input routes that also mix
  parent-port support
- stateful helper-through-flop child-input routes that also mix
  parent-port support
- parent-cone helper instances that source parent outputs through
  parent-local flops
- parent-cone helper instances that source parent-composed child-input
  logic through parent-local flops
- recursive non-top parent-cone helper instances that source
  parent-composed child-input logic through parent-local flops below the
  top parent
- recursive non-top parent-cone helper instances that source direct
  sibling-routed child-input bindings below the top parent
- parent-cone helper instances that source registered parent-composed
  child-input D cones
- parent-cone helper instances that source direct sibling routes
- parent-cone helper instances that source direct registered sibling
  route D inputs
- budgeted multi-helper allocation in one hierarchy parent
- recursive non-top parent-output multi-helper budget evidence
- recursive non-top child-input multi-helper budget evidence
- recursive non-top stateful multi-helper budget evidence
- per-depth override profile `0=4:4,1=2:2`
- real recursive design emission
- real mixed shallow/deep recursive realization
- real per-depth branching metrics
- real parent-side composition above instance outputs
- real mixed parent-port / child-output parent outputs
- real sibling-routed hierarchy child inputs
- real parent-composed hierarchy child-input bindings
- real local parent flops in hierarchy modules
- real structural proof that on-demand child sourcing emitted fresh
  child definitions per planned instance slot
- real exact profiled child-interface synthesis in the on-demand lane

Earlier current-code coverage-only Phase 4 matrix probes at
`/tmp/anvil-tool-matrix-phase4-parent-port-coverage-r1/tool_matrix_report.json`,
`/tmp/anvil-tool-matrix-phase4-registered-mixed-r1/tool_matrix_report.json`,
and
`/tmp/anvil-tool-matrix-phase4-registered-multistage-r1/tool_matrix_report.json`,
and
`/tmp/anvil-tool-matrix-phase4-parent-cone-instance-r1/tool_matrix_report.json`
and
`/tmp/anvil-tool-matrix-phase4-parent-output-helper-state-r3/tool_matrix_report.json`
remain useful targeted policy breadcrumbs for the mixed parent-output,
registered mixed-support, multi-stage registered, and parent-cone
helper-instance slices plus the stateful parent-output helper slice.
They were run with Verilator/Yosys skipped; the full downstream-clean
`r30` bank
above now carries those same coverage facts with real tool validation.

**Focused parent-output helper-instance proof (new targeted evidence):**
current HEAD also lets parent-output cones instantiate a helper child as
an internal parent-cone source independently of child-input cone
bindings. The focused regression is
`cargo test hierarchy_parent_outputs_can_depend_on_helper_instance_outputs`;
the design metrics prove the route numerically via
`top_parent_cone_instances`,
`top_outputs_reaching_parent_cone_instances`,
`hierarchy_outputs_reaching_parent_cone_instances`, and
`top_parent_cone_instance_output_fraction`. The repo-owned Phase 4
scenario set includes a dedicated
`phase4_hier2_inst4_parent_output_cone_instance` axis, now banked in
`r30`.

**Focused recursive non-top parent-output helper proof (new targeted evidence):**
current HEAD now proves parent-output helper routing below the top
parent in an exact-depth-2 recursive hierarchy. The focused proof is
`cargo test recursive_hierarchy_parent_outputs_can_depend_on_helper_instances_below_top`;
it requires `realized_min_leaf_depth = realized_max_leaf_depth = 2`,
`hierarchy_parent_cone_instances > top_parent_cone_instances`,
`hierarchy_outputs_reaching_parent_cone_instances >
top_outputs_reaching_parent_cone_instances`,
`child_input_bindings_from_parent_cone_instances = 0`, and
`hierarchy_outputs_reaching_parent_cone_instances_through_parent_flops = 0`.
The live Phase 4 matrix policy includes the dedicated
`phase4_recur_d2_parent_output_cone_instance` axis; the full
downstream-clean `r42` report proves it with `coverage_gaps = []` and
`348/0` pass-fail in Verilator plus both repo-owned Yosys modes.

**Focused recursive non-top parent-output helper mixed-support proof (new targeted evidence):**
current HEAD now proves that recursive non-top parent-output helper
cones can also mix parent data-port support in the same helper-backed
output cone. The focused proof is
`cargo test recursive_hierarchy_parent_outputs_mix_helper_instances_with_parent_ports_below_top`;
it requires `realized_min_leaf_depth = realized_max_leaf_depth = 2`,
`hierarchy_parent_cone_instances > top_parent_cone_instances`,
`hierarchy_outputs_reaching_parent_cone_instances >
top_outputs_reaching_parent_cone_instances`,
`hierarchy_outputs_reaching_parent_cone_instance_mixed_support >
top_outputs_reaching_parent_cone_instance_mixed_support`,
`child_input_bindings_from_parent_cone_instances = 0`,
`child_input_bindings_from_registered_parent_cone_instances = 0`, and
`hierarchy_outputs_reaching_parent_cone_instances_through_parent_flops = 0`.
The live Phase 4 matrix policy now requires
`saw_recursive_hierarchy_parent_cone_instance_mixed_support_outputs`;
the full downstream-clean `r49` report first proved it, `r50` banked the accumulated mixed-support surface, and `r51` through `r84` carry it forward with
`coverage_gaps = []` and `804/0` pass-fail in Verilator plus both
repo-owned Yosys modes.

**Focused recursive non-top parent-output helper budget proof (new targeted evidence):**
current HEAD now proves that the same recursive non-top parent-output
helper route can spend a multi-helper local budget below the top parent.
The focused proof is
`cargo test recursive_hierarchy_parent_outputs_can_spend_helper_budget_below_top`;
it requires `realized_min_leaf_depth = realized_max_leaf_depth = 2`,
`max_parent_cone_instances_per_internal_module = 3`,
`top_parent_cone_instances = 3`,
`hierarchy_parent_cone_instances > top_parent_cone_instances`,
`hierarchy_outputs_reaching_parent_cone_instances >
top_outputs_reaching_parent_cone_instances`,
`child_input_bindings_from_parent_cone_instances = 0`, and
`child_input_bindings_from_registered_parent_cone_instances = 0`. The
full downstream-clean `r42` report proves this policy fact with
`saw_recursive_multiple_parent_cone_instances_per_parent = true`,
`coverage_gaps = []`, and `348/0` pass-fail in Verilator plus both
repo-owned Yosys modes.

**Focused recursive non-top child-input helper budget proof (new targeted evidence):**
current HEAD now proves that parent-composed child-input helper routes
can spend a multi-helper local budget below the top parent too. The
focused proof is
`cargo test recursive_hierarchy_parent_cone_helper_budget_allows_multiple_helpers_below_top`;
it requires `realized_min_leaf_depth = realized_max_leaf_depth = 2`,
`max_parent_cone_instances_per_internal_module = 3`,
`top_parent_cone_instances = 3`,
`hierarchy_parent_cone_instances > top_parent_cone_instances`,
`child_input_bindings_from_parent_composed_logic >
top_child_input_bindings_from_parent_composed_logic`,
`child_input_bindings_from_parent_cone_instances >
top_child_input_bindings_from_parent_cone_instances`,
`child_input_bindings_from_parent_cone_instances_through_parent_flops = 0`,
and `child_input_bindings_from_registered_parent_cone_instances = 0`.
The full downstream-clean `r45` report carries this policy fact with
`saw_recursive_multiple_parent_cone_instances_per_parent_child_inputs = true`,
`coverage_gaps = []`, and `384/0` pass-fail in Verilator plus both
repo-owned Yosys modes.

**Focused recursive non-top registered mixed-support proof (new targeted evidence):**
current HEAD now proves that registered parent-composed child-input
routes can mix parent data ports with child outputs below the top
parent without helper instances. The focused proof is
`cargo test recursive_hierarchy_registered_mixed_support_routes_below_top`;
it requires `realized_min_leaf_depth = realized_max_leaf_depth = 2`,
non-top parent-local flops,
`child_input_bindings_from_registered_parent_composed_logic >
top_child_input_bindings_from_registered_parent_composed_logic`,
`child_input_bindings_from_registered_instance_outputs >
top_child_input_bindings_from_registered_instance_outputs`,
`child_input_bindings_from_registered_mixed_support >
top_child_input_bindings_from_registered_mixed_support`, and
`child_input_bindings_from_registered_parent_cone_instances = 0`.
The full downstream-clean `r45` report proves this policy fact with
`saw_recursive_hierarchy_registered_mixed_support_routing = true`,
`coverage_gaps = []`, and `384/0` pass-fail in Verilator plus both
repo-owned Yosys modes.

**Focused recursive non-top registered multistage no-helper proof (new targeted evidence):**
current HEAD now proves that registered parent-composed child-input
routes can chain through earlier parent-local Qs below the top parent
without helper instances. The focused proof is
`cargo test recursive_hierarchy_registered_parent_composed_routes_can_chain_without_helpers_below_top`;
it requires `realized_min_leaf_depth = realized_max_leaf_depth = 2`,
non-top parent-local flops,
`child_input_bindings_from_registered_parent_composed_logic >
top_child_input_bindings_from_registered_parent_composed_logic`,
`child_input_bindings_from_registered_multistage_parent_composed_logic >
top_child_input_bindings_from_registered_multistage_parent_composed_logic`,
`registered_multistage_parent_composed_child_input_binding_fraction > 0.0`,
and zero registered helper-chain counters. The full downstream-clean
`r45` report proves this policy fact with
`saw_recursive_hierarchy_registered_multistage_routing = true`,
`coverage_gaps = []`, and `384/0` pass-fail in Verilator plus both
repo-owned Yosys modes.

**Focused recursive non-top registered sibling multistage no-helper proof (new targeted evidence):**
current HEAD now proves that direct registered sibling-routed
child-input routes can chain through earlier parent-local Qs below the
top parent without helper instances or parent-composed D logic. The
focused proof is
`cargo test recursive_hierarchy_registered_sibling_routes_can_chain_without_helpers_below_top`;
it requires `realized_min_leaf_depth = realized_max_leaf_depth = 2`,
non-top parent-local flops,
`child_input_bindings_from_registered_instance_outputs >
top_child_input_bindings_from_registered_instance_outputs`,
`child_input_bindings_from_registered_multistage_instance_outputs >
top_child_input_bindings_from_registered_multistage_instance_outputs`,
`registered_multistage_instance_output_child_input_binding_fraction > 0.0`,
and zero registered parent-composed and registered helper-chain
counters. The full downstream-clean `r46` report proves this policy
fact with
`saw_recursive_hierarchy_registered_multistage_sibling_routing = true`,
`coverage_gaps = []`, and `396/0` pass-fail in Verilator plus both
repo-owned Yosys modes.

**Focused recursive non-top registered multistage mixed-support no-helper proof (new targeted evidence):**
current HEAD now proves that registered parent-composed child-input
routes can simultaneously mix parent ports, child outputs, and earlier
parent-local Qs below the top parent without helper instances. The
focused proof is
`cargo test recursive_hierarchy_registered_multistage_mixed_support_routes_below_top`;
it requires `realized_min_leaf_depth = realized_max_leaf_depth = 2`,
non-top parent-local flops,
`child_input_bindings_from_registered_parent_composed_logic >
top_child_input_bindings_from_registered_parent_composed_logic`,
`child_input_bindings_from_registered_mixed_support >
top_child_input_bindings_from_registered_mixed_support`,
`child_input_bindings_from_registered_multistage_parent_composed_logic >
top_child_input_bindings_from_registered_multistage_parent_composed_logic`,
`child_input_bindings_from_registered_multistage_mixed_support >
top_child_input_bindings_from_registered_multistage_mixed_support`,
`registered_multistage_mixed_support_child_input_binding_fraction > 0.0`,
and zero registered helper-chain counters. The full downstream-clean
`r47` report proves this policy fact with
`saw_recursive_hierarchy_registered_multistage_mixed_support_routing = true`,
`coverage_gaps = []`, and `396/0` pass-fail in Verilator plus both
repo-owned Yosys modes.

**Focused stateful parent-output helper proof (new targeted evidence):**
current HEAD also lets the parent-output helper source pass through a
parent-local flop before reaching the parent output. The focused
regression is
`cargo test hierarchy_parent_outputs_can_route_helper_instances_through_parent_flops`;
the design metrics prove the route through
`top_outputs_reaching_parent_cone_instances_through_parent_flops`,
`hierarchy_outputs_reaching_parent_cone_instances_through_parent_flops`,
`top_parent_cone_instance_flop_output_fraction`, and
`hierarchy_parent_cone_instance_flop_output_fraction`, while keeping
child-input helper bindings at zero for the output-only proof. The
repo-owned Phase 4 scenario set includes a dedicated
`phase4_hier2_inst4_parent_output_cone_instance_state` axis, banked in
`r30`.

**Focused recursive non-top stateful parent-output helper proof (new targeted evidence):**
current HEAD now proves the same stateful parent-output helper route
below the top parent in an exact-depth-2 recursive hierarchy. The
focused regression is
`cargo test recursive_hierarchy_parent_outputs_can_route_helper_instances_through_parent_flops_below_top`;
it requires `realized_min_leaf_depth = realized_max_leaf_depth = 2`,
`hierarchy_parent_cone_instances > top_parent_cone_instances`,
`hierarchy_parent_local_flops > top_local_flops`,
`hierarchy_outputs_reaching_parent_cone_instances_through_parent_flops >
top_outputs_reaching_parent_cone_instances_through_parent_flops`,
`child_input_bindings_from_parent_cone_instances = 0`, and
`child_input_bindings_from_registered_parent_cone_instances = 0`. The
live Phase 4 matrix policy includes the dedicated
`phase4_recur_d2_parent_output_cone_instance_state` axis; the full
downstream-clean `r42` report proves it with `coverage_gaps = []` and
`348/0` pass-fail in Verilator plus both repo-owned Yosys modes.

**Focused recursive non-top stateful parent-output helper budget proof (new targeted evidence):**
current HEAD now proves that the stateful recursive non-top
parent-output helper route can also spend a multi-helper local budget
below the top parent. The focused proof is
`cargo test recursive_hierarchy_parent_outputs_can_spend_stateful_helper_budget_below_top`;
it requires `realized_min_leaf_depth = realized_max_leaf_depth = 2`,
`max_parent_cone_instances_per_internal_module = 3`,
`top_parent_cone_instances = 3`,
`hierarchy_parent_cone_instances > top_parent_cone_instances`,
`hierarchy_parent_local_flops > top_local_flops`,
`hierarchy_outputs_reaching_parent_cone_instances_through_parent_flops >
top_outputs_reaching_parent_cone_instances_through_parent_flops`,
`child_input_bindings_from_parent_cone_instances = 0`,
`child_input_bindings_from_parent_cone_instances_through_parent_flops = 0`,
and `child_input_bindings_from_registered_parent_cone_instances = 0`.
The full downstream-clean `r42` report proves this policy fact with
`saw_recursive_multiple_parent_cone_instances_per_parent_through_flops = true`,
`coverage_gaps = []`, and `348/0` pass-fail in Verilator plus both
repo-owned Yosys modes.

**Focused budgeted helper-instance proof (new targeted evidence):**
current HEAD lets one hierarchy parent instantiate more than one
parent-cone helper child when
`max_parent_cone_instances_per_module` is raised. The focused
regression is
`cargo test hierarchy_parent_cone_helper_budget_allows_multiple_helpers`;
the design metrics prove budget `3` via `top_parent_cone_instances`
and `max_parent_cone_instances_per_internal_module`. The repo-owned
Phase 4 scenario set includes a dedicated
`phase4_hier2_inst4_parent_cone_instance_budget3` axis too. Together
with the parent-output helper axis and the registered helper axis below,
this is now banked in the full downstream-clean `r30` report at
63 scenarios / 252 designs.

**Focused budgeted parent-output helper proof (new targeted evidence):**
current HEAD also proves that parent-output composition can spend that
same helper budget without relying on child-input helper bindings. The
focused regression is
`cargo test hierarchy_parent_outputs_can_spend_helper_budget`; the
design metrics prove budget `3` through `top_parent_cone_instances`
and `max_parent_cone_instances_per_internal_module`, require
`child_input_bindings_from_parent_cone_instances = 0`, and require
parent outputs to reach helper outputs.

**Focused registered helper-instance proof (new targeted evidence):**
current HEAD now lets registered parent-composed child-input D cones
instantiate and use parent-cone helper outputs when both
`hierarchy_registered_child_input_cone_prob` and
`hierarchy_parent_cone_instance_prob` are active. The focused
regressions are
`cargo test hierarchy_registered_child_input_cones_can_use_helper_instances`
and
`cargo test design_metrics_capture_registered_parent_cone_instance_routes`;
the design metrics prove the route numerically through
`child_input_bindings_from_registered_parent_cone_instances`,
`top_child_input_bindings_from_registered_parent_cone_instances`,
`registered_parent_cone_instance_child_input_binding_fraction`, and
`top_registered_parent_cone_instance_child_input_binding_fraction`.
The repo-owned Phase 4 scenario set includes a dedicated
`phase4_hier2_inst4_registered_parent_cone_instance_state` axis too,
now banked in `r30`.

**Focused multi-stage registered parent-composed helper proof (new targeted evidence):**
current HEAD now also lets a registered parent-composed helper route
seed a parent-local Q from a parent-cone helper output and lets later
registered parent-composed D logic reuse that Q. The focused
regressions are
`cargo test hierarchy_registered_parent_composed_routes_can_chain_helper_instances_through_parent_flops`
and
`cargo test design_metrics_capture_multistage_registered_parent_composed_parent_cone_instance_routes`;
the design metrics prove the route numerically through
`child_input_bindings_from_registered_multistage_parent_composed_parent_cone_instances`,
`top_child_input_bindings_from_registered_multistage_parent_composed_parent_cone_instances`,
`registered_multistage_parent_composed_parent_cone_instance_child_input_binding_fraction`,
and
`top_registered_multistage_parent_composed_parent_cone_instance_child_input_binding_fraction`,
while keeping the direct registered sibling helper multistage counter
at zero in the focused proof. The repo-owned Phase 4 scenario set
includes a dedicated
`phase4_hier2_inst4_registered_parent_cone_instance_multistage_state`
axis too, now banked in `r30`.

**Focused stateful parent-composed helper child-input proof (new targeted evidence):**
current HEAD now also lets parent-composed child-input helper routes
register a helper output into parent-local state and then feed that
helper Q into unregistered parent-composed child-input logic. The
focused regressions are
`cargo test hierarchy_parent_composed_helper_routes_can_use_parent_flops`
and
`cargo test design_metrics_capture_parent_composed_parent_cone_instance_flop_routes`;
the design metrics prove the route numerically through
`child_input_bindings_from_parent_cone_instances_through_parent_flops`,
`top_child_input_bindings_from_parent_cone_instances_through_parent_flops`,
`parent_cone_instance_flop_child_input_binding_fraction`, and
`top_parent_cone_instance_flop_child_input_binding_fraction`, while
keeping `child_input_bindings_from_registered_parent_cone_instances = 0`
in the focused proof. The repo-owned Phase 4 scenario set includes a
dedicated `phase4_hier2_inst4_parent_cone_instance_state` axis too,
now banked in `r30`.

**Focused recursive non-top stateful parent-composed helper proof (new targeted evidence):**
current HEAD now proves the same helper-through-parent-flop child-input
shape below the top parent in an exact-depth-2 recursive hierarchy. The
focused proof is
`cargo test recursive_hierarchy_parent_composed_helper_routes_can_use_parent_flops_below_top`;
it requires `realized_min_leaf_depth = realized_max_leaf_depth = 2`,
`hierarchy_parent_cone_instances > top_parent_cone_instances`,
`hierarchy_parent_local_flops > top_local_flops`, and
`child_input_bindings_from_parent_cone_instances_through_parent_flops >
top_child_input_bindings_from_parent_cone_instances_through_parent_flops`.
The live Phase 4 matrix policy includes the dedicated
`phase4_recur_d2_parent_cone_instance_state` axis; the full
downstream-clean `r41` report proves it with `coverage_gaps = []` and
`348/0` pass-fail in Verilator plus both repo-owned Yosys modes.

**Focused recursive non-top direct sibling helper proof (new targeted evidence):**
current HEAD now proves direct sibling helper routing below the top
parent in an exact-depth-2 recursive hierarchy. The focused proof is
`cargo test recursive_hierarchy_sibling_routes_can_use_helper_instances_below_top`;
it requires `realized_min_leaf_depth = realized_max_leaf_depth = 2`,
`hierarchy_parent_cone_instances > top_parent_cone_instances`,
`child_input_bindings_from_instance_outputs >
top_child_input_bindings_from_instance_outputs`,
`child_input_bindings_from_parent_cone_instances >
top_child_input_bindings_from_parent_cone_instances`, and both registered
helper counters to stay at zero. The live Phase 4 matrix policy includes
the dedicated `phase4_recur_d2_direct_sibling_parent_cone_instance`
axis; the full downstream-clean `r42` report proves it with
`coverage_gaps = []` and `348/0` pass-fail in Verilator plus both
repo-owned Yosys modes.

**Focused recursive non-top direct registered sibling helper proof (new targeted evidence):**
current HEAD now proves direct registered sibling helper routing below
the top parent in an exact-depth-2 recursive hierarchy. The focused
proof is
`cargo test recursive_hierarchy_registered_sibling_routes_can_use_helper_instances_below_top`;
it requires `realized_min_leaf_depth = realized_max_leaf_depth = 2`,
`hierarchy_parent_cone_instances > top_parent_cone_instances`,
`hierarchy_parent_local_flops > top_local_flops`,
`child_input_bindings_from_registered_instance_outputs >
top_child_input_bindings_from_registered_instance_outputs`,
`child_input_bindings_from_registered_parent_cone_instances >
top_child_input_bindings_from_registered_parent_cone_instances`, and
`child_input_bindings_from_registered_parent_composed_logic = 0`. The
live Phase 4 matrix policy includes the dedicated
`phase4_recur_d2_direct_registered_sibling_parent_cone_instance_state`
axis; the full downstream-clean `r42` report proves it with
`coverage_gaps = []` and `348/0` pass-fail in Verilator plus both
repo-owned Yosys modes.

**Focused recursive non-top multi-stage direct registered sibling helper proof (new targeted evidence):**
current HEAD now proves that direct registered sibling helper routing
can chain through helper-sourced parent-local Qs below the top parent in
an exact-depth-2 recursive hierarchy. The focused proof is
`cargo test recursive_hierarchy_registered_sibling_routes_can_chain_helper_instances_below_top`;
it requires `realized_min_leaf_depth = realized_max_leaf_depth = 2`,
`hierarchy_parent_cone_instances > top_parent_cone_instances`,
`hierarchy_parent_local_flops > top_local_flops`,
`child_input_bindings_from_registered_multistage_instance_outputs >
top_child_input_bindings_from_registered_multistage_instance_outputs`,
`child_input_bindings_from_registered_multistage_parent_cone_instances >
top_child_input_bindings_from_registered_multistage_parent_cone_instances`,
and both parent-composed registered counters to stay at zero. The live
Phase 4 matrix policy includes the dedicated
`phase4_recur_d2_registered_sibling_parent_cone_instance_multistage_state`
axis; the full downstream-clean `r42` report proves it with
`coverage_gaps = []` and `348/0` pass-fail in Verilator plus both
repo-owned Yosys modes.

**Focused recursive non-top multi-stage registered parent-composed helper proof (new targeted evidence):**
current HEAD now proves that registered parent-composed helper routing
can chain through helper-sourced parent-local Qs below the top parent in
an exact-depth-2 recursive hierarchy. The focused proof is
`cargo test recursive_hierarchy_registered_parent_composed_routes_can_chain_helper_instances_below_top`;
it requires `realized_min_leaf_depth = realized_max_leaf_depth = 2`,
`hierarchy_parent_cone_instances > top_parent_cone_instances`,
`hierarchy_parent_local_flops > top_local_flops`,
`child_input_bindings_from_registered_multistage_parent_composed_logic >
top_child_input_bindings_from_registered_multistage_parent_composed_logic`,
`child_input_bindings_from_registered_multistage_parent_composed_parent_cone_instances >
top_child_input_bindings_from_registered_multistage_parent_composed_parent_cone_instances`,
and the direct multi-stage helper counter to stay at zero. The live
Phase 4 matrix policy includes the dedicated
`phase4_recur_d2_registered_parent_cone_instance_multistage_state`
axis; the full downstream-clean `r42` report proves it with
`coverage_gaps = []` and `348/0` pass-fail in Verilator plus both
repo-owned Yosys modes.

**Focused recursive non-top registered parent-composed helper proof (new targeted evidence):**
current HEAD now proves registered parent-composed helper D-cone routing
below the top parent in an exact-depth-2 recursive hierarchy. The focused
proof is
`cargo test recursive_hierarchy_registered_child_input_cones_can_use_helper_instances_below_top`;
it requires `realized_min_leaf_depth = realized_max_leaf_depth = 2`,
`hierarchy_parent_cone_instances > top_parent_cone_instances`,
`hierarchy_parent_local_flops > top_local_flops`,
`child_input_bindings_from_registered_parent_composed_logic >
top_child_input_bindings_from_registered_parent_composed_logic`, and
`child_input_bindings_from_registered_parent_cone_instances >
top_child_input_bindings_from_registered_parent_cone_instances`. The
live Phase 4 matrix policy includes the dedicated
`phase4_recur_d2_registered_parent_cone_instance_state` axis; the full
downstream-clean `r41` report proves it with `coverage_gaps = []` and
`348/0` pass-fail in Verilator plus both repo-owned Yosys modes.

**Focused recursive non-top registered helper mixed-support proof (new targeted evidence):**
current HEAD now proves that the same non-top registered
parent-composed helper D-cone route can also mix parent data-port
support into the helper-sourced D cone. The focused proof is
`cargo test recursive_hierarchy_registered_helper_routes_mix_parent_ports_below_top`;
it requires `realized_min_leaf_depth = realized_max_leaf_depth = 2`,
`hierarchy_parent_cone_instances > top_parent_cone_instances`,
`hierarchy_parent_local_flops > top_local_flops`,
`child_input_bindings_from_registered_parent_composed_logic >
top_child_input_bindings_from_registered_parent_composed_logic`,
`child_input_bindings_from_registered_parent_cone_instances >
top_child_input_bindings_from_registered_parent_cone_instances`,
`child_input_bindings_from_registered_parent_cone_instance_mixed_support >
top_child_input_bindings_from_registered_parent_cone_instance_mixed_support`,
and
`registered_parent_cone_instance_mixed_support_child_input_binding_fraction > 0.0`.
The live Phase 4 matrix policy now requires
`saw_recursive_hierarchy_registered_parent_cone_instance_mixed_support_routing`;
the full downstream-clean `r48` report proves it with
`coverage_gaps = []` and `396/0` pass-fail in Verilator plus both
repo-owned Yosys modes.

**Focused direct sibling helper proof (new targeted evidence):**
current HEAD now lets the direct unregistered sibling route allocate and
use a parent-cone helper instance as the child-input source when both
`hierarchy_sibling_route_prob` and
`hierarchy_parent_cone_instance_prob` are active. The focused regression
is `cargo test hierarchy_sibling_routes_can_use_helper_instances`; the
design metrics prove this is not a registered route by requiring
`child_input_bindings_from_registered_instance_outputs = 0` and
`child_input_bindings_from_registered_parent_cone_instances = 0` while
`child_input_bindings_from_parent_cone_instances > 0`,
`parent_cone_instance_child_input_binding_fraction > 0.0`,
`top_parent_cone_instance_child_input_binding_fraction > 0.0`, and
helper instances are present beyond the planned child slots. This route
is now banked in the full downstream-clean `r34` Phase 4 matrix through
the dedicated direct sibling helper scenario.

**Focused direct registered sibling helper proof (new targeted evidence):**
current HEAD also lets the direct registered sibling route allocate and
use a parent-cone helper instance as the parent-flop D source when both
`hierarchy_registered_sibling_route_prob` and
`hierarchy_parent_cone_instance_prob` are active. The focused regression
is `cargo test hierarchy_registered_sibling_routes_can_use_helper_instances`;
the design metrics prove this is not the older registered
parent-composed D-cone path by requiring
`child_input_bindings_from_registered_parent_composed_logic = 0` while
`child_input_bindings_from_registered_parent_cone_instances > 0`,
`registered_parent_cone_instance_child_input_binding_fraction > 0.0`,
and helper instances are present beyond the planned child slots. This
route is now banked in the full downstream-clean `r30` Phase 4 matrix
through the dedicated direct registered sibling helper scenario.

**Focused multi-stage registered sibling proof (new targeted evidence):**
current HEAD also lets direct registered sibling routes chain through
earlier parent-local Qs. This is the narrow registered child-to-child
path, not the registered parent-composed route: the later child input
still receives a parent-local flop Q, but that flop's D source can be a
prior parent Q that ultimately came from an earlier sibling output. The
focused regression is
`cargo test hierarchy_registered_sibling_routes_can_chain_through_parent_flops`;
the design metrics prove the route through
`child_input_bindings_from_registered_multistage_instance_outputs`,
`top_child_input_bindings_from_registered_multistage_instance_outputs`,
`registered_multistage_instance_output_child_input_binding_fraction`,
and
`top_registered_multistage_instance_output_child_input_binding_fraction`,
while keeping the registered parent-composed counters at zero. This is
banked in the full downstream-clean `r30` Phase 4 matrix through the
dedicated
`phase4_hier2_inst4_registered_sibling_multistage_state` scenario.

**Focused multi-stage direct registered sibling helper proof (new targeted evidence):**
current HEAD also lets a direct registered sibling route seed a
parent-local Q from a parent-cone helper output and lets a later direct
registered sibling route reuse that Q as the next flop D source. The
focused regression is
`cargo test hierarchy_registered_sibling_routes_can_chain_helper_instances_through_parent_flops`;
the design metrics prove the route through
`child_input_bindings_from_registered_multistage_parent_cone_instances`,
`top_child_input_bindings_from_registered_multistage_parent_cone_instances`,
`registered_multistage_parent_cone_instance_child_input_binding_fraction`,
and
`top_registered_multistage_parent_cone_instance_child_input_binding_fraction`,
while keeping registered parent-composed counters at zero. This is
banked in the full downstream-clean `r30` Phase 4 matrix through the
dedicated
`phase4_hier2_inst4_registered_sibling_parent_cone_instance_multistage_state`
scenario.

**Focused recursive-shape proof (still useful targeted evidence):**
current HEAD also has bounded recursive hierarchy proven directly at
`/tmp/anvil-hier-range-smoke-r1/manifest.json`, clean in Verilator,
Yosys `synth -noabc`, and the repo-owned Yosys with-ABC path. The
design metrics there still prove the tree numerically:
`realized_min_leaf_depth = 2`, `realized_max_leaf_depth = 2`,
`instance_slots_by_parent_depth = {0: 2, 1: 5}`,
`min_child_instances_per_internal_module = 2`,
`max_child_instances_per_internal_module = 3`,
`hierarchy_parent_composed_outputs = 22`, and
`top_parent_composed_outputs = 11`.

**Focused mixed-depth recursive proof (new targeted evidence):**
current HEAD can now mix shallow and deep branches inside one bounded
recursive tree. The focused proof artifact is
`/tmp/anvil-hier-mixed-depth-smoke-r1/manifest.json`, clean in
Verilator, Yosys `synth -noabc`, and the repo-owned Yosys with-ABC
path. The design metrics there prove the mixed shape numerically:
`realized_min_leaf_depth = 2`,
`realized_max_leaf_depth = 3`,
`leaf_module_occurrences_by_depth = {"2": 2, "3": 4}`,
`avg_child_instances_by_parent_depth = {"0": 2.0, "1": 2.0, "2": 2.0}`,
`hierarchy_parent_composed_outputs = 40`, and
`top_parent_composed_outputs = 14`.

**Focused per-depth-branching proof (still useful targeted evidence):**
current HEAD also supports depth-specific recursive branching control
via repeated `--child-instances-per-depth DEPTH=MIN:MAX` overrides.
The focused proof artifact is
`/tmp/anvil-hier-depth-profile-smoke-r1/manifest.json`, clean in
Verilator, Yosys `synth -noabc`, and the repo-owned Yosys with-ABC
path. The design metrics there prove the depth-specific shape without
SV inspection:
`realized_min_leaf_depth = 2`,
`realized_max_leaf_depth = 2`,
`avg_child_instances_by_parent_depth = {"0": 4.0, "1": 2.0}`,
`min_child_instances_by_parent_depth = {"0": 4, "1": 2}`,
`max_child_instances_by_parent_depth = {"0": 4, "1": 2}`,
`hierarchy_parent_composed_outputs = 36`, and
`top_parent_composed_outputs = 18`.

**Focused parent-composed child-input proof (new targeted evidence):**
current HEAD also supports parent-local combinational cones for child
data input bindings. The focused proof artifact is
`/tmp/anvil-hier-child-input-cone-smoke-r1/manifest.json`, clean in
Verilator, Yosys `synth -noabc`, and the repo-owned Yosys with-ABC
path. The design metrics there prove the route numerically:
`child_input_bindings_from_parent_composed_logic = 13`,
`top_child_input_bindings_from_parent_composed_logic = 13`,
`parent_composed_child_input_binding_fraction = 0.9285714285714286`,
and `top_parent_composed_child_input_binding_fraction = 0.9285714285714286`.

**Focused mixed parent-output proof (new targeted evidence):**
current HEAD also supports parent outputs that mix parent data ports
with child instance outputs while preserving child-output support. The
focused proof artifact is
`/tmp/anvil-hier-parent-output-mix-smoke-r1/manifest.json`, clean in
Verilator, Yosys `synth -noabc`, and the repo-owned Yosys with-ABC
path. The design metrics there prove the route numerically:
`top_parent_port_composed_outputs = 8`,
`hierarchy_parent_port_composed_outputs = 8`,
`top_outputs_reaching_instance_outputs = 8`, and
`top_outputs_without_instance_outputs = 0`.

**Focused registered parent-composed child-input proof (new targeted evidence):**
current HEAD also supports registered parent-composed child-input
bindings through `--hierarchy-registered-child-input-cone-prob`. The
focused proof artifact is
`/tmp/anvil-hier-registered-child-input-cone-smoke-r2/manifest.json`,
clean in Verilator, Yosys `synth -noabc`, and the repo-owned Yosys
with-ABC path. The design metrics there prove the route numerically:
`child_input_bindings_from_registered_parent_composed_logic = 3`,
`top_child_input_bindings_from_registered_parent_composed_logic = 3`,
`registered_parent_composed_child_input_binding_fraction = 0.75`,
`top_registered_parent_composed_child_input_binding_fraction = 0.75`,
`child_input_bindings_from_registered_instance_outputs = 3`, and
`hierarchy_parent_local_flops = 3`.

**Focused registered mixed-support child-input proof (new targeted evidence):**
current HEAD now lets that registered parent-composed route mix parent
data ports with sibling outputs when both supports are live. The
focused proof artifact is
`/tmp/anvil-hier-registered-mixed-child-input-smoke-r1/manifest.json`,
clean in Verilator, Yosys `synth -noabc`, and the repo-owned Yosys
with-ABC path. The design metrics prove the mixed registered route:
`child_input_bindings_from_registered_mixed_support = 3`,
`top_child_input_bindings_from_registered_mixed_support = 3`,
`registered_mixed_support_child_input_binding_fraction = 0.75`, and
`top_registered_mixed_support_child_input_binding_fraction = 0.75`.

**Focused multi-stage registered parent-composed child-input proof (new targeted evidence):**
current HEAD now also lets later registered parent-composed
child-input routes chain through earlier parent-local Qs. The focused
proof artifact is
`/tmp/anvil-hier-registered-multistage-child-input-smoke-r1/manifest.json`,
clean in Verilator, Yosys `synth -noabc`, and the repo-owned Yosys
with-ABC path. The design metrics prove the multi-stage route:
`child_input_bindings_from_registered_multistage_parent_composed_logic = 2`,
`top_child_input_bindings_from_registered_multistage_parent_composed_logic = 2`,
and
`registered_multistage_parent_composed_child_input_binding_fraction = 0.5`.

**Focused local-parent-state proof (new targeted evidence):**
current HEAD also supports local parent flops in hierarchy parent-side
cones through `--hierarchy-parent-flop-prob`. The focused proof
artifact is `/tmp/anvil-hier-parent-state-smoke-r1/manifest.json`,
clean in Verilator, Yosys `synth -noabc`, and the repo-owned Yosys
with-ABC path. The design metrics there prove the state surface
numerically: `hierarchy_parent_local_flops = 8`,
`top_local_flops = 8`, `top_clock_inputs = 1`,
`top_reset_inputs = 1`, and
`child_input_bindings_from_parent_flops = 1`.

**Broadened wrapper planning (landed, closure refreshed):** the legacy
wrapper code and tests separate `num_leaf_modules` from
`num_child_instances`, and that behavior is now backed by both focused
smokes and the fresh full repo-owned gate above. The old `r7` report is
now the historical wrapper-baseline artifact; `r9` is the pre-mixed
recursive bank, `r10` is the pre-child-sourcing recursive bank,
`r13` is the pre-parent-input-cone bank, `r15` is the pre-parent-state
bank, `r16` is the pre-registered-sibling-route bank, `r17` is the
pre-registered-parent-composed-route bank, `r18` is the first
registered-parent-composed bank, `r19` is the pre-full parent-port /
registered-mixed / multi-stage bank, `r20` is the pre-parent-cone
helper-instance bank, `r21` is the historical pre-parent-output-helper
full bank, `r22` is the clean but insufficient 126-design pre-fix
budget-mismatch run, `r23` is the pre-direct-helper full bank, `r24`
is the historical coverage-only direct-helper policy proof, `r25` is
the previous direct-helper full bank, `r26` is the previous multi-stage
registered sibling bank, `r27` is the previous stateful
parent-output-helper bank, `r28` is the previous multi-stage direct
registered sibling helper bank, `r29` is the previous multi-stage
registered parent-composed helper bank, `r30` is the previous stateful
parent-composed helper full bank, `r31` is the previous recursive
helper-state full bank, `r32` is the failed direct-helper/Yosys-warning
attempt, `r33` is the pre-compact-normalization direct-helper bank, and
`r34` is the previous full downstream-clean 69-scenario recursive
direct-helper Phase 4 hierarchy closure artifact, `r35` is the previous
full downstream-clean 72-scenario recursive direct registered-helper
artifact, `r36` is the previous full downstream-clean 75-scenario
recursive registered parent-composed helper artifact, `r37` is the
previous full downstream-clean 78-scenario recursive non-top multi-stage
direct registered helper artifact, `r38` is the previous full
downstream-clean 81-scenario recursive non-top multi-stage registered
parent-composed helper artifact, `r39` is the previous full
downstream-clean 84-scenario recursive non-top parent-output helper
artifact, `r40` is the previous full downstream-clean 87-scenario
recursive non-top stateful parent-output helper artifact, and `r41` is
the previous full downstream-clean 87-scenario recursive non-top
parent-output multi-helper budget artifact. `r42` is the previous full
downstream-clean 87-scenario recursive non-top stateful multi-helper
budget artifact. `r43` is the previous full downstream-clean
90-scenario recursive non-top child-input multi-helper budget artifact.
`r44` is the previous full downstream-clean 93-scenario recursive
non-top registered mixed-support routing artifact. `r45` is the
previous full downstream-clean 96-scenario recursive non-top
multi-stage registered parent-composed no-helper routing artifact.
`r46` is the previous full downstream-clean 99-scenario recursive
non-top multi-stage registered sibling no-helper routing artifact.
`r47` is the previous full downstream-clean 99-scenario recursive
non-top multi-stage registered mixed-support no-helper routing artifact.
`r48` is the previous full downstream-clean 99-scenario recursive
non-top registered parent-composed helper mixed-support routing artifact.
`r49` is the previous full downstream-clean 99-scenario recursive
non-top parent-output helper mixed-support routing artifact. `r50` is
the previous full downstream-clean 99-scenario accumulated mixed-support
hierarchy artifact. `r51` is the previous full downstream-clean
102-scenario direct registered sibling mixed-support hierarchy artifact.
`r52` is the previous full downstream-clean 105-scenario recursive direct
registered sibling mixed-support hierarchy artifact. `r53` is the previous
full downstream-clean 108-scenario recursive parent-composed mixed-support
child-input hierarchy artifact. `r54` is the previous full downstream-clean
111-scenario recursive parent-port-composed parent-output hierarchy artifact.
`r55` is the previous full downstream-clean 114-scenario recursive stateful
parent-port-composed parent-output hierarchy artifact.
`r56` is the previous full downstream-clean 117-scenario recursive stateful
unregistered parent-composed mixed-support child-input hierarchy artifact.
`r57` is the previous full downstream-clean 120-scenario recursive
parent-local-flops gated coverage hierarchy artifact.
`r58` is the previous full downstream-clean 123-scenario recursive
depth-3 parent-local-flops gated coverage hierarchy artifact.
`r59` is the previous full downstream-clean 126-scenario recursive
depth-3 unregistered parent-composed mixed-support child-input gated
coverage hierarchy artifact.
`r60` is the previous full downstream-clean 129-scenario recursive
depth-3 parent-port-composed parent-output gated coverage hierarchy
artifact.
`r61` is the previous full downstream-clean 132-scenario recursive
depth-3 stateful parent-port-composed parent-output gated coverage
hierarchy artifact.
`r62` is the previous full downstream-clean 135-scenario recursive
depth-3 stateful parent-composed mixed-support child-input gated
coverage hierarchy artifact, completing the depth-3 sweep.
`r63` is the previous full downstream-clean 138-scenario recursive
depth-4 parent-local-flop gated coverage hierarchy artifact, opening
the depth-4 axis.
`r64` is the previous full downstream-clean 141-scenario recursive
depth-4 mixed-support child-input gated coverage hierarchy artifact.
`r65` is the previous full downstream-clean 144-scenario recursive
depth-4 parent-port-composed parent-output gated coverage hierarchy
artifact.
`r66` is the previous full downstream-clean 147-scenario recursive
depth-4 stateful parent-port-composed parent-output gated coverage
hierarchy artifact.
`r67` is the previous full downstream-clean 150-scenario recursive
depth-4 stateful parent-composed mixed-support child-input gated
coverage hierarchy artifact, completing the depth-4 sweep.
`r68` is the previous full downstream-clean 153-scenario recursive
depth-5 parent-local-flop gated coverage hierarchy artifact, opening
the depth-5 axis.
`r69` is the previous full downstream-clean 156-scenario recursive
depth-5 mixed-support child-input gated coverage hierarchy artifact.
`r70` is the previous full downstream-clean 159-scenario recursive
depth-5 parent-port-composed parent-output gated coverage hierarchy
artifact.
`r71` is the previous full downstream-clean 162-scenario recursive
depth-5 stateful parent-port-composed parent-output gated coverage
hierarchy artifact.
`r72` is the previous full downstream-clean 165-scenario recursive
depth-5 stateful unregistered parent-composed mixed-support child-input
gated coverage hierarchy artifact, closing the depth-5 sweep.
`r73` is the previous full downstream-clean 168-scenario recursive
depth-6 parent-local-flop gated coverage hierarchy artifact, opening
the depth-6 axis.
`r74` is the previous full downstream-clean 171-scenario recursive
depth-6 mixed-support child-input gated coverage hierarchy artifact (2,2 calibrated).
`r75` is the previous full downstream-clean 174-scenario recursive
depth-6 parent-port-composed parent-output gated coverage hierarchy
artifact.
`r76` is the previous full downstream-clean 177-scenario recursive
depth-6 stateful parent-port-composed parent-output gated coverage
hierarchy artifact.
`r77` is the previous full downstream-clean 180-scenario recursive
depth-6 stateful unregistered parent-composed mixed-support child-input
gated coverage hierarchy artifact (2,2 calibrated), closing the
depth-6 sweep.
`r78` is the previous full downstream-clean 183-scenario recursive
depth-7 parent-local-flop gated coverage hierarchy artifact, opening
the depth-7 axis.
`r79` is the previous full downstream-clean 186-scenario recursive
depth-7 mixed-support child-input gated coverage hierarchy artifact
(2,2 calibrated).
`r80` is the previous full downstream-clean 189-scenario recursive
depth-7 parent-port-composed parent-output gated coverage hierarchy
artifact.
`r81` is the previous full downstream-clean 192-scenario recursive
depth-7 stateful parent-port-composed parent-output gated coverage
hierarchy artifact.
`r82` is the previous full downstream-clean 195-scenario recursive
depth-7 stateful unregistered parent-composed mixed-support child-input
gated coverage hierarchy artifact (2,2 calibrated) — closed the
depth-7 sweep.
`r83` is the previous full downstream-clean 198-scenario recursive
three-stage registered parent-composed chain gated coverage hierarchy
artifact, opening a chain-depth axis above the closed depth-3..7
sweeps.
`r84` is the current full downstream-clean 201-scenario recursive
parent-cone helper budget 5 gated coverage hierarchy artifact,
extending the helper-budget axis above the previous budget-3 baseline.

Current-code coverage-only probes after `r19` first aligned the gate
policy with newer focused slices: `/tmp/anvil-tool-matrix-phase4-parent-port-coverage-r1/tool_matrix_report.json`
requires mixed parent-output composition, and
`/tmp/anvil-tool-matrix-phase4-registered-mixed-r1/tool_matrix_report.json`
requires registered mixed-support child-input routing, and
`/tmp/anvil-tool-matrix-phase4-registered-multistage-r1/tool_matrix_report.json`
requires multi-stage registered parent-composed routing. All record
`coverage_gaps = []` with Verilator/Yosys skipped; `r20` folded those
three coverage facts into a full downstream-clean bank, `r21` added the
parent-cone helper-instance child-input coverage fact, `r23` banked the
pre-direct-helper 42-scenario helper surface, `r24` first proved the
48-scenario direct-helper policy with tools skipped, `r25` banked that
direct-helper policy through Verilator and both repo-owned Yosys modes,
`r26` adds and banks the 51-scenario multi-stage registered sibling
policy, `r27` adds and banks the 54-scenario stateful
parent-output-helper policy, `r28` adds and banks the 57-scenario
multi-stage direct registered sibling helper policy, `r29` adds and
banks the 60-scenario multi-stage registered parent-composed helper
policy, `r30` adds and banks the 63-scenario stateful
parent-composed helper child-input policy, `r31` adds and banks the
66-scenario recursive non-top helper-through-state policy, `r33` first
banks the 69-scenario recursive non-top direct sibling helper policy
after fixing the `r32` CaseMux/Casez shift-cleanup warning, `r34`
refreshes that bank after the post-remap idempotent duplicate cleanup,
`r35` adds and banks the 72-scenario recursive non-top direct registered
sibling helper policy, `r36` adds and banks the 75-scenario recursive
non-top registered parent-composed helper policy, `r37` adds and banks
the 78-scenario recursive non-top multi-stage direct registered sibling
helper policy, `r38` adds and banks the 81-scenario recursive non-top
multi-stage registered parent-composed helper policy, `r39` adds and
banks the 84-scenario recursive non-top parent-output helper policy, and
`r40` adds and banks the 87-scenario recursive non-top stateful
parent-output helper policy. `r41` adds and banks the recursive non-top
multi-helper budget policy on the same 87-scenario matrix. `r42` adds
and banks the recursive non-top stateful multi-helper budget policy on
that same 87-scenario matrix. `r43` adds and banks the recursive
non-top child-input multi-helper budget policy on the expanded
90-scenario matrix. `r44` adds and banks the recursive non-top
registered mixed-support routing policy on the expanded 93-scenario
matrix. `r45` adds and banks the recursive non-top multi-stage
registered parent-composed no-helper routing policy on the expanded
96-scenario matrix. `r46` adds and banks the recursive non-top
multi-stage registered sibling no-helper routing policy on the expanded
99-scenario matrix. `r47` adds and banks the recursive non-top
multi-stage registered mixed-support no-helper routing policy on that
same 99-scenario matrix. `r48` adds and banks the recursive non-top
registered parent-composed helper mixed-support routing policy on that
same 99-scenario matrix. `r49` adds and banks the recursive non-top
parent-output helper mixed-support routing policy on that same
99-scenario matrix. `r50` banks the accumulated stateful parent-output
helper mixed-support, unregistered helper child-input mixed-support, and
stateful helper-through-flop child-input mixed-support policy facts
through Verilator and both repo-owned Yosys modes on the same
99-scenario matrix. `r51` adds and banks direct registered sibling
mixed-support routing on the expanded 102-scenario matrix. `r52` adds
and banks recursive non-top direct registered sibling mixed-support
routing on the expanded 105-scenario matrix. `r53` adds and banks
recursive non-top unregistered parent-composed mixed-support child-input
routing without helper instances on the expanded 108-scenario matrix.
`r54` adds and banks recursive non-top parent-port-composed parent-output
routing without helper instances or parent-local state on the expanded
111-scenario matrix. `r55` adds and banks recursive non-top stateful
parent-port-composed parent-output routing without helper instances on
the expanded 114-scenario matrix. `r56` adds and banks recursive non-top
stateful unregistered parent-composed mixed-support child-input routing
through parent-local Qs without helper instances on the expanded
117-scenario matrix. `r57` adds and banks recursive non-top
parent-local flops as a first-class gated coverage fact (with the
focused `phase4_recur_d2_parent_state` matrix scenario per construction
strategy) on the expanded 120-scenario matrix. `r58` extends that
coverage to exact hierarchy depth 3 (with the focused
`phase4_recur_d3_parent_state` matrix scenario per construction
strategy and the new `saw_recursive_hierarchy_depth_3_parent_local_flops`
fact) on the expanded 123-scenario matrix. `r59` extends the depth-3
coverage to the unregistered parent-composed mixed-support child-input
surface (with the focused `phase4_recur_d3_parent_composed_mixed_support_child_input`
matrix scenario per construction strategy and the new
`saw_recursive_hierarchy_depth_3_mixed_support_child_inputs` fact)
on the expanded 126-scenario matrix. `r60` extends the depth-3 coverage
to the parent-port-composed parent-output surface (with the focused
`phase4_recur_d3_parent_port_composed_output` matrix scenario per
construction strategy and the new
`saw_recursive_hierarchy_depth_3_parent_port_composed_outputs` fact) on
the expanded 129-scenario matrix. `r61` extends the depth-3 coverage to
the stateful parent-port-composed parent-output surface (with the
focused `phase4_recur_d3_stateful_parent_port_composed_output` matrix
scenario per construction strategy and the new
`saw_recursive_hierarchy_depth_3_stateful_parent_port_composed_outputs`
fact) on the expanded 132-scenario matrix. `r62` closes the depth-3
sweep by extending coverage to the stateful parent-composed
mixed-support child-input surface (with the focused
`phase4_recur_d3_stateful_parent_composed_mixed_support_child_input`
matrix scenario per construction strategy and the new
`saw_recursive_hierarchy_depth_3_stateful_parent_composed_mixed_support_child_inputs`
fact) on the expanded 135-scenario matrix. `r63` opens the depth-4
axis on top of the completed depth-3 sweep with the parent-flop surface
at depth 4 (focused `phase4_recur_d4_parent_state` matrix scenario per
construction strategy and the new
`saw_recursive_hierarchy_depth_4_parent_local_flops` fact) on the
expanded 138-scenario matrix. `r64` extends the depth-4 axis to the
unregistered parent-composed mixed-support child-input surface (focused
`phase4_recur_d4_parent_composed_mixed_support_child_input` matrix
scenario per construction strategy and the new
`saw_recursive_hierarchy_depth_4_mixed_support_child_inputs` fact) on
the expanded 141-scenario matrix. `r65` extends the depth-4 axis to the
parent-port-composed parent-output surface (focused
`phase4_recur_d4_parent_port_composed_output` matrix scenario per
construction strategy and the new
`saw_recursive_hierarchy_depth_4_parent_port_composed_outputs` fact) on
the expanded 144-scenario matrix. `r66` extends the depth-4 axis to the
stateful parent-port-composed parent-output surface (focused
`phase4_recur_d4_stateful_parent_port_composed_output` matrix scenario
per construction strategy and the new
`saw_recursive_hierarchy_depth_4_stateful_parent_port_composed_outputs`
fact) on the expanded 147-scenario matrix. `r67` closes the depth-4
sweep by extending coverage to the stateful unregistered parent-composed
mixed-support child-input surface (focused
`phase4_recur_d4_stateful_parent_composed_mixed_support_child_input`
matrix scenario per construction strategy and the new
`saw_recursive_hierarchy_depth_4_stateful_parent_composed_mixed_support_child_inputs`
fact) on the expanded 150-scenario matrix. `r68` opens the depth-5 axis
on top of the completed depth-4 sweep with the parent-flop surface at
depth 5 (focused `phase4_recur_d5_parent_state` matrix scenario per
construction strategy and the new
`saw_recursive_hierarchy_depth_5_parent_local_flops` fact) on the
expanded 153-scenario matrix. `r69` extends the depth-5 axis to the
unregistered parent-composed mixed-support child-input surface (focused
`phase4_recur_d5_parent_composed_mixed_support_child_input` matrix
scenario per construction strategy and the new
`saw_recursive_hierarchy_depth_5_mixed_support_child_inputs` fact) on
the expanded 156-scenario matrix. `r70` extends the depth-5 axis to the
unregistered parent-port-composed parent-output surface (focused
`phase4_recur_d5_parent_port_composed_output` matrix scenario per
construction strategy and the new
`saw_recursive_hierarchy_depth_5_parent_port_composed_outputs` fact) on
the expanded 159-scenario matrix. `r71` extends the depth-5 axis to the
stateful parent-port-composed parent-output surface (focused
`phase4_recur_d5_stateful_parent_port_composed_output` matrix scenario
per construction strategy and the new
`saw_recursive_hierarchy_depth_5_stateful_parent_port_composed_outputs`
fact) on the expanded 162-scenario matrix. `r72` closes the depth-5
sweep with the stateful unregistered parent-composed mixed-support
child-input surface (focused
`phase4_recur_d5_stateful_parent_composed_mixed_support_child_input`
matrix scenario per construction strategy and the new
`saw_recursive_hierarchy_depth_5_stateful_parent_composed_mixed_support_child_inputs`
fact) on the expanded 165-scenario matrix. `r73` opens the depth-6
axis on top of the closed depth-5 sweep with the parent-flop surface at
depth 6 (focused `phase4_recur_d6_parent_state` matrix scenario per
construction strategy and the new
`saw_recursive_hierarchy_depth_6_parent_local_flops` fact) on the
expanded 168-scenario matrix. `r74` extends the depth-6 axis to the
unregistered parent-composed mixed-support child-input surface (focused
`phase4_recur_d6_parent_composed_mixed_support_child_input` matrix
scenario per construction strategy and the new
`saw_recursive_hierarchy_depth_6_mixed_support_child_inputs` fact) on
the expanded 171-scenario matrix (with a 2,2 child-instance calibration
at depth 6 instead of the 4,4 used at depths 3-5; see
DEVELOPMENT_NOTES.md). `r75` extends the depth-6 axis to the
unregistered parent-port-composed parent-output surface (focused
`phase4_recur_d6_parent_port_composed_output` matrix scenario per
construction strategy and the new
`saw_recursive_hierarchy_depth_6_parent_port_composed_outputs` fact) on
the expanded 174-scenario matrix. `r76` extends the depth-6 axis to the
stateful parent-port-composed parent-output surface (focused
`phase4_recur_d6_stateful_parent_port_composed_output` matrix scenario
per construction strategy and the new
`saw_recursive_hierarchy_depth_6_stateful_parent_port_composed_outputs`
fact) on the expanded 177-scenario matrix. `r77` closes the depth-6
sweep with the stateful unregistered parent-composed mixed-support
child-input surface (focused
`phase4_recur_d6_stateful_parent_composed_mixed_support_child_input`
matrix scenario per construction strategy and the new
`saw_recursive_hierarchy_depth_6_stateful_parent_composed_mixed_support_child_inputs`
fact) on the expanded 180-scenario matrix (with the same 2,2
child-instance calibration adopted by r74). `r78` opens the depth-7
axis on top of the closed depth-6 sweep with the parent-flop surface
at depth 7 (focused `phase4_recur_d7_parent_state` matrix scenario per
construction strategy and the new
`saw_recursive_hierarchy_depth_7_parent_local_flops` fact) on the
expanded 183-scenario matrix. `r79` extends the depth-7 axis to the
unregistered parent-composed mixed-support child-input surface (focused
`phase4_recur_d7_parent_composed_mixed_support_child_input` matrix
scenario per construction strategy and the new
`saw_recursive_hierarchy_depth_7_mixed_support_child_inputs` fact) on
the expanded 186-scenario matrix (2,2 calibration continues from
depth 6). `r80` extends the depth-7 axis to the unregistered
parent-port-composed parent-output surface (focused
`phase4_recur_d7_parent_port_composed_output` matrix scenario per
construction strategy and the new
`saw_recursive_hierarchy_depth_7_parent_port_composed_outputs` fact) on
the expanded 189-scenario matrix. `r81` extends the depth-7 axis to
the stateful parent-port-composed parent-output surface (focused
`phase4_recur_d7_stateful_parent_port_composed_output` matrix scenario
per construction strategy and the new
`saw_recursive_hierarchy_depth_7_stateful_parent_port_composed_outputs`
fact) on the expanded 192-scenario matrix. `r82` closes the depth-7
sweep by extending the axis to the stateful unregistered parent-composed
mixed-support child-input surface (focused
`phase4_recur_d7_stateful_parent_composed_mixed_support_child_input`
matrix scenario per construction strategy and the new
`saw_recursive_hierarchy_depth_7_stateful_parent_composed_mixed_support_child_inputs`
fact) on the expanded 195-scenario matrix (same 2,2 calibration as
r77/r79). `r83` opens a chain-depth axis above the closed
depth-3..7 sweeps by proving recursive non-top registered
parent-composed child-input bindings can chain through three or more
parent-local flop stages (focused
`phase4_recur_d3_registered_three_stage_parent_composed_chain`
matrix scenario per construction strategy and the new
`saw_recursive_hierarchy_three_stage_registered_parent_composed_chain`
fact) on the expanded 198-scenario matrix. `r84` extends the
helper-budget axis above the previous budget-3 baseline by proving a
recursive non-top internal parent can saturate a parent-cone helper
budget of 5 helpers (focused
`phase4_recur_d2_parent_cone_instance_budget5` matrix scenario per
construction strategy and the new
`saw_recursive_parent_cone_helper_budget_5` fact) on the expanded
201-scenario matrix.

**Phase 4 still remains in progress** because the phase is broader than
the current landed slice. The remaining substantive work is to continue
with broader helper-instance placement beyond the current
parent-composed child-input, parent-port-composed parent-output,
direct sibling, direct registered sibling, registered child-input,
parent-output, stateful parent-output,
stateful parent-composed child-input, recursive non-top stateful
parent-composed child-input, recursive non-top direct sibling,
recursive non-top direct registered sibling, recursive non-top
multi-stage direct registered sibling, recursive non-top multi-stage
registered parent-composed helper, recursive non-top registered
parent-composed helper, recursive non-top parent-output helper,
recursive non-top stateful parent-output helper, recursive non-top
parent-output budget, recursive non-top child-input budget,
recursive non-top stateful parent-output budget, recursive non-top
multi-stage registered parent-composed no-helper, recursive
non-top multi-stage registered sibling no-helper, and recursive non-top
registered parent-composed helper mixed-support slices,
richer registered
hierarchy patterns beyond the first multi-stage parent-flop chain, and
eventual hierarchy-aware identity/factorization.

## Phase 5 — Parameterization (not started)

- Generated modules take `parameter` declarations for widths.
- Instantiation picks parameter values from allowed ranges.
- Parameter-dependent widths propagate correctly through cone generation.
- **Hard prerequisite:** Phase 4 hierarchy as a real design/instance
  layer. The current wrapper slice is the first foothold, not the full
  parameter story; parameter-aware child selection and parameter-driven
  parent generation still belong to this phase.
- Parameter-aware identity must remain sound: different parameter values
  cannot accidentally alias to one `NodeId` or one module instance
  unless the resulting structure is genuinely equivalent.
- IR-level design recorded in `book/src/ir.md` "Future extensions /
  Parameters and generics".

## Phase 5b — Synthesizable aggregates (not started)

Scheduled alongside Phase 5; order is not fixed.

Three sub-paths, each with its own cost and payoff (full analysis in
`book/src/ir.md` "Future extensions / Synthesizable aggregates"):

- **Packed struct / union / array** — emitter-layer change only; IR
  stays flat. Low cost. Primary value: parser / elaboration coverage
  in downstream tools. Can land independently of Phase 4.
- **Unpacked arrays** — the memory-inference pattern. Covered by
  Phase 6 below.
- **Unpacked struct / union for datapath, enums** — deprioritised
  (unpacked datapath is mostly non-synthesizable; enums add no
  distinct stress value beyond typed constants).

## Phase 6 — Advanced motifs (not started)

- Memories (single-port, dual-port, with inferrable patterns only).
- FSMs with explicitly generated state encodings.
- Multi-clock with CDC-safe handshakes — optional, expensive. Until
  this lands, every module remains fully synchronous to a single clock.
- These motifs are not just feature-count work; they are a major part of
  the legal interaction richness needed for ANVIL to help find
  downstream tool bugs without sacrificing downstream-acceptance quality.

## Phase 7 — Oracle-backed micro-design artifacts (not started)

- Add a new artifact family for **small, self-contained `.sv` files
  with known expected facts** rather than broad cone complexity.
- Initial target: `rtl_const_expr`-style corpora:
  - parameter / localparam dependency chains;
  - widths and ranges derived from expressions (`[DEPTH-1:0]`, etc.);
  - generate conditions and loop bounds driven by expressions;
  - package-qualified constant use;
  - precedence-sensitive arithmetic / shift / comparison / equality /
    bitwise / logical / ternary expressions.
- Typical artifact size: one module, or a tiny cluster of modules when
  the pressure point needs local hierarchy.
- Every emitted file gets an **expected-facts manifest** capturing
  things like parameter values, resolved ranges, generate decisions, and
  other obviously-checkable elaboration facts.

**Exit criteria:** reproducible micro-design corpus, explicit
expected-facts contract, and parity checks showing downstream consumers
either agree with the manifest or produce a retained counterexample.

## Phase 8 — Frontend/elaboration accept corpora (not started)

- Add a source-level artifact family for **compact elaboratable
  hierarchies** rather than only the current circuit-IR leaf modules.
- Required surfaces include:
  - ANSI ports and parameter lists;
  - parameter / localparam flows;
  - module instantiation variants (named / ordered overrides, named /
    ordered / wildcard ports, instance arrays);
  - package imports and package-qualified constants/types;
  - typedef-backed types, structs, unions, enums, builtin integral
    atom types;
  - assign, `always_comb`, `always @(*)`, `always_ff`, and
    `always_latch`;
  - generate `if` / `for`.
- Add a **source-level parameter / hierarchy / package IR** suitable
  for this family instead of forcing everything through the current
  gate-level circuit IR.
- Emit an expected-facts manifest describing top parameter values,
  instance paths, child parameter values, child port bindings, selected
  generate branches, and similar elaboration facts.

**Exit criteria:** reproducible 1–3 module accept corpora with clear
tops, manifests of expected elaboration facts, and downstream parity
checks against those facts.

## Phase 9 — Multi-artifact ANVIL umbrella (not started)

- Add an **artifact-family selector** so one tool can drive all of the
  valid-by-construction synthesizable families above without
  overloading one generator path with contradictory promises.
- Unify reproducibility, manifests, seed handling, knob plumbing,
  corpus output layout, and downstream checking across artifact
  families.
- Preserve the doctrinal distinction:
  - synthesizable DUT RTL lane;
  - oracle-backed positive micro-design lane;
  - frontend/elaboration accept lane;
  - future valid synthesizable artifact lanes of similar kind.

**Exit criteria:** ANVIL can honestly present itself as the go-to tool
for pseudo-random HDL artifact generation, with explicit mode/lane
selection instead of one blurred notion of "random SV files."

## Non-goals

- Testbenches, assertions, coverage — `anvil` generates DUT code only.
- Non-synthesizable constructs (`initial`, delays, system tasks beyond
  `$display` in debug comments).
- Language coverage beyond the synthesizable SV subset.
- Bundled oracle / reference simulator — `anvil` does not embed a
  shadow RTL semantics engine. Generated artifacts can still stress
  downstream tools, but by being high-quality legal RTL with explicit
  expected-facts contracts rather than by turning `anvil` into a second
  simulator.
