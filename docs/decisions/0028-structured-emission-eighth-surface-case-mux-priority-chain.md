---
id: structured-emission-eighth-surface-case-mux-priority-chain
title: ANVIL's eighth richer-structured SV surface is a default-off, valid-by-construction procedural `always_comb` `if`/`else if` priority-chain emit-projection of a `CaseMux` gate
answers:
  - "what is the eighth STRUCTURED-EMISSION-EXPANSION surface"
  - "does ANVIL emit an if else if priority chain"
  - "can ANVIL render an N-way case as an if/else if chain"
  - "what does case_mux_if_emit_prob do"
  - "does ANVIL project a CaseMux to a procedural priority chain"
  - "can ANVIL emit a case statement as a chain of if else if"
  - "what structured SV surface comes after the procedural if/else mux"
  - "how does ANVIL emit an N-way priority mux"
  - "why an if/else if priority chain after the 2:1 mux if/else"
  - "is ANVIL case-mux priority-chain emission default-off and byte-identical"
date: 2026-06-22
status: accepted
tags: [capability, structured-emission, if-else-if, priority-chain, case-mux, procedural, always-comb, emission, downstream, valid-by-construction, rules-first, breadth, north-star]
evidence: docs/decisions/0027-structured-emission-seventh-surface-procedural-if-else.md (the procedural-conditional family this generalizes from the 2:1 Mux to the N-way CaseMux; it recorded the CaseMux -> if/else if priority chain as the .16+ follow-up); src/emit/sv.rs (the CaseMux gate renders today as `always_comb begin case (sel) W'dk: name = arm_k; default: name = WIDTH'h0; endcase end` at the structured-case block; the projection re-expresses exactly that case as an if/else if chain); docs/tasks/STRUCTURED-EMISSION-EXPANSION.md (the .16 leaf records the eighth surface); empirical tool-acceptance + simulation-equivalence probe this session (Verilator 5.046 -Wall accepts the if/else if priority chain warning-clean under --language 1800-2012/2017/2023; Yosys 0.64 synth -noabc clean [32 cells, no warnings] and abc -fast clean; iverilog -g2012 compiles; iverilog vvp proves the chain bit-equal to the parallel `case (sel) ... default` form over 20000 random vectors plus an exhaustive selector sweep -> EQUIV OK)
---

# 0028 - STRUCTURED-EMISSION-EXPANSION: the eighth richer-structured surface is a procedural `always_comb` `if`/`else if` priority-chain projection of a `CaseMux`

- Date: 2026-06-22
- Status: accepted
- Tree: `STRUCTURED-EMISSION-EXPANSION.16` (design leaf; picks the eighth surface,
  splits `.16` + `.17` + future)
- Activated by: autonomous PNT selection (`2026-06-22`) at a no-active-frontier
  boundary (`feedback_pick_and_roll_at_no_frontier`), after the seventh surface (the
  procedural `always_comb` `if`/`else` projection of a 2:1 `Mux`, decision `0027`)
  closed end-to-end. The N-way `CaseMux` -> `if`/`else if` priority chain was the
  explicitly recorded `.16+` follow-up of decision `0027`.

## Context

`STRUCTURED-EMISSION-EXPANSION` broadens ANVIL's emitted SystemVerilog past its flat
`module` + per-gate `assign` / `always` + instance shape into richer *structured*
constructs, each a new legal interaction surface a downstream tool must parse,
elaborate, and lower — and so a new place to surface a real bug
(`project_anvil_north_star`). The lane's pattern is fixed by decisions `0012`
(combinational `function automatic`), `0013`/`0015` (`generate for` loop, 1-bit and
wider lane), `0014` (single-gate combinational `task automatic`), `0016`
(multi-gate-cone `function automatic`), `0025` (multi-output combinational `task
automatic`), and `0027` (procedural `always_comb` `if`/`else` projection of a 2:1
`Mux`): *an emit-projection of an existing valid construct, rules-first, default-off /
byte-identical, proven downstream-clean before it ships.*

The seventh surface (decision `0027`) shipped the lane's **first
procedural-conditional** shape — but only the **2:1** case: a `GateOp::Mux`
(`[sel, a, b]`, `sel.width == 1`) re-expressed from its continuous-assign ternary into
a procedural `always_comb if (sel) <g>__cv = a; else <g>__cv = b;`. The **N-way**
selector is a separate, genuinely-distinct construct. ANVIL's `GateOp::CaseMux`
(a dynamic-selector N-way mux) renders today (`src/emit/sv.rs`, the structured-case
block) as a **parallel `case` statement**:

```systemverilog
logic [W-1:0] casemux_0;        // structured gate -> declared as a logic var
always_comb begin
    case (sel)
        SW'd0: casemux_0 = arm_0;
        SW'd1: casemux_0 = arm_1;
        ...
        SW'd{k-1}: casemux_0 = arm_{k-1};
        default: casemux_0 = W'h0;
    endcase
end
```

where `sel` is operand 0 (selector width `SW`), the `k` arms are operands `1..=k` with
the literal labels `SW'd0 .. SW'd{k-1}`, and the trailing `default` drives `W'h0`. A
**procedural `if`/`else if` priority chain** over the same selector is a construct the
DUT emitter has never produced — a genuinely new elaboration path (a priority chain of
equality tests resolves through a different frontend/synthesis code path than a
parallel `case` statement, even though the two are functionally identical when the
labels are distinct constants — which they always are here).

A fresh empirical probe this session confirms the projected construct directly with the
installed tools. The procedural priority-chain form

```systemverilog
always_comb begin
    if (sel == 3'd0) casemux_0 = arm_0;
    else if (sel == 3'd1) casemux_0 = arm_1;
    else if (sel == 3'd2) casemux_0 = arm_2;
    else if (sel == 3'd3) casemux_0 = arm_3;
    else if (sel == 3'd4) casemux_0 = arm_4;
    else casemux_0 = 4'h0;
end
```

is accepted **warning-clean** by Verilator 5.046 (`-Wall --lint-only`) under
`--language 1800-2012`, `1800-2017`, and `1800-2023`, by **both** repo Yosys modes
(`synth -noabc` [32 cells: 20 `$_MUX_`, 3 `$_NOT_`, 9 `$_OR_`, no warnings] and
`abc -fast; opt -fast; check`, no warnings/errors), and by Icarus (`iverilog -g2012`),
and `iverilog`+`vvp` prove it **bit-equal to the parallel `case (sel) ... default`
form** over 20000 random vectors **plus an exhaustive 3-bit selector sweep** (EQUIV
OK).

## Decision

**The eighth richer-structured surface is a default-off, opt-in,
valid-by-construction procedural `always_comb` `if`/`else if` priority-chain
emit-projection of a `CaseMux` gate** — a behaviour-preserving re-expression of the
N-way selection the `CaseMux` renders today as a parallel `case` statement, projected
into a priority chain of selector-equality tests writing the gate's existing structured
`logic` var. For a marked `CaseMux` gate `g` of width `W`, selector `sel` of width
`SW`, and arms `arm_0 .. arm_{k-1}` it renders

```systemverilog
always_comb begin
    if (<sel> == SW'd0) <g> = <arm_0>;
    else if (<sel> == SW'd1) <g> = <arm_1>;
    ...
    else if (<sel> == SW'd{k-1}) <g> = <arm_{k-1}>;
    else <g> = W'h0;
end
```

instead of the inline `case (<sel>) SW'd0: <g> = <arm_0>; ...; default: <g> = W'h0;
endcase`. `<sel>` / `<arm_i>` are the operand refs the emitter already resolves for the
`case` block (operand 0 = the selector; operands `1..=k` = the arms, in order). The
chain tests each label in **ascending arm order** and falls through to the same
`default` value, so — because the `case` labels `SW'd0 .. SW'd{k-1}` are **distinct
constants by construction** (arm index `i` -> label `SW'd{i}`) — the priority chain and
the parallel `case` are **identical for every selector value**: at most one equality is
true, and the trailing `else` covers exactly the labels the `case` left to `default`.
The projection is therefore **behaviour-preserving by construction.** First cut = the
plain `CaseMux` (equality `case`); the wildcard `CasezMux` -> masked `if`/`else if`
chain is a recorded follow-up (see below).

### The candidate set (rules-first, valid-by-construction)

- **Candidate = a `Node::Gate` whose op is `GateOp::CaseMux` that actually renders as
  the dynamic `always_comb case` block** — i.e. one for which the existing
  `render_static_structured_gate` constant-selector collapse returns `None` (a
  non-constant selector). A constant-selector `CaseMux` already lowers to a continuous
  `assign` of the selected arm and is **not** a candidate (there is no conditional to
  re-express).
- **`CasezMux` is not a candidate.** Its `casez` arms carry `?`-wildcard patterns whose
  faithful priority-chain form is a *masked* comparison (`(sel & mask) == pattern`), a
  strictly bigger construct. The first cut scopes to the plain equality `CaseMux`,
  exactly as the seventh surface scoped to the 2:1 `Mux` (the `CasezMux` chain is a
  recorded `.17+`/future follow-up, nothing retired). `ForFold` is not a candidate (it
  is a `for`-loop fold, not a selector).
- **Minus any gate already marked by a sibling projection.** A `CaseMux` is a
  structured selector and is **not** in the `function_emit` / `task_emit` /
  `cone_function` / `multi_output_task` / `generate_loop` / `soft_union` candidate sets
  (those target plain combinational gates / replications), so in practice the exclusion
  is vacuous — but the pass still excludes any already-marked gate for robustness, and
  runs **after** the seven existing projections (the established "later pass excludes
  earlier marks" ordering). A gate is projected by **at most one** of the eight
  surfaces.
- **Rolled at the call site like every other knob.** The per-gate decision is a seeded
  `gen_bool(prob)` (reproducible; never `thread_rng`). The generator guards the call on
  `Config::case_mux_if_emit_prob > 0.0`, so the default (`0.0`) draws nothing and marks
  nothing => byte-identical stream + output.

### Construction discipline (the lane invariants)

- **Rules-first** (`feedback_rules_first_generation`): selection re-expresses an
  *already-valid* `CaseMux` gate at construction time; the priority chain is a
  deterministic re-rendering, behaviour-preserving by construction — never
  generate-then-filter.
- **Default-off / byte-identical:** a new opt-in `case_mux_if_emit_prob` (default `0.0`)
  + its `--case-mux-if-emit-prob` CLI flag; with it off the output is byte-identical and
  `tests/snapshots.rs` is untouched (`feedback_never_retire_strategies`). An unmarked
  `CaseMux` still emits the inline `case`.
- **Its own knob (nothing retired).** Separate from `mux_if_emit_prob` (which targets
  the 2:1 `Mux`) so the shipped surfaces stay byte-identical (reusing an existing knob
  rejected — it would change that knob's output and blur two distinct surfaces; the
  decision `0016` / `0025` / `0027` separate-knob precedent).
- **Mutually exclusive with the sibling projections.** A gate is projected by at most
  one of `function_emit` / `generate_loop` / `task_emit` / `multi_output_task` /
  `cone_function` / `soft_union` / `mux_if` / `case_mux_if`.
- **Combinational only, no output-var/passthrough needed.** The `CaseMux` structured
  gate is **already declared as a `logic` var** (not a net) and **already written from
  an `always_comb` block** — so, unlike the seventh surface's 2:1 `Mux` (which converts
  a continuous-assign net), this projection only swaps the *body* of an existing
  `always_comb` (`case ... endcase` -> `if ... else`). No `<g>__cv` output var and no
  passthrough `assign` are required; `<g>` stays exactly what it is today. The block
  reads only the gate's direct operand refs and writes only `<g>` — exactly the `case`
  block's read/write set (no soundness cycle risk).
- **No new IR node / no new computed truth.** The priority chain is a pure emit-time
  projection of an existing `CaseMux`; the flat IR body, validators, CSE keys, and
  `canonical_module_signature` are untouched (the `mux_if` / `task_emit` / `cone_function`
  precedent). The structure-first ceiling of decisions `0004` / `0011` is unaffected —
  this adds emission *shape*, not behaviour.

### Why an N-way priority chain eighth (not nested `generate` or `interface`/`modport`)

- **Universally downstream-clean (verified).** The procedural `always_comb` `if`/`else
  if` priority chain elaborates and synthesizes cleanly in Verilator (`-Wall`, all three
  `--language` standards), both repo Yosys modes (no warnings/errors), and Icarus, and
  is sim-equivalent to the `case` it replaces (probe above). `interface` / `modport` is
  **empirically disqualified** (since `.7`/decision `0015`: Icarus syntax-fails the
  modport port and both Yosys modes warn on the implicit interface-member decl). Nested
  / multi-level `generate` has **no routine by-construction source** (operand-uniqueness
  CSE shares the inner `{N{x}}` replication into one wire, so the existing single-level
  `generate for` already fires on the outer `{M{y}}`; a doubly-nested `{M{{N{x}}}}`
  essentially never survives factorization).
- **A genuinely new elaboration interaction, distinct from the seventh surface.** The
  seventh surface (`0027`) projects a **2:1 `Mux`** into a **single** `if`/`else`; this
  surface projects an **N-way `CaseMux`** into an `if`/`else if` **priority chain** — a
  different IR candidate (`CaseMux` vs `Mux`) and a different emitted shape (a chain of
  equality tests vs one conditional). It is also distinct from the existing `case`
  render: a priority `if`/`else if` chain is a *sequential-priority* construct, where a
  `case` is *parallel-match* — synthesis and lint tools take different code paths for
  the two even when the result is identical. Good downstream bug bait, not a cosmetic
  variant.
- **High yield, minimal blast radius / maximal reuse.** `CaseMux` gates are produced by
  the Phase-3 structured selector path (`case_mux_prob`), so the surface fires readily
  by construction when that path is biased on. The mechanism reuses the **existing
  structured-case `always_comb` section** and the **operand-ref rendering** the `case`
  block already uses — one body-render branch, no parallel machinery
  (`feedback_full_factorization`). It is *simpler* than the seventh surface (no
  output-var/passthrough), because the gate is already an `always_comb`-written `logic`.

### Downstream gate

A focused repo-owned gate (a `tool_matrix --case-mux-if-gate` scenario, templated on
`--mux-if-gate`) forces `case_mux_if_emit_prob = 1.0` over a comb-only DUT across the
three construction strategies, with a `CaseMux`-biased focus config (raise
`case_mux_prob` so dynamic-selector `CaseMux` candidates exist), and fails on coverage
gaps unless the emitted priority chains are accepted **warning-clean** by Verilator +
both Yosys modes + Icarus, gated on a `saw_case_mux_if_emit` coverage fact. Detection:
because this surface introduces **no new identifier token** (it writes the gate's
existing var, unlike the seventh surface's `__cv`), the gate keys the coverage fact on
the per-module metric `num_emitted_case_mux_if_chains > 0` (which already flows through
the per-module `Metrics` into the matrix report), a strictly more robust signal than a
text scan. Like a single-gate task (and unlike the `union soft` up-opt), a procedural
`always_comb if/else if` chain is universally synthesizable, so the gate runs the full
tool plan. The exact knob/metric names and calibration are pinned at `.17`.

## Decisive test applied

"Does the surface add a new legal structural shape **without** new whole-module
behaviour or a default-output change, and is it reliably accepted by every repo
downstream tool?" A procedural `always_comb` `if`/`else if` priority-chain projection of
a `CaseMux` passes: it is a new sequential-priority procedural shape (no surface emits
one — the seventh surface emits a single 2:1 conditional, not an N-way chain),
behaviour-preserving, default-off byte-identical, broadly synthesizable +
sim-equivalent (empirically verified). `interface` / `modport` still fails the "every
Yosys mode clean" sub-test; nested multi-level `generate` lacks a by-construction
source.

## Rejected alternatives

- **Reusing `mux_if_emit_prob` (the seventh-surface knob).** Rejected — it would change
  a shipped knob's output and blur two distinct surfaces (the 2:1 `Mux` single
  conditional vs the N-way `CaseMux` chain); the priority chain gets its **own**
  `case_mux_if_emit_prob` (the decision `0016` / `0025` / `0027` separate-knob
  precedent).
- **Adding a `<g>__cv` output var + passthrough (mirroring the seventh surface).**
  Rejected as unnecessary machinery: a `CaseMux` is *already* an `always_comb`-written
  `logic` var, so the projection swaps only the block body. Introducing a passthrough
  would convert `<g>` from its current var to a net + var pair for no benefit. The
  faithful minimal change keeps `<g>` exactly as it is.
- **Folding the `CasezMux` wildcard case into the same first cut.** Rejected for this
  cut: `casez` arms are `?`-wildcard patterns whose faithful chain form is a *masked*
  comparison — a bigger, separate construct. The simplest-clean-cut-first discipline
  (surfaces 2/6/7) ships the plain equality `CaseMux` -> `if`/`else if` chain now; the
  `CasezMux` masked chain is the recorded follow-up, none retired.
- **Emitting `unique`/`priority` `if` qualifiers.** Rejected for the first cut: a bare
  `if`/`else if` chain is the maximally-portable shape (the probe proves it clean across
  every tool/standard). `unique if` / `priority if` qualifiers are a distinct,
  optionally-richer construct that can land later as their own vetted variant if ever
  desired — adding them now would mix two changes and risk a tool-specific warning.
- **A new IR node (e.g. a `PriorityCaseMux`).** Rejected: the chain is an emit-time
  projection of an existing `CaseMux` — no new IR truth, default-off byte-identical (the
  `mux_if` / `task_emit` / `cone_function` precedent).
- **Generate-then-filter** (emit arbitrary priority chains, then validate/discard).
  Forbidden (`feedback_rules_first_generation`).
- **Changing the default output.** Rejected: opt-in only, even once proven
  downstream-clean (`feedback_never_retire_strategies`).

## Consequences

- ANVIL gains its **eighth** richer-structured emit surface; the default `anvil` build
  and `--artifact dut` stay byte-identical (knob default `0.0`).
- The DUT lane gains a procedural `always_comb` `if`/`else if` **priority chain** — a new
  legal sequential-priority procedural shape to parse / elaborate / lower, distinct from
  the parallel `case`/`casez` selectors and from the seventh surface's single 2:1
  conditional — a new bug-surfacing surface.
- The lane's pattern holds: an emit-projection of an existing valid construct,
  rules-first, default-off / byte-identical, proven downstream-clean before it ships,
  reusing existing render machinery. The `CasezMux` masked chain, nested / multi-level
  `generate`, and `interface` / `modport` each land later as their own decided leaves,
  none retired.

## Open questions (to be resolved at `.17a`)

- The exact `Module` carrier: a `case_mux_if_gates: BTreeSet<NodeId>` of marked
  `CaseMux` gates (the `mux_if_gates` / `task_emit_gates` precedent), iterated in
  `NodeId` order for determinism.
- The exact emitter integration: branch the existing structured-case `always_comb`
  section (`src/emit/sv.rs`) so a marked `CaseMux` emits the `if`/`else if` chain in
  place of the `case ... endcase` body — reusing the same `node_ref` operand refs and
  the same `default` literal — vs a separate section. The in-place branch is the minimal
  change.
- The exact candidate predicate wording: "a `CaseMux` for which
  `render_static_structured_gate` returns `None`" (the dynamic-selector test) — and
  whether to compute that once or re-derive it in the annotation pass.
- The exact knob name + semantics (`case_mux_if_emit_prob` proposed), the metric
  (`num_emitted_case_mux_if_chains` proposed, introspection schema `1.15 -> 1.16`), and
  the downstream-gate scenario shape (`saw_case_mux_if_emit` keyed on the metric, a
  `case_mux_prob`-biased focus config).

## Tree split

`STRUCTURED-EMISSION-EXPANSION` continues (the lane stays `active`):

- **`.16`** (this leaf, design) — decision `0028`: the eighth surface, its
  valid-by-construction discipline, its candidate set, its opt-in knob, its downstream
  gate, and the rejected alternatives. Docs-only.
- **`.17`** (impl, `pending`) — the procedural `always_comb` `if`/`else if` priority-chain
  surface: the `case_mux_if_emit_prob` knob + the gen-time annotation + the emitter
  rendering + the metric + the downstream-clean gate + book/USER_GUIDE/KM. Default-off /
  DUT byte-identical. Pre-split into `.17a` (design-detail, grounded in the real
  structured-case emitter section + the `mux_if_emit.rs` annotation precedent) + `.17b`
  (impl, itself pre-split `.17b.1` live / `.17b.2` metric+gate / `.17b.3` docs) when
  picked.
- **future (`.18`+)** — the `CasezMux` masked priority chain, nested / multi-level
  `generate`, `interface` / `modport`, each a new vetted surface with its own decision
  when picked.

## Links

- Owner doctrine: `feedback_pick_and_roll_at_no_frontier` (autonomous surface selection
  at the no-frontier boundary), `feedback_dont_ask_just_do`.
- Lane / ROADMAP: steering gap 1 (richer structured emission), the structure-first
  ceiling (steering gap 4 — this adds shape, not behaviour).
- Doctrine: `feedback_rules_first_generation` (no generate-then-filter),
  `feedback_never_retire_strategies` (opt-in, default byte-identical),
  `feedback_full_factorization` (reuse the structured-case `always_comb` section + the
  operand-ref rendering + the `mux_if` annotation mechanism; one mechanism, not two).
- Precedents: decision `0027` (the seventh surface — the procedural `always_comb`
  `if`/`else` projection of a 2:1 `Mux` this generalizes to the N-way `CaseMux`) +
  decision `0014` (the single-gate combinational `task automatic` output-var family) +
  decisions `0012` / `0013` / `0015` / `0016` / `0025` (the emit-projection family + the
  separate-knob discipline).
- Reuse / touch points: `src/emit/sv.rs` (the structured-case `always_comb` block — the
  `case ... endcase` body branched to the `if`/`else if` chain; the existing operand-ref
  rendering reused), `src/config.rs` (the `case_mux_if_emit_prob` knob + the
  `--case-mux-if-emit-prob` flag beside `mux_if_emit_prob`), `src/ir/` (a `case_mux_if_emit`
  gen-time-annotation pass beside `mux_if_emit.rs`), `src/metrics.rs`
  (`num_emitted_case_mux_if_chains`, introspection schema `1.15 -> 1.16`),
  `src/bin/tool_matrix.rs` (the `--case-mux-if-gate` downstream gate),
  `book/src/structured-emission.md` (user-facing — extend the chapter).
