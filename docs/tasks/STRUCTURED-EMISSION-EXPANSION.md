# STRUCTURED-EMISSION-EXPANSION: richer structured SystemVerilog surfaces

## Metadata

- Tree ID: `STRUCTURED-EMISSION-EXPANSION`
- Status: `active`
- Roadmap lane: `Capability / breadth — richer structured emission (ROADMAP steering gap 1)`
- Created: `2026-06-15`
- Last updated: `2026-06-16` (**activated by explicit owner directive** after `SEMANTIC-INTROSPECTION-EXPANSION` delivered all four query kinds; `.1` design — decision `0012`: the first richer-structured surface is a default-off, valid-by-construction combinational `function automatic` emit-projection of an existing combinational cone, the `output_support` cone-support boundary giving its parameter list; chosen over interface/modport (weak Yosys synth support) and nested generate (bigger blast radius); opt-in `function_emit_prob` (default `0.0`) ⇒ byte-identical; downstream gate proves Verilator + both Yosys modes + Icarus accept it. Pre-split `.2` → `.2a` (design-detail, **frontier**) + `.2b` (impl).)
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
  Status: `pending`
  Goal: `Design-detail leaf (no source): ground the combinational function automatic surface in the real src/emit/sv.rs to_sv_with_modules + the soft_union.rs / aggregate_layout emit-projection precedents + src/config.rs. Pin: (1) the cone-selection rule (which Gate nodes qualify; size/depth bounds so the function is non-trivial yet bounded; how it stays rules-first); (2) whether selection is a generation-time annotation (the soft_union.rs / aggregate_layout precedent — likely, so the IR carries the choice deterministically and emission projects it) or a pure emit-time pass; (3) the function signature + body rendering (parameter list = the cone's support leaves; local decls vs single return expr; width/logic typing); (4) the function_emit_prob knob semantics + default 0.0 byte-identical contract; (5) the downstream-gate scenario shape (saw_combinational_function_emit). DEVELOPMENT_NOTES design-detail entry + the .2b impl shape.`
  Acceptance: `A DEVELOPMENT_NOTES design-detail entry resolving the five points grounded in real code; tree split recorded; no source change; docs/workflow self-checks clean.`
  Verification: `pending`
  Commit: `pending`

- ID: `STRUCTURED-EMISSION-EXPANSION.2b`
  Status: `pending`
  Goal: `Implement the .2a design: the function_emit_prob knob, the rules-first cone selection (gen-time annotation or emit-time pass per .2a), the function automatic emitter rendering (decl + call site), lib proofs (the function is behaviour-preserving + selected by construction + default-off byte-identical), the downstream-clean gate (Verilator + both Yosys modes + Icarus + the saw_combinational_function_emit fact), and book/USER_GUIDE/KM closeout. Default-off / DUT byte-identical (snapshots untouched).`
  Acceptance: `cargo check/clippy(-D warnings)/fmt clean; cargo test --lib + snapshots 6/6 byte-identical (default-off); the function-emit knob produces downstream-clean function automatic SV (Verilator + both Yosys modes + Icarus) with the coverage fact lit; book/USER_GUIDE + a KM fact; committed through COMMIT.md with the leaf id. (Pre-split into .2b.1 pure construction/projection + .2b.2 the gate/closeout if broad.)`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

**Active frontier: `STRUCTURED-EMISSION-EXPANSION.2a`** (the design-detail leaf for
the combinational `function automatic` surface). `.1` (decision `0012`) is done.
Nothing retired.

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `STRUCTURED-EMISSION-EXPANSION.2a` | `pending` | Design-detail (no source): ground the combinational `function automatic` surface in the real `to_sv_with_modules` + the `soft_union.rs`/`aggregate_layout` precedents — pin the cone-selection rule, gen-time-annotation-vs-emit-time choice, the function signature/body rendering, the `function_emit_prob` knob, and the downstream-gate shape. Pre-split `.2b` if broad. |
| 2 | `STRUCTURED-EMISSION-EXPANSION.2b` | `pending` | Impl: the knob + rules-first cone selection + the `function automatic` emitter rendering + the downstream-clean gate + closeout. Default-off / DUT byte-identical. |
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
| `2026-06-16` | `STRUCTURED-EMISSION-EXPANSION.1` | Design/decision leaf, **no source change** (grounded in a fresh read of `src/emit/sv.rs` `to_sv_with_modules` + the `aggregate_layout` projection + `soft_union_slice_overlay`, `src/ir/soft_union.rs`, and the `aggregate_prob`/`soft_union_slice_prob` default-off emit-projection knobs in `src/config.rs`). Decision `0012` + `INDEX.md` row; tree activated (`proposed → active`); `.2`/`.2a`/`.2b` registered; frontier set to `.2a`. `bash scripts/check_memory_architecture.sh` + `bash knowledge-map/scripts/check_knowledge_map.sh` clean; `KNOWLEDGE_MAP.md` regenerated (decision `0012` carries `answers:` front-matter). Baseline `cargo check --all-targets` clean (from the prior gate; no source touched). | `done` |
| `2026-06-15` | `STRUCTURED-EMISSION-EXPANSION` | Tree registered `proposed` (ownership only, no leaf executed). | `done` (registration) |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `STRUCTURED-EMISSION-EXPANSION.1` | `STRUCTURED-EMISSION-EXPANSION.1 — activate lane + decision 0012` | Decision `0012`: the first structured surface is a default-off, valid-by-construction combinational `function automatic` emit-projection of an existing cone (over interface/modport + nested generate). Activated the lane by owner directive; split `.1`/`.2`/future; pre-split `.2` → `.2a`/`.2b`. No source change. |
| `STRUCTURED-EMISSION-EXPANSION` | `SV-VERSION-TARGETING.1 — open SV-version lane + decision 0009` | Registered `proposed` alongside the activated `SV-VERSION-TARGETING` lane. |

## Changelog

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
