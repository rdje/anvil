---
id: readme-landing-page-restoration
title: The README is restored to a landing page by **deletion, not relocation** — 92 % of it is a lossy précis of content that already lives in `docs/decisions/`, `docs/evidence/` and `USER_GUIDE.md`; a *generated* CLI reference is rejected because the SCHEMA-DERIVED one already exists
answers:
  - "why is ANVIL's README so long"
  - "where should new CLI documentation go"
  - "should ANVIL generate its CLI reference"
  - "what belongs in the ANVIL README"
  - "what are the README line and byte caps"
  - "where did Current CLI truth go"
  - "how is the README growth guard enforced"
  - "why was a generated CLI reference rejected"
date: 2026-07-30
status: accepted
tags: [docs, readme, policy, shadow-list, de-duplication, doctrine, enforcement, workflow]
evidence: README.md (measured 2026-07-30 at HEAD `fcb9e58`: 1771 lines / 122,767 bytes; per-section table in Context §1); ../fsmgen/README_POLICY.md (the owner-provided policy, CLAUDE.md §14); docs/decisions/0021-knob-ergonomics-presets-and-queryable-catalog.md (the SCHEMA-DERIVED knob catalog that already is the machine-readable CLI truth); docs/decisions/0030-durable-closure-evidence-citations.md (`docs/evidence/`, the home for the banked tallies); docs/decisions/0033-shadow-enumeration-classification.md (the repair ladder this applies); measured duplication probe in Context §3
---

# 0036 - README-POLICY-ADOPTION: restore the landing page by deleting the duplicate

- Date: 2026-07-30
- Status: accepted
- Tree: `README-POLICY-ADOPTION.1` (audit + design leaf; classifies every section, sets
  the caps from what survives, and picks the enforcement mechanism)
- Activated by: owner directive `CLAUDE.md` §14 (*"make sure README.md doesn't grow, grow
  and grow"*), unimplemented since it was issued; and an explicit owner instruction on
  `2026-07-30` to *"take a SOTA, signoff decision"* on the destination question.

## Context

### 1. The measurement

`README.md` at `fcb9e58`: **1771 lines / 122,767 bytes**, against the policy's
illustrative `300` / `16,384`. Per section:

| section | lines | bytes | share |
| --- | ---: | ---: | ---: |
| Project objective (+ the three principles) | 38 | 2,566 | 2.1 % |
| Fast ramp-up (reading order) | 36 | 4,256 | 2.0 % |
| Key project file paths | 55 | 3,126 | 3.1 % |
| **Build and validation commands** | **487** | **28,786** | **27.5 %** |
| **Current CLI truth** | **1141** | **83,243** | **64.4 %** |
| Maintenance rule | 3 | 200 | 0.2 % |
| License | 9 | 149 | 0.5 % |

**Two sections are 92 % of the file.** And the pivotal fact:

> Everything the policy's content contract actually asks for — objective and scope,
> reading order, architecture at a glance, maintenance rule, license — totals
> **141 lines**. The landing page ANVIL needs already exists, and already fits.

This is not a document that must be rewritten. It is a compliant landing page with two
large appendices bolted on.

### 2. `## Current CLI truth` is not a CLI reference

The name says reference; the content is not. Measured across its 1141 lines:

- **0** command or code-fence lines.
- **50** top-level bullets — averaging ~23 lines each.
- **33** lines citing a decision record by number.

It is fifty short design essays. A representative bullet explains why the ninth emission
surface ships a masked `(sel & care) == val` form: *because Yosys 0.64 rejects the concise
`sel ==? pattern` wildcard-equality syntax in both repo modes*. That is engineering
rationale, and rationale is not derivable from `Config`.

### 3. It is a **lossy copy** — measured, not assumed

Each of those bullets cites a decision record. Probe: count distinctive phrases in the
README bullet versus the record and book chapter it points at.

| phrase | README | its decision record | book chapter |
| --- | ---: | ---: | ---: |
| `Yosys 0.64` (the `casez` rejection) | 1 | 3 (`0029`) | — |
| `care_mask` | 2 | 9 (`0029`) | — |
| `__cv` (the `mux_if` output var) | 7 | 15 (`0027`) | 19 |
| `passthrough` | 7 | 9 (`0027`) | 16 |

**Every phrase is better covered where it belongs than in the README.** So this content
does not need *moving*. It needs **deleting**, with a link left behind. That reframing is
the whole decision: the expensive, risky operation (relocate 1141 lines without loss)
turns out to be unnecessary for almost all of it.

`## Build and validation commands` splits differently but just as cleanly: **63**
code-fence/command lines (a genuine quick start plus gate invocations) around **78**
`saw_*` coverage-fact lines and **15** banked-evidence lines — a closure-evidence log
that decision [`0030`](0030-durable-closure-evidence-citations.md) already built
`docs/evidence/` to hold.

## Decision

### (a) Route by kind; the dominant route is **delete-and-link**

| content | destination | operation |
| --- | --- | --- |
| Objective, three principles, reading order, key paths, maintenance rule, license | **stays** | keep |
| One minimal verified quick start | **stays** (the policy requires exactly one) | keep, trimmed |
| The 50 design-rationale bullets | `docs/decisions/` + `book/src/*` — **already there** | **delete**, link to `docs/decisions/INDEX.md` |
| User-facing flag/knob detail | `USER_GUIDE.md` — already the user-facing surface | delete, link |
| Gate invocations (`tool_matrix --…-gate`) | `USER_GUIDE.md` + `TOOLBOX.md` | move |
| `saw_*` fact lists and banked tallies | `docs/evidence/` (decision `0030`) | move |
| Anything with rationale **not** already in a durable layer | its owning decision record or book chapter | **move first, then delete** |

That last row is the audit's real work product and `.2`'s precondition. It is the only
place information can be lost, so `.2` must prove per-bullet coverage before deleting —
by the §3 probe, mechanically, not by reading.

### (b) A **generated** CLI reference is rejected

This is the option this decision was asked to weigh, and the one I leaned toward before
measuring. Rejected, for three independent reasons:

1. **There is nothing to generate.** `## Current CLI truth` contains **zero** command
   lines. Generation would produce a flag table that does not currently exist anywhere in
   the README, while deleting the essays it actually contains — a different change wearing
   this one's name.
2. **The SCHEMA-DERIVED reference already exists.** Decision
   [`0021`](0021-knob-ergonomics-presets-and-queryable-catalog.md) shipped a queryable
   knob catalog plus `anvil://catalog/presets`, and `--dump-config` emits the effective
   knobs as JSON. The machine-readable truth is built, tested and API-exposed. A generated
   Markdown reference would be a **fourth** surface for a problem already solved — and by
   this project's own `feedback_full_factorization` doctrine, a second classifier for one
   job is the thing to avoid.
3. **It would re-import the drift it claims to prevent.** A generator needs a template, a
   build step, and a "regenerate produces no diff" gate — machinery justified when the
   alternative is a hand-maintained list, and unjustified when the alternative is *a link*.

**The correct R1 already shipped; the README's job is to point at it, not to re-host it.**

### (c) Caps derived from what survives: **250 lines / 12,288 bytes**

The policy forbids choosing a cap to accommodate existing content, so the number comes
from the measured survivors: 141 lines today, plus a trimmed quick start (~25) and the
navigation expansion that replaces the deleted sections (~20) ⇒ ~186, rounded to **250**
with modest headroom. Deliberately **below** the policy's illustrative `300` / `16,384`,
because the policy says to leave only modest headroom and 300 would leave 60 %.

### (d) Enforced as the **8th registered doctrine**

`README-GROWTH` in `scripts/check_doctrines.sh`, so the git hook **and** CI both run it —
exactly what the policy's "mechanical growth guard" clause asks for, using the mechanism
`DOCTRINE_ENFORCEMENT.md` already provides rather than a standalone script. Non-mutating,
deterministic, and it fails with a **routing hint** naming the canonical home:

```
[readme-growth] FAIL: README.md is <lines> lines / <bytes> bytes (caps: 250 / 12288).
[readme-growth]   Route the overflow, do not raise the cap:
[readme-growth]     user-facing flags/knobs  -> USER_GUIDE.md
[readme-growth]     design rationale         -> docs/decisions/ (add a record)
[readme-growth]     banked gate evidence     -> docs/evidence/ (decision 0030)
[readme-growth]     concepts and examples    -> book/src/
```

The caps live in the check, which is authoritative; `README_POLICY.md` states them in
prose and is bound to it by an `ENUMERATION-PARITY`-style pair only if a second copy
proves necessary — by default the policy document will **cite** the check rather than
restate the numbers, per decision `0033`'s rule that a number beside a list is one more
copy of it.

### (e) `README_POLICY.md` lands at the repo root

The policy's own storage clause: an external copy may serve as a template but must not
*replace* the project-owned copy. Verbatim, with an ANVIL-specific preamble recording the
adopted caps and pointing at this record.

## Decisive test applied

"Could a first-time visitor do the thing the README exists for, and could a contributor
find what was removed?" Yes on both: the surviving 141 lines already carry objective,
scope, reading order and architecture, and every deleted bullet is replaced by a link to a
layer that documents it **more** fully (§3). The operation removes duplication, not
information — which is why it is safe to do by deletion.

## Rejected alternatives

- **Generate a CLI reference.** Rejected — §(b). Nothing to generate, the SCHEMA-DERIVED
  reference already exists, and it would add a fourth surface for a solved problem.
- **Relocate all 1141 lines into `USER_GUIDE.md`.** Rejected — it would move a *lossy
  copy* next to the fuller originals, making `USER_GUIDE.md` the next file to outgrow its
  purpose. Duplication is the defect; moving it relocates the defect.
- **Raise the cap to fit the current file.** Explicitly forbidden by the policy, and it
  would ratify the growth the directive exists to stop.
- **A cap of 300 / 16,384** (the policy's illustrative numbers). Rejected as too loose
  against a measured 141-line survivor: 300 leaves 60 % headroom, which is room to regrow.
- **A standalone CI-only script.** Rejected — the policy asks for hook *and* CI, and
  `scripts/check_doctrines.sh` already is that mechanism.
- **Deleting the content outright without the §3 coverage probe.** Rejected — the probe is
  cheap and it is the only thing standing between "de-duplication" and "data loss".
- **Applying the caps to other long live docs.** Rejected for now: `CHANGES.md` is
  append-only *by doctrine* (decision `0031`) and must be exempt, and `USER_GUIDE.md`'s
  length is its purpose. The policy is written for a landing page; this adopts it for one.

## Consequences

- `README.md` returns to ~186 lines from 1771 — a **89 % reduction** — with no information
  lost, because the removed content is a lossy copy of three layers that own it.
- The project gains an 8th registered doctrine and, with it, the first mechanical
  guarantee that its entry point stays an entry point.
- A contributor adding a knob stops having a README bullet to write. That is the durable
  win: the growth was structural, not accidental — the file grew because the workflow
  *asked* it to.
- Nothing is retired; no code changes; DUT byte-identical throughout the tree.

## Open questions (to be resolved at `.2` / `.3`)

- Whether the §3 coverage probe should ship as a one-off audit script or as a permanent
  check. Proposed: **one-off** — once the duplicate is deleted there is no second copy to
  drift, so a permanent check would guard a set that no longer exists.
- Whether `TOOLBOX.md` or `USER_GUIDE.md` should own the `tool_matrix --…-gate`
  invocations. `TOOLBOX.md` is described as the catalog of ANVIL's own diagnostic
  instruments, which fits; `USER_GUIDE.md` already documents several. `.2` picks one and
  links from the other.
- Whether the byte cap should be enforced on the rendered file or on tracked bytes
  (identical today; they diverge only if binary content is ever added).

## Tree split

`README-POLICY-ADOPTION` continues as registered:

- **`.1`** (this leaf, audit + design) — decision `0036`: the per-section measurement, the
  route-by-kind table, the rejection of a generated reference, the derived caps, and the
  enforcement mechanism. Docs-only.
- **`.2`** (`pending`) — execute: run the coverage probe per bullet, move the residue that
  is *not* already covered, delete the duplicated sections, land `README_POLICY.md`, and
  expand the navigation section to replace what was removed.
- **`.3`** (`pending`) — the `README-GROWTH` doctrine with the routing hint, registered in
  `DOCTRINE_ENFORCEMENT.md` §10, negative-controlled both ways; close the tree.

## Amendment (`2026-07-31`, owner ruling — `README-POLICY-PROVENANCE.1`)

**ANVIL keeps and follows its own copy of the policy.** Asked whether the repo should track
the external template it was adopted from, the owner ruled: *"let ANVIL keep and follow its
own copy of the README policy."* So `README_POLICY.md` at the repository root is
**authoritative**; the file it was copied from is the **template of origin, not an
upstream**. Nothing syncs from it, no mechanism watches it, and a later edit there has no
standing here. This is what the policy's own *"Storage location"* clause already required —
a project-owned copy that an external one must not replace — but it is recorded explicitly
because **the absence of a sync is a deliberate choice that looks exactly like an
oversight**, and the next session would otherwise be free to "helpfully" re-sync.

Verified at the same time, so the ruling rests on measurement rather than assumption: a
token-level comparison of the origin template against ANVIL's copy finds **0 of 227 source
tokens missing** — the local copy is a superset, adapted with ANVIL's routing table and its
derived caps. Nothing was lost in adoption and there is nothing to re-sync.

**Consequent correction: doctrine provenance is cited by owner + date, never by a harness
bootstrap file.** This record, the `README-GROWTH` registry line, both check scripts and
four other live docs cited the directive as *"`CLAUDE.md` §14"*. Measured `2026-07-31`, the
tracked `CLAUDE.md` is **3 lines with one heading and no numbered sections at all** — the
directive is real, and was delivered through the agent session bootstrap, but it is not in
the file those citations name. Independently of that, the citation is **wrong in kind**:
ANVIL ships **five** harness bootstrap files (`CLAUDE.md`, `AGENTS.md`, `GEMINI.md`,
`.cursorrules`, `.windsurfrules`, body-identical), and `DOCTRINE_ENFORCEMENT.md` §7 makes
enforcement git-level precisely so it binds **every** author. Anchoring a repo-wide
doctrine to one vendor's file implies the rule holds because *that harness* was told.

Live docs are corrected to *"owner directive `2026-07-30`, recorded in `README_POLICY.md`"*.
**This record's own body is not rewritten** — it is history, and the citation was accurate
to how the directive was delivered on `2026-07-30`; the same applies to `CHANGES.md` and the
layer-B task files, which keep theirs raw (decision `0031`). Nothing about the caps, the
audit, or the enforcement in this decision changes.

## Links

- Owner directive: `CLAUDE.md` §14; the explicit `2026-07-30` instruction to take a
  signoff decision on the destination question. Standing directive **DECIDE, DON'T ASK**.
- Source policy: `../fsmgen/README_POLICY.md` (read-only, cross-repo, same volume).
- Doctrine: decision [`0033`](0033-shadow-enumeration-classification.md) (the repair
  ladder — and its rule that a count beside a list is one more copy of it),
  `feedback_full_factorization` (why a second reference surface is rejected), decision
  [`0030`](0030-durable-closure-evidence-citations.md) (`docs/evidence/` owns the banked
  tallies), decision [`0021`](0021-knob-ergonomics-presets-and-queryable-catalog.md) (the
  SCHEMA-DERIVED catalog that makes generation redundant), decision
  [`0031`](0031-ssd-volume-exclusivity.md) (`CHANGES.md` is append-only and exempt).
- Reuse / touch points: `README.md`, `README_POLICY.md` (new), `USER_GUIDE.md`,
  `TOOLBOX.md`, `docs/evidence/`, `scripts/check_doctrines.sh`,
  `DOCTRINE_ENFORCEMENT.md` §10.
