# VOLUME-DATA-LOCALITY: project data must live on the repository volume

## Metadata

- Tree ID: `VOLUME-DATA-LOCALITY`
- Status: `active`
- Roadmap lane: Quality / data locality + path portability (cross-cutting; no phase reopened)
- Created: `2026-07-29`
- Last updated: `2026-07-29`
- Owner: repo-local workflow (opened on an explicit owner directive)

## Goal

Make ANVIL comply with the owner's **§12 repo-root-relative paths** and
**§13 project data locality / same-volume storage** policy: no project
data — generated outputs, build artifacts, caches, logs, runtime-created
test fixtures, or temporary workspaces — may default to `/tmp`,
`/private/tmp`, a home cache, or any other off-volume location. Paths
must be repo-root-relative when persisted, and absolute paths must be
derived at runtime from the current root.

## Observation that opened this tree (`2026-07-29`, owner directive)

The owner moved all projects to a 4TB SSD. Measured this session:

- repo volume: `/dev/disk5s1` (`/Volumes/SSD`)
- `/tmp` + `/private/tmp`: `/dev/disk3s1s1` (the boot volume) ⇒ **every
  OS-temp write by ANVIL is off-volume**.

### Audit — `std::env::temp_dir()` call sites (the off-volume default)

**(a) PRODUCTION — the one user-facing violation.**
`src/downstream/mod.rs:1185` — `impl Default for ValidateOptions` sets
`sandbox_root: std::env::temp_dir()`. This is the sandbox root for the
hardened downstream-tool runner, so **every** `validate` / `minimize` /
`hunt` / `divergence` run (CLI *and* MCP) writes the generated `.sv` and
the tools' outputs off-volume by default. `src/hunt/mod.rs` and
`src/divergence/mod.rs` inherit it.

**(b) DEBUG DUMP.** `src/ir/compact.rs:3167` writes
`/tmp/anvil-bisim-merged.sv` (env-gated by `ANVIL_DUMP_BISIM_SV`) — a
hardcoded absolute off-volume path, also a §12 violation. It is cited as
the `reverify` command of the KM card `bisimulation-flop-merge`, so the
card must move with it.

**(c) TEST FIXTURES / TEMPORARY WORKSPACES** — in policy scope (§13 names
"runtime-created test fixtures, and temporary workspaces" explicitly),
lower risk because they are not user-facing:
`src/hunt/mod.rs` (4 sites), `src/bin/tool_matrix.rs` (2),
`src/mcp/mod.rs` (1), `src/diff_sim/mod.rs` (1),
`src/downstream/mod.rs:2283` (1), `src/divergence/mod.rs` (1),
`tests/slang_e2e.rs`, `tests/sv2v_e2e.rs`, `tests/book_examples.rs` (2).

**(d) NOT violations** — `/tmp/...` string literals in CLI-argument
*parse* tests (`src/main.rs`, `src/bin/tool_matrix.rs`): nothing is
written to disk; they only assert flag parsing. Leave them, or switch
them for readability only, never as a policy fix.

**Checked and clean (do not re-derive).** `HuntRequest::bundle_root` is
`Option<PathBuf>` defaulting to `None` — a reproducer bundle is written
only where the user explicitly points `--out`, so the hunt lane has **no**
off-volume default of its own; it inherits the sandbox fix through
`ValidateOptions`. Likewise `anvil --out DIR` (generation) is always
caller-supplied. The sandbox root really was the single production
default.

### The design tension this tree must resolve

ANVIL is **distributed** (tagged release binaries + a composite GitHub
Action). A user running `anvil hunt` in their own CI has no ANVIL repo,
so "put it under the ANVIL repo root" is not a universally valid rule —
the honest generalization of §13 is *"the project's own volume, derived
at runtime, never an unconditional OS temp dir."* Hence the resolution
order proposed in `.2`.

### Also surfaced (owner decision required, NOT actioned)

The pre-move checkout at `~/Documents/github/anvil` still exists:
**1.1G off-volume** (877M of it `target/`). Its HEAD `ecda0e7` is a
**strict ancestor** of this checkout's HEAD, and its
`.cache/local-references` is identical to ours (118 files / 6.7M) ⇒ it
holds **no unique work**. Deleting a repository checkout is destructive,
so it is surfaced, not performed. Recorded in `.4`.

## Non-Goals

- Not changing what any tool *does* — only where its bytes land.
- Not changing generated RTL. Every leaf must keep the DUT lane
  byte-identical (`tests/snapshots.rs` untouched).
- Not deleting the old off-volume checkout without an explicit owner
  decision (`.4` records it; it does not act).
- Not touching the CLI-parse string literals in (d) as if they were
  violations.

## Acceptance Criteria

- No production code path defaults to an off-volume location; a single
  shared resolver owns the decision (one helper, not per-call-site
  `temp_dir()` — the `feedback_full_factorization` discipline).
- The resolver is overridable, derives from the current root at runtime,
  and persists nothing absolute.
- Test fixtures and temporary workspaces land on the repo volume.
- A decision record states the contract; `0030` is amended where its
  rejected option (iii) is now partly mandated.
- Mechanically checkable where possible (a `check_*.sh` in the
  `scripts/check_doctrines.sh` registry), or the tree records why not.
- DUT byte-identical; live docs + book updated; each leaf via `COMMIT.md`.

## Task Tree

- ID: `VOLUME-DATA-LOCALITY`
  Status: `active`
  Goal: bring all ANVIL-owned data onto the repository volume, by a
        runtime-derived root rather than an OS temp default.
  Children: `.1`, `.2`, `.3`, `.4`, `.5`

- ID: `VOLUME-DATA-LOCALITY.1`
  Status: `done`
  Goal: register the tree and record the measured audit before any edit
        (the `EVIDENCE-BANK-DURABILITY.1` / `BOOK-LANE-COVERAGE.1`
        precedent).
  Acceptance: this file + a `docs/TASK_TREE.md` row + live docs;
        docs-only ⇒ DUT byte-identical.
  Verification: `df` volume split measured; `temp_dir()` / `/tmp`
        call-site audit enumerated above and classified (a)-(d);
        `scripts/check_doctrines.sh` 4/4 PASS.
  Commit: `VOLUME-DATA-LOCALITY.1 — register + audit: off-volume project data`

- ID: `VOLUME-DATA-LOCALITY.2`
  Status: `done`
  Goal: the shared **sandbox-root resolver** + rewire the production
        default. Proposed resolution order (design-first, record in the
        ADR): explicit `ANVIL_SANDBOX_ROOT` override → else a
        repo-root-derived path when a root is detectable by walking up
        for a `.git` / `Cargo.toml` marker → else a
        current-working-directory-derived path. Never `temp_dir()`.
        Rewire `ValidateOptions::default()` (and therefore hunt /
        divergence / minimize / the MCP adapter) onto it; move the
        `compact.rs` bisim dump onto it too and update the KM card's
        `reverify`.
  Acceptance: no production `temp_dir()` remains; resolver unit-tested
        incl. the override and the marker-walk; sandbox still removed
        after a run unless `keep_sandbox`; DUT byte-identical
        (`tests/snapshots.rs` 6/6); full gate green.
  Verification: New `src/paths.rs` (+ `pub mod paths` in `lib.rs`):
        `sandbox_root()` = `$ANVIL_SANDBOX_ROOT` → marker-walked project
        root → CWD, each suffixed `.cache/anvil-sandbox`; never
        `temp_dir()`. `ValidateOptions::default()` rewired onto it (so
        validate / minimize / hunt / divergence, CLI **and** MCP, all
        inherit); `ir/compact.rs`'s `ANVIL_DUMP_BISIM_SV` dump moved off
        the hardcoded `/tmp/anvil-bisim-merged.sv` and now prints the
        path it wrote. 5 resolver unit tests (never-under-temp-dir,
        lands-under-project-root, marker walk-up, nearest-marker-wins,
        env-override) — all green.
        **Measured end-to-end (the REJECT→PASS):** a real
        `downstream::validate(7, …)` with Verilator 5.x + Yosys installed
        now reports
        `sandbox = <repo>/.cache/anvil-sandbox/anvil-validate-d8420426e78b2d05`
        with `ok = true` and both tools `success = true` — previously
        this landed under the OS temp dir on a different volume.
        `.cache/` is already gitignored, so the sandbox never pollutes
        `git status` (confirmed). Gate: `cargo check --all-targets`,
        `cargo clippy --all-targets -- -D warnings`, `cargo fmt --all
        --check`, `mdbook build book` all clean; full `cargo test` under
        `scripts/ram_guard.sh --threshold 90` → **exit 0**
        (`tests/snapshots.rs` **6/6 byte-identical** — the change moves
        bytes, never generates different ones).
        Book/doc sync folded in here rather than deferred to `.5`,
        because the change made two `book/src/api-tools.md` examples
        (`"sandbox": "/tmp/anvil-validate-…"`) wrong on landing: both
        updated, plus a "Where the sandbox lands" note in the book and a
        matching `USER_GUIDE.md` section documenting the resolution order
        and `ANVIL_SANDBOX_ROOT`.
  Commit: `VOLUME-DATA-LOCALITY.2 — sandbox root resolves to the project volume, never OS temp`

- ID: `VOLUME-DATA-LOCALITY.3`
  Status: `pending`
  Goal: migrate the (c) test-fixture / temporary-workspace sites onto the
        same resolver (or a test-scoped sibling), so `cargo test` writes
        nothing off-volume.
  Acceptance: no `temp_dir()` outside the resolver; full suite green;
        a residue census shows no new `/private/tmp` ANVIL data after a
        full `cargo test`.
  Verification: `pending`
  Commit: `pending`

- ID: `VOLUME-DATA-LOCALITY.4`
  Status: `pending`
  Goal: the decision record (`docs/decisions/0031-*.md`) stating the
        data-locality contract + the resolver's resolution order +
        rejected alternatives; amend decision `0030` (its rejected
        option (iii) is now **partly mandated**: a future evidence bank
        must be repo-derived and on-volume — the committed digest
        remains the citation form); record the old-checkout finding for
        the owner's decision; add the doctrine check if feasible.
  Acceptance: ADR + `INDEX.md` row + KM front-matter; `0030` amended,
        not silently rewritten (supersede-don't-mutate).
  Verification: `pending`
  Commit: `pending`

- ID: `VOLUME-DATA-LOCALITY.5`
  Status: `pending`
  Goal: docs + book sync — `USER_GUIDE.md` (the new env override and
        where sandboxes land), `TOOLBOX.md`, and the book chapters that
        mention sandbox paths (`book/src/api-tools.md` shows a
        `"sandbox": "/tmp/anvil-validate-…"` example that will become
        wrong). **Narrowed after `.2`:** the two `api-tools.md` examples,
        the book's "Where the sandbox lands" note, and the
        `USER_GUIDE.md` resolution-order section already landed *in* `.2`
        (book-sync doctrine — they became wrong the moment `.2` landed).
        Remaining here: `TOOLBOX.md` + a sweep for any other doc
        implying an OS-temp sandbox.
  Acceptance: no live doc or book page shows an off-volume sandbox as
        current behaviour; `mdbook build` + `book_examples` green.
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `VOLUME-DATA-LOCALITY.3` | `pending` | Test fixtures + temporary workspaces onto the resolver, now that it exists and is proven. 13 sites; `cargo test` should write nothing off-volume, verified by a residue census after a full run. |
| 2 | `VOLUME-DATA-LOCALITY.4` | `pending` | ADR `0031` + amend `0030`, now that the resolution order is proven in code rather than proposed. |
| 3 | `VOLUME-DATA-LOCALITY.5` | `pending` | **Narrowed** — the `book/src/api-tools.md` examples and the `USER_GUIDE.md` section landed in `.2` (they went stale the moment `.2` landed). What remains: `TOOLBOX.md` and a sweep for any other doc implying an OS-temp sandbox. |

## Decisions

- `2026-07-29`: Opened as a tracked tree on an explicit owner directive,
  registered before any `src/` edit (code-change doctrine).
- `2026-07-29`: Ordering is **production-first**, unlike the
  usual design-first cadence: the audit already pins the defect and the
  fix shape, and every hour the default stands is more off-volume data.
  The ADR (`.4`) follows the proven resolver rather than preceding it.
- `2026-07-29`: The old off-volume checkout is **surfaced, not deleted** —
  destructive and the owner's call.

## Open Questions

- For a **distributed** ANVIL binary run outside this repo, is the
  CWD-derived fallback the right final rung? (Proposed in `.2`;
  alternatives: refuse to run without an explicit root, or an
  XDG-style project cache. The fallback must never be an OS temp dir.)
- Should the resolver be a new doctrine check (`check_no_off_volume_defaults.sh`
  grepping for `temp_dir()` outside the resolver module)? Cheap and
  structural — likely yes, decided in `.4`.

## Blockers

- None.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-07-29` | `VOLUME-DATA-LOCALITY.2` | 5 new `paths` unit tests green (incl. `sandbox_root_is_never_the_os_temp_dir`, which compares against `temp_dir()` itself — on macOS that is a per-user `/var/folders/…` path, so a `"/tmp"` grep would have missed it); real `validate(7)` with Verilator + Yosys → `sandbox = <repo>/.cache/anvil-sandbox/anvil-validate-d8420426e78b2d05`, `ok = true`, both tools `success = true`; `.cache/` gitignored ⇒ no `git status` pollution; `cargo check --all-targets` + `clippy -D warnings` + `fmt --check` + `mdbook build book` clean; full `cargo test` under `ram_guard --threshold 90` exit 0 with `tests/snapshots.rs` 6/6 byte-identical | `done` — the one production off-volume default is gone |
| `2026-07-29` | `VOLUME-DATA-LOCALITY.1` | `df -h` on the repo vs `/private/tmp` → `/dev/disk5s1` vs `/dev/disk3s1s1` (off-volume confirmed); `grep -rn "temp_dir()" src/ tests/ scripts/` → 15 sites classified (a) 1 production, (b) 1 debug dump, (c) 13 test/workspace, plus (d) non-violating parse literals; old-checkout inspection read-only (`git merge-base --is-ancestor ecda0e7 HEAD` → true; caches identical); `scripts/check_doctrines.sh` 4/4 PASS | `done` — tree registered; docs-only ⇒ DUT byte-identical |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `VOLUME-DATA-LOCALITY.1` | `VOLUME-DATA-LOCALITY.1 — register + audit: off-volume project data` | docs-only |
| `VOLUME-DATA-LOCALITY.2` | `VOLUME-DATA-LOCALITY.2 — sandbox root resolves to the project volume, never OS temp` | code + book/doc sync; moves where bytes land, never which bytes ⇒ DUT byte-identical |

## Changelog

- `2026-07-29`: Created from the owner's §12/§13 directive after the
  projects moved to the 4TB SSD; audit found one production off-volume
  default (the downstream sandbox root) plus a hardcoded debug-dump path
  and 13 test-fixture sites.
