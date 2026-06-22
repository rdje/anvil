#!/usr/bin/env bash
#
# Single source of truth for ANVIL's memory-architecture invariants.
# Called by .githooks/pre-commit and by CI.
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT_DIR}"

MEMORY_POINTER_LINE_CAP="${MEMORY_POINTER_LINE_CAP:-50}"
TASKS_DIR="docs/tasks"
DECISIONS_DIR="docs/decisions"
BOOTSTRAP_FILES=(
  "AGENTS.md"
  "CLAUDE.md"
  ".cursorrules"
  ".github/copilot-instructions.md"
  "GEMINI.md"
  ".windsurfrules"
)

fail=0
note() {
  printf '[memory-arch] FAIL: %s\n' "$1" >&2
  fail=1
}
ok() {
  printf '[memory-arch] ok:   %s\n' "$1"
}

if [[ -f MEMORY_ARCHITECTURE.md ]]; then
  ok "MEMORY_ARCHITECTURE.md present"
else
  note "MEMORY_ARCHITECTURE.md is missing"
fi

if [[ -f MEMORY.md ]]; then
  lines="$(wc -l < MEMORY.md | tr -d ' ')"
  if [[ "${lines}" -le "${MEMORY_POINTER_LINE_CAP}" ]]; then
    ok "MEMORY.md is ${lines} lines (<= cap ${MEMORY_POINTER_LINE_CAP})"
  else
    note "MEMORY.md is ${lines} lines (> cap ${MEMORY_POINTER_LINE_CAP})"
  fi

  for needle in "active_work_unit" "next_action" "in_flight_uncommitted" "blockers"; do
    if grep -q "${needle}" MEMORY.md; then
      ok "MEMORY.md contains ${needle}"
    else
      note "MEMORY.md is missing ${needle}"
    fi
  done
else
  note "MEMORY.md is missing"
fi

for file in "${BOOTSTRAP_FILES[@]}"; do
  if [[ ! -f "${file}" ]]; then
    note "${file} bootstrap pointer is missing"
    continue
  fi
  if grep -q "README.md" "${file}" && grep -q "MEMORY_ARCHITECTURE.md" "${file}"; then
    ok "${file} points at README.md and MEMORY_ARCHITECTURE.md"
  else
    note "${file} must point at README.md and MEMORY_ARCHITECTURE.md"
  fi
done

if [[ -d "${TASKS_DIR}" && -f docs/TASK_TREE.md ]]; then
  ok "${TASKS_DIR}/ and docs/TASK_TREE.md present"
else
  note "${TASKS_DIR}/ or docs/TASK_TREE.md is missing"
fi

if [[ -d "${DECISIONS_DIR}" && -f "${DECISIONS_DIR}/INDEX.md" ]]; then
  ok "${DECISIONS_DIR}/INDEX.md present"
  shopt -s nullglob
  for record in "${DECISIONS_DIR}"/[0-9][0-9][0-9][0-9]-*.md; do
    base="$(basename "${record}")"
    if grep -q "${base}" "${DECISIONS_DIR}/INDEX.md"; then
      ok "${base} indexed"
    else
      note "${base} missing from ${DECISIONS_DIR}/INDEX.md"
    fi
  done
  shopt -u nullglob
else
  note "${DECISIONS_DIR}/INDEX.md is missing"
fi

if grep -R -n -E 'SpecForge|specforge|Docling|docling|Ollama|qwen|LLM|VLM' \
  MEMORY_ARCHITECTURE.md README.md CHANGES.md MEMORY.md docs/TASK_TREE.md \
  docs/tasks/MEMORY-ARCHITECTURE-DOC.md "${DECISIONS_DIR}" >/dev/null 2>&1; then
  note "donor-project-specific residue found in memory architecture files"
else
  ok "no donor-project-specific residue in memory architecture files"
fi

if [[ "${fail}" -ne 0 ]]; then
  printf '[memory-arch] memory-architecture check FAILED\n' >&2
  exit 1
fi

printf '[memory-arch] all memory-architecture invariants hold\n'
