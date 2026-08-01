# Structured Emission Surfaces

Most of this book is about *what logic* anvil builds: cones, flops,
sharing, hierarchy. This chapter is about something narrower and
later in the pipeline — the **shape** of the SystemVerilog text the
emitter prints for logic that is already decided.

By default that shape is deliberately flat: a `module`, one `assign`
(or `always_comb` / `always_ff`) per node, child instantiations, and
output drives. Every downstream parser, elaborator, linter, and synth
tool therefore only ever sees that one structural form. **Structured
emission** is the lane that lets anvil re-render an *already valid*
construction in a richer SystemVerilog surface — today, across nine surfaces: a
single-gate `function`, a `generate for` loop, a `task`, a wider-lane `generate
for` part-select, a whole-cone `function`, a multi-output `task`, a procedural
`if`/`else`, an `if`/`else if` priority chain, and a masked `if`/`else if`
priority chain, and later nested `generate` or an `interface` — so the
tools have more legal structural variety to ingest,
and more places to trip over a real bug. (That bug-surfacing purpose is the
[project's north star](core-idea.md); structured emission adds
*shape*, never *behaviour*.)

This is the same trick the
[SystemVerilog-2023 `union soft` overlay](knobs.md#systemverilog-version-target)
and the packed-`struct` aggregate already use: an **emit-time
projection** of an existing construct, default-off so the byte-identical
contract holds, and proven downstream-clean before it ships.

## The first surface: a combinational `function automatic`

The first structured surface anvil ships is a **combinational
`function automatic`**. Conceptually, a combinational gate plus its
fan-in is a little expression tree whose leaves are the module's own
signals. SystemVerilog already has a name for "a named, reusable
expression over some inputs": a function. So anvil can take a gate that
it was *about to print inline* and instead print it as a function
declaration plus a call — without changing what the circuit computes.

It is governed by one knob,
[`function_emit_prob`](knobs.md#structured-emission) (the
`--function-emit-prob` CLI flag since `KNOB-ERGONOMICS-AND-PRESETS.2b.1`, or
`--config` JSON; default `0.0`), so with the knob off the output is
byte-identical and nothing in the default lane changes.

### Before and after

Here is a small combinational module with the knob **off** (the
default). The adder `add_0` is printed inline:

```systemverilog
    wire [3:0] add_0;

    assign slice_0 = i_2[2:0];
    assign add_0 = i_1 + casez_mux_0;
```

With `function_emit_prob = 1.0`, the *same* `add_0` gate is projected
to a `function automatic` of its two operands, and its use site becomes
a call. Nothing else in the module moves:

```systemverilog
    wire [3:0] add_0;

    function automatic logic [3:0] add_0__f(input logic [3:0] a0, input logic [3:0] a1);
        add_0__f = a0 + a1;
    endfunction

    assign slice_0 = i_2[2:0];
    assign add_0 = add_0__f(i_1, casez_mux_0);
```

`add_0__f(i_1, casez_mux_0)` evaluates to exactly `i_1 + casez_mux_0`,
so the module's behaviour is unchanged. The only difference is a new
*structural shape* the downstream tools must parse, elaborate, and
inline — which is the whole point.

## What gets wrapped (and what doesn't)

The first cut is intentionally minimal — the **single-gate "operand
function"**. anvil wraps *one* selected `Gate` node as a function of
its **direct operands**. Because those operands are already module-level
wires, ports, or literals, the function needs no private locals and
there is zero scoping or sharing hazard: the call site just passes the
same references the inline `assign` would have used.

Selection is **rules-first** ([by construction](by-construction.md), never
generate-then-filter): at construction time anvil rolls
`function_emit_prob` for each *qualifying* gate and marks the winners.
A gate qualifies when it is an ordinary combinational operation used in
full. Two kinds are deliberately **excluded**, and neither is retired —
they still emit exactly as before, just inline:

- **Structured selectors** (`case` / `casez` muxes, bounded `for`-folds)
  are already their own richer surface; they are not re-wrapped.
- **`Slice`** (a bit-select like `a[3:0]`) reads only a *sub-range* of
  its operand. Passing the full-width operand into a function parameter
  would leave the upper bits unused, which a strict Verilator lint
  (`-Wall UNUSEDSIGNAL`) correctly flags. A slice-aware projection that
  passes only `src[hi:lo]` is a recorded follow-up; until then a slice
  stays inline.

Because the body is a re-expression over **positional** parameters
(`a0`, `a1`, …) rather than a name-to-node mapping, a gate whose operands
repeat is handled cleanly — each occurrence becomes its own positional
parameter:

```systemverilog
    function automatic logic [7:0] concat_0__f(input logic [3:0] a0, input logic [3:0] a1);
        concat_0__f = {a0, a1};
    endfunction
    ...
    assign concat_0 = concat_0__f(case_mux_0, case_mux_0);
```

The function is **combinational only**. A flop's `Q` is a *leaf*
parameter — the projection never recurses through a register edge or a
child-instance boundary — so a `function automatic` never carries clock
or sequential logic. This is exactly the
[`output_support` support-leaf boundary](agent-mcp.md) the introspection
cone already uses.

## Reproducing it

The knob has a `--function-emit-prob` CLI flag (since
`KNOB-ERGONOMICS-AND-PRESETS.2b.1`) or you can set it through a `--config`
JSON. The example above comes from this recipe:

<!-- book-test: skip — config-file edit + a forced-knob sweep; not the default generator one-liner -->
```bash
anvil --seed 42 --dump-config > base.json
# edit base.json: set "function_emit_prob": 1.0 (a comb-only shape makes it easy to read:
# "flop_prob": 0.0, "min_width": 4, "max_width": 4, "gate_struct_weight": 0)
anvil --seed 42 --config base.json
```

Flip `function_emit_prob` back to `0.0` and the output is byte-identical
to the default lane — the contract the
[reproducibility guarantee](knobs.md) depends on.

## Why this surface first

Three properties make a combinational function the right first cut, and
they are recorded in full in decision `0012`
(`docs/decisions/0012-structured-emission-first-surface-combinational-function.md`):

- **Universally downstream-clean.** Automatic combinational functions
  are inlined cleanly by Verilator, *both* repo Yosys modes, and Icarus.
  `interface` / `modport` synthesis support in Yosys is weak and
  version-inconsistent, which would put the "clean across every tool"
  bar at risk — so it is deferred.
- **Minimal blast radius.** It is an emit-time projection — no new IR
  node, no new generator truth, default-off byte-identical. Nested
  `generate` is more emitter surgery (genvar scoping, loop bounds) for
  comparable first-cut value, so it too is deferred.
- **A genuinely new structural shape.** A function declaration and a
  call are a real new thing for a tool to parse, elaborate, and lower —
  not a cosmetic rewrite.

`task`, nested `generate`, and `interface` / `modport` each remain
candidate *future* surfaces, to be decided on their own merits when
picked. Consistent with anvil's
[scope discipline](non-goals.md), each lands as its own opt-in knob and
none of today's inline shapes is removed when they do.

## How anvil proves it

Producing a new surface is not enough; anvil proves the tools *accept*
it. Two repo-owned mechanisms back this surface:

- A `num_emitted_combinational_functions` metric (a post-hoc count of
  the marked gates) is surfaced in the
  [introspection document](agent-mcp.md) (schema `1.8`), so an agent can
  see how many functions a run emitted.
- The repo-owned `tool_matrix --function-emit-gate` forces
  `function_emit_prob = 1.0` over comb-only DUTs across all three
  construction strategies and fails unless the emitted functions are
  accepted **warning-clean** by Verilator and both Yosys modes (and
  Icarus when enabled), gated on a `saw_combinational_function_emit`
  coverage fact. It is banked clean (3 scenarios / 12 modules / 608
  emitted functions / `coverage_gaps = []`).

See the [Knobs reference](knobs.md#structured-emission) for the knob
itself; the `tool_matrix --function-emit-gate` acceptance gate is
documented in `USER_GUIDE.md` ("Tool matrix sweeps" → "Gate
invocations"), which owns every gate invocation.

## The second surface: a `generate for` loop

The second structured surface is a **`generate for` loop**. A
`generate` loop produces genuine *repeated* structure that an elaborator
must unroll — a richer thing to ingest than a flat `assign`. But a
faithful loop needs an **index-regular** source: bit (or lane) `g` of
the result has to be a clean function of the loop variable, or the
unrolled loop would not match what anvil already decided to build.

anvil's one cleanly index-regular construction is a **replication** of
the `{N{x}}` form — the `concat_0 = {5{slice_0}}` broadcast anvil
routinely emits (it is the idiom for fanning a 1-bit select out across a
mask in one-hot muxes). Bit `g` of `{N{x}}` is *exactly* `x`, so the
replication re-renders as a loop with no change in meaning. It is
governed by one knob,
[`generate_loop_emit_prob`](knobs.md#structured-emission) (the
`--generate-loop-emit-prob` CLI flag since `KNOB-ERGONOMICS-AND-PRESETS.2b.1`,
or `--config` JSON; default `0.0`), so with the knob off the output is
byte-identical.

### Before and after

Here is a small combinational module with the knob **off** (the
default). The 5-bit replication `concat_0` is printed inline:

```systemverilog
    wire  slice_0;
    wire [4:0] concat_0;

    assign slice_0 = i_2;
    assign concat_0 = {5{slice_0}};

    assign o_0 = concat_0;
```

With `generate_loop_emit_prob = 1.0`, the *same* `concat_0` replication
is projected to a single-level `generate for` loop over its 5 bits, and
the inline `assign concat_0 = {5{slice_0}};` is suppressed. Nothing else
in the module moves:

```systemverilog
    wire  slice_0;
    wire [4:0] concat_0;

    genvar concat_0__gi;
    generate
        for (concat_0__gi = 0; concat_0__gi < 5; concat_0__gi = concat_0__gi + 1) begin : concat_0__gen
            assign concat_0[concat_0__gi] = slice_0;
        end
    endgenerate

    assign slice_0 = i_2;

    assign o_0 = concat_0;
```

The unrolled loop assigns `concat_0[0] … concat_0[4]` each to `slice_0`
— exactly `{5{slice_0}}` — so the module's behaviour is unchanged. The
only difference is a `generate` / `genvar` construct (the DUT lane's
first) for the tools to parse, elaborate, and unroll.

### What gets wrapped (and what doesn't)

Like the function surface, selection is **rules-first**
([by construction](by-construction.md)): at construction time anvil rolls
`generate_loop_emit_prob` for each *qualifying* replication and marks the
winners. A replication qualifies when it is a `{N{x}}` `Concat` — `N ≥ 2`
operands that are all the **same** signal — of **any lane width `LW ≥ 1`**
(the result is then `N·LW` bits wide). Two body shapes cover that:

- a **1-bit lane** drives one bit per iteration —
  `assign <wire>[gi] = x;` (bit `g` of the result is exactly `x`);
- a **wider lane** (`LW > 1`) drives one `LW`-wide group per iteration via an
  indexed **part-select** — `assign <wire>[gi*LW +: LW] = x;` (this is the
  [fourth surface](#the-fourth-surface-wider-lanes-via-a-part-select), decision
  `0015`; before it shipped, a wider lane stayed inline).

Both unroll byte-faithfully to `{N{x}}` because every group is the same lane.
The `generate for` and `function automatic` projections are mutually exclusive
on a gate (a replication marked for one is never also marked for the other), and
nothing is retired — a replication still emits inline `{N{x}}` when the knob is
off.

The loop increment is written `gi = gi + 1` — the most portable form,
accepted identically by every repo tool (`gi++` is equally valid and is
not foreclosed).

### How anvil proves it

The same two-mechanism proof as the function surface:

- A `num_emitted_generate_loops` metric (a post-hoc count of the marked
  replications) is surfaced in the
  [introspection document](agent-mcp.md) (schema `1.9`).
- The repo-owned `tool_matrix --generate-loop-gate` forces
  `generate_loop_emit_prob = 1.0` over comb-only DUTs across all three
  construction strategies and fails unless the emitted loops are accepted
  **warning-clean** by Verilator and both Yosys modes (and Icarus when
  enabled), gated on a `saw_generate_loop_emit` coverage fact. It is
  banked clean (3 scenarios / 12 modules / `coverage_gaps = []`).

The picked-second rationale (a `generate for` over `task` /
`interface` / a constant-predicate `generate if`) is recorded in decision
`0013`
(`docs/decisions/0013-structured-emission-second-surface-generate-loop.md`).

### Reproducing it

<!-- book-test: skip — config-file edit + a forced-knob comb-only shape; not the default generator one-liner -->
```bash
anvil --seed 12 --dump-config > base.json
# edit base.json: set "generate_loop_emit_prob": 1.0 (a small comb-only shape makes the
# one loop easy to read: "flop_prob": 0.0, "constant_prob": 0.0, "min_width": 4,
# "max_width": 8, "min_inputs": 3, "max_inputs": 5, "min_outputs": 1, "max_outputs": 2,
# "max_depth": 3)
anvil --seed 12 --config base.json
```

Flip `generate_loop_emit_prob` back to `0.0` and the output is
byte-identical to the default lane.

## The third surface: a combinational `task automatic`

The third structured surface is a combinational **`task automatic`**. It
is the exact parallel of the
[first surface](#the-first-surface-a-combinational-function-automatic) —
the same single combinational gate, the same direct-operand parameter
list — but expressed as a *procedural* `task` called from an
`always_comb` rather than a value-returning `function`. A `task` is a
genuinely different elaboration surface: it writes through an `output`
argument and is *called* as a statement, where a function is a
continuous-assign value. Giving a tool both forms is two distinct
"named, reusable computation" shapes to lower, not one shape twice.

It is governed by one knob,
[`task_emit_prob`](knobs.md#structured-emission) (the `--task-emit-prob`
CLI flag since `KNOB-ERGONOMICS-AND-PRESETS.2b.1`, or `--config` JSON;
default `0.0`), so with the knob off the output is byte-identical.

### Before and after

Here is a small combinational module with the knob **off** (the
default). The shift `shr_0` is printed inline:

```systemverilog
    wire [3:0] shr_0;

    assign shr_0 = i_2 >> 2'h3;

    assign o_0 = shr_0;
```

With `task_emit_prob = 1.0`, the *same* `shr_0` gate is projected to a
`task automatic` over its operands. The task writes its result into a
local `shr_0__tv` variable from an `always_comb`, and the gate's net is
then driven from that variable — so `shr_0` stays an ordinary
continuous-assign net and nothing downstream of it moves:

```systemverilog
    wire [3:0] shr_0;

    task automatic shr_0__t(output logic [3:0] o, input logic [3:0] a0, input logic [1:0] a1);
        o = a0 >> a1;
    endtask
    logic [3:0] shr_0__tv;
    always_comb shr_0__t(shr_0__tv, i_2, 2'h3);

    assign shr_0 = shr_0__tv;

    assign o_0 = shr_0;
```

The `always_comb` call computes `i_2 >> 2'h3` into `shr_0__tv`, and
`assign shr_0 = shr_0__tv;` drives the original net — so the module's
behaviour is unchanged. The only difference is a `task` declaration, an
`always_comb` task call, and an output-var passthrough for the tools to
parse, elaborate, and lower. (The constant operand `2'h3` folds to a
literal argument exactly as it would inline.)

### What gets wrapped (and what doesn't)

The candidate set is **identical to the function surface**: one
*ordinary combinational* `Gate` used in full. Structured selectors
(`case` / `casez` muxes, bounded `for`-folds) and `Slice` bit-selects are
excluded for the same reasons, and neither is retired — they still emit
inline. Selection is **rules-first**
([by construction](by-construction.md)): at construction time anvil rolls
`task_emit_prob` for each qualifying gate and marks the winners.

The four emit-projections are **mutually exclusive on a gate**: the task
pass runs last and skips any gate already marked for the
`function automatic`, `generate for`, or `union soft` projections, so a
gate is re-rendered by at most one surface.

The **integration form** is deliberately minimal — the *output-var +
passthrough* form shown above. The gate's wire stays a continuous-assign
*net*; the task writes a separate `logic` variable; a passthrough
`assign` connects them. Only the gate's own drive changes, exactly like
the function surface ("only the gate's own drive changes"). Making the
gate's wire *itself* the procedural variable was considered and rejected
for the first cut (it would perturb the uniform wire-declaration
section). Each task call gets its own `always_comb`.

Like the function, the task is **combinational only** — a flop's `Q` is a
leaf parameter, and the task never recurses through a register edge or a
child-instance boundary.

### How anvil proves it

The same two-mechanism proof as the prior surfaces:

- A `num_emitted_combinational_tasks` metric (a post-hoc count of the
  marked gates) is surfaced in the
  [introspection document](agent-mcp.md) (schema `1.10`).
- The repo-owned `tool_matrix --task-emit-gate` forces
  `task_emit_prob = 1.0` over comb-only DUTs across all three
  construction strategies and fails unless the emitted tasks are accepted
  **warning-clean** by Verilator and both Yosys modes (and Icarus when
  enabled), gated on a `saw_combinational_task_emit` coverage fact. It is
  banked clean (3 scenarios / 12 modules / 12 emitting a task /
  `coverage_gaps = []`).

The picked-third rationale (a combinational `task` over nested
`generate` / `interface` / `modport`) is recorded in decision `0014`
(`docs/decisions/0014-structured-emission-third-surface-combinational-task.md`).

### Reproducing it

<!-- book-test: skip — config-file edit + a forced-knob comb-only shape; not the default generator one-liner -->
```bash
anvil --seed 1 --dump-config > base.json
# edit base.json: set "task_emit_prob": 1.0 (a small comb-only shape makes the one
# task easy to read: "flop_prob": 0.0, "constant_prob": 0.0, "gate_struct_weight": 0,
# "min_width": 4, "max_width": 4, "min_inputs": 2, "max_inputs": 3, "min_outputs": 1,
# "max_outputs": 1, "max_depth": 2)
anvil --seed 1 --config base.json
```

Flip `task_emit_prob` back to `0.0` and the output is byte-identical to
the default lane.

## The fourth surface: wider lanes via a part-select

The fourth surface is not a new construct — it is a **broadening of the
[second surface](#the-second-surface-a-generate-for-loop)**. The first cut of
the `generate for` loop took only a **1-bit lane** (`{N{sel}}`), because then
each result *bit* is exactly the lane and the body `assign <wire>[gi] = sel;` is
trivially faithful. A **wider lane** — `{N{x}}` where `x` is `LW > 1` bits, like
the `{2{i_2}}` anvil routinely builds — is just as index-regular, but each
iteration now owns an `LW`-wide *group* of the result, so the body becomes an
indexed **part-select** `assign <wire>[gi*LW +: LW] = x;`. That part-select with
a genvar-computed base is a genuinely new elaboration shape for a tool to lower.

It shares the second surface's knob — [`generate_loop_emit_prob`](knobs.md#structured-emission)
(default `0.0`) — so there is **no new knob and no introspection schema bump**;
a marked wider-lane replication simply renders the part-select loop instead of
the inline `{N{x}}`.

### Before and after

Here is a small combinational module with the knob **off** (the default). The
2-bit input `i_2` is replicated to a 4-bit `concat_0` inline:

```systemverilog
    wire [3:0] concat_0;

    assign concat_0 = {2{i_2}};

    assign o_0 = concat_0;
```

With `generate_loop_emit_prob = 1.0`, the *same* `concat_0` replication is
projected to a `generate for` loop whose body is a 2-bit part-select, and the
inline `assign concat_0 = {2{i_2}};` is suppressed:

```systemverilog
    wire [3:0] concat_0;

    genvar concat_0__gi;
    generate
        for (concat_0__gi = 0; concat_0__gi < 2; concat_0__gi = concat_0__gi + 1) begin : concat_0__gen
            assign concat_0[concat_0__gi*2 +: 2] = i_2;
        end
    endgenerate

    assign o_0 = concat_0;
```

The loop drives `concat_0[0 +: 2]` then `concat_0[2 +: 2]`, each to `i_2` —
exactly `{2{i_2}}` — so the module's behaviour is unchanged. Only the marked
gate's drive changes; everything downstream of `concat_0` is byte-identical.

A **1-bit lane keeps the original `[gi]` body verbatim** — the part-select form
is taken only when `LW > 1`, so the second surface's shipped 1-bit output (and
its proofs) are untouched.

### How anvil proves it

- The wider lane reuses the second surface's
  [`num_emitted_generate_loops`](agent-mcp.md) metric and the repo-owned
  `tool_matrix --generate-loop-gate` (the corpus naturally contains wider-lane
  replications, so the gate exercises the part-select body once enabled).
- A deterministic library test asserts a marked wider-lane replication renders
  `assign <wire>[gi*LW +: LW] = x;` while a 1-bit lane still renders `[gi]`
  (the byte-identity guard).
- The construct is downstream-clean: a forced-knob sweep emits real wider-lane
  part-selects (e.g. `concat_0[concat_0__gi*16 +: 16] = i_2;`) accepted
  **warning-clean** by Verilator `-Wall` (zero new warnings vs the inline
  baseline), both Yosys modes, and Icarus — and the part-select is
  simulation-proven equal to `{N{x}}`.

The picked-fourth rationale (a wider-lane part-select over `interface` /
`modport` — empirically rejected — and nested `generate`) is recorded in
decision `0015`
(`docs/decisions/0015-structured-emission-fourth-surface-wide-lane-generate-loop.md`).

### Reproducing it

<!-- book-test: skip — config-file edit + a forced-knob comb-only shape; not the default generator one-liner -->
```bash
anvil --seed 74 --dump-config > base.json
# edit base.json: set "generate_loop_emit_prob": 1.0 (a small comb-only shape
# with a multi-bit replicated lane: "flop_prob": 0.0, "constant_prob": 0.0,
# "terminal_reuse_prob": 0.9, "gate_struct_weight": 8, "min_width": 2,
# "max_width": 4, "min_inputs": 2, "max_inputs": 3, "min_outputs": 1,
# "max_outputs": 1, "max_depth": 2)
anvil --seed 74 --config base.json
```

Flip `generate_loop_emit_prob` back to `0.0` and the output is byte-identical to
the default lane.

## The fifth surface: a multi-gate-cone `function automatic`

The fifth surface **deepens the [first surface](#the-first-surface-a-combinational-function-automatic)**.
The first cut wrapped a *single* gate over its direct operands — a one-line
function body. The fifth surface wraps a whole combinational **cone**: a root
gate plus the chain of interior gates that feed it, rendered as one `function
automatic` whose body is a topologically-ordered sequence of function-local
temporaries (one per interior gate) and whose return value is the root. The
function's parameters are the cone's **boundary leaves** (the primary inputs,
flop `Q`s, instance outputs, and other signals the cone reads), so it evaluates
to exactly the inline per-gate chain — **behaviour-preserving by construction**.

It uses its **own** knob,
[`cone_function_emit_prob`](knobs.md#structured-emission) (default `0.0`),
*separate* from the single-gate `function_emit_prob`, so the shipped single-gate
surface stays byte-identical and the two surfaces never blur. A new
[`num_emitted_cone_functions`](agent-mcp.md) metric counts the cones it emits,
bumping the introspection schema to `1.11`.

### Before and after

Here is a small combinational module with the knob **off** (the default). The
cone `i_2 - (i_1 ^ i_3)` is built as two inline gates:

```systemverilog
    wire [3:0] xor_0;
    wire [3:0] sub_0;

    assign xor_0 = i_1 ^ i_3;
    assign sub_0 = i_2 - xor_0;

    assign o_0 = sub_0;
```

With `cone_function_emit_prob = 1.0`, the *same* cone is projected to one
`function automatic`. The root `sub_0` becomes a call over the cone's three
boundary leaves; the interior gate `xor_0` becomes a function-local temporary;
and `xor_0`'s module wire **and** its inline `assign` are suppressed (it now
lives only inside the function):

```systemverilog
    wire [3:0] sub_0;

    function automatic logic [3:0] sub_0__cf(input logic [3:0] a0, input logic [3:0] a1, input logic [3:0] a2);
        logic [3:0] xor_0;
        xor_0 = a0 ^ a2;
        sub_0__cf = a1 - xor_0;
    endfunction

    assign sub_0 = sub_0__cf(i_1, i_2, i_3);

    assign o_0 = sub_0;
```

The function computes `xor_0 = i_1 ^ i_3` then returns `i_2 - xor_0` — exactly
the inline chain — so the module's behaviour is unchanged. Only the cone root's
drive changes; the output drive `assign o_0 = sub_0;` is byte-identical.

### What gets wrapped (and what doesn't)

- **The root** is any admissible combinational gate (not a `Slice`, not a
  procedural structured selector — the `function_emit` candidate rules) whose
  cone has **at least one** absorbable interior gate. A root with only leaf
  operands has no interior to absorb, so it is left to the single-gate surface.
- **An interior gate is absorbed only when it is used exactly once** in the whole
  module. Then its sole consumer is the cone edge that reached it, so suppressing
  its module wire and inline assign is provably safe. A **multi-use** (shared)
  gate stays a boundary parameter — keeping its own wire and assign — so the
  function still reads it by name. This keeps the emission `-Wall` clean: every
  parameter is used, and nothing is left undriven.
- **Constants fold inline** as literals inside the function body (they are not
  parameters).
- The cone surface is **mutually exclusive** with the four per-gate projections
  (single-gate `function`, `generate for` loop, `task`, `union soft`): it runs
  last and never absorbs or roots a gate already marked by one of them.
- **Combinational only** — the cone walk stops at flop `Q`s, instance outputs,
  and primary inputs (the support-leaf boundary).

### How anvil proves it

- The [`num_emitted_cone_functions`](agent-mcp.md) metric (a post-hoc count of
  `Module.cone_function_gates`) is surfaced in `--introspect` at schema `1.11`,
  so a sweep can confirm the surface fired.
- The repo-owned `tool_matrix --cone-function-gate` forces
  `cone_function_emit_prob = 1.0` over comb-only DUTs across all three
  construction strategies and requires the `saw_cone_function_emit` coverage
  fact — a genuinely-emitted cone function (detected from the SV text's
  `<root>__cf(` token, distinct from the single-gate `<wire>__f(`) accepted by
  Verilator **and** Yosys. Banked clean (3 scenarios / 12 modules / 148 cone
  functions / `coverage_gaps = []` / `12/0` Verilator + both Yosys + Icarus).
- Library tests pin the cone walk: a single-use interior is absorbed, a
  multi-use interior stays a boundary parameter, a zero-interior root is not
  marked, a sibling-marked gate is excluded, and a marked cone emits the
  multi-statement function while the unmarked default stays the inline chain.

The picked-fifth rationale (a multi-gate cone over the deferred multi-output
`task` and the source-less nested `generate`, with `interface` / `modport` still
disqualified) is recorded in decision `0016`
(`docs/decisions/0016-structured-emission-fifth-surface-cone-function.md`).

### Reproducing it

<!-- book-test: skip — config-file edit + a forced-knob comb-only shape; not the default generator one-liner -->
```bash
anvil --seed 4 --dump-config > base.json
# edit base.json: set "cone_function_emit_prob": 1.0 (a small comb-only shape
# makes the one cone easy to read: "flop_prob": 0.0, "constant_prob": 0.0,
# "gate_struct_weight": 0, "terminal_reuse_prob": 0.1, "min_width": 4,
# "max_width": 4, "min_inputs": 3, "max_inputs": 4, "min_outputs": 1,
# "max_outputs": 1, "max_depth": 2)
anvil --seed 4 --config base.json
```

Flip `cone_function_emit_prob` back to `0.0` and the output is byte-identical to
the default lane.

## The sixth surface: a multi-output `task automatic`

The sixth surface **generalizes the [third surface](#the-third-surface-a-combinational-task-automatic)**.
The single-gate task had one `output`. The sixth surface co-emits a
**co-supported group** of combinational gates (`k >= 2`, up to 8) into **one**
`task automatic` with several `output` arguments and a **deduplicated** `input`
list: a non-constant operand the gates *share* becomes **one** input formal feeding
multiple outputs — the "co-supported sink". One `always_comb` call drives a
per-member output var, and each member's net is driven by a passthrough `assign`.
Because each output is the member gate's exact operation over those formals, the
task computes exactly the inline assigns it replaces — **behaviour-preserving by
construction**.

It uses its **own** knob,
[`multi_output_task_emit_prob`](knobs.md#structured-emission) (default `0.0`),
*separate* from the single-gate `task_emit_prob`, so the shipped single-gate
surface stays byte-identical. A new
[`num_emitted_multi_output_tasks`](agent-mcp.md) metric counts the task groups it
emits, bumping the introspection schema to `1.14`.

### Before and after

Here is a small combinational module with the knob **off** (the default). Two
outputs are driven by two sibling gates that share the input `i_2`:

```systemverilog
    wire [3:0] xor_0;
    wire [3:0] not_0;

    assign xor_0 = i_1 ^ i_2;
    assign not_0 = ~i_2;

    assign o_0 = xor_0;
    assign o_1 = not_0;
```

With `multi_output_task_emit_prob = 1.0`, the *same* pair is co-emitted as one
multi-output `task automatic`. The shared operand `i_2` becomes a single input
formal `a1` that feeds **both** outputs; each member's `assign` becomes a
passthrough from its `<wire>__mtv` output var:

```systemverilog
    wire [3:0] xor_0;
    wire [3:0] not_0;

    task automatic xor_0__mt(output logic [3:0] o0, output logic [3:0] o1, input logic [3:0] a0, input logic [3:0] a1);
        o0 = a0 ^ a1;
        o1 = ~a1;
    endtask
    logic [3:0] xor_0__mtv;
    logic [3:0] not_0__mtv;
    always_comb xor_0__mt(xor_0__mtv, not_0__mtv, i_1, i_2);

    assign xor_0 = xor_0__mtv;
    assign not_0 = not_0__mtv;

    assign o_0 = xor_0;
    assign o_1 = not_0;
```

The task computes `o0 = i_1 ^ i_2` and `o1 = ~i_2` — exactly the inline pair —
with the shared `i_2` passed once as `a1`. Only the two members' drives change;
the output drives `assign o_0 = xor_0;` / `assign o_1 = not_0;` are byte-identical.

### Wider groups (`k > 2`)

A group is **not** limited to a pair. When more co-supported, mutually-independent
gates are available, the task absorbs them too — up to a bounded **8 members** —
so a single `task automatic` can carry three, four, or more `output`s over one
deduplicated `input` list. Here is a real **three-member** task (a forced
`multi_output_task_emit_prob = 1.0` run, seed 22), co-emitting `shr_0`, `mux_0`,
and `mux_1`:

```systemverilog
    task automatic shr_0__mt(output logic [30:0] o0, output logic [30:0] o1, output logic [18:0] o2,
                             input logic a0, input logic [18:0] a1, input logic [30:0] a2);
        o0 = a2 >> 3'h5;
        o1 = (a0) ? (a2) : (31'h3e5748b0);
        o2 = (a0) ? (a1) : (19'h21d25);
    endtask
    logic [30:0] shr_0__mtv;
    logic [30:0] mux_0__mtv;
    logic [18:0] mux_1__mtv;
    always_comb shr_0__mt(shr_0__mtv, mux_0__mtv, mux_1__mtv, i_4, slice_0, concat_0);

    assign shr_0 = shr_0__mtv;
    assign mux_0 = mux_0__mtv;
    assign mux_1 = mux_1__mtv;
```

Three outputs, **three** deduplicated inputs: the select `a0` (the module's `i_4`)
is shared — it feeds **both** `o1` and `o2` — and `a2` (the wire `concat_0`) feeds
both `o0` and `o1`. The group is built greedily: starting from the lowest member,
anvil admits each further gate that (1) shares a non-constant operand with **at
least one** member already in the group (so the group stays connected through shared
formals) and (2) is mutually fan-in-independent with **every** member (so no cycle
can close through the shared `always_comb`). This `k = 3` task is accepted
warning-clean by Verilator `-Wall` with **zero** new warnings versus the knob-off
build, and an `iverilog` simulation proves it bit-identical to the three inline
assigns it replaces.

### What gets wrapped (and what doesn't)

- **The members** are admissible combinational gates (not a `Slice`, not a
  procedural structured selector — the same candidate rules as the single-gate
  `task`). A group is a **`k >= 2`** set, bounded at **8 members** so any one task
  stays readable and a dense module still forms several distinct groups.
- **Each new member must be connected by a shared non-constant operand.** A
  candidate joins only if it shares a non-constant operand with **at least one**
  member already in the group. A shared *constant* folds inline as a literal (so it
  is never a shared formal); without a shared non-constant operand the task would be
  merely unrelated tasks fused, with no new interaction — so such gates are not
  grouped.
- **The members must be mutually fan-in-independent** — no member may lie in
  another member's fan-in cone. If one did, its net (driven by the shared task's
  passthrough) would feed, through gates outside the task, into a direct operand
  the task reads, closing a combinational cycle through the single `always_comb`
  call (a Verilator `UNOPTFLAT`). Each new member is checked against **every**
  current member, so the co-emitted task is cycle-free by construction at any size.
- **Members keep their module wires** (unlike a cone-function interior): they are
  co-equal roots, not absorbed, so there is no use-count rule and DAG-shared
  members are fine — only their drive changes.
- The surface is **mutually exclusive** with the five per-gate / per-cone
  projections: it runs after the single-gate `task` and before the cone `function`,
  and never groups a gate already marked by one of them.
- **Combinational only** — each member is a combinational gate; a flop `Q` is a
  leaf formal.

### How anvil proves it

- The [`num_emitted_multi_output_tasks`](agent-mcp.md) metric (a post-hoc count of
  `Module.multi_output_task_groups`) is surfaced in `--introspect` at schema
  `1.14`, so a sweep can confirm the surface fired.
- The repo-owned `tool_matrix --multi-output-task-gate` forces
  `multi_output_task_emit_prob = 1.0` over comb-only DUTs across all three
  construction strategies and requires the `saw_multi_output_task_emit` coverage
  fact — a genuinely-emitted multi-output task (detected from the SV text's
  `<leader>__mt(` token, distinct from the single-gate `<wire>__t(` and the cone
  `<root>__cf(`) accepted by Verilator **and** Yosys. Banked clean (3 scenarios /
  12 modules / 6 emitting a multi-output task / `coverage_gaps = []` / `12/0`
  Verilator + both Yosys + Icarus), with a **`k = 3`** group present among the
  emitted modules.
- Library tests pin the grouping at every size: a co-supported independent **pair**
  groups, a co-supported independent **triple** groups, the extension stops at a
  gate that shares no operand with the group, a fan-in-dependent gate is excluded
  even when it co-supports, the group is capped at 8 members, gates without a shared
  non-constant operand do not group, a shared *constant* alone does not, a `Slice` /
  structured / sibling-marked member is excluded, and a grouped triple emits a
  three-`output` task while the unmarked default stays the inline assigns.

The picked-sixth rationale (the deferred runner-up from the fifth-surface probe,
chosen for the genuinely-new "multiple `output` formals + a shared input formal"
elaboration interaction) is recorded in decision `0025`
(`docs/decisions/0025-structured-emission-sixth-surface-multi-output-task.md`).

### Reproducing it

<!-- book-test: skip — config-file edit + a forced-knob comb-only shape; not the default generator one-liner -->
```bash
anvil --seed 3 --dump-config > base.json
# edit base.json: set "multi_output_task_emit_prob": 1.0 (a small comb-only shape
# makes the one task pair easy to read: "flop_prob": 0.0, "constant_prob": 0.0,
# "terminal_reuse_prob": 0.9, "min_inputs": 3, "max_inputs": 3, "min_outputs": 2,
# "max_outputs": 2, "min_width": 4, "max_width": 4, "max_depth": 1)
anvil --seed 3 --config base.json
```

Flip `multi_output_task_emit_prob` back to `0.0` and the output is byte-identical
to the default lane.

## The seventh surface: a procedural `if`/`else`

The seventh surface is the **first procedural-conditional** shape in the lane. The
six surfaces above are `function` / `task` / `generate` projections; a 2:1 `Mux`
gate (`[sel, a, b]`, `sel` one bit) renders today as the **continuous-assign
ternary** `assign <wire> = (sel) ? (a) : (b);`, and the structured selectors
`CaseMux` / `CasezMux` render as `always_comb case` / `casez`. None of them emits a
procedural `always_comb` block with an **`if`/`else` statement** — a distinct
frontend/elaboration path. The seventh surface re-expresses a marked `Mux` as exactly
that: a per-gate `<wire>__cv` output var written by an `if`/`else`, the existing net
driven from it by a passthrough `assign` (the [third surface](#the-third-surface-a-combinational-task-automatic)'s
output-var + passthrough mechanism, but a bare `always_comb` `if`/`else` rather than
a `task` call). Because the `if`/`else` writes the gate's exact value (`sel == 1 ⇒ a`,
`sel == 0 ⇒ b` — the ternary's operand mapping), it is **behaviour-preserving by
construction**.

It uses its **own** knob,
[`mux_if_emit_prob`](knobs.md#structured-emission) (default `0.0`), separate from the
`task_emit_prob` / `function_emit_prob` family, so the shipped surfaces stay
byte-identical. A new [`num_emitted_mux_if_blocks`](agent-mcp.md) metric counts the
blocks it emits, bumping the introspection schema to `1.15`.

### Before and after

Here is a small combinational module with the knob **off** (the default). Two muxes
are emitted as continuous-assign ternaries (the second selects between a constant and
the first mux's wire):

```systemverilog
    wire [3:0] mux_0;
    wire [3:0] mux_1;

    assign mux_0 = (slice_0) ? (4'hf) : (4'h0);
    assign mux_1 = (eq_0) ? (4'he) : (mux_0);
```

With `mux_if_emit_prob = 1.0`, the *same* muxes are re-expressed as procedural
`always_comb` `if`/`else` blocks writing a `<wire>__cv` output var; each mux's
`assign` becomes a passthrough from that var:

```systemverilog
    wire [3:0] mux_0;
    wire [3:0] mux_1;

    logic [3:0] mux_0__cv;
    always_comb begin
        if (slice_0) mux_0__cv = 4'hf;
        else mux_0__cv = 4'h0;
    end
    logic [3:0] mux_1__cv;
    always_comb begin
        if (eq_0) mux_1__cv = 4'he;
        else mux_1__cv = mux_0;
    end

    assign mux_0 = mux_0__cv;
    assign mux_1 = mux_1__cv;
```

Each block writes exactly the ternary's value (`sel ⇒ a`, else `b`). Only the muxes'
own drives change — the `<wire>__cv` var carries the value and the net `mux_0` /
`mux_1` is driven from it by a passthrough, so every downstream consumer of `mux_0` /
`mux_1` is unchanged.

### What gets wrapped (and what doesn't)

- **The candidate is a plain 2:1 `Mux`** (a `GateOp::Mux` gate, exactly three
  operands, a one-bit selector). The structured selectors (`CaseMux` / `CasezMux` /
  `ForFold`) already have their own `always_comb` rendering and are **not**
  candidates; a `Slice` (a bit-select, no conditional) is not a candidate. The first
  cut scopes deliberately to the plain `Mux` — the simplest, highest-yield 2:1
  conditional.
- **Minus any gate already marked by a sibling projection.** A `Mux` is also a
  `function_emit` / `task_emit` candidate; the pass runs **last** and excludes any
  gate already claimed by one of the other six surfaces, so a gate is projected by
  **at most one** of the seven.
- **The net stays a net.** The gate's `<wire>` keeps its `wire`/`assign`; only the
  *source* of that assign changes (from the inline ternary to the `<wire>__cv`
  passthrough). The procedural var is the new thing, not the net — minimal blast
  radius, no downstream consumer rewrite.
- **Combinational only.** The `Mux` is combinational; its operand refs are leaves of
  the block. The block reads only the gate's direct operands and writes only its own
  `<wire>__cv` — exactly the inline ternary's read/write set, so there is no cycle
  risk.
- **No new IR node / no new computed truth.** The block is a pure emit-time
  projection of an existing `Mux`; the flat IR, validators, CSE keys, and the
  canonical module signature are untouched.

### How anvil proves it

The same two-mechanism proof as the prior surfaces:

- The [`num_emitted_mux_if_blocks`](agent-mcp.md) metric (a post-hoc count of
  `Module.mux_if_gates`) is surfaced in `--introspect` at schema `1.15`, so a sweep
  can confirm the surface fired.
- The repo-owned `tool_matrix --mux-if-gate` forces `mux_if_emit_prob = 1.0` over
  comb-only DUTs across all three construction strategies and requires the
  `saw_mux_if_emit` coverage fact — a genuinely-emitted procedural block (detected
  from the SV text's `<wire>__cv` token, distinct from the `<wire>__f(` / `<wire>__t(`
  / `<leader>__mt(` / `<root>__cf(` surfaces) accepted by Verilator **and** Yosys.
  Banked clean (3 scenarios / 12 modules / 12 emitting a block / 215 blocks /
  `coverage_gaps = []` / `12/0` Verilator + both Yosys + Icarus). Across that bank the
  `if`/`else` projection adds **zero** new Verilator `-Wall` warnings versus the
  knob-off build, and an `iverilog` simulation proves it bit-identical to the inline
  ternaries it replaces.
- Library tests pin the marking: a plain `Mux` qualifies, a non-`Mux` and a
  `Slice`-or-structured gate do not, a gate already marked by any of the six sibling
  projections is excluded, the prob-`0.0` path marks nothing (byte-identical), and a
  marked `Mux` emits the `<wire>__cv` `always_comb` `if`/`else` + passthrough while
  the unmarked default stays the inline ternary.

The picked-seventh rationale (the first procedural-conditional shape, chosen over
nested/multi-level `generate` — which has no routine by-construction source — and
`interface` / `modport` — empirically disqualified) is recorded in decision `0027`
(`docs/decisions/0027-structured-emission-seventh-surface-procedural-if-else.md`). The
N-way `CaseMux` → `if`/`else if` priority chain is the recorded follow-up.

### Reproducing it

<!-- book-test: skip — config-file edit + a forced-knob comb-only shape; not the default generator one-liner -->
```bash
anvil --seed 1 --dump-config > base.json
# edit base.json: set "mux_if_emit_prob": 1.0 (a small comb-only mux-heavy shape
# makes the blocks easy to read: "flop_prob": 0.0, "constant_prob": 0.0,
# "comb_mux_prob": 1.0, "comb_mux_encoding_prob": 1.0, "min_inputs": 3,
# "max_inputs": 3, "min_outputs": 1, "max_outputs": 1, "min_width": 4,
# "max_width": 4, "max_depth": 1, "min_mux_arms": 2, "max_mux_arms": 2)
anvil --seed 1 --config base.json
```

Flip `mux_if_emit_prob` back to `0.0` and the output is byte-identical to the default
lane.

## The eighth surface: a procedural `if`/`else if` priority chain

The eighth surface is the **first N-way procedural priority chain** in the lane, and a
direct sibling of the [seventh](#the-seventh-surface-a-procedural-ifelse). Where the
seventh re-expresses a single 2:1 `Mux`, the eighth re-expresses the N-way structured
selector `CaseMux`. A `CaseMux` with a **dynamic** selector renders today as a parallel
`always_comb case (sel) … default` statement; the eighth surface re-expresses that same
`always_comb` block's *body* as an `if`/`else if` **priority chain** of selector-equality
tests, falling through to the same `default` value.

Because the `case` labels are **distinct constants by construction** (arm `i` ⇒ label
`SW'd{i}`), at most one equality is ever true, so the priority chain and the parallel
`case` select the same arm for every selector value, and the trailing `else` covers exactly
the `case` `default` — it is **behaviour-preserving by construction**.

This surface is **simpler than the seventh**: a `CaseMux` is *already* declared as an
`always_comb`-written `logic` var, so it needs **no** `<wire>__cv` output var + passthrough.
Only the `always_comb` *body* swaps `case … endcase` → `if … else if`; the net, its width,
and every operand reference are untouched. It uses its **own** knob,
[`case_mux_if_emit_prob`](knobs.md#structured-emission) (default `0.0`), separate from
`mux_if_emit_prob`, so the shipped surfaces stay byte-identical. A new
[`num_emitted_case_mux_if_chains`](agent-mcp.md) metric counts the chains it emits, bumping
the introspection schema to `1.16`.

### Before and after

Here is a small combinational module with the knob **off** (the default). A 4-bit
`CaseMux` over a one-bit dynamic selector renders as a parallel `always_comb case`:

```systemverilog
    logic [3:0] case_mux_0;

    always_comb begin
        case (slice_0)
            1'd0: case_mux_0 = 4'h5;
            1'd1: case_mux_0 = 4'ha;
            default: case_mux_0 = 4'h0;
        endcase
    end
```

With `case_mux_if_emit_prob = 1.0`, the *same* `CaseMux` is re-expressed as an `if`/`else
if` priority chain writing the *same* `case_mux_0` var — the parallel `case`/`endcase` is
suppressed, the `default` becomes the trailing `else`, and there is **no** `<wire>__cv` var
(unlike the seventh surface) because `case_mux_0` is already an `always_comb` var:

```systemverilog
    logic [3:0] case_mux_0;

    always_comb begin
        if (slice_0 == 1'd0) case_mux_0 = 4'h5;
        else if (slice_0 == 1'd1) case_mux_0 = 4'ha;
        else case_mux_0 = 4'h0;
    end
```

Each arm tests its `case` label as a selector equality, in ascending arm order; the trailing
`else` carries the former `default`. Only the block's *body* changed — the net `case_mux_0`,
its declaration, and every downstream consumer are unchanged.

### What gets wrapped (and what doesn't)

- **The candidate is a dynamic-selector `CaseMux`** (a `GateOp::CaseMux` gate whose selector
  operand is *not* a constant, with at least one arm). A **constant-selector** `CaseMux` is
  statically collapsed by the emitter to a continuous `assign` of the selected arm — it
  never emits an `always_comb` block, so it is **not** a candidate (and excluding it keeps
  the chain count exact). A `CasezMux` (whose `casez ?` wildcards need a *masked* comparison,
  not a plain equality) is **not** a candidate — that masked priority chain is the recorded
  follow-up.
- **Minus any gate already marked by a sibling projection.** The pass runs **last** and
  excludes any gate already claimed by one of the seven other surfaces (vacuous in practice —
  no other pass marks a `CaseMux` — but kept for robustness), so a gate is projected by **at
  most one** of the eight.
- **The body swaps in place; the net stays a net.** Unlike the seventh surface there is no
  new declaration and no passthrough — the `CaseMux` is already an `always_comb`-written
  `logic` var, so only its block body changes form. Minimal blast radius.
- **Combinational only.** The chain reads only the selector + arm operand refs the `case`
  arm already reads and writes only the gate's own var — exactly the parallel `case`'s
  read/write set, so there is no cycle risk.
- **No new IR node / no new computed truth.** The chain is a pure emit-time projection of an
  existing `CaseMux`; the flat IR, validators, CSE keys, and the canonical module signature
  are untouched.

### How anvil proves it

The same two-mechanism proof as the prior surfaces:

- The [`num_emitted_case_mux_if_chains`](agent-mcp.md) metric (a post-hoc count of
  `Module.case_mux_if_gates`) is surfaced in `--introspect` at schema `1.16`, so a sweep can
  confirm the surface fired. It is **exact** — because constant-selector `CaseMux` is
  excluded, the count never over-reports.
- The repo-owned `tool_matrix --case-mux-if-gate` forces `case_mux_if_emit_prob = 1.0` over
  `case_mux_prob`-biased comb-only DUTs across all three construction strategies and requires
  the `saw_case_mux_if_emit` coverage fact. Detection here is **metric-keyed** rather than a
  text scan: this surface emits **no new identifier token** (only the `always_comb` body
  changes form), and an `if (… == …)` scan would also match FSM decode blocks, so the gate
  keys on the exact `num_emitted_case_mux_if_chains` metric instead. Banked clean (3 scenarios
  / 12 modules / 12 emitting a chain / 83 chains / `coverage_gaps = []` / `12/0` Verilator +
  both Yosys + Icarus). Across that bank the priority chain adds **zero** new Verilator
  `-Wall` warnings versus the knob-off parallel `case`.
- Library tests pin the marking: a dynamic-selector `CaseMux` qualifies, a constant-selector
  `CaseMux`, a plain `Mux`, and a `CasezMux` do not, a gate already marked by a sibling
  projection is excluded, the prob-`0.0` path marks nothing (byte-identical), and a marked
  `CaseMux` emits the `if`/`else if` chain while the unmarked default stays the parallel
  `case`.

The picked-eighth rationale (the first N-way procedural priority chain, the recorded
decision-`0027` follow-up, chosen over the `CasezMux` masked chain — which needs a masked
comparison — and nested/multi-level `generate` and `interface` / `modport`) is recorded in
decision `0028`
(`docs/decisions/0028-structured-emission-eighth-surface-case-mux-priority-chain.md`). The
`CasezMux` masked priority chain is the recorded follow-up.

### Reproducing it

<!-- book-test: skip — config-file edit + a forced-knob comb-only shape; not the default generator one-liner -->
```bash
anvil --seed 1 --dump-config > base.json
# edit base.json: set "case_mux_if_emit_prob": 1.0 (a small comb-only case-mux shape
# makes the chain easy to read: "flop_prob": 0.0, "constant_prob": 0.0,
# "comb_mux_prob": 0.0, "case_mux_prob": 1.0, "casez_mux_prob": 0.0, "min_inputs": 3,
# "max_inputs": 3, "min_outputs": 1, "max_outputs": 1, "min_width": 4, "max_width": 4,
# "max_depth": 1, "min_mux_arms": 2, "max_mux_arms": 2)
anvil --seed 1 --config base.json
```

Flip `case_mux_if_emit_prob` back to `0.0` and the output is byte-identical to the default
lane.

## The ninth surface: a masked `if`/`else if` priority chain

The ninth surface **generalizes the
[eighth](#the-eighth-surface-a-procedural-ifelse-if-priority-chain)** from the bare-equality
`CaseMux` to the **wildcard `CasezMux`**. Where the eighth re-expresses a `case (sel)` over
distinct constant labels, the ninth re-expresses a `casez (sel)` over `?`-wildcard patterns. A
`CasezMux` with a **dynamic** selector renders today as a parallel `always_comb casez (sel) …
default` statement; the ninth surface re-expresses that same block's *body* as an `if`/`else if`
**masked** priority chain — each arm a *masked* equality `(sel & care_mask) == value_masked`,
falling through to the same `default`.

The wildcard is what forces the mask: a `casez` arm `2'b0?` matches every selector whose high
bit is `0`, regardless of the low (`?`) bit. A plain equality cannot express that, so each arm
compares only its **care** bits: `care_mask = ~wildcard_mask` (the non-`?` bits) and
`value_masked = pattern & care_mask`. Because anvil builds `casez` patterns with exactly one
wildcard bit per arm and non-overlapping care patterns, at most one masked equality is ever true,
so the masked chain selects the same arm as the parallel `casez` and the trailing `else` covers
exactly the `default` — it is **behaviour-preserving by construction**.

Like the eighth surface it is **simpler than the seventh**: a `CasezMux` is *already* an
`always_comb`-written `logic` var, so it needs **no** `<wire>__cv` output var + passthrough. Only
the `always_comb` *body* swaps `casez … endcase` → masked `if … else if`. It uses its **own**
knob, [`casez_mux_if_emit_prob`](knobs.md#structured-emission) (default `0.0`), separate from
`case_mux_if_emit_prob`, so the shipped surfaces stay byte-identical. A new
[`num_emitted_casez_mux_if_chains`](agent-mcp.md) metric counts the chains it emits, bumping the
introspection schema to `1.17`.

### Before and after

Here is a small combinational module with the knob **off** (the default). A 4-bit `CasezMux` over
a two-bit dynamic selector renders as a parallel `always_comb casez`:

```systemverilog
    logic [3:0] casez_mux_0;

    always_comb begin
        casez (slice_0)
            2'b0?: casez_mux_0 = 4'hd;
            2'b1?: casez_mux_0 = 4'h9;
            default: casez_mux_0 = 4'h0;
        endcase
    end
```

With `casez_mux_if_emit_prob = 1.0`, the *same* `CasezMux` is re-expressed as a masked `if`/`else
if` priority chain writing the *same* `casez_mux_0` var — the parallel `casez`/`endcase` is
suppressed, the `default` becomes the trailing `else`, and there is **no** `<wire>__cv` var:

```systemverilog
    logic [3:0] casez_mux_0;

    always_comb begin
        if ((slice_0 & 2'h2) == 2'h0) casez_mux_0 = 4'hd;
        else if ((slice_0 & 2'h2) == 2'h2) casez_mux_0 = 4'h9;
        else casez_mux_0 = 4'h0;
    end
```

Each arm masks the selector to its **care** bits (here `2'h2` — the high bit; the low bit is the
`?` wildcard) and compares against the masked pattern (`2'h0` for `2'b0?`, `2'h2` for `2'b1?`), in
ascending arm order; the trailing `else` carries the former `default`. Only the block's *body*
changed — the net `casez_mux_0`, its declaration, and every downstream consumer are unchanged.

### What gets wrapped (and what doesn't)

- **The candidate is a dynamic-selector `CasezMux`** (a `GateOp::CasezMux` gate whose selector
  operand is *not* a constant, with at least one arm). A **constant-selector** `CasezMux` is
  statically collapsed by the emitter to a continuous `assign` — it never emits an `always_comb`
  block, so it is **not** a candidate (and excluding it keeps the chain count exact). The
  bare-equality `CaseMux` is owned by the
  [eighth surface](#the-eighth-surface-a-procedural-ifelse-if-priority-chain); it is **not**
  re-claimed here.
- **Minus any gate already marked by a sibling projection.** The pass runs **last** (after
  `case_mux_if` and the seven earlier projections), so a gate is projected by **at most one** of
  the nine surfaces.
- **The body swaps in place; the net stays a net.** Like the eighth surface there is no new
  declaration and no passthrough — the `CasezMux` is already an `always_comb`-written `logic`
  var, so only its block body changes form. Minimal blast radius.
- **Combinational only.** The masked chain reads only the selector + arm operand refs the `casez`
  arm already reads and writes only the gate's own var — exactly the parallel `casez`'s
  read/write set, so there is no cycle risk.
- **No new IR node / no new computed truth.** The masked chain is a pure emit-time projection of
  an existing `CasezMux`; the flat IR, validators, CSE keys, and the canonical module signature
  are untouched.

### How anvil proves it

The same two-mechanism proof as the prior surfaces:

- The [`num_emitted_casez_mux_if_chains`](agent-mcp.md) metric (a post-hoc count of
  `Module.casez_mux_if_gates`) is surfaced in `--introspect` at schema `1.17`, so a sweep can
  confirm the surface fired. It is **exact** — because constant-selector `CasezMux` is excluded,
  the count never over-reports.
- The repo-owned `tool_matrix --casez-mux-if-gate` forces `casez_mux_if_emit_prob = 1.0` over
  `casez_mux_prob`-biased comb-only DUTs across all three construction strategies and requires the
  `saw_casez_mux_if_emit` coverage fact. Detection here is **metric-keyed** rather than a text
  scan: this surface emits **no new identifier token** (only the `always_comb` body changes form),
  and an `if ((… & …) == …)` scan would also match the eighth surface's chain, so the gate keys on
  the exact `num_emitted_casez_mux_if_chains` metric instead. The focus config zeros **both**
  `comb_mux_prob` and `case_mux_prob` — both roll *before* `casez_mux` in the cone builder and
  would otherwise pre-empt it. Banked clean (3 scenarios / 12 modules / 12 emitting a chain / 108
  chains / `coverage_gaps = []` / `12/0` Verilator + both Yosys + Icarus). Across that bank the
  masked chain adds **zero** new Verilator `-Wall` warnings versus the knob-off parallel `casez`.
- Library tests pin the marking: a dynamic-selector `CasezMux` qualifies, a constant-selector
  `CasezMux`, a plain `Mux`, and a bare `CaseMux` do not, a gate already marked by a sibling
  projection is excluded, the prob-`0.0` path marks nothing (byte-identical), and a marked
  `CasezMux` emits the masked chain while the unmarked default stays the parallel `casez`.

The picked-ninth rationale (generalize the eighth's bare-equality chain to the wildcard
`CasezMux`, chosen over the concise `sel ==? pattern` wildcard-equality form — which Yosys 0.64
rejects in both repo modes — and nested/multi-level `generate` and `interface` / `modport`) is
recorded in decision `0029`
(`docs/decisions/0029-structured-emission-ninth-surface-casez-mux-masked-priority-chain.md`).
Nested/multi-level `generate` and `interface` / `modport` remain the recorded future surfaces.

### Reproducing it

<!-- book-test: skip — config-file edit + a forced-knob comb-only shape; not the default generator one-liner -->
```bash
anvil --seed 1 --dump-config > base.json
# edit base.json: set "casez_mux_if_emit_prob": 1.0 (a small comb-only casez-mux shape
# makes the masked chain easy to read: "flop_prob": 0.0, "constant_prob": 0.0,
# "comb_mux_prob": 0.0, "case_mux_prob": 0.0, "casez_mux_prob": 1.0, "min_inputs": 3,
# "max_inputs": 3, "min_outputs": 1, "max_outputs": 1, "min_width": 4, "max_width": 4,
# "max_depth": 1, "min_mux_arms": 2, "max_mux_arms": 2)
anvil --seed 1 --config base.json
```

Flip `casez_mux_if_emit_prob` back to `0.0` and the output is byte-identical to the default lane.

## Combining the surfaces

Every section above turns on **one** surface. That is how each was designed,
tested, and banked downstream-clean — and it is also how the whole lane was
exercised for its first nine increments. Combining them is a different question,
with a genuinely surprising answer.

### The surfaces are mutually exclusive, and that has a consequence

A gate is projected by **at most one** surface. The nine annotation passes run in
a fixed order

```text
soft_union → function_emit → generate_loop → task_emit → multi_output_task
           → cone_function → mux_if → case_mux_if → casez_mux_if
```

and each pass skips any gate an earlier pass already claimed. That exclusion is
what makes stacking nine overlapping projections sound: without it, two passes
could both claim a gate and the emitter would render it twice.

The consequence catches everyone the first time:

> **Under mutual exclusion with a fixed pass order, a probability is a
> *priority*, not an intensity.** Turning every knob up does not turn every
> surface up — it hands the whole gate graph to whichever pass runs first.

Set all eight non-version-gated knobs to `1.0` and `function_emit`, second in the
chain, claims **every** admissible gate. That set is a superset of
`generate_loop`'s `{N{x}}` replications, `task_emit`'s candidates,
`multi_output_task`'s members, `cone_function`'s roots and interiors, and
`mux_if`'s 2:1 muxes. Each later pass then correctly finds nothing left, and the
emitted module carries **three** shapes, not eight — only `case_mux_if` and
`casez_mux_if` survive alongside, because `CaseMux` and `CasezMux` are outside
`function_emit`'s admissible set in the first place.

This is not a bug in any surface. It is what mutual exclusion *means*. But it does
mean **maximum coverage and maximum diversity are opposite goals**, and diversity
— many distinct legal shapes in one module for a tool to trip over — is the one
worth having.

### The fix: an intermediate probability

Give every surface a *partial* claim and each pass leaves work for the next.
Around `0.25` all eight surfaces appear together in a single module, and the value
is chosen to maximise the **least-represented** surface's count rather than the
total. That is exactly what the `structured-emission-max` preset applies, together
with raised `comb_mux_prob` / `case_mux_prob` / `casez_mux_prob` so the three
procedural surfaces have candidate gates at all:

```bash
cargo run --release -- --seed 42 --count 12 \
  --profile structured-emission-max --out ./sem
grep -c "endfunction" ./sem/mod_42_0000.sv
grep -c "endtask" ./sem/mod_42_0000.sv
grep -c "endgenerate" ./sem/mod_42_0000.sv
```

Each of those counts is non-zero on the same module — a `function automatic`, a
`task automatic`, and a `generate for` loop coexisting in one file, alongside the
cone functions, multi-output tasks, and the three procedural conditional shapes
that do not mint their own keyword.

To see the per-surface counts directly, ask the introspection document:

```bash
cargo run --release -- --seed 42 --profile structured-emission-max --introspect \
  | grep num_emitted
```

### Why this is the interesting artifact

A module carrying a `function automatic`, a `generate for`, a multi-output `task`,
a whole-cone `function`, a procedural `if`/`else`, and two priority chains **at
once** is a materially harder parse, elaborate, and lower than the same constructs
spread across eight files. Downstream tools handle each shape in isolation every
day; they see them interleaved far less often, and that is where the interesting
rejections live.

Combining the surfaces is also the only way to exercise the mutual-exclusion
invariant end-to-end. With one knob at `1.0` and the rest at `0.0` — the shape of
all nine single-surface gates — every exclusion check sees an empty sibling set
and trivially passes. Only a combined run puts a real sibling set in front of it.

### How anvil proves it

The repo-owned `tool_matrix --emit-surface-interaction-gate` is the only gate that
runs the surfaces together. Five scenarios cover both extremes of the same
invariant:

- **three comb-only DUTs**, one per construction strategy, with all eight
  non-version-gated surfaces at the shared intermediate probability — each pass
  claims some gates and leaves others, so several surfaces co-occur in one module;
- **one saturation scenario** at `1.0`, where every later pass sees a *full* sibling
  set and must skip — the opposite extreme, and the configuration that demonstrates
  the collapse described above;
- **one Verilator-only IEEE 1800-2023 scenario** adding the ninth surface, kept
  separate so the other four keep their Yosys and Icarus columns.

Co-occurrence is proven **by projection, not by a new token**: the report already
carries nine per-module `emitted_*` booleans (each a text-token or metric-keyed
proof that the surface actually fired), so the gate simply counts them into a
derived `distinct_emit_surfaces` and requires
`saw_multi_surface_emit_interaction` (≥ 2 surfaces in one downstream-accepted
module), `saw_all_emit_surfaces_in_one_module` (≥ 8), and
`saw_all_nine_emit_surfaces_in_one_module`. The per-run maximum is recorded as
`max_distinct_emit_surfaces` so achieved strength is visible without making the
gate brittle.

Banked clean with a committed digest
(`docs/evidence/anvil-emit-surface-interaction-r1.md`): 5 scenarios / 20 modules /
`coverage_gaps = []` / Verilator 20/0 / Yosys 16/0 in both modes / Icarus 16/0,
the four `union soft` modules being the recorded Yosys/Icarus no-op. The measured
shape matched the prediction exactly — **8** distinct surfaces in every module of
the three universal scenarios, **9** in the 2023 scenario, and **exactly 3** under
saturation, those three being `function_emit`, `case_mux_if`, and `casez_mux_if`:
the one pass that runs first, plus the two whose targets it cannot claim.

The full reasoning, the pass-by-pass exclusion audit, the probability calibration,
and the downstream results are in decision `0032`
(`docs/decisions/0032-emit-surface-interaction-gate.md`, relative to the
repository root).

### The one version-gated surface stays separate

`soft_union_slice_prob` — the IEEE 1800-2023 `union soft` overlay — is *not* in
the preset. It is disjoint from the other eight by target type (it claims `Slice`
gates, which none of the others touch), so it adds no exclusion pressure; and
Yosys and Icarus reject the syntax, so enabling it costs two of the four
acceptance columns. Combine it explicitly when you want it:

```bash
cargo run --release -- --seed 42 --count 4 \
  --profile structured-emission-max \
  --sv-version 2023 --soft-union-slice-prob 1.0 --out ./sem2023
```

That corpus is Verilator-clean under `--language 1800-2023` and is a recorded
no-op for Yosys and Icarus (see
[the version target](knobs.md#systemverilog-version-target)).

## A different kind of thing: case qualifiers

Everything above is a **projection** — a pass that changes *how* a gate renders. The
`unique` / `priority` **case qualifiers** are the first construct in this chapter that
changes nothing about the rendering at all. They prefix a keyword onto the
`case` / `casez` statement a dynamic-selector `CaseMux` / `CasezMux` already emits:

```systemverilog
    always_comb begin
        unique case (sel)          // or: priority case (sel)
            2'd0: case_mux_0 = i_2;
            2'd1: case_mux_0 = i_1;
            default: case_mux_0 = 4'h0;
        endcase
    end
```

Strip the keyword and you get the default output **byte for byte**. That is the whole
construct — which is exactly why it is not a tenth surface, and why the
[mutual-exclusion](#the-surfaces-are-mutually-exclusive-and-that-has-a-consequence)
argument above does not apply to it. It claims no rendering, so it competes with nothing.

### A qualifier is an assertion, and that changes what "safe" means

The nine surfaces are judged by *does the downstream tool ingest this shape?* A
qualifier is judged by a harder question, because IEEE 1800-2017 §12.5.3 makes it a
**claim about the design** that a simulator checks at runtime and a synthesizer may act
on:

| qualifier | asserts |
| --- | --- |
| `priority` | **FULL** — some `case_item` matches (first-match semantics) |
| `unique` | **FULL and PARALLEL** — some `case_item` matches, and no two match |

Emitting a qualifier that could be **false** would manufacture precisely the
simulation/synthesis divergence ANVIL exists to *find in tools*, never to inject into its
own output. So the construct is only admissible because both properties are free from the
generator, with no analysis pass and no generate-then-filter:

- **FULL** holds because `emit/sv.rs` writes a `default:` arm for **every**
  `CaseMux`/`CasezMux` that renders as a statement. `default` is itself a `case_item`, so
  a match always exists. (This is also why the `default:` arm is *not* made conditional on
  the qualifier — it is the thing that makes the assertion true.)
- **PARALLEL** holds because a `CaseMux` labels arm `i` with the integer `i` — distinct by
  construction — and because `build_casez_patterns` is the **sole** `casez` pattern source
  in the generator and gives arm `i` the care-value `i` with exactly one don't-care bit, so
  two arms overlap only if two arm indices coincide, which they cannot.

Both halves are asserted over **real generated output** by a property test in
`src/ir/case_qualifier.rs`, each with its own firing negative control: delete the emitter's
`default:` arm and the FULL check reddens; widen the `casez` wildcard mask by one bit and
the PARALLEL check reddens. See [Correctness guarantees](by-construction.md) for why that
negative control is the load-bearing half.

### The one exclusion in this lane that actually bites

Every sibling exclusion in the nine surfaces is vacuous in practice — no two passes want the
same gate kind. This pass carries one that is real: a gate claimed by the
[eighth](#the-eighth-surface-a-procedural-ifelse-if-priority-chain) or
[ninth](#the-ninth-surface-a-masked-ifelse-if-priority-chain) surface renders as an
`if`/`else if` chain and has **no `case` keyword to prefix**. So the qualifier pass runs
**last**, after both, and skips exactly those gates.

That is why the repo-owned gate (below) carries a dedicated **co-occurrence** scenario. A
predicate that is written, unit-tested, and never fired end-to-end is the hole decision
`0032` was opened to close; leaving this one unexercised would reproduce it.

### Turning them on

Two knobs, both default `0.0`, each with its own `KnobId` in the **`qualifiers`** steering
category — deliberately *not* `emission`, which is the nine projections. The two families
are anti-correlated (a qualifier claims only the gates the projections declined), so
pooling them would make `--steer emission` average a self-cancelling mixture.

<!-- book-test: skip — a forced-knob comb-only shape, not the default generator one-liner -->
```bash
# every eligible case/casez statement gets `unique`
cargo run --release -- --seed 7 --count 8 \
  --flop-prob 0.0 --comb-mux-prob 0.0 --case-mux-prob 0.5 --casez-mux-prob 0.5 \
  --unique-case-prob 1.0 --out ./cq-unique

# ...or `priority`; swap the knob, nothing else changes
cargo run --release -- --seed 7 --count 8 \
  --flop-prob 0.0 --comb-mux-prob 0.0 --case-mux-prob 0.5 --casez-mux-prob 0.5 \
  --priority-case-prob 1.0 --out ./cq-priority
```

`unique` is rolled first, and a gate it claims is **not** rolled for `priority` — so each
knob's recorded fires equal its metric exactly, and
`num_emitted_unique_cases + num_emitted_priority_cases` is the total number of qualified
statements. With both knobs at `1.0`, `priority` therefore reports *no attempts at all*
rather than a fire rate of `0/0`.

### How anvil proves it

- The [`num_emitted_unique_cases` / `num_emitted_priority_cases`](agent-mcp.md) metrics are
  surfaced in `--introspect` at schema `1.28`. They are **exact**: the pass excludes
  statically-collapsed gates (a constant selector lowers to a continuous `assign`) and
  chain-projected gates, so every counted gate emits exactly one qualified statement.
- **Behaviour preservation is proven structurally, not statistically.** The sibling surfaces
  prove theirs with an ON-vs-OFF Verilator `-Wall` delta of zero — a statement about a
  warning count. A qualifier admits something stronger: run the same seed with the knob on
  and off, strip the keyword from the ON corpus with one `sed`, and `diff`.

  <!-- book-test: skip — writes two corpora and diffs them; the harness runs single commands -->
  ```bash
  # ...against the ./cq-unique corpus from above
  cargo run --release -- --seed 7 --count 8 \
    --flop-prob 0.0 --comb-mux-prob 0.0 --case-mux-prob 0.5 --casez-mux-prob 0.5 \
    --out ./cq-off
  mkdir -p ./cq-stripped
  for f in ./cq-unique/*.sv; do
    sed 's/unique case/case/; s/unique casez/casez/' "$f" > "./cq-stripped/$(basename "$f")"
  done
  diff -r ./cq-off ./cq-stripped --exclude=manifest.json && echo "byte-identical"
  ```

  Over those **8 modules and 176 qualified statements** the diff is empty, so nothing but
  the token moved. Where a delta-of-zero says *no new warnings appeared*, this says *there
  is nothing else to warn about*.
- The repo-owned `tool_matrix --case-qualifier-gate` runs **13** scenarios —
  `{unique, priority}` × `{case_mux-biased, casez_mux-biased}` × three construction
  strategies, plus the co-occurrence scenario — and requires three coverage facts:
  `saw_unique_case_qualifier`, `saw_priority_case_qualifier`, and
  `saw_case_qualifier_beside_if_chain`. Detection is **metric-keyed**, not a text scan.
  Banked clean: 52 modules, **359** qualified statements, `coverage_gaps = []`, Verilator
  `52/0` and both Yosys modes `52/0`.

### The one tool that says something, and what it means

The gate's tool plan is **per qualifier**, which no earlier gate needed:

| tool | `priority` | `unique` |
| --- | --- | --- |
| Verilator `--lint-only` | clean | clean |
| Yosys, both repo modes | clean, **identical cell counts** | clean, **identical cell counts** |
| Icarus `-g2012` | silent | exits `0`, prints `vvp.tgt sorry: Case unique/unique0 qualities are ignored.` per block |

The Icarus note is an **accepting no-op**, so `unique` scenarios record it rather than run
it. Two things are worth reading off this table. First, the reduction drops the Icarus
column *alone* — not the `union soft` reduction, which also drops Yosys — because Yosys is
this construct's strongest evidence: **identical synthesized cell counts** with and without
the qualifier are the synthesis-side statement that an assertion which holds gives the
synthesizer no new freedom. Second, `sorry:` does not contain the substring `warning:`, so
the shared warning classifier would have passed this corpus *by lexical accident*. The gate
splits the tool plan instead of teaching a shared classifier about one construct.

Full rationale — including why `unique0` is deferred and why the `default:` arm is never
dropped — is in decision `0044`
(`docs/decisions/0044-capability-breadth-unique-priority-case-qualifiers.md`).

## Measuring the surfaces: per-gate fire rates

Every knob above rolls **once per candidate gate**, and since
`COVERAGE-STEERED-GENERATION.4b.2` each of those rolls is recorded. That makes
the nine surfaces the highest-resolution part of ANVIL's
[coverage readout](knobs.md#per-knob-roll-rate-validation): a single module
contributes thousands of attempts, so the achieved rate is a real measurement
rather than a small-sample estimate.

<!-- book-test: skip — reads the readout with jq/python, which the harness does not shim -->
```bash
cargo run --release -- --seed 7 --profile structured-emission-max --introspect
```

Read `introspection.coverage_readout.knob_fire_rates`. On seed 7 it reports:

| surface | fires/attempts | achieved rate |
| --- | --- | --- |
| `function_emit_prob` | 881/3420 | `0.258` |
| `task_emit_prob` | 626/2463 | `0.254` |
| `multi_output_task_emit_prob` | 401/1631 | `0.246` |
| `case_mux_if_emit_prob` | 120/425 | `0.282` |
| `casez_mux_if_emit_prob` | 77/285 | `0.270` |
| `cone_function_emit_prob` | 78/305 | `0.256` |
| `generate_loop_emit_prob` | 76/325 | `0.234` |
| `mux_if_emit_prob` | 61/253 | `0.241` |
| **category `emission`** | **2320/9107** | **`0.255`** |

Every rate lands on `0.25` — the preset's value, and the one the
[Combining the surfaces](#combining-the-surfaces) section derived by sweeping
per-surface counts externally. It is now visible **from the artifact itself**,
which is what lets an agent close the loop: measure the achieved rates, notice a
surface is under-represented, and steer toward it.

Steering the whole family is one key, because all nine share the `emission`
category:

<!-- book-test: skip — comparative measurement; the runnable form is the plain --introspect above -->
```bash
# halve the emission family's achieved rate without touching any other knob
cargo run --release -- --seed 7 --profile structured-emission-max \
  --steer emission=0.5 --introspect
```

Two cautions worth internalising before you steer:

- **A raised probability is not more surfaces.** Mutual exclusion plus the fixed
  pass order means an *earlier* surface claiming more gates leaves *later* ones
  with fewer candidates. Steering `emission` up moves every surface's rate up,
  but it moves the lane back toward the saturation regime where `function_emit`
  claims nearly everything and three shapes survive. If you want diversity,
  steer *individual* under-represented surfaces, not the category.
- **`soft_union_slice_prob` is in the category but version-gated.** It only
  emits under `--sv-version 2023`, so an `emission` steer will move its roll
  rate even in a run where the emitter down-gates every marked slice back to a
  plain bit-select.
