# README-POLICY-ADOPTION: adopt the README Stability Policy and stop the landing page growing

## Metadata

- Tree ID: `README-POLICY-ADOPTION`
- Status: `closed`
- Roadmap lane: Workflow / live-doc hygiene — owner-directed README policy
- Created: `2026-07-30`
- Last updated: `2026-07-30` (`.3` landed — `README-GROWTH` is the 8th registered doctrine; **tree CLOSED**, all three leaves done)
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
  > **Superseded in mechanism, not in intent, by decision `0036` (`.1`).** The
  > audit measured that the displaced content was already at those canonical
  > homes — more fully than in the README — so the operation became
  > **delete-and-link**. The non-goal itself stands and was met: `.2` proved
  > zero information loss mechanically (879 tokens swept; the only 32
  > uncovered strings are composites of covered parts), and the one thing that
  > genuinely was *not* already covered — runnable invocations for 7 of the 15
  > gates — was **moved** to `USER_GUIDE.md` before anything was cut.
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
  Status: `closed`
  Goal: `Adopt the README Stability Policy: land the repo-owned policy copy, relocate the non-landing-page content to its canonical homes, and gate the result with a mechanical cap.`
  Children: `README-POLICY-ADOPTION.1` (audit + design), `.2` (relocation), `.3` (the gate + close)

- ID: `README-POLICY-ADOPTION.1`
  Status: `done`
  Goal: `Audit + design (docs-only). Classify EVERY section of the current README against the policy's content contract into keep / relocate-to-<file> / delete-as-duplicate, with the line count of each bucket; propose the line and byte caps from what actually remains after the trim (the policy forbids picking a cap to fit existing content); decide where the growth check lives (proposed: a new registered doctrine in scripts/check_doctrines.sh, since the policy explicitly asks for hook+CI enforcement and DOCTRINE_ENFORCEMENT.md is exactly that mechanism); and name the routing hint text the failure prints. Record as a decision record.`
  Acceptance: `A decision record + this tree updated with the per-section classification table and the proposed caps; docs-only.`
  Verification: `done — decision 0036. MEASURED at fcb9e58: 1771 lines / 122767 bytes across 7 sections; TWO sections are 92% of the file (Current CLI truth 1141 lines / 64.4%, Build and validation 487 / 27.5%) while everything the policy's content contract asks for totals 141 LINES — the landing page already exists and already fits. KEY FINDING: `## Current CLI truth` is NOT a CLI reference — ZERO command/code-fence lines, 50 top-level bullets averaging ~23 lines, 33 citing a decision record. A phrase probe proves it is a LOSSY COPY of layers that own the content: `__cv` 7x in README vs 15x in decision 0027 and 19x in book/src/structured-emission.md; `passthrough` 7 vs 9 vs 16; `care_mask` 2 vs 9 in decision 0029; `Yosys 0.64` 1 vs 3. So the operation is DELETE-AND-LINK, not the expensive 1141-line relocation. DECIDED: route by kind (rationale -> already in docs/decisions + book, delete+link; user-facing flags -> USER_GUIDE.md; gate invocations -> USER_GUIDE/TOOLBOX; the 78 saw_* fact lines + 15 banked tallies -> docs/evidence per decision 0030; anything NOT already covered -> move FIRST, then delete, proven per-bullet by the probe); caps 250 lines / 12288 bytes DERIVED from the survivors (141 + trimmed quick start + navigation ~= 186) and deliberately BELOW the policy's illustrative 300/16384 which would leave 60% room to regrow; enforced as the 8th registered doctrine README-GROWTH in scripts/check_doctrines.sh (hook + CI, per the policy's own clause) failing with a routing hint naming the canonical home per kind. A GENERATED CLI reference is REJECTED — the option the owner asked to weigh and the one initially favoured: nothing to generate (zero command lines), the SCHEMA-DERIVED reference already exists (decision 0021's queryable knob catalog + --dump-config), and a generator's template/build-step/no-diff-gate is justified against a hand-maintained list but not against a link. Docs-only ⇒ DUT byte-identical.`
  Commit: `README-POLICY-ADOPTION.1 — audit + design ADR (decision 0036)`

- ID: `README-POLICY-ADOPTION.2`
  Status: `done`
  Goal: `Execute the relocation decided at .1. Move each classified block to its canonical home (USER_GUIDE.md / book/src/*.md / ROADMAP.md / docs/decisions/), leaving a navigation link where the content was. Land README_POLICY.md at the repo root.`
  Acceptance: `README.md within the .1 caps; every relocated block present at its destination and linked from the README; mdbook build clean; cargo test --test book_examples green (README is not book-tested, but relocated examples may become book examples); no code change.`
  Verification: `done. README.md 1771 -> 156 lines and 122,767 -> 10,163 bytes (91% reduction), against caps 250/12288 => 62% / 83% used. COVERAGE PROBE RUN THREE WAYS BEFORE ANY DELETION (decision 0036 §3 named this the only place information could be lost): (a) 57 identifying flag/knob tokens, one per bullet -> 57/57 covered elsewhere; (b) 79 distinctive rationale phrases (__cv, care_mask, fan-in-independen, SvVersion::permits, exit code 99, MAX_MULTI_OUTPUT_TASK_GROUP_MEMBERS, ...) -> 79/79 covered, most better covered at the destination; (c) EXHAUSTIVE sweep of every backticked token in the deleted range, 879 distinct -> 32 uncovered, ALL 32 composite invocation strings whose components are individually covered => zero atomic facts lost; (d) the 78 saw_* Phase-4 facts -> 0 missing from all four of USER_GUIDE/book/ROADMAP/tasks. Sweep (c) is the load-bearing one: it searches from the AUTHORITATIVE SET (the README own tokens) not from the shape of the first duplicate found (decision 0033 rule 2). ONE THING ACTUALLY MOVED, and it was a GAP not a duplicate: USER_GUIDE.md documented every gate in prose but had runnable command lines for only 8 of the 15; it now carries a "### Gate invocations" subsection with all 15 plus the composable --resume/--yosys-mode/--iverilog-compile/--sv2v/--slang/--diff-sim forms, derived from tool_matrix.rs own gate registry rather than from the README list. USER_GUIDE.md owns gate invocations (the open question 0036 left to .2); TOOLBOX.md keeps its one-row catalog entry and hosts no command blocks - it is a 106-line instrument catalog and ~20 blocks would recreate the README problem in a second file. QUICK START VERIFIED, not asserted (policy adoption step 3): cargo build + cargo test + cargo run -- --seed 42 + cargo run -- --seed 42 --count 100 --out ./generated (100 .sv + manifest.json) + verilator --lint-only clean on Verilator 5.046; all 21 relative links resolve; ./generated removed, tree clean. RESULT VS PLAN: 0036 projected ~186 lines, landed 156, because the reading order compressed from 36 lines/4,256 B (118 B/line) to a 17-row table (~70 B/entry) - which mattered, because the survivors measured 10,297 B for 141 lines so the projected file was ~13.4 KB, OVER the 12,288 B cap while sitting at 74% of the line cap. Trimmed to fit the cap; cap not raised (README_POLICY.md "Mechanical growth guard"). ONE DOCTRINE SITE DROPPED DELIBERATELY: README.md was a declared site of ENUMERATION-PARITY pair 4 (--steer categories <-> KnobId::category); deleting the --steer bullet would have failed the hook, so the choice was re-add a category list to the landing page or drop the site. Site dropped - under README_POLICY.md a landing page does not enumerate a knob taxonomy, and a list kept alive solely to satisfy a doctrine grows by one line per future category, the exact growth-coupling that produced 1771 lines; this is decision 0033 own R1 rung (repair a shadow by DELETING it, never by gating it forever), and the four surviving sites are the canonical homes the policy routes that content to. Pair 1b UNTOUCHED: README still names all 7 registry ids (DOCTRINE_ENFORCEMENT.md E1 requires it). STALE CROSS-REFERENCES REPAIRED: src/bin/tool_matrix.rs twice claimed banked digests / Phase-4 facts are "cited in README.md" (comment-only edit, invisible to every docs-side check because it lives in Rust); book/src/structured-emission.md said --function-emit-gate is documented in USER_GUIDE.md AND README.md; ROADMAP.md single historical mention gets an APPENDED clarifier, not an edit (the sentence was true when written); CHANGES.md + DEVELOPMENT_NOTES.md mentions left raw (append-only, decision 0031). GATES: cargo fmt --all --check PASS; cargo check --all-targets PASS; cargo clippy --all-targets -- -D warnings PASS; cargo test PASS incl. tests/snapshots.rs untouched => DUT byte-identical; mdbook build book PASS; scripts/check_doctrines.sh 7/7 PASS.`
  Commit: `README-POLICY-ADOPTION.2 — restore the landing page by deletion (1771 -> 156)`

- ID: `README-POLICY-ADOPTION.3`
  Status: `done`
  Goal: `Land the mechanical growth guard in scripts/check_doctrines.sh with the routing hint, negative-control it BOTH ways (an over-cap README must fail the hook; the real README must pass), register it in DOCTRINE_ENFORCEMENT.md §10, and close the tree.`
  Acceptance: `scripts/check_doctrines.sh reports the new doctrine PASS; a synthetic over-cap README makes it FAIL with the routing hint; ENUMERATION-PARITY stays green (the doctrine registry has several documented shadows of itself — the new entry must appear in every one of them, which that doctrine will enforce).`
  Verification: `done. scripts/check_readme_growth.sh landed (caps 250 lines / 12288 bytes from decision 0036 §(c), non-mutating, deterministic, DELIBERATELY NOT scope-aware — landing-page size is a property of the TREE, not of a change, so a commit that does not touch README.md is still checked; otherwise an over-cap README could arrive via a revert or a merge and never be re-examined). Registered as the 8th doctrine README-GROWTH in scripts/check_doctrines.sh; driver reports 8/8 PASS. Added to EVERY documented copy of the registry that ENUMERATION-PARITY gates: DOCTRINE_ENFORCEMENT.md §10 (pair 1, exact parity) + README.md, book/src/architecture.md, docs/knowledge/doctrine-enforcement.md, CODEBASE_ANALYSIS.md (pair 1b, covers_set). CI needs no edit — .github/workflows/ci.yml already runs the driver (E4), and .githooks/pre-commit already runs it (E3). NEGATIVE-CONTROLLED FIVE WAYS in an isolated on-volume fixture root (never `git checkout --` on the real tree — that gotcha cost a README citation once): (1) 251 short lines / 502 bytes -> FAIL, proving the LINE cap catches wrapped prose that the byte cap misses; (2) 100 lines / 20,100 bytes -> FAIL, proving the BYTE cap catches long lines that the line cap misses — the two caps are demonstrably not redundant, which is the claim the design rests on; (3) README_POLICY.md removed -> FAIL (the policy own storage clause: without the project-owned copy the caps are folklore and the routing hint points at a missing document); (4) EXACTLY 250 lines / 12288 bytes -> PASS, proving the comparison is inclusive; (5) one byte more (12289) -> FAIL, proving the boundary is exact with no off-by-one. Plus a sixth on the registry side: removing README-GROWTH from ONE live-registry site (CODEBASE_ANALYSIS.md) makes ENUMERATION-PARITY FAIL naming the file, restored clean and verified byte-exact against the index. Real README passes at 157 lines / 10,226 bytes = 63% / 83% of the caps. mdbook build clean. No src/ change ⇒ DUT byte-identical (cargo test was run green at .2, the last slice that touched Rust).`
  Commit: `README-POLICY-ADOPTION.3 — the README-GROWTH doctrine + close the tree`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `README-POLICY-ADOPTION.1` | `done` | Decision `0036`. The measurement reframed the task: the compliant landing page **already exists** (141 lines) with two appendices bolted on, and `## Current CLI truth` is a **lossy copy** of `docs/decisions/` + the book (phrase probe), so the operation is **delete-and-link**, not relocation. A generated CLI reference is rejected — the SCHEMA-DERIVED one already exists. Caps `250` / `12,288`, derived from the survivors. |
| 2 | `README-POLICY-ADOPTION.2` | `done` | Executed. **1771 → 156 lines / 122,767 → 10,163 bytes (91 %)**, at 62 % / 83 % of the caps. Probe run **three ways** (57 flag tokens, 79 rationale phrases, and an exhaustive sweep of all **879** backticked tokens in the deleted range) ⇒ **zero atomic facts lost**; the only 32 uncovered strings are composites of covered parts. One genuine *gap* filled: `USER_GUIDE.md` gains runnable invocations for all **15** gates (it had 8). `README_POLICY.md` landed. Quick start re-run end-to-end. One doctrine site dropped on purpose (see `.2` verification). |
| 3 | `README-POLICY-ADOPTION.3` | `done` | `README-GROWTH` is the **8th registered doctrine** (`scripts/check_readme_growth.sh`, caps `250` / `12,288`, routing hint), gated by the git hook (E3) **and** CI (E4) through the existing driver. Negative-controlled **five ways** in an isolated fixture root — line-cap-only, byte-cap-only (proving the two caps are not redundant), missing `README_POLICY.md`, exactly-at-cap PASS, one-byte-over FAIL — plus a sixth on the registry side. Driver **8/8 PASS**. |

**Tree status: `closed`.** All three leaves are `done`; the owner directive
(`CLAUDE.md` §14) is implemented, mechanically enforced, and cannot silently
regress. There is no residual work.

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

- **RESOLVED at `.1`** (decision `0036`): the growth guard becomes the **8th
  registered doctrine**, `README-GROWTH`. The policy asks for hook + CI
  enforcement, which is precisely what `DOCTRINE_ENFORCEMENT.md` provides; the
  cost is an entry in every documented copy of the registry, and
  `ENUMERATION-PARITY` already gates those copies, so the cost is mechanical.
- **RESOLVED at `.1`**: the caps apply to `README.md` **alone**. `CHANGES.md`
  and `DEVELOPMENT_NOTES.md` are append-only *by doctrine* (decision `0031`) and
  are exempt; `USER_GUIDE.md`'s length is its purpose. Recorded in
  `README_POLICY.md`'s ANVIL adoption note.
- **RESOLVED at `.1`**: `## Current CLI truth` neither moves wholesale nor is
  generated — it is **deleted and linked**, because a phrase probe proved it a
  lossy copy of layers that already own it. `.2` confirmed this exhaustively
  (879 tokens, zero atomic facts unique to the README).
- **RESOLVED at `.2`** (the question `0036` left open): **`USER_GUIDE.md` owns
  the `tool_matrix --…-gate` invocations**, and now carries runnable command
  lines for all 15 (it had 8). `TOOLBOX.md` keeps its single catalog row and
  hosts no command blocks — it is a 106-line instrument catalog, and ~20 blocks
  would recreate the README problem in a second file.
- **Still open, for `.3`:** whether the byte cap is enforced on the rendered file
  or on tracked bytes (identical today; they diverge only if binary content is
  ever added). Proposed: tracked bytes, read with `wc -c` as the policy's own
  reference check does.

## Blockers

- None. The work is docs-only and depends on nothing in flight.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-07-30` | `README-POLICY-ADOPTION` | `tree registered (docs-only); measured README.md = 1771 lines / 122767 bytes (at HEAD ff506e1, before this session's net-neutral steering-bullet correction); confirmed no repo-root README_POLICY.md; no code touched` | `registered` |
| `2026-07-30` | `README-POLICY-ADOPTION.2` | `coverage probe x4 before deletion: 57/57 flag tokens covered; 79/79 rationale phrases covered; 879 backticked tokens swept exhaustively -> 32 uncovered, all composite invocation strings; 78/78 saw_* facts covered. README.md 1771 -> 156 lines / 122767 -> 10163 bytes (caps 250/12288). Quick start re-run end-to-end: cargo build, cargo test, cargo run -- --seed 42, --count 100 --out ./generated (100 .sv + manifest.json), verilator --lint-only clean (5.046); ./generated removed. 21/21 relative links resolve. USER_GUIDE.md gains 15/15 runnable gate invocations (had 8). README_POLICY.md landed at root.` | `no information lost` |
| `2026-07-30` | `README-POLICY-ADOPTION.2` | `cargo fmt --all --check; cargo check --all-targets; cargo clippy --all-targets -- -D warnings; cargo test (incl. tests/snapshots.rs, untouched); mdbook build book; scripts/check_doctrines.sh (7/7 incl. ENUMERATION-PARITY after dropping the README site from pair 4, and EVIDENCE-CITATIONS after removing ~40 bank citations)` | `all green; DUT byte-identical` |
| `2026-07-30` | `README-POLICY-ADOPTION.3` | `scripts/check_doctrines.sh 8/8 PASS with README-GROWTH registered; real README 157 lines / 10226 bytes (63% / 83% of caps). NEGATIVE CONTROLS in an isolated on-volume fixture root: (1) 251 lines / 502 bytes -> FAIL (line cap catches wrapped prose); (2) 100 lines / 20100 bytes -> FAIL (byte cap catches long lines) => the two caps are demonstrably NOT redundant; (3) README_POLICY.md absent -> FAIL; (4) exactly 250 / 12288 -> PASS (inclusive); (5) 12289 bytes -> FAIL (no off-by-one). Registry side: removing README-GROWTH from CODEBASE_ANALYSIS.md alone -> ENUMERATION-PARITY FAILs naming the file; restored and verified byte-exact against the index. mdbook build clean; CI needs no edit (ci.yml already runs the driver).` | `negative-controlled both ways; tree CLOSED` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `README-POLICY-ADOPTION` | `7a1fc50` — `COVERAGE-STEERED-GENERATION.3c — steering docs + close .3` | Registered (not started) in the docs slice that surfaced it; no README content moved in that commit. |
| `README-POLICY-ADOPTION.1` | `b50ff9e` — `README-POLICY-ADOPTION.1 — audit + design ADR (decision 0036)` | Docs-only; hash backfilled by `55b84d2`. |
| `README-POLICY-ADOPTION.2` | `d6cca64` — `README-POLICY-ADOPTION.2 — restore the landing page by deletion` | Comment-only `src/` touch ⇒ DUT byte-identical. Hash backfilled by `1287b8d`. |
| `README-POLICY-ADOPTION.3` | `5775766` — `README-POLICY-ADOPTION.3 — the README-GROWTH doctrine + close the tree` | Scripts + docs only; no `src/` change. |

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
- `2026-07-30`: `.2` executed the deletion. **`README.md`: 1771 → 156 lines,
  122,767 → 10,163 bytes (91 %)**, at 62 % / 83 % of the caps. The probe ran
  **three ways** before anything was cut — 57 flag tokens, 79 rationale phrases,
  and an **exhaustive sweep of all 879 backticked tokens** in the deleted range —
  and the only 32 uncovered strings are composites of individually-covered parts,
  so **no atomic fact was unique to the README**. `README_POLICY.md` landed at the
  root with an ANVIL adoption note; the quick start was re-run end-to-end rather
  than assumed.
- `2026-07-30`: `.2` found the **one genuine gap** the audit's route table
  predicted ("anything not already covered → move first"). It was not rationale:
  `USER_GUIDE.md` documented every gate in prose but shipped runnable command
  lines for only **8 of 15**. It now carries all 15, derived from
  `tool_matrix.rs`'s own gate registry rather than from the README's list — so
  the new section cannot inherit an omission the README already had.
- `2026-07-30`: `.2` **dropped `README.md` as a declared site of
  `ENUMERATION-PARITY` pair 4** rather than re-adding a `--steer` category list to
  keep the gate green. A landing page does not enumerate a knob taxonomy, and a
  list kept alive solely to satisfy a doctrine grows by one line per future
  category — the exact growth-coupling that produced 1771 lines. Decision `0033`'s
  R1 rung is repair-by-deletion; gating a copy forever is what it warns against.
  Pair **1b** is untouched: the README still names all seven registry ids, which
  `DOCTRINE_ENFORCEMENT.md` E1 (discovery) requires.
- `2026-07-30`: `.3` landed `README-GROWTH` as the **8th registered doctrine** and
  **closed the tree**. The check is deliberately **not scope-aware** — landing-page
  size is a property of the tree, not of a change, so a commit that never touches
  `README.md` is still checked; otherwise an over-cap README could arrive via a
  revert or a merge and never be re-examined. It also enforces the policy's own
  **storage clause** (`README_POLICY.md` must exist beside the file it governs),
  because without the project-owned copy the caps are folklore and the failure's
  routing hint points at a missing document.
- `2026-07-30`: `.3`'s negative controls proved the design claim that the two caps
  are **not redundant**: a 251-line / 502-byte fixture fails on lines while far
  under the byte cap, and a 100-line / 20,100-byte fixture fails on bytes while far
  under the line cap. The boundary is exact and inclusive (250 / 12,288 passes;
  12,289 fails). A sixth control on the registry side confirmed that omitting the
  new entry from a single live-registry site is caught by `ENUMERATION-PARITY`.
- `2026-07-30`: `.2` recorded a cap-calibration finding for `.3`. The survivors
  measured **10,297 bytes for 141 lines**, so the file `0036` projected (~186
  lines) would have been ~**13.4 KB** — *over* the 12,288-byte cap while sitting
  at 74 % of the line cap. **The byte cap had no headroom and the line cap had
  25 %.** The trim was deepened to fit (the reading order became a table at ~70
  B/entry instead of prose at 118 B/line); the cap was **not** raised. Both caps
  stand as `0036` set them.
