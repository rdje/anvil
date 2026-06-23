# anvil
Single entry point for the project.

## Project objective
`anvil` is a random by-construction generator of **synthesizable
SystemVerilog RTL artifacts**. Its default DUT lane produces
syntactically valid, semantically correct, synthesizable, and
structurally non-trivial modules by building a typed circuit graph via
fanin-cone recursion and emitting SV from it.

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

The scope is broader than one leaf-module format. ANVIL now ships three
artifact lanes through the same `anvil` binary: the default DUT RTL
lane, an oracle-backed micro-design lane, and a source-level
frontend/elaboration accept lane with explicit expected-facts
manifests. The default remains `--artifact dut`; the other lanes are
opt-in and keep their generators decoupled from the DUT path.

**Three load-bearing principles:**
1. **Recursion is the core algorithm.** The generator answers one question — *"what drives this signal?"* — and recurses. Every level of abstraction (gate, cone, module, hierarchy) is the same recursion with a richer choice set. Iteration is the exception; recursion is the default. Anything that can be expressed as a recursive descent over a typed circuit graph should be.
2. **Every emitted module is valid by construction.** No generate-then-filter. No post-hoc repair. If a generator output fails semantic validation or synthesis, that is a generator bug, not expected behavior.
3. **Every output is reproducible.** Byte-identical output for the same `(seed, knobs)` pair, across platforms, forever. Seeded ChaCha8; no `thread_rng`; no wall-clock entropy; no hash-map iteration order in output paths.

See `ROADMAP.md` for the phased scope of the DUT RTL lane, the
delivered non-DUT artifact lanes, and the post-phase follow-up trees
that are now tracked through explicit current-proof boundaries.

## Fast ramp-up (recommended reading order)
1. `README.md` (this file): canonical entry point and project map.
2. `MEMORY_ARCHITECTURE.md`: durable agent-memory standard; defines the resume pointer, task-tree layer, decision-record layer, git-history layer, and enforcement.
3. `KNOWLEDGE_MAP.md`: generated question-keyed retrieval index for durable facts already logged in the repo.
4. `knowledge-map/KNOWLEDGE_MAP_ARCHITECTURE.md`: additive retrieval-layer standard; durable facts get question keys so future agents do not re-derive already-logged facts.
5. `knowledge-map/FAQ.md`: plain-language Knowledge Map boundaries, including no conversion sweeps and diagnostics-vs-archaeology.
6. `SESSION_BOOTSTRAP.md`: what a fresh session should read first to regain full context.
7. `USER_GUIDE.md`: live CLI, knobs, and downstream verification workflow.
8. `ROADMAP.md`: current priorities and phased milestones.
9. `CODEBASE_ANALYSIS.md`: live Rust-workspace analysis aligned to the roadmap and active code reality.
10. `DEVELOPMENT_NOTES.md`: engineering rationale behind design decisions.
11. `MEMORY.md`: compact, operational continuity/handoff snapshot with git hashes.
12. `CHANGES.md`: fully detailed description of completed changes.
13. `COMMIT.md`: canonical commit workflow.
14. `docs/TASK_TREE.md`: repo-local task-tree workflow and active-tree index. **Doctrine (2026-05-17, non-negotiable): no code change may be made without a task-tree leaf owning it first** (pure-docs/mdBook/workflow edits exempt; see its "ANVIL Adoption Scope" for the code/not-code boundary and `docs/TASK_TREE_README.md` for the portable setup guide).
15. `docs/tasks/*.md`: one file per top-level task tree, with stable leaf IDs, current frontier or closure state, blockers, decisions, and verification log.
16. `docs/decisions/*.md`: durable layer-C decision/fact records used by the memory architecture.
17. `book/`: mdBook — a live doc of equal standing with the short-form files. Structured in five parts: *Using anvil* (Getting Started / Tutorial / Recipes), *How It Works* (Core Idea / Algorithm / IR), *Correctness Guarantees*, *Motif Catalogue*, *Reference*. The user-facing chapters lead; design chapters follow. Recovery requires reading it.
18. `DOCTRINE_ENFORCEMENT.md`: the fourth portable architecture — every load-bearing doctrine is paired with a deterministic check run from one registry+driver (`scripts/check_doctrines.sh`) gated by the git hook (E3) + CI (E4). Live registry: `MEMORY-ARCH`, `KNOWLEDGE-MAP`, `CODE-CHANGE-EVIDENCE`, `TASK-TREE-OWNERSHIP` (decision `0026`).
19. `TOOLBOX.md`: the catalog of **ANVIL's own diagnostic instruments** (trace / metrics / introspect / `analyze` / `coverage` / `validate` / `minimize` / `hunt` / `divergence` / `--diff-sim` / `tool_matrix` gates / snapshots / `ram_guard`) for pinpointing issues ANVIL may have, plus the acceptance-checklist a code change must satisfy.

Only the documents above are status authority. The mdBook is explicitly part of this set — not reference material adjacent to it.

## Key project file paths
### Crate layout
- `Cargo.toml`
- `src/main.rs`            CLI entry point
- `src/lib.rs`              library root
- `src/config.rs`           knobs, CLI overlay, validation
- `src/ir/types.rs`         `Module`, `Node`, `GateOp`, `Flop`, `DepSet`
- `src/ir/validate.rs`      IR invariant checker (safety net)
- `src/ir/soft_union.rs`    SV-2023 `union soft` low-bits-slice up-opt
                            annotation pass (`--sv-version 2023` +
                            `soft_union_slice_prob`)
- `src/ir/function_emit.rs` combinational `function automatic`
                            emit-projection annotation pass
                            (`function_emit_prob`; decision `0012`)
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
- `src/introspect/mod.rs`   versioned introspection document builder (`--introspect`)
- `src/downstream/mod.rs`   shared hardened downstream-tool invocation surface
                            (verilator/yosys/iverilog/sv2v/slang) + `validate` / `minimize`
- `src/mcp/mod.rs`          read-mostly MCP server (tools / resources / prompts)

### Tests and examples
- `tests/pipeline.rs`       end-to-end: generate → validate → emit
- `examples/generate_one.rs` minimal library usage
- `src/bin/tool_matrix.rs`  curated Verilator/Yosys/Icarus scenario-matrix harness
- `src/bin/anvil_mcp.rs`    `anvil-mcp` stdio transport over `src/mcp`

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

# Compile/elaboration sanity check (requires Icarus Verilog)
iverilog -g2012 -o generated/mod_42_0000.vvp generated/mod_42_0000.sv
```

All enabled smoke tools should succeed on every generated file. In this
repository, Verilator, Yosys, and the optional Icarus compile column are
validation tools: they check syntax, elaboration/lint, compile
acceptance, and synthesis acceptability of the emitted HDL. They are not
the only intended consumers of ANVIL output, and a failure is a
generator bug; file it with the seed and the effective knobs from
`manifest.json`.

For a broader repo-owned sweep across construction strategies,
identity modes, factorization levels, and stress profiles:

```bash
cargo run --bin tool_matrix -- --out ./tool-matrix
```

That writes per-scenario generated corpora plus
`tool_matrix_report.json`, and exits non-zero if any enabled
downstream tool fails on any generated file or emits any warning.
Current focused smoke status after `SIGNOFF-SURFACE-EXPANSION.3`: the
built-in matrix is clean across Verilator, both repo-owned Yosys modes,
and the opt-in Icarus compile column:
`Verilator 17/0`, `Yosys without-abc 17/0`, `Yosys with-abc 17/0`,
`Icarus compile 17/0`
(`/tmp/anvil-signoff-surface-iverilog-r1/tool_matrix_report.json`).

The harness now has an explicit Yosys mode axis too:

```bash
cargo run --bin tool_matrix -- --out ./tool-matrix --yosys-mode without-abc
cargo run --bin tool_matrix -- --out ./tool-matrix --yosys-mode with-abc
cargo run --bin tool_matrix -- --out ./tool-matrix --yosys-mode both
```

The harness also has an optional Icarus Verilog compile/elaboration
column:

```bash
cargo run --bin tool_matrix -- --out ./tool-matrix --iverilog-compile
cargo run --bin tool_matrix -- --out ./tool-matrix --yosys-mode both --iverilog-compile
```

`--iverilog-compile` shells `iverilog -g2012` for each emitted module
or design and records the result in `ModuleReport.iverilog_compile` or
`DesignReport.iverilog_compile`. This is an acceptance gate, not a
behavioral testbench: it proves an additional simulator frontend can
compile/elaborate the emitted SV. For trace agreement, use
`--diff-sim`.

#### `--diff-sim`: cross-simulator semantic agreement

The matrix gains an opt-in column that asserts **semantic
agreement** across two independent simulators (iverilog + verilator),
not just that each tool accepts the SV. Per
`DIFFERENTIAL-SIMULATION` (`docs/tasks/DIFFERENTIAL-SIMULATION.md`):
the existing parse/synth columns prove ANVIL output is *accepted*;
this column proves it is *semantically equivalent*. That is the
signoff-quality bar — and the first gate in the repo to test it.

```bash
# Add the diff-sim column on top of the default scenario sweep
cargo run --bin tool_matrix -- --diff-sim --out ./tool-matrix
```

A per-axis subset selector picks the first scenario per major axis
(combinational / sequential-flop / hierarchy / memory / fsm),
capped at 5, deterministic. The selected names land in the report
under `diff_sim_subset` and are persisted to
`<out>/.diff-sim-subset` for `--resume`. The harness shells
`iverilog -g2012 + vvp` and `verilator --binary`, normalizes the
fixed-width-hex traces, byte-compares, and records the outcome
under each module's `diff_sim` field
(`ran`/`success`/`n_samples`/`skip_reason`/`mismatch_excerpt`).
Any DUT with `ran=true && success=true` lights the
`saw_design_with_cross_simulator_agreement` coverage fact.

The column is a friendly no-op when either simulator is absent
(`tools_present()` probe → `ran: false` with a clear skip
reason); the matrix still exits clean unless you also pass
`--fail-on-coverage-gap`. It runs AFTER Verilator and Yosys are
both clean on the module — there is no point asking simulators to
agree on output a parse/synth tool already rejected.

`without-abc` remains the default because it is the current stable
baseline. `with-abc` now means the repo-owned warning-clean ABC path
(`synth -noabc; abc -fast; opt -fast; stat; check`) rather than the
raw default `synth` script, because the latter's ABC flow was tripping
non-actionable combinational-network warnings on valid generated
designs. A previous small repo-owned `--yosys-mode both` probe was
clean in both sub-modes: `without-abc = 15/15 pass`, `with-abc =
15/15 pass`.
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

For the repo-owned signoff knob-sweep gate shape:

```bash
cargo run --bin tool_matrix -- --out ./tool-matrix-signoff-knobs --signoff-knob-sweep-gate --yosys-mode both
```

That runs the focused richer-knob-sweep matrix
(`SIGNOFF-AUTOMATION-EXPANSION.2b`) and fails on coverage gaps unless the
report proves the four previously-unswept generator knobs fire by
construction (operand/mux-arm duplication, array-packed aggregate, and
the memory×fsm interplay), with clean Verilator + both Yosys modes —
exercising adversarial axes that previously fired only by chance
(ROADMAP steering gap 3).

## Current CLI truth
- `anvil --artifact <dut|microdesign|frontend>` selects the artifact
  lane. `dut` is the default and preserves the historical no-flag DUT
  RTL path. `microdesign` and `frontend` are opt-in non-DUT lanes with
  expected-facts manifests.
- `anvil --artifact microdesign --lane-n-params N` controls the number
  of parameter/localparam declarations in the micro-design lane.
- `anvil --artifact frontend --lane-n-params N --lane-n-children M`
  controls the top parameter/localparam count and child-instance count
  in the frontend/elaboration lane.
- `anvil --seed N` generates a single module to stdout.
- `anvil --seed N --count M --out DIR` generates M modules into DIR with a `manifest.json`.
- `anvil --max-rss-mb <MiB>` / `anvil --ram-abort-pct <1..=100>` are the
  opt-in internal memory governor (`WORKLOAD-MEMORY-SAFETY.4`): they abort
  an `--out` run cleanly (deterministic exit code `99` + a stderr message
  naming the seed + effective knobs) once this process's RSS, or host used
  RAM%, crosses the ceiling, sampled between generated units. Both default
  to the sentinel `0` = off ⇒ byte-identical; they never change emitted
  RTL. They guard `anvil`'s own process from the inside, complementing
  `scripts/ram_guard.sh` (which guards external jobs from the outside).
- `anvil --dump-config` prints the effective knobs as JSON.
- `anvil hunt` is ANVIL's **first subcommand** (`BUG-HUNT-ORCHESTRATION.2d`): the
  turnkey downstream bug-hunt loop as one command — fuzz a deterministic seed
  sweep (`--seed`/`--seeds`), run the vetted tools (`--tools verilator,yosys`,
  `--yosys-mode`), treat any reject/warning (and, with `--diff-sim`, a
  cross-simulator trace mismatch; and, with `--divergence`, a cross-*tool*
  acceptance disagreement) as a finding, auto-minimize each failure
  (`--no-minimize` / `--budget`), and print a JSON `HuntReport` to stdout. A thin
  shim over the same `hunt::run` the MCP `hunt` tool drives (decision `0017`);
  `--out <dir>` additionally drops a self-contained reproducer bundle per finding
  (the MCP path instead serves each reproducer as an `anvil://artifact/<run_id>/…`
  resource). The flat-flag default path (`anvil --seed N …`) is unchanged when no
  subcommand is given ⇒ DUT byte-identical. See `USER_GUIDE.md`.
- **Use ANVIL in your CI** (`CI-PACKAGING-DISTRIBUTION`, decision `0022`): a
  tag-triggered release workflow (`.github/workflows/release.yml`) publishes
  prebuilt per-platform `anvil`+`anvil-mcp` archives + `SHA256SUMS` on every `v*`
  tag, and a **drop-in composite GitHub Action** (root `action.yml`) wraps `anvil
  hunt` so a downstream-tool maintainer adds one `uses: <owner>/anvil@<tag>` step
  (naming their installed `tools` + an optional `--profile`) and gets red CI plus
  reproducer-bundle artifacts on any finding. The Action is a thin shim over the
  same `anvil hunt` engine (no Action-only path, decision `0017`); user-installed
  tools (no vendoring). CI-infra only ⇒ DUT byte-identical. See `USER_GUIDE.md`
  ("Use ANVIL in your CI") and `book/src/recipes.md`.
- `anvil --introspect` prints the versioned agent-introspection JSON document
  (schema `1.21`) for a single-artifact run instead of SystemVerilog
  (`AGENT-INTROSPECTION-MCP`): a thin envelope whose payload is the exact serde
  projection of existing `Config`/`Metrics`/`DesignMetrics` (zero new computed
  truth), with a content-addressed `run_id`. Since schema `1.12`
  (`COVERAGE-STEERED-GENERATION.2b`, decision `0023`) the DUT payload also carries
  a `coverage_readout` section — the run's achieved per-knob + per-category
  construction-time fire rates + gate/operand/depth histograms (a SCHEMA-DERIVED
  read surface for coverage steering; also a standalone MCP `coverage` query).
  Requires a single-artifact stdout
  run (no `--out`, `--count 1`); default-off ⇒ DUT byte-identical. Contract:
  `docs/AGENT_INTROSPECTION_SCHEMA.md`.
- `anvil-mcp` is a separate default-off binary: a read-mostly MCP server
  (JSON-RPC 2.0 over **stdio** by default, or **HTTP** via the opt-in
  `--http <addr>` flag — a hand-rolled loopback-default transport driving the
  same dispatcher, no new dependency) that drives the agent bug-hunting loop. It
  exposes pure tools (`generate`/`introspect`/`analyze`/`coverage`/`dump_config`/`coverage_gaps`,
  where `generate`/`introspect` cover all three lanes via a `lane` arg defaulting
  to `dut`, `analyze` answers a derived-relation query over the DUT IR — the
  output **support cone** (`output_support`: what an output depends on), its dual
  fan-out (`input_reach`: what a source reaches), per-flop reset/data
  provenance (`flop_reset_provenance`), per-module reachability from the top
  (`module_reachability`: which modules in a design are reachable via the instance
  graph), the per-module register-to-register dependency graph
  (`flop_dependencies`: each flop's direct register predecessors/successors +
  self-feedback flag), per-inferrable-memory port provenance
  (`memory_provenance`: each memory's shape + the support cone of its read/write
  address, write-data, and write-enable ports — opening the opaque-memory-read-leaf
  boundary), and per-generated-encoding-FSM provenance
  (`fsm_provenance`: each FSM's shape — num_states/encoding/state_width/sel_width/
  out_width/is_mealy — + the support cone of its transition-select `sel` input —
  opening the opaque-FSM-output-leaf boundary, the sibling of `memory_provenance`),
  and per-node immediate (1-hop) driver adjacency
  (`node_drivers`: each IR node's kind/width/gate-op + its direct operand drivers
  in operand order — the atomic node-level primitive complementing the transitive
  `output_support` cone, surfacing each node's `GateOp`),
  schema `1.21`, unknown query/target ⇒
  `-32602`, `coverage` (`COVERAGE-STEERED-GENERATION.2b`, decision `0023`) returns
  the DUT run's achieved-coverage readout — per-knob + per-category
  construction-time fire rates + gate/operand/depth histograms, the same
  SCHEMA-DERIVED projection embedded in `--introspect`'s `coverage_readout` — and
  `coverage_gaps` projects the recorded
  `tool_matrix_report.json` gap list read-only), controlled tools
  (`validate`/`minimize`/`hunt`/`divergence` — where `hunt` (`BUG-HUNT-ORCHESTRATION.2c`)
  is the turnkey fuzz → detect → minimize loop over a deterministic seed sweep that
  returns a structured `HuntReport` and caches each failing `run_id` so its
  `anvil://artifact/<run_id>/{sv,introspection}` resolve, and `divergence`
  (`ACCEPTANCE-DIVERGENCE-HUNTING`, decision `0019`) classifies cross-tool
  **acceptance divergence** — one tool accepts while another warns/rejects
  valid-by-construction RTL, returning a `DivergenceReport` — all run only through
  the hardened `verilator`/`yosys`/`iverilog`/`sv2v`/`slang` allow-list, sandboxed + RAM-guarded
  + audit-logged), resources (artifact `.sv`/introspection/`manifest`/`analysis`,
  `knobs`/`lanes` catalogs, `audit/log`), and five workflow prompts
  (`find_downstream_bug`, `close_coverage_gap`, `minimize_reproducer`,
  `triage_tool_failures`, `explain_artifact`). It runs no generation path of its
  own; the default `anvil` build and `--artifact dut` stay byte-identical. See
  `book/src/agent-mcp.md`.
- `anvil --identity-mode <node-id|relaxed>` is the coarse NodeId semantics switch; `node-id` selects the full-factorization doctrine (`NodeId` = expression identity), while `relaxed` is the intentional off-switch where equivalent expressions may keep different `NodeId`s.
- `anvil --factorization-level <none|cse|operand-unique|commutative|associative|constant-fold|peephole|e-graph>` is the current-build implementation/proof-depth dial inside `node-id`; lower rungs are weaker enforcement of the same doctrine, not a different meaning of `node-id`.
- `anvil --full-factorization` requests `--identity-mode node-id --factorization-level e-graph`; `anvil --no-full-factorization` requests `--identity-mode relaxed --factorization-level none`.
- `anvil --sv-version <2012|2017|2023>` is the opt-in IEEE 1800 emission-target capability gate (`SV-VERSION-TARGETING.2b.1`, decision `0009`). Default `2012` is the honest floor: ANVIL's default emitted subset is 1800-2012-valid, so the default reproduces today's output byte-for-byte (`tests/snapshots.rs` untouched, and — with every up-opt knob off — all three targets are byte-identical). It is a **down-gating guarantee** — the emitter never emits a construct newer than the target — threaded into the emitter as a read-only capability bound (`SvVersion::permits`). Surfaced in `--dump-config` and `--introspect` (introspection schema MINOR-bumped `1.1`→`1.2`). The per-version downstream acceptance axis landed as `.2b.2`. The first **up-opt** now ships (`SV-VERSION-TARGETING.3b.2a`, decision `0010`): the default-off config knob `soft_union_slice_prob` renders a *proper low-bits* `Slice` (`a[hi:0]`) through an internal IEEE 1800-2023 `union soft` overlay (`u.w = src; gate = u.n`) — behaviour-preserving (packed-union members are LSB-aligned) and genuinely 2023 (heterogeneous-width packed-union members are legal only as `union soft`) — but only when the target is also `2023`; below 2023 it down-gates to the plain slice. Verilator accepts it under `--language 1800-2023`; Yosys/Icarus reject the syntax and are a recorded no-op. Orthogonal to `--identity-mode` / `--factorization-level`.
- `tool_matrix --yosys-mode <without-abc|with-abc|both>` controls
  whether the repo-owned Yosys harness runs the current `synth -noabc`
  path, the explicit ABC-enabled `abc -fast` path, or both.
- `tool_matrix --iverilog-compile` adds an opt-in Icarus Verilog
  compile/elaboration column (`iverilog -g2012`) to each generated
  artifact. It is warning-clean acceptance evidence only; it does not
  run a testbench or compare traces.
- `tool_matrix --sv2v` (`DOWNSTREAM-ADAPTER-EXPANSION.2b.2`, decision
  `0020`) adds an opt-in `sv2v` SystemVerilog→Verilog-2005 **transpile**
  acceptance column (`--sv2v-bin` overrides the binary). A clean transpile
  accepts; a non-zero exit or a warning is a finding (a candidate
  downstream-tool bug). Like `--iverilog-compile` it is an acceptance gate,
  not a behavioural testbench — the transpiled Verilog is discarded. `sv2v`
  is absent on most hosts; when so this column is a **friendly no-op** (a
  presence probe means a requested-but-missing `sv2v` records no column and
  never fails the run). Recorded as `ModuleReport.sv2v` /
  `DesignReport.sv2v` and tallied as `sv2v pass/fail` in the summary; a
  `union soft` up-opt module skips it alongside Yosys/Icarus. `sv2v` is also
  the first new entry in the closed downstream-adapter registry, selectable
  via the `validate`/`hunt`/`divergence` `tools` arg + the `anvil hunt
  --tools` CLI.
- `tool_matrix --slang` (`DOWNSTREAM-ADAPTER-EXPANSION.2c.2a`, decision
  `0020`) adds an opt-in `slang` SystemVerilog **elaboration** acceptance
  column (`--slang-bin` overrides the binary). A clean elaboration accepts; a
  non-zero exit or a warning is a finding. Like `--sv2v` it is an acceptance
  gate, not a behavioural testbench. `slang` is the closed registry's
  **second** new adapter and the first **fact-bearing** one — it also dumps a
  `--ast-json` view of the top's ports + child instances (the
  `extract_facts` hook, `.2c.1`); those facts are surfaced in the matrix
  report as `ModuleReport.slang_facts` / `DesignReport.slang_facts` (`.2c.2b`).
  `slang` is absent on most hosts; when so this column is a
  **friendly no-op** (a presence probe means a requested-but-missing `slang`
  records no column and never fails the run). Recorded as `ModuleReport.slang`
  / `DesignReport.slang` and tallied as `slang pass/fail`, selectable via the
  `validate`/`hunt`/`divergence` `tools` arg + the `anvil hunt --tools` CLI.
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
  results. It also folds in the Phase-6 motif gates — an inferrable
  memory design (`saw_inferrable_memory_design`), a generated-encoding
  Moore FSM design (`saw_fsm_design`), and, since
  `CAPABILITY-BREADTH-EXPANSION.2b.2b` (decision `0024`), a **Mealy** FSM
  design (`saw_mealy_fsm_design`): the focused `phase6_mealy_fsm` scenario
  (`fsm_mealy_prob = 1.0`) emits the input-dependent `(state_q, sel)`
  output decode and must be downstream-clean. Because Mealy is universally
  synthesizable, it takes the same acceptance columns as any FSM design
  (Verilator + Yosys, plus the with-ABC Yosys mode under `--yosys-mode
  both` and Icarus under `--iverilog-compile`) — no Verilator-only carve-out
  like the `union soft` up-opt. Default-off `fsm_mealy_prob` ⇒ DUT
  byte-identical.
- `tool_matrix --signoff-knob-sweep-gate` runs the repo-owned focused
  richer-knob-sweep matrix (`SIGNOFF-AUTOMATION-EXPANSION.2b`) and fails
  on coverage gaps unless the report proves the four previously-unswept
  generator knobs fire by construction — `operand_duplication_rate`
  (`saw_operand_duplication`), `mux_arm_duplication_rate`
  (`saw_mux_arm_duplication`), `aggregate_array_prob`
  (`saw_array_packed_aggregate_design`), and the memory×fsm interplay
  (`saw_memory_fsm_interplay_design`) — with clean Verilator + both
  Yosys modes. One focused scenario per knob across all three
  construction strategies. Banked clean at
  `/tmp/anvil-signoff-knob-sweep-r1` (12 scenarios, 48 modules,
  `coverage_gaps = []`, `48/0` Verilator + both Yosys; closes ROADMAP
  steering gap 3's hidden-bias hole for these knobs).
- `tool_matrix --sv-version-gate` runs the repo-owned per-version
  acceptance matrix (`SV-VERSION-TARGETING.2b.2b` + `.3b.2b`) and fails
  on coverage gaps unless every targeted IEEE 1800 standard's corpus is
  accepted in the matching tool standard mode. It sweeps the three
  targets (2012/2017/2023) over a focused corpus (a combinational e-graph
  leaf, a sequential motif leaf, and a recursive depth-2 hierarchy design
  per version), runs **Verilator in the matching `--language 1800-20xx`
  mode** (via the `.2b.2a` selector) plus Yosys `-sv`, and requires
  `saw_sv_version_2012_targeted_acceptance`,
  `saw_sv_version_2017_targeted_acceptance`,
  `saw_sv_version_2023_targeted_acceptance`, and the umbrella
  `saw_sv_version_targeted_acceptance`. Those nine common-floor scenarios
  emit byte-identical SV across the three targets, so their value is the
  per-version downstream acceptance axis. A tenth **up-opt scenario**
  (`.3b.2b`) carries the live divergence: a slice-heavy 2023-targeted
  leaf with `soft_union_slice_prob = 1.0` that genuinely emits the IEEE
  1800-2023 `union soft` overlay accepted by Verilator `--language
  1800-2023`. Yosys/Icarus reject the `union soft` syntax, so that
  scenario runs **Verilator-only** (Yosys/Icarus recorded no-op, decision
  `0010`) and the report requires the dedicated
  `saw_sv_version_2023_soft_union_upopt` fact. Uses the `Interleaved`
  strategy only (other gates own strategy breadth). Banked clean at
  `/tmp/anvil-sv-version-gate-upopt-r1` (10 scenarios, 20 units,
  `coverage_gaps = []`, Verilator `20/0`, Yosys `18/0` both modes — the
  up-opt scenario's two modules are the Yosys no-op).
- `anvil --profile <name>` applies a curated knob preset before explicit
  flags (`KNOB-ERGONOMICS-AND-PRESETS.2b.1`, decision `0021`):
  `arithmetic-heavy` (datapath bias), `deep-hierarchy` (bounded recursive
  hierarchy with sibling routing + parent-local flops),
  `structured-emission-max` (all four emit-projections on), and
  `sv2023-upopts` (`--sv-version 2023` + the `union soft` up-opt). The
  resolution order is `default → --config → --profile → explicit flags →
  --seed`, so an explicit knob always **overrides** the preset; an unknown
  name errors with the valid list. Default-off (no `--profile`) ⇒ DUT
  byte-identical.
- `anvil --steer <key>=<weight>` (repeatable) biases construction-time coverage
  steering (`COVERAGE-STEERED-GENERATION.2c.1`, decision `0023`): `key` is a knob
  name (e.g. `flop_prob`) or a steering category
  (`state`/`selectors`/`datapath`/`terminals`/`sharing`/`hierarchy`) and `weight`
  is a non-negative multiplier on that roll's probability (`>1` emphasizes, `<1`
  de-emphasizes, `1` neutral). It is the ergonomic shim over `Config.steering`
  (which is also `--config`/MCP-settable); it layers after `--profile` (explicit
  wins per key) and is applied as a construction-time **prior** at the single
  `roll_knob` draw — **rules-first**, never generate-then-filter. An unknown key
  errors naming the valid categories; a non-finite/negative weight is rejected.
  Default-off (no `--steer`) and a neutral `=1.0` are both DUT byte-identical. The
  achieved coverage to steer *toward* is read from `--introspect`'s
  `coverage_readout` / the MCP `coverage` tool, and
  `introspect::coverage::derive_steering_from_coverage` turns a readout into a
  steering target (the outer measure→derive→re-steer loop).
- The 16 previously-config-file-only knobs are now also first-class CLI
  flags (`KNOB-ERGONOMICS-AND-PRESETS.2b.1`), each the kebab-case of the
  field: `--function-emit-prob`, `--generate-loop-emit-prob`,
  `--task-emit-prob`, `--cone-function-emit-prob`, `--soft-union-slice-prob`,
  `--width-parameterization-prob`, `--aggregate-prob`,
  `--aggregate-array-prob`, `--memory-prob`, `--fsm-prob`,
  `--multi-clock-prob`, `--cdc-synchronizer-stages`, plus the four on-only
  `SetTrue` toggles `--hierarchy-module-dedup`,
  `--hierarchy-semantic-module-dedup`, `--hierarchy-sequential-module-dedup`,
  and `--bisimulation-flop-merge`. `library_prob`, `use_async_reset`, and
  `max_nodes_per_module` stay config-file-only (still `--config`/MCP
  settable). All default-off ⇒ DUT byte-identical.
- `anvil --fsm-mealy-prob <p>` (`CAPABILITY-BREADTH-EXPANSION.2b`, decision
  `0024`) is the default-off **Mealy FSM output** knob (also `--config` / MCP
  settable; `fsm_mealy_prob`). When an FSM block is built (pair it with
  `--fsm-prob > 0`), it is the probability the FSM's output is **Mealy** — it
  depends on the current input as well as the current state — instead of
  **Moore**. It adds a second nested `case (state)` → `case (sel)` output
  decode driving the opaque `Node::FsmOut` (a per-`(state, sel)` constant table
  mirroring the transition table); the state register stays Moore-clocked. A
  behaviour-preserving extension of the `Fsm` block (no new IR node,
  rules-first; Mealy FSMs excluded from FSM dedup — nothing retired). Counted by
  `num_mealy_fsm_modules` (`--introspect`, schema `1.13`) and gated
  downstream-clean by the `phase6_mealy_fsm` `tool_matrix` scenario
  (`saw_mealy_fsm_design`). Default `0.0` ⇒ Moore, DUT byte-identical. See
  `book/src/sequential.md` "FSM outputs: Moore vs Mealy".
- `function_emit_prob` is a default-off knob (the `--function-emit-prob` CLI
  flag since `KNOB-ERGONOMICS-AND-PRESETS.2b.1`, or `--config` JSON;
  `STRUCTURED-EMISSION-EXPANSION.2b.1`, decision `0012`) — ANVIL's
  **first richer-structured emission surface**. Per *qualifying*
  combinational `Gate`, it is the probability the emitter re-renders the
  gate as a `function automatic` of its direct operands
  (`assign add_0 = add_0__f(i_1, casez_mux_0);` + a matching
  `function automatic` decl) instead of the inline `assign add_0 = i_1 +
  casez_mux_0;`. It is a behaviour-preserving **emit-time projection** of
  an already-valid cone (no new IR node / no new computed truth — the
  `soft_union`/aggregate precedent), rules-first (selection at
  construction time, no generate-then-filter). The first cut wraps a
  single gate over its direct operands; structured selectors (`case` /
  `casez` / `for`-fold) and `Slice` are excluded and still emit inline
  (a full-width `Slice` param would trip `-Wall UNUSEDSIGNAL`; nothing
  retired). Combinational only (a flop `Q` is a leaf parameter). Default
  `0.0` ⇒ DUT byte-identical (`tests/snapshots.rs` untouched); the
  emitted-function count is surfaced as
  `num_emitted_combinational_functions` in `--introspect` (schema
  `1.8`). Set it via `--function-emit-prob` or `--config` JSON. See
  `book/src/structured-emission.md`.
- `generate_loop_emit_prob` is a default-off knob (the
  `--generate-loop-emit-prob` CLI flag since
  `KNOB-ERGONOMICS-AND-PRESETS.2b.1`, or `--config` JSON;
  `STRUCTURED-EMISSION-EXPANSION.4b.1`, decision `0013`) — ANVIL's
  **second richer-structured emission surface** (its wider-lane case is the
  **fourth** surface, decision `0015`). Per *qualifying* `{N{x}}`
  replication (a `Concat` of `N >= 2` operands that are all the *same*
  signal, of any lane width `LW >= 1`), it is the probability the emitter
  re-renders it as a single-level `generate for` loop (`genvar <wire>__gi;
  generate for (<wire>__gi=0; <wire>__gi<N; <wire>__gi=<wire>__gi+1) begin :
  <wire>__gen assign <wire>[<wire>__gi] = <x>; end endgenerate` for a 1-bit
  lane — the one-hot `{W{sel}}` mux-mask idiom — or the indexed part-select
  body `assign <wire>[<wire>__gi*LW +: LW] = <x>;` for a wider lane)
  instead of the inline `assign <wire> = {N{x}};`. The unrolled loop is
  exactly `{N{x}}`, so it is a behaviour-preserving **emit-time projection**
  (no new IR node / no new computed truth — the
  `function_emit`/`soft_union`/aggregate precedent), rules-first; nothing
  retired. The projection is mutually exclusive with `function_emit_prob`
  on a gate; the increment is the maximally-portable `gi = gi + 1`.
  Combinational only. Default
  `0.0` ⇒ DUT byte-identical (`tests/snapshots.rs` untouched); the
  emitted-loop count is surfaced as `num_emitted_generate_loops` in
  `--introspect` (schema `1.9`). Set it via `--generate-loop-emit-prob` or
  `--config` JSON. See `book/src/structured-emission.md`.
- `task_emit_prob` is a default-off knob (the `--task-emit-prob` CLI flag
  since `KNOB-ERGONOMICS-AND-PRESETS.2b.1`, or `--config` JSON;
  `STRUCTURED-EMISSION-EXPANSION.6b.1`, decision `0014`) — ANVIL's
  **third richer-structured emission surface**. Per *qualifying*
  combinational gate (the **same candidate set as `function_emit_prob`** — a
  non-structured, non-`Slice` `Gate` with `>= 1` operand), it is the
  probability the emitter re-renders it as a combinational `task automatic`
  over its direct operands, called from `always_comb` into an output var
  (`task automatic <wire>__t(output logic[W-1:0] o, input …); o = <op>;
  endtask` + `logic <wire>__tv; always_comb <wire>__t(<wire>__tv, <refs>);` +
  the passthrough `assign <wire> = <wire>__tv;`) instead of the inline
  `assign <wire> = <op>;`. It is the decision `0012` single-gate
  `function automatic` parallel, but a *procedural* `task` — a
  behaviour-preserving **emit-time projection** (no new IR node / no new
  computed truth — the `function_emit`/`generate_loop`/`soft_union`/aggregate
  precedent), rules-first. The output-var + passthrough form keeps `<wire>` a
  net (only the gate's own drive changes). The four emit-projections
  (`function_emit` / `generate_loop` / `task_emit` / `soft_union`) are
  mutually exclusive on a gate; structured selectors + `Slice` are excluded
  (nothing retired). Combinational only. Default `0.0` ⇒ DUT byte-identical
  (`tests/snapshots.rs` untouched); the emitted-task count is surfaced as
  `num_emitted_combinational_tasks` in `--introspect` (schema `1.10`). Set it
  via `--task-emit-prob` or `--config` JSON. See `book/src/structured-emission.md`.
- `cone_function_emit_prob` is a default-off knob (the
  `--cone-function-emit-prob` CLI flag since `KNOB-ERGONOMICS-AND-PRESETS.2b.1`,
  or `--config` JSON; `STRUCTURED-EMISSION-EXPANSION.10b.1`, decision `0016`) —
  ANVIL's **fifth
  richer-structured emission surface**, a **deepening of the first surface** from
  a single gate to a whole combinational **cone**. Per *qualifying* combinational
  cone (a root gate plus the interior gates feeding it; the root is an
  admissible — non-structured, non-`Slice` — gate whose cone has `>= 1`
  absorbable interior gate), it is the probability the emitter re-renders the
  whole cone as one `function automatic` over the cone's boundary leaves — body
  = one function-local `logic` per absorbed interior gate in topological order +
  constants folded inline + `return` the root; the use site becomes a call
  (`assign <root> = <root>__cf(<leaf refs>);`) — instead of the inline per-gate
  `assign` chain. An interior gate is absorbed only when it is **used exactly
  once** in the module (so suppressing its module wire + inline assign is safe);
  a multi-use gate stays a boundary parameter. It is a behaviour-preserving
  **emit-time projection** (no new IR node / no new computed truth — the
  `function_emit` precedent), rules-first. It has its **own** knob (separate from
  `function_emit_prob`) so the shipped single-gate surface stays byte-identical
  (reusing it rejected). The five emit-projections (`function_emit` /
  `generate_loop` / `task_emit` / `cone_function` / `soft_union`) are mutually
  exclusive on a gate; the cone pass runs last. Combinational only. Default `0.0`
  ⇒ DUT byte-identical (`tests/snapshots.rs` untouched); the emitted-cone-function
  count is surfaced as `num_emitted_cone_functions` in `--introspect` (schema
  `1.11`). Set it via `--cone-function-emit-prob` or `--config` JSON. See
  `book/src/structured-emission.md`.
- `multi_output_task_emit_prob` is a default-off knob (the
  `--multi-output-task-emit-prob` CLI flag, or `--config` JSON;
  `STRUCTURED-EMISSION-EXPANSION.12b.1` (pair) + `.13b` (wider `k>2` groups),
  decision `0025`) — ANVIL's **sixth
  richer-structured emission surface**, a **generalization of the third surface**
  (the single-gate `task_emit_prob`) from one `output` to several. Per ungrouped
  *qualifying* combinational gate (the same candidate set as `task_emit_prob`), it
  is the probability the emitter makes it the leader of a **co-supported group**
  (`k >= 2`, bounded at 8 members) — greedily admitting each further qualifying gate
  that **shares a non-constant operand** with some current member and is
  **fan-in-independent** of every member — and co-emits the whole group as one
  multi-output `task automatic` with a **deduplicated** input list
  (`task automatic <leader>__mt(output …o0, o1, o2…, input …a0, a1…); o0 = …; o1 = …;
  endtask` + per-member `logic <wire>__mtv;` + one `always_comb <leader>__mt(…)` +
  the passthrough `assign <wire> = <wire>__mtv;`) instead of the inline
  `assign`s. A shared non-constant operand becomes **one** input formal feeding
  multiple outputs (the "co-supported sink"); a shared constant folds inline. It is
  a behaviour-preserving **emit-time projection** (no new IR node / no new computed
  truth — the `task_emit`/`cone_function` precedent), rules-first; its **own** knob
  so the shipped single-gate `task` surface stays byte-identical (reusing
  `task_emit_prob` rejected). The **fan-in-independence** rule is the soundness
  condition (a fan-in-dependent member would close a combinational cycle through
  the shared `always_comb`; each new member is checked against every member, so the
  group is cycle-free at any size); members keep their module wires (co-equal roots,
  not absorbed). The group grows greedily from the leader up to the 8-member cap
  (`MAX_MULTI_OUTPUT_TASK_GROUP_MEMBERS`; the first cut shipped a pair, `k > 2` now
  delivered by `.13b`). The six emit-projections (`function_emit` / `generate_loop` /
  `task_emit` / `multi_output_task` / `cone_function` / `soft_union`) are mutually
  exclusive on a gate; this pass runs after `task_emit`, before `cone_function`.
  Combinational only. Default `0.0` ⇒ DUT byte-identical (`tests/snapshots.rs`
  untouched); the emitted-task-group count is surfaced as
  `num_emitted_multi_output_tasks` in `--introspect` (schema `1.14`). Set it via
  `--multi-output-task-emit-prob` or `--config` JSON. See
  `book/src/structured-emission.md`.
- `mux_if_emit_prob` is a default-off knob (the `--mux-if-emit-prob` CLI flag, or
  `--config` JSON; `STRUCTURED-EMISSION-EXPANSION.15b`, decision `0027`) — ANVIL's
  **seventh richer-structured emission surface** and its **first
  procedural-conditional** shape. Per *qualifying* 2:1 `Mux` gate (a `GateOp::Mux`
  with a one-bit selector, not already marked by one of the six sibling
  projections), it is the probability the emitter re-expresses its continuous-assign
  ternary `assign <wire> = (sel) ? (a) : (b);` as a procedural `always_comb`
  `if`/`else` block writing a per-gate output var (`logic [W-1:0] <wire>__cv;
  always_comb begin if (sel) <wire>__cv = a; else <wire>__cv = b; end`), the net
  driven from it by the passthrough `assign <wire> = <wire>__cv;`. It is the decision
  `0014` single-gate-task **output-var + passthrough** mechanism, but a bare
  `always_comb` `if`/`else` rather than a `task` call — a genuinely new procedural
  shape (the six delivered surfaces are `function`/`task`/`generate` projections; the
  `Mux` is a continuous-assign ternary; `CaseMux`/`CasezMux` are `case`/`casez`). A
  behaviour-preserving **emit-time projection** (no new IR node / no new computed
  truth — the `task_emit`/`cone_function` precedent), rules-first; its **own** knob so
  the shipped surfaces stay byte-identical (reusing `task_emit_prob`/`function_emit_prob`
  rejected). The seven emit-projections (`function_emit` / `generate_loop` /
  `task_emit` / `multi_output_task` / `cone_function` / `soft_union` / `mux_if`) are
  mutually exclusive on a gate; this pass runs **last** (so it only excludes
  already-marked gates). Combinational only; the net stays a net (only its drive
  changes). Default `0.0` ⇒ DUT byte-identical (`tests/snapshots.rs` untouched); the
  emitted-block count is surfaced as `num_emitted_mux_if_blocks` in `--introspect`
  (schema `1.15`). Set it via `--mux-if-emit-prob` or `--config` JSON. See
  `book/src/structured-emission.md`.
- `case_mux_if_emit_prob` is a default-off knob (the `--case-mux-if-emit-prob` CLI flag,
  or `--config` JSON, like `mux_if_emit_prob`; `STRUCTURED-EMISSION-EXPANSION.17b`,
  decision `0028`) — ANVIL's **eighth richer-structured emission surface** and its **first
  N-way procedural priority chain**. Per *qualifying* dynamic-selector `CaseMux` gate (a
  `GateOp::CaseMux` whose selector operand is **not** a `Node::Constant`, with `>= 1` arm,
  not already marked by one of the seven sibling projections), it is the probability the
  emitter re-expresses its parallel `always_comb case (sel) … default` body as an
  `if`/`else if` **priority chain** over the same operand refs (`if (sel == SW'd0) g =
  arm_0; else if (sel == SW'd1) g = arm_1; … else g = W'h0;`) instead of the `case …
  endcase`. It is the decision-`0027` single-`Mux` `mux_if` parallel, but **simpler**: a
  `CaseMux` is already an `always_comb`-written `logic` var, so it needs **no** `<wire>__cv`
  output var + passthrough — only the block *body* swaps `case…endcase` → `if…else if`.
  Behaviour-preserving by construction (the `case` labels `SW'd0..SW'd{k-1}` are distinct
  constants ⇒ priority == parallel; the trailing `else` covers exactly the `default`),
  rules-first. A **constant-selector** `CaseMux` (statically collapsed to a continuous
  `assign`) and a `CasezMux` (masked `casez` wildcards — the recorded follow-up) are
  excluded (nothing retired). It has its **own** knob (reusing `mux_if_emit_prob`
  rejected). The eight emit-projections (`function_emit` / `generate_loop` / `task_emit` /
  `multi_output_task` / `cone_function` / `soft_union` / `mux_if` / `case_mux_if`) are
  mutually exclusive on a gate; this pass runs **last**. Combinational only. Default `0.0`
  ⇒ DUT byte-identical (`tests/snapshots.rs` untouched); the emitted-chain count is
  surfaced as `num_emitted_case_mux_if_chains` in `--introspect` (schema `1.16`; exact
  because constant-selector `CaseMux` is excluded). Proven downstream-clean by the
  repo-owned `tool_matrix --case-mux-if-gate` (**metric-keyed** `saw_case_mux_if_emit` —
  no new identifier token). Set it via `--case-mux-if-emit-prob` or `--config` JSON. See
  `book/src/structured-emission.md`.
- `casez_mux_if_emit_prob` is a default-off knob (the `--casez-mux-if-emit-prob` CLI flag,
  or `--config` JSON, like `case_mux_if_emit_prob`; `STRUCTURED-EMISSION-EXPANSION.19b`,
  decision `0029`) — ANVIL's **ninth richer-structured emission surface** and its **first
  masked priority chain**, a **generalization of the eighth surface** from the bare-equality
  `CaseMux` to the wildcard `CasezMux`. Per *qualifying* dynamic-selector `CasezMux` gate (a
  `GateOp::CasezMux` whose selector operand is **not** a `Node::Constant`, with `>= 1` arm,
  not already marked by one of the eight sibling projections), it is the probability the
  emitter re-expresses its parallel `always_comb casez (sel) … default` body as a **masked**
  `if`/`else if` priority chain over the same operand refs (`if ((sel & SW'h{care}) ==
  SW'h{val}) g = arm_0; else if … else g = W'h0;`) instead of the `casez … endcase`. The
  wildcard forces the mask: each arm compares only its **care** bits (`care_mask =
  ~wildcard_mask`, `value_masked = pattern & care_mask`, the established
  `metrics.rs`/`compact.rs` idiom), since a `casez` arm `2'b0?` ignores the `?` bit.
  Behaviour-preserving by construction (anvil builds `casez` patterns with one wildcard bit
  per arm + non-overlapping care patterns ⇒ masked priority == parallel `casez`; the trailing
  `else` covers exactly the `default`), rules-first. Like the eighth surface it needs **no**
  `<wire>__cv` output var + passthrough — a `CasezMux` is already an `always_comb`-written
  `logic` var, so only the block *body* swaps. A **constant-selector** `CasezMux` (statically
  collapsed to a continuous `assign`) and the bare-equality `CaseMux` (owned by the eighth
  surface) are excluded (nothing retired). It has its **own** knob (reusing
  `case_mux_if_emit_prob` rejected). The nine emit-projections (`function_emit` /
  `generate_loop` / `task_emit` / `multi_output_task` / `cone_function` / `soft_union` /
  `mux_if` / `case_mux_if` / `casez_mux_if`) are mutually exclusive on a gate; this pass runs
  **last**. The lowered masked-AND form ships because the concise `sel ==? pattern`
  wildcard-equality form is rejected by Yosys `0.64` in both repo modes. Combinational only.
  Default `0.0` ⇒ DUT byte-identical (`tests/snapshots.rs` untouched); the emitted-chain count
  is surfaced as `num_emitted_casez_mux_if_chains` in `--introspect` (schema `1.17`; exact
  because constant-selector `CasezMux` is excluded). Proven downstream-clean by the repo-owned
  `tool_matrix --casez-mux-if-gate` (**metric-keyed** `saw_casez_mux_if_emit` — no new
  identifier token). Set it via `--casez-mux-if-emit-prob` or `--config` JSON. See
  `book/src/structured-emission.md`.
- `tool_matrix --function-emit-gate` runs the repo-owned combinational
  `function automatic` emit gate (`STRUCTURED-EMISSION-EXPANSION.2b.2b`)
  and fails on coverage gaps unless the report proves the first
  richer-structured emission surface (decision `0012`) fires by
  construction and is downstream-accepted. It forces
  `function_emit_prob = 1.0` over a comb-only single-module DUT across all
  three construction strategies, so every qualifying combinational gate is
  projected to a behaviour-preserving `function automatic` over its direct
  operands, and requires the `saw_combinational_function_emit` fact (a
  genuinely-emitted function — detected from the emitted SV text —
  accepted by Verilator **and** Yosys). Unlike the `union soft` up-opt, a
  synthesizable function is accepted by every tool, so the gate runs the
  full Verilator + both Yosys modes (+ Icarus when `--iverilog-compile` is
  set) plan rather than Verilator-only. Banked clean at
  `/tmp/anvil-function-emit-gate-r1` (3 scenarios, 12 modules, 608 emitted
  functions, `coverage_gaps = []`, `12/0` Verilator + both Yosys + Icarus
  compile). Default `function_emit_prob = 0.0` emission stays
  byte-identical; the gate is the opt-in proof axis.
- `tool_matrix --generate-loop-gate` runs the repo-owned `generate for`
  loop emit gate (`STRUCTURED-EMISSION-EXPANSION.4b.2b`) and fails on
  coverage gaps unless the report proves the second richer-structured
  emission surface (decision `0013`) fires by construction and is
  downstream-accepted. It forces `generate_loop_emit_prob = 1.0` over a
  comb-only single-module DUT across all three construction strategies, so
  every qualifying `{N{x}}` replication (a 1-bit lane → `[<wire>__gi]`, a
  wider lane → the decision-`0015` `[<wire>__gi*LW +: LW]` part-select) is
  projected to a behaviour-preserving
  single-level `generate for` loop, and requires the `saw_generate_loop_emit`
  fact (a genuinely-emitted loop — detected from the emitted SV text —
  accepted by Verilator **and** Yosys). Like a function (and unlike the
  `union soft` up-opt), a `generate for` is universally synthesizable, so the
  gate runs the full Verilator + both Yosys modes (+ Icarus when
  `--iverilog-compile` is set) plan. Banked clean at
  `/tmp/anvil-generate-loop-gate-r1` (3 scenarios, 12 modules, 8 emitting a
  loop, `coverage_gaps = []`, `12/0` Verilator + both Yosys + Icarus
  compile). Default `generate_loop_emit_prob = 0.0` emission stays
  byte-identical; the gate is the opt-in proof axis.
- `tool_matrix --task-emit-gate` runs the repo-owned combinational
  `task automatic` emit gate (`STRUCTURED-EMISSION-EXPANSION.6b.2b`) and
  fails on coverage gaps unless the report proves the third richer-structured
  emission surface (decision `0014`) fires by construction and is
  downstream-accepted. It forces `task_emit_prob = 1.0` over a comb-only
  single-module DUT across all three construction strategies, so every
  qualifying combinational gate is projected to a behaviour-preserving
  `task automatic` over its direct operands (called from `always_comb` into a
  `<wire>__tv` output var), and requires the `saw_combinational_task_emit`
  fact (a genuinely-emitted task — detected from the emitted SV text —
  accepted by Verilator **and** Yosys). Like a function (and unlike the
  `union soft` up-opt), a combinational `task` is universally synthesizable,
  so the gate runs the full Verilator + both Yosys modes (+ Icarus when
  `--iverilog-compile` is set) plan. Banked clean at
  `/tmp/anvil-task-emit-gate-r1` (3 scenarios, 12 modules, 12 emitting a
  task, `coverage_gaps = []`, `12/0` Verilator + both Yosys + Icarus
  compile). Default `task_emit_prob = 0.0` emission stays byte-identical; the
  gate is the opt-in proof axis.
- `tool_matrix --cone-function-gate` runs the repo-owned multi-gate-cone
  `function automatic` emit gate (`STRUCTURED-EMISSION-EXPANSION.10b.2`) and
  fails on coverage gaps unless the report proves the fifth richer-structured
  emission surface (decision `0016`) fires by construction and is
  downstream-accepted. It forces `cone_function_emit_prob = 1.0` over a
  comb-only single-module DUT across all three construction strategies, so
  every qualifying combinational cone (a root gate plus its single-use interior
  gates) is projected to one behaviour-preserving `function automatic` over the
  cone's boundary leaves (one function-local per absorbed interior gate), and
  requires the `saw_cone_function_emit` fact (a genuinely-emitted cone function
  — detected from the emitted SV text's `<root>__cf(` token — accepted by
  Verilator **and** Yosys). Like a single-gate function (and unlike the
  `union soft` up-opt), a cone function is universally synthesizable, so the
  gate runs the full Verilator + both Yosys modes (+ Icarus when
  `--iverilog-compile` is set) plan. Banked clean at
  `/tmp/anvil-cone-function-gate-r1` (3 scenarios, 12 modules, 12 emitting a
  cone function / 148 cone functions, `coverage_gaps = []`, `12/0` Verilator +
  both Yosys + Icarus compile). Separate from `--function-emit-gate` (the
  single-gate surface); default `cone_function_emit_prob = 0.0` emission stays
  byte-identical; the gate is the opt-in proof axis.
- `tool_matrix --multi-output-task-gate` runs the repo-owned multi-output
  combinational `task automatic` emit gate (`STRUCTURED-EMISSION-EXPANSION.12b.2b`)
  and fails on coverage gaps unless the report proves the sixth richer-structured
  emission surface (decision `0025`) fires by construction and is
  downstream-accepted. It forces `multi_output_task_emit_prob = 1.0` over a
  comb-only single-module DUT across all three construction strategies, so every
  qualifying co-supported, fan-in-independent gate pair is co-emitted as one
  multi-output `task automatic` with deduplicated input formals + per-member
  output args, and requires the `saw_multi_output_task_emit` fact (a
  genuinely-emitted multi-output task — detected from the emitted SV text's
  `<leader>__mt(` token, distinct from the single-gate `<wire>__t(` and the cone
  `<root>__cf(` — accepted by Verilator **and** Yosys). Like a single-gate task
  (and unlike the `union soft` up-opt), a multi-output task is universally
  synthesizable, so the gate runs the full Verilator + both Yosys modes (+ Icarus
  when `--iverilog-compile` is set) plan. Banked clean at
  `/tmp/anvil-multi-output-task-gate-r1` (3 scenarios, 12 modules, 6 emitting a
  multi-output task, `coverage_gaps = []`, `12/0` Verilator + both Yosys + Icarus
  compile). Separate from `--task-emit-gate` (the single-gate surface); default
  `multi_output_task_emit_prob = 0.0` emission stays byte-identical; the gate is
  the opt-in proof axis.
- `tool_matrix --mux-if-gate` runs the repo-owned procedural `always_comb`
  `if`/`else` emit gate (`STRUCTURED-EMISSION-EXPANSION.15b.2`) and fails on coverage
  gaps unless the report proves the seventh richer-structured emission surface (the
  first procedural-conditional one, decision `0027`) fires by construction and is
  downstream-accepted. It forces `mux_if_emit_prob = 1.0` over a comb-only
  single-module DUT across all three construction strategies (a Mux-biased focus
  config — `comb_mux_prob = 0.9` + `comb_mux_encoding_prob = 1.0`, forcing the encoded
  chained-ternary path that builds plain `GateOp::Mux` gates), so every qualifying 2:1
  `Mux` gate is re-expressed as a behaviour-preserving procedural `if`/`else` block
  writing a `<wire>__cv` output var, and requires the `saw_mux_if_emit` fact (a
  genuinely-emitted block — detected from the emitted SV text's `<wire>__cv` token,
  distinct from the `<wire>__f(` / `<wire>__t(` / `<leader>__mt(` / `<root>__cf(`
  surfaces — accepted by Verilator **and** Yosys). Like a single-gate task (and unlike
  the `union soft` up-opt), a procedural `always_comb if/else` is universally
  synthesizable, so the gate runs the full Verilator + both Yosys modes (+ Icarus when
  `--iverilog-compile` is set) plan. Banked clean at `/tmp/anvil-mux-if-gate-r1` (3
  scenarios, 12 modules, 12 emitting a block / 215 blocks, `coverage_gaps = []`, `12/0`
  Verilator + both Yosys + Icarus compile). Separate from the per-gate/per-cone gates;
  default `mux_if_emit_prob = 0.0` emission stays byte-identical; the gate is the
  opt-in proof axis.
- `tool_matrix --case-mux-if-gate` runs the repo-owned procedural `always_comb`
  `if`/`else if` **priority-chain** emit gate (`STRUCTURED-EMISSION-EXPANSION.17b.2b`)
  and fails on coverage gaps unless the report proves the eighth richer-structured
  emission surface (the first N-way procedural priority chain, decision `0028`) fires
  by construction and is downstream-accepted. It forces `case_mux_if_emit_prob = 1.0`
  over a comb-only single-module DUT across all three construction strategies (a
  `case_mux_prob`-biased focus config — `case_mux_prob = 0.9` with `comb_mux_prob = 0.0`
  so the earlier-rolling comb-mux block never short-circuits the `case`-mux roll; no
  `comb_mux_encoding_prob` steering is needed because a `CaseMux` selector is a
  generated dynamic cone by construction, so there is no encoding-path trap like
  `--mux-if-gate` has), so every qualifying dynamic-selector `CaseMux` gate is
  re-expressed as a behaviour-preserving `if`/`else if` priority chain over the same
  operand refs (instead of the parallel `case` statement), and requires the
  `saw_case_mux_if_emit` fact. Unlike the sibling gates the detection is
  **metric-keyed** — this surface emits **no new identifier token** (a marked `CaseMux`
  is already an `always_comb`-written `logic` var, so only the body swaps
  `case…endcase` → `if…else if`), and a text scan for `if (… == …)` would also match
  FSM decode blocks, so the report keys `emitted_case_mux_if` off the exact
  `num_emitted_case_mux_if_chains` metric (`> 0`) rather than a substring. Like a
  single-gate task (and unlike the `union soft` up-opt), a procedural `always_comb
  if/else if` chain is universally synthesizable, so the gate runs the full Verilator +
  both Yosys modes (+ Icarus when `--iverilog-compile` is set) plan. Banked clean at
  `/tmp/anvil-case-mux-if-gate-r1` (3 scenarios, 12 modules, 12 emitting a chain / 83
  chains, `coverage_gaps = []`, `12/0` Verilator + both Yosys + Icarus compile).
  Separate from the per-gate/per-cone gates and `--mux-if-gate`; default
  `case_mux_if_emit_prob = 0.0` emission stays byte-identical; the gate is the opt-in
  proof axis.
- `tool_matrix --casez-mux-if-gate` runs the repo-owned procedural `always_comb`
  `if`/`else if` **masked priority-chain** emit gate (`STRUCTURED-EMISSION-EXPANSION.19b.2b`)
  and fails on coverage gaps unless the report proves the ninth richer-structured emission
  surface (the first masked priority chain, decision `0029`) fires by construction and is
  downstream-accepted. It forces `casez_mux_if_emit_prob = 1.0` over a comb-only
  single-module DUT across all three construction strategies (a `casez_mux_prob`-biased focus
  config — `casez_mux_prob = 0.9` with **both** `comb_mux_prob` and `case_mux_prob` zeroed,
  since both roll before `casez_mux` in the cone builder and would otherwise short-circuit the
  `casez`-mux roll; this is the eighth surface's single-zero generalized to a double-zero), so
  every qualifying dynamic-selector `CasezMux` gate is re-expressed as a behaviour-preserving
  masked `if`/`else if` priority chain (`(sel & care) == val`) over the same operand refs
  (instead of the parallel `casez` statement), and requires the `saw_casez_mux_if_emit` fact.
  Like the `--case-mux-if-gate` the detection is **metric-keyed** — this surface emits **no
  new identifier token** (a marked `CasezMux` is already an `always_comb`-written `logic` var,
  so only the body swaps `casez…endcase` → masked `if…else if`), and a text scan for `if ((…
  & …) == …)` would also match the eighth surface's chain, so the report keys
  `emitted_casez_mux_if` off the exact `num_emitted_casez_mux_if_chains` metric (`> 0`) rather
  than a substring. Like a single-gate task (and unlike the `union soft` up-opt), a procedural
  `always_comb if/else if` chain is universally synthesizable, so the gate runs the full
  Verilator + both Yosys modes (+ Icarus when `--iverilog-compile` is set) plan. Banked clean
  at `/tmp/anvil-casez-mux-if-gate-r1` (3 scenarios, 12 modules, 12 emitting a chain / 108
  chains, `coverage_gaps = []`, `12/0` Verilator + both Yosys + Icarus compile). Separate from
  the per-gate/per-cone gates, `--mux-if-gate`, and `--case-mux-if-gate`; default
  `casez_mux_if_emit_prob = 0.0` emission stays byte-identical; the gate is the opt-in proof
  axis.
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
  `e-graph` fragment is live under `--identity-mode node-id` (including
  small-support gate-to-gate merges and gate-to-existing-endpoint /
  constant folds when helper endpoints cancel out; current truth-table
  proofs cover up to 12 endpoint-support bits only while the cone fits
  the node/work budget), and Phase 4
  hierarchy now has two real lanes:
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
  post-Phase-4 `rN` slices without reopening the phase. **Phase 5 —
  Parameterization is also `done`** (2026-05-17): modules can carry a
  width `parameter` and be instantiated at multiple widths via
  `#(.W(v))`, rules-first and downstream-clean (closing artifact
  `/tmp/anvil-tool-matrix-phase5-p1`: 213 scenarios / 852 designs,
  `coverage_gaps = []`, 852/0 Verilator + both Yosys); parameter-aware
  child selection / parameter-driven parent generation are open-ended
  post-Phase-5 work (scope-cut, not a blocker). **Phase 5b —
  Synthesizable aggregates is also `done`** (2026-05-18): the opt-in
  `aggregate_prob` knob folds a non-instantiated module's
  same-direction data ports into one packed-`struct` emitter
  projection (flat IR / validators / dedup untouched; default-off
  byte-identical), closed against the `Phase4Hierarchy` matrix gate
  (closing artifact `/tmp/anvil-tool-matrix-phase5b-p1`: 216 scenarios
  / 864 designs, `coverage_gaps = []`, 864/0 Verilator + both Yosys,
  `saw_packed_aggregate_design = true`); `union`/`array` packing,
  parent-side aggregate connections and the param/aggregate
  cross-product are open-ended post-Phase-5b sub-slices (scope-cut,
  not a blocker). **Phase 6 — Advanced motifs is done (2026-05-20)**:
  both substantive motifs landed and are verified downstream-clean
  against the banked `Phase4Hierarchy` gate. The **memory motif**
  (delivered 2026-05-18) — a first-class `Memory` block + opaque
  `Node::MemRead` leaf rendering the Yosys-`$mem_v2`-inferrable
  synchronous template behind the opt-in `memory_prob` (default-off
  byte-identical) — closed against
  `/tmp/anvil-tool-matrix-phase6-p1` (219 scenarios / 876 designs,
  `coverage_gaps = []`, 876/0 Verilator + both Yosys,
  `saw_inferrable_memory_design = true`). The
  **generated-encoding FSM motif** (delivered 2026-05-20) — a
  first-class `Fsm` block + opaque `Node::FsmOut` + encoding-derived
  emitter (binary / one-hot / gray) behind the opt-in `fsm_prob`
  (default-off byte-identical), Moore outputs only — closed against
  `/tmp/anvil-tool-matrix-phase6-fsm-p1` (222 scenarios / 888
  designs, `coverage_gaps = []`, 888/0 Verilator + both Yosys,
  `saw_fsm_design = true` **and** `saw_inferrable_memory_design =
  true`, with Phase 4/5/5b regressions still proven in the same
  banked artifact). Multi-clock CDC remains the optional,
  separately-prioritised deferral (every module stays fully
  synchronous to a single clock). **Phase 7 — Oracle-backed
  micro-design artifacts is done (2026-05-20,
  `PHASE-7-ORACLE-MICRODESIGN` tree CLOSED):** delivered the
  `rtl_const_expr`-family micro-design lane (`src/microdesign/`,
  a separate generator path that never touches the DUT lane —
  default-off byte-identical) where the generator IS the
  oracle: every const-expr/parameter value is resolved at
  construction time and the same resolved value is shipped in a
  JSON manifest while held *symbolic* in the emitted `.sv`. The
  gap between symbolic text and resolved manifest is exactly the
  front-end-elaboration behaviour Phase 7 stresses. A parity
  harness (`tests/microdesign_parity.rs`) drives a fixed
  deterministic corpus through the consumer (currently yosys 0.64
  `write_json`) and a scoped comparator
  (`ToolReport`/`Divergence`/`FactCategory`/`ParityScope`/
  `compare_manifest_to_tool_report_in_scope`) reports exact
  agreement or retains a counterexample per axis. Closing
  artifact `/tmp/anvil-microdesign-parity-phase7-yosys-p1/`
  (5 reproducibility-set seeds × {`.sv`, `.json`, `.yosys.json`};
  `parity gate clean across 5 seeds`); the closing run found and
  fixed an ANVIL-self-consistency bug in `width_expr` (oracle
  used `rem_euclid`, SV used `%`; diverged for negative
  `last.value`) — exactly what `.1` designed the gate to do.
  Scope caveat: yosys 0.64 covers 4 of the 7 manifest fact
  categories (Seed/Top/Params/Widths/Generate); richer-AST
  coverage via a future microdesign-specific AST extractor is a
  recorded post-Phase-7 follow-up that does NOT retract closure (the
  manifest already covers all 7 categories). **Phase 8 —
  Frontend/elaboration accept corpora is done (2026-05-20,
  `PHASE-8-FRONTEND-ACCEPT` tree CLOSED):** delivered a
  source-level **AST IR** + construction-time
  elaboration-evaluator + un-elaborated-SV emitter +
  elaborated-facts JSON manifest emitter in
  `src/frontend/` (depth-1 elaboratable hierarchies: one
  package + one top module + N named-binding child stub
  instances + chained body localparams + a generate-if).
  Cross-tree reuse of Phase 7's `ConstExpr`/`eval`/
  `expr_to_sv` keeps the full-factorization doctrine
  satisfied and carries Phase 7's `.2c.2b.1`
  non-negative-modulo-idiom fix forward for free — which
  is exactly why Phase 8's parity gate came back clean on
  the **first** real-tool run (contrast with Phase 7's
  fix-and-retry). The repo-owned hierarchy-aware
  comparator (`ToolReport`/`InstanceToolReport`/
  `Divergence` × 23 variants including the load-bearing
  hierarchy-aware `Instance*` additions/`FactCategory`/
  `ParityScope`/`compare_manifest_to_tool_report_in_scope`)
  + yosys-specific extractor + `parity_against_real_yosys_hierarchy_write_json`
  end-to-end gate close Phase 8. Closing artifact
  `/tmp/anvil-frontend-parity-phase8-yosys-p1/` (5
  reproducibility-set seeds × {`.sv`, `.json`,
  `.yosys.json`}; "parity gate clean across 5 seeds"); per-
  seed fact agreement verified including the
  load-bearing per-instance per-binding values (yosys's
  `.cells[<inst>].parameters`) AND both generate
  branches exercised (seed 12345 takes `g_else`, others
  take `g_taken`). Scope caveat: yosys covers 5 of the 7
  manifest fact categories
  (Seed/Top/TopParams/Instances/GenerateBranches);
  top_localparams + package_constants are folded by
  yosys. `SIGNOFF-SURFACE-EXPANSION.2` adds a richer
  optional Verilator JSON-AST gate
  (`tests/frontend_parity.rs::parity_against_real_verilator_json_frontend_ast`)
  for local Verilator builds that support `--json-only`.
  That gate parses Verilator's specialized-module AST,
  enforces all 7 Phase-8 manifest categories, and is
  verified clean across the same 5 reproducibility seeds
  with artifacts in
  `target/tmp/frontend-parity-signoff-verilator-json`.
  `slang` is not required and was not present in the
  local tool environment. ANVIL now ships **three**
  complementary lanes: the DUT lane
  (Phases 1–6), the oracle-backed micro-design lane
  (Phase 7), and the source-level frontend/elaboration
  accept lane (Phase 8). **Phase 9 — Multi-artifact ANVIL
  umbrella is done (2026-05-20,
  `PHASE-9-MULTI-ARTIFACT-UMBRELLA` tree CLOSED):** the
  artifact-family selector + shared plumbing landed in
  `src/umbrella/` — `pub trait ArtifactLane` + the
  `LaneArtifact` carrier + the `CheckPlan` enum + the
  `DutLane`/`MicrodesignLane`/`FrontendLane` impls + the
  `--artifact <lane>` top-level CLI flag on the `anvil`
  binary (default `dut`). The explicit anti-goal from
  `.1` is preserved: only the plumbing
  (seed→artifact, byte-stable output, optional manifest,
  downstream check plan) unifies; the three lanes'
  rules-first generators stay decoupled in their own
  modules. The default `--artifact dut` invocation is
  byte-identical to today's no-flag invocation —
  load-bearing for `BOOK-EXAMPLES-RUNNABLE` + every CI
  gate, enforced from `.2a` by
  `dut_lane_is_byte_identical_to_direct_generator_path`
  AND verified end-to-end at `.2c` by
  `every_runnable_book_bash_block_succeeds`.
  **All 9 numbered roadmap phases now delivered.**
  The post-phase quality + capability lanes are closed
  end-to-end: **`DIFFERENTIAL-SIMULATION` closed
  (`2026-05-24`)** — `--diff-sim` opt-in cross-simulator
  semantic-agreement column in `tool_matrix` (per-axis K=5
  subset; iverilog 13.0 ↔ verilator 5.046 byte-equal
  post-reset trace check; `saw_design_with_cross_simulator_agreement`
  coverage fact). **`MULTI-CLOCK-CDC` `.3` container closed
  (`2026-05-24`)** — `Config.multi_clock_prob` knob +
  per-module promotion pass + default 2-flop synchronizer
  construction primitive + `int_multi_clock_2flop_sync`
  default-set scenario + `saw_multi_clock_design` /
  `saw_cdc_2_flop_synchronizer` coverage facts; the first
  ANVIL multi-clock SV passed Verilator + Yosys first try.
  `SIGNOFF-SURFACE-EXPANSION.1` extends that CDC lane with
  `Config.cdc_synchronizer_stages`, `int_multi_clock_3flop_sync`,
  `num_cdc_synchronizer_chains`, `max_cdc_synchronizer_stages`,
  and `saw_cdc_nflop_synchronizer` while preserving default
  2-stage behavior.
  See `docs/TASK_TREE.md` for the active-tree index,
  `ROADMAP.md` for phase gating, and
  `book/src/sequential.md` "Multi-clock and CDC" for the
  user-facing contract.

## Maintenance rule
`README.md` is updated whenever project entry-point information changes materially (objective, ramp-up flow, key paths, or CLI surface). It does not need updates for every commit.

## License
Licensed under either of:
- Apache License, Version 2.0
- MIT License

at your option.

Read `SESSION_BOOTSTRAP.md` and start from there.
