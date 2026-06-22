---
id: structured-emission-sixth-surface-multi-output-task
title: ANVIL's sixth richer-structured SV surface is a default-off, valid-by-construction multi-output combinational `task automatic` co-emitting a mutually-independent group of co-supported gates
answers:
  - "what is the sixth STRUCTURED-EMISSION-EXPANSION surface"
  - "does ANVIL emit a multi-output task"
  - "can ANVIL emit a task with multiple output arguments"
  - "what does multi_output_task_emit_prob do"
  - "does ANVIL emit a task automatic with more than one output"
  - "can ANVIL co-emit two gates that share an input into one task"
  - "what structured SV surface comes after the multi-gate-cone function"
  - "how does ANVIL emit a co-supported-sink task"
  - "why multi-output task after the cone function"
  - "is ANVIL multi-output task emission default-off and byte-identical"
date: 2026-06-22
status: accepted
tags: [capability, structured-emission, task, multi-output, emission, downstream, valid-by-construction, rules-first, breadth, north-star]
evidence: docs/decisions/0014-structured-emission-third-surface-combinational-task.md (the single-gate `task automatic` surface this generalizes); docs/decisions/0016-structured-emission-fifth-surface-cone-function.md (the deduplicated-shared-formal cone-render primitives reused: cone_function_params / cone_operand_ref / render_cone_gate_expr); docs/tasks/STRUCTURED-EMISSION-EXPANSION.md (the `.9` probe recorded the multi-output task as the deferred runner-up); src/ir/task_emit.rs + src/emit/sv.rs (render_gate_task_decl / render_gate_task_call / render_cone_gate_expr); empirical tool-acceptance + simulation-equivalence probe this session (Verilator 5.046 -Wall accepts a 2-output combinational `task automatic` called once from `always_comb` warning-clean under --language 1800-2012/2017/2023; Yosys 0.64 synth -noabc and abc -fast both clean; iverilog -g2012 compiles; iverilog vvp proves the 2-output task bit-equal to the inline two assigns over 5000 random vectors)
---

# 0025 - STRUCTURED-EMISSION-EXPANSION: the sixth richer-structured surface is a multi-output combinational `task automatic`

- Date: 2026-06-22
- Status: accepted
- Tree: `STRUCTURED-EMISSION-EXPANSION.11` (design leaf; picks the sixth surface,
  splits `.11` + `.12` + future)
- Activated by: autonomous PNT selection (`2026-06-22`) at a no-active-frontier
  boundary (`feedback_pick_and_roll_at_no_frontier`), after the fifth surface (the
  multi-gate-cone `function automatic`, decision `0016`) closed end-to-end. The
  multi-output `task` was already recorded as the **deferred runner-up** in the
  `.9` probe (decision `0016`) — empirically clean + sim-equiv but deferred there
  because its "co-supported-sink" source is policy-laden while the cone function's
  source (any cone with `>= 2` interior gates) is pervasive.

## Context

`STRUCTURED-EMISSION-EXPANSION` broadens ANVIL's emitted SystemVerilog past its
flat `module` + per-gate `assign` / `always` + instance shape into richer
*structured* constructs, each a new legal interaction surface a downstream tool
must parse, elaborate, and lower — and so a new place to surface a real bug
(`project_anvil_north_star`). The lane's pattern is fixed by decisions `0012`
(combinational `function automatic`), `0013`/`0015` (`generate for` loop, 1-bit
and wider lane), `0014` (single-gate combinational `task automatic`), and `0016`
(multi-gate-cone `function automatic`): *an emit-projection of an existing valid
construct, rules-first, default-off / byte-identical, proven downstream-clean
before it ships.*

Decision `0014` shipped the **single-output** combinational `task automatic`
(one gate → one `output` arg) and explicitly recorded **"richer (multi-output)
tasks"** as a future vetted surface (`.7+`), cautioning that the original
"weak task synth" worry was specific to multi-output / side-effecting tasks. The
`.9` probe (decision `0016`) then **narrowed that caution with evidence**: a
multi-output combinational `task` is clean + simulation-equivalent on the current
toolchain; it was deferred only because the cone function had the more pervasive
by-construction source.

A fresh empirical probe this session confirms it directly with the installed
tools. A combinational `task automatic` with **two `output` arguments** and a
**deduplicated `input` list** in which one formal is shared by both outputs,
called once from an `always_comb`, is accepted **warning-clean** by Verilator
5.046 (`-Wall --lint-only`) under `--language 1800-2012`, `1800-2017`, and
`1800-2023`, by **both** repo Yosys modes (`synth -noabc` and `abc -fast; opt
-fast; check`), and by Icarus (`iverilog -g2012`), and `iverilog`+`vvp` prove the
two-output task **bit-equal to the inline two `assign`s** over 5000 random
vectors.

## Decision

**The sixth richer-structured surface is a default-off, opt-in,
valid-by-construction multi-output combinational `task automatic`** — a
behaviour-preserving **generalization of the third surface** (decision `0014`'s
single-gate task) from one `output` to several: a **co-supported group** of
qualifying combinational gates is co-emitted into **one** `task automatic` whose
**deduplicated input formals** carry their shared support, called once from an
`always_comb`. For a group of member gates `g0, g1, …` (the first cut is a
**pair**, `k = 2`) it renders

```systemverilog
task automatic <g0>__mt(output logic [W0-1:0] o0, output logic [W1-1:0] o1,
                        input logic [..] a0, input logic [..] a1, ...);
    o0 = <op0 over the shared formals a*>;
    o1 = <op1 over the shared formals a*>;
endtask
...
logic [W0-1:0] <g0>__mtv;
logic [W1-1:0] <g1>__mtv;
always_comb <g0>__mt(<g0>__mtv, <g1>__mtv, <deduped operand refs>);
assign <g0> = <g0>__mtv;   // each member's net, unchanged downstream
assign <g1> = <g1>__mtv;
```

instead of the two inline `assign <g0> = …;` / `assign <g1> = …;`. The input
formals are the **deduplicated union** of the members' non-constant direct
operands (ascending `NodeId`); a **shared** operand becomes **one** formal feeding
multiple outputs (the genuine "co-supported sink"); a `Constant` operand folds
inline as a literal (the cone-function precedent). Each output `oj` is the member
gate's exact operation over those formals, so the task computes exactly the
members' values into the per-member `<gj>__mtv` vars, and the existing `<gj>` nets
are driven from them — **behaviour-preserving by construction.** First cut group
size is a **pair** (`k = 2`); wider groups are a recorded follow-up.

### The soundness rule: members must be mutually fan-in-independent

A group is admissible only when **no member lies in another member's fan-in cone**
(transitively). This is the multi-output analogue of the cone function's
single-use rule. If member `gb` were (transitively) in member `ga`'s fan-in, then
`gb`'s net — driven by the shared task's `<gb>__mtv` passthrough — would feed,
through gates outside the task, into a direct operand the task reads, closing a
combinational cycle through the single `always_comb` task call (a Verilator
`UNOPTFLAT` even though it converges functionally). Requiring mutual fan-in
independence makes the co-emitted task cycle-free by construction. Because the IR
maintains the topological invariant that a gate's operands always have strictly
smaller `NodeId` than the gate (`Module::intern_gate` appends after its operands),
the check is a cheap bounded backward DFS: for a pair `(ga, gb)` with `ga < gb`,
`ga` cannot be in `gb`'s fan-in only if it is absent from `gb`'s operand cone, and
`gb` (larger id) is never in `ga`'s fan-in. The exact mechanism is pinned at
`.12a`.

### The grouping policy (rules-first, valid-by-construction)

- **Candidate set = the single-gate task candidate set** (the decision `0014`
  `gate_qualifies`: a computational `Node::Gate` that is not a procedural
  structured block — `CaseMux` / `CasezMux` / `ForFold` — not a `Slice`
  bit-select, with `>= 1` operand) **minus** any gate already marked by a sibling
  projection.
- **Pairing rule:** scanning candidates in ascending `NodeId`, pair the lowest
  ungrouped candidate `ga` with the next ungrouped candidate `gb` such that (1)
  they **share at least one non-constant direct operand** (so the deduplicated
  task genuinely has a shared input formal — without this it is merely two
  unrelated tasks fused, no new interaction), and (2) they are **mutually
  fan-in-independent** (the soundness rule). Each gate is used by at most one
  group.
- **Rolled at the call site like every other knob.** The per-leader decision is a
  seeded `gen_bool(prob)` (reproducible; never `thread_rng`). The generator guards
  the call on `Config::multi_output_task_emit_prob > 0.0`, so the default (`0.0`)
  draws nothing and groups nothing ⇒ byte-identical stream + output.

### Construction discipline (the lane invariants)

- **Rules-first** (`feedback_rules_first_generation`): selection groups
  *already-valid* combinational gates at construction time; the task is a
  deterministic re-expression, behaviour-preserving by construction — never
  generate-then-filter.
- **Default-off / byte-identical:** a new opt-in `multi_output_task_emit_prob`
  (default `0.0`) + its `--multi-output-task-emit-prob` CLI flag; with it off the
  output is byte-identical and `tests/snapshots.rs` is untouched
  (`feedback_never_retire_strategies`). Grouped members still emit inline when the
  knob is off.
- **Its own knob (nothing retired).** Separate from `task_emit_prob` so the shipped
  single-gate task surface stays byte-identical (reusing `task_emit_prob` rejected
  — it would change that knob's output and blur two surfaces).
- **Mutually exclusive with the sibling projections.** A gate is projected by at
  most one of `function_emit` / `generate_loop` / `task_emit` /
  `multi_output_task` / `cone_function` / `soft_union`. The multi-output pass runs
  **after** `task_emit` (excludes its single-gate marks) and **before**
  `cone_function` (which is extended to exclude multi-output members as roots /
  interiors) — the established "later pass excludes earlier marks" ordering.
- **Combinational only.** Each member is a combinational gate; a flop `Q` is a
  leaf formal; the task never recurses through a register edge or instance
  boundary. Structured selectors and `Slice` are excluded (same reasons as
  `task_emit`). Nothing retired — excluded gates still emit inline.
- **No new IR node / no new computed truth.** The task is a pure emit-time
  projection of existing gates; the flat IR body, validators, CSE keys, and
  `canonical_module_signature` are untouched (the `task_emit` / `cone_function`
  precedent). The structure-first ceiling of decisions `0004` / `0011` is
  unaffected — this adds emission *shape*, not behaviour.

### Why a multi-output task sixth (not nested `generate` or `interface`/`modport`)

- **Universally downstream-clean (verified).** The multi-output combinational
  `task` elaborates and synthesizes cleanly in Verilator (`-Wall`, all three
  `--language` standards), both repo Yosys modes, and Icarus, and is
  sim-equivalent (probe above). `interface` / `modport` synthesis in Yosys is
  still weak and version-inconsistent (deferred since decisions `0012`–`0015`);
  nested / multi-level `generate` has no routine by-construction 2D source
  (factorization collapses `{N{{M{x}}}}`; decision `0016`).
- **A genuinely new elaboration interaction.** Multiple `output` formals on a task
  plus a **shared input formal feeding several outputs** is an uncommon construct
  (good downstream bug bait) and a real second multi-sink procedural form — not a
  cosmetic variant of the single-output task.
- **Minimal blast radius / maximal reuse.** It is an emit-time projection (no new
  IR node, default-off byte-identical) that **reuses the cone-function emitter
  primitives** for the deduplicated shared-formal body (`cone_function_params`-style
  operand dedup, `cone_operand_ref`, `render_cone_gate_expr` with an empty interior
  set) and the single-gate task candidate predicate — one body-render family, one
  selection predicate, no parallel machinery (`feedback_full_factorization`).
- **The recorded runner-up.** The `.9` probe (decision `0016`) already named the
  multi-output task the deferred next candidate and verified it clean + sim-equiv;
  this leaf executes that, grounded in a fresh installed-tool probe.

### Downstream gate

A focused repo-owned gate (a `tool_matrix --multi-output-task-gate` scenario,
templated on `--task-emit-gate` / `--cone-function-gate`) forces
`multi_output_task_emit_prob = 1.0` over a comb-only DUT across the three
construction strategies and fails on coverage gaps unless the emitted tasks are
accepted **warning-clean** by Verilator + both Yosys modes + Icarus, gated on a
`saw_multi_output_task_emit` coverage fact (detected from the emitted SV text via
the `__mt(` token, distinct from the single-gate `__t(` and the cone `__cf(`).
Like a single-gate task (and unlike the `union soft` up-opt), a multi-output
combinational task is universally synthesizable, so the gate runs the full tool
plan.

## Decisive test applied

"Does the surface add a new legal structural shape **without** new whole-module
behaviour or a default-output change, and is it reliably accepted by every repo
downstream tool?" A multi-output combinational `task` co-emitting a
mutually-independent, co-supported group passes: it is a richer procedural shape
(multiple outputs + a shared input formal), behaviour-preserving, default-off
byte-identical, broadly synthesizable + sim-equivalent (empirically verified).
`interface` / `modport` still fails the "every Yosys mode clean" sub-test; nested
multi-level `generate` lacks a by-construction source.

## Rejected alternatives

- **Positional, non-deduplicated formals (two single-gate tasks fused).** Listing
  each member's operands positionally with no dedup (a shared operand passed
  twice into two distinct formals) is trivially simple but adds **no new
  interaction** over emitting two single-gate tasks — it is not a genuine
  co-supported sink. Rejected in favour of the deduplicated shared-formal form
  (the "co-supported-sink" essence) which reuses the cone-function dedup
  primitives.
- **Grouping gates that share only a constant operand.** Constants fold inline as
  literals, so a shared constant yields no shared *formal* — no real co-support.
  The pairing rule requires a shared **non-constant** operand.
- **Grouping fan-in-dependent members.** Co-emitting a member that lies in
  another's fan-in closes a combinational cycle through the shared `always_comb`
  task (`UNOPTFLAT`). Excluded by the mutual-fan-in-independence soundness rule.
- **Wider groups (`k > 2`) in the first cut.** A pair is the minimal multi-output
  form and keeps the selection bounded and reviewable; wider co-supported groups
  are a recorded follow-up (`.13+`), none retired.
- **Reusing `task_emit_prob`.** Rejected — it would change the shipped single-gate
  task knob's output and blur two surfaces; the multi-output task gets its **own**
  `multi_output_task_emit_prob` (the decision `0016` separate-knob precedent).
- **A new IR `Task` node with its own semantics.** Rejected: the task is an
  emit-time projection of existing gates — no new IR truth, default-off
  byte-identical (the `task_emit` / `cone_function` precedent).
- **Generate-then-filter** (emit arbitrary multi-output tasks, then
  validate/discard). Forbidden (`feedback_rules_first_generation`).
- **Changing the default output.** Rejected: opt-in only, even once proven
  downstream-clean (`feedback_never_retire_strategies`).

## Consequences

- ANVIL gains its **sixth** richer-structured emit surface; the default `anvil`
  build and `--artifact dut` stay byte-identical (knob default `0.0`).
- The DUT lane gains a multi-output `task` with a shared input formal feeding
  several outputs — a new uncommon legal procedural shape to parse / elaborate /
  lower — a new bug-surfacing surface.
- The lane's pattern holds: an emit-projection of an existing valid construct,
  rules-first, default-off / byte-identical, proven downstream-clean before it
  ships, reusing existing render machinery. Wider groups, nested / multi-level
  `generate`, and `interface` / `modport` each land later as their own decided
  leaves, none retired.

## Open questions (to be resolved at `.12a`)

- The exact mutual-fan-in-independence mechanism: a bounded backward DFS over the
  larger member's operand cone (leveraging the topological `NodeId` invariant) vs
  reusing the `src/introspect/analyze.rs` support-cone builder. The bounded DFS is
  the leading first cut (no cross-module dependency, naturally bounded).
- The exact `Module` carrier: a `multi_output_task_groups: BTreeMap<NodeId,
  Vec<NodeId>>` keyed by the group leader (lowest `NodeId`) → the partner members
  (the `cone_function_gates` precedent), iterated in leader-`NodeId` order for
  determinism.
- Whether each multi-output task call gets its own `always_comb` (likely — one per
  group, mirroring the single-gate task) or shares one block.
- The exact knob name + semantics (`multi_output_task_emit_prob` proposed) and the
  downstream-gate scenario shape (`saw_multi_output_task_emit`, `__mt(` detection).

## Tree split

`STRUCTURED-EMISSION-EXPANSION` continues (the lane stays `active`):

- **`.11`** (this leaf, design) — decision `0025`: the sixth surface, its
  valid-by-construction discipline, the soundness rule, the grouping policy, its
  opt-in knob, its downstream gate, and the rejected alternatives. Docs-only.
- **`.12`** (impl, `pending`) — the multi-output combinational `task automatic`
  surface: the `multi_output_task_emit_prob` knob + the construction/projection +
  the emitter rendering + the downstream-clean gate + book/USER_GUIDE/KM.
  Default-off / DUT byte-identical. Pre-split into `.12a` (design-detail, grounded
  in the real `task_emit.rs` / `cone_function_emit.rs` / `to_sv_with_modules` +
  the gate source) + `.12b` (impl, itself pre-split `.12b.1` live / `.12b.2`
  metric+gate / `.12b.3` docs) when picked.
- **future (`.13`+)** — wider (`k > 2`) co-supported task groups, nested /
  multi-level `generate`, `interface` / `modport`, each a new vetted surface with
  its own decision when picked.

## Links

- Owner doctrine: `feedback_pick_and_roll_at_no_frontier` (autonomous surface
  selection at the no-frontier boundary), `feedback_dont_ask_just_do`.
- Lane / ROADMAP: steering gap 1 (richer structured emission), the structure-first
  ceiling (steering gap 4 — this adds shape, not behaviour).
- Doctrine: `feedback_rules_first_generation` (no generate-then-filter),
  `feedback_never_retire_strategies` (opt-in, default byte-identical),
  `feedback_full_factorization` (reuse the cone-function render primitives + the
  single-gate task predicate; one mechanism, not two).
- Precedents: decision `0014` (the single-gate combinational `task automatic` this
  generalizes) + decision `0016` (the multi-gate-cone function — the deduplicated
  shared-formal render primitives reused, and the separate-knob discipline) +
  decisions `0012` / `0013` / `0015` (the emit-projection family) + the
  `soft_union` overlay (`0010`).
- Reuse / touch points: `src/emit/sv.rs` (`to_sv_with_modules` + the projection
  sections; `cone_function_params` / `cone_operand_ref` / `render_cone_gate_expr`
  reused for the deduplicated body), `src/config.rs` (the
  `multi_output_task_emit_prob` knob + the `--multi-output-task-emit-prob` flag
  beside `task_emit_prob` / `cone_function_emit_prob`), `src/ir/` (a
  `multi_output_task_emit` gen-time-annotation pass beside `task_emit.rs` /
  `cone_function_emit.rs`), `src/ir/cone_function_emit.rs` (extend `sibling_marked`
  to exclude multi-output members), `src/metrics.rs`
  (`num_emitted_multi_output_tasks`, introspection schema `1.13 → 1.14`),
  `src/bin/tool_matrix.rs` (the `--multi-output-task-gate` downstream gate),
  `book/src/structured-emission.md` (user-facing — extend the chapter).
