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
construction in a richer SystemVerilog surface — a `function`, and
later a `task`, nested `generate`, or an `interface` — so the tools
have more legal structural variety to ingest, and more places to trip
over a real bug. (That bug-surfacing purpose is the
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
