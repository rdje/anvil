# SHADOW-ENUMERATION-SWEEP: hand-maintained lists that shadow a set which already exists

## Metadata

- Tree ID: `SHADOW-ENUMERATION-SWEEP`
- Status: `active`
- Roadmap lane: Quality / defect-class elimination (cross-cutting; no phase reopened)
- Created: `2026-07-30`
- Last updated: `2026-07-30`
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
  Children: `.1` (register + audit), `.2` (design ADR), `.3`+ (one fix per site)

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
  Status: `pending`
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
  Goal: guard `merge_coverage` — 149 hand-merged `CoverageSummary` fields with no
        divergence check. Likely repair: a test that serialises a `CoverageSummary`
        with every `bool` set `true` / every set non-empty, merges it into
        `default()`, and asserts the result equals the source — which fails for any
        field the merge forgot, without enumerating the fields.
  Acceptance: the test fails when a merge line is deleted; passes when restored;
        no new hand-maintained list introduced by the fix itself.

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `.2` | `pending` | **Current frontier.** Design-first, per the lane's cadence. The classification rule must exist before any site is touched — the non-goals show that "looks like a shadow" and "is a shadow" are different, and getting that wrong would damage load-bearing allow-lists. |
| 2 | `.3` | `pending` | Highest severity: a silently-omitted `fail_on_coverage_gap` produces a gate that cannot fail, and its output would be banked as clean evidence. |
| 3 | `.4` | `pending` | Largest surface (149 entries), currently correct; the round-trip test is cheap and needs no per-field list. |
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

## Open Questions

- Is the class mechanizable at all? "List L mirrors set S" is not syntactically
  detectable in general. The realistic ceiling may be a per-site derived test rather
  than a registered doctrine — `.2` must give an honest verdict, and "not
  mechanizable, here is why" is an acceptable outcome (`DOCTRINE_ENFORCEMENT.md` §9
  documents honest limits).
- Where else does the repo shadow a growing set? The audit so far is opportunistic —
  three bugs plus two sites found while writing this file. `.2` should decide whether
  a systematic search is worth it, or whether the pattern is rare enough that
  fixing the known instances plus recording the rule is the better trade.
- Does `src/config.rs`'s `Overrides` → `apply_cli_overrides` field-by-field
  application have the same shape? Unaudited; a forgotten field would mean a CLI flag
  that silently does nothing.

## Blockers

- None. `.2` is a docs/decision leaf and needs no tool run.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-07-30` | `.1` | `CoverageSummary` struct fields vs `merge_coverage` `dst.` assignments | **149 / 149** — complete today, guarded by nothing |
| `2026-07-30` | `.1` | Every non-test line in `src/bin/tool_matrix.rs` naming one specific gate | **7 production sites**; 5 silent on omission, 2 compiler-enforced |
| `2026-07-30` | `.1` | `src/bin/tool_matrix.rs:1490` read directly | `fail_on_coverage_gap` gates the `bail!`, so omitting a gate from its or-chain yields a gate that exits 0 with non-empty `coverage_gaps` |
| `2026-07-30` | `.1` | `MEMORY_ARCHITECTURE.md` §71 | layer A is "Overwritten each update; hard size cap" — confirms a `MEMORY.md`-only finding is not retained |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `.1` | `SHADOW-ENUMERATION-SWEEP.1 — register: lists that shadow a growing set` | Docs-only registration + audit |

## Changelog

- `2026-07-30`: Created on owner directive from three same-shape bugs landed in one
  session (`EMIT-SURFACE-INTERACTION-GATE.2` preset knob list,
  `EMIT-SURFACE-INTERACTION-GATE.4` consumer census,
  `EVIDENCE-BANK-DURABILITY.6` gate-flag list), plus two further sites measured while
  registering: the seven per-gate `tool_matrix` enumerations (five silent, one of
  which yields a gate that cannot fail) and the 149-field `merge_coverage`.
