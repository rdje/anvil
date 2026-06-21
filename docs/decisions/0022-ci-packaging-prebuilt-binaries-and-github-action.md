---
id: ci-packaging-prebuilt-binaries-and-github-action
title: Drop-in CI packaging — a hand-rolled tag-triggered release workflow (prebuilt anvil/anvil-mcp binaries) + a composite GitHub Action that wraps `anvil hunt`/`divergence` through the same CLI-shim-over-API surface (decision 0017)
answers:
  - "how do I run ANVIL in CI"
  - "is there an ANVIL GitHub Action"
  - "how do I fuzz my SystemVerilog tool with ANVIL in CI"
  - "are there prebuilt ANVIL release binaries"
  - "how do I get ANVIL release binaries"
  - "what is the ANVIL CI packaging plan"
  - "does ANVIL ship a GitHub Action to fuzz my parser or elaborator"
  - "how do I continuously fuzz my downstream tool against ANVIL"
  - "what release mechanism does ANVIL use"
  - "does the ANVIL Action vendor the downstream tools"
date: 2026-06-18
status: accepted
tags: [usability, ci, packaging, release, github-action, bug-hunt, divergence, mcp, api, reproducible, north-star, distribution]
evidence: docs/decisions/0022-ci-packaging-prebuilt-binaries-and-github-action.md; docs/decisions/0017-api-first-everything-mcp-accessible.md; docs/decisions/0018-bug-hunt-orchestration-loop.md; docs/decisions/0019-acceptance-divergence-hunting.md; docs/decisions/0021-knob-ergonomics-presets-and-queryable-catalog.md; docs/tasks/CI-PACKAGING-DISTRIBUTION.md; src/hunt/mod.rs; src/main.rs (the `anvil hunt` subcommand)
---

# 0022 - CI packaging: a hand-rolled release workflow + a composite GitHub Action over `anvil hunt`

- Date: 2026-06-18
- Status: accepted (design; implementation pending under the pre-split `.2`)
- Tree: `CI-PACKAGING-DISTRIBUTION.1` (design/decision leaf; no code/CI yet).
- Binds: decision [`0017`](0017-api-first-everything-mcp-accessible.md) (the Action
  drives the same CLI-shim-over-API surface — no Action-only private path).
- Wraps: decision [`0018`](0018-bug-hunt-orchestration-loop.md) (the `anvil hunt`
  engine) and [`0019`](0019-acceptance-divergence-hunting.md) (the `divergence`
  detector) — **both already shipped**, so the Action has a real engine to wrap.
- Feeds from: decision [`0021`](0021-knob-ergonomics-presets-and-queryable-catalog.md)
  (`--profile` presets — a maintainer fuzzes with `--profile sv2023-upopts` instead
  of hand-authored config JSON).

## Context

The north star is to **surface downstream-tool bugs** (`project_anvil_north_star`).
The `anvil hunt` loop (decision `0018`) already makes that turnkey *locally*: fuzz a
deterministic seed sweep, run the vetted tools, treat any reject/warning (and, with
`--diff-sim`/`--divergence`, a cross-simulator mismatch / cross-tool acceptance
disagreement) as a finding, auto-minimize, and drop a one-command-reproducible
bundle per finding (`anvil hunt … --out <dir>`). What is missing is **adoption
friction**: a downstream-tool maintainer who wants ANVIL fuzzing their
parser/elaborator/synth in CI today must build ANVIL from source and hand-wire the
invocation. This lane removes both frictions — **prebuilt binaries** + a **drop-in
GitHub Action** — without adding a parallel engine.

## Decision

Two artifacts, both thin wrappers over what already ships; no generator change,
default DUT output byte-identical.

### 1. Release mechanism — a hand-rolled, tag-triggered GitHub Actions workflow

A `.github/workflows/release.yml` triggered on `v*` tags that, over a build matrix,
compiles `--release` and uploads per-platform tarballs (each containing **both**
`anvil` and `anvil-mcp`) plus a `SHA256SUMS` file to the GitHub Release.

- **Target matrix:** `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu`,
  `x86_64-apple-darwin`, `aarch64-apple-darwin`, `x86_64-pc-windows-msvc`. (musl
  Linux is an easy add if static linking is wanted; not required for the first cut.)
- **Reproducibility / version-pin:** a pinned toolchain (a `rust-toolchain.toml`
  or a pinned `actions` toolchain step) + the committed `Cargo.lock` give a
  repeatable build; the artifacts are checksummed; the release tag *is* the version
  pin the Action consumes.
- **Why hand-rolled, not `cargo-dist`:** it matches ANVIL's explicit
  dependency-averse, transparent-infra ethos (the README's "hand-rolled
  loopback-default transport … no new dependency" precedent) — a plain matrix
  workflow is auditable in-repo, adds no release-tool dependency or generated
  config to maintain, and the target set is small. `cargo-dist` is recorded as a
  rejected alternative (revisitable if the matrix/installer surface grows).

### 2. A **composite** GitHub Action wrapping `anvil hunt`

A composite action (`action.yml` + a small POSIX entrypoint), **not** a container
action. It (a) downloads the pinned ANVIL release tarball for the runner OS, (b)
runs `anvil hunt` with the user's inputs, (c) uploads the reproducer-bundle dir as
a CI artifact, and (d) sets the job's exit status from the `HuntReport` (red on a
finding, configurable).

**Inputs** (each maps 1:1 onto an `anvil hunt` flag / a `hunt`-tool control — the
decision-`0017` "same controls an MCP agent would set", no Action-only path):

| Input | Maps to | Default |
| --- | --- | --- |
| `tools` | `--tools verilator,yosys` (the user-installed downstream tools to fuzz) | `verilator,yosys` |
| `seed` / `seeds` | `--seed` / `--seeds` (deterministic sweep) | `42` / `64` |
| `profile` | `--profile <name>` (a decision-`0021` preset) | none |
| `config` | `--config <path>` (a Config JSON in the user's repo) | none |
| `yosys-mode` | `--yosys-mode` | `without-abc` |
| `diff-sim` / `divergence` | `--diff-sim` / `--divergence` (extra detectors) | `false` |
| `budget` / `no-minimize` | `--budget` / `--no-minimize` | `200` / `false` |
| `out` | `--out <dir>` (bundle dir → uploaded artifact) | a workspace path |
| `anvil-version` | which release tarball to download | the action's pinned release |
| `fail-on-finding` | whether a finding fails the job | `true` |

The composite action is **transparent** (just steps), runs on the user's chosen
runner, and invokes only the user's installed tool(s).

### 3. How the Action invokes the surface (decision 0017)

Through the `anvil hunt` **CLI** (itself the shim over `hunt::run`; decision `0018`),
with `--out` writing the byte-identical-reproducible bundles. The Action introduces
**no** private invocation path: its inputs are the same controls the MCP `hunt`
tool exposes, so a CI run and an interactive/agent run share one engine. (A future
MCP-driven variant could call the `hunt` tool directly; same engine either way.)

### 4. Version-pin + reproducibility contract for CI failures

A CI finding is locally reproducible because: (a) the Action pins an ANVIL release
(`anvil-version`); (b) each bundle already carries `knobs.json` + `repro.sh`
(`anvil --seed S --config knobs.json` regenerates the exact `.sv` byte-for-byte,
then re-runs the captured tool `argv`); and (c) the hunt is deterministic (seeded
ChaCha8), so `(anvil-version, seed, seeds, profile/config, tools)` reproduces the
sweep. The Action surfaces the effective request + the observed tool versions in
its log + the uploaded bundles.

### 5. Non-goals (restated, load-bearing)

- **No generator/semantics change** — release/packaging/CI only; default DUT output
  byte-identical.
- **No vendoring of downstream tools** — the Action invokes the user's installed
  tool(s); it does not bundle Verilator/Yosys/etc. (Resolves the tree's open
  question: default = user-installed.)
- **Not a hosted service** — local/CI artifacts only.

## Pre-split of `.2` (implementation)

- `.2a` — the **release workflow** (`.github/workflows/release.yml`: the build
  matrix, `--release` build of `anvil`+`anvil-mcp`, tarball + `SHA256SUMS`, GitHub
  Release upload on `v*`; a pinned toolchain). CI-infra; task-tree-owned.
- `.2b` — the **composite Action** (`action.yml` + entrypoint): download the pinned
  tarball, run `anvil hunt` with the mapped inputs, upload the bundle artifact,
  exit-on-finding. A self-test job that runs the Action against the repo's own tools
  (skips clean when absent).
- `.2c` — **docs + close**: README + USER_GUIDE "Use ANVIL in your CI" (a copy-paste
  `uses:` snippet) + a KM card; close `.2` and leave the tree `active` (open-ended:
  more targets / a Marketplace listing / an MCP-driven variant are optional `.N`).

## Rejected alternatives

- **`cargo-dist` for releases.** Rejected for the first cut: it adds a release-tool
  dependency + generated workflow config and is more opinionated than a small,
  in-repo, auditable matrix workflow — against the project's hand-rolled,
  dependency-averse ethos. Revisitable if the target matrix / installer surface
  grows.
- **A container (Docker) GitHub Action.** Rejected: it would bake in a fixed OS +
  (tempting to vendor) downstream tools, contradicting the no-vendoring non-goal,
  and is heavier to publish/maintain than a composite action. A composite action
  runs on the user's runner with the user's tools.
- **A bespoke CI fuzzing script (no shared engine).** Rejected by decision `0017`:
  the Action must drive the same `anvil hunt`/`hunt::run` surface as the CLI/MCP —
  no Action-only path, or detection logic would fork and drift.
- **Vendoring Verilator/Yosys into the Action image.** Rejected (non-goal): licensing
  + image-size + staleness; the user installs and pins their own tool versions (the
  versions whose bugs they care about).
- **Publishing to crates.io / a package manager as the primary distribution.**
  Out of scope here (a possible later add): the immediate need is *prebuilt
  binaries + a CI Action*, not a source package.

## Consequences

- `.2` lands `.github/workflows/release.yml` + a composite `action.yml`, both thin
  wrappers over the shipped `anvil hunt` engine — no new generation path,
  default-off / DUT byte-identical.
- ANVIL becomes **drop-in adoptable** as a continuous downstream fuzzer: a
  maintainer adds a `uses:` step pinned to an ANVIL release, names their tool +
  a `--profile`, and gets red CI + reproducer bundles on any reject/warning/
  divergence — the north star, delivered to *other people's* CI.
- The feeding lanes compose without reopening: `--profile` presets (decision `0021`)
  shape the corpus, `DOWNSTREAM-ADAPTER-EXPANSION` adds more `--tools`, and
  `ACCEPTANCE-DIVERGENCE-HUNTING` adds the `--divergence` detector — all reachable
  through the same Action inputs.

## Links

- Owning tree: `docs/tasks/CI-PACKAGING-DISTRIBUTION.md` (this is its `.1` leaf;
  pre-splits `.2a`/`.2b`/`.2c`).
- Parent/wrapped decisions: `0017` (API-completeness gate), `0018` (the `anvil hunt`
  engine), `0019` (the `divergence` detector), `0021` (`--profile` presets).
- Memory: `project_anvil_north_star` (surface downstream-tool bugs — to *other*
  toolchains' CI), `feedback_api_for_agents_not_humans` (one engine, CLI/MCP/Action
  are shims).
