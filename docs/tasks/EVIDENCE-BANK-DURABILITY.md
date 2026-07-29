# EVIDENCE-BANK-DURABILITY: banked closure artifacts live in volatile `/tmp`

## Metadata

- Tree ID: `EVIDENCE-BANK-DURABILITY`
- Status: `active`
- Roadmap lane: Quality / evidence architecture (cross-cutting; no phase reopened)
- Created: `2026-07-25`
- Last updated: `2026-07-29`
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
  Children: `EVIDENCE-BANK-DURABILITY.1`, `EVIDENCE-BANK-DURABILITY.2`,
        `EVIDENCE-BANK-DURABILITY.3`, `EVIDENCE-BANK-DURABILITY.4`,
        `EVIDENCE-BANK-DURABILITY.5`

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
  Status: `done`
  Goal: design-first ADR — audit the 77 cited paths, classify each as
        (a) phase-closing, (b) surface-gate, or (c) focused smoke; then
        pick ONE durable-evidence mechanism and record it as a decision
        record. Candidate mechanisms weighed: (i) re-runnable-command
        citations; (ii) committed per-bank digests under `docs/evidence/`;
        (iii) a stable gitignored non-`/tmp` location; (iv) explicit
        breadcrumb demotion.
  Acceptance: a `docs/decisions/00NN-*.md` record with Context / Decision /
        Consequences, naming the chosen mechanism, the rejected
        alternatives and why, and the leaf shape for implementation.
  Verification: decision `0030`
        (`docs/decisions/0030-durable-closure-evidence-citations.md`)
        landed: the chosen mechanism is the **committed per-bank digest**
        under `docs/evidence/` (embedding the exact re-runnable command,
        so (i) is inside the digest); pre-0030 citations are demoted to
        labelled historical breadcrumbs ((iv) applied retroactively);
        (iii) rejected (fails all four `MEMORY_ARCHITECTURE.md` §2
        durability properties). Audit result: 77 raw strings = 73
        canonical banks (4 punctuation/line-wrap variants) = (a) 10
        phase-closing + (b) 15 repo-owned surface/sweep gate banks +
        (c) 47 focused smokes/probes/e2e banks + (d) 1 illustrative
        API-example string (`/tmp/anvil-validate-3f1c…`, not evidence);
        full classified list in the section below.
  Commit: `EVIDENCE-BANK-DURABILITY.2 — ADR 0030: durable closure-evidence citations`

- ID: `EVIDENCE-BANK-DURABILITY.3`
  Status: `done`
  Goal: mechanize the 0030 contract: `docs/evidence/README.md` (digest
        schema), `scripts/evidence_digest.sh` (derive a digest from a
        `tool_matrix_report.json`), `scripts/check_evidence_citations.sh`
        + `check_doctrines.sh` registry line + the frozen grandfathered
        list under `docs/evidence/`.
        **Recognition rule restated (0030 amendment `2026-07-30`):** the ADR
        keyed on a bare `/tmp/anvil-*` path, but `VOLUME-DATA-LOCALITY.5`
        removed every `/tmp/` prefix, so the citation form is now a bare
        `anvil-<name>` token — a shape ANVIL also uses for binaries, dirs,
        Action inputs and prose. The check therefore **classifies** rather
        than pattern-matches, and fails closed.
  Acceptance: driver reports the new check PASS on the current tree; a
        synthetic new uncited bank makes it FAIL; widening the frozen
        grandfathered list makes it FAIL; a malformed digest makes it FAIL;
        a schema-valid digest admits the citation.
  Verification: Driver **6/6 PASS** (`EVIDENCE-CITATIONS` registered).
        Four negative controls, each applied then reverted:
        **NC1** a new `anvil-brandnew-gate-r9` citation in `README.md` with no
        digest → **exit 1**; **NC2** appending that name to the frozen
        `INVENTORY.md` §1 — the "just grandfather it" escape hatch → **exit 1**
        (count + SHA-256 pin); **NC3** a digest missing required fields →
        **exit 1**; **NC4** a schema-valid digest → **exit 0**, the citation
        admitted. Baseline and post-cleanup both exit 0.
        Classification cross-check: the inventory derived from today's tree
        yields **10 / 15 / 47** phase-closing / surface-gate / focused-smoke
        grandfathered banks — **identical to decision `0030`'s own audit**,
        reached independently over a broader scan set (72 vs 73 canonical
        differs only in variant handling, which a mechanical check counts
        separately). Docs + shell only, no `src/`/`tests/` ⇒ DUT byte-identical.
  Commit: `EVIDENCE-BANK-DURABILITY.3 — mechanize decision 0030: the EVIDENCE-CITATIONS doctrine`

- ID: `EVIDENCE-BANK-DURABILITY.4`
  Status: `pending`
  Goal: **RE-SCOPED `2026-07-30`** (0030 amendment). The original goal was a
        per-document sweep labelling pre-0030 `/tmp/anvil-*` citations as
        historical breadcrumbs. That assumed a visible `/tmp/` prefix to
        label; `VOLUME-DATA-LOCALITY.5` removed it, so there is nothing in
        the prose to mark and 73 in-line labels would be noise. The
        breadcrumb record now has one durable home — `docs/evidence/INVENTORY.md`
        §1, which states per bank that the artifact is gone and that
        re-verification is the named gate command at the recorded commit.
        `.4` reduces to: a short normative pointer from each affected
        top-level live doc + book chapter to that file, and one note in
        `docs/TASK_TREE.md` covering `docs/tasks/*.md` (layer-B history).
  Acceptance: a reader who meets a bare `anvil-<bank>` citation in any live
        doc can reach, in one hop, the statement that pre-0030 banks are
        gone and how to re-verify.
  Verification: `pending`
  Commit: `pending`

- ID: `EVIDENCE-BANK-DURABILITY.5`
  Status: `pending` (deferred, opt-in)
  Goal: bank a `docs/evidence/` digest opportunistically the next time a
        gate actually re-runs. Explicitly NOT a mass re-run of historical
        gates (tree non-goal).
  Acceptance: the first post-0030 gate run lands with its digest.
  Verification: `pending`
  Commit: `pending`

## `.2` audit — the 73 canonical banks behind the 77 raw citation strings

Raw list banked at decision time (re-derivable):
`grep -rhoE '/tmp/anvil-[A-Za-z0-9_.-]+' README.md ROADMAP.md USER_GUIDE.md
CODEBASE_ANALYSIS.md TOOLBOX.md book/src/*.md docs/tasks/*.md | sort -u`
→ 77 strings; 4 are variants (`-case-mux-if-gate-r1.`,
`-casez-mux-if-gate-r1.`, `-tool-matrix-phase5b-p1.` trailing-dot forms,
and `-microdesign-parity-phase7-` line-wrap truncation).

- **(a) Phase-closing (10):** `-tool-matrix-phase1-real-r21`,
  `-tool-matrix-phase2-share-r1`, `-tool-matrix-phase3-structured-r4`,
  `-tool-matrix-phase4-hierarchy-r87`, `-tool-matrix-phase5-p1`,
  `-tool-matrix-phase5b-p1`, `-tool-matrix-phase6-p1`,
  `-tool-matrix-phase6-fsm-p1`, `-microdesign-parity-phase7-yosys-p1`,
  `-frontend-parity-phase8-yosys-p1`.
- **(b) Repo-owned surface/sweep gate banks (15):**
  `-function-emit-gate-r1`, `-generate-loop-gate-r1`,
  `-generate-loop-gate-8b`, `-task-emit-gate-r1`, `-cone-function-gate-r1`,
  `-multi-output-task-gate-r1`, `-mo-k3-gate-r1`, `-mux-if-gate-r1`,
  `-case-mux-if-gate-r1`, `-casez-mux-if-gate-r1`, `-sv-version-gate-r1`,
  `-sv-version-gate-upopt-r1`, `-signoff-knob-sweep-r1`,
  `-signoff-surface-iverilog-r1`, `-signoff-surface-nflop-r1`.
- **(c) Focused smokes / probes / e2e verification banks (47):**
  the 16 `-hier-*-smoke-*` + `-hierarchy-smoke-r1` family,
  `-parent-cone-instance-smoke-r1`, the historical Phase 4 root-cause /
  coverage runs (`-tool-matrix-phase4-hierarchy-r7`, `-…-r22`,
  `-…-mixed-helper-check`, `-…-parent-cone-instance-r1`,
  `-…-parent-helper-child-input-mixed-check`,
  `-…-parent-output-helper-state-r3`, `-…-parent-port-coverage-r1`,
  `-…-recursive-direct-helper-r32`, `-…-recursive-helper-state-r31`,
  `-…-registered-mixed-r1`, `-…-registered-multistage-r1`,
  `-…-stateful-helper-child-input-mixed-check`), and the per-surface
  forced sweeps / probes / e2e banks (`-gl-r1`, `-gl8b`, `-te-r1`,
  `-fe-r2`, `-cf-sweep`, `-mo-sweep`, `-muxif-genproof.*`,
  `-ifelse-probe.*`, `-probe-se4`, `-widelane-probe`, `-se9-probe`,
  `-se-motask-probe`, `-seq-bank`, `-diff-sim-p1`, `-multi-clock-p2`,
  `-divergence-col-smoke`, `-iverilog-compile-smoke-r2`,
  `-reset-mem-probe.sv`).
- **(d) Illustrative, not evidence (1):** `-validate-3f1c…` — a sample
  MCP `validate` sandbox path inside a book API-reference example. The
  `.4` sweep must NOT breadcrumb-label it; it is not a claim.

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `EVIDENCE-BANK-DURABILITY.3` | `done` | Mechanized `2026-07-30`; `EVIDENCE-CITATIONS` is live in the driver (6/6). |
| 2 | `EVIDENCE-BANK-DURABILITY.4` | `pending` | **Current frontier.** Re-scoped to a pointer-per-doc now that the inventory carries the breadcrumb record. |
| 3 | `EVIDENCE-BANK-DURABILITY.5` | `pending` | Bank the first real digest from an actual gate re-run — the end-to-end proof that the `.3` chain works on real output rather than on fixtures. |

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

- ~~Digest vs citation-as-command?~~ **Resolved by `.2` / decision `0030`:**
  the committed digest is the mechanism, and it embeds the re-runnable
  command — the two are one artifact, not a choice. Decided autonomously
  (the digest is the only candidate satisfying all four
  `MEMORY_ARCHITECTURE.md` §2 durability properties); the owner can
  supersede `0030` if the repo-size cost is unwanted.
- ~~Re-run historical gates?~~ **Resolved by `.2` / decision `0030`:** out
  of scope as a sweep; `.5` banks digests opportunistically on future
  re-runs only.

## Blockers

- None. `.2` is a docs/decision leaf and needs no tool run.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-07-25` | (tree opened) | `ls -d /tmp/anvil-*` → 0 dirs; `du -sh target/tmp` → 0; `grep -rhoE '/tmp/anvil-[…]'` over the live-doc set → 77 distinct cited paths | observation recorded; tree opened |
| `2026-07-25` | `EVIDENCE-BANK-DURABILITY.1` | `scripts/check_doctrines.sh` 4/4 PASS; `check_memory_architecture.sh` green (`MEMORY.md` 21 lines / cap 60); bootstrap sweep on the unchanged tree under `ram_guard --threshold 90`: `cargo check --all-targets` clean, `cargo test` exit 0 (pipeline 125/0, snapshots 6/6), `clippy -D warnings` clean, `fmt --check` clean | `done` — observation recorded, tree registered; docs-only ⇒ DUT byte-identical |
| `2026-07-30` | `EVIDENCE-BANK-DURABILITY.3` | Driver **6/6 PASS**. 4 negative controls, applied+reverted: new uncited bank → exit 1; widening the frozen §1 (the escape hatch) → exit 1; malformed digest → exit 1; schema-valid digest → exit 0. Classification cross-check **10/15/47 == decision `0030`'s own audit**, reached independently. `mdbook build` n/a (no book change). Docs + shell only ⇒ DUT byte-identical. | `done` — the contract is mechanical; a new bank cannot land uncited |
| `2026-07-29` | `EVIDENCE-BANK-DURABILITY.2` | Re-measured the `.1` observation on the current tree (exact recorded command → 77 raw strings, reproduced); classified 73 canonical banks into (a) 10 / (b) 15 / (c) 47 / (d) 1; decision `0030` written + `INDEX.md` row; `scripts/check_doctrines.sh` 4/4 PASS (docs-only leaf, no tool run needed per tree Blockers) | `done` — ADR landed; mechanism = committed per-bank digest; frontier → `.3` (after the owner-steered mdBook-lanes unit) |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `EVIDENCE-BANK-DURABILITY.1` | `EVIDENCE-BANK-DURABILITY.1 — open tree: closure artifacts live in volatile /tmp` | docs-only; no `src/` touched ⇒ DUT byte-identical |
| `EVIDENCE-BANK-DURABILITY.2` | `EVIDENCE-BANK-DURABILITY.2 — ADR 0030: durable closure-evidence citations` | docs-only; decision record + tree update ⇒ DUT byte-identical |
| `EVIDENCE-BANK-DURABILITY.3` | `EVIDENCE-BANK-DURABILITY.3 — mechanize decision 0030: the EVIDENCE-CITATIONS doctrine` | `docs/evidence/{README,INVENTORY}.md` + 2 scripts + registry + the `0030` amendment; no generator code ⇒ DUT byte-identical |

## Changelog

- `2026-07-25`: Created task tree from a session-bootstrap observation that
  every cited banked artifact is absent from the machine.
- `2026-07-29`: `.2` done — decision `0030` (committed per-bank digests;
  pre-0030 citations demoted to breadcrumbs; `.3`/`.4`/`.5` implementation
  leaves defined). Frontier paused after `.2` for an owner-steered unit
  (PGEN-reported mdBook lane-chapter gap); resumes at `.3`.
- `2026-07-30`: `.3` done — `EVIDENCE-CITATIONS` is live (driver 6/6). The
  decision the leaf had to make: `0030`'s `/tmp/anvil-*` discriminator no
  longer exists, so the check **classifies** every `anvil-<name>` token into
  digest-backed / grandfathered / not-evidence and fails closed, rather than
  guessing which prose tokens are claims. The frozen §1 is pinned by count and
  membership SHA-256 — without that pin "just grandfather it" would make the
  whole doctrine decorative. Recorded as a dated amendment to `0030`
  (original text untouched: supersede, do not mutate). `.4` narrowed
  accordingly; `.5` promoted from deferred to the real end-to-end proof.
