# anvil
Single entry point for the project.

## Project objective
`anvil` is a random by-construction generator of **synthesizable
SystemVerilog RTL**. Today its implemented lane produces syntactically
valid, semantically correct, synthesizable, and structurally
non-trivial modules by building a typed circuit graph via fanin-cone
recursion and emitting SV from it.

The intended destination is stronger than "valid enough": `anvil`
should become a **signoff-level-quality random RTL generator** whose
outputs are boringly clean for mainstream downstream HDL consumers. The
product goal is **legal, reproducible, unusual RTL** that parsers,
elaborators, RTL compilers, linters, simulators, and synthesis tools
should accept. Those artifacts can be used to stress such tools and
expose real bugs precisely because they stay inside the accepted
synthesizable envelope.

Whole-module intended functionality is not the target. By construction,
the recursive fanin-cone process mainly aims at legal structure and
tool-ingestible complexity; absent a specification, most generated
modules are expected to be functionally arbitrary or outright
gibberish, and that is acceptable.

The long-term scope is broader than one leaf-module format. The user
has now made that explicit: the current "leaf-module typed circuit
generator" is the starting point, not the end state. ANVIL is meant to
grow into the go-to tool for **multiple families of pseudo-random,
valid-by-construction, synthesizable HDL artifacts** — for example the
current DUT RTL lane, future oracle-backed micro-design corpora, and
future frontend/elaboration-oriented accept corpora with explicit
expected-facts manifests.

**Three load-bearing principles:**
1. **Recursion is the core algorithm.** The generator answers one question — *"what drives this signal?"* — and recurses. Every level of abstraction (gate, cone, module, hierarchy) is the same recursion with a richer choice set. Iteration is the exception; recursion is the default. Anything that can be expressed as a recursive descent over a typed circuit graph should be.
2. **Every emitted module is valid by construction.** No generate-then-filter. No post-hoc repair. If a generator output fails semantic validation or synthesis, that is a generator bug, not expected behavior.
3. **Every output is reproducible.** Byte-identical output for the same `(seed, knobs)` pair, across platforms, forever. Seeded ChaCha8; no `thread_rng`; no wall-clock entropy; no hash-map iteration order in output paths.

See `ROADMAP.md` for the phased scope of the current leaf RTL lane plus
the broader future artifact families.

## Fast ramp-up (recommended reading order)
1. `README.md` (this file): canonical entry point and project map.
2. `SESSION_BOOTSTRAP.md`: what a fresh session should read first to regain full context.
3. `USER_GUIDE.md`: live CLI, knobs, and downstream verification workflow.
4. `ROADMAP.md`: current priorities and phased milestones.
5. `CODEBASE_ANALYSIS.md`: live Rust-workspace analysis aligned to the roadmap and active code reality.
6. `DEVELOPMENT_NOTES.md`: engineering rationale behind design decisions.
7. `MEMORY.md`: compact, operational continuity/handoff snapshot with git hashes.
8. `CHANGES.md`: fully detailed description of completed changes.
9. `COMMIT.md`: canonical commit workflow.
10. `docs/TASK_TREE.md`: repo-local task-tree workflow and active-tree index. Opt-in per top-level task — used for multi-slice work where the linear `rN` cadence does not capture the decomposition (see `docs/TASK_TREE_README.md` for the portable setup guide).
11. `docs/tasks/*.md`: one file per active top-level task tree, with stable leaf IDs, current frontier, blockers, decisions, and verification log.
12. `book/`: mdBook — a live doc of equal standing with the short-form files. Structured in five parts: *Using anvil* (Getting Started / Tutorial / Recipes), *How It Works* (Core Idea / Algorithm / IR), *Correctness Guarantees*, *Motif Catalogue*, *Reference*. The user-facing chapters lead; design chapters follow. Recovery requires reading it.

Only the documents above are status authority. The mdBook is explicitly part of this set — not reference material adjacent to it.

## Key project file paths
### Crate layout
- `Cargo.toml`
- `src/main.rs`            CLI entry point
- `src/lib.rs`              library root
- `src/config.rs`           knobs, CLI overlay, validation
- `src/ir/types.rs`         `Module`, `Node`, `GateOp`, `Flop`, `DepSet`
- `src/ir/validate.rs`      IR invariant checker (safety net)
- `src/gen/mod.rs`          `Generator` entry points
- `src/gen/cone.rs`         fanin-cone recursion
- `src/gen/module.rs`       leaf-module generator
- `src/gen/hierarchy.rs`    Phase 4 hierarchy planner: legacy exact
                            depth-1 wrapper lane plus bounded recursive
                            lane, both with explicit child sourcing,
                            parent-side composition, direct child-input
                            routing, registered child-input routing,
                            parent-composed child-input routing,
                            parent-cone helper instances, and optional
                            parent-local state
- `src/gen/pool.rs`         `SignalPool` for terminal selection
- `src/emit/sv.rs`          IR → SystemVerilog pretty-printer

### Tests and examples
- `tests/pipeline.rs`       end-to-end: generate → validate → emit
- `examples/generate_one.rs` minimal library usage
- `src/bin/tool_matrix.rs`  curated Verilator/Yosys scenario-matrix harness

### Design docs (mdBook, live)
- `book/book.toml`
- `book/src/SUMMARY.md`
- `book/src/core-idea.md`           canonical statement of the algorithm
- `book/src/algorithm.md`           fanin-cone pseudocode and width rules
- `book/src/ir.md`                   circuit IR reference
- `book/src/by-construction.md`      generation-by-construction argument
- `book/src/synthesizability.md`     subset-enforcement discipline
- `book/src/non-triviality.md`       dep-set tracking, anti-collapse rules
- `book/src/sequential.md`           Phase 2 cone boundaries
- `book/src/sharing.md`              Phase 2 DAG sharing
- `book/src/hierarchy.md`            hierarchy and future composition layers
- `book/src/knobs.md`                knob taxonomy, reproducibility contract
- `book/src/architecture.md`         Rust module layout and testing strategy
- `book/src/non-goals.md`            explicit scope refusals
- `book/src/why-not-grammar.md`      IR vs annotated EBNF

## Build and validation commands
Cargo's default run target is `anvil`, so plain `cargo run -- ...`
invokes the generator even though the repository also has the auxiliary
`tool_matrix` binary. Select the harness explicitly with
`cargo run --bin tool_matrix -- ...`.

```bash
# Build
cargo build

# Core tests (IR validation + reproducibility)
cargo test

# Generate one module to stdout
cargo run -- --seed 42

# Generate 100 modules into a directory
cargo run -- --seed 42 --count 100 --out ./generated

# Generate one real depth-1 hierarchical design
cargo run -- --seed 42 --hierarchy-depth 1 --num-leaf-modules 3

# Generate one depth-1 hierarchical design that reuses child definitions
cargo run -- --seed 42 --hierarchy-depth 1 --num-leaf-modules 2 --num-child-instances 5

# Generate one bounded recursive hierarchy tree
cargo run -- --seed 42 --min-hierarchy-depth 2 --max-hierarchy-depth 3 --min-child-instances-per-module 2 --max-child-instances-per-module 4

# Generate one bounded recursive hierarchy tree with per-depth branching
cargo run -- --seed 42 --min-hierarchy-depth 2 --max-hierarchy-depth 2 --min-child-instances-per-module 1 --max-child-instances-per-module 3 --child-instances-per-depth 0=4:4 --child-instances-per-depth 1=2:2

# Force sibling-routed hierarchy child inputs in the current
# combinational parent-composition slice
cargo run -- --seed 42 --hierarchy-depth 1 --num-leaf-modules 2 --num-child-instances 4 --hierarchy-sibling-route-prob 1.0

# Force sibling-routed hierarchy child inputs from helper instances
cargo run -- --seed 42 --hierarchy-depth 1 --num-leaf-modules 2 --num-child-instances 4 --hierarchy-sibling-route-prob 1.0 --hierarchy-registered-sibling-route-prob 0.0 --hierarchy-registered-child-input-cone-prob 0.0 --hierarchy-child-input-cone-prob 0.0 --hierarchy-parent-cone-instance-prob 1.0 --max-parent-cone-instances-per-module 3 --hierarchy-parent-flop-prob 0.0 --terminal-reuse-prob 1.0 --constant-prob 0.0

# Force parent-composed hierarchy child-input bindings in the current
# combinational parent-composition slice
cargo run -- --seed 42 --hierarchy-depth 1 --num-leaf-modules 2 --num-child-instances 4 --hierarchy-child-input-cone-prob 1.0

# Force local parent flops in hierarchy parent-side cones
cargo run -- --seed 42 --hierarchy-depth 1 --num-leaf-modules 2 --num-child-instances 4 --hierarchy-parent-flop-prob 1.0

# Force registered sibling-routed hierarchy child inputs
cargo run -- --seed 42 --hierarchy-depth 1 --num-leaf-modules 2 --num-child-instances 4 --hierarchy-sibling-route-prob 0.0 --hierarchy-registered-sibling-route-prob 1.0 --hierarchy-child-input-cone-prob 0.0 --max-flops-per-module 8

# Force multi-stage registered sibling-routed hierarchy child inputs
cargo run -- --seed 42 --hierarchy-depth 1 --num-leaf-modules 2 --num-child-instances 4 --hierarchy-sibling-route-prob 0.0 --hierarchy-registered-sibling-route-prob 1.0 --hierarchy-registered-child-input-cone-prob 0.0 --hierarchy-child-input-cone-prob 0.0 --hierarchy-parent-cone-instance-prob 0.0 --hierarchy-parent-flop-prob 0.0 --max-flops-per-module 8 --terminal-reuse-prob 1.0 --constant-prob 0.0

# Force registered sibling-routed hierarchy child inputs whose D side uses a helper instance
cargo run -- --seed 42 --hierarchy-depth 1 --num-leaf-modules 2 --num-child-instances 4 --hierarchy-sibling-route-prob 0.0 --hierarchy-registered-sibling-route-prob 1.0 --hierarchy-registered-child-input-cone-prob 0.0 --hierarchy-child-input-cone-prob 0.0 --hierarchy-parent-cone-instance-prob 1.0 --max-parent-cone-instances-per-module 3 --hierarchy-parent-flop-prob 0.0 --max-flops-per-module 8 --terminal-reuse-prob 1.0 --constant-prob 0.0

# Force registered parent-composed hierarchy child inputs
cargo run -- --seed 42 --hierarchy-depth 1 --num-leaf-modules 2 --num-child-instances 4 --hierarchy-sibling-route-prob 0.0 --hierarchy-registered-sibling-route-prob 0.0 --hierarchy-registered-child-input-cone-prob 1.0 --hierarchy-child-input-cone-prob 0.0 --max-flops-per-module 8

# Force registered parent-composed child inputs whose D cones use helper instances
cargo run -- --seed 42 --hierarchy-depth 1 --num-leaf-modules 2 --num-child-instances 4 --hierarchy-sibling-route-prob 0.0 --hierarchy-registered-sibling-route-prob 0.0 --hierarchy-registered-child-input-cone-prob 1.0 --hierarchy-child-input-cone-prob 0.0 --hierarchy-parent-cone-instance-prob 1.0 --max-parent-cone-instances-per-module 3 --max-flops-per-module 8 --terminal-reuse-prob 1.0 --constant-prob 0.0

# Force multi-stage registered parent-composed helper routing
cargo run -- --seed 42 --hierarchy-depth 1 --num-leaf-modules 2 --num-child-instances 4 --hierarchy-sibling-route-prob 0.0 --hierarchy-registered-sibling-route-prob 0.0 --hierarchy-registered-child-input-cone-prob 1.0 --hierarchy-child-input-cone-prob 0.0 --hierarchy-parent-cone-instance-prob 1.0 --max-parent-cone-instances-per-module 1 --hierarchy-parent-flop-prob 0.0 --max-flops-per-module 8 --terminal-reuse-prob 1.0 --constant-prob 0.0

# Force parent-composed child-input helper routing through parent-local state
cargo run -- --seed 42 --hierarchy-depth 1 --num-leaf-modules 2 --num-child-instances 4 --hierarchy-sibling-route-prob 0.0 --hierarchy-registered-sibling-route-prob 0.0 --hierarchy-registered-child-input-cone-prob 0.0 --hierarchy-child-input-cone-prob 1.0 --hierarchy-parent-cone-instance-prob 1.0 --max-parent-cone-instances-per-module 1 --hierarchy-parent-flop-prob 1.0 --max-flops-per-module 64 --terminal-reuse-prob 1.0 --constant-prob 0.0 --min-width 1 --max-width 8 --max-depth 1

# Force the same helper-state child-input route below the top parent in a recursive hierarchy
cargo run -- --seed 42 --min-hierarchy-depth 2 --max-hierarchy-depth 2 --min-child-instances-per-module 2 --max-child-instances-per-module 2 --hierarchy-sibling-route-prob 0.0 --hierarchy-registered-sibling-route-prob 0.0 --hierarchy-registered-child-input-cone-prob 0.0 --hierarchy-child-input-cone-prob 1.0 --hierarchy-parent-cone-instance-prob 1.0 --max-parent-cone-instances-per-module 1 --hierarchy-parent-flop-prob 1.0 --max-flops-per-module 64 --terminal-reuse-prob 1.0 --constant-prob 0.0 --min-width 1 --max-width 8 --max-depth 1

# Force parent-output helper composition to spend a 3-helper budget
cargo run -- --seed 42 --hierarchy-depth 1 --num-leaf-modules 2 --num-child-instances 4 --hierarchy-sibling-route-prob 0.0 --hierarchy-registered-sibling-route-prob 0.0 --hierarchy-registered-child-input-cone-prob 0.0 --hierarchy-child-input-cone-prob 0.0 --hierarchy-parent-cone-instance-prob 1.0 --max-parent-cone-instances-per-module 3 --terminal-reuse-prob 1.0 --constant-prob 0.0

# Force the same 3-helper parent-output budget below the top parent
cargo run -- --seed 42 --min-hierarchy-depth 2 --max-hierarchy-depth 2 --min-child-instances-per-module 2 --max-child-instances-per-module 2 --hierarchy-sibling-route-prob 0.0 --hierarchy-registered-sibling-route-prob 0.0 --hierarchy-registered-child-input-cone-prob 0.0 --hierarchy-child-input-cone-prob 0.0 --hierarchy-parent-cone-instance-prob 1.0 --max-parent-cone-instances-per-module 3 --hierarchy-parent-flop-prob 0.0 --terminal-reuse-prob 1.0 --constant-prob 0.0 --max-depth 4

# Force parent-output helper composition through parent-local state
cargo run -- --seed 42 --hierarchy-depth 1 --num-leaf-modules 2 --num-child-instances 4 --hierarchy-sibling-route-prob 0.0 --hierarchy-registered-sibling-route-prob 0.0 --hierarchy-registered-child-input-cone-prob 0.0 --hierarchy-child-input-cone-prob 0.0 --hierarchy-parent-cone-instance-prob 1.0 --hierarchy-parent-flop-prob 1.0 --max-flops-per-module 64 --terminal-reuse-prob 1.0 --constant-prob 0.0 --min-width 1 --max-width 8 --max-depth 1

# Force the same stateful 3-helper parent-output budget below the top parent
cargo run -- --seed 42 --min-hierarchy-depth 2 --max-hierarchy-depth 2 --min-child-instances-per-module 2 --max-child-instances-per-module 2 --hierarchy-sibling-route-prob 0.0 --hierarchy-registered-sibling-route-prob 0.0 --hierarchy-registered-child-input-cone-prob 0.0 --hierarchy-child-input-cone-prob 0.0 --hierarchy-parent-cone-instance-prob 1.0 --max-parent-cone-instances-per-module 3 --hierarchy-parent-flop-prob 1.0 --max-flops-per-module 64 --terminal-reuse-prob 1.0 --constant-prob 0.0 --min-width 1 --max-width 8 --max-depth 1

# Generate hierarchical designs into a directory
cargo run -- --seed 42 --count 10 --out ./generated-hier --hierarchy-depth 1 --num-leaf-modules 3

# Library-usage example
cargo run --example generate_one

# Tool-clean matrix sweep
cargo run --bin tool_matrix -- --out ./tool-matrix

# Lint and formatting
cargo clippy --all-targets
cargo fmt --all

# mdBook (design docs)
mdbook build book
mdbook serve book
```

### Downstream smoke tests (optional, require external tools)
```bash
# Elaboration sanity check (requires Verilator)
verilator --lint-only generated/mod_42_0000.sv

# Synthesis sanity check (requires Yosys)
yosys -p "read_verilog -sv generated/mod_42_0000.sv; synth -noabc; stat"
```

Both should succeed on every generated file. In this repository,
Verilator and Yosys are validation tools: they check syntax,
elaboration/lint, and synthesis acceptability of the emitted HDL. They
are not the only intended consumers of ANVIL output, and a failure is a
generator bug; file it with the seed and the effective knobs from
`manifest.json`.

For a broader repo-owned sweep across construction strategies,
identity modes, factorization levels, and stress profiles:

```bash
cargo run --bin tool_matrix -- --out ./tool-matrix
```

That writes per-scenario generated corpora plus
`tool_matrix_report.json`, and exits non-zero if Verilator or Yosys
fails on any generated file or emits any warning. Current local smoke
status after the post-construction proof-cleanup slice: the built-in
matrix is 15/15 clean in Verilator and 15/15 clean in Yosys under the
current default `--yosys-mode without-abc`.

The harness now has an explicit Yosys mode axis too:

```bash
cargo run --bin tool_matrix -- --out ./tool-matrix --yosys-mode without-abc
cargo run --bin tool_matrix -- --out ./tool-matrix --yosys-mode with-abc
cargo run --bin tool_matrix -- --out ./tool-matrix --yosys-mode both
```

`without-abc` remains the default because it is the current stable
baseline. `with-abc` now means the repo-owned warning-clean ABC path
(`synth -noabc; abc -fast; opt -fast; stat; check`) rather than the
raw default `synth` script, because the latter's ABC flow was tripping
non-actionable combinational-network warnings on valid generated
designs. A small repo-owned `--yosys-mode both` probe is now clean in
both sub-modes: `without-abc = 15/15 pass`, `with-abc = 15/15 pass`.
A completed current-code `--phase1-gate --yosys-mode both` report now
exists at `/tmp/anvil-tool-matrix-phase1-real-r21`. The final
`tool_matrix_report.json` records:

- `15` scenarios
- `67` modules per scenario
- `1005` total modules
- `coverage_gaps = []`
- `Verilator pass/fail = 1005/0`
- `Yosys without-abc pass/fail = 1005/0`
- `Yosys with-abc pass/fail = 1005/0`

The completed current-code Phase 2 sharing report now also exists at
`/tmp/anvil-tool-matrix-phase2-share-r1`. Its final
`tool_matrix_report.json` records:

- `18` scenarios
- `12` modules per scenario
- `216` total modules
- `coverage_gaps = []`
- `Verilator pass/fail = 216/0`
- `Yosys without-abc pass/fail = 216/0`
- `Yosys with-abc pass/fail = 216/0`
- normalized share sweep:
  - `share_prob = 0.0`: `shared_node_fraction = 0.4122`
  - `share_prob = 0.3`: `shared_node_fraction = 0.4232`
  - `share_prob = 0.9`: `shared_node_fraction = 0.4386`

The completed current-code Phase 3 structured-surface report now also
exists at `/tmp/anvil-tool-matrix-phase3-structured-r4`. Its final
`tool_matrix_report.json` records:

- `21` scenarios
- `10` modules per scenario
- `210` total modules
- `coverage_gaps = []`
- `Verilator pass/fail = 210/0`
- `Yosys without-abc pass/fail = 210/0`
- `Yosys with-abc pass/fail = 210/0`

The completed current-code Phase 4 hierarchy report now also
exists at `/tmp/anvil-tool-matrix-phase4-hierarchy-r87`. Its final
`tool_matrix_report.json` records:

- `210` scenarios
- `4` designs per scenario
- `840` total designs
- `artifact_kind = "design"`
- `coverage_gaps = []`
- `Verilator pass/fail = 840/0`
- `Yosys without-abc pass/fail = 840/0`
- `Yosys with-abc pass/fail = 840/0`
- `saw_recursive_multiple_parent_cone_instances_per_parent = true`
- `saw_recursive_multiple_parent_cone_instances_per_parent_child_inputs = true`
- `saw_recursive_multiple_parent_cone_instances_per_parent_through_flops = true`
- `saw_recursive_hierarchy_parent_cone_instance_flop_outputs = true`
- `saw_recursive_hierarchy_parent_cone_instance_outputs = true`
- `saw_recursive_hierarchy_parent_cone_instance_mixed_support_outputs = true`
- `saw_recursive_hierarchy_direct_sibling_parent_cone_instance_routing = true`
- `saw_recursive_hierarchy_direct_registered_sibling_parent_cone_instance_routing = true`
- `saw_recursive_hierarchy_registered_multistage_parent_cone_instance_routing = true`
- `saw_recursive_hierarchy_registered_multistage_parent_composed_parent_cone_instance_routing = true`
- `saw_recursive_hierarchy_registered_parent_composed_parent_cone_instance_routing = true`
- `saw_recursive_hierarchy_parent_composed_parent_cone_instance_flop_routing = true`
- `saw_hierarchy_parent_cone_instance_flop_mixed_support_outputs = true`
- `saw_recursive_hierarchy_parent_cone_instance_flop_mixed_support_outputs = true`
- `saw_hierarchy_parent_cone_instance_mixed_support_routing = true`
- `saw_recursive_hierarchy_parent_cone_instance_mixed_support_routing = true`
- `saw_hierarchy_parent_composed_parent_cone_instance_flop_mixed_support_routing = true`
- `saw_recursive_hierarchy_parent_composed_parent_cone_instance_flop_mixed_support_routing = true`
- `saw_hierarchy_direct_sibling_parent_cone_instance_routing = true`
- `saw_hierarchy_direct_registered_sibling_parent_cone_instance_routing = true`
- `saw_hierarchy_registered_multistage_parent_cone_instance_routing = true`
- `saw_hierarchy_registered_multistage_parent_composed_parent_cone_instance_routing = true`
- `saw_hierarchy_parent_composed_parent_cone_instance_flop_routing = true`
- `saw_hierarchy_parent_cone_instance_outputs = true`
- `saw_hierarchy_parent_cone_instance_flop_outputs = true`
- `saw_multiple_parent_cone_instances_per_parent = true`
- `saw_hierarchy_registered_parent_cone_instance_routing = true`
- `saw_recursive_hierarchy_registered_parent_cone_instance_mixed_support_routing = true`
- `saw_hierarchy_parent_cone_instance_routing = true`
- `saw_hierarchy_parent_port_composed_outputs = true`
- `saw_hierarchy_registered_mixed_support_routing = true`
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
- `saw_recursive_hierarchy_depth_6_stateful_parent_composed_mixed_support_child_inputs = true`
- `saw_recursive_hierarchy_depth_7_parent_local_flops = true`
- `saw_recursive_hierarchy_depth_7_mixed_support_child_inputs = true`
- `saw_recursive_hierarchy_depth_7_parent_port_composed_outputs = true`
- `saw_recursive_hierarchy_depth_7_stateful_parent_port_composed_outputs = true`
- `saw_recursive_hierarchy_depth_7_stateful_parent_composed_mixed_support_child_inputs = true`
- `saw_recursive_hierarchy_three_stage_registered_parent_composed_chain = true`
- `saw_recursive_parent_cone_helper_budget_5 = true`
- `saw_recursive_hierarchy_canonical_module_signature_diversity = true`
- `saw_design_with_structurally_duplicate_modules = true`
- `saw_recursive_hierarchy_module_dedup_active = true`
- `saw_recursive_hierarchy_registered_mixed_support_routing = true`
- `saw_hierarchy_registered_multistage_routing = true`
- `saw_recursive_hierarchy_registered_multistage_routing = true`
- `saw_recursive_hierarchy_registered_multistage_mixed_support_routing = true`
- `saw_hierarchy_registered_multistage_sibling_routing = true`
- `saw_recursive_hierarchy_registered_multistage_sibling_routing = true`
- `saw_hierarchy_parent_local_flops = true`
- `saw_profiled_child_interface_synthesis = true`
- `saw_on_demand_child_sourcing = true`

The `r87` report is the latest fully banked downstream-clean repo-owned
Phase 4 closure artifact, not only the older wrapper baseline. It covers the broadened
`--num-child-instances` planner directly, bounded recursive depth `2`,
mixed recursive depth range `2:3`, child-instance profiles `2`, `4`,
`2:3`, and `1:3`, the per-depth override profile `0=4:4,1=2:2`, the
explicit hierarchy child-sourcing axis
`--hierarchy-child-source-mode <library|on-demand>`, exact profiled
child-interface synthesis in the on-demand lane, real recursive design
emission, real per-depth branching metrics, real mixed shallow/deep
recursive realization, real parent-side composition above instance
outputs, real sibling-routed hierarchy child inputs and
parent-composed child-input bindings, registered sibling-routed
hierarchy child inputs, direct registered sibling mixed-support
child-input binding, recursive non-top direct registered sibling
mixed-support child-input binding, registered parent-composed child-input
bindings, registered mixed-support child-input binding, recursive
non-top registered mixed-support child-input binding, multi-stage
registered parent-composed child-input binding, recursive non-top
multi-stage registered parent-composed child-input binding without
helper instances, multi-stage registered sibling-routed child-input
binding through earlier parent-local Qs,
recursive non-top multi-stage registered sibling-routed child-input
binding through earlier parent-local Qs without helper instances,
multi-stage direct registered sibling helper binding where a
helper-sourced parent Q feeds a later parent flop,
recursive non-top multi-stage direct registered sibling helper binding
where a non-top helper-sourced parent Q feeds a later non-top parent
flop,
recursive non-top multi-stage registered parent-composed helper binding
where a non-top helper-sourced parent Q feeds later non-top
parent-composed D logic,
multi-stage registered parent-composed helper binding where a
helper-sourced parent Q feeds later parent-composed D logic,
stateful parent-composed helper child-input binding where a
helper-sourced parent Q feeds unregistered parent-composed child-input
logic,
mixed parent-port / child-output parent outputs, recursive exact-depth-2
parent-output helper cones that also mix parent data-port support below
the top parent, explicit local parent
flops in hierarchy modules, parent-cone helper-instance child-input binding,
parent-output helper-instance composition, budgeted multi-helper
allocation including recursive non-top parent-output multi-helper
budgets, recursive non-top child-input multi-helper budgets, and
recursive non-top stateful multi-helper budgets, registered
parent-composed helper-sourced child-input D
cones, direct sibling helper routing, direct registered sibling
helper routing, direct registered sibling mixed-support routing,
recursive non-top direct registered sibling mixed-support routing,
and multi-stage direct registered sibling helper routing,
parent-output helper routes that pass through parent-local flops, plus
recursive exact-depth-2 parent-output helper routes below the top parent,
recursive exact-depth-2 parent-output helper routes through parent-local
flops below the top parent,
stateful parent-composed helper child-input routes, and the recursive
exact-depth-2 axis proving that a non-top hierarchy parent can source
parent-composed child inputs from parent-cone helper instances through
parent-local flops, plus the recursive exact-depth-2 axis proving that a
non-top hierarchy parent can source direct sibling-routed child inputs
from parent-cone helper instances, plus the recursive exact-depth-2 axis
proving that a non-top hierarchy parent can source direct registered
sibling-routed child inputs from parent-cone helper instances, plus the
recursive exact-depth-2 axis proving that a non-top hierarchy parent can
source registered parent-composed child-input D cones from parent-cone
helper instances, plus the recursive exact-depth-2 axis proving that
those helper-sourced registered parent-composed D cones can also mix
parent data-port support below the top parent, plus the recursive
exact-depth-2 axis proving that a non-top hierarchy parent can chain
direct registered sibling helper routes through helper-sourced
parent-local Qs, plus current mixed-support facts for stateful
helper-backed parent outputs, unregistered helper child-input routing,
stateful helper-through-flop child-input routing, and direct registered
sibling mixed-support routing. The earlier
coverage-only proofs at
`/tmp/anvil-tool-matrix-phase4-recursive-direct-helper-r32/tool_matrix_report.json`
and
`/tmp/anvil-tool-matrix-phase4-recursive-helper-state-r31/tool_matrix_report.json`
are superseded by the full downstream-clean `r87` bank.

The clean pre-fix `/tmp/anvil-tool-matrix-phase4-hierarchy-r22` run is
kept only as root-cause evidence: the stale total-design budget let the
42-scenario gate run `3` designs/scenario (`126` total). The live gate
now uses a `4` designs/scenario floor directly, so future scenario-count
growth cannot silently weaken the Phase 4 matrix. The focused clean
smokes at `/tmp/anvil-hier-reuse-smoke-r1`,
`/tmp/anvil-hier-under-smoke-r2`,
`/tmp/anvil-hier-parent-compose-smoke-r1/manifest.json`,
`/tmp/anvil-hier-range-smoke-r1/manifest.json`,
`/tmp/anvil-hier-depth-profile-smoke-r1/manifest.json`,
`/tmp/anvil-hier-mixed-depth-smoke-r1/manifest.json`,
`/tmp/anvil-hier-parent-state-smoke-r1/manifest.json`,
`/tmp/anvil-hier-registered-sibling-smoke-r1/manifest.json`,
`/tmp/anvil-hier-registered-child-input-cone-smoke-r2/manifest.json`,
`cargo test hierarchy_sibling_routes_can_use_helper_instances`, and
`cargo test hierarchy_registered_sibling_routes_can_use_helper_instances`
still remain useful targeted proof points. The older `r21` report is
historical pre-parent-output-helper evidence. The aborted `r8` rerun is
now only historical runtime evidence: it showed that the Phase 4 gate
should use a hierarchy-focused sequential leaf profile instead of
reusing the fattest Phase 1 motif-heavy sequential stress shape.

`tool_matrix` writes per-module or per-design checkpoint sidecars and
supports `--resume`, so interrupted output trees can be continued in
place instead of always forking a fresh `--out` directory.

For the repo-owned Phase 1 gate shape:

```bash
cargo run --bin tool_matrix -- --out ./tool-matrix-phase1 --phase1-gate
```

That auto-enables coverage-gap failure and raises the per-scenario
module count high enough to generate at least 1000 modules total. To
continue an interrupted run on the same tree:

```bash
cargo run --bin tool_matrix -- --out ./tool-matrix-phase1 --phase1-gate --resume
```

For the repo-owned Phase 2 sharing gate shape:

```bash
cargo run --bin tool_matrix -- --out ./tool-matrix-phase2-share --phase2-share-gate --yosys-mode both
```

That runs the representative `share_prob` sweep (`0.0`, `0.3`, `0.9`)
across 18 built-in sharing scenarios and records a normalized
`share_sweep` summary in the report so the knob can be proven against
the landed graph shape rather than only against generator-side rolls.

For the repo-owned Phase 3 structured-surface gate shape:

```bash
cargo run --bin tool_matrix -- --out ./tool-matrix-phase3 --phase3-structured-gate --yosys-mode both
```

That runs the dedicated structured-surface matrix and fails on coverage
gaps unless the report proves live exercise of the landed Phase 3
surfaces: priority encoder, comb/flop mux encodings, procedural
`case` / `casez`, bounded procedural `for`-fold, selectable
`Slice` / `Concat`, and variable shifts.

## Current CLI truth
- `anvil --seed N` generates a single module to stdout.
- `anvil --seed N --count M --out DIR` generates M modules into DIR with a `manifest.json`.
- `anvil --dump-config` prints the effective knobs as JSON.
- `anvil --identity-mode <node-id|relaxed>` is the coarse NodeId semantics switch; `node-id` selects the full-factorization doctrine (`NodeId` = expression identity), while `relaxed` is the intentional off-switch where equivalent expressions may keep different `NodeId`s.
- `anvil --factorization-level <none|cse|operand-unique|commutative|associative|constant-fold|peephole|e-graph>` is the current-build implementation/proof-depth dial inside `node-id`; lower rungs are weaker enforcement of the same doctrine, not a different meaning of `node-id`.
- `anvil --full-factorization` requests `--identity-mode node-id --factorization-level e-graph`; `anvil --no-full-factorization` requests `--identity-mode relaxed --factorization-level none`.
- `tool_matrix --yosys-mode <without-abc|with-abc|both>` controls
  whether the repo-owned Yosys harness runs the current `synth -noabc`
  path, the explicit ABC-enabled `abc -fast` path, or both.
- `tool_matrix --resume` reuses per-module checkpoints from an existing
  `--out` tree when the saved tool surface matches the current run. New
  same-binary checkpoints also carry a generator checkpoint, an `sv`
  hash, and a runtime fingerprint, so a rerun on the same binary can
  skip replaying already-proven modules while still checking file
  integrity. Older trees without that metadata fall back to the strict
  replay-and-validate path and are upgraded in place. Resume is
  intentionally byte-stable: if regenerated `.sv` no longer matches the
  saved artifact after a generator-semantics change, start from a fresh
  `--out` tree instead of forcing reuse across that boundary.
- `tool_matrix --phase2-share-gate` runs the repo-owned representative
  sharing sweep over `share_prob ∈ {0.0, 0.3, 0.9}` and fails on
  coverage gaps. Its report now includes a `share_sweep` summary with
  normalized `shared_node_fraction` because stronger sharing collapses
  total node count and therefore makes the raw shared-node count a bad
  control metric.
- `tool_matrix --phase3-structured-gate` runs the repo-owned
  structured-surface closure matrix and fails on coverage gaps unless
  the report proves the landed Phase 3 surfaces directly from emitted
  metrics and tool results.
- `tool_matrix --phase4-hierarchy-gate` runs the repo-owned hierarchy
  matrix and fails on coverage gaps unless the report proves multifile
  hierarchy designs with real instances, instance outputs, the declared
  top module, representative wrapper and recursive child-instance
  profiles, per-depth branching overrides, mixed shallow/deep recursive
  realization, explicit `library` vs `on-demand` child-sourcing
  coverage, sibling-routed, registered sibling-routed, and registered
  parent-composed child-input bindings, and clean downstream tool
  results.
- `anvil --hierarchy-child-source-mode <library|on-demand>` selects how
  hierarchy parents obtain child definitions. `library` keeps reusable
  child-definition pools; the current `on-demand` slice now
  synthesizes children against parent-planned exact data-interface
  profiles.
- `anvil --hierarchy-sibling-route-prob <p>` controls whether later
  sibling child inputs may bind from earlier sibling instance outputs
  instead of always binding from parent-boundary inputs. When
  `--hierarchy-parent-cone-instance-prob` also fires, this direct
  unregistered route can allocate a helper child and bind from its
  output. The route stays combinational.
- `anvil --hierarchy-registered-sibling-route-prob <p>` controls
  whether later child data inputs bind from earlier sibling outputs
  through one local parent flop. This is a separate registered
  child-to-child routing axis; default `0.0` preserves the current
  combinational hierarchy unless explicitly requested. When
  `--hierarchy-parent-cone-instance-prob` also fires, this direct
  registered route can allocate a helper child and use its output as
  the parent-flop D source. The optional default-off
  `--hierarchy-registered-sibling-mixed-support-prob <p>` sub-route can
  mix an available parent data-port companion into that D path before
  the parent-local flop while keeping the binding classified as direct
  registered sibling routing.
- `anvil --hierarchy-registered-child-input-cone-prob <p>` controls
  whether later child data inputs bind through parent-local
  combinational logic over already-available parent sources and then
  one local parent flop. When parent data inputs and earlier sibling
  outputs are both live, this route can mix both supports; when earlier
  parent flops are live, later routes can chain through those Qs; when
  `--hierarchy-parent-cone-instance-prob` fires, the registered D cone
  can also include a parent-cone helper instance output.
  Default `0.0`; this keeps the registered parent-composed route
  distinct from direct registered sibling routing.
- `anvil --hierarchy-child-input-cone-prob <p>` controls whether child
  data inputs may bind through parent-local combinational cones over
  already-available parent sources: parent data inputs, earlier sibling
  instance outputs, and earlier parent-side route gates.
- `anvil --hierarchy-parent-flop-prob <p>` controls whether
  parent-side hierarchy cones may emit local parent flops. The default
  is `0.0`, preserving the combinational hierarchy unless explicitly
  requested; setting it high lets child inputs and parent outputs route
  through registered parent state.
- Current scope: single-module combinational **and sequential**
  generation is mature, DAG sharing is default-on, the bounded semantic
  `e-graph` fragment is live under `--identity-mode node-id`, and
  Phase 4 hierarchy now has two real lanes:
  - legacy exact depth-1 wrapper mode via `--hierarchy-depth 1`
    plus `--num-leaf-modules` / `--num-child-instances`
  - bounded recursive hierarchy via
    `--min-hierarchy-depth..--max-hierarchy-depth` and
    `--min-child-instances-per-module..--max-child-instances-per-module`
    with optional per-parent-depth overrides via repeated
    `--child-instances-per-depth DEPTH=MIN:MAX`
  Both hierarchy lanes now also expose an explicit child-sourcing mode
  via `--hierarchy-child-source-mode <library|on-demand>`. `library`
  keeps the reusable child-definition pool live; the currently-landed
  `on-demand` slice now gives each child slot a parent-planned exact
  data-interface profile and requires the emitted child module to
  realize that exact data boundary. Control ports remain structural:
  `clk` / `rst_n` still propagate only when sequential state is
  present.
  Both lanes now also expose a sibling-routing dial via
  `--hierarchy-sibling-route-prob <p>`, so later child inputs may bind
  from earlier sibling instance outputs through the same dep-bearing
  width-adaptation machinery used elsewhere in the generator. When
  helper placement is enabled, that direct unregistered route can use a
  helper instance output instead of only a planned sibling output. The
  route remains intentionally combinational.
  Both lanes also expose
  `--hierarchy-registered-sibling-route-prob <p>`, which routes an
  earlier sibling output through one parent-local flop before binding a
  later child input. When helper placement is enabled, that same direct
  registered route can use a helper instance output on the flop D side.
  The default-off
  `--hierarchy-registered-sibling-mixed-support-prob <p>` sub-route can
  mix a parent data-port companion into that direct registered D path
  before the parent-local flop while keeping the route classified as
  direct registered sibling routing.
  Both lanes also expose
  `--hierarchy-registered-child-input-cone-prob <p>`, which routes a
  parent source through parent-local logic and then one parent-local
  flop before binding a later child input. When parent data inputs and
  earlier sibling outputs are both live, that registered route can mix
  both supports; when earlier parent flops are live, later routes can
  chain through those Qs; when `--hierarchy-parent-cone-instance-prob`
  fires, the registered D cone can also include a helper instance
  output.
  Both lanes also expose `--hierarchy-child-input-cone-prob <p>`, which
  lets child data inputs bind through parent-local combinational cones
  over parent data inputs, earlier sibling instance outputs, and earlier
  parent-side route gates.
  Both lanes also expose
  `--hierarchy-parent-cone-instance-prob <p>`, which lets those
  parent-local combinational cones instantiate a helper child as an
  internal parent-cone source. Helper outputs can now feed
  parent-composed child-input bindings, direct sibling child-input
  bindings, direct registered sibling route D inputs, registered
  child-input D cones, parent-output composition, or parent-composed
  child-input logic through parent-local helper Qs, and
  `--max-parent-cone-instances-per-module <N>` now controls the
  per-parent helper budget. This is the first landed slice where
  module instantiation participates directly in parent-side cone choice:
  the helper instance is separate from the planned child slots, and
  manifests report the route through `top_parent_cone_instances`,
  `hierarchy_parent_cone_instances`,
  `max_parent_cone_instances_per_internal_module`,
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
  `top_outputs_reaching_parent_cone_instances`,
  `hierarchy_outputs_reaching_parent_cone_instances`,
  `top_outputs_reaching_parent_cone_instance_mixed_support`,
  `hierarchy_outputs_reaching_parent_cone_instance_mixed_support`,
  `top_outputs_reaching_parent_cone_instances_through_parent_flops`,
  `hierarchy_outputs_reaching_parent_cone_instances_through_parent_flops`,
  `top_outputs_reaching_parent_cone_instances_through_parent_flops_with_mixed_support`,
  `hierarchy_outputs_reaching_parent_cone_instances_through_parent_flops_with_mixed_support`,
  and the matching
  child-input / parent-output fractions.
  Both lanes now also expose `--hierarchy-parent-flop-prob <p>`, which
  lets those parent-side cones emit local parent flops under a separate
  hierarchy-specific state knob.
  Control-port visibility follows the hierarchy doctrine exactly: pure
  comb-only modules omit `clk` / `rst_n`, sequential leaves emit them,
  and hierarchy parents keep them visible iff they carry local state or
  sequential descendants. Module names are allocated from one
  generator-global sequence across leaves, recursive parents, and
  repeated hierarchy designs in one run, so directory output can safely
  write one `.sv` file per module definition without name collisions.
  Parent outputs can be genuine parent-side
  cones that mix parent data inputs with child instance outputs while
  preserving child-output support, combinational by default and
  optionally stateful when requested. Hierarchy manifests now report
  both the composition facts and the realized tree shape numerically, including
  per-parent-depth branching summaries,
  `leaf_module_occurrences_by_depth` for mixed-depth trust. The
  latest repo-owned Phase 4 hierarchy matrix is banked downstream-clean at
  `/tmp/anvil-tool-matrix-phase4-hierarchy-r87/tool_matrix_report.json`
  for the wrapper, exact-depth recursive, mixed-depth recursive,
  explicit child-sourcing, exact profiled on-demand child synthesis,
  sibling-routed child-input binding, parent-composed child-input
  binding, registered sibling-routed child-input binding, direct registered
  sibling mixed-support child-input binding, registered
  parent-composed child-input binding, registered mixed-support
  child-input binding, recursive non-top registered mixed-support
  child-input binding, multi-stage registered parent-composed
  child-input binding, recursive non-top multi-stage registered
  parent-composed child-input binding without helpers, multi-stage registered sibling-routed child-input
  binding, recursive non-top multi-stage registered sibling-routed
  child-input binding without helpers, recursive non-top multi-stage
  registered mixed-support child-input binding without helpers, mixed
  parent-port / child-output parent outputs,
  parent-cone helper-instance child-input binding, parent-output
  helper-instance composition, budgeted multi-helper allocation,
  recursive non-top multi-helper budget evidence,
  recursive non-top child-input multi-helper budget evidence,
  recursive non-top stateful multi-helper budget evidence,
  recursive non-top parent-output helper routing,
  recursive non-top parent-output helper routing with mixed parent-port
  support,
  recursive non-top stateful parent-output helper routing,
  parent-output helper routing through parent-local flops,
  parent-composed helper child-input routing through parent-local
  flops,
  registered parent-composed helper-sourced child-input D cones,
  direct sibling helper routing, direct registered sibling helper
  routing, direct registered sibling mixed-support routing,
  recursive non-top direct registered sibling mixed-support routing,
  multi-stage direct registered sibling helper routing,
  recursive non-top multi-stage direct registered sibling helper
  routing,
  recursive non-top multi-stage registered parent-composed helper
  routing,
  multi-stage registered parent-composed helper routing,
  recursive non-top registered parent-composed helper D-cone routing
  with mixed parent-port support,
  stateful helper-backed parent-output mixed-support routing,
  unregistered parent-composed helper child-input mixed-support routing,
  stateful helper-through-flop child-input mixed-support routing,
  recursive non-top stateful parent-composed child-input route,
  recursive non-top direct sibling helper route,
  recursive non-top direct registered sibling helper route,
  recursive non-top registered parent-composed helper route,
  parent-local flop state, and per-depth-override profiles folded into
  `tool_matrix`, with `210` scenarios, `840` total designs,
  `coverage_gaps = []`, and `840/0` pass-fail in Verilator plus both
  repo-owned Yosys modes.
  The older `r21` report remains useful historical evidence for the
  pre-parent-output-helper surface, `r31` remains the previous
  66-scenario full bank, `r36` is the previous recursive registered
  parent-composed helper bank, `r37` is the previous recursive
  multi-stage direct registered helper bank, `r38` is the previous
  recursive multi-stage registered parent-composed helper bank, `r39`
  is the previous recursive non-top parent-output helper bank, `r40`
  is the previous recursive non-top stateful parent-output helper bank,
  `r41` is the previous recursive non-top parent-output multi-helper
  budget bank, `r42` is the previous recursive non-top stateful
  multi-helper budget bank, `r43` is the previous recursive non-top
  child-input multi-helper budget bank, `r44` is the previous recursive
  non-top registered mixed-support routing bank, `r45` is the previous
  recursive non-top multi-stage registered parent-composed no-helper
  routing bank, `r46` is the previous recursive non-top multi-stage
  registered sibling no-helper routing bank, `r47` is the previous
  recursive non-top multi-stage registered mixed-support no-helper
  routing bank, `r48` is the previous recursive non-top registered
  parent-composed helper mixed-support routing bank, `r49` is the
  previous recursive non-top parent-output helper mixed-support routing
  bank, `r50` is the previous accumulated mixed-support hierarchy bank,
  `r51` is the previous direct registered sibling mixed-support hierarchy bank,
  `r52` is the previous recursive direct registered sibling mixed-support hierarchy bank,
  `r53` is the previous recursive parent-composed mixed-support child-input hierarchy bank,
  `r54` is the previous recursive parent-port-composed parent-output hierarchy bank,
  `r55` is the previous recursive stateful parent-port-composed parent-output hierarchy bank,
  `r56` is the previous recursive stateful parent-composed mixed-support child-input hierarchy bank,
  `r57` is the previous bank that gated recursive non-top parent-local flops as first-class coverage,
  `r58` is the previous bank that pushed recursive parent-local flops to exact hierarchy depth 3,
  `r59` is the previous bank that pushed recursive non-top mixed-support child inputs to exact hierarchy depth 3 without helpers,
  `r60` is the previous bank that pushed recursive non-top parent-port-composed parent outputs to exact hierarchy depth 3 without helpers or state,
  `r61` is the previous bank that pushed recursive non-top stateful parent-port-composed parent outputs to exact hierarchy depth 3 without helpers,
  `r62` is the previous bank that closed the depth-3 push with recursive non-top stateful parent-composed mixed-support child inputs at exact hierarchy depth 3 without helpers,
  `r63` is the previous bank that opened the depth-4 axis with recursive non-top parent-local flops at exact hierarchy depth 4,
  `r64` is the previous bank that extended the depth-4 axis with recursive non-top mixed-support child inputs at exact hierarchy depth 4 without helpers,
  `r65` is the previous bank that extended the depth-4 axis with recursive non-top parent-port-composed parent outputs at exact hierarchy depth 4 without helpers or state,
  `r66` is the previous bank that extended the depth-4 axis with recursive non-top stateful parent-port-composed parent outputs at exact hierarchy depth 4 without helpers,
  `r67` is the previous bank that closed the depth-4 sweep with recursive non-top stateful parent-composed mixed-support child inputs at exact hierarchy depth 4 without helpers,
  `r68` is the previous bank that opened the depth-5 axis with recursive non-top parent-local flops at exact hierarchy depth 5,
  `r69` is the previous bank that extended the depth-5 axis with recursive non-top mixed-support child inputs at exact hierarchy depth 5 without helpers,
  `r70` is the previous bank that extended the depth-5 axis with recursive non-top parent-port-composed parent outputs at exact hierarchy depth 5 without helpers or state,
  `r71` is the previous bank that extended the depth-5 axis with recursive non-top stateful parent-port-composed parent outputs at exact hierarchy depth 5 without helpers,
  `r72` is the previous bank that closed the depth-5 sweep with recursive non-top stateful unregistered parent-composed mixed-support child inputs at exact hierarchy depth 5 without helpers,
  `r73` is the previous bank that opened the depth-6 axis with recursive non-top parent-local flops at exact hierarchy depth 6,
  `r74` is the previous bank that extended the depth-6 axis with recursive non-top mixed-support child inputs at exact hierarchy depth 6 without helpers,
  `r75` is the previous bank that extended the depth-6 axis with recursive non-top parent-port-composed parent outputs at exact hierarchy depth 6 without helpers or state,
  `r76` is the previous bank that extended the depth-6 axis with recursive non-top stateful parent-port-composed parent outputs at exact hierarchy depth 6 without helpers,
  `r77` is the previous bank that closed the depth-6 sweep with recursive non-top stateful unregistered parent-composed mixed-support child inputs at exact hierarchy depth 6 without helpers (2,2 calibrated),
  `r78` is the previous bank that opened the depth-7 axis with recursive non-top parent-local flops at exact hierarchy depth 7,
  `r79` is the previous bank that extended the depth-7 axis with recursive non-top mixed-support child inputs at exact hierarchy depth 7 without helpers (2,2 calibrated),
  `r80` is the previous bank that extended the depth-7 axis with recursive non-top parent-port-composed parent outputs at exact hierarchy depth 7 without helpers or state,
  `r81` is the previous bank that extended the depth-7 axis with recursive non-top stateful parent-port-composed parent outputs at exact hierarchy depth 7 without helpers,
  `r82` is the previous bank that closed the depth-7 sweep with recursive non-top stateful unregistered parent-composed mixed-support child inputs at exact hierarchy depth 7 without helpers (2,2 calibrated),
  `r83` is the previous bank that proved recursive non-top registered parent-composed child-input bindings can chain through three parent-local flop stages without helpers,
  `r84` is the previous bank that proved a recursive non-top internal parent can saturate a parent-cone helper budget of 5 helpers,
  `r85` is the previous bank that added canonical module signatures as the first slice of hierarchy-aware identity instrumentation,
  `r86` is the previous bank that proved the planner can emit structurally-duplicate Module definitions under tight constraints (HIERARCHY-AWARE-IDENTITY.2),
  `r87` is the current bank that implements the post-finalisation module-dedup pass under the opt-in `Config::hierarchy_module_dedup` knob and proves it downstream-clean (HIERARCHY-AWARE-IDENTITY.4 + .5),
  and the clean `r22` run records the
  pre-fix 126-design budget mismatch. The live gate now preserves four
  designs per Phase 4 scenario directly. **Phase 4 is `done`** as of
  `2026-05-16`: closed by the `PHASE-4-HIERARCHY.3` deliberate,
  evidence-backed scope cut against explicit `ROADMAP.md` Phase 4 exit
  criteria, with the r87 gate (210 scenarios, 840 designs,
  `coverage_gaps = []`, 840/0 in Verilator and both Yosys modes) as the
  closing artifact. Hierarchy-aware identity is delivered
  (`HIERARCHY-AWARE-IDENTITY` tree, r85–r87). The residual *"broader
  registered hierarchy patterns"* (further helper-instance placements,
  deeper registered child-to-child routing, richer parent-side
  composition) is open-ended capability-deepening with no finite
  completion point; it is explicitly **not** a Phase 4 blocker and no
  mode was retired — any future breadth lands as optional
  post-Phase-4 `rN` slices without reopening the phase. The next
  numbered phase is **Phase 5 — Parameterization** (now unblocked);
  every remaining roadmap phase is tracked as a task tree under
  `docs/TASK_TREE.md`. See `ROADMAP.md` for phase gating.

## Maintenance rule
`README.md` is updated whenever project entry-point information changes materially (objective, ramp-up flow, key paths, or CLI surface). It does not need updates for every commit.

## License
Licensed under either of:
- Apache License, Version 2.0
- MIT License

at your option.

Read `SESSION_BOOTSTRAP.md` and start from there.
