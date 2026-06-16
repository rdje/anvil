---
id: structured-emission-second-surface-generate-loop
title: ANVIL's second richer-structured SV surface is a default-off, valid-by-construction `generate for` loop emit-projection of an existing replicated construction
answers:
  - "what is the second STRUCTURED-EMISSION-EXPANSION surface"
  - "does ANVIL emit a generate for loop"
  - "does ANVIL emit a generate block or genvar"
  - "can ANVIL emit a SystemVerilog generate construct in the DUT lane"
  - "what does generate_loop_emit_prob do"
  - "why generate before task or interface"
  - "is ANVIL generate emission default-off and byte-identical"
  - "how does ANVIL emit a generate-for as an emit-projection"
  - "does ANVIL re-render a replication as a generate loop"
  - "what structured SV surface comes after the combinational function"
date: 2026-06-16
status: accepted
tags: [capability, structured-emission, generate, genvar, emission, downstream, valid-by-construction, rules-first, breadth, north-star]
evidence: docs/decisions/0012-structured-emission-first-surface-combinational-function.md; docs/tasks/STRUCTURED-EMISSION-EXPANSION.md; src/emit/sv.rs (to_sv_with_modules — no generate/genvar in the DUT lane today); src/frontend/mod.rs (the Phase-8 `generate if` precedent in the frontend lane); src/ir/function_emit.rs + src/ir/soft_union.rs (the gen-time-annotation emit-projection precedents); empirical tool-acceptance probe (Verilator 5.046 -Wall + Yosys 0.64 both modes + Icarus iverilog -g2012 accept a `generate for` lane unroll and a replication->generate-for projection warning-clean)
---

# 0013 - STRUCTURED-EMISSION-EXPANSION: the second richer-structured surface is a `generate for` loop emit-projection

- Date: 2026-06-16
- Status: accepted
- Tree: `STRUCTURED-EMISSION-EXPANSION.3` (design leaf; picks the second surface,
  splits `.3` + `.4` + future)
- Activated by: explicit owner directive (`2026-06-16`) selecting
  *"structured emission: next surface"* after the first surface (the
  combinational `function automatic`, decision `0012`) closed end-to-end. The
  owner steer named `generate` as the recommended next surface.

## Context

`STRUCTURED-EMISSION-EXPANSION` broadens ANVIL's emitted SystemVerilog past its
flat `module` + per-gate `assign` / `always` + instance shape into richer
*structured* constructs, each a new legal interaction surface a downstream tool
must parse, elaborate, and lower — and so a new place to surface a real bug
(`project_anvil_north_star`). Decision `0012` delivered the **first** such
surface (a default-off combinational `function automatic` emit-projection of an
existing cone) and fixed the lane's pattern: *an emit-projection of an existing
valid construct, rules-first, default-off / byte-identical, proven
downstream-clean before it ships*. The lane's `.3+` plan is `task`, nested
`generate`, and `interface` / `modport`, each its own decided leaf when picked.

Two facts ground the second pick:

- **The DUT emitter has no `generate` / `genvar` today** (`src/emit/sv.rs`
  `to_sv_with_modules` — confirmed by inspection), so a `generate` block is
  genuinely new structural variety for the DUT lane. The **frontend lane**
  (Phase 8, `src/frontend/mod.rs`) already emits a `generate if`, so the
  *project* has a generate precedent, but the *DUT* lane does not.
- **A `generate for` loop is universally downstream-clean.** An empirical probe
  (representative `generate for` lane unrolls + a replication->`generate for`
  projection) is accepted **warning-clean** by Verilator 5.046 (`-Wall
  --lint-only`), **both** repo Yosys modes (`synth -noabc` and the
  `abc -fast; opt -fast; check` path), and Icarus (`iverilog -g2012`) — the
  load-bearing "clean across every repo tool" bar.

## Decision

**The second richer-structured surface is a default-off, opt-in,
valid-by-construction `generate for` loop**, emitted as a
**behaviour-preserving projection of an existing replicated construction**. The
leading concrete source is a **replication** ANVIL already builds — a
`GateOp::Concat` of the `{N{x}}` form (an `N`-fold replication of one operand,
e.g. the `assign concat_1 = {11{or_0}};` ANVIL routinely emits). Such a node is
**index-regular by construction**: bit `g` of the result is exactly `x` (for a
1-bit `x`) or the matching lane of `x`. It is rendered as

```systemverilog
genvar gi;
generate
    for (gi = 0; gi < N; gi++) begin : <label>
        assign <wire>[gi] = <x>;   // unrolls to exactly {N{x}}
    end
endgenerate
```

so the unrolled loop is **byte-equivalent in behaviour** to the inline
replication it replaces. The first cut is a **single-level** `generate for`
(the minimal faithful loop, analogous to decision `0012`'s single-gate
function); **nested / multi-level** `generate` is a recorded follow-up.

### Why `generate for` second (not `task`, `interface`/`modport`, or `generate if`)

- **Universally downstream-clean (verified).** `generate for` elaborates
  cleanly in Verilator, both repo Yosys modes, and Icarus (empirical probe
  above). `interface` / `modport` synthesis support in Yosys is still weak and
  version-inconsistent, which would put the both-Yosys-modes-clean bar at risk
  — it stays deferred (decision `0012`).
- **Real replicated structure (the DUT-stress value).** A `generate for`
  produces genuine repeated structure the elaborator must unroll and lower —
  richer than a `generate if` with a constant predicate (whose untaken branch
  is dead, and which the frontend lane already exercises). It is a real new
  elaboration surface for the DUT lane.
- **Minimal blast radius / emit-projection family.** Projecting an existing
  `{N{x}}` replication into a loop is an emit-time projection — **no new IR
  node, no new whole-module behaviour, default-off byte-identical** — exactly
  the `soft_union` / aggregate / `function_emit` precedent. The unrolled loop is
  the inline replication, so there is nothing to "check and discard."
- **Owner steer.** The owner selected `generate` as the next surface.

### Construction discipline (valid-by-construction, rules-first)

- **Rules-first** (`feedback_rules_first_generation`): selection marks an
  *already-valid* replication node at construction time; the loop is a
  deterministic re-expression of that node, behaviour-preserving by
  construction — never generate-then-filter.
- **Default-off / byte-identical**: a new opt-in probability knob (proposed
  `generate_loop_emit_prob`, default `0.0`, exact name pinned at `.4a`); with it
  off the output is byte-identical and `tests/snapshots.rs` is untouched
  (`feedback_never_retire_strategies`). The marked replication still emits inline
  when the knob is off.
- **No new computed truth**: the loop is a pure re-projection of an existing
  replication (the `soft_union` / aggregate / `function_emit` precedent); the
  structure-first ceiling of decisions `0004` / `0011` is unaffected — this adds
  emission *shape*, not behaviour.

### Downstream gate

A focused repo-owned gate (a `tool_matrix` scenario, templated on
`--function-emit-gate`) forces the knob on over a focused DUT corpus and fails
on coverage gaps unless the emitted `generate for` loops are accepted
**warning-clean** by Verilator + both Yosys modes + Icarus, gated on a
`saw_generate_loop_emit` coverage fact — the same "prove the new surface is
accepted, not just produced" bar the prior breadth lanes hold.

## Decisive test applied

"Does the surface add a new legal structural shape **without** new whole-module
behaviour or a default-output change, and is it reliably accepted by every repo
downstream tool?" A `generate for` that re-expresses an existing replication
passes: it is a richer structural shape, behaviour-preserving, default-off
byte-identical, and broadly synthesizable (empirically verified). `interface` /
`modport` still fails the "every Yosys mode clean" sub-test; nested
multi-level `generate` fails the "minimal blast radius" sub-test for a *first*
cut.

## Rejected alternatives

- **`task` first.** A combinational void `task` is *also* universally clean on
  the current toolchain (the empirical probe accepts an `always_comb`-called
  `task automatic` with a single `ref`/`output` across Verilator + both Yosys +
  Icarus) — so the decision `0012` "weak `task` synth" caution is, more
  precisely, a caution about *multi-output / side-effecting* tasks, not simple
  combinational void ones. `task` therefore remains a strong **next** candidate
  (`.5+`), but `generate for` was the owner steer and gives genuinely *replicated*
  structure (a richer, more distinctive elaboration surface) for comparable
  blast radius. `task` is not retired — it is the leading future surface.
- **`interface` / `modport` first.** Still the weakest / most
  version-inconsistent Yosys synthesis support — high risk against the
  clean-across-both-Yosys-modes bar; larger blast radius. Deferred (decision
  `0012`).
- **`generate if` first (DUT lane).** Synthesizable, but with a constant
  predicate (every ANVIL choice is seed-resolved at construction time) the
  untaken branch is dead — lower DUT-stress value than a real replicated loop —
  and the frontend lane already exercises `generate if`. A constant-predicate
  `generate if` in the DUT lane is a candidate later sub-slice, not the first
  generate cut.
- **Nested / multi-level `generate` in the first cut.** More emitter surgery
  (nested genvar scoping) than a single-level loop for comparable first-cut
  value; a recorded follow-up (the multi-level-cone-was-a-follow-up parallel to
  decision `0012`).
- **A new IR `Generate` / loop node with its own semantics.** Rejected: the loop
  is an *emit-time projection* of an existing replication — no new IR truth,
  default-off byte-identical (the `soft_union` / aggregate / `function_emit`
  precedent). A semantic IR construct would risk the byte-identical contract and
  add truth the structure-first doctrine forbids.
- **Generate-then-filter** (emit arbitrary generate blocks, then
  validate/discard). Forbidden (`feedback_rules_first_generation`).
- **Changing the default output.** Rejected: opt-in only until proven
  downstream-clean, and even then it stays an opt-in knob, not a default
  (`feedback_never_retire_strategies`).

## Consequences

- ANVIL gains its **second** richer-structured emit surface; the default `anvil`
  build and `--artifact dut` stay byte-identical (knob default `0.0`).
- The DUT lane gains a `generate` / `genvar` construct for the first time — a
  new legal structural shape (a `generate for` declaration + genvar elaboration)
  to parse / elaborate / unroll / synthesize — a new bug-surfacing surface.
- The lane's pattern holds: an emit-projection of an existing valid construct,
  rules-first, default-off / byte-identical, proven downstream-clean before it
  ships. `task` (the strongest next candidate), nested / multi-level `generate`,
  and `interface` / `modport` each land later as their own decided leaves, none
  retired.

## Open questions (to be resolved at `.4a`)

- The exact index-regular **source**: the `{N{x}}` replication `Concat` is the
  leading candidate (cleanest pure projection); whether to also cover a `Concat`
  of `N` identical lanes over the same operand, or index-regular replicated
  child instances in the hierarchy lane, is a `.4a` scoping question.
- Whether selection is a **generation-time annotation**
  (`Module.generate_loop_gates`, the `soft_union.rs` / `function_emit.rs`
  precedent — likely) or a pure emit-time pass.
- The `generate for` **rendering** against `to_sv_with_modules`: genvar
  declaration scope, the loop label naming, the loop body (`assign <wire>[gi] =
  <x>;`), and how the call-site inline `assign` is suppressed when the loop is
  emitted.
- The exact knob name + semantics (`generate_loop_emit_prob` proposed) and the
  downstream-gate scenario shape (`saw_generate_loop_emit`).

## Tree split

`STRUCTURED-EMISSION-EXPANSION` continues (the lane stays `active`):

- **`.3`** (this leaf, design) — decision `0013`: the second surface, its
  valid-by-construction discipline, its opt-in knob, its downstream gate, and the
  rejected alternatives. Docs-only.
- **`.4`** (impl, `pending`) — the `generate for` loop surface: the
  `generate_loop_emit_prob` knob + the construction/projection + the emitter
  rendering + the downstream-clean gate + book/USER_GUIDE/KM. Default-off / DUT
  byte-identical. Pre-split into `.4a` (design-detail, grounded in the real
  `to_sv_with_modules` + the replication source) + `.4b` (impl) when picked.
- **future (`.5`+)** — `task` (leading next), nested / multi-level `generate`,
  `interface` / `modport`, each a new vetted surface with its own decision when
  picked.

## Links

- Owner directive: `2026-06-16` (select `generate` as the next structured
  surface).
- Lane / ROADMAP: steering gap 1 (richer structured emission), the structure-first
  ceiling (steering gap 4 — this adds shape, not behaviour).
- Doctrine: `feedback_rules_first_generation` (no generate-then-filter),
  `feedback_never_retire_strategies` (opt-in, default byte-identical).
- Precedents: decision `0012` (the combinational `function automatic`
  emit-projection — the surface pattern this follows) + the `soft_union`
  emit-overlay (`0010`) + the Phase-5b `aggregate_layout` projection (all
  default-off emit-projections in `src/emit/sv.rs`); the Phase-8 `generate if`
  in `src/frontend/mod.rs` (the project's existing generate precedent).
- Reuse / touch points: `src/emit/sv.rs` (`to_sv_with_modules` + the projection
  hooks), `src/config.rs` (the `generate_loop_emit_prob` knob beside
  `function_emit_prob` / `soft_union_slice_prob` / `aggregate_prob`),
  `src/ir/` (a `generate_loop` gen-time-annotation pass beside `function_emit.rs`
  / `soft_union.rs`), `src/bin/tool_matrix.rs` (the downstream gate),
  `book/src/structured-emission.md` (user-facing — extend the chapter).
