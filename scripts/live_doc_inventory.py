#!/usr/bin/env python3
"""Derive the live-document inventory and its route graph from the tree.

LIVE_DOCUMENT_SIZE_CONTAINMENT.md adoption checklist step 2:
"Inventory every live document, generated view, collection, route, and historical
terminal; follow routes transitively."

WHY THIS IS A SCRIPT AND NOT A LIST. A hand-maintained inventory mirroring the set of
tracked documents is derivable, growth-coupled and silent -- decision 0033's three-part
test, passed on all three counts -- so writing one down would create the exact shadow
this project repairs by deletion. The authority is `git ls-files`; this script is the
derivation, and every consumer (the .6 registry, the .7 checker) reads it rather than a
copy of it.

TWO ROUTE KINDS, DELIBERATELY SEPARATED. The doctrine requires it:

  "Reader navigation and author overflow are different route kinds. A reader may
   legitimately navigate to immutable history. [...] Derive author candidates from
   enforcers' emitted failure guidance as well as hand-authored route data; an
   undeclared path-shaped hint is a real pressure edge and must fail closed."

  * NAV   -- a markdown link from one tracked document to another. Harmless to point at
             history: a reader choosing to open an archive is not pressure.
  * FLOW  -- a destination a *tool* tells an *author* to move content INTO. These are the
             pressure edges, and ANVIL's are emitted by the doctrine checks themselves
             (a cap failure prints "move it to X"). An author does what the failing gate
             says, so a hint naming an uncontained neighbour is a containment hole no
             amount of prose elsewhere closes.

Deterministic by construction: sorted output, no clock, no randomness, repo-relative
paths only. Reads the repository and mutates nothing (DOCTRINE_ENFORCEMENT.md §4).
"""

from __future__ import annotations

import argparse
import json
import os
import re
import subprocess
import sys
from collections import defaultdict

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

# A "collection" is a directory whose members are one-unit-per-file and are read by
# lookup rather than by reading through. Declared here because membership is derived
# but the GROUPING is a judgement -- and a judgement in a script is reviewable, whereas
# the same judgement spread across prose is not.
COLLECTIONS = (
    "docs/tasks/",
    "docs/decisions/",
    "docs/knowledge/",
    "book/src/",
    "docs/evidence/",
)

# Files that emit author-overflow guidance. Derived from the enforcement surface rather
# than listed by hand: anything that can fail a commit can tell an author where to put
# the bytes that made it fail.
ENFORCER_GLOBS = ("scripts/", "knowledge-map/scripts/", ".githooks/")

FENCE_RE = re.compile(r"^\s*(```|~~~)")
# Markdown inline links. Whole-file (not line-wise) because a link's TEXT may wrap across
# a newline -- the failure BOOK-LINK-INTEGRITY.3 measured and fixed in its own extractor.
LINK_RE = re.compile(r"\[[^\]]*\]\(\s*(<[^>]*>|[^)\s]+)")
# A path-shaped token inside enforcer output: a repo-relative path or a directory.
HINT_RE = re.compile(r"\b((?:[A-Za-z0-9_.-]+/)*[A-Za-z0-9_.-]+\.(?:md|jsonl|json|sh|rs|toml)|(?:[A-Za-z0-9_.-]+/)+)")


def sh(*args: str) -> str:
    return subprocess.run(args, cwd=ROOT, check=True, capture_output=True, text=True).stdout


def tracked() -> list[str]:
    return sorted(p for p in sh("git", "ls-files").splitlines() if p)


def strip_fences(text: str) -> str:
    """Blank out fenced code blocks, keeping line count stable.

    Load-bearing rather than tidy: a documented example link inside a fence is an
    example, not a route, and counting it would cry wolf -- the same reasoning
    check_book_link_targets.sh proved both ways with a neutered fence mask.
    """
    out, fenced = [], False
    for line in text.split("\n"):
        if FENCE_RE.match(line):
            fenced = not fenced
            out.append("")
            continue
        out.append("" if fenced else line)
    return "\n".join(out)


def measure(path: str) -> dict:
    """The five axes, measured on every surface -- never on a sample.

    0040 named maximum content-line bytes as "the axis nobody measures" and then measured
    it on one file; docs/TASK_TREE.md's 39,591-byte line is what that omission cost.
    Raw content bytes exclude LF and an optional preceding CR, per the neutral body.
    """
    with open(os.path.join(ROOT, path), "rb") as fh:
        raw = fh.read()
    lines = raw.split(b"\n")
    if lines and lines[-1] == b"":
        lines.pop()
    widest = max((len(ln[:-1] if ln.endswith(b"\r") else ln) for ln in lines), default=0)
    return {"lines": len(lines), "bytes": len(raw), "max_line_bytes": widest}


def nav_edges(docs: list[str]) -> list[tuple[str, str]]:
    """Reader navigation: tracked doc -> tracked file, resolved relative to the source."""
    edges = set()
    trackedset = set(tracked())
    for src in docs:
        text = strip_fences(open(os.path.join(ROOT, src), encoding="utf-8", errors="replace").read())
        for m in LINK_RE.finditer(text):
            target = m.group(1).strip("<>")
            if re.match(r"^[a-zA-Z][a-zA-Z0-9+.-]*:", target) or target.startswith("#"):
                continue  # external scheme or same-page anchor: not a repository route
            target = target.split("#", 1)[0].split("?", 1)[0]
            if not target:
                continue
            resolved = os.path.normpath(os.path.join(os.path.dirname(src), target))
            if resolved in trackedset:
                edges.add((src, resolved))
            elif resolved + "/" in COLLECTIONS or os.path.isdir(os.path.join(ROOT, resolved)):
                edges.add((src, resolved.rstrip("/") + "/"))
    return sorted(edges)


def flow_edges() -> list[tuple[str, str]]:
    """Author overflow: destinations named in enforcer output that an author must obey."""
    edges = set()
    enforcers = [p for p in tracked() if p.startswith(ENFORCER_GLOBS) and not p.endswith(".md")]
    for src in enforcers:
        try:
            text = open(os.path.join(ROOT, src), encoding="utf-8", errors="replace").read()
        except OSError:
            continue
        for line in text.split("\n"):
            # Only lines that PRINT guidance to a human; a path in ordinary logic is not
            # an instruction to move content there.
            if not re.search(r"(printf|echo|note|FAIL|hint|->)", line):
                continue
            if not re.search(r"->|move|route|belongs|instead", line, re.I):
                continue
            for m in HINT_RE.finditer(line):
                dest = m.group(1)
                if dest.startswith(("scripts/", ".githooks/", "knowledge-map/scripts/")):
                    continue  # naming another check is not an overflow destination
                # Reject a single-character path segment: hint text says "layer B", and
                # "B/" is prose caught by a path shape, not a directory. This is the ONLY
                # filter here, and it is a parse fix rather than a judgement -- the
                # extractor stays deliberately OVER-collecting, because the doctrine makes
                # an undeclared path-shaped hint fail closed. Deciding which candidates are
                # genuine pressure edges is a DECLARATION, and declarations belong in .6's
                # registry where they are reviewable; a hand-curated exclusion list here
                # would be the shadow 0033 repairs by deletion.
                if any(len(seg) < 2 for seg in dest.rstrip("/").split("/")):
                    continue
                edges.add((src, dest))
    return sorted(edges)


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--json", action="store_true", help="emit the full inventory as JSON")
    args = ap.parse_args()

    all_tracked = tracked()
    docs = [p for p in all_tracked if p.endswith(".md")]

    surfaces: dict[str, dict] = {}
    coll_members: dict[str, list[str]] = defaultdict(list)
    for p in docs:
        owner = next((c for c in COLLECTIONS if p.startswith(c)), None)
        if owner:
            coll_members[owner].append(p)
        else:
            surfaces[p] = {"kind": "singleton", **measure(p)}
    for c, members in coll_members.items():
        per = [measure(m) for m in members]
        surfaces[c] = {
            "kind": "collection",
            "files": len(members),
            "lines": sum(x["lines"] for x in per),
            "bytes": sum(x["bytes"] for x in per),
            "max_line_bytes": max(x["max_line_bytes"] for x in per),
            "largest_member": max(members, key=lambda m: measure(m)["bytes"]),
        }

    nav, flow = nav_edges(docs), flow_edges()

    # THE RESIDUAL. An inventory that silently omits a file is worse than none, so the
    # count-floor here is an identity: every tracked *.md is either a singleton surface
    # or a member of exactly one collection. Anything else is a bug in this script.
    covered = len([p for p in docs if p in surfaces]) + sum(len(v) for v in coll_members.values())
    residual = len(docs) - covered

    if args.json:
        json.dump(
            {
                "surfaces": dict(sorted(surfaces.items())),
                "routes": {"nav": [list(e) for e in nav], "flow": [list(e) for e in flow]},
                "residual": residual,
            },
            sys.stdout,
            indent=2,
            sort_keys=True,
        )
        print()
        return 0 if residual == 0 else 1

    print(f"{'SURFACE':<34}{'KIND':>11}{'FILES':>7}{'LINES':>9}{'BYTES':>12}{'MAXLINE':>9}")
    for name, s in sorted(surfaces.items(), key=lambda kv: -kv[1]["bytes"]):
        print(
            f"{name:<34}{s['kind']:>11}{s.get('files', 1):>7}"
            f"{s['lines']:>9}{s['bytes']:>12}{s['max_line_bytes']:>9}"
        )
    print(
        f"\ntracked *.md: {len(docs)}  |  surfaces: {len(surfaces)}"
        f" ({sum(1 for s in surfaces.values() if s['kind']=='collection')} collections)"
        f"  |  total bytes: {sum(measure(p)['bytes'] for p in docs)}"
    )
    print(f"routes: {len(nav)} reader-navigation, {len(flow)} author-overflow")
    print(f"residual (tracked *.md not inventoried): {residual}")
    if flow:
        print("\nAUTHOR-OVERFLOW EDGES (the pressure edges):")
        for src, dest in flow:
            print(f"  {src}  ->  {dest}")
    return 0 if residual == 0 else 1


if __name__ == "__main__":
    sys.exit(main())
