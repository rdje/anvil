#!/usr/bin/env bash
#
# scripts/check_markdown_tables.sh — the TABLE-RENDER-FIDELITY doctrine.
#
# DOCTRINE_ENFORCEMENT.md §4 (contract) / §10 (registry). Registered by
# OVERFLOW-DESTINATION-INSTRUMENTATION.7 on the defect .6 measured.
#
# WHAT IT CATCHES, AND WHY IT IS NOT A STYLE RULE. In a GFM table the row is split
# on unescaped `|` BEFORE inline parsing — so a code span gives a pipe no
# protection — and the spec then says cells beyond the header's column count are
# IGNORED. A row that over-splits therefore loses the tail of its content from
# EVERY rendered page while the source still shows it. Nothing marks the gap: the
# text is visible to the author, absent for the reader, and no size, link or
# spelling check can see it.
#
# THIS IS NOT A HYPOTHETICAL. Measured tree-wide at .6 over 243 tracked *.md
# (394 tables, 2,310 data rows): 36 malformed rows silently dropped 57,283
# rendered characters, including 5 cells that were a row's own link to its detail
# file. The worst single case — CODEBASE_ANALYSIS.md's 24,990-byte line, cited by
# three leaves and by decision 0040 as a SIZE exhibit — was dropping 97.0 % of
# itself. It was not too long to read; it was never rendered.
#
# WHY A NINTH DOCTRINE RATHER THAN AN ASSERTION INSIDE AN EXISTING ONE. The
# feedback_full_factorization rule forbids a SECOND registered mechanism for a job
# that already has one — which is why .3 put MEMORY.md's byte cap inside
# check_memory_architecture.sh rather than registering a doctrine of its own:
# MEMORY-ARCH already owned that file's size. Here no registered doctrine owns
# markdown WELL-FORMEDNESS. MEMORY-ARCH and README-GROWTH own the SIZE of two
# named files; ENUMERATION-PARITY owns declared list pairs; the rest own staging,
# paths and citations. The subject is unowned, the check is deterministic and
# repo-wide, so it registers as its own doctrine.
#
# WHY IT IS ESCAPE-AWARE AND FENCE-AWARE, both learned the hard way at .6:
#   * A naive `|` count flagged 8 rows in docs/TASK_TREE.md where 5 were real —
#     three used the correct `\|` idiom. A checker that cannot tell the escaped
#     form from the bare one reports something plausible instead of failing.
#   * The fact card .6 wrote DEMONSTRATES a broken row inside a ```markdown fence.
#     A fence-blind scan would make the documentation of the defect the gate's
#     first false positive.
#
# WHY ONLY "MORE CELLS THAN THE HEADER" FAILS. A row with FEWER cells is padded
# with empty cells by the spec: it may look untidy, but nothing is lost. This gate
# is about CONTENT LOSS, not tidiness, so it stays silent on short rows — a gate
# that also cried wolf on those would be argued with instead of obeyed.
#
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT}"

note() { printf '[md-tables] %s\n' "$1" >&2; }
ok()   { printf '[md-tables] ok: %s\n' "$1" >&2; }

# Tracked *.md only: untracked scratch files are not part of the rendered docs,
# and a gate must run after `git add` to see newly staged content.
mapfile -t FILES < <(git ls-files '*.md')

if [ "${#FILES[@]}" -eq 0 ]; then
  note "FAIL: no tracked *.md found — the check would pass vacuously."
  exit 1
fi

report="$(perl -CSD - "${FILES[@]}" <<'PERL'
use strict; use warnings;

# Count the separators GFM would honour: a `|` preceded by an ODD number of
# backslashes is escaped and is content, not a cell boundary.
sub seps {
  my $s = shift; my $n = 0;
  for my $i (0 .. length($s) - 1) {
    next unless substr($s, $i, 1) eq '|';
    my $b = 0; my $j = $i - 1;
    $b++, $j-- while $j >= 0 && substr($s, $j, 1) eq "\\";
    $n++ unless $b % 2;
  }
  return $n;
}

my ($files, $tables, $rows, $bad) = (0, 0, 0, 0);
for my $f (@ARGV) {
  open my $fh, '<:encoding(UTF-8)', $f or next;
  my @L = <$fh>; close $fh; chomp @L; $files++;

  # Mark fenced code blocks; a table drawn inside one is an example, not a table.
  my @fence = (0) x scalar(@L); my $in = 0;
  for my $i (0 .. $#L) {
    if ($L[$i] =~ /^\s*(```|~~~)/) { $in = !$in; $fence[$i] = 1; next }
    $fence[$i] = $in;
  }

  for (my $i = 0; $i <= $#L; $i++) {
    next if $fence[$i];
    next unless $L[$i] =~ /^\s*\|/
             && defined $L[$i+1] && !$fence[$i+1]
             && $L[$i+1] =~ /^\s*\|[\s:-]*-{2,}/;
    my $hdr = seps($L[$i]); $tables++;
    my $k = $i + 2;
    while (defined $L[$k] && !$fence[$k] && $L[$k] =~ /^\s*\|/) {
      $rows++;
      my $n = seps($L[$k]);
      if ($n > $hdr) {
        $bad++;
        my @c = split /(?<!\\)\|/, $L[$k], -1; shift @c; pop @c;
        my $drop = 0;
        for my $q ($hdr - 1 .. $#c) { $drop += length($c[$q]) if $q >= 0 }
        printf "BAD\t%s\t%d\t%d\t%d\t%d\n", $f, $k + 1, $n - $hdr, $hdr - 1, $drop;
      }
      $k++;
    }
    $i = $k - 1;
  }
}
printf "SUM\t%d\t%d\t%d\t%d\n", $files, $tables, $rows, $bad;
PERL
)"

summary="$(printf '%s\n' "${report}" | awk -F'\t' '$1=="SUM"{print $2, $3, $4, $5}')"
read -r n_files n_tables n_rows n_bad <<<"${summary}"

if [ -z "${n_bad:-}" ]; then
  note "FAIL: the scanner produced no summary line — treat as a broken gate, not a pass."
  exit 1
fi

if [ "${n_bad}" -ne 0 ]; then
  printf '%s\n' "${report}" | awk -F'\t' '$1=="BAD"{
    printf "[md-tables] FAIL: %s:%s has %s cell(s) past the header'"'"'s %s column(s) — %s rendered characters are DROPPED.\n", $2, $3, $4, $5, $6 > "/dev/stderr" }'
  note ""
  note "  A GFM table row splits on every UNESCAPED | — including inside a code span —"
  note "  and the spec IGNORES cells past the header's column count. That content is in"
  note "  the source and absent from every rendered page."
  note ""
  note "  Fix: escape the pipe that belongs to the cell's text as \\| (one backslash)."
  note "  Do NOT widen the header to absorb the split, and do NOT delete the text."
  note "  A row with FEWER cells than the header is fine — the spec pads it, nothing is lost."
  note ""
  note "  Scanned ${n_files} tracked *.md, ${n_tables} tables, ${n_rows} data rows."
  note "TABLE-RENDER-FIDELITY breached — commit blocked. See"
  note "docs/tasks/OVERFLOW-DESTINATION-INSTRUMENTATION.md (.6 measured it, .7 gated it)"
  note "and docs/knowledge/gfm-table-unescaped-pipe-drops-content.md."
  exit 1
fi

ok "${n_rows} data rows across ${n_tables} tables in ${n_files} tracked *.md render every cell"
exit 0
