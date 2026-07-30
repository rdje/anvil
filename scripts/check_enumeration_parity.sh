#!/usr/bin/env bash
#
# scripts/check_enumeration_parity.sh — the ENUMERATION-PARITY doctrine.
#
# DOCTRINE_ENFORCEMENT.md §3 (structural) / §4 (contract). Mechanizes decision
# 0033 (SHADOW-ENUMERATION-SWEEP) for the sites that have NEITHER a compiler NOR
# `cargo test`: a hand-maintained list in a doc or a script that mirrors a set
# which is already authoritative somewhere else, and therefore falls silently out
# of date when that set grows.
#
# Decision 0033 rule (a): a list L is a SHADOW iff all three hold —
#   (1) derivable      — a set S in the repo already enumerates the same
#                        membership and is reachable by ordinary script means;
#   (2) growth-coupled — S grows, and every growth REQUIRES a matching entry in L;
#   (3) silent         — the omission produces no compile error, no failing test,
#                        no runtime error.
# Rust-side pairs are held by in-crate #[test]s (already gated by `cargo test` per
# COMMIT.md + CI); a shell doctrine for them would be a second mechanism for one
# job (feedback_full_factorization). This check holds ONLY the docs/script pairs.
#
# It is deliberately NOT a shadow DETECTOR. Decision 0033 (c) records that as an
# honest limit: test (1) is a semantic relation between two sets, so a syntactic
# detector would have to already know the pairing — the exact human judgement the
# rule encodes — and its only failure modes are MISS (false confidence) and CRY
# WOLF (and a gate that cries wolf gets deleted, taking its real coverage with
# it). So this check holds CLASSIFIED PAIRS, declared in the PAIRS table below.
#
# The PAIRS table is itself AUTHORITATIVE under rule (a), not a shadow: no set in
# the repo enumerates "which lists shadow which sets" (test (1) fails), so the
# mechanism does not recurse. It lives in this script rather than in a separate
# data file for the same reason the DOCTRINES registry lives in the driver — one
# mechanism, no schema + parser + "does the data file exist" meta-check.
#
# Every extraction is COUNT-FLOORED. An extractor that silently matches nothing
# would make this gate pass vacuously, which is the exact failure mode the tree
# exists to remove (and EVIDENCE-BANK-DURABILITY.5's deriver already demonstrated
# once: a broken extractor reports something plausible rather than dying).
#
# Reads the repository, mutates nothing. Deterministic: no clocks, no network, no
# randomness. Not scope-aware — parity is a property of the tree, not of a change,
# so it is checked on every commit. POSIX/bash-3.2 compatible (no mapfile, no
# associative arrays).
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT}"

fail=0
note() { printf '[enumeration-parity] %s\n' "$1" >&2; }
ok() { printf '[enumeration-parity] ok: %s\n' "$1"; }

# --- extractors -------------------------------------------------------------
#
# Each prints one token per line, sorted and de-duplicated. Each is paired with a
# floor in the PAIRS table; a result below the floor is a BROKEN EXTRACTOR, which
# is reported as a breach rather than silently passing.

# Authoritative: the doctrine ids in the driver's own DOCTRINES registry array.
extract_doctrine_registry_ids() {
  grep -oE '^  "[A-Z][A-Z0-9-]*\|' scripts/check_doctrines.sh | tr -d ' "|' | sort -u
}

# Shadow: the first column of DOCTRINE_ENFORCEMENT.md §10's live-instance table.
extract_doctrine_table_ids() {
  grep -oE '^\| `[A-Z][A-Z0-9-]*` \|' DOCTRINE_ENFORCEMENT.md | tr -d '|` ' | sort -u
}

# Authoritative: the mdBook chapters that actually exist on disk.
extract_book_chapter_files() {
  ls book/src/*.md | xargs -n1 basename | grep -vx 'SUMMARY.md' | sort -u
}

# Shadow: the chapter targets linked from book/src/SUMMARY.md's table of contents.
extract_book_summary_links() {
  grep -oE '\]\([^)]+\.md\)' book/src/SUMMARY.md | sed -E 's/^\]\(//; s/\)$//' | sort -u
}

# Authoritative: the downstream adapter ids — every `Adapter::id()` implementation
# in src/downstream/mod.rs. The trait's own declaration ends in `;` rather than
# `{`, so only the impls match. Registry membership is held on the Rust side by
# `adapter_registry_lists_the_originals_then_new_adapters`; this is the shell-side
# read of the same ids for the docs that quote them.
extract_adapter_ids() {
  grep -A1 "fn id(&self) -> &'static str {" src/downstream/mod.rs |
    grep -oE '"[a-z0-9]+"' | tr -d '"' | sort -u
}

# Authoritative: the steering category names in `KnobId::category`'s exhaustive
# match. The match arms are the ONLY place the taxonomy is defined; every prose
# copy of the list is a shadow of this (COVERAGE-STEERED-GENERATION.4c).
#
# The path moved from src/ir/types.rs to src/ir/knob_id.rs at
# IR-TYPES-DECOMPOSITION.2. That this extractor had to name `types.rs` at all was
# the decisive evidence FOR that split: a doctrine check should not have to know
# that a file called "types" houses a steering taxonomy. The count floor below is
# what makes repointing it safe — a wrong path yields 0 categories, which trips
# `floor_or_fail` loudly instead of passing vacuously.
#
# ── WHY THIS DOES NOT MATCH ON `=>` (PARITY-EXTRACTOR-ARM-SHAPE-GAP.1) ────────
# It used to be `grep -oE '=> "[a-z]+"'`, and that silently read **7 of 8**
# categories for as long as it existed: `datapath` was invisible. Not because the
# taxonomy was wrong, but because that arm's pattern is three `|`-joined variants,
# which overflows the line width, so `rustfmt` renders it as a block:
#
#     KnobId::CoefficientProb | KnobId::ConstShiftAmountProb | ... => {
#         "datapath"
#     }
#
# The `=>` and the string end up on different lines. **The regex encoded a source
# FORMATTING assumption, not a source FACT** — and nobody wrote that formatting;
# `rustfmt` chose it because the pattern got long. So the trigger was "a category
# gains enough knobs to wrap", which biases the extractor against exactly the
# categories that grow.
#
# The floor could not catch it: a floor catches "matched nothing", not "matched
# most" (7 >= 6). And `covers_set` is per-category, so a category this never
# produces is verified at NO doc site — the gate silently exempted one eighth of
# the taxonomy. (Measured at the time of the fix: all four sites did name
# `datapath`, so the docs were fine and only the gate was blind.)
#
# The fix is format-independent by construction: `category()` returns
# `&'static str` and every arm's value is a bare string literal, so **the set of
# string literals in that function body IS the taxonomy**, however rustfmt chooses
# to lay the arms out. Comment lines are dropped first so a future `// "note"`
# cannot inject a phantom category — this check must fail loud, never cry wolf.
# Deliberately NOT "widen the regex to also accept the block form": that fixes
# this instance and leaves the class, and the next arm rustfmt reshapes breaks it
# again.
extract_steering_categories() {
  sed -n '/pub fn category(&self)/,/^    }$/p' src/ir/knob_id.rs |
    grep -v '^[[:space:]]*//' |
    grep -oE '"[a-z]+"' | tr -d '"' | sort -u
}

# --- helpers ----------------------------------------------------------------

count_of() { printf '%s\n' "$1" | grep -c . ; }

# floor_or_fail <label> <min> <list>  — a broken extractor is a breach, not a pass.
floor_or_fail() {
  local label="$1" min="$2" list="$3" n
  n="$(count_of "${list}")"
  if [ "${n}" -lt "${min}" ]; then
    note "FAIL: extractor '${label}' produced ${n} entries (floor ${min}) — the"
    note "      extractor is broken, not the enumeration. Fix the extractor in"
    note "      scripts/check_enumeration_parity.sh; do NOT lower the floor."
    fail=1
    return 1
  fi
  return 0
}

# equal_sets <pair> <src label> <src list> <shadow label> <shadow list>
# Exact parity in BOTH directions: a missing entry means the shadow fell behind
# the set; an extra entry means it names something that no longer exists.
equal_sets() {
  local pair="$1" src_label="$2" src="$3" shadow_label="$4" shadow="$5" missing extra
  missing="$(comm -23 <(printf '%s\n' "${src}") <(printf '%s\n' "${shadow}") | grep . || true)"
  extra="$(comm -13 <(printf '%s\n' "${src}") <(printf '%s\n' "${shadow}") | grep . || true)"
  if [ -n "${missing}" ] || [ -n "${extra}" ]; then
    note "FAIL: ${pair}"
    [ -n "${missing}" ] && note "      in ${src_label} but MISSING from ${shadow_label}: $(printf '%s' "${missing}" | tr '\n' ' ')"
    [ -n "${extra}" ] && note "      in ${shadow_label} but NOT in ${src_label}: $(printf '%s' "${extra}" | tr '\n' ' ')"
    fail=1
    return 1
  fi
  ok "${pair} ($(count_of "${src}") entries, exact parity)"
  return 0
}

# covers_set <pair> <src list> <file>  — every authoritative id must be named in
# the declared documentation site. One-directional by design: a chapter may
# legitimately name a subset in an example, and nothing is ever retired
# (feedback_never_retire_strategies), so the failure that matters is a doc that
# fell BEHIND the set.
covers_set() {
  local pair="$1" src="$2" file="$3" missing="" id
  if [ ! -f "${file}" ]; then
    note "FAIL: ${pair} — declared documentation site is missing: ${file}"
    fail=1
    return 1
  fi
  for id in ${src}; do
    grep -q -- "${id}" "${file}" || missing="${missing} ${id}"
  done
  if [ -n "${missing}" ]; then
    note "FAIL: ${pair} — ${file} does not name:${missing}"
    note "      it documents the allow-list, so it must name every registered entry."
    fail=1
    return 1
  fi
  ok "${pair} — ${file} names every entry"
  return 0
}

# --- PAIRS: the declared (shadow site, authoritative source, extractor) table ---
#
# Each pair below is a classified instance of decision 0033 rule (a). To add one:
# write the two extractors, give the extraction a floor, and add a block here.

# Pair 1 — DOCTRINE_ENFORCEMENT.md §10's table (shadow) mirrors the driver's
# DOCTRINES registry (authoritative). A documented-but-unregistered doctrine is
# the §11 "trust me" anti-pattern; a registered-but-undocumented one is invisible
# to the reader the standard is written for.
registry_ids="$(extract_doctrine_registry_ids)"
table_ids="$(extract_doctrine_table_ids)"
if floor_or_fail 'DOCTRINES registry ids' 5 "${registry_ids}" &&
  floor_or_fail 'DOCTRINE_ENFORCEMENT.md §10 table ids' 5 "${table_ids}"; then
  equal_sets 'DOCTRINE_ENFORCEMENT.md §10 table <-> the DOCTRINES registry' \
    'the DOCTRINES registry' "${registry_ids}" \
    'DOCTRINE_ENFORCEMENT.md §10' "${table_ids}"
fi

# Pair 1b — the four live-doc sites that publish the SAME registry as a "live
# registry" list (shadow). Found by this doctrine's own first run, and THREE OF
# THEM WERE ALREADY STALE: book/src/architecture.md, the
# docs/knowledge/doctrine-enforcement.md fact card, and CODEBASE_ANALYSIS.md all
# advertised a FOUR-doctrine registry while six were enforced. The book is the
# owner's only window into the project, a Knowledge Map card is what a future
# agent reads INSTEAD of re-deriving, and CODEBASE_ANALYSIS.md is required by
# COMMIT.md to reflect the code as it now is — so a stale one does not merely
# omit, it misinforms. Sites are named here (not discovered) for the same reason
# as pair 3.
if floor_or_fail 'DOCTRINES registry ids (live-registry sites)' 5 "${registry_ids}"; then
  covers_set 'live-registry list <-> the DOCTRINES registry' \
    "${registry_ids}" 'README.md'
  covers_set 'live-registry list <-> the DOCTRINES registry' \
    "${registry_ids}" 'book/src/architecture.md'
  covers_set 'live-registry list <-> the DOCTRINES registry' \
    "${registry_ids}" 'docs/knowledge/doctrine-enforcement.md'
  covers_set 'live-registry list <-> the DOCTRINES registry' \
    "${registry_ids}" 'CODEBASE_ANALYSIS.md'
fi

# Pair 2 — book/src/SUMMARY.md (shadow) mirrors book/src/*.md (authoritative).
# mdBook renders only what SUMMARY.md links, so an unlinked chapter is written and
# NEVER RENDERED — and the book is the owner's only window into the project.
chapter_files="$(extract_book_chapter_files)"
summary_links="$(extract_book_summary_links)"
if floor_or_fail 'book/src/*.md chapters' 20 "${chapter_files}" &&
  floor_or_fail 'book/src/SUMMARY.md links' 20 "${summary_links}"; then
  equal_sets 'book/src/SUMMARY.md <-> book/src/*.md' \
    'book/src/*.md' "${chapter_files}" \
    'book/src/SUMMARY.md' "${summary_links}"
fi

# Pair 3 — the two book chapters that document the downstream-tool allow-list
# (shadow) mirror the adapter registry (authoritative). Added by
# SHADOW-ENUMERATION-SWEEP.6, which derived every Rust-side copy of this list from
# the registry and left these two as the only copies outside it. The sites are
# NAMED here rather than discovered, because "any chapter mentioning two adapter
# ids" also matches an ordinary `--tools verilator,yosys` example — and a gate
# that cries wolf gets deleted.
adapter_ids="$(extract_adapter_ids)"
if floor_or_fail 'downstream adapter ids' 5 "${adapter_ids}"; then
  covers_set 'adapter allow-list <-> the adapter registry' \
    "${adapter_ids}" 'book/src/api-tools.md'
  covers_set 'adapter allow-list <-> the adapter registry' \
    "${adapter_ids}" 'book/src/agent-mcp.md'
fi

# Pair 4 — the live docs that enumerate the `--steer` category taxonomy (shadow)
# mirror `KnobId::category`'s match arms (authoritative). Added by
# COVERAGE-STEERED-GENERATION.4c, which added two categories (`motifs`,
# `emission`) and found the six-name list copied into five live docs plus the
# book. A stale copy here is worse than an omission: `--steer` *errors* on an
# unknown key, so a user reading a short list simply never learns the category
# exists — the feature is delivered and invisible. The sites are NAMED rather
# than discovered, because a grep for "any file mentioning two category words"
# also matches ordinary prose about state and sharing.
#
# README.md was a fifth site until README-POLICY-ADOPTION.2 (decision 0036)
# deleted the `--steer` bullet along with the rest of `## Current CLI truth`.
# It is dropped from this pair rather than re-added to the README: under
# README_POLICY.md the landing page does not enumerate a knob taxonomy, and a
# list kept alive solely to satisfy a doctrine site would grow by one line per
# future category — the exact growth-coupling that made that file 1771 lines.
# This is decision 0033's own preferred rung: repair a shadow by DELETING it
# (R1), not by gating it forever. The four surviving sites are the canonical
# homes the policy routes this content to.
steering_categories="$(extract_steering_categories)"
# Floor re-derived from the measured count at PARITY-EXTRACTOR-ARM-SHAPE-GAP.1
# (8: state, selectors, datapath, terminals, sharing, hierarchy, motifs,
# emission). Raised from 6, which was loose enough to sit under the broken
# extractor's 7 and so never fired.
#
# A floor is NOT a shadow of the taxonomy, by decision 0033 rule (a): it is
# **shrink-coupled, not growth-coupled**. Adding a 9th category never requires
# touching this number — test (2) fails, so the three-part test fails. Only
# *removing* a category would, and that is exactly the event worth stopping to
# look at. That asymmetry is why a floor can be tightened to the real count
# without becoming the hand-maintained list the doctrine forbids.
if floor_or_fail 'KnobId steering categories' 8 "${steering_categories}"; then
  covers_set 'steer categories <-> KnobId::category' \
    "${steering_categories}" 'book/src/algorithm.md'
  covers_set 'steer categories <-> KnobId::category' \
    "${steering_categories}" 'book/src/knobs.md'
  covers_set 'steer categories <-> KnobId::category' \
    "${steering_categories}" 'USER_GUIDE.md'
  covers_set 'steer categories <-> KnobId::category' \
    "${steering_categories}" 'docs/AGENT_INTROSPECTION_SCHEMA.md'
fi

# --- verdict ----------------------------------------------------------------
if [ "${fail}" -ne 0 ]; then
  note "one or more declared enumeration pairs are out of parity — commit blocked."
  note "See decision 0033 (docs/decisions/0033-shadow-enumeration-classification.md)"
  note "and docs/tasks/SHADOW-ENUMERATION-SWEEP.md. Fix the SHADOW to match the"
  note "authoritative set; never fix the set to match the shadow."
  exit 1
fi

ok "all declared enumeration pairs are in parity"
exit 0
