---
id: structured-emission-ninth-surface-casez-mux-masked-priority-chain
title: ANVIL's ninth richer-structured SV surface is a default-off, valid-by-construction procedural `always_comb` `if`/`else if` **masked** priority-chain emit-projection of a `CasezMux` gate
answers:
  - "what is the ninth STRUCTURED-EMISSION-EXPANSION surface"
  - "does ANVIL emit a masked if else if priority chain"
  - "can ANVIL render a casez as an if/else if chain"
  - "what does casez_mux_if_emit_prob do"
  - "does ANVIL project a CasezMux to a procedural masked priority chain"
  - "can ANVIL emit a casez statement as a chain of if else if with a mask"
  - "what structured SV surface comes after the case-mux if/else if priority chain"
  - "how does ANVIL emit a wildcard casez priority mux"
  - "why a masked (sel & care_mask) == value comparison instead of ==?"
  - "is the casez ==? wildcard-equality operator accepted by yosys"
  - "is ANVIL casez-mux priority-chain emission default-off and byte-identical"
date: 2026-06-23
status: accepted
tags: [capability, structured-emission, if-else-if, priority-chain, masked-comparison, casez-mux, wildcard, procedural, always-comb, emission, downstream, valid-by-construction, rules-first, breadth, north-star]
evidence: docs/decisions/0028-structured-emission-eighth-surface-case-mux-priority-chain.md (the plain-CaseMux equality priority chain this generalizes to the wildcard CasezMux; it recorded the CasezMux masked chain as the follow-up); src/emit/sv.rs (the CasezMux gate renders today as `always_comb begin casez (sel) SW'b<bits>: name = data; default: name = WIDTH'h0; endcase end` at the structured-case block :724; `render_casez_pattern` :2171 builds the `SW'b<bits>` string with `?` at wildcard positions); src/gen/cone/motifs.rs:832 (`build_casez_patterns`: `wildcard_bits = 1`, arms `(idx << 1, width_mask(1))` ⇒ arms NON-OVERLAPPING by construction, all-wildcard arm impossible); src/metrics.rs:2940 + src/ir/compact.rs:603 (the established `care_mask = (!wildcard_mask) & sel_mask` idiom; match = `(sel & care_mask) == (pattern & care_mask)`); empirical tool-acceptance + simulation-equivalence probe this session (scratchpad/probe — see Context)
---

# 0029 - STRUCTURED-EMISSION-EXPANSION: the ninth richer-structured surface is a procedural `always_comb` `if`/`else if` **masked** priority-chain projection of a `CasezMux`

- Date: 2026-06-23
- Status: accepted
- Tree: `STRUCTURED-EMISSION-EXPANSION.18` (design leaf; picks the ninth surface,
  splits `.18` + `.19` + future)
- Activated by: autonomous PNT selection (`2026-06-23`) at a no-active-frontier
  boundary (`feedback_pick_and_roll_at_no_frontier`), after the eighth surface (the
  procedural `always_comb` `if`/`else if` priority-chain projection of a plain
  `CaseMux`, decision `0028`) closed end-to-end. The wildcard `CasezMux` → **masked**
  `if`/`else if` chain was the explicitly recorded `.18+` follow-up of decision `0028`
  (and earlier of `0027`).

## Context

`STRUCTURED-EMISSION-EXPANSION` broadens ANVIL's emitted SystemVerilog past its flat
`module` + per-gate `assign` / `always` + instance shape into richer *structured*
constructs, each a new legal interaction surface a downstream tool must parse,
elaborate, and lower — and so a new place to surface a real bug
(`project_anvil_north_star`). The lane's pattern is fixed by decisions `0012`
(combinational `function automatic`), `0013`/`0015` (`generate for` loop, 1-bit and
wider lane), `0014` (single-gate combinational `task automatic`), `0016`
(multi-gate-cone `function automatic`), `0025` (multi-output combinational `task
automatic`), `0027` (procedural `always_comb` `if`/`else` projection of a 2:1 `Mux`),
and `0028` (procedural `always_comb` `if`/`else if` priority chain of a plain
`CaseMux`): *an emit-projection of an existing valid construct, rules-first, default-off
/ byte-identical, proven downstream-clean before it ships.*

The eighth surface (decision `0028`) shipped the lane's first **N-way procedural priority
chain** — but only for the **plain equality** `CaseMux`, whose arm labels are distinct
constants tested by a bare `sel == SW'd{i}`. ANVIL's other N-way structured selector,
`GateOp::CasezMux`, is a **wildcard** selector: its arms carry `casez`-style
`?`-wildcard patterns and render today (`src/emit/sv.rs`, the structured-case block
`:724`) as a parallel `casez` statement:

```systemverilog
logic [W-1:0] casez_mux_0;        // structured gate -> declared as a logic var
always_comb begin
    casez (sel)
        SW'b00?: casez_mux_0 = arm_0;
        SW'b01?: casez_mux_0 = arm_1;
        SW'b10?: casez_mux_0 = arm_2;
        default: casez_mux_0 = W'h0;
    endcase
end
```

where operand 0 is the selector `sel` (width `SW`), and operands `1..` come in **chunks
of three** — `(pattern_value_id, wildcard_mask_id, data_id)` — both pattern and mask
being `Node::Constant`s. `render_casez_pattern` (`:2171`) turns each `(value, mask)` pair
into the `SW'b<bits>` label, emitting `?` at every position where the wildcard-mask bit
is set, `1`/`0` elsewhere. The trailing `default` drives `W'h0`.

The generator builds those patterns deterministically (`src/gen/cone/motifs.rs:832`,
`build_casez_patterns`): `wildcard_bits = 1` (fixed), `sel_width = ceil_log2(n_arms) +
1`, and arm `idx` is `(idx << 1, width_mask(1))` — i.e. the **low bit is always the lone
wildcard** and the upper bits hold a distinct `idx` per arm. Two consequences matter
here: the arms are **non-overlapping by construction** (each carries a distinct care-bit
value), and an **all-wildcard arm can never occur** (the care mask always retains the
upper bits), so the masked comparison never degenerates to a constant-true condition.

A **faithful** projection of `casez (sel) pattern: g = data;` into a priority chain needs
a *wildcard-aware* comparison, not the plain equality the eighth surface used. There are
two candidate faithful forms, and a fresh empirical probe this session
(`scratchpad/probe`, grounded in a real ANVIL-emitted `casez_mux_0` block — seed 4 of a
`casez_mux_prob = 1.0` comb-only shape) settles which one ships:

- **Form A — the SystemVerilog wildcard-equality operator `sel ==? pattern`**, reusing
  the existing `SW'b<bits>` `?`-pattern verbatim:
  `if (sel ==? SW'b00?) g = arm_0; else if (sel ==? SW'b01?) g = arm_1; … else g = W'h0;`.
  **DISQUALIFIED:** Yosys `0.64` `read_verilog -sv` **rejects `==?`** with `syntax
  error, unexpected '?'` in **both** repo modes (`synth -noabc` and the `abc -fast` path).
  This fails the lane's non-negotiable "clean across *every* repo tool" bar — the same
  bar that empirically disqualified `interface`/`modport` (decision `0015`). (Verilator
  `5.046` `-Wall` and Icarus `-g2012` accept `==?` cleanly, but one rejecting tool is
  fatal.)
- **Form B — a lowered *masked* equality `(sel & care_mask) == value_masked`**, where
  `care_mask = (~wildcard_mask) & sel_mask` and `value_masked = pattern_value &
  care_mask` (the **existing** match idiom already used by `src/metrics.rs:2940` and
  `src/ir/compact.rs:603`):
  ```systemverilog
  always_comb begin
      if ((sel & 3'h6) == 3'h0) g = arm_0;
      else if ((sel & 3'h6) == 3'h2) g = arm_1;
      else if ((sel & 3'h6) == 3'h4) g = arm_2;
      else g = W'h0;
  end
  ```
  **CLEAN across every repo tool:** Verilator `5.046` `-Wall --lint-only` under
  `--language 1800-2012`, `1800-2017`, **and** `1800-2023`; **both** repo Yosys modes
  (`synth -noabc` and `synth -noabc; abc -fast; opt -fast; stat; check`) with **zero**
  warnings and `check` passing; and Icarus `iverilog -g2012` compile. And it is
  **simulation-equivalent** to the `casez` it replaces: `iverilog` + `vvp` proved the
  masked chain **bit-identical** to the parallel `casez (sel) … default` over the
  **exhaustive** selector × data space (128/128 vectors, 0 mismatches) — plus a separate
  hand-constructed **overlapping-pattern** probe (`00?` / `0??` / `1??`) confirmed the
  `if`/`else if` chain preserves `casez` **first-match priority** in general, not only
  for ANVIL's non-overlapping-by-construction arms (128/128, 0 mismatches).

Form B is therefore the surface. It is exactly the "*masked* comparison (`(sel & mask)
== pattern`)" decision `0028` recorded as the bigger, separate construct deferred from
the eighth surface's first cut.

## Decision

**The ninth richer-structured surface is a default-off, opt-in, valid-by-construction
procedural `always_comb` `if`/`else if` *masked* priority-chain emit-projection of a
`CasezMux` gate** — a behaviour-preserving re-expression of the wildcard N-way selection
the `CasezMux` renders today as a parallel `casez` statement, projected into a priority
chain of **masked** selector-equality tests writing the gate's existing structured
`logic` var. For a marked `CasezMux` gate `g` of width `W`, selector `sel` of width `SW`,
and arms `(value_i, mask_i, data_i)` (`i = 0..k`) it renders

```systemverilog
always_comb begin
    if ((<sel> & SW'h<care_0>) == SW'h<val_0>) <g> = <data_0>;
    else if ((<sel> & SW'h<care_1>) == SW'h<val_1>) <g> = <data_1>;
    ...
    else if ((<sel> & SW'h<care_{k-1}>) == SW'h<val_{k-1}>) <g> = <data_{k-1}>;
    else <g> = W'h0;
end
```

where, per arm `i`, `care_i = (~mask_i) & bitmask(SW)` and `val_i = value_i & care_i`
(both rendered as `SW'h…` hex literals, mirroring the existing `W'h0` default literal),
and `<sel>` / `<data_i>` are the operand refs the emitter **already** resolves for the
`casez` block. The chain tests each arm's masked equality in **ascending arm order** and
falls through to the same `default` value.

The projection is **behaviour-preserving by construction** on two independent grounds:
(1) `(sel & care_i) == val_i` is exactly the `casez_pattern_matches` predicate the
metrics / compact / static-collapse paths already use (`((sel ^ value_i) & care_i) ==
0`), so each arm matches the same selector values as the `casez` label; and (2) a
priority `if`/`else if` chain is first-match-wins, exactly like `casez`, so even
overlapping arms (which ANVIL does not currently build — `build_casez_patterns` makes
arms disjoint) resolve to the same arm. The trailing `else` covers exactly the values the
`casez` left to `default`. (Because `wildcard_bits = 1` always, `care_i` always retains ≥
1 care bit, so no arm degenerates to a constant-true `(sel & 0) == 0` condition.)

### The candidate set (rules-first, valid-by-construction)

- **Candidate = a `Node::Gate` whose op is `GateOp::CasezMux` that actually renders as
  the dynamic `always_comb casez` block** — i.e. one whose selector operand is **not** a
  `Node::Constant` (equivalently, the existing `render_static_structured_gate`
  constant-selector collapse returns `None`). A constant-selector `CasezMux` already
  lowers to a continuous `assign` of the selected arm and is **not** a candidate (there is
  no conditional to re-express; excluding it keeps the chain count exact). This is the
  exact dynamic-selector predicate decision `0028` used for the plain `CaseMux`, applied
  to `CasezMux`.
- **`CaseMux` is owned by the eighth surface; `ForFold` is not a candidate** (it is a
  `for`-loop fold, not a selector). A gate is projected by at most one surface.
- **Minus any gate already marked by a sibling projection.** A `CasezMux` is a structured
  selector and is **not** in the `function_emit` / `task_emit` / `cone_function` /
  `multi_output_task` / `generate_loop` / `soft_union` / `mux_if` / `case_mux_if`
  candidate sets, so the exclusion is vacuous in practice — but the pass still excludes
  any already-marked gate for robustness and runs **last** (after the eight existing
  projections; the established "later pass excludes earlier marks" ordering).
- **Rolled at the call site like every other knob.** The per-gate decision is a seeded
  `gen_bool(prob)` (reproducible; never `thread_rng`). The generator guards the call on
  `Config::casez_mux_if_emit_prob > 0.0`, so the default (`0.0`) draws nothing and marks
  nothing ⇒ byte-identical stream + output.

### Construction discipline (the lane invariants)

- **Rules-first** (`feedback_rules_first_generation`): selection re-expresses an
  *already-valid* `CasezMux` gate at construction time; the masked priority chain is a
  deterministic re-rendering, behaviour-preserving by construction — never
  generate-then-filter.
- **Default-off / byte-identical:** a new opt-in `casez_mux_if_emit_prob` (default `0.0`)
  + its `--casez-mux-if-emit-prob` CLI flag; with it off the output is byte-identical and
  `tests/snapshots.rs` is untouched (`feedback_never_retire_strategies`). An unmarked
  `CasezMux` still emits the inline `casez`.
- **Its own knob (nothing retired).** Separate from `case_mux_if_emit_prob` (which targets
  the plain `CaseMux`) and `mux_if_emit_prob` (the 2:1 `Mux`) so the shipped surfaces stay
  byte-identical (reusing an existing knob rejected — it would change that knob's output
  and blur distinct surfaces; the decision `0016` / `0025` / `0027` / `0028`
  separate-knob precedent).
- **Mutually exclusive with the sibling projections.** A gate is projected by at most one
  of `function_emit` / `generate_loop` / `task_emit` / `multi_output_task` /
  `cone_function` / `soft_union` / `mux_if` / `case_mux_if` / `casez_mux_if`.
- **Combinational only, no output-var/passthrough needed.** The `CasezMux` structured
  gate is **already declared as a `logic` var** and **already written from an
  `always_comb` block** — so, exactly like the eighth surface (and unlike the seventh
  surface's 2:1 `Mux`), this projection only swaps the *body* of an existing `always_comb`
  (`casez … endcase` → `if … else if`). No `<g>__cv` output var and no passthrough
  `assign` are required; `<g>` stays exactly what it is today. The block reads only the
  gate's direct operand refs (the selector + the per-arm data) and writes only `<g>` —
  exactly the `casez` block's read/write set (no soundness cycle risk).
- **No new IR node / no new computed truth.** The masked chain is a pure emit-time
  projection of an existing `CasezMux`; the flat IR body, validators, CSE keys, and
  `canonical_module_signature` are untouched (the `case_mux_if` / `mux_if` / `task_emit`
  precedent). The structure-first ceiling of decisions `0004` / `0011` is unaffected —
  this adds emission *shape*, not behaviour.

### Why a masked N-way priority chain ninth (not `==?`, nested `generate`, or `interface`/`modport`)

- **The `==?` operator is empirically disqualified.** The most concise faithful form
  (`sel ==? SW'b<bits>`) reusing the `?`-pattern verbatim is **rejected by Yosys `0.64`**
  in both repo modes (syntax error). The lowered masked-AND form is the maximally-portable
  shape that every repo tool accepts warning-clean (probe above) — so the surface ships
  Form B and records `==?` as a rejected alternative.
- **Universally downstream-clean (verified).** The procedural `always_comb` `if`/`else if`
  masked priority chain elaborates and synthesizes cleanly in Verilator (`-Wall`, all
  three `--language` standards), both repo Yosys modes (no warnings/errors), and Icarus,
  and is exhaustively sim-equivalent to the `casez` it replaces (probe above).
  `interface` / `modport` is **empirically disqualified** (since `.7`/decision `0015`).
  Nested / multi-level `generate` has **no routine by-construction source**
  (operand-uniqueness CSE shares the inner `{N{x}}` replication, so a doubly-nested
  `{M{{N{x}}}}` essentially never survives factorization).
- **A genuinely new elaboration interaction, distinct from the eighth surface.** The
  eighth surface (`0028`) projects a **plain `CaseMux`** into a chain of **bare-equality**
  tests (`sel == SW'd{i}`); this surface projects a **wildcard `CasezMux`** into a chain
  of **masked-equality** tests (`(sel & care) == val`) — a different IR candidate
  (`CasezMux` vs `CaseMux`), a different emitted shape (a masked compare vs a bare equality),
  and a different source `casez` construct (parallel wildcard-match vs parallel
  equality-match). It is also distinct from the existing `casez` render: a priority
  `if`/`else if` chain of masked compares is a *sequential-priority* construct, where a
  `casez` is *parallel wildcard-match* — synthesis and lint tools take different code
  paths for the two even when the result is identical. Good downstream bug bait, not a
  cosmetic variant.
- **High yield, minimal blast radius / maximal reuse.** `CasezMux` gates are produced by
  the Phase-3 structured selector path (`casez_mux_prob`), so the surface fires readily by
  construction when that path is biased on. The mechanism reuses the **existing
  structured-case `always_comb` section**, the **operand-ref rendering** the `casez` block
  already uses, and the **established `care_mask`/`value_masked` idiom** from
  `metrics.rs` / `compact.rs` — one body-render branch, no parallel machinery
  (`feedback_full_factorization`). Like the eighth surface, it is simpler than the seventh
  (no output-var/passthrough), because the gate is already an `always_comb`-written
  `logic`.

### Downstream gate

A focused repo-owned gate (a `tool_matrix --casez-mux-if-gate` scenario, templated on
`--case-mux-if-gate`) forces `casez_mux_if_emit_prob = 1.0` over a comb-only DUT across
the three construction strategies, with a `casez_mux_prob`-biased focus config (raise
`casez_mux_prob` so dynamic-selector `CasezMux` candidates exist; calibrate the
preempting earlier-rolling selector knobs — `comb_mux_prob` / `case_mux_prob` — exactly as
`--case-mux-if-gate` zeroed `comb_mux_prob`; the precise roll-order calibration is pinned
at `.19`), and fails on coverage gaps unless the emitted masked priority chains are
accepted **warning-clean** by Verilator + both Yosys modes + Icarus, gated on a
`saw_casez_mux_if_emit` coverage fact. Detection: because this surface introduces **no new
identifier token** (it writes the gate's existing var, like the eighth surface), the gate
keys the coverage fact on the per-module metric `num_emitted_casez_mux_if_chains > 0`
(which flows through the per-module `Metrics` into the matrix report), a strictly more
robust signal than a text scan. Like the eighth surface (and unlike the `union soft`
up-opt), a procedural `always_comb if/else if` masked chain is universally synthesizable,
so the gate runs the full tool plan. The exact knob/metric names and calibration are
pinned at `.19`.

## Decisive test applied

"Does the surface add a new legal structural shape **without** new whole-module behaviour
or a default-output change, and is it reliably accepted by every repo downstream tool?" A
procedural `always_comb` `if`/`else if` *masked* priority-chain projection of a `CasezMux`
passes: it is a new sequential-priority masked-compare procedural shape (no surface emits
one — the eighth surface emits bare equalities, not masked compares),
behaviour-preserving, default-off byte-identical, broadly synthesizable + exhaustively
sim-equivalent (empirically verified). The `==?` wildcard-operator form **fails** the
"every repo tool clean" sub-test (Yosys rejects it); `interface` / `modport` still fails
the "every Yosys mode clean" sub-test; nested multi-level `generate` lacks a
by-construction source.

## Rejected alternatives

- **The `==?` wildcard-equality operator (Form A).** Rejected — empirically: Yosys `0.64`
  `read_verilog -sv` rejects `==?` with a syntax error in both repo modes, so it fails the
  lane's "clean across every repo tool" bar. The lowered masked-AND form (`(sel &
  care_mask) == value_masked`) is the maximally-portable shape every tool accepts
  warning-clean.
- **Reusing `case_mux_if_emit_prob` (the eighth-surface knob) or `mux_if_emit_prob`.**
  Rejected — it would change a shipped knob's output and blur distinct surfaces (the plain
  bare-equality `CaseMux` chain vs the wildcard masked `CasezMux` chain); the masked chain
  gets its **own** `casez_mux_if_emit_prob` (the decision `0016` / `0025` / `0027` / `0028`
  separate-knob precedent).
- **Adding a `<g>__cv` output var + passthrough (mirroring the seventh surface).** Rejected
  as unnecessary machinery: a `CasezMux` is *already* an `always_comb`-written `logic` var,
  so the projection swaps only the block body (the eighth-surface precedent).
- **Emitting `unique`/`priority` `if` qualifiers.** Rejected for the first cut: a bare
  `if`/`else if` chain is the maximally-portable shape (the probe proves it clean across
  every tool/standard). `unique if` / `priority if` qualifiers are a distinct,
  optionally-richer construct that can land later as their own vetted variant.
- **A new IR node (e.g. a `PriorityCasezMux`).** Rejected: the masked chain is an
  emit-time projection of an existing `CasezMux` — no new IR truth, default-off
  byte-identical (the `case_mux_if` / `mux_if` / `task_emit` precedent).
- **Generate-then-filter** (emit arbitrary masked chains, then validate/discard).
  Forbidden (`feedback_rules_first_generation`).
- **Changing the default output.** Rejected: opt-in only, even once proven
  downstream-clean (`feedback_never_retire_strategies`).

## Consequences

- ANVIL gains its **ninth** richer-structured emit surface; the default `anvil` build and
  `--artifact dut` stay byte-identical (knob default `0.0`).
- The DUT lane gains a procedural `always_comb` `if`/`else if` **masked priority chain** — a
  new legal sequential-priority masked-compare procedural shape to parse / elaborate /
  lower, distinct from the parallel `casez`/`case` selectors, from the seventh surface's
  single 2:1 conditional, and from the eighth surface's bare-equality chain — a new
  bug-surfacing surface.
- The lane's pattern holds: an emit-projection of an existing valid construct, rules-first,
  default-off / byte-identical, proven downstream-clean before it ships, reusing existing
  render machinery + the established `care_mask` idiom. Nested / multi-level `generate` and
  `interface` / `modport` each land later as their own decided leaves, none retired.

## Open questions (to be resolved at `.19a`)

- The exact `Module` carrier: a `casez_mux_if_gates: BTreeSet<NodeId>` of marked
  `CasezMux` gates (the `case_mux_if_gates` / `mux_if_gates` precedent), iterated in
  `NodeId` order for determinism.
- The exact emitter integration: branch the existing structured-case `always_comb` section
  (`src/emit/sv.rs`, the `GateOp::CasezMux` arm `:724`) so a marked `CasezMux` emits the
  masked `if`/`else if` chain in place of the `casez … endcase` body — reusing the same
  `node_ref` operand refs, computing `care_i`/`val_i` per arm from the pattern/mask
  constants (the `render_casez_pattern` inputs), and the same `W'h0` default — vs a
  separate section. The in-place branch is the minimal change.
- The exact candidate predicate wording: "a `CasezMux` whose selector operand is not a
  `Node::Constant`" (the dynamic-selector test, the inverse of the static-collapse) — and
  whether to share a helper with the eighth surface's analogous predicate.
- Whether to render the masked comparison as `'h` hex (proposed, matching the `W'h0`
  default literal) vs `'b` binary; and whether to factor the per-arm `care`/`val` extraction
  into a small helper reused by the emitter (and possibly aligned with the existing
  `casez_pattern_matches` constants).
- The exact knob name + semantics (`casez_mux_if_emit_prob` proposed), the metric
  (`num_emitted_casez_mux_if_chains` proposed, introspection schema `1.16 → 1.17`), and the
  downstream-gate scenario shape (`saw_casez_mux_if_emit` keyed on the metric, a
  `casez_mux_prob`-biased focus config with the correct earlier-selector-knob zeroing).

## Tree split

`STRUCTURED-EMISSION-EXPANSION` continues (the lane stays `active`):

- **`.18`** (this leaf, design) — decision `0029`: the ninth surface, its
  valid-by-construction discipline, its candidate set, its opt-in knob, its downstream
  gate, the `==?`-disqualified / masked-AND-chosen empirical result, and the rejected
  alternatives. Docs-only.
- **`.19`** (impl, `pending`) — the procedural `always_comb` `if`/`else if` masked
  priority-chain surface: the `casez_mux_if_emit_prob` knob + the gen-time annotation + the
  emitter rendering + the metric + the downstream-clean gate + book/USER_GUIDE/KM.
  Default-off / DUT byte-identical. Pre-split into `.19a` (design-detail, grounded in the
  real structured-case `CasezMux` emitter section + `render_casez_pattern` + the
  `case_mux_if_emit.rs` annotation precedent) + `.19b` (impl, itself pre-split `.19b.1`
  live / `.19b.2` metric+gate / `.19b.3` docs) when picked.
- **future (`.20`+)** — nested / multi-level `generate`, `interface` / `modport`, each a
  new vetted surface with its own decision when picked.

## Links

- Owner doctrine: `feedback_pick_and_roll_at_no_frontier` (autonomous surface selection at
  the no-frontier boundary), `feedback_dont_ask_just_do`.
- Lane / ROADMAP: steering gap 1 (richer structured emission), the structure-first ceiling
  (steering gap 4 — this adds shape, not behaviour).
- Doctrine: `feedback_rules_first_generation` (no generate-then-filter),
  `feedback_never_retire_strategies` (opt-in, default byte-identical),
  `feedback_full_factorization` (reuse the structured-case `always_comb` section + the
  operand-ref rendering + the `case_mux_if` annotation mechanism + the existing
  `care_mask` idiom; one mechanism, not two).
- Precedents: decision `0028` (the eighth surface — the plain-`CaseMux` bare-equality
  priority chain this generalizes to the wildcard `CasezMux`) + decision `0027` (the
  procedural `always_comb` `if`/`else` projection of a 2:1 `Mux`) + decisions `0012` /
  `0013` / `0015` / `0016` / `0025` (the emit-projection family + the separate-knob
  discipline).
- Reuse / touch points: `src/emit/sv.rs` (the structured-case `always_comb` block — the
  `casez … endcase` body branched to the masked `if`/`else if` chain; the existing
  operand-ref rendering + `render_casez_pattern` inputs reused), `src/config.rs` (the
  `casez_mux_if_emit_prob` knob + the `--casez-mux-if-emit-prob` flag beside
  `case_mux_if_emit_prob`), `src/ir/` (a `casez_mux_if_emit` gen-time-annotation pass
  beside `case_mux_if_emit.rs`), `src/metrics.rs` (`num_emitted_casez_mux_if_chains`,
  introspection schema `1.16 → 1.17`), `src/bin/tool_matrix.rs` (the `--casez-mux-if-gate`
  downstream gate), `book/src/structured-emission.md` (user-facing — extend the chapter).
