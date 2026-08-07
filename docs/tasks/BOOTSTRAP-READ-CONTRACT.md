# BOOTSTRAP-READ-CONTRACT: the mandatory session-bootstrap read is ~3.2× a full context window, so no session has ever satisfied it

## Metadata

- Tree ID: `BOOTSTRAP-READ-CONTRACT`
- Status: `active`
- Roadmap lane: Workflow / gate quality — session recovery
- Created: `2026-08-02`
- Last updated: `2026-08-07` (`.1` done — the tiered contract landed; `.3` registered)
- Owner: repo-local workflow

## Goal

`SESSION_BOOTSTRAP.md` §1 instructs every session to *"Read every live doc, in this order, **in
full**"*, and §2 to *"Walk **every** source file under `src/`, every test under `tests/`, and every
example under `examples/`."* Both are stated as preconditions: *"No code or doc change may happen
until you have read and understood all three."*

**Measured `2026-08-02` at `089566b`, before registering:**

| Mandate | bytes | ≈ tokens |
| --- | --- | --- |
| §1 live docs (9 files) + `book/src/*.md` (31 chapters) | 5,499,811 | ~1.37 M |
| §1's "if any tree is `active`, read the linked file too" — `docs/tasks/*.md` (84 trees) | 3,238,704 | ~0.81 M |
| §2 walk every `.rs` (95,473 lines) | 3,933,732 | ~0.98 M |
| **total** | **12,672,247** | **~3.17 M** |

A 1 M-token context holds roughly 4 MiB. **§1 alone exceeds a full context window** at ~1.37 M
tokens, before a single task tree is opened, before the source walk, and before any work.

So the contract is **unsatisfiable by construction**, and has been for some time. Every session
either reads a fraction and proceeds, or spends its entire context on the read and has none left to
work with. The first is what actually happens, and **nothing observes it** — which makes this an
instance of the rule `UNGATED-PRACTICE-AUDIT.1` measured (`0e4654f`):

> *a practice survives where its output is a by-product of work the author is already forced to do.*

Bootstrap-read completeness is a by-product of nothing. No commit is blocked by it, no gate reads it,
and the only evidence a session ever produces is its own claim to have done it — precisely the
self-ticked box `DOCTRINE_ENFORCEMENT.md` §6.1 says must never be the proof.

**It gets worse monotonically, and that is the part that makes it a defect rather than a
curiosity.** `CHANGES.md` (2.64 MB) and `DEVELOPMENT_NOTES.md` (1.13 MB) are **append-only by
explicit design** and are **68 %** of §1's live-doc mass. Every commit made under this workflow moves
the contract further out of reach. `SESSION_BOOTSTRAP.md` already hedges for exactly one of them
(*"`CHANGES.md` (most recent entries at minimum)"*) and for none of the others.

## Non-Goals

- **Not a proposal to shrink or truncate the append-only files.** `CHANGES.md` and
  `DEVELOPMENT_NOTES.md` are append-only by owner direction and `0031` forbids rewriting history.
  The defect is in the *read contract*, not in the files.
- **Not the file-size question.** [`OVERFLOW-DESTINATION-INSTRUMENTATION`](OVERFLOW-DESTINATION-INSTRUMENTATION.md)
  already owns *"which overflow destinations are unmeasured"* and already names all five of these
  files. That tree is **PAUSED and must not be resumed without an owner nudge**. This tree's subject
  is disjoint: whether the **mandated read** is achievable, which is a property of
  `SESSION_BOOTSTRAP.md`, not of any file's size. Stated here so the boundary is explicit rather
  than discovered later.
- **Not a mandate to gate this.** Whether anything should *check* that a bootstrap happened is `.2`'s
  question, and decision [`0047`](../decisions/0047-negative-control-carrier-is-the-mutation.md)
  prefers *removing the need* over *watching harder*. **"Recorded acceptance" is a legitimate
  outcome.**
- **No `src/` change.** Workflow/docs only ⇒ DUT byte-identical.

## Acceptance Criteria

- `.1` states, with a denominator, what a session **can** read in a working budget, and proposes a
  bootstrap contract that is **achievable as written** — a tiered read (what is mandatory, what is
  on-demand, what is derived) rather than an aspiration. It must say what is **lost** by not reading
  each demoted item, measured rather than asserted.
- Any revision to `SESSION_BOOTSTRAP.md` must keep the property that made the current text valuable:
  a cold session reaching *"the same operational state as the session that just ended."* A shorter
  contract that loses recovery fidelity is a worse defect than a long one nobody follows.
- `.2` decides whether anything watches bootstrap compliance, and states the by-product alternative
  considered **before** proposing a gate.
- The `MEMORY_ARCHITECTURE.md` §5 READ path — *"A resume reads A + one unit of B + a few C records —
  never a monolith"* — is treated as the **already-correct answer** that `SESSION_BOOTSTRAP.md`
  contradicts. Reconciling the two is `.1`'s core task; they are two mechanisms for one job
  (`feedback_full_factorization`).

## Task Tree

- ID: `BOOTSTRAP-READ-CONTRACT`
  Status: `active`
  Goal: `The session-bootstrap contract is achievable as written, and a session that follows it reaches full operational state.`
  Children: `.1` (design an achievable tiered contract), `.2` (decide whether anything watches it),
  `.3` (reconcile the *second* read contract carried by the six harness bootstrap pointers — measured
  at `.1`, registered rather than absorbed)

- ID: `BOOTSTRAP-READ-CONTRACT.1`
  Status: `done`
  Goal: `Replace the unsatisfiable "read everything in full" precondition with a tiered contract that is achievable in a real session budget, without losing recovery fidelity.`
  Acceptance: `Encodes the 2026-08-07 owner directive as its design principle: the contract is SCOPED TO THE WORK IN HAND, not a shortened universal list, and its sufficiency test is understanding the subject of the change rather than covering a corpus. States the working budget it designs against and where that number comes from. Classifies EVERY item currently mandated by SESSION_BOOTSTRAP.md §1/§2 into mandatory / on-demand / derived, and for each demotion states what a session loses, measured. Must reconcile with MEMORY_ARCHITECTURE.md §5, which already prescribes a bounded read and is the mechanism this contract duplicates and contradicts. Must NOT resume OVERFLOW-DESTINATION-INSTRUMENTATION.`
  Verification: `Contract rewritten to three tiers in SESSION_BOOTSTRAP.md; design recorded in docs/decisions/0049. Budget DERIVED (0036 §(c) / 0040 §(c) method): demonstrated-sufficient fixed read 114,689 B = 2.73% of a 4 MiB window at 13c9cdf, doubled and rounded to the next binary round number => 256 KiB = 262,144 B, which that tier filled to 43.8% — a cap that lands with ~55% headroom is a bound, not a description. After this commit the tier is 120,178 B = 2.87%, 45.8% of budget, and the record's reverify expects that post-landing figure rather than the flattering one. Every §1/§2 item classified with its demotion loss measured at 13c9cdf; see the Verification Log row for the full set.`
  Commit: `f9e1c61`

- ID: `BOOTSTRAP-READ-CONTRACT.2`
  Status: `pending`
  Goal: `Decide whether bootstrap compliance should be watched, and by what.`
  Acceptance: `States the by-product route considered before any gate is proposed. A self-reported read is the self-ticked box DOCTRINE_ENFORCEMENT.md §6.1 disqualifies, so a "check" here would have to be structural or it is decorative. Recorded acceptance is a legitimate outcome. Must carry the two inputs .1 hands it: SESSION_BOOTSTRAP.md is a 0040 class-B1 surface (bounded by a stated contract, and PAID AGAIN on every compaction via the PostCompact hook), so it is a legitimate cap subject that .1 deliberately did not cap; and the only readable trace of a bootstrap is the author's own claim, which §6.1 disqualifies before the question is asked.`
  Verification: `pending`
  Commit: `pending`

- ID: `BOOTSTRAP-READ-CONTRACT.3`
  Status: `pending`
  Goal: `Reconcile the SECOND mandatory-read contract — the one carried by the six harness bootstrap pointers — with the tiered contract .1 landed.`
  Acceptance: `Measured at .1 and recorded in 0049 §(c): the six pointers (AGENTS.md, CLAUDE.md, GEMINI.md, .cursorrules, .windsurfrules, .github/copilot-instructions.md — identical bodies) mandate a 7-item list of 801,434 B whose intersection with SESSION_BOOTSTRAP.md §1 is EXACTLY TWO files (README.md, MEMORY.md); none of them routes to SESSION_BOOTSTRAP.md at all, which is reachable only from README.md's closing line; and the list mandates the 694,469 B DERIVED KNOWLEDGE_MAP.md in full, against its own standard's §7 lookup contract and §1 statement that the map may be skipped entirely. The repair must be R1 repair-by-deletion (0033) — route to the one contract, do not restate a second list — and must respect two hard constraints: DOCTRINE_ENFORCEMENT.md §8 Group C requires the six BODIES stay identical (title lines may differ), and scripts/check_memory_architecture.sh asserts each file names README.md AND MEMORY_ARCHITECTURE.md. Negative-control the check both ways before and after.`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `BOOTSTRAP-READ-CONTRACT.3` | `pending` | **Next.** `.1` left a stated residue: two read contracts exist, one is now correct and the other still names a list. Ordered before `.2` because deciding what *watches* a contract while two contradictory ones are live would gate the wrong object. |
| 2 | `BOOTSTRAP-READ-CONTRACT.2` | `pending` | Deliberately last — deciding what watches a contract before the contract is achievable would gate an impossibility, and `.1` hands it two inputs it did not have. |
| — | `BOOTSTRAP-READ-CONTRACT.1` | `done` | Tiered contract landed in `SESSION_BOOTSTRAP.md`; design and measurements in [`0049`](../decisions/0049-the-bootstrap-read-is-scoped-to-the-work.md). |

## Decisions

- `2026-08-02` (registration): **Registered rather than mentioned in passing.** The standing
  directive is that a defect is handled only when a tree owns it
  ([`0041`](../decisions/0041-owner-standing-directives-recorded-in-layer-c.md)). This one is
  especially prone to being lost, because the only session positioned to notice it is one that has
  just failed to satisfy it — and that session has every incentive to move on.
- `2026-08-02` (registration): **Measured before registering, with a denominator.** *"The bootstrap
  is long"* is an impression; **12.7 MB against a ~4 MiB context, ~3.2×** is a scope. The ratio is
  what makes this a contract defect rather than a request for more discipline.
- `2026-08-02` (registration): **Ownership search run, not assumed.** Four trees mention
  `SESSION_BOOTSTRAP.md` (`DATED-COUNT-SWEEP-EXEMPTION`, `LIVE-DOC-CODEBASE-ALIGNMENT`,
  `OVERFLOW-DESTINATION-INSTRUMENTATION`, `RESUME-POINTER-CONTRACT`); none owns its **read
  contract**. `RESUME-POINTER-CONTRACT` is `closed` with its hypothesis measured **false** and must
  not be reopened without new evidence; `OVERFLOW-DESTINATION-INSTRUMENTATION` owns file *size* and
  is **paused**. So this is a first mechanism, not a second.
- `2026-08-02` (registration): **The report that produced this is itself the evidence.** The session
  that registered it had read `README.md`, `MEMORY.md`, `MEMORY_ARCHITECTURE.md`,
  `DOCTRINE_ENFORCEMENT.md`, `TOOLBOX.md`, `COMMIT.md`, `docs/TASK_TREE.md`, the active tree, and the
  two book chapters its leaf touched — and had **not** read `ROADMAP.md`, `CHANGES.md`,
  `DEVELOPMENT_NOTES.md`, `CODEBASE_ANALYSIS.md`, `USER_GUIDE.md`, the other 29 book chapters, or any
  of the 95,473 lines of Rust, in full. It shipped a correct leaf anyway, which is the useful
  datum: the *effective* contract that produces good work is much smaller than the written one, and
  `.1`'s job is to write down the effective one.

- `2026-08-07` (**OWNER DIRECTIVE — this answers the tree's central question**), verbatim: *"read
  whatever is necessary for the new session. The idea is to be knowledgable about what need to be
  worked on, that is, because I find it dangerous to fix something you don't understand."*

  **This is the design principle `.1` must encode, and it settles the tier axis.** The contract is
  **not** a fixed reading list to be shortened; it is **scoped to the work in hand** — a session
  reads what it needs to *understand what it is about to change*, and the sufficiency test is
  comprehension of the subject, not coverage of a corpus. That reframing dissolves the 3.2×
  arithmetic rather than negotiating it: an exhaustive read was never the goal the owner wanted, and
  a session that read all 12.7 MB but did not understand its subject would still be doing the
  dangerous thing this directive forbids.

  **Two consequences `.1` must carry rather than re-derive.** (a) The **danger** named is *fixing
  what you do not understand*, so the mandatory tier is whatever grounds the **specific change**,
  which varies per leaf and therefore cannot be a static list — the current §1/§2 enumeration is the
  wrong *shape*, not merely the wrong *length*. (b) `MEMORY_ARCHITECTURE.md` §5's *"A resume reads A
  + one unit of B + a few C records — never a monolith"* is **already** this directive expressed as a
  read path, which confirms the two mechanisms must be reconciled into one, not balanced.

  **Recorded here rather than only in layer C because it is the acceptance input for `.1`.** It is
  also a standing directive of general scope and should be promoted to a `docs/decisions/` record by
  `.1`, alongside [`0041`](../decisions/0041-owner-standing-directives-recorded-in-layer-c.md).
  **Promoted at `.1`** to [`0049`](../decisions/0049-the-bootstrap-read-is-scoped-to-the-work.md),
  verbatim and with the design it settles.

- `2026-08-07` (`.1`): **The budget is derived by the `0036` §(c) / `0040` §(c) method, and the test
  applied is that it must land with headroom.** A cap fitted to what already exists sits at ~100 % of
  itself on the day it lands and can only ever be raised — which is how `MEMORY.md`'s line cap failed
  for two months (`0040`). So: take the **demonstrated-sufficient** fixed read (**114,689 B**, the
  invariant part of what `.0`'s session had actually read when it shipped a correct leaf), double it,
  round up to the next binary round number ⇒ **256 KiB = 262,144 B**, which that tier filled to
  **43.8 %**. The number worth quoting is the complement: **≥ 93.75 %** of the window left for the
  work, an axis the 3.03× contract had none of. **The circularity is real and reported, not
  absorbed:** `SESSION_BOOTSTRAP.md` is itself in Tier 1 and this leaf rewrites it, so the derivation
  uses the **pre-change** state and then states the cost — after this commit the tier is
  **120,178 B (2.87 %, 45.8 % of budget)**, the file having grown **4,277 → 9,709 B** to carry the
  tiers. Quoting only the smaller figure would be fitting the evidence to the conclusion.

- `2026-08-07` (`.1`): **The `PostCompact` hook is what makes the tiering structural rather than
  stylistic, and it was found by reading the repo's own configuration.** `.claude/settings.json`
  re-injects `SESSION_BOOTSTRAP.md` **verbatim** after every compaction, and the file's own text says
  the protocol is *freshly in force*. So the fixed tier is paid **once per compaction cycle**, not
  once per session: a session that compacts *N* times pays it *N+1* times. At **2.73 %** that is
  sustainable; at the old **1.31×** it was not survivable even once. This also classifies
  `SESSION_BOOTSTRAP.md` as a `0040` **class-B1** surface — handed to `.2`, not acted on here.

- `2026-08-07` (`.1`): **Deleting `SESSION_BOOTSTRAP.md` outright was considered and rejected on what
  it destroys.** It is the tempting `0033` R1 repair once `MEMORY_ARCHITECTURE.md` §5 is recognised as
  the same mechanism — but §5 is deliberately **project-agnostic** and names no file in any codebase,
  so deletion would leave nothing saying which file is layer A *here*, what the sanity checks are, or
  that the task-tree doctrine binds immediately on recovery — and it would empty the `PostCompact`
  hook. The merge is therefore by **subordination**: §5 owns the shape, this file owns the binding.

- `2026-08-07` (`.1`): **A second read contract was measured and deliberately not absorbed.** The six
  harness bootstrap pointers mandate a different 7-item list; the intersection with §1 is **exactly
  two files**. Registered as `.3` rather than fixed in-leaf: `COMMIT.md` requires one completed leaf
  per commit, the six files are under a Group-C identical-body constraint, and
  `scripts/check_memory_architecture.sh` asserts content in each of them — three reasons the repair is
  its own unit of work. The residue is **stated, not hidden**: until `.3` lands, one contract is
  correct and the other still names a list.

## Open Questions

- ~~**What is the working budget?**~~ **Answered at `.1`:** **256 KiB = 262,144 B** for the fixed
  tier, derived from the demonstrated-sufficient read × 2 rounded to the next binary round number, and
  landing at **43.8 %** occupancy. Tiers 2 and 3 deliberately carry **no** byte figure — one would be
  fitted, and would re-import the corpus-coverage thinking the contract removes.
- ~~**Is `CODEBASE_ANALYSIS.md` the intended answer to §2 already?**~~ **Answered at `.1`, and the
  answer is "half".** Measured: it names **55 of 55** `src/*.rs` files in **7.44 %** of the source
  mass (13.4× compression), so as an *orientation map* it is complete and §2's walk was indeed a
  second mechanism. It is **not** a substitute for reading the file you are about to change —
  `0039` measured **9 of 13** of its per-file claims stale, every error an under-count — so Tier 2
  requires the touched source **in full, at source level**. The wall-of-text register
  `BOOK-PARAGRAPH-BLOBS.3a` surfaced is a *readability* defect of that file and is not this tree's.
- ~~**Does the mdBook belong in the mandatory tier?**~~ **Answered at `.1` from the file's own text.**
  `SESSION_BOOTSTRAP.md` mandated all 30 chapters in §1 while, four headings later, naming exactly
  three as too load-bearing to edit casually (`core-idea.md`, `non-goals.md`, `why-not-grammar.md`).
  Those three plus `SUMMARY.md` are **20,391 B — 2.7 %** of the book and **0.49 %** of a window, so
  the invariant part is affordable and the rest is subject-scoped through `SUMMARY.md`. The file
  already knew the answer; §1 never used it.
- **Open, and handed to `.2`:** nothing observes whether a session understood its subject, and
  nothing can — the only trace is a self-report, which `DOCTRINE_ENFORCEMENT.md` §6.1 disqualifies
  before the question is asked. `.2` must state the by-product route before proposing any gate.

## Blockers

- None.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-08-07` | `.1` | `Re-measured at 13c9cdf over the same file set, against a 4 MiB / ~1 M-token window: §1 live docs + book 5,507,558 B = 1.31x a window ON ITS OWN; docs/tasks/*.md 3,252,947; src+tests+examples *.rs 3,933,732 across 68 files; total 12,694,237 B = 3.03x. Append-only share of §1 mass: CHANGES.md 2,644,624 + DEVELOPMENT_NOTES.md 1,134,982 = 68.6%. TIER 1 (fixed) = README + MEMORY + MEMORY_ARCHITECTURE + DOCTRINE_ENFORCEMENT + TOOLBOX + COMMIT + SESSION_BOOTSTRAP = 114,689 B = 2.73% of a window at 13c9cdf; budget 262,144 B, occupancy 43.8%. Re-run AFTER this commit: 120,178 B = 2.87%, occupancy 45.8% — SESSION_BOOTSTRAP.md itself grew 4,277 -> 9,709 B carrying the tiers, so the derivation uses the pre-change state and the +4.7% is reported as the contract's cost rather than absorbed. Per-demotion loss, measured: docs/TASK_TREE.md splits 11,466 B rules (3.8%) vs 288,109 B of 91 index rows (96.2%), mean row 3,166 B, and 0042 measured those rows re-stating 416 of 508 leaves (81.9%) the tree files already hold; CHANGES.md reachable by key — 632 of 709 entries carry **Landed as:**, 459 name a leaf id in the heading; ROADMAP.md per-leaf content already carried by every tree file's own "- Roadmap lane:" field, 84 of 84; CODEBASE_ANALYSIS.md names 55 of 55 src/*.rs basenames at 7.44% of the source mass (13.4x), complete as a map, stale as a substitute per 0039's 9 of 13; book core (SUMMARY + the 3 chapters this file itself names load-bearing) 20,391 B = 2.7% of the book, 0.49% of a window; KNOWLEDGE_MAP.md 694,469 B is derived and lookup-contracted by its own standard §7/§1. Second-contract census: the 6 harness pointers mandate 7 items = 801,434 B = 0.19x window, intersection with §1 exactly {README.md, MEMORY.md}, and none routes to SESSION_BOOTSTRAP.md. No src/ touched.` | `.1 done` (docs/workflow only; DUT byte-identical, tests/snapshots.rs untouched) |
| `2026-08-02` | `.0` | `Measured at 089566b by wc -c over the exact file set SESSION_BOOTSTRAP.md §1/§2 names: §1 live docs + book 5,499,811 B; docs/tasks/*.md 3,238,704 B; src+tests+examples *.rs 3,933,732 B across 95,473 lines; total 12,672,247 B ~ 3.17 M tokens against a ~4 MiB / 1 M-token context. Append-only share of §1 measured separately: CHANGES.md 2,639,128 + DEVELOPMENT_NOTES.md 1,134,982 = 68 % of the live-doc mass, both append-only by design, so the gap widens monotonically. Ownership search run over the four trees naming SESSION_BOOTSTRAP.md and the four whose subject is doc size; none owns the read contract.` | `registered` (docs-only; DUT byte-identical) |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `.0` (registration) | `cb24484` — `BOOTSTRAP-READ-CONTRACT.0 — register the unsatisfiable session-bootstrap read contract` | Docs-only; no work leaf executed yet. `.0` is this repo's registration-commit convention, required because `.githooks/commit-msg` rejects a subject naming no leaf. |
| `.0` (owner directive) | `8101dbb` — `BOOTSTRAP-READ-CONTRACT.0 — record the owner directive answering the tree's question` | Recorded the `2026-08-07` directive verbatim and folded it into `.1`'s acceptance. Hash backfilled by `13c9cdf`. |
| `.1` | `f9e1c61` — `BOOTSTRAP-READ-CONTRACT.1 — replace the unsatisfiable bootstrap read with a work-scoped tiered contract` | Rewrote `SESSION_BOOTSTRAP.md` to three tiers; recorded the design and the owner directive in [`0049`](../decisions/0049-the-bootstrap-read-is-scoped-to-the-work.md); registered `.3`. Docs/workflow only. |

## Changelog

- `2026-08-07` (`.1`): The unsatisfiable *"read every live doc, in full"* precondition is replaced by
  a **work-scoped, three-tier** contract, encoding the owner directive as its design principle: the
  read is scoped to the change in hand and its sufficiency test is **understanding the subject**, not
  covering a corpus — so the **3.03×** arithmetic dissolves rather than being negotiated down. Fixed
  tier measured **114,689 B = 2.73 %** of a window against a **derived** budget of **256 KiB**
  (43.8 % occupancy — headroom is the test that it bounds rather than describes). Every §1/§2 item
  classified with its demotion loss measured. Two questions the tree flagged as hardest answered from
  the repository's own text: `CODEBASE_ANALYSIS.md` is a **complete map and a stale substitute**
  (55/55 files, but `0039`'s 9-of-13), and the book's invariant core is the **three chapters
  `SESSION_BOOTSTRAP.md` itself already forbade editing casually** — §1 mandated thirty while the
  same file knew which three mattered. Reconciled with `MEMORY_ARCHITECTURE.md` §5 by
  **subordination**, not balance. Design recorded in
  [`0049`](../decisions/0049-the-bootstrap-read-is-scoped-to-the-work.md). New leaf `.3` registered
  for a **second** read contract found while classifying — the six harness bootstrap pointers mandate
  a different list whose intersection with §1 is **exactly two files**, and none of them routes here
  at all. `OVERFLOW-DESTINATION-INSTRUMENTATION` **not** resumed; no cap set on any file.

- `2026-08-02`: Created from a measurement taken while executing `BOOK-PARAGRAPH-BLOBS.3c`, when the
  startup checklist forced the question *"did I actually read all of that?"* and the honest answer
  was no. Measured rather than confessed: the mandated read is **12.7 MB ≈ 3.17 M tokens against a
  ~1 M-token context — about 3.2×** — and **§1 alone exceeds a full context window** before any task
  tree or source file is opened. **68 %** of the live-doc mass is two files that are append-only by
  design, so the contract recedes with every commit. Nothing observes bootstrap completeness, which
  makes it a fresh instance of `UNGATED-PRACTICE-AUDIT.1`'s rule: it is a by-product of nothing.
  Scoped away from file size — [`OVERFLOW-DESTINATION-INSTRUMENTATION`](OVERFLOW-DESTINATION-INSTRUMENTATION.md)
  owns that and is **paused** — and explicitly **not** a mandate to gate.
