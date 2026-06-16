---
id: structured-emission-first-surface-combinational-function
title: ANVIL's first richer-structured SV surface is a default-off, valid-by-construction combinational `function automatic` emit-projection of an existing cone
answers:
  - "can ANVIL emit a SystemVerilog function"
  - "does ANVIL emit function automatic or task bodies"
  - "what is the first STRUCTURED-EMISSION-EXPANSION surface"
  - "how does ANVIL emit richer structured SystemVerilog"
  - "what does function_emit_prob do"
  - "is ANVIL function emission default-off and byte-identical"
  - "why does ANVIL not emit interfaces or modports first"
  - "what structured SV surfaces does ANVIL support"
  - "does ANVIL wrap a combinational cone as a function"
  - "is the function emit surface rules-first or generate-then-filter"
date: 2026-06-16
status: accepted
tags: [capability, structured-emission, function, emission, downstream, valid-by-construction, rules-first, breadth, north-star]
evidence: docs/decisions/0012-structured-emission-first-surface-combinational-function.md; docs/tasks/STRUCTURED-EMISSION-EXPANSION.md; src/emit/sv.rs (to_sv_with_modules + the aggregate_layout projection + soft_union_slice_overlay precedents); src/ir/soft_union.rs; src/config.rs (the default-off emit-projection knobs aggregate_prob / soft_union_slice_prob); docs/decisions/0010-sv-version-first-upopt-soft-packed-union.md; docs/decisions/0011-semantic-introspection-derived-query-surface.md
---

# 0012 - STRUCTURED-EMISSION-EXPANSION: the first richer-structured surface is a combinational `function automatic` emit-projection

- Date: 2026-06-16
- Status: accepted
- Tree: `STRUCTURED-EMISSION-EXPANSION.1` (design leaf; activates the lane, splits
  `.1` + `.2` + future)
- Activated by: explicit owner directive (`2026-06-16`) selecting
  `STRUCTURED-EMISSION-EXPANSION` as the next lane after
  `SEMANTIC-INTROSPECTION-EXPANSION` delivered all four named query kinds.

## Context

ROADMAP steering gap 1: ANVIL's emitted SystemVerilog is structurally **flat** —
`module` + per-gate `assign` (or `always_comb`) + flop `always_ff` +
child-instance instantiations + output drives (`src/emit/sv.rs`
`to_sv_with_modules`). Downstream parsers / elaborators / synth tools therefore
only ever see that one structural shape. Richer **structured** SV constructs —
`function` / `task` bodies, `interface` / `modport` boundaries, nested / multi-level
`generate` — are each a *new legal interaction surface* a tool must parse,
elaborate, and lower, and so a new place to surface a downstream bug
(`project_anvil_north_star`). The lane must pick the **first** such surface that is
(a) synthesizable, (b) reliably downstream-clean across **all** repo tools
(Verilator + both Yosys modes + Icarus), and (c) minimal blast radius.

Two default-off **emit-projection** precedents already exist and define the safe
shape for this lane:

- **Phase-5b packed aggregate** — keyed on `Module.aggregate_layout` (default
  `None` ⇒ byte-identical); the emitter projects a `struct` typedef + one
  aggregate port + boundary aliases over the same flat ports.
- **The `union soft` up-opt** (decision `0010`) — `soft_union_slice_overlay(m,
  idx, sv_version)` consulted per node during gate emission, gated on
  `soft_union_slice_prob` + `sv_version`; default-off ⇒ byte-identical.

Both add **zero new whole-module behaviour**: they re-render an *existing, already
valid* construction in a richer surface form. That is the template this lane
follows.

## Decision

**The first richer-structured surface is a default-off, opt-in,
valid-by-construction combinational `function automatic`** emitted as a
**behaviour-preserving projection of an existing combinational cone**. A selected
combinational gate node — together with its fan-in cone, stopping at the same
support-leaf boundary the `output_support` analysis uses (primary inputs, flop
`Q`s, child-instance outputs, constants; never crossing a register edge or an
instance boundary) — is rendered as a `function automatic logic [W-1:0]
<name>(...)` whose parameter list is the cone's support leaves and whose body is
the straight-line evaluation of the cone's internal gates (topological order),
returning the cone root. The original use site becomes a call `<name>(<actual
support refs>)`.

### Why a combinational `function` first (not interface/modport or generate)

- **Universally downstream-clean.** Automatic combinational functions are inlined
  cleanly by Verilator, **both** repo Yosys modes, and Icarus — the load-bearing
  bar (`feedback`/north-star). SystemVerilog **`interface` / `modport`** synthesis
  support in Yosys is weak and version-inconsistent, putting the "clean across both
  Yosys modes" bar at real risk; it is deferred.
- **Minimal blast radius.** It is an emit-time projection (the aggregate /
  `soft_union` precedent) — **no new IR node, no new generator semantic truth, no
  whole-module behaviour**; default-off ⇒ byte-identical. **Nested `generate`**,
  though synthesizable, is more emitter surgery (genvar scoping, loop bounds) for
  comparable value; deferred.
- **A real new structural surface.** A function declaration + a call the tools must
  parse, elaborate, and inline is genuinely new structural variety, not a cosmetic
  rewrite.
- **Synergy with the introspection cone.** The function's parameter list is
  exactly a cone's **support leaves** — the relation decision `0011`'s
  `output_support` already computes — so the construction reuses an
  already-understood, already-tested boundary.

### Construction discipline (valid-by-construction, rules-first)

- **Rules-first** (`feedback_rules_first_generation`): the function wraps a cone
  that is *already valid* in the flat emission; selection happens at construction
  time, never generate-then-filter. The body is a deterministic re-expression of
  the cone's existing operations — behaviour-preserving by construction (the
  function returns exactly the cone's value), so there is nothing to "check and
  discard."
- **Default-off / byte-identical**: a new `function_emit_prob` knob, default `0.0`;
  with it off the output is byte-identical and `tests/snapshots.rs` is untouched
  (`feedback_never_retire_strategies`). Combinational only — `function automatic`
  bodies carry no clock/sequential logic; a flop `Q` is a leaf parameter, never
  recursed through.
- **No new computed truth**: the function is a pure re-projection of an existing
  cone (the `soft_union`/aggregate emit-projection precedent; the structure-first
  ceiling of decisions `0004`/`0011` is unaffected — this adds emission *shape*,
  not behaviour).

### Downstream gate

A focused repo-owned gate (a `tool_matrix` scenario or a dedicated test bank)
proves the emitted functions are accepted **warning-clean** by Verilator + both
Yosys modes + Icarus, gated on a `saw_combinational_function_emit` coverage fact —
the same "prove the new surface is accepted, not just produced" bar the prior
breadth lanes hold.

## Decisive test applied

"Does the surface add a new legal structural shape **without** new whole-module
behaviour or a default-output change, and is it reliably accepted by every repo
downstream tool?" A combinational `function automatic` that re-expresses an
existing cone passes: it is a richer structural shape, behaviour-preserving,
default-off byte-identical, and broadly synthesizable. Interface/modport fails the
"every Yosys mode clean" sub-test today; nested generate fails the "minimal blast
radius" sub-test.

## Rejected alternatives

- **`interface` / `modport` first.** Highest structural novelty but weakest /
  most version-inconsistent Yosys synthesis support — high risk against the
  clean-across-both-Yosys-modes bar; larger blast radius (port + connection
  emission). Deferred to a later, separately-decided leaf.
- **Nested / multi-level `generate` first.** Synthesizable but more emitter
  surgery (genvar scoping, loop bounds) than a combinational function for
  comparable first-cut value. Deferred.
- **`task` first.** A `task` is for procedural / multi-output / side-effecting
  bodies; a combinational `function` (single return) is the simpler, more
  uniformly synthesizable first cut. `task` is a candidate future sub-slice.
- **A new IR `Function` node with its own semantics.** Rejected: the function is
  an *emit-time projection* of an existing cone — no new IR truth, default-off
  byte-identical (the `soft_union`/aggregate precedent). A semantic IR construct
  would risk the byte-identical contract and add truth the structure-first doctrine
  forbids.
- **Generate-then-filter** (emit arbitrary functions, then validate/discard).
  Forbidden (`feedback_rules_first_generation`).
- **Changing the default output.** Rejected: opt-in only until proven
  downstream-clean, and even then it stays an opt-in knob, not a default
  (`feedback_never_retire_strategies`).

## Consequences

- ANVIL gains its **first** richer-structured emit surface; the default `anvil`
  build and `--artifact dut` stay byte-identical (knob default `0.0`).
- Downstream tools gain a new legal structural shape (a `function automatic`
  declaration + call) to parse / elaborate / inline / synthesize — a new
  bug-surfacing surface.
- The lane's pattern is fixed for the future surfaces: **an emit-projection of an
  existing valid construct, rules-first, default-off / byte-identical, proven
  downstream-clean before it ships** — `task`, nested `generate`, and
  `interface`/`modport` each land later as their own decided leaves, none retired.

## Open questions (resolved at `.2a`)

- The exact cone-selection rule (which gate nodes qualify; size / depth bounds so
  the function is non-trivial yet bounded) and how the selection stays rules-first.
- The function signature + body rendering against `to_sv_with_modules` (local
  declarations vs a single return expression; width/`logic` typing of parameters).
- Whether selection is a generation-time annotation (the `soft_union.rs` /
  `aggregate_layout` precedent) or a pure deterministic emit-time pass.
- The `function_emit_prob` knob's exact semantics and the downstream-gate scenario
  shape (`saw_combinational_function_emit`).

## Tree split

`STRUCTURED-EMISSION-EXPANSION` is activated:

- **`.1`** (this leaf, design) — decision `0012`: the first surface, its
  valid-by-construction discipline, its opt-in knob, its downstream gate, and the
  rejected alternatives. Docs-only.
- **`.2`** (impl, `pending`) — the combinational `function automatic` surface:
  the `function_emit_prob` knob + the construction/projection + the emitter
  rendering + the downstream-clean gate + book/USER_GUIDE/KM. Default-off / DUT
  byte-identical. Pre-split into `.2a` (design-detail, grounded in the real
  `to_sv_with_modules`) + `.2b` (impl) when picked.
- **future (`.3`+)** — `task`, nested `generate`, `interface`/`modport`, each a new
  vetted surface with its own decision when picked.

## Links

- Owner directive: `2026-06-16` (activate `STRUCTURED-EMISSION-EXPANSION` as the
  next lane).
- Lane / ROADMAP: steering gap 1 (richer structured emission), the structure-first
  ceiling (steering gap 4 — this adds shape, not behaviour).
- Doctrine: `feedback_rules_first_generation` (no generate-then-filter),
  `feedback_never_retire_strategies` (opt-in, default byte-identical).
- Precedents: decision `0010` (the `soft_union` emit-overlay) + the Phase-5b
  `aggregate_layout` projection (both default-off emit-projections in
  `src/emit/sv.rs`); decision `0011` (`output_support` — the cone-support boundary
  the function's parameter list reuses).
- Reuse / touch points: `src/emit/sv.rs` (`to_sv_with_modules` + the projection
  hooks), `src/config.rs` (the `function_emit_prob` knob beside `aggregate_prob` /
  `soft_union_slice_prob`), `src/bin/tool_matrix.rs` (the downstream gate),
  `book/src/` (user-facing).
