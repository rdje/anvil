# README-POLICY-PROVENANCE: ANVIL's copy is authoritative, and every citation of its authority must resolve

## Metadata

- Tree ID: `README-POLICY-PROVENANCE`
- Status: `done`
- Roadmap lane: Live-doc hygiene / doctrine provenance
- Created: `2026-07-31`
- Last updated: `2026-07-31` (`.1` **done** — **tree CLOSED**)
- Owner: repo-local workflow

## Goal

Two things, both about **where the README policy's authority lives**.

**1. The owner ruling (`2026-07-31`).** Asked whether ANVIL should track the
external template at `/Volumes/SSD/Documents/github/fsmgen/README_POLICY.md`, the
owner answered: *"let ANVIL keep and follow its own copy of the README policy."*
So the repo-local `README_POLICY.md` is **authoritative**, and the fsmgen file is
the **template of origin, not an upstream**. Nothing syncs from it; a later edit
there has no standing here. This is exactly what the policy's own *"Storage
location"* clause asks for — the project-owned copy must not be replaced by an
external one — but ANVIL's copy did not say so explicitly, which left a future
session free to "helpfully" re-sync from a file the owner has now ruled out of
scope.

**2. The citation defect the ruling exposed.** Measured at `61eac73`: **seven
live sites** cite `CLAUDE.md` **§14** as the provenance of the `README-GROWTH`
doctrine — including the doctrine **registry line itself** and two check scripts.
The tracked `CLAUDE.md` is **3 lines long, has one heading, and contains no
numbered sections at all**:

| site | kind |
| --- | --- |
| `scripts/check_doctrines.sh` | the `README-GROWTH` **registry line** |
| `scripts/check_readme_growth.sh` | the check's own header comment |
| `README_POLICY.md` | the adoption note |
| `DOCTRINE_ENFORCEMENT.md` | §10 live-instance table |
| `book/src/architecture.md` | the doctrine list |
| `docs/knowledge/doctrine-enforcement.md` | the Knowledge Map fact card |
| `docs/TASK_TREE.md` | the `README-POLICY-ADOPTION` row |

The directive is real — it is an owner instruction delivered through the agent
session bootstrap — but it is **not in the tracked file those citations name**.
Sweeping from the authoritative set rather than from the reported instance
(decision `0033` rule (2)) widened the class beyond `§14`: the same shape appears
as **`§13`** (volume locality, `docs/TASK_TREE.md`) and **`§3`**, for **23**
citations across **13** files in total — **8** of them live and correctable, the
rest history that keeps its citation raw.
So a reader who opens `CLAUDE.md` to check what authorises a registered doctrine
finds a four-line bootstrap pointer and nothing else.

This is the same class this lane has been clearing all session, at the provenance
layer: **a live doc naming a referent that does not hold.** It is not severe — no
gate is weakened, and the doctrine is correctly enforced — but a doctrine whose
stated authority does not resolve is one step from `DOCTRINE_ENFORCEMENT.md`
§11's *"trust me"* rule, which is the thing that standard exists to forbid.

**3. Harness neutrality — the second, independent reason the citation is wrong**
(owner, `2026-07-31`: *"I will amend the readme policy upstream to mention that it
shall be harness neutral too"*). `CLAUDE.md` is **one vendor's** agent bootstrap
file. ANVIL ships **five** of them, measured at `61eac73` — `CLAUDE.md` (511 B),
`AGENTS.md` (510 B), `GEMINI.md` (511 B), `.cursorrules` and `.windsurfrules`
(491 B, byte-identical to each other) — four distinct contents, all saying the
same thing to different harnesses. The repo's bootstrap layer is therefore
**already harness-neutral by construction**; `MEMORY_ARCHITECTURE.md` and
`DOCTRINE_ENFORCEMENT.md` both state the property explicitly, and enforcement is
git-level precisely so it fires identically whoever made the commit.

Anchoring a repo-wide doctrine's provenance to `CLAUDE.md` contradicts that in
one line: it implies the rule holds because *Claude* was told, when in fact it
holds because the **owner** set it and the **git hook + CI** enforce it against
any author. So the correction is not cosmetic tidying — it is the same
harness-neutrality property the owner is now writing into the upstream policy,
applied to ANVIL's own citations. The replacement referent names the owner, the
date, and the two repo-side durable homes, and names **no harness**.

## Non-Goals

- **Not "add a §14 to `CLAUDE.md`."** That would put the policy text in a second
  place and make it a shadow of `README_POLICY.md` — decision `0033` rule (a),
  and the very failure `README_POLICY.md` exists to prevent. `CLAUDE.md` is a
  deliberate 3-line bootstrap pointer and stays one.
- **Not a history rewrite.** `CHANGES.md`, `DEVELOPMENT_NOTES.md` and
  `docs/tasks/*` also carry the `§14` citation. They are append-only / layer-B
  history and **keep it raw** (decision `0031`) — the citation was accurate to how
  the directive was delivered on `2026-07-30`, and history records what was
  believed then. Only **live** docs are corrected.
- **Not a new mechanism.** There is no gate here. A "every `§`-citation resolves"
  check would have to parse arbitrary prose references and would cry wolf; per
  decision `0033` (c) this class is discovered by review, and the review just
  happened.

## Acceptance Criteria

- `README_POLICY.md` states plainly that ANVIL's copy is authoritative and the
  fsmgen file is the template of origin, not an upstream to sync from.
- Every **live** citation names a referent that exists: the owner directive with
  its date and delivery channel, plus the repo-side durable homes
  (`README_POLICY.md`, decision `0036`).
- Append-only and layer-B history is **untouched**, and the record says so.
- Decision `0036` gains a **dated amendment**, not an edit.
- `scripts/check_doctrines.sh` 8/8; `mdbook build` clean; Knowledge Map in sync.
- Docs + comments only ⇒ DUT byte-identical.

## Task Tree

- ID: `README-POLICY-PROVENANCE`
  Status: `done`
  Goal: `Pin the README policy's provenance: ANVIL's copy is authoritative, and every live citation of its authority resolves to something that exists.`
  Children: `.1` (record the ruling + correct the live citations)

- ID: `README-POLICY-PROVENANCE.1`
  Status: `done`
  Goal: `Record the owner's 2026-07-31 ruling in README_POLICY.md and amend decision 0036 with a dated note; sweep from the authoritative set (every harness-file section citation, not just the reported §14) and correct every LIVE one to a resolving referent; leave append-only and layer-B history raw.`
  Acceptance: `The tracked CLAUDE.md is confirmed to have no numbered sections before anything is claimed about it; the seven live sites are corrected and re-greped to zero; the history sites are re-greped and confirmed UNCHANGED, since leaving them alone is a deliberate act that must be provable, not an omission; decision 0036 amended by appending a dated note; KNOWLEDGE_MAP.md regenerated from the corrected card; check_doctrines.sh 8/8 after git add; mdbook build clean.`
  Verification: `done — VERIFIED THE PREMISE FIRST, before claiming anything about it: the tracked CLAUDE.md is 3 lines, one "# Claude Bootstrap" heading, and a grep for a numbered-section heading returns 0. THE CLASS WAS WIDER THAN THE REPORTED INSTANCE (decision 0033 rule (2) again — search from the authoritative set, here every harness-file section citation, not the one that surfaced): a full sweep found not only §14 but §13 and §3, across 23 citations in 13 files. CORRECTED, LIVE DOCS ONLY (8 sites): scripts/check_doctrines.sh (the README-GROWTH REGISTRY LINE itself), scripts/check_readme_growth.sh, README_POLICY.md, DOCTRINE_ENFORCEMENT.md §10, book/src/architecture.md, docs/knowledge/doctrine-enforcement.md, and two docs/TASK_TREE.md rows (§14 README policy + §13 volume locality). Re-greped: ZERO live citations remain. HISTORY DELIBERATELY UNTOUCHED AND PROVEN SO, because leaving it alone is an act that must be provable rather than an omission: CHANGES.md still carries 4, docs/tasks/COVERAGE-STEERED-GENERATION.md 4, docs/tasks/README-POLICY-ADOPTION.md 4, decision 0031 2, decision 0036 3 — all keep theirs raw under decision 0031, since the citation was accurate to how the directive was delivered on 2026-07-30. Decision 0036 gained a DATED AMENDMENT (not an edit) recording the owner ruling plus the citation policy. ALSO CORRECTED, same subject: DOCTRINE_ENFORCEMENT.md §8 group C said to keep the harness bootstrap files "byte-identical" — measured, ANVIL's five differ only in their title line (511/510/511/491/491 bytes, four distinct contents, bodies identical), so the standard now says keep the BODY identical and adds the rule that provenance is cited by owner+date, never by one of these files. THE HARNESS-NEUTRALITY ARGUMENT IS INDEPENDENT OF THE BROKEN REFERENT and is the stronger of the two: even if CLAUDE.md did have a §14, citing it would imply a repo-wide doctrine holds because one vendor's agent was told, when DOCTRINE_ENFORCEMENT.md §7 makes enforcement git-level precisely so it binds every author. Separately re-verified that the ADOPTION ITSELF is complete and sound and needs no re-sync: README-GROWTH registered and enforcing (README.md 159 lines / 10375 bytes vs caps 250 / 12288), README_POLICY.md present and tracked, and 0 of 227 origin-template tokens absent from ANVIL's copy. Checks: check_doctrines.sh 8/8 after git add; mdbook build clean; check_knowledge_map.sh in sync. Docs + two script comments => DUT byte-identical.`
  Commit: `1c9b865` — `README-POLICY-PROVENANCE.1 — cite the owner, not the harness (tree CLOSED)`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `README-POLICY-PROVENANCE.1` | `done` | **Tree complete.** One leaf: the ruling and the citations are the same subject — where the policy's authority lives — and correcting citations without recording the ruling would leave the next session re-deriving both. |

## Decisions

- `2026-07-31` (owner): **ANVIL keeps and follows its own copy.** The fsmgen file
  is the template of origin. It is not an upstream, nothing syncs from it, and no
  mechanism watches it. Recorded because the *absence* of a sync is a deliberate
  choice that looks exactly like an oversight.
- `2026-07-31`: the citations are **corrected, not deleted**. The provenance is
  worth keeping — this doctrine exists because the owner asked for it, and that is
  a fact about why the repo is shaped this way. What changes is that the citation
  names a referent that resolves.

## Blockers

- None.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-07-31` | `README-POLICY-PROVENANCE` | `measured at 61eac73: the tracked CLAUDE.md is 3 lines with a single "# Claude Bootstrap" heading and ZERO numbered sections (grep -cE for a numbered-section heading returns 0), while 7 LIVE sites cite CLAUDE.md §14 as the provenance of the README-GROWTH doctrine — scripts/check_doctrines.sh (the registry line itself), scripts/check_readme_growth.sh, README_POLICY.md, DOCTRINE_ENFORCEMENT.md, book/src/architecture.md, docs/knowledge/doctrine-enforcement.md and docs/TASK_TREE.md. A further 4 sites carry the same citation in append-only or layer-B history (CHANGES.md, docs/tasks/COVERAGE-STEERED-GENERATION.md, docs/tasks/README-POLICY-ADOPTION.md, decision 0036) plus the generated KNOWLEDGE_MAP.md. Separately verified that the adoption itself is COMPLETE and sound: README-GROWTH is registered and enforcing (README.md 159 lines / 10375 bytes against caps 250 / 12288), README_POLICY.md is present and git-tracked, and a token-level comparison of the fsmgen source policy against ANVIL's copy shows 0 of 227 source tokens missing — the local copy is a superset, so nothing was lost in adoption and no re-sync is needed` | `defect confirmed, pre-existing, live (provenance only — no gate is weakened)` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `README-POLICY-PROVENANCE.1` | `README-POLICY-PROVENANCE.1 — cite the owner, not the harness` | Owner ruling recorded (`0036` dated amendment); 8 live citations corrected to owner+date, history kept raw and proven so; the standard's group-C identity clause corrected to body-identical. |

## Changelog

- `2026-07-31`: Created from an owner ruling that confirmed the status quo. The
  tree exists because confirming a policy raised the question *"where is that
  policy's authority written down?"* — and the answer, measured, was **a section
  number in a file that has no sections**. Worth recording that the finding came
  from being asked to double-check something already believed correct: the
  adoption *was* correct and complete; only its cited provenance was not.
