# CI-PACKAGING-DISTRIBUTION: prebuilt binaries + a drop-in GitHub Action

## Metadata

- Tree ID: `CI-PACKAGING-DISTRIBUTION`
- Status: `active`
- Roadmap lane: `Usability — drop-in CI packaging (north star, idea 5)`
- Created: `2026-06-17`
- Last updated: `2026-06-18`
- Owner: repo-local workflow

## Goal

Make ANVIL trivial to adopt as a continuous fuzzer in someone else's toolchain:
**prebuilt binaries** (per-platform release artifacts) and a **GitHub Action** so
a downstream-tool maintainer can drop ANVIL into their CI and continuously fuzz
their parser/elaborator/synth against valid-by-construction RTL. The Action wraps
the same bug-hunt / acceptance-divergence surface (driven through the API, not a
bespoke script), so CI usage and interactive usage share one engine.

## Non-Goals

- No change to generator semantics; this is release/packaging/CI infrastructure.
  Default DUT output stays byte-identical.
- No bundling of the downstream tools into the Action image beyond what a user
  opts into; the Action invokes the user's installed tool(s).
- Not a hosted service — local/CI artifacts only.

## Acceptance Criteria

- Reproducible release artifacts (prebuilt `anvil` + `anvil-mcp` binaries for the
  target platforms) are produced by a documented, repeatable release path.
- A GitHub Action (composite or container) runs an ANVIL bug-hunt /
  acceptance-divergence pass against a user-named tool and surfaces failures
  (with reproducer bundles) as CI output/artifacts.
- **API-completeness gate (decision `0017`):** the Action drives ANVIL through
  the same CLI-shim-over-API surface the `BUG-HUNT-ORCHESTRATION` /
  `ACCEPTANCE-DIVERGENCE-HUNTING` lanes expose — no Action-only private path; its
  configuration maps onto the same controls an MCP agent would set.
- Version-pinned + reproducible (the Action pins an ANVIL release + records the
  effective knobs/seeds so a CI failure is reproducible locally).
- Documented in README + USER_GUIDE (a "use ANVIL in your CI" section);
  committed through `COMMIT.md`.

## Task Tree

- ID: `CI-PACKAGING-DISTRIBUTION`
  Status: `active`
  Goal: `Prebuilt per-platform release binaries + a drop-in GitHub Action that runs an ANVIL bug-hunt/divergence pass against a user's tool, driven through the same API surface.`
  Children: `CI-PACKAGING-DISTRIBUTION.1`

- ID: `CI-PACKAGING-DISTRIBUTION.1`
  Status: `done`
  Goal: `Design/decision leaf (ADR, no code/CI yet): pin the release mechanism (e.g. cargo-dist vs hand-rolled GitHub release workflow; which targets), the GitHub Action shape (composite action vs container; inputs = tool, seeds, profile, budgets), how the Action invokes the bug-hunt/divergence surface (CLI shim over API, decision 0017), and the version-pin + reproducibility contract for CI failures. Note the dependency on BUG-HUNT-ORCHESTRATION (the engine the Action wraps). Record as the next decision record + pre-split .2 (impl).`
  Acceptance: `A decision record + a tree/DEVELOPMENT_NOTES entry pinning the release path, the Action shape, and the API-driven invocation; docs-only; INDEX + this tree + docs/TASK_TREE.md updated.`
  Verification: `done — decision 0022: hand-rolled tag-triggered release.yml (5-target matrix, anvil+anvil-mcp tarballs + SHA256SUMS; cargo-dist rejected for the dependency-averse ethos); a composite GitHub Action (not container) wrapping anvil hunt with inputs mapped 1:1 onto the CLI/MCP controls (decision 0017); user-installed tools (no vendoring); version-pin + per-bundle repro.sh/knobs.json reproducibility. Wraps the already-shipped anvil hunt (0018) + divergence (0019). Pre-split .2a/.2b/.2c. INDEX + tree + TASK_TREE + DEVELOPMENT_NOTES updated; KM regen; docs-only / DUT byte-identical.`
  Commit: `CI-PACKAGING-DISTRIBUTION.1 — design ADR (decision 0022): release workflow + composite GitHub Action over anvil hunt`

- ID: `CI-PACKAGING-DISTRIBUTION.2`
  Status: `pending`
  Goal: `Implement the .1 design (decision 0022). Pre-split: .2a (.github/workflows/release.yml — the hand-rolled v* tag build matrix → anvil+anvil-mcp tarballs + SHA256SUMS), .2b (the composite action.yml + entrypoint wrapping anvil hunt + artifact upload + exit-on-finding + a self-test job), .2c (README/USER_GUIDE "Use ANVIL in your CI" + a KM card; close).`
  Acceptance: `set at .1 (decision 0022): reproducible per-platform release artifacts on v* tags; a composite Action that runs anvil hunt against a user-named tool and surfaces findings (reproducer bundles) as CI artifacts, driven through the same CLI-shim-over-API surface (no Action-only path); user-installed tools; version-pinned + locally reproducible.`
  Verification: `pending`
  Commit: `pending`

  Children: `CI-PACKAGING-DISTRIBUTION.2a` (release workflow), `.2b` (the Action), `.2c` (docs + close).

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `CI-PACKAGING-DISTRIBUTION.1` | `done` | Design ADR (decision `0022`) pinned the hand-rolled release workflow, the composite Action shape, the CLI-shim-over-API invocation (decision `0017`), and the version-pin/reproducibility contract. The wrapped engine (`anvil hunt`, decision `0018`) already ships. |
| 2 | `CI-PACKAGING-DISTRIBUTION.2a` | `pending` | First impl slice: the tag-triggered `release.yml` build matrix (prebuilt `anvil`/`anvil-mcp` tarballs + checksums). CI-infra; task-tree-owned. |

## Decisions

- `2026-06-17`: Registered as an owner-directed usability lane (idea 5). Binds
  decision [`0017`](../decisions/0017-api-first-everything-mcp-accessible.md):
  the Action drives the same API surface, no private path. Design-first ADR
  before any workflow/CI YAML.
- `2026-06-18` (`.1`): Design ADR landed as decision
  [`0022`](../decisions/0022-ci-packaging-prebuilt-binaries-and-github-action.md):
  a **hand-rolled** `v*`-tag `release.yml` (5-target matrix → `anvil`+`anvil-mcp`
  tarballs + `SHA256SUMS`; `cargo-dist` rejected per the dependency-averse ethos);
  a **composite** GitHub Action (not container) wrapping `anvil hunt` with inputs
  (`tools`/`seed`/`seeds`/`profile`/`config`/`yosys-mode`/`diff-sim`/`divergence`/
  `budget`/`out`/`anvil-version`/`fail-on-finding`) mapped 1:1 onto the CLI/MCP
  controls; **user-installed tools (no vendoring)**; version-pin + per-bundle
  `repro.sh`/`knobs.json` reproducibility. Wraps the **already-shipped** `anvil
  hunt` (decision `0018`) + `divergence` (decision `0019`), and consumes
  `--profile` presets (decision `0021`). Pre-split `.2a`/`.2b`/`.2c`.

## Open Questions

- Release tooling choice (`cargo-dist` vs. a hand-rolled workflow) + target
  platform matrix. *(Resolved at `.1` / decision `0022`: hand-rolled matrix over
  5 targets — linux gnu x86_64/aarch64, macOS x86_64/aarch64, windows msvc;
  `cargo-dist` rejected for the first cut.)*
- Whether the Action ships its own pinned downstream tool(s) or requires the
  user to install them. *(Resolved at `.1`; default = user-installed, per the
  no-vendoring non-goal.)*

## Blockers

- Soft dependency on `BUG-HUNT-ORCHESTRATION` (the engine the Action wraps). Not
  a hard blocker for `.1` (the design can proceed and reference the planned
  surface); `.2` should follow `BUG-HUNT-ORCHESTRATION.2`.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-06-17` | `CI-PACKAGING-DISTRIBUTION` | `tree registered (docs-only); no code` | `registered` |
| `2026-06-18` | `CI-PACKAGING-DISTRIBUTION.1` | `decision 0022 written; INDEX + tree + TASK_TREE + DEVELOPMENT_NOTES updated; KM regen+check green; mem-arch green; docs-only / DUT byte-identical` | `done` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `CI-PACKAGING-DISTRIBUTION` | `USABILITY-LANE-OWNERSHIP.1 — register 7 owner-directed usability/capability lanes + API-first decision 0017` | Tree registered (not yet started); frontier `.1` (design ADR) pending. |
| `CI-PACKAGING-DISTRIBUTION.1` | `CI-PACKAGING-DISTRIBUTION.1 — design ADR (decision 0022): release workflow + composite GitHub Action over anvil hunt` | Design-only; pins the hand-rolled release matrix, the composite Action shape + inputs, the CLI-shim-over-API invocation, and reproducibility; pre-splits `.2` into `.2a`/`.2b`/`.2c`. |

## Changelog

- `2026-06-17`: Created task tree (registration via `USABILITY-LANE-OWNERSHIP.1`).
- `2026-06-18`: `.1` design ADR landed (decision `0022`); frontier advances to
  `.2a` (the release workflow). Docs-only / DUT byte-identical.
