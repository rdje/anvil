# SHADOW-ENUMERATION-SWEEP: hand-maintained lists that shadow a set which already exists

## Metadata

- Tree ID: `SHADOW-ENUMERATION-SWEEP`
- Status: `active`
- Roadmap lane: Quality / defect-class elimination (cross-cutting; no phase reopened)
- Created: `2026-07-30`
- Last updated: `2026-07-30` (`.1`/`.2`/`.3`/`.4`/`.5`/`.6` done — decision
  [`0033`](../decisions/0033-shadow-enumeration-classification.md); frontier `.7`)
- Owner: repo-local workflow (owner-directed `2026-07-30`: *"the only way to have any
  defect handled is to have it task-tree tracked"*)

## Goal

Find and eliminate the repo's **shadow enumerations**: hand-maintained lists that
mirror a set which is already authoritative somewhere else, and which therefore
fall silently out of date when that set grows.

The outcome this tree must deliver is not "fix three files". It is a recorded
answer to *"which shadow enumerations does this repo still contain, which of them
fail **silently**, and what is the standard repair for each class?"* — with the
silent ones either derived from the real set or made loud.

## Why this is a tree and not a `MEMORY.md` note

It was recorded in `MEMORY.md` first. That was wrong, and the owner caught it.

`MEMORY_ARCHITECTURE.md` §71 defines layer A as **"Overwritten each update; hard
size cap."** A finding parked there is not stored — it is *queued for deletion* by
the next state refresh, which in this repo is the next commit. The layers that
persist are B (task trees), C (decision records) and the git history. A defect that
lives only in the resume pointer is a defect nobody will ever be asked to fix.

Owner directive, `2026-07-30`: **the only way to have a defect handled is to have it
task-tree tracked.** Recorded here rather than only obeyed.

## Observation that opened this tree (`2026-07-30`, measured)

Three independent bugs of one shape landed in a single session, each in a different
language and subsystem:

| where | the shadow list | the real set it shadowed | how it failed |
| --- | --- | --- | --- |
| `--profile structured-emission-max` (`src/config.rs`) | 4 hand-listed `*_emit_prob` knobs | the `structured_emission` group of `knob_catalog()` | surfaces 6–9 shipped; the preset was never updated; it set 4 knobs and **emitted 1** |
| `cone_function_emit::compute_use_counts` | hand-listed consumer kinds | the `NodeId`-bearing fields of `Module` | `Memory`/`Fsm` ports were added; the census was not; absorption became unsound-in-waiting |
| `scripts/evidence_digest.sh` | 14 hand-listed `*_gate` report fields | the `"*_gate": true` keys in the report itself | a 15th gate shipped; the deriver emitted a **flagless** re-verification command |

In every case the authoritative set was **already reachable** — from a catalog, a
struct definition, or the JSON being parsed. The list was a second copy, and nothing
failed when it fell behind.

### The measured instance this tree opens on

`src/bin/tool_matrix.rs` enumerates the **same growing set of gate flags in seven
production sites**. Measured by listing every non-test line naming one specific gate
(`casez_mux_if_gate`):

| site | line | omitting it… |
| --- | --- | --- |
| `Cli` field | `:201` | won't compile (clap derive + `test_cli()` literal) |
| `MatrixReport` field | `:1204` | **silent in `tool_matrix`** — the report just lacks the flag |
| report assignment | `:1418` | won't compile once the field exists (E0063) |
| `derive_run_plan` if-chain | `:1553` | **silent** — the gate runs 1 unit/scenario instead of 4 |
| `fail_on_coverage_gap` or-chain | `:1578` | **silent — the gate CANNOT FAIL** (see below) |
| mutual-exclusion sum | `:1598` | **silent** — two gates can be enabled at once |
| `select_scenario_set` if-chain | `:1629` | **silent** — the flag quietly selects `ScenarioSet::Default` |

plus the `bail!` prose that lists every flag by name (cosmetic, silent), and the
`ScenarioSet` variant + its `match` arms, which **are** compiler-enforced and are
therefore the safe ones.

**The severe one is `fail_on_coverage_gap`.** `src/bin/tool_matrix.rs:1490` reads

```rust
if plan.fail_on_coverage_gap && !report.coverage_gaps.is_empty() { bail!(…) }
```

so a gate whose flag is missing from that or-chain **runs, produces a report with
coverage gaps, and exits 0**. It would then be banked as clean closure evidence. A
gate that cannot fail is worse than no gate: it manufactures false confidence and
the digest mechanism would faithfully record it.

Today each gate does have a `assert!(plan.fail_on_coverage_gap)` unit test — but
writing that test is *itself* part of the same hand-maintained set. The guard is
"remember to add a test", not a derived property, so it decays at exactly the same
rate as the thing it guards.

**Nothing here is a live bug.** All seven sites are currently complete for all
fifteen gates (`EMIT-SURFACE-INTERACTION-GATE.3` added the fifteenth and touched
every one). The defect is that correctness rests on diligence, and the session that
opened this tree contains three proofs that diligence is not sufficient.

### The largest one found, currently correct and entirely unguarded

`merge_coverage` in `src/bin/tool_matrix.rs` hand-merges **every** field of
`CoverageSummary`. Measured `2026-07-30`:

```
struct fields: 149    fields merged: 149    never-merged: (none)
```

Complete today. Guarded by nothing. A forgotten 150th line means a coverage fact
never unions across scenarios — so a gate reports a gap that a sibling scenario
already closed, or (worse, if the fact is a *negative* check) misses one. This is
the single highest-value derived-check candidate in the repo, and it is not a bug
report — it is a 149-entry list that is one omission away from being one.

## Non-Goals

- **Not** a mechanical rewrite of every list in the repo. Some enumerations are the
  *authoritative* set (the `DOCTRINES` registry in `scripts/check_doctrines.sh`, the
  `presets()` table) and must stay hand-written — they are the source of truth, not a
  shadow of one.
- **Not** weakening any deliberate allow-list. `check_no_boot_volume_refs.sh`'s
  allow-list and `check_evidence_citations.sh`'s frozen §1 pin are **load-bearing by
  design** (`MEMORY.md` standing directives); this tree must not "improve" them into
  derivation.
- **Not** a refactor for elegance. A shadow enumeration that fails **loudly** (a
  non-exhaustive `match`) is already safe; only silent ones are in scope.
- **Not** changing generator behaviour. Every leaf must be DUT byte-identical.

## Acceptance Criteria

- A recorded audit classifying each candidate site as **authoritative** (leave),
  **compiler-enforced** (leave), or **silent shadow** (fix).
- Every confirmed silent shadow either derives its set from the authoritative source,
  or gains a test that fails when the two diverge — the
  `EMIT-SURFACE-INTERACTION-GATE.2` pattern (derive the expected set from
  `knob_catalog()`), not a second hardcoded list.
- Each fix negative-controlled in both directions: remove an entry ⇒ the guard fails;
  restore ⇒ it passes. A guard nobody has watched fail is a guard nobody knows works.
- A decision record if the class turns out to be mechanizable as a doctrine
  (`DOCTRINE_ENFORCEMENT.md` §4) — or a recorded finding that it is not, and why.
- Live docs updated; each leaf lands through `COMMIT.md`.

## Task Tree

- ID: `SHADOW-ENUMERATION-SWEEP`
  Status: `active`
  Goal: eliminate silent shadow enumerations; record the ones that must stay.
  Children: `.1` (register + audit), `.2` (design ADR — decision `0033`), `.3`–`.7`
        (one fix per site, in the severity order `.3` → `.5` → `.4` → `.6` → `.7`)

- ID: `SHADOW-ENUMERATION-SWEEP.1`
  Status: `done` (`2026-07-30`)
  Goal: record the observation and register the tree before any edit — the
        `EVIDENCE-BANK-DURABILITY.1` / `EMIT-SURFACE-INTERACTION-GATE.0` precedent
        (a lane is a tracked tree *before* implementation starts).
  Acceptance: the measured audit above is in this file, `CHANGES.md` and
        `MEMORY.md`; a `docs/TASK_TREE.md` row exists; docs-only ⇒ DUT
        byte-identical.
  Verification: `CoverageSummary` 149 fields / 149 merged (complete, unguarded);
        the seven per-gate `tool_matrix` sites enumerated with their failure modes;
        `fail_on_coverage_gap` confirmed load-bearing at `src/bin/tool_matrix.rs:1490`.

- ID: `SHADOW-ENUMERATION-SWEEP.2`
  Status: `done` (`2026-07-30`) — decision
        [`0033`](../decisions/0033-shadow-enumeration-classification.md)
  Goal: design ADR. Decide (a) the **classification rule** separating an
        authoritative enumeration from a shadow one — this is the load-bearing
        definition, because the non-goals above are all "authoritative lists that
        look like shadows"; (b) the standard repair per class (derive from the real
        set / add a divergence test / make the omission loud); (c) whether the class
        is mechanizable as a registered doctrine, or whether it is inherently a
        review property — `DOCTRINE_ENFORCEMENT.md` §3 archetypes, and the honest
        answer may be *no*, since "this list mirrors that set" is not syntactically
        detectable in general; (d) the ordering of `.3`+ by **severity of silent
        failure**, not by size.
  Acceptance: a `docs/decisions/00NN-*.md` with Context / Decision / Consequences,
        naming the rule, the repairs, the mechanizability verdict, and the leaf order.
  Delivered: decision `0033`.
        **(a)** A list `L` is a shadow **iff all three** hold — (1) *derivable*: a set
        `S` in the repo already enumerates the same membership and is reachable by
        ordinary program/script means; (2) *growth-coupled*: `S` grows and every growth
        requires a matching entry in `L`; (3) *silent*: the omission produces no compile
        error, no failing test, no runtime error. Test (2) is load-bearing — it is what
        rejects `check_no_boot_volume_refs.sh`'s allow-list and the `EVIDENCE-CITATIONS`
        §1 pin, both of which satisfy (1) and are authoritative anyway *because the gap
        between `L` and `S` is the content of the rule*. Test (1) rejects `presets()` and
        `DOCTRINES` (no `S` exists).
        **(b)** A four-rung repair ladder in strict preference order: **R1** derive (the
        list stops existing) → **R2** make it loud (a compile error) → **R3** guard with a
        test that *derives* the expectation (models: `knob_catalog_classifies_every_field`,
        `config.rs:2664`, and the decision-`0032` preset drift test, `config.rs:2569`) →
        **R4** a registered doctrine check, for sites with no compiler and no `cargo test`.
        Two binding constraints: a repair may not introduce a new hand-maintained list, and
        every guard is negative-controlled in both directions.
        **(c)** Mechanizable as *discovery*: **no**, and stated as an honest limit
        (`DOCTRINE_ENFORCEMENT.md` §9). Test (1) is a semantic relation between two sets;
        nothing syntactic separates `presets()` from a shadow, so a detector would have to
        already know the pairing — the exact human judgement rule (a) encodes. Its only two
        failure modes are *miss* (false confidence — this tree's own defect, at the meta
        level) and *cry wolf* (and a gate that cries wolf gets deleted). Mechanizable as
        *holding classified pairs*: **yes**, split by language — Rust sites get an in-crate
        `#[test]` (already gated by `cargo test` per `COMMIT.md` + CI; a shell doctrine
        would be a second mechanism for one job, `feedback_full_factorization`), docs/script
        sites get **one** `ENUMERATION-PARITY` check over a declared table of
        `(shadow, source, extractor)` triples — a table that is itself authoritative under
        rule (a), so the mechanism does not recurse.
        **(d)** Severity tiers **S3** (a gate reports success it did not earn — a *false
        green*, banked as `0030` evidence and cited) / **S2** (a user-requested behaviour
        silently does not happen) / **S1** (a contract or report understates reality —
        fail-safe in direction). Execution order `.3` → `.5` → `.4` → `.6` → `.7`; leaf
        numbers stay in creation order because a leaf id is the durable commit↔tree join
        key (`COMMIT.md` task-tree rule #1).
  Also delivered — the full audit (20 sites) and **three corrections to `.1`**:
        - **`merge_coverage` is S1, not the top candidate.** Measured: all 149 merges are
          monotone (`135 × |=` + `13 × .extend()` + `1 × .max()`) ⇒ an omission can only
          **under**-report, never over-report; and **134 of the 149** fields are referenced
          by `compute_coverage_gaps`, every gap having the positive-polarity shape
          `if !coverage.saw_x { … }` ⇒ a forgotten merge yields a *spurious gap* ⇒ under any
          gate the run **bails loudly**. The genuinely silent surface is **15 fields**, not
          149. `.1` ranked it #2 by list length; severity is what an omission corrupts.
        - **`Config::apply_cli_overrides` (87 fields, unguarded) takes #2.** The tree's own
          open question, now measured: `Overrides` 87 fields / 87 read / 87 written, complete
          and guarded by nothing. Its omission makes a CLI flag **and every preset that sets
          that knob** silently inert — the exact shape of the bug decision `0032` found in
          `structured-emission-max`, since presets apply through this same applier
          (`config.rs:2465`).
        - **Three further sites `.1` never reached**, all silent shadows: the four hardcoded
          adapter-id JSON-schema `enum`s in `src/mcp/mod.rs` (`:278`, `:303`, `:327`, `:458`)
          shadowing `ADAPTER_REGISTRY`; `DOCTRINE_ENFORCEMENT.md` §10's table shadowing the
          `DOCTRINES` array; and `book/src/SUMMARY.md` shadowing `book/src/*.md`.
  Verification: docs-only ⇒ **DUT byte-identical**, `tests/snapshots.rs` untouched.

- ID: `SHADOW-ENUMERATION-SWEEP.5`
  Status: `done` (`2026-07-30`)
  Goal: **execution order 2** (S2). Guard `Config::apply_cli_overrides`
        (`src/config.rs:1734`) — 87 `if let Some(v) = o.x { self.x = v }` lines shadowing
        `Overrides`'s 87 fields, with no guard of any kind. A forgotten line makes the
        knob's CLI flag *and* any `--profile` preset that sets it silently inert
        (presets apply through the same applier, `config.rs:2465`). Preferred repair
        **R2** if an exhaustive form exists; otherwise **R3**: a round-trip test that sets
        every `Overrides` field to a non-default `Some(_)`, applies it to
        `Config::default()`, and asserts every corresponding `Config` field moved —
        deriving the field set from the serde projection, never from a hand-written list.
  Acceptance: the test fails when one applier line is deleted and passes when restored;
        no new hand-maintained list introduced; `tests/snapshots.rs` untouched.
  Delivered: **R3 for the applier, with an R2-protected fixture** — the combination is
        the point, because the naive R3 needs a fully-populated `Overrides`, and writing
        that by hand would be a *fresh* shadow of the very struct under test.
        - `every_override_set()` is an **exhaustive** `Overrides { … }` literal with no
          `..Default::default()`, so adding a field to `Overrides` without adding it here
          is an `E0063` compile error. The compiler maintains the fixture; nobody has to.
          Values are uniform by type (`0.937` / `4243` / `true` / a non-default enum
          variant), chosen only to differ from every current default.
        - `apply_cli_overrides_moves_every_overridable_config_field` derives the
          expectation by intersecting the **serde projections** of `Overrides` and
          `Config`, applies the fixture to `Config::default()`, and asserts every knob in
          that intersection moved. A new knob joins the expectation *by existing*.
        - It also pins the **complement** as a decision rather than an accident: exactly
          four `Config` knobs are not CLI-overridable — `library_prob`,
          `max_nodes_per_module`, `use_async_reset` (the documented config-file-only three,
          decision `0021`) and `seed` (stamped last by `resolve_config`). Moving that
          boundary now fails loudly and points at `knob_catalog()`'s `cli_flag` /
          `config_only` partition.
        - `apply_cli_overrides_moves_only_the_knob_it_was_given` is the second, independent
          leg: a *leaky* applier line (one that also writes a neighbouring field) leaves the
          whole-struct test green — every knob still moves — and only this one catches it.
          Demonstrated, not assumed (NC-D below).
        - Measured while writing it: `Overrides` has **88** fields, not 87. The 88th is
          `steer`, the one override that is **not** a `Config` field — it is a repeatable
          list folded into `Config.steering` by `resolve_config`, not a single-value knob.
          Pinned by name so the "no orphan override" assertion stays exact.
  Verification: five negative controls, all confirmed — see the Verification Log.
        `src/config.rs` test-module + no production change ⇒ **DUT byte-identical**;
        `tests/snapshots.rs` untouched (6/6).

- ID: `SHADOW-ENUMERATION-SWEEP.6`
  Status: `done` (`2026-07-30`)
  Goal: **execution order 4** (S1). The four hardcoded adapter-id JSON-schema `enum`
        literals in `src/mcp/mod.rs` (`:278`, `:303`, `:327`, `:458`) shadow the
        closed `ADAPTER_REGISTRY` (`src/downstream/mod.rs:1080`, 5 entries). A sixth
        adapter is selectable by `validate`/`hunt`/`divergence` yet unadvertised to
        agents — an API-contract divergence against decision `0017`'s API-first mandate,
        and invisible to the compiler (they are string literals inside a JSON blob).
        Preferred repair **R1**: build the `enum` array from `adapters()` so the literals
        disappear; fallback **R3**: a test that parses the emitted tool schemas and
        asserts each `enum` equals the registry's ids.
  Acceptance: adding a registry entry cannot leave a schema behind; negative-controlled;
        `tests/snapshots.rs` untouched.
  Delivered: **R1 at every site, and the audit grew from four copies to seven** —
        `registered_adapter_ids()` (`src/mcp/mod.rs`) is the one derivation from
        `downstream::adapters()`, read by:
        - the `tools` JSON-schema `enum` in all four controlled tools
          (`validate` / `divergence` / `minimize` / `hunt`) — computed once per
          `tools_list()` call;
        - the unknown-tool error in `parse_validate_tools` (was a retyped
          `"allowed = verilator, yosys, iverilog, sv2v, slang"` literal);
        - `validate`'s **tool description**, and the server **`instructions`** —
          two prose sites `.2`'s audit did not count.
        - **The shadow had already failed — this is the tree's first LIVE defect,
          not a latent one.** Measured before the fix: the `instructions` named
          **three** adapters (`verilator / yosys / iverilog`) and `validate`'s
          description **four** (missing `slang`), against a registry of five. So an
          agent reading ANVIL's own API description was told it accepts less than it
          accepts, for two adapter-landing slices running. Decision `0033` ranked this
          site S1 *"one omission away"*; the honest reading is that two omissions had
          already happened and nothing anywhere noticed. **Prose is a contract too**
          — it is what an agent reads before it ever inspects a schema — so it now
          derives like the `enum`s do, and the new prose guard is the one that caught
          a defect rather than a hypothetical (NC-A replays it).
        - **`AcceptanceTool::from_name`'s `_ => None` catch-all was audited and
          deliberately left alone.** It looks like the same shadow, but
          `adapter_registry_lists_the_originals_then_new_adapters`
          (`src/downstream/mod.rs:2360`) already loops **every registry id** through
          `from_name` and asserts it parses — derived from `adapters()`, not
          hand-listed. Adding a second guard would be a second mechanism for one job
          (`feedback_full_factorization`). Recorded because the site reads as unguarded
          until the existing test's loop is read.
        - Three derived guards, none introducing a new hand-maintained list (each
          builds its expectation from `adapters()`):
          `every_controlled_tool_schema_advertises_the_whole_adapter_registry` (walks
          the real `tools/list` output and checks **every** tool that takes a `tools`
          argument, so a *future* tool that retypes the array is caught too, plus a
          `>= 4` anti-decay floor so a walk that silently matches nothing cannot pass
          vacuously), `agent_facing_prose_names_every_registered_adapter`, and
          `the_unknown_tool_error_names_every_registered_adapter` (which generalizes
          the two per-adapter `sv2v`/`slang` pins, kept as landing-slice regressions).
        - No new hand-written list anywhere: the schema `enum`s, the error, and both
          prose strings are all rendered from the same `Vec<&'static str>`.
  Verification: five controls, all confirmed — see the Verification Log. The decisive
        one is **NC-E**: a sixth adapter added to `ADAPTER_REGISTRY` **and nothing
        else** propagated to all four schemas, both prose sites, and the error message
        with zero edits under `src/mcp/` — the leaf's acceptance criterion demonstrated
        rather than argued. `src/mcp/mod.rs` only, and MCP is beside the generator ⇒
        **DUT byte-identical**; `tests/snapshots.rs` untouched.

- ID: `SHADOW-ENUMERATION-SWEEP.7`
  Status: `pending`
  Goal: **execution order 5** (S1). The two docs/script-side pairs, which have neither a
        compiler nor `cargo test` and so can only take **R4**: `DOCTRINE_ENFORCEMENT.md`
        §10's table (6 rows) shadowing the `DOCTRINES` array (6 entries) — a documented
        but unregistered doctrine is a "trust me" rule, the §11 anti-pattern — and
        `book/src/SUMMARY.md` (29 links) shadowing `book/src/*.md` (29 chapters) — an
        unlinked chapter is written and **never rendered**, and the book is the owner's
        only window into the project. One new registered doctrine `ENUMERATION-PARITY`
        over a declared table of `(shadow site, authoritative source, extractor)` triples,
        not one doctrine per pair; the pairs table is authoritative under decision `0033`
        rule (a), so the mechanism does not recurse.
  Acceptance: `scripts/check_enumeration_parity.sh` obeys the `DOCTRINE_ENFORCEMENT.md`
        §4 contract, is registered in `DOCTRINES`, and is negative-controlled in both
        directions on both pairs; the §10 table gains a row for it; docs/script only ⇒
        DUT byte-identical.

- ID: `SHADOW-ENUMERATION-SWEEP.3`
  Status: `done` (`2026-07-30`)
  Goal: the highest-severity fix — the `tool_matrix` gate-flag sites, above all the
        `fail_on_coverage_gap` or-chain whose omission yields a gate that cannot
        fail. Candidate shape: one `const GATES: &[GateSpec]` table (flag name,
        `ScenarioSet`, min units) that every site reads, so a gate is declared once;
        or, if a table is too invasive, a test that derives the expected flag set
        from the `Cli` struct and asserts every site covers it.
  Acceptance: adding a gate cannot silently miss a site; negative-controlled;
        `tests/snapshots.rs` untouched.
  Delivered: **R1 (derive) for four of the five silent sites, plus R3 for the table
        itself** — the shape the `.2` open question left to be decided against real code.
        - `static GATES: &[GateSpec]` (`src/bin/tool_matrix.rs`) declares each gate
          **once**: `flag`, `enabled: fn(&Cli) -> bool`, `scenario_set`, `unit_floor`.
          `enabled_gates(cli)` is the one iterator both consumers read.
        - `derive_run_plan`'s 15-arm `if`-chain → a `map_or` over that iterator, and the
          **S3 line** becomes `cli.fail_on_coverage_gap || gate.is_some()` — *derived,
          not enumerated*. Any registered gate now arms the `bail!` with nothing left
          to forget.
        - `select_scenario_set`'s 15-arm `if`-chain, its 15-term exclusivity sum, and
          its retyped 15-flag `bail!` prose all collapse into the same iterator; the
          message is joined from the table.
        - **R2 via `ScenarioSet` was evaluated and rejected against the code** (`.2`
          open question 2). `--phase1-gate` maps to `ScenarioSet::Default` — it raises
          the corpus size over the *built-in* set rather than selecting a dedicated one
          — so `fail_on_coverage_gap = scenario_set != Default` would have silently
          **disarmed the Phase 1 gate**. Deriving from *"some registered gate is
          enabled"* keeps every gate's behaviour identical. Recorded because the
          rejected form looks obviously right until `phase1` is read.
        - The one remaining silent site, `MatrixReport`'s 15 `*_gate` field
          declarations, is guarded by a serde round-trip test keyed off a
          flag→field-name derivation (`--phase1-gate` ⇒ `phase1_gate`); `MatrixReport`
          gained `Default` **solely** so that test needs no hand-written literal (a
          literal would be a fresh shadow of the struct). The report *assignment* site
          was already E0063-enforced and is unchanged.
        - Four derived guards, none introducing a new hand-maintained list:
          `every_cli_gate_flag_is_registered` (expected set derived from **clap's own
          `Cli` metadata** — the `knob_catalog_classifies_every_field` pattern),
          `every_registered_gate_selects_a_distinct_scenario_set`,
          `every_registered_gate_arms_the_coverage_gap_check_and_raises_units` (which
          turns the flag on **through `Cli::try_parse_from`** rather than a
          flag→field switch — that switch was drafted and deleted on realising it
          would be a *seventh* copy of the same set, and parsing additionally proves
          each `flag` string is a real CLI flag), and
          `matrix_report_records_every_registered_gate_flag`.
  Verification: see the Verification Log. Negative-controlled in **both** directions
        and, separately, the S3 defect was **reproduced on purpose**: with the
        fifteenth `GATES` row deleted, a temporary probe measured
        `fail_on_coverage_gap = false, modules_per_scenario = 1` for
        `--emit-surface-interaction-gate` — a gate that runs 1 unit per scenario and
        **cannot fail** — while the new derived guard named the exact missing flag.
        Harness binary only (no `src/gen`/`src/emit`/`src/ir`/`src/config` change) ⇒
        **DUT byte-identical**; `tests/snapshots.rs` untouched.

- ID: `SHADOW-ENUMERATION-SWEEP.4`
  Status: `done` (`2026-07-30`)
  Goal: **execution order 3** (S1 — re-scoped by decision `0033` §3). Guard
        `merge_coverage` — 149 hand-merged `CoverageSummary` fields with no divergence
        check. Likely repair: a test that serialises a `CoverageSummary`
        with every `bool` set `true` / every set non-empty, merges it into
        `default()`, and asserts the result equals the source — which fails for any
        field the merge forgot, without enumerating the fields.
  Acceptance: the test fails when a merge line is deleted; passes when restored;
        no new hand-maintained list introduced by the fix itself.
  Re-scoped `2026-07-30` (`.2`, decision `0033`): `.1` ranked this second by list
        length; measurement says third by severity. All 149 merges are **monotone**
        (`135 × |=` + `13 × .extend()` + `1 × .max()`) ⇒ an omission can only
        **under**-report, never over-report; and **134 of 149** fields are read by
        `compute_coverage_gaps`, whose every gap has the positive-polarity shape
        `if !coverage.saw_x { … }` ⇒ a forgotten merge produces a *spurious gap* ⇒ under
        any gate the run **bails loudly**. The genuinely silent surface is the **15**
        ungated fields, several of which `README.md` cites as Phase-4 hierarchy evidence.
        Still in scope — the round-trip guard costs one test and needs no per-field
        list — but it is not a false-green site.
  Delivered: the `.5` pattern reused verbatim — **R3 on an R2-protected fixture, two legs**.
        - `every_coverage_fact_set()` is an **exhaustive** `CoverageSummary { … }` literal
          (149 fields, no `..Default::default()`), so adding a fact to the struct without
          adding it here is an `E0063` compile error. The compiler maintains it.
        - **Leg 1** `merge_coverage_unions_every_coverage_fact` — merge the full fixture
          into `default()` and compare the two **serde projections**; any forgotten merge
          line names itself. Plus the anti-decay assert (`> 100` facts projected) so a
          derivation that silently collapses fails instead of passing vacuously.
        - **Leg 2** `merge_coverage_unions_each_fact_into_its_own_field` — feed **one**
          fact at a time and assert exactly that field moved. This is the leg that matters
          here: 149 near-identical merge lines with long shared name prefixes are exactly
          where a cross-wire (`dst.a |= src.b`) is likely, and leg 1 **cannot see one** —
          with every source fact set, both fields end `true` either way. The per-field
          sources are built by serde from the same fixture, so leg 2 adds no second list.
        - `CoverageSummary` gained `Deserialize` + `#[serde(default)]` **solely** so leg 2
          can build a single-fact source without a 149-entry per-field literal. Both are
          inert for the run path — nothing deserializes a `CoverageSummary` — and neither
          changes a byte of the emitted `tool_matrix_report.json`. Same justification, and
          same doc-comment discipline, as `.3`'s `MatrixReport: Default`.
  Verification: four negative controls (below). The decisive one is **NC-C**: a cross-wired
        merge line left **leg 1 green** and was caught only by leg 2 — the predicted blind
        spot, measured rather than assumed. Harness binary only ⇒ **DUT byte-identical**;
        `tests/snapshots.rs` untouched.

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| — | `.6` | `done` | **S1**, closed by **R1** at seven sites, not the four `.2` counted: the schema `enum`s plus the unknown-tool error and two prose strings, all rendered from one `registered_adapter_ids()`. The find: **two of them had already fallen behind** — the tree's first *live* defect. NC-E is the payoff — a sixth registry entry propagated everywhere with zero edits under `src/mcp/`. |
| — | `.4` | `done` | **S1**, closed with the `.5` pattern reused verbatim: an exhaustive 149-field `CoverageSummary` fixture the compiler maintains, a serde-projection leg-1 equality, and a per-fact leg 2. NC-C is the payoff — a cross-wired merge line left leg 1 **green**. |
| — | `.3` | `done` | The only **S3** site in the audit, closed by **R1**: one `static GATES` table, four sites derived from it, and `fail_on_coverage_gap` reduced to *"some registered gate is enabled"*. Negative-controlled both ways, and the S3 defect reproduced on purpose before the guard was proven to catch it. |
| — | `.5` | `done` | **S2**, closed by **R3 with an R2-protected fixture**: an exhaustive `Overrides` literal the compiler maintains, a serde-derived expectation, a pinned not-overridable complement, and a second leg catching leaky applier lines the first cannot see. Five negative controls. |
| 1 | `.7` | `pending` | **Current frontier. S1**, and the last leaf. The two docs/script pairs — the only sites with no compiler and no `cargo test`, so the only ones that need a registered doctrine (`ENUMERATION-PARITY`). `.6` adds a third candidate pair for its table: `book/src/api-tools.md`'s hand-written `tools` `enum` (now the only remaining copy of the adapter-id list outside the registry). |
| — | `.2` | `done` | The classification rule, the repair ladder, the mechanizability verdict, the severity tiers, and the 20-site audit — decision `0033`. |
| — | `.1` | `done` | Registered + audited. |

## Decisions

- `2026-07-30`: Opened on **owner directive** after being raised as a suggestion and
  parked in `MEMORY.md`. The owner's point is recorded as the tree's own rationale
  (see "Why this is a tree"): layer A is overwrite-only, so a finding stored only
  there is scheduled for deletion, not retention.
- `2026-07-30`: Scoped as **defect-class elimination**, not a refactor. Only
  enumerations whose omission is *silent* are in scope; a non-exhaustive `match` is
  already a working guard and stays as it is.
- `2026-07-30`: The three originating bugs are **not** re-opened here. Each was fixed
  in its own leaf (`EMIT-SURFACE-INTERACTION-GATE.2` / `.4`,
  `EVIDENCE-BANK-DURABILITY.6`). This tree owns the *class*, not the instances.
- `2026-07-30` (`.2`, decision
  [`0033`](../decisions/0033-shadow-enumeration-classification.md)): the classification
  rule is a **conjunction of three tests** (derivable ∧ growth-coupled ∧ silent), not
  "L duplicates S". The middle test is the load-bearing one: dropping it would classify
  `check_no_boot_volume_refs.sh`'s allow-list and the `EVIDENCE-CITATIONS` §1 pin as
  shadows, and "fixing" them would delete decision `0031`'s history-stays-raw guarantee
  and decision `0030`'s un-growable grandfather list.
- `2026-07-30` (`.2`): the class is **not mechanized as a discovery doctrine** — stated
  as an honest limit rather than shipped as a guessing gate, because a gate that cries
  wolf gets deleted and takes its real coverage with it. Only classified *pairs* are
  held mechanically, and only docs/script pairs need a new doctrine (`.7`); Rust pairs
  are held by `cargo test`, which is already `COMMIT.md`'s mandatory gate.
- `2026-07-30` (`.2`): `.1`'s severity ranking is **corrected from measurement**.
  `merge_coverage` drops from #2 to #3 (monotone merges + 134/149 gated ⇒ loud), and
  the previously-unaudited `apply_cli_overrides` takes #2. Recorded rather than quietly
  reordered — the reusable lesson is that `.1` reasoned from a list's *length* and `.2`
  measured what an *omission does*.
- `2026-07-30` (`.2`): leaf **numbers** are creation order and are never renumbered (a
  leaf id is the durable commit↔tree join key, `COMMIT.md` task-tree rule #1). The
  corrected **execution** order lives in the `Current Frontier` table, which exists for
  exactly this.

## Open Questions

**Answered by `.2` (decision `0033`):**

- ~~Is the class mechanizable at all?~~ **Answered: not as *discovery*; yes as
  *holding classified pairs*.** Rule (a)'s test (1) is a semantic relation between two
  sets, so a syntactic detector would have to already know the pairing — the exact
  judgement the rule encodes — and its only failure modes are *miss* (false confidence)
  and *cry wolf* (and a gate that cries wolf gets deleted). Recorded as an honest limit
  per `DOCTRINE_ENFORCEMENT.md` §9. Holding the pairs splits by language: Rust sites →
  an in-crate `#[test]` (already gated by `cargo test`; a shell doctrine would be a
  second mechanism for one job), docs/script sites → one `ENUMERATION-PARITY` check
  (leaf `.7`).
- ~~Where else does the repo shadow a growing set — is a systematic search worth it?~~
  **Answered: a directed sweep, not an exhaustive one.** `.2` swept the repo's
  registries (config, `tool_matrix`, MCP, downstream adapters, doctrines, book) and
  recorded a 20-site audit; exhaustiveness is unachievable for a semantic property, so
  the rule is written down instead, and the next site is classified in one reading.
  Three new silent shadows found: `apply_cli_overrides` (`.5`), the MCP adapter-id
  schema enums (`.6`), and the two docs-side pairs (`.7`).
- ~~Does `Overrides` → `apply_cli_overrides` have the same shape?~~ **Answered: yes,
  and it is now execution-order #2.** Measured `87` `Overrides` fields / `87` read /
  `87` written — complete today, guarded by nothing. `main.rs:1066`'s `cli_overrides`
  is the safe half (an exhaustive struct literal ⇒ E0063). Owned by `.5`.

**Answered by `.3`:**

- ~~`.3`'s shape: R1 table versus R3 derived test?~~ **Answered: both, and they are not
  alternatives.** R1 (`static GATES`) removes four of the five silent sites outright;
  R3 (`every_cli_gate_flag_is_registered`, expectation derived from clap's `Cli`
  metadata) is what stops the *table itself* becoming the next shadow. The differing
  payloads the question worried about resolved into four fields — `flag`, `enabled`,
  `scenario_set`, `unit_floor` — with the two incompatible unit shapes modelled
  explicitly as `UnitFloor::{TotalAtLeast, PerScenario}` rather than flattened.
- ~~Can `.3` reach **R2** by deriving `fail_on_coverage_gap` from the `ScenarioSet`?~~
  **Answered: no — and the reason is a trap.** `--phase1-gate` maps to
  `ScenarioSet::Default` (it raises the corpus size over the *built-in* scenario set
  rather than selecting a dedicated one), so `fail_on_coverage_gap = set != Default`
  would have silently **disarmed the Phase 1 gate** — introducing the exact S3 defect
  the leaf exists to remove, inside the fix. Deriving from *"some registered gate is
  enabled"* is behaviour-preserving for all fifteen. Recorded because the rejected form
  reads as obviously correct until `phase1`'s arm is looked for and found missing.

**Answered by `.5`:**

- ~~Can `.5`'s guard reach R2, or must it be R3?~~ **Answered: R3 for the applier, but
  with an R2-protected *fixture* — and that pairing is the reusable pattern.** A pure R3
  round-trip needs a fully-populated `Overrides`, and hand-writing that would be a fresh
  shadow of the struct under test. Writing it as an **exhaustive literal** (no
  `..Default::default()`) makes the compiler maintain it (`E0063`, demonstrated by NC-E),
  so the guard introduces no hand-maintained list. Reaching R2 for the *applier itself*
  would take a macro generating both the struct and the applier from one field list —
  rejected: it would obscure an 88-field, per-field-documented public struct for a
  guarantee the fixture already provides.
- Also learned: a whole-struct round trip is **not sufficient on its own**. A *leaky*
  applier line — one that writes its own field and a neighbour's — leaves it green
  (every knob still moves). The second, single-knob leg catches it, and NC-D demonstrates
  that rather than asserting it. `.4` should carry the same two legs.

**Answered by `.6`:**

- ~~Whether `.6` should derive the four JSON-schema enums from `adapters()` (eliminating
  the literals) or guard them with a test that parses the emitted schema.~~ **Answered:
  derive (R1), and the count was wrong — there were seven copies, not four.** `.2`'s
  audit swept for the *schema* shape and found the four `enum` literals; sweeping instead
  for *every line naming ≥ 2 adapter ids* found three more — the unknown-tool error and
  two prose strings. And the prose is where the shadow had **already failed**: the server
  `instructions` named three adapters and `validate`'s description four. The reusable
  lesson is that an audit keyed to the *shape* of the known instance under-counts; keying
  it to the *content* (the ids themselves) is what found the live one.
- Also learned: **a derived guard should walk the real output, not the known sites.**
  `every_controlled_tool_schema_advertises_the_whole_adapter_registry` enumerates every
  tool in the emitted `tools/list` that takes a `tools` argument rather than checking the
  four by name, so a *future* tool that retypes the array is caught by a test written
  before it existed. Checking the four known sites would have been a fifth copy of the
  same set.

**Open for `.7`:**

- Should `ENUMERATION-PARITY`'s declared-pairs table live inside the check script, or in
  a small tracked data file the check reads?
- `.6` leaves one adapter-id copy outside the registry: `book/src/api-tools.md`'s
  hand-written `tools` `enum` (`"verilator" | "yosys" | …`). It is a docs pair with no
  compiler and no `cargo test`, so it is a candidate row for `.7`'s pairs table alongside
  the `DOCTRINE_ENFORCEMENT.md` §10 and `SUMMARY.md` pairs.

## Blockers

- None. `.2` was a docs/decision leaf and needed no tool run. `.3` is the first leaf
  that touches `src/` and takes the full `COMMIT.md` §1 code-hygiene gate.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-07-30` | `.1` | `CoverageSummary` struct fields vs `merge_coverage` `dst.` assignments | **149 / 149** — complete today, guarded by nothing |
| `2026-07-30` | `.1` | Every non-test line in `src/bin/tool_matrix.rs` naming one specific gate | **7 production sites**; 5 silent on omission, 2 compiler-enforced |
| `2026-07-30` | `.1` | `src/bin/tool_matrix.rs:1490` read directly | `fail_on_coverage_gap` gates the `bail!`, so omitting a gate from its or-chain yields a gate that exits 0 with non-empty `coverage_gaps` |
| `2026-07-30` | `.1` | `MEMORY_ARCHITECTURE.md` §71 | layer A is "Overwritten each update; hard size cap" — confirms a `MEMORY.md`-only finding is not retained |
| `2026-07-30` | `.2` | `Cli` gate-bool fields vs `ScenarioSet` variants vs `MatrixReport` gate fields vs the `fail_on_coverage_gap` or-chain | **15 / 15 / 15 / 15** — the growing set is 15 gates, and all four sites are complete today |
| `2026-07-30` | `.2` | `merge_coverage` merge operators | **`135 × \|=` + `13 × .extend()` + `1 × .max()` = 149**, zero assignments ⇒ **monotone**; an omission can only under-report |
| `2026-07-30` | `.2` | `CoverageSummary` fields vs fields read by `compute_coverage_gaps` | **149 vs 134** ⇒ only **15** are ungated; and every gap is `if !coverage.saw_x {…}` (positive polarity) ⇒ a forgotten merge on a gated fact yields a *spurious gap* ⇒ the gate **bails loudly**. `.1`'s "highest-value candidate" ranking corrected to S1 |
| `2026-07-30` | `.2` | `Overrides` fields vs `apply_cli_overrides` reads vs writes | **87 / 87 / 87** — complete, **unguarded**; the tree's open question answered. `main.rs:1066` `cli_overrides` is an exhaustive struct literal ⇒ compiler-enforced |
| `2026-07-30` | `.2` | `ADAPTER_REGISTRY` entries vs hardcoded adapter-id JSON `enum`s in `src/mcp/mod.rs` | **5 vs 5, copied 4×** (`:278`, `:303`, `:327`, `:458`) — complete today, unguarded, invisible to the compiler (string literals in a JSON blob) |
| `2026-07-30` | `.2` | `DOCTRINES` entries vs `DOCTRINE_ENFORCEMENT.md` §10 rows | **6 vs 6** — complete, unguarded (no compiler, no `cargo test`) |
| `2026-07-30` | `.2` | `book/src/SUMMARY.md` links vs `book/src/*.md` | **29 vs 29** (30 files − `SUMMARY.md` itself) — complete, unguarded; an unlinked chapter is never rendered |
| `2026-07-30` | `.2` | Rule (a) applied to the four must-stay-hand-written cases | `presets()` + `DOCTRINES` rejected on test (1) (no `S`); `check_no_boot_volume_refs.sh`'s allow-list + `EVIDENCE-CITATIONS` §1 pin rejected on test (2) (must differ / must not grow) — the rule's calibration holds |
| `2026-07-30` | `.2` | Docs-only leaf | no `src/` change ⇒ **DUT byte-identical**, `tests/snapshots.rs` untouched |
| `2026-07-30` | `.3` | `cargo check --all-targets` · `cargo clippy --all-targets -- -D warnings` · `cargo fmt --all --check` | exit `0` / `0` / `0` |
| `2026-07-30` | `.3` | `cargo test --bin tool_matrix` (re-run after `cargo fmt`) | **111 passed, 0 failed** (4 new derived guards) |
| `2026-07-30` | `.3` | `cargo test` (full suite, under `ram_guard.sh --threshold 90`) | exit `0` — **17 test binaries, 1038 passed, 0 failed, 18 ignored** |
| `2026-07-30` | `.3` | `cargo test --test snapshots` | **6 passed, 0 failed** — byte-identical |
| `2026-07-30` | `.3` | **NC-A** — delete the fifteenth `GATES` row, rerun | **FAILED 3** — `every_cli_gate_flag_is_registered` names the exact gap: *"missing here: [\"--emit-surface-interaction-gate\"]; stale here: []"* |
| `2026-07-30` | `.3` | **NC-A′ (the S3 defect, reproduced on purpose)** — with that row deleted, a temporary probe read `derive_run_plan` for `--emit-surface-interaction-gate` | `fail_on_coverage_gap = false`, `modules_per_scenario = 1` — **a gate that runs 1 unit/scenario and cannot fail**, exactly the failure mode `.1` predicted. Probe removed; source restored byte-identical |
| `2026-07-30` | `.3` | **NC-B** — restore the row, rerun | **111 passed, 0 failed**; `diff` vs the pre-NC backup byte-identical |
| `2026-07-30` | `.3` | **NC-C** — point the `--casez-mux-if-gate` row at the `case` scenario set (the realistic copy-paste error) | **FAILED** — *"--casez-mux-if-gate reuses scenario set case-mux-if-sweep"*; restored byte-identical |
| `2026-07-30` | `.3` | R2-via-`ScenarioSet` evaluated against the code (`.2` open question 2) | **rejected**: `--phase1-gate` maps to `ScenarioSet::Default`, so `fail_on_coverage_gap = set != Default` would have silently **disarmed the Phase 1 gate** |
| `2026-07-30` | `.3` | Diff scope | `src/bin/tool_matrix.rs` only — no `src/gen`, `src/emit`, `src/ir`, `src/config` ⇒ **DUT byte-identical** |
| `2026-07-30` | `.5` | `Overrides` fields vs `apply_cli_overrides` vs `Config` keys | **88** override fields = **87** applied + `steer`; `Config` has **91** keys; `Overrides − Config = {steer}` (a repeatable list folded into `Config.steering` by `resolve_config`, not a knob); `Config − Overrides = {library_prob, max_nodes_per_module, seed, use_async_reset}` — now pinned by the test |
| `2026-07-30` | `.5` | `cargo check --all-targets` · `clippy --all-targets -- -D warnings` · `fmt --all --check` | exit `0` / `0` / `0` |
| `2026-07-30` | `.5` | `cargo test --lib` | **742 passed, 0 failed** (was 740 — the two new guards) |
| `2026-07-30` | `.5` | `cargo test` (full suite, under `ram_guard.sh --threshold 90`) | exit `0` — **17 test binaries, 1040 passed, 0 failed, 18 ignored** |
| `2026-07-30` | `.5` | `cargo test --test snapshots` | **6 passed, 0 failed** — byte-identical |
| `2026-07-30` | `.5` | **NC-A** — delete the `max_depth` applier line | **FAILED 2** — *"apply_cli_overrides ignored these knobs … : [\"max_depth\"]"* |
| `2026-07-30` | `.5` | **NC-B** — restore | **38 passed, 0 failed**; `diff` vs backup byte-identical |
| `2026-07-30` | `.5` | **NC-C** — cross-wire `o.min_width → self.max_width` | **FAILED** — *"… : [\"min_width\"]"*; restored byte-identical |
| `2026-07-30` | `.5` | **NC-D** — a *leaky* applier line (`o.max_depth` also writes `self.max_width`), the slip the whole-struct test **cannot** see | whole-struct test stayed **green**; the second leg caught it: *"a single override must move exactly its own knob — left: [\"max_depth\", \"max_width\"], right: [\"max_depth\"]"*. Proves the second test's independent value rather than assuming it |
| `2026-07-30` | `.5` | **NC-E** (the R2 leg) — add a field to `Overrides`, touch nothing else | **`E0063`** at the fixture *and* at `main.rs:1066` — the fixture is compiler-maintained, not hand-maintained; restored byte-identical |
| `2026-07-30` | `.5` | Diff scope | `src/config.rs` **test module only**, no production line changed ⇒ **DUT byte-identical** |
| `2026-07-30` | `.4` | `CoverageSummary` field census re-derived | **149** = `135` `bool` + `13` `BTreeSet<String>` + `1` `usize` (one `bool` is line-wrapped and was missed by a naive per-line type regex — the fixture generator was fixed rather than the count guessed) |
| `2026-07-30` | `.4` | `cargo check --all-targets` · `clippy --all-targets -- -D warnings` · `fmt --all --check` | exit `0` / `0` / `0` |
| `2026-07-30` | `.4` | `cargo test --bin tool_matrix` | **113 passed, 0 failed** (was 111 — the two new legs) |
| `2026-07-30` | `.4` | `cargo test --test snapshots` | exit `0` — byte-identical |
| `2026-07-30` | `.4` | `cargo test` (full suite, under `ram_guard.sh --threshold 90`) | exit `0` — **17 test binaries, 1042 passed, 0 failed, 18 ignored** |
| `2026-07-30` | `.4` | **NC-A** — delete the `saw_flop_merge` merge line | **FAILED 2** (both legs) — *"merge_coverage never unions these facts across scenarios: [\"saw_flop_merge\"]"* |
| `2026-07-30` | `.4` | **NC-B** — restore | **113 passed, 0 failed**; `diff` vs backup byte-identical |
| `2026-07-30` | `.4` | **NC-C** — cross-wire `dst.saw_flop_merge \|= src.saw_semantic_gate_merge` (the slip leg 1 **cannot** see) | **leg 1 stayed GREEN** (7 passed / 1 failed); only leg 2 fired: *"merging only `saw_flop_merge` must move only `saw_flop_merge` — left: [], right: [\"saw_flop_merge\"]"*. The two-leg design justified by measurement, not assertion |
| `2026-07-30` | `.4` | **NC-D** (the R2 leg) — add a field to `CoverageSummary`, touch nothing else | **`E0063`** at the fixture — compiler-maintained; restored byte-identical |
| `2026-07-30` | `.4` | Diff scope | `src/bin/tool_matrix.rs` only (two derives + a `#[serde(default)]` on a private struct, inert for the run path; the rest is the test module) ⇒ **DUT byte-identical** |
| `2026-07-30` | `.6` | Every non-test line in `src/mcp/mod.rs` naming ≥ 2 adapter ids, vs `adapters()` | **7 copies**, not the 4 `.2` counted: the four schema `enum`s (`:278`/`:303`/`:327`/`:458`), the `parse_validate_tools` error (`:1649`), `validate`'s description (`:520`), the server `instructions` (`:210`) |
| `2026-07-30` | `.6` | **The shadow measured against the registry — a LIVE defect, not a latent one** | `instructions` named **3 of 5** (missing `sv2v`, `slang`); `validate`'s description **4 of 5** (missing `slang`). The API described itself as accepting less than it accepts, undetected across two adapter-landing slices |
| `2026-07-30` | `.6` | `AcceptanceTool::from_name`'s `_ => None` catch-all — a candidate shadow | **already guarded, no new test**: `adapter_registry_lists_the_originals_then_new_adapters` (`src/downstream/mod.rs:2360`) loops every `adapters()` id through `from_name`. A second guard would duplicate a mechanism (`feedback_full_factorization`) |
| `2026-07-30` | `.6` | Live `anvil-mcp` `initialize` + `tools/list`, read back | all four `tools` `enum`s = `[verilator, yosys, iverilog, sv2v, slang]`; instructions + `validate` description name all five |
| `2026-07-30` | `.6` | `cargo check --all-targets` · `clippy --all-targets -- -D warnings` · `fmt --all --check` | exit `0` / `0` / `0` |
| `2026-07-30` | `.6` | `cargo test --lib mcp::` | **103 passed, 0 failed** (was 100 — the three new guards) |
| `2026-07-30` | `.6` | `cargo test` (full suite, under `ram_guard.sh --threshold 90`) | exit `0` — **17 test binaries, 1045 passed, 0 failed, 18 ignored** |
| `2026-07-30` | `.6` | `cargo test --test snapshots` | **6 passed, 0 failed** — byte-identical |
| `2026-07-30` | `.6` | **NC-A (the live defect, replayed)** — retype the pre-fix 3-adapter `instructions` prose | **FAILED** — *"the server instructions must name every vetted adapter; missing `sv2v`"*. The guard reproduces the defect that was actually shipped, not a hypothetical |
| `2026-07-30` | `.6` | **NC-B** — restore | green; `diff` vs the pre-NC backup byte-identical |
| `2026-07-30` | `.6` | **NC-C** — retype `hunt`'s schema `enum` as a hand-written 4-entry literal (the realistic future regression) | **FAILED** — *"tool `hunt` advertises a `tools` allow-list that is not the adapter registry"*, with both sides printed. Names the offending tool, not just the mismatch |
| `2026-07-30` | `.6` | **NC-D** — restore | green; `diff` vs backup byte-identical |
| `2026-07-30` | `.6` | **NC-E (the acceptance criterion, demonstrated)** — add a sixth `ProbeAdapter` to `ADAPTER_REGISTRY` and change **nothing** under `src/mcp/` | all four schema `enum`s, both prose sites, and the allow-list error picked up `probe`; the three derived guards stayed **green** because they derive too. Separately confirmed the registry's own landing pin (`adapter_registry_lists_the_originals_then_new_adapters`) fails on the 6th entry ⇒ the registry itself is not silently growable. `src/downstream/mod.rs` restored byte-identical |
| `2026-07-30` | `.6` | Diff scope | `src/mcp/mod.rs` only; MCP is beside the generator (no `src/gen`/`src/emit`/`src/ir`/`src/config`) ⇒ **DUT byte-identical** |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `.1` | `SHADOW-ENUMERATION-SWEEP.1 — register: lists that shadow a growing set` | Docs-only registration + audit |
| `.2` | `SHADOW-ENUMERATION-SWEEP.2 — decision 0033: what is a shadow, and what is not` (`9ffabea`) | Docs-only design ADR; adds `.5`/`.6`/`.7`, re-scopes `.4` |
| `.3` | `SHADOW-ENUMERATION-SWEEP.3 — one GATES table; the gate that could not fail` (`4f9720f`) | The S3 fix: `static GATES` + four derived sites + four derived guards |
| `.5` | `SHADOW-ENUMERATION-SWEEP.5 — the applier the compiler now maintains` (`9a082e9`) | The S2 fix: an E0063-enforced fixture + a serde-derived expectation |
| `.4` | `SHADOW-ENUMERATION-SWEEP.4 — 149 merges, two legs, one blind spot closed` (`25b4ebf`) | The S1 fix: the `.5` pattern reused; NC-C proves leg 2's independent value |
| `.6` | `SHADOW-ENUMERATION-SWEEP.6 — the allow-list the API now reads back` | The S1 fix: one `registered_adapter_ids()` behind seven sites; the tree's first **live** defect (two prose sites already stale) |

## Changelog

- `2026-07-30`: Created on owner directive from three same-shape bugs landed in one
  session (`EMIT-SURFACE-INTERACTION-GATE.2` preset knob list,
  `EMIT-SURFACE-INTERACTION-GATE.4` consumer census,
  `EVIDENCE-BANK-DURABILITY.6` gate-flag list), plus two further sites measured while
  registering: the seven per-gate `tool_matrix` enumerations (five silent, one of
  which yields a gate that cannot fail) and the 149-field `merge_coverage`.
- `2026-07-30` (`.6`): **prose is in scope.** `.2`'s audit classified only structured
  copies (a `enum` array, a `match`, a merge line). `.6` extends the class to
  natural-language API text — the server `instructions` and a tool `description` — on the
  grounds that under decision `0017` those *are* the contract an agent reads first, and
  they satisfy all three tests of rule (a) exactly as a schema does. That extension is
  what surfaced the tree's first live defect; a shape-keyed audit had walked past it
  twice.
- `2026-07-30` (`.6`): **an existing derived guard is not re-guarded.**
  `AcceptanceTool::from_name`'s `_ => None` catch-all reads as an unguarded shadow, but
  `adapter_registry_lists_the_originals_then_new_adapters` already loops every registry id
  through it. Left alone per `feedback_full_factorization` (one mechanism per job), and
  recorded here so the next audit does not "fix" it into a duplicate.
- `2026-07-30`: `.2` done — decision
  [`0033`](../decisions/0033-shadow-enumeration-classification.md). Supplies the
  three-question classification rule (derivable ∧ growth-coupled ∧ silent), the
  four-rung repair ladder (derive → make loud → derived-expectation test → registered
  doctrine), the honest *not-mechanizable-as-discovery* verdict, the S3/S2/S1 severity
  tiers, and a **20-site audit** that measures every count from the tree. Grows the tree
  from four leaves to seven (`.5` `apply_cli_overrides`, `.6` the MCP adapter-id schema
  enums, `.7` the `ENUMERATION-PARITY` docs/script doctrine) and re-scopes `.4` from
  "largest surface" to S1 after measuring that `merge_coverage` is monotone and 134 of
  its 149 fields are gated. Docs-only ⇒ DUT byte-identical.
- `2026-07-30`: `.6` done — the MCP adapter-id allow-list is now **derived** from the
  closed `ADAPTER_REGISTRY` at every agent-facing site (`registered_adapter_ids()`), and
  the audit's four copies turned out to be seven. Two of them — the server `instructions`
  and `validate`'s description — had **already fallen behind the registry**, making this
  the tree's first *live* defect rather than a latent one: the API advertised three and
  four adapters respectively against a registry of five. Three derived guards added, none
  a new list; NC-E adds a sixth adapter to the registry alone and watches all seven sites
  follow. Frontier `.7`, the last leaf. `src/mcp/mod.rs` only ⇒ DUT byte-identical.
