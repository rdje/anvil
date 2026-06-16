---
id: structured-emission-third-surface-combinational-task
title: ANVIL's third richer-structured SV surface is a default-off, valid-by-construction combinational `task automatic` emit-projection of an existing combinational gate
answers:
  - "what is the third STRUCTURED-EMISSION-EXPANSION surface"
  - "does ANVIL emit a task automatic"
  - "can ANVIL emit a SystemVerilog task in the DUT lane"
  - "what does task_emit_prob do"
  - "why task after function and generate"
  - "is ANVIL task emission default-off and byte-identical"
  - "how does ANVIL emit a combinational task as an emit-projection"
  - "does ANVIL call a task from always_comb"
  - "what structured SV surface comes after the generate for loop"
date: 2026-06-16
status: accepted
tags: [capability, structured-emission, task, emission, downstream, valid-by-construction, rules-first, breadth, north-star]
evidence: docs/decisions/0012-structured-emission-first-surface-combinational-function.md; docs/decisions/0013-structured-emission-second-surface-generate-loop.md; docs/tasks/STRUCTURED-EMISSION-EXPANSION.md; src/ir/function_emit.rs + src/ir/generate_loop.rs + src/ir/soft_union.rs (the gen-time-annotation emit-projection precedents); src/emit/sv.rs (to_sv_with_modules — the function-decl / generate-block sections + the per-gate assign loop); empirical tool-acceptance probe this session (Verilator 5.046 -Wall + Yosys 0.64 both modes + Icarus iverilog -g2012 accept a combinational `task automatic` called from `always_comb`, in both the direct-output form and the minimal-blast-radius output-var + passthrough-assign form, warning-clean)
---

# 0014 - STRUCTURED-EMISSION-EXPANSION: the third richer-structured surface is a combinational `task automatic` emit-projection

- Date: 2026-06-16
- Status: accepted
- Tree: `STRUCTURED-EMISSION-EXPANSION.5` (design leaf; picks the third surface,
  splits `.5` + `.6` + future)
- Activated by: autonomous PNT selection (`2026-06-16`) at a no-active-frontier
  boundary, by explicit owner directive (*"Pick any tree and roll with it, you
  decide the best route"*) after the second surface (the `generate for` loop,
  decision `0013`) closed end-to-end. `task` was already recorded as the
  **leading future candidate** in decision `0013`.

## Context

`STRUCTURED-EMISSION-EXPANSION` broadens ANVIL's emitted SystemVerilog past its
flat `module` + per-gate `assign` / `always` + instance shape into richer
*structured* constructs, each a new legal interaction surface a downstream tool
must parse, elaborate, and lower — and so a new place to surface a real bug
(`project_anvil_north_star`). The lane's pattern is fixed by decisions `0012`
(combinational `function automatic`) and `0013` (`generate for` loop): *an
emit-projection of an existing valid construct, rules-first, default-off /
byte-identical, proven downstream-clean before it ships.*

Decision `0012` deferred `task` citing "weak `task` synth", but decision `0013`
**narrowed that caution with evidence**: a *simple combinational void* `task`
is universally clean on the current toolchain — the weakness is specific to
*multi-output / side-effecting / multi-statement* tasks. `task` was therefore
recorded as the lane's **leading future** surface.

A fresh empirical probe this session confirms it directly. A combinational
`task automatic` with one `output` and positional `input`s, called from an
`always_comb`, is accepted **warning-clean** by Verilator 5.046 (`-Wall
--lint-only`), **both** repo Yosys modes (`synth -noabc` and `abc -fast; opt
-fast; check`), and Icarus (`iverilog -g2012`) — in *both* a direct-output form
(the task writes the module output) and a minimal-blast-radius form (the task
writes a local `logic` var, and a continuous `assign` drives the gate's net
from that var).

## Decision

**The third richer-structured surface is a default-off, opt-in,
valid-by-construction combinational `task automatic`**, emitted as a
**behaviour-preserving projection of a single combinational gate** — the exact
parallel of decision `0012`'s combinational function, but expressed as a
procedural `task` with an `output` argument rather than a value-returning
`function`. For a marked gate `<wire> = op(o0, o1, …)` of width `W` it renders

```systemverilog
task automatic <wire>__t(output logic [W-1:0] o, input logic [W0-1:0] a0, ...);
    o = a0 <op> a1 ...;
endtask
...
logic [W-1:0] <wire>__tv;
always_comb <wire>__t(<wire>__tv, <operand refs>);
assign <wire> = <wire>__tv;   // the gate's net, unchanged downstream
```

so the task call computes exactly the gate's value into `<wire>__tv`, and the
existing `<wire>` net is driven from it. The first cut is a **single-gate
"operand task"** (the minimal projection, the decision `0012` single-gate
parallel): the operands are already module-level wires/literals, so there is
zero sharing/scoping hazard; only the marked gate's own drive changes. The
exact net-vs-var integration (the output-var + passthrough-assign form above,
or making `<wire>` itself the procedural var) is pinned at `.6a`.

### Why `task` third (not nested `generate` or `interface`/`modport`)

- **Universally downstream-clean (verified).** The combinational `task`
  elaborates and synthesizes cleanly in Verilator, both repo Yosys modes, and
  Icarus (empirical probe above). `interface` / `modport` synthesis support in
  Yosys is still weak and version-inconsistent, which would put the
  both-Yosys-modes-clean bar at risk — it stays deferred (decisions `0012` /
  `0013`).
- **A genuinely distinct elaboration surface.** A `task` is *procedural*
  (called from `always_comb`, writes through an `output`/`ref` argument) where
  the `function` is a continuous-assign value — a real second "named reusable
  computation" form for a tool to lower, not a cosmetic variant.
- **Minimal blast radius / emit-projection family.** Projecting one already-valid
  combinational gate into a task is an emit-time projection — **no new IR node,
  no new whole-module behaviour, default-off byte-identical** — exactly the
  `function_emit` / `generate_loop` / `soft_union` / aggregate precedent.
- **The recorded leading candidate.** Decision `0013` already named `task` the
  next surface; this leaf executes that, grounded in a fresh probe.

### Construction discipline (valid-by-construction, rules-first)

- **Rules-first** (`feedback_rules_first_generation`): selection marks an
  *already-valid* combinational gate at construction time; the task is a
  deterministic re-expression of that gate, behaviour-preserving by
  construction — never generate-then-filter.
- **Default-off / byte-identical**: a new opt-in probability knob (proposed
  `task_emit_prob`, default `0.0`, exact name pinned at `.6a`); with it off the
  output is byte-identical and `tests/snapshots.rs` is untouched
  (`feedback_never_retire_strategies`). The marked gate still emits inline when
  the knob is off.
- **Mutually exclusive with the sibling projections.** A gate is projected by at
  most one of `function_emit` / `generate_loop` / `task_emit` / `soft_union`; the
  task pass runs after the others and excludes already-marked gates (the
  established "later pass excludes earlier marks" ordering).
- **Combinational only.** A flop `Q` is a leaf parameter; the task never recurses
  through a register edge or instance boundary. Structured selectors
  (`CaseMux` / `CasezMux` / `ForFold`) and `Slice` are excluded (the same
  reasons as `function_emit`: structured selectors have their own procedural
  rendering; a full-width `Slice` param trips `-Wall UNUSEDSIGNAL`). Nothing
  retired — excluded gates still emit inline.
- **No new computed truth**: the task is a pure re-projection of an existing
  gate; the structure-first ceiling of decisions `0004` / `0011` is unaffected —
  this adds emission *shape*, not behaviour.

### Downstream gate

A focused repo-owned gate (a `tool_matrix --task-emit-gate` scenario, templated
on `--function-emit-gate` / `--generate-loop-gate`) forces `task_emit_prob =
1.0` over a comb-only DUT across the three construction strategies and fails on
coverage gaps unless the emitted tasks are accepted **warning-clean** by
Verilator + both Yosys modes + Icarus, gated on a `saw_combinational_task_emit`
coverage fact — the same "prove the new surface is accepted, not just produced"
bar the prior surfaces hold. Like a function (and unlike the `union soft`
up-opt), a combinational `task` is universally synthesizable, so the gate runs
the full tool plan.

## Decisive test applied

"Does the surface add a new legal structural shape **without** new whole-module
behaviour or a default-output change, and is it reliably accepted by every repo
downstream tool?" A combinational `task` that re-expresses a single gate passes:
it is a richer procedural shape, behaviour-preserving, default-off
byte-identical, and broadly synthesizable (empirically verified). `interface` /
`modport` still fails the "every Yosys mode clean" sub-test; nested multi-level
`generate` is the deeper variant of an already-shipped surface and is its own
later sub-slice.

## Rejected alternatives

- **`interface` / `modport` third.** Still the weakest / most
  version-inconsistent Yosys synthesis support — high risk against the
  clean-across-both-Yosys-modes bar; larger blast radius. Deferred (decisions
  `0012` / `0013`).
- **Nested / multi-level `generate` third.** A natural deepening of decision
  `0013`'s single-level loop, but it is more emitter surgery (nested genvar
  scoping) and reuses an already-shipped *kind* of surface; `task` adds a
  genuinely new procedural surface for comparable blast radius. Recorded as a
  `generate`-deepening follow-up.
- **A multi-output / side-effecting `task` in the first cut.** This is exactly
  the form decisions `0012` / `0013` cautioned about (weaker, less-consistent
  synth). The first cut is a single-output combinational task only; richer task
  bodies are a recorded follow-up.
- **A multi-gate-cone task body.** Like the function surface, the multi-level
  cone (private internals as task locals) is the harder sharing-aware version;
  the single-gate operand task is the first cut.
- **A new IR `Task` node with its own semantics.** Rejected: the task is an
  *emit-time projection* of an existing gate — no new IR truth, default-off
  byte-identical (the `function_emit` / `generate_loop` precedent).
- **Generate-then-filter** (emit arbitrary tasks, then validate/discard).
  Forbidden (`feedback_rules_first_generation`).
- **Changing the default output.** Rejected: opt-in only, even once proven
  downstream-clean (`feedback_never_retire_strategies`).

## Consequences

- ANVIL gains its **third** richer-structured emit surface; the default `anvil`
  build and `--artifact dut` stay byte-identical (knob default `0.0`).
- The DUT lane gains a `task` / `always_comb`-call construct — a new legal
  procedural shape to parse / elaborate / lower — a new bug-surfacing surface.
- The lane's pattern holds: an emit-projection of an existing valid construct,
  rules-first, default-off / byte-identical, proven downstream-clean before it
  ships. Nested / multi-level `generate` and `interface` / `modport` each land
  later as their own decided leaves, none retired.

## Open questions (to be resolved at `.6a`)

- The exact net-vs-var integration: the output-var + passthrough-`assign` form
  (keeps `<wire>` a net, minimal downstream change) vs making `<wire>` itself a
  procedural `logic` var driven by the `always_comb` task call. Both probed
  clean; the var+passthrough form is the leading first cut (smaller blast
  radius, parallels the `function_emit` "only the gate's own drive changes").
- Whether each task call gets its own `always_comb` or whether task calls share
  one `always_comb` block.
- Whether selection is a **generation-time annotation**
  (`Module.task_emit_gates`, the `function_emit.rs` / `generate_loop.rs`
  precedent — likely) or a pure emit-time pass.
- The exact knob name + semantics (`task_emit_prob` proposed) and the
  downstream-gate scenario shape (`saw_combinational_task_emit`).

## Tree split

`STRUCTURED-EMISSION-EXPANSION` continues (the lane stays `active`):

- **`.5`** (this leaf, design) — decision `0014`: the third surface, its
  valid-by-construction discipline, its opt-in knob, its downstream gate, and the
  rejected alternatives. Docs-only.
- **`.6`** (impl, `pending`) — the combinational `task automatic` surface: the
  `task_emit_prob` knob + the construction/projection + the emitter rendering +
  the downstream-clean gate + book/USER_GUIDE/KM. Default-off / DUT
  byte-identical. Pre-split into `.6a` (design-detail, grounded in the real
  `to_sv_with_modules` + the gate source) + `.6b` (impl) when picked.
- **future (`.7`+)** — nested / multi-level `generate`, `interface` / `modport`,
  richer (multi-output) tasks, each a new vetted surface with its own decision
  when picked.

## Links

- Owner directive: `2026-06-16` (*"Pick any tree and roll with it, you decide the
  best route"*) — autonomous selection of the next structured surface.
- Lane / ROADMAP: steering gap 1 (richer structured emission), the structure-first
  ceiling (steering gap 4 — this adds shape, not behaviour).
- Doctrine: `feedback_rules_first_generation` (no generate-then-filter),
  `feedback_never_retire_strategies` (opt-in, default byte-identical).
- Precedents: decision `0012` (the combinational `function automatic`
  emit-projection — the single-gate surface pattern this follows) + decision
  `0013` (the `generate for` loop; named `task` the leading next candidate) + the
  `soft_union` overlay (`0010`) + the Phase-5b `aggregate_layout` projection (all
  default-off emit-projections in `src/emit/sv.rs`).
- Reuse / touch points: `src/emit/sv.rs` (`to_sv_with_modules` + the projection
  sections), `src/config.rs` (the `task_emit_prob` knob beside
  `function_emit_prob` / `generate_loop_emit_prob`), `src/ir/` (a `task_emit`
  gen-time-annotation pass beside `function_emit.rs` / `generate_loop.rs`),
  `src/bin/tool_matrix.rs` (the downstream gate), `book/src/structured-emission.md`
  (user-facing — extend the chapter).
