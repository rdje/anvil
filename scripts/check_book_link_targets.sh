#!/usr/bin/env bash
#
# scripts/check_book_link_targets.sh — the BOOK-LINK-TARGETS doctrine.
#
# DOCTRINE_ENFORCEMENT.md §4 (contract) / §10 (registry). Registered by
# BOOK-LINK-INTEGRITY.3 on the defect .1 measured and .2 decided the form for.
#
# WHAT IT CATCHES. mdBook rewrites every `.md` link target to `.html` when it
# renders. Inside the book that is correct and necessary — `knobs.md#anchor`
# becomes `knobs.html#anchor`. It is a trap the moment the target leaves
# book/src: the natural, correct-looking
#
#     [USER_GUIDE.md](../../USER_GUIDE.md#tracing-and-debugging)
#
# renders as `../../USER_GUIDE.html`, which DOES NOT EXIST. It resolves when the
# Markdown source is read on GitHub and is DEAD in the rendered book — which is
# the surface the owner reviews ("I review the book, not the code"). `mdbook
# build` exits 0 on it.
#
# WHY THE OBVIOUS CHECK WOULD BE VACUOUS HERE. The portable check anyone would
# reach for is "does the link target exist?". Point it at this repo and it
# reports clean: ../../USER_GUIDE.md EXISTS. The defect is not a missing file,
# it is a rendered target that ESCAPES the build directory. A check that models
# only existence passes the one link this doctrine was written for. So the
# ESCAPE test is the load-bearing half, and it is tested first.
#
# THE TWO RULES, both exact and both source-level — no mdbook required:
#   1. ESCAPE  — a link target in book/src must not resolve outside book/src.
#   2. MISSING — a link target inside book/src must exist.
# Decision 0046 records rule 1 as a convention with a measured basis: 31 of the
# 32 book -> repo-root references already named the file in backticks instead of
# linking to it. This gate holds the convention that was already unanimous.
#
# WHAT IT DELIBERATELY DOES NOT CHECK, stated rather than implied (§9). It does
# not resolve #fragments to headings. Doing that at source level means
# reimplementing mdBook's heading slugifier, and a wrong slug model cries wolf —
# "a gate that cries wolf gets deleted, taking its real coverage with it". The
# anchor class was MEASURED at .1 and is empty (0 dead of 71 authored / 1136
# rendered, negative-controlled), so this is an argument about present risk, not
# permanent immunity. If anchors ever rot, that is a new leaf, not a silent gap.
#
# WHY A DOCTRINE RATHER THAN AN ASSERTION INSIDE AN EXISTING ONE. The
# feedback_full_factorization rule forbids a SECOND mechanism for a job that
# already has one. TABLE-RENDER-FIDELITY owns markdown table well-formedness;
# ENUMERATION-PARITY owns declared list pairs (its book pair holds SUMMARY.md
# against the chapter files — which chapters are LINKED, not whether a link
# RESOLVES); MEMORY-ARCH and README-GROWTH own two named files' size; the rest
# own staging, paths and citations. No registered doctrine owns link targets.
# The subject is unowned, so this registers the first mechanism, not a second.
#
# NOT SCOPE-AWARE, DELIBERATELY. Link integrity is a property of the TREE, not
# of a change — the same reasoning README-GROWTH uses. A dead link can arrive by
# revert, by merge, or by a rename in a commit that never touches the linking
# file; a staged-set-only gate would never re-examine it.
#
# COUNT-FLOORED, because an extractor that silently matches nothing makes the
# gate pass vacuously — the exact failure this doctrine exists to remove. The
# floor is DERIVED, not fitted: book/src/SUMMARY.md must link every chapter for
# mdBook to render it (a pair ENUMERATION-PARITY already holds), so the book
# structurally cannot contain fewer than (chapters - 1) links. It self-adjusts
# as chapters are added, and today's real count clears it by ~8x.
#
# FENCE-AWARE. A link drawn inside a ``` fence is an example, not a link, and
# the book documents its own defects. Measured at registration: zero such links
# exist in book/src today, so this is precautionary — and it is negative-
# controlled anyway (a fenced dead link must NOT fire while the same line
# unfenced MUST), because precautionary code that is never exercised is how a
# gate silently degrades.
#
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT}"

note() { printf '[book-links] %s\n' "$1" >&2; }
ok()   { printf '[book-links] ok: %s\n' "$1" >&2; }

# Tracked files only: untracked scratch chapters are not part of the rendered
# book, and a gate must run after `git add` to see newly staged content.
mapfile -t FILES < <(git ls-files 'book/src/*.md')

if [ "${#FILES[@]}" -eq 0 ]; then
  note "FAIL: no tracked book/src/*.md found — the check would pass vacuously."
  exit 1
fi

report="$(perl -CSD - "${#FILES[@]}" "${FILES[@]}" <<'PERL'
use strict; use warnings;

my $n_files = shift @ARGV;

# Resolve a repo-relative path containing . and .. without touching the disk,
# so the verdict is a fact about the TEXT and cannot depend on what happens to
# exist outside the repository.
sub norm {
  my @out;
  for my $seg (split m{/}, shift) {
    next if $seg eq '' || $seg eq '.';
    if ($seg eq '..') { if (@out && $out[-1] ne '..') { pop @out } else { push @out, '..' } }
    else { push @out, $seg }
  }
  return join '/', @out;
}

my ($links, $local, $bad) = (0, 0, 0);

for my $f (@ARGV) {
  open my $fh, '<:encoding(UTF-8)', $f or next;
  my @L = <$fh>; close $fh; chomp @L;

  # A link inside a fenced code block is an example, not a link.
  my @fence = (0) x scalar(@L); my $in = 0;
  for my $i (0 .. $#L) {
    if ($L[$i] =~ /^\s*(```|~~~)/) { $in = !$in; $fence[$i] = 1; next }
    $fence[$i] = $in;
  }

  (my $dir = $f) =~ s{/[^/]*$}{};

  # Scan the file as ONE string, not line by line. A markdown link's TEXT may
  # wrap across a newline — measured at registration: 2 such links exist in this
  # book (api-tools.md, faq.md) — and a line-wise scan silently skips them. That
  # is the extractor-charset-narrower-than-source defect: the miss is invisible,
  # and a wrapped link that escapes book/src would sail straight through.
  my $text = join "\n", @L;
  my @line_of;                        # char offset -> 0-based line index
  { my $ln = 0;
    for my $i (0 .. length($text) - 1) {
      $line_of[$i] = $ln;
      $ln++ if substr($text, $i, 1) eq "\n";
    } }

  my @targets;                        # [target, 0-based line]
  # inline links [text](target), excluding images ![alt](src)
  while ($text =~ /(?<!!)\[(?:[^\]\\]|\\.)*\]\(\s*<?([^)\s>]+)>?(?:\s+"[^"]*")?\s*\)/g) {
    my $t = $1;
    my $start = $-[0];
    push @targets, [$t, ($line_of[$start] // 0)];
  }
  # reference definitions [id]: target — inherently single-line
  for my $i (0 .. $#L) {
    push @targets, [$1, $i] if $L[$i] =~ /^\s{0,3}\[[^\]]+\]:\s*<?(\S+)>?\s*$/;
  }

  {
    for my $rec (@targets) {
      my ($t, $i) = @$rec;
      next if $fence[$i];             # a link inside a fence is an example
      $links++;
      next if $t =~ m{^(?:https?|mailto|javascript|data|ftp):}i || $t =~ m{^//};
      next if $t =~ /^#/;                       # in-page anchor: not a file target
      $local++;
      (my $path = $t) =~ s/#.*$//;              # the fragment is out of scope (see header)
      next if $path eq '';
      my $resolved = norm("$dir/$path");
      if ($resolved !~ m{^book/src/} ) {
        $bad++;
        printf "BAD\tESCAPE\t%s\t%d\t%s\t%s\n", $f, $i + 1, $t, ($resolved eq '' ? '<repo root>' : $resolved);
      } elsif (! -e $resolved) {
        $bad++;
        printf "BAD\tMISSING\t%s\t%d\t%s\t%s\n", $f, $i + 1, $t, $resolved;
      }
    }
  }
}
printf "SUM\t%d\t%d\t%d\t%d\n", $n_files, $links, $local, $bad;
PERL
)"

summary="$(printf '%s\n' "${report}" | awk -F'\t' '$1=="SUM"{print $2, $3, $4, $5}')"
read -r n_files n_links n_local n_bad <<<"${summary}"

if [ -z "${n_bad:-}" ]; then
  note "FAIL: the scanner produced no summary line — treat as a broken gate, not a pass."
  exit 1
fi

# Derived floor: SUMMARY.md must link every chapter for mdBook to render it, so
# the book cannot hold fewer than (chapters - 1) links. Fewer means the
# extractor broke, not that the book shrank.
floor=$(( n_files - 1 ))
if [ "${n_links}" -lt "${floor}" ]; then
  note "FAIL: extracted only ${n_links} link(s) from ${n_files} chapter(s); at least"
  note "  ${floor} are structurally required (SUMMARY.md links every chapter)."
  note "  This is a BROKEN EXTRACTOR, not a clean book — failing rather than passing"
  note "  vacuously. Fix the scanner before trusting any verdict from it."
  exit 1
fi

if [ "${n_bad}" -ne 0 ]; then
  printf '%s\n' "${report}" | awk -F'\t' '
    $1=="BAD" && $2=="ESCAPE" {
      printf "[book-links] FAIL: %s:%s links to `%s`, which resolves to %s — OUTSIDE book/src.\n", $3, $4, $5, $6 > "/dev/stderr" }
    $1=="BAD" && $2=="MISSING" {
      printf "[book-links] FAIL: %s:%s links to `%s` — %s does not exist.\n", $3, $4, $5, $6 > "/dev/stderr" }'
  note ""
  note "  ESCAPE: mdBook rewrites a .md target to .html, so a link out of book/src"
  note "  renders as a path that does not exist. It works on GitHub and is DEAD in"
  note "  the rendered book — and `mdbook build` exits 0 on it."
  note ""
  note "  Fix (decision 0046): NAME the file in backticks instead of linking to it,"
  note "  with any section named in parentheses as prose —"
  note "      See \`USER_GUIDE.md\` (\"Tracing and debugging\") for the level table."
  note "  Do NOT substitute an absolute github.com URL: book/book.toml sets"
  note "  git-repository-url = \"\", i.e. the book is built host-agnostic on purpose."
  note "  Do NOT copy the target into the book: decision 0033 (shadow enumeration)."
  note ""
  note "  MISSING: an in-book link whose chapter is gone or renamed. Repoint it."
  note ""
  note "  Scanned ${n_links} link(s) (${n_local} local) across ${n_files} tracked chapters."
  note "BOOK-LINK-TARGETS breached — commit blocked. See"
  note "docs/tasks/BOOK-LINK-INTEGRITY.md, docs/decisions/0046-book-never-links-outside-book-src.md"
  note "and docs/knowledge/mdbook-md-to-html-rewrite-trap.md."
  exit 1
fi

ok "${n_local} local link target(s) of ${n_links} across ${n_files} tracked book/src chapters resolve inside book/src"
exit 0
