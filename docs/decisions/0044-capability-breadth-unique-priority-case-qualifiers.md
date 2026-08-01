---
id: capability-breadth-unique-priority-case-qualifiers
title: The next CAPABILITY-BREADTH-EXPANSION construct is a default-off `unique` / `priority` **case-qualifier** projection of `CaseMux` and `CasezMux` — registered as a THIRD strand `.3` beside the deferred `.1` up-opt strand, not as a reframing of it
answers:
  - "can ANVIL emit a unique case or a priority case"
  - "what does unique_case_prob do"
  - "what does priority_case_prob do"
  - "is a unique case qualifier valid by construction in ANVIL"
  - "are ANVIL's emitted case and casez blocks full and parallel"
  - "does every anvil case block have a default arm"
  - "can two casez arms overlap in anvil output"
  - "why is unique casez safe in ANVIL"
  - "what is the third CAPABILITY-BREADTH-EXPANSION strand"
  - "why was CAPABILITY-BREADTH-EXPANSION.1 not reframed to synthesizable breadth"
  - "is the version-distinctive up-opt seam exhausted"
  - "does iverilog support unique case"
  - "what does vvp.tgt sorry Case unique/unique0 qualities are ignored mean"
  - "how do I prove a full/parallel case claim over a real corpus"
  - "which qualifier is safe on a casez"
date: 2026-08-01
status: accepted
tags: [capability, breadth, case-qualifier, unique, priority, case-mux, casez-mux, emission, downstream, valid-by-construction, rules-first, north-star, measurement]
reverify: "sed -n '797,806p' src/emit/sv.rs   # the `default:` arm is written unconditionally for every non-projected CaseMux/CasezMux => FULL; then: grep -n 'wildcard_bits = 1' -A 6 src/gen/cone/motifs.rs and grep -rn 'build_casez_patterns' src/ --include='*.rs'   # the SOLE casez pattern source, giving arm i the care-value i with one don't-care LSB => PARALLEL. These two reads re-establish the by-construction claim from tracked source alone. The corpus/runtime numbers below came from a scratch checker under .cache/ (untracked by design, 0031/0043); its DURABLE replacement is the in-crate #[test] required by `.4`'s acceptance, not a tracked shell script."
evidence: >
  Measured 2026-08-01 with Verilator 5.046, Yosys 0.64 and Icarus Verilog 13.0.
  (a) Emitter reading — src/emit/sv.rs:800-806 writes a `default:` arm for every non-projected
  CaseMux/CasezMux (FULL by construction); src/emit/sv.rs:712-721 labels CaseMux arm i as
  `SW'd{i}` (distinct by construction); src/gen/cone/motifs.rs:832-841 `build_casez_patterns` is
  the SOLE casez pattern source and gives arm i the care-value i with exactly one don't-care LSB
  (disjoint by construction). (b) Corpus measurement — 120 generated modules over 3 construction
  strategies yielded 50,761 case/casez blocks (36,346 case + 14,415 casez): 50,761 FULL, 50,761
  PARALLEL, 0 nested, 0 unparsed, 0 violations; a separate fsm_prob=1.0 corpus yielded 130 blocks
  (106 of them nested case(state_q)->case(sel) with symbolic localparam labels): 130 FULL, 130
  PARALLEL, 0 violations. (c) Checker non-vacuity — the same checker on a hand-written overlapping
  casez + a default-less case reports both and exits 1. (d) Downstream ON-vs-OFF over 24 gate-shaped
  modules / 169 qualified blocks — verilator --lint-only (the repo-owned argv) 0 warnings ON and
  OFF; verilator -Wall at --language 1800-2012/2017/2023 68 vs 68, delta 0; Yosys both repo modes
  0 failures / 0 messages ON and OFF, and the synthesized cell counts are identical for 24/24
  modules; iverilog -g2012 exit 0 ON and OFF. (e) Runtime violation checking — verilator --binary
  --assert on a real CaseMux module (exhaustive selector sweep) and on a real CasezMux module with
  `unique casez` on all 5 blocks (20,000 vectors) reports ZERO violations and output-identical
  results, while the hand-written overlapping negative control reports
  "Assertion failed ... unique case, but multiple matches found for '3'h0'". (f) The one
  non-silent tool result — iverilog emits `vvp.tgt sorry: Case unique/unique0 qualities are
  ignored.` per unique/unique0 block (exit 0; `priority` is silent). (g) LRM grounding — IEEE
  1800-2017 section 12.5.3, local cache .cache/local-references/sv/2017.
---

# 0044 - `unique` / `priority` case qualifiers: a third CAPABILITY-BREADTH-EXPANSION strand

- Date: 2026-08-01
- Status: accepted
- Tree: `CAPABILITY-BREADTH-EXPANSION.3` (design leaf; opens the third strand, splits `.3` + `.4`)
- Activated by: autonomous PNT selection at the tree's `.1`-deferred frontier
  (`0041` §(b) *decide, don't ask*). The scope call this record makes — a **new strand**
  rather than a rewrite of `.1` — is recorded in full below precisely because it was
  the one judgement an agent could have got wrong silently.

## Context

### The `.1` premise, and why this record does not rewrite it

`CAPABILITY-BREADTH-EXPANSION.1` asks for the *"next **version-distinctive** up-opt after
`union soft`"* and names `enum`/`typedef` and packed multi-dimensional arrays as candidates.
Two independent probes have now found that premise close to exhausted:

- the `.2a` probe (`2026-06-22`, decision `0024`) found the named candidates accepted at
  **every** Verilator `--language` mode plus Yosys and Icarus — so they are not
  version-distinctive and carry **no down-gating teeth**, re-confirming decision `0010`;
- the pre-`.1` probe (`2026-08-01`) added the structural reason: post-2012 SystemVerilog
  additions are overwhelmingly **verification** features (assertions, classes, coverage,
  randomization) that a *synthesizable* generator cannot emit. The genuinely-2023 clean space
  with the installed tools is essentially `union soft`, which already ships.

That same probe measured a large **2012-legal** gap — **zero** emitter string literals for
`enum`, `unique`, `priority`, `always_latch`, `casex`, `interface`, `modport`, `inside` — and
recommended reframing `.1` from *version-distinctive up-opt* to *synthesizable breadth*.

**This record declines the reframing and opens a third strand instead.** The two questions are
genuinely different:

- **`.1` asks:** *which post-2012 construct is both synthesizable and distinctive?* That is a
  live question whose answer depends on the **standard** and on **tool support**, both of which
  move. Rewriting `.1`'s goal would delete the question, and **nothing else in the repository
  asks it** — the `SV-VERSION-TARGETING` tree is closed.
- **`.3` asks:** *which 2012-legal synthesizable construct is missing from the emitter?* A
  different, larger and immediately-actionable space.

Merging them would bury a specific question inside a general one — the error decision
[`0039`](0039-sweep-exemption-past-vs-present-and-recorded-recall.md) §(e) rejected for `0033`
and [`0041`](0041-owner-standing-directives-recorded-in-layer-c.md) rejected for folding
workflow directives into `0031`. The tree is named **CAPABILITY-BREADTH-EXPANSION** and its
stated goal is *"the highest user-visible value per effort capability breadth"*, so a breadth
strand needs no goal rewrite at all: only *"two strands"* becomes *"three"*. `.1` stays
`pending`, **deferred-not-retired**, with its evidence attached
(`feedback_never_retire_strategies`).

### The construct

IEEE 1800-2017 **§12.5.3** lets `case`, `casez` and `casex` be qualified by `priority`,
`unique` or `unique0`:

> *"A priority-case shall act on the first match only. Unique-case and unique0-case assert that
> there are no overlapping case_items and hence that it is safe for the case_items to be
> evaluated in parallel. … If the case is qualified as priority or unique, the simulator shall
> issue a violation report if no case_item matches."*

So the two qualifiers assert two different things:

| qualifier | asserts FULL (some item matches) | asserts PARALLEL (no two items match) |
| --- | --- | --- |
| `priority` | yes | no |
| `unique` | yes | yes |
| `unique0` | no | yes |

These are **SV-2005/2009-era** constructs, hence 2012-legal — which is exactly why they belong
to `.3` (breadth) and not to `.1` (up-opts). Their value is not novelty of syntax: a qualifier
routes a tool down a **different code path** — full/parallel-case *inference* in synthesis, and
violation-check *instrumentation* in simulation — which is precisely where lint, synthesis and
simulation implementations disagree. Generating that legally, on purpose, is the project's
stated north star (*"stress such tools and expose real bugs"*).

ANVIL emits **zero** of them today.

## The valid-by-construction argument, and the measurement that checked it

A qualifier is an **assertion**. Emitting one that can be false would be the opposite of
valid-by-construction: it manufactures exactly the sim/synth divergence class that makes tools
disagree, which is worth *generating deliberately* but never worth emitting *accidentally*. So
the load-bearing question is not *"do tools accept it?"* but *"is the assertion true for every
block ANVIL can emit?"*

Both properties fall out of the emitter, with **no analysis pass and no
generate-then-filter** — the generator already knows the answer:

- **FULL.** `src/emit/sv.rs:800-806` writes a `default:` arm for every `CaseMux`/`CasezMux`
  that renders as a `case`/`casez` statement. Because `default` is itself a `case_item`, a
  match always exists, so the "no case_item matches" violation **cannot** fire — for either
  qualifier, and regardless of how few arms the gate has.
- **PARALLEL.** For `CaseMux`, `src/emit/sv.rs:712-721` labels arm `i` as `SW'd{i}` — the
  sequential integers `0..N-1`, distinct by construction. For `CasezMux`,
  `src/gen/cone/motifs.rs:832-841` (`build_casez_patterns`) is the **sole** pattern source in
  the generator, and it gives arm `i` the care-value `i` in the high bits with exactly **one**
  don't-care LSB — so two arms overlap iff two arm indices coincide, which they cannot.

### Correction: `CasezMux` IS parallel by construction

The pre-`.1` findings block asserted the opposite — that `CasezMux` arms *"can overlap for a
given selector value"*, so `unique` there would be a false assertion and the qualifiers had to
be gated per gate kind. **That claim is withdrawn.** It reasoned from the *IR shape* (the
operand triples `(pattern, wildcard_mask, data)` **can** express overlap) rather than from the
**generator** (which only ever builds one disjoint family). The distinction matters and is worth
carrying: *what the IR can represent* is not *what the generator constructs*, and a
by-construction claim must be read off the constructor, never off the type.

### What was measured

Reading the constructor is an argument; the corpus is the check. A nesting- and
symbol-aware checker was run over real emitted output:

| corpus | blocks | nested | unparsed | FULL | PARALLEL | violations |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `CaseMux`/`CasezMux`, 120 modules × 3 construction strategies | **50,761** | 0 | 0 | **50,761** | **50,761** | **0** |
| FSM (`fsm_prob = 1.0`, Mealy on), 12 modules | **130** | 106 | 0 | **130** | **130** | **0** |
| hand-written negative control | 2 | 0 | 0 | 1 | 1 | **2** → exit 1 |

The negative-control row is not decoration. `DOCTRINE_ENFORCEMENT.md` §9 states the general
acceptance test for any coverage-shaped check — *delete the subject and re-run it* — and the
first version of this checker **failed that test on its own author**: it was nesting-unaware, so
an FSM's outer `case (state_q)` swallowed the arms of its first inner `case (sel)`, and it
**silently skipped** every block whose labels it could not parse. It reported a clean FSM result
it had never actually looked at. The rewritten checker tracks nesting, resolves symbolic
`localparam` state labels, and **counts unparsed labels as violations** — which immediately
surfaced 8 more (a 1-bit FSM state is emitted as a bare `localparam logic NAME = 1'h0;` with no
`[W:0]` range). The `nested: 0 / unparsed: 0` cells in the table above are what makes the
50,761 number trustworthy rather than merely large.

## Downstream acceptance (measured, ON vs OFF)

24 gate-shaped modules (the vetted `case_mux_if` / `casez_mux_if` focus configs with the
if-chain projection off, so the `case`/`casez` statement form is emitted), duplicated into an
OFF copy and an ON copy differing **only** in the qualifier token; 169 blocks qualified in the
ON copy, alternating `unique` and `priority`:

| check | OFF | ON |
| --- | --- | --- |
| `verilator --lint-only` (the repo-owned argv) | 0 warn/err | **0 warn/err** |
| `verilator --lint-only -Wall`, `--language 1800-2012` / `2017` / `2023` | 68 | **68 (Δ = 0)** |
| `yosys synth -noabc; check` | 0 fail, 0 msg | **0 fail, 0 msg** |
| `yosys synth; abc -fast; opt -fast; check` | 0 fail, 0 msg | **0 fail, 0 msg** |
| synthesized cell counts, per module | — | **identical, 24/24** |
| `iverilog -g2012` | exit 0, silent | **exit 0**, see below |

The 68 `-Wall` findings are pre-existing `UNUSEDSIGNAL` notes belonging to the focus config, not
to the qualifier; the repo-owned bar (`verilator --lint-only`, no `-Wall`) is clean at 0. **Δ = 0
is the measurement that matters** — the lane's established ON-vs-OFF method (decisions `0028`,
`0029`).

Identical cell counts are the synthesis-side confirmation of the by-construction argument: a
qualifier whose assertion holds gives the synthesizer no new freedom, so the netlist cannot move.

### Runtime violation checking — the strongest leg

Verilator's `--assert` instrumentation *executes* the §12.5.3 checks:

- a real `CaseMux` module, `unique` and `priority` copies, **exhaustive** selector sweep
  (including the value that falls through to `default`): **zero violation reports**, outputs
  identical to the unqualified copy;
- a real `CasezMux` module with `unique casez` on all five of its blocks, **20,000 vectors**:
  **zero violation reports**, outputs identical;
- the hand-written overlapping negative control: `%Error: Assertion failed in tb_neg.u: unique
  case, but multiple matches found for '3'h0'`.

The third bullet is what makes the first two mean anything. Without it, "zero violations" is
indistinguishable from "the checker never ran".

### The one non-silent result, recorded rather than glossed

`iverilog -g2012` exits 0 on every qualified module, but for each `unique` / `unique0` block it
prints:

```
vvp.tgt sorry: Case unique/unique0 qualities are ignored.
```

`priority` is silent. This is an **acceptance** result — Icarus parses, elaborates and compiles
the construct, and ANVIL's Icarus column is a compile/elaboration accept column
(`run_iverilog_compile`) — but it is a diagnostic line, and ANVIL's bar is *boringly clean*.

It must be handled by an explicit decision, and **not** by the accident that saves it today:
`downstream::first_tool_warning`'s `iverilog-compile` arm matches lines containing `warning:`,
and `sorry:` does not match, so the gate would pass **without anyone having decided that it
should**. `DOCTRINE_ENFORCEMENT.md` §6.1 is explicit that a box must be *earned*, not passed on
a lexical coincidence. Pinned here, for `.4` to implement:

> **The `unique` scenario runs the Verilator + both-Yosys plan, with Icarus recorded as an
> accepting no-op** — the `union soft` precedent this tree's own Non-Goals already sanction
> (*"Yosys/Icarus a recorded no-op where they don't support the syntax"*), with the exact
> diagnostic quoted in the gate. **The `priority` scenario runs the full three-tool plan.**
> `first_tool_warning` is **not** widened: it is a shared surface every gate depends on, and
> re-classifying `sorry:` repo-wide to serve one scenario is a change to the wrong mechanism
> (`feedback_full_factorization`).

## Decision

**The third `CAPABILITY-BREADTH-EXPANSION` strand is a default-off, opt-in,
valid-by-construction `unique` / `priority` **case-qualifier** projection of the `CaseMux` and
`CasezMux` gates** — a behaviour-preserving prefix on the `case` / `casez` statement the gate
already emits, whose assertion is guaranteed true by the emitter's always-present `default:`
arm (FULL) and by the generator's sequential arm indices (PARALLEL).

```systemverilog
always_comb begin
    unique case (sel)          // or: priority case (sel)
        2'd0: case_mux_0 = i_2;
        2'd1: case_mux_0 = i_2;
        2'd2: case_mux_0 = 4'hc;
        default: case_mux_0 = 4'h0;
    endcase
end
```

### The candidate set (rules-first, valid-by-construction)

- **A `Node::Gate` whose op is `GateOp::CaseMux` or `GateOp::CasezMux` that actually renders as
  the dynamic `always_comb case` / `casez` statement** — i.e. one for which
  `render_static_structured_gate` returns `None`. A constant-selector gate lowers to a
  continuous `assign` and has no `case` statement to qualify.
- **Minus any gate already claimed by the eighth or ninth surface.** A `CaseMux` in
  `case_mux_if_gates`, or a `CasezMux` in `casez_mux_if_gates`, emits an `if`/`else if` chain
  and **has no `case` keyword to prefix**. This is the exclusion that actually bites — unlike
  the vacuous ones the sibling projections carry — so the pass runs **after** those two and
  excludes their marks. (Qualifying the *chains* instead, via `unique if` / `priority if`, is a
  distinct construct and a recorded follow-up; decision `0028` already parked it.)
- **`ForFold` is not a candidate** (a loop fold, not a selector).
- **Rolled at the call site like every other knob**, through the one steering-aware
  `roll_knob` primitive (decision `0034`) with its own `KnobId`, so the knob is steerable and
  its fire rate lands in `coverage_readout` (decision `0035`).

### Construction discipline (the lane invariants)

- **Rules-first**: the qualifier is a deterministic re-rendering of an already-valid gate whose
  asserted properties the generator established when it built the arms. Never
  generate-then-filter.
- **Default-off / byte-identical**: `unique_case_prob` and `priority_case_prob` both default
  `0.0`; with them off the output is byte-identical and `tests/snapshots.rs` is untouched.
- **Two knobs, one mechanism.** The qualifiers are genuinely different assertions and must be
  independently steerable and independently gated (their tool plans differ, above), so each
  gets its own knob, `KnobId`, metric and coverage fact. They feed **one** annotation pass and
  **one** carrier — a `case_qualifiers: BTreeMap<NodeId, CaseQualifier>` — rolled in a fixed
  order so a gate receives **at most one** qualifier. Two knobs into one mechanism is not two
  mechanisms (`feedback_full_factorization`).
- **No new IR node, no new computed truth.** A pure emit-time annotation; the flat IR body,
  validators, CSE keys and `canonical_module_signature` are untouched, exactly as for the
  `mux_if` / `case_mux_if` / `casez_mux_if` projections. The structure-first ceiling of
  decisions `0004` / `0011` is unaffected — this adds emission *shape*, not behaviour.

### Downstream gate

A repo-owned `tool_matrix --case-qualifier-gate`, templated on `--case-mux-if-gate`: one
`unique_case_prob = 1.0` scenario and one `priority_case_prob = 1.0` scenario per construction
strategy, over both a `case_mux_prob`-biased and a `casez_mux_prob`-biased focus config, keyed
on the metrics (`saw_unique_case_qualifier`, `saw_priority_case_qualifier`) rather than on a
text scan. Tool plans per the split pinned above.

## Decisive test applied

*"Does the construct add a new legal shape without new whole-module behaviour or a default
output change; is the assertion it makes true for **every** block the generator can emit; and is
it accepted by every repo downstream tool?"*

Yes, yes with a 50,891-block measurement plus a firing runtime checker, and yes with one
recorded Icarus no-op for `unique`. `unique0` fails the *value* half of the test rather than the
safety half — because ANVIL always emits `default:`, `unique0` (parallel-only) asserts a strict
subset of `unique` (parallel + full) and adds nothing today; deferred, not retired.

## Rejected alternatives

- **Reframing `.1` instead of opening `.3`.** Rejected — see Context. It would delete the only
  place the version-distinctive question is asked, to save renaming *"two strands"* to
  *"three"*.
- **Escalating the reframing to the owner.** Rejected under `0041` §(b) (*decide, don't ask*):
  proceeding is neither unsafe nor useless-if-wrong — `.1` is untouched, `.3` is additive, and
  a reversal costs one tree edit. The disclosure obligation is met by this record, which
  `0041` §(b) is explicit is independent of the question.
  **Confirmed by the owner `2026-08-01`** (*"It is you to decide … you have everything you need
  to answer your own question"*): the scope call **stands unchanged**. The owner's ruling was
  about the *framing*, not the decision — this record's reasoning had been surfaced under a
  *"your call"* label with an offer to reverse, which re-opens at disclosure time the question
  §(b) closes at decision time. Recorded as [`0041`](0041-owner-standing-directives-recorded-in-layer-c.md)
  §(e); nothing in this record changed as a result.
- **`priority`-only, to dodge the Icarus diagnostic.** Rejected: `priority` is the *weaker*
  construct here. With a `default:` always present, `priority case` and plain `case` are
  semantically identical in both simulation and synthesis — first-match either way, and the
  `full_case` hint is moot — so `priority` alone would ship the qualifier with **no new tool
  code path exercised**. `unique` is the one that adds the runtime uniqueness check and the
  `parallel_case` inference. Picking the silent-but-inert qualifier to avoid a recorded no-op
  would be optimising the gate report instead of the product.
- **Widening `first_tool_warning`'s iverilog arm to catch `sorry:`.** Rejected as a change to
  the wrong mechanism — a shared classifier every gate depends on, altered to serve one
  scenario. The scenario's tool plan is the right lever.
- **Emitting a qualifier and *dropping* the `default:` arm** (which §12.5.3's NOTE says the
  qualifier makes unnecessary). Rejected, twice over: it would change existing output (not
  default-off), and it would convert a trivially-true FULL assertion into one that depends on
  `n_arms == 2**sel_width` — true only when the arm count is a power of two. The `default:`
  arm is what makes this construct free of analysis.
- **`unique` on `CasezMux` gated off** (the pre-`.1` recommendation). Rejected on measurement —
  see the Correction above.
- **A new IR node, or a `unique`-specific gate kind.** Rejected: an emit-time annotation of an
  existing gate, per the whole projection family.
- **Generate-then-filter** (emit qualifiers, then check the assertion post-hoc). Forbidden
  (`feedback_rules_first_generation`) — and unnecessary, since both properties are established
  at construction.
- **Changing the default output.** Rejected: opt-in only (`feedback_never_retire_strategies`).

## Consequences

- ANVIL gains a third `CAPABILITY-BREADTH-EXPANSION` strand and its first **case-qualifier**
  construct; the default build and `--artifact dut` stay byte-identical (both knobs default
  `0.0`).
- The DUT lane gains a legal shape that routes downstream tools through full/parallel-case
  inference and violation-check instrumentation — a new bug-surfacing surface, and one the
  project can emit *safely* only because the assertion is guaranteed by construction.
- **A prior finding is corrected on measurement.** The `CasezMux` overlap claim is withdrawn;
  the reasoning error it came from — reading a by-construction property off the IR type instead
  of off the constructor — is recorded above as the durable lesson.
- **The pre-`.1` findings block is superseded, not deleted** — it is marked in place on the
  tree and points here (`MEMORY_ARCHITECTURE.md` §10).
- `.1` remains `pending` and deferred-not-retired, with both probes attached. `unique0`,
  `unique if` / `priority if` on the eighth/ninth-surface chains, and the FSM `case (state_q)`
  blocks (measured full+parallel above, so already known safe) are recorded follow-ups.

## Open questions (to be resolved at `.4a`)

- The exact `Module` carrier and enum: `case_qualifiers: BTreeMap<NodeId, CaseQualifier>` with
  `enum CaseQualifier { Unique, Priority }`, iterated in `NodeId` order for determinism.
- Knob names (`unique_case_prob` / `priority_case_prob` proposed) and their `KnobId` category —
  `emission` (the nine structured-emission surfaces' category, decision `0035`) vs `selectors`
  (where `case_mux_prob` / `casez_mux_prob` live). `emission` is proposed: this is an
  emit-projection, not a selector-shape choice.
- Metric names (`num_emitted_unique_cases` / `num_emitted_priority_cases` proposed) and the
  additive introspection schema MINOR bump **`1.27 → 1.28`**.
- Whether the two rolls are independent per gate with a fixed precedence, or one roll followed
  by a qualifier choice. Independent-with-precedence is proposed — it keeps each knob's fire
  rate directly interpretable in `coverage_readout`.
- Whether the emit-surface interaction gate (decision `0032`) gains the qualifier knobs as a
  tenth and eleventh axis, given the real exclusion against the eighth/ninth surfaces.

## Tree split

`CAPABILITY-BREADTH-EXPANSION` stays `active`:

- **`.1`** (SV-version up-opt strand) — unchanged, `pending`, **deferred-not-retired**, with the
  `.2a` and pre-`.1` probes attached.
- **`.2`** (Mealy FSM outputs) — `done`.
- **`.3`** (this leaf, design) — decision `0044`: the construct, the by-construction argument,
  the corpus and runtime measurements, the candidate set, the knobs, the gate and its per-
  qualifier tool plans, and the rejected alternatives. Docs-only.
- **`.4`** (impl, `pending`) — the knobs + the gen-time annotation pass + the emitter prefix +
  the metrics + the downstream-clean gate + book/USER_GUIDE/KM. Default-off / DUT
  byte-identical. Pre-split into `.4a` (design-detail) + `.4b` (impl, itself `.4b.1` live /
  `.4b.2` metric+gate / `.4b.3` docs) when picked.
- **future (`.5`+)** — `unique0`; `unique if` / `priority if` on the eighth/ninth-surface
  chains; the FSM `case (state_q)` / `case (sel)` blocks; each with its own decision.

## Links

- Owner doctrine: [`0041`](0041-owner-standing-directives-recorded-in-layer-c.md) §(b)
  (*decide, don't ask* — and its explicit carve-out that disclosure is still mandatory).
- Lane / ROADMAP: steering gap 1 (feature breadth), the structure-first ceiling (steering gap 4
  — this adds shape, not behaviour).
- Doctrine: `feedback_rules_first_generation`, `feedback_never_retire_strategies`,
  `feedback_full_factorization`.
- Precedents: [`0010`](0010-sv-version-first-upopt-soft-packed-union.md) (the recorded-no-op
  tool plan), [`0028`](0028-structured-emission-eighth-surface-case-mux-priority-chain.md) and
  [`0029`](0029-structured-emission-ninth-surface-casez-mux-masked-priority-chain.md) (the
  `CaseMux`/`CasezMux` projections this must exclude, and the ON-vs-OFF Δ method),
  [`0024`](0024-mealy-fsm-outputs.md) (the `.2a` not-version-distinctive probe),
  [`0032`](0032-emit-surface-interaction-gate.md) (surfaces interacting),
  [`0034`](0034-one-steering-aware-knob-roll-primitive.md) / [`0035`](0035-steering-width-motif-and-emission-knobs.md)
  (every knob rolls through one steering-aware primitive and carries a `KnobId`).
- Evidence discipline: `DOCTRINE_ENFORCEMENT.md` §6.1 (a box is earned, not ticked) and §9
  (delete the subject and re-run the check) — applied to this record's own checker, which
  failed it on the first attempt.
- Reuse / touch points: `src/emit/sv.rs` (the structured-case `always_comb` section — a
  keyword prefix on the `case`/`casez` line), `src/config.rs` (the two knobs + CLI flags),
  `src/ir/` (a `case_qualifier.rs` annotation pass beside `case_mux_if_emit.rs`),
  `src/ir/knob_id.rs` (two `knob_ids!` rows), `src/metrics.rs` (the two metrics, schema
  `1.27 → 1.28`), `src/bin/tool_matrix.rs` (`--case-qualifier-gate`),
  `book/src/structured-emission.md` + `book/src/knobs.md` (user-facing, at `.4b.3`).
