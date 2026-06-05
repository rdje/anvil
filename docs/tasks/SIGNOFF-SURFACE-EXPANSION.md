# SIGNOFF-SURFACE-EXPANSION: Broader Signoff Surfaces

## Metadata

- Tree ID: `SIGNOFF-SURFACE-EXPANSION`
- Status: `active`
- Roadmap lane: `Quality / signoff-level downstream confidence`
- Created: `2026-06-05`
- Last updated: `2026-06-05`
- Owner: repo-local workflow

## Goal

Exhaust the next practical signoff-surface expansions: richer CDC
primitive coverage, richer AST/source extractor parity, broader
simulator/tool parity, and larger but resource-aware regression sweeps.

## Non-Goals

- No LLM/VLM or SpecForge-specific capability.
- No tool gate that assumes a commercial/proprietary tool is present.
- No full-suite run without RAM monitoring and the 90% danger-zone
  stop rule.
- No user-facing claim that is not backed by a repo-owned check or a
  clearly marked optional external-tool gate.

## Acceptance Criteria

- Every source edit is owned by a leaf before it occurs.
- At least one richer signoff axis lands with tests and documentation,
  or the current environment/tooling blocker is recorded.
- Existing `tool_matrix`, diff-sim, mdBook example, and snapshot
  contracts remain aligned.
- Any new user-facing gate or CLI option is documented in `USER_GUIDE.md`
  and the mdBook with meaningful examples.
- Each completed leaf is committed through `COMMIT.md`.
- The tree closes only when all listed signoff axes are landed or
  explicitly deferred with evidence.

## Task Tree

- ID: `SIGNOFF-SURFACE-EXPANSION`
  Status: `active`
  Goal: `Broaden downstream and signoff confidence surfaces.`
  Children: `SIGNOFF-SURFACE-EXPANSION.1`, `SIGNOFF-SURFACE-EXPANSION.2`, `SIGNOFF-SURFACE-EXPANSION.3`, `SIGNOFF-SURFACE-EXPANSION.4`

- ID: `SIGNOFF-SURFACE-EXPANSION.1`
  Status: `done`
  Goal: `Add the next CDC primitive or record the concrete proof/tooling blocker.`
  Acceptance: `A CDC primitive beyond the existing 2-flop synchronizer lands with generation, metrics, matrix coverage, and docs, or a blocker records why the next primitive is not yet safe.`
  Verification: `focused cargo/config/matrix tests, snapshots, book_examples, mdBook, clippy, check, Knowledge Map, memory architecture, focused 17-scenario tool_matrix smoke clean; full cargo test monitored and stopped at 90.7% RAM per policy.`
  Commit: `SIGNOFF-SURFACE-EXPANSION.1 - add N-flop CDC synchronizer`

- ID: `SIGNOFF-SURFACE-EXPANSION.2`
  Status: `done`
  Goal: `Add richer AST/source extractor parity where available.`
  Acceptance: `A richer optional frontend AST/source extractor path lands with scoped facts, or tool availability/scope blockers are recorded.`
  Verification: `frontend parity portable suite, optional real Yosys gate, optional real Verilator JSON gate, check, clippy, fmt, mdBook, book examples, snapshots, Knowledge Map, memory architecture, diff whitespace all clean.`
  Commit: `SIGNOFF-SURFACE-EXPANSION.2 - add Verilator JSON frontend parity`

- ID: `SIGNOFF-SURFACE-EXPANSION.3`
  Status: `pending`
  Goal: `Broaden simulator/tool parity beyond the current matrix where practical.`
  Acceptance: `A new optional parity axis or larger resource-aware sweep lands, with RAM-monitoring policy observed for any full-suite run.`
  Verification: `pending`
  Commit: `pending`

- ID: `SIGNOFF-SURFACE-EXPANSION.4`
  Status: `pending`
  Goal: `Close the signoff-surface frontier.`
  Acceptance: `The tree records landed axes, optional-gate boundaries, deferred tool blockers, and an empty frontier.`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `SIGNOFF-SURFACE-EXPANSION.3` | `pending` | CDC and richer frontend AST parity are now covered; broader simulator/tool parity or resource-aware sweeps are the next signoff surface. |

## Decisions

- `2026-06-05`: Keep all richer tool integrations optional and
  repo-portable. ANVIL-specific signoff work must not import SpecForge,
  docling, LLM, or VLM assumptions.
- `2026-06-05`: The next CDC primitive is the N-flop 1-bit
  synchronizer, not async FIFO or handshake. It is a safe extension of
  the existing by-construction 2-flop lane: `cdc_synchronizer_stages`
  defaults to `2`, validates `>= 2`, and values `>= 3` generate longer
  destination-domain chains. Multi-bit CDC fabrics remain separate
  future trees.
- `2026-06-05`: Use Verilator JSON, not Verilator XML, for the richer
  Phase-8 frontend parity follow-up in this environment. Local
  Verilator 5.046 rejects `--xml-only` but supports `--json-only`;
  `slang` is absent. The new gate stays optional and harness-local.

## Open Questions

- None for the current frontier.

## Blockers

- None.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-06-05` | `SIGNOFF-SURFACE-EXPANSION.2` | `cargo test --test frontend_parity -- --nocapture`; `cargo test --test frontend_parity -- --ignored parity_against_real_yosys_hierarchy_write_json --nocapture`; `cargo test --test frontend_parity -- --ignored parity_against_real_verilator_json_frontend_ast --nocapture`; `cargo check --all-targets`; `cargo clippy --all-targets -- -D warnings`; `cargo fmt --all --check`; `mdbook build book`; `cargo test --test book_examples`; `cargo test --test snapshots`; `knowledge-map/scripts/check_knowledge_map.sh`; `scripts/check_memory_architecture.sh`; `git diff --check`. | Clean. Portable frontend suite: 15 passed / 3 ignored. Real Yosys gate clean across 5 seeds. Real Verilator JSON gate clean across 5 seeds with artifacts in `target/tmp/frontend-parity-signoff-verilator-json` and all 7 Phase-8 manifest categories enforced. Full `cargo test` intentionally not rerun after the prior monitored resource stop at 90.7% RAM. |
| `2026-06-05` | `SIGNOFF-SURFACE-EXPANSION.1` | `cargo check --all-targets`; `cargo test -q synchronizer`; `cargo test -q --bin tool_matrix coverage_gaps_detect_missing_categories`; `cargo test -q --bin tool_matrix phase1_gate_raises_modules_per_scenario_to_cover_1000_modules`; `cargo test -q --bin tool_matrix phase1_gate_preserves_larger_explicit_module_count`; `cargo test -q --bin tool_matrix build_default_scenarios_includes_multi_clock_scenario`; `cargo test -q --bin tool_matrix summarize_coverage_lights_multi_clock_facts_from_module_metrics`; `cargo test -q validate_rejects_cdc`; `cargo test -q --bin tool_matrix merge_coverage_unions_saw_cdc_nflop_synchronizer`; `cargo test -q --bin tool_matrix diff_sim_subset_against_default_scenarios_is_nonempty_and_capped`; `cargo clippy --all-targets -- -D warnings`; `cargo fmt --all --check`; `mdbook build book`; `cargo test --test book_examples`; `cargo test --test snapshots`; `knowledge-map/scripts/check_knowledge_map.sh`; `scripts/check_memory_architecture.sh`; `git diff --check`; focused `cargo run --bin tool_matrix -- --out /tmp/anvil-signoff-surface-nflop-r1 --fail-on-coverage-gap --yosys-mode without-abc`; monitored `cargo test` attempt. | Focused checks clean. Focused matrix: 17 scenarios / 17 modules, `coverage_gaps=[]`, Verilator 17/0, Yosys without-abc 17/0, `saw_multi_clock_design=true`, `saw_cdc_2_flop_synchronizer=true`, `saw_cdc_nflop_synchronizer=true`. Full `cargo test` was stopped at 90.7% RAM per owner resource policy. |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `SIGNOFF-SURFACE-EXPANSION.2` | `SIGNOFF-SURFACE-EXPANSION.2 - add Verilator JSON frontend parity` | Lands optional Verilator JSON-AST frontend extractor, full-scope real-tool gate, docs, and Knowledge Map fact. |
| `SIGNOFF-SURFACE-EXPANSION.1` | `SIGNOFF-SURFACE-EXPANSION.1 - add N-flop CDC synchronizer` | Lands `cdc_synchronizer_stages`, N-flop generation, metrics, matrix coverage, user docs, and Knowledge Map fact. |

## Changelog

- `2026-06-05`: Created task tree and opened
  `SIGNOFF-SURFACE-EXPANSION.1`.
- `2026-06-05`: Landed `SIGNOFF-SURFACE-EXPANSION.1`; frontier moves
  to `.2`.
- `2026-06-05`: Landed `SIGNOFF-SURFACE-EXPANSION.2`; frontier moves
  to `.3`.
