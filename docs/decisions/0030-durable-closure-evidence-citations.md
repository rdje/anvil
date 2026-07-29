---
id: durable-closure-evidence-citations
title: Closure evidence is cited through committed digests; bare /tmp paths are historical breadcrumbs
answers:
  - "how is a closure claim re-verified when its banked artifact is gone"
  - "what must an ANVIL closure claim cite"
  - "are /tmp evidence paths durable"
  - "where do ANVIL evidence digests live"
  - "what is a historical evidence breadcrumb"
  - "do banked tool_matrix reports survive reboot"
  - "why does a cited /tmp/anvil path not exist"
  - "what goes in docs/evidence"
  - "how do I bank a new evidence digest"
  - "can I add a bank to the grandfathered list"
  - "what is the EVIDENCE-CITATIONS doctrine"
  - "why is the grandfathered evidence list frozen"
  - "where should a tool_matrix bank be written"
date: 2026-07-29
status: current
tags: [evidence, durability, docs, doctrine, process]
evidence: docs/tasks/EVIDENCE-BANK-DURABILITY.md; docs/evidence/README.md; docs/evidence/INVENTORY.md; CHANGES.md
reverify: "bash scripts/check_evidence_citations.sh   (every cited bank is digest-backed under docs/evidence/ or classified in its frozen inventory; the 2026-07-30 amendment replaced this ADR's original /tmp-path grep, which VOLUME-DATA-LOCALITY.5 rendered moot by stripping every /tmp/ prefix)"
---

# 0030 - Closure evidence is cited through committed digests; bare `/tmp` paths are historical breadcrumbs

- Date: 2026-07-29
- Status: accepted
- Tags: evidence, durability, docs, doctrine, process
- Owning leaf: `EVIDENCE-BANK-DURABILITY.2`

## Context

Every numbered-phase exit and every opt-in surface gate is justified by a
*banked artifact* (a `tool_matrix_report.json` or a parity corpus) cited
by an absolute path under volatile `/tmp`. At the `2026-07-25` session
bootstrap, **every one of those artifacts was gone** (`ls -d /tmp/anvil-*`
→ 0 directories; `target/tmp` → 0 bytes) while **77 raw `/tmp/anvil-*`
citation strings** (73 canonical banks; 4 are trailing-punctuation or
line-wrap variants) remained cited across `README.md`, `ROADMAP.md`,
`USER_GUIDE.md`, `CODEBASE_ANALYSIS.md`, `TOOLBOX.md`, `book/src/*.md`,
and `docs/tasks/*.md`.

The audit for this decision classified the 73 canonical banks:

- **(a) 10 phase-closing banks** — `-phase1-real-r21`, `-phase2-share-r1`,
  `-phase3-structured-r4`, `-phase4-hierarchy-r87`, `-phase5-p1`,
  `-phase5b-p1`, `-phase6-p1`, `-phase6-fsm-p1`,
  `-microdesign-parity-phase7-yosys-p1`, `-frontend-parity-phase8-yosys-p1`.
- **(b) 15 repo-owned surface / sweep gate banks** — the structured-surface
  gates (`-function-emit-gate-r1` … `-casez-mux-if-gate-r1`,
  `-mo-k3-gate-r1`, `-generate-loop-gate-8b`), `-sv-version-gate-r1`,
  `-sv-version-gate-upopt-r1`, `-signoff-knob-sweep-r1`,
  `-signoff-surface-iverilog-r1`, `-signoff-surface-nflop-r1`.
- **(c) 47 focused smokes / probes / e2e verification banks** — the
  `-hier-*-smoke-*` family, the historical Phase 4 root-cause and
  coverage-check runs (`-r7`, `-r22`, `-recursive-*-r3x`, `*-check`), and
  the per-surface forced sweeps / probes (`-gl-r1`, `-te-r1`, `-fe-r2`,
  `-cf-sweep`, `-mo-sweep`, `-muxif-genproof.*`, `-ifelse-probe.*`,
  `-probe-se4`, `-widelane-probe`, `-seq-bank`, `-diff-sim-p1`,
  `-multi-clock-p2`, `-divergence-col-smoke`, `-iverilog-compile-smoke-r2`,
  `-reset-mem-probe.sv`, …).
- **(d) 1 illustrative string** — `/tmp/anvil-validate-3f1c…` is a sample
  sandbox path inside a book API-reference example, not an evidence
  citation.

This is **not** a finding that any closure was wrong: per
`DOCTRINE_ENFORCEMENT.md` §3 every gate remains a re-runnable oracle, and
the recorded numbers were real when banked. The defect is in the
**evidence architecture**: the cited artifact fails all four durability
properties of `MEMORY_ARCHITECTURE.md` §2 (not in-repo, not git-tracked,
not reachable from any entrypoint), so re-verifying a closure means
re-deriving it — the archaeology the Knowledge Map exists to eliminate.
It is also a partial regression against decision
[`0002`](0002-live-doc-path-portability.md), which fixed repo paths but
explicitly allowed absolute `/tmp` evidence paths.

## Decision

**The one durable-evidence mechanism is the committed per-bank digest.**

1. **Going forward, a closure or gate claim cites a digest file under
   `docs/evidence/`, not a `/tmp` output path.** A digest is one small
   tracked markdown file per bank, named after the bank's basename
   (`docs/evidence/<bank-basename>.md`), containing:
   - the owning task-tree leaf and the claim it backs;
   - the git commit hash the run executed at;
   - the **exact re-runnable command** (the oracle leg — mechanism (i) is
     thereby embedded in the digest, not rejected);
   - scenario/unit counts, per-tool pass/fail, `coverage_gaps`, and any
     load-bearing coverage facts;
   - the SHA-256 of the source `tool_matrix_report.json` (or parity
     summary), and the run date.

   A few KB each: greppable, diffable, clone-portable, and enough to
   detect a regression by comparing a fresh re-run against the recorded
   numbers without storing the corpus. The `/tmp` output path may appear
   inside the digest as a convenience breadcrumb only.

2. **Retroactively, every pre-0030 `/tmp/anvil-*` citation is a labelled
   historical breadcrumb.** The artifacts are gone; a digest cannot be
   written after the fact without re-running, and a re-run on today's
   code produces *new* evidence, not the cited artifact — fabricating a
   digest for an old claim would violate the no-aspirational-claims rule
   (`COMMIT.md`). For those claims the re-verification path is the named
   re-runnable gate command at the recorded commit (oracle re-run), and
   the breadcrumb label says exactly that.

3. **The contract is mechanically checkable** (structural archetype,
   `DOCTRINE_ENFORCEMENT.md` §3): a `scripts/check_evidence_citations.sh`
   registered in the `scripts/check_doctrines.sh` driver fails when a
   `/tmp/anvil-*` path cited in the live-doc set is neither in the frozen
   grandfathered list (the 77 raw strings above, tracked under
   `docs/evidence/`) nor matched by a digest file
   `docs/evidence/<bank-basename>.md`. New evidence therefore cannot land
   as a bare volatile path.

## Rejected alternatives

- **(i) Re-runnable-command citations alone** — oracle-grade and zero
  storage, but every audit pays full re-derivation (a Phase 4 bank is 840
  designs), and a re-run on current code re-verifies *today's* generator,
  not the recorded numbers. The commands were already documented
  throughout the live docs and did not prevent this failure. Kept, but
  *inside* the digest, not as the citation form.
- **(iii) Relocating banks to a stable non-`/tmp` path** (e.g. a
  gitignored `.cache/evidence/`) — survives reboots on one machine only;
  still fails all four durability properties (untracked, invisible to a
  clone, lost with the machine). Also solves the wrong problem: the
  corpus is bulk; the *claim-bearing numbers* are what must survive.
- **(iv) Breadcrumb demotion alone** — honest about the past but abandons
  durability for future banks, leaving re-derivation as the only audit
  path forever. Adopted only as the retroactive rule (Decision point 2),
  not as the forward mechanism.
- **Committing the corpora** — explicitly out of scope (task-tree
  non-goal): multi-hundred-MB banks do not belong in git.

## Consequences

- `docs/evidence/` becomes a tracked layer beside `docs/decisions/`:
  small, per-bank, derived-from-report digest files; the citation target
  for every future closure claim.
- Implementation leaf shape (owned by `EVIDENCE-BANK-DURABILITY`):
  - **`.3`** — mechanize the contract: `docs/evidence/README.md` (digest
    schema), `scripts/evidence_digest.sh` (derive a digest from a
    `tool_matrix_report.json`), `scripts/check_evidence_citations.sh` +
    driver registry line + the frozen grandfathered list.
  - **`.4`** — the live-doc sweep: label the pre-0030 citations as
    historical breadcrumbs (a normative note per affected top-level live
    doc and book chapter; `docs/tasks/*.md` covered by one note in
    `docs/TASK_TREE.md`, since task files are layer-B history).
  - **`.5` (deferred, opt-in)** — bank a digest opportunistically the
    next time a gate actually re-runs. Explicitly **not** a mass re-run
    of historical gates.
- Decision [`0002`](0002-live-doc-path-portability.md)'s allowance for
  absolute `/tmp` evidence paths is narrowed by this record: still legal
  inside a digest as a breadcrumb, no longer legal as the citation
  itself.

## Links

- Task tree: `docs/tasks/EVIDENCE-BANK-DURABILITY.md`
- Related decisions: [`0002`](0002-live-doc-path-portability.md) (path
  portability), [`0026`](0026-doctrine-enforcement-adoption.md) (check
  registry the `.3` leaf extends)
- Standards: `MEMORY_ARCHITECTURE.md` §2 (durability properties),
  `DOCTRINE_ENFORCEMENT.md` §3 (oracle vs evidence archetypes),
  `knowledge-map/KNOWLEDGE_MAP_ARCHITECTURE.md` §0 (no archaeology)

---

## Amendment — `2026-07-30` (`EVIDENCE-BANK-DURABILITY.3`)

*The original Context / Decision / Rejected-alternatives text above is
unchanged (`MEMORY_ARCHITECTURE.md` §10: supersede, do not mutate). This
amendment records what mechanization found, and narrows exactly one clause.*

**What changed under the decision.** `0030` point 3 specified that the check key
on a bare **`/tmp/anvil-*`** path. Between this ADR and its implementation,
`VOLUME-DATA-LOCALITY.5` swept every `/tmp/` prefix out of the live docs, so
that discriminator no longer exists anywhere in the tree. What a document cites
today is a bare **`anvil-<name>`** token — a shape ANVIL also uses for binaries
(`anvil-mcp`), directories (`anvil-sandbox`), Action inputs (`anvil-bin`),
negative-control fixtures, and English prose (`anvil-emitted`). There is no
lexical rule separating `anvil-cf-sweep` (a bank) from `anvil-hunt-bundles`
(not one).

**The mechanism is unchanged: the committed per-bank digest.** Only the
*recognition* rule is restated, and it is restated as classification rather than
pattern-matching, because a check that must guess which prose tokens are claims
is the wrong archetype — it can only fail by ignoring a real bank or by crying
wolf on prose, and a gate that cries wolf gets disabled.

1. **Three buckets, fail-closed.** Every `anvil-<name>` token in scope is
   classified exactly once: **digest-backed** (`docs/evidence/<token>.md`
   exists and is schema-valid — the forward path, unbounded), **grandfathered**
   (`docs/evidence/INVENTORY.md` §1), or **not evidence** (§2). An unclassified
   token is a breach.

2. **§1 is frozen; §2 is not.** The check pins §1's entry count *and* the
   SHA-256 of its sorted membership. The asymmetry is **semantic, not
   convenience**: §1 is the set of banks that existed before this ADR — a
   historical fact that cannot grow, since you cannot retroactively acquire
   pre-`0030` evidence. Left unpinned it would be the obvious escape hatch
   ("just grandfather it") and the doctrine would be decorative. §2 is a living
   vocabulary and legitimately grows under review.

3. **Scan set by exclusion, not enumeration.** Every tracked `*.md` except the
   append-only history (`CHANGES.md`, `DEVELOPMENT_NOTES.md` — decision `0031`
   forbids retro-editing them), the generated `KNOWLEDGE_MAP.md`, and
   `docs/evidence/` itself (which must name the tokens it classifies — the
   policy-document principle `NO-BOOT-VOLUME-REFS` already uses). This replaces
   `0030`'s hand-listed set: an enumerated list goes stale silently, which is
   precisely how the `/tmp` citations survived `LIVE-DOC-PATH-HYGIENE.1`.

4. **Banks are written under `.cache/anvil-sandbox/<bank>/`** — on-volume
   (decision `0031`), gitignored. The digest is the only tracked artifact.

**Classification cross-check.** The inventory derived from today's tree yields
**10 phase-closing / 15 surface-gate / 47 focused-smoke** grandfathered banks —
identical to this ADR's own `(a) 10 / (b) 15 / (c) 47` audit, reached
independently over a *broader* scan set. The count differs, 72 here vs the
ADR's 73 canonical banks, only in variant handling (line-wrap and
trailing-punctuation fragments are separate tokens to a mechanical check).

**`.4` is narrowed by this amendment.** `0030` scheduled a per-document sweep
labelling pre-`0030` citations as historical breadcrumbs. That assumed a visible
`/tmp/` prefix to label; with the prefix gone there is nothing in the prose to
mark, and 73 in-line labels would be noise. The breadcrumb record now has one
durable home — `INVENTORY.md` §1, which states per bank that the artifact is
gone and that the re-verification path is the named gate command at the recorded
commit. `.4` therefore reduces to a normative pointer from the affected live
docs to that file.
