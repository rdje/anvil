# Why Not a Grammar?

An earlier design considered walking an annotated EBNF of SystemVerilog
with semantic attributes (inherited width, sign, scope; synthesized
dependency sets). That is a known-correct approach — it's the
attribute-grammar formalism Knuth described in 1968 — and would in
principle produce the same output `anvil` produces.

We did not go that way. Here is why.

## The two approaches converge

A grammar walk and a circuit-cone recursion are the same computation
viewed from different angles:

| Grammar view                                | Circuit view                    |
|---------------------------------------------|---------------------------------|
| Pick a production rule alternative          | Pick what drives this node      |
| Inherited attribute (width flows down)      | Parent sets child's width       |
| Synthesized attribute (deps flow up)        | Child tells parent its deps     |
| Scope attribute updated by declarations     | SignalPool grows as wires added |
| Production for `expr ::= expr '+' expr`     | Pick an `Add` gate, recurse     |

They produce the same tree of decisions. The correctness argument is
the same: every choice preserves invariants, therefore the derivation
is valid.

## So why pick one over the other?

**The grammar view is more formal.** It gives you a clean statement:
"this is an attribute grammar; every derivation is a valid SV program;
therefore our generator is correct by construction." That's a nice
proof.

**The circuit view is more direct.** You're building the object you
actually care about — a circuit — and emitting SV from it. The SV
grammar only matters at the pretty-printing step. There is no reason
to thread SV's syntactic idiosyncrasies through the generation logic.

**SV's grammar is enormous.** The LRM defines hundreds of productions,
most of which are not synthesizable. Writing annotated versions of all
of them is a lot of work, most of which gets thrown away because we
only emit a small subset. The circuit IR is a clean distillation of
just the constructs we care about.

**Attributes that mutate across siblings are awkward.** Pure attribute
grammars split attributes into inherited (flow down) and synthesized
(flow up). But scope-after-declaration, driven-set, and the flop
worklist are really *threaded state* — they mutate across sibling
productions in order. Modeling these as attribute grammar gets clumsy;
modeling them as a recursive function with `&mut Context` is natural.

**Extensibility is local.** Adding a new motif (a new gate type, a new
flop style, a new structured op) is one case in one enum and one arm
in the emitter. Adding it to a grammar means new productions, new
annotation propagation rules, and changes to the grammar walker.

## The grammar framing still has value

Even though the code is not grammar-driven, it is worth keeping the
grammar view in mind:

- It gives the **formal correctness argument**: every constructor in
  the IR preserves its invariants, which is isomorphic to every
  production in an annotated grammar being valid under its attributes.
- It suggests **what invariants to track**: the attributes of an
  attribute grammar for SV are exactly the things the IR must track
  (width, sign, scope, deps, driven-set, clock domain).
- It offers **a path to generalization**: if later we want a declarative
  spec of the generator (e.g., for users to extend without writing
  Rust), an annotated grammar is a plausible format.

For now, the grammar lives only as a mental model. The code is a
direct recursion over a typed IR.
