# MEMORY - resume pointer (layer A; lifecycle `bounded_snapshot`; overwrite-only, <= 50 lines / 6,144 B)

## How to resume
- Read `README.md`, then `MEMORY_ARCHITECTURE.md`.
- Work is task-tree tracked under `docs/tasks/`; index: `docs/TASK_TREE.md`.
- Durable cross-task facts live in `docs/decisions/`.
- Question-keyed retrieval facts are indexed in `KNOWLEDGE_MAP.md`.
- Commit completed leaves per `COMMIT.md`; include the leaf id in the subject.
- **Where you are in history is DERIVED, never written here:** `git log -1 --oneline` for HEAD, `git log --oneline` for what preceded it, `ls docs/decisions/` for the decision set. A hand-copied `latest_commit` is **stale on arrival** — it can only name the commit *before* the one that writes it ⇒ `0051`.

## Current state (OVERWRITE this block; do not append history — that is git + the task trees)
- active_work_unit: **`LIVE-DOCUMENT-SIZE-CONTAINMENT-ADOPTION`** → frontier **`.8b`**. **ONE unit, never a roster**, per `MEMORY_ARCHITECTURE.md` §6's singular template; the roster of every `active` tree is [`docs/TASK_TREE.md`](docs/TASK_TREE.md)'s job ⇒ `0050`.
- next_action: **ONE action, never a queue** — `.8b`: **`git rm CHANGES.md`** and stop writes (owner directive `2026-08-08`; `.8a2` measured **918 of 5,958** pre-era words uncovered by *every* durable layer — ordinary English plus ~15 `src/` identifiers — so **delete, not seal**). Salvage those identifiers first. Two doctrines retire with it; the evidence leg collapses onto the task leaf. Reachability precondition **met** — owner granted an *exceptional* push `2026-08-08` ⇒ `0041` §(d.1); the **200-commit cadence is unchanged**. What comes *after* is not written here: cross-tree order is `docs/TASK_TREE.md`'s first `active` row (its own §PNT Selection Rules), within-tree order is each tree's `Current Frontier` ⇒ `0051`.
- in_flight_uncommitted: none. **`.cache/local-references/` is NEVER tracked** ⇒ `0043`. **Never trust a piped exit status.**
- blockers: none. Gotchas are **cards, not summaries** — retrieve by *question* from `KNOWLEDGE_MAP.md`; enumerate with `grep -l 'gotcha' docs/knowledge/*.md` (a **derivation**, `0033`). **Adding to this file requires routing, not appending** — and the routing hint means it: *move content down, do not trim prose*.

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
