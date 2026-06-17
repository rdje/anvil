# ACCEPTANCE-DIVERGENCE-HUNTING: tool-A-accepts / tool-B-rejects divergence finder

## Metadata

- Tree ID: `ACCEPTANCE-DIVERGENCE-HUNTING`
- Status: `active`
- Roadmap lane: `Usability — acceptance-divergence bug-finder (north star, idea 2)`
- Created: `2026-06-17`
- Last updated: `2026-06-17`
- Owner: repo-local workflow

## Goal

Make **acceptance divergence** a first-class signal. `--diff-sim` already proves
cross-*simulator* trace agreement; this lane adds the complementary axis:
detecting and reporting where **one tool accepts an artifact and another rejects
it** (or where two *versions* of the same tool disagree). Such accept/reject
divergence on valid-by-construction RTL is exactly where real downstream-tool
bugs live. Deliver a per-unit per-tool accept/warn/reject matrix, a divergence
classifier, and a report — surfaced as a `tool_matrix` column **and** as an MCP
query — building on the existing hardened `src/downstream/` adapters and the
`src/diff_sim/` precedent.

## Non-Goals

- No behavioural oracle (decision `0004`, ROADMAP gap 4) — this is about
  *acceptance* divergence (parse/elaborate/lint/synth verdicts), composed with
  the existing semantic-agreement column, not a new truth model.
- No new generator semantics; default DUT output stays byte-identical.
- No vendoring of tools; divergence is computed over external, sandboxed
  invocations.

## Acceptance Criteria

- A run produces a per-artifact accept/warn/reject matrix across the enabled
  tools (and/or tool versions) and flags every divergence, with the divergent
  artifact retained as a reproducer (seed + effective knobs + `.sv` + each tool's
  log).
- **API-completeness gate (decision `0017`):** the divergence run is invocable
  over MCP and every divergence verdict/report is queryable via the
  MCP/introspection API (SCHEMA-DERIVED — a projection of the recorded verdicts,
  not a recomputed truth); the CLI/`tool_matrix` flag is a shim over the same
  surface.
- Reproducible + sandboxed (seeded; allow-list + RAM guard + audit log).
- Default-off / DUT byte-identical; downstream-clean; documented in
  `book/src/agent-mcp.md` + `book/src/synthesizability.md` + USER_GUIDE + README;
  committed through `COMMIT.md`.

## Task Tree

- ID: `ACCEPTANCE-DIVERGENCE-HUNTING`
  Status: `active`
  Goal: `A first-class accept/warn/reject divergence finder across tools (and tool versions), surfaced as a tool_matrix column + an MCP query, built on the existing downstream adapters + the diff_sim precedent.`
  Children: `ACCEPTANCE-DIVERGENCE-HUNTING.1`

- ID: `ACCEPTANCE-DIVERGENCE-HUNTING.1`
  Status: `pending`
  Goal: `Design/decision leaf (ADR, no code): pin the divergence model (per-unit per-tool verdict = accept/warn/reject + the divergence classification, incl. tool-version-vs-version), the report shape (a DivergenceReport beside DiffSimReport), the tool_matrix column + the MCP query surface (decision 0017 API-completeness), and the reproducer-retention policy. Decide reuse of run_verilator/run_yosys/run_iverilog + the diff_sim subset-selection pattern. Record as the next decision record + pre-split .2 (impl).`
  Acceptance: `A decision record + a tree/DEVELOPMENT_NOTES entry pinning the divergence model, the report, and the MCP+matrix surface; docs-only; INDEX + this tree + docs/TASK_TREE.md updated.`
  Verification: `pending`
  Commit: `pending`

- ID: `ACCEPTANCE-DIVERGENCE-HUNTING.2`
  Status: `pending`
  Goal: `Implement the .1 design: the divergence column/report + the MCP query + reproducer retention + proofs + a real-tool end-to-end gate + book/USER_GUIDE/README/KM. Default-off / DUT byte-identical. Pre-split when picked.`
  Acceptance: `pending (set at .1)`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `ACCEPTANCE-DIVERGENCE-HUNTING.1` | `pending` | Design-first ADR pins the divergence model + the MCP/matrix surface (decision `0017`) before any code; reuses the `diff_sim` + `downstream` precedents. |

## Decisions

- `2026-06-17`: Registered as an owner-directed usability lane (idea 2). Binds
  decision [`0017`](../decisions/0017-api-first-everything-mcp-accessible.md).
  Complements `DIFFERENTIAL-SIMULATION` (cross-sim trace agreement) with
  accept/reject divergence; design-first ADR before code.

## Open Questions

- Tool-version-vs-version divergence: how versions are pinned/selected portably
  (PATH shims vs. explicit binaries). *(Resolved at `.1`.)*
- Whether divergence detection rides the `BUG-HUNT-ORCHESTRATION` loop or is an
  independent `tool_matrix` column (likely both — a shared detector). *(Cross-lane;
  resolve at `.1`.)*

## Blockers

- None. (Synergistic with `BUG-HUNT-ORCHESTRATION` and
  `DOWNSTREAM-ADAPTER-EXPANSION`; not blocked by them.)

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-06-17` | `ACCEPTANCE-DIVERGENCE-HUNTING` | `tree registered (docs-only); no code` | `registered` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `ACCEPTANCE-DIVERGENCE-HUNTING` | `USABILITY-LANE-OWNERSHIP.1 — register 7 owner-directed usability/capability lanes + API-first decision 0017` | Tree registered (not yet started); frontier `.1` (design ADR) pending. |

## Changelog

- `2026-06-17`: Created task tree (registration via `USABILITY-LANE-OWNERSHIP.1`).
