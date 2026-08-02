---
id: clause-boundary-is-destroyed-by-the-list-conversion
title: A proof whose unit is the **clause** cries wolf on a run-on → list conversion, because the conversion deletes the separators that *define* a clause boundary — the stable unit is the word
answers:
  - "my clause-multiset proof reports clauses removed on an edit I know is correct"
  - "how do I prove a lift moved text without losing any of it"
  - "why does prove_clauses_unchanged.py fire when I move a paragraph"
  - "what unit should a move-tolerant text proof compare"
  - "the proof says N clauses removed and 1 added but nothing was lost"
  - "how do I prove a GFM table cell was lifted into a list correctly"
  - "why did my negative control fire on everything"
date: 2026-08-02
status: current
tags: [book, live-doc, evidence, gate-quality, markdown, mdbook, gotcha]
evidence: docs/tasks/BOOK-PARAGRAPH-BLOBS.md (`.3c` Findings — the cried-wolf output and the rebuild on words); scripts/prove_clauses_unchanged.py (`words()` and its docstring)
reverify: "scripts/prove_clauses_unchanged.py --allow-move --ref HEAD book/src/api-introspection.md  # OK, 1648 words, multiset identical"
---

**A "lift" is not a list conversion and not a whitespace split.** It moves a run of text out of a
container it does not fit — the case that produced this card was a **GFM pipe-table cell**, which
cannot hold a block-level list, so the fourteen sentences crammed into one `<td>` had no in-place
repair at all. Because a lift *relocates* text, every order-sensitive proof fires on it by
construction and reports nothing useful.

The obvious move-tolerant proof is to compare **multisets** instead of sequences. The trap is
choosing the wrong unit.

## The clause is the wrong unit, and it fails loudly rather than quietly

`BOOK-PARAGRAPH-BLOBS.3c` built exactly that mode over **clauses** and ran it on a known-correct
edit:

```
REMOVED: `memory_provenance` (`1.18 → 1.19`)                                  <- 10 reported destroyed
REMOVED: `fsm_provenance` (`1.19 → 1.20`)
…
ADDED:   `module_reachability` (…) `flop_dependencies` (…) `memory_provenance` (…) …   <- 1 created
```

Nothing had been destroyed. **A list conversion deletes the separating commas — that is the entire
point of the conversion — and the commas are what define a clause boundary.** So

```
A, B, C          ->      - A
                         - B
                         - C
```

is *three* clauses before and *one* after. The clause multiset is **not invariant under the very
edit the mode exists to permit**.

This is worse than a blind spot. A blind instrument stays silent and you may notice; this one
**manufactures findings on correct work**, and a gate that cries wolf gets argued with and then
deleted, taking its real coverage with it.

## The word is invariant, because normalization already removed the separators

`normalize()` strips list markers, separators and connectives *before* anything is split, so
splitting its output on whitespace yields a unit that survives both the separator removal and the
move. Rebuilt on words, the same edit reports:

```
book/src/agent-mcp.md:          WORDS DIFFER (8332 -> 8347)   +15 added, 0 removed, each printed
book/src/api-introspection.md:  OK (1648 words, multiset identical)
```

`0 removed` is the fact that matters — it is the direct answer to *"did the lift lose anything?"*,
and the fifteen added words were this leaf's two stated additions.

## A lift still needs two proofs

A word multiset is **order-blind by construction** — the exact property the default sequence mode
exists to have — so it cannot distinguish a lift from a shuffle. Prove the two halves separately:

| Half | How | `.3c`'s result |
| --- | --- | --- |
| **content** — did the moved run change? | extract the run from both sides, compare **in order** | 14/14 byte-identical |
| **remainder** — did anything else change? | excise the run from both sides, compare | 54,791 = 54,791 chars |

Each is blind to the other's half. Cite which mode you ran; `--allow-move` alone is not a proof that
the text is unchanged, only that none of it is missing.

## Two control failures worth expecting

- **Saturation.** A control run on a file the proof is *already* firing on tells you nothing — it
  fires whatever you do. `.3c`'s reorder control had to be moved to the file that still passed
  cleanly before it could discriminate. Confirm the proof is **silent before the mutation**.
- **The hard wrap.** A `perl`/`sed` carrier is line-wise and the book is hard-wrapped, so a
  perfectly reasonable substitution matches nothing and `scripts/negative_control.sh` exits `9`.
  That is the harness working (decision [`0047`](../decisions/0047-negative-control-carrier-is-the-mutation.md));
  retarget at a span that does not wrap. The same wrap made this leaf's *first* book-only
  measurement wrong — see [[matched-mutation-is-not-the-intended-mutation]] and
  [[paragraph-split-can-cut-a-sentence-in-half]].
