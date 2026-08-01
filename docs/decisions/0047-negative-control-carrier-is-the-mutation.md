---
id: negative-control-carrier-is-the-mutation
title: The carrier is the **mutation**, not the control — prefer a mutation form that *cannot* silently no-op (R1), and where a textual substitution is unavoidable use a helper whose `apply` step refuses a zero-count substitution (R2); a doctrine check is **structurally disqualified**, because the only commit-visible trace is the author's own prose claim
answers:
  - "how do I stop a negative control passing on a sabotage that never applied"
  - "should proving the sabotage landed be a doctrine check"
  - "why is there no check that a negative control asserted its mutation"
  - "what is the right way to sabotage a file for a negative control"
  - "my sed or perl mutation may not have matched, how do I know"
  - "why does sed exit 0 when the pattern did not match"
  - "which mutation forms do not need a landing assertion"
  - "can a helper script make people run negative controls correctly"
  - "where should a negative-control mutation helper live"
  - "is a TOOLBOX entry enough to enforce a control practice"
date: 2026-08-01
status: accepted
tags: [testing, control, gate-quality, doctrine-enforcement, tooling, workflow, honest-limits]
evidence: docs/tasks/NEGATIVE-CONTROL-HARNESS.md (`.1`'s measurement — 39 leaf-episodes, leg 3 visibly reported in 2, the 27-vs-2 revert/mutation asymmetry, and the shipped-control census); docs/decisions/0033-shadow-enumeration-classification.md (b) (the R1→R4 repair ladder this decision applies); DOCTRINE_ENFORCEMENT.md §6.1 (a box is EARNED, not ticked — why a prose-reading check is disqualified) and §7 (E1 discovery is not enforcement); src/ir/knob_roll.rs (the control that cannot ship, kept as prose)
reverify: "printf 'alpha\\n' > /dev/null; f=$(mktemp); printf 'alpha\\n' > \"$f\"; perl -pi -e 's/NOMATCH/x/' \"$f\"; echo \"non-matching substitution exit=$?\"   # 0 — indistinguishable from a matching one; then: perl -pi -e 'BEGIN{$c=0} $c+=s/NOMATCH/x/g; END{exit($c?0:9)}' \"$f\"; echo \"count-asserting exit=$?\"   # 9. The census behind the decision is docs/tasks/NEGATIVE-CONTROL-HARNESS.md 'The measurement'."
---

# 0047 - NEGATIVE-CONTROL-HARNESS.2: the carrier is the mutation, not the control

- Date: 2026-08-01
- Status: accepted
- Tree: `NEGATIVE-CONTROL-HARNESS.2` (the carrier decision; `.1` supplied the measurement)
- Activated by: autonomous PNT selection under the owner's standing **DECIDE, DON'T ASK**
  directive ([`0041`](0041-owner-standing-directives-recorded-in-layer-c.md))

## Context

A negative control needs three legs: the check **can** fire, it fires on the **right input**
(the `DOCTRINE_ENFORCEMENT.md` §9 vacuity probe), and **the experiment ran at all**. The third
fails silently and in the dangerous direction — a substitution that does not match leaves the tree
unchanged, the check passes, and that is indistinguishable from a control that correctly did not
fire.

The rule was written down twice — `USER-GUIDE-CLI-TABLE-SHADOW.7`'s tree entry and a
`DEVELOPMENT_NOTES.md` section — and then cost **two false results in one hour** at
`BOOK-LINK-INTEGRITY.3`. This tree exists because *writing it down a third time* is the one remedy
already known not to work.

`.1` measured the record rather than guessing at it. Four findings bind this decision:

1. **Leg 3 is visibly reported in 2 of 39 leaf-episodes**, both dated `2026-08-01`, both the leaves
   that *produced* the rule. No leaf recorded it before that day. The other 37 are **unknowable** —
   nothing distinguishes *asserted and fine* from *never checked*.
2. **The failure mode belongs to one primitive.** Measured: `sed -i` and `perl -pi -e` exit **`0`**
   whether or not the pattern matched; asserting the substitution count is the entire difference
   (`9` vs `0`). All three recorded leg-3 failures in this repository are textual substitutions.
   **No** in-language, compiler-checked, constructed or `git show HEAD:`-derived mutation has ever
   failed this way, because none of them has a *match* step to miss.
3. **The habit follows the gate, not the rule.** The identical primitive — *compare the file against
   a known baseline* — appears **27 times across 6 trees** proving the **revert** landed and
   **twice** proving the **mutation** did. A failed revert leaves a **dirty** tree, which the
   pre-commit driver, `COMMIT.md` §10's `git status` review and the pivot rule all punish; a failed
   mutation leaves a **clean** tree, which all three read as success.
4. **Commit-visible artifacts exist for exactly one class** — controls that **ship as code**
   (4 self-declared `#[test]` controls, 1 `--self-test`, 2 mutation-free `DOCTRINE_STAGED_OVERRIDE`
   test seams). A probe that must *not compile* cannot ship at all; `src/ir/knob_roll.rs` keeps its
   `E0624` control as **prose** for exactly that reason.

Finding 3 is the one that grades the candidates. **A carrier nothing observes will not be used** —
which is not a hypothesis here but the measured history of this very defect.

## Decision

**Carry the *mutation*, not the control, and carry it on decision `0033`'s ladder.**

**R1 — preferred, and it removes the need rather than guarding it.** Where a control can be written
with a mutation form that has no *match* step, use one and leg 3 **ceases to exist**:

| form | why it cannot silently no-op | live example |
| --- | --- | --- |
| in-language, inside a `#[test]` | the mutation is a statement; it compiles or it does not | `src/ir/case_qualifier.rs` breaks its fixture with `m.nodes[4] = Node::Constant { value: 3, width: 2 };` **after** asserting the fixture is clean *before* the break |
| compiler-checked probe | the landing **is** the diagnostic | `E0624` / `E0616` / `E0004` / `E0599` |
| constructed fixture | built from nothing, so there is nothing to match | `README-POLICY-ADOPTION.3`'s isolated fixture root, at exact line and byte counts the check echoes back |
| `git show HEAD:`-derived baseline | the *before* side is read, never edited | `LIVE-DOC-REGISTRY-SHADOWS.1` |
| mutation-free test seam | the check is fed a synthetic input; the tree is untouched | `DOCTRINE_STAGED_OVERRIDE` in `check_diagnosis_evidence.sh` / `check_task_tree_ownership.sh` |

A seam is admissible **only** when it does not bypass the extraction under test — otherwise it
converts the oracle into a tautology (`test-seam-bypassing-the-extraction-under-test`).

**R2 — when a textual substitution is genuinely unavoidable** (sabotaging a tracked document, or a
shell check whose subject *is* file text), the mutation goes through **one tracked helper that
refuses to no-op**: its `apply` step asserts the substitution count and exits nonzero when it is
zero, before any verdict can be read. `.3` builds it to the contract pinned below.

**No doctrine check. The disqualification is structural, not a preference** — and it is the
objection this tree registered itself with, now confirmed by measurement rather than assumed.

## The candidates, each with its failure mode

### A — a sourceable `scripts/` helper that makes the right probe easy ⚠️ **necessary, not sufficient**

Failure mode, and it is **measured rather than hypothesised**: a helper cannot compel its own use.
`.cache/book_link_controls.sh` **existed**, its author **knew** it existed, and both leg-3 failures
were written *outside* it in the same hour. An easier path is not an observed path.

Kept — as R2 — because of *what it guards*, not because it is easy: it owns the **mutation**, which
is the object that fails, and it is the thing reached for at the moment of the mistake. This is
`COVERAGE-STEERED-GENERATION.3b`'s rule turned on a shell tool: **guard the effect, not the
wrapper.**

### B — a `TOOLBOX.md` instrument ❌ **as the carrier**

`DOCTRINE_ENFORCEMENT.md` §7 classifies a document entry as **E1, discovery**, and states plainly
that discovery is *not* enforcement. This tree exists because the rule was already discoverable in
**two** documents. Making it three is the remedy with a measured failure rate.

Kept as the **pointer beside** whatever lands, never as the mechanism: `TOOLBOX.md` Part 1 gains one
row for the helper, so it is findable, and that row makes no claim to enforce anything.

### C — a registered doctrine check ❌ **structurally disqualified**

It has nothing to read. A control's mutation is reverted before the commit **by construction** — the
pivot rule and the pre-commit driver both demand a clean tree — so the mutation is never *in* a
commit. The only readable trace is the **prose claim** in a Verification Log, and a check over that:

- gates a **self-tick** (§6.1: *ticking must never be the proof*) — the author writes both the claim
  and the evidence for it, one level up from the box the standard already warns about;
- would **cry wolf** on every legitimately mutation-free control (the whole R1 table above), which
  needs no landing assertion at all — and a gate that cries wolf gets deleted, taking its real
  coverage with it (`0033` test (2));
- is satisfiable by **typing the words**, which is precisely the failure `.1` measured: the record
  cannot distinguish *asserted* from *unchecked*, so neither can a check that reads it.

Recorded so a future session does not re-propose it: the tempting shape — *"a leaf whose record says
`negative control` must also say it asserted the mutation"* — is a **lexical** gate over an author's
self-description. It would raise the leg-3 report rate to 39 of 39 without changing what anyone did.

### D — nothing (`DOCTRINE_ENFORCEMENT.md` §9) ❌

Legitimate when no mechanism genuinely helps. Not the case here, and the evidence is in the
measurement: the **only two** episodes that ran leg 3 both ran it with a **tool** — `.1`'s
distribution was **0 failures in 10** harness-written probes against **2 in 3** ad-hoc ones. The
mechanism that works is known; declining to build it would be a choice to keep paying for it.

### E — a helper that owns the mutation, refusing a zero-count substitution ✅ **chosen (R2)**, under R1

Chosen with its limit stated up front: **it cannot compel its own use either.** What it changes is
the shape of the failure. Today a mistyped substitution produces a **plausible wrong finding**;
with the helper it produces a **loud error**, and the wrong finding is unreachable *through the
tool*. That is a strictly smaller surface than "remember to assert the count", and it is the only
candidate that acts at the moment and place of the mistake.

## The contract `.3` implements (pinned here so `.3` builds rather than re-derives)

- **`apply <file> <perl-expr>`** — snapshot the file to an on-volume backup under `.cache/`
  (decision [`0031`](0031-ssd-volume-exclusivity.md): same volume, repo-root-derived, never the OS
  temp dir), apply the substitution, and **exit nonzero with a named message when the substitution
  count is zero**. This is the whole point of the tool; everything else is convenience.
- **`restore <file>`** — restore from the backup and **verify with `cmp`**, exiting nonzero if the
  file differs. The repo already does this 27 times by hand; folding it in costs nothing and makes
  the pair symmetric.
- **`probe <file> <perl-expr> <fires|silent> <cmd…>`** — the whole cycle in one call: apply (with
  the landing assertion), run the check, compare the verdict against the **declared expectation**,
  restore, verify the restore. One command, so the correct thing is also the short thing.
- **`--self-test`** — the **control on the control**, required by `.3`'s acceptance: a deliberately
  non-matching expression must make `apply` exit nonzero, and **removing the count assertion must
  make the self-test fail**, not silently pass.
- **Not** registered in `DOCTRINES`. It is an instrument, not a gate — per candidate C, there is
  nothing for a gate to read.

## Honest limits (§9 — stated, not discovered later)

- **Use is not compelled.** Nothing forces a probe through the helper; an author can still write a
  bare `perl -pi -e`. The claim is narrower and true: the *tool* cannot produce the silent failure.
- **R1 is a preference, not a check.** No mechanism verifies that an author chose a no-match-free
  mutation form when one was available. Recording the R1 table is discovery (E1), with E1's known
  ceiling.
- **The 37 unknowable episodes stay unknowable.** Nothing here retro-validates them, and they are
  deliberately not re-run: the record is the object of study, and re-conducting history could not
  change what it says.
- **This decision does not raise the leg-3 report rate, and that is intentional.** A carrier that
  made the *record* look complete without changing conduct is exactly candidate C.

## Consequences

- `.3` builds the R2 helper to the contract above, negative-controls it **both ways including on
  itself**, and adds one `TOOLBOX.md` row for discovery.
- The R1 table becomes the first thing to consult when a control is designed — *can this mutation be
  written so that leg 3 does not exist?* — with R2 as the fallback rather than the default.
- `NEGATIVE-CONTROL-HARNESS` closes at `.3`. The class it leaves behind is not "controls" but
  **"a mutation primitive whose success and no-op are the same exit code"**, which is a
  transferable rule: *never let a tool report the same success for having done nothing.*
