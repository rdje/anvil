# API Reference: Introspection & Analysis Schemas

This page documents the **document shapes** the [`introspect`](api-tools.md#introspect)
and [`analyze`](api-tools.md#analyze) tools return (and that
`anvil --introspect` prints): the introspection envelope, the four `analyze`
query result schemas, and the `schema_version` stability contract. The exact,
field-by-field wire contract is
[`docs/AGENT_INTROSPECTION_SCHEMA.md`](https://github.com/rdje/anvil/blob/main/docs/AGENT_INTROSPECTION_SCHEMA.md);
this page is the navigable summary.

All of it is **`SCHEMA-DERIVED`**: every field is a projection of something ANVIL
already computed by construction (`Config`, `Metrics` / `DesignMetrics`, the IR
graph). No new truth, no behavioural oracle.

## `anvil --introspect` (the CLI one-shot)

Add `--introspect` to a **single-artifact** run (no `--out`, `--count 1`) and
ANVIL prints the introspection document to stdout instead of SystemVerilog:

<!-- book-test: skip — illustrative invocation; the runnable `cargo run --release -- --seed 42 --introspect` proof lives in the agent-mcp tutorial chapter -->
```bash
anvil --seed 42 --introspect
```

This is the exact document the `introspect` MCP tool returns — the CLI is a shim
over the same projection.

## The introspection document envelope

Every document is a thin, versioned envelope:

```json
{
  "schema_version": "1.13",
  "anvil_version": "0.1.0",
  "lane": "dut",
  "request": {
    "seed": 42,
    "lane": "dut",
    "knobs": { "…": "the full effective Config" },
    "run_id": "3f1cad578805bd04"
  },
  "artifact": {
    "kind": "module",
    "top": "mod_42_0000",
    "sv": { "uri": "anvil://artifact/3f1cad578805bd04/mod_42_0000.sv", "bytes": 80383 },
    "sv_sha256": null,
    "manifest": null
  },
  "introspection": {
    "module_metrics": { "avg_fanout": "…", "gates_by_kind": { "…": "…" } },
    "coverage_readout": { "knob_fire_rates": { "…": "…" }, "category_fire_rates": { "…": "…" },
                          "gate_kind_histogram": { "…": "…" } }
  },
  "warnings": [ "coverage section absent: single-artifact generate, not a tool_matrix run" ]
}
```

| Field | Meaning |
| --- | --- |
| `schema_version` | the document schema version (currently `1.13`) — see [stability](#schema_version-stability-contract) |
| `anvil_version` | the generating `anvil` crate version |
| `lane` | `dut` / `microdesign` / `frontend` |
| `request` | the echoed `seed` / `lane` / `knobs` plus the content-addressed `run_id` |
| `artifact` | the descriptor: `kind` (`module` / `design`), `top`, the `sv` resource pointer (`uri` + byte length — never inlined), optional `sv_sha256`, and the lane `manifest` pointer |
| `introspection` | the payload (below) |
| `warnings` | non-fatal notes (e.g. coverage is absent for a single artifact) |

**Payload by artifact kind:**

- a **module** carries `module_metrics` — exactly `metrics::compute(&module)`, the
  same metrics the manifest carries, re-projected under a stable key;
- a **design** carries `design_metrics` plus a per-child `modules` list;
- the **microdesign / frontend** lanes carry the expected-facts manifest
  (`microdesign_manifest` / `frontend_manifest`), also served as the
  `anvil://artifact/<run_id>/manifest` resource.

Since schema `1.12`, a DUT **module** or **design** also carries
`coverage_readout` — the achieved-coverage readout (below).

`coverage` (the matrix `saw_*` facts) is **absent** for a single artifact — a
lone module cannot prove a `saw_*` coverage fact; that coverage is a property of a
`tool_matrix` sweep, and the document says so in `warnings`. It is **distinct**
from the per-run `coverage_readout`, which every DUT artifact carries.

### `coverage_readout` — the achieved-coverage readout (schema `1.12`)

`coverage_readout` is the **read** half of coverage steering
(`COVERAGE-STEERED-GENERATION`): a SCHEMA-DERIVED projection of the per-knob roll
telemetry + construct histograms ANVIL already records. The same object is
returned standalone by the [`coverage`](api-tools.md#coverage) tool (wrapped in
the envelope as a `CoverageDocument`'s `coverage` payload).

```json
{
  "knob_fire_rates": {
    "flop_prob": { "attempts": 295, "fires": 36, "fire_rate": 0.122034 }
  },
  "category_fire_rates": {
    "state": { "attempts": 331, "fires": 53, "fire_rate": 0.160121 }
  },
  "gate_kind_histogram": { "and": 136, "mux": 158 },
  "gate_operand_count_histogram": { "2": 497, "3": 269 },
  "gate_depth_histogram": { "1": 21, "2": 26 }
}
```

| Field | Meaning |
| --- | --- |
| `knob_fire_rates` | per-knob (keyed by knob name) `{ attempts, fires, fire_rate }` — the empirical fire rate `fires / attempts` over the construction-time rolls. Only knobs rolled at least once appear. |
| `category_fire_rates` | the same cell pooled over each coarse category (`state` / `selectors` / `datapath` / `terminals` / `sharing` / `hierarchy`) — attempt-weighted. |
| `gate_kind_histogram` | count of emitted gates per `GateOp` kind. |
| `gate_operand_count_histogram` | histogram of gate operand counts (arity). |
| `gate_depth_histogram` | histogram of per-gate combinational depth. |

For a **design** the counts aggregate across all child modules. `attempts` /
`fires` are the exact integers; `fire_rate` is rounded to parts-per-million so the
document stays byte-stable.

## `analyze` result schemas

`analyze` returns a `DerivedAnalysisDocument` — the **same envelope** as
`introspect`, but with an `analysis` payload instead of `introspection`. The
payload's array key depends on the `query` kind (each is `skip_serializing_if`,
so a reply for one query is byte-identical to before the others were added — the
additive-MINOR discipline).

### `output_support` — an output's fan-in cone

The transitive **combinational fan-in support cone** of each target (what it
depends on). Payload key: `results`.

```json
{ "analysis": { "query": "output_support",
  "results": [ {
    "target": "o_0",
    "support_inputs": ["i_1"],
    "support_flops": [],
    "support_instance_outputs": [],
    "cone_nodes": 3,
    "cone_depth": 2 } ] } }
```

| Field | Meaning |
| --- | --- |
| `target` | the output port, or `"flop:<id>"` for a flop's D-cone |
| `support_inputs` / `support_flops` / `support_instance_outputs` | the support **leaves**: primary inputs, flop Qs (the cone stops at the register boundary), and child-instance outputs |
| `cone_nodes` | number of distinct fan-in nodes |
| `cone_depth` | combinational gate depth |

### `input_reach` — the dual fan-out

The **transpose** of `output_support`: what a source reaches. Payload key:
`reach_results`. Computed by inverting the support cones, so the two queries
cannot drift.

```json
{ "analysis": { "query": "input_reach",
  "reach_results": [ {
    "target": "i_1",
    "reaches_outputs": ["o_0"],
    "reaches_flops": [],
    "fanout_targets": 1 } ] } }
```

`"flop:<id>"` as a *source* is the flop's **Q** (its fan-out); as an
`output_support` *target* it is the flop's **D** cone — same register boundary,
opposite direction.

### `flop_reset_provenance` — per-flop reset/data provenance

A direct projection of each `Flop` (no graph walk). Payload key:
`flop_provenance`.

```json
{ "analysis": { "query": "flop_reset_provenance",
  "flop_provenance": [ {
    "flop": 0, "width": 8,
    "has_reset": true, "reset_kind": "async", "reset_value": "0",
    "default_behavior": "zero", "mux_kind": "one_hot", "mux_arms": 2,
    "has_d": true } ] } }
```

| Field | Values |
| --- | --- |
| `reset_kind` | `none` / `sync` / `async` |
| `reset_value` | the reset value as a **decimal string** (exact for 128-bit values) |
| `default_behavior` | `zero` (load 0 when no select asserted) / `hold` (keep `Q`) |
| `mux_kind` | `none` / `one_hot` / `encoded`, with `mux_arms` the arm count |
| `has_d` | whether a `D` cone is present |

A flopless module yields an empty result, not an error.

### `module_reachability` — the design module tree

A min-depth BFS over `Design.modules` + the `Module.instances[].module` edges
(no gate-graph walk). Most useful on a hierarchy design. Payload key:
`module_reachability`.

```json
{ "analysis": { "query": "module_reachability",
  "module_reachability": [
    { "module": "child_a", "reachable": true, "depth": 1, "instantiates": [], "instance_count": 0 },
    { "module": "top",     "reachable": true, "depth": 0,
      "instantiates": ["child_a", "child_b"], "instance_count": 2 } ] } }
```

| Field | Meaning |
| --- | --- |
| `reachable` | reachable from `design.top` |
| `depth` | min instance-graph distance from the top (`0` = top); present only when reachable |
| `instantiates` | the distinct child module names it directly instantiates (sorted) |
| `instance_count` | its direct-instance count (`≥ instantiates.len()` when a child is instantiated more than once) |

A dead (unreachable) module is reported `reachable: false` with no `depth`.

## `schema_version` stability contract

The document schema is **`1.13`** and evolves under a strict MINOR/MAJOR policy:

- a **MINOR** bump (e.g. `1.12 → 1.13`) is purely **additive** — a new optional
  field or a whole new payload array — and leaves every prior reply
  **byte-identical** (new sections are `skip_serializing_if`, so a query that does
  not use them is unchanged). Adding the four `analyze` query kinds was a sequence
  of MINOR bumps, each leaving the earlier queries' bytes untouched; `1.11 → 1.12`
  added the `coverage_readout` section + the standalone `coverage` tool document
  the same way, and `1.12 → 1.13` added the `design_metrics.num_mealy_fsm_modules`
  count (the Mealy FSM extension, decision `0024`).
- a **MAJOR** bump would be a breaking change to an existing field — none has
  occurred.

So an agent can pin a MINOR floor and rely on every field it already reads
staying put. Combined with content-addressing (same `(seed, knobs)` ⇒ same
`run_id` ⇒ same bytes), the API is deterministic and forward-compatible.

---

See also: [Tools](api-tools.md) (`introspect` / `analyze` call surfaces) and
[Overview & Protocol](api-reference.md) (the envelope, errors, versioning).
