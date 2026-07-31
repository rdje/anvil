#!/usr/bin/env bash
#
# Single source of truth for ANVIL's memory-architecture invariants.
# Called by .githooks/pre-commit and by CI.
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT_DIR}"

MEMORY_POINTER_LINE_CAP="${MEMORY_POINTER_LINE_CAP:-50}"

# --- the byte cap (decision 0040 §(c), OVERFLOW-DESTINATION-INSTRUMENTATION.3) ---
#
# WHY A SECOND AXIS. A line cap bounds a PROJECTION of a file, not the file, and
# content flows into the axis nobody measures. Measured here: after this line cap
# landed at 2d01e8e (2026-06-05) — truncating MEMORY.md from 2,399 lines /
# 306,099 B to 19 / 1,227 / 64 bytes-per-line — the line count went 19 -> 50 and
# STOPPED while bytes went 1,227 -> 20,311 (x16.6) and density 64 -> 406 B/line
# (x6.3), against 65 B/line for README.md. The cap bound perfectly on the axis it
# measured for two months while the file grew to 1.65x the byte cap its sibling
# landing page is held to. Nobody evaded anything; they wrote what they had to say
# on the one line the cap left. This is decision 0036 §(c)'s "prose density is the
# hidden variable" arriving in the file every session reads first.
#
# NOT A NEW DOCTRINE, deliberately. MEMORY-ARCH already owns this file's size, so
# the byte cap is a second assertion here rather than a registered check of its
# own; a second mechanism for one job is what feedback_full_factorization forbids.
#
# THE CAP IS DERIVED, NOT FITTED. Contract: "roughly one screen" at <= 50 lines
# (MEMORY_ARCHITECTURE.md §6). Demonstrated-achievable density for THIS file is
# 64 B/line (5043547, the first commit after the architecture landed, when the
# content was correctly layered); README.md — a LARGER contract — runs 65 B/line;
# decision 0036 measured 118 B/line as the highest legitimate prose density.
# Budgeting generously at ~120 B/line x 50 lines => 6,144 bytes, deliberately HALF
# README.md's 12,288, a resume pointer being a strictly smaller contract than a
# landing page.
#
# RAISING THE CAP IS NOT A FIX. If MEMORY.md exceeds it, information is in the
# wrong layer (MEMORY_ARCHITECTURE.md §6: "move it down to B or C"). Raising this
# number requires a new decision record stating that the resume-pointer contract
# itself expanded — not an edit to this line. See decision 0040 non-license 3.
MEMORY_POINTER_BYTE_CAP="${MEMORY_POINTER_BYTE_CAP:-6144}"
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

  bytes="$(wc -c < MEMORY.md | tr -d ' ')"
  if [[ "${bytes}" -le "${MEMORY_POINTER_BYTE_CAP}" ]]; then
    ok "MEMORY.md is ${bytes} bytes (<= cap ${MEMORY_POINTER_BYTE_CAP})"
  else
    note "MEMORY.md is ${bytes} bytes (> cap ${MEMORY_POINTER_BYTE_CAP})"
    # Routing hint. Both destinations are CLASSIFIED (decision 0040): layer B and
    # layer C are append-only records, which are SUPPOSED to grow and are never
    # capped — so this hint does not reproduce the hole it exists to close.
    printf '[memory-arch]   Layer A is a POINTER, not a summary. Move content down, do not trim prose:\n' >&2
    printf '[memory-arch]     a durable cross-cutting fact  -> docs/decisions/ (layer C, append-only)\n' >&2
    printf '[memory-arch]     an operating gotcha           -> docs/knowledge/ fact card (question-keyed)\n' >&2
    printf '[memory-arch]     per-unit work state           -> docs/tasks/ (layer B, append-only)\n' >&2
    printf '[memory-arch]     history of what changed       -> git log + CHANGES.md\n' >&2
    printf '[memory-arch]   Leave a POINTER behind, never a second copy (decisions 0033, 0038).\n' >&2
    printf '[memory-arch]   Raising this cap requires a new decision record, not an edit to the script.\n' >&2
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
