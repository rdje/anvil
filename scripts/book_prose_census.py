#!/usr/bin/env python3
"""book_prose_census.py — measure wall-of-text blocks in the RENDERED book.

WHY THIS EXISTS (BOOK-PARAGRAPH-BLOBS). The owner reviews the book, not the
code (`COMMIT.md` §9), so a readability defect in the rendered HTML is a defect
in the only surface they see. Nothing an author is forced to do reports how long
a paragraph has become: `mdbook build` exits 0 on a 22,908-character paragraph,
and every diff that grew it was three lines long. This instrument is the missing
report — see [[practice-survives-as-a-by-product-not-by-a-gate]].

WHAT A "BLOCK" IS. One rendered block-level PROSE container under `<main>`:
`<p>`, `<li>`, `<blockquote>`, `<td>`, `<th>`, `<dd>`. Three rules, each of which
was earned by an instrument that got it wrong first (BOOK-PARAGRAPH-BLOBS.1):

  1. CODE IS EXCLUDED. `<pre>` is meant to be long; counting it as prose is what
     made the first instrument report a 7,913-character "paragraph" that was a
     Rust struct definition.
  2. NOT `<p>`-ONLY. The first census counted `<p>` alone and was blind to five
     of eleven oversized blocks — including a 10,781-character `<li>`, the
     second-worst blob in the book. A wall of text does not become readable by
     being inside a list item.
  3. TEXT BELONGS TO THE INNERMOST OPEN BLOCK. A `<li>` containing a `<p>` is
     not charged for the `<p>`'s text. Without this, nesting double-counts and
     every outer container looks oversized.

`print.html` and `404.html` are excluded as generated chrome: `print.html`
concatenates every chapter, so counting it double-counts the whole book.

REPAIRABILITY. The census also states, per block, what could actually fix it —
`.1` measured that "insert a blank line" is not available for most of them:

  SPLITTABLE   several sentences run together in a `<p>`; a blank line at a
               topic boundary is the whole repair, and it changes no words.
  RUN-ON       ONE sentence listing dozens of clauses. Whitespace cannot split a
               sentence. Repairing it needs wording or structure changes.
  LIST-ITEM    inside `<li>`. A break here must preserve the continuation
               indent, or the text silently escapes the `<li>` and ends the list
               — a real regression `.1` shipped once and now guards against with
               `scripts/book_list_signature.py`.
  TABLE-CELL   inside `<td>`/`<th>`. A blank line cannot split a table cell at
               all; `TABLE-RENDER-FIDELITY` owns table well-formedness.

NOT A GATE, DELIBERATELY. This exits 0 whatever it finds, and it is absent from
`scripts/check_doctrines.sh`. Whether anything should *watch* paragraph size is
`BOOK-PARAGRAPH-BLOBS.2`'s question, and decision `0047` prefers removing the
need over watching harder. Turning this into a check means registering it there,
with a derived threshold — the 1,500-char default was read off a distribution
and is a reporting convenience, not a justified limit.

USAGE
  mdbook build book                       # the census reads the RENDERED book
  scripts/book_prose_census.py            # default: threshold 1500, text report
  scripts/book_prose_census.py --threshold 2000
  scripts/book_prose_census.py --json > /path/to/before.json   # machine-readable

EXIT STATUS
  0  the census ran (whatever it found)
  1  environment error — the book is not built, or no chapters were parsed
"""

from __future__ import annotations

import argparse
import glob
import json
import os
import re
import sys
from html.parser import HTMLParser

# Block-level containers that can hold a wall of prose.
BLOCK_TAGS = frozenset({"p", "li", "blockquote", "td", "th", "dd"})
# Subtrees whose text is not prose and must not be measured as prose.
SKIP_TAGS = frozenset({"pre", "script", "style"})
# Generated chrome: print.html concatenates every chapter (double-counting).
EXCLUDED_PAGES = frozenset({"print.html", "404.html"})

DEFAULT_THRESHOLD = 1500


class ProseCensus(HTMLParser):
    """Collect (tag, visible_text) for every prose block under <main>."""

    def __init__(self) -> None:
        super().__init__(convert_charrefs=True)
        self.in_main = 0
        self.in_skip = 0
        self.open_blocks: list[list] = []
        self.blocks: list[tuple[str, str]] = []

    def handle_starttag(self, tag: str, attrs) -> None:
        if tag == "main":
            self.in_main += 1
            return
        if not self.in_main:
            return
        if tag in SKIP_TAGS:
            self.in_skip += 1
            return
        if self.in_skip:
            return
        if tag in BLOCK_TAGS:
            self.open_blocks.append([tag, []])

    def handle_endtag(self, tag: str) -> None:
        if tag == "main":
            self.in_main = max(0, self.in_main - 1)
            return
        if not self.in_main:
            return
        if tag in SKIP_TAGS:
            self.in_skip = max(0, self.in_skip - 1)
            return
        if self.in_skip or tag not in BLOCK_TAGS:
            return
        # Close the innermost block with this tag; mdBook's HTML is well-formed
        # but `<li>` and `<p>` end tags may be implied, so search rather than
        # assume the top of the stack matches.
        for i in range(len(self.open_blocks) - 1, -1, -1):
            if self.open_blocks[i][0] == tag:
                _, parts = self.open_blocks.pop(i)
                # Join with "", never " ": the data chunks either side of an
                # inline `<code>` are adjacent in the rendered output, so a
                # separator here would invent a space per inline element and
                # inflate every block that contains one.
                text = normalize("".join(parts))
                if text:
                    self.blocks.append((tag, text))
                return

    def handle_data(self, data: str) -> None:
        # Rule 3: charge text to the innermost open block only.
        if self.in_main and not self.in_skip and self.open_blocks:
            self.open_blocks[-1][1].append(data)


def normalize(text: str) -> str:
    """Collapse every whitespace run to one space. Rendered length is what a
    reader faces, so source line breaks must not count as structure."""
    return re.sub(r"\s+", " ", text).strip()


def demark(text: str) -> str:
    """Strip the markdown syntax that exists in source but not in rendered text,
    so a rendered needle can be located in the source file."""
    return normalize(re.sub(r"[`*_\[\]|>]", "", text))


def source_anchor(chapter_stem: str, rendered_head: str, src_dir: str) -> str:
    """Best-effort `<file>.md:<line>` for a rendered block.

    Approximate by construction — mdBook renders markdown, so there is no exact
    inverse. The chapter source is flattened into one demarked string with a
    line map (a rendered block routinely spans many hard-wrapped source lines,
    and a table cell spans a row), then the head of the rendered text is located
    in it. Reported as `?` rather than guessed when nothing matches: a wrong line
    number is worse than none.
    """
    path = os.path.join(src_dir, chapter_stem + ".md")
    try:
        with open(path, encoding="utf-8") as handle:
            lines = handle.read().split("\n")
    except OSError:
        return "?"

    flat_parts: list[str] = []
    line_starts: list[tuple[int, int]] = []  # (offset in flat text, 1-based line)
    offset = 0
    for lineno, line in enumerate(lines, 1):
        piece = demark(line)
        line_starts.append((offset, lineno))
        flat_parts.append(piece)
        offset += len(piece) + 1  # the single space the join inserts
    flat = " ".join(flat_parts)

    needle = demark(rendered_head)[:40]
    if len(needle) < 12:
        return "?"
    found = flat.find(needle)
    if found < 0:
        return "?"
    lineno = 1
    for start, candidate in line_starts:
        if start > found:
            break
        lineno = candidate
    return f"{chapter_stem}.md:{lineno}"


def sentence_boundaries(text: str) -> int:
    """Count places a blank line could be inserted without splitting a sentence."""
    return len(re.findall(r"\.\s+(?=[A-Z])", text))


def classify(tag: str, text: str) -> str:
    if tag in ("td", "th"):
        return "TABLE-CELL"
    if tag == "li":
        return "LIST-ITEM"
    return "SPLITTABLE" if sentence_boundaries(text) >= 1 else "RUN-ON"


def run_census(book_out: str, src_dir: str, threshold: int) -> dict:
    pages = sorted(glob.glob(os.path.join(book_out, "*.html")))
    total_blocks = 0
    chapters = 0
    worst_by_chapter: dict[str, int] = {}
    oversized: list[dict] = []

    for page in pages:
        name = os.path.basename(page)
        if name in EXCLUDED_PAGES:
            continue
        with open(page, encoding="utf-8", errors="replace") as handle:
            parser = ProseCensus()
            parser.feed(handle.read())
        if not parser.blocks:
            continue
        chapters += 1
        total_blocks += len(parser.blocks)
        stem = name[:-5]
        worst_by_chapter[name] = max(len(text) for _, text in parser.blocks)
        for tag, text in parser.blocks:
            if len(text) > threshold:
                oversized.append(
                    {
                        "chars": len(text),
                        "tag": tag,
                        "chapter": name,
                        "anchor": source_anchor(stem, text[:120], src_dir),
                        "repairable_by": classify(tag, text),
                        "sentence_breaks": sentence_boundaries(text),
                        "head": text[:70],
                    }
                )

    oversized.sort(key=lambda row: -row["chars"])
    return {
        "threshold": threshold,
        "prose_blocks": total_blocks,
        "chapters": chapters,
        "over_threshold": len(oversized),
        "oversized_mass": sum(row["chars"] for row in oversized),
        "worst_block": oversized[0]["chars"] if oversized else 0,
        "worst_by_chapter": worst_by_chapter,
        "blocks": oversized,
    }


def print_report(result: dict, top_chapters: int) -> None:
    print(
        f"prose blocks scanned (code excluded): {result['prose_blocks']} "
        f"across {result['chapters']} chapters"
    )
    print(f"over {result['threshold']} chars: {result['over_threshold']}")
    print(
        f"worst block: {result['worst_block']} chars; "
        f"total mass in oversized blocks: {result['oversized_mass']}"
    )
    if result["blocks"]:
        print()
        print(f"{'chars':>6}  {'elem':<6} {'repair':<11} {'chapter':<24} source anchor")
        for row in result["blocks"]:
            print(
                f"{row['chars']:6d}  <{row['tag']}>{'':<{max(0, 4 - len(row['tag']))}} "
                f"{row['repairable_by']:<11} {row['chapter']:<24} {row['anchor']}"
            )
            print(f"{'':8}“{row['head']}…”")
    if top_chapters:
        print()
        print(f"worst prose block per chapter (top {top_chapters}):")
        ranked = sorted(result["worst_by_chapter"].items(), key=lambda kv: -kv[1])
        for name, worst in ranked[:top_chapters]:
            print(f"  {worst:6d}  {name}")


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Census of wall-of-text blocks in the rendered mdBook.",
        epilog="Run `mdbook build book` first; this measures the RENDERED book.",
    )
    parser.add_argument(
        "--threshold",
        type=int,
        default=DEFAULT_THRESHOLD,
        help=f"oversized-block threshold in characters (default {DEFAULT_THRESHOLD}; "
        "a reporting convenience read off a distribution, not a justified limit)",
    )
    parser.add_argument(
        "--book",
        default="book",
        help="book directory, relative to the repository root (default: book)",
    )
    parser.add_argument("--json", action="store_true", help="emit JSON instead of a report")
    parser.add_argument(
        "--top",
        type=int,
        default=10,
        help="how many chapters to rank by worst block (default 10; 0 to omit)",
    )
    args = parser.parse_args()

    # Repo-root-derived, never an absolute path baked into the file (CLAUDE.md §12).
    root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    book_out = os.path.join(root, args.book, "book-out")
    src_dir = os.path.join(root, args.book, "src")

    if not os.path.isdir(book_out):
        print(
            f"book_prose_census: {os.path.relpath(book_out, root)} does not exist — "
            f"run `mdbook build {args.book}` first.",
            file=sys.stderr,
        )
        return 1

    result = run_census(book_out, src_dir, args.threshold)
    if result["chapters"] == 0:
        print(
            f"book_prose_census: parsed 0 chapters under {os.path.relpath(book_out, root)} — "
            "the build is empty or the HTML shape changed.",
            file=sys.stderr,
        )
        return 1

    if args.json:
        json.dump(result, sys.stdout, indent=2, sort_keys=True)
        sys.stdout.write("\n")
    else:
        print_report(result, args.top)
    return 0


if __name__ == "__main__":
    sys.exit(main())
