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
#   --commit     the commit whose code produced these numbers. Defaults to HEAD,
#                which is the PARENT of the banking commit whenever the tree is
#                dirty (the normal case) — the script warns loudly when so. See
#                the `commit:` derivation below (EVIDENCE-BANK-DURABILITY.6).
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

# coverage_gaps is `[]` on one line when empty, otherwise a multi-line array.
# Echoes the count, or nothing when the field is absent — the caller MUST treat
# empty as fatal. A silently-wrong number in a signoff artifact is worse than no
# artifact, which is not hypothetical: the first real use of this script emitted
# "2907 gap(s)" for an empty array, because the empty-array branch was written
# with a BRE interval `\{0,1\}` inside `grep -E`, where `\{` is a LITERAL brace.
# It never matched, and the multi-line fallback then counted quoted strings to
# end-of-file. Hence `--self-test` below.
extract_gaps() {
  awk '
    /^  "coverage_gaps": \[\],?$/ { print 0; found=1; exit }
    /^  "coverage_gaps": \[$/     { s=1; next }
    s && /^  \],?$/               { print n+0; found=1; exit }
    s && /"/                      { n++ }
    END { if (!found && !s) exit 0 }
  ' "$1"
}

# EVIDENCE-BANK-DURABILITY.6 — every `*_gate` field the report records as `true`,
# one per line. The ONE extractor: the caller-side classification and the
# self-test both call this, so a change to the pattern cannot pass a test that
# kept a stale copy of it. (Defect A was a hardcoded list shadowing the report;
# a self-test with its own copy of the regex would be the same mistake one level
# up.)
true_gates_in() { sed -n 's/^  "\([a-z0-9_]*_gate\)": true,\{0,1\}$/\1/p' "$1"; }

self_test() {
  local d="${ROOT}/.cache/anvil-sandbox/evidence-digest-selftest"
  rm -rf "${d}"; mkdir -p "${d}"; local rc=0
  printf '{\n  "coverage_gaps": [],\n  "x": 1\n}\n' > "${d}/empty.json"
  printf '{\n  "coverage_gaps": [\n    "saw_a",\n    "saw_b"\n  ],\n  "x": 1\n}\n' > "${d}/two.json"
  printf '{\n  "x": 1\n}\n' > "${d}/absent.json"
  check() { # label expected actual
    if [ "$2" = "$3" ]; then printf '  ok   %-28s -> %s\n' "$1" "$3"
    else printf '  FAIL %-28s -> got [%s], want [%s]\n' "$1" "$3" "$2"; rc=1; fi
  }
  check "empty array => 0"      "0"  "$(extract_gaps "${d}/empty.json")"
  check "two gaps => 2"         "2"  "$(extract_gaps "${d}/two.json")"
  check "absent field => empty" ""   "$(extract_gaps "${d}/absent.json")"

  # EVIDENCE-BANK-DURABILITY.6 — the gate-flag classification. These four cases
  # are the negative controls for defect A: the old hardcoded enumeration passed
  # case 1 only for gates it already knew, and silently produced a FLAGLESS
  # command for cases 1 and 3 alike. A rule nobody has watched fail is a rule
  # nobody knows works, so the controls live here rather than in a session log.
  # 1: a gate name this script has never been told about.
  printf '{\n  "scenario_set": "brand-new-set",\n  "some_future_gate": true,\n  "mux_if_gate": false\n}\n' > "${d}/future.json"
  check "unknown gate is derived"  "some_future_gate" "$(true_gates_in "${d}/future.json")"
  # 2: two true flags — mutually exclusive, so malformed.
  printf '{\n  "scenario_set": "x",\n  "a_gate": true,\n  "b_gate": true\n}\n' > "${d}/two-gates.json"
  check "two true gates => 2"      "2" "$(true_gates_in "${d}/two-gates.json" | grep -c .)"
  # 3: non-default set, no true flag — the silent-wrong case; must be 0 here so
  #    the caller-side `case` can die on it.
  printf '{\n  "scenario_set": "gated",\n  "x_gate": false\n}\n' > "${d}/none.json"
  check "no true gate => 0"        "0" "$(true_gates_in "${d}/none.json" | grep -c .)"
  # 4: a false flag is never mistaken for a true one.
  printf '{\n  "scenario_set": "default",\n  "phase1_gate": false\n}\n' > "${d}/false.json"
  check "false flag not derived"   ""  "$(true_gates_in "${d}/false.json")"

  rm -rf "${d}"
  [ "${rc}" -eq 0 ] && echo "evidence-digest: self-test PASS" || echo "evidence-digest: self-test FAIL" >&2
  return "${rc}"
}

if [ "${1:-}" = "--self-test" ]; then self_test; exit $?; fi

[ $# -ge 2 ] || die "usage: $0 <report.json> <bank-name> --leaf <ID> --claim '<text>' [--command '<cmd>'] [--commit <sha>] [--out <path>]
       $0 --self-test"
REPORT="$1"; BANK="$2"; shift 2
LEAF=""; CLAIM=""; COMMAND=""; OUT=""; COMMIT=""; COMMIT_PENDING=0
while [ $# -gt 0 ]; do
  case "$1" in
    --leaf)    LEAF="${2:-}";    shift 2 ;;
    --claim)   CLAIM="${2:-}";   shift 2 ;;
    --command) COMMAND="${2:-}"; shift 2 ;;
    --commit)  COMMIT="${2:-}";  shift 2 ;;
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

GAPS_N="$(extract_gaps "${REPORT}")"
[ -n "${GAPS_N}" ] || die "coverage_gaps not found in ${REPORT} — refusing to emit a digest that cannot state it"
if [ "${GAPS_N}" -eq 0 ]; then
  GAPS='`[]` — none'
else
  GAPS="**${GAPS_N} gap(s)** — see the report; a digest with gaps does NOT back a closure claim"
fi

# Which gate flag drove the run (for the reconstructed command).
#
# DERIVED FROM THE REPORT, NOT ENUMERATED (EVIDENCE-BANK-DURABILITY.6). This used
# to scan a hardcoded list of fourteen `*_gate` field names. A gate added after the
# script was written was absent from that list, so the deriver emitted a command
# with NO gate flag at all — one that runs the *default* scenario set. A reader
# following it would "re-verify" a completely different run and see numbers that do
# not match, with nothing to indicate why: silent, plausible, and wrong, the worst
# failure an evidence mechanism can have. `tool_matrix` already serializes one
# boolean per gate, so the truth is in the report; read it instead of remembering
# it. Same rule the EVIDENCE-CITATIONS check follows: classify, never guess.
TRUE_GATES="$(true_gates_in "${REPORT}")"
N_TRUE=$(printf '%s' "${TRUE_GATES}" | grep -c . || true)
GATE_FLAG=""
case "${N_TRUE}" in
  0)
    # Legitimate ONLY for the default scenario set (a plain matrix run, or a
    # `--phase1-gate` run which also reports `scenario_set: "default"`). A
    # non-default set with no true flag is a contradiction the deriver must not
    # paper over: it means this script cannot reconstruct the command.
    if [ "${SET}" != "default" ]; then
      die "report says scenario_set='${SET}' but records no '*_gate: true' field.
       The re-runnable command cannot be reconstructed, and emitting one without a
       gate flag would silently describe a DEFAULT-set run. Pass --command '<exact cmd>'
       (and teach tool_matrix to record the flag) rather than banking a wrong pointer."
    fi
    ;;
  1) GATE_FLAG="--$(printf '%s' "${TRUE_GATES}" | tr '_' '-')" ;;
  *)
    die "report records ${N_TRUE} '*_gate: true' fields ($(printf '%s' "${TRUE_GATES}" | tr '\n' ' ')).
       The gates are mutually exclusive, so this report is malformed; refusing to guess."
    ;;
esac
if [ -z "${COMMAND}" ]; then
  # `${GATE_FLAG}` may be empty (a default-set bank); build the argv without a
  # double space, because this string is meant to be copy-pasted by a human.
  COMMAND="cargo run --release --bin tool_matrix --${GATE_FLAG:+ ${GATE_FLAG}} --yosys-mode ${YMODE} --out .cache/anvil-sandbox/${BANK}"
fi

# `commit:` must name the code that PRODUCED these numbers, which is not always
# HEAD (EVIDENCE-BANK-DURABILITY.6). When the working tree is dirty — the normal
# case, because a digest is derived as part of the leaf that adds the gate — the
# numbers came from uncommitted code, and HEAD is the PARENT of the commit that
# will bank them. For a gate introduced by that same commit, HEAD is a revision
# where the gate's flag does not exist, so the digest's own re-verification
# instruction cannot be followed. Hit exactly that on
# `anvil-emit-surface-interaction-r1`, whose pointer had to be corrected by hand.
# So: `--commit` wins; otherwise use HEAD, and when the tree is dirty say loudly
# that the value needs backfilling once the banking commit lands.
if [ -z "${COMMIT}" ]; then
  COMMIT="$(cd "${ROOT}" && git rev-parse --short=12 HEAD)"
  if [ -n "$(cd "${ROOT}" && git status --porcelain 2>/dev/null)" ]; then
    COMMIT_PENDING=1
  fi
fi
DATE="$(date +%F)"   # when the digest was derived; `commit` carries the code identity
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
  printf '\n## Coverage facts lit\n\n'
  facts="$(sed -n 's/^    "\(saw_[a-z0-9_]*\)": true,\{0,1\}$/\1/p' "${REPORT}" | sort -u)"
  if [ -n "${facts}" ]; then printf -- '- `%s`\n' ${facts}
  else printf -- '- (none recorded true in this report)\n'; fi
  printf '\n## Re-verification\n\n'
  printf 'Re-run `command` above at commit `%s`. The numbers here are what a clean\n' "${COMMIT}"
  printf 're-run must reproduce; a divergence is a finding, not a stale digest.\n'
  printf 'Later commits may legitimately produce different numbers — that is why the\n'
  printf 'commit is recorded alongside them.\n'
} > "${OUT}"

printf 'evidence-digest: wrote %s\n' "${OUT#"${ROOT}"/}" >&2
printf 'evidence-digest: REVIEW the derived `command` before committing.\n' >&2
if [ "${COMMIT_PENDING}" -eq 1 ]; then
  printf 'evidence-digest: WARNING — the working tree was DIRTY, so these numbers came from\n' >&2
  printf '                 UNCOMMITTED code and `commit: %s` names its PARENT.\n' "${COMMIT}" >&2
  printf '                 If the code that produced them lands in the commit that banks this\n' >&2
  printf '                 digest (always true for a gate introduced by its own first bank),\n' >&2
  printf '                 that value is WRONG and the re-verification instruction cannot be\n' >&2
  printf '                 followed. Backfill it in a follow-up commit, or re-derive with\n' >&2
  printf '                 --commit <sha> once the hash is known.\n' >&2
fi
