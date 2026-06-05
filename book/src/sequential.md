# Sequential Logic: Flops and Cone Boundaries

Flops are part of the same fanin-cone recursion as combinational logic
— they are not a later phase. The recursion principle handles them
naturally:

- **Q is a leaf in the *current* cone.** When the recursion picks
  "this signal is driven by a flop," the flop's Q output terminates
  the descent for this cone, exactly like a primary input does.
- **D opens a *new* cone.** The flop's D input becomes a fresh sub-cone
  rooted at D, queued on a worklist for later construction by the same
  `build_cone` function. That sub-cone may itself contain flops; their
  Ds get queued in turn.
- **The worklist drains to quiescence.** The main loop pops flops one
  at a time and recursively builds their D-cones until no flops remain
  pending.

This is why we did not split sequential into a later phase: it is the
same recursion with one extra choice in the node-kind picker.

## Synchronous-design discipline

Every module is **synchronous, with one or more declared clock
domains**:

- The K=1 default ("single clock domain") uses one `clk` input
  port (1 bit, posedge) + one `rst_n` input port (1 bit, async,
  active-low). Every flop uses these two ports; all flops emit
  into a single `always_ff @(posedge clk or negedge rst_n)`
  block.

- The K=N multi-clock case (`MULTI-CLOCK-CDC`) declares N pairs
  of `clk_X`/`rst_n_X` input ports + N `ClockDomain` entries on
  `Module.clock_domains`; every flop carries a `domain` tag in
  `Module.flop_domains` (defaulting to 0 — the K=1 special
  case); the emitter generates one `always_ff @(posedge clk_X
  or negedge rst_n_X)` block per (domain, polarity) tuple.
  Every supported cross-domain signal is wrapped by-construction in a
  synchronizer chain. The default chain is the original 2-flop
  primitive; `Config::cdc_synchronizer_stages >= 3` opts into the
  N-flop variant. There is no generate-then-filter step (rules-first
  generation, `feedback_rules_first_generation.md`).

- *No* per-flop *clock-source* choice, per-flop *reset polarity*
  choice, or mixed sync/async semantics. The IR has no field
  for per-flop clock or per-flop reset polarity; the only
  per-flop axis is the **domain tag**, which selects which
  declared `(clk, rst_n)` pair the flop is clocked by.

The K=1 default is byte-identical to pre-`MULTI-CLOCK-CDC` ANVIL
(`Module.clock_domains.len() == 0` triggers the synthesised
single-domain default in `Module::effective_clock_domains`);
K≥2 is opt-in via `Config.multi_clock_prob > 0.0`.

## Why this discipline

A real synchronous digital design — the kind that ships in silicon —
has exactly this shape: one or more clocks, one or more resets, all
sequential elements within a domain clocked together, every
domain-crossing signal properly synchronized.

Forcing the discipline by construction — there is no IR field for
per-flop clock-source choice (only the domain tag) or per-flop
reset polarity — guarantees that no random choice can violate it.
For the multi-clock case, the by-construction rule extends to
domain-crossing signals: the generator never emits a flop in
domain B whose D-cone references a domain-A flop output directly;
instead, the synchronizer wrap is constructed in place via
`construct_nflop_synchronizer` (rules-first generation per
`feedback_rules_first_generation.md`). The default stage count is 2,
and higher stage counts remain the same 1-bit crossing primitive with
additional destination-domain flops.

## Cone boundaries

Without flops, every primary output is the root of one combinational
cone that recurses until it hits primary inputs. A module is one
forest of combinational cones, one per output.

With flops, a cone can terminate at a flop's Q output. From the
perspective of the cone being built, Q is a leaf — same as a primary
input. But the flop itself has a D input, which must be driven by
*another* combinational cone, generated separately.

The result is a circuit with multiple "cone regions" stitched together
by flops. This matches the standard definition of synchronous digital
logic: combinational logic between registers.

## The worklist

```text
flop_worklist = Queue::new()

# generate output cones first
for out in outputs:
    drive_cone(out, build_cone(...))

# drain flop worklist
while not flop_worklist.empty():
    flop = flop_worklist.pop()
    drive_cone(flop.d, build_cone(...))
```

Inside `build_cone`, when the recursion picks "this node is a flop":

1. Allocate a new `Flop` with the requested width.
2. Add it to the module.
3. Push it onto the worklist (its D-cone will be generated later).
4. Add its Q to the signal pool (so subsequent cones can share it).
5. Return the FlopQ node as the chosen sub-expression.

The worklist may grow during draining (a D-cone may itself contain
flops). The loop terminates because cone recursion has a finite depth
budget and module construction has a finite `max_flops_per_module`
budget; `flop_prob` may be set to `1.0` for stress profiles without
making the worklist unbounded.

## Flop reuse

Once a flop exists, subsequent cones (other outputs, other flops' Ds)
may pick that flop's Q from the signal pool. This is how sequential
circuits get *shared state* — multiple downstream signals reading the
same register.

Without reuse, every output would have its own private flop chain,
producing unrealistic and bloated designs. Reuse probability is
controlled by the `--share-prob` knob.

Under `identity_mode = node-id`, there is a second sequential-sharing
path after drain: if two flops ended up with the same emitted state
semantics over the same canonical leaf endpoints, they are merged even
if they were born as distinct registers. In the current generator flow,
this pass runs during leaf finalisation before the opt-in multi-clock
promotion pass, so promotion-added synchronizer flops are not
re-merged by this pass. The signature still includes
`Module::flop_domain`, so a library caller or future pass that runs
after explicit domain tags exist will not merge equal-looking flops
across domains. The proof is still bounded:
normalized structural proof first, plus a bounded semantic check for
small-support cones. The semantic branch has the same support/node/work
budget as the combinational merge proof, so shallow 12-endpoint-bit
cones can qualify and larger candidates fall back to structural proof.
That is stronger than exact `d: NodeId` equality, but still not full
sequential equivalence.

There is one deliberately narrow coinductive class: exact reset-defined
self-hold. If two flops have the same width, reset kind, reset value,
and clock/reset domain, and each D input is exactly its own Q, they can
share one state element:

```systemverilog
always_ff @(posedge clk or negedge rst_n) begin
    if (!rst_n) begin
        a <= 8'h00;
        b <= 8'h00;
    end else begin
        a <= a;
        b <= b;
    end
end
```

After reset, `a` and `b` are equal; each next state is its current
state, so equality is preserved forever. ANVIL does **not** extend that
reasoning to reset-less self-hold, cross-domain self-hold, reset-value
mismatches, width mismatches, mutually-recursive registers, retiming, or
feedback that is only semantically equivalent after additional rewriting.

The same identity discipline now applies to deterministic generated FSM
blocks. FSMs reset to state 0 and carry explicit transition and
Moore-output tables, so two FSM blocks can share one state machine when
their selector proof, encoding, state count, transition table, output
table, and output width match. Memories stay opaque because their
stored contents are not reset-defined in the current template.

## K=1 clock and reset shape

The default K=1 path declares one clock and one reset as ports of the
module whenever the module contains at least one flop:

```systemverilog
module mod_42_0007 (
    input  logic        clk,
    input  logic        rst_n,
    input  logic [7:0]  i_0,
    ...
    output logic [7:0]  o_0,
    ...
);

    always_ff @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            flop_0 <= 8'h0;
            ...
        end else begin
            flop_0 <= add_3;
            ...
        end
    end
```

In this K=1 shape, every flop uses `clk` (posedge) and `rst_n`
(async, active-low). Reset value is chosen per flop, biased toward 0.
For K=N multi-clock modules, each flop's domain tag selects one of the
declared `(clk_X, rst_n_X)` pairs. There is no per-flop reset polarity
choice or mixed-edge flop choice — see "Synchronous-design discipline"
above.

(Flop names are `flop_<id>` per Rule 12; the D-driving wire is a
gate named `<kind>_<N>` — `add_3` above stands in for whatever op
the generator picked. See [Rule 12](structural-rules.md) for the
full naming contract.)

When a module carries no sequential state anywhere, the `clk` and
`rst_n` ports are omitted from the port list. This avoids spurious
"unused input" lint warnings on pure comb-only modules.

For hierarchy, the rule is inductive:

- a sequential leaf emits `clk` / `rst_n`;
- a pure comb-only module does not; and
- a wrapper emits `clk` / `rst_n` iff it carries sequential
  descendants through instantiated children.

### Multi-clock and CDC

Multi-clock support landed via `MULTI-CLOCK-CDC`. Opt-in via
`Config.multi_clock_prob > 0.0` (`Cli.multi_clock_prob` is
configuration-only — same convention as `memory_prob` /
`fsm_prob`). When the per-module Bernoulli roll fires, the
`Generator::generate_module` /
`Generator::generate_design` paths apply the
`multi_clock::promote_to_multi_clock` post-construction pass:

1. Allocates two new ports (`clk_b` + `rst_n_b`) + pushes two
   `ClockDomain` entries (named `"a"` and `"b"`) onto
   `Module.clock_domains`.
2. Picks the first 1-bit output port directly driven by a
   flop's Q (declines cleanly on multi-bit outputs — those need
   handshake or async FIFO, deferred to a follow-up tree per
   `MULTI-CLOCK-CDC.1`'s catalogue tier 3-5).
3. Constructs a synchronizer chain in domain 1. The default
   `Config::cdc_synchronizer_stages = 2` calls the compatibility
   `src/gen/multi_clock.rs::construct_2flop_synchronizer` path; setting
   the config value to `N >= 3` uses the N-flop synchronizer extension.
   Every stage is a new flop in domain 1, chained
   D=src_q → stage0 → ... → synced_q.
4. Rewires the chosen output's drive to the synced Q.

The result is a K=2 module whose B-domain output is synchronized from
the A-domain source flop. Both Verilator and Yosys accept the emitted SV
without configuration (`int_multi_clock_2flop_sync` and
`int_multi_clock_3flop_sync` scenarios in the default `tool_matrix`
sweep exercise the default and N-flop paths). The
`saw_multi_clock_design`, `saw_cdc_2_flop_synchronizer`, and
`saw_cdc_nflop_synchronizer` coverage facts surface in
`tool_matrix_report.json`.

For direct library use:

```rust,ignore
let mut cfg = anvil::Config {
    multi_clock_prob: 1.0,
    cdc_synchronizer_stages: 3,
    flop_prob: 1.0,
    min_width: 1,
    max_width: 1,
    ..anvil::Config::default()
};
let module = anvil::Generator::new(cfg).generate_module();
let metrics = anvil::metrics::compute(&module);
assert!(metrics.max_cdc_synchronizer_stages >= 3);
```

The pass is **rules-first** (`feedback_rules_first_generation.md`):
the synchronizer is constructed in place at the moment of the
domain-crossing decision; there is no post-pass filter.

**Current scope.** The current pass promotes one 1-bit flop-driven
output per fired module and supports 2-stage or N-stage synchronizer
chains for that crossing. Multi-bit signal transfer (async FIFO,
gray-code pointer, handshake), pulse synchronizers, and reset
synchronizers remain explicit follow-up tiers per
`MULTI-CLOCK-CDC.1`'s catalogue.

## Combinational cycles

Forbidden by construction. The cone recursion only references signals
that already exist in the pool when picked. The pool only contains
primary inputs, flop-Qs, and previously-generated internal wires from
*earlier* recursions. Newly-created gates are added to the pool only
*after* their operands are resolved. Therefore no gate can transitively
reference itself through pure combinational logic.

Sequential cycles (state machines) are *expected*: a flop's D can
reference its own Q, and that's a valid storage element. The flop
breaks the loop temporally.

## Flop motifs: M-to-1 mux on D

Every flop's D input is driven by either:

- **M = 0** — no mux; D is generated by a single recursive cone of width N
  (the simplest case, equivalent to a standard register).
- **M >= 2** — an M-to-1 mux in one of two encoding styles. M = 1 is
  excluded by design (a 1-arm mux is just a wire).

The encoding style is picked per-flop (`flop_mux_encoding_prob` knob):

- **One-hot style**: M 1-bit select bits, each a recursion point. The
  design contract is that at most one select fires at a time. Assembled
  as `OR_i({N{sel_i}} & data_i)` plus an optional Q-feedback term.
- **Encoded style**: one select bus of width `ceil(log2(M))`, a single
  recursion point. Value `k` routes `data_k` onto D. When `sel` falls
  outside `[0, M)` (possible when M is not a power of 2), the
  fall-through routes 0 (ZeroDefault) or Q (QFeedback).

Within each encoding style, `FlopKind` further chooses the
"no-valid-selection" behavior (ZeroDefault vs QFeedback). The two axes
are orthogonal:

| Style    | Kind          | No-select behavior                          |
|----------|---------------|---------------------------------------------|
| OneHot   | ZeroDefault   | D = 0 when every sel is 0                   |
| OneHot   | QFeedback     | D = Q when every sel is 0                   |
| Encoded  | ZeroDefault   | D = 0 when sel >= M; D = data_k when sel=k  |
| Encoded  | QFeedback     | D = Q when sel >= M; D = Q when sel=0; D = data_k when sel=k for k in [1, M) |

In the Encoded + QFeedback case, the slot at index 0 is *replaced* by
Q — there is no `data_0` sub-cone; the recursion builds only M-1
data sub-cones (indices 1..M).

The one-hot variants below describe the assembled gate tree; the
encoded variants use a chained ternary over `Eq(sel, k)` for each k.

### Kind 1 — `ZeroDefault`
```text
D = ({N{sel_0}} & data_0) | ({N{sel_1}} & data_1) | ... | ({N{sel_{M-1}}} & data_{M-1})
```
When all M selects are 0, all AND-masked terms are 0 and D = 0. The
flop loads zero on the next clock edge.

### Kind 2 — `QFeedback`
```text
none_selected = ~(sel_0 | sel_1 | ... | sel_{M-1})
D = ({N{sel_0}} & data_0) | ... | ({N{none_selected}} & Q)
```
When all M selects are 0, `none_selected` is 1 and the Q-feedback term
holds: D = Q. The flop holds its current value.

### Recursion structure

When M = 0, the flop's D is built by a single recursive cone of width
N. When M >= 2, every one of the M N-bit data entries and every one
of the M 1-bit select bits is a **recursion point**:

```text
build_flop_d(width N, kind):
    M = pick_M()                              // 0 or 2..=max_mux_arms
    if M == 0:
        return build_cone_with_retry(N)    # Q can be a leaf freely
    arms = []
    for i in 0..M:
        data_i = build_cone_with_retry(N)  # Q can appear any number of times
        sel_i  = build_cone_with_retry(1)  # Q can appear any number of times
        arms.push(data_i, sel_i)
    return assemble(arms, kind, Q)
```

The assembly step builds `replicate-AND-OR` gate trees from the
recursively-generated leaves. Sub-cones may themselves spawn flops,
which are queued on the same worklist; the drainer loops to quiescence.

### One-hot is a contract, not enforced

The select bits are recursively generated and not constrained to be
mutually exclusive. The hardware *assumes* they are one-hot (this is a
design contract). When the contract is violated, multiple data paths
OR together — which is exactly what the gate structure produces. There
is no extra logic in the generator to enforce one-hot at runtime.

### Q-feedback in the D-cone is freely permitted

A flop's own Q may appear **any number of times** as a leaf inside
any of its data, select, or direct-D sub-cones. Q→D feedback through
arbitrary combinational logic is a legal synchronous pattern —
counters, toggles, accumulators, state machines all work this way.
The clock edge breaks the loop temporally: `Q[n+1]` depends on
`Q[n]` plus possibly other inputs.

This is independent of the explicit Q-feedback mux term in
`FlopKind::QFeedback`. The mux term fires when *no* select asserts
and is a structured fall-through path. The Q-in-sub-cone freedom
just makes Q a normal shareable leaf during cone construction.

Combinational self-reference within the Q→D logic is still impossible
— that is Rule 1 (Combinational no-loop) in the
[Structural Rules catalog](structural-rules.md).

See Rule 2 in the [Structural Rules catalog](structural-rules.md) for
the authoritative statement.

## Reset value selection

Reset values are chosen randomly per flop, with a bias toward zero
(zero is by far the most common reset value in real designs). The
current distribution: 50% zero, 25% all-ones, 25% other random value.

This is a knob if it ever needs tuning, but the default is sensible.
