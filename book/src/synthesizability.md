# Synthesizability as a Subset Constraint

Synthesizability is a stricter condition than semantic validity. An SV
program can be semantically legal and still rejected by synthesis —
e.g., `initial` blocks, delays, `$display`, dynamic arrays, non-static
loops, unsynthesizable memory patterns.

The critical design choice in `anvil`: **synthesizability is enforced
by grammar restriction, not by post-hoc filtering.**

## How

The `GateOp` enum lists only synthesizable operators. There is no
`GateOp::InitialBlock` variant. There is no way to emit `#5` delays
because the emitter has no code path that produces delays. The emitter
emits:

- `module ... endmodule`
- `input`, `output` port declarations
- `wire`, `logic` internal signal declarations
- `assign` for combinational drives
- `always_ff @(posedge clk [or negedge rst_n])` blocks for flops
- No `always @*`; `always_comb` only if ever needed (Phase 4+).
- No `initial`. No `final`. No `fork`/`join`. No `wait`. No `#delay`.
- No `$display`, `$monitor`, `$finish`, `$stop`, or similar.
- No `real`, `time`, `event`, `class`, `queue`, dynamic arrays.
- No tasks or functions with side effects; only pure `function` if ever
  used (Phase 4+).

Because these are absent from the IR and the emitter, they cannot
appear in output.

## The flop pattern

Exactly one canonical flop template:

```systemverilog
always_ff @(posedge clk or negedge rst_n) begin
    if (!rst_n)
        flop_0 <= <reset_val>;
    else
        flop_0 <= <d_signal>;
end
```

Every flop in a module shares this single block with `clk` and
`rst_n` — one async-active-low reset, posedge clock, per
[Rule 5 (Single-clock / single-reset discipline)](structural-rules.md).
No sync-reset or no-reset variants are generated; per-flop clock or
reset polarity doesn't exist in the IR. No other `always_ff` shapes
are emitted.

## Memories (future, Phase 7)

When memories are added, they follow inferrable patterns only:

```systemverilog
reg [W-1:0] mem [0:DEPTH-1];
always_ff @(posedge clk) begin
    if (we) mem[addr] <= wdata;
    rdata <= mem[addr];
end
```

The generator templates these; it does not construct them from
arbitrary combinational logic.

## Latches

Latches are not synthesized by accident. The cone-recursion never
produces `always_comb` blocks with conditional assignments that leave
some signals unassigned. `assign` statements always fully define their
target. No latch can be inferred from `anvil` output.

## Sanity check

Despite all of this, the generator's "only-synthesizable-by-design"
promise is a claim, not a proof. The project-level safety net is:

> Periodically run a sample of `anvil` output through Yosys
> (`synth -top <name>; stat`) and assert that synthesis completes
> and produces a non-empty netlist.

Any failure is a generator bug, filed with the seed for reproduction.
