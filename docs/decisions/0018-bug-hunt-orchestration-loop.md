---
id: bug-hunt-orchestration-loop
title: ANVIL's turnkey downstream bug-hunt loop is a thin, MCP-driveable orchestrator (fuzz → detect → minimize → reproducer bundle) composing the existing validate/minimize/diff-sim/introspect surfaces
answers:
  - "how do I run a turnkey downstream bug hunt with ANVIL"
  - "does ANVIL have a fuzz minimize reproducer loop"
  - "what is the anvil hunt command"
  - "is there an MCP hunt tool"
  - "what is the reproducer bundle format"
  - "how does the bug-hunt loop compose validate and minimize"
  - "can an agent drive the whole bug-hunt loop over MCP"
  - "what counts as a downstream-tool bug finding in ANVIL"
  - "does the hunt detect warnings and rejects and cross-sim mismatches"
  - "where do hunt reproducer bundles get written"
  - "is the anvil hunt CLI a shim over the MCP hunt tool"
  - "how is the bug-hunt loop sandboxed and reproducible"
date: 2026-06-17
status: accepted
tags: [usability, bug-hunt, mcp, api, downstream, minimize, reproducer, diff-sim, north-star, orchestration, agent]
evidence: docs/tasks/BUG-HUNT-ORCHESTRATION.md (the owning tree; this is its .1 design leaf); docs/decisions/0017-api-first-everything-mcp-accessible.md (the API-completeness gate this loop must satisfy); docs/decisions/0004-agent-introspection-mcp-lane.md (the MCP lane + SCHEMA-DERIVED / no-shadow-simulator ceiling + the sandbox/allow-list/RAM-guard/audit discipline the controlled tools already enforce); docs/decisions/0011-semantic-introspection-derived-query-surface.md (the derived-query surface the hunt results project through); src/downstream/mod.rs (the EXISTING library functions this composes — validate(seed,cfg,&ValidateOptions)->ValidateReport, minimize(seed,cfg,&MinimizeOptions)->MinimizeReport, ToolInvocation, AcceptanceTool allow-list, first_tool_warning folding warning into success=false); src/diff_sim/mod.rs (the cross-sim primitives baked_input_vectors/emit_testbench/is_sequential — the run+compare currently lives in src/bin/tool_matrix.rs and is extracted-then-reused, the DIFFERENTIAL-SIMULATION.3b.1 precedent); src/mcp/mod.rs (the dispatcher + run_validate/run_minimize + the anvil://audit/log surface the hunt tool extends); src/introspect/mod.rs (content_run_id FNV-1a content addressing + IntrospectionDocument the bundle carries); book/src/agent-mcp.md ("The bug-hunting loop, end to end" — the manual loop this lane makes turnkey)
---

# 0018 - BUG-HUNT-ORCHESTRATION: the turnkey downstream bug-hunt loop is a thin orchestrator over the existing surfaces

- Date: 2026-06-17
- Status: accepted (design accepted; implementation pending under the pre-split `BUG-HUNT-ORCHESTRATION.2`)
- Tree: `BUG-HUNT-ORCHESTRATION.1` (design/decision leaf; no code — pins the loop
  shape, the reproducer-bundle format, the MCP `hunt` tool + CLI shim, the
  detection policy, and the sandbox/reproducibility discipline; pre-splits `.2`)
- Activated by: autonomous PNT selection (`2026-06-17`) — the owner-recommended
  highest-leverage usability lane ("the single biggest usability multiplier"),
  registered by `USABILITY-LANE-OWNERSHIP.1`.
- Binds: decision [`0017`](0017-api-first-everything-mcp-accessible.md) (every
  control MCP-settable, every action MCP-invocable, every result queryable, all
  documented) and decision [`0004`](0004-agent-introspection-mcp-lane.md) (the
  controlled-tool sandbox/allow-list/RAM-guard/audit discipline + the
  no-shadow-simulator ceiling).

## Context

ANVIL's whole reason to exist is to **surface downstream-tool bugs** with RTL
that is legal by construction (`project_anvil_north_star`). The book already
describes the manual loop (`book/src/agent-mcp.md`, "The bug-hunting loop, end
to end"):

> generate → validate against Verilator / Yosys / iverilog → on a
> rejection/warning, shrink `(seed, knobs)` to a minimal reproducer → file it.

Every piece already exists, but **as separate surfaces** the user must wire up
by hand:

- `src/downstream/mod.rs` already exposes `validate(seed, cfg, &ValidateOptions)
  -> ValidateReport` and `minimize(seed, cfg, &MinimizeOptions) ->
  MinimizeReport` as library functions (the hardened, allow-listed,
  sandboxed, RAM-guarded, audit-logged invocation surface from
  `AGENT-INTROSPECTION-MCP.5.1`/`.5.2`/`.5.3`). `validate` already folds a
  *warning* into `success = false` via `first_tool_warning`, so reject and
  warning are already one unified failure signal. `minimize` already does
  deterministic coordinate-descent over the int + prob knob registries with a
  `validate`-backed failure oracle.
- `src/bin/tool_matrix.rs` already fuzzes scenarios across seeds and records
  per-module / per-design tool results + coverage facts, and (with `--diff-sim`)
  asserts cross-simulator trace agreement.
- `src/diff_sim/mod.rs` already holds the deterministic cross-sim primitives
  (`baked_input_vectors`, `emit_testbench`, `is_sequential`).
- `src/introspect/mod.rs` already content-addresses every `(seed, knobs)` to a
  reproducible `run_id` (`content_run_id`, FNV-1a) and projects construction
  truth into an `IntrospectionDocument`.
- `src/mcp/mod.rs` already dispatches `validate`/`minimize` as controlled tools
  and records every call to the `anvil://audit/log` resource.

What is missing is the **composition**: a single turnkey loop a user (or an
agent) invokes once that fuzzes a chosen tool across seeds, detects any
reject/warning/mismatch, auto-minimizes the failure, and drops a self-contained,
one-command-reproducible bundle. The user should not have to be the integration
layer.

## Decision

**The bug-hunt loop is a thin orchestrator, not a new engine.** It adds **no**
generation path, **no** new detection logic, and **no** new computed truth — it
*composes* the existing `generate` + `downstream::validate`/`minimize` +
(optional) diff-sim + `introspect` surfaces into one deterministic,
MCP-driveable loop. The default `anvil` build and `--artifact dut` stay
byte-identical; the loop is an opt-in subcommand + an opt-in MCP tool.

### 1. The orchestration-loop shape

A loop library lands as `src/hunt/mod.rs` exposing one pure-composition entry
point — `hunt::run(&HuntRequest) -> Result<HuntReport>` — so that **both** the
MCP `hunt` tool and the `anvil hunt` CLI are thin shims over the *same* function
(decision `0017`: the CLI is a shim over the API, never a superset). This
mirrors how `validate`/`minimize` live in `src/downstream/` and are wrapped by
the MCP `run_validate`/`run_minimize`.

The loop, per seed in a deterministic sweep `[base_seed, base_seed+1, …]`:

```
hunt(req):
  cfg0 := req.config (the knob profile; seed-independent knobs)
  for k in 0 .. req.seeds:
    seed := req.base_seed + k
    report := downstream::validate(seed, cfg0, ValidateOptions{
                tools, yosys_mode, mem_limits, sandbox_root (caller-set), keep_sandbox })
    failure := !report.ok                       # reject OR warning (validate already unifies these)
              OR (req.diff_sim AND cross-sim mismatch on this artifact)
    if report.declined: record memory-decline, continue       # RAM guard
    if not failure:
      record CLEAN verdict {seed, run_id}; continue
    # a candidate downstream-tool bug
    if req.minimize:
      m := downstream::minimize(seed, cfg0, MinimizeOptions{ tools, yosys_mode, max_oracle_calls, … })
      (failing_cfg, final_validate) := (m.minimized_config, m.final_validation)
    else:
      (failing_cfg, final_validate) := (cfg0, report)
    bundle := emit_reproducer_bundle(seed, failing_cfg, final_validate, diff_sim?, bundle_root)
    record FAILING verdict + minimize reductions + bundle ref
  return HuntReport { verdicts, failures, summary, declined }
```

Composition guarantees, by reuse rather than reinvention:

- **Detection** reuses `ValidateReport.ok` — no new warning/reject parser. The
  only *added* detector is the optional cross-sim mismatch, and that detector is
  itself a reuse: the `tool_matrix` diff-sim run+compare is **first extracted**
  into a reusable `diff_sim::run_agreement(...)` library entry (the
  `DIFFERENTIAL-SIMULATION.3b.1` extract-then-reuse precedent) so the loop calls
  it through a hardened surface instead of duplicating the harness.
- **Minimization** reuses `downstream::minimize` verbatim (its oracle is
  `validate`, so a minimized reproducer is guaranteed still-failing or the report
  says `reproduced_initial = false`).
- **Reproducibility** reuses `content_run_id`: the per-seed artifact is
  content-addressed, so the *whole hunt* is reproducible from
  `(base_seed, seeds, config, tools, yosys_mode, budgets)`.

### 2. The reproducer-bundle format

A **directory**, not an archive — chosen to match ANVIL's existing `--out` /
`tool_matrix` directory-tree convention, to stay inspectable/diffable/git-
attachable for filing, and so an agent gets `anvil://…` resource pointers into
it without unpacking. One bundle per failing run, at `<bundle_root>/<run_id>/`:

| File | Contents | Source |
| --- | --- | --- |
| `repro.sv` (+ extra `.sv` for a design) | the emitted SystemVerilog | the generator (deterministic from `seed` + `knobs.json`) |
| `knobs.json` | the effective (minimized) `Config` — the exact `(seed, knobs)` to reproduce | `dump_config` projection of the failing `Config` |
| `introspection.json` | the `IntrospectionDocument` (construction-truth) | `src/introspect` |
| `manifest.json` | expected-facts manifest (non-DUT lanes only; absent for DUT, matching the introspect contract) | the lane manifest |
| `tool-logs/` | the captured `stdout_log` / `stderr_log` per `ToolInvocation` | the sandbox logs `validate` already writes |
| `hunt-verdict.json` | `HuntFailure`: seed, run_id, failing tool+mode, failing `argv`, `first_error` (the reject/warning line), detection kind, minimize `reductions` + `oracle_calls`, and `mismatch_excerpt` when diff-sim fired | projection of `ValidateReport`/`MinimizeReport`/`DiffSimReport` |
| `repro.sh` | a one-command repro: regenerate the `.sv` from `(seed, knobs.json)` then re-run the exact failing `argv` | composed from `ToolInvocation.argv` |

`repro.sh` invokes the external tool by its allow-listed binary name and records
the tool version observed; it is portable because the `.sv` regeneration is
`anvil --seed <s> --config knobs.json` (byte-identical by the reproducibility
contract) and the tool line is the captured `argv` verbatim.

### 3. The MCP `hunt` tool + the `anvil hunt` CLI shim (decision `0017`)

`hunt` is a **controlled** MCP tool (it runs external tools, so it inherits the
exact allow-list / sandbox / RAM-guard / audit discipline of `validate` /
`minimize`). Input schema (every loop control is API-settable):

```json
{
  "lane": "dut|microdesign|frontend",   // default "dut"
  "seed": 42,                            // base seed (default 42)
  "seeds": 16,                           // sweep length (default 16)
  "config": { /* Config */ },            // the knob profile (optional)
  "n_params": 5, "n_children": 2,        // non-DUT lane knobs
  "tools": ["verilator", "yosys"],       // default ["verilator","yosys"]
  "yosys_mode": "without-abc|with-abc|both",
  "minimize": true,                      // default true
  "max_oracle_calls": 200,               // minimize budget per failure
  "diff_sim": false                      // cross-sim mismatch detection (default off)
}
```

Result (`HuntReport`) — every produced fact is queryable, and every field is a
**SCHEMA-DERIVED** projection of `ValidateReport`/`MinimizeReport`/
`DiffSimReport`/`ToolInvocation` (no new computed truth):

```json
{
  "lane": "dut", "base_seed": 42, "seeds": 16,
  "tools": ["verilator","yosys"], "yosys_mode": "without-abc",
  "verdicts": [ { "seed": 42, "run_id": "…", "ok": true }, … ],
  "failures": [
    { "seed": 57, "run_id": "…",
      "failing_tool": "yosys-with-abc", "failing_argv": ["…"],
      "first_error": "<reject/warning line>",
      "detection": "reject|warning|cross_sim_mismatch",
      "minimized": { "reproduced_initial": true, "reductions": [...],
                     "oracle_calls": 31, "minimized_run_id": "…",
                     "minimized_knobs": { /* Config */ } },
      "diff_sim": { "ran": true, "success": false, "n_samples": 8,
                    "mismatch_excerpt": "…" },
      "bundle": { "path": "<dir>",
                  "sv": "anvil://artifact/<run_id>/sv",
                  "introspection": "anvil://artifact/<run_id>/introspection",
                  "manifest": "anvil://artifact/<run_id>/manifest" } }
  ],
  "summary": { "n_seeds": 16, "n_clean": 15, "n_failures": 1, "n_reproduced": 1 },
  "declined": null
}
```

The hunt populates the MCP artifact cache for each failing `run_id`, so the
existing `anvil://artifact/<run_id>/{sv,introspection,manifest}` resource reads
work unchanged, and it appends a top-level `hunt` record to `anvil://audit/log`
(the sweep parameters + summary) on top of the per-call `validate`/`minimize`
audit records those functions already emit.

**The CLI shim** introduces ANVIL's first subcommand, `anvil hunt`:

```
anvil hunt [--lane dut] [--seed N] [--seeds K] [--config <path>]
           [--tools verilator,yosys,iverilog] [--yosys-mode <m>]
           [--no-minimize] [--budget N] [--diff-sim] [--out <dir>]
```

It parses into a `HuntRequest` and calls the *same* `hunt::run`. Two binding
constraints:

- **`--out` is a human-CLI convenience, not an agent capability.** The
  *tool sandbox* is always caller-set (never agent-supplied), exactly like
  `ValidateOptions.sandbox_root`. The CLI human may direct bundles to a chosen
  `--out <dir>`; the MCP `hunt` tool writes to a fixed sandboxed per-run dir and
  returns its path + resource URIs (the agent never supplies a path — the
  decision-`0004` sandbox rule).
- **The default path stays byte-identical.** Adding a `hunt` subcommand must not
  perturb the existing flat-flag default invocation (`anvil --seed N …`). The
  implementation must prove `tests/snapshots.rs` 6/6 and
  `tests/book_examples.rs::every_runnable_book_bash_block_succeeds` unchanged.

### 4. The detection policy

A **finding** (a candidate *downstream-tool* bug — never an ANVIL bug, because
the output is legal by construction) is any of:

1. **Reject** — a vetted tool exits non-zero (`ToolInvocation.success == false`
   with a non-zero `exit_code`).
2. **Warning** — a tool exits zero but emits a warning. ANVIL output is
   warning-clean by construction, so any warning is a finding; `validate`
   already folds it into `success == false` (`first_tool_warning`), and the
   `ToolInvocation.error` field carries the first warning line.
3. **Cross-sim mismatch** — when `diff_sim` is on, two independent simulators
   disagree on the post-reset trace (`DiffSimReport.success == false`).

The hunt **classifies, it does not adjudicate**: the real tools' results and
ANVIL's manifests/metrics remain the source of truth, and there is no shadow
oracle (decision `0004`, ROADMAP steering gap 4). Acceptance *divergence*
("tool A accepts / tool B rejects", and version-vs-version) is the natural next
detector and is owned by the synergistic `ACCEPTANCE-DIVERGENCE-HUNTING` lane;
the hunt loop is the engine it will plug into.

### 5. The sandbox / reproducibility discipline (decision `0004`)

Inherited wholesale by composing through `downstream::validate`/`minimize`:

- **Reproducible** — seeded ChaCha8 throughout; no wall-clock, no `thread_rng`;
  same `(seed, knobs)` ⇒ same `run_id` ⇒ byte-identical artifact. The seed sweep
  is deterministic, so a hunt run is itself reproducible from its request.
- **Sandboxed + allow-listed** — every spawn goes through the fixed
  `AcceptanceTool::from_name` allow-list (`verilator`/`yosys`/`iverilog`; unknown
  ⇒ clean error, never a spawn), a per-run sandbox temp dir, and no arbitrary
  shell.
- **RAM-guarded** — the `MemGuard`/`MemLimits` decline-under-pressure surfaces as
  `declined` per seed and in the report.
- **Audit-logged** — each underlying `validate`/`minimize` already appends a
  record to `anvil://audit/log`; the hunt adds its own top-level record.
- **Default-off / byte-identical** — opt-in subcommand + opt-in MCP tool; no new
  generation path; no emitted-RTL change.

## Pre-split of `BUG-HUNT-ORCHESTRATION.2` (implementation)

Ordered sub-leaves (refinable at pick time; each default-off / DUT
byte-identical, each carrying the decision-`0017` API-completeness gate):

- `.2a` — **extract** the `tool_matrix` diff-sim run+compare into a reusable
  `diff_sim::run_agreement(...)` library entry (pure refactor; the `.3b.1`
  precedent) so the loop can detect cross-sim mismatch through a hardened
  surface. *(Orderable first; the first hunt cut may ship reject/warning-only and
  fold this in next.)*
- `.2b` — the `src/hunt/` **library core**: `HuntRequest`/`HuntReport`/
  `HuntFailure` types + `hunt::run` composing `validate`/`minimize` (+ optional
  diff-sim) over the deterministic seed sweep + the reproducer-bundle emitter;
  cargo-portable proofs. No CLI/MCP yet.
- `.2c` — the MCP **`hunt` controlled tool** wired into the dispatcher (input
  schema, `HuntReport` result, failing-run cache population, audit record);
  introspection/MCP doc + schema note; proofs.
- `.2d` — the **`anvil hunt` CLI subcommand** shim over `hunt::run`, with the
  byte-identical default-path guard; proofs incl. snapshots 6/6 + book-examples
  3/3 unchanged.
- `.2e` — a **real-tool end-to-end gate** (`#[ignore]`, tool-gated) that runs a
  hunt against Verilator/Yosys and produces a one-command-reproducible bundle for
  an injected/known failure; `book/src/agent-mcp.md` + USER_GUIDE + README + a KM
  card; close `.2` and the tree.

## Rejected alternatives

- **A bundle as a single archive (`.tar`/`.zip`).** Rejected: a directory matches
  the existing `--out`/`tool_matrix` convention, is inspectable/diffable/git-
  attachable, and lets an agent fetch parts as `anvil://…` resources without
  unpacking. (An archive view can be a trivial later add-on if a user asks.)
- **A `hunt` engine that re-implements fuzz/detect/minimize.** Rejected: it would
  fork detection logic away from `validate` (drift risk) and violate
  full-factorization. The loop *composes* `downstream::validate`/`minimize`; it
  adds no second detector and no second minimizer.
- **A new bespoke warning/reject parser in the hunt.** Rejected: `validate`
  already unifies reject+warning into `ok == false`; reusing it keeps one source
  of truth for "is this a finding?".
- **An `anvil --hunt` flag instead of a subcommand.** Rejected: the hunt has its
  own rich, mutually-exclusive option set (`--seeds`, `--budget`, `--no-minimize`,
  …) that is awkward to overlay on the generate flags; a `hunt` subcommand keeps
  the surfaces clean and the default generate path untouched. Nothing is retired.
- **CLI-only (no MCP tool), or MCP-only (no CLI).** Both rejected by decision
  `0017`: the action must be MCP-invocable *and* the CLI must be a shim over the
  same `hunt::run` — never a parallel or superset surface.
- **An agent-supplied bundle/sandbox path over MCP.** Rejected by decision
  `0004`: the agent never supplies a filesystem path for a controlled tool; the
  MCP `hunt` writes to a fixed sandbox and returns the path + resource URIs.
- **A behavioural / "what is the right answer" oracle to adjudicate findings.**
  Forbidden (decision `0004`, gap 4): the hunt classifies reject/warning/mismatch;
  the real tools remain the source of truth.

## Consequences

- The implementation (`.2`) lands `src/hunt/` + the MCP `hunt` tool + the
  `anvil hunt` CLI + the reproducer-bundle emitter, all as composition over
  proven surfaces — small, auditable, and default-off / DUT byte-identical.
- `--diff-sim`'s run+compare gets promoted from a `tool_matrix`-private path to a
  reusable `diff_sim::run_agreement` library entry (`.2a`), which also makes the
  cross-sim detector available to `ACCEPTANCE-DIVERGENCE-HUNTING` and any future
  consumer — a reuse win beyond this lane.
- The MCP surface gains its first **orchestration** tool (vs. the existing
  single-step tools) while staying inside the controlled-tool discipline; the
  introspection schema grows additively (per its MINOR policy) for any new
  `HuntReport` projection that is also served as a resource.
- ANVIL becomes *directly usable as a downstream bug-finder* — the north star —
  rather than a generator the user must wrap. The feeding lanes
  (`KNOB-ERGONOMICS-AND-PRESETS` for `--profile` knob bundles,
  `DOWNSTREAM-ADAPTER-EXPANSION` for more acceptance tools) plug into this engine
  without reopening it.

## Links

- Owning tree: `BUG-HUNT-ORCHESTRATION` (this is its `.1` design leaf; pre-splits
  `.2a`…`.2e`).
- Parent decisions: `0017` (API-completeness gate), `0004` (MCP lane +
  sandbox/allow-list/RAM-guard/audit + no-shadow-simulator), `0011` (derived-query
  surface the results project through).
- Composed surfaces: `src/downstream/mod.rs` (validate/minimize/ToolInvocation/
  AcceptanceTool), `src/diff_sim/mod.rs` (cross-sim primitives), `src/mcp/mod.rs`
  (dispatcher + audit log), `src/introspect/mod.rs` (content addressing +
  document), `src/bin/tool_matrix.rs` (the fuzz+diff-sim precedent).
- Book: `book/src/agent-mcp.md` ("The bug-hunting loop, end to end") — the manual
  loop this lane makes turnkey.
- Memory: `project_anvil_north_star` (surface downstream-tool bugs),
  `feedback_api_for_agents_not_humans` (design the API for agents).
- Synergistic lanes: `ACCEPTANCE-DIVERGENCE-HUNTING`,
  `DOWNSTREAM-ADAPTER-EXPANSION`, `KNOB-ERGONOMICS-AND-PRESETS`,
  `CI-PACKAGING-DISTRIBUTION` (the CI wrapper around this engine).
