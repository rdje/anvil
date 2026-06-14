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

## 4. The introspection envelope (v1.0)

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
`fsms_merged`, `semantic_gates_merged`, `nested_associative_operand_count`);
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
  early: `schema_version = "1.0"`, `anvil_version = "0.1.0"`.
- **Negotiation.** The `.4` MCP server / `.3` CLI surface advertise the
  `schema_version`(s) they emit. A consumer pins or range-matches on
  `schema_version`; an emitter asked for an unsupported version MUST refuse
  explicitly (a typed error), never silently emit a different shape.
- **Determinism preserved across versions.** A version bump never introduces
  wall-clock, randomness, or host-specific data into the envelope; documents
  stay pure functions of `(schema_version, anvil_version, lane, seed, knobs)`
  (§3).

This document defines **`schema_version = "1.0"`**.

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
- ✅ Versioning policy stated (§7), with `schema_version = "1.0"`.
- ✅ Docs-only; no code; DUT byte-identical contract untouched.
