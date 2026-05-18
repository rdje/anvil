# BOOK-EXAMPLES-RUNNABLE: every mdBook example is copy-paste runnable and drift-proof

## Metadata

- Tree ID: `BOOK-EXAMPLES-RUNNABLE`
- Status: `active`
- Roadmap lane: Quality — user-facing book correctness
- Created: `2026-05-18`
- Last updated: `2026-05-18` (`.1` design landed; frontier → `.2`)
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
  Status: `active`
  Goal: `Make every mdBook example copy-paste runnable from a fresh clone and CI-enforced against drift.`
  Children: `BOOK-EXAMPLES-RUNNABLE.1` (design), `BOOK-EXAMPLES-RUNNABLE.2` (implement)

- ID: `BOOK-EXAMPLES-RUNNABLE.1`
  Status: `done`
  Goal: `Design (DEVELOPMENT_NOTES.md): a codebase-grounded audit of all book/src fenced blocks (62 bash, 8 rust, 9 systemverilog, 4 text — exact per-chapter inventory + classify each bash block runnable-vs-illustrative); the anvil→cargo-run migration map; the extraction-harness design (fence parsing, a recognised "not-run" marker for illustrative blocks, fresh-build binary, per-command timeout, offline/no-network constraint, sample-output-match policy where a fenced output block follows a command, exit-code policy); the mdbook-test wiring + rust-sketch annotation policy; CI integration point; >=1 rejected alternative; proof shape. Design-only; no code; mdbook clean.`
  Acceptance: `DEVELOPMENT_NOTES.md "Book-examples-runnable design" entry with the full block inventory, the convention/marker design, the harness architecture, >=1 rejected alternative, and the .2 proof shape; no code change; mdbook build clean.`
  Verification: `DEVELOPMENT_NOTES.md "Book-examples-runnable design (2026-05-18, BOOK-EXAMPLES-RUNNABLE.1)" entry landed: audited fenced-block inventory (62 bash / 8 rust / 9 systemverilog / 4 text with exact per-chapter counts — recipes.md 41 + tutorial.md 10 dominate the bash surface; ~58 bare-anvil occurrences; getting-started already on cargo run, the rest not — the core defect); owner decisions recorded (cargo run --release -- convention + CI-gated harness); chosen architecture (cargo integration test tests/book_examples.rs that builds once + runs every non-skipped bash block against the fresh binary, offline, with timeout + tagged sample-output match, PLUS mdbook test with the 8 rust sketches annotated rust,ignore, both wired into ci.yml); HTML-comment skip sentinel design (mandatory reason, invisible to readers, default=run); 3 rejected alternatives (doctest-only / CI-only .sh / golden-doc generation); .2 proof shape incl. negative control + split candidates. Design-only; no code change; git diff = DEVELOPMENT_NOTES.md + docs/TASK_TREE.md only; mdbook build clean; cargo fmt --check clean; cargo test unchanged-green (no src/tests touched).`
  Commit: `Docs: BOOK-EXAMPLES-RUNNABLE.1 book-examples-runnable design + tree`

- ID: `BOOK-EXAMPLES-RUNNABLE.2`
  Status: `pending`
  Goal: `Implement per .1: migrate every runnable bash block to cargo run --release -- (+ one documented optional 'cargo install --path . → anvil shorthand' note); annotate rust sketches; land the extraction+run harness as a cargo integration test (so cargo test + CI cover it) + wire mdbook test; make CI gate on both. Expected to split into signoff-sized leaves when reached (harness vs migration vs CI wiring review independently).`
  Acceptance: `Harness runs every runnable bash block against a fresh build and passes; mdbook test green; CI gates on both; cargo fmt/clippy/test green; book unchanged in meaning, only examples normalised.`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `BOOK-EXAMPLES-RUNNABLE.2` | `pending` | `.1` design done (inventory + `cargo run --release --` convention + skip-sentinel + `tests/book_examples.rs` harness + `mdbook test` rust-sketch policy + CI wiring; 3 rejected alternatives). `.2` implements it; expected to split (harness impl / ~62-block migration / CI wiring review independently). Independent of the running Phase 6 gate. |

## Decisions

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
  snippets (e.g. a trailing `# illustrative` or an HTML-comment
  sentinel the harness recognises) — owner: `.1` design.
- Whether sample-output blocks are asserted verbatim or
  shape-matched (seed-stable output can be exact; tool-version-
  sensitive output shape-matched) — owner: `.1` design.

## Blockers

- None. Fully independent of the Phase 6 `.2.4` gate (docs + a test
  harness; does not touch the generator).

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-05-18` | `BOOK-EXAMPLES-RUNNABLE.1` | `DEVELOPMENT_NOTES.md` design entry landed (full fenced-block inventory; `cargo run --release --` convention + HTML-comment skip sentinel; `tests/book_examples.rs` integration-harness architecture; `mdbook test` + `rust,ignore` sketch policy; CI wiring; 3 rejected alternatives; `.2` proof shape). Design-only, no code (diff = DEVELOPMENT_NOTES.md + docs/TASK_TREE.md + new tree file); `mdbook build book` clean; `cargo fmt --check` clean; `cargo test` unchanged-green. | Done. |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `BOOK-EXAMPLES-RUNNABLE.1` | `Docs: BOOK-EXAMPLES-RUNNABLE.1 book-examples-runnable design + tree` | Tree created + registered; design-only DEVELOPMENT_NOTES.md entry; architecture + 3 rejected alternatives. No code. |

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
