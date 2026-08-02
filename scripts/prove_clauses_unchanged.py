#!/usr/bin/env python3
"""Prove a run-on -> markdown-list conversion preserved every clause.

`prove_words_unchanged.py` is the right proof for a WHITESPACE-only edit: it
requires byte-identity after collapsing whitespace. Converting a run-on
enumeration into a list is not whitespace-only — it removes the separating
commas and the `and`/`plus` connectives and adds `- ` markers — so the word
proof necessarily fires, and a proof that always fires proves nothing.

This proof answers the question that actually matters for that conversion:
**was any clause dropped, invented, reworded, or reordered?** It normalizes
both sides by removing exactly what a list conversion is allowed to change —
list markers, clause separators, and standalone connectives — and then
requires the remaining word SEQUENCE to be byte-identical. So

    A, B, and C.                    ->    - A
                                          - B
                                          - C

passes, while dropping B, rewording B, or moving B after C does not.

It is order-sensitive by construction, which is what makes it stronger than
the clause *set* comparison it replaces: a set is blind to reordering, and a
reordered capability register is a changed claim about what a gate proved.

**Stated blind spot.** Because standalone `and`/`plus`/`or`/`including`/
`then`/`also` are dropped from both sides, an edit that consists *only* of
adding or removing one of those words is invisible here. That is the exact
token class a list conversion legitimately edits, which is why it is dropped;
it means this proof must not be cited as evidence that no word changed. For
that claim, use `prove_words_unchanged.py`, which permits nothing.

Exit 0 when every file's normalized sequence is identical, 1 when one is not,
2 on a usage or git error.
"""
from __future__ import annotations

import argparse
import re
import subprocess
import sys
from pathlib import Path

# Leading list marker on a source line: `- `, `* `, `+ `, `1. `.
LIST_MARKER = re.compile(r"^[ \t]*(?:[-*+]|\d+\.)[ \t]+", re.MULTILINE)

# Clause separators a list conversion removes. `.` counts only when it ends a
# word, so `manifest.json`, `1.28` and `depth-3..7` stay whole.
SEPARATOR = re.compile(r"[,;:]|\.(?=\s|$)")

# Standalone connectives a list conversion legitimately drops. See the blind
# spot noted above.
CONNECTIVE = re.compile(r"\b(?:and|plus|then|also|including|or)\b", re.IGNORECASE)

# Inline code spans are lifted out before any of the above, so a comma or colon
# inside `0=4:4,1=2:2` cannot manufacture a boundary and `and` inside an
# identifier cannot be mistaken for a connective.
CODE_SPAN = re.compile(r"`[^`\n]*`")


def normalize(text: str) -> str:
    spans: list[str] = []

    def lift(m: re.Match[str]) -> str:
        spans.append(m.group(0))
        return f"\x00{len(spans) - 1}\x00"

    text = CODE_SPAN.sub(lift, text)
    text = LIST_MARKER.sub("", text)
    text = SEPARATOR.sub(" ", text)
    text = CONNECTIVE.sub(" ", text)
    text = re.sub(r"\s+", " ", text).strip()
    return re.sub(r"\x00(\d+)\x00", lambda m: spans[int(m.group(1))], text)


def at_ref(ref: str, path: Path, root: Path) -> str:
    rel = path.resolve().relative_to(root)
    proc = subprocess.run(
        ["git", "-C", str(root), "show", f"{ref}:{rel}"],
        capture_output=True,
        text=True,
    )
    if proc.returncode != 0:
        print(f"error: git show {ref}:{rel} failed: {proc.stderr.strip()}",
              file=sys.stderr)
        raise SystemExit(2)
    return proc.stdout


def locate(before: str, after: str, context: int) -> str:
    """Report the first divergence with surrounding text."""
    limit = min(len(before), len(after))
    i = 0
    while i < limit and before[i] == after[i]:
        i += 1
    lo = max(0, i - context)
    return (f"      at normalized offset {i}\n"
            f"      before: …{before[lo:i + context]}…\n"
            f"      after:  …{after[lo:i + context]}…")


def main() -> int:
    ap = argparse.ArgumentParser(
        description=__doc__,
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    ap.add_argument("--ref", default="HEAD", help="git ref to compare against")
    ap.add_argument("--context", type=int, default=90,
                    help="characters of context around a divergence")
    ap.add_argument("paths", nargs="+", help="files to prove")
    args = ap.parse_args()

    root = Path(
        subprocess.run(["git", "rev-parse", "--show-toplevel"],
                       capture_output=True, text=True, check=True).stdout.strip()
    )

    ok = True
    for raw in args.paths:
        path = Path(raw).resolve()
        rel = path.relative_to(root)
        before = normalize(at_ref(args.ref, path, root))
        after = normalize(path.read_text(encoding="utf-8"))

        if before == after:
            print(f"{rel}: OK ({len(before)} normalized chars)")
            continue
        ok = False
        print(f"{rel}: CLAUSES DIFFER "
              f"({len(before)} -> {len(after)} normalized chars)")
        print(locate(before, after, args.context))

    print()
    print("PROOF: every clause preserved, in order" if ok
          else "PROOF FAILED: a clause was dropped, invented, reworded, or moved")
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main())
