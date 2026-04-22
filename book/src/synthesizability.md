# Synthesizability as a Subset Constraint

Synthesizability is a stricter condition than semantic validity. An SV
program can be semantically legal and still rejected by synthesis —
e.g., `initial` blocks, delays, `$display`, dynamic arrays, non-static
loops, unsynthesizable memory patterns.

The critical design choice in `anvil`: **synthesizability is enforced
by grammar restriction, not by post-hoc filtering.**

This chapter describes the **current primary synthesizable RTL lane**.
Future artifact families may broaden the emitted source forms, but the
user has explicitly re-affirmed that they are still meant to be
valid-by-construction and synthesizable.

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
- `always_comb` for the landed procedural case/casez mux blocks
  (`case_mux_prob`, `casez_mux_prob`); no latch-y partial-assignment
  form.
- No `always @*`.
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

## Memories (future, advanced motifs)

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
some signals unassigned: the procedural case-mux block always emits an
explicit `default` assignment, and `assign` statements always fully
define their target. No latch can be inferred from `anvil` output.

## Sanity check

Despite all of this, the generator's "only-synthesizable-by-design"
promise is a claim, not a proof. The quality bar is not "some sample
passes sometimes"; the intended default is that generated modules are
clean in Verilator and Yosys. The project-level safety net is the
evidence plan for that claim:

> Periodically run representative `anvil` output through Verilator and
> Yosys, assert that lint / elaboration / synthesis complete cleanly,
> and treat every failure as a generator bug.

Any failure is a generator bug, filed with the seed for reproduction.
