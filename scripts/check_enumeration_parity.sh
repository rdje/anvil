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
    grep -oE '"[a-z0-9_-]+"' | tr -d '"' | sort -u
}

# Authoritative: the steering category names in the `knob_ids!` table in
# src/ir/knob_id.rs — the third column of each row. That table is the ONLY place
# the taxonomy is defined; every prose copy of the list is a shadow of it
# (COVERAGE-STEERED-GENERATION.4c).
#
# ── WHERE THIS HAS POINTED, AND WHY IT MOVED EACH TIME ────────────────────────
# (1) `src/ir/types.rs`, `KnobId::category`'s match arms. That a doctrine check
#     had to name `types.rs` to find a steering taxonomy was the decisive
#     evidence FOR splitting that file (IR-TYPES-DECOMPOSITION).
# (2) `src/ir/knob_id.rs`, the same match arms, at IR-TYPES-DECOMPOSITION.2.
# (3) Here: the `knob_ids!` table, at COVERAGE-STEERED-GENERATION.6, which made
#     the enum + all() + name() + category() all EXPAND FROM that table (rung R1
#     of decision 0033 — retire the duplicated list rather than gate it).
#
# Repoint (3) was LOAD-BEARING, and not for the reason first assumed. The guess
# was that an extractor left on `pub fn category` would read ZERO (its body now
# holds `$category`, not literals) and trip the floor loudly. MEASURED, it reads
# the correct 8 — and that is worse. Two accidents stack:
#
#   (a) its range terminator `/^    }$/` no longer exists where it used to. The
#       macro definition closes with `    };` (semicolon) and the invocation with
#       `}` at column 0, so the range matches NEITHER and OVER-RUNS 162 lines,
#       swallowing the whole table on its way to `category_of_name`'s brace;
#   (b) `grep -oE '"[a-z]+"'` over that over-run then skips every knob NAME only
#       because each one happens to contain `_`, and no category does.
#
# So it returns the right answer for the wrong reason, and the coincidence is one
# row deep: a knob named `"probe"` makes it emit a PHANTOM category, which fails
# at every doc site for something that does not exist — the cry-wolf failure this
# file says elsewhere gets a gate deleted. Verified by probe at `.6`.
#
# The transferable rule: **a `sed` line-range whose terminator stops existing does
# not fail — it runs on and returns something plausible.** Anchor an extraction to
# a shape that cannot silently widen, and re-measure it whenever its target moves.
#
# ── WHY THE CATEGORY CAPTURE IS `[a-z0-9_]+` (PARITY-EXTRACTOR-CHARSET-GAP.1) ──
# It was `[a-z]+` until `2026-08-01`, and note what point (b) above says: the old
# extractor worked "only because each [knob name] happens to contain `_`, and no
# category does." That sentence records an accident being relied upon — and the
# rewrite fixed the over-run (a) while KEEPING the charset (b), which turned the
# noticed accident into a live silent filter.
#
# Measured at `CAPABILITY-BREADTH-EXPANSION.4b.1`: a new steering category named
# `case_qualifier` appears in **none** of the seven fenced doc sites, and this
# check reported **ok on all seven**. Not mis-read — never read, so it never
# entered the authoritative set and every assertion about it passed vacuously.
#
# **Why the count floor cannot cover this, and the general rule.** The floor is
# *shrink-coupled, not growth-coupled* (see its own note below). A charset gap
# that hides a **renamed** id shows up as a shrink and the floor trips — which is
# exactly what the adapter extractor did when probed (5 → 4, "produced 4 entries
# (floor 5)"). A charset gap that hides a **new** id is born invisible: the other
# eight still extract, the floor is satisfied, and nothing fires. Since ids are
# added far more often than renamed, **the case a floor cannot see is the common
# one.**
#
# So the rule for any extractor here: **capture the charset the SOURCE permits,
# not the charset its current members happen to use.** The name column on this
# very row already allowed `_`; the asymmetry was accidental. The adapter capture
# below is `[a-z0-9_-]+` for the same reason — no current adapter id carries a
# separator, but `iverilog-compile` is already the tool-column spelling elsewhere
# in this codebase, so the shape is not hypothetical.
#
# Both are held by a control, per DOCTRINE_ENFORCEMENT.md §9: temporarily give a
# category an `_` (or an adapter id a `-`) and this check must go RED naming it.
# Before the widening the first probe printed `ok` seven times; after it, seven
# FAILs. That difference is the whole assertion.
#
# ── WHY THIS MATCHES A WHOLE ROW (PARITY-EXTRACTOR-ARM-SHAPE-GAP.1) ───────────
# It was once `grep -oE '=> "[a-z]+"'` over the match arms, and that silently
# read **7 of 8** categories for as long as it existed: `datapath` was invisible.
# Not because the taxonomy was wrong, but because that arm's pattern is three
# `|`-joined variants, which overflows the line width, so `rustfmt` rendered it
# as a block:
#
#     KnobId::CoefficientProb | KnobId::ConstShiftAmountProb | ... => {
#         "datapath"
#     }
#
# The `=>` and the string ended up on different lines. **The regex encoded a
# source FORMATTING assumption, not a source FACT** — and nobody wrote that
# formatting; `rustfmt` chose it because the pattern got long. So the trigger was
# "a category gains enough knobs to wrap", which biased the extractor against
# exactly the categories that grow.
#
# The floor could not catch it: a floor catches "matched nothing", not "matched
# most" (7 >= 6). And `covers_set` is per-category, so a category this never
# produces is verified at NO doc site — the gate silently exempted one eighth of
# the taxonomy. (Measured at the time of the fix: all four sites did name
# `datapath`, so the docs were fine and only the gate was blind.)
#
# The table removes the hazard at its root rather than working around it:
# `rustfmt` does not format the body of this macro invocation at all, so the row
# layout is a source fact chosen by the author. Measured, not assumed — the
# longest row is 113 characters and survives `cargo fmt` untouched, well past
# rustfmt's 100-column max_width, which is precisely what it would have wrapped
# if it owned this text.
#
# The match is therefore the WHOLE row (`Ident => "name", "category";`), not a
# loose scan for quoted lowercase words. Two consequences, both wanted: a
# reshaped row produces nothing and trips the floor loudly, and a comment or a
# doc line cannot inject a phantom category, because neither can start with an
# identifier followed by `=>` — this check must fail loud, never cry wolf.
extract_steering_categories() {
  sed -n '/^knob_ids! {$/,/^}$/p' src/ir/knob_id.rs |
    sed -nE 's/^[[:space:]]*[A-Za-z0-9_]+[[:space:]]*=>[[:space:]]*"[a-z0-9_]+",[[:space:]]*"([a-z0-9_]+)";[[:space:]]*$/\1/p' |
    sort -u
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

# --- the fence ---------------------------------------------------------------
#
# A declared documentation site marks its enumeration with an INLINE HTML comment
# pair carrying the SET ID:
#
#     … one of the eight coarse categories — <!--enum:steer-categories-->`state`,
#     `selectors`, … `emission`<!--/enum:steer-categories--> — so one entry can …
#
# ── WHY A FENCE, AND NOT THE WHOLE FILE (decision 0037) ───────────────────────
# This check used to ask "does this file contain each id anywhere?". MEASURED at
# LIVE-DOC-REGISTRY-SHADOWS.2, that predicate was VACUOUS at 3 of its 10 sites:
# deleting the guarded enumeration outright left the check GREEN at
# book/src/api-tools.md, book/src/agent-mcp.md (both pair-3 sites — so the
# adapter pair protected NOTHING) and book/src/knobs.md, because `verilator`,
# `yosys`, `state` and `sharing` are ordinary vocabulary in the very chapters
# that list them. USER_GUIDE.md missed vacuity by one word.
#
# The transferable rule, and the reason the fix is not "declare more sites":
# **a coverage check's strength is inversely proportional to how ordinary its ids
# are as WORDS in the checked document — and they are most ordinary in exactly
# the document that documents them.** Pair 1b survived only on an accident of
# vocabulary (`MEMORY-ARCH`, `README-GROWTH` occur nowhere but their list).
#
# A proximity window (all ids within K lines) was measured and REJECTED: with the
# enumeration deleted the minimal spanning window at BOTH pair-3 sites is still
# 2 lines, and a single-id omission leaves a 6-line window at algorithm.md and 4
# at agent-mcp.md — because the explanatory prose that makes those chapters good
# sits beside the list. A heuristic whose blind spots track documentation QUALITY
# is measuring the wrong thing.
#
# ── WHY THE MARKERS ARE INLINE ────────────────────────────────────────────────
# Measured against the real sites: most enumerations live mid-paragraph, inside a
# Markdown table row, or inside a bulleted list. An HTML comment on its OWN line
# is a CommonMark HTML *block*, which interrupts a paragraph and splits a table
# or list — i.e. it would change the rendered book. Inline, it is raw inline HTML
# and renders as nothing. The fence must be invisible: a gate that forces prose
# to reflow so it stays checkable has inverted the relationship.
#
# ── WHY THE MARKER CARRIES A SET ID ───────────────────────────────────────────
# book/src/agent-mcp.md and CODEBASE_ANALYSIS.md are each declared sites for TWO
# different sets. An anonymous fence could not tell them apart.
#
# ── WHY THE FENCE IS NOT ITSELF A SHADOW (decision 0033 rule (a)) ─────────────
# It names NO members. Growing the set from 8 to 9 never requires touching a
# fence, so test (2) — growth-coupled — fails, and the repair does not introduce
# the very thing the doctrine forbids.

# fenced_region <file> <set-id> — print the text lying strictly BETWEEN the
# markers, across any number of lines. Prints nothing when the fence is absent.
# Character-scoped, not line-scoped: a marker that opens mid-line must not drag
# in the words before it, or the vacuity leaks straight back in.
fenced_region() {
  awk -v id="$2" '
    BEGIN { s = "<!--enum:" id "-->"; e = "<!--/enum:" id "-->"; inside = 0 }
    {
      line = $0
      while (length(line) > 0) {
        if (inside == 0) {
          p = index(line, s)
          if (p == 0) break
          line = substr(line, p + length(s))
          inside = 1
        } else {
          q = index(line, e)
          if (q == 0) { print line; break }
          print substr(line, 1, q - 1)
          line = substr(line, q + length(e))
          inside = 0
        }
      }
      if (inside == 1 && length(line) == 0) print ""
    }
  ' "$1"
}

# covers_fenced_set <pair> <src list> <file> <set-id> — every authoritative id
# must be named INSIDE the site's fence for that set.
#
# A missing fence is a HARD FAILURE, not a skip. That is what stops this repair
# from silently degrading back into whole-file matching the first time somebody
# reflows a chapter — and it is the same reasoning as the extractor count floors
# above: a check that matches nothing must die loudly rather than pass vacuously.
#
# Still ONE-DIRECTIONAL (coverage, not exact parity). See the dated Correction in
# decision 0037: exact parity is not uniformly available, because several fences
# must enclose list items that carry prose (`- MEMORY-ARCH — the durable memory
# invariants …`), so harvesting "the ids this region names" would also harvest
# every other backticked token in that prose and cry wolf. The reverse direction
# is in any case near-empty by policy: nothing is ever retired
# (feedback_never_retire_strategies), so "the doc names an id that no longer
# exists" is not a live failure mode in this repo. The failure that matters, and
# the one this now genuinely catches, is a doc that fell BEHIND the set.
covers_fenced_set() {
  local pair="$1" src="$2" file="$3" set_id="$4" missing="" id region
  if [ ! -f "${file}" ]; then
    note "FAIL: ${pair} — declared documentation site is missing: ${file}"
    fail=1
    return 1
  fi
  region="$(fenced_region "${file}" "${set_id}")"
  if [ -z "$(printf '%s' "${region}" | tr -d '[:space:]')" ]; then
    note "FAIL: ${pair} — ${file} has no <!--enum:${set_id}--> … <!--/enum:${set_id}--> fence"
    note "      (or the fence is empty). A declared site must mark the enumeration"
    note "      this pair checks; without it the check would silently fall back to"
    note "      matching the whole file, which decision 0037 measured as vacuous."
    fail=1
    return 1
  fi
  for id in ${src}; do
    printf '%s\n' "${region}" | grep -q -- "${id}" || missing="${missing} ${id}"
  done
  if [ -n "${missing}" ]; then
    note "FAIL: ${pair} — ${file}'s <!--enum:${set_id}--> fence does not name:${missing}"
    note "      it documents the registry, so its enumeration must name every entry."
    fail=1
    return 1
  fi
  ok "${pair} — ${file} (fenced: ${set_id}) names every entry"
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
  covers_fenced_set 'live-registry list <-> the DOCTRINES registry' \
    "${registry_ids}" 'README.md' 'doctrine-ids'
  covers_fenced_set 'live-registry list <-> the DOCTRINES registry' \
    "${registry_ids}" 'book/src/architecture.md' 'doctrine-ids'
  covers_fenced_set 'live-registry list <-> the DOCTRINES registry' \
    "${registry_ids}" 'docs/knowledge/doctrine-enforcement.md' 'doctrine-ids'
  covers_fenced_set 'live-registry list <-> the DOCTRINES registry' \
    "${registry_ids}" 'CODEBASE_ANALYSIS.md' 'doctrine-ids'
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
#
# MEASURED at LIVE-DOC-REGISTRY-SHADOWS.2: under the old whole-file predicate
# BOTH of these sites passed with the allow-list DELETED — this pair protected
# nothing for its entire life, because `verilator` (12 further occurrences in
# api-tools.md) and `yosys` (18) are the ordinary vocabulary of a chapter about
# running those tools. The fence is what makes the pair real. See decision 0037.
adapter_ids="$(extract_adapter_ids)"
if floor_or_fail 'downstream adapter ids' 5 "${adapter_ids}"; then
  covers_fenced_set 'adapter allow-list <-> the adapter registry' \
    "${adapter_ids}" 'book/src/api-tools.md' 'adapter-ids'
  covers_fenced_set 'adapter allow-list <-> the adapter registry' \
    "${adapter_ids}" 'book/src/agent-mcp.md' 'adapter-ids'
fi

# Pair 4 — the live docs that enumerate the `--steer` category taxonomy (shadow)
# mirror the `knob_ids!` table's category column (authoritative). Added by
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
# The site list grew from four to SEVEN at LIVE-DOC-REGISTRY-SHADOWS.3, and the
# PROCEDURE matters more than the three names: decision 0037 measured that this
# list CANNOT be derived (a threshold selector over tracked *.md fails in both
# directions at once — loose, it selects append-only history whose OLD six-name
# list is correct and may never be retro-edited under decision 0031; tight, it
# drops algorithm.md and knobs.md because their lists wrap across two lines), so
# the list is AUTHORITATIVE under decision 0033 rule (a) test (2), exactly like
# check_no_boot_volume_refs.sh's allow-list and for the same underlying reason.
# What replaced derivation is a written procedure: run the discovery selector
# over the WHOLE TREE from the authoritative set and classify EVERY candidate as
# live doc (declare) or history / incidental prose (do not). These three were
# added that way — agent-mcp.md and api-introspection.md from the repair at .1,
# CODEBASE_ANALYSIS.md:2349 from the sweep itself, which no bug report had
# surfaced. Never add a site because it turned up in a bug report.
if floor_or_fail 'KnobId steering categories' 8 "${steering_categories}"; then
  covers_fenced_set 'steer categories <-> the knob_ids! table' \
    "${steering_categories}" 'book/src/algorithm.md' 'steer-categories'
  covers_fenced_set 'steer categories <-> the knob_ids! table' \
    "${steering_categories}" 'book/src/knobs.md' 'steer-categories'
  covers_fenced_set 'steer categories <-> the knob_ids! table' \
    "${steering_categories}" 'USER_GUIDE.md' 'steer-categories'
  covers_fenced_set 'steer categories <-> the knob_ids! table' \
    "${steering_categories}" 'docs/AGENT_INTROSPECTION_SCHEMA.md' 'steer-categories'
  covers_fenced_set 'steer categories <-> the knob_ids! table' \
    "${steering_categories}" 'book/src/agent-mcp.md' 'steer-categories'
  covers_fenced_set 'steer categories <-> the knob_ids! table' \
    "${steering_categories}" 'book/src/api-introspection.md' 'steer-categories'
  covers_fenced_set 'steer categories <-> the knob_ids! table' \
    "${steering_categories}" 'CODEBASE_ANALYSIS.md' 'steer-categories'
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
