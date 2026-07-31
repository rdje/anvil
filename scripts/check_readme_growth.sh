#!/usr/bin/env bash
#
# scripts/check_readme_growth.sh — the README-GROWTH doctrine.
#
# DOCTRINE_ENFORCEMENT.md §4 (contract) / §10 (registry). Mechanizes the owner
# owner directive of 2026-07-30 ("make sure README.md doesn't grow, grow and grow"),
# recorded in the project-owned README_POLICY.md. Cited by owner+date rather than by
# a harness bootstrap file: ANVIL ships five of those (CLAUDE.md / AGENTS.md /
# GEMINI.md / .cursorrules / .windsurfrules) and this doctrine binds every author
# equally, being enforced at git level. README-POLICY-PROVENANCE.1, 2026-07-31.
# and the "Mechanical growth guard" clause of the project-owned README_POLICY.md,
# as designed at decision 0036 (README-POLICY-ADOPTION).
#
# WHY A CAP RATHER THAN A STYLE RULE. The growth this stops was STRUCTURAL, not
# accidental: before decision 0036 the workflow *asked* every new knob for a
# README bullet, so the file reached 1771 lines / 122,767 bytes — 92 % of it a
# lossy copy of docs/decisions/, book/src/, USER_GUIDE.md and ROADMAP.md. A
# review habit cannot hold that back; a number that fails the commit can.
#
# WHY BOTH CAPS. They are not belt-and-braces — each catches a different bloat,
# and measurement at README-POLICY-ADOPTION.2 proved it: the surviving landing
# page was 10,297 bytes across 141 lines, so the *projected* file sat at 74 % of
# the line cap while already EXCEEDING the byte cap. Prose density is the hidden
# variable (118 bytes/line for a numbered reading list vs 57 for a path list).
# Neither wrapped prose nor very long lines can bypass the budget.
#
# THE CAPS ARE DERIVED, NOT CHOSEN TO FIT. Decision 0036 §(c) set them from what
# SURVIVED the audit (141 lines + a trimmed quick start + navigation), then
# rounded to modest headroom — deliberately BELOW README_POLICY.md's illustrative
# 300 / 16,384, which against a 141-line survivor would leave 60 % room to regrow
# into. README_POLICY.md cites THIS FILE as authoritative rather than restating
# the numbers, per decision 0033's rule that a number written beside the thing
# that defines it is one more copy of it.
#
# RAISING A CAP IS NOT A FIX. The policy is explicit: never raise a cap merely to
# land new content; route the detail to its canonical home. A cap increase
# requires an explicit reviewed decision that the landing-page contract itself
# expanded — i.e. a new record in docs/decisions/, not an edit to these two lines.
#
# Reads the repository, mutates nothing. Deterministic: no clocks, no network, no
# randomness. NOT scope-aware: the size of the landing page is a property of the
# tree, not of a change, so a commit that does not touch README.md is still
# checked — otherwise an over-cap README could be introduced by a revert or a
# merge and never re-examined. POSIX/bash-3.2 compatible.
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT}"

# --- the reviewed caps (decision 0036 §(c)) ---------------------------------
LINE_CAP=250
BYTE_CAP=12288

README="README.md"
POLICY="README_POLICY.md"

fail=0
note() { printf '[readme-growth] %s\n' "$1" >&2; }
ok() { printf '[readme-growth] ok: %s\n' "$1"; }

routing_hint() {
  note "  Route the overflow, do not raise the cap:"
  note "    user-facing flags/knobs/examples -> USER_GUIDE.md, book/src/"
  note "    current work, priorities, status  -> ROADMAP.md, docs/tasks/"
  note "    cross-tree status, one row/tree   -> docs/TASK_TREE.md (its ROW contract:"
  note "                                         one frontier, not a per-leaf journal —"
  note "                                         decision 0042)"
  note "    design rationale                  -> docs/decisions/ (add a record)"
  note "    banked gate evidence, saw_* facts -> docs/evidence/ (decision 0030)"
  note "    diagnostics and procedures        -> TOOLBOX.md"
  note "  See ${POLICY}. Raising a cap requires a new decision record stating"
  note "  that the landing-page contract itself expanded — not an edit here."
}

# The policy's own storage clause: the project-owned copy must exist beside the
# file it governs. Without it the caps are folklore, and the routing hint above
# points at a document that is not there.
if [ ! -f "${POLICY}" ]; then
  note "FAIL: ${POLICY} is missing."
  note "  The README Stability Policy requires the project-owned copy at the"
  note "  repository root, alongside README.md (an external copy may be a"
  note "  template but must not replace it). See decision 0036 §(e)."
  fail=1
else
  ok "${POLICY} present (the project-owned policy copy)"
fi

if [ ! -f "${README}" ]; then
  note "FAIL: ${README} is missing — the repository has no landing page."
  fail=1
else
  lines="$(wc -l < "${README}" | tr -d ' ')"
  bytes="$(wc -c < "${README}" | tr -d ' ')"

  if [ "${lines}" -gt "${LINE_CAP}" ] || [ "${bytes}" -gt "${BYTE_CAP}" ]; then
    note "FAIL: ${README} is ${lines} lines / ${bytes} bytes (caps: ${LINE_CAP} / ${BYTE_CAP})."
    routing_hint
    fail=1
  else
    ok "${README} is ${lines} lines / ${bytes} bytes (caps: ${LINE_CAP} / ${BYTE_CAP})"
  fi
fi

if [ "${fail}" -ne 0 ]; then
  note "README-GROWTH breached — commit blocked. See ${POLICY} and"
  note "docs/decisions/0036-readme-landing-page-restoration.md."
  exit 1
fi

exit 0
