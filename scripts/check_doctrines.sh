#!/usr/bin/env bash
#
# scripts/check_doctrines.sh — ANVIL's doctrine-enforcement registry + driver.
#
# See DOCTRINE_ENFORCEMENT.md. This is the single general enforcer for portable
# architecture #4: it runs every registered doctrine check, COLLECTS ALL RESULTS
# (it does not stop at the first failure), META-CHECKS that each registered check
# exists and is executable (so a registry entry is never a dangling promise),
# prints a per-doctrine PASS/FAIL report, and exits nonzero iff any check failed.
#
# Called by .githooks/pre-commit (E3) and .github/workflows/ci.yml (E4). The same
# driver fires identically no matter which harness (or human) made the commit.
#
# Add a doctrine = write scripts/check_<id>.sh obeying the DOCTRINE_ENFORCEMENT.md
# §4 contract (exit code is the verdict; deterministic; scope-aware where relevant;
# reads the repo, mutates nothing) and add one line to the DOCTRINES array below.
#
# NOTE: deliberately NOT `set -e` — the driver must run every check and aggregate.
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT}"

# --- Registry: "id|what-it-proves|path/to/check.sh" (the single source of truth) ---
# Mirror DOCTRINE_ENFORCEMENT.md §10. Heavy oracle doctrines (tests/snapshots.rs
# byte-identical reproducibility, the tool_matrix --<surface>-gate downstream gates)
# stay where they re-execute the real generator + tools (cargo test / local matrix
# per COMMIT.md) and are referenced there, not duplicated into this fast gate.
DOCTRINES=(
  "MEMORY-ARCH|durable 4-layer memory-architecture invariants (MEMORY_ARCHITECTURE.md §9)|scripts/check_memory_architecture.sh"
  "KNOWLEDGE-MAP|the derived KNOWLEDGE_MAP.md is in sync with its fact sources|knowledge-map/scripts/check_knowledge_map.sh"
)

fail=0
meta_fail=0
declare -a results=()

note() { printf '[doctrines] %s\n' "$1" >&2; }

for entry in "${DOCTRINES[@]}"; do
  id="${entry%%|*}"
  rest="${entry#*|}"
  proves="${rest%%|*}"
  script="${rest##*|}"

  # Meta-check: a registered check must exist and be executable (no dangling promise).
  if [[ ! -f "${script}" ]]; then
    note "META-FAIL: ${id} — registered check missing: ${script}"
    results+=("META  ${id}  (${script} missing)")
    meta_fail=1; fail=1
    continue
  fi
  if [[ ! -x "${script}" ]]; then
    note "META-FAIL: ${id} — registered check not executable: ${script} (chmod +x it)"
    results+=("META  ${id}  (${script} not executable)")
    meta_fail=1; fail=1
    continue
  fi

  # Run the check; its exit code is the verdict. Do NOT abort the driver on failure.
  if bash "${script}"; then
    results+=("PASS  ${id}")
  else
    note "FAIL: ${id} — ${proves}"
    results+=("FAIL  ${id}")
    fail=1
  fi
done

note "===== doctrine report ====="
for r in "${results[@]}"; do note "${r}"; done

if [[ "${fail}" -ne 0 ]]; then
  [[ "${meta_fail}" -ne 0 ]] && note "REGISTRY ERROR: a registered check is missing or not executable."
  note "one or more doctrines FAILED — commit blocked. See DOCTRINE_ENFORCEMENT.md."
  exit 1
fi

note "all ${#DOCTRINES[@]} registered doctrines hold."
exit 0
