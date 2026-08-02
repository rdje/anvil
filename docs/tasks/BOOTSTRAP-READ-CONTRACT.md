# BOOTSTRAP-READ-CONTRACT: the mandatory session-bootstrap read is ~3.2× a full context window, so no session has ever satisfied it

## Metadata

- Tree ID: `BOOTSTRAP-READ-CONTRACT`
- Status: `active`
- Roadmap lane: Workflow / gate quality — session recovery
- Created: `2026-08-02`
- Last updated: `2026-08-02` (registered from a measurement taken while executing an unrelated leaf)
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
  Children: `.1` (design an achievable tiered contract), `.2` (decide whether anything watches it)

- ID: `BOOTSTRAP-READ-CONTRACT.1`
  Status: `pending`
  Goal: `Replace the unsatisfiable "read everything in full" precondition with a tiered contract that is achievable in a real session budget, without losing recovery fidelity.`
  Acceptance: `States the working budget it designs against and where that number comes from. Classifies EVERY item currently mandated by SESSION_BOOTSTRAP.md §1/§2 into mandatory / on-demand / derived, and for each demotion states what a session loses, measured. Must reconcile with MEMORY_ARCHITECTURE.md §5, which already prescribes a bounded read and is the mechanism this contract duplicates and contradicts. Must NOT resume OVERFLOW-DESTINATION-INSTRUMENTATION.`
  Verification: `pending`
  Commit: `pending`

- ID: `BOOTSTRAP-READ-CONTRACT.2`
  Status: `pending`
  Goal: `Decide whether bootstrap compliance should be watched, and by what.`
  Acceptance: `States the by-product route considered before any gate is proposed. A self-reported read is the self-ticked box DOCTRINE_ENFORCEMENT.md §6.1 disqualifies, so a "check" here would have to be structural or it is decorative. Recorded acceptance is a legitimate outcome.`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `BOOTSTRAP-READ-CONTRACT.1` | `pending` | **Next.** The measurement is done and registered; the design is the work. |
| 2 | `BOOTSTRAP-READ-CONTRACT.2` | `pending` | Deliberately after `.1` — deciding what watches a contract before the contract is achievable would gate an impossibility. |

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

## Open Questions

- **What is the working budget?** A number is needed to design against, and it must be *derived*
  rather than fitted — a context window is not a budget, because a session must leave room to work.
  The same failure decisions `0036` §(c) and `0040` §(c) warn about applies here.
- **Is `CODEBASE_ANALYSIS.md` the intended answer to §2 already?** It exists as *"the authoritative
  snapshot of the workspace"* and `SESSION_BOOTSTRAP.md` §3 asks sessions to amend it. If it is the
  derived substitute for walking 95,473 lines, then §2 as written is a second mechanism for a job
  layer C already does — but `CODEBASE_ANALYSIS.md` is 292 KB and itself carries a known wall-of-text
  register (surfaced by `BOOK-PARAGRAPH-BLOBS.3a`), so its fitness for that role must be measured,
  not assumed.
- **Does the mdBook belong in the mandatory tier?** `SESSION_BOOTSTRAP.md` argues hardest for it
  (*"a session that skips the mdBook will make locally-correct but globally-wrong decisions"*), and
  it is also the surface the owner reviews (`COMMIT.md` §9). At 752 KB it is ~19 % of a context
  window. This is the sharpest instance of the trade-off `.1` must resolve.

## Blockers

- None.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-08-02` | `.0` | `Measured at 089566b by wc -c over the exact file set SESSION_BOOTSTRAP.md §1/§2 names: §1 live docs + book 5,499,811 B; docs/tasks/*.md 3,238,704 B; src+tests+examples *.rs 3,933,732 B across 95,473 lines; total 12,672,247 B ~ 3.17 M tokens against a ~4 MiB / 1 M-token context. Append-only share of §1 measured separately: CHANGES.md 2,639,128 + DEVELOPMENT_NOTES.md 1,134,982 = 68 % of the live-doc mass, both append-only by design, so the gap widens monotonically. Ownership search run over the four trees naming SESSION_BOOTSTRAP.md and the four whose subject is doc size; none owns the read contract.` | `registered` (docs-only; DUT byte-identical) |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `.0` (registration) | `cb24484` — `BOOTSTRAP-READ-CONTRACT.0 — register the unsatisfiable session-bootstrap read contract` | Docs-only; no work leaf executed yet. `.0` is this repo's registration-commit convention, required because `.githooks/commit-msg` rejects a subject naming no leaf. |

## Changelog

- `2026-08-02`: Created from a measurement taken while executing `BOOK-PARAGRAPH-BLOBS.3c`, when the
  startup checklist forced the question *"did I actually read all of that?"* and the honest answer
  was no. Measured rather than confessed: the mandated read is **12.7 MB ≈ 3.17 M tokens against a
  ~1 M-token context — about 3.2×** — and **§1 alone exceeds a full context window** before any task
  tree or source file is opened. **68 %** of the live-doc mass is two files that are append-only by
  design, so the contract recedes with every commit. Nothing observes bootstrap completeness, which
  makes it a fresh instance of `UNGATED-PRACTICE-AUDIT.1`'s rule: it is a by-product of nothing.
  Scoped away from file size — [`OVERFLOW-DESTINATION-INSTRUMENTATION`](OVERFLOW-DESTINATION-INSTRUMENTATION.md)
  owns that and is **paused** — and explicitly **not** a mandate to gate.
