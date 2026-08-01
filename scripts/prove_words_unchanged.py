#!/usr/bin/env python3
"""prove_words_unchanged.py — prove an edit changed WHITESPACE ONLY.

WHY THIS EXISTS (BOOK-PARAGRAPH-BLOBS.1). The obvious proof for a "insert blank
lines only" repair — `git diff --ignore-blank-lines` shows no content hunks — is
too narrow to be usable. In hard-wrapped source, the sentence that begins a new
topic almost always begins MID-LINE, so splitting there requires moving a line
break, which that diff reports as a content change. Held to the letter, the
criterion permits a break only at the three places a sentence happened to start
a line, and would have produced a worse book.

This is the stronger replacement: collapse every whitespace run in the file to a
single space and require the result to be byte-identical to the same file at a
git ref. That permits arbitrary re-wrapping and forbids any word change.
Negative-controlled at `.1`: it passes on the real repair and FAILS on a
one-character word change, with the divergence located and printed.

ITS BLIND SPOT, STATED. It collapses exactly the whitespace that carries list
nesting, so it CANNOT see a continuation that lost its indent and escaped its
`<li>`. That is a real regression `.1` shipped once. Pair it with
`scripts/book_list_signature.py`, which catches precisely that.

USAGE
  scripts/prove_words_unchanged.py book/src/architecture.md book/src/ir.md
  scripts/prove_words_unchanged.py --ref HEAD~3 book/src/hierarchy.md

EXIT STATUS
  0  every named file has identical words at the ref and in the worktree
  1  at least one file's words changed (or a file could not be read)
"""

from __future__ import annotations

import argparse
import re
import subprocess
import sys

CONTEXT = 70


def normalized(text: str) -> str:
    return re.sub(r"\s+", " ", text).strip()


def at_ref(ref: str, path: str) -> str | None:
    result = subprocess.run(
        ["git", "show", f"{ref}:{path}"],
        capture_output=True,
        text=True,
        check=False,
    )
    if result.returncode != 0:
        return None
    return result.stdout


def report_divergence(old: str, new: str) -> None:
    for index, (a, b) in enumerate(zip(old, new)):
        if a != b:
            lo = max(0, index - CONTEXT)
            print(f"       first divergence at normalized char {index}:")
            print(f"       ref: …{old[lo:index + CONTEXT]}…")
            print(f"       now: …{new[lo:index + CONTEXT]}…")
            return
    shorter, longer = sorted((old, new), key=len)
    print(
        f"       one is a prefix of the other (ref {len(old)} chars, "
        f"now {len(new)} chars); first extra text: "
        f"…{longer[len(shorter):len(shorter) + CONTEXT]}…"
    )


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Prove an edit changed whitespace only, against a git ref."
    )
    parser.add_argument("--ref", default="HEAD", help="git ref to compare against (default HEAD)")
    parser.add_argument("paths", nargs="+", help="repository-relative file paths")
    args = parser.parse_args()

    ok = True
    for path in args.paths:
        old_raw = at_ref(args.ref, path)
        if old_raw is None:
            print(f"MISS  {path}   (not present at {args.ref})")
            ok = False
            continue
        try:
            with open(path, encoding="utf-8") as handle:
                new_raw = handle.read()
        except OSError as exc:
            print(f"MISS  {path}   ({exc})")
            ok = False
            continue

        old, new = normalized(old_raw), normalized(new_raw)
        same = old == new
        ok = ok and same
        verdict = "OK  " if same else "DIFF"
        print(f"{verdict}  {path}   ({len(old)} -> {len(new)} normalized chars)")
        if not same:
            report_divergence(old, new)

    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main())
