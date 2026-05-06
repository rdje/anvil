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
  composition can spend multiple helpers from that budget directly, and
  can route the required helper source through parent-local state when
  `hierarchy_parent_flop_prob` is enabled. Parent-composed child-input
  cones can also register a required helper source first, then consume
  the helper Q in unregistered parent logic. If a required helper-backed
  unregistered child-input cone would otherwise lack parent-port
  dependencies, the generator can add a parent-port companion so the
  same binding proves helper and parent-port mixed support, including
  when the helper support is consumed through a parent-local helper Q;
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
route parent-composed helper child inputs through parent-local state,
mix helper output support with parent-port support in the same
unregistered parent-composed child-input binding, mix helper-through-state
support with parent-port support in the same unregistered
parent-composed child-input binding, and add local parent
flops. It still does not
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
| Let parent-composed helper child-input cones also mix parent data-port support | `hierarchy_child_input_cone_prob` + `hierarchy_parent_cone_instance_prob` | helper child output + parent port -> parent logic -> later child input | `child_input_bindings_from_parent_cone_instance_mixed_support`, `top_child_input_bindings_from_parent_cone_instance_mixed_support`, `parent_cone_instance_mixed_support_child_input_binding_fraction`, `top_parent_cone_instance_mixed_support_child_input_binding_fraction` |
| Let parent-output cones instantiate a helper child source | `hierarchy_parent_cone_instance_prob` | helper child output -> parent logic -> parent output | `top_outputs_reaching_parent_cone_instances`, `hierarchy_outputs_reaching_parent_cone_instances`, `top_parent_cone_instance_output_fraction`, `hierarchy_parent_cone_instance_output_fraction` |
| Let parent-output helper cones also mix parent data-port support | `hierarchy_parent_cone_instance_prob` | helper child output + parent port -> parent logic -> parent output | `top_outputs_reaching_parent_cone_instance_mixed_support`, `hierarchy_outputs_reaching_parent_cone_instance_mixed_support`, `top_parent_cone_instance_mixed_support_output_fraction`, `hierarchy_parent_cone_instance_mixed_support_output_fraction` |
| Route parent-output helper sources through parent-local state | `hierarchy_parent_cone_instance_prob` + `hierarchy_parent_flop_prob` | helper child output -> parent flop -> parent logic -> parent output | `top_outputs_reaching_parent_cone_instances_through_parent_flops`, `hierarchy_outputs_reaching_parent_cone_instances_through_parent_flops`, `top_parent_cone_instance_flop_output_fraction`, `hierarchy_parent_cone_instance_flop_output_fraction` |
| Route parent-composed helper child inputs through parent-local state | `hierarchy_child_input_cone_prob` + `hierarchy_parent_cone_instance_prob` + `hierarchy_parent_flop_prob` | helper child output -> parent flop -> parent logic -> later child input | `child_input_bindings_from_parent_cone_instances_through_parent_flops`, `top_child_input_bindings_from_parent_cone_instances_through_parent_flops`, `parent_cone_instance_flop_child_input_binding_fraction`, `top_parent_cone_instance_flop_child_input_binding_fraction` |
| Route parent-composed helper child inputs through parent-local state and also mix parent data-port support | `hierarchy_child_input_cone_prob` + `hierarchy_parent_cone_instance_prob` + `hierarchy_parent_flop_prob` | helper child output -> parent flop -> parent logic + parent port -> later child input | `child_input_bindings_from_parent_cone_instance_flop_mixed_support`, `top_child_input_bindings_from_parent_cone_instance_flop_mixed_support`, `parent_cone_instance_flop_mixed_support_child_input_binding_fraction`, `top_parent_cone_instance_flop_mixed_support_child_input_binding_fraction` |
| Allow more helper children per parent | `max_parent_cone_instances_per_module` | multiple helper child outputs -> parent logic | `max_parent_cone_instances_per_internal_module`, `top_parent_cone_instances`, `hierarchy_parent_cone_instances` |
| Bind later child inputs through one parent flop | `hierarchy_registered_sibling_route_prob` | earlier child output -> parent flop -> later child input | `child_input_bindings_from_registered_instance_outputs`, `registered_instance_output_child_input_binding_fraction`, `top_registered_instance_output_child_input_binding_fraction` |
| Chain direct registered sibling routes through earlier parent state | `hierarchy_registered_sibling_route_prob` | earlier child output -> parent flop -> later parent flop -> later child input | `child_input_bindings_from_registered_multistage_instance_outputs`, `top_child_input_bindings_from_registered_multistage_instance_outputs`, `registered_multistage_instance_output_child_input_binding_fraction`, `top_registered_multistage_instance_output_child_input_binding_fraction` |
| Let direct registered sibling routes use a helper child source | `hierarchy_registered_sibling_route_prob` + `hierarchy_parent_cone_instance_prob` | helper child output -> parent flop -> later child input | `child_input_bindings_from_registered_instance_outputs`, `child_input_bindings_from_registered_parent_cone_instances`, `registered_parent_cone_instance_child_input_binding_fraction`, `top_registered_parent_cone_instance_child_input_binding_fraction` |
| Chain direct registered sibling helper routes through earlier parent state | `hierarchy_registered_sibling_route_prob` + `hierarchy_parent_cone_instance_prob` | helper child output -> parent flop -> later parent flop -> later child input | `child_input_bindings_from_registered_multistage_parent_cone_instances`, `top_child_input_bindings_from_registered_multistage_parent_cone_instances`, `registered_multistage_parent_cone_instance_child_input_binding_fraction`, `top_registered_multistage_parent_cone_instance_child_input_binding_fraction` |
| Bind later child inputs through registered parent-composed logic | `hierarchy_registered_child_input_cone_prob` | parent source(s), optionally including earlier parent Q -> parent logic -> parent flop -> later child input | `child_input_bindings_from_registered_parent_composed_logic`, `registered_parent_composed_child_input_binding_fraction`, `child_input_bindings_from_registered_mixed_support`, `registered_mixed_support_child_input_binding_fraction`, `child_input_bindings_from_registered_multistage_parent_composed_logic`, `registered_multistage_parent_composed_child_input_binding_fraction` |
| Let registered parent-composed child-input D cones instantiate a helper child source | `hierarchy_registered_child_input_cone_prob` + `hierarchy_parent_cone_instance_prob` | helper child output -> parent logic -> parent flop -> later child input | `child_input_bindings_from_registered_parent_cone_instances`, `top_child_input_bindings_from_registered_parent_cone_instances`, `registered_parent_cone_instance_child_input_binding_fraction`, `top_registered_parent_cone_instance_child_input_binding_fraction` |
| Let registered parent-composed helper D cones also mix parent data-port support | `hierarchy_registered_child_input_cone_prob` + `hierarchy_parent_cone_instance_prob` | helper child output + parent port -> parent logic -> parent flop -> later child input | `child_input_bindings_from_registered_parent_cone_instance_mixed_support`, `top_child_input_bindings_from_registered_parent_cone_instance_mixed_support`, `registered_parent_cone_instance_mixed_support_child_input_binding_fraction`, `top_registered_parent_cone_instance_mixed_support_child_input_binding_fraction` |
| Chain registered parent-composed helper routes through earlier parent state | `hierarchy_registered_child_input_cone_prob` + `hierarchy_parent_cone_instance_prob` | helper child output -> parent flop -> later parent logic -> later parent flop -> later child input | `child_input_bindings_from_registered_multistage_parent_composed_parent_cone_instances`, `top_child_input_bindings_from_registered_multistage_parent_composed_parent_cone_instances`, `registered_multistage_parent_composed_parent_cone_instance_child_input_binding_fraction`, `top_registered_multistage_parent_composed_parent_cone_instance_child_input_binding_fraction` |
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

The direct registered helper route can chain in the same narrow form.
With a one-helper budget, the first registered sibling route can seed a
parent-local Q from a helper output, and later registered sibling routes
can reuse that Q as their next D source. The
`registered_multistage_parent_cone_instance_*` counters prove that
helper-sourced multi-stage route without conflating it with registered
parent-composed logic.

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
When `hierarchy_parent_flop_prob` is enabled, that required helper
source may be registered into parent-local state before the parent
output cone consumes it. The dedicated metrics
`top_outputs_reaching_parent_cone_instances_through_parent_flops` and
`hierarchy_outputs_reaching_parent_cone_instances_through_parent_flops`
prove that the output route passed through parent-local state rather
than only through combinational parent logic.

Parent-composed child-input helper routing has its own stateful lane.
When `hierarchy_child_input_cone_prob`,
`hierarchy_parent_cone_instance_prob`, and `hierarchy_parent_flop_prob`
are all active, a required helper output may first be registered into a
parent-local Q and then combined with the unregistered child-input
parent logic. This produces a different shape from registered
child-input routing: the final child binding is parent-composed logic,
not a flop Q. Use
`child_input_bindings_from_parent_cone_instances_through_parent_flops`
and `parent_cone_instance_flop_child_input_binding_fraction` to prove
that helper-through-state path without conflating it with
`child_input_bindings_from_registered_parent_cone_instances`.

The same stateful child-input helper route can also prove mixed parent
support. In that shape, the child input remains an unregistered
parent-composed binding, but the helper support enters through a
parent-local Q and the final binding also depends on a parent data port.
Use `child_input_bindings_from_parent_cone_instance_flop_mixed_support`
and `parent_cone_instance_flop_mixed_support_child_input_binding_fraction`
to prove that stricter helper-through-state plus parent-port overlap
without conflating it with the registered helper mixed-support route.

Unregistered parent-composed helper child-input routing also has a
mixed-support lane. When `hierarchy_child_input_cone_prob` and
`hierarchy_parent_cone_instance_prob` are active, a required helper
output can be combined with parent-port support in the final
parent-composed child-input binding without inserting a parent-local
flop. Use
`child_input_bindings_from_parent_cone_instance_mixed_support` and
`parent_cone_instance_mixed_support_child_input_binding_fraction` to
prove the helper output and parent-port support appeared in the same
unregistered binding. The registered helper mixed-support counters stay
separate and require the final binding to be a `FlopQ`.

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
When the registered parent-composed helper D cone also carries parent
data-port support, use
`child_input_bindings_from_registered_parent_cone_instance_mixed_support`
and
`registered_parent_cone_instance_mixed_support_child_input_binding_fraction`
to prove the helper source and parent-port support appeared in the same
registered D cone.

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
  `hierarchy_parent_port_composed_output_fraction`,
  `top_parent_port_composed_outputs_through_parent_flops`,
  `hierarchy_parent_port_composed_outputs_through_parent_flops`,
  `top_parent_port_composed_parent_flop_output_fraction`,
  `hierarchy_parent_port_composed_parent_flop_output_fraction`),
- whether top outputs actually depend on child outputs at all,
- average / maximum child-output support per top output,
- child-input provenance
  (`child_input_bindings_from_parent_ports`,
  `child_input_bindings_from_instance_outputs`,
  `child_input_bindings_from_mixed_support`,
  `top_child_input_bindings_from_mixed_support`,
  `child_input_bindings_from_constants`,
  `child_input_bindings_from_parent_composed_logic`,
  `child_input_bindings_from_parent_flops`,
  `child_input_bindings_from_registered_instance_outputs`,
  `child_input_bindings_from_registered_sibling_mixed_support`,
  `top_child_input_bindings_from_registered_sibling_mixed_support`,
  `child_input_bindings_from_registered_parent_composed_logic`,
  `child_input_bindings_from_registered_mixed_support`,
  `child_input_bindings_from_registered_multistage_parent_composed_logic`,
  `child_input_bindings_from_registered_multistage_instance_outputs`,
  `child_input_bindings_from_registered_multistage_parent_cone_instances`,
  `child_input_bindings_from_registered_multistage_parent_composed_parent_cone_instances`,
  `child_input_bindings_from_parent_cone_instances`,
  `child_input_bindings_from_parent_cone_instances_through_parent_flops`,
  `child_input_bindings_from_registered_parent_cone_instances`,
  `top_child_input_bindings_from_registered_parent_cone_instances`,
  `child_input_bindings_from_registered_parent_cone_instance_mixed_support`,
  `top_child_input_bindings_from_registered_parent_cone_instance_mixed_support`,
  `child_input_bindings_from_parent_cone_instance_mixed_support`,
  `top_child_input_bindings_from_parent_cone_instance_mixed_support`,
  `child_input_bindings_from_parent_cone_instance_flop_mixed_support`,
  `top_child_input_bindings_from_parent_cone_instance_flop_mixed_support`,
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
- hierarchy- and top-level direct registered sibling mixed-support
  route fractions
  (`registered_sibling_mixed_support_child_input_binding_fraction`,
  `top_registered_sibling_mixed_support_child_input_binding_fraction`),
- hierarchy- and top-level multi-stage registered sibling-route
  fractions
  (`registered_multistage_instance_output_child_input_binding_fraction`,
  `top_registered_multistage_instance_output_child_input_binding_fraction`),
- hierarchy- and top-level multi-stage registered helper-sourced route
  fractions
  (`registered_multistage_parent_cone_instance_child_input_binding_fraction`,
  `top_registered_multistage_parent_cone_instance_child_input_binding_fraction`),
- hierarchy- and top-level multi-stage registered parent-composed
  helper-sourced route fractions
  (`registered_multistage_parent_composed_parent_cone_instance_child_input_binding_fraction`,
  `top_registered_multistage_parent_composed_parent_cone_instance_child_input_binding_fraction`),
- hierarchy- and top-level registered parent-composed route fractions
  (`registered_parent_composed_child_input_binding_fraction`,
  `top_registered_parent_composed_child_input_binding_fraction`),
- hierarchy- and top-level registered mixed-support route fractions
  (`registered_mixed_support_child_input_binding_fraction`,
  `top_registered_mixed_support_child_input_binding_fraction`),
- hierarchy- and top-level multi-stage registered mixed-support route
  fractions
  (`registered_multistage_mixed_support_child_input_binding_fraction`,
  `top_registered_multistage_mixed_support_child_input_binding_fraction`),
- hierarchy- and top-level multi-stage registered parent-composed route
  fractions
  (`registered_multistage_parent_composed_child_input_binding_fraction`,
  `top_registered_multistage_parent_composed_child_input_binding_fraction`),
- hierarchy- and top-level parent-cone helper-instance route fractions
  (`parent_cone_instance_child_input_binding_fraction`,
  `top_parent_cone_instance_child_input_binding_fraction`),
- hierarchy- and top-level parent-cone helper mixed-support child-input
  fractions
  (`parent_cone_instance_mixed_support_child_input_binding_fraction`,
  `top_parent_cone_instance_mixed_support_child_input_binding_fraction`),
- hierarchy- and top-level parent-composed helper-through-state
  child-input fractions
  (`parent_cone_instance_flop_child_input_binding_fraction`,
  `top_parent_cone_instance_flop_child_input_binding_fraction`),
- hierarchy- and top-level parent-composed helper-through-state
  mixed-support child-input fractions
  (`parent_cone_instance_flop_mixed_support_child_input_binding_fraction`,
  `top_parent_cone_instance_flop_mixed_support_child_input_binding_fraction`),
- hierarchy- and top-level registered parent-cone helper route fractions
  (`registered_parent_cone_instance_child_input_binding_fraction`,
  `top_registered_parent_cone_instance_child_input_binding_fraction`),
- hierarchy- and top-level registered parent-cone helper mixed-support
  route fractions
  (`registered_parent_cone_instance_mixed_support_child_input_binding_fraction`,
  `top_registered_parent_cone_instance_mixed_support_child_input_binding_fraction`),
- hierarchy- and top-level parent-cone helper-instance output support
  (`hierarchy_outputs_reaching_parent_cone_instances`,
  `top_outputs_reaching_parent_cone_instances`,
  `hierarchy_parent_cone_instance_output_fraction`,
  `top_parent_cone_instance_output_fraction`,
  `hierarchy_outputs_reaching_parent_cone_instances_through_parent_flops`,
  `top_outputs_reaching_parent_cone_instances_through_parent_flops`,
  `hierarchy_parent_cone_instance_flop_output_fraction`,
  `top_parent_cone_instance_flop_output_fraction`,
  `hierarchy_outputs_reaching_parent_cone_instances_through_parent_flops_with_mixed_support`,
  `top_outputs_reaching_parent_cone_instances_through_parent_flops_with_mixed_support`,
  `hierarchy_parent_cone_instance_flop_mixed_support_output_fraction`,
  `top_parent_cone_instance_flop_mixed_support_output_fraction`),
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
hierarchy gate. The latest full downstream-clean bank is:

- `/tmp/anvil-tool-matrix-phase4-hierarchy-r76/tool_matrix_report.json`
- `177` scenarios
- `4` designs/scenario
- `708` total designs
- `coverage_gaps = []`
- `Verilator 708/0`
- `Yosys without-abc 708/0`
- `Yosys with-abc 708/0`
- `saw_recursive_multiple_parent_cone_instances_per_parent = true`
- `saw_recursive_multiple_parent_cone_instances_per_parent_child_inputs = true`
- `saw_recursive_multiple_parent_cone_instances_per_parent_through_flops = true`
- `saw_recursive_hierarchy_registered_mixed_support_routing = true`
- `saw_recursive_hierarchy_registered_multistage_routing = true`
- `saw_recursive_hierarchy_registered_multistage_mixed_support_routing = true`
- `saw_recursive_hierarchy_registered_multistage_sibling_routing = true`
- `saw_recursive_hierarchy_parent_cone_instance_flop_outputs = true`
- `saw_recursive_hierarchy_parent_cone_instance_outputs = true`
- `saw_recursive_hierarchy_parent_cone_instance_mixed_support_outputs = true`
- `saw_recursive_hierarchy_direct_sibling_parent_cone_instance_routing = true`
- `saw_recursive_hierarchy_direct_registered_sibling_parent_cone_instance_routing = true`
- `saw_recursive_hierarchy_registered_multistage_parent_cone_instance_routing = true`
- `saw_recursive_hierarchy_registered_multistage_parent_composed_parent_cone_instance_routing = true`
- `saw_recursive_hierarchy_registered_parent_composed_parent_cone_instance_routing = true`
- `saw_recursive_hierarchy_registered_parent_cone_instance_mixed_support_routing = true`
- `saw_recursive_hierarchy_parent_composed_parent_cone_instance_flop_routing = true`
- `saw_hierarchy_parent_cone_instance_flop_mixed_support_outputs = true`
- `saw_recursive_hierarchy_parent_cone_instance_flop_mixed_support_outputs = true`
- `saw_hierarchy_parent_cone_instance_mixed_support_routing = true`
- `saw_recursive_hierarchy_parent_cone_instance_mixed_support_routing = true`
- `saw_hierarchy_parent_composed_parent_cone_instance_flop_mixed_support_routing = true`
- `saw_recursive_hierarchy_parent_composed_parent_cone_instance_flop_mixed_support_routing = true`
- `saw_hierarchy_registered_sibling_mixed_support_routing = true`
- `saw_recursive_hierarchy_registered_sibling_mixed_support_routing = true`
- `saw_hierarchy_mixed_support_child_inputs = true`
- `saw_recursive_hierarchy_mixed_support_child_inputs = true`
- `saw_recursive_hierarchy_parent_port_composed_outputs = true`
- `saw_recursive_hierarchy_stateful_parent_port_composed_outputs = true`
- `saw_recursive_hierarchy_stateful_parent_composed_mixed_support_child_inputs = true`
- `saw_recursive_hierarchy_parent_local_flops = true`
- `saw_recursive_hierarchy_depth_3_parent_local_flops = true`
- `saw_recursive_hierarchy_depth_3_mixed_support_child_inputs = true`
- `saw_recursive_hierarchy_depth_3_parent_port_composed_outputs = true`
- `saw_recursive_hierarchy_depth_3_stateful_parent_port_composed_outputs = true`
- `saw_recursive_hierarchy_depth_3_stateful_parent_composed_mixed_support_child_inputs = true`
- `saw_recursive_hierarchy_depth_4_parent_local_flops = true`
- `saw_recursive_hierarchy_depth_4_mixed_support_child_inputs = true`
- `saw_recursive_hierarchy_depth_4_parent_port_composed_outputs = true`
- `saw_recursive_hierarchy_depth_4_stateful_parent_port_composed_outputs = true`
- `saw_recursive_hierarchy_depth_4_stateful_parent_composed_mixed_support_child_inputs = true`
- `saw_recursive_hierarchy_depth_5_parent_local_flops = true`
- `saw_recursive_hierarchy_depth_5_mixed_support_child_inputs = true`
- `saw_recursive_hierarchy_depth_5_parent_port_composed_outputs = true`
- `saw_recursive_hierarchy_depth_5_stateful_parent_port_composed_outputs = true`
- `saw_recursive_hierarchy_depth_5_stateful_parent_composed_mixed_support_child_inputs = true`
- `saw_recursive_hierarchy_depth_6_parent_local_flops = true`
- `saw_recursive_hierarchy_depth_6_mixed_support_child_inputs = true`
- `saw_recursive_hierarchy_depth_6_parent_port_composed_outputs = true`
- `saw_recursive_hierarchy_depth_6_stateful_parent_port_composed_outputs = true`

The `r76` bank extends the depth-6 axis to the stateful
parent-port-composed parent-output surface (r71's depth-5 territory,
r66's depth-4 territory, r61's depth-3 territory) at depth 6. The new
focus scenario `phase4_recur_d6_stateful_parent_port_composed_output`
per construction strategy uses `2,2` child-instance bounds with
parent-flop probability 1.0 and a 64-flop budget per module, isolating
the surface across five intermediate parent layers below the top while
still excluding helpers, sibling routing, registered routing, and
parent-composed child-input cones.

The earlier `r75` bank extends the depth-6 axis to the unregistered
parent-port-composed parent-output surface (r70's depth-5 territory,
r65's depth-4 territory, r60's depth-3 territory) at depth 6. The new
focus scenario `phase4_recur_d6_parent_port_composed_output` per
construction strategy uses `2,2` child-instance bounds, isolating the
surface across five intermediate parent layers below the top with no
helpers, no sibling routing, no registered routing, no parent-composed
child-input cones, and no parent-local state.

The earlier `r74` bank extends the depth-6 axis to the unregistered
parent-composed mixed-support child-input surface (r69's depth-5
territory, r64's depth-4 territory, r59's depth-3 territory) at
depth 6. The new focus scenario
`phase4_recur_d6_parent_composed_mixed_support_child_input` per
construction strategy uses `2,2` child-instance bounds — a calibration
choice (depths 3-5 used `4,4` for mixed-support cells; at depth 6 the
`4,4` case grew the design to 1365 internal module occurrences and
pushed the downstream-clean gate beyond a safe slice). The 2,2
calibration still proves the mixed-support surface at exact depth 6
across five intermediate parent layers below the top.

The earlier `r73` bank opened the depth-6 axis on top of the closed
depth-5 sweep, mirroring how r68 opened depth-5 above the closed
depth-4 sweep and r63 opened depth-4 above the closed depth-3 sweep.
The focus scenario `phase4_recur_d6_parent_state` per construction
strategy uses `min/max_hierarchy_depth = 6` and `2,2` child-instance
bounds, isolating the parent-flop surface across five intermediate
parent layers below the top.

The earlier `r72` bank closed the depth-5 sweep with the stateful unregistered
parent-composed mixed-support child-input surface (r67's depth-4
territory, r62's depth-3 territory) at depth 5. The new focus scenario
`phase4_recur_d5_stateful_parent_composed_mixed_support_child_input`
per construction strategy uses `4,4` child-instance bounds with
parent-composed child-input cones at probability 1.0, parent-flop
probability 1.0, and a 64-flop budget per module, isolating the
surface across four intermediate parent layers below the top while
still excluding helpers, sibling routing, and registered routing. The
depth-5 axis now has all five mixed-support cells gated as first-class
coverage facts, mirroring the closed depth-3 (r58..r62) and depth-4
(r63..r67) sweeps.

The earlier `r71` bank extended the depth-5 axis to the stateful
parent-port-composed parent-output surface (r66's depth-4 territory,
r61's depth-3 territory) at depth 5. The focus scenario
`phase4_recur_d5_stateful_parent_port_composed_output` per construction
strategy uses `2,2` child-instance bounds with parent-flop probability
1.0 and a 64-flop budget per module, isolating the surface across four
intermediate parent layers below the top while still excluding helpers,
sibling routing, registered routing, and parent-composed child-input
cones.

The earlier `r70` bank extended the depth-5 axis to the unregistered
parent-port-composed parent-output surface (r65's depth-4 territory,
r60's depth-3 territory) at depth 5. The focus scenario
`phase4_recur_d5_parent_port_composed_output` per construction strategy
uses `2,2` child-instance bounds, isolating the surface across four
intermediate parent layers below the top with no helpers, no sibling
routing, no registered routing, no parent-composed child-input cones,
and no parent-local state.

The earlier `r69` bank extended the depth-5 axis to the unregistered
parent-composed mixed-support child-input surface (r64's depth-4
territory) at depth 5. The focus scenario
`phase4_recur_d5_parent_composed_mixed_support_child_input` per
construction strategy uses `4,4` child-instance bounds, isolating the
surface across four intermediate parent layers below the top.

The earlier `r68` bank opened the depth-5 axis on top of the completed
depth-4 sweep. The new focus scenario `phase4_recur_d5_parent_state` per
construction strategy uses `min/max_hierarchy_depth = 5` and `2,2`
child-instance bounds, isolating the parent-flop surface across four
intermediate parent layers below the top. It carries forward the
entire depth-4 sweep (r63–r67), the depth-3 sweep (r58–r62), the
underlying depth-2 mixed-support proofs, and the helper-route proofs.

The earlier `r67` bank closed the depth-4 sweep by extending coverage
to the stateful unregistered parent-composed mixed-support child-input
surface (r62's depth-3 territory) at depth 4. The new focus scenario
`phase4_recur_d4_stateful_parent_composed_mixed_support_child_input`
per construction strategy uses `4,4` child-instance bounds and
`hierarchy_parent_flop_prob = 1.0`, forcing unregistered parent-composed
child-input cones that mix parent ports + child outputs + parent-local
Qs across three intermediate parent layers below the top. The depth-4
sweep is now complete across all five mixed-support cells:
parent-flops (r63), no-state child-input mixed-support (r64), no-state
parent-output mixed-support (r65), stateful parent-output mixed-support
(r66), and stateful child-input mixed-support (r67).

The earlier `r66` bank extended the depth-4 axis to the stateful
parent-port-composed parent-output surface (r61's depth-3 territory)
at depth 4. The new focus scenario
`phase4_recur_d4_stateful_parent_port_composed_output` per construction
strategy uses `2,2` child-instance bounds and `hierarchy_parent_flop_prob = 1.0`,
forcing parent-output cones that mix parent ports + child outputs +
parent-local Qs across three intermediate parent layers below the top.

The earlier `r65` bank extended the depth-4 axis to the
parent-port-composed parent-output surface (r60's depth-3 territory)
at depth 4. The new
focus scenario `phase4_recur_d4_parent_port_composed_output` per
construction strategy uses `2,2` child-instance bounds, isolating
parent-output cones across three intermediate parent layers below the
top.

The earlier `r64` bank extended the depth-4 axis to the unregistered
parent-composed mixed-support child-input surface (r59's depth-3
territory) at depth 4. The new focus scenario
`phase4_recur_d4_parent_composed_mixed_support_child_input` per
construction strategy uses `4,4` child-instance bounds, isolating the
surface across three intermediate parent layers below the top.

The earlier `r63` bank opened the depth-4 axis on top of the completed
depth-3 sweep. The new focus scenario `phase4_recur_d4_parent_state` per
construction strategy uses `min/max_hierarchy_depth = 4` and `2,2`
child-instance bounds, isolating the parent-flop surface across three
intermediate parent layers below the top. It carries forward the entire
depth-3 sweep (r58–r62), the underlying depth-2 mixed-support proofs,
and the helper-route proofs.

The earlier `r62` bank closed the depth-3 push by pushing the recursive
non-top stateful parent-composed mixed-support child-input surface
(r56's depth-2 territory) to exact hierarchy depth 3 without helpers. The new
focus scenario `phase4_recur_d3_stateful_parent_composed_mixed_support_child_input`
per construction strategy uses `4,4` child-instance bounds and
`hierarchy_parent_flop_prob = 1.0`, forcing unregistered parent-composed
child-input cones that mix parent ports + child outputs + parent-local
Qs across two intermediate parent layers below the top without helpers,
sibling routing, or registered routing. It carries forward the r61
depth-3 stateful parent-port-composed gating, the r60 depth-3 no-state
parent-port-composed gating, the r59 depth-3 mixed-support child-input
gating, the r58 depth-3 parent-flop gating, and the underlying
depth-2 mixed-support proofs. The depth-3 push is now complete across
all four mixed-support cells.

The earlier `r61` bank pushed the recursive non-top stateful
parent-port-composed parent-output surface (r55's depth-2 territory) to
exact hierarchy depth 3 without helpers. The new focus scenario
`phase4_recur_d3_stateful_parent_port_composed_output` per construction
strategy uses `2,2` child-instance bounds and `hierarchy_parent_flop_prob = 1.0`,
forcing parent-output cones that mix parent ports + child outputs +
parent-local Qs across two intermediate parent layers below the top
without helpers, sibling routing, registered routing, or
parent-composed child-input cones. It carries forward the r60 depth-3
no-state parent-port-composed gating, the r59 depth-3 mixed-support
child-input gating, the r58 depth-3 parent-flop gating, and the
underlying depth-2 mixed-support proofs.

The earlier `r60` bank pushed the recursive non-top parent-port-composed
parent-output surface (r54's depth-2 territory) to exact hierarchy
depth 3 without helpers or state. The new focus scenario
`phase4_recur_d3_parent_port_composed_output` per construction strategy
uses `2,2` child-instance bounds and forces parent-output cones across
two intermediate parent layers below the top with helpers, sibling
routing, registered routing, parent-composed child-input cones, and
parent-local flops all disabled. It carries forward the r59 depth-3
mixed-support child-input gating, the r58 depth-3 parent-flop gating,
the r57 first-class parent-local-flop gating, and the underlying
depth-2 mixed-support proofs.

The earlier `r59` bank pushed the recursive non-top unregistered
parent-composed mixed-support child-input surface from exact depth 2
(r53) to exact depth 3 without helpers. The new focus scenario
`phase4_recur_d3_parent_composed_mixed_support_child_input` per
construction strategy uses `4,4` child-instance bounds (distinct from
r58's depth-3 / `2,2` parent-state shape) and forces hierarchy depth 3
across two intermediate parent layers below the top with mixed-support
child-input cones enabled. It carries forward the r58 depth-3
parent-flop gating, the r57 first-class parent-local-flop gating, the
r56 stateful parent-composed mixed-support child-input proof, and the
r53/r54/r55 underlying mixed-support proofs.

The earlier `r58` bank pushed the recursive parent-state surface from
exact depth 2 to exact depth 3. The new focus scenario
`phase4_recur_d3_parent_state` per construction strategy uses `2,2`
child-instance bounds (distinct from r57's depth-2 / `4,4` shape) and
forces hierarchy depth 3 across two intermediate parent layers below the
top. It carries forward the r57 first-class parent-local-flop gating,
the r56 stateful parent-composed mixed-support child-input proof, the
r55 stateful parent-port-composed output proof, and the r53 / r54
no-helper parent-composed and parent-output mixed-support proofs
underneath.

The earlier `r57` bank promoted recursive non-top parent-local flops to
a first-class gated coverage fact, with a dedicated focus scenario
`phase4_recur_d2_parent_state` per construction strategy that isolates
the parent-flop surface (no helpers, no sibling routing, no
parent-composed child-input cones).

The earlier `r56` bank extended the accumulated mixed-support evidence with
recursive non-top unregistered parent-composed child-input cones that
mix parent data ports, child outputs, and parent-local Qs without helper
instances, while carrying forward the r55 stateful parent-port-composed
parent-output proof, the r54 no-state parent-port-composed output proof,
and the r53 unregistered parent-composed mixed-support child-input
proof.
The earlier `r50` bank promoted
the recent coverage-only mixed-support probes into full downstream-clean
evidence: stateful helper-backed parent outputs that also carry
parent-port support, unregistered parent-composed helper child-input
bindings that also carry parent-port support, and stateful
helper-through-flop child-input bindings that also carry parent-port
support. The coverage-only reports remain focused
policy breadcrumbs.

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
sibling-routed child-input bindings, direct registered sibling
mixed-support child-input bindings, recursive non-top direct registered
sibling mixed-support child-input bindings, registered parent-composed
child-input bindings, registered mixed-support child-input bindings
that mix parent ports with child outputs, recursive non-top registered
mixed-support child-input bindings below the top parent, multi-stage registered
parent-composed child-input bindings that chain through earlier
parent-local Qs, recursive non-top multi-stage registered
parent-composed child-input bindings that chain through earlier
parent-local Qs below the top parent without helper instances,
multi-stage registered sibling-routed child-input
bindings that chain through earlier parent-local Qs without
parent-composed logic, recursive non-top multi-stage registered
sibling-routed child-input bindings that chain through earlier
parent-local Qs below the top parent without helper instances,
recursive non-top multi-stage registered mixed-support child-input
bindings that combine parent ports, child outputs, and earlier
parent-local Qs below the top parent without helper instances,
real local parent flops, parent-cone helper instances
sourcing parent-composed child-input bindings, parent-output helper
instance composition, recursive non-top parent-output helper routing,
recursive non-top parent-output multi-helper budget evidence,
recursive non-top child-input multi-helper budget evidence,
recursive non-top stateful multi-helper budget evidence,
stateful parent-output helper routing through parent-local flops,
stateful parent-output helper routing through parent-local flops with
mixed parent-port support, stateful parent-composed helper child-input routing
through parent-local flops, stateful parent-composed helper child-input
mixed-support routing through parent-local flops, recursive non-top direct sibling helper
routing, budgeted multi-helper allocation, registered
parent-composed helper-sourced child-input D cones, direct sibling
helper routing, direct registered sibling helper routing, and
multi-stage direct registered sibling helper routing, plus unregistered
parent-composed mixed-support child-input bindings without helpers, plus
multi-stage registered parent-composed helper routing, recursive non-top
multi-stage direct registered sibling helper routing, recursive non-top
multi-stage registered parent-composed helper routing, and recursive non-top
registered parent-composed helper mixed-support routing, unregistered
parent-composed helper child-input mixed-support routing, and recursive non-top
stateful parent-composed helper child-input routing through
parent-local flops, plus recursive non-top multi-stage registered
parent-composed no-helper routing, plus recursive non-top multi-stage
registered sibling no-helper routing, plus recursive non-top
multi-stage registered mixed-support no-helper routing. The earlier coverage-only proofs at
`/tmp/anvil-tool-matrix-phase4-recursive-direct-helper-r32/tool_matrix_report.json`
and
`/tmp/anvil-tool-matrix-phase4-recursive-helper-state-r31/tool_matrix_report.json`
are now historical policy breadcrumbs. The older `r21` bank remains
historical pre-parent-output-helper evidence;
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
- `cargo test recursive_hierarchy_parent_outputs_can_depend_on_helper_instances_below_top`
  proves parent-output helper routing below the top parent in an
  exact-depth-2 recursive hierarchy:
  - `realized_min_leaf_depth = realized_max_leaf_depth = 2`
  - `hierarchy_parent_cone_instances > top_parent_cone_instances`
  - `hierarchy_outputs_reaching_parent_cone_instances > top_outputs_reaching_parent_cone_instances`
  - `child_input_bindings_from_parent_cone_instances = 0`
  - `hierarchy_outputs_reaching_parent_cone_instances_through_parent_flops = 0`
  This route is banked in the full downstream-clean `r45` Phase 4
  matrix through the dedicated
  `phase4_recur_d2_parent_output_cone_instance` scenario.
- `cargo test recursive_hierarchy_parent_outputs_mix_helper_instances_with_parent_ports_below_top`
  proves recursive non-top parent-output helper routing can also mix
  parent data-port support in the same helper-backed output cone:
  - `realized_min_leaf_depth = realized_max_leaf_depth = 2`
  - `hierarchy_parent_cone_instances > top_parent_cone_instances`
  - `hierarchy_outputs_reaching_parent_cone_instances > top_outputs_reaching_parent_cone_instances`
  - `hierarchy_outputs_reaching_parent_cone_instance_mixed_support > top_outputs_reaching_parent_cone_instance_mixed_support`
  - `child_input_bindings_from_parent_cone_instances = 0`
  - `child_input_bindings_from_registered_parent_cone_instances = 0`
  - `hierarchy_outputs_reaching_parent_cone_instances_through_parent_flops = 0`
  The route is banked in the full downstream-clean `r49` Phase 4 matrix
  through
  `saw_recursive_hierarchy_parent_cone_instance_mixed_support_outputs = true`.
- `cargo test hierarchy_parent_outputs_can_route_helper_instances_through_parent_flops`
  proves the stateful parent-output helper route numerically:
  - `top_outputs_reaching_parent_cone_instances_through_parent_flops > 0`
  - `hierarchy_outputs_reaching_parent_cone_instances_through_parent_flops > 0`
  - `top_parent_cone_instance_flop_output_fraction > 0.0`
  - `hierarchy_parent_cone_instance_flop_output_fraction > 0.0`
  - `child_input_bindings_from_parent_cone_instances = 0`
- `cargo test recursive_hierarchy_parent_outputs_can_route_helper_instances_through_parent_flops_below_top`
  proves stateful parent-output helper routing below the top parent in
  an exact-depth-2 recursive hierarchy:
  - `realized_min_leaf_depth = realized_max_leaf_depth = 2`
  - `hierarchy_parent_cone_instances > top_parent_cone_instances`
  - `hierarchy_parent_local_flops > top_local_flops`
  - `hierarchy_outputs_reaching_parent_cone_instances_through_parent_flops > top_outputs_reaching_parent_cone_instances_through_parent_flops`
  - `child_input_bindings_from_parent_cone_instances = 0`
  - `child_input_bindings_from_registered_parent_cone_instances = 0`
  This route is banked in the full downstream-clean `r45` Phase 4
  matrix through the dedicated
  `phase4_recur_d2_parent_output_cone_instance_state` scenario.
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
- `cargo test recursive_hierarchy_parent_outputs_can_spend_helper_budget_below_top`
  proves budgeted parent-output-only helper composition below the top
  parent in an exact-depth-2 recursive hierarchy:
  - `realized_min_leaf_depth = realized_max_leaf_depth = 2`
  - `max_parent_cone_instances_per_internal_module = 3`
  - `top_parent_cone_instances = 3`
  - `hierarchy_parent_cone_instances > top_parent_cone_instances`
  - `hierarchy_outputs_reaching_parent_cone_instances > top_outputs_reaching_parent_cone_instances`
  - `child_input_bindings_from_parent_cone_instances = 0`
  - `child_input_bindings_from_registered_parent_cone_instances = 0`
  This policy fact is banked in the full downstream-clean `r45` Phase 4
  matrix as `saw_recursive_multiple_parent_cone_instances_per_parent`.
- `cargo test recursive_hierarchy_parent_cone_helper_budget_allows_multiple_helpers_below_top`
  proves budgeted parent-composed child-input helper routing below the
  top parent in an exact-depth-2 recursive hierarchy:
  - `realized_min_leaf_depth = realized_max_leaf_depth = 2`
  - `max_parent_cone_instances_per_internal_module = 3`
  - `top_parent_cone_instances = 3`
  - `hierarchy_parent_cone_instances > top_parent_cone_instances`
  - `child_input_bindings_from_parent_composed_logic > top_child_input_bindings_from_parent_composed_logic`
  - `child_input_bindings_from_parent_cone_instances > top_child_input_bindings_from_parent_cone_instances`
  - `child_input_bindings_from_parent_cone_instances_through_parent_flops = 0`
  - `child_input_bindings_from_registered_parent_cone_instances = 0`
  This policy fact is banked in the full downstream-clean `r45` Phase 4
  matrix as
  `saw_recursive_multiple_parent_cone_instances_per_parent_child_inputs`.
- `cargo test metrics::tests::design_metrics_capture_multiple_parent_cone_instance_budget`
  now also proves unregistered parent-composed helper child-input mixed
  support numerically in the budgeted helper case:
  - `child_input_bindings_from_parent_cone_instance_mixed_support > 0`
  - `top_child_input_bindings_from_parent_cone_instance_mixed_support > 0`
  - `parent_cone_instance_mixed_support_child_input_binding_fraction > 0.0`
  - `top_parent_cone_instance_mixed_support_child_input_binding_fraction > 0.0`
  - `child_input_bindings_from_registered_parent_cone_instances = 0`
  - `child_input_bindings_from_parent_cone_instances_through_parent_flops = 0`
  The coverage-only Phase 4 matrix probe at
  `/tmp/anvil-tool-matrix-phase4-parent-helper-child-input-mixed-check/tool_matrix_report.json`
  first recorded this as
  `saw_hierarchy_parent_cone_instance_mixed_support_routing = true` and
  `saw_recursive_hierarchy_parent_cone_instance_mixed_support_routing = true`;
  the full downstream-clean `r50` bank now carries the same facts through
  Verilator and both repo-owned Yosys modes.
- `cargo test metrics::tests::design_metrics_capture_parent_composed_parent_cone_instance_flop_routes`
  now also proves stateful parent-composed helper child-input mixed
  support numerically in the unregistered helper-through-flop route:
  - `child_input_bindings_from_parent_cone_instance_flop_mixed_support > 0`
  - `top_child_input_bindings_from_parent_cone_instance_flop_mixed_support > 0`
  - `parent_cone_instance_flop_mixed_support_child_input_binding_fraction > 0.0`
  - `top_parent_cone_instance_flop_mixed_support_child_input_binding_fraction > 0.0`
  - `child_input_bindings_from_registered_parent_cone_instances = 0`
  The coverage-only Phase 4 matrix probe at
  `/tmp/anvil-tool-matrix-phase4-stateful-helper-child-input-mixed-check/tool_matrix_report.json`
  first recorded this as
  `saw_hierarchy_parent_composed_parent_cone_instance_flop_mixed_support_routing = true`
  and
  `saw_recursive_hierarchy_parent_composed_parent_cone_instance_flop_mixed_support_routing = true`;
  the full downstream-clean `r50` bank now carries the same facts through
  Verilator and both repo-owned Yosys modes.
- `cargo test recursive_hierarchy_parent_outputs_can_spend_stateful_helper_budget_below_top`
  proves budgeted stateful parent-output helper composition below the
  top parent in an exact-depth-2 recursive hierarchy:
  - `realized_min_leaf_depth = realized_max_leaf_depth = 2`
  - `max_parent_cone_instances_per_internal_module = 3`
  - `top_parent_cone_instances = 3`
  - `hierarchy_parent_cone_instances > top_parent_cone_instances`
  - `hierarchy_parent_local_flops > top_local_flops`
  - `hierarchy_outputs_reaching_parent_cone_instances_through_parent_flops > top_outputs_reaching_parent_cone_instances_through_parent_flops`
  - `child_input_bindings_from_parent_cone_instances = 0`
  - `child_input_bindings_from_parent_cone_instances_through_parent_flops = 0`
  - `child_input_bindings_from_registered_parent_cone_instances = 0`
  This policy fact is banked in the full downstream-clean `r45` Phase 4
  matrix as
  `saw_recursive_multiple_parent_cone_instances_per_parent_through_flops`.
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
  This route is now also banked in the full downstream-clean `r34`
  Phase 4 matrix.
- `cargo test recursive_hierarchy_sibling_routes_can_use_helper_instances_below_top`
  proves direct sibling helper routing below the top parent in an
  exact-depth-2 recursive hierarchy:
  - `realized_min_leaf_depth = realized_max_leaf_depth = 2`
  - `hierarchy_parent_cone_instances > top_parent_cone_instances`
  - `child_input_bindings_from_parent_cone_instances > top_child_input_bindings_from_parent_cone_instances`
  - `child_input_bindings_from_registered_parent_cone_instances = 0`
  - `child_input_bindings_from_registered_multistage_parent_cone_instances = 0`
  This route is banked in the full downstream-clean `r45` Phase 4
  matrix through the dedicated
  `phase4_recur_d2_direct_sibling_parent_cone_instance` scenario.
- `cargo test recursive_hierarchy_registered_sibling_routes_can_use_helper_instances_below_top`
  proves direct registered sibling helper routing below the top parent
  in an exact-depth-2 recursive hierarchy:
  - `realized_min_leaf_depth = realized_max_leaf_depth = 2`
  - `hierarchy_parent_cone_instances > top_parent_cone_instances`
  - `hierarchy_parent_local_flops > top_local_flops`
  - `child_input_bindings_from_registered_instance_outputs > top_child_input_bindings_from_registered_instance_outputs`
  - `child_input_bindings_from_registered_parent_cone_instances > top_child_input_bindings_from_registered_parent_cone_instances`
  - `child_input_bindings_from_registered_parent_composed_logic = 0`
  This route is banked in the full downstream-clean `r45` Phase 4
  matrix through the dedicated
  `phase4_recur_d2_direct_registered_sibling_parent_cone_instance_state`
  scenario.
- `cargo test recursive_hierarchy_registered_sibling_routes_can_chain_helper_instances_below_top`
  proves multi-stage direct registered sibling helper routing below the
  top parent in an exact-depth-2 recursive hierarchy:
  - `realized_min_leaf_depth = realized_max_leaf_depth = 2`
  - `hierarchy_parent_cone_instances > top_parent_cone_instances`
  - `hierarchy_parent_local_flops > top_local_flops`
  - `child_input_bindings_from_registered_multistage_instance_outputs > top_child_input_bindings_from_registered_multistage_instance_outputs`
  - `child_input_bindings_from_registered_multistage_parent_cone_instances > top_child_input_bindings_from_registered_multistage_parent_cone_instances`
  - `child_input_bindings_from_registered_parent_composed_logic = 0`
  - `child_input_bindings_from_registered_multistage_parent_composed_logic = 0`
  This route is banked in the full downstream-clean `r45` Phase 4
  matrix through the dedicated
  `phase4_recur_d2_registered_sibling_parent_cone_instance_multistage_state`
  scenario.
- `cargo test recursive_hierarchy_registered_parent_composed_routes_can_chain_helper_instances_below_top`
  proves multi-stage registered parent-composed helper routing below the
  top parent in an exact-depth-2 recursive hierarchy:
  - `realized_min_leaf_depth = realized_max_leaf_depth = 2`
  - `hierarchy_parent_cone_instances > top_parent_cone_instances`
  - `hierarchy_parent_local_flops > top_local_flops`
  - `child_input_bindings_from_registered_multistage_parent_composed_logic > top_child_input_bindings_from_registered_multistage_parent_composed_logic`
  - `child_input_bindings_from_registered_multistage_parent_composed_parent_cone_instances > top_child_input_bindings_from_registered_multistage_parent_composed_parent_cone_instances`
  - `child_input_bindings_from_registered_multistage_parent_cone_instances = 0`
  This route is banked in the full downstream-clean `r45` Phase 4
  matrix through the dedicated
  `phase4_recur_d2_registered_parent_cone_instance_multistage_state`
  scenario.
- `cargo test recursive_hierarchy_registered_child_input_cones_can_use_helper_instances_below_top`
  proves registered parent-composed helper D-cone routing below the top
  parent in an exact-depth-2 recursive hierarchy:
  - `realized_min_leaf_depth = realized_max_leaf_depth = 2`
  - `hierarchy_parent_cone_instances > top_parent_cone_instances`
  - `hierarchy_parent_local_flops > top_local_flops`
  - `child_input_bindings_from_registered_parent_composed_logic > top_child_input_bindings_from_registered_parent_composed_logic`
  - `child_input_bindings_from_registered_parent_cone_instances > top_child_input_bindings_from_registered_parent_cone_instances`
  This route is banked in the full downstream-clean `r45` Phase 4
  matrix through the dedicated
  `phase4_recur_d2_registered_parent_cone_instance_state` scenario.
- `cargo test hierarchy_registered_sibling_routes_can_use_helper_instances`
  proves direct registered sibling helper routing numerically:
  - `top_parent_cone_instances > 0`
  - `child_input_bindings_from_registered_instance_outputs > 0`
  - `child_input_bindings_from_registered_parent_composed_logic = 0`
  - `child_input_bindings_from_registered_parent_cone_instances > 0`
  - `registered_parent_cone_instance_child_input_binding_fraction > 0.0`
  - `num_instances > planned_child_instances`
  This route is now also banked in the full downstream-clean `r30`
  Phase 4 matrix.
- `cargo test hierarchy_registered_sibling_routes_can_chain_through_parent_flops`
  proves multi-stage direct registered sibling routing numerically:
  - `child_input_bindings_from_registered_instance_outputs > 0`
  - `child_input_bindings_from_registered_multistage_instance_outputs > 0`
  - `top_child_input_bindings_from_registered_multistage_instance_outputs > 0`
  - `child_input_bindings_from_registered_parent_composed_logic = 0`
  - `registered_multistage_instance_output_child_input_binding_fraction > 0.0`
  This route is banked in the full downstream-clean `r30` Phase 4
  matrix through the dedicated
  `phase4_hier2_inst4_registered_sibling_multistage_state` scenario.
- `cargo test hierarchy_registered_sibling_routes_can_chain_helper_instances_through_parent_flops`
  proves multi-stage direct registered sibling helper routing
  numerically:
  - `child_input_bindings_from_registered_multistage_parent_cone_instances > 0`
  - `top_child_input_bindings_from_registered_multistage_parent_cone_instances > 0`
  - `registered_multistage_parent_cone_instance_child_input_binding_fraction > 0.0`
  - `child_input_bindings_from_registered_parent_composed_logic = 0`
  - `child_input_bindings_from_registered_multistage_parent_composed_logic = 0`
  This route is banked in the full downstream-clean `r30` Phase 4
  matrix through the dedicated
  `phase4_hier2_inst4_registered_sibling_parent_cone_instance_multistage_state`
  scenario.
- `cargo test hierarchy_registered_parent_composed_routes_can_chain_helper_instances_through_parent_flops`
  proves multi-stage registered parent-composed helper routing
  numerically:
  - `child_input_bindings_from_registered_multistage_parent_composed_parent_cone_instances > 0`
  - `top_child_input_bindings_from_registered_multistage_parent_composed_parent_cone_instances > 0`
  - `registered_multistage_parent_composed_parent_cone_instance_child_input_binding_fraction > 0.0`
  - `child_input_bindings_from_registered_parent_composed_logic > 0`
  - `child_input_bindings_from_registered_multistage_parent_cone_instances = 0`
  This route is banked in the full downstream-clean `r30` Phase 4
  matrix through the dedicated
  `phase4_hier2_inst4_registered_parent_cone_instance_multistage_state`
  scenario.
- `cargo test hierarchy_parent_composed_helper_routes_can_use_parent_flops`
  proves stateful parent-composed helper child-input routing
  numerically:
  - `child_input_bindings_from_parent_cone_instances_through_parent_flops > 0`
  - `top_child_input_bindings_from_parent_cone_instances_through_parent_flops > 0`
  - `parent_cone_instance_flop_child_input_binding_fraction > 0.0`
  - `top_parent_cone_instance_flop_child_input_binding_fraction > 0.0`
  - `child_input_bindings_from_registered_parent_cone_instances = 0`
  This route is banked in the full downstream-clean `r30` Phase 4
  matrix through the dedicated
  `phase4_hier2_inst4_parent_cone_instance_state` scenario.
- `cargo test recursive_hierarchy_parent_composed_helper_routes_can_use_parent_flops_below_top`
  proves the same stateful parent-composed helper child-input route
  below the top parent in an exact-depth-2 recursive hierarchy:
  - `realized_min_leaf_depth = realized_max_leaf_depth = 2`
  - `hierarchy_parent_cone_instances > top_parent_cone_instances`
  - `hierarchy_parent_local_flops > top_local_flops`
  - `child_input_bindings_from_parent_cone_instances_through_parent_flops > top_child_input_bindings_from_parent_cone_instances_through_parent_flops`
  This route is banked in the full downstream-clean `r34` Phase 4
  matrix through the dedicated
  `phase4_recur_d2_parent_cone_instance_state` scenario.
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
  skipped Verilator/Yosys; the full downstream-clean `r30` bank now
  carries the same fact with real tool validation.
- `cargo test recursive_hierarchy_registered_mixed_support_routes_below_top`
  proves the same registered mixed-support route below the top parent
  in an exact-depth-2 recursive hierarchy without helper instances:
  - `realized_min_leaf_depth = realized_max_leaf_depth = 2`
  - `hierarchy_parent_local_flops > top_local_flops`
  - `child_input_bindings_from_registered_parent_composed_logic > top_child_input_bindings_from_registered_parent_composed_logic`
  - `child_input_bindings_from_registered_instance_outputs > top_child_input_bindings_from_registered_instance_outputs`
  - `child_input_bindings_from_registered_mixed_support > top_child_input_bindings_from_registered_mixed_support`
  - `child_input_bindings_from_registered_parent_cone_instances = 0`
  The route is banked in the full downstream-clean `r45` Phase 4 matrix
  through
  `saw_recursive_hierarchy_registered_mixed_support_routing = true`.
- `cargo test recursive_hierarchy_registered_parent_composed_routes_can_chain_without_helpers_below_top`
  proves registered parent-composed child-input routes can chain through
  earlier parent-local Qs below the top parent without helper
  instances:
  - `realized_min_leaf_depth = realized_max_leaf_depth = 2`
  - `hierarchy_parent_local_flops > top_local_flops`
  - `child_input_bindings_from_registered_parent_composed_logic > top_child_input_bindings_from_registered_parent_composed_logic`
  - `child_input_bindings_from_registered_multistage_parent_composed_logic > top_child_input_bindings_from_registered_multistage_parent_composed_logic`
  - `registered_multistage_parent_composed_child_input_binding_fraction > 0.0`
  - `child_input_bindings_from_registered_parent_cone_instances = 0`
  - `child_input_bindings_from_registered_multistage_parent_cone_instances = 0`
  - `child_input_bindings_from_registered_multistage_parent_composed_parent_cone_instances = 0`
  The route is banked in the full downstream-clean `r45` Phase 4 matrix
  through
  `saw_recursive_hierarchy_registered_multistage_routing = true`.
- `cargo test recursive_hierarchy_registered_sibling_routes_can_chain_without_helpers_below_top`
  proves direct registered sibling-routed child-input routes can chain
  through earlier parent-local Qs below the top parent without helper
  instances or parent-composed D logic:
  - `realized_min_leaf_depth = realized_max_leaf_depth = 2`
  - `hierarchy_parent_local_flops > top_local_flops`
  - `child_input_bindings_from_registered_instance_outputs > top_child_input_bindings_from_registered_instance_outputs`
  - `child_input_bindings_from_registered_multistage_instance_outputs > top_child_input_bindings_from_registered_multistage_instance_outputs`
  - `registered_multistage_instance_output_child_input_binding_fraction > 0.0`
  - `child_input_bindings_from_registered_parent_composed_logic = 0`
  - `child_input_bindings_from_registered_multistage_parent_composed_logic = 0`
  - `child_input_bindings_from_registered_parent_cone_instances = 0`
  - `child_input_bindings_from_registered_multistage_parent_cone_instances = 0`
  The route is banked in the full downstream-clean `r46` Phase 4 matrix
  through
  `saw_recursive_hierarchy_registered_multistage_sibling_routing = true`.
- `cargo test recursive_hierarchy_registered_multistage_mixed_support_routes_below_top`
  proves registered parent-composed child-input routes can combine
  parent ports, child outputs, and earlier parent-local Qs below the
  top parent without helper instances:
  - `realized_min_leaf_depth = realized_max_leaf_depth = 2`
  - `hierarchy_parent_local_flops > top_local_flops`
  - `child_input_bindings_from_registered_parent_composed_logic > top_child_input_bindings_from_registered_parent_composed_logic`
  - `child_input_bindings_from_registered_mixed_support > top_child_input_bindings_from_registered_mixed_support`
  - `child_input_bindings_from_registered_multistage_parent_composed_logic > top_child_input_bindings_from_registered_multistage_parent_composed_logic`
  - `child_input_bindings_from_registered_multistage_mixed_support > top_child_input_bindings_from_registered_multistage_mixed_support`
  - `registered_multistage_mixed_support_child_input_binding_fraction > 0.0`
  - `child_input_bindings_from_registered_parent_cone_instances = 0`
  - `child_input_bindings_from_registered_multistage_parent_cone_instances = 0`
  - `child_input_bindings_from_registered_multistage_parent_composed_parent_cone_instances = 0`
  The route is banked in the full downstream-clean `r47` Phase 4 matrix
  through
  `saw_recursive_hierarchy_registered_multistage_mixed_support_routing = true`.
- `cargo test recursive_hierarchy_registered_helper_routes_mix_parent_ports_below_top`
  proves recursive non-top registered parent-composed helper D-cone
  routing can also mix parent data-port support in the same
  helper-sourced D cone:
  - `realized_min_leaf_depth = realized_max_leaf_depth = 2`
  - `hierarchy_parent_cone_instances > top_parent_cone_instances`
  - `hierarchy_parent_local_flops > top_local_flops`
  - `child_input_bindings_from_registered_parent_composed_logic > top_child_input_bindings_from_registered_parent_composed_logic`
  - `child_input_bindings_from_registered_parent_cone_instances > top_child_input_bindings_from_registered_parent_cone_instances`
  - `child_input_bindings_from_registered_parent_cone_instance_mixed_support > top_child_input_bindings_from_registered_parent_cone_instance_mixed_support`
  - `registered_parent_cone_instance_mixed_support_child_input_binding_fraction > 0.0`
  The route is banked in the full downstream-clean `r48` Phase 4 matrix
  through
  `saw_recursive_hierarchy_registered_parent_cone_instance_mixed_support_routing = true`.
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
  skipped Verilator/Yosys; the full downstream-clean `r30` bank now
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
  skipped Verilator/Yosys; the full downstream-clean `r30` bank now
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
  at `/tmp/anvil-tool-matrix-phase4-hierarchy-r76` closes them cleanly
  with `coverage_gaps = []` and `708/0` pass-fail in Verilator plus both
  repo-owned Yosys modes, including the direct sibling helper, direct
  registered sibling helper, direct registered sibling mixed-support,
  recursive non-top direct registered sibling mixed-support,
  multi-stage registered sibling,
  multi-stage direct registered sibling helper, multi-stage registered
  parent-composed helper, stateful parent-output helper routes, and
  stateful parent-composed helper child-input routes, plus recursive
  non-top stateful parent-composed helper child-input routes,
  recursive non-top direct sibling helper routes, and recursive non-top
  direct registered sibling helper routes, and recursive non-top
  multi-stage direct registered sibling helper routes, recursive non-top
  multi-stage registered parent-composed helper routes, and recursive
  non-top registered parent-composed helper routes, and recursive non-top
  parent-output helper routes, and recursive non-top parent-output
  helper mixed-support routes, and recursive non-top stateful
  parent-output helper routes, and recursive non-top stateful
  multi-helper budget routes, and recursive non-top registered
  mixed-support child-input routes, and recursive non-top registered
  parent-composed helper mixed-support routes, and recursive non-top multi-stage
  registered parent-composed child-input routes without helper
  instances, and recursive non-top multi-stage registered
  sibling-routed child-input routes without helper instances, and
  recursive non-top multi-stage registered mixed-support child-input
  routes without helper instances, plus stateful helper-backed
  parent-output mixed-support routes, unregistered parent-composed helper
  child-input mixed-support routes, stateful helper-through-flop
  child-input mixed-support routes, direct registered sibling
  mixed-support routes, and recursive non-top parent-port-composed
  parent-output routes.
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
  is the previous multi-stage registered sibling bank, `r27` is the
  previous stateful parent-output helper bank, `r28` is the previous
  multi-stage direct registered sibling helper bank, `r29` is the
  previous multi-stage registered parent-composed helper bank, `r30` is
  the previous stateful parent-composed helper full bank, `r31` is the
  previous recursive helper-state full bank, `r32` is the failed
  direct-helper run that exposed the CaseMux/Casez shift-cleanup gap,
  `r33` is the pre-compact-normalization direct-helper bank, `r34` is
  the previous recursive direct-helper hierarchy bank, `r35` is the
  previous recursive direct registered-helper hierarchy bank, `r36` is
  the previous recursive registered parent-composed helper hierarchy
  bank, `r37` is the previous recursive non-top multi-stage direct
  registered helper hierarchy bank, `r38` is the previous recursive
  non-top multi-stage registered parent-composed helper hierarchy bank,
  `r39` is the previous recursive non-top parent-output helper
  hierarchy bank, `r40` is the previous full downstream-clean recursive
  non-top stateful parent-output helper hierarchy bank, `r41` is the
  previous full downstream-clean recursive non-top parent-output multi-helper budget
  hierarchy bank, `r42` is the previous full downstream-clean recursive
  non-top stateful multi-helper budget hierarchy bank, `r43` is the
  previous full downstream-clean recursive non-top child-input
  multi-helper budget hierarchy bank, `r44` is the previous full
  downstream-clean recursive non-top registered mixed-support hierarchy
  bank, `r45` is the previous full downstream-clean recursive non-top
  multi-stage registered parent-composed no-helper hierarchy bank, `r46`
  is the previous full downstream-clean recursive non-top multi-stage
  registered sibling no-helper hierarchy bank, `r47` is the previous full
  downstream-clean recursive non-top multi-stage registered
  mixed-support no-helper hierarchy bank, `r48` is the previous full
  downstream-clean recursive non-top registered parent-composed helper
  mixed-support hierarchy bank, `r49` is the previous full
  downstream-clean recursive non-top parent-output helper mixed-support
  hierarchy bank, `r50` is the previous accumulated mixed-support
  hierarchy full bank, `r51` is the previous direct registered sibling
  mixed-support hierarchy full bank, `r52` is the previous recursive direct
  registered sibling mixed-support hierarchy full bank, `r53` is the previous
  recursive parent-composed mixed-support child-input hierarchy full bank,
  `r54` is the previous recursive parent-port-composed parent-output hierarchy
  full bank, `r55` is the previous recursive stateful parent-port-composed
  parent-output hierarchy full bank, `r56` is the previous recursive stateful
  unregistered parent-composed mixed-support child-input hierarchy full bank,
  `r57` is the previous hierarchy full bank that gated recursive non-top
  parent-local flops as a first-class coverage fact, `r58` is the previous
  hierarchy full bank that pushed recursive parent-local flops to exact
  hierarchy depth 3, `r59` is the previous hierarchy full bank that pushed
  recursive non-top unregistered parent-composed mixed-support child
  inputs to exact hierarchy depth 3 without helpers, `r60` is the previous
  hierarchy full bank that pushed recursive non-top parent-port-composed
  parent outputs to exact hierarchy depth 3 without helpers or state,
  `r61` is the previous hierarchy full bank that pushed recursive non-top
  stateful parent-port-composed parent outputs to exact hierarchy depth 3
  without helpers, `r62` is the previous hierarchy full bank that closed
  the depth-3 push with recursive non-top stateful parent-composed
  mixed-support child inputs at exact hierarchy depth 3 without helpers,
  `r63` is the previous hierarchy full bank that opened the depth-4 axis
  with recursive non-top parent-local flops at exact hierarchy depth 4,
  `r64` is the previous hierarchy full bank that extended the depth-4
  axis with recursive non-top mixed-support child inputs at exact
  hierarchy depth 4 without helpers, `r65` is the previous hierarchy
  full bank that extended the depth-4 axis with recursive non-top
  parent-port-composed parent outputs at exact hierarchy depth 4
  without helpers or state, `r66` is the previous hierarchy full bank
  that extended the depth-4 axis with recursive non-top stateful
  parent-port-composed parent outputs at exact hierarchy depth 4
  without helpers, `r67` is the previous hierarchy full bank that closed
  the depth-4 sweep with recursive non-top stateful parent-composed
  mixed-support child inputs at exact hierarchy depth 4 without
  helpers, `r68` is the previous hierarchy full bank that opened the
  depth-5 axis with recursive non-top parent-local flops at exact
  hierarchy depth 5, `r69` is the previous hierarchy full bank that
  extended the depth-5 axis with recursive non-top mixed-support child
  inputs at exact hierarchy depth 5 without helpers, `r70` is the
  previous hierarchy full bank that extended the depth-5 axis with
  recursive non-top parent-port-composed parent outputs at exact
  hierarchy depth 5 without helpers or state, `r71` is the previous
  hierarchy full bank that extended the depth-5 axis with recursive
  non-top stateful parent-port-composed parent outputs at exact
  hierarchy depth 5 without helpers, `r72` is the previous hierarchy
  full bank that closed the depth-5 sweep with recursive non-top
  stateful unregistered parent-composed mixed-support child inputs at
  exact hierarchy depth 5 without helpers, `r73` is the previous
  hierarchy full bank that opened the depth-6 axis with recursive
  non-top parent-local flops at exact hierarchy depth 6, `r74` is the
  previous hierarchy full bank that extended the depth-6 axis with
  recursive non-top mixed-support child inputs at exact hierarchy
  depth 6 without helpers (2,2 calibrated), `r75` is the previous
  hierarchy full bank that extended the depth-6 axis with recursive
  non-top parent-port-composed parent outputs at exact hierarchy
  depth 6 without helpers or state, `r76` is the current hierarchy
  full bank that extends the depth-6 axis with recursive non-top
  stateful parent-port-composed parent outputs at exact hierarchy
  depth 6 without helpers, and the aborted
  `r8`
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
   parent-composed child-input, stateful parent-composed child-input,
   direct sibling, direct registered sibling, registered child-input,
   parent-output, stateful parent-output, budgeted helper, and
   multi-stage helper slices;
2. deepen registered child-to-child routing using the local
   parent-state surface;
3. deepen the parent-side routing/composition surface beyond the
   current mixed-output, sibling-binding, parent-input-cone, and
   local-parent-flop surfaces.

Only after that does Phase 4 become "done" in the same sense that the
leaf-kernel phases are done today.
