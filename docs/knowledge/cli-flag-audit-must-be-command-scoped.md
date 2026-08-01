---
id: cli-flag-audit-must-be-command-scoped
title: A flag audit over a multi-command CLI must be command-scoped — pooling one document's flag rows against one command's `--help` reports live flags as deleted and prose as flags, and `--help` quotes OTHER commands' flags in its own text
answers:
  - "the docs list a flag that anvil --help does not have"
  - "is --divergence a stale row in USER_GUIDE"
  - "why did my flag audit say a live flag no longer exists"
  - "how do I count the CLI flags of a clap app"
  - "why is my flag count one too high"
  - "can I grep -- --x out of --help to get the flag set"
  - "anvil hunt flags vs anvil flags"
  - "should USER_GUIDE's knob table list --seed and --out"
  - "what does the CLI flag table in USER_GUIDE contain"
  - "how do I know if a new flag needs a USER_GUIDE row"
  - "what is the authority for the knob set"
  - "why does an audit need a total partition"
  - "two audits of the same doc section disagree on the count"
  - "what region should a shadow-enumeration audit measure"
  - "does a cross-reference in prose count as a list entry"
  - "should USER_GUIDE document tool_matrix's options"
date: 2026-08-01
status: current
tags: [live-docs, shadow-enumeration, extractor, gotcha, cli, clap, audit, gate-quality]
reverify: "./target/release/anvil --help | grep -c -- '--diff-sim'   # 1 — yet `anvil --diff-sim` is an error: the match is inside the `hunt` subcommand's one-line description in the `Commands:` block. Then: ./target/release/anvil hunt --help | grep -c -- '--divergence' # 1 — the flag USER-GUIDE-CLI-TABLE-SHADOW.1 recorded as 'no longer exists'. The command-scoped extraction is in that tree's Correction block; re-run it by scoping to each command's own `Options:` section."
evidence: docs/tasks/USER-GUIDE-CLI-TABLE-SHADOW.md (the `.1` measurement; `.2`'s Correction block re-measuring it at `f2d282e` — 107 not 108, 0 stale rows not 2; and the region-scope pairs `.2`/`.4` = 11 vs 14 and `.4`/`.5` = 22 vs 21); src/main.rs (`Cli` and `HuntCommand` — two `#[derive(Parser)]` structs, two disjoint namespaces); src/bin/tool_matrix.rs (a third registry, with no `Overrides` projection at all); src/config.rs (`Overrides` — the derivable knob authority); USER_GUIDE.md "Knobs" (the contract paragraph `.2` wrote) and "Tool matrix sweeps" → "Options" (the one `.5` wrote)
---

An audit that asks *"which documented flags no longer exist?"* needs an authoritative set `S`.
Getting `S` wrong does not produce a small error — it produces **confident, specific, wrong
findings**, because every document row is then measured against it.

**Two ways `S` goes wrong on a `clap` app, and ANVIL hit both in one measurement.**

1. **The document covers more than one command.** `USER_GUIDE.md` has a knob table for `anvil`
   *and* a flag table for `anvil hunt`. They are **disjoint namespaces** — two
   `#[derive(Parser)]` structs. Pooling the document's flag rows and comparing them to `anvil
   --help` reported `--divergence` and `--no-minimize` as *"rows naming a flag that no longer
   exists"*. Both are live `anvil hunt` options, in the right table, and that table is **10/10
   complete** against `anvil hunt --help`.
2. **`--help` output is not a flag list.** It is *prose that contains flags*. The `Commands:`
   block prints each subcommand's one-line description, and ANVIL's `hunt` description quotes
   `` `--diff-sim` ``. A `grep -o -- '--[a-z-]*'` over `anvil --help` therefore yields **108**
   where the option count is **107**. The phantom then propagates: it also inflated the
   "mentioned in the guide but not tabled" bucket from 19 to 20.

**The rule:** *scope the extraction to one command's own `Options:` section, and take options
from option position — not from anywhere the string appears.* Every command in the app gets its
own `S`, and every documented table is measured against exactly one of them.

**`--help` is a formatter, so this is [[never-parse-a-formatter-for-a-semantic-set]] with a second
axis.** That card's extractor read 7 of 8 steering categories because `rustfmt` wrapped one match
arm; this one over- *and* under-counted because `clap`'s renderer interleaves two commands' prose
in one stream. Same lesson — the layout you are grepping is owned by a tool that may re-lay it out
— plus a new one: **a formatter for a multi-command app emits several sets into one document, and
nothing in the text marks the boundary except the section headers.**

**This is the fifth instrument error in this repo's recent history**, and the pattern across them
is worth more than any one fix. [[extractor-charset-narrower-than-source]] captured too narrow a
*charset*; the `rustfmt` case above assumed too narrow a *row shape*; `.1` of this very tree used a
backtick-anchored matcher that missed `` `--artifact dut` `` because flag and value share one code
span. Each time the extractor's **domain** was narrower or wider than the source it claimed to
read, and each time the number came out plausible. **A plausible number is the failure mode** —
nothing about `108` or `2 stale rows` looks wrong.

**The counter-discipline is a total partition, not a spot check.** `.2` did not stop at "the 18
missing rows"; it classified **every** member of `S` into *knob* or *mode* and required

    |knobs| + |modes| == |S|,  with an empty intersection

which came out `93 + 14 == 107`. A residue on either side would have meant the criterion was a
preference rather than a contract. This is [[extractor-charset-narrower-than-source]]'s
`total_or_fail` — an extractor must account for every item it walks — applied to a **docs audit**
rather than to a gate script.

**What the contract turned out to be, and why "exhaustive vs curated" was the wrong question.**
The `USER_GUIDE.md` knob table is exhaustive over a **derived** set: every flag that sets a knob —
directly (its value is a `Config` field, so it is equally settable in `--config knobs.json`) or as
a convenience flag setting several at once. The authority is the CLI projection of
`config.rs::Overrides`, plus `--profile` / `--full-factorization` / `--no-full-factorization`.
Flags that select a **mode** rather than a knob **value** — run size and destination, artifact
lane, tracing, the introspection dumps, `--help`, `--version` — belong to their own sections.

Two things fall out that the original framing could not have reached:

- **The criterion earns rows an "absent flags" repair would miss.** 12 of the 18 additions were
  absent from the document entirely; **6 were mentioned in prose and are genuine knobs**. Repairing
  "the 13 absent" would have left the table incomplete on its own new contract the day it shipped.
- **`--version` stops being an awkward caveat.** `.1` had to record it as a footnote to a count
  ("arguably not a flag this document owes the reader"). Under a criterion it is simply a mode
  flag, excluded on principle. *A number needing an asterisk is usually a criterion that has not
  been stated yet.*

**Do not fix a shadow by writing down its members.** The contract paragraph names **no** flags and
gives **no** count, because a hand-kept list of "the mode flags" would be derivable,
growth-coupled and silent — decision `0033`'s three tests, all passing — i.e. the same shadow,
recreated inside the sentence that removes it. See [[coverage-check-vacuity]] for the same
vacuity at gate scale, and [[defect-class-audit-rules]] for the sweep discipline this audit follows.

## The other half of the same rule: scope the SHADOW too, not just `S`

Everything above scopes the **authoritative** side. Two later leaves in the same tree showed the
**documented** side needs the identical discipline, and got two different numbers for one
unchanged file each time:

- `.2` registered *"the book's `## CLI coverage` is missing **11**"*; `.4` measured **14**.
  `book/src/knobs.md` was byte-identical between the two commits — the *instrument* moved, not the
  file. 11 reproduces only at whole-**section** scope (counting a "not yet exposed" prose list and
  a second binary's block as coverage) against a 105-flag `S`.
- `.4` registered *"the `tool_matrix` block names **22** of 37"*; `.5` measured the fenced snapshot
  at **21**. The extra is `--diff-sim`, which the closing prose mentions as a *cross-reference*
  ("use `--diff-sim` for cross-simulator trace agreement"), never as an entry.

**The rule: name the region, not the neighbourhood. "The block", "the section", "the list" are not
scopes** — a fence, a heading range, or a line range is. And **a cross-reference is not a list
member**: prose that mentions a flag in order to send you elsewhere is doing the opposite of
enumerating it. Publish the region alongside the number, or the next reader re-measures and
"disagrees" with you about a file neither of you changed.

Note the asymmetry with the `S`-side errors above: those produced **wrong** numbers. These two
produced numbers that are each **right for their region** — so a floor, a diff, or a re-run cannot
catch them. Only stating the region can.

**And state the denominator, not just the region — the two travel together.** Once the contract
excluded `clap`'s built-ins, `tool_matrix` had *two* live totals: **37** options it prints and
**35** it declares. `.5`'s own first draft wrote *"the snapshot named 21 of the 35 declared
options"* — a **37**-scope numerator over a **35**-scope denominator, in the leaf whose whole
subject is scope. It named 21 of 37, equivalently **19 of 35**. Caught by re-deriving the
numerator on the denominator's scope before publishing, which is the only thing that catches it:
both numbers are individually correct, so nothing downstream disagrees.

## When the knob/mode criterion returns *"neither"*

`.5` applied `.2`'s partition to `tool_matrix`'s options and it **settled the question by
failing**: that binary has **no `Overrides` projection and no knobs** — not one of its 37 options
sets a `Config` field — so under the partition every option is a *mode* flag and a chapter about
knobs owes **zero** rows. The list was deleted (decision `0033` rung R1) rather than refreshed, and
the reference gained the contract instead: *exhaustive over the options the binary **declares***,
with `clap`'s built-ins outside the set by a **derivable** rule — a built-in is exactly an option
with no `Cli` field. `35 + 2 == 37`, the same total partition demanded above.

Two traps that only a command-scoped instrument sees, both live here:

- **One spelling, two registries.** `--divergence` is a `tool_matrix` column *and* an `anvil hunt`
  axis. A document-wide grep reports it "documented" while the reference has no entry for the
  matrix's column. This is the card's opening error read backwards — `.1` measured two commands
  against one registry; this is one flag name read across two.
- **Delete-and-point is not automatically lossless.** `.4` could cut immediately (0 of 107 flags
  lost a home). `.5` could not: four options were documented **nowhere** outside the copy it was
  about to delete. Complete the destination and prove losslessness **before** the cut, not after.

**The rule is now mechanical, and the mechanism obeys it.** `USER-GUIDE-CLI-TABLE-SHADOW.3` gated
the table as `ENUMERATION-PARITY` **pair 5** — and the check derives its set from **`cli_overrides`
in `src/main.rs`, never from `anvil --help`**, for the reason this card exists plus a second one: a
doctrine check reads the repository (`DOCTRINE_ENFORCEMENT.md` §4(4)), and `--help` needs a built
binary that a fresh clone and a pre-build CI step do not have. Reading source instead of a
formatter's output answers both objections at once — which is the general shape of the fix, not a
coincidence.
