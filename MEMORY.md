# MEMORY - resume pointer (layer A; overwrite-only, keep <= 50 lines)

## How to resume
- Read `README.md`, then `MEMORY_ARCHITECTURE.md`.
- Work is task-tree tracked under `docs/tasks/`; index: `docs/TASK_TREE.md`.
- Durable cross-task facts live in `docs/decisions/`.
- Question-keyed retrieval facts are indexed in `KNOWLEDGE_MAP.md`.
- Commit completed leaves per `COMMIT.md`; include the leaf id in the subject.

## Current state (OVERWRITE this block; do not append history — that is git + the task trees)
- latest_commit: **`OVERFLOW-DESTINATION-INSTRUMENTATION.8`** (backfill next commit). Prior: `6b00357` (`.7`), `bb446fc` (`.6`), `f33601e`+`e801617` (`.3`), `6c0c953` (`.5b`), `4f3d508` (`.5a`). Recent decisions: **`0040`–`0042`**. Older commits NOT listed (`MEMORY_ARCHITECTURE.md` §12) — use `git log --oneline`.
- active_work_unit: **`OVERFLOW-DESTINATION-INSTRUMENTATION`** (`active`; `.1`/`.2`/`.5a`/`.5b`/`.3`/`.6`/`.7`/`.8` done, frontier **`.9`**). **This file is inside its own instrument** — 30 lines / ~6.0 KB against caps of 50 / **6,144**, down from 19,885 B. `0040` classifies destinations in **three** kinds; `.3` added the byte axis; **`.6` found what no size cap can see (36 malformed table rows dropping 57,283 *rendered* chars), `.7` repaired all 36 + registered the NINTH doctrine `TABLE-RENDER-FIDELITY`, and `.8` classified `docs/TASK_TREE.md` MIXED — decision `0042`, superseding `0040` §(f).** **Read the records, not a summary here.** Also `active`: **`CHANGES-ENTRY-PLACEMENT`** (frontier `.4`) · **`IR-TYPES-DECOMPOSITION`** (`.3`/`.4`, the big code leaf).
- next_action: **`OVERFLOW-DESTINATION-INSTRUMENTATION.9`** — **separate** the per-leaf journal out of `docs/TASK_TREE.md`'s *Current frontier* column back to the tree files that already own **81.9 %** of it. **Prove duplication PER ROW before moving anything** (`0042`; the `.5a` lesson — a coarse probe reports *"has a home"* for hits pointing back at the source); separation **before** any cap. Then **`.4`** (carry the correction into the **portable** standards — `MEMORY_ARCHITECTURE.md` §6/§9 still ship the single-axis hole `.3` closed locally, and `0040` §(g) now has **two** instances). Then **`CHANGES-ENTRY-PLACEMENT.4`**, then **`IR-TYPES-DECOMPOSITION.3`**. **Every commit subject must name a LEAF** (`TREE.N`) — the `commit-msg` hook rejects a bare tree name.
- in_flight_uncommitted: none once `.8` lands. Docs + `scripts/` only ⇒ **DUT byte-identical**; `src/`, `tests/`, `examples/` untouched. `check_doctrines.sh` **9/9** after `git add`; KM **107** facts / **1,029** keys.
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
