# EMIT-SURFACE-INTERACTION-GATE: nine emit projections, never proven together

## Metadata

- Tree ID: `EMIT-SURFACE-INTERACTION-GATE`
- Status: `active`
- Roadmap lane: Quality / signoff (steering gaps 1 + 3); serves `STRUCTURED-EMISSION-EXPANSION`
- Created: `2026-07-30`
- Last updated: `2026-07-30`
- Owner: repo-local workflow (agent-picked under the owner's `2026-07-30` autonomy directive)

## Goal

Prove the nine structured-emission surfaces are downstream-clean **in combination**,
and make `--profile structured-emission-max` mean what its name says.

## Observation that opened this tree (`2026-07-30`, measured)

Two facts, both re-derivable:

**1. Every gate exercises exactly one surface.**

```
$ for f in function_emit generate_loop task_emit multi_output_task cone_function \
           mux_if case_mux_if casez_mux_if; do
    awk "/fn ${f}_focus_config/,/^}/" src/bin/tool_matrix.rs | grep -cE '_emit_prob = 1\.0'
  done
1 1 1 1 1 1 1 1
```

**2. The preset covers 4 of 9.**

```
$ awk '/structured-emission-max/,/^\s*}/' src/config.rs | grep 'emit_prob'
function_emit_prob: Some(1.0)
generate_loop_emit_prob: Some(1.0)
task_emit_prob: Some(1.0)
cone_function_emit_prob: Some(1.0)
```

Missing: `multi_output_task_emit_prob`, `mux_if_emit_prob`, `case_mux_if_emit_prob`,
`casez_mux_if_emit_prob` (and `soft_union_slice_prob`, which is version-gated and
may reasonably stay out). The preset predates surfaces 6–9 and its name is now
false.

## Why this matters more than it looks

The nine passes are **mutually exclusive on a gate**. That exclusion is the entire
soundness argument for stacking nine overlapping projections: each pass calls a
`sibling_marked`-style predicate and skips any gate another pass already claimed,
and the passes run in a fixed order (`function_emit` → `generate_loop` →
`task_emit` → `multi_output_task` → `cone_function` → `mux_if` → `case_mux_if` →
`casez_mux_if`, with `soft_union` earliest).

With exactly one probability at `1.0` and the rest at `0.0`, **every sibling set is
empty**. So the exclusion predicates are, under every gate this repo owns, dead
code that always returns `false`. The one property holding the surfaces apart has
no test.

That is also precisely the shape ANVIL exists to generate: legal, unusual,
interaction-heavy RTL (ROADMAP steering gap 1). A module carrying a `function
automatic`, a `generate for`, a multi-output `task`, a cone function, a procedural
`if/else` and two masked priority chains **at once** is far more interesting to a
downstream tool than eight modules each carrying one.

**This is not a claim that any surface is wrong.** Each is individually banked
downstream-clean, and the exclusion logic is written and unit-tested at the
function level. The gap is that no *end-to-end downstream* run has ever had two of
them live simultaneously.

## Non-Goals

- **Not** a tenth surface. `STRUCTURED-EMISSION-EXPANSION` `.20+` owns new surfaces
  (nested `generate`; `interface`/`modport` remains empirically disqualified).
- **Not** changing any surface's semantics or its default. All nine stay
  default-off ⇒ DUT byte-identical.
- **Not** retiring the eight single-surface gates. They isolate a regression to one
  surface; a combined gate cannot. Both are wanted (`feedback_never_retire_strategies`).

## Acceptance Criteria

- A repo-owned `tool_matrix` gate runs all eight `*_emit_prob` surfaces at once and
  is downstream-clean (Verilator + both Yosys modes + Icarus), `coverage_gaps = []`.
- The report proves **co-occurrence**, not just cleanliness: a single emitted module
  carries ≥ 2 distinct surfaces, keyed off the existing per-surface metrics rather
  than a new identifier token (the `case_mux_if` metric-keyed precedent).
- `--profile structured-emission-max` sets every non-version-gated surface, with a
  test pinning preset ↔ knob-list agreement so it cannot drift again.
- Evidence banked as a `docs/evidence/` digest (decision `0030`) — this tree is the
  natural second customer of that mechanism.

## Task Tree

- ID: `EMIT-SURFACE-INTERACTION-GATE`
  Status: `active`
  Children: `.1` (design), `.2` (preset + drift test), `.3` (the combined gate)

- ID: `EMIT-SURFACE-INTERACTION-GATE.1`
  Status: `pending`
  Goal: design ADR — decide (a) whether the combined gate is one scenario with all
        eight on or a small sweep, (b) how co-occurrence is *proven* from metrics
        without a new token, (c) whether `soft_union_slice_prob` joins (it is
        version-gated and Yosys/Icarus reject it, so probably a separate
        Verilator-only scenario), and (d) the expected interaction risks worth
        naming up front — above all `cone_function`'s interior **absorption** vs the
        other passes' per-gate marking, which is the one pair that does not merely
        skip but *suppresses another gate's module wire*.
  Acceptance: a `docs/decisions/00NN-*.md` with Context / Decision / Consequences.

- ID: `EMIT-SURFACE-INTERACTION-GATE.2`
  Status: `pending`
  Goal: make `--profile structured-emission-max` set every non-version-gated
        surface, plus a test asserting the preset covers exactly the intended knob
        set so a tenth surface cannot silently omit itself.
  Acceptance: preset ↔ knob-list test green; `--dump-config --profile
        structured-emission-max` shows all eight; default path byte-identical.

- ID: `EMIT-SURFACE-INTERACTION-GATE.3`
  Status: `pending`
  Goal: the repo-owned combined gate + coverage fact + banked digest.
  Acceptance: `coverage_gaps = []`, all tool columns clean, co-occurrence fact lit,
        digest committed under `docs/evidence/`.

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `.1` | `pending` | **Current frontier.** Design-first, per the lane's own cadence: every structured-emission increment in this repo landed as ADR → impl → gate → docs. |
| 2 | `.2` | `pending` | Cheap, self-contained, and makes the preset honest independently of the gate. |
| 3 | `.3` | `pending` | The gate + the digest. |

## Decisions

- `2026-07-30`: Opened as its own tree rather than as `STRUCTURED-EMISSION-EXPANSION.20`,
  because `.20+` in that lane is reserved for *new surfaces* and this is a gate over
  existing ones. Cross-referenced from that tree instead.
- `2026-07-30`: Agent-picked under the owner's standing autonomy directive
  (`MEMORY.md` standing directives, `2026-07-30`) rather than surfaced as a question.

## Open Questions

- Does `cone_function` absorption interact safely with a `multi_output_task` member
  or a `mux_if` output var when both are live? `.1` must reason this through from
  the source before the gate runs, so a failure is a *prediction confirmed*, not a
  surprise.

## Blockers

- None.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-07-30` | (tree opened) | Measured: 8/8 focus configs set exactly one `_emit_prob = 1.0`; the `structured-emission-max` preset sets 4 of 9 | observation recorded; tree registered |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |

## Changelog

- `2026-07-30`: Created from a source-level observation that the nine emit
  projections' mutual-exclusion invariant has no end-to-end gate, and that
  `--profile structured-emission-max` covers 4 of 9 surfaces.
