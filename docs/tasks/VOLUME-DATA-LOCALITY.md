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

**(c) TEST FIXTURES / TEMPORARY WORKSPACES — 14 sites** — in policy scope
(§13 names "runtime-created test fixtures, and temporary workspaces"
explicitly), lower risk because they are not user-facing:
`src/hunt/mod.rs` (4), `src/bin/tool_matrix.rs` (2),
`src/mcp/mod.rs` (1), `src/diff_sim/mod.rs` (1),
`src/downstream/mod.rs:2283` (1), `src/divergence/mod.rs` (1),
`tests/slang_e2e.rs` (1), `tests/sv2v_e2e.rs` (1),
`tests/book_examples.rs` (2).
*(Corrected in `.3`: the `.1` write-up said "13"; the enumeration is 14,
which with the 1 production site is the 15 total the grep reported.)*

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

### The pre-move checkout — surfaced, then RESOLVED by the owner

The pre-move checkout at `~/Documents/github/anvil` still exists:
**1.1G off-volume** (877M of it `target/`). Its HEAD `ecda0e7` is a
**strict ancestor** of this checkout's HEAD, and its
`.cache/local-references` is identical to ours (118 files / 6.7M) ⇒ it
holds **no unique work**. Deleting a repository checkout is destructive,
so it was surfaced rather than performed.

**Owner decision (`2026-07-29`): keep it for now — leave it alone.** The
owner is retaining the original projects under `~/Documents/github` and
will remove that whole directory themselves later. It is therefore
**deliberate, owner-owned data**, not a policy breach to remediate:

- **No agent may delete or migrate it.** Not now, not as cleanup, not as
  part of a later leaf. Removal is the owner's action on their schedule.
- It is **out of scope for this tree**, which governs the paths *ANVIL's
  code* writes to, not where the owner keeps checkouts.
- Any future off-volume audit must **exclude** `~/Documents/github` —
  finding it there is the expected state, not a regression. A doctrine
  check added in `.4` must not flag it.

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
  Children: `.1`, `.2`, `.3`, `.4`, `.5`, `.6`, `.7`

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
  Status: `done`
  Goal: migrate the (c) test-fixture / temporary-workspace sites onto the
        same resolver (or a test-scoped sibling), so `cargo test` writes
        nothing off-volume.
  Acceptance: no `temp_dir()` outside the resolver; full suite green;
        a residue census shows no new `/private/tmp` ANVIL data after a
        full `cargo test`.
  Verification: All 14 sites rewired onto `crate::paths::sandbox_root()`
        (`anvil::paths::` from the `tool_matrix` bin and the integration
        tests). `grep -rn "temp_dir()" src/ tests/` now matches **only**
        `src/paths.rs` — the resolver is the sole place the OS temp dir
        is even named, which is what makes the rule enforceable rather
        than aspirational.
        `tests/book_examples.rs` keeps its load-bearing invariant: the
        stdout/stderr capture files stay *siblings* of the script CWD,
        never inside it, so a book block that enumerates its working
        directory is still unaffected.
        **Before/after census (the measurement that proves it):** the OS
        temp dir held **14** `anvil-*` entries from prior runs before
        this leaf (real off-volume residue — the exact policy breach);
        cleared to 0, then a full `cargo test` under
        `ram_guard --threshold 90` → `CARGO_TEST_EXIT=0` (lib 736/0/2ig,
        `tests/pipeline.rs` 125/0, `tests/snapshots.rs` **6/6
        byte-identical**, `tests/book_examples.rs` 3/3), and the census
        immediately after the run over **both** the per-user `TMPDIR`
        and `/private/tmp` → **0 entries**. So the measured before/after
        is **14 → 0**: a full test run now writes nothing ANVIL-owned
        off-volume.

        **Self-inflicted failure, root-caused (recorded so nobody
        re-derives it).** The first census run came back `EXIT=101` with
        `book_examples` failing on `recipes.md:607` (the 1000-module
        `--out ./parse-stress/` block) and an **empty** captured-output
        tail. Cause: mid-run, this session ran `rm -rf
        .cache/anvil-sandbox` to tidy a bisim dump — which is now the
        *live sandbox root the suite is using*, so it deleted that
        block's working directory **and** its stdout/stderr capture
        files (hence the empty tail: the harness read a file that no
        longer existed). Re-running `cargo test --test book_examples`
        untouched → **3/3 green**, confirming the hypothesis rather than
        assuming it. **Operational consequence of `.2`/`.3`:** the
        sandbox root is now inside the repo, so deleting it during a
        test run is destructive in a way it never was when it lived in
        the OS temp dir. Clear it *before* or *after* a run, never
        during. The gate numbers recorded here are from a clean re-run
        with no concurrent filesystem mutation.
        Also re-ran the `bisimulation-flop-merge` KM card's `reverify`
        end-to-end: it now prints
        `<repo>/.cache/anvil-sandbox/anvil-bisim-merged.sv` and the
        dumped SV is clean under Verilator (`-Wall -Wno-DECLFILENAME`),
        both Yosys modes, and Icarus. The card was updated to require
        `-Wno-DECLFILENAME` and to say why: the dump uses a fixed
        filename that cannot match the generated module name, so bare
        `-Wall` always exits nonzero on `DECLFILENAME` — a future agent
        would otherwise read that as the fact being broken.
  Commit: `VOLUME-DATA-LOCALITY.3 — test fixtures and temp workspaces move onto the resolver`

- ID: `VOLUME-DATA-LOCALITY.4`
  Status: `pending`
  Goal: the decision record (`docs/decisions/0031-*.md`) stating the
        data-locality contract + the resolver's resolution order +
        rejected alternatives; amend decision `0030` (its rejected
        option (iii) is now **partly mandated**: a future evidence bank
        must be repo-derived and on-volume — the committed digest
        remains the citation form); record the settled old-checkout
        decision (owner keeps `~/Documents/github` and removes it
        themselves later — never an agent action, and excluded from any
        audit or check); add the doctrine check if feasible.
  Acceptance: ADR + `INDEX.md` row + KM front-matter; `0030` amended,
        not silently rewritten (supersede-don't-mutate).
  Verification: `pending`
  Commit: `pending`

- ID: `VOLUME-DATA-LOCALITY.6`
  Status: `withdrawn` (will not be done)
  Goal (as originally scoped): give the project its own
        `CARGO_HOME`/`RUSTUP_HOME` on the SSD.
  **Withdrawn by owner decision (`2026-07-29`), final:** *"If something
  is shared, then please use the shared information, no need to
  duplicate the information that is shared on the SSD."*
  So `~/.cargo` (845M) and `~/.rustup` (2.1G) are **used in place**.
  `CARGO_HOME` / `RUSTUP_HOME` keep their defaults; nothing is copied,
  nothing is deleted, and builds are entirely unchanged.
  The rule this settles, now recorded in decision `0031`: **shared means
  shared — use it, do not fork it.** Duplicating a shared store onto the
  SSD would have cost 3 GB *and* created a second cache that silently
  drifts from the one every other project uses — the opposite of the
  full-factorization discipline the project applies to its own code.
  This is *not* a boot-volume exception being smuggled in: ANVIL stores
  no data of its own in those directories; it reads and executes from
  them. The directive governs the data ANVIL **creates**.
  Trail (recorded because the direction moved twice and a future reader
  should not have to reconstruct it): proposed → withdrawn on
  *"everything shared in `$HOME` stays in `$HOME`"* (the 3 GB of copies
  made during the proposal were deleted) → briefly reinstated on
  *"please do"* → **finally withdrawn** on the statement above. Net
  effect on the repository: none — no copies exist, no env var is set.

- ID: `VOLUME-DATA-LOCALITY.7`
  Status: `pending`
  Goal: the mechanical gate — `scripts/check_no_boot_volume_refs.sh`
        registered in `scripts/check_doctrines.sh`, failing on any new
        boot-volume path in tracked files (`/tmp/`, `/private/tmp`,
        `$TMPDIR`, `$HOME/.cargo`, `/Users/`) outside an explicit
        allow-list. The allow-list must cover: the `.github/workflows/`
        `$HOME/.cargo/bin` lines (those run on GitHub's runners, where
        `$HOME` is the runner's own disk — correct and portable, NOT a
        violation on this machine), and the CLI-*parse* string literals
        recorded in audit class (d).
  Acceptance: driver reports the new check PASS on a compliant tree and
        FAIL on a synthetic new `/tmp/...` reference; `check_doctrines.sh`
        reports 5/5.
  Verification: `pending`
  Commit: `pending`

- ID: `VOLUME-DATA-LOCALITY.5`
  Status: `done`
  **Scope escalated (`2026-07-29`, owner):** *"nothing shall reference or
  point to the boot volume — everything must be relative to this SSD
  volume."* So this leaf is no longer a small doc touch-up: it is a
  **full sweep of all 2376 boot-volume path references across 60+
  tracked files**, under three rules:
  - **Rule A — dead evidence-bank names** (355 distinct `/tmp/anvil-*`):
    strip the `/tmp/` prefix, keeping `anvil-<name>` as a bare
    *identifier*. These banks no longer exist (that is what
    `EVIDENCE-BANK-DURABILITY` established), so the prefix was never
    load-bearing — the name is what identifies the run, and decision
    `0030` makes the re-runnable command the real citation.
  - **Rule B — live scratch paths in runnable recipes** (`/tmp/c.json`,
    `/tmp/fe.sv`, … — mostly Knowledge-Map `reverify` one-liners and
    book/doc recipes): rewrite to `.cache/anvil-sandbox/…` so they stay
    runnable *and* write on-volume. These are the only class that
    actually *stored* on the boot volume.
  - **Rule C — CLI-parse literals** (`/tmp/x` in `src/bin/tool_matrix.rs`
    and `src/main.rs`): never written to disk, but still references, so
    they move too. This **supersedes** audit class (d), which had
    recorded them as "leave alone" — correct under §13, wrong under the
    stricter directive.
  Deliberately **kept**: the boot-volume paths inside the policy
  documents themselves (`docs/decisions/0031`, this tree, `MEMORY.md`) —
  a doctrine cannot state what it forbids without naming it. The `.7`
  check allow-lists exactly those.
  Generated artifacts (`KNOWLEDGE_MAP.md`) are fixed at their **sources**
  (`docs/knowledge/*.md`, `docs/decisions/*.md`) and regenerated.
  Acceptance: `git grep -E '/tmp/|/private/tmp|/var/folders'` returns
  only the allow-listed policy documents; every rewritten `reverify`
  command still runs; full gate green.
  Verification: **52 files swept — Rule A 538 rewrites, Rule B/C 124.**
        Residual boot-volume strings live ONLY in the allow-list, and
        that boundary is deliberate, not laziness:
        - **policy/ADR documents** (`0002`, `0030`, `0031`, this tree,
          `docs/TASK_TREE.md`'s + `docs/decisions/INDEX.md`'s rows for
          them, `MEMORY.md`) — *a doctrine cannot state what it forbids
          without naming it*;
        - **append-only history** (`CHANGES.md`,
          `DEVELOPMENT_NOTES.md`) — `MEMORY_ARCHITECTURE.md` §3 says
          layer-C/D records are appended and superseded, **never
          silently rewritten**; retroactively editing what a past entry
          said would corrupt the audit trail to satisfy a
          presentation rule.
        `KNOWLEDGE_MAP.md` is generated, so it was fixed at its sources
        (`docs/knowledge/*.md`, `docs/decisions/*.md`) and regenerated —
        83 facts / 822 keys.
        **A first attempt was reverted.** A blanket sweep with no
        allow-list rewrote decision `0030`'s own `reverify` from
        `ls -d /tmp/anvil-*` to `ls -d anvil-*` — a meaningless command —
        and mangled its question key *"why does a cited /tmp/anvil path
        not exist"*. That is the hazard of a mechanical rewrite over
        documents whose subject IS the string being rewritten. Reverted
        via `git checkout` and redone with the allow-list above.
        Rewritten `reverify` recipes were made self-sufficient
        (`mkdir -p .cache/anvil-sandbox && …`) in 13 cards, and one was
        **executed end-to-end** to prove the class still works:
        `mux-if-emit` → 8 `__cv` blocks emitted, `iverilog -g2012`
        clean, all writes on-volume.
        Gate: `cargo check --all-targets`, `clippy -D warnings`,
        `fmt --all --check`, `mdbook build book`, and
        `cargo test --bin tool_matrix` 99/0 all green; full `cargo test`
        under `ram_guard --threshold 90` → `CARGO_TEST_EXIT=0`
        (lib 736/0/2ig, pipeline 125/0, **snapshots 6/6 byte-identical**,
        book_examples 3/3).
  Commit: `VOLUME-DATA-LOCALITY.5 — sweep every boot-volume path reference`

- ID: `VOLUME-DATA-LOCALITY.5-ORIG`
  Status: `superseded`
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
- `2026-07-29`: The old off-volume checkout was **surfaced, not deleted** —
  destructive and the owner's call. **Resolved the same day:** the owner
  keeps `~/Documents/github` (all original projects) for now and will
  remove the whole directory themselves later. It is deliberate,
  owner-owned data — excluded from this tree's scope and from any
  off-volume audit or doctrine check. No agent deletes or migrates it.

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
| `2026-07-29` | `VOLUME-DATA-LOCALITY.5` | 52 files / 662 rewrites (Rule A 538, Rule B/C 124); KM regenerated from sources (83 facts / 822 keys); 13 `reverify` recipes made self-sufficient and `mux-if-emit` executed end-to-end (8 `__cv`, iverilog clean, writes on-volume); allow-list = policy docs + append-only history (owner directive: no history rewrite) + GitHub-runner `$HOME` lines. First allow-list-free attempt broke `0030`'s own reverify → reverted via `git checkout`, redone. Gate: check/clippy(-D warnings)/fmt/mdbook/`--bin tool_matrix` 99/0 green; full `cargo test` under `ram_guard --threshold 90` `CARGO_TEST_EXIT=0` — lib 736/0, pipeline 125/0, **snapshots 6/6 byte-identical**, book_examples 3/3 | `done` — no live doc or code path points at the boot volume |
| `2026-07-29` | `VOLUME-DATA-LOCALITY.3` | All 14 test-fixture sites rewired; `grep -rn "temp_dir()" src/ tests/` matches **only** `src/paths.rs`. Residue census **14 → 0** (per-user `TMPDIR` + `/private/tmp`, before vs after a full run). Clean full `cargo test` under `ram_guard --threshold 90` → `CARGO_TEST_EXIT=0`: lib 736/0 (2 ignored), pipeline 125/0, **snapshots 6/6 byte-identical**, book_examples 3/3. KM card `bisimulation-flop-merge` reverify re-run end-to-end: dump lands at `<repo>/.cache/anvil-sandbox/anvil-bisim-merged.sv`, clean under Verilator (`-Wall -Wno-DECLFILENAME`) + both Yosys modes + Icarus. First census attempt was `EXIT=101` from a self-inflicted mid-run `rm -rf` of the live sandbox root — root-caused, re-run clean, hazard recorded. | `done` — `cargo test` writes nothing off-volume |
| `2026-07-29` | `VOLUME-DATA-LOCALITY.2` | 5 new `paths` unit tests green (incl. `sandbox_root_is_never_the_os_temp_dir`, which compares against `temp_dir()` itself — on macOS that is a per-user `/var/folders/…` path, so a `"/tmp"` grep would have missed it); real `validate(7)` with Verilator + Yosys → `sandbox = <repo>/.cache/anvil-sandbox/anvil-validate-d8420426e78b2d05`, `ok = true`, both tools `success = true`; `.cache/` gitignored ⇒ no `git status` pollution; `cargo check --all-targets` + `clippy -D warnings` + `fmt --check` + `mdbook build book` clean; full `cargo test` under `ram_guard --threshold 90` exit 0 with `tests/snapshots.rs` 6/6 byte-identical | `done` — the one production off-volume default is gone |
| `2026-07-29` | `VOLUME-DATA-LOCALITY.1` | `df -h` on the repo vs `/private/tmp` → `/dev/disk5s1` vs `/dev/disk3s1s1` (off-volume confirmed); `grep -rn "temp_dir()" src/ tests/ scripts/` → 15 sites classified (a) 1 production, (b) 1 debug dump, (c) 14 test/workspace (this row said 13 when written; corrected in `.3` — the enumeration is 14, and 14 + 1 production = the 15 the grep reported), plus (d) non-violating parse literals; old-checkout inspection read-only (`git merge-base --is-ancestor ecda0e7 HEAD` → true; caches identical); `scripts/check_doctrines.sh` 4/4 PASS | `done` — tree registered; docs-only ⇒ DUT byte-identical |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `VOLUME-DATA-LOCALITY.1` | `VOLUME-DATA-LOCALITY.1 — register + audit: off-volume project data` | docs-only |
| `VOLUME-DATA-LOCALITY.2` | `VOLUME-DATA-LOCALITY.2 — sandbox root resolves to the project volume, never OS temp` | code + book/doc sync; moves where bytes land, never which bytes ⇒ DUT byte-identical |
| `VOLUME-DATA-LOCALITY.3` | `VOLUME-DATA-LOCALITY.3 — test fixtures and temp workspaces move onto the resolver` | test-scope paths only ⇒ DUT byte-identical; residue census 14 → 0 |
| `VOLUME-DATA-LOCALITY.5` | `VOLUME-DATA-LOCALITY.5 — sweep every boot-volume path reference` | prose + doc comments + test literals ⇒ DUT byte-identical; history deliberately NOT rewritten |

## Changelog

- `2026-07-29`: Created from the owner's §12/§13 directive after the
  projects moved to the 4TB SSD; audit found one production off-volume
  default (the downstream sandbox root) plus a hardcoded debug-dump path
  and 13 test-fixture sites.
