# Development Notes
Engineering rationale behind design decisions. The "why" that does not belong in code comments and is too detailed for `MEMORY.md`.

For the canonical statement of the algorithm and load-bearing decisions, see `book/src/`. This file is the contributor-facing scratchpad: rejected alternatives, calibration notes, gotchas, and the reasoning behind small choices the book does not cover.

---

## Core design decisions (recap)

These are documented in detail in the mdBook. They are restated here only as anchors:

- **Recursion is the core principle.** Every non-trivial generation step is a recursive descent over the typed circuit graph. Iteration is the exception, used only where termination or ordering genuinely require it (e.g., the flop worklist drainer, the per-output driver loop). When in doubt, recurse. See `book/src/core-idea.md` "The single guiding principle".
- **Synchronous-design discipline.** Every module is fully synchronous to a single clock domain: one `clk` (posedge), one `rst_n` (async, active-low), every flop emitted into one `always_ff` block. Enforced by construction — there is no IR field for per-flop clock or per-flop reset polarity. See `book/src/sequential.md` "Synchronous-design discipline".
- **Flop-D mux motifs.** Every flop's D input is constructed from one of: M=0 (direct cone), M≥2 OneHot (OR-of-masked arms), M≥2 Encoded (chained ternary over `Eq(sel, k)`). M=1 is excluded by design; it collapses to a wire. The style (OneHot vs Encoded) and kind (ZeroDefault vs QFeedback) are chosen per-flop and orthogonal — four motif variants plus the M=0 plain register. See `book/src/sequential.md` "Flop motifs".
- **Q-feedback freedom (revised).** A flop's own Q may appear freely — any number of times — as a leaf in any of its data, select, or direct-D sub-cones. The clock edge breaks the Q→D loop temporally; this is the standard synchronous feedback pattern (counters, accumulators, state machines). Independently, `FlopKind::QFeedback` adds an explicit Q fall-through term in the mux when no select fires. Both are legal; both can be active at the same flop. Combinational self-reference (Rule 1) is still forbidden. See `book/src/structural-rules.md` Rules 2 and 3.
- **Structural rules catalog.** Every load-bearing generator invariant is documented in `book/src/structural-rules.md`. That chapter is the durable source of truth — new rules land there as they become invariants. Inline design-decision recaps in this file should *point* to the catalog, not duplicate rule text.
- **Operators vs blocks.** Load-bearing conceptual distinction. An operator is an associative primitive function; its generalization is **arity** (N same-width operands). A block is a functional unit with internal structure; its generalization is **ports / port counts / arms**, encoding choices, feedback topology. Arity is operator vocabulary only — blocks have ports, not arity. `And / Or / Xor / Add / Mul` are operators and got N-arity in `2026-04-15-0015`. `Sub` is not associative and stays 2-arity. `Mux` and `Flop` are blocks and are governed by block rules, not arity knobs. See `book/src/structural-rules.md` "Operators vs blocks" preamble and Rule 14.
- **Roles of constants in RTL.** Integer literals appear as operands with three *distinct* semantic roles: **coefficient** (multiplicative weight in arithmetic linear combinations; per-op constraints: Add `ci ≠ 0`, Sub `ci > 0` strictly positive, Mul TBD), **shift amount** (structural parameter of `Shl/Shr` — `a << 2`; constant-amount vs variable-amount are both legal, with real designs biased heavily toward constant), and **comparand** (threshold / sentinel on the RHS of a comparison — `a == 7`; additive to signal-vs-signal comparisons, not a replacement). These three are *not interchangeable*: each has its own motif family, its own constraints, and its own knob(s). Do not unify them under a single `constant_prob` knob — doing so loses the semantic distinctions. See `book/src/structural-rules.md` "Roles of constants in RTL".
- **Construction strategies.** Three live strategies construct a
  module's internal logic: `sequential` (per-output cone recursion in
  declaration order), `shuffled` (same, randomised output order), and
  `interleaved` (frames interleaved via random-pop work queue — cones
  grow in lockstep). `graph-first` remains as a deprecated CLI/config
  alias for `interleaved`; the original speculative pool-growth
  implementation is retired. The strategy is a property of **how** the
  generator builds; the emitted SV is a DAG regardless. Different
  strategies produce different output *distributions*
  (declaration-order bias, within-module sharing symmetry). See
  `book/src/construction-strategies.md`.
- **Circuit IR over annotated EBNF.** The generator builds a typed circuit graph and emits SV from it. See `book/src/why-not-grammar.md`.
- **Generation by construction, not generate-then-filter.** Validity is structural; the validator is a safety net, not a gate. See `book/src/by-construction.md`.
- **Synthesizability is a subset constraint.** The gate set, flop
  pattern, and emitter cover only the synthesizable subset. Broader
  artifact families must keep that contract too; the project is
  broadening to more kinds of valid-by-construction synthesizable
  artifacts, not abandoning synthesizability. See
  `book/src/synthesizability.md`.
- **Non-triviality via dep-set tracking + structural anti-collapse
  rules.** No bundled oracle. Expected-facts manifests for specific
  artifact families are acceptable; a shadow simulator used as a global
  filter is not. See `book/src/non-triviality.md`.
- **Random by-construction synthesizable RTL is the product goal.**
  `anvil` is not trying to be merely "valid enough". The target is a
  signoff-level quality random synthesizable RTL generator whose outputs
  are accepted by mainstream downstream HDL consumers by default and
  remain rich enough to expose real bugs in parsers, elaborators, RTL
  compilers, linters, simulators, synthesizers, and similar tools.
  Feature growth and downstream-acceptance robustness are both
  first-class; neither is optional garnish for the other.
- **No oracle, no reference simulator.** `anvil` is still a generator,
  not a bundled shadow simulator. It can stress downstream tools by
  emitting high-quality legal RTL and explicit expected-facts contracts
  where appropriate, not by embedding a second implementation of RTL
  semantics. See `book/src/non-goals.md`.

If you need to revise any of these, that is a deliberate task with its own commit and a `DEVELOPMENT_NOTES.md` entry.

---

## Design notes
### Multi-clock + CDC primitives design (2026-05-24, MULTI-CLOCK-CDC.1)

Research-only slice (no code; `.2`+ implement). `MULTI-CLOCK-CDC.1`
opens the only remaining named follow-up tree on the repo after
`DIFFERENTIAL-SIMULATION` closed `2026-05-24`. Per the proven
Phase-7/8/9 + `DIFFERENTIAL-SIMULATION.2a`/`.3a` design-first
discipline: the IR extension (per-flop clock + per-flop reset),
the CDC primitive catalogue, the by-construction rule, and the
downstream-tool gate are all load-bearing structural decisions to
settle before code. Multi-clock CDC touches the most load-bearing
ANVIL invariant (`book/src/sequential.md` "Synchronous-design
discipline": "Every module is fully synchronous to a single clock
domain"), so the design-first slice is mandatory.

**Goal.** Generate modules with N≥2 declared clock domains whose
inter-domain signals are wrapped by-construction in a CDC
primitive (2-flop synchronizer at minimum); every emitted
multi-clock module passes the chosen downstream-tool CDC check
(Verilator `--cdc=metastable` is the first-cut candidate) and
shows cross-simulator agreement under
`tool_matrix --diff-sim` on a synchronised stimulus.

**CDC primitive catalogue — first-cut scope.** The IEEE CDC
literature names ~7 patterns; ANVIL adopts them in priority order:

| Tier | Primitive | First cut? | Notes |
| --- | --- | --- | --- |
| 1 | **2-flop synchronizer** (1-bit) | **Yes** | The minimum-viable CDC building block. Every 1-bit signal crossing domain A → domain B is two flops registered in B's domain; the metastability is captured + resolved by the second flop. Covers ~80% of real CDC paths. |
| 2 | N-flop synchronizer (1-bit) | Deferred (`.5` or follow-up) | Same as tier-1 with N≥3 flops; needed for very-high-speed paths where 2 flops is insufficient. Adds a knob, not a structural change. |
| 3 | Async FIFO (multi-bit) | Deferred (own tree) | Major structural change: depth, gray-code pointers, empty/full handshake, separate read/write domains. Phase-sized. |
| 4 | Gray-code pointer transfer | Deferred (own tree) | Foundation for async FIFO; gray code's single-bit transition prevents pointer corruption mid-flight. |
| 5 | Req/ack handshake (multi-bit) | Deferred (`.6` or follow-up) | 4-phase or 2-phase handshake for word transfer; smaller than FIFO but still structural. |
| 6 | Pulse synchronizer | Deferred (`.7` or follow-up) | Toggle + 2-flop sync + XOR; transfers an event across domains. |
| 7 | Reset synchronizer | Deferred (`.4` or follow-up) | Async-assert + sync-deassert; each domain gets its own. |

**Tier 1 (2-flop synchronizer)** is the minimum viable cut.
The deferred tiers either reuse tier-1 mechanically (N-flop)
or are large enough to warrant their own task tree (FIFO,
handshake, gray code). Per `feedback_full_factorization.md`
and `feedback_rules_first_generation.md`: when the generator
makes a domain-crossing decision, the synchronizer wrap is
issued by-construction — there is never a "generate the path
then check for synchronizer" filter pass.

**Minimum-viable IR extension.** The single-clock invariant lives
in `Module.clock: Option<Port>` and `Module.reset: Option<Port>`
(single reserved slots) plus the `always_ff @(posedge clk or
negedge rst_n)` template in `src/emit/sv.rs`. Two surface IR
changes:

- **Multi-domain Module shape.** `Module.clock_domains:
  Vec<ClockDomain>` where each `ClockDomain` carries
  `{ clk_port, rst_n_port, name }`. The existing single-domain
  Module continues to exist as the K=1 special case
  (`clock_domains.len() == 1` with `name = "default"`); this
  keeps the by-construction default behavior byte-identical
  unless a multi-clock knob fires. The existing `Module.clock`
  / `Module.reset` accessors stay (delegate to
  `clock_domains[0]`) so callers that don't care about
  multi-clock see no change.
- **Per-flop domain tag.** `Flop.domain: usize` (index into
  `Module.clock_domains`) — every flop knows which domain it
  belongs to. The emitter groups flops by domain and produces
  one `always_ff` block per (domain, polarity) tuple. The
  Phase-1 doctrine "one `always_ff` per module" is preserved
  for K=1; for K=N it generalises to "one `always_ff` per
  domain", which is the standard SV idiom.

The IR extension is backward-compatible. Existing modules with
no multi-clock knob fire stay K=1 with `domain = 0` for every
flop, and the emit is byte-identical.

**By-construction rule** (`book/src/structural-rules.md`, new
Rule for multi-clock). When the generator emits a flop in
domain B whose D-cone references a flop output in domain A,
the cone is rewritten to dereference a **2-flop synchronizer**
in domain B instead — that is, the flop sees `Synchronizer{
src_flop_q, dst_domain }` as its operand, never the bare
cross-domain flop output. The synchronizer is two newly-minted
flops, both in dst_domain. The rule fires at *construction
time*; there is no post-pass filter. The bookkeeping that
discovers domain-crossing operands is `Flop.domain` + the
cone-recursion that ANVIL already does.

This is exactly the rules-first generation pattern
(`feedback_rules_first_generation.md`): we never generate an
unsynchronised cross-domain path then filter it out; the rule
**constructs** the synchronizer in place.

**Downstream-tool gate.** Two candidates, evaluated:

- (a) **`verilator --cdc=metastable`** — a Verilator linter
  flag that flags cross-clock-domain paths without registered
  synchronizers. Pros: already integrated with the
  `tool_matrix` Verilator column; one flag toggle. Cons:
  experimental Verilator feature, may have false positives /
  miss real bugs. First-cut choice.
- (b) **`yosys read_verilog -cdc`** — explored: Yosys doesn't
  have a built-in CDC check in stable 0.64; the `-cdc` flag
  is project-folklore that doesn't exist. Rejected.
- (c) **Custom oracle.** ANVIL is a generator; we can record
  every constructed synchronizer in a manifest and emit a
  matching `cdc_manifest.json`, then assert the manifest
  matches what Verilator's linter reports. Defers to `.4`
  once `.3` lands. This mirrors the Phase-7 parity oracle
  pattern.

**Cross-simulator agreement (`tool_matrix --diff-sim`).** The
just-landed `.3b.2` `--diff-sim` column trivially extends to
multi-clock: the testbench drives multiple clocks (independent
periods) and stimulates inputs in each source domain;
outputs are sampled in their declared domain. For the *first
real-tool gate* on multi-clock, we sample only domain-B
outputs at domain-B sample points (a "synchronised stimulus"
flow) — this avoids the metastability-glass-jaw problem
where a transition mid-sync-flop produces different
trace-line values in iverilog (4-state) vs verilator
(2-state). Sequential domain-A→B paths with proper
synchronizers will produce byte-equal traces in both sims by
the cycle-accurate `@(negedge clk_B)` sample.

**Rejected alternatives.**

- (A) **Single-flop synchronizer.** Rejected — even 1-bit
  cross-domain paths need ≥2 flops to resolve metastability
  per standard CDC literature. A single flop is not a
  synchronizer.
- (B) **Clock-gating-instead-of-multi-clock.** Rejected — ICG
  is a power-optimisation concern, orthogonal to CDC. ANVIL's
  stance is "emit always-on flops; let downstream insert ICG".
- (C) **Latches for level-sensitive crossing.** Rejected —
  ANVIL's synchronous-design discipline forbids latches
  (`book/src/sequential.md`).
- (D) **Async-FIFO as the minimum viable cut.** Rejected —
  too large for the first multi-clock slice. FIFO requires
  gray-code pointer + handshake + depth; pushes outside the
  by-construction `.2`/`.3` envelope. Lands in its own
  follow-up tree.
- (E) **Generate-then-filter** (synchronizer-or-bust
  post-pass). Rejected — violates
  `feedback_rules_first_generation.md`. The synchronizer
  must be constructed in place.
- (F) **Dynamic frequency / dynamic clock ratios.** Rejected
  — the IR records a fixed declared frequency per port (or
  just a domain-name tag); runtime-dynamic frequency is a
  testbench concern, not a generator concern.

**Leaf shape.** `.2` implements the IR extension (multi-domain
`Module`, per-flop `domain`, 2-flop synchronizer construction
rule, emitter); `.3` adds the downstream-tool gate (Verilator
`--cdc=metastable`) and the matrix wiring (`--multi-clock-prob`
knob, `saw_multi_clock_design` + `saw_cdc_2_flop_synchronizer`
coverage facts); `.4` documents the contract (README +
USER_GUIDE + `book/src/sequential.md` updates removing the
"Multi-clock deferred" caveat).

**Knob shape.** Single `--multi-clock-prob: f64` per-module
roll (defaults to `0.0` for byte-identical backward
compatibility). When fired, the generator picks `N` from
`--num-clock-domains-min`/`--num-clock-domains-max` range
(defaults `2..=2` — start simple). Per-module roll because
hierarchy is orthogonal: a multi-clock parent may have
single-clock children or vice versa; the generator handles
this generically via `Flop.domain`.

This entry is design-only and is itself task-tree owned
(`MULTI-CLOCK-CDC.1`); it makes no code change, consistent
with the task-tree-ownership doctrine's code/not-code
boundary.

### Tool-matrix `--diff-sim` wiring + representative-subset selector + coverage fact design (2026-05-24, DIFFERENTIAL-SIMULATION.3a)

Design-only slice (no code; `.3b` implements). `.3` split mirrors
the proven Phase 7/8/9 design-first discipline + the
`PHASE-7-ORACLE-MICRODESIGN.2c.2a`/`.2c.2b` precedent: the
module-extraction decision, the CLI shape, the subset selector,
and the coverage-fact wiring are load-bearing choices to settle
before code; the design itself is docs-only.

**Goal.** Wire the `tests/diff_sim.rs::emit_testbench` +
`run_iverilog` + `run_verilator` + `normalize_trace` machinery
landed in `.2b.2` into `src/bin/tool_matrix.rs` as an opt-in
`--diff-sim` mode, so the matrix records cross-simulator semantic
agreement per scenario alongside its existing parse/synth/lint
columns. A new `saw_design_with_cross_simulator_agreement`
coverage fact fires when at least one DUT in the run achieves
byte-equal post-reset traces.

**Module-extraction decision (the structural choice that justified
splitting `.3`).** The harness helpers currently live in
`tests/diff_sim.rs` and are NOT exported from the `anvil` library
crate — `src/bin/tool_matrix.rs` cannot reach them today. Two
options:

- (A) **Extract to `src/diff_sim/mod.rs`** (library module).
  `tests/diff_sim.rs` switches to `use anvil::diff_sim::{…}`;
  `src/bin/tool_matrix.rs` does likewise. Full-factorization
  doctrine satisfied (one home for the testbench emitter +
  orchestration). Cost: one module move + two `use` updates.

- (B) **Duplicate the helpers in `tool_matrix.rs`** (or copy
  paste). Violates the full-factorization doctrine
  (`feedback_full_factorization.md`) — two homes for the
  testbench-emitter code, divergence inevitable. Rejected.

`.3b` takes (A). The new `src/diff_sim/mod.rs` exports
`baked_input_vectors`, `mask_to_width`, `fmt_sv_hex`,
`is_sequential`, `emit_testbench`, `run_iverilog`, `run_verilator`,
`normalize_trace`, `tools_present`, plus a thin façade
`run_differential(top: &Module, vectors: &[Vec<u128>], work_dir:
&Path) -> Result<DiffOutcome, DiffError>` that orchestrates the
whole flow (emit testbench → emit DUT SV → invoke iverilog →
invoke verilator → normalize + compare → return aligned
traces/diff). The façade is what `tool_matrix.rs` calls per
scenario.

**CLI flag shape.** New `--diff-sim` opt-in flag on `Cli` (mirrors
the existing `--skip-verilator`/`--skip-yosys` opt-out flags and
the `--phase4-hierarchy-gate` opt-in elevation flag). Default:
`false`. When set: every scenario in the selected scenario set
runs the differential harness AFTER the existing parse/synth/lint
columns succeed (gated on Verilator AND Yosys both clean — no
point asking simulators to agree on output that one tool already
rejected). The flag is orthogonal to `--phase4-hierarchy-gate` /
the other gate-elevation flags; it adds a new column, it does not
change which scenarios run.

**Representative-subset selector.** The full 204-scenario matrix
is computationally infeasible for the differential harness
(per-design wall-clock cost: ~5-10 s for iverilog +
~10-20 s for verilator compile+run = ~20 s/scenario × 204 ≈
68 min just for diff-sim). Three options for subset selection:

- (1) **`--diff-sim-subset <integer>`** — randomly sample N
  scenarios (seeded). Simple; reproducible; representative of
  the distribution. Default `N=5`. Rejected: random sampling
  loses the curated coverage structure (e.g., always picking 5
  combinational misses sequential coverage).

- (2) **Hand-curated subset** — a fixed list of scenario names
  (e.g., `["minimal-comb", "minimal-seq", "phase4-hier-comb",
  "phase4-hier-seq", "phase6-fsm-leaf"]`). Coverage-aware
  (one per major axis). Rejected: brittle — every new
  scenario-set requires updating the list; doesn't scale with
  `Phase4Hierarchy`/`Phase3Structured` etc.

- (3) **Per-axis sampling** — for the selected scenario set,
  pick the first scenario that satisfies each major coverage
  axis (combinational, sequential-flop, hierarchy, memory,
  fsm), capped at K=5. Coverage-aware AND self-maintaining.
  **Chosen for `.3b`.** Selection is deterministic (first match
  per axis in scenario-set declaration order), reproducible,
  and naturally adapts as new scenarios land.

The selected subset is recorded in the matrix report under
`diff_sim_subset: Vec<String>` (scenario names) so the report
itself is self-describing.

**Coverage-fact wiring.** New `saw_design_with_cross_simulator_
agreement: bool` field on `CoverageSummary` (alongside the
existing `saw_inferrable_memory_design`/`saw_fsm_design` from
Phase 6). Fires when at least one DUT in the subset achieves
byte-equal post-reset traces. Merged into the aggregate `dst |=
src` per the existing pattern at `tool_matrix.rs:5847`.
`--diff-sim` is NOT a gate-elevation flag by default (the matrix
will not exit non-zero if the fact is false unless
`--fail-on-coverage-gap` is set AND `--diff-sim` is set — the
existing opt-in coverage-gap semantics, no new flag needed).

**Per-scenario report shape.** New optional field on
`ModuleReport`: `diff_sim: Option<DiffSimReport>`. `DiffSimReport`
records: `ran: bool` (was this scenario in the subset?),
`success: bool` (byte-equal post-reset traces?), `n_samples:
u32` (sample count), `iverilog: Option<ToolInvocation>`,
`verilator: Option<ToolInvocation>`, `mismatch_excerpt:
Option<String>` (first 10 lines of the diff, retained per the
Phase-7 counterexample doctrine — never a silent pass).
`tools_present()` guard makes the column a friendly no-op when
either simulator is absent (the column reports `ran: false`
with a clear reason; matrix exits clean).

**Wiring point in `tool_matrix.rs`.** Inserted as a new
per-module step in the existing per-module pipeline, AFTER
Verilator + Yosys (the existing tools) and BEFORE checkpoint
write (so a `--resume` re-run replays the diff-sim column from
checkpoint without re-invoking the simulators). Gated by
`cli.diff_sim` AND scenario presence in the subset AND
Verilator+Yosys both clean — the existing "downstream tools
already accepted the SV" precondition.

**Rejected alternatives.**

- (i) **`--diff-sim` as a gate-elevation flag** (always
  required to pass) — rejected: the simulator runtime is too
  large to gate-mandatorily in CI; the existing `--phase4-
  hierarchy-gate` already takes ~75 min, and a mandatory
  diff-sim column on top would push the gate over 2 h. Opt-in
  with explicit `--fail-on-coverage-gap` is the right
  trade-off.

- (ii) **Duplicate the helpers in `src/bin/tool_matrix.rs`** —
  rejected per the module-extraction discussion above
  (full-factorization doctrine).

- (iii) **Random subset sampler** — rejected per the
  per-axis-sampling discussion above (loses curated coverage
  structure).

- (iv) **Hand-curated subset** — rejected per the
  per-axis-sampling discussion above (brittle, doesn't scale).

- (v) **Move `tests/diff_sim.rs` entirely** (delete the file,
  put the gated tests inside `src/diff_sim/mod.rs` as
  `#[cfg(test)]`) — rejected: separation of library API surface
  from the gated integration tests is the established convention
  (cf. `tests/microdesign_parity.rs` consumes
  `src::microdesign::*`, `tests/frontend_parity.rs` consumes
  `src::frontend::*`). Library exports the API; the integration
  test owns the `#[ignore]` gates.

**Proof shape (`.3b`).** `cargo fmt`/clippy(-D warnings)/check/
test all clean. New `src/diff_sim/mod.rs` carries the extracted
helpers + the `run_differential` façade. `tests/diff_sim.rs`
updated to `use anvil::diff_sim::{…}` (no logic change). New
`src/bin/tool_matrix.rs` `--diff-sim` flag + per-module wiring
+ subset selector + `saw_design_with_cross_simulator_agreement`
coverage fact + `DiffSimReport` per-module field + merge into
aggregate. Cargo-portable proofs: subset selector picks one per
axis deterministically; coverage fact merges correctly; CLI
parse smoke. Tool-gated `#[ignore]` proof: end-to-end
`tool_matrix --diff-sim --base-seed 0 --modules-per-scenario 1
--out /tmp/anvil-diff-sim-p1` exits 0 with
`saw_design_with_cross_simulator_agreement=true` on a machine
with both simulators installed. `.4` documents the contract.

This entry is design-only and is itself task-tree owned
(`DIFFERENTIAL-SIMULATION.3a`); it makes no code change,
consistent with the task-tree-ownership doctrine's code/not-code
boundary.

### Book-examples-runnable design (2026-05-18, BOOK-EXAMPLES-RUNNABLE.1)

Design-only slice. No code. The repo is now public with the mdBook
live at `https://rdje.github.io/anvil/`; every example is a
copy-paste contract with users. This entry inventories the
fenced-block reality and designs the convention migration + the
CI-gated drift-proof harness, so `.2` has an unambiguous target.

**Fenced-block inventory (audited `book/src/*.md`).** 62 ```bash`` ,
8 ```rust`` , 9 ```systemverilog`` , 4 ```text`` .

- **`bash` (62) — the runnable copy-paste surface.** `recipes.md` 41,
  `tutorial.md` 10, `getting-started.md` 6, `knobs.md` 2,
  `factorization.md`/`faq.md`/`introduction.md` 1 each. Leading
  tokens: ~44 lines start with bare `anvil`, ~24 with `cargo`, plus
  `\`-continued multi-line commands and `| …` pipes. ~58 bare-`anvil`
  occurrences total. **Defect:** bare `anvil …` is not runnable from
  a fresh clone (no binary on PATH) — `getting-started.md` already
  uses `cargo run --release --`, the rest don't. This is the core
  break the owner flagged.
- **`rust` (8) — illustrative IR/struct sketches**, not programs:
  `ir.md` 3, `hierarchy.md` 3, `architecture.md` 1, `knobs.md` 1.
  Partial (reference internal types, no imports/`fn main`) → would
  fail `mdbook test` if treated as doctests.
- **`systemverilog` (9) + `text` (4) — emitted-output samples**, not
  commands; never executed, but some directly follow a command as
  its shown output.

**Owner decisions (2026-05-18), recorded in the tree:** (1) runnable
blocks standardize on **`cargo run --release --`** (+ one optional
`cargo install --path .` → `anvil` shorthand note); (2) correctness
is **CI-gated** via an extraction harness + `mdbook test`, not a
one-time audit.

**Architecture — chosen: a `cargo test` integration harness +
`mdbook test`, both in CI.**

1. **Convention migration.** In every runnable `bash` block, the
   command head `anvil ` → `cargo run --release -- ` (preserving
   `\`-continuations, `| …` pipes, and redirections). One shorthand
   note (getting-started + knobs reference): "`cargo install --path
   .` once, then use `anvil` instead of `cargo run --release --`".
2. **Skip marker for genuinely illustrative bash.** Default = run.
   A block opted out with an HTML-comment sentinel on the line
   immediately before the fence: `<!-- book-test: skip — <reason>
   -->`. HTML comments don't render in mdBook output and aren't in
   the copy-paste body, so users never see noise; the harness keys
   off it. Reason string is mandatory (no silent skips).
3. **Harness = `tests/book_examples.rs` (cargo integration test).**
   Not a CI-only shell script (that would drift from the `cargo
   test` gate — the project convention is *everything is
   `cargo test`-gated*; CI already runs `cargo test`). It: walks
   `book/src/*.md`; parses ```bash`` fences; honours the skip
   sentinel; builds the binary once (`cargo build --release`, then
   invokes `target/release/anvil` so per-example cost excludes
   rebuild); runs each block in a fresh temp CWD with a per-command
   timeout and `CARGO_NET_OFFLINE=true` (anvil examples are fully
   local — no network); asserts exit 0. Where a ```text`` /
   ```systemverilog`` / ```console`` block immediately follows a
   command block and is tagged asserted, compare: seed-stable
   commands (anvil is reproducible by `--seed`) → exact match;
   tool-version-sensitive → shape/prefix match. Untagged output
   blocks are documentation only (not asserted) — recorded so a
   future contributor doesn't assume all output is checked.
4. **`mdbook test` + rust sketches.** Annotate the 8 ```rust`` blocks
   `rust,ignore` (still rendered in the book; not compiled), so
   `mdbook test book` is green and *meaningful*: any future real
   ```rust`` example is compiled, sketches are explicitly exempt.
5. **CI.** `tests/book_examples.rs` runs under the existing `cargo
   test` step in `.github/workflows/ci.yml`; add an `mdbook test
   book` step. Both gate `main`.

**Rejected alternatives.** (A) Rust `doctest`/`mdbook test` only —
covers just the 8 ```rust`` sketches, **not** the 62 ```bash``
blocks that are the actual copy-paste surface; leaves the real
defect unenforced. (B) A standalone CI-only `.sh` extractor — works
on GitHub but is invisible to local `cargo test`, so it drifts from
the COMMIT.md gate and a contributor can't reproduce a failure
locally with one command; violates the "everything is
`cargo test`-gated" project convention. (C) Generate the book
examples *from* tests (golden-doc) — strongest anti-drift but a
large restructuring of authored prose, fights the book-doctrine's
hand-written friendly voice, and is disproportionate; the
extraction harness gets ~all the safety at a fraction of the churn.

**Proof shape (`.2`, expected to split).** Harness enumerates ≥ the
runnable-block count, builds once, runs each against the fresh
binary, all exit 0; tagged sample outputs match; `mdbook test book`
green; CI gates on `cargo test` (incl. `book_examples`) + `mdbook
test`; a deliberate broken example fails the harness (negative
control); book meaning unchanged (only invocation normalised + the
one shorthand note). Split candidates: harness impl / the ~62-block
migration / CI wiring (independently reviewable).

This entry is design-only and is itself task-tree owned
(`BOOK-EXAMPLES-RUNNABLE.1`); it makes no code change, consistent
with the task-tree-ownership doctrine's code/not-code boundary.

**As-built resolution (2026-05-18, `.2.2` — tree CLOSED).** The
harness landed as `tests/book_examples.rs` essentially per design.
One implementation deviation worth recording: blocks are run with the
**shell-script model** (`bash -eu -o pipefail`, one child per block,
`cargo run --release --` text-substituted to `"$ANVIL"` = the
once-built release binary) rather than a parsed-command model — this
is what makes `$()`/for-loop/`# comment` blocks runnable verbatim,
and a classification guard *panics* on any unclassified residual so a
silent gap is impossible.

**Non-obvious gotcha (cost a full debugging cycle — record
permanently).** The first full runs reported 12 blocks "TIMED OUT
after 600 s". This was **not** a book defect: a default `--seed 42`
module is ≈86 KB on stdout (a 5-level `factorization` sweep ≈525 KB),
but the OS pipe buffer is ≈64 KB. The original `run_script` used
`Stdio::piped()` and a `try_wait()` poll loop that **never drained
the pipe until after the child exited** — so `anvil` blocked forever
in `write()`, the child never exited, and the loop spun to the
timeout (12 × 600 s ≈ the observed 7273 s total). Directly invoking
each "timed-out" command proved the examples are correct (0.03–0.15 s
each). **Rule for any future child-process harness here: never pair
`Stdio::piped()` with a non-draining wait loop for a child whose
output can exceed ~64 KB.** Fix chosen: redirect child stdout/stderr
to temp **files** (no buffer limit, std-only, no reader-thread
plumbing) and reap the child after a timeout kill. Post-fix:
`cargo test --test book_examples` = 3/3, 54 runnable blocks exit-0,
9 skip-sentineled, 76.4 s (down from 7273 s). The `PER_BLOCK_TIMEOUT`
is now purely a defensive backstop, not a hot path. CI gates this via
`cargo test` + the added `.github/workflows/ci.yml` `mdbook test
book` step.

### Phase 6 inferrable-memory motif design (2026-05-18, PHASE-6-ADVANCED-MOTIFS.1)

Design-only slice. No code. Lifts `book/src/ir.md` "Synthesizable
aggregates → Unpacked arrays (the memory-inference pattern)" /
ROADMAP Phase 6 into a concrete, codebase-grounded plan with an
empirical Yosys-inference probe, a chosen architecture, rejected
alternatives, and a proof shape, so the implementation leaf
(`PHASE-6-ADVANCED-MOTIFS.2`) has an unambiguous target.

**Goal (from ROADMAP Phase 6 / book).** Emit inferrable memory
motifs (single-port and simple dual-port; inferrable patterns only),
valid by construction, downstream-clean (Verilator + both Yosys
modes), and **recognised as memory by Yosys** (`$mem_v2` after the
memory pass). Default-off / byte-identical; never retire existing
behaviour; single-clock discipline preserved.

**Empirical Yosys-inference probe (resolves the tree's Open
Question).** Two LRM-synthesizable templates were probed in `/tmp`
through Verilator and **both** repo Yosys modes:

- **Single-port** — `logic [DW-1:0] mem [0:2**AW-1]`;
  `always_ff @(posedge clk) begin if (we) mem[addr] <= wdata;
  rdata <= mem[addr]; end`.
- **Simple dual-port** — same array, one write port (`waddr`/`we`)
  and one independent read port (`raddr`), synchronous read.

Result: `read_verilog -sv; proc; opt; memory_collect; stat` yields
**exactly `1 $mem_v2`** for *both* templates; `verilator
--lint-only` exits 0; `synth -noabc; check -assert` and `synth;
abc -fast; check -assert` both exit 0 with no `ERROR`. Conclusion:
both shapes are reliably memory-inferred and downstream-clean across
the repo's exact gate toolchain. (Plain `synth` then maps tiny mems
down to FFs for the final netlist — expected with no BRAM target;
the *inference* is what Phase 6 asserts, captured at the
`memory_collect` stage, not the post-`memory_map` cell mix.)

**Code reality that constrains the design** (audited; key anchors).
The IR has **no array/memory concept**: `Port` (`src/ir/types.rs`),
every `Node::*` and `Flop` carry a scalar `u32` width; the only
stateful element is `Flop` (one shared `always_ff` per module,
`src/emit/sv.rs`); `Node` leaves are `PrimaryInput`, `Constant`,
`FlopQ`, `InstanceOutput`. Per the **operators-vs-blocks doctrine**
(`DEVELOPMENT_NOTES.md` "Core design decisions"), a memory is a
*block* (a functional unit with internal structure and its own
state/ports), not an operator and not a datatype — exactly like
`Flop`/`Mux`. The emitter is a dumb serialiser; blocks are
first-class motifs it renders verbatim.

**Architectural decision — chosen: (M) a first-class `Memory` block,
sibling to `Flop`, kept out of the NodeId expression graph.** A
memory is *state with identity by instance*, not an expression
(re-evaluating `mem[a]` is not pure), so — exactly as `Flop` is a
module-level element with a `FlopQ` leaf, not a `Gate` — Phase 6
adds:

1. A `Memory` element on `Module` (additive `Vec<Memory>`, `Default`
   empty ⇒ zero churn to `..Module::default()` sites, the proven
   Phase 5/5b additive pattern): `{ id, addr_width, data_width,
   kind: SinglePort | SimpleDualPort, write port (we/addr/data
   source `NodeId`s), read port(s) }`.
2. A new gate-graph **leaf** `Node::MemRead { mem, ... }` (sibling to
   `FlopQ`) so a memory read result can feed cones without the array
   itself ever entering combinational factorization (a `MemRead`
   leaf is opaque to CSE, like `FlopQ` — it is identity-by-instance,
   never merged with anything).
3. Emitter: render the **empirically-validated inferrable template**
   verbatim (`logic [DW-1:0] mem_k [0:2**AW-1]` + the synchronous
   write/read `always_ff`), wired to the existing `clk`. Validator:
   address/data widths consistent; read leaves resolve to a declared
   memory.
4. Opt-in `Config::memory_prob` (`f64`, serde-default `0.0`,
   probability-range validated — the Phase 5/5b knob pattern).
   Default-off ⇒ no `Memory`, byte-identical for fixed seeds.

This keeps **valid-by-construction** (a rules-first generator block,
no post-hoc filter), preserves single-clock discipline (the memory
shares the module `clk`), and keeps the full-factorization doctrine
intact (the array is never a NodeId; `MemRead` is an opaque leaf).
It mirrors how `Flop` was integrated, which is the lowest-risk
precedent in the codebase.

**Rejected alternatives.**

- **(A) Model memory as a register file of `Flop`s + address mux.**
  Rejected: Yosys does **not** infer a flop-array + mux as `$mem`
  (the probe's whole point) — it would defeat Phase 6's purpose
  (memory-inference stress) entirely, and explodes node/flop counts
  (2^AW flops + a 2^AW-way mux per read). It is not "a memory" to
  any downstream tool.
- **(B) Emitter-only string template with no IR representation.**
  Rejected: not valid-by-construction — the memory's write/read
  data sources must be real generated cones with dependency
  tracking and validation; a free-floating text template cannot be
  driven, validated, or factored, and is the post-hoc-template
  anti-pattern the project forbids (rules-first construction).
- **(C) A generic unpacked-array *datatype* threaded through
  `Port`/`Node` width arithmetic.** Rejected: memory is a *block*,
  not a datatype (operators-vs-blocks doctrine); threading an
  array type through every width check / CSE key / validator /
  emitter is a massive invasive change for zero gain over (M),
  and conflates two orthogonal concepts. (M) confines memory to a
  new block + one opaque leaf, exactly as `Flop` is confined.

**Proof shape (for `.2`).** (1) Default-off byte-identical for fixed
seeds across all `ConstructionStrategy` values (no `Memory` ⇒
identical `to_sv`). (2) Forced-on: a focused proof that a generated
memory module emits the inferrable template, `validate_design`
passes, and **Yosys `memory_collect` reports ≥1 `$mem_v2`** in both
repo modes (the inference assertion — the Phase 6 contract), plus
`verilator --lint-only` clean. (3) A `tool_matrix`
`phase6_inferrable_memory` scenario shaped like the dedup/phase5/5b
anchor (shape-coverage sets unperturbed) + `DesignMetrics
.num_memory_modules` + a `saw_inferrable_memory_design` coverage
fact/gap + non-vacuity test; **no ROADMAP promotion** until the real
repo-owned gate is run and verified clean (r87 no-aspirational-claims
— same `.2.x` decomposition as Phase 5/5b). (4) Full `cargo` hygiene
gate; `mdbook` reconciled (`ir.md` memory delivered note +
`knobs.md` `memory_prob`).

This entry is design-only and is itself task-tree owned
(`PHASE-6-ADVANCED-MOTIFS.1`); it makes no code change, consistent
with the task-tree-ownership doctrine's code/not-code boundary.

### Phase 6 generated-encoding FSM motif design (2026-05-18, PHASE-6-ADVANCED-MOTIFS.3.1)

Design-only slice. No code. Lifts ROADMAP Phase 6 "FSMs with
explicitly generated state encodings" into a concrete,
codebase-grounded plan with an empirical downstream-tool probe, a
chosen architecture, rejected alternatives, the `.3` proof shape and
split, so `.3.2`+ have an unambiguous target. Mirrors the proven
Phase 5 / 5b / `.1`-memory design-first method.

**Codebase grounding.** The IR has no state-machine concept. `Flop`
is the only stateful element; `Node` is a scalar `u32`-typed
expression graph; the operators-vs-blocks doctrine (established by
the `.1`/`.2.1` memory work) says a stateful, non-CSE-able motif is a
**block**, not an operator and not a datatype. A "generated-encoding
FSM" decomposes exactly into primitives the emitter already proves
synthesizable: (1) a **state register** — a flop holding the
encoded-state bits; (2) **combinational next-state decode** — a
`case (state_q)` over the generated state constants; (3)
**combinational output decode** — a Moore `case (state_q)`; (4)
**generated state constants** — `localparam` values whose width and
bit-pattern are fixed *by the chosen encoding*. The encoding choice
(binary / one-hot / gray) is the entire novel surface; everything
else is flop + comb logic ANVIL already emits valid-by-construction.

**Empirical downstream probe (resolves the open design question:
"is a generated-encoding FSM downstream-clean in both repo Yosys
modes + Verilator, for every encoding?").** Hand-wrote the exact SV
template ANVIL would emit (localparam state constants + `state_q`
flop with async-low reset + `always_comb` next-state `case` +
`always_comb` Moore output `case`) for a 4-state FSM in all three
encodings:

- **binary** (`state_q` width `ceil(log2 N)` = 2 bits; constants
  `2'd0..2'd3`),
- **one-hot** (`state_q` width `N` = 4 bits; constants `4'b0001`,
  `4'b0010`, `4'b0100`, `4'b1000`),
- **gray** (`state_q` width 2 bits; constants `00,01,11,10`).

Result — **all three are downstream-clean**: `verilator --lint-only
-Wall` exit 0; `yosys read_verilog -sv; synth -noabc; check -assert`
clean; `yosys synth; abc -fast; check -assert` clean — i.e. clean in
**both** repo-owned Yosys modes and Verilator. The state-register
width and constants differ by encoding (`[1:0]` for binary/gray vs
`[3:0]` for one-hot), so **"encoding selectable" is a structural
fact, not cosmetic** — exactly the ROADMAP Phase 6 requirement. (The
case-decode shape Yosys also recognises via its `fsm` pass, but that
is a bonus; the inference contract here is plain clean synthesis,
unlike memory whose contract was the `$mem_v2` template — an FSM is
"just" flop + comb logic, so the risk is encoding correctness, not
inferability.)

**Chosen architecture — (F): first-class `Fsm` block + opaque
`Node::FsmOut` leaf + generated-encoding emitter + opt-in knob.**
Mirrors the landed memory motif ((M)) so it reuses the proven
opaque-stateful-leaf pipeline integration:

1. **IR.** Additive `Vec<Fsm>` on `Module` (Default-empty → trees
   without FSMs are byte-identical). An `Fsm` carries: state count
   `N`, the chosen `FsmEncoding { Binary, OneHot, Gray }`, the
   per-state next-state transition table (indices into states,
   selected by a bounded input/condition cone), and the per-state
   Moore output value. State constants are *derived* from
   `(encoding, N)` at emit — never stored redundantly (full
   factorization: the encoding is the identity of the constants).
2. **Opaque leaf.** `Node::FsmOut { fsm: FsmId }` — a sibling to
   `FlopQ`/`MemRead`, **never CSE'd / never factorized** (the FSM is
   a block; its output is an opaque source like a flop's Q). Same
   `compact.rs` reachability obligation discovered in `.2.1a`: a
   reachable `FsmOut` must transitively keep the FSM's
   transition/condition source cones alive (sibling rule to
   `FlopQ`/`MemRead` keeping their D/we/addr cones).
3. **Emitter.** Renders the probed-clean template: generated
   `localparam` state constants per the encoding, the `state_q`
   flop on the shared `clk` with async-low reset to `S0`, the
   next-state `always_comb` `case`, the Moore output `always_comb`
   `case`. Single-clock invariant preserved (no new clock).
4. **Knob.** Opt-in `Config::fsm_prob` serde-default `0.0`
   (default-off ⇒ byte-identical), one roll in the same
   mutually-exclusive opt-in lane as `memory_prob` /
   `width_parameterization_prob` (rules-first `build_fsm_block`,
   never generate-then-filter).

**Rejected alternatives.** (A) **Build the FSM from existing
primitives** (a flop + a hand-rolled mux/`Eq` tree) with *no* block —
rejected: the state encoding is then implicit and *not selectable*,
the motif is unrecognisable as an FSM to a reader or to Yosys's
`fsm` pass, and it defeats the ROADMAP's *"explicitly generated
state encodings"* requirement. (B) **Emitter-only string template**
(no IR `Fsm`) — rejected: not valid-by-construction, can't be
validated, breaks the operators-vs-blocks doctrine the memory work
established. (C) **A generic `enum`/typedef datatype threaded
through the width/IR machinery** — rejected for the same reason
memory's (C) was: a massive invasive change to scalar IR arithmetic;
an FSM is a *block*, not a datatype. (D) **Mealy outputs (outputs a
function of state *and* input)** — deferred, not rejected: Moore-only
keeps the output decode a pure `case (state_q)` (matches the probed-
clean template and the deterministic-output contract); Mealy is a
recorded post-`.3` extension, not a `.3` blocker.

**Proof shape (`.3`, split mirrors `.2`).** `.3` becomes a container
mirroring the proven memory `.2.1`–`.2.4`:

- **`.3.1`** (this slice) — design; design-only, no code.
- **`.3.2`** — IR + opaque `FsmOut` leaf + `compact.rs` reachability
  + emitter + validator scaffold + `fsm_prob` knob + rules-first
  `build_fsm_block` (default-off byte-identical; forced-on focused
  proof). May sub-split `.3.2a`/`.3.2b` (IR-core+reachability /
  knob+generator) **if** implementing it surfaces a lower-level
  dependency, exactly as `.2.1` split on the compaction-reachability
  discovery — decided when reached, not pre-emptively.
- **`.3.3`** — cargo-portable proof (`tests/pipeline.rs`): across
  `ConstructionStrategy × FactorizationLevel × seeds`, the emitted SV
  is *exactly* the probed-clean per-encoding template, exactly one
  `FsmOut` survives every factorization level (CSE/EGraph-opaque),
  all three encodings are reachable and structurally distinct;
  `validate_design` clean; default-off byte-identical reaffirmed.
- **`.3.4`** — `phase6_fsm` matrix scenario + `num_fsm_modules`
  metric + `saw_fsm_design` fact/`Phase4Hierarchy` gap (no ROADMAP
  advance), then the **real repo-owned gate** verified downstream-
  clean (`coverage_gaps=[]`, Verilator + both Yosys all-pass,
  `saw_fsm_design=true`, P4/P5/P5b/P6-memory regressions clean)
  *before* any promotion (r87 no-aspirational-claims). FSM is the
  **last** Phase 6 motif: when `.3.4` verifies clean it both records
  FSM delivered **and** — memory already delivered at `.2.4` — closes
  ROADMAP Phase 6 and the `PHASE-6-ADVANCED-MOTIFS` tree (multi-clock
  CDC stays the explicitly-optional, separately-prioritised deferral
  per the 2026-05-16 Decision; not a Phase 6 blocker).

The cargo gate **cannot** shell Yosys/Verilator (project convention
since Phase 1); downstream cleanliness is proved by `.1`-style probe
(done, above) + the `.3.4` repo-owned `tool_matrix` gate, never in
`cargo test` — identical to how memory and Phase 5/5b were proved.

This entry is design-only and is itself task-tree owned
(`PHASE-6-ADVANCED-MOTIFS.3.1`); it makes no code change, consistent
with the task-tree-ownership doctrine's code/not-code boundary.

**As-built IR shape (2026-05-18, `.3.2a`).** `.3.2` was split up
front into `.3.2a` (IR core + opaque-leaf pipeline integration) and
`.3.2b` (knob + rules-first generator) — the opaque-stateful-leaf
compaction-reachability is correctness-critical pipeline code, known
concretely from the landed memory `.2.1a` (it is *not* mechanical
`FlopQ`-mirroring). `.3.2a` landed and fixes the concrete IR shape
the architecture-(F) sketch left open:

- `FsmEncoding{Binary,OneHot,Gray}` owns the encoding maths:
  `state_width(N)` = `ceil(log2 N)` for Binary/Gray, `N` for OneHot;
  `state_const(s)` = `s` / `1<<s` / `s ^ (s>>1)`. The state constants
  are **derived**, never stored (full factorization: the encoding is
  the identity of the constants).
- `Fsm { num_states, encoding, sel:NodeId, sel_width,
  transitions:[N][1<<sel_width], outputs:[N], out_width }`. A single
  generated `sel` cone drives the next-state decode
  (`next = transitions[state][sel]`); Moore outputs are a per-state
  value table. Reset state is index 0. This is the minimal shape
  that is valid-by-construction and downstream-clean per the `.3.1`
  probe; richer transition conditions are a post-`.3` extension, not
  a `.3` blocker (recorded, like Mealy).
- Emitter detail worth recording: state `localparam`s are emitted
  **per-FSM-prefixed** (`FSM<id>_S<k>`), not the probe's bare `Sk`,
  so multiple FSMs in one module never collide; they are emitted in
  module body just before the FSM `always` blocks (LRM-legal;
  Verilator/Yosys-accepted; the authoritative tool re-verification
  is the `.3.4` repo gate, exactly as for memory).
- Default-off is **trivially** byte-identical: the emitter blocks are
  gated on `!m.fsms.is_empty()`, the predicates only OR when `fsms`
  is non-empty, and the `FsmOut` match arms only fire when a `FsmOut`
  node exists — none of which occur without the (`.3.2b`) generator.

`.3.2b` landed the generator/knob: new calibration knob
`Config::fsm_prob` (`f64`, serde-default `0.0`, probability-range
validated — the same shape as `memory_prob`/`aggregate_prob`/
`width_parameterization_prob`). Rules-first `build_fsm_block`
constructs the FSM leaf *by rule* (it is never a generate-then-filter
— `num_states`/`encoding`/`sel_width`/`out_width` are rolled via
`g.rng` for reproducibility; transitions and distinct masked Moore
outputs are filled deterministically). The opt-in roll is a single
`g.rng.gen_bool` in `generate_leaf_module_with_interface_profile`,
placed **after** the Phase 5 width-parameterization lane and the
Phase 6 memory lane and therefore **mutually exclusive** with both —
the established Phase-5/5b/6-memory opt-in-lane discipline (one
exclusive motif per free-standing single-module design;
`interface_profile.is_none()` only; default-off never enters, so
emission is byte-identical). This keeps the four opt-in motif lanes
(param / aggregate-via-annotation / memory / FSM) from interacting.

### Phase 7 oracle-backed micro-design artifact family design (2026-05-18, PHASE-7-ORACLE-MICRODESIGN.1)

Design-only slice. No code. Lifts ROADMAP Phase 7 ("oracle-backed
micro-design artifacts — `rtl_const_expr`-style corpora") into a
concrete, codebase-grounded plan: the expected-facts schema, the
oracle-by-construction generation strategy, the reproducibility
contract, the parity-check harness shape, the boundary with the
existing DUT lane and Phases 8/9, rejected alternatives, and the
`.2` proof shape + split. Mirrors the proven Phase 5/5b/6
design-first method.

**The conceptual shift (why this is a new family, not a knob).**
Phases 1–6 generate *structurally valid random RTL* whose function
is deliberately meaningless — the contract is "lints/elaborates/
synthesizes clean", there is **no semantic oracle** ("structural,
not meaningful" — `book/src/non-triviality.md`). Phase 7 is the
**opposite**: tiny `.sv` files whose *elaboration facts are exactly
known by construction*, shipped with a machine-checkable manifest,
so a downstream tool can be checked against an **oracle** (does the
tool resolve this parameter / width / generate-branch to the value
ANVIL already knows?), not merely "did it not error". Pressure point
= front-end constant-expression / parameter / elaboration
correctness, not cone-synthesis robustness.

**Codebase grounding.** The existing IR (`src/ir/types.rs`) is a
scalar-`u32` gate-level circuit graph: `Port`/`Node`/`Flop`/
`Memory`/`Fsm`/`Instance`, no notion of `parameter`/`localparam`,
elaboration-time expressions, `generate`, packages, or typed
constants. `WidthExpr{Lit,Param}`/`ParamEnv` (Phase 5) is the
*closest* existing concept but is a narrow width-only annotation on
the circuit IR, not a general constant-expression/elaboration model.
Therefore Phase 7 needs its **own small source-level
constant/parameter IR** — a parameter+localparam dependency DAG of
typed constant expressions with their *evaluated* values — distinct
from and not threaded through the circuit IR (same operators-vs-
blocks / category-boundary discipline that kept memory and FSM as
blocks rather than datatypes). It reuses ANVIL's seeding (ChaCha8,
no `thread_rng`), CLI/knob plumbing, and reproducibility doctrine,
but is a **separate generator path** — it does not go through
`build_cone`.

**Artifact family — `rtl_const_expr` (per ROADMAP).** One module (or
a tiny package+module cluster) exercising, by construction:
parameter/localparam dependency chains
(`localparam B = A*2; localparam C = B + W;`); expression-derived
widths/ranges (`logic [DEPTH-1:0]`, `[$clog2(N)-1:0]`); `generate
if`/`for` whose conditions and bounds are expression-driven;
package-qualified constants (`pkg::WIDTH`); and precedence-sensitive
arithmetic / shift / comparison / equality / bitwise / logical /
ternary expressions. Typical size: one module, or a small cluster
when the pressure point needs local hierarchy.

**Expected-facts manifest (schema sketch).** One JSON manifest per
emitted `.sv`, capturing only *obviously-checkable elaboration
facts* the generator already knows:

```json
{ "seed": <u64>, "top": "<module>",
  "params":   { "<name>": { "value": <int>, "expr": "<src>" } },
  "localparams": { "<name>": { "value": <int>, "expr": "<src>" } },
  "widths":   { "<signal>": { "msb": <int>, "lsb": <int>, "bits": <int> } },
  "generate": { "<label>": { "taken": <bool> | "iterations": <int> } },
  "package_constants": { "pkg::<name>": <int> },
  "const_exprs": [ { "expr": "<src>", "value": <int>, "width": <int> } ] }
```

**Generation strategy — oracle by construction (the key idea).** The
generator builds the parameter/localparam/const-expression DAG and
**evaluates every node as it constructs it** (it chose the literals
and operators, so it computes the resolved integer/width *the same
way SV elaboration must*). The `.sv` text is emitted *from* that
evaluated DAG; the manifest is emitted *from the same resolved
values*. The generator **is** the oracle — there is no separate
analysis pass and no re-parsing of generated text. This is the exact
valid-by-construction / rules-first doctrine that governs the rest
of ANVIL (compute the fact at construction time; never
generate-then-analyze). Evaluation uses wide integer semantics
matching SV's constant-expression rules (2-state, sign/width per
LRM) for the integer subset Phase 7 emits — deliberately bounded so
the oracle is trivially correct.

**Reproducibility contract.** Identical to the DUT lane: `(seed,
knobs)` → byte-identical `.sv` **and** byte-identical `.json`
manifest, on any platform, forever. The manifest is part of the
reproducible artifact, not a side report.

**Parity-check harness.** Separate from the `tool_matrix`
lint/synth DUT gate (that proves *acceptance*; Phase 7 proves *fact
agreement*). The harness elaborates each emitted `.sv` with a
downstream consumer that can report resolved facts — candidate:
Yosys `read_verilog -sv; ... ; write_json` (parameter/width facts)
and/or Verilator/`slang` parameter introspection — and compares the
reported facts to the manifest: exact agreement, or a **retained
counterexample** (the `.sv` + manifest + tool output kept for
triage), never a silent pass. As with memory/FSM, a cargo-portable
formalization is available — the emitted declarations' widths/param
values equal the manifest *by construction* (structural-equivalence,
`cargo test`-able) — while the genuine downstream parity runs in the
repo-owned gate (cargo cannot shell yosys/verilator; project
convention since Phase 1).

**Boundaries.** Phase 7 = *constant/elaboration facts on tiny
modules*. Phase 8 (frontend/elaboration accept corpora) = *compact
elaboratable hierarchies* with a richer source-level hierarchy/
package IR — Phase 7's const-expr IR is the seed of, but smaller
than, Phase 8's. Phase 9 (umbrella) = the artifact-family selector
unifying DUT / Phase-7 / Phase-8 lanes; Phase 7 lands behind an
explicit family flag now and is rehomed under the Phase 9 selector
later. Phase 7 does **not** build the selector (that is Phase 9's
`.1`-blocked-until-≥2-lanes leaf).

**Rejected alternatives.** (A) **Reuse the gate-level circuit IR**
for const-expr artifacts — rejected: it has no parameter/localparam/
generate/package/typed-constant concept; forcing them through scalar
`u32` node graphs is the same category error as memory's rejected
datatype option (C). (B) **Generate random SV then parse it back to
derive the manifest** (generate-then-analyze) — rejected: violates
the oracle-by-construction doctrine and re-implements elaboration in
the oracle, so the oracle can be as wrong as the tool under test;
the generator already holds every resolved value. (C) **Bundle a
reference elaborator** to compute expected facts — rejected: project
non-goal (no bundled reference simulator); the construction-time
oracle is exact and free. (D) **Emit facts as SV comments instead of
a separate manifest** — rejected: not machine-checkable without
re-parsing, and couples the oracle to comment-formatting; a typed
JSON manifest is the durable contract.

**Proof shape (`.2`, expected to split).** Reproducible corpus
(byte-stable `.sv` + `.json` across re-runs and the existing
cross-platform reproducibility harness); a manifest-schema
validator; the parity harness over ≥1 downstream consumer green (or
counterexamples retained); behind an explicit artifact-family flag;
no regression to the DUT lane. Split candidates (independently
reviewable): const-expr/parameter IR + construction-time evaluator /
SV emitter + manifest emitter / parity harness + repo-owned gate.

This entry is design-only and is itself task-tree owned
(`PHASE-7-ORACLE-MICRODESIGN.1`); it makes no code change,
consistent with the task-tree-ownership doctrine's code/not-code
boundary.

**As-built — `.2a` IR + evaluator (2026-05-19).** `.2` split into
`.2a` (IR + evaluator/oracle) / `.2b` (SV + manifest emitters) /
`.2c` (parity harness + gate). `.2a` landed as a **new separate
top-level module `src/microdesign/`** (`pub mod microdesign` in
`src/lib.rs`) — *not* under `src/ir/`, exactly as the design's
rejected-alternative (A) requires (the gate-level circuit IR has no
parameter/localparam/expression concept; it must stay a separate
generator path). Concrete shape decisions worth recording: the
const-expr value type is **`i128`** (the rules-first builder keeps
every intermediate well inside it, so the oracle is *trivially
exact*; width-sized truncation against declared port/param widths is
deferred to `.2b` where widths exist — `.2a` is purely the value
DAG). `eval()` is total except two **defensive** `EvalError`s
(`DivByZero`, `UndefinedParam`) that the rules-first builder never
triggers but a hand-malformed unit must classify rather than panic;
shift amounts are clamped `[0,127]` so a (builder-impossible) huge
amount cannot panic Rust's shift. `resolve()` *is* the oracle: it
runs once at construction time and fills every `ParamDecl.value`;
the load-bearing `.2a` invariant (unit-proven) is that this stored
value never drifts from a fresh re-evaluation of its expression over
the resolved prefix — that equality is *why* `.2b` can emit both the
SV and the JSON manifest from `value` without a second analysis pass
or a re-parse. `build_constexpr_unit(seed,n)` uses the project
ChaCha8 convention verbatim (`ChaCha8Rng::seed_from_u64`, no
`thread_rng`).

**As-built — `.2b` emitters (2026-05-19).** SV + JSON manifest
emitters in the same module, both reading `ParamDecl.value` (the
`.2a` oracle). Decisions worth recording: (1) `expr_to_sv` is
**fully parenthesized** — the evaluator already fixed semantics; a
minimal-parens printer would risk the *downstream* front-end
parsing a different precedence than the oracle computed, so the
printer must not be clever. The precedence-sensitive-expression
axis is still exercised because the `.2a` builder emits genuinely
nested `a + b*c` / ternary shapes that round-trip *as written*.
(2) **Default-off DUT-byte-identical is structural, not a flag
check**: `microdesign` is a separate top-level module that the DUT
generate path never calls, so "the artifact-family flag is off" is
the *absence of a call site* — there is nothing to gate and nothing
that could perturb DUT output. The actual `--artifact` selector is
Phase 9's; `.2b` deliberately does not wire a CLI flag (that would
be premature and is Phase 9's lane-migration concern). (3) The
manifest uses `BTreeMap` for every object so `serde_json`
pretty-output key order is deterministic ⇒ the `.json` is a
byte-stable part of the reproducible artifact, exactly like the
`.sv`. (4) `widths`/`generate`/`package_constants` are derived by
small fixed rules (`(last % 8)+1`; `P0 >= pkg_const(seed)`;
`seed % 64 + 1`) whose *resolved* values come from the oracle —
the SV carries the symbolic form, the manifest the resolved form,
and `manifest_mirrors_the_oracle` pins their equality.

### Phase 8 frontend/elaboration accept-corpus source-IR design (2026-05-18, PHASE-8-FRONTEND-ACCEPT.1)

Design-only slice. No code. Lifts ROADMAP Phase 8 ("frontend/
elaboration accept corpora — compact elaboratable hierarchies")
into a concrete, codebase-grounded plan: why a dedicated
source-level IR, the surfaces it must express, the
expected-elaboration-facts manifest schema, the parity harness, the
relationship to Phase 7 / Phase 9, rejected alternatives, the `.2`
proof shape + split. Mirrors the proven design-first method.

**The shift (and the boundary with Phase 7).** Phases 1–6 emit
*already-elaborated, parameter-resolved* gate-level RTL (the
"structural, not meaningful" DUT lane). Phase 7 is a tiny
*single-module* const-expr oracle (one module, constant facts).
Phase 8 is the **frontend/elaboration** lane: *compact elaboratable
hierarchies* (1–3 modules + packages) emitted with **parameters
unresolved in the SV text**, shipped with a manifest of what
*elaboration must resolve them to*. The pressure point is the
downstream tool's **front-end / elaboration** (parameter override
resolution, instance binding, generate selection, package/type
resolution) — a surface the gate-level circuit IR cannot represent
*at all*.

**Codebase grounding.** The circuit IR (`Port`/`Node`/`Flop`/
`Memory`/`Fsm`/`Instance` in `src/ir/types.rs`) is *post-
elaboration*: scalar `u32` nets, resolved widths, flattened/
instantiated modules. It has no module-declaration, parameter-port,
`localparam`, package, `typedef`/struct/union/enum, procedural-block
or `generate` concept. Phase 5's `ParamEnv` and Phase 7's const-expr
DAG are *sub-models* (resolved-width annotation; single-module
constant facts) — neither expresses a hierarchy of un-elaborated
module declarations. Phase 8 therefore needs a first-class
**source-level AST IR** that emits *un-elaborated* SV, distinct
from and not threaded through the circuit IR (the roadmap decree +
the same category-boundary discipline that kept memory/FSM as
blocks). It **reuses Phase 7's construction-time integer/const-expr
evaluator and JSON-manifest core** (do not reimplement) and ANVIL's
seeding/CLI/reproducibility; it is a **separate generator path**.

**Surfaces the source IR must express (per ROADMAP).** ANSI port
lists + parameter ports; parameter/localparam flows across
instances; instantiation variants — named/ordered parameter
overrides, named/ordered/wildcard (`.*`) port connections, instance
arrays; package imports + package-qualified constants/types;
typedef-backed types — packed/unpacked structs, unions, enums,
builtin integral atoms (`int`/`byte`/`logic`/…); the full
`assign` / `always_comb` / `always @(*)` / `always_ff` /
`always_latch` set; `generate if` / `for`.

**Source-IR sketch.**

```
SourceUnit   = { packages: Vec<Package>, modules: Vec<Module> }   // ordered, top last
Package      = { name, items: Vec<PkgItem /* Localparam | Typedef */> }
Module       = { name, params: Vec<ParamDecl>,            // #(parameter ...)
                 ports:  Vec<PortDecl /* ANSI, typed, dir */>,
                 items:  Vec<ModuleItem> }
ModuleItem   = Localparam(name, Expr)
             | VarDecl(name, Type)
             | Typedef(name, Type)
             | ContinuousAssign(lhs, Expr)
             | Always(kind: Comb|FfPosedge|Latch|StarAt, body)
             | Instance{ target, params: Named|Ordered(Vec<Expr>),
                         ports: Named|Ordered|Wildcard, array: Option<RangeExpr> }
             | Generate(If{cond: Expr} | For{genvar, bound: Expr})
Type         = Logic{packed_dims} | Atom(int|byte|…) | Enum{base,members}
             | Struct{packed,fields} | Union{fields} | Named(typedef)
             | PkgQual(pkg,name)
Expr         = the Phase 7 const-expr node set (reused), over
               parameters/localparams/genvars/package constants.
```

Every `ParamDecl`/`Localparam`/generate condition carries its
**construction-time-evaluated** value (Phase 7's evaluator), so the
manifest is exact and the SV text can stay un-elaborated.

**Expected-elaboration-facts manifest (extends Phase 7's schema).**
Per emitted top, JSON, byte-stable: resolved top parameter values;
the **instance tree** (instance path → target module → resolved
child parameter values → child port bindings); selected `generate`
branches / unrolled `for` iteration counts; package constant/type
resolutions; typedef-resolved widths. The Phase 7
`params`/`localparams`/`widths`/`const_exprs` blocks are reused
verbatim; Phase 8 adds `instances`, `generate`, `packages`,
`typedefs`.

**Generation strategy — oracle by construction (reuse Phase 7's
evaluator).** Identical doctrine: the generator chooses the
hierarchy + parameter values and *performs the elaboration itself*
at construction time (it knows the instance tree, override
resolution, and generate selection because it built them); it emits
*un-elaborated* SV **and** the elaborated-facts manifest from the
same resolved knowledge — no analysis pass, no re-parse, no bundled
elaborator. The novelty vs Phase 7: the SV text deliberately keeps
parameters symbolic (`foo #(.W(W*2)) u();`) and the manifest asserts
the elaboration result (`u.W == 16`) — that gap is exactly the
front-end behaviour under test.

**Open-Question resolution (reuse of Phase 7 manifest machinery).**
**Resolved**: Phase 8 *reuses* Phase 7's construction-time
evaluator + JSON-manifest emitter core and *extends* the schema
with hierarchy/instance/generate/package facts. Dependency
direction: `PHASE-8-FRONTEND-ACCEPT.2` depends on
`PHASE-7-ORACLE-MICRODESIGN.2`'s evaluator/manifest core landing
first (recorded so `.2` sequences correctly). Phase 9 unifies the
artifact-family selector; Phase 8 lands behind an explicit family
flag, not the selector.

**Parity harness.** Same shape as Phase 7 but hierarchy-aware: a
downstream elaborator (Yosys `read_verilog -sv; hierarchy -top …;
write_json`, and/or `slang`/Verilator hierarchy+param
introspection) reports the elaborated hierarchy facts; the harness
compares to the manifest — exact agreement or a **retained
counterexample**. Repo-owned gate (cargo cannot shell
yosys/verilator — the Phase-1 convention); a cargo-portable
structural-consistency slice (emitted declarations vs the
generator's own resolved values) complements it.

**Rejected alternatives.** (A) **Reuse the gate-level circuit IR** —
rejected by roadmap decree *and* structurally: it is
post-elaboration and cannot express modules/parameters/packages/
generate. (B) **Emit already-elaborated SV** (parameters
pre-resolved in text) — rejected: that is the Phases 1–6 DUT lane;
it exercises synthesis, not the front-end/elaboration path Phase 8
exists to stress; un-resolved-text-plus-manifest *is* the contract.
(C) **A full SV parser/elaborator inside ANVIL to derive facts** —
rejected: oracle-by-construction makes it unnecessary and it would
re-introduce the very elaboration bugs under test (same as Phase
7's (B)/(C)). (D) **Extend Phase 7's single-module const-expr IR
in place** instead of a dedicated hierarchy/package IR — rejected:
hierarchy/instantiation/packages are a categorically larger
surface; cramming them into the const-expr DAG repeats the
circuit-IR category error. Phase 8 *reuses Phase 7's evaluator* but
is its own structural source IR.

**Proof shape (`.2`, expected to split).** Reproducible 1–3 module
accept corpora (byte-stable SV + manifest, cross-platform); the
source IR emits valid un-elaborated SV (the downstream tool
elaborates it clean); manifest-schema validation; parity harness
green or retained counterexamples; behind the artifact-family flag;
no DUT-lane regression. Split candidates (independently reviewable):
source IR + construction-time elaboration-evaluator (reusing the
Phase 7 core) / SV emitter + manifest emitter / parity harness +
repo-owned gate.

This entry is design-only and is itself task-tree owned
(`PHASE-8-FRONTEND-ACCEPT.1`); it makes no code change, consistent
with the task-tree-ownership doctrine's code/not-code boundary.

### Phase 9 multi-artifact umbrella selector design (2026-05-18, PHASE-9-MULTI-ARTIFACT-UMBRELLA.1)

Design-only slice. No code. Lifts ROADMAP Phase 9 ("multi-artifact
ANVIL umbrella — an artifact-family selector with shared plumbing")
into a concrete plan: the lane interface, the shared
reproducibility/manifest/seed/output/check contract, the
CLI/selector surface, the lane-migration plan, rejected
alternatives, the `.2` proof shape + split. Designed *now* (per the
tree's 2026-05-16 Decision) so Phases 7/8 are built
selector-compatible rather than retrofitted.

**The point (and the explicit anti-goal).** Phase 9 makes one tool
drive every valid-by-construction lane *with the lanes kept
separate*. The anti-goal it exists to prevent: collapsing into "one
generator that emits random SV files" with contradictory promises.
The lane *interface* unifies **plumbing** (seed, knobs,
reproducibility, manifest, output layout, downstream dispatch); it
does **not** merge the generators.

**The lanes.**

- **L1 — DUT RTL** (Phases 1–6): structurally-valid random
  synthesizable RTL; oracle = lint/elaborate/synth-clean (the
  `tool_matrix` gate). Generator = `build_cone`/hierarchy; circuit
  IR. **No semantic manifest** (deliberate — "structural, not
  meaningful").
- **L2 — oracle-backed micro-design** (Phase 7): tiny const-expr
  `.sv` + expected-facts manifest; oracle = fact agreement (parity).
  Const/param IR.
- **L3 — frontend/elaboration accept** (Phase 8): compact
  un-elaborated hierarchies + elaborated-facts manifest; oracle =
  elaboration-fact agreement (hierarchy parity). Source AST IR.
- Future valid synthesizable lanes plug in via the same contract.

**Lane interface (the abstraction).**

```
trait ArtifactLane {
    fn name(&self) -> &str;                 // "dut" | "oracle-microdesign" | "frontend-accept"
    fn validate_knobs(&self, &Config) -> Result<(), ConfigError>; // lane-scoped only
    fn generate(&self, seed, &Config) -> Corpus;   // (seed,knobs) -> byte-stable artifacts
    fn manifest(&self, &Corpus) -> Option<Manifest>;// None for L1 (first-class, not a hack)
    fn check_plan(&self, &Corpus) -> CheckPlan;     // SynthAccept (L1) | ParityVsManifest (L2/L3)
}
```

Shared plumbing the umbrella owns (never duplicated per lane):
ChaCha8 seed→artifact derivation + byte-stable cross-platform output
(today's doctrine, centralized); the JSON manifest emitter + schema
versioning (Phase 7 core; `Option` so L1's absence is typed, not a
sentinel); a lane-scoped knob namespace (each lane validates only
its knobs; cross-lane knob bleed is rejected); a uniform on-disk
layout (`<out>/<lane>/<scenario>/… [+ manifest.json]`); a uniform
`CheckPlan` the repo-owned gate dispatches (synth-accept for L1,
parity-vs-manifest for L2/L3).

**CLI/selector surface — Open-Question resolution.** **Resolved**:
a top-level **`--artifact <lane>` flag on the existing `anvil`
binary, default `dut`**. Default-`dut` ⇒ every current invocation,
the entire book, and CI keep working **byte-identically** (this is
load-bearing — `BOOK-EXAMPLES-RUNNABLE` made hundreds of
`cargo run --release -- …` examples a CI-gated contract; a
subcommand-only redesign would regress all of them). `--artifact
oracle-microdesign` / `--artifact frontend-accept` opt into L2/L3.
`tool_matrix` stays the L1 gate harness; the umbrella adds lane
dispatch, not a rewrite. Rejected forms recorded below.

**Lane-migration plan.** L1 is wrapped as the **default** lane with
**zero behaviour change**: `DutLane::generate` *is* today's
`generate_design`; the default selector reproduces every existing
seed byte-identically (a hard regression gate in `.2`). L2/L3 are
built against this `ArtifactLane` contract from the start (Phases
7.2 / 8.2 implement to it — the reason `.1` is designed early), so
there is **no retrofit**. The shared
`(lane, seed, lane_knobs) → byte-identical corpus (+ manifest)`
contract is a strict superset of today's `(seed, knobs)` DUT
contract with `lane` prepended and `dut` defaulted.

**Rejected alternatives.** (A) **Separate binaries per lane** —
rejected: duplicates seed/knob/reproducibility plumbing, fragments
the "one go-to tool" goal, multiplies the CI/book surface. (B)
**One generator path emitting all families via mode flags inside
`build_cone`** — rejected: the explicit anti-goal; synth-clean vs
oracle-exact vs elaboration-accept are contradictory promises that
cannot share one generator without category errors — unify the
*interface*, not the generators. (C) **Subcommand-only CLI**
(`anvil gen-dut …`) — rejected: breaks the existing flat CLI and the
entire CI-gated book example surface for no plumbing benefit a
default-`dut` `--artifact` flag does not already provide. (D)
**Defer the abstraction until ≥2 lanes exist** — rejected by the
tree's standing Decision: designing it now is exactly what keeps
Phases 7/8 lane-compatible instead of retrofitted.

**Proof shape (`.2`, blocked until ≥2 delivered lanes).** The
`ArtifactLane` contract + shared plumbing implemented; the DUT lane
wrapped default-`dut` **byte-identical** (every existing seed
reproduces — hard regression gate, incl. the book/CI examples); ≥1
of L2/L3 selectable via `--artifact`; uniform output layout +
manifest plumbing; lane-scoped knob validation; no
DUT-lane/book/CI regression. Unblock condition (recorded in the
tree): the DUT lane plus ≥1 of Phase 7/8 lanes exist. Split
candidates (independently reviewable): lane trait + shared plumbing
/ DUT-lane wrap (byte-identical regression-gated) / first non-DUT
lane wired to the selector.

This entry is design-only and is itself task-tree owned
(`PHASE-9-MULTI-ARTIFACT-UMBRELLA.1`); it makes no code change,
consistent with the task-tree-ownership doctrine's code/not-code
boundary.

### Second-simulator (iverilog) compatibility note (2026-05-18, DIFFERENTIAL-SIMULATION.1)

Research-only slice (no code). Establishes which second simulator
can ingest ANVIL's existing Verilator-clean SV and where it would
diverge, so `DIFFERENTIAL-SIMULATION.2`'s harness has a concrete
target.

**Empirical ingest probe.** Installed Icarus Verilog **13.0
(stable)** and ran `iverilog -g2012 -o /dev/null <files>`
(SV-2012, full parse + elaborate) against freshly-generated release
output for every ANVIL output category, with `verilator
--lint-only` on the same files as the contrast:

| Category | sample | `iverilog -g2012` | `verilator --lint-only` |
| --- | --- | --- | --- |
| combinational leaf | `--seed 7 --flop-prob 0` | **exit 0, silent** | exit 0, clean |
| sequential leaf (flops) | `--seed 5 --flop-prob 1.0` | **exit 0, silent** | exit 0, clean |
| bounded recursive hierarchy (4 modules) | `--min/max-hierarchy-depth 2`, 2 inst | **exit 0, silent** | exit 0, clean |
| helper-instance / sibling routes (3 modules) | `--hierarchy-sibling-route-prob 1.0` | **exit 0, silent** | exit 0, clean |

**Verdict: iverilog is a zero-configuration second simulator for
every ANVIL output category.** No source edits, no compat shims,
no per-category flags — only the standard `-g2012` SV-2012 select
(ANVIL emits SystemVerilog: `always_ff`/`always_comb`, packed
part-selects, `{N{x}}` replication, async-reset flops, ANSI ports,
multi-module hierarchies). Both engines accept all four categories,
so the **chosen differential pair is Verilator ↔ iverilog** — and
it is a *strong* pair precisely because the engines are
semantically independent: **Verilator** is a compiled,
2-state-by-default, cycle-driven simulator; **iverilog** is an
interpreted, 4-state (`0/1/x/z`), event-driven simulator. Agreement
across that gap is meaningful corroboration, not two views of the
same engine.

**Where they will diverge (the `.2`/`.3` harness must design around
this — not an ingest blocker).** ANVIL output is combinational +
synchronous-reset flops with no `X`/`Z` injection, so the only
material Verilator/iverilog semantic gap is **pre-reset 4-state
behaviour**: iverilog drives flops `x` until the async reset
deasserts; Verilator (2-state default) starts them `0`. Therefore
the differential harness must (a) drive a deterministic reset
sequence first, (b) sample outputs **only at a single canonical
post-reset point**, and (c) compare defined bits only. Combinational
cones are pure functions of inputs ⇒ no timing gap once inputs are
held. These are exactly the Open Questions the tree already routes
to `.2` (input-vector scheme; canonical sample point; timing) —
this note confirms they are *design* problems, not *feasibility*
blockers.

**Rejected alternatives.** (A) `verilator --binary` self-vs-self —
rejected: same engine, zero independent corroboration (the whole
point is engine independence). (B) Yosys as the sim peer — rejected
(already in tree Decisions): Yosys is a *synthesizer*, not an
event-driven simulator; it cannot be a semantic-equivalence peer.
(C) Commercial simulators (VCS/Xcelium/Questa) — deferred (tree
Decision): unavailable in-environment; the open-source pair already
gives independent corroboration. (D) Single-simulator (Verilator
only) — rejected: cannot prove *cross-simulator* agreement, which
is the signoff-quality bar this tree exists to raise.

This entry is research-only and is itself task-tree owned
(`DIFFERENTIAL-SIMULATION.1`); it makes no code change, consistent
with the task-tree-ownership doctrine's code/not-code boundary
(`.2`+ build the harness).

### Single-design differential harness design (2026-05-18, DIFFERENTIAL-SIMULATION.2a)

Design-only slice (no code; `.2b` implements). `.2` was split —
the harness's testbench-generation strategy, reset/sample
alignment, stimulus determinism, dual-simulator orchestration, and
the tool-gated-test convention are load-bearing decisions that
should be settled and reviewed before code (and the design itself
is docs-only, ~zero contention on the near-complete Phase 6 gate,
mirroring the Phase 7/8/9 design-first discipline).

**Goal.** A single-design utility: given a canonical
`(seed, config)`, drive the generated module through **both**
Verilator and iverilog and return aligned output traces, so `.2b`'s
focused test can assert they agree byte-for-byte. Builds directly
on `.1` (iverilog is zero-config-compatible; the only divergence is
pre-reset 4-state).

**Testbench generation — from the IR, not by parsing SV.** The
harness generates the design *in-process* via the library (exactly
like `tests/snapshots.rs`), so it already holds the typed
`Design`/`Module`: port names (`i_*`/`o_*`/`clk`/`rst_n`), widths,
directions, and whether the module carries sequential state
(`has_local_flops()/has_local_memories()/has_local_fsms()`). The
generic SystemVerilog testbench is emitted **from that IR** — never
by re-parsing emitted SV (brittle, a re-implementation of the
front-end). The testbench: instantiates the DUT, drives each input
from a baked deterministic vector sequence, and `$display`s each
output as fixed-width hex at the canonical sample point(s) into a
trace file. One identical testbench file feeds both simulators.

**Reset + canonical sample point (neutralises `.1`'s divergence).**
Per `.1`, the only Verilator/iverilog semantic gap on ANVIL output
is pre-reset 4-state (`iverilog` flops `x` until async reset
deasserts; Verilator-2-state starts `0`). The testbench therefore:
combinational module → hold each input vector, sample the outputs
after a settle delay (no clock); sequential module → assert
`rst_n = 0` for a fixed K cycles, deassert, then for each of N
cycles apply the next input vector and sample outputs **at a single
fixed post-reset cycle offset** (a deterministic warmup then
per-cycle sampling). Only post-reset, fully-defined samples are
compared — the pre-reset `x`/`0` gap is never observed.

**Deterministic stimulus — baked, not per-sim `$random`.** Input
vectors are computed in Rust from the seed (a reproducible
sequence: zero, all-ones, walking-1, then seeded pseudo-random) and
**baked into the testbench as constants**. `$random` is *not* used:
iverilog and Verilator have different `$random` streams, which
would inject false mismatches. Baked identical stimulus guarantees
both simulators see exactly the same inputs.

**Dual-simulator orchestration.** (a) iverilog:
`iverilog -g2012 -o sim.vvp dut.sv tb.sv` then `vvp sim.vvp`
→ trace A. (b) Verilator:
`verilator --binary -j0 -sv --top-module tb dut.sv tb.sv` (5.x
`--binary` builds a runnable directly from the *same* testbench)
then run the produced binary → trace B. Both `$display` the
identical fixed-width-hex trace format; the harness byte-compares A
vs B and returns the aligned traces (+ a structured diff on
mismatch — never a silent pass; a mismatch is a *retained
counterexample* with the SV + stimulus, mirroring the Phase 7
parity-harness discipline).

**Tool-gated test convention (load-bearing — Phase-1 doctrine).**
`cargo test` must pass on machines without verilator/iverilog (the
convention since Phase 1; reaffirmed for memory/FSM `.2.2` and the
tool_matrix gate). So `.2b`'s focused differential test is
`#[ignore]` by default — run explicitly (`cargo test -- --ignored
diff_sim`) or from a repo-owned context where both simulators are
present. The harness itself is a plain utility fn; the *gated* test
is the only tool-requiring surface, so the portable `cargo test`
stays green tool-less and `.2b` adds ~zero mandatory-gate runtime.

**Rejected alternatives.** (A) Parse emitted SV text to discover
ports — rejected: brittle front-end re-implementation; the IR has
exact port info already. (B) Per-simulator `$random` stimulus —
rejected: divergent streams ⇒ false mismatches; bake identical
vectors. (C) Make the differential test a normal (non-`#[ignore]`)
`cargo test` — rejected: breaks the tool-less-portability doctrine.
(D) Verilator `--cc` + a hand-written C++ main — rejected vs
`--binary`: more moving parts and a second harness language;
`--binary` runs the *same* SV testbench iverilog uses, keeping one
testbench for both. (E) Compare full cycle-by-cycle traces incl.
pre-reset — rejected: re-introduces exactly the `.1` 4-state gap;
post-reset canonical sampling is the correct contract.

**Proof shape (`.2b`).** A `#[ignore]` focused test builds a
hand-picked combinational and a sequential `(seed, config)` leaf,
runs the harness (both simulators, post-reset aligned traces), and
asserts byte-equality; `cargo fmt/clippy/check/test` green with the
diff-sim test ignored by default. `.3` wires it into `tool_matrix
--diff-sim` over a representative subset + the
`saw_design_with_cross_simulator_agreement` fact; `.4` documents
the contract (README/USER_GUIDE/book).

This entry is design-only and is itself task-tree owned
(`DIFFERENTIAL-SIMULATION.2a`); it makes no code change, consistent
with the task-tree-ownership doctrine's code/not-code boundary.

### Coverage baseline triage — top-5 under-covered files (2026-05-18, COVERAGE-INSTRUMENTATION.2)

Triage-only slice (no code; `.3` acts on these findings). Classifies
every top-5 under-covered file from `docs/coverage-baseline.md`
(85.26% lines overall) into: **(a) dead code → remove**,
**(b) rarely-fired real path → add a focused proof**, **(c)
intentionally unreachable / integration-only → leave + document**.
Method: reasoned code inspection (orphan-symbol audit, panic/
rollback-site enumeration, `Err`-return vs inline-test count), not a
coverage re-measure.

| # | File | Uncov / % | Disposition | `.3` action |
| --- | --- | --- | --- | --- |
| 1 | `bin/tool_matrix.rs` | 1951 / 72.07% | **(c)** gate-exclusive. Every `*_focus_config` / scenario-builder is referenced from `build_scenarios` — **no orphan/retired builders, zero dead code**. The miss is the `Phase4Hierarchy` scenario + per-scenario config helpers, which fire only under the matrix gate the baseline *deliberately* excludes (75-min runtime). Already exercised by the repo-owned gate. | None. Optionally a "deep" `cargo llvm-cov` incl. the gate for an occasional refresh — not every-slice discipline. |
| 2 | `gen/cone.rs` | 454 / 88.65% | **(b) + (c)** — the **only real proof-gap in the top-5**. 45 panic/`expect`/`unreachable` sites: most are (c) by-construction-invariant guards. But `build_cone_with_retry`'s **retry-budget-exhaustion** path (`⚠️ cone retry budget exhausted`), `rollback_construction_snapshot`, the **anti-collapse reject / skipped-emission** branches, and `pick_terminal`'s adapter fallback are (b) genuinely reachable under specific knob/seed pressure. | `.3`: add focused proofs forcing (i) empty-dep-root retry→rollback→exhaustion, (ii) an anti-collapse reject, (iii) the `pick_terminal` adapter fallback. Leave the invariant-guard `expect`s. |
| 3 | `ir/validate.rs` | 254 / 75.07% | **(c)** intentional defensive validation. 62 `return Err(ValidateError::…)` arms; 26 inline tests already drive the malformed-input-reachable subset (hand-crafted broken modules). The residual arms guard "cannot happen from any generator path" invariants — the safety net the valid-by-construction doctrine relies on; **not dead, not a meaningful proof gap**. | Leave + documented here. Optional low-priority `.3`: a few more hand-broken-IR unit tests for the highest-value invariants. |
| 4 | `config.rs` | 250 / 67.87% | **(c) + audit** integration-only. Unit tests build `Config` via `..Config::default()`, bypassing the clap/serde-default + probability-range validation arms (only a real binary invocation drives them). 137 `pub` fields / 37 validate sites; the orphan-builder-style check found no retired symbols, but a per-field *wiring* audit was out of scope for triage. | `.3`: spot-audit for orphan knobs no longer wired (baseline-flagged); otherwise integration-style binary invocations, lower leverage. |
| 5 | `main.rs` | 142 / 60.56% | **(c)** clap-derive + flag→`Config` overlay boilerplate, exercised only by real binary runs (no test spawns the binary). **Lowest leverage of the five**; not dead, not a real proof gap. | None / optional `.3` binary-smoke with a few flag combos. |

**Headline finding (right-sizes `.3`).** There is **no confirmed
dead code** in the top-5 — the 3314 headline uncovered lines are
*gate-exclusive* (`tool_matrix`), *intentional defensive*
(`validate.rs`), or *integration-only* (`config.rs`/`main.rs`) **by
design**, not test debt. The single high-value `.3` target is a
**handful of `gen/cone.rs` focused proofs** (retry-exhaustion /
anti-collapse-reject / adapter-fallback). `.3` should therefore be
scoped to those cone proofs + an optional `config.rs` orphan-knob
spot-audit — *not* a broad coverage-chasing exercise. This is the
honest disposition the baseline's "(a)/(b)/(c) per file" promise
asked for.

This entry is triage-only and is itself task-tree owned
(`COVERAGE-INSTRUMENTATION.2`); it makes no code change, consistent
with the task-tree-ownership doctrine's code/not-code boundary
(`.3` performs the code actions).

**`.3` outcome (2026-05-18, COVERAGE-INSTRUMENTATION.3 — tree
CLOSED).** Acted on the triage exactly as scoped, nothing more:

- **(b) cone.rs #2 — the one real proof-gap — closed.** Added
  `tests/pipeline.rs::constant_pressure_exhausts_cone_retry_and_stays_valid_and_reproducible`
  (4 `ConstructionStrategy` × 4 seeds, `constant_prob = 1.0`,
  `max_depth = 1`). `constant_prob = 1.0` makes `pick_terminal`
  always take its "emit fresh constant" branch ⇒ every cone root is
  empty-dep ⇒ `build_cone_with_retry` runs the empty-dep retry +
  `rollback_construction_snapshot` loop across all `MAX_RETRIES`
  then the "⚠️ retry budget exhausted, accepting last attempt"
  fallback. The proof pins the invariant those branches exist to
  guarantee: *maximum constant pressure cannot break the pipeline*
  — `generate_design` stays `validate_design`-clean and
  byte-reproducible (no panic / infinite-loop / invalid IR;
  trivially-constant outputs are accepted, not fatal). Soundness +
  reproducibility are asserted, *not* non-triviality (the fallback
  is documented to allow trivially-constant outputs).
- **(a) config.rs #4 — orphan-knob spot-audit — no dead code.** Of
  74 `pub Config` fields, exactly 3 have zero external field-access:
  `library_prob`, `max_nodes_per_module`, `use_async_reset`. All
  three are **intentionally-reserved** and *already documented as
  such* in `book/src/knobs.md` — a future Phase-4+ probabilistic
  dial, a safety ceiling "not typically tuned", and "currently
  unused; flops are always async-reset by discipline",
  respectively. They are serde/CLI-stable knobs whose removal would
  break config compatibility and contradict the book. **Disposition:
  leave as-is** — confirms `.2`'s "no confirmed dead code"
  headline, with the orphan-knob question now positively resolved.
- **(c)** the gate-exclusive (`tool_matrix.rs`), intentional-
  defensive (`validate.rs`), and integration-only
  (`config.rs`/`main.rs`) regions are left exactly as `.2`
  documented — not test debt.
- **Baseline refreshed** via `cargo llvm-cov --release` (the
  instrumented full suite, which also served as this slice's
  COMMIT.md `cargo test` gate); `docs/coverage-baseline.md` carries
  the refreshed numbers + a `.3` addendum. Net: the
  `COVERAGE-INSTRUMENTATION` tree is **closed** with the single real
  proof-gap closed and every other "gap" positively confirmed
  intentional — no broad coverage-chasing, exactly the honest
  outcome `.2` argued for.

### Phase 5b packed-aggregate emitter projection design (2026-05-17, PHASE-5B-AGGREGATES.1)

Design-only slice. No code. Lifts `book/src/ir.md` "Synthesizable
aggregates" (the **packed** sub-question only) into a concrete,
codebase-grounded implementation plan with a rejected-alternatives
trail and a proof shape, so the implementation leaf
(`PHASE-5B-AGGREGATES.2`) has an unambiguous target.

**Goal (from ROADMAP Phase 5b / book "Synthesizable aggregates").**
Emit packed `struct` / `union` / `array` as an **opt-in projection
over the existing flat IR**, valid by construction, downstream-clean
(Verilator + both Yosys modes). Purpose is **parser/elaboration
coverage** in downstream tools, not new synthesis behaviour: a packed
aggregate is semantically a flat bit vector (synthesis treats it as
concatenation with named field-access sugar). Default-off /
byte-identical; no IR restructuring; no Phase-4/Phase-5 dependency;
never retire existing behaviour.

**Code reality that constrains the design** (audited; key anchors):

- The emitter is an explicit **dumb serialiser**
  (`src/emit/sv.rs:49-56` `to_sv_with_modules`): it walks `m.nodes`
  in order, assumes every IR invariant was enforced upstream, does no
  filtering or reachability. The module surface is built from flat
  scalar vectors only: header `module {name} (` / `#( parameter int
  {W} = {D} )` (sv.rs:79-118), ports `input|output logic {wd} {name}`
  via `param_width_decl` (sv.rs:91-116), internal `wire|logic {wd}
  {name};` per `Node::Gate`/`InstanceOutput` (sv.rs:140-173), flop
  `logic {wd} {name};` (sv.rs:122-130), then combinational `assign`s,
  child instance port connections (sv.rs:~315) and output-port
  `assign`s (sv.rs:376-380).
- `Port { id, name, width: u32, dir }` (`src/ir/types.rs:24-29`) and
  every `Node::*`/`Flop` width is a bare `u32`. There is **no**
  aggregate/struct concept anywhere in the IR, validators
  (`src/ir/validate.rs`), CSE keys (`intern_gate`/`intern_constant`),
  or the dedup signature (`canonical_module_signature`,
  `src/metrics.rs`).
- **Phase 5 set the exact precedent to follow.** `param_env:
  Option<ParamEnv>` + `WidthExpr` (`src/ir/types.rs:31-69`) is a
  per-module annotation the IR body never reads; only the emitter
  consults it at the `param_width_decl` width chokepoint, and the
  identity rule consults it in `canonical_module_signature`. The flat
  `width: u32` fields were intentionally untouched. Default-off
  (`param_env == None`) ⇒ byte-identical emission. Phase 5b is the
  same shape one layer out: an emitter-consulted annotation that
  regroups *which* ports render as a packed aggregate, with the IR
  body still flat.

**Architectural decision — chosen: (P) emitter-only packed-aggregate
projection driven by a per-module annotation.** Mirror Phase 5's
annotation-consulted-only-by-emitter architecture (C):

1. Construction is **unchanged**. Modules are built exactly as today
   over flat `u32`-width ports/nodes; all fold/validate/CSE/dedup
   machinery runs untouched.
2. A post-construction, opt-in pass records a lightweight per-module
   annotation (working name `AggregateLayout`): a small additive,
   `Default`-able `Module` field (zero churn to `..Module::default()`
   sites, exactly as `param_env`/`parameterized_*_ports` were added)
   describing **how a contiguous, same-direction subset of ports
   maps onto one packed type**: kind (`StructPacked` |
   `UnionPacked` | `ArrayPacked`), the chosen type name
   (`{module}_{in|out}_t`), and the ordered `(field_name, PortId)`
   list. The bit layout is the existing port concatenation order — a
   **bijective, bit-layout-preserving regrouping**, semantically a
   no-op (the synthesised netlist is identical to the flat form).
3. Emitter learns the projection at the same chokepoints Phase 5
   touched: emit `typedef struct packed { logic [w-1:0] f0; … }
   {module}_in_t;` (and/or union/array) before the module, replace
   the grouped port list with one aggregate port, and rewrite
   references to a grouped port from `name` to `agg.fieldN` (a pure
   rename at the SV surface — the internal flat wires/assigns are
   unchanged; only the port-boundary read/drive uses `.fieldN`). For
   `union`, all members share the same total width (legal because the
   group's total width is fixed); for `array`, the fields are
   same-width slots. No annotation (default-off) ⇒ byte-identical.
4. Knob surface: opt-in `aggregate_*_prob` (`f64`, serde-default
   `0.0`, probability-range validated) — same pattern as
   `width_parameterization_prob`. Default 0.0 ⇒ no annotation ⇒
   byte-identical for fixed seeds. (Single `aggregate_prob` + a
   kind-choice sub-roll vs three separate probs is a `.2`
   calibration sub-decision; the design only fixes "opt-in,
   default-off, serde-default".)

**Soundness rule.** A packed `struct`/`union`/`array` is *defined* by
the SV LRM to be bit-equivalent to the concatenation of its members;
the projection only chooses a syntactic surface for a fixed bit
layout the flat form already had. Therefore the projection is **valid
by construction** for *every* generated module whose grouped ports are
contiguous and same-direction, with **no** validator participation and
**no** generate-then-filter: it is a construction-time emitter rule,
not a post-hoc text rewrite. Downstream-cleanliness follows from the
equivalence and is *proven*, not assumed, by the matrix gate.

**Identity interaction (resolves the tree's Open Question).**
`canonical_module_signature` is computed from the **flat IR**, which
the projection never mutates. The aggregate annotation is *not* hashed
into the signature (unlike Phase 5's `param_env`, which had to be,
because parameterization changes the legal width set — aggregates
change *nothing* semantic). Consequence: a module and its
aggregate-projected twin share one signature and **dedup-collapse**,
which is correct (they are the identical circuit). `dedup_modules`
unchanged. This is the opposite of the Phase 5 identity rule and is
deliberate.

**Rejected alternatives.**

- **(A) First-class aggregate IR nodes** (`struct`/`union`/`array`
  variants in `Port`/`Node`, width-aware). Rejected: a massive
  invasive change rippling through `validate.rs`, the
  `intern_gate`/`intern_constant` CSE keys, `canonical_module_signature`
  + `dedup.rs`, and all per-op width arithmetic — for **zero new
  synthesis behaviour** (packed aggregates are semantically flat).
  Directly violates the book's "keep the IR flat" and this tree's
  Non-Goal "any IR restructuring; aggregates are an emitter projection
  over the existing flat IR". It is the strict superset only if/when a
  *semantically distinct* aggregate (unpacked memory) is pursued —
  that is Phase 6, not here.
- **(B) Post-hoc textual/AST rewrite of the emitted SV string.**
  Rejected: fragile, can desync from the IR, and is exactly the
  post-hoc-rewrite / generate-then-filter anti-pattern the project
  doctrine forbids (rules-first, construction-time). The projection
  must be a deterministic emitter rule reading a recorded annotation,
  not a regex pass over `to_sv` output.
- **(C) Unpacked aggregates / enums in this phase.** Rejected /
  deferred (restated so the deferral is not silently revisited, per
  the existing 2026-05-16 tree Decision): unpacked array is the Phase
  6 memory-inference motif; unpacked datapath `struct`/`union` is
  mostly non-synthesizable; enums are thin (typed constant sets with
  no stress value beyond constants). Phase 5b is **packed-only**.

**Proof shape (for `.2`).** (1) Default-off byte-identical for fixed
seeds across all `ConstructionStrategy` values (no annotation ⇒
identical `to_sv`). (2) Forced-on: a focused proof that a projected
module's emitted SV declares a `typedef … packed` and a single
aggregate port, and that field references resolve. (3) A
`tool_matrix` aggregate scenario downstream-clean: Verilator
`--lint-only` + both Yosys modes all-pass, `coverage_gaps=[]`, a new
`saw_packed_aggregate_design` coverage fact. (4) Identity-invariance:
a unit test that a module and its aggregate-projected twin produce the
**same** `canonical_module_signature` (annotation not hashed) and
dedup-collapse. (5) Full `cargo` hygiene gate; `mdbook` clean with
`book/src/ir.md` "Synthesizable aggregates" reconciled to what landed
and `book/src/knobs.md` documenting `aggregate_*_prob`.

This entry is design-only and is itself task-tree owned
(`PHASE-5B-AGGREGATES.1`); it makes no code change, consistent with
the task-tree-ownership doctrine's code/not-code boundary.

### Phase 5 rules-first pivot (2026-05-16, PHASE-5-PARAMETERIZATION.2.2.1)

Implementation finding that corrects the `.1` design's instantiation
assumption. The `.1`/`.2.1` plan was: build modules normally, then a
post-construction pass marks the width-homogeneous ones parameterized.
A 64-seed forced-on sweep (`width_parameterization_prob = 1.0`,
single width, `constant_prob = 0`, `max_depth = 1`) produced **zero**
width-homogeneous modules: the unconstrained cone generator almost
always introduces a constant, a comparison, a mux, a slice/concat, or
mixed operand widths. Two consequences:

1. **Inert.** A parameterization that only fires when the RNG happens
   to emit a homogeneous module would essentially never fire on real
   output — a feature that cannot trigger is not a capability.
2. **Doctrine violation.** "Generate, then keep the ones that happen to
   qualify" is precisely the generate-then-filter anti-pattern ANVIL
   forbids (valid/structured *by construction*, not by post-hoc
   selection).

**Decision:** keep the `is_width_generic` gate (it is correct and
cheap) but demote it to a post-construction *assertion*, and add a
**rules-first parameterizable-leaf constructor** (`.2.2.2`): when the
knob fires for a module, *construct* it width-homogeneously by rule
(one design width; only width-preserving same-width gates; no
`Constant`/`Slice`/`Concat`/`ForFold`/`Mux`/compare), valid by
construction. The gate then always accepts it. Rejected alternative:
"loosen the gate to parameterize partially-homogeneous modules" —
rejected because a module mixing `[W-1:0]` and `[7:0]` logic that must
agree in width is unsound when `W ≠ 8`; partial parameterization
re-introduces exactly the multi-width unsoundness `.1` set out to
avoid. This does not change architecture (C) (still post-construction
annotation + monomorphic body); it changes *how the body is built* so
the sound subset is reached by rule instead of by luck.

### Phase 5 parameterization design (2026-05-16, PHASE-5-PARAMETERIZATION.1)

Design-only slice. No code. Lifts `book/src/ir.md` "Parameters and
generics (Phase 5)" into a concrete, codebase-grounded implementation +
parameter-aware-identity plan, with rejected alternatives and a proof
shape, so the implementation leaf (`.2`) has an unambiguous target.

**Goal (from ROADMAP Phase 5).** Emitted modules carry `parameter`
declarations for widths; instances pick parameter values from allowed
ranges and override via `#(.W(value))`; parameter-dependent widths
propagate correctly; parameter-aware identity stays sound (distinct
parameter values must not alias to one `NodeId` or one module template
unless genuinely equivalent). Default-off; never retire existing
behaviour.

**Code reality that constrains the design** (audited; key anchors):
width is a bare `u32` everywhere — `Port.width`, `Node::*` width fields,
`Flop.width`, the `intern_gate`/`intern_constant` CSE keys
(`src/ir/types.rs`), the per-op width arithmetic in
`input_widths_for` / `make_width_adapter` (`src/gen/cone.rs`), the
gate-shape + design child-width equality rules (`src/ir/validate.rs`),
the single `width_decl` rendering chokepoint and the parameterless
module header / instance emission (`src/emit/sv.rs`), and the
width-hashing in `canonical_module_signature` (`src/metrics.rs:2187`)
that `src/ir/dedup.rs` groups on. Constant folding/peephole in
`intern_gate`, `make_width_adapter`, `input_widths_for`, `ForFold`
(`trip_count*chunk_width`) and `Slice` (`hi`/`lo` are themselves bare
indices) do **genuine integer arithmetic** and cannot run on opaque
symbolic widths. `shrink_primary_inputs_to_live_width`
(`src/gen/module.rs`) actively rewrites port widths post-construction.

**Architectural decision — chosen: (C) post-construction
parameterization pass + monomorphic instantiation.** Phase 5 lands as a
*post-finalisation pass* (sibling in spirit to the module-dedup pass),
not as a symbolic type threaded through construction:

1. The cone/module is constructed exactly as today, at a concrete
   "design" width `W0` drawn (reproducibly, via `g.rng`) from the new
   parameter's allowed range. All existing fold/validate/cse machinery
   runs unchanged on concrete `u32` — **valid-by-construction is
   preserved with zero changes to the invasive width-arithmetic code**.
2. A post-construction pass marks a *sound parameterizable subset* of
   widths as symbolic in `W`: the interface port widths chosen to carry
   the parameter, plus exactly those internal node widths that the
   construction-time width relations make **affine in `W0`** and that
   stay legal for the whole declared `W` range. Widths that enter
   structurally-constrained integer math (`ForFold trip_count*chunk`,
   `Slice hi/lo`, replicate counts in `make_width_adapter`,
   constant-fold masks) are **excluded from the parameterized set in the
   first slice** — they keep concrete `u32`. The pass records a
   per-module `ParamEnv { name: "W", range: CountRange, design_value:
   W0 }` and a lightweight `WidthExpr` (small enum
   `{ Lit(u32), Param }`, deliberately *not* the full
   `Add/Mul/Clog2/...` algebra yet — see rejected (B)) only on the
   parameterized width sites, each also retaining its resolved `u32`.
3. Instantiation (`src/gen/hierarchy.rs`, between child selection and
   the input-binding loop) picks a value from the param range via
   `g.rng`, records it in a new `Instance.param_bindings`, and binds
   child ports at the **resolved** width so the existing exact-equality
   child-width validation still holds.
4. Emitter: `width_decl` and the module header learn the symbolic form
   (`logic [W-1:0]`, `#( parameter int W = W0 )`); instance emission
   gains `#(.W(value))`. Everywhere a width is *not* in the
   parameterized set, emission is byte-identical to today.

Soundness rule: a module is only emitted parameterized when its chosen
parameterized widths remain legal (validator-clean, downstream-clean)
for **every** value in the declared range — guaranteed by restricting
the parameterized set to affine-in-`W` interface/derived widths and by
the matrix gate sweeping ≥2 values per parameterized scenario. This is
construction-time soundness (a generator rule), not generate-then-filter.

**Parameter-aware identity rule.** The single place width enters module
identity is the per-port/per-node `fnv1a_64_u32(h, width)` calls in
`canonical_module_signature` (`src/metrics.rs`). The rule:
parameterized width sites hash their **normalized symbolic form**
(`WidthExpr::Param` → a fixed sentinel, not `W0`); non-parameterized
sites hash their concrete `u32` as today. Consequence: two
instantiations / monomorphic emissions of the *same template* at W=8
and W=16 produce the **same** signature (legitimately one template — the
existing `dedup_modules` then collapses them with no change to
`dedup.rs`); a genuinely concrete width-7 module still hashes distinctly
and never aliases a parameterized one. `Instance.param_bindings` is
*not* hashed into the parent signature (consistent with the existing
exclusion of `Instance.module`/`name`), so a parent that instantiates
one template at several values keeps one child template — which is the
entire point of parameterization. This extends the doctrine "NodeId =
identity of an expression" / "ModuleId = identity of a hierarchical
module template" to "a parameterized template is one identity across its
legal parameter range".

**Rejected alternatives.**
- **(A) Monomorphize only, emit a symbolic header over a fixed body.**
  Pick `W0`, build the body at `W0`, emit `parameter W=W0` + `[W-1:0]`
  but never make the body width-generic. Rejected: the emitted module is
  a *lie* — overriding `#(.W(16))` on a body built for `W0=8` is not
  valid-by-construction (it would only be correct at `W==W0`). It would
  also force generate-then-filter to avoid bad overrides. Violates the
  by-construction and no-post-hoc-repair doctrines.
- **(B) Full symbolic `WidthExpr{Add,Sub,Mul,Div,Clog2,Max,Min}`
  threaded through the IR from construction.** The book's eventual
  target. Rejected *as the first slice*: it propagates through every
  invasive site in §6 of the audit (all constant folding/peephole, the
  width adapter, `input_widths_for`, `ForFold`, symbolic `Slice`
  indices) — constant folding cannot operate on symbolic widths at all,
  so the e-graph/factorization doctrine would have to be suspended for
  parameterized cones. Too large for one signoff-quality slice and
  high-risk to the existing proven surface. Recorded as the **Phase 5
  follow-on** once (C) is downstream-clean: (C)'s `WidthExpr{Lit,Param}`
  is deliberately the minimal seed of (B)'s algebra, so (B) is a strict
  extension, not a rework.
- **(C') Symbolic widths but disable factorization for parameterized
  modules.** Rejected: silently weakening `identity_mode = node-id`
  for a whole class of modules is exactly the kind of silent
  mode-retirement the project forbids.

**Proof shape for `.2`.** (1) Focused proof: a parameterized module is
emitted with `parameter W` and instantiated at ≥2 distinct in-range
values via `#(.W(v))`; `ir::validate::validate_design` passes; the
emitted SV elaborates/synthesizes clean at each value. (2) Identity
proof: same template at W=8 and W=16 → one `canonical_module_signature`
(and `dedup_modules` collapses them); a concrete non-parameterized
module of width 8 keeps a distinct signature (extends the existing
`dedup_is_a_no_op_when_modules_are_structurally_distinct` test). (3)
Matrix gate: new opt-in knob `width_parameterization_prob` (f64, default
`0.0`, serde-default pattern like `hierarchy_module_dedup`), a
`phase5_*` focus config sweeping the param range, a new
`saw_width_parameterized_design` coverage fact gated under a new
`ScenarioSet::Phase5` (or folded into the Phase 4 design set initially),
proven downstream-clean (Verilator + both Yosys modes) with
`coverage_gaps=[]`. Default-off keeps every existing scenario
byte-identical.

**Open questions (do not block `.2`; recorded for it).**
- Whether Phase 5 gets its own `ScenarioSet::Phase5` gate or rides the
  Phase 4 design harness for the first slice. Lean: ride Phase 4
  harness first (cheaper), split when the parameterized matrix grows.
- Whether `.2` should be split (IR+emit scaffold → instantiation
  substitution → identity rule → matrix gate) — likely yes; `.2` will
  be re-decomposed in the tree when reached.
- Multi-parameter modules and parameter-dependent *depth/count* (not
  just width) are explicitly out of the first slice (ROADMAP notes
  parameter-aware child selection / parameter-driven parent generation
  remain later Phase 5 work).

### Module-dedup pass implemented (2026-05-15, r87, HIERARCHY-AWARE-IDENTITY.4 + .5)
The dedup pass design sketched in `HIERARCHY-AWARE-IDENTITY.3` is now
live as `src/ir/dedup.rs`. Implementation matches the sketch
exactly: pipeline placement (post-finalisation, called from
`Generator::generate_design`), instance-rewrite policy (fixed-point
iteration with lexicographic-smallest-name survivor, top always
preserved by name), toggle/API choice (new `Config::hierarchy_module_dedup:
bool`, default `false`, orthogonal to `IdentityMode`). The
canonical-signature hash is reused from `src/metrics.rs` (exposed as
`pub(crate)` for that purpose — single source of truth).

**Validation evidence:** r87 gate downstream-clean at 210 scenarios /
840 designs / `coverage_gaps = []`. The new
`phase4_hier1_module_dedup_active` matrix scenario per construction
strategy proves dedup runs cleanly through Verilator and both Yosys
modes; the earlier `phase4_hier1_structurally_duplicate_modules`
scenario remains in the bank with dedup off, providing the
side-by-side before/after comparison.

**`HIERARCHY-AWARE-IDENTITY` tree status:** complete. All five leaves
(`.1` canonical signatures, `.2` existence proof, `.3` design sketch,
`.4` implementation, `.5` matrix gate proof) are `done`. The doctrine
extension — "ModuleId = identity of a hierarchical module template"
— is now live under the opt-in `Config::hierarchy_module_dedup`
knob.

### Module-dedup pass design sketch (2026-05-15, HIERARCHY-AWARE-IDENTITY.3)
This is the pre-implementation design sketch for the eventual
`H-A-I.4` dedup pass. No code lands in this slice.

**Pre-conditions established by earlier slices.**

- `H-A-I.1` (r85) gives every `Module` a deterministic 64-bit FNV-1a
  canonical signature exposed as
  `DesignMetrics.canonical_module_signatures`. The signature covers
  port shape, node sequence, drive structure, flop structure, and
  instance interfaces but intentionally excludes `instance.module`
  and `instance.name`. Two structurally-identical Modules with
  distinctly-named children therefore share a signature.
- `H-A-I.2` (r86) proves the planner can emit structurally-duplicate
  Modules under tight 1-in/1-out/width-1 / `max_depth=1` /
  `terminal_reuse_prob=1.0` leaf constraints. The dedup pass has a
  live exercise.

**Pass goal.** Given a finished `Design`, collapse every group of
Modules in `design.modules` that share a canonical signature to a
single surviving entry, and rewrite every `Instance.module` reference
in the remaining Modules so they point at the surviving canonical
peer. Default behaviour stays identical to today; the pass is opt-in.

**Pipeline placement.**

- **Chosen placement:** post-finalisation, after the existing
  per-module `compact_node_ids` pass and right before `Design` is
  returned from `generate_design`. The post-finalisation point is
  the only point where every Module's canonical structure is settled
  (every gate has been compacted, every flop merge has run, every
  `intern_*` retry has completed) — running before then would dedup
  Modules that are not yet in their canonical form.
- **Module location:** new `src/ir/dedup.rs`, alongside
  `src/ir/compact.rs`. Separate file because the operation is
  Design-level (cross-Module), not Module-level (per-Module compaction).
- **Rejected alternative — incremental dedup during construction:**
  i.e., dedup each Module against the existing pool as soon as it's
  emitted. Rejected because (a) ANVIL's planner emits parents
  bottom-up, so dedup at emission time would dedup leaves before
  their children's instances are wired, breaking the
  instance-rewrite contract; and (b) it couples the dedup pass to
  the generator's emission ordering, making future planner changes
  hostile to dedup.
- **Rejected alternative — dedup as an emitter pass in
  `src/emit/sv.rs`:** rejected because emitter doctrine is
  "dumb serialiser, no transformation". Rule 21 / dumb-emitter
  doctrine forbids semantic transformations during emit.

**Instance-rewrite policy.**

1. Compute signatures and group Modules by signature.
2. Within each group, pick the **canonical survivor** as the one
   with the lexicographically-smallest `Module.name`. Deterministic
   tiebreaker for stable output.
3. Build a `name_remap: HashMap<String, String>` from
   merged-away → survivor.
4. Walk every surviving Module's `instances` list; for each
   `Instance.module`, replace with `name_remap.get(...).unwrap_or(self)`.
5. Drop the merged-away Modules from `design.modules`.
6. **Iterate to fixed point.** After one pass, second-level parents
   may now have IDENTICAL instance-graph shapes (because their
   leaves were deduped to a common name). Re-run the pass; new
   duplicates may emerge. Repeat until a pass produces no merges.
   Bottom-up dedup order is the result of fixed-point iteration,
   not an explicit traversal — simpler and provably correct.

**Edge cases.**

- **Top module.** The Design's top must NEVER be merged away. The
  canonical-survivor pick must skip the top, or equivalently always
  pick the top when it appears in a group. Practical implementation:
  exclude the top from the grouping step.
- **Empty design / single-Module design.** No work to do; pass
  returns the Design unchanged. No special-case needed; the grouping
  produces no groups with `count > 1`.
- **Library-mode duplicates.** When `hierarchy_child_source_mode =
  library`, the planner already reuses one Module definition across
  multiple instance slots — so the signature-collision rate at
  library mode is already 0 by construction (the duplicates that
  *would* exist are folded into the library's single definition
  before dedup runs). Dedup is a no-op for library mode unless the
  planner's library construction itself emits structural twins,
  which `H-A-I.2` shows is rare. Most dedup benefit will come from
  on-demand mode.
- **Cycles in instance graph.** Cannot happen — `Design::modules`
  forms a strict DAG (top depends on children depend on
  grandchildren). The fixed-point iteration terminates because
  each iteration reduces `design.modules.len()` strictly, bounded
  below by 1 (the top).
- **Mismatched instance counts after a merged-away module had
  different child references than the survivor.** Cannot happen if
  the signature excludes child-module names but INCLUDES the
  instance interface structure (`role`, `inputs` shape). My current
  signature does both — see `canonical_module_signature` in
  `src/metrics.rs`. So two Modules sharing a signature have the
  same number of instances with the same input wiring; only the
  child names differ, and the rewrite handles that.

**Toggle and API.**

- **Chosen toggle:** a new `Config` knob
  `hierarchy_module_dedup: bool`, default `false`. Plain bool rather
  than an enum variant because the operation is binary (do dedup /
  don't). Future extensions (e.g., dedup-with-aggressive-merging
  beyond canonical signature) would warrant an enum.
- **Rejected alternative — extend `IdentityMode` with a new
  `HierarchicalNodeId` variant.** Rejected because `IdentityMode`
  governs *gate-level* expression identity; extending it to also
  control module-level identity overloads the enum's meaning. The
  existing `IdentityMode::NodeId` doctrine ("NodeId = identity of an
  expression") stands unchanged; the module-level analogue is a
  separate concern and gets its own knob. (`feedback_never_retire_strategies`
  applies: don't retire `IdentityMode::NodeId`, don't silently
  redefine it.)
- **Rejected alternative — extend `FactorizationLevel` ladder with
  a `module-dedup` rung.** Rejected for the same reason: the ladder
  is about gate-level factorization strength, not hierarchy-level
  identity. Dedup at the Module level is orthogonal.

**Proof shape for `H-A-I.4`.**

- **Focused proof:** build a 4-leaf design under the
  `H-A-I.2` tight-leaf config. Compute metrics without dedup:
  `num_modules = 5`, `num_distinct_module_signatures = 2`,
  `num_structurally_duplicate_module_pairs = 6`. Run dedup. Re-compute
  metrics: `num_modules = 2` (top + the surviving leaf),
  `num_distinct = 2`, `num_pairs = 0`. Validate the resulting Design
  via `validate_design` to ensure no broken instance references.
- **Matrix scenario:** mirror `H-A-I.2`'s tight-leaf scenario but
  with the dedup toggle on. New saw fact
  `saw_design_with_module_dedup_active` requires `num_pairs == 0`
  AND `num_modules` strictly less than what `H-A-I.2`'s peer
  scenario emits. Both scenarios stay in the bank so the
  before/after comparison is visible.
- **Default-off preservation:** the existing `H-A-I.2` scenario
  (`phase4_hier1_structurally_duplicate_modules`) must continue to
  produce `num_structurally_duplicate_module_pairs > 0` after
  `H-A-I.4` lands, proving the toggle defaults off.

**Open questions for `H-A-I.4` implementation.**

- Should dedup also remove unused Modules (modules that no Instance
  in the surviving Module set references)? The existing
  `num_unused_module_definitions` metric flags this — the dedup
  pass could opportunistically clean up, OR a separate
  `prune_unused_modules` pass could be a sibling slice. Likely the
  latter (single responsibility).
- Should the survivor's name be re-emitted (e.g., `mod_42_merged`)
  to make the dedup visible in the SV output? Or keep the
  lexicographically-smallest original name? Default-keep is
  cheaper; explicit re-emit is more debuggable.
- Should we emit a manifest entry recording which Modules were
  deduped onto which survivors? Useful for downstream tools that
  want to back-trace; trivial to add via a new
  `DesignMetrics.dedup_remap: BTreeMap<String, String>`.

**Slice budget for `H-A-I.4`.** Implementation should fit in one
slice: ~50 lines in `src/ir/dedup.rs`, ~20 lines wiring the toggle
in `Config`, ~30 lines of focused proof, ~15 lines of matrix
scenario + saw fact. No new dependency on external crates.

## Workflow notes
### Task-tree ownership is mandatory for all code changes (2026-05-17, owner directive)

**Doctrine, non-negotiable, no compromise.** It is strictly forbidden
to make any code change without it being task-tree tracked or
task-tree owned **first**. This supersedes the earlier "task trees are
opt-in per top-level task" / "stay on `rN` for linear coverage" scope:
that softer framing no longer governs code.

**Why:** the owner observed that task-tree ownership improved code
review and code quality *tremendously* over the ad-hoc / linear-`rN`
cadence — the recursive breakdown, explicit frontier, recorded
decisions/blockers, and the 1:1 leaf↔commit mapping force each change
to be scoped, justified, and reviewable before it lands, and make
pause/resume recovery lossless. The empirical improvement, not a
process preference, is the rationale.

**Boundary.** "Code" = anything that changes program/generator
behaviour or generated RTL (`src/`, `tests/`, `examples/`,
build/codegen logic, behaviour-altering `Cargo` manifests). Pure-docs
/ live-doc / mdBook / workflow-config edits and recording doctrine
itself are *not* code changes and need no tree (this very entry is an
example). `rN` is **not** retired — it survives only as the optional
within-leaf slice cadence *inside* a tree; a bare unowned `rN` code
slice is no longer legal.

**Mechanics.** Before editing code, confirm/create the owning leaf
(`docs/tasks/<TREE>.md` + `docs/TASK_TREE.md` row); leaf ID in the
commit subject; one completed leaf per commit; the frontier names the
next eligible leaf. Recorded across `COMMIT.md`, `docs/TASK_TREE.md`
("ANVIL Adoption Scope"), `SESSION_BOOTSTRAP.md`, this file,
`README.md`, and the mdBook (`architecture.md`); session memory
`feedback_task_tree_available.md`. Keep all in sync if the policy
ever changes.

### Coverage baseline established (2026-05-14, COVERAGE-INSTRUMENTATION.1)
cargo-llvm-cov 0.8.7 + llvm-tools-aarch64-apple-darwin already
installed locally. Baseline run via `cargo llvm-cov --release`
(intentionally excludes the 75-min Phase 4 hierarchy matrix gate so
the baseline stays reproducible in minutes). Result: **85.26% lines,
91.95% functions, 87.61% regions** across 14 crate files. Full
per-file breakdown lives in `docs/coverage-baseline.md`.

**Key signal:** the planner core (`gen/hierarchy.rs`, `gen/module.rs`,
`gen/cone.rs`, `ir/compact.rs`, `emit/sv.rs`) sits at 88-99% lines
*without* the matrix gate's 204 scenarios contributing. That confirms
the focused-proof + unit-test combination already exercises the
construction discipline comprehensively, not just at the macro
(matrix-gate) level. `metrics.rs` is at 99.66% — meaning the
detection helpers (`binding_uses_*`, canonical-signature hash,
ratio computations) are very densely tested by the focused proofs
the recent rN slices added.

**Top-5 under-covered files (for `.2` triage):**

1. `bin/tool_matrix.rs` — 1951 lines, 72.07%. Matrix-gate-only paths.
2. `gen/cone.rs` — 454 lines, 88.65%. The only planner-core file
   outside the 95%+ band; likely anti-collapse rollback paths.
3. `ir/validate.rs` — 254 lines, 75.07%. Mostly defensive panics
   ("this case cannot happen" invariants); expected.
4. `config.rs` — 250 lines, 67.87%. CLI overlay variants.
5. `main.rs` — 142 lines, 60.56%. Clap derives + flag plumbing.

`.2` produces a disposition matrix per file: (a) dead code -> remove,
(b) rarely-fired path -> add focused proof, (c) defensive
unreachable -> leave and document.

### Registered three quality-improvement task trees (2026-05-14)
Added active task trees for the three quality dials discussed in
the session that prompted task-tree adoption itself:

- `INSTA-SNAPSHOTS` — `insta`-backed snapshot tests of generator
  output, enforcing the "byte-identical forever" reproducibility
  contract directly. Currently provable only by intent.
- `DIFFERENTIAL-SIMULATION` — cross-simulator semantic equivalence
  (Verilator + iverilog at minimum). Raises the downstream contract
  from "parses and synthesises" to "all observers agree on semantics".
- `COVERAGE-INSTRUMENTATION` — `cargo-llvm-cov`-backed coverage
  reports converting matrix-comprehensiveness from intent to
  measurement.

**Rationale.** ANVIL already does the rarest hard thing right:
validity by construction. The remaining quality dial is *consistency
across observers* — different simulators, different runs, different
platforms, different code paths. Each tree owns one orthogonal axis
of that dial; together they cover the "signoff-level random RTL"
ambition stated in `README.md` along its three reachable directions.

**Sequencing intent.** No leaf is `in_progress`. When the user opens
a quality slice, the natural order is INSTA-SNAPSHOTS.1 (cheapest,
nothing else depends on it), then COVERAGE-INSTRUMENTATION.1 (medium
cost, exposes planner test gaps), then DIFFERENTIAL-SIMULATION.1
(highest cost but highest signoff payoff). The user picks; the trees
just make the scope durable.

**Rejected alternative.** Folding all three into a single
`SIGNOFF-QUALITY` umbrella tree. Rejected: the three are
operationally independent (one can ship without the others), and
collapsing them would hide which axis is being worked on at any
given moment.

### Adopted FSMGen task-tree workflow on ANVIL (2026-05-14)
Added a repo-local task-tree tracking workflow at `docs/TASK_TREE.md`
plus the portable setup guide at `docs/TASK_TREE_README.md` (lifted from
FSMGen's `docs/TASK_TREE_README.md`). One initial active tree:
`docs/tasks/HIERARCHY-AWARE-IDENTITY.md`, covering the hierarchy-aware
identity work that r85 opened.

**Scope decision:** task trees are opt-in per top-level task on ANVIL,
not mandatory. Linear `rN` coverage slices (r73-r82 depth sweeps, r83
three-stage chain, r84 helper budget 5) already had clean handoff under
the `rN` + `CHANGES.md` + `MEMORY.md` combination — adding leaf-IDs and
per-leaf task files there would mostly add overhead without solving a
real problem. The value of task-tree is highest where the work has:
more than ~3 planned sub-slices, real blockers or design decisions to
record, parallel sub-axes that do not fit a single linear `rN` ladder,
or is likely to span multiple sessions with pause/resume cycles. The
upcoming hierarchy-aware-identity dedup work fits all four; the closed
depth sweeps fit none.

**Rejected alternative:** full FSMGen-style mandate ("all work is
task-tree-managed by default"). FSMGen's ISF lane has that policy
because every ISF objective has multiple independent dimensions; ANVIL's
linear `rN` shape does not.

**Commit-workflow tie-in:** `COMMIT.md` gained a "Task-tree-managed
commits" section requiring the leaf ID in commit subjects when work is
task-tree-managed, and same-commit updates to the owning
`docs/tasks/<TREE>.md` file. Non-task-tree commits (linear `rN`,
isolated doc edits) follow the standard checklist without the
leaf-ID rule.

## Calibration notes
### Phase 4 r86 proves the planner can emit structurally-duplicate Modules downstream-clean (HIERARCHY-AWARE-IDENTITY.2)
The latest full downstream-clean Phase 4 hierarchy evidence anchor is
now `/tmp/anvil-tool-matrix-phase4-hierarchy-r86/tool_matrix_report.json`:
207 scenarios / 828 designs, `coverage_gaps = []`, Verilator/Yosys
all 828/0. Closes leaf `HIERARCHY-AWARE-IDENTITY.2`.

**Calibration discovery.** Initial 500-config sweep (varying
num_leaf_modules, num_child_instances, seed, strategy with default
leaf-input/output ranges) produced **zero** structurally-duplicate
Module pairs. The leaf generator's RNG advances between calls, so two
leaves with the same interface profile but different RNG states
produce different gate structures by default.

**Calibration choice.** Tight 1-input / 1-output / width-1 leaves with
`max_depth = 1` and `terminal_reuse_prob = 1.0` collapse the leaf
generator's degrees of freedom: there's essentially one legal
"drive output from the lone input" structure. Under these
constraints, every library leaf hashes to the same canonical
signature, so a depth-1 wrapper with 4 leaves produces a
4*(4-1)/2 = 6 duplicate-pair design.

**Implication for `H-A-I.4` (dedup pass).** Dedup is therefore real
and applicable to ANVIL's planner. The dedup pass will need to:
(a) merge Module definitions sharing a canonical signature, and
(b) remap every `Instance.module` string in the rest of the design
to point at the surviving merged definition. Both passes are
straightforward over `Design::modules`. The opt-in toggle is left
to `H-A-I.4` for the design sketch.

### Phase 4 r85 lands canonical module signatures as the first slice of hierarchy-aware identity downstream-clean
The latest full downstream-clean Phase 4 hierarchy evidence anchor is now
`/tmp/anvil-tool-matrix-phase4-hierarchy-r85/tool_matrix_report.json`:
204 scenarios / 816 designs, `coverage_gaps = []`, Verilator/Yosys all
816/0. PNT-3 of the autonomous-PNT chain. Each module gets a
dependency-free FNV-1a 64-bit signature covering port shape, node
sequence, drive structure, flop structure, and instance interfaces. The
hash deliberately omits `instance.module` and `instance.name` so two
parents that instantiate distinctly-named-but-identically-shaped
children share a signature — that isomorphism awareness is what makes
the signature useful for future `Design::modules` deduplication.
Calibration: depth 2, 4,4 child instances,
`hierarchy_child_input_cone_prob = 1.0`, no helpers, no flops, no
sibling routing — a vanilla recursive hierarchy that produces multiple
distinct module shapes so the diversity fact (`num_distinct >= 2`)
fires reliably.

### Phase 4 r84 proves a recursive non-top internal parent can saturate a parent-cone helper budget of 5 helpers downstream-clean
The latest full downstream-clean Phase 4 hierarchy evidence anchor is now
`/tmp/anvil-tool-matrix-phase4-hierarchy-r84/tool_matrix_report.json`:
201 scenarios / 804 designs, `coverage_gaps = []`, Verilator/Yosys all
804/0. Second slice of the broader-Phase-4 work (PNT-2 of the
autonomous-PNT chain). Extends the helper-budget axis from 3 (previous
saturating proof) to 5. Calibration: depth 2, 4,4 child instances,
`max_parent_cone_instances_per_module = 5`,
`hierarchy_child_input_cone_prob = 1.0`, and
`hierarchy_parent_cone_instance_prob = 1.0`. Each non-top internal
parent has ~4 children x ~2 inputs = 8 child-input decision sites,
giving the planner enough demand to fully saturate the budget-5
allocation per parent.

### Phase 4 r83 proves recursive non-top registered parent-composed three-stage chain downstream-clean
The latest full downstream-clean Phase 4 hierarchy evidence anchor is now
`/tmp/anvil-tool-matrix-phase4-hierarchy-r83/tool_matrix_report.json`:
198 scenarios / 792 designs, `coverage_gaps = []`, Verilator/Yosys all
792/0. First slice of the broader-Phase-4 work after the depth-7 sweep
closed in r82. Promotes a new chain-depth axis on top of the closed
depth-3..7 sweeps: registered parent-composed child-input bindings can
chain through three parent-local flop stages without helper instances
below the top parent. Calibration: depth 3, 4,4 child instances,
`max_flops_per_module = 128`, `max_depth = 8`. These limits give the
planner enough flop budget and cone depth to naturally produce
chain-length-3 structures below the top across all four construction
strategies; the planner has no explicit chain-length knob, so the new
detection just walks the existing FlopQ -> D chain three deep and
counts bindings whose Q's D is a non-slice/non-concat gate over both
instance outputs and another Q.

### Phase 4 r82 closes the depth-7 sweep with recursive non-top stateful unregistered parent-composed mixed-support child inputs without helpers downstream-clean (2,2 calibrated)
The latest full downstream-clean Phase 4 hierarchy evidence anchor is now
`/tmp/anvil-tool-matrix-phase4-hierarchy-r82/tool_matrix_report.json`:
195 scenarios / 780 designs, `coverage_gaps = []`, Verilator/Yosys all
780/0. Fifth and final slice of the depth-7 sweep mirroring
r77/r72/r67/r62 — closes the depth-7 axis. Calibration: depth-7 stateful
mixed-support cells use the same 2,2 child-instance bounds as r77 at
depth 6 and r79 at depth 7 (mixed-support cells at depths ≥ 6 use 2,2
because the 4,4 tree at depth 7 would yield ~5461 internal occurrences,
far beyond a safe-slice budget). The depth-7 axis is now fully closed:
all five cells (parent-flops r78, mixed-support child inputs r79,
parent-port-composed outputs r80, stateful parent-port-composed outputs
r81, stateful mixed-support child inputs r82) are first-class
downstream-clean coverage facts.

### Phase 4 r81 extended the depth-7 axis with recursive non-top stateful parent-port-composed parent outputs without helpers downstream-clean
The latest full downstream-clean Phase 4 hierarchy evidence anchor is now
`/tmp/anvil-tool-matrix-phase4-hierarchy-r81/tool_matrix_report.json`:
192 scenarios / 768 designs, `coverage_gaps = []`, Verilator/Yosys all
768/0. Fourth slice of the depth-7 sweep mirroring r76/r71/r66/r61.
Only one cell remained to close depth-7: stateful mixed-support child
inputs (r82, with the same 2,2 calibration as r74/r77/r79).

### Phase 4 r80 extended the depth-7 axis with recursive non-top parent-port-composed parent outputs without helpers or parent-local state downstream-clean
The latest full downstream-clean Phase 4 hierarchy evidence anchor is now
`/tmp/anvil-tool-matrix-phase4-hierarchy-r80/tool_matrix_report.json`:
189 scenarios / 756 designs, `coverage_gaps = []`, Verilator/Yosys all
756/0. Third slice of the depth-7 sweep mirroring r75/r70/r65/r60.
Parent-port-composed cells already use 2,2 children at all depths so no
calibration drift here.

### Phase 4 r79 extended the depth-7 axis with recursive non-top mixed-support child inputs without helpers downstream-clean (2,2 calibrated)
The latest full downstream-clean Phase 4 hierarchy evidence anchor is now
`/tmp/anvil-tool-matrix-phase4-hierarchy-r79/tool_matrix_report.json`. It
keeps the live hierarchy policy at four designs per scenario and expands
it to 186 scenarios / 744 designs, with `coverage_gaps = []`,
`artifact_kind = "design"`, Verilator `744/0`, Yosys without-ABC
`744/0`, and Yosys with-ABC `744/0`.

This bank extends the depth-7 axis (opened by r78 with parent flops) to
the unregistered parent-composed mixed-support child-input surface,
mirroring r74 (depth 6), r69 (depth 5), r64 (depth 4), and r59
(depth 3). Smoke at depth 7 with 2,2 child instances confirmed 127
internal module occurrences with `child_input_bindings_from_parent_composed_logic = 219`
versus 1 top-only and `child_input_bindings_from_mixed_support = 173`
versus 1 top-only.

**Calibration:** depth-7 mixed-support cells continue the 2,2
child-instance calibration introduced at depth 6. The 4,4 tree at
depth 7 would yield ~5461 internal occurrences, far beyond a safe-slice
budget for downstream-clean tools. 2,2 at depth 7 still proves the
mixed-support surface cleanly.

The slice does not change the generator: it tightens the gate around an
already-supported capability.

Current-code validation includes the focused recursive pipeline
regression, `cargo test --bin tool_matrix`, and the full r79 Phase 4
hierarchy gate through Verilator plus both repo-owned Yosys modes.

### Phase 4 r78 opened the depth-7 axis with recursive non-top parent-local flops downstream-clean
The latest full downstream-clean Phase 4 hierarchy evidence anchor is now
`/tmp/anvil-tool-matrix-phase4-hierarchy-r78/tool_matrix_report.json`. It
keeps the live hierarchy policy at four designs per scenario and expands
it to 183 scenarios / 732 designs, with `coverage_gaps = []`,
`artifact_kind = "design"`, Verilator `732/0`, Yosys without-ABC
`732/0`, and Yosys with-ABC `732/0`.

This bank opens the depth-7 axis, mirroring how r73 opened depth-6
above the closed depth-5 sweep, r68 opened depth-5 above the closed
depth-4 sweep, and r63 opened depth-4 above the closed depth-3 sweep.
Smoke at depth 7 with 2,2 child instances confirmed 127 non-top
internal-parent occurrences with `hierarchy_parent_local_flops = 8122`
versus `top_local_flops = 64` and 127 internal occurrences carrying
parent-local flops.

The depth-6 sweep closed in r77 with all five mixed-support cells gated
as first-class facts; r78 now starts the depth-7 sweep with the
simplest surface — parent flops at depth 7 — as a foothold. Future
r79..r82 will close the depth-7 sweep mirroring r58..r62 (depth 3),
r63..r67 (depth 4), r68..r72 (depth 5), and r73..r77 (depth 6).
Mixed-support cells at depth 7 will adopt the 2,2 child-instance
calibration introduced at depth 6.

The slice does not change the generator: it tightens the gate around an
already-supported capability.

Current-code validation includes the focused recursive pipeline
regression, `cargo test --bin tool_matrix`, and the full r78 Phase 4
hierarchy gate through Verilator plus both repo-owned Yosys modes.

### Phase 4 r77 closed the depth-6 sweep with recursive non-top stateful unregistered parent-composed mixed-support child inputs without helpers downstream-clean (2,2 calibrated)
The latest full downstream-clean Phase 4 hierarchy evidence anchor is now
`/tmp/anvil-tool-matrix-phase4-hierarchy-r77/tool_matrix_report.json`. It
keeps the live hierarchy policy at four designs per scenario and expands
it to 180 scenarios / 720 designs, with `coverage_gaps = []`,
`artifact_kind = "design"`, Verilator `720/0`, Yosys without-ABC
`720/0`, and Yosys with-ABC `720/0`.

This bank closes the depth-6 sweep. r73 opened the depth-6 axis with
parent flops, r74 extended with mixed-support child inputs (2,2
calibrated), r75 with parent-port-composed parent outputs, r76 with
stateful parent-port-composed parent outputs. r77 closes the sweep
with stateful unregistered parent-composed mixed-support child inputs,
mirroring r72 (depth 5), r67 (depth 4), and r62 (depth 3).

**Calibration follow-on:** depth-6 stateful mixed-support cells use the
same 2,2 child-instance calibration adopted by r74. Smoke confirmed 63
internal module occurrences with `hierarchy_parent_local_flops = 4032`
versus `top_local_flops = 64`,
`child_input_bindings_from_stateful_parent_composed_mixed_support = 74`
versus 1 top-only, and
`stateful_parent_composed_mixed_support_child_input_binding_fraction
= 0.454`.

The depth-6 axis now has all five mixed-support cells gated as
first-class coverage facts, mirroring closed depth-3 (r58..r62),
depth-4 (r63..r67), and depth-5 (r68..r72) sweeps. The Phase 4 depth
sweep template is now consistent across depths 3-6.

The slice does not change the generator: it tightens the gate around an
already-supported capability.

Current-code validation includes the focused recursive pipeline
regression, `cargo test --bin tool_matrix`, and the full r77 Phase 4
hierarchy gate through Verilator plus both repo-owned Yosys modes.

### Phase 4 r76 extended the depth-6 axis with recursive non-top stateful parent-port-composed parent outputs without helpers downstream-clean
The latest full downstream-clean Phase 4 hierarchy evidence anchor is now
`/tmp/anvil-tool-matrix-phase4-hierarchy-r76/tool_matrix_report.json`. It
keeps the live hierarchy policy at four designs per scenario and expands
it to 177 scenarios / 708 designs, with `coverage_gaps = []`,
`artifact_kind = "design"`, Verilator `708/0`, Yosys without-ABC
`708/0`, and Yosys with-ABC `708/0`.

This bank extends the depth-6 axis (opened by r73 with parent flops,
extended by r74 with mixed-support child inputs and r75 with
parent-port-composed parent outputs) to the stateful
parent-port-composed parent-output surface, mirroring r71 (depth 5),
r66 (depth 4), and r61 (depth 3). Smoke at depth 6 with 2,2 child
instances confirmed 63 internal module occurrences with
`hierarchy_parent_local_flops = 4028` versus `top_local_flops = 64`,
`hierarchy_parent_port_composed_outputs = 960` versus 160 top-only,
`hierarchy_parent_port_composed_outputs_through_parent_flops = 890`
versus 109 top-only, and
`hierarchy_parent_port_composed_parent_flop_output_fraction = 0.927`.

Only one cell remains to close the depth-6 sweep: stateful unregistered
parent-composed mixed-support child inputs (r77).

The slice does not change the generator: it tightens the gate around an
already-supported capability.

Current-code validation includes the focused recursive pipeline
regression, `cargo test --bin tool_matrix`, and the full r76 Phase 4
hierarchy gate through Verilator plus both repo-owned Yosys modes.

### Phase 4 r75 extended the depth-6 axis with recursive non-top parent-port-composed parent outputs without helpers or parent-local state downstream-clean
The latest full downstream-clean Phase 4 hierarchy evidence anchor is now
`/tmp/anvil-tool-matrix-phase4-hierarchy-r75/tool_matrix_report.json`. It
keeps the live hierarchy policy at four designs per scenario and expands
it to 174 scenarios / 696 designs, with `coverage_gaps = []`,
`artifact_kind = "design"`, Verilator `696/0`, Yosys without-ABC
`696/0`, and Yosys with-ABC `696/0`.

This bank extends the depth-6 axis (opened by r73 with parent flops and
extended by r74 with mixed-support child inputs) to the unregistered
parent-port-composed parent-output surface, mirroring r70 (depth 5),
r65 (depth 4), and r60 (depth 3). Smoke confirmed 63 internal module
occurrences with `hierarchy_parent_port_composed_outputs = 1008` versus
`top_parent_port_composed_outputs = 168` and a
`hierarchy_parent_port_composed_output_fraction = 1.0` at depth 6 with
2,2 child-instance bounds. No calibration drift — parent-port-composed
cells use 2,2 at all depths.

The slice does not change the generator: it tightens the gate around an
already-supported capability.

Current-code validation includes the focused recursive pipeline
regression, `cargo test --bin tool_matrix`, and the full r75 Phase 4
hierarchy gate through Verilator plus both repo-owned Yosys modes.

### Phase 4 r74 extended the depth-6 axis with recursive non-top mixed-support child inputs without helpers downstream-clean
The latest full downstream-clean Phase 4 hierarchy evidence anchor is now
`/tmp/anvil-tool-matrix-phase4-hierarchy-r74/tool_matrix_report.json`. It
keeps the live hierarchy policy at four designs per scenario and expands
it to 171 scenarios / 684 designs, with `coverage_gaps = []`,
`artifact_kind = "design"`, Verilator `684/0`, Yosys without-ABC
`684/0`, and Yosys with-ABC `684/0`.

This bank extends the depth-6 axis (opened by r73 with parent flops) to
the unregistered parent-composed mixed-support child-input surface,
mirroring how r69 followed r68 at depth 5, r64 followed r63 at depth 4,
and r59 followed r58 at depth 3.

**Calibration: depth-6 mixed-support cells use 2,2 child-instance
bounds, not the 4,4 used at depths 3-5.** Smoke at depth 6 with 4,4
showed 1365 internal module occurrences (4× the d5 count of 341);
yosys-with-abc spent 22+ minutes on a single design, projecting to ~10h
per gate. That exceeds a safe-slice budget for a 10-step batch. The 2,2
calibration at depth 6 yields 63 occurrences (matching r73's
parent-flop scenario) and proves the same surface cleanly: focused
proof passes in 0.42s release. This is a slice-time calibration choice,
not a strategy retirement — the 4,4 mixed-support cells at d3-d5 remain
unchanged. r77 (stateful mixed-support at d6) will adopt the same 2,2
calibration. If a future workstream wants a downstream-clean d6 4,4
mixed-support proof, that can land as a separate slice with a longer
budget.

The slice does not change the generator: it tightens the gate around an
already-supported capability.

Current-code validation includes the focused recursive pipeline
regression, `cargo test --bin tool_matrix`, and the full r74 Phase 4
hierarchy gate through Verilator plus both repo-owned Yosys modes.

### Phase 4 r73 opened the depth-6 axis with recursive non-top parent-local flops downstream-clean
The latest full downstream-clean Phase 4 hierarchy evidence anchor is now
`/tmp/anvil-tool-matrix-phase4-hierarchy-r73/tool_matrix_report.json`. It
keeps the live hierarchy policy at four designs per scenario and expands
it to 168 scenarios / 672 designs, with `coverage_gaps = []`,
`artifact_kind = "design"`, Verilator `672/0`, Yosys without-ABC
`672/0`, and Yosys with-ABC `672/0`.

This bank opens the depth-6 axis, mirroring how r68 opened depth-5
above the closed depth-4 sweep, and r63 opened depth-4 above the closed
depth-3 sweep. Smoke at depth 6 with 2,2 child instances confirmed 63
non-top internal-parent occurrences with `hierarchy_parent_local_flops
= 4028` versus `top_local_flops = 64` and 63 internal occurrences
carrying parent-local flops.

The depth-5 sweep closed in r72 with all five mixed-support cells gated
as first-class facts; r73 now starts the depth-6 sweep with the
simplest surface — parent flops at depth 6 — as a foothold. Future
r74..r77 slices will close the depth-6 sweep mirroring r58..r62 (depth
3), r63..r67 (depth 4), and r68..r72 (depth 5).

The slice does not change the generator: it tightens the gate around an
already-supported capability.

Current-code validation includes the focused recursive pipeline
regression, `cargo test --bin tool_matrix`, and the full r73 Phase 4
hierarchy gate through Verilator plus both repo-owned Yosys modes.

### Phase 4 r72 closed the depth-5 sweep with recursive non-top stateful unregistered parent-composed mixed-support child inputs without helpers downstream-clean
The latest full downstream-clean Phase 4 hierarchy evidence anchor is now
`/tmp/anvil-tool-matrix-phase4-hierarchy-r72/tool_matrix_report.json`. It
keeps the live hierarchy policy at four designs per scenario and expands
it to 165 scenarios / 660 designs, with `coverage_gaps = []`,
`artifact_kind = "design"`, Verilator `660/0`, Yosys without-ABC
`660/0`, and Yosys with-ABC `660/0`.

This bank closes the depth-5 sweep. r68 opened the depth-5 axis with
parent flops, r69 extended it with mixed-support child inputs, r70 with
parent-port-composed parent outputs, r71 with stateful parent-port-composed
parent outputs. r72 closes the sweep with stateful unregistered
parent-composed mixed-support child inputs, mirroring how r67 closed
depth 4 and r62 closed depth 3. Smoke confirmed 341 internal module
occurrences with `hierarchy_parent_local_flops = 21820` versus
`top_local_flops = 64`, `child_input_bindings_from_parent_composed_logic
= 1777` versus 3 top-only, `child_input_bindings_from_stateful_parent_composed_mixed_support
= 1460` versus 2 top-only, and
`stateful_parent_composed_mixed_support_child_input_binding_fraction
= 0.642` at depth 5 with `4,4` child-instance bounds.

The depth-5 axis now has all five mixed-support cells gated as
first-class coverage facts, mirroring the closed depth-3 (r58..r62) and
depth-4 (r63..r67) sweeps. Future Phase 4 work can pursue depth 6 or
broaden the registered-helper / multi-helper surface.

The slice does not change the generator: it tightens the gate around an
already-supported capability.

Current-code validation includes the focused recursive pipeline
regression, `cargo test --bin tool_matrix`, and the full r72 Phase 4
hierarchy gate through Verilator plus both repo-owned Yosys modes.

### Phase 4 r71 extended the depth-5 axis with recursive non-top stateful parent-port-composed parent outputs without helpers downstream-clean
The previous full downstream-clean Phase 4 hierarchy evidence anchor is
`/tmp/anvil-tool-matrix-phase4-hierarchy-r71/tool_matrix_report.json`. It
keeps the live hierarchy policy at four designs per scenario and expands
it to 162 scenarios / 648 designs, with `coverage_gaps = []`,
`artifact_kind = "design"`, Verilator `648/0`, Yosys without-ABC
`648/0`, and Yosys with-ABC `648/0`.

This bank extends the depth-5 axis (opened by r68 with parent flops,
extended by r69 with mixed-support child inputs, and extended by r70
with parent-port-composed parent outputs) to the stateful
parent-port-composed parent-output surface, mirroring how r66 followed
r65 at depth 4 and r61 followed r60 at depth 3. Smoke confirmed 31
internal module occurrences with `hierarchy_parent_local_flops = 1980`
versus `top_local_flops = 64`, `hierarchy_parent_port_composed_outputs
= 340` versus 68 top-only, `hierarchy_parent_port_composed_outputs_through_parent_flops
= 336` versus 64 top-only, and `hierarchy_parent_port_composed_parent_flop_output_fraction
= 0.988` at depth 5 with `2,2` child-instance bounds.

Only one cell remains to close the depth-5 sweep: stateful unregistered
parent-composed mixed-support child inputs (depth-3 territory r62 /
depth-4 territory r67).

The slice does not change the generator: it tightens the gate around an
already-supported capability.

Current-code validation includes the focused recursive pipeline
regression, `cargo test --bin tool_matrix`, and the full r71 Phase 4
hierarchy gate through Verilator plus both repo-owned Yosys modes.

### Phase 4 r70 extended the depth-5 axis with recursive non-top parent-port-composed parent outputs without helpers or parent-local state downstream-clean
The previous full downstream-clean Phase 4 hierarchy evidence anchor is
`/tmp/anvil-tool-matrix-phase4-hierarchy-r70/tool_matrix_report.json`. It
keeps the live hierarchy policy at four designs per scenario and expands
it to 159 scenarios / 636 designs, with `coverage_gaps = []`,
`artifact_kind = "design"`, Verilator `636/0`, Yosys without-ABC
`636/0`, and Yosys with-ABC `636/0`.

This bank extends the depth-5 axis (opened by r68 with parent flops and
extended by r69 with mixed-support child inputs) to the unregistered
parent-port-composed parent-output surface, mirroring how r65 followed
r64 at depth 4 and r60 followed r59 at depth 3. Smoke confirmed 31
internal module occurrences with `hierarchy_parent_port_composed_outputs
= 390` versus `top_parent_port_composed_outputs = 78` and a
`hierarchy_parent_port_composed_output_fraction = 1.0` at depth 5 with
`2,2` child-instance bounds.

The slice does not change the generator: it tightens the gate around an
already-supported capability.

Current-code validation includes the focused recursive pipeline
regression, `cargo test --bin tool_matrix`, and the full r70 Phase 4
hierarchy gate through Verilator plus both repo-owned Yosys modes.

### Phase 4 r69 extended the depth-5 axis with recursive non-top mixed-support child inputs without helpers downstream-clean
The previous full downstream-clean Phase 4 hierarchy evidence anchor is
`/tmp/anvil-tool-matrix-phase4-hierarchy-r69/tool_matrix_report.json`. It
keeps the live hierarchy policy at four designs per scenario and expands
it to 156 scenarios / 624 designs, with `coverage_gaps = []`,
`artifact_kind = "design"`, Verilator `624/0`, Yosys without-ABC
`624/0`, and Yosys with-ABC `624/0`.

This bank extends the depth-5 axis (opened by r68) to the unregistered
parent-composed mixed-support child-input surface, mirroring how r64
followed r63 at depth 4 and r59 followed r58 at depth 3. Smoke confirmed
341 internal module occurrences with 1457 hierarchy-wide vs 3 top-only
mixed-support bindings and 1599 vs 3 parent-composed bindings at
depth 5.

The slice does not change the generator: it tightens the gate around an
already-supported capability.

Current-code validation includes the focused recursive pipeline
regression, `cargo test --bin tool_matrix`, and the full r69 Phase 4
hierarchy gate through Verilator plus both repo-owned Yosys modes.

### Phase 4 r68 opened the depth-5 axis with recursive non-top parent-local flops downstream-clean
The previous full downstream-clean Phase 4 hierarchy evidence anchor is
`/tmp/anvil-tool-matrix-phase4-hierarchy-r68/tool_matrix_report.json`. It
keeps the live hierarchy policy at four designs per scenario and expands
it to 153 scenarios / 612 designs, with `coverage_gaps = []`,
`artifact_kind = "design"`, Verilator `612/0`, Yosys without-ABC
`612/0`, and Yosys with-ABC `612/0`.

This bank opens the depth-5 axis. The depth-4 sweep was structurally
complete in r67 (all five mixed-support cells covered: parent-flops,
no-state and stateful child-input mixed-support, no-state and stateful
parent-output mixed-support). r68 starts the depth-5 axis with the
simplest surface — parent flops at depth 5 — by adding
`saw_recursive_hierarchy_depth_5_parent_local_flops` (coverage gap when
missing) plus the focused proof
`recursive_hierarchy_parents_can_emit_local_flops_at_depth_5` and the
matrix scenario `phase4_recur_d5_parent_state` per construction strategy
(`2,2` child-instance bounds, four intermediate parent layers below the
top). Smoke at depth 5 confirmed 31 internal module occurrences with
1984 hierarchy-wide parent-local flops versus 64 top-only, so the
recursive generator handles depth-5 nesting cleanly.

The slice does not change the generator: it tightens the gate around an
already-supported capability.

Current-code validation includes the focused recursive pipeline
regression, `cargo test --bin tool_matrix`, and the full r68 Phase 4
hierarchy gate through Verilator plus both repo-owned Yosys modes.

### Phase 4 r67 closed the depth-4 sweep with recursive non-top stateful parent-composed mixed-support child inputs without helpers downstream-clean
The previous full downstream-clean Phase 4 hierarchy evidence anchor is
`/tmp/anvil-tool-matrix-phase4-hierarchy-r67/tool_matrix_report.json`. It
keeps the live hierarchy policy at four designs per scenario and expands
it to 150 scenarios / 600 designs, with `coverage_gaps = []`,
`artifact_kind = "design"`, Verilator `600/0`, Yosys without-ABC
`600/0`, and Yosys with-ABC `600/0`.

This bank closes the depth-4 sweep, mirroring how r62 closed the depth-3
sweep. The depth-4 axis now covers parent-flops (r63), no-state
mixed-support child inputs (r64), no-state parent-port-composed outputs
(r65), stateful parent-port-composed outputs (r66), and stateful
unregistered parent-composed mixed-support child inputs (r67). The new
`saw_recursive_hierarchy_depth_4_stateful_parent_composed_mixed_support_child_inputs`
fact (coverage gap when missing) plus the focused proof
`recursive_hierarchy_stateful_parent_composed_routes_mix_parent_ports_at_depth_4_without_helpers`
and the matrix scenario `phase4_recur_d4_stateful_parent_composed_mixed_support_child_input`
per construction strategy isolate the surface across three intermediate
parent layers below the top. Smoke confirmed 85 internal module
occurrences with 471 hierarchy-wide vs 3 top-only
stateful-parent-composed-mixed-support bindings and 5438 vs 64
parent-local flops at depth 4.

The slice does not change the generator: it tightens the gate around an
already-supported capability.

Current-code validation includes the focused recursive pipeline
regression, `cargo test --bin tool_matrix`, and the full r67 Phase 4
hierarchy gate through Verilator plus both repo-owned Yosys modes.

### Phase 4 r66 extended the depth-4 axis with recursive non-top stateful parent-port-composed parent outputs without helpers downstream-clean
The previous full downstream-clean Phase 4 hierarchy evidence anchor is
`/tmp/anvil-tool-matrix-phase4-hierarchy-r66/tool_matrix_report.json`. It
keeps the live hierarchy policy at four designs per scenario and expands
it to 147 scenarios / 588 designs, with `coverage_gaps = []`,
`artifact_kind = "design"`, Verilator `588/0`, Yosys without-ABC
`588/0`, and Yosys with-ABC `588/0`.

This bank extends the depth-4 axis (r63 parent-flops, r64 mixed-support
child inputs, r65 no-state parent-port-composed outputs) to the
stateful parent-port-composed parent-output surface, mirroring how r61
followed r60 at depth 3. The new
`saw_recursive_hierarchy_depth_4_stateful_parent_port_composed_outputs`
fact (coverage gap when missing) plus the focused proof
`recursive_hierarchy_stateful_parent_outputs_mix_parent_ports_at_depth_4_without_helpers`
and the matrix scenario `phase4_recur_d4_stateful_parent_port_composed_output`
per construction strategy isolate the surface across three intermediate
parent layers below the top. Smoke confirmed 15 internal module
occurrences with 128 hierarchy-wide vs 32 top-only
parent-port-composed-through-flops outputs and 960 vs 64 parent-local
flops at depth 4.

The slice does not change the generator: it tightens the gate around an
already-supported capability.

Current-code validation includes the focused recursive pipeline
regression, `cargo test --bin tool_matrix`, and the full r66 Phase 4
hierarchy gate through Verilator plus both repo-owned Yosys modes.

### Phase 4 r65 extended the depth-4 axis with recursive non-top parent-port-composed parent outputs without helpers or state downstream-clean
The previous full downstream-clean Phase 4 hierarchy evidence anchor is
`/tmp/anvil-tool-matrix-phase4-hierarchy-r65/tool_matrix_report.json`. It
keeps the live hierarchy policy at four designs per scenario and expands
it to 144 scenarios / 576 designs, with `coverage_gaps = []`,
`artifact_kind = "design"`, Verilator `576/0`, Yosys without-ABC
`576/0`, and Yosys with-ABC `576/0`.

This bank extends the depth-4 axis (r63 parent-flops, r64 mixed-support
child inputs) to the parent-port-composed parent-output surface,
mirroring how r60 followed r59 at depth 3. The new
`saw_recursive_hierarchy_depth_4_parent_port_composed_outputs` fact
(coverage gap when missing) plus the focused proof
`recursive_hierarchy_parent_outputs_mix_parent_ports_at_depth_4_without_helpers`
and the matrix scenario `phase4_recur_d4_parent_port_composed_output`
per construction strategy isolate the surface across three intermediate
parent layers below the top. Smoke confirmed 15 internal module
occurrences with 176 hierarchy-wide vs 44 top-only parent-port-composed
outputs at depth 4.

The slice does not change the generator: it tightens the gate around an
already-supported capability.

Current-code validation includes the focused recursive pipeline
regression, `cargo test --bin tool_matrix`, and the full r65 Phase 4
hierarchy gate through Verilator plus both repo-owned Yosys modes.

### Phase 4 r64 extended the depth-4 axis with recursive non-top mixed-support child inputs without helpers downstream-clean
The previous full downstream-clean Phase 4 hierarchy evidence anchor is
`/tmp/anvil-tool-matrix-phase4-hierarchy-r64/tool_matrix_report.json`. It
keeps the live hierarchy policy at four designs per scenario and expands
it to 141 scenarios / 564 designs, with `coverage_gaps = []`,
`artifact_kind = "design"`, Verilator `564/0`, Yosys without-ABC
`564/0`, and Yosys with-ABC `564/0`.

This bank extends the depth-4 axis (opened by r63) to the unregistered
parent-composed mixed-support child-input surface, mirroring how r59
followed r58 at depth 3. The new
`saw_recursive_hierarchy_depth_4_mixed_support_child_inputs` fact
(coverage gap when missing) plus the focused proof
`recursive_hierarchy_parent_composed_routes_mix_parent_ports_at_depth_4_without_helpers`
and the matrix scenario `phase4_recur_d4_parent_composed_mixed_support_child_input`
per construction strategy isolate the surface across three intermediate
parent layers below the top. Smoke confirmed 85 internal module
occurrences with 315 hierarchy-wide vs 3 top-only mixed-support
bindings and 355 vs 3 parent-composed bindings at depth 4.

The slice does not change the generator: it tightens the gate around an
already-supported capability.

Current-code validation includes the focused recursive pipeline
regression, `cargo test --bin tool_matrix`, and the full r64 Phase 4
hierarchy gate through Verilator plus both repo-owned Yosys modes.

### Phase 4 r63 opened the depth-4 axis with recursive non-top parent-local flops downstream-clean
The previous full downstream-clean Phase 4 hierarchy evidence anchor is
`/tmp/anvil-tool-matrix-phase4-hierarchy-r63/tool_matrix_report.json`. It
keeps the live hierarchy policy at four designs per scenario and expands
it to 138 scenarios / 552 designs, with `coverage_gaps = []`,
`artifact_kind = "design"`, Verilator `552/0`, Yosys without-ABC
`552/0`, and Yosys with-ABC `552/0`.

This bank opens the depth-4 axis. The depth-3 push was structurally
complete in r62 (all four mixed-support cells covered at depth 3:
parent-flops, no-state child-input, no-state parent-output, stateful
parent-output, stateful child-input). r63 starts the depth-4 axis with
the simplest surface — parent flops at depth 4 — by adding
`saw_recursive_hierarchy_depth_4_parent_local_flops` (coverage gap when
missing) plus the focused proof
`recursive_hierarchy_parents_can_emit_local_flops_at_depth_4` and the
matrix scenario `phase4_recur_d4_parent_state` per construction strategy
(`2,2` child-instance bounds, three intermediate parent layers below
the top). The smoke run at depth 4 confirmed 15 internal module
occurrences with 960 hierarchy-wide parent-local flops versus 64
top-only, so the recursive generator handles depth-4 nesting cleanly.

The slice does not change the generator: it tightens the gate around an
already-supported capability. No new metric is needed.

Current-code validation includes the focused recursive pipeline
regression, `cargo test --bin tool_matrix`, and the full r63 Phase 4
hierarchy gate through Verilator plus both repo-owned Yosys modes.

### Phase 4 r62 closed the depth-3 push by gating recursive non-top stateful parent-composed mixed-support child inputs without helpers downstream-clean
The previous full downstream-clean Phase 4 hierarchy evidence anchor is
`/tmp/anvil-tool-matrix-phase4-hierarchy-r62/tool_matrix_report.json`. It
keeps the live hierarchy policy at four designs per scenario and expands
it to 135 scenarios / 540 designs, with `coverage_gaps = []`,
`artifact_kind = "design"`, Verilator `540/0`, Yosys without-ABC
`540/0`, and Yosys with-ABC `540/0`.

This bank closes the final symmetric cell of the depth-3 push. The
sweep has now covered parent-flops (r58), no-state mixed-support child
inputs (r59), no-state parent-port-composed outputs (r60), stateful
parent-port-composed outputs (r61), and now stateful unregistered
parent-composed mixed-support child inputs (r62). r62 adds
`saw_recursive_hierarchy_depth_3_stateful_parent_composed_mixed_support_child_inputs`
(coverage gap when missing) plus the focused proof
`recursive_hierarchy_stateful_parent_composed_routes_mix_parent_ports_at_depth_3_without_helpers`
and the matrix scenario `phase4_recur_d3_stateful_parent_composed_mixed_support_child_input`
per construction strategy. The smoke run at depth 3 confirmed 21
internal module occurrences with 129 hierarchy-wide
stateful-parent-composed-mixed-support bindings versus 3 top-only and
1344 vs 64 parent-local flops, so the recursive generator handles
depth-3 stateful child-input mixed-support cleanly.

The slice does not change the generator: it tightens the gate around an
already-supported capability. No new metric is needed because the
`child_input_bindings_from_stateful_parent_composed_mixed_support`
counter added in r56 already populates correctly at depth 3.

Current-code validation includes the focused recursive pipeline
regression, `cargo test --bin tool_matrix`, and the full r62 Phase 4
hierarchy gate through Verilator plus both repo-owned Yosys modes.

### Phase 4 r61 pushed recursive non-top stateful parent-port-composed parent outputs to exact hierarchy depth 3 without helpers downstream-clean
The previous full downstream-clean Phase 4 hierarchy evidence anchor is
`/tmp/anvil-tool-matrix-phase4-hierarchy-r61/tool_matrix_report.json`. It
keeps the live hierarchy policy at four designs per scenario and expands
it to 132 scenarios / 528 designs, with `coverage_gaps = []`,
`artifact_kind = "design"`, Verilator `528/0`, Yosys without-ABC
`528/0`, and Yosys with-ABC `528/0`.

This bank closes the last symmetric gap in the depth-3 push. r58/r59/r60
covered parent-flops, mixed-support child inputs, and no-state
parent-port-composed outputs at depth 3. r61 adds the stateful version
of the parent-output surface (r55's depth-2 territory) at depth 3 by
adding `saw_recursive_hierarchy_depth_3_stateful_parent_port_composed_outputs`
(coverage gap when missing) plus the focused proof
`recursive_hierarchy_stateful_parent_outputs_mix_parent_ports_at_depth_3_without_helpers`
and the matrix scenario `phase4_recur_d3_stateful_parent_port_composed_output`
per construction strategy. The smoke run at depth 3 confirmed 7 internal
module occurrences with 36 hierarchy-wide parent-port-composed outputs
through parent-local Qs versus 12 top-only and 448 vs 64 parent-local
flops, so the recursive generator handles depth-3 stateful parent-output
composition cleanly.

The slice does not change the generator: it tightens the gate around an
already-supported capability. No new metric is needed because the
through-parent-flop output counters added in r55 already populate
correctly at depth 3.

Current-code validation includes the focused recursive pipeline
regression, `cargo test --bin tool_matrix`, and the full r61 Phase 4
hierarchy gate through Verilator plus both repo-owned Yosys modes.

### Phase 4 r60 pushed recursive non-top parent-port-composed parent outputs to exact hierarchy depth 3 without helpers or state downstream-clean
The previous full downstream-clean Phase 4 hierarchy evidence anchor is
`/tmp/anvil-tool-matrix-phase4-hierarchy-r60/tool_matrix_report.json`. It
keeps the live hierarchy policy at four designs per scenario and expands
it to 129 scenarios / 516 designs, with `coverage_gaps = []`,
`artifact_kind = "design"`, Verilator `516/0`, Yosys without-ABC
`516/0`, and Yosys with-ABC `516/0`.

This bank closes the remaining symmetry gap in the depth-3 push.
r58 took parent-flops to depth 3 and r59 took unregistered
parent-composed mixed-support child inputs to depth 3, but the
parent-output cone surface (r54's depth-2 territory) had no
exact-depth-3 focused proof. r60 closes that gap by adding
`saw_recursive_hierarchy_depth_3_parent_port_composed_outputs`
(coverage gap when missing) plus the focused proof
`recursive_hierarchy_parent_outputs_mix_parent_ports_at_depth_3_without_helpers`
and the matrix scenario `phase4_recur_d3_parent_port_composed_output`
per construction strategy. The scenario uses `2,2` child-instance bounds
(matching r58's depth-3 parent-state shape but with parent flops off and
the parent-output cone surface as the only active route). The smoke run
at depth 3 confirmed 7 internal module occurrences with 72 hierarchy-wide
parent-port-composed outputs versus 24 top-only, so the recursive
generator handles depth-3 parent-output composition cleanly.

The slice does not change the generator: it tightens the gate around an
already-supported capability. No new metric is needed because
`hierarchy_parent_composed_outputs`, `top_parent_composed_outputs`,
`hierarchy_parent_port_composed_outputs`, `top_parent_port_composed_outputs`,
and `realized_max_leaf_depth` are already populated.

Current-code validation includes the focused recursive pipeline
regression, `cargo test --bin tool_matrix`, and the full r60 Phase 4
hierarchy gate through Verilator plus both repo-owned Yosys modes.

### Phase 4 r59 pushed recursive non-top mixed-support child inputs to exact hierarchy depth 3 without helpers downstream-clean
The previous full downstream-clean Phase 4 hierarchy evidence anchor is
`/tmp/anvil-tool-matrix-phase4-hierarchy-r59/tool_matrix_report.json`. It
keeps the live hierarchy policy at four designs per scenario and expands
it to 126 scenarios / 504 designs, with `coverage_gaps = []`,
`artifact_kind = "design"`, Verilator `504/0`, Yosys without-ABC
`504/0`, and Yosys with-ABC `504/0`.

This bank pushes the unregistered parent-composed mixed-support
child-input surface from exact depth 2 (r53) to exact depth 3. r58
already pushed the parent-flop surface to depth 3 but left the
mixed-support child-input surface depth-bound at 2. r59 closes that
asymmetry by adding `saw_recursive_hierarchy_depth_3_mixed_support_child_inputs`
(coverage gap when missing) plus the focused proof
`recursive_hierarchy_parent_composed_routes_mix_parent_ports_at_depth_3_without_helpers`
and the matrix scenario `phase4_recur_d3_parent_composed_mixed_support_child_input`
per construction strategy. The scenario uses `4,4` child-instance bounds
(distinct from r58's depth-3 / `2,2` parent-state shape) to broaden the
depth-3 evidence across different design shapes. The smoke run at
depth 3 confirmed 21 internal module occurrences with 115 hierarchy-wide
mixed-support bindings versus 3 top-only, so the recursive generator
handles depth-3 mixed-support routing cleanly.

The slice does not change the generator: it tightens the gate around an
already-supported capability. No new metric is needed because
`child_input_bindings_from_parent_composed_logic`,
`child_input_bindings_from_mixed_support`, the corresponding top
counters, and `realized_max_leaf_depth` are already populated.

Current-code validation includes the focused recursive pipeline
regression, `cargo test --bin tool_matrix`, and the full r59 Phase 4
hierarchy gate through Verilator plus both repo-owned Yosys modes.

### Phase 4 r58 pushed recursive parent-local flops to exact hierarchy depth 3 downstream-clean
The previous full downstream-clean Phase 4 hierarchy evidence anchor is
`/tmp/anvil-tool-matrix-phase4-hierarchy-r58/tool_matrix_report.json`. It
keeps the live hierarchy policy at four designs per scenario and expands
it to 123 scenarios / 492 designs, with `coverage_gaps = []`,
`artifact_kind = "design"`, Verilator `492/0`, Yosys without-ABC
`492/0`, and Yosys with-ABC `492/0`.

This bank pushes the parent-state surface from exact depth 2 to exact
depth 3. All r51-r57 focused proofs use depth 2 (one layer of internal
parents below the top). The mixed-range `2:3` scenario already produces
depth-3 designs sometimes, but no focused proof asserts the parent-state
surface fires AT depth 3 specifically. r58 closes that asymmetry by
adding `saw_recursive_hierarchy_depth_3_parent_local_flops` (coverage
gap when missing) plus the focused proof
`recursive_hierarchy_parents_can_emit_local_flops_at_depth_3` and the
matrix scenario `phase4_recur_d3_parent_state` per construction strategy
(2,2 child-instance bounds, distinct from r57's depth-2 / 4,4 shape).
The smoke run at depth 3 confirmed 7 internal module occurrences and
448 parent-local flops with `top_local_flops = 64`, so the recursive
generator handles depth-3 nesting cleanly.

The slice does not change the generator: it tightens the gate around an
already-supported capability. No new metric is needed because
`realized_max_leaf_depth`, `hierarchy_parent_local_flops`,
`top_local_flops`, and `internal_module_occurrences_with_local_flops`
are already populated.

Current-code validation includes the focused recursive pipeline
regression, `cargo test --bin tool_matrix`, and the full r58 Phase 4
hierarchy gate through Verilator plus both repo-owned Yosys modes.

### Phase 4 r57 gated recursive non-top parent-local flops as first-class coverage downstream-clean
The previous full downstream-clean Phase 4 hierarchy evidence anchor is
`/tmp/anvil-tool-matrix-phase4-hierarchy-r57/tool_matrix_report.json`. It
keeps the live hierarchy policy at four designs per scenario and expands
it to 120 scenarios / 480 designs, with `coverage_gaps = []`,
`artifact_kind = "design"`, Verilator `480/0`, Yosys without-ABC
`480/0`, and Yosys with-ABC `480/0`.

This bank promotes recursive non-top parent-local flops to first-class
gated coverage. r55 and r56 already evidenced non-top parent-local flops
as a side-channel of their richer mixed-support assertions, but the gate
did not enforce the parent-flop surface below the top parent on its own.
A regression that broke parent-flop emission specifically for non-top
parents could therefore have slipped past the existing matrix. r57
closes that gap by adding `saw_recursive_hierarchy_parent_local_flops`
(coverage gap when missing) plus a dedicated focused proof
`recursive_hierarchy_parents_can_emit_local_flops_below_top` that
isolates the parent-flop surface by disabling helpers, sibling routing,
registered routing, and parent-composed child-input cones. The new
matrix scenario `phase4_recur_d2_parent_state` uses `4,4` child-instance
bounds (distinct from r55's `2,2`) so the parent-state surface has its
own labeled focus point in the matrix rather than relying on
side-channel evidence from richer scenarios.

The slice does not change the generator: it tightens the gate around an
already-supported capability. No new metric is needed because
`hierarchy_parent_local_flops`, `top_local_flops`,
`internal_module_occurrences_with_local_flops`, and
`realized_max_leaf_depth` are already populated.

Current-code validation includes the focused recursive pipeline
regression, `cargo test --bin tool_matrix`, and the full r57 Phase 4
hierarchy gate through Verilator plus both repo-owned Yosys modes.

### Phase 4 r56 proved recursive stateful no-helper parent-composed mixed-support child inputs downstream-clean
The previous full downstream-clean Phase 4 hierarchy evidence anchor is
`/tmp/anvil-tool-matrix-phase4-hierarchy-r56/tool_matrix_report.json`. It
keeps the live hierarchy policy at four designs per scenario and expands
it to 117 scenarios / 468 designs, with `coverage_gaps = []`,
`artifact_kind = "design"`, Verilator `468/0`, Yosys without-ABC
`468/0`, and Yosys with-ABC `468/0`.

This bank adds the child-input sibling of the r55 parent-output proof. r53
proved recursive non-top unregistered parent-composed mixed-support child
inputs in a stateless setup; r56 keeps the same no-helper, no-registered
shape but turns on parent-local flops and requires the new
`child_input_bindings_from_stateful_parent_composed_mixed_support` counter
to exceed top-only below the top parent. That proves a non-top parent's
unregistered parent-composed child-input cone can simultaneously source
parent ports, child outputs, and parent-local Qs without using helper
instances or registered routing.

The new metric is computed at the existing parent-composed child-input
binding site by intersecting the binding's dep set across `has_ports`,
`has_instance_outputs`, and `has_flop_virtuals`. No new IR construct
appears in the generator path: the slice exposes a stricter cell of the
existing parent-composed mixed-support surface.

Current-code validation includes the focused recursive pipeline
regression, `cargo test --bin tool_matrix`, and the full r56 Phase 4
hierarchy gate through Verilator plus both repo-owned Yosys modes.

### Phase 4 r55 proved recursive stateful no-helper parent-port-composed outputs downstream-clean
The previous full downstream-clean Phase 4 hierarchy evidence anchor is
`/tmp/anvil-tool-matrix-phase4-hierarchy-r55/tool_matrix_report.json`. It
keeps the live hierarchy policy at four designs per scenario and expands
it to 114 scenarios / 456 designs, with `coverage_gaps = []`,
`artifact_kind = "design"`, Verilator `456/0`, Yosys without-ABC
`456/0`, and Yosys with-ABC `456/0`.

This bank adds the stateful sibling of the r54 parent-output proof. The
focused exact-depth-2 lane disables helper instances, direct sibling
routing, registered sibling routing, and child-input parent-cone routes,
then enables parent-local flops and requires hierarchy-wide
parent-port-composed parent-output counters through parent-local Qs to
exceed their top-only counterparts. That proves recursive non-top parent
outputs can mix parent data ports, child outputs, and parent-local Qs
without using helper instances.

Current-code validation includes the focused recursive pipeline
regression, `cargo test --bin tool_matrix`, and the full r55 Phase 4
hierarchy gate through Verilator plus both repo-owned Yosys modes.

### Phase 4 r54 proved recursive no-helper parent-port-composed outputs downstream-clean
The previous full downstream-clean Phase 4 hierarchy evidence anchor is
`/tmp/anvil-tool-matrix-phase4-hierarchy-r54/tool_matrix_report.json`. It
keeps the live hierarchy policy at four designs per scenario and expands
it to 111 scenarios / 444 designs, with `coverage_gaps = []`,
`artifact_kind = "design"`, Verilator `444/0`, Yosys without-ABC
`444/0`, and Yosys with-ABC `444/0`.

This bank adds a focused exact-depth-2 recursive parent-output proof below
the top parent. The focused lane disables helper instances, parent-local
flops, direct sibling routing, registered sibling routing, and child-input
parent-cone routes, then requires hierarchy-wide parent-port-composed
parent-output counters to exceed their top-only counterparts. That makes
recursive non-top parent outputs that mix parent data ports with child
outputs a first-class coverage fact instead of inferring the case from the
older top-parent parent-output evidence.

Current-code validation includes the focused recursive pipeline
regression, `cargo test --bin tool_matrix`, and the full r54 Phase 4
hierarchy gate through Verilator plus both repo-owned Yosys modes.

### Phase 4 r53 proved recursive no-helper parent-composed mixed support downstream-clean
The previous full downstream-clean Phase 4 hierarchy evidence anchor is
`/tmp/anvil-tool-matrix-phase4-hierarchy-r53/tool_matrix_report.json`. It
kept the live hierarchy policy at four designs per scenario and expanded
it to 108 scenarios / 432 designs, with `coverage_gaps = []`,
`artifact_kind = "design"`, Verilator `432/0`, Yosys without-ABC
`432/0`, and Yosys with-ABC `432/0`.

This bank adds an ordinary unregistered parent-composed child-input
mixed-support proof below the top parent. No-helper child-input cones now
promote their root when needed so the same parent-composed binding can
carry both parent data-port support and sibling child-output support.
The focused lane disables direct sibling routes, registered child-input
routes, helper instances, and parent-local flops, so the proof stays in
the unregistered parent-composed bucket instead of being classified as a
helper-backed or registered route.

Current-code validation includes the focused recursive pipeline
regression, `cargo test --bin tool_matrix`, and the full r53 Phase 4
hierarchy gate through Verilator plus both repo-owned Yosys modes.

### Phase 4 r52 proves recursive direct registered sibling mixed support downstream-clean
The latest full downstream-clean Phase 4 hierarchy evidence anchor is now
`/tmp/anvil-tool-matrix-phase4-hierarchy-r52/tool_matrix_report.json`. It
keeps the live hierarchy policy at four designs per scenario and expands
it to 105 scenarios / 420 designs, with `coverage_gaps = []`,
`artifact_kind = "design"`, Verilator `420/0`, Yosys without-ABC
`420/0`, and Yosys with-ABC `420/0`.

This bank does not need a new generator path. The r51 direct registered
sibling mixed-support route is generated by the same parent-generation
logic below the top parent, so r52 adds a focused exact-depth-2 recursive
scenario and a stricter coverage fact that requires hierarchy-wide
registered sibling mixed-support counters to exceed the top-only
counters. The focused lane disables registered parent-composed and
helper-instance sources, so the recursive proof stays classified as
non-top direct registered sibling routing rather than registered
parent-composed or helper-backed D-cone routing.

Current-code validation includes the focused recursive pipeline
regression, `cargo test --bin tool_matrix`, and the full r52 Phase 4
hierarchy gate through Verilator plus both repo-owned Yosys modes.

### Phase 4 r51 adds direct registered sibling mixed support downstream-clean
The previous direct registered sibling mixed-support Phase 4 hierarchy evidence anchor is
`/tmp/anvil-tool-matrix-phase4-hierarchy-r51/tool_matrix_report.json`. It
keeps the live hierarchy policy at four designs per scenario and expands
it to 102 scenarios / 408 designs, with `coverage_gaps = []`,
`artifact_kind = "design"`, Verilator `408/0`, Yosys without-ABC
`408/0`, and Yosys with-ABC `408/0`.

This bank adds the default-off
`hierarchy_registered_sibling_mixed_support_prob` route. When a direct
registered sibling D source has instance-output support but lacks parent
ports, the route may mix in one compatible parent data-port companion
before the parent-local flop. The mixed D expression is wrapped before
registration so the binding still proves direct registered sibling
routing and does not satisfy the registered parent-composed classifier.

The new metric is intentionally narrow:
`binding_uses_registered_sibling_mixed_support` requires a final
child-input binding sourced by a `FlopQ`, port support in that flop's D
cone, virtual instance-output support in the same D cone, and no
registered parent-composed D-cone classification. The focused pipeline
regression disables parent-composed routes and proves positive direct
registered sibling mixed-support while keeping registered
parent-composed and registered mixed-support parent-composed counters at
zero.

Current-code validation includes the focused metrics regression, the
focused pipeline regression, `cargo test --bin tool_matrix`, and the
full r51 Phase 4 hierarchy gate through Verilator plus both repo-owned
Yosys modes.

### Phase 4 r50 banks accumulated mixed-support hierarchy coverage downstream-clean
The previous accumulated mixed-support Phase 4 hierarchy evidence anchor is
`/tmp/anvil-tool-matrix-phase4-hierarchy-r50/tool_matrix_report.json`. It
kept the live policy at 99 scenarios / 396 designs with four designs per
scenario and recorded `coverage_gaps = []`, `artifact_kind = "design"`,
Verilator `396/0`, Yosys without-ABC `396/0`, and Yosys with-ABC
`396/0`.

This bank promotes the three current mixed-support coverage-only slices
into the full downstream-clean surface: stateful helper-backed parent
outputs with parent-port support, unregistered parent-composed helper
child-input mixed support, and stateful helper-through-parent-flop
unregistered child-input mixed support. The prior coverage-only report
trees remain useful focused breadcrumbs, and `r50` remains the previous
full downstream-clean evidence for those policy facts before `r51` carried
them forward.

### Phase 4 stateful parent-composed helper child-input mixed support
The hierarchy gate now distinguishes the stateful parent-composed helper
child-input route from the stricter overlap where the same unregistered
final child-input binding both consumes a helper-sourced parent-local Q
and also carries parent data-port support. This is separate from the
plain helper-through-parent-flop child-input counter and from the plain
unregistered helper mixed-support counter.

The metric intentionally requires both halves on the same binding:
`binding_uses_parent_cone_instance_flop_mixed_support` first reuses the
helper-through-parent-flop classifier, then requires parent-port support
on the final child-input binding's dependency set. Because the
helper-through-parent-flop classifier rejects final `FlopQ` registered
bindings, the new metric stays focused on unregistered parent-composed
child-input logic that reads helper-sourced parent state.

The Phase 4 coverage facts are narrow. The nonrecursive fact requires
child-input cone routing, parent-cone helper instances, parent-local
flops, no direct sibling or registered helper routes in the focused
lane, positive
`child_input_bindings_from_parent_cone_instance_flop_mixed_support`, and
zero registered helper counters. The recursive fact additionally
requires the hierarchy-wide stateful helper and mixed-support counters
to exceed their top-only counterparts.

Current-code validation includes the focused metrics regression,
`cargo test --bin tool_matrix`, and a coverage-only 99-scenario /
396-design Phase 4 dry run at
`/tmp/anvil-tool-matrix-phase4-stateful-helper-child-input-mixed-check`
with `coverage_gaps = []`,
`saw_hierarchy_parent_composed_parent_cone_instance_flop_mixed_support_routing = true`,
and
`saw_recursive_hierarchy_parent_composed_parent_cone_instance_flop_mixed_support_routing = true`.
The previous full downstream-clean `r50` bank carried these facts through
Verilator and both repo-owned Yosys modes; `r51` carries them forward, and
the coverage-only dry run remains a focused breadcrumb.
### Phase 4 unregistered parent-composed helper child-input mixed support
The hierarchy gate now distinguishes parent-composed child-input
bindings that merely reach parent-cone helper outputs from the stricter
overlap where the same unregistered binding also carries parent data-port
support. The generator now repairs required helper-backed child-input
cones by adding a parent-port companion when the helper route would
otherwise lack ports.

The metric is intentionally separate from the registered helper
mixed-support route: `binding_uses_parent_cone_instance_mixed_support`
rejects final `FlopQ` child-input bindings and requires the final
binding to be parent-composed logic. The focused regression reuses the
budgeted helper case because `max_parent_cone_instances_per_module = 3`
already forces helper-backed child-input bindings without needing a new
scenario.

The Phase 4 coverage facts are also narrow. The nonrecursive fact
requires unregistered child-input cones, helper instances, no
parent-flop route, positive
`child_input_bindings_from_parent_cone_instance_mixed_support`, and zero
registered helper child-input bindings. The recursive fact additionally
requires the non-top hierarchy counters to exceed the top counters while
helper-through-flop and registered-helper counters remain zero.

Current-code validation includes the focused metrics regression,
`cargo test --bin tool_matrix`, and a coverage-only 99-scenario /
396-design Phase 4 dry run at
`/tmp/anvil-tool-matrix-phase4-parent-helper-child-input-mixed-check`
with `coverage_gaps = []`,
`saw_hierarchy_parent_cone_instance_mixed_support_routing = true`, and
`saw_recursive_hierarchy_parent_cone_instance_mixed_support_routing =
true`. The full downstream-clean `r50` bank now carries these facts
through Verilator and both repo-owned Yosys modes; the coverage-only dry
run remains a focused breadcrumb.

### Phase 4 stateful parent-output helper mixed-support metrics
The hierarchy gate now distinguishes parent outputs that reach
parent-cone helper instance outputs through parent-local flops from the
stricter overlap where that same output cone also carries parent-port
support. The implementation adds hierarchy/top counters and fractions in
`DesignMetrics`, plus nonrecursive and recursive coverage facts in
`src/bin/tool_matrix.rs`.

The recursive fact stays intentionally narrow: it requires the
hierarchy-wide mixed-through-flop counter to exceed the top-only counter
while child-input helper and registered-helper binding counters stay
zero, so the proof remains a parent-output route instead of drifting
into child-input helper evidence. The Phase 4 required-knob list also
now includes the plain `hierarchy_sibling_route_prob` attempt, closing
the last missing decision-site requirement for the direct sibling route
axis.

Validation included the focused metrics regression,
`cargo test --bin tool_matrix`, a coverage-only 99-scenario / 396-design
Phase 4 dry run at
`/tmp/anvil-tool-matrix-phase4-mixed-helper-check`,
`cargo check --all-targets`, and the full `cargo test` suite with 302 passing
tests. The full downstream-clean `r50` bank now carries these facts
through Verilator and both repo-owned Yosys modes; the coverage-only dry
run remains a focused breadcrumb.

### Phase 4 r49 banks recursive parent-output helper mixed-support downstream-clean
The live Phase 4 hierarchy policy now requires recursive non-top parent
outputs to prove the same output cone can carry both parent-port support
and parent-cone helper output support. The focused regression is
`cargo test recursive_hierarchy_parent_outputs_mix_helper_instances_with_parent_ports_below_top`.

This needed a dedicated output mixed-support metric instead of inferring
the fact from `hierarchy_parent_port_composed_outputs` and
`hierarchy_outputs_reaching_parent_cone_instances`. Those counters can
both be true in a design while describing different parent outputs. The
new `*_outputs_reaching_parent_cone_instance_mixed_support` counters
make the overlap explicit.

The full downstream-clean evidence anchor for that slice was
`/tmp/anvil-tool-matrix-phase4-hierarchy-r49/tool_matrix_report.json`:
`99` scenarios, `4` designs/scenario, `396` total designs,
`coverage_gaps = []`,
`saw_recursive_hierarchy_parent_cone_instance_mixed_support_outputs = true`,
`saw_recursive_hierarchy_parent_cone_instance_outputs = true`,
`saw_hierarchy_parent_port_composed_outputs = true`, and
`saw_recursive_hierarchy_registered_parent_cone_instance_mixed_support_routing = true`
with `396/0` pass-fail in Verilator plus both repo-owned Yosys modes.
The previous recursive non-top registered parent-composed helper
mixed-support full bank is `r48`; `r50` is the previous accumulated
mixed-support hierarchy bank, and `r51` is the current full downstream-clean
Phase 4 hierarchy bank.

### Phase 4 r48 banks recursive registered helper mixed-support routing downstream-clean
The live Phase 4 hierarchy policy now requires the recursive
registered parent-composed helper route to carry parent-port support in
the same D cone below the top parent. In an exact-depth-2 recursive
hierarchy, a parent-cone helper instance can feed registered
parent-composed child-input logic, that logic can also consume parent
data ports, and the resulting parent-local Q can bind a later child
input. The focused regression is
`cargo test recursive_hierarchy_registered_helper_routes_mix_parent_ports_below_top`.

This needed a dedicated helper-mixed metric instead of inferring the
fact from the older registered helper and registered mixed-support
counters. Those counters can both be true in a design without proving
that parent-port support and the parent-cone helper output occur in the
same registered D cone. The new
`registered_parent_cone_instance_mixed_support_*` counters make that
overlap explicit.

The full downstream-clean evidence anchor for that slice was
`/tmp/anvil-tool-matrix-phase4-hierarchy-r48/tool_matrix_report.json`:
`99` scenarios, `4` designs/scenario, `396` total designs,
`coverage_gaps = []`,
`saw_recursive_hierarchy_registered_parent_cone_instance_mixed_support_routing = true`,
`saw_recursive_hierarchy_registered_parent_composed_parent_cone_instance_routing = true`,
`saw_recursive_hierarchy_registered_multistage_mixed_support_routing = true`,
`saw_recursive_hierarchy_registered_multistage_sibling_routing = true`,
`saw_recursive_hierarchy_registered_multistage_routing = true`,
`saw_recursive_hierarchy_registered_mixed_support_routing = true`,
`saw_hierarchy_registered_multistage_routing = true`,
`saw_hierarchy_registered_multistage_sibling_routing = true`,
`saw_recursive_hierarchy_registered_multistage_parent_cone_instance_routing = true`,
`saw_recursive_hierarchy_registered_multistage_parent_composed_parent_cone_instance_routing = true`,
and
`saw_recursive_hierarchy_registered_parent_composed_parent_cone_instance_routing = true`
with `396/0` pass-fail in Verilator plus both repo-owned Yosys modes.
The previous recursive non-top registered multistage mixed-support
no-helper full bank is `r47`.

### Phase 4 r47 banks recursive registered multistage mixed-support routing downstream-clean
The live Phase 4 hierarchy policy now requires the recursive no-helper
overlap between registered mixed support and multi-stage registered
parent-composed routing. Below the top parent, an exact-depth-2
recursive hierarchy can build a registered D cone that simultaneously
uses parent data ports, child instance outputs, and an earlier
parent-local Q, then bind a later child input through the resulting
parent-local state without relying on parent-cone helper instances. The
focused regression is
`cargo test recursive_hierarchy_registered_multistage_mixed_support_routes_below_top`.

This needed a dedicated metric instead of inferring the fact from the
existing mixed-support and multistage counters. Those older counters can
be true in the same design while describing different bindings; the new
`registered_multistage_mixed_support_*` counters only fire when one
registered route contains both kinds of support in the same D cone and
then participates in later Q reuse.

The `r47` full downstream-clean evidence anchor is
`/tmp/anvil-tool-matrix-phase4-hierarchy-r47/tool_matrix_report.json`:
`99` scenarios, `4` designs/scenario, `396` total designs,
`coverage_gaps = []`,
`saw_recursive_hierarchy_registered_multistage_mixed_support_routing = true`,
`saw_recursive_hierarchy_registered_multistage_sibling_routing = true`,
`saw_recursive_hierarchy_registered_multistage_routing = true`,
`saw_recursive_hierarchy_registered_mixed_support_routing = true`,
`saw_hierarchy_registered_multistage_routing = true`,
`saw_hierarchy_registered_multistage_sibling_routing = true`,
`saw_recursive_hierarchy_registered_multistage_parent_cone_instance_routing = true`,
`saw_recursive_hierarchy_registered_multistage_parent_composed_parent_cone_instance_routing = true`,
and
`saw_recursive_hierarchy_registered_parent_composed_parent_cone_instance_routing = true`
with `396/0` pass-fail in Verilator plus both repo-owned Yosys modes.
The previous recursive non-top registered sibling multistage no-helper
full bank is `r46`.

### Cargo default-run is part of the README contract
The repository has two binaries: the generator (`anvil`) and the
auxiliary `tool_matrix` harness. Cargo cannot infer which one plain
`cargo run -- ...` should execute unless `Cargo.toml` keeps
`default-run = "anvil"` in the `[package]` section.

This is a user-facing contract, not cosmetic metadata: README and the
mdBook intentionally teach `cargo run -- ...` for generator examples,
while `tool_matrix` is always selected explicitly with
`cargo run --bin tool_matrix -- ...`. Future auxiliary binaries must
preserve that default-run setting or update every source-tree command
surface in the live docs at the same time.

### Phase 4 r46 banks recursive registered sibling multistage routing downstream-clean
The live Phase 4 hierarchy policy now requires the recursive no-helper
multi-stage registered sibling-routed child-input cross product. Below
the top parent, an exact-depth-2 recursive hierarchy can bind one child
input from an earlier child output through parent-local state, then
reuse that earlier parent-local Q as the D source for a later direct
registered sibling route, without relying on parent-composed D logic or
parent-cone helper instances. The focused regression is
`cargo test recursive_hierarchy_registered_sibling_routes_can_chain_without_helpers_below_top`.
It uses four child instances per recursive parent so the sibling-output
route has enough earlier sources to force both the first registered
binding and the later Q-reuse binding across every construction
strategy.

The full downstream-clean evidence anchor for that slice was
`/tmp/anvil-tool-matrix-phase4-hierarchy-r46/tool_matrix_report.json`:
`99` scenarios, `4` designs/scenario, `396` total designs,
`coverage_gaps = []`,
`saw_recursive_hierarchy_registered_multistage_sibling_routing = true`,
`saw_recursive_hierarchy_registered_multistage_routing = true`,
`saw_recursive_hierarchy_registered_mixed_support_routing = true`,
`saw_hierarchy_registered_multistage_routing = true`,
`saw_hierarchy_registered_multistage_sibling_routing = true`,
`saw_recursive_hierarchy_registered_multistage_parent_cone_instance_routing = true`,
`saw_recursive_hierarchy_registered_multistage_parent_composed_parent_cone_instance_routing = true`,
and
`saw_recursive_hierarchy_registered_parent_composed_parent_cone_instance_routing = true`
with `396/0` pass-fail in Verilator plus both repo-owned Yosys modes.
The previous recursive non-top registered parent-composed multistage
no-helper full bank is `r45`.

### Phase 4 r45 banks recursive registered multistage routing downstream-clean
The live Phase 4 hierarchy policy now requires the recursive no-helper
multi-stage registered parent-composed cross product. Below the top
parent, an exact-depth-2 recursive hierarchy can first bind a child input
through parent-local state, then reuse that earlier parent-local Q in a
later registered parent-composed child-input D cone, without relying on
parent-cone helper instances. The focused regression is
`cargo test recursive_hierarchy_registered_parent_composed_routes_can_chain_without_helpers_below_top`.
It uses four child instances per recursive parent because the two-child
registered mixed-support calibration is too sparse to force this
multi-stage subcase across every construction strategy.

The full downstream-clean evidence anchor for that slice was
`/tmp/anvil-tool-matrix-phase4-hierarchy-r45/tool_matrix_report.json`:
`96` scenarios, `4` designs/scenario, `384` total designs,
`coverage_gaps = []`,
`saw_recursive_hierarchy_registered_multistage_routing = true`,
`saw_recursive_hierarchy_registered_mixed_support_routing = true`,
`saw_hierarchy_registered_multistage_routing = true`,
`saw_recursive_hierarchy_registered_multistage_parent_cone_instance_routing = true`,
`saw_recursive_hierarchy_registered_multistage_parent_composed_parent_cone_instance_routing = true`,
and
`saw_recursive_hierarchy_registered_parent_composed_parent_cone_instance_routing = true`
with `384/0` pass-fail in Verilator plus both repo-owned Yosys modes.
The previous recursive non-top registered mixed-support full bank is
`r44`.

### Phase 4 r44 banks recursive registered mixed-support routing downstream-clean
The live Phase 4 hierarchy policy now requires the recursive no-helper
registered mixed-support cross product. Below the top parent, an
exact-depth-2 recursive hierarchy can build registered parent-composed
child-input D logic from both parent data ports and child outputs, then
drive later child inputs through parent-local state without relying on
parent-cone helper instances. The focused regression is
`cargo test recursive_hierarchy_registered_mixed_support_routes_below_top`.
It requires the recursive tree shape, non-top parent-local flops,
non-top registered parent-composed child-input bindings, non-top
registered child-output support, non-top registered mixed-support
bindings, and zero registered helper-sourced D-cone bindings.

The previous full downstream-clean evidence anchor is
`/tmp/anvil-tool-matrix-phase4-hierarchy-r44/tool_matrix_report.json`:
`93` scenarios, `4` designs/scenario, `372` total designs,
`coverage_gaps = []`,
`saw_recursive_hierarchy_registered_mixed_support_routing = true`,
`saw_hierarchy_registered_mixed_support_routing = true`,
`saw_recursive_multiple_parent_cone_instances_per_parent_child_inputs = true`,
`saw_recursive_multiple_parent_cone_instances_per_parent_through_flops = true`,
`saw_recursive_multiple_parent_cone_instances_per_parent = true`,
`saw_recursive_hierarchy_registered_multistage_parent_cone_instance_routing = true`,
`saw_recursive_hierarchy_registered_multistage_parent_composed_parent_cone_instance_routing = true`,
and
`saw_recursive_hierarchy_registered_parent_composed_parent_cone_instance_routing = true`
with `372/0` pass-fail in Verilator plus both repo-owned Yosys modes.
The previous recursive non-top child-input multi-helper budget full bank
is `r43`.

### Phase 4 r43 banks recursive non-top child-input helper budgets downstream-clean
The live Phase 4 hierarchy policy now closes the child-input local-budget
cross product for recursive parent-cone helpers. Below the top parent,
an exact-depth-2 recursive hierarchy can spend a multi-helper
`max_parent_cone_instances_per_module = 3` budget while driving
parent-composed child-input bindings directly from helper outputs. The
focused regression is
`cargo test recursive_hierarchy_parent_cone_helper_budget_allows_multiple_helpers_below_top`.
It requires the recursive tree shape, the configured helper budget in
`max_parent_cone_instances_per_internal_module`, helper instances beyond
the top parent, non-top parent-composed child-input bindings, non-top
child-input bindings sourced from helper outputs, and zero
helper-through-flop or registered helper child-input bindings.

The current full downstream-clean evidence anchor is
`/tmp/anvil-tool-matrix-phase4-hierarchy-r43/tool_matrix_report.json`:
`90` scenarios, `4` designs/scenario, `360` total designs,
`coverage_gaps = []`,
`saw_recursive_multiple_parent_cone_instances_per_parent_child_inputs = true`,
`saw_recursive_multiple_parent_cone_instances_per_parent_through_flops = true`,
`saw_recursive_multiple_parent_cone_instances_per_parent = true`,
`saw_multiple_parent_cone_instances_per_parent = true`,
`saw_recursive_hierarchy_parent_cone_instance_flop_outputs = true`,
`saw_recursive_hierarchy_parent_cone_instance_outputs = true`,
`saw_recursive_hierarchy_direct_sibling_parent_cone_instance_routing = true`,
`saw_recursive_hierarchy_direct_registered_sibling_parent_cone_instance_routing = true`,
`saw_recursive_hierarchy_registered_multistage_parent_cone_instance_routing = true`,
`saw_recursive_hierarchy_registered_multistage_parent_composed_parent_cone_instance_routing = true`,
`saw_recursive_hierarchy_registered_parent_composed_parent_cone_instance_routing = true`,
and
`saw_recursive_hierarchy_parent_composed_parent_cone_instance_flop_routing = true`
with `360/0` pass-fail in Verilator plus both repo-owned Yosys modes.
The previous recursive non-top stateful multi-helper budget full bank is
`r42`.

### Phase 4 r42 banks recursive non-top stateful helper budgets downstream-clean
Phase 4 r42 closed the stateful local-budget
cross product for recursive parent-output helpers. Below the top parent,
an exact-depth-2 recursive hierarchy can spend a multi-helper
`max_parent_cone_instances_per_module = 3` budget, register the helper
outputs into parent-local flops, and drive parent outputs from those
helper-sourced Qs. The focused regression is
`cargo test recursive_hierarchy_parent_outputs_can_spend_stateful_helper_budget_below_top`.
It requires the recursive tree shape, the configured helper budget in
`max_parent_cone_instances_per_internal_module`, helper instances beyond
the top parent, parent-local flops below the top parent, parent outputs
that depend on helper outputs through those flops, and zero child-input
helper bindings through either direct, stateful, or registered helper
routes.

The full downstream-clean evidence anchor for that slice was
`/tmp/anvil-tool-matrix-phase4-hierarchy-r42/tool_matrix_report.json`:
`87` scenarios, `4` designs/scenario, `348` total designs,
`coverage_gaps = []`,
`saw_recursive_multiple_parent_cone_instances_per_parent_through_flops = true`,
`saw_recursive_multiple_parent_cone_instances_per_parent = true`,
`saw_multiple_parent_cone_instances_per_parent = true`,
`saw_recursive_hierarchy_parent_cone_instance_flop_outputs = true`,
`saw_recursive_hierarchy_parent_cone_instance_outputs = true`,
`saw_recursive_hierarchy_direct_sibling_parent_cone_instance_routing = true`,
`saw_recursive_hierarchy_direct_registered_sibling_parent_cone_instance_routing = true`,
`saw_recursive_hierarchy_registered_multistage_parent_cone_instance_routing = true`,
`saw_recursive_hierarchy_registered_multistage_parent_composed_parent_cone_instance_routing = true`,
`saw_recursive_hierarchy_registered_parent_composed_parent_cone_instance_routing = true`,
and
`saw_recursive_hierarchy_parent_composed_parent_cone_instance_flop_routing = true`
with `348/0` pass-fail in Verilator plus both repo-owned Yosys modes.
The previous recursive non-top multi-helper budget full bank is `r41`.

### Phase 4 r41 banks recursive non-top helper budgets downstream-clean
The live Phase 4 hierarchy policy now names the local-budget half of the
recursive parent-output helper surface. Below the top parent, an
exact-depth-2 recursive hierarchy can spend a multi-helper
`max_parent_cone_instances_per_module = 3` budget for parent-output
composition, not just accumulate one helper per parent across multiple
parents. The focused regression is
`cargo test recursive_hierarchy_parent_outputs_can_spend_helper_budget_below_top`.
It requires the recursive tree shape, the configured helper budget in
`max_parent_cone_instances_per_internal_module`, helper instances beyond
the top parent, parent outputs that depend on those helper outputs, no
child-input helper bindings, and no registered child-input helper D
cones.

The full downstream-clean evidence anchor for that slice was
`/tmp/anvil-tool-matrix-phase4-hierarchy-r41/tool_matrix_report.json`:
`87` scenarios, `4` designs/scenario, `348` total designs,
`coverage_gaps = []`,
`saw_recursive_multiple_parent_cone_instances_per_parent = true`,
`saw_multiple_parent_cone_instances_per_parent = true`,
`saw_recursive_hierarchy_parent_cone_instance_flop_outputs = true`,
`saw_recursive_hierarchy_parent_cone_instance_outputs = true`,
`saw_recursive_hierarchy_direct_sibling_parent_cone_instance_routing = true`,
`saw_recursive_hierarchy_direct_registered_sibling_parent_cone_instance_routing = true`,
`saw_recursive_hierarchy_registered_multistage_parent_cone_instance_routing = true`,
`saw_recursive_hierarchy_registered_multistage_parent_composed_parent_cone_instance_routing = true`,
`saw_recursive_hierarchy_registered_parent_composed_parent_cone_instance_routing = true`,
and
`saw_recursive_hierarchy_parent_composed_parent_cone_instance_flop_routing = true`
with `348/0` pass-fail in Verilator plus both repo-owned Yosys modes.
The previous recursive non-top stateful parent-output helper full bank
is `r40`.

### Phase 4 r40 banks recursive non-top stateful parent-output helper routing downstream-clean
The live Phase 4 hierarchy policy now includes the stateful
parent-output version of the recursive exact-depth-2 helper axis: below
the top parent, a non-top parent can instantiate helper children as
internal parent-cone sources, register those helper outputs into
parent-local flops, and drive parent outputs from the helper-sourced
state. The focused regression is
`cargo test recursive_hierarchy_parent_outputs_can_route_helper_instances_through_parent_flops_below_top`.
It requires the recursive tree shape, more helper instances below top
than at top, more parent-local flops below top than at top, more
helper-through-flop parent-output support across the hierarchy than at
top, no child-input helper bindings, and no registered child-input
helper D cones so the route stays distinct from both child-input helper
routing and direct recursive parent-output helper routing.

The full downstream-clean evidence anchor for that slice was
`/tmp/anvil-tool-matrix-phase4-hierarchy-r40/tool_matrix_report.json`:
`87` scenarios, `4` designs/scenario, `348` total designs,
`coverage_gaps = []`,
`saw_recursive_hierarchy_parent_cone_instance_flop_outputs = true`,
`saw_recursive_hierarchy_parent_cone_instance_outputs = true`,
`saw_recursive_hierarchy_direct_sibling_parent_cone_instance_routing = true`,
`saw_recursive_hierarchy_direct_registered_sibling_parent_cone_instance_routing = true`,
`saw_recursive_hierarchy_registered_multistage_parent_cone_instance_routing = true`,
`saw_recursive_hierarchy_registered_multistage_parent_composed_parent_cone_instance_routing = true`,
`saw_recursive_hierarchy_registered_parent_composed_parent_cone_instance_routing = true`,
and
`saw_recursive_hierarchy_parent_composed_parent_cone_instance_flop_routing = true`
with `348/0` pass-fail in Verilator plus both repo-owned Yosys modes.
The previous recursive non-top parent-output helper full bank is `r39`.

### Phase 4 r39 banks recursive non-top parent-output helper routing downstream-clean
The live Phase 4 hierarchy policy now includes the parent-output version
of the recursive exact-depth-2 helper axis: below the top parent, a
non-top parent can instantiate helper children as internal parent-cone
sources and drive its own parent outputs from those helper outputs. The
focused regression is
`cargo test recursive_hierarchy_parent_outputs_can_depend_on_helper_instances_below_top`.
It requires the recursive tree shape, more helper instances below top
than at top, more parent outputs reaching helper instances across the
hierarchy than at top, no child-input helper bindings, and no
helper-through-parent-flop output counts so the route stays distinct
from child-input helper routing and stateful parent-output helper
routing.

The full downstream-clean evidence anchor for that slice was
`/tmp/anvil-tool-matrix-phase4-hierarchy-r39/tool_matrix_report.json`:
`84` scenarios, `4` designs/scenario, `336` total designs,
`coverage_gaps = []`,
`saw_recursive_hierarchy_parent_cone_instance_outputs = true`,
`saw_recursive_hierarchy_direct_sibling_parent_cone_instance_routing = true`,
`saw_recursive_hierarchy_direct_registered_sibling_parent_cone_instance_routing = true`,
`saw_recursive_hierarchy_registered_multistage_parent_cone_instance_routing = true`,
`saw_recursive_hierarchy_registered_multistage_parent_composed_parent_cone_instance_routing = true`,
`saw_recursive_hierarchy_registered_parent_composed_parent_cone_instance_routing = true`,
and
`saw_recursive_hierarchy_parent_composed_parent_cone_instance_flop_routing = true`
with `336/0` pass-fail in Verilator plus both repo-owned Yosys modes.
The previous recursive multi-stage registered parent-composed helper
full bank is `r38`.

### Phase 4 r38 banks recursive non-top multi-stage registered parent-composed helper routing downstream-clean
The live Phase 4 hierarchy policy now includes the multi-stage
registered parent-composed version of the recursive exact-depth-2 helper
axis: below the top parent, a parent-cone helper output can seed a
parent-local Q, and later registered parent-composed D logic can reuse
that helper-sourced Q before driving a later child input. The focused
regression is
`cargo test recursive_hierarchy_registered_parent_composed_routes_can_chain_helper_instances_below_top`.
It requires the recursive tree shape, more helper instances below top
than at top, more multi-stage registered parent-composed bindings below
top than at top, more multi-stage helper-sourced parent-composed
bindings below top than at top, local parent flops below top, and zero
direct multi-stage registered helper counters so the route stays
distinct from the direct registered sibling helper-chain axis.

The full downstream-clean evidence anchor for that slice was
`/tmp/anvil-tool-matrix-phase4-hierarchy-r38/tool_matrix_report.json`:
`81` scenarios, `4` designs/scenario, `324` total designs,
`coverage_gaps = []`,
`saw_recursive_hierarchy_direct_sibling_parent_cone_instance_routing = true`,
`saw_recursive_hierarchy_direct_registered_sibling_parent_cone_instance_routing = true`,
`saw_recursive_hierarchy_registered_multistage_parent_cone_instance_routing = true`,
`saw_recursive_hierarchy_registered_multistage_parent_composed_parent_cone_instance_routing = true`,
`saw_recursive_hierarchy_registered_parent_composed_parent_cone_instance_routing = true`,
and
`saw_recursive_hierarchy_parent_composed_parent_cone_instance_flop_routing = true`
with `324/0` pass-fail in Verilator plus both repo-owned Yosys modes.
The previous recursive multi-stage direct registered-helper full bank is
`r37`.

### Phase 4 r37 banks recursive non-top multi-stage direct registered helper routing downstream-clean
The live Phase 4 hierarchy policy now includes the multi-stage version
of the recursive exact-depth-2 direct registered helper axis: below the
top parent, a direct registered sibling route can seed a parent-local Q
from a parent-cone helper instance, and a later direct registered sibling
route can reuse that helper-sourced Q as the next parent-flop D source.
The focused regression is
`cargo test recursive_hierarchy_registered_sibling_routes_can_chain_helper_instances_below_top`.
It requires the recursive tree shape, more helper instances below top
than at top, more multi-stage registered sibling bindings below top than
at top, more multi-stage helper-sourced registered sibling bindings below
top than at top, local parent flops below top, and zero registered
parent-composed counters so the route stays distinct from the
parent-composed helper-chain axis.

The full downstream-clean evidence anchor for that slice was
`/tmp/anvil-tool-matrix-phase4-hierarchy-r37/tool_matrix_report.json`:
`78` scenarios, `4` designs/scenario, `312` total designs,
`coverage_gaps = []`,
`saw_recursive_hierarchy_direct_sibling_parent_cone_instance_routing = true`,
`saw_recursive_hierarchy_direct_registered_sibling_parent_cone_instance_routing = true`,
`saw_recursive_hierarchy_registered_multistage_parent_cone_instance_routing = true`,
`saw_recursive_hierarchy_registered_parent_composed_parent_cone_instance_routing = true`,
and
`saw_recursive_hierarchy_parent_composed_parent_cone_instance_flop_routing = true`
with `312/0` pass-fail in Verilator plus both repo-owned Yosys modes.
The previous recursive registered parent-composed helper full bank is
`r36`.

### Phase 4 r36 banks recursive non-top registered parent-composed helper routing downstream-clean
The live Phase 4 hierarchy policy now includes the registered
parent-composed version of the recursive exact-depth-2 helper axis:
non-top registered parent-composed child-input D cones can source from
parent-cone helper instances below the top parent. The focused
regression is
`cargo test recursive_hierarchy_registered_child_input_cones_can_use_helper_instances_below_top`.
It requires the recursive tree shape, more helper instances below top
than at top, more registered parent-composed bindings below top than at
top, more registered helper bindings below top than at top, and local
parent flops below top.

The coverage-only policy anchor is
`/tmp/anvil-tool-matrix-phase4-recursive-registered-parent-helper-r36/tool_matrix_report.json`:
`75` scenarios, `4` designs/scenario, `300` total designs,
`coverage_gaps = []`, and
`saw_recursive_hierarchy_registered_parent_composed_parent_cone_instance_routing = true`.

The full downstream-clean evidence anchor for that slice was
`/tmp/anvil-tool-matrix-phase4-hierarchy-r36/tool_matrix_report.json`:
`75` scenarios, `4` designs/scenario, `300` total designs,
`coverage_gaps = []`,
`saw_recursive_hierarchy_direct_sibling_parent_cone_instance_routing = true`,
`saw_recursive_hierarchy_direct_registered_sibling_parent_cone_instance_routing = true`,
`saw_recursive_hierarchy_registered_parent_composed_parent_cone_instance_routing = true`,
and
`saw_recursive_hierarchy_parent_composed_parent_cone_instance_flop_routing = true`
with `300/0` pass-fail in Verilator plus both repo-owned Yosys modes.
The previous recursive direct registered-helper full bank is `r35`.

### Phase 4 r35 banks recursive non-top direct registered helper routing downstream-clean
The live Phase 4 hierarchy policy now includes the registered sibling
version of the recursive exact-depth-2 helper axis: non-top direct
registered sibling-routed child-input D paths can source from parent-cone
helper instances below the top parent. The focused regression is
`cargo test recursive_hierarchy_registered_sibling_routes_can_use_helper_instances_below_top`.
It requires the recursive tree shape, more helper instances below top
than at top, more registered sibling helper bindings below top than at
top, local parent flops below top, and zero registered parent-composed
D-cone counters so the route stays distinct from registered
parent-composed helper routing.

The coverage-only policy anchor is
`/tmp/anvil-tool-matrix-phase4-recursive-direct-registered-helper-r35/tool_matrix_report.json`:
`72` scenarios, `4` designs/scenario, `288` total designs,
`coverage_gaps = []`, and
`saw_recursive_hierarchy_direct_registered_sibling_parent_cone_instance_routing = true`.

The current full downstream-clean evidence anchor is
`/tmp/anvil-tool-matrix-phase4-hierarchy-r35/tool_matrix_report.json`:
`72` scenarios, `4` designs/scenario, `288` total designs,
`coverage_gaps = []`,
`saw_recursive_hierarchy_direct_sibling_parent_cone_instance_routing = true`,
`saw_recursive_hierarchy_direct_registered_sibling_parent_cone_instance_routing = true`,
and
`saw_recursive_hierarchy_parent_composed_parent_cone_instance_flop_routing = true`
with `288/0` pass-fail in Verilator plus both repo-owned Yosys modes.
The previous recursive direct-helper full bank is `r34`.

### Phase 4 r34 banks recursive non-top direct helper routing downstream-clean
The live Phase 4 hierarchy policy now includes a recursive exact-depth-2
axis that proves direct sibling-routed child inputs below the top parent
can source from parent-cone helper instances. The focused regression is
`cargo test recursive_hierarchy_sibling_routes_can_use_helper_instances_below_top`.
It requires the recursive tree shape, more helper instances and helper
bindings below top than at top, and zero registered helper counters so the
route stays distinct from registered sibling/helper D routing.

The first coverage-only policy anchor was
`/tmp/anvil-tool-matrix-phase4-recursive-direct-helper-r32/tool_matrix_report.json`;
the first full downstream attempt at
`/tmp/anvil-tool-matrix-phase4-hierarchy-r32/tool_matrix_report.json`
correctly failed because Yosys found one warning in both modes. The
repro was `int_nodeid_egraph_phase4_recur_profile_d2_top4_mid2_seq`,
`design_0002`, top `mod_50_0019`: a procedural `case` with an exact
selector chose an arm whose bounds made a later shift provably constant.
That exposed a real cleanup gap rather than a hierarchy bug: the cheap
bounds revisit handled shifts and ternary muxes, but not exact-selector
`CaseMux` / `CasezMux` arms. `src/gen/cone.rs` now teaches
`node_unsigned_bounds` and `exact_gate_value` to follow those procedural
mux arms conservatively, with regressions for the CaseMux overshift shape
and exact matching Casez patterns.

The current full downstream-clean evidence anchor is
`/tmp/anvil-tool-matrix-phase4-hierarchy-r34/tool_matrix_report.json`:
`69` scenarios, `4` designs/scenario, `276` total designs,
`coverage_gaps = []`,
`saw_recursive_hierarchy_direct_sibling_parent_cone_instance_routing = true`,
and
`saw_recursive_hierarchy_parent_composed_parent_cone_instance_flop_routing = true`
with `276/0` pass-fail in Verilator plus both repo-owned Yosys modes.
The previous recursive helper-state full bank is `r31` for the
66-scenario policy. The first clean direct-helper full bank was `r33`;
`r34` refreshes it after the post-remap idempotent duplicate cleanup.

### Phase 4 r31 banks recursive non-top helper state downstream-clean
The live Phase 4 hierarchy policy now includes a recursive exact-depth-2
axis that proves stateful parent-composed helper child-input routing
below the top parent. The focused regression is
`cargo test recursive_hierarchy_parent_composed_helper_routes_can_use_parent_flops_below_top`.
It is deliberately stronger than the depth-1 stateful helper proof:
it requires `hierarchy_parent_cone_instances > top_parent_cone_instances`,
`hierarchy_parent_local_flops > top_local_flops`, and
`child_input_bindings_from_parent_cone_instances_through_parent_flops >
top_child_input_bindings_from_parent_cone_instances_through_parent_flops`.

The first coverage-only policy anchor was
`/tmp/anvil-tool-matrix-phase4-recursive-helper-state-r31/tool_matrix_report.json`;
the current full downstream-clean evidence anchor is
`/tmp/anvil-tool-matrix-phase4-hierarchy-r31/tool_matrix_report.json`:
`66` scenarios, `4` designs/scenario, `264` total designs,
`coverage_gaps = []`, and
`saw_recursive_hierarchy_parent_composed_parent_cone_instance_flop_routing = true`
with `264/0` pass-fail in Verilator plus both repo-owned Yosys modes.
The previous full downstream-clean bank is `r30` for the 63-scenario
stateful parent-composed helper policy.

### Phase 4 r30 superseded the r29 registered parent-composed helper bank
Stateful parent-composed helper child-input routing now has its own
proof, distinct from registered child-input helper D cones. In the new
shape, a parent-cone helper output seeds a parent-local Q, and
unregistered parent-composed child-input logic consumes that helper Q
before binding the later child input. The focused proof should assert
`child_input_bindings_from_parent_cone_instances_through_parent_flops > 0`
and
`parent_cone_instance_flop_child_input_binding_fraction > 0.0` while
keeping `child_input_bindings_from_registered_parent_cone_instances = 0`.

The current evidence anchor is
`/tmp/anvil-tool-matrix-phase4-hierarchy-r30/tool_matrix_report.json`:
`63` scenarios, `4` designs/scenario, `252` total designs,
`coverage_gaps = []`, and `252/0` pass-fail in Verilator plus both
repo-owned Yosys modes. It fully banks the direct sibling helper,
direct registered sibling helper, multi-stage registered sibling,
stateful parent-output helper, multi-stage direct registered sibling
helper, multi-stage registered parent-composed helper, and stateful
parent-composed helper child-input routes. Keep `r23` as the
pre-direct-helper full-bank breadcrumb, `r24` as the coverage-only
direct-helper proof, `r25` as the direct-helper full bank, `r26` as the
previous multi-stage sibling full bank, `r27` as the previous stateful
parent-output helper bank, `r28` as the previous multi-stage direct
registered sibling helper bank, and `r29` as the previous
multi-stage registered parent-composed helper bank.

### Phase 4 r29 supersedes the r28 direct registered helper-chain bank
Registered parent-composed helper routing now has its own multi-stage
proof, distinct from the direct registered sibling helper proof. In the
new shape, a parent-cone helper output seeds an earlier parent-local Q,
and later registered parent-composed D logic reuses that Q before
driving a later child input. The focused proof should keep
`child_input_bindings_from_registered_multistage_parent_cone_instances = 0`
while asserting
`child_input_bindings_from_registered_multistage_parent_composed_parent_cone_instances > 0`,
so the direct sibling helper chain and parent-composed helper chain
stay observably separate.

The current evidence anchor is
`/tmp/anvil-tool-matrix-phase4-hierarchy-r29/tool_matrix_report.json`:
`60` scenarios, `4` designs/scenario, `240` total designs,
`coverage_gaps = []`, and `240/0` pass-fail in Verilator plus both
repo-owned Yosys modes. It fully banks the direct sibling helper,
direct registered sibling helper, multi-stage registered sibling,
stateful parent-output helper, multi-stage direct registered sibling
helper, and multi-stage registered parent-composed helper routes. Keep
`r23` as the pre-direct-helper full-bank breadcrumb, `r24` as the
coverage-only direct-helper proof, `r25` as the direct-helper full bank,
`r26` as the previous multi-stage sibling full bank, `r27` as the
previous stateful parent-output helper bank, and `r28` as the previous
multi-stage direct registered sibling helper bank.

### Phase 4 r28 superseded the r27 stateful parent-output helper bank
Direct registered sibling helper routing now has two distinct
child-input-proven forms: a helper output can feed the immediate
parent-local D path, and a helper output can first seed a parent-local Q
that a later registered sibling route reuses as the next flop's D
source. The second form is not registered parent-composed logic: the
focused proof should keep both
`child_input_bindings_from_registered_parent_composed_logic = 0` and
`child_input_bindings_from_registered_multistage_parent_composed_logic = 0`
while asserting
`child_input_bindings_from_registered_multistage_parent_cone_instances > 0`.

The evidence anchor was
`/tmp/anvil-tool-matrix-phase4-hierarchy-r28/tool_matrix_report.json`:
`57` scenarios, `4` designs/scenario, `228` total designs,
`coverage_gaps = []`, and `228/0` pass-fail in Verilator plus both
repo-owned Yosys modes. It fully banks the direct sibling helper,
direct registered sibling helper, multi-stage registered sibling,
stateful parent-output helper, and multi-stage direct registered
sibling helper routes. Keep `r23` as the pre-direct-helper full-bank
breadcrumb, `r24` as the coverage-only direct-helper proof, `r25` as
the direct-helper full bank, `r26` as the previous multi-stage sibling
full bank, and `r27` as the previous stateful parent-output helper
bank.

### Phase 4 r27 superseded the r26 multi-stage sibling bank
Parent-output helper routing now has two distinct output-proven forms:
direct helper-to-parent-output composition, and helper-to-parent-output
composition through parent-local state. The second form is not the same
as registered child-input routing: no child input needs to bind from a
helper output, and the proof should keep
`child_input_bindings_from_parent_cone_instances = 0` while asserting
that parent outputs reach helper instances through flop Qs.

That evidence anchor was
`/tmp/anvil-tool-matrix-phase4-hierarchy-r27/tool_matrix_report.json`:
`54` scenarios, `4` designs/scenario, `216` total designs,
`coverage_gaps = []`, and `216/0` pass-fail in Verilator plus both
repo-owned Yosys modes. It fully banks the direct sibling helper, direct
registered sibling helper, multi-stage registered sibling, and stateful
parent-output helper routes. Keep `r23` as the pre-direct-helper
full-bank breadcrumb, `r24` as the coverage-only direct-helper proof,
`r25` as the direct-helper full bank, and `r26` as the previous
multi-stage sibling full bank.

### Parent-output helper-through-flop metrics should stay dependency-based
The tempting implementation of the stateful parent-output helper metric
is to recursively walk every output cone looking for `FlopQ` nodes, then
recursively walk every flop's D cone looking for parent-cone helper
instance outputs. That is correct on small examples but too expensive
on parent-state-heavy scenarios that have many local flops and no
parent-cone helpers, and it re-walks the same D cones repeatedly.

The metric now first checks whether the module even has
`InstanceRole::ParentCone` instances. When it does, it uses the output
root's `DepSet` to find flop virtuals and the existing
`collect_instance_output_support` memo to ask whether each flop D side
reaches a parent-cone helper output. This preserves the structural fact
while keeping the Phase 4 matrix from turning a coverage metric into a
generation-time hotspot.

### Phase 4 r26 supersedes the r25 direct-helper bank
Direct sibling helper routes originally landed after the `r23`
full downstream-clean Phase 4 hierarchy bank, so `r24` was deliberately
coverage-only evidence for the expanded 48-scenario policy. `r25`
banked that direct-helper policy through the downstream tools. `r26`
adds the multi-stage registered sibling route and is now the current
historical 51-scenario evidence anchor.

That evidence anchor is
`/tmp/anvil-tool-matrix-phase4-hierarchy-r26/tool_matrix_report.json`:
`51` scenarios, `4` designs/scenario, `204` total designs,
`coverage_gaps = []`, and `204/0` pass-fail in Verilator plus both
repo-owned Yosys modes. It fully banks the direct sibling helper and
direct registered sibling helper routes, plus the registered sibling
route that chains through earlier parent-local Qs. Keep `r23` only as
the pre-direct-helper full-bank breadcrumb, `r24` only as the
coverage-only direct-helper proof, and `r25` only as the previous
direct-helper full bank.

### Direct sibling helper routes must stay unregistered in the metrics
Direct sibling routing can now request a parent-cone helper instance
source when `hierarchy_parent_cone_instance_prob` fires. That route
still binds the later child input directly from a dep-bearing parent
source; it does not allocate a parent-local flop and it must not be
counted as registered routing just because the source is a helper
instance output.

The metric contract is therefore split deliberately:
`child_input_bindings_from_parent_cone_instances` and the matching
plain helper fractions prove that a helper output reached a child input,
while `child_input_bindings_from_registered_parent_cone_instances`
stays zero unless the final binding goes through a registered route.
The focused direct sibling helper regression forces the registered
sibling and registered parent-composed axes off to preserve that
separation.

### Registered helper metrics must include direct registered sibling routes
The registered helper-instance metric originally grew out of the
registered parent-composed child-input route, where the child binding is
proved by a parent-composed D cone followed by one local parent flop.
That shape is still important, but it is not the only registered route
that can legitimately use a parent-cone helper instance.

Direct registered sibling routing can now request a helper instance
source when `hierarchy_parent_cone_instance_prob` fires. In that case
the route remains a registered child-input binding, but the flop D side
may be the helper `InstanceOutput` itself or a width-adapter gate over
that output, not a registered parent-composed D-cone root.

Therefore the metric contract is dependency-based: inspect the final
registered flop D dependencies and ask whether they include a
parent-cone helper instance output. Requiring the D node to also look
like registered parent-composed logic would undercount the direct
registered sibling helper route and would make the test prove the wrong
shape.

### Parent-output helper budgeting must be output-proven, not helper-to-helper-proven
The parent-output helper route originally proved only one fact: a parent
output could depend on a parent-cone helper instance output. The
separate helper-budget route proved a different fact through
child-input bindings: a parent could allocate multiple helper children
when `max_parent_cone_instances_per_module` was raised. Those two facts
did not prove that parent-output composition itself could spend the
budget.

The current implementation therefore collects parent-output helper
sources before parent-output root construction and lets promotion select
a required helper source per output. That makes the budget visible at
the output-composition seam instead of relying on later child-input
routes.

One gotcha is important: helper instances created specifically for
parent-output composition should not bind their own child inputs from
earlier helper outputs. If that helper-to-helper chaining is allowed,
`child_input_bindings_from_parent_cone_instances` becomes non-zero and
the test no longer proves an output-only path. The parent-output helper
collector therefore uses non-helper parent sources for helper child
inputs while still publishing the helper outputs to the real parent
source pool for output composition.

### Phase 4 starts as wrapper hierarchy on purpose
The first hierarchy slice is deliberately **not** "instances can appear
anywhere in any parent cone". That broader story is the destination,
but it is not the cheapest truthful first landing.

What landed instead is:

- generate a library of leaf modules with the already-proven leaf
  kernel,
- choose the wrapper's instantiated-child count separately from the
  library size,
- keep `num_child_instances = 0` as the legacy compatibility mode
  meaning "instantiate every generated leaf definition exactly once",
- if fewer instances than library entries are requested, instantiate a
  shuffled subset without replacement,
- if more instances than library entries are requested, cover every
  library entry once and then fill the remaining slots by reuse with
  replacement,
- build a real top wrapper module,
- treat child instance outputs as real parent-side leaf variables for
  top-output construction, and
- make emission / validation / manifest handling design-aware.

That buys several real things immediately:

- ANVIL now emits genuine multi-module SV, not just disconnected leaf
  files;
- downstream tools now see elaboration and inter-module port binding;
- the IR and validator now carry explicit instance structure; and
- the hierarchy layer stays above `generate_leaf_module` instead of
  smearing inter-module behavior into the leaf kernel.

Just as importantly, it keeps the open work honest. The first-landed
top layer was **combinational only**; since then, bounded recursive
sub-hierarchy growth, local parent flops, child-input routing, mixed
parent-port / child-output parent outputs, parent-cone helper instances
for parent-composed child-input cones, direct sibling child-input
routes, direct registered sibling D sources, registered child-input D
cones, parent-output cones, and explicit helper budgeting have landed
as separate slices. Broader helper-instance placement
beyond those seams, broader registered hierarchy-local routing, and
hierarchical identity remain future work.

Two narrow implementation choices are load-bearing in this slice:

- the wrapper top marks shared `clk` / `rst_n` as
  `Module.clock` / `Module.reset`, and control-port visibility is now
  design-aware instead of leaf-local: pure comb-only modules omit those
  ports, while hierarchy parents keep them visible iff they carry local
  state or sequential descendants;
- `Node::InstanceOutput` now carries a real dep-bearing leaf identity,
  so parent cones can use child outputs without being mistaken for
  empty-dep constants by later cleanup/finalisation passes.

That second point shook loose three old wrapper-era assumptions that had
to be fixed at the root instead of papered over:

- `compact_node_ids` was only treating output drives and flop holders as
  liveness roots, so instance input bindings could survive with stale
  `NodeId`s after compaction. The real fix was to mark instance-input
  bindings as holders and remap them just like drives and flops.
- `validate_design` was still enforcing "every child output is exposed
  exactly once", which was only true for the pass-through wrapper era.
  The right rule is narrower: any *referenced* child output node must
  name a real child port at the right width, but unreferenced child
  outputs are legal.

### Depth ranges must stay ranges in recursive hierarchy
The original bounded recursive planner was honest enough as a first
landing, but it was still throwing away too much information: it took
`min_hierarchy_depth..=max_hierarchy_depth` and collapsed that interval
to one exact realized depth for the whole design.

That was acceptable as a foothold. It is not the right long-term
algorithm, because the point of a bounded depth interval is to describe
allowed leaf-depth variation, not to sample one global scalar and ignore
the rest.

The strengthened planner now carries a remaining `[min,max]` depth
interval per subtree:

- `max == 0` still means a mandatory leaf;
- if a flexible subtree has only one child, it may sample one exact
  depth inside the still-allowed interval for that chain; and
- when a subtree is both depth-flexible and branching (`instances >= 2`)
  it now deliberately generates child definitions that realize both the
  shallowest and deepest still-legal descendants, instead of hoping RNG
  stumbles into both.

That last point is the load-bearing part. Mixed-depth recursion should
not be a rare accident of repeated runs; when the structure can support
it, the planner should intentionally exercise it.

The metrics contract grew with the planner: `DesignMetrics` now expose
`leaf_module_occurrences_by_depth`, so "did we really get both shallow
and deep leaves?" is answerable numerically from the manifest rather
than by reading emitted SV.

The focused artifact at `/tmp/anvil-hier-mixed-depth-smoke-r1/manifest.json`
was the first clean proof of that new mixed-depth recursive axis. The
current repo-owned Phase 4 gate at
`/tmp/anvil-tool-matrix-phase4-hierarchy-r27/tool_matrix_report.json`
also proves it, so the mixed-depth story is no longer "focused-only"
evidence.
- the emitter was still assuming every child output had a corresponding
  `Node::InstanceOutput`. That is no longer true once the parent may use
  only a subset of child outputs, so unused outputs are now rendered as
  explicit unconnected instance ports (`.port()`).

The separation between **library size** and **instance count** matters
enough to say plainly here: they are different planning decisions and
should not be welded together. Repeated instantiation of the same child
definition stresses elaboration and sharing pressure in a different way
from simply generating more unique children. Under-instantiating the
library is also useful because it exercises real unused-module cleanup
in downstream tools. The wrapper slice still is not the final hierarchy
algorithm, but this split is a real step toward a budget-driven one.

Module names are also a hierarchy resource now, not an incidental string
format at each construction site. Leaf modules, recursive parent
modules, and later designs in the same generator run all reserve names
from the same `Generator` sequence. That keeps `--count N --out DIR`
safe for hierarchy output: one module definition still maps to one
`.sv` file, and no later design can overwrite an earlier definition by
reusing the same `mod_<seed>_<index>` name.

### Bounded recursive hierarchy keeps the old wrapper lane, then adds a real tree planner
The next honest Phase 4 step was **not** to quietly overload the old
depth-1 wrapper knobs until they accidentally meant recursion. That
would have blurred the already-banked wrapper evidence and made the
meaning of the config surface harder to recover later.

So the deliberate rule now is:

- keep the legacy exact wrapper lane alive:
  `hierarchy_depth = 1`, `num_leaf_modules`, `num_child_instances`;
- add a separate bounded recursive lane:
  `min_hierarchy_depth..=max_hierarchy_depth` and
  `min_child_instances_per_module..=max_child_instances_per_module`; and
- make the two planning surfaces mutually exclusive.

That gives us a clean story:

- old repo-owned wrapper closure artifacts stay truthful;
- new recursive hierarchy is explicit in configs and manifests; and
- future recursive work can evolve without pretending the exact wrapper
  lane already solved arbitrary tree planning.

The current recursive planner is now intentionally exact about the
interval contract rather than about one sampled scalar:

- every realized leaf depth stays inside the requested `[min:max]`
  interval;
- when a subtree is both depth-flexible and branching, the planner
  deliberately exercises both the shallowest and deepest still-legal
  descendants instead of relying on luck; and
- each non-leaf module still picks its child-instance count uniformly
  inside the requested child-instance interval.

So the current guarantees are:

- realized leaf depths stay inside the requested interval;
- mixed shallow/deep trees are now intentional when the structure can
  support them;
- realized branching always stays inside the requested interval; and
- the metrics can prove all of that numerically.

One more planning layer is now live on top of that baseline:

- `min_child_instances_per_module..=max_child_instances_per_module`
  remains the global fallback range for recursive branching; and
- repeated `child_instances_per_depth` overrides can tighten or replace
  that range at specific parent depths (`0` = top, `1` = its direct
  children, ...).

That keeps the control surface honest: users can ask for "top is wide,
lower levels are narrower" without inventing a separate planner mode or
forcing the manifest reader to reverse-engineer the realized tree by
hand.

What it does **not** do yet is make every parent-side cone free to
instantiate arbitrary helper modules. The narrow helper-instance seams
are now live for parent-composed child-input cones, direct sibling
routes, direct registered sibling D sources, registered child-input D
cones, parent-output cones, multi-stage direct registered sibling
helper chains, and multi-stage registered parent-composed helper
chains, with explicit per-parent budgeting. Broader helper placement
beyond those seams remains future hierarchy work.

One more gate-level rule turned out to matter here: when a repo-owned
matrix grows new representative scenarios, its per-scenario evidence
budget must not shrink by accident. The Phase 4 gate moved from 15 to
18 scenarios once the mixed-depth recursive axis was added, so its
minimum total design budget was raised from 48 to 60 to preserve the
old 4 designs/scenario sampling depth instead of silently falling to 3.

That lesson is now encoded directly for Phase 4. After the
parent-output helper, budgeted-helper, and registered helper-sourced
child-input axes raised the scenario set to 42, the old
`PHASE4_HIERARCHY_MIN_TOTAL_DESIGNS = 120` rule silently produced only
3 designs/scenario (`126` total) in the clean pre-fix `r22` run. The
live gate now uses a per-scenario floor
`PHASE4_HIERARCHY_MIN_DESIGNS_PER_SCENARIO = 4`. After the direct
sibling helper and direct registered sibling helper axes raised the
scenario set to `48`, the live regression expected `192` total designs.
After the multi-stage registered sibling route raised the scenario set
to `51`, the live regression expects `204` total designs. The
stateful parent-output helper route raised the scenario set to `54`,
and the live regression expected `216` total designs. The multi-stage
direct registered sibling helper route raised it to `57` scenarios /
`228` total designs in `r28`, and the multi-stage registered
parent-composed helper route raises it to `60` scenarios / `240` total
designs in `r29`.

One more planner rule is load-bearing here: in recursive range mode,
child libraries are generated **on demand per parent**, and every
generated direct child definition is instantiated at least once. That
keeps reuse live without manufacturing dead unreachable subtrees just to
inflate counts. The legacy exact wrapper lane remains the place where
top-level under-instantiation of a pre-generated library is exercised.

### Explicit child sourcing must be a real axis, not a vague future promise
Once the wrapper/reuse/under-instantiation story and the recursive
mixed-depth story were both banked, the next honest Phase 4 question
was no longer "can hierarchy exist?" but "how do parents obtain child
definitions?"

That decision is too load-bearing to hide behind ad hoc planner
behavior. It needs to be a user-visible, measurable axis:

- `library` means pre-generate a reusable child-definition pool and let
  instance slots pick from it;
- `on-demand` means synthesize child definitions against exact
  parent-planned data-interface profiles per planned instance slot.

The current landed `on-demand` slice is now the stronger honest one:
each planned child slot carries an exact parent-planned data-interface
profile, and the realized child definition is validated against that
exact emitted data-input/output shape. Control ports stay structural,
which is also the right rule: `clk` / `rst_n` are propagated by
sequential-state presence, not by the data-profile planner.

That is also why the metrics contract grew again. The hierarchy reports
now need to distinguish:

- reused child definitions,
- single-use instantiated definitions, and
- the average instance count per unique instantiated module,
- exact profiled instance-slot coverage, and
- whether child data-input bindings stay dep-bearing instead of
  collapsing to constants.

Without those numbers, `library` vs `on-demand` would still force a
human to open the emitted `.sv`, which is exactly the trust failure we
want to avoid.

The repo-owned Phase 4 gate has now caught up here too. The current
artifact is `/tmp/anvil-tool-matrix-phase4-hierarchy-r27/tool_matrix_report.json`,
and it explicitly proves both child-sourcing modes (`library` and
`on-demand`) together with structural proof that the on-demand
scenarios really emitted fresh child definitions per planned instance
slot and exact profiled child-interface synthesis.

### Combinational sibling routing was the right next layer before local parent state
Once parent-composed outputs and exact profiled child sourcing were
real, the next honest hierarchy question for that slice was not
"should parents have local flops yet?" It was "can one child feed
another through the parent without us faking it as a top-level wrapper
input?"

The current answer is now yes, but intentionally only on the simpler
surface:

- later child data inputs may bind from earlier sibling instance
  outputs;
- the routing stays acyclic by construction because only already-built
  sibling outputs are eligible;
- the routing stayed purely combinational in that slice; and
- local parent flops deliberately remained future work instead of being
  smuggled into the same step.

That last point matters. Child-output -> child-input through local flop
layers is a valid future hierarchy surface, but it is a different
question from the one we needed to close here. This slice was about
making the parent behave more like the leaf generator's cone builder,
except with child-module outputs as additional dep-bearing leaves, while
keeping the phase boundary honest.

The metrics contract had to grow again for the same reason. A sibling
routing feature that can only be confirmed by opening `.sv` is not a
trustworthy feature. The design reports now distinguish:

- child inputs bound from parent ports,
- child inputs bound from sibling instance outputs,
- mixed-support child inputs, and
- the hierarchy-wide and top-level fractions of child inputs that come
  from sibling instance outputs.

The focused proof artifact is now
`/tmp/anvil-hier-sibling-routing-smoke-r1/manifest.json`, and the
repo-owned Phase 4 gate at
`/tmp/anvil-tool-matrix-phase4-hierarchy-r27/tool_matrix_report.json`
now requires `saw_hierarchy_sibling_routing = true`.

### Parent-composed child-input bindings are the cone-builder analogue of sibling routing
The next hierarchy routing step keeps the same phase boundary but
removes one more artificial flat-wrapper shape. Direct sibling routing
answers "can a later child consume an earlier child output?" Parent-
composed child-input binding answers "can the parent build a small
combinational cone for a child input, using the same generator machinery
as leaf cones?"

For that slice, the rule was deliberately narrow and structural:

- `hierarchy_child_input_cone_prob` controls the probability of this
  route;
- the cone's source pool contains only already-available parent sources:
  parent data inputs, earlier sibling instance outputs, and earlier
  parent-side route gates;
- local parent flops stayed disabled, so the parent-composed
  child-input route was a purely combinational composition surface; and
- the rule applies to both the legacy wrapper lane and the bounded
  recursive lane.

This is the shape the user suggested: at the composition level, replace
"gate" by "child module" where it makes sense, but keep that first
slice combinational. Local parent flops are now landed under
`hierarchy_parent_flop_prob`; the first one-flop registered sibling
route is now landed under `hierarchy_registered_sibling_route_prob`,
and the first registered parent-composed child-input route is now
landed under `hierarchy_registered_child_input_cone_prob`. The first
multi-stage registered parent-composed subcase is also live now, while
broader registered hierarchy routing remains a later, separate
hierarchy surface.

The metrics contract grew again with
`child_input_bindings_from_parent_composed_logic`,
`parent_composed_child_input_binding_fraction`, and
`top_parent_composed_child_input_binding_fraction`. The repo-owned
Phase 4 gate treats this as a required coverage fact via
`saw_hierarchy_parent_composed_child_inputs`; the current banked gate at
`/tmp/anvil-tool-matrix-phase4-hierarchy-r27/tool_matrix_report.json`
proves it together with local parent state, `coverage_gaps = []`, and
216/0 clean pass-fail in Verilator plus both repo-owned Yosys modes.
The focused targeted proof is
`/tmp/anvil-hier-child-input-cone-smoke-r1/manifest.json`.

### Parent-cone helper instances make module instantiation a parent source choice
The first helper-instantiation slice is intentionally small: when
`hierarchy_parent_cone_instance_prob` fires during a
parent-composed child-input route, the parent may instantiate one helper
child as an internal parent-cone source. That helper is not one of the
planned child slots. It is tagged with `InstanceRole::ParentCone`, bound
from the parent source pool, and its outputs can feed later child inputs
through ordinary parent combinational logic.

That gives the hierarchy planner a first real "module instance as cone
source" behavior without making every parent-side cone recursive all at
once. The metrics contract is explicit:
`top_parent_cone_instances`, `hierarchy_parent_cone_instances`,
`child_input_bindings_from_parent_cone_instances`,
`top_child_input_bindings_from_parent_cone_instances`,
`parent_cone_instance_child_input_binding_fraction`, and
`top_parent_cone_instance_child_input_binding_fraction`.

The focused proof is
`/tmp/anvil-parent-cone-instance-smoke-r1/manifest.json`
(`top_parent_cone_instances = 1`, `hierarchy_parent_cone_instances = 1`,
`child_input_bindings_from_parent_cone_instances = 4`, and
`top_child_input_bindings_from_parent_cone_instances = 4`), clean in
Verilator, Yosys `synth -noabc`, and the repo-owned Yosys with-ABC
path. The repo-owned Phase 4 gate now banks this as a required coverage
fact at `/tmp/anvil-tool-matrix-phase4-hierarchy-r27/tool_matrix_report.json`
with `coverage_gaps = []` and 216/0 pass-fail in Verilator plus both
repo-owned Yosys modes.

Current HEAD has broadened that helper source beyond the original
parent-composed child-input seam. Helper outputs can now feed direct
unregistered sibling routes, direct registered sibling-route D inputs,
registered parent-composed child-input D cones, and parent-output
composition. The direct sibling helper proof is
`cargo test hierarchy_sibling_routes_can_use_helper_instances`; it
requires registered helper counters to stay zero while
`child_input_bindings_from_parent_cone_instances > 0`,
`parent_cone_instance_child_input_binding_fraction > 0.0`,
`top_parent_cone_instance_child_input_binding_fraction > 0.0`, and
helper instances are present beyond the planned child slots.

### Local parent flops are a separate hierarchy state axis
The next Phase 4 step deliberately does not overload leaf `flop_prob`.
Hierarchy parent state is controlled by its own knob,
`hierarchy_parent_flop_prob`, because the parent layer is a different
structural axis from leaf-module sequential richness. The default is
`0.0`, which preserves the previously banked combinational hierarchy
surface; setting it non-zero lets parent output cones and
parent-composed child-input cones emit local parent flops.

The important invariant is the same one used for sequential
descendants: `clk` and `rst_n` are structural, not decorative. A parent
module reserves those control ports when local parent state is possible,
but the emitter only exposes them when the module actually carries
local flops or sequential descendants. Pure comb-only modules remain
free of control ports.

The implementation reuses the normal cone/flop worklist machinery.
While building a hierarchy parent cone, the generator temporarily maps
flop rolls to `KnobId::HierarchyParentFlopProb`, then drains the parent
flop worklist before finalization. That keeps telemetry honest: leaf
flop attempts still count as `flop_prob`, while parent-state attempts
count as `hierarchy_parent_flop_prob`.

Metrics now expose this state surface directly:
`hierarchy_parent_local_flops`,
`internal_module_occurrences_with_local_flops`, `top_local_flops`,
`child_input_bindings_from_parent_flops`,
`parent_flop_child_input_binding_fraction`, and
`top_parent_flop_child_input_binding_fraction`.
The focused proof is
`/tmp/anvil-hier-parent-state-smoke-r1/manifest.json`
(`hierarchy_parent_local_flops = 8`, `top_local_flops = 8`,
`top_clock_inputs = 1`, `top_reset_inputs = 1`,
`child_input_bindings_from_parent_flops = 1`), clean in Verilator,
Yosys `synth -noabc`, and the repo-owned Yosys with-ABC path. The
repo-owned Phase 4 gate now also banks this as a required coverage fact
at `/tmp/anvil-tool-matrix-phase4-hierarchy-r27/tool_matrix_report.json`
with `coverage_gaps = []` and 216/0 pass-fail in Verilator plus both
repo-owned Yosys modes.

### Registered sibling routing is a distinct hierarchy route axis
Direct sibling routing and registered sibling routing are deliberately
separate knobs. `hierarchy_sibling_route_prob` keeps the acyclic
combinational route live: earlier child output directly feeds a later
child input. `hierarchy_registered_sibling_route_prob` adds a
parent-local flop between those endpoints. A later registered sibling
route can now also choose an earlier parent-local Q as its D source,
creating a multi-stage registered sibling chain without parent-composed
logic. That route is still
acyclic at the module-instance level, but it introduces real state in
the parent, so it must be measured as both child-input provenance and
parent-local state.

The initial implementation intentionally used one flop and no extra mux
or cone around it. That is not "good enough"; it is the smallest
signoff-clean primitive for this axis. Richer registered
child-to-child patterns build from the same invariant:
earlier child output -> parent state -> later child input, with metrics
proving the route instead of requiring SV inspection. The multi-stage
direct sibling subcase is now reported separately through
`child_input_bindings_from_registered_multistage_instance_outputs`,
`top_child_input_bindings_from_registered_multistage_instance_outputs`,
`registered_multistage_instance_output_child_input_binding_fraction`,
and
`top_registered_multistage_instance_output_child_input_binding_fraction`.

This slice exposed a real finalization gotcha: post-construction remap
passes already rewrote output drives and flop fields, but instance
input bindings were also live NodeId consumers. Once a child input
could bind to a parent-local Q node, flop merging could leave an
instance input pointing at a stale duplicate FlopQ. The fix belongs in
`ir::compact`: every partial NodeId remap now rewrites instance input
bindings too. The focused unit test covers that root cause, not only
the hierarchy symptom.

### Registered parent-composed routing is not the same as registered sibling routing
The registered sibling route proves a minimal stateful handoff:
earlier child output -> parent flop -> later child input. The next
route axis deliberately adds parent logic before that flop:
earlier child output or earlier parent route gate -> parent-local
combinational logic -> parent flop -> later child input.

This is controlled by
`hierarchy_registered_child_input_cone_prob`, not by overloading
`hierarchy_registered_sibling_route_prob` or
`hierarchy_parent_flop_prob`. The distinction matters because the
metric has to prove a different structure: the binding must pass
through a parent-local flop whose D input is itself a parent-local gate
with instance-output support.

The implementation now builds the D cone from the full available parent
source pool, with spontaneous nested flop generation disabled for this
route. It then repairs the root before allocating the final flop: the D
path must keep sibling-output support, when parent data inputs are live
it can add parent-port support, and when earlier parent flops are live
it can add a prior-Q companion to create a multi-stage registered
chain. If the repaired root is not already a substantive parent gate,
the generator wraps it in a non-collapsing XOR-with-all-ones parent
gate. That keeps the route signoff-clean and construction-time
deterministic while preserving the structural proof obligation.

Metrics now expose the route directly through
`child_input_bindings_from_registered_parent_composed_logic`,
`top_child_input_bindings_from_registered_parent_composed_logic`,
`registered_parent_composed_child_input_binding_fraction`, and
`top_registered_parent_composed_child_input_binding_fraction`. Current
HEAD also exposes the mixed registered-support subcase through
`child_input_bindings_from_registered_mixed_support`,
`top_child_input_bindings_from_registered_mixed_support`,
`registered_mixed_support_child_input_binding_fraction`, and
`top_registered_mixed_support_child_input_binding_fraction`. Current
HEAD also exposes the first multi-stage registered subcase through
`child_input_bindings_from_registered_multistage_parent_composed_logic`,
`top_child_input_bindings_from_registered_multistage_parent_composed_logic`,
`registered_multistage_parent_composed_child_input_binding_fraction`,
and
`top_registered_multistage_parent_composed_child_input_binding_fraction`.
The original focused proof is
`/tmp/anvil-hier-registered-child-input-cone-smoke-r2/manifest.json`
(`child_input_bindings_from_registered_parent_composed_logic = 3`,
`top_child_input_bindings_from_registered_parent_composed_logic = 3`,
`registered_parent_composed_child_input_binding_fraction = 0.75`,
`top_registered_parent_composed_child_input_binding_fraction = 0.75`,
`hierarchy_parent_local_flops = 3`), clean in Verilator, Yosys
`synth -noabc`, and the repo-owned Yosys with-ABC path. The repo-owned
Phase 4 gate now banks this as a required coverage fact at
`/tmp/anvil-tool-matrix-phase4-hierarchy-r27/tool_matrix_report.json`
with `coverage_gaps = []` and 216/0 pass-fail in Verilator plus both
repo-owned Yosys modes.

The focused mixed-support proof is
`/tmp/anvil-hier-registered-mixed-child-input-smoke-r1/manifest.json`
(`child_input_bindings_from_registered_mixed_support = 3`,
`top_child_input_bindings_from_registered_mixed_support = 3`,
`registered_mixed_support_child_input_binding_fraction = 0.75`), clean
in Verilator, Yosys `synth -noabc`, and the repo-owned Yosys with-ABC
path. The current-code coverage-only Phase 4 matrix probe at
`/tmp/anvil-tool-matrix-phase4-registered-mixed-r1/tool_matrix_report.json`
first banked `saw_hierarchy_registered_mixed_support_routing = true`
with `coverage_gaps = []`; the full downstream-clean `r27` bank now
carries the same fact with Verilator and both repo-owned Yosys modes.

The focused multi-stage registered proof is
`/tmp/anvil-hier-registered-multistage-child-input-smoke-r1/manifest.json`
(`child_input_bindings_from_registered_multistage_parent_composed_logic = 2`,
`top_child_input_bindings_from_registered_multistage_parent_composed_logic = 2`,
`registered_multistage_parent_composed_child_input_binding_fraction = 0.5`),
clean in Verilator, Yosys `synth -noabc`, and the repo-owned Yosys
with-ABC path. The current-code coverage-only Phase 4 matrix probe at
`/tmp/anvil-tool-matrix-phase4-registered-multistage-r1/tool_matrix_report.json`
first banked `saw_hierarchy_registered_multistage_routing = true` with
`coverage_gaps = []`; the full downstream-clean `r27` bank now carries
the same fact with Verilator and both repo-owned Yosys modes.

The focused multi-stage registered sibling proof is
`cargo test hierarchy_registered_sibling_routes_can_chain_through_parent_flops`.
It proves the direct registered sibling route can chain through earlier
parent-local Qs while keeping registered parent-composed counters at
zero. The `r27` Phase 4 matrix banks that as
`saw_hierarchy_registered_multistage_sibling_routing = true` through
the dedicated
`phase4_hier2_inst4_registered_sibling_multistage_state` scenario.

### Parent outputs can mix parent ports with child outputs
The first parent-output composition slice built output cones from child
`InstanceOutput` leaves only. That proved real parent-side logic above
children, but it left parent data inputs out of the parent output
surface.

The current parent-output builder now starts from the full parent
source pool. After module finalization, it rebuilds live pools from the
settled parent module and repairs every parent output that lost
structural child-output support. When live parent data inputs exist,
the same repair path also adds parent-port support to outputs that
otherwise only reached child outputs.

The post-final repair point matters. Cleanup can fold or replace drive
roots, so the invariant has to be checked after compaction, input
pruning, and profile enforcement have settled. The repair adds ordinary
parent gates over live nodes; it does not patch emitted SV.

Metrics expose the result as
`top_parent_port_composed_outputs`,
`hierarchy_parent_port_composed_outputs`,
`top_parent_port_composed_output_fraction`, and
`hierarchy_parent_port_composed_output_fraction`. The focused
regression is
`cargo test --test pipeline hierarchy_parent_outputs_can_mix_parent_ports_with_child_outputs`.
The repo-owned Phase 4 coverage gate now tracks this as
`saw_hierarchy_parent_port_composed_outputs`; the current-code
coverage-only matrix probe at
`/tmp/anvil-tool-matrix-phase4-parent-port-coverage-r1/tool_matrix_report.json`
first recorded `coverage_gaps = []` with that fact true. It skipped
Verilator/Yosys; the full downstream-clean Phase 4 `r27` bank now
carries the same fact with real tool validation.

### Hierarchy quality has to be visible in the numbers
The user requirement here is the right one: for hierarchy, ANVIL should
not depend on someone opening the emitted `.sv` and eyeballing whether
the composition looks plausible. The reports and manifests need to
carry enough exact facts that the result can be trusted numerically.

That is why the current hierarchy slice now has a dedicated
`DesignMetrics` layer instead of only per-module metrics and a few
coarse booleans. The current trustworthy design facts are:

- library size vs instantiated child count,
- unique-instantiated-module count and unused-library count,
- reuse / coverage ratios,
- top interface shape, including `top_clock_inputs` and
  `top_reset_inputs`,
- direct-vs-composed outputs and parent-port-composed output counts,
- control fanout to child instances,
- weighted child interface / node / flop load, and
- per-definition instantiation histograms.

The smoke at `/tmp/anvil-hier-metrics-smoke-r1` mattered because it did
more than prove the metrics serializer. It exposed two real root-cause
bugs that would have made those numbers lie:

- wrapper tops were creating shared `clk` / `rst_n` ports without
  tagging them as `Module.clock` / `Module.reset`; and
- control-port emission was using a too-local rule, so wrappers with no
  local flops could hide `clk` / `rst_n` even when those ports were
  still required by sequential descendants.

The durable rule now is exact and inductive:

- pure comb-only modules do not emit `clk` / `rst_n`;
- sequential leaves do emit `clk` / `rst_n`; and
- hierarchy parents keep `clk` / `rst_n` visible iff they carry local
  state or sequential descendants, all the way up the instantiated
  chain.

That rule is now pinned in IR helpers, validation, metrics, and the SV
emitter, plus direct regression tests for both the comb-only and
grandparent-wrapper cases.

The recursive planner widened the metrics contract too. Wrapper-only
facts were no longer enough; the numbers now have to describe the
**tree**. So `DesignMetrics` now also carries:

- `realized_min_leaf_depth`, `realized_max_leaf_depth`,
  `avg_leaf_depth`, `max_module_depth`;
- `module_defs_by_depth`, `module_occurrences_by_depth`,
  `instance_slots_by_parent_depth`;
- `avg_child_instances_by_parent_depth`,
  `min_child_instances_by_parent_depth`,
  `max_child_instances_by_parent_depth`;
- `child_instances_per_internal_module_histogram`,
  `min/avg/max_child_instances_per_internal_module`; and
- hierarchy-wide composition counters in addition to the top-only ones.

That is the current trust surface for recursive hierarchy quality: the
user should not have to inspect the `.sv` to tell whether ANVIL built
the requested tree shape.

### Literal-backed for-fold sources must be materialized before procedural part-selects
The repo-owned Phase 4 hierarchy gate exposed a real emitter defect in
the bounded procedural `for` surface.

The bad shape was not subtle:

- direct literal indexing such as `24'h86899[(i * 12) +: 12]`, and then
- an attempted blanket fix that emitted `(signal)[(i * 12) +: 12]`.

Neither is a robust answer for the downstream tools we care about.
Verilator and Yosys both rejected those forms during the hierarchy
matrix.

The correct fix is narrower and more truthful:

- keep ordinary named packed sources as `src[(i * K) +: K]`;
- but when the fold source is a constant, materialize it through a
  packed procedural temporary inside the surrounding `always_comb`;
- then index that temporary.

That preserves the intended structured surface, keeps the emitted SV
legal, and fixes the root cause instead of weakening the gate or hiding
the fold behind different syntax.

### Constant-backed slices must fold to literals, not literal indexing
The new under-instantiation hierarchy smoke exposed another emitter bug
in a different surface:

- `assign slice_26 = 20'h0[18:1];`

That is just as wrong as the old procedural literal-indexing bug, but
the right fix is even narrower here. When a `Slice` operand is a
constant, there is no need to emit a slice at all. We already know the
answer exactly.

So the deliberate rule now is:

- if the slice source is non-constant, emit the normal `src[hi:lo]` or
  `src[bit]` form;
- if the slice source is a constant, compute the sliced value in the
  emitter and print the narrower constant literal directly.

That keeps the output legal and simple, and it fixes the real cause
instead of wrapping an invalid shape in more syntax.

### The broadened Phase 4 matrix found the next runtime cost shape
After landing `num_child_instances`, the Phase 4 `tool_matrix` planning
was widened from the old "leaf count x comb/seq" wrapper sweep to four
more truthful representative profiles:

- `phase4_hier2_inst2_comb`  — exact library/instance cardinality
- `phase4_hier2_inst4_seq`   — repeated child-definition reuse
- `phase4_hier4_inst2_comb`  — under-instantiated library
- `phase4_hier4_inst4_seq`   — exact cardinality at the heavier end

That is the right coverage model for the current wrapper slice, and the
full refreshed rerun now closes cleanly at
`/tmp/anvil-tool-matrix-phase4-hierarchy-r7/tool_matrix_report.json`.
But the reruns also made the runtime story obvious: the heavy
sequential `hier4_inst4_seq` cases spend real time inside Yosys because
they elaborate/synthesize tiny wrapper tops over very large sequential
child libraries.

So the durable lesson is:

- this is a downstream cost shape, not a malformed-output bug;
- the refreshed exact / reuse / under-instantiation matrix is now
  actually banked cleanly at `r7`; and
- future Phase 4 work should keep watching those heavy sequential
  corners, because they are the place where hierarchy cost surfaces
  first even when the emitted RTL is valid.

### The recursive hierarchy gate must prove hierarchy, not quietly re-run the fattest leaf stress lane
When the Phase 4 gate was widened again to cover the newer recursive
and per-depth-branching surfaces, the first full rerun (`r8`) exposed a
different version of the same problem. The new coverage logic itself
was fine, but the recursive sequential scenarios were still borrowing
the heaviest Phase 1 motif-heavy sequential leaf profile.

That made the hierarchy gate pay for a huge amount of downstream Yosys
work that belonged to leaf stress, not to hierarchy proof. The proof was
therefore answering the right structural question with the wrong leaf
payload.

The right fix was not to drop the recursive scenarios and not to weaken
the coverage facts. The right fix was to decouple concerns:

- keep the recursive depth-2 and per-depth override profiles in the
  repo-owned Phase 4 matrix;
- keep the clean-tool requirement exactly the same; but
- switch the Phase 4 sequential hierarchy scenarios to a
  hierarchy-focused sequential leaf profile sized for hierarchy proof
  rather than Phase-1-scale leaf stress.

That is why the banked `r9` report closes quickly and honestly:

- the gate still proves wrapper exact / reuse / under-instantiation;
- it still proves recursive depth `2`;
- it still proves the per-depth override profile `0=4:4,1=2:2`;
- it still proves parent-side composition above instance outputs; and
- it no longer burns runtime re-proving the fattest leaf-stress shape
  just to answer a hierarchy question.

### Wrapped-add bounds must preserve a shifted single interval when it stays linear
The `e-graph` warning in
`/tmp/anvil-tool-matrix-phase1-real-r20/int_nodeid_e-graph_default/mod_8_0053.sv`
turned out not to be a generic "Yosys got grumpy" case. It exposed a
specific gap in ANVIL's unsigned-bounds reasoning for `GateOp::Add`.

The old logic did this:

- collect operand bounds;
- if the sum might wrap the target width, fall back to full-range.

That is safe, but it was too blunt for the real rhs shape:

- one non-exact interval (`or_22` bounded to `[0xe7, 0xff]`);
- plus exact constants (`0x0c` and `0xc4`).

In that case, the exact constants are not adding uncertainty. They are
just translating one interval around the unsigned ring. If the
translated interval still lands as one linear interval in unsigned
space, we should keep it. For the real failing case, `[0xe7, 0xff] +
0xd0 (mod 256)` becomes `[183, 207]`, which is still linear and is more
than enough to prove a 3-bit shift is always an overshift.

So the deliberate rule now is:

- if an `Add` node has exactly one non-exact interval operand and the
  rest are exact constants, combine the exact constants first;
- translate the one live interval by that exact wrapped addend; and
- keep the translated interval only when it stays linear (`start <= end`
  after modular translation), otherwise fall back to full-range.

That rule is intentionally narrow. It improves downstream cleanliness on
the real `shift >> wrapped_add` warning shape without reopening the
broader exact-set proof surface that earlier slices had to cap for
runtime reasons.

### `tool_matrix` frontier runs now use per-module checkpoints
`tool_matrix` now writes `<stem>.module-report.json` after each fully
processed module and supports `--resume`.

The resume contract is intentionally narrow:

- checkpoint reuse is allowed only when the current tool surface matches
  the checkpoint (`skip_verilator`, `skip_yosys`, `yosys_mode`);
- same-binary fast resume is allowed only when the checkpoint also
  carries a matching runtime fingerprint, a matching saved-`sv` hash,
  and a saved generator checkpoint;
- otherwise the regenerated module must still match the saved `.sv`
  text and module identity; and
- metrics are refreshed locally on resume instead of being treated as
  the reuse key.

This means resume is intentionally **byte-stable**, not "best effort".
If generator semantics change and a regenerated module no longer matches
the saved `.sv`, that old tree is evidence only; use a fresh `--out`
tree for the new semantics instead of trying to cross that boundary in
place.

That last point is important. In the real smoke proof, the saved `.sv`
matched exactly while the checkpointed metrics did not, which means
metrics are too strict a resume key even when the emitted artifact is
unchanged. The load-bearing truth for reuse is therefore the emitted
module, not the old metric blob.

The newer fast path exists to avoid replaying hundreds of already-proven
modules on the **same binary** just to reconstruct RNG state. Each
fresh checkpoint now records:

- a generator checkpoint (ChaCha stream position + next module index),
- a hash of the emitted `.sv`, and
- a fingerprint of the current `tool_matrix` binary.

When all three match, resume can restore the generator directly and
reuse the saved report without regenerating that module. If any of them
do not match, the old strict replay path stays in force. That keeps the
same byte-stable correctness bar while removing the most painful
same-build resume cost.

Older output trees without sidecars are still resumable: `--resume`
will validate the saved `.sv`, rerun the current tool surface once for
that module, and then write the new checkpoint sidecar.

Likewise, older sidecars that predate the generator-checkpoint metadata
still resume correctly; they simply pay the strict replay cost once and
are upgraded in place to the newer, faster format.

One more operational detail now matters in practice: once a proof or
cleanup change alters emitted `.sv`, an older frontier tree becomes
historical evidence only even if it was the latest live checkpoint at
the time. That happened to `/tmp/anvil-tool-matrix-phase1-real-r18`
after the rollback / compare-cleanup repairs: the tree still records a
real 372-checkpoint both-mode frontier, but current code must continue
from a fresh output tree instead of trying to "upgrade" it in place.

### Cleanup exact proofs must stay compare-aware without becoming broad again
The post-construction `fold_proven_gates` pass now follows a deliberate
split:

- the **general** cleanup exact prover stays tiny-only (small width,
  small support, small endpoint count) so it cannot reintroduce the old
  large-cone runtime blowups; but
- compare gates still get the bounded unsigned-compare proof even when
  the cone is too large for the general cleanup exact gate; and
- shift gates (`Shl` / `Shr`) may still use the **bounds-only** exact
  result even when the cone is too large for the general cleanup exact
  gate.

That split exists because "large cone" and "cheap compare tautology"
are not the same thing. A dead-selector rhs can make `x >= 0` or
`1 < dead_rhs` obviously constant even when the whole cone's endpoint
set is wider than the general cleanup exact gate allows. Likewise, a
large-endpoint rhs range can still make `2'h1 >> rhs` or `x << rhs`
obviously zero. Those compare/shift revisit paths are therefore
downstream-cleanliness exceptions worth keeping separate from the
broader exact-value cleanup budget.

### `constant_prob = 0.1`
Default chosen to prevent constants from dominating cone leaves. Real synthesis-stress workloads may want lower (≤ 0.05); aggressive pattern coverage may want higher. Revisit after first seed sweep with metrics on what fraction of generated cones survive non-triviality on the first attempt.

### `terminal_reuse_prob = 0.3`
Probability that, when a cone reaches a leaf decision and the signal pool has matching-width entries, it picks an existing pool entry rather than emitting a constant or recursing further. Higher = more sharing-like behavior even before Phase 3 explicitly turns on `share_prob`. Default is a guess; tune after Phase 1.

### `share_prob = 0.3` default
The non-leaf DAG-sharing fork is enabled by default at a modest rate. Every operand has a 30% chance of terminating at an existing pool entry rather than recursing. This is the Phase 2 guiding mode: cones are a mix of tree and DAG shapes, chosen per recursion point. Raise (0.5–0.9) for fanout-stress generation; lower (0.0–0.1) for wide-sprawling tree-ish cones. `share_prob = 0.0` does not produce *pure* trees — `pick_terminal` still reuses matching-width pool entries at forced leaves. The distinction is: `share_prob` controls *non-leaf* sharing; leaf-level reuse is always on.

### Phase 2 share-gate metric: normalize by total nodes
The first repo-owned Phase 2 gate attempt tried to prove "controlled
sharing factor" with raw `total_shared_nodes`. The real run showed that
proxy was backwards: when `share_prob` rises, ANVIL often reuses enough
existing structure that the entire graph collapses, so the *absolute*
count of shared nodes can fall even while the graph becomes more
shared. The repo-owned `tool_matrix --phase2-share-gate` therefore uses
`shared_node_fraction = total_shared_nodes / total_nodes` as the
monotonic proof metric and records node-count collapse alongside it.
Current closure proof on `/tmp/anvil-tool-matrix-phase2-share-r1`:
`0.4122 @ share_prob=0.0`, `0.4232 @ 0.3`, `0.4386 @ 0.9`, while
`avg_nodes/module` drops from `4727.56` to `3525.01` to `2117.76`.

### Phase 3 should have its own structured-surface gate
Once the `case`, `casez`, bounded `for`-fold, selectable
`Slice` / `Concat`, and variable-shift surfaces were all landed, the
remaining honest Phase 3 blocker was no longer feature breadth. It was
evidence breadth.

That shape now lives in the harness itself as `tool_matrix
--phase3-structured-gate`. The dedicated matrix covers all three live
construction strategies under `identity_mode = node-id` +
`factorization_level = e-graph`, and the report is allowed to go green
only if it proves the landed Phase 3 surfaces directly:

- priority encoder
- one-hot and encoded comb mux
- procedural `case`
- procedural `casez`
- bounded procedural `for`-fold
- one-hot and encoded flop mux
- selectable `Slice`
- selectable `Concat`
- variable shifts

The closure proof now lives at
`/tmp/anvil-tool-matrix-phase3-structured-r4/tool_matrix_report.json`
with `21` scenarios, `210` total modules, `coverage_gaps = []`, and
`210/0` pass-fail in Verilator plus both repo-owned Yosys modes.

### Semantic merge proofs also need a cone-size budget
The first real Phase 3 gate run did not fail in Yosys or Verilator. It
stalled inside `merge_equivalent_gates`, specifically
`semantic_cone_proof -> evaluate_node_under_assignment`.

The root cause was subtle but real: *small endpoint support is not a
sufficient runtime guard by itself*. A settled cone can depend on only
2 or 3 canonical leaf endpoints and still contain a very large internal
graph. Brute-forcing every assignment through that whole graph turns
compaction into a whole-cone evaluator.

The durable fix is now explicit in `src/ir/compact.rs`:

- cleanup-time exact proofs stay on their already-strict tiny-cone path
- semantic merge proofs have their own reachable-cone budget
- once that budget is exceeded, compaction falls back to the
  structural proof path instead of chasing semantic equivalence at any
  cost

That keeps the semantic merge fragment live where it is valuable while
stopping large settled cones from becoming a runtime trap.

### `gate_*_weight` defaults
3:2:1:1:1 (bitwise:arith:struct:compare:reduce). Bitwise dominates because bitwise gates are the most type-flexible and produce the widest cones. Comparisons are weighted lower because they collapse the width to 1, which limits downstream cone depth. These are gut-feel; replace with measurements when phase-1 sweeps land.

### `flop_mux_encoding_prob = 0.5`
Default chosen to give equal motif exposure to OneHot and Encoded styles across a random seed sweep. If post-synthesis metrics show that one style dominates as a bug-finding target, bias the default. The knob also allows users to run workloads stressing only one style for targeted testing.

### `flop_qfeedback_prob = 0.5`
Default 50/50. No empirical data yet. Real designs probably lean heavier on QFeedback (hold-on-no-write is far more common than zero-on-no-write), but generating the less-common pattern is precisely where random generation earns its keep. Revisit with data.

### QFeedback-in-Encoded: replace `data_0` with Q
Alternative considered: add Q as an extra (M+1)th entry encoded with the largest select value. **Rejected** because:
- It would require the sel bus to be one bit wider than `ceil(log2(M))` whenever M is a power of 2, breaking the clean "M mux entries ⇔ `ceil(log2(M))`-bit sel" invariant.
- The "slot 0 is Q" convention mirrors common RTL idioms where the zero-index / reset state is treated specially.
- It keeps M as the single knob for mux entry count across both styles.

---

## Rejected alternatives

### Annotated-EBNF runtime engine
Considered: a generic attribute-grammar interpreter that reads an annotated SV grammar at runtime and produces output. **Rejected** because:
- SV's grammar is enormous; encoding all of it is months of work for productions we will never emit.
- Threading mutable scope/driven-set/flop-worklist state through pure inherited/synthesized attributes is awkward; it really wants `&mut Context`.
- Extending the grammar engine for a new motif is comparable in effort to adding a Rust enum variant + emitter arm, with much worse error messages.

The grammar view is preserved as a *correctness argument* (every constructor preserves invariants ⇔ every production is valid under its attributes). Not as a runtime artifact.

### Oracle / reference simulator
Considered: a Rust evaluator that walks the IR with concrete input vectors and produces expected output values, used both for non-triviality filtering and for downstream tool testing. **Rejected** because:
- Doubles implementation effort.
- Introduces a second correctness question (is our interpreter LRM-correct?).
- The user's stated goal is *generation*, not building a full shadow
  simulator or tool-oracle inside `anvil`.
- Non-triviality is cheaper to enforce by dep-set tracking + structural rules; multi-vector evaluation is overkill for that use case.

That does **not** lower the output-quality bar. The generator is still
expected to emit modules that run cleanly in downstream tools.
Verilator / Yosys are external validators, not the place where
`anvil` gets to finish the job.

### `always_comb` + `case` for encoded-mux flop D

Considered for the Encoded-style flop D: emit an `always_comb` block with a `case (sel)` statement driving D. **Rejected** in favor of a chained ternary over `Eq(sel, k)` because:

- The emitter already handles `Mux` and `Eq` as ordinary `GateOp` variants; nothing new is required.
- `case` would require introducing procedural block emission (`always_comb`) and name-binding for the case target, which is a bigger scope than a uniform expression-level SV emitter.
- Synthesis tools produce the same netlist from both forms for well-formed one-cycle muxes; the readability difference only matters to a human reader.

If a future motif (e.g., FSM state encoding) genuinely requires `case`, revisit then.

This remains the right decision for **flop D** muxes even after the
Phase 3 case-mux slice landed. The new case surface is a separate
combinational block motif with its own knob (`case_mux_prob`) and its
own structured gate kind; the flop path stays expression-based and
keeps its existing chained-ternary semantics.

### Casez muxes are a separate structured surface, not a decorated case-mux

The right shape for the `casez` slice was **not** to smuggle wildcard
syntax into the existing `CaseMux` gate or to make the emitter infer
question-mark patterns from ordinary indexed arms. `case` and `casez`
exercise different frontend/elaboration paths, so the IR should say so
explicitly.

That is why the slice introduced a distinct `GateOp::CasezMux` plus its
own knob (`casez_mux_prob`). Each arm stores a constant pattern, a
constant wildcard mask, and a data node. The emitter renders those as a
procedural `always_comb casez (sel)` block; the validator enforces the
constant-pattern contract; and the exact evaluator in `ir::compact`
understands the same first-match semantics.

Generation deliberately keeps the wildcard patterns **non-overlapping**
by construction. That preserves the intended "wildcarded mux" surface
without accidentally turning the new motif into a priority-case stressor
on top of the syntax stress we actually wanted.

### Bounded unrolled logic belongs in the IR as a block, not as emitter sugar

The right shape for the statically bounded `for` slice was to model it
as its own structured combinational block, not to hope that repeated
operator trees would "look enough like a loop" in emitted SV.

That is why the slice introduced a distinct
`GateOp::ForFold { kind, trip_count, chunk_width }` plus its own knob
(`for_fold_prob`). The IR carries the fold kind (`xor` / `or` / `and` /
`add`), the exact static trip count, and the chunk width. The single
operand is a packed source bus of width `trip_count * chunk_width`.

The emitter then has one honest job: declare the target as `logic`,
emit an `always_comb begin`, initialize the accumulator, and render a
bounded `for (int i = 0; i < N; i++)` loop over
`src[(i * chunk_width) +: chunk_width]`. The validator enforces that
shape directly, and the exact evaluator in `ir::compact` evaluates the
same chunk-fold semantics.

This keeps the syntax surface real. Downstream tools see an actual
procedural bounded loop, not just an expression tree that happens to
resemble one semantically.

### Selectable Slice/Concat must be non-degenerate by construction

Making generic `Slice` / `Concat` first-class selectable shapes was not
just a matter of adding them to `pick_gate`. The naive version would
have "landed" them and then immediately lost them again:

- selectable `Slice` would often degenerate to the full-width identity
  and disappear under the peephole layer
- selectable `Concat` would sometimes degenerate to the single-operand
  identity and disappear the same way

So the right design is to make the selectable forms intentionally
non-degenerate:

- selectable `Slice` always uses a source wider than its high bit
- selectable `Concat` always partitions the output width across at
  least 2 operands

That keeps the new surface honest. We are exercising real frontend
surface area, not just incrementing counters on gates that the settled
graph will erase as trivial identities.

### Late mixed-constant cleanup after remaps

Intern-time constant folding is not enough by itself once the
post-construction cleanup passes start remapping settled graphs. A gate
that was clean when originally interned can later become something like
`1 + x + inner`, where `inner` is subsequently proven/remapped to `1`.

The right place to address that is **not** to overcomplicate
associative flattening or to relax the strict duplicate doctrine; it is
to run a small late cleanup pass on the settled graph. That is now
`fold_mixed_associative_constants` in `src/ir/compact.rs`, wired after
the posthoc associative-normalisation points. It re-aggregates
associative constants (`1 + x + 1 -> x` at width 1, `1 + x + 1 -> 2 +
x` at width 8, `3 * x * 5 -> 15 * x`, etc.) after remaps expose those
opportunities.

### M = 1 mux arm

Excluded from `pick_mux_arm_count` by design. A 1-arm mux is algebraically `sel ? data_0 : 0` (ZeroDefault) or `sel ? data_0 : Q` (QFeedback) — in either case a trivially-simplified shape that adds no motif diversity over what a simple 2-arm mux or an M=0 direct cone already covers. Allowing M=1 would bloat the generator's decision space without expanding the generated-SV distribution meaningfully.

### `#![allow(clippy::too_many_arguments)]` in `src/gen/cone.rs`

The cone-recursion helpers legitimately thread 5–8 context references (`Generator`, `Module`, `SignalPool`, `FlopWorklist`, `width`, `depth`, `exclude`, sometimes more). Packaging them into a `Ctx` struct would help readability but also forces mutable-borrow juggling that fragments the code with no semantic benefit. The lint is silenced at the module level rather than per-function to avoid the ceremony of annotating every helper. Not recommended for modules outside `gen/cone.rs`.

### Generate-then-validate (filter loop)
Considered: emit random IR with looser invariants, then run the validator and discard rejected outputs. **Rejected** because:
- Untestable bound on generation time.
- Tempts contributors to weaken constructors and rely on the validator, leading to silent correctness drift.
- Complex invariants (dep-set non-emptiness) are far more expensive to check post-hoc than to maintain incrementally.

The bounded retry in `cone::build_cone_with_retry` is the *only* exception — it exists because dep-set non-emptiness depends on terminal selection in a way that cannot always be predicted at the gate level (e.g., when all available pool entries happen to be constants). Retry budget is small (4) and falls back to accepting the last attempt.

---

## Implementation gotchas

### Reproducibility hazards
- `HashMap` iteration order is *not* stable across builds. If iteration order ever affects output, switch to `BTreeMap` or sort the keys explicitly. The current code avoids this; new contributions must too.
- `f64` non-associativity is fine for probability comparisons but never use `f64` arithmetic to compute IR fields — only RNG-driven discrete choices.
- `rand::thread_rng()` is forbidden everywhere. All randomness flows from the seeded `ChaCha8Rng` in the `Generator`.

### IR arena indexing
`NodeId` is `u32`. We use `Vec<Node>` indexed by `u32`. This is fine for the foreseeable size range (modules of ≤ 10⁶ nodes). If we ever need more, the change is local to `ir/types.rs`.

Indices are stable for the lifetime of a `Module` because we only ever push, never remove. The bounded retry in `cone::build_cone_with_retry` rewinds by `Vec::truncate`, which is safe because no other code holds `NodeId`s referring to the rewound region.

### Width 0 is illegal
`Config::validate` requires `min_width >= 1`. Width-0 signals are not synthesizable and SV does not allow them. Do not relax this.

### 128-bit constant cap
Constants fit in `u128`. Modules with `max_width > 128` are technically allowed, but the constant generator emits `0` for any width ≥ 128. This is a deliberate simplification; widening the constant representation is straightforward when needed.

---

## Testing strategy notes

- **Unit tests** live in each module under `#[cfg(test)] mod tests`. Test IR constructors enforce invariants; test gate width rules; test dep-set propagation; test the emitter on hand-built IRs.
- **Integration tests** in `tests/`: cross-seed generation + IR validation + reproducibility.
- **External smoke tests** (Verilator lint, Yosys synth) are gated by env vars so they are skippable for developers without those tools. CI must enable them.

A failed external smoke test is always a generator bug. Do not "fix" by tweaking generator output — find the root invariant violation and fix it.

Same principle for the IR validator (`src/ir/validate.rs`): if it rejects real generator output, that's a generator bug. The validator is an active safety net, not a gate to be worked around. The per-gate arity + width checker added in slice `2026-04-15-0008` is specifically designed to catch width bugs in the new flop-mux assembly code, where gates are constructed by hand rather than by recursion — the most likely place for a width-arithmetic slip.

### Canonical state backreferences are validator-owned (2026-04-20)

Once `merge_equivalent_flops` started rewriting state after drain,
`Flop.id`, `Flop.q`, and `Node::FlopQ { flop, .. }` stopped being
"born correct and forgotten" fields. They are now recovery-critical
identity links that a bad renumbering pass can corrupt.

`ir::validate::validate` now owns that contract:

- every output drive root exists before root inspection;
- `m.flops[idx].id == idx`;
- `Flop.d`, `Flop.q`, and every `NodeId` stored inside `FlopMux`
  exist;
- `Flop.q` points at a `Node::FlopQ` whose backref and width match
  the owning flop; and
- every `Node::FlopQ` points at a real flop and is that flop's
  canonical `q` node.

Keep the emitter dumb. If any of these invariants fail, fix the
producer or rewrite pass; do not add emitter-side repair logic.

### Compaction now legitimises dynamic absorbing folds (2026-04-20)

Before `compact_node_ids`, the cautious rule for absorbing constants
was "only fold if the other operand is not a gate", because
`x & 0 -> 0`, `x | all_ones -> all_ones`, and `x * 0 -> 0` would
otherwise orphan a dynamic subgraph immediately.

That restriction is now obsolete. Finalisation already performs a
reachability compaction from real roots and rebuilds the dedup tables,
so these local identities are safe to fire regardless of whether the
other operand is a gate. In other words: once compaction exists, the
correctness risk is no longer "did we orphan something?" but "did we
miss an identity we should have collapsed?"

The practical consequence showed up in tool smoke:

- the remaining seed-42 Verilator `UNSIGNED` / `CMPCONST` warnings
  were not tool quirks;
- they were missed IR-local tautologies; and
- the right fix was to strengthen the rewrite ladder
  (absorbing folds, unsigned boundary comparisons, const-selector
  muxes), not to suppress or special-case Verilator.

This is the pattern to keep following for the NodeId-identity roadmap:
when equivalent local forms are discovered in emitted SV, first ask
whether they should have already become the same node in the IR.

### Signoff-quality and downstream-tool exercise are not competing goals (2026-04-20, refined 2026-04-26)

The user clarified the product direction explicitly, and later refined
the terminology around it:

- `anvil` should become a signoff-level quality random
  by-construction synthesizable RTL generator;
- generated HDL artifacts should be accepted by downstream HDL
  consumers by default; and
- `anvil` corpora should still be rich enough to exercise parsers,
  elaborators, RTL compilers, linters, simulators, synthesizers, and
  similar consumers.

Those statements are compatible. The project is **not** trying to expose
tool bugs by emitting junk, malformed syntax, or semantically dubious
RTL. The downstream-tool exercise value comes from breadth, interaction
richness, factorization pressure, stateful motifs, hierarchy, memories,
and other legal-but-hard combinations that downstream HDL consumers
should accept. Verilator and Yosys are repository validation tools for
that acceptance promise, not the only product targets.

When choosing between slices, prefer work that strengthens one of these
two axes without regressing the other:

1. broader / harder legal design space; or
2. stronger confidence that generated output is clean and robust in
   downstream HDL consumers.

### Purpose terminology clarification (2026-04-26)

The user clarified the wording around ANVIL's purpose:

- avoid calling ANVIL "constrained-random" unless that term is
  explicitly redefined away from SystemVerilog/UVM-style user-authored
  constraints or solver-driven randomization;
- the preferred short description is **random by-construction
  synthesizable SystemVerilog RTL generator**;
- ANVIL targets generated HDL artifacts that downstream consumers can
  accept: parsers, elaborators, RTL compilers, linters, simulators,
  synthesizers, and related tools;
- Verilator and Yosys are repository validation tools for syntax,
  elaboration/lint, and synthesis acceptability, not the only product
  targets; and
- ANVIL-generated corpora can still be used to stress downstream tools,
  but that is a use of the legal generated artifacts, not a license to
  describe ANVIL as primarily a malformed-input fuzzer or generic
  toolchain stress tester.

Follow-up gotcha (2026-04-27): package metadata is part of that same
terminology surface. `Cargo.toml` must not keep stale
`constrained-random` wording after README, Rustdoc, and mdBook text have
been corrected; Cargo metadata is visible to tooling and cold-start
readers before they open the longer docs.

### Verbatim user doctrine: structure over intended functionality (2026-04-20)

The following user guidance is intentionally logged **verbatim** because
it is doctrinal and should steer future implementation choices:

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

Operational consequence: optimize ANVIL primarily for structural
legitimacy, synthesizability, complexity, and downstream-tool
ingestibility. Treat whole-module function correctness as out of scope
unless a feature introduces a local block motif whose own behavior is
well-defined by construction.

### Broader artifact-family mandate (2026-04-20)

The user then broadened the scope again and explicitly corrected one
important boundary:

> It might sound contradictory but in addition to what's already
> described in the roadmap, book and live docs, I think it would good
> to include support for such things in the roadmap, book and live docs
> in order for ANVIL to be able address a lot more types of SV files
> formats as output. Being able to generate various types of pseudo
> random files for various types on downstream consumers would be a
> great plus, I think.
>
> In fine, I want ANVIL to be able accurately and precisely address the
> initial request, in full.
>
> ANVIL shall be the go to tool for everything (pseudo random) HDL
> generation related thing.
>
> I don't think this contradict the current roadmap of AMVIL that much,
> it is just that we are broadening the type HDL outputs we can target.
>
> As you wrote it clearly above, right now ANVIL is still a "leaf-module
> typed circuit generator", I agree.
>
> We need to start somewhere, but that is not the end goal.
>
> So we need to be able to embrass more output artifact types.
>
> This "valid-by-construction synthesizable lane” is still valid, and
> it will stay that way!
>
> We are just generating more types of valid-by-construction
> synthesizable artifacts.

Operational consequence:

- the current leaf-module typed circuit generator is now explicitly the
  **first artifact family**, not the whole product;
- future broadening still stays inside the
  **valid-by-construction synthesizable** contract;
- the first requested additions are oracle-backed micro-design corpora,
  source-level parameter / hierarchy / package IR, and explicit
  expected-facts manifests; and
- an earlier idea of broadening via invalid/reject corpora is **not**
  the adopted direction for ANVIL after the user's correction above.

This is a real scope change for planning, not a soft aspiration. The
roadmap now needs explicit phases for these broader synthesizable
artifact families.

### Repo-owned tool matrix harness (2026-04-20)

The "no hidden bias" / "exercise all axes" doctrine now has an
executable first form in the repo: `src/bin/tool_matrix.rs`.

The design choices for this harness are deliberate:

- it is a Rust binary in-repo, not an external shell script, so it can
  reuse `Config`, `Generator`, metrics, and manifest formats directly;
- it uses a **curated matrix**, not one giant Cartesian product, so the
  sweep stays fast enough to run routinely while still covering the
  load-bearing axes:
  - interleaved ladder sweep across `relaxed` plus every
    `factorization_level` rung,
  - strategy sweep across `sequential` / `shuffled` / `interleaved`,
  - a share-heavy comb-only profile,
  - a motif-heavy sequential profile;
- it reuses structural metrics as the coverage surface instead of
  inventing a second observability stack; gate kinds, block counters,
  and knob roll attempts/fires already tell us whether a scenario
  actually exercised what it claimed to stress; and
- it exits non-zero on downstream-tool failures because the point is to
  surface generator bugs, not to produce a pretty report while quietly
  accepting red runs.

The first smoke run after landing the harness was immediately useful:
it found one real emitter bug (`logic[0:0]`-style scalar slice
emission) and, after that fix, reduced the remaining failures to the
warning-cleanliness bucket (`CMPCONST` / `UNSIGNED` under Verilator).
That is exactly the intended feedback loop for the tool-clean
industrialization lane.

### Comparison warning-cleanliness is partly a generator concern, not only a factorization concern

The follow-up `tool_matrix` slice made an important distinction
explicit in code: obviously-constant unsigned comparisons are not just
"optional peephole opportunities". They are also by-construction
tool-cleanliness hazards.

That means ANVIL now has an **always-on generator-side proof path** for
comparisons in `src/gen/cone.rs`, independent of
`identity_mode` / `factorization_level`. If the generator can already
prove that a comparison is constant, it emits the constant directly
instead of relying on the factorization ladder to clean the shape up
later.

Current proof layers:

- conservative unsigned bounds for easy local identities (`x & 0 = 0`,
  `x | all_ones = all_ones`, `x * 0 = 0`, overshift-to-zero,
  select-known muxes, etc.);
- exact finite-set reasoning for comparison operands up to 8 bits
  wide; and
- replicated-concat correlation handling for shapes like `{N{bit}}`,
  so repeated copies of the same leaf are not treated as independent
  free variables during the proof.

This is intentionally narrower than full semantic factorization: it is
there to keep emitted RTL cleaner across *all* identity/factorization
modes, including `relaxed` and low rungs like `none` / `cse`.

Two implementation refinements became load-bearing once the real
`--phase1-gate` run started surfacing concrete warning files:

- **Exact proof must short-circuit once the result is already forced.**
  A small-width node can depend on a wider cone through `Slice`, so
  "walk every operand recursively until all are exact" is too blunt. If
  an exact prefix has already forced the result, the helper must stop:
  `6'h16 | 6'h39 | tail` at width 6 is already `6'h3f`; `2'h1 * 2'h2 *
  2'h2 * tail` at width 2 is already `0`; `x ^ x` is already `0`; and
  `x <= x` is already `1`. Letting the proof recurse into an irrelevant
  non-exact tail just turns an exact fact into an unnecessary `None`.
- **The small finite-set engine and the settled-graph exact-value
  engine need the same short-circuit doctrine.** The first catches
  narrow local cones directly; the second matters because
  `node_unsigned_bounds` asks "is this gate already exact?" before it
  falls back to interval reasoning. If only one engine gets the
  shortcut, the other can still miss exactly the same downstream
  warning.

Another refinement became necessary once the real `int_nodeid_cse`
frontier hit a correlation-heavy one-hot-mux cone: **exact finite-set
reasoning must also be budgeted.** The helper now carries a shared work
budget and memoizes both exact results and "unknown" results, so it can
still prove small exact facts on narrow cones without turning itself
into an exponential runtime trap on shared cartesian searches. The
durable contract is "prove what is cheap and crisp; otherwise return
`None` and fall back to the cheaper proof layers."

The next fresh-current-code `operand-unique` frontier made one more
refinement necessary: **budget alone is not a good enough admission
rule.** Even a budgeted proof can still waste generator time if ANVIL
keeps entering exact finite-set reasoning on larger shared cones whose
endpoint support is already beyond the intended proof domain.

So the contract is now sharper:

- exact finite-set reasoning is for **small width and small endpoint
  support**, not just small width; and
- the current support cap is **3 canonical leaf endpoints**.

That support cap applies both to `prove_node_exact_value` on one cone
and to the combined endpoint set used by comparison folding. This keeps
the proof useful where it is strongest, while making larger shared
cones stay on the cheaper proof layers instead of burning CPU proving
finite-set facts that are not load-bearing for cleanliness.

The first fresh-current-code both-mode rerun exposed a second, more
basic compare-cleanliness gap: **the cheap proof layer must know a few
arithmetic reflexive identities too, not only comparison tautologies.**

The concrete failing shape was:

- `sub_16 = mul_17 - mul_17`
- `and_49 = mul_18 & mul_18 & sub_16`
- `lt_0 = add_13 < and_49`

Verilator quite reasonably warned that the unsigned comparison was
constant. The missing fact was just `x - x = 0`.

The exact finite-set engine was not the right place to rely on for this
because it may legitimately decline a cone. The **cheap** layer has to
know it too. So `exact_gate_value` and `node_unsigned_bounds` now both
encode reflexive subtraction directly. Durable rule:

- local exact/bounds proofs should carry the cheapest algebraic facts
  that directly prevent mainstream tool warnings, even when those facts
  do not require the heavier finite-set prover at all.

### Downstream warnings are a generator bug, and the final graph gets a last proof pass

The follow-up slice closed the remaining `tool_matrix` warning bucket by
making two policy changes explicit in code.

First, ANVIL now runs a post-construction proof-cleanup pass in
`src/ir/compact.rs` (`fold_proven_gates`) after cone construction and
again after the sharing/remap passes settle. The key distinction is
timing: some exact proofs are not visible when a gate is first
constructed, but become visible later once remaps, merges, or other
local simplifications have changed the graph that the gate actually
sees. That pass:

- rewrites any gate whose current cone is provably exact into a
  constant in place; and
- rewires muxes whose selector is now provably constant.

One more settled-graph wrinkle showed up immediately afterwards:
remap-producing post-construction passes can reintroduce legal
associative nestings **after** the intern-time Associative layer has
already done its work. The live example was a width-1 `Add` whose
operand was later remapped to another width-1 `Add`, leaving
`nested_associative_operand_count = 1` at default knobs even though
flattening was still legal under the strict duplicate policy.

The durable rule is: **any pass that can change which already-built
node an operand points at may need to restore associative normal form
afterwards.** ANVIL now does that with
`flatten_posthoc_associative_gates(&mut Module)` in `src/ir/compact.rs`
after `fold_proven_gates` and after `merge_equivalent_gates`. The pass
uses the same duplicate policy as the intern-time Associative layer:
`And`/`Or` dedup, `Xor` pair-cancels, `Add`/`Mul` flatten only when the
flat list would still be legal at the current
`operand_duplication_rate`.

The proof stack now has three complementary layers:

- construction-time local proofs in `src/gen/cone.rs`,
- post-construction exact-value cleanup on the settled graph, and
- bounded semantic identity / sharing for the `e-graph` fragment.

One more durable constraint became explicit when the fresh current-code
`nodeid-cse` frontier stalled during resume: sampling the live process
showed the hotspot in `ir::compact::fold_proven_gates` /
`semantic_exact_value`, not in Yosys or Verilator. The settled-graph
cleanup prover is therefore intentionally **stricter** than the
generator-side semantic-sharing passes. Today it only brute-forces cones
that are all of:

- at most 8 bits wide;
- at most 10 total support bits; and
- at most 3 canonical leaf endpoints.

If a cone falls outside that tiny cleanup surface, the pass memoizes
`None` immediately and moves on. Durable rule: late proof-cleanup exists
to scrub obvious constants for downstream-tool cleanliness, not to widen
the main identity/factorization contract at arbitrary runtime cost.

### Narrow slices of wide cones are still narrow proof domains (2026-04-20)

The next live warning bucket made a subtle point painfully concrete:
the small finite-set engine is allowed to be width-bounded, but it is
not allowed to treat a narrow `Slice` result as "unprovable" just
because the source cone is wider than 8 bits. A 14-bit or 25-bit source
feeding an 8-bit slice still yields an 8-bit proof problem.

The durable implementation rule is:

- if a narrow slice's source is already exact, use that exact value;
- otherwise, if the source is too wide for direct enumeration, fall
  back to the full narrow output domain instead of returning `None`.

That fallback is conservative but still useful: it keeps later local
operations (`Or` with forcing constants, exact shifts, subtract-small,
dynamic overshift) in the proof path, which is enough to recover exact
facts like "this `Shr` is forced to zero". Returning `None` too early
throws away that whole proof chain.

One more shift-specific wrinkle showed up later in the fresh
`associative` frontier: some rhs cones are too large for the general
small-support exact enumerator **as whole cones**, but still have a
tiny value domain because they are really just boolean-mask arithmetic
(`{8{bit}} + constant`, similar patterns). The durable rule is:

- shift overshift proofs may use a tiny-domain rhs fallback for narrow
  boolean-mask arithmetic, even when the whole cone is too large for
  the main exact small-set engine.

That fallback stays intentionally narrow: width <= 8, tiny result-set
cap, and only a few structural forms. It exists to suppress pointless
dynamic shifts whose rhs is semantically always oversized, not to
replace the main semantic-sharing machinery.

### Finalisation liveness must be output-rooted, not flop-table-rooted (2026-04-20)

The dead-register Verilator warning exposed a mismatch between Rule-18
gate liveness and sequential liveness. The old compaction pass rooted
every `flop.q` unconditionally because the flop existed in `m.flops`.
That preserved dead state even when no output cone, live flop D-cone,
or other retained logic ever consumed that Q.

The durable rule is:

- start final liveness from output drive-roots;
- when the walk reaches a live `Node::FlopQ`, mark the owning flop
  live and pull in its `d` / mux-held nodes;
- drop any flop whose `Q` is never reached by the live graph.

That is the sequential analogue of Rule 18: state is live because it is
observed by retained logic, not because it once got allocated.

### Post-remap identity cannot violate strict Add/Mul duplicate policy (2026-04-20)

Late proof / sharing passes operate after construction, so they can
collapse two previously-distinct child cones to one canonical node.
That is fine in general, but under strict `operand_duplication_rate`
the final emitted IR is still not allowed to contain duplicate
`NodeId`s inside an `Add` or `Mul` operand list.

The durable rule is therefore stronger than "the remap is semantically
valid":

- a candidate remap is only acceptable if every strict `Add` / `Mul`
  consumer remains duplicate-free after the rewrite.

ANVIL now enforces that by pruning duplicate-introducing remaps before
they are applied in `fold_proven_gates` and `merge_equivalent_gates`.
This preserves the default "zero duplicate operands" doctrine without
backing away from late exact-value cleanup or bounded semantic sharing.

### Evidence slices are legitimate when the real gate frontier moves materially (2026-04-20)

Not every important slice changes code. Once the user set the quality
bar as "no warnings or errors from Verilator and Yosys", the real
`tool_matrix --phase1-gate` run became part of the implementation loop,
not just a nice-to-have afterthought.

That means there is a legitimate kind of slice whose output is:

- a materially advanced real downstream-clean frontier,
- recorded precisely (scenario names, module counts, command line), and
- committed into the live docs so the next session does not restart the
  same evidence climb from memory or vibes.

The key is that the checkpoint must be **material**, not cosmetic. In
this session, moving from the earlier 76-module clean frontier to 246
clean modules across multiple identity/factorization lanes cleared that
bar easily. It changed what we know about the repaired generator.

So the durable rule is:

- if a long real gate run advances the proven clean frontier
  substantially, it is acceptable to checkpoint that evidence as its own
  slice, even if no code changed in that commit.

That rule matters for crash recovery too, which is exactly why the
commit workflow is strict in the first place.

Second, the repo-owned downstream harness now treats warnings as
failures rather than as "successful but noisy" runs. `tool_matrix`
scans tool output for warning markers and marks the invocation failed
even if the process exit status is zero. The Yosys script was also
tightened from `synth` to `synth -noabc` so the matrix does not accept a
self-inflicted ABC combinational-network warning and then pretend the
run was clean.

This is a durable project rule now: for repo-owned Verilator/Yosys
evidence, "green" means no errors and no warnings.

### The 1000-module Phase 1 gate should be a first-class harness mode

Once the smoke matrix was green, the next missing piece was not more
doctrine. It was executable ergonomics. The Phase 1 exit criterion had
become "run the same harness, but remember to multiply the scenario
count, pick a large enough `--modules-per-scenario`, and also remember
that coverage gaps must fail."

That shape now lives in the harness itself as `tool_matrix
--phase1-gate`:

- it auto-enables coverage-gap failure; and
- it raises `modules_per_scenario` high enough to generate at least
  1000 modules total across the built-in scenario set.

The deliberate choice here is to encode the gate in the repo-owned tool
rather than leaving the phase-exit arithmetic in roadmap prose. When a
quality gate matters, the project should be able to invoke it directly.

### Codebase suitability assessment: four steering gaps (2026-04-20)

The short answer to "is the existing codebase suited to the goal?" is:
**yes, as a foundation; no, not yet as a finished system**.

Why "yes": the architecture already matches the problem. `gen` builds a
typed IR instead of text, `Module::intern_gate` is a single
construction-time chokepoint for combinational identity,
`ir::compact` owns post-drain cleanup and state-finalisation work,
`validate` owns the invariant contract, `config` keeps the control
surface explicit, and the SV emitter stays deliberately dumb. That is
the right shape for a signoff-grade legal-RTL generator.

What still needs to stay explicit:

1. **Feature breadth grows above the leaf kernel, not by muddying it.**
   `src/gen/module.rs` is the leaf-module kernel. Hierarchy should land
   as a higher layer (planned `src/gen/hierarchy.rs`), not as ad hoc
   special cases in the leaf path. Likewise, memories/FSMs/aggregates
   should become first-class motifs or module-level generators, not
   emitter tricks.
2. **`NodeId`-as-identity must keep expanding through the IR, not via
   emitter magic.** Today's live coverage is normalized combinational
   identity plus a conservative endpoint-preserving state merge.
   Future work is stronger state identity across richer state graphs and
   later hierarchical/block identity, but it must stay faithful to the
   doctrine: same identity requires proven same functionality with
   respect to the same canonical leaf variables. Keep
   `--identity-mode` as the coarse on/off switch and
   `--factorization-level` as the finer dial; construction strategy
   must stay orthogonal.
3. **Tool cleanliness must be industrialized.** Seed 42 being clean is
   good news, not a stopping point. Each new motif/category/knob needs
   matrixed Verilator/Yosys evidence, retained seed+config
   counterexamples, and root-cause fixes at the IR/generator layer
   rather than warning suppressions.
4. **Structure-first doctrine remains load-bearing.** Absent a
   specification, whole-module functional intent is not the optimization
   target. Invest in legal interaction surfaces, factorization
   pressure, hierarchy, and stateful richness. Functionally correct
   local blocks are welcome; a bundled whole-module oracle is not the
   direction.

### Endpoint-preserving functional doctrine for state identity (2026-04-20)

The user clarified the intended meaning of state equality sharply:

- two fanin cones may **not** share one `NodeId` if they do not have the
  same leaf endpoints as variables;
- the relevant variables are the canonical leaf endpoints: primary
  inputs and/or flop `Q` outputs; and
- the goal is equality by proven same functionality with respect to
  those same endpoints, not equality by visual resemblance or by
  matching graph skeleton alone.

Operational consequence:

- `merge_equivalent_flops` now uses a conservative leaf-aware proof form
  over the already-normalized IR rather than exact `d: NodeId`;
- that proof form now includes a bounded semantic check for
  small-support cones, so some different-shape cones can merge when
  they evaluate identically over the same canonical endpoint set; and
- any future strengthening of sequential identity must preserve the
  canonical leaf namespace. "Rename each owning `q` to SELF" is **not**
  acceptable in strict `NodeId as identity` mode, and neither is
  equating cones solely because they happen to look structurally alike.

---

## Generation-time defects observed in sample output (pending fixes)

Cataloguing real defects observed in sample module `mod_1_0000`
(3 outputs, 10-level fanin, default knobs, graph-first strategy).
These are generator bugs — not SV-emitter or validator bugs.
Enumerated here so the next session can fix them at the root.

- **Constant-select muxes.** Every `wN = (2'h2 == 2'hK) ? ... : ...`
  in the sample is a mux whose select is a *literal* comparison of
  two literals. The select folds at elaboration. Root cause: the
  encoded-mux assembler feeds the select-side recursion through
  the same `pick_terminal` path that can terminate on a constant
  leaf, and the one-hot-mux assembler similarly accepts a constant
  for the per-arm select bit. Fix: in mux-select position, forbid
  constant termination — require a non-constant signal source.
- **N-arity self-cancellation.** `w_21 = i_2 ^ i_2 ^ i_2 ^ i_2 = 0`.
  The N-arity operator expansion re-picks the same pool entry for
  every operand, and `Xor` of even repetitions is zero. Fix: the
  anti-collapse check must look at operand *multiset equality* for
  idempotent / self-inverse operators, not just dep-set
  non-emptiness. (And for `And`/`Or` the same issue produces
  `x & x & x = x` which is a structural collapse, not a zero, but
  still a motif violation.)
- **Coefficient width overflow.** `1'h6` appears — a 6 encoded in a
  1-bit literal, which truncates to 0. Root cause: the linear-
  combination coefficient generator picks the coefficient value
  independently of the operand width. Fix: clamp the coefficient to
  `bits ≤ operand_width`, or widen the literal to the operand width
  and let the top bits be real.
- **Dead wires.** `w_17`, `w_26`, `w_27`, `w_29` are declared and
  assigned but never read. Graph-first speculative pool growth is
  the source; Rule 18 (proposed) addresses this.
- **Stranded flop.** `r_3 <= r_3` — a flop whose D is its own Q and
  whose Q is never read. A no-op. Rule 18 covers this too, as long
  as "consumer" is defined to exclude the flop's own Q feedback.
- **Structurally-identical one-hot arms.** `w_8`, `w_10`, `w_12`,
  `w_14` are all `{w_6,...} & w_5`, meaning four arms of the one-
  hot mux have the same per-arm product. OR-reducing identical
  arms collapses to just the arm value. Fix: in one-hot assembly,
  require per-arm *data* distinctness (or require the per-arm
  select to differ; the current issue is that all arms share the
  same broadcast select bit `w_6`).

All six share a theme the user articulated: signals are being
created without a *reason to exist*. The fixes are three-category:
(1) tighten anti-collapse (operand-multiset check); (2) position-
dependent leaf rules (no const in mux select); (3) width-aware
constant generation. Rule 18 addresses the orthogonal
"unconsumed output" axis.

---

## File-level conventions

- Every Rust source file starts with a doc comment explaining its scope.
- Public types in `ir/types.rs` and `config.rs` get full doc comments. Internal helpers do not need them.
- No multi-paragraph docstrings. One short line; if more is needed, link to `book/`.
- No comments explaining *what* the code does; only *why* when non-obvious.

---

## Construction-time CSE via `Module::intern_gate` (2026-04-15 → 2026-04-16)

Design decision: *all* `Node::Gate` and `Node::Constant` creation is routed through two inherent methods on `Module`:

```rust
pub fn intern_gate(&mut self, op, operands, width, deps) -> (NodeId, bool);
pub fn intern_constant(&mut self, width, value) -> (NodeId, bool);
```

The boolean return is `is_new`: callers that also maintain a `SignalPool` must call `pool.add` only when `is_new` is true, otherwise the pool accumulates duplicate entries for deduped nodes.

Rationale: we need CSE at *construction* time, not as a post-pass. Rule 21 ("AST-instance cap") uses the dedup tables on `Module` as the single source of truth for "which NodeIds represent which expressions."

Rejected alternative: decouple the dedup table from `Module`, keep it in the generator. Rejected because the dedup is an IR-level invariant — the emitter and validator may also want to reason about it, and the tables must survive a `Module::clone()`.

### Snapshot contract with `build_cone_with_retry`

`build_cone_with_retry` rewinds state on empty-dep retries. Before the snapshot fix, it rolled back `m.nodes.truncate(snap_len)` but *not* `gate_instances` / `const_instances`. Stale entries then pointed at truncated `NodeId`s; subsequent intern calls would return a different node than the key promised (witnessed by `const_comparand_across_all_strategies_is_valid` failing at seed 2 Interleaved during the migration).

Fix: snapshot and restore `gate_instances` and `const_instances` alongside `m.nodes`, `m.flops`, pool, and worklist. The `HashMap::clone` cost is bounded by module size — measured negligible on the default knob range.

## Rule 18 "No orphan gates": α construction-time (2026-04-16)

Two enforcement paths were considered:

- **(α) Construction-time:** only create a gate when a specific consumer is already waiting for it. `build_cone` snapshots state before operand construction; on anti-collapse rejection, the snapshot is restored — operand sub-trees vanish from the IR. `process_signal_frame` (interleaved) can't snapshot per-gate because sibling frames have committed, so it delivers one of the existing operand NodeIds as the fallback instead of calling `pick_terminal` (which would create a fresh orphan-prone node).
- **(β) Emission-time tree-shake:** post-generation, compute the live set from drive-roots + flop D/Q transitive fanin, emit only that set.

Rejected β: it's a generate-then-filter step, violating the "by construction" doctrine. User-memory feedback: *"Rule-based generation, not post-hoc filtering."* α is adopted.

Corollary: GraphFirst retired. Its phase-1 speculative pool growth produced 13–27 % orphan gates per module. The variant is kept as a silent CLI alias for Interleaved for backward compat; the dedicated code path (`build_graph_first`, `grow_pool_one_unit`, `*_pool_only` helpers) is unreachable at runtime and may be removed in a future cleanup slice.

## Full factorization doctrine (2026-04-16)

User framing: **`NodeId` is the identity of an expression**; two expressions that are the same mathematically must share one NodeId, different expressions must have different NodeIds.

Implementation ladder (see `book/src/structural-rules.md` Rule 21c):

1. Syntactic CSE (Rule 21) — `(op, operands, width)` key. **Implemented.**
2. Operand-uniqueness (Rule 8 extended) — no NodeId twice in one operand list. **Implemented.**
3. Commutative normalization (Rule 21b) — sort commutative operands before interning. **Implemented.**
4. Associative flattening — flatten `(a+b)+c` to `Add(a,b,c)` when semantically safe. **Implemented.**
5. Constant folding — `x+0 → x`, all-constant evaluation, etc. **Implemented.**
6. Peephole — local algebraic / structural rewrites. **Implemented.**
7. E-graph — full semantic equivalence. **Partially implemented.**
   Default user-requested level. Today's live fragment is still bounded:
   small-support combinational cones can merge post-construction when
   they are proven equivalent over the same canonical leaf endpoints.

`FactorizationLevel::effective()` clamps user requests down to the highest implemented layer so aspirational levels don't error. Today `e-graph` remains the strongest implemented rung, but only as a bounded fragment rather than the full semantic-equivalence aspiration. Construction strategy is orthogonal: `sequential` / `shuffled` / `interleaved` decide build order, while the factorization ladder records how much of the `node-id` identity contract the current build can currently enforce/prove.

## Identity mode is orthogonal to construction strategy (2026-04-20)

User clarification that should remain durable:
**"NodeId as identity" is a mode of operation, not a cone-builder.**

That means:
- `construction_strategy` answers *how fanin cones are walked/built*
  (`sequential`, `shuffled`, `interleaved`, graph-first alias);
- factorization / identity mode answers *when two built objects are
  considered the same thing* and therefore must share one NodeId.

Implementation consequence: expose the peak-sharing / no-sharing
switch as a separate CLI axis (`--full-factorization`,
`--no-full-factorization`) rather than pretending it is another
construction strategy value. Future work on the true NodeId-as-
identity engine must preserve this separation.

## Identity mode is now a first-class typed axis (2026-04-20)

The separation above now lives in the code, not just in the docs:

- `Config` owns a new `IdentityMode` enum with `node-id`
  (default) and `relaxed`.
- `Module` mirrors both `identity_mode` and the requested
  `factorization_level`.
- The actual gating sites consult
  `effective_factorization_level()` instead of reading the raw
  ladder directly.

Design consequence:
- `identity_mode = relaxed` is the coarse hard-off switch. It
  forces the effective level to `none`, so `intern_gate` and
  `intern_constant` always allocate fresh NodeIds.
- `identity_mode = node-id` selects the full-factorization doctrine:
  `NodeId` is the identity of an expression.
- `factorization_level` is then the fine-grained implementation /
  proof-depth selector inside that doctrine. Lower rungs are useful
  diagnostic and stress modes, but they are not alternate semantics for
  `node-id`.

This is the minimum architectural move that makes the future
"NodeId as identity" engine honest: the repo can now talk about
identity mode without smuggling it through the ladder alone.

## Adversarial generation must be modeled as orthogonal axes (2026-04-20)

User clarification that should remain durable:
ANVIL must model all axes of adversarial generation explicitly and use
them efficiently during actual generation; there should be no hidden
bias toward whichever path the current implementation happens to favor.

Practically that means:
- construction strategy (`sequential`, `shuffled`, `interleaved`,
  graph-first alias) is one axis;
- identity mode (`node-id` vs `relaxed`) is another;
- factorization level is a third;
- motif/category weights, sequential density, width/depth ranges, and
  the probability knobs are additional orthogonal axes.

Implementation consequence: whenever a new generator feature lands, the
question is not only "does it work?" but also "which axis did it add,
how is that axis surfaced, how is it measured, and how do we avoid
silently under-sampling it during real workloads?"

## Stateful identity must be decided post-drain (2026-04-20)

For gates and constants, identity is knowable at intern time: the
full key exists when `intern_gate` / `intern_constant` runs.

Flops are different. `build_flop_leaf` allocates a Q leaf
immediately, but the flop's semantics are not complete until the
worklist later constructs its D-cone. So the first honest stateful
extension of "NodeId as identity" cannot be an allocation-time guess;
it has to run after drain.

Current rule: after `summarize_flop_mux_metadata`, flops are merged
iff they have the same emitted-state signature over the same canonical
leaf variables: same `width`, `reset_kind`, `reset_val`, and the same
leaf-aware D-cone proof form. Today that proof form has two rungs:

1. normalized structural proof over the already-canonicalized IR; and
2. bounded semantic proof for small-support cones (enumerate every
   endpoint assignment, key by the resulting truth table).

Construction provenance (`FlopKind`, cleared mux operand metadata) is
deliberately ignored once D exists, because emitted hardware semantics
are carried by width/reset/D-cone meaning, not by how the generator
happened to assemble them.

This is intentionally narrower than full sequential equivalence. Two
cones that happen to compute the same function but are not reduced to
the same proof form by the current ladder, or whose endpoint support is
too large for the bounded semantic check, are not merged yet. That
deeper coinductive story remains a
future slice.

## Bounded E-graph fragment for combinational identity (2026-04-20)

`merge_equivalent_gates(&mut Module)` is now the first live
post-construction combinational extension of the `e-graph` rung.

Current rule:
- gated by `identity_mode = node-id`;
- gated by effective factorization level `>= e-graph`;
- same canonical leaf endpoints are mandatory; and
- functionality may be proven either by the already-normalized
  structural proof form or by a bounded semantic truth table for
  small-support cones.

This is deliberately not the whole e-graph story. It is a bounded proof
fragment that makes the strongest mode honest today while preserving the
user's doctrine that `relaxed` remains a real no-sharing mode and that
construction strategy stays a separate axis.

## Emitter is a dumb serialiser (2026-04-16)

User-memory feedback: *"All thinking, checks, rules' enforcement ought to be done solely at the IR level. By the time you reach emission it is too late to roll back."*

Consequence: `emit::to_sv` iterates `m.nodes` in order and writes. No filtering, no reachability check, no live-set computation. Any invariant worth enforcing must be enforced at IR construction or at a `generate_leaf_module` finalization step — never at the emitter.

The safety-net audit in `generate_leaf_module` (`count_orphan_gates`) is *at the IR level* and warns on Rule 18 violations; it does not modify the IR. The emitter trusts what it is given.

## Rejected: without-replacement operand picking as the default

For And/Or/Xor/Add/Mul operand lists, operand duplicates are caught by `violates_anti_collapse` after operands are picked. A natural alternative is to pick operands *without replacement* at the source — maintain a `HashSet<NodeId>` during the per-operand loop and exclude already-picked NodeIds.

Considered and not adopted as the default because:
1. Pool sizes at default knobs are often ≤ N (the requested arity). Without-replacement falls back to "partial arity" + distribution shift.
2. Anti-collapse + rollback already gives 0 duplicates at default. The without-replacement change would save RNG cycles at the cost of a distribution shift that has no empirically measured benefit.
3. `operand_duplication_rate` is the documented knob for users who want the alternative behaviour.

Retained for reference in case a future motif benefits from it.

## Finalisation trims metadata-only and unused-bit surface (2026-04-19)

This slice locked in a small but important finalisation doctrine:
**emit what the live hardware uses, not the generator's provisional
scratch structure.**

- **Width adapters now expand to the exact target width.** The old
  non-multiple up-width adapter built an oversized replicated `Concat`
  and then sliced it back down. Functionally fine, but it manufactured
  dead high bits that lint tools quite rightly flagged. The adapter now
  builds the exact-width shape directly (`{src[rem-1:0], src, ...}`).
- **`Flop.mux` operand NodeIds are construction-time metadata, not
  emitted hardware roots.** Once `flop.d` is assembled, keeping the
  original select/data operand references around lets metadata-only
  cones survive liveness/compaction even though the emitter never reads
  them. Finalisation now keeps only the variant shape and discards
  those operand references before compaction.
- **Primary inputs are shrunk/pruned to the live bit surface.** After
  compaction, each surviving primary input is reduced to the highest bit
  any live consumer touches, and entirely unused data inputs are
  dropped from the emitted interface. This keeps Verilator from
  reporting unused input bits or dead ports.
- **Residual associative-opportunity metrics now respect duplicate
  policy.** Nested `Add`/`Mul` slots that would introduce duplicates if
  flattened are intentionally preserved at strict
  `operand_duplication_rate`; the metric now matches that semantic
  policy instead of counting those slots as "missed" flattening.

Rejected alternative: paper over the issue in the emitter with
tool-specific lint pragmas. That would hide the symptom without fixing
the IR/finalisation mismatch.

## Yosys ABC/no-ABC is now an explicit harness axis (2026-04-21)

Historically, the repo-owned Yosys smoke path settled on
`synth -noabc` because some runs with the default ABC-enabled `synth`
were reported to blow up or time out. That was useful operationally,
but it left the distinction implicit: future sessions could see one
hardcoded `-noabc` script and have no way to tell whether it was a
deliberate stability baseline, a temporary workaround, or stale cargo
cult.

`tool_matrix` now makes that choice explicit with a Yosys mode axis:

- `without-abc` — current stable baseline, still the default;
- `with-abc` — the repo-owned ABC-enabled harness path; and
- `both` — run both sub-modes per generated file and report them
  separately.

The default remains `without-abc` because that is the last known-good
repo-owned baseline. The point of adding `with-abc` and `both` is not
to silently relax warnings; it is to make the instability visible and
reproducible.

On the first small repo-owned probe, `without-abc` passed 15/15 while
the original `with-abc` path failed 14/15, not from a crash but from
ABC's `Warning: The network is combinational` line. Yosys's own `help abc`
text explains why that can happen even on sequential modules: ABC is
run on logic snippets extracted from the design, not necessarily on the
whole module as one sequential network.

The repo-owned harness now treats `with-abc` as the explicit
warning-clean script:

`synth -noabc; abc -fast; opt -fast; stat; check`

That keeps ABC in the loop while avoiding the default `scorr`-based ABC
script that was producing the non-actionable warning bucket. The
follow-up small `--yosys-mode both` probe is now clean in both
sub-modes: `without-abc = 15/15 pass`, `with-abc = 15/15 pass`.
