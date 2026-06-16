# STRUCTURED-EMISSION-EXPANSION: richer structured SystemVerilog surfaces

## Metadata

- Tree ID: `STRUCTURED-EMISSION-EXPANSION`
- Status: `active`
- Roadmap lane: `Capability / breadth — richer structured emission (ROADMAP steering gap 1)`
- Created: `2026-06-15`
- Last updated: `2026-06-16` (**`.2a` design-detail landed** — pinned the first concrete cut of the combinational `function automatic` surface, grounded in the real `to_sv_with_modules` + the `soft_union.rs` gen-time-annotation precedent: a single-gate "operand function" (the minimal cone ⇒ zero sharing/scoping hazard; the multi-level cone body is a recorded follow-up); a gen-time `annotate_function_emit_gates` pass marking `Module.function_emit_gates: BTreeSet<NodeId>` (emitter-surface annotation only); the `<wire>__f` `function automatic` decl + positional-param body + call rendering; the `function_emit_prob` knob (default `0.0` ⇒ byte-identical); the `saw_combinational_function_emit` downstream gate. Pre-split `.2b` → `.2b.1` (live surface, **frontier**) + `.2b.2` (gate + closeout). Prior: `.1` design — decision `0012`.)
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
  Status: `pending`
  Goal: `The live first-cut surface: Config::function_emit_prob (default 0.0, serde default) + Module.function_emit_gates: BTreeSet<NodeId> + src/ir/function_emit.rs annotate_function_emit_gates(m, rng, prob) (collect non-structured/non-soft_union Gate candidates, roll gen_bool(prob), mark) + the generator call-site roll (guarded prob > 0.0) + the to_sv_with_modules rendering (a function automatic decl section + positional-param body via a render_gate positional variant + the call-site assign) + lib proofs (a marked gate emits a behaviour-preserving function + call; default-off byte-identical; the mark leaves CSE/canonical_module_signature untouched) + a forced-knob Verilator --lint-only spot-check. Default-off / DUT byte-identical (snapshots 6/6).`
  Acceptance: `cargo check/clippy(-D warnings)/fmt clean; cargo test --lib green incl. the new function_emit proofs; cargo test --test snapshots 6/6 byte-identical (default-off); a forced function_emit_prob=1.0 sample lints clean under Verilator. Committed through COMMIT.md with the leaf id.`
  Verification: `pending`
  Commit: `pending`

- ID: `STRUCTURED-EMISSION-EXPANSION.2b.2`
  Status: `pending`
  Goal: `The repo-owned downstream gate + closeout: a saw_combinational_function_emit coverage fact + a num_emitted_combinational_functions metric (structural scan of function_emit_gates) + a tool_matrix scenario (or dedicated bank) proving Verilator + both Yosys modes + Icarus accept the emitted functions warning-clean + book(structured-emission)/USER_GUIDE/KM closeout. Default-off / DUT byte-identical.`
  Acceptance: `cargo check/clippy(-D warnings)/fmt clean; the repo-owned gate is banked clean (Verilator + both Yosys + Icarus) with the coverage fact lit; snapshots 6/6 byte-identical; book/USER_GUIDE + a KM fact; committed through COMMIT.md with the leaf id.`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

**Active frontier: `STRUCTURED-EMISSION-EXPANSION.2b.1`** (the live first-cut
combinational `function automatic` surface). `.1` (decision `0012`) and `.2a`
(design-detail) are done. Nothing retired.

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `STRUCTURED-EMISSION-EXPANSION.2b.1` | `pending` | Live surface: `Config::function_emit_prob` + `Module.function_emit_gates` + `src/ir/function_emit.rs` (`annotate_function_emit_gates`) + the gen-time call-site roll + the `to_sv_with_modules` `function automatic` decl/positional-body/call rendering + lib proofs + a forced-knob Verilator lint. Default-off / DUT byte-identical. **A real emitter change + a downstream spot-check — a focus-intensive slice; a fresh session is reasonable here.** |
| 2 | `STRUCTURED-EMISSION-EXPANSION.2b.2` | `pending` | The repo-owned gate + closeout: `saw_combinational_function_emit` + `num_emitted_combinational_functions` + a `tool_matrix` scenario (Verilator + both Yosys + Icarus) + book/USER_GUIDE/KM. |
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
| `2026-06-16` | `STRUCTURED-EMISSION-EXPANSION.2a` | Design-detail leaf, **no source change** (grounded in a fresh read of `src/emit/sv.rs` — `to_sv_with_modules` gate-emission loop + `build_names`/`node_ref`/`render_gate`/`param_width_decl_w`; `src/ir/soft_union.rs` + `Module.soft_union_slice_gates` — the gen-time-annotation precedent; the `aggregate_layout` projection). `DEVELOPMENT_NOTES.md` design-detail entry (the five points + the `.2b` pre-split): first-cut single-gate "operand function"; gen-time `annotate_function_emit_gates` + `Module.function_emit_gates`; the `<wire>__f` `function automatic` decl + positional-param body + call; `function_emit_prob` (default `0.0` byte-identical); the `saw_combinational_function_emit` gate. `.2b` pre-split → `.2b.1`/`.2b.2`; frontier set to `.2b.1`. `bash scripts/check_memory_architecture.sh` + `bash knowledge-map/scripts/check_knowledge_map.sh` clean. Baseline `cargo check --all-targets` clean (no source touched). | `done` |
| `2026-06-16` | `STRUCTURED-EMISSION-EXPANSION.1` | Design/decision leaf, **no source change** (grounded in a fresh read of `src/emit/sv.rs` `to_sv_with_modules` + the `aggregate_layout` projection + `soft_union_slice_overlay`, `src/ir/soft_union.rs`, and the `aggregate_prob`/`soft_union_slice_prob` default-off emit-projection knobs in `src/config.rs`). Decision `0012` + `INDEX.md` row; tree activated (`proposed → active`); `.2`/`.2a`/`.2b` registered; frontier set to `.2a`. `bash scripts/check_memory_architecture.sh` + `bash knowledge-map/scripts/check_knowledge_map.sh` clean; `KNOWLEDGE_MAP.md` regenerated (decision `0012` carries `answers:` front-matter). Baseline `cargo check --all-targets` clean (from the prior gate; no source touched). | `done` |
| `2026-06-15` | `STRUCTURED-EMISSION-EXPANSION` | Tree registered `proposed` (ownership only, no leaf executed). | `done` (registration) |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `STRUCTURED-EMISSION-EXPANSION.2a` | `STRUCTURED-EMISSION-EXPANSION.2a — combinational function impl design-detail` | Design-detail (no source): pinned the first-cut single-gate "operand function" (minimal cone ⇒ zero sharing hazard), the gen-time `annotate_function_emit_gates` + `Module.function_emit_gates` annotation (the `soft_union.rs` precedent), the `function automatic` decl/positional-body/call rendering, the `function_emit_prob` knob, and the downstream gate. Pre-split `.2b` → `.2b.1`/`.2b.2`. |
| `STRUCTURED-EMISSION-EXPANSION.1` | `STRUCTURED-EMISSION-EXPANSION.1 — activate lane + decision 0012` | Decision `0012`: the first structured surface is a default-off, valid-by-construction combinational `function automatic` emit-projection of an existing cone (over interface/modport + nested generate). Activated the lane by owner directive; split `.1`/`.2`/future; pre-split `.2` → `.2a`/`.2b`. No source change. |
| `STRUCTURED-EMISSION-EXPANSION` | `SV-VERSION-TARGETING.1 — open SV-version lane + decision 0009` | Registered `proposed` alongside the activated `SV-VERSION-TARGETING` lane. |

## Changelog

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
