# Tutorial

A progression of small, self-contained examples. Each one introduces
one new knob or motif, shows the exact command, and walks through
what appears in the generated SV.

> **Before you start:** `anvil`'s generated logic is deliberately
> nonsensical in function. The goal is structurally valid random
> RTL, not meaningful circuits. A gate doing `a + a + a` is expected
> — it tests the tooling, not your design intent.

**Naming convention you'll see throughout:** every internal gate is
named `<kind>_<N>` where `<kind>` is the lowercase operator name
(`and`, `or`, `xor`, `not`, `add`, `sub`, `mul`, `eq`, `neq`, `lt`,
`gt`, `le`, `ge`, `mux`, `slice`, `concat`, `red_and`, `red_or`,
`red_xor`, `shl`, `shr`) and `<N>` counts per-kind from 0 within the
module. Flops are `flop_<id>`. So `and_5` is the sixth `And` gate in
the module, `flop_2` is the third flop. See
[Structural Rules — Rule 12](structural-rules.md) for the full naming
contract.

## Example 1 — The smallest useful module

```bash
cargo run --release -- \
    --seed 1 --max-depth 1 --max-inputs 1 --max-outputs 1 \
    --flop-prob 0 --share-prob 0
```

With `max-depth 1`, the cone has essentially no room to grow; the
output either passes an input through directly or through a single
gate.

This is the absolute minimum: one data input, one output, no flops,
no sharing. If this works end-to-end (builds, passes Verilator lint),
the rest of the tutorial will too.

## Example 2 — Deeper cones

```bash
cargo run --release -- --seed 42 --max-depth 4 --max-inputs 2 --max-outputs 1
```

Raising `--max-depth` gives the recursion more room. Expect more
intermediate `<kind>_N` wires and a visible tree of `assign`
statements before `o_0` settles on its driver.

## Example 3 — A combinational module with multiple outputs

```bash
cargo run --release -- --seed 3 --max-depth 3 --max-inputs 3 --max-outputs 3
```

Each output gets its own fanin cone, independently generated. The
outputs may share inputs (they always can) but will mostly have
independent intermediate logic unless `--share-prob` is raised.

## Example 4 — Enabling flops (direct D, no mux)

```bash
cargo run --release -- --seed 5 --max-depth 1 --max-inputs 2 --max-outputs 1 \
                      --flop-prob 1.0 --max-flops-per-module 1 \
                      --min-mux-arms 2 --max-mux-arms 2 \
                      --flop-qfeedback-prob 0
```

With `--flop-prob 1.0` every non-leaf recursion point becomes a flop.
The `--max-flops-per-module 1` cap prevents runaway. When the flop
draws `M = 0` from `pick_mux_arm_count`, its D input is a direct
recursive cone — no mux. Emitted SV (verbatim):

```systemverilog
    logic [9:0] flop_0;

    wire [9:0] shl_0;

    assign shl_0 = flop_0 << 1'h0;

    always_ff @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            flop_0 <= 10'h3ff;
        end else begin
            flop_0 <= shl_0;
        end
    end

    assign o_0 = flop_0;
```

Things to notice:

- `clk` and `rst_n` appear in the port list only when at least one
  flop exists.
- The `always_ff @(posedge clk or negedge rst_n)` header is
  canonical. `anvil` only ever emits this shape (one clock, async
  active-low reset).
- The flop is reset to `10'h3ff` (all-ones). Reset values are
  randomized per flop with bias toward 0.
- The D input `flop_0 << 1'h0` is a shift-by-zero — structurally
  valid but semantically a no-op. That's `anvil`'s "structural, not
  meaningful" promise in action.

## Example 5 — One-hot mux on D

Force a multi-arm mux, with the one-hot encoding style:

```bash
cargo run --release -- --seed 1 --max-depth 1 --max-inputs 2 --max-outputs 1 \
                      --flop-prob 1.0 --max-flops-per-module 2 \
                      --min-mux-arms 2 --max-mux-arms 2 \
                      --flop-mux-encoding-prob 0.0
```

When a flop draws `M = 2` with the one-hot style, its D input is
built as `D = ({W{sel_0}} & data_0) | ({W{sel_1}} & data_1)`.
Illustrative lines from the emitted SV (the actual module contains
more gates due to cross-arm sharing and the Q-feedback structure):

```systemverilog
    assign slice_0  = i_0[0:0];          // sel_0 (1-bit)
    assign concat_0 = {8{slice_0}};      // {W{sel_0}}
    assign and_1    = flop_1 & concat_0; // data_0 & mask_0
    assign not_0    = ~slice_0;          // sel_1 = ~sel_0 (toggle)
    assign concat_1 = {8{not_0}};        // {W{sel_1}}
    assign and_3    = flop_0 & concat_1; // data_1 & mask_1
    assign or_0     = and_1 | and_3;     // OR-combine
```

(Note: the `{8{slice_0}}` form is SystemVerilog's replication syntax;
`anvil` emits it whenever a `Concat` has all operands identical.)

This is the canonical one-hot shape. The design contract is that at
most one select bit fires at a time. `anvil` does **not** enforce
one-hot at runtime; it assumes the design convention. If the selects
happen to both fire, the outputs OR together — which is what the
gates actually compute.

## Example 6 — Encoded-select mux on D

Same setup, but flip the encoding probability to the encoded style:

```bash
cargo run --release -- --seed 11 --max-depth 1 --max-inputs 2 --max-outputs 1 \
                      --flop-prob 1.0 --max-flops-per-module 1 \
                      --min-mux-arms 2 --max-mux-arms 2 \
                      --flop-mux-encoding-prob 1.0 --flop-qfeedback-prob 0.0
```

Now the mux uses a single `ceil(log2(M))`-bit select bus with an
internal decoder, expressed as a chained ternary:

```systemverilog
    assign slice_0 = flop_0[0:0];             // sel (1-bit for M=2)
    assign eq_0    = slice_0 == 1'h1;         // sel == 1?
    assign mux_0   = (eq_0) ? (flop_0) : (32'h0);
    assign eq_1    = slice_0 == 1'h0;         // sel == 0?
    assign mux_1   = (eq_1) ? (flop_0) : (mux_0);

    always_ff @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            flop_0 <= 32'h0;
        end else begin
            flop_0 <= mux_1;
        end
    end
```

Read from the bottom up: `flop_0 <= mux_1`, which is
`sel == 0 ? data_0 : (sel == 1 ? data_1 : 0)`. The final `0` is the
fall-through for out-of-range select values (relevant when M is not
a power of 2). In this minimal seed the "data" operand happens to
be `flop_0` itself — that's the CSE + limited-pool combination
collapsing to the only available signal; with more inputs you'd see
distinct data references.

## Example 7 — Q-feedback flavor

Replace `--flop-qfeedback-prob 0.0` with `1.0`:

```bash
cargo run --release -- --seed 11 --max-depth 1 --max-inputs 2 --max-outputs 1 \
                      --flop-prob 1.0 --max-flops-per-module 1 \
                      --min-mux-arms 2 --max-mux-arms 2 \
                      --flop-mux-encoding-prob 1.0 --flop-qfeedback-prob 1.0
```

`QFeedback` changes what happens when no select fires: instead of
forcing D = 0, the flop holds — D = Q. In the encoded style, index 0
is routed from Q (not from a recursive cone). In the one-hot style,
an additional `~(OR of sels) & Q` term is OR'd in.

See [Sequential logic](sequential.md) for the full 2×2 matrix.

## Example 8 — DAG-shaped cones (signal sharing)

Flip sharing on hard:

```bash
cargo run --release -- --seed 42 --max-depth 4 --max-inputs 3 --max-outputs 2 \
                      --flop-prob 0 --share-prob 0.8
```

With high `--share-prob`, each operand has an 80% chance of
terminating at an existing pool entry instead of recursing to create
fresh logic. Internal wires acquire multiple consumers — the module
is now a DAG, not a tree. You'll see the same `<kind>_N` appear as
an operand of several later `assign` statements.

The distinction matters:

- **Low `share_prob`** → wide, sprawling trees; each intermediate
  wire used once; stresses synthesis on large cones.
- **High `share_prob`** → tight DAGs with realistic fanout; stresses
  synthesis on common-subexpression elimination.

## Example 9 — Combinational M-to-1 mux block

```bash
cargo run --release -- --seed 3 --max-depth 2 --max-inputs 3 --max-outputs 1 \
                      --flop-prob 0 --share-prob 0 \
                      --comb-mux-prob 1.0 --comb-mux-encoding-prob 1.0 \
                      --min-mux-arms 2 --max-mux-arms 3
```

Forcing `--comb-mux-prob 1.0` turns every non-leaf recursion point
into a combinational mux. With `--comb-mux-encoding-prob 1.0` the
style is Encoded (chained ternary over equality checks). Excerpt:

```systemverilog
    assign slice_0 = i_0[1:0];                 // sel (2-bit for M=3)
    assign slice_1 = i_0[0:0];                 // a data operand
    assign eq_0    = slice_0 == 2'h2;          // sel == 2?
    assign mux_0   = (eq_0) ? (slice_1) : (1'h0);
    assign eq_1    = slice_0 == 2'h1;          // sel == 1?
    assign mux_1   = (eq_1) ? (slice_1) : (mux_0);
    assign eq_2    = slice_0 == 2'h0;          // sel == 0?
    assign mux_2   = (eq_2) ? (slice_1) : (mux_1);
```

Read from the bottom: `mux_2` is the 3-to-1 chained ternary result.
The constant `1'h0` at the deepest `mux_0` is the out-of-range
fall-through.

Swap `--comb-mux-encoding-prob 0.0` and the same module emits the
OneHot shape instead: `{W{sel_i}} & data_i` terms OR'd together, no
chained ternaries.

Combinational muxes have no Q-feedback — the fall-through is always
`0` (visible as `20'h0` above). The flop D-mux path is where
Q-feedback lives; see [Sequential Logic](sequential.md).

## Example 10 — Mixing everything

```bash
cargo run --release -- \
    --seed 42 --max-depth 6 \
    --min-inputs 3 --max-inputs 6 \
    --min-outputs 2 --max-outputs 4 \
    --min-width 4 --max-width 16 \
    --flop-prob 0.15 --max-flops-per-module 8 \
    --min-mux-arms 2 --max-mux-arms 4 \
    --flop-qfeedback-prob 0.5 --flop-mux-encoding-prob 0.5 \
    --share-prob 0.3
```

Those are close to the default knobs. Expect:

- 3–6 data inputs, 2–4 outputs, widths 4–16 bits.
- Several flops, a mix of M=0 / M=2 / M=3 / M=4.
- A mix of one-hot and encoded mux styles.
- A mix of ZeroDefault and QFeedback kinds.
- DAG fanout throughout.

This is what a typical `anvil` invocation produces. Vary seeds to
get different topologies while keeping the overall shape.

## Where to go next

- **[Recipes](recipes.md)** — common scenarios and the knob
  combinations that produce them.
- **[Knobs reference](knobs.md)** — full parameter catalog.
- **[The Fanin Cone Algorithm](algorithm.md)** — pseudocode of
  exactly what `build_cone` does, if you want to read the source.
