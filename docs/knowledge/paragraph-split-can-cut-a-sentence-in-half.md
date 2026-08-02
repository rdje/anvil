---
id: paragraph-split-can-cut-a-sentence-in-half
title: A paragraph splitter that breaks **before** the sentence-final token cuts a sentence in half — and every instrument in the kit either cannot see it or **rewards** it
answers:
  - "my rendered paragraph starts mid-sentence"
  - "a book paragraph ends with a dangling word and the next one starts lowercase"
  - "the word-identity proof passed but the prose is broken"
  - "how do I check a whitespace-only book edit did not split a sentence"
  - "why did the prose census not catch a bad paragraph break"
  - "is a smaller worst-block number always an improvement"
  - "what proof is missing when I insert blank lines into markdown"
  - "mdbook build exits 0 on a paragraph break inside a sentence"
date: 2026-08-02
status: current
tags: [mdbook, markdown, book, live-doc, evidence, gate-quality, gotcha]
evidence: docs/tasks/BOOK-PARAGRAPH-BLOBS.md (`.3b` Findings — the four breaks `.1` shipped, and the 4-of-2,181 census that bounded them); git show df7bc6e -- book/src/architecture.md (the four `+` hunks that introduced them)
reverify: "python3 - <<'PY'  # 0 = clean; any hit is a break inside a sentence\nimport re,pathlib\nbad=0\nfor p in sorted(pathlib.Path('book/src').rglob('*.md')):\n    L=p.read_text().splitlines()\n    for i,l in enumerate(L):\n        if l.strip() or i==0 or i+1>=len(L): continue\n        b,a=L[i-1].rstrip(),L[i+1]\n        if b and a and not re.search(r'[.!?:;][\\\"'`)\\]]*$',b) and not re.match(r'^\\s*([A-Z0-9`*_\\[(|#>-]|$)',a):\n            print(f'{p}:{i}'); bad+=1\nprint('breaks inside a sentence:',bad)\nPY"
---

**Splitting a wall-of-text paragraph is a whitespace edit, so it feels safe. It is not.** In
hard-wrapped source the sentence boundary you want to break at is almost never at a line
boundary, so a splitter must choose *which* line break to promote to a blank line — and choosing
the one **before** the sentence-final token leaves the rendered book with a paragraph that ends
mid-clause and a next paragraph that starts mid-clause:

```markdown
840/0 pass-fail in Verilator plus both repo-owned Yosys      <- paragraph ends here

modes). That report banks wrapper exact / reuse / …          <- next paragraph starts here
```

`BOOK-PARAGRAPH-BLOBS.1` shipped **four** of these in `book/src/architecture.md` (`df7bc6e`) —
`Yosys` / `modes).` once and `downstream tool` / `bank.` three times.

**Why nothing caught it — this is the part worth remembering.** The kit that leaf carried had four
instruments, and the defect is invisible to all four:

| Instrument | Verdict on a sentence cut in half | Why |
| --- | --- | --- |
| `prove_words_unchanged.py` | **passes** | a newline and a blank line both normalize to one space, so the word sequence is untouched |
| `book_list_signature.py` | **passes** | it watches `<li>` structure; these were `<p>` |
| `mdbook build` | **exit 0** | two paragraphs are perfectly legal markdown |
| `book_prose_census.py` | **improves** | the block got *smaller*, which is the number the leaf was trying to move |

The fourth row is the trap. **A splitter's own success metric goes up when it cuts a sentence in
half**, because worst-block size and oversized mass both fall. An instrument that rewards the
defect it should catch is worse than one that is merely blind to it.

**The check that does see it** is a source-level predicate, not a rendered one: at every blank line,
the text before must *end* a sentence (sentence-final punctuation, a table row, a heading, a fence, a
list marker) or the text after must *begin* one (capital, digit, code span, markdown marker). Run over
the whole book it gives a denominator — **4 of 2,181 breaks**, all four in one chapter — which is what
turns "the splitter has a bug" into a bounded, closable finding.

**The repair is not to delete the break.** Rejoining the sentence puts the wall of text back. Move the
break to the true sentence boundary — `…downstream tool bank.` ⟶ blank ⟶ `It also proves…` — which
preserves the word sequence (so the word proof still passes) *and* the paragraph split.

Related: [[matched-mutation-is-not-the-intended-mutation]] (a control's carrier must be checked
against the rendered artifact), and [[practice-survives-as-a-by-product-not-by-a-gate]] — nothing an
author is forced to do reports that a paragraph now begins mid-sentence.
