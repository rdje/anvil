# README-POLICY-ADOPTION: adopt the README Stability Policy and stop the landing page growing

## Metadata

- Tree ID: `README-POLICY-ADOPTION`
- Status: `active`
- Roadmap lane: Workflow / live-doc hygiene — owner-directed README policy
- Created: `2026-07-30`
- Last updated: `2026-07-30` (registered; no work started)
- Owner: repo-local workflow

## Goal

Adopt the owner-provided **README Stability Policy** (`CLAUDE.md` §14, source copy
at `../fsmgen/README_POLICY.md`) so `README.md` is a stable landing page rather
than a second changelog, roadmap, and CLI reference — and so it **cannot** grow
back, because a mechanical cap fails the commit.

## Non-Goals

- **No information loss.** Nothing is deleted; every displaced paragraph moves to
  the canonical home the policy names (`USER_GUIDE.md`, the mdBook, `ROADMAP.md`,
  `CHANGES.md`, `docs/decisions/`). This tree is a *relocation*, not a trim.
- **No history rewrite.** `CHANGES.md` / `DEVELOPMENT_NOTES.md` stay append-only
  (decision `0031`); README content that is genuinely historical is moved, not
  edited to look like it was never there.
- No change to any other live doc's contract, and no code change ⇒ DUT
  byte-identical throughout.

## Acceptance Criteria

- `README_POLICY.md` exists at the repository root as the project-owned copy (the
  policy's own "Storage location" clause: an external copy may be a template but
  must not *replace* the repo copy).
- `README.md` satisfies a deliberately-chosen line cap and byte cap with modest
  headroom, and still contains everything the policy's content contract requires:
  purpose/audience/scope, prerequisites + **one** verified quick start, stable
  architecture at a glance, canonical-doc navigation, license/notices.
- A deterministic, non-mutating growth check runs from
  `scripts/check_doctrines.sh` (so it is gated by the git hook **and** CI, per
  `DOCTRINE_ENFORCEMENT.md`), returning non-zero with a routing hint naming the
  canonical home for the content that overflowed.
- Every relocated paragraph is reachable from the README's navigation section —
  measured, not assumed (no orphaned content).

## Task Tree

- ID: `README-POLICY-ADOPTION`
  Status: `active`
  Goal: `Adopt the README Stability Policy: land the repo-owned policy copy, relocate the non-landing-page content to its canonical homes, and gate the result with a mechanical cap.`
  Children: `README-POLICY-ADOPTION.1` (audit + design), `.2` (relocation), `.3` (the gate + close)

- ID: `README-POLICY-ADOPTION.1`
  Status: `pending`
  Goal: `Audit + design (docs-only). Classify EVERY section of the current README against the policy's content contract into keep / relocate-to-<file> / delete-as-duplicate, with the line count of each bucket; propose the line and byte caps from what actually remains after the trim (the policy forbids picking a cap to fit existing content); decide where the growth check lives (proposed: a new registered doctrine in scripts/check_doctrines.sh, since the policy explicitly asks for hook+CI enforcement and DOCTRINE_ENFORCEMENT.md is exactly that mechanism); and name the routing hint text the failure prints. Record as a decision record.`
  Acceptance: `A decision record + this tree updated with the per-section classification table and the proposed caps; docs-only.`
  Verification: `pending`
  Commit: `pending`

- ID: `README-POLICY-ADOPTION.2`
  Status: `pending`
  Goal: `Execute the relocation decided at .1. Move each classified block to its canonical home (USER_GUIDE.md / book/src/*.md / ROADMAP.md / docs/decisions/), leaving a navigation link where the content was. Land README_POLICY.md at the repo root.`
  Acceptance: `README.md within the .1 caps; every relocated block present at its destination and linked from the README; mdbook build clean; cargo test --test book_examples green (README is not book-tested, but relocated examples may become book examples); no code change.`
  Verification: `pending`
  Commit: `pending`

- ID: `README-POLICY-ADOPTION.3`
  Status: `pending`
  Goal: `Land the mechanical growth guard in scripts/check_doctrines.sh with the routing hint, negative-control it BOTH ways (an over-cap README must fail the hook; the real README must pass), register it in DOCTRINE_ENFORCEMENT.md §10, and close the tree.`
  Acceptance: `scripts/check_doctrines.sh reports the new doctrine PASS; a synthetic over-cap README makes it FAIL with the routing hint; ENUMERATION-PARITY stays green (the doctrine registry has several documented shadows of itself — the new entry must appear in every one of them, which that doctrine will enforce).`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `README-POLICY-ADOPTION.1` | `pending` | Audit + design first: the caps must be derived from what survives the trim, and the policy explicitly forbids choosing a cap to accommodate existing content. Nothing can be relocated safely before the per-section classification exists. |
| 2 | `README-POLICY-ADOPTION.2` | `pending` | The relocation itself. |
| 3 | `README-POLICY-ADOPTION.3` | `pending` | The gate, last — a cap is only meaningful once the file is under it. |

## Decisions

- `2026-07-30`: Registered as a task tree before any edit, per the task-tree
  ownership doctrine. The owner directive is in `CLAUDE.md` §14 ("Please adopt
  this new policy regarding README.md, that make sure README.md doesn't grow,
  grow and grow") and has **not** been implemented: measured `2026-07-30`,
  `README.md` is **1771 lines / 122,767 bytes**, against the policy's illustrative
  `300` lines / `16,384` bytes — roughly **6×** the line cap and **7.5×** the byte
  cap — and no `README_POLICY.md` exists at the repo root. The file currently
  carries an exhaustive CLI reference (`## Current CLI truth` alone spans lines
  620–1762, ~64 % of the file), banked evidence tallies, and per-knob design
  rationale: three categories the policy routes to the user guide, the release
  notes, and the decision records respectively.
- `2026-07-30`: Deliberately **not** started inside the slice that discovered it
  (`COVERAGE-STEERED-GENERATION.3c`), which needed a one-phrase correction in the
  README's steering bullet. That edit was made net-neutral in length rather than
  additive, so this tree does not inherit new debt from it.

## Open Questions

- Whether the growth guard becomes an **8th registered doctrine** (proposed —
  the policy asks for hook + CI enforcement, which is precisely what
  `DOCTRINE_ENFORCEMENT.md` provides) or a standalone script wired into CI only.
  The doctrine route costs an entry in every documented copy of the registry, but
  `ENUMERATION-PARITY` already gates those copies, so the cost is mechanical.
- Whether the caps apply to `README.md` alone or to a small set of landing-page
  files. The policy is written for one README; ANVIL also has several long-form
  live docs (`CHANGES.md` is append-only **by doctrine** and must be exempt).
- Whether `## Current CLI truth` moves wholesale into `USER_GUIDE.md` (which
  already documents most of it, so the move is largely a **de-duplication**) or
  into a generated reference. A generated CLI reference would be the stronger
  answer — it cannot drift — but it is a bigger change than the policy requires.

## Blockers

- None. The work is docs-only and depends on nothing in flight.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-07-30` | `README-POLICY-ADOPTION` | `tree registered (docs-only); measured README.md = 1771 lines / 122767 bytes (at HEAD ff506e1, before this session's net-neutral steering-bullet correction); confirmed no repo-root README_POLICY.md; no code touched` | `registered` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `README-POLICY-ADOPTION` | `COVERAGE-STEERED-GENERATION.3c — steering docs + close .3` | Registered (not started) in the docs slice that surfaced it; no README content moved in that commit. |

## Changelog

- `2026-07-30`: Created. Registered the owner-directed `CLAUDE.md` §14 README
  Stability Policy as a task tree so the directive is owned rather than
  remembered. Surfaced while reading the policy to make a one-phrase correction in
  the README's coverage-steering bullet during
  `COVERAGE-STEERED-GENERATION.3c`.
