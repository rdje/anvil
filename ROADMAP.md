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
- oracle-backed micro-design corpora are new artifact families,
  not a weakening of the existing lane; and
- broader source-level frontend/elaboration artifacts also remain
  valid-by-construction and synthesizable, not a license to blur
  invalid files into the project scope.

The first explicitly-requested families beyond the current lane are now
delivered:

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
   combinational identity plus a closed bounded semantic fragment at the
   `e-graph` rung for small-support gate cones over the same canonical
   endpoints, including gate-to-existing-endpoint / constant folds when
   helper endpoints cancel out and tiny 12-endpoint-bit proofs that fit
   the current node/work budget, together with
   conservative post-drain flop merging and deterministic generated-FSM
   merging over the same endpoint-preserving proof discipline, plus the
   opt-in `bisimulation_flop_merge` pass (`IDENTITY-DEEPENING.2b`) that
   merges flops proven sequentially equivalent *up to a state
   correspondence* — e.g. mutually-recursive registers — via bounded
   greatest-fixpoint bisimulation over that same proof budget (default-off
   / byte-identical, node-id / e-graph only, resetless flops excluded);
   broader *whole-module* sequential equivalence, retimed-state
   equivalence, memory-state merging beyond the current
   instance-local proof boundary, and hierarchy equivalence beyond
   canonical structural module signatures are still open work. The next
   step on the whole-module axis is now **designed but not yet
   implemented**: decision
   [`0008`](docs/decisions/0008-identity-deepening-whole-module-sequential-equivalence.md)
   (`IDENTITY-DEEPENING.3a`) fixes the soundness discipline, budget, and
   downstream gate for bounded whole-leaf-module sequential equivalence —
   a default-off pass *beside* `dedup_semantic_modules` that proves two
   stateful (flops-only) leaf modules observationally equivalent via a
   cross-module bisimulation (the `.2b` partition refinement lifted to the
   disjoint union of the two modules' flops, primary inputs unified by
   `(PortId, width)`) plus bounded output-cone equality under the quotient,
   reusing the same 12-bit/128-node/131072-work budget. The implementation
   is the named future leaf `IDENTITY-DEEPENING.3b`; memory / FSM / wrapper
   / retimed-state module equivalence remain excluded boundaries. Current
   inferrable memories deliberately remain state-by-instance because
   their stored contents are not reset-defined, and current module dedup
   deliberately does not merge semantically-equivalent-but-structurally
   different module bodies.
   This mode must remain
   user-controllable from the CLI:
   `--identity-mode relaxed` is the real semantic off-switch.
   Within `node-id`, `--factorization-level` remains an
   implementation/proof-depth and stress-coverage dial while the build
   climbs toward the doctrine; it must not be treated as redefining what
   `node-id` means.

3. **Signoff-quality downstream-acceptance industrialization**
   Seed-level cleanliness is not enough. The project needs automated
   Verilator/Yosys validation evidence across seeds, construction
   strategies, identity modes, factorization levels, category mixes,
   flop/no-flop cases, and deeper hierarchy/memory/FSM features, plus
   broader optional simulator/frontend acceptance columns where they
   are available. `tool_matrix --iverilog-compile` now adds Icarus
   Verilog compile/elaboration acceptance (`iverilog -g2012`) as an
   opt-in warning-clean column; the focused `SIGNOFF-SURFACE-EXPANSION.3`
   smoke is clean at 17/0 across Verilator, both repo-owned Yosys
   modes, and Icarus compile.
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

   Progress (`SIGNOFF-AUTOMATION-EXPANSION.2b`, `2026-06-15`): the first
   batch of previously-implicit knobs is now promoted into explicit
   first-class matrix axes under `tool_matrix --signoff-knob-sweep-gate`
   — `operand_duplication_rate`, `mux_arm_duplication_rate`,
   `aggregate_array_prob`, and the memory×fsm interplay — each with one
   focused scenario per construction strategy and a `saw_*` coverage
   fact so it fires by construction, not by chance. Banked
   downstream-clean at `/tmp/anvil-signoff-knob-sweep-r1` (12 scenarios,
   48 modules, `coverage_gaps = []`, `48/0` Verilator + both Yosys).
   Remaining knobs/axes and the higher-ceiling paths (new acceptance
   columns; non-DUT lanes under the acceptance columns) stay named
   future leaves of that lane — nothing retired.

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

## Owner-directed capability lanes (2026-06-15)

Beyond the closed numbered phases and the open post-phase identity lane
(`IDENTITY-DEEPENING`), owner roadmap steering on `2026-06-15` named three
new capability lanes, each now task-tree-owned (`docs/TASK_TREE.md`):

1. **`SV-VERSION-TARGETING`** (`active`, recommended highest-leverage) — an
   opt-in `--sv-version <2012|2017|2023>` gate (`Config::sv_version`) that
   targets a chosen IEEE 1800 standard valid-by-construction: **down-gating**
   (never emit a construct newer than the target — a standard-validity
   guarantee) and **up-opting** (deliberately emit a higher standard's
   distinctive synthesizable constructs, each proven downstream-clean in the
   matching tool standard mode). Default byte-identical; rules-first; a new
   per-version downstream acceptance axis. Decision
   [`0009`](docs/decisions/0009-sv-version-targeting.md); this directly serves
   the north star (expose version-specific downstream-tool bugs) and adds an
   explicit `sv_version` adversarial axis (steering gap 3) plus version-targeted
   breadth (steering gap 1).
2. **`STRUCTURED-EMISSION-EXPANSION`** (`proposed`) — richer structured
   synthesizable SV surfaces (function/task, interface/modport, nested
   generate), valid-by-construction; bigger and more open-ended.
3. **`SEMANTIC-INTROSPECTION-EXPANSION`** (`proposed`) — a behavioral / derived
   query surface beyond today's structural/metric projection, kept
   `SCHEMA-DERIVED` (no new oracle), extending `AGENT-INTROSPECTION-MCP` /
   `AGENT-MCP-EXPANSION`.

Nothing is retired; all three are tracked task trees, and the open
`IDENTITY-DEEPENING.3b.2b` frontier (cross-module whole-module sequential
equivalence) is parked-but-fully-designed, not abandoned.

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
  combinational cones over the same canonical leaf variables, with
  12-bit shallow truth-table proofs admitted only inside the current
  node/work budget.
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
  combinational case-style blocks: dynamic selectors emit
  `always_comb case` / `always_comb casez`, while constant selectors
  lower to continuous `assign` statements.**
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
  packed chunks via `for_fold_prob` and `GateOp::ForFold`; constant
  packed sources lower to a continuous `assign` of the folded literal.
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

## Phase 4 — Hierarchy (done)

**Status:** `done` as of `2026-05-16`. Both Phase 4 task trees are
complete: [`HIERARCHY-AWARE-IDENTITY`](docs/tasks/HIERARCHY-AWARE-IDENTITY.md)
(all five leaves, r85–r87 — the doctrine "NodeId = identity of an
expression" now extends to "ModuleId = identity of a hierarchical module
template" under the opt-in `Config::hierarchy_module_dedup` knob) and
[`PHASE-4-HIERARCHY`](docs/tasks/PHASE-4-HIERARCHY.md) (the
Surface-Inventory audit `.1` proved the instrumented hierarchy surface
is fully landed-proven, and `.3` closed Phase 4 against the explicit
exit criteria below). Phase 4's `rN`-named linear coverage slices landed
under the rN cadence; multi-slice sub-objectives were task-tree-managed
per [docs/TASK_TREE.md](docs/TASK_TREE.md). Phase 4 is **closed by a
deliberate, evidence-backed scope cut** (see `PHASE-4-HIERARCHY`
Decisions): the implemented surface is declared a sufficient real
design/instance layer; the residual "broader registered hierarchy
patterns" is open-ended capability-deepening with no completion point
and is **not** a Phase 4 blocker. No mode or strategy was retired — every
implemented route remains; further breadth, if ever pursued, lands as
post-Phase-4 `rN` slices without reopening the phase.

Post-closure identity follow-up `HIERARCHY-DEDUP-PRUNE.1` tightened the
opt-in module-dedup cleanup: after a real structural merge, definitions
that were reachable before dedup but become unreachable from the top are
pruned; no-merge calls are preserved, and pre-existing
under-instantiated library modules are not removed by the reachability
cleanup unless the structural dedup merge itself collapses them.

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
    `/tmp/anvil-tool-matrix-phase4-hierarchy-r87/tool_matrix_report.json`.
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
    recursive non-top direct registered sibling mixed-support routing, and recursive non-top unregistered parent-composed mixed-support child-input routing without helper instances, plus recursive non-top stateful parent-port-composed parent-output routing without helper instances, plus recursive non-top stateful unregistered parent-composed mixed-support child-input routing through parent-local Qs without helper instances, plus recursive non-top parent-local flops gated as a first-class coverage fact, plus recursive parent-local flops at exact hierarchy depth 3, plus recursive non-top unregistered parent-composed mixed-support child inputs at exact hierarchy depth 3 without helpers, plus recursive non-top parent-port-composed parent outputs at exact hierarchy depth 3 without helpers or state, plus recursive non-top stateful parent-port-composed parent outputs at exact hierarchy depth 3 without helpers, plus recursive non-top stateful parent-composed mixed-support child inputs at exact hierarchy depth 3 without helpers, plus recursive non-top parent-local flops at exact hierarchy depth 4, plus recursive non-top mixed-support child inputs at exact hierarchy depth 4 without helpers, plus recursive non-top parent-port-composed parent outputs at exact hierarchy depth 4 without helpers or state, plus recursive non-top stateful parent-port-composed parent outputs at exact hierarchy depth 4 without helpers, plus recursive non-top stateful parent-composed mixed-support child inputs at exact hierarchy depth 4 without helpers, plus recursive non-top parent-local flops at exact hierarchy depth 5, plus recursive non-top mixed-support child inputs at exact hierarchy depth 5 without helpers, plus recursive non-top parent-port-composed parent outputs at exact hierarchy depth 5 without helpers or state, plus recursive non-top stateful parent-port-composed parent outputs at exact hierarchy depth 5 without helpers, plus recursive non-top stateful unregistered parent-composed mixed-support child inputs at exact hierarchy depth 5 without helpers — closing the depth-5 sweep, plus recursive non-top parent-local flops at exact hierarchy depth 6 — opening the depth-6 axis, plus recursive non-top mixed-support child inputs at exact hierarchy depth 6 without helpers, plus recursive non-top parent-port-composed parent outputs at exact hierarchy depth 6 without helpers or state, plus recursive non-top stateful parent-port-composed parent outputs at exact hierarchy depth 6 without helpers, plus recursive non-top stateful unregistered parent-composed mixed-support child inputs at exact hierarchy depth 6 without helpers (2,2 calibrated) — closing the depth-6 sweep, plus recursive non-top parent-local flops at exact hierarchy depth 7 — opening the depth-7 axis, plus recursive non-top mixed-support child inputs at exact hierarchy depth 7 without helpers (2,2 calibrated), plus recursive non-top parent-port-composed parent outputs at exact hierarchy depth 7 without helpers or state, plus recursive non-top stateful parent-port-composed parent outputs at exact hierarchy depth 7 without helpers, plus recursive non-top stateful unregistered parent-composed mixed-support child inputs at exact hierarchy depth 7 without helpers (2,2 calibrated) — closing the depth-7 sweep, plus recursive non-top registered parent-composed child-input bindings that chain through at least three parent-local flop stages without helpers, plus a recursive non-top internal parent saturating a parent-cone helper budget of 5 helpers, plus per-module canonical signatures as the first slice of hierarchy-aware identity instrumentation. The `r85`
    report records `204` scenarios, `4` designs/scenario, `816` total designs,
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
    `saw_recursive_hierarchy_canonical_module_signature_diversity = true`,
    and `816/0` pass-fail in Verilator plus both repo-owned Yosys modes.
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

**Exit criteria (met):** Phase 4 closes when all of the following hold,
each visible in a repo-owned artifact (not narrative):

1. ANVIL emits real multi-file hierarchical `Design`s — a declared top
   plus instantiated children, design-level validated — across both the
   legacy exact wrapper lane and the bounded recursive lane, including
   mixed leaf depth and per-depth branching. *(met: r87, `artifact_kind
   = "design"`, recursive + wrapper scenarios)*
2. Both child-sourcing modes (`library` reuse/under-instantiation and
   `on-demand` profiled child synthesis) are exercised. *(met: r87
   `saw_on_demand_child_sourcing`, `saw_profiled_child_interface_synthesis`,
   `saw_reused_child_definition`, `saw_underinstantiated_library`)*
3. The full instrumented hierarchy routing/composition surface
   (combinational + registered sibling and parent-composed child-input
   binding, parent-cone helper instances and multi-helper budgets,
   parent-local flops, parent-port-composed parent outputs, recursive
   non-top variants, hierarchy-aware identity/dedup) is proven by gated
   coverage facts. *(met: r87 — 92 Phase4-gated hierarchy `saw_*` facts
   all `true`, `coverage_gaps = []`)*
4. The closing matrix gate is downstream-clean. *(met: r87 — 210
   scenarios, 840 designs, `840/0` in Verilator and both repo-owned
   Yosys modes)*
5. The surface is a sufficient real design/instance layer to unblock
   Phase 5 parameterization. *(met: criteria 1–4 jointly satisfy the
   Phase 5 hard prerequisite; remaining parameter-aware work belongs to
   Phase 5 by roadmap decree, not Phase 4)*

Scope note: "broader registered hierarchy patterns" is open-ended
capability-deepening, not an exit criterion; per the `PHASE-4-HIERARCHY`
audit it has no finite completion point and is explicitly excluded from
the Phase 4 bar (no mode/strategy retired; future breadth is optional
post-Phase-4 `rN` work). Closing artifact:
`/tmp/anvil-tool-matrix-phase4-hierarchy-r87/tool_matrix_report.json`.

**Repo-owned Phase 4 hierarchy closure (latest full bank met locally):** the refreshed
hierarchy gate now exists at
`/tmp/anvil-tool-matrix-phase4-hierarchy-r87/tool_matrix_report.json`
with multi-file output, correct top declaration, design-level
validation, representative wrapper and recursive profiles,
`210` scenarios, `840` total designs, `coverage_gaps = []`, and clean Verilator + Yosys
elaboration/synthesis on the broadened hierarchy matrix
(`840/0` in Verilator plus both repo-owned Yosys modes). The `r87` report
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
the full downstream-clean `r49` report first proved it, `r50` banked the accumulated mixed-support surface, and `r51` through `r85` carry it forward with
`coverage_gaps = []` and `816/0` pass-fail in Verilator plus both
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
`r84` is the previous full downstream-clean 201-scenario recursive
parent-cone helper budget 5 gated coverage hierarchy artifact,
extending the helper-budget axis above the previous budget-3 baseline.
`r85` is the current full downstream-clean 204-scenario hierarchy
artifact that adds per-module canonical signatures as the first slice
of hierarchy-aware identity instrumentation.

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
201-scenario matrix. `r85` lands the first slice of hierarchy-aware
identity by adding per-module canonical signatures
(`DesignMetrics.canonical_module_signatures`, `num_distinct_module_signatures`,
`num_structurally_duplicate_module_pairs`) plus a focus scenario and
the new `saw_recursive_hierarchy_canonical_module_signature_diversity`
fact on the expanded 204-scenario matrix.

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

## Phase 5 — Parameterization (done)

**Status:** `done` as of `2026-05-17`. Delivered by the
[`PHASE-5-PARAMETERIZATION`](docs/tasks/PHASE-5-PARAMETERIZATION.md)
task tree (`.1` design → `.2.1` scaffold → `.2.2.*` soundness gate,
rules-first parameterizable-leaf constructor, instantiation
substitution → `.2.3` parameter-aware identity → `.2.4` matrix gate).
Original intent of the phase:

- Generated modules take `parameter` declarations for widths.
- Instantiation picks parameter values from allowed ranges.
- Parameter-dependent widths propagate correctly through cone generation.
- Parameter-aware identity must remain sound: different parameter values
  cannot accidentally alias to one `NodeId` or one module template
  unless the resulting structure is genuinely equivalent.
- IR-level design recorded in `book/src/ir.md` "Future extensions /
  Parameters and generics" and `DEVELOPMENT_NOTES.md` "Phase 5
  parameterization design".

**Exit criteria (met):** Phase 5 closes when all of the following hold,
each visible in a repo-owned artifact (not narrative):

1. ANVIL emits modules with a real width `parameter`
   (`module M #(parameter int W = D) (input [W-1:0] …, output
   [W-1:0] …)`), valid by construction, built rules-first (the
   width-homogeneous `build_parameterizable_leaf`, not
   generate-then-filter). *(met: `width_parameterization_prob` opt-in;
   `is_width_generic` soundness gate; default-off byte-identical.)*
2. Instances pick parameter values reproducibly from the allowed range
   and override via `#(.W(v))`, with the design still passing
   `validate_design` (resolved-width child-port checks). *(met:
   `Instance.param_bindings`; per-instance `g.rng` pick; focused proof
   `width_parameterization_instances_override_at_multiple_values` shows
   ≥2 distinct in-range overrides of one template across all four
   `ConstructionStrategy` values.)*
3. Parameter-aware identity is sound: two parameterizable templates
   differing only in design width share one canonical signature; a
   concrete module never aliases a parameterized one; `dedup_modules`
   unchanged; the H-A-I.1/.2/.4 regressions still pass. *(met:
   `.2.3` `canonical_module_signature` marker + width sentinel.)*
4. A repo-owned matrix gate proves parameterized designs
   downstream-clean. *(met: `/tmp/anvil-tool-matrix-phase5-p1/tool_matrix_report.json`
   — 213 scenarios, 852 designs, `coverage_gaps = []`,
   `saw_width_parameterized_design = true`, Verilator 852/0,
   Yosys-without-abc 852/0, Yosys-with-abc 852/0.)*

Scope note: parameter-aware *child selection* and *parameter-driven
parent generation* (a parent choosing children/structure as a function
of a parameter) are open-ended capability-deepening with no finite
completion point — they are **not** a Phase 5 blocker (same
deliberate, evidence-backed scope-cut doctrine used to close Phase 4).
No mode/strategy retired; default-off keeps every prior artifact
byte-identical; further breadth, if pursued, lands as optional
post-Phase-5 `rN`/task-tree work without reopening the phase. Closing
artifact: `/tmp/anvil-tool-matrix-phase5-p1/tool_matrix_report.json`.

## Phase 5b — Synthesizable aggregates (done)

**Status:** done as of 2026-05-18 (`PHASE-5B-AGGREGATES` tree
complete). Scheduled alongside Phase 5; order was not fixed.

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

**Exit criteria (met):**

1. **Packed-aggregate emission as a flat-IR projection, valid by
   construction.** Architecture (P): an additive `Default`-able
   `Module.aggregate_layout` annotation consulted only by the emitter;
   construction, validators, CSE keys and `canonical_module_signature`
   are all untouched. A packed `struct` is LRM-defined bit-equivalent
   to the concatenation of its members, so the boundary-alias
   projection (`typedef struct packed` + one aggregate port/side +
   alias wires/assigns) is a semantically-empty regrouping. Opt-in
   `Config::aggregate_prob` (serde-default `0.0`); default-off is
   byte-identical for fixed seeds across all `ConstructionStrategy`
   values (`PHASE-5B-AGGREGATES.2.1`).
2. **Not inert; sound identity.** Organic existence proven — with
   default port ranges the projection fires on ~85 % of organic
   single-module designs, so no rules-first pivot was needed (unlike
   Phase 5 width-homogeneity). The annotation is deliberately **not**
   hashed into module identity: a module and its aggregate-projected
   twin share one `canonical_module_signature` and dedup-collapse
   (`PHASE-5B-AGGREGATES.2.2`).
3. **Repr matrix gate, verified downstream-clean.** The repo-owned
   `Phase4Hierarchy` gate, now including the `phase5b_packed_aggregate`
   scenario, ran to completion (background, exit 0) and the banked
   artifact `/tmp/anvil-tool-matrix-phase5b-p1/tool_matrix_report.json`
   was verified CLEAN: **216 scenarios / 864 designs**,
   `coverage_gaps = []`, **Verilator 864/0**, **Yosys without-abc
   864/0**, **Yosys with-abc 864/0**, `saw_packed_aggregate_design =
   true`; Phase 5 (`saw_width_parameterized_design`) and Phase 4
   (`saw_recursive_hierarchy_module_dedup_active`) regressions stay
   clean (`PHASE-5B-AGGREGATES.2.3` scenario/metrics/gap,
   `PHASE-5B-AGGREGATES.2.4` verify + promote).

**Scope note (not blockers; no mode retired):** the `.2.1` scaffold is
deliberately scoped to `AggregateKind::StructPacked` only
(`UnionPacked`/`ArrayPacked` need a same-width group), to
**non-instantiated** modules (parent-side aggregate connections are
deferred so hierarchy child connections stay byte-identical), and
skips Phase 5 `param_env` modules (the param/aggregate cross-product
is deferred). These are open-ended capability deepenings recorded in
the tree's Decisions; they are explicitly **not** Phase 5b blockers
and land as optional post-Phase-5b sub-slices without reopening the
phase. Unpacked arrays remain Phase 6; unpacked datapath/enums stay
deprioritised. The `ArrayPacked` sub-slice is now task-tree-owned and
in progress under
[`AGGREGATE-ARRAY-PACKING`](docs/tasks/AGGREGATE-ARRAY-PACKING.md)
(default-off `aggregate_array_prob`; does not reopen Phase 5b);
`UnionPacked` stays deferred because a union aliases distinct ports and
is not a faithful projection.

## Phase 6 — Advanced motifs (done)

- Memories (single-port, dual-port, with inferrable patterns only).
  **Memory motif delivered (2026-05-18, `PHASE-6-ADVANCED-MOTIFS.2`
  container done).** A first-class `Memory` block (additive
  `Default`-empty `Module.memories`) + an opaque `Node::MemRead`
  leaf (sibling to `FlopQ`, never CSE'd; load-bearing `compact.rs`
  reachability) renders the empirically-validated synchronous
  write / registered read template Yosys infers as `$mem_v2`,
  behind the opt-in `Config::memory_prob` (serde-default `0.0` →
  byte-identical). Verified downstream-clean against the real
  repo-owned `Phase4Hierarchy` gate — closing artifact
  `/tmp/anvil-tool-matrix-phase6-p1`: **219 scenarios / 876
  designs, `coverage_gaps = []`, 876/0 Verilator + both Yosys
  (`without_abc`/`with_abc`), `saw_inferrable_memory_design =
  true`**, with Phase 4/5/5b regressions still proven in the same
  banked artifact (`saw_width_parameterized_design`,
  `saw_packed_aggregate_design`,
  `saw_recursive_hierarchy_module_dedup_active` = true).
  `SinglePort` + `SimpleDualPort` only; `param_env`-skipped /
  non-instantiated library leaves are the scaffold scope (recorded,
  not blockers). Memory delivery **advances** Phase 6; it does not
  close it.
- FSMs with explicitly generated state encodings. **FSM motif
  delivered (2026-05-20, `PHASE-6-ADVANCED-MOTIFS.3` container done
  → tree CLOSED).** A first-class `Fsm` block (additive
  `Default`-empty `Module.fsms`) + an opaque `Node::FsmOut` leaf
  (sibling to `FlopQ`/`MemRead`, never CSE'd; identical
  `compact.rs` reachability obligation as `MemRead`) +
  encoding-derived `localparam` state constants + async-reset
  state register + `always_comb` next-state / Moore-output `case`s
  on the shared `clk`/`rst_n`, behind the opt-in `Config::fsm_prob`
  (serde-default `0.0` → byte-identical). Verified downstream-clean
  against the real repo-owned `Phase4Hierarchy` gate — closing
  artifact `/tmp/anvil-tool-matrix-phase6-fsm-p1`: **222 scenarios
  / 888 designs, `coverage_gaps = []`, 888/0 Verilator + both
  Yosys (`without_abc`/`with_abc`), `saw_fsm_design = true` and
  `saw_inferrable_memory_design = true`**, with Phase 4/5/5b
  regressions still proven in the same banked artifact
  (`saw_width_parameterized_design`,
  `saw_packed_aggregate_design`,
  `saw_recursive_hierarchy_module_dedup_active` = true). All three
  generated encodings (binary / one-hot / gray) are emitted by
  rule; Moore-output only (Mealy is a recorded post-`.3`
  extension, not a Phase-6 blocker).
- Multi-clock with CDC-safe handshakes — optional, expensive. Until
  this lands, every module remains fully synchronous to a single clock.
- These motifs are not just feature-count work; they are a major part of
  the legal interaction richness needed for ANVIL to help find
  downstream tool bugs without sacrificing downstream-acceptance quality.

**Exit criteria (met):** Phase 6 closes when both substantive
sub-objectives — the inferrable-memory motif **and** the
generated-encoding FSM motif — are verified delivered against the
banked `Phase4Hierarchy` gate (`coverage_gaps = []`, all-pass
Verilator + both Yosys, the corresponding `saw_*_design` facts
true, P4/P5/P5b regressions clean in the same artifact). Both
are met: the memory motif against
`/tmp/anvil-tool-matrix-phase6-p1` (219/876, 2026-05-18) and the
FSM motif against `/tmp/anvil-tool-matrix-phase6-fsm-p1` (222/888,
2026-05-20). r87 no-aspirational-claims: the verified reports
precede this promotion. Multi-clock CDC is the
explicitly-optional, separately-prioritised deferral and was
**not** a Phase 6 closure blocker; it remains a future,
separately-prioritised item (every module stays fully synchronous
to a single clock until/unless that lane is taken up).

## Phase 7 — Oracle-backed micro-design artifacts (done)

- A new artifact family for **small, self-contained `.sv` files
  with known expected facts** rather than broad cone complexity.
  **Delivered (2026-05-20, `PHASE-7-ORACLE-MICRODESIGN` tree
  CLOSED.)**
- Initial target landed (`rtl_const_expr`-style corpora):
  - parameter / localparam dependency chains;
  - widths and ranges derived from expressions (`[DEPTH-1:0]`, etc.);
  - generate conditions and loop bounds driven by expressions;
  - package-qualified constant use;
  - precedence-sensitive arithmetic / shift / comparison / equality /
    bitwise / logical / ternary expressions.
- Typical artifact size: one module, or a tiny cluster of modules when
  the pressure point needs local hierarchy.
- Every emitted file gets an **expected-facts manifest** capturing
  parameter values, resolved ranges, generate decisions, and other
  obviously-checkable elaboration facts. The generator IS the
  oracle: every const-expr/parameter value is resolved at
  construction time (one `ChaCha8` stream per seed) and the same
  resolved value is shipped in the manifest + held symbolic in the
  emitted `.sv` (the gap is exactly what front-end elaboration is
  supposed to close). A separate top-level module `src/microdesign/`
  carries the IR + evaluator + emitters; the DUT lane stays
  byte-identical by construction (default-off; never invoked from
  the DUT generate path).

**Exit criteria (met):** Phase 7 closes when a parity gate against
a real downstream consumer reports exact agreement on the
tool-supported fact categories across a reproducible corpus, with
a verified-clean banked artifact (r87 no-aspirational-claims). The
gate is the repo-owned `parity_against_real_yosys_write_json`
`#[ignore]` test in `tests/microdesign_parity.rs`; the cargo-
portable comparator core (`ToolReport`/`Divergence` ×
17 variants/`compare_manifest_to_tool_report_in_scope`) +
`FactCategory`+`ParityScope` live in `src/microdesign/`. Closing
artifact `/tmp/anvil-microdesign-parity-phase7-yosys-p1/` (15
files: 5 × {`mc_<seed>.sv`, `mc_<seed>.json`, `mc_<seed>.yosys.json`}
for the reproducibility seeds `{0, 1, 7, 42, 12345}`): `cargo
test --test microdesign_parity -- --ignored
parity_against_real_yosys_write_json --nocapture` against yosys
0.64 exits 0 with "parity gate clean across 5 seeds" and zero
retained counterexamples. Per-seed fact agreement verified
including the previously-divergent seed 7 (P4 = -1; both sides
report `widths.sig.bits = 8` post-`.2c.2b.1`'s
non-negative-modulo-idiom fix) and both generate branches
(seed 12345 takes `g_else`, the others take `g_taken`).

**Scope caveat (explicit):** yosys 0.64 `write_json` exposes 4 of
the 7 manifest fact categories — Seed/Top/Params/Widths/Generate.
Localparams and package-qualified constants are folded by the
elaborator and not name-introspectable from `write_json` alone;
the parity comparator scopes accordingly. Richer-AST coverage via a
future microdesign-specific AST extractor would surface the
folded categories and is a recorded post-Phase-7 follow-up that
**does NOT** retract Phase 7 closure — ANVIL's by-construction
oracle already covers all 7 categories in the manifest; the parity
gate exercises whatever the tool reports.

**Notable surfacing during closure:** the very first real-tool
run of the parity gate found an ANVIL-self-consistency bug in the
`width_expr` emitter (oracle used `rem_euclid`, SV used `%`;
diverged for negative `last.value`). This is exactly what `.1`
designed the gate to do — surface oracle/downstream semantic
disagreement at the fact-by-fact level, not a single "tool exit
non-zero" bit. The bug was fixed in `.2c.2b.1`, re-run was clean,
ROADMAP promotion strictly followed the verified artifact.

## Phase 8 — Frontend/elaboration accept corpora (done)

- A source-level artifact family for **compact elaboratable
  hierarchies** rather than only the current circuit-IR leaf
  modules. **Delivered (2026-05-20, `PHASE-8-FRONTEND-ACCEPT`
  tree CLOSED.)**
- Initial surfaces landed (the minimum-viable set sufficient to
  stress every elaboration axis the parity gate checks):
  - ANSI parameter lists with default expressions kept symbolic
    in the emit;
  - parameter / localparam chains (top-level parameters; body
    localparams chained over earlier names);
  - module instantiation with **named** parameter-override
    bindings (ordered bindings are a recorded extension, not a
    closure blocker — named is the modern SV style downstream
    tools document best);
  - package-qualified constant use (`acc_<seed>_pkg::K`);
  - `generate if / else` over a parameter predicate.
- Source-level **AST IR** (`SourceUnit` → `Package` → `Module` →
  `ModuleItem`) in `src/frontend/` — a separate generator path
  that never touches the DUT lane (default-off byte-identical,
  per the same rules-first construction-time-evaluator pattern
  Phase 7 established). Cross-tree reuse of Phase 7's `ConstExpr`
  / `eval` / `expr_to_sv` at the expression layer keeps the
  full-factorization doctrine satisfied — Phase 7's hard-won
  non-negative-modulo-idiom fix (the `.2c.2b.1` semantic
  alignment) carries forward for free, which is exactly why
  Phase 8's parity gate came back clean on the first try.
- An **elaborated-facts manifest** carries every fact yosys (or
  any richer-AST tool) needs to verify: per-package localparams,
  top parameter / localparam values + symbolic expressions, the
  full instance tree (`inst_name` → `child_module` → per-binding
  resolved values), and per-label generate-branch decisions.
  `BTreeMap` ordering ⇒ byte-stable JSON.

**Exit criteria (met):** Phase 8 closes when a parity gate
against a real downstream elaborator reports exact agreement on
the tool-supported fact categories across a reproducible corpus,
with a verified-clean banked artifact (r87 no-aspirational-
claims). The gate is the repo-owned
`parity_against_real_yosys_hierarchy_write_json` `#[ignore]` test
in `tests/frontend_parity.rs`; the cargo-portable comparator core
(`ToolReport`/`InstanceToolReport`/`Divergence` × 23 variants
including the hierarchy-aware `Instance*` additions /
`FactCategory` / `ParityScope` /
`compare_manifest_to_tool_report_in_scope`) lives in
`src/frontend/`. Closing artifact
`/tmp/anvil-frontend-parity-phase8-yosys-p1/` (15 files: 5 ×
`{acc_<seed>.sv, acc_<seed>.json, acc_<seed>.yosys.json}` for
the reproducibility seeds `{0, 1, 7, 42, 12345}`): `cargo test
--test frontend_parity -- --ignored
parity_against_real_yosys_hierarchy_write_json` against yosys
0.64 exits 0 with "parity gate clean across 5 seeds" and zero
retained counterexamples. Per-seed fact agreement verified
including both generate branches (seed 12345 takes `g_else`, the
others take `g_taken`) and the load-bearing hierarchy-aware
Phase-8 axis (every seed has 2 instances × 4 per-instance
per-binding values matched against yosys's `.cells[<inst>].parameters`).

**Scope caveat (explicit):** yosys 0.64's `hierarchy + write_json`
exposes 5 of the 7 manifest fact categories — Seed / Top /
TopParams / Instances / GenerateBranches. Top localparams and
package-qualified constants are folded by yosys's elaborator and
not name-introspectable from `write_json` alone; the parity
comparator scopes accordingly via `yosys_hierarchy_scope`.
`SIGNOFF-SURFACE-EXPANSION.2` adds the richer optional Verilator
JSON-AST gate
`tests/frontend_parity.rs::parity_against_real_verilator_json_frontend_ast`
for local Verilator builds that support `--json-only`. It parses
Verilator's specialized child modules and direct package/top
parameter declarations, enforces `ParityScope::all()` across all 7
Phase-8 categories, and is verified clean across the 5 reproducibility
seeds with artifacts in
`target/tmp/frontend-parity-signoff-verilator-json`. `slang` remains
optional and was not present in the local tool environment.

**Notable during closure:** Phase 8's parity gate came back
clean on the **first** real-tool run, **without** needing a
fix-and-retry slice (contrast with Phase 7's `.2c.2b.1`
`width_expr` non-negative-modulo-idiom fix). The reason is
exactly the cross-tree reuse the full-factorization doctrine
asks for: Phase 8's emit composes Phase 7's `expr_to_sv`, so
Phase 7's hard-won expression-layer fix carries forward at zero
incremental cost. An empirical-probe-driven discovery during
`.2c.2`'s split — that yosys's `proc; opt` collapses
empty-bodied child instances out of `.cells` — was the only
Phase-8-specific tool-capability dependency surfaced, and was
folded into `.2c.2a`'s yosys invocation (omit `proc; opt`).

## Phase 9 — Multi-artifact ANVIL umbrella (done)

- An **artifact-family selector** so one tool drives all of the
  valid-by-construction synthesizable families without overloading
  one generator path with contradictory promises. **Delivered
  (2026-05-20, `PHASE-9-MULTI-ARTIFACT-UMBRELLA` tree CLOSED.)**
- Unified reproducibility, manifests, seed handling, knob plumbing,
  corpus output layout, and downstream-check shape across the three
  delivered artifact lanes — via the `pub trait ArtifactLane`
  in `src/umbrella/` (`name`/`validate_knobs`/`generate(seed)`/
  `check_plan`) + the `LaneArtifact{lane, seed, sv,
  manifest: Option<String>}` carrier + the `CheckPlan` enum
  (`SynthAccept` for L1; `ParityVsManifest` for L2/L3).
- Preserves the doctrinal lane separation by design (explicit
  anti-goal recorded in `.1`'s `DEVELOPMENT_NOTES` entry: never
  collapse the three lanes' rules-first generators into one
  "random SV generator"; only their plumbing unifies):
  - L1 **synthesizable DUT RTL lane** — `DutLane` wrapping
    `src/gen/` (Phases 1–6);
  - L2 **oracle-backed micro-design lane** — `MicrodesignLane`
    wrapping `src/microdesign/` (Phase 7);
  - L3 **frontend/elaboration accept lane** — `FrontendLane`
    wrapping `src/frontend/` (Phase 8);
  - future synthesizable lanes plug in by implementing the same
    `ArtifactLane` trait.
- A top-level **`--artifact <lane>` CLI flag** on the `anvil`
  binary (default `dut`) dispatches via `Box<dyn ArtifactLane>`.
  `--artifact dut` falls through to the historical code path
  UNCHANGED (early-return guard at the top of `fn main`); the
  load-bearing **byte-identical default-`dut` contract**
  (`BOOK-EXAMPLES-RUNNABLE` + every CI gate) is preserved by
  construction + verified by
  `tests/book_examples::every_runnable_book_bash_block_succeeds`.

**Exit criteria (met):** ANVIL can honestly present itself as the
go-to tool for pseudo-random HDL artifact generation, with
explicit mode/lane selection instead of one blurred notion of
"random SV files". Closure evidence:

- The `ArtifactLane` trait + all 3 lane impls landed in
  `src/umbrella/` (`PHASE-9-MULTI-ARTIFACT-UMBRELLA.2a`/`.2b`);
- 8 cargo-portable umbrella proofs incl. per-lane byte-identical
  regression (L1/L2/L3) + cross-lane heterogeneous dispatch
  through `Box<dyn ArtifactLane>`;
- `--artifact <lane>` CLI flag landed in `src/main.rs`
  (`.2c`) with the byte-identical default-`dut` early-return;
- `cargo test --test book_examples` ran clean — 3/3 in 80s —
  proving every runnable book bash block still exits 0
  byte-identically through the new code path (load-bearing vs
  `BOOK-EXAMPLES-RUNNABLE`);
- Full `cargo test` green throughout: lib **244**, pipeline 121,
  snapshots 6, microdesign_parity 15+1, frontend_parity 12+2,
  bin tests 5+29+3, doc-tests unchanged.

**Notable during closure:** the cross-lane heterogeneous dispatch
proof (`cross_lane_dispatch_through_dyn_artifact_lane` in `.2b`)
landed BEFORE `.2c`'s CLI work — meaning the CLI dispatch via
`Box<dyn ArtifactLane>` was correct-by-construction the moment it
compiled, because the trait + dispatch contract had already been
proven cargo-portably. The book/CI byte-identical verification
came back clean on the first run, validating the
default-`dut`-early-return-guard architecture.

## All 9 numbered roadmap phases delivered

With Phase 9 closed, every numbered roadmap phase from 0 through
9 is now delivered:

- Phase 0 — initialization (done historically).
- Phase 1 — Single-module MVP (done).
- Phase 2 — Signal sharing / DAG cones (done).
- Phase 3 — Structured combinational ops (done).
- Phase 4 — Hierarchy (done 2026-05-16; closing artifact r87).
- Phase 5 — Width parameterization (done 2026-05-17;
  `/tmp/anvil-tool-matrix-phase5-p1`).
- Phase 5b — Synthesizable aggregates (done 2026-05-18;
  `/tmp/anvil-tool-matrix-phase5b-p1`).
- Phase 6 — Advanced motifs: memories + FSMs (done 2026-05-20;
  `/tmp/anvil-tool-matrix-phase6-p1` + `-phase6-fsm-p1`).
- Phase 7 — Oracle-backed micro-design artifacts (done
  2026-05-20; `/tmp/anvil-microdesign-parity-phase7-yosys-p1/`).
- Phase 8 — Frontend/elaboration accept corpora (done
  2026-05-20; `/tmp/anvil-frontend-parity-phase8-yosys-p1/`).
- Phase 9 — Multi-artifact ANVIL umbrella (done 2026-05-20).

Five post-phase follow-up trees are tracked in `docs/TASK_TREE.md` as
of `2026-06-05`, and all five are now exhausted to their current
proof/tool boundaries:

- `COMBINATIONAL-SEMANTIC-IDENTITY` is closed for the current broader
  same-endpoint combinational semantic identity frontier beyond the
  bounded gate-to-gate merge.
- `SEQUENTIAL-COINDUCTIVE-IDENTITY` is closed for the current bounded
  proof model: flop identity now includes clock/reset domain, exact
  reset-defined self-hold flops can merge, deterministic FSM exact-table
  sharing remains covered, and broader coinduction is blocked until a
  transition-relation proof and/or extra IR domain facts exist.
- `MEMORY-STATE-IDENTITY` is closed for the current memory-inference
  lane: reset-less memories remain state-by-instance, and the
  reset-defined-memory merge candidate is blocked because the reset-all
  array probe is not warning-clean `$mem_v2` memory inference in Yosys.
- `HIERARCHY-SEMANTIC-IDENTITY` is closed after landing the first two bounded
  semantic module-identity classes: opt-in pure-combinational leaf
  modules and bounded pure-combinational wrappers with recursively
  proven children can merge beyond canonical structural signatures when
  `(PortId, width)` interfaces and small truth-table proofs match.
  Unsupported stateful, memory/FSM, parameterized, aggregate, oversized,
  and unsafe hierarchy-cycle classes are explicitly recorded as deferred
  proof boundaries.
- `SIGNOFF-SURFACE-EXPANSION` is closed for the current richer CDC,
  AST/source extractor, simulator/tool parity, and resource-aware
  signoff sweep frontier.
  `.1` landed the next CDC primitive: configurable N-flop 1-bit
  synchronizer chains via `cdc_synchronizer_stages`, with default
  2-stage behavior preserved and general CDC fabrics still deferred.
  `.2` landed the optional Verilator JSON-AST frontend parity gate,
  which checks all 7 Phase-8 manifest categories when local Verilator
  supports `--json-only`; `slang` was absent locally and is not
  required. `.3` landed the optional Icarus Verilog compile/elaboration
  matrix column and the static structured-gate lowering needed for
  warning-clean Icarus acceptance. `.4` records that there is no
  remaining frontier inside this tree: broader CDC fabrics, proprietary
  or absent tools, larger RAM-sensitive full-suite sweeps, and new
  artifact-family stress surfaces require future task-tree leaves.

The previously separate quality/capability follow-ups are closed:
`DIFFERENTIAL-SIMULATION` landed its cross-simulator semantic-agreement
lane on `2026-05-24`, and `MULTI-CLOCK-CDC` landed the opt-in
multi-clock/2-flop-synchronizer lane on `2026-05-24`.

## Post-phase capability lanes (owner-directed, 2026-06-15)

After the `AGENT-INTROSPECTION-MCP` tree closed (`2026-06-15`), the
project reached full task-tree exhaustion: every numbered phase and
every post-phase tree was `done`. The owner then authorized three new
post-phase capability lanes ("do these in any order"), now registered as
task trees via `CAPABILITY-LANE-OWNERSHIP.1` and tracked in
`docs/TASK_TREE.md`. None reopens a closed phase; each is open-ended
capability-deepening that lands as task-tree leaves. Agent-chosen
execution order is `2 → 3 → 1`:

- `AGENT-MCP-EXPANSION` (`active`) — broaden the read-mostly agent/MCP
  interface (coverage-gap MCP tool, non-DUT lanes over MCP, optional
  HTTP transport); every lane invariant from decision `0004` preserved,
  default `--artifact dut` byte-identical.
- `SIGNOFF-AUTOMATION-EXPANSION` (`proposed`) — broaden downstream
  signoff automation (richer adversarial knob sweeps, additional
  simulator/frontend acceptance columns, new valid-by-construction
  artifact families), warning-as-failure preserved.
- `IDENTITY-DEEPENING` (`active`) — advance steering gap 2
  (NodeId-as-identity): bounded hierarchical/module semantic identity
  beyond canonical structural signatures and broader bounded sequential
  equivalence, proof discipline and budgets unchanged. `.1` chose the
  first extension (decision `0007`); `.2` (`= .2a` design + `.2b` impl)
  **delivered** the opt-in bounded **bisimulation flop merge**
  (`merge_bisimilar_flops`, default-off / byte-identical, banked
  downstream-clean across Verilator + both Yosys + Icarus). `.3`
  (whole-module sequential equivalence) is the named future leaf.

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
