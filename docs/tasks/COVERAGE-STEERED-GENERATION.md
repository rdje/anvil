# COVERAGE-STEERED-GENERATION: construction-time coverage-feedback steering

## Metadata

- Tree ID: `COVERAGE-STEERED-GENERATION`
- Status: `active`
- Roadmap lane: `Usability / effectiveness — coverage-steered generation (north star, idea 6)`
- Created: `2026-06-17`
- Last updated: `2026-06-17`
- Owner: repo-local workflow

## Goal

Find bugs faster by **biasing generation toward under-exercised constructs** —
but strictly at **construction time**, by adjusting the seeded construction-time
choices (the `roll_knob` decision sites), **never** by generate-then-filter. A
coverage target (which constructs/categories/surfaces to emphasize) and the
achieved coverage are both first-class, API-settable and API-queryable. This
turns ANVIL from uniform-random toward goal-directed exploration of the legal
design space while preserving every lane invariant.

## Non-Goals

- **No generate-then-filter / no post-hoc rejection** (`feedback_rules_first_generation`
  — the load-bearing doctrine). Steering biases the *construction-time* choice
  distribution; it never builds-then-discards.
- No behavioural oracle; coverage is over *structural* constructs (gate kinds,
  motifs, emission surfaces, hierarchy/identity features), not behaviour.
- No break to reproducibility: a given `(seed, knobs, steering-config)` stays
  byte-identical; default (no steering) is byte-identical to today.

## Acceptance Criteria

- A steering mechanism biases construction-time rolls toward a named coverage
  target and measurably shifts the achieved construct distribution vs. unsteered,
  on a seed sweep — while staying rules-first (no filtering) and reproducible.
- **API-completeness gate (decision `0017`):** the coverage **target** is
  settable via the MCP/config API and the **achieved coverage** is queryable via
  the MCP/introspection API (SCHEMA-DERIVED — projected from the existing
  metrics/knob-roll telemetry + the construct histograms). The CLI is a shim over
  the same surface.
- Default-off / DUT byte-identical (unsteered output unchanged); the byte-stable
  contract holds per `(seed, knobs, steering-config)`; downstream-clean.
- Documented in `book/src/algorithm.md` (or a steering subsection) +
  `book/src/agent-mcp.md` + USER_GUIDE; committed through `COMMIT.md`.

## Task Tree

- ID: `COVERAGE-STEERED-GENERATION`
  Status: `active`
  Goal: `Construction-time coverage-feedback steering (rules-first, reproducible) with an API-settable coverage target + an API-queryable achieved-coverage readout.`
  Children: `COVERAGE-STEERED-GENERATION.1`

- ID: `COVERAGE-STEERED-GENERATION.1`
  Status: `pending`
  Goal: `Design/decision leaf (ADR, no code): pin HOW coverage feedback biases construction WITHOUT generate-then-filter (e.g. per-category/per-surface weight multipliers applied to the existing roll_knob decision sites; or a deterministic schedule across a --count run that nudges weights toward under-hit constructs) while keeping byte-stability per (seed, knobs, steering-config); define the coverage-target model + the achieved-coverage readout (reuse knob_roll_attempts/fires + gate/category/surface histograms in Metrics); pin the MCP target-set + coverage-query surface (decision 0017); and EXPLICITLY reconcile with feedback_rules_first_generation (steering is a construction-time prior, not a post-hoc filter). Record as the next decision record + pre-split .2 (impl).`
  Acceptance: `A decision record + a tree/DEVELOPMENT_NOTES entry pinning the rules-first steering model, the reproducibility contract, the coverage target/readout, and the MCP surface; docs-only; INDEX + this tree + docs/TASK_TREE.md updated.`
  Verification: `pending`
  Commit: `pending`

- ID: `COVERAGE-STEERED-GENERATION.2`
  Status: `pending`
  Goal: `Implement the .1 design: the construction-time steering weights + the coverage-target config + the API-queryable achieved-coverage readout + proofs (rules-first: no filter path; byte-stability per steering-config; distribution shift vs unsteered) + book/USER_GUIDE/KM. Default-off / DUT byte-identical. Pre-split when picked.`
  Acceptance: `pending (set at .1)`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `COVERAGE-STEERED-GENERATION.1` | `pending` | Design-first ADR is essential here — the rules-first boundary (`feedback_rules_first_generation`) and the reproducibility contract must be pinned in writing before any code. |

## Decisions

- `2026-06-17`: Registered as an owner-directed usability/effectiveness lane
  (idea 6). Binds decision [`0017`](../decisions/0017-api-first-everything-mcp-accessible.md)
  (API-settable target + API-queryable coverage) and is explicitly bounded by
  `feedback_rules_first_generation` (construction-time prior, never
  generate-then-filter). Design-first ADR before code.

## Open Questions

- The steering primitive: per-roll weight multipliers vs. a deterministic
  per-`--count` schedule vs. a seeded distribution prior — which best biases
  construction while staying byte-stable per `(seed, knobs, steering-config)`.
  *(This is the crux `.1` decides.)*
- Whether steering targets categories, emission surfaces, or both, and how the
  target is expressed in the API. *(Resolved at `.1`.)*

## Blockers

- None. (Reuses the existing `knob_roll_attempts`/`fires` + histogram telemetry;
  the rules-first boundary is a design constraint, not a blocker.)

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-06-17` | `COVERAGE-STEERED-GENERATION` | `tree registered (docs-only); no code` | `registered` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `COVERAGE-STEERED-GENERATION` | `USABILITY-LANE-OWNERSHIP.1 — register 7 owner-directed usability/capability lanes + API-first decision 0017` | Tree registered (not yet started); frontier `.1` (design ADR) pending. |

## Changelog

- `2026-06-17`: Created task tree (registration via `USABILITY-LANE-OWNERSHIP.1`).
