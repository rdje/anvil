---
id: matched-mutation-is-not-the-intended-mutation
title: An asserted substitution count proves the experiment **ran**, not that it ran the **intended** experiment — check a control's carrier against the rendered artifact, never against your intent for the regex
answers:
  - "my negative control applied but the check stayed silent — is the check blind"
  - "negative_control.sh said applied and the proof did not fire"
  - "how do I know my mutation mutated what I meant"
  - "is a substitution count of 1 enough to trust a control"
  - "why did dedenting a markdown list continuation not break the list"
  - "what is a lazy continuation in mdbook"
  - "my control passed but I changed the wrong whitespace"
  - "my scripted edit asserted the string was present and still edited nothing"
  - "is `assert needle in text` enough before a scripted replace"
date: 2026-08-02
status: current
tags: [testing, control, evidence, gate-quality, mdbook, markdown, gotcha]
evidence: docs/tasks/BOOK-PARAGRAPH-BLOBS.md (`.4` Findings — the control that failed with an innocent instrument, and the carrier table); docs/decisions/0047-negative-control-carrier-is-the-mutation.md (the count assertion this refines); scripts/negative_control.sh (`apply` exits 9 on a zero-count substitution)
reverify: "scripts/negative_control.sh probe book/src/knobs.md 's{(\\n\\n)(  \\*\\*Default `2012` is the honest floor\\.\\*\\*.*?)(\\n\\n)}{ $1 . ($2 =~ s/^  //mgr) . $3 }se' fires sh -c 'mdbook build book >/dev/null 2>&1 && scripts/book_list_signature.py --compare .cache/book-blob/li-baseline.json'   # save the baseline first with --save. Capturing (\\n) instead of (\\n\\n) still substitutes (count 1, `applied`) but deletes the blank line, so mdBook lazily continues the paragraph inside the same <li> and the proof is correctly SILENT."
---

**`scripts/negative_control.sh` guarantees the substitution *matched*. It cannot guarantee the
substitution *meant what you meant*.** Decision [`0047`](../decisions/0047-negative-control-carrier-is-the-mutation.md)
closed the loud failure — `sed -i` / `perl -pi -e` exit `0` whether or not the pattern matched, so a
mistyped expression leaves the tree unchanged and the resulting green reads exactly like a control
that correctly did not fire. The count assertion fixes that.

**The quieter failure survives it.** A regex can match, mutate exactly one place, report `applied` —
and produce a *different* mutation from the one the experiment needs. The count is `1` either way,
and the verdict that comes back looks like a finding about the check.

**The episode (`BOOK-PARAGRAPH-BLOBS.4`).** A control meant to strip a markdown list item's
continuation indent, so its text escapes the `<li>`:

```perl
s{(\n)(  \*\*Default `2012`…)(\n\n)}{ $1 . ($2 =~ s/^  //mgr) . $3 }se   # WRONG
s{(\n\n)(  \*\*Default `2012`…)(\n\n)}{ $1 . ($2 =~ s/^  //mgr) . $3 }se  # right
```

Capturing `(\n)` consumes the **second** newline of the blank line, so the replacement deletes the
blank line along with the indent. **mdBook then reads the dedented text as a *lazy continuation* of
the preceding paragraph** — still inside the same `<li>`. List structure genuinely did not change, so
`scripts/book_list_signature.py` was right to stay silent. Ten minutes were spent suspecting the
instrument; the instrument was innocent.

**The rule.** Before reading a control's verdict, confirm the mutation produced the *state* the
experiment is about — inspect the rendered artifact, not the diff and not the regex. `applied` means
the file changed; it never means the intended thing changed.

**The same root cause outside negative controls: `assert needle in text` before a scripted replace.**
It proves the needle exists *somewhere in the file*, not that it exists *at the site you are about to
edit*. Twice in `BOOK-PARAGRAPH-BLOBS` a Python edit over `docs/tasks/*.md` passed its assertion and
changed nothing or changed the wrong block — once writing a commit hash into the **previous leaf's**
`Commit:` field (it matched the first `` Commit: `pending` ``), once asserting on a sentence that also
appears in a verification-log row, so the multi-line replace silently found no match.

Two habits remove the class outright:

- **Anchor structurally, not textually.** Locate the record first (`lines[i] == '- ID: \`…3a\`'`),
  assert the field's line, then assign it. There is no needle to be ambiguous.
- **Assert the count, then assert the result.** `text.count(needle) == 1` before replacing, and
  re-read the field afterwards. A silent zero-replacement is the file-edit twin of a
  zero-count substitution.

**A second, related trap from the same leaf.** Comparing two runs of an instrument, compare the
**measurement**, not the whole document. `scripts/book_prose_census.py --json` carries both a
measurement (sizes, counts, mass) and a *source locator* (`chapter.md:line`), and re-wrapping a
source line legitimately moves line numbers while the rendered book is byte-identical. Diffed whole,
that control fires for a reason that is not the reason you are testing.

Sits alongside [[negative-control-must-be-able-to-fail]] (*a control that passes first try deserves
suspicion*) and [[coverage-check-vacuity]] (*prove it fires on the right input*). Those two ask
whether the **check** is capable. This one asks whether the **carrier** is what you think it is —
and it is the leg that is checked least, because the tooling reports success.
