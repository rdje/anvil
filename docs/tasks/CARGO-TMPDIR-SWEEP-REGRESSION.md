# CARGO-TMPDIR-SWEEP-REGRESSION: the boot-volume sweep mangled `target/tmp`

## Metadata

- Tree ID: `CARGO-TMPDIR-SWEEP-REGRESSION`
- Status: `done`
- Roadmap lane: Quality / live-doc + doctrine-check precision (no phase reopened)
- Created: `2026-07-29`
- Last updated: `2026-07-29`
- Owner: repo-local workflow

## Goal

Repair a regression introduced by `VOLUME-DATA-LOCALITY.5` (`bdb6b0a`), and
close the doctrine-check gap that would otherwise block the repair.

`VOLUME-DATA-LOCALITY.5`'s Rule B rewrote the substring `/tmp/` to
`.cache/anvil-sandbox/`. That is correct for an **absolute** boot-volume path
(`/tmp/foo` → `.cache/anvil-sandbox/foo`) but wrong when `/tmp/` is a *segment
of a longer relative path*. Ten live-doc citations of `target/tmp/<name>` —
Cargo's `CARGO_TARGET_TMPDIR`, which lives **inside the repository** and is
therefore already on the project volume — became the non-existent
`target.cache/anvil-sandbox/<name>`.

Restoring the correct text is blocked by the `NO-BOOT-VOLUME-REFS` doctrine
check itself: its `BANNED_RE='/tmp/|/private/tmp|/var/folders'` matches
`/tmp/` at *any* position, so it cannot distinguish the absolute boot-volume
path it exists to forbid from a repo-relative path that merely contains the
same three characters. The check must be made precise before the docs can be
made correct.

## Observation that opened this tree (`2026-07-29`, session bootstrap)

```
$ git show 4bb1822:README.md | sed -n '1662p'
  `target/tmp/frontend-parity-signoff-verilator-json`.
$ sed -n '1662p' README.md
  `target.cache/anvil-sandbox/frontend-parity-signoff-verilator-json`.
$ sed -n '1220,1221p' tests/frontend_parity.rs
    let dir = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("frontend-parity-signoff-verilator-json");
```

`grep -rnE '[A-Za-z0-9_]\.cache/anvil-sandbox'` over the tracked tree →
**10 occurrences**, all of the same shape, across 8 files:

| File | Line | Correct value (pre-sweep, `4bb1822`) |
| --- | --- | --- |
| `README.md` | 1662 | `target/tmp/frontend-parity-signoff-verilator-json` |
| `ROADMAP.md` | 2439 | `target/tmp/frontend-parity-signoff-verilator-json` |
| `USER_GUIDE.md` | 2031 | `target/tmp/frontend-parity-signoff-verilator-json` |
| `CODEBASE_ANALYSIS.md` | 2109 | `target/tmp/frontend-parity-signoff-verilator-json` |
| `book/src/ir.md` | 621 | `target/tmp/frontend-parity-signoff-verilator-json` |
| `docs/tasks/SIGNOFF-SURFACE-EXPANSION.md` | 130 | `target/tmp/frontend-parity-signoff-verilator-json` |
| `docs/tasks/PHASE-8-FRONTEND-ACCEPT.md` | 101 | `target/tmp/frontend-parity-phase8-yosys` |
| `docs/tasks/PHASE-7-ORACLE-MICRODESIGN.md` | 110, 252, 622 | `target/tmp/microdesign-parity-phase7-yosys` |

**This is not a claim that `VOLUME-DATA-LOCALITY` was wrong.** Its 662
rewrites moved ANVIL's real off-volume writes onto the project volume, and
the residue census 14 → 0 stands. The defect is narrow: one substring rule
was applied without an absolute-path anchor, and no check could catch the
result because the guard shares the same imprecision.

It is also a second instance of the hazard `MEMORY.md` already records —
*"never mass-rewrite strings across docs whose subject is that string"* — in
its sibling form: **never mass-rewrite a path prefix without anchoring it to
the start of a path.**

## Non-Goals

- **Not** re-opening `VOLUME-DATA-LOCALITY`. That tree is closed and its
  conclusions hold; this tree owns one narrow regression it left behind.
- **Not** rewriting history. `CHANGES.md` / `DEVELOPMENT_NOTES.md` stay raw
  (decision `0031`); neither contains the mangled form.
- **Not** relaxing the boot-volume doctrine. `.1` makes the check *more*
  precise, not more permissive: every absolute `/tmp/`, `/private/tmp`,
  and `/var/folders` path is still a breach.

## Acceptance Criteria

- `NO-BOOT-VOLUME-REFS` distinguishes an absolute boot-volume path from a
  repo-relative path containing the same characters, proven by negative
  controls in **both** directions.
- All 10 mangled citations name the directory the harness actually writes.
- `scripts/check_doctrines.sh` 5/5 PASS; `mdbook build book` clean.

## Task Tree

- ID: `CARGO-TMPDIR-SWEEP-REGRESSION`
  Status: `done`
  Goal: repair the sweep regression and the check gap that hid it.
  Children: `CARGO-TMPDIR-SWEEP-REGRESSION.1`, `CARGO-TMPDIR-SWEEP-REGRESSION.2`

- ID: `CARGO-TMPDIR-SWEEP-REGRESSION.1`
  Status: `done`
  Goal: anchor `NO-BOOT-VOLUME-REFS` to absolute paths — a banned shape
        counts only at a path start (line start, or a character that cannot
        continue a path), so `target/tmp/x` is not a boot-volume reference
        while `/tmp/x`, `~/tmp/x`, and `/var/folders/x` still are.
  Acceptance: driver 5/5 PASS on the unchanged tree; a synthetic absolute
        `/tmp/…` citation FAILs; a synthetic `target/tmp/…` citation PASSes;
        the set of files the check currently flags is unchanged (0 before,
        0 after) so the fix removes a false-positive class and nothing else.
  Verification: `BANNED_RE` → `(^|[^A-Za-z0-9_.-])(/tmp/|/private/tmp|/var/folders)`.
        Nine negative controls, each injected into `README.md` then reverted —
        must-FAIL `/tmp/anvil-negctl`, `` `/tmp/anvil-negctl` ``,
        `TMPDIR=/tmp/anvil-negctl`, `~/tmp/anvil-negctl`,
        `/private/tmp/anvil-negctl`, `/var/folders/ab/cd` → **all exit 1**;
        must-PASS `target/tmp/anvil-negctl`, `./tmp/anvil-negctl`,
        `../tmp/anvil-negctl` → **all exit 0**. Narrowing proof: the offender
        set under the old vs new regex is byte-identical on the current tree
        (0 files each), and repo-wide — allow-listed files included — no
        tracked file changes verdict, so nothing previously caught is now
        tolerated. `scripts/check_doctrines.sh` **5/5 PASS**.
  Commit: `CARGO-TMPDIR-SWEEP-REGRESSION.1 — anchor the boot-volume check to absolute paths`

- ID: `CARGO-TMPDIR-SWEEP-REGRESSION.2`
  Status: `done`
  Goal: restore the 10 mangled citations to the directory the harness writes,
        naming `CARGO_TARGET_TMPDIR` in the user-facing docs so a reader who
        overrides `CARGO_TARGET_DIR` is not misled.
  Acceptance: `grep -rnE '[A-Za-z0-9_]\.cache/anvil-sandbox'` → 0 tracked
        citations; driver 5/5; `mdbook build book` clean.
  Verification: All 10 restored across 8 files. **Restored, not re-authored:**
        per file, the extracted `target/tmp/…` token set is byte-identical to
        its pre-sweep text at `4bb1822` (8/8 MATCH), so the edit is provably
        an inversion of the sweep rather than a fresh guess. The three
        task-tree rows are layer-B verification history and were restored
        verbatim; `README.md` / `USER_GUIDE.md` / `book/src/ir.md`
        additionally name `CARGO_TARGET_TMPDIR`. Residue census:
        `git grep -nE '[A-Za-z0-9_]\.cache/anvil-sandbox'` → **0 citations**
        (6 occurrences remain on purpose in the five records that *describe*
        the defect — a record of a wrong string must quote it, the same
        principle that keeps `/tmp` in the policy documents).
        `mdbook build book` exit 0; `cargo test --test book_examples`
        **3/3** in 87.06s under `ram_guard --threshold 90`;
        `scripts/check_doctrines.sh` **5/5 PASS**. No `src/`/`tests/`
        touched ⇒ DUT byte-identical.
  Commit: `CARGO-TMPDIR-SWEEP-REGRESSION.2 — repair the 10 mangled target/tmp citations`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `CARGO-TMPDIR-SWEEP-REGRESSION.1` | `done` | The check must accept the correct text before `.2` can write it; landing `.2` first would need `--no-verify`, which defeats the gate. |
| 2 | `CARGO-TMPDIR-SWEEP-REGRESSION.2` | `done` | The doc repair, once the gate held it in place. |

**No frontier — tree CLOSED `2026-07-29`.** Both leaves done: the guard states the actual rule, and every citation names a directory that exists.

## Decisions

- `2026-07-29`: Opened as its own tree rather than re-opening the closed
  `VOLUME-DATA-LOCALITY` — the `HIERARCHY-DEDUP-PRUNE` / `LIVE-DOC-DRIFT-FIX`
  precedent for a post-closure corrective follow-up.
- `2026-07-29`: `.1` before `.2`, against the usual docs-last cadence,
  because the doctrine gate is what makes `.2` landable at all.
- `2026-07-29`: The fix is an **anchor**, not an allow-list entry. Adding
  `target/tmp` to `ALLOW_RE` would exempt whole *files* from the doctrine
  and would have to be repeated for every future repo-relative path that
  happens to contain `/tmp/`. Anchoring states the actual rule once.
- `2026-07-29`: **This tree file is allow-listed** — it must quote the banned
  strings to describe the defect, so `NO-BOOT-VOLUME-REFS`'s documented
  policy-document category (a doctrine cannot state what it prohibits
  without naming it) applies, exactly as it already does for
  `docs/tasks/VOLUME-DATA-LOCALITY.md`. The gate discovered this itself: a
  manual run passed while the file was untracked (`git grep` sees only
  tracked files), then the pre-commit hook staged it and failed. That is
  the second time this check has caught its own author — the first was
  `DOCTRINE_ENFORCEMENT.md` at `VOLUME-DATA-LOCALITY.7` — and it is worth
  recording as a workflow fact: **run the driver after `git add`, not
  before**, or a new file's content is invisible to it.

## Open Questions

- None. `CARGO_TARGET_TMPDIR`'s value is fixed by Cargo (`<target-dir>/tmp`)
  and the harness reads it through `env!`, so the corrected text is
  re-derivable from the source, not from memory.

## Blockers

- None.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-07-29` | `CARGO-TMPDIR-SWEEP-REGRESSION.2` | 10/10 citations restored across 8 files; per-file token-set diff vs pre-sweep `4bb1822` **8/8 MATCH** (an inversion of the sweep, not a re-guess); residue census → **0 citations** (6 defect-describing occurrences retained on purpose); `mdbook build book` exit 0; `cargo test --test book_examples` **3/3** in 87.06s under `ram_guard --threshold 90`; `scripts/check_doctrines.sh` **5/5 PASS**. No `src/`/`tests/` touched ⇒ DUT byte-identical. | `done` — every cited artifact directory now exists |
| `2026-07-29` | `CARGO-TMPDIR-SWEEP-REGRESSION.1` | 9 negative controls (6 must-FAIL absolute forms → exit 1; 3 must-PASS relative forms → exit 0), each injected into `README.md` and reverted. Narrowing proof by old-vs-new offender-set diff: identical (0 files) after the allow-list filter, and identical repo-wide including allow-listed files ⇒ nothing previously caught is now tolerated. `scripts/check_doctrines.sh` **5/5 PASS**. No `src/`/`tests/` touched ⇒ DUT byte-identical (no `cargo` run warranted; the changed artifact is a shell check, and its oracle is the negative controls above). | `done` — the guard now states the actual rule: a boot-volume path is absolute |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `CARGO-TMPDIR-SWEEP-REGRESSION.1` | `CARGO-TMPDIR-SWEEP-REGRESSION.1 — anchor the boot-volume check to absolute paths` | doctrine script + standard + tree registration; no generator code ⇒ DUT byte-identical |
| `CARGO-TMPDIR-SWEEP-REGRESSION.2` | `CARGO-TMPDIR-SWEEP-REGRESSION.2 — repair the 10 mangled target/tmp citations` | live docs + book + layer-B history; no generator code ⇒ DUT byte-identical |

## Changelog

- `2026-07-29`: Created from a session-bootstrap deep-dive that found 10
  live-doc paths pointing at a directory that has never existed, and the
  doctrine check unable to distinguish the correct text from a breach.
- `2026-07-29`: Tree CLOSED. `.1` anchored `NO-BOOT-VOLUME-REFS` to absolute
  paths (9 negative controls both directions; narrows false positives only),
  `.2` restored all 10 citations byte-identically to their pre-sweep values.
  The lasting output is not the ten strings but the recorded rule now in
  `DEVELOPMENT_NOTES.md`: **a gate written from a sweep's own search string
  inherits that sweep's blind spot** — write the check from the property.
