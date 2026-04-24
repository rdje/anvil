# Roadmap

`anvil` grows in phases. Each phase delivers a working generator with a
larger expressive subset. No phase should land without end-to-end tests
and at least one `.sv` artifact run through Yosys or Verilator as a
synthesizability smoke check. Those sweeps are evidence, not the end
goal: the intended steady-state is that generated modules are boringly
clean in Verilator and Yosys by default.

That quality bar coexists with breadth. `anvil` is meant to grow into a
signoff-grade random synthesizable RTL generator that finds bugs in
downstream tools by feeding them legal, unusual, feature-rich designs,
not by relying on malformed input or low-quality noise.

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

3. **Signoff-quality tool-clean industrialization**
   Seed-level cleanliness is not enough. The project needs automated
   Verilator/Yosys evidence across seeds, construction strategies,
   identity modes, factorization levels, category mixes, flop/no-flop
   cases, and deeper hierarchy/memory/FSM features. Counterexamples must
   be retained with exact seed+config evidence and fed back into IR
   invariants or rewrites, not hidden behind warning suppressions. The
   intended steady-state remains: generated RTL is boringly clean in
   mainstream tools by default.

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
    parent-composed child-input cones, registered child-input D cones,
    and parent-output cones can instantiate one helper child as an
    internal parent-cone source, then route its output through parent
    logic.
    `--max-parent-cone-instances-per-module <N>` controls the
    per-parent helper budget; default `1` preserves the first helper
    slice, and focused tests now prove budget `3` is reachable.
- Current slice constraints:
  - direct sibling routing is still combinational; the first one-flop
    registered sibling route and the first registered parent-composed
    child-input route are live, and the registered parent-composed
    route now has mixed parent-port / child-output support plus a first
    multi-stage parent-composed chaining subcase. Broader multi-stage
    registered hierarchy patterns remain future work
  - the fully banked repo-owned Phase 4 matrix now covers both the
    wrapper lane and the representative recursive lane, including the
    mixed-depth recursive axis, the explicit child-sourcing axis, local
    parent state, registered sibling routing, and registered
    parent-composed child-input routing, registered mixed-support
    routing, multi-stage registered parent-composed routing, mixed
    parent-port / child-output parent outputs, and parent-cone
    helper-instance child-input routing. Current HEAD adds
    parent-cone helper-instance parent-output composition, budgeted
    multi-helper allocation, and registered helper-sourced child-input
    D cones as focused post-`r21` slices and matrix-plan axes.
  - module names are now allocated from one generator-global sequence
    across leaf modules, recursive parent modules, and repeated
    hierarchical designs in one output run, so multi-file hierarchy
    output cannot collide or overwrite module definition files by
    reusing a name
- Open Phase 4 work:
  - broaden helper-instance placement beyond the current
    parent-composed child-input, registered child-input, parent-output,
    and per-parent-budget slices
  - deeper parent-side routing/composition beyond the current mixed
    parent-output, combinational sibling-binding, and parent-input-cone
    surfaces
  - richer registered child-to-child and parent-composed routing using
    the landed local parent-state surface
  - hierarchical identity as future required work: under
    `identity_mode = node-id`, equivalent instantiated structures
    should eventually participate in the same sharing story instead of
    creating a second identity system beside gates/flops

**Repo-owned Phase 4 hierarchy closure (met locally):** the refreshed
hierarchy gate now exists at
`/tmp/anvil-tool-matrix-phase4-hierarchy-r21/tool_matrix_report.json`
with multi-file output, correct top declaration, design-level
validation, representative wrapper and recursive profiles,
`coverage_gaps = []`, and clean Verilator + Yosys
elaboration/synthesis on the broadened hierarchy matrix
(`132/0` in Verilator plus both repo-owned Yosys modes). That report
proves all of the then-banked representative hierarchy axes directly:
- wrapper exact / reuse / under-instantiation profiles
- recursive depth `2`
- mixed recursive depth range `2:3`
- explicit child-sourcing modes `library` and `on-demand`
- child-instance profiles `2`, `4`, `2:3`, and `1:3`
- registered sibling-routed hierarchy child inputs through parent-local
  state
- registered parent-composed hierarchy child-input bindings through
  parent logic plus parent-local state
- registered mixed-support child-input bindings that mix parent data
  ports with child outputs
- multi-stage registered parent-composed child-input bindings that chain
  through earlier parent-local Qs
- parent-cone helper instances that source parent-composed child-input
  bindings
- parent-cone helper instances that source registered parent-composed
  child-input D cones
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
remain useful targeted policy breadcrumbs for the mixed parent-output,
registered mixed-support, multi-stage registered, and parent-cone
helper-instance slices. They were
run with Verilator/Yosys skipped; the full downstream-clean `r21` bank
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
scenario set now has a dedicated
`phase4_hier2_inst4_parent_output_cone_instance` axis as well.

**Focused budgeted helper-instance proof (new targeted evidence):**
current HEAD lets one hierarchy parent instantiate more than one
parent-cone helper child when
`max_parent_cone_instances_per_module` is raised. The focused
regression is
`cargo test hierarchy_parent_cone_helper_budget_allows_multiple_helpers`;
the design metrics prove budget `3` via `top_parent_cone_instances`
and `max_parent_cone_instances_per_internal_module`. The repo-owned
Phase 4 scenario set now has a dedicated
`phase4_hier2_inst4_parent_cone_instance_budget3` axis too. Together
with the parent-output helper axis and the registered helper axis below,
the next full downstream-clean bank should refresh the historical `r21`
counts from 33 scenarios / 132 designs to 42 scenarios / 168 designs.

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
The repo-owned Phase 4 scenario set now has a dedicated
`phase4_hier2_inst4_registered_parent_cone_instance_state` axis too.

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
helper-instance bank, and `r21` is the latest full downstream-clean
Phase 4 hierarchy closure artifact. The current scenario plan has
parent-output helper-instance and budgeted helper-instance axes after
`r21`.

Current-code coverage-only probes after `r19` first aligned the gate
policy with newer focused slices: `/tmp/anvil-tool-matrix-phase4-parent-port-coverage-r1/tool_matrix_report.json`
requires mixed parent-output composition, and
`/tmp/anvil-tool-matrix-phase4-registered-mixed-r1/tool_matrix_report.json`
requires registered mixed-support child-input routing, and
`/tmp/anvil-tool-matrix-phase4-registered-multistage-r1/tool_matrix_report.json`
requires multi-stage registered parent-composed routing. All record
`coverage_gaps = []` with Verilator/Yosys skipped; `r20` folded those
three coverage facts into the full downstream-clean bank, and `r21`
adds the parent-cone helper-instance coverage fact as well.

**Phase 4 still remains in progress** because the phase is broader than
the current landed slice. The remaining substantive work is to continue
with broader helper-instance placement beyond the current unregistered
child-input, registered child-input, parent-output, and
per-parent-budget slices, richer registered
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
  the legal interaction richness needed for ANVIL to become a strong
  downstream bug finder without sacrificing clean-tool quality.

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
  shadow RTL semantics engine. The goal is still to stress downstream
  tools aggressively, but by generating high-quality legal RTL and
  explicit expected-facts contracts rather than by turning `anvil` into
  a second simulator.
