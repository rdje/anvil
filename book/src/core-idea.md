# The Core Idea

This chapter captures, in detail, the reasoning that led to `anvil`'s
design. It is the single most important document in this book. Later
chapters elaborate individual aspects; this one states the whole thesis.

## The problem

An RTL file has to be functionally correct for it to make sense to use.
It is not like a regex that can be random and still exercise a parser
usefully. Unfortunately, there does not seem to exist any practical way
to pseudo-randomly generate RTL that is semantically correct — meaning
it elaborates, types check, widths align, names resolve, no net is
driven twice, and every referenced signal exists.

Grammar-based generation (walk an EBNF, emit tokens) produces
syntactically valid text but almost never semantically valid text. The
semantic constraints of RTL are tight enough that random derivations
are overwhelmingly rejected during elaboration. You end up filtering
99%+ of your output, which is both expensive and biased toward the
easiest-to-generate patterns.

## The insight

Do not generate RTL as text. Generate it as a **typed circuit graph**,
built by recursive construction where every step maintains invariants
by definition, and then emit SystemVerilog from that graph as a final
pretty-printing step.

The key realization: what looks like an expression-language problem
("generate a valid SV expression") is actually a circuit-topology
problem ("build a fanin cone that drives this signal"). The two are
isomorphic, but the circuit framing is dramatically easier to reason
about because it matches the physical object being described.

## The algorithm

For each module:

1. Pick N inputs and M outputs, each with a randomly-chosen bit width.
2. For each output signal S in turn, build its fanin cone by recursion.

The fanin cone recursion is the whole generator. At each node, answer
one question: **what drives this signal?**

The choices are:

- **Primary input** — terminates the cone at a module input port.
- **Flop output (Q)** — terminates the *combinational* cone at a flop;
  the flop's D input opens a new cone to be generated separately.
- **Constant** — terminates with a literal; restricted to avoid
  producing trivially constant outputs.
- **Combinational gate** — pick a gate type (`and`, `or`, `xor`, `not`,
  `mux`, `+`, `-`, `==`, `<`, `slice`, `concat`, …), pick its arity,
  then recurse on each input.
- **Existing wire from the pool** (DAG mode) — terminate at a signal
  that has already been generated elsewhere in the module, producing
  sharing and fanout.

Termination is controlled by a depth budget. As depth increases, the
probability of picking a terminal (input, flop-Q, constant, shared
wire) rises until it hits 1.0 at the maximum depth.

## Why this works

The recursion naturally builds a DAG rooted at each module output. When
a node picks a gate, the recursion continues. When a node picks a
terminal, that branch stops. When a node picks a flop, the current
combinational cone terminates at Q, and the flop's D input becomes the
root of a fresh combinational cone that will be generated later (driven
by a worklist).

Every choice is a local decision. Every local decision is constrained
to preserve the invariants we care about:

- **Width consistency** — the parent tells the child what width to
  produce; the child's generator is parameterized by that width and
  can only pick gates that produce that width.
- **Name resolution** — terminals are chosen from pools of already-
  declared signals; there is no way to reference an undeclared name.
- **Drive uniqueness** — each output and internal wire is the target
  of exactly one cone-generation call; the builder never produces two
  assignments to the same target.
- **Synthesizability** — the gate set is restricted to synthesizable
  operators; the flop pattern is canonical (`always_ff` with a clean
  clock and optional reset); no `initial`, no delays, no system tasks.
- **Non-triviality** — every expression node tracks its *dependency
  set* (the subset of primary inputs it actually depends on); the
  generator rejects output cones whose dep-set is empty.

Because every step preserves these invariants, the finished graph is
valid by construction. No post-hoc filtering. No validator that might
reject output. The graph *cannot* be invalid, because there was never
a moment in its construction when it could become invalid.

## Two levels of abstraction

The cone-recursion algorithm produces one **leaf module**: N inputs,
M outputs, internal combinational logic and flops, no sub-module
instances. That is the first level.

The second level is **hierarchy**. A higher-level generator picks (or
creates on demand) sub-modules and instantiates them inside a parent
module. The parent is generated with the *same* cone-recursion
algorithm, except that at each cone node the choice set is extended
with "instantiate a sub-module and use one of its output ports." The
sub-module's input ports then become new sub-cones to drive.

The algorithm is structurally identical at both levels. The only
difference is the set of choices available at each node.

Complexity grows incrementally by turning knobs: port counts, widths,
depth, flop probability, sharing probability, hierarchy depth, and
gate-type weights. Every knob is explicit. Every output is reproducible
from `(seed, knobs)`.

## What we deliberately do not do

- **No oracle.** `anvil` is a generator, not a tool tester. Downstream
  users can run Verilator or Yosys against the output for
  differential/sanity testing; `anvil` does not build a reference
  simulator. This is a deliberate scope reduction.
- **No grammar.** An annotated EBNF is a valid way to describe this
  generator formally, and attribute grammars would yield an equivalent
  result. But the circuit-graph view is more direct, more visual, and
  easier to extend. The grammar framing remains useful as a
  *correctness argument* — every constructor preserves invariants,
  therefore all output is valid — but no grammar notation appears in
  the code.
- **No generate-then-filter.** Validity is structural, not checked.
- **No non-synthesizable output.** The gate set, the flop pattern, and
  the emitter are all restricted to the synthesizable subset. There is
  no mode that emits `initial` blocks or delays.

## Why it didn't exist already

The RTL community is smaller than the C compiler community, and RTL
semantics are harder (concurrency, synthesis-vs-simulation divergence,
clocking, reset). Verification has embraced constrained-random
testbench generation (UVM, SystemVerilog's randomization features) but
has not applied the same philosophy to the RTL source itself. The
tools that exist in this space (commercial IP generators, academic
fuzzers) either produce narrow patterns or produce mostly-invalid text.

`anvil` is an attempt to fill that gap with a principled,
by-construction approach — small, extensible, reproducible, and honest
about its scope.
