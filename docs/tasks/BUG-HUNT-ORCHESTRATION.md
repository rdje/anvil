# BUG-HUNT-ORCHESTRATION: a turnkey, MCP-driven downstream bug-hunt loop

## Metadata

- Tree ID: `BUG-HUNT-ORCHESTRATION`
- Status: `active`
- Roadmap lane: `Usability — turnkey bug-finder (north star, idea 1)`
- Created: `2026-06-17`
- Last updated: `2026-06-17`
- Owner: repo-local workflow

## Goal

Make ANVIL **directly usable as a downstream-tool bug-finder**, not a generator
the user has to wrap. Deliver a single turnkey loop — surfaced as a CLI
(`anvil hunt --tool <verilator|yosys|iverilog|…> --seeds N`) **and** as an MCP
tool — that: (1) fuzzes a chosen downstream tool across seeds and knob profiles,
(2) catches any reject / warning / cross-tool mismatch, (3) **auto-minimizes**
the failing artifact via the existing `minimize` coordinate-descent, and (4)
drops a self-contained **reproducer bundle** (seed + effective knobs + `.sv` +
`manifest.json` + expected-facts + the tool's log + a one-command repro script).
The pieces already exist but are separate (`src/bin/tool_matrix.rs`, the hardened
`src/downstream/` `validate`/`minimize` surface, `--diff-sim`, `src/introspect`);
this lane composes them into one bug-hunt orchestrator.

## Non-Goals

- No behavioural oracle / shadow simulator (decision `0004`, ROADMAP gap 4).
- No embedding/vendoring of the downstream tools — they stay external,
  allow-listed, sandboxed, RAM-guarded invocations.
- No new generator semantics; the hunt drives the existing valid-by-construction
  lanes. Default DUT output stays byte-identical.

## Acceptance Criteria

- A turnkey hunt loop (fuzz → detect → minimize → reproducer bundle) runs
  end-to-end against at least one real downstream tool and produces a
  one-command-reproducible bundle for an injected/known failure.
- **API-completeness gate (decision `0017`):** the hunt is fully driveable over
  MCP — an `hunt` (or equivalently-named) MCP tool sets every control
  (tool, seed range, knob profile, budgets, minimize on/off), invokes the run,
  and the result (per-seed verdicts, the minimized reproducer, the bundle
  `ResourceRef`) is queryable via the MCP/introspection API. The CLI is a thin
  shim over the same API.
- Reproducible + sandboxed: seeded, no wall-clock / no `thread_rng`; controlled
  tool calls go through the `src/downstream/` allow-list + RAM guard +
  `anvil://audit/log`.
- Default-off / DUT byte-identical; downstream-clean; documented in
  `book/src/agent-mcp.md` + USER_GUIDE + README; committed through `COMMIT.md`.

## Task Tree

- ID: `BUG-HUNT-ORCHESTRATION`
  Status: `active`
  Goal: `A turnkey, MCP-driven fuzz → detect → minimize → reproducer-bundle bug-hunt loop over the existing tool_matrix / downstream / diff-sim / introspect surfaces.`
  Children: `BUG-HUNT-ORCHESTRATION.1`

- ID: `BUG-HUNT-ORCHESTRATION.1`
  Status: `pending`
  Goal: `Design/decision leaf (ADR, no code): pin the orchestration-loop shape (how it composes generate + the existing downstream validate/minimize + diff-sim + introspect), the reproducer-bundle format (seed + effective knobs + .sv + manifest + expected-facts + tool log + one-command repro), the MCP "hunt" tool input/result schema + the CLI shim over it (decision 0017 API-completeness), the detection policy (reject/warning/mismatch as a failure), and the sandbox/reproducibility discipline (decision 0004). Record as the next decision record + pre-split .2 (impl).`
  Acceptance: `A decision record (next sequential id) + a DEVELOPMENT_NOTES/tree entry pinning the loop, the bundle format, and the MCP+CLI surface; docs-only; INDEX + this tree + docs/TASK_TREE.md updated.`
  Verification: `pending`
  Commit: `pending`

- ID: `BUG-HUNT-ORCHESTRATION.2`
  Status: `pending`
  Goal: `Implement the .1 design: the hunt orchestrator + the MCP hunt tool + the CLI shim + the reproducer-bundle emitter + proofs + a real-tool end-to-end gate + book/USER_GUIDE/README/KM. Default-off / DUT byte-identical. Pre-split when picked.`
  Acceptance: `pending (set at .1)`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `BUG-HUNT-ORCHESTRATION.1` | `pending` | Highest-leverage usability lane (the owner's "single biggest usability multiplier"); design-first ADR pins the loop + the MCP/CLI surface (decision `0017`) before any code. |

## Decisions

- `2026-06-17`: Registered as an owner-directed usability lane (idea 1). Binds
  decision [`0017`](../decisions/0017-api-first-everything-mcp-accessible.md)
  (API-first: the hunt must be fully MCP-driveable + its results queryable). The
  first leaf is a design/decision ADR per the project's design-first cadence; no
  code before `.1` lands.

## Open Questions

- Bundle format: a directory vs a single self-describing archive; how the
  one-command repro invokes the (external) downstream tool portably. *(Does not
  block `.1` — it is what `.1` decides.)*
- Knob-profile source: reuse `KNOB-ERGONOMICS-AND-PRESETS` presets once that lane
  lands, vs. an interim inline profile set. *(Cross-lane; resolve at `.1`.)*

## Blockers

- None. (Synergistic with `ACCEPTANCE-DIVERGENCE-HUNTING`,
  `DOWNSTREAM-ADAPTER-EXPANSION`, and `KNOB-ERGONOMICS-AND-PRESETS`, but not
  blocked by them.)

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-06-17` | `BUG-HUNT-ORCHESTRATION` | `tree registered (docs-only); no code` | `registered` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `BUG-HUNT-ORCHESTRATION` | `USABILITY-LANE-OWNERSHIP.1 — register 7 owner-directed usability/capability lanes + API-first decision 0017` | Tree registered (not yet started); frontier `.1` (design ADR) pending. |

## Changelog

- `2026-06-17`: Created task tree (registration via `USABILITY-LANE-OWNERSHIP.1`).
