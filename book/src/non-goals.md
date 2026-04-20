# What We Explicitly Do Not Do

A project's clarity comes as much from what it refuses to do as from
what it delivers. These are `anvil`'s non-goals.

## No bundled oracle, no reference simulator

`anvil` absolutely aims to stress downstream tools and expose bugs in
them. What it does **not** do is ship a SystemVerilog interpreter, a
reference simulator, or a golden-model evaluator of its own. Users who
want differential testing between tools should run Verilator, Icarus,
or a commercial simulator against the generated output themselves.

Why: the scope is already large. An oracle doubles the implementation
effort and introduces a second correctness question (is our
interpreter correct?). Most users who want RTL generation do not want
a bundled simulator.

This is an implementation-boundary statement, not a lowering of the
quality bar. `anvil` is still intended to generate high-quality legal
RTL that is clean in downstream tools by default.

## No non-synthesizable output

The gate set, the flop pattern, and the emitter are all restricted to
the synthesizable subset of SystemVerilog. There is no mode that
emits `initial` blocks, delays, `$display`, `fork`/`join`, dynamic
arrays, classes, or other non-synthesizable constructs.

Why: the value proposition is "random *synthesizable* RTL." Relaxing
that makes the tool another grammar fuzzer.

## No testbenches

`anvil` generates DUT code only. It does not generate testbenches,
assertions, cover properties, or stimulus.

Why: good testbenches require semantic understanding of the DUT
(what inputs are legal, what outputs mean). A random testbench for
a random DUT tests nothing. Users who want stimulus can write their
own wrappers.

## No semantic documentation

Generated modules do not include comments explaining what they "do"
because they do not do anything meaningful. The logic is intentionally
random. Emitting fake functional descriptions would be dishonest and
would mislead automated tools that try to reason about design intent.

The emitter may include a header comment with generation metadata
(seed, knobs, node count, generation time) for traceability, but
nothing about functionality.

## No grammar engine

`anvil` does not interpret an annotated EBNF at runtime. The generator
is handwritten Rust. The grammar view is useful as a correctness
argument but does not drive the implementation.

Why: see [Why Not a Grammar?](why-not-grammar.md). Briefly: the IR
approach is more direct, easier to extend, and does not require
threading SV's syntactic idiosyncrasies through every generation
decision.

## No attempt at realistic designs

`anvil` does not try to produce RTL that looks like something a human
would write for a real purpose. The outputs are intentionally
nonsensical in function, even though they are structurally valid.

Why: constraining output toward "realistic" patterns defeats the
whole point of random generation, which is to exercise the vast space
of *unusual but legal* constructs that humans never write. The target
is adversarial but valid workloads, not realistic business logic. If
you want realistic RTL, hire an engineer.

## No attempt at coverage guarantees

`anvil` does not claim to cover all of synthesizable SystemVerilog,
or any particular fraction of it. The motif set grows incrementally
and represents what the maintainers have bothered to implement. There
is no coverage model, no gap analysis, no "we guarantee every
synthesizable construct appears eventually."

Why: honest scope. Coverage metrics for constrained-random RTL are an
unsolved research problem.

## No CDC, no multi-clock (until maybe Phase 7)

Multi-clock designs require CDC-safe handshake synthesis, which is
subtle. `anvil` uses one clock and one reset until/unless Phase 7
lands with explicit CDC motif support.

## No formal proof of correctness

The "by construction" argument is a design principle, not a machine-
checked proof. We do not use a theorem prover or a verified compiler.
The generator could still have bugs; the validator exists as a
safety net.

Why: formal verification of a generator of this size is an academic
project, not a tool. We rely on invariants, tests, and post-hoc
smoke-checks with real SV tools.
