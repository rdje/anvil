---
id: ci-github-action
title: ANVIL ships a drop-in composite GitHub Action that runs `anvil hunt` in your CI
answers:
  - "how do I add the ANVIL GitHub Action to my workflow"
  - "what is the anvil action.yml uses snippet"
  - "what inputs does the ANVIL GitHub Action take"
  - "how do I pin the ANVIL Action to a release"
  - "does the ANVIL Action fail my CI on a finding"
  - "where does the ANVIL Action upload reproducer bundles"
  - "how do I use a locally-built anvil with the Action (anvil-bin)"
  - "how does the ANVIL Action find its release binary"
date: 2026-06-21
status: current
tags: [ci, github-action, composite, bug-hunt, hunt, release, distribution, usability, mcp, api]
evidence: 'action.yml (root composite action) + scripts/anvil_hunt_action.sh (entrypoint) + .github/workflows/action-selftest.yml. Reverify: bash -n scripts/anvil_hunt_action.sh && grep -q "using: composite" action.yml && grep -q "summary.\\?\\[.\\?.n_failures" scripts/anvil_hunt_action.sh && echo OK  (the entrypoint parses HuntReport.summary.n_failures; the composite action wires the hunt step + artifact upload + fail-on-finding).'
---

`CI-PACKAGING-DISTRIBUTION.2b` (decision [`0022`](../decisions/0022-ci-packaging-prebuilt-binaries-and-github-action.md))
ships a **drop-in composite GitHub Action** so a downstream-tool maintainer can
continuously fuzz their parser/elaborator/synth against valid-by-construction
SystemVerilog. It is a thin shim over the already-shipped `anvil hunt` engine
(decision `0018`), driven through the same CLI-shim-over-API surface (decision
`0017`) — **no Action-only path**, no vendored tools.

```yaml
# A downstream-tool maintainer's CI:
- uses: <owner>/anvil@v0.1.0       # pins the Action AND the downloaded binary
  with:
    tools: verilator,yosys         # the user's installed tools (no vendoring)
    profile: sv2023-upopts         # a curated knob preset (optional)
    seeds: 128
```

- **Inputs.** The hunt-flag set maps 1:1 onto `anvil hunt`
  (`tools`/`seed`/`seeds`/`profile`/`config`/`yosys-mode`/`diff-sim`/
  `divergence`/`budget`/`no-minimize`/`out`) plus Action-level plumbing:
  `anvil-version` (release to download; defaults to the ref the Action is pinned
  to), `anvil-bin` (use a prebuilt binary instead of downloading — for source
  builds / self-test), `artifact-name`, and `fail-on-finding`.
- **Binary resolution.** With no `anvil-bin`, the entrypoint downloads
  `anvil-<version>-<target>.{tar.gz|zip}` from the `.2a` release for the runner
  OS/arch (`RUNNER_OS`/`RUNNER_ARCH` → target triple; repo from
  `GITHUB_ACTION_REPOSITORY`, version from `GITHUB_ACTION_REF`).
- **Red/green.** `anvil hunt` always exits 0 and prints a `HuntReport`; the
  entrypoint parses `summary.n_failures` into a `findings` step output, and the
  Action's `fail-on-finding` step fails the job when it is nonzero. The
  reproducer-bundle dir (per finding: `repro.sv`, `knobs.json`, `repro.sh`, …) is
  uploaded as a CI artifact.
- **Self-test.** `.github/workflows/action-selftest.yml` builds `anvil` locally,
  installs Verilator/Yosys best-effort, and runs the Action via `uses: ./` with
  `anvil-bin`; it **skips clean** when no downstream tools are present (an absent
  tool is a spawn-failure *finding* in `hunt`, not a no-op).

Because ANVIL output is valid by construction, a clean sweep (`findings = 0`) is
the **expected** result — a finding is a candidate **downstream-tool** bug, never
an ANVIL bug. The release workflow itself is [`.2a`](../decisions/0022-ci-packaging-prebuilt-binaries-and-github-action.md);
user-facing usage is in `USER_GUIDE.md` ("Use ANVIL in your CI") and
`book/src/recipes.md`. CI-infra only ⇒ default DUT output byte-identical.
