# Evidence inventory — the frozen classification of every `anvil-<name>` token

**Derived once, on `2026-07-30`, from the tree itself. Not hand-curated prose.**
`scripts/check_evidence_citations.sh` reads this file. Decision
[`0030`](../decisions/0030-durable-closure-evidence-citations.md) and its
`2026-07-30` amendment define what the two sections mean.

Every `anvil-<name>` token appearing in the scanned document set must be
classified **exactly once**, into one of three buckets:

| Bucket | Where it lives | Grows? |
| --- | --- | --- |
| **digest-backed** | a file `docs/evidence/<name>.md` | **yes, without limit** — this is the forward path |
| **grandfathered** | §1 of this file | **no — frozen** |
| **not evidence** | §2 of this file | yes, under review |

An unclassified token is a **breach**: the check fails closed.

---

## 1. Grandfathered banks — pre-`0030`, artifact gone (72)

These are the closure and gate artifacts this repository cited **before**
decision `0030` existed. Every one of them was a directory under the OS
temp dir; all were purged (`ls -d /tmp/anvil-*` → 0 directories at the
`2026-07-25` audit that opened `EVIDENCE-BANK-DURABILITY`). No digest can
be written for them after the fact: a re-run on today's generator produces
*new* evidence, not the cited artifact, and fabricating a digest for an old
claim would violate the no-aspirational-claims rule (`COMMIT.md`).

**How to re-verify a claim backed by one of these:** run the named gate
command at the commit the claim was made at. That is the oracle leg, and it
is the only honest path. The bank name is a **breadcrumb** identifying the
run, not a retrievable artifact.

**THIS LIST IS FROZEN AND CANNOT GROW.** The check pins both its entry count
and the SHA-256 of its sorted membership. That is not bureaucracy: the set of
banks that existed before `0030` is a *historical fact*, so a grandfathered
list that grows is simply a false statement. Adding an entry requires editing
the pin in `scripts/check_evidence_citations.sh` too — a deliberate,
reviewable act. If you are reaching for it to admit a **new** bank, you want
a digest instead.

### (a) Phase-closing banks (10)

- `anvil-frontend-parity-phase8-yosys-p1`
- `anvil-microdesign-parity-phase7-yosys-p1`
- `anvil-tool-matrix-phase1-real-r21`
- `anvil-tool-matrix-phase2-share-r1`
- `anvil-tool-matrix-phase3-structured-r4`
- `anvil-tool-matrix-phase4-hierarchy-r87`
- `anvil-tool-matrix-phase5-p1`
- `anvil-tool-matrix-phase5b-p1`
- `anvil-tool-matrix-phase6-fsm-p1`
- `anvil-tool-matrix-phase6-p1`

### (b) Repo-owned surface / sweep gate banks (15)

- `anvil-case-mux-if-gate-r1`
- `anvil-casez-mux-if-gate-r1`
- `anvil-cone-function-gate-r1`
- `anvil-function-emit-gate-r1`
- `anvil-generate-loop-gate-8b`
- `anvil-generate-loop-gate-r1`
- `anvil-mo-k3-gate-r1`
- `anvil-multi-output-task-gate-r1`
- `anvil-mux-if-gate-r1`
- `anvil-signoff-knob-sweep-r1`
- `anvil-signoff-surface-iverilog-r1`
- `anvil-signoff-surface-nflop-r1`
- `anvil-sv-version-gate-r1`
- `anvil-sv-version-gate-upopt-r1`
- `anvil-task-emit-gate-r1`

### (c) Focused smokes, probes, and e2e verification banks (47)

- `anvil-cf-sweep`
- `anvil-diff-sim-p1`
- `anvil-divergence-col-smoke`
- `anvil-fe-r2`
- `anvil-gl-r1`
- `anvil-gl8b`
- `anvil-hier-child-input-cone-smoke-r1`
- `anvil-hier-depth-profile-smoke-r1`
- `anvil-hier-mixed-depth-smoke-r1`
- `anvil-hier-parent-compose-smoke-r1`
- `anvil-hier-parent-output-mix-smoke-r1`
- `anvil-hier-parent-state-smoke-r1`
- `anvil-hier-profiled-ondemand-smoke-r1`
- `anvil-hier-range-smoke-r1`
- `anvil-hier-registered-child-input-cone-smoke-r2`
- `anvil-hier-registered-mixed-child-input-smoke-r1`
- `anvil-hier-registered-multistage-child-input-smoke-r1`
- `anvil-hier-registered-sibling-smoke-r1`
- `anvil-hier-reuse-smoke-r1`
- `anvil-hier-sibling-routing-smoke-r1`
- `anvil-hier-under-smoke-r2`
- `anvil-hierarchy-smoke-r1`
- `anvil-ifelse-probe`
- `anvil-iverilog-compile-smoke-r2`
- `anvil-microdesign-parity-phase7-`
- `anvil-mo-sweep`
- `anvil-multi-clock-p2`
- `anvil-muxif-genproof`
- `anvil-parent-cone-instance-smoke-r1`
- `anvil-probe-se4`
- `anvil-se-motask-probe`
- `anvil-se9-probe`
- `anvil-seq-bank`
- `anvil-te-r1`
- `anvil-tool-matrix-phase4-hierarchy-r22`
- `anvil-tool-matrix-phase4-hierarchy-r7`
- `anvil-tool-matrix-phase4-mixed-helper-check`
- `anvil-tool-matrix-phase4-parent-cone-instance-r1`
- `anvil-tool-matrix-phase4-parent-helper-child-input-mixed-check`
- `anvil-tool-matrix-phase4-parent-output-helper-state-r3`
- `anvil-tool-matrix-phase4-parent-port-coverage-r1`
- `anvil-tool-matrix-phase4-recursive-direct-helper-r32`
- `anvil-tool-matrix-phase4-recursive-helper-state-r31`
- `anvil-tool-matrix-phase4-registered-mixed-r1`
- `anvil-tool-matrix-phase4-registered-multistage-r1`
- `anvil-tool-matrix-phase4-stateful-helper-child-input-mixed-check`
- `anvil-widelane-probe`

---

## 2. Not evidence — ANVIL's own vocabulary (21)

Tokens that match the `anvil-<name>` shape but are **not** citations of a
banked artifact: binary names, directory names, Action inputs, negative-control
fixtures, re-run output paths, and ordinary English prose. They are listed so
the check does not have to *guess* which tokens are claims — guessing is what
made the previous design unworkable once the `/tmp/` prefix disappeared.

This list **may grow under review** — ANVIL legitimately gains binaries and
directories. That asymmetry with §1 is deliberate and semantic: §1 describes a
closed historical fact, §2 describes a living vocabulary.

- `anvil-bin` — a GitHub Action input name (`action.yml`)
- `anvil-bisim-merged.sv` — the `ANVIL_DUMP_BISIM_SV` debug dump *file*
- `anvil-brandnew-gate-r9` — the `EVIDENCE-BANK-DURABILITY.3` negative-control fixture token
- `anvil-emitted` — English prose (“an anvil-emitted module”)
- `anvil-fake-bank-r1` — the `VOLUME-DATA-LOCALITY.7` negative-control fixture token
- `anvil-hunt-bundles` — the Action’s default reproducer-bundle artifact name
- `anvil-hunt-turnkey-cli-bug-hunt` — a **Markdown heading-anchor slug**: the `USER_GUIDE.md`
  in-page link target for the `### \`anvil hunt\` (turnkey CLI bug-hunt)` section
  (`USER-GUIDE-CLI-TABLE-SHADOW.2`). A *third* collision class for the citation shape, after
  binaries and directories — any heading whose text begins "anvil …" slugifies into it
- `anvil-hunt` — the CLI subcommand and the Action step id
- `anvil-mcp-http` — a rejected separate-binary name, kept in the `AGENT-MCP-EXPANSION.4a` design record
- `anvil-mcp` — the MCP server binary (`src/bin/anvil_mcp.rs`)
- `anvil-negctl` — the `CARGO-TMPDIR-SWEEP-REGRESSION.1` negative-control fixture token
- `anvil-proves-it` — English prose
- `anvil-reset-mem-probe.sv` — a one-off probe *file*, not a bank directory
- `anvil-sandbox` — the on-volume working directory `.cache/anvil-sandbox` (`src/paths.rs`)
- `anvil-signoff-knob-sweep-check` — a re-run OUTPUT directory in a Knowledge Map `reverify` command — a future path, not a claim
- `anvil-sv-version-gate-check` — a re-run OUTPUT directory in a Knowledge Map `reverify` command — a future path, not a claim
- `anvil-validate-3f1c` — an elided sandbox path in a book API example — illustrative, decision `0030` class (d)
- `anvil-validate-` — the `anvil-validate-<run_id>` placeholder form used in prose
- `anvil-validate-d8420426e78b2d05` — one real `validate` sandbox name quoted in a verification log
- `anvil-validate` — the `validate` sandbox directory prefix (`downstream::prepare_dut_sandbox`)
- `anvil-version` — a GitHub Action input name (`action.yml`)
