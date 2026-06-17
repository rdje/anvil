# ACCEPTANCE-DIVERGENCE-HUNTING: tool-A-accepts / tool-B-rejects divergence finder

## Metadata

- Tree ID: `ACCEPTANCE-DIVERGENCE-HUNTING`
- Status: `active`
- Roadmap lane: `Usability — acceptance-divergence bug-finder (north star, idea 2)`
- Created: `2026-06-17`
- Last updated: `2026-06-17` (`.1` ADR `0019` + `.2a` `tool_verdict` + `.2b` `src/divergence/` core + `.2c.1` hunt fold + `.2c.2` `tool_matrix` column done; `.2c` closed; frontier `.2d` — the MCP `divergence` tool + CLI shim)
- Owner: repo-local workflow

## Goal

Make **acceptance divergence** a first-class signal. `--diff-sim` already proves
cross-*simulator* trace agreement; this lane adds the complementary axis:
detecting and reporting where **one tool accepts an artifact and another rejects
it** (or where two *versions* of the same tool disagree). Such accept/reject
divergence on valid-by-construction RTL is exactly where real downstream-tool
bugs live. Deliver a per-unit per-tool accept/warn/reject matrix, a divergence
classifier, and a report — surfaced as a `tool_matrix` column **and** as an MCP
query — building on the existing hardened `src/downstream/` adapters and the
`src/diff_sim/` precedent.

## Non-Goals

- No behavioural oracle (decision `0004`, ROADMAP gap 4) — this is about
  *acceptance* divergence (parse/elaborate/lint/synth verdicts), composed with
  the existing semantic-agreement column, not a new truth model.
- No new generator semantics; default DUT output stays byte-identical.
- No vendoring of tools; divergence is computed over external, sandboxed
  invocations.

## Acceptance Criteria

- A run produces a per-artifact accept/warn/reject matrix across the enabled
  tools (and/or tool versions) and flags every divergence, with the divergent
  artifact retained as a reproducer (seed + effective knobs + `.sv` + each tool's
  log).
- **API-completeness gate (decision `0017`):** the divergence run is invocable
  over MCP and every divergence verdict/report is queryable via the
  MCP/introspection API (SCHEMA-DERIVED — a projection of the recorded verdicts,
  not a recomputed truth); the CLI/`tool_matrix` flag is a shim over the same
  surface.
- Reproducible + sandboxed (seeded; allow-list + RAM guard + audit log).
- Default-off / DUT byte-identical; downstream-clean; documented in
  `book/src/agent-mcp.md` + `book/src/synthesizability.md` + USER_GUIDE + README;
  committed through `COMMIT.md`.

## Task Tree

- ID: `ACCEPTANCE-DIVERGENCE-HUNTING`
  Status: `active`
  Goal: `A first-class accept/warn/reject divergence finder across tools (and tool versions), surfaced as a tool_matrix column + an MCP query, built on the existing downstream adapters + the diff_sim precedent.`
  Children: `ACCEPTANCE-DIVERGENCE-HUNTING.1, ACCEPTANCE-DIVERGENCE-HUNTING.2`

- ID: `ACCEPTANCE-DIVERGENCE-HUNTING.1`
  Status: `done`
  Goal: `Design/decision leaf (ADR, no code): pin the divergence model (per-unit per-tool verdict = accept/warn/reject + the divergence classification, incl. tool-version-vs-version), the report shape (a DivergenceReport beside DiffSimReport), the tool_matrix column + the MCP query surface (decision 0017 API-completeness), and the reproducer-retention policy. Decide reuse of run_verilator/run_yosys/run_iverilog + the diff_sim subset-selection pattern. Record as the next decision record + pre-split .2 (impl).`
  Acceptance: `A decision record + a tree/DEVELOPMENT_NOTES entry pinning the divergence model, the report, and the MCP+matrix surface; docs-only; INDEX + this tree + docs/TASK_TREE.md updated.`
  Result: `Done. Wrote docs/decisions/0019-acceptance-divergence-hunting.md (the design ADR; KM answers: front-matter; binds 0017 + 0004 + 0011 + 0018; evidence grounded in the real src/downstream / src/hunt / src/diff_sim / src/bin/tool_matrix / src/mcp / src/introspect surfaces verified this session via a code-surface recon agent — exact signatures cited). It pins: (verdict) a trinary accept/warn/reject projection of one ToolInvocation — the same classification hunt::run already does inline — extracted to a shared downstream::tool_verdict so there is no second classifier (full-factorization); (classifier) a divergence = "not all verdicts equal", classed accept_reject | accept_warn | warn_reject | version_mismatch over labelled tools (verilator / yosys-without-abc / yosys-with-abc / iverilog), so --yosys-mode both can itself diverge; (detector) a library composer divergence::run(seed,cfg,&DivergenceOptions)->DivergenceReport in src/divergence/ (symmetry with src/diff_sim + src/hunt) that runs every enabled tool/mode to completion (no fold, no short-circuit) over generate_dut_artifact + the existing run_* primitives; (report) a SCHEMA-DERIVED DivergenceReport{run_id,lane,kind,top,sandbox,verdicts:[ToolVerdict],diverged,divergences:[Divergence],declined} beside DiffSimReport/HuntReport; (three surfaces, one detector) a hunt detection axis (HuntRequest.divergence → "acceptance_divergence" finding), a tool_matrix column (ModuleReport/DesignReport.divergence + saw_acceptance_divergence + classify_diff_sim_axis subset), and a controlled MCP divergence tool (cache + audit) + CLI shim — resolving the "hunt vs matrix" open question to BOTH; (reproducer) reuse write_bundle / tool_matrix .sv+log retention — no new format, repro.sh records EACH labelled tool's argv; (version axis) deferred to .2e (caller supplies binaries/labels, kind stays allow-listed); (honesty) saw_acceptance_divergence is opportunistic, NEVER a required gate (all-agree is the valid-by-construction steady state); (discipline) seeded/sandboxed/allow-listed/RAM-guarded/audit-logged, default-off / DUT byte-identical. Added the docs/decisions/INDEX.md row (0019), a DEVELOPMENT_NOTES.md entry, refreshed MEMORY.md + CHANGES.md + the docs/TASK_TREE.md frontier + the ROADMAP lane-2 note. Pre-split .2 into .2a..2f (below). Docs-only — no src/ touched ⇒ DUT byte-identical.`
  Verification: `Docs-only / no src/ ⇒ cargo check/clippy/fmt/test unaffected (code state = green BUG-HUNT-ORCHESTRATION.2e baseline). bash scripts/check_memory_architecture.sh OK; knowledge-map gen+check OK (new 0019 card folded in, 47→48 facts). DUT byte-identical.`
  Commit: `this ACCEPTANCE-DIVERGENCE-HUNTING.1 commit`

- ID: `ACCEPTANCE-DIVERGENCE-HUNTING.2`
  Status: `pending`
  Goal: `Implement the .1 design: the shared classifier + the src/divergence/ core + the hunt axis + the tool_matrix column + the MCP tool + version-vs-version + a real-tool end-to-end gate + book/USER_GUIDE/README/KM. Default-off / DUT byte-identical. Pre-split at .1 into .2a..2f (below).`
  Acceptance: `All of .2a..2f done; divergence::run composes the existing run_* primitives + the shared tool_verdict; surfaced as a hunt axis + a tool_matrix column + an MCP divergence tool (decision-0017 gate: MCP-invocable + queryable + CLI a shim); version-vs-version landed; an injected accept/reject pair is classified accept_reject AND an all-agree real-tool run records diverged=false; snapshots 6/6 + book-examples 3/3 unchanged; downstream-clean; documented; committed per COMMIT.md.`
  Verification: `pending`
  Commit: `pending`
  Children: `ACCEPTANCE-DIVERGENCE-HUNTING.2a, .2b, .2c, .2d, .2e, .2f`

- ID: `ACCEPTANCE-DIVERGENCE-HUNTING.2a`
  Status: `done`
  Goal: `Extract the inline accept/warn/reject classifier from hunt::run into a shared downstream::tool_verdict(&ToolInvocation) -> ToolVerdict (pure refactor; the BUG-HUNT-ORCHESTRATION.2a extract-then-reuse precedent) so the divergence detector and the hunt share one classifier. Byte-identical hunt behaviour. Orderable first.`
  Acceptance: `downstream::tool_verdict + a ToolVerdict{Accept,Warn,Reject} enum (serde) live in src/downstream/mod.rs; hunt::run's HuntFailure.detection derives from it (reject/warning unchanged on the wire); cargo check/test/clippy/fmt green; hunt:: tests unchanged; snapshots 6/6 byte-identical.`
  Result: `Done. (1) src/downstream/mod.rs: added pub enum ToolVerdict{Accept,Warn,Reject} (Serialize/Deserialize, #[serde(rename_all="snake_case")]) + pub fn tool_verdict(&ToolInvocation)->ToolVerdict (success ⇒ Accept; clean exit Some(0) + !success ⇒ Warn; non-zero/unknown ⇒ Reject) — the single accept/warn/reject classifier, a SCHEMA-DERIVED projection of one ToolInvocation (no new computed truth; feedback_full_factorization). (2) src/hunt/mod.rs: classify_detection now derives from downstream::tool_verdict (Warn ⇒ "warning", Accept|Reject ⇒ "reject"; first_failing_tool only yields a non-succeeding invocation so Accept is unreachable, treated as reject defensively) — byte-identical to the prior inline exit_code==Some(0)?"warning":"reject"; imported tool_verdict + ToolVerdict. (3) Proof downstream::tests::tool_verdict_classifies_accept_warn_reject (the 3 verdicts + the unreachable success/exit!=0 case + the "warn" wire form). The hunt classify_detection_distinguishes_warning_from_reject proof still passes unchanged (byte-identical). No CLI/MCP/divergence-core yet (those are .2b+). Default-off / DUT byte-identical.`
  Verification: `cargo check --all-targets OK; cargo fmt --all --check OK; cargo clippy --all-targets -- -D warnings OK; cargo test --lib downstream:: 21/0 (2 ignored; +1 new tool_verdict proof) + hunt:: 11/11 (classify_detection proof unchanged); full cargo test --lib 522/0 (2 ignored); tests/snapshots.rs 6/6 byte-identical (the refactor is byte-identical; ToolInvocation wire shape unchanged).`
  Commit: `this ACCEPTANCE-DIVERGENCE-HUNTING.2a commit`

- ID: `ACCEPTANCE-DIVERGENCE-HUNTING.2b`
  Status: `done`
  Goal: `The src/divergence/ library core: ToolVerdict (reused) / Divergence / DivergenceReport / DivergenceOptions types + divergence::run(seed,cfg,&DivergenceOptions)->Result<DivergenceReport> composing generate_dut_artifact + run_verilator/run_yosys/run_iverilog_compile (all enabled tools/modes to completion, no fold, no short-circuit) + the shared classifier + the multi-tool same-version divergence classifier. Cargo-portable proofs. No CLI/MCP/version axis yet. Default-off / DUT byte-identical.`
  Acceptance: `src/divergence/mod.rs + lib.rs pub mod divergence; divergence::run runs every enabled labelled tool to completion, classifies each verdict, and flags any disagreement; DivergenceReport SCHEMA-DERIVED; cargo-portable proofs (a synthetic accept/reject ToolInvocation set ⇒ accept_reject; a no-tools run ⇒ friendly no-op/empty verdicts); cargo check/test/clippy/fmt green; snapshots 6/6 byte-identical.`
  Result: `Done. New src/divergence/mod.rs + pub mod divergence in lib.rs. Types: DivergenceOptions { validate: ValidateOptions } (wraps ValidateOptions — the MinimizeOptions precedent — so the .2e version axis extends it); ToolDecision { tool, verdict: ToolVerdict, exit_code, first_message } (the per-tool record — renamed from the ADR's "ToolVerdict { tool, … }" sketch to avoid clashing with the .2a enum); Divergence { kind, tools }; DivergenceReport { run_id, lane, kind, top, sandbox, verdicts, diverged, divergences, declined } — all serde, every field SCHEMA-DERIVED. divergence::run REUSES the one hardened downstream::validate orchestration (which already runs every enabled tool/mode to completion — no short-circuit on reject; only MemGuard declines) and projects report.tools into ToolDecisions via the shared downstream::tool_verdict (.2a), then classifies disagreement — NOT a forked sandbox loop / no second classifier (full-factorization). classify_divergences emits one Divergence per present pair-class (accept_reject/accept_warn/warn_reject), deterministic (sorted+deduped tools, fixed order); up to all three co-occur when all three verdict values are present; Yosys `both` ⇒ two labelled verdicts so without-abc-vs-with-abc is itself a divergence. 7 cargo-portable proofs: to_decision projection, accept_reject classification (the synthetic accept/reject set the .1 ADR requires), all-agree ⇒ no divergence, all-three-pair-classes, the Yosys-mode divergence, the no-tools friendly no-op run (generate+sandbox only, run_id non-empty), and the report serde round-trip (absent optional fields off the wire + the "accept" snake_case form). No CLI/MCP yet (.2c/.2d); no version axis yet (.2e). Default-off / DUT byte-identical.`
  Verification: `cargo check --all-targets OK; cargo fmt --all --check OK; cargo clippy --all-targets -- -D warnings OK; cargo test --lib divergence:: 7/7; full cargo test --lib 529/0 (522→529, +7); tests/snapshots.rs 6/6 byte-identical (divergence is default-off, wired into no generate/emit path ⇒ DUT byte-identical).`
  Commit: `this ACCEPTANCE-DIVERGENCE-HUNTING.2b commit`

- ID: `ACCEPTANCE-DIVERGENCE-HUNTING.2c`
  Status: `done`
  Goal: `Fold the detector into hunt::run (HuntRequest.divergence → an "acceptance_divergence" finding + HuntFailure.divergence, no minimize by default) AND add the tool_matrix column (--divergence, ModuleReport/DesignReport.divergence, the opportunistic saw_acceptance_divergence fact, the classify_diff_sim_axis subset reuse). Default-off / DUT byte-identical. Pre-split at pick into .2c.1 (hunt fold) + .2c.2 (tool_matrix column).`
  Acceptance: `Both .2c.1 + .2c.2 done; a hunt finding where some tool accepted while another rejected/warned is an acceptance_divergence; tool_matrix records the per-unit divergence column + the opportunistic fact; cargo-portable proofs; cargo check/test/clippy/fmt green; snapshots 6/6 + book_examples unchanged.`
  Result: `Done — both children landed. .2c.1 folded the detector into hunt::run (HuntRequest.divergence → an acceptance_divergence finding via the extracted classify_report, no minimize). .2c.2 added the tool_matrix --divergence column (ModuleReport/DesignReport.divergence via the same classify_report over the tools the matrix already ran, the opportunistic saw_acceptance_divergence fact never a gate, the classify_diff_sim_axis subset reuse). One shared detector, two surfaces, no drift. Default-off / DUT byte-identical.`
  Verification: `Both children green (see .2c.1 and .2c.2 rows + the Verification Log). cargo check/clippy/fmt green; snapshots 6/6 byte-identical + book_examples 3/3 across both.`
  Commit: `closed by the .2c.1 + .2c.2 commits (one leaf per commit)`
  Children: `ACCEPTANCE-DIVERGENCE-HUNTING.2c.1, ACCEPTANCE-DIVERGENCE-HUNTING.2c.2`

- ID: `ACCEPTANCE-DIVERGENCE-HUNTING.2c.1`
  Status: `done`
  Goal: `Fold the detector into hunt::run: HuntRequest.divergence: bool + HuntFailure.divergence: Option<DivergenceReport>; on a finding (!report.ok), classify the tools validate ALREADY ran (divergence::classify_report — no re-validation) and, when some tool accepted while another rejected/warned, mark the finding detection="acceptance_divergence" + attach the report. Not minimized (the validate oracle can't preserve a cross-tool disagreement, like cross_sim_mismatch). Cargo-portable proofs. Default-off / DUT byte-identical.`
  Acceptance: `divergence::classify_report(&ValidateReport) extracted (run refactored to use it, byte-identical); HuntRequest.divergence + HuntFailure.divergence added; the finding path classifies + refines detection; minimize skipped for a divergence finding; MCP run_hunt + CLI build_hunt_request set divergence:false (arg-wiring is .2d, byte-identical); cargo-portable proofs; cargo check/test/clippy/fmt green; snapshots 6/6 + book_examples unchanged.`
  Result: `Done. (1) src/divergence/mod.rs: extracted pub fn classify_report(&ValidateReport) -> DivergenceReport (the pure projection half of run; run now = validate + classify_report, byte-identical) so the hunt classifies the tools validate already ran without re-validating; +1 proof classify_report_projects_a_validate_report (divergence:: 7→8). (2) src/hunt/mod.rs: HuntRequest.divergence: bool + HuntFailure.divergence: Option<DivergenceReport> (skip_serializing_if) + import classify_report/DivergenceReport. On the finding path (!report.ok), when req.divergence, classify_report(&report); a diverged result refines detection to "acceptance_divergence" and attaches the report; minimize is skipped for a divergence finding (the validate oracle can't preserve a cross-tool disagreement — the cross_sim_mismatch rationale). Cross-sim + finding HuntFailure sites get divergence: None. +1 proof divergence_flag_is_inert_on_a_clean_sweep (hunt:: 11→12). (3) src/mcp/mod.rs run_hunt + src/main.rs build_hunt_request set divergence: false (default-off; the MCP/CLI arg-wiring is .2d) so both shims stay byte-identical; fixed the mcp test HuntFailure literal. No MCP schema / no CLI flag change yet. Default-off / DUT byte-identical.`
  Verification: `cargo check --all-targets OK; cargo fmt --all --check OK; cargo clippy --all-targets -- -D warnings OK; cargo test --lib divergence:: 8/8 + hunt:: 12/12; full cargo test --lib 531/0 (529→531); tests/snapshots.rs 6/6 byte-identical + tests/book_examples.rs 3/3 (default anvil path untouched — divergence default-off).`
  Commit: `this ACCEPTANCE-DIVERGENCE-HUNTING.2c.1 commit`

- ID: `ACCEPTANCE-DIVERGENCE-HUNTING.2c.2`
  Status: `done`
  Goal: `Add the tool_matrix divergence column: a --divergence opt-in, ModuleReport/DesignReport.divergence: Option<DivergenceReport> (the diff_sim column precedent), the opportunistic saw_acceptance_divergence coverage fact (NEVER a required gate — all-agree is the valid-by-construction steady state), and the classify_diff_sim_axis subset reuse. Cargo-portable proofs. Default-off / DUT byte-identical.`
  Acceptance: `tool_matrix gains --divergence + the per-unit divergence column via divergence::classify_report over the tools it already ran; the opportunistic saw_acceptance_divergence fact (recorded when seen, not required); cargo check/test/clippy/fmt green; snapshots 6/6 byte-identical.`
  Result: `Done. src/bin/tool_matrix.rs gains: (1) the --divergence opt-in flag (default-off) + MatrixReport.divergence_enabled/divergence_subset header fields (the diff_sim_* precedent). (2) ModuleReport.divergence + DesignReport.divergence: Option<DivergenceReport> (#[serde(default, skip_serializing_if)] — old checkpoints still deserialize). (3) unit_divergence(...) — the per-unit projection: it gathers the verilator/yosys/iverilog_compile ToolInvocations the matrix ALREADY ran on the unit, assembles a ValidateReport, and classifies it through the ONE shared divergence::classify_report (no second classifier; feedback_full_factorization). It spawns NO extra tool and — unlike --diff-sim — has NO tool-clean precondition (a divergence is most interesting when one tool rejects what another accepts), so its gate is only cli.divergence && in-subset && >=1 tool ran. (4) Subset reuse: gated by the same select_diff_sim_subset/classify_diff_sim_axis selector via its own .divergence-subset sentinel; the shared membership check was factored into scenario_in_named_subset (one impl, used by both columns). (5) CoverageSummary.saw_acceptance_divergence — the OPPORTUNISTIC fact lit by summarize_coverage/summarize_design_coverage from a diverged unit + unioned by merge_coverage; NEVER a required gate (compute_coverage_gaps untouched). +4 cargo-portable proofs (CLI flag default/parse; unit_divergence off⇒None and on⇒accept_reject on a synthetic accept(verilator)+reject(yosys); merge_coverage unions the fact; summarize_coverage lights it only from a diverged report). Default-off / DUT byte-identical. README/USER_GUIDE/book/KM closeout is .2f per the pre-split.`
  Verification: `cargo check --all-targets OK; cargo fmt --all --check OK; cargo clippy --all-targets -- -D warnings OK (all under scripts/ram_guard.sh --threshold 90). cargo test --bin tool_matrix 75/0 (1 ignored; +4 new proofs); full cargo test --lib 531/0 (2 ignored; lib untouched). tests/snapshots.rs 6/6 byte-identical + tests/book_examples.rs 3/3. Real-tool smoke /tmp/anvil-divergence-col-smoke (Verilator 5.046 + Yosys 0.64 both modes): 17 scenarios / 17 modules, 17/0 Verilator + both Yosys; divergence_enabled=true, divergence_subset=2 axes, each subset unit carries 3 labelled accept verdicts with diverged=false, saw_acceptance_divergence=false — the all-agree steady state, exactly as designed.`
  Commit: `this ACCEPTANCE-DIVERGENCE-HUNTING.2c.2 commit`

- ID: `ACCEPTANCE-DIVERGENCE-HUNTING.2d`
  Status: `pending`
  Goal: `The MCP divergence controlled tool wired into src/mcp dispatcher: input schema (lane/seed/seeds/config/tools/yosys_mode), DivergenceReport result, divergent-run_id cache population (so anvil://artifact/<run_id>/{sv,introspection} resolve), a top-level divergence audit record; the anvil CLI shim; book/src/agent-mcp.md tool list/table; proofs (decision-0017 API-completeness gate met).`
  Acceptance: `A controlled divergence tool in tools_list + tools_call shimming divergence::run; DivergenceReport returned as JSON; each divergent run_id cached; a top-level divergence audit record; CLI shim (a --divergence flag); book agent-mcp updated; no introspection schema bump unless a DivergenceReport projection is served as a resource; cargo-portable proofs; cargo check/test/clippy/fmt green; snapshots 6/6 + book_examples unchanged.`
  Verification: `pending`
  Commit: `pending`

- ID: `ACCEPTANCE-DIVERGENCE-HUNTING.2e`
  Status: `pending`
  Goal: `The tool-version-vs-version axis: DivergenceOptions.tool_specs carrying (AcceptanceTool kind, binary, label) so two versions of the same tool kind are run + compared (kind stays allow-listed; binary is a caller-supplied path/PATH shim); a ToolInvocation observed-version capture (parsed from --version); the version_mismatch classification. Portability note (caller supplies binaries/labels; ANVIL never manages installs). Default-off / DUT byte-identical.`
  Acceptance: `DivergenceOptions.tool_specs + ToolInvocation.version + the version_mismatch divergence kind; a same-kind two-binary run produces two labelled verdicts and classifies version_mismatch on disagreement; cargo-portable proofs (synthetic same-kind differing verdicts); cargo check/test/clippy/fmt green; snapshots 6/6 byte-identical.`
  Verification: `pending`
  Commit: `pending`

- ID: `ACCEPTANCE-DIVERGENCE-HUNTING.2f`
  Status: `pending`
  Goal: `A real-tool end-to-end gate (#[ignore], tool-gated) proving the divergence matrix is produced + correctly classified + queryable (the all-agree steady state records diverged=false; a synthetic-injected accept/reject pair classifies accept_reject) and the book/USER_GUIDE/README/KM closeout; close .2 and the tree.`
  Acceptance: `A #[ignore] tool-gated tests/divergence_e2e.rs (tool-less ⇒ skips green) proving an all-agree real-tool run records diverged=false + the report is queryable; book/src/synthesizability.md + book/src/agent-mcp.md + USER_GUIDE + README updated; a KM how-to card; ROADMAP lane 2 marked delivered; the tree + .2 + root closed; cargo check/test/clippy/fmt green incl. snapshots 6/6 + book_examples.`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `ACCEPTANCE-DIVERGENCE-HUNTING.2d` | `pending` | The MCP `divergence` controlled tool (shimming `divergence::run`, caching each divergent `run_id`, a top-level audit record) + the `anvil` CLI `--divergence` shim (decision `0017` API-completeness gate). |
| 2 | `ACCEPTANCE-DIVERGENCE-HUNTING.2e` | `pending` | The tool-version-vs-version axis (`DivergenceOptions.tool_specs` + `ToolInvocation.version` + `version_mismatch`). |
| 3 | `ACCEPTANCE-DIVERGENCE-HUNTING.2f` | `pending` | The real-tool e2e gate + book/USER_GUIDE/README/KM closeout; closes the tree. |

(`.1` design ADR `done 2026-06-17` — decision `0019`. `.2a` shared `downstream::tool_verdict` extract `done`. `.2b` `src/divergence/` core `done`. `.2c` pre-split → `.2c.1` (hunt fold) `done 2026-06-17` + `.2c.2` (the `tool_matrix --divergence` column) `done 2026-06-17` — `ModuleReport`/`DesignReport.divergence` via the same `classify_report` over the tools the matrix already ran, the opportunistic `saw_acceptance_divergence` fact, the `classify_diff_sim_axis` subset reuse; matrix-bin 75/0, lib 531/0, snapshots 6/6 + book_examples, real-tool smoke clean; **`.2c` closed**. Frontier `.2d` (the MCP `divergence` tool + CLI shim).)

## Decisions

- `2026-06-17`: Registered as an owner-directed usability lane (idea 2). Binds
  decision [`0017`](../decisions/0017-api-first-everything-mcp-accessible.md).
  Complements `DIFFERENTIAL-SIMULATION` (cross-sim trace agreement) with
  accept/reject divergence; design-first ADR before code.
- `2026-06-17` (`.1` done): Recorded decision
  [`0019`](../decisions/0019-acceptance-divergence-hunting.md). Acceptance
  divergence is a **first-class, default-off, SCHEMA-DERIVED detector** with one
  shared library home (`src/divergence/`'s `divergence::run`) reused by three
  surfaces — a `hunt::run` detection axis (`acceptance_divergence`), a `tool_matrix`
  column, and a controlled MCP `divergence` tool — so there is **one** detector and
  no drift (resolves the "hunt vs matrix" open question to **both**). A per-tool
  verdict is a trinary accept/warn/reject projection of one `ToolInvocation` (the
  classifier extracted from `hunt::run` into `downstream::tool_verdict` — no second
  classifier). A divergence is "not all labelled-tool verdicts equal"
  (accept_reject | accept_warn | warn_reject | version_mismatch), so `--yosys-mode
  both` can itself diverge. Reproducers reuse `write_bundle` / the `tool_matrix`
  `.sv`+log retention (no new format). `saw_acceptance_divergence` is opportunistic,
  **never a required gate** (all-agree is the valid-by-construction steady state).
  Sandbox path caller-set (decision `0004`); default-off / DUT byte-identical.
  Pre-split `.2` into `.2a`…`.2f`.

## Open Questions

- ~~Tool-version-vs-version divergence: how versions are pinned/selected portably
  (PATH shims vs. explicit binaries).~~ **Resolved at `.1`**: a labelled tool is
  `(AcceptanceTool kind, resolved binary, observed version)`; the **kind stays
  allow-listed** (`AcceptanceTool::from_name`), but the *binary* may be a
  caller-supplied path/PATH shim for that kind. ANVIL never manages tool installs —
  the caller supplies the binaries and labels. The version axis is a *later*
  increment (`.2e`); the first cut (`.2b`/`.2c`) is multi-tool same-version.
- ~~Whether divergence detection rides the `BUG-HUNT-ORCHESTRATION` loop or is an
  independent `tool_matrix` column.~~ **Resolved at `.1`**: **both**, via one shared
  `divergence::run` — a `hunt::run` axis (`HuntRequest.divergence`) **and** a
  `tool_matrix` column **and** an MCP tool, so there is exactly one detector and no
  drift (decision `0017`: MCP-invocable + queryable + CLI a shim).

## Blockers

- None. (Synergistic with `BUG-HUNT-ORCHESTRATION` and
  `DOWNSTREAM-ADAPTER-EXPANSION`; not blocked by them.)

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-06-17` | `ACCEPTANCE-DIVERGENCE-HUNTING` | `tree registered (docs-only); no code` | `registered` |
| `2026-06-17` | `ACCEPTANCE-DIVERGENCE-HUNTING.1` | `decision 0019 + INDEX + DEVELOPMENT_NOTES + MEMORY + CHANGES + docs/TASK_TREE row + ROADMAP lane-2 note; check_memory_architecture OK; KM gen+check OK (47→48 facts); docs-only (no src/) ⇒ DUT byte-identical` | `done` |
| `2026-06-17` | `ACCEPTANCE-DIVERGENCE-HUNTING.2a` | `downstream::tool_verdict + ToolVerdict{Accept,Warn,Reject} extracted; hunt::classify_detection derives from it (byte-identical); +1 downstream proof; cargo check/clippy/fmt green; cargo test --lib 522/0 (downstream:: 21/0 + hunt:: 11/11); snapshots 6/6 byte-identical` | `done` |
| `2026-06-17` | `ACCEPTANCE-DIVERGENCE-HUNTING.2b` | `src/divergence/mod.rs + pub mod divergence; DivergenceOptions/ToolDecision/Divergence/DivergenceReport + divergence::run reuses downstream::validate + the shared tool_verdict + classify_divergences (accept_reject/accept_warn/warn_reject, deterministic); 7 cargo-portable proofs; cargo check/clippy/fmt green; cargo test --lib divergence:: 7/7 + full lib 529/0; snapshots 6/6 byte-identical` | `done` |
| `2026-06-17` | `ACCEPTANCE-DIVERGENCE-HUNTING.2c.1` | `divergence::classify_report extracted (run refactored, byte-identical); hunt::run gains HuntRequest.divergence + HuntFailure.divergence — a finding where some tool accepted while another rejected/warned is classified acceptance_divergence (no minimize, like cross_sim); mcp/main set divergence:false (arg-wiring=.2d); +2 proofs; cargo check/clippy/fmt green; cargo test --lib divergence:: 8/8 + hunt:: 12/12 + full lib 531/0; snapshots 6/6 + book_examples 3/3` | `done` |
| `2026-06-17` | `ACCEPTANCE-DIVERGENCE-HUNTING.2c.2` | `tool_matrix --divergence column: ModuleReport/DesignReport.divergence via the shared classify_report over a ValidateReport assembled from the tools the matrix already ran (no extra spawn, no tool-clean precondition); .divergence-subset sentinel reusing select_diff_sim_subset/classify_diff_sim_axis (membership factored to scenario_in_named_subset); opportunistic saw_acceptance_divergence fact (never a gate, compute_coverage_gaps untouched); +4 cargo-portable proofs; cargo check/clippy/fmt green; cargo test --bin tool_matrix 75/0 + full lib 531/0; snapshots 6/6 + book_examples 3/3; real-tool smoke /tmp/anvil-divergence-col-smoke 17/0 (divergence_enabled, subset=2 axes, diverged=false, saw_acceptance_divergence=false)` | `done` |
| `2026-06-17` | `ACCEPTANCE-DIVERGENCE-HUNTING.2c` | `both children (.2c.1 hunt fold + .2c.2 matrix column) done ⇒ .2c closed; one shared detector reused by two surfaces, no drift` | `done` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `ACCEPTANCE-DIVERGENCE-HUNTING` | `USABILITY-LANE-OWNERSHIP.1 — register 7 owner-directed usability/capability lanes + API-first decision 0017` | Tree registered (not yet started); frontier `.1` (design ADR) pending. |
| `ACCEPTANCE-DIVERGENCE-HUNTING.1` | `ACCEPTANCE-DIVERGENCE-HUNTING.1 — design ADR (decision 0019): acceptance-divergence detector (accept/warn/reject verdicts + classifier) shared by the hunt loop, tool_matrix, and MCP` | Design/decision leaf (docs-only). Pins the verdict/classifier/report, the three surfaces (one shared `divergence::run`), the reproducer reuse, the version axis, the honesty boundary; pre-splits `.2` into `.2a`…`.2f`. DUT byte-identical. |
| `ACCEPTANCE-DIVERGENCE-HUNTING.2a` | `ACCEPTANCE-DIVERGENCE-HUNTING.2a — extract shared downstream::tool_verdict accept/warn/reject classifier from hunt::run` | First code leaf. `ToolVerdict` enum + `tool_verdict` in `src/downstream/mod.rs`; `hunt::classify_detection` derives from it (byte-identical). The one accept/warn/reject classifier `.2b`'s divergence detector reuses. Default-off / DUT byte-identical (snapshots 6/6). |
| `ACCEPTANCE-DIVERGENCE-HUNTING.2b` | `ACCEPTANCE-DIVERGENCE-HUNTING.2b — src/divergence/ library core (divergence::run + DivergenceReport, reusing validate + the shared classifier)` | New `src/divergence/mod.rs` + `pub mod divergence`. `divergence::run` reuses the one `downstream::validate` orchestration + the shared `tool_verdict`, classifies disagreement (`accept_reject`/`accept_warn`/`warn_reject`, deterministic). 7 cargo-portable proofs; lib 529/0; snapshots 6/6. No CLI/MCP/version axis yet. Default-off / DUT byte-identical. |
| `ACCEPTANCE-DIVERGENCE-HUNTING.2c.1` | `ACCEPTANCE-DIVERGENCE-HUNTING.2c.1 — fold the acceptance-divergence detector into hunt::run (classify_report + HuntRequest.divergence)` | `divergence::classify_report` (the pure projection of an already-run `ValidateReport`) + `hunt::run`'s `HuntRequest.divergence`/`HuntFailure.divergence`: a finding where one tool accepted what another rejected/warned is classified `acceptance_divergence` (no minimize). MCP/CLI set `divergence:false` (arg-wiring=`.2d`). lib 531/0; snapshots 6/6 + book_examples. Default-off / DUT byte-identical. |
| `ACCEPTANCE-DIVERGENCE-HUNTING.2c.2` | `ACCEPTANCE-DIVERGENCE-HUNTING.2c.2 — the tool_matrix --divergence column (ModuleReport/DesignReport.divergence via the shared classify_report)` | The second surface: `tool_matrix --divergence` + `ModuleReport`/`DesignReport.divergence: Option<DivergenceReport>` populated by `unit_divergence` — the same `divergence::classify_report` over the tools the matrix already ran (a pure projection; no extra spawn / no tool-clean precondition). `.divergence-subset` sentinel reuses `select_diff_sim_subset`/`classify_diff_sim_axis` (membership factored to `scenario_in_named_subset`). Opportunistic `saw_acceptance_divergence` fact, never a gate. +4 proofs; matrix-bin 75/0, lib 531/0; snapshots 6/6 + book_examples; real-tool smoke clean. Closes `.2c`. Default-off / DUT byte-identical. |

## Changelog

- `2026-06-17`: Created task tree (registration via `USABILITY-LANE-OWNERSHIP.1`).
- `2026-06-17`: `.1` done — recorded decision `0019` (the acceptance-divergence
  detector design). Acceptance divergence = a default-off, SCHEMA-DERIVED detector
  with one shared home (`src/divergence/`'s `divergence::run`) reused by three
  surfaces (a `hunt::run` axis, a `tool_matrix` column, an MCP `divergence` tool);
  a per-tool accept/warn/reject verdict is a projection of one `ToolInvocation` via
  the extracted shared `downstream::tool_verdict`; a divergence is "not all
  labelled-tool verdicts equal". Reproducers reuse `write_bundle` / the matrix
  `.sv`+log retention; the version-vs-version axis is the later `.2e`;
  `saw_acceptance_divergence` is opportunistic (never a required gate — all-agree is
  the valid-by-construction steady state). Pre-split `.2` into `.2a` (classifier
  extract), `.2b` (`src/divergence/` core), `.2c` (hunt fold + matrix column), `.2d`
  (MCP tool + CLI), `.2e` (version axis), `.2f` (real-tool gate + docs). Frontier
  advanced to `.2a`. Docs-only / DUT byte-identical.
- `2026-06-17`: `.2a` done — extracted the inline accept/warn/reject classifier from
  `hunt::run` into a shared `pub fn downstream::tool_verdict(&ToolInvocation) ->
  ToolVerdict` (+ a `ToolVerdict{Accept,Warn,Reject}` snake_case-serde enum), the
  single accept/warn/reject definition `.2b`'s `divergence::run` will reuse (no
  second classifier; `feedback_full_factorization`). `hunt::classify_detection` now
  derives from it — byte-identical (`Warn` ⇒ `"warning"`, else ⇒ `"reject"`;
  `first_failing_tool` only yields a non-succeeding invocation, so `Accept` is
  unreachable and treated as reject defensively). +1 `downstream` proof; the `hunt`
  `classify_detection` proof passes unchanged. `cargo test --lib` 522/0 (downstream::
  21/0, hunt:: 11/11); snapshots 6/6 byte-identical; clippy/fmt green. First code
  leaf; default-off / DUT byte-identical. Frontier advanced to `.2b` (the
  `src/divergence/` library core).
- `2026-06-17`: `.2b` done — the `src/divergence/` library core. New
  `src/divergence/mod.rs` (`pub mod divergence`): `DivergenceOptions { validate:
  ValidateOptions }` (the `MinimizeOptions` wrap precedent), `ToolDecision` (the
  per-tool labelled record carrying a `downstream::ToolVerdict`), `Divergence
  { kind, tools }`, `DivergenceReport`, and `divergence::run(seed, cfg,
  &DivergenceOptions) -> DivergenceReport`. `run` **reuses** the one hardened
  `downstream::validate` orchestration (which already runs every enabled tool/mode
  to completion — no short-circuit on reject) and projects its per-tool invocations
  via the shared `downstream::tool_verdict` (no second sandbox loop / no second
  classifier; full-factorization), then `classify_divergences` emits one
  `Divergence` per present pair-class (`accept_reject`/`accept_warn`/`warn_reject`),
  deterministic. 7 cargo-portable proofs (incl. the synthetic accept/reject set ⇒
  `accept_reject`, all-three-pair-classes, the Yosys-mode divergence, the no-tools
  friendly no-op, the serde round-trip). `cargo test --lib` 529/0; snapshots 6/6
  byte-identical; clippy/fmt green. No CLI/MCP yet (`.2c`/`.2d`); no version axis
  (`.2e`). Default-off / DUT byte-identical. Frontier advanced to `.2c` (the
  `hunt::run` fold + the `tool_matrix` column).
- `2026-06-17`: pre-split `.2c` into `.2c.1` (the `hunt::run` fold) + `.2c.2` (the
  `tool_matrix` column). `.2c.1` done — extracted
  `divergence::classify_report(&ValidateReport)` (the pure projection half of `run`;
  `run` refactored to use it, byte-identical) so the hunt classifies the tools
  `validate` **already ran** without re-validating. `hunt::run` gains
  `HuntRequest.divergence: bool` + `HuntFailure.divergence: Option<DivergenceReport>`;
  on the finding path (`!report.ok`), a result where some tool accepted what another
  rejected/warned refines `detection` to `"acceptance_divergence"` and attaches the
  report — and is **not** minimized (the `validate` oracle can't preserve a cross-tool
  disagreement, the `cross_sim_mismatch` rationale). The MCP `run_hunt` + CLI
  `build_hunt_request` set `divergence: false` (the arg-wiring is `.2d`,
  byte-identical until then). +2 proofs (`classify_report_projects_a_validate_report`,
  `divergence_flag_is_inert_on_a_clean_sweep`). `cargo test --lib` 531/0 (divergence::
  8/8, hunt:: 12/12); snapshots 6/6 byte-identical + book_examples 3/3; clippy/fmt
  green. Default-off / DUT byte-identical. Frontier advanced to `.2c.2` (the
  `tool_matrix` column).
- `2026-06-17`: `.2c.2` done — the `tool_matrix --divergence` column (the second of
  the one-detector-two-surfaces pair; closes `.2c`). `src/bin/tool_matrix.rs` gains
  the `--divergence` opt-in flag, `MatrixReport.divergence_enabled`/`divergence_subset`
  header fields, `ModuleReport.divergence` + `DesignReport.divergence:
  Option<DivergenceReport>` (the `diff_sim` column precedent), and `unit_divergence` —
  which gathers the verilator/yosys/iverilog `ToolInvocation`s the matrix **already
  ran** on a unit, assembles a `ValidateReport`, and classifies it through the **one**
  shared `divergence::classify_report` (no second classifier). It spawns **no** extra
  tool and (unlike `--diff-sim`) has **no** tool-clean precondition — a divergence is
  most interesting when one tool rejects what another accepts. Gated by a
  `.divergence-subset` sentinel reusing `select_diff_sim_subset`/`classify_diff_sim_axis`
  (the shared membership check factored into `scenario_in_named_subset`). The
  **opportunistic** `saw_acceptance_divergence` fact (lit by
  `summarize_coverage`/`summarize_design_coverage` from a `diverged` unit, unioned by
  `merge_coverage`) is **never** a required gate — `compute_coverage_gaps` is untouched,
  because all-agree is the valid-by-construction steady state. +4 cargo-portable proofs.
  `cargo test --bin tool_matrix` 75/0 (1 ignored); full `cargo test --lib` 531/0;
  snapshots 6/6 byte-identical + book_examples 3/3; clippy/fmt green. Real-tool smoke
  `/tmp/anvil-divergence-col-smoke` (Verilator 5.046 + Yosys 0.64 both modes): 17/0,
  `divergence_enabled=true`, subset 2 axes, each subset unit 3× `accept` `diverged=false`,
  `saw_acceptance_divergence=false` — the all-agree steady state. Default-off / DUT
  byte-identical. `.2c` closed; frontier advanced to `.2d` (the MCP `divergence` tool +
  CLI shim). README/USER_GUIDE/book/KM closeout is `.2f`.
