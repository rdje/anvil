---
id: structured-emission-fourth-surface-wide-lane-generate-loop
title: ANVIL's fourth richer-structured SV surface is a default-off, valid-by-construction wider-lane `generate for` part-select emit-projection of an existing `{N{x}}` replication
answers:
  - "what is the fourth STRUCTURED-EMISSION-EXPANSION surface"
  - "does ANVIL emit a wider-lane generate for loop"
  - "can ANVIL emit a generate for loop with a part-select body"
  - "does ANVIL emit an indexed part-select inside a generate loop"
  - "does ANVIL handle a multi-bit lane replication as a generate loop"
  - "what structured SV surface comes after the task automatic"
  - "why a wider-lane generate-for over nested generate or interface modport"
  - "is the wider-lane generate loop default-off and byte-identical"
  - "how does ANVIL render a W-bit replication as a generate for loop"
date: 2026-06-17
status: accepted
tags: [capability, structured-emission, generate, genvar, part-select, emission, downstream, valid-by-construction, rules-first, breadth, north-star]
evidence: docs/decisions/0012-structured-emission-first-surface-combinational-function.md; docs/decisions/0013-structured-emission-second-surface-generate-loop.md; docs/decisions/0014-structured-emission-third-surface-combinational-task.md; docs/tasks/STRUCTURED-EMISSION-EXPANSION.md; src/ir/generate_loop.rs (annotate_generate_loop_gates + gate_qualifies — the existing 1-bit-lane predicate this leaf broadens); src/emit/sv.rs (generate_loop_gate + render_generate_loop_block — the existing `assign <wire>[gi] = <x>;` body this leaf extends to `assign <wire>[gi*LW +: LW] = <x>;`); empirical tool-acceptance probe this session (Verilator 5.046 -Wall --lint-only + Yosys 0.64 both modes + Icarus iverilog -g2012 accept a wider-lane `generate for` part-select warning-clean, and iverilog simulation proves the unrolled loop is bit-equal to `{4{b}}`; the same probe DISQUALIFIES interface/modport — Icarus syntax-fails the modport port and Yosys warns on implicit `.data` declaration)
---

# 0015 - STRUCTURED-EMISSION-EXPANSION: the fourth richer-structured surface is a wider-lane `generate for` part-select emit-projection

- Date: 2026-06-17
- Status: accepted
- Tree: `STRUCTURED-EMISSION-EXPANSION.7` (design leaf; picks the fourth surface,
  splits `.7` + `.8` + future)
- Activated by: autonomous PNT selection (`2026-06-17`) at a no-active-frontier
  boundary, per the standing owner directive to pick-and-roll at the no-frontier
  boundary (`feedback_pick_and_roll_at_no_frontier`) after the third surface (the
  combinational `task automatic`, decision `0014`) closed end-to-end. The
  wider-lane `generate for` part-select was already recorded as a **follow-up to
  the second surface** (decision `0013` / `book/src/structured-emission.md`: *"A
  wider lane would need a part-select body and stays inline — a recorded
  follow-up"*).

## Context

`STRUCTURED-EMISSION-EXPANSION` broadens ANVIL's emitted SystemVerilog past its
flat `module` + per-gate `assign` / `always` + instance shape into richer
*structured* constructs, each a new legal interaction surface a downstream tool
must parse, elaborate, and lower — and so a new place to surface a real bug
(`project_anvil_north_star`). The lane's pattern is fixed by decisions `0012`
(combinational `function automatic`), `0013` (`generate for` loop), and `0014`
(combinational `task automatic`): *an emit-projection of an existing valid
construct, rules-first, default-off / byte-identical, proven downstream-clean
before it ships.*

The second surface (decision `0013`) deliberately took **only the 1-bit-lane**
`{N{x}}` replication — where bit `g` of the result is exactly the lane `x`, so
the loop body `assign <wire>[gi] = <x>;` is byte-faithful. It explicitly
**excluded the wider lane** (`{N{x}}` where `x` is `LW > 1` bits): that case is
still index-regular but needs a *part-select* body, and it was recorded as a
follow-up — *nothing retired; the wider replication still emits inline*
(`src/ir/generate_loop.rs::gate_qualifies` returns `false` when the lane width
is not 1, and the generator already constructs such wider replications, e.g. the
`{4{byte}}` shape exercised in that pass's `wide_lane_replication_does_not_qualify`
test).

A fresh empirical probe this session settles the choice of the fourth surface
directly:

- A wider-lane `generate for` over an indexed part-select body —
  `for (gi=0; gi<N; gi=gi+1) assign w[gi*LW +: LW] = x;` — is accepted
  **warning-clean** by Verilator 5.046 (`-Wall --lint-only`), **both** repo
  Yosys modes (`synth -noabc` and `abc -fast; opt -fast; check`), and Icarus
  (`iverilog -g2012`); and an iverilog simulation proves the unrolled loop is
  **bit-equal to `{N{x}}`** across sampled inputs.
- `interface` / `modport` is **disqualified by the same probe**: Icarus
  syntax-fails the `modport`-typed port, and *both* Yosys modes warn on the
  implicit declaration of the `interface`-member reference (`\p.data` /
  `\intf.data`). This confirms the recorded weak / version-inconsistent
  downstream-support claim (decisions `0012` / `0013` / `0014`) with current
  tools.
- Nested / multi-level `generate` is clean across the tools, but it is a
  *bigger* emitter change (nested genvar scoping) and lacks a clean
  by-construction source today (ANVIL's replications are 1-dimensional;
  `{N{ {M{x}} }}` is not a routine construction), so it stays a later surface.

## Decision

**The fourth richer-structured surface is a default-off, opt-in,
valid-by-construction wider-lane `generate for` part-select** — a
**behaviour-preserving broadening of the second surface (decision `0013`)** from
the 1-bit lane to a lane of any width `LW >= 1`. For a marked replication gate
`<wire> = {N{x}}` whose lane `x` is `LW` bits (so the result is `N*LW` bits) it
renders

```systemverilog
genvar <wire>__gi;
generate
    for (<wire>__gi = 0; <wire>__gi < N; <wire>__gi = <wire>__gi + 1) begin : <wire>__gen
        assign <wire>[<wire>__gi*LW +: LW] = <x>;
    end
endgenerate
```

instead of the inline `assign <wire> = {N{x}};`. Bit-group `g` of `{N{x}}` —
bits `[g*LW +: LW]` — is *exactly* the lane `x`, so the unrolled loop is
**byte-equivalent** to the inline replication; the projection is
behaviour-preserving by construction. The `LW == 1` case keeps the existing
`assign <wire>[<wire>__gi] = <x>;` body verbatim, so the already-shipped
1-bit-lane surface and its proofs stay **byte-identical** (the part-select form
is taken only when `LW > 1`).

### Why a wider-lane `generate for` fourth (not nested `generate` or `interface`/`modport`)

- **Universally downstream-clean (verified).** The wider-lane part-select loop
  elaborates and synthesizes clean in Verilator, both repo Yosys modes, and
  Icarus, and is simulation-proven equal to `{N{x}}` (empirical probe above).
  `interface` / `modport` fails the same probe (Icarus syntax-fail + both-Yosys
  implicit-decl warnings) — it stays deferred.
- **A genuinely new emitter shape.** The body becomes an **indexed part-select
  write with a genvar-computed base** (`<wire>[gi*LW +: LW]`) — a real new
  elaboration construct (a common real-RTL idiom) the 1-bit loop never emitted,
  not a cosmetic variant.
- **Minimal blast radius / emit-projection family.** It broadens one existing
  pass: relax `gate_qualifies` from "1-bit lane" to "`LW >= 1` lane with
  `width == N*LW`" and add one `LW > 1` branch to `render_generate_loop_block`.
  **No new IR node, no new whole-module behaviour, no new knob (it reuses
  `generate_loop_emit_prob`), no new metric (it reuses
  `num_emitted_generate_loops`), and no introspection schema bump** —
  default-off byte-identical, exactly the
  `function_emit` / `generate_loop` / `task_emit` / `soft_union` precedent.
- **The recorded follow-up, with a live by-construction source.** Decision
  `0013` and the book already named the wider lane the next step, and the
  generator already builds wider-lane `{N{x}}` replications (they currently emit
  inline), so the surface has real candidates the `--generate-loop-gate` will
  exercise the moment the predicate is relaxed.

### Construction discipline (valid-by-construction, rules-first)

- **Rules-first** (`feedback_rules_first_generation`): selection marks an
  *already-valid* wider-lane replication at construction time (the same
  `annotate_generate_loop_gates` roll, now with a relaxed predicate); the loop is
  a deterministic re-expression of `{N{x}}`, behaviour-preserving by
  construction — never generate-then-filter.
- **Default-off / byte-identical**: the surface reuses the existing
  `generate_loop_emit_prob` (default `0.0`); with it off the output is
  byte-identical and `tests/snapshots.rs` is untouched
  (`feedback_never_retire_strategies`). The wider replication still emits inline
  when the knob is off, and the `LW == 1` rendering is unchanged.
- **Mutually exclusive with the sibling projections.** Unchanged: a replication
  marked for `function_emit` is excluded here, and the generate-loop pass runs
  after `function_emit` (the established "later pass excludes earlier marks"
  ordering). A wider replication is not a `task_emit` / `soft_union` candidate.
- **Combinational only.** Unchanged: the lane `x` is rendered with the normal
  module-level `node_ref`; the loop never recurses through a register edge or
  instance boundary.
- **No new computed truth**: the loop is a pure re-projection of an existing
  replication gate; the structure-first ceiling of decisions `0004` / `0011` is
  unaffected — this adds emission *shape*, not behaviour.

### Downstream gate

The existing repo-owned `tool_matrix --generate-loop-gate` already forces
`generate_loop_emit_prob = 1.0` over comb-only DUTs across the three
construction strategies and fails on coverage gaps unless the emitted loops are
accepted **warning-clean** by Verilator + both Yosys modes + Icarus, gated on
the `saw_generate_loop_emit` coverage fact. Once the predicate admits wider
lanes, that gate covers them automatically; `.8` adds a focused assertion (and,
if warranted, a dedicated wider-lane coverage signal) so the wider-lane case is
*proven exercised*, not merely *possible* — the same "prove the new shape is
accepted, not just produced" bar the prior surfaces hold.

## Decisive test applied

"Does the surface add a new legal structural shape **without** new whole-module
behaviour or a default-output change, and is it reliably accepted by every repo
downstream tool?" The wider-lane part-select loop passes: it is a richer
structural shape (a variable-base indexed part-select write), behaviour-preserving
(sim-proven `== {N{x}}`), default-off byte-identical, and broadly synthesizable
(empirically verified across Verilator + both Yosys + Icarus). `interface` /
`modport` fails the "every tool clean" sub-test (Icarus + Yosys); nested
multi-level `generate` is a deeper variant of an already-shipped surface and
lacks a routine by-construction source, so it is its own later sub-slice.

## Rejected alternatives

- **`interface` / `modport` fourth.** Empirically disqualified this session:
  Icarus syntax-fails the modport-typed port and both Yosys modes warn on the
  implicit `interface`-member declaration — it would fail the
  clean-across-every-tool bar. Larger blast radius. Deferred (now with fresh
  evidence, not only the inherited caution).
- **Nested / multi-level `generate` fourth.** Clean across the tools, but a
  bigger emitter change (nested genvar scoping) and it lacks a clean
  by-construction source (ANVIL's replications are 1-dimensional). Recorded as a
  later `generate`-deepening surface; the wider-lane part-select is the
  smaller-blast-radius, recorded-follow-up next step.
- **Constant-predicate `generate if` fourth.** Clean across the tools, but it
  introduces a *dead untaken branch* (unused logic) and the source-level
  frontend lane already exercises `generate if`; lower marginal value than a new
  emitter shape in the DUT lane. Deferred.
- **A wider-lane body via an explicit per-bit unroll instead of a part-select.**
  Rejected: the part-select `<wire>[gi*LW +: LW]` is the idiomatic, compact,
  index-regular form and is exactly what makes the loop a genuinely new shape; a
  bit-by-bit inner loop would be the nested-generate surface, not this one.
- **A new IR node / new knob / new metric / a schema bump.** Rejected: the
  wider lane is an *emit-time broadening* of an existing projection — it reuses
  `generate_loop_emit_prob` and `num_emitted_generate_loops` and adds no IR
  truth, so it is default-off byte-identical with no schema change (the
  `function_emit` / `generate_loop` / `task_emit` precedent).
- **Generate-then-filter** (emit arbitrary loops, then validate/discard).
  Forbidden (`feedback_rules_first_generation`).
- **Changing the default output / retiring the inline wider replication.**
  Rejected: opt-in only, even once proven downstream-clean
  (`feedback_never_retire_strategies`).

## Consequences

- ANVIL gains its **fourth** richer-structured emit surface; the default `anvil`
  build and `--artifact dut` stay byte-identical (knob default `0.0`, and the
  1-bit-lane rendering is unchanged).
- The DUT lane gains a wider-lane `generate for` with an indexed part-select
  body — a new legal structural shape to parse / elaborate / unroll / lower — a
  new bug-surfacing surface, and it closes the recorded wider-lane follow-up.
- The lane's pattern holds: an emit-projection of an existing valid construct,
  rules-first, default-off / byte-identical, proven downstream-clean before it
  ships. Nested / multi-level `generate`, `interface` / `modport`, and richer
  tasks each land later as their own decided leaves, none retired.

## Open questions (to be resolved at `.8a`)

- The exact return shape of `generate_loop_gate` — it currently returns
  `(lane, N)`; the wider lane needs the lane width `LW` too (either returned, or
  recomputed in `render_generate_loop_block` from `m.nodes[lane].width()`).
- Whether `render_generate_loop_block` branches on `LW == 1` vs `LW > 1`
  (keeping the 1-bit body byte-identical) or always emits the part-select form
  for `LW >= 1` (`[gi*1 +: 1]` is legal but would change the byte output of the
  shipped 1-bit surface — so the branch is the leading choice).
- Whether `.8` adds a dedicated wider-lane coverage signal /
  `--generate-loop-gate` scenario, or asserts the wider lane inside the existing
  gate, to *prove* the wider lane is exercised (not merely possible).

## Tree split

`STRUCTURED-EMISSION-EXPANSION` continues (the lane stays `active`):

- **`.7`** (this leaf, design) — decision `0015`: the fourth surface, its
  valid-by-construction discipline, its reuse of the existing knob / metric /
  gate, and the rejected alternatives (with the fresh empirical probe). Docs-only.
- **`.8`** (impl, `pending`) — the wider-lane `generate for` part-select: relax
  `gate_qualifies` (1-bit lane → `LW >= 1`, `width == N*LW`) + the
  `render_generate_loop_block` part-select branch + lib proofs (wider-lane mark;
  wider-lane emit shape; `LW == 1` still `[gi]` byte-identical; sim-faithful) +
  the wider-lane `--generate-loop-gate` proof + book/USER_GUIDE. Default-off /
  DUT byte-identical. Pre-split into `.8a` (design-detail, grounded in the real
  `generate_loop.rs` + `render_generate_loop_block`) + `.8b` (impl) when picked.
- **future (`.9`+)** — nested / multi-level `generate`, `interface` / `modport`,
  richer (multi-output) tasks, each a new vetted surface with its own decision
  when picked.

## Links

- Standing owner directive: pick-and-roll at the no-frontier boundary
  (`feedback_pick_and_roll_at_no_frontier`) — autonomous selection of the next
  structured surface.
- Lane / ROADMAP: steering gap 1 (richer structured emission), the structure-first
  ceiling (steering gap 4 — this adds shape, not behaviour).
- Doctrine: `feedback_rules_first_generation` (no generate-then-filter),
  `feedback_never_retire_strategies` (opt-in, default byte-identical).
- Precedents: decision `0013` (the 1-bit-lane `generate for` loop this leaf
  broadens; it recorded the wider lane as the follow-up) + decision `0012` (the
  combinational `function automatic`) + decision `0014` (the combinational
  `task automatic`) + the `soft_union` overlay (`0010`) + the Phase-5b
  `aggregate_layout` projection (all default-off emit-projections in
  `src/emit/sv.rs`).
- Reuse / touch points: `src/ir/generate_loop.rs` (`gate_qualifies` — relax the
  lane-width restriction), `src/emit/sv.rs` (`generate_loop_gate` +
  `render_generate_loop_block` — the part-select branch),
  `src/bin/tool_matrix.rs` (the existing `--generate-loop-gate`, extend the
  proof to the wider lane), `book/src/structured-emission.md` (user-facing —
  replace the "wider lane stays inline" caveat with the shipped wider-lane
  surface). No `src/config.rs` / `src/metrics.rs` / `src/introspect/` change
  needed (reuses the existing knob + metric; no schema bump).
