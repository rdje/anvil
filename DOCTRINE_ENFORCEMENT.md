# Doctrine Enforcement Architecture

A portable, **project-agnostic** standard for turning written rules ("doctrines") into
**mechanically enforced** ones — so compliance is *provable and re-checkable*, never a
"trust me" claim. Drop the kit (§8) into any repository and a non-compliant change cannot
land: a local git hook blocks it, and CI makes it un-mergeable.

> **👉 Adopting this in your project? THIS is the only document you need to follow.** Go straight to
> **§8 — The portable replay manifest**: copy the core files (Group A), adapt a handful of knobs
> (Group B), add your harness's bootstrap pointer (Group C), run the 3 setup commands. Sections 1–7
> are the rationale + the check-script contract; §9 is the honest limits; §10 is the live ANVIL
> instance.

> One-line thesis: **a doctrine that is not mechanically checked is not enforced — it is a
> suggestion.** The fix is to pair every doctrine with a deterministic check, run all checks
> from one registry/driver, and gate commits + CI on it.

This file is the **4th portable architecture** ANVIL adopts, alongside the three it already had:

| # | Portable architecture | Owns | Standard |
|---|---|---|---|
| 1 | **Task-trees** | per-unit work memory (goal/frontier/acceptance/verification) | `docs/TASK_TREE.md` |
| 2 | **Memory-architecture** | durable harness-agnostic agent memory (4 layers) | `MEMORY_ARCHITECTURE.md` |
| 3 | **Knowledge-map** | a retrieval layer over fact cards | `knowledge-map/KNOWLEDGE_MAP_ARCHITECTURE.md` |
| 4 | **Doctrine-enforcement** | turning every rule into a mechanically-gated check | **this file** |

All four are **project- and harness-agnostic**: a project backed by Codex, Claude Code, Gemini, or
a human adopts each by replaying its standard. This one is the sibling of `MEMORY_ARCHITECTURE.md` —
that standard mechanizes the *memory* doctrine; this one generalizes the *same E1→E4
defense-in-depth* to **every** doctrine. The enforcement is **git-level** (hooks + CI), so it fires
identically no matter which harness made the commit.

---

## 0. How to use this file

1. Read it once. Adopt the **check-script contract** (§4) and the **driver+registry** (§5).
2. Copy the agnostic kit (§8): the driver, one example check, the hook, the CI step.
3. For each doctrine you want enforced, write a `check_<doctrine>.sh` and register it.
4. Run the three setup commands (§8). From then on, non-compliance fails fast (hook) and cannot
   merge (CI).

If you remember one rule: **route every doctrine to a check, register it, gate on the driver.**

---

## 1. The problem

Most doctrines live as prose (a README section, a decision record, a code comment). Prose is
**discoverable but not enforceable** — an agent or human can read it and still ignore it, and
nothing catches the violation until much later (or never). The two failure modes:

- **"Trust me" compliance** — a change claims it followed the rule; no artifact proves it.
- **Silent drift** — a rule erodes one exception at a time because nothing re-checks it.

The cure is not more prose. It is to make the **compliant path the gated path**: every doctrine
gets a check that *re-derives the truth from the repository*, and the gates run that check.

---

## 2. The core idea

> **doctrine = a rule + a deterministic check that exits nonzero on any breach.**

Once a doctrine has such a check, enforcement is mechanical:

- one **driver** runs every registered check and reports per-doctrine PASS/FAIL (§5);
- the **git hook** runs the driver (fast local gate, E3);
- **CI** runs the *same* driver (un-bypassable backstop, E4).

The check is the single source of truth for the rule; the prose doc explains *why*, the check
decides *whether*.

---

## 3. The three check archetypes (pick one per doctrine)

Every mechanizable doctrine fits one of three shapes. Pick by what makes the proof real.

| Archetype | The check… | Proof strength | Cost / where to run | ANVIL example |
|---|---|---|---|---|
| **Structural** | re-derives an invariant from the tree (allowlist match, file presence, lockstep/derived-artifact sync) | a fact about the files — cannot be faked | cheap → pre-commit | "the Knowledge Map is regenerated + staged + in sync"; "`MEMORY.md` is ≤ the line cap and the bootstrap pointers route correctly" |
| **Oracle (re-run)** | re-EXECUTES a deterministic tool at fixed inputs (fixed seeds / golden inputs) and asserts the result | strongest — a fabricated claim does not reproduce | may be heavy → defer to CI / local | "`tests/snapshots.rs` byte-identical reproducibility at the canonical seeds"; "`tool_matrix --<surface>-gate` is downstream-clean with `coverage_gaps = []`" |
| **Evidence (artifact)** | requires a re-checkable artifact for an action that cannot be re-derived (e.g. *how* a finding was diagnosed) — pasted tool output in a tracked location, ideally with the cited command re-run | medium → strong (strong when the cited command is re-run) | cheap (presence) / heavy (re-run) | "a code change records its validation (named checks + downstream results) in `CHANGES.md` and the owning task leaf's Verification Log" |

Rule of thumb: prefer **structural** (cannot be faked) → then **oracle** (re-run beats trust) →
use **evidence** only where the thing being enforced is an *action/process* that leaves no other
re-derivable trace. For evidence checks, make them as oracle-like as possible (re-run the cited
command) so they are not bypassable by pasting fake output.

---

## 4. The check-script contract (precise — this is what makes it portable)

A doctrine check is **any executable** that obeys this contract. Get this right and any project,
any language, can add doctrines that "just work" with the driver.

1. **Exit code is the verdict.** `exit 0` ⟺ the doctrine holds; **any nonzero** ⟺ a breach.
2. **Explain on breach.** On nonzero, print a human-actionable message to **stderr** (what broke,
   where, how to fix). On pass, stay quiet or print one OK line.
3. **Deterministic.** Same repository state → same verdict. No clocks, no network, no randomness
   (or pin the seed). This is what lets the gate be trusted and CI re-run it.
4. **Reads the repository (+ `git`), mutates nothing** (a *derive-and-stage* step — like
   regenerating the Knowledge Map — is allowed but must be idempotent and explicit).
5. **Scope-aware where relevant.** A check about a *change* should look at the staged set
   (`git diff --cached --name-only`) or an explicit range, and **exempt** changes it does not
   govern (e.g. a code-only doctrine exempts pure-docs / workflow commits) — so it never blocks
   unrelated work.
6. **Self-contained + path-agnostic.** Resolve the repo root from the script's own location
   (`ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"`); reference repo-relative paths only.
7. **Fast, or deferred.** If a check is too slow for pre-commit, keep it in the registry but mark
   it CI-only (run the cheap structural proxy locally, the full oracle in CI / locally per
   `COMMIT.md`).

A check that obeys (1)–(7) is portable: the driver does not care what it checks or how.

---

## 5. The registry + driver (the general enforcer)

One driver owns the list of doctrines and runs them all. The **registry is the source of truth**
for "which doctrines are enforced by what"; a human-readable manifest (§10) mirrors it.

- **Registry**: a list of `id | what-it-proves | path/to/check.sh`.
- **Driver**: runs every check (collecting *all* results, not stopping at the first failure),
  prints a per-doctrine report, and exits nonzero iff any failed. It also **meta-checks** that
  every registered check exists and is executable — so a registry entry can never be a dangling
  promise.
- **Adding a doctrine** = write a `check_*.sh` obeying §4 + add one registry line. Nothing else.

ANVIL ships the driver at [`scripts/check_doctrines.sh`](scripts/check_doctrines.sh). The
acceptance-checklist template that an ANVIL code change must satisfy — and the catalog of ANVIL's
own diagnostic tools — lives in [`TOOLBOX.md`](TOOLBOX.md).

---

## 6. The "reasoned-from-evidence" pattern (process made checkable)

The hardest doctrine to enforce is a *process* ("you validated the change and reasoned from the
evidence"). You cannot read an author's mind — so reframe it into something mechanical:

> **A correct change is one whose documented cause→fix→effect (or build→validate→accept) chain
> REPRODUCES under independent re-execution.**

Mechanize it as a **two-signal evidence check** (the procedure made checkable):

1. **DIAGNOSIS / BUILD signal (WHY+WHERE)** — the change names the tool that *located and explained*
   the issue (a `--trace` excerpt, an `analyze` support cone, a downstream rejection trace) or the
   construction it adds.
2. **VERIFICATION signal (effect)** — the change records the *measured* result (named `cargo`
   checks, the relevant `tool_matrix` gate going `coverage_gaps = []`, a REJECT→PASS, byte-identical
   determinism across the canonical seeds) in `CHANGES.md` + the owning task leaf.

The gate requires **both**. The **oracle leg** then re-runs the cited deterministic commands (in CI
/ locally per `COMMIT.md`): a fabricated chain will not reproduce, so it fails. At that point the
distinction between "reasoned" and "fabricated" collapses — *a reproducible chain is, operationally,
a correct change.*

### 6.1 A box is EARNED, not ticked (self-ticking is not proof)

A checklist `[x]` an author writes is a **claim**, not proof. So **ticking must never be the proof;
the oracle re-run is.** Three legs, in increasing strength:

1. **Presence (cheap, local hook):** the box exists / the mandatory live-doc is staged with the
   code change. Catches "forgot the step." Self-tickable — necessary, not sufficient.
2. **Evidence-shape:** the change co-occurs with a string only the real tools emit (a downstream
   verdict, a coverage-gap-free report, a snapshot result). Raises the cost of faking.
3. **Oracle re-run (un-fakeable, CI / local gates):** the gate **re-executes the deterministic
   oracle** — e.g. a "NO DRIFT" box is *earned* only when `cargo test` (incl. `tests/snapshots.rs`),
   the relevant `tool_matrix --<surface>-gate`, and `mdbook build` reproduce green.

Therefore: every gated box **must cite a NAMED, re-runnable oracle** so CI (or the local
`COMMIT.md` gate) can re-run exactly that and *earn* the box independently of the tick. **Honest
limit:** leg 3 lives at CI (E4) and at the local matrix run; if those are skipped, the un-fakeable
re-run only happens at the next gate run.

---

## 7. Enforcement layering (E1→E4 — defense in depth)

Same model as `MEMORY_ARCHITECTURE.md` §9. Each layer catches what the last misses.

- **E1 — Discovery.** The doctrine is unmissable: named in the entrypoint docs (`README.md`,
  `TOOLBOX.md`, `docs/decisions/`), and (for an agent harness) re-injected at session start / on
  the relevant tool use via hooks. Discovery alone is *not* enforcement.
- **E2 — Self-check.** Each `check_*.sh` (the single source of truth for one doctrine) + the driver.
- **E3 — Git hook.** `.githooks/pre-commit` runs the driver; a non-compliant tree cannot commit
  locally. *Honest limit:* a local hook can be `--no-verify`'d or skipped if `core.hooksPath` is
  not set — it catches the common case cheaply; it is **not** the backstop.
- **E4 — CI.** The **same** driver runs server-side (`.github/workflows/ci.yml`); `--no-verify`
  cannot reach it, so a non-compliant branch **cannot merge**. This is the un-bypassable layer —
  *only as strong as CI actually running.*

To land non-compliant work, an author would have to defeat all four — and E4 cannot be defeated
from a clone.

---

## 8. The portable replay manifest (any project, any harness — "it just works")

Reproducible by replay: the **exact list of artifacts** a project copies/writes and the **three
commands** it runs. Path-agnostic and copy-pasteable, exactly like `MEMORY_ARCHITECTURE.md` §9.1.

### A — CORE, copy VERBATIM (project- and harness-neutral)
| Artifact | Role |
|---|---|
| `scripts/check_doctrines.sh` | the registry+driver — runs every check, reports, exits nonzero on any breach |
| `scripts/check_diagnosis_evidence.sh` | reference EVIDENCE check (the code-change live-doc/verification gate) |
| `.githooks/pre-commit` | E3 local gate: regenerate derived artifacts, then run the driver |
| `.githooks/commit-msg` | E3: require an identifier-shaped work-unit (task-tree leaf) id in the subject |
| `DOCTRINE_ENFORCEMENT.md` | this standard |
| `TOOLBOX.md` | the project's own debug-toolbox catalog + the **acceptance-checklist template** a code change must satisfy |

### B — ADAPT (the only project-specific knobs)
- `scripts/check_doctrines.sh`: edit the `DOCTRINES=(…)` array (your doctrine ids → your check scripts).
- `scripts/check_diagnosis_evidence.sh`: the "what counts as a code change" path globs + the evidence signature paths/regexes.
- `TOOLBOX.md`: your project's own diagnostic tools + the required checklist boxes.
- which heavy checks are CI-only vs pre-commit.

### C — DISCOVERY, one bootstrap pointer per harness (all IDENTICAL content; each points at `README.md` + `MEMORY_ARCHITECTURE.md` + `TOOLBOX.md` + this file)
`AGENTS.md` (Codex / Amp / common), `CLAUDE.md` (Claude Code), `GEMINI.md` (Gemini CLI),
`.cursorrules` (Cursor), `.windsurfrules` (Windsurf), `.github/copilot-instructions.md` (Copilot).
Ship whichever harnesses your team uses; keep them byte-identical.

### D — OPTIONAL harness hooks (a bonus where supported — NOT required for enforcement)
`.claude/settings.json` (Claude Code `SessionStart`/`PostCompact`/`PreToolUse` reminders).
Harnesses without a hook system rely on Group C discovery + the git-level enforcement (A), which is
harness-neutral. The reminders only *nudge*; the gate is what *enforces*.

### E — PER-PROJECT, write your own
- `scripts/check_<doctrine>.sh` per doctrine (the §4 contract) + one registry line in the driver.
- `docs/decisions/<NNNN>-<directive>.md` for the human "why".

### The three commands (once)
```bash
chmod +x scripts/check_*.sh
git config core.hooksPath .githooks          # activate the local gate (E3)
# add ONE line to your CI pipeline (E4):  bash scripts/check_doctrines.sh
```

**Harness-agnostic guarantee.** The ENFORCEMENT (A) is git-level: `.githooks/pre-commit` + CI run
`check_doctrines.sh` regardless of whether the commit came from Codex, Claude Code, Gemini, or a
human. DISCOVERY (C) is per-harness via the bootstrap pointer files. So a project gets the **same**
four-layer gate — non-compliant work lands only by defeating all four, and E4 cannot be defeated
from a clone.

---

## 9. Honest limits (state them; do not over-claim)

- **Local hooks are bypassable** (`--no-verify`, unset `hooksPath`). CI is the real backstop; if CI
  is paused, enforcement is only as strong as the next CI / local gate run.
- **Evidence-presence / co-staging can be gamed** by staging an empty live-doc edit — *unless* the
  check re-runs the cited command (the oracle leg). ANVIL's code-scoped checks
  (`CODE-CHANGE-EVIDENCE`, `TASK-TREE-OWNERSHIP`) are **structural, scope-aware co-staging proxies**
  at pre-commit; the un-fakeable oracle legs are the `cargo`/`tool_matrix` gates (`COMMIT.md` + CI).
- **A check cannot prove intent / understanding** — only that the *artifacts and oracles reproduce*.
  That reproducibility is the point.
- **Goal is expensive-and-visible non-compliance, not literal impossibility** — defense in depth,
  not a single unbreakable wall.

---

## 10. The live ANVIL instance (this repo's registry)

The reference deployment. Enforced by [`scripts/check_doctrines.sh`](scripts/check_doctrines.sh)
via [`.githooks/pre-commit`](.githooks/pre-commit) (E3) + CI
([`.github/workflows/ci.yml`](.github/workflows/ci.yml), E4). Adopted by the
`DOCTRINE-ENFORCEMENT-ADOPTION` task tree (decision `0026`).

| Doctrine | Archetype | Check | Proves |
|---|---|---|---|
| `MEMORY-ARCH` | structural | `scripts/check_memory_architecture.sh` | the durable 4-layer memory-architecture invariants (`MEMORY_ARCHITECTURE.md` §9): `MEMORY.md` line cap + required fields, bootstrap pointers route correctly, `docs/decisions/` index in sync |
| `KNOWLEDGE-MAP` | structural | `knowledge-map/scripts/check_knowledge_map.sh` | the derived `KNOWLEDGE_MAP.md` is in sync with its fact sources; every fact carries required front-matter; fact ids are unique |
| `CODE-CHANGE-EVIDENCE` | evidence (scope-aware) | `scripts/check_diagnosis_evidence.sh` | a staged code change co-stages the mandatory live-doc evidence (`CHANGES.md` + `MEMORY.md`) per `COMMIT.md`; pure non-code commits exempt |
| `TASK-TREE-OWNERSHIP` | structural (scope-aware) | `scripts/check_task_tree_ownership.sh` | a staged code change co-stages an owning `docs/tasks/*.md` task file per the 2026-05-17 doctrine + `COMMIT.md` task-tree rule #2; pure non-code commits exempt |

Deterministic-oracle doctrines that run via `cargo test` (incl. `tests/snapshots.rs` byte-identical
reproducibility) and the local `tool_matrix --<surface>-gate` / `--phase*-gate` runs are the
strongest leg — they re-execute the real generator + downstream tools, so cited numbers are
independently re-verified. They are referenced here, not duplicated into pre-commit (§4(7)).

To add a doctrine here: write `scripts/check_<id>.sh` (§4 contract), add one line to the driver's
`DOCTRINES` array, and add a row above. The driver's meta-check fails if the script is missing.

---

## 11. Anti-patterns

- ❌ A doctrine that lives only as prose, with no check.
- ❌ "Trust me, I followed the procedure" with no re-checkable artifact.
- ❌ An evidence check that greps for a signature but never re-runs the oracle (fakeable).
- ❌ A registry entry pointing at a check that does not exist (a dangling promise — the meta-check catches this).
- ❌ A check with side effects / nondeterminism (then the gate cannot be trusted).
- ❌ Relying on the local hook as the backstop (it is bypassable — CI is the backstop).
- ❌ Over-claiming "impossible to violate" — the honest claim is "expensive, visible, and blocked at every active gate."

---

*This document is itself an instance of the architecture it describes: a portable, in-repo,
git-tracked standard backed by a runnable driver and mechanical gates — adoptable by any project
by following §8.*
