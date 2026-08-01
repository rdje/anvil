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
date: 2026-08-01
status: current
tags: [live-docs, shadow-enumeration, extractor, gotcha, cli, clap, audit, gate-quality]
reverify: "./target/release/anvil --help | grep -c -- '--diff-sim'   # 1 — yet `anvil --diff-sim` is an error: the match is inside the `hunt` subcommand's one-line description in the `Commands:` block. Then: ./target/release/anvil hunt --help | grep -c -- '--divergence' # 1 — the flag USER-GUIDE-CLI-TABLE-SHADOW.1 recorded as 'no longer exists'. The command-scoped extraction is in that tree's Correction block; re-run it by scoping to each command's own `Options:` section."
evidence: docs/tasks/USER-GUIDE-CLI-TABLE-SHADOW.md (the `.1` measurement, and `.2`'s Correction block re-measuring it at `f2d282e`: 107 not 108, 0 stale rows not 2); src/main.rs (`Cli` and `HuntCommand` — two `#[derive(Parser)]` structs, two disjoint namespaces); src/config.rs (`Overrides` — the derivable knob authority); USER_GUIDE.md "Knobs" (the contract paragraph `.2` wrote)
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

**The rule is now mechanical, and the mechanism obeys it.** `USER-GUIDE-CLI-TABLE-SHADOW.3` gated
the table as `ENUMERATION-PARITY` **pair 5** — and the check derives its set from **`cli_overrides`
in `src/main.rs`, never from `anvil --help`**, for the reason this card exists plus a second one: a
doctrine check reads the repository (`DOCTRINE_ENFORCEMENT.md` §4(4)), and `--help` needs a built
binary that a fresh clone and a pre-build CI step do not have. Reading source instead of a
formatter's output answers both objections at once — which is the general shape of the fix, not a
coincidence.
