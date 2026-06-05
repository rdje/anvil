# The Factorization Pipeline

Every gate that enters an anvil module passes through a single
chokepoint: `Module::intern_gate` in `src/ir/types.rs`. This
chapter walks through that pipeline layer by layer. It's aimed
at the reader who wants to know what *exactly* happens between
"build_cone picks an Add" and "a `Node::Gate` shows up in
`m.nodes`".

For the formal rule catalogue (which layer owns which rule), see
[Rule 21c in Structural Rules](structural-rules.md#21c--identity-mode--factorization-level-user-controllable-dial).
This chapter is the narrative complement.

## Why factorize?

The doctrinal anchor is the user's "full factorization" rule:
**`NodeId` is the identity of an expression.** Two expressions
that are the same in the mathematical or logical sense should
share one `NodeId`; different expressions should have different
`NodeId`s.

The perfect version of this is the e-graph problem — proving
semantic equivalence of arbitrary RTL trees — which nobody has
solved completely. So anvil climbs a ladder of approximations.
Each rung catches a specific class of "same expression, different
syntax" cases and collapses them to a shared `NodeId`. For
combinational nodes that mostly happens at intern time. There is now
one bounded post-construction combinational pass at the live `e-graph`
rung, and one conservative post-drain state pass: once the full cones
exist, endpoint-preserving proofs over the current normalized IR can be
merged.

Why mostly at intern time and not as a post-pass? Three reasons:

1. **Rule-based generation** doctrine — we never materialise a
   gate and then filter it out, because the construction-time
   rule IS the statement. A post-hoc filter can be bypassed; a
   construction-time rule defines the IR.
2. **Reproducibility** — intern-time rewrites are deterministic
   in the seed: the same input RNG path produces the same
   factorized output. Post-hoc passes that work over `m.nodes`
   would need their own determinism story.
3. **Some proofs need the finished cone** — a flop's identity is not
   fully knowable when `build_flop_leaf` allocates its Q, and bounded
   semantic proof of a gate cone only makes sense once the whole cone
   exists. So the strongest live sharing rules necessarily run after
   construction for those cases.

## The ladder

`IdentityMode` + `FactorizationLevel` in `src/config.rs`:

```text
identity_mode: node-id | relaxed
```

`node-id` (default) means `NodeId` is expression identity, which
implies full factorization by definition. `relaxed` is the coarse
off-switch: it is the only intentional mode where equivalent
expressions may keep different `NodeId`s, and the ladder is forced to
`none` regardless of the requested rung.

Within `identity_mode = node-id`, the ladder is:

```text
none → cse → operand-unique → commutative → associative →
constant-fold → peephole → e-graph (default)
```

Each level implies all lower ones. Default is `e-graph` — the
theoretical ceiling. `effective()` walks down from the requested
level and returns the highest layer that is actually implemented
in the current build. Today that's the bounded live `e-graph`
fragment: `e-graph` activates everything currently live plus every
future layer for free.

Interpretation note: this ladder does **not** redefine what
`identity_mode = node-id` means. It is the current build's
implementation/proof-depth dial inside the full-factorization
doctrine. In other words, `none`, `cse`, `operand-unique`, and the
higher rungs are staged approximations and stress/debug settings, not
alternate meanings of `NodeId` identity.

Selection via `--identity-mode`, `--factorization-level`, the
convenience aliases `--full-factorization` /
`--no-full-factorization`, or `Config::{identity_mode,
factorization_level}` in a config file.

## Pipeline, in execution order

When `intern_gate(op, operands, width, deps)` is called, the
following runs in sequence. Each step is gated on
`self.effective_factorization_level() >= <LayerThreshold>`.

### 1. Associative flattening (`>= Associative`)

[`Module::flatten_associative`] splices operand trees:
`Add(a, Add(b, c))` becomes `Add(a, b, c)` with the inner `Add`
left unreferenced.

Per-op semantic normalisation runs after the splice, because
splicing can create operand-list shapes that pre-splicing rules
would never have allowed:

- `And` / `Or`: deduplicate (idempotent — `a & a = a`, `a | a = a`).
- `Xor`: pair-cancel (self-inverse — `a ^ a = 0`). Count each
  operand's occurrences, drop even-count operands entirely, keep
  one copy of each odd-count operand.
- `Add` / `Mul`: **skip** the flatten entirely when it would
  produce duplicates under strict `operand_duplication_rate`.
  Dropping duplicates would silently change semantics
  (`x + x = 2x`, `x * x = x²`), so we preserve the nested shape
  instead.

Short-circuits:
- 0 operands remain (only reachable for `Xor`-all-cancel) →
  return the zero constant.
- 1 operand remains → return that operand's `NodeId` directly.
- ≥ 2 operands → rewrite the operand list in place; the caller
  proceeds through subsequent layers.

Runs BEFORE commutative sort so the flattened list is what gets
sorted.

### 2. Commutative sort (`>= Commutative`)

For `And`/`Or`/`Xor`/`Add`/`Mul`, operands are sorted
ascending by `NodeId`. This collapses
`Add(a, b)` and `Add(b, a)` into the same canonical form
(Rule 21b).

### 3. Constant folding (`>= ConstantFold`)

[`Module::fold_constants`] applies algebraic identities and
fully-evaluates any all-constant expression at intern time:

**Associative ops (`And`/`Or`/`Xor`/`Add`/`Mul`):**

| Op    | All-const evaluation                | Identity drop   | Absorbing                              |
|-------|-------------------------------------|-----------------|----------------------------------------|
| `And` | bitwise AND over values             | drop `all_ones` | `0`                                    |
| `Or`  | bitwise OR over values              | drop `0`        | `all_ones`                             |
| `Xor` | bitwise XOR over values             | drop `0`        | —                                      |
| `Add` | sum mod 2^width                     | drop `0`        | —                                      |
| `Mul` | product mod 2^width                 | drop `1`        | `0`                                    |

All-const evaluation supersedes the absorbing and identity-drop
paths for the all-const subcase — e.g. `Add(3, 5)` folds to 8
directly without going through identity-drop. Mixed operand
lists (one constant + one primary input, say) reach the
identity-drop / absorbing paths.

**Non-commutative 2-arity ops:**

| Op    | All-const evaluation                             | Rhs-zero identity |
|-------|--------------------------------------------------|-------------------|
| `Sub` | `(lhs - rhs) mod 2^width`                        | `a - 0 → a`       |
| `Shl` | `(lhs << rhs) mod 2^width` (over-shift → 0)      | `a << 0 → a`      |
| `Shr` | `lhs >> rhs` (over-shift → 0)                    | `a >> 0 → a`      |

**Absorbing and orphan safety:** turning the whole expression into a
constant can orphan `Node::Gate` operands, but module finalisation now
runs `compact_node_ids`, so those dead gates are removed before
emission. That makes mixed dynamic absorbing (`x & 0`, `x | all_ones`,
`x * 0`) safe again.

Non-commutative ops fold only the rhs-constant case for the
identity shortcut. `a - 0` is `a`, but `0 - a` isn't — we don't
silently rewrite it. All-const evaluation doesn't have this
restriction because both operands are known constants.

### 4. Peephole rewrites (`>= Peephole`)

[`Module::apply_peephole`] applies local identities keyed on the
outer operator. The current catalogue:

**For `Not` (1 operand):**
- `Not(c) → ~c & mask(width)` — constant evaluation.
- `Not(Not(x)) → x` — involutive collapse. Inner `Not` may be
  orphaned.
- `Not(Eq(a, b)) → Neq(a, b)` and symmetric flips for
  `Neq`/`Lt`/`Gt`/`Le`/`Ge` — cross-gate comparison inversion.
  The inner comparison gate may be orphaned; the inverted
  comparison is interned through the full pipeline so it picks
  up CSE, constant folding, etc.

**For comparison ops `Eq`/`Neq`/`Lt`/`Gt`/`Le`/`Ge`:**
- All-constant evaluation: if both operands are same-width
  constants, return a 1-bit constant with the evaluated
  boolean.
- Unsigned boundary tautologies:
  - `x < 0 → 0`, `x >= 0 → 1`
  - `x <= all_ones → 1`, `x > all_ones → 0`
  - `0 > x → 0`, `0 <= x → 1`
  - `all_ones < x → 0`, `all_ones >= x → 1`

**For `Mux(sel, a, b)` (3 operands):**
- Constant selector: `Mux(0, a, b) → b`, `Mux(1, a, b) → a`.

**For `Slice { hi, lo }` (1 operand):**
- Full-width slice (`lo == 0`, `hi + 1 == src_width`) → src.
- Constant operand: `(c >> lo) & mask(hi - lo + 1)`.

**For `Concat` (1 or more operands):**
- Single-operand with matching width → that operand.
- All-constant bit assembly: every operand is a constant →
  pack MSB-first into one output constant (matches SV emit
  convention, `{c1, c2, c3}` places `c1` in the high bits).
  Operand widths must sum to the gate width; mismatch
  defensively skips the fold.

**For reductions `RedAnd`/`RedOr`/`RedXor` (1 operand):**
- Constant operand:
  - `RedAnd(c) → (c == all_ones(src_width)) as 1-bit`
  - `RedOr(c) → (c != 0) as 1-bit`
  - `RedXor(c) → popcount(c) & 1` as 1-bit.

Every rule is an unambiguous local identity. Broader cross-gate
rewrites like `(a + b) - b → a` or `(a & b) | (a & ~b) → a`
require symbolic reasoning over the expression tree (the e-graph
problem) and aren't implemented here.

### 5. Level-None bypass (`== None`)

A deliberate escape hatch: at `FactorizationLevel::None`, every
`intern_gate` / `intern_constant` call creates a fresh `NodeId`,
no dedup, no CSE, no fold. Useful for stress-testing downstream
CSE in consumer tools — does Yosys produce the same gate count
whether anvil hands it a factorized tree or a fully-expanded one?

### 6. AST-cap + CSE dedup (`>= Cse`)

With the final operand list, look up `(op, operands, width)` in
`m.gate_instances`. The cap is `max_ast_instances` (default 1,
strict uniqueness). On cap hit, return the most recently created
instance (`is_new = false`). Otherwise append a new `Node::Gate`
and register it.

This is the oldest layer in the ladder — the one that implements
Rule 21 directly. Everything above it exists to make sure
syntactically-different-but-semantically-equivalent expressions
both land on the same dedup key.

### 7. Post-construction bounded semantic gate merge (`identity_mode = node-id`, effective `>= EGraph`)

After every output cone and flop D-cone exists,
[`crate::ir::compact::merge_equivalent_gates`] walks the gate arena and
builds an endpoint-preserving proof for each combinational cone:

- same `width`
- same canonical leaf endpoints (`PrimaryInput`s / `FlopQ`s)
- same proof of functionality over those endpoints

The proof is structural over the normalized IR by default. For
small-support cones, ANVIL also computes a bounded semantic proof by
enumerating every endpoint assignment and keying the cone by its truth
table. If two gates match, later users are rewired to the canonical
gate and compaction removes the dead duplicate subtree.

This is the first live `e-graph` fragment. It is intentionally bounded:
full semantic equivalence across arbitrary-width cones is still future
work.

The settled-graph exact-value cleanup that feeds these remaps is also
allowed to reason through narrow `Slice` results even when the source
cone is wider than the small finite-set engine's direct domain. A
wide-source / narrow-slice cone is still a narrow proof problem.

Later remap-producing passes can themselves create fresh legal
associative opportunities by changing which already-built node an
operand points at. For example, semantic gate merge or a constant-
selector mux rewrite can turn an outer `Add` operand into another
same-width `Add` even though the intern-time Associative layer had
already normalized the original shape. ANVIL therefore re-runs a
settled-graph associative normalization pass
(`flatten_posthoc_associative_gates`) after remap-producing cleanup
passes. It uses the same duplicate policy as the intern-time layer:
`And`/`Or` dedup, `Xor` pair-cancels, `Add`/`Mul` flatten only when the
flat list is still legal at the current `operand_duplication_rate`.
In addition, candidate remaps are pruned if they would directly create
duplicate operands inside a strict `Add` or `Mul`.

### 8. Post-drain endpoint-aware flop merge (`identity_mode = node-id`, effective `>= Cse`)

`intern_gate` only sees combinational nodes. Flops are born before
their D-cones exist, so their identity cannot be decided at birth.
After `drain_flop_worklist` finishes, `generate_leaf_module` runs
[`crate::ir::compact::merge_equivalent_flops`].

The current proof is intentionally conservative:

- same `width`
- same `reset_kind`
- same `reset_val`
- same canonical leaf endpoints (`PrimaryInput`s / `FlopQ`s)
- same D-cone proof form after the current normalization ladder
  (commutative canonicalization, associative flattening, constant fold,
  peephole, etc.) has done what it can
- for small-support cones, an extra bounded semantic proof:
  enumerate every assignment over the canonical endpoint bits and key
  the cone by its resulting truth table

If those match, every consumer of the duplicate Q is rewired to the
canonical Q, virtual flop deps are remapped, surviving flops are
renumbered densely, and the later compaction pass drops the now-dead
duplicate Q nodes.

What it deliberately does **not** do yet:

- prove arbitrary semantic equivalence across larger or unreduced
  D-cone forms;
- merge cones that depend on different canonical leaf endpoints;
- merge wider sequentially-equivalent machines.

### 9. Post-construction deterministic FSM merge (`identity_mode = node-id`, effective `>= Cse`)

Generated FSM blocks are also state, but unlike memories they are
reset-defined and fully described by ANVIL-owned tables:

- reset state is always state 0;
- `encoding` and `num_states` define the state register values;
- `transitions[state][sel]` defines the next-state function;
- `outputs[state]` defines the registered Moore output; and
- `sel` is a normal `NodeId` cone with the same endpoint-preserving
  proof machinery used by flop merging.

After flop merging, `generate_leaf_module` runs
[`crate::ir::compact::merge_equivalent_fsms`]. If two FSM blocks have
the same selector proof, selector width, encoding, state count,
transition table, output table, and output width, consumers of the
duplicate `FsmOut` leaf are rewired to the canonical block. Virtual FSM
dependencies are remapped, surviving FSM ids are renumbered densely,
and compaction removes the now-dead duplicate `FsmOut` node.

Memories deliberately stay outside this pass. The current inferrable
memory template does not reset array contents, so two memories with the
same address/data cones are not treated as one proven state object.
That boundary is regression-protected: under node-id/e-graph
factorization, two independent memories with identical source cones must
still survive as two `Memory` blocks and two `MemRead` leaves after the
state-sharing passes and compaction.

## What "full factorization" still means

The strong-form user doctrine is:

- assign one identity to one expression;
- if two expressions are equivalent, they should not end up with
  different `NodeId`s; and
- sharing across output cones should be as high as the current build
  knows how to prove.

In roadmap terms, that means the fanin cones of different outputs and
flop-D inputs should eventually share gates, blocks, modules, and flops
whenever those structures are equivalent.

Today, ANVIL is **part-way there**:

- combinational expressions are canonicalized through the intern-time
  ladder described above;
- endpoint-preserving duplicate flops merge once their D-cones exist;
- deterministic duplicate FSM blocks merge when their selector proof and
  table/encoding/output signatures match; but
- broader sequential equivalence, memory-state merging beyond the
  current instance-local boundary, and hierarchy identity beyond
  canonical structural module signatures are still open work.

This remains deliberately user-controllable:

- `--identity-mode relaxed` turns the identity contract off and forces
  the effective ladder to `none`;
- `--identity-mode node-id` keeps the identity contract live; and
- `--factorization-level` selects how strong the currently implemented
  canonicalization should be within `node-id`.

So "full factorization" is not marketing shorthand for "already solved";
it is the direction of travel for the strongest `node-id` mode. New
identity work should always strengthen the IR's proof that two
structures are the same, never blur genuinely different structures into
one `NodeId`.

## Orphan safety: the compaction pass

Layers 1, 3, and 4 can leave gates unreferenced. When
`Not(Not(x)) → x` fires, the outer `Not` short-circuits but
the inner `Not` was already materialised and is now held by
nobody. Rule 18 (zero orphan gates) would fail.

To resolve this, `src/gen/module.rs` calls
[`crate::ir::compact::merge_equivalent_flops`] and then
[`crate::ir::compact::compact_node_ids`] at the end of
`generate_leaf_module`, after all cones and flop D-cones are
built. `merge_equivalent_flops` is the conservative stateful
sharing step; `compact_node_ids` then:

1. BFS from output drive-roots.
2. When the walk reaches a live `FlopQ`, mark the owning flop
   live and pull in its `d` / mux-held nodes.
3. Marks reachable nodes and reachable flops.
4. Rewrites `m.nodes` in topological order, keeping only
   reachable nodes.
5. Rewrites `m.flops`, dropping dead state elements whose `Q`
   was never reached by the live graph.
6. Remaps every `NodeId` / `FlopId` holder plus virtual flop
   deps in surviving gate dep-sets and the dedup tables.

Result: Rule 18 is re-established at module finalisation, and
dead sequential state does not survive into emitted SV just
because it happened to be allocated earlier. The count of
removed nodes is exposed as `Metrics::nodes_compacted`.

A subtle consequence: without this pass, each of layers 1, 3, 4
would have to be *orphan-suppressed* — either not firing when
orphaning would occur, or recording the orphan as a permitted
exception. With the pass, they can fire freely.

One more post-remap wrinkle matters at strict default knobs:
late proof / sharing passes are allowed to collapse equivalent
children, but they are **not** allowed to leave a strict `Add`
or `Mul` with the same `NodeId` twice in its operand list. So
ANVIL now prunes candidate remaps that would introduce duplicate
operands into `Add` / `Mul` when
`operand_duplication_rate < 1.0`.

## Empirical counters

Each layer exposes a counter on `Module`, surfaced via `Metrics`:

| Layer | Counter | Metric field |
|-------|---------|--------------|
| Associative | `flatten_associative_applied: u64` | `Metrics::flatten_associative_applied` |
| ConstantFold | `fold_identities_applied: u64` | `Metrics::fold_identities_applied` |
| Peephole | `peephole_rewrites_applied: u64` | `Metrics::peephole_rewrites_applied` |
| Semantic gate merge | `semantic_gates_merged: u32` | `Metrics::semantic_gates_merged` |
| Flop merge | `flops_merged: u32` | `Metrics::flops_merged` |
| FSM merge | `fsms_merged: u32` | `Metrics::fsms_merged` |
| Compaction | `nodes_compacted: u32` | `Metrics::nodes_compacted` |

Plus a structural post-construction metric:
`nested_associative_operand_count` — the number of operand slots
on associative gates whose operand is itself a same-op same-width
gate. At default knobs with Associative live, this is **0** — direct
empirical validation that the combined intern-time plus post-remap
associative normalization is exhaustive for legal flattening
opportunities. See the
`nested_associative_opportunities_flatten_to_zero` regression test.

Empirical baseline (seed 42, default knobs):

| Metric | Value |
|--------|-------|
| `flatten_associative_applied` | 268 |
| `fold_identities_applied` | 91 |
| `peephole_rewrites_applied` | 31 |
| `nodes_compacted` | 96 |
| `nested_associative_operand_count` | 0 |

Dump them via `--metrics` for single-module runs or look at
`manifest.json` for multi-module runs.

## Turning layers off

Useful for isolating the effect of a single rung:

```bash
# Disable every factorization layer — stress test CSE downstream.
cargo run --release -- --seed 42 --factorization-level none

# CSE only, no operand uniqueness. Shows why CSE alone isn't enough.
cargo run --release -- --seed 42 --factorization-level cse

# Walk up one rung at a time.
for lvl in none cse operand-unique commutative associative \
           constant-fold peephole e-graph; do
    echo "=== $lvl ==="
    cargo run --release -- --seed 42 --factorization-level "$lvl" --metrics 2>&1 \
        | grep -E 'num_gates|flatten_associative|fold_identities|peephole|nodes_compacted' \
        | head -6
done
```

Higher rungs usually reduce named-node count on a broad seed sweep, but
one fixed seed is not a strict monotonic proof: enabling a lower rung can
change retry paths, compaction opportunities, and legal operand shapes.
Use the metrics as evidence for each rung's effect rather than assuming
per-seed gate-count monotonicity.

## Pointers

- Rule 21c in [Structural Rules](structural-rules.md#21c--identity-mode--factorization-level-user-controllable-dial)
  — the formal rule catalogue and per-level table.
- [Non-Triviality and Dependency Tracking](non-triviality.md)
  — how the factorization layers interact with the Rule 18
  zero-orphans invariant via compaction.
- [Sharing](sharing.md) — CSE in the wider context of
  intra-module signal sharing.
- [Knobs and Reproducibility](knobs.md) "Per-knob roll-rate
  validation" — how the probability-roll counters complement
  the factorization counters for the measurability doctrine.
- Source: `src/ir/types.rs` (`intern_gate`, `fold_constants`,
  `flatten_associative`, `apply_peephole`) and
  `src/ir/compact.rs` (the compaction pass).
