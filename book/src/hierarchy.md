# Hierarchy: Modules of Modules

ANVIL no longer stops at isolated leaf modules only. Phase 4 is now
live in two deliberately scoped but real forms:

- the legacy exact **depth-1 wrapper** lane, and
- the newer **bounded recursive hierarchy** lane.

That means:

- ANVIL can generate a **library of leaf modules** with the existing
  leaf kernel,
- choose separate hierarchy-planning knobs depending on the lane,
- generate real parent modules above child modules,
- instantiate those children inside the parent, and
- build parent outputs from child instance outputs as real leaf
  variables.

This is genuine module composition. It exercises elaboration,
inter-module port binding, multi-file emission, and downstream top
selection. It is not yet the full future hierarchy story.

## Current live slice

The legacy exact wrapper entry point is:

```text
--hierarchy-depth 1 --num-leaf-modules N [--num-child-instances M]
```

The bounded recursive entry point is:

```text
--min-hierarchy-depth A --max-hierarchy-depth B
--min-child-instances-per-module C --max-child-instances-per-module D
--child-instances-per-depth DEPTH=MIN:MAX   # optional, repeatable
```

`num_child_instances = 0` preserves the legacy wrapper behavior:
instantiate every generated leaf definition exactly once. When it is
non-zero, the wrapper's child-instance count is chosen explicitly and is
allowed to differ from the library size.

Generation order today is:

```text
generate_design(rng, knobs):
    library = []
    for _ in 0..num_leaf_modules:
        library.push(generate_leaf_module(rng, knobs))

    instance_plan = plan_child_instance_indices(rng, library, knobs)
    top = generate_wrapper_top(library, instance_plan)
    return Design { top, modules: library + [top] }
```

The current wrapper planner intentionally covers three structurally
different cases:

- **Exact** — instantiate every definition once
  (`num_child_instances = num_leaf_modules`, or legacy `0`)
- **Under-instantiated** — instantiate only a shuffled subset of the
  library (`num_child_instances < num_leaf_modules`)
- **Reuse** — cover every definition once, then reuse definitions with
  replacement to reach the requested child count
  (`num_child_instances > num_leaf_modules`)

The bounded recursive planner follows a different rule set:

- it keeps every realized leaf depth inside `[A:B]`, and can now mix
  shallow and deep branches inside one tree when the interval is open
  and the structure allows it;
- every non-leaf module picks a child-instance count uniformly inside
  `[C:D]`, unless a per-parent-depth override is present;
- repeated `child_instances_per_depth` overrides are keyed by parent
  depth (`0` = top, `1` = its direct children, ...), and they take
  priority over the global fallback range at the matching depth;
- every direct child definition generated for a parent is instantiated
  at least once; and
- the resulting design metrics report the realized depth and branching
  numerically.

The current top planner is intentionally narrow but no longer a pure
pass-through shell:

- if the instantiated children carry sequential state, the wrapper gets
  shared `clk` and `rst_n` inputs;
- every child emitted input becomes a wrapper input (prefixed with the
  instance name), unless a later sibling input is routed from an
  earlier sibling instance output;
- every instantiated child output is still available as a real
  `Node::InstanceOutput` leaf in the parent IR;
- both wrapper and recursive parents now also expose a real
  combinational sibling-routing surface via
  `hierarchy_sibling_route_prob`, so later child inputs may bind from
  earlier sibling outputs through the same dep-bearing width-adaptation
  machinery used elsewhere in the generator;
- both lanes can also route a later child input through local parent
  flops via `hierarchy_registered_sibling_route_prob`. The default D
  source is an earlier sibling output; later routes may also use an
  earlier parent-local Q as the next flop's D source, creating a
  multi-stage registered child-to-child chain. The route stays separate
  from direct combinational sibling routing. When
  `hierarchy_parent_cone_instance_prob` also fires, the same direct
  registered route can use a helper instance output as the parent-flop
  D source;
- both lanes can also route a later child input through parent-local
  combinational logic over the full available parent source pool and
  then one local parent flop via
  `hierarchy_registered_child_input_cone_prob`. This route is separate
  from direct registered sibling routing because its flop D input is
  parent-composed logic, not a direct sibling output, and it can mix
  parent data inputs with sibling child outputs when both supports are
  live. Later registered parent-composed routes can also chain through
  earlier parent-local Qs when those are already available;
- both lanes can also bind child data inputs through parent-local
  combinational cones via `hierarchy_child_input_cone_prob`. Those
  cones are built over already-available parent sources: parent data
  inputs, earlier sibling instance outputs, and earlier parent-side
  route gates;
- both lanes can also instantiate helper children as internal sources
  for parent-composed child-input cones, direct sibling routes, direct
  registered sibling-route D inputs, registered child-input D cones,
  and parent-output cones via `hierarchy_parent_cone_instance_prob`.
  Helper instances are separate from planned child slots, their outputs
  can be routed directly, through parent combinational logic, or through
  one parent-local flop into later child inputs
  or parent outputs, and `max_parent_cone_instances_per_module`
  controls how many such helpers one parent may allocate. Parent-output
  composition can spend multiple helpers from that budget directly;
- both parent output cones and parent-composed child-input cones may
  now emit local parent flops when `hierarchy_parent_flop_prob` is
  non-zero. The default is `0.0`, so the hierarchy layer stays
  combinational unless the caller explicitly enables parent state;
- top outputs are now built from the full parent source pool by the
  existing cone builder, then repaired after finalization so every
  output retains child-output support and, when parent data inputs are
  live, can also carry parent-port support; and
- unused child outputs are left explicitly unconnected at the instance
  site (`.port()`) instead of being forced into fake top-level
  pass-throughs.

The control-port rule is deliberate and inductive:

- pure comb-only modules do **not** emit `clk` / `rst_n`;
- sequential leaves do emit `clk` / `rst_n`; and
- once a parent carries local state or sequential descendants, `clk` /
  `rst_n` stay visible all the way up the instantiated ancestor chain.

So the current hierarchy slice is **real** but also **honest**: parent
modules are real parent-side composition layers over child outputs,
combinational by default and optionally stateful when requested. They
can now mix parent data inputs into parent outputs while preserving
child-output support, mix shallow and deep branches in one recursive
tree, route later child inputs from earlier sibling outputs directly or
through one parent-local flop, compose child input bindings through
parent-local logic, instantiate helper children as parent-cone sources,
use those helpers as direct sibling child-input sources or direct
registered sibling-route D sources, force those helper children into
parent-output composition, spend the
per-parent helper budget through parent-output-only composition, and
add local parent flops. It still does not
solve hierarchy-aware identity.

## Choosing a hierarchy routing surface

For casual use, the important choice is whether the parent should only
wire children together, build combinational parent logic, or introduce
parent-local state. The defaults keep hierarchy mostly combinational.
The registered and local-parent-flop routes are opt-in so a user can
ask for state deliberately.

For advanced users and developers, each surface has a corresponding
metric contract. Those metrics are what the `tool_matrix` Phase 4 gate
uses to prove that a matrix did more than merely set a knob.

| Goal | Main knob | Shape produced | Metrics to inspect |
| ---- | --------- | -------------- | ------------------ |
| Bind later child inputs from earlier sibling outputs | `hierarchy_sibling_route_prob` | earlier child output -> later child input | `child_input_bindings_from_instance_outputs`, `instance_output_child_input_binding_fraction`, `top_instance_output_child_input_binding_fraction` |
| Let direct sibling routes use a helper child source | `hierarchy_sibling_route_prob` + `hierarchy_parent_cone_instance_prob` | helper child output -> later child input | `child_input_bindings_from_instance_outputs`, `child_input_bindings_from_parent_cone_instances`, `parent_cone_instance_child_input_binding_fraction`, `top_parent_cone_instance_child_input_binding_fraction` |
| Bind later child inputs through parent combinational logic | `hierarchy_child_input_cone_prob` | parent source(s) -> parent logic -> later child input | `child_input_bindings_from_parent_composed_logic`, `parent_composed_child_input_binding_fraction`, `top_parent_composed_child_input_binding_fraction` |
| Let parent-composed child-input cones instantiate a helper child source | `hierarchy_parent_cone_instance_prob` | helper child output -> parent logic -> later child input | `top_parent_cone_instances`, `hierarchy_parent_cone_instances`, `child_input_bindings_from_parent_cone_instances`, `parent_cone_instance_child_input_binding_fraction`, `top_parent_cone_instance_child_input_binding_fraction` |
| Let parent-output cones instantiate a helper child source | `hierarchy_parent_cone_instance_prob` | helper child output -> parent logic -> parent output | `top_outputs_reaching_parent_cone_instances`, `hierarchy_outputs_reaching_parent_cone_instances`, `top_parent_cone_instance_output_fraction`, `hierarchy_parent_cone_instance_output_fraction` |
| Allow more helper children per parent | `max_parent_cone_instances_per_module` | multiple helper child outputs -> parent logic | `max_parent_cone_instances_per_internal_module`, `top_parent_cone_instances`, `hierarchy_parent_cone_instances` |
| Bind later child inputs through one parent flop | `hierarchy_registered_sibling_route_prob` | earlier child output -> parent flop -> later child input | `child_input_bindings_from_registered_instance_outputs`, `registered_instance_output_child_input_binding_fraction`, `top_registered_instance_output_child_input_binding_fraction` |
| Chain direct registered sibling routes through earlier parent state | `hierarchy_registered_sibling_route_prob` | earlier child output -> parent flop -> later parent flop -> later child input | `child_input_bindings_from_registered_multistage_instance_outputs`, `top_child_input_bindings_from_registered_multistage_instance_outputs`, `registered_multistage_instance_output_child_input_binding_fraction`, `top_registered_multistage_instance_output_child_input_binding_fraction` |
| Let direct registered sibling routes use a helper child source | `hierarchy_registered_sibling_route_prob` + `hierarchy_parent_cone_instance_prob` | helper child output -> parent flop -> later child input | `child_input_bindings_from_registered_instance_outputs`, `child_input_bindings_from_registered_parent_cone_instances`, `registered_parent_cone_instance_child_input_binding_fraction`, `top_registered_parent_cone_instance_child_input_binding_fraction` |
| Bind later child inputs through registered parent-composed logic | `hierarchy_registered_child_input_cone_prob` | parent source(s), optionally including earlier parent Q -> parent logic -> parent flop -> later child input | `child_input_bindings_from_registered_parent_composed_logic`, `registered_parent_composed_child_input_binding_fraction`, `child_input_bindings_from_registered_mixed_support`, `registered_mixed_support_child_input_binding_fraction`, `child_input_bindings_from_registered_multistage_parent_composed_logic`, `registered_multistage_parent_composed_child_input_binding_fraction` |
| Let registered parent-composed child-input D cones instantiate a helper child source | `hierarchy_registered_child_input_cone_prob` + `hierarchy_parent_cone_instance_prob` | helper child output -> parent logic -> parent flop -> later child input | `child_input_bindings_from_registered_parent_cone_instances`, `top_child_input_bindings_from_registered_parent_cone_instances`, `registered_parent_cone_instance_child_input_binding_fraction`, `top_registered_parent_cone_instance_child_input_binding_fraction` |
| Allow parent cones to contain local flops | `hierarchy_parent_flop_prob` | parent source(s) -> parent cone with local flop(s) -> output or child input | `hierarchy_parent_local_flops`, `internal_module_occurrences_with_local_flops`, `top_local_flops`, `child_input_bindings_from_parent_flops` |

The registered parent-composed route now uses the full available parent
source pool. When both supports are live, the D side of the parent flop
can depend on parent data ports and sibling child outputs at the same
time. That is why the book distinguishes the broader
`registered_parent_composed_*` counters from the stricter
`registered_mixed_support_*` counters: the first proves registered
parent logic exists; the second proves that registered parent logic
actually mixed parent-port and child-output support.

The same route can also chain through earlier parent-local flops. In
that case the next registered D cone contains a prior parent Q and then
allocates a new parent flop for the child input. The
`registered_multistage_parent_composed_*` counters are the proof that
this first multi-stage registered parent-composed pattern appeared.

Direct registered sibling routing can chain too. In that narrower
shape, a later registered sibling route chooses an earlier parent-local
Q as the next flop's D source without inserting parent-composed logic.
The `registered_multistage_instance_output_*` counters prove that
registered child-to-child chain separately from the broader
parent-composed route above.

The parent-cone helper-instance route is separate from planned child
slots. It is opt-in, defaults to `0.0`, and the helper budget defaults
to one helper child per parent. Raise
`max_parent_cone_instances_per_module` to let the same parent allocate
multiple helper children; set it to `0` to suppress helper allocation
even when the probability fires. The budget is local to each parent, so
recursive designs can have `hierarchy_parent_cone_instances` above the
configured value across multiple internal modules. Use
`max_parent_cone_instances_per_internal_module` to verify the local
budget actually appeared.

Parent-output helper collection is intentionally output-proven. It
collects helper sources before building parent-output roots and then
selects a required helper source per output. For helper instances
created by this route, the helper child inputs are bound from
non-helper parent sources so `child_input_bindings_from_parent_cone_instances`
can remain zero in the focused output-only proof.

When helper placement combines with `hierarchy_sibling_route_prob`, the
helper output can be used directly as the later child input source. This
keeps the route unregistered; the registered helper counters remain zero
unless a parent-local flop is actually inserted.

When helper placement combines with
`hierarchy_registered_sibling_route_prob`, the helper output can be used
directly as the registered route's D source before the parent-local
flop. When helper placement combines with
`hierarchy_registered_child_input_cone_prob`, the helper output is
folded into the registered route's D cone before the final parent-local
flop. The plain `child_input_bindings_from_parent_cone_instances`
counter includes combinational and registered helper-sourced
child-input bindings; the stricter
`child_input_bindings_from_registered_parent_cone_instances` counter
proves the helper source appeared specifically on the registered D side,
whether the route came through direct registered sibling routing or
registered parent-composed logic.

## Current IR shape

Hierarchy now lives directly in the circuit IR:

```rust
pub struct Design {
    pub top: String,
    pub modules: Vec<Module>,
}

pub struct Module {
    // ...
    pub instances: Vec<Instance>,
}

pub enum InstanceRole {
    PlannedChild,
    ParentCone,
}

pub struct Instance {
    pub id: InstanceId,
    pub name: String,
    pub module: String,
    pub role: InstanceRole,
    pub inputs: Vec<(PortId, NodeId)>,
}

pub enum Node {
    // existing leaf forms ...
    InstanceOutput { instance: InstanceId, port: PortId, width: u32 },
}
```

Two details matter:

1. `Instance.inputs` are keyed by the **child's input port ids**.
   Design validation checks that every emitted child input is bound
   exactly once and at the right width.
2. `Node::InstanceOutput` is a real node kind and now carries a real
   leaf-variable identity in dependency tracking, so parent modules can
   build new cones over child outputs instead of treating them as
   emitter-only wiring.

## Design validation

Local module validation still exists, but hierarchy needs a second
layer:

```rust
pub fn validate_design(d: &Design) -> Result<(), DesignValidateError>
```

The design-level validator checks:

- the top module exists,
- module names are unique,
- every module passes local validation,
- every instance references a real child module,
- every child emitted input is bound exactly once,
- every referenced child output node names a real child output port at
  the right width,
- widths match at every binding/exposure point, and
- the module graph is acyclic.

That separation is deliberate. `validate(&Module)` guards one module's
internal circuit invariants. `validate_design(&Design)` guards the
cross-module contract.

## Emission model

The emitter now has three entry points:

```rust
to_sv(&Module)
to_sv_in_design(&Module, &Design)
to_sv_design(&Design)
```

`to_sv(&Module)` remains the leaf-only path. Hierarchical modules must
be emitted with design context so child modules can be resolved and
instantiation wiring can be rendered.

Directory output in hierarchy mode now writes:

- one `.sv` file per module in the design, and
- a `manifest.json` whose top-level payload uses `designs: [...]`
  rather than the old flat `modules: [...]` list.

Module names come from one generator-global sequence. That matters for
`--count N --out DIR`: every leaf, intermediate parent, top parent, and
later design in the same run gets a fresh `mod_<seed>_<index>` name, so
multi-file hierarchy output does not overwrite an earlier module
definition.

Each design entry now carries both:

- `hierarchy` facts (leaf count, child-instance count, reuse /
  under-instantiation flags), and
- exact per-design `metrics` describing composition quality directly.

Those design metrics are the intended trust surface for the current
Phase 4 slice. They let you judge hierarchy quality without opening the
emitted `.sv`, including:

- library size vs instantiated child count,
- unique-instantiated-module count and unused-library count,
- reuse / coverage ratios,
- top interface shape,
- direct-pass-through vs parent-composed top-output counts,
- parent-port-composed output counts and fractions
  (`top_parent_port_composed_outputs`,
  `hierarchy_parent_port_composed_outputs`,
  `top_parent_port_composed_output_fraction`,
  `hierarchy_parent_port_composed_output_fraction`),
- whether top outputs actually depend on child outputs at all,
- average / maximum child-output support per top output,
- child-input provenance
  (`child_input_bindings_from_parent_ports`,
  `child_input_bindings_from_instance_outputs`,
  `child_input_bindings_from_mixed_support`,
  `child_input_bindings_from_constants`,
  `child_input_bindings_from_parent_composed_logic`,
  `child_input_bindings_from_parent_flops`,
  `child_input_bindings_from_registered_instance_outputs`,
  `child_input_bindings_from_registered_parent_composed_logic`,
  `child_input_bindings_from_registered_mixed_support`,
  `child_input_bindings_from_registered_multistage_parent_composed_logic`,
  `child_input_bindings_from_registered_multistage_instance_outputs`,
  `child_input_bindings_from_parent_cone_instances`,
  `child_input_bindings_from_registered_parent_cone_instances`,
  `top_child_input_bindings_from_parent_cone_instances`),
- hierarchy- and top-level sibling-routing fractions
  (`instance_output_child_input_binding_fraction`,
  `top_instance_output_child_input_binding_fraction`),
- hierarchy- and top-level parent-composed child-input fractions
  (`parent_composed_child_input_binding_fraction`,
  `top_parent_composed_child_input_binding_fraction`),
- hierarchy- and top-level parent-flop child-input fractions
  (`parent_flop_child_input_binding_fraction`,
  `top_parent_flop_child_input_binding_fraction`),
- hierarchy- and top-level registered sibling-route fractions
  (`registered_instance_output_child_input_binding_fraction`,
  `top_registered_instance_output_child_input_binding_fraction`),
- hierarchy- and top-level multi-stage registered sibling-route
  fractions
  (`registered_multistage_instance_output_child_input_binding_fraction`,
  `top_registered_multistage_instance_output_child_input_binding_fraction`),
- hierarchy- and top-level registered parent-composed route fractions
  (`registered_parent_composed_child_input_binding_fraction`,
  `top_registered_parent_composed_child_input_binding_fraction`),
- hierarchy- and top-level registered mixed-support route fractions
  (`registered_mixed_support_child_input_binding_fraction`,
  `top_registered_mixed_support_child_input_binding_fraction`),
- hierarchy- and top-level multi-stage registered parent-composed route
  fractions
  (`registered_multistage_parent_composed_child_input_binding_fraction`,
  `top_registered_multistage_parent_composed_child_input_binding_fraction`),
- hierarchy- and top-level parent-cone helper-instance route fractions
  (`parent_cone_instance_child_input_binding_fraction`,
  `top_parent_cone_instance_child_input_binding_fraction`),
- hierarchy- and top-level registered parent-cone helper route fractions
  (`registered_parent_cone_instance_child_input_binding_fraction`,
  `top_registered_parent_cone_instance_child_input_binding_fraction`),
- hierarchy- and top-level parent-cone helper-instance output support
  (`hierarchy_outputs_reaching_parent_cone_instances`,
  `top_outputs_reaching_parent_cone_instances`,
  `hierarchy_parent_cone_instance_output_fraction`,
  `top_parent_cone_instance_output_fraction`),
- parent-cone helper-instance counts
  (`hierarchy_parent_cone_instances`, `top_parent_cone_instances`,
  `max_parent_cone_instances_per_internal_module`),
- local parent-state counts
  (`hierarchy_parent_local_flops`,
  `internal_module_occurrences_with_local_flops`,
  `top_local_flops`),
- control fanout to child instances,
- weighted child interface / node / flop load, and
- per-definition instantiation histograms,
- realized leaf depth / module depth,
- leaf-occurrence depth histogram,
- module-definition and module-occurrence depth histograms, and
- child-instance histograms plus per-depth instance-slot totals, and
- per-parent-depth branching summaries
  (`avg/min/max_child_instances_by_parent_depth`).

## Why the first slice started wrapper-only

This was a deliberate engineering choice, not a half-finished accident.

The original wrapper slice bought several important things immediately:

- real multi-module emitted RTL,
- real elaboration pressure in Verilator/Yosys,
- real design-aware validation and emission APIs, and
- a clean boundary above the leaf kernel instead of folding hierarchy
  into `generate_leaf_module`.

It also keeps the open work honest. The following is **not** live yet:

- hierarchy-aware `NodeId` identity/factorization.

What **is** now live beyond the original smoke is the repo-owned Phase 4
hierarchy gate:

- `/tmp/anvil-tool-matrix-phase4-hierarchy-r26/tool_matrix_report.json`
- `51` scenarios
- `4` designs/scenario
- `204` total designs
- `coverage_gaps = []`
- `Verilator 204/0`
- `Yosys without-abc 204/0`
- `Yosys with-abc 204/0`

That gate proves the current representative hierarchy surface directly
from saved report facts: multifile hierarchy designs, correct
top-module tool invocation, real child instances, real
`Node::InstanceOutput` use, wrapper exact / reuse / under-instantiation
profiles, recursive depth `2`, mixed recursive depth range `2:3`,
explicit child-sourcing modes `library` and `on-demand`,
child-instance profiles `2`, `4`, `2:3` and `1:3`, the per-depth
override profile `0=4:4,1=2:2`, real per-depth branching metrics, real
mixed shallow/deep recursive realization, real on-demand child
sourcing, exact profiled child-interface synthesis in the on-demand
lane, real sibling-routed hierarchy child inputs, real parent-side
composition above instance outputs, mixed parent-port / child-output
parent outputs, real parent-composed child-input bindings, registered
sibling-routed child-input bindings, registered parent-composed
child-input bindings, registered mixed-support child-input bindings
that mix parent ports with child outputs, multi-stage registered
parent-composed child-input bindings that chain through earlier
parent-local Qs, multi-stage registered sibling-routed child-input
bindings that chain through earlier parent-local Qs without
parent-composed logic, real local parent flops, parent-cone helper instances
sourcing parent-composed child-input bindings, parent-output helper
instance composition, budgeted multi-helper allocation, and registered
parent-composed helper-sourced child-input D cones, direct sibling
helper routing, and direct registered sibling helper routing. The
older `r21` bank remains historical pre-parent-output-helper evidence;
the clean `r22` run is root-cause evidence for the stale 126-design
budget mismatch that the per-scenario Phase 4 gate floor now prevents.
The `r23` full bank and the `r24` coverage-only direct-helper proof are
now historical breadcrumbs. The focused
proof artifact for that composed-parent
behavior remains:

- `/tmp/anvil-hier-parent-compose-smoke-r1/manifest.json`
- clean in Verilator
- clean in Yosys `synth -noabc`
- clean in the repo-owned Yosys with-ABC path
- metrics proving genuine parent composition:
  - `top_parent_composed_outputs = 10`
  - `top_direct_instance_output_drives = 0`
  - `top_instance_output_dependency_fraction = 1.0`
  - `avg_instance_output_support_per_top_output = 2.5`

Current HEAD has widened the wrapper planner beyond the exact-once case,
and that broadened repo-owned rerun is now banked too. The focused
local proofs remain useful:

- `/tmp/anvil-hier-reuse-smoke-r1` is clean in Verilator, Yosys
  `synth -noabc`, and the repo-owned ABC-enabled Yosys path, and proves
  repeated child-definition reuse;
- `/tmp/anvil-hier-under-smoke-r2` is clean in the same three lanes and
  proves under-instantiation of the leaf library; and
- `/tmp/anvil-hier-profiled-ondemand-smoke-r1/manifest.json` is clean
  in the same three lanes and proves exact profiled `on-demand`
  child-interface synthesis numerically:
  - `num_profiled_instance_slots = 3`
  - `profiled_instance_fraction = 1.0`
  - `profiled_instantiated_module_fraction = 1.0`
  - `dep_bearing_child_input_binding_fraction = 1.0`
- `/tmp/anvil-hier-sibling-routing-smoke-r1/manifest.json` is clean
  in the same three lanes and proves sibling-routed child-input binding
  numerically:
  - `child_input_bindings_from_instance_outputs = 6`
  - `top_child_input_bindings_from_instance_outputs = 6`
  - `instance_output_child_input_binding_fraction = 0.75`
  - `top_instance_output_child_input_binding_fraction = 0.75`
- `/tmp/anvil-hier-child-input-cone-smoke-r1/manifest.json` is clean
  in the same three lanes and proves parent-composed child-input binding
  numerically:
  - `child_input_bindings_from_parent_composed_logic = 13`
  - `top_child_input_bindings_from_parent_composed_logic = 13`
  - `parent_composed_child_input_binding_fraction = 0.9285714285714286`
  - `top_parent_composed_child_input_binding_fraction = 0.9285714285714286`
- `/tmp/anvil-parent-cone-instance-smoke-r1/manifest.json` is clean in
  the same three lanes and proves parent-cone helper-instance routing
  into child-input bindings numerically:
  - `top_parent_cone_instances = 1`
  - `hierarchy_parent_cone_instances = 1`
  - `child_input_bindings_from_parent_cone_instances = 4`
  - `top_child_input_bindings_from_parent_cone_instances = 4`
  - `parent_cone_instance_child_input_binding_fraction = 0.4444444444444444`
  - `top_parent_cone_instance_child_input_binding_fraction = 0.4444444444444444`
- `cargo test hierarchy_parent_outputs_can_depend_on_helper_instance_outputs`
  proves the newer parent-output helper-instance route numerically:
  - `top_parent_cone_instances > 0`
  - `top_outputs_reaching_parent_cone_instances > 0`
  - `hierarchy_outputs_reaching_parent_cone_instances > 0`
  - `top_parent_cone_instance_output_fraction > 0.0`
- `cargo test hierarchy_parent_cone_helper_budget_allows_multiple_helpers`
  proves budgeted helper allocation through child-input routing
  numerically:
  - `top_parent_cone_instances = 3`
  - `max_parent_cone_instances_per_internal_module = 3`
  - `child_input_bindings_from_parent_cone_instances > 0`
- `cargo test hierarchy_parent_outputs_can_spend_helper_budget` proves
  budgeted parent-output-only helper composition numerically:
  - `top_parent_cone_instances = 3`
  - `max_parent_cone_instances_per_internal_module = 3`
  - `child_input_bindings_from_parent_cone_instances = 0`
  - `top_outputs_reaching_parent_cone_instances >= 3`
- `cargo test hierarchy_registered_child_input_cones_can_use_helper_instances`
  proves registered helper-sourced child-input D cones numerically:
  - `child_input_bindings_from_registered_parent_cone_instances > 0`
  - `top_child_input_bindings_from_registered_parent_cone_instances > 0`
  - `registered_parent_cone_instance_child_input_binding_fraction > 0.0`
  - `top_registered_parent_cone_instance_child_input_binding_fraction > 0.0`
- `cargo test hierarchy_sibling_routes_can_use_helper_instances` proves
  direct sibling helper routing numerically:
  - `top_parent_cone_instances > 0`
  - `child_input_bindings_from_instance_outputs > 0`
  - `child_input_bindings_from_registered_instance_outputs = 0`
  - `child_input_bindings_from_registered_parent_cone_instances = 0`
  - `child_input_bindings_from_parent_cone_instances > 0`
  - `parent_cone_instance_child_input_binding_fraction > 0.0`
  - `top_parent_cone_instance_child_input_binding_fraction > 0.0`
  - `num_instances > planned_child_instances`
  This route is now also banked in the full downstream-clean `r26`
  Phase 4 matrix.
- `cargo test hierarchy_registered_sibling_routes_can_use_helper_instances`
  proves direct registered sibling helper routing numerically:
  - `top_parent_cone_instances > 0`
  - `child_input_bindings_from_registered_instance_outputs > 0`
  - `child_input_bindings_from_registered_parent_composed_logic = 0`
  - `child_input_bindings_from_registered_parent_cone_instances > 0`
  - `registered_parent_cone_instance_child_input_binding_fraction > 0.0`
  - `num_instances > planned_child_instances`
  This route is now also banked in the full downstream-clean `r26`
  Phase 4 matrix.
- `cargo test hierarchy_registered_sibling_routes_can_chain_through_parent_flops`
  proves multi-stage direct registered sibling routing numerically:
  - `child_input_bindings_from_registered_instance_outputs > 0`
  - `child_input_bindings_from_registered_multistage_instance_outputs > 0`
  - `top_child_input_bindings_from_registered_multistage_instance_outputs > 0`
  - `child_input_bindings_from_registered_parent_composed_logic = 0`
  - `registered_multistage_instance_output_child_input_binding_fraction > 0.0`
  This route is banked in the full downstream-clean `r26` Phase 4
  matrix through the dedicated
  `phase4_hier2_inst4_registered_sibling_multistage_state` scenario.
- `/tmp/anvil-hier-registered-mixed-child-input-smoke-r1/manifest.json`
  is clean in the same three lanes and proves registered mixed-support
  child-input binding numerically:
  - `child_input_bindings_from_registered_mixed_support = 3`
  - `top_child_input_bindings_from_registered_mixed_support = 3`
  - `registered_mixed_support_child_input_binding_fraction = 0.75`
  - `top_registered_mixed_support_child_input_binding_fraction = 0.75`
  The current-code coverage-only Phase 4 matrix probe at
  `/tmp/anvil-tool-matrix-phase4-registered-mixed-r1/tool_matrix_report.json`
  first banked this as a required coverage fact with
  `coverage_gaps = []` and
  `saw_hierarchy_registered_mixed_support_routing = true`. That probe
  skipped Verilator/Yosys; the full downstream-clean `r26` bank now
  carries the same fact with real tool validation.
- `/tmp/anvil-hier-registered-multistage-child-input-smoke-r1/manifest.json`
  is clean in the same three lanes and proves multi-stage registered
  parent-composed child-input binding numerically:
  - `child_input_bindings_from_registered_multistage_parent_composed_logic = 2`
  - `top_child_input_bindings_from_registered_multistage_parent_composed_logic = 2`
  - `registered_multistage_parent_composed_child_input_binding_fraction = 0.5`
  The current-code coverage-only Phase 4 matrix probe at
  `/tmp/anvil-tool-matrix-phase4-registered-multistage-r1/tool_matrix_report.json`
  first banked this as a required coverage fact with
  `coverage_gaps = []` and
  `saw_hierarchy_registered_multistage_routing = true`. That probe
  skipped Verilator/Yosys; the full downstream-clean `r26` bank now
  carries the same fact with real tool validation.
- `/tmp/anvil-hier-parent-output-mix-smoke-r1/manifest.json` is clean
  in the same three lanes and proves mixed parent-port / child-output
  parent outputs numerically:
  - `top_parent_port_composed_outputs = 8`
  - `hierarchy_parent_port_composed_outputs = 8`
  - `top_outputs_reaching_instance_outputs = 8`
  - `top_outputs_without_instance_outputs = 0`
  The current-code coverage-only Phase 4 matrix probe at
  `/tmp/anvil-tool-matrix-phase4-parent-port-coverage-r1/tool_matrix_report.json`
  first banked this as a required coverage fact with
  `coverage_gaps = []` and
  `saw_hierarchy_parent_port_composed_outputs = true`. That probe
  skipped Verilator/Yosys; the full downstream-clean `r26` bank now
  carries the same fact with real tool validation.
- `/tmp/anvil-hier-parent-state-smoke-r1/manifest.json` is clean in
  the same three lanes and proves local parent state numerically:
  - `hierarchy_parent_local_flops = 8`
  - `top_local_flops = 8`
  - `top_clock_inputs = 1`
  - `top_reset_inputs = 1`
  - `child_input_bindings_from_parent_flops = 1`
- `/tmp/anvil-hier-registered-sibling-smoke-r1/manifest.json` is clean
  in the same three lanes and proves registered sibling-routed
  child-input binding numerically:
  - `child_input_bindings_from_registered_instance_outputs = 4`
  - `top_child_input_bindings_from_registered_instance_outputs = 4`
  - `registered_instance_output_child_input_binding_fraction = 0.8`
  - `hierarchy_parent_local_flops = 3`
  - `top_clock_inputs = 1`
  - `top_reset_inputs = 1`
- `/tmp/anvil-hier-registered-child-input-cone-smoke-r2/manifest.json`
  is clean in the same three lanes and proves registered
  parent-composed child-input binding numerically:
  - `child_input_bindings_from_registered_parent_composed_logic = 3`
  - `top_child_input_bindings_from_registered_parent_composed_logic = 3`
  - `registered_parent_composed_child_input_binding_fraction = 0.75`
  - `top_registered_parent_composed_child_input_binding_fraction = 0.75`
  - `hierarchy_parent_local_flops = 3`
- the refreshed `tool_matrix` Phase 4 scenario set now explicitly
  targets wrapper and recursive hierarchy profiles, and the fresh rerun
  at `/tmp/anvil-tool-matrix-phase4-hierarchy-r26` closes them cleanly
  with `coverage_gaps = []` and `204/0` pass-fail in Verilator plus both
  repo-owned Yosys modes, including the direct sibling helper and direct
  registered sibling helper routes.
  The older `r7` report is now the historical
  wrapper-baseline artifact, `r9` is the pre-mixed recursive bank,
  `r10` is the pre-on-demand mixed-depth bank, `r11` is the first
  explicit child-sourcing bank, `r12` is the first exact profiled
  child-interface bank, `r13` is the pre-parent-input-cone bank, `r15`
  is the pre-parent-state bank, `r16` is the pre-registered-sibling-route
  bank, `r17` is the pre-registered-parent-composed-route bank, `r19`
  is the pre-full parent-port / registered-mixed / multi-stage bank,
  `r20` is the pre-parent-cone helper-instance bank, `r21` is the
  historical pre-parent-output-helper full bank, `r22` is the clean but
  insufficient 126-design pre-fix budget-mismatch run, `r23` is the
  pre-direct-helper full bank, `r24` is the coverage-only direct-helper
  policy proof, `r25` is the previous direct-helper full bank, `r26`
  is the latest full downstream-clean hierarchy bank, and the aborted `r8`
  rerun is historical evidence that the Phase 4 gate should use a
  hierarchy-focused sequential leaf profile instead of silently
  borrowing the fattest Phase 1 leaf-stress shape.

Current HEAD now also has a focused clean proof for the bounded
recursive lane:

- `/tmp/anvil-hier-range-smoke-r1/manifest.json`
- clean in Verilator
- clean in Yosys `synth -noabc`
- clean in the repo-owned Yosys with-ABC path
- metrics proving the realized tree directly:
  - `realized_min_leaf_depth = 2`
  - `realized_max_leaf_depth = 2`
  - `instance_slots_by_parent_depth = {0: 2, 1: 5}`
  - `min_child_instances_per_internal_module = 2`
- `max_child_instances_per_internal_module = 3`
- `hierarchy_parent_composed_outputs = 22`

Current HEAD now also has a focused clean proof for mixed-depth
recursive hierarchy:

- `/tmp/anvil-hier-mixed-depth-smoke-r1/manifest.json`
- clean in Verilator
- clean in Yosys `synth -noabc`
- clean in the repo-owned Yosys with-ABC path
- metrics proving the mixed tree directly:
  - `realized_min_leaf_depth = 2`
  - `realized_max_leaf_depth = 3`
  - `leaf_module_occurrences_by_depth = {"2": 2, "3": 4}`
  - `avg_child_instances_by_parent_depth = {"0": 2.0, "1": 2.0, "2": 2.0}`
  - `hierarchy_parent_composed_outputs = 40`
  - `top_parent_composed_outputs = 14`

Current HEAD also has a focused clean proof for depth-specific
branching in the recursive lane:

- `/tmp/anvil-hier-depth-profile-smoke-r1/manifest.json`
- clean in Verilator
- clean in Yosys `synth -noabc`
- clean in the repo-owned Yosys with-ABC path
- metrics proving the depth-specific branching profile directly:
  - `realized_min_leaf_depth = 2`
  - `realized_max_leaf_depth = 2`
  - `avg_child_instances_by_parent_depth = {"0": 4.0, "1": 2.0}`
  - `min_child_instances_by_parent_depth = {"0": 4, "1": 2}`
  - `max_child_instances_by_parent_depth = {"0": 4, "1": 2}`
  - `hierarchy_parent_composed_outputs = 36`
  - `top_parent_composed_outputs = 18`

## The next real steps

Phase 4 is now `in progress`, not `not started`. The next honest work
items are:

1. broaden helper-instance placement beyond the current
   parent-composed child-input, direct sibling, direct registered
   sibling, registered child-input, and parent-output slices;
2. deepen registered child-to-child routing using the local
   parent-state surface;
3. deepen the parent-side routing/composition surface beyond the
   current mixed-output, sibling-binding, parent-input-cone, and
   local-parent-flop surfaces.

Only after that does Phase 4 become "done" in the same sense that the
leaf-kernel phases are done today.
