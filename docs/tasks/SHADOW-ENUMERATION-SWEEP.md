# SHADOW-ENUMERATION-SWEEP: hand-maintained lists that shadow a set which already exists

## Metadata

- Tree ID: `SHADOW-ENUMERATION-SWEEP`
- Status: `active`
- Roadmap lane: Quality / defect-class elimination (cross-cutting; no phase reopened)
- Created: `2026-07-30`
- Last updated: `2026-07-30` (`.2` done — decision
  [`0033`](../decisions/0033-shadow-enumeration-classification.md))
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
  Status: `pending`
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

- ID: `SHADOW-ENUMERATION-SWEEP.6`
  Status: `pending`
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
  Status: `pending`
  Goal: the highest-severity fix — the `tool_matrix` gate-flag sites, above all the
        `fail_on_coverage_gap` or-chain whose omission yields a gate that cannot
        fail. Candidate shape: one `const GATES: &[GateSpec]` table (flag name,
        `ScenarioSet`, min units) that every site reads, so a gate is declared once;
        or, if a table is too invasive, a test that derives the expected flag set
        from the `Cli` struct and asserts every site covers it.
  Acceptance: adding a gate cannot silently miss a site; negative-controlled;
        `tests/snapshots.rs` untouched.

- ID: `SHADOW-ENUMERATION-SWEEP.4`
  Status: `pending`
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

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `.3` | `pending` | **Current frontier.** The only **S3** site in the 20-site audit: a gate flag missing from the `fail_on_coverage_gap` or-chain (`src/bin/tool_matrix.rs:1564`) leaves `:1490`'s `bail!` disarmed, so the gate runs, computes gaps, ignores them and **exits 0** — a false green that decision `0030` then banks as a committed digest and `README.md` cites. With a missing `select_scenario_set` arm it is worse: the flag silently selects `ScenarioSet::Default`, so the gate runs the wrong scenarios and still exits 0. |
| 2 | `.5` | `pending` | **S2.** `Config::apply_cli_overrides`, 87 fields, guarded by nothing; an omission makes a CLI flag *and every preset that sets that knob* silently inert — the exact shape of the decision-`0032` preset bug, since presets apply through the same applier. |
| 3 | `.4` | `pending` | **S1.** Re-scoped by decision `0033` §3: monotone merges + 134/149 gated ⇒ loud; 15 genuinely-silent fields, under-reporting only. Cheap round-trip guard; real but not urgent. |
| 4 | `.6` | `pending` | **S1.** Four MCP adapter-id JSON-schema `enum` literals shadowing `ADAPTER_REGISTRY`; a sixth adapter would be usable but unadvertised to agents (decision `0017`). |
| 5 | `.7` | `pending` | **S1.** The two docs/script pairs — the only sites with no compiler and no `cargo test`, so the only ones that need a registered doctrine (`ENUMERATION-PARITY`). |
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

**Open for `.3`+:**

- `.3`'s shape: one `const GATES: &[GateSpec]` table that all six silent sites read
  (**R1** — the flag stops being enumerable more than once) versus a derived test that
  reads the `Cli` gate-flag set and asserts each site covers it (**R3**). R1 is preferred
  by the ladder, but the six sites want different payloads (a `ScenarioSet`, a min-units
  constant, a report field, a display name), so `.3` decides against the real code.
- Can `.3` reach **R2** instead, by deriving `fail_on_coverage_gap` from the selected
  `ScenarioSet` — already a compiler-enforced enum — rather than from a parallel
  disjunction over flags? If so the S3 site **disappears** rather than being guarded.
- Should `ENUMERATION-PARITY`'s declared-pairs table live inside the check script, or in
  a small tracked data file the check reads?

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

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `.1` | `SHADOW-ENUMERATION-SWEEP.1 — register: lists that shadow a growing set` | Docs-only registration + audit |
| `.2` | `SHADOW-ENUMERATION-SWEEP.2 — decision 0033: what is a shadow, and what is not` | Docs-only design ADR; adds `.5`/`.6`/`.7`, re-scopes `.4` |

## Changelog

- `2026-07-30`: Created on owner directive from three same-shape bugs landed in one
  session (`EMIT-SURFACE-INTERACTION-GATE.2` preset knob list,
  `EMIT-SURFACE-INTERACTION-GATE.4` consumer census,
  `EVIDENCE-BANK-DURABILITY.6` gate-flag list), plus two further sites measured while
  registering: the seven per-gate `tool_matrix` enumerations (five silent, one of
  which yields a gate that cannot fail) and the 149-field `merge_coverage`.
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
