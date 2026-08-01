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
Ship whichever harnesses your team uses; keep their **body** identical (each may
carry its own title line — ANVIL's five differ only there, measured
`2026-07-31`). Cite doctrine provenance by **owner + date**, never by one of these
files: a rule that binds every author must not appear to rest on what a single
harness was told.

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
- **A "does the doc name every id" check can pass VACUOUSLY**, and it does so exactly where it is
  needed most. Its strength is inversely proportional to how ordinary its ids are as *words* in the
  document being checked — and ids are most ordinary in the very document that documents them.
  Measured in this repo (`2026-07-31`, decision `0037`): **3 of `ENUMERATION-PARITY`'s 10 coverage
  sites passed with the checked enumeration deleted outright**, because `verilator` / `yosys` /
  `sharing` / `state` are ordinary vocabulary in the chapters that list them; the sites that survived
  the probe did so on an accident of vocabulary (`MEMORY-ARCH`, `README-GROWTH` appear nowhere but
  their list). **The general acceptance test for any coverage-shaped check is therefore: delete the
  subject and re-run it.** A check that still passes with the thing it checks removed is checking
  nothing — and per §6.1 it is *manufacturing* the confidence, not earning it.
  **Repaired** at `LIVE-DOC-REGISTRY-SHADOWS.3`: each site marks its enumeration with an invisible
  inline HTML-comment fence carrying the set id, the check reads only inside it, and a **missing
  fence is a hard failure** — so the predicate cannot silently degrade back to whole-file matching.
  The marker names **no members**, so it is not itself a shadow. Two limits stay stated: the check is
  **coverage, not exact parity** (a fence that must enclose prose-bearing list items is not
  losslessly extractable, and nothing is ever retired here anyway), and **a fence must contain the
  enumeration and nothing that merely mentions its ids** — commentary inside a fence re-imports the
  vacuity at fence scale, which is how this repo's own honest-limit paragraph produced the single
  wrong pass in a 98-control sweep.
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
| `MEMORY-ARCH` | structural | `scripts/check_memory_architecture.sh` | the durable 4-layer memory-architecture invariants (`MEMORY_ARCHITECTURE.md` §9): `MEMORY.md` line **and byte** caps + required fields, bootstrap pointers route correctly, `docs/decisions/` index in sync. **Why two axes:** a line cap bounds a *projection* of a file, and content flows into the axis nobody measures. Measured at `OVERFLOW-DESTINATION-INSTRUMENTATION.2` (decision `0040`): after the 50-line cap landed (`2d01e8e`, `2026-06-05`, truncating 2,399 lines / 306,099 B to 19 / 1,227 / **64 B-per-line**), the line count went 19 → 50 and **stopped** while bytes went **×16.6** and density **64 → 406 B/line** against `README.md`'s 65 — the cap binding perfectly on the axis it measured for two months while the file reached **1.65×** the byte cap its sibling landing page is held to, with **65 %** of it layer-C content the standard assigns elsewhere. The byte cap (**6,144**) is a **second assertion inside this check, not a ninth doctrine** — `MEMORY-ARCH` already owns this file's size, and a second mechanism for one job is what `feedback_full_factorization` forbids. It is **derived, not fitted** (decision `0040` §(c)): the contract's *one screen* at ≤ 50 lines × the demonstrated-achievable 64 B/line, budgeted to ~120, i.e. deliberately **half** `README.md`'s 12,288, a resume pointer being a strictly smaller contract than a landing page. Fails with a **routing hint whose destinations are themselves classified** — layer B and layer C are append-only records that are *supposed* to grow — so the hint does not reproduce the hole it exists to close. **Raising the cap is not a fix**: it requires a new decision record stating the resume-pointer contract expanded, never an edit to the constant |
| `KNOWLEDGE-MAP` | structural | `knowledge-map/scripts/check_knowledge_map.sh` | the derived `KNOWLEDGE_MAP.md` is in sync with its fact sources; every fact carries required front-matter; fact ids are unique |
| `CODE-CHANGE-EVIDENCE` | evidence (scope-aware) | `scripts/check_diagnosis_evidence.sh` | a staged code change co-stages the mandatory live-doc evidence (`CHANGES.md` + `MEMORY.md`) per `COMMIT.md`; pure non-code commits exempt |
| `TASK-TREE-OWNERSHIP` | structural (scope-aware) | `scripts/check_task_tree_ownership.sh` | a staged code change co-stages an owning `docs/tasks/*.md` task file per the 2026-05-17 doctrine + `COMMIT.md` task-tree rule #2; pure non-code commits exempt |
| `EVIDENCE-CITATIONS` | structural | `scripts/check_evidence_citations.sh` | every cited evidence bank is either **digest-backed** (a schema-valid `docs/evidence/<bank>.md`) or explicitly classified in `docs/evidence/INVENTORY.md` — decision `0030` + its `2026-07-30` amendment. Three buckets, **fail-closed**: an unclassified `anvil-<name>` token is a breach. **§1 (grandfathered) is pinned by entry count and membership SHA-256** because it is a historical fact — the banks that existed before `0030` — so it cannot grow; unpinned it would be the escape hatch that makes the doctrine decorative. §2 (not-evidence: binaries, directories, prose) is unpinned because ANVIL's vocabulary legitimately grows. The check **classifies rather than pattern-matches**: after `VOLUME-DATA-LOCALITY.5` removed the `/tmp/` prefix, no lexical rule separates a bank name from a binary name, and a guessing gate can only fail by ignoring a real bank or crying wolf on prose |
| `ENUMERATION-PARITY` | structural | `scripts/check_enumeration_parity.sh` | every **declared** docs/script enumeration pair is in parity with the set it mirrors — decision `0033` (`SHADOW-ENUMERATION-SWEEP.7`). A *shadow enumeration* is a hand-maintained list that mirrors an already-authoritative set and so falls silently out of date when that set grows; rule (a) classifies one as **derivable ∧ growth-coupled ∧ silent**. Rust-side pairs are held by in-crate `#[test]`s (already gated by `cargo test` + CI) — a shell doctrine for them would be a second mechanism for one job — so this check holds **only** the docs/script pairs, which have neither a compiler nor `cargo test`. The declared pairs: §10's own table ↔ the `DOCTRINES` registry; `book/src/SUMMARY.md` ↔ `book/src/*.md` (mdBook renders only what `SUMMARY.md` links, so an unlinked chapter is written and **never rendered**); the two book chapters documenting the downstream-tool allow-list ↔ the adapter registry; the live docs that enumerate the `--steer` category taxonomy ↔ `KnobId::category`'s match arms (`COVERAGE-STEERED-GENERATION.4c` — a stale copy there is worse than an omission, because `--steer` *errors* on an unknown key, so a user reading a short list simply never learns the category exists and the feature ships invisible; `README.md` left that pair at `README-POLICY-ADOPTION.2` when the landing page stopped enumerating knob taxonomies — decision `0033`'s R1 repair-by-deletion, not a weakening, since the copy it guarded no longer exists); and `USER_GUIDE.md`'s CLI flag table ↔ the knob flags `cli_overrides` projects onto `config::Overrides` (`USER-GUIDE-CLI-TABLE-SHADOW.3` — the table's contract, recorded above it, is *exhaustive over the knobs*, so a new knob without a row is a defect; **the set comes from the source, never from `anvil --help`**, because a doctrine check reads the repository rather than a built binary, and because `--help` is prose that quotes *other* commands' flags — scraping it is what made `.1` count 108 top-level flags where 107 exist and report two live `anvil hunt` options as deleted. The extractor **strips all whitespace before matching**, and that is load-bearing rather than tidy: `rustfmt` had already split `cli` from `.hierarchy_registered_sibling_mixed_support_prob` across two lines, so the naive line-wise scan reads **91 of 92** — the `PARITY-EXTRACTOR-ARM-SHAPE-GAP` defect, live, caught by a control before it shipped); and `USER_GUIDE.md`'s **`tool_matrix` option list** ↔ the options that binary's clap `Cli` **declares** (`USER-GUIDE-CLI-TABLE-SHADOW.6` — a *second* binary's registry, needing its own extractor because `tool_matrix` has no `Overrides` projection at all. **The only pair held at EXACT parity rather than coverage, and the reason is measured, not stylistic:** `covers_fenced_set` asks *"is this id named inside the fence?"*, and over this region that predicate is **vacuous for 10 of the 35 options** — the gate bullets legitimately cross-reference each other, `--iverilog-compile` **eleven** times, so deleting its own entry leaves ten matches behind and the check stays green while the reader loses the definition. Probed both ways on the same mutated file: the coverage predicate **passes** with the entry deleted, the bullet-head predicate **fails**. Extracting *bullet heads* turns the shadow side into a derived set, which admits exact parity because prose cross-references are not heads — so the three foreign tokens in the region (`--ast-json`, `--binary`, `--language`, other tools' flags quoted correctly) cannot cry wolf. A `long = "…"` spelling override is a **hard failure**, not a silent miss: the field name would no longer be the flag name and the derivation would publish a confident wrong set); and `USER_GUIDE.md`'s **`anvil hunt` flag table** ↔ `HuntCommand` in `src/main.rs` (`USER-GUIDE-CLI-TABLE-SHADOW.7` — the **third and last** clap registry, and the census that establishes "last" was **run**, not assumed: `src/` holds exactly three `#[derive(Parser)]` structs, all now gated, while `anvil-mcp` hand-parses a single `--http` with no registry to derive from and is stated out of scope rather than left silent. `.3` had declined this table for a reason true of *pair 5's* extractor — `HuntCommand` has no `Overrides` projection — which **pair 6 removed** one commit earlier by reading a clap struct directly; a recorded *"we can't because X"* goes stale the moment something removes X, and nothing re-examines it on its own. Vacuity here is **denser** than pair 6's — **7 of 10**, because the section opens with four runnable `anvil hunt …` examples that name the flags they demonstrate, a second and independent source of the same failure. **Nothing was behind:** the table measured 10/10, so this pair mechanizes rather than repairs, and says so. Pairs 6 and 7 are **two argument lists over one `clap_struct_pair` helper**, not two copies — a fork would have to be re-taught each lesson above one at a time).

> **A negative control must prove its sabotage landed.** Recorded at `.7` because it nearly cost a control: a mutation whose `perl` substitution silently failed to match left the tree unchanged, so the check passed — **indistinguishable from a control that failed to fire.** A control proves the check *can* fire; the §9 vacuity probe proves it fires on the *right* input; asserting the mutation applied proves *the experiment ran at all*. All three are needed, and only the third is routinely skipped. *(No count is given, deliberately: a number beside a list is one more copy of it, and decision `0033` repairs that by **deletion** rather than by gating the count too — the script's `PAIRS` table is the authority. This cell said "the **five** live docs" until `README-POLICY-ADOPTION.2` made it four: the anti-pattern this sentence warns about had reappeared **inside the sentence that warns about it**.)* **Deliberately not a shadow DETECTOR** (§9): rule (a)'s first test is a *semantic* relation between two sets, so a syntactic detector would have to already know the pairing — and its only failure modes are *miss* (false confidence) and *cry wolf* (and a gate that cries wolf gets deleted, taking its real coverage with it). It therefore holds **classified pairs**, declared in the script's `PAIRS` table. **That table is authoritative, not a shadow** — no set in the repo enumerates "which lists shadow which sets", so test (1) fails and the mechanism does not recurse. Every extraction is **count-floored**, because an extractor that silently matches nothing would make the gate pass vacuously — the exact failure mode the doctrine exists to remove |
| `README-GROWTH` | structural | `scripts/check_readme_growth.sh` | `README.md` stays a **landing page**: within the reviewed line **and** byte caps, with the project-owned `README_POLICY.md` present beside it. Owner directive `2026-07-30` (*"make sure README.md doesn't grow, grow and grow"*), recorded in the project-owned `README_POLICY.md` and designed at decision `0036`. Cited by owner+date, not by a harness bootstrap file: this rule binds every author and is enforced at git level (`README-POLICY-PROVENANCE.1`). **Why a cap and not a style rule:** the growth was *structural* — the workflow asked every new knob for a README bullet, so the file reached **1771 lines / 122,767 bytes**, 92 % of it a lossy copy of `docs/decisions/`, `book/src/`, `USER_GUIDE.md` and `ROADMAP.md`. A review habit cannot hold that back; a number that fails the commit can. **Why both caps:** they catch different bloat, measured at `README-POLICY-ADOPTION.2` — the surviving landing page was 10,297 bytes across 141 lines, so the projected file sat at 74 % of the *line* cap while already exceeding the *byte* cap; prose density is the hidden variable (118 B/line for a numbered list vs 57 for a path list). **The caps are derived, not chosen to fit** — decision `0036` §(c) took them from what survived the audit and set them deliberately **below** `README_POLICY.md`'s illustrative `300`/`16,384`, which against a 141-line survivor would leave 60 % room to regrow into. The check is authoritative and the policy document **cites** it rather than restating the numbers (decision `0033`: a number beside the thing that defines it is one more copy of it). **Not scope-aware, deliberately:** landing-page size is a property of the tree, not of a change, so a commit that does not touch `README.md` is still checked — otherwise an over-cap README could arrive via a revert or a merge and never be re-examined. Fails with a **routing hint** naming the canonical home per kind; raising a cap is not a fix and requires a new decision record stating that the landing-page contract itself expanded |
| `NO-BOOT-VOLUME-REFS` | structural | `scripts/check_no_boot_volume_refs.sh` | no tracked file stores on or points at the boot volume — an **absolute** `/tmp/`, `/private/tmp`, `/var/folders`, or `~/tmp/` path — and `std::env::temp_dir()` is named in exactly one file — `src/paths.rs`, the runtime resolver (decision `0031`). **Its allow-list is load-bearing:** policy documents keep the strings they forbid (a doctrine cannot state what it prohibits without naming it), and `CHANGES.md` / `DEVELOPMENT_NOTES.md` are append-only history the owner has directed must stay raw — a check that flagged them would pressure authors into the history rewrite the project forbids. **Its leading anchor is equally load-bearing:** the bare substring `/tmp/` also sits inside repo-*relative* paths that are on-volume by construction — above all `target/tmp/…`, Cargo's `CARGO_TARGET_TMPDIR` — so a banned shape counts only where a path can start (`CARGO-TMPDIR-SWEEP-REGRESSION.1`) |
| `CHANGES-ENTRY-PLACEMENT` | structural | `scripts/check_changes_entry_placement.sh` | a newly added `CHANGES.md` entry is at the **top** of the file: *if the staged `CHANGES.md` diff adds at least one `## ` heading, the first `## ` heading in the resulting file must be one of those added lines.* No date, no hash, no knowledge of the ordering convention — decision `0045`. **Keyed on the authoring path, because both content-keyed designs were measured dead first:** a **date**-keyed ordering scan *cries wolf* (3 findings, **2 false** — mis-dated headings above correctly-ordered entries), and a **hash**-keyed one is *vacuous* (**0 of 3** real offenders visible). The hash failure is the deep one and yields a transferable rule: **a detector must not depend on a field that the defect it detects also destroys** — the stale entry template that misplaces an entry is the same one that omits its `Landed as:` line. **Measured before registration** over every commit touching `CHANGES.md` (766 at `928817f`): 664 ok, **99 correctly skipped** (they add no entry — backfills and typo fixes, which are the whole false-alarm risk), **3 fires, all true positives, 0 false alarms** — and it found a third offender (`f9cf50a`, heading **6 of 379**) that three prior leaves had missed. **Not scope-aware, deliberately:** both original offenders were **docs-only** commits, and that exemption is exactly what let them through; placement has nothing to do with whether a commit touches code. Its one exemption is intrinsic rather than declared — a commit adding no `## ` heading has no subject and is silent. It does **not** check ordering, does **not** require a `Landed as:` hash (the newest entry structurally cannot carry its own, so such a check would be permanently red), and does **not** license moving a **landed** entry — decision `0038` stands, and what the failure message asks you to move is the entry still in your index |
| `TABLE-RENDER-FIDELITY` | structural | `scripts/check_markdown_tables.sh` | no tracked `*.md` table row over-splits past its header. A GFM row splits on every **unescaped** `\|` *before* inline parsing — a code span gives a pipe no protection — and the spec then **ignores** cells beyond the header's column count, so that content sits in the source and reaches **no rendered page**. Measured at `OVERFLOW-DESTINATION-INSTRUMENTATION.6` over 243 tracked `*.md` (394 tables, 2,310 data rows): **36** malformed rows dropping **57,283** characters, including **5** cells that were a row's own link to its detail file — and the 24,990-byte line three leaves had cited as a *size* exhibit was dropping **97.0 %** of itself. **A ninth doctrine rather than an assertion inside an existing one**, and the `feedback_full_factorization` test was applied rather than waved through: that rule forbids a *second* mechanism for a job that already has one — which is exactly why `.3` put `MEMORY.md`'s byte cap *inside* `MEMORY-ARCH` — but no registered doctrine owns markdown **well-formedness**, so the subject is unowned and registers on its own. **Escape-aware** (a naive pipe count yields 3 false positives out of 8 on one file) and **fence-aware** (proven load-bearing: a fenced pseudo-table passes, and the same tree fails once the fence handling is neutered). Deliberately **silent on rows with fewer cells** — the spec pads those and nothing is lost; a gate that also cried wolf on untidiness gets argued with instead of obeyed. |

| `BOOK-LINK-TARGETS` | structural | `scripts/check_book_link_targets.sh` | every markdown link target in `book/src` resolves to a real file **inside** `book/src` — decision `0046` (`BOOK-LINK-INTEGRITY.3`). **mdBook rewrites every `.md` target to `.html`**, so `[USER_GUIDE.md](../../USER_GUIDE.md#…)` renders as `../../USER_GUIDE.html`, which does not exist: **alive on GitHub, dead in the rendered book** — the surface the owner reviews — and **`mdbook build` exits `0` on it**. **The ESCAPE half is the load-bearing one, and the obvious check misses it:** *"does the link target exist?"* **passes this very defect**, because `../../USER_GUIDE.md` *does* exist; what is broken is a rendered target that leaves the build dir. So the check tests **escape first, existence second**. **Source-level, therefore portable:** it needs no `mdbook` — the tool ANVIL does not vendor, whose absence would make a rendered-output check *skip on a fresh clone*, the exact failure `USER-GUIDE-CLI-TABLE-SHADOW.3` rejected for `anvil --help`. **Count-floored with a DERIVED floor, not a fitted one:** `SUMMARY.md` must link every chapter for mdBook to render it, so the book structurally cannot hold fewer than *(chapters − 1)* links; the floor self-adjusts as chapters are added and today's real count clears it ~8×. Neutering the extractor makes it **fail on the floor** rather than pass vacuously. **The extractor is whole-file, not line-wise, and that is measured rather than stylistic:** a markdown link's *text* may wrap across a newline — **2** such links exist in this book — and the first draft, scanning line by line, **silently skipped them**; a wrapped link that escaped would have sailed through. Proven load-bearing by reverting the fix: line-wise **misses** a wrapped escape (exit 0) where whole-file **catches** it. *(It took three attempts to run that control — the first two mutations silently failed to apply and the check "passed", exactly the `.7` trap of a control that never ran.)* **Fence-aware**, and proven so both ways: a dead link inside a ``` fence is an example and stays silent, and the same tree **fails** once the fence mask is neutered. **Not scope-aware, deliberately** — link integrity is a property of the *tree*, like `README-GROWTH`'s: a dead link can arrive by revert, by merge, or by a rename in a commit that never touches the linking file. **Anchors are deliberately OUT of scope, stated rather than implied** (§9): resolving a `#fragment` at source level means reimplementing mdBook's heading slugifier, and a wrong slug model cries wolf — while the anchor class was *measured* empty at `.1` (**0** dead of **71** authored / **1136** rendered, negative-controlled). That is a claim about present risk, not permanent immunity. **A doctrine rather than an assertion inside an existing one:** `TABLE-RENDER-FIDELITY` owns table well-formedness, `ENUMERATION-PARITY` owns declared list pairs — its book pair holds `SUMMARY.md` against the chapter files, i.e. which chapters are **linked**, never whether a link **resolves** — and the rest own file size, staging, paths and citations. The subject is unowned, so this is the first mechanism, not a second |

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
