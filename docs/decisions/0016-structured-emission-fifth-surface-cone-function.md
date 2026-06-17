---
id: structured-emission-fifth-surface-cone-function
title: ANVIL's fifth richer-structured SV surface is a default-off, valid-by-construction multi-gate-cone `function automatic` emit-projection of an existing combinational cone
answers:
  - "what is the fifth STRUCTURED-EMISSION-EXPANSION surface"
  - "does ANVIL emit a multi-statement function automatic"
  - "does ANVIL emit a function with local variables"
  - "can ANVIL wrap a whole combinational cone as one function"
  - "does ANVIL emit a function body with more than one statement"
  - "what structured SV surface comes after the wider-lane generate for"
  - "why a cone function over nested generate or a multi-output task"
  - "is the cone function default-off and byte-identical"
  - "how does ANVIL render a combinational cone as a function automatic"
  - "does cone_function_emit_prob change the default output"
date: 2026-06-17
status: accepted
tags: [capability, structured-emission, function, cone, emission, downstream, valid-by-construction, rules-first, breadth, north-star]
evidence: docs/decisions/0012-structured-emission-first-surface-combinational-function.md (the first surface — a SINGLE gate over its direct operands; the cone/slice-aware projection recorded there as a follow-up); docs/decisions/0013-structured-emission-second-surface-generate-loop.md; docs/decisions/0014-structured-emission-third-surface-combinational-task.md; docs/decisions/0015-structured-emission-fourth-surface-wide-lane-generate-loop.md; docs/tasks/STRUCTURED-EMISSION-EXPANSION.md; src/ir/function_emit.rs (annotate_function_emit_gates + gate_qualifies — the single-gate predicate this deepens; positional params = the gate's DIRECT operands); src/emit/sv.rs (the <wire>__f function automatic decl/body/call rendering this extends to a multi-statement cone body); src/introspect/analyze.rs (output_support — the existing cone-walk to the support-leaf boundary the impl reuses); empirical tool-acceptance probe this session (/tmp/anvil-se9-probe/) — a multi-gate-cone function automatic with function-local logic temps + a topo-ordered statement sequence is accepted with ZERO %Warning by Verilator 5.046 -Wall --lint-only + both repo Yosys 0.64 modes (synth -noabc and abc -fast; opt -fast; check) + Icarus iverilog -g2012, and an iverilog simulation proves it bit-equal to the inline cone over 4000 random vectors; the same probe confirms a multi-output combinational task automatic is also clean + sim-equiv (the strong runner-up, deferred for a policy-laden source) and that nested/multi-level generate is clean but has NO routine by-construction 2D source (full factorization collapses {N{ {M{x}} }})
---

# 0016 - STRUCTURED-EMISSION-EXPANSION: the fifth richer-structured surface is a multi-gate-cone `function automatic` emit-projection

- Date: 2026-06-17
- Status: accepted
- Tree: `STRUCTURED-EMISSION-EXPANSION.9` (design leaf; picks the fifth surface,
  splits `.9` + `.10` + future)
- Activated by: autonomous PNT selection (`2026-06-17`) at a no-active-frontier
  boundary, per the standing owner directive to pick-and-roll at the no-frontier
  boundary (`feedback_pick_and_roll_at_no_frontier`) after the fourth surface (the
  wider-lane `generate for` part-select, decision `0015`) closed end-to-end. The
  richer multi-gate cone was already recorded as a **follow-up to the first
  surface** (decision `0012`: the first cut "wraps a single gate over its direct
  operands"; the cone / slice-aware projection is "a recorded follow-up").

## Context

`STRUCTURED-EMISSION-EXPANSION` broadens ANVIL's emitted SystemVerilog past its
flat `module` + per-gate `assign` / `always` + instance shape into richer
*structured* constructs, each a new legal interaction surface a downstream tool
must parse, elaborate, and lower — and so a new place to surface a real bug
(`project_anvil_north_star`). The lane's pattern is fixed by decisions `0012`
(combinational `function automatic`), `0013` (`generate for` loop), `0014`
(combinational `task automatic`), and `0015` (wider-lane `generate for`
part-select): *an emit-projection of an existing valid construct, rules-first,
default-off / byte-identical, proven downstream-clean before it ships.*

The **first** surface (decision `0012`) deliberately took **only a single gate**
over its **direct operands**: the emitter renders

```systemverilog
function automatic logic [W-1:0] <gate>__f(input logic [W0-1:0] a0, input logic [W1-1:0] a1);
    <gate>__f = a0 <op> a1;        // one statement, no locals
endfunction
assign <gate> = <gate>__f(<operand refs>);
```

decision `0012` itself recorded the **richer cone** — a function wrapping a
*Gate node + its fan-in down to the support-leaf boundary* — as the follow-up it
narrowed away from for the first cut (to get zero sharing hazard quickly). That
recorded follow-up is the natural fifth surface, and the by-construction source
for it is abundant.

A fresh empirical probe this session (`/tmp/anvil-se9-probe/`) settles the choice
of the fifth surface directly across the recorded candidates:

- A **multi-gate-cone `function automatic`** — a function whose **parameters are
  the cone's support leaves** and whose **body is a topo-ordered sequence of
  function-local `logic` temporaries** for the cone's interior gates, returning
  the root — is accepted with **zero `%Warning`** by Verilator 5.046
  (`-Wall --lint-only`), **both** repo Yosys modes (`synth -noabc` and
  `abc -fast; opt -fast; check`), and Icarus (`iverilog -g2012`); and an iverilog
  simulation proves it **bit-equal to the inline cone** over 4000 random vectors.
- A **multi-output combinational `task automatic`** (several sink gates over a
  shared support set, projected to one `task` with several `output` args called
  from `always_comb`) is **also** clean + sim-equiv on the same probe — clearing,
  with fresh evidence, the decision-`0012` "multi-output task" caution (it is the
  strong runner-up).
- **Nested / multi-level `generate`** is clean across the tools, but it still has
  **no routine by-construction source**: ANVIL's replications are
  one-dimensional, and a 2-D replication `{N{ {M{x}} }}` is collapsed by full
  factorization to `{N*M{x}}`, so there is no `{N{ {M{x}} }}` node to project. It
  stays a later surface (confirming decision `0015`'s recorded finding).

So **downstream cleanliness does not discriminate** among the candidates — all
three are universally accepted. The decisive axis is **by-construction source
availability** (the exact axis decision `0015` flagged), and the cone-function
wins it: *every* output / interior gate whose combinational fan-in cone has `>= 2`
internal gates is a candidate, and such cones are pervasive.

## Decision

**The fifth richer-structured surface is a default-off, opt-in,
valid-by-construction multi-gate-cone `function automatic`** — a
**behaviour-preserving deepening of the first surface (decision `0012`)** from a
single gate to an entire combinational cone. For a selected **root** gate whose
combinational fan-in (stopping at the support-leaf boundary — primary inputs /
flop `Q`s / instance outputs / constants, the existing `output_support` boundary)
contains `>= 2` interior gates, the emitter renders

```systemverilog
function automatic logic [W-1:0] <root>__cf(input logic [L0-1:0] p0, input logic [L1-1:0] p1, ...);
    logic [G0-1:0] <g0>;            // one function-local per interior gate,
    logic [G1-1:0] <g1>;            // declared up front
    <g0> = <expr over params / earlier locals>;   // topo-ordered straight-line body
    <g1> = <expr over params / earlier locals>;
    <root>__cf = <expr over params / earlier locals>;   // returns the root
endfunction
assign <root> = <root>__cf(<support-leaf refs>);
```

instead of the inline per-gate `assign` chain. The function-local temporaries are
the cone's interior gates in topological order, the parameters are the cone's
support leaves (positional, like decision `0012`, so duplicate leaves get
distinct slots), and the return value is the root — so the unrolled function is
**behaviour-identical** to the inline cone by construction (verified by the
iverilog equivalence sim). The cone stops at the support-leaf boundary exactly
like `output_support`, so the function never crosses a register edge or an
instance boundary (combinational only).

### Why a multi-gate-cone `function` fifth (not nested `generate`, a multi-output `task`, or `interface`/`modport`)

- **Most abundant by-construction source (the decisive axis).** Any cone with
  `>= 2` interior gates qualifies — pervasive in real generation — whereas the
  multi-output task needs co-supported sink *groups* (a policy-laden grouping
  heuristic) and nested generate needs a 2-D replication ANVIL does not routinely
  build. Decision `0015` made by-construction source the discriminator; the
  cone-function dominates it.
- **A genuinely new emitter shape.** The body becomes a **multi-statement
  procedural function body with function-local declarations** — a real new
  elaboration construct (function-local nets + a straight-line statement
  sequence) the single-gate function never emitted, not a cosmetic variant.
- **Universally downstream-clean (verified).** Zero `%Warning` in Verilator
  `-Wall`, clean in both repo Yosys modes and Icarus, and sim-proven equal to the
  inline cone (empirical probe above). `interface` / `modport` fails the same
  family of probe (decision `0015`: Icarus syntax-fail + both-Yosys implicit-decl
  warnings) and stays deferred.
- **The recorded follow-up of the first surface, with a live source.** Decision
  `0012` explicitly narrowed the first cut to a single gate and recorded the cone
  projection as the follow-up; the generator already builds the cones (they
  currently emit inline as a per-gate `assign` chain), so the surface has real
  candidates the moment the root is marked.
- **Bounded blast radius, reusing proven machinery.** The cone-walk to the
  support-leaf boundary is exactly the existing `src/introspect/analyze.rs`
  `output_support` traversal; the rendering extends the existing `<wire>__f`
  function decl/call path in `src/emit/sv.rs` to a multi-statement body. No new
  IR node, no new whole-module behaviour.

### Construction discipline (valid-by-construction, rules-first)

- **Rules-first** (`feedback_rules_first_generation`): selection marks an
  *already-valid* root gate at construction time (a new
  `annotate_cone_function_gates` roll, the `function_emit.rs` precedent); the
  function is a deterministic re-expression of the cone, behaviour-preserving by
  construction — never generate-then-filter.
- **Its own opt-in knob — the single-gate surface stays byte-identical.** A new
  default-off `cone_function_emit_prob` (proposed; pinned at `.10a`), **separate
  from `function_emit_prob`**, so the shipped single-gate surface and its proofs
  are **untouched** — `function_emit_prob`'s output at any value is exactly as it
  ships today. Nothing is retired (`feedback_never_retire_strategies`): with
  `cone_function_emit_prob = 0.0` the cone still emits as the inline per-gate
  chain and `tests/snapshots.rs` is untouched. (Reusing `function_emit_prob`
  was **rejected** — it would change that knob's existing emitted output and blur
  two distinct surfaces.)
- **Mutually exclusive with the four existing per-gate projections.** A cone
  root absorbs its interior gates into the function body, so those interior gates
  must be excluded from being separately `function_emit` / `generate_loop` /
  `task_emit` / `soft_union` projected (and from being a second cone root). The
  cone pass runs with the sibling marks visible and excludes them — the
  established "later pass excludes earlier marks" ordering. Exact pass ordering is
  an `.10a` detail.
- **Combinational only.** The cone stops at the `output_support` support-leaf
  boundary (primary inputs / flop `Q`s / instance outputs / constants); the
  function never recurses through a register edge or an instance boundary.
- **`Slice` and structured selectors stay excluded** as interior nodes that would
  break the clean-function contract, exactly as in decision `0012` (a full-width
  `Slice` parameter trips `-Wall UNUSEDSIGNAL`); the precise interior-node
  admissibility set is an `.10a` detail. Nothing retired — excluded nodes keep
  emitting inline.
- **No new computed truth**: the function is a pure re-projection of an existing
  cone; the structure-first ceiling of decisions `0004` / `0011` is unaffected —
  this adds emission *shape*, not behaviour.

### Downstream gate

A new repo-owned `tool_matrix --cone-function-gate` (templated on
`--function-emit-gate`) forces `cone_function_emit_prob = 1.0` over comb-only
DUTs across the three construction strategies and fails on coverage gaps unless
the emitted cone functions are accepted **warning-clean** by Verilator + both
Yosys modes + Icarus, gated on a new `saw_cone_function_emit` coverage fact — the
same "prove the new shape is accepted, not just produced" bar the prior surfaces
hold. Because a synthesizable function is accepted by every tool (unlike the
`union soft` up-opt), the gate runs the full plan, not Verilator-only.

## Decisive test applied

"Does the surface add a new legal structural shape **without** new whole-module
behaviour or a default-output change, is it reliably accepted by every repo
downstream tool, and does it have a routine by-construction source?" The
multi-gate-cone function passes all four: it is a richer structural shape (a
multi-statement function body with locals), behaviour-preserving (sim-proven ==
the inline cone), default-off byte-identical, broadly synthesizable (empirically
verified across Verilator + both Yosys + Icarus), and its source (any cone with
`>= 2` interior gates) is pervasive. The multi-output `task` passes the first
three but its source is policy-laden (co-supported sink groups); nested
`generate` passes the first three but fails the source sub-test; `interface` /
`modport` fails the "every tool clean" sub-test.

## Rejected alternatives

- **Multi-output combinational `task automatic` fifth.** Clean + sim-equiv on
  the fresh probe (4000 vectors) — the strong runner-up, and it clears the
  decision-`0012` multi-output-task caution with evidence. Deferred because its
  by-construction source is **policy-laden**: it needs *groups* of sink gates that
  share a support set, plus a grouping heuristic, where the cone-function's source
  (any deep cone) is immediate and pervasive. Recorded as the leading future
  surface; nothing retired.
- **Nested / multi-level `generate` fifth.** Clean across the tools, but **no
  routine by-construction source** — full factorization collapses
  `{N{ {M{x}} }}` to `{N*M{x}}`, so there is no 2-D replication node to project
  (confirms decision `0015`). Deferred again, now with fresh evidence; a later
  `generate`-deepening surface if a 2-D source is ever constructed.
- **`interface` / `modport` fifth.** Empirically disqualified at decision `0015`
  (Icarus syntax-fails the modport port; both Yosys modes warn on the implicit
  interface-member declaration) — it would fail the clean-across-every-tool bar.
  Deferred.
- **Reusing `function_emit_prob` for cones instead of a new knob.** Rejected: it
  would change the existing single-gate knob's emitted output (no longer
  byte-identical for that knob) and blur two distinct surfaces. A separate
  `cone_function_emit_prob` keeps the shipped single-gate surface byte-identical
  and nothing retired — the "one knob per surface, default-off" precedent.
- **A new IR `Function` node with its own semantics.** Rejected: the function is
  an *emit-time projection* of an existing cone — no new IR truth, default-off
  byte-identical (the `function_emit` / `generate_loop` / `task_emit` /
  `soft_union` precedent). A semantic IR construct would risk the byte-identical
  contract and add truth the structure-first doctrine forbids.
- **Generate-then-filter** (emit arbitrary cone functions, then
  validate/discard). Forbidden (`feedback_rules_first_generation`).
- **Changing the default output / retiring the inline per-gate chain.** Rejected:
  opt-in only, even once proven downstream-clean
  (`feedback_never_retire_strategies`).

## Consequences

- ANVIL gains its **fifth** richer-structured emit surface; the default `anvil`
  build and `--artifact dut` stay byte-identical (the new knob defaults `0.0`,
  and the shipped single-gate `function_emit_prob` surface is untouched).
- The DUT lane gains a `function automatic` with **function-local declarations
  and a multi-statement procedural body** — a new legal structural shape to
  parse / elaborate / inline / lower — a new bug-surfacing surface, and it closes
  the recorded cone-projection follow-up of decision `0012`.
- The lane's pattern holds: an emit-projection of an existing valid construct,
  rules-first, default-off / byte-identical, proven downstream-clean before it
  ships. The multi-output `task` (leading next), nested / multi-level `generate`,
  and `interface` / `modport` each land later as their own decided leaves, none
  retired. A new `num_emitted_cone_functions` metric will MINOR-bump the
  introspection schema (`1.10 → 1.11`) at the impl's metric sub-leaf.

## Open questions (to be resolved at `.10a`)

- The exact knob name (`cone_function_emit_prob` proposed) and whether the per-
  module roll marks a root then walks its cone, or walks candidate roots and rolls
  per qualifying cone.
- The precise interior-node admissibility set (which `GateOp`s may be interior
  locals vs. force a cone boundary — e.g. `Slice` / structured selectors /
  already-sibling-marked nodes), and how a fanout interior node (used outside the
  cone) is handled (boundary-stop vs. duplicate-into-the-function).
- The topological-ordering + local-naming scheme for the function body, reusing
  the `output_support` cone-walk in `src/introspect/analyze.rs`.
- The pass ordering relative to `function_emit` / `generate_loop` / `task_emit` /
  `soft_union` and the exact mutual-exclusion bookkeeping.
- Whether `.10` adds a dedicated `--cone-function-gate` + `saw_cone_function_emit`
  (proposed) or extends the existing `--function-emit-gate`.

## Tree split

`STRUCTURED-EMISSION-EXPANSION` continues (the lane stays `active`):

- **`.9`** (this leaf, design) — decision `0016`: the fifth surface, its
  valid-by-construction discipline, its own opt-in knob, its downstream gate, and
  the rejected alternatives (with the fresh empirical probe). Docs-only.
- **`.10`** (impl, `pending`) — the multi-gate-cone `function automatic`: the
  `cone_function_emit_prob` knob + the rules-first cone selection + the
  multi-statement emitter rendering (function-local decls + topo body + call) +
  the `num_emitted_cone_functions` metric (schema `1.10 → 1.11`) + the
  downstream-clean gate + book/USER_GUIDE/KM. Default-off / DUT byte-identical.
  Pre-split into `.10a` (design-detail, grounded in the real `function_emit.rs` +
  `analyze.rs` cone-walk + `to_sv_with_modules`) + `.10b` (impl) when picked.
- **future (`.11`+)** — multi-output `task` (leading next), nested / multi-level
  `generate`, `interface` / `modport`, each a new vetted surface with its own
  decision when picked.

## Links

- Standing owner directive: pick-and-roll at the no-frontier boundary
  (`feedback_pick_and_roll_at_no_frontier`) — autonomous selection of the next
  structured surface.
- Lane / ROADMAP: steering gap 1 (richer structured emission), the structure-first
  ceiling (steering gap 4 — this adds shape, not behaviour).
- Doctrine: `feedback_rules_first_generation` (no generate-then-filter),
  `feedback_never_retire_strategies` (opt-in, default byte-identical, separate
  knob so nothing is retired).
- Precedents: decision `0012` (the single-gate `function automatic` this deepens;
  it recorded the cone projection as the follow-up) + decisions `0013` / `0014` /
  `0015` (the `generate for` / `task automatic` / wider-lane sibling surfaces) +
  the `soft_union` overlay (`0010`) + the Phase-5b `aggregate_layout` projection
  (all default-off emit-projections in `src/emit/sv.rs`).
- Reuse / touch points: `src/ir/function_emit.rs` (the annotation precedent — a
  new sibling `annotate_cone_function_gates`), `src/introspect/analyze.rs`
  (`output_support` — the cone-walk to the support-leaf boundary), `src/emit/sv.rs`
  (the `<wire>__f` function decl/call path — extend to a multi-statement body),
  `src/config.rs` (the `cone_function_emit_prob` knob beside `function_emit_prob`),
  `src/metrics.rs` + `src/introspect/mod.rs` (`num_emitted_cone_functions`, schema
  `1.10 → 1.11`), `src/bin/tool_matrix.rs` (the `--cone-function-gate`),
  `book/src/structured-emission.md` (user-facing — the fifth-surface section at
  the impl closeout).
