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
construction in a richer SystemVerilog surface — today a single-gate
`function`, a `generate for` loop, a `task`, a whole-cone `function`, and a
multi-output `task`, and later nested `generate` or an `interface` — so the
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
documented in `USER_GUIDE.md` and `README.md`.

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
