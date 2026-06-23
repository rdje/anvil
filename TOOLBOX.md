# ANVIL Toolbox

The catalog of **ANVIL's own diagnostic instruments** — the tools ANVIL ships to
*pinpoint issues ANVIL may have* — plus the **acceptance-checklist template** every
code change must satisfy before it commits.

This file is part of the doctrine-enforcement kit (`DOCTRINE_ENFORCEMENT.md`). The
`CODE-CHANGE-EVIDENCE` doctrine (`scripts/check_diagnosis_evidence.sh`) references the
checklist in Part 2; the acceptance gate itself is the `COMMIT.md` workflow.

> When a generated artifact misbehaves — a downstream tool rejects it, a flop drives the
> wrong thing, an output looks trivial, a knob seems to fire by chance, two tools
> disagree — reach for the matching instrument below **first**, before reading `src/`.
> ANVIL is built to be introspected: every fact about a run is queryable, and every
> finding reduces to a minimal, reproducible seed+knobs bundle.

---

## Part 1 — ANVIL's diagnostic toolbox

All instruments are deterministic for a fixed `(seed, knobs)` and emit no wall-clock /
thread / colour noise, so their output is itself re-checkable evidence.

### 1. Construction introspection — *"why did the generator build this?"*

| Instrument | Pinpoints | Invocation | Output |
|---|---|---|---|
| `--trace <low\|medium\|high\|debug>` (`--trace-file F`) | the exact construction path: strategy chosen, phase transitions, per-cone/per-frame events, motif dispatch, terminal picks, anti-collapse rollbacks, every `pick_gate` / intern / depth/width/NodeId (`debug`) | `anvil --seed 42 --trace high` | deterministic trace to stderr (or `--trace-file`) |
| `--dump-config` | the *effective* knobs after `default → --config → --profile → explicit → seed` resolution — the first thing to check when a run behaves unexpectedly | `anvil --seed 42 --profile deep-hierarchy --dump-config` | effective `Config` as JSON |
| `--metrics` | post-hoc structural telemetry (35+ per-module counters + hierarchy metrics) — did the knob actually fire? how many flops / shared nodes / emitted functions? | `anvil --seed 42 --metrics` | metrics JSON to stderr |

### 2. Structural / semantic introspection — *"what does the emitted IR actually depend on?"*

| Instrument | Pinpoints | Invocation | Output |
|---|---|---|---|
| `--introspect` (schema `1.22`) | the SCHEMA-DERIVED projection of `Config`/`Metrics`/`DesignMetrics` + a content-addressed `run_id` + the `coverage_readout` (per-knob/per-category achieved fire rates + gate/operand/depth histograms) | `anvil --seed 42 --introspect` | introspection JSON to stdout (single-artifact run; `docs/AGENT_INTROSPECTION_SCHEMA.md`) |
| MCP `analyze` | derived-relation queries over the DUT IR (nine kinds): `output_support` (an output's transitive support cone), `input_reach` (its dual fan-out), `flop_reset_provenance` (per-flop reset/data provenance), `module_reachability` (which modules a design reaches from the top), `flop_dependencies` (per-flop register-to-register predecessors/successors + self-feedback flag), `memory_provenance` (per-inferrable-memory shape + its read/write-port support cones), `fsm_provenance` (per-generated-FSM shape + its transition-select `sel` support cone), `node_drivers` (per-node immediate 1-hop driver adjacency + `GateOp`), `node_readers` (its transpose — per-node immediate 1-hop reader adjacency) | `anvil-mcp` tool `analyze` `{query, target}` | derived-relation JSON (no new computed truth) |
| MCP `coverage` / `coverage_gaps` | the achieved-coverage readout (same as `--introspect`'s `coverage_readout`) and the recorded `tool_matrix` gap list | `anvil-mcp` tools `coverage` / `coverage_gaps` | coverage JSON |

### 3. Downstream acceptance — *"does a tool reject ANVIL's output, and which one?"*

| Instrument | Pinpoints | Invocation | Output |
|---|---|---|---|
| `validate` (CLI/MCP) | per-tool accept/warn/reject verdicts through the hardened allow-list runner (`verilator`/`yosys`/`iverilog`/`sv2v`/`slang`) + the `tool_verdict` classifier | `anvil-mcp` tool `validate` `{sv, tools}` | per-tool `ToolReport` |
| `tool_matrix` | the repo-owned scenario sweeps and gates: `--phase1..4-gate`, the structured-surface `--function-emit-gate` / `--generate-loop-gate` / `--task-emit-gate` / `--cone-function-gate` / `--multi-output-task-gate`, `--signoff-knob-sweep-gate`, `--sv-version-gate`; `--yosys-mode <without-abc\|with-abc\|both>`, `--iverilog-compile`, `--sv2v`, `--slang` | `cargo run --bin tool_matrix -- --out ./tm --phase4-hierarchy-gate --yosys-mode both` | per-module/-design pass-fail + `tool_matrix_report.json` (`coverage_gaps = []` is the exit criterion) |
| `tool_matrix --diff-sim` | **semantic** disagreement: cross-simulator trace mismatch (iverilog ↔ verilator), not just acceptance | `cargo run --bin tool_matrix -- --diff-sim --out ./tm` | per-DUT `diff_sim` field + `saw_design_with_cross_simulator_agreement` |
| `divergence` (CLI/MCP; `tool_matrix --divergence`) | **acceptance** disagreement: one tool accepts while another warns/rejects valid-by-construction RTL | `anvil-mcp` tool `divergence` / `anvil hunt --divergence` | a `DivergenceReport` (accept_reject / accept_warn / warn_reject classes) |

### 4. Reduce a finding to a minimal reproducer — *"what is the smallest failing case?"*

| Instrument | Pinpoints | Invocation | Output |
|---|---|---|---|
| `minimize` (CLI/MCP) | the minimal still-failing RTL for a rejected artifact (budgeted shrink) | `anvil-mcp` tool `minimize` `{sv, tools, budget}` | minimized `.sv` + trace |
| `anvil hunt` | the turnkey loop: fuzz a deterministic seed sweep → detect (reject/warn, optional `--diff-sim` / `--divergence`) → auto-minimize → emit a self-contained reproducer bundle | `anvil hunt --seeds 100 --tools verilator,yosys --yosys-mode both --out ./bundle` | a JSON `HuntReport` + per-finding reproducer bundle (`--out`) |
| `manifest.json` | the seed + **effective knobs** that produced any `--out` artifact — the reproduction key to attach to every bug report | written per `--out` run | `manifest.json` (per-module / per-design) |
| MCP `anvil://artifact/<run_id>/{sv,introspection}` | the cached artifacts for a hunt finding's `run_id`, served as resources | `anvil-mcp` resource read | the `.sv` / introspection doc |

### 5. Reproducibility & resource safety — *"is the output still byte-stable, and is the run bounded?"*

| Instrument | Pinpoints | Invocation | Output |
|---|---|---|---|
| `tests/snapshots.rs` (`insta`) | a *real* change in generated SystemVerilog for a canonical `(seed, config)` — the byte-identical contract | `cargo test --test snapshots` (or `cargo insta test`) | snapshot pass/fail; an intended change is a deliberate `cargo insta accept` in the same slice (`COMMIT.md` INSTA-SNAPSHOTS protocol) |
| `--max-rss-mb <MiB>` / `--ram-abort-pct <1..=100>` | runaway memory inside ANVIL's own process — abort an `--out` run cleanly (exit `99` + seed+knobs on stderr) once RSS / host RAM% crosses the ceiling | `anvil --seed 42 --count 1000 --out ./g --max-rss-mb 8192` | clean deterministic abort; never changes emitted RTL |
| `scripts/ram_guard.sh` | runaway memory in an *external* job (a heavy `cargo test` / matrix sweep) — kill it before the host thrashes | `scripts/ram_guard.sh --threshold 90 -- cargo test` | guarded run (note the `--` separator) |

---

## Part 2 — The acceptance checklist a code change must satisfy

Mirror of the `COMMIT.md` non-negotiable checklist, expressed as **earned, not ticked**
boxes (`DOCTRINE_ENFORCEMENT.md` §6.1): every box cites a **named, re-runnable oracle**,
so the gate (CI / the local `COMMIT.md` run) can re-execute exactly that and earn the
box independently of the tick. A self-ticked-but-false box dies at the oracle re-run.

A change is a **code change** if it stages `src/`, `tests/`, `examples/`, `build.rs`, or
a behaviour-altering `Cargo.toml`/`Cargo.lock`. Pure docs / workflow commits are exempt
from the code-only boxes (the scope-aware checks pass them through).

- [ ] **CODE HYGIENE** — oracle: `cargo check --all-targets` · `cargo clippy
  --all-targets -- -D warnings` · `cargo fmt --all --check` all green.
- [ ] **NO UNINTENDED DRIFT (byte-identical)** — oracle: `cargo test` incl.
  `tests/snapshots.rs`. A snapshot change is *either* a bug (fix the cause, do not touch
  the `.snap`) *or* an intended output change accepted via `cargo insta accept` **in the
  same slice** that caused it.
- [ ] **DOWNSTREAM-CLEAN** (only if generator output changed) — oracle: the relevant
  `tool_matrix --<surface>-gate` / `--phase*-gate` reports `coverage_gaps = []` with the
  tool columns clean; or, for a focused change, a seed spot-check (`verilator
  --lint-only` + `yosys -p "read_verilog -sv …; synth -noabc"`). Record the run path.
- [ ] **DIAGNOSIS (WHY+WHERE)** (for a bug-fix) — evidence: the root cause located with an
  ANVIL instrument from Part 1 (a `--trace` excerpt, an `analyze` support cone, a
  `validate` / `divergence` rejection trace), pasted in the owning task leaf.
- [ ] **VERIFICATION (effect)** — evidence: the measured before→after (a metric delta, a
  REJECT→PASS, byte-identical determinism across the canonical seeds) in `CHANGES.md` +
  the owning task leaf's Verification Log.
- [ ] **TASK-TREE OWNERSHIP** — structural (`scripts/check_task_tree_ownership.sh`): a
  task-tree leaf owns the change *before* the edit; the owning `docs/tasks/*.md` is
  updated in the same commit; the leaf id is in the commit subject (`commit-msg` hook).
- [ ] **LIVE-DOC EVIDENCE** — structural (`scripts/check_diagnosis_evidence.sh`): the
  mandatory `CHANGES.md` + `MEMORY.md` are amended in the same commit (`COMMIT.md`).
- [ ] **BOOK SYNC** — if the change touched a documented concept (algorithm, IR, knobs,
  synthesizability, non-triviality, sequential motifs, hierarchy, structured emission),
  the relevant `book/src/*.md` chapter is updated (the book must not drift).

The first two structural boxes are mechanically gated at pre-commit (E3) + CI (E4) via
the driver; the oracle boxes are earned at `cargo test` / the local `tool_matrix` run and
in CI. See `DOCTRINE_ENFORCEMENT.md` §10 for the live registry.
