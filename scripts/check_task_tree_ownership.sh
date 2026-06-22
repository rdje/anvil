#!/usr/bin/env bash
#
# scripts/check_task_tree_ownership.sh — the TASK-TREE-OWNERSHIP doctrine (scope-aware).
#
# DOCTRINE_ENFORCEMENT.md §3 (structural) / §4 (contract). The 2026-05-17 owner directive
# (docs/TASK_TREE.md "ANVIL Adoption Scope"; SESSION_BOOTSTRAP.md; COMMIT.md task-tree
# rules) is non-negotiable: NO code change may be made without a task-tree leaf owning it,
# and COMMIT.md task-tree rule #2 requires the owning docs/tasks/<TREE>.md to be updated
# in the same commit. This check mechanizes that as a structural co-staging proxy:
#
#   if the staged set touches code, then at least one docs/tasks/*.md task file
#   (other than TEMPLATE.md) must be staged in the same commit.
#
# Pure non-code commits (docs / live-doc / mdBook / workflow) are EXEMPT (§4(5)).
#
# Honest limit (§9): this proves an owning task file is *co-staged*. The un-fakeable leg
# is the commit-msg hook (the subject must carry a task-tree leaf id) plus review that
# the staged task file is the genuine owner. Reads git, mutates nothing. Deterministic.
# POSIX/bash-3.2 compatible (no mapfile).
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT}"

# The staged set. DOCTRINE_STAGED_OVERRIDE is a newline-separated path list used ONLY by
# the self-test; default reads the git index.
if [ -n "${DOCTRINE_STAGED_OVERRIDE:-}" ]; then
  staged="${DOCTRINE_STAGED_OVERRIDE}"
else
  staged="$(git diff --cached --name-only 2>/dev/null || true)"
fi

# "Code" = anything that changes program/generator behaviour or generated RTL
# (docs/TASK_TREE.md "ANVIL Adoption Scope"). Identical scope to CODE-CHANGE-EVIDENCE.
code_re='^(src/|tests/|examples/|build\.rs$|Cargo\.toml$|Cargo\.lock$)'

if ! printf '%s\n' "${staged}" | grep -Eq "${code_re}"; then
  printf '[task-tree-ownership] ok: no code staged — exempt (docs/workflow commit).\n'
  exit 0
fi

# Code is staged ⇒ require an owning task file co-staged (TEMPLATE.md does not count).
if printf '%s\n' "${staged}" | grep -E '^docs/tasks/.+\.md$' | grep -qv '^docs/tasks/TEMPLATE\.md$'; then
  printf '[task-tree-ownership] ok: code change co-stages an owning docs/tasks/*.md task file.\n'
  exit 0
fi

printf '[task-tree-ownership] FAIL: a code change must be owned by a task-tree leaf — stage the owning docs/tasks/<TREE>.md in the same commit (2026-05-17 doctrine; COMMIT.md task-tree rule #2).\n' >&2
printf '[task-tree-ownership] also ensure the commit subject carries the leaf id (.githooks/commit-msg). See docs/TASK_TREE.md "ANVIL Adoption Scope".\n' >&2
exit 1
