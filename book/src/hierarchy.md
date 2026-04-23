# Hierarchy: Modules of Modules

ANVIL no longer stops at isolated leaf modules only. Phase 4 is now
live in a deliberately narrow but real form: **depth-1 wrapper
hierarchy**.

That means:

- ANVIL can generate a **library of leaf modules** with the existing
  leaf kernel,
- choose a separate **instantiated child count** for the wrapper,
- then generate a real **top wrapper module**,
- instantiate those leaves inside the wrapper, and
- expose every child output as a top-level output.

This is genuine module composition. It exercises elaboration,
inter-module port binding, multi-file emission, and downstream top
selection. It is not yet the full future hierarchy story.

## Current live slice

The current entry point is:

```text
--hierarchy-depth 1 --num-leaf-modules N [--num-child-instances M]
```

Depth `0` keeps the existing leaf-only path. Depth `1` enables the
first Phase 4 slice. Depths above `1` are still rejected by config
validation.

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

The wrapper top is intentionally simple:

- if the instantiated children carry sequential state, the wrapper gets
  shared `clk` and `rst_n` inputs;
- every child emitted input becomes a wrapper input (prefixed with the
  instance name);
- every instantiated child emitted output becomes a wrapper output; and
- each wrapper output is driven by a `Node::InstanceOutput`.

The control-port rule is deliberate and inductive:

- pure comb-only modules do **not** emit `clk` / `rst_n`;
- sequential leaves do emit `clk` / `rst_n`; and
- once a wrapper carries sequential descendants, `clk` / `rst_n` stay
  visible all the way up the instantiated ancestor chain.

So the first hierarchy slice is **real** but also **honest**: the top
module is presently a composition layer, not yet a new fanin-cone
generator that recursively mixes gates, flops, and sub-instances in the
same parent cone.

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
2. `Node::InstanceOutput` is a real node kind, so parent modules can
   name and emit child outputs explicitly instead of relying on emitter
   side tables or implicit wiring.

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
- every child output is exposed exactly once,
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
Phase 4 slice. They let you judge wrapper quality without opening the
emitted `.sv`, including:

- library size vs instantiated child count,
- unique-instantiated-module count and unused-library count,
- reuse / coverage ratios,
- top interface shape,
- control fanout to child instances,
- weighted child interface / node / flop load, and
- per-definition instantiation histograms.

## Why the first slice is wrapper-only

This was a deliberate engineering choice, not a half-finished accident.

The wrapper slice buys several important things immediately:

- real multi-module emitted RTL,
- real elaboration pressure in Verilator/Yosys,
- real design-aware validation and emission APIs, and
- a clean boundary above the leaf kernel instead of folding hierarchy
  into `generate_leaf_module`.

It also keeps the open work honest. The following are **not** live yet:

- using an instance output as a pickable signal inside a freshly-built
  parent cone,
- recursive depth > 1 hierarchy,
- on-demand child generation sized to parent needs,
- hierarchy-aware `NodeId` identity/factorization.

What **is** now live beyond the original smoke is the repo-owned Phase 4
wrapper gate:

- `/tmp/anvil-tool-matrix-phase4-hierarchy-r7/tool_matrix_report.json`
- `12` scenarios
- `4` designs/scenario
- `48` total designs
- `coverage_gaps = []`
- `Verilator 48/0`
- `Yosys without-abc 48/0`
- `Yosys with-abc 48/0`

That gate proves the current wrapper slice directly from saved report
facts: multifile hierarchy designs, correct top-module tool invocation,
real child instances, and real `Node::InstanceOutput` use.

Current HEAD has widened the wrapper planner beyond the exact-once case,
and that broadened repo-owned rerun is now banked too. The focused
local proofs remain useful:

- `/tmp/anvil-hier-reuse-smoke-r1` is clean in Verilator, Yosys
  `synth -noabc`, and the repo-owned ABC-enabled Yosys path, and proves
  repeated child-definition reuse;
- `/tmp/anvil-hier-under-smoke-r2` is clean in the same three lanes and
  proves under-instantiation of the leaf library; and
- the refreshed `tool_matrix` Phase 4 scenario set now explicitly
  targets exact / reuse / under-instantiation profiles, and the fresh
  rerun at `/tmp/anvil-tool-matrix-phase4-hierarchy-r7` closes them
  cleanly with `coverage_gaps = []` and `48/0` pass-fail in Verilator
  plus both repo-owned Yosys modes. The older `r6` partial rerun is now
  only historical evidence that the heavy sequential hierarchy corners
  are runtime-expensive rather than malformed.

## The next real steps

Phase 4 is now `in progress`, not `not started`. The next honest work
items are:

1. let parent cone generation choose sub-instances as one of the real
   answers to "what drives this signal?";
2. add deeper bounded recursion (`hierarchy_depth > 1`);
3. add the on-demand child-sourcing path beside the current
   pre-generated library path.

Only after that does Phase 4 become "done" in the same sense that the
leaf-kernel phases are done today.
