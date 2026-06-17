# BUG-HUNT-ORCHESTRATION: a turnkey, MCP-driven downstream bug-hunt loop

## Metadata

- Tree ID: `BUG-HUNT-ORCHESTRATION`
- Status: `active`
- Roadmap lane: `Usability — turnkey bug-finder (north star, idea 1)`
- Created: `2026-06-17`
- Last updated: `2026-06-17`
- Owner: repo-local workflow

## Goal

Make ANVIL **directly usable as a downstream-tool bug-finder**, not a generator
the user has to wrap. Deliver a single turnkey loop — surfaced as a CLI
(`anvil hunt --tool <verilator|yosys|iverilog|…> --seeds N`) **and** as an MCP
tool — that: (1) fuzzes a chosen downstream tool across seeds and knob profiles,
(2) catches any reject / warning / cross-tool mismatch, (3) **auto-minimizes**
the failing artifact via the existing `minimize` coordinate-descent, and (4)
drops a self-contained **reproducer bundle** (seed + effective knobs + `.sv` +
`manifest.json` + expected-facts + the tool's log + a one-command repro script).
The pieces already exist but are separate (`src/bin/tool_matrix.rs`, the hardened
`src/downstream/` `validate`/`minimize` surface, `--diff-sim`, `src/introspect`);
this lane composes them into one bug-hunt orchestrator.

## Non-Goals

- No behavioural oracle / shadow simulator (decision `0004`, ROADMAP gap 4).
- No embedding/vendoring of the downstream tools — they stay external,
  allow-listed, sandboxed, RAM-guarded invocations.
- No new generator semantics; the hunt drives the existing valid-by-construction
  lanes. Default DUT output stays byte-identical.

## Acceptance Criteria

- A turnkey hunt loop (fuzz → detect → minimize → reproducer bundle) runs
  end-to-end against at least one real downstream tool and produces a
  one-command-reproducible bundle for an injected/known failure.
- **API-completeness gate (decision `0017`):** the hunt is fully driveable over
  MCP — an `hunt` (or equivalently-named) MCP tool sets every control
  (tool, seed range, knob profile, budgets, minimize on/off), invokes the run,
  and the result (per-seed verdicts, the minimized reproducer, the bundle
  `ResourceRef`) is queryable via the MCP/introspection API. The CLI is a thin
  shim over the same API.
- Reproducible + sandboxed: seeded, no wall-clock / no `thread_rng`; controlled
  tool calls go through the `src/downstream/` allow-list + RAM guard +
  `anvil://audit/log`.
- Default-off / DUT byte-identical; downstream-clean; documented in
  `book/src/agent-mcp.md` + USER_GUIDE + README; committed through `COMMIT.md`.

## Task Tree

- ID: `BUG-HUNT-ORCHESTRATION`
  Status: `active`
  Goal: `A turnkey, MCP-driven fuzz → detect → minimize → reproducer-bundle bug-hunt loop over the existing tool_matrix / downstream / diff-sim / introspect surfaces.`
  Children: `BUG-HUNT-ORCHESTRATION.1`

- ID: `BUG-HUNT-ORCHESTRATION.1`
  Status: `done`
  Goal: `Design/decision leaf (ADR, no code): pin the orchestration-loop shape (how it composes generate + the existing downstream validate/minimize + diff-sim + introspect), the reproducer-bundle format (seed + effective knobs + .sv + manifest + expected-facts + tool log + one-command repro), the MCP "hunt" tool input/result schema + the CLI shim over it (decision 0017 API-completeness), the detection policy (reject/warning/mismatch as a failure), and the sandbox/reproducibility discipline (decision 0004). Record as the next decision record + pre-split .2 (impl).`
  Acceptance: `A decision record (next sequential id) + a DEVELOPMENT_NOTES/tree entry pinning the loop, the bundle format, and the MCP+CLI surface; docs-only; INDEX + this tree + docs/TASK_TREE.md updated.`
  Result: `Done. Wrote docs/decisions/0018-bug-hunt-orchestration-loop.md (the design ADR; KM answers: front-matter; binds 0017 + 0004 + 0011; evidence grounded in the real src/downstream / src/diff_sim / src/mcp / src/introspect surfaces verified this session via a code-map recon agent). It pins: (loop) src/hunt/mod.rs exposing one hunt::run(&HuntRequest)->HuntReport that BOTH the MCP hunt tool and the anvil hunt CLI shim over — composing downstream::validate (whose first_tool_warning already unifies reject+warning into ok=false) + downstream::minimize (coordinate-descent oracle) + optional cross-sim mismatch + content-addressed run_id; (bundle) a directory <bundle_root>/<run_id>/ with repro.sv, knobs.json, introspection.json, manifest.json (non-DUT), tool-logs/, hunt-verdict.json, repro.sh; (MCP+CLI) the controlled hunt tool I/O schema + the first anvil subcommand anvil hunt, CLI --out a human convenience while the MCP sandbox stays caller-set (decision 0004), default path byte-identical; (detection) reject | warning | cross_sim_mismatch, classify-not-adjudicate; (discipline) seeded/sandboxed/allow-listed/RAM-guarded/audit-logged, default-off. Added the docs/decisions/INDEX.md row, a DEVELOPMENT_NOTES.md entry, and refreshed MEMORY.md + CHANGES.md + the docs/TASK_TREE.md frontier. Pre-split .2 into .2a..2e (below). Docs-only — no src/ touched ⇒ DUT byte-identical.`
  Verification: `Docs-only / no src/ ⇒ cargo check/clippy/fmt/test unaffected (code state = green .10b.3 baseline). bash scripts/check_memory_architecture.sh OK; knowledge-map gen+check OK (new 0018 card folded in). DUT byte-identical.`
  Commit: `this BUG-HUNT-ORCHESTRATION.1 commit`

- ID: `BUG-HUNT-ORCHESTRATION.2`
  Status: `pending`
  Goal: `Implement the .1 design: the hunt orchestrator + the MCP hunt tool + the CLI shim + the reproducer-bundle emitter + proofs + a real-tool end-to-end gate + book/USER_GUIDE/README/KM. Default-off / DUT byte-identical. Pre-split at .1 into .2a..2e (below).`
  Acceptance: `All of .2a..2e done; hunt loop runs end-to-end against a real downstream tool and drops a one-command-reproducible bundle; decision-0017 API-completeness gate met (hunt MCP-invocable + results queryable + CLI a shim); snapshots 6/6 + book-examples 3/3 unchanged; downstream-clean; documented; committed per COMMIT.md.`
  Verification: `pending`
  Commit: `pending`
  Children: `BUG-HUNT-ORCHESTRATION.2a, .2b (.2b.1/.2b.2), .2c, .2d, .2e`

- ID: `BUG-HUNT-ORCHESTRATION.2a`
  Status: `done`
  Goal: `Pure refactor: extract the tool_matrix diff-sim run+compare into a reusable diff_sim::run_agreement(...) library entry (the DIFFERENTIAL-SIMULATION.3b.1 extract-then-reuse precedent) so the hunt loop (and ACCEPTANCE-DIVERGENCE-HUNTING) detect cross-sim mismatch through a hardened surface. Byte-identical tool_matrix behaviour. Orderable first; the first hunt cut may ship reject/warning-only and fold this in next.`
  Acceptance: `diff_sim::run_agreement(work_dir, top, sv_text, n_vectors) -> DiffSimReport (+ the moved DiffSimReport / DutPort / parse_dut_ports / emit_testbench_for_ports) lives in src/diff_sim/mod.rs and is reusable; tool_matrix's run_diff_sim_for_module is a thin wrapper; emitted tb.sv + serialized DiffSimReport byte-identical (tool_matrix_report.json schema unchanged); cargo check/test/clippy/fmt green; snapshots 6/6 byte-identical; no new public-API regression.`
  Result: `Done. Moved into src/diff_sim/mod.rs (made pub): the DiffSimReport struct (serde shape unchanged), DutPort, parse_dut_ports (the strict-subset SV port parser), emit_testbench_for_ports (the SV-text-driven testbench), push_display_for_ports, and a NEW pub fn run_agreement(work_dir, top_name, sv_text, n_vectors) -> DiffSimReport containing the verbatim run+compare pipeline (tools_present → parse_dut_ports → create work_dir → write dut.sv/tb.sv → run_iverilog/run_verilator → normalize_trace + byte-compare; friendly no-op when a simulator is absent). src/bin/tool_matrix.rs now imports DiffSimReport from the library and reduces run_diff_sim_for_module to a 2-line wrapper computing dir = scenario_dir.join("<stem>-diff-sim") and delegating to run_agreement(.., 8). Moved the two pure-unit tests (parse_dut_ports_recognises_anvil_emitter_shape, emit_testbench_for_ports_renders_combinational_and_sequential_shapes) into the diff_sim test module + added run_agreement_is_a_friendly_no_op_without_tools; kept the tool_matrix #[ignore] e2e gate (over the wrapper) + the coverage-fact test (over the imported type). The IR-driven emit_testbench stays canonical; unifying the two testbench emitters is a deferred cleanup (.2a is a byte-identical move, not a merge).`
  Verification: `cargo check --all-targets OK; cargo fmt --all --check OK; cargo clippy --all-targets -- -D warnings OK; cargo test green — lib 502→505 (the 2 moved unit tests + the new friendly-no-op), tool_matrix 73→71 passed + the e2e gate ignored, tests/diff_sim.rs 2 passed/2 tool-gated, tests/snapshots.rs 6/6 byte-identical (DUT output unchanged ⇒ the refactor is provably byte-identical). No .snap change.`
  Commit: `this BUG-HUNT-ORCHESTRATION.2a commit`

- ID: `BUG-HUNT-ORCHESTRATION.2b`
  Status: `active`
  Goal: `The src/hunt/ library core: HuntRequest/HuntReport/HuntFailure types + hunt::run(&HuntRequest)->HuntReport composing downstream::validate/minimize (+ optional diff-sim via .2a) over a deterministic seed sweep + the reproducer-bundle emitter; cargo-portable proofs. No CLI/MCP yet. Default-off / DUT byte-identical. Pre-split at pick into .2b.1 (loop core + types, reject/warning detection) + .2b.2 (cross-sim detection via run_agreement + the reproducer-bundle emitter).`
  Acceptance: `Both .2b.1 + .2b.2 done; hunt::run composes validate/minimize/run_agreement; the reproducer bundle is emitted as a directory; cargo-portable proofs; default-off / DUT byte-identical (snapshots 6/6).`
  Verification: `pending`
  Commit: `pending`
  Children: `BUG-HUNT-ORCHESTRATION.2b.1, .2b.2`

- ID: `BUG-HUNT-ORCHESTRATION.2b.1`
  Status: `done`
  Goal: `The src/hunt/ library core (loop + types) with reject/warning detection: HuntRequest/HuntReport/HuntVerdict/HuntFailure/HuntMinimized/HuntSummary + hunt::run(&HuntRequest)->HuntReport composing downstream::validate (detection = !ValidateReport.ok, which already unifies reject+warning) + optional downstream::minimize over a deterministic seed sweep. Every report field SCHEMA-DERIVED. No cross-sim, no on-disk bundle, no CLI/MCP yet. Cargo-portable proofs. Default-off / DUT byte-identical.`
  Acceptance: `src/hunt/mod.rs + lib.rs pub mod hunt; hunt::run sweeps base_seed..base_seed+seeds, validates each, classifies reject/warning, optionally minimizes, returns HuntReport{verdicts,failures,summary}; cargo-portable proofs (no real tools) green; cargo check/test/clippy/fmt green; snapshots 6/6 byte-identical (hunt wired into no generate/emit path).`
  Result: `Done. New src/hunt/mod.rs: HuntRequest (base_seed/seeds/config/validate:ValidateOptions/minimize/max_oracle_calls), HuntVerdict, HuntFailure, HuntMinimized (projected from MinimizeReport), HuntSummary, HuntReport — all serde, every field a SCHEMA-DERIVED projection of ValidateReport/MinimizeReport/ToolInvocation (decision 0017's queryable gate; no new computed truth, no shadow oracle). hunt::run composes downstream::validate per seed (declined→declined verdict; ok→clean verdict; else a finding), classifies reject (non-zero exit) vs warning (clean exit + !success), and—when minimize—composes downstream::minimize (oracle = the same ValidateOptions). Registered pub mod hunt in lib.rs. 5 cargo-portable proofs: no-tool smoke is all-clean + seeds swept consecutively, reproducible run_ids, classify_detection warning-vs-reject, first_failing_tool, HuntReport serde round-trip (+ skip_serializing_if keeps absent fields out of the wire form). The library core only — cross-sim detection (diff_sim::run_agreement from .2a) + the reproducer-bundle emitter are .2b.2; the MCP tool is .2c, the CLI is .2d.`
  Verification: `cargo check --all-targets OK; cargo fmt --all --check OK; cargo clippy --all-targets -- -D warnings OK; focused cargo test --lib hunt:: = 5/5; full cargo test green incl. tests/snapshots.rs 6/6 byte-identical (hunt is default-off, wired into no generate/emit path ⇒ DUT byte-identical).`
  Commit: `this BUG-HUNT-ORCHESTRATION.2b.1 commit`

- ID: `BUG-HUNT-ORCHESTRATION.2b.2`
  Status: `pending`
  Goal: `Fold the cross-simulator mismatch detector (anvil::diff_sim::run_agreement from .2a) into hunt::run as an optional detection axis (detection = "cross_sim_mismatch"), AND add the reproducer-bundle emitter: write <bundle_root>/<run_id>/ with repro.sv, knobs.json, introspection.json, manifest.json (non-DUT), tool-logs/, hunt-verdict.json, repro.sh. HuntFailure gains the bundle ref. Cargo-portable proofs (bundle emitted to a temp dir; cross-sim no-op when sims absent). Default-off / DUT byte-identical.`
  Acceptance: `pending (set when picked)`
  Verification: `pending`
  Commit: `pending`

- ID: `BUG-HUNT-ORCHESTRATION.2c`
  Status: `pending`
  Goal: `The MCP hunt controlled tool wired into src/mcp dispatcher: input schema, HuntReport result, failing-run artifact-cache population (so anvil://artifact/<run_id>/{sv,introspection,manifest} reads work), a top-level hunt audit record; introspection/MCP doc + schema note; proofs.`
  Acceptance: `pending (set when picked)`
  Verification: `pending`
  Commit: `pending`

- ID: `BUG-HUNT-ORCHESTRATION.2d`
  Status: `pending`
  Goal: `The anvil hunt CLI subcommand (ANVIL's first subcommand) as a thin shim over hunt::run, with --out as a human-only convenience; the byte-identical default-path guard (snapshots 6/6 + book-examples 3/3 unchanged); proofs.`
  Acceptance: `pending (set when picked)`
  Verification: `pending`
  Commit: `pending`

- ID: `BUG-HUNT-ORCHESTRATION.2e`
  Status: `pending`
  Goal: `A real-tool end-to-end gate (#[ignore], tool-gated) that runs a hunt against Verilator/Yosys and produces a one-command-reproducible bundle for an injected/known failure; book/src/agent-mcp.md + USER_GUIDE + README + a KM card; close .2 and the tree.`
  Acceptance: `pending (set when picked)`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `BUG-HUNT-ORCHESTRATION.2b.2` | `pending` | Fold cross-sim mismatch detection (`diff_sim::run_agreement`) into `hunt::run` + add the reproducer-bundle emitter (the directory). Completes the `.2b` engine. |
| 2 | `BUG-HUNT-ORCHESTRATION.2c` | `pending` | The MCP `hunt` controlled tool (decision `0017` invocable + queryable). |
| 3 | `BUG-HUNT-ORCHESTRATION.2d` | `pending` | The `anvil hunt` CLI shim + the byte-identical default-path guard. |
| 4 | `BUG-HUNT-ORCHESTRATION.2e` | `pending` | The real-tool end-to-end gate + book/USER_GUIDE/README/KM; closes the tree. |
| — | `BUG-HUNT-ORCHESTRATION.2a` | `done` | Extracted the diff-sim run+compare into `diff_sim::run_agreement` (byte-identical; snapshots 6/6). |
| — | `BUG-HUNT-ORCHESTRATION.2b.1` | `done` | The `src/hunt/` library core (`hunt::run` + types, reject/warning detection); 5 cargo-portable proofs; snapshots 6/6. |

## Decisions

- `2026-06-17`: Registered as an owner-directed usability lane (idea 1). Binds
  decision [`0017`](../decisions/0017-api-first-everything-mcp-accessible.md)
  (API-first: the hunt must be fully MCP-driveable + its results queryable). The
  first leaf is a design/decision ADR per the project's design-first cadence; no
  code before `.1` lands.
- `2026-06-17` (`.1` done): Recorded decision
  [`0018`](../decisions/0018-bug-hunt-orchestration-loop.md). The loop is a
  **thin orchestrator, not a new engine** — `hunt::run` composes the existing
  `downstream::validate`/`minimize` (+ optional extracted diff-sim) and adds no
  detector and no minimizer of its own. Reproducer bundle = a **directory**
  (matches `--out`/`tool_matrix`; inspectable; agent-fetchable as resources).
  `hunt` is a controlled MCP tool **and** the first `anvil` subcommand, both
  shims over `hunt::run`. Detection = reject | warning | cross_sim_mismatch
  (`validate` already unifies reject+warning into `ok=false`). Sandbox path is
  caller-set, never agent-supplied (decision `0004`). Pre-split `.2` into
  `.2a`…`.2e`.

## Open Questions

- ~~Bundle format: directory vs single archive.~~ **Resolved at `.1`**: a
  directory (`<bundle_root>/<run_id>/`) — matches the `--out`/`tool_matrix`
  convention, stays inspectable/diffable/git-attachable, and lets an agent fetch
  parts as `anvil://…` resources without unpacking. An archive view is a trivial
  later add-on if asked.
- Knob-profile source: reuse `KNOB-ERGONOMICS-AND-PRESETS` presets once that lane
  lands, vs. an interim inline profile set. **Partially resolved at `.1`**: the
  hunt's `config` input *is* the knob profile (a full `Config`); curated
  `--profile` names are deferred to `KNOB-ERGONOMICS-AND-PRESETS` and plug in
  without reopening this lane. *(Cross-lane; not a `.2` blocker.)*

## Blockers

- None. (Synergistic with `ACCEPTANCE-DIVERGENCE-HUNTING`,
  `DOWNSTREAM-ADAPTER-EXPANSION`, and `KNOB-ERGONOMICS-AND-PRESETS`, but not
  blocked by them.)

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-06-17` | `BUG-HUNT-ORCHESTRATION` | `tree registered (docs-only); no code` | `registered` |
| `2026-06-17` | `BUG-HUNT-ORCHESTRATION.1` | `decision 0018 + INDEX + DEVELOPMENT_NOTES + MEMORY + CHANGES + docs/TASK_TREE row; check_memory_architecture OK; KM gen+check OK; docs-only (no src/) ⇒ DUT byte-identical` | `done` |
| `2026-06-17` | `BUG-HUNT-ORCHESTRATION.2a` | `cargo check/test/clippy/fmt green; lib 502→505, tool_matrix 73→71+ignored, tests/diff_sim.rs 2 pass/2 gated, snapshots 6/6 byte-identical; run_agreement extracted; tool_matrix_report.json schema unchanged` | `done` |
| `2026-06-17` | `BUG-HUNT-ORCHESTRATION.2b.1` | `cargo check/clippy/fmt green; cargo test --lib hunt:: 5/5; full cargo test green incl. snapshots 6/6 byte-identical (hunt default-off, no generate/emit path)` | `done` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `BUG-HUNT-ORCHESTRATION` | `USABILITY-LANE-OWNERSHIP.1 — register 7 owner-directed usability/capability lanes + API-first decision 0017` | Tree registered (not yet started); frontier `.1` (design ADR) pending. |
| `BUG-HUNT-ORCHESTRATION.1` | `BUG-HUNT-ORCHESTRATION.1 — design ADR (decision 0018): turnkey fuzz→detect→minimize→bundle loop + MCP hunt tool + anvil hunt CLI` | Design/decision leaf (docs-only). Pins the loop/bundle/MCP+CLI/detection/sandbox; pre-splits `.2` into `.2a`…`.2e`. DUT byte-identical. |
| `BUG-HUNT-ORCHESTRATION.2a` | `BUG-HUNT-ORCHESTRATION.2a — extract diff-sim run+compare into reusable diff_sim::run_agreement` | Pure byte-identical refactor; `src/diff_sim/` now owns `run_agreement` + `DiffSimReport` + the SV-text testbench; `tool_matrix` wraps it. First impl leaf of `.2`. |
| `BUG-HUNT-ORCHESTRATION.2b.1` | `BUG-HUNT-ORCHESTRATION.2b.1 — src/hunt/ library core (hunt::run loop + types, reject/warning detection)` | New `src/hunt/mod.rs` + `pub mod hunt`; composes `downstream::validate`/`minimize`; SCHEMA-DERIVED `HuntReport`; 5 cargo-portable proofs; default-off / DUT byte-identical. Cross-sim + bundle = `.2b.2`. |

## Changelog

- `2026-06-17`: Created task tree (registration via `USABILITY-LANE-OWNERSHIP.1`).
- `2026-06-17`: `.1` done — recorded decision `0018` (the bug-hunt loop design);
  pre-split `.2` into `.2a` (diff-sim extract), `.2b` (`src/hunt/` core), `.2c`
  (MCP `hunt` tool), `.2d` (`anvil hunt` CLI), `.2e` (real-tool gate + docs).
  Frontier advanced to `.2a`. Docs-only / DUT byte-identical.
- `2026-06-17`: `.2a` done — extracted the diff-sim run+compare into
  `anvil::diff_sim::run_agreement` (a byte-identical move; the bug-hunt loop and
  `ACCEPTANCE-DIVERGENCE-HUNTING` now reuse it). `tool_matrix`'s
  `run_diff_sim_for_module` is a thin wrapper; snapshots 6/6 byte-identical.
  Frontier advanced to `.2b` (the `src/hunt/` core).
- `2026-06-17`: pre-split `.2b` into `.2b.1` (loop core + types,
  reject/warning detection) + `.2b.2` (cross-sim detection + reproducer-bundle
  emitter). `.2b.1` done — new `src/hunt/mod.rs` (`hunt::run` + the SCHEMA-DERIVED
  `HuntRequest`/`HuntReport`/`HuntFailure`/`HuntMinimized`/`HuntSummary`)
  composing `downstream::validate`/`minimize`; 5 cargo-portable proofs;
  default-off / DUT byte-identical (snapshots 6/6). Frontier advanced to `.2b.2`.
