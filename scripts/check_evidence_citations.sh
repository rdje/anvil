#!/usr/bin/env bash
#
# scripts/check_evidence_citations.sh — the EVIDENCE-CITATIONS doctrine.
# Owning leaf: EVIDENCE-BANK-DURABILITY.3. Contract: decision 0030 plus its
# 2026-07-30 amendment (docs/decisions/0030-durable-closure-evidence-citations.md).
#
# DOCTRINE: a closure or gate claim must cite evidence that outlives the machine
# it ran on. Concretely: a committed digest under docs/evidence/, never a bare
# output-directory name whose artifact has evaporated.
#
# ARCHETYPE: structural (DOCTRINE_ENFORCEMENT.md §3) — it re-derives the fact
# from the tree, so it cannot be satisfied by a claim in a commit message.
#
# CONTRACT (§4): exit 0 iff the doctrine holds; nonzero + an actionable stderr
# message on breach; deterministic; reads the repo and mutates nothing.
#
# ── WHY THIS CHECK LOOKS THE WAY IT DOES ────────────────────────────────────
# Decision 0030 specified keying on a bare `/tmp/anvil-*` path. That
# discriminator no longer exists: VOLUME-DATA-LOCALITY.5 stripped every `/tmp/`
# prefix, so what a live document cites today is a bare `anvil-<name>` token —
# a shape ANVIL also uses for binaries (anvil-mcp), directories (anvil-sandbox),
# Action inputs (anvil-bin) and even English prose ("anvil-emitted"). There is
# no lexical rule separating `anvil-cf-sweep` (a bank) from `anvil-hunt-bundles`
# (not one).
#
# So this check does NOT guess. Every token is classified exactly once, and an
# unclassified token is a breach — it fails CLOSED:
#
#   1. digest-backed  docs/evidence/<token>.md exists and is schema-valid.
#                     The forward path; grows without limit.
#   2. grandfathered  listed in docs/evidence/INVENTORY.md §1. FROZEN — see below.
#   3. not evidence   listed in docs/evidence/INVENTORY.md §2. Grows under review.
#
# ── WHY §1 IS PINNED AND §2 IS NOT ──────────────────────────────────────────
# The membership of §1 is a HISTORICAL FACT: the set of banks that existed
# before decision 0030. It cannot legitimately grow, because you cannot
# retroactively acquire pre-0030 evidence — so a grandfathered list that grows
# is a false statement, and would be the obvious escape hatch from the whole
# doctrine ("just grandfather it"). Both its entry count and the SHA-256 of its
# sorted membership are therefore pinned below; widening it means editing this
# script too, in the same reviewed commit.
#
# §2 is NOT pinned because ANVIL's vocabulary legitimately grows — a new binary
# or directory is normal. The asymmetry is semantic, not convenience.
#
# ── SCAN SET BY EXCLUSION, NOT ENUMERATION ──────────────────────────────────
# Every tracked *.md except:
#   CHANGES.md, DEVELOPMENT_NOTES.md  append-only history; decision 0031 forbids
#                                     retro-editing them, and a past entry citing
#                                     a past bank must stay raw
#   KNOWLEDGE_MAP.md                  generated; it mirrors docs/knowledge sources
#   docs/evidence/                    the inventory and the digests must name the
#                                     tokens they classify (the policy-document
#                                     principle NO-BOOT-VOLUME-REFS already uses)
# Defining the set by exclusion means a NEW live document is in scope
# automatically. An enumerated list would silently go stale — which is exactly
# how the /tmp citations survived LIVE-DOC-PATH-HYGIENE.
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT}"

EVIDENCE_DIR="docs/evidence"
INVENTORY="${EVIDENCE_DIR}/INVENTORY.md"

# The frozen §1 pin. Update ONLY together with a reviewed INVENTORY.md §1 edit.
GRANDFATHERED_COUNT=72
GRANDFATHERED_SHA256="6d80a802d64829a05abe8b89ec283897e66b54023bf3bcc0c3691bbdb6da6892"

# Files excluded from the scan (see rationale above).
SCAN_SKIP_RE='^(CHANGES\.md|DEVELOPMENT_NOTES\.md|KNOWLEDGE_MAP\.md|docs/evidence/)'

# A citation-shaped token. Deliberately broad: narrowing it would be guessing.
TOKEN_RE='anvil-[a-z0-9][A-Za-z0-9_.-]*'

fail=0
note() { printf 'evidence-citations: %s\n' "$1" >&2; }

sha256_of() {  # portable: macOS shasum, GNU sha256sum
  if command -v shasum >/dev/null 2>&1; then shasum -a 256 "$1" | cut -d' ' -f1
  else sha256sum "$1" | cut -d' ' -f1; fi
}

[ -f "${INVENTORY}" ] || { note "${INVENTORY} is missing — the classification cannot be read"; exit 1; }

# ── Read the two inventory sections ─────────────────────────────────────────
# §1 runs from the "## 1." heading to the "## 2." heading; §2 to end of file.
grandfathered="$(awk '/^## 1\./{s=1} /^## 2\./{s=0} s' "${INVENTORY}" \
                 | sed -n 's/^- `\('"${TOKEN_RE}"'\)`.*/\1/p' | sort -u)"
not_evidence="$(awk '/^## 2\./{s=1} s' "${INVENTORY}" \
                 | sed -n 's/^- `\('"${TOKEN_RE}"'\)`.*/\1/p' | sort -u)"

# ── Leg 1: §1 is frozen ─────────────────────────────────────────────────────
g_count=$(printf '%s\n' "${grandfathered}" | grep -c . || true)
g_tmp="$(mktemp)"; printf '%s\n' "${grandfathered}" > "${g_tmp}"
g_sha="$(sha256_of "${g_tmp}")"; rm -f "${g_tmp}"

if [ "${g_count}" -ne "${GRANDFATHERED_COUNT}" ] || [ "${g_sha}" != "${GRANDFATHERED_SHA256}" ]; then
  fail=1
  note "the grandfathered list in ${INVENTORY} §1 has changed — it is frozen."
  note "  expected ${GRANDFATHERED_COUNT} entries, sha256 ${GRANDFATHERED_SHA256}"
  note "  found    ${g_count} entries, sha256 ${g_sha}"
  note "WHY: §1 is the set of banks that existed BEFORE decision 0030. That is a"
  note "     historical fact, so it cannot grow. To admit a NEW bank, write a"
  note "     digest: scripts/evidence_digest.sh <report.json> <bank-name> ..."
  note "     If you are genuinely correcting the historical record, update the"
  note "     GRANDFATHERED_* pin in this script in the same reviewed commit."
fi

# ── Leg 2: every digest is schema-valid ─────────────────────────────────────
shopt -s nullglob
for d in "${EVIDENCE_DIR}"/*.md; do
  base="$(basename "${d}" .md)"
  case "${base}" in README|INVENTORY) continue ;; esac
  miss=()
  grep -qE '^- bank: `'"${base}"'`$'                  "${d}" || miss+=("bank (must be \`${base}\`, matching the filename)")
  grep -qE '^- claim: .+'                             "${d}" || miss+=("claim")
  grep -qE '^- owning_leaf: `[A-Z][A-Z0-9-]+(\.[0-9A-Za-z]+)*`$' "${d}" || miss+=("owning_leaf")
  grep -qE '^- commit: `[0-9a-f]{7,40}`$'             "${d}" || miss+=("commit (7-40 hex)")
  grep -qE '^- date: `[0-9]{4}-[0-9]{2}-[0-9]{2}`$'   "${d}" || miss+=("date (YYYY-MM-DD)")
  grep -qE '^- command: `.+`$'                        "${d}" || miss+=("command (the re-runnable oracle)")
  grep -qE '^- report_sha256: `[0-9a-f]{64}`$'        "${d}" || miss+=("report_sha256 (64 hex)")
  grep -qE '^- coverage_gaps: '                       "${d}" || miss+=("coverage_gaps")
  if [ ${#miss[@]} -gt 0 ]; then
    fail=1
    note "digest ${d} is not schema-valid; missing or malformed:"
    for m in "${miss[@]}"; do note "  · ${m}"; done
    note "  see ${EVIDENCE_DIR}/README.md for the schema."
  fi
done

# ── Leg 3: every cited token is classified ──────────────────────────────────
mapfile -t scan < <(git ls-files '*.md' | grep -vE "${SCAN_SKIP_RE}" || true)
[ ${#scan[@]} -gt 0 ] || { note "scan set is empty — refusing to pass vacuously"; exit 1; }

unclassified=0
while read -r token; do
  [ -n "${token}" ] || continue
  [ -f "${EVIDENCE_DIR}/${token}.md" ] && continue
  printf '%s\n' "${grandfathered}" | grep -qxF "${token}" && continue
  printf '%s\n' "${not_evidence}"   | grep -qxF "${token}" && continue
  fail=1; unclassified=$((unclassified + 1))
  # Resolve the file only for a breach: attribution is for the error message,
  # so paying for it per-token on a clean tree would be pure waste. -F because
  # a token may contain `.` (e.g. anvil-bisim-merged.sv).
  where="$(grep -lF "${token}" -- "${scan[@]}" 2>/dev/null | head -3 | tr '\n' ' ')"
  note "unclassified evidence citation \`${token}\` in ${where:-<unknown>}"
done < <(grep -ohE "${TOKEN_RE}" -- "${scan[@]}" 2>/dev/null | sed 's/\.*$//' | sort -u)

if [ "${unclassified}" -gt 0 ]; then
  note ""
  note "FIX: ${unclassified} token(s) match the citation shape but are classified nowhere."
  note "  · a NEW banked run -> write a digest, the forward path:"
  note "      scripts/evidence_digest.sh <report.json> <bank-name> --leaf <ID> --claim '<what it backs>'"
  note "  · not evidence at all (a binary, a directory, prose)? add it to"
  note "      ${INVENTORY} §2, with the reason."
  note "  · §1 is FROZEN and is not the answer for anything new."
fi

if [ "${fail}" -eq 0 ]; then
  n_dig=$(ls -1 "${EVIDENCE_DIR}"/*.md 2>/dev/null | grep -vcE '/(README|INVENTORY)\.md$' || true)
  echo "[evidence-citations] ok: every cited bank is digest-backed or classified (${n_dig} digest(s), ${GRANDFATHERED_COUNT} grandfathered, frozen)"
fi
exit "${fail}"
