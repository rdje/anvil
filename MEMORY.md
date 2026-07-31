# MEMORY - resume pointer (layer A; overwrite-only, keep <= 50 lines)

## How to resume
- Read `README.md`, then `MEMORY_ARCHITECTURE.md`.
- Work is task-tree tracked under `docs/tasks/`; index: `docs/TASK_TREE.md`.
- Durable cross-task facts live in `docs/decisions/`.
- Question-keyed retrieval facts are indexed in `KNOWLEDGE_MAP.md`.
- Commit completed leaves per `COMMIT.md`; include the leaf id in the subject.

## Current state (OVERWRITE this block; do not append history — that is git + the task trees)
- latest_commit: **`OVERFLOW-DESTINATION-INSTRUMENTATION.6`** (backfill next commit). Prior: `f33601e`+`e801617` (`.3`), `6c0c953` (`.5b`), `4f3d508` (`.5a` = decision `0041`), `4a91c45` (`.2` = decision `0040`). Older commits NOT listed (`MEMORY_ARCHITECTURE.md` §12) — use `git log --oneline`.
- active_work_unit: **`OVERFLOW-DESTINATION-INSTRUMENTATION`** (`active`; `.1`/`.2`/`.5a`/`.5b`/`.3`/`.6` done, frontier **`.7`**). **This file is inside its own instrument** — 30 lines / ~6.0 KB against caps of 50 / **6,144**, down from 19,885 B. `0040` classifies every overflow destination in **three** kinds and measures that **a cap binds only on the axis it measures**; `.3` added the byte axis. **`.6` then found what no size cap can see: 36 malformed table rows silently drop 57,283 rendered chars tree-wide, and 97.0 % of the 24,990-byte line this tree has cited since `.1` has never rendered at all.** **Read the records, not a summary here.** Also `active`: **`CHANGES-ENTRY-PLACEMENT`** (frontier `.4`) · **`IR-TYPES-DECOMPOSITION`** (`.3`/`.4`, the big code leaf).
- next_action: **`OVERFLOW-DESTINATION-INSTRUMENTATION.7`** — repair the 36 rows (one `\|` per offending pipe; row 131's File link is duplicated) and gate the class **escape-aware + fence-aware**; argue rather than assume whether the 29 rows in append-only `docs/tasks/` are in scope. Then **`.8`** (classify `docs/TASK_TREE.md` — `0040` never did and neither router names it; **648 B/line**, 79 % in one column), then **`.4`** (carry the correction into the **portable** standards — `MEMORY_ARCHITECTURE.md` §6/§9 still ship the single-axis hole `.3` closed locally). Then **`CHANGES-ENTRY-PLACEMENT.4`**, then **`IR-TYPES-DECOMPOSITION.3`**. **Every commit subject must name a LEAF** (`TREE.N`) — the `commit-msg` hook rejects a bare tree name.
- in_flight_uncommitted: none once `.6` lands. Docs only ⇒ **DUT byte-identical**; `src/`, `tests/`, `examples/` untouched. `check_doctrines.sh` **8/8** after `git add`; KM **107** facts / **1,029** keys.
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
