---
id: mealy-fsm-outputs
title: ANVIL's Mealy FSM output is a default-off combinational decode over (state_q, sel) extending the Phase-6 Moore-only `Fsm`
answers:
  - "does ANVIL emit a Mealy FSM"
  - "can ANVIL generate Mealy state-machine outputs"
  - "is the ANVIL FSM output Moore or Mealy"
  - "does an ANVIL FSM output depend on the current input"
  - "how does ANVIL extend the Phase-6 FSM motif with input-dependent outputs"
  - "what knob enables Mealy FSM outputs"
  - "does the ANVIL FSM output decode read the transition-select cone"
  - "why was Mealy FSM picked before the next SV up-opt"
  - "are enum/typedef or packed multidimensional arrays version-distinctive SV up-opts"
  - "is a Mealy output a behaviour-preserving emit-projection like the structured surfaces"
date: 2026-06-22
status: accepted
tags: [capability, fsm, mealy, moore, sequential, emission, downstream, valid-by-construction, rules-first, breadth, north-star]
evidence: docs/decisions/0024-mealy-fsm-outputs.md; docs/tasks/CAPABILITY-BREADTH-EXPANSION.md; src/ir/types.rs (Fsm, FsmEncoding, Node::FsmOut); src/emit/sv.rs (FSM next-state + Moore output decode); src/gen/module.rs (build_fsm_block, fsm_prob)
reverify: 'printf ''module mealy_clean(input logic clk,input logic rst_n,input logic [1:0] sel,output logic [3:0] y);localparam logic[1:0] S0=2'"'"'d0,S1=2'"'"'d1,S2=2'"'"'d2,S3=2'"'"'d3;logic[1:0] state_q,next_state;always_comb unique case(state_q) S0:next_state=sel[0]?S1:S2;S1:next_state=sel[1]?S2:S3;S2:next_state=sel[0]?S3:S0;default:next_state=S0;endcase always_ff @(posedge clk or negedge rst_n) if(!rst_n) state_q<=S0; else state_q<=next_state;always_comb unique case(state_q) S0:y=(sel==2'"'"'d0)?4'"'"'h1:(sel==2'"'"'d1)?4'"'"'h2:4'"'"'h4;S1:y=(sel==2'"'"'d0)?4'"'"'h3:4'"'"'h5;S2:y=(sel==2'"'"'d0)?4'"'"'hA:4'"'"'hB;default:y=4'"'"'hE;endcase endmodule\n'' > /tmp/mealy_clean.sv && for L in 1800-2012 1800-2017 1800-2023; do verilator --lint-only -Wall --language $L /tmp/mealy_clean.sv && echo "verilator $L CLEAN"; done && yosys -q -p "read_verilog -sv /tmp/mealy_clean.sv; synth -noabc; check" && iverilog -g2012 -o /dev/null /tmp/mealy_clean.sv && echo ALL-TOOL-CLEAN'
---

# 0024 - CAPABILITY-BREADTH-EXPANSION: Mealy FSM outputs as a default-off combinational decode over `(state_q, sel)`

- Date: 2026-06-22
- Status: accepted
- Tree: `CAPABILITY-BREADTH-EXPANSION.2a` (design leaf; splits `.2` into `.2a` design + `.2b` impl)
- Extends: the Phase-6 generated-encoding FSM motif (`PHASE-6-ADVANCED-MOTIFS.3`, Moore-only)

## Context

`CAPABILITY-BREADTH-EXPANSION` (owner-directed, `2026-06-17`) has two design-first
breadth strands: `.1` *more SV-2017/2023 up-opts* (continuing the `union soft`
precedent, decision [`0010`](0010-sv-version-first-upopt-soft-packed-union.md))
and `.2` *Mealy FSM outputs* (extending the Phase-6 `Fsm`). Each new construct
must be default-off, valid-by-construction, proven downstream-clean,
API-selectable, and introspectable (decision
[`0017`](0017-api-first-everything-mcp-accessible.md)). This record is the `.2`
design leaf (`.2a`); it pins the Mealy output model, the knob, the metric/gate,
and the MCP surface, grounded in the real Phase-6 FSM surface and a fresh
empirical probe of the installed tools.

### Why `.2` (Mealy) is advanced ahead of the frontier-ordered `.1` (SV up-opt)

The frontier table ordered `.1` first ("highest value-per-effort, continues the
closed `SV-VERSION-TARGETING` pattern"). A **fresh empirical probe** — exactly
the probe `.1` itself mandates — was run against the installed toolchain
(Verilator 5.046, Yosys 0.64, Icarus 13.0) for the two named `.1` candidates:

| Candidate | Verilator `--language 1800-2012 / 2017 / 2023` | Yosys `read_verilog -sv` | Icarus `-g2012` |
|---|---|---|---|
| `enum` / `typedef` | ACCEPT / ACCEPT / ACCEPT | ACCEPT | ACCEPT |
| packed multidimensional array (`logic [1:0][3:0]`) | ACCEPT / ACCEPT / ACCEPT | ACCEPT | ACCEPT |

Both are accepted at **every** version mode and by every tool ⇒ **not
version-distinctive**, with **no down-gating teeth** — they are legal at the
1800-2012 floor. This re-confirms decision `0010`'s load-bearing finding that the
installed tools do not enforce IEEE-1800 version rejection and that the genuinely
2023-distinctive *synthesizable* space (where the pre-2023 form is illegal,
forcing the newer construct) was, for these tools, essentially **only**
`union soft` (already shipped). `0010` already recorded that 1800-2017 is a
maintenance revision with *no* synthesizable construct distinctive over 2012.

Consequence: `.1`'s remaining "**next** version-distinctive up-opt" has thin,
uncertain yield with the installed tools (its most defensible outcome would be
broadening `union soft` to more IR shapes — same construct, not a *new* one).
`.2` (Mealy), by contrast, is a genuinely-new synthesizable surface, accepted
**warning-clean by all three tools** (a *stronger* downstream story than the
Verilator-only `union soft`), and a small, well-bounded extension of an existing,
proven motif. It is therefore the higher value-per-effort pick *with confidence*.
`.1` stays `pending` (nothing retired); the probe evidence above is recorded so a
future `.1` either finds a genuinely-2023 construct or deliberately rescopes to
`union soft` breadth. Self-selecting `.2` here follows
`feedback_pick_and_roll_at_no_frontier` (both leaves are eligible pending design
ADRs) and `feedback_never_retire_strategies`.

## The Phase-6 FSM surface today (Moore-only)

`src/ir/types.rs` `Fsm` is a first-class generated-encoding state machine:

- `encoding: FsmEncoding` (`Binary` / `OneHot` / `Gray`) fixes the `state_q`
  width and the `localparam` state constants.
- `sel: NodeId` (`sel_width` bits) is a **real generated cone** — the
  transition-select source, dependency-tracked + validated. It already depends on
  the module's inputs / state.
- `transitions[state][sel_value] -> next_state_index` (shape
  `[num_states][1 << sel_width]`) is the next-state table.
- `outputs[state] -> u128` (length `num_states`, masked to `out_width`) is the
  **Moore** output table: one constant per state.
- `Node::FsmOut { fsm, width }` is the **opaque leaf** exposing the output into
  the gate graph (identity-by-instance, never CSE-merged, the clock edge breaks
  the combinational path — exactly like `FlopQ` / `MemRead`).

The emitter (`src/emit/sv.rs`) renders, per FSM:
1. a **next-state** `always_comb` as a nested `case (state_q)` → `case (sel)` →
   `next_state = <const>` (lines ~824-846), and
2. a **Moore output** `always_comb` as a flat `case (state_q)` →
   `fsm_out = <out_width>'h<outputs[state]>` (lines ~859-875).

So the next-state decode is **already** a `(state_q, sel)`-indexed nested case;
the Moore output decode is only `state_q`-indexed.

## Decision

**A Mealy FSM output is a default-off, valid-by-construction combinational decode
of `(state_q, sel)`** — i.e. the registered current state **and** the existing
input-dependent transition-select cone `sel` — exactly the textbook Mealy form
(*output = f(current state, current input)*). It is the minimal extension of the
Moore output table from a per-state constant to a per-`(state, sel_value)`
constant, mirroring the `transitions` table 1:1.

### The model (pinned)

- **Output table.** A Mealy FSM's output table is
  `mealy_outputs[state][sel_value] -> u128` (shape `[num_states][1 << sel_width]`,
  masked to `out_width`) — the structural twin of `transitions`. Each entry is a
  generated constant (the existing `outputs[state]` generation discipline, lifted
  to two dimensions). Behaviour is defined **by construction** from this table; it
  is not a derived/over-specified function.
- **Emitter delta (one block).** The Moore output `always_comb` (flat
  `case (state_q)` → const) becomes, for a Mealy FSM, a nested
  `case (state_q)` → `case (sel)` → `fsm_out = <out_width>'h<mealy_outputs[s][sv]>`
  with a `default` arm — **structurally identical to the already-proven
  next-state decode**, just driving the `FsmOut` signal. No other emitter change.
- **`Node::FsmOut` stays opaque.** Identity-by-instance, never CSE-merged, exactly
  as today. Only its *decode* gains the `sel` read.
- **Whole-FSM mode (first cut).** The single output of an `Fsm` is either Moore or
  Mealy, carried by a per-FSM discriminator (working name `FsmOutputKind { Moore,
  Mealy }`, default `Moore`). Per-output Moore/Mealy choice is **moot** until
  multi-output FSMs exist (a separate future surface); pinning whole-FSM here
  retires nothing.
- **Combinational input→output path (the defining Mealy property).** Under Mealy,
  `FsmOut` has a real combinational path from the inputs through `sel`. The
  `FsmOut` leaf's **virtual dependency set** (`DepSet::from_fsm_virtual` today)
  must therefore also cover `sel`'s support so non-triviality + IR validation stay
  correct. This is the one substantive impl subtlety; resolved at `.2b`.
- **Single-clock discipline preserved.** The state register stays Moore-clocked by
  the module's shared `clk` / async-low `rst_n`. Mealy changes only the
  *combinational output decode*, never the clocking.

### Knob, default, steering

- A new default-off knob **`fsm_mealy_prob`** (`f64`, default `0.0`): the
  per-generated-FSM probability that its output is Mealy rather than Moore. It is
  rolled **inside** the existing `fsm_prob` construction path (only FSMs that are
  actually built can become Mealy), at a single seeded `gen_bool` site, so
  `fsm_mealy_prob == 0.0` draws nothing ⇒ **byte-identical** stream + output (the
  `soft_union_slice_prob` / `aggregate_prob` precedent; `tests/snapshots.rs`
  untouched). Steering category `fsm` (joins `fsm_prob` in `config_category`).

### Metric + schema

- A new metric **`num_mealy_fsm_modules`** (count of modules carrying ≥1 Mealy
  FSM), surfaced in `--introspect`. Introspection schema MINOR-bumps **`1.12 →
  1.13`** (additive-only, per the established policy). Default-off ⇒ the field is
  `0` and the existing payload is otherwise unchanged.

### Downstream proof (gate)

- A repo-owned `tool_matrix` coverage fact **`saw_mealy_fsm_design`** + a focused
  scenario forcing `fsm_mealy_prob = 1.0` (with `fsm_prob` high) across the three
  construction strategies, proven **downstream-clean across Verilator `-Wall` +
  both repo Yosys modes + Icarus** (the FSM-motif-gate precedent;
  `--mealy-fsm-gate` or folded into the existing FSM/signoff gate — pinned at
  `.2b`). Unlike the `union soft` up-opt (Verilator-only, Yosys/Icarus no-op), a
  Mealy output is **universally synthesizable**, so the gate runs the full
  multi-tool plan.

### MCP selectability + queryability (decision `0017`)

- `fsm_mealy_prob` is settable via `--config` JSON and the MCP `generate` settings
  (generic f64-knob plumbing) and gets a kebab-case CLI flag `--fsm-mealy-prob`
  (the `KNOB-ERGONOMICS-AND-PRESETS` pattern); it auto-appears in the
  SCHEMA-DERIVED knob catalog. The `num_mealy_fsm_modules` metric is queryable via
  `--introspect` and the MCP `introspect` tool. CLI is a shim over the same
  surface.

## Empirical tool-reality finding (load-bearing)

A filename-matched Mealy reference (`mealy_clean.sv`: a 4-state encoded FSM whose
output is a per-`(state, sel)` `case (state_q)` → `case (sel)` decode — the exact
shape this decision emits) was probed against the installed tools:

- **Verilator 5.046 `--lint-only -Wall`** at `--language 1800-2012`, `1800-2017`,
  **and** `1800-2023`: **ACCEPT, warning-clean (exit 0)** in all three modes.
- **Yosys 0.64**: `synth -noabc` and the repo ABC path (`synth -noabc; abc -fast;
  opt -fast; check`) both **ACCEPT**.
- **Icarus 13.0 `-g2012`**: **ACCEPT**.

A Mealy output is therefore inside the synthesizable subset and accepted by every
installed tool — *not* version-gated (no `sv_version` interaction; unlike the
up-opts, no down-gate fallback is needed). LRM grounding: the construct is an
`always_comb` `case` decode over a registered state variable and a combinational
input cone — ordinary synthesizable procedural logic (IEEE 1800-2023 §9 processes,
§12 `case`); nothing in the model is newer than the 2012 floor, so it composes
with every `--sv-version` target unchanged.

## Rejected alternatives (with reasoning)

- **A free combinational function of arbitrary inputs (not `sel`) for the output.**
  Rejected: it would add a *second* input-cone notion beside `sel`, complicate the
  `FsmOut` virtual-deps story, and break the clean 1:1 parallel with
  `transitions`. Reusing `sel` keeps the model minimal, already-validated, and
  by-construction (full-factorization / `feedback_full_factorization`: one cone
  notion, not two).
- **A new IR node / making `FsmOut` an expression.** Rejected: `FsmOut` stays the
  opaque clock-edge-breaking leaf (the `FlopQ` / `MemRead` precedent); only its
  decode changes. No new computed truth — the emit-projection discipline of the
  structured-emission surfaces (decisions `0012`–`0016`).
- **Reusing `fsm_prob` for Mealy.** Rejected: it would change default emission
  when `fsm_prob > 0`. Mealy gets its **own** default-off knob so the shipped
  Moore path stays byte-identical (`feedback_never_retire_strategies`).
- **Per-output Mealy/Moore mode now.** Rejected as premature: an `Fsm` has exactly
  one output today; per-output choice is meaningless until multi-output FSMs land
  (a future surface). Whole-FSM mode first; nothing retired.
- **Doing `.1` (next SV up-opt) first as the frontier ordered.** Deferred (not
  rejected): the fresh probe above shows its new-construct yield is thin/uncertain
  with the installed tools, while Mealy is high-certainty all-tool-clean breadth.
  `.1` remains `pending` with the probe evidence recorded.
- **Generate-then-filter / a non-byte-identical default.** Rejected by doctrine
  (`feedback_rules_first_generation`, core principle 2).

## Tree split

`CAPABILITY-BREADTH-EXPANSION.2` becomes a container:

- **`.2a`** (this leaf, design) — names the Mealy output model
  (`mealy_outputs[state][sel_value]` decode over `(state_q, sel)`), the empirical
  all-tool-clean finding, the `fsm_mealy_prob` knob + default-off discipline, the
  `num_mealy_fsm_modules` metric + schema `1.13`, the `saw_mealy_fsm_design` gate,
  and the MCP surface. Docs-only.
- **`.2b`** (impl, `proposed`) — implement the Mealy output table + the emitter
  nested-case decode gated on the per-FSM Mealy discriminator, the
  `fsm_mealy_prob` roll inside `build_fsm_block`, the `FsmOut` virtual-deps fix,
  the metric (schema `1.13`), the `tool_matrix` gate + fact, and the
  book/USER_GUIDE/README/KM docs; default-off / byte-identical, snapshots
  untouched. Pre-split into `.2b.1` (design-detail) + `.2b.2` (impl) + `.2b.3`
  (docs) when picked, per the `.2b` precedent, if it proves broad.

## Open questions (resolved at `.2b` / `.2b.1`)

- **Exact IR field layout.** A new `Fsm.output_kind: FsmOutputKind` + a 2-D
  `mealy_outputs` (an `Option`, or `outputs` reshaped) vs a `mealy: bool` flag —
  the lowest-blast-radius shape that keeps the Moore path byte-identical.
- **`FsmOut` virtual-deps construction under Mealy** — folding `sel`'s support
  into `DepSet::from_fsm_virtual` (and the parallel sites in
  `metrics.rs` / `compact.rs`) so non-triviality + validation + dedup stay sound.
- **Mealy + FSM identity/dedup interaction.** Two Mealy FSMs are equivalent only
  when their full `mealy_outputs` tables (and `transitions` / `sel` / encoding)
  match; confirm the existing FSM-merge keying covers the 2-D table.
- **Gate wiring** — a dedicated `--mealy-fsm-gate` scenario set vs folding the
  `saw_mealy_fsm_design` fact into the existing FSM/signoff gate.

## Consequences

- ANVIL gains a genuinely-new synthesizable motif breadth surface — Mealy state
  machines with input-dependent outputs — that **all three installed downstream
  tools accept warning-clean**, a real motif stress for the north star (expose
  downstream-tool bugs on legal, unusual RTL).
- The default and every existing gate stay **byte-identical** (`fsm_mealy_prob`
  default `0.0` + the Moore path untouched); orthogonal to `--identity-mode`,
  factorization, `--sv-version`, and every other knob.
- The adversarial matrix gains a dedicated Mealy axis (ROADMAP steering gap 1 —
  feature breadth; gap 3 — explicit adversarial axis).

## Links

- Task-tree: `CAPABILITY-BREADTH-EXPANSION.2a` (this leaf); frontier advances to
  `.2b`. Sibling `.1` (SV up-opt design ADR) stays `pending`.
- Parent lane: `CAPABILITY-BREADTH-EXPANSION` (owner-directed `2026-06-17`).
- Reuses / extends: the Phase-6 FSM motif (`Fsm`, `FsmEncoding`, `Node::FsmOut`,
  `build_fsm_block`, the emitter FSM block); the `union soft` empirical precedent
  (decision `0010`); the structured-emission emit-projection discipline (decisions
  `0012`–`0016`); the API-first mandate (decision `0017`).
- North star: `project_anvil_north_star`; ROADMAP steering gaps 1 (breadth) + 3
  (explicit adversarial axis matrix).
- Doctrine: `feedback_rules_first_generation`, `feedback_never_retire_strategies`,
  `feedback_full_factorization`, `feedback_pick_and_roll_at_no_frontier`,
  `feedback_book_doctrine` (the construct is user-facing → book chapter at `.2b`).
- Touch points (for `.2b`): `src/ir/types.rs` (`Fsm`, a Mealy discriminator +
  2-D output table), `src/emit/sv.rs` (the FSM output-decode block), `src/gen/module.rs`
  (`build_fsm_block`, the `fsm_mealy_prob` roll), `src/config.rs` (the knob +
  `config_category` + the `--fsm-mealy-prob` flag), `src/metrics.rs`
  (`num_mealy_fsm_modules`), `src/introspect/mod.rs` (schema `1.13`),
  `src/bin/tool_matrix.rs` (the Mealy scenario + `saw_mealy_fsm_design`),
  `src/ir/compact.rs` (`FsmOut` deps / FSM dedup), `book/src/sequential.md`,
  `book/src/knobs.md`.
