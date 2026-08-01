#!/usr/bin/env python3
"""book_list_signature.py — prove a book edit did not change list STRUCTURE.

WHY THIS EXISTS (BOOK-PARAGRAPH-BLOBS.1, the regression it caught). Splitting a
paragraph inside a markdown list item requires preserving the item's two-space
continuation indent. Drop it and the continuation silently escapes its `<li>`,
becomes a top-level paragraph, and ends the list. `mdbook build` exits 0. The
words are all still there in the same order, so the whitespace-normalized word
proof (`scripts/prove_words_unchanged.py`) PASSES — it collapses exactly the
whitespace that carries list nesting, so it is blind to this by construction.

This is the second, independent proof: the count of rendered `<li>` elements per
chapter plus a SHA-256 over their whitespace-normalized text. It fires on the
escape the word proof misses. Both were negative-controlled at `.1`; on a
sabotage that strips one list continuation's indent, this signature FIRES
(`n_li` unchanged, content SHA changed) and the word proof passes.

Two proofs, because either alone ships a defect:
  words     catches a changed word that leaves structure intact
  structure catches changed structure that leaves every word intact

USAGE
  mdbook build book
  scripts/book_list_signature.py --save .cache/book-list/before.json
  …edit book/src/*.md…
  mdbook build book
  scripts/book_list_signature.py --compare .cache/book-list/before.json

EXIT STATUS
  0  saved, or compared and identical
  1  compared and DIFFERENT (or an environment error) — list structure moved
"""

from __future__ import annotations

import argparse
import glob
import hashlib
import json
import os
import re
import sys
from html.parser import HTMLParser

EXCLUDED_PAGES = frozenset({"print.html", "404.html"})
# Separator between items in the hashed blob: a unit separator cannot occur in
# rendered prose, so it cannot be forged by item text that merely looks joined.
ITEM_SEP = "\x1f"


class ListSignature(HTMLParser):
    """Collect the visible text of every <li> under <main>."""

    def __init__(self) -> None:
        super().__init__(convert_charrefs=True)
        self.in_main = 0
        self.open_items: list[list[str]] = []
        self.items: list[str] = []

    def handle_starttag(self, tag: str, attrs) -> None:
        if tag == "main":
            self.in_main += 1
        elif self.in_main and tag == "li":
            self.open_items.append([])

    def handle_endtag(self, tag: str) -> None:
        if tag == "main":
            self.in_main = max(0, self.in_main - 1)
        elif self.in_main and tag == "li" and self.open_items:
            self.items.append(re.sub(r"\s+", " ", "".join(self.open_items.pop())).strip())

    def handle_data(self, data: str) -> None:
        if self.in_main and self.open_items:
            self.open_items[-1].append(data)


def signature(book_out: str) -> dict:
    out: dict[str, dict] = {}
    for page in sorted(glob.glob(os.path.join(book_out, "*.html"))):
        name = os.path.basename(page)
        if name in EXCLUDED_PAGES:
            continue
        with open(page, encoding="utf-8", errors="replace") as handle:
            parser = ListSignature()
            parser.feed(handle.read())
        # Sorted, so a pure reordering within a chapter is not reported as a
        # content change — the proof is about text escaping its item, not order.
        blob = ITEM_SEP.join(sorted(parser.items))
        out[name] = {
            "n_li": len(parser.items),
            "sha256": hashlib.sha256(blob.encode("utf-8")).hexdigest()[:16],
        }
    return out


def compare(baseline: dict, current: dict) -> int:
    differences = 0
    for name in sorted(set(baseline) | set(current)):
        was, now = baseline.get(name), current.get(name)
        if was == now:
            continue
        differences += 1
        if was is None:
            print(f"FIRES  {name}: chapter added ({now['n_li']} <li>)")
        elif now is None:
            print(f"FIRES  {name}: chapter removed (was {was['n_li']} <li>)")
        else:
            detail = []
            if was["n_li"] != now["n_li"]:
                detail.append(f"n_li {was['n_li']} -> {now['n_li']}")
            if was["sha256"] != now["sha256"]:
                detail.append(f"content sha {was['sha256']} -> {now['sha256']}")
            print(f"FIRES  {name}: {', '.join(detail)}")
    total_now = sum(v["n_li"] for v in current.values())
    if differences:
        print(
            f"\nlist structure CHANGED in {differences} chapter(s) "
            f"({total_now} <li> across {len(current)} chapters now)."
        )
        return 1
    print(
        f"OK  list structure identical: {total_now} <li> across "
        f"{len(current)} chapters, every chapter's content SHA unchanged."
    )
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Signature of the rendered book's list structure.",
        epilog="Run `mdbook build book` before each invocation.",
    )
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--save", metavar="FILE", help="write the current signature to FILE")
    mode.add_argument("--compare", metavar="FILE", help="compare the current signature to FILE")
    parser.add_argument(
        "--book",
        default="book",
        help="book directory, relative to the repository root (default: book)",
    )
    args = parser.parse_args()

    # Repo-root-derived; every path this tool writes stays on the repo volume
    # (CLAUDE.md §12/§13, decision 0031).
    root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    book_out = os.path.join(root, args.book, "book-out")
    if not os.path.isdir(book_out):
        print(
            f"book_list_signature: {os.path.relpath(book_out, root)} does not exist — "
            f"run `mdbook build {args.book}` first.",
            file=sys.stderr,
        )
        return 1

    current = signature(book_out)
    if not current:
        print("book_list_signature: parsed 0 chapters — the build is empty.", file=sys.stderr)
        return 1

    if args.save:
        target = args.save if os.path.isabs(args.save) else os.path.join(root, args.save)
        os.makedirs(os.path.dirname(target) or ".", exist_ok=True)
        with open(target, "w", encoding="utf-8") as handle:
            json.dump(current, handle, indent=1, sort_keys=True)
            handle.write("\n")
        total = sum(v["n_li"] for v in current.values())
        print(
            f"wrote {os.path.relpath(target, root)}: {total} <li> "
            f"across {len(current)} chapters"
        )
        return 0

    source = args.compare if os.path.isabs(args.compare) else os.path.join(root, args.compare)
    try:
        with open(source, encoding="utf-8") as handle:
            baseline = json.load(handle)
    except OSError as exc:
        print(f"book_list_signature: cannot read baseline {args.compare}: {exc}", file=sys.stderr)
        return 1
    return compare(baseline, current)


if __name__ == "__main__":
    sys.exit(main())
