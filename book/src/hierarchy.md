# Hierarchy: Modules of Modules

ANVIL no longer stops at isolated leaf modules only. Phase 4 is now
live in a deliberately narrow but real form: **depth-1 wrapper
hierarchy**.

That means:

- ANVIL can generate a **library of leaf modules** with the existing
  leaf kernel,
- then generate a real **top wrapper module**,
- instantiate those leaves inside the wrapper, and
- expose every child output as a top-level output.

This is genuine module composition. It exercises elaboration,
inter-module port binding, multi-file emission, and downstream top
selection. It is not yet the full future hierarchy story.

## Current live slice

The current entry point is:

```text
--hierarchy-depth 1 --num-leaf-modules N
```

Depth `0` keeps the existing leaf-only path. Depth `1` enables the
first Phase 4 slice. Depths above `1` are still rejected by config
validation.

Generation order today is:

```text
generate_design(rng, knobs):
    library = []
    for _ in 0..num_leaf_modules:
        library.push(generate_leaf_module(rng, knobs))

    top = generate_wrapper_top(library)
    return Design { top, modules: library + [top] }
```

The wrapper top is intentionally simple:

- if any child has local flops, the wrapper gets shared `clk` and
  `rst_n` inputs;
- every child emitted input becomes a wrapper input (prefixed with the
  instance name);
- every child emitted output becomes a wrapper output; and
- each wrapper output is driven by a `Node::InstanceOutput`.

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
- hierarchy-aware `NodeId` identity/factorization, and
- a repo-owned Phase 4 closure gate comparable to the Phase 1/2/3
  gates.

## The next real steps

Phase 4 is now `in progress`, not `not started`. The next honest work
items are:

1. let parent cone generation choose sub-instances as one of the real
   answers to "what drives this signal?";
2. add deeper bounded recursion (`hierarchy_depth > 1`);
3. add the on-demand child-sourcing path beside the current
   pre-generated library path; and
4. add repo-owned hierarchy closure evidence in `tool_matrix`.

Only after that does Phase 4 become "done" in the same sense that the
leaf-kernel phases are done today.
