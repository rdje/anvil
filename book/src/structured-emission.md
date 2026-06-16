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
construction in a richer SystemVerilog surface — today a `function`, a
`generate for` loop, and a `task`, and later nested `generate` or an
`interface` — so the tools have more legal structural variety to ingest,
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

It is governed by one config-file knob,
[`function_emit_prob`](knobs.md#structured-emission) (default `0.0`),
so with the knob off the output is byte-identical and nothing in the
default lane changes.

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

The knob is a config-file knob (no CLI flag), so set it through a
`--config` JSON. The example above comes from this recipe:

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
governed by one config-file knob,
[`generate_loop_emit_prob`](knobs.md#structured-emission) (default
`0.0`), so with the knob off the output is byte-identical.

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
operands that are all the **same** signal — **and** the replicated lane
`x` is exactly **1 bit** wide. With a 1-bit lane the result width is
exactly `N`, so the loop body `assign <wire>[gi] = x;` is bit-faithful.

A *wider* lane (say `{4{byte}}` where `byte` is 8 bits) is still
index-regular but would need a part-select body
(`<wire>[gi*8 +: 8] = byte`); that is a recorded follow-up. Until then a
wider replication stays inline — **nothing is retired**. The
`generate for` and `function automatic` projections are also mutually
exclusive on a gate (a replication marked for one is never also marked
for the other).

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

It is governed by one config-file knob,
[`task_emit_prob`](knobs.md#structured-emission) (default `0.0`), so with
the knob off the output is byte-identical.

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
