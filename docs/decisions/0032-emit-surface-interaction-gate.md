---
id: emit-surface-interaction-gate
title: The nine emit-projections are gated **in combination** by an intermediate-probability interaction sweep; `structured-emission-max` means maximal surface *diversity*, not every knob at `1.0`
answers:
  - "are ANVIL's nine structured-emission surfaces proven downstream-clean together"
  - "what does the emit-surface interaction gate prove"
  - "why does setting every emit_prob to 1.0 emit only one surface"
  - "why does --profile structured-emission-max emit only combinational functions"
  - "how many structured-emission surfaces can one ANVIL module carry at once"
  - "how is emit-surface co-occurrence proven without a new identifier token"
  - "is the emit-projection mutual-exclusion invariant tested end to end"
  - "does cone_function absorption interact safely with the other emit surfaces"
  - "why is soft_union a separate scenario in the emit-interaction gate"
  - "what probability should structured-emission-max set"
  - "does anvil count memory and fsm ports when deciding cone absorption"
date: 2026-07-30
status: accepted
tags: [quality, structured-emission, interaction, mutual-exclusion, tool-matrix, gate, coverage, preset, profile, cone-function, absorption, downstream, rules-first, north-star]
evidence: src/gen/mod.rs:93-190 (the fixed nine-pass projection chain, both call sites); src/ir/{soft_union,function_emit,generate_loop,task_emit,multi_output_task_emit,cone_function_emit,mux_if_emit,case_mux_if_emit,casez_mux_if_emit}.rs (the nine exclusion predicates audited pass-by-pass in Context §2); src/ir/cone_function_emit.rs:103-157 (`compute_use_counts` + `should_absorb` — the absorption soundness rule and its uncounted-consumer gap); src/gen/module.rs:125,248 (`build_memory_leaf` / `build_fsm_block` build gate-free modules — why that gap is unreachable today); src/bin/tool_matrix.rs:364-471 (the nine per-module `emitted_*` booleans the co-occurrence fact projects); measured probes re-runnable from the commands in Context §3-§5
---

# 0032 - EMIT-SURFACE-INTERACTION-GATE: prove the nine emit-projections in combination, at the probability where they actually coexist

- Date: 2026-07-30
- Status: accepted
- Tree: `EMIT-SURFACE-INTERACTION-GATE.1` (design leaf; decides the gate shape, the
  co-occurrence proof, `soft_union`'s placement, and the named interaction risks)
- Activated by: autonomous PNT selection (`2026-07-30`) under the owner's standing
  **DECIDE, DON'T ASK** directive, from a source-level observation that the nine
  emit-projections' mutual-exclusion invariant has no end-to-end downstream gate.

## Context

`STRUCTURED-EMISSION-EXPANSION` has shipped **nine** richer-structured emit surfaces
(decisions `0010`, `0012`–`0016`, `0025`, `0027`–`0029`). Each is individually banked
downstream-clean. None has ever run **beside another** in an end-to-end downstream gate.

### 1. The invariant that holds them together

The nine passes run in one fixed order at both generator call sites
(`src/gen/mod.rs:93-190`):

```
soft_union → function_emit → generate_loop → task_emit → multi_output_task
           → cone_function → mux_if → case_mux_if → casez_mux_if
```

Each pass excludes any gate an earlier pass already marked. **That exclusion is the
entire soundness argument** for stacking nine overlapping projections onto one gate
graph: without it, two passes could both claim a gate and the emitter would render it
twice.

Every one of the eight surface focus configs in `tool_matrix` sets exactly **one**
`*_emit_prob` to `1.0` and leaves the rest at `0.0`. Under every gate this repo owns,
each exclusion predicate therefore evaluates against an **empty** sibling set and
returns `false` unconditionally. The property is written, unit-tested at the function
level, and **never exercised end-to-end**.

### 2. The exclusion matrix is complete (audited, `2026-07-30`)

Read pass-by-pass from the nine `annotate_*` sources. `✓` = excluded; `—` = not
applicable (the pass runs earlier).

| pass (in run order) | excludes soft_union | function | generate_loop | task | multi_output | cone (root ∪ interior) | mux_if | case_mux_if |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `soft_union` | — | — | — | — | — | — | — | — |
| `function_emit` | ✓ | — | — | — | — | — | — | — |
| `generate_loop` | ✓ | ✓ | — | — | — | — | — | — |
| `task_emit` | ✓ | ✓ | ✓ | — | — | — | — | — |
| `multi_output_task` | ✓ | ✓ | ✓ | ✓ | — | — | — | — |
| `cone_function` | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | — |
| `mux_if` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | — | — |
| `case_mux_if` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | — |
| `casez_mux_if` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |

Every pass excludes **exactly** its predecessors. The matrix is lower-triangular and
complete — no hole. This audit is a precondition of the gate, not a substitute for it:
it proves the *predicates* are written correctly; only a downstream run proves the
*emitted SV* is clean when several fire in one module.

### 3. Measured: "all knobs on" is not "all surfaces emitted"

The tree opened on the observation that `--profile structured-emission-max` covers 4 of
9 surfaces. The measurement is worse than that, and it is the pivotal fact of this
decision.

```
$ anvil --seed 1 --profile structured-emission-max --introspect | \
    <read introspection.module_metrics.num_emitted_*>
num_emitted_combinational_functions 796
num_emitted_generate_loops            0
num_emitted_combinational_tasks       0
num_emitted_cone_functions            0
  (+ the four surfaces the preset does not set: all 0)
```

The preset sets four surfaces to `1.0` and emits **one**. Reproduced on seeds 1/2/3
(796 / 1057 / 917 functions; every other surface exactly `0`).

The mechanism is the interaction of mutual exclusion with the fixed order. At
`prob = 1.0`, `function_emit` runs second and claims **every** admissible gate — which
is a superset of `generate_loop`'s `{N{x}}` `Concat`s, `task_emit`'s candidates,
`multi_output_task`'s members, `cone_function`'s roots and interiors, and `mux_if`'s 2:1
`Mux`es. Each later pass then correctly finds nothing left. **The preset is not merely
incomplete; three of its four surfaces are inert.**

Setting *all eight* to `1.0` does not fix it — measured, 12/12 modules, exactly **3**
live surfaces (`function_emit` + `case_mux_if` + `casez_mux_if`, the last two surviving
only because `CaseMux`/`CasezMux` are outside `function_emit`'s admissible set):

```
$ anvil --seed 7 --count 12 --out <dir> \
    --function-emit-prob 1.0 --generate-loop-emit-prob 1.0 --task-emit-prob 1.0 \
    --multi-output-task-emit-prob 1.0 --cone-function-emit-prob 1.0 \
    --mux-if-emit-prob 1.0 --case-mux-if-emit-prob 1.0 --casez-mux-if-emit-prob 1.0 \
    --flop-prob 0.0 --comb-mux-prob 0.35 --case-mux-prob 0.35 --casez-mux-prob 0.35 \
    --terminal-reuse-prob 0.6 --constant-prob 0.05 --max-depth 6 \
    --min-inputs 6 --max-inputs 8 --min-outputs 3 --max-outputs 4
⇒ distinct live surfaces per module: [3,3,3,3,3,3,3,3,3,3,3,3]
```

Saturation maximises *coverage* (nearly every gate carries some projection) and
minimises *diversity* (three shapes). Diversity is what ANVIL is for: a module carrying
a `function automatic`, a `generate for`, a multi-output `task`, a cone function, a
procedural `if`/`else` and two priority chains **at once** is far better bug-bait than
eight modules each carrying one (ROADMAP steering gap 1).

### 4. Measured: an intermediate probability makes all eight coexist, cleanly

Same shape, all eight at `0.25`, seed 7, 24 modules:

```
distinct live surfaces per module: 8 in 20 modules, 7 in 4 modules
totals — function 1530 · generate_loop 163 · task 1158 · multi_output_task 215 ·
         cone_function 131 · mux_if 138 · case_mux_if 203 · casez_mux_if 97
```

and downstream-**clean on every column the sibling gates use**, zero warnings:

| tool column | invocation | result |
| --- | --- | --- |
| Verilator | `verilator --lint-only` | 24/24, 0 warnings |
| Yosys without-abc | `read_verilog -sv; synth -noabc; stat; check` | 24/24, 0 warnings |
| Yosys with-abc | `… synth -noabc; abc -fast; opt -fast; stat; check` | 24/24, 0 warnings |
| Icarus | `iverilog -g2012` | 24/24 |

The floor holds across all three construction strategies (12 modules each, seed 7):
`sequential` min 6 / max 8, `shuffled` 8 / 8, `interleaved` 7 / 8. **All eight surfaces
in a single module is reachable on every strategy.**

> Probe hygiene: an earlier pass of this probe used `verilator --lint-only -Wall` and
> failed 24/24 on `UNUSEDSIGNAL`. A negative control with **all eight surfaces off**
> failed 23/24 on the same shape — the warnings were an artifact of the ad-hoc shape
> config, not of surface stacking. `-Wall` is not this repo's acceptance bar;
> `src/downstream/mod.rs:154` runs bare `--lint-only`. Recorded because the wrong bar
> nearly produced a false "the surfaces conflict" finding.

### 5. Calibrating the probability (max-min over the eight surfaces)

Raising `p` monotonically starves the *later* passes (`multi_output_task`,
`cone_function`) as the earlier ones claim more gates. The right criterion is not "how
many surfaces fire" (all eight fire for every `p ∈ [0.15, 0.5]` on the default shape)
but **how well represented the least-represented surface is**. Minimum per-surface count
across the eight, default shape, seeds 1–5:

| `p` | per-seed min | mean min |
| --- | --- | --- |
| `0.15` | 39, 49, 59, 62, 48 | 51.4 |
| **`0.25`** | **48, 76, 98, 86, 61** | **73.8** |
| `0.35` | 47, 68, 89, 84, 63 | 70.2 |
| `0.50` | 26, 38, 52, 33, 36 | 37.0 |

`0.25` maximises the floor. That is the value this decision adopts for both the gate and
the preset.

### 6. The one pass that does more than skip: `cone_function` absorption

Eight passes *mark* a gate. `cone_function` also **absorbs**: an interior gate folded
into a cone loses its module `wire` **and** its inline `assign`, living only inside the
emitted function (`src/ir/cone_function_emit.rs`, and the emitter suppression proven by
`marked_cone_emits_multi_statement_function_unmarked_is_inline`). It is the only pass
that can make another gate's name **disappear**. Reasoned through from source:

- **Absorption requires `use_count == 1`** (`should_absorb`), and the walk only reaches
  a node through the cone edge — so the single consumer *is* that edge. A gate consumed
  by anything else has count ≥ 2 and stays a boundary parameter with its own wire.
- **A cone root keeps its wire** (`assign <root> = <root>__cf(…);`), so a sibling
  surface referencing the root as an operand is unaffected.
- **A sibling-marked gate is neither root nor interior** (`sibling_marked` covers all
  five earlier passes), and the three later passes exclude cone roots **∪** interiors.
  So no marked gate is ever absorbed, and no absorbed gate is ever marked.
- **`CaseMux` / `CasezMux` / `ForFold` / `Slice` are not `admissible`**, so the eighth
  and ninth surfaces' targets can never be absorbed — their arm operands are reachable
  only through a non-admissible parent, which can never be a cone root.

Absorption is therefore sound against all eight siblings. **But the soundness rests
entirely on `compute_use_counts` counting every consumer, and it does not.** It counts
gate operands, output drives, flop `D`/mux refs, and instance inputs — it does **not**
count `Memory.we/waddr/wdata/raddr` or `Fsm.sel`, which the emitter renders by wire name
(`src/emit/sv.rs:928-931`). A gate consumed once by a cone edge and once by a memory
port would be seen as single-use, absorbed, and its wire deleted out from under the
memory block.

That is unreachable **today**, and only by accident of module shape: `build_memory_leaf`
(`src/gen/module.rs:125`) and `build_fsm_block` (`:248`) construct **gate-free** modules
whose memory/FSM ports are `PrimaryInput` nodes, and those are the only two sites in the
generator that push a `Memory` or an `Fsm`. Empirically confirmed: a
`--memory-prob 1.0 --cone-function-emit-prob 1.0` corpus (40 modules) produces no
undeclared-identifier failure, because no module ever holds both a memory and a gate.

This is a **latent trap, not a live bug** — and it is exactly the kind that detonates
later, silently, when someone makes memories or FSMs coexist with combinational logic in
one module. It gets a hardening leaf.

### 7. `soft_union` is disjoint by target type, and version-gated

`soft_union` targets `Slice` gates. `Slice` is excluded from `function_emit` /
`task_emit` / `cone_function` admissibility; `generate_loop` targets `Concat`; the three
procedural surfaces target `Mux` / `CaseMux` / `CasezMux`. So `soft_union` adds a ninth
*emitted* surface but **zero** new exclusion pressure — nothing else can ever claim its
gates. It is also the one surface Yosys and Icarus **reject** (decision `0010`), costing
two of the four tool columns.

Measured: nine surfaces stacked under `--sv-version 2023 --soft-union-slice-prob 1.0`
plus the eight at `0.25` is Verilator-clean under `--language 1800-2023` (8/8 modules,
all 8 genuinely emitting `union soft`; 6 of 8 also carrying all eight other surfaces).

## Decision

### (a) The gate is a **small sweep**, not one scenario — 3 + 1 scenarios

`tool_matrix --emit-surface-interaction-gate` runs:

1. **Three universal scenarios** — one per construction strategy
   (`Sequential` / `Shuffled` / `Interleaved`, matching every sibling gate's shape),
   each a comb-only single-module DUT with **all eight** non-version-gated
   `*_emit_prob` set to **`0.25`** over a selector-rich shape (all three of
   `comb_mux_prob` / `case_mux_prob` / `casez_mux_prob` live, so the three procedural
   surfaces all have candidates). Full tool plan: Verilator + both Yosys modes
   (+ Icarus under `--iverilog-compile`).
2. **One saturation scenario** — all eight at `1.0`, `Interleaved`. This is the
   *exclusion-pressure* scenario: it is the configuration where every later pass sees a
   **full** sibling set and must skip, the exact inverse of today's always-empty case.
   Measured clean on all four columns. It is cheap and it tests the opposite extreme of
   the same invariant, so it earns its place.

Three strategies (not one) because the surfaces' candidate sets depend on graph shape,
and the measurement shows the per-module surface floor varies by strategy (6 / 8 / 7).
One scenario would under-sample the very interaction the gate exists to prove.

`soft_union` joins as a **separate tenth-style scenario** (below), never folded into the
universal three.

### (b) Co-occurrence is proven **metric-keyed**, by projection — no new token

`ModuleReport` already carries all nine per-surface booleans
(`emitted_soft_union_overlay`, `emitted_combinational_function`, `emitted_generate_loop`,
`emitted_combinational_task`, `emitted_cone_function`, `emitted_multi_output_task`,
`emitted_mux_if`, `emitted_case_mux_if`, `emitted_casez_mux_if` —
`src/bin/tool_matrix.rs:364-471`). The gate adds one **derived** per-module scalar:

```
distinct_emit_surfaces = count of those booleans that are true
```

and two coverage facts, both requiring the module to be **accepted** by Verilator *and*
Yosys (the sibling-gate convention):

- `saw_multi_surface_emit_interaction` — some accepted module has
  `distinct_emit_surfaces >= 2`. This is the invariant under test, at its robust floor.
- `saw_all_emit_surfaces_in_one_module` — some accepted module has
  `distinct_emit_surfaces >= 8`. Measured reachable on all three strategies with margin
  (20/24 modules at the chosen `p`), so it is a real bar, not a brittle one.

This follows the decision-`0028` **metric-keyed** precedent exactly: no new identifier
token, no new IR truth, no text scan — a pure projection of facts the report already
computes. The report additionally records the per-scenario **maximum**
`distinct_emit_surfaces` so the achieved strength is visible without being gated on.

### (c) `soft_union` joins as its own Verilator-only scenario

One extra scenario: `--sv-version 2023`, `soft_union_slice_prob = 1.0`, plus the eight at
`0.25`, `Interleaved`, run **Verilator-only** under `--language 1800-2023` with
Yosys/Icarus recorded as a no-op — the exact `--sv-version-gate` up-opt precedent
(decision `0010`). It lights `saw_all_nine_emit_surfaces_in_one_module`.

It is kept out of the universal three because folding it in would cost those scenarios
their Yosys and Icarus columns — trading the gate's strongest evidence (four tools clean)
for one surface that adds no exclusion pressure at all (§7). Separate scenario, both
properties kept.

### (d) The named interaction risks, decided up front

The `.1` contract is that a gate failure must be a *prediction confirmed*, not a
surprise. The risks, ranked, with the call on each:

1. **`cone_function` absorption vs a later surface's operand reference** — analysed sound
   in §6 (absorption requires global single-use; every later pass excludes roots ∪
   interiors). **Predicted clean**, and the measured run agrees.
2. **`cone_function` absorption vs an uncounted consumer** (`Memory` ports / `Fsm.sel`) —
   a real gap in `compute_use_counts`, unreachable today only because memory and FSM
   leaves are gate-free modules. **Decision: harden it now** by counting
   `m.memories` and `m.fsms` consumers in `compute_use_counts`. The change is a
   provable no-op on every configuration the generator can currently produce (so DUT
   output stays byte-identical), and it removes a trap that a future memory/FSM-in-cone
   capability would otherwise spring silently. Owned by a new leaf `.4`.
3. **`multi_output_task` group members vs cone absorption** — a member is
   `sibling_marked`, so it is never a root and never an interior; a member's *operand*
   with a single use is reachable only through the member, which cannot be a root.
   **Predicted clean**; measured clean (215 multi-output tasks co-existing with 131 cone
   functions in the 24-module corpus).
4. **`mux_if`'s `<wire>__cv` output var vs cone absorption** — a `Mux` claimed by
   `mux_if` is excluded from cone absorption (the pass runs after `cone_function` and
   excludes roots ∪ interiors), and an absorbed `Mux` is never marked. Disjoint by
   construction. **Predicted clean**; measured clean.
5. **Name collisions across surfaces** — each surface mints a distinct suffix
   (`__f` / `__gi` / `__t` / `__mt` / `__cf` / `__cv`) off the gate's own wire name, and a
   gate carries at most one suffix. No collision is constructible. **Predicted clean**;
   measured clean.

### (e) `structured-emission-max` means maximal **diversity**, at `0.25`

The preset's current description — *"Turn on every richer-structured emit-projection …;
they are mutually exclusive per gate, so all-on is safe and behaviour-preserving"* — is
true and misleading in the same sentence. All-on *is* safe. It is also, measurably,
one-surface.

The preset is therefore redefined to set **all eight** non-version-gated `*_emit_prob` to
**`0.25`**, and to raise `comb_mux_prob` / `case_mux_prob` / `casez_mux_prob` so the three
procedural surfaces have candidates (the `deep-hierarchy` precedent, where a preset
legitimately sets structural knobs to make its own surfaces reachable). Its description
must state what "max" means: **the largest number of distinct structured surfaces present
in one module**, not the largest fraction of gates projected.

`soft_union_slice_prob` stays out — it is version-gated and owned by the `sv2023-upopts`
preset. A drift test pins preset ↔ knob-list agreement so a tenth surface cannot silently
omit itself (leaf `.2`).

## Decisive test applied

"Does the gate prove something no existing gate can, and would its failure be
diagnosable?" Yes on both. The eight sibling gates prove each surface in isolation and
*cannot* observe the mutual-exclusion invariant, because each leaves the other seven
knobs at `0.0`. This gate is the only place the invariant is observable end-to-end, at
both extremes (partial sibling sets at `0.25`, full sibling sets at `1.0`). And every
interaction pair is reasoned through from source in §6/(d) before the gate runs, so a
failure lands against a named prediction. The sibling gates are **not** retired: they
isolate a regression to one surface, which a combined gate cannot
(`feedback_never_retire_strategies`).

## Rejected alternatives

- **One scenario with all eight at `1.0`.** Rejected — measured: 3 live surfaces, 12/12
  modules. It proves saturation-exclusion and nothing about co-occurrence. Kept as *one*
  scenario for that narrow value, never as the gate.
- **One scenario, one strategy.** Rejected — the per-module surface floor varies by
  construction strategy (6 / 8 / 7 measured); one strategy under-samples the interaction.
- **Folding `soft_union` into the universal scenarios.** Rejected — it would cost every
  scenario its Yosys and Icarus columns (decision `0010`) in exchange for a surface that
  is disjoint by target type and adds no exclusion pressure.
- **A new identifier token to detect co-occurrence.** Rejected — all nine per-module
  `emitted_*` booleans already exist; co-occurrence is a projection of them. Minting a
  token would add emitted-output truth for a report's convenience (the decision-`0028`
  metric-keyed precedent).
- **Gating on `distinct_emit_surfaces >= 2` only.** Rejected as too weak given the
  measurement — `>= 8` is reachable on all three strategies with margin, so the gate
  asserts both, the weak floor for the invariant and the strong bar for the capability.
- **Gating on an exact count (`== 8` on every module).** Rejected as brittle: 4 of 24
  modules legitimately carry 7 (a shape may simply contain no `{N{x}}` replication).
  Per-scenario maximum is the right shape for the strong fact.
- **Setting the preset to all-eight-at-`1.0`.** Rejected — it is the exact configuration
  measured to emit 3 surfaces. A preset named `-max` that minimises diversity is the bug
  this decision exists to fix.
- **Leaving `compute_use_counts` alone because the gap is unreachable.** Rejected — it is
  unreachable by accident of module shape, not by any stated invariant, and the failure
  mode (a deleted wire under a memory port) is silent until a downstream tool rejects it.
  Hardening is a measured no-op today.
- **Retiring the eight single-surface gates.** Rejected
  (`feedback_never_retire_strategies`): they localise a regression to one surface.
- **Changing any surface's semantics or default.** Rejected: all nine stay default-off ⇒
  DUT byte-identical. This tree adds a gate and fixes a preset; it changes no surface.

## Consequences

- ANVIL gains its first gate over the **interaction** of emit surfaces rather than any
  single surface — and, with it, the first end-to-end exercise of the mutual-exclusion
  invariant that makes stacking nine projections sound.
- The DUT lane gains its densest legal artifact to date: a single module carrying up to
  eight (nine under 2023) distinct structured constructs, which is materially better
  bug-bait than the same constructs spread across eight modules (steering gap 1).
- `--profile structured-emission-max` becomes true to its name. **`.2` must also correct
  the two user-facing statements that describe it** — `USER_GUIDE.md:363` ("turns on all
  four emit-projections") and `README.md:875` ("all four emit-projections on") — and the
  Knowledge Map `reverify` line for `knob-presets-and-cli-flags`, which pins the old
  four-knobs-at-`1.0` expectation.
- The `cone_function` absorption rule gains a complete consumer census (`.4`), closing a
  latent trap before any future capability springs it. Byte-identical today.
- The lane's discipline holds: no new IR node, no new computed truth, no new emitted
  token, no default changed, nothing retired.
- Evidence is banked as a `docs/evidence/` digest (decision `0030`) — this tree is that
  mechanism's second customer, as intended.

## Open questions (to be resolved at `.2` / `.3` / `.4`)

- The exact flag name (`--emit-surface-interaction-gate` proposed) and `ScenarioSet`
  variant, and whether the saturation scenario shares the universal focus-config
  constructor with a probability parameter (proposed) or gets its own.
- Whether `distinct_emit_surfaces` is stored per-module in `ModuleReport` (proposed —
  it makes the report self-describing) or recomputed only in the coverage pass.
- The exact selector-shape values for the universal focus config. `comb_mux_prob = 0.35`
  / `case_mux_prob = 0.35` / `casez_mux_prob = 0.35` measured clean and gives all eight
  surfaces on all three strategies; `.3` pins the final numbers against the real gate.
- Whether the preset should also raise `terminal_reuse_prob` (it feeds `{N{x}}`
  replication, and so `generate_loop` candidate density) or leave graph shape alone
  beyond the three selector knobs.
- Whether `.4`'s consumer census should be factored into a shared
  `ir::use_counts` helper (one mechanism — `feedback_full_factorization`) rather than
  living privately in `cone_function_emit.rs`, given that any future absorbing pass would
  need the identical census.

## Tree split

`EMIT-SURFACE-INTERACTION-GATE` continues:

- **`.1`** (this leaf, design) — decision `0032`: the gate shape, the co-occurrence proof
  mechanism, `soft_union`'s placement, the preset's redefined meaning, the named
  interaction risks, and the measurements behind each. Docs-only.
- **`.2`** (`pending`) — the preset: all eight surfaces at `0.25` + the three selector
  knobs, the corrected description, the preset ↔ knob-list drift test, and the two
  user-facing doc corrections + the Knowledge Map `reverify` line.
- **`.3`** (`pending`) — the gate: the scenario set, `distinct_emit_surfaces`, the three
  coverage facts, the downstream-clean run, and the banked `docs/evidence/` digest.
- **`.4`** (`pending`, new) — harden `cone_function_emit::compute_use_counts` to count
  `Memory` and `Fsm` consumers; a regression test pinning that a gate feeding a memory
  port is never absorbed; byte-identical (a no-op on every currently-constructible
  module).

## Links

- Owner doctrine: **DECIDE, DON'T ASK** (`MEMORY.md` standing directives, `2026-07-30`),
  `feedback_pick_and_roll_at_no_frontier`.
- Lane / ROADMAP: steering gap 1 (richer structured emission — interaction density),
  steering gap 3 (axes that fire only by chance; here, an invariant that never fires).
- Doctrine: `feedback_rules_first_generation` (the gate changes no generation rule),
  `feedback_never_retire_strategies` (the eight single-surface gates stay; every surface
  stays default-off), `feedback_full_factorization` (co-occurrence is projected from the
  existing `emitted_*` booleans — one mechanism, not two), decision `0030`
  (`EVIDENCE-CITATIONS` — `.3` banks a digest).
- Precedents: decisions `0012`–`0016`, `0025`, `0027`–`0029` (the nine surfaces),
  `0028` (metric-keyed coverage detection), `0010` (the Verilator-only up-opt scenario
  shape), `0021` (the preset registry and the `default → config → profile → explicit`
  resolution order).
- Reuse / touch points: `src/config.rs` (the `structured-emission-max` preset +
  its drift test), `src/bin/tool_matrix.rs` (the gate flag, the `ScenarioSet` variant,
  the focus configs, `distinct_emit_surfaces`, the coverage facts),
  `src/ir/cone_function_emit.rs` (`.4`'s consumer census),
  `book/src/structured-emission.md` + `USER_GUIDE.md` + `README.md` (the preset's meaning),
  `docs/evidence/` (`.3`'s digest).
