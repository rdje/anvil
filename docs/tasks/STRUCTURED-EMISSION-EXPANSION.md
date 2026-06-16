# STRUCTURED-EMISSION-EXPANSION: richer structured SystemVerilog surfaces

## Metadata

- Tree ID: `STRUCTURED-EMISSION-EXPANSION`
- Status: `active`
- Roadmap lane: `Capability / breadth — richer structured emission (ROADMAP steering gap 1)`
- Created: `2026-06-15`
- Last updated: `2026-06-16` (**`.2b.2` pre-split + `.2b.2a` landed** — `.2b.2` split into `.2b.2a` (metric + schema), `.2b.2b` (the `tool_matrix` gate, **frontier**), `.2b.2c` (book/USER_GUIDE/KM/README closeout). `.2b.2a` added `Metrics::num_emitted_combinational_functions` (`= function_emit_gates.len()`) ⇒ introspection schema MINOR bump `1.7 → 1.8` (the metric bumps; the `.2b.1` knob rode the version); 468 lib tests / snapshots 6/6 / mdbook all green; default-off / DUT byte-identical. Prior: **`.2b.1` live surface** — the first richer-structured emit surface goes live: `Config::function_emit_prob` + `Module.function_emit_gates` + new `src/ir/function_emit.rs` `annotate_function_emit_gates` (gen-time mark, the `soft_union.rs` precedent) + two generator call-site rolls (after soft_union) + `to_sv_with_modules` `<wire>__f` `function automatic` decl/positional-body/call rendering + 9 lib proofs. `Slice` excluded from the first cut (`-Wall UNUSEDSIGNAL` on a full-width param; still emitted inline, nothing retired; slice-aware projection = follow-up). No schema bump (default-off prob-knob precedent). Default-off / DUT byte-identical (snapshots 6/6); forced `function_emit_prob=1.0` sweep clean across Verilator `--lint-only` + both Yosys modes + Icarus (`/tmp/anvil-fe-r2/`). Frontier → `.2b.2` (the repo-owned gate + metric + coverage fact + book/USER_GUIDE/KM closeout). Prior: `.2a` design-detail; `.1` design — decision `0012`.)
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
  Goal: `Richer structured synthesizable SV surfaces (functions / interfaces / nested generate), valid-by-construction.`
  Children: `STRUCTURED-EMISSION-EXPANSION.1`, `STRUCTURED-EMISSION-EXPANSION.2`

- ID: `STRUCTURED-EMISSION-EXPANSION.1`
  Status: `done`
  Goal: `Design/decision leaf: inventory candidate structured surfaces (function/task, interface/modport, nested generate), pick the first concrete synthesizable + downstream-clean one, define its valid-by-construction discipline + opt-in knob + downstream gate, and split the tree — before any code.`
  Acceptance: `A decision record naming the first surface, its construction discipline, and its downstream gate; no source change; self-checks clean.`
  Result: `Decision 0012. The first richer-structured surface is a default-off, opt-in, valid-by-construction combinational function automatic emitted as a behaviour-preserving projection of an existing combinational cone: a selected Gate node + its fan-in (stopping at the output_support support-leaf boundary — primary inputs / flop Qs / instance outputs / constants) rendered as function automatic logic[W-1:0] <name>(...) whose parameter list is the cone's support leaves and whose body is the straight-line evaluation of the cone's internal gates, returning the root; the use site becomes a call. Chosen over interface/modport (weak/version-inconsistent Yosys synth support ⇒ fails the both-Yosys-modes-clean bar) and nested generate (bigger emitter blast radius) and task (procedural/multi-output — a combinational function is the simpler first cut). Discipline: rules-first (wraps an already-valid cone; selection at construction time, never generate-then-filter); default-off function_emit_prob (default 0.0) ⇒ byte-identical, snapshots untouched; no new IR node / no new computed truth (the soft_union/aggregate emit-projection precedent). Downstream gate: a repo-owned gate proving Verilator + both Yosys modes + Icarus accept the emitted functions warning-clean, gated on a saw_combinational_function_emit coverage fact. Rejected: interface/modport first, nested generate first, task first, a semantic IR Function node, generate-then-filter, changing the default. Split into .1 (done) + .2 (impl) + future kinds (.3+: task, nested generate, interface/modport). Pre-split .2 → .2a (design-detail) + .2b (impl).`
  Verification: `done`
  Commit: `done`

- ID: `STRUCTURED-EMISSION-EXPANSION.2`
  Status: `active`
  Goal: `Implement the first structured surface (the combinational function automatic emit-projection) per decision 0012: the function_emit_prob knob + the rules-first cone selection + the emitter rendering (function automatic decl + call site) + the downstream-clean gate + book/USER_GUIDE/KM. Default-off / DUT byte-identical.`
  Children: `STRUCTURED-EMISSION-EXPANSION.2a`, `STRUCTURED-EMISSION-EXPANSION.2b`

- ID: `STRUCTURED-EMISSION-EXPANSION.2a`
  Status: `done`
  Goal: `Design-detail leaf (no source): ground the combinational function automatic surface in the real src/emit/sv.rs to_sv_with_modules + the soft_union.rs / aggregate_layout emit-projection precedents + src/config.rs. Pin: (1) the cone-selection rule (which Gate nodes qualify; size/depth bounds so the function is non-trivial yet bounded; how it stays rules-first); (2) whether selection is a generation-time annotation (the soft_union.rs / aggregate_layout precedent — likely, so the IR carries the choice deterministically and emission projects it) or a pure emit-time pass; (3) the function signature + body rendering (parameter list = the cone's support leaves; local decls vs single return expr; width/logic typing); (4) the function_emit_prob knob semantics + default 0.0 byte-identical contract; (5) the downstream-gate scenario shape (saw_combinational_function_emit). DEVELOPMENT_NOTES design-detail entry + the .2b impl shape.`
  Acceptance: `A DEVELOPMENT_NOTES design-detail entry resolving the five points grounded in real code; tree split recorded; no source change; docs/workflow self-checks clean.`
  Result: `Done. DEVELOPMENT_NOTES design-detail entry resolves all five points, grounded in a fresh read of src/emit/sv.rs (to_sv_with_modules gate-emission loop + build_names/node_ref/render_gate/param_width_decl_w), src/ir/soft_union.rs + Module.soft_union_slice_gates (the gen-time-annotation precedent), and the aggregate_layout projection. (1) First-cut cone selection = the MINIMAL cone: wrap ONE selected Node::Gate as a function automatic of its DIRECT operands (operands are already module wires/literals ⇒ zero sharing/scoping hazard; the multi-level-cone body with private-internal locals is a recorded follow-up). Candidate = a non-structured (not CaseMux/CasezMux/ForFold), non-soft_union-marked Gate with >= 1 operand; selection rules-first at gen time. (2) Gen-time annotation (the soft_union.rs precedent): a new src/ir/function_emit.rs annotate_function_emit_gates(m, rng, prob) rolls gen_bool(prob) per candidate into a new Module.function_emit_gates: BTreeSet<NodeId> (emitter-surface annotation only — flat IR/validators/CSE/canonical_signature untouched); call-site guard on prob > 0.0 ⇒ default byte-identical. (3) Signature = function automatic logic[W-1:0] <wire>__f(positional input logic[Wi-1:0] ai,...); body = op over the positional param names (a render_gate-parallel positional variant — positional, not node-id-mapped, to handle duplicate operands); call site = assign <wire> = <wire>__f(node_ref(o0),...); behaviour-preserving by construction. (4) Config::function_emit_prob (default 0.0) beside aggregate_prob/soft_union_slice_prob ⇒ default byte-identical, snapshots untouched; surfaced in dump-config/introspect (a Config-field schema MINOR bump, confirmed in .2b). (5) Downstream gate = Verilator + both Yosys modes + Icarus warning-clean on a saw_combinational_function_emit fact (+ a num_emitted_combinational_functions metric), shape in .2b.2. Pre-split .2b → .2b.1 (the live surface: knob + annotation + Module field + emitter rendering + lib proofs + Verilator lint) + .2b.2 (the repo-owned gate + metric + coverage fact + book/USER_GUIDE/KM). Rejected: multi-level cone body in the first cut, a pure emit-time pass, node-id operand→param mapping.`
  Verification: `done`
  Commit: `done`

- ID: `STRUCTURED-EMISSION-EXPANSION.2b`
  Status: `active`
  Goal: `Implement the .2a design: the function_emit_prob knob, the rules-first single-gate selection (gen-time annotation src/ir/function_emit.rs + Module.function_emit_gates), the function automatic emitter rendering (decl + positional-param body + call site) in to_sv_with_modules, lib proofs (behaviour-preserving + selected-by-construction + default-off byte-identical + CSE/canonical-signature untouched), the downstream-clean gate (Verilator + both Yosys modes + Icarus + the saw_combinational_function_emit fact + a num_emitted_combinational_functions metric), and book/USER_GUIDE/KM closeout. Default-off / DUT byte-identical (snapshots untouched).`
  Children: `STRUCTURED-EMISSION-EXPANSION.2b.1`, `STRUCTURED-EMISSION-EXPANSION.2b.2`

- ID: `STRUCTURED-EMISSION-EXPANSION.2b.1`
  Status: `done`
  Goal: `The live first-cut surface: Config::function_emit_prob (default 0.0, serde default) + Module.function_emit_gates: BTreeSet<NodeId> + src/ir/function_emit.rs annotate_function_emit_gates(m, rng, prob) (collect non-structured/non-soft_union Gate candidates, roll gen_bool(prob), mark) + the generator call-site roll (guarded prob > 0.0) + the to_sv_with_modules rendering (a function automatic decl section + positional-param body via a render_gate positional variant + the call-site assign) + lib proofs (a marked gate emits a behaviour-preserving function + call; default-off byte-identical; the mark leaves CSE/canonical_module_signature untouched) + a forced-knob Verilator --lint-only spot-check. Default-off / DUT byte-identical (snapshots 6/6).`
  Acceptance: `cargo check/clippy(-D warnings)/fmt clean; cargo test --lib green incl. the new function_emit proofs; cargo test --test snapshots 6/6 byte-identical (default-off); a forced function_emit_prob=1.0 sample lints clean under Verilator. Committed through COMMIT.md with the leaf id.`
  Result: `Done. Config::function_emit_prob (default 0.0, default_function_emit_prob() serde default; added to the Default impl + the 0.0..=1.0 validation list) + Module.function_emit_gates: BTreeSet<NodeId> (Default-empty; emitter-surface annotation only — flat IR / validators / CSE / canonical_module_signature untouched, disjoint from soft_union_slice_gates) + new src/ir/function_emit.rs annotate_function_emit_gates(m, rng, prob) (gen-time mark; the soft_union.rs precedent; rolls gen_bool(prob) per qualifying candidate; skips param_env modules) + call-site rolls in BOTH generate_module and generate_design guarded on prob > 0.0, run AFTER the soft_union pass (so union soft marks are excluded) + src/emit/sv.rs rendering: a function automatic decl section (after the wire decls, before the gate assigns) emitting per marked gate function automatic logic[W-1:0] <wire>__f(input logic[Wi-1:0] a0,...); <wire>__f = <op over a0..a{n-1}>; endfunction, and a call-site substitution making the marked gate's assign become assign <wire> = <wire>__f(<operand refs>). Helpers: function_emit_gate (marked + defensively-revalidated lookup), render_gate_function_body (positional behaviour-preserving counterpart of render_gate), render_gate_function_decl, render_gate_function_call. FIRST-CUT SCOPING REFINEMENT: Slice EXCLUDED from candidacy — a forced function_emit_prob=1.0 verilator -Wall sweep flagged UNUSEDSIGNAL on every slice_*__f param (a bit-select reads only a sub-range of its operand, so a full-width param leaves bits unused); Slice still emits inline (NOTHING RETIRED), a slice-aware projection that passes only src[hi:lo] is a recorded follow-up. All other ops use operands in full and are warning-clean. NO schema bump (default-off prob-knob precedent: soft_union/aggregate/memory/fsm/multi_clock all rode the existing schema_version via #[serde(default)]; only the sv_version enum took a dedicated 1.1->1.2 bump; introspect schema tests stay green at 1.7). 9 lib proofs (mark/skip/structured/slice/soft-union/param-env exclusions + identity-and-node-count-untouched + end-to-end emit + duplicate-operand positional params).`
  Verification: `cargo check --all-targets clean; cargo clippy --all-targets -- -D warnings clean; cargo fmt --all --check clean; cargo test --lib 467 passed / 2 ignored (incl. 9 new function_emit proofs; introspect schema_version 1.7 + umbrella DUT-byte-identical still green); cargo test --test snapshots 6/6 byte-identical (default-off). Forced function_emit_prob=1.0 sweep (5 seeds: 1/7/42/100/2024, 830-1299 functions each, banked /tmp/anvil-fe-r2/): Verilator --lint-only 5/5 CLEAN (repo bar), 0 __f-param -Wall warnings (slice fix resolved every change-introduced warning; residual -Wall UNUSEDSIGNAL on ordinary gate wires is pre-existing — the function-emit-OFF baseline has 20), Yosys without-abc 5/5 + with-abc 5/5, Icarus iverilog -g2012 5/5 CLEAN.`
  Commit: `done`

- ID: `STRUCTURED-EMISSION-EXPANSION.2b.2`
  Status: `active`
  Goal: `The repo-owned downstream gate + closeout for the combinational function automatic surface: a num_emitted_combinational_functions metric + a saw_combinational_function_emit coverage fact + a tool_matrix gate proving Verilator + both Yosys modes + Icarus accept the emitted functions warning-clean + book/USER_GUIDE/KM/README closeout. Default-off / DUT byte-identical. Pre-split (2026-06-16) into .2b.2a (metric + schema bump) + .2b.2b (the tool_matrix gate + coverage fact) + .2b.2c (docs closeout) — the metric is a Metrics field surfaced in introspection (schema MINOR bump, like 1.0->1.1 bisimulation_flops_merged); the tool_matrix gate is a large, fragile change (flag + ScenarioSet + config builder + coverage fact + detection + merge + gap enforcement + many ModuleReport/Cli test fixtures) that warrants its own focused slice; the book chapter + USER_GUIDE + KM + README CLI-truth entry are the user-facing closeout.`
  Children: `STRUCTURED-EMISSION-EXPANSION.2b.2a`, `STRUCTURED-EMISSION-EXPANSION.2b.2b`, `STRUCTURED-EMISSION-EXPANSION.2b.2c`

- ID: `STRUCTURED-EMISSION-EXPANSION.2b.2a`
  Status: `done`
  Goal: `The num_emitted_combinational_functions metric: add Metrics::num_emitted_combinational_functions (usize, #[serde(default)]) computed in metrics::compute() as m.function_emit_gates.len(); it surfaces in introspection module_metrics (the SCHEMA-DERIVED projection), so bump the introspection schema MINOR 1.7 -> 1.8 (SCHEMA_VERSION const + the 9 "1.7" test assertions in src/introspect/mod.rs + src/mcp/mod.rs + the docs/AGENT_INTROSPECTION_SCHEMA.md changelog/§7 lines). A lib proof that a module with marked function_emit_gates reports the count. Default-off / DUT byte-identical (a post-hoc Metrics field changes no emitted RTL; snapshots untouched).`
  Acceptance: `cargo check/clippy(-D warnings)/fmt clean; cargo test --lib green incl. the metric proof + the schema_version 1.8 assertions; cargo test --test snapshots 6/6 byte-identical; the schema doc records the 1.7 -> 1.8 additive MINOR bump. Committed through COMMIT.md with the leaf id.`
  Result: `Done. Metrics::num_emitted_combinational_functions: usize (#[serde(default)]) added to src/metrics.rs, computed in metrics::compute() as m.function_emit_gates.len() (a post-hoc structural count of an emitter-surface annotation; reads 0 by default, the configured count when function_emit_prob fired). Surfaced in introspection module_metrics (Metrics is the exact serde projection), so SCHEMA_VERSION bumped 1.7 -> 1.8 in src/introspect/mod.rs. The metric BUMPS the schema (new derived Metrics field — the 1.0->1.1 bisimulation_flops_merged precedent) whereas the .2b.1 knob did NOT (default-off prob-knob rides request.knobs via #[serde(default)]). Bumped all current-output schema refs to 1.8: the 9 schema_version assertions (src/introspect/mod.rs + src/mcp/mod.rs), the schema doc (1.7->1.8 changelog entry + the defines/lockstep/checklist lines), README (--introspect + analyze), USER_GUIDE (--introspect), the 5 book agent-mcp.md example JSONs, and the CODEBASE_ANALYSIS envelope line (which had drifted, frozen at 1.4). Historical "landed at schema X" attributions left intact. Lib proof metrics_count_emitted_combinational_functions (unmarked 0, marked 1).`
  Verification: `cargo clippy --all-targets -- -D warnings clean; cargo fmt --all --check clean; cargo test --lib 468 passed / 2 ignored (the new metric proof + all schema_version assertions green at 1.8); cargo test --test snapshots 6/6 byte-identical (default-off; metric changes no RTL); end-to-end --introspect: default seed => schema_version 1.8 + num_emitted_combinational_functions 0; forced function_emit_prob=1.0 => 1.8 + 1256; mdbook build book OK.`
  Commit: `done`

- ID: `STRUCTURED-EMISSION-EXPANSION.2b.2b`
  Status: `pending`
  Goal: `The repo-owned tool_matrix gate: a saw_combinational_function_emit coverage fact + a --function-emit-gate flag (or a ScenarioSet) forcing function_emit_prob=1.0 over comb-only DUTs across the three construction strategies + a ModuleReport.emitted_combinational_function detection (from emitted SV or num_emitted_combinational_functions) + coverage-gap enforcement, proving Verilator + both Yosys modes + Icarus accept the emitted functions warning-clean. Bank a clean report. Default-off / DUT byte-identical. Template: --signoff-knob-sweep-gate; precedent for emitted-construct detection: the soft_union emitted_soft_union_overlay / saw_sv_version_2023_soft_union_upopt path. (Large, fragile change — many ModuleReport/Cli test fixtures must gain the new field.) Forced-sweep evidence already banked at /tmp/anvil-fe-r2/ (5 seeds, 3 tools, both Yosys modes).`
  Acceptance: `cargo check/clippy(-D warnings)/fmt clean; the repo-owned gate is banked clean (Verilator + both Yosys + Icarus) with saw_combinational_function_emit lit and coverage_gaps=[]; snapshots 6/6 byte-identical; committed through COMMIT.md with the leaf id.`
  Verification: `pending`
  Commit: `pending`

- ID: `STRUCTURED-EMISSION-EXPANSION.2b.2c`
  Status: `pending`
  Goal: `The user-facing closeout: a book chapter (or section) on structured emission / the combinational function automatic surface (under "How It Works" or "Reference") with examples; the USER_GUIDE function_emit_prob knob entry; the README "Current CLI truth" knob entry; and a Knowledge Map card if a durable how-to is warranted (decision 0012 already carries answers:). Default-off / DUT byte-identical (docs-only).`
  Acceptance: `book builds (mdbook build book); USER_GUIDE + README updated; KM regenerated + check_knowledge_map clean; self-checks clean; committed through COMMIT.md with the leaf id.`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

**Active frontier: `STRUCTURED-EMISSION-EXPANSION.2b.2b`** (the repo-owned
`tool_matrix` gate). `.2b.2` was pre-split (`2026-06-16`) into `.2b.2a` (metric +
schema — **done**), `.2b.2b` (the `tool_matrix` gate — frontier), `.2b.2c` (docs
closeout). `.1` (decision `0012`), `.2a` (design-detail), `.2b.1` (the live
combinational `function automatic` surface), and `.2b.2a` (the metric + schema
`1.8`) are done. Nothing retired.

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `STRUCTURED-EMISSION-EXPANSION.2b.2b` | `pending` | The repo-owned `tool_matrix` gate: `saw_combinational_function_emit` + a `--function-emit-gate` flag/ScenarioSet forcing `function_emit_prob=1.0` over comb-only DUTs + `ModuleReport.emitted_combinational_function` detection (from emitted SV `"function "` or `num_emitted_combinational_functions > 0`) + coverage-gap enforcement; bank clean across Verilator + both Yosys + Icarus. Template: `--signoff-knob-sweep-gate`; precedent: the soft_union `emitted_soft_union_overlay` detection. **Large, fragile change** (many `ModuleReport`/`Cli` test fixtures must gain the new field) — a focus-intensive slice; a fresh session is reasonable here. Forced-sweep evidence already banked at `/tmp/anvil-fe-r2/`. |
| 2 | `STRUCTURED-EMISSION-EXPANSION.2b.2c` | `pending` | The user-facing closeout: a book chapter/section on the combinational `function automatic` surface (examples) + the USER_GUIDE `function_emit_prob` entry + the README "Current CLI truth" knob entry + a KM card if warranted. Docs-only / byte-identical. |
| — | `STRUCTURED-EMISSION-EXPANSION.2b.2a` | `done` | The metric `Metrics::num_emitted_combinational_functions` (= `m.function_emit_gates.len()`) surfaced in introspection `module_metrics` ⇒ schema MINOR bump `1.7 -> 1.8`. Lib proof; 468 lib tests + snapshots 6/6 + mdbook all green; end-to-end introspect default `0` / forced `1256`. Precedented (1.0->1.1 `bisimulation_flops_merged`). |
| — | `STRUCTURED-EMISSION-EXPANSION.2b.1` | `done` | Live surface delivered: `Config::function_emit_prob` + `Module.function_emit_gates` + `src/ir/function_emit.rs` (`annotate_function_emit_gates`) + the gen-time call-site rolls + the `to_sv_with_modules` `<wire>__f` `function automatic` decl/positional-body/call rendering + 9 lib proofs + a forced-knob downstream sweep. **`Slice` excluded** (a bit-select uses only a sub-range ⇒ `-Wall UNUSEDSIGNAL` on a full-width param; still emitted inline, nothing retired). No schema bump (default-off prob-knob precedent). Default-off / DUT byte-identical (snapshots 6/6). |
| — | `STRUCTURED-EMISSION-EXPANSION.2a` | `done` | Design-detail (no source): pinned the first-cut single-gate "operand function" (minimal cone ⇒ zero sharing hazard), the gen-time annotation (`Module.function_emit_gates` + `annotate_function_emit_gates`, the `soft_union.rs` precedent), the `function automatic` signature/positional-body/call rendering, the `function_emit_prob` knob, and the downstream gate. Pre-split `.2b` → `.2b.1`/`.2b.2`. |
| — | `STRUCTURED-EMISSION-EXPANSION.1` | `done` | Decision `0012`: picked the combinational `function automatic` emit-projection as the first surface (over interface/modport + nested generate), with its valid-by-construction discipline, opt-in `function_emit_prob`, and downstream gate. Split `.1`/`.2`/future. No source change. |

## Decisions

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
  interface/modport vs nested generate) — resolved by `.1` when activated.

## Blockers

- None (not active by choice, not dependency).

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-06-16` | `STRUCTURED-EMISSION-EXPANSION.2b.2a` | **Metric + schema bump** (`src/metrics.rs` `num_emitted_combinational_functions` + `src/introspect/mod.rs` `SCHEMA_VERSION` `1.7→1.8` + the 9 `schema_version` test assertions + the schema doc + README/USER_GUIDE/book current-output refs + the stale `CODEBASE_ANALYSIS` envelope line). `cargo clippy --all-targets -- -D warnings` clean; `cargo fmt --all --check` clean; `cargo test --lib` **468 passed** / 2 ignored (new metric proof + all `schema_version` assertions green at `1.8`); `cargo test --test snapshots` **6/6 byte-identical** (default-off). End-to-end `--introspect`: default ⇒ `schema_version "1.8"` + metric `0`; forced `function_emit_prob=1.0` ⇒ `1.8` + `1256`. `mdbook build book` OK. | `done` |
| `2026-06-16` | `STRUCTURED-EMISSION-EXPANSION.2b.1` | **Live emitter change** (`src/config.rs` knob + `src/ir/types.rs` `Module.function_emit_gates` + new `src/ir/function_emit.rs` annotate pass + `src/gen/mod.rs` two call-site rolls + `src/emit/sv.rs` `function automatic` decl/body/call rendering). `cargo check --all-targets` clean; `cargo clippy --all-targets -- -D warnings` clean; `cargo fmt --all --check` clean; `cargo test --lib` 467 passed / 2 ignored (incl. 9 new `function_emit` proofs; introspect `schema_version` 1.7 + `umbrella` DUT-byte-identical still green); `cargo test --test snapshots` **6/6 byte-identical** (default-off). Forced `function_emit_prob=1.0` sweep (5 seeds 1/7/42/100/2024, 830–1299 functions each, `/tmp/anvil-fe-r2/`): Verilator `--lint-only` **5/5 CLEAN**, **0** `__f`-param `-Wall` warnings (`Slice` excluded; residual `-Wall UNUSEDSIGNAL` is pre-existing — OFF baseline has 20), Yosys without-abc **5/5** + with-abc **5/5**, Icarus `iverilog -g2012` **5/5 CLEAN**. | `done` |
| `2026-06-16` | `STRUCTURED-EMISSION-EXPANSION.2a` | Design-detail leaf, **no source change** (grounded in a fresh read of `src/emit/sv.rs` — `to_sv_with_modules` gate-emission loop + `build_names`/`node_ref`/`render_gate`/`param_width_decl_w`; `src/ir/soft_union.rs` + `Module.soft_union_slice_gates` — the gen-time-annotation precedent; the `aggregate_layout` projection). `DEVELOPMENT_NOTES.md` design-detail entry (the five points + the `.2b` pre-split): first-cut single-gate "operand function"; gen-time `annotate_function_emit_gates` + `Module.function_emit_gates`; the `<wire>__f` `function automatic` decl + positional-param body + call; `function_emit_prob` (default `0.0` byte-identical); the `saw_combinational_function_emit` gate. `.2b` pre-split → `.2b.1`/`.2b.2`; frontier set to `.2b.1`. `bash scripts/check_memory_architecture.sh` + `bash knowledge-map/scripts/check_knowledge_map.sh` clean. Baseline `cargo check --all-targets` clean (no source touched). | `done` |
| `2026-06-16` | `STRUCTURED-EMISSION-EXPANSION.1` | Design/decision leaf, **no source change** (grounded in a fresh read of `src/emit/sv.rs` `to_sv_with_modules` + the `aggregate_layout` projection + `soft_union_slice_overlay`, `src/ir/soft_union.rs`, and the `aggregate_prob`/`soft_union_slice_prob` default-off emit-projection knobs in `src/config.rs`). Decision `0012` + `INDEX.md` row; tree activated (`proposed → active`); `.2`/`.2a`/`.2b` registered; frontier set to `.2a`. `bash scripts/check_memory_architecture.sh` + `bash knowledge-map/scripts/check_knowledge_map.sh` clean; `KNOWLEDGE_MAP.md` regenerated (decision `0012` carries `answers:` front-matter). Baseline `cargo check --all-targets` clean (from the prior gate; no source touched). | `done` |
| `2026-06-15` | `STRUCTURED-EMISSION-EXPANSION` | Tree registered `proposed` (ownership only, no leaf executed). | `done` (registration) |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `STRUCTURED-EMISSION-EXPANSION.2b.2a` | `STRUCTURED-EMISSION-EXPANSION.2b.2a — emit metric + introspection schema 1.8` | `Metrics::num_emitted_combinational_functions` (= `function_emit_gates.len()`) + introspection schema MINOR bump `1.7 → 1.8` (the metric bumps; the `.2b.1` knob rode the version). Bumped all current-output schema refs (tests + schema doc + README + USER_GUIDE + 5 book example JSONs + the stale `CODEBASE_ANALYSIS` envelope line). Lib proof; default-off / DUT byte-identical (snapshots 6/6). |
| `STRUCTURED-EMISSION-EXPANSION.2b.1` | `STRUCTURED-EMISSION-EXPANSION.2b.1 — combinational function automatic emit-projection (live surface)` | Live emitter change: `function_emit_prob` knob + `Module.function_emit_gates` + `src/ir/function_emit.rs` gen-time mark + two generator call-site rolls (after soft_union) + `to_sv_with_modules` `<wire>__f` `function automatic` decl/positional-body/call rendering + 9 lib proofs. `Slice` excluded from the first cut (`-Wall UNUSEDSIGNAL` on a full-width param; still emitted inline, nothing retired). No schema bump (default-off prob-knob precedent). Default-off / DUT byte-identical (snapshots 6/6); forced sweep clean across Verilator + both Yosys + Icarus (`/tmp/anvil-fe-r2/`). |
| `STRUCTURED-EMISSION-EXPANSION.2a` | `STRUCTURED-EMISSION-EXPANSION.2a — combinational function impl design-detail` | Design-detail (no source): pinned the first-cut single-gate "operand function" (minimal cone ⇒ zero sharing hazard), the gen-time `annotate_function_emit_gates` + `Module.function_emit_gates` annotation (the `soft_union.rs` precedent), the `function automatic` decl/positional-body/call rendering, the `function_emit_prob` knob, and the downstream gate. Pre-split `.2b` → `.2b.1`/`.2b.2`. |
| `STRUCTURED-EMISSION-EXPANSION.1` | `STRUCTURED-EMISSION-EXPANSION.1 — activate lane + decision 0012` | Decision `0012`: the first structured surface is a default-off, valid-by-construction combinational `function automatic` emit-projection of an existing cone (over interface/modport + nested generate). Activated the lane by owner directive; split `.1`/`.2`/future; pre-split `.2` → `.2a`/`.2b`. No source change. |
| `STRUCTURED-EMISSION-EXPANSION` | `SV-VERSION-TARGETING.1 — open SV-version lane + decision 0009` | Registered `proposed` alongside the activated `SV-VERSION-TARGETING` lane. |

## Changelog

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
