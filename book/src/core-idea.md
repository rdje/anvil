# The Core Idea

This chapter captures, in detail, the reasoning that led to `anvil`'s
design. It is the single most important document in this book. Later
chapters elaborate individual aspects; this one states the whole thesis.

## The single guiding principle: recursion

Before any of the discussion that follows, hold this thought: **`anvil`
is recursive by design, and recursion is its core principle.** Every
non-trivial generation step in `anvil` is — or should be — a recursive
descent over the typed circuit graph. The fanin-cone builder is
recursive. Hierarchical module instantiation is the same recursion at a
larger granularity. Advanced motifs (FSMs, memories, parameterized
sub-designs) are added by extending the recursion's choice set,
not by introducing iterative scaffolding around it.

When a contributor asks "should this be a loop or a recursion?", the
default answer is recursion. Iteration is reserved for cases where
ordering or termination semantics genuinely require it (the flop
worklist drainer, the per-output cone driver). Even those iterative
shells exist only to *kick off* recursive cone construction.

The reason this is load-bearing: the entire correctness argument for
"valid by construction" rests on the recursive structure. Each
recursive call carries the constraints (target width, scope, depth
budget, dep-set context) that make the local decision valid. Replacing
recursion with iteration tends to push those constraints into shared
state, which is where invariants silently break.

## The problem

For ANVIL's purpose, an RTL file does **not** need intended top-level
functionality in order to be useful. What it needs is to be legal,
synthesizable, structurally rich, and ingestible by downstream tools.
Whether a whole module is "functionally correct" is a different
question: that requires a specification, and most generated modules do
not have one.

Unfortunately, there does not seem to exist any practical way to
pseudo-randomly generate RTL that is semantically correct — meaning it
elaborates, types check, widths align, names resolve, no net is driven
twice, and every referenced signal exists.

Grammar-based generation (walk an EBNF, emit tokens) produces
syntactically valid text but almost never semantically valid text. The
semantic constraints of RTL are tight enough that random derivations
are overwhelmingly rejected during elaboration. You end up filtering
99%+ of your output, which is both expensive and biased toward the
easiest-to-generate patterns.

## Verbatim doctrinal anchor: structure over intended functionality

The following user guidance is preserved **verbatim** because it is the
clearest statement of what ANVIL is, and is not, trying to do:

> Let's be clear. Generating module by recursively generating fanin cones of its outputs, mechanically means that the resulting functionality will be gibberish but that's not the point. Having functioning behavior makes no sense here. For some modules, we might get some usable functionality but that's not the goal. The ultimate goal is to be able to generate synthesable legit RTL code that downstream tools (parser, synthesizer, linter, ...) can ingest.
>
> My construction we are not aiming at functionality but at structure, capiche.
>
> ANVIL will be able to create complex to very complex synthesizable RTL code.
>
> Any functionally correct synthesizable RTL code is undistinguishable from an functionally incorrect or even gibberish code at first sight, to ensure function correctioness one need functonal verification which needs to match a specification against a RTL module.
>
> So no one can tell at first glance whether a RTL is gibberish or functionally correct with a specification, meaning for most of what will be generated, function correctness is not the goal and can't be by construction.
>
> But they are features that will create functionally correct blocks.

Derived reading:

- whole-module intended behavior is usually arbitrary and often
  gibberish;
- that is acceptable because ANVIL targets structure, not design intent;
  and
- some **local motifs** can still be functionally correct blocks by
  construction, even when the enclosing module has no meaningful
  top-level specification.

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

Every choice is a local decision. Every local decision is bounded by
the invariants we care about:

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

- **No oracle.** `anvil` is a generator, not a bundled semantic oracle.
  Downstream users can run Verilator or Yosys against the output for
  validation, differential checks, or sanity testing; `anvil` does not
  build a reference simulator. This is a deliberate scope reduction, not
  a retreat from the goal of generating high-quality RTL that downstream
  HDL consumers should ingest cleanly.
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
clocking, reset). Verification has embraced randomized testbench
generation (UVM, SystemVerilog's randomization features), but the RTL
source itself still lacks a broad random by-construction generator. The
tools that exist in this space (commercial IP generators, academic
fuzzers) either produce narrow patterns or produce mostly-invalid text.

`anvil` is an attempt to fill that gap with a principled,
by-construction approach — small, extensible, reproducible, and honest
about its scope.
