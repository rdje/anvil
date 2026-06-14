# AGENT-INTROSPECTION-MCP: Agent-Drivable Introspection + MCP Interface

## Metadata

- Tree ID: `AGENT-INTROSPECTION-MCP`
- Status: `done`
- Roadmap lane: `Capability — agent-drivable introspection + MCP interface`
- Created: `2026-06-14`
- Last updated: `2026-06-15`
- Closed: `2026-06-15` (all leaves `.1`–`.7` done)
- Owner: repo-local workflow (owner-directed lane)

## Goal

Make ANVIL agent-drivable for the downstream-bug-hunting loop by exposing
deep *construction-truth* introspection and control through a stable,
versioned, default-off API, with a thin read-mostly MCP adapter **beside**
the generator core. The closing capability is the autonomous loop:
generate → validate (Verilator / Yosys / iverilog / `--diff-sim`) → on a
tool failure shrink `(seed, knobs)` to a minimal reproducer → emit it.

Architecture and the transferred-vs-dropped reference-advice analysis are
recorded in
[`docs/decisions/0004-agent-introspection-mcp-lane.md`](../decisions/0004-agent-introspection-mcp-lane.md).

## Non-Goals

- **No stateful simulator-style session API** (`run_until`, `force_signal`,
  waveform DB, signal-over-time, `explain_x`, sensitivity trees,
  interactive stepping). ANVIL is a pure `(seed, knobs) -> artifact`
  function plus pure post-hoc analysis; it has no temporal session.
- No MCP logic inside the generator kernel; no new generation path; no
  generate-then-filter or output mutation/repair via the API.
- The AI agent is never a signoff oracle — ANVIL's manifests/metrics/tool
  results remain the source of truth.
- No second source of introspection truth (the schema is derived from
  existing `metrics`/`manifest`/`config`).
- No arbitrary-shell tool; no change to the default `--artifact dut`
  byte-identical contract.

## Acceptance Criteria

- A stable, versioned introspection schema exists, derived from existing
  `metrics`/`manifest`/`config` (no parallel re-implementation).
- A default-off MCP adapter (separate target) exposes resources
  (`.sv`/manifest/metrics/coverage/config + knob & motif catalogs) and a
  safe tool set (`generate`/`introspect`/`dump_config`/`coverage_gaps`)
  with deterministic run ids.
- A controlled `validate`/`minimize` tool set runs external tools only
  through the existing hardened `tool_matrix` invocations, sandboxed and
  resource-guarded.
- Agent workflows (find downstream bug, close coverage gap, minimize
  reproducer, triage failures, explain artifact) are exercised end-to-end.
- DUT lane stays byte-identical (snapshots 6/6); book + USER_GUIDE document
  the lane; downstream/self-checks clean.

## Task Tree

- ID: `AGENT-INTROSPECTION-MCP`
  Status: `done`
  Goal: `Agent-drivable introspection + MCP interface beside the core.`
  Children: `.1`, `.2`, `.3`, `.4`, `.5`, `.6`, `.7` (all done)

- ID: `AGENT-INTROSPECTION-MCP.1`
  Status: `done`
  Goal: `Design the lane + land decision record 0004 (architecture, transferred-vs-dropped reference advice, guardrails, phasing).`
  Acceptance: `docs/decisions/0004 + DEVELOPMENT_NOTES design note + this tree landed; no code; doctrine guardrails explicit.`
  Verification: `scripts/check_memory_architecture.sh`; `knowledge-map/scripts/check_knowledge_map.sh`; `git diff --check`.
  Commit: `AGENT-INTROSPECTION-MCP.1 - design + decision record 0004`

- ID: `AGENT-INTROSPECTION-MCP.2`
  Status: `done`
  Goal: `Specify the stable, versioned introspection JSON schema, derived strictly from existing metrics/manifest/config; map each field to its existing source. Docs-only.`
  Acceptance: `Schema spec doc lists every field + provenance; confirms zero new computed truth; versioning policy stated.`
  Verification: `docs/AGENT_INTROSPECTION_SCHEMA.md landed; scripts/check_memory_architecture.sh; knowledge-map/scripts/check_knowledge_map.sh; git diff --check.`
  Commit: `AGENT-INTROSPECTION-MCP.2 - introspection schema spec (docs)`

- ID: `AGENT-INTROSPECTION-MCP.3`
  Status: `done`
  Goal: `Implement the introspection emission surface (anvil introspect / structured JSON dump) over the .2 schema. Additive, default-off-equivalent, DUT byte-identical.`
  Acceptance: `New surface emits the .2 schema from existing facts; snapshots 6/6; no change to existing stdout/manifest/CLI defaults.`
  Verification: `src/introspect/mod.rs (6 lib tests, all pass) + --introspect CLI flag; cargo fmt/check/clippy -D warnings clean; cargo test --test snapshots 6/6 byte-identical; CLI smoke (module + design + guard + JSON validity).`
  Commit: `AGENT-INTROSPECTION-MCP.3 - introspection emission surface`

- ID: `AGENT-INTROSPECTION-MCP.4`
  Status: `done`
  Goal: `Implement the read-only in-process MCP server (separate target): resources + pure/safe tools (generate/introspect/dump_config/coverage_gaps); deterministic run ids; content-addressed artifact cache; no external-tool exec.`
  Acceptance: `Server lists resources + safe tools; round-trips a generate+introspect; default anvil build unaffected; DUT byte-identical.`
  Verification: `src/mcp/mod.rs (12 lib tests) + src/bin/anvil_mcp.rs (anvil-mcp target) + Cargo.toml [[bin]]; cargo fmt/check/clippy -D warnings clean; cargo test --lib 338/338; snapshots 6/6 byte-identical; end-to-end stdio smoke (initialize/tools.list/generate/resources). coverage_gaps deferred to .5 (needs external-tool exec).`
  Commit: `AGENT-INTROSPECTION-MCP.4 - read-only MCP server`

- ID: `AGENT-INTROSPECTION-MCP.5`
  Status: `done`
  Goal: `Add the controlled validate + minimize tools: external tools only via existing tool_matrix invocations, sandboxed + ram-guarded; minimize shrinks (seed, knobs); audit log + reproducible command line per call.`
  Children: `.5.1`, `.5.2`, `.5.3` (all `done`)
  Split rationale (`2026-06-14`): the original `.5` leaf bundled three
  independently-reviewable concerns — a cross-binary tool-invocation
  extraction (a lower-level dependency that must land first), the sandboxed
  `validate` orchestration, and the `minimize` delta-debugger — plus security
  guardrails, so per `docs/TASK_TREE.md` "Splitting Rules" it is split rather
  than landed as one over-broad slice.

- ID: `AGENT-INTROSPECTION-MCP.5.1`
  Status: `done`
  Goal: `Extract the hardened downstream-tool invocation surface (verilator --lint-only / yosys synth / iverilog -g2012 acceptance command lines + warning-as-failure detection + the ToolInvocation report row + YosysMode) from the tool_matrix binary into a shared library module so the validate/minimize tools reuse the existing hardened invocations instead of forking a second, drift-prone set. Pure behavior-preserving refactor.`
  Acceptance: `New src/downstream/mod.rs owns the invocations; src/bin/tool_matrix.rs uses them via use anvil::downstream::{…}; serialized ToolInvocation/report shape unchanged (banked reports + --resume valid); matrix tool tests + snapshots prove no drift; DUT byte-identical.`
  Verification: `cargo fmt --all --check; cargo check --all-targets; cargo clippy --all-targets -- -D warnings; cargo test --lib downstream:: (7/7); cargo test --bin tool_matrix (41 pass, 1 ignored); cargo test --test snapshots (6/6 byte-identical).`
  Commit: `AGENT-INTROSPECTION-MCP.5.1 - shared downstream-tool invocation surface`

- ID: `AGENT-INTROSPECTION-MCP.5.2`
  Status: `done`
  Goal: `The controlled validate tool over the .5.1 surface: generate (seed, knobs) into a sandboxed temp dir under a project-root/tmp scope, run the selected acceptance tools, ram-guard the run (reuse mem_guard / scripts/ram_guard.sh envelope), return structured ToolInvocation reports + an overall verdict, and audit-log the reproducible (seed, knobs) + exact command line per call; no arbitrary shell.`
  Acceptance: `validate(seed, knobs, tools) returns structured per-tool reports + overall verdict; sandbox + ram-guard + audit-log guardrails enforced and unit-tested; tool-gated end-to-end smoke when tools are present.`
  Verification: `cargo fmt/check/clippy -D warnings clean; cargo test --lib downstream:: (12/12 + 1 tool-gated) + mcp:: (15/15); cargo test --test snapshots (6/6 byte-identical); tool-gated e2e (--ignored) clean vs real Verilator+Yosys (seed 42 ok=true); anvil-mcp stdio smoke (initialize → validate → anvil://audit/log).`
  Commit: `AGENT-INTROSPECTION-MCP.5.2 - controlled validate tool`

- ID: `AGENT-INTROSPECTION-MCP.5.3`
  Status: `done`
  Goal: `The minimize tool: deterministic delta-debug of (seed, knobs) to a smaller failing reproducer using .5.2's validate as the failure oracle; bounded work budget; audit-logged.`
  Acceptance: `minimize produces a smaller failing (seed, knobs) for a seeded failing case; bounded + deterministic; guardrails tested.`
  Verification: `cargo fmt/check/clippy -D warnings clean; cargo test --lib downstream:: (20/20 + 1 tool-gated) — shrink logic proven with a synthetic predicate oracle (bisection finds the monotone boundary; unconstrained bounds collapse to floors; a depended-on knob is preserved; budget + guard-decline both stop the search) + real-oracle no-repro/determinism/invalid-config paths; cargo test --lib mcp:: (18/18) incl. minimize round-trip + audit + off-allow-list/zero-budget rejection; cargo test --test snapshots (6/6 byte-identical); tool-gated e2e (--ignored) clean vs real Verilator 5.046 + Yosys 0.64 (seed 42 reproduced_initial=false); anvil-mcp stdio smoke (initialize → minimize → anvil://audit/log).`
  Commit: `AGENT-INTROSPECTION-MCP.5.3 - controlled minimize tool`

- ID: `AGENT-INTROSPECTION-MCP.6`
  Status: `done`
  Goal: `Package the agent-workflow prompts (find_downstream_bug, close_coverage_gap, minimize_reproducer, triage_tool_failures, explain_artifact).`
  Acceptance: `Each prompt drives its tool chain end-to-end on a sample.`
  Verification: `Implemented as first-class MCP prompts in src/mcp/ (prompts capability + prompts/list + prompts/get over a fixed PROMPTS registry; pure renderers with sample-arg substitution + required-arg + type validation). cargo fmt/check/clippy -D warnings clean; cargo test --lib mcp:: (24/24, +6 prompt tests incl. each_workflow_tool_chain_runs_end_to_end_on_a_sample which executes all five chains portably via tools:[]); cargo test --lib (370/370, 2 gated); cargo test --test snapshots (6/6 byte-identical); anvil-mcp stdio smoke (initialize advertises prompts; prompts/list lists the 5; prompts/get renders + substitutes args; required-arg error -32602).`
  Commit: `AGENT-INTROSPECTION-MCP.6 - agent-workflow prompts`

- ID: `AGENT-INTROSPECTION-MCP.7`
  Status: `done`
  Goal: `Book chapter + USER_GUIDE section + CODEBASE_ANALYSIS update + closeout.`
  Acceptance: `mdBook documents the lane; USER_GUIDE shows invocation; live docs synced; tree closed.`
  Verification: `New mdBook chapter book/src/agent-mcp.md (added to SUMMARY.md under Reference) documents the whole lane (--introspect, anvil-mcp tools/resources/prompts, the bug-hunting loop, guardrails) with real captured examples; USER_GUIDE.md "Agent introspection and the MCP server" section shows --introspect + anvil-mcp invocation; README.md CLI-truth + key-paths updated (--introspect, anvil-mcp, src/introspect|downstream|mcp); CODEBASE_ANALYSIS mcp section synced (done in .6). mdbook build clean; cargo test --test book_examples 3/3 (runnable cargo run --release -- --seed 42 --introspect block proven; two MCP-setup blocks skip-sentinelled with reasons). Pure-docs leaf — no code changed, snapshots remain 6/6.`
  Commit: `AGENT-INTROSPECTION-MCP.7 - book + USER_GUIDE + README closeout`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `AGENT-INTROSPECTION-MCP.1` | `done` | Design + decision record `0004` landed. |
| 2 | `AGENT-INTROSPECTION-MCP.2` | `done` | Schema spec landed: `docs/AGENT_INTROSPECTION_SCHEMA.md`. |
| 3 | `AGENT-INTROSPECTION-MCP.3` | `done` | Emission surface landed: `src/introspect/` + `--introspect` flag; DUT byte-identical. |
| 4 | `AGENT-INTROSPECTION-MCP.4` | `done` | Read-only MCP server landed: `src/mcp/` + `anvil-mcp` bin (stdio JSON-RPC; generate/introspect/dump_config + resources). |
| 5 | `AGENT-INTROSPECTION-MCP.5.1` | `done` | Shared downstream-tool invocation surface extracted to `src/downstream/`; `tool_matrix` rewired; behavior-preserving (snapshots 6/6). |
| 6 | `AGENT-INTROSPECTION-MCP.5.2` | `done` | Controlled `validate` tool: `downstream::validate` (sandboxed temp dir + ram-guard + fixed allow-list) + MCP `validate` tool + `anvil://audit/log`; e2e clean vs Verilator+Yosys. |
| 7 | `AGENT-INTROSPECTION-MCP.5.3` | `done` | `minimize` delta-debugger: `downstream::minimize` (deterministic coordinate-descent over size bounds + optional-motif probs, budget-bounded, seed fixed) + MCP `minimize` tool + audit log; e2e clean vs real Verilator+Yosys. |
| 8 | `AGENT-INTROSPECTION-MCP.6` | `done` | Agent-workflow prompts landed as first-class **MCP prompts** (`prompts/list` / `prompts/get`) in `src/mcp/`: `find_downstream_bug`, `close_coverage_gap`, `minimize_reproducer`, `triage_tool_failures`, `explain_artifact` — each renders its ordered tool chain with sample-arg substitution; every chain proven runnable end-to-end through the server. |
| 9 | `AGENT-INTROSPECTION-MCP.7` | `done` | User-facing closeout: new mdBook chapter `book/src/agent-mcp.md` (Reference) + USER_GUIDE section + README CLI-truth/key-paths sync, documenting the whole lane with real examples. `book_examples` gate 3/3. **Tree closed.** |

**Tree closed `2026-06-15`.** All leaves `.1`–`.7` done. Owner **accepted** the
`.1`/`.2` design (`2026-06-14`), unblocking the code leaves: `.3` introspection
emission surface, `.4` read-only MCP server, `.5` (`.5.1`/`.5.2`/`.5.3`)
controlled `validate`/`minimize` tools, `.6` agent-workflow prompts as MCP
prompts, `.7` the user-facing book/USER_GUIDE/README closeout. The full
acceptance is met: versioned introspection schema derived from existing facts;
default-off MCP adapter exposing resources + pure tools + controlled tools +
prompts with deterministic run ids; controlled tools run external tools only
through the hardened `downstream` invocations (sandboxed + RAM-guarded +
audit-logged); the five agent workflows packaged and proven runnable;
DUT lane byte-identical throughout (snapshots 6/6); book + USER_GUIDE document
the lane. Any further breadth (e.g. exposing `coverage_gaps` as an MCP tool, an
HTTP transport, microdesign/frontend lanes over MCP) is optional post-closure
work and does not reopen the tree.

## Decisions

- `2026-06-15`: **Landed `.7` and closed the tree.** The user-facing closeout:
  a new mdBook chapter `book/src/agent-mcp.md` (added to `SUMMARY.md` under
  Reference) documents the whole lane — `--introspect`, the `anvil-mcp` server
  (tools/resources/prompts), the bug-hunting loop, and the guardrails — with
  examples captured from real runs (the `--introspect` document, the tool /
  resource / prompt listings). A `USER_GUIDE.md` "Agent introspection and the
  MCP server" section shows invocation; `README.md` gained the `--introspect`
  and `anvil-mcp` CLI-truth bullets plus the `src/introspect`/`downstream`/`mcp`
  key paths. Per the book doctrine, the one genuinely-runnable example uses
  `cargo run --release -- --seed 42 --introspect` and is proven by the
  `book_examples` gate; the two MCP-setup blocks (build `anvil-mcp`; `claude mcp
  add`) carry skip sentinels with reasons (they invoke a different binary / an
  external CLI). Pure-docs leaf — no `src/`/`tests/` change, so the DUT
  byte-identical contract is untouched (snapshots remain 6/6 from `.6`). With
  `.7` done, **all seven leaves are complete and the `AGENT-INTROSPECTION-MCP`
  tree is closed**; the lane is now documented as a stable, default-off feature.
- `2026-06-15`: **Landed `.6`** — the five agent-workflow prompts, packaged as
  first-class **MCP prompts** (the third MCP primitive beside tools +
  resources). Rationale: decision `0004` maps "Prompts (workflows)" onto ANVIL,
  and MCP's `prompts/list`/`prompts/get` is the canonical, agent-drivable way to
  package a workflow so a client can fetch and execute it — strictly better than
  static doc text (the `.1` phasing hint), and it satisfies the leaf acceptance
  ("each prompt drives its tool chain end-to-end on a sample") directly. Design:
  (a) a fixed `PROMPTS` registry of `PromptSpec { name, description, args,
  render }` is the single owner of the prompt set, so it cannot drift from the
  dispatch; (b) each prompt is **pure guidance** — its renderer instantiates an
  ordered chain over the *existing* tools/resources with the caller's sample
  arguments; it adds **no** new capability and computes **no** new truth
  (consistent with the lane's read-mostly, no-second-source-of-truth doctrine);
  (c) `prompts/get` validates the prompt name, that argument values are strings
  (the MCP prompt-argument contract), and that every declared-required argument
  is present, before rendering — a malformed request is a clean `-32602` error,
  never a panic; (d) the five chains are: `find_downstream_bug`
  (generate → validate → on-failure minimize), `close_coverage_gap`
  (knobs catalog → dump_config → introspect to confirm the metric lit →
  validate), `minimize_reproducer` (minimize → audit log; seed held fixed),
  `triage_tool_failures` (validate → per-tool argv/output → audit log), and
  `explain_artifact` (generate → introspect → read the `.sv` resource). The
  end-to-end test drives all five chains through the server portably (the
  external-tool legs use `tools: []`), proving each prompt names a real,
  runnable sequence. `initialize` now advertises the `prompts` capability.
  Default `anvil` build / DUT byte-identical untouched (snapshots 6/6).
  User-facing book/USER_GUIDE/README docs remain deferred to `.7`.
- `2026-06-15`: **Landed `.5.3`** — the controlled `minimize` delta-debugger,
  closing the `.5` container. `downstream::minimize(seed, &Config,
  &MinimizeOptions) -> MinimizeReport` delta-debugs `(seed, knobs)` to a smaller
  failing reproducer using `.5.2`'s `downstream::validate` as a **pure failure
  oracle** (a candidate "reproduces" iff its `validate` run completes — guard
  did not decline — and the verdict is not `ok`). Design decisions: (a) the
  **seed is held fixed** — it pins the reproducer's identity; only knobs shrink;
  (b) a **deterministic coordinate-descent** over two fixed-order registries —
  integer size bounds bisected toward each knob's floor (floor tracks the
  companion `min_*` so the range stays valid) and optional-motif probabilities
  driven to `0.0` ("feature off"); sharing/reuse/library/constant knobs are
  excluded because `0.0` there is not unambiguously simpler; (c) **bounded +
  safe** — every candidate is re-checked with `Config::validate` before it can
  reach the generator, and the search is hard-capped by
  `max_oracle_calls` (default 200) + a `MINIMIZE_MAX_PASSES` fixpoint bound, a
  decline unwinds cleanly; (d) **no monotonicity assumed** — the result is *a*
  smaller reproducer, not a proven global minimum (the standard delta-debug
  trade-off, documented). The MCP `minimize` tool reuses the shared
  `parse_validate_tools`/`parse_yosys_mode_arg` helpers (so it cannot drift from
  `validate`), fixes the sandbox to the OS temp dir, and audit-logs each call
  (minimized `run_id`, seed, reductions, budget, surviving command lines). The
  shrink logic is unit-tested portably via a **synthetic predicate oracle**
  (ANVIL output is valid-by-construction, so no real tool can manufacture a
  failing case to delta-debug); the real-oracle wiring is proven by the
  `tools: []` no-repro test and the tool-gated e2e (`reproduced_initial=false`
  on seed 42 — the honest valid-by-construction outcome). Default `anvil`
  build / DUT byte-identical untouched (snapshots 6/6). User-facing docs remain
  deferred to `.7`.
- `2026-06-14`: **Landed `.5.2`** — the controlled `validate` tool.
  `downstream::validate(seed, &Config, &ValidateOptions)` regenerates the DUT
  artifact deterministically into a fresh per-run sandbox
  (`<root>/anvil-validate-<run_id>/`), runs the selected `AcceptanceTool`
  allow-list (`verilator`/`yosys`/`iverilog`, fixed binary names) via the
  `.5.1` runners, checks `MemGuard` before each spawn (decline-to-start-more,
  new `MemGuard::from_limits`), and returns per-tool `ToolInvocation`s + an
  overall verdict (`ValidateReport`). The MCP `validate` tool fixes the sandbox
  to the OS temp dir (never agent-supplied), audit-logs each call to
  `anvil://audit/log` with the exact reproducible command lines, and rejects
  off-allow-list tool names / yosys modes with clean errors. `validate` reuses
  the now-`pub` `introspect::content_run_id` so it shares the one content
  address with `generate`/`introspect`. Guardrails (decision `0004`): no
  arbitrary shell, no agent-supplied path, ram-guard decline path — all
  unit-tested; tool-gated e2e clean vs real Verilator+Yosys. Default `anvil`
  build / DUT byte-identical untouched (snapshots 6/6). User-facing docs remain
  deferred to `.7`.
- `2026-06-14`: **Split `.5` and landed `.5.1`** — the controlled-tools leaf
  `.5` was split into `.5.1` (shared invocation surface), `.5.2` (validate),
  `.5.3` (minimize) per the `docs/TASK_TREE.md` splitting rules (it bundled a
  lower-level dependency + two independently-reviewable features). `.5.1`
  extracted the hardened acceptance-tool invocations
  (`verilator --lint-only` / `yosys synth` / `iverilog -g2012`, the
  warning-as-failure detector, `ToolInvocation`, `YosysMode`,
  `yosys_mode_slug`, and the double-quote escapers) out of
  `src/bin/tool_matrix.rs` into a new library module `src/downstream/mod.rs`,
  and rewired the binary to `use anvil::downstream::{…}`. This is the
  full-factorization move (`feedback_full_factorization.md`) that `0004`
  requires so the `.5.2`/`.5.3` tools reuse the **existing** vetted invocations
  rather than forking a second source of truth — the same pattern
  `DIFFERENTIAL-SIMULATION.3a` used for `src/diff_sim/`. Pure
  behavior-preserving refactor: the serialized `ToolInvocation` JSON shape is
  unchanged (banked matrix reports + `--resume` checkpoints stay valid), the
  matrix's own tool tests pass unchanged, and `tests/snapshots.rs` stays 6/6
  byte-identical (DUT contract preserved). No new CLI surface; user-facing docs
  remain deferred to `.7`.
- `2026-06-14`: **Owner accepted the `.1`/`.2` design** — code leaves
  `.3`–`.7` are unblocked; execution proceeds under continuous PNT.
- `2026-06-14`: `.4` landed the read-only MCP server — `src/mcp/mod.rs` (pure
  JSON-RPC 2.0 dispatch + content-addressed artifact cache) and a thin stdio
  bin `src/bin/anvil_mcp.rs` (explicit `anvil-mcp` `[[bin]]` in `Cargo.toml`).
  Design points: (a) **hand-rolled** newline-delimited JSON-RPC over stdio —
  no async/SDK dependency (rejected `rmcp` + `tokio`), matching `0004`'s
  "simple in-process stdio server"; (b) `McpServer::handle` is a pure `Value →
  Option<Value>` function so the whole protocol surface is unit-tested
  in-process (12 tests), the bin is just transport; (c) determinism →
  content-addressed cache: `generate` caches by document `run_id`,
  `resources/read` serves the cached `.sv` / introspection back; (d) pure/safe
  tools only (`generate`/`introspect`/`dump_config`) — no FS writes, no shell,
  no external tools; `coverage_gaps`/`validate`/`minimize` (external-tool
  exec) are `.5`. DUT byte-identical (snapshots 6/6); default `anvil` build
  unaffected (separate target).
- `2026-06-14`: `.3` landed the emission surface — `src/introspect/mod.rs`
  (typed envelope + pure `module_document` / `design_document` builders) and a
  default-off `--introspect` CLI flag that, on a single-artifact stdout run,
  prints the schema document instead of SV. Design points: (a) `run_id` is a
  content address (FNV-1a 64-bit over `(schema_version, anvil_version, lane,
  seed, knobs)`), not a nonce — deterministic, matching `0004`'s
  content-addressed cache; (b) the surface is single-shot-only (rejects
  `--out` / `--count > 1`) to keep the streamed `--out` path byte-identical
  and never touched; (c) `coverage` + lane manifests are deferred (matrix-only
  / `.4`+), recorded via a `warnings[]` note. DUT byte-identical verified by
  snapshots 6/6.
- `2026-06-14`: Architecture, the transferred-vs-dropped reference-advice
  analysis, the security model, and the determinism→content-addressed-cache
  simplification are recorded in
  [`docs/decisions/0004`](../decisions/0004-agent-introspection-mcp-lane.md)
  and `DEVELOPMENT_NOTES.md`. Summary: MCP is a thin read-mostly adapter
  beside a deterministic core; the introspection schema is derived from
  existing facts; ANVIL needs no stateful simulator-style session.
- `2026-06-14`: Design-first cadence — `.1`/`.2` are docs; no code until
  the schema/architecture is accepted.
- `2026-06-14`: `.2` landed the introspection schema spec
  (`docs/AGENT_INTROSPECTION_SCHEMA.md`). Key contract decisions: the schema
  is a thin **versioned envelope** (`schema_version = "1.0"`, `anvil_version`,
  `lane`, `request` determinism-tuple echo with content-addressed `run_id`,
  `artifact` descriptor, `introspection` payload, `warnings`) whose payload
  sections are the **exact serde projections** of existing structs — `config`
  ← `Config`, `module_metrics` ← `Metrics`, `design_metrics` ←
  `DesignMetrics`, `coverage` ← `tool_matrix::CoverageSummary`, the lane
  manifests ← `microdesign`/`frontend::Manifest`, and `.sv` as a
  fetch-on-demand resource. Invariant SCHEMA-DERIVED: the adapter computes
  **zero** new truth; struct field lists stay owned by the code (no second
  source of truth, per `0004`). Versioning: `MAJOR.MINOR`, additive
  `#[serde(default)]` growth = MINOR, rename/retype/semantic change = MAJOR,
  lockstep with `anvil_version`, determinism preserved across versions.

## Open Questions

- Transport: stdio MCP server first vs HTTP/service later (recommend stdio).
- Crate layout: separate `anvil-mcp` target vs feature-gated module
  (recommend separate target).
- Whether `validate` ships in the first MCP cut or stays CLI-only initially
  (recommend read-only introspection + `generate` first; guarded `validate`
  at `.5`).
- Introspection-schema versioning policy.

## Blockers

- None. Implementation leaves (`.3`+) are gated on owner acceptance of the
  `.1`/`.2` design, not on a technical blocker.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-06-14` | `AGENT-INTROSPECTION-MCP.1` | `scripts/check_memory_architecture.sh`; `knowledge-map/scripts/check_knowledge_map.sh`; `git diff --check` | passed |
| `2026-06-14` | `AGENT-INTROSPECTION-MCP.2` | `scripts/check_memory_architecture.sh`; `knowledge-map/scripts/check_knowledge_map.sh`; `git diff --check`; `cargo check --all-targets` (no code touched) | passed |
| `2026-06-14` | `AGENT-INTROSPECTION-MCP.3` | `cargo fmt --all --check`; `cargo check --all-targets`; `cargo clippy --all-targets -- -D warnings`; `cargo test --lib introspect` (6/6); `cargo test --test snapshots` (6/6 byte-identical); CLI smoke (module/design/guard/JSON) | passed |
| `2026-06-14` | `AGENT-INTROSPECTION-MCP.4` | `cargo fmt --all --check`; `cargo check --all-targets` (no dup-bin); `cargo clippy --all-targets -- -D warnings`; `cargo test --lib` (338/338, incl 12 mcp); `cargo test --test snapshots` (6/6 byte-identical); end-to-end `anvil-mcp` stdio smoke (initialize/tools.list/generate/resources) | passed |
| `2026-06-14` | `AGENT-INTROSPECTION-MCP.5.1` | `cargo fmt --all --check`; `cargo check --all-targets`; `cargo clippy --all-targets -- -D warnings`; `cargo test --lib downstream::` (7/7); `cargo test --bin tool_matrix` (41 pass, 1 ignored); `cargo test --test snapshots` (6/6 byte-identical) | passed |
| `2026-06-14` | `AGENT-INTROSPECTION-MCP.5.2` | `cargo fmt/check/clippy -D warnings`; `cargo test --lib downstream::` (12/12 + 1 gated) + `mcp::` (15/15); `cargo test --test snapshots` (6/6 byte-identical); tool-gated e2e `--ignored` vs real Verilator+Yosys (seed 42 `ok=true`); `anvil-mcp` stdio smoke (initialize → validate → `anvil://audit/log`) | passed |
| `2026-06-15` | `AGENT-INTROSPECTION-MCP.5.3` | `cargo fmt/check/clippy -D warnings`; `cargo test --lib downstream::` (20/20 + 1 gated, synthetic-oracle shrink proofs) + `mcp::` (18/18); `cargo test --test snapshots` (6/6 byte-identical); tool-gated e2e `--ignored` vs real Verilator 5.046 + Yosys 0.64 (seed 42 `reproduced_initial=false`); `anvil-mcp` stdio smoke (initialize → minimize → `anvil://audit/log`) | passed |
| `2026-06-15` | `AGENT-INTROSPECTION-MCP.6` | `cargo fmt --all --check`; `cargo check --all-targets`; `cargo clippy --all-targets -- -D warnings` (factored the renderer fn-pointer into the `PromptRender` type alias for `type_complexity`); `cargo test --lib mcp::` (24/24, +6 prompt tests); `cargo test --lib` (370/370, 2 gated); `cargo test --test snapshots` (6/6 byte-identical); `anvil-mcp` stdio smoke (initialize advertises `prompts`; `prompts/list` lists the 5; `prompts/get` renders + substitutes args; required-arg → `-32602`) | passed |
| `2026-06-15` | `AGENT-INTROSPECTION-MCP.7` | `mdbook build book` (clean); `cargo test --test book_examples` (3/3 — runnable `cargo run --release -- --seed 42 --introspect` block proven; two MCP-setup blocks skip-sentinelled with reasons); `bash scripts/check_memory_architecture.sh`; `bash knowledge-map/scripts/check_knowledge_map.sh`; `git diff --check`. Pure-docs leaf — no code touched, snapshots remain 6/6. | passed |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `AGENT-INTROSPECTION-MCP.1` | `AGENT-INTROSPECTION-MCP.1 - design + decision record 0004` | Commit `9ac5ef3`; opens the tree. |
| `AGENT-INTROSPECTION-MCP.2` | `AGENT-INTROSPECTION-MCP.2 - introspection schema spec (docs)` | Commit `defc196`; lands `docs/AGENT_INTROSPECTION_SCHEMA.md`. |
| `AGENT-INTROSPECTION-MCP.3` | `AGENT-INTROSPECTION-MCP.3 - introspection emission surface` | Commit `aec51e2`; lands `src/introspect/` + `--introspect`. |
| `AGENT-INTROSPECTION-MCP.4` | `AGENT-INTROSPECTION-MCP.4 - read-only MCP server` | Commit `5db5ebc`; lands `src/mcp/` + `anvil-mcp` bin. |
| `AGENT-INTROSPECTION-MCP.5.1` | `AGENT-INTROSPECTION-MCP.5.1 - shared downstream-tool invocation surface` | Commit `64f0bbe`; lands `src/downstream/`, rewires `tool_matrix`. |
| `AGENT-INTROSPECTION-MCP.5.2` | `AGENT-INTROSPECTION-MCP.5.2 - controlled validate tool` | Commit `65db6c3`; lands `downstream::validate` + MCP `validate` tool + `anvil://audit/log`. |
| `AGENT-INTROSPECTION-MCP.5.3` | `AGENT-INTROSPECTION-MCP.5.3 - controlled minimize tool` | Commit `381ec01`; lands `downstream::minimize` + MCP `minimize` tool; closes the `.5` container. |
| `AGENT-INTROSPECTION-MCP.6` | `AGENT-INTROSPECTION-MCP.6 - agent-workflow prompts` | Commit `b6f02ea`; lands the five MCP prompts (`prompts/list`/`prompts/get`) in `src/mcp/`. |
| `AGENT-INTROSPECTION-MCP.7` | `AGENT-INTROSPECTION-MCP.7 - book + USER_GUIDE + README closeout` | Pending hash; lands `book/src/agent-mcp.md` + USER_GUIDE/README sync; **closes the tree**. |

## Changelog

- `2026-06-14`: Created the tree; landed `.1` design + decision record 0004;
  frontier advanced to `.2` (schema spec).
- `2026-06-14`: Landed `.2` — `docs/AGENT_INTROSPECTION_SCHEMA.md` (versioned
  introspection schema, derived strictly from existing
  metrics/manifest/config/coverage; zero new computed truth; versioning policy
  with `schema_version = "1.0"`). Frontier is now design-complete; `.3` (first
  code leaf) is parked on owner acceptance of the `.1`/`.2` design.
- `2026-06-14`: Owner accepted the design; landed `.3` — `src/introspect/`
  emission surface + default-off `--introspect` CLI flag (DUT byte-identical,
  snapshots 6/6, 6 lib tests). Frontier advanced to `.4` (read-only MCP
  server).
- `2026-06-14`: Landed `.4` — `src/mcp/` read-only MCP server + `anvil-mcp`
  bin (stdio JSON-RPC; generate/introspect/dump_config tools + resources over
  a content-addressed cache; no external-tool exec; 12 lib tests; DUT
  byte-identical). Frontier advanced to `.5` (controlled validate/minimize).
- `2026-06-14`: Split `.5` into `.5.1`/`.5.2`/`.5.3` and landed `.5.1` — the
  hardened downstream-tool invocation surface moved from
  `src/bin/tool_matrix.rs` into the new library module `src/downstream/mod.rs`
  (`verilator --lint-only` / `yosys synth` / `iverilog -g2012` acceptance
  command lines, warning-as-failure detection, `ToolInvocation`, `YosysMode`,
  `yosys_mode_slug`, double-quote escapers; 7 lib tests). `tool_matrix` rewired
  to `use anvil::downstream::{…}`; behavior-preserving (matrix tool tests pass,
  snapshots 6/6 byte-identical). Frontier advanced to `.5.2` (controlled
  validate tool).
- `2026-06-14`: Landed `.5.2` — the controlled `validate` tool.
  `downstream::validate` (sandboxed per-run temp dir + `AcceptanceTool`
  allow-list + `MemGuard` decline-before-spawn + `ValidateReport`) and the MCP
  `validate` tool + `anvil://audit/log` resource; `introspect::content_run_id`
  made `pub` for the shared content address; `MemGuard::from_limits` added.
  12 downstream + 15 mcp lib tests, tool-gated e2e clean vs real
  Verilator+Yosys, `anvil-mcp` stdio smoke clean, snapshots 6/6 byte-identical.
  Frontier advanced to `.5.3` (minimize).
- `2026-06-15`: Landed `.5.3` — the controlled `minimize` delta-debugger,
  closing the `.5` container. `downstream::minimize` (`MinimizeOptions` /
  `MinimizeReport` / `KnobReduction`): a deterministic coordinate-descent that
  bisects integer size bounds toward their floors and drives optional-motif
  probabilities to `0.0`, to a fixpoint, using `.5.2`'s `validate` as a pure
  failure oracle; seed held fixed; hard-bounded by `max_oracle_calls`; a
  guard-decline unwinds cleanly. MCP `minimize` tool reuses the new shared
  `parse_validate_tools`/`parse_yosys_mode_arg` helpers and audit-logs each
  call. 20 downstream + 18 mcp lib tests (shrink logic proven via a synthetic
  predicate oracle), tool-gated e2e clean vs real Verilator 5.046 + Yosys 0.64
  (`reproduced_initial=false`), `anvil-mcp` stdio smoke clean, snapshots 6/6
  byte-identical. Frontier advanced to `.6` (agent-workflow prompts).
- `2026-06-15`: Landed `.6` — the five agent-workflow prompts as first-class
  **MCP prompts** in `src/mcp/` (`prompts` capability + `prompts/list` +
  `prompts/get` over a fixed `PROMPTS` registry; pure renderers that
  instantiate each workflow's ordered tool chain with sample-arg substitution;
  name/type/required-arg validation → clean `-32602` errors). Workflows:
  `find_downstream_bug`, `close_coverage_gap`, `minimize_reproducer`,
  `triage_tool_failures`, `explain_artifact`. 24 mcp lib tests (+6 prompt
  tests, incl. an end-to-end test that drives all five chains through the
  server portably via `tools: []`), full lib 370/370, snapshots 6/6
  byte-identical, `anvil-mcp` stdio smoke clean. Adds no new capability and
  computes no new truth (read-mostly doctrine preserved). Frontier advanced to
  `.7` (book + USER_GUIDE + README + CODEBASE_ANALYSIS closeout).
- `2026-06-15`: Landed `.7` and **closed the tree**. User-facing closeout: new
  mdBook chapter `book/src/agent-mcp.md` (Reference) documenting `--introspect`,
  the `anvil-mcp` tools/resources/prompts, the bug-hunting loop, and the
  guardrails, with real captured examples; `USER_GUIDE.md` "Agent introspection
  and the MCP server" section; `README.md` CLI-truth + key-paths sync. `mdbook
  build` clean; `book_examples` gate 3/3 (runnable `--introspect` block proven;
  two MCP-setup blocks skip-sentinelled). Pure-docs — snapshots remain 6/6. All
  seven leaves `.1`–`.7` done; `AGENT-INTROSPECTION-MCP` is `done`.
