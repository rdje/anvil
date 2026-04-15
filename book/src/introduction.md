# Introduction

`anvil` is a constrained-random generator of synthesizable SystemVerilog
RTL. It exists because, as of this writing, there is no good way to
pseudo-randomly produce RTL that is simultaneously:

1. **Syntactically valid** — parses without error.
2. **Semantically correct** — elaborates; widths match; names resolve;
   no multi-driven nets; no illegal lvalues.
3. **Synthesizable** — lives inside the subset of SystemVerilog that
   synthesis tools actually accept.
4. **Functionally non-trivial** — outputs genuinely depend on inputs;
   the whole circuit does not collapse to a constant under optimization.

Grammar-based fuzzers hit (1) easily but almost never satisfy (2).
Hand-written test suites hit all four but have terrible coverage —
they cluster around whatever patterns the author happened to think of.
There is no equivalent of Csmith for hardware.

`anvil` fills that gap. It builds a typed circuit IR by direct
recursion ("what drives this signal?"), where every construction step
is constrained to preserve validity. The IR is then emitted as SV text.
Every module is reproducible from a seed. Every knob is explicit.

The rest of this book explains *why* the design is the way it is.
If you want to use the tool, see [USER_GUIDE.md](../../USER_GUIDE.md).
If you want to extend it, read [The Core Idea](core-idea.md) first.
