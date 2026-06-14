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
- `always_comb` for dynamic procedural case/casez mux blocks
  (`case_mux_prob`, `casez_mux_prob`) and dynamic bounded procedural
  `for`-fold blocks (`for_fold_prob`); no latch-y partial-assignment
  form.
- Continuous `assign` lowering for structured case/casez/for-fold
  gates whose controlling source is already constant. That keeps the
  emitted value identical while avoiding empty-sensitivity
  `always_comb` warnings in strict frontends such as Icarus Verilog.
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

## Memories (delivered, advanced motif)

Memories are a delivered Phase 6 motif behind the opt-in
`memory_prob` knob (default `0.0` → byte-identical output). They
follow inferrable patterns only:

```systemverilog
reg [W-1:0] mem [0:DEPTH-1];
always_ff @(posedge clk) begin
    if (we) mem[addr] <= wdata;
    rdata <= mem[addr];
end
```

The generator templates these from a first-class `Memory` block
whose registered read enters the gate graph only through an opaque
`Node::MemRead` leaf; it does not construct them from arbitrary
combinational logic. Yosys infers the emitted template as
`$mem_v2` (single-port, or simple-dual-port with an independent
read port). The stored contents are not reset-defined, so each
memory stays state-by-instance under the factorization passes.
See [Knobs and Reproducibility](knobs.md) (`memory_prob`) and
[The Circuit IR](ir.md).

## Latches

Latches are not synthesized by accident. The cone-recursion never
produces `always_comb` blocks with conditional assignments that leave
some signals unassigned: dynamic procedural case/casez blocks always
emit an explicit `default` assignment, static structured blocks lower
to continuous `assign`, and `assign` statements always fully define
their target. No latch can be inferred from `anvil` output.

## Sanity check

Despite all of this, the generator's "only-synthesizable-by-design"
promise is a claim, not a proof. The quality bar is not "some sample
passes sometimes"; the intended default is that generated modules are
clean in Verilator, Yosys, and any optional downstream column the
matrix enables. The project-level safety net is the evidence plan for
that claim:

> Periodically run representative `anvil` output through Verilator and
> Yosys, optionally through Icarus Verilog compile/elaboration, assert
> that lint / elaboration / synthesis complete cleanly, and treat every
> failure as a generator bug.

Any failure is a generator bug, filed with the seed for reproduction.

## Cross-simulator semantic agreement

The `tool_matrix` parse/synth columns prove every emitted artifact
is *accepted* by Verilator and Yosys. `--iverilog-compile` adds a
third optional acceptance column: each emitted module/design is
compiled with `iverilog -g2012`, warnings included as failures. This
does not run a testbench.

<!-- book-test: skip — opt-in column requires Icarus Verilog on PATH; documented in the verification log of SIGNOFF-SURFACE-EXPANSION.3 -->
```bash
# Add Icarus compile/elaboration acceptance to a matrix run
cargo run --bin tool_matrix -- --out ./tool-matrix --iverilog-compile

# Exercise Verilator, both Yosys modes, and Icarus together
cargo run --bin tool_matrix -- --out ./tool-matrix --yosys-mode both --iverilog-compile
```

The `--diff-sim` column raises the bar further to **semantic
equivalence** across two independent simulators — iverilog
(interpreted, 4-state, event-driven) and verilator (compiled,
2-state-default, cycle-driven). The two engines are deliberately
chosen for engine independence; agreement between them is strong
evidence that the emitted SV has a single intended meaning rather than
tool-specific behavior.

<!-- book-test: skip — opt-in column requires iverilog + verilator on PATH; runtime is multi-minute even on the per-axis subset; documented in the verification log of DIFFERENTIAL-SIMULATION.3b.2 -->
```bash
# Add the diff-sim column to a tool_matrix run
cargo run --bin tool_matrix -- --diff-sim --out ./tool-matrix
```

A per-axis subset selector picks the first scenario per major axis
(combinational, sequential-flop, hierarchy, memory, fsm), capped at
K=5, deterministic. The matrix selects the subset once at startup,
persists the names to `<out>/.diff-sim-subset`, and runs the
column on each selected scenario AFTER Verilator and Yosys are
both clean on the module — there is no point asking simulators to
agree on output a parse/synth tool already rejected.

For each module in the subset the harness:

1. Emits a generic SystemVerilog testbench from the parsed port
   section of the already-emitted DUT (whitespace-robust strict
   subset — handles aggregate ports as "skip diff-sim for this
   module").
2. Drives a deterministic baked stimulus (canonical edge cases —
   all-zeros, all-ones, walking-1 — then seeded `ChaCha8`
   pseudo-random). `$random` is intentionally *not* used:
   iverilog and verilator have different `$random` streams, which
   would inject false mismatches.
3. Holds reset, deasserts at a known negedge, runs a fixed
   warmup, then samples outputs at a single canonical post-reset
   cycle offset (combinational: `#1` settle + sample; sequential:
   `@(negedge clk)` drive → `@(posedge clk)` latch →
   `@(negedge clk)` sample). The post-reset canonical sample
   neutralises iverilog's pre-reset 4-state (`x`) vs Verilator's
   2-state-default (`0`) divergence.
4. Shells `iverilog -g2012 + vvp` and `verilator --binary` on
   the *same* testbench file, normalizes the fixed-width-hex
   traces, and byte-compares.

The per-module outcome is recorded under
`ModuleReport.diff_sim`:

```json
{
  "ran": true,
  "success": true,
  "n_samples": 8,
  "skip_reason": "",
  "mismatch_excerpt": null
}
```

On mismatch, `mismatch_excerpt` retains the first 10 lines from
each side side-by-side — a **retained counterexample** per the
Phase-7 doctrine; never a silent pass.

The `saw_design_with_cross_simulator_agreement` coverage fact
fires when at least one DUT in the subset achieves byte-equal
post-reset traces. The column is a **friendly no-op when either
simulator is absent** (`tools_present()` probe → `ran: false`
with a clear skip reason); the matrix still exits clean. To gate
on the fact, combine `--diff-sim` with `--fail-on-coverage-gap`.

The contract: `cargo run --bin tool_matrix -- --diff-sim` on a
machine with iverilog and verilator installed should record at
least one DUT with `diff_sim = { ran: true, success: true }`.
Any disagreement is either a generator bug (file with seed) or a
real downstream-tool bug — both are valuable signal per the
project's north star (surface downstream-tool bugs via
valid-by-construction + downstream-acceptance-quality output).
