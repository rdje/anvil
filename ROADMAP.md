# Roadmap

`anvil` grows in phases. Each phase delivers a working generator with a
larger expressive subset. No phase should land without end-to-end tests
and at least one `.sv` artifact run through Yosys or Verilator as a
synthesizability smoke check. Those sweeps are evidence, not the end
goal: the intended steady-state is that generated modules are boringly
clean in Verilator and Yosys by default.

That quality bar coexists with breadth. `anvil` is meant to grow into a
signoff-grade random synthesizable RTL generator that finds bugs in
downstream tools by feeding them legal, unusual, feature-rich designs,
not by relying on malformed input or low-quality noise.

Whole-module intended functionality is not a roadmap goal. The roadmap
optimizes for structurally rich, legitimate, synthesizable RTL that
tools can ingest; local motifs may be functionally correct blocks, but
the top-level module usually has no meaningful specification.

## Four steering gaps from the codebase suitability assessment (2026-04-20)

The current codebase is suited to the product goal as a **foundation**,
but these four gaps must stay explicit. They are already spread across
the phased plan below; this section makes them durable as a steering map
instead of leaving them implicit.

1. **Feature breadth / legal design-space width**
   The current engine is still fundamentally a leaf-module generator.
   Reaching "complex to very complex synthesizable RTL" requires Phases
   3, 4, 5, 5b, and 6 to land as real generator surfaces: richer
   structured combinational blocks, hierarchy, parameterization, packed
   aggregates, memories, FSMs, and other legal interaction-heavy
   motifs. Every new category and knob must be exercised in generation
   paths, tests, metrics, and downstream tool sweeps; dead knobs or
   paper-only categories are regressions.

2. **`NodeId` as identity / full-factorization mode**
   The strong-form target is: under `identity_mode = node-id`,
   equivalent expressions anywhere in any output cone or flop-D cone
   should converge to one `NodeId`, so sharing of gates, blocks,
   modules, and flops is as high as the current build knows how to
   prove. Today's implementation covers normalized combinational
   identity plus exact-signature duplicate-flop merge; stronger
   sequential and hierarchical equivalence are still open work. This
   mode must remain user-controllable from the CLI: `--identity-mode
   relaxed` is a real off-switch, while `--factorization-level`
   continues to express weaker or stronger canonicalization within
   `node-id`.

3. **Signoff-quality tool-clean industrialization**
   Seed-level cleanliness is not enough. The project needs automated
   Verilator/Yosys evidence across seeds, construction strategies,
   identity modes, factorization levels, category mixes, flop/no-flop
   cases, and future hierarchy/memory/FSM features. Counterexamples must
   be retained with exact seed+config evidence and fed back into IR
   invariants or rewrites, not hidden behind warning suppressions. The
   intended steady-state remains: generated RTL is boringly clean in
   mainstream tools by default.

4. **Structure-first, not whole-module specification-first**
   ANVIL optimizes for structural legitimacy, synthesizability,
   complexity, factorization pressure, and downstream-tool ingestibility
   rather than intended top-level behavior. Features that create locally
   meaningful or functionally correct blocks are welcome, but ANVIL is
   not turning into a bundled oracle or spec-driven synthesis engine.
   When choosing between slices, prefer new legal interaction surfaces
   and stronger by-construction invariants over post-hoc whole-module
   "meaningfulness" scoring.

## Phase 0 — Scaffolding (done)

- Cargo project, module skeleton, CLI entry point.
- Design docs (`book/`) capturing the core algorithm.
- `Module`, `Port`, `Net`, `Node`, `Gate`, `Flop` IR types defined.
- CLI accepts `--seed`, `--count`, `--out`, `--config`, `--dump-config`.

**Exit criteria (met locally):** `cargo build`, `cargo test`,
`cargo clippy -D warnings`, `cargo fmt --check` all clean. Reproducibility
test passes byte-identical output for the same seed.

## Phase 1 — Single-module MVP (mostly done)

One module, no hierarchy, no inter-module sharing. Combinational *and*
sequential logic from the start — flops are part of the same fanin-cone
recursion (Q is a leaf, D opens a new sub-cone, worklist drains).

- Random N inputs, M outputs, random widths per port.
- Per-output fanin cone recursion with depth budget.
- Gate set: bitwise (`and`, `or`, `xor`, `not`), arithmetic
  (`+`, `-`, `==`, `<`), `mux`, `slice`, `concat`, constants.
- **Sequential discipline:** single `clk` (posedge) and single `rst_n`
  (async, active-low) shared by every flop in the module. One
  `always_ff @(posedge clk or negedge rst_n)` block per module.
- Width propagates top-down; dependency set propagates bottom-up.
- Non-triviality: every output and every flop-D cone has dep-set ≥ 1,
  enforced at cone root.
- Tree-shaped cones only (each internal signal has one consumer).
- SV emitter produces `module` + `assign` + `always_ff`.

**Exit criteria:** 1000 modules generated from random seeds, all parse
and elaborate in Verilator without error, all Yosys-synthesize to
non-empty netlists, both with and without flops. **Not yet met:**
local tools are now available and seed-level smoke checks pass, but
the 1000-module Verilator+Yosys sweep has not been run yet. Internal
validation (123 tests, unused-signal Verilator sweep over seeds 0..4
for the default path and the `graph-first` alias, plus a warning-clean
seed-42 Verilator lint and seed-42 Yosys synthesis) is clean.

## Phase 2 — Signal sharing (DAG cones) (in progress)

- Signal pool of already-created internal wires.
- Per-operand `share_prob` decision: recurse (tree) or reuse (DAG).
  Mixing is the default — a single gate's operands can freely combine
  shared and freshly-built sub-cones.
- Under `identity_mode = node-id` with effective factorization level
  `>= cse`, an exact-signature post-drain flop merge now extends sharing
  to state elements too: duplicate flops with the same `width`, reset,
  and `d` collapse to one register.
- Dep-set propagation correctly handles shared fanout.
- Fanout stress: a single wire can drive many consumers.
- Anti-collapse rules still apply post-share (no `x ^ x` even when both
  operands come from pool reuse).

**Exit criteria:** generator produces cones with controlled sharing
factor; synthesis still succeeds; no multi-driver violations; Verilator
lint passes on a representative seed sweep with `share_prob` ∈ {0.0, 0.3, 0.9}.

## Phase 3 — Structured combinational ops (in progress)

- `case`/`casez` expressions. **Not started.**
- Priority encoders, one-hot decoders. **Priority encoder landed
  (Rule 17).**
- Reduction operators (`&`, `|`, `^` unary). **Selectable gate
  category landed.**
- Shift by variable amount. **Constant-amount shifts landed
  (Rule 19 knob + const_shift_amount_prob); variable amount not
  started.**
- `for`-loop unrolled logic (statically bounded). **Not started.**
- Linear-combination compound motif (`Σ sᵢ·cᵢ`, etc.) **landed.**

**Exit criteria:** motif library covers common synthesizable idioms.

## Phase 4 — Hierarchy (not started)

- Module instantiation: at any cone node, optionally emit a sub-module
  call instead of a gate.
- Two sourcing modes:
  - **Library**: pre-generate a pool, pick from it.
  - **On-demand**: generate a fresh sub-module with required port widths.
- Arbitrary hierarchy depth, bounded by knob.
- Name uniqueness across the module set.
- Hierarchical identity is future required work: under
  `identity_mode = node-id`, equivalent instantiated structures should
  eventually participate in the same sharing story instead of creating a
  second identity system beside gates/flops.

**Exit criteria:** multi-file output directory with correct top module
declared; Verilator elaboration of hierarchy succeeds.

## Phase 5 — Parameterization (not started)

- Generated modules take `parameter` declarations for widths.
- Instantiation picks parameter values from allowed ranges.
- Parameter-dependent widths propagate correctly through cone generation.
- **Hard prerequisite:** Phase 4 (hierarchy). Parameters only matter
  at instantiation time.
- Parameter-aware identity must remain sound: different parameter values
  cannot accidentally alias to one `NodeId` or one module instance
  unless the resulting structure is genuinely equivalent.
- IR-level design recorded in `book/src/ir.md` "Future extensions /
  Parameters and generics".

## Phase 5b — Synthesizable aggregates (not started)

Scheduled alongside Phase 5; order is not fixed.

Three sub-paths, each with its own cost and payoff (full analysis in
`book/src/ir.md` "Future extensions / Synthesizable aggregates"):

- **Packed struct / union / array** — emitter-layer change only; IR
  stays flat. Low cost. Primary value: parser / elaboration coverage
  in downstream tools. Can land independently of Phase 4.
- **Unpacked arrays** — the memory-inference pattern. Covered by
  Phase 6 below.
- **Unpacked struct / union for datapath, enums** — deprioritised
  (unpacked datapath is mostly non-synthesizable; enums add no
  distinct stress value beyond typed constants).

## Phase 6 — Advanced motifs (not started)

- Memories (single-port, dual-port, with inferrable patterns only).
- FSMs with explicitly generated state encodings.
- Multi-clock with CDC-safe handshakes — optional, expensive. Until
  this lands, every module remains fully synchronous to a single clock.
- These motifs are not just feature-count work; they are a major part of
  the legal interaction richness needed for ANVIL to become a strong
  downstream bug finder without sacrificing clean-tool quality.

## Non-goals

- Testbenches, assertions, coverage — `anvil` generates DUT code only.
- Non-synthesizable constructs (`initial`, delays, system tasks beyond
  `$display` in debug comments).
- Language coverage beyond the synthesizable SV subset.
- Bundled oracle / reference simulator — `anvil` does not embed a
  shadow RTL semantics engine. The goal is still to stress downstream
  tools aggressively, but by generating high-quality legal RTL rather
  than by turning `anvil` into a second simulator.
