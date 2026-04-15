# Changes
Fully detailed change history. Newest entries at the top. One entry per commit.

---

## 2026-04-15-0020 — Construction-strategies chapter: 4 named strategies, graph-first planned default

**What changed**
- **NEW `book/src/construction-strategies.md`**. Dedicated chapter under "How It Works" documenting four named strategies for module construction:
  - **`sequential`** — current behavior: per-output cone recursion in declaration order. Baseline; has declaration-order bias and within-module ordering asymmetry.
  - **`shuffled`** — per-output cone recursion in a random permutation of declaration order. Removes declaration-order bias; within-module asymmetry randomized per seed.
  - **`interleaved`** — frames from all cones interleaved via a random-pop work queue; cones grow in lockstep. Near-symmetric within-module sharing.
  - **`graph-first`** — no per-output cone recursion at all. Grow a gate pool with no output attribution; pick drive-roots from the pool at the end. True symmetric sharing. **Planned default** once implementation lands.
  Chapter covers: why this is a knob (it shapes the output distribution), per-strategy complexity and tradeoffs, a comparison table, rule-interaction summary (Rules 1, 9, 16 all preserved across strategies), and implementation status.
- `book/src/SUMMARY.md`: new chapter added under "How It Works" after `algorithm.md`.
- `book/src/algorithm.md`: strategy note near the top referencing the new chapter so readers know the pseudocode describes `sequential` specifically.
- `book/src/sharing.md`: cross-output sharing section updated to call out the sequential-order asymmetry as a construction artifact and point to the new chapter.
- `MEMORY.md`: next-up list reorganized. Construction-strategies machinery is now item 1 (land the knob and implement sequencing); the motif slices (coefficients / shift-amount bias / comparands) follow. Recent-commits list gains `126411d`.
- `DEVELOPMENT_NOTES.md`: new core design decision entry "Construction strategies" pointing to the book chapter. Captures the load-bearing framing: strategy is how-we-build, not what-we-emit; each strategy has its own output distribution properties.

**Why**
User flagged that declaration-order asymmetry is a construction artifact, not a design property, and asked for true symmetric sharing. The discussion surfaced three alternatives (shuffled / interleaved / graph-first). User then noted the current behavior deserves a name too — hence four strategies, not three.

The chapter codifies all four as a first-class design choice: what strategy the generator uses is a *per-run knob*, not a hidden implementation detail. Users who want reproducibility of prior outputs pin to `sequential`; users who want maximum realistic sharing use `graph-first` (the planned default). The knob stays unimplemented until the machinery lands, but the doctrine is now fixed.

User's choice of `graph-first` as the default is aligned with the project's overall framing (think in terms of the object — a DAG — not the construction order). `sequential` and `shuffled` keep a per-output-cone construction idiom that is a human-friendly fiction; `graph-first` drops the fiction in favor of the DAG.

**Validation**
- Documentation-only slice; no source touched.
- `mdbook build book` succeeds with the new chapter rendered.
- `cargo check`, `cargo test` (27 tests), `cargo clippy --all-targets -- -D warnings`, `cargo fmt --all --check`: all still clean.

**Impact**
- The book now documents an explicit four-way knob that governs a major axis of generator behavior. Future sessions have clear guidance on the implementation sequence and which strategy becomes default.
- The cone-per-output construction idiom remains valid for `sequential`/`shuffled`/`interleaved` but is explicitly retrospective (not construction-time) for `graph-first`. This is doctrine now, not just my preference.

**Files touched**
`book/src/construction-strategies.md` (new), `book/src/SUMMARY.md`, `book/src/algorithm.md`, `book/src/sharing.md`, `MEMORY.md`, `DEVELOPMENT_NOTES.md`, `CHANGES.md`.

**Commit hash:** _to be filled in after this commit_

---

## 2026-04-15-0019 — Rule 16: cross-output sharing via the module-wide signal pool

**Commit hash:** `126411d`

**What changed**
- `book/src/structural-rules.md`: new Rule 16 "Cross-output sharing via the module-wide signal pool". States that there is no per-output isolation — gates built while constructing output A's cone are immediately available as leaves / DAG-sharing candidates in output B's cone and in every flop's D-cone. Calls out the ordering asymmetry (outputs built in declaration order; later outputs see more sharing candidates) and the combinational-no-loop preservation (Rule 1 holds cross-cone because arena-index monotonicity is module-wide, not per cone).
- "Operators vs blocks" preamble's grouping list updated with a "Module-wide sharing: Rule 16" entry.
- `book/src/sharing.md`: new "Cross-output and cross-cone sharing" section that names the behavior and points to Rule 16.

**Why**
User flagged: "Nodes inside the fanin cone of one top level output can be used as inputs of gates/blocks in the fanin cone of another top level output. I guess you are already allowing that." The behavior was already in place (the `SignalPool` is constructed once per module and shared across all cone builds), but it was implicit — a reader would have to infer it from the code rather than find it in the rule catalog. Making it Rule 16 closes the gap.

The ordering asymmetry (output 0 sees fewer candidates than output N-1) is worth documenting explicitly so a reader isn't surprised when output 0 tends to have more standalone logic than later outputs.

**Validation**
- Documentation-only slice; no source touched.
- `cargo check`, `cargo test` (27 tests), `cargo clippy --all-targets -- -D warnings`, `cargo fmt --all --check`: all still clean.
- Behavior claim verified against code: `src/gen/module.rs::generate_leaf_module` constructs exactly one `SignalPool` and threads it by `&mut` through every `build_cone_with_retry` call; `src/gen/cone.rs::pick_terminal` and `try_share` iterate the pool with no cone-identity filter.

**Impact**
- The structural rules catalog is more complete. A reader coming cold can now see explicitly that the generator does not isolate output cones from each other.
- The book's sharing chapter now points to Rule 16 for the authoritative statement.

**Files touched**
`book/src/structural-rules.md`, `book/src/sharing.md`, `MEMORY.md`, `CHANGES.md`.

---

## 2026-04-15-0018 — Log the constants-roles clarification in the book + two corrections

**Commit hash:** `8ff1d84`

**What changed**
- `book/src/structural-rules.md`: added a new "Roles of constants in RTL" section to the preamble (right after "Operators vs blocks"). Three distinct roles — coefficient, shift amount, comparand — each with its own scope, constraints, and motif family. Explicitly lists why flattening them into a single mechanism would break the semantic structure.
- Within that new section, two corrections the user surfaced:
  - **Shifts:** both variable-amount (`a << count` with `count` a signal) and constant-amount (`a << 2`) are legal SV. `anvil` today always emits variable-amount; real designs overwhelmingly use constant. A bias knob is on the roadmap so defaults match prevalence. Both modes coexist.
  - **Comparisons:** the RHS of a comparison can be *another signal* (signal-vs-signal, the default today) OR a *constant comparand* (threshold/sentinel pattern). The comparand motif is *additive* — it does not replace signal-vs-signal comparisons.
- `MEMORY.md` next-up list rewritten to reflect both corrections precisely:
  - Shift-motif next-up is now framed as a constant-vs-variable bias (not "replace variable with constant").
  - Comparison-motif next-up is now framed as an additive constant-comparand option alongside the existing signal-vs-signal default.
- `DEVELOPMENT_NOTES.md`: added "Roles of constants in RTL" to the core design decisions recap, pointing to the new book section.

**Why**
The user asked that the coefficient/shift-amount/comparand clarification be logged in the book, not just in the CHANGES / MEMORY ledgers. They also caught two follow-on imprecisions in my prior framing:

1. I had implicitly suggested shifts should switch from variable-amount to constant-amount. The user correctly pointed out that we can (and do) do `a << b` with `b` a signal, and the question is bias — both modes have a place.
2. I had implicitly suggested all comparands are constants. The user correctly pointed out that the RHS of a comparison can be (and routinely is) another signal.

Both corrections are now in the doctrine alongside the original distinction. Future implementation of these motifs will follow the corrected framing.

**Validation**
- Documentation-only slice; no source touched.
- `cargo check`, `cargo test` (27 tests), `cargo clippy --all-targets -- -D warnings`, `cargo fmt --all --check`: all still clean.

**Impact**
- The book's `structural-rules.md` is now the durable reference for the three constant roles. Short-form docs point to it.
- A session recovering cold from `git log + live docs` has precise, corrected guidance for the next three motif slices (coefficients, shift-amount bias, constant comparands).

**Files touched**
`book/src/structural-rules.md`, `MEMORY.md`, `DEVELOPMENT_NOTES.md`, `CHANGES.md`.

---

## 2026-04-15-0017 — Doctrinal fix: coefficient / shift amount / comparand are distinct motifs

**Commit hash:** `dde27a2`

**What changed**
- `MEMORY.md` next-up list split the prior lumped "coefficient as general arithmetic motif" entry into three distinct motif families:
  1. **Coefficients** — multiplicative weights in arithmetic linear combinations (Add/Sub/Mul). `ci ≠ 0` for Add. Knob family `coefficient_*`.
  2. **Shift amounts** — structural parameters of shift ops. Typical range `[0, W-1]`. Knob family `shift_amount_*`.
  3. **Comparands** — thresholds / sentinels for comparisons. No zero-exclusion. Knob family `comparand_*`.
- Added an explicit reminder that the three are semantically distinct and should not be collapsed into a single `constant_prob` knob.

**Why**
In the prior slice's next-up list I wrote "Generalize coefficient-as-arithmetic-motif to Sub/Mul/Shift/Compare". User (rightly) pushed back: coefficient is arithmetic vocabulary (a multiplicative weight in a linear combination). It is not the correct word for:
- Shift amounts (`a << 2`): the `2` is a structural parameter of the shift op, not a weight. Yes, `a << 2` is arithmetically `a * 4`, but in representation and synthesis cost they are distinct.
- Comparands (`a == 7`): the `7` is a threshold / sentinel / target value, not a weight.

Lumping all three under "coefficient" conflates three distinct motifs. The correction preserves the vocabulary discipline the project has been accumulating (operators vs blocks, arity vs ports, etc.).

**Validation**
- Documentation-only slice; no source touched.
- `cargo check`, `cargo test` (27 tests), `cargo clippy --all-targets -- -D warnings`, `cargo fmt --all --check`: all still clean.

**Impact**
- The next-up list now correctly decomposes the work into three separate motif families with their own knobs and constraints.
- A session that crashes between here and the first motif-family implementation recovers with accurate guidance rather than the lumped-and-wrong original.
- Vocabulary discipline accumulates: "coefficient" joins "arity" and "port" as terms with restricted, precise meaning.

**Files touched**
`MEMORY.md`, `CHANGES.md`.

---

## 2026-04-15-0016 — M-to-1 combinational mux as a first-class block

**Commit hash:** `0564a49`

**What changed**
- `src/config.rs`: two new knobs.
  - `comb_mux_prob` (default `0.1`): probability that a non-leaf
    recursion point becomes an M-to-1 combinational mux block
    instead of an operator gate. Flop block takes priority; comb-mux
    block takes priority over operator gate.
  - `comb_mux_encoding_prob` (default `0.5`): per-mux probability of
    the Encoded style (chained ternary over `Eq(sel, k)` with a
    `ceil(log2(M))`-bit select bus) vs the OneHot style (M 1-bit
    select signals, OR of masked arms).
  - Both threaded into `Overrides`, `apply_cli_overrides`, and the
    probability-range validation loop.
- `src/main.rs`: two new CLI flags `--comb-mux-prob` and
  `--comb-mux-encoding-prob`.
- `src/gen/cone.rs`:
  - `build_cone` adds a new branch between the flop branch and the
    operator gate branch: if `rand() < comb_mux_prob`, dispatch to
    `build_comb_mux`.
  - New `build_comb_mux` — picks M from `[max(2, min_mux_arms),
    max_mux_arms]` (M=0 and M=1 excluded: no sensible fall-back for
    stateless muxes, 1-arm mux is a wire), picks encoding style via
    `comb_mux_encoding_prob`, dispatches to the style-specific helper.
  - New `build_comb_mux_one_hot` — recursively builds M (data, sel)
    arms, then assembles `D = OR_i({W{sel_i}} & data_i)` using the
    same `replicate_to_width` / `make_and` / `or_reduce_terms`
    primitives as the flop D-mux one-hot path. No Q-feedback term.
  - New `build_comb_mux_encoded` — recursively builds one
    `ceil(log2(M))`-bit select sub-cone + M data sub-cones, then
    assembles a chained ternary via `make_eq_const` / `make_mux`
    with a zero fall-through.
  - New inline unit test `comb_mux_block_produces_valid_output`:
    10 seeds × 2 encoding styles = 20 modules, all pass IR
    validation with `comb_mux_prob = 1.0`.
- `book/src/structural-rules.md`:
  - New Rule 15 "M-to-1 combinational mux block" codifying both
    shapes, the M range, the "no Q-feedback axis" constraint, and
    the block-vs-operator framing (muxes have ports, not arity).
  - "Operators vs blocks" preamble updated: the future-placeholder
    entry for "Block: mux (combinational)" is replaced with a
    pointer to Rule 15.
- `book/src/knobs.md`: new "Combinational mux block" subsection
  documenting the two knobs with cross-references to Rule 15.
- `book/src/algorithm.md`: `build_cone` pseudocode gains the comb-mux
  branch in its correct dispatch position (after flop, before operator).
- `book/src/tutorial.md`: new Example 9 "Combinational M-to-1 mux
  block" with actual captured SV excerpt showing the chained-ternary
  form; Example 10 (was 9) "Mixing everything" follows.
- `book/src/recipes.md`: new entry "I want combinational muxes, not
  just flop D-muxes" with a tuned knob combo.
- `USER_GUIDE.md`: two new CLI flags added to the knob table.
- `CODEBASE_ANALYSIS.md`: module map for `cone.rs` updated to list
  the three new build_comb_mux helpers and the new dispatch branch
  in `build_cone`.
- `MEMORY.md` / `CHANGES.md`: per workflow.

**Why**
Per user direction: promote the M-to-1 mux to a first-class
combinational motif. Prior to this slice, M-to-1 muxes existed only
as compound gate trees buried inside flop D-input construction;
combinational logic could only emit 2:1 muxes via `GateOp::Mux`.
Real designs use M-to-1 muxes extensively in combinational datapaths
(selectors, bus steering, priority encoders). Making them a
first-class block motif closes a large expressiveness gap.

This slice is also a direct application of the operators-vs-blocks
doctrine established in the prior slice: Mux is a block, so its
generalization is a *structural* motif (port counts, encoding
style), not an arity bump. No new `GateOp` variant — the mux is a
compound gate tree, same as the flop D-mux.

**Validation**
- `cargo check --all-targets`, `cargo test` (25 unit + 2 integration =
  27 tests, was 26), `cargo clippy --all-targets -- -D warnings`,
  `cargo fmt --all --check`: all clean.
- End-to-end: `cargo run -- --comb-mux-prob 1.0
  --comb-mux-encoding-prob 0.0 ...` emits the one-hot OR-of-masks
  shape; with `--comb-mux-encoding-prob 1.0` the same knobs produce
  the chained-ternary shape with a `20'h0` fall-through (no
  Q-feedback).

**Impact**
- M-to-1 combinational muxes are now routinely emitted. Generated SV
  shape distribution is closer to real-world datapath idioms.
- Phase 2 still in progress; Verilator-lint smoke now needs to
  also cover `comb_mux_prob` settings as well as `share_prob` and
  the flop styles.
- The prior conceptual plan "land M-to-1 combinational mux block"
  from the previous slice's next-up list is complete.

**Files touched**
`src/config.rs`, `src/main.rs`, `src/gen/cone.rs`, `book/src/structural-rules.md`, `book/src/knobs.md`, `book/src/algorithm.md`, `book/src/tutorial.md`, `book/src/recipes.md`, `USER_GUIDE.md`, `CODEBASE_ANALYSIS.md`, `MEMORY.md`, `CHANGES.md`.

---

## 2026-04-15-0015 — N-arity for associative operators + operators-vs-blocks doctrine

**What changed**
- `src/config.rs`: new knobs `min_gate_arity` (default 2) and
  `max_gate_arity` (default 4). `Config::validate` enforces `min >= 2`
  and `max >= min`. New `ConfigError::GateArityRange`. Overrides and
  `apply_cli_overrides` updated. Comment on the knob explicitly states
  that arity applies to operators only (And/Or/Xor/Add/Mul), not to
  blocks; Sub is excluded because it is not associative.
- `src/main.rs`: new CLI flags `--min-gate-arity` and
  `--max-gate-arity`, threaded into `Overrides`.
- `src/gen/cone.rs`: `input_widths_for` now returns N-wide operand
  lists for `And`, `Or`, `Xor`, `Add`, `Mul` (N drawn from the new
  knob range). `Sub` remains strictly 2-arity (documented inline with
  the reason: subtraction is not associative, so N-arity chains
  `a - b - c` come from cascaded 2-arity nodes, not a single N-arity
  Sub). Added `use crate::config::Config` so `input_widths_for` can
  read the new range.
- `src/emit/sv.rs`: `render_gate` uses a `joined(sep)` helper to emit
  any-arity infix expressions for the associative ops (`a & b & c`,
  `a + b + c + d`, etc.). `Sub` retained as the explicit 2-operand
  form.
- `src/ir/validate.rs`: `check_gate_shape` accepts `operands.len() >= 2`
  for the associative ops, exactly 2 for `Sub`. Added 3 tests:
  - `accepts_nary_and_with_three_operands`
  - `rejects_and_with_fewer_than_two_operands`
  - `rejects_nary_add_operand_width_mismatch` (4-way Add with one
    mismatched-width operand)
- `src/ir/types.rs`: header doc comment updated; "operand arity"
  replaced with "operand count", plus a vocabulary-discipline note
  pointing to the book's operators-vs-blocks preamble.
- `book/src/structural-rules.md`:
  - New "Operators vs blocks" preamble up front. Explicit vocabulary
    discipline: *arity* is operator vocabulary only; *ports / arms /
    port count* is block vocabulary. Rules grouped by what they
    govern (combinational integrity / flop block / future mux block
    / correctness guarantees).
  - New Rule 14 "Operator N-arity for associative operators". States
    which ops are associative (And/Or/Xor/Add/Mul), which are not
    (Sub, comparisons, shifts), and why operator arity is a
    different kind of generalization than block port-counts.
  - Rule 10 width table updated: associative ops show `[W, W, ...] (N ≥ 2)`;
    Sub shown separately as strictly 2-arity.
  - Mux entry in the unary/special-arity list rewritten to state
    explicitly that Mux is a block with *ports*, not arity.
- `book/src/algorithm.md`: width-rules table matches the catalog.
  Added a sentence explaining that the associative operators draw
  arity from `cfg.min_gate_arity..=cfg.max_gate_arity`.
- `book/src/knobs.md`: new "Operator N-arity" subsection documenting
  the two knobs with the operators-only framing.
- `USER_GUIDE.md`: two new CLI flags in the knobs table.
- `DEVELOPMENT_NOTES.md`: new "Operators vs blocks" entry in the core
  design decisions recap. Points to the book preamble + Rule 14.
- `CODEBASE_ANALYSIS.md`: invariants list gains the operator N-arity
  entry with a cross-reference.
- `MEMORY.md` / `CHANGES.md`: per workflow. Next-up list re-prioritized
  to queue up the M-to-1 combinational mux block and the linear-
  combination ADD coefficient motif that the user introduced during
  this slice's discussion.

**Why**
Per user direction: let logic and arithmetic operators have random
arity N ≥ 2 so the generator emits `a & b & c`, `w + x + y + z`, etc.
Not just 2-input trees. This is straightforward for associative ops
— grouping doesn't matter algebraically — but doesn't apply to Sub,
which the user flagged mid-slice. Sub was removed from the associative
set accordingly.

The deeper outcome of this slice is the operators-vs-blocks doctrine
that the user made explicit during discussion. Arity is the correct
word for operators; blocks have ports / arms / port count. Conflating
the two obscures the fact that operator generalization (N-arity) and
block generalization (enumerating motif shapes) are fundamentally
different activities. The book's rule catalog now opens with that
distinction so future rules land in the right category.

**Validation**
- `cargo check --all-targets`, `cargo test` (24 unit + 2 integration =
  26 tests), `cargo clippy --all-targets -- -D warnings`,
  `cargo fmt --all --check`: all clean.
- End-to-end: `cargo run -- --seed 3 --max-depth 3 --max-inputs 3
  --max-outputs 1 --flop-prob 0 --share-prob 0 --min-gate-arity 3
  --max-gate-arity 4` produces assign statements like
  `w_4 = w_2 + w_3 + w_3 + w_3` and `w_5 = w_2 + w_3 + w_2 + w_4`,
  confirming N-arity in emitted SV.

**Impact**
- Generated RTL now exhibits N-arity associative operators — closer
  to typical hand-written logic and arithmetic shapes.
- The operators-vs-blocks doctrine is now load-bearing and feeds
  straight into the next two slices' scope.

**Files touched**
`src/config.rs`, `src/main.rs`, `src/gen/cone.rs`, `src/emit/sv.rs`, `src/ir/validate.rs`, `src/ir/types.rs`, `book/src/structural-rules.md`, `book/src/algorithm.md`, `book/src/knobs.md`, `USER_GUIDE.md`, `DEVELOPMENT_NOTES.md`, `CODEBASE_ANALYSIS.md`, `MEMORY.md`, `CHANGES.md`.

---

## 2026-04-15-0014 — Q-feedback rule relaxation + structural-rules catalog

**Commit hash:** `6cbcbff`

**What changed**
- **Rule change (code):** `src/gen/cone.rs` — three sites in
  `drain_flop_worklist`, `drain_flop_one_hot`, `drain_flop_encoded`
  now pass `exclude = None` to `build_cone_with_retry` for the D
  sub-cones. Previously they passed `Some(q_node)`, forbidding the
  flop's own Q from being a leaf in its data/select/direct-D
  sub-cones. Q-feedback through arbitrary combinational logic in the
  D-cone is now freely permitted. The clock edge breaks the loop
  temporally; this matches standard synchronous feedback patterns
  (counters, toggles, accumulators, state machines).
- **Combinational no-loop preserved:** Rule 1 — a combinational gate
  output cannot appear upstream in its own fanin cone — is
  unchanged. It is enforced by arena-index monotonicity (pool entries
  pre-date each recursion step), not by the `exclude` parameter.
- **New durable artifact:** `book/src/structural-rules.md`. A
  catalog of 13 load-bearing generator invariants, each stated with
  its rationale, its "enforced where" location, and cross-references
  to the relevant code. Expected to grow as new rules become
  invariants (Phase 3+ placeholders already listed).
- **`book/src/SUMMARY.md`:** new chapter added to *Correctness
  Guarantees* section between "Generation by Construction" and
  "Synthesizability".
- **`book/src/sequential.md`:** retired the "No Q→D feedback through
  the mux datapath" section. Replaced with "Q-feedback in the D-cone
  is freely permitted" pointing to Structural Rules Rules 2 and 3.
  Pseudocode updated to drop the `exclude=Q` parameter.
- **`DEVELOPMENT_NOTES.md`:** the old "Q-exclusion contract" core
  design decision replaced with "Q-feedback freedom (revised)" that
  references the new catalog. Added a "Structural rules catalog"
  core decision establishing the book chapter as the durable source
  of truth — recaps point to it, do not duplicate rule text.
- **`CODEBASE_ANALYSIS.md`:** the `drain_flop_worklist` bullet
  updated to reflect `exclude = None` and to point to Rules 2 and 3.
  Added a pointer stating the full invariant catalog lives in the
  book.

**Why**
Per user direction: "Flop's Q output may be loopback to any input
and any number of times to inputs in the flop's D fanin cone."
Combined with the pre-existing QFeedback mux term (orthogonal), this
makes every legal synchronous feedback pattern expressible. The
previous Q-exclusion contract was an over-constraint I had inferred
from an earlier, tighter phrasing; the user has since clarified that
Q-in-sub-cones is intended.

Separately, the user asked that these kinds of rules make their way
into the book and into live docs, with an accumulating catalog as
the project matures. The `structural-rules.md` chapter is that
catalog. It is now the canonical location for every load-bearing
invariant. Inline rule restatements in short-form docs should point
to the catalog, not duplicate it — duplication leads to drift.

**Validation**
- Q-in-sub-cone working end-to-end: at `--seed 2 --max-depth 3
  --max-inputs 2 --max-outputs 1 --flop-prob 1.0 --max-flops 1
  --min-mux-arms 2 --max-mux-arms 2 --flop-mux-encoding-prob 0.0
  --share-prob 0.5`, the emitted SV contains `assign w_4 = r_0 + r_0`
  — the flop's Q (`r_0`) appears twice in a gate in its own D cone.
- `cargo check --all-targets`, `cargo test` (23 tests), `cargo
  clippy --all-targets -- -D warnings`, `cargo fmt --all --check`:
  all clean.
- Integration sweep of 20 seeds still passes with the relaxed rule.

**Impact**
- Generated RTL now exhibits real synchronous feedback patterns
  (counters, accumulators, state-returning logic) rather than only
  pass-through or clean-data registers.
- The book gains a durable, growing catalog of structural rules that
  a future session can scan to understand every invariant without
  archaeologizing commits.
- Future rule additions have a natural home. No more inline
  restatement and drift.

**Files touched**
`src/gen/cone.rs`, `book/src/structural-rules.md` (new), `book/src/SUMMARY.md`, `book/src/sequential.md`, `DEVELOPMENT_NOTES.md`, `CODEBASE_ANALYSIS.md`, `MEMORY.md`, `CHANGES.md`.

---

## 2026-04-15-0013 — mdBook becomes user-facing: Getting Started, Tutorial, Recipes

**Commit hash:** `bac6060`

**What changed**
- **`book/src/getting-started.md`** (new): installation, first module (with full annotated SV output), reading the output line-by-line, reproducibility explanation, batch generation via `--out`, dumping effective knobs. Ends with a pointer to Tutorial / Recipes / Knobs / Core Idea.
- **`book/src/tutorial.md`** (new): 9 progressive examples, each with the exact command and an excerpt of the generated SV. Progression: minimal combinational → deeper cones → multi-output → flops with direct D (M=0) → one-hot mux on D → encoded-select mux on D → Q-feedback variant → DAG-shaped cones → everything mixed. Opens with a "logic is deliberately nonsensical, that's the point" disclaimer so users aren't confused when the first `a + a + a` appears.
- **`book/src/recipes.md`** (new): 9 "I want to do X" cookbook entries — minimal smoke-test corpus, fanout stress, flop-heavy, encoded-mux stress, one-hot-mux stress, narrow/wide-data stress, reproduce a module, parser-only stress, formal-equivalence sizing. Each recipe states the goal, gives the CLI command, explains which knobs matter.
- **`book/src/introduction.md`** (rewritten): now leads with what anvil is (not with the "problem" section) and who it's for. Adds a five-minute pitch (command + output). Describes what makes anvil different (vs grammar fuzzers vs hand-written suites). Ends with a "what you'll find in this book" outline and a clear invitation to jump to Getting Started.
- **`book/src/SUMMARY.md`** (restructured): five parts —
  - *Using anvil* (Getting Started, Tutorial, Recipes) — leads the book.
  - *How It Works* (Core Idea, Why Not a Grammar?, Algorithm, IR).
  - *Correctness Guarantees* (By Construction, Synthesizability, Non-Triviality).
  - *Motif Catalogue* (Sequential, Sharing, Hierarchy).
  - *Reference* (Knobs, Architecture, Non-Goals).
  Users arrive at the welcoming part first; contributors find design content in the middle; everyone finds reference material at the end.
- **`book/book.toml`**: removed obsolete `multilingual = false` field that mdbook 0.4.51 now rejects. Updated book title and description to reflect the book's dual user/design role.

**Why**
Per user direction: "the book is the user facing surface to the project... documentation is key to attract and retain users... top-notch and littered with examples with increasing complexity. We should not scare users."

Prior to this slice the book was correct and thorough but relentlessly design-focused. A user arriving at the book's first page would land on "The Core Idea" — a philosophical argument about circuit-graph IRs vs EBNF — before ever seeing a single command. That is backward for a tool that people need to actually run. This slice fixes the on-ramp.

The user-facing chapters are copy-pasteable, progress by one concept per example, and show real generated SV at each step (not hypothetical snippets). The SV fragments in Tutorial were captured from actual `cargo run --` invocations during authoring.

**Validation**
- `mdbook build book` succeeds and produces `book/book-out/` with all chapters rendered.
- All code gates remain clean (no source touched): `cargo check`, `cargo test` (23 tests), `cargo clippy -- -D warnings`, `cargo fmt --check`.
- Cross-read new chapters against the code (`src/main.rs` CLI flags, `src/config.rs` defaults, `src/gen/cone.rs` flop motifs) to verify every command in the Tutorial and every recipe in Recipes actually works with the currently-implemented flags.

**Impact**
- The book is now the intended first-stop for users, not just contributors.
- Every user-exposed feature (`CLI flags`, flop motifs, DAG sharing, reproducibility) has at least one worked example.
- Design chapters remain for anyone who wants them — just accessible via a clearly-labeled "How It Works" section rather than as the book's opening.

**Files touched**
`book/src/getting-started.md` (new), `book/src/tutorial.md` (new), `book/src/recipes.md` (new), `book/src/introduction.md` (rewritten), `book/src/SUMMARY.md` (restructured), `book/book.toml` (obsolete field removed), `MEMORY.md`, `CHANGES.md`.

---

## 2026-04-15-0012 — mdBook staleness refresh: knobs, IR, algorithm, architecture

**Commit hash:** `62fdeaa`

**What changed**
- `book/src/knobs.md`:
  - Rewrote the knob taxonomy into four categories: Structural, Sequential, Sharing, Mix/Termination, Hierarchy.
  - Added every sequential knob that was missing: `max_flops_per_module`, `min_mux_arms`, `max_mux_arms`, `flop_qfeedback_prob`, `flop_mux_encoding_prob`, `use_async_reset`.
  - Updated defaults block to match current `Config::default()` (was showing Phase-0 defaults like `flop_prob: 0.0`, `share_prob: 0.0`).
  - Added a "CLI coverage" section listing every flag so users know what's reachable without a config file.
- `book/src/ir.md`:
  - Added `FlopKind`, `FlopMux`, `MuxArm` types to the core-types block.
  - Updated `Flop` to include `kind` and `mux` fields.
  - Clarified `Module.nodes` is `Vec<Node>` (not `Arena<Node>`) with arena *semantics* via `NodeId` indexing.
  - Removed "Phase 2+" annotations on clock/reset/flops (now live).
  - Fixed "Phase 3 (sharing)" reference (now Phase 2 after renumber).
  - Rewrote the Invariants section to distinguish what's enforced by construction vs by the per-gate width validator in `ir::validate::validate`.
  - Added the validator's rich error categories (per-gate arity, operand widths, output widths).
  - Updated name-generation section to cover clk/rst_n.
- `book/src/algorithm.md`:
  - Module-level pseudocode now shows clk/rst_n port reservation and the exclusion of those ports from the signal pool.
  - Cone recursion pseudocode shows the DAG-sharing fork (`rand() < share_prob` → `try_share`) and the `exclude` parameter for Q-isolation.
  - New "Flop worklist drain" section covering M ∈ {0, 2..=max}, per-flop FlopKind choice, per-flop mux style choice (one-hot vs encoded), with cross-references to `sequential.md`.
  - Terminal selection pseudocode rewritten to match current behavior: prefer dep-bearing matching-width entries, fall back to any matching-width, then lazy width-adapter, then constant as last resort.
  - Width-rules table: added `Shl/Shr` row.
  - Anti-collapse section: clarified `NodeId` equality catches sharing-induced self-reference.
- `book/src/architecture.md`:
  - Crate-layout comments updated: `main.rs` notes CLI coverage; `validate.rs` notes unit tests; `cone.rs` notes DAG sharing and flop-mux assembly; removed placeholder `hierarchy.rs # Phase 5+` (not in source yet; will land in Phase 4).
  - Renumbered "Phase 5" references to Phase 4.
  - Key-types block updated with `FlopKind`, `FlopMux`, and the `kind`/`mux` fields on `Flop`.
  - Testing-strategy section replaced placeholder with concrete counts: 8 validator tests, 7 cone tests, 6 emitter tests, 2 integration = 23 total.
  - CLI section replaced "..." placeholder with the full flag surface and defaults.
- `MEMORY.md` and `CHANGES.md` updated per the mandatory pre-commit docs.

**Why**
The user flagged that several book chapters had gone stale relative to recent code slices. Specifically:
- `knobs.md` still showed `flop_prob: 0.0` and `share_prob: 0.0` as defaults, which is contradicted by `Config::default()` (0.15 and 0.3 respectively) and would mislead anyone reading the book to understand tunable ranges.
- `ir.md` did not document the new `FlopKind`, `FlopMux`, `MuxArm` types at all, and still described clock/reset/flops as "Phase 2+" aspirations rather than live features.
- `algorithm.md` showed an outdated pseudocode with `pick_node_kind(gate | flop | terminal)` and a `terminal_reuse_prob` / `constant_prob` coin-flip that doesn't match the current `pick_terminal` implementation.
- `architecture.md` referenced Phase 5 for hierarchy (now Phase 4 after the renumbering in commit `4317c82`), had a `...` placeholder in the CLI section, and listed no test counts.

This slice closes those gaps. The book's design chapters now match the code at commit `c9ec12c`.

**Validation**
- Documentation-only slice; no source changes.
- `cargo check`, `cargo test` (23 tests), `cargo clippy -- -D warnings`, `cargo fmt --check`: all still clean (no code touched).
- Cross-read each updated chapter against the corresponding source file to verify no dangling references to removed/renamed types.

**Impact**
- A contributor reading the book to understand anvil's IR or algorithm now gets a faithful current-state picture.
- The knob defaults in `knobs.md` match what `cargo run -- --dump-config` actually prints.
- Phase numbering is consistent across the book, `ROADMAP.md`, and `CODEBASE_ANALYSIS.md`.

**Follow-up (flagged in next-up)**
The user additionally asked that the book serve as the user-facing surface — with progressive examples and a welcoming on-ramp, not just design reference. The existing chapters are correct but contributor-oriented. A follow-up slice will add Getting Started, Tutorial (progressive examples), and Recipes chapters, and restructure `SUMMARY.md` to lead with user material.

**Files touched**
`book/src/knobs.md`, `book/src/ir.md`, `book/src/algorithm.md`, `book/src/architecture.md`, `MEMORY.md`, `CHANGES.md`.

---

## 2026-04-15-0011 — CLI coverage for all Phase 1/2 motif knobs

**Commit hash:** `c9ec12c`

**What changed**
- `src/main.rs`:
  - New CLI flags on `Cli`: `--max-flops-per-module`, `--min-mux-arms`, `--max-mux-arms`, `--flop-qfeedback-prob`, `--flop-mux-encoding-prob`.
  - `cli_overrides` function threads the new flags into `anvil::config::Overrides`.
- `src/config.rs`:
  - `Overrides` struct gains five new `Option<_>` fields matching the new CLI flags.
  - `Config::apply_cli_overrides` handles each new override.

**Why**
Every Phase 1/2 motif knob now has a dedicated CLI flag. Previously, exercising flop motifs required editing a JSON config file and passing `--config`, which is enough friction to discourage casual experimentation and to make CLI-based reproducibility less pleasant. After this slice, a user can force any combination — e.g., encoded-mux-only QFeedback flops with M ≤ 3 — in a single command line.

This is the "Consider adding a `--share-prob` CLI flag" item from the prior `MEMORY.md` next-up list, broadened to include all the other Phase 1/2 motif knobs that were similarly JSON-only.

**Validation**
- `cargo check --all-targets`, `cargo test` (23 tests), `cargo clippy --all-targets -- -D warnings`, `cargo fmt --all --check`: all clean.
- `cargo run -- --help` surfaces all five new flags with their expected names.
- End-to-end check: `cargo run -- --seed 1 --max-depth 2 --max-inputs 2 --flop-prob 1.0 --flop-mux-encoding-prob 0.0 --max-mux-arms 2` produces the one-hot replicate-AND pattern (confirming `--flop-mux-encoding-prob 0.0` is actually honored).

**Impact**
- Phase 1/2 motif exploration is now CLI-native.
- Removes one friction point before the Verilator-lint smoke run: that smoke run will ultimately need to sweep both `share_prob` and the flop encoding probability to satisfy Phase 2's exit criterion, and CLI-driven sweeps are far easier to script than JSON-config-driven ones.

**Files touched**
`src/main.rs`, `src/config.rs`, `MEMORY.md`, `CODEBASE_ANALYSIS.md`, `CHANGES.md`.

---

## 2026-04-15-0010 — Phase 2 start: per-operand DAG-cone sharing

**Commit hash:** `6ba646b`

**What changed**
- `src/gen/cone.rs`:
  - `build_cone` operand loop now consults `cfg.share_prob` per operand. With that probability it calls the new `try_share` helper; on `Some(node)` the operand terminates at that existing pool entry, on `None` it falls back to normal recursion.
  - New `try_share(g, pool, width, exclude)` helper: returns a random matching-width pool entry with non-empty deps, honoring the `exclude` filter used for flop Q-exclusion.
  - New unit test `share_prob_high_shares_internal_gates`: a 32-seed sweep at `share_prob=0.9` must produce at least one Gate (not just a primary input) with fanout ≥ 2. This verifies the non-leaf DAG mechanism actually fires and is not masked by leaf-level reuse.
- `src/config.rs`: `share_prob` default raised from `0.0` to `0.3`, making DAG-ish cones the generator's default shape.
- `book/src/sharing.md` rewritten:
  - States that tree-and-DAG is a per-operand decision, not a global mode. The generator mixes both freely.
  - Explains the distinction between leaf-level reuse (always on) and non-leaf sharing (controlled by `share_prob`).
  - Includes the `try_share`/`build_cone` pseudocode.
  - Documents the anti-collapse guards still applying post-share.
- `ROADMAP.md`: Phase 2 status flipped to `in progress`. Exit criterion extended to cover Verilator-lint on `share_prob ∈ {0.0, 0.3, 0.9}`.
- `USER_GUIDE.md`: `--share-prob` default updated to 0.3; description rewritten as per-operand probability.
- `CODEBASE_ANALYSIS.md`:
  - Module map for `cone.rs` gains `try_share` and the DAG-sharing summary.
  - Phase coverage map: Phase 2 now `in progress`.
  - Invariants-enforced list gains the `share_prob` / `try_share` entry.
  - Testing surface: 7 cone unit tests (was 6), total 23 (was 22).
- `DEVELOPMENT_NOTES.md`: calibration section gains a `share_prob = 0.3` entry explaining the default and clarifying that `share_prob = 0.0` is not pure tree (leaf-level reuse via `pick_terminal` is always on).
- `MEMORY.md`: Current state, next-up, recent commits, known-gaps all refreshed.

**Why**
Phase 2 per user direction: enable DAG cones. User framing: "tree or DAG, randomly picked per recursion point" — exactly what a per-operand `share_prob` coin gives. For this slice we set `share_prob = 0.3` as the default so the generator produces DAG-shaped cones by default; users who want pure-tree or maximally-shared modes set `share_prob` explicitly to 0.0 or ~1.0.

The mechanism is intentionally minimal: two lines in `build_cone` plus one helper. The pool already contained every `Gate` node on creation from Phase 1 work, so the infrastructure was in place; what was missing was the non-leaf hook to consult it.

**Validation**
- `cargo check --all-targets`: clean.
- `cargo test`: 21 unit + 2 integration = 23 tests, all pass.
- `cargo clippy --all-targets -- -D warnings`: clean.
- `cargo fmt --all --check`: clean.
- Pipeline sweep of 20 seeds passes with DAG-sharing on by default — no multi-driver violations, no IR-validation failures, no empty dep-sets. The lazy-adapter path continues to operate when widths don't match any pool entry.
- New `share_prob_high_shares_internal_gates` unit test passes.

**Impact**
- Generated SV now routinely has internal gate fanout > 1: one wire drives multiple consumers. This is the first motif-diversity step that makes `anvil` output resemble real hand-written RTL rather than pure random trees.
- Phase 2 exit gate is now Verilator-lint on representative `share_prob` values, identical in form to the Phase 1 Verilator gate — both block on tooling availability.
- The `share_prob = 0.0` → pure tree framing in `book/src/sharing.md` is corrected: pure tree is impossible because leaf-level reuse is always on. The book now reflects that nuance.

**Files touched**
`src/gen/cone.rs`, `src/config.rs`, `book/src/sharing.md`, `ROADMAP.md`, `USER_GUIDE.md`, `CODEBASE_ANALYSIS.md`, `DEVELOPMENT_NOTES.md`, `MEMORY.md`, `CHANGES.md`.

---

## 2026-04-15-0009 — Inline unit tests for cone helpers and SV emitter

**Commit hash:** `c8043c3`

**What changed**
- `src/gen/cone.rs`: added `#[cfg(test)] mod tests` with 6 tests:
  - `ceil_log2_expected_values` — hand-picked values plus a 62-value sweep asserting the `2^ceil_log2(n) >= n` invariant.
  - `pick_mux_arm_count_never_returns_one` — 10K draws confirming the `M ∈ {0, 2..=max}` discipline is structurally enforced, not accidentally.
  - `width_adapter_identity` — passthrough when src == target, no IR nodes added.
  - `width_adapter_slice_shrinks` — src > target emits a `Slice{hi: target-1, lo: 0}` with correct operand.
  - `width_adapter_concat_expands_exact_multiple` — src < target and src divides target emits a single Concat with the right number of copies.
  - `width_adapter_concat_expands_non_multiple` — src < target and non-multiple emits Concat + Slice; outer node is a Slice of target width; a 9-bit Concat exists as its source (example: 3-bit src, 8-bit target, copies = 3, concat_width = 9, slice to 8).
- `src/emit/sv.rs`: added `#[cfg(test)] mod tests` with 6 tests on hand-built IRs:
  - `emits_module_header_and_endmodule` — module declaration shape + port typing + passthrough assign.
  - `omits_clk_rst_n_when_no_flops` — even when `Module.clock` and `Module.reset` are set, clk/rst_n are absent from the port list if `m.flops.is_empty()`.
  - `emits_always_ff_with_single_clk_and_async_rst_n` — canonical `always_ff @(posedge clk or negedge rst_n)` header, `if (!rst_n)` active-low reset branch, `r_0 <= 4'h0;` reset value, `r_0 <= a;` clocked assignment, output wired to Q.
  - `constant_and_operators_rendered` — `{W}'h{hex}` constant form, `a & b` for And, `w_3 ^ 8'h5a` for Xor with a constant operand.
  - `slice_and_concat_rendered` — `a[3:0]` for Slice, `{a, a}` for a 2-copy Concat.
  - `mux_rendered_with_ternary` — `(s) ? (a) : (b)` for Mux.
- `CODEBASE_ANALYSIS.md`: "Testing surface" section now enumerates all three inline test modules with counts; total is 22 tests.
- `MEMORY.md`: Current state, next-up, and recent commits refreshed. Phase 1's remaining exit gate is now just the Verilator-lint smoke run.

**Why**
The validator landed in the previous slice plus the 22-seed integration sweep cover "does the output validate?" — but the individual helpers (`make_width_adapter`, `ceil_log2`, `pick_mux_arm_count`) and the emitter's per-form rendering had no direct pin. A regression in, say, the `ceil_log2` function or the `always_ff` emitter shape would only be caught indirectly (or not at all, in the emitter's case, since a change to the `always_ff` header text would still validate). Direct unit tests convert those implicit regressions into visible test failures.

**Validation**
- `cargo test`: 20 unit + 2 integration = 22 tests, all pass.
- `cargo check --all-targets`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --all --check`: all clean.

**Impact**
- Phase 1 exit gate reduced to just "Verilator-lint pass on a representative seed range." All Rust-side checks are in place.
- Future refactors of cone helpers or the emitter will fail tests loudly rather than silently drift.

**Files touched**
`src/gen/cone.rs`, `src/emit/sv.rs`, `CODEBASE_ANALYSIS.md`, `MEMORY.md`, `CHANGES.md`.

---

## 2026-04-15-0008 — Per-gate width/arity validator + inline unit tests

**Commit hash:** `4eb5daa`

**What changed**
- `src/ir/validate.rs`:
  - Replaced the TODO `// 5. Gate operand widths agree with declared output width rules.` with a full per-gate shape checker. Every `GateOp` variant has explicit arity and width rules:
    - `And / Or / Xor / Add / Sub / Mul` — 2 operands, each width = output width.
    - `Not` — 1 operand, width = output width.
    - `Mux` — 3 operands, `[sel 1-bit, a out_w, b out_w]`.
    - `Eq / Neq / Lt / Gt / Le / Ge` — 2 operands, equal width, output = 1-bit.
    - `RedAnd / RedOr / RedXor` — 1 operand of any width, output = 1-bit.
    - `Shl / Shr` — 2 operands, value operand width = output width, shift amount unconstrained.
    - `Slice { hi, lo }` — 1 operand, `hi >= lo`, `out_w == hi - lo + 1`, source width > `hi`.
    - `Concat` — variadic (>= 1 operand), `out_w == sum(operand widths)`.
  - New richer `ValidateError` variants: `GateArity`, `GateOperandWidth`, `GateOutputWidth`, `GateOperandsMustMatch`. Old `OperandWidth` and `WidthMismatch` variants retired.
  - New inline `#[cfg(test)] mod tests` (8 tests):
    - `accepts_minimal_valid_module`
    - `rejects_and_operand_width_mismatch`
    - `rejects_mux_non_1bit_selector`
    - `rejects_eq_output_not_1bit`
    - `rejects_concat_sum_mismatch`
    - `rejects_slice_out_of_bounds`
    - `rejects_not_wrong_arity`
    - `accepts_concat_variadic_replicate` (the N-copy pattern used by the width adapter and flop-mux assembly).
- `CODEBASE_ANALYSIS.md`:
  - Module map for `validate.rs` updated to note the width-rule checker and inline unit tests.
  - "Invariants currently enforced" / `ir::validate::validate` section now enumerates the per-gate width contract.
  - "Testing surface" entry for `src/ir/validate.rs` added.
  - "Known weaknesses": removed the now-closed "validator does not check per-gate operand widths" item.
- `DEVELOPMENT_NOTES.md`:
  - Testing-strategy section gains a paragraph on the validator's new role: an active safety net specifically designed to catch width bugs in the hand-constructed flop-mux assembly code (where gate-building does not go through the recursion).
- `MEMORY.md`:
  - Next-up list updated to reflect the closed validator task.
  - Recent-commits list gains `f2a3d81` (the previous commit).
  - Known-gaps list retires the per-gate validator TODO.

**Why**
Phase 1's exit criteria call for a working, audited single-module generator. Without a per-gate width validator, generator bugs in the hand-constructed flop-mux assembly (where gates like `Mux`, `And`, `Eq`, `Concat` are built by hand rather than via the recursion's `input_widths_for`) could emit subtly malformed IR that happens to parse but violates SV semantics. The width validator catches these at the IR level, before the emitter or any downstream tool ever sees them.

The inline unit tests pin the validator's behavior: each rejection class has a dedicated test so future changes to the width rules cannot silently drop a case.

**Validation**
- `cargo check --all-targets`: clean.
- `cargo test`: 8 new unit tests + 2 pipeline integration tests = 10 total, all pass.
- `cargo clippy --all-targets -- -D warnings`: clean.
- `cargo fmt --all --check`: clean.
- Pipeline sweep of 20 seeds passes with the stricter validator active, confirming the generator is currently producing width-correct IR and the validator is an *active* (not drift-prone) safety net.

**Impact**
- Generator bugs that produce width-mismatched gates are now caught at validation time with specific, actionable error messages (node id, op, operand index, expected vs got widths).
- Phase 1 exit is one step closer: the remaining Phase 1 tasks are in-source unit tests for `cone.rs` / `sv.rs` and the Verilator/Yosys smoke run.

**Files touched**
`src/ir/validate.rs`, `CODEBASE_ANALYSIS.md`, `DEVELOPMENT_NOTES.md`, `MEMORY.md`, `CHANGES.md`.

---

## 2026-04-15-0007 — Elevate mdBook to equal-standing live doc in session recovery

**Commit hash:** `f2a3d81`

**What changed**
- `SESSION_BOOTSTRAP.md`: reworded the mdBook entry in the bootstrap reading order. The book is now described explicitly as a live doc, not reference material, with language stating that a session skipping the book will make locally-correct but globally-wrong decisions.
- `COMMIT.md`:
  - Reworded the `book/` files-involved section: the mdBook is "a live doc of equal standing" and is "load-bearing" for session recovery.
  - Item 9 of the 12-item pre-commit checklist now explicitly states the mdBook's role and mandates adding permanent design decisions there, not just in commit messages.
- `README.md`: the ramp-up reading list entry for `book/` now states equal standing and the recovery-requires-reading-it stance. Follow-up sentence clarifies the book is part of the status-authority set, not adjacent to it.

**Why**
The user pointed out that the mdBook is part of the context-rebuild surface for post-crash / post-session-loss recovery, not a separate reference tier. The short-form live docs (`README`, `ROADMAP`, `MEMORY`, `CHANGES`, `DEVELOPMENT_NOTES`, `CODEBASE_ANALYSIS`, `USER_GUIDE`, `COMMIT`) carry *operational* state; the mdBook carries *design* context — why the generator is shaped the way it is, what has been deliberately rejected, what the motif catalogue looks like. A session that reconstructs operational state without the design context will make decisions that are locally coherent but globally wrong.

This slice makes the mdBook's recovery role explicit in three places (`SESSION_BOOTSTRAP.md`, `COMMIT.md` preamble + checklist, `README.md` reading list) so no future session can miss it.

**Validation**
- Documentation-only slice; no source changes.
- `cargo check`, `cargo test`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --all --check`: all still clean (no code touched).

**Impact**
- The 12-item pre-commit checklist now has an explicitly strengthened item 9 that closes a gap where design decisions might have landed in commit messages and `DEVELOPMENT_NOTES.md` but not in the mdBook.
- New sessions reading `SESSION_BOOTSTRAP.md` will not mistake the mdBook for optional reading.

**Files touched**
`SESSION_BOOTSTRAP.md`, `COMMIT.md`, `README.md`, `MEMORY.md`, `CHANGES.md`.

---

## 2026-04-15-0006 — Live-doc catch-up: capture flop-mux rationale + tighten commit workflow

**Commit hash:** `a1a9ea9`

**What changed**
- `DEVELOPMENT_NOTES.md`:
  - Added "Flop-D mux motifs" and "Q-exclusion contract" to the Core design decisions recap.
  - Added rejected alternative: `always_comb` + `case` for Encoded-mux flop D (why chained ternary wins).
  - Added rejected alternative: M = 1 mux arm (why it's excluded by design).
  - Added gotcha: module-level `#![allow(clippy::too_many_arguments)]` in `src/gen/cone.rs` with rationale.
  - Added calibration notes for `flop_mux_encoding_prob = 0.5` and `flop_qfeedback_prob = 0.5`.
  - Documented the QFeedback-in-Encoded design choice (replace `data_0` with Q) and the rejected alternative (extra (M+1)th entry).
- `MEMORY.md`:
  - Recent-commits list updated with `10090c2`.
  - Open-questions list updated with the `flop_mux_encoding_prob` calibration entry and the ternary-vs-case revisit trigger.
- `COMMIT.md`:
  - Added a non-negotiable 12-item pre-commit checklist. Every item is listed explicitly. The checklist makes skipping any live-doc update a visible workflow violation rather than a silent drift.

**Why**
Prior to this slice, the last two commits (`47675df` and `10090c2`) landed load-bearing design rationale — why M=1 is excluded, why chained ternary over `case`, why the Q-exclusion contract — that was captured in `CHANGES.md` and `book/src/sequential.md` but not in `DEVELOPMENT_NOTES.md`, which is the contributor-facing design-decision ledger. `MEMORY.md`'s recent-commits list was also one commit behind. The user flagged the slippage.

The fix has two parts: (1) a factual catch-up of the missed content, and (2) a structural fix to the commit workflow itself — an explicit 12-item pre-commit checklist in `COMMIT.md` that makes every live-doc gate impossible to skip implicitly.

**Validation**
- Documentation-only slice; no source changes.
- `cargo check`, `cargo test`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --all --check`: all still clean (no code touched).

**Impact**
- Future sessions can reconstruct the full design rationale from `DEVELOPMENT_NOTES.md` alone, without having to archaeologize across commit messages.
- The pre-commit checklist makes workflow compliance auditable: each item is either affirmatively satisfied or the commit does not proceed.

**Files touched**
`DEVELOPMENT_NOTES.md`, `MEMORY.md`, `COMMIT.md`, `CHANGES.md`.

---

## 2026-04-15-0005 — Encoded-select flop mux (chained ternary) alongside one-hot

**Commit hash:** `10090c2`

**What changed**
- `src/ir/types.rs`:
  - Replaced `Flop.arms: Vec<MuxArm>` with `Flop.mux: FlopMux`.
  - `FlopMux` enum: `None` (M=0), `OneHot(Vec<MuxArm>)`, `Encoded { sel: NodeId, data: Vec<NodeId> }`.
- `src/config.rs`:
  - New knob `flop_mux_encoding_prob` (default `0.5`): per-flop probability of using the encoded-select style instead of one-hot.
- `src/gen/cone.rs`:
  - New `drain_flop_encoded`: builds one select sub-cone of width `ceil(log2(M))` and M (or M-1 for QFeedback) data sub-cones, assembles D as a chained ternary over `Eq(sel, k)` with a `0` or `Q` fall-through.
  - New `drain_flop_one_hot`: extracts the previous one-hot assembly into its own function.
  - New `assemble_flop_d_encoded`, `make_constant`, `make_eq_const`, `make_mux`, `ceil_log2` helpers.
  - Renamed `assemble_flop_d` → `assemble_flop_d_one_hot`.
  - Per-flop dispatch in `drain_flop_worklist`: picks encoded or one-hot via `cfg.flop_mux_encoding_prob`.
  - Module-level `#![allow(clippy::too_many_arguments)]` to silence the lint on helpers that legitimately thread many context refs.
- `book/src/sequential.md`: documents both encoding styles, the 2×2 style-kind matrix, and the QFeedback+Encoded special case where index 0 is replaced by Q.
- `USER_GUIDE.md`: documents `--flop-mux-encoding-prob`.
- `CODEBASE_ANALYSIS.md`: module map, helper list, and invariants updated for the new drain path.
- `MEMORY.md`: state, next-up, recent commits refreshed.

**Why**
The user asked for an encoded-select variant alongside the existing one-hot, with the Q-feedback case routing Q on `sel == 0` and on out-of-range values. Both styles correspond to real synchronous-design shapes (one-hot for arbitration-driven register banks, encoded for opcode/address/state-selected registers) and exercise different synthesis paths. Picking per-flop preserves motif diversity within a single generated module.

**Validation**
- `cargo check`, `cargo test` (2 tests pass, ~2s for 20-seed sweep), `cargo clippy --all-targets -- -D warnings`, `cargo fmt --all --check`: all clean.
- Visual inspection with `--seed 5 --max-depth 2 --flop-prob 1.0` shows chained ternaries in the output: `(eq_k) ? data_k : (eq_{k-1}) ? data_{k-1} : ... : fall_through`, confirming the encoded-mux assembly.

**Impact**
- Phase 1 now emits two distinct flop motifs. Motif diversity is no longer bound by encoding style.
- The `FlopMux` enum carries introspective information about each flop's mux shape, useful for future debugging/inspection tooling even though it is not load-bearing for emission today.

**Files touched**
`src/ir/types.rs`, `src/config.rs`, `src/gen/cone.rs`, `book/src/sequential.md`, `USER_GUIDE.md`, `CODEBASE_ANALYSIS.md`, `MEMORY.md`, `CHANGES.md`.

---

## 2026-04-15-0004 — M-to-1 one-hot mux flops with two motifs

**Commit hash:** `47675df`

**What changed**
- `src/ir/types.rs`:
  - New `FlopKind` enum: `ZeroDefault` (D = 0 when no select fires) and `QFeedback` (D = Q when no select fires).
  - New `MuxArm { data: NodeId, sel: NodeId }` representing one arm of a flop's input mux.
  - `Flop` gains `kind: FlopKind` and `arms: Vec<MuxArm>` fields.
- `src/gen/cone.rs`:
  - `build_cone_with_retry` and `build_cone` gain an `exclude: Option<NodeId>` parameter threaded into `pick_terminal`. Used to forbid this flop's own Q from being a leaf in any of its data or select sub-cones.
  - `pick_mux_arm_count` returns M from {0, 2, 3, ..., max_mux_arms}. M = 1 excluded by design (a 1-arm mux is a wire).
  - `drain_flop_worklist` rewritten:
    - For M = 0: D = recursive cone of width N (no mux).
    - For M >= 2: build M data sub-cones (width N) + M select sub-cones (1-bit), every one a recursion point. Assemble `D = OR_i({N{sel_i}} & data_i)`, plus `({N{~(OR sel_i)}} & Q)` for `QFeedback`.
  - New helpers: `assemble_flop_d`, `replicate_to_width` (N-fold Concat of a 1-bit signal), `make_and`, `make_none_selected`, `or_reduce_terms`.
  - `build_flop_leaf` picks a random `FlopKind` per flop (`flop_qfeedback_prob` knob).
- `src/config.rs`:
  - New knobs: `min_mux_arms` (default 1, becomes effective floor of 2 inside `pick_mux_arm_count`), `max_mux_arms` (default 4), `flop_qfeedback_prob` (default 0.5).
  - `Config::validate` checks the mux-arm range and the new probability.
  - New error variant `MuxArmsRange`.
- `src/gen/module.rs`: passes `None` exclusion for output cones.
- `book/src/sequential.md`: documents M=0 vs M>=2 cases, both flop kinds, and the Q-exclusion contract enforced via `exclude: Option<NodeId>`.
- `USER_GUIDE.md`: documents `--min-mux-arms`, `--max-mux-arms`, `--flop-qfeedback-prob` knobs.
- `CODEBASE_ANALYSIS.md`: module map updated for new helpers; invariants list updated.
- `MEMORY.md`: state, next-up, recent commits refreshed.

**Why**
The user specified the precise flop motif `anvil` should generate:
1. M ∈ {0, 2, 3, ...}. M = 0 means no mux, D recurses directly.
2. For M >= 2: each of the M data inputs (width N) is a recursion point; each of the M 1-bit select bits is a recursion point. Selects are one-hot (a design contract, not enforced).
3. Two kinds: `ZeroDefault` (D = 0 on no-select) and `QFeedback` (D = Q on no-select).
4. The flop's own Q is forbidden from feeding any of its data or select sub-cones — the *only* permitted Q→D path is the explicit Q-feedback term in `QFeedback`.

This produces RTL that resembles real synchronous datapath idioms (one-hot-controlled register banks, holding registers, etc.) rather than generic register-of-arbitrary-cone shapes.

**Validation**
- `cargo check`, `cargo test`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --all --check`: all clean.
- Visual inspection of `seed=3, max-depth=2, flop-prob=1.0` confirms:
  - `assign w_X = {bit, bit, ..., bit};` (replicate sel_i to N bits)
  - `assign w_Y = w_X & data_i;` (mask)
  - `assign w_Z = w_A | w_B;` (OR-reduce arm terms)
  - For `QFeedback`: extra `~(OR of sels)` term ANDed with Q.

**Impact**
- Generated flop motifs now match a real-world synchronous-design pattern.
- Tests run slower (~3-4s for the 20-seed sweep vs ~0.04s previously) due to the M+M sub-cone fan-out per flop. Tolerable; tunable via `max_mux_arms` and `max_flops_per_module`.

**Files touched**
`src/ir/types.rs`, `src/config.rs`, `src/gen/cone.rs`, `src/gen/module.rs`, `book/src/sequential.md`, `USER_GUIDE.md`, `CODEBASE_ANALYSIS.md`, `MEMORY.md`, `CHANGES.md`.

---

## 2026-04-15-0003 — Fold flops into the cone recursion (single-clock synchronous discipline)

**Commit hash:** `4317c82`

**What changed**
- `src/gen/cone.rs`:
  - New `FlopWorklist` type alias (`Vec<FlopId>`).
  - `build_cone` now decides between `Gate` and `Flop` at each non-leaf node, gated by `cfg.flop_prob` and `cfg.max_flops_per_module`.
  - New `build_flop_leaf`: allocates a `Flop`, pushes a `FlopQ` node, queues the flop for D-cone construction, returns Q as the leaf for the current cone.
  - New `drain_flop_worklist`: pops queued flops one at a time, recursively builds each D-cone with `build_cone_with_retry` (which itself may push more flops); loops to quiescence.
  - `build_cone_with_retry` now also snapshots/rewinds `m.flops` and the worklist.
  - All flops use `ResetKind::Async` unconditionally (single-CLK / single-RST_N discipline).
  - New `pick_reset_value` (50% zero, 25% all-ones, 25% random).
- `src/gen/module.rs`:
  - Reserves port id 0 for `clk` and 1 for `rst_n`. Sets `Module.clock` and `Module.reset`. Excludes them from the signal pool so cones cannot terminate at them.
  - Drains the flop worklist after building all output cones.
- `src/emit/sv.rs`:
  - Emits `logic [W-1:0] r_<id>;` for every flop.
  - Emits a single `always_ff @(posedge clk or negedge rst_n)` block containing all flops, with reset-branch initializing every flop and else-branch sequencing every flop's D.
  - Conditionally omits `clk`/`rst_n` from the port list when the module has no flops.
- `src/config.rs`:
  - `flop_prob` default raised to `0.15` (was `0.0`).
  - New knob `max_flops_per_module` (default `32`) capping flop count to bound generation time.
- `book/src/sequential.md`:
  - Reframed: flops are part of the same cone recursion, not a later phase.
  - New "Synchronous-design discipline" section spelling out the single-CLK / single-RST_N async constraint.
  - Updated example `always_ff` block.
- `ROADMAP.md`:
  - Phase 1 collapsed: combinational + sequential together. Old Phase 3/5/7 renumbered to new Phase 2/4/6.
- `USER_GUIDE.md`:
  - Updated `flop_prob` default.
  - Documented `max_flops_per_module` knob.
- `DEVELOPMENT_NOTES.md`:
  - Added "Synchronous-design discipline" as a core design decision.
- `CODEBASE_ANALYSIS.md`:
  - Updated module map for new cone helpers.
  - Updated phase coverage map (collapse + renumber).
  - Documented new construction-time invariants (flop allocation, single-clock, clk/rst_n exclusion from pool).
- `MEMORY.md`:
  - Recorded `c4668a2`.
  - Refreshed current state, next-up, open questions, known gaps.

**Why**
The user pointed out that artificially deferring flops to a later phase contradicts the recursion-as-core-principle stance: Q is just another leaf, D is just another sub-cone, the worklist is the same iterative shell that drives output cones. Folding sequential into Phase 1 also unlocks meaningful synthesis testing — purely combinational random RTL is far less representative of real designs than mixed sequential/combinational.

The single-CLK / single-RST_N (async, active-low) constraint matches real fully-synchronous design practice. Enforcing it by construction (no IR field for per-flop clock or polarity) means no random choice can violate it.

**Validation**
- `cargo check --all-targets`, `cargo test` (2 tests pass), `cargo clippy --all-targets -- -D warnings`, `cargo fmt --all --check`: all clean.
- `cargo run -- --seed 7`: produces a module with `always_ff @(posedge clk or negedge rst_n)`, all flops in one block, async-reset to per-flop reset values.
- IR validator passes across the 20-seed sweep with flops enabled.

**Impact**
- Phase 1 is now a meaningful single-module MVP rather than a combinational stub.
- Generated RTL now includes registered state, which is far more representative for downstream synthesis tooling.

**Files touched**
`src/config.rs`, `src/gen/cone.rs`, `src/gen/module.rs`, `src/emit/sv.rs`, `book/src/sequential.md`, `ROADMAP.md`, `USER_GUIDE.md`, `DEVELOPMENT_NOTES.md`, `CODEBASE_ANALYSIS.md`, `MEMORY.md`, `CHANGES.md`.

---

## 2026-04-15-0002 — Elevate "recursion is the core principle" to load-bearing status

**Commit hash:** `c4668a2`

**What changed**
- `README.md`: rewrote the project-objective section as **three** load-bearing principles, with recursion as the first. Recursion is now stated explicitly as the default algorithmic shape for any non-trivial generation step.
- `book/src/core-idea.md`: prepended a "The single guiding principle: recursion" section before the existing thesis. States that recursion is the default; iteration is the exception (flop worklist, per-output driver loop) and exists only to *kick off* recursive cone construction. Anchors the correctness argument: each recursive call carries its own constraints, which is what makes "valid by construction" hold.
- `DEVELOPMENT_NOTES.md`: added recursion as the first entry in the "Core design decisions" recap, with a pointer to the new book section.
- `MEMORY.md`: recorded `5f6022f` (the previous slice's commit hash).

**Why**
The user explicitly stated: "By design, anvil shall be heavily recursive — recursion is its core principle." The design as implemented already follows this, but the docs only hinted at it. Elevating it to first-class status ensures future contributors do not casually replace recursion with iteration in places where the recursion structure is what guarantees invariant preservation.

**Validation**
- Docs-only slice; no code changes.
- `cargo check`, `cargo test`: still clean (no source touched).

**Impact**
- Future PRs that introduce iterative scaffolding around generation logic should now expect to justify the choice against the "recursion is the default" principle.

**Files touched**
`README.md`, `book/src/core-idea.md`, `DEVELOPMENT_NOTES.md`, `MEMORY.md`, `CHANGES.md`.

---

## 2026-04-15-0001 — Initial scaffold + Phase 1 cone-adapter hardening

**Commit hash:** `5f6022f`

**What changed**
- Created Cargo project `anvil` with binary + library targets.
- Added `Cargo.toml` with deps: `rand`, `rand_chacha`, `clap` (derive), `serde`, `serde_json`, `thiserror`, `anyhow`.
- Added crate skeleton:
  - `src/lib.rs` — public re-exports (`Config`, `Generator`, `Module`).
  - `src/main.rs` — CLI (`--seed`, `--count`, `--out`, `--config`, `--dump-config`, knob overrides).
  - `src/config.rs` — `Config` struct, defaults, `validate()`, CLI overlay.
  - `src/ir/types.rs` — `Module`, `Port`, `Node`, `GateOp`, `Flop`, `DepSet`.
  - `src/ir/validate.rs` — IR invariant checker (safety net).
  - `src/gen/mod.rs` — `Generator` entry points, ChaCha8-seeded RNG.
  - `src/gen/module.rs` — leaf-module generator (N inputs, M outputs, cone per output).
  - `src/gen/cone.rs` — fanin-cone recursion with depth budget, anti-collapse rules, dep-set tracking, bounded retry on trivial cones.
  - `src/gen/pool.rs` — `SignalPool` for terminal selection.
  - `src/emit/sv.rs` — IR → SystemVerilog pretty-printer.
- Added `tests/pipeline.rs` — generates 20 seeds, asserts IR validation passes and SV output is non-empty; reproducibility test.
- Added `examples/generate_one.rs` — minimal library-usage example.
- Added live-doc set:
  - `README.md` — entry point.
  - `SESSION_BOOTSTRAP.md` — read-first on session recovery.
  - `ROADMAP.md` — 7-phase plan, exit criteria per phase.
  - `USER_GUIDE.md` — CLI, knobs, downstream verification.
  - `MEMORY.md` — operational continuity snapshot.
  - `CHANGES.md` (this file).
  - `DEVELOPMENT_NOTES.md` — engineering rationale.
  - `CODEBASE_ANALYSIS.md` — live workspace analysis.
  - `COMMIT.md` — commit workflow.
- Added mdBook design rationale at `book/`:
  - `core-idea.md`, `why-not-grammar.md`, `algorithm.md`, `ir.md`,
    `by-construction.md`, `synthesizability.md`, `non-triviality.md`,
    `sequential.md`, `sharing.md`, `hierarchy.md`, `knobs.md`,
    `architecture.md`, `non-goals.md`.
- Added `.gitignore` covering `/target`, `book-out`, `Cargo.lock`, swap files, and `git_message_brief.txt`.
- **Phase 1 hardening:** lazy width-adapter in `gen::cone::pick_terminal`. When the signal pool has no matching-width entry, build a Slice (or replicating Concat + Slice) from the widest available pool entry with non-empty deps, instead of falling back to a bare constant. Preserves dep-set propagation and resolves the seed-0 IR-validation failure where output cones were collapsing to constants.
- Added `gen::cone::make_width_adapter` helper.
- `gen::pool::SignalPool::iter()` exposed for adapter source selection.
- Clippy cleanups: `Config { seed, ..Default::default() }` patterns in tests/example; `u32::div_ceil` for adapter copy count.
- All `cargo fmt` corrections applied.

**Why**
Project bootstrap. The brainstorming session that preceded this slice converged on a circuit-graph-IR generator with by-construction validity, dep-set tracking for non-triviality, and explicit synthesizability-as-subset enforcement.

The lazy adapter fixes a Phase 1 bug surfaced on the first `cargo test` run: when randomly-chosen output port widths do not match any randomly-chosen input port width, the cone has no signal of the required width to terminate at, falls back to a constant, and the cone root's dep-set is empty. The validator correctly rejects this, but the bounded retry loop cannot recover because the pool composition does not change between attempts. The adapter resolves this structurally — any output width can now reach an input via Slice/Concat — without weakening the by-construction discipline.

**Validation**
- `cargo check --all-targets` clean.
- `cargo test`: 2 tests pass (`generates_valid_modules_across_seeds` over seeds 0..20, `reproducibility` byte-identical for seed 12345).
- `cargo clippy --all-targets -- -D warnings`: clean.
- `cargo fmt --all --check`: clean.
- `cargo run -- --seed 42`: produces a 4-output, 3-input module with a coherent assign net (visual spot-check).
- `cargo run -- --seed 7 --count 5 --out /tmp/anvil_out`: 5 .sv files + manifest.json written.
- External smoke tests (Verilator, Yosys): tools not installed locally; smoke runs are deferred until the dev environment provides them or CI is wired.

**Impact**
- Phase 0 (Scaffolding) exit criteria met: `cargo build` and `cargo test` pass.
- Phase 1 (Combinational MVP) is in progress: cone recursion functional and dep-set-correct across the seed sweep; remaining Phase 1 work is per-gate width-rule validation in `ir::validate`, unit tests inside source modules, and Verilator-lint smoke once available.
- `CODEBASE_ANALYSIS.md` "Known weaknesses" item #1 is resolved by this slice.

**Files touched**
All files in the repository (initial creation), plus subsequent edits to `src/gen/cone.rs`, `src/gen/pool.rs`, `tests/pipeline.rs`, `examples/generate_one.rs`.
