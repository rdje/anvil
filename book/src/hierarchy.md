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
  instance name);
- every instantiated child output is still available as a real
  `Node::InstanceOutput` leaf in the parent IR;
- top outputs are now built from those child-output leaves by the
  existing cone builder, with local parent flops disabled in the
  current slice; and
- unused child outputs are left explicitly unconnected at the instance
  site (`.port()`) instead of being forced into fake top-level
  pass-throughs.

The control-port rule is deliberate and inductive:

- pure comb-only modules do **not** emit `clk` / `rst_n`;
- sequential leaves do emit `clk` / `rst_n`; and
- once a wrapper carries sequential descendants, `clk` / `rst_n` stay
  visible all the way up the instantiated ancestor chain.

So the current hierarchy slice is **real** but also **honest**: parent
modules are real parent-side combinational composition layers over
child outputs, but the planner still does not yet mix shallow and deep
branches in one tree, add local parent flops, or solve hierarchy-aware
identity.

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

pub struct Instance {
    pub id: InstanceId,
    pub name: String,
    pub module: String,
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
- whether top outputs actually depend on child outputs at all,
- average / maximum child-output support per top output,
- control fanout to child instances,
- weighted child interface / node / flop load, and
- per-definition instantiation histograms,
- realized leaf depth / module depth,
- leaf-occurrence depth histogram,
- module-definition and module-occurrence depth histograms, and
- child-instance histograms plus per-depth instance-slot totals, and
- per-parent-depth branching summaries
  (`avg/min/max_child_instances_by_parent_depth`).

## Why the first slice is wrapper-only

This was a deliberate engineering choice, not a half-finished accident.

The wrapper slice buys several important things immediately:

- real multi-module emitted RTL,
- real elaboration pressure in Verilator/Yosys,
- real design-aware validation and emission APIs, and
- a clean boundary above the leaf kernel instead of folding hierarchy
  into `generate_leaf_module`.

It also keeps the open work honest. The following are **not** live yet:

- local parent flops inside the composed top layer,
- hierarchy-aware `NodeId` identity/factorization.

What **is** now live beyond the original smoke is the repo-owned Phase 4
hierarchy gate:

- `/tmp/anvil-tool-matrix-phase4-hierarchy-r9/tool_matrix_report.json`
- `15` scenarios
- `4` designs/scenario
- `60` total designs
- `coverage_gaps = []`
- `Verilator 60/0`
- `Yosys without-abc 60/0`
- `Yosys with-abc 60/0`

That gate proves the current representative hierarchy surface directly
from saved report facts: multifile hierarchy designs, correct
top-module tool invocation, real child instances, real
`Node::InstanceOutput` use, wrapper exact / reuse / under-instantiation
profiles, recursive depth `2`, child-instance profiles `2`, `4`, `2:3`
and `1:3`, the per-depth override profile `0=4:4,1=2:2`, real
per-depth branching metrics, and real parent-side composition above
instance outputs. The focused proof artifact for that composed-parent
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
- the refreshed `tool_matrix` Phase 4 scenario set now explicitly
  targets wrapper and recursive hierarchy profiles, and the fresh rerun
  at `/tmp/anvil-tool-matrix-phase4-hierarchy-r9` closes them cleanly
  with `coverage_gaps = []` and `60/0` pass-fail in Verilator plus both
  repo-owned Yosys modes. The older `r7` report is now the historical
  wrapper-baseline artifact, and the aborted `r8` rerun is historical
  evidence that the Phase 4 gate should use a hierarchy-focused
  sequential leaf profile instead of silently borrowing the fattest
  Phase 1 leaf-stress shape.

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

1. fold the new mixed-depth recursive axis into the repo-owned Phase 4
   gate so the closure artifact matches current HEAD again;
2. add local parent flops where structurally warranted;
3. add the on-demand child-sourcing / library-sourcing split as an
   explicit user-controllable axis.

Only after that does Phase 4 become "done" in the same sense that the
leaf-kernel phases are done today.
