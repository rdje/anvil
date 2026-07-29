#!/usr/bin/env bash
#
# scripts/evidence_digest.sh — derive a committed evidence digest from a
# tool_matrix_report.json. Owning leaf: EVIDENCE-BANK-DURABILITY.3.
# Contract: decision 0030 + its 2026-07-30 amendment.
#
# A banked corpus is bulk and belongs nowhere near git (a --phase4-hierarchy-gate
# bank is 840 designs). The claim-bearing NUMBERS are what must survive, so this
# derives a few-KB tracked digest from the report and leaves the corpus on disk.
#
# USAGE
#   scripts/evidence_digest.sh <report.json> <bank-name> \
#       --leaf <TREE.LEAF> --claim '<what this backs>' [--command '<cmd>'] [--out <path>]
#
#   <bank-name>  the run's directory basename, e.g. anvil-case-mux-if-gate-r2.
#                It becomes docs/evidence/<bank-name>.md, and the digest's
#                `bank:` field must match the filename (the check enforces it).
#   --command    the exact re-runnable oracle. Defaults to a reconstruction from
#                the report's own gate flags — always review it before committing.
#
# CONVENTION: write banks under .cache/anvil-sandbox/<bank-name>/ (on-volume,
# gitignored). The digest is the only tracked artifact.
#
# PARSING: tool_matrix writes serde_json PRETTY output, so every top-level
# scalar is `  "key": value,` on its own line. This reads exactly that shape and
# FAILS LOUDLY on a missing field rather than emitting a digest with blanks — a
# signoff artifact with a silently-wrong number is worse than no artifact.
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

die() { printf 'evidence-digest: %s\n' "$1" >&2; exit 1; }

[ $# -ge 2 ] || die "usage: $0 <report.json> <bank-name> --leaf <ID> --claim '<text>' [--command '<cmd>'] [--out <path>]"
REPORT="$1"; BANK="$2"; shift 2
LEAF=""; CLAIM=""; COMMAND=""; OUT=""
while [ $# -gt 0 ]; do
  case "$1" in
    --leaf)    LEAF="${2:-}";    shift 2 ;;
    --claim)   CLAIM="${2:-}";   shift 2 ;;
    --command) COMMAND="${2:-}"; shift 2 ;;
    --out)     OUT="${2:-}";     shift 2 ;;
    *) die "unknown argument: $1" ;;
  esac
done

[ -f "${REPORT}" ] || die "report not found: ${REPORT}"
[ -n "${BANK}" ]   || die "bank name is required"
[ -n "${LEAF}" ]   || die "--leaf is required (the task-tree leaf that owns the claim)"
[ -n "${CLAIM}" ]  || die "--claim is required (what closure or gate this evidence backs)"
printf '%s' "${BANK}" | grep -qE '^anvil-[a-z0-9][A-Za-z0-9_.-]*$' \
  || die "bank name must match anvil-<name>: ${BANK}"

OUT="${OUT:-${ROOT}/docs/evidence/${BANK}.md}"

sha256_of() {
  if command -v shasum >/dev/null 2>&1; then shasum -a 256 "$1" | cut -d' ' -f1
  else sha256sum "$1" | cut -d' ' -f1; fi
}

# Top-level scalar out of serde-pretty JSON. Empty result => caller decides.
jnum() { sed -n 's/^  "'"$1"'": \([0-9][0-9]*\),\{0,1\}$/\1/p' "${REPORT}" | head -1; }
jstr() { sed -n 's/^  "'"$1"'": "\([^"]*\)",\{0,1\}$/\1/p' "${REPORT}" | head -1; }
jbool(){ sed -n 's/^  "'"$1"'": \(true\|false\),\{0,1\}$/\1/p' "${REPORT}" | head -1; }
# Nested one level (inside tool_summary), serde-pretty indents by 4.
jnum2() { sed -n 's/^    "'"$1"'": \([0-9][0-9]*\),\{0,1\}$/\1/p' "${REPORT}" | head -1; }

require() { [ -n "$2" ] || die "field '$1' not found in ${REPORT} — is it a tool_matrix_report.json (serde-pretty)?"; printf '%s' "$2"; }

SCENARIOS=$(require scenario_count       "$(jnum scenario_count)")
PER_SCEN=$(require modules_per_scenario  "$(jnum modules_per_scenario)")
TOTAL=$(require total_modules            "$(jnum total_modules)")
BASE_SEED=$(require base_seed            "$(jnum base_seed)")
SET=$(require scenario_set               "$(jstr scenario_set)")
KIND=$(require artifact_kind             "$(jstr artifact_kind)")
YMODE=$(require yosys_mode               "$(jstr yosys_mode)")

# coverage_gaps: `[]` inline when empty, otherwise a multi-line array.
if grep -qE '^  "coverage_gaps": \[\],\{0,1\}$' "${REPORT}"; then
  GAPS="\`[]\` — none"
  GAPS_N=0
else
  GAPS_N=$(awk '/^  "coverage_gaps": \[/{s=1;next} s&&/^  \]/{exit} s&&/"/{n++} END{print n+0}' "${REPORT}")
  GAPS="**${GAPS_N} gap(s)** — see the report; a digest with gaps does NOT back a closure claim"
fi

# Which gate flag drove the run (for the reconstructed command).
GATE_FLAG=""
for g in phase1_gate phase2_share_gate phase3_structured_gate phase4_hierarchy_gate \
         signoff_knob_sweep_gate sv_version_gate function_emit_gate generate_loop_gate \
         task_emit_gate cone_function_gate multi_output_task_gate mux_if_gate \
         case_mux_if_gate casez_mux_if_gate; do
  [ "$(jbool "$g")" = "true" ] && GATE_FLAG="--$(printf '%s' "$g" | tr '_' '-')"
done
if [ -z "${COMMAND}" ]; then
  COMMAND="cargo run --release --bin tool_matrix -- ${GATE_FLAG} --yosys-mode ${YMODE} --out .cache/anvil-sandbox/${BANK}"
fi

COMMIT="$(cd "${ROOT}" && git rev-parse --short=12 HEAD)"
DATE="$(cd "${ROOT}" && git log -1 --date=short --format=%cd)"
SHA="$(sha256_of "${REPORT}")"

row() { # name pass fail
  [ -n "$2" ] && [ -n "$3" ] && printf '| %s | %s | %s |\n' "$1" "$2" "$3"
}

mkdir -p "$(dirname "${OUT}")"
{
  printf '# %s\n\n' "${BANK}"
  printf 'Evidence digest — decision [`0030`](../decisions/0030-durable-closure-evidence-citations.md).\n'
  printf 'Derived from the run'"'"'s `tool_matrix_report.json` by `scripts/evidence_digest.sh`; the\n'
  printf 'corpus itself is not tracked (bulk), this digest is.\n\n'
  printf -- '- bank: `%s`\n' "${BANK}"
  printf -- '- claim: %s\n' "${CLAIM}"
  printf -- '- owning_leaf: `%s`\n' "${LEAF}"
  printf -- '- commit: `%s`\n' "${COMMIT}"
  printf -- '- date: `%s`\n' "${DATE}"
  printf -- '- command: `%s`\n' "${COMMAND}"
  printf -- '- report_sha256: `%s`\n' "${SHA}"
  printf -- '- coverage_gaps: %s\n\n' "${GAPS}"
  printf '## Run shape\n\n'
  printf -- '- scenario_set: `%s`\n' "${SET}"
  printf -- '- artifact_kind: `%s`\n' "${KIND}"
  printf -- '- base_seed: `%s`\n' "${BASE_SEED}"
  printf -- '- scenarios: `%s` × `%s` per scenario = **`%s` units**\n' "${SCENARIOS}" "${PER_SCEN}" "${TOTAL}"
  printf -- '- yosys_mode: `%s`\n\n' "${YMODE}"
  printf '## Tool columns\n\n'
  printf '| column | pass | fail |\n| --- | --- | --- |\n'
  row 'Verilator'         "$(jnum2 verilator_passed)"          "$(jnum2 verilator_failed)"
  row 'Yosys without-abc' "$(jnum2 yosys_without_abc_passed)"  "$(jnum2 yosys_without_abc_failed)"
  row 'Yosys with-abc'    "$(jnum2 yosys_with_abc_passed)"     "$(jnum2 yosys_with_abc_failed)"
  row 'Icarus compile'    "$(jnum2 iverilog_compile_passed)"   "$(jnum2 iverilog_compile_failed)"
  row 'sv2v'              "$(jnum2 sv2v_passed)"               "$(jnum2 sv2v_failed)"
  row 'slang'             "$(jnum2 slang_passed)"              "$(jnum2 slang_failed)"
  printf '\n## Re-verification\n\n'
  printf 'Re-run `command` above at commit `%s`. The numbers here are what a clean\n' "${COMMIT}"
  printf 're-run must reproduce; a divergence is a finding, not a stale digest.\n'
  printf 'Later commits may legitimately produce different numbers — that is why the\n'
  printf 'commit is recorded alongside them.\n'
} > "${OUT}"

printf 'evidence-digest: wrote %s\n' "${OUT#"${ROOT}"/}" >&2
printf 'evidence-digest: REVIEW the derived `command` before committing.\n' >&2
