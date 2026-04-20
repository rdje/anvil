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
combinational nodes that mostly happens at intern time. For state,
there is one conservative post-drain pass: once flop D-cones are
known, exact duplicate state signatures are merged.

Why mostly at intern time and not as a post-pass? Three reasons:

1. **Rule-based generation** doctrine — we never materialise a
   gate and then filter it out, because the construction-time
   rule IS the statement. A post-hoc filter can be bypassed; a
   construction-time rule defines the IR.
2. **Reproducibility** — intern-time rewrites are deterministic
   in the seed: the same input RNG path produces the same
   factorized output. Post-hoc passes that work over `m.nodes`
   would need their own determinism story.
3. **State is a real exception** — a flop's identity is not fully
   knowable when `build_flop_leaf` allocates its Q. Its semantics
   only become concrete once the worklist later builds `d`, so the
   current stateful sharing rule necessarily runs after drain.

## The ladder

`IdentityMode` + `FactorizationLevel` in `src/config.rs`:

```
identity_mode: node-id | relaxed
```

`node-id` (default) means NodeId is expression identity and keeps
the ladder live. `relaxed` is the coarse off-switch: the ladder is
forced to `none` regardless of the requested rung.

Within `identity_mode = node-id`, the ladder is:

```
none → cse → operand-unique → commutative → associative →
constant-fold → peephole → e-graph (default)
```

Each level implies all lower ones. Default is `e-graph` — the
theoretical ceiling. `effective()` walks down from the requested
level and returns the highest layer that is actually implemented
in the current build. Today that's **peephole**; `e-graph`
activates everything currently live plus every future layer for
free.

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

### 7. Post-drain exact-state flop merge (`identity_mode = node-id`, effective `>= Cse`)

`intern_gate` only sees combinational nodes. Flops are born before
their D-cones exist, so their identity cannot be decided at birth.
After `drain_flop_worklist` finishes, `generate_leaf_module` runs
[`crate::ir::compact::merge_equivalent_flops`].

The current signature is intentionally conservative:

- same `width`
- same `reset_kind`
- same `reset_val`
- same exact `d: NodeId`; or
- D-cones that become identical after renaming each flop's own `q` to a
  synthetic "self" leaf.

If those match, every consumer of the duplicate Q is rewired to the
canonical Q, virtual flop deps are remapped, surviving flops are
renumbered densely, and the later compaction pass drops the now-dead
duplicate Q nodes.

What it deliberately does **not** do yet:

- prove graph isomorphism across D-cones;
- structurally merge non-self D-cones that merely happen to be
  duplicate ASTs under looser duplication settings;
- merge wider sequentially-equivalent machines.

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
- exact duplicate flops, plus self-feedback-isomorphic duplicates,
  merge once their D-cones exist; but
- stronger sequential equivalence, block/module identity, and future
  parameter-aware hierarchical identity are still open work.

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

1. BFS from all roots (output drives, flop fields).
2. Marks reachable nodes.
3. Rewrites `m.nodes` in topological order, keeping only
   reachable nodes.
4. Remaps every `NodeId` in `m.drives`, `m.flops`, `m.nodes[*]
   .operands`, and the dedup tables.

Result: Rule 18 is re-established at module finalisation. The
count of removed nodes is exposed as `Metrics::nodes_compacted`.

A subtle consequence: without this pass, each of layers 1, 3, 4
would have to be *orphan-suppressed* — either not firing when
orphaning would occur, or recording the orphan as a permitted
exception. With the pass, they can fire freely.

## Empirical counters

Each layer exposes a counter on `Module`, surfaced via `Metrics`:

| Layer | Counter | Metric field |
|-------|---------|--------------|
| Associative | `flatten_associative_applied: u64` | `Metrics::flatten_associative_applied` |
| ConstantFold | `fold_identities_applied: u64` | `Metrics::fold_identities_applied` |
| Peephole | `peephole_rewrites_applied: u64` | `Metrics::peephole_rewrites_applied` |
| Flop merge | `flops_merged: u32` | `Metrics::flops_merged` |
| Compaction | `nodes_compacted: u32` | `Metrics::nodes_compacted` |

Plus a structural post-construction metric:
`nested_associative_operand_count` — the number of operand slots
on associative gates whose operand is itself a same-op same-width
gate. At default knobs with Associative live, this is **0** —
direct empirical validation that the layer is exhaustive. See
the `nested_associative_opportunities_flatten_to_zero` regression
test.

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
anvil --seed 42 --factorization-level none

# CSE only, no operand uniqueness. Shows why CSE alone isn't enough.
anvil --seed 42 --factorization-level cse

# Walk up one rung at a time.
for lvl in none cse operand-unique commutative associative \
           constant-fold peephole e-graph; do
    echo "=== $lvl ==="
    anvil --seed 42 --factorization-level "$lvl" --metrics 2>&1 \
        | grep -E 'num_gates|flatten_associative|fold_identities|peephole|nodes_compacted' \
        | head -6
done
```

The gate count monotonically decreases (or stays equal) as the
level climbs — more factorization always implies fewer named
nodes.

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
