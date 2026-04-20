# Changes
Fully detailed change history. Newest entries at the top. One entry per commit.

---

## 2026-04-20-0079 — Harden canonical flop identity validation

**Landed as:** _to be filled in after this commit_

**What changed**

This slice turns `ir::validate::validate` into a real
stateful-identity contract checker, not just a gate-shape checker.

### Drive roots and flop-held `NodeId`s are now validated before indexing

- Added three new structural error classes:
  - `UndefinedDriveRoot`
  - `FlopIdMismatch`
  - `UndefinedFlopNode`
- Every output drive root must exist before the validator inspects
  cone roots or gate shape.
- Every flop table slot must keep the dense canonical relation
  `m.flops[idx].id == idx`.
- `Flop.d`, `Flop.q`, and every `NodeId` stored inside `FlopMux`
  must point at a live node.

### Canonical `Flop.q <-> Node::FlopQ` backreferences are now enforced

- `Flop.q` must point at a `Node::FlopQ`.
- That `Node::FlopQ` must point back to the same flop and carry
  the same width as the owning `Flop`.
- Every `Node::FlopQ` in the arena must:
  - reference a real flop;
  - match that flop's width; and
  - be the canonical `q` node for that flop.
- This catches stale duplicate Q nodes, renumbering mistakes, and
  dangling post-merge state references before later passes or the
  emitter can trust a broken IR.

### Validator helpers and tests were expanded alongside the contract

- Added `node_exists` and `validate_flop_mux_refs`.
- The node-side `FlopQ` width check now runs before the canonical-q
  check so the dedicated width-mismatch error path is reachable on
  stale duplicate `FlopQ` nodes too.
- `src/ir/validate.rs` gained 10 new stateful-invariant unit tests
  covering:
  - undefined drive roots;
  - flop-id mismatch;
  - missing D;
  - non-`FlopQ` q node;
  - q backref mismatch;
  - q width mismatch;
  - dangling / noncanonical / wrong-width `FlopQ` nodes; and
  - undefined mux-held node references.
- Live docs + book were refreshed so the validator is now described
  as owning the canonical state-backreference contract.

**Why**

The previous slice introduced post-drain flop merging under
`identity_mode = node-id`. That made `Flop.id`, `Flop.q`, and
`Node::FlopQ { flop, .. }` part of the recovery-critical identity
fabric, not incidental metadata.

Per-gate arity/width checks were no longer enough: a bad
renumbering pass, stale duplicate Q node, or dangling drive root
could panic later passes or silently corrupt the "NodeId means
identity" story. This hardening moves those failures into one
explicit development-time safety net.

**Validation**

- `cargo check --all-targets`
- `cargo test` (96 unit + 24 integration = 120 passing tests)
- `cargo clippy --all-targets -- -D warnings`
- `cargo fmt --all --check`
- `mdbook build book`

**Impact**

- Stateful identity now has a validator-backed contract instead of a
  best-effort convention.
- Post-drain state-rewrite passes may still renumber and merge, but
  they now have to leave a provably self-consistent IR behind.
- Session recovery docs now point at the exact invariants protecting
  future deeper sequential-factorization work.

**Files touched**

- `src/ir/validate.rs`
- `CHANGES.md`
- `MEMORY.md`
- `DEVELOPMENT_NOTES.md`
- `CODEBASE_ANALYSIS.md`
- `book/src/ir.md`
- `book/src/architecture.md`
- `book/src/by-construction.md`

## 2026-04-20-0078 — Extend NodeId identity to exact-signature flops

**Landed as:** `420fbd4`

**What changed**

This slice extends the `node-id` identity story from purely
combinational nodes into the first conservative sequential case:
duplicate flops.

### Post-drain exact-signature flop merge now runs at module finalisation

- Added `crate::ir::compact::merge_equivalent_flops(&mut Module) ->
  u32`.
- The pass runs only when:
  - `identity_mode = node-id`; and
  - the effective factorization level is at least `cse`.
- It executes after `drain_flop_worklist` and after
  `summarize_flop_mux_metadata`, when every flop finally has a
  concrete `d`.
- Flops merge when they have the same exact emitted-state
  signature:
  - `width`
  - `reset_kind`
  - `reset_val`
  - exact same `d: NodeId`

### The merge rewires state consumers, not just the flop table

- Duplicate Q users are rewritten to the canonical Q.
- Virtual flop deps inside `DepSet` are remapped through the
  old-flop-id -> new-flop-id table, so dependency tracking stays
  truthful after the merge.
- Surviving flops are renumbered densely; their `Flop.id` and
  `Node::FlopQ { flop, .. }` references are kept in sync.
- Dedup tables are rebuilt after the rewrite so the final module
  metadata matches the post-merge IR.
- The later `compact_node_ids` pass then removes the now-dead
  duplicate `FlopQ` nodes.

### Construction-only flop provenance is intentionally ignored

- The signature does **not** include `FlopKind`.
- The signature does **not** include `FlopMux` operand metadata.
- Rationale: by the time this pass runs, emitted hardware
  semantics are determined by width/reset/D. `FlopKind` and the
  cleared mux operands are construction provenance / telemetry,
  not emitted behavior.
- This means a `ZeroDefault`-born flop and a `QFeedback`-born
  flop can merge if they ended up with the same actual `d`.

### New telemetry surfaced the state-sharing result

- Added `Module::flops_merged: u32`.
- Added `Metrics::flops_merged: u32`.
- `generate_leaf_module` now records the merge count and logs it
  alongside node/flop/compaction totals.

### Tests and docs were updated to match the new semantics

- `src/ir/compact.rs` gained three merge-specific unit tests:
  - merge rewrites gate operands and virtual deps correctly;
  - `identity_mode = relaxed` bypasses the pass;
  - different reset signatures do not merge.
- Existing compaction tests remain intact.
- The live docs and book now state the important nuance
  explicitly:
  combinational factorization is mainly intern-time, but stateful
  exact-signature sharing is a post-drain finalisation step.

**Why**

The previous slice made identity mode a real typed axis, but the
stateful side still had a visible hole: `build_flop_leaf` always
allocated a fresh flop, so "NodeId is the identity of an
expression" silently stopped at registers.

There is no honest way to solve that at allocation time because a
flop's semantics are not known when its Q is born; the D-cone only
exists after the worklist drains. So the right next step was a
conservative post-drain pass that merges the cases we can prove
exactly today without pretending to solve general sequential
equivalence.

**Validation**

- `cargo check --all-targets`
- `cargo test` (86 unit + 24 integration = 110 passing tests)
- `cargo clippy --all-targets -- -D warnings`
- `cargo fmt --all --check`
- `mdbook build book`

**Impact**

- `identity_mode = node-id` now reaches one level deeper into the
  sequential fabric: exact duplicate state elements can share one
  identity too.
- The project now has a clean architectural split:
  - combinational identity at intern time;
  - conservative state identity after drain.
- `flops_merged` makes the new behavior measurable, so future
  stateful-factorization work has a baseline.
- The remaining gap is clear and documented: deeper sequential
  equivalence still needs a stronger coinductive / e-graph-style
  story.

**Files touched**

- `src/ir/types.rs`
- `src/ir/compact.rs`
- `src/gen/module.rs`
- `src/metrics.rs`
- `README.md`
- `USER_GUIDE.md`
- `DEVELOPMENT_NOTES.md`
- `CODEBASE_ANALYSIS.md`
- `ROADMAP.md`
- `book/src/algorithm.md`
- `book/src/architecture.md`
- `book/src/factorization.md`
- `book/src/faq.md`
- `book/src/ir.md`
- `book/src/knobs.md`
- `book/src/non-triviality.md`
- `book/src/sequential.md`
- `book/src/sharing.md`
- `book/src/structural-rules.md`
- `CHANGES.md`
- `MEMORY.md`

## 2026-04-20-0077 — Make identity mode a first-class typed axis

**Landed as:** `033e03d`

**What changed**

This slice turns "NodeId as identity" from a documented doctrine
plus CLI sugar into a first-class typed axis in the codebase.

### Config / CLI / IR now model identity mode explicitly

- Added `IdentityMode` in `src/config.rs`:
  - `node-id` (default): NodeId means expression identity; the
    factorization ladder stays live.
  - `relaxed`: disable the identity/factorization ladder
    entirely and allocate fresh NodeIds for every AST.
- Added `identity_mode` to `Config`, `Overrides`, `Cli`, and
  `Module`.
- Added `Config::effective_factorization_level()` and
  `Module::effective_factorization_level()` so the coarse
  identity mode is applied before every factorization gate.
- Added the new CLI flag:
  `--identity-mode <node-id|relaxed>`.

### Convenience aliases now expand to the explicit coarse+fine pair

- `--full-factorization` now means:
  `--identity-mode node-id --factorization-level e-graph`.
- `--no-full-factorization` now means:
  `--identity-mode relaxed --factorization-level none`.
- The aliases now conflict with explicit `--identity-mode` /
  `--factorization-level` so the CLI no longer silently mixes
  sugar and direct control.

### All identity/factorization gating now consults the effective mode

- `Module::intern_gate` and `Module::intern_constant` no longer
  read the raw ladder directly; they consult
  `self.effective_factorization_level()`.
- `gen::cone::{make_and, make_mul, violates_anti_collapse}` do
  the same, so operand-uniqueness / anti-collapse behavior now
  tracks the coarse identity mode consistently.
- `generate_leaf_module` now copies `cfg.identity_mode` into the
  per-module IR mirror just like the other construction-time
  knobs.

### Proof tests landed for the new semantics

- Added a config unit test proving:
  - `identity_mode = node-id, factorization_level = e-graph`
    resolves to `peephole` today;
  - `identity_mode = relaxed` forces the effective level to
    `none`.
- Added CLI unit tests for:
  - direct `--identity-mode relaxed` parsing;
  - `--full-factorization` setting both `identity_mode` and
    `factorization_level`;
  - `--no-full-factorization` doing the inverse.
- Added an IR unit test proving the same requested
  `factorization_level = e-graph`:
  - dedupes under `IdentityMode::NodeId`;
  - allocates fresh NodeIds under `IdentityMode::Relaxed`.

### Docs now describe the same model the code implements

- README / USER_GUIDE now document `--identity-mode` directly.
- The book chapters on knobs, factorization, structural rules,
  IR, architecture, sharing, non-triviality, algorithm, and FAQ
  now distinguish:
  - coarse identity mode;
  - fine-grained factorization rung;
  - construction strategy as a separate axis.
- `DEVELOPMENT_NOTES.md` records the design consequence:
  the separation is now in types and gating sites, not just in
  prose.
- `CODEBASE_ANALYSIS.md` and `MEMORY.md` were refreshed for
  session recovery.

**Why**

The repo had drifted into an awkward in-between state:

- the docs correctly said "NodeId as identity" is orthogonal to
  cone-construction strategy;
- the CLI had sugar for "full factorization on/off";
- but the code still treated the identity story mostly as a raw
  `factorization_level` ladder.

That mismatch was survivable for users, but bad for recovery and
future work. The next deeper step toward full factorization must
reason about identity mode explicitly, especially once flops and
future hierarchy enter the question. This slice makes the
separation real without changing the already-working ladder.

**Validation**

- `cargo check --all-targets`
- `cargo test` (83 unit + 24 integration = 107 passing tests)
- `cargo clippy --all-targets -- -D warnings`
- `cargo fmt --all --check`
- `mdbook build book`

**Impact**

- The codebase now has an honest first-class place to talk about
  "NodeId as identity" without smuggling it through the ladder.
- CLI/config/IR all agree on the model:
  coarse identity mode + fine-grained ladder.
- Future identity work for stateful / hierarchical objects can
  build on a real axis instead of more aliases and prose.

**Files touched**

- `src/config.rs`
- `src/main.rs`
- `src/gen/module.rs`
- `src/gen/cone.rs`
- `src/ir/types.rs`
- `README.md`
- `USER_GUIDE.md`
- `DEVELOPMENT_NOTES.md`
- `CODEBASE_ANALYSIS.md`
- `MEMORY.md`
- `ROADMAP.md`
- `book/src/algorithm.md`
- `book/src/architecture.md`
- `book/src/factorization.md`
- `book/src/faq.md`
- `book/src/ir.md`
- `book/src/knobs.md`
- `book/src/non-triviality.md`
- `book/src/sharing.md`
- `book/src/structural-rules.md`
- `CHANGES.md`

---

## 2026-04-20-0076 — Expose peak-sharing controls and exercise live categories

**Landed as:** `dd28086`

**What changed**

This slice tightens the control surface around the user's
"NodeId as identity" doctrine without conflating it with cone
construction strategy.

### CLI surface: peak sharing is now an explicit mode

- Added `--full-factorization` as a convenience alias for
  `--factorization-level e-graph` (request the strongest
  currently-implemented identity mode).
- Added `--no-full-factorization` as the coarse off-switch
  (`--factorization-level none`).
- Kept the existing detailed ladder under
  `--factorization-level <none|cse|operand-unique|commutative|associative|constant-fold|peephole|e-graph>`.
- Exposed previously config-only *live* knobs on the CLI:
  `--terminal-reuse-prob`, `--constant-prob`,
  `--gate-bitwise-weight`, `--gate-arith-weight`,
  `--gate-struct-weight`, `--gate-compare-weight`,
  `--gate-reduce-weight`.

### Dead leaf knobs are live now

- `gen::cone::pick_terminal` now consults
  `terminal_reuse_prob` at forced leaves with an exact-width
  pool candidate:
  - `1.0` = always reuse the matching-width signal;
  - `0.0` = never reuse it, emit a fresh constant instead.
- `pick_terminal` now consults `constant_prob` when no
  matching-width signal exists but a dep-bearing width-adapter
  source does:
  - hit = emit a fresh constant;
  - miss = build the width adapter.
- Both decisions route through `roll_knob`, so they become
  measurable in `knob_roll_attempts` / `knob_roll_fires`.

### Every live gate category is now genuinely exercisable

- `pick_gate`'s compare bucket now contains the full comparison
  family: `Eq`, `Neq`, `Lt`, `Gt`, `Le`, `Ge`.
- The reduction bucket is now live in `pick_gate`:
  `RedAnd`, `RedOr`, `RedXor` can be selected at 1-bit target
  width.
- `gate_reduce_weight` is therefore no longer a dead config
  field.

### Config / test hardening

- `Config::validate()` now rejects out-of-range
  `mux_arm_duplication_rate` and `operand_duplication_rate`
  values, matching their documented `[0.0, 1.0]` contract.
- Added unit tests for:
  - the new CLI aliases and newly-exposed CLI knobs;
  - probability validation of the two duplication-rate knobs;
  - `pick_gate` coverage across every live category;
  - `pick_terminal` edge behavior for
    `terminal_reuse_prob` and `constant_prob`.
- Added an end-to-end integration test proving that each live
  gate category is reachable in generated modules and still
  IR-valid.
- Extended the per-knob roll telemetry test to cover
  `constant_prob` and `terminal_reuse_prob`.

### Docs synced to shipping reality

- Refreshed stale factorization passages that still claimed only
  the first three rungs were implemented.
- Added the load-bearing clarification that construction
  strategy (`sequential` / `shuffled` / `interleaved`) is a
  separate axis from identity / sharing mode.
- Updated knob docs to reflect that `constant_prob`,
  `terminal_reuse_prob`, and the gate-category weights are live
  CLI-controlled knobs today.

**Why**

The user clarified the doctrinal target precisely:

- output drives and flop D inputs are cone roots;
- primary inputs and flop Qs are the leaves;
- the entire fanin forest should collapse toward a maximally
  shared DAG when the "NodeId as identity" mode is enabled;
- this identity mode is **orthogonal** to cone-construction
  strategy.

Before this slice, the coarse factorization dial existed, but
the user-facing on/off control was awkward, one gate category
(`reduce`) was effectively dead, and two documented leaf knobs
(`constant_prob`, `terminal_reuse_prob`) were not actually
consulted. Several guide/book passages also understated the
current live factorization ladder. This slice cleans that up so
the next architectural step toward deeper semantic identity can
start from a coherent, fully-exercised surface.

**Validation**

- `cargo check --all-targets`
- `cargo test` (80 unit + 24 integration = 104 passing tests)
- `cargo clippy --all-targets -- -D warnings`
- `cargo fmt --all --check`
- `mdbook build book`

**Impact**

- Peak sharing / "full factorization" now has an obvious coarse
  CLI on/off control that sits alongside, not inside, the
  construction-strategy choice.
- Every *live* gate category is now reachable and tested.
- Leaf-level sharing-vs-constant choices are no longer dead
  docs-only knobs; they affect generation and are measurable.
- The book/live docs now consistently describe the current
  factorization ladder as live through `peephole`, with
  `e-graph` still aspirational.

**Files touched**

- `src/main.rs`
- `src/config.rs`
- `src/gen/cone.rs`
- `src/ir/types.rs`
- `tests/pipeline.rs`
- `book/src/algorithm.md`
- `book/src/faq.md`
- `book/src/factorization.md`
- `book/src/knobs.md`
- `book/src/recipes.md`
- `book/src/structural-rules.md`
- `USER_GUIDE.md`
- `README.md`
- `DEVELOPMENT_NOTES.md`
- `CODEBASE_ANALYSIS.md`
- `ROADMAP.md`
- `MEMORY.md`
- `CHANGES.md`

---

## 2026-04-19-0075 — Finalise the live-signal surface for lint-clean output

**Landed as:** `e973d30`

**What changed**

This slice closes the "unused bits / unused signal" defect at the
generator/finalisation layer instead of hiding it in the emitter.

### Exact-width width adapters

- `gen::cone::make_width_adapter` no longer builds a wider replicated
  `Concat` and slices it back down for non-multiple up-width
  expansions.
- The adapter now emits the exact-width shape directly:
  `{src[rem-1:0], src, src, ...}`.
- This removes dead high bits like the old seed-42
  `concat_0[41:27]` case that Verilator flagged as unused.

### Finalisation now matches emitted hardware

- `gen::module::generate_leaf_module` gained a proper
  post-construction clean-up sequence:
  1. summarize `Flop.mux` metadata so construction-only select/data
     operand NodeIds do not keep dead cones alive;
  2. orphan audit;
  3. `compact_node_ids`;
  4. shrink surviving primary inputs to the highest bit any live
     consumer touches;
  5. prune entirely unused primary data-input ports from the emitted
     interface.
- This fixes the mismatch where IR liveness used `Flop.mux` bookkeeping
  as if it were emitted hardware, so metadata-only gates survived and
  later triggered `%Warning-UNUSEDSIGNAL`.

### Metrics / test semantics aligned with duplicate-preserving flattening

- `Metrics::nested_associative_operand_count` now counts only same-op
  nested slots that are still flattenable under the current duplicate
  policy.
- This stops strict `operand_duplication_rate = 0.0` `Add`/`Mul`
  shapes like `x * (x * y)` from being misreported as "missed"
  associative opportunities when flattening them would change
  semantics.

### Tests and docs

- Added two `src/gen/module.rs` unit tests for primary-input shrinking.
- Updated the width-adapter non-multiple unit test to pin the new
  exact-width Concat shape.
- Added integration test
  `no_unused_primary_data_inputs_remain_after_finalisation`.
- Renamed stale pipeline tests/comments that still described
  `graph-first` as the default strategy.
- Refreshed `src/config.rs`, `src/main.rs`, `USER_GUIDE.md`,
  `ROADMAP.md`, `DEVELOPMENT_NOTES.md`, `CODEBASE_ANALYSIS.md`, and
  `book/src/knobs.md` to reflect:
  - `interleaved` is the live default;
  - `graph-first` is a deprecated alias;
  - `graph_first_pool_size` is legacy on the live path;
  - finalisation trims dead input ports / dead input high bits.

**Why**

The reported bug was "unused bits of signal", initially blamed on the
graph-first family. Reproduction showed two distinct root causes:

1. width-adapter expansion created intentionally oversized Concats,
   leaving dead high bits in otherwise-live internal wires; and
2. `Flop.mux` metadata was being treated as a liveness root even
   though the emitter only consumes `flop.d`.

Once those were fixed, a seed sweep still exposed unused *input* high
bits coming from low-slice-only consumers, so finalisation now shrinks
the live input surface as well. The graph-first diagnosis was partly a
doc/test mirage: the current CLI `graph-first` path is just the
interleaved builder under a deprecated alias, so the stale comments had
to be cleaned up in the same slice.

**Validation**

- `cargo check --all-targets`
- `cargo test`
- `cargo clippy --all-targets -- -D warnings`
- `cargo fmt --all --check`
- `mdbook build book`
- `verilator --lint-only -Wall -Wno-DECLFILENAME -Wno-UNSIGNED /tmp/anvil_seed42_final.sv`
- Verilator unused-signal sweep (seeds 0..4) clean for both the default
  path and the `graph-first` alias
- `yosys -p "read_verilog -sv /tmp/anvil_seed42_final.sv; synth"`

**Impact**

- Emitted modules no longer carry metadata-only orphan wires into SV.
- Non-multiple width adaptation no longer manufactures dead high bits.
- Final SV interfaces no longer include dead data-input ports or dead
  input high bits.
- The graph-first / interleaved story is now factually documented at
  the CLI, test, user-guide, roadmap, and book levels.
- The associative-opportunity metric now matches the live semantic
  policy for strict duplicate preservation.

**Files touched**

- `src/gen/cone.rs`
- `src/gen/module.rs`
- `src/metrics.rs`
- `src/main.rs`
- `src/config.rs`
- `src/ir/types.rs`
- `src/ir/compact.rs`
- `tests/pipeline.rs`
- `USER_GUIDE.md`
- `ROADMAP.md`
- `DEVELOPMENT_NOTES.md`
- `CODEBASE_ANALYSIS.md`
- `book/src/knobs.md`

---

## 2026-04-17-0074 — All-constant evaluation completes the constant-fold surface

**Landed as:** `30753c8`

**What changed**

Closes the remaining all-const evaluation gaps flagged in the
last two slices. The factorization pipeline now evaluates every
pure-function gate at intern time when every operand is a
constant of the expected width.

### `Module::fold_constants` — extended

- **All-const associative evaluation** (Layer 5): for
  `And`/`Or`/`Xor`/`Add`/`Mul` with every operand a same-width
  constant, compute the result directly (bitwise AND / OR / XOR
  over values, sum / product mod 2^width) and intern the
  resulting constant. Inserted before the existing absorbing
  and identity-drop branches so it supersedes them for the
  all-const subcase — `Add(3, 5)` folds to 8 directly instead
  of going through identity-drop (which wouldn't have worked
  anyway since 5 isn't the identity).
- **All-const Sub / Shl / Shr**: for the existing 2-arity
  non-commutative arm, added upfront all-const evaluation that
  handles `Sub(c1, c2) → (c1 - c2) mod 2^width`,
  `Shl(c1, c2) → (c1 << c2) mod 2^width` (over-shift → 0), and
  `Shr(c1, c2) → c1 >> c2` (over-shift → 0). For Shl/Shr the
  shift-amount constant can have its own narrower width — we
  read the value and only require the lhs to match the gate
  width.

The existing identity-drop + absorbing paths remain for mixed
operand lists (one constant + one primary input / flop Q). This
is intentional and desirable: those paths catch valuable
partial folds even when the expression isn't fully constant.

### `Module::apply_peephole` — extended

- **`Concat([c1, c2, ...]) → assembled const`** when every
  operand is a constant. MSB-first bit assembly matching the SV
  emit convention in `src/emit/sv.rs` — `{c1, c2, c3}` packs
  `c1` into the high bits. Widths must sum to the gate width;
  any mismatch defensively skips the fold rather than emit a
  wrong-width constant.

### Counters

Both helpers keep reusing the existing counters
(`Module::fold_identities_applied` and
`peephole_rewrites_applied`) — all-const evaluation fires in
the same helper, so the count aggregates with the existing
rules for that helper.

### Tests

Eight new unit tests in `src/ir/types.rs`:

- `fold_all_const_add_evaluates`
- `fold_all_const_mul_wraps_modulo_width` (verifies 8-bit
  `Mul(100, 3)` → 44, i.e. 300 mod 256)
- `fold_all_const_xor_evaluates`
- `fold_all_const_sub_evaluates` (both positive and
  wrap-around cases)
- `fold_all_const_shl_evaluates_and_clamps` (over-shift clamps
  to zero)
- `fold_all_const_shr_evaluates`
- `peephole_concat_of_constants_assembles_msb_first`
  (`{4'hA, 4'h5}` → 8'hA5)
- `peephole_concat_of_constants_variadic`
  (`{3'b101, 2'b01, 1'b1}` → 6'b101011)

### Docs

- `book/src/factorization.md`: Layer 3 (constant folding) rule
  table split into associative + non-commutative sub-tables
  and extended with the all-const columns. Peephole's Concat
  bullet updated with the bit-assembly rule.
- `book/src/structural-rules.md` Rule 21c: `constant-fold` and
  `peephole` level-table rows extended to list the new rules.
- `src/ir/types.rs`:
  [`Module::fold_constants`] Rustdoc rule tables split into
  associative (with All-const / Identity-drop / Absorbing
  columns) and non-commutative (with All-const / Rhs-zero
  identity columns). `apply_peephole` Rustdoc gains the
  Concat all-const bullet and the
  `peephole_rewrites_applied` field comment lists it.

**Why**

The previous slices (`2de8855` peephole Not/Slice/reductions,
`5f51c3b` associative flattening) flagged two known gaps:
`Concat(all-const)` and `Shl/Shr(const, const)`. Plus one gap
surfaced while writing unit tests for the Slice slice —
`Add(3, 5)` and similar fully-constant expressions weren't
evaluated (the existing absorbing + identity-drop paths only
fire when a specific absorbing/identity value is present).
This slice closes all three in one pass under the common
framing "evaluate the gate at intern time when every operand
is a constant".

After this slice, the `NodeId = expression identity` contract
holds for every syntactically-or-algebraically-evaluable
expression: every all-constant expression collapses to a
single constant node; every same-op-same-width nesting
flattens; every commutative-reordering is canonicalised;
every identity / absorbing / peephole identity collapses.
What remains genuinely unaddressed is **cross-gate symbolic
rewrites** over non-constant expressions (`(a + b) - b → a`,
`(a & b) | (a & ~b) → a`, etc.), which is the e-graph
problem — research-adjacent and still aspirational.

**Empirical (seed 42, default knobs):**

Counters unchanged at seed 42 (default construction path
rarely produces all-constant operand lists directly). The new
rules fire whenever earlier folds / peepholes produce constant
intermediates that flow into these shapes — same pattern as
the Not/Slice/reduction slice.

**Tests**
- 70 unit tests pass (was 62 — added 8).
- 22 integration tests pass (unchanged).
- Total test count: 92.
- `cargo build` clean; `mdbook build book` clean.

**Impact**
- The "constants flow through pure operators" story is now
  complete for every operator class. Pipelines that construct
  a constant-heavy shape via CSE / fold chains collapse all
  the way to a single constant.
- Rule 21c level table and `factorization.md` reflect every
  implemented rule — no known gaps remain in the
  documentation.

---

## 2026-04-17-0073 — Thorough docs pass on factorization pipeline (docs only)

**What changed**

User asked for the `apply_peephole` / `fold_constants` /
`flatten_associative` / `compact_node_ids` / knob-telemetry
surface to be thoroughly documented, both in-code and in the
book. This slice closes the gap.

### In-code Rustdoc upgrades (`src/ir/types.rs`)

- `Module::intern_gate` — rewrote from a one-paragraph note
  into a full pipeline-orchestrator spec: returns convention,
  numbered six-step pipeline (Associative → Commutative → Fold
  → Peephole → None bypass → CSE), orphan-safety contract,
  determinism guarantee.
- `Module::intern_constant` — expanded to cover the dedup key,
  cap semantics, why no factorization layers apply, and the
  `make_constant` wrapper pattern.
- `Module::fold_constants` — added a table of all rules by op
  class, the absorbing-rule orphan-safety restriction, the
  non-commutative position-sensitivity note, the explicit
  out-of-scope list.
- `Module::flatten_associative` — upgraded header to call out
  Layer-4 / `FactorizationLevel::Associative` framing and
  clarified the return convention.
- `Module::apply_peephole` — **rewrote the stale doc** (listed
  only the original four rules and missed every rule added
  since). New doc groups rules by outer operator: Not (const
  eval, involutive, comparison inversion), comparisons
  (all-const eval), Slice (full-width identity + const eval),
  Concat (single-operand identity), reductions (const eval).
  Counter mechanism documented.
- `KnobRollCounters::record` — gained a doc comment explaining
  when it's called (`roll_knob` helper in cone.rs) and how to
  interpret `fires / attempts`.
- `Module::factorization_level` field — stale "Default `Full`"
  replaced with the real default (`EGraph`) and the ladder
  listing.
- `Module::peephole_rewrites_applied` field — stale list
  (missed Not(cmp) inversions, all-const Not/Slice/reduction
  evaluations) replaced with the complete catalogue and a
  pointer to Rule 21c.

### New book chapter: `book/src/factorization.md`

A dedicated "How It Works" chapter walking through the full
`intern_gate` pipeline end-to-end. Sections:

1. Why factorize (doctrinal anchor: NodeId = expression identity)
2. The ladder (enum layers + selection)
3. Pipeline in execution order (layers 1–6 with per-layer
   tables, short-circuit semantics, and reasoning)
4. Orphan safety and the compaction pass
5. Empirical counters with a seed-42 baseline table
6. Turning layers off — paste-and-run knob-sweep recipe
7. Pointers to related chapters and source

Progressive disclosure per the book doctrine: Rule 21c in
`structural-rules.md` stays as the rule-catalogue entry;
`factorization.md` is the narrative walkthrough for readers
who want to understand what anvil does to every gate.

Added to `book/src/SUMMARY.md` under "How It Works".

### `book/src/ir.md` updates

- `Module` struct listing refreshed to include every current
  field: `operand_duplication_rate`, `factorization_level`,
  block counters, fold/peephole/flatten/compaction counters,
  `knob_rolls`.
- "Node construction" section rewritten with three subsections:
  CSE semantics (Rule 21), The full intern pipeline (Rule 21c
  with numbered layer list + pointer to `factorization.md`),
  Orphan safety via compaction. Snapshot contract kept as the
  final subsection.

### `book/src/architecture.md` updates

- Crate layout refreshed: `types.rs` description now mentions
  all current fields and helpers; new entry for
  `compact.rs` that was missing entirely.
- "Key types at a glance" `Module` block extended to show the
  factorization counters + `knob_rolls`. `intern_gate`
  signature gains a layered-pipeline doc comment. New entries
  for `compact_node_ids`, `KnobId`, `KnobRollCounters`.

### Doctrinal consistency

Every public / `pub(crate)` factorization function now has a
doc comment that (a) names its layer position, (b) lists its
rules, (c) documents its return convention, (d) cross-links
the book chapter where applicable. The book chapters
cross-link back to the source. No more "what does this do" for
a user reading `-- --help` output plus the book.

**Why**

User directive: "all these functions and every else shall be
thoroughly documented when they are part of the user facing
surface. Not only that, the book shall contain the accurate
description of all these internal algorithms and functions."
Prior slices landed the functionality but inherited stale or
terse doc strings; this slice catches up.

**Tests**
- 62 unit tests pass (unchanged — docs-only slice).
- 22 integration tests pass (unchanged).
- Total test count: 84.
- `cargo build` clean; `mdbook build book` clean.

**Impact**
- Readers of `cargo doc --open` now get a complete
  factorization-pipeline reference via the module-level
  summaries on `intern_gate` and the layer helpers.
- Readers of the book get a dedicated chapter that explains
  the pipeline rather than having to assemble the picture
  from Rule 21c, architecture.md, and ir.md.
- Future slices that add a layer (or promote an aspirational
  one) have a clear template: update the `intern_gate`
  pipeline list, Rule 21c table, `factorization.md` layer
  list, and the per-field counter doc.

---

## 2026-04-17-0072 — Peephole all-const evaluation: Not, Slice, reductions

**What changed**

Extended `Module::apply_peephole` with four new constant-folding
rules that extend the "evaluate at intern time when all operands
are constants" pattern (previously restricted to comparisons):

- **`Not(c)`** → `~c & mask(width)`. Handled first in the `Not`
  arm, before the `Not(Not)` and `Not(cmp)` cases.
- **`Slice(hi, lo)(c)`** → `(c >> lo) & mask(hi - lo + 1)`.
  Added to the existing `Slice` arm alongside the full-width
  identity rule.
- **`RedAnd(c)`** → `(c == all_ones(src_width)) as 1-bit`.
- **`RedOr(c)`** → `(c != 0) as 1-bit`.
- **`RedXor(c)`** → `popcount(c) & 1` as 1-bit.

All three reductions share a new arm matching
`GateOp::RedAnd | GateOp::RedOr | GateOp::RedXor` with
`operands.len() == 1`. Width invariants: reductions always
produce 1-bit output regardless of operand width.

Fires share the existing `peephole_rewrites_applied` counter.
Constants folded by these rules are orphan-safe (the outer
unary op never materialises; the inner constant operand may be
unreferenced but `compact_node_ids` only tracks Gate orphans,
not Constant orphans).

Tests:
- `src/ir/types.rs`: three new unit tests —
  `peephole_not_of_constant_folds`, `peephole_slice_of_constant_folds`,
  `peephole_reductions_of_constants_fold`. Plus the previous
  slice's `peephole_not_eq_of_constants_folds_to_bit` is
  upgraded to `peephole_not_eq_of_constants_folds_end_to_end`
  — it now asserts the full `Not(Eq(5, 7)) → 1'b1` collapse
  rather than stopping at the Eq fold (the boundary it noted
  as an outstanding gap).

Docs:
- `book/src/structural-rules.md` Rule 21c `peephole` row
  expanded to list all five new constant-evaluation rules
  alongside the existing ones. "Constant evaluation
  (all-operand-constants → evaluated constant)" framing
  groups them cleanly.

**Why**

Closes a gap noted in the previous slice: `Not(Eq(c1, c2))`
rewrites via the comparison-inversion rule to `Neq(c1, c2)`,
which folds to a 1-bit constant. But `Not(already_folded_const)`
was left as a real Not gate because `Not(const)` wasn't wired.
Same pattern for any path where ConstantFold or a peephole
produces a constant that flows into a Not / Slice / reduction.

The slice generalises the existing all-const-comparison-fold
pattern to the remaining unary and unary-like gates, completing
the "constants flow through pure unary operators" story at
intern time. `Concat(all-const)` and `Shl/Shr(const, const)`
remain as known gaps — they're slightly more involved (width
accounting for Concat, shift-amount clamping for shifts) and
deferred to follow-ups.

**Empirical (seed 42, default knobs):**
- `peephole_rewrites_applied`: unchanged at 31 (none of the
  new patterns arise at default knobs on this seed — it takes
  a constant to flow directly into a Not/Slice/reduction,
  which is rare in current construction). The rules still fire
  in targeted unit tests and will activate whenever ConstantFold
  produces a constant that flows into one of these unary gates.
- Other metrics unchanged.

**Tests**
- 62 unit tests pass (was 59 — added 3 new + 1 upgraded).
- 22 integration tests pass (unchanged).
- Total test count: 84.
- `cargo build` clean; `mdbook build book` clean.

**Impact**
- Default-config output is unchanged at seed 42, but any
  module where a constant reaches a unary op via some
  sequence of folds now avoids the literal `~`, `[3:0]`, or
  `&`/`|`/`^` reduction wrapper around the constant in SV.
- Conceptual gap closed: the peephole layer now handles
  all-const evaluation for every operator class except Concat
  and Shl/Shr.

---

## 2026-04-17-0071 — Cross-gate peephole: Not(comparison) → inverted comparison

**What changed**

Extended `Module::apply_peephole`'s `Not` arm from a single-gate
rewrite (`Not(Not(x)) → x`) to cover six cross-gate comparison
inversions:

| Inner op | Rewrite      |
|----------|--------------|
| `Eq`     | `→ Neq`      |
| `Neq`    | `→ Eq`       |
| `Lt`     | `→ Ge`       |
| `Gt`     | `→ Le`       |
| `Le`     | `→ Gt`       |
| `Ge`     | `→ Lt`       |

When `intern_gate(Not, [cmp_gate_id], 1, deps)` sees its single
operand is a 1-bit comparison gate, it interns the inverted
comparison through the normal pipeline (CSE, constant fold,
etc.) and returns that NodeId directly. The original inner
comparison becomes orphaned (its only referencing call was the
outer `Not`, which collapsed); the post-construction
`compact_node_ids` pass cleans it up at module finalisation.

No new counters — fires share the existing
`peephole_rewrites_applied` counter.

Implementation detail: the `Not` arm of `apply_peephole` now
extracts `(op, operands, width, deps)` from the inner gate into
owned values before touching `self.intern_gate` recursively,
because holding an immutable borrow of `self.nodes[...]` across
a `&mut self` call would alias `self`.

Tests:
- `src/ir/types.rs`: three new unit tests —
  `peephole_not_eq_becomes_neq` (happy path),
  `peephole_not_comparison_inversions` (sweep over the five
  remaining rewrites), `peephole_not_eq_of_constants_folds_to_bit`
  (boundary: Not of a folded-to-const comparison stays as a Not
  on a constant — we don't wire Not-of-const into the pipeline
  here, that's ConstantFold's domain).

Docs:
- `book/src/structural-rules.md` Rule 21c `peephole` row
  expanded with the six inversions, kept in the same
  "orphan-safe via compaction" framing as the other peephole
  rules. Broader cross-gate rewrites like `(a + b) - b → a`
  remain flagged as e-graph work.

**Why**

First concrete step toward the `EGraph` ceiling (cross-gate
semantic equivalence). The inversions are narrow, unambiguous,
and rely on the same compaction infrastructure that enabled
`Not(Not(x))` in slice `2cd8b7a`. Picking this slice over broader
e-graph work keeps the deliverable well-scoped and empirically
measurable.

**Empirical (seed 42, default knobs):**
- `peephole_rewrites_applied`: 9 → **31** (+22 Not(cmp) fires)
- `nodes_compacted`: 94 → **96** (only +2 new orphans because
  comparison gates are usually shared via CSE and remain
  reachable from other consumers post-inversion)

**Tests**
- 59 unit tests pass (was 56 — added 3 new cross-gate tests).
- 22 integration tests pass (unchanged).
- Total test count: 81.
- `cargo build` clean; `mdbook build book` clean.

**Impact**
- Default-config output emits `neq`/`ge`/`le`/`gt`/`lt`
  operators directly where the RTL semantically calls for a
  negated comparison, instead of `!(eq/lt/...)`. Downstream
  tools see the canonical operator form.
- Infrastructure pattern established for future cross-gate
  peepholes: read inner gate into owned values, call
  `intern_gate` recursively, trust compaction for orphan
  cleanup.

---

## 2026-04-17-0070 — Associative flattening factorization layer goes live (Rule 21c layer 4)

**What changed**

Factorization ladder:
- `src/config.rs` promotes `FactorizationLevel::Associative`
  from aspirational to implemented. `is_implemented()` now
  covers it. `effective()` walker still handles out-of-order
  activation correctly; default `EGraph` walks down to
  `Peephole` (the highest rung) which transitively enables
  `Associative` and all lower layers.
- `src/ir/types.rs` adds `Module::flatten_associative`,
  dispatched from `intern_gate` **before** commutative sort
  and constant fold. For associative ops
  (`And`/`Or`/`Xor`/`Add`/`Mul`), it scans operands for any
  same-op same-width inner gate and splices its operands into
  the outer operand list. Per-op semantic normalisation after
  the splice:
  - **`And` / `Or`**: dedup (idempotent — `a & a = a`).
  - **`Xor`**: pair-cancel (self-inverse — `a ^ a = 0`). Count
    occurrences, drop even-count operands entirely, keep one
    copy of each odd-count operand.
  - **`Add` / `Mul`**: skip the flatten when flattening would
    produce duplicates AND `operand_duplication_rate < 1.0`.
    Preserves both the Rule 8 uniqueness contract and the
    `x + x = 2x` / `x * x = x²` semantics (dropping duplicates
    here would silently change arithmetic).

Short-circuits match the other intern-time helpers: post-
normalisation an empty operand list returns the op's identity
constant (only reachable for `Xor`-all-cancel → zero); a
single survivor returns that operand's NodeId directly; ≥ 2
operands overwrite the caller's operand list and intern
proceeds normally.

Live counter:
- `Module::flatten_associative_applied: u64` increments on
  each fire. Surfaced via `Metrics::flatten_associative_applied`.

Canary flipped:
- `tests/pipeline.rs`
  `nested_associative_opportunities_exist_today` (which
  previously asserted `> 0` to verify the layer hadn't landed
  yet) renamed to `nested_associative_opportunities_flatten_to_zero`
  and now asserts `== 0` at default knobs. This is the direct
  doctrine check that every post-construction IR is free of
  remaining associative-flattening opportunities. Complements
  `flatten_associative_applied` — the former is the
  post-construction state, the latter is the event count.

Tests:
- `src/ir/types.rs`: four new unit tests
  (`flatten_associative_splices_same_op`,
  `flatten_associative_and_dedups`,
  `flatten_associative_xor_pair_cancels`,
  `flatten_associative_xor_all_cancel_to_zero`,
  `flatten_associative_add_skips_on_duplicates`) covering the
  splice mechanics and per-op normalisation.

Docs:
- `book/src/structural-rules.md` Rule 21b: ladder prose +
  syntactic-vs-semantic framing now include `associative` as
  live, citing structural identities like
  `Add(a, Add(b, c)) = Add(a, b, c)` and `a ^ a = 0`.
- `book/src/structural-rules.md` Rule 21c: level table entry
  for `associative` promoted to a concrete description of the
  splice + per-op normalisation; "Doctrinal anchor" paragraph
  lists `associative` alongside the other implemented layers.
  `highest_implemented` prose updated: no more "skipping the
  not-yet-live associative rung".
- `book/src/non-triviality.md`: rewritten "NodeId compaction"
  paragraph to acknowledge its new role as enabler for
  associative flattening, followed by a dedicated paragraph on
  the Associative layer with the per-op semantics and the
  `nested_associative_operand_count = 0` empirical validation.
  The aspirational-layer list narrows to "cross-gate
  identities → e-graph".

**Why**

Layer 4 of the factorization ladder, enabled by the NodeId
compaction pass from the previous slice. Previously-deferred
because of the orphan-safety problem: splicing `Add(b, c)` into
`Add(a, ...)` leaves the inner `Add(b, c)` unreferenced. Now
compaction removes it at finalisation, so the rewrite is legal.

With this slice, the `NodeId = expression identity` doctrine
holds for every case where **syntactic identity after associative
normalisation** is sufficient: `Add(a, Add(b, c))`,
`Add(Add(a, b), c)`, and `Add(a, b, c)` all produce the same
NodeId at default knobs. The only residual divergence is for
semantically-equivalent-but-structurally-different expressions
(`(a + b) - b = a`, `(a & b) | (a & ~b) = a`, `a + 2 - 1 = a + 1`),
which are the e-graph / deeper peephole domain.

**Empirical (seed 42, default knobs):**
- `nested_associative_operand_count`: 0 (was 373 pre-slice)
- `flatten_associative_applied`: 268
- `nodes_compacted`: 94 (was 7 — jump driven by Associative
  orphaning inner gates at splice time)
- `fold_identities_applied`: 91 (was 28 — more ConstantFold
  opportunities opened up by flattening)
- `peephole_rewrites_applied`: 9 (unchanged)

**Tests**
- 56 unit tests pass (was 51 — added 4 new associative tests +
  a minor rewrite of the existing test cluster).
- 22 integration tests pass (unchanged in count; the canary
  test was renamed and its assertion flipped).
- Total test count: 78.
- `cargo build` clean; `mdbook build book` clean.

**Impact**
- Default-config output contains zero nested associative
  shapes: every `Add`/`Or`/`And`/`Xor`/`Mul` tree is fully
  flattened into its maximum-arity operator form. Downstream
  synthesis / formal tools see the canonical shape instead of
  random nesting.
- ConstantFold's identity-drop reach expands: flattened lists
  that happen to contain an identity constant now collapse
  correctly (previously the constant was hidden inside an
  inner gate).
- `factorization_level` dial is consistent: `associative` no
  longer silently degrades to `commutative` via the walker.
  Only `e-graph` (the theoretical ceiling) remains aspirational.

---

## 2026-04-17-0069 — NodeId compaction pass + Not(Not(x)) peephole unlock

**What changed**

New post-construction compaction pass: `src/ir/compact.rs` adds
`compact_node_ids(&mut Module) -> u32`. BFS from all roots
(drives, flop.d, flop.q, flop.mux.sel/data, flop.mux.arms) marks
reachable nodes; unreachable ones are removed and `m.nodes` is
rewritten with an old→new `NodeId` map. Every NodeId holder is
remapped in place: `m.nodes[*].operands`, `m.drives`, `flop.d`,
`flop.q`, `FlopMux::OneHot(arms)`, `FlopMux::Encoded { sel,
data }`. Dedup tables (`gate_instances`, `const_instances`)
are rebuilt under the new NodeId space — entries whose targets
were unreachable are dropped; surviving ones are remapped.
Topological order is preserved by walking old indices in
ascending order.

Integration:
- `src/gen/module.rs` `generate_leaf_module` calls
  `compact_node_ids` after `drain_flop_worklist`. Removed count
  stored on `Module::nodes_compacted`. Orphan-audit warning now
  fires only if compaction left orphans (indicating a BFS or
  holder-enumeration bug).

Peephole unlock:
- `Not(Not(x)) → x` re-enabled in `Module::apply_peephole`. The
  previous slice (`88c268d`) disabled it because it orphaned the
  inner `Not`. Compaction makes the rewrite safe — the inner gate
  is removed at module finalisation.

Metrics:
- `Module::nodes_compacted: u32` and `Metrics::nodes_compacted`
  surface the removed count. Zero when every rewrite happens to
  be orphan-safe; non-zero (seed-42: 7) when Not(Not) fires.

Tests:
- `src/ir/compact.rs`: 3 unit tests — no-op on clean IR, removes
  injected orphan gate, preserves topological order.
- `src/ir/types.rs`: reinstated
  `peephole_double_not_collapses_with_inner_orphaned` (asserts
  the inner Not is left in place at intern time; compaction is
  a separate concern).
- `tests/pipeline.rs`: new
  `compaction_preserves_rule_18_and_records_removals` — across
  40 seeds at default knobs, asserts (a) zero orphan gates
  post-compaction, (b) validator accepts post-compaction IR,
  (c) total `nodes_compacted > 0` (i.e. Not(Not) actually fires).

Docs:
- `book/src/structural-rules.md` Rule 21c: peephole row updated
  to include `Not(Not(x)) → x` with a note about the compaction
  pass. Cross-gate rewrites (`(a + b) - b → a`) still flagged
  as deferred.
- `book/src/non-triviality.md`: new paragraph describing the
  compaction pass, its role in enabling orphan-tolerant
  rewrites, and the path to Associative (which needs
  intern-time merge logic on top of compaction).

**Why**

Compaction is the architectural prerequisite for Associative
flattening (Layer 4) and the deferred `Not(Not(x))` peephole
rule. Landing it now — together with re-enabling Not(Not(x)) as
the first concrete consumer — keeps the slice tied to observable
output rather than being pure infrastructure. Associative stays
deferred for a follow-up (the intern-time merge logic is
independent work).

**Empirical (seed 42, default knobs):**
- `peephole_rewrites_applied`: 9 (was 2 before Not(Not) re-enable)
- `nodes_compacted`: 7 (→ 7 of 9 peephole fires were Not(Not);
  the other 2 are constant-comparison / Slice / Concat, which
  don't orphan)

**Tests**
- 51 unit tests pass (was 47 — added 3 compact tests + 1 restored
  peephole test).
- 22 integration tests pass (was 21 — added
  `compaction_preserves_rule_18_and_records_removals`).
- Total test count: 73.
- `cargo build` clean; `mdbook build book` clean.

**Impact**
- Rule 18 (zero orphans) holds post-finalisation, as before —
  but now via a construction-plus-compaction pipeline rather
  than construction-alone. Orphan-tolerant rewrites become
  legal.
- `Not(Not(x))` now collapses at intern time everywhere it
  arises via CSE chains — downstream output is slightly smaller
  and cleaner.
- Infrastructure in place for Associative flattening (next
  factorization-layer slice) and future cross-gate peephole
  rewrites that would otherwise leave intermediate gates
  orphaned.

---

## 2026-04-17-0068 — Per-knob probability-roll counters (attempts / fires) live

**What changed**

Live per-knob telemetry for every probability-roll knob. New
`KnobId` enum in `src/ir/types.rs` with one variant per
`gen_bool(cfg.<prob>)` site (10 total). New
`KnobRollCounters` struct on `Module` tracking
`attempts: HashMap<KnobId, u64>` and
`fires: HashMap<KnobId, u64>`. New helper `roll_knob(g, m,
knob, prob)` in `src/gen/cone.rs` replaces all 25
`gen_bool(cfg.<prob>)` sites; it rolls, records the outcome,
and returns the bool.

Knobs instrumented:
- `flop_prob`
- `comb_mux_prob`
- `priority_encoder_prob`
- `coefficient_prob`
- `const_shift_amount_prob`
- `const_comparand_prob`
- `comb_mux_encoding_prob`
- `flop_mux_encoding_prob`
- `share_prob`
- `flop_qfeedback_prob`

Surfaced via new `Metrics` fields:
- `knob_roll_attempts: BTreeMap<String, u64>`
- `knob_roll_fires: BTreeMap<String, u64>`
(converted from `KnobId` → canonical string name via
`KnobId::name()` at `compute()` time).

Tests:
- New integration test
  `knob_rolls_recorded_across_seeds` in `tests/pipeline.rs`:
  across 20 seeds at default knobs, every one of the 10
  expected knobs must appear in `knob_roll_attempts` with
  `attempts > 0` and `fires <= attempts`. Catches regressions
  where a knob stops being consulted or its roll site becomes
  unreachable.

Docs:
- `book/src/knobs.md` "Knob effectiveness map" gains a new
  "Per-knob roll-rate validation" subsection explaining the
  empirical-fire-rate test (`fires / attempts` should track
  the configured probability), with concrete seed-42 numbers.

**Why**

Completes the measurability doctrine for every probability
knob: the effect is now a simple division away. Previously
only *shape* metrics (`num_flops`, `num_muxes_2to1`, etc.)
measured these knobs — useful but indirect (they conflate the
roll rate with the number of reachable roll sites). The new
ratio `knob_roll_fires[k] / knob_roll_attempts[k]` is a
direct check.

Picked as the next slice because `Associative` and the
deeper peephole rules are both blocked on NodeId compaction —
a larger architectural slice. This slice is a clean
well-scoped completion of the measurability goal, with no
risk of orphan cascades.

**Empirical spot-check (seed 42, default knobs):**

| Knob                      | Default | attempts | fires | ratio |
|---------------------------|---------|----------|-------|-------|
| `share_prob`              | 0.30    | 1999     | 607   | 0.304 |
| `comb_mux_encoding_prob`  | 0.50    | 94       | 49    | 0.521 |
| `coefficient_prob`        | 0.20    | 256      | 51    | 0.199 |
| `const_shift_amount_prob` | 0.75    | 55       | 40    | 0.727 |
| `flop_qfeedback_prob`     | 0.50    | 34       | 15    | 0.441 |
| `comb_mux_prob`           | 0.10    | 1010     | 94    | 0.093 |
| `flop_prob`               | 0.10    | 261      | 34    | 0.130 |

All ratios track their configured values within sampling
noise — the telemetry is faithful.

**Tests**
- 47 unit tests pass (unchanged).
- 21 integration tests pass (was 20 — added
  `knob_rolls_recorded_across_seeds`).
- Total test count: 68.
- `cargo build` clean; `mdbook build book` clean.

**Impact**
- Every probability knob now has a direct empirical check
  on its effective rate — the measurability doctrine's
  strongest form.
- `manifest.json` / `--metrics` dumps gain two new maps per
  module, keyed by knob name. Consumers can build sweep
  scripts that assert `|empirical - configured| < ε` across
  many seeds and flag regressions.
- No behavioural change to the generator: `roll_knob` is
  byte-identical to the previous `g.rng.gen_bool(...)` on
  the output path, only adds the counter record. Verified
  by reproducibility tests (byte-identical output across
  `--trace` levels, which includes this change).

---

## 2026-04-17-0067 — Peephole factorization layer goes live (Rule 21c layer 6), orphan-safe subset

**What changed**

Factorization ladder:
- `src/config.rs` promotes `FactorizationLevel::Peephole` from
  aspirational to implemented. `is_implemented()` now covers
  `Peephole`; default `EGraph` walks down to `Peephole` as the
  effective layer (was `ConstantFold`).
- `src/ir/types.rs` adds `Module::apply_peephole`, dispatched
  from `intern_gate` after `fold_constants`. Rules wired —
  each one is narrow and orphan-safe:
  - **Fully-constant comparisons**: `Eq`/`Neq`/`Lt`/`Gt`/`Le`/
    `Ge` with both operands same-width constants are evaluated
    at intern time to a 1-bit constant. Constants orphaned as
    a side effect don't count as gate orphans, so the rule is
    Rule-18-safe.
  - **Full-width `Slice`**: `Slice(hi, 0)` with
    `hi + 1 == src_width` returns the source NodeId. The
    source is used by the caller — no orphan.
  - **Single-operand `Concat`**: `Concat([x]) → x`. Same
    orphan-safety.

Orphan-safety hardening in ConstantFold's absorbing rule:
- Previous slice's absorbing rule (`x * 0 → 0`, `x & 0 → 0`,
  `x | all_ones → all_ones`) would orphan any Gate operand —
  the outer gate collapses to a constant and the Gate
  operand's only consumer (this call) disappears. The peephole
  slice's RNG-path shift exposed the latent orphan: a pre-
  existing build-Eq-gate, later consumed by an `And([eq, 0])`
  which absorbed to 0, left the Eq unreferenced.
- Fix: absorbing now fires only when **no operand is a Gate**
  — i.e. every operand is a Constant, PrimaryInput, or FlopQ.
  This restricts absorbing to the "evaluate all-constant
  expression" subset, which is strictly orphan-safe. Dynamic
  absorbing (`x & 0` where `x` is a Gate sub-tree) now waits
  for the compaction-equipped future layer.

Rule NOT implemented and why:
- **`Not(Not(x)) → x`** — would orphan the inner `Not` gate
  because the outer `Not` call's only reference to inner
  (the operand) disappears after the rewrite returns `x`
  directly. Without NodeId compaction this is a Rule 18
  violation. Documented in `apply_peephole` doc comment +
  Rule 21c table. Waits for the e-graph / compaction layer.

Live counter:
- `Module::peephole_rewrites_applied` (u64) increments on
  each fire. Surfaced via `Metrics::peephole_rewrites_applied`.

Tests:
- `src/ir/types.rs`: four new peephole unit tests
  (`peephole_constant_comparison_evaluates`,
  `peephole_full_width_slice_identity`,
  `peephole_single_operand_concat_identity`,
  `peephole_disabled_below_peephole_level`). The previously-
  written `peephole_double_not_collapses` test was dropped
  alongside its rule.
- `tests/pipeline.rs`: new integration test
  `peephole_layer_fires_at_default_knobs` — sums
  `peephole_rewrites_applied` over 40 seeds at default knobs
  and asserts > 0.

Docs:
- `book/src/structural-rules.md` Rule 21b: factorization-
  ladder prose updated to list peephole as live; syntactic-
  vs-semantic framing extended with the peephole identities.
- `book/src/structural-rules.md` Rule 21c: level table
  entry for `peephole` promoted from "Not implemented yet"
  to the concrete rule list, with a note about why
  `Not(Not(x)) → x` is deferred. ConstantFold entry
  updated to note the non-Gate-operand restriction on
  absorbing.
- `book/src/structural-rules.md` Rule 21c: `highest_implemented`
  prose updated `constant-fold` → `peephole`.
- `book/src/non-triviality.md`: "Factorization ladder"
  subsection updated; aspirational layer list now just
  `Associative` + `EGraph`, both noted as blocked on
  NodeId compaction.

**Why**

Layer 6 of 8 on the factorization ladder, picked over the
harder `Associative` layer (Layer 4) because peephole rules
land as construction-time short-circuits with no NodeId
compaction required — same architectural shape as ConstantFold.
The effective-level walker I added in slice `82b2213` already
handles the out-of-order activation (layer 6 live, layer 4
still aspirational). Peephole advances the ladder and
exposed+fixed the absorbing-rule orphan hazard that was latent
in ConstantFold.

The absorbing-rule restriction is the doctrinally-correct
call: Rule 18 (zero orphans) is a strict invariant, and
until we have a compaction pass, absorbing on Gate operands
can't be made orphan-safe. The restricted form (absorb only
when all operands are non-Gate) is a proper subset that still
legitimately fires — just less often than the unrestricted
form.

**Tests**
- 47 unit tests pass (was 49 after the slice; two Not(Not)
  tests + one `double_not` test removed; four new peephole
  tests added; net +3 from the prior slice's +6).
- 20 integration tests pass (was 19 — added
  `peephole_layer_fires_at_default_knobs`).
- Total test count: 67.
- `cargo build` clean; `mdbook build book` clean.

**Impact**
- Default-config output contains fewer trivial gates:
  fully-constant comparisons are evaluated at intern
  time, full-width slices disappear, single-operand
  concats disappear.
- `Metrics::peephole_rewrites_applied` gives empirical
  visibility into Peephole-layer activity.
- The absorbing-rule restriction tightens the Rule 18
  guarantee: no construction-time rewrite can orphan a
  Gate. This pushes `Not(Not(x)) → x` and
  dynamic-absorbing (`x & 0` with gate `x`) into the
  compaction-equipped future layer, preserving the
  strict-orphan-free doctrine today.

---

## 2026-04-17-0066 — ConstantFold factorization layer goes live (Rule 21c layer 5)

**What changed**

Factorization ladder:
- `src/config.rs` promotes `FactorizationLevel::ConstantFold`
  from aspirational to implemented. `is_implemented()` and
  `effective()` now walk the enum order top-down and skip
  any not-yet-live middle rungs: a request for `Associative`
  correctly drops to `Commutative`, while `EGraph` / default
  activates up to `ConstantFold`. The enum-order quirk that
  `Associative` sits between `Commutative` and `ConstantFold`
  is handled by the walker, not by reshuffling variants.
- `src/ir/types.rs` adds the fold dispatcher
  `Module::fold_constants` (called from `intern_gate` after
  commutative sort, before dedup). Rules wired:
  - **Absorbing**: `x & 0 → 0`, `x | all_ones → all_ones`,
    `x * 0 → 0` (returns a same-width constant via
    `intern_constant`).
  - **Identity drop**: `Add`/`Xor`/`Or` drop `0` operands,
    `Mul` drops `1` operands, `And` drops `all_ones`
    operands. Post-shrink: 0 operands → identity constant,
    1 operand → that operand's NodeId, ≥ 2 → caller
    continues with the shrunken list.
  - **2-arity Sub / Shl / Shr**: rhs-zero short-circuit
    (`a - 0 → a`, `a << 0 → a`, `a >> 0 → a`). The lhs-zero
    cases (`0 - a`, `0 << a`, `0 >> a`) are deliberately not
    folded — they're not algebraic identities.
  Comparison ops, reductions, `Not`, `Slice`, `Concat`, `Mux`
  are out of scope for this layer (they belong to `Peephole`).

Live counter:
- `Module::fold_identities_applied` (u64) increments on each
  fire. Surfaced via `Metrics::fold_identities_applied`,
  sourced directly from the per-module counter.

Pre-existing bugs exposed by fold and fixed in the same slice:

- `assemble_mul_linear_combination` didn't dedup the coefficient
  constant against its signal list — when coef == const_k and a
  signal happened to be the literal const_k (same NodeId via
  CSE), operands became `[c, c]`, tripping Rule 8 operand
  uniqueness. Fixed with a post-assembly dedup pass; single-
  operand residual returns directly.
- `make_mul` / `make_sub` lacked the `a == b` degeneracy guard
  that `make_and` already had. When CSE / fold collapsed two
  callers' ids into one, `make_mul(a, a)` hit the same
  duplicate-operand failure as above. Added guards mirroring
  `make_and`: `make_mul` short-circuits to `a` under strict
  operand-uniqueness; `make_sub` short-circuits to a zero
  constant (Sub is algebraically `x - x = 0`).
- `deliver`'s interleaved anti-collapse fallback used
  `operands[0]` as fallback for all ops, which works for
  gates whose operand width equals output width but BREAKS for
  comparisons: `Eq`/`Neq` output 1-bit but operand width is
  the comparand width K. When `violates_anti_collapse`
  flagged `Eq(a, a)` or `Neq(a, a)` during interleaved
  construction, delivering `operands[0]` (width K) into a
  slot expecting width-1 (the comparison output) yielded
  mismatched operand widths in the parent. Fixed with
  comparison-specific width-correct fallbacks:
  `Eq(a, a) → 1`, `Neq(a, a) → 0`. Mux, Sub, And/Or/Xor/Add/Mul
  cases unchanged since they already had the correct width.

Tests:
- `src/ir/types.rs`: five new unit tests covering fold
  identities (`fold_add_zero_collapses_to_x`,
  `fold_and_all_ones_collapses_to_x`,
  `fold_mul_zero_absorbs`, `fold_or_all_ones_absorbs`,
  `fold_miscellaneous_identities`) and a gating test
  (`fold_disabled_below_constant_fold_level`) that confirms
  the layer is inert at `FactorizationLevel::Commutative`.
- `tests/pipeline.rs`: new integration test
  `constant_fold_layer_fires_at_default_knobs` sums
  `fold_identities_applied` over 40 seeds at default knobs
  and asserts > 0 — a regression guard against the fold layer
  silently switching off (or `constant_prob` no longer
  producing identity-valued constants).

Docs:
- `book/src/structural-rules.md` Rule 21b: factorization-
  ladder prose updated to list constant-folding as live;
  syntactic-vs-semantic framing extended to cite the curated
  identities now caught at intern time.
- `book/src/structural-rules.md` Rule 21c: level table entry
  for `constant-fold` promoted from "Not implemented yet" to
  the concrete identity list, with a pointer to
  `Metrics::fold_identities_applied` for empirical
  measurement. Effective-level prose rewritten to document
  the walker semantics.
- `book/src/non-triviality.md` "Factorization ladder"
  paragraph: constant-folding added to the within-gate
  surface; aspirational layer list slimmed to
  `Associative`/`Peephole`/`EGraph`.

**Why**

Next rung on the factorization ladder (Layer 5 of 8). Picked
over `Associative` (Layer 4) because it's strictly simpler —
no NodeId compaction, no finalization pass, no dedup-table
rebuild — while still advancing the ladder and surfacing
latent bugs in adjacent code (the three fixed above). The
`Associative` rung stays aspirational for now with its
regression canary (`nested_associative_opportunities_exist_today`)
still in place; when that layer lands the canary flips to
`== 0`.

**Tests**
- 19 integration tests pass (was 18 — added
  `constant_fold_layer_fires_at_default_knobs`).
- 49 unit tests pass (was 43 — added 6 fold tests).
- Total test count: 68.
- `cargo build` clean; no warnings introduced.

**Impact**
- Default-config output contains fewer trivial-algebraic
  gates. Specifically: `x + 0`, `x * 1`, `x & all_ones`, and
  `x | 0` now disappear at intern time rather than landing
  as literal nodes. Downstream synthesis tools would fold
  these anyway; anvil now matches their view one step
  earlier.
- `Metrics::fold_identities_applied` exposes an empirical
  handle on how much work the fold layer does per seed /
  per module — useful for knob tuning (does
  `constant_prob` produce enough identity-valued literals
  to make fold meaningful? turns out yes at default).
- Three latent bugs in adjacent code paths (linear-comb
  dedup, make_mul degeneracy, comparison anti-collapse
  fallback width) landed fixes while I was there, each
  defensive against RNG-path shifts the fold layer
  introduced.

---

## 2026-04-17-0065 — Syntactic-vs-semantic-identity framing in the factorization-ladder narrative (docs only)

**What changed**
- `book/src/structural-rules.md` (Rule 21b, the "Position in the
  factorization ladder" paragraph): new follow-up paragraph
  making the syntactic-vs-semantic identity distinction
  explicit. What today's implemented layers guarantee is that
  **two syntactically identical expressions share one node**.
  The aspirational layers above extend the contract toward
  **two semantically equivalent expressions share one node** — a
  strictly harder problem that synthesis tools themselves solve
  incompletely.
- `book/src/non-triviality.md` (the "Factorization ladder"
  sub-section of "Algebraic residue"): same framing mirrored,
  tied to the local narrative about what anti-collapse rules
  catch and what they don't.

**Why**
A durable framing surfaced in the conversation: the contract
we actually ship today is *syntactic* identity; the goal is
*semantic* identity; the asymptote matters because synthesis
tools themselves solve semantic equivalence incompletely.
Recording the framing in the book makes it survive session loss
and sets reader expectations appropriately — neither overclaim
nor underclaim what the `NodeId = expression identity` doctrine
delivers in the current build.

**Tests**
- No code changed.
- 57 tests pass.
- `mdbook build book` succeeds.

**Impact**
- Readers learning about the factorization ladder now have a
  single-sentence summary of where we are vs where we aim, and
  an honest acknowledgment that the ceiling is an asymptote
  the synthesis industry itself hasn't closed.

---

## 2026-04-17-0064 — Regression tests pinning three doctrine-level invariants

**What changed**
- `tests/pipeline.rs` gains three integration tests:
  - **`zero_orphans_at_default_knobs`** — Rule 18 regression
    guard. Generates modules across all four strategy values ×
    6 seeds and asserts every `Node::Gate` has at least one
    consumer (gate operand, flop field, or output drive).
  - **`zero_duplicate_operands_at_default_knobs`** — Rule 8
    extended regression guard. At `operand_duplication_rate =
    0.0` (default), no `And`/`Or`/`Xor`/`Add`/`Mul` gate may
    have a duplicate `NodeId` in its operand list. Checked
    across 5 seeds.
  - **`nested_associative_opportunities_exist_today`** —
    informational guard. Asserts
    `nested_associative_operand_count > 0` at seed 42 today
    (Associative layer not implemented). When that layer lands,
    this test should flip to `== 0` as direct validation that
    flattening collapses the opportunity.
- `CODEBASE_ANALYSIS.md`: test count updated 54 → 57.

**Why**
Each of the three assertions captures a doctrine-level
invariant established in recent slices but not pinned by a
test:

- Rule 18 zero-orphans — enforced by build_cone
  snapshot/rollback + process_signal_frame existing-operand
  fallback. Slice `b78550d` validated manually across
  strategies × seeds; now a test catches regressions
  automatically.
- Rule 8 zero-duplicates at default — enforced by
  `violates_anti_collapse` + the post-assemble dedup in
  linear-combination + `make_and` idempotent short-circuit.
  Slice `9e18c89` drove the duplicate count to 0 at default;
  now regression-guarded.
- Associative-opportunity non-zero — direct complement to the
  metric added in `99084a8`. Serves as a canary: when the
  Associative layer lands and flips this to zero, the
  implementation is working.

**Tests**
- All four cargo gates green.
- **57 tests** pass (39 unit + 18 integration, +3 new).

**Impact**
- Future slices that break Rule 18 or the operand-uniqueness
  contract now fail CI instead of being spotted by manual
  `grep` audit.
- The associative-flattening regression test flipping direction
  is a simple, definite signal that the Associative layer has
  landed and works.

---

## 2026-04-17-0063 — Associative-flattening opportunity metric (informational, pre-implementation)

**What changed**
- `src/metrics.rs`: new `Metrics::nested_associative_operand_count:
  usize`. Post-hoc walk counts every operand slot on an associative
  gate (`And`/`Or`/`Xor`/`Add`/`Mul`) whose operand is itself a
  `Node::Gate` of the same op and width — i.e., a slot the
  not-yet-implemented `Associative` factorization layer would
  absorb.
- `book/src/knobs.md`: knob-effectiveness map gains an entry for
  `operand_duplication_rate` (previously missing) and extends the
  `factorization_level` entry with the new metric.
- `USER_GUIDE.md`: knob-effects bullet list gains an entry for
  the new metric.
- `CODEBASE_ANALYSIS.md`: `metrics.rs` one-liner extended.

**Why**
The factorization ladder has three implemented layers (CSE,
operand-uniqueness, commutative) and four aspirational ones
(Associative, ConstantFold, Peephole, EGraph). Before investing
in the full `Associative` implementation — which involves
non-trivial design (finalization pass vs construction-time;
NodeId compaction vs leaving orphans; pool coordination) — this
slice measures *how much flattening would actually happen*, so
the cost/benefit is data-driven rather than speculative.

**Tests**
- All four cargo gates green.
- 54 tests pass.
- `mdbook build book` succeeds.
- Seed sweep at default knobs:

  ```
  seed=1     num_gates=1999 nested_associative_operand_count=261 (13%)
  seed=42    num_gates=2368 nested_associative_operand_count=373 (16%)
  seed=100   num_gates=2311 nested_associative_operand_count=266 (12%)
  seed=777   num_gates=2861 nested_associative_operand_count=386 (13%)
  seed=9999  num_gates=20   nested_associative_operand_count=1   (5%)
  ```

  **10–16% of operand slots on associative gates would be
  absorbed by flattening.** Meaningful reduction target; the
  Associative slice is worth queuing.

**Impact**
- No behaviour change.
- Factorization-level effectiveness-map entry goes from
  qualitative ("`num_gates` shift across dial") to quantitative
  (concrete opportunity count).
- Data to justify (or postpone) the full Associative
  implementation.

---

## 2026-04-17-0062 — FAQ chapter refresh: strategies + full-factorization Q (docs only)

**What changed**
- `book/src/faq.md`:
  - "Why four construction strategies instead of just the default?"
    → "Why three". graph-first removed from the canonical list,
    retirement rationale + silent-alias behaviour noted with
    cross-link to the construction-strategies chapter. Interleaved
    described as the default.
  - "Can output J's cone reference a gate from output I's cone?"
    — stale `graph-first`-specific language replaced; added a
    mention that Rule 21 CSE makes the cross-cone identity
    automatic.
  - New entry: **"What does 'full factorization' mean in the
    book? Does `anvil` deduplicate expressions?"** Answers the
    user doctrine. Names the three implemented layers (CSE,
    operand uniqueness, commutative normalization) and the four
    aspirational layers (Associative, ConstantFold, Peephole,
    EGraph), with the `factorization_level` dial.

**Why**
FAQ chapter predated `graph-first` retirement (`b78550d`) and
the factorization-ladder work (`f425657`, `c9c2f98`, `d2aefba`,
`5a9b477`). A user landing on the FAQ first now sees the correct
strategy story and a direct answer to the "does anvil dedupe?"
question that the full-factorization doctrine prompts.

**Tests**
- No code changed.
- 54 tests pass.
- `mdbook build book` succeeds.

**Impact**
- Closes the book audit. Every authored chapter now reflects
  shipping code. A session that recovers cold from
  `SESSION_BOOTSTRAP.md`'s reading order won't find drift
  between the book's narrative and the generator's behaviour.

**Book audit completion status**

| Chapter | Status |
|---|---|
| introduction, getting-started, tutorial, recipes, knobs, construction-strategies, ir, algorithm, sequential, synthesizability, structural-rules, architecture, by-construction, non-triviality, sharing, faq | **Fresh** |
| hierarchy.md | Phase 4+ — intentionally placeholder |
| core-idea, non-goals, why-not-grammar | Doctrine — not casually edited |

---

## 2026-04-17-0061 — Sharing chapter refresh: Rule 2 + Rule 18 + Rule 21 CSE (docs only)

**What changed**
- `book/src/sharing.md`:
  - `try_share` description: removed the stale "Q-exclusion
    contract" reference; replaced with a pointer to Rule 2
    (Q-feedback freedom — Q is a free leaf inside its own
    D-cone; the clock edge breaks the Q→D loop temporally).
  - "Forbidden sharing patterns" section rewritten to match the
    current Rule 8 extended rule set: N-arity And/Or/Xor
    operand-multiset distinctness, 2-arity Sub/Eq/Neq
    degeneracy, Add/Mul gated on `operand_duplication_rate`,
    Mux gated on `mux_arm_duplication_rate`. Added a paragraph
    on the Rule 18 α snapshot-restore on rejection — rejected
    sub-trees don't orphan.
  - **New "Construction-time CSE (Rule 21)" section** replaces
    the old "What sharing does not do" paragraph. The old text
    said "does not deduplicate equivalent sub-expressions… CSE
    is the synthesizer's job" — this was reversed by slice
    `f425657`. New section explains that `intern_gate` dedupes
    by `(op, operands, width)`, with `max_ast_instances` cap
    knob, commutative sort at level ≥ `Commutative`, and
    articulates how per-operand `share_prob` and CSE compose
    (share_prob = early cut-off; CSE = identity of identical
    expressions).
  - Cross-output sharing section: "current sequential" +
    "graph-first will be the default" corrected. `interleaved`
    is default; `graph-first` retired as silent alias.
  - "No cycles possible" section retitled "No combinational
    cycles possible"; removed the stale Q-exclusion reference;
    added Rule 1 + Rule 2 cross-links and the explicit
    clock-edge-breaks-the-loop-temporally story.

**Why**
Per book doctrine. `sharing.md` predated four big changes:
- Rule 2 Q-feedback freedom (slice `6cbcbff`).
- Rule 8 extension (`3544a0c`).
- Rule 18 α enforcement (`b78550d`).
- Rule 21 CSE via intern_gate (`f425657`) + Rule 21b commutative
  normalization (`c9c2f98`).

The chapter's previous "sharing does not CSE" paragraph actively
contradicted the shipping code — the most misleading kind of
drift.

**Tests**
- No code changed.
- 54 tests pass.
- `mdbook build book` succeeds.

**Impact**
- Readers learning DAG sharing now see the full story:
  per-operand share/recurse coin **plus** construction-time CSE
  **plus** commutative normalization, and how the three compose.
- No more stale "Q-exclusion" references in the book (verified;
  this chapter was the last holdout).

---

## 2026-04-17-0060 — Non-triviality chapter: anti-collapse rule table + factorization-ladder framing (docs only)

**What changed**
- `book/src/non-triviality.md`:
  - Anti-collapse rules table (the heart of the chapter)
    rewritten to match actual `violates_anti_collapse` in
    `src/gen/cone.rs`. Old table listed rules that were never
    implemented (`a & 0`, `a | all_ones`, `a & all_ones`,
    `a | 0`, "minimum shift amount = 1") and missed the
    current reality (N-arity operand-multiset distinctness,
    `operand_duplication_rate` / `mux_arm_duplication_rate`
    gating). New table has five rows covering the actual
    implementation plus a paragraph on the factorization-level
    gating (rules relax at level `cse` / `none`).
  - New snapshot-restore note under the table: explains the
    Rule 18 α connection — on anti-collapse rejection,
    `build_cone` rolls back its pre-operand-construction
    snapshot so the rejected sub-tree doesn't orphan.
  - "Algebraic residue" section reframed. Old text: "the fix is
    to add a cheap canonicalizer". New text: "anvil has started
    climbing this ladder" — points at CSE / operand-uniqueness /
    commutative landed, and notes the four aspirational
    FactorizationLevel layers (Associative / ConstantFold /
    Peephole / EGraph) still to implement. Cross-links to
    Rule 21c and DEVELOPMENT_NOTES.

**Why**
Per book doctrine. `non-triviality.md` predated:
- Rule 8 extension (slice `3544a0c`) — N-arity duplicate check.
- Rule 18 α enforcement (`b78550d`) — snapshot/rollback.
- Rule 21 CSE (`f425657`).
- Rule 21b commutative normalization (`c9c2f98`).
- Rule 21c factorization dial (`c9c2f98`).
- Rule 22 mux-arm duplication knob (`d2aefba`).
- `operand_duplication_rate` knob (`5a9b477`).

Most of what the chapter described as "future canonicalizer" has
now landed — the framing was stale.

**Tests**
- No code changed.
- 54 tests pass.
- `mdbook build book` succeeds.

**Impact**
- Readers learning about anti-collapse see the current enforcement
  surface and its knob-driven relaxations, not the
  never-implemented aspirational table.
- The factorization-ladder narrative ties this chapter to
  `structural-rules.md` Rule 21c and the knobs chapter's dial,
  reinforcing the single doctrine across the book.

---

## 2026-04-17-0059 — By-construction chapter: validator tense + Rule 18 exemplar + retry grandfather clause (docs only)

**What changed**
- `book/src/by-construction.md`:
  - "Validator is a safety net" section: tense correction. Was
    "`anvil` will likely include an IR validator" — factually
    stale since `src/ir/validate.rs` has shipped with an inline
    test suite covering every rejection class. Now stated as
    present-tense reality with a note on the CI failure-conversion.
  - New sub-section "Exemplar: Rule 18 (no orphan gates)" — the
    cleanest illustration of by-construction discipline in action.
    Records the α vs β decision (β rejected as generate-then-filter),
    names the mechanism (build_cone snapshot/rollback +
    process_signal_frame existing-operand fallback), and cites the
    current empirical result (0 orphans across 4 strategies × 6
    seeds).
  - New sub-section "Grandfather clause: bounded retry" — makes
    explicit that the *one* retry-and-discard pattern in the
    generator (`build_cone_with_retry` on empty-dep cone roots) is
    bounded, snapshots state between attempts, and differs from
    "generate-then-filter" in that the rejected attempt leaves
    zero trace in the IR. Any other retry-and-filter pattern would
    be a design regression.

**Why**
by-construction.md is a doctrinal chapter (not on the don't-touch
list — that's core-idea / non-goals / why-not-grammar). Current
text predated Rule 18 α enforcement (slice `b78550d`), the CSE
snapshot-table fix (`f425657`), and the validator's actual
shipping; adding the Rule 18 exemplar strengthens the thesis
rather than changing it, and the tense/grandfather-clause edits
are factual corrections.

**Tests**
- No code changed.
- 54 tests pass.
- `mdbook build book` succeeds.

**Impact**
- A reader meeting the by-construction doctrine for the first
  time now sees both the principle and the freshest concrete
  example of the principle being applied.
- The grandfather-clause explicit statement makes the one
  "retry-and-discard" pattern in the codebase (bounded retry
  of empty-dep cone roots) unambiguously doctrine-compliant,
  which forestalls future arguments for relaxing the rule.

---

## 2026-04-17-0058 — Architecture chapter refresh: align with current workspace reality (docs only)

**What changed**
- `book/src/architecture.md`:
  - Crate layout: added `src/metrics.rs` (missing since slice
    `6fb5b9b`). Extended descriptions of `main.rs` (tracing wire-
    up), `lib.rs` (TRACE_DEBUG + trace_verbose! macro), `config.rs`
    (ConstructionStrategy + FactorizationLevel enums), `ir/types.rs`
    (intern_gate / intern_constant API + dedup tables + per-module
    knob mirrors + block-build counters), `gen/cone.rs` (motif
    dispatch, snapshot/rollback, terminal picker variants, dup-cap
    helpers), `gen/module.rs` (orphan audit), `emit/sv.rs`
    (dumb-serialiser doctrine + typed naming).
  - "Key types at a glance" rewritten:
    - `Module` struct showing dedup tables, per-module knob
      mirrors, block-build counters, and the `intern_gate` /
      `intern_constant` method signatures.
    - `GateOp` enum expanded with all variants (was
      `..., Mux, Slice{..}, ...`).
    - Added `ConstructionStrategy` and `FactorizationLevel` enums.
    - Added `metrics::Metrics` + `compute` signatures.
    - Added `lib.rs` trace infrastructure.
  - Testing strategy: per-file unit counts updated (cone.rs 7→13,
    types.rs 0→2, added metrics.rs 3 tests). Integration count
    2→15. Total 23→54.
  - CLI section: old 15-flag listing replaced with a pointer to
    `knobs.md`'s categorised CLI-coverage section and a note that
    `anvil --help` is canonical. Eliminates duplication and
    drift risk.

**Why**
Per book doctrine (up-to-date). `architecture.md` predated 10+
src-touching slices: metrics module, intern_gate API, dedup
tables, FactorizationLevel, ConstructionStrategy enum, trace
infrastructure, most cone.rs helpers, every new test, every new
CLI flag. CODEBASE_ANALYSIS.md was already updated for these
(slice `c0ba963`) but the book's mirror chapter had drifted.

**Tests**
- No code changed.
- 54 tests pass.
- `mdbook build book` succeeds.

**Impact**
- The architecture chapter now accurately describes the Rust
  workspace at HEAD. A cold reader gets the same picture from
  `book/src/architecture.md` as from `CODEBASE_ANALYSIS.md`.
- CLI is canonicalised to one place (`knobs.md`) instead of
  being duplicated in three (README, USER_GUIDE, knobs.md,
  architecture.md). Drift risk reduced.

---

## 2026-04-17-0057 — Algorithm chapter refresh: strategies, Rule 2, Rule 18, CSE, motif dispatch (docs only)

**What changed**
- `book/src/algorithm.md`:
  - Strategy note rewritten. Was: "current `sequential`; three
    *planned* alternatives." Now: all three strategies landed;
    default is `interleaved`; `graph-first` retired as a silent
    alias (pointer to the construction-strategies chapter's
    retirement rationale).
  - `build_cone` pseudocode expanded:
    - New branches for `priority_encoder_prob`, linear-combination
      motif (`coefficient_prob`), const-shift motif
      (`const_shift_amount_prob`), const-comparand motif
      (`const_comparand_prob`). These were all implemented but
      missing from the pseudocode.
    - Added the snapshot/rollback around operand construction
      (Rule 18 α enforcement). Rejected gates restore state so
      operand sub-trees don't orphan.
    - Final node creation goes through `intern_gate` (Rules 21 +
      21b — CSE + commutative normalization). Pool add gated on
      `is_new`.
  - Flop-drain pseudocode: corrected `exclude = Some(q_node)`
    (old "Q-exclusion contract") to `exclude = None` with a
    comment pointing to Rule 2 (Q-feedback freedom). The
    Q-exclusion was relaxed in slice `6cbcbff`.
  - Retry-loop section: now mentions that the snapshot also
    restores `gate_instances` / `const_instances` — the CSE
    dedup tables — and explains the failure mode when they are
    not (pointer to `DEVELOPMENT_NOTES.md`).
  - Anti-collapse section: old 5-line rule list replaced with
    the full current set:
    - Idempotent N-arity (And/Or/Xor) multiset-distinctness.
    - 2-arity algebraic degeneracy (Sub/Eq/Neq).
    - Mux duplicate-arm (gated on `mux_arm_duplication_rate`).
    - Add/Mul duplicate (gated on `operand_duplication_rate`).
    - `factorization_level`-dependent relaxation (cse / none).
    - Note that rejection restores snapshot (Rule 18).
    - Pointer to the factorization ladder for future layers.

**Why**
Audit found the algorithm chapter predated every major
2026-04-15 → 2026-04-17 slice touching `build_cone`:
snapshot/rollback (`b78550d`), CSE via intern_gate (`f425657`),
commutative normalization (`c9c2f98`), motif dispatch (already
landed when the chapter was written but only partially
described), and the Rule 2 Q-feedback relaxation (`6cbcbff`).

**Tests**
- No code changed.
- 54 tests pass.
- `mdbook build book` succeeds.

**Impact**
- The algorithm chapter is now a faithful pseudocode
  transcription of `src/gen/cone.rs` as it stands today. A
  reader can hold them side-by-side without discovering
  discrepancies.
- The anti-collapse section now reflects the
  `factorization_level` dial and the three duplication-rate
  knobs — ties the chapter back into the knob catalog.

---

## 2026-04-17-0056 — Book audit: last `w_N`/`r_N` naming remnants (docs only)

**What changed**
- `book/src/introduction.md`: five-minute pitch replaced. The old
  snippet was hand-written with `w_2 / w_3 / w_4` names that
  never matched current output even under the old opaque scheme.
  New snippet is a real seed-20 output: 23 lines, a single
  `flop_0` hold-register with canonical `always_ff` block,
  showing Rule 12 naming (`flop_<id>`) in action. Added a brief
  paragraph pointing at the `<kind>_<N>` / `flop_<id>`
  convention.
- `book/src/sequential.md`: clock-and-reset SV snippet refreshed
  from `r_0 <= 8'h0` / `r_0 <= w_42` to `flop_0 <= 8'h0` /
  `flop_0 <= add_3`. Added a parenthetical pointing to Rule 12
  for the naming scheme.
- `book/src/synthesizability.md`: canonical flop template's
  `r_0` → `flop_0`. Also corrected an aspirational footnote
  ("or the sync-reset variant, or the no-reset variant, chosen
  per flop at generation time") — this never shipped. Replaced
  with the actual discipline per Rule 5 (single-clock /
  single-reset, async active-low, one `always_ff` block per
  module).

**Why**
Grep of `book/src/` for `\bw_[0-9]+|\br_[0-9]+` (the retired
opaque-naming pattern) found three remaining files. After this
slice, the only remaining match is `w_0 … w_47` in Rule 12's
motivation paragraph where the old naming is deliberately
contrasted with the current scheme — intentional.

**Tests**
- No code changed.
- 54 tests pass.
- `mdbook build book` succeeds.

**Impact**
- Book's flop-related SV excerpts now all match current output.
- The `synthesizability.md` correction removes a misleading
  aspirational claim (sync-reset / no-reset variants) and
  aligns with the actual Rule 5 discipline.

---

## 2026-04-17-0055 — Construction-strategies chapter: graph-first retirement + interleaved-as-default (docs only)

**What changed**
- `book/src/construction-strategies.md`:
  - Lede rewritten: "three strategies" (was four); default is
    `interleaved` (was `graph-first`); graph-first noted as a
    silent alias.
  - `sequential` section: `*(current behavior)*` subtitle
    dropped (no longer true).
  - `interleaved` section: `*(default)*` subtitle added.
  - `graph-first` dedicated section deleted and replaced with a
    "Retired" section that explains *why* (Rule 18 orphan
    violation, pointer to slice `b78550d`), *why not just fix
    graph-first* (the demand-driven version IS interleaved),
    and *what to use instead*.
  - Comparison table: graph-first row removed; `interleaved`
    marked as default.
  - Interaction-with-rules section: updated Rule 9 bullet
    (unified path via `build_cone_with_retry`); Rule 16
    reworded ("strongest in interleaved"); **new Rule 18
    bullet** making the zero-orphan construction contract
    explicit and noting the snapshot/rollback + existing-
    operand-fallback mechanics.
  - Implementation status: graph-first marked retired; silent-
    alias behavior documented; historical-reproducibility note
    pointing to pre-`b78550d` commits.

**Why**
User doctrine: the book must be up-to-date. Audit found
`construction-strategies.md` still described a four-strategy
lineup with graph-first as default and lauded as "the most
realistic shared-DAG output" — but graph-first was retired for
producing 13–27 % orphan gates per module (slice `b78550d`)
and is now a silent alias for interleaved.

**Tests**
- No code changed.
- 54 tests pass.
- `mdbook build book` succeeds.

**Impact**
- Readers coming in cold see the actual strategy surface, the
  retirement rationale (which is also a user-memorable lesson
  about the "by construction" doctrine), and the clear
  migration guidance.
- `--construction-strategy graph-first` still works, so
  existing configs / scripts are unaffected.

---

## 2026-04-17-0054 — Tutorial chapter refresh: naming scheme + re-captured examples (docs only)

**What changed**
- `book/src/tutorial.md`:
  - New "Naming convention" lede paragraph introducing the
    `<kind>_<N>` / `flop_<N>` scheme up front with a pointer to
    Rule 12. New readers now know what `and_5`, `slice_2`,
    `flop_1` mean before encountering them.
  - Example 2's prose updated: `w_N` → `<kind>_N`.
  - Example 4 (direct-D flop): sample SV re-captured from
    current generator output — `r_0` → `flop_0`, `w_3 = a + a`
    → `shl_0 = flop_0 << 1'h0` (the seed-5 run now produces a
    shift, not an add; structural point unchanged). New bullet
    explaining the shift-by-zero is a structural-not-meaningful
    quirk.
  - Example 5 (one-hot D mux): sample lines re-captured with
    current typed naming and the canonical `{W{sel}} & data`
    pattern annotated. Note added that CSE + limited-pool can
    produce richer actual output than the illustrative excerpt.
    Replication-syntax callout (`{8{slice_0}}` vs expanded list)
    added.
  - Example 6 (encoded D mux): re-captured verbatim from seed 11
    output. Shows the full `slice_0` / `eq_0` / `mux_0` /
    `eq_1` / `mux_1` / `always_ff` structure; bottom-up read
    explanation updated.
  - Example 8 (sharing): `w_N` → `<kind>_N` in the prose.
  - Example 9 (comb-mux Encoded): re-captured. Shows the 3-arm
    chained ternary with `slice_0` / `slice_1` as sel / data
    and the `2'h0` fall-through; bottom-up read added.

**Why**
User directive: the book must be up-to-date with actual output.
Audit found every SV excerpt in the Tutorial chapter used the
retired `w_N` / `r_N` naming scheme (superseded by Rule 12
typed-per-kind naming in slice `26f90a3`). Prose in
Examples 2 and 8 also still used `w_N`.

**Tests**
- No code changed.
- 54 tests pass.
- `mdbook build book` succeeds.
- Every sample SV excerpt was re-captured by running the
  example's exact `cargo run` command against HEAD.

**Impact**
- Tutorial now faithful to shipping output. A reader following
  the chapter step-by-step sees the commands and their real
  output, not a historical snapshot.
- Book doctrine (up-to-date / example-heavy / not scary /
  progressive) reinforced.

---

## 2026-04-16-0053 — Knobs chapter alignment with actual config + CLI surface (docs only)

**What changed**
- `book/src/knobs.md`:
  - Quick Reference table: added missing `--operand-duplication-rate`
    row; fixed stale `construction-strategy` default
    (`graph-first` → `interleaved`); fixed `--trace` default
    (`off` → `none`).
  - Knob catalog body: added the missing `operand_duplication_rate`
    entry in the Factorization sub-section (it had landed in slice
    `5a9b477` but was never documented in the book).
  - Defaults block: refreshed to mirror `Config::default()`
    exactly — ~20 knobs were previously missing (`min_gate_arity`,
    `max_gate_arity`, `coefficient_prob`, `min/max_coefficient`,
    `const_shift_amount_prob`, `min/max_shift_amount`,
    `gate_shift_weight`, `const_comparand_prob`, `min/max_comparand`,
    `priority_encoder_prob`, `comb_mux_prob`,
    `comb_mux_encoding_prob`, `construction_strategy`,
    `graph_first_pool_size`, `factorization_level`,
    `max_ast_instances`, `mux_arm_duplication_rate`,
    `operand_duplication_rate`).
  - CLI coverage section: rewritten. Old list covered only
    structural + sequential + share knobs; now 44 flags
    organised by category (Run control / Structure /
    Sequential / Sharing / Operator arity / Coefficient /
    Shift / Comparand / Blocks / Construction strategy /
    Factorization). Explicit list of knobs NOT yet exposed
    via CLI.

**Why**
User directive: *"Make sure these knobs are thoroughly
documented in the book too."* Audit showed the
`operand_duplication_rate` knob (landed in `5a9b477`) was not
in the book catalog; the defaults block had ~20 missing
entries; the CLI coverage block listed only the original Phase
1 subset.

**Validation**
A script that grep-extracts every `--flag` mention from the
book's CLI-coverage section and compares against `anvil --help`
reports:
```
flags named in book:  44
flags in --help:      45
book-only (broken):   []
help-only (undoc'd):  ['--version']   ← clap boilerplate, expected
```

**Tests**
- No code changed.
- 54 tests pass.
- `mdbook build book` succeeds.

**Impact**
- Book's knob catalog now matches the shipping CLI 1:1.
- A reader opening `book/src/knobs.md` sees every flag they
  can pass on the command line, with defaults and intended
  effect. Session-recovery resilience restored for the knobs
  chapter.

---

## 2026-04-16-0052 — Block counters: priority_encoder + comb_mux_encoding (closes the last pending effectiveness-map entries)

**What changed**
- `src/ir/types.rs`: three new live-counter fields on `Module`:
  - `priority_encoder_built: u32`
  - `comb_mux_one_hot_built: u32`
  - `comb_mux_encoded_built: u32`
  Each is a per-module tally incremented at the block-build site.
- `src/gen/cone.rs`: increments at four sites:
  - `build_priority_encoder_recursive` — increments once the
    assemble succeeds.
  - `build_priority_encoder_pool` — same.
  - `build_comb_mux` — increments either the `one_hot` or
    `encoded` counter based on the `comb_mux_encoding_prob` coin
    before dispatching to the assembly helper.
  - `build_comb_mux_pool_only` — same counter pair inside its own
    encoded/one-hot branches.
- `src/metrics.rs`: three new `Metrics` fields
  (`num_priority_encoder_blocks`, `num_comb_muxes_one_hot`,
  `num_comb_muxes_encoded`) populated from the Module counters.
- `book/src/knobs.md` effectiveness map:
  - `priority_encoder_prob` → concrete metric.
  - `comb_mux_prob` → sum-of-two-counters (plus
    `num_muxes_2to1` still applicable).
  - `comb_mux_encoding_prob` → ratio of encoded / total comb muxes.
  - **No *pending* entries remain.** Closing paragraph rewritten.
- `USER_GUIDE.md`: knob-effects bullet list extended with the two
  new rate knobs' expected shifts.
- `CODEBASE_ANALYSIS.md`: `metrics.rs` one-liner extended.

**Why**
Two entries still marked *pending* on the effectiveness map:
`priority_encoder_prob` and `comb_mux_encoding_prob`. Both
required detecting block shape after the block had lowered to
a chain of gates, which is brittle post-hoc. The better fix is a
live counter at the block-build site — cheap, accurate, and
matches the "construction-time measurement" doctrine.

**Tests**
- All four cargo gates green.
- 54 tests pass.
- Demo sweeps at seed 42:

  ```
  priority-encoder-prob=0.0:  num_priority_encoder_blocks=0
  priority-encoder-prob=0.05: num_priority_encoder_blocks=49
  priority-encoder-prob=0.2:  num_priority_encoder_blocks=221
  priority-encoder-prob=0.5:  num_priority_encoder_blocks=454

  comb-mux-encoding-prob=0.0 (--comb-mux-prob 0.4): one_hot=2229, encoded=0
  comb-mux-encoding-prob=0.5 (--comb-mux-prob 0.4): one_hot=887,  encoded=859
  comb-mux-encoding-prob=1.0 (--comb-mux-prob 0.4): one_hot=0,    encoded=959
  ```

  Clean monotone for `priority_encoder_prob`; near-even split at
  `comb_mux_encoding_prob=0.5` (887/859 ≈ 50.8%/49.2%).

**Impact**
- Every knob in the catalog now has a concrete, measurable
  effect. The "every knob must be measurable" doctrine (Rule:
  knob measurement) is empirically satisfied across the full
  knob set.
- No behaviour change.

---

## 2026-04-16-0051 — Combinational-depth metrics (closes another pending effectiveness-map entry)

**What changed**
- `src/metrics.rs`: two new `Metrics` fields
  - `max_gate_depth: usize` — longest combinational fan-in path
    through any gate. Primary inputs, constants, and flop Q act
    as depth-0 leaves (clock edge breaks Q→D loop temporally).
  - `gate_depth_histogram: BTreeMap<usize, usize>` — count of
    gates at each depth.
  - `compute()` adds a single forward walk over `m.nodes` (which
    is always in topological order) that assigns depth =
    `max(operand depth) + 1`.
- `book/src/knobs.md` effectiveness map: `max_depth` moves from
  `pending` to `max_gate_depth` / `gate_depth_histogram`.
- `USER_GUIDE.md`: metrics section and knob-effects bullet list
  extended with the new fields.
- `CODEBASE_ANALYSIS.md`: metrics.rs one-liner updated.

**Why**
Two entries still listed as *pending* in the knob-effectiveness
map: `max_depth` and `priority_encoder_prob` / `comb_mux_encoding_prob`.
`max_depth` is the easier one — a single topological walk gives
both `max_gate_depth` and a full histogram. Closes one of the
three remaining pending entries.

**Relationship between `max_depth` knob and `max_gate_depth`
metric:** the knob bounds `build_cone` recursion depth; the
metric measures IR gate-chain depth. They are NOT 1:1 because
block-assembly helpers (chained-ternary muxes, OR-of-masked-arms
muxes, linear-combination trees) expand each recursion level
into many gate layers. The metric is typically 10–100× the knob
value but monotone in it — useful for verifying knob effect.

**Tests**
- All four cargo gates green.
- 54 tests pass (no new tests; single-walk metric is obvious
  from the existing `compute` structure).
- Demo sweep at seed 42:
  ```
  max-depth=2:  max_gate_depth=54
  max-depth=4:  max_gate_depth=115
  max-depth=6:  max_gate_depth=154
  max-depth=8:  max_gate_depth=206
  max-depth=10: max_gate_depth=354
  ```
  Clean monotone.

**Impact**
- Effectiveness-map *pending* list shrinks from 3 → 2
  (`priority_encoder_prob`, `comb_mux_encoding_prob` still
  pending).
- No behaviour change.

---

## 2026-04-16-0050 — Live-doc catch-up: CODEBASE_ANALYSIS, USER_GUIDE, DEVELOPMENT_NOTES, ROADMAP (docs only)

**What changed**
- `CODEBASE_ANALYSIS.md`:
  - Module map refreshed: `lib.rs` trace infrastructure
    (TRACE_DEBUG, set_trace_debug, trace_verbose! macro);
    `metrics.rs` new operand-arity fields; `config.rs`
    FactorizationLevel + the three duplication-rate knobs;
    `ir/types.rs` intern_gate / intern_constant API + dedup
    tables + per-module knob mirrors; `gen/module.rs` orphan
    audit; `gen/cone.rs` GraphFirst retirement + build_cone
    snapshot/rollback + process_signal_frame existing-operand
    fallback + pick_terminal_dep_bearing +
    pick_datas_with_dup_cap + pick_signals_with_dup_rate.
  - Phase coverage map: Phase 1 promoted in-progress → mostly
    done (22 structural rules enforced, 0 orphans, 0 default
    duplicate operands; blocked only on external Verilator/Yosys
    smoke). Phase 2 entry notes CSE + operand-uniqueness +
    commutative. Phase 3 entry notes priority-encoder landed.
  - Invariants list: intern_gate CSE + commutative contract;
    build_cone snapshot/rollback; process_signal_frame
    anti-collapse fallback; generate_leaf_module orphan audit.
  - Test count: 35 + 15 → 39 + 15 = **54**.
- `USER_GUIDE.md`:
  - Metrics section lists the operand-arity fields explicitly.
  - Knob-effects bullet list extended with
    `operand_duplication_rate`, `max_gate_arity`, and
    `factorization_level` entries.
- `DEVELOPMENT_NOTES.md` (first update since `e6850fc` —
  15 src-touching commits of drift):
  - "Construction-time CSE via Module::intern_gate" — method
    contract, dedup-table rationale, snapshot contract with
    build_cone_with_retry.
  - "Rule 18 α construction-time" — α vs β decision record +
    GraphFirst retirement rationale.
  - "Full factorization doctrine" — NodeId = expression identity,
    the 7-layer implementation ladder, FactorizationLevel::
    effective() clamping.
  - "Emitter is a dumb serialiser" — doctrinal anchor.
  - "Rejected: without-replacement operand picking as default"
    — why the anti-collapse + rollback path was chosen.
- `ROADMAP.md`:
  - Phase 1 label: in progress → mostly done; exit-criteria note
    that external smoke tests are blocked locally; internal
    validation (54 tests, 0 orphans, 0 default duplicates) is
    complete.
  - Phase 3: in progress; per-item status (priority encoder
    landed, constant-shift landed, linear-combination landed;
    case/casez / reductions / variable-shift / for-loop not
    started).

**Why**
Per user directive: *"Please strictly follow the commit workflow
w.r.t. which live docs shall be updated. It is not cosmetic, it
is of utmost importance to ensure the continuity of the project
following session loss or crash."*

Audit showed DEVELOPMENT_NOTES.md had not been touched in 15
src-changing commits (`92d43f8` through `64850da`). Several of
those slices embedded design decisions (Rule 18 α, intern_gate
contract, FactorizationLevel) that deserved permanence beyond
commit messages. CODEBASE_ANALYSIS.md was stale on module
ownership. USER_GUIDE.md missed the new knob surface. ROADMAP.md
had Phase 1 still at "in progress" despite Rule-18 enforcement
and the full factorization work.

**Tests**
- No code changed.
- All four cargo gates green.
- 54 tests unchanged.
- `mdbook build book` succeeds.

**Impact**
- A session that recovers cold from `git clone` → `SESSION_BOOTSTRAP.md`
  now sees the actual workspace reality in `CODEBASE_ANALYSIS.md`
  and the design rationale in `DEVELOPMENT_NOTES.md`.
- No behavioural change.

---

## 2026-04-16-0049 — Operand-arity metrics (closes a pending effectiveness-map entry)

**What changed**
- `src/metrics.rs`:
  - New `Metrics::gate_operand_count_histogram: BTreeMap<usize,
    usize>` — count of gates per operand-count value.
  - New `Metrics::max_gate_operand_count: usize` — the largest
    operand list observed on any single gate.
  - New `Metrics::max_operand_count_by_kind:
    BTreeMap<String, usize>` — per-`GateOp`-kind ceiling,
    distinguishing e.g. `add`'s arity (bounded by
    `max_gate_arity`) from `concat`'s arity (driven by
    replicate-to-width).
  - `compute()` populates all three during the existing single
    walk over `m.nodes`; no new passes.
- `book/src/knobs.md`: effectiveness-map entry for
  `min_gate_arity` / `max_gate_arity` moves from *pending* to
  concrete metric names.

**Why**
The knob-effectiveness map had `min_gate_arity` / `max_gate_arity`
marked *pending*. Per the measurement doctrine ("no knob is
privileged"), every knob needs a metric. This slice provides one.

**Tests**
- All four cargo gates green.
- 54 tests pass.
- Demo sweep at seed 42:
  ```
  max-gate-arity=2: max_gate_operand_count=8  (add max=2, mul max=3)
  max-gate-arity=4: max_gate_operand_count=27 (add max=4, mul max=5)
  max-gate-arity=6: max_gate_operand_count=11 (add max=6, mul max=7)
  max-gate-arity=8: max_gate_operand_count=27 (add max=8, mul max=9)
  ```
  - `add` max tracks the knob exactly.
  - `mul` max is `knob + 1` because the Mul linear-combination
    motif prepends a coefficient operand.
  - `max_gate_operand_count` top end is driven by `concat`
    (replicate-to-width can produce 27-operand concats
    regardless of the gate-arity knob) — exactly the reason a
    per-kind breakdown is useful.

**Impact**
- No behavioral change. Observability gain only.
- The effectiveness map moves one more knob off the *pending*
  list; remaining gaps: `max_depth`, `priority_encoder_prob`,
  `comb_mux_encoding_prob`.

---

## 2026-04-16-0048 — Close residual Add/Mul/And duplicate operands at default knobs

**What changed**
- `src/gen/cone.rs`:
  - `assemble_mul_linear_combination`: when
    `operand_duplication_rate < 1.0`, dedupes the `signals` list
    before building the N-ary `Mul`. `c * x * x * y` becomes
    `c * x * y` (loses the x² factor — that's the user's
    explicit no-duplicates contract). Preserves the `coef == 1 ⇒
    n >= 2` invariant via a degenerate passthrough when
    dedup produces a single signal.
  - `assemble_add_linear_combination`: same post-Mul
    dedup on the outer `Add`'s `terms` list. Two terms can
    coincide when signal + coefficient both match (CSE-collapse);
    we drop the duplicate.
  - `make_and`: idempotent short-circuit `x & x = x` when
    `factorization_level.effective() >= OperandUnique`. Closes
    the last escape path — the one-hot-mux mask assembly can
    produce `And(sel, data)` where `sel == data` via CSE at
    width=1, which bypassed anti-collapse because `make_and`
    calls `intern_gate` directly.
- `src/config.rs`: `FactorizationLevel` default now written
  via `#[default]` derive instead of hand-rolled `impl Default`
  (clippy hint). Doc comments for the enum variants reworded to
  avoid the `+ X` leading character that clippy parsed as a
  bullet list.

**Why**
Previous audit showed 0.09% duplicate-operand Add/Mul/And gates
at default `operand_duplication_rate = 0.0`. Per the user's full-
factorization doctrine these should be exactly zero.

**Tests**
- All four cargo gates green.
- 54 tests pass.
- Orphan audit: 0 across 4 strategies × 6 seeds (Rule 18 holds).
- Duplicate-operand audit:
  - `rate=0.0` (default): 4633 gates, **0 duplicates (0.000%)**
    — exactly zero, down from 0.09%.
  - `rate=1.0`: 5336 gates, 57 duplicates (1.07%) — knob
    still active.

**Impact**
- Syntactic factorization layer (CSE + operand-unique +
  commutative) is now *complete* at default knobs — no operand
  duplication anywhere across the tested seed range.
- Next layer (associative flattening) can now be designed
  without the noise of residual duplicates.

---

## 2026-04-16-0047 — Recipe: "I want to see how the factorization dial affects output" (docs only)

**What changed**
- `book/src/recipes.md`: new recipe walking a user through the
  `--factorization-level` dial with a real shell sweep, captured
  output at seed 42, and a layer-by-layer explanation of the
  deltas.

**Why**
Per user book doctrine — "littered with examples." The
factorization dial landed in the previous slice with catalog docs
in `knobs.md` and the rule text in `structural-rules.md`, but
there was no paste-and-run recipe for a user who wants to *see*
the knob. This slice provides one.

**Tests**
- No code changed.
- `mdbook build book` succeeds.
- 54 tests unchanged.

---

## 2026-04-16-0046 — Commutative normalization + factorization-level dial

**What changed**
- `src/ir/types.rs` `Module::intern_gate`:
  - Sorts operands of commutative ops (`And`/`Or`/`Xor`/`Add`/`Mul`)
    before building the dedup key, so `a + b` and `b + a` share
    a single NodeId. Gated on `factorization_level >=
    Commutative`.
  - New `None` fast path: every intern_gate / intern_constant
    call bypasses the dedup table and creates a fresh node.
    Useful for stress-testing downstream CSE.
  - Two unit tests covering commutative-vs-non-commutative
    interning.
- `src/config.rs`:
  - New enum `FactorizationLevel` with 8 levels:
    `None → Cse → OperandUnique → Commutative → Associative →
    ConstantFold → Peephole → EGraph` (default `EGraph`).
  - `Config::factorization_level` field, threaded through
    `Overrides` and `apply_cli_overrides`.
  - `FactorizationLevel::highest_implemented()` returns
    `Commutative` today; `effective()` clamps user requests
    down to that ceiling so aspirational levels don't error.
- `src/main.rs`: `--factorization-level <LEVEL>` CLI flag.
- `src/ir/types.rs` `Module.factorization_level` mirror field.
- `src/gen/module.rs`: wire from config.
- `src/gen/cone.rs` `violates_anti_collapse`: operand-uniqueness
  checks now gated on `factorization_level.effective() >=
  OperandUnique`.
- `book/src/structural-rules.md`:
  - New Rule 21b "Commutative normalization".
  - New Rule 21c "Factorization level" with the level table,
    doctrinal anchor ("NodeId = expression identity"), and the
    aspirational-anchor mechanism.
- `book/src/knobs.md`: new catalog entry + quick-reference row.

**Why**
User coined "full factorization" as the doctrine: NodeId is the
identity of an expression; no expression / sub-expression ever
duplicated. User directive: "we need a knob to control where on
the chain we want to be, default e-graph."

**Tests**
- All four cargo gates green.
- 39 unit + 15 integration = **54 tests, all passing**.
- Empirical dial at seed 42:

```
none             gates=1961     (no dedup — fresh node per call)
cse              gates=1776     (syntactic CSE only)
operand-unique   gates=2368     (+ Rule 8 operand uniqueness)
commutative      gates=2368     (+ commutative sort)
associative      gates=2368     (aspirational → commutative today)
constant-fold    gates=2368     (aspirational → commutative today)
peephole         gates=2368     (aspirational → commutative today)
e-graph          gates=2368     (aspirational → commutative today; DEFAULT)
```

**Impact**
- Default behaviour unchanged vs previous default (both land at
  `commutative`).
- Users can now dial *down* (for stress-testing) via
  `--factorization-level none` or `cse`.
- Aspirational levels (`associative`, `constant-fold`,
  `peephole`, `e-graph`) compile without behavioural surprise
  — when future slices implement them, users at those levels
  automatically gain the tighter factorization. No config
  migration needed.

---

## 2026-04-16-0045 — Operand-uniqueness knob (`--operand-duplication-rate`)

**What changed**
- `src/config.rs`: new knob `operand_duplication_rate: f64` in
  `[0.0, 1.0]`, default `0.0`. Threaded through `Overrides` +
  `apply_cli_overrides`. Applies to Add/Mul operand lists only —
  And/Or/Xor are always strict (algebraic collapses). Comparisons,
  Sub, Mux retain their 2-operand degenerate-shape rejection
  governed by Rule 8 / Rule 22.
- `src/main.rs`: CLI flag `--operand-duplication-rate <F>`.
- `src/ir/types.rs`: `Module.operand_duplication_rate` field.
- `src/gen/module.rs`: `generate_leaf_module` clamps + forwards
  the config value to the module.
- `src/gen/cone.rs`:
  - `violates_anti_collapse` signature change: `_m` → `m`
    (uses the module's knob fields).
  - Add/Mul operand-list duplicates are now flagged when
    `m.operand_duplication_rate < 1.0`.
  - Mux degenerate data-arm shape uses `m.mux_arm_duplication_rate
    < 1.0` (cleaner, same semantics).
  - New helper `pick_signals_with_dup_rate` mirrors
    `pick_datas_with_dup_cap` — used in
    `build_linear_combination_pool` so pool-mode Add/Sub/Mul
    signals are distinct.
  - Existing anti-collapse test updated to assert Add/Mul
    duplicates ARE flagged at default rate (was the inverse).
- USER_GUIDE / Knobs references updated via follow-up book slice.

**Why**
User directive — "we need to opt in to allow duplicates; by
default they are not allowed." Previously `Add` and `Mul` were
Rule-8 exempt (duplicates algebraically meaningful), so you could
get `mul = c * s * s * s * s` or `add = s + s + s` at default
knobs. Now default is strict uniqueness; the user explicitly
passes `--operand-duplication-rate 1.0` to exercise those shapes.

**Tests**
- All four cargo gates green.
- 49 tests pass.
- Empirical verification across 5 seeds:
  - `rate=0.0` (default): 4374 gates, 4 with duplicate operands
    (0.09%). Residuals come from the recursive linear-combination
    path where `build_cone` recursion + CSE can collapse two
    distinct sub-cones to the same NodeId; rewriting that path to
    enforce uniqueness without introducing orphans is a deferred
    follow-up.
  - `rate=1.0`: 5184 gates, 56 duplicates (1.08%). Knob clearly
    active.
- No orphans introduced (Rule 18 still holds).

**Impact**
- Default output has `x + y + z` instead of `x + x + y`.
- `x + x = 2x` / `x * x = x²` shapes reachable on demand.

---

## 2026-04-16-0044 — `--trace debug` is now strictly more verbose than `high`; `off` → `none`

**What changed**
- `src/lib.rs`:
  - New static `TRACE_DEBUG: AtomicBool` + public
    `set_trace_debug(bool)` / `trace_debug_enabled()` helpers.
  - New `trace_verbose!` macro: fires `tracing::trace!` only when
    `TRACE_DEBUG` is true. Used for super-verbose events that
    would flood `--trace high`.
- `src/main.rs`:
  - `TraceLevel` renamed `Off` → `None` with `#[value(alias = "off")]`
    so `--trace off` still works. Default remains silent.
  - `TraceLevel::debug_verbose()` returns true only for `Debug`.
  - `init_tracing` calls `anvil::set_trace_debug(cli.trace.debug_verbose())`.
- `src/ir/types.rs`: `intern_gate` and `intern_constant` emit
  `trace_verbose!` events on both creation (`🔗 new`) and
  reuse-on-cap-hit (`♻️ reuse`). Every node that enters the IR
  is now traceable, with span context showing which call path
  created it.
- `src/gen/cone.rs`: `pick_gate` return logged with
  `trace_verbose!(?op, depth, width, "🎲 pick_gate")` in both
  `build_cone` (recursive path) and `process_signal_frame`
  (interleaved path). Linear-combination motif dispatch also
  gets a `trace_verbose!` marker.
- `USER_GUIDE.md`: tracing level table updated with accurate
  descriptions and the `none` / `off` alias note.

**Why**
User directly tested `--trace debug` vs `--trace high` and found
them identical (both mapped to `LevelFilter::TRACE`). They also
expected `--trace none` to work, but the CLI only accepted `off`.
Both were real defects — the CLI advertised a level that did
nothing, and the naming didn't match user expectations. User
directive: "we should be able to see everything, start/end of
every function, every branch. Without this it is painful to
debug efficiently."

**Tests**
- All four cargo gates green.
- 49 tests pass.
- Empirical verification at seed 42:
  - `--trace none` → 0 lines
  - `--trace low` → 5 lines
  - `--trace medium` → 141 lines
  - `--trace high` → 3779 lines
  - `--trace debug` → 8241 lines (+4462 strict super-set)
  - `--trace off` still accepted as silent-alias.
- Sample `debug`-only events (not visible at `high`):
  - `🎲 interleaved pick_gate op=Mux depth=0 width=21`
  - `🔗 intern_gate new node=5 op=Not width=11 n_operands=1`
  - `🔗 intern_constant new node=6 width=1 value=0`
  - `♻️ intern_gate reuse (AST cap hit) node=N op=X width=W`

**Impact**
- `--trace debug` is now the tool for answering "who created this
  node?" — every `intern_*` call surfaces with its span context.
- Zero performance impact at `--trace none` (default) or at
  `high` (the `trace_verbose!` guard is an atomic load + `false`
  short-circuit).
- Trace output remains deterministic across runs with the same
  `(seed, knobs)`.

**Known residual trace gaps** (future slices):
- `pick_terminal` doesn't emit a `trace_verbose!` on every call
  (only tier-pick events at `high`). Already covered sufficiently
  for most debugging.
- Block-assembly helpers (`assemble_flop_d_encoded`,
  `build_comb_mux_encoded`, priority encoder) don't emit a
  `trace_verbose!` event for each assembly step. Adding them is
  a straightforward follow-up if block-debug is needed.

---

## 2026-04-16-0043 — Zero orphans: Rule 18 enforced (construction-time)

**What changed**
- `src/gen/cone.rs`:
  - `build_cone` (recursive path) snapshots `m.nodes.len()`,
    `m.flops.len()`, pool, worklist, `gate_instances`, and
    `const_instances` before building operand sub-trees. On
    anti-collapse rejection the snapshot is restored and
    `pick_terminal` provides a safe fallback. Sub-tree nodes
    that were built for the rejected gate are truncated — no
    orphan leaks.
  - `process_signal_frame` (interleaved path): the frame machine
    cannot snapshot per-gate because sibling sub-frames have
    already committed. On anti-collapse rejection it delivers
    one of the existing operand `NodeId`s as the fallback instead
    of creating a new `pick_terminal` node. For idempotent /
    self-inverse / comparison collapses the operands share a
    NodeId, so the fallback is semantically correct. For
    `mux(s, a, a)` it uses `operands[1]` (= operands[2]).
- `src/config.rs`:
  - Default `construction_strategy` switches from `GraphFirst` to
    `Interleaved`. GraphFirst was the only strategy that
    speculatively created pool units before knowing who would
    consume them (13–27% orphan rate).
  - `GraphFirst` enum variant retained as a silent alias for
    `Interleaved` so existing CLI invocations / config files
    continue to work; the speculative pool-growth code path is
    unreachable.
- `src/gen/module.rs`:
  - Match on `construction_strategy` routes both `Interleaved`
    and `GraphFirst` to `cone::build_outputs_interleaved`.
  - New safety-net audit `count_orphan_gates(m)` called after
    flop drain; warns via `tracing::warn!` if any Gate has no
    consumer.
- `src/emit/sv.rs`: emitter goes back to a dumb serialiser.
  Per user doctrine — "all thinking, checks, rules' enforcement
  ought to be done solely at the IR level; by the time you reach
  emission, it is too late to roll back." The brief live-set
  filter added in a previous iteration is removed; `to_sv`
  iterates `m.nodes` and dumps.
- Emitter test updated: `slice_and_concat_rendered` now chains
  the slice + concat through the drive-root so both are live.

**Why**
User directive: "A gate/module/block shall come into existence
solely when needed, not speculatively created beforehand in the
hope they will be picked and connected." — "Not acceptable!"

**Tests**
- All four cargo gates green.
- 34 unit (+0 net) + 15 integration = **49 tests, all passing**.
- Orphan audit across 4 strategies × 6 seeds (1, 42, 100, 777,
  9999, 12345): **0 orphans in every run.**
- Reproducibility holds: graph-first and interleaved now produce
  byte-identical output for the same seed (graph-first is an
  alias).
- No undeclared references in any emitted SV (verified 4 × 4
  strategy×seed matrix).

**Impact**
- Default output is smaller and cleaner. No declared wire goes
  unused; no referenced signal is undeclared.
- For users who explicitly selected `--construction-strategy
  graph-first`: behavior is now identical to `interleaved`. No
  CLI break.
- Generator's "by construction" contract is now honoured for
  Rule 18 too — no post-emission filter exists.

**Known trace-coverage gap (deferred)**
User flagged that the trace doesn't clearly show "which node
requested this new gate." `build_cone` and `process_signal_frame`
don't emit an op-pick trace event, so `--trace high` can't be
used to answer "who created `not_0`?" Follow-up slice will add
explicit trace events at every intern_gate call site with
requester context.

---

## 2026-04-16-0042 — IR chapter refresh + future-extensions roadmap (docs only)

**What changed**
- `book/src/ir.md`:
  - `Module` struct snippet refreshed: now shows `gate_instances`,
    `const_instances`, `max_ast_instances`,
    `mux_arm_duplication_rate` fields.
  - New section "Node construction: `intern_gate` /
    `intern_constant`" documenting the method signatures, cap
    semantics, why the dedup tables live on `Module`, and the
    snapshot/rollback contract with `build_cone_with_retry`.
  - Emitter naming section updated for Rule 12: no more `w_N`
    or `r_N`; current naming is `<gate_kind>_<N>` per-kind + `flop_<id>`.
  - New "Future extensions" section capturing the parameters /
    synthesizable-aggregates / first-class-blocks roadmap analysis
    in durable form. Parameters (Phase 5, hard-requires Phase 4).
    Aggregates split into four sub-paths with explicit
    cost/payoff per path (packed = cheap emitter-only; unpacked
    arrays = memories, already Phase 6; unpacked datapath
    aggregates + enums = deprioritised). Blocks as first-class
    IR cross-references the session memory on
    hierarchical-vs-flatten-with-mangling.
- `ROADMAP.md`:
  - Phase 5 entry adds cross-reference to IR chapter and names
    Phase 4 as hard prerequisite.
  - New Phase 5b entry for aggregates (scheduled alongside
    Phase 5, order not fixed), pointing to IR chapter for the
    four-sub-path breakdown.

**Why**
User direction: the book must thoroughly document the IR as it
evolves, and the parameters/aggregates discussion from the
preceding exchange must land in durable docs, not just commit
messages.

**Tests**
- No code changed.
- `mdbook build book` succeeds.
- 50 tests unchanged.

**Impact**
- Next session (or a cold reader) can open `book/src/ir.md` and
  see the full current IR plus the design record for the two
  roadmap axes, without losing context to session compaction.

---

## 2026-04-16-0041 — Friendly docs: quick ref, naming refresh, recipe examples (docs only)

**What changed**
- `book/src/getting-started.md`:
  - Refresh the sample module output to match current typed-per-
    kind naming (`slice_0`, `add_0`, `mul_0` — was `w_2`, `w_3`,
    `w_4`). Added `--construction-strategy sequential` to the
    starter command so the output stays small.
  - New paragraph explaining the naming scheme briefly with a
    pointer to Rule 12.
- `book/src/knobs.md`:
  - New reassuring opening: "you don't need to read this
    top-to-bottom". Points new readers at the Recipes chapter
    first.
  - New "Quick reference" table covering the ~13 knobs most users
    actually touch, with defaults and one-line descriptions.
- `book/src/recipes.md`: six new recipes covering the knobs and
  workflows that landed in this session:
  - "I want to see fewer redundant expressions" (strict CSE —
    the default).
  - "I want duplicated expressions anyway" (bounded duplication
    via `--max-ast-instances`).
  - "I want pathological mux shapes" (arm duplication via
    `--mux-arm-duplication-rate`).
  - "I want to verify a knob is doing something" (the metric-
    grep workflow).
  - "I want to sweep a knob and compare" (shell loop + jq,
    with a real `--flop-prob` sweep as the example).
  - "I want to trace what the generator is doing" (--trace
    levels with sample output).

**Why**
Per user direction: the knobs+metrics+tracing information that
landed in the last several slices needs to be user-facing and
*not scary*. The getting-started sample was out of date (old
`w_N` naming); the knobs reference didn't tell newcomers it's
a catalog they can skim, not a syllabus; the recipes chapter
didn't cover any of the new knobs.

**Tests**
- No code changed; no test impact.
- `mdbook build book` succeeds.
- 50 tests unchanged.
- The `--flop-prob` sweep values in the recipe were verified
  against real CLI output at seed 42.

**Impact**
- New reader's path: README → SESSION_BOOTSTRAP → book
  Getting Started → Tutorial → Recipes. All four now show
  current naming, current knobs, current workflows.
- Every landed knob now has a recipe or quick-reference entry
  somewhere in the book — no knob is orphaned in code only.

---

## 2026-04-16-0040 — Knob measurement doctrine + effectiveness map (docs only)

**What changed**
- `book/src/knobs.md`:
  - New opening section "Measurement doctrine": every knob is
    subject to the same rule — its effect must be empirically
    measurable via `Metrics` and/or `--trace`. No knob is
    privileged. Three landing requirements: (1) a metric captures
    the knob's effect; (2) the knob's section names the metric;
    (3) a CLI spot-check at boundary values shows the metric
    shifting.
  - New sub-section "AST uniqueness / duplication" covering the
    two recent knobs (`max_ast_instances`, `mux_arm_duplication_rate`)
    with cross-references to Rules 21 and 22 in the structural-
    rules catalog.
  - New table at the bottom, "Knob effectiveness map" — one row
    per knob listing the metric(s) that measure its effect.
    Entries marked *pending* flag knobs whose effect the current
    metric set does not yet capture (candidates for a follow-up
    slice).
- No code changed.

**Why**
Per user direction: the knobs + metrics design discussion from
this session must land in durable docs, not just commit
messages. The knobs chapter was already the canonical knob
reference but lacked (a) the doctrinal line that no knob is
privileged, (b) the two new knobs, (c) the explicit knob → metric
mapping.

**Tests**
- No code changed; no test impact.
- `mdbook build book` succeeds (HTML written to `book/book-out`).
- 50 tests unchanged.

**Impact**
- Durable design record for the next session.
- Explicit catalog of gaps (pending metrics) to address in
  follow-up slices.

---

## 2026-04-16-0039 — Structural metrics (per-module observability)

**What changed**
- New module `src/metrics.rs` with `Metrics` struct and
  `metrics::compute(m: &Module) -> Metrics`. Post-hoc walk over a
  generated module — no generator instrumentation required. Fields:
  - Size: `num_inputs`, `num_outputs`, `num_nodes`, `num_gates`,
    `num_constants`, `num_primary_inputs`, `num_flop_q_refs`,
    `num_flops`.
  - Per-kind distribution: `gates_by_kind` (BTreeMap<kind, count>),
    `constants_by_width`.
  - Constants: `constants_zero`, `constants_all_ones`,
    `constants_other`.
  - Mux shape: `num_muxes_2to1`, `num_muxes_degenerate`.
  - Concat shape: `num_concats_replication` (all operands
    identical → `{N{expr}}`) vs `num_concats_heterogeneous`.
  - Sharing / fanout: `num_shared_nodes` (fanout ≥ 2),
    `max_fanout`, `avg_fanout`.
  - Flops: `flops_zero_default`, `flops_qfeedback`,
    `flops_mux_none`, `flops_mux_one_hot`, `flops_mux_encoded`.
  - AST-instance saturation: `max_gate_ast_multiplicity`,
    `max_constant_ast_multiplicity`.
- `src/main.rs`: new CLI flag `--metrics`. For the single-module
  path it prints metrics JSON to stderr. For multi-module runs
  the metrics block is always embedded in `manifest.json` per
  entry (replacing the tiny `{file, name, inputs, outputs, nodes}`
  summary).
- 3 new unit tests in `metrics` module (empty, per-kind, flop
  shape).
- `USER_GUIDE.md`: new "Metrics" section with CLI examples and a
  list of typical sweep-verify workflows.

**Why**
User directive: "every aspect of what is generated, every knob
related generated shall be able to measure the effectiveness of
the knobs." Metrics give us empirical grounding — without them
we can't tell whether `mux_arm_duplication_rate = 0.0` actually
produces 0 degenerate muxes, or whether `max_ast_instances = 5`
lets expressions reach the cap, or whether `flop_prob = 0.15`
produces the expected flop-density. Now each is a grep away.

**Scope chosen (post-hoc, structural only)**
Live counters (probability rolls fired/missed, anti-collapse
retries, terminal-tier picks) are deliberately deferred — they
need instrumentation at every decision site, ~40 edit points.
Most are already surfaced as `--trace high` events; aggregating
them into counters is a future extension if the post-hoc
structural metrics aren't sufficient.

**Tests**
- All four cargo gates green.
- 35 unit (+3 new) + 15 integration = **50 tests, all passing**.
- Demonstrated observability: at seed 42 default,
  `num_muxes_degenerate = 0` (matches Rule 22 at rate 0.0);
  at `--mux-arm-duplication-rate 1.0`, it jumps to 1.
  `max_gate_ast_multiplicity = 1` at default; at
  `--max-ast-instances 5`, rises to 3 with 29 more nodes in
  the module.

**Impact**
- New public API: `anvil::metrics::{Metrics, compute}`.
- `manifest.json` shape changed: `inputs`/`outputs`/`nodes`
  summary replaced with a full `metrics` field. Consumers of the
  old shape need to update.

---

## 2026-04-16-0038 — Mux arm-duplication rate (Rule 22)

**What changed**
- `src/config.rs`: new knob `mux_arm_duplication_rate: f64` with
  range `[0.0, 1.0]`; default `0.0` = all arms of any N-to-1 mux
  must be distinct signals. Threaded through `Overrides` and
  `apply_cli_overrides`.
- `src/main.rs`: new CLI flag `--mux-arm-duplication-rate <F>`.
- `src/ir/types.rs`: `Module.mux_arm_duplication_rate` field.
  `generate_leaf_module` initialises from config (clamped to
  `[0.0, 1.0]`).
- `src/gen/cone.rs`:
  - New helper `pick_datas_with_dup_cap(g, m, pool, width, count,
    exclude)`: picks `count` arm signals; on a duplicate candidate,
    keeps with probability `mux_arm_duplication_rate`, otherwise
    re-picks (bounded 8-try budget). Used at all pool-mode mux
    assembly sites: encoded/one-hot comb-mux, encoded/one-hot
    flop-D drain.
  - `make_mux`: at rate `0.0`, `a == b` collapses the layer to
    return `a` directly (the 2-to-1 case). At any rate `> 0.0`,
    the mux is emitted as-is — the upstream caller has already
    decided whether duplication is permitted for this arm.
- `book/src/structural-rules.md`: new Rule 22 "Mux arm-duplication
  rate" with motivation, construction-time enforcement, and knob
  semantics.

**Why**
User flagged `mux_9 = (eq_0) ? (flop_0) : (flop_0)` as a
pathological form: a mux with both data arms connected to the
same signal is structurally redundant (equivalent to the data
signal alone). Rule 8 already forbade the 2-to-1 case at the
`Mux` gate level, but `make_mux` bypassed the anti-collapse
check when called from the chained-ternary assembly. The
broader N-to-1 generalisation — "m arms out of M share the same
data" — was uncontrolled until this slice.

The knob exists because the pathological form is genuine
downstream-tool input: we want to emit it *on request* (for
stress testing) but not by default.

**Tests**
- All four cargo gates green.
- 32 unit + 15 integration = **47 tests, all passing**.
- Verified knob behavior at seed 42:
  - Default (rate = 0.0): 17 ternary expressions, **0** with
    the degenerate `(X)?(Y):(Y)` shape.
  - `--mux-arm-duplication-rate 1.0`: 11 ternary expressions,
    **1** degenerate (chained-ternary layers collapse more
    often when arms repeat → fewer total mux nodes).

**Impact**
- Default output no longer contains any `(s)?(x):(x) = x`
  redundant-mux lines. Module semantics unchanged.
- At high rates, downstream synthesis tools see redundant-arm
  patterns for stress coverage.

---

## 2026-04-16-0037 — Construction-time CSE with tunable AST-instance cap (Rule 21)

**What changed**
- `src/ir/types.rs`:
  - `Module` gains `gate_instances: HashMap<(GateOp, Vec<NodeId>, u32), Vec<NodeId>>`,
    `const_instances: HashMap<(u32, u128), Vec<NodeId>>`, and
    `max_ast_instances: u32`.
  - New methods `Module::intern_gate(op, operands, width, deps) → (NodeId, is_new)`
    and `Module::intern_constant(width, value) → (NodeId, is_new)`.
    Cap behavior: create new if `vec.len() < max_ast_instances`,
    else return `*vec.last()`.
  - `GateOp` gains `Hash` derive.
- `src/config.rs`: new knob `max_ast_instances: u32` (default 1 = strict CSE).
  Threaded through `Overrides` and `apply_cli_overrides`.
- `src/main.rs`: new CLI flag `--max-ast-instances <N>`.
- `src/gen/module.rs`: `generate_leaf_module` sets
  `m.max_ast_instances = g.cfg.max_ast_instances.max(1)`.
- `src/gen/cone.rs`: every `m.nodes.push(Node::Gate|Constant)` site
  migrated to `intern_gate` / `intern_constant`. Callers only
  `pool.add` when `is_new = true`. Helpers: `make_constant`,
  `make_eq_const`, `make_mux`, `make_and`, `make_mul`, `make_sub`,
  `make_nary_add`, `make_nary_mul`, `replicate_to_width`,
  `or_reduce_terms`, `make_width_adapter`, the deliver-path in
  `process_signal_frame`, the operator-gate-creation block in
  `grow_pool_one_unit`, `build_cone`, and `pick_terminal`'s fresh-
  constant fallback.
- Critical snapshot fix: `build_cone_with_retry` now snapshots and
  restores `m.gate_instances` and `m.const_instances` alongside
  `m.nodes` / `m.flops` / pool / worklist. Without this, a retry
  rolls back the node vec but leaves stale dedup entries pointing
  at truncated NodeIds, causing `intern_gate` to return nodes of
  wrong kind/width on subsequent calls.
- `book/src/structural-rules.md`: new Rule 21 "AST-instance cap
  (construction-time CSE)" documenting the rule, motivation,
  enforcement, and snapshot/rollback interaction.

**Why**
User flagged observable RHS duplication:
`eq_4 = slice_17 == 2'h2; … eq_9 = slice_17 == 2'h2; …`.
Construction-time hash-consing is the right answer — one RHS =
one signal = one node. But blanket CSE is too opinionated for a
stress-test generator, so the cap is a knob: default 1 (strict
CSE), raise for bounded duplication, `u32::MAX` to disable.

**Tests**
- All four cargo gates green.
- 32 unit + 15 integration = **47 tests, all passing**.
- Spot-check seed 42: `slice_17 == 2'h2` now appears exactly once
  (`eq_0`). Downstream muxes reference `eq_0` instead of creating
  copies. Verified knob behavior: at `--max-ast-instances 3`, Eq
  count doubles from 6 to 12.

**Impact**
- **Structural change.** Modules generated under default knobs are
  smaller (fewer nodes) and more shared. The SV is semantically
  equivalent to the pre-CSE output for the same `(seed, knobs)`
  only when `max_ast_instances = u32::MAX`. Otherwise output
  differs and is denser.
- Integration tests needed to account for the interaction between
  dedup and retry rollback; the snapshot-restore of dedup tables
  is the load-bearing piece.

---

## 2026-04-16-0036 — Emit `{N{expr}}` replication for same-operand Concat

**What changed**
- `src/emit/sv.rs` `render_gate` for `GateOp::Concat`: when every
  operand points at the same `NodeId`, emit the canonical SV
  replication form `{N{expr}}` instead of the flat list
  `{expr, expr, …, expr}`. Covers the `replicate_to_width` helper's
  output in one-hot mux assembly (`{W{sel_i}} & data_i`).
- Emitter unit test updated to expect `{2{a}}` instead of `{a, a}`.

**Why**
User flagged lines like
`assign concat_15 = {eq_0, eq_0, …, eq_0};` (22 copies) as "uncontrolled."
The logic is intentional (one-hot mask broadcast) but the expanded
emission form hid the idiom. The replication form is synthesis-
equivalent and matches the SV convention every synthesizer already
recognizes.

**Tests**
- All four cargo gates green.
- 32 unit + 15 integration = **47 tests, all passing**.
- Spot-check seed 42: former 22-operand `concat_15 = {eq_0, eq_0, …}`
  now emits as `concat_15 = {22{eq_0}}`. Gate count and module
  semantics unchanged.

**Impact**
- No behavior change — only emission format. Byte-identical SV
  structure modulo the replication shortcut.
- Any downstream tool that parsed the flat-list form sees the
  replication form now, which is standard SV and synthesized
  identically.

---

## 2026-04-16-0035 — UVM-style tracing (`--trace` / `--trace-file`)

**What changed**
- `Cargo.toml`: adds `tracing` (with `release_max_level_info`) and
  `tracing-subscriber`.
- `src/main.rs`: new CLI flags `--trace <off|low|medium|high|debug>`
  (default `off`) and `--trace-file <path>`. `init_tracing`
  initialises a deterministic subscriber — no timestamps, no thread
  IDs, no ANSI — with output to stderr (default) or a file. Level
  mapping: `low = INFO`, `medium = DEBUG`, `high = TRACE`,
  `debug = TRACE`.
- `src/gen/module.rs`: `#[instrument(level="info")]` on
  `generate_leaf_module`; milestone logs for module start/done with
  n_in, n_out, strategy, final node/flop/drive counts.
- `src/gen/cone.rs`: `#[instrument]` on `build_cone_with_retry` (debug),
  `build_graph_first` (info), `grow_pool_one_unit` (trace),
  `build_outputs_interleaved` (info), `process_signal_frame` (trace),
  `build_cone` (trace), `drain_flop_worklist` (debug),
  `build_comb_mux` (trace), `build_flop_leaf` (trace),
  `pick_terminal` (trace), `pick_terminal_dep_bearing` (trace).
  Explicit `trace!` / `debug!` / `warn!` at named control points:
  motif dispatch forks (flop / comb-mux / priority-encoder / operator
  gate, linear-combination, const-shift, const-comparand), retry /
  fallback (cone retry, anti-collapse retry and exhaustion, terminal
  tier 1/2/3/4 picks), leaf vs recursion decision. Emoji tags at
  milestone / retry / fallback events only.
- `src/emit/sv.rs`: `#[instrument(level="info")]` on `to_sv`; info-
  level summary of gates/flops/inputs/outputs; debug-level dump of
  per-kind counter totals from `build_names`. `build_names` now uses
  `BTreeMap` instead of `HashMap` for deterministic counter-log
  ordering (no iteration-order leak into trace output).
- `USER_GUIDE.md`: new "Tracing and debugging" section with level
  table, CLI examples, and emoji legend.

**Why**
Per user direction: generator debugging needs UVM-style graduated
verbosity with broad coverage. The project's "by construction"
contract makes *silent* bugs the main debugging hazard (wrong motif
dispatch, unexpected retry / fallback paths, width-adapter surprises)
— tracing at the named control points is the cheapest way to surface
them without touching generator logic.

**Non-negotiables honored**
- Output goes to stderr (or file); stdout stays byte-clean for SV.
- No wall-clock, no thread IDs, no ANSI, no hash-map iteration in
  trace output. Verified: `diff` of `--trace off` vs `--trace high`
  generated SV is empty for the same `(seed, knobs)`.
- Release build compiles out below `info` via the
  `release_max_level_info` feature flag.

**Tests**
- All four cargo gates green.
- 32 unit + 15 integration = **47 tests, all passing** with default
  `--trace off`.
- Reproducibility spot-check: `--trace off` and `--trace high` on
  seed 42 produce byte-identical stdout.

**Impact**
- `--trace off` is the default — zero behavioral change for existing
  users or CI.
- Release builds compile out below `info`; `low` / default (off)
  have near-zero overhead. `high` / `debug` add measurable overhead
  and should not be used in seed sweeps.
- No CLI flag was renamed; only additions.

---

## 2026-04-16-0034 — Typed per-kind naming in emitted SV (Rule 12 revised)

**What changed**
- `src/emit/sv.rs`:
  - `build_names(m) -> Vec<Option<String>>`: single-pass walk that
    assigns each `Node::Gate` a name `<kind>_<counter>` with the
    counter maintained per `GateOp` kind. Non-gate nodes get
    `None`.
  - `gate_kind_name(op) -> &'static str`: canonical lowercase
    prefix for each `GateOp` variant (`and`, `or`, `xor`, `not`,
    `add`, `sub`, `mul`, `eq`, `neq`, `lt`, `gt`, `le`, `ge`,
    `mux`, `slice`, `concat`, `red_and`, `red_or`, `red_xor`,
    `shl`, `shr`).
  - `flop_name(id) -> String`: `flop_<id>`.
  - `node_ref` / `render_gate` threaded with the `&[Option<String>]`
    name table. Non-gate nodes resolve as before (primary input
    port name, literal constant, flop Q = `flop_<id>`).
- Emitter unit tests updated to the new naming: `flop_0` replaces
  `r_0`; gate references become `and_0`, `xor_0`, `mux_0`,
  `slice_0`, `concat_0`.
- `book/src/structural-rules.md`: Rule 12 rewritten — table now
  shows `<gate_kind>_N` and `flop_N`; lists all kind prefixes;
  explains per-kind counter rationale; documents SV identifier
  legality for gate-primitive-prefixed names (`and_0` is a legal
  identifier distinct from the `and` keyword).

**Why**
Per user direction — generated SV must be inspectable at a glance.
The opaque `w_<NodeId>` scheme collapsed all structural variety
into a uniform wire name; `<kind>_<counter>` makes the gate mix
visible and aligns emitted SV with the `GateOp` taxonomy already
used in the IR.

**Tests**
- All four cargo gates green.
- 32 unit + 15 integration = **47 tests, all passing**.
- Spot-check: `cargo run -- --seed 42` now emits
  `flop_0 … flop_9`, `and_0 … and_N`, `slice_0`, `mux_0`, `concat_0`
  and similar. No `w_<N>` / `r_<N>` identifiers remain.

**Impact**
- **Breaking for downstream tools that parsed the old `w_` / `r_`
  naming.** No users yet; the change is internal to a pre-release
  generator.
- Reproducibility contract holds: names are a deterministic
  function of declaration order, which is itself a deterministic
  function of `(seed, knobs)`.
- No IR or generator changes — naming is emission-time only.
- Block-level names (`priority_encoder_0`, `comb_mux_N`) are
  deferred: today blocks decompose into gate chains at
  construction time with no IR-level block identity to attach a
  name to. Follow-up slice if needed.

---

## 2026-04-16-0033 — N-arity anti-collapse + OR-reduce dedup (Rule 8 extended)

**What changed**
- `src/gen/cone.rs`:
  - `violates_anti_collapse` now catches duplicates at any arity
    for idempotent / self-inverse operators (`And`, `Or`, `Xor`).
    Helper `has_duplicate_operand` does the operand-multiset
    distinctness check (O(N²), N bounded by `max_gate_arity`).
    `Add` and `Mul` deliberately remain exempt (duplicates are
    algebraically meaningful).
  - `or_reduce_terms` deduplicates input terms before building
    the 2-arity `Or` chain, so identical per-arm products do not
    produce `x | x` gates.
  - `make_none_selected` (QFeedback one-hot fall-through) now
    routes through `or_reduce_terms`, inheriting the dedup.
  - New unit test `anti_collapse_catches_nary_duplicates` pins
    the broadened check on Xor/And/Or at arities 3 and 4 (with
    and without duplicates) and confirms Add/Mul are not
    flagged.
- `book/src/structural-rules.md`: Rule 8 rewritten to state the
  N-arity rule explicitly; lists the exempt ops (Add, Mul); notes
  the downstream dedup in `or_reduce_terms` / `make_none_selected`.

**Why**
Sample module `mod_1_0000` contained `w_21 = i_2 ^ i_2 ^ i_2 ^ i_2`
(constant 0) and multiple identical one-hot arms producing
downstream `w_A | w_A` gates. The pairwise `operands[0] ==
operands[1]` check in the old `violates_anti_collapse` did not cover
these. Root-cause fix per the rule-based-generation doctrine.

**Tests**
- All four cargo gates green.
- 32 unit (+1 new) + 15 integration = **47 tests, all passing**.
- Spot-check across 8 seeds (1, 2, 3, 42, 100, 777, 9999, 12345):
  zero self-operand chains (`x OP x`) in generated SV. Seed 100
  previously emitted `w_120 = w_104 | w_104` from
  `make_none_selected`; now clean.

**Impact**
- Default config paths produce strictly higher-entropy gate
  operand lists: `And`/`Or`/`Xor` never repeat an operand. The
  pick-retry path absorbs the rare case where the picker
  re-selects a duplicate; after retry exhaustion it falls back to
  `pick_terminal`.
- No CLI or config-surface change.

---

## 2026-04-16-0032 — Dep-bearing source at elaboration-sensitive positions (Rule 20)

**What changed**
- `src/gen/cone.rs`: new `pick_terminal_dep_bearing(g, m, pool,
  width, exclude)` helper. Two-tier picker: (1) random dep-bearing
  matching-width pool entry; (2) width-adapter from widest
  dep-bearing pool entry of any width. Panics if the pool has no
  dep-bearing entry at all (invariant — primary inputs are always
  seeded with non-empty deps).
- Seven pool-mode call sites migrated from `pick_terminal` to
  `pick_terminal_dep_bearing`:
  - `grow_pool_one_unit`: const-shift value operand, const-comparand
    LHS.
  - `build_comb_mux_pool_only`: encoded sel, one-hot per-arm sel.
  - `drain_flop_worklist_pool_only`: encoded sel, one-hot per-arm
    sel.
  - `build_priority_encoder_pool`: request bits.
- New unit test `pick_terminal_dep_bearing_rejects_constants` (100
  iterations against a pool polluted with a matching-width
  dep-empty constant).
- `book/src/structural-rules.md`: new Rule 20 "Dep-bearing source
  required at elaboration-sensitive positions" with the four
  positions covered, motivation, and enforcement.

**Why**
Sample module `mod_1_0000` contained `w_35 = 2'h2 == 2'h2` — a
comparison with both operands literal, folding to a constant 1 at
elaboration. Root cause: the comparison's LHS picker
(`pick_terminal`) permits dep-empty pool entries in its tier-2
fallback and, at tier 4, emits a fresh constant. The same hazard
applies to mux selects and priority-encoder request bits. Fixed at
the root per the user's rule-based-generation doctrine.

**Tests**
- All four cargo gates green.
- 31 unit (+1 new) + 15 integration = **46 tests, all passing**.
- Spot-check across six seeds (1, 2, 3, 42, 100, 777): zero
  literal-vs-literal comparison lines in generated SV (was several
  per module before).

**Impact**
- Default config paths produce muxes, priority encoders, comparisons,
  and shifts whose selects / LHS / value sides are now always
  dep-bearing signals (primary input or flop Q, possibly adapted by
  Slice/Concat).
- No CLI or config-surface change.

---

## 2026-04-15-0031 — Coefficient fits operand width (Rule 19)

**What changed**
- `src/gen/cone.rs`:
  - `pick_coefficient(g)` → `pick_coefficient(g, width)`. The picker
    now narrows the draw range to
    `[max(min_coefficient, 1), min(max_coefficient, 2^W − 1)]` so a
    coefficient can never overflow the `W`-bit `Constant` node it
    will be emitted as.
  - `pick_mul_coefficient_and_arity(g)` → `(g, width)`, threads
    through.
  - All three callers (`assemble_add_linear_combination`,
    `assemble_sub_linear_combination`, the `Mul` arms of
    `build_linear_combination_recursive` /
    `build_linear_combination_pool`) pass their local `width`.
  - New unit test `pick_coefficient_respects_target_width` pins the
    width-aware clamp across widths 1, 2, 4, 8 (200 iterations).
- `book/src/structural-rules.md`: new Rule 19 "Coefficient fits
  operand width" with motivation, enforcement, and edge case
  (`width = 1` → coefficient is always 1).

**Why**
Sample module `mod_1_0000` contained `1'h6`, a 6 in a 1-bit literal —
the emitter truncates it to `1'h0`. Root cause: `pick_coefficient`
drew from `[min_coefficient, max_coefficient]` without reference to
the operand width. This slice fixes the bug at the root (the
picker) rather than with a post-hoc filter, per the user's "rule-
based generation" doctrine.

**Tests**
- All four cargo gates green.
- 30 unit (+1 new) + 15 integration = **45 tests, all passing**.

**Impact**
- Default config paths unaffected for `width ≥ 4` (unclamped range
  fits). Width-1 paths now emit `1'h1` constants instead of
  truncating larger values. Width-2/3 paths see slightly tighter
  distributions.
- No CLI or config-surface change (range knobs still exist and
  still accept their original values; clamping is silent per-call).

---

## 2026-04-15-0030 — Rule 18 proposal + sample-output defect catalogue (docs only)

**What changed**
- `book/src/structural-rules.md`: add Rule 18 "No orphan gates"
  (proposed, not yet enforced). Documents the rule, motivation,
  status, and the two enforcement paths under consideration:
  (α) construction-time demand-driven vs (β) emission-time
  tree-shake. Decision deferred.
- `DEVELOPMENT_NOTES.md`: new section "Generation-time defects
  observed in sample output (pending fixes)" cataloguing six
  concrete defects seen in sample module `mod_1_0000`:
  constant-select muxes, N-arity self-cancellation
  (`i_2^i_2^i_2^i_2 = 0`), coefficient width overflow (`1'h6`),
  dead wires, stranded flop (`r_3<=r_3`), structurally-identical
  one-hot arms. Attributes each to a root cause and sketches a
  fix. Three categories: anti-collapse operand-multiset, position-
  dependent leaf rules, width-aware coefficient generation — plus
  the orthogonal orphan-gate axis covered by Rule 18.

**Why**
User flagged the anomalies in a generated sample module and framed
the issue philosophically: "when block or gate is created it is
before it needs to be used, connected somewhere... some of those
signals are created with no proper reason." This slice captures the
observations so the next session can fix the defects at the root
rather than rediscovering them.

**Tests**
No code changed. `cargo fmt / build / clippy / test` unchanged from
the previous commit.

---

## 2026-04-15-0029 — Priority-encoder block (Rule 17)

**What changed**
- `src/config.rs`: new `priority_encoder_prob` knob (default 0.05).
  New `CoefficientRange`-style error handling for the probability
  (via the existing probability-range loop). Threaded through
  `Overrides` and `apply_cli_overrides`.
- `src/main.rs`: new CLI flag `--priority-encoder-prob`.
- `src/gen/cone.rs`:
  - `pick_priority_encoder_n(g, target_width) -> Option<u32>`: finds an
    N ∈ `[min_mux_arms, max_mux_arms]` with
    `ceil_log2(N) == target_width`. Returns None if none fits the range.
  - `assemble_priority_encoder(m, pool, target_width, req_bits) -> NodeId`:
    emits the chained ternary `req_0 ? 0 : req_1 ? 1 : ... : 0`.
    Every priority level becomes one `Mux` node; the output width is
    `target_width`.
  - `build_priority_encoder_recursive` / `build_priority_encoder_pool`:
    dispatch helpers that source request bits via `build_cone` or
    `pick_terminal` respectively.
  - Three dispatch sites (`build_cone`, `process_signal_frame`,
    `grow_pool_one_unit`) call the appropriate build helper. Dispatch
    has applicability-check-then-fall-through semantics: if no N
    matches the target width, the block roll is wasted and the code
    continues to the usual operator gate path.
- `tests/pipeline.rs`: new
  `priority_encoder_block_across_all_strategies_is_valid` — all four
  strategies × 5 seeds × `priority_encoder_prob = 1.0` must produce
  IR-valid modules. Uses `max_depth = 3` to bound test runtime under
  heavy PE recursion.
- `book/src/structural-rules.md`:
  - New Rule 17 describing the priority-encoder block: shape,
    applicability constraint (`ceil_log2(N) == W`), fall-through
    convention, and the place it lives in the generator.
  - Operators-vs-blocks preamble gains an entry for the priority-
    encoder block.
- `book/src/knobs.md`: new "Priority-encoder block" subsection.
- `USER_GUIDE.md`: `--priority-encoder-prob` row added.
- `CODEBASE_ANALYSIS.md`: `cone.rs` module map extended.
- `MEMORY.md`: last-completed-slice refreshed; next-up list
  re-scoped per user direction ("close all small-to-medium first")
  into case/casez → memories → FSMs → motif-trait refactor, with
  hierarchy / parameterization deferred.

**Why**
Per user direction to "close all small to medium first" ahead of
the large Phase 3+ items (hierarchy, parameterization). Priority
encoder is the smallest self-contained block motif on the list —
clean shape, single-output-width applicability check, no Q-feedback
or mux-style variant axes. Also a classic synthesizer idiom
(arbitration, interrupt-level encoding, one-hot-to-binary
conversion) worth exercising.

**Doctrinal note.** User observed mid-slice that every new block
follows the same pattern (knob, assembly helper, three dispatch
sites, tests, docs) and asked whether a motif library is feasible.
Agreed in principle; deferred until we have 6-8 concrete block
motifs to factor from (currently mux, flop-mux-family, comb-mux,
priority-encoder, with case/casez/memories/FSMs planned next).

**Validation**
- `cargo check --all-targets`, `cargo test` (29 unit + 15
  integration = 44 tests), `cargo clippy --all-targets -- -D
  warnings`, `cargo fmt --all --check`: all clean.
- End-to-end at `--priority-encoder-prob 1.0`: emitted SV contains
  chained ternaries like
  `assign w_18 = (w_3) ? (3'h0) : (w_16);`
  `assign w_16 = (w_3) ? (3'h1) : (w_14);`
  `... assign w_6 = (w_3) ? (3'h6) : (3'h0);` — full 7-level PE.

**Impact**
- First Phase 3+ block motif landed.
- Pattern for the remaining small-to-medium motifs (case/casez,
  memories, FSMs) is now well-established.

**Files touched**
`src/config.rs`, `src/main.rs`, `src/gen/cone.rs`, `tests/pipeline.rs`, `book/src/structural-rules.md`, `book/src/knobs.md`, `USER_GUIDE.md`, `CODEBASE_ANALYSIS.md`, `MEMORY.md`, `CHANGES.md`.

**Commit hash:** `b4c489a`

---

## 2026-04-15-0028 — Flop-assembler unit tests + FAQ chapter

**Commit hash:** `06b5a52`

**What changed**
- `src/gen/cone.rs`: 4 new inline unit tests for the flop-D assembly helpers.
  - `assemble_flop_d_one_hot_zero_default_top_is_or` — verifies the OneHot/ZeroDefault emission produces an OR-rooted tree of width W.
  - `assemble_flop_d_one_hot_qfeedback_includes_q_term` — verifies the QFeedback variant adds a Not for `~(OR of sels)` and preserves the OR root.
  - `assemble_flop_d_encoded_zero_default_top_is_mux` — verifies the Encoded/ZeroDefault chained-ternary top-level is a Mux of width W.
  - `assemble_flop_d_encoded_qfeedback_fallthrough_is_q` — verifies QFeedback+Encoded with M=2 still builds a Mux-rooted tree when index 0 is Q.
  - New test-fixture helpers `fixture_with_inputs` (n wide + n 1-bit primary inputs seeded into pool) and `alloc_flop` (register a flop + FlopQ node). Reusable across future flop-assembler tests.
- `book/src/faq.md` (new): 12-entry FAQ chapter answering vocabulary/doctrine questions that have come up during design discussion. Topics: Sub non-associativity, operators-vs-blocks vocabulary, Q-feedback-vs-combinational-no-loop, coefficient vs shift-amount vs comparand roles, four construction strategies rationale, cross-output sharing, reproducibility, testbench non-goal, synthesizability guarantee, "meaningful logic" disclaimer, SV language standard targeting, clk/rst_n port emission, version-to-version reproducibility, Verilator/Yosys invocation.
- `book/src/SUMMARY.md`: FAQ added to Reference section.
- `CODEBASE_ANALYSIS.md`: testing surface updated — 11 cone unit tests (was 7), 43 total (was 39).
- `MEMORY.md`: last-completed slice refreshed; next-up list re-scoped per user direction ("switch to Phase 3+ since Verilator unavailable") with 6 ranked candidate scopes.

**Why**
Per `MEMORY.md` next-up item (2): the flop-D assembly helpers were previously covered only indirectly by the pipeline integration sweep. Direct unit tests give faster feedback on their top-level shape invariants and pin the expected emission forms (OR root for OneHot, Mux root for Encoded, extra Not for QFeedback+OneHot). Tests are shape-level rather than exact-node-count to stay robust under future refactor.

The FAQ chapter consolidates the doctrine questions that accumulated across ~15 slices of vocabulary / design / rule-catalog work. It's the user-facing entry point for "why is `anvil` shaped this way" without having to dig through the structural-rules catalog or commit history.

**Validation**
- `cargo check --all-targets`, `cargo test` (29 unit + 14 integration = 43 tests), `cargo clippy --all-targets -- -D warnings`, `cargo fmt --all --check`: all clean.
- `mdbook build book` succeeds with the new FAQ chapter rendered.

**Impact**
- Flop-assembler regressions now caught at the unit level, not just when the pipeline sweep happens to fail.
- Book gains a welcoming "why" entry point. Users arriving cold have a fast path to understanding anvil's doctrine without reading the full structural-rules catalog.

**Files touched**
`src/gen/cone.rs`, `book/src/faq.md` (new), `book/src/SUMMARY.md`, `CODEBASE_ANALYSIS.md`, `MEMORY.md`, `CHANGES.md`.

**Commit hash:** `06b5a52`

---

## 2026-04-15-0027 — Constant comparand motif: third and final constant-role motif

**Commit hash:** `1211120`

**What changed**
- `src/config.rs`: three new knobs.
  - `const_comparand_prob` (default 0.3): per-comparison probability the RHS is a constant literal instead of a recursive signal cone.
  - `min_comparand` (0), `max_comparand` (255): value range, clamped to `[0, 2^K - 1]` for the chosen internal operand width K.
  - Threaded through `Overrides`, `apply_cli_overrides`, and the probability-range validation loop.
- `src/main.rs`: three new CLI flags.
- `src/gen/cone.rs`:
  - New helpers: `pick_comparison_operand_width` (matches `input_widths_for`'s 1..=8 draw), `pick_comparand_value` (clamped range draw), `build_comparison_const_comparand` (emits `lhs_signal OP const` with 1-bit output), `is_comparison_op` (predicate).
  - Three dispatch sites after `pick_gate` returns a comparison op: `build_cone` (recursive LHS), `process_signal_frame` (interleaved recursive LHS), `grow_pool_one_unit` (graph-first pool-only LHS).
- `tests/pipeline.rs`: new `const_comparand_across_all_strategies_is_valid` — `const_comparand_prob = 1.0` × all four strategies × 5 seeds, all IR-valid.
- `book/src/structural-rules.md`: "Roles of constants in RTL" → Comparand subsection updated. Previous "Not yet emitted by anvil" note retired.
- `book/src/knobs.md`: new "Comparand motif" subsection.
- `USER_GUIDE.md`: three new CLI flag rows.
- `CODEBASE_ANALYSIS.md`: `cone.rs` module map extended.
- `MEMORY.md`: current-state refreshed; next-up list trimmed (all three constant-role motifs done). Recent commits list gains `2da9d3d`.

**Why**
Third and final constant-role motif from the catalog. Comparisons in real RTL frequently have constant RHS (`state == IDLE`, `counter >= LIMIT`, `error_code != 0`) — a threshold / sentinel / target pattern, not a coefficient. Per the vocabulary-discipline doctrine the motif has its own knob family, distinct from coefficients and shift amounts.

The motif is *additive* to signal-vs-signal comparisons: when the coin doesn't fire, the existing path emits both operands as recursive signals. Users who want purely-signal comparisons pin `--const-comparand-prob 0.0`; users who want threshold-stress pin `1.0`.

**Validation**
- `cargo check --all-targets`, `cargo test` (25 unit + 14 integration = 39 tests), `cargo clippy --all-targets -- -D warnings`, `cargo fmt --all --check`: all clean.
- End-to-end: on seed 1 with `--const-comparand-prob 1.0 --max-width 1`, emitted SV contains `assign w_10 = w_7 == 2'h3;`, `assign w_24 = w_22 == 8'h7a;`, etc.

**Impact**
- All three constant-role motifs implemented. The generator now emits realistic RTL idioms across the three semantic roles for integer literals.
- Phase 1/2 feature work is effectively done; remaining exit gate is the Verilator-lint smoke run across construction strategies and motif probabilities.

**Files touched**
`src/config.rs`, `src/main.rs`, `src/gen/cone.rs`, `tests/pipeline.rs`, `book/src/structural-rules.md`, `book/src/knobs.md`, `USER_GUIDE.md`, `CODEBASE_ANALYSIS.md`, `MEMORY.md`, `CHANGES.md`.

---

## 2026-04-15-0026 — Constant shift-amount motif + Shl/Shr added to pick_gate

**Commit hash:** `2da9d3d`

**What changed**
- `src/config.rs`: four new knobs.
  - `const_shift_amount_prob` (default 0.8): per-shift probability the amount operand is a constant literal instead of a variable-amount signal (barrel shifter).
  - `min_shift_amount` (default 0) / `max_shift_amount` (default 7): range for the drawn constant amount, clamped to `[0, W-1]` for a W-bit value.
  - `gate_shift_weight` (default 1): relative weight for the shifts bucket in `pick_gate`.
  - Threaded through `Overrides`, `apply_cli_overrides`, and the probability-range validation loop.
- `src/main.rs`: four new CLI flags.
- `src/gen/cone.rs`:
  - `pick_gate` now has a fifth bucket (`shifts: &[Shl, Shr]`) with weight `gate_shift_weight`. Shifts are disabled at `target_width == 1` (shift on a 1-bit value is trivial).
  - New helpers: `pick_shift_amount` (draws from `[min_shift_amount, max_shift_amount]` clamped to `[0, value_width-1]`), `build_shift_const_amount` (emits `value OP const` — a single 2-operand Shl/Shr node with a compact-width constant).
  - Three dispatch sites after `pick_gate` returns Shl/Shr:
    - `build_cone` (sequential / shuffled / interleaved-block-internal paths): value from recursive `build_cone`.
    - `process_signal_frame` (interleaved top-level): value from recursive `build_cone` at `depth+1`.
    - `grow_pool_one_unit` (graph-first): value from `pick_terminal`.
- `tests/pipeline.rs`: new `const_shift_amount_appears_in_output` — 32-seed sweep at `const_shift_amount_prob = 1.0, gate_shift_weight = 10` must produce at least one `<< N'hX` or `>> N'hX` emission.
- `book/src/structural-rules.md`: "Roles of constants in RTL" → Shift Amount subsection updated with the knob list and the implementation site; previous "today always variable-amount" note retired.
- `book/src/knobs.md`: new "Shift-amount motif" subsection.
- `USER_GUIDE.md`: four new CLI flag rows.
- `CODEBASE_ANALYSIS.md`: `cone.rs` module map extended.
- `MEMORY.md` / `CHANGES.md`: per workflow.

**Why**
Per MEMORY next-up item 1 and the roles-of-constants doctrine. Shifts in real RTL are predominantly constant-amount (wire reroutes, cheap) rather than variable-amount barrel shifters. The default probability is set high (0.8) to match that prevalence; users wanting to stress barrel-shifter synthesis can lower it to 0.0 for purely variable amounts.

Adding Shl/Shr to `pick_gate` fixes a longstanding absence — the shifts were defined in `GateOp` and `input_widths_for` but never selectable. Same pattern as the earlier Mul fix (slice `2026-04-15-0025`).

The knob set is its own family — distinct from `coefficient_prob` and (future) `const_comparand_prob`. Per the vocabulary-discipline doctrine, "shift amount" is a structural parameter, not a coefficient.

**Validation**
- `cargo check --all-targets`, `cargo test` (25 unit + 13 integration = 38 tests), `cargo clippy --all-targets -- -D warnings`, `cargo fmt --all --check`: all clean.
- End-to-end spot check at `--const-shift-amount-prob 1.0`: emitted SV contains `w_X >> 3'h5`, `w_X << 2'h3`, `w_X << 1'h0`, etc. Both operator directions and a range of amounts observed.

**Impact**
- Generated RTL now routinely includes constant-amount shifts — the dominant pattern in real datapaths (scaling by powers of two, alignment, field extraction).
- Barrel-shifter stress is still reachable by pinning `--const-shift-amount-prob 0.0`.
- Two of three constant-role motifs now implemented (coefficients ✅, shift amounts ✅); comparands remain.

**Files touched**
`src/config.rs`, `src/main.rs`, `src/gen/cone.rs`, `tests/pipeline.rs`, `book/src/structural-rules.md`, `book/src/knobs.md`, `USER_GUIDE.md`, `CODEBASE_ANALYSIS.md`, `MEMORY.md`, `CHANGES.md`.

---

## 2026-04-15-0025 — Linear-combination coefficient motif for Add / Sub / Mul

**Commit hash:** `7290e3d`

**What changed**
- `src/config.rs`: three new knobs —
  - `coefficient_prob` (default 0.2): per-op probability that Add/Sub/Mul emits the linear-combination compound form instead of a standard operator.
  - `min_coefficient` (default 1) and `max_coefficient` (default 15): strictly-positive integer coefficient range.
  - New `ConfigError::CoefficientRange`; validation enforces `1 <= min <= max`.
  - Overrides and `apply_cli_overrides` wired for all three.
- `src/main.rs`: CLI flags `--coefficient-prob`, `--min-coefficient`, `--max-coefficient`.
- `src/gen/cone.rs`:
  - Added `Mul` to `pick_gate`'s arith bucket (was absent — so the Mul compound could never previously fire).
  - New helpers: `make_mul`, `make_sub`, `make_nary_add`, `make_nary_mul`, `pick_coefficient`, `pick_linear_combination_arity`, `pick_mul_coefficient_and_arity`.
  - New assemblers: `assemble_add_linear_combination` (N Mul + one N-ary Add), `assemble_sub_linear_combination` (N Mul + N-1 chained 2-ary Sub), `assemble_mul_linear_combination` (one N+1-ary Mul with leading constant).
  - New dispatchers: `build_linear_combination_recursive` (signals via `build_cone`) and `build_linear_combination_pool` (signals via `pick_terminal`).
  - Three dispatch sites inserted:
    - `build_cone` (sequential/shuffled paths): after `pick_gate`, before operand loop.
    - `process_signal_frame` (interleaved): after `pick_gate`, before frame/operand enqueue. Compound tree built synchronously within the frame step (like blocks).
    - `grow_pool_one_unit` (graph-first): after `pick_gate`, before standard operand pool-pick loop.
- `tests/pipeline.rs`:
  - New `coefficient_motif_emits_compound_shapes`: 16-seed sweep at `coefficient_prob=1.0` must produce at least one front-constant Mul expression (`<W>'h<hex> * w_...`).
  - New `coefficient_motif_across_all_strategies`: all four construction strategies × 5 seeds × `coefficient_prob=1.0` must produce valid IR.
- `book/src/structural-rules.md` "Roles of constants in RTL" → Coefficient subsection updated with:
  - Mul shape `y = c * s1 * s2 * … * sn` spelled out.
  - `c >= 1`; `c == 1` forces `N >= 2`.
  - Knob list (`coefficient_prob`, `min_coefficient`, `max_coefficient`).
- `book/src/knobs.md`: new "Coefficient motif (linear combinations)" subsection.
- `USER_GUIDE.md`: three new CLI flag rows.
- `CODEBASE_ANALYSIS.md`: `cone.rs` module map extended with the new helpers, assemblers, and dispatchers.
- `MEMORY.md`: last-completed-slice refreshed; next-up list trimmed — coefficient motif done, shift-amount bias is now item 1.

**Why**
Per user direction: arithmetic operators benefit from a compound linear-combination motif that emits realistic RTL idioms (`3*a + 2*b + c` for Add, `a*5 - b*3` for Sub, `c * s1 * s2 * s3` for Mul). Constants in this role are **coefficients** (multiplicative weights), distinct from shift amounts or comparands. Per-op constraints:
- Add: `ci ≠ 0` (non-zero). Implementation uses positive-only.
- Sub: `ci > 0` (strictly positive). Negative would flip to Add contribution.
- Mul: single `c >= 1` scalar multiplier; `c == 1` forces `N >= 2` to avoid the dead `1 * s1 = s1` case.

This lands the first of three constant-role motifs (coefficients → shift amounts → comparands) the project committed to. Each has its own knob family per "do not collapse into a single `constant_prob` knob" doctrine.

**Validation**
- `cargo check --all-targets`, `cargo test` (25 unit + 12 integration = 37 tests), `cargo clippy --all-targets -- -D warnings`, `cargo fmt --all --check`: all clean.
- End-to-end spot check at `--coefficient-prob 1.0`:
  - `assign w_9 = 11'h2 * w_7 * w_7;` (Mul compound, c=2, 2 signals)
  - `assign w_12 = 26'hc * w_10 * w_10 * w_10 * w_10;` (Mul compound, c=12, 4 signals)
  - `assign w_49 = 5'hf * w_30 * w_32 * w_47;` (Mul compound, c=15, 3 distinct signals)
  - `assign w_32 = w_30 * 5'h8; w_34 = w_30 * 5'hc; ... w_44 = w_39 + w_41 + w_43;` (Add compound terms + N-ary sum)
  - `assign w_22 = w_15 - w_17; w_23 = w_22 - w_19; w_24 = w_23 - w_21;` (Sub compound chain)

**Impact**
- Generated RTL now routinely contains realistic arithmetic datapath idioms (scaled-sum accumulators, weighted differences, product chains with constant multipliers).
- `Mul` is now selectable by `pick_gate` (previously omitted from the menu). This also means the non-coefficient Mul path can now emit binary multipliers even when the coefficient motif doesn't fire.
- Three planned constant-role motifs: one done, two to go (shift amounts, comparands).

**Known simplification**
- Add's theoretical `ci ≠ 0` allows negative coefficients; the implementation draws positive-only from `[min_coefficient, max_coefficient]`. Signed-negative coefficients are a future extension. Sub's `ci > 0` and Mul's `c >= 1` are honored exactly.

**Files touched**
`src/config.rs`, `src/main.rs`, `src/gen/cone.rs`, `tests/pipeline.rs`, `book/src/structural-rules.md`, `book/src/knobs.md`, `USER_GUIDE.md`, `CODEBASE_ANALYSIS.md`, `MEMORY.md`, `CHANGES.md`.

---

## 2026-04-15-0024 — Sub coefficient constraint: `ck > 0` for all k

**Commit hash:** `b0f84fd`

**What changed**
- `book/src/structural-rules.md` "Roles of constants in RTL" → Coefficient subsection: expanded with per-op shapes and constraints.
  - Add: `y = s1*c1 + ... + sn*cn`, `ci ≠ 0` for all i (non-zero; positive or negative both legal).
  - Sub: `y = s1*c1 - s2*c2 - ... - sn*cn` (left-associative), **`ci > 0` for all i** (strictly positive). Rationale in-line: a negative `ci` on a `- sk*ck` term flips to `+ sk*|ck|` — an Add contribution disguised as a Sub term. Zero kills the term. Strictly positive preserves subtractive character.
  - Mul: shape + constraints TBD (pending user spec).
- `MEMORY.md` next-up item 1 rewritten to carry the per-op constraints, not just the Add shape.
- `DEVELOPMENT_NOTES.md` "Roles of constants in RTL" core-decision entry extended with the per-op constraint summary.

**Why**
User: "This 'Linear-combination ADD motif' shall also be true for SUB too. ck > 0 for all k." The distinction between Add's `ci ≠ 0` (non-zero) and Sub's `ci > 0` (strictly positive) is semantic, not arbitrary — negative coefficients inside a subtractive chain mean the term is an Add contribution rather than a Sub one, which defeats the purpose of generating a Sub-shaped motif.

Logging the clarification now so the next-up motif slice implements the correct per-op constraints without rediscovering them.

**Validation**
- Documentation-only slice; no source touched.
- `cargo check`, `cargo test` (35 tests), `cargo clippy --all-targets -- -D warnings`, `cargo fmt --all --check`: all still clean.

**Impact**
- Coefficient motif implementation now has precise per-op specs for Add and Sub ahead of implementation. Mul remains under-specified until you weigh in.
- Structural-rules catalog's coefficient section is now the durable reference for the per-op constraint set.

**Files touched**
`book/src/structural-rules.md`, `MEMORY.md`, `DEVELOPMENT_NOTES.md`, `CHANGES.md`.

---

## 2026-04-15-0023 — `graph-first` strategy landed, becomes the new default

**Commit hash:** `4085401`

**What changed**
- `src/config.rs`:
  - New `ConstructionStrategy::GraphFirst` variant.
  - New `graph_first_pool_size: u32` field on `Config` (default 32). Target number of top-level units (operator gate / flop / comb-mux block) to emit during pool growth. Does not count internal primitives generated by comb-mux or flop-mux assembly.
  - **Default flipped:** `Config::default().construction_strategy = GraphFirst`.
  - Overrides and `apply_cli_overrides` wired.
- `src/main.rs`: new CLI flags `--construction-strategy graph-first` (variant visible) and `--graph-first-pool-size`.
- `src/gen/cone.rs`: new `build_graph_first` entry point plus three helpers:
  - `grow_pool_one_unit`: emits one top-level unit. Picks flop (with deferred D) / comb-mux block / operator gate according to the usual probabilities. Operator-gate operands come from `pick_terminal` (no recursion). Anti-collapse retry up to 4× then skip. Returns a boolean indicating success; caller loops with an iteration budget of `8 × pool_size` to prevent pathological infinite loops.
  - `build_comb_mux_pool_only`: mirrors `build_comb_mux` but data/sel operands are pool picks. Reuses `replicate_to_width`, `make_and`, `or_reduce_terms`, `make_eq_const`, `make_mux`, `make_constant`.
  - `drain_flop_worklist_pool_only`: mirrors `drain_flop_worklist` but every data / select / direct-D sub-cone is a pool pick. Reuses `assemble_flop_d_one_hot` and `assemble_flop_d_encoded`.
- `src/gen/module.rs`: strategy dispatch updated — `GraphFirst` arm delegates to `cone::build_graph_first`. Subsequent `cone::drain_flop_worklist` is a no-op for GraphFirst (worklist already drained via the pool-only variant).
- `tests/pipeline.rs`:
  - `all_strategies_produce_valid_modules` extended to cover `GraphFirst`.
  - New `graph_first_is_default` — omitting `--construction-strategy` produces byte-identical output to `--construction-strategy graph-first`.
  - New `graph_first_reproducibility` — same seed + GraphFirst = byte-identical output twice.
  - New `graph_first_differs_from_sequential` — on a 3-output seed, GraphFirst produces different SV than Sequential.
- `book/src/construction-strategies.md`: implementation status table updated — all four strategies implemented; `graph-first` marked **default**. Top-of-chapter text updated to reflect that `graph-first` is current default. Implementation-sequence prose updated.
- `book/src/knobs.md`: construction-strategy subsection rewritten — all four values listed, `graph-first` marked default, `graph_first_pool_size` knob documented.
- `USER_GUIDE.md`: CLI table updated — `--construction-strategy` default flipped to `graph-first`, new `--graph-first-pool-size` row.
- `CODEBASE_ANALYSIS.md`: `config.rs` and `cone.rs` entries extended; `module.rs` dispatch documented to include GraphFirst.
- `MEMORY.md`: last-completed-slice refreshed; next-up reorganized — all construction-strategy work items removed (done); remaining items are motif slices (coefficients, shift-amount bias, comparands) + Verilator-lint smoke. Recent-commits list gains `6d2da98`.

**Why**
Per user direction: `graph-first` is the correct default because the user-visible output is a DAG, not a union of per-output cones. The cone-per-output construction idiom is a human-friendly fiction; `graph-first` drops it in favor of growing a gate pool with no output attribution and picking drive-roots from the pool. Sharing is truly symmetric including through block internals (flop D-cones, comb-mux sub-cones) — a property none of the prior strategies achieves.

Landing `graph-first` completes the four-strategy commitment. Users who want the old `sequential` behavior pin it explicitly via `--construction-strategy sequential`.

**Validation**
- `cargo check --all-targets`, `cargo test` (25 unit + 10 integration = 35 tests), `cargo clippy --all-targets -- -D warnings`, `cargo fmt --all --check`: all clean.
- End-to-end: `cargo run -- --seed 42 --min-outputs 3 --max-outputs 3` (default knobs) produces identical output to `cargo run -- ... --construction-strategy graph-first`, confirming the default flip. Diffing against `--construction-strategy sequential` on the same seed shows different module shape (different flop widths, different gate structure, different pool entries) — the strategy knob is load-bearing.

**Impact**
- Four construction strategies implemented; `graph-first` is now the default behavior of `anvil`.
- True module-wide symmetric sharing for the default strategy: every data / select / direct-D sub-cone — whether in an output cone, a flop D, or a comb-mux — is picked from the same fully-grown pool.
- Reproducibility preserved for prior-generated output by pinning the strategy at invocation time and recording effective knobs in the manifest.
- The construction-strategy work item from the last seven slices' next-up queues is complete.

**Files touched**
`src/config.rs`, `src/main.rs`, `src/gen/cone.rs`, `src/gen/module.rs`, `tests/pipeline.rs`, `book/src/construction-strategies.md`, `book/src/knobs.md`, `USER_GUIDE.md`, `CODEBASE_ANALYSIS.md`, `MEMORY.md`, `CHANGES.md`.

---

## 2026-04-15-0022 — `interleaved` construction strategy: frame state machine

**Commit hash:** `6d2da98`

**What changed**
- `src/config.rs`: new `ConstructionStrategy::Interleaved` variant.
- `src/gen/cone.rs`: new frame state machine at module level.
  - Types: `Dest` (Output(idx) | Slot { frame_id, slot }), `SignalFrame` (pending signal construction at given width/depth/exclude with a Dest), `GateFrame` (in-flight gate waiting on N operand slots).
  - Public entry: `build_outputs_interleaved(g, m, pool, worklist) -> Vec<NodeId>`. Seeds the queue with one `SignalFrame` per output, pops a random frame each step, processes it.
  - `process_signal_frame`: mirrors `build_cone`'s decision tree. force_leaf → `pick_terminal` → deliver. Flop → `build_flop_leaf` (synchronous block) → deliver. Comb-mux → `build_comb_mux` (synchronous block) → deliver. Operator gate → allocate a `GateFrame` in the in-flight table + enqueue N child `SignalFrame`s (with per-operand share check reusing `try_share`).
  - `deliver`: writes the resolved NodeId to the Dest. For `Slot`, decrements pending; when pending hits 0, the `GateFrame` fires (same anti-collapse check, same dep-set union, same pool.add), and its own Dest is then resolved (recursively).
- `src/gen/module.rs`: `generate_leaf_module` dispatches on strategy. For `Interleaved`, delegates to `cone::build_outputs_interleaved`. For `Sequential` / `Shuffled`, uses the existing recursive `build_cone_with_retry` path.
- `tests/pipeline.rs`:
  - `all_strategies_produce_valid_modules` extended to cover `Interleaved`.
  - New `interleaved_reproducibility` — same seed + Interleaved = byte-identical output twice.
  - New `interleaved_differs_from_sequential` — on a 3-output seed, the emitted SV differs between strategies.
- `book/src/construction-strategies.md` and `book/src/knobs.md`: implementation status flipped to "implemented" for `interleaved`; scope note clarifying that block internals are not interleaved in this slice (only output-cone frames).
- `USER_GUIDE.md`: `--construction-strategy` row updated to list `interleaved` as supported.
- `CODEBASE_ANALYSIS.md`: `cone.rs` module map expanded to document the frame machine; `config.rs` lists the three variants; `module.rs` describes the dispatch.
- `MEMORY.md` / `CHANGES.md`: per workflow.

**Why**
Per the user's direction that construction-order asymmetry is a construction artifact and not a design property, `interleaved` was the next milestone after `shuffled`. The frame state machine achieves near-symmetric per-module sharing for output-cone construction: by the time any given cone picks its deeper leaves, many other cones have already contributed gates to the pool. Declaration-order bias is gone; within-module ordering is still present for *block internals* (flop D-cones, comb-mux sub-cones built depth-first within one frame step) but much weaker than in `sequential` or `shuffled`.

Scope was deliberately kept to output-cone frames — block internals remain synchronous — because folding blocks into the frame machine adds meaningful complexity without buying proportional symmetry (flop Qs enter the pool when flops are allocated, so cross-flop sharing works regardless). Full symmetry awaits `graph-first`.

**Validation**
- `cargo check --all-targets`, `cargo test` (25 unit + 7 integration = 32 tests), `cargo clippy --all-targets -- -D warnings`, `cargo fmt --all --check`: all clean.
- End-to-end: `cargo run -- --seed 42 --min-outputs 3 --max-outputs 3 --construction-strategy interleaved` produces a valid ~4k-line module. Diffing against `sequential` on the same seed shows different internal shape; IR validator accepts both.

**Impact**
- Three of four construction strategies now implemented.
- Users can pick `interleaved` for realistic cross-output sharing without waiting for `graph-first`.
- `build_outputs_interleaved` is a self-contained alternative entry point; the recursive `build_cone` path is untouched.

**Known limitations (documented)**
- Block internals (flop D-cones, comb-mux sub-cones) still build depth-first. Full symmetry including blocks is the `graph-first` target.
- The `interleaved` path does not have a retry-on-trivial mechanism equivalent to `build_cone_with_retry`. If an output cone ends up with an empty dep-set it will fail validation. In practice this has not been observed under default knobs; the validator catches it if it happens.

**Files touched**
`src/config.rs`, `src/gen/cone.rs`, `src/gen/module.rs`, `tests/pipeline.rs`, `book/src/construction-strategies.md`, `book/src/knobs.md`, `USER_GUIDE.md`, `CODEBASE_ANALYSIS.md`, `MEMORY.md`, `CHANGES.md`.

---

## 2026-04-15-0021 — Construction-strategy machinery + `shuffled` strategy landed

**Commit hash:** `2d038a9`

**What changed**
- `src/config.rs`: new `ConstructionStrategy` enum with `Sequential` and `Shuffled` variants. Derives `Serialize`, `Deserialize`, `clap::ValueEnum`; uses kebab-case for both serde and clap. New `Config.construction_strategy` field (default `Sequential`). Threaded through `Overrides` and `apply_cli_overrides`.
- `src/main.rs`: new CLI flag `--construction-strategy <sequential|shuffled>` via `value_enum`. Imports the enum through the public `anvil::config::ConstructionStrategy` path.
- `src/gen/module.rs`: `generate_leaf_module` dispatches on `cfg.construction_strategy`.
  - `Sequential`: outputs built in declaration order (`0, 1, ..., n_out-1`). Current behavior preserved exactly.
  - `Shuffled`: a random permutation of the output indices is drawn from the seeded RNG and used as the build order.
  - Either way, drives are recorded in declaration order in `m.drives`, so SV port/assign emission is unaffected by build order. Only *which pool state each output's leaf-selection sees* changes.
- `tests/pipeline.rs`: three new integration tests.
  - `shuffled_reproducibility` — same seed + `Shuffled` strategy produces byte-identical output twice.
  - `shuffled_differs_from_sequential` — on a 4-output seed, `Shuffled` produces different SV than `Sequential`, confirming the knob actually changes output.
  - `all_strategies_produce_valid_modules` — both strategies × 10 seeds = 20 modules all pass `ir::validate`.
- `book/src/construction-strategies.md`: "Implementation status" section updated — `sequential` and `shuffled` now marked implemented; `interleaved` and `graph-first` still planned.
- `book/src/knobs.md`: new "Construction strategy" subsection documenting the knob and its values.
- `USER_GUIDE.md`: `--construction-strategy` added to the CLI flags table.
- `CODEBASE_ANALYSIS.md`: module-map entries for `config.rs` and `gen/module.rs` updated to reflect the new enum and the strategy-dispatching build-order logic.
- `MEMORY.md`: next-up list retires the "add the machinery" and "land shuffled" items; items 1 and 2 are now `interleaved` and `graph-first` respectively. Current-state snapshot refreshed. Recent commits list gains `8eb03f0`.

**Why**
User said the asymmetry of sequential declaration-order construction is a construction artifact, not a design property, and asked for all four strategies supported with `graph-first` as the eventual default. This slice lands the knob infrastructure plus the cheapest of the four alternative strategies (`Shuffled`), giving an immediate user-visible improvement (declaration-order bias removed) without the architectural rewrite that `Interleaved` / `GraphFirst` require.

Landing `Sequential` + `Shuffled` together in one slice is one coherent task — the knob has at least one non-trivial value from day one, rather than being a placeholder with only a single option. Future slices add `Interleaved` and then `GraphFirst` + default-flip.

**Validation**
- `cargo check --all-targets`, `cargo test` (25 unit + 5 integration = 30 tests), `cargo clippy --all-targets -- -D warnings`, `cargo fmt --all --check`: all clean.
- `cargo run -- --help` shows `--construction-strategy <CONSTRUCTION_STRATEGY>` with the clap-generated description.
- End-to-end: `cargo run -- --seed 42 --min-outputs 4 --max-outputs 4 --construction-strategy shuffled` produces a valid module; diffing against the `sequential` run on the same seed shows different internal shape (different gates, different sharing pattern) while the port list remains in declaration order.

**Impact**
- Users can now pick between `sequential` and `shuffled` at the CLI. The declaration-order bias is no longer mandatory.
- The knob scaffolding is in place for the two remaining strategies; adding them is a matter of extending the `ConstructionStrategy` enum and adding a match arm in `generate_leaf_module` (plus for `graph-first`, a new `build_module_graph_first` path).

**Files touched**
`src/config.rs`, `src/main.rs`, `src/gen/module.rs`, `tests/pipeline.rs`, `book/src/construction-strategies.md`, `book/src/knobs.md`, `USER_GUIDE.md`, `CODEBASE_ANALYSIS.md`, `MEMORY.md`, `CHANGES.md`.

---

## 2026-04-15-0020 — Construction-strategies chapter: 4 named strategies, graph-first planned default

**Commit hash:** `8eb03f0`

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
