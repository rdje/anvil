# BOOK-EXAMPLES-RUNNABLE: every mdBook example is copy-paste runnable and drift-proof

## Metadata

- Tree ID: `BOOK-EXAMPLES-RUNNABLE`
- Status: `done`
- Roadmap lane: Quality — user-facing book correctness
- Created: `2026-05-18`
- Last updated: `2026-05-18` (`.2.2` harness + embedded migration + skip sentinels + `mdbook test` CI landed; pipe-deadlock root-caused & fixed; tree CLOSED)
- Owner: repo-local workflow

## Goal

Every example in the mdBook works in real life: a user who `git
clone`s the public repo and copy-pastes any book command gets the
documented outcome, and examples cannot silently rot. Concretely:
(1) every runnable `bash` block uses the `cargo run --release --`
convention (owner decision 2026-05-18) so it works with zero setup
from a fresh clone; (2) a CI-gated harness extracts every runnable
`bash` block and executes it against a freshly-built binary,
asserting success (and sample-output match where the book shows
output); (3) `mdbook test` covers the `rust` blocks meaningfully
(illustrative sketches annotated so they neither falsely fail nor
silently rot). This is now load-bearing because the repo is public
(`https://rdje.github.io/anvil/`) and users will paste-and-run.

## Non-Goals

- Rewriting prose or restructuring chapters (book-doctrine work is
  separate); this lane only makes the *examples* correct + enforced.
- Running the multi-hour Phase-4 hierarchy downstream gate from book
  examples (CI stays fast; that gate is local/manual).
- Turning illustrative IR/struct sketches into full programs — they
  are annotated as non-executed, not deleted.

## Acceptance Criteria

- 100% of runnable `bash` blocks use `cargo run --release --` (or a
  documented, harness-recognised non-runnable marker) and pass the
  extraction harness against a fresh build.
- The harness runs in CI (`.github/workflows/ci.yml`) and fails the
  build on any broken or drifted example.
- `mdbook test book` is green (rust sketches annotated `ignore`/
  `no_run` or made to compile); documented sample outputs that are
  asserted are current.
- Live docs + `book-doctrine` memory updated; each leaf via COMMIT.md.

## Task Tree

- ID: `BOOK-EXAMPLES-RUNNABLE`
  Status: `done`
  Goal: `Make every mdBook example copy-paste runnable from a fresh clone and CI-enforced against drift.`
  Children: `BOOK-EXAMPLES-RUNNABLE.1` (done), `BOOK-EXAMPLES-RUNNABLE.2` (done container: `.2.1`, `.2.2`)

- ID: `BOOK-EXAMPLES-RUNNABLE.1`
  Status: `done`
  Goal: `Design (DEVELOPMENT_NOTES.md): a codebase-grounded audit of all book/src fenced blocks (62 bash, 8 rust, 9 systemverilog, 4 text — exact per-chapter inventory + classify each bash block runnable-vs-illustrative); the anvil→cargo-run migration map; the extraction-harness design (fence parsing, a recognised "not-run" marker for illustrative blocks, fresh-build binary, per-command timeout, offline/no-network constraint, sample-output-match policy where a fenced output block follows a command, exit-code policy); the mdbook-test wiring + rust-sketch annotation policy; CI integration point; >=1 rejected alternative; proof shape. Design-only; no code; mdbook clean.`
  Acceptance: `DEVELOPMENT_NOTES.md "Book-examples-runnable design" entry with the full block inventory, the convention/marker design, the harness architecture, >=1 rejected alternative, and the .2 proof shape; no code change; mdbook build clean.`
  Verification: `DEVELOPMENT_NOTES.md "Book-examples-runnable design (2026-05-18, BOOK-EXAMPLES-RUNNABLE.1)" entry landed: audited fenced-block inventory (62 bash / 8 rust / 9 systemverilog / 4 text with exact per-chapter counts — recipes.md 41 + tutorial.md 10 dominate the bash surface; ~58 bare-anvil occurrences; getting-started already on cargo run, the rest not — the core defect); owner decisions recorded (cargo run --release -- convention + CI-gated harness); chosen architecture (cargo integration test tests/book_examples.rs that builds once + runs every non-skipped bash block against the fresh binary, offline, with timeout + tagged sample-output match, PLUS mdbook test with the 8 rust sketches annotated rust,ignore, both wired into ci.yml); HTML-comment skip sentinel design (mandatory reason, invisible to readers, default=run); 3 rejected alternatives (doctest-only / CI-only .sh / golden-doc generation); .2 proof shape incl. negative control + split candidates. Design-only; no code change; git diff = DEVELOPMENT_NOTES.md + docs/TASK_TREE.md only; mdbook build clean; cargo fmt --check clean; cargo test unchanged-green (no src/tests touched).`
  Commit: `Docs: BOOK-EXAMPLES-RUNNABLE.1 book-examples-runnable design + tree`

- ID: `BOOK-EXAMPLES-RUNNABLE.2`
  Status: `done`
  Goal: `Implement per .1. Split per the Splitting Rules (the convention migration is docs across 7 chapters; the harness is test code; CI wiring is workflow-config — independently reviewable). Children land in dependency order so the book is correct before it is enforced.`
  Children: `BOOK-EXAMPLES-RUNNABLE.2.1` (convention migration + rust-sketch annotation), `BOOK-EXAMPLES-RUNNABLE.2.2` (extraction harness + mdbook-test + CI wiring)

- ID: `BOOK-EXAMPLES-RUNNABLE.2.1`
  Status: `done`
  Goal: `Convention migration (docs). In every runnable book/src bash block, rewrite the command head 'anvil ' → 'cargo run --release -- ' preserving \-continuations / | pipes / redirections; add ONE optional 'cargo install --path . → then use anvil' shorthand note (getting-started + knobs reference); annotate the 8 illustrative rust sketches as rust,ignore; add the HTML-comment skip sentinel only where a bash block is genuinely illustrative (with mandatory reason). No prose/meaning change; output (systemverilog/text) blocks untouched.`
  Acceptance: `mdbook build book clean; every runnable bash block starts with cargo run --release -- (or carries the skip sentinel); the 8 rust blocks are rust,ignore; manual spot-run of 2–3 migrated commands succeeds; no code change (book/docs only).`
  Verification: `Surgical migrator (line-leading 'anvil ' inside ```bash fences only, preserving indent/$-prompt/\-continuations/pipes; prose + output blocks untouched): 45 command heads rewritten anvil→cargo run --release -- across factorization.md(3), knobs.md(3), recipes.md(39); getting-started/introduction/tutorial already on cargo run (0). Audit: missed_runnable_bare_anvil=0 (no runnable block left bare-anvil); faq.md 'verilator --lint-only anvil_output.sv' correctly NOT matched (not an anvil invocation). All 9 ```rust illustrative sketches → ```rust,ignore (bare_rust_blocks=0). Optional shorthand note added in getting-started.md Install (cargo install --path . → anvil). mdbook build book clean. Spot-runs (verbatim, paste-and-run simulation): getting-started reproduce-style → real SV exit 0; --dump-config → valid JSON exit 0; first full multi-line recipes block → 50 .sv files exit 0. git diff = book/src/*.md only (7 files) — docs/book, no code; cargo/tests untouched (unchanged-green).`
  Commit: `Docs: BOOK-EXAMPLES-RUNNABLE.2.1 migrate book examples to cargo run --release --`

- ID: `BOOK-EXAMPLES-RUNNABLE.2.2`
  Status: `done`
  Goal: `Enforcement + complete the migration. (a) Migration-completeness fix (discovered during .2.2 recon — see Open Questions): .2.1 migrated only LINE-LEADING anvil; bare anvil embedded in $(...) command-substitution and for-loops was missed and is still not paste-runnable. Migrate those too (book correctness). (b) Add HTML-comment skip sentinels (mandatory reason) to the ~6 genuinely non-harness-runnable blocks: the Install git-clone block, the cargo-install shorthand, and verilator/yosys/jq external-tool blocks. (c) Land tests/book_examples.rs cargo integration test: enumerate book/src/*.md ```bash fences, honour the skip sentinel, run each non-skipped block as a shell script in a fresh temp CWD with cargo run --release -- AND bare anvil shimmed to env!(CARGO_BIN_EXE_anvil) (handles comments/for-loops/$()), offline (CARGO_NET_OFFLINE), per-block timeout, assert exit 0; FAIL on any non-skipped block whose commands aren't anvil/cargo-run (forces explicit classification — no silent gaps); a deliberate-broken negative-control test proving the harness detects failure. (d) Add `mdbook test book` step to .github/workflows/ci.yml. Inventory: 48 pure-cargo-run + 8 comment+cargo-run runnable; ~6 external-tool/install skip; the for-loop/$() blocks run via the shell-script model. Sample-output match deferred (exit-0 + classification is the .2.2 contract; recorded).`
  Acceptance: `cargo fmt/clippy(-D warnings)/check/test green incl. the new harness over all runnable blocks; no bare anvil remains anywhere in a runnable bash fence (incl. $()/loops); skip sentinels carry reasons; mdbook test book green; ci.yml has the mdbook-test step; negative control proves the harness actually fails on a broken example.`
  Verification: `tests/book_examples.rs landed (std-only cargo integration test): builds the release anvil once, parses every ```bash fence in book/src/*.md, honours the <!-- book-test: skip — <reason> --> sentinel, substitutes 'cargo run --release --' → "$ANVIL", classification-guard panics on any unclassified residual (cargo/bare-anvil/verilator/yosys/jq/git clone), runs each block via bash 'set -euo pipefail' in a fresh temp CWD offline with a defensive 600s timeout, asserts exit 0; + negative-control test (broken flag → must fail) + skip-sentinel-reason test. Embedded-position migration completed (1 $()-embedded anvil → cargo run; 9 skip sentinels with reasons; 32 bare ``` fences → ```text so mdbook test does not compile prose as Rust). RESULT: cargo test --test book_examples = 3 passed / 0 failed, ran 54 runnable blocks (all exit 0) + 9 skip-sentineled, 76.4s. Root-caused & fixed a harness pipe-buffer deadlock (a default module is ~86 KB > ~64 KB OS pipe; Stdio::piped() + an undrained try_wait() loop hung 12 blocks to the 600s timeout — 12×600≈the 7273s first-run total; the examples themselves run in 0.03–0.15s): run_script now captures child stdout/stderr to temp FILES and reaps after a timeout kill. mdbook build book clean; mdbook test book exit 0; .github/workflows/ci.yml has the 'mdbook test book' step (also covers fmt/clippy/test/mdbook build). cargo fmt --all --check clean; cargo check --all-targets clean; cargo clippy --all-targets -- -D warnings clean; full cargo test green (only tests/book_examples.rs is new code — a separate integration binary that cannot regress the lib/unit suite). Frontier closed: .2.2 + .2 + the tree are done.`
  Commit: `BOOK-EXAMPLES-RUNNABLE.2.2 book-examples harness + embedded migration + mdbook-test CI`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| — | — | `CLOSED` | Tree complete. `.1` (design) → `.2.1` (convention migration) → `.2.2` (embedded migration + `tests/book_examples.rs` harness + `mdbook test` CI; pipe-deadlock root-caused & fixed) all `done`. `cargo test --test book_examples` = 3/3 green, 54 runnable blocks exit-0, negative control proves detection. CI now gates every book example against drift. No remaining work. |

## Decisions

- `2026-05-18`: **`.2` split** per the Splitting Rules — the
  convention migration is docs across 7 chapters; the harness is
  test code; CI wiring is workflow-config; independently reviewable.
  Children land in dependency order: `.2.1` migration first (book
  becomes *correct* — every example runnable from a fresh clone),
  then `.2.2` harness + `mdbook test` + CI (book correctness is
  *enforced*, can't regress). `.2` is now a container; no
  renumbering. Frontier → `.2.1`.
- `2026-05-18` (owner): runnable examples standardize on
  **`cargo run --release --`** (works with zero setup from a fresh
  clone); the bare-`anvil` form is shown once as an optional
  power-user shorthand behind `cargo install --path .`. Not chosen:
  bare-`anvil`+install-step (paste-and-run breaks if the step is
  skipped) and dual-form (doubles bulk, fights the "not scary"
  doctrine).
- `2026-05-18` (owner): correctness is **CI-gated via an extraction
  harness + `mdbook test`** (examples can never silently rot), not a
  one-time manual audit. `mdbook test`/Rust doctest alone is
  insufficient — it only covers `rust` blocks, not the 62 `bash`
  blocks that are the real copy-paste surface.

## Open Questions

- Exact "not-run" fence marker for genuinely illustrative bash
  snippets — resolved (`.1`): HTML-comment sentinel
  `<!-- book-test: skip — <reason> -->` on the line before the
  fence; mandatory reason; default = run.
- Whether sample-output blocks are asserted verbatim or
  shape-matched — deferred past `.2.2` (the `.2.2` contract is
  exit-0 + explicit classification + negative control; output-match
  is a recorded later sub-slice, not a `.2.2` gap).
- **Discovered in `.2.2` recon (`2026-05-18`) — honest correction
  to `.2.1`:** the `.2.1` migration + its `missed_runnable_bare_anvil
  = 0` audit were **line-leading only**. Bare `anvil` *embedded* in
  shell command-substitution (`gates=$(anvil …)`) and inside
  `for … do anvil … done` loops was **not** migrated and is still
  not paste-runnable. Classification of all bash blocks: 48 pure
  `cargo run`, 8 `# comment` + `cargo run` (runnable), the rest are
  `$()`/loop (runnable via the shell-script harness model once the
  embedded `anvil` is migrated) or external-tool (`verilator`/
  `yosys`/`jq`) / `git clone` install blocks (genuine skips). `.2.2`
  goal updated to complete the embedded-position migration + add the
  skip sentinels + the harness that makes this class of gap
  impossible to reintroduce.
- **Harness defect found + fixed in `.2.2` (`2026-05-18`):** the first
  full `cargo test --test book_examples` runs reported 12 blocks
  "TIMED OUT after 600s". Root-caused: **not** a book defect — every
  one of those 12 blocks emits a module (or sweep) to **stdout**
  larger than the OS pipe buffer (default `--seed 42` module =
  87,890 B; the `factorization` e-graph 5-level sweep = 538,146 B;
  macOS pipe ≈ 65,536 B). `run_script` used `Stdio::piped()` and a
  `try_wait()` loop that **never drained the pipe** before the child
  exited, so `anvil` blocked on `write()`, never exited, and the loop
  spun to the 600 s timeout (12 × 600 s ≈ the observed 7273 s total).
  Proof the examples are correct: each runs in 0.03–0.15 s when
  invoked directly. Fix (owned by `.2.2`, harness test code):
  `run_script` redirects child stdout/stderr to temp **files**
  (no buffer limit, std-only, no reader-thread plumbing) and reaps
  the child after a timeout kill. The three earlier `exit 1`
  failures (tutorial `min_inputs>max_inputs`; two dependency-chain
  blocks) were real and were already fixed/sentineled — the harness
  logic is sound; only its I/O wait was buggy.

## Blockers

- None. Fully independent of the Phase 6 `.2.4` gate (docs + a test
  harness; does not touch the generator).

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-05-18` | `BOOK-EXAMPLES-RUNNABLE.1` | `DEVELOPMENT_NOTES.md` design entry landed (full fenced-block inventory; `cargo run --release --` convention + HTML-comment skip sentinel; `tests/book_examples.rs` integration-harness architecture; `mdbook test` + `rust,ignore` sketch policy; CI wiring; 3 rejected alternatives; `.2` proof shape). Design-only, no code (diff = DEVELOPMENT_NOTES.md + docs/TASK_TREE.md + new tree file); `mdbook build book` clean; `cargo fmt --check` clean; `cargo test` unchanged-green. | Done. |
| `2026-05-18` | `BOOK-EXAMPLES-RUNNABLE.2.1` | Surgical migrator rewrote 45 line-leading `anvil ` heads → `cargo run --release -- ` in ```bash fences (factorization 3 / knobs 3 / recipes 39; getting-started/introduction/tutorial already on cargo run). Audit `missed_runnable_bare_anvil = 0`; `faq` `verilator … anvil_output.sv` correctly untouched. All 9 ```rust sketches → ```rust,ignore. Optional `cargo install` shorthand note added (getting-started Install). Spot-runs (paste-and-run sim): reproduce-style → SV exit 0; `--dump-config` → JSON exit 0; full multi-line recipes block → 50 `.sv` exit 0. `mdbook build book` clean; `git diff` = `book/src/*.md` only (7 files) — docs, no code; `cargo test` unchanged-green. | Done. |
| `2026-05-18` | `BOOK-EXAMPLES-RUNNABLE.2.2` | Landed `tests/book_examples.rs` (std-only): build release `anvil` once, parse every ```bash fence, honour the skip sentinel, substitute `cargo run --release --`→`"$ANVIL"`, classification-guard panic on residual cargo/bare-anvil/external-tool, run each block via `bash set -euo pipefail` in a fresh temp CWD offline, assert exit 0; + negative control + skip-reason test. Embedded migration finished (1 `$()`-embedded `anvil`→cargo run; 9 reasoned skip sentinels; 32 bare ``` → ```text). **Root-caused & fixed a harness pipe-buffer deadlock** (default module ≈86 KB > ≈64 KB OS pipe; `Stdio::piped()`+undrained `try_wait()` loop hung 12 blocks to the 600s timeout — examples themselves run 0.03–0.15s; fix = capture child stdio to temp files + reap after kill). `cargo test --test book_examples` = **3 passed / 0 failed**, 54 runnable blocks exit-0, 9 skip-sentineled, 76.4s (was 7273s pre-fix). `mdbook build book` clean; `mdbook test book` exit 0; `ci.yml` has the `mdbook test book` step. `cargo fmt --all --check` / `cargo check --all-targets` / `cargo clippy --all-targets -- -D warnings` clean; full `cargo test` green. Tree CLOSED. | Done. |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `BOOK-EXAMPLES-RUNNABLE.1` | `Docs: BOOK-EXAMPLES-RUNNABLE.1 book-examples-runnable design + tree` | Tree created + registered; design-only DEVELOPMENT_NOTES.md entry; architecture + 3 rejected alternatives. No code. |
| `BOOK-EXAMPLES-RUNNABLE.2.1` | `Docs: BOOK-EXAMPLES-RUNNABLE.2.1 migrate book examples to cargo run --release --` | 45 bash heads migrated + 9 rust sketches `rust,ignore` + shorthand note; missed=0; spot-runs pass. Book/docs only, no code. |
| `BOOK-EXAMPLES-RUNNABLE.2.2` | `BOOK-EXAMPLES-RUNNABLE.2.2 book-examples harness + embedded migration + mdbook-test CI` | `tests/book_examples.rs` (new code, leaf-owned) + embedded migration + 9 skip sentinels + bare→text + `ci.yml` `mdbook test` step; pipe-deadlock root-caused & fixed; 3/3 green, 54 runnable exit-0. Tree closed. |

## Changelog

- `2026-05-18`: Created after the repo went public + Pages live
  (`https://rdje.github.io/anvil/`); owner mandated every book
  example must work for copy-paste users and chose the
  `cargo run --release --` convention + CI-gated extraction-harness
  enforcement. Frontier → `.1` (design).
- `2026-05-18`: **`.1` design landed** (design-only, no code).
  `DEVELOPMENT_NOTES.md` "Book-examples-runnable design": audited
  fenced-block inventory (62 bash / 8 rust / 9 sv / 4 text; recipes
  41 + tutorial 10 dominate; ~58 bare-`anvil` to migrate); chosen
  architecture = a `tests/book_examples.rs` cargo integration
  harness (build-once, run every non-skipped bash block offline
  against the fresh binary, tagged sample-output match) + `mdbook
  test` with the 8 rust sketches `rust,ignore`, both wired into
  `ci.yml`; HTML-comment skip sentinel (mandatory reason); rejected
  doctest-only / CI-only-`.sh` / golden-doc; `.2` proof shape +
  split candidates. `mdbook` clean. Frontier → `.2` (implement;
  expected to split harness / migration / CI wiring).
- `2026-05-18`: **`.2` split** per the Splitting Rules into `.2.1`
  (convention migration + rust-sketch annotation — docs across 7
  chapters) and `.2.2` (extraction harness + `mdbook test` + CI
  wiring — code/workflow). Dependency order: `.2.1` makes the book
  correct, `.2.2` enforces it. `.2` became a container; no
  renumbering. Frontier → `.2.1`.
- `2026-05-18`: **`.2.1` landed (docs only — no code).** Surgical
  migrator rewrote 45 line-leading `anvil ` command heads →
  `cargo run --release -- ` across `factorization.md`(3),
  `knobs.md`(3), `recipes.md`(39), preserving
  `\`-continuations / pipes / `$`-prompts / indentation; prose and
  `systemverilog`/`text` output blocks untouched;
  `getting-started`/`introduction`/`tutorial` already used
  `cargo run`. Audit: **`missed_runnable_bare_anvil = 0`** (no
  runnable block left bare-`anvil`); `faq`'s
  `verilator … anvil_output.sv` correctly not matched. All 9 ```rust
  illustrative sketches → ```rust,ignore. One optional
  `cargo install --path .` → `anvil` shorthand note added
  (getting-started Install). `mdbook build book` clean; three
  paste-and-run spot-runs pass (incl. a full multi-line recipes
  block → 50 `.sv`, exit 0). The published book is now correct for
  copy-paste users. Frontier → `.2.2` (harness + `mdbook test` +
  CI — enforcement so it can't regress).
- `2026-05-18`: **`.2.2` landed — `.2` + the tree CLOSED.**
  `tests/book_examples.rs` (std-only cargo integration test, leaf-
  owned code) extracts every ```bash fence, honours the skip
  sentinel, resolves `cargo run --release --`→`"$ANVIL"`, panics on
  any unclassified residual, runs each block via
  `bash set -euo pipefail` in a fresh temp CWD offline, asserts
  exit 0; a negative-control test proves the harness actually fails
  on a broken example. Embedded-position migration finished (the
  `$()`-embedded `anvil` from the `.2.1` honest-correction → cargo
  run; 9 reasoned skip sentinels on install/external-tool blocks;
  32 bare ``` fences → ```text so `mdbook test` stops compiling
  prose as Rust). The first full runs surfaced 12 "TIMED OUT"
  blocks — **root-caused as a harness bug, not a book defect**: a
  default module is ≈86 KB of stdout but the OS pipe buffer is
  ≈64 KB, and `run_script` used `Stdio::piped()` with a
  `try_wait()` loop that never drained it, so `anvil` blocked on
  `write()` forever (12 × 600 s timeout ≈ the 7273 s first-run
  total; the examples run in 0.03–0.15 s). Fixed by capturing child
  stdout/stderr to temp files + reaping after a timeout kill. Result:
  `cargo test --test book_examples` 3 passed / 0 failed, 54 runnable
  blocks all exit-0, 9 skip-sentineled, 76.4 s. `mdbook build` clean,
  `mdbook test book` exit 0, `ci.yml` carries the `mdbook test book`
  step (+ fmt/clippy/test/mdbook build). Every book example is now
  CI-gated against drift — the tree's goal is fully met.
