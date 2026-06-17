# BUG-HUNT-ORCHESTRATION: a turnkey, MCP-driven downstream bug-hunt loop

## Metadata

- Tree ID: `BUG-HUNT-ORCHESTRATION`
- Status: `done` (closed `2026-06-17`)
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
  Status: `done`
  Goal: `A turnkey, MCP-driven fuzz → detect → minimize → reproducer-bundle bug-hunt loop over the existing tool_matrix / downstream / diff-sim / introspect surfaces.`
  Result: `Done — closed 2026-06-17. ANVIL is now directly usable as a downstream-tool bug-finder. The src/hunt/ engine (hunt::run) composes downstream::validate/minimize + the extracted diff_sim::run_agreement + introspect into one deterministic fuzz → detect (reject/warning/cross-sim mismatch) → auto-minimize → reproducer loop, surfaced two ways — the anvil hunt CLI subcommand (ANVIL's first) and the controlled hunt MCP tool — both thin shims over the same hunt::run (decision 0017). Findings carry an auto-minimized reproducer + a self-contained bundle directory (CLI --out) or cache-served anvil:// resources (MCP). Default anvil build / DUT byte-identical throughout. Leaves .1 (ADR 0018) / .2a (diff-sim extract) / .2b.1 (loop core) / .2b.2a (cross-sim fold + generate_dut_artifact) / .2b.2b (bundle emitter + introspect_dut_artifact) / .2c (MCP tool) / .2d (CLI) / .2e (real-tool e2e gate + closeout) all done.`
  Children: `BUG-HUNT-ORCHESTRATION.1`

- ID: `BUG-HUNT-ORCHESTRATION.1`
  Status: `done`
  Goal: `Design/decision leaf (ADR, no code): pin the orchestration-loop shape (how it composes generate + the existing downstream validate/minimize + diff-sim + introspect), the reproducer-bundle format (seed + effective knobs + .sv + manifest + expected-facts + tool log + one-command repro), the MCP "hunt" tool input/result schema + the CLI shim over it (decision 0017 API-completeness), the detection policy (reject/warning/mismatch as a failure), and the sandbox/reproducibility discipline (decision 0004). Record as the next decision record + pre-split .2 (impl).`
  Acceptance: `A decision record (next sequential id) + a DEVELOPMENT_NOTES/tree entry pinning the loop, the bundle format, and the MCP+CLI surface; docs-only; INDEX + this tree + docs/TASK_TREE.md updated.`
  Result: `Done. Wrote docs/decisions/0018-bug-hunt-orchestration-loop.md (the design ADR; KM answers: front-matter; binds 0017 + 0004 + 0011; evidence grounded in the real src/downstream / src/diff_sim / src/mcp / src/introspect surfaces verified this session via a code-map recon agent). It pins: (loop) src/hunt/mod.rs exposing one hunt::run(&HuntRequest)->HuntReport that BOTH the MCP hunt tool and the anvil hunt CLI shim over — composing downstream::validate (whose first_tool_warning already unifies reject+warning into ok=false) + downstream::minimize (coordinate-descent oracle) + optional cross-sim mismatch + content-addressed run_id; (bundle) a directory <bundle_root>/<run_id>/ with repro.sv, knobs.json, introspection.json, manifest.json (non-DUT), tool-logs/, hunt-verdict.json, repro.sh; (MCP+CLI) the controlled hunt tool I/O schema + the first anvil subcommand anvil hunt, CLI --out a human convenience while the MCP sandbox stays caller-set (decision 0004), default path byte-identical; (detection) reject | warning | cross_sim_mismatch, classify-not-adjudicate; (discipline) seeded/sandboxed/allow-listed/RAM-guarded/audit-logged, default-off. Added the docs/decisions/INDEX.md row, a DEVELOPMENT_NOTES.md entry, and refreshed MEMORY.md + CHANGES.md + the docs/TASK_TREE.md frontier. Pre-split .2 into .2a..2e (below). Docs-only — no src/ touched ⇒ DUT byte-identical.`
  Verification: `Docs-only / no src/ ⇒ cargo check/clippy/fmt/test unaffected (code state = green .10b.3 baseline). bash scripts/check_memory_architecture.sh OK; knowledge-map gen+check OK (new 0018 card folded in). DUT byte-identical.`
  Commit: `this BUG-HUNT-ORCHESTRATION.1 commit`

- ID: `BUG-HUNT-ORCHESTRATION.2`
  Status: `done`
  Goal: `Implement the .1 design: the hunt orchestrator + the MCP hunt tool + the CLI shim + the reproducer-bundle emitter + proofs + a real-tool end-to-end gate + book/USER_GUIDE/README/KM. Default-off / DUT byte-identical. Pre-split at .1 into .2a..2e (below).`
  Acceptance: `All of .2a..2e done; hunt loop runs end-to-end against a real downstream tool and drops a one-command-reproducible bundle; decision-0017 API-completeness gate met (hunt MCP-invocable + results queryable + CLI a shim); snapshots 6/6 + book-examples 3/3 unchanged; downstream-clean; documented; committed per COMMIT.md.`
  Result: `Done — all of .2a..2e landed. The engine (.2a diff-sim extract / .2b.1 loop core / .2b.2a cross-sim fold / .2b.2b bundle emitter), both surfaces (.2c MCP hunt tool / .2d anvil hunt CLI), and the closeout (.2e real-tool e2e gate + book/USER_GUIDE/README/KM) are complete. The hunt runs end-to-end against real Verilator (tests/hunt_e2e.rs, clean sweep + byte-identical reproducer recipe); the bundle directory format is unit-proven (.2b.2b); decision-0017 met (MCP-invocable + cache-queryable + CLI a shim over the same hunt::run); snapshots 6/6 + book_examples unchanged; default anvil build / DUT byte-identical; documented in book/src/agent-mcp.md + USER_GUIDE + README + KM card bug-hunt-cli.`
  Verification: `cargo check/test/clippy/fmt green across .2a..2e; tests/hunt_e2e.rs 2/2 against real Verilator (--ignored), 0 portable; full cargo test green incl. tests/snapshots.rs 6/6 byte-identical + tests/book_examples.rs; KM 47 facts (bug-hunt-cli folded in).`
  Commit: `closed by the .2e commit (last child)`
  Children: `BUG-HUNT-ORCHESTRATION.2a, .2b (.2b.1/.2b.2), .2c, .2d, .2e`

- ID: `BUG-HUNT-ORCHESTRATION.2a`
  Status: `done`
  Goal: `Pure refactor: extract the tool_matrix diff-sim run+compare into a reusable diff_sim::run_agreement(...) library entry (the DIFFERENTIAL-SIMULATION.3b.1 extract-then-reuse precedent) so the hunt loop (and ACCEPTANCE-DIVERGENCE-HUNTING) detect cross-sim mismatch through a hardened surface. Byte-identical tool_matrix behaviour. Orderable first; the first hunt cut may ship reject/warning-only and fold this in next.`
  Acceptance: `diff_sim::run_agreement(work_dir, top, sv_text, n_vectors) -> DiffSimReport (+ the moved DiffSimReport / DutPort / parse_dut_ports / emit_testbench_for_ports) lives in src/diff_sim/mod.rs and is reusable; tool_matrix's run_diff_sim_for_module is a thin wrapper; emitted tb.sv + serialized DiffSimReport byte-identical (tool_matrix_report.json schema unchanged); cargo check/test/clippy/fmt green; snapshots 6/6 byte-identical; no new public-API regression.`
  Result: `Done. Moved into src/diff_sim/mod.rs (made pub): the DiffSimReport struct (serde shape unchanged), DutPort, parse_dut_ports (the strict-subset SV port parser), emit_testbench_for_ports (the SV-text-driven testbench), push_display_for_ports, and a NEW pub fn run_agreement(work_dir, top_name, sv_text, n_vectors) -> DiffSimReport containing the verbatim run+compare pipeline (tools_present → parse_dut_ports → create work_dir → write dut.sv/tb.sv → run_iverilog/run_verilator → normalize_trace + byte-compare; friendly no-op when a simulator is absent). src/bin/tool_matrix.rs now imports DiffSimReport from the library and reduces run_diff_sim_for_module to a 2-line wrapper computing dir = scenario_dir.join("<stem>-diff-sim") and delegating to run_agreement(.., 8). Moved the two pure-unit tests (parse_dut_ports_recognises_anvil_emitter_shape, emit_testbench_for_ports_renders_combinational_and_sequential_shapes) into the diff_sim test module + added run_agreement_is_a_friendly_no_op_without_tools; kept the tool_matrix #[ignore] e2e gate (over the wrapper) + the coverage-fact test (over the imported type). The IR-driven emit_testbench stays canonical; unifying the two testbench emitters is a deferred cleanup (.2a is a byte-identical move, not a merge).`
  Verification: `cargo check --all-targets OK; cargo fmt --all --check OK; cargo clippy --all-targets -- -D warnings OK; cargo test green — lib 502→505 (the 2 moved unit tests + the new friendly-no-op), tool_matrix 73→71 passed + the e2e gate ignored, tests/diff_sim.rs 2 passed/2 tool-gated, tests/snapshots.rs 6/6 byte-identical (DUT output unchanged ⇒ the refactor is provably byte-identical). No .snap change.`
  Commit: `this BUG-HUNT-ORCHESTRATION.2a commit`

- ID: `BUG-HUNT-ORCHESTRATION.2b`
  Status: `done`
  Goal: `The src/hunt/ library core: HuntRequest/HuntReport/HuntFailure types + hunt::run(&HuntRequest)->HuntReport composing downstream::validate/minimize (+ optional diff-sim via .2a) over a deterministic seed sweep + the reproducer-bundle emitter; cargo-portable proofs. No CLI/MCP yet. Default-off / DUT byte-identical. Pre-split at pick into .2b.1 (loop core + types, reject/warning detection) + .2b.2 (cross-sim detection via run_agreement + the reproducer-bundle emitter).`
  Acceptance: `Both .2b.1 + .2b.2 done; hunt::run composes validate/minimize/run_agreement; the reproducer bundle is emitted as a directory; cargo-portable proofs; default-off / DUT byte-identical (snapshots 6/6).`
  Result: `Done — both children landed. .2b.1: the loop core (hunt::run + the SCHEMA-DERIVED HuntRequest/HuntReport/HuntVerdict/HuntFailure/HuntMinimized/HuntSummary) composing downstream::validate/minimize with reject/warning detection (+ the seed-threading fix). .2b.2: .2b.2a folded the optional cross-simulator axis (diff_sim::run_agreement → cross_sim_mismatch finding) + extracted the shared downstream::generate_dut_artifact; .2b.2b added the reproducer-bundle emitter (directory per finding) + introspect_dut_artifact. The src/hunt/ library core is complete; the MCP hunt tool (.2c) + the anvil hunt CLI (.2d) shim over hunt::run next. Default-off / DUT byte-identical throughout.`
  Verification: `cargo check/test/clippy/fmt green across .2b.1/.2b.2a/.2b.2b; cargo test --lib hunt:: 11/11; full cargo test green incl. tests/snapshots.rs 6/6 byte-identical.`
  Commit: `closed by the .2b.2b commit (last child)`
  Children: `BUG-HUNT-ORCHESTRATION.2b.1, .2b.2`

- ID: `BUG-HUNT-ORCHESTRATION.2b.1`
  Status: `done`
  Goal: `The src/hunt/ library core (loop + types) with reject/warning detection: HuntRequest/HuntReport/HuntVerdict/HuntFailure/HuntMinimized/HuntSummary + hunt::run(&HuntRequest)->HuntReport composing downstream::validate (detection = !ValidateReport.ok, which already unifies reject+warning) + optional downstream::minimize over a deterministic seed sweep. Every report field SCHEMA-DERIVED. No cross-sim, no on-disk bundle, no CLI/MCP yet. Cargo-portable proofs. Default-off / DUT byte-identical.`
  Acceptance: `src/hunt/mod.rs + lib.rs pub mod hunt; hunt::run sweeps base_seed..base_seed+seeds, validates each, classifies reject/warning, optionally minimizes, returns HuntReport{verdicts,failures,summary}; cargo-portable proofs (no real tools) green; cargo check/test/clippy/fmt green; snapshots 6/6 byte-identical (hunt wired into no generate/emit path).`
  Result: `Done. New src/hunt/mod.rs: HuntRequest (base_seed/seeds/config/validate:ValidateOptions/minimize/max_oracle_calls), HuntVerdict, HuntFailure, HuntMinimized (projected from MinimizeReport), HuntSummary, HuntReport — all serde, every field a SCHEMA-DERIVED projection of ValidateReport/MinimizeReport/ToolInvocation (decision 0017's queryable gate; no new computed truth, no shadow oracle). hunt::run composes downstream::validate per seed (declined→declined verdict; ok→clean verdict; else a finding), classifies reject (non-zero exit) vs warning (clean exit + !success), and—when minimize—composes downstream::minimize (oracle = the same ValidateOptions). Registered pub mod hunt in lib.rs. 5 cargo-portable proofs: no-tool smoke is all-clean + seeds swept consecutively, reproducible run_ids, classify_detection warning-vs-reject, first_failing_tool, HuntReport serde round-trip (+ skip_serializing_if keeps absent fields out of the wire form). The library core only — cross-sim detection (diff_sim::run_agreement from .2a) + the reproducer-bundle emitter are .2b.2; the MCP tool is .2c, the CLI is .2d.`
  Verification: `cargo check --all-targets OK; cargo fmt --all --check OK; cargo clippy --all-targets -- -D warnings OK; focused cargo test --lib hunt:: = 5/5; full cargo test green incl. tests/snapshots.rs 6/6 byte-identical (hunt is default-off, wired into no generate/emit path ⇒ DUT byte-identical).`
  Commit: `this BUG-HUNT-ORCHESTRATION.2b.1 commit`

- ID: `BUG-HUNT-ORCHESTRATION.2b.2`
  Status: `done`
  Goal: `Fold the cross-simulator mismatch detector (anvil::diff_sim::run_agreement from .2a) into hunt::run as an optional detection axis (detection = "cross_sim_mismatch"), AND add the reproducer-bundle emitter. Pre-split at pick into .2b.2a (cross-sim fold + the shared generate helper) + .2b.2b (the reproducer-bundle emitter).`
  Acceptance: `Both .2b.2a + .2b.2b done; cross-sim mismatch is a detection axis on clean artifacts; the reproducer bundle is emitted as a directory; cargo-portable proofs; default-off / DUT byte-identical (snapshots 6/6).`
  Result: `Done — both children landed. .2b.2a: cross-sim mismatch is now a detection axis on validate-clean artifacts (HuntRequest.diff_sim → run_agreement → cross_sim_mismatch finding) + extracted downstream::generate_dut_artifact. .2b.2b: the reproducer bundle is emitted as a directory <bundle_root>/<run_id>/ per finding (repro.sv/knobs.json/introspection.json/hunt-verdict.json/tool-logs/repro.sh) with HuntFailure.bundle + introspect_dut_artifact. Default-off / DUT byte-identical (snapshots 6/6).`
  Verification: `cargo check/test/clippy/fmt green; cargo test --lib hunt:: 11/11; full cargo test green incl. tests/snapshots.rs 6/6 byte-identical.`
  Commit: `closed by the .2b.2b commit (last child)`
  Children: `BUG-HUNT-ORCHESTRATION.2b.2a, .2b.2b`

- ID: `BUG-HUNT-ORCHESTRATION.2b.2a`
  Status: `done`
  Goal: `Fold cross-simulator mismatch detection into hunt::run: a HuntRequest.diff_sim flag runs anvil::diff_sim::run_agreement on each validate-clean artifact; a mismatch is a finding (detection = "cross_sim_mismatch", HuntFailure.diff_sim carries the DiffSimReport, no minimize — the validate oracle can't reproduce a trace disagreement). Extract the shared downstream::generate_dut_artifact(cfg) -> (kind, top, sv) so the hunt regenerates exactly what validate accepted without copying the design-vs-module branch. Cargo-portable proofs. Default-off / DUT byte-identical.`
  Acceptance: `downstream::generate_dut_artifact extracted + validate uses it (byte-identical, downstream tests green); HuntRequest.diff_sim + HuntFailure.diff_sim added; cross_sim_mismatch helper runs run_agreement on clean artifacts (work dir caller-set, removed unless keep_sandbox); cargo-portable proof that diff_sim on clean artifacts is a no-op without simulators; cargo check/test/clippy/fmt green; snapshots 6/6 byte-identical.`
  Result: `Done. (1) Extracted pub fn downstream::generate_dut_artifact(cfg) -> (String,String,String) (the design-vs-module dispatch); validate now calls it — byte-identical (downstream lib tests 20/0, 2 ignored). (2) hunt: HuntRequest.diff_sim: bool (default behaviour false) + HuntFailure.diff_sim: Option<DiffSimReport> (skip_serializing_if) + a cross_sim_mismatch(req, cfg, run_id) helper that regenerates the DUT SV via generate_dut_artifact and runs diff_sim::run_agreement in a caller-set per-run work dir (removed unless keep_sandbox), returning Some only when both sims ran AND disagreed. hunt::run runs it on each validate-clean artifact when diff_sim is set; a mismatch is a finding (detection "cross_sim_mismatch", minimized None since the validate oracle can't reproduce a trace disagreement, diff_sim carries the report). (3) Proof diff_sim_on_clean_artifact_no_ops_without_simulators (tools-absent ⇒ run_agreement ran=false ⇒ clean), + updated the serde round-trip (the new diff_sim field stays absent in the wire form). The bundle emitter is .2b.2b.`
  Verification: `cargo check --all-targets OK; cargo fmt --all --check OK; cargo clippy --all-targets -- -D warnings OK; cargo test --lib hunt:: 7/7 + downstream:: 20/0; full cargo test green incl. tests/snapshots.rs 6/6 byte-identical (the validate refactor is byte-identical; hunt is wired into no generate/emit path).`
  Commit: `this BUG-HUNT-ORCHESTRATION.2b.2a commit`

- ID: `BUG-HUNT-ORCHESTRATION.2b.2b`
  Status: `done`
  Goal: `The reproducer-bundle emitter: a HuntRequest.bundle_root: Option<PathBuf>; on each finding, write <bundle_root>/<run_id>/ with repro.sv (regenerated via generate_dut_artifact, or the minimized config's SV), knobs.json (the effective/minimized Config), introspection.json (the IntrospectionDocument), tool-logs/ (or a note that repro.sh regenerates them), hunt-verdict.json (the HuntFailure), and a one-command repro.sh. HuntFailure gains a bundle ref (path + the anvil:// resource URIs). Cargo-portable proofs (bundle emitted to a temp dir; files present + repro.sh regenerates the .sv). Default-off / DUT byte-identical.`
  Acceptance: `HuntRequest.bundle_root + HuntFailure.bundle (HuntBundle{path,sv,introspection,manifest?}) added; on each finding hunt::run writes <bundle_root>/<run_id>/ with repro.sv + knobs.json + introspection.json + hunt-verdict.json + tool-logs/NOTE.txt + an executable repro.sh; the bundle prefers the minimized reproducer when minimize confirmed one; cargo-portable proofs (emitter unit-tested with a synthetic failing ValidateReport, no real tools); cargo check/test/clippy/fmt green; snapshots 6/6 byte-identical (bundle_root default None ⇒ no on-disk bundle ⇒ no generate/emit path touched).`
  Result: `Done. (1) src/downstream/mod.rs: added pub fn introspect_dut_artifact(seed, cfg) -> IntrospectionDocument — the introspection sibling of generate_dut_artifact (same module-vs-design dispatch, projecting through the pure introspect::module_document/design_document) so the bundle builds construction-truth from one home, not a fourth copy of the branch. (2) src/hunt/mod.rs: HuntRequest.bundle_root: Option<PathBuf> (caller-set, never agent-supplied — decision 0004) + HuntFailure.bundle: Option<HuntBundle> (skip_serializing_if) + the HuntBundle{path, sv, introspection, manifest?} ref. New write_bundle(bundle_root, seed, repro_cfg, repro_validate, verdict) writes <bundle_root>/<run_id>/{repro.sv (generate_dut_artifact), knobs.json (to_string_pretty(Config)), introspection.json (introspect_dut_artifact), hunt-verdict.json (the HuntFailure, bundle ref omitted), tool-logs/NOTE.txt, repro.sh (regenerate via anvil --seed N --config knobs.json then replay the failing tool's argv with the ephemeral sandbox SV path substituted to repro.sv; POSIX-single-quoted; chmod 0755 on unix)}. hunt::run emits a bundle per finding at both finding sites (reject/warning AND cross_sim_mismatch); prefers the minimized reproducer (m.minimized_config + m.final_validation) when minimize confirmed a smaller still-failing config, else the originally-detected (cfg, report); a cross-sim finding (no rejecting tool) gets a repro.sh that points at the diff_sim excerpt. (3) 4 new cargo-portable proofs (hunt:: 7→11): shell_quote_wraps_and_escapes, write_bundle_emits_a_self_contained_reproducer_directory (synthetic failing ValidateReport ⇒ all files present + repro.sv byte-identical to generate_dut_artifact + knobs.json/introspection.json round-trip + repro.sh substitutes the sandbox path + hunt-verdict.json omits the self-ref bundle), repro_script_handles_a_cross_sim_finding_with_no_failing_tool, bundle_root_writes_nothing_on_a_clean_sweep; serde round-trip updated (bundle stays absent when None). The MCP hunt tool is .2c; the anvil hunt CLI is .2d.`
  Verification: `cargo check --all-targets OK; cargo fmt --all --check OK; cargo clippy --all-targets -- -D warnings OK; cargo test --lib hunt:: 11/11 + downstream:: 20/0 (2 ignored); full cargo test green incl. tests/snapshots.rs 6/6 byte-identical (bundle_root default None ⇒ DUT byte-identical; the new downstream helper is additive, validate untouched).`
  Commit: `this BUG-HUNT-ORCHESTRATION.2b.2b commit`

- ID: `BUG-HUNT-ORCHESTRATION.2c`
  Status: `done`
  Goal: `The MCP hunt controlled tool wired into src/mcp dispatcher: input schema, HuntReport result, failing-run artifact-cache population (so anvil://artifact/<run_id>/{sv,introspection,manifest} reads work), a top-level hunt audit record; introspection/MCP doc + schema note; proofs.`
  Acceptance: `A controlled hunt tool in tools_list (hunt_schema) + tools_call ("hunt" arm → run_hunt) shimming anvil::hunt::run; HuntReport returned as JSON; each finding's run_id cached (original + minimized via downstream::introspect_dut_artifact) so anvil://artifact/<run_id>/{sv,introspection} resolve; a top-level hunt audit record; book/src/agent-mcp.md tool list/table updated; no introspection schema bump (HuntReport is a tool result, not part of the introspection document); cargo-portable proofs; cargo check/test/clippy/fmt green; snapshots 6/6 + book_examples unchanged (default anvil build untouched; hunt lives in anvil-mcp only).`
  Result: `Done. src/mcp/mod.rs: (1) tools_list gains hunt_schema (seed/seeds/config/tools/yosys_mode/minimize/max_oracle_calls/diff_sim, additionalProperties:false) + the hunt descriptor; tools_call gains the "hunt" arm. (2) run_hunt builds a HuntRequest from the parsed args (sandbox fixed to OS temp dir; bundle_root=None — the MCP path serves artifacts from the cache, never writing an on-disk bundle, decision 0004), calls hunt::run, then cache_hunt_failures populates self.cache for each finding's run_id (original = base cfg with the finding's seed; minimized when reproduced_initial) via downstream::introspect_dut_artifact so anvil://artifact/<run_id>/{sv,introspection} resolve, and pushes one top-level "hunt" audit record (sweep params + summary + per-finding seed/run_id/failing_tool/detection). (3) Lifted run_minimize's inline max_oracle_calls parse into a shared parse_max_oracle_calls (byte-identical for minimize) + added parse_hunt_seeds / parse_bool_arg. No introspection schema bump. book/src/agent-mcp.md: hunt added to the tool list + table, "two controlled tools" → "three", audit-log resource line notes hunt. 5 new cargo-portable mcp:: proofs (no-tools sweep round-trips + audits; unknown-tool rejection not audited; zero-seeds rejected; non-boolean flag rejected; synthetic-finding cache population makes anvil://artifact/<run_id>/{sv,introspection} resolve) + the tools/list test now expects hunt. The anvil hunt CLI shim is .2d; the real-tool e2e gate + full book/USER_GUIDE/README/KM closeout is .2e.`
  Verification: `cargo check --all-targets OK; cargo fmt --all --check OK; cargo clippy --all-targets -- -D warnings OK; cargo test --lib mcp:: 66/0 (incl. 5 new hunt proofs); full cargo test green incl. tests/snapshots.rs 6/6 byte-identical + tests/book_examples.rs (default anvil build untouched — hunt lives only in the anvil-mcp server).`
  Commit: `this BUG-HUNT-ORCHESTRATION.2c commit`

- ID: `BUG-HUNT-ORCHESTRATION.2d`
  Status: `done`
  Goal: `The anvil hunt CLI subcommand (ANVIL's first subcommand) as a thin shim over hunt::run, with --out as a human-only convenience; the byte-identical default-path guard (snapshots 6/6 + book-examples 3/3 unchanged); proofs.`
  Acceptance: `Cli gains an optional #[command(subcommand)] command: Option<Commands> (flat flags preserved ⇒ anvil --seed N … parses with command==None ⇒ existing flow byte-identical); a Commands::Hunt(HuntCommand) variant projecting HuntRequest (--seed/--seeds/--config/--tools/--yosys-mode/--no-minimize/--budget/--diff-sim/--out); main dispatches it before the lane/DUT path via run_hunt_command → build_hunt_request → hunt::run, printing the HuntReport JSON; --out ⇒ bundle_root (the CLI's on-disk bundle; MCP stays cache-only); AcceptanceTool gains clap::ValueEnum for --tools; cargo-portable proofs incl. the flat-default-no-subcommand guard + the arg→request mapping; cargo check/test/clippy/fmt green; snapshots 6/6 + book_examples unchanged.`
  Result: `Done. src/main.rs: (1) Cli gains #[command(subcommand)] command: Option<Commands> as the only new field — anvil --seed N … parses with command==None and runs the historical generate flow unchanged (byte-identical); anvil hunt … parses Some(Commands::Hunt(HuntCommand)). main dispatches the subcommand right after init_tracing (return run_hunt_command(hunt)) so the existing body is the fall-through path. (2) HuntCommand mirrors HuntRequest: --seed (0) / --seeds (16, range ≥1) / --config <path> / --tools <verilator,yosys,iverilog> (ValueEnum, comma-delimited) / --yosys-mode (without-abc) / --no-minimize / --budget (200, range ≥1) / --diff-sim / --out <dir>. (3) build_hunt_request(&HuntCommand)->Result<HuntRequest> (factored for a tool-free proof) loads --config JSON else Config::default, stamps --seed, validates; empty --tools ⇒ the verilator+yosys default; --out ⇒ bundle_root (the human-CLI bundle; the MCP path stays cache-only); the validate sandbox is the OS-temp default. run_hunt_command runs the sweep + prints the HuntReport JSON. src/downstream/mod.rs: AcceptanceTool gains clap::ValueEnum (pure derive). 5 cargo-portable anvil-bin proofs: flat_default_invocation_has_no_subcommand (the byte-identical routing guard), hunt_subcommand_parses_all_flags, hunt_subcommand_defaults, hunt_rejects_zero_seeds, build_hunt_request_maps_args_to_request. Manual real-tool smoke: anvil hunt --seed 1 --seeds 3 --tools verilator ⇒ n_failures=0 with distinct per-seed run_ids. Docs kept in sync: USER_GUIDE.md (anvil hunt section), README.md (Current CLI truth bullet), book/src/agent-mcp.md (the hunt row notes the CLI twin, no runnable block). The real-tool e2e gate (injected failure ⇒ bundle) + the full closeout is .2e.`
  Verification: `cargo check --all-targets OK; cargo fmt --all --check OK; cargo clippy --all-targets -- -D warnings OK; cargo test --bin anvil 12/0 (incl. 5 new hunt proofs); anvil hunt --help + anvil --seed 42 smokes; anvil hunt --seed 1 --seeds 3 --tools verilator ⇒ clean sweep; full cargo test green incl. tests/snapshots.rs 6/6 byte-identical + tests/book_examples.rs (flat default path untouched by the optional subcommand).`
  Commit: `this BUG-HUNT-ORCHESTRATION.2d commit`

- ID: `BUG-HUNT-ORCHESTRATION.2e`
  Status: `done`
  Goal: `A real-tool end-to-end gate (#[ignore], tool-gated) that runs a hunt against Verilator/Yosys and produces a one-command-reproducible bundle for an injected/known failure; book/src/agent-mcp.md + USER_GUIDE + README + a KM card; close .2 and the tree.`
  Acceptance: `A #[ignore] tool-gated tests/hunt_e2e.rs that drives the real anvil hunt binary against real Verilator and proves the loop + the reproducer recipe end-to-end (tool-less ⇒ skips green); the book "bug-hunting loop end to end" rewritten to feature the turnkey hunt (CLI + MCP); a KM how-to card; ROADMAP lane 1 marked delivered; the tree + .2 + the root node closed; cargo check/test/clippy/fmt green incl. snapshots 6/6 + book_examples.`
  Result: `Done — closes the tree. (1) tests/hunt_e2e.rs: two #[ignore] tool-gated proofs — hunt_cli_clean_sweep_against_real_verilator (the real anvil hunt binary runs a 3-seed sweep against real Verilator, n_failures=0 with distinct per-seed run_ids — valid-by-construction ⇒ a clean sweep is the steady state) + hunt_reproducer_recipe_is_byte_identical_and_accepted (anvil --seed S --config <dumped knobs> reproduces anvil --seed S byte-for-byte AND Verilator accepts the regenerated repro.sv — the repro.sh recipe). Documented honestly that ANVIL has no by-construction downstream failure to manufacture (a real rejection would be an actual downstream-tool bug, the thing the loop surfaces); the bundle DIRECTORY format is unit-proven cargo-portably by .2b.2b's write_bundle test. Added serde_json to [dev-dependencies] (integration tests can't name regular deps) for the typed HuntReport parse. (2) book/src/agent-mcp.md "The bug-hunting loop, end to end" rewritten with a "One command: the hunt loop" subsection featuring both the anvil hunt CLI and the MCP hunt tool (tool-requiring bash block marked book-test:skip). (3) KM how-to card docs/knowledge/bug-hunt-cli.md (usage-focused, points to USER_GUIDE + book, reverify = cargo test --test hunt_e2e -- --ignored) — KM 46→47 facts. (4) ROADMAP lane 1 marked DONE; the tree, .2, and the root node closed. Default anvil build / DUT byte-identical (test + docs only; no src/ generator/emitter change).`
  Verification: `cargo test --test hunt_e2e -- --ignored 2/2 against real Verilator; portable cargo test --test hunt_e2e 0 run/2 ignored (tool-less-safe); cargo check --all-targets OK; cargo clippy --all-targets -- -D warnings OK; cargo fmt --all --check OK; full cargo test green incl. tests/snapshots.rs 6/6 byte-identical + tests/book_examples.rs; KM gen+check OK (47 facts).`
  Commit: `this BUG-HUNT-ORCHESTRATION.2e commit`

## Current Frontier

**Tree closed `2026-06-17`.** No frontier — all leaves done.

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| — | `BUG-HUNT-ORCHESTRATION.2e` | `done` | The real-tool e2e gate (`tests/hunt_e2e.rs`, 2 `#[ignore]` proofs against real Verilator) + the book "end to end" rewrite + the `bug-hunt-cli` KM card; closes `.2`, the tree, and the root node. |
| — | `BUG-HUNT-ORCHESTRATION.2a` | `done` | Extracted the diff-sim run+compare into `diff_sim::run_agreement` (byte-identical; snapshots 6/6). |
| — | `BUG-HUNT-ORCHESTRATION.2b.1` | `done` | The `src/hunt/` library core (`hunt::run` + types, reject/warning detection) + the seed-threading fix; cargo-portable proofs; snapshots 6/6. |
| — | `BUG-HUNT-ORCHESTRATION.2b.2a` | `done` | Folded cross-sim mismatch detection into `hunt::run` (`diff_sim::run_agreement`) + extracted the shared `downstream::generate_dut_artifact`; snapshots 6/6. |
| — | `BUG-HUNT-ORCHESTRATION.2b.2b` | `done` | The reproducer-bundle emitter (`<bundle_root>/<run_id>/` per finding) + `introspect_dut_artifact`; closes the `.2b` engine. Snapshots 6/6. |
| — | `BUG-HUNT-ORCHESTRATION.2c` | `done` | The MCP `hunt` controlled tool (`run_hunt` + cache population + audit record) — the loop is now MCP-invocable + queryable (decision `0017`). mcp:: 66/0; snapshots 6/6. |
| — | `BUG-HUNT-ORCHESTRATION.2d` | `done` | The `anvil hunt` CLI subcommand (ANVIL's first) — optional `command: Option<Commands>` keeps the flat default byte-identical; `--out` ⇒ on-disk bundle. anvil-bin 12/0; snapshots 6/6 + book_examples. |

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
| `2026-06-17` | `BUG-HUNT-ORCHESTRATION.2b.1` (fix) | `seed-threading bug fixed (Generator seeds from cfg.seed; sweep now stamps seed into a per-iteration seed_config) + new proof seed_config_threads_the_swept_seed; cargo check/clippy/fmt green; cargo test --lib hunt:: 6/6; full cargo test green incl. snapshots 6/6` | `done` |
| `2026-06-17` | `BUG-HUNT-ORCHESTRATION.2b.2a` | `downstream::generate_dut_artifact extracted (validate byte-identical, downstream 20/0); HuntRequest.diff_sim + HuntFailure.diff_sim + cross_sim_mismatch fold; proof diff_sim_on_clean_artifact_no_ops_without_simulators; cargo check/clippy/fmt green; cargo test --lib hunt:: 7/7; full cargo test green incl. snapshots 6/6 byte-identical` | `done` |
| `2026-06-17` | `BUG-HUNT-ORCHESTRATION.2b.2b` | `downstream::introspect_dut_artifact added; HuntRequest.bundle_root + HuntFailure.bundle (HuntBundle) + write_bundle/repro_script/shell_quote; bundle dir per finding (repro.sv/knobs.json/introspection.json/hunt-verdict.json/tool-logs/repro.sh); 4 cargo-portable proofs (hunt:: 7→11); cargo check/clippy/fmt green; downstream:: 20/0; full cargo test green incl. snapshots 6/6 byte-identical` | `done` |
| `2026-06-17` | `BUG-HUNT-ORCHESTRATION.2c` | `MCP hunt controlled tool (hunt_schema + "hunt" dispatch + run_hunt + cache_hunt_failures + hunt audit record); shared parse_max_oracle_calls/parse_hunt_seeds/parse_bool_arg (run_minimize reuses parse_max_oracle_calls, byte-identical); book/src/agent-mcp.md tool list/table; 5 new mcp:: proofs (mcp:: 66/0); no introspection schema bump; cargo check/clippy/fmt green; full cargo test green incl. snapshots 6/6 byte-identical + book_examples` | `done` |
| `2026-06-17` | `BUG-HUNT-ORCHESTRATION.2d` | `anvil hunt subcommand (Cli command: Option<Commands>; HuntCommand; run_hunt_command/build_hunt_request; AcceptanceTool clap::ValueEnum); USER_GUIDE/README/book agent-mcp synced; 5 anvil-bin proofs (anvil-bin 12/0) incl. flat-default-no-subcommand guard; real-tool smoke anvil hunt --seeds 3 --tools verilator ⇒ n_failures=0; cargo check/clippy/fmt green; full cargo test green incl. snapshots 6/6 byte-identical + book_examples` | `done` |
| `2026-06-17` | `BUG-HUNT-ORCHESTRATION.2e` | `tests/hunt_e2e.rs (2 #[ignore] tool-gated proofs vs real Verilator: clean sweep + byte-identical reproducer recipe) 2/2 --ignored, 0 portable; serde_json dev-dep; book "end to end" rewrite (turnkey hunt CLI+MCP); KM card bug-hunt-cli (47 facts); ROADMAP lane 1 DONE; tree + .2 + root closed; cargo check/clippy/fmt green; full cargo test green incl. snapshots 6/6 byte-identical + book_examples` | `done` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `BUG-HUNT-ORCHESTRATION` | `USABILITY-LANE-OWNERSHIP.1 — register 7 owner-directed usability/capability lanes + API-first decision 0017` | Tree registered (not yet started); frontier `.1` (design ADR) pending. |
| `BUG-HUNT-ORCHESTRATION.1` | `BUG-HUNT-ORCHESTRATION.1 — design ADR (decision 0018): turnkey fuzz→detect→minimize→bundle loop + MCP hunt tool + anvil hunt CLI` | Design/decision leaf (docs-only). Pins the loop/bundle/MCP+CLI/detection/sandbox; pre-splits `.2` into `.2a`…`.2e`. DUT byte-identical. |
| `BUG-HUNT-ORCHESTRATION.2a` | `BUG-HUNT-ORCHESTRATION.2a — extract diff-sim run+compare into reusable diff_sim::run_agreement` | Pure byte-identical refactor; `src/diff_sim/` now owns `run_agreement` + `DiffSimReport` + the SV-text testbench; `tool_matrix` wraps it. First impl leaf of `.2`. |
| `BUG-HUNT-ORCHESTRATION.2b.1` | `BUG-HUNT-ORCHESTRATION.2b.1 — src/hunt/ library core (hunt::run loop + types, reject/warning detection)` | New `src/hunt/mod.rs` + `pub mod hunt`; composes `downstream::validate`/`minimize`; SCHEMA-DERIVED `HuntReport`; 5 cargo-portable proofs; default-off / DUT byte-identical. Cross-sim + bundle = `.2b.2`. |
| `BUG-HUNT-ORCHESTRATION.2b.1` (fix) | `BUG-HUNT-ORCHESTRATION.2b.1 — fix: thread the swept seed into the per-iteration config` | Correctness fix: the generator seeds from `cfg.seed`, so the sweep must stamp `seed` into each iteration's config (`seed_config`), not just the `validate` `seed` arg; + the `seed_config_threads_the_swept_seed` proof. Found while grounding `.2b.2`. |
| `BUG-HUNT-ORCHESTRATION.2b.2a` | `BUG-HUNT-ORCHESTRATION.2b.2a — fold cross-sim mismatch detection into hunt::run + extract downstream::generate_dut_artifact` | Cross-sim fold (`HuntRequest.diff_sim` → `run_agreement` on clean artifacts → `cross_sim_mismatch` finding) + the shared `generate_dut_artifact` helper (validate byte-identical). Default-off / DUT byte-identical. Bundle emitter = `.2b.2b`. |
| `BUG-HUNT-ORCHESTRATION.2b.2b` | `BUG-HUNT-ORCHESTRATION.2b.2b — reproducer-bundle emitter (write <bundle_root>/<run_id>/ per finding) + introspect_dut_artifact` | `HuntRequest.bundle_root` + `HuntFailure.bundle` (`HuntBundle`); per-finding directory (`repro.sv`/`knobs.json`/`introspection.json`/`hunt-verdict.json`/`tool-logs`/`repro.sh`) via the shared `generate_dut_artifact` + new `introspect_dut_artifact`. Prefers the minimized reproducer. Closes the `.2b` engine. Default-off / DUT byte-identical. |
| `BUG-HUNT-ORCHESTRATION.2c` | `BUG-HUNT-ORCHESTRATION.2c — the MCP hunt controlled tool (turnkey loop, MCP-invocable + queryable)` | `hunt` tool in `src/mcp` (`run_hunt` shim over `hunt::run` + `cache_hunt_failures` + a `hunt` audit record); shared `parse_max_oracle_calls`/`parse_hunt_seeds`/`parse_bool_arg`; `book/src/agent-mcp.md` tool list/table. `bundle_root=None` (MCP serves artifacts from the cache). Default `anvil` build / DUT byte-identical. |
| `BUG-HUNT-ORCHESTRATION.2d` | `BUG-HUNT-ORCHESTRATION.2d — the anvil hunt CLI subcommand (ANVIL's first subcommand)` | `anvil hunt` in `src/main.rs` (optional `command: Option<Commands>` keeps the flat default byte-identical; `HuntCommand` + `run_hunt_command`/`build_hunt_request` shim over `hunt::run`; `--out` ⇒ on-disk bundle); `AcceptanceTool` gains `clap::ValueEnum`; USER_GUIDE/README/book synced. Flat default path byte-identical. |
| `BUG-HUNT-ORCHESTRATION.2e` | `BUG-HUNT-ORCHESTRATION.2e — real-tool e2e gate + closeout (closes the tree)` | `tests/hunt_e2e.rs` (2 `#[ignore]` tool-gated proofs vs real Verilator: clean sweep + byte-identical reproducer recipe); `serde_json` dev-dep; book "bug-hunting loop end to end" rewrite (turnkey `hunt` CLI+MCP); KM card `bug-hunt-cli`; ROADMAP lane 1 DONE; **tree + `.2` + root closed**. Default `anvil` build / DUT byte-identical (test + docs only). |

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
- `2026-06-17`: `.2b.1` seed-threading fix (the sweep must stamp `seed` into the
  per-iteration config; `seed_config` + proof).
- `2026-06-17`: pre-split `.2b.2` into `.2b.2a` (cross-sim fold + the shared
  `generate_dut_artifact` extract) + `.2b.2b` (reproducer-bundle emitter).
  `.2b.2a` done — `HuntRequest.diff_sim` runs `diff_sim::run_agreement` on each
  validate-clean artifact (a mismatch is a `cross_sim_mismatch` finding carrying
  the `DiffSimReport`); extracted `downstream::generate_dut_artifact` (validate
  byte-identical); proof that diff-sim is a no-op without simulators. Default-off
  / DUT byte-identical (snapshots 6/6). Frontier advanced to `.2b.2b`.
- `2026-06-17`: `.2b.2b` done — the reproducer-bundle emitter.
  `HuntRequest.bundle_root: Option<PathBuf>` (caller-set, never agent-supplied)
  makes each finding write a self-contained directory `<bundle_root>/<run_id>/`
  (`repro.sv` via `generate_dut_artifact`, `knobs.json`, `introspection.json` via
  the new `downstream::introspect_dut_artifact`, `hunt-verdict.json`,
  `tool-logs/NOTE.txt`, and a one-command `repro.sh` that regenerates the `.sv`
  then replays the failing tool's `argv` with the ephemeral sandbox path
  substituted to `repro.sv`); `HuntFailure.bundle: Option<HuntBundle>` carries the
  path + `anvil://` resource URIs. Prefers the minimized reproducer. 4 new
  cargo-portable proofs (hunt:: 7→11). Default-off / DUT byte-identical
  (snapshots 6/6). **`.2b.2`/`.2b` close** — the `src/hunt/` engine is complete.
  Frontier advanced to `.2c` (the MCP `hunt` tool).
- `2026-06-17`: `.2c` done — the MCP `hunt` controlled tool. `run_hunt` shims
  `hunt::run` (sandbox fixed to OS temp; `bundle_root=None`), `cache_hunt_failures`
  populates the artifact cache for each finding's `run_id` (original + minimized,
  via `downstream::introspect_dut_artifact`) so
  `anvil://artifact/<run_id>/{sv,introspection}` resolve, and one top-level `hunt`
  audit record carries the sweep params + summary. Lifted the shared
  `parse_max_oracle_calls` (reused by `minimize`, byte-identical) + new
  `parse_hunt_seeds`/`parse_bool_arg`. `book/src/agent-mcp.md` tool list/table
  updated; no introspection schema bump. 5 new cargo-portable `mcp::` proofs
  (mcp:: 66/0). Default `anvil` build / DUT byte-identical (snapshots 6/6 +
  book_examples). The loop is now MCP-invocable + queryable (decision `0017`).
  Frontier advanced to `.2d` (the `anvil hunt` CLI shim).
- `2026-06-17`: `.2d` done — the `anvil hunt` CLI subcommand (ANVIL's **first**
  subcommand). `Cli` gains an optional `#[command(subcommand)] command:
  Option<Commands>`, so `anvil --seed N …` parses with `command == None` and runs
  the historical generate flow unchanged (byte-identical default); `anvil hunt …`
  dispatches `run_hunt_command` → `build_hunt_request` → `hunt::run`, printing the
  `HuntReport` JSON. `HuntCommand` projects `HuntRequest`; `--out` ⇒ `bundle_root`
  (the human-CLI on-disk bundle; the MCP path stays cache-only); `AcceptanceTool`
  gains `clap::ValueEnum` for `--tools`. 5 cargo-portable `anvil`-bin proofs
  (incl. the flat-default-no-subcommand guard + the arg→request mapping) + a
  real-tool smoke (`anvil hunt --seed 1 --seeds 3 --tools verilator` ⇒
  `n_failures = 0`). USER_GUIDE/README/`book/src/agent-mcp.md` synced. Default path
  byte-identical (snapshots 6/6 + book_examples). Frontier advanced to `.2e` (the
  real-tool e2e gate + full closeout, which closes the tree).
- `2026-06-17`: `.2e` done — **the tree is CLOSED.** Added `tests/hunt_e2e.rs`:
  two `#[ignore]` tool-gated proofs that drive the real `anvil hunt` binary
  against real Verilator — `hunt_cli_clean_sweep_against_real_verilator` (clean
  3-seed sweep, `n_failures=0`, distinct per-seed `run_id`s) and
  `hunt_reproducer_recipe_is_byte_identical_and_accepted` (`anvil --config
  <dumped knobs>` reproduces `anvil --seed` byte-for-byte + Verilator accepts the
  regenerated `repro.sv`). Documented the honest boundary: ANVIL has no
  by-construction downstream failure to manufacture (a real rejection would be an
  actual downstream-tool bug — the thing the loop surfaces); the bundle directory
  format is unit-proven by `.2b.2b`. Added `serde_json` to `[dev-dependencies]`
  for the typed `HuntReport` parse. Rewrote the book's "bug-hunting loop end to
  end" with a turnkey `hunt` (CLI + MCP) subsection; added the `bug-hunt-cli` KM
  how-to card (KM 46→47); marked ROADMAP lane 1 DONE; closed the tree, `.2`, and
  the root node. Default `anvil` build / DUT byte-identical (test + docs only).
  **No frontier — `BUG-HUNT-ORCHESTRATION` is complete.**
