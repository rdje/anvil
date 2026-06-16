# STRUCTURED-EMISSION-EXPANSION: richer structured SystemVerilog surfaces

## Metadata

- Tree ID: `STRUCTURED-EMISSION-EXPANSION`
- Status: `active`
- Roadmap lane: `Capability / breadth — richer structured emission (ROADMAP steering gap 1)`
- Created: `2026-06-15`
- Last updated: `2026-06-17` (**`.8b` landed — the FOURTH structured surface
  (the wider-lane `generate for` part-select) is delivered end-to-end; `.8b` /
  `.8` close; the lane returns to no-active-frontier (open-ended).** First source
  change of the fourth surface — two surgical edits: `src/ir/generate_loop.rs`
  `gate_qualifies` relaxed to `LW >= 1` (`width == N*LW`) + `src/emit/sv.rs`
  `render_generate_loop_block` branches `LW==1` (verbatim `[gi]`, byte-identical)
  vs `LW>1` (`[gi*LW +: LW]`), with `generate_loop_gate`'s defensive re-check
  mirrored. 4 lib proofs; book §"The fourth surface" (byte-verified seed-74
  before/after) + knobs/USER_GUIDE/README/CODEBASE_ANALYSIS/KM closeout. Reuses
  `generate_loop_emit_prob` + `num_emitted_generate_loops` — **no new knob / no
  new metric / no introspection schema bump**. `cargo test --lib` 493 +
  snapshots 6/6 byte-identical (default-off); a per-seed ON-vs-OFF downstream
  sweep (8 seeds, `/tmp/anvil-gl8b/`) emits 9 wider-lane part-selects with
  Verilator `-Wall` Δ=0 + Yosys both + Icarus rc=0, and the `--generate-loop-gate`
  bank stays regression-clean (12/0). Prior: **`.8a` landed — the wider-lane
  `generate for` part-select impl design-detail; `.8` frontier → `.8b`.**
  Design-detail leaf,
  no source change (a `DEVELOPMENT_NOTES.md` entry). Grounded decision `0015` in
  the real `src/ir/generate_loop.rs` `gate_qualifies` + `src/emit/sv.rs`
  `generate_loop_gate`/`render_generate_loop_block` and resolved every open
  question: keep `generate_loop_gate` returning `(lane, N)` (recompute
  `LW = m.nodes[lane].width()` in the renderer — it already has `m`); branch the
  body `LW==1` (verbatim `[gi]`, byte-identical) vs `LW>1`
  (`[gi*LW +: LW]`); relax the predicate to `LW >= 1` / `width == N*LW`
  (`function_emit`/`soft_union` exclusions unchanged); the shipped 1-bit surface
  + its proofs + the `.4b` gate stay green; reuses `generate_loop_emit_prob` +
  `num_emitted_generate_loops` (no new knob / no schema bump). **Corpus-liveness
  proven**: a 300-module comb-only sweep (`/tmp/anvil-widelane-probe/`) emits 20
  multi-bit-lane replications (of 448) — the surface fires on real generation
  (~4.5%), not hand-built-only. `.8b` impl shape recorded. Prior: **`.7` landed
  — picked the FOURTH structured surface (the wider-lane `generate for`
  part-select); decision `0015`; design/decision leaf, no source change;
  frontier → `.8`.** At a
  no-active-frontier boundary, autonomously selected per
  `feedback_pick_and_roll_at_no_frontier` the recorded wider-lane follow-up to
  the second surface. A fresh empirical probe this session (Verilator 5.046
  `-Wall` + Yosys 0.64 both modes + Icarus 13.0) accepts a wider-lane
  `generate for` part-select warning-clean and iverilog simulation proves it
  bit-equal to `{N{x}}`; the same probe **disqualifies** `interface`/`modport`
  (Icarus syntax-fails the modport port, both Yosys modes warn on the implicit
  `.data` decl — confirming the recorded weak-support claim) and records
  nested-generate as clean-but-bigger-blast-radius. The fourth surface broadens
  the `generate for` lane from 1-bit to `LW >= 1` via
  `assign <wire>[gi*LW +: LW] = <x>;`, reusing the existing
  `generate_loop_emit_prob` knob + `num_emitted_generate_loops` metric (no new
  knob / no schema bump). Split `.7` (design, done) + `.8` (impl, pending;
  pre-split `.8a` design-detail + `.8b`). Prior: **`.6b.3` landed — the
  user-facing closeout; the THIRD structured surface (the combinational
  `task automatic`) is delivered end-to-end; `.6b.3` / `.6b` / `.6` all close;
  the lane returns to no-active-frontier (open-ended).** Docs-only / DUT
  byte-identical: a `## The
  third surface: a combinational task automatic` section in
  `book/src/structured-emission.md` (byte-verified seed-1 before/after — the
  inline shift becomes the `task automatic` decl + `logic <wire>__tv` + the
  `always_comb` call + the passthrough `assign`; the function-surface candidate
  parallel; the output-var passthrough form; the four-way mutual exclusion; the
  metric @ schema `1.10` + the gate) + the `task_emit_prob` knob entry in
  `book/src/knobs.md` / `USER_GUIDE.md` / README + the KM how-to card
  `combinational-task-emit` (41 facts / 331 keys). `mdbook build` +
  `check_knowledge_map` + `check_memory_architecture` + `cargo test --test
  book_examples` 3/3 green. Prior: **`.6b.2b` landed — the repo-owned
  `tool_matrix --task-emit-gate`; `.6b.2` closes; frontier → `.6b.3`.**
  `src/bin/tool_matrix.rs` gains `--task-emit-gate` + `ScenarioSet::TaskEmitSweep`
  + `build_task_emit_sweep_scenarios`/`task_emit_focus_config` (comb-only
  `task_emit_prob=1.0` × 3 strategies) + `ModuleReport.emitted_combinational_task`
  (`"task automatic"` SV-text detection) + `saw_combinational_task_emit` +
  `MatrixReport.task_emit_gate` + early-return gap arm + 5 proofs + 6 fixture
  updates + the `test_cli` default; README + USER_GUIDE + CODEBASE_ANALYSIS gate
  entries. Banked clean `/tmp/anvil-task-emit-gate-r1` (3 scenarios / 12 modules /
  12 emitting a task / `coverage_gaps=[]` / `12/0` Verilator + both Yosys +
  Icarus). No schema bump (harness-only); snapshots 6/6 byte-identical; `cargo
  test --bin tool_matrix` 68. Prior: **`.6b.2a` landed — the
  `num_emitted_combinational_tasks` metric + introspection schema `1.9 → 1.10`;
  `.6b.2` split into `.6b.2a` (done) + `.6b.2b` (the `tool_matrix` gate); frontier
  → `.6b.2b`.** `Metrics::num_emitted_combinational_tasks` (`= m.task_emit_gates.len()`,
  `#[serde(default)]`) surfaced in introspection `module_metrics` ⇒ `SCHEMA_VERSION`
  `1.9 → 1.10` (the metric bumps; the `.6b.1` knob rode the version). MINOR is an
  integer, so `1.9 → 1.10` (ten), not a decimal — recorded in the doc comment +
  schema changelog. Bumped all current-output schema refs (9 assertions + schema
  doc + README + USER_GUIDE + 5 book example JSONs + the CODEBASE_ANALYSIS envelope
  line); historical landing attributions left intact. Lib proof; default-off / DUT
  byte-identical (snapshots 6/6, lib 490); end-to-end introspect default `0` /
  forced `39`. Prior: **`.6b.1` landed — the combinational `task automatic`
  live surface; `.6b` split into `.6b.1` (done) + `.6b.2` (metric + gate) + `.6b.3`
  (docs closeout); frontier → `.6b.2`.** First source change since `.4b.1`. Live
  emitter change: `Config::task_emit_prob` (default `0.0`, config-file-only) +
  `Module.task_emit_gates` + new `src/ir/task_emit.rs`
  (`annotate_task_emit_gates`, the function-emit candidate predicate **plus**
  exclusion of the three sibling projections) + two guarded gen-time call-site
  rolls (after generate_loop) + the `to_sv_with_modules` `task_emit_gate`
  accessor + `render_gate_task_decl` (body via the reused
  `render_gate_function_body`) + `render_gate_task_call` (the `logic <wire>__tv`
  var + the `always_comb <wire>__t(<wire>__tv, …)` call) + the gate-assign-loop
  passthrough `assign <wire> = <wire>__tv;` + 11 lib proofs. Output-var +
  passthrough integration (the `.6a` first cut). No schema bump (the knob rides
  `#[serde(default)]`; the metric bumps `1.9→1.10` at `.6b.2`). Default-off / DUT
  byte-identical (snapshots 6/6; lib 489); forced `task_emit_prob=1.0` sweep clean
  across Verilator `--lint-only` (`-Wall` Δ=0 vs OFF) + both Yosys + Icarus
  (`/tmp/anvil-te-r1/`, 5 seeds, 4–39 tasks each). Prior: **`.6a` landed — the
  combinational `task automatic` impl design-detail; `.6` split into `.6a` (done)
  + `.6b` (impl pending); frontier → `.6b`.** Design-detail leaf, no source change
  (a `DEVELOPMENT_NOTES.md` entry + the tree split). Grounded decision `0014` in the real emitter (the
  `to_sv_with_modules` section template; the **reuse of `render_gate_function_body`**
  as the task body) and resolved all five points: the output-var + passthrough
  integration; gen-time `src/ir/task_emit.rs` + `Module.task_emit_gates`
  (function-emit predicate plus exclusion of the sibling projections);
  `task automatic` decl + `always_comb` call + assign-RHS swap;
  `Config::task_emit_prob` config-file-only default `0.0`;
  `num_emitted_combinational_tasks` metric (schema `1.9→1.10`) +
  `tool_matrix --task-emit-gate` / `saw_combinational_task_emit`. Prior: **`.5`
  landed — picked the THIRD structured surface
  (`task`); decision `0014`; frontier → `.6`.** Design/decision leaf, no source
  change. At a no-active-frontier boundary the owner directed *"pick any tree and
  roll"*; I autonomously selected `task` (the recorded leading candidate from
  decision `0013`). The third surface is a default-off, valid-by-construction
  combinational `task automatic` emit-projection of a single combinational gate
  (the decision `0012` single-gate parallel, but a procedural `task` with an
  `output` arg called from `always_comb`). Empirically grounded clean across
  Verilator `-Wall` + both Yosys + Icarus (both the direct-output and the
  output-var passthrough forms). Discipline / opt-in `task_emit_prob` /
  `saw_combinational_task_emit` gate; split `.5`/`.6`/`.7+`. Prior: **`.4b.3`
  landed — the user-facing closeout; the
  SECOND structured surface (the `generate for` loop) is delivered end-to-end;
  `.4b.3` / `.4b` / `.4` all close; the lane returns to no-active-frontier
  (open-ended).** Docs-only / DUT byte-identical: a `## The second surface: a
  generate for loop` section in `book/src/structured-emission.md` (byte-verified
  seed-12 before/after; the `{N{x}}` 1-bit-lane rule; the wider-lane exclusion;
  the `function_emit` mutual exclusion; the `gi = gi + 1` form; metric + gate) +
  the `generate_loop_emit_prob` knob entry in `book/src/knobs.md` /
  `USER_GUIDE.md` / README + the KM how-to card `generate-loop-emit` (39 facts /
  309 keys). `mdbook build` + `check_knowledge_map` + `check_memory_architecture`
  + `cargo test --test book_examples` 3/3 green. Prior: **`.4b.2b` landed — the
  repo-owned `tool_matrix --generate-loop-gate`; `.4b.2` closes; frontier →
  `.4b.3` (the user-facing closeout).** `src/bin/tool_matrix.rs` gains
  `--generate-loop-gate` +
  `ScenarioSet::GenerateLoopSweep` + `build_generate_loop_sweep_scenarios` +
  `ModuleReport.emitted_generate_loop` + `saw_generate_loop_emit` +
  `MatrixReport.generate_loop_gate` + 5 proofs + 6 fixture updates; README +
  USER_GUIDE + CODEBASE_ANALYSIS gate entries. Banked clean
  `/tmp/anvil-generate-loop-gate-r1` (3 scenarios / 12 modules / 8 emitting a
  loop / `coverage_gaps=[]` / `12/0` Verilator + both Yosys + Icarus). No schema
  bump (harness-only); snapshots 6/6 byte-identical; `cargo test --bin
  tool_matrix` 63. Prior: **`.4b.2a` landed — the `num_emitted_generate_loops`
  metric + introspection schema `1.8 → 1.9`; `.4b.2` split into `.4b.2a` (done) +
  `.4b.2b` (the `tool_matrix` gate, frontier).** `Metrics::num_emitted_generate_loops`
  (`= m.generate_loop_gates.len()`) surfaced in introspection `module_metrics` ⇒
  `SCHEMA_VERSION` `1.8→1.9` (the metric bumps; the `.4b.1` knob rode the version).
  Bumped all current-output schema refs (9 test assertions + schema doc + README +
  USER_GUIDE + 5 book example JSONs + the CODEBASE_ANALYSIS envelope line);
  historical landing attributions left intact. Lib proof; default-off / DUT
  byte-identical (snapshots 6/6, lib 478); end-to-end introspect default `0` /
  forced `50`. Prior: **`.4b.1` landed — the `generate for` loop live
  surface; `.4b` split into `.4b.1` (done) + `.4b.2` (gate + metric) + `.4b.3`
  (docs closeout); frontier → `.4b.2`.** Live emitter change:
  `Config::generate_loop_emit_prob` (default `0.0`, config-file-only) +
  `Module.generate_loop_gates` + new `src/ir/generate_loop.rs`
  (`annotate_generate_loop_gates`, candidate = a `{N{x}}` **1-bit-lane**
  replication `Concat` excluding function-emit marks) + two guarded gen-time
  call-site rolls (after function_emit) + the `to_sv_with_modules`
  `generate_loop_gate` accessor + `render_generate_loop_block` + the
  generate-block section + the assign-loop inline-replication suppression + 9 lib
  proofs. Increment form `gi = gi + 1` (portable; `gi++` not retired). No schema
  bump (default-off prob-knob precedent; the `.4b.2` metric bumps `1.8→1.9`).
  Default-off / DUT byte-identical (snapshots 6/6; lib 477); forced
  `generate_loop_emit_prob=1.0` sweep clean across Verilator `--lint-only`
  (`-Wall` Δ=0 vs OFF) + both Yosys + Icarus (`/tmp/anvil-gl-r1/`, 5 seeds,
  62–168 loops each). Prior: **`.4a` landed — the `generate for` loop impl
  design-detail; `.4` split into `.4a` (done) + `.4b` (impl pending); frontier →
  `.4b`.** Design-detail leaf, no source change (a `DEVELOPMENT_NOTES.md` entry +
  the tree split). Grounded decision `0013` in the real emitter — `render_gate`'s
  existing `{N{x}}` replication predicate (`Concat`, all-same-NodeId, `sv.rs:1159`)
  is the index-regular source; the `function_emit.rs`/`soft_union.rs` gen-time
  `annotate_*` + `Module` `BTreeSet<NodeId>` marker is the mechanism — and resolved
  all five `.4a` points: first-cut selection = a `{N{x}}` **1-bit-lane**
  replication `Concat` (excluding function-emit marks, run after function-emit);
  gen-time `src/ir/generate_loop.rs annotate_generate_loop_gates` +
  `Module.generate_loop_gates`; a `genvar <wire>__gi` / `generate for` block +
  assign-loop `continue` suppression; `Config::generate_loop_emit_prob`
  config-file-only default `0.0` byte-identical (a `num_emitted_generate_loops`
  metric in `.4b` bumps schema `1.8→1.9`); `tool_matrix --generate-loop-gate` /
  `saw_generate_loop_emit` (full Verilator + both Yosys plan). Flagged the
  gate-shape replication-availability risk for `.4b`. Self-checks clean. Prior:
  **`.3` landed — picked the SECOND structured
  surface, a `generate for` loop emit-projection; decision `0013`.** By owner
  steer (*"structured emission: next surface"* → `generate`): a default-off,
  valid-by-construction `generate for` loop projecting an existing `{N{x}}`
  replication (index-regular by construction), over `task` [leading future, also
  clean for simple comb void tasks], `interface`/`modport` [weak Yosys synth], and
  constant-predicate `generate if` [dead untaken branch]. Empirically grounded:
  Verilator `-Wall` + both Yosys + Icarus accept `generate for` clean; DUT emitter
  has no generate today; frontend lane has `generate if`. Split `.3` (done) + `.4`
  (impl; pre-split `.4a`/`.4b`) + future `.5+`. Frontier → `.4`. Design/decision
  leaf, no source change; self-checks clean. Prior: **`.2b.2c` landed — the
  user-facing closeout; the first structured surface is delivered end-to-end and
  `.2`/`.2b`/`.2b.2` all close**. New `How It Works` book chapter `book/src/structured-emission.md`
  (byte-verified seed-42 before/after; single-gate rule; `Slice`/structured
  exclusions; duplicate-operand positional params; combinational-only) + the
  `function_emit_prob` knob entry in `book/src/knobs.md` / `USER_GUIDE.md` /
  README "Current CLI truth" (documented accurately as a config-file-only knob)
  + the Knowledge Map how-to card `combinational-function-emit` (KM 36→37 facts /
  272→286 keys). Docs-only / DUT byte-identical. `mdbook build` + `check_knowledge_map`
  + `check_memory_architecture` + `cargo test --test book_examples` 3/3 all green.
  The tree stays `active` as an open-ended lane with **no current frontier**;
  future surfaces (`task`/nested `generate`/`interface`/`modport`) are `.3+`,
  each its own decision when picked. Nothing retired. Prior: **`.2b.2b` landed** —
  the repo-owned `tool_matrix --function-emit-gate`: `ScenarioSet::FunctionEmitSweep` +
  `build_function_emit_sweep_scenarios` (comb-only `function_emit_prob=1.0` × 3
  strategies) + `ModuleReport.emitted_combinational_function` SV-text detection +
  `saw_combinational_function_emit` coverage fact + early-return gap enforcement +
  5 cargo-portable proofs; banked clean `/tmp/anvil-function-emit-gate-r1` (3
  scenarios / 12 modules / 608 emitted functions / `coverage_gaps=[]` / `12/0`
  Verilator + both Yosys + Icarus); default-off / DUT byte-identical, snapshots
  6/6; **frontier → `.2b.2c`** the user-facing closeout. Prior: **`.2b.2` pre-split
  + `.2b.2a` landed** — `.2b.2` split into `.2b.2a` (metric + schema), `.2b.2b` (the `tool_matrix` gate, **frontier**), `.2b.2c` (book/USER_GUIDE/KM/README closeout). `.2b.2a` added `Metrics::num_emitted_combinational_functions` (`= function_emit_gates.len()`) ⇒ introspection schema MINOR bump `1.7 → 1.8` (the metric bumps; the `.2b.1` knob rode the version); 468 lib tests / snapshots 6/6 / mdbook all green; default-off / DUT byte-identical. Prior: **`.2b.1` live surface** — the first richer-structured emit surface goes live: `Config::function_emit_prob` + `Module.function_emit_gates` + new `src/ir/function_emit.rs` `annotate_function_emit_gates` (gen-time mark, the `soft_union.rs` precedent) + two generator call-site rolls (after soft_union) + `to_sv_with_modules` `<wire>__f` `function automatic` decl/positional-body/call rendering + 9 lib proofs. `Slice` excluded from the first cut (`-Wall UNUSEDSIGNAL` on a full-width param; still emitted inline, nothing retired; slice-aware projection = follow-up). No schema bump (default-off prob-knob precedent). Default-off / DUT byte-identical (snapshots 6/6); forced `function_emit_prob=1.0` sweep clean across Verilator `--lint-only` + both Yosys modes + Icarus (`/tmp/anvil-fe-r2/`). Frontier → `.2b.2` (the repo-owned gate + metric + coverage fact + book/USER_GUIDE/KM closeout). Prior: `.2a` design-detail; `.1` design — decision `0012`.)
- Owner: repo-local workflow
- Note: registered `proposed` by owner roadmap steering (`2026-06-15`) as a named
  sibling of `SV-VERSION-TARGETING`; **activated `2026-06-16`** by explicit owner
  directive selecting this lane next.

## Goal

Broaden ANVIL's emitted SystemVerilog surface beyond today's flat
module/`always`/instance shape into richer **structured** constructs —
synthesizable, valid-by-construction — to give downstream tools more legal
structural variety to ingest: e.g. `function` / `task` bodies, `interface` /
`modport` boundaries, and nested / multi-level `generate` constructs. Each is a
new legal interaction surface (ROADMAP steering gap 1), not whole-module
behaviour.

## Non-Goals

- No generate-then-filter; every structured construct is valid-by-construction
  (`feedback_rules_first_generation`).
- No default output change until a construct is proven downstream-clean and
  opt-in (`feedback_never_retire_strategies`).
- Not whole-module specification / functional correctness (structure-first per
  ROADMAP steering gap 4).

## Acceptance Criteria

- Each landed structured surface is rules-first, opt-in / default byte-identical
  where it could change output, and proven downstream-clean (Verilator + both
  Yosys modes, and Icarus where applicable).
- Live docs + book + a Knowledge Map fact per durable surface.
- Every leaf committed through `COMMIT.md` with its leaf id.

## Task Tree

- ID: `STRUCTURED-EMISSION-EXPANSION`
  Status: `active`
  Goal: `Richer structured synthesizable SV surfaces (functions / generate / tasks / interfaces), valid-by-construction. FOUR surfaces delivered end-to-end: combinational function automatic (.1+.2), generate for loop (.3+.4), combinational task automatic (.5+.6), and the wider-lane generate for part-select (.7 design + .8 impl, decision 0015). Open-ended lane with no current frontier: nested/multi-level generate / interface-modport / richer tasks are future (.9+), each its own decision.`
  Children: `STRUCTURED-EMISSION-EXPANSION.1`, `STRUCTURED-EMISSION-EXPANSION.2`, `STRUCTURED-EMISSION-EXPANSION.3`, `STRUCTURED-EMISSION-EXPANSION.4`, `STRUCTURED-EMISSION-EXPANSION.5`, `STRUCTURED-EMISSION-EXPANSION.6`, `STRUCTURED-EMISSION-EXPANSION.7`, `STRUCTURED-EMISSION-EXPANSION.8`

- ID: `STRUCTURED-EMISSION-EXPANSION.1`
  Status: `done`
  Goal: `Design/decision leaf: inventory candidate structured surfaces (function/task, interface/modport, nested generate), pick the first concrete synthesizable + downstream-clean one, define its valid-by-construction discipline + opt-in knob + downstream gate, and split the tree — before any code.`
  Acceptance: `A decision record naming the first surface, its construction discipline, and its downstream gate; no source change; self-checks clean.`
  Result: `Decision 0012. The first richer-structured surface is a default-off, opt-in, valid-by-construction combinational function automatic emitted as a behaviour-preserving projection of an existing combinational cone: a selected Gate node + its fan-in (stopping at the output_support support-leaf boundary — primary inputs / flop Qs / instance outputs / constants) rendered as function automatic logic[W-1:0] <name>(...) whose parameter list is the cone's support leaves and whose body is the straight-line evaluation of the cone's internal gates, returning the root; the use site becomes a call. Chosen over interface/modport (weak/version-inconsistent Yosys synth support ⇒ fails the both-Yosys-modes-clean bar) and nested generate (bigger emitter blast radius) and task (procedural/multi-output — a combinational function is the simpler first cut). Discipline: rules-first (wraps an already-valid cone; selection at construction time, never generate-then-filter); default-off function_emit_prob (default 0.0) ⇒ byte-identical, snapshots untouched; no new IR node / no new computed truth (the soft_union/aggregate emit-projection precedent). Downstream gate: a repo-owned gate proving Verilator + both Yosys modes + Icarus accept the emitted functions warning-clean, gated on a saw_combinational_function_emit coverage fact. Rejected: interface/modport first, nested generate first, task first, a semantic IR Function node, generate-then-filter, changing the default. Split into .1 (done) + .2 (impl) + future kinds (.3+: task, nested generate, interface/modport). Pre-split .2 → .2a (design-detail) + .2b (impl).`
  Verification: `done`
  Commit: `done`

- ID: `STRUCTURED-EMISSION-EXPANSION.2`
  Status: `done`
  Goal: `Implement the first structured surface (the combinational function automatic emit-projection) per decision 0012: the function_emit_prob knob + the rules-first cone selection + the emitter rendering (function automatic decl + call site) + the downstream-clean gate + book/USER_GUIDE/KM. Default-off / DUT byte-identical.`
  Children: `STRUCTURED-EMISSION-EXPANSION.2a`, `STRUCTURED-EMISSION-EXPANSION.2b`
  Result: `Done (closed by .2b.2c, 2026-06-16). The combinational function automatic emit-projection is delivered end-to-end: the function_emit_prob knob + Module.function_emit_gates + the gen-time annotate_function_emit_gates selection + the to_sv_with_modules <wire>__f decl/positional-body/call rendering (.2b.1), the num_emitted_combinational_functions metric + introspection schema 1.8 (.2b.2a), the repo-owned tool_matrix --function-emit-gate downstream-clean gate (.2b.2b, banked /tmp/anvil-function-emit-gate-r1), and the book/USER_GUIDE/README/KM user-facing closeout (.2b.2c). Default-off / DUT byte-identical throughout (snapshots 6/6). Nothing retired.`

- ID: `STRUCTURED-EMISSION-EXPANSION.2a`
  Status: `done`
  Goal: `Design-detail leaf (no source): ground the combinational function automatic surface in the real src/emit/sv.rs to_sv_with_modules + the soft_union.rs / aggregate_layout emit-projection precedents + src/config.rs. Pin: (1) the cone-selection rule (which Gate nodes qualify; size/depth bounds so the function is non-trivial yet bounded; how it stays rules-first); (2) whether selection is a generation-time annotation (the soft_union.rs / aggregate_layout precedent — likely, so the IR carries the choice deterministically and emission projects it) or a pure emit-time pass; (3) the function signature + body rendering (parameter list = the cone's support leaves; local decls vs single return expr; width/logic typing); (4) the function_emit_prob knob semantics + default 0.0 byte-identical contract; (5) the downstream-gate scenario shape (saw_combinational_function_emit). DEVELOPMENT_NOTES design-detail entry + the .2b impl shape.`
  Acceptance: `A DEVELOPMENT_NOTES design-detail entry resolving the five points grounded in real code; tree split recorded; no source change; docs/workflow self-checks clean.`
  Result: `Done. DEVELOPMENT_NOTES design-detail entry resolves all five points, grounded in a fresh read of src/emit/sv.rs (to_sv_with_modules gate-emission loop + build_names/node_ref/render_gate/param_width_decl_w), src/ir/soft_union.rs + Module.soft_union_slice_gates (the gen-time-annotation precedent), and the aggregate_layout projection. (1) First-cut cone selection = the MINIMAL cone: wrap ONE selected Node::Gate as a function automatic of its DIRECT operands (operands are already module wires/literals ⇒ zero sharing/scoping hazard; the multi-level-cone body with private-internal locals is a recorded follow-up). Candidate = a non-structured (not CaseMux/CasezMux/ForFold), non-soft_union-marked Gate with >= 1 operand; selection rules-first at gen time. (2) Gen-time annotation (the soft_union.rs precedent): a new src/ir/function_emit.rs annotate_function_emit_gates(m, rng, prob) rolls gen_bool(prob) per candidate into a new Module.function_emit_gates: BTreeSet<NodeId> (emitter-surface annotation only — flat IR/validators/CSE/canonical_signature untouched); call-site guard on prob > 0.0 ⇒ default byte-identical. (3) Signature = function automatic logic[W-1:0] <wire>__f(positional input logic[Wi-1:0] ai,...); body = op over the positional param names (a render_gate-parallel positional variant — positional, not node-id-mapped, to handle duplicate operands); call site = assign <wire> = <wire>__f(node_ref(o0),...); behaviour-preserving by construction. (4) Config::function_emit_prob (default 0.0) beside aggregate_prob/soft_union_slice_prob ⇒ default byte-identical, snapshots untouched; surfaced in dump-config/introspect (a Config-field schema MINOR bump, confirmed in .2b). (5) Downstream gate = Verilator + both Yosys modes + Icarus warning-clean on a saw_combinational_function_emit fact (+ a num_emitted_combinational_functions metric), shape in .2b.2. Pre-split .2b → .2b.1 (the live surface: knob + annotation + Module field + emitter rendering + lib proofs + Verilator lint) + .2b.2 (the repo-owned gate + metric + coverage fact + book/USER_GUIDE/KM). Rejected: multi-level cone body in the first cut, a pure emit-time pass, node-id operand→param mapping.`
  Verification: `done`
  Commit: `done`

- ID: `STRUCTURED-EMISSION-EXPANSION.2b`
  Status: `done`
  Goal: `Implement the .2a design: the function_emit_prob knob, the rules-first single-gate selection (gen-time annotation src/ir/function_emit.rs + Module.function_emit_gates), the function automatic emitter rendering (decl + positional-param body + call site) in to_sv_with_modules, lib proofs (behaviour-preserving + selected-by-construction + default-off byte-identical + CSE/canonical-signature untouched), the downstream-clean gate (Verilator + both Yosys modes + Icarus + the saw_combinational_function_emit fact + a num_emitted_combinational_functions metric), and book/USER_GUIDE/KM closeout. Default-off / DUT byte-identical (snapshots untouched).`
  Children: `STRUCTURED-EMISSION-EXPANSION.2b.1`, `STRUCTURED-EMISSION-EXPANSION.2b.2`
  Result: `Done (closed by .2b.2c, 2026-06-16). All of .2b.1 (live surface), .2b.2a (metric + schema 1.8), .2b.2b (the tool_matrix gate), and .2b.2c (docs closeout) complete. Default-off / DUT byte-identical.`

- ID: `STRUCTURED-EMISSION-EXPANSION.2b.1`
  Status: `done`
  Goal: `The live first-cut surface: Config::function_emit_prob (default 0.0, serde default) + Module.function_emit_gates: BTreeSet<NodeId> + src/ir/function_emit.rs annotate_function_emit_gates(m, rng, prob) (collect non-structured/non-soft_union Gate candidates, roll gen_bool(prob), mark) + the generator call-site roll (guarded prob > 0.0) + the to_sv_with_modules rendering (a function automatic decl section + positional-param body via a render_gate positional variant + the call-site assign) + lib proofs (a marked gate emits a behaviour-preserving function + call; default-off byte-identical; the mark leaves CSE/canonical_module_signature untouched) + a forced-knob Verilator --lint-only spot-check. Default-off / DUT byte-identical (snapshots 6/6).`
  Acceptance: `cargo check/clippy(-D warnings)/fmt clean; cargo test --lib green incl. the new function_emit proofs; cargo test --test snapshots 6/6 byte-identical (default-off); a forced function_emit_prob=1.0 sample lints clean under Verilator. Committed through COMMIT.md with the leaf id.`
  Result: `Done. Config::function_emit_prob (default 0.0, default_function_emit_prob() serde default; added to the Default impl + the 0.0..=1.0 validation list) + Module.function_emit_gates: BTreeSet<NodeId> (Default-empty; emitter-surface annotation only — flat IR / validators / CSE / canonical_module_signature untouched, disjoint from soft_union_slice_gates) + new src/ir/function_emit.rs annotate_function_emit_gates(m, rng, prob) (gen-time mark; the soft_union.rs precedent; rolls gen_bool(prob) per qualifying candidate; skips param_env modules) + call-site rolls in BOTH generate_module and generate_design guarded on prob > 0.0, run AFTER the soft_union pass (so union soft marks are excluded) + src/emit/sv.rs rendering: a function automatic decl section (after the wire decls, before the gate assigns) emitting per marked gate function automatic logic[W-1:0] <wire>__f(input logic[Wi-1:0] a0,...); <wire>__f = <op over a0..a{n-1}>; endfunction, and a call-site substitution making the marked gate's assign become assign <wire> = <wire>__f(<operand refs>). Helpers: function_emit_gate (marked + defensively-revalidated lookup), render_gate_function_body (positional behaviour-preserving counterpart of render_gate), render_gate_function_decl, render_gate_function_call. FIRST-CUT SCOPING REFINEMENT: Slice EXCLUDED from candidacy — a forced function_emit_prob=1.0 verilator -Wall sweep flagged UNUSEDSIGNAL on every slice_*__f param (a bit-select reads only a sub-range of its operand, so a full-width param leaves bits unused); Slice still emits inline (NOTHING RETIRED), a slice-aware projection that passes only src[hi:lo] is a recorded follow-up. All other ops use operands in full and are warning-clean. NO schema bump (default-off prob-knob precedent: soft_union/aggregate/memory/fsm/multi_clock all rode the existing schema_version via #[serde(default)]; only the sv_version enum took a dedicated 1.1->1.2 bump; introspect schema tests stay green at 1.7). 9 lib proofs (mark/skip/structured/slice/soft-union/param-env exclusions + identity-and-node-count-untouched + end-to-end emit + duplicate-operand positional params).`
  Verification: `cargo check --all-targets clean; cargo clippy --all-targets -- -D warnings clean; cargo fmt --all --check clean; cargo test --lib 467 passed / 2 ignored (incl. 9 new function_emit proofs; introspect schema_version 1.7 + umbrella DUT-byte-identical still green); cargo test --test snapshots 6/6 byte-identical (default-off). Forced function_emit_prob=1.0 sweep (5 seeds: 1/7/42/100/2024, 830-1299 functions each, banked /tmp/anvil-fe-r2/): Verilator --lint-only 5/5 CLEAN (repo bar), 0 __f-param -Wall warnings (slice fix resolved every change-introduced warning; residual -Wall UNUSEDSIGNAL on ordinary gate wires is pre-existing — the function-emit-OFF baseline has 20), Yosys without-abc 5/5 + with-abc 5/5, Icarus iverilog -g2012 5/5 CLEAN.`
  Commit: `done`

- ID: `STRUCTURED-EMISSION-EXPANSION.2b.2`
  Status: `done`
  Goal: `The repo-owned downstream gate + closeout for the combinational function automatic surface: a num_emitted_combinational_functions metric + a saw_combinational_function_emit coverage fact + a tool_matrix gate proving Verilator + both Yosys modes + Icarus accept the emitted functions warning-clean + book/USER_GUIDE/KM/README closeout. Default-off / DUT byte-identical. Pre-split (2026-06-16) into .2b.2a (metric + schema bump) + .2b.2b (the tool_matrix gate + coverage fact) + .2b.2c (docs closeout) — the metric is a Metrics field surfaced in introspection (schema MINOR bump, like 1.0->1.1 bisimulation_flops_merged); the tool_matrix gate is a large, fragile change (flag + ScenarioSet + config builder + coverage fact + detection + merge + gap enforcement + many ModuleReport/Cli test fixtures) that warrants its own focused slice; the book chapter + USER_GUIDE + KM + README CLI-truth entry are the user-facing closeout.`
  Children: `STRUCTURED-EMISSION-EXPANSION.2b.2a`, `STRUCTURED-EMISSION-EXPANSION.2b.2b`, `STRUCTURED-EMISSION-EXPANSION.2b.2c`
  Result: `Done (closed by .2b.2c, 2026-06-16). .2b.2a (metric + introspection schema 1.8), .2b.2b (the repo-owned tool_matrix --function-emit-gate, banked clean), and .2b.2c (book/knobs/USER_GUIDE/README/KM closeout) all complete. Default-off / DUT byte-identical.`

- ID: `STRUCTURED-EMISSION-EXPANSION.2b.2a`
  Status: `done`
  Goal: `The num_emitted_combinational_functions metric: add Metrics::num_emitted_combinational_functions (usize, #[serde(default)]) computed in metrics::compute() as m.function_emit_gates.len(); it surfaces in introspection module_metrics (the SCHEMA-DERIVED projection), so bump the introspection schema MINOR 1.7 -> 1.8 (SCHEMA_VERSION const + the 9 "1.7" test assertions in src/introspect/mod.rs + src/mcp/mod.rs + the docs/AGENT_INTROSPECTION_SCHEMA.md changelog/§7 lines). A lib proof that a module with marked function_emit_gates reports the count. Default-off / DUT byte-identical (a post-hoc Metrics field changes no emitted RTL; snapshots untouched).`
  Acceptance: `cargo check/clippy(-D warnings)/fmt clean; cargo test --lib green incl. the metric proof + the schema_version 1.8 assertions; cargo test --test snapshots 6/6 byte-identical; the schema doc records the 1.7 -> 1.8 additive MINOR bump. Committed through COMMIT.md with the leaf id.`
  Result: `Done. Metrics::num_emitted_combinational_functions: usize (#[serde(default)]) added to src/metrics.rs, computed in metrics::compute() as m.function_emit_gates.len() (a post-hoc structural count of an emitter-surface annotation; reads 0 by default, the configured count when function_emit_prob fired). Surfaced in introspection module_metrics (Metrics is the exact serde projection), so SCHEMA_VERSION bumped 1.7 -> 1.8 in src/introspect/mod.rs. The metric BUMPS the schema (new derived Metrics field — the 1.0->1.1 bisimulation_flops_merged precedent) whereas the .2b.1 knob did NOT (default-off prob-knob rides request.knobs via #[serde(default)]). Bumped all current-output schema refs to 1.8: the 9 schema_version assertions (src/introspect/mod.rs + src/mcp/mod.rs), the schema doc (1.7->1.8 changelog entry + the defines/lockstep/checklist lines), README (--introspect + analyze), USER_GUIDE (--introspect), the 5 book agent-mcp.md example JSONs, and the CODEBASE_ANALYSIS envelope line (which had drifted, frozen at 1.4). Historical "landed at schema X" attributions left intact. Lib proof metrics_count_emitted_combinational_functions (unmarked 0, marked 1).`
  Verification: `cargo clippy --all-targets -- -D warnings clean; cargo fmt --all --check clean; cargo test --lib 468 passed / 2 ignored (the new metric proof + all schema_version assertions green at 1.8); cargo test --test snapshots 6/6 byte-identical (default-off; metric changes no RTL); end-to-end --introspect: default seed => schema_version 1.8 + num_emitted_combinational_functions 0; forced function_emit_prob=1.0 => 1.8 + 1256; mdbook build book OK.`
  Commit: `done`

- ID: `STRUCTURED-EMISSION-EXPANSION.2b.2b`
  Status: `done`
  Goal: `The repo-owned tool_matrix gate: a saw_combinational_function_emit coverage fact + a --function-emit-gate flag (or a ScenarioSet) forcing function_emit_prob=1.0 over comb-only DUTs across the three construction strategies + a ModuleReport.emitted_combinational_function detection (from emitted SV or num_emitted_combinational_functions) + coverage-gap enforcement, proving Verilator + both Yosys modes + Icarus accept the emitted functions warning-clean. Bank a clean report. Default-off / DUT byte-identical. Template: --signoff-knob-sweep-gate; precedent for emitted-construct detection: the soft_union emitted_soft_union_overlay / saw_sv_version_2023_soft_union_upopt path. (Large, fragile change — many ModuleReport/Cli test fixtures must gain the new field.) Forced-sweep evidence already banked at /tmp/anvil-fe-r2/ (5 seeds, 3 tools, both Yosys modes).`
  Acceptance: `cargo check/clippy(-D warnings)/fmt clean; the repo-owned gate is banked clean (Verilator + both Yosys + Icarus) with saw_combinational_function_emit lit and coverage_gaps=[]; snapshots 6/6 byte-identical; committed through COMMIT.md with the leaf id.`
  Result: `Done. src/bin/tool_matrix.rs gains the repo-owned --function-emit-gate, templated on --signoff-knob-sweep-gate (scaffolding) + the union soft up-opt (emitted-construct detection). New: --function-emit-gate CLI flag + ScenarioSet::FunctionEmitSweep + MatrixReport.function_emit_gate (wired into select_scenario_set [mutually exclusive], derive_run_plan [4 units/scenario floor + fail_on_coverage_gap], build_scenarios, scenario_set_slug, artifact_kind_slug). build_function_emit_sweep_scenarios + function_emit_focus_config: one comb-only single-module DUT (share_heavy_comb_only_config-shaped: node-id + e-graph, flop_prob = 0.0) with function_emit_prob = 1.0 across all three construction strategies (3 scenarios). ModuleReport.emitted_combinational_function (#[serde(default)]) set in materialize_prepared_module from prepared.sv_text.contains("function automatic") (mirrors emitted_soft_union_overlay). CoverageSummary.saw_combinational_function_emit lit in summarize_coverage when an emitted-function module is accepted by Verilator success AND a non-empty clean Yosys vec (a synthesizable function is universally accepted, so unlike the Verilator-only union soft up-opt the gate runs the full tool plan; Icarus, when --iverilog-compile is set, rides the ToolSummary::any_failed bail); merged in merge_coverage; enforced by an early-return arm in compute_coverage_gaps after the universal construction-strategy coverage (so no broad-motif richness leaks in). 5 cargo-portable proofs + the new field threaded through 6 ModuleReport fixtures. clippy::explicit_counter_loop fixed by switching the builder to .enumerate(). No schema bump (harness-only). Default function_emit_prob = 0.0 emission byte-identical (snapshots 6/6). Frontier -> .2b.2c.`
  Verification: `cargo check --bin tool_matrix clean; cargo clippy --all-targets -- -D warnings clean; cargo fmt --all --check clean; cargo test --bin tool_matrix 58 passed / 1 ignored (incl. 5 new function-emit gate proofs); cargo test --lib 468 passed / 2 ignored (unchanged — harness-only); cargo test --test snapshots 6/6 byte-identical. Repo-owned downstream bank /tmp/anvil-function-emit-gate-r1 (./target/release/tool_matrix --function-emit-gate --yosys-mode both --iverilog-compile): 3 scenarios / 12 modules / 608 emitted functions / coverage_gaps = [] / saw_combinational_function_emit = true / Verilator 12/0 / Yosys without-abc 12/0 / Yosys with-abc 12/0 / Icarus compile 12/0; all 12 modules emitted_combinational_function = true.`
  Commit: `done`

- ID: `STRUCTURED-EMISSION-EXPANSION.2b.2c`
  Status: `done`
  Goal: `The user-facing closeout: a book chapter (or section) on structured emission / the combinational function automatic surface (under "How It Works" or "Reference") with examples; the USER_GUIDE function_emit_prob knob entry; the README "Current CLI truth" knob entry; and a Knowledge Map card if a durable how-to is warranted (decision 0012 already carries answers:). Default-off / DUT byte-identical (docs-only).`
  Acceptance: `book builds (mdbook build book); USER_GUIDE + README updated; KM regenerated + check_knowledge_map clean; self-checks clean; committed through COMMIT.md with the leaf id.`
  Result: `Done. New "How It Works" book chapter book/src/structured-emission.md (added to SUMMARY.md after factorization.md): the concept (emit-time projection of an already-valid cone, the soft_union/aggregate precedent), a byte-verified seed-42 before/after example (function_emit_prob 0.0 -> 1.0 adds the add_0__f decl + rewrites only that gate's assign to a call; everything else byte-identical), the single-gate first-cut rule, the Slice/structured-selector exclusions (Slice = -Wall UNUSEDSIGNAL on a full-width param; nothing retired), duplicate-operand positional params (concat_0__f(case_mux_0, case_mux_0)), combinational-only (flop Q is a leaf), the why-this-surface-first rationale, and the metric + tool_matrix --function-emit-gate proof. A skip-sentinelled repro bash block (config-file edit; not the default one-liner). function_emit_prob knob entry added to the canonical knob reference book/src/knobs.md (new "### Structured emission" subsection after the SystemVerilog-version subsection), USER_GUIDE.md (after the soft_union_slice_prob config-knob section), and the README "Current CLI truth" (a dedicated config-file-knob bullet before the tool_matrix --function-emit-gate gate bullet) — all documenting it accurately as a config-file-only knob (no CLI flag, like soft_union_slice_prob/aggregate_prob; the .2b.2b gate README/USER_GUIDE entries already landed). New Knowledge Map how-to card docs/knowledge/combinational-function-emit.md (id combinational-function-emit) with how-to question keys distinct from decision 0012's conceptual keys + a validated reverify command (dump-config -> set function_emit_prob=1.0 + comb-only -> generate -> grep "function automatic" -> verilator --lint-only). KM regenerated (36 -> 37 facts, 272 -> 286 question keys). Docs-only / DUT byte-identical (no source touched). With this leaf, .2b.2 / .2b / .2 all close: the first structured surface (the combinational function automatic emit-projection) is delivered end-to-end. The tree stays active as an open-ended lane with no current frontier; future surfaces (task / nested generate / interface/modport) are .3+, each its own decision when picked. Nothing retired.`
  Verification: `mdbook build book clean (HTML written, no broken-link warnings); bash knowledge-map/scripts/gen_knowledge_map.sh (37 facts / 286 keys) + bash knowledge-map/scripts/check_knowledge_map.sh OK (facts valid, ids unique, map in sync); bash scripts/check_memory_architecture.sh all invariants hold (0012 indexed); cargo test --test book_examples 3/3 (skip_sentinels_have_reasons + every_runnable_book_bash_block_succeeds green — the new repro block correctly skipped). Docs-only: no src/ touched, so cargo check/clippy/fmt unaffected; the seed-42 before/after and the seed-11 reverify were byte-verified against the release binary (function_emit_prob 0.0 vs 1.0 diff = exactly the add_0__f decl + the one assign; reverify emits 10 functions, Verilator clean).`
  Commit: `done`

- ID: `STRUCTURED-EMISSION-EXPANSION.3`
  Status: `done`
  Goal: `Design/decision leaf for the SECOND structured surface (owner steer: "structured emission: next surface" -> generate): re-confirm the candidate ranking (task / nested generate / interface-modport) with current tool evidence, pick the next concrete synthesizable + downstream-clean surface, define its valid-by-construction discipline + opt-in knob + downstream gate, and split the tree — before any code.`
  Acceptance: `A decision record naming the second surface, its construction discipline, and its downstream gate; an empirical tool-acceptance grounding; no source change; self-checks clean (mdbook + check_knowledge_map + check_memory_architecture). Committed through COMMIT.md with the leaf id.`
  Result: `Decision 0013. The second richer-structured surface is a default-off, opt-in, valid-by-construction generate for loop emitted as a behaviour-preserving projection of an existing REPLICATED construction — leading source = a GateOp::Concat of the {N{x}} form (an N-fold replication ANVIL already builds, e.g. assign concat_1 = {11{or_0}};), which is index-regular by construction, rendered as genvar gi; generate for (gi=0; gi<N; gi++) begin : <label> assign <wire>[gi] = <x>; end endgenerate (the unrolled loop == the inline replication). First cut = single-level generate for (the minimal faithful loop, the single-gate-function parallel); nested/multi-level generate = follow-up. Grounding (empirical, this session): the DUT emitter has NO generate/genvar today (src/emit/sv.rs); the frontend lane already emits generate if (src/frontend/mod.rs); a representative generate-for lane unroll + a replication->generate-for projection are accepted warning-clean by Verilator 5.046 -Wall + both repo Yosys modes + Icarus iverilog -g2012. Chosen over task (ALSO clean for simple combinational void tasks on this toolchain — so 0012's "weak task synth" is, precisely, a multi-output/side-effecting caution; task is the leading FUTURE candidate, .5+, not retired), interface/modport (still weak/inconsistent Yosys synth), and generate-if-only (constant predicate => dead untaken branch, lower DUT value; frontend lane already has it). Discipline: rules-first (marks an already-valid replication node at construction time; never generate-then-filter); default-off generate_loop_emit_prob (proposed name, default 0.0) => byte-identical, snapshots untouched; no new IR node / no new whole-module behaviour (the soft_union/aggregate/function_emit emit-projection precedent). Downstream gate: a repo-owned tool_matrix gate (templated on --function-emit-gate) proving Verilator + both Yosys modes + Icarus accept the emitted loops warning-clean, gated on a saw_generate_loop_emit coverage fact. Rejected: task first, interface/modport first, generate-if first, nested/multi-level generate in the first cut, a semantic IR generate node, generate-then-filter, changing the default. Split into .3 (done) + .4 (impl) + future (.5+: task [leading], nested/multi-level generate, interface/modport). Pre-split .4 -> .4a (design-detail) + .4b (impl) when picked. KM card structured-emission-second-surface-generate-loop (decision carries answers:).`
  Verification: `done`
  Commit: `done`

- ID: `STRUCTURED-EMISSION-EXPANSION.4`
  Status: `done`
  Goal: `Implement the second structured surface (the generate for loop emit-projection) per decision 0013: the generate_loop_emit_prob knob + the rules-first replication-node selection + the emitter rendering (genvar + generate for + call-site suppression of the inline replication assign) + the downstream-clean gate (saw_generate_loop_emit) + book/USER_GUIDE/KM. Default-off / DUT byte-identical.`
  Children: `STRUCTURED-EMISSION-EXPANSION.4a`, `STRUCTURED-EMISSION-EXPANSION.4b`
  Result: `Done (closed by .4b.3, 2026-06-16). The second structured surface — the generate for loop emit-projection of a {N{x}} 1-bit-lane replication (decision 0013) — is delivered end-to-end: .4a (design-detail) + .4b.1 (live surface: generate_loop_emit_prob knob + Module.generate_loop_gates + src/ir/generate_loop.rs + the to_sv_with_modules rendering + 9 lib proofs) + .4b.2a (num_emitted_generate_loops metric + introspection schema 1.9) + .4b.2b (the repo-owned tool_matrix --generate-loop-gate, banked /tmp/anvil-generate-loop-gate-r1) + .4b.3 (book/knobs/USER_GUIDE/README/KM closeout). Default-off / DUT byte-identical throughout (snapshots 6/6). Nothing retired.`

- ID: `STRUCTURED-EMISSION-EXPANSION.4a`
  Status: `done`
  Goal: `Design-detail leaf (no source): ground decision 0013's generate for loop surface in the real src/emit/sv.rs to_sv_with_modules + the {N{x}} replication source (the render_gate Concat predicate) + the function_emit.rs / soft_union.rs gen-time-annotation precedents + src/config.rs. Pin: (1) the replication-node selection rule (which Concats qualify; index-regularity); (2) gen-time annotation (Module.generate_loop_gates) vs emit-time; (3) the genvar / generate for rendering + inline-assign suppression; (4) the generate_loop_emit_prob knob semantics (default 0.0 byte-identical); (5) the saw_generate_loop_emit downstream-gate shape. DEVELOPMENT_NOTES design-detail entry + the .4b impl shape.`
  Acceptance: `A DEVELOPMENT_NOTES design-detail entry resolving the five points grounded in real code; tree split recorded; no source change; docs/workflow self-checks clean.`
  Result: `Done. DEVELOPMENT_NOTES design-detail entry resolves all five points, grounded in a fresh read of src/emit/sv.rs (render_gate's Concat replication predicate at sv.rs:1159 — operands.len() >= 2 && operands.iter().all(same NodeId) ⇒ {N{x}}; the to_sv_with_modules function-decl section template; build_names/node_ref/param_width_decl_w), src/ir/function_emit.rs + src/ir/soft_union.rs (the gen-time-annotation precedent + the function_emit_gate defensive re-check), src/gen/mod.rs (the two guarded call-site rolls), src/config.rs (default_function_emit_prob / validation list), and src/ir/mod.rs (pub mod registration). (1) First-cut selection = a {N{x}} replication Concat with a 1-BIT LANE (operands all the same NodeId, lane width == 1 ⇒ W == N ⇒ assign <wire>[gi] = <x> is byte-faithful); the common one-hot {W{sel}} broadcast idiom. Wider-lane part-select = recorded follow-up (nothing retired). Mutual exclusion with function_emit (which accepts Concat): run generate-loop annotation AFTER function_emit, exclude m.function_emit_gates (the soft_union→function_emit "later pass excludes earlier marks" precedent). (2) Gen-time annotation: new src/ir/generate_loop.rs annotate_generate_loop_gates(m, rng, prob) + Module.generate_loop_gates: BTreeSet<NodeId> (emitter-surface annotation only — flat IR / validators / CSE / canonical_module_signature untouched; param_env modules skipped); two guarded call-site rolls (generate_module + generate_design). (3) Rendering: a generate_loop_gate(m, idx) defensive accessor + a new generate-block section after the function-decl section emitting genvar <wire>__gi; generate for (<wire>__gi=0; <gi> < N; <gi>++) begin : <wire>__gen assign <wire>[<gi>] = <x>; end endgenerate; the per-gate assign loop continues past a marked gate to suppress the inline {N{x}} assign. gi++ probed clean; gi=gi+1 fallback. (4) Config::generate_loop_emit_prob (default 0.0, default_generate_loop_emit_prob serde default + Default + 0.0..=1.0 validation), config-file-only (no CLI flag, the function_emit_prob precedent) ⇒ default byte-identical, snapshots untouched; no introspection schema bump for the knob (rides request.knobs); a num_emitted_generate_loops metric in .4b would bump 1.8→1.9 (the .2b.2a precedent). (5) Downstream gate = tool_matrix --generate-loop-gate + ScenarioSet::GenerateLoopSweep (comb-only function-emit-gate parallel) + ModuleReport.emitted_generate_loop SV-text detection + saw_generate_loop_emit fact (Verilator + both Yosys, full plan — a generate for is universally synthesizable, unlike the Verilator-only union soft up-opt) + early-return gap enforcement; flagged the load-bearing gate-shape risk (the corpus must actually emit {N{x}} 1-bit replications — the one-hot mux-mask idiom — verified via the banked forced sweep). .4b impl shape recorded (single slice, or pre-split .4b.1 live / .4b.2 gate+metric / .4b.3 closeout if too broad). Rejected: wider-lane part-select first cut, pure emit-time pass, new IR Generate node, changing the default.`
  Verification: `done`
  Commit: `done`

- ID: `STRUCTURED-EMISSION-EXPANSION.4b`
  Status: `done`
  Goal: `Implement the .4a design: the generate_loop_emit_prob knob + Module.generate_loop_gates + src/ir/generate_loop.rs + the emitter rendering + lib proofs (.4b.1) + the repo-owned tool_matrix --generate-loop-gate + the num_emitted_generate_loops metric + saw_generate_loop_emit (.4b.2) + book/USER_GUIDE/README/KM closeout (.4b.3). Default-off / DUT byte-identical (snapshots untouched).`
  Children: `STRUCTURED-EMISSION-EXPANSION.4b.1`, `STRUCTURED-EMISSION-EXPANSION.4b.2`, `STRUCTURED-EMISSION-EXPANSION.4b.3`
  Result: `Done (closed by .4b.3, 2026-06-16). All of .4b.1 (live surface), .4b.2 (.4b.2a metric + schema 1.9 + .4b.2b the tool_matrix gate), and .4b.3 (docs closeout) complete. The generate for loop emit-projection is delivered end-to-end and downstream-clean (banked /tmp/anvil-generate-loop-gate-r1). Default-off / DUT byte-identical.`

- ID: `STRUCTURED-EMISSION-EXPANSION.4b.1`
  Status: `done`
  Goal: `The live first-cut surface: Config::generate_loop_emit_prob (default 0.0, serde default) + Module.generate_loop_gates: BTreeSet<NodeId> + src/ir/generate_loop.rs annotate_generate_loop_gates(m, rng, prob) (collect {N{x}} 1-bit-lane replication Concat candidates excluding function_emit marks, roll gen_bool(prob), mark) + the two guarded generator call-site rolls (after function_emit) + the to_sv_with_modules generate_loop_gate accessor + render_generate_loop_block + the generate-block section + the assign-loop inline-replication suppression + lib proofs (a marked gate emits a behaviour-preserving generate for + the inline {N{x}} suppressed; default-off byte-identical; the mark leaves CSE/canonical_module_signature untouched) + a forced-knob Verilator/Yosys/Icarus spot-check. Default-off / DUT byte-identical (snapshots 6/6).`
  Acceptance: `cargo check/clippy(-D warnings)/fmt clean; cargo test --lib green incl. the new generate_loop proofs; cargo test --test snapshots 6/6 byte-identical (default-off); a forced generate_loop_emit_prob=1.0 sample lints clean under Verilator + both Yosys + Icarus. Committed through COMMIT.md with the leaf id.`
  Result: `Done. Config::generate_loop_emit_prob (default 0.0, default_generate_loop_emit_prob() serde default; added to the Default impl + the 0.0..=1.0 validation list) + Module.generate_loop_gates: BTreeSet<NodeId> (Default-empty; emitter-surface annotation only — flat IR / validators / CSE / canonical_module_signature untouched, disjoint from function_emit_gates) + new src/ir/generate_loop.rs annotate_generate_loop_gates(m, rng, prob) (gen-time mark; the function_emit.rs precedent; candidate = a GateOp::Concat of the {N{x}} form — >= 2 operands all the same NodeId — with a 1-BIT LANE so result width == N and assign <wire>[gi] = <x> is byte-faithful; excludes function_emit_gates + soft_union_slice_gates; skips param_env modules) + call-site rolls in BOTH generate_module and generate_design guarded on prob > 0.0, run AFTER the function_emit pass (so function-emit marks are excluded) + src/emit/sv.rs rendering: a generate-block section (after the function-decl section, before the gate assigns) emitting per marked gate genvar <wire>__gi; generate for (<wire>__gi = 0; <wire>__gi < N; <wire>__gi = <wire>__gi + 1) begin : <wire>__gen assign <wire>[<wire>__gi] = <x>; end endgenerate, and the per-gate assign loop continues past a marked gate so the inline assign <wire> = {N{x}}; is suppressed. Helpers: generate_loop_gate (marked + defensively-revalidated lookup returning (lane, N)), render_generate_loop_block. INCREMENT FORM: gi = gi + 1 (the maximally-portable form; decision 0013 rendered gi++, .4a recorded gi=gi+1 as the portable fallback — implemented the fallback; verified clean; nothing retired). NO schema bump (default-off prob-knob precedent: the .2b.1 function_emit_prob knob also rode the existing schema_version via #[serde(default)]; the .4b.2 num_emitted_generate_loops metric bumps 1.8→1.9). 9 lib proofs (mark/skip single-operand/non-replication/wide-lane/function-emit-excluded/param-env + identity-and-node-count-untouched + end-to-end emit). 1-bit-lane replications are the common one-hot {W{sel}} mux-mask idiom: a forced generate_loop_emit_prob=1.0 default-config probe lit a generate for on 27/30 seeds.`
  Verification: `cargo check --all-targets clean; cargo clippy --all-targets -- -D warnings clean; cargo fmt --all --check clean; cargo test --lib 477 passed / 2 ignored (incl. 9 new generate_loop proofs; introspect schema_version 1.8 + umbrella DUT-byte-identical still green); cargo test --test snapshots 6/6 byte-identical (default-off). Forced generate_loop_emit_prob=1.0 sweep (5 seeds 1-5, 62-168 loops each, banked /tmp/anvil-gl-r1/): Verilator --lint-only 5/5 rc=0 / 0 warnings (repo bar), -Wall ON-vs-OFF delta = 0 (change adds no new warnings; residual -Wall UNUSEDSIGNAL is pre-existing, identical ON and OFF), Yosys without-abc 5/5, with-abc 5/5, Icarus iverilog -g2012 5/5.`
  Commit: `done`

- ID: `STRUCTURED-EMISSION-EXPANSION.4b.2`
  Status: `done`
  Goal: `The repo-owned downstream gate + metric for the generate for loop surface. Pre-split (2026-06-16) into .4b.2a (the num_emitted_generate_loops metric + introspection schema bump 1.8→1.9 — done) + .4b.2b (the tool_matrix --generate-loop-gate + saw_generate_loop_emit coverage fact + ModuleReport.emitted_generate_loop detection + early-return gap enforcement — done). Default-off / DUT byte-identical.`
  Children: `STRUCTURED-EMISSION-EXPANSION.4b.2a`, `STRUCTURED-EMISSION-EXPANSION.4b.2b`
  Result: `Done (closed by .4b.2b, 2026-06-16). .4b.2a (metric + introspection schema 1.9) + .4b.2b (the repo-owned tool_matrix --generate-loop-gate, banked clean /tmp/anvil-generate-loop-gate-r1) both complete. Default-off / DUT byte-identical.`

- ID: `STRUCTURED-EMISSION-EXPANSION.4b.2a`
  Status: `done`
  Goal: `The num_emitted_generate_loops metric: add Metrics::num_emitted_generate_loops (usize, #[serde(default)]) computed in metrics::compute() as m.generate_loop_gates.len(); it surfaces in introspection module_metrics (the SCHEMA-DERIVED projection), so bump the introspection schema MINOR 1.8 → 1.9 (SCHEMA_VERSION const + the schema_version test assertions in src/introspect/mod.rs + src/mcp/mod.rs + the docs/AGENT_INTROSPECTION_SCHEMA.md changelog/§7 lines + README/USER_GUIDE/book current-output refs + the CODEBASE_ANALYSIS envelope line). A lib proof that a module with marked generate_loop_gates reports the count. Default-off / DUT byte-identical (a post-hoc Metrics field changes no emitted RTL; snapshots untouched).`
  Acceptance: `cargo check/clippy(-D warnings)/fmt clean; cargo test --lib green incl. the metric proof + the schema_version 1.9 assertions; cargo test --test snapshots 6/6 byte-identical; the schema doc records the 1.8 → 1.9 additive MINOR bump. Committed through COMMIT.md with the leaf id.`
  Result: `Done. Metrics::num_emitted_generate_loops: usize (#[serde(default)]) added to src/metrics.rs, computed in metrics::compute() as m.generate_loop_gates.len() (a post-hoc structural count of an emitter-surface annotation; reads 0 by default, the configured count when generate_loop_emit_prob fired). Surfaced in introspection module_metrics (Metrics is the exact serde projection), so SCHEMA_VERSION bumped 1.8 → 1.9 in src/introspect/mod.rs. The metric BUMPS the schema (new derived Metrics field — the 1.7→1.8 num_emitted_combinational_functions precedent) whereas the .4b.1 knob did NOT (default-off prob-knob rides request.knobs via #[serde(default)]). Bumped all current-output schema refs to 1.9: the 9 schema_version assertions (2 in src/introspect/mod.rs + 7 in src/mcp/mod.rs), the schema doc (1.8→1.9 changelog entry + the defines/checklist lines), README (--introspect + analyze), USER_GUIDE (--introspect), the 5 book agent-mcp.md example JSONs, and the CODEBASE_ANALYSIS envelope line. Historical "landed at schema X" attributions left intact (README/USER_GUIDE num_emitted_combinational_functions @ 1.8; sv-version @ 1.2; the schema-doc 1.7→1.8 changelog entry). Lib proof metrics_count_emitted_generate_loops (unmarked 0, marked 1).`
  Verification: `cargo clippy --all-targets -- -D warnings clean; cargo fmt --all --check clean; cargo test --lib 478 passed / 2 ignored (the new metric proof + all schema_version assertions green at 1.9); cargo test --test snapshots 6/6 byte-identical (default-off; metric changes no RTL); end-to-end --introspect: default seed ⇒ schema_version 1.9 + num_emitted_generate_loops 0; forced generate_loop_emit_prob=1.0 ⇒ 1.9 + 50; mdbook build book OK.`
  Commit: `done`

- ID: `STRUCTURED-EMISSION-EXPANSION.4b.2b`
  Status: `done`
  Goal: `The repo-owned tool_matrix gate: a saw_generate_loop_emit coverage fact + a --generate-loop-gate flag + ScenarioSet::GenerateLoopSweep + build_generate_loop_sweep_scenarios (a replication-rich comb-only DUT forcing generate_loop_emit_prob=1.0 across the three construction strategies — must actually emit {N{x}} 1-bit replications; the share-heavy comb config with one-hot mux-mask broadcasts is the source) + a ModuleReport.emitted_generate_loop detection (SV-text contains "generate"/"genvar", #[serde(default)]) + coverage-gap enforcement (early-return arm in compute_coverage_gaps), proving Verilator + both Yosys modes + Icarus accept the emitted loops warning-clean. Bank a clean report (/tmp/anvil-generate-loop-gate-r1). Default-off / DUT byte-identical. Template: --function-emit-gate; the new field threaded through the ModuleReport fixtures.`
  Acceptance: `cargo check/clippy(-D warnings)/fmt clean; the repo-owned gate is banked clean (Verilator + both Yosys + Icarus) with saw_generate_loop_emit lit and coverage_gaps=[]; snapshots 6/6 byte-identical; committed through COMMIT.md with the leaf id.`
  Result: `Done. src/bin/tool_matrix.rs gains the repo-owned --generate-loop-gate, templated on --function-emit-gate (.2b.2b). New: --generate-loop-gate CLI flag + ScenarioSet::GenerateLoopSweep + MatrixReport.generate_loop_gate (wired into select_scenario_set [mutually exclusive], derive_run_plan [GENERATE_LOOP_SWEEP_MIN_UNITS_PER_SCENARIO=4 units/scenario floor + fail_on_coverage_gap], build_scenarios, scenario_set_slug "generate-loop-sweep", artifact_kind_slug "module"). build_generate_loop_sweep_scenarios + generate_loop_focus_config: one comb-only single-module DUT (function_emit_focus_config-shaped: node-id + e-graph, flop_prob = 0.0) with generate_loop_emit_prob = 1.0 across all three construction strategies (3 scenarios). ModuleReport.emitted_generate_loop (#[serde(default)]) set in materialize_prepared_module from prepared.sv_text.contains("generate"). CoverageSummary.saw_generate_loop_emit lit in summarize_coverage when an emitted-loop module is accepted by Verilator success AND a non-empty clean Yosys vec (a generate for is universally synthesizable like a function, so the gate runs the full tool plan; Icarus rides ToolSummary::any_failed); merged in merge_coverage; enforced by an early-return arm in compute_coverage_gaps after the universal construction-strategy coverage. 5 cargo-portable proofs + the new field threaded through 6 ModuleReport fixtures + the test_cli default. No schema bump (harness-only). Default generate_loop_emit_prob = 0.0 emission byte-identical (snapshots 6/6). Closes .4b.2 / frontier -> .4b.3.`
  Verification: `cargo check --bin tool_matrix clean; cargo clippy --all-targets -- -D warnings clean; cargo fmt --all --check clean; cargo test --bin tool_matrix 63 passed / 1 ignored (incl. 5 new generate-loop gate proofs); cargo test --test snapshots 6/6 byte-identical (harness-only). Repo-owned downstream bank /tmp/anvil-generate-loop-gate-r1 (--generate-loop-gate --yosys-mode both --iverilog-compile): 3 scenarios / 12 modules / 8 emitting a generate loop / coverage_gaps = [] / saw_generate_loop_emit = true / Verilator 12/0 / Yosys without-abc 12/0 / Yosys with-abc 12/0 / Icarus compile 12/0.`
  Commit: `done`

- ID: `STRUCTURED-EMISSION-EXPANSION.4b.3`
  Status: `done`
  Goal: `The user-facing closeout: extend the How It Works book chapter book/src/structured-emission.md with the generate for loop surface (byte-verified before/after; the {N{x}} 1-bit-lane rule; the wider-lane exclusion; the gi=gi+1 form) + the generate_loop_emit_prob knob entry in book/src/knobs.md / USER_GUIDE.md / README "Current CLI truth" (config-file-only knob) + a Knowledge Map how-to card if warranted (decision 0013 already carries answers:). Default-off / DUT byte-identical (docs-only).`
  Acceptance: `book builds (mdbook build book); USER_GUIDE + README updated; KM regenerated + check_knowledge_map clean; self-checks clean; cargo test --test book_examples 3/3; committed through COMMIT.md with the leaf id.`
  Result: `Done. book/src/structured-emission.md gains a "## The second surface: a generate for loop" section (the index-regular {N{x}} source rationale; a BYTE-VERIFIED seed-12 before/after — the inline assign concat_0 = {5{slice_0}}; becomes the genvar/generate for block, everything else byte-identical; the 1-bit-lane qualification rule + the wider-lane part-select exclusion [nothing retired] + the function_emit mutual exclusion; the gi = gi + 1 increment note; the num_emitted_generate_loops metric @ schema 1.9 + the tool_matrix --generate-loop-gate proof; a skip-sentinelled repro bash block) + the chapter intro updated to note generate is now live. The generate_loop_emit_prob knob entry added to book/src/knobs.md (the ### Structured emission subsection, beside function_emit_prob), USER_GUIDE.md (after the function_emit_prob config-knob bullet; intro pluralised), and the README "Current CLI truth" (a config-file-only knob bullet after the function_emit_prob bullet). New Knowledge Map how-to card docs/knowledge/generate-loop-emit.md (id generate-loop-emit) with how-to question keys distinct from decision 0013's conceptual keys + a validated reverify command (dump-config -> set generate_loop_emit_prob=1.0 + small comb shape -> generate seed 12 -> grep "generate" -> verilator --lint-only). KM regenerated (38 -> 39 facts, 296 -> 309 question keys). The book example is byte-verified downstream-clean (Verilator -Wall with the matching filename + both Yosys + Icarus). Docs-only / DUT byte-identical (no source touched). With this leaf, .4b.3 / .4b / .4 all close: the second structured surface (the generate for loop emit-projection) is delivered end-to-end. The tree stays active as an open-ended lane with no current frontier; future surfaces (task / nested generate / interface/modport) are .5+, each its own decision when picked. Nothing retired.`
  Verification: `mdbook build book clean (HTML written, no broken-link warnings); bash knowledge-map/scripts/gen_knowledge_map.sh (39 facts / 309 keys) + bash knowledge-map/scripts/check_knowledge_map.sh OK (facts valid, ids unique, map in sync); bash scripts/check_memory_architecture.sh all invariants hold (0013 indexed); cargo test --test book_examples 3/3 (skip_sentinels_have_reasons + every_runnable_book_bash_block_succeeds green — the new repro block correctly skip-sentinelled). Docs-only: no src/ touched, so cargo check/clippy/fmt unaffected; the seed-12 before/after was byte-verified against the release binary (generate_loop_emit_prob 0.0 vs 1.0 diff = exactly the {5{slice_0}} replication becoming the genvar/generate for block) and the example lints clean under verilator --lint-only -Wall (matching filename) + both Yosys + Icarus.`
  Commit: `done`

- ID: `STRUCTURED-EMISSION-EXPANSION.5`
  Status: `done`
  Goal: `Design/decision leaf for the THIRD structured surface (the leading future candidate from decision 0013: task): re-confirm the candidate ranking with current tool evidence, pick the next concrete synthesizable + downstream-clean surface, define its valid-by-construction discipline + opt-in knob + downstream gate, and split the tree — before any code.`
  Acceptance: `A decision record naming the third surface, its construction discipline, and its downstream gate; an empirical tool-acceptance grounding; no source change; self-checks clean (mdbook + check_knowledge_map + check_memory_architecture). Committed through COMMIT.md with the leaf id.`
  Result: `Decision 0014. The third richer-structured surface is a default-off, opt-in, valid-by-construction combinational task automatic emitted as a behaviour-preserving projection of a single combinational gate — the exact parallel of decision 0012's combinational function, but a procedural task with an output argument (called from always_comb) rather than a value-returning function. For a marked gate <wire> = op(o0,o1,…) of width W: task automatic <wire>__t(output logic [W-1:0] o, input logic [Wi-1:0] a0, …); o = a0 op a1 …; endtask + (minimal-blast-radius form) logic [W-1:0] <wire>__tv; always_comb <wire>__t(<wire>__tv, <operand refs>); assign <wire> = <wire>__tv; — so the existing <wire> net is driven from the task output, downstream refs unchanged. First cut = single-gate operand task (the decision 0012 single-gate parallel; zero sharing/scoping hazard). Grounding (empirical, this session): a combinational task automatic called from always_comb is accepted warning-clean by Verilator 5.046 -Wall + both repo Yosys modes + Icarus iverilog -g2012, in BOTH the direct-output form and the output-var + passthrough-assign minimal-blast-radius form. Chosen over nested/multi-level generate (deeper variant of an already-shipped surface; more emitter surgery) and interface/modport (still weak/inconsistent Yosys synth). task was already the recorded leading future candidate (decision 0013). Discipline: rules-first (mark an already-valid gate; never generate-then-filter), default-off task_emit_prob (proposed; default 0.0) ⇒ byte-identical / snapshots untouched, no new IR node / no new whole-module behaviour (the function_emit/generate_loop/soft_union precedent); mutually exclusive with the sibling projections; combinational only; structured selectors + Slice excluded (same reasons as function_emit); nothing retired. Downstream gate: tool_matrix --task-emit-gate (templated on --function-emit-gate) proving Verilator + both Yosys modes + Icarus accept the tasks warning-clean, gated on a saw_combinational_task_emit fact. Rejected: interface/modport first, nested generate first, multi-output/side-effecting task in the first cut, multi-gate-cone task body, a semantic IR Task node, generate-then-filter, changing the default. Split into .5 (done) + .6 (impl; pre-split .6a design-detail + .6b impl) + future (.7+: nested/multi-level generate, interface/modport, richer tasks). KM card structured-emission-third-surface-combinational-task (decision carries answers:).`
  Verification: `done`
  Commit: `done`

- ID: `STRUCTURED-EMISSION-EXPANSION.6`
  Status: `done`
  Goal: `Implement the third structured surface (the combinational task automatic emit-projection) per decision 0014: the task_emit_prob knob + the rules-first single-gate selection (gen-time annotation + Module.task_emit_gates, excluding the sibling projections) + the emitter rendering (task automatic decl + the always_comb call + the output-var passthrough) + the downstream-clean gate (saw_combinational_task_emit) + the num_emitted_combinational_tasks metric + book/USER_GUIDE/KM. Default-off / DUT byte-identical.`
  Children: `STRUCTURED-EMISSION-EXPANSION.6a`, `STRUCTURED-EMISSION-EXPANSION.6b`
  Result: `Done (closed by .6b.3, 2026-06-16). The third structured surface — the combinational task automatic emit-projection (decision 0014) — is delivered end-to-end: .6a (design-detail) + .6b.1 (live surface: task_emit_prob knob + Module.task_emit_gates + src/ir/task_emit.rs + the to_sv_with_modules task section reusing render_gate_function_body + 11 lib proofs) + .6b.2a (num_emitted_combinational_tasks metric + introspection schema 1.10) + .6b.2b (the repo-owned tool_matrix --task-emit-gate, banked /tmp/anvil-task-emit-gate-r1) + .6b.3 (book/knobs/USER_GUIDE/README/KM closeout). Default-off / DUT byte-identical throughout (snapshots 6/6). Nothing retired.`

- ID: `STRUCTURED-EMISSION-EXPANSION.6a`
  Status: `done`
  Goal: `Design-detail leaf (no source): ground decision 0014's combinational task automatic surface in the real src/emit/sv.rs to_sv_with_modules (the function-decl / generate-block sections + the per-gate assign loop) + the function_emit.rs / generate_loop.rs gen-time-annotation precedents + src/config.rs / src/gen/mod.rs. Pin: (1) the net-vs-var integration; (2) gen-time annotation (Module.task_emit_gates) + candidate predicate; (3) the task automatic decl + body (reuse render_gate_function_body) + always_comb call rendering + suppression; (4) the task_emit_prob knob; (5) the saw_combinational_task_emit gate + num_emitted_combinational_tasks metric. DEVELOPMENT_NOTES design-detail entry + the .6b impl shape.`
  Acceptance: `A DEVELOPMENT_NOTES design-detail entry resolving the five points grounded in real code; tree split recorded; no source change; docs/workflow self-checks clean.`
  Result: `Done. DEVELOPMENT_NOTES design-detail entry resolves all five points, grounded in a fresh read of src/emit/sv.rs (the to_sv_with_modules function-decl + generate-block sections as the structural template; the per-gate assign loop continue pattern; the REUSE of render_gate_function_body verbatim as the task body), src/ir/function_emit.rs + src/ir/generate_loop.rs (the gen-time-annotation chain), src/gen/mod.rs (the guarded call-site rolls), src/config.rs (default + validation), src/ir/mod.rs (pub mod registration). (1) Net-vs-var = the output-var + passthrough-assign form (keep <wire> a net, add logic <wire>__tv, always_comb <wire>__t(<wire>__tv, …), change the gate's assign RHS to <wire>__tv — only the gate's own drive changes, wire-decl section uniform, the function_emit parallel); <wire>-as-var rejected for the first cut (touches the wire-decl section). One always_comb per task gate. (2) Gen-time src/ir/task_emit.rs annotate_task_emit_gates(m, rng, prob) + Module.task_emit_gates: BTreeSet<NodeId>; candidate = the SAME predicate as function_emit (ordinary combinational Gate, not structured/Slice, ≥1 operand) PLUS exclusion of function_emit_gates + generate_loop_gates + soft_union_slice_gates; runs AFTER generate_loop (later pass excludes earlier marks); param_env skipped. (3) task_emit_gate accessor + a task automatic <wire>__t(output logic [W-1:0] o, input …); o = render_gate_function_body(op, widths); endtask decl + logic <wire>__tv; always_comb <wire>__t(<wire>__tv, <operand refs>); + the assign-RHS swap to <wire>__tv (positional args handle duplicate operands). (4) Config::task_emit_prob (default 0.0, config-file-only) ⇒ byte-identical, no schema bump for the knob. (5) num_emitted_combinational_tasks metric ⇒ schema 1.9→1.10; tool_matrix --task-emit-gate + ScenarioSet::TaskEmitSweep + ModuleReport.emitted_combinational_task (sv_text.contains("task automatic")) + saw_combinational_task_emit (Verilator + both Yosys, full plan). .6b impl shape recorded (pre-split .6b.1 live / .6b.2 metric+gate / .6b.3 docs per the .4b precedent). Rejected: <wire>-as-var first cut, multi-output/side-effecting task, multi-gate-cone body, a new IR Task node, changing the default.`
  Verification: `done`
  Commit: `done`

- ID: `STRUCTURED-EMISSION-EXPANSION.6b`
  Status: `done`
  Goal: `Implement the .6a design: Config::task_emit_prob (default 0.0) + Module.task_emit_gates + src/ir/task_emit.rs (annotate_task_emit_gates + the candidate predicate excluding the sibling projections) + the two guarded gen-time call-site rolls (after generate_loop) + the to_sv_with_modules task_emit_gate accessor + the task automatic decl (body via render_gate_function_body) + the logic <wire>__tv var + the always_comb call + the assign-RHS swap + lib proofs + the num_emitted_combinational_tasks metric (schema 1.9→1.10) + the repo-owned tool_matrix --task-emit-gate + ModuleReport.emitted_combinational_task + saw_combinational_task_emit + book/USER_GUIDE/KM. Default-off / DUT byte-identical (snapshots untouched). Pre-split into .6b.1 (live surface) + .6b.2 (metric + gate) + .6b.3 (docs closeout) per the .4b precedent.`
  Children: `STRUCTURED-EMISSION-EXPANSION.6b.1`, `STRUCTURED-EMISSION-EXPANSION.6b.2`, `STRUCTURED-EMISSION-EXPANSION.6b.3`
  Result: `Done (closed by .6b.3, 2026-06-16). All of .6b.1 (live surface), .6b.2 (.6b.2a metric + schema 1.10 + .6b.2b the tool_matrix gate), and .6b.3 (docs closeout) complete. The combinational task automatic emit-projection is delivered end-to-end and downstream-clean (banked /tmp/anvil-task-emit-gate-r1). Default-off / DUT byte-identical.`

- ID: `STRUCTURED-EMISSION-EXPANSION.6b.1`
  Status: `done`
  Goal: `The live first-cut surface: Config::task_emit_prob (default 0.0, serde default) + Module.task_emit_gates: BTreeSet<NodeId> + src/ir/task_emit.rs annotate_task_emit_gates(m, rng, prob) (collect the function-emit candidate set — non-structured/non-Slice Gate with >= 1 operand — excluding function_emit_gates/generate_loop_gates/soft_union_slice_gates, roll gen_bool(prob), mark) + the two guarded generator call-site rolls (after generate_loop) + the to_sv_with_modules task_emit_gate accessor + render_gate_task_decl (task automatic <wire>__t(output logic [W-1:0] o, input ...); o = render_gate_function_body; endtask) + render_gate_task_call (logic [W-1:0] <wire>__tv; always_comb <wire>__t(<wire>__tv, <refs>);) + the gate-assign-loop passthrough assign <wire> = <wire>__tv; + lib proofs (a marked gate emits a behaviour-preserving task + always_comb call + passthrough; default-off byte-identical; the mark leaves CSE/canonical_module_signature untouched) + a forced-knob Verilator/Yosys/Icarus spot-check. Default-off / DUT byte-identical (snapshots 6/6).`
  Acceptance: `cargo check/clippy(-D warnings)/fmt clean; cargo test --lib green incl. the new task_emit proofs; cargo test --test snapshots 6/6 byte-identical (default-off); a forced task_emit_prob=1.0 sample lints clean under Verilator + both Yosys + Icarus. Committed through COMMIT.md with the leaf id.`
  Result: `Done. Config::task_emit_prob (default 0.0, default_task_emit_prob() serde default; added to the Default impl + the 0.0..=1.0 validation list) + Module.task_emit_gates: BTreeSet<NodeId> (Default-empty; emitter-surface annotation only — flat IR / validators / CSE / canonical_module_signature untouched, disjoint from function_emit_gates/generate_loop_gates/soft_union_slice_gates) + new src/ir/task_emit.rs annotate_task_emit_gates(m, rng, prob) (gen-time mark; the function_emit.rs precedent; candidate = the function-emit candidate set PLUS exclusion of the three sibling projections; skips param_env modules) + call-site rolls in BOTH generate_module and generate_design guarded on prob > 0.0, run AFTER the generate_loop pass (so the sibling marks are excluded) + src/emit/sv.rs rendering: a task section (after the generate-loop section) emitting per marked gate task automatic <wire>__t(output logic [W-1:0] o, input logic [Wi-1:0] a0, ...); o = <op over a0..a{n-1}>; endtask (render_gate_task_decl, body REUSES render_gate_function_body verbatim) + logic [W-1:0] <wire>__tv; always_comb <wire>__t(<wire>__tv, <operand refs>); (render_gate_task_call), and the gate-assign loop rewrites the marked gate's assign to the passthrough assign <wire> = <wire>__tv;. Helpers: task_emit_gate (marked + defensively-revalidated lookup, mirrors function_emit_gate), render_gate_task_decl, render_gate_task_call. Output-var + passthrough integration (the .6a first cut; <wire> stays a net; <wire>-as-var rejected). NO schema bump (default-off prob-knob precedent: function_emit_prob/generate_loop_emit_prob also rode the existing schema_version via #[serde(default)]; the .6b.2 num_emitted_combinational_tasks metric bumps 1.9→1.10). 11 lib proofs (mark/prob-0/structured/Slice/each-sibling-exclusion/param-env + identity-and-node-count-untouched + end-to-end emit shape + duplicate-operand positional params). Frontier -> .6b.2.`
  Verification: `cargo check --all-targets clean; cargo clippy --all-targets -- -D warnings clean; cargo fmt --all --check clean; cargo test --lib 489 passed / 2 ignored (incl. 11 new task_emit proofs; introspect schema_version 1.9 + umbrella DUT-byte-identical still green); cargo test --test snapshots 6/6 byte-identical (default-off). Forced task_emit_prob=1.0 sweep (5 seeds: 1/7/42/100/2024, 4-39 tasks each, banked /tmp/anvil-te-r1/): Verilator --lint-only 5/5 CLEAN (repo bar) + -Wall ON-vs-OFF delta = 0 (the task projection adds no new warnings), Yosys without-abc 5/5 + with-abc 5/5, Icarus iverilog -g2012 5/5 CLEAN. (Tools: Verilator 5.046, Yosys 0.64, iverilog.)`
  Commit: `done`

- ID: `STRUCTURED-EMISSION-EXPANSION.6b.2`
  Status: `active`
  Goal: `The repo-owned downstream gate + metric for the combinational task automatic surface. Pre-split per the .4b.2 precedent into .6b.2a (the num_emitted_combinational_tasks metric = m.task_emit_gates.len() + introspection schema bump 1.9→1.10) + .6b.2b (the tool_matrix --task-emit-gate + ScenarioSet::TaskEmitSweep + ModuleReport.emitted_combinational_task SV-text detection ("task automatic") + saw_combinational_task_emit coverage fact + early-return gap enforcement, templated on --function-emit-gate / --generate-loop-gate; banked clean Verilator + both Yosys + Icarus). Default-off / DUT byte-identical.`
  Children: `STRUCTURED-EMISSION-EXPANSION.6b.2a`, `STRUCTURED-EMISSION-EXPANSION.6b.2b`
  Result: `Done (closed by .6b.2b, 2026-06-16). .6b.2a (the num_emitted_combinational_tasks metric + introspection schema 1.10) + .6b.2b (the repo-owned tool_matrix --task-emit-gate, banked clean /tmp/anvil-task-emit-gate-r1) both complete. Default-off / DUT byte-identical.`

- ID: `STRUCTURED-EMISSION-EXPANSION.6b.2a`
  Status: `done`
  Goal: `The num_emitted_combinational_tasks metric: add Metrics::num_emitted_combinational_tasks (usize, #[serde(default)]) computed in metrics::compute() as m.task_emit_gates.len(); it surfaces in introspection module_metrics (the SCHEMA-DERIVED projection), so bump the introspection schema MINOR 1.9 → 1.10 (SCHEMA_VERSION const + the schema_version assertions in src/introspect/mod.rs + src/mcp/mod.rs + the docs/AGENT_INTROSPECTION_SCHEMA.md changelog/version lines + README/USER_GUIDE current-output refs + the 5 book/src/agent-mcp.md example JSONs + the CODEBASE_ANALYSIS envelope line). A lib proof that a module with marked task_emit_gates reports the count. Default-off / DUT byte-identical (a post-hoc Metrics field changes no emitted RTL; snapshots untouched).`
  Acceptance: `cargo check/clippy(-D warnings)/fmt clean; cargo test --lib green incl. the metric proof + the schema_version 1.10 assertions; cargo test --test snapshots 6/6 byte-identical; the schema doc records the 1.9 → 1.10 additive MINOR bump. Committed through COMMIT.md with the leaf id.`
  Result: `Done. Metrics::num_emitted_combinational_tasks: usize (#[serde(default)]) added to src/metrics.rs, computed in metrics::compute() as m.task_emit_gates.len() (a post-hoc structural count of an emitter-surface annotation; reads 0 by default, the configured count when task_emit_prob fired). Surfaced in introspection module_metrics (Metrics is the exact serde projection), so SCHEMA_VERSION bumped 1.9 → 1.10 in src/introspect/mod.rs. The metric BUMPS the schema (new derived Metrics field — the 1.8→1.9 num_emitted_generate_loops precedent) whereas the .6b.1 knob did NOT (default-off prob-knob rides request.knobs via #[serde(default)]). MINOR is an integer, so 1.9 → 1.10 (ten), not a decimal — recorded in the SCHEMA_VERSION doc comment + the schema-doc changelog. Bumped all current-output schema refs to 1.10: the 9 schema_version assertions (2 in src/introspect/mod.rs + 7 in src/mcp/mod.rs), the schema doc (1.9→1.10 changelog entry + the early-example/defines/checklist lines), README (--introspect + analyze), USER_GUIDE (--sv-version --introspect row), the 5 book agent-mcp.md example JSONs, and the CODEBASE_ANALYSIS envelope line. Historical "landed at schema X" attributions left intact (README/USER_GUIDE num_emitted_generate_loops @ 1.9; num_emitted_combinational_functions @ 1.8; sv-version @ 1.2; the schema-doc 1.8→1.9 changelog entry). Lib proof metrics_count_emitted_combinational_tasks (unmarked 0, marked 1).`
  Verification: `cargo clippy --all-targets -- -D warnings clean; cargo fmt --all --check clean; cargo test --lib 490 passed / 2 ignored (the new metric proof + all schema_version assertions green at 1.10); cargo test --test snapshots 6/6 byte-identical (default-off; metric changes no RTL); end-to-end --introspect: default seed ⇒ schema_version 1.10 + num_emitted_combinational_tasks 0; forced task_emit_prob=1.0 (seed 42) ⇒ 1.10 + 39; mdbook build book OK.`
  Commit: `done`

- ID: `STRUCTURED-EMISSION-EXPANSION.6b.2b`
  Status: `done`
  Goal: `The repo-owned tool_matrix gate: a saw_combinational_task_emit coverage fact + a --task-emit-gate flag + ScenarioSet::TaskEmitSweep + build_task_emit_sweep_scenarios (one comb-only task_emit_prob=1.0 DUT forcing the task projection across the three construction strategies) + a ModuleReport.emitted_combinational_task detection (SV-text contains "task automatic", #[serde(default)]) + coverage-gap enforcement (early-return arm in compute_coverage_gaps), proving Verilator + both Yosys modes + Icarus accept the emitted tasks warning-clean. Bank a clean report (/tmp/anvil-task-emit-gate-r1). Default-off / DUT byte-identical. Template: --function-emit-gate / --generate-loop-gate; the new field threaded through the ModuleReport fixtures + the test_cli default.`
  Acceptance: `cargo check/clippy(-D warnings)/fmt clean; the repo-owned gate is banked clean (Verilator + both Yosys + Icarus) with saw_combinational_task_emit lit and coverage_gaps=[]; snapshots 6/6 byte-identical; committed through COMMIT.md with the leaf id.`
  Result: `Done. src/bin/tool_matrix.rs gains the repo-owned --task-emit-gate, templated on --generate-loop-gate (.4b.2b). New: --task-emit-gate CLI flag + ScenarioSet::TaskEmitSweep + MatrixReport.task_emit_gate (wired into select_scenario_set [mutually exclusive], derive_run_plan [TASK_EMIT_SWEEP_MIN_UNITS_PER_SCENARIO=4 units/scenario floor + fail_on_coverage_gap], build_scenarios, scenario_set_slug "task-emit-sweep", artifact_kind_slug "module"). build_task_emit_sweep_scenarios + task_emit_focus_config: one comb-only single-module DUT (function_emit_focus_config-shaped: node-id + e-graph, flop_prob = 0.0) with task_emit_prob = 1.0 across all three construction strategies (3 scenarios). ModuleReport.emitted_combinational_task (#[serde(default)]) set in materialize_prepared_module from prepared.sv_text.contains("task automatic"). CoverageSummary.saw_combinational_task_emit lit in summarize_coverage when an emitted-task module is accepted by Verilator success AND a non-empty clean Yosys vec (a combinational task is universally synthesizable like a function, so the gate runs the full tool plan; Icarus rides ToolSummary::any_failed); merged in merge_coverage; enforced by an early-return arm in compute_coverage_gaps after the universal construction-strategy coverage. 5 cargo-portable proofs + the new field threaded through 6 ModuleReport fixtures + the test_cli default. No schema bump (harness-only). Default task_emit_prob = 0.0 emission byte-identical (snapshots 6/6). Closes .6b.2 / frontier -> .6b.3.`
  Verification: `cargo check --bin tool_matrix clean; cargo clippy --bin tool_matrix -- -D warnings clean; cargo fmt --all --check clean; cargo test --bin tool_matrix 68 passed / 1 ignored (incl. 5 new task-emit gate proofs); cargo test --test snapshots 6/6 byte-identical (harness-only). Repo-owned downstream bank /tmp/anvil-task-emit-gate-r1 (--task-emit-gate --yosys-mode both --iverilog-compile): 3 scenarios / 12 modules / 12 emitting a task / coverage_gaps = [] / saw_combinational_task_emit = true / Verilator 12/0 / Yosys without-abc 12/0 / Yosys with-abc 12/0 / Icarus compile 12/0; all 12 modules emitted_combinational_task = true.`
  Commit: `done`

- ID: `STRUCTURED-EMISSION-EXPANSION.6b.3`
  Status: `done`
  Goal: `The user-facing closeout: extend the How It Works book chapter book/src/structured-emission.md with the task automatic surface (byte-verified before/after; the single-gate rule; the output-var passthrough form; the always_comb call; the metric + gate) + the task_emit_prob knob entry in book/src/knobs.md / USER_GUIDE.md / README "Current CLI truth" (config-file-only knob) + a Knowledge Map how-to card if warranted (decision 0014 already carries answers:). Default-off / DUT byte-identical (docs-only).`
  Acceptance: `book builds (mdbook build book); USER_GUIDE + README updated; KM regenerated + check_knowledge_map clean; self-checks clean; cargo test --test book_examples 3/3; committed through COMMIT.md with the leaf id.`
  Result: `Done. book/src/structured-emission.md gains a "## The third surface: a combinational task automatic" section (the function-surface parallel; a BYTE-VERIFIED seed-1 before/after — the inline assign shr_0 = i_2 >> 2'h3; becomes the task automatic shr_0__t(...) decl + logic shr_0__tv; + always_comb shr_0__t(shr_0__tv, i_2, 2'h3); + the passthrough assign shr_0 = shr_0__tv;, everything else byte-identical; the same candidate set as function_emit; the structured/Slice exclusions; the four-way mutual exclusion; the output-var passthrough integration; combinational-only; the metric @ schema 1.10 + the tool_matrix --task-emit-gate proof; a skip-sentinelled repro bash block) + the chapter intro updated to list task as live. The task_emit_prob knob entry added to book/src/knobs.md (the ### Structured emission subsection, beside function_emit_prob/generate_loop_emit_prob), USER_GUIDE.md (after the generate_loop_emit_prob config-knob bullet), and the README "Current CLI truth" (a config-file knob bullet after the generate_loop_emit_prob bullet). New Knowledge Map how-to card docs/knowledge/combinational-task-emit.md (id combinational-task-emit) with how-to question keys distinct from decision 0014's conceptual keys + a validated reverify command (dump-config -> set task_emit_prob=1.0 + comb-only -> generate seed 1 -> grep "task automatic" -> verilator --lint-only). KM regenerated (40 -> 41 facts, 318 -> 331 question keys). The book example is byte-verified downstream-clean (Verilator -Wall with matching filename + both Yosys + Icarus). Docs-only / DUT byte-identical (no source touched). With this leaf, .6b.3 / .6b / .6 all close: the third structured surface (the combinational task automatic emit-projection) is delivered end-to-end. The tree stays active as an open-ended lane with no current frontier; future surfaces (nested/multi-level generate / interface/modport / richer tasks) are .7+, each its own decision when picked. Nothing retired.`
  Verification: `mdbook build book clean (HTML written, no broken-link warnings); bash knowledge-map/scripts/gen_knowledge_map.sh (41 facts / 331 keys) + bash knowledge-map/scripts/check_knowledge_map.sh OK (facts valid, ids unique, map in sync); bash scripts/check_memory_architecture.sh all invariants hold (0014 indexed); cargo test --test book_examples 3/3 (skip_sentinels_have_reasons + every_runnable_book_bash_block_succeeds green — the new repro block correctly skip-sentinelled). Docs-only: no src/ touched, so cargo check/clippy/fmt unaffected; the seed-1 before/after was byte-verified against the release binary (task_emit_prob 0.0 vs 1.0 diff = exactly the shr_0__t task decl + the logic shr_0__tv var + the always_comb call + the assign rewritten to the passthrough) and the example lints clean under verilator --lint-only -Wall (matching filename) + both Yosys + Icarus.`
  Commit: `done`

- ID: `STRUCTURED-EMISSION-EXPANSION.7`
  Status: `done`
  Goal: `Design/decision leaf: at the no-active-frontier boundary, pick the FOURTH structured surface, define its valid-by-construction discipline + opt-in knob + downstream gate, re-confirm with a fresh empirical tool-acceptance probe, and split the tree — before any code.`
  Acceptance: `A decision record naming the fourth surface, its construction discipline, and its downstream gate, grounded in a fresh empirical probe; no source change; self-checks clean.`
  Result: `Decision 0015. The fourth richer-structured surface is a default-off, opt-in, valid-by-construction wider-lane generate for part-select — a behaviour-preserving broadening of the second surface (decision 0013) from the 1-bit lane to a lane of any width LW >= 1. For a marked replication {N{x}} whose lane x is LW bits (result N*LW), it renders generate for (gi=0; gi<N; gi=gi+1) assign <wire>[gi*LW +: LW] = <x>; (the LW==1 case keeps the existing assign <wire>[gi] = <x>; verbatim ⇒ the shipped 1-bit surface stays byte-identical). Bit-group g of {N{x}} is exactly the lane, so the unrolled loop is byte-equivalent to the inline replication. Chosen over interface/modport (EMPIRICALLY DISQUALIFIED this session: Icarus syntax-fails the modport port + both Yosys modes warn on the implicit interface-member decl — confirms the recorded weak-support claim) and nested/multi-level generate (clean but bigger blast radius + no routine by-construction 2D source) and constant-predicate generate if (dead untaken branch; frontend lane already exercises it). Fresh empirical probe (Verilator 5.046 -Wall + Yosys 0.64 both modes + Icarus 13.0): wider-lane part-select universally warning-clean + iverilog-simulation-proven bit-equal to {4{b}}. Discipline: rules-first (broaden the annotate_generate_loop_gates predicate; never generate-then-filter); REUSES the existing generate_loop_emit_prob knob (default 0.0 ⇒ byte-identical, snapshots untouched) and the num_emitted_generate_loops metric (NO new knob / NO new metric / NO introspection schema bump); no new IR node / no new computed truth. Downstream gate: the existing tool_matrix --generate-loop-gate covers wider lanes once the predicate is relaxed; .8 adds a focused wider-lane assertion. Rejected: interface/modport fourth, nested generate fourth, generate if fourth, an explicit per-bit unroll, a new IR node/knob/metric/schema bump, generate-then-filter, changing the default. Split into .7 (done) + .8 (impl) + future (.9+: nested/multi-level generate, interface/modport, richer tasks). Pre-split .8 → .8a (design-detail) + .8b (impl).`
  Verification: `done`
  Commit: `done`

- ID: `STRUCTURED-EMISSION-EXPANSION.8`
  Status: `done`
  Goal: `Implement the fourth structured surface (the wider-lane generate for part-select) per decision 0015: relax the generate_loop predicate from 1-bit-lane to LW >= 1 (width == N*LW); render the part-select loop body assign <wire>[gi*LW +: LW] = <x>; for LW > 1 while keeping the LW==1 body byte-identical; lib proofs (wider-lane mark; wider-lane emit shape; LW==1 still [gi] byte-identical; sim-faithful); prove the wider lane is exercised in tool_matrix --generate-loop-gate (Verilator + both Yosys + Icarus); book/USER_GUIDE update (replace the "wider lane stays inline" caveat). Default-off / DUT byte-identical (snapshots untouched). Reuses generate_loop_emit_prob + num_emitted_generate_loops (no new knob / no schema bump).`
  Children: `STRUCTURED-EMISSION-EXPANSION.8a`, `STRUCTURED-EMISSION-EXPANSION.8b`
  Result: `Done (closed by .8b, 2026-06-17). The fourth structured surface — the wider-lane generate for part-select — is delivered end-to-end: .8a (design-detail: resolved the open questions against the real generate_loop.rs/sv.rs + corpus-liveness probe) + .8b (the two surgical edits: relax gate_qualifies to LW >= 1 / width == N*LW + the render_generate_loop_block [gi*LW +: LW] branch for LW>1 keeping LW==1 byte-identical; 4 lib proofs; book/USER_GUIDE/README/knobs/CODEBASE_ANALYSIS/KM closeout; downstream-clean per-seed ON-vs-OFF sweep + the regression-clean --generate-loop-gate bank). Reuses generate_loop_emit_prob + num_emitted_generate_loops — no new knob, no new metric, no introspection schema bump. Default-off / DUT byte-identical (snapshots 6/6). Nothing retired.`

- ID: `STRUCTURED-EMISSION-EXPANSION.8a`
  Status: `done`
  Goal: `Design-detail leaf (no source): ground decision 0015 in the real src/ir/generate_loop.rs (gate_qualifies — the 1-bit-lane restriction to relax) + src/emit/sv.rs (generate_loop_gate returning (lane, N) — and whether to also return LW or recompute m.nodes[lane].width(); render_generate_loop_block — the [gi] vs [gi*LW +: LW] branch). Pin: (1) the relaxed predicate (LW >= 1, width == N*LW; keep the function-emit/soft_union exclusions); (2) the generate_loop_gate signature change ((lane, N) -> (lane, N) with LW recomputed, or (lane, N, LW)); (3) the render branch (LW==1 keeps [gi] byte-identical; LW>1 emits [gi*LW +: LW]); (4) the byte-identity contract for the shipped 1-bit surface (its proofs + the .4b gate must stay green unchanged); (5) the wider-lane downstream proof (a dedicated --generate-loop-gate wide-lane scenario/assertion vs asserting inside the existing gate, and whether a wider-lane coverage signal is warranted). DEVELOPMENT_NOTES design-detail entry + the .8b impl shape.`
  Acceptance: `A DEVELOPMENT_NOTES design-detail entry resolving the five points grounded in real code; tree split recorded; no source change; docs/workflow self-checks clean.`
  Result: `Done. DEVELOPMENT_NOTES design-detail entry resolves all open questions grounded in a fresh read of src/ir/generate_loop.rs (gate_qualifies) + src/emit/sv.rs (generate_loop_gate ~1512, render_generate_loop_block ~1548) AND a corpus-liveness probe. CORPUS-LIVENESS: a 300-module comb-only sweep (/tmp/anvil-widelane-probe/, seed 1, terminal_reuse_prob=0.95, gate_struct_weight=12, widths 4-16) emits 448 {N{x}} replications of which 20 have a multi-bit lane (LW>1) — e.g. {2{i_4}} 7b->14b, {3{case_mux_0}} 12b->36b, {6{i_1}} 8b->48b, {4{concat_7}} 20b->80b — so the broadened predicate fires on REAL generation (~4.5%), not hand-built-only; the existing --generate-loop-gate corpus exercises the new branch once relaxed. RESOLVED: (1) keep generate_loop_gate -> Option<(NodeId, usize)> (lane, N) unchanged; recompute LW = m.nodes[lane].width() in render_generate_loop_block (it already has m); (2) render branches if lw == 1 { assign <name>[<gi>] = <x>; } else { assign <name>[<gi>*LW +: LW] = <x>; } — do NOT collapse 1-bit into [gi*1 +: 1] (would change shipped bytes); (3) relax gate_qualifies: replace lane.width() != 1 || *width != operands.len() with lw = lane.width(); lw == 0 || *width != operands.len() * lw (any LW >= 1, width == N*LW), function_emit/soft_union exclusions unchanged, mirrored in the emitter defensive re-check; (4) byte-identity: 1-bit rendering verbatim so snapshots (default-off) + every shipped 1-bit generate_loop proof + the .4b gate stay green; reuses generate_loop_emit_prob + num_emitted_generate_loops => NO new knob / NO new metric / NO schema bump; (5) downstream proof = a deterministic lib emit-test (hand-built {3{x}} 4-bit lane -> assert [<gi>*4 +: 4]) + a 1-bit-still-[gi] byte-identity guard + the existing --generate-loop-gate bank stays clean and now also projects wider-lane corpus replications (sim faithfulness already proven in /tmp/anvil-probe-se4/: assign y[gi*8 +: 8] = b ≡ {4{b}} under iverilog). .8b impl shape recorded (the two surgical edits + the lib proofs + the gate confirmation + the book/USER_GUIDE caveat replacement); pre-split .8b only if it grows beyond a clean single slice.`
  Verification: `done`
  Commit: `done`

- ID: `STRUCTURED-EMISSION-EXPANSION.8b`
  Status: `done`
  Goal: `Implement the .8a design: relax src/ir/generate_loop.rs gate_qualifies to LW >= 1 (width == N*LW); extend src/emit/sv.rs generate_loop_gate + render_generate_loop_block with the LW>1 part-select body (LW==1 unchanged); lib proofs (wider-lane mark; wider-lane emit shape; LW==1 byte-identical; identity/node-count untouched; an emit/sim faithfulness check); prove the wider lane exercised + downstream-clean in tool_matrix --generate-loop-gate (Verilator + both Yosys modes + Icarus); book/USER_GUIDE closeout (replace the "wider lane stays inline" caveat with the shipped wider-lane surface). Default-off / DUT byte-identical (snapshots untouched; no new knob / no schema bump). Pre-split further (.8b.1 live + .8b.2 gate/docs) if warranted when picked.`
  Acceptance: `cargo check/clippy(-D warnings)/fmt clean; cargo test --lib green incl. new wider-lane proofs + the 1-bit-lane proofs unchanged; cargo test --test snapshots 6/6 byte-identical (default-off); the --generate-loop-gate bank stays clean and proves a wider-lane loop emitted + accepted; book/USER_GUIDE updated; committed through COMMIT.md with the leaf id.`
  Result: `Done — the fourth structured surface (the wider-lane generate for part-select) is delivered end-to-end as a single clean slice (no .8b.N pre-split needed). TWO surgical source edits: (1) src/ir/generate_loop.rs gate_qualifies relaxed — lane.width() != 1 || *width != operands.len() replaced with lw = lane.width() as usize; lw == 0 || *width != operands.len() * lw (any LW >= 1, width == N*LW; function_emit/soft_union + all-same-operand/N>=2 exclusions unchanged) + module/predicate doc updates; (2) src/emit/sv.rs — generate_loop_gate defensive re-check mirrored to the same LW >= 1 / width == N*LW condition (still returns (lane, N)); render_generate_loop_block computes lw = m.nodes[lane as usize].width() and branches: lw == 1 keeps the verbatim assign <name>[gi] = <x>; (shipped 1-bit surface byte-identical), lw > 1 emits assign <name>[gi*LW +: LW] = <x>; + doc updates. NO config/metrics/introspect change — reuses generate_loop_emit_prob + num_emitted_generate_loops (no new knob / no schema bump). 4 generate_loop test changes: wide_lane_replication_does_not_qualify -> wide_lane_replication_qualifies (marked==1), new mismatched_result_width_replication_does_not_qualify (width != N*LW rejected), new module_wide_replication helper + marked_wide_lane_gate_emits_part_select_loop (a {3{lane}} 4-bit lane renders [gi*4 +: 4] + suppresses inline + the 1-bit [gi] body is absent), new marked_one_bit_lane_keeps_index_body_byte_identical ([gi] kept, no +:). Book: book/src/structured-emission.md second-surface "What gets wrapped" rewritten to LW >= 1 + a new "## The fourth surface: wider lanes via a part-select" section with a BYTE-VERIFIED seed-74 before/after ({2{i_2}} 2-bit lane -> [gi*2 +: 2], fully -Wall clean) + reproduce recipe; knobs.md/USER_GUIDE/README generate_loop_emit_prob entries + the --generate-loop-gate description + CODEBASE_ANALYSIS generate_loop.rs block + the KM card generate-loop-emit (the "excluded wider lane" framing replaced with the shipped fourth surface). DOWNSTREAM: a forced generate_loop_emit_prob=1.0 per-seed ON-vs-OFF sweep over 8 single seeds with wider-lane replications (/tmp/anvil-gl8b/) emits 9 wider-lane part-selects (e.g. [gi*14 +: 14], [gi*16 +: 16]) all Verilator -Wall delta=0 vs OFF + Yosys both modes + Icarus rc=0/0-warnings; the existing tool_matrix --generate-loop-gate bank stays regression-clean (/tmp/anvil-generate-loop-gate-8b: 3 scenarios / 12 modules / coverage_gaps=[] / saw_generate_loop_emit / 12/0 Verilator + both Yosys + Icarus). Default-off / DUT byte-identical (snapshots 6/6). With .8b the fourth surface is delivered end-to-end; .8 / .8a / .8b all close; the lane returns to no-current-frontier (open-ended). Nothing retired.`
  Verification: `cargo check --all-targets clean; cargo clippy --all-targets -- -D warnings clean; cargo fmt --all --check clean; cargo test --lib 493 passed / 2 ignored (incl. the 4 changed/new generate_loop proofs: wide_lane_replication_qualifies, mismatched_result_width_replication_does_not_qualify, marked_wide_lane_gate_emits_part_select_loop, marked_one_bit_lane_keeps_index_body_byte_identical); cargo test --test snapshots 6/6 byte-identical (default-off). Forced per-seed ON-vs-OFF wider-lane downstream sweep (/tmp/anvil-gl8b/, 8 seeds 58/94/97/110/118/126/147/148): 9 wider-lane part-selects emitted, Verilator -Wall delta=0 on every seed, Yosys without-abc + with-abc + Icarus rc=0/0-warnings. Existing gate /tmp/anvil-generate-loop-gate-8b regression-clean (12/0 all tools, coverage_gaps=[], saw_generate_loop_emit=true). mdbook build book clean; bash knowledge-map/scripts/gen_knowledge_map.sh (42 facts / 341 keys) + check_knowledge_map.sh OK; cargo test --test book_examples 3/3 (the new fourth-surface repro block skip-sentinelled). Book before/after byte-verified vs the release binary (seed 74, generate_loop_emit_prob 0.0 vs 1.0 diff = exactly the {2{i_2}} assign becoming the genvar/generate-for + [gi*2 +: 2] body).`
  Commit: `done`

## Current Frontier

**No current frontier.** The tree stays `active` as an **open-ended capability
lane** (richer structured emission, ROADMAP steering gap 1). **Four** structured
surfaces are now delivered end-to-end: the combinational `function automatic`
(`.1`+`.2`), the `generate for` loop (`.3`+`.4`), the combinational
`task automatic` (`.5`+`.6`), and the **wider-lane `generate for` part-select**
(`.7` design + `.8` impl, closed `2026-06-17` by `.8b`, decision `0015`). Future
surfaces — nested/multi-level `generate`, `interface` / `modport`, and richer
(multi-output) tasks — are `.9+`, each its own decision when picked (none
retired). When PNT next selects this lane, open `.9` with a design/decision leaf
naming the next surface.

_No active leaves — the lane has no current frontier. The most recent completions:_

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| — | `STRUCTURED-EMISSION-EXPANSION.8b` | `done` | Impl of the fourth surface (the wider-lane `generate for` part-select): relaxed `src/ir/generate_loop.rs` `gate_qualifies` to `LW >= 1` (`width == N*LW`) + the `src/emit/sv.rs` `render_generate_loop_block` `[gi*LW +: LW]` branch (`LW==1` `[gi]` kept byte-identical) + 4 lib proofs (`wide_lane_replication_qualifies`, `mismatched_result_width_replication_does_not_qualify`, `marked_wide_lane_gate_emits_part_select_loop`, `marked_one_bit_lane_keeps_index_body_byte_identical`) + book/knobs/USER_GUIDE/README/CODEBASE_ANALYSIS/KM closeout. Downstream-clean: a per-seed ON-vs-OFF sweep (8 seeds) emits 9 wider-lane part-selects with Verilator `-Wall` Δ=0 + Yosys both + Icarus rc=0; `--generate-loop-gate` regression-clean (12/0). Reuses `generate_loop_emit_prob` + `num_emitted_generate_loops` (no new knob / no schema bump). `cargo test --lib` 493; snapshots 6/6 byte-identical. Closes `.8b` / `.8` — the fourth structured surface delivered end-to-end. |
| — | `STRUCTURED-EMISSION-EXPANSION.8a` | `done` | Design-detail (no source): resolved decision `0015`'s open questions against the real `generate_loop.rs` `gate_qualifies` + `sv.rs` `generate_loop_gate`/`render_generate_loop_block` — keep `generate_loop_gate` returning `(lane, N)` (recompute `LW` in the renderer); branch `LW==1` (verbatim `[gi]`) vs `LW>1` (`[gi*LW +: LW]`); relax the predicate to `LW >= 1` / `width == N*LW` (exclusions unchanged); byte-identity contract for the shipped 1-bit surface; downstream proof = a deterministic lib emit-test + the existing `--generate-loop-gate`. **Corpus-liveness proven**: a 300-module sweep emits 20 multi-bit-lane replications (of 448) — the surface fires on real generation, not hand-built-only. No new knob / no schema bump. |
| — | `STRUCTURED-EMISSION-EXPANSION.7` | `done` | Decision `0015`: picked the FOURTH structured surface — the **wider-lane `generate for` part-select** (broaden the `generate for` lane from 1-bit to `LW >= 1`, rendering `assign <wire>[gi*LW +: LW] = <x>;`, closing the recorded wider-lane follow-up). A fresh empirical probe (Verilator 5.046 `-Wall` + Yosys 0.64 both modes + Icarus 13.0) accepts it warning-clean + iverilog-sim-proves it `== {N{x}}`; `interface`/`modport` empirically DISQUALIFIED (Icarus syntax-fail + both-Yosys implicit-decl warn); nested-generate recorded clean-but-bigger-blast-radius. Reuses the existing `generate_loop_emit_prob` knob + `num_emitted_generate_loops` metric (no new knob / no schema bump). Split `.7` (design) + `.8` (impl, pre-split `.8a`/`.8b`) + future `.9+`. No source change. |
| — | `STRUCTURED-EMISSION-EXPANSION.6b.3` | `done` | The user-facing closeout: a `## The third surface: a combinational task automatic` section in `book/src/structured-emission.md` (byte-verified seed-1 before/after — the inline `shr_0 = i_2 >> 2'h3` becomes the `task automatic`/`always_comb`/passthrough form; the function-surface candidate parallel; the output-var passthrough; the four-way mutual exclusion; the `gi`-free single-gate rule; metric @ schema `1.10` + gate) + the `task_emit_prob` knob entry in `book/src/knobs.md` / `USER_GUIDE.md` / README "Current CLI truth" (config-file-only knob) + the KM how-to card `combinational-task-emit` (KM 40→41 facts / 318→331 keys). Docs-only / DUT byte-identical. `mdbook build` + `check_knowledge_map` + `check_memory_architecture` + `cargo test --test book_examples` 3/3 green. Closes `.6b.3` / `.6b` / `.6` — the third structured surface delivered end-to-end. |
| — | `STRUCTURED-EMISSION-EXPANSION.6b.2b` | `done` | The repo-owned `tool_matrix --task-emit-gate`: `--task-emit-gate` flag + `ScenarioSet::TaskEmitSweep` + `build_task_emit_sweep_scenarios`/`task_emit_focus_config` (one comb-only `task_emit_prob=1.0` DUT × three construction strategies) + `ModuleReport.emitted_combinational_task` SV-text detection (`"task automatic"`) + `saw_combinational_task_emit` coverage fact + `MatrixReport.task_emit_gate` + early-return gap enforcement + 5 proofs + 6 fixture updates + the `test_cli` default. Banked clean `/tmp/anvil-task-emit-gate-r1` (3 scenarios / 12 modules / 12 emitting a task / `coverage_gaps = []` / `12/0` Verilator + both Yosys + Icarus compile). README + USER_GUIDE + CODEBASE_ANALYSIS gate entries. Templated on `--generate-loop-gate`. Default-off / DUT byte-identical (snapshots 6/6); `cargo test --bin tool_matrix` 68. |
| — | `STRUCTURED-EMISSION-EXPANSION.6b.2a` | `done` | The metric + schema bump: `Metrics::num_emitted_combinational_tasks` (`= m.task_emit_gates.len()`, `#[serde(default)]`) surfaced in introspection `module_metrics` ⇒ schema MINOR bump `1.9 → 1.10` (the metric bumps; the `.6b.1` knob rode the version). MINOR is an integer ⇒ `1.9 → 1.10` (ten), not a decimal. Bumped all current-output schema refs (9 assertions + schema doc + README + USER_GUIDE + 5 book example JSONs + the CODEBASE_ANALYSIS envelope line); historical landing attributions left intact. Lib proof; `cargo test --lib` 490 + snapshots 6/6 + mdbook green; end-to-end introspect default `0` / forced `39`. Precedented (`1.8→1.9` `num_emitted_generate_loops`). |
| — | `STRUCTURED-EMISSION-EXPANSION.6b.1` | `done` | Live surface delivered: `Config::task_emit_prob` + `Module.task_emit_gates` + new `src/ir/task_emit.rs` (`annotate_task_emit_gates`, the function-emit candidate predicate **plus** exclusion of the three sibling projections) + the two guarded gen-time call-site rolls (after generate_loop) + the `to_sv_with_modules` `task_emit_gate` accessor + `render_gate_task_decl` (body via the reused `render_gate_function_body`) + `render_gate_task_call` (the `logic <wire>__tv` var + the `always_comb <wire>__t(...)` call) + the gate-assign-loop passthrough `assign <wire> = <wire>__tv;` + 11 lib proofs. Output-var + passthrough integration (the `.6a` first cut). No schema bump (default-off prob-knob precedent; the `.6b.2` metric bumps `1.9→1.10`). Default-off / DUT byte-identical (snapshots 6/6; lib 489); forced `task_emit_prob=1.0` sweep clean across Verilator `--lint-only` (`-Wall` Δ=0 vs OFF) + both Yosys + Icarus (`/tmp/anvil-te-r1/`, 5 seeds, 4–39 tasks each). |
| — | `STRUCTURED-EMISSION-EXPANSION.6a` | `done` | Design-detail (no source): grounded decision `0014` in the real emitter (the `to_sv_with_modules` function-decl + generate-block sections as the template; the per-gate assign-loop `continue` pattern; the **reuse of `render_gate_function_body` verbatim** as the task body) + the `function_emit.rs`/`generate_loop.rs` gen-time-annotation chain. Pinned all five points: (1) the **output-var + passthrough-`assign`** integration (keep `<wire>` a net, add `logic <wire>__tv`, `always_comb <wire>__t(<wire>__tv, …)`, swap the gate's assign RHS to `<wire>__tv` — only the gate's own drive changes; `<wire>`-as-var rejected for the first cut); (2) gen-time `src/ir/task_emit.rs` `annotate_task_emit_gates` + `Module.task_emit_gates`, the function-emit predicate plus exclusion of the three sibling projections, run after generate_loop; (3) the `task automatic` decl + `always_comb` call + assign-RHS swap; (4) `Config::task_emit_prob` (config-file-only, default `0.0`, byte-identical); (5) `num_emitted_combinational_tasks` metric (schema `1.9→1.10`) + `tool_matrix --task-emit-gate` / `saw_combinational_task_emit`. `.6b` impl shape recorded. |
| — | `STRUCTURED-EMISSION-EXPANSION.5` | `done` | Decision `0014`: picked the third surface — a default-off, valid-by-construction combinational `task automatic` emit-projection of a single combinational gate (the decision `0012` single-gate parallel, but a procedural `task` with an `output` arg called from `always_comb`), over nested `generate` and `interface`/`modport`. `task` was the recorded leading candidate (decision `0013`). Empirically grounded this session: a combinational `task` called from `always_comb` is clean across Verilator `-Wall` + both Yosys + Icarus, in both the direct-output and the output-var passthrough forms. Discipline, opt-in `task_emit_prob`, `saw_combinational_task_emit` gate. Split `.5`/`.6`/`.7+`. No source change. |
| — | `STRUCTURED-EMISSION-EXPANSION.4b.3` | `done` | The user-facing closeout of the SECOND surface: a `## The second surface: a generate for loop` section in `book/src/structured-emission.md` (byte-verified seed-12 before/after — the inline `{5{slice_0}}` becomes the `genvar`/`generate for` block; the `{N{x}}` 1-bit-lane rule; the wider-lane part-select exclusion; the `function_emit` mutual exclusion; the `gi = gi + 1` form; metric + gate) + the `generate_loop_emit_prob` knob entry in `book/src/knobs.md` (the `### Structured emission` subsection), `USER_GUIDE.md`, and the README "Current CLI truth" (config-file-only knob) + the Knowledge Map how-to card `generate-loop-emit` (KM 38→39 facts / 296→309 keys). Docs-only / DUT byte-identical. `mdbook build` + `check_knowledge_map` + `check_memory_architecture` + `cargo test --test book_examples` 3/3 green. |
| — | `STRUCTURED-EMISSION-EXPANSION.4b.2b` | `done` | The repo-owned `tool_matrix --generate-loop-gate`: `ScenarioSet::GenerateLoopSweep` + `build_generate_loop_sweep_scenarios` (one comb-only `generate_loop_emit_prob=1.0` DUT × three construction strategies) + `ModuleReport.emitted_generate_loop` SV-text detection + `saw_generate_loop_emit` coverage fact + early-return gap enforcement + 5 cargo-portable proofs + 6 fixture updates. Banked clean `/tmp/anvil-generate-loop-gate-r1` (3 scenarios / 12 modules / 8 emitting a loop / `coverage_gaps = []` / `12/0` Verilator + both Yosys + Icarus compile). Templated on `--function-emit-gate`. Default-off / DUT byte-identical (snapshots 6/6). |
| — | `STRUCTURED-EMISSION-EXPANSION.4b.2a` | `done` | The metric `Metrics::num_emitted_generate_loops` (`= m.generate_loop_gates.len()`) surfaced in introspection `module_metrics` ⇒ schema MINOR bump `1.8 → 1.9`. Lib proof; `cargo test --lib` 478 + snapshots 6/6 + mdbook all green; end-to-end introspect default `0` / forced `50`. Precedented (`1.7→1.8` `num_emitted_combinational_functions`). Bumped all current-output schema refs (9 test assertions + schema doc + README + USER_GUIDE + 5 book example JSONs + the CODEBASE_ANALYSIS envelope line); historical landing attributions left intact. |
| — | `STRUCTURED-EMISSION-EXPANSION.4b.1` | `done` | Live surface delivered: `Config::generate_loop_emit_prob` + `Module.generate_loop_gates` + new `src/ir/generate_loop.rs` (`annotate_generate_loop_gates`, the `{N{x}}` 1-bit-lane replication candidate predicate excluding function-emit marks) + the two guarded gen-time call-site rolls (after function_emit) + the `to_sv_with_modules` `generate_loop_gate` accessor + `render_generate_loop_block` + the generate-block section + the assign-loop inline-replication suppression + 9 lib proofs. Increment form `gi = gi + 1` (the portable form; `gi++` not retired). No schema bump (default-off prob-knob precedent). Default-off / DUT byte-identical (snapshots 6/6; lib 477); forced `generate_loop_emit_prob=1.0` sweep clean across Verilator `--lint-only` (`-Wall` Δ=0 vs OFF) + both Yosys + Icarus (`/tmp/anvil-gl-r1/`, 5 seeds, 62–168 loops each). |
| — | `STRUCTURED-EMISSION-EXPANSION.4a` | `done` | Design-detail (no source): grounded decision `0013` in the real emitter (`render_gate`'s `Concat` replication predicate at `sv.rs:1159` — `operands.len() >= 2 && all-same-NodeId ⇒ {N{x}}`; the `to_sv_with_modules` function-decl-section template) + the `function_emit.rs`/`soft_union.rs` gen-time-annotation precedent + `src/config.rs`/`src/gen/mod.rs`. Pinned all five points: (1) selection = a `{N{x}}` replication `Concat` with a **1-bit lane** (⇒ `W == N`, `assign <wire>[gi] = <x>` byte-faithful), mutually exclusive with function-emit (excludes `m.function_emit_gates`, run after function_emit); (2) gen-time `annotate_generate_loop_gates` + `Module.generate_loop_gates`; (3) the `genvar <wire>__gi` / `generate for` rendering + the assign-loop `continue` suppression; (4) `Config::generate_loop_emit_prob` (default `0.0`, config-file-only, byte-identical); (5) `tool_matrix --generate-loop-gate` / `saw_generate_loop_emit` (full Verilator + both Yosys plan). Flagged the gate-shape risk (the corpus must emit `{N{x}}` 1-bit replications — the one-hot mux-mask idiom). `.4b` impl shape recorded. |
| — | `STRUCTURED-EMISSION-EXPANSION.3` | `done` | Decision `0013`: picked the second surface — a default-off, valid-by-construction `generate for` loop emit-projection of an existing `{N{x}}` replication (over `task` [leading future], `interface`/`modport`, and `generate if`), with its discipline, opt-in `generate_loop_emit_prob`, and downstream gate. Empirically grounded (Verilator `-Wall` + both Yosys + Icarus accept `generate for` clean; the DUT emitter has no generate today; the frontend lane has `generate if`). Split `.3`/`.4`/future. No source change. |
| — | `STRUCTURED-EMISSION-EXPANSION.2b.2c` | `done` | The user-facing closeout of the FIRST surface: a new `How It Works` book chapter `book/src/structured-emission.md` (byte-verified seed-42 before/after; single-gate rule; `Slice`/structured exclusions; duplicate-operand positional params; combinational-only; why-first rationale; metric + gate) + the `function_emit_prob` knob entry in `book/src/knobs.md` (new `### Structured emission` subsection), `USER_GUIDE.md`, and the README "Current CLI truth" (config-file-only knob) + the Knowledge Map how-to card `combinational-function-emit`. Docs-only / DUT byte-identical. `mdbook build` + `check_knowledge_map` + `check_memory_architecture` + `cargo test --test book_examples` 3/3 green. |
| — | `STRUCTURED-EMISSION-EXPANSION.2b.2b` | `done` | The repo-owned `tool_matrix --function-emit-gate`: `ScenarioSet::FunctionEmitSweep` + `build_function_emit_sweep_scenarios` (one comb-only `function_emit_prob=1.0` DUT × three construction strategies) + `ModuleReport.emitted_combinational_function` SV-text detection + `saw_combinational_function_emit` coverage fact + early-return gap enforcement + 5 cargo-portable proofs. Banked clean `/tmp/anvil-function-emit-gate-r1` (3 scenarios / 12 modules / 608 emitted functions / `coverage_gaps = []` / `12/0` Verilator + both Yosys + Icarus compile). Templated on `--signoff-knob-sweep-gate` + the soft_union detection precedent. Default-off / DUT byte-identical (snapshots 6/6). |
| — | `STRUCTURED-EMISSION-EXPANSION.2b.2a` | `done` | The metric `Metrics::num_emitted_combinational_functions` (= `m.function_emit_gates.len()`) surfaced in introspection `module_metrics` ⇒ schema MINOR bump `1.7 -> 1.8`. Lib proof; 468 lib tests + snapshots 6/6 + mdbook all green; end-to-end introspect default `0` / forced `1256`. Precedented (1.0->1.1 `bisimulation_flops_merged`). |
| — | `STRUCTURED-EMISSION-EXPANSION.2b.1` | `done` | Live surface delivered: `Config::function_emit_prob` + `Module.function_emit_gates` + `src/ir/function_emit.rs` (`annotate_function_emit_gates`) + the gen-time call-site rolls + the `to_sv_with_modules` `<wire>__f` `function automatic` decl/positional-body/call rendering + 9 lib proofs + a forced-knob downstream sweep. **`Slice` excluded** (a bit-select uses only a sub-range ⇒ `-Wall UNUSEDSIGNAL` on a full-width param; still emitted inline, nothing retired). No schema bump (default-off prob-knob precedent). Default-off / DUT byte-identical (snapshots 6/6). |
| — | `STRUCTURED-EMISSION-EXPANSION.2a` | `done` | Design-detail (no source): pinned the first-cut single-gate "operand function" (minimal cone ⇒ zero sharing hazard), the gen-time annotation (`Module.function_emit_gates` + `annotate_function_emit_gates`, the `soft_union.rs` precedent), the `function automatic` signature/positional-body/call rendering, the `function_emit_prob` knob, and the downstream gate. Pre-split `.2b` → `.2b.1`/`.2b.2`. |
| — | `STRUCTURED-EMISSION-EXPANSION.1` | `done` | Decision `0012`: picked the combinational `function automatic` emit-projection as the first surface (over interface/modport + nested generate), with its valid-by-construction discipline, opt-in `function_emit_prob`, and downstream gate. Split `.1`/`.2`/future. No source change. |

## Decisions

- `2026-06-17` (`.7`, decision [`0015`](../decisions/0015-structured-emission-fourth-surface-wide-lane-generate-loop.md)):
  picked the **fourth** richer-structured surface autonomously at a
  no-active-frontier boundary (`feedback_pick_and_roll_at_no_frontier`). It is a
  default-off, opt-in, **valid-by-construction wider-lane `generate for`
  part-select** — a behaviour-preserving broadening of the second surface
  (decision `0013`) from the 1-bit lane to a lane of any width `LW >= 1`: a marked
  `{N{x}}` replication whose lane is `LW` bits (result `N*LW`) renders
  `generate for (gi=0; gi<N; gi=gi+1) assign <wire>[gi*LW +: LW] = <x>;`, while
  the `LW==1` case keeps the existing `assign <wire>[gi] = <x>;` verbatim (so the
  shipped 1-bit surface stays byte-identical). Bit-group `g` of `{N{x}}` is
  exactly the lane ⇒ byte-equivalent to the inline replication. Chosen over
  `interface`/`modport` (**empirically disqualified this session**: Icarus
  syntax-fails the modport port + both Yosys modes warn on the implicit
  interface-member decl — confirms the recorded weak-support claim), nested/
  multi-level `generate` (clean but bigger blast radius + no routine
  by-construction 2D source), and constant-predicate `generate if` (dead untaken
  branch; the frontend lane already exercises it). Empirically grounded this
  session: a wider-lane `generate for` part-select is accepted warning-clean by
  Verilator 5.046 `-Wall` + both repo Yosys modes + Icarus `iverilog -g2012`,
  and iverilog simulation proves the unrolled loop bit-equal to `{4{b}}`.
  Discipline: rules-first (broaden the `annotate_generate_loop_gates` predicate;
  never generate-then-filter); **reuses** the existing `generate_loop_emit_prob`
  knob (default `0.0` ⇒ byte-identical / snapshots untouched) and the
  `num_emitted_generate_loops` metric — **no new knob, no new metric, no
  introspection schema bump**; no new IR node / no new computed truth (the
  emit-projection precedent). Downstream gate: the existing
  `tool_matrix --generate-loop-gate` covers wider lanes once the predicate is
  relaxed; `.8` adds a focused wider-lane assertion. Split `.7` (done) + `.8`
  (impl; pre-split `.8a` design-detail + `.8b` impl) + future (`.9+`:
  nested/multi-level `generate`, `interface`/`modport`, richer tasks).
- `2026-06-16` (`.3`, decision [`0013`](../decisions/0013-structured-emission-second-surface-generate-loop.md)):
  picked the **second** richer-structured surface by explicit owner steer
  (*"structured emission: next surface"* → `generate`). It is a default-off,
  opt-in, **valid-by-construction `generate for` loop** emitted as a
  behaviour-preserving projection of an existing **replication** (leading source =
  a `{N{x}}` `Concat`, index-regular by construction, rendered as a single-level
  `generate for (genvar gi …) assign <wire>[gi] = <x>;` that unrolls to exactly
  the inline replication). Chosen over `task` (also clean for *simple combinational
  void* tasks on the current toolchain — so `0012`'s "weak task synth" is precisely
  a multi-output/side-effecting caution; `task` is the **leading future** candidate,
  `.5+`, not retired), `interface`/`modport` (still weak/inconsistent Yosys synth),
  and a constant-predicate `generate if` (dead untaken branch; the frontend lane
  already exercises it). Empirically grounded this session: the DUT emitter has no
  `generate`/`genvar` today; the frontend lane has `generate if`; and a
  representative `generate for` + a replication→loop projection are accepted
  warning-clean by Verilator 5.046 `-Wall` + both repo Yosys modes + Icarus.
  Discipline: rules-first (mark an already-valid replication node; never
  generate-then-filter), default-off `generate_loop_emit_prob` (proposed; default
  `0.0`) ⇒ byte-identical / snapshots untouched, no new IR node / no new whole-module
  behaviour (the `soft_union`/aggregate/`function_emit` precedent). Downstream gate:
  Verilator + both Yosys modes + Icarus accept the loops warning-clean, gated on a
  `saw_generate_loop_emit` fact. Split `.3` (done) + `.4` (impl; pre-split `.4a`
  design-detail + `.4b` impl) + future (`.5+`: `task`, nested/multi-level `generate`,
  `interface`/`modport`).
- `2026-06-16` (`.1`, decision [`0012`](../decisions/0012-structured-emission-first-surface-combinational-function.md)):
  activated the lane by explicit owner directive. The **first** richer-structured
  surface is a default-off, opt-in, **valid-by-construction combinational
  `function automatic`** emitted as a behaviour-preserving projection of an
  existing combinational cone (a `Gate` node + its fan-in, stopping at the
  `output_support` support-leaf boundary; the cone's support leaves are the
  function's parameter list; the body is the straight-line evaluation of the cone's
  internal gates, returning the root; the use site becomes a call). Chosen over
  `interface`/`modport` (weak/version-inconsistent Yosys synthesis ⇒ fails the
  both-Yosys-modes-clean bar) and nested `generate` (bigger emitter blast radius)
  and `task` (procedural/multi-output — a combinational function is the simpler
  first cut). Discipline: rules-first (no generate-then-filter; selection at
  construction time), default-off `function_emit_prob` (default `0.0`) ⇒
  byte-identical / snapshots untouched, no new IR node / no new computed truth (the
  `soft_union`/aggregate emit-projection precedent). Downstream gate: Verilator +
  both Yosys modes + Icarus accept the functions warning-clean, gated on a
  `saw_combinational_function_emit` fact. Split `.1` (done) + `.2` (impl) + future;
  pre-split `.2` → `.2a` (design-detail) + `.2b` (impl).
- `2026-06-15`: Registered `proposed` by owner roadmap steering as a named future
  capability lane. Not started; `SV-VERSION-TARGETING` was activated first.

## Open Questions

- Which structured surface is highest-leverage first (function/task vs
  interface/modport vs nested generate) — resolved by `.1` (function, decision
  `0012`).
- Which structured surface is next after the function — resolved by `.3`
  (`generate for`, decision `0013`, owner steer).
- The exact `generate for` index-regular source (`{N{x}}` replication leading) +
  selection mechanism (gen-time annotation vs emit-time) + genvar/loop rendering
  + the exact knob name — **resolved by `.4a`** (design-detail): first-cut source =
  a `{N{x}}` replication `Concat` with a **1-bit lane** (`render_gate`'s existing
  replication predicate, `sv.rs:1159`); **gen-time annotation**
  (`Module.generate_loop_gates` + `src/ir/generate_loop.rs`); a `genvar <wire>__gi`
  / `generate for` block + assign-loop `continue` suppression; knob
  `generate_loop_emit_prob` (config-file-only, default `0.0`); gate
  `tool_matrix --generate-loop-gate` / `saw_generate_loop_emit`.
- (`.4b`) Does the forced `generate_loop_emit_prob=1.0` comb-only gate corpus
  actually emit `{N{x}}` 1-bit replications (the one-hot mux-mask broadcast idiom)
  so the loops fire? Pinned as the load-bearing gate-shape risk at `.4a`; resolved
  at `.4b` by the banked forced-sweep evidence — `.4b.2b`'s
  `/tmp/anvil-generate-loop-gate-r1` shows 8/12 modules emitting a loop, fact lit.
- Which structured surface is next after the `generate for` loop — resolved by
  `.5` (`task`, decision `0014`; the recorded leading candidate, autonomously
  selected at a no-frontier boundary per the owner *"pick any tree and roll"*
  directive).
- (`.6a`) The exact `task` net-vs-var integration (the output-var +
  passthrough-`assign` form vs making `<wire>` itself the `always_comb` var) +
  one-`always_comb`-per-call vs shared + the selection mechanism (gen-time
  annotation vs emit-time) + the exact knob name — deferred to `.6a`
  (design-detail) per decision `0014`. Both integration forms probed clean.

## Blockers

- None (not active by choice, not dependency).

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-06-17` | `STRUCTURED-EMISSION-EXPANSION.8b` | **Live emitter change** (`src/ir/generate_loop.rs` `gate_qualifies` relaxed to `LW >= 1` / `width == N*LW` + module/predicate doc; `src/emit/sv.rs` `generate_loop_gate` defensive re-check mirrored + `render_generate_loop_block` `lw==1` `[gi]` vs `lw>1` `[gi*LW +: LW]` branch + doc; 4 generate_loop test changes; `book/src/structured-emission.md` + `book/src/knobs.md` + `USER_GUIDE.md` + `README.md` + `CODEBASE_ANALYSIS.md` + `docs/knowledge/generate-loop-emit.md` updated). `cargo check --all-targets` clean; `cargo clippy --all-targets -- -D warnings` clean; `cargo fmt --all --check` clean; `cargo test --lib` **493 passed** / 2 ignored (incl. `wide_lane_replication_qualifies`, `mismatched_result_width_replication_does_not_qualify`, `marked_wide_lane_gate_emits_part_select_loop`, `marked_one_bit_lane_keeps_index_body_byte_identical`); `cargo test --test snapshots` **6/6 byte-identical** (default-off — the wider-lane branch is reached only when the knob is on). **Forced per-seed ON-vs-OFF wider-lane downstream sweep** (`/tmp/anvil-gl8b/`, 8 seeds 58/94/97/110/118/126/147/148 each with a wider-lane replication): **9 wider-lane part-selects emitted** (e.g. `[gi*14 +: 14]`, `[gi*16 +: 16]`), Verilator `-Wall` **delta = 0** on every seed, Yosys without-abc + with-abc + Icarus `iverilog -g2012` **rc=0 / 0 warnings**. Existing gate `/tmp/anvil-generate-loop-gate-8b` (`--generate-loop-gate --yosys-mode both --iverilog-compile`) **regression-clean**: 3 scenarios / 12 modules / `coverage_gaps = []` / `saw_generate_loop_emit = true` / `12/0` Verilator + both Yosys + Icarus. `mdbook build book` clean; `bash knowledge-map/scripts/gen_knowledge_map.sh` (42 facts / 341 keys) + `check_knowledge_map.sh` **OK**; `cargo test --test book_examples` **3/3** (the new fourth-surface repro block skip-sentinelled). Book before/after byte-verified vs the release binary (seed 74: `{2{i_2}}` → the genvar/generate-for + `[gi*2 +: 2]` body, fully `-Wall` clean). No new knob / no new metric / no schema bump. | `done` |
| `2026-06-17` | `STRUCTURED-EMISSION-EXPANSION.8a` | **Design-detail leaf, no source change** (a `DEVELOPMENT_NOTES.md` design-detail entry; no `src/` touched). Grounded in a fresh read of `src/ir/generate_loop.rs` (`gate_qualifies`) + `src/emit/sv.rs` (`generate_loop_gate` ~1512, `render_generate_loop_block` ~1548) and a **corpus-liveness probe** (`/tmp/anvil-widelane-probe/`: a 300-module comb-only sweep emits 448 `{N{x}}` replications, **20 with a multi-bit lane** — `{2{i_4}}` 7b→14b, `{3{case_mux_0}}` 12b→36b, `{6{i_1}}` 8b→48b, `{4{concat_7}}` 20b→80b — proving the broadened predicate fires on real generation, ~4.5%, not hand-built-only). Resolved every open question (keep `generate_loop_gate -> (lane, N)` + recompute `LW` in the renderer; `LW==1` `[gi]` byte-identical vs `LW>1` `[gi*LW +: LW]` branch; predicate `LW >= 1` / `width == N*LW` with exclusions unchanged; byte-identity contract; downstream proof = deterministic lib emit-test + the existing `--generate-loop-gate`). `bash scripts/check_memory_architecture.sh` ✅; `bash knowledge-map/scripts/gen_knowledge_map.sh` + `check_knowledge_map.sh` ✅ (no card change — `0015` already carries `answers:`). No source touched ⇒ `cargo check/clippy/fmt` unaffected. | `done` |
| `2026-06-17` | `STRUCTURED-EMISSION-EXPANSION.7` | **Design/decision leaf, no source change.** Decision `0015` (`docs/decisions/0015-structured-emission-fourth-surface-wide-lane-generate-loop.md`) + `INDEX.md` row + tree split (`.7` done + `.8` impl pending, pre-split `.8a`/`.8b`). **Fresh empirical tool-acceptance probe** (this session, `/tmp/anvil-probe-se4/`): a wider-lane `generate for` part-select (`assign y[gi*8 +: 8] = b;` ≡ `{4{b}}`) accepted warning-clean by **Verilator 5.046 `-Wall --lint-only`** (with the `DECLFILENAME` filename-artifact suppressed) + **Yosys 0.64 both modes** (`synth -noabc` and `abc -fast; opt -fast; check`) + **Icarus `iverilog -g2012`**, and **iverilog simulation proves it bit-equal to `{4{b}}`** across sampled inputs (`ALL-MATCH`); the same probe **disqualifies `interface`/`modport`** (Icarus syntax-fails the modport-typed port; both Yosys modes warn `Identifier '\p.data'/'\intf.data' is implicitly declared`) and records nested-generate clean-but-bigger-blast-radius + `generate if` clean-but-dead-branch. `bash scripts/check_memory_architecture.sh` ✅ (`0015` indexed); `bash knowledge-map/scripts/gen_knowledge_map.sh` + `check_knowledge_map.sh` ✅ (decision `0015` carries `answers:`); `mdbook build book` ✅. No source touched ⇒ `cargo check/clippy/fmt` unaffected (`cargo check --all-targets` was clean at session start). | `done` |
| `2026-06-16` | `STRUCTURED-EMISSION-EXPANSION.6b.3` | **User-facing closeout, docs-only** (a `## The third surface: a combinational task automatic` section in `book/src/structured-emission.md` + the intro update + the `task_emit_prob` knob entry in `book/src/knobs.md` `### Structured emission` + `USER_GUIDE.md` + README "Current CLI truth" + new KM card `docs/knowledge/combinational-task-emit.md`; no `src/` touched). `mdbook build book` clean (HTML written, no broken-link warnings); `bash knowledge-map/scripts/gen_knowledge_map.sh` (**41 facts / 331 keys**, was 40 / 318) + `bash knowledge-map/scripts/check_knowledge_map.sh` **OK** (facts valid, ids unique, map in sync); `bash scripts/check_memory_architecture.sh` **all invariants hold** (`0014` indexed); `cargo test --test book_examples` **3/3** (`skip_sentinels_have_reasons` + `every_runnable_book_bash_block_succeeds` green — the new repro block correctly skip-sentinelled). Docs-only ⇒ `cargo check/clippy/fmt` unaffected (no source). Byte-verified against the release binary: seed-1 `task_emit_prob` 0.0→1.0 diff = exactly the `shr_0__t` task decl + the `logic shr_0__tv` var + the `always_comb` call + the `assign` rewritten to the passthrough (rest byte-identical); the example lints clean under `verilator --lint-only -Wall` (matching filename) + both Yosys + Icarus. With this leaf `.6b.3`/`.6b`/`.6` all close — the third structured surface is delivered end-to-end; the lane returns to no-active-frontier. | `done` |
| `2026-06-16` | `STRUCTURED-EMISSION-EXPANSION.6b.2b` | **Repo-owned `tool_matrix` gate** (`src/bin/tool_matrix.rs`: `--task-emit-gate` + `ScenarioSet::TaskEmitSweep` + `build_task_emit_sweep_scenarios`/`task_emit_focus_config` + `ModuleReport.emitted_combinational_task` + `saw_combinational_task_emit` + `MatrixReport.task_emit_gate` + merge/early-return-gap + slugs + 5 proofs + 6 fixture updates + the `test_cli` default; README + USER_GUIDE + CODEBASE_ANALYSIS gate entries). `cargo check --bin tool_matrix` clean; `cargo clippy --bin tool_matrix -- -D warnings` clean; `cargo fmt --all --check` clean; `cargo test --bin tool_matrix` **68 passed** / 1 ignored (incl. 5 new gate proofs); `cargo test --test snapshots` **6/6 byte-identical** (harness-only). Repo-owned bank `/tmp/anvil-task-emit-gate-r1` (`--task-emit-gate --yosys-mode both --iverilog-compile`): 3 scenarios / 12 modules / **12 emitting a task** / `coverage_gaps = []` / `saw_combinational_task_emit = true` / Verilator `12/0` / Yosys without-abc `12/0` / Yosys with-abc `12/0` / Icarus compile `12/0`; all 12 modules `emitted_combinational_task = true`. | `done` |
| `2026-06-16` | `STRUCTURED-EMISSION-EXPANSION.6b.2a` | **Metric + schema bump** (`src/metrics.rs` `num_emitted_combinational_tasks` field + `compute()` + a lib proof; `src/introspect/mod.rs` `SCHEMA_VERSION` `1.9→1.10` + its doc comment + 2 `schema_version` assertions; `src/mcp/mod.rs` 7 `schema_version` assertions; `docs/AGENT_INTROSPECTION_SCHEMA.md` `1.9→1.10` changelog entry + the early-example/defines/checklist lines; README `--introspect`+`analyze` current refs; USER_GUIDE `--sv-version --introspect` row; the 5 `book/src/agent-mcp.md` example JSONs; the `CODEBASE_ANALYSIS.md` envelope line). `cargo clippy --all-targets -- -D warnings` clean; `cargo fmt --all --check` clean; `cargo test --lib` **490 passed** / 2 ignored (the new metric proof + all `schema_version` assertions green at `1.10`); `cargo test --test snapshots` **6/6 byte-identical** (default-off; metric changes no RTL). End-to-end `--introspect`: default ⇒ `schema_version "1.10"` + metric `0`; forced `task_emit_prob=1.0` (seed 42) ⇒ `1.10` + `39`. `mdbook build book` OK. MINOR is an integer ⇒ `1.9 → 1.10` (ten), not a decimal (recorded in the doc comment + changelog). Historical "landed at schema X" attributions left intact (`num_emitted_generate_loops` @ 1.9; `num_emitted_combinational_functions` @ 1.8; sv-version @ 1.2; the schema-doc `1.8→1.9` changelog entry). | `done` |
| `2026-06-16` | `STRUCTURED-EMISSION-EXPANSION.6b.1` | **Live emitter change** (`src/config.rs` `task_emit_prob` knob + default + `0.0..=1.0` validation + dump-config; `src/ir/types.rs` `Module.task_emit_gates`; new `src/ir/task_emit.rs` `annotate_task_emit_gates` (the function-emit candidate predicate plus exclusion of the three sibling projections) + `src/ir/mod.rs` registration; `src/gen/mod.rs` two call-site rolls after generate_loop; `src/emit/sv.rs` `task_emit_gate` accessor + the task section (`render_gate_task_decl` reusing `render_gate_function_body` + `render_gate_task_call`) + the gate-assign-loop passthrough; `DEVELOPMENT_NOTES.md` + `CODEBASE_ANALYSIS.md` updated). `cargo check --all-targets` clean; `cargo clippy --all-targets -- -D warnings` clean; `cargo fmt --all --check` clean; `cargo test --lib` **489 passed** / 2 ignored (incl. 11 new `task_emit` proofs; introspect `schema_version` 1.9 + `umbrella` DUT-byte-identical still green); `cargo test --test snapshots` **6/6 byte-identical** (default-off). Forced `task_emit_prob=1.0` sweep (5 seeds 1/7/42/100/2024, 4–39 tasks each, banked `/tmp/anvil-te-r1/`): Verilator `--lint-only` **5/5 CLEAN** (repo bar) + **`-Wall` ON-vs-OFF delta = 0** (the task projection adds no new warnings; the transient `DECLFILENAME` during the sweep was a filename≠module-name harness artifact, not a task warning), Yosys without-abc **5/5** + with-abc **5/5**, Icarus `iverilog -g2012` **5/5 CLEAN**. Output-var + passthrough integration (the `.6a` first cut). No schema bump (default-off prob-knob precedent; the `.6b.2` metric bumps `1.9→1.10`). | `done` |
| `2026-06-16` | `STRUCTURED-EMISSION-EXPANSION.6a` | **Design-detail leaf, no source change** (a `DEVELOPMENT_NOTES.md` design-detail entry + the `.6` tree split; no `src/` touched). Grounded in a fresh read of `src/emit/sv.rs` (the `to_sv_with_modules` function-decl + generate-block sections as the structural template; the per-gate assign-loop `continue` pattern; the **reuse of `render_gate_function_body` verbatim** as the task body) + `src/ir/function_emit.rs` / `src/ir/generate_loop.rs` (the gen-time-annotation chain + the defensive `*_gate` accessor) + `src/gen/mod.rs` (the guarded call-site rolls) + `src/config.rs` (default + `0.0..=1.0` validation) + `src/ir/mod.rs` (`pub mod` registration). Resolved all five `.6a` points: (1) the output-var + passthrough-`assign` integration (`<wire>`-as-var rejected for the first cut); (2) gen-time `annotate_task_emit_gates` + `Module.task_emit_gates`, the function-emit predicate plus exclusion of the three sibling projections, run after generate_loop; (3) the `task automatic` decl + `always_comb` call + assign-RHS swap; (4) `Config::task_emit_prob` config-file-only default `0.0`; (5) `num_emitted_combinational_tasks` metric (schema `1.9→1.10`) + `tool_matrix --task-emit-gate` / `saw_combinational_task_emit`. Recorded the `.6b` impl shape (pre-split `.6b.1`/`.6b.2`/`.6b.3`). `bash scripts/check_memory_architecture.sh` ✅; `bash knowledge-map/scripts/gen_knowledge_map.sh` + `check_knowledge_map.sh` ✅ (no card change — `0014` already carries `answers:`); `mdbook build book` ✅. No source touched ⇒ `cargo check/clippy/fmt` unaffected. | `done` |
| `2026-06-16` | `STRUCTURED-EMISSION-EXPANSION.5` | **Design/decision leaf, no source change.** Decision `0014` (`docs/decisions/0014-structured-emission-third-surface-combinational-task.md`) + `INDEX.md` row + tree split (`.5` done + `.6` impl pending, pre-split `.6a`/`.6b`). Empirical tool-acceptance grounding (this session): a combinational `task automatic` called from `always_comb` accepted warning-clean by **Verilator 5.046 `-Wall --lint-only`** + **Yosys 0.64 both modes** (`synth -noabc` and `abc -fast; opt -fast; check`) + **Icarus `iverilog -g2012`**, in **both** the direct-output form (the task writes the module output) and the minimal-blast-radius output-var + passthrough-`assign` form (the task writes a local `logic` var, a continuous `assign` drives the gate's net). Confirms decision `0013`'s narrowed `task` caution (simple combinational void tasks are clean; the weakness is multi-output/side-effecting). `bash scripts/check_memory_architecture.sh` ✅ (`0014` indexed); `bash knowledge-map/scripts/gen_knowledge_map.sh` + `check_knowledge_map.sh` ✅ (decision `0014` carries `answers:`); `mdbook build book` ✅. No source touched ⇒ `cargo check/clippy/fmt` unaffected. | `done` |
| `2026-06-16` | `STRUCTURED-EMISSION-EXPANSION.4b.3` | **User-facing closeout, docs-only** (a `## The second surface: a generate for loop` section in `book/src/structured-emission.md` + the intro update + the `generate_loop_emit_prob` knob entry in `book/src/knobs.md` `### Structured emission` + `USER_GUIDE.md` + README "Current CLI truth" + new KM card `docs/knowledge/generate-loop-emit.md`; no `src/` touched). `mdbook build book` clean (HTML written, no broken-link warnings); `bash knowledge-map/scripts/gen_knowledge_map.sh` (**39 facts / 309 keys**, was 38 / 296) + `bash knowledge-map/scripts/check_knowledge_map.sh` **OK** (facts valid, ids unique, map in sync); `bash scripts/check_memory_architecture.sh` **all invariants hold** (`0013` indexed); `cargo test --test book_examples` **3/3** (`skip_sentinels_have_reasons` + `every_runnable_book_bash_block_succeeds` green — the new repro block correctly skip-sentinelled). Docs-only ⇒ `cargo check/clippy/fmt` unaffected (no source). Byte-verified against the release binary: seed-12 `generate_loop_emit_prob` 0.0→1.0 diff = exactly the `{5{slice_0}}` replication becoming the `genvar`/`generate for` block (rest byte-identical); the example lints clean under `verilator --lint-only -Wall` (matching filename) + both Yosys + Icarus. With this leaf `.4b.3`/`.4b`/`.4` all close — the second structured surface is delivered end-to-end. | `done` |
| `2026-06-16` | `STRUCTURED-EMISSION-EXPANSION.4b.2b` | **Repo-owned `tool_matrix` gate** (`src/bin/tool_matrix.rs`: `--generate-loop-gate` + `ScenarioSet::GenerateLoopSweep` + `build_generate_loop_sweep_scenarios`/`generate_loop_focus_config` + `ModuleReport.emitted_generate_loop` + `saw_generate_loop_emit` + `MatrixReport.generate_loop_gate` + merge/early-return-gap + slugs + 5 proofs + 6 fixture updates + the `test_cli` default; README + USER_GUIDE + CODEBASE_ANALYSIS gate entries). `cargo check --bin tool_matrix` clean; `cargo clippy --all-targets -- -D warnings` clean; `cargo fmt --all --check` clean; `cargo test --bin tool_matrix` **63 passed** / 1 ignored (incl. 5 new gate proofs); `cargo test --test snapshots` **6/6 byte-identical** (harness-only). Repo-owned bank `/tmp/anvil-generate-loop-gate-r1` (`--generate-loop-gate --yosys-mode both --iverilog-compile`): 3 scenarios / 12 modules / **8 emitting a generate loop** / `coverage_gaps = []` / `saw_generate_loop_emit = true` / Verilator `12/0` / Yosys without-abc `12/0` / Yosys with-abc `12/0` / Icarus compile `12/0`. | `done` |
| `2026-06-16` | `STRUCTURED-EMISSION-EXPANSION.4b.2a` | **Metric + schema bump** (`src/metrics.rs` `num_emitted_generate_loops` field + `compute()` + a lib proof; `src/introspect/mod.rs` `SCHEMA_VERSION` `1.8→1.9` + its doc comment + 2 `schema_version` assertions; `src/mcp/mod.rs` 7 `schema_version` assertions; `docs/AGENT_INTROSPECTION_SCHEMA.md` `1.8→1.9` changelog entry + the defines/checklist lines; README `--introspect`+`analyze` current refs; USER_GUIDE `--introspect` ref; the 5 `book/src/agent-mcp.md` example JSONs; the `CODEBASE_ANALYSIS.md` envelope line). `cargo clippy --all-targets -- -D warnings` clean; `cargo fmt --all --check` clean; `cargo test --lib` **478 passed** / 2 ignored (the new metric proof + all `schema_version` assertions green at `1.9`); `cargo test --test snapshots` **6/6 byte-identical** (default-off; metric changes no RTL). End-to-end `--introspect`: default ⇒ `schema_version "1.9"` + metric `0`; forced `generate_loop_emit_prob=1.0` ⇒ `1.9` + `50`. `mdbook build book` OK. Historical "landed at schema X" attributions left intact (`num_emitted_combinational_functions` @ 1.8; sv-version @ 1.2). | `done` |
| `2026-06-16` | `STRUCTURED-EMISSION-EXPANSION.4b.1` | **Live emitter change** (`src/config.rs` `generate_loop_emit_prob` knob + `src/ir/types.rs` `Module.generate_loop_gates` + new `src/ir/generate_loop.rs` annotate pass + `src/ir/mod.rs` registration + `src/gen/mod.rs` two call-site rolls after function_emit + `src/emit/sv.rs` `generate for` block rendering + assign-loop suppression; `DEVELOPMENT_NOTES.md` + `CODEBASE_ANALYSIS.md` updated). `cargo check --all-targets` clean; `cargo clippy --all-targets -- -D warnings` clean; `cargo fmt --all --check` clean; `cargo test --lib` **477 passed** / 2 ignored (incl. 9 new `generate_loop` proofs; introspect `schema_version` 1.8 + `umbrella` DUT-byte-identical still green); `cargo test --test snapshots` **6/6 byte-identical** (default-off). Forced `generate_loop_emit_prob=1.0` sweep (5 seeds 1–5, 62–168 loops each, banked `/tmp/anvil-gl-r1/`): Verilator `--lint-only` **5/5 rc=0 / 0 warnings** (repo bar), **`-Wall` ON-vs-OFF delta = 0** (the change adds no new warnings; residual `-Wall UNUSEDSIGNAL` is pre-existing, identical ON and OFF), Yosys without-abc **5/5** + with-abc **5/5**, Icarus `iverilog -g2012` **5/5**. Increment form `gi = gi + 1` (the portable form; `gi++` not retired). No schema bump (default-off prob-knob precedent; the `.4b.2` metric bumps `1.8→1.9`). | `done` |
| `2026-06-16` | `STRUCTURED-EMISSION-EXPANSION.4a` | **Design-detail leaf, no source change** (a `DEVELOPMENT_NOTES.md` design-detail entry + the `.4` tree split; no `src/` touched). Grounded in a fresh read of `src/emit/sv.rs` (`render_gate`'s `Concat` replication predicate at `sv.rs:1159` + the `to_sv_with_modules` function-decl section + `build_names`/`node_ref`/`param_width_decl_w`), `src/ir/function_emit.rs` + `src/ir/soft_union.rs` (the gen-time-annotation precedent + `function_emit_gate` defensive re-check), `src/gen/mod.rs` (the two guarded call-site rolls), `src/config.rs` (`default_function_emit_prob` + the `0.0..=1.0` validation list), `src/ir/mod.rs` (`pub mod` registration). Resolved all five `.4a` points (selection = `{N{x}}` 1-bit-lane replication `Concat` excluding function-emit marks; gen-time `annotate_generate_loop_gates` + `Module.generate_loop_gates`; the `genvar <wire>__gi` / `generate for` rendering + assign-loop `continue` suppression; `Config::generate_loop_emit_prob` config-file-only default `0.0` byte-identical; `tool_matrix --generate-loop-gate` / `saw_generate_loop_emit` full Verilator + both Yosys plan) + flagged the gate-shape replication-availability risk + recorded the `.4b` impl shape. `bash scripts/check_memory_architecture.sh` ✅; `bash knowledge-map/scripts/gen_knowledge_map.sh` + `check_knowledge_map.sh` ✅ (no card change — `0013` already carries `answers:`); `mdbook build book` ✅; `cargo test --test book_examples` 3/3 ✅. No source touched ⇒ `cargo check/clippy/fmt` unaffected. | `done` |
| `2026-06-16` | `STRUCTURED-EMISSION-EXPANSION.3` | **Design/decision leaf, no source change.** Decision `0013` (`docs/decisions/0013-structured-emission-second-surface-generate-loop.md`) + `INDEX.md` row + tree split (`.3` done + `.4` impl pending, pre-split `.4a`/`.4b`). Empirical tool-acceptance grounding (this session): a `generate for` lane unroll + a replication→`generate for` projection accepted warning-clean by **Verilator 5.046 `-Wall --lint-only`** + **Yosys 0.64 both modes** (`synth -noabc` and `abc -fast; opt -fast; check`) + **Icarus `iverilog -g2012`**; a simple combinational void `task` is *also* clean (recorded — `task` is the leading future surface); confirmed the DUT emitter (`src/emit/sv.rs`) has no `generate`/`genvar` today and the frontend lane (`src/frontend/mod.rs`) already emits `generate if`. `bash scripts/check_memory_architecture.sh` ✅ (`0013` indexed); `bash knowledge-map/scripts/gen_knowledge_map.sh` + `check_knowledge_map.sh` ✅ (decision `0013` carries `answers:`); `mdbook build book` ✅. No source touched ⇒ `cargo check/clippy/fmt` unaffected. | `done` |
| `2026-06-16` | `STRUCTURED-EMISSION-EXPANSION.2b.2c` | **User-facing closeout, docs-only** (new `book/src/structured-emission.md` + `book/src/SUMMARY.md` link + `book/src/knobs.md` `### Structured emission` entry + `USER_GUIDE.md` knob section + README "Current CLI truth" bullet + new KM card `docs/knowledge/combinational-function-emit.md`; no `src/` touched). `mdbook build book` clean (HTML written, no broken-link warnings); `bash knowledge-map/scripts/gen_knowledge_map.sh` (**37 facts / 286 keys**, was 36 / 272) + `bash knowledge-map/scripts/check_knowledge_map.sh` **OK** (facts valid, ids unique, map in sync); `bash scripts/check_memory_architecture.sh` **all invariants hold** (`0012` indexed); `cargo test --test book_examples` **3/3** (`skip_sentinels_have_reasons` + `every_runnable_book_bash_block_succeeds` green — the new repro block correctly skip-sentinelled). Docs-only ⇒ `cargo check/clippy/fmt` unaffected (no source). Byte-verified against the release binary: seed-42 `function_emit_prob` 0.0→1.0 diff = exactly the `add_0__f` decl + the one `assign` rewritten to a call (rest byte-identical); the KM reverify recipe emits 10 functions, Verilator `--lint-only` CLEAN. | `done` |
| `2026-06-16` | `STRUCTURED-EMISSION-EXPANSION.2b.2b` | **Repo-owned `tool_matrix` gate** (`src/bin/tool_matrix.rs`: `--function-emit-gate` + `ScenarioSet::FunctionEmitSweep` + `build_function_emit_sweep_scenarios`/`function_emit_focus_config` + `ModuleReport.emitted_combinational_function` + `saw_combinational_function_emit` + merge/early-return-gap + 5 proofs + 6 fixture updates). `cargo check --bin tool_matrix` clean; `cargo clippy --all-targets -- -D warnings` clean (fixed a `clippy::explicit_counter_loop` via `.enumerate()`); `cargo fmt --all --check` clean; `cargo test --bin tool_matrix` **58 passed** / 1 ignored (incl. 5 new gate proofs); `cargo test --lib` **468 passed** / 2 ignored (unchanged); `cargo test --test snapshots` **6/6 byte-identical**. Repo-owned bank `/tmp/anvil-function-emit-gate-r1` (`--function-emit-gate --yosys-mode both --iverilog-compile`): 3 scenarios / 12 modules / **608 emitted functions** / `coverage_gaps = []` / `saw_combinational_function_emit = true` / Verilator `12/0` / Yosys without-abc `12/0` / Yosys with-abc `12/0` / Icarus compile `12/0`. | `done` |
| `2026-06-16` | `STRUCTURED-EMISSION-EXPANSION.2b.2a` | **Metric + schema bump** (`src/metrics.rs` `num_emitted_combinational_functions` + `src/introspect/mod.rs` `SCHEMA_VERSION` `1.7→1.8` + the 9 `schema_version` test assertions + the schema doc + README/USER_GUIDE/book current-output refs + the stale `CODEBASE_ANALYSIS` envelope line). `cargo clippy --all-targets -- -D warnings` clean; `cargo fmt --all --check` clean; `cargo test --lib` **468 passed** / 2 ignored (new metric proof + all `schema_version` assertions green at `1.8`); `cargo test --test snapshots` **6/6 byte-identical** (default-off). End-to-end `--introspect`: default ⇒ `schema_version "1.8"` + metric `0`; forced `function_emit_prob=1.0` ⇒ `1.8` + `1256`. `mdbook build book` OK. | `done` |
| `2026-06-16` | `STRUCTURED-EMISSION-EXPANSION.2b.1` | **Live emitter change** (`src/config.rs` knob + `src/ir/types.rs` `Module.function_emit_gates` + new `src/ir/function_emit.rs` annotate pass + `src/gen/mod.rs` two call-site rolls + `src/emit/sv.rs` `function automatic` decl/body/call rendering). `cargo check --all-targets` clean; `cargo clippy --all-targets -- -D warnings` clean; `cargo fmt --all --check` clean; `cargo test --lib` 467 passed / 2 ignored (incl. 9 new `function_emit` proofs; introspect `schema_version` 1.7 + `umbrella` DUT-byte-identical still green); `cargo test --test snapshots` **6/6 byte-identical** (default-off). Forced `function_emit_prob=1.0` sweep (5 seeds 1/7/42/100/2024, 830–1299 functions each, `/tmp/anvil-fe-r2/`): Verilator `--lint-only` **5/5 CLEAN**, **0** `__f`-param `-Wall` warnings (`Slice` excluded; residual `-Wall UNUSEDSIGNAL` is pre-existing — OFF baseline has 20), Yosys without-abc **5/5** + with-abc **5/5**, Icarus `iverilog -g2012` **5/5 CLEAN**. | `done` |
| `2026-06-16` | `STRUCTURED-EMISSION-EXPANSION.2a` | Design-detail leaf, **no source change** (grounded in a fresh read of `src/emit/sv.rs` — `to_sv_with_modules` gate-emission loop + `build_names`/`node_ref`/`render_gate`/`param_width_decl_w`; `src/ir/soft_union.rs` + `Module.soft_union_slice_gates` — the gen-time-annotation precedent; the `aggregate_layout` projection). `DEVELOPMENT_NOTES.md` design-detail entry (the five points + the `.2b` pre-split): first-cut single-gate "operand function"; gen-time `annotate_function_emit_gates` + `Module.function_emit_gates`; the `<wire>__f` `function automatic` decl + positional-param body + call; `function_emit_prob` (default `0.0` byte-identical); the `saw_combinational_function_emit` gate. `.2b` pre-split → `.2b.1`/`.2b.2`; frontier set to `.2b.1`. `bash scripts/check_memory_architecture.sh` + `bash knowledge-map/scripts/check_knowledge_map.sh` clean. Baseline `cargo check --all-targets` clean (no source touched). | `done` |
| `2026-06-16` | `STRUCTURED-EMISSION-EXPANSION.1` | Design/decision leaf, **no source change** (grounded in a fresh read of `src/emit/sv.rs` `to_sv_with_modules` + the `aggregate_layout` projection + `soft_union_slice_overlay`, `src/ir/soft_union.rs`, and the `aggregate_prob`/`soft_union_slice_prob` default-off emit-projection knobs in `src/config.rs`). Decision `0012` + `INDEX.md` row; tree activated (`proposed → active`); `.2`/`.2a`/`.2b` registered; frontier set to `.2a`. `bash scripts/check_memory_architecture.sh` + `bash knowledge-map/scripts/check_knowledge_map.sh` clean; `KNOWLEDGE_MAP.md` regenerated (decision `0012` carries `answers:` front-matter). Baseline `cargo check --all-targets` clean (from the prior gate; no source touched). | `done` |
| `2026-06-15` | `STRUCTURED-EMISSION-EXPANSION` | Tree registered `proposed` (ownership only, no leaf executed). | `done` (registration) |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `STRUCTURED-EMISSION-EXPANSION.8b` | `STRUCTURED-EMISSION-EXPANSION.8b — wide-lane generate-loop part-select surface` | Impl of the fourth structured surface: relaxed `generate_loop.rs` `gate_qualifies` to `LW >= 1` (`width == N*LW`) + the `sv.rs` `render_generate_loop_block` `[gi*LW +: LW]` branch (`LW==1` `[gi]` kept byte-identical) + the mirrored `generate_loop_gate` re-check + 4 lib proofs + book/knobs/USER_GUIDE/README/CODEBASE_ANALYSIS/KM closeout. Reuses `generate_loop_emit_prob` + `num_emitted_generate_loops` (no new knob / no schema bump). Downstream-clean (per-seed ON-vs-OFF sweep: 9 wider-lane part-selects, Verilator `-Wall` Δ=0 + Yosys both + Icarus; `--generate-loop-gate` regression-clean 12/0). `cargo test --lib` 493 + snapshots 6/6. Closes `.8b` / `.8` — the fourth structured surface delivered end-to-end; lane returns to no-frontier. |
| `STRUCTURED-EMISSION-EXPANSION.8a` | `STRUCTURED-EMISSION-EXPANSION.8a — wide-lane generate-loop impl design-detail` | Design-detail (no source): a `DEVELOPMENT_NOTES.md` entry grounding decision `0015` in the real `generate_loop.rs` `gate_qualifies` + `sv.rs` `generate_loop_gate`/`render_generate_loop_block` + a corpus-liveness probe (20/448 replications are multi-bit-lane ⇒ the surface fires on real generation). Resolved every open question: keep `generate_loop_gate -> (lane, N)` (recompute `LW` in the renderer); `LW==1` `[gi]` byte-identical vs `LW>1` `[gi*LW +: LW]` branch; predicate `LW >= 1` / `width == N*LW`; byte-identity contract; the lib-emit-test + `--generate-loop-gate` proof plan. No new knob / no schema bump. Split `.8` into `.8a` (done) + `.8b` (impl pending); frontier → `.8b`. No source change; self-checks clean. |
| `STRUCTURED-EMISSION-EXPANSION.7` | `STRUCTURED-EMISSION-EXPANSION.7 — pick wide-lane generate-loop surface + decision 0015` | Design/decision leaf (no source): decision `0015` picks the fourth structured surface — a default-off, valid-by-construction **wider-lane `generate for` part-select** (broaden the `generate for` lane from 1-bit to `LW >= 1`, render `assign <wire>[gi*LW +: LW] = <x>;`, keep `LW==1` byte-identical), the recorded wider-lane follow-up to the second surface. Chosen via a fresh empirical probe (Verilator `-Wall` + both Yosys + Icarus clean + iverilog-sim-proven `== {N{x}}`) that **disqualifies** `interface`/`modport` (Icarus syntax-fail + both-Yosys implicit-decl warn) and records nested-generate as bigger-blast-radius. Reuses the existing `generate_loop_emit_prob` knob + `num_emitted_generate_loops` metric (no new knob / no schema bump). `INDEX.md` row; tree split `.7`/`.8` (pre-split `.8a`/`.8b`)/`.9+`; frontier → `.8a`. No source change; self-checks clean. |
| `STRUCTURED-EMISSION-EXPANSION.6b.3` | `STRUCTURED-EMISSION-EXPANSION.6b.3 — combinational task automatic user docs` | Docs-only closeout: a `## The third surface: a combinational task automatic` section in `book/src/structured-emission.md` (byte-verified seed-1 before/after; the function-surface candidate parallel; the output-var passthrough form; the four-way mutual exclusion; the metric @ schema `1.10` + gate) + the `task_emit_prob` knob entry in `book/src/knobs.md` / `USER_GUIDE.md` / README "Current CLI truth" (config-file-only knob) + KM how-to card `combinational-task-emit` (41 facts / 331 keys). Closes `.6b.3` / `.6b` / `.6` — the third structured surface delivered end-to-end. DUT byte-identical. Nothing retired. |
| `STRUCTURED-EMISSION-EXPANSION.6b.2b` | `STRUCTURED-EMISSION-EXPANSION.6b.2b — task-emit tool_matrix gate` | The repo-owned `tool_matrix --task-emit-gate`: `ScenarioSet::TaskEmitSweep` + `build_task_emit_sweep_scenarios` (comb-only `task_emit_prob=1.0` × 3 strategies) + `ModuleReport.emitted_combinational_task` SV-text detection + `saw_combinational_task_emit` fact + `MatrixReport.task_emit_gate` + early-return gap enforcement + 5 proofs + 6 fixture updates + the `test_cli` default. Banked clean `/tmp/anvil-task-emit-gate-r1` (3 scenarios / 12 modules / 12 emitting a task / `coverage_gaps=[]` / `12/0` Verilator + both Yosys + Icarus). README + USER_GUIDE + CODEBASE_ANALYSIS gate entries. Templated on `--generate-loop-gate`. Default-off / DUT byte-identical (snapshots 6/6). Closes `.6b.2`; frontier → `.6b.3`. |
| `STRUCTURED-EMISSION-EXPANSION.6b.2a` | `STRUCTURED-EMISSION-EXPANSION.6b.2a — task emit metric + introspection schema 1.10` | `Metrics::num_emitted_combinational_tasks` (= `task_emit_gates.len()`) + introspection schema MINOR bump `1.9 → 1.10` (the metric bumps; the `.6b.1` knob rode the version). MINOR is an integer ⇒ `1.9 → 1.10` (ten), not a decimal. Bumped all current-output schema refs (9 test assertions + schema doc + README + USER_GUIDE + 5 book example JSONs + the CODEBASE_ANALYSIS envelope line); historical landing attributions left intact. Lib proof; default-off / DUT byte-identical (snapshots 6/6, lib 490); end-to-end introspect default `0` / forced `39`. Pre-split `.6b.2` → `.6b.2a`/`.6b.2b`; frontier → `.6b.2b`. |
| `STRUCTURED-EMISSION-EXPANSION.6b.1` | `STRUCTURED-EMISSION-EXPANSION.6b.1 — combinational task automatic emit-projection (live surface)` | Live emitter change: `task_emit_prob` knob + `Module.task_emit_gates` + new `src/ir/task_emit.rs` gen-time mark (the function-emit candidate predicate plus exclusion of the three sibling projections) + two generator call-site rolls (after generate_loop) + `to_sv_with_modules` `task_emit_gate` accessor + `render_gate_task_decl` (body via the reused `render_gate_function_body`) + `render_gate_task_call` (the `logic <wire>__tv` var + the `always_comb` call) + the gate-assign-loop passthrough `assign <wire> = <wire>__tv;` + 11 lib proofs. Output-var + passthrough integration (the `.6a` first cut). No schema bump (default-off prob-knob precedent). Default-off / DUT byte-identical (snapshots 6/6, lib 489); forced sweep clean across Verilator `--lint-only` (`-Wall` Δ=0 vs OFF) + both Yosys + Icarus (`/tmp/anvil-te-r1/`, 5 seeds, 4–39 tasks each). Pre-split `.6b` → `.6b.1`/`.6b.2`/`.6b.3`; frontier → `.6b.2`. |
| `STRUCTURED-EMISSION-EXPANSION.6a` | `STRUCTURED-EMISSION-EXPANSION.6a — combinational task impl design-detail` | Design-detail (no source): a `DEVELOPMENT_NOTES.md` entry grounding decision `0014`'s `task` surface in the real emitter (the `to_sv_with_modules` section template; the **reuse of `render_gate_function_body`** as the task body) + the `function_emit.rs`/`generate_loop.rs` chain, resolving all five `.6a` points (output-var + passthrough integration; gen-time `Module.task_emit_gates` excluding the sibling projections; the `task automatic` decl + `always_comb` call + assign-RHS swap; `task_emit_prob` config-file knob; `num_emitted_combinational_tasks` metric schema `1.9→1.10` + `tool_matrix --task-emit-gate`/`saw_combinational_task_emit`) + the `.6b` impl shape. Split `.6` into `.6a` (done) + `.6b` (impl pending); frontier → `.6b`. No source change; self-checks clean. |
| `STRUCTURED-EMISSION-EXPANSION.5` | `STRUCTURED-EMISSION-EXPANSION.5 — pick task surface + decision 0014` | Design/decision leaf (no source): decision `0014` picks the third structured surface — a default-off, valid-by-construction combinational `task automatic` emit-projection of a single combinational gate (the decision `0012` single-gate parallel, but a procedural `task` called from `always_comb`), over nested `generate` + `interface`/`modport`. `task` was the recorded leading candidate (decision `0013`); autonomously selected at a no-frontier boundary per the owner *"pick any tree and roll"* directive. Empirically grounded clean across Verilator `-Wall` + both Yosys + Icarus (both the direct-output and the output-var passthrough forms). `INDEX.md` row; KM card `structured-emission-third-surface-combinational-task`; tree split `.5`/`.6`/`.7+`; frontier → `.6`. No source change; self-checks clean. |
| `STRUCTURED-EMISSION-EXPANSION.4b.3` | `STRUCTURED-EMISSION-EXPANSION.4b.3 — generate-for loop user docs` | Docs-only closeout: a `## The second surface: a generate for loop` section in `book/src/structured-emission.md` (byte-verified seed-12 before/after; the `{N{x}}` 1-bit-lane rule; wider-lane exclusion; `function_emit` mutual exclusion; `gi = gi + 1`) + the `generate_loop_emit_prob` knob entry in `book/src/knobs.md` / `USER_GUIDE.md` / README "Current CLI truth" (config-file-only knob) + KM how-to card `generate-loop-emit` (39 facts / 309 keys). Closes `.4b.3` / `.4b` / `.4` — the second structured surface delivered end-to-end. DUT byte-identical. Nothing retired. |
| `STRUCTURED-EMISSION-EXPANSION.4b.2b` | `STRUCTURED-EMISSION-EXPANSION.4b.2b — generate-loop tool_matrix gate` | The repo-owned `tool_matrix --generate-loop-gate`: `ScenarioSet::GenerateLoopSweep` + `build_generate_loop_sweep_scenarios` (comb-only `generate_loop_emit_prob=1.0` × 3 strategies) + `ModuleReport.emitted_generate_loop` SV-text detection + `saw_generate_loop_emit` fact + early-return gap enforcement + 5 proofs + 6 fixture updates. Banked clean `/tmp/anvil-generate-loop-gate-r1` (3 scenarios / 12 modules / 8 emitting a loop / `coverage_gaps=[]` / `12/0` Verilator + both Yosys + Icarus). README + USER_GUIDE + CODEBASE_ANALYSIS gate entries. Default-off / DUT byte-identical (snapshots 6/6). Closes `.4b.2`; frontier → `.4b.3`. |
| `STRUCTURED-EMISSION-EXPANSION.4b.2a` | `STRUCTURED-EMISSION-EXPANSION.4b.2a — generate-loop emit metric + introspection schema 1.9` | `Metrics::num_emitted_generate_loops` (= `generate_loop_gates.len()`) + introspection schema MINOR bump `1.8 → 1.9` (the metric bumps; the `.4b.1` knob rode the version). Bumped all current-output schema refs (9 test assertions + schema doc + README + USER_GUIDE + 5 book example JSONs + the CODEBASE_ANALYSIS envelope line); historical landing attributions left intact. Lib proof; default-off / DUT byte-identical (snapshots 6/6, lib 478); end-to-end introspect default `0` / forced `50`. Pre-split `.4b.2` → `.4b.2a`/`.4b.2b`; frontier → `.4b.2b`. |
| `STRUCTURED-EMISSION-EXPANSION.4b.1` | `STRUCTURED-EMISSION-EXPANSION.4b.1 — generate-for loop emit-projection (live surface)` | Live emitter change: `generate_loop_emit_prob` knob + `Module.generate_loop_gates` + new `src/ir/generate_loop.rs` gen-time mark (`{N{x}}` 1-bit-lane replication candidate excluding function-emit marks) + two generator call-site rolls (after function_emit) + `to_sv_with_modules` `generate_loop_gate` accessor + `render_generate_loop_block` + the generate-block section + assign-loop inline-replication suppression + 9 lib proofs. Increment form `gi = gi + 1` (portable; `gi++` not retired). No schema bump (default-off prob-knob precedent). Default-off / DUT byte-identical (snapshots 6/6, lib 477); forced sweep clean across Verilator `--lint-only` (`-Wall` Δ=0 vs OFF) + both Yosys + Icarus (`/tmp/anvil-gl-r1/`, 5 seeds). Pre-split `.4b` → `.4b.1`/`.4b.2`/`.4b.3`; frontier → `.4b.2`. |
| `STRUCTURED-EMISSION-EXPANSION.4a` | `STRUCTURED-EMISSION-EXPANSION.4a — generate-for loop impl design-detail` | Design-detail leaf (no source): a `DEVELOPMENT_NOTES.md` entry grounding decision `0013`'s `generate for` loop surface in the real emitter (`render_gate`'s `Concat` replication predicate `sv.rs:1159`) + the `function_emit.rs`/`soft_union.rs` gen-time-annotation precedent, resolving all five `.4a` points (selection rule = `{N{x}}` 1-bit-lane replication excluding function-emit marks; gen-time `Module.generate_loop_gates`; `genvar`/`generate for` rendering + assign suppression; `generate_loop_emit_prob` config-file-only knob; `tool_matrix --generate-loop-gate` / `saw_generate_loop_emit`) + the `.4b` impl shape. Split `.4` into `.4a` (done) + `.4b` (impl pending); frontier → `.4b`. No source change; self-checks clean. |
| `STRUCTURED-EMISSION-EXPANSION.3` | `STRUCTURED-EMISSION-EXPANSION.3 — pick generate-for surface + decision 0013` | Design/decision leaf (no source): decision `0013` picks the second structured surface — a default-off, valid-by-construction `generate for` loop emit-projection of an existing `{N{x}}` replication (over `task` [leading future], `interface`/`modport`, `generate if`), empirically grounded clean across Verilator `-Wall` + both Yosys + Icarus. `INDEX.md` row; KM card `structured-emission-second-surface-generate-loop`; tree split `.3`/`.4`/`.5+`; frontier → `.4`. No source change; self-checks clean. |
| `STRUCTURED-EMISSION-EXPANSION.2b.2c` | `STRUCTURED-EMISSION-EXPANSION.2b.2c — combinational function emit user docs` | Docs-only closeout: new `How It Works` book chapter `book/src/structured-emission.md` (byte-verified before/after; single-gate rule; `Slice`/structured exclusions; duplicate-operand positional params; combinational-only) + `function_emit_prob` knob entry in `book/src/knobs.md` / `USER_GUIDE.md` / README "Current CLI truth" (config-file-only knob) + KM how-to card `combinational-function-emit` (37 facts / 286 keys). Closes `.2b.2` / `.2b` / `.2` — the first structured surface delivered end-to-end. DUT byte-identical. Nothing retired. |
| `STRUCTURED-EMISSION-EXPANSION.2b.2b` | `STRUCTURED-EMISSION-EXPANSION.2b.2b — function-emit tool_matrix gate` | The repo-owned `tool_matrix --function-emit-gate`: `ScenarioSet::FunctionEmitSweep` + `build_function_emit_sweep_scenarios` (comb-only `function_emit_prob=1.0` × 3 strategies) + `ModuleReport.emitted_combinational_function` SV-text detection + `saw_combinational_function_emit` fact + early-return gap enforcement + 5 proofs. Banked clean `/tmp/anvil-function-emit-gate-r1` (3 scenarios / 12 modules / 608 functions / `coverage_gaps=[]` / `12/0` Verilator + both Yosys + Icarus). Default-off / DUT byte-identical (snapshots 6/6). |
| `STRUCTURED-EMISSION-EXPANSION.2b.2a` | `STRUCTURED-EMISSION-EXPANSION.2b.2a — emit metric + introspection schema 1.8` | `Metrics::num_emitted_combinational_functions` (= `function_emit_gates.len()`) + introspection schema MINOR bump `1.7 → 1.8` (the metric bumps; the `.2b.1` knob rode the version). Bumped all current-output schema refs (tests + schema doc + README + USER_GUIDE + 5 book example JSONs + the stale `CODEBASE_ANALYSIS` envelope line). Lib proof; default-off / DUT byte-identical (snapshots 6/6). |
| `STRUCTURED-EMISSION-EXPANSION.2b.1` | `STRUCTURED-EMISSION-EXPANSION.2b.1 — combinational function automatic emit-projection (live surface)` | Live emitter change: `function_emit_prob` knob + `Module.function_emit_gates` + `src/ir/function_emit.rs` gen-time mark + two generator call-site rolls (after soft_union) + `to_sv_with_modules` `<wire>__f` `function automatic` decl/positional-body/call rendering + 9 lib proofs. `Slice` excluded from the first cut (`-Wall UNUSEDSIGNAL` on a full-width param; still emitted inline, nothing retired). No schema bump (default-off prob-knob precedent). Default-off / DUT byte-identical (snapshots 6/6); forced sweep clean across Verilator + both Yosys + Icarus (`/tmp/anvil-fe-r2/`). |
| `STRUCTURED-EMISSION-EXPANSION.2a` | `STRUCTURED-EMISSION-EXPANSION.2a — combinational function impl design-detail` | Design-detail (no source): pinned the first-cut single-gate "operand function" (minimal cone ⇒ zero sharing hazard), the gen-time `annotate_function_emit_gates` + `Module.function_emit_gates` annotation (the `soft_union.rs` precedent), the `function automatic` decl/positional-body/call rendering, the `function_emit_prob` knob, and the downstream gate. Pre-split `.2b` → `.2b.1`/`.2b.2`. |
| `STRUCTURED-EMISSION-EXPANSION.1` | `STRUCTURED-EMISSION-EXPANSION.1 — activate lane + decision 0012` | Decision `0012`: the first structured surface is a default-off, valid-by-construction combinational `function automatic` emit-projection of an existing cone (over interface/modport + nested generate). Activated the lane by owner directive; split `.1`/`.2`/future; pre-split `.2` → `.2a`/`.2b`. No source change. |
| `STRUCTURED-EMISSION-EXPANSION` | `SV-VERSION-TARGETING.1 — open SV-version lane + decision 0009` | Registered `proposed` alongside the activated `SV-VERSION-TARGETING` lane. |

## Changelog

- `2026-06-17`: **`.8b` landed — the FOURTH structured surface (the wider-lane
  `generate for` part-select) is delivered end-to-end; `.8b` / `.8` close; the
  lane returns to no-active-frontier (open-ended).** First source change of the
  fourth surface. Two surgical edits: `src/ir/generate_loop.rs` `gate_qualifies`
  relaxed (`lane.width() != 1 || *width != operands.len()` → `lw = lane.width();
  lw == 0 || *width != operands.len() * lw`, i.e. any `LW >= 1` with
  `width == N*LW`; the `function_emit`/`soft_union` + all-same-operand/`N>=2`
  exclusions unchanged) + `src/emit/sv.rs` (`generate_loop_gate` defensive
  re-check mirrored to the same condition, still returns `(lane, N)`;
  `render_generate_loop_block` computes `lw = m.nodes[lane].width()` and branches
  `lw == 1` → verbatim `assign <name>[gi] = <x>;` (the shipped 1-bit surface
  byte-identical) vs `lw > 1` → `assign <name>[gi*LW +: LW] = <x>;`). 4 lib test
  changes (`wide_lane_replication_qualifies`,
  `mismatched_result_width_replication_does_not_qualify`, the
  `module_wide_replication` helper + `marked_wide_lane_gate_emits_part_select_loop`,
  `marked_one_bit_lane_keeps_index_body_byte_identical`). Book: the second-surface
  "What gets wrapped" rewritten to `LW >= 1` + a new `## The fourth surface:
  wider lanes via a part-select` section (byte-verified seed-74 before/after,
  `{2{i_2}}` → `[gi*2 +: 2]`, fully `-Wall` clean) + the `generate_loop_emit_prob`
  entry in `knobs.md` / `USER_GUIDE.md` / README + the `--generate-loop-gate`
  description + the `CODEBASE_ANALYSIS.md` `generate_loop.rs` block + the KM card
  `generate-loop-emit` (the "excluded wider lane" framing replaced with the
  shipped fourth surface). **Reuses** `generate_loop_emit_prob` +
  `num_emitted_generate_loops` — no new knob / no new metric / no introspection
  schema bump. `cargo check/clippy -D warnings/fmt` clean; `cargo test --lib` 493
  + snapshots 6/6 byte-identical (default-off); a per-seed ON-vs-OFF downstream
  sweep (`/tmp/anvil-gl8b/`, 8 seeds) emits 9 wider-lane part-selects with
  Verilator `-Wall` Δ=0 + Yosys both modes + Icarus rc=0; the
  `--generate-loop-gate` bank (`/tmp/anvil-generate-loop-gate-8b`) stays
  regression-clean (12/0, `coverage_gaps=[]`, `saw_generate_loop_emit`); `mdbook
  build` + `check_knowledge_map` + `book_examples` 3/3 green. Default-off / DUT
  byte-identical. Nothing retired.
- `2026-06-17`: **`.8a` landed — the wider-lane `generate for` part-select impl
  design-detail; `.8` frontier → `.8b`.** Design-detail leaf, no source change (a
  `DEVELOPMENT_NOTES.md` entry). Grounded decision `0015` in the real
  `src/ir/generate_loop.rs` `gate_qualifies` + `src/emit/sv.rs`
  `generate_loop_gate`/`render_generate_loop_block` and resolved every open
  question (keep `generate_loop_gate -> (lane, N)` + recompute
  `LW = m.nodes[lane].width()` in the renderer; branch `LW==1` verbatim `[gi]`
  vs `LW>1` `[gi*LW +: LW]`; relax the predicate to `LW >= 1` / `width == N*LW`
  with the `function_emit`/`soft_union` exclusions unchanged; the byte-identity
  contract for the shipped 1-bit surface; the lib-emit-test + `--generate-loop-gate`
  proof plan). **Corpus-liveness proven**: a 300-module comb-only sweep
  (`/tmp/anvil-widelane-probe/`) emits 448 `{N{x}}` replications, 20 with a
  multi-bit lane (`{2{i_4}}` 7b→14b, `{3{case_mux_0}}` 12b→36b, `{6{i_1}}`
  8b→48b, `{4{concat_7}}` 20b→80b) — the surface fires on real generation
  (~4.5%), not hand-built-only. Reuses `generate_loop_emit_prob` +
  `num_emitted_generate_loops` ⇒ no new knob / no new metric / no schema bump.
  `.8b` impl shape recorded (the two surgical edits + the lib proofs + the gate
  confirmation + the book/USER_GUIDE caveat replacement).
  `check_memory_architecture` + `check_knowledge_map` green; no source touched.
- `2026-06-17`: **`.7` landed — picked the FOURTH structured surface (the
  wider-lane `generate for` part-select); decision `0015`; frontier → `.8a`.**
  Design/decision leaf, no source change. At a no-active-frontier boundary,
  autonomously selected (`feedback_pick_and_roll_at_no_frontier`) the recorded
  wider-lane follow-up to the second surface (decision `0013` / the book both
  recorded "a wider lane would need a part-select body and stays inline — a
  recorded follow-up"). It broadens the `generate for` lane from 1-bit to
  `LW >= 1`: a `{N{x}}` replication whose lane is `LW` bits (result `N*LW`)
  renders `assign <wire>[gi*LW +: LW] = <x>;`, while `LW==1` keeps the existing
  `assign <wire>[gi] = <x>;` verbatim (the shipped 1-bit surface stays
  byte-identical); bit-group `g` of `{N{x}}` is exactly the lane ⇒
  byte-equivalent. A **fresh empirical probe** (Verilator 5.046 `-Wall` + Yosys
  0.64 both modes + Icarus 13.0, `/tmp/anvil-probe-se4/`) accepts it
  warning-clean and iverilog simulation proves it bit-equal to `{4{b}}`; the same
  probe **disqualifies `interface`/`modport`** (Icarus syntax-fails the modport
  port; both Yosys modes warn on the implicit interface-member decl — confirming
  the recorded weak-support claim) and records nested-generate as
  clean-but-bigger-blast-radius + `generate if` as clean-but-dead-branch.
  Discipline: rules-first; **reuses** `generate_loop_emit_prob` +
  `num_emitted_generate_loops` (no new knob / no new metric / no schema bump);
  default-off / DUT byte-identical. `docs/decisions/0015-...md` + `INDEX.md` row;
  KM regenerated (decision `0015` carries `answers:`); tree split `.7` (done) +
  `.8` (impl pending, pre-split `.8a` design-detail + `.8b` impl) + future
  `.9+` (nested/multi-level `generate`, `interface`/`modport`, richer tasks).
  `check_memory_architecture` + `check_knowledge_map` + `mdbook build` green;
  no source touched. Nothing retired.
- `2026-06-16`: **`.6b.3` landed — the user-facing closeout; the THIRD structured
  surface (the combinational `task automatic`) is delivered end-to-end; `.6b.3` /
  `.6b` / `.6` all close; the lane returns to no-active-frontier (open-ended).**
  Docs-only / DUT byte-identical. `book/src/structured-emission.md` gains a `## The
  third surface: a combinational task automatic` section: the function-surface
  parallel, a **byte-verified seed-1 before/after** (the inline `assign shr_0 =
  i_2 >> 2'h3;` becomes the `task automatic shr_0__t(output logic [3:0] o, input
  …); o = a0 >> a1; endtask` decl + `logic [3:0] shr_0__tv;` + `always_comb
  shr_0__t(shr_0__tv, i_2, 2'h3);` + the passthrough `assign shr_0 = shr_0__tv;`
  — everything else byte-identical), the same candidate set as `function_emit`,
  the structured/`Slice` exclusions, the four-way mutual exclusion, the output-var
  passthrough integration, combinational-only, and the
  `num_emitted_combinational_tasks` metric (@ schema `1.10`) + the `tool_matrix
  --task-emit-gate` proof, plus a skip-sentinelled repro `bash` block; the chapter
  intro now lists `task` as live. The `task_emit_prob` knob is added to
  `book/src/knobs.md` (the `### Structured emission` subsection, beside
  `function_emit_prob` / `generate_loop_emit_prob`), `USER_GUIDE.md` (after the
  `generate_loop_emit_prob` config-knob bullet), and the README "Current CLI
  truth" (a config-file knob bullet). New Knowledge Map how-to card
  `docs/knowledge/combinational-task-emit.md` (id `combinational-task-emit`) with
  how-to question keys + a validated `reverify` command; KM regenerated 40→41
  facts / 318→331 keys. The book example is byte-verified downstream-clean
  (Verilator `-Wall` with the matching filename + both Yosys + Icarus). `mdbook
  build` + `check_knowledge_map` + `check_memory_architecture` + `cargo test
  --test book_examples` 3/3 green. With this leaf the third structured surface is
  delivered end-to-end; the tree stays `active` as an open-ended lane with no
  current frontier (future surfaces — nested/multi-level `generate`,
  `interface`/`modport`, richer tasks — are `.7+`). Nothing retired.
- `2026-06-16`: **`.6b.2b` landed — the repo-owned `tool_matrix --task-emit-gate`;
  `.6b.2` closes; frontier → `.6b.3`.** `src/bin/tool_matrix.rs` gains the
  `--task-emit-gate` CLI flag + `ScenarioSet::TaskEmitSweep` +
  `MatrixReport.task_emit_gate` (wired into `select_scenario_set` [mutually
  exclusive], `derive_run_plan` [`TASK_EMIT_SWEEP_MIN_UNITS_PER_SCENARIO=4`
  units/scenario floor + `fail_on_coverage_gap`], `build_scenarios`,
  `scenario_set_slug` "task-emit-sweep", `artifact_kind_slug` "module") +
  `build_task_emit_sweep_scenarios`/`task_emit_focus_config` (the
  `function_emit_focus_config`-shaped comb-only single-module DUT — node-id +
  e-graph, `flop_prob=0.0` — with `task_emit_prob=1.0` × all three construction
  strategies = 3 scenarios) + `ModuleReport.emitted_combinational_task`
  (`#[serde(default)]`, set from `prepared.sv_text.contains("task automatic")`) +
  `CoverageSummary.saw_combinational_task_emit` (lit in `summarize_coverage` when
  an emitted-task module is Verilator-success AND has a non-empty clean Yosys vec
  — a combinational task is universally synthesizable like a function, so the full
  tool plan; Icarus rides `ToolSummary::any_failed`) + merge in `merge_coverage` +
  an early-return arm in `compute_coverage_gaps` + 5 cargo-portable proofs + the
  new field threaded through 6 `ModuleReport` fixtures + the `test_cli` default;
  README + USER_GUIDE + CODEBASE_ANALYSIS gate entries. Banked clean
  `/tmp/anvil-task-emit-gate-r1` (`--task-emit-gate --yosys-mode both
  --iverilog-compile`): 3 scenarios / 12 modules / **12 emitting a task** /
  `coverage_gaps = []` / `saw_combinational_task_emit = true` / Verilator `12/0` /
  Yosys without-abc `12/0` / Yosys with-abc `12/0` / Icarus compile `12/0`. No
  schema bump (harness-only); `cargo test --bin tool_matrix` 68 / 1 ignored;
  snapshots 6/6 byte-identical (default `task_emit_prob = 0.0` emission
  unchanged). Closes `.6b.2`; frontier → `.6b.3`. Nothing retired.
- `2026-06-16`: **`.6b.2a` landed — the `num_emitted_combinational_tasks` metric +
  introspection schema `1.9 → 1.10`; `.6b.2` split into `.6b.2a` (done) + `.6b.2b`
  (the `tool_matrix` gate); frontier → `.6b.2b`.** `Metrics::num_emitted_combinational_tasks`
  (`= m.task_emit_gates.len()`, `#[serde(default)]`) computed in `metrics::compute()`
  and surfaced in introspection `module_metrics` (the SCHEMA-DERIVED projection) ⇒
  `SCHEMA_VERSION` `1.9 → 1.10` in `src/introspect/mod.rs`. The metric **bumps** the
  schema (new derived `Metrics` field — the `1.8→1.9` `num_emitted_generate_loops`
  precedent) whereas the `.6b.1` knob did **not** (default-off prob-knob rides
  `request.knobs` via `#[serde(default)]`). MINOR is an integer, so `1.9 → 1.10`
  (ten), not a decimal — recorded in the `SCHEMA_VERSION` doc comment + the
  schema-doc changelog. Bumped all current-output schema refs to `1.10`: the 9
  `schema_version` assertions (2 in `src/introspect/mod.rs` + 7 in `src/mcp/mod.rs`),
  the schema doc (`1.9→1.10` changelog entry + the early-example/defines/checklist
  lines), README (`--introspect` + `analyze`), USER_GUIDE (the `--sv-version
  --introspect` row), the 5 `book/src/agent-mcp.md` example JSONs, and the
  `CODEBASE_ANALYSIS` envelope line. Historical landing attributions left intact
  (`num_emitted_generate_loops` @ 1.9; `num_emitted_combinational_functions` @ 1.8;
  sv-version @ 1.2; the schema-doc `1.8→1.9` changelog entry). Lib proof
  `metrics_count_emitted_combinational_tasks`. `cargo clippy(-D warnings)/fmt`
  clean; `cargo test --lib` 490 / 2 ignored; snapshots 6/6 byte-identical
  (default-off; metric changes no RTL); end-to-end `--introspect` default `0` /
  forced `39`; `mdbook build` OK. Frontier → `.6b.2b`. Nothing retired.
- `2026-06-16`: **`.6b.1` landed — the combinational `task automatic` live
  surface; `.6b` split into `.6b.1` (done) + `.6b.2` (metric + gate) + `.6b.3`
  (docs closeout); frontier → `.6b.2`.** First source change since `.4b.1`. The
  third richer-structured emit surface (decision `0014`) goes live: ANVIL's DUT
  lane can project a single combinational gate into a procedural `task automatic`
  called from `always_comb`. Live emitter change: `Config::task_emit_prob`
  (default `0.0`, config-file-only) + `Module.task_emit_gates` + new
  `src/ir/task_emit.rs` (`annotate_task_emit_gates`, the function-emit candidate
  predicate **plus** exclusion of `function_emit_gates` / `generate_loop_gates` /
  `soft_union_slice_gates`) + two guarded gen-time call-site rolls (after
  generate_loop) + the `to_sv_with_modules` `task_emit_gate` accessor +
  `render_gate_task_decl` (`task automatic <wire>__t(output logic [W-1:0] o,
  input …); o = <body>; endtask`, body **reusing `render_gate_function_body`
  verbatim**) + `render_gate_task_call` (`logic [W-1:0] <wire>__tv; always_comb
  <wire>__t(<wire>__tv, <refs>);`) + the gate-assign-loop passthrough `assign
  <wire> = <wire>__tv;` + 11 lib proofs. Output-var + passthrough integration
  (the `.6a` first cut; `<wire>` stays a net; `<wire>`-as-var rejected). No schema
  bump (the knob rides `#[serde(default)]`; the `num_emitted_combinational_tasks`
  metric bumps `1.9→1.10` at `.6b.2`). `cargo check/clippy(-D warnings)/fmt`
  clean; `cargo test --lib` 489 / 2 ignored (incl. 11 new proofs); snapshots 6/6
  byte-identical (default-off). Forced `task_emit_prob=1.0` sweep clean across
  Verilator `--lint-only` (`-Wall` Δ=0 vs OFF) + both Yosys + Icarus
  (`/tmp/anvil-te-r1/`, 5 seeds, 4–39 tasks each). Frontier → `.6b.2`. Nothing
  retired.
- `2026-06-16`: **`.6a` landed — the combinational `task automatic` impl
  design-detail; `.6` split into `.6a` (done) + `.6b` (impl pending); frontier →
  `.6b`.** Design-detail leaf, no source change (a `DEVELOPMENT_NOTES.md` entry +
  the tree split). Grounded decision `0014` in the real emitter — the
  `to_sv_with_modules` function-decl + generate-block sections are the structural
  template for a third (task) section, the per-gate assign-loop `continue` is the
  suppression pattern, and **`render_gate_function_body` is reused verbatim** as
  the task body (`o = a0 op a1 …` over positional params, with the `output` param
  `o` as LHS). Resolved all five `.6a` points: (1) the **output-var +
  passthrough-`assign`** integration (keep `<wire>` a net, add `logic
  <wire>__tv`, `always_comb <wire>__t(<wire>__tv, …)`, swap the gate's assign RHS
  to `<wire>__tv` — only the gate's own drive changes, the `function_emit`
  parallel; `<wire>`-as-var rejected for the first cut, one `always_comb` per task
  gate); (2) gen-time `src/ir/task_emit.rs annotate_task_emit_gates` +
  `Module.task_emit_gates`, candidate = the function-emit predicate **plus**
  exclusion of `function_emit_gates`/`generate_loop_gates`/`soft_union_slice_gates`,
  run after generate_loop (later pass excludes earlier marks); (3) a
  `task_emit_gate` accessor + the `task automatic <wire>__t(output …, input …)`
  decl + the `always_comb` call + the assign-RHS swap; (4)
  `Config::task_emit_prob` (config-file-only, default `0.0`, byte-identical, no
  schema bump for the knob); (5) a `num_emitted_combinational_tasks` metric
  (schema `1.9→1.10`) + `tool_matrix --task-emit-gate` + `ScenarioSet::TaskEmitSweep`
  + `ModuleReport.emitted_combinational_task` + `saw_combinational_task_emit`
  (full Verilator + both Yosys plan). Recorded the `.6b` impl shape (pre-split
  `.6b.1` live / `.6b.2` metric+gate / `.6b.3` docs per the `.4b` precedent).
  Self-checks clean (`mdbook build` + `check_knowledge_map` +
  `check_memory_architecture`). Frontier → `.6b`. Nothing retired.
- `2026-06-16`: **`.5` landed — picked the THIRD structured surface (`task`);
  decision `0014`.** Design/decision leaf, no source change. At a no-active-frontier
  boundary (the `generate for` surface fully delivered), the owner directed *"pick
  any tree and roll with it, you decide the best route"*; I autonomously selected
  the **third structured-emission surface** — `task`, the recorded leading future
  candidate from decision `0013` (highest-confidence/lowest-risk continuation with
  maximal warm context on the emitter/annotation/gate machinery). The third
  richer-structured surface is a default-off, valid-by-construction combinational
  **`task automatic`** emit-projection of a single combinational gate — the
  decision `0012` single-gate parallel, but a *procedural* `task` with an `output`
  argument called from `always_comb` (vs the value-returning `function`). For a
  marked gate `<wire> = op(o0,o1,…)`: `task automatic <wire>__t(output …, input
  …); o = a0 op a1 …; endtask` + (minimal-blast-radius) `logic <wire>__tv;
  always_comb <wire>__t(<wire>__tv, …); assign <wire> = <wire>__tv;`. **Empirically
  grounded this session:** a combinational `task` called from `always_comb` is
  accepted warning-clean by Verilator 5.046 `-Wall` + both repo Yosys modes +
  Icarus, in *both* the direct-output and the output-var passthrough forms —
  confirming decision `0013`'s narrowed caution (simple combinational void tasks
  are clean; the weakness is multi-output/side-effecting). Chosen over nested
  `generate` (deeper variant of an already-shipped surface) + `interface`/`modport`
  (still weak Yosys synth). Discipline: rules-first (mark an already-valid gate;
  no generate-then-filter), default-off `task_emit_prob` (proposed; default `0.0`)
  ⇒ byte-identical, no new IR node / no new whole-module behaviour; mutually
  exclusive with the sibling projections; combinational only; structured selectors
  + `Slice` excluded (same reasons as `function_emit`). Downstream gate
  (`saw_combinational_task_emit`). Decision `0014` + `INDEX.md` row + KM card
  `structured-emission-third-surface-combinational-task`. Tree split `.5` (done) +
  `.6` (impl; pre-split `.6a`/`.6b`) + future (`.7+`). Frontier → `.6`. Self-checks
  clean (`mdbook build` + `check_knowledge_map` + `check_memory_architecture`).
- `2026-06-16`: **`.4b.3` landed — the user-facing closeout; the second
  structured surface (the `generate for` loop) is delivered end-to-end; `.4b.3` /
  `.4b` / `.4` all close.** Docs-only / DUT byte-identical. `book/src/structured-emission.md`
  gains a `## The second surface: a generate for loop` section: the index-regular
  `{N{x}}` source rationale, a **byte-verified seed-12 before/after** (the inline
  `assign concat_0 = {5{slice_0}};` becomes the `genvar concat_0__gi; generate
  for (… = … + 1) begin : concat_0__gen assign concat_0[concat_0__gi] = slice_0;
  end endgenerate` block — everything else byte-identical), the `{N{x}}`
  1-bit-lane qualification rule (`W == N` ⇒ bit-faithful), the wider-lane
  part-select exclusion (a recorded follow-up; nothing retired), the
  `function_emit` mutual exclusion, the `gi = gi + 1` increment, and the
  `num_emitted_generate_loops` metric (@ schema `1.9`) + the `tool_matrix
  --generate-loop-gate` proof, plus a skip-sentinelled repro `bash` block; the
  chapter intro now notes `generate` is live. The `generate_loop_emit_prob` knob
  is added to `book/src/knobs.md` (the `### Structured emission` subsection,
  beside `function_emit_prob`), `USER_GUIDE.md` (after the `function_emit_prob`
  bullet; intro pluralised), and the README "Current CLI truth" (a config-file
  knob bullet). New Knowledge Map how-to card `docs/knowledge/generate-loop-emit.md`
  (id `generate-loop-emit`) with how-to question keys + a validated `reverify`
  command; KM regenerated 38→39 facts / 296→309 keys. The book example is
  byte-verified downstream-clean (Verilator `-Wall` with the matching filename +
  both Yosys + Icarus). `mdbook build` + `check_knowledge_map` +
  `check_memory_architecture` + `cargo test --test book_examples` 3/3 green.
  Docs-only ⇒ no `src/` touched. **The tree stays `active` as an open-ended lane
  with no current frontier**; future surfaces (`task` [leading], nested/multi-level
  `generate`, `interface`/`modport`) are `.5+`, each its own decision. Nothing
  retired.
- `2026-06-16`: **`.4b.2b` landed — the repo-owned `tool_matrix
  --generate-loop-gate`; `.4b.2` closes.** `src/bin/tool_matrix.rs` gains the
  `--generate-loop-gate` flag + `ScenarioSet::GenerateLoopSweep` +
  `build_generate_loop_sweep_scenarios`/`generate_loop_focus_config` (one
  comb-only `generate_loop_emit_prob=1.0` DUT × three construction strategies) +
  `MatrixReport.generate_loop_gate` (wired through `select_scenario_set`
  [mutually exclusive], `derive_run_plan` [4 units/scenario floor +
  fail-on-gap], `build_scenarios`, `scenario_set_slug` "generate-loop-sweep",
  `artifact_kind_slug` "module") + `ModuleReport.emitted_generate_loop`
  (`#[serde(default)]`, from `prepared.sv_text.contains("generate")`) +
  `CoverageSummary.saw_generate_loop_emit` (lit in `summarize_coverage` on
  Verilator success AND non-empty clean Yosys — a `generate for` is universally
  synthesizable like a function, so the full tool plan runs; Icarus rides the
  `ToolSummary::any_failed` bail) + `merge_coverage` + an early-return arm in
  `compute_coverage_gaps`. 5 cargo-portable proofs + the new field threaded
  through 6 `ModuleReport` fixtures + the `test_cli` default. README "Current
  CLI truth" + USER_GUIDE gate-list + `CODEBASE_ANALYSIS.md` tool_matrix section
  gain the `--generate-loop-gate` entry. No schema bump (harness-only). `cargo
  check --bin tool_matrix` + clippy `-D warnings` + fmt clean; `cargo test --bin
  tool_matrix` 63 / 1 ignored (5 new gate proofs); `cargo test --test
  snapshots` 6/6 byte-identical (harness-only). **Banked downstream-clean**
  `/tmp/anvil-generate-loop-gate-r1` (`--generate-loop-gate --yosys-mode both
  --iverilog-compile`): 3 scenarios / 12 modules / **8 emitting a generate
  loop** / `coverage_gaps = []` / `saw_generate_loop_emit = true` / Verilator
  `12/0` / Yosys without-abc `12/0` / Yosys with-abc `12/0` / Icarus compile
  `12/0`. Frontier → `.4b.3` (the user-facing closeout). Nothing retired.
- `2026-06-16`: **`.4b.2a` landed — the `num_emitted_generate_loops` metric +
  introspection schema `1.8 → 1.9`; `.4b.2` split into `.4b.2a` (done) + `.4b.2b`
  (the gate).** `Metrics::num_emitted_generate_loops` (`= m.generate_loop_gates.len()`,
  `#[serde(default)]`) added to `src/metrics.rs` + computed in `compute()`,
  surfaced in introspection `module_metrics` ⇒ `SCHEMA_VERSION` bumped `1.8 → 1.9`
  in `src/introspect/mod.rs`. The metric **bumps** the schema (new derived
  `Metrics` field — the `1.7→1.8` `num_emitted_combinational_functions` precedent)
  whereas the `.4b.1` knob did **not** (default-off prob-knob rides `request.knobs`
  via `#[serde(default)]`). Bumped all current-output schema refs to `1.9`: the 9
  `schema_version` assertions (2 in `introspect/mod.rs` + 7 in `mcp/mod.rs`), the
  schema doc (`1.8→1.9` changelog entry + the defines/checklist lines), README
  (`--introspect` + `analyze`), USER_GUIDE (`--introspect`), the 5
  `book/src/agent-mcp.md` example JSONs, and the `CODEBASE_ANALYSIS.md` envelope
  line. Historical "landed at schema X" attributions left intact
  (`num_emitted_combinational_functions` @ `1.8`; sv-version @ `1.2`; the
  schema-doc `1.7→1.8` entry). Lib proof `metrics_count_emitted_generate_loops`
  (unmarked `0`, marked `1`). `cargo clippy -D warnings` + fmt clean; `cargo test
  --lib` 478 / 2 ignored; `cargo test --test snapshots` 6/6 byte-identical
  (default-off; the metric changes no RTL); end-to-end `--introspect` default ⇒
  `1.9` + `0`, forced `generate_loop_emit_prob=1.0` ⇒ `1.9` + `50`; `mdbook build`
  OK. Frontier → `.4b.2b`. Nothing retired.
- `2026-06-16`: **`.4b.1` landed — the `generate for` loop live surface; `.4b`
  split into `.4b.1` (done) + `.4b.2` (gate + metric) + `.4b.3` (docs closeout).**
  The second richer-structured emit surface (decision `0013`) goes live exactly
  per the `.4a` design. `Config::generate_loop_emit_prob` (default `0.0`,
  config-file-only) + `Module.generate_loop_gates: BTreeSet<NodeId>` + new
  `src/ir/generate_loop.rs annotate_generate_loop_gates` (candidate = a
  `GateOp::Concat` of the `{N{x}}` form — `≥ 2` operands all the same `NodeId` —
  with a **1-bit lane** so result width `== N`; excludes `function_emit_gates` +
  `soft_union_slice_gates`; `param_env` modules skipped) + the two guarded
  gen-time call-site rolls in `generate_module` + `generate_design` (after
  function_emit) + the `src/emit/sv.rs` rendering (`generate_loop_gate` accessor
  + `render_generate_loop_block` + a generate-block section after the
  function-decl section + the per-gate assign-loop `continue` suppressing the
  inline `{N{x}}`). Increment form `gi = gi + 1` (the maximally-portable form;
  decision `0013` rendered `gi++`, `.4a` recorded `gi = gi + 1` as the portable
  fallback — implemented the fallback, verified clean; `gi++` not retired). 9 lib
  proofs. No introspection schema bump (the default-off prob-knob rides
  `request.knobs` via `#[serde(default)]`, the `.2b.1` precedent; the `.4b.2`
  `num_emitted_generate_loops` metric bumps `1.8→1.9`). Default-off / DUT
  byte-identical (snapshots 6/6; `cargo test --lib` 477 / 2 ignored; clippy `-D
  warnings` + fmt clean). Forced `generate_loop_emit_prob=1.0` sweep (5 seeds,
  `/tmp/anvil-gl-r1/`, 62–168 loops each): Verilator `--lint-only` 5/5 rc=0 / 0
  warnings, **`-Wall` ON-vs-OFF delta = 0**, Yosys without-abc 5/5 + with-abc
  5/5, Icarus `iverilog -g2012` 5/5. The 1-bit-lane `{W{sel}}` broadcast is the
  common one-hot mux-mask idiom — a forced-knob default-config probe lit a
  `generate for` on **27/30** seeds, so the surface is not rare. `CODEBASE_ANALYSIS.md`
  module map gains the `ir/generate_loop.rs` entry. Frontier → `.4b.2`. Nothing
  retired.
- `2026-06-16`: **`.4a` landed — the `generate for` loop impl design-detail;
  `.4` split into `.4a` (done) + `.4b` (impl pending).** Design-detail leaf, no
  source change (a `DEVELOPMENT_NOTES.md` entry + the tree split). Grounded
  decision `0013` in the real emitter: `render_gate`'s existing `{N{x}}`
  replication predicate (`Concat`, `operands.len() >= 2 && all-same-NodeId`,
  `sv.rs:1159`) **is** the index-regular source the loop projects; the
  `to_sv_with_modules` function-decl section is the structural template; the
  `function_emit.rs`/`soft_union.rs` `annotate_*` + `Module` `BTreeSet<NodeId>`
  marker + `function_emit_gate` defensive re-check are the mechanism mirrored.
  Resolved all five `.4a` points — (1) first-cut selection = a `{N{x}}`
  replication `Concat` with a **1-bit lane** (⇒ `W == N`, `assign <wire>[gi] =
  <x>` byte-faithful; the common one-hot `{W{sel}}` broadcast idiom), mutually
  exclusive with function-emit (which accepts `Concat`) by running the
  generate-loop annotation **after** function-emit and excluding
  `m.function_emit_gates` (the soft_union→function_emit "later pass excludes
  earlier marks" precedent); wider-lane part-select = recorded follow-up, nothing
  retired; (2) **gen-time annotation** `src/ir/generate_loop.rs
  annotate_generate_loop_gates` + `Module.generate_loop_gates`; (3) a
  `generate_loop_gate` accessor + a `genvar <wire>__gi; generate for (…;
  …<gi>++) begin : <wire>__gen assign <wire>[<gi>] = <x>; end endgenerate` block
  after the function-decl section + the per-gate assign-loop `continue` that
  suppresses the inline `{N{x}}`; (4) `Config::generate_loop_emit_prob` (default
  `0.0`, config-file-only — no CLI flag, the `function_emit_prob` precedent ⇒
  byte-identical, snapshots untouched; a `num_emitted_generate_loops` metric in
  `.4b` would bump schema `1.8→1.9`); (5) `tool_matrix --generate-loop-gate` +
  `ScenarioSet::GenerateLoopSweep` + `ModuleReport.emitted_generate_loop` +
  `saw_generate_loop_emit` (full Verilator + both Yosys plan — a `generate for`
  is universally synthesizable, unlike the Verilator-only `union soft` up-opt).
  Flagged the load-bearing gate-shape risk (the corpus must actually emit `{N{x}}`
  1-bit replications) for `.4b`; recorded the `.4b` impl shape (single slice or
  pre-split `.4b.1`/`.4b.2`/`.4b.3`). Self-checks clean (`mdbook build` +
  `check_knowledge_map` + `check_memory_architecture` + `book_examples` 3/3).
  Frontier → `.4b`.
- `2026-06-16`: **`.3` landed — picked the second structured surface
  (`generate for`); decision `0013`.** Design/decision leaf, no source change.
  By explicit owner steer (*"structured emission: next surface"* → `generate`),
  the second richer-structured surface is a default-off, valid-by-construction
  **`generate for` loop** emit-projection of an existing **replication** (leading
  source = a `{N{x}}` `Concat`, index-regular by construction, rendered as a
  single-level `generate for (genvar gi …) assign <wire>[gi] = <x>;` that unrolls
  to exactly the inline replication). Chosen over `task` (also clean for *simple
  combinational void* tasks on the current toolchain — the leading **future**
  candidate, `.5+`, not retired), `interface`/`modport` (still weak Yosys synth),
  and constant-predicate `generate if` (dead untaken branch; frontend lane already
  has it). **Empirically grounded this session:** the DUT emitter has no
  `generate`/`genvar`; the frontend lane has `generate if`; a representative
  `generate for` + a replication→loop projection are accepted warning-clean by
  Verilator 5.046 `-Wall` + both repo Yosys modes + Icarus. Discipline: rules-first
  (mark an already-valid replication node; no generate-then-filter), default-off
  `generate_loop_emit_prob` (proposed; default `0.0`) ⇒ byte-identical, no new IR
  node / no new whole-module behaviour. Downstream gate (`saw_generate_loop_emit`).
  Decision `0013` + `INDEX.md` row + KM card
  `structured-emission-second-surface-generate-loop`. Tree split `.3` (done) +
  `.4` (impl; pre-split `.4a`/`.4b`) + future (`.5+`). Frontier → `.4`. Self-checks
  clean (`mdbook build` + `check_knowledge_map` + `check_memory_architecture`).
- `2026-06-16`: **`.2b.2c` landed — the user-facing closeout; the first
  structured surface is delivered end-to-end.** Docs-only / DUT byte-identical.
  A new `How It Works` book chapter `book/src/structured-emission.md` (added to
  `book/src/SUMMARY.md` after `factorization.md`) teaches the combinational
  `function automatic` surface: the emit-time-projection concept (the
  `soft_union`/aggregate precedent), a **byte-verified seed-42 before/after**
  (`function_emit_prob` 0.0→1.0 adds the `add_0__f` decl and rewrites *only*
  that gate's `assign` to a call — everything else byte-identical), the
  single-gate first-cut rule, the `Slice`/structured-selector exclusions
  (`Slice` = `-Wall UNUSEDSIGNAL` on a full-width param; nothing retired),
  duplicate-operand positional params (`concat_0__f(case_mux_0, case_mux_0)`),
  combinational-only (a flop `Q` is a leaf), the why-this-surface-first
  rationale, and the metric + `tool_matrix --function-emit-gate` proof, plus a
  skip-sentinelled repro `bash` block. The `function_emit_prob` knob is added to
  the canonical knob reference `book/src/knobs.md` (new `### Structured emission`
  subsection), `USER_GUIDE.md` (after the `soft_union_slice_prob` config-knob
  section), and the README "Current CLI truth" (a dedicated bullet before the
  `tool_matrix --function-emit-gate` gate bullet) — all documenting it
  **accurately as a config-file-only knob** (no CLI flag, like
  `soft_union_slice_prob`/`aggregate_prob`; the `.2b.2b` gate README/USER_GUIDE
  entries already landed). A new Knowledge Map how-to card
  `docs/knowledge/combinational-function-emit.md` carries how-to question keys
  distinct from decision `0012`'s conceptual keys + a validated `reverify`
  command; KM regenerated 36→37 facts / 272→286 keys. `mdbook build` +
  `check_knowledge_map` + `check_memory_architecture` +
  `cargo test --test book_examples` 3/3 all green. Closes `.2b.2` / `.2b` /
  `.2`; the tree stays `active` as an open-ended lane with **no current
  frontier** (future `task`/nested `generate`/`interface`/`modport` surfaces are
  `.3+`, each its own decision). Nothing retired.
- `2026-06-16`: **`.2b.2b` landed — the repo-owned `tool_matrix --function-emit-gate`.**
  `src/bin/tool_matrix.rs` gains a new gate templated on `--signoff-knob-sweep-gate`
  (scaffolding) + the `union soft` up-opt (emitted-construct detection):
  `--function-emit-gate` CLI flag + `ScenarioSet::FunctionEmitSweep` +
  `MatrixReport.function_emit_gate` + `build_function_emit_sweep_scenarios` /
  `function_emit_focus_config` (one comb-only `function_emit_prob=1.0` DUT × three
  construction strategies) + `ModuleReport.emitted_combinational_function` (from
  `sv_text.contains("function automatic")`) + `CoverageSummary.saw_combinational_function_emit`
  (lit when an emitted-function module is Verilator-success + clean-Yosys; the
  fact requires BOTH tools because — unlike the Verilator-only `union soft`
  up-opt — a synthesizable function is universally accepted, so the gate runs the
  full tool plan) + merge + an early-return `compute_coverage_gaps` arm + 5
  cargo-portable proofs + 6 `ModuleReport` fixture updates. A
  `clippy::explicit_counter_loop` was fixed via `.enumerate()`. No schema bump
  (harness-only). Default `function_emit_prob = 0.0` emission byte-identical
  (snapshots 6/6). Banked downstream-clean `/tmp/anvil-function-emit-gate-r1`
  (3 scenarios / 12 modules / 608 emitted functions / `coverage_gaps = []` /
  `12/0` Verilator + both Yosys modes + Icarus compile). Frontier → `.2b.2c`
  (the `anvil`-side `function_emit_prob` knob user docs + book chapter; the
  `tool_matrix` gate README/USER_GUIDE entries already landed here).
- `2026-06-16`: **`.2b.2` pre-split + `.2b.2a` landed.** Pre-split `.2b.2` into
  `.2b.2a` (metric + schema), `.2b.2b` (the `tool_matrix` gate), `.2b.2c` (book /
  USER_GUIDE / KM / README closeout) — the metric is a `Metrics` field surfaced in
  introspection (a precedented schema MINOR bump), the gate is a large/fragile
  harness change, and the user-facing docs are the closeout. **`.2b.2a` landed:**
  `Metrics::num_emitted_combinational_functions` (`= m.function_emit_gates.len()`,
  computed in `metrics::compute()`; `#[serde(default)]`) surfaced in introspection
  `module_metrics` ⇒ schema MINOR bump `1.7 → 1.8` (`SCHEMA_VERSION` + the 9
  `schema_version` test assertions + the schema-doc `1.7 → 1.8` changelog entry +
  the README / USER_GUIDE / 5 book `agent-mcp.md` current-output refs + the stale
  `CODEBASE_ANALYSIS` envelope line, which was frozen at `1.4`). The metric bumps
  the schema (new derived `Metrics` field — the `1.0 → 1.1` `bisimulation_flops_merged`
  precedent) whereas the `.2b.1` knob rode the version (default-off prob-knob
  precedent). Lib proof + end-to-end introspect (default `0`, forced `1256`); 468
  lib tests / snapshots 6/6 / mdbook build all green; default-off / DUT
  byte-identical. Frontier → `.2b.2b`.
- `2026-06-16`: **`.2b.1` live surface landed** — the first richer-structured
  emit surface (decision `0012`) goes live, default-off / DUT byte-identical.
  `Config::function_emit_prob` (default `0.0`) + `Module.function_emit_gates:
  BTreeSet<NodeId>` (emitter-surface annotation only; identity/CSE/validators
  untouched; disjoint from `soft_union_slice_gates`) + new
  `src/ir/function_emit.rs` `annotate_function_emit_gates(m, rng, prob)` (gen-time
  mark, the `soft_union.rs` precedent; skips `param_env` modules) + call-site
  rolls in `generate_module`/`generate_design` (after the soft_union pass) +
  `src/emit/sv.rs` rendering: a `function automatic` decl section + a call-site
  substitution (`assign <wire> = <wire>__f(<operand refs>);`), with
  `render_gate_function_body` the positional behaviour-preserving counterpart of
  `render_gate`. **First-cut scoping refinement: `Slice` excluded** — a forced
  `function_emit_prob=1.0` `verilator -Wall` sweep flagged `UNUSEDSIGNAL` on every
  `slice_*__f` param (a bit-select reads only a sub-range of its operand, so a
  full-width param leaves bits unused); `Slice` still emits inline (nothing
  retired), a slice-aware projection passing only `src[hi:lo]` is a recorded
  follow-up. **No schema bump** (the default-off prob-knob precedent —
  `soft_union`/`aggregate`/`memory`/`fsm`/`multi_clock` all rode the existing
  `schema_version` via `#[serde(default)]`; only the `sv_version` enum took a
  dedicated bump; introspect tests stay green at `1.7`). 9 lib proofs; snapshots
  6/6 byte-identical; forced sweep (5 seeds) clean across Verilator `--lint-only`
  + both Yosys modes + Icarus (`/tmp/anvil-fe-r2/`). Frontier → `.2b.2` (the
  repo-owned gate + metric + coverage fact + book/USER_GUIDE/KM closeout).
- `2026-06-16`: **`.2a` design-detail landed** (no source change) — pinned the
  first concrete cut of the combinational `function automatic` surface, grounded in
  the real `to_sv_with_modules` + the `soft_union.rs` gen-time-annotation precedent.
  First cut = a **single-gate "operand function"** (the minimal cone: wrap one
  `Gate` as a `function automatic` of its direct operands — operands are already
  module wires/literals ⇒ **zero** sharing/scoping hazard; the multi-level cone body
  is a recorded follow-up). Mechanism: a gen-time `annotate_function_emit_gates(m,
  rng, prob)` pass (new `src/ir/function_emit.rs`) marks `Module.function_emit_gates:
  BTreeSet<NodeId>` (an emitter-surface annotation only — flat IR / validators / CSE
  / `canonical_module_signature` untouched), call-site-guarded on `function_emit_prob
  > 0.0` ⇒ default byte-identical. Rendering: a `<wire>__f` `function automatic
  logic[W-1:0]` decl with **positional** params (handles duplicate operands) + a body
  = `op` over the param names (a `render_gate` positional variant) + the call-site
  `assign <wire> = <wire>__f(<operand refs>);` — behaviour-preserving by
  construction. Downstream gate = Verilator + both Yosys modes + Icarus warning-clean
  on a `saw_combinational_function_emit` fact (+ a `num_emitted_combinational_functions`
  metric). Pre-split `.2b` → `.2b.1` (live surface, **new frontier**) + `.2b.2` (gate
  + closeout). Rejected: a multi-level cone body in the first cut, a pure emit-time
  pass, node-id operand→param mapping. Self-checks clean; baseline `cargo check`
  clean.
- `2026-06-16`: **Activated by explicit owner directive** (the owner selected this
  lane after `SEMANTIC-INTROSPECTION-EXPANSION` delivered all four query kinds).
  `.1` design landed — decision `0012`: the first richer-structured surface is a
  default-off, valid-by-construction combinational `function automatic`
  emit-projection of an existing combinational cone (the `output_support`
  support-leaf boundary gives its parameter list), chosen over `interface`/`modport`
  (weak Yosys synth support) and nested `generate` (bigger blast radius); opt-in
  `function_emit_prob` (default `0.0`) ⇒ byte-identical; downstream gate proves
  Verilator + both Yosys modes + Icarus accept it (`saw_combinational_function_emit`).
  Activated the tree (`proposed → active`), split `.1`/`.2`/future, pre-split `.2`
  → `.2a` (design-detail, **frontier**) + `.2b` (impl). No source change;
  self-checks clean.
- `2026-06-15`: Created and registered `proposed` (owner-directed sibling lane).
