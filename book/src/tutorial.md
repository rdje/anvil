# Tutorial

A progression of small, self-contained examples. Each one introduces
one new knob or motif, shows the exact command, and walks through
what appears in the generated SV.

> **Before you start:** `anvil`'s generated logic is deliberately
> nonsensical in function. The goal is structurally valid random
> RTL, not meaningful circuits. A gate doing `a + a + a` is expected
> — it tests the tooling, not your design intent.

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
intermediate `w_N` wires and a visible tree of `assign` statements
before `o_0` settles on its driver.

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
recursive cone — no mux. Excerpt of the emitted SV:

```systemverilog
    logic [9:0] r_0;

    wire [9:0] w_3;
    wire [9:0] w_4;

    assign w_3 = i_0[9:0];
    assign w_4 = w_3 + w_3;

    always_ff @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            r_0 <= 10'h3ff;
        end else begin
            r_0 <= w_4;
        end
    end

    assign o_0 = r_0;
```

Things to notice:

- `clk` and `rst_n` appear in the port list only when at least one
  flop exists.
- The `always_ff @(posedge clk or negedge rst_n)` header is
  canonical. `anvil` only ever emits this shape (one clock, async
  active-low reset).
- The flop is reset to `10'h3ff` (all-ones). Reset values are
  randomized per flop with bias toward 0.

## Example 5 — One-hot mux on D

Force a multi-arm mux, with the one-hot encoding style:

```bash
cargo run --release -- --seed 1 --max-depth 1 --max-inputs 2 --max-outputs 1 \
                      --flop-prob 1.0 --max-flops-per-module 2 \
                      --min-mux-arms 2 --max-mux-arms 2 \
                      --flop-mux-encoding-prob 0.0
```

When a flop draws `M = 2` with the one-hot style, its D input is
built as `D = ({W{sel_0}} & data_0) | ({W{sel_1}} & data_1)`. You'll
see patterns like:

```systemverilog
    assign w_11 = {r_4, r_4, r_4, r_4, r_4, r_4, r_4, r_4};  // {W{sel_0}}
    assign w_12 = w_11 & r_3;                                // mask_0 & data_0
    assign w_13 = {r_6, r_6, r_6, r_6, r_6, r_6, r_6, r_6};  // {W{sel_1}}
    assign w_14 = w_13 & r_5;                                // mask_1 & data_1
    assign w_17 = w_12 | w_14;                               // OR-combine
```

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
    assign w_11 = w_4 == 1'h1;         // sel == 1?
    assign w_12 = (w_11) ? (w_8) : (32'h0);
    assign w_14 = w_4 == 1'h0;         // sel == 0?
    assign w_15 = (w_14) ? (w_7) : (w_12);
    // ...
    always_ff @(posedge clk or negedge rst_n) begin
        ...
        else begin
            r_0 <= w_15;
        end
    end
```

Read from the bottom up: `r_0 <= w_15`, which is `sel == 0 ? data_0
: (sel == 1 ? data_1 : 0)`. The final `0` is the fall-through for
out-of-range select values (relevant when M is not a power of 2).

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
is now a DAG, not a tree. You'll see the same `w_N` appear as an
operand of several later `assign` statements.

The distinction matters:

- **Low `share_prob`** → wide, sprawling trees; each intermediate
  wire used once; stresses synthesis on large cones.
- **High `share_prob`** → tight DAGs with realistic fanout; stresses
  synthesis on common-subexpression elimination.

## Example 9 — Mixing everything

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
