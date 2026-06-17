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

`--sv2v` adds a fourth optional acceptance column: each emitted
module/design is transpiled to Verilog-2005 with `sv2v` — an
*independent* SystemVerilog front-end, so it trips bugs the other
three cannot. A clean transpile accepts; a non-zero exit or a warning
is a finding. Like `--iverilog-compile` it is an acceptance gate, not
a behavioural testbench (the transpiled output is discarded). `sv2v`
is the first new entry in ANVIL's closed
[downstream-adapter registry](./agent-mcp.md), so it is also selectable
over the MCP/CLI `tools` arg. It is absent on most hosts; when so the
column is a **friendly no-op** (a presence probe means a
requested-but-missing `sv2v` records no column and never fails the
run — `brew install sv2v` to light it up).

<!-- book-test: skip — opt-in column requires sv2v on PATH; documented in the verification log of DOWNSTREAM-ADAPTER-EXPANSION.2b.2 -->
```bash
# Add the sv2v transpile-acceptance column to a matrix run
cargo run --bin tool_matrix -- --out ./tool-matrix --sv2v
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

## Acceptance divergence across tools

`--diff-sim` asks *do two simulators compute the same values?* — a
question about **behaviour**, and only after both tools already
**accept** the artifact. The complementary, earlier-in-the-pipeline
question is *do two tools even agree the artifact is legal?* When one
tool accepts an artifact and another **warns or rejects** it, that is
an **acceptance divergence**. On RTL that is legal by construction
every such disagreement is a real downstream-tool bug — exactly the
north star — because the RTL cannot be at fault.

The acceptance-divergence detector lives in one shared place
(`divergence::run`, decision
[`0019`](https://github.com/rdje/anvil/blob/main/docs/decisions/0019-acceptance-divergence-hunting.md))
and is **default-off** everywhere it is surfaced — it changes no
emitted RTL. It does not run a generator or a behavioural oracle of
its own: it **composes** the same hardened `validate` orchestration the
parse/synth columns already use, projects each tool's run into an
`accept` / `warn` / `reject` **verdict**, and classifies any
disagreement. The unit of comparison is a *labelled tool*, so
`--yosys-mode both` contributes two labelled verdicts and a
without-abc-vs-with-abc disagreement is itself a divergence.

It is surfaced three ways over the one detector (decision `0017`: one
home, no drift):

- **the `tool_matrix --divergence` column** — a per-unit
  `DivergenceReport` over the tools the matrix already ran (no extra
  tool spawn, and — unlike `--diff-sim` — no tool-clean precondition,
  because a divergence is most interesting when one tool rejects what
  another accepts);
- **the `anvil hunt --divergence` axis** — a swept finding with
  `detection = "acceptance_divergence"` (see the User Guide);
- **the MCP `divergence` controlled tool** — for an agent (see
  [Driving anvil from an AI Agent](agent-mcp.md)).

<!-- book-test: skip — opt-in column requires verilator + yosys on PATH; documented in the verification log of ACCEPTANCE-DIVERGENCE-HUNTING.2c.2 -->
```bash
# Add the per-unit acceptance-divergence column to a tool_matrix run
cargo run --bin tool_matrix -- --divergence --out ./tool-matrix
```

The per-unit outcome is recorded under `ModuleReport.divergence` /
`DesignReport.divergence`:

```json
{
  "run_id": "…",
  "lane": "dut",
  "kind": "module",
  "top": "mod_1_0000",
  "sandbox": "…",
  "verdicts": [
    { "tool": "verilator",         "verdict": "accept", "exit_code": 0 },
    { "tool": "yosys-without-abc", "verdict": "accept", "exit_code": 0 },
    { "tool": "yosys-with-abc",    "verdict": "accept", "exit_code": 0 }
  ],
  "diverged": false,
  "divergences": []
}
```

The `saw_acceptance_divergence` coverage fact is **opportunistic** —
it fires when a divergence is seen but is **never a required gate**,
because on valid-by-construction RTL the steady state is that all tools
**agree** (`diverged: false`, as above). The detector also has a
tool-version-vs-version axis (one tool *kind*, two caller-supplied
binaries — e.g. `verilator-5.046` vs `verilator-4.228` — classified
`version_mismatch`); that axis is a library surface only, because an
allow-listed kind with a caller-supplied binary path is a larger trust
surface than the fixed-binary tools and is not exposed over the agent
interface.

The contract (`ACCEPTANCE-DIVERGENCE-HUNTING.2f`,
`tests/divergence_e2e.rs`): on a machine with Verilator (and
optionally Yosys) installed, an all-agree sweep records the full
per-tool verdict matrix with `diverged: false`, and an injected
accept/reject pair classifies `accept_reject` — proving the matrix is
produced, correctly classified, and queryable.
