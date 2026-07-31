# MEMORY - resume pointer (layer A; overwrite-only, keep <= 50 lines)

## How to resume
- Read `README.md`, then `MEMORY_ARCHITECTURE.md`.
- Work is task-tree tracked under `docs/tasks/`; index: `docs/TASK_TREE.md`.
- Durable cross-task facts live in `docs/decisions/`.
- Question-keyed retrieval facts are indexed in `KNOWLEDGE_MAP.md`.
- Commit completed leaves per `COMMIT.md`; include the leaf id in the subject.

## Current state (OVERWRITE this block; do not append history — that is git + the task trees)
- latest_commit: **`OVERFLOW-DESTINATION-INSTRUMENTATION.5b`** (pending — backfill next). Prior: `4f3d508`+`62f9e21` (`.5a`), `4a91c45` (`.2` = decision `0040`), `8473821` (`.6`). Recent decisions: **`0039`–`0041`**. Older commits NOT listed (`MEMORY_ARCHITECTURE.md` §12) — use `git log --oneline`.
- active_work_unit: **`OVERFLOW-DESTINATION-INSTRUMENTATION`** (`active`; `.2`/`.5a`/`.5b` done, frontier **`.3`**). **This file is the subject.** Decision [`0040`](docs/decisions/0040-overflow-destination-classification-and-the-unmeasured-axis.md) classifies every overflow destination in **three** kinds and measures that a cap binds only on the axis it measures — `MEMORY.md` held at exactly 50/50 lines while bytes went **×16.6** and density **64→406 B/line**. `.5a`+`.5b` relocated the layer-C content out (decision [`0041`](docs/decisions/0041-owner-standing-directives-recorded-in-layer-c.md) + 12 fact cards). **Read the records, not a summary here.** Also `active`: **`CHANGES-ENTRY-PLACEMENT`** (frontier `.4`; 181 of 574 provenance lines never hash-backfilled) · **`IR-TYPES-DECOMPOSITION`** (`.3`/`.4` — the ~1830-line interning engine, the big code leaf).
- next_action: **`OVERFLOW-DESTINATION-INSTRUMENTATION.3`** — add the derived **6,144-byte** cap to `scripts/check_memory_architecture.sh` as a **second assertion**, NOT a new doctrine (`MEMORY-ARCH` already owns this file). Negative-control it **both ways** and prove the control *can* fail before trusting it ([[negative-control-must-be-able-to-fail]]). Then **`.4`** (carry into the portable policy: *instrument every axis of a bounded surface, or the cap measures the one thing that is not growing*), **`.6`**, **`CHANGES-ENTRY-PLACEMENT.4`**, **`IR-TYPES-DECOMPOSITION.3`**. **Every commit subject must name a LEAF** (`TREE.N`) — the `commit-msg` hook rejects a bare tree name.
- in_flight_uncommitted: none once `.5b` lands. Docs-only ⇒ **DUT byte-identical**; `src/`, `tests/`, `examples/` untouched. `cargo check --all-targets` clean; `check_knowledge_map` OK (**106** facts / **1,022** keys); `check_doctrines.sh` **8/8** after `git add`.
- blockers: none. **`MEMORY.md` is now within its derived cap** — see the leaf's Verification Log for the measured before/after and the loss-proof residue.

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
