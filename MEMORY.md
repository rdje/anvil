# MEMORY - resume pointer (layer A; overwrite-only, keep <= 50 lines)

## How to resume
- Read `README.md`, then `MEMORY_ARCHITECTURE.md`.
- Work is task-tree tracked under `docs/tasks/`; index: `docs/TASK_TREE.md`.
- Durable cross-task facts live in `docs/decisions/`.
- Question-keyed retrieval facts are indexed in `KNOWLEDGE_MAP.md`.
- Commit completed leaves per `COMMIT.md`; include the leaf id in the subject.

## Current state (OVERWRITE this block; do not append history — that is git + the task trees)
- latest_commit: **`IR-TYPES-DECOMPOSITION.3`** (backfill next commit). Prior: `dd3aadc` (`ODI.10`), `567d284` (`ODI.8` = decision `0042`), `6b00357` (`ODI.7`), `bb446fc` (`ODI.6`). Recent decisions: **`0040`–`0042`**. Older commits NOT listed (`MEMORY_ARCHITECTURE.md` §12) — use `git log --oneline`.
- active_work_unit: **`IR-TYPES-DECOMPOSITION`** (`active`; `.1`/`.2`/`.3` done, frontier **`.4`**). `.3` extracted the canonicalization engine to `src/ir/intern.rs` — **`types.rs` 3607 → 1485**, verbatim proven by whole-file line census (residue **0**), `#[test]` **40 → 3 + 37**, blast radius exactly `src/ir/`, **1,058** tests pass, **no `.snap` rewritten ⇒ DUT byte-identical**. Also `active`: **`OVERFLOW-DESTINATION-INSTRUMENTATION`** (`.9`/`.4` pending — **PAUSED by owner redirect, do not resume without a nudge**) · **`CHANGES-ENTRY-PLACEMENT`** (`.4`, deferred).
- next_action: **owner redirect `2026-08-01`: *"switch back to feature related work."*** So: **`CAPABILITY-BREADTH-EXPANSION.1`** — pick the next version-distinctive SV up-opt after `union soft` (enum/typedef, packed multi-dim arrays, …), grounded in a fresh tool probe + the local LRM cache, pinning its default-off knob + `sv_version` gate + down-gate fallback + `num_emitted_*` metric + MCP selectability (decision `0017`). `IR-TYPES-DECOMPOSITION.4` (re-measure + close) is a small tail and may be folded in later. **Every commit subject must name a LEAF** (`TREE.N`) — the `commit-msg` hook rejects a bare tree name.
- in_flight_uncommitted: none once `.3` lands. **Never trust a piped exit status** — `cargo test | tail` reports *tail's* status; redirect to a file and read `$?` (this bit `.3`).
- blockers: none. **Adding to this file now requires routing, not appending** — that is the instrument working as designed.

## Standing directives (owner-set; violating these is worse than not shipping)
- **They are RECORDED, not summarised here.** Read both before acting: [`docs/decisions/0041`](docs/decisions/0041-owner-standing-directives-recorded-in-layer-c.md) — *a defect is only handled if a task-tree owns it* · *decide, don't ask* · *`~/Documents/github` is owner-owned*; and [`docs/decisions/0031`](docs/decisions/0031-ssd-volume-exclusivity.md) — *never rewrite history* · *the SSD is the only project volume* · *shared means shared* · *harness runtime files belong to the harness*.
- **Why a pointer and not a copy:** this file is layer **A** — *overwritten* on every update with a hard cap (`MEMORY_ARCHITECTURE.md` §3/§6) — so a directive stored here is queued for deletion, which is precisely what directive one warns about. 13 citations in 9 decision records used to point here; they cite by **owner + date**, so they now resolve to `0041`/`0031` unchanged.

## Operating gotchas (earned the hard way — do not relearn)
- **They are RECORDED as Knowledge Map fact cards, not summarised here.** Retrieve by *question* from [`KNOWLEDGE_MAP.md`](KNOWLEDGE_MAP.md); enumerate with `grep -l 'gotcha' docs/knowledge/*.md` — a **derivation**, deliberately not a list, since a hand-kept list of card names is the exact shadow [`0033`](docs/decisions/0033-shadow-enumeration-classification.md) repairs by deletion.
- **Search before you act.** A card exists because someone already paid for the lesson; re-deriving it from source is the archaeology the map exists to eliminate.

## Lane invariants (all lanes)
- **Recorded in layer C and the book, not here.** Rules-first / valid-by-construction / no generate-then-filter → [`book/src/by-construction.md`](book/src/by-construction.md) + [`algorithm.md`](book/src/algorithm.md). No retirement (`feedback_never_retire_strategies`) and one mechanism never two (`feedback_full_factorization`) → decisions [`0006`](docs/decisions/0006-signoff-automation-first-increment.md)/[`0007`](docs/decisions/0007-identity-deepening-first-extension.md). Opt-in + MCP-invocable + queryable, CLI-as-shim, API for agents → [`0017`](docs/decisions/0017-api-first-everything-mcp-accessible.md). SCHEMA-DERIVED / no shadow simulator → [`0004`](docs/decisions/0004-artifact-lane-separation.md)/[`0011`](docs/decisions/0011-semantic-introspection-derived-query-surface.md). Book must not drift (`feedback_book_doctrine`) → `COMMIT.md` §9. Every doctrine mechanically gated → [`0026`](docs/decisions/0026-doctrine-enforcement-adoption.md) + [[doctrine-enforcement]].
- **Default-off + byte-identical wherever output could change; `tests/snapshots.rs` untouched by default.** Kept here: it is the precondition every code leaf is checked against before it starts.

## Validation policy
- **Recorded in decision [`0003`](docs/decisions/0003-resource-safe-validation.md)** — heavy builds run under `scripts/ram_guard.sh --threshold 90 -- <cmd>` (note the `--`), stop above 90 % RAM and record it as an *environment* stop not a product failure; workflow/memory/retrieval doc leaves may use focused functional checks, a full `cargo test` not being required for them per owner instruction.
