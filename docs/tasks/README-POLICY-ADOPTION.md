# README-POLICY-ADOPTION: adopt the README Stability Policy and stop the landing page growing

## Metadata

- Tree ID: `README-POLICY-ADOPTION`
- Status: `active`
- Roadmap lane: Workflow / live-doc hygiene — owner-directed README policy
- Created: `2026-07-30`
- Last updated: `2026-07-30` (`.1` audit + design ADR landed — decision `0036`; frontier `.2`)
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
  Status: `done`
  Goal: `Audit + design (docs-only). Classify EVERY section of the current README against the policy's content contract into keep / relocate-to-<file> / delete-as-duplicate, with the line count of each bucket; propose the line and byte caps from what actually remains after the trim (the policy forbids picking a cap to fit existing content); decide where the growth check lives (proposed: a new registered doctrine in scripts/check_doctrines.sh, since the policy explicitly asks for hook+CI enforcement and DOCTRINE_ENFORCEMENT.md is exactly that mechanism); and name the routing hint text the failure prints. Record as a decision record.`
  Acceptance: `A decision record + this tree updated with the per-section classification table and the proposed caps; docs-only.`
  Verification: `done — decision 0036. MEASURED at fcb9e58: 1771 lines / 122767 bytes across 7 sections; TWO sections are 92% of the file (Current CLI truth 1141 lines / 64.4%, Build and validation 487 / 27.5%) while everything the policy's content contract asks for totals 141 LINES — the landing page already exists and already fits. KEY FINDING: `## Current CLI truth` is NOT a CLI reference — ZERO command/code-fence lines, 50 top-level bullets averaging ~23 lines, 33 citing a decision record. A phrase probe proves it is a LOSSY COPY of layers that own the content: `__cv` 7x in README vs 15x in decision 0027 and 19x in book/src/structured-emission.md; `passthrough` 7 vs 9 vs 16; `care_mask` 2 vs 9 in decision 0029; `Yosys 0.64` 1 vs 3. So the operation is DELETE-AND-LINK, not the expensive 1141-line relocation. DECIDED: route by kind (rationale -> already in docs/decisions + book, delete+link; user-facing flags -> USER_GUIDE.md; gate invocations -> USER_GUIDE/TOOLBOX; the 78 saw_* fact lines + 15 banked tallies -> docs/evidence per decision 0030; anything NOT already covered -> move FIRST, then delete, proven per-bullet by the probe); caps 250 lines / 12288 bytes DERIVED from the survivors (141 + trimmed quick start + navigation ~= 186) and deliberately BELOW the policy's illustrative 300/16384 which would leave 60% room to regrow; enforced as the 8th registered doctrine README-GROWTH in scripts/check_doctrines.sh (hook + CI, per the policy's own clause) failing with a routing hint naming the canonical home per kind. A GENERATED CLI reference is REJECTED — the option the owner asked to weigh and the one initially favoured: nothing to generate (zero command lines), the SCHEMA-DERIVED reference already exists (decision 0021's queryable knob catalog + --dump-config), and a generator's template/build-step/no-diff-gate is justified against a hand-maintained list but not against a link. Docs-only ⇒ DUT byte-identical.`
  Commit: `README-POLICY-ADOPTION.1 — audit + design ADR (decision 0036)`

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
| 1 | `README-POLICY-ADOPTION.1` | `done` | Decision `0036`. The measurement reframed the task: the compliant landing page **already exists** (141 lines) with two appendices bolted on, and `## Current CLI truth` is a **lossy copy** of `docs/decisions/` + the book (phrase probe), so the operation is **delete-and-link**, not relocation. A generated CLI reference is rejected — the SCHEMA-DERIVED one already exists. Caps `250` / `12,288`, derived from the survivors. |
| 2 | `README-POLICY-ADOPTION.2` | `pending` | **Next.** Execute: run the phrase probe per bullet (the only place information could be lost), move the residue that is *not* already covered, delete the duplicated sections, land `README_POLICY.md` at the root, and expand the navigation section to replace what was removed. Expected: **1771 → ~186 lines**. |
| 3 | `README-POLICY-ADOPTION.3` | `pending` | The `README-GROWTH` doctrine with the routing hint, registered in `DOCTRINE_ENFORCEMENT.md` §10, negative-controlled both ways; close the tree. A cap is only meaningful once the file is under it. |

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
| `README-POLICY-ADOPTION` | `7a1fc50` — `COVERAGE-STEERED-GENERATION.3c — steering docs + close .3` | Registered (not started) in the docs slice that surfaced it; no README content moved in that commit. |

## Changelog

- `2026-07-30`: Created. Registered the owner-directed `CLAUDE.md` §14 README
  Stability Policy as a task tree so the directive is owned rather than
  remembered. Surfaced while reading the policy to make a one-phrase correction in
  the README's coverage-steering bullet during
  `COVERAGE-STEERED-GENERATION.3c`.
- `2026-07-30`: `.1` audit + design ADR landed (decision
  [`0036`](../decisions/0036-readme-landing-page-restoration.md)), on an explicit owner
  instruction to take a signoff decision on the destination question. **The measurement
  reframed the task.** `README.md` is 1771 lines, but everything the policy's content
  contract asks for totals **141** — the compliant landing page already exists, with two
  appendices bolted on that are 92 % of the file. And `## Current CLI truth` (64 %) is
  **not a CLI reference**: zero command lines, 50 design essays, 33 citing a decision
  record. A phrase probe showed it is a **lossy copy** of the layers that own that content
  (`__cv` appears 7× in the README vs 15× in decision `0027` and 19× in the book). So the
  operation is **delete-and-link**, not the expensive and risky 1141-line relocation the
  tree originally assumed.
- `2026-07-30`: `.1` **rejected the generated CLI reference** — the option the owner asked
  to weigh, and the one I favoured before measuring. Three independent reasons: there is
  nothing to generate (zero command lines in the section); the SCHEMA-DERIVED reference
  **already exists** (decision `0021`'s queryable knob catalog + `--dump-config`), so a
  generated Markdown surface would be a *fourth* copy of solved truth and a
  `feedback_full_factorization` violation; and a generator's template + build step +
  no-diff gate is machinery justified against a hand-maintained list, not against *a
  link*. **The correct R1 already shipped; the README's job is to point at it.** Caps set
  at **250 lines / 12,288 bytes**, derived from the survivors and deliberately below the
  policy's illustrative numbers, which would leave 60 % headroom to regrow into.
