#!/usr/bin/env bash
# negative_control.sh — the mutation half of a negative control, and the only part
# of one that fails silently.
#
# WHY THIS EXISTS (decision 0047, from NEGATIVE-CONTROL-HARNESS.1's measurement):
# a negative control needs three legs — the check CAN fire, it fires on the RIGHT
# INPUT (the DOCTRINE_ENFORCEMENT.md §9 vacuity probe), and THE EXPERIMENT RAN AT
# ALL. Only the third fails silently, and it does so in one primitive: `sed -i`
# and `perl -pi -e` exit 0 whether or not the pattern matched, so a mistyped
# substitution leaves the tree unchanged, the check passes, and that reads exactly
# like a control that correctly did not fire. Measured over 39 recorded control
# episodes in docs/tasks/*.md, leg 3 was reported in 2.
#
# So this tool refuses to no-op. `apply` exits nonzero when the substitution count
# is zero, BEFORE any verdict can be read, and it distinguishes that from a broken
# expression rather than reporting one status for both.
#
# PREFER NOT TO NEED IT (decision 0047, rung R1). A mutation with no *match* step
# cannot fail this way, so where a control can be written as one, write it there
# and this tool is unnecessary:
#   - in-language, inside a #[test]   (src/ir/case_qualifier.rs breaks its fixture
#     with a Rust statement, after asserting the fixture is clean beforehand)
#   - compiler-checked                 (the landing IS the E0624 / E0616 diagnostic)
#   - a constructed fixture            (README-POLICY-ADOPTION.3's isolated root)
#   - a `git show HEAD:` baseline      (LIVE-DOC-REGISTRY-SHADOWS.1)
#   - a mutation-free test seam        (DOCTRINE_STAGED_OVERRIDE), admissible only
#     where the seam does not bypass the extraction under test.
# This tool is rung R2: for when the subject genuinely IS file text.
#
# CONTRACT NOTE. This is an INSTRUMENT, not a doctrine check, and it is
# deliberately absent from scripts/check_doctrines.sh — per decision 0047 (C) a
# control's mutation is reverted before the commit by construction, so there is
# nothing for a gate to read. It therefore meets the DOCTRINE_ENFORCEMENT.md §4
# contract on determinism, path-agnosticism and exit-code-as-verdict, and
# knowingly departs from §4(4) "mutates nothing": mutating is the job. The
# mutation is bounded — every `apply` writes a backup first and every `restore`
# is verified with `cmp`.
#
# USAGE
#   negative_control.sh apply   <file> <perl-substitution>
#   negative_control.sh restore <file>
#   negative_control.sh probe   <file> <perl-substitution> <fires|silent> <cmd...>
#   negative_control.sh --self-test
#
# EXIT STATUS
#   0  the requested operation succeeded (for `probe`: the observed verdict
#      matched the declared expectation)
#   1  usage / environment error, or a `probe` whose verdict contradicted its
#      expectation, or a restore that did not reproduce the backup
#   2  the substitution expression itself is broken (a perl compile error)
#   9  THE ONE THIS TOOL EXISTS FOR: the substitution matched nothing, so the
#      experiment never ran. Not a finding about the check.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
# On-volume, repo-root-derived, never the OS temp dir (decision 0031).
BACKUP_DIR="${NEGATIVE_CONTROL_BACKUP_DIR:-$ROOT/.cache/negative-control}"

say()  { printf 'negative-control: %s\n' "$1"; }
warn() { printf 'negative-control: %s\n' "$1" >&2; }
die()  { warn "$1"; exit "${2:-1}"; }

backup_path_for() {
  # One backup slot per absolute source path, flattened so nested files cannot
  # collide (a/b.md and a-b.md would otherwise share a slot).
  local abs; abs="$(cd "$(dirname "$1")" && pwd)/$(basename "$1")"
  printf '%s/%s.orig' "$BACKUP_DIR" "$(printf '%s' "${abs#/}" | tr '/' '%')"
}

# The whole point of the tool. Applies EXPR to FILE and refuses to report success
# when nothing changed. Slurp mode (-0777) so a multi-line pattern works; add /m
# to the caller's expression when ^ and $ should be per-line.
#
# NEGATIVE_CONTROL_SKIP_COUNT_ASSERT is a seam used ONLY by --self-test, to prove
# the assertion below is load-bearing. It is deliberately a seam and not a source
# edit: a control on THIS tool must not use the primitive this tool exists to
# police, or it inherits the very defect (decision 0047).
apply_mutation() {
  local file="$1" expr="$2" backup tmp rc
  [ -f "$file" ] || die "not a file: $file"
  mkdir -p "$BACKUP_DIR"
  backup="$(backup_path_for "$file")"
  [ -e "$backup" ] && die "a backup already exists for $file — restore it first: $backup"
  cp "$file" "$backup"

  tmp="$backup.new"
  rc=0
  NC_EXPR="$expr" perl -0777 -pe '
      BEGIN { $n = 0; $bad = 0 }
      $n += eval($ENV{NC_EXPR});
      if ($@) { $bad = 1; warn $@ }
      END { exit($bad ? 2 : ($n > 0 ? 0 : 9)) }
    ' "$file" > "$tmp" || rc=$?

  if [ "$rc" -eq 2 ]; then
    rm -f "$tmp" "$backup"
    die "the substitution expression is broken (perl could not compile it): $expr" 2
  fi
  if [ "$rc" -eq 9 ] && [ -z "${NEGATIVE_CONTROL_SKIP_COUNT_ASSERT:-}" ]; then
    rm -f "$tmp" "$backup"
    die "SUBSTITUTION MATCHED NOTHING in $file — the experiment did not run, so no
  verdict from it means anything. This is NOT a finding about the check. Fix the
  expression (escaping is the usual culprit) and re-run:
    $expr" 9
  fi
  if [ "$rc" -ne 0 ] && [ "$rc" -ne 9 ]; then
    rm -f "$tmp" "$backup"
    die "perl failed with status $rc" 1
  fi

  mv "$tmp" "$file"
  if [ "$rc" -eq 9 ]; then
    say "applied 0 substitutions to $file (count assertion SKIPPED — self-test only)"
  else
    say "applied to $file; backup at ${backup#"$ROOT"/}"
  fi
}

# The half this repo already does by hand 27 times over: prove the revert landed.
restore_mutation() {
  local file="$1" backup
  backup="$(backup_path_for "$file")"
  [ -e "$backup" ] || die "no backup recorded for $file"
  cp "$backup" "$file"
  cmp -s "$file" "$backup" || die "RESTORE DID NOT REPRODUCE THE BACKUP for $file" 1
  rm -f "$backup"
  say "restored $file, verified byte-identical to the backup"
}

# apply -> run the check -> compare against the DECLARED expectation -> restore.
# Declaring the expectation up front is what makes a `silent` probe meaningful:
# without it, "the check passed" is the same sentence whether the mutation landed
# or not.
run_probe() {
  local file="$1" expr="$2" want="$3"; shift 3
  case "$want" in
    fires|silent) ;;
    *) die "expectation must be 'fires' or 'silent', got: $want" ;;
  esac
  [ "$#" -ge 1 ] || die "probe needs a command to run"

  apply_mutation "$file" "$expr"
  local rc=0
  "$@" >/dev/null 2>&1 || rc=$?
  restore_mutation "$file"

  local observed="silent"
  [ "$rc" -ne 0 ] && observed="fires"
  if [ "$observed" = "$want" ]; then
    say "PROBE OK: expected $want, observed $observed (check exit $rc)"
    return 0
  fi
  die "PROBE FAILED: expected $want, observed $observed (check exit $rc)" 1
}

# The control on the control. Every case here is mutation-free in the sense that
# matters: the subject files are CONSTRUCTED by this function, so nothing can
# silently fail to match while setting up the test itself.
self_test() {
  local dir status=0 f
  dir="$(mktemp -d "$BACKUP_DIR.selftest.XXXXXX")"
  trap 'rm -rf "$dir"' RETURN
  f="$dir/subject.txt"

  check() { # check <label> <expected-status> <actual-status>
    if [ "$2" = "$3" ]; then printf '  ok   %s (exit %s)\n' "$1" "$3"
    else printf '  FAIL %s: expected exit %s, got %s\n' "$1" "$2" "$3"; status=1; fi
  }

  local rc

  # 1. A matching substitution applies and reports success.
  printf 'alpha\nbeta\n' > "$f"
  rc=0; ( apply_mutation "$f" 's/alpha/ALPHA/g' ) >/dev/null || rc=$?
  check "a matching substitution applies" 0 "$rc"
  grep -q ALPHA "$f" || { printf '  FAIL the file was not actually changed\n'; status=1; }

  # 2. Restore reproduces the original byte-for-byte.
  rc=0; ( restore_mutation "$f" ) >/dev/null || rc=$?
  check "restore succeeds" 0 "$rc"
  [ "$(cat "$f")" = "$(printf 'alpha\nbeta')" ] || { printf '  FAIL restore did not reproduce the original\n'; status=1; }

  # 3. THE SUBJECT: a non-matching substitution must FAIL, not pass quietly.
  rc=0; ( apply_mutation "$f" 's/NOMATCH/x/g' ) >/dev/null 2>&1 || rc=$?
  check "a non-matching substitution is rejected" 9 "$rc"

  # 4. A broken expression is distinguishable from a non-matching one — two
  #    different failures must not share an exit code, which is this tool's own
  #    subject turned on itself.
  rc=0; ( apply_mutation "$f" 's/unterminated' ) >/dev/null 2>&1 || rc=$?
  check "a broken expression is its own status" 2 "$rc"

  # 5. CONTROL ON THE CONTROL: with the count assertion disabled through the
  #    seam, case 3 must SUCCEED. If this passes while case 3 also passes, the
  #    assertion is not what is doing the work.
  rc=0; ( NEGATIVE_CONTROL_SKIP_COUNT_ASSERT=1 apply_mutation "$f" 's/NOMATCH/x/g' ) >/dev/null 2>&1 || rc=$?
  check "without the assertion the same no-op SUCCEEDS (so the assertion is load-bearing)" 0 "$rc"
  rc=0; ( restore_mutation "$f" ) >/dev/null 2>&1 || rc=$?
  check "cleanup restore after the seam case" 0 "$rc"

  # 6. probe honours its declared expectation in both directions, against a
  #    check whose verdict is a real function of the file's content.
  printf 'alpha\n' > "$f"
  rc=0; ( run_probe "$f" 's/alpha/BROKEN/g' fires grep -q '^alpha$' "$f" ) >/dev/null 2>&1 || rc=$?
  check "probe: mutation makes the check fire, as declared" 0 "$rc"
  rc=0; ( run_probe "$f" 's/alpha/ALPHA/g' silent grep -q '.' "$f" ) >/dev/null 2>&1 || rc=$?
  check "probe: mutation leaves the check silent, as declared" 0 "$rc"
  rc=0; ( run_probe "$f" 's/alpha/BROKEN/g' silent grep -q '^alpha$' "$f" ) >/dev/null 2>&1 || rc=$?
  check "probe: a contradicted expectation FAILS" 1 "$rc"

  # 7. A probe whose mutation never lands must NOT be reported as a verdict.
  rc=0; ( run_probe "$f" 's/NOMATCH/x/g' silent true ) >/dev/null 2>&1 || rc=$?
  check "probe: a no-op mutation is rejected before any verdict is read" 9 "$rc"

  if [ "$status" -eq 0 ]; then say "--self-test: all cases pass"; else warn "--self-test: FAILURES above"; fi
  return "$status"
}

case "${1:---help}" in
  apply)      shift; [ "$#" -eq 2 ] || die "usage: apply <file> <perl-substitution>"; apply_mutation "$1" "$2" ;;
  restore)    shift; [ "$#" -eq 1 ] || die "usage: restore <file>"; restore_mutation "$1" ;;
  probe)      shift; [ "$#" -ge 4 ] || die "usage: probe <file> <perl-substitution> <fires|silent> <cmd...>"; run_probe "$@" ;;
  --self-test) self_test ;;
  --help|-h)  sed -n '/^# USAGE/,/^# 9 /p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//' ;;
  *)          die "unknown command: $1 (try --help)" ;;
esac
