#!/usr/bin/env bash
#
# scripts/check_changes_entry_placement.sh — the CHANGES-ENTRY-PLACEMENT doctrine.
#
# DOCTRINE_ENFORCEMENT.md §3 (structural) / §4 (contract) / §10 (registry).
# Mechanizes decision 0045 (CHANGES-ENTRY-PLACEMENT.5), which answered the
# mechanism question on measurement rather than on intuition.
#
# THE PREDICATE, IN ONE SENTENCE:
#
#   If a commit's staged CHANGES.md diff adds at least one `## ` heading, the
#   FIRST `## ` heading in the resulting file must be one of those added lines.
#
# That is the whole rule. It needs NO date, NO commit hash, and no knowledge of
# the file's ordering convention — it reads the staged diff and the staged blob,
# both of which git already has.
#
# ── WHY THIS DESIGN AND NOT THE TWO OBVIOUS ONES ──────────────────────────────
# Both obvious candidates were measured and DISQUALIFIED before this one was
# written (decision 0038 at `.2`, re-measured at `.3`):
#
#   * a DATE-keyed ordering scan CRIES WOLF — 3 findings, 2 of them false, being
#     mis-dated headings sitting above correctly-ordered entries. A gate that
#     cries wolf gets deleted, taking its real coverage with it.
#   * a HASH-keyed ordering scan is VACUOUS for this exact defect — 0 of 3 real
#     offenders are visible to it, because the offending entries carry no
#     resolvable `Landed as:` hash.
#
# The hash failure is the deep one, and decision 0045 states the transferable
# rule it produced: **a detector must not depend on a field that the defect it
# detects also destroys.** The stale entry template that misplaces an entry is
# the SAME one that omits its provenance line, so any check needing that line is
# guaranteed to miss precisely the population it was built to find.
#
# ── WHAT IT MEASURED, BEFORE IT WAS REGISTERED ────────────────────────────────
# Run over every commit in history that touches CHANGES.md — 766 commits at
# 928817f: 664 ok, 99 correctly SKIPPED (they add no entry: hash backfills and
# typo fixes, which are the whole false-alarm risk), and 3 FIRES, ALL THREE TRUE
# POSITIVES, ZERO FALSE ALARMS.
#
# It also found an offender nobody knew about: f9cf50a (RESOURCE-SAFE-TOOLING.2,
# 2026-06-14) placed its entry at heading 6 of 379 — five below the top. `.1`,
# `.2` and `.3` of the owning tree all reported the class as TWO members; it is
# THREE. The mild case is exactly the one a human reviewer never notices, which
# is the argument for a gate rather than for diligence.
#
# ── WHY IT IS NOT SCOPE-AWARE, DELIBERATELY ───────────────────────────────────
# DOCTRINE_ENFORCEMENT.md §4(5) asks a check about a CHANGE to exempt what it
# does not govern, and the code-scoped checks (CODE-CHANGE-EVIDENCE,
# TASK-TREE-OWNERSHIP) do exactly that. This one must NOT: both original
# offenders were DOCS-ONLY commits, and that exemption is precisely what let them
# through. Entry placement has nothing to do with whether a commit touches code.
#
# The exemption this check DOES have is a different one, and it is intrinsic
# rather than declared: a commit that adds no `## ` heading to CHANGES.md — a
# provenance backfill, a typo fix, or a commit that does not stage the file at
# all — is silent, because the predicate has no subject.
#
# ── WHAT IT DOES NOT DO ───────────────────────────────────────────────────────
#   * It does NOT check ordering. It checks that a NEWLY ADDED entry is at the
#     top. An edit to a landed entry adds no heading and is correctly silent.
#   * It does NOT require a `Landed as:` hash, and must never be extended to.
#     The newest entry STRUCTURALLY cannot carry its own hash — the commit does
#     not exist while its message is being written — so such a check would be
#     permanently red by at least one entry.
#   * It does NOT authorise moving or rewriting any LANDED entry. Decision 0038
#     stands: position is itself a record, and the repair for a PAST
#     misplacement is an additive pointer stub, never a relocation. What this
#     check asks you to move is the entry you have not committed yet.
#
# ── KNOWN INTERACTION: the 0038 pointer stub ──────────────────────────────────
# A 0038 repair inserts a dated pointer stub MID-FILE, and a stub carries a `## `
# heading of its own. Such a commit passes here only because it also adds its own
# entry at the top — which COMMIT.md §2 mandates on every commit anyway, so the
# combination is the normal case and not a special one. Measured: 80edd42 (the
# leaf that inserted the two stubs) added three headings and is not among the 3
# historical fires.
#
# ── HONEST LIMIT (DOCTRINE_ENFORCEMENT.md §9) ─────────────────────────────────
# The check governs entries written in the `## ` convention. An entry written at
# another heading depth is invisible to it. That is a deliberate non-extension
# rather than an oversight: `###` subheadings are ordinary INSIDE an entry body
# (86 of them at the time of writing), so a gate that also fired on heading depth
# would cry wolf on every commit that adds a subsection to a landed entry — the
# failure mode that gets a gate deleted. `## ` is the convention all 680 headings
# in the file use and the one COMMIT.md's template produces.
#
# Reads the repository and git, mutates nothing. Deterministic: no clocks, no
# network, no randomness. POSIX/bash-3.2 compatible (no mapfile, no associative
# arrays).
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT}"

FILE="CHANGES.md"

note() { printf '[changes-entry-placement] %s\n' "$1" >&2; }
ok() { printf '[changes-entry-placement] ok: %s\n' "$1"; }

# --- the anti-vacuity floor -------------------------------------------------
#
# Every other path through this check is an EXEMPTION, so without this the gate
# would pass forever and silently the moment its subject stopped existing. Same
# reasoning as check_enumeration_parity.sh's count floors: a check that matches
# nothing must die loudly rather than pass vacuously.
if ! git ls-files --error-unmatch "${FILE}" >/dev/null 2>&1; then
  note "FAIL: ${FILE} is not tracked — the evidence artifact this doctrine governs"
  note "      does not exist, so every commit would pass vacuously. COMMIT.md makes"
  note "      ${FILE} mandatory on every commit; CODE-CHANGE-EVIDENCE gates code"
  note "      changes on it, and DOCTRINE_ENFORCEMENT.md §6 builds the"
  note "      reasoned-from-evidence pattern on top of it."
  exit 1
fi

# --- the staged diff --------------------------------------------------------
#
# -U0 so the hunks carry ZERO context lines: every line then beginning with '+'
# is an ADDED line, and the only other '+'-leading line is the `+++ b/…` file
# header, which cannot match `## ` after its first character is stripped. So the
# extraction below cannot mistake surrounding text for new content.
if git rev-parse --verify -q HEAD >/dev/null 2>&1; then
  diff_out="$(git diff --cached -U0 -- "${FILE}" 2>/dev/null)"
else
  # No HEAD yet (initial commit): diff the index against the empty tree, so the
  # whole staged file reads as added rather than erroring out.
  empty_tree="$(git hash-object -t tree /dev/null)"
  diff_out="$(git diff --cached -U0 "${empty_tree}" -- "${FILE}" 2>/dev/null)"
fi

added="$(printf '%s\n' "${diff_out}" | sed -n 's/^+\(## .*\)$/\1/p')"

if [ -z "${added}" ]; then
  ok "no new ${FILE} entry staged — the predicate has no subject (backfill, edit, or unrelated commit)"
  exit 0
fi

n_added="$(printf '%s\n' "${added}" | grep -c .)"

# --- the resulting file -----------------------------------------------------
#
# The INDEX blob, not the worktree: what matters is the file the commit will
# contain, and a partially-staged worktree must not be able to talk the gate out
# of a breach (or into one).
first="$(git show ":${FILE}" 2>/dev/null | sed -n '/^## /{p;q;}')"

if [ -z "${first}" ]; then
  note "FAIL: the staged ${FILE} diff adds ${n_added} '## ' heading(s), but the staged"
  note "      file contains no '## ' heading at all. That is a contradiction, so"
  note "      treat it as a broken gate rather than as a pass."
  exit 1
fi

if printf '%s\n' "${added}" | grep -qxF -- "${first}"; then
  ok "the newest ${FILE} entry is at the top (${n_added} heading(s) added; first heading is one of them)"
  exit 0
fi

note "FAIL: a new ${FILE} entry was added, but it is NOT at the top of the file."
note ""
note "  the file's first '## ' heading is:"
note "    ${first}"
note "  the heading(s) this commit adds are:"
printf '%s\n' "${added}" | while IFS= read -r h; do note "    ${h}"; done
note ""
note "  ${FILE} declares its own ordering on line 2 — 'Newest entries at the top' —"
note "  and COMMIT.md §2 restates it as a mandatory pre-commit step. A cold session"
note "  reads this file top-down and concludes the last change was whatever sits at"
note "  the top; an entry placed anywhere else is invisible to that reader."
note ""
note "  FIX: move the entry you are about to land so it sits ABOVE the current first"
note "  heading, then re-stage ${FILE}. This is NOT the act decision 0038 forbids —"
note "  that ruling protects a LANDED entry, whose position is itself a record. The"
note "  entry in your index has not landed yet."
note ""
note "  Do NOT 'fix' this by relocating the entry that is currently at the top, and"
note "  do NOT add a pointer stub for an entry that has not been committed."
note ""
note "CHANGES-ENTRY-PLACEMENT breached — commit blocked. See"
note "docs/decisions/0045-changes-entry-placement-authoring-path-check.md and"
note "docs/tasks/CHANGES-ENTRY-PLACEMENT.md."
exit 1
