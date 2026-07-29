#!/usr/bin/env bash
#
# scripts/check_no_boot_volume_refs.sh — VOLUME-DATA-LOCALITY.7 / decision 0031.
#
# DOCTRINE: this project neither stores on nor points at the boot volume. Every
# byte ANVIL creates lands on the repository volume (resolved at runtime by
# src/paths.rs), and no tracked file references a boot-volume path as though it
# were live project data.
#
# Owner directive (2026-07-29): "Nothing should reference or point into the
# Macintosh HD, the boot volume ever again."
#
# ARCHETYPE: structural (DOCTRINE_ENFORCEMENT.md §3) — it re-derives the fact
# from the tree itself, so it cannot be faked by a claim in a commit message.
#
# CONTRACT (§4): exit 0 iff the doctrine holds; nonzero + an actionable stderr
# message on breach; deterministic; reads the repo and mutates nothing.
#
# ── WHY THE ALLOW-LIST IS LOAD-BEARING, NOT LAZINESS ────────────────────────
# Two categories keep boot-volume strings ON PURPOSE, and a check that flagged
# them would be pressuring an author to do something the owner has forbidden:
#
#   1. POLICY DOCUMENTS. A doctrine cannot state what it forbids without naming
#      it. Stripping `/tmp` from decision 0031 would make it unreadable — and a
#      real, reverted incident proved the danger: a blanket sweep once rewrote
#      decision 0030's own reverify command from `ls -d /tmp/anvil-*` to
#      `ls -d anvil-*`, i.e. into nonsense.
#      This covers docs/evidence/ for the same reason: the evidence inventory
#      has to say WHY the grandfathered banks are unrecoverable, and that reason
#      IS "they lived under /tmp, which is purged". Strip the string and the
#      record stops explaining itself.
#
#   2. APPEND-ONLY HISTORY. Owner directive (2026-07-29): "NO, no history
#      rewrite, no at all. Keep it raw, keep honest, so that people can follow
#      the whole history." CHANGES.md and DEVELOPMENT_NOTES.md are layer-C/D
#      records — appended and superseded, NEVER retro-edited
#      (MEMORY_ARCHITECTURE.md §3). A swept history is a dishonest history:
#      someone auditing why the evidence banks evaporated must be able to read
#      the entries that cited /tmp exactly as they were written.
#
# Also exempt: .github/workflows/* `$HOME/...` — those run on GitHub's runners,
# where $HOME is the runner's own disk, so they are correct and portable as
# written. And shared machine-wide stores (~/.cargo, ~/.rustup, /opt/homebrew)
# are USED IN PLACE by owner decision — "if something is shared, use the shared
# information, no need to duplicate" — so $HOME is not itself a banned string.
#
# What this check therefore guards is the thing that actually regresses: a NEW
# /tmp, /private/tmp, or /var/folders path appearing in a live document or in
# code.
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT}"

# Paths whose boot-volume strings are deliberate (see the rationale above).
ALLOW_RE='^(CHANGES\.md|DEVELOPMENT_NOTES\.md|MEMORY\.md|KNOWLEDGE_MAP\.md|DOCTRINE_ENFORCEMENT\.md|scripts/check_(no_boot_volume_refs|evidence_citations)\.sh|docs/evidence/|docs/decisions/(0002|0030|0031)-|docs/decisions/INDEX\.md|docs/tasks/(EVIDENCE-BANK-DURABILITY|VOLUME-DATA-LOCALITY|CARGO-TMPDIR-SWEEP-REGRESSION)\.md|docs/TASK_TREE\.md|\.github/workflows/)'

# The banned shapes: OS temp dirs on every platform this repo runs on.
#   /tmp/…            Unix temp
#   /private/tmp/…    macOS's real /tmp
#   /var/folders/…    macOS per-user TMPDIR (NOT caught by grepping "/tmp"!)
#
# ── WHY THE LEADING ANCHOR ──────────────────────────────────────────────────
# A boot-volume path is ABSOLUTE. The bare substring `/tmp/` also occurs in
# the middle of repo-RELATIVE paths that are perfectly on-volume — above all
# `target/tmp/…`, Cargo's CARGO_TARGET_TMPDIR, where every integration test
# puts its scratch. That directory is inside the checkout, hence on the
# project volume, hence exactly what decision 0031 asks for.
#
# Without the anchor the check cannot tell the two apart, and that
# imprecision has already cost us once: VOLUME-DATA-LOCALITY.5's sweep
# rewrote `/tmp/` -> `.cache/anvil-sandbox/` inside `target/tmp/…`, minting
# 10 live-doc paths to a directory that has never existed
# (CARGO-TMPDIR-SWEEP-REGRESSION). An unanchored guard could neither catch
# that nor accept the repair.
#
# So a banned shape counts only where a path can START: at the beginning of
# the line, or after a character that cannot continue a path. `[A-Za-z0-9_.-]`
# are the continuation characters; `~` is deliberately NOT among them, so
# `~/tmp/…` (the home directory, on the boot volume) still counts.
#
# This narrows FALSE POSITIVES ONLY. Every absolute boot-volume path is still
# a breach: `/tmp/x`, `"/tmp/x"`, `` `/tmp/x` ``, `TMPDIR=/tmp/x`, `cd /tmp/x`,
# `~/tmp/x`, `/private/tmp/x`, `/var/folders/x`.
BANNED_RE='(^|[^A-Za-z0-9_.-])(/tmp/|/private/tmp|/var/folders)'

fail=0
note() { printf 'boot-volume-refs: %s\n' "$1" >&2; }

# Only tracked files — untracked scratch is the author's business, and .cache/
# is gitignored by design.
mapfile -t offenders < <(
  git grep -lE "${BANNED_RE}" -- . 2>/dev/null | grep -vE "${ALLOW_RE}" || true
)

if [[ ${#offenders[@]} -gt 0 ]]; then
  fail=1
  note "tracked files reference a boot-volume path (decision 0031 forbids this):"
  for f in "${offenders[@]}"; do
    n=$(git grep -cE "${BANNED_RE}" -- "${f}" 2>/dev/null | cut -d: -f2)
    note "  ${f}  (${n} occurrence(s))"
    git grep -nE "${BANNED_RE}" -- "${f}" 2>/dev/null | head -3 | sed 's/^/boot-volume-refs:      /' >&2
  done
  note ""
  note "FIX: point it at the repository volume instead."
  note "  · a path ANVIL writes  -> resolve through crate::paths::sandbox_root()"
  note "  · a scratch file in a doc recipe -> .cache/anvil-sandbox/<name>"
  note "  · a dead evidence bank -> drop the prefix, keep the bank NAME"
  note "  · genuinely a policy/history doc? add it to ALLOW_RE, with a reason."
fi

# Second leg: the OS temp dir must be named in exactly ONE place in the code —
# the resolver. This is what makes the rule enforceable by inspection instead of
# by everyone remembering it (feedback_full_factorization).
mapfile -t temp_dir_users < <(
  git grep -l 'temp_dir()' -- 'src/*' 'tests/*' 2>/dev/null | grep -v '^src/paths\.rs$' || true
)
if [[ ${#temp_dir_users[@]} -gt 0 ]]; then
  fail=1
  note "std::env::temp_dir() is called outside src/paths.rs:"
  for f in "${temp_dir_users[@]}"; do note "  ${f}"; done
  note "FIX: call crate::paths::sandbox_root() (anvil::paths:: from bins/tests)."
  note "     src/paths.rs is the ONLY place allowed to name the OS temp dir."
fi

if [[ "${fail}" -eq 0 ]]; then
  echo "[boot-volume-refs] ok: no tracked file points at the boot volume; temp_dir() confined to src/paths.rs"
fi
exit "${fail}"
