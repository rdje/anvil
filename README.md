# anvil
Single entry point for the project.

## Project objective
`anvil` is a constrained-random generator of **synthesizable
SystemVerilog RTL**. Today its implemented lane produces syntactically
valid, semantically correct, synthesizable, and structurally
non-trivial modules by building a typed circuit graph via fanin-cone
recursion and emitting SV from it.

The intended destination is stronger than "valid enough": `anvil`
should become a **signoff-level-quality random RTL generator** whose
outputs are boringly clean in mainstream downstream tools while still
being rich enough to break them. The product goal is **legal,
reproducible, adversarial RTL** that can expose real parser,
elaboration, synthesis, and lint bugs precisely because it stays inside
the accepted synthesizable envelope.

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
10. `book/`: mdBook — a live doc of equal standing with the short-form files. Structured in five parts: *Using anvil* (Getting Started / Tutorial / Recipes), *How It Works* (Core Idea / Algorithm / IR), *Correctness Guarantees*, *Motif Catalogue*, *Reference*. The user-facing chapters lead; design chapters follow. Recovery requires reading it.

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

# Force parent-composed hierarchy child-input bindings in the current
# combinational parent-composition slice
cargo run -- --seed 42 --hierarchy-depth 1 --num-leaf-modules 2 --num-child-instances 4 --hierarchy-child-input-cone-prob 1.0

# Force local parent flops in hierarchy parent-side cones
cargo run -- --seed 42 --hierarchy-depth 1 --num-leaf-modules 2 --num-child-instances 4 --hierarchy-parent-flop-prob 1.0

# Force registered sibling-routed hierarchy child inputs
cargo run -- --seed 42 --hierarchy-depth 1 --num-leaf-modules 2 --num-child-instances 4 --hierarchy-sibling-route-prob 0.0 --hierarchy-registered-sibling-route-prob 1.0 --hierarchy-child-input-cone-prob 0.0 --max-flops-per-module 8

# Force registered parent-composed hierarchy child inputs
cargo run -- --seed 42 --hierarchy-depth 1 --num-leaf-modules 2 --num-child-instances 4 --hierarchy-sibling-route-prob 0.0 --hierarchy-registered-sibling-route-prob 0.0 --hierarchy-registered-child-input-cone-prob 1.0 --hierarchy-child-input-cone-prob 0.0 --max-flops-per-module 8

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

Both should succeed on every generated file. A failure is a generator bug; file with the seed and the effective knobs from `manifest.json`.

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
exists at `/tmp/anvil-tool-matrix-phase4-hierarchy-r21`. Its final
`tool_matrix_report.json` records:

- `33` scenarios
- `4` designs per scenario
- `132` total designs
- `artifact_kind = "design"`
- `coverage_gaps = []`
- `Verilator pass/fail = 132/0`
- `Yosys without-abc pass/fail = 132/0`
- `Yosys with-abc pass/fail = 132/0`
- `saw_hierarchy_parent_port_composed_outputs = true`
- `saw_hierarchy_registered_mixed_support_routing = true`
- `saw_hierarchy_registered_multistage_routing = true`
- `saw_hierarchy_parent_cone_instance_routing = true`

That refreshed report is the latest fully banked repo-owned Phase 4
closure artifact, not only the older wrapper baseline. It covers the
pre-parent-output-helper hierarchy surface: the broadened
`--num-child-instances`
planner directly, and it also proves the current recursive hierarchy
surface directly: depth `2`, mixed recursive depth range `2:3`,
child-instance profiles `2`, `4`, `2:3`, and `1:3`, the per-depth
override profile `0=4:4,1=2:2`, real recursive design emission, real
per-depth branching metrics, real mixed shallow/deep recursive
realization, real parent-side composition above instance outputs, and
the explicit hierarchy child-sourcing axis
`--hierarchy-child-source-mode <library|on-demand>`, including exact
profiled child-interface synthesis in the on-demand lane, plus real
sibling-routed hierarchy child inputs and parent-composed child-input
bindings proved numerically, plus registered sibling-routed hierarchy
child inputs through parent-local state, plus registered
parent-composed child-input bindings through parent logic and
parent-local state, plus registered mixed-support child-input binding,
multi-stage registered parent-composed child-input binding, mixed
parent-port / child-output parent outputs, and explicit local parent
flops in hierarchy modules.
The focused clean
smokes at `/tmp/anvil-hier-reuse-smoke-r1`,
`/tmp/anvil-hier-under-smoke-r2`,
`/tmp/anvil-hier-parent-compose-smoke-r1/manifest.json`,
`/tmp/anvil-hier-range-smoke-r1/manifest.json`,
`/tmp/anvil-hier-depth-profile-smoke-r1/manifest.json`,
`/tmp/anvil-hier-mixed-depth-smoke-r1/manifest.json`,
`/tmp/anvil-hier-parent-state-smoke-r1/manifest.json`, and
`/tmp/anvil-hier-registered-sibling-smoke-r1/manifest.json`, and
`/tmp/anvil-hier-registered-child-input-cone-smoke-r2/manifest.json`
still remain
useful targeted proof points. The aborted `r8` rerun is now only
historical runtime evidence: it showed that the Phase 4 gate should use
a hierarchy-focused sequential leaf profile instead of reusing the
fattest Phase 1 motif-heavy sequential stress shape.

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
  instead of always binding from parent-boundary inputs. The current
  direct sibling-routing slice is combinational.
- `anvil --hierarchy-registered-sibling-route-prob <p>` controls
  whether later child data inputs bind from earlier sibling outputs
  through one local parent flop. This is a separate registered
  child-to-child routing axis; default `0.0` preserves the current
  combinational hierarchy unless explicitly requested.
- `anvil --hierarchy-registered-child-input-cone-prob <p>` controls
  whether later child data inputs bind through parent-local
  combinational logic over already-available parent sources and then
  one local parent flop. When parent data inputs and earlier sibling
  outputs are both live, this route can mix both supports; when earlier
  parent flops are live, later routes can chain through those Qs.
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
  width-adaptation machinery used elsewhere in the generator. That
  routing remains intentionally combinational in the current slice.
  Both lanes also expose
  `--hierarchy-registered-sibling-route-prob <p>`, which routes an
  earlier sibling output through one parent-local flop before binding a
  later child input.
  Both lanes also expose
  `--hierarchy-registered-child-input-cone-prob <p>`, which routes a
  parent source through parent-local logic and then one parent-local
  flop before binding a later child input. When parent data inputs and
  earlier sibling outputs are both live, that registered route can mix
  both supports; when earlier parent flops are live, later routes can
  chain through those Qs.
  Both lanes also expose `--hierarchy-child-input-cone-prob <p>`, which
  lets child data inputs bind through parent-local combinational cones
  over parent data inputs, earlier sibling instance outputs, and earlier
  parent-side route gates.
  Both lanes also expose
  `--hierarchy-parent-cone-instance-prob <p>`, which lets those
  parent-local combinational cones instantiate a helper child as an
  internal parent-cone source. Helper outputs can now feed either
  child-input bindings or parent-output composition, and
  `--max-parent-cone-instances-per-module <N>` now controls the
  per-parent helper budget. This is the first landed slice where
  module instantiation participates directly in parent-side cone choice:
  the helper instance is separate from the planned child slots, and
  manifests report the route through `top_parent_cone_instances`,
  `hierarchy_parent_cone_instances`,
  `max_parent_cone_instances_per_internal_module`,
  `child_input_bindings_from_parent_cone_instances`,
  `top_outputs_reaching_parent_cone_instances`,
  `hierarchy_outputs_reaching_parent_cone_instances`, and the matching
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
  repo-owned Phase 4 hierarchy matrix is now banked at
  `/tmp/anvil-tool-matrix-phase4-hierarchy-r21/tool_matrix_report.json`
  for the wrapper, exact-depth recursive, mixed-depth recursive,
  explicit child-sourcing, exact profiled on-demand child synthesis,
  sibling-routed child-input binding, parent-composed child-input
  binding, registered sibling-routed child-input binding, registered
  parent-composed child-input binding, registered mixed-support
  child-input binding, multi-stage registered parent-composed
  child-input binding, mixed parent-port / child-output parent outputs,
  parent-cone helper-instance child-input binding,
  parent-local flop state, and
  per-depth-override profiles folded into `tool_matrix`,
  while the
  focused smokes
  at
  `/tmp/anvil-hier-range-smoke-r1/manifest.json` and
  `/tmp/anvil-hier-depth-profile-smoke-r1/manifest.json` remain useful
  targeted proofs. The latter proves depth-specific branching control
  numerically with
  `realized_min_leaf_depth = 2`, `realized_max_leaf_depth = 2`,
  `avg_child_instances_by_parent_depth = {"0": 4.0, "1": 2.0}`,
  `hierarchy_parent_composed_outputs = 36`, and
  `top_parent_composed_outputs = 18`. Current HEAD now also has focused
  mixed-depth recursive proof at
  `/tmp/anvil-hier-mixed-depth-smoke-r1/manifest.json`, where the
  design metrics show
  `realized_min_leaf_depth = 2`,
  `realized_max_leaf_depth = 3`, and
  `leaf_module_occurrences_by_depth = {"2": 2, "3": 4}` with clean
  Verilator plus both repo-owned Yosys modes. Current HEAD also has a
  focused clean profiled on-demand proof at
  `/tmp/anvil-hier-profiled-ondemand-smoke-r1/manifest.json`, where
  the design metrics show `num_profiled_instance_slots = 3`,
  `profiled_instance_fraction = 1.0`,
  `profiled_instantiated_module_fraction = 1.0`, and
  `dep_bearing_child_input_binding_fraction = 1.0`. Current HEAD also
  has a focused clean parent-composed child-input proof at
  `/tmp/anvil-hier-child-input-cone-smoke-r1/manifest.json`, where the
  design metrics show
  `child_input_bindings_from_parent_composed_logic = 13` and
  `parent_composed_child_input_binding_fraction = 0.9285714285714286`.
  Current HEAD also has a focused clean parent-cone helper-instance
  proof at `/tmp/anvil-parent-cone-instance-smoke-r1/manifest.json`,
  where the design metrics show `top_parent_cone_instances = 1`,
  `hierarchy_parent_cone_instances = 1`,
  `child_input_bindings_from_parent_cone_instances = 4`,
  `top_child_input_bindings_from_parent_cone_instances = 4`,
  `parent_cone_instance_child_input_binding_fraction = 0.4444444444444444`,
  and
  `top_parent_cone_instance_child_input_binding_fraction = 0.4444444444444444`.
  Current HEAD also has a focused clean parent-output helper-instance
  proof through
  `cargo test hierarchy_parent_outputs_can_depend_on_helper_instance_outputs`,
  where the design metrics show helper instances are additional to the
  planned child slots and top outputs reach those helper outputs through
  `top_outputs_reaching_parent_cone_instances` /
  `hierarchy_outputs_reaching_parent_cone_instances`.
  Current HEAD also has a focused clean budgeted helper proof through
  `cargo test hierarchy_parent_cone_helper_budget_allows_multiple_helpers`,
  where `--max-parent-cone-instances-per-module 3` produces
  `top_parent_cone_instances = 3` and
  `max_parent_cone_instances_per_internal_module = 3`.
  Current HEAD also has a focused clean parent-state proof at
  `/tmp/anvil-hier-parent-state-smoke-r1/manifest.json`, where the
  design metrics show `hierarchy_parent_local_flops = 8`,
  `top_local_flops = 8`, `top_clock_inputs = 1`,
  `top_reset_inputs = 1`, and
  `child_input_bindings_from_parent_flops = 1`.
  Current HEAD also has a focused clean registered parent-composed
  child-input proof at
  `/tmp/anvil-hier-registered-child-input-cone-smoke-r2/manifest.json`,
  where the design metrics show
  `child_input_bindings_from_registered_parent_composed_logic = 3`,
  `top_child_input_bindings_from_registered_parent_composed_logic = 3`,
  `registered_parent_composed_child_input_binding_fraction = 0.75`,
  `top_registered_parent_composed_child_input_binding_fraction = 0.75`,
  and `hierarchy_parent_local_flops = 3`.
  Current HEAD also has a focused clean registered mixed-support
  child-input proof at
  `/tmp/anvil-hier-registered-mixed-child-input-smoke-r1/manifest.json`,
  where the design metrics show
  `child_input_bindings_from_registered_mixed_support = 3`,
  `top_child_input_bindings_from_registered_mixed_support = 3`, and
  `registered_mixed_support_child_input_binding_fraction = 0.75`.
  Current HEAD also has a focused clean multi-stage registered
  parent-composed child-input proof at
  `/tmp/anvil-hier-registered-multistage-child-input-smoke-r1/manifest.json`,
  where the design metrics show
  `child_input_bindings_from_registered_multistage_parent_composed_logic = 2`,
  `top_child_input_bindings_from_registered_multistage_parent_composed_logic = 2`,
  and
  `registered_multistage_parent_composed_child_input_binding_fraction = 0.5`.
  Current HEAD also has a focused clean mixed parent-output proof at
  `/tmp/anvil-hier-parent-output-mix-smoke-r1/manifest.json`, where
  the design metrics show `top_parent_port_composed_outputs = 8`,
  `hierarchy_parent_port_composed_outputs = 8`,
  `top_outputs_reaching_instance_outputs = 8`, and
  `top_outputs_without_instance_outputs = 0`.
  Earlier current-code coverage-only Phase 4 matrix probes at
  `/tmp/anvil-tool-matrix-phase4-parent-port-coverage-r1/tool_matrix_report.json`,
  `/tmp/anvil-tool-matrix-phase4-registered-mixed-r1/tool_matrix_report.json`,
  and
  `/tmp/anvil-tool-matrix-phase4-registered-multistage-r1/tool_matrix_report.json`,
  and
  `/tmp/anvil-tool-matrix-phase4-parent-cone-instance-r1/tool_matrix_report.json`
  remain useful focused policy breadcrumbs, but the full
  downstream-clean `r21` bank above now carries those coverage facts with
  Verilator and both repo-owned Yosys modes enabled.
  Current HEAD has also added repo-owned Phase 4 matrix scenarios for
  parent-output helper-instance composition and budgeted helper
  allocation; the next full Phase 4 bank should refresh the historical
  `r21` counts from 33 scenarios / 132 designs to the now-planned
  39 scenarios / 156 designs.
  The next honest
  work is deeper hierarchy capability beyond the banked gate:
  broader helper-instance placement beyond the current child-input,
  parent-output, and per-parent-budget slices, broader registered
  hierarchy patterns, and later hierarchy-aware identity.
  Parameterization and broader artifact-family selection are still
  roadmap work. See
  `ROADMAP.md` for phase gating.

## Maintenance rule
`README.md` is updated whenever project entry-point information changes materially (objective, ramp-up flow, key paths, or CLI surface). It does not need updates for every commit.

## License
Licensed under either of:
- Apache License, Version 2.0
- MIT License

at your option.

Read `SESSION_BOOTSTRAP.md` and start from there.
