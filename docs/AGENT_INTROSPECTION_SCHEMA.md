# ANVIL Agent-Introspection Schema (`AGENT-INTROSPECTION-MCP.2`)

- Owning task-tree leaf: `AGENT-INTROSPECTION-MCP.2`
- Status: **design / docs-only** (no code; this leaf defines the contract
  that the `.3` emitter and the `.4` MCP server stand on)
- Architecture record: [`docs/decisions/0004-agent-introspection-mcp-lane.md`](decisions/0004-agent-introspection-mcp-lane.md)
- Created: `2026-06-14`

> One-line thesis: **the introspection schema is a thin, versioned
> *envelope* around facts ANVIL already records — every payload field is the
> serde projection of an existing `metrics` / `manifest` / `config` /
> coverage struct, so the schema adds a stable shape and a version number,
> never a second source of truth that could drift.**

This document is the contract. It does **not** add any computed value, and it
deliberately does **not** re-list the fields of the existing structs (that
would fork the single source of truth — see §2 and the doctrine in
`docs/decisions/0004`). It pins the *envelope*, maps every embedded section to
its exact source, and states the versioning policy.

---

## 1. Scope and relationship to the lane

The `AGENT-INTROSPECTION-MCP` lane (decision `0004`) exposes ANVIL to an AI
agent as a thin, read-mostly adapter **beside** the deterministic generator
core. Its highest-value workflow is the bug-hunting loop: `generate →
validate → minimize → emit reproducer`. For an agent to drive that loop it
needs a **stable, versioned, machine-shaped** view of *construction truth*.

- `.2` (this doc) specifies that view: the introspection JSON **schema**.
- `.3` implements the *emission surface* (`anvil introspect` / a structured
  JSON dump) that produces documents conforming to this schema, additively
  and DUT-byte-identically.
- `.4` exposes the same documents as MCP **resources**; `.5` adds the
  controlled `validate` / `minimize` tools; `.6` packages the prompts; `.7`
  documents the lane in the book + USER_GUIDE and closes the tree.

Nothing here changes the default `anvil` build or the `--artifact dut`
byte-identical contract. The schema is *descriptive of existing facts*, so a
conforming emitter is a pure re-projection, not a new generation path.

---

## 2. Design principle — derived, not re-implemented (zero new computed truth)

This is the load-bearing invariant, inherited verbatim from `0004` ("the
schema is *derived* from the existing `metrics`/`manifest`/`config`, never a
parallel re-implementation that can drift") and from the Knowledge Map's
anti-archaeology principle.

**Invariant SCHEMA-DERIVED.** Every field in an introspection document is one
of exactly two kinds:

1. **Envelope field** — defined *by this document* (§4). These are new, but
   they are pure metadata (version strings, the request echo, a content
   address, resource pointers). They carry no analysis ANVIL did not already
   perform. This document is their single source of truth.
2. **Payload field** — a field re-projected, unchanged, from an existing Rust
   struct via `serde` (§6). The Rust struct is its single source of truth;
   this document references the struct, names its provenance, and **does not
   copy its field list**. A conforming emitter MUST obtain payload sections by
   `serde`-serializing the live struct value, never by re-deriving fields.

**Consequence.** "Lists every field + provenance" (the `.2` acceptance
criterion) is satisfied at the doctrinally-correct granularity: this document
enumerates **every envelope field explicitly** (it owns them) and maps **every
embedded payload section to the exact struct / file / producer that owns its
fields** (§6), plus enumerates each large struct's **category groups** so a
reviewer sees full coverage without the schema becoming a drifting field
mirror. The leaf-level field list lives in code, where it cannot fall out of
sync, and is enumerated by `serde` at emit time.

**No new computed truth.** The adapter computes nothing. `metrics::compute`,
`metrics::compute_design`, the manifest builders, `--dump-config`, and the
`tool_matrix` coverage roll-up already exist and already run; the emitter only
chooses which of their outputs to place under which envelope key.

---

## 3. Determinism and identity

ANVIL artifacts are pure functions of `(seed, knobs, lane, version)`
(`README.md` principle 3; `0004` "determinism collapses the service session
into a content-addressed cache"). The schema makes that tuple explicit so a
document is self-identifying and reproducible:

- An introspection document is itself a deterministic function of
  `(schema_version, anvil_version, lane, seed, canonicalized knobs)`. Two runs
  of a conforming emitter with the same inputs MUST produce byte-identical
  documents (modulo any field the schema explicitly marks volatile — there are
  none in v1.0; wall-clock, host names, and absolute paths are forbidden in
  the envelope, mirroring the determinism rules ANVIL already enforces).
- `request.run_id` is a **content address**: a hash over the canonical
  encoding of `(schema_version, anvil_version, lane, seed, knobs)`. It is *not*
  a random nonce. Identical inputs ⇒ identical `run_id` ⇒ the cache hit `0004`
  relies on. (The exact hash function is an implementation detail fixed by
  `.4`; the schema only requires that it is a pure function of those inputs and
  is recorded as a hex string.)

---

## 4. The introspection envelope (v1.4)

The top-level object. **Every field below is owned by this document.** Types
use TypeScript-ish notation for brevity; the wire format is JSON.

```
IntrospectionDocument {
  schema_version: string        // e.g. "1.0"; semver, see §7. REQUIRED.
  anvil_version:  string        // crate version, env!("CARGO_PKG_VERSION")
                                //   = "0.1.0" today. REQUIRED.
  lane:           "dut" | "microdesign" | "frontend"   // REQUIRED.
  request: {                    // the determinism tuple, echoed. REQUIRED.
    seed:   integer             // u64; the generation seed.
    lane:   same as top-level `lane`.
    knobs:  object              // see §6.1 — the effective Config (dut) OR
                                //   the lane param echo (microdesign/frontend).
    run_id: string              // content address (hex), see §3. REQUIRED.
  }
  artifact: {                   // descriptor of the produced artifact. REQUIRED.
    kind:   "module" | "design" | "microdesign" | "frontend"
    top:    string | null       // top/module name; null for a bare leaf dump.
    sv:     ResourceRef         // pointer to the emitted SystemVerilog (§6.6);
                                //   NOT inlined by default (bulk → resource).
    sv_sha256: string | null    // optional content hash of the .sv resource.
    manifest: ResourceRef | null// pointer to the lane manifest resource, if any.
  }
  introspection: IntrospectionPayload   // the structured facts. REQUIRED.
  warnings: string[]            // non-fatal notes (e.g. "coverage section
                                //   absent: single-artifact generate, not a
                                //   matrix run"). REQUIRED (may be empty).
}

ResourceRef {                   // a deliberate, fetch-on-demand pointer
  uri:   string                 // e.g. "anvil://artifact/<run_id>/top.sv"
                                //   (MCP) or a filesystem path (CLI dump).
  bytes: integer | null         // size hint, if known.
}
```

`IntrospectionPayload` is the union of the sections in §6; which sections are
present depends on `lane` and on whether the producing call was a
single-artifact generate or a `tool_matrix` run (see §5).

---

## 5. Section presence by lane / call

| Section (envelope key)      | `dut` module | `dut` design | `microdesign` | `frontend` | Needs matrix run |
| --- | --- | --- | --- | --- | --- |
| `config`                    | ✅ | ✅ | echo¹ | echo¹ | — |
| `module_metrics`            | ✅ | per child | — | — | — |
| `design_metrics`            | — | ✅ | — | — | — |
| `microdesign_manifest`      | — | — | ✅ | — | — |
| `frontend_manifest`         | — | — | — | ✅ | — |
| `coverage`                  | optional² | optional² | optional² | optional² | ✅ |
| `artifact.sv` (resource)    | ✅ | ✅ | ✅ | ✅ | — |

¹ The non-DUT lanes are parameterized by their lane params
(`--lane-n-params`, `--lane-n-children`), not the full DUT `Config`; their
`request.knobs` echoes those lane params (§6.1). ² `coverage` is only
meaningful for a `tool_matrix` sweep (it aggregates the `saw_*` facts and
`coverage_gaps` across a scenario corpus); a single-artifact `generate` omits
it and records a `warnings[]` note.

---

## 6. Section → source provenance map

For each embedded section: its JSON type, the Rust struct that owns its
fields, the source file, the producing function, and the `serde` guarantee.
Per §2, the struct is the single source of truth for its field list.

### 6.1 `config` — the effective knobs

| | |
| --- | --- |
| **JSON** | object (the serde map of every knob) |
| **Source struct** | `Config` |
| **File** | `src/config.rs` |
| **Producer** | `--dump-config` today (`serde_json::to_value(&cfg)`); the DUT manifest's `config` scalar (`src/main.rs`) |
| **Serde guarantee** | exact serde projection of `Config`; new knobs carry `#[serde(default)]`, which is what keeps the schema additive (§7) |

`Config` knob **category groups** (full field list owned by `src/config.rs`):
`seed`; structural bounds (`min/max_inputs`, `min/max_outputs`,
`min/max_width`, `max_depth`, `max_nodes_per_module`); process-safety governor
(`max_rss_mb`, `ram_abort_pct`); probability knobs (`flop_prob`, `share_prob`,
`terminal_reuse_prob`, `constant_prob`, `library_prob`); gate-mix weights
(`gate_{bitwise,arith,struct,compare,reduce}_weight`); operator arity
(`min/max_gate_arity`); coefficient motif (`coefficient_prob`,
`min/max_coefficient`); shift-amount motif (`const_shift_amount_prob`,
`min/max_shift_amount`, `gate_shift_weight`); comparand motif
(`const_comparand_prob`, `min/max_comparand`); structured-block motifs
(`priority_encoder_prob`, `case_mux_prob`, `casez_mux_prob`, `for_fold_prob`);
sequential bounds (`max_flops_per_module`, `min/max_mux_arms`,
`flop_qfeedback_prob`, `flop_mux_encoding_prob`, `comb_mux_prob`,
`comb_mux_encoding_prob`); hierarchy (the `hierarchy_*`, `num_*`,
`*_child_instances*`, `*_parent_cone_instance*`, `*_parent_flop*` family);
module dedup (`hierarchy_module_dedup`, `hierarchy_semantic_module_dedup`);
Phase 5 (`width_parameterization_prob`); Phase 5b (`aggregate_prob`,
`aggregate_array_prob`); Phase 6 (`memory_prob`, `fsm_prob`); multi-clock
(`multi_clock_prob`, `cdc_synchronizer_stages`); clocking (`use_async_reset`);
construction (`construction_strategy`, `graph_first_pool_size`); identity /
factorization (`identity_mode`, `factorization_level`,
`operand_duplication_rate`, `mux_arm_duplication_rate`, `max_ast_instances`).
Enum value sets are owned by `ConstructionStrategy`, `HierarchyChildSourceMode`,
`IdentityMode`, `FactorizationLevel` in the same file.

### 6.2 `module_metrics` — per-module structural facts

| | |
| --- | --- |
| **JSON** | object |
| **Source struct** | `Metrics` |
| **File** | `src/metrics.rs` |
| **Producer** | `metrics::compute(&Module)`; the DUT manifest per-module element's `metrics` (`src/main.rs`) |
| **Serde guarantee** | exact serde projection of `Metrics`; emitted-empty maps omitted as today |

`Metrics` **category groups** (fields owned by `src/metrics.rs`): module id;
size (incl. `num_clock_domains`, `num_cdc_2_flop_synchronizers`,
`num_cdc_synchronizer_chains`, `max_cdc_synchronizer_stages`); per-gate-kind
distribution (`gates_by_kind`); constants distribution; mux shape; concat
shape; shift shape; sharing / fanout; flops; AST-instance saturation;
operand-arity distribution; combinational depth; factorization-ladder
telemetry (`fold_identities_applied`, `peephole_rewrites_applied`,
`flatten_associative_applied`, `nodes_compacted`, `flops_merged`,
`bisimulation_flops_merged`, `fsms_merged`, `semantic_gates_merged`,
`nested_associative_operand_count`);
per-knob probability-roll counters (`knob_roll_attempts`, `knob_roll_fires`);
block-build counters.

### 6.3 `design_metrics` — per-design composition facts

| | |
| --- | --- |
| **JSON** | object |
| **Source struct** | `DesignMetrics` |
| **File** | `src/metrics.rs` |
| **Producer** | `metrics::compute_design(&Design)`; the DUT manifest per-design element's `metrics` (`src/main.rs`) |
| **Serde guarantee** | exact serde projection of `DesignMetrics` |

`DesignMetrics` **category groups** (fields owned by `src/metrics.rs`): design
id; hierarchy-aware identity instrumentation (`canonical_module_signatures`,
`semantic_module_signatures`, the distinct/duplicate counts, the Phase-5/5b/6
module counts); overall size; composition ratios; hierarchy shape (incl. the
`*_by_depth` maps); top interface (the large `top_*` child-input-binding /
parent-cone-instance / parent-composed family); composition across the whole
hierarchy (the `hierarchy_*` family); child-interface load (the
`child_input_bindings_from_*` family + fractions); sequential / combinational
mix; weighted child complexity; reuse histogram
(`instantiated_module_histogram`).

### 6.4 `coverage` — adversarial-matrix coverage facts

| | |
| --- | --- |
| **JSON** | object |
| **Source struct** | `CoverageSummary` (incl. `coverage_gaps: string[]` and the `saw_*` boolean facts) |
| **File** | `src/bin/tool_matrix.rs` |
| **Producer** | a `tool_matrix` run (the harness aggregates per-scenario reports) |
| **Serde guarantee** | exact serde projection of `CoverageSummary` |

Provenance note: coverage facts are a property of a **matrix sweep**, not of a
single artifact (a lone module cannot prove `saw_recursive_hierarchy_*`). The
schema therefore exposes `coverage` only when the producing call ran the
matrix; otherwise it is absent with a `warnings[]` note. The agent's
`close_coverage_gap` / `triage_tool_failures` prompts (`.6`) consume this
section together with `coverage_gaps`.

### 6.5 `microdesign_manifest` / `frontend_manifest` — lane expected-facts

These are small and stable; every field is listed (they are the lane oracles'
expected-facts manifests, already byte-stable and parity-gated).

`microdesign_manifest` ← `microdesign::Manifest` (`src/microdesign/mod.rs`,
`build_manifest` / `emit_manifest`): `seed`, `top`, `params`, `localparams`,
`widths`, `generate`, `package_constants`, `const_exprs`.

`frontend_manifest` ← `frontend::Manifest` (`src/frontend/mod.rs`,
`build_manifest` / `emit_manifest`): `seed`, `top`, `packages`, `top_params`,
`top_localparams`, `instances`, `generate_branches`.

Serde guarantee: exact serde projection of the named struct; these are the
same JSON the lanes already write to `<top>.json` / stderr, so introspection
adds zero new truth for the non-DUT lanes.

### 6.6 `artifact.sv` — the emitted SystemVerilog (resource)

| | |
| --- | --- |
| **JSON** | `ResourceRef` (pointer, not inlined) |
| **Source** | `emit::to_sv(&Module)` / `emit::to_sv_in_design(&Module, &Design)`; the lane `LaneArtifact.sv` for non-DUT lanes (`src/umbrella/mod.rs`) |
| **Rationale** | `0004` "structured queries, not bulk dumps": the agent fetches the full `.sv` deliberately as a resource, not embedded in every introspection reply |

### 6.7 `analysis` — derived-relation queries (the `analyze` surface)

`SEMANTIC-INTROSPECTION-EXPANSION` (decision `0011`, schema `1.3`) adds a
**derived-RELATION** query surface: *what does this output structurally depend
on?* It is **not** part of the default `IntrospectionDocument` (the default
`--introspect` payload stays lean — decision `0011` Q2). Instead it is a
sibling document, the **`DerivedAnalysisDocument`**, returned by the pure MCP
`analyze` tool and served as the `anvil://artifact/<run_id>/analysis/<query>`
resource.

| | |
| --- | --- |
| **JSON** | the introspection **envelope** (`schema_version` / `anvil_version` / `lane` / `request` / `artifact` / `warnings`, §4) with `introspection` replaced by an `analysis` payload |
| **Source struct** | `DerivedAnalysisDocument { …envelope…, analysis: DerivedAnalysis }` |
| **File** | `src/introspect/mod.rs` (envelope) + `src/introspect/analyze.rs` (`DerivedAnalysis` / `SupportCone` / `ReachResult` / `FlopProvenance`) |
| **Producer** | `output_support`: `module_support_cones` / `design_support_cones`; `input_reach`: `module_input_reach` / `design_input_reach`; `flop_reset_provenance`: `module_flop_provenance` / `design_flop_provenance` — all pure (`introspect::analyze::*`) over the already-emitted `Module` / `Design`; wrapped by `introspect::derived_analysis_document` |
| **Serde guarantee** | exact serde projection of `DerivedAnalysis`; `BTreeSet` → sorted `Vec` ⇒ byte-stable |

**Invariant SCHEMA-DERIVED holds.** `DerivedAnalysis` is a pure post-hoc
traversal of the IR graph the generator already produced (the same graph
`metrics::compute` walks) — **no new computed truth, no IR field, no generator
change**, exactly like the `coverage_gaps` projection. It reports **relations**
(structural dependency), never behaviour: the `0004` no-shadow-simulator /
structure-first boundary is the permanent ceiling.

`DerivedAnalysis` **category groups** (fields owned by `src/introspect/analyze.rs`):
the `query` kind (`output_support`, `input_reach`, `flop_reset_provenance`, and
`module_reachability` — all four named kinds from decision `0011` are now
delivered) + **one of four parallel result vecs**, the one the query kind populates
(the others are empty and, except for the always-present `results`, omitted via
`skip_serializing_if`):

- **`results: Vec<SupportCone>`** — the `output_support` payload. A `SupportCone`
  is the transitive **combinational** fan-in support of one target — an output
  port, or a flop `D` addressed `"flop:<id>"`: the primary-input port names
  (`support_inputs`), flop ids (`support_flops`, a register boundary — the cone
  feeding a flop's `D` is the separate `"flop:<id>"` target), and child-instance
  outputs (`support_instance_outputs`, the cone stops at the instance boundary),
  plus `cone_nodes` (distinct fan-in nodes) and `cone_depth` (max combinational
  gate depth). Opaque registered leaves (`MemRead` / `FsmOut`) terminate the cone
  (counted, listed nowhere — surfacing memory/FSM provenance is a reserved future
  kind).
- **`reach_results: Vec<ReachResult>`** (schema `1.5`, `SEMANTIC-INTROSPECTION-EXPANSION.3b.2`)
  — the `input_reach` payload, the **dual fan-out** of `output_support`. A
  `ReachResult` is what one **source** structurally reaches: `target` is the
  source (an input port name, a flop `Q` addressed `"flop:<id>"`, or a
  child-instance output `"<instance>.<port>"`), `reaches_outputs` are the output
  port names whose support cone contains it, `reaches_flops` are the flop ids
  whose `D` cone contains it, and `fanout_targets` is their total. It is computed
  by **inverting** the support cones, so `X` reaches `Y` iff `Y`'s `SupportCone`
  lists `X` — the two queries cannot drift. `reach_results` carries
  `#[serde(default, skip_serializing_if = "Vec::is_empty")]`, so an
  `output_support` document never serializes the key and stays byte-identical
  across the `1.4 → 1.5` bump; an `input_reach` document carries it with
  `results: []`.
- **`flop_provenance: Vec<FlopProvenance>`** (schema `1.6`, `SEMANTIC-INTROSPECTION-EXPANSION.4b.2`)
  — the `flop_reset_provenance` payload: per-flop **reset/data provenance** (*is
  this register reset-defined or data-driven, and how is its next state built?*).
  A `FlopProvenance` is a direct projection of one `Flop`: `flop` (id, addressed
  `"flop:<id>"`), `width`, `has_reset`, `reset_kind` (`"none"`/`"sync"`/`"async"`),
  `reset_value` (the `u128` reset value as a **decimal string** — exact on any JSON
  consumer), `default_behavior` (`"zero"` for `ZeroDefault`, `"hold"` for
  `QFeedback` — what `D` becomes when no mux select is asserted), `mux_kind`
  (`"none"`/`"one_hot"`/`"encoded"`), `mux_arms` (arm/data-slot count), and
  `has_d`. It is a direct read of `Module.flops` (no graph walk). `flop_provenance`
  carries the same `skip_serializing_if`, so `output_support`/`input_reach`
  documents stay byte-identical across the `1.5 → 1.6` bump; a
  `flop_reset_provenance` document carries it with `results: []`.
- **`module_reachability: Vec<ModuleReachability>`** (schema `1.7`, `SEMANTIC-INTROSPECTION-EXPANSION.5b.2`)
  — the `module_reachability` payload: which modules in a design are reachable from
  `design.top` via the instance graph. A `ModuleReachability` is a projection of the
  design's module table + instance edges: `module` (the module name), `reachable`
  (from the top over the `Module.instances[].module` edges), `depth` (the minimum
  instance-graph distance from the top — `0` for the top; present iff `reachable`,
  omitted otherwise), `instantiates` (the distinct child module names it directly
  instantiates, sorted), and `instance_count` (its direct-instance count, `>=
  instantiates.len()`). Computed by a min-depth BFS from `design.top` — a pure
  projection of `Design.modules` + the instance edges, no gate-graph walk; one entry
  per module, sorted by module name. `module_reachability` carries the same
  `skip_serializing_if`, so the prior three documents stay byte-identical across the
  `1.6 → 1.7` bump; a `module_reachability` document carries it with `results: []`.
  Unlike the prior three queries, `target` is a **module name** (not a port name or
  `"flop:<id>"`), the natural identifier for a module-level query.

`target = None` ⇒ all targets/sources/flops/modules (per the agent-audience
completeness rule); an unknown `query` or `target` is rejected with JSON-RPC
`-32602`.

---

## 7. Versioning policy

The introspection contract is versioned with `schema_version`, a `MAJOR.MINOR`
string carried in every document (§4). The policy is anchored to the `serde`
behaviour the source structs already use.

- **MINOR bump (backward-compatible).** Adding a new envelope field, a new
  embedded section, or surfacing struct fields that are added with
  `#[serde(default)]`. Existing consumers keep working: unknown keys are
  ignored by tolerant readers, and absent new keys fall back to defaults. The
  many `#[serde(default)]` annotations already on `Config` / `Metrics` /
  `DesignMetrics` fields are exactly what makes additive growth a MINOR change.
- **MAJOR bump (breaking).** Removing or renaming an envelope field; changing
  a field's type or units; changing the meaning of an existing field; removing
  a section; or any change that an existing consumer pinned to the prior
  `schema_version` could misread. A struct field rename in `metrics`/`config`
  that reaches the wire surface is a MAJOR change and travels with a
  `schema_version` MAJOR bump.
- **Lockstep with `anvil_version`.** `anvil_version` (crate version) is always
  present so an agent can distinguish "same schema, newer generator" (facts may
  differ in value) from "newer schema" (shape may differ). Today both are
  early: `schema_version = "1.11"`, `anvil_version = "0.1.0"`.
- **Negotiation.** The `.4` MCP server / `.3` CLI surface advertise the
  `schema_version`(s) they emit. A consumer pins or range-matches on
  `schema_version`; an emitter asked for an unsupported version MUST refuse
  explicitly (a typed error), never silently emit a different shape.
- **Determinism preserved across versions.** A version bump never introduces
  wall-clock, randomness, or host-specific data into the envelope; documents
  stay pure functions of `(schema_version, anvil_version, lane, seed, knobs)`
  (§3).

This document defines **`schema_version = "1.11"`**.

- **`1.0` → `1.1` (`IDENTITY-DEEPENING.2b`).** Additive MINOR bump:
  surfaced the new `Metrics::bisimulation_flops_merged` field (the opt-in
  bounded bisimulation flop-merge count) in `module_metrics`. Backward
  compatible — a `1.0` consumer simply ignores the new key. No envelope
  field was removed, renamed, or retyped; determinism is preserved.
- **`1.1` → `1.2` (`SV-VERSION-TARGETING.2b.1`).** Additive MINOR bump:
  surfaced the new `Config::sv_version` field (the opt-in `--sv-version`
  emission-target capability, an `SvVersion` enum serialized as the bare
  year `"2012"`/`"2017"`/`"2023"`, `#[serde(default)]` = `"2012"`) in
  `request.knobs`. Backward compatible — a `1.1` consumer ignores the new
  key, and an absent key reads back as the `"2012"` floor. No envelope
  field was removed, renamed, or retyped; the default-`dut` artifact stays
  byte-identical, so determinism is preserved.
- **`1.2` → `1.3` (`SEMANTIC-INTROSPECTION-EXPANSION.2b`).** Additive MINOR
  bump: added the derived-relation **analysis** surface (§6.7) — the pure MCP
  `analyze` tool + the sibling `DerivedAnalysisDocument` (envelope reuse + an
  `analysis: DerivedAnalysis` payload). The **default `IntrospectionDocument`
  shape is unchanged** — only its `schema_version` string advances — so a `1.2`
  consumer of the default `--introspect` document keeps working; the new
  document is reached only via the opt-in `analyze` tool. No envelope field was
  removed, renamed, or retyped; `analysis` is SCHEMA-DERIVED (a pure IR-graph
  projection, §6.7) so it adds no new computed truth; the default-`dut`
  artifact stays byte-identical and determinism is preserved.
- **`1.3` → `1.4` (`IDENTITY-DEEPENING.3b.2b.2a`).** Additive MINOR bump:
  surfaced the new `DesignMetrics::sequential_module_proof_signatures`
  (`Vec<Option<u64>>`, one sequential proof-class id per module) and
  `DesignMetrics::num_sequentially_duplicate_module_pairs` fields (the
  whole-leaf-module sequential-equivalence projection) in `design_metrics`. Both
  are `#[serde(default)]`, so a `1.3` consumer ignores them and an absent key
  reads back as empty / `0`. RTL-invisible (a post-hoc `DesignMetrics`
  projection — exactly the additive-growth case §7 names); the default-`dut`
  artifact stays byte-identical, so determinism is preserved.
- **`1.4` → `1.5` (`SEMANTIC-INTROSPECTION-EXPANSION.3b.2`).** Additive MINOR
  bump: added the **second** derived-query kind `input_reach` (§6.7) — the dual
  fan-out of `output_support`. `DerivedAnalysis` gains a second
  `reach_results: Vec<ReachResult>` field, `#[serde(default,
  skip_serializing_if = "Vec::is_empty")]`, so an `output_support` analysis
  document is **byte-identical to `1.4`** (the key is omitted) and only an
  `input_reach` document carries it (with `results: []`). A `1.4` consumer of an
  `output_support` document keeps working unchanged; the new kind is reached only
  via `analyze {query: "input_reach"}`. No envelope field was removed, renamed,
  or retyped; `reach_results` is SCHEMA-DERIVED (a pure inversion of the support
  cones, §6.7) so it adds no new computed truth; the default-`dut` artifact stays
  byte-identical and determinism is preserved.
- **`1.5` → `1.6` (`SEMANTIC-INTROSPECTION-EXPANSION.4b.2`).** Additive MINOR
  bump: added the **third** derived-query kind `flop_reset_provenance` (§6.7) —
  per-flop reset/data provenance. `DerivedAnalysis` gains a third
  `flop_provenance: Vec<FlopProvenance>` field, `#[serde(default,
  skip_serializing_if = "Vec::is_empty")]`, so `output_support` / `input_reach`
  documents are **byte-identical to `1.5`** (the key is omitted) and only a
  `flop_reset_provenance` document carries it (with `results: []`). A `1.5`
  consumer of the prior documents keeps working unchanged; the new kind is reached
  only via `analyze {query: "flop_reset_provenance"}`. No envelope field was
  removed, renamed, or retyped; `flop_provenance` is SCHEMA-DERIVED (a direct
  projection of `Module.flops`, §6.7) so it adds no new computed truth; the
  default-`dut` artifact stays byte-identical and determinism is preserved.
- **`1.6` → `1.7` (`SEMANTIC-INTROSPECTION-EXPANSION.5b.2`).** Additive MINOR
  bump: added the **fourth** derived-query kind `module_reachability` (§6.7) —
  which modules in a design are reachable from `design.top` via the instance graph.
  `DerivedAnalysis` gains a fourth `module_reachability: Vec<ModuleReachability>`
  field, `#[serde(default, skip_serializing_if = "Vec::is_empty")]`, so
  `output_support` / `input_reach` / `flop_reset_provenance` documents are
  **byte-identical to `1.6`** (the key is omitted) and only a `module_reachability`
  document carries it (with `results: []`). A `1.6` consumer of the prior documents
  keeps working unchanged; the new kind is reached only via `analyze {query:
  "module_reachability"}`. No envelope field was removed, renamed, or retyped;
  `module_reachability` is SCHEMA-DERIVED (a pure BFS projection of `Design.modules`
  + the instance edges, §6.7) so it adds no new computed truth; the default-`dut`
  artifact stays byte-identical and determinism is preserved. This is the **fourth
  and last named query kind** from decision `0011`.
- **`1.7` → `1.8` (`STRUCTURED-EMISSION-EXPANSION.2b.2a`).** Additive MINOR bump:
  surfaced the new `Metrics::num_emitted_combinational_functions` field (the count
  of gates a module emits as a combinational `function automatic` projection —
  `Module.function_emit_gates.len()`, the opt-in `function_emit_prob` knob,
  decision `0012`) in `module_metrics`. `#[serde(default)]`, so a `1.7` consumer
  ignores the new key and an absent key reads back as `0`. RTL-invisible (a
  post-hoc structural count of an emitter-surface annotation — exactly the
  additive-growth case §7 names, like the `1.0 → 1.1` `bisimulation_flops_merged`
  Metrics-field bump); the default-`dut` artifact stays byte-identical, so
  determinism is preserved. (The companion `function_emit_prob` *knob* was added
  to `request.knobs` at `.2b.1` under the existing version via `#[serde(default)]`,
  per the default-off probability-knob precedent — `soft_union_slice_prob` /
  `aggregate_prob` / `memory_prob` / `fsm_prob` / `multi_clock_prob`; this bump is
  for the new derived **metric**, not the knob.)
- **`1.8` → `1.9` (`STRUCTURED-EMISSION-EXPANSION.4b.2a`).** Additive MINOR bump:
  surfaced the new `Metrics::num_emitted_generate_loops` field (the count of
  `{N{x}}` replication gates a module emits as a single-level `generate for` loop
  projection — `Module.generate_loop_gates.len()`, the opt-in
  `generate_loop_emit_prob` knob, decision `0013`) in `module_metrics`.
  `#[serde(default)]`, so a `1.8` consumer ignores the new key and an absent key
  reads back as `0`. RTL-invisible (a post-hoc structural count of an
  emitter-surface annotation — exactly the additive-growth case §7 names, like the
  `1.7 → 1.8` `num_emitted_combinational_functions` Metrics-field bump); the
  default-`dut` artifact stays byte-identical, so determinism is preserved. (The
  companion `generate_loop_emit_prob` *knob* was added to `request.knobs` at
  `.4b.1` under the existing version via `#[serde(default)]`, per the default-off
  probability-knob precedent — `function_emit_prob` / `soft_union_slice_prob` /
  `aggregate_prob` / `memory_prob` / `fsm_prob` / `multi_clock_prob`; this bump is
  for the new derived **metric**, not the knob.)
- **`1.9` → `1.10` (`STRUCTURED-EMISSION-EXPANSION.6b.2a`).** Additive MINOR bump:
  surfaced the new `Metrics::num_emitted_combinational_tasks` field (the count of
  combinational gates a module emits as a `task automatic` projection —
  `Module.task_emit_gates.len()`, the opt-in `task_emit_prob` knob, decision
  `0014`) in `module_metrics`. `#[serde(default)]`, so a `1.9` consumer ignores the
  new key and an absent key reads back as `0`. RTL-invisible (a post-hoc structural
  count of an emitter-surface annotation — exactly the additive-growth case §7
  names, like the `1.8 → 1.9` `num_emitted_generate_loops` Metrics-field bump); the
  default-`dut` artifact stays byte-identical, so determinism is preserved. (The
  companion `task_emit_prob` *knob* was added to `request.knobs` at `.6b.1` under
  the existing version via `#[serde(default)]`, per the default-off
  probability-knob precedent; this bump is for the new derived **metric**, not the
  knob.) MINOR is an integer, so this is `1.9 → 1.10` (ten), not a decimal.
- **`1.10` → `1.11` (`STRUCTURED-EMISSION-EXPANSION.10b.2`).** Additive MINOR bump:
  surfaced the new `Metrics::num_emitted_cone_functions` field (the count of
  combinational *cones* a module emits as a multi-gate `function automatic`
  projection — `Module.cone_function_gates.len()`, the opt-in
  `cone_function_emit_prob` knob, decision `0016`) in `module_metrics`.
  `#[serde(default)]`, so a `1.10` consumer ignores the new key and an absent key
  reads back as `0`. RTL-invisible (a post-hoc structural count of an
  emitter-surface annotation — exactly the additive-growth case §7 names, like the
  `1.9 → 1.10` `num_emitted_combinational_tasks` Metrics-field bump); the
  default-`dut` artifact stays byte-identical, so determinism is preserved. This
  metric is **separate** from `num_emitted_combinational_functions` (the
  single-gate `function_emit_prob` surface); the cone surface has its own knob, so
  the shipped single-gate surface is untouched. (The companion
  `cone_function_emit_prob` *knob* was added to `request.knobs` at `.10b.1` under
  the existing version via `#[serde(default)]`, per the default-off
  probability-knob precedent; this bump is for the new derived **metric**, not the
  knob.) MINOR is an integer, so this is `1.10 → 1.11` (eleven), not a decimal.

---

## 8. Non-goals (restating the lane guardrails for this surface)

- **No new computed truth** (§2). The schema re-projects existing facts only.
- **No inferred whole-module "intended behavior."** ANVIL has none by
  doctrine; the schema exposes structure / provenance / coverage /
  resolved-facts, never claimed functional intent (`0004` "honest scope").
- **No stateful session fields** (`run_until`, signal-over-time, waveform
  handles): ANVIL is a pure `(seed, knobs) → artifact` function (`0004`
  non-goals).
- **No bulk `.sv`/manifest inlined by default**: those are resources (§6.6).
- **No effect on the default build**: a conforming emitter is additive and
  keeps `--artifact dut` byte-identical (verified at `.3` via snapshots).

---

## 9. Deferred to implementation leaves

These are intentionally out of scope for `.2` (they are transport / surface
shape, not the data contract) and are tracked in the
[`AGENT-INTROSPECTION-MCP`](tasks/AGENT-INTROSPECTION-MCP.md) Open Questions:

- The exact CLI shape of the `.3` emission surface (`anvil introspect …`) and
  whether it shares the `--artifact` selector.
- The MCP transport (stdio first) and crate layout (separate `anvil-mcp`
  target) — `.4`.
- The `run_id` hash function and the content-addressed cache layout — `.4`.
- Whether `validate` ships in the first MCP cut or stays CLI-only — `.5`.

## 10. Acceptance self-check (`.2`)

- ✅ Stable, versioned schema specified, **derived strictly** from existing
  `metrics` / `manifest` / `config` / coverage (§2, §6).
- ✅ Every envelope field listed with its type (§4); every embedded section
  mapped to its source struct / file / producer / serde guarantee (§6).
- ✅ Confirms **zero new computed truth** (invariant SCHEMA-DERIVED, §2).
- ✅ Versioning policy stated (§7), with `schema_version = "1.11"`.
- ✅ Docs-only; no code; DUT byte-identical contract untouched.
