# NEGATIVE-CONTROL-HARNESS: "prove the sabotage landed" is carried as a habit, and habits fail

## Metadata

- Tree ID: `NEGATIVE-CONTROL-HARNESS`
- Status: `active`
- Roadmap lane: Workflow / gate quality
- Created: `2026-08-01`
- Last updated: `2026-08-01` (`.1` **done** — the record measured; frontier `.2`, the carrier decision)
- Owner: repo-local workflow

## Goal

A negative control is how this repo proves a gate can actually fire. Three legs are needed, and
`USER-GUIDE-CLI-TABLE-SHADOW.7` recorded that only the third is routinely skipped:

1. a control proves the check **can** fire,
2. the `DOCTRINE_ENFORCEMENT.md` §9 vacuity probe proves it fires on the **right input**,
3. asserting the mutation **landed** proves **the experiment ran at all**.

Leg 3 is the one that fails silently, and it fails in the dangerous direction: a substitution that
does not match leaves the tree unchanged, the check passes, and that is **indistinguishable from a
control that correctly did not fire**. The author reads it as a finding about the gate.

**The lesson is already written down, and it still cost two false results.** That is the subject
of this tree — not the lesson, which is fine, but its *carrier*.

## How it was found (`2026-08-01`, at `9ad7385`)

Not by a sweep. `BOOK-LINK-INTEGRITY.3` had to prove that its new check's whole-file extractor was
load-bearing, by reverting it to the line-wise form and confirming the gate then **misses** a
wrapped escape. **That single control took three attempts to actually run.** Twice the mutating
substitution silently failed to apply — once a `sed` whose escaping was wrong, once a `perl -pe`
whose was — and both times the check "passed", which is exactly the reading the rule warns about.

The distribution of the failures is the finding:

| where the probe was written | leg-3 failures |
| --- | ---: |
| inside the reusable harness (`.cache/book_link_controls.sh`, whose `probe` helper asserts its marker before reading any verdict) | **0** of 10 |
| ad-hoc, inline, outside the harness | **2** of 3 |

Same author, same leaf, same hour, same rule in mind. The variable was not diligence; it was
**whether the assertion was structural or remembered**.

## The measurement (`.1`, `2026-08-01`, at `da73827`)

### Region and denominator, stated before any number

- **Region: all 80 `docs/tasks/*.md`, every section** — deliberately **not** the `## Verification
  Log` sections the acceptance named. Of the **180** control mentions in that corpus, only **30
  (17 %)** sit inside a `## Verification Log`; the rest are in the `Verification:` field of a
  `## Task Tree` node, in `## Decisions`, or in `## Changelog`. Scoping to verification logs, as
  written, would have measured **one sixth** of the record and reported it as the whole. The
  acceptance is corrected here rather than quietly followed.
- **Files carrying a control record: 19.** `S` was **derived twice**, and the second derivation
  changed the answer: the narrow term set (`negative[- ]control|sabotag`) returns **18**, and a
  broadened set adds `PARITY-EXTRACTOR-CHARSET-GAP.md`, which records **four** controls without
  ever writing the word *negative*. The broadened set's 20th file,
  `SEMANTIC-INTROSPECTION-EXPANSION.md`, is a **false positive** — *"control ports"* — and is named
  so the widening's cost is on the record too, not only its yield.
- **Unit of count: a leaf-episode** — one leaf's recorded control activity, which is the granularity
  at which leg 3 is or is not reported. **39 leaf-episodes.**
- **Individual probes, as the record itself counts them: ~240**, of which **101** belong to one leaf
  (`LIVE-DOC-REGISTRY-SHADOWS.3`: 98 single-id drops, 2 empty-fence probes, 1 baseline). These are
  the record's **own** numbers, harvested from its prose; they are reported counts, not events this
  leaf independently re-ran.

Instrument, stated precisely enough to re-run: `TERM = negative[- ]control(s|led|ling)?|sabotag(e|ed|ing)`
over `docs/tasks/*.md`, matches attributed to the enclosing `## ` heading, each match expanded to a
±700-character window and classified **by reading all 114 windows** — not by counting keywords.

### The answer: leg 3 is visibly reported in 2 of 39 leaf-episodes

| leaf-episode | what the record says about leg 3 |
| --- | --- |
| `BOOK-LINK-INTEGRITY.3` | *"10 controls over `scripts/check_book_link_targets.sh`, **each asserting its mutation LANDED before reading the verdict** (`.cache/book_link_controls.sh`)"* |
| `USER-GUIDE-CLI-TABLE-SHADOW.7` | control 5 *"did not match the source and so sabotaged nothing — caught by asserting the substitution count"* |

**The other 37 are unknowable, and unknowable is the honest word.** Not "absent": nothing in those
records distinguishes *asserted and fine* from *never checked*. Inferring compliance from silence is
exactly what this leaf was told not to do.

Both reporting leaves are dated **`2026-08-01`** and both are the leaves that *produced* the rule.
**No leaf recorded leg 3 before the day the rule was written, and none has since.** The two
mentions in this tree's own file are the rule being **stated**, not performed, and are excluded.

### The asymmetry that explains it — the finding

The same primitive — *compare the file against a known baseline* — is run **religiously on the way
back and once on the way in**:

| direction | what it proves | where it appears |
| --- | --- | ---: |
| revert verified (`restored byte-exact`, `verified with cmp`, `sha256 identity`, `byte-identical to HEAD`, `git diff --stat, empty`, `git checkout-index -f`, on-volume backup) | the tree is clean again | **6** files, **27** occurrences |
| mutation verified, before the verdict is read | the experiment ran at all | **2** leaves |

**Why the habit points that way, and it is not carelessness.** A failed *revert* leaves a **dirty**
tree, and three separate mechanisms punish that: the pre-commit driver, `COMMIT.md` §10's
`git status` review, and the pivot rule that forbids switching trees on a dirty repo. A failed
*mutation* leaves a **clean** tree — which every one of those mechanisms reads as success. The repo
verifies its edit at the end, where a gate is watching, and skips the identical check at the start,
where none is. **Diligence tracks the gates, not the experiment.** That is a stronger explanation
than "the author forgot", and it is testable: it predicts that leg 3 will keep being skipped for
exactly as long as nothing looks at it, however many times the rule is written down.

### The mechanism, measured rather than asserted

Measured on this host at `da73827`, on a two-line scratch file under `.cache/`:

| command | matched? | exit | file |
| --- | --- | ---: | --- |
| `sed -i.bak 's/NOMATCH/x/' f` | no | **0** | unchanged (`cmp` identical) |
| `sed -i.bak 's/alpha/ALPHA/' f` | yes | **0** | changed |
| `perl -pi -e 's/NOMATCH/x/' f` | no | **0** | unchanged |
| `perl -pi -e 's/alpha/ALPHA/' f` | yes | **0** | changed |
| `perl -pi -e 'BEGIN{$c=0} $c+=s/NOMATCH/x/g; END{exit($c?0:9)}' f` | no | **9** | unchanged |
| `perl -pi -e 'BEGIN{$c=0} $c+=s/alpha/ALPHA/g; END{exit($c?0:9)}' f` | yes | **0** | changed |

**The mutation primitive reports the same success for *"I changed nothing"* and for *"I changed
exactly what you asked"*.** Asserting the substitution count is the whole difference. All **three**
recorded leg-3 failures in this repository are textual substitutions — two `sed`/`perl -pe` escaping
errors in `BOOK-LINK-INTEGRITY.3`, one non-matching `perl` at `USER-GUIDE-CLI-TABLE-SHADOW.7` — and
**no recorded failure involves any other mutation primitive.** This answers Open Question 3: the
subject *is* narrower than "controls".

*Recorded because it is the same class, live, one level up:* the **first** run of this very probe
mis-fired — `sed -i '' …` was mis-parsed here and exited **2** — and would have supported the same
conclusion for the wrong reason. It was re-run correctly before any number above was written down.

### The negative space: mutation primitives that cannot silently no-op

`.2` needs these named, because they bound what a carrier has to cover:

- **in-language** — a statement inside a `#[test]`: `src/ir/case_qualifier.rs` breaks its fixture
  with `m.nodes[4] = Node::Constant { value: 3, width: 2 };` **after** asserting *"the disjoint
  fixture is clean before the arms are broken"*. There is no *match* step, so there is nothing to
  fail silently;
- **compiler-checked** — the landing *is* the diagnostic: `E0624`, `E0616`, `E0004`, `E0599`;
- **constructed** — a fixture built from nothing: `README-POLICY-ADOPTION.3`'s isolated on-volume
  fixture root at exact line and byte counts *that the check then echoes back*;
  `OVERFLOW-DESTINATION-INSTRUMENTATION.6`'s probe table;
- **git-derived** — the *before* side taken from `git show HEAD:` rather than made by editing
  (`LIVE-DOC-REGISTRY-SHADOWS.1`).

**The risk is not a property of negative controls. It is a property of one mutation primitive.**

### The second half of `.1`: is there a commit-visible artifact for a check to read?

A control's mutation is reverted before the commit **by construction** — the pivot rule and the
pre-commit driver both demand a clean tree — so the mutation is never *in* the commit. What is
commit-visible splits three ways, and only the first is checkable:

1. **Controls that ship as code** — re-runnable forever. They exist today, measured, not assumed:
   **4** `#[test]`s whose doc comment declares them a negative control
   (`src/ir/case_qualifier.rs::parallel_violations_fires_on_overlapping_casez_arms`,
   `::parallel_violations_fires_on_arm_labels_that_do_not_fit`,
   `::full_violations_fires_on_a_default_less_case`,
   `tests/book_examples.rs::harness_detects_a_broken_command`), **1** shipped `--self-test`
   (`scripts/evidence_digest.sh`), and **2** shipped **test seams** (`DOCTRINE_STAGED_OVERRIDE` in
   `check_diagnosis_evidence.sh` and `check_task_tree_ownership.sh`) that feed a check a synthetic
   input **without touching the tree** — a mutation-free control, for which leg 3 does not exist.
2. **Controls that cannot ship.** A probe that must *not compile* is the clearest case:
   `src/ir/knob_roll.rs` carries *"`E0624: method 'record' is private`, verified by negative control
   at `.3b`"* as **prose**, because committing the probe would break the build. Every
   gate-neutering probe is in this class too — shipping it would disable the gate.
3. **The prose record itself.** A check over it could assert only that a leaf claiming a control
   also claims a landing assertion. The author writes both, so it is `DOCTRINE_ENFORCEMENT.md` §3's
   **weakest** evidence archetype and §6.1's self-tick one level up.

**So an artifact exists for class 1 only.** `.2` must decide against that, not against a hoped-for
artifact — and the honest reading is that a doctrine check over classes 2 and 3 would be gating a
claim rather than a fact.

## The honest obstacle, stated up front

**A control runs *during* a leaf and may leave no artifact in the commit.** Every doctrine in this
repo checks *repository state*; a check cannot see that an experiment was conducted correctly an
hour before the commit. So this may be **not doctrine-shaped at all** — the reflex to reach for
`scripts/check_*.sh` should be resisted until `.2` decides. A `TOOLBOX.md` instrument, or a
sourceable helper that makes the correct thing the easy thing, may be the honest answer. So may
"nothing" (`DOCTRINE_ENFORCEMENT.md` §9).

## Why a new tree rather than a leaf of an existing one

`feedback_full_factorization` applied, not waved through — and the search was **run**, not assumed:

- `DOCTRINE_ENFORCEMENT.md` §6.1 (*a box is EARNED, not ticked*) is the nearest neighbour and does
  **not** cover this. It governs a **gated checklist box** citing a named re-runnable oracle. A
  negative control is not a gated box; nothing ticks, and its output is a verdict about the *gate*,
  not about the change.
- `ENUMERATION-PARITY` owns declared list pairs. `TABLE-RENDER-FIDELITY` owns markdown
  well-formedness. `BOOK-LINK-TARGETS` owns book link targets. None touches control conduct.
- `grep` over `scripts/` for an existing sabotage/mutation helper returns **nothing**. There is no
  mechanism to extend, so this would create the first, not a second.

## Non-Goals

- **Not a re-statement of the rule.** It is already recorded (`USER-GUIDE-CLI-TABLE-SHADOW.7`,
  `DEVELOPMENT_NOTES.md`). Writing it down again is precisely the remedy that has already failed.
- **Not a mandate that every leaf run controls.** Controls are for gates and instruments, not for
  every edit.
- **No code change** in the generator sense; this is workflow tooling and docs.

## Acceptance Criteria

- `.1` measures rather than asserts: how many recorded control episodes exist across
  `docs/tasks/*.md` verification logs, and in how many is there **any way to tell** whether leg 3
  was performed. The denominator is stated, per decision `0039`. A likely and legitimate outcome is
  *"unknowable from the record"* — which is itself the finding, and is stronger than a guess.
- `.2` decides the **carrier** before building anything, with each candidate's failure mode stated:
  a sourceable `scripts/` helper (makes the right thing easy; cannot compel use), a `TOOLBOX.md`
  instrument (discoverable; still a habit), a doctrine check (structural — *if* an artifact exists
  for it to read, which `.1` must establish), or nothing.
- If a mechanism lands it obeys `DOCTRINE_ENFORCEMENT.md` §4 and is negative-controlled both
  ways — **including a control on the control**: the harness must fail loudly when its own marker
  assertion is removed.
- `scripts/check_doctrines.sh` stays green; docs/workflow-only ⇒ DUT byte-identical.

## Task Tree

- ID: `NEGATIVE-CONTROL-HARNESS`
  Status: `active`
  Goal: `Decide whether "prove the sabotage landed" can be carried by something other than the author's attention, and if so by what.`
  Children: `.1` (measure the record), `.2` (decide the carrier), `.3` (build it, or record why not)

- ID: `NEGATIVE-CONTROL-HARNESS.1`
  Status: `done`
  Goal: `Measure how the repo's recorded control episodes report leg 3, and whether any commit-visible artifact exists that a check could read. No mechanism in this leaf.`
  Acceptance: `A stated denominator over docs/tasks/*.md verification logs, not a sample. Names the episodes where leg 3 is visibly reported, visibly absent, or unknowable. "Unknowable from the record" is a legitimate and likely result — say so plainly rather than inferring compliance from silence.`
  Verification: `done — see "The measurement" above. REGION CORRECTED, not quietly followed: only 30 of 180 control mentions (17 percent) sit in a "## Verification Log", so the acceptance's own region would have measured one sixth of the record; the region used is all 80 docs/tasks/*.md, every section. S DERIVED TWICE and the second derivation changed it: 18 files by the narrow term set, 19 once broadened (PARITY-EXTRACTOR-CHARSET-GAP.md records four controls without ever writing "negative"), with the widening's one false positive named too (SEMANTIC-INTROSPECTION-EXPANSION.md, "control ports"). DENOMINATOR: 39 leaf-episodes, ~240 self-reported probes, 101 of them in one leaf. ANSWER: leg 3 is visibly reported in 2 of 39 — BOOK-LINK-INTEGRITY.3 ("each asserting its mutation LANDED before reading the verdict") and USER-GUIDE-CLI-TABLE-SHADOW.7 ("caught by asserting the substitution count") — both dated 2026-08-01, both the leaves that PRODUCED the rule; no leaf recorded leg 3 before that day and none has since. The other 37 are UNKNOWABLE, not absent: nothing distinguishes asserted-and-fine from never-checked. THE FINDING is an asymmetry with a mechanical cause: the identical primitive (compare the file against a baseline) is run on the way BACK in 6 files / 27 occurrences and on the way IN twice, because a failed revert leaves a DIRTY tree that three mechanisms punish while a failed mutation leaves a CLEAN one that all three read as success. MECHANISM MEASURED, not asserted: sed -i and perl -pi -e both exit 0 whether or not the pattern matched, and asserting the substitution count is the whole difference (exit 9 vs 0); all three recorded leg-3 failures in the repo are textual substitutions and no other primitive has ever failed this way, which answers Open Question 3. COMMIT-VISIBLE ARTIFACT: exists for exactly one class — controls that ship as code (4 self-declared #[test] controls, 1 --self-test, 2 DOCTRINE_STAGED_OVERRIDE test seams that need no mutation at all); a probe that must not compile CANNOT ship (src/ir/knob_roll.rs keeps its E0624 control as prose), and the prose record is section 6.1's self-tick one level up. No mechanism written, deliberately.`
  Commit: `NEGATIVE-CONTROL-HARNESS.1 — measure the record: leg 3 is reported in 2 of 39 episodes`

- ID: `NEGATIVE-CONTROL-HARNESS.2`
  Status: `pending`
  Goal: `Decide the carrier — sourceable helper, TOOLBOX instrument, doctrine check, or nothing — and record it BEFORE building.`
  Acceptance: `Each candidate's failure mode stated, including the structural objection that a control leaves no commit artifact, which may disqualify the doctrine shape outright. The decision cites .1's measurement rather than intuition.`
  Verification: `pending`
  Commit: `pending`

- ID: `NEGATIVE-CONTROL-HARNESS.3`
  Status: `pending`
  Goal: `Implement .2's choice, or record why nothing is warranted per DOCTRINE_ENFORCEMENT.md section 9.`
  Acceptance: `If a harness lands it is itself negative-controlled: removing its marker assertion must make it fail loudly, not silently pass. If nothing lands, the reason is recorded where a future session will meet it.`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `NEGATIVE-CONTROL-HARNESS.2` | `pending` | **Next.** The carrier decision, and `.1` narrowed what it has to carry: the failure mode belongs to **one mutation primitive** (textual substitution, which exits `0` either way), not to controls in general, and the only commit-visible artifact is the class of controls that **ship as code**. Both facts disqualify candidates rather than merely informing them. |
| 2 | `NEGATIVE-CONTROL-HARNESS.3` | `pending` | Build `.2`'s choice, or record why nothing is warranted. |
| — | `NEGATIVE-CONTROL-HARNESS.1` | `done` | Measured, not asserted. **Leg 3 is visibly reported in 2 of 39 leaf-episodes**, both on the day the rule was written; the other 37 are **unknowable**, not absent. The finding is the asymmetry: the same file-versus-baseline check is run **27** times on the *revert* and **twice** on the *mutation*, because only the revert has a gate watching it. |

## Decisions

- `2026-08-01` (registration): **Registered rather than fixed in passing.** A `probe` helper
  already exists at `.cache/book_link_controls.sh` and could have been promoted to `scripts/` in
  one commit during `BOOK-LINK-INTEGRITY.3`. It was deliberately not: the standing directive is
  that a defect is only handled if a tree owns it, `COMMIT.md` lands one leaf per commit, and the
  pivot rule wants a clean tree first. Promoting it silently would also have **assumed** the
  carrier question that `.2` exists to decide.
- `2026-08-01` (`.1`): **The acceptance's region was corrected, not quietly followed.** It named
  *"`docs/tasks/*.md` verification logs"*; the measurement found **83 %** of the control record
  lives outside a `## Verification Log`. Following the region as written would have produced a
  confident number over one sixth of the corpus — the failure
  [[cli-flag-audit-must-be-command-scoped]] records, arriving through the *acceptance* this time
  rather than through the instrument. Stating the region beside the denominator is what caught it.
- `2026-08-01` (`.1`): **No Knowledge Map card was written, deliberately.** The measured mechanism
  (a substitution exits `0` either way) is durable and card-shaped, and writing one is *tempting for
  the same reason promoting the helper was*: it is cheap, obviously useful, and it **pre-decides
  `.2`**. A KM card is one of the candidate carriers — discoverable, but still a habit, and this
  tree exists precisely because the rule was already written down in two places. `.2` picks the
  carrier; if a card is part of the answer it lands there, on the record, rather than by reflex.
- `2026-08-01` (`.1`): **No mechanism, and no repair, in a measurement leaf.** The 37 unknowable
  episodes are **not** re-opened or re-run. Re-conducting historical controls would cost days and
  could not change what the record says, which is the thing under study.

## Open Questions

- ~~**Is this doctrine-shaped at all?**~~ **ANSWERED by `.1`, and the answer constrains `.2`
  sharply.** A commit-visible artifact exists for **one** class only — controls that **ship as
  code** (4 self-declared `#[test]` controls, 1 `--self-test`, 2 `DOCTRINE_STAGED_OVERRIDE` seams).
  The two other classes are structurally unreachable: a probe that must **not compile** cannot be
  committed (`src/ir/knob_roll.rs` keeps its `E0624` control as prose), and the prose record is a
  claim the author writes about their own conduct — `DOCTRINE_ENFORCEMENT.md` §6.1's self-tick one
  level up. A doctrine over classes 2 and 3 would gate a *claim*, not a *fact*.
- **Can a helper compel its own use?** Still open. A sourceable helper only protects probes written
  inside it — and both failures here were written *outside* it, by an author who knew it existed.
  `.1` sharpens the question rather than answering it: the helper does not have to compel *general*
  use, only use of **one primitive**, and the repo already has a **mutation-free** alternative in
  the two staged-set test seams, where the question dissolves instead of being enforced.
- ~~**Is the real subject narrower — shell substitutions?**~~ **ANSWERED by `.1`: yes, measured.**
  `sed -i` and `perl -pi -e` exit **`0`** whether or not the pattern matched; all **three** recorded
  leg-3 failures in the repo are textual substitutions; **no** in-language, compiler-checked,
  constructed or `git show HEAD:`-derived mutation has ever failed this way, because none of them
  has a *match* step. So a helper that owns **the mutation** (apply, **assert the count**, revert,
  verify the revert) addresses the whole measured class, and a helper that owns *the probe* is wider
  than the defect. `.2` decides against that, not against intuition.

## Blockers

- None.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-08-01` | `.1` | `region STATED and CORRECTED before counting: all 80 docs/tasks/*.md, every section, because only 30 of 180 control mentions (17 percent) sit in a "## Verification Log" — the acceptance's own region would have measured one sixth of the corpus. S derived TWICE: narrow term set 18 files, broadened 19 (PARITY-EXTRACTOR-CHARSET-GAP.md records four controls without writing "negative"), plus one named false positive from the widening (SEMANTIC-INTROSPECTION-EXPANSION.md, "control ports"). 180 mentions expanded to 114 context windows and classified BY READING all of them, not by keyword. Denominator 39 leaf-episodes / ~240 self-reported probes (101 in one leaf). Mechanism measured on a scratch file: sed -i and perl -pi -e exit 0 on a NON-matching substitution and 0 on a matching one; a count-asserting form exits 9 vs 0. First attempt at that probe mis-fired (sed -i '' mis-parsed, exit 2) and was re-run correctly before anything was recorded. Shipped-control census: 4 self-declared #[test] controls, 1 --self-test, 2 DOCTRINE_STAGED_OVERRIDE test seams.` | **leg 3 visibly reported in 2 of 39** (`BOOK-LINK-INTEGRITY.3`, `USER-GUIDE-CLI-TABLE-SHADOW.7`, both `2026-08-01`, both the leaves that produced the rule); **37 unknowable, not absent**; revert-verified **27** occurrences across 6 files vs mutation-verified **2** leaves; commit-visible artifact exists for the ship-as-code class only. Docs-only ⇒ DUT byte-identical |
| `2026-08-01` | `.0` | `found while BOOK-LINK-INTEGRITY.3 proved its extractor load-bearing: 3 attempts for 1 control, 2 silently-failed mutations. Distribution: 0 leg-3 failures in 10 harness-written probes, 2 in 3 ad-hoc probes. Ownership search run, not assumed: DOCTRINE_ENFORCEMENT.md 6.1 governs gated checklist boxes (not controls); grep over scripts/ for a sabotage/mutation helper returns nothing.` | `registered` (docs-only; DUT byte-identical) |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `.1` | `4ad09a4` — `NEGATIVE-CONTROL-HARNESS.1 — leg 3 is reported in 2 of 39 episodes` | Measurement only; **no mechanism, no KM card, no repair** — each deliberately deferred to `.2` with the reason recorded. Docs-only ⇒ DUT byte-identical. |
| `.0` (registration) | `3b0d2c2` — `NEGATIVE-CONTROL-HARNESS.0 — register the finding from BOOK-LINK-INTEGRITY.3` | Docs-only; no work leaf executed yet. `.0` is this repo's registration-commit convention (`RESUME-POINTER-CONTRACT.0`, `EMIT-SURFACE-INTERACTION-GATE.0`), required because `.githooks/commit-msg` rejects a subject that names no leaf. |

## Changelog

- `2026-08-01`: `.1` done — **the record is measured, and it says leg 3 is reported in 2 of 39
  leaf-episodes.** Both reporting leaves are dated the same day as the rule they produced
  (`BOOK-LINK-INTEGRITY.3`, `USER-GUIDE-CLI-TABLE-SHADOW.7`); no leaf recorded it before, and none
  has since. The other **37** are **unknowable**, not absent — the distinction matters, because
  silence is what this leaf was told not to read as compliance.
  **The finding is not the ratio, it is the asymmetry underneath it.** The identical primitive —
  *compare the file against a known baseline* — appears **27** times across **6** files proving the
  **revert** landed, and **twice** proving the **mutation** did. That is not carelessness with a
  known rule; it is diligence tracking the gates: a failed revert leaves a **dirty** tree that the
  pre-commit driver, the `COMMIT.md` `git status` review, and the pivot rule all punish, while a
  failed mutation leaves a **clean** tree that all three read as success. The prediction it makes is
  uncomfortable and testable — leg 3 stays skipped for as long as nothing looks at it, no matter how
  many documents restate it.
  **And the subject is narrower than the tree assumed.** Measured, not argued: `sed -i` and
  `perl -pi -e` exit **`0`** whether or not the pattern matched, and asserting the substitution count
  is the entire difference (`9` vs `0`). All three recorded leg-3 failures are textual substitutions;
  in-language, compiler-checked, constructed and `git show HEAD:`-derived mutations have never failed
  this way because none of them has a *match* step. `.2` therefore has to carry **one primitive**,
  not "controls".
  On the commit-visible question: an artifact exists for **one** class, controls that **ship as
  code** (4 self-declared `#[test]` controls, 1 `--self-test`, 2 mutation-free staged-set seams). A
  probe that must not compile **cannot** ship, and the prose record is `DOCTRINE_ENFORCEMENT.md`
  §6.1's self-tick one level up. Two temptations were declined and recorded rather than acted on: no
  helper (that is `.2`'s decision) and no Knowledge Map card (a card is itself a candidate carrier,
  and writing one would pre-decide the question exactly as promoting the helper would have).
  *One more thing worth the honesty:* the acceptance's own region — *"verification logs"* — was
  **wrong**, holding 17 % of the record, and the first derivation of the file set **under-counted**,
  missing a tree that ran four controls without ever writing the word *negative*. Both were caught by
  the standing discipline of stating the region beside the denominator and deriving `S` twice.
- `2026-08-01`: Created. `BOOK-LINK-INTEGRITY.3` needed one load-bearing control and it took
  **three attempts to run**, twice passing on a mutation that had silently failed to apply — the
  exact trap `USER-GUIDE-CLI-TABLE-SHADOW.7` recorded and wrote into `DEVELOPMENT_NOTES.md`. The
  rule was known, recent, and in mind. What separated the 10 clean probes from the 2 broken ones
  was not care but **carrier**: the clean ones ran inside a helper that asserts its marker before
  reading a verdict, the broken ones were ad-hoc one-liners. Registered rather than repaired in
  passing, because promoting that helper would have pre-decided the carrier question — and because
  the honest obstacle (a control leaves no artifact in the commit) may mean the answer is not a
  doctrine at all.
