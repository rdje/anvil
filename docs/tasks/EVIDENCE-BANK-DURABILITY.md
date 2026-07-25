# EVIDENCE-BANK-DURABILITY: banked closure artifacts live in volatile `/tmp`

## Metadata

- Tree ID: `EVIDENCE-BANK-DURABILITY`
- Status: `active`
- Roadmap lane: Quality / evidence architecture (cross-cutting; no phase reopened)
- Created: `2026-07-25`
- Last updated: `2026-07-25`
- Owner: repo-local workflow

## Goal

Make the project's **closure evidence** as durable as the claims that cite it.

Every numbered-phase exit and every opt-in surface gate in ANVIL is
justified by a *banked artifact* — a `tool_matrix_report.json` or a parity
corpus — cited by an **absolute path under `/tmp`**. `/tmp` is volatile
(macOS periodically purges it; a reboot clears it), so those artifacts do
not survive the machine, let alone a clone. The claims in `README.md` /
`ROADMAP.md` / `book/src/*.md` / `docs/tasks/*.md` therefore point at paths
that a fresh session, a different machine, or a reviewer cannot open.

The outcome this tree must deliver: a recorded, checkable answer to *"how
is a closure claim re-verified when its banked artifact is gone?"* — either
by making the evidence durable, by making the citation a **re-runnable
command** rather than a path, or by an explicit, documented decision that
re-execution is the only proof and paths are breadcrumbs.

## Observation that opened this tree (`2026-07-25`, session bootstrap)

- `ls -d /tmp/anvil-*` → **0 directories**. Every banked bank is gone.
- `target/tmp` (the `SIGNOFF-SURFACE-EXPANSION.2` Verilator JSON-AST parity
  artifact directory) → **0 bytes**.
- `grep -rhoE '/tmp/anvil-[A-Za-z0-9_.-]+'` across `README.md`,
  `ROADMAP.md`, `USER_GUIDE.md`, `CODEBASE_ANALYSIS.md`, `TOOLBOX.md`,
  `book/src/*.md`, `docs/tasks/*.md` → **77 distinct cited paths**,
  including every phase-closing artifact:
  `/tmp/anvil-tool-matrix-phase1-real-r21`,
  `/tmp/anvil-tool-matrix-phase2-share-r1`,
  `/tmp/anvil-tool-matrix-phase3-structured-r4`,
  `/tmp/anvil-tool-matrix-phase4-hierarchy-r87`,
  `/tmp/anvil-tool-matrix-phase5-p1`, `-phase5b-p1`, `-phase6-p1`,
  `-phase6-fsm-p1`, `/tmp/anvil-microdesign-parity-phase7-yosys-p1`,
  `/tmp/anvil-frontend-parity-phase8-yosys-p1`, plus every structured-surface
  gate bank (`-function-emit-gate-r1`, `-generate-loop-gate-r1`,
  `-task-emit-gate-r1`, `-cone-function-gate-r1`,
  `-multi-output-task-gate-r1`, `-mux-if-gate-r1`, `-case-mux-if-gate-r1`,
  `-casez-mux-if-gate-r1`), `-sv-version-gate-upopt-r1`, and
  `-signoff-knob-sweep-r1`.

**This is not a claim that the closures were wrong.** Per
`DOCTRINE_ENFORCEMENT.md` §3, the strongest archetype is the **oracle
(re-run)**, and every one of these gates is still a re-runnable command
(`cargo run --bin tool_matrix -- --<surface>-gate …`, `cargo test --
--ignored parity_against_real_*`). The defect is in the **evidence
architecture**, not in the generator: the *artifact* a reader is pointed to
no longer exists, so the only way to re-verify is to re-derive — which is
exactly the archaeology `knowledge-map/KNOWLEDGE_MAP_ARCHITECTURE.md` §0
sets out to eliminate.

It is also a partial regression against `LIVE-DOC-PATH-HYGIENE.1`, which
rewrote local **repo** paths to repo-root-relative form but left the `/tmp`
evidence citations absolute and volatile.

## Non-Goals

- **Not** re-opening any closed phase. Phases 1–9 stay `done`; this tree
  changes how their evidence is *cited and re-verified*, never their status.
- **Not** committing multi-hundred-MB corpora into git. A full
  `--phase4-hierarchy-gate` bank is 840 designs; the repo is not the place
  for it.
- **Not** re-running every historical gate to "restore" the banks. Most
  cite a generator version that no longer exists; a re-run produces new
  evidence, not the old artifact.
- **Not** weakening the no-aspirational-claims rule (`COMMIT.md`): a claim
  still needs evidence that existed when it was made.

## Acceptance Criteria

- A decision record states the durable-evidence contract: what a closure
  claim must cite (a re-runnable command, a committed digest, or both), and
  what a bare `/tmp` path means going forward (breadcrumb, not proof).
- Every live-doc closure claim either carries its re-runnable command or is
  explicitly marked as a historical breadcrumb.
- The contract is mechanically checkable where it can be (a
  `check_*.sh` in the `scripts/check_doctrines.sh` registry, per
  `DOCTRINE_ENFORCEMENT.md` §4-§5) — or the tree records why it cannot be.
- Live docs updated; each leaf lands through `COMMIT.md`.

## Task Tree

- ID: `EVIDENCE-BANK-DURABILITY`
  Status: `active`
  Goal: make closure evidence durable / re-derivable, or explicitly
        redefine what a closure citation means.
  Children: `EVIDENCE-BANK-DURABILITY.1`, `EVIDENCE-BANK-DURABILITY.2`

- ID: `EVIDENCE-BANK-DURABILITY.1`
  Status: `done`
  Goal: record the observation and register the tree before any edit —
        the `ROADMAP-FOLLOWUP-OWNERSHIP.1` / `CAPABILITY-LANE-OWNERSHIP.1`
        precedent (a lane is registered as a tracked tree *before*
        implementation resumes).
  Acceptance: the measured observation (the three commands + their
        outputs) is recorded in this file, `CHANGES.md`, and `MEMORY.md`;
        a `docs/TASK_TREE.md` row exists; docs-only ⇒ DUT byte-identical.
  Verification: `ls -d /tmp/anvil-*` → 0 dirs; `du -sh target/tmp` → 0;
        `grep -rhoE '/tmp/anvil-[A-Za-z0-9_.-]+'` over the live-doc set →
        77 distinct paths. `scripts/check_doctrines.sh` 4/4 PASS.
  Commit: this commit.

- ID: `EVIDENCE-BANK-DURABILITY.2`
  Status: `pending`
  Goal: design-first ADR — audit the 77 cited paths, classify each as
        (a) phase-closing, (b) surface-gate, or (c) focused smoke; then
        pick ONE durable-evidence mechanism and record it as a decision
        record. Candidate mechanisms to weigh (each with its cost):
        (i) cite the re-runnable gate command beside every claim
            (cheap, oracle-grade, no storage — but re-derivation is not free);
        (ii) commit a small **digest** per bank (scenario/unit counts,
            `coverage_gaps`, per-tool pass/fail, the report's SHA-256)
            under `docs/evidence/` — a few KB each, greppable, diffable,
            and enough to detect a regression without the corpus;
        (iii) move banks out of `/tmp` into a gitignored but stable
            location (e.g. `.cache/evidence/`) so at least the *current*
            machine keeps them across reboots;
        (iv) accept volatility explicitly and demote every `/tmp` path to
            a labelled breadcrumb.
  Acceptance: a `docs/decisions/00NN-*.md` record with Context / Decision /
        Consequences, naming the chosen mechanism, the rejected
        alternatives and why, and the leaf shape for implementation.
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `EVIDENCE-BANK-DURABILITY.2` | `pending` | Design-first: the fix is a *policy* choice (what a citation must be) before it is an edit. Writing 77 path rewrites before choosing the mechanism would be the expensive wrong order. (`.1` — record + register — is `done`.) |

## Decisions

- `2026-07-25`: Opened as a task tree rather than a silent doc fix, per the
  surfacing directive ("if a result is foundationally surprising, root-cause
  it, flag it, and open a tracked task"). Split `.1` (record + register the
  tree — the `ROADMAP-FOLLOWUP-OWNERSHIP.1` / `CAPABILITY-LANE-OWNERSHIP.1`
  precedent) from `.2` (the design-first ADR), matching the
  `SIGNOFF-AUTOMATION-EXPANSION` / `CAPABILITY-BREADTH-EXPANSION` precedent
  that an open-ended lane must pick one concrete increment before any edit.
- `2026-07-25`: Scoped as **evidence architecture**, not phase status. No
  `ROADMAP.md` phase label moves because of this tree.

## Open Questions

- Does the owner want the digest committed (mechanism (ii), a real but small
  repo-size cost) or the citation-as-command form (mechanism (i), zero
  storage but re-derivation on every audit)? This is a durability-vs-cost
  judgement and is the reason `.2` is design-first — it does not block the
  frontier, it *is* the frontier.
- Should re-running the phase gates to produce fresh, digest-backed banks be
  a separate leaf, or out of scope (the claims are already recorded with the
  numbers they produced)?

## Blockers

- None. `.2` is a docs/decision leaf and needs no tool run.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-07-25` | (tree opened) | `ls -d /tmp/anvil-*` → 0 dirs; `du -sh target/tmp` → 0; `grep -rhoE '/tmp/anvil-[…]'` over the live-doc set → 77 distinct cited paths | observation recorded; tree opened |
| `2026-07-25` | `EVIDENCE-BANK-DURABILITY.1` | `scripts/check_doctrines.sh` 4/4 PASS; `check_memory_architecture.sh` green (`MEMORY.md` 21 lines / cap 60); bootstrap sweep on the unchanged tree under `ram_guard --threshold 90`: `cargo check --all-targets` clean, `cargo test` exit 0 (pipeline 125/0, snapshots 6/6), `clippy -D warnings` clean, `fmt --check` clean | `done` — observation recorded, tree registered; docs-only ⇒ DUT byte-identical |
| `2026-07-25` | `EVIDENCE-BANK-DURABILITY.2` | `pending` | `pending` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `EVIDENCE-BANK-DURABILITY.1` | `EVIDENCE-BANK-DURABILITY.1 — open tree: closure artifacts live in volatile /tmp` | docs-only; no `src/` touched ⇒ DUT byte-identical |
| `EVIDENCE-BANK-DURABILITY.2` | `pending` | `pending` |

## Changelog

- `2026-07-25`: Created task tree from a session-bootstrap observation that
  every cited banked artifact is absent from the machine.
