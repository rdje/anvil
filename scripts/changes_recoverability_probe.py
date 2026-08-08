#!/usr/bin/env python3
"""Measure how much of CHANGES.md is recoverable from git log + the task trees.

LIVE_DOCUMENT_SIZE_CONTAINMENT.md, "choose the storage topology from the information
role", rule 5: "Content already present in a richer canonical source is proved
duplicate, then removed with a link rather than copied into another store."

PROVED, not assumed. The owner's directive settles that CHANGES.md is retired; it does
not settle HOW, and the two answers are materially different operations:

  * if an entry's content is genuinely present in its commit message and its owning task
    leaf, the entry is a DUPLICATE and is removed with a link;
  * if it is not, the entry is UNIQUE CONTENT, and deleting it destroys information. The
    doctrine's answer there is an archive_terminal -- the bytes leave the bounded working
    set and stay retrievable -- never a deletion justified by "git has it somewhere".

That second case is why this probe exists rather than a count of `Landed as:` hashes. A
hash proves an entry is ADDRESSABLE. It says nothing about whether the prose beside it
survives anywhere else, and addressability has been mistaken for recoverability before.

METHOD. For each entry: normalize to a word multiset, then subtract every word available
in its canonical sources (the commit message at its recorded hash, plus the full text of
the task file its leaf id names). What remains is content that exists ONLY in CHANGES.md.
Word-level rather than line-level because the sources legitimately rephrase -- a commit
subject and a CHANGES heading say the same thing in different shapes -- so a line diff
would report a total loss on entries that are in fact well covered.

Deterministic: no clock, no randomness, sorted output. Reads the repository and git;
mutates nothing (DOCTRINE_ENFORCEMENT.md §4).
"""

from __future__ import annotations

import argparse
import json
import os
import re
import subprocess
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

ENTRY_RE = re.compile(r"^## (.+)$", re.M)
HASH_RE = re.compile(r"\*\*Landed as:\*\*\s*`([0-9a-f]{6,40})`")
LEAF_RE = re.compile(r"\b([A-Z][A-Z0-9-]{3,})(\.[0-9a-z]+)*\b")
WORD_RE = re.compile(r"[a-z0-9_]+")

# Words that carry no information about WHAT changed: markdown furniture, and the
# boilerplate every entry repeats. Counting these as "unique content" would inflate the
# residue with the very scaffolding the retirement removes.
STOP = frozenset("""a an and are as at be been but by for from has have if in into is it its of on or
that the their there this to was were what when where which who why with landed as previous commit
files touched impact why validation what md rs sh""".split())


def sh(*args: str) -> str:
    return subprocess.run(args, cwd=ROOT, check=True, capture_output=True, text=True).stdout


def words(text: str) -> set[str]:
    return {w for w in WORD_RE.findall(text.lower()) if len(w) > 2 and w not in STOP}


def commit_messages() -> dict[str, str]:
    """hash -> full commit message, for every commit reachable from HEAD."""
    out = sh("git", "log", "--format=%H%x00%B%x01")
    m = {}
    for rec in out.split("\x01"):
        rec = rec.strip("\n")
        if not rec or "\x00" not in rec:
            continue
        h, body = rec.split("\x00", 1)
        m[h] = body
    return m


def durable_layers() -> dict[str, set[str]]:
    """Every layer that could legitimately hold an entry's content.

    ADDED AT .8a2, AND ITS ABSENCE IS WHY .8a's HEADLINE WAS WRONG. The first pass scored
    each entry against only its OWN commit message and its OWN task file. Where neither
    resolved, residue was 100 % BY CONSTRUCTION -- and 162 entries with date-slug headings
    resolved to neither, contributing nearly half the total. That measured the matcher, not
    the content. The transferable rule: a coverage metric whose denominator is "sources I
    managed to locate" reports the locator's competence and calls it a property of the
    subject.
    """
    layers = {}
    layers["commits"] = words("\n".join(commit_messages().values()))
    for name, path in (("devnotes", "DEVELOPMENT_NOTES.md"),):
        p = os.path.join(ROOT, path)
        layers[name] = words(open(p, encoding="utf-8").read()) if os.path.exists(p) else set()
    for name, d in (("book", "book/src"), ("adrs", "docs/decisions"), ("km", "docs/knowledge"),
                    ("tasks", "docs/tasks")):
        acc: set[str] = set()
        full = os.path.join(ROOT, d)
        if os.path.isdir(full):
            for f in sorted(os.listdir(full)):
                if f.endswith(".md"):
                    acc |= words(open(os.path.join(full, f), encoding="utf-8").read())
        layers[name] = acc
    return layers


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--json", action="store_true")
    ap.add_argument("--worst", type=int, default=10, help="show N least-recoverable entries")
    ap.add_argument("--layers", action="store_true",
                    help="score the pre-task-tree era against EVERY durable layer, cumulatively")
    args = ap.parse_args()

    if args.layers:
        text = open(os.path.join(ROOT, "CHANGES.md"), encoding="utf-8").read()
        heads = list(ENTRY_RE.finditer(text))
        date_re = re.compile(r"(20\d\d-\d\d-\d\d)")
        # 2026-05-17 is the task-tree ownership doctrine. Before it, no task file existed to
        # hold an entry's reasoning; after it, one exists by construction. The boundary is a
        # recorded doctrine date, not a percentile chosen to make the answer come out well.
        pre = set()
        for i, m in enumerate(heads):
            d = date_re.search(m.group(1))
            if d and d.group(1) < "2026-05-17":
                pre |= words(text[m.start(): heads[i + 1].start() if i + 1 < len(heads) else len(text)])
        L = durable_layers()
        print(f"pre-task-tree (< 2026-05-17) distinct content words: {len(pre)}")
        cov: set[str] = set()
        for name in ("commits", "devnotes", "book", "adrs", "km", "tasks"):
            cov |= L[name]
            r = pre - cov
            print(f"  + {name:<10} uncovered {len(r):>5}  ({100*len(r)/len(pre):>5.1f} %)")
        rest = pre - cov
        alpha = sorted(x for x in rest if re.fullmatch(r"[a-z]{4,}", x))
        numeric = [x for x in rest if re.search(r"\d", x)]
        print(f"\nof the {len(rest)} words in NO durable layer: {len(numeric)} numeric/identifier, "
              f"{len(alpha)} prose (4+ alpha)")
        print("prose words:", ", ".join(alpha))
        return 0

    text = open(os.path.join(ROOT, "CHANGES.md"), encoding="utf-8").read()
    heads = list(ENTRY_RE.finditer(text))
    msgs = commit_messages()
    by_prefix = {}
    for h in msgs:
        for n in range(7, 13):
            by_prefix.setdefault(h[:n], h)

    task_cache: dict[str, str] = {}

    def task_text(leaf: str) -> str:
        tree = leaf.split(".")[0]
        if tree not in task_cache:
            p = os.path.join(ROOT, "docs", "tasks", f"{tree}.md")
            task_cache[tree] = open(p, encoding="utf-8").read() if os.path.exists(p) else ""
        return task_cache[tree]

    entries = []
    for i, m in enumerate(heads):
        body = text[m.start(): heads[i + 1].start() if i + 1 < len(heads) else len(text)]
        heading = m.group(1)
        hm = HASH_RE.search(body)
        lm = LEAF_RE.search(heading)
        leaf = lm.group(0) if lm else None
        src = ""
        resolved = False
        if hm:
            full = by_prefix.get(hm.group(1)[:12]) or by_prefix.get(hm.group(1)[:7])
            if full:
                src += msgs[full]
                resolved = True
        if leaf:
            src += "\n" + task_text(leaf)
        ew = words(body)
        residue = ew - words(src)
        entries.append(
            {
                "heading": heading[:90],
                "bytes": len(body.encode()),
                "leaf": leaf,
                "hash_resolved": resolved,
                "words": len(ew),
                "residue_words": len(residue),
                "recovered_pct": round(100 * (1 - len(residue) / len(ew)), 1) if ew else 100.0,
            }
        )

    total = len(entries)
    with_hash = sum(1 for e in entries if e["hash_resolved"])
    with_leaf = sum(1 for e in entries if e["leaf"])
    fully = sum(1 for e in entries if e["residue_words"] == 0)
    tot_w = sum(e["words"] for e in entries)
    tot_r = sum(e["residue_words"] for e in entries)

    if args.json:
        json.dump({"total": total, "entries": entries}, sys.stdout, indent=2)
        print()
        return 0

    print(f"CHANGES.md entries                         : {total}")
    print(f"  with a `Landed as:` hash resolvable in git: {with_hash}  ({100*with_hash/total:.1f} %)")
    print(f"  naming a task-tree leaf in the heading    : {with_leaf}  ({100*with_leaf/total:.1f} %)")
    print(f"  FULLY recoverable (zero residue words)    : {fully}  ({100*fully/total:.1f} %)")
    print()
    print(f"content words across all entries           : {tot_w}")
    print(f"  present ONLY in CHANGES.md (residue)     : {tot_r}  ({100*tot_r/tot_w:.1f} %)")
    print(f"  recoverable from git log + task trees    : {tot_w-tot_r}  ({100*(tot_w-tot_r)/tot_w:.1f} %)")
    print()
    worst = sorted(entries, key=lambda e: e["recovered_pct"])[: args.worst]
    print(f"LEAST recoverable {args.worst} entries:")
    for e in worst:
        print(f"  {e['recovered_pct']:>5.1f}%  {e['residue_words']:>5}w  {e['heading'][:70]}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
