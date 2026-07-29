# MEMORY - resume pointer (layer A; overwrite-only, keep <= 50 lines)

## How to resume
- Read `README.md`, then `MEMORY_ARCHITECTURE.md`.
- Work is task-tree tracked under `docs/tasks/`; index: `docs/TASK_TREE.md`.
- Durable cross-task facts live in `docs/decisions/`.
- Question-keyed retrieval facts are indexed in `KNOWLEDGE_MAP.md`.
- Commit completed leaves per `COMMIT.md`; include the leaf id in the subject.

## Current state (OVERWRITE this block; do not append history — that is git + the task trees)
- latest_commit: **`EVIDENCE-BANK-DURABILITY.4`** — closes that tree; `EVIDENCE-CITATIONS` live at driver **6/6**, first digest banked, breadcrumb pointers in place. Session history: `git log --oneline` (do not re-narrate it here).
- active_work_unit: **`EMIT-SURFACE-INTERACTION-GATE`** (registered, not started) → frontier `.1`. Closed this session: `CARGO-TMPDIR-SWEEP-REGRESSION`, `EVIDENCE-BANK-DURABILITY`.
- next_action: `EMIT-SURFACE-INTERACTION-GATE.1` — the design ADR. Measured gap: all 8 `*_emit_prob` focus configs set exactly ONE surface to 1.0, so the nine passes' mutual-exclusion predicates always see an empty sibling set and are never exercised; and `--profile structured-emission-max` sets 4 of 9. Bank the gate's evidence as the second `docs/evidence/` digest.
- in_flight_uncommitted: none. Tree clean, self-checks green, resume pointer current.
- blockers: none.

## Standing directives (owner-set; violating these is worse than not shipping)
- **DECIDE, DON'T ASK** (`2026-07-30`, owner-set). *"You are an elite coder and you know the roadmap and the objectives of ANVIL, so take SOTA and signoff-level decisions whenever you can."* Pick the next work unit, narrow or supersede a decision record, choose the mechanism — autonomously, and record the reasoning where it survives. Escalate only when proceeding under any assumption would be unsafe or would make the work useless if wrong. Surfacing a finding is still mandatory; turning it into a **question** is not.
- **NEVER REWRITE HISTORY** (`2026-07-29`, absolute, decision `0031`). No `rebase` / `--amend` of landed commits / `reset --hard` / `filter-branch` / force-push. `CHANGES.md` + `DEVELOPMENT_NOTES.md` are **never** retro-edited — append only; their pre-`0031` `/tmp` references stay raw. Owner: *"Keep it raw, keep honest, so that people can follow the whole history."* A swept history is a dishonest history.
- **THE SSD IS THE ONLY PROJECT VOLUME** (decision `0031`). ANVIL stores nothing on, and no live doc/code points at, `/tmp` · `/private/tmp` · `$TMPDIR` (macOS `/var/folders/…`, which a `"/tmp"` grep MISSES). Repo = `/dev/disk5s1`; boot = `/dev/disk3s1s1`. All ANVIL scratch resolves through `crate::paths::sandbox_root()` → `.cache/anvil-sandbox/`. Enforced by `NO-BOOT-VOLUME-REFS`; its **allow-list is load-bearing** (policy docs must name what they forbid; history must stay raw) — never "tighten" it into a history rewrite.
- **SHARED MEANS SHARED — use it, never duplicate.** `~/.cargo`, `~/.rustup`, `/opt/homebrew` tools stay in place; `CARGO_HOME`/`RUSTUP_HOME` at defaults. Owner: *"if something is shared, use the shared information."* Forking a shared cache costs disk twice and creates a second source of truth.
- **`~/Documents/github` is owner-owned** — the pre-move checkouts. No agent deletes or migrates it; the owner removes it themselves. Excluded from every audit.
- Harness runtime files (`/private/tmp/claude-501/…`, `~/.claude/…`) belong to the harness and cannot be moved from the repo — never *depend* on them; redirect real output to `.cache/scratch/`.

## Operating gotchas (earned the hard way — do not relearn)
- **Never clear `.cache/anvil-sandbox` while a test run is in flight.** It is the live sandbox root now, so clearing it deletes running tests' working dirs *and* their stdout/stderr capture files — surfacing as a `book_examples` failure with **empty** output. Clear before or after, never during. A failure with no captured output at all ⇒ suspect the harness's scratch before the code.
- **Never mass-rewrite strings across docs whose *subject* is that string.** A blanket `/tmp` sweep once turned decision `0030`'s own `reverify` into the meaningless `ls -d anvil-*`. Always allow-list the policy/history documents first.
- **`git checkout -- <file>` during a negative-control sweep discards unrelated edits to that file.** It silently cost a README citation that a commit message then claimed; `CHANGES.md` is append-only so the fix is to make the claim true in the next leaf, never to rewrite it. Re-read `git diff --cached --name-only` before writing commit prose.
- **The fixture agrees with you; the tool does not.** Three independent instances now (`DIFFERENTIAL-SIMULATION.3b.2` two-space `input  logic`; `PHASE-7 .2c.2b.1` `rem_euclid` vs `%`; `EVIDENCE-BANK-DURABILITY.5` a deriver reporting *2907 gaps* for an empty array). Run a new tool against **real** output before trusting it — a hand-made fixture is shaped by the same assumption as the parser.
- **`grep -E` uses ERE: the interval is `{0,1}`, NOT `\{0,1\}`.** `sed` is BRE and wants the backslashes. This repo's scripts mix both dialects; the escaped form inside `grep -E` matches a literal brace and silently never fires. An extractor must die on a missing field, never fall through to something plausible.
- **A doctrine check must classify, never guess.** `EVIDENCE-CITATIONS` requires every `anvil-<name>` token to be in one of three buckets and fails closed on an unknown one; a heuristic would either miss a real uncited bank or cry wolf on prose, and a gate that cries wolf gets deleted. Its grandfathered list is pinned by count+SHA **because its membership is a historical fact** — unpinned, "just grandfather it" bypasses the doctrine in one line.
- **Run `scripts/check_doctrines.sh` AFTER `git add`.** `git grep`/`git ls-files` see tracked content only, so a new file's contents are invisible to every structural check until staged. Three self-catches in two days were all this blind spot.
- **Anchor a path-prefix rewrite to a path start.** The same sweep, applied unanchored, also fired *inside* `target/tmp/…` (Cargo's `CARGO_TARGET_TMPDIR`, on-volume) and minted 10 paths to a directory that never existed. A wrong path that looks plausible survives review. And a gate built from the sweep's own search string shares its blind spot — write the check from the property ("a boot-volume path is absolute"), not the string.
- `verilator -Wall` on a fixed-filename dump always fails `DECLFILENAME` — pass `-Wno-DECLFILENAME`; it is a filename artifact, not a defect.

## Lane invariants (all lanes)
- Rules-first / no generate-then-filter (valid-by-construction); default-off + byte-identical wherever output could change; `tests/snapshots.rs` untouched by default.
- **No retirement** (`feedback_never_retire_strategies`); one runner + one classifier, never two (`feedback_full_factorization`).
- Every capability opt-in + MCP-invocable + queryable, CLI-as-shim (decision `0017`); design the API for agents, not humans (`feedback_api_for_agents_not_humans`).
- SEMANTIC-INTROSPECTION stays SCHEMA-DERIVED / no shadow simulator (decisions `0004`/`0011`).
- The book is the user-facing surface and must not drift (`feedback_book_doctrine`); a decision record or KM fact per durable capability or boundary.
- Doctrine-enforcement (decision `0026`): every doctrine is a deterministic check in `scripts/check_doctrines.sh`; code-scoped checks are scope-aware (non-code commits exempt).

## Validation policy
- Heavy builds run under `scripts/ram_guard.sh --threshold 90 -- <cmd>` (note the `--` separator); stop above 90% RAM and record it as an environment stop (decision `0003`).
- Workflow/memory/retrieval doc leaves may use focused functional checks; a full `cargo test` is not required for them per owner instruction.
