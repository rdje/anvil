# MEMORY - resume pointer (layer A; overwrite-only, keep <= 50 lines)

## How to resume
- Read `README.md`, then `MEMORY_ARCHITECTURE.md`.
- Work is task-tree tracked under `docs/tasks/`; index: `docs/TASK_TREE.md`.
- Durable cross-task facts live in `docs/decisions/`.
- Question-keyed retrieval facts are indexed in `KNOWLEDGE_MAP.md`.
- Commit completed leaves per `COMMIT.md`; include the leaf id in the subject.

## Current state (OVERWRITE this block; do not append history — that is git + the task trees)
- latest_commit: **`beb1ebb`** (`CHANGES-ENTRY-PLACEMENT.5`; backfill next commit). Prior: `8c93d1e`, `1cd46c3`, `6ca3c29`. Recent decisions: **`0040`–`0045`**. Older commits NOT listed (`MEMORY_ARCHITECTURE.md` §12) — use `git log --oneline`.
- active_work_unit: **none mid-flight; THREE trees CLOSED this session.** `CHANGES-ENTRY-PLACEMENT` **closed** — `0045` is live as the **TENTH** registered doctrine (driver **10/10**); the historical scan (768 commits, same 3 fires) and all three controls were **reproduced on the real staged path**, plus three more and a `0037` vacuity probe. Also closed: the `unique`/`priority` **case-qualifier** strand (decision **`0044`**, schema **`1.28`**) and **`IR-TYPES-DECOMPOSITION`** (`types.rs` **4,069 → 1,473**). Still `active`: **`USER-GUIDE-CLI-TABLE-SHADOW`** (`.2`) · **`CAPABILITY-BREADTH-EXPANSION`** (only `.1`, SV up-opt, **deferred-not-retired**) · **`OVERFLOW-DESTINATION-INSTRUMENTATION`** (`.9`/`.4` — **PAUSED by owner redirect, do not resume without a nudge**).
- next_action: **`USER-GUIDE-CLI-TABLE-SHADOW.2`** — **decide the CLI flag table's contract BEFORE writing a row** (exhaustive vs curated); 20 flags are mentioned in prose but not tabled, so repairing first produces a table nobody can maintain. Read the tree's `.2` Acceptance. **Every commit subject must name a LEAF** (`TREE.N`).
- in_flight_uncommitted: none. **`.cache/local-references/` is NEVER tracked** — public repo + IEEE copyright, and `0031` forbids the usual remedy ⇒ [`0043`](docs/decisions/0043-reference-cache-stays-untracked-public-repo.md). **Never trust a piped exit status** — `cargo test | tail` reports *tail's*.
- blockers: none. **A finding is not closed until something MECHANICAL fails if it recurs** — `PARITY-EXTRACTOR-CHARSET-GAP` (closed `2026-08-01`): `.1` fixed the values and wrote prose, which caught nothing; `.2` added `total_or_fail` (an extractor must account for every item it walks) so a silent skip is a hard failure. When a delimiter exists, capture `[^"]+` — the delimiter IS the spec. **A decision you already made is DISCLOSED, never re-opened as a question** — [`0041`](docs/decisions/0041-owner-standing-directives-recorded-in-layer-c.md) §(e). **Adding to this file requires routing, not appending.**

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
