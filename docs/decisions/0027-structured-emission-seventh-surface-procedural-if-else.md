---
id: structured-emission-seventh-surface-procedural-if-else
title: ANVIL's seventh richer-structured SV surface is a default-off, valid-by-construction procedural `always_comb` `if`/`else` emit-projection of a `Mux` gate
answers:
  - "what is the seventh STRUCTURED-EMISSION-EXPANSION surface"
  - "does ANVIL emit a procedural if/else"
  - "can ANVIL emit an always_comb if else block"
  - "what does mux_if_emit_prob do"
  - "does ANVIL render a mux as a procedural conditional"
  - "can ANVIL emit a 2:1 mux as an if/else statement instead of a ternary"
  - "what structured SV surface comes after the multi-output task"
  - "how does ANVIL emit a procedural conditional mux"
  - "why procedural if/else after the multi-output task"
  - "is ANVIL procedural if/else emission default-off and byte-identical"
date: 2026-06-22
status: accepted
tags: [capability, structured-emission, if-else, procedural, mux, always-comb, emission, downstream, valid-by-construction, rules-first, breadth, north-star]
evidence: docs/decisions/0014-structured-emission-third-surface-combinational-task.md (the single-gate task's output-var + passthrough projection mechanism reused); docs/decisions/0012-structured-emission-first-surface-combinational-function.md (the per-gate emit-projection family); src/ir/task_emit.rs (annotate_task_emit_gates — the exact gen-time annotation precedent) + src/emit/sv.rs (the Mux gate renders today as the ternary `({}) ? ({}) : ({})`, and no surface emits a procedural `if`/`else` statement); docs/tasks/STRUCTURED-EMISSION-EXPANSION.md (the `.14` leaf records the seventh surface); empirical tool-acceptance + simulation-equivalence probe this session (Verilator 5.046 -Wall accepts the `always_comb if/else` projection of a 2:1 mux into a `<wire>__cv` output var warning-clean under --language 1800-2012/2017/2023; Yosys 0.64 synth -noabc and abc -fast both clean with no warnings; iverilog -g2012 compiles; iverilog vvp proves the if/else block bit-equal to the inline `(sel)?(a):(b)` ternary over 20000 random vectors)
---

# 0027 - STRUCTURED-EMISSION-EXPANSION: the seventh richer-structured surface is a procedural `always_comb` `if`/`else` projection of a `Mux`

- Date: 2026-06-22
- Status: accepted
- Tree: `STRUCTURED-EMISSION-EXPANSION.14` (design leaf; picks the seventh surface,
  splits `.14` + `.15` + future)
- Activated by: autonomous PNT selection (`2026-06-22`) at a no-active-frontier
  boundary (`feedback_pick_and_roll_at_no_frontier`), after the sixth surface (the
  multi-output combinational `task automatic`, decision `0025`) and its first
  deepening (wider `k > 2` groups, `.13`) closed end-to-end.

## Context

`STRUCTURED-EMISSION-EXPANSION` broadens ANVIL's emitted SystemVerilog past its
flat `module` + per-gate `assign` / `always` + instance shape into richer
*structured* constructs, each a new legal interaction surface a downstream tool
must parse, elaborate, and lower — and so a new place to surface a real bug
(`project_anvil_north_star`). The lane's pattern is fixed by decisions `0012`
(combinational `function automatic`), `0013`/`0015` (`generate for` loop, 1-bit
and wider lane), `0014` (single-gate combinational `task automatic`), `0016`
(multi-gate-cone `function automatic`), and `0025` (multi-output combinational
`task automatic`): *an emit-projection of an existing valid construct, rules-first,
default-off / byte-identical, proven downstream-clean before it ships.*

All six delivered surfaces project a combinational gate (or cone, or group) into a
`function` / `task` / `generate for` form. **None of them emits a procedural
`if`/`else` statement.** The structured selectors `CaseMux` / `CasezMux` already
render as `always_comb case` / `always_comb casez` (Phase 3), and a plain `Mux`
gate (`[sel, a, b]`, `sel.width == 1`) renders today as the **continuous-assign
ternary** `assign <wire> = (sel) ? (a) : (b);` (`src/emit/sv.rs`). The `?:`
operator is also already reachable inside a `function`/`task` body, because `Mux`
is in the `function_emit` / `task_emit` candidate set. But a **procedural
`always_comb` block with an `if`/`else` statement** is a construct the DUT emitter
has never produced — a genuinely new elaboration path for a downstream tool to
exercise (a procedural conditional resolves through a different frontend code path
than a continuous-assign ternary or a `case`).

A fresh empirical probe this session confirms the projected construct directly with
the installed tools. The procedural form

```systemverilog
logic [7:0] mux_0__cv;
always_comb begin
    if (sel) mux_0__cv = a;
    else mux_0__cv = b;
end
assign y_if = mux_0__cv;
```

is accepted **warning-clean** by Verilator 5.046 (`-Wall --lint-only`) under
`--language 1800-2012`, `1800-2017`, and `1800-2023`, by **both** repo Yosys modes
(`synth -noabc` and `abc -fast; opt -fast; check`, no warnings/errors), and by
Icarus (`iverilog -g2012`), and `iverilog`+`vvp` prove it **bit-equal to the inline
`(sel) ? (a) : (b)` ternary** over 20000 random vectors.

## Decision

**The seventh richer-structured surface is a default-off, opt-in,
valid-by-construction procedural `always_comb` `if`/`else` emit-projection of a
`Mux` gate** — a behaviour-preserving re-expression of the 2:1 selection the `Mux`
renders today as a ternary, projected into a procedural conditional that writes a
per-gate **output var**, the existing net driven from it by a passthrough `assign`
(the decision `0014` single-gate `task` mechanism, but expressed as a bare
`always_comb` `if`/`else` rather than a `task` call). For a marked `Mux` gate
`g = Mux[sel, a, b]` of width `W` it renders

```systemverilog
logic [W-1:0] <g>__cv;
always_comb begin
    if (<sel>) <g>__cv = <a>;
    else <g>__cv = <b>;
end
assign <g> = <g>__cv;   // the gate's net, unchanged downstream
```

instead of the inline `assign <g> = (<sel>) ? (<a>) : (<b>);`. `<sel>` / `<a>` /
`<b>` are the operand refs the emitter already resolves for the ternary (operand 0
= the 1-bit selector, operand 1 = the `sel == 1` value, operand 2 = the `sel == 0`
value). The `if`/`else` writes exactly the gate's value into `<g>__cv` (`sel == 1`
⇒ `a`, `sel == 0` ⇒ `b` — identical to the ternary's operand mapping), and the
existing `<g>` net is driven from it — so the projection is **behaviour-preserving
by construction.** First cut = the 2:1 `Mux`; the N-way `CaseMux` → `if`/`else if`
priority chain is a recorded follow-up (see below).

### The candidate set (rules-first, valid-by-construction)

- **Candidate = a `Node::Gate` whose op is `GateOp::Mux`** (exactly three operands,
  a 1-bit selector). The structured selectors (`CaseMux` / `CasezMux` / `ForFold`)
  already have their own `always_comb` rendering and are *not* candidates; `Slice`
  is not a candidate (a bit-select has no conditional). The first cut deliberately
  scopes to the plain `Mux` — the simplest, highest-yield 2:1 conditional.
- **Minus any gate already marked by a sibling projection.** A `Mux` is also a
  `function_emit` / `task_emit` candidate; a gate is projected by **at most one** of
  the seven surfaces. The exact pass ordering (this pass relative to the others) is
  pinned at `.15a`; the leading first cut runs it **after** the existing six and
  excludes any gate already in `function_emit_gates` / `generate_loop_gates` /
  `task_emit_gates` / `multi_output_task_groups` / `cone_function_gates` /
  `soft_union_slice_gates` — the established "later pass excludes earlier marks"
  ordering (`task_emit` runs after `function_emit` runs after `soft_union`).
- **Rolled at the call site like every other knob.** The per-gate decision is a
  seeded `gen_bool(prob)` (reproducible; never `thread_rng`). The generator guards
  the call on `Config::mux_if_emit_prob > 0.0`, so the default (`0.0`) draws nothing
  and marks nothing ⇒ byte-identical stream + output.

### Construction discipline (the lane invariants)

- **Rules-first** (`feedback_rules_first_generation`): selection re-expresses an
  *already-valid* `Mux` gate at construction time; the procedural block is a
  deterministic re-rendering, behaviour-preserving by construction — never
  generate-then-filter.
- **Default-off / byte-identical:** a new opt-in `mux_if_emit_prob` (default `0.0`)
  + its `--mux-if-emit-prob` CLI flag; with it off the output is byte-identical and
  `tests/snapshots.rs` is untouched (`feedback_never_retire_strategies`). An
  unmarked `Mux` still emits the inline ternary.
- **Its own knob (nothing retired).** Separate from `task_emit_prob` /
  `function_emit_prob` so the shipped surfaces stay byte-identical (reusing an
  existing knob rejected — it would change that knob's output and blur two
  surfaces; the decision `0016` / `0025` separate-knob precedent).
- **Mutually exclusive with the sibling projections.** A gate is projected by at
  most one of `function_emit` / `generate_loop` / `task_emit` / `multi_output_task`
  / `cone_function` / `soft_union` / `mux_if`.
- **Combinational only.** The `Mux` is a combinational gate; its operand refs are
  leaves of the procedural block. The block never recurses through a register edge
  or instance boundary. There is no soundness cycle risk: the block reads only the
  gate's direct operand refs and writes only its own `<g>__cv` var — exactly the
  inline ternary's read/write set (unlike the multi-output task, no cross-member
  fan-in interaction).
- **No new IR node / no new computed truth.** The procedural block is a pure
  emit-time projection of an existing `Mux`; the flat IR body, validators, CSE keys,
  and `canonical_module_signature` are untouched (the `task_emit` / `cone_function`
  precedent). The structure-first ceiling of decisions `0004` / `0011` is unaffected
  — this adds emission *shape*, not behaviour.

### Why a procedural `if`/`else` seventh (not nested `generate` or `interface`/`modport`)

- **Universally downstream-clean (verified).** The procedural `always_comb`
  `if`/`else` into an output var elaborates and synthesizes cleanly in Verilator
  (`-Wall`, all three `--language` standards), both repo Yosys modes (no
  warnings/errors), and Icarus, and is sim-equivalent (probe above). `interface` /
  `modport` is **empirically disqualified** (since `.7`/decision `0015`: Icarus
  syntax-fails the modport port and both Yosys modes warn on the implicit
  interface-member decl). Nested / multi-level `generate` has **no routine
  by-construction source**: operand-uniqueness CSE shares the inner `{N{x}}`
  replication into one wire, so the existing single-level `generate for` (surfaces 2
  & 4) already fires on the resulting outer `{M{y}}` — a doubly-nested
  `{M{{N{x}}}}` essentially never survives factorization as a distinct shape.
- **A genuinely new elaboration interaction.** A procedural `always_comb` block with
  an `if`/`else` statement is a construct **no** delivered surface emits (the six are
  `function`/`task`/`generate` projections; the `Mux` ternary is a continuous
  assign; `CaseMux`/`CasezMux` are `case`/`casez`). A procedural conditional is a
  distinct frontend/elaboration path — good downstream bug bait — not a cosmetic
  variant.
- **High yield, minimal blast radius / maximal reuse.** `Mux` gates are pervasive in
  generated cones, so the surface fires readily by construction. The mechanism is the
  **decision `0014` output-var + passthrough** projection (the closest precedent,
  `src/ir/task_emit.rs` + the `to_sv_with_modules` task section), reusing the
  operand-ref rendering the ternary already uses — one annotation family, one
  body-render path, no parallel machinery (`feedback_full_factorization`).

### Downstream gate

A focused repo-owned gate (a `tool_matrix --mux-if-gate` scenario, templated on
`--task-emit-gate`) forces `mux_if_emit_prob = 1.0` over a comb-only DUT across the
three construction strategies and fails on coverage gaps unless the emitted
procedural conditionals are accepted **warning-clean** by Verilator + both Yosys
modes + Icarus, gated on a `saw_mux_if_emit` coverage fact (detected from the
emitted SV text via the `__cv` token, distinct from `__f` / `__tv` / `__mtv` /
`__cf` / `__mt`). Like a single-gate task (and unlike the `union soft` up-opt), a
procedural `always_comb if/else` is universally synthesizable, so the gate runs the
full tool plan. The scenario shape must bias toward `Mux` gates (raise the mux
selection so candidates exist); the exact calibration is pinned at `.15`.

## Decisive test applied

"Does the surface add a new legal structural shape **without** new whole-module
behaviour or a default-output change, and is it reliably accepted by every repo
downstream tool?" A procedural `always_comb` `if`/`else` projection of a `Mux`
passes: it is a new procedural conditional shape (no surface emits one),
behaviour-preserving, default-off byte-identical, broadly synthesizable +
sim-equivalent (empirically verified). `interface` / `modport` still fails the
"every Yosys mode clean" sub-test; nested multi-level `generate` lacks a
by-construction source.

## Rejected alternatives

- **Making `<g>` itself the `always_comb` var (no passthrough net).** Declaring the
  gate's wire as a `logic` written by the block (dropping the `<g>__cv` + passthrough)
  is fewer lines but changes `<g>` from a net to a var and would require rewriting
  every downstream consumer's view of `<g>`. Rejected in favour of the
  decision-`0014` output-var + passthrough form, which keeps `<g>` a net and changes
  only the gate's own drive (minimal blast radius, the established mechanism).
- **A `case (sel)` instead of `if`/`else`.** A 2-arm `always_comb case (sel)` is also
  clean, but `CaseMux` already renders `always_comb case` — so a `case` projection of
  `Mux` would duplicate an existing construct rather than add a new one. The
  `if`/`else` is the genuinely new procedural shape.
- **Projecting the N-way `CaseMux` to an `if`/`else if` priority chain in the first
  cut.** A richer, genuinely-distinct construct (a priority chain vs the parallel
  `case`), but a bigger blast radius (selector-arm bounds, out-of-range default
  matching the current `case` semantics). The pair-then-widen discipline (surfaces 2
  and 6) ships the simplest clean cut first: the 2:1 `Mux` → `if`/`else`. The
  `CaseMux` → `if`/`else if` chain is the recorded follow-up (`.16+`), none retired.
- **Reusing `task_emit_prob` / `function_emit_prob`.** Rejected — it would change a
  shipped knob's output and blur two surfaces; the procedural conditional gets its
  **own** `mux_if_emit_prob` (the decision `0016` / `0025` separate-knob precedent).
- **A new IR node (e.g. a `ProceduralMux`).** Rejected: the block is an emit-time
  projection of an existing `Mux` — no new IR truth, default-off byte-identical (the
  `task_emit` / `cone_function` precedent).
- **Generate-then-filter** (emit arbitrary procedural blocks, then validate/discard).
  Forbidden (`feedback_rules_first_generation`).
- **Changing the default output.** Rejected: opt-in only, even once proven
  downstream-clean (`feedback_never_retire_strategies`).

## Consequences

- ANVIL gains its **seventh** richer-structured emit surface; the default `anvil`
  build and `--artifact dut` stay byte-identical (knob default `0.0`).
- The DUT lane gains a procedural `always_comb` `if`/`else` block — a new legal
  procedural-conditional shape to parse / elaborate / lower, distinct from the
  continuous-assign ternary and the `case`/`casez` selectors — a new bug-surfacing
  surface.
- The lane's pattern holds: an emit-projection of an existing valid construct,
  rules-first, default-off / byte-identical, proven downstream-clean before it
  ships, reusing existing render machinery. The N-way `CaseMux` → `if`/`else if`
  priority chain, nested / multi-level `generate`, and `interface` / `modport` each
  land later as their own decided leaves, none retired.

## Open questions (to be resolved at `.15a`)

- The exact pass ordering: run the `mux_if` annotation **after** the existing six
  (the leading first cut, excluding already-marked gates) vs interleaving it. The
  after-the-six order is the natural extension of the established convention.
- The exact `Module` carrier: a `mux_if_gates: BTreeSet<NodeId>` of marked `Mux`
  gates (the `task_emit_gates` / `function_emit_gates` precedent), iterated in
  `NodeId` order for determinism.
- The exact emitter integration: a per-gate `always_comb` block emitted in the
  gate-assign loop (the gate's inline ternary suppressed; the `<g>__cv` decl + the
  block + the passthrough `assign` emitted) — mirroring the `task_emit` call
  section.
- The exact knob name + semantics (`mux_if_emit_prob` proposed), the metric
  (`num_emitted_mux_if_blocks` proposed, introspection schema `1.14 → 1.15`), and
  the downstream-gate scenario shape (`saw_mux_if_emit`, `__cv` detection, mux-biased
  calibration).

## Tree split

`STRUCTURED-EMISSION-EXPANSION` continues (the lane stays `active`):

- **`.14`** (this leaf, design) — decision `0027`: the seventh surface, its
  valid-by-construction discipline, its candidate set, its opt-in knob, its
  downstream gate, and the rejected alternatives. Docs-only.
- **`.15`** (impl, `pending`) — the procedural `always_comb` `if`/`else` surface:
  the `mux_if_emit_prob` knob + the gen-time annotation + the emitter rendering +
  the metric + the downstream-clean gate + book/USER_GUIDE/KM. Default-off / DUT
  byte-identical. Pre-split into `.15a` (design-detail, grounded in the real
  `task_emit.rs` / `to_sv_with_modules` Mux-render + gate source) + `.15b` (impl,
  itself pre-split `.15b.1` live / `.15b.2` metric+gate / `.15b.3` docs) when picked.
- **future (`.16`+)** — the N-way `CaseMux` → `if`/`else if` priority chain, nested
  / multi-level `generate`, `interface` / `modport`, each a new vetted surface with
  its own decision when picked.

## Links

- Owner doctrine: `feedback_pick_and_roll_at_no_frontier` (autonomous surface
  selection at the no-frontier boundary), `feedback_dont_ask_just_do`.
- Lane / ROADMAP: steering gap 1 (richer structured emission), the structure-first
  ceiling (steering gap 4 — this adds shape, not behaviour).
- Doctrine: `feedback_rules_first_generation` (no generate-then-filter),
  `feedback_never_retire_strategies` (opt-in, default byte-identical),
  `feedback_full_factorization` (reuse the single-gate task output-var + passthrough
  mechanism + the operand-ref rendering; one mechanism, not two).
- Precedents: decision `0014` (the single-gate combinational `task automatic` whose
  output-var + passthrough projection mechanism this reuses) + decision `0012` (the
  per-gate emit-projection family) + decisions `0013` / `0015` / `0016` / `0025`
  (the emit-projection family + the separate-knob discipline) + the `soft_union`
  overlay (`0010`).
- Reuse / touch points: `src/emit/sv.rs` (`to_sv_with_modules` + the projection
  sections; the existing `Mux` ternary operand-ref rendering reused for the block
  body), `src/config.rs` (the `mux_if_emit_prob` knob + the `--mux-if-emit-prob`
  flag beside `task_emit_prob` / `cone_function_emit_prob` /
  `multi_output_task_emit_prob`), `src/ir/` (a `mux_if_emit` gen-time-annotation
  pass beside `task_emit.rs`), `src/metrics.rs` (`num_emitted_mux_if_blocks`,
  introspection schema `1.14 → 1.15`), `src/bin/tool_matrix.rs` (the `--mux-if-gate`
  downstream gate), `book/src/structured-emission.md` (user-facing — extend the
  chapter).
