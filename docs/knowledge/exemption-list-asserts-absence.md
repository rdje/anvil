---
id: exemption-list-asserts-absence
title: Audit the **exemption** half of a shadow list, not just the positive half — an omission under-reports, but a false exemption *asserts an absence the reader acts on*, and it is derivable as the complement of the set it exempts from
answers:
  - "which half of a hand-maintained list should I audit first"
  - "how bad is a stale not-yet-supported list"
  - "is a not-yet-exposed-via-CLI list a shadow enumeration"
  - "the docs say this knob has no CLI flag but it does"
  - "why is a missing row less serious than a wrong exemption"
  - "how do I derive the set of config fields with no CLI flag"
  - "a docs section contradicts itself — repair it or delete it"
  - "when is refreshing a duplicated list the wrong repair"
  - "why did USER-GUIDE-CLI-TABLE-SHADOW.4 delete the book section instead of fixing it"
  - "how many anvil knobs are config-file-only"
  - "my count disagrees with the number recorded in the task tree"
  - "how do I derive an authoritative flag set without trusting --help"
date: 2026-08-01
status: current
tags: [live-docs, shadow-enumeration, extractor, gotcha, audit, book, doctrine, gate-quality]
reverify: "./target/release/anvil --dump-config | python3 -c \"import json,sys,re,subprocess; cfg=json.load(sys.stdin); leaves=[]; walk=lambda d:[(walk(v) if isinstance(v,dict) else leaves.append(k)) for k,v in d.items()]; walk(cfg); cli=subprocess.run(['sed','-n','/^struct Cli {$/,/^}$/p','src/main.rs'],capture_output=True,text=True).stdout; fields={m for m in re.findall(r'^    ([a-z0-9_]+):', cli, re.M)}; print(sorted(set(leaves)-fields))\"   # must print exactly ['library_prob', 'max_nodes_per_module', 'use_async_reset'] — the THREE knobs with no CLI flag. book/src/knobs.md claimed TWELVE until 2026-08-01, nine of them false. NOTE THE AUTHORITY: the `Cli` STRUCT, not `cli_overrides`. Deriving from `cli_overrides` prints a fourth member, `seed` — which has a flag (`--seed`) and is simply applied outside the `Overrides` projection. That near-miss is this card's own lesson landing on its own reverify line: the complement of the WRONG set is a false exemption."
evidence: docs/tasks/USER-GUIDE-CLI-TABLE-SHADOW.md (`.4` — the measurement, the R1 decision, and the reproduction of the registered `11`); book/src/knobs.md (the repaired §*CLI coverage*, and the `### Opt-in capability knobs` heading that replaced the false one); src/config.rs (`Config` — the serde projection that makes the exemption set derivable); src/main.rs (the `Cli` struct — the set it is the complement of; NOT `cli_overrides`, which omits `seed` and would add a false fourth member); docs/decisions/0033-shadow-enumeration-classification.md §3 (the same asymmetry measured inside `merge_coverage`)
---

Every shadow-enumeration audit in this repo asked one question: **how far behind is the copy?**
`USER-GUIDE-CLI-TABLE-SHADOW.1`/`.2`/`.3` counted absent flags, absent rows, and gated against
absence. `.4` asked it of `book/src/knobs.md`, got **14**, and then found something worse sitting
directly beneath — in the part no prior leaf had thought to measure.

## The exemption half

A section carrying a hand-maintained list almost always carries a second, smaller one beside it:
the **exemptions**. *"Not yet exposed via CLI."* *"Not covered by this gate."* *"Reachable only
via `--config`."* *"Unsupported on this platform."*

In `book/src/knobs.md` that list named **12** knobs. **Nine of them had CLI flags** — and eight
of those nine were listed **as CLI flags, in the same section, about sixty lines earlier**.

## The two halves fail asymmetrically, and only one is safe

| half | what an omission does | what the reader does with it |
| --- | --- | --- |
| the **positive** list (*"here are the flags"*) | under-reports — a real member goes unmentioned | reaches for `--help`, finds it, is mildly annoyed |
| the **exemption** list (*"these have none"*) | **asserts an absence that is false** | believes the capability is missing — writes a config file to reach what a flag already sets, or concludes a delivered feature was never built |

An omission is a **gap**. A false exemption is a **claim**. Decision `0033` §3 measured this same
asymmetry from the other end: `merge_coverage`'s merges are monotone, so a forgotten line can only
*under*-report, and the site was re-scoped from S3 down to S1 on exactly that reasoning. Read
forwards, it says: **a list that can only under-report is fail-safe; a list that asserts absence
is not.**

## It is derivable, in the same way its twin is

The exemption list is the **complement** of a derivable set, so it inherits derivability. Here it
is `Config`'s serde fields minus the flag-bearing fields of the clap `Cli` struct — three lines of
shell — and it yields **three**: `library_prob`, `use_async_reset`, `max_nodes_per_module`. The
hand-kept version said twelve.

**But pick the right set to complement, or you manufacture the very defect you are removing.**
Complementing `cli_overrides` instead of `Cli` — the natural choice, since `cli_overrides` is what
`ENUMERATION-PARITY` pair 5 derives from — yields a **fourth** member, `seed`. `--seed` plainly
exists; it is simply applied outside the `Overrides` projection. Writing that down would have
published a false exemption in the card that exists to warn about false exemptions. It was caught
by *running* the reverify line rather than trusting it, which is what the `reverify:` field is for.

So the exemption half passes all three of decision `0033`'s tests (derivable, growth-coupled,
silent) and is a shadow in its own right. It rarely gets classified as one, because it reads as
boilerplate and nobody re-reads it.

## A section that contradicts itself is past repair

Eight of the nine false claims were refuted **by the same section that made them**. That is the
signal that decides the repair rung, and it is worth stating as a rule:

> When a copy has drifted far enough to disagree with itself, **refreshing** it returns it to
> *behind* and **gating** it *freezes* it in place. Only decision `0033`'s rung **R1** — delete
> the copy, point at the reference — removes the failure mode instead of relocating it.

That is what turned `.4` from *"add the 14 missing rows"* into *"delete the section"*. Note the
scoping discipline that came with it: the nine long capability-knob **paragraphs were kept**. They
fail `0033` test (2) — a new capability knob does not *require* a bullet for the chapter to be
correct — so they are prose, not a shadow. Only the heading above them was false, and only the
heading changed. **Repair the shadow and the false claim; do not use the occasion to prune.**

And the replacement names the three config-file-only knobs **once**, elsewhere, by pointer: that
set is derivable and growth-coupled, so re-listing it under the new heading would have created a
fresh shadow *inside the repair* — the mistake [[cli-flag-audit-must-be-command-scoped]] records
`.2` avoiding when it refused to enumerate the mode flags.

## Reproduce a disagreeing number before you publish your own

The tree had **11** registered; `.4` measured **14**. Publishing the new number and moving on is
the tempting move and the wrong one. Reproducing the old number is cheap and tells you *which
thing moved*: `book/src/knobs.md` is **byte-identical** between `f2d282e` and `5ce2dd3`, so the
file never changed. `11` falls out only under a scope that counts the exemption prose **and a
different binary's flag block** as coverage, against a 105-flag `S`. The file was innocent; the
instrument moved.

That is the **sixth** instrument note in this repo's recent history and the second in one tree —
see [[cli-flag-audit-must-be-command-scoped]], [[extractor-charset-narrower-than-source]] and
[[never-parse-a-formatter-for-a-semantic-set]] for the others.

**The countermeasure `.4` used throughout: derive `S` twice by independent routes and diff them
before comparing anything to it.** The `Cli` struct in `src/main.rs` (declared fields, minus the
subcommand field, plus clap's built-ins) and the `Options:` block of `anvil --help` read
command-scoped — **107** both ways, empty diff. Two instruments that agree are evidence; one
instrument is a number.

## The two rules to carry forward

1. **Audit the exemption half of any shadow you audit.** It is smaller, it looks like boilerplate,
   nobody re-reads it, and it fails in the direction that costs the reader something. Prefer
   deriving it — the complement of a derivable set is derivable.
2. **Prove the deletion lossless *before* cutting.** `.4` checked that all 107 flags kept a home
   across `USER_GUIDE.md` + `book/src/*.md` (**0** undocumented) and that the three truthful
   bullets were strict *subsets* of richer entries already in the chapter. A repair that cannot
   show what it kept is a deletion with a rationale attached. See [[defect-class-audit-rules]] for
   the sweep discipline, and record the sweep's **match count**, not only its finds — 30
   `book/src/*.md` walked, `knobs.md` the lone outlier.
