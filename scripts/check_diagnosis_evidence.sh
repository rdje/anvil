#!/usr/bin/env bash
#
# scripts/check_diagnosis_evidence.sh — the CODE-CHANGE-EVIDENCE doctrine (scope-aware).
#
# DOCTRINE_ENFORCEMENT.md §3 (evidence archetype) / §4 (contract) / §6 / §9. COMMIT.md
# mandates: "CHANGES.md and MEMORY.md MUST be amended before every git commit, without
# exception." This check mechanizes the live-doc EVIDENCE leg of that for *code* changes:
#
#   if the staged set touches code, then CHANGES.md AND MEMORY.md must be staged too.
#
# Pure non-code commits (docs / live-doc / mdBook / workflow) are EXEMPT — the doctrine
# does not govern them (DOCTRINE_ENFORCEMENT.md §4(5)).
#
# This is a STRUCTURAL co-staging proxy at pre-commit; the un-fakeable oracle leg is the
# cargo + tool_matrix re-run at COMMIT.md / CI (§6.1, §9 honest limit). The acceptance
# checklist the change must satisfy lives in TOOLBOX.md Part 2.
#
# Reads git, mutates nothing. Deterministic. POSIX/bash-3.2 compatible (no mapfile).
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT}"

# The staged set (the change under consideration). DOCTRINE_STAGED_OVERRIDE is a
# newline-separated path list used ONLY by the self-test; default reads the git index.
if [ -n "${DOCTRINE_STAGED_OVERRIDE:-}" ]; then
  staged="${DOCTRINE_STAGED_OVERRIDE}"
else
  staged="$(git diff --cached --name-only 2>/dev/null || true)"
fi

# "Code" = anything that changes program/generator behaviour or generated RTL
# (docs/TASK_TREE.md "ANVIL Adoption Scope"): src/, tests/, examples/, build/codegen,
# behaviour-altering Cargo manifests. scripts/, .githooks/, docs/, *.md are NOT code.
code_re='^(src/|tests/|examples/|build\.rs$|Cargo\.toml$|Cargo\.lock$)'

if ! printf '%s\n' "${staged}" | grep -Eq "${code_re}"; then
  printf '[code-change-evidence] ok: no code staged — exempt (docs/workflow commit).\n'
  exit 0
fi

fail=0
printf '%s\n' "${staged}" | grep -qx 'CHANGES.md' || {
  printf '[code-change-evidence] FAIL: a code change must stage CHANGES.md in the same commit (COMMIT.md mandatory; TOOLBOX.md Part 2).\n' >&2
  fail=1
}
printf '%s\n' "${staged}" | grep -qx 'MEMORY.md' || {
  printf '[code-change-evidence] FAIL: a code change must stage MEMORY.md in the same commit (COMMIT.md mandatory; TOOLBOX.md Part 2).\n' >&2
  fail=1
}

if [ "${fail}" -ne 0 ]; then
  printf '[code-change-evidence] the un-fakeable proof is the cargo + tool_matrix oracle re-run (COMMIT.md / CI); see DOCTRINE_ENFORCEMENT.md §6.1.\n' >&2
  exit 1
fi

printf '[code-change-evidence] ok: code change co-stages CHANGES.md + MEMORY.md.\n'
exit 0
