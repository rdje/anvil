---
id: shadow-enumeration-classification
title: A hand-maintained list is a **shadow enumeration** only if it is derivable, growth-coupled, and silent on omission — the three-question rule, the four standard repairs, and the honest verdict that the class is not mechanizable as a discovery doctrine
answers:
  - "how do I tell an authoritative enumeration from a shadow one"
  - "when must a hand-maintained list in anvil be derived or guarded"
  - "why are check_no_boot_volume_refs's allow-list and the EVIDENCE-CITATIONS §1 pin not shadow enumerations"
  - "what is the standard repair for a list that mirrors a growing set"
  - "is the shadow-enumeration defect class mechanizable as a registered doctrine"
  - "why does anvil not add a doctrine check that finds shadow enumerations"
  - "which hand-maintained lists in anvil are silent when they fall behind"
  - "how severe is a forgotten merge_coverage line"
  - "can a forgotten merge_coverage line make a tool_matrix gate report false coverage"
  - "why can omitting a gate flag from tool_matrix produce a gate that cannot fail"
  - "what guards Config::apply_cli_overrides against a forgotten knob"
  - "what is the model derived-expectation test in anvil"
date: 2026-07-30
status: accepted
tags: [quality, doctrine, enumeration, shadow-list, derivation, drift, tool-matrix, config, coverage, mcp, adapter, book, enforcement, north-star]
evidence: src/bin/tool_matrix.rs:70-231 (the `Cli` gate-flag fields — 15), :952-970 (`ScenarioSet` — 15 variants, compiler-enforced), :1136-1230 (`MatrixReport` gate fields — 15), :1490 (the `fail_on_coverage_gap`-gated `bail!`), :1526-1582 (`derive_run_plan` + the or-chain), :1584-1636 (`select_scenario_set` + the exclusivity sum), :642-800 (`CoverageSummary` — 149 fields), :8484-8720 (`merge_coverage` — 149 merged, 135 `|=` + 13 `.extend` + 1 `.max`, all monotone), :8938+ (`compute_coverage_gaps` — every gap is `if !coverage.saw_x`, 134 of 149 fields referenced); src/config.rs:1734 (`apply_cli_overrides` — 87 fields, unguarded), :2005 (`Overrides` — 87 fields), :2154 (`presets()` — authoritative), :2401 (`knob_catalog()`), :2664 (`knob_catalog_classifies_every_field` — the model derived test), :2569 (`structured_emission_max_preset_covers_every_non_version_gated_surface` — the second model); src/main.rs:1066 (`cli_overrides` — exhaustive struct literal, compiler-enforced); src/downstream/mod.rs:1080 (`ADAPTER_REGISTRY` — 5), src/mcp/mod.rs:278,303,327,458 (4 hardcoded adapter-id JSON-schema enums); scripts/check_doctrines.sh:30-38 (`DOCTRINES` — 6, authoritative, meta-checked) vs DOCTRINE_ENFORCEMENT.md §10 (6 rows, unguarded); book/src/SUMMARY.md (29 links) vs book/src/*.md (29 chapters + SUMMARY). All counts measured `2026-07-30`.
---

# 0033 - SHADOW-ENUMERATION-SWEEP: the classification rule, the repair ladder, and why this class is guarded per site rather than discovered by a doctrine

- Date: 2026-07-30
- Status: accepted
- Tree: `SHADOW-ENUMERATION-SWEEP.2` (design leaf; decides the classification rule, the
  standard repair per class, the mechanizability verdict, and the `.3`+ order)
- Activated by: autonomous PNT selection under the owner's standing **DECIDE, DON'T ASK**
  directive, from the frontier row set by `.1`.

## Context

`.1` opened this tree on a measured observation: three bugs of one shape landed in a
single session, each in a different language and subsystem, each a **hand-maintained list
that mirrored a set which already existed somewhere else** and fell silently behind when
that set grew. `.1` then measured two further sites and stopped, deliberately, before
touching anything — because the tree's own Non-Goals show that *"looks like a shadow"*
and *"is a shadow"* are different, and getting that difference wrong would damage
load-bearing allow-lists that the repo depends on.

This leaf supplies the missing definition, and the audit that tests it.

### 1. The rule has to survive its hardest cases first

Four enumerations in this repo look exactly like the three that failed, and **must not be
touched**:

- `presets()` (`src/config.rs:2154`) — the curated `--profile` registry. It *is* the
  source of truth for what a preset means. There is no other set to derive it from.
- the `DOCTRINES` array (`scripts/check_doctrines.sh:30`) — the doctrine registry.
  Likewise authoritative; `DOCTRINE_ENFORCEMENT.md` §5 says so explicitly.
- `check_no_boot_volume_refs.sh`'s allow-list — load-bearing *because* it differs from
  "every tracked file": a policy document must be allowed to name the string it forbids,
  and `CHANGES.md` / `DEVELOPMENT_NOTES.md` are append-only history the owner has directed
  must stay raw (decision `0031`). Deriving it would pressure authors into the history
  rewrite the project forbids.
- `check_evidence_citations.sh`'s §1 grandfathered list — pinned by entry count **and**
  membership SHA-256 *because it is a historical fact* (decision `0030`). It must be
  incapable of growing. Deriving it would make the doctrine decorative in one line.

Any rule that classifies these as shadows is wrong, no matter how well it handles the easy
cases. So the rule is written to fail on them first.

### 2. The audit (measured `2026-07-30`, every count re-derived from the tree)

`✓` = the rule's test passes (the property holds). Verdict is the rule of §Decision (a).

| # | site | shadow list | authoritative set S | (1) derivable | (2) growth-coupled | (3) silent | verdict |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | `tool_matrix.rs:1564` `fail_on_coverage_gap` or-chain | 15 gate flags | `Cli` gate-flag fields (15) | ✓ | ✓ | ✓ | **silent shadow** |
| 2 | `:1584` `select_scenario_set` if-chain | 15 | same | ✓ | ✓ | ✓ | **silent shadow** |
| 3 | `:1526` `derive_run_plan` if-chain | 15 | same | ✓ | ✓ | ✓ | **silent shadow** |
| 4 | `:1585` mutual-exclusion sum | 15 | same | ✓ | ✓ | ✓ | **silent shadow** |
| 5 | `:1136` `MatrixReport` gate fields | 15 | same | ✓ | ✓ | ✓ | **silent shadow** |
| 6 | `:1602` `bail!` prose | 15 names | same | ✓ | ✓ | ✓ | **silent shadow** (cosmetic) |
| 7 | `:70` `Cli` fields · `:1418` report assignment · `:952` `ScenarioSet` + its `match` | 15 each | — | ✓ | ✓ | ✗ (clap derive / E0063 / non-exhaustive `match`) | guarded — **leave** |
| 8 | `config.rs:1734` `apply_cli_overrides` | 87 `if let Some` | `Overrides` fields (87) | ✓ | ✓ | ✓ | **silent shadow** |
| 9 | `main.rs:1066` `cli_overrides` | 87 | `Overrides` | ✓ | ✓ | ✗ (exhaustive struct literal, no `..`) | guarded — **leave** |
| 10 | `tool_matrix.rs:8484` `merge_coverage` | 149 merges | `CoverageSummary` (149) | ✓ | ✓ | **partly** (15 of 149) | **silent shadow** (re-scoped — §3) |
| 11 | `mcp/mod.rs:278,303,327,458` adapter-id JSON `enum` | 5 ids × 4 copies | `ADAPTER_REGISTRY` (5) | ✓ | ✓ | ✓ | **silent shadow** |
| 12 | `DOCTRINE_ENFORCEMENT.md` §10 table | 6 rows | `DOCTRINES` array (6) | ✓ | ✓ | ✓ | **silent shadow** (docs) |
| 13 | `book/src/SUMMARY.md` | 29 links | `book/src/*.md` (29) | ✓ | ✓ | ✓ | **silent shadow** (docs) |
| 14 | `config.rs:2401` `knob_catalog()` | one row per knob | `Config` serde fields | ✓ | ✓ | ✗ (`knob_catalog_classifies_every_field`) | **already repaired — the model** |
| 15 | `config.rs:2569` preset ↔ surface list | 8 knobs | `knob_catalog()` group | ✓ | ✓ | ✗ (the `0032` drift test) | **already repaired — the model** |
| 16 | `config.rs:2154` `presets()` | 4 presets | **none** | ✗ | — | — | **authoritative — leave** |
| 17 | `check_doctrines.sh:30` `DOCTRINES` | 6 | **none** | ✗ | — | — | **authoritative — leave** |
| 18 | `check_no_boot_volume_refs.sh` allow-list | — | tracked files | ✓ | **✗ — must differ** | — | **authoritative — leave** |
| 19 | `check_evidence_citations.sh` §1 pin | — | historical banks | ✓ | **✗ — must not grow** | — | **authoritative — leave** |
| 20 | `mcp/mod.rs:226` `tools_list` vs the dispatcher | 10 names | — | ✓ | ✓ | ✗ (unknown name ⇒ a JSON-RPC error at call time) | loud at runtime — **leave** |

Rows 16–19 are the four hard cases of §1, and the rule rejects all four — 16 and 17 on
test (1) (no S exists), 18 and 19 on test (2) (S exists, but the list is *supposed* to
differ from it). That is the rule earning its keep.

Rows 8, 11, 12 and 13 are **new finds** from this leaf; `.1`'s audit was opportunistic and
did not reach them.

### 3. Correcting `.1` on `merge_coverage` — measured, and it changes the ordering

`.1` ranked `merge_coverage` as *"the single highest-value derived-check candidate in the
repo"* on the strength of its size (149 hand-merged fields). Re-measured directly, that
ranking is wrong, and the reason is worth recording because it is the difference between
severity and surface area.

**Measured — every merge is monotone.** The 149 merges are `135 × |=` + `13 × .extend()` +
`1 × .max()`. There is no assignment, no overwrite, no subtraction. So a forgotten line can
only leave `dst.field` at its `default()` — `false`, empty, `0`. **The omission
under-reports; it can never over-report.**

**Measured — 134 of the 149 fields are gated, and gated means loud.** Every gap in
`compute_coverage_gaps` has the shape `if !coverage.saw_x { gaps.push(…) }` — positive
polarity, without exception. So for a gated fact, a forgotten merge yields `false` ⇒ a
**spurious gap** ⇒ under any gate (which sets `fail_on_coverage_gap`) the run **bails**.
That is a loud failure, arriving at the next gate run.

Only **15** of the 149 fields are never referenced by `compute_coverage_gaps`:

```
hierarchy_child_instance_counts · hierarchy_child_instance_override_profiles ·
knob_fires_seen · saw_acceptance_divergence ·
saw_design_with_cross_simulator_agreement · saw_flop_merge · saw_semantic_gate_merge ·
saw_hierarchy_registered_multistage_parent_composed_parent_cone_instance_routing ·
saw_recursive_hierarchy_parent_composed_parent_cone_instance_flop_mixed_support_routing ·
saw_recursive_hierarchy_registered_multistage_parent_composed_parent_cone_instance_routing ·
saw_recursive_hierarchy_depth_{3,4,5,6,7}_stateful_parent_composed_mixed_support_child_inputs
```

For those, a forgotten merge is genuinely silent — and its effect is to **understate** a
fact the report records but no gate checks. Several are quoted in `README.md` as
`… = true` evidence of Phase-4 hierarchy coverage; understating one means a *claim
we could have made and did not*, never a claim we made falsely.

So the real silent surface of `merge_coverage` is **15 fields, fail-safe in direction** —
not 149 in the dangerous direction. It stays in scope (a round-trip guard is cheap and the
15 are real), but it is **not** the second-highest-severity site. It drops behind
`apply_cli_overrides`.

This is the tree's own doctrine applied to the tree: *the fixture agrees with you; the tool
does not*. `.1` reasoned from the list's length; `.2` measured what an omission actually
does.

### 4. The one severity that is genuinely S3

`src/bin/tool_matrix.rs:1490`:

```rust
if plan.fail_on_coverage_gap && !report.coverage_gaps.is_empty() { bail!(…) }
```

A gate flag missing from the `:1564` or-chain leaves `fail_on_coverage_gap = false`, so
the gate **runs, computes its gaps, ignores them, and exits 0**. Combined with a missing
`select_scenario_set` arm it is worse still: the flag silently selects `ScenarioSet::Default`,
so the gate runs *the wrong scenarios* and exits 0 having proven nothing about its surface.
Under decision `0030` that clean exit is then banked as a committed closure digest and
cited in `README.md`.

A gate that cannot fail is worse than no gate: it manufactures confidence, and every
downstream mechanism in this repo faithfully records it. This is the only site in the audit
whose omission produces a **false green**.

## Decision

### (a) The classification rule — three questions, asked in order

> A hand-maintained list **L** is a **shadow enumeration** if and only if **all three**
> hold:
>
> 1. **Derivable.** Another artifact **S** in the repository already enumerates the same
>    membership, and S is reachable from L's site by ordinary program or script means — a
>    struct's serde projection, a catalog function, a `static` registry, the JSON being
>    parsed, a directory listing, `git ls-files`.
>    *Fails ⇒ **L is authoritative**: it is the source of truth. Leave it hand-written.*
> 2. **Growth-coupled.** S is expected to grow, and every growth of S **requires** a
>    matching entry in L for the system to be correct.
>    *Fails ⇒ **L is authoritative**: L is supposed to differ from S — a deliberate
>    allow-list, a curated subset, or a frozen historical fact. Deriving it would destroy
>    the very property it exists to hold.*
> 3. **Silent.** Omitting an entry produces no compile error, no failing test, and no
>    runtime error on the normal path.
>    *Fails ⇒ **L is already guarded**. A non-exhaustive `match`, an exhaustive struct
>    literal, or an existing derived test is a working guard. Leave it.*
>
> All three ⇒ **silent shadow: repair it** by the ladder in (b).

Test (2) is the load-bearing one, and it is why the rule is stated as a *conjunction* rather
than as "L duplicates S". `check_no_boot_volume_refs.sh`'s allow-list and
`check_evidence_citations.sh`'s §1 pin both satisfy (1) — the set of tracked files, the set
of historical banks, are both derivable — and both are authoritative anyway, because the
gap between L and S **is the content of the rule**. Any formulation that omits (2) deletes
two load-bearing doctrines.

Test (3) is what keeps this a defect-class elimination and not a refactor: a list that fails
loudly already has its guard, and replacing it with a derivation buys nothing while risking
a regression.

### (b) The repair ladder — four rungs, in strict preference order

Mirrors `DOCTRINE_ENFORCEMENT.md` §3's archetype preference: prefer the repair that cannot
be faked or forgotten over the one that must be remembered.

- **R1 — Derive: delete the list.** Replace every site with one table/registry that all of
  them read, so the entry is declared **once** and the list stops existing. Strongest —
  there is nothing left to fall behind. Cost: invasive when the sites disagree on shape.
- **R2 — Make it loud: convert silence into a compile error.** Restructure so the omission
  cannot build: an exhaustive struct literal with no `..`, a non-exhaustive `match` over an
  enum, a fixed-length array. Free forever once done, and it needs no test to be remembered.
  This is what already protects rows 7 and 9 of the audit.
- **R3 — Guard: derive the *expectation* in a test.** Keep the list; add **one** test that
  computes S at run time and asserts L covers it. The two in-repo exemplars are the model
  and must be copied rather than reinvented:
  - `knob_catalog_classifies_every_field` (`src/config.rs:2664`) derives the expected set
    from `serde_json::to_value(Config::default())` — the real struct — and asserts set
    equality.
  - `structured_emission_max_preset_covers_every_non_version_gated_surface`
    (`src/config.rs:2569`, decision `0032`) derives it from `knob_catalog()`'s
    `structured_emission` group.
- **R4 — Gate: a registered doctrine check.** For sites with no compiler and no
  `cargo test` — a Markdown table shadowing a shell array, a `SUMMARY.md` shadowing a
  directory — write a `scripts/check_*.sh` obeying the §4 contract and register it in
  `DOCTRINES`. This is the *only* rung available off the Rust side.

Two constraints bind every rung:

1. **The repair may not introduce a new hand-maintained list.** A guard that is itself a
   second copy has moved the defect, not removed it. If the fix needs an expected set, it
   derives it — the R3 exemplars do exactly this.
2. **Negative-control in both directions.** Delete one entry ⇒ the guard fails; restore it
   ⇒ the guard passes. A guard nobody has watched fail is a guard nobody knows works. The
   `.1` acceptance criteria already require this; it is restated here as part of the
   standard repair, not as a per-leaf nicety.

### (c) Mechanizability: **not** as a discovery doctrine — deliberately, and here is why

The tree's Open Question asked for an honest verdict. It is **no**, and the honest form of
the no matters more than the answer.

Test (1) of the rule — *"another artifact already enumerates the same membership"* — is a
**semantic** relation between two sets, not a syntactic property of either. Nothing in the
token stream distinguishes

```rust
if let Some(v) = o.max_depth { self.max_depth = v; }        // a shadow of Overrides
```

from a deliberately partial application, or `presets()` from `DOCTRINES` from
`check_no_boot_volume_refs.sh`'s allow-list — all three are `Vec`-ish literals of records.
A detector would have to already know **which pairs of sets are supposed to correspond**,
and choosing those pairs is exactly the human judgement the rule in (a) encodes.

A guessing check has only two failure modes, and both are worse than nothing:

- **Miss.** It ignores a real shadow, and the repo now believes the class is covered — the
  false-confidence failure this tree exists to eliminate, reproduced at the meta level.
- **Cry wolf.** It flags `presets()` or an allow-list on every commit. Per the standing
  gotcha earned by `EVIDENCE-CITATIONS`: **a gate that cries wolf gets deleted**, and its
  deletion takes the real coverage with it.

`DOCTRINE_ENFORCEMENT.md` §9 exists for exactly this — state the limit, do not over-claim.

**What *is* mechanizable is holding the pairs we have already classified**, and the right
mechanism is decided by which side of the language boundary the site sits on:

- **Rust-side shadows → an in-crate `#[test]`, no new doctrine.** They already run under
  `cargo test`, which is `COMMIT.md`'s mandatory gate and CI's. Registering a shell doctrine
  that re-checks what `cargo test` checks would be a second mechanism for one job —
  precisely what `feedback_full_factorization` forbids. Rows 1–6, 8, 10 and 11 take this
  route.
- **Docs/script-side shadows → one new registered doctrine.** Rows 12 and 13 have no
  compiler and no `cargo test`; R4 is their only rung. They get **one** check
  (`ENUMERATION-PARITY`) that reads a small declared table of
  `(shadow site, authoritative source, extractor)` triples and diffs each pair. The table of
  *pairings* is itself authoritative under rule (a) — there is no S from which "which pairs
  we care about" could be derived — so the mechanism does not recurse.

The honest summary: **this class is discovered by review and held by derivation.** The
review half is what `.1` and this leaf are; the derivation half is `.3`–`.7`.

### (d) The `.3`+ order, by severity of silent failure

Severity is defined as **what one forgotten line corrupts**, not how many lines the list has:

| tier | the omission causes | why it ranks there |
| --- | --- | --- |
| **S3** | a gate reports success it did not earn | the result is banked as closure evidence (decision `0030`) and cited in live docs. A **false green**. |
| **S2** | a user-requested behaviour silently does not happen | the run succeeds and the output is wrong-by-omission; nothing reports it. |
| **S1** | an advertised contract or report understates reality | fail-safe in direction — no false claim is made — but an interface diverges from the code. |

| order | leaf | site | tier | why here |
| --- | --- | --- | --- | --- |
| 1 | `.3` | `tool_matrix` gate-flag sites (6 silent of 7) | **S3** | the only false-green site in the whole audit (§4). |
| 2 | `.5` | `Config::apply_cli_overrides` (87) | **S2** | a knob added to `Overrides` but forgotten here makes its **CLI flag and every preset that sets it silently inert** — which is, exactly, the bug decision `0032` found in `structured-emission-max`. Unguarded; presets flow through this same applier (`config.rs:2465`). |
| 3 | `.4` | `merge_coverage` (15 silent of 149) | **S1** | re-scoped by §3: monotone merges ⇒ under-report only; 134 of 149 fields are gated ⇒ loud. Cheap round-trip guard, real but not urgent. |
| 4 | `.6` | MCP adapter-id JSON-schema `enum` ×4 | **S1** | a sixth adapter is usable by `validate`/`hunt`/`divergence` yet unadvertised to agents — an API-contract divergence, against decision `0017`'s API-first mandate. |
| 5 | `.7` | `ENUMERATION-PARITY` doctrine: §10 table ↔ `DOCTRINES`; `SUMMARY.md` ↔ `book/src/*.md` | **S1** | a documented-but-unregistered doctrine is a "trust me" rule (`DOCTRINE_ENFORCEMENT.md` §11); an unlinked chapter is written and **never rendered**, and the book is the owner's only window into the project. |

Leaf **numbers** stay in creation order — a leaf id is the durable join key between a commit
and the tree and is never renumbered — so the **execution** order above is carried by the
tree's `Current Frontier` table, which exists for this.

## Decisive test applied

*"Does the rule reject the four enumerations that must stay hand-written, and accept the
three that already failed?"* Yes on both, and that is the whole bar. It rejects `presets()`
and `DOCTRINES` on test (1), and `check_no_boot_volume_refs.sh`'s allow-list and the
`EVIDENCE-CITATIONS` §1 pin on test (2). It accepts all three of the originating bugs — the
`structured-emission-max` knob list, `compute_use_counts`'s consumer census, and
`evidence_digest.sh`'s gate list — on all three tests. A rule that passed only the easy half
would have licensed exactly the damage the tree's Non-Goals forbid.

## Rejected alternatives

- **A `SHADOW-ENUMERATION` discovery doctrine that scans for repeated literal lists.**
  Rejected — §(c). It cannot distinguish an authoritative registry from a shadow without
  already knowing the pairing, so it either misses (false confidence) or cries wolf (and is
  deleted, taking the real coverage with it).
- **Defining a shadow as "L duplicates S"** (dropping test (2)). Rejected — it classifies
  `check_no_boot_volume_refs.sh`'s allow-list and the `EVIDENCE-CITATIONS` §1 pin as
  shadows, and "fixing" them would delete decision `0031`'s history-stays-raw guarantee and
  decision `0030`'s un-growable grandfather list. The gap between L and S *is* the rule in
  both cases.
- **Ordering `.3`+ by list length.** Rejected — measured wrong. `merge_coverage` is the
  longest list (149) and the *least* dangerous of the four (§3): monotone, 90% gated, and
  under-reporting. Severity is what an omission corrupts.
- **Fixing `merge_coverage` first because `.1` said so.** Rejected on the same measurement.
  Recorded rather than quietly reordered, because the correction — reasoning from a list's
  length instead of from what an omission does — is the reusable lesson.
- **Dropping `merge_coverage` from scope entirely** now that it is S1. Rejected: 15 fields
  are genuinely silent, the round-trip guard needs no per-field list (so it costs one test
  and never rots), and several of the 15 are cited in `README.md`.
- **Registering one doctrine per docs-side pair.** Rejected — two pairs today, and the
  registry is a curated surface; one `ENUMERATION-PARITY` check over a declared table of
  pairs scales without a registry line per site.
- **Registering an `ENUMERATION-PARITY` doctrine for the Rust sites too.** Rejected —
  `cargo test` is already the mandatory gate for those (`COMMIT.md` §1 + CI). A second
  mechanism for one job is what `feedback_full_factorization` forbids.
- **Renumbering `.4` and `.5` so leaf order matches severity order.** Rejected — the leaf id
  is the durable join key between commits and the tree (`COMMIT.md` task-tree rule #1). The
  tree's `Current Frontier` table already carries execution order.
- **Auditing every enumeration in the repo before fixing any.** Rejected — the audit here
  covers every site reachable from the three originating bugs plus a directed sweep of the
  registries (config, tool_matrix, MCP, downstream, doctrines, book). Exhaustiveness is not
  achievable for a semantic property (§(c)); the rule is recorded so the *next* site is
  classified in one reading rather than re-derived.

## Consequences

- The repo gains a **stated, testable rule** for a defect class it had been handling by
  diligence. New enumerations are classified by three questions, and the four hard cases
  are recorded as the rule's calibration.
- **The class is honestly declared not-mechanizable as discovery.** ANVIL adds no gate that
  guesses; it adds per-site derived guards plus one docs-side parity check over declared
  pairs. `DOCTRINE_ENFORCEMENT.md` §9's honest-limits discipline is followed rather than
  quoted.
- `.1`'s severity ranking is **corrected from measurement**, not from re-reading:
  `merge_coverage` drops from #2 to #3 (monotone + 134/149 gated ⇒ loud), and
  `apply_cli_overrides` — unaudited when the tree opened — takes #2 because its omission
  makes a CLI flag *and every preset that sets it* silently inert.
- Three sites the `.1` audit never reached are now tracked: `apply_cli_overrides`, the four
  MCP adapter-id schema enums, and the two docs-side pairs.
- The tree grows from four leaves to seven; `.3` is unchanged and remains the frontier's
  next action.
- Docs-only leaf: no `src/` change ⇒ **DUT byte-identical**, `tests/snapshots.rs` untouched.

## Open questions (for `.3`+)

- `.3`'s shape: a `const GATES: &[GateSpec]` table that all six silent sites read (**R1**,
  strongest — the flag stops being enumerable more than once) versus a derived test that
  reads the `Cli` gate-flag set and asserts each site covers it (**R3**, cheaper). R1 is
  preferred by the ladder; `.3` decides against the real code, since the six sites want
  different payloads (a `ScenarioSet`, a min-units constant, a report field, a display name).
- Whether `.3` can additionally reach **R2** for the or-chain by making
  `fail_on_coverage_gap` a *derived* property of the selected `ScenarioSet` — which is
  already a compiler-enforced enum — rather than a parallel disjunction over flags. If so,
  the S3 site disappears instead of being guarded.
- Whether `.5`'s guard can reach R2 (an exhaustive `match` over a generated field enum) or
  must be R3 (a round-trip test: set every `Overrides` field to a non-default `Some`, apply,
  assert every corresponding `Config` field moved).
- Whether `.6` should derive the four JSON-schema enums from `adapters()` at build time
  (eliminating the literals) or guard them with a test that parses the emitted schema.
- Whether `ENUMERATION-PARITY`'s declared-pairs table lives in the check script or in a
  small tracked data file that the check reads.

## Links

- Tree: [`docs/tasks/SHADOW-ENUMERATION-SWEEP.md`](../tasks/SHADOW-ENUMERATION-SWEEP.md)
  (`.1` registered the class and the first audit; this leaf is `.2`).
- Owner doctrine: **A DEFECT IS ONLY HANDLED IF A TASK-TREE OWNS IT** and **DECIDE, DON'T
  ASK** (`MEMORY.md` standing directives, `2026-07-30`).
- Standards: `DOCTRINE_ENFORCEMENT.md` §3 (the three check archetypes the repair ladder
  mirrors), §4 (the check contract `.7` must obey), §5 (the registry is authoritative), §9
  (honest limits — the basis for the (c) verdict), §11 (a doctrine with no check is a
  suggestion).
- Precedents this decision generalizes: decision `0032` (`EMIT-SURFACE-INTERACTION-GATE.2`'s
  preset ↔ `knob_catalog()` drift test — the R3 model), `EVIDENCE-BANK-DURABILITY.6` (the
  deriver that classifies from the report's own keys instead of remembering a list),
  `EMIT-SURFACE-INTERACTION-GATE.4` (the `compute_use_counts` consumer census).
- Doctrines that constrain the repairs: `feedback_full_factorization` (one mechanism, never
  two — why the Rust sites get no shell doctrine), `feedback_never_retire_strategies`
  (nothing is deleted by a repair), decision `0030` (why an S3 false green is uniquely
  costly: it is banked and cited), decision `0031` (why the boot-volume allow-list is
  authoritative), decision `0017` (why `.6`'s API-contract divergence matters).
- Touch points for `.3`–`.7`: `src/bin/tool_matrix.rs` (the gate-flag sites,
  `merge_coverage`), `src/config.rs` (`apply_cli_overrides`), `src/mcp/mod.rs` (the adapter
  schema enums), `scripts/check_doctrines.sh` + `DOCTRINE_ENFORCEMENT.md` §10,
  `book/src/SUMMARY.md`.
