# CI-PACKAGING-DISTRIBUTION: prebuilt binaries + a drop-in GitHub Action

## Metadata

- Tree ID: `CI-PACKAGING-DISTRIBUTION`
- Status: `active`
- Roadmap lane: `Usability — drop-in CI packaging (north star, idea 5)`
- Created: `2026-06-17`
- Last updated: `2026-06-21`
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

- ID: `CI-PACKAGING-DISTRIBUTION.2a`
  Status: `done`
  Goal: `The tag-triggered hand-rolled release workflow (.github/workflows/release.yml): a v* build matrix over the 5 decision-0022 targets, each building --release --locked anvil+anvil-mcp, packaging both binaries + README into a per-platform archive (.tar.gz Unix / .zip Windows) + per-archive sha256, then a least-privilege publish job that assembles one SHA256SUMS and creates/updates the GitHub Release.`
  Acceptance: `release.yml present and structurally valid; tag-only v* trigger; 5 targets (linux gnu x86_64/aarch64, macOS x86_64/aarch64, windows msvc); --release --locked --bin anvil --bin anvil-mcp; per-platform archive bundling both binaries; SHA256SUMS aggregated and uploaded; no third-party release dep (gh CLI); pinned toolchain + Cargo.lock; default DUT byte-identical (no src change).`
  Verification: `done — release.yml authored; pure-Python structural lint clean (no pyyaml/actionlint/yq offline): no tabs/trailing-ws/odd map indents, all 5 targets + all required tokens present (on/tags/v*, both permissions scopes, --release --locked, --bin anvil --bin anvil-mcp, SHA256SUMS, gh release create/upload). mem-arch + KM (56) gates green. No Rust touched ⇒ cargo suite unaffected (full cargo test green at 51d97d9; snapshots 6/6).`
  Commit: `CI-PACKAGING-DISTRIBUTION.2a — hand-rolled v*-tag release workflow (release.yml)`

- ID: `CI-PACKAGING-DISTRIBUTION.2b`
  Status: `done`
  Goal: `The drop-in composite GitHub Action wrapping anvil hunt: a root action.yml + scripts/anvil_hunt_action.sh entrypoint that resolves an anvil binary (anvil-bin escape hatch, else the pinned release tarball for the runner OS/arch), runs anvil hunt with inputs mapped 1:1 onto its flags into a bundle dir, parses HuntReport.summary.n_failures, uploads the bundle as a CI artifact, and fails the job on a finding (configurable). Plus a presence-gated self-test workflow.`
  Acceptance: `action.yml (composite, root) + entrypoint present; hunt-flag inputs map 1:1 onto anvil hunt (tools/seed/seeds/profile/config/yosys-mode/diff-sim/divergence/budget/no-minimize/out) + Action-level plumbing (anvil-version/anvil-bin/artifact-name/fail-on-finding); invocation via the anvil hunt CLI shim over hunt::run (no Action-only path, decision 0017); artifact upload; exit-on-finding; a self-test that runs the Action against the repo's own tools and skips clean when absent; default DUT byte-identical (no src change).`
  Verification: `done — action.yml + scripts/anvil_hunt_action.sh + .github/workflows/action-selftest.yml authored. REAL local end-to-end smoke of the entrypoint (anvil-bin path) against the release anvil + verilator + yosys over a 3-seed sweep: exit 0, $GITHUB_OUTPUT findings=0 + report path + bundle-dir, report valid JSON (summary n_seeds:3/n_clean:3/n_failures:0), human summary printed. bash -n clean; pure-Python structural lint of both YAML files clean (all tokens present; offline). mem-arch + KM(56) green. No Rust ⇒ cargo suite unaffected (full cargo test green at 51d97d9; snapshots 6/6).`
  Commit: `CI-PACKAGING-DISTRIBUTION.2b — composite Action over anvil hunt`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `CI-PACKAGING-DISTRIBUTION.1` | `done` | Design ADR (decision `0022`) pinned the hand-rolled release workflow, the composite Action shape, the CLI-shim-over-API invocation (decision `0017`), and the version-pin/reproducibility contract. The wrapped engine (`anvil hunt`, decision `0018`) already ships. |
| 2 | `CI-PACKAGING-DISTRIBUTION.2a` | `done` | Landed `.github/workflows/release.yml`: tag-triggered 5-target build matrix → `anvil`+`anvil-mcp` archives + `SHA256SUMS` to the GitHub Release, `gh`-CLI publish (no third-party release dep). CI-infra; task-tree-owned. |
| 3 | `CI-PACKAGING-DISTRIBUTION.2b` | `done` | Landed the composite Action (`action.yml` + `scripts/anvil_hunt_action.sh` + a presence-gated self-test), driven through the same `anvil hunt` CLI-shim-over-API surface (decision `0017`); proven end-to-end by a real local smoke (findings=0 on clean DUT output). |
| 4 | `CI-PACKAGING-DISTRIBUTION.2c` | `pending` | Docs + close: README/USER_GUIDE "Use ANVIL in your CI" (a copy-paste `uses:` snippet) + a KM card; close `.2`. The tree stays `active` (more targets / a Marketplace listing / an MCP-driven variant are optional `.N`). |

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
| `2026-06-21` | `CI-PACKAGING-DISTRIBUTION.2a` | `release.yml authored; pure-Python structural lint clean (5 targets + all required tokens; no tabs/trailing-ws/odd indents); mem-arch + KM(56) green; no Rust touched ⇒ DUT byte-identical` | `done` |
| `2026-06-21` | `CI-PACKAGING-DISTRIBUTION.2b` | `action.yml + scripts/anvil_hunt_action.sh + action-selftest.yml authored; REAL local entrypoint smoke (anvil-bin + verilator+yosys, 3 seeds): exit 0, findings=0, valid-JSON report, outputs wired; bash -n + structural YAML lint clean; mem-arch + KM(56) green; no Rust ⇒ DUT byte-identical` | `done` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `CI-PACKAGING-DISTRIBUTION` | `USABILITY-LANE-OWNERSHIP.1 — register 7 owner-directed usability/capability lanes + API-first decision 0017` | Tree registered (not yet started); frontier `.1` (design ADR) pending. |
| `CI-PACKAGING-DISTRIBUTION.1` | `CI-PACKAGING-DISTRIBUTION.1 — design ADR (decision 0022): release workflow + composite GitHub Action over anvil hunt` | Design-only; pins the hand-rolled release matrix, the composite Action shape + inputs, the CLI-shim-over-API invocation, and reproducibility; pre-splits `.2` into `.2a`/`.2b`/`.2c`. |
| `CI-PACKAGING-DISTRIBUTION.2a` | `CI-PACKAGING-DISTRIBUTION.2a — hand-rolled v*-tag release workflow (release.yml)` | First impl slice of decision `0022`: `.github/workflows/release.yml` (5-target build matrix → `anvil`+`anvil-mcp` archives + `SHA256SUMS`, `gh`-CLI publish). CI-infra; no `src` change ⇒ DUT byte-identical. |
| `CI-PACKAGING-DISTRIBUTION.2b` | `CI-PACKAGING-DISTRIBUTION.2b — composite Action over anvil hunt` | Second impl slice: root `action.yml` + `scripts/anvil_hunt_action.sh` entrypoint + a presence-gated self-test; a thin shim over `anvil hunt` (decision `0017`/`0018`). Proven by a real local entrypoint smoke. CI-infra; no `src` change ⇒ DUT byte-identical. |

## Changelog

- `2026-06-17`: Created task tree (registration via `USABILITY-LANE-OWNERSHIP.1`).
- `2026-06-18`: `.1` design ADR landed (decision `0022`); frontier advances to
  `.2a` (the release workflow). Docs-only / DUT byte-identical.
- `2026-06-21`: `.2a` landed (`.github/workflows/release.yml` — the hand-rolled
  `v*`-tag 5-target release matrix → `anvil`+`anvil-mcp` archives + `SHA256SUMS`,
  `gh`-CLI publish, no third-party release dep). Frontier advances to `.2b` (the
  composite Action). CI-infra / DUT byte-identical.
- `2026-06-21`: `.2b` landed (root `action.yml` + `scripts/anvil_hunt_action.sh`
  entrypoint + `.github/workflows/action-selftest.yml` — the drop-in composite
  Action wrapping `anvil hunt`, proven end-to-end by a real local entrypoint
  smoke). Frontier advances to `.2c` (docs + close). CI-infra / DUT
  byte-identical.
