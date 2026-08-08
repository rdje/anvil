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
| `--introspect` (schema `1.28`) | the SCHEMA-DERIVED projection of `Config`/`Metrics`/`DesignMetrics` + a content-addressed `run_id` + the `coverage_readout` (per-knob/per-category achieved fire rates + gate/operand/depth histograms) | `anvil --seed 42 --introspect` | introspection JSON to stdout (single-artifact run; `docs/AGENT_INTROSPECTION_SCHEMA.md`) |
| MCP `analyze` | derived-relation queries over the DUT IR (fourteen kinds): `output_support` (an output's transitive support cone), `input_reach` (its dual fan-out), `flop_reset_provenance` (per-flop reset/data provenance), `module_reachability` (which modules a design reaches from the top), `flop_dependencies` (per-flop register-to-register predecessors/successors + self-feedback flag), `memory_provenance` (per-inferrable-memory shape + its read/write-port support cones), `fsm_provenance` (per-generated-FSM shape + its transition-select `sel` support cone), `node_drivers` (per-node immediate 1-hop driver adjacency + `GateOp`), `node_readers` (its transpose — per-node immediate 1-hop reader adjacency), `instance_provenance` (per-child-instance descent — each child output's support cone built inside the child module's graph; design-only), `instance_input_bindings` (its parent-side dual — the parent node driving each child input port), `longest_path` (one representative longest combinational fan-in path per target — the gate chain realizing `output_support`'s `cone_depth`; the witness for the scalar depth), `node_reach` (per-node transitive combinational fan-OUT — the output ports + flop `D` cones it reaches; the transitive complement to `node_readers`, the node-addressed generalization of `input_reach`), `reach_path` (per-node longest combinational fan-OUT path — the gate chain to a boundary sink; the forward complement to `longest_path`, the path-witness for `node_reach`) | `anvil-mcp` tool `analyze` `{query, target}` | derived-relation JSON (no new computed truth) |
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

### 6. Gate quality — *"can this check actually fire, and did my experiment run?"*

The instruments above pinpoint issues in ANVIL's **artifacts**. This one pinpoints an issue in
ANVIL's **gates** — or rather in the experiment used to trust one.

| Instrument | Pinpoints | Invocation | Output |
|---|---|---|---|
| `scripts/negative_control.sh` | a negative control whose **mutation never landed** — `sed -i` / `perl -pi -e` exit `0` whether or not the pattern matched, so a mistyped substitution leaves the tree unchanged, the check passes, and that reads exactly like a control that correctly did not fire. `apply` refuses a zero-count substitution *before* any verdict can be read; `restore` is verified with `cmp`; `probe` runs the whole cycle against a **declared** expectation | `scripts/negative_control.sh probe <file> '<perl-subst>' <fires\|silent> <cmd…>` · `--self-test` | per-probe PASS/FAIL; exit `9` means *the experiment did not run* (distinct from `2`, a broken expression) |

**Reach for a mutation that cannot no-op first** (decision
[`0047`](docs/decisions/0047-negative-control-carrier-is-the-mutation.md), rung R1): an in-language
mutation inside a `#[test]`, a compiler-checked probe, a constructed fixture, a `git show HEAD:`
baseline, or a mutation-free test seam has no *match* step, so the failure this instrument catches
cannot occur. Use the instrument when the subject genuinely **is** file text. It is deliberately
**not** a registered doctrine: a control's mutation is reverted before the commit by construction,
so there is nothing for a gate to read.

### 7. Book readability — *"is the rendered book readable, and did my book edit change only what I meant?"*

The owner reviews the **book**, not the code (`COMMIT.md` §9), so a defect in the rendered HTML is a
defect in the only surface they see. These instruments measure that surface. All three read the
**rendered** book, so `mdbook build book` runs first.

| Instrument | Pinpoints | Invocation | Output |
|---|---|---|---|
| `scripts/book_prose_census.py` | wall-of-text blocks: every rendered prose block over a threshold, with its source anchor and **what could actually repair it** (`SPLITTABLE` / `RUN-ON` / `LIST-ITEM` / `TABLE-CELL`). Code (`<pre>`) is excluded and `<li>`/`<td>`/`<blockquote>` are counted, because a `<p>`-only regex census was blind to 5 of 11 oversized blocks | `scripts/book_prose_census.py --threshold 1500` · `--json` | per-block table + denominator, over-threshold count, worst block, oversized mass |
| `scripts/book_list_signature.py` | a book edit that silently **changed list structure** — dropping a list item's continuation indent promotes the continuation out of its `<li>` and ends the list, with `mdbook build` exiting `0` | `--save F` before the edit, `--compare F` after | per-chapter `<li>` count + content SHA; exit `1` on any change |
| `scripts/prove_words_unchanged.py` | that an edit was **whitespace-only** — collapses all whitespace and requires byte-identity against a git ref. Strictly stronger than `git diff --ignore-blank-lines`, which cannot permit a break at a sentence that begins mid-line | `scripts/prove_words_unchanged.py --ref HEAD book/src/*.md` | per-file `OK`/`DIFF` + the located divergence |
| `scripts/prove_clauses_unchanged.py` | that a **run-on → markdown-list** conversion dropped, invented, reworded or **reordered** no clause. That edit is not whitespace-only — it removes the separating commas and the `and`/`plus` connectives — so the word proof necessarily fires and proves nothing; this one removes exactly what a list conversion may change and requires the remaining word *sequence* to be identical | `scripts/prove_clauses_unchanged.py --ref HEAD book/src/*.md` | per-file `OK`/`CLAUSES DIFFER` + the located divergence |
| `scripts/prove_clauses_unchanged.py --allow-move` | that a **lift** — moving a run out of a container it does not fit, e.g. a GFM cell that cannot hold a block-level list — dropped or invented no **word**. A lift is a reordering of the file by construction, so the default mode fires on it and reports nothing useful. The unit is the **word, not the clause**, and that is measured rather than stylistic: a list conversion deletes the separators that *define* a clause boundary, so a clause multiset reports a spurious *N removed / 1 added* on a correct edit | `scripts/prove_clauses_unchanged.py --allow-move --ref HEAD book/src/*.md` | per-file `OK`/`WORDS DIFFER` + every word added and removed |

**Pick the proof that matches the edit, and know what it cannot see.** `prove_words_unchanged.py`
permits nothing, so it is the proof for a whitespace-only split and it is *useless* on a list
conversion (it fires by construction). `prove_clauses_unchanged.py` permits exactly the separators
and connectives a list conversion removes — and is therefore **blind to an edit that only adds or
removes an `and`/`plus`/`or`**, measured rather than assumed (`BOOK-PARAGRAPH-BLOBS.3b`). Its
`--allow-move` mode permits a *move* as well, and is therefore blind to **reordering** — the exact
property the default mode exists to have — so it is opt-in and must never be the only proof cited.
Neither sees a **paragraph break inserted inside a sentence**, and the census *rewards* one by
reporting a smaller block — see [[paragraph-split-can-cut-a-sentence-in-half]] for the source-level
predicate that does catch it.

**A lift takes two proofs, not one, because content and remainder are separate questions.**
`--allow-move` is the cheap re-runnable summary; on its own it cannot tell a lift from a shuffle. Pair
it with an explicit comparison of the *moved run's own sequence* (extract it from both sides and
require byte-identity) and of the *file with that run excised*. `BOOK-PARAGRAPH-BLOBS.3c` ran both:
14/14 items identical in order, and 54,791 = 54,791 normalized remainder chars.

**A control on a file the proof is already firing on proves nothing** — the instrument is
**saturated** and fires whatever you do. Both `.3b` and `.3c` hit this and had to move the carrier to
a file that still passes cleanly. Check that the proof is *silent before the mutation* before reading
any verdict from it.

**Use the list signature with either of them — each is blind exactly where the other fires.** Measured on one carrier
(a paragraph break inside a list item, given no continuation indent): the word proof reports `OK`
(90,542 → 90,542 normalized chars — it collapses the very whitespace that carries list nesting) while
the list signature **FIRES** on the changed content SHA. That blindness is by construction, not a bug;
`BOOK-PARAGRAPH-BLOBS.1` shipped exactly this regression with only the word proof in hand.

**The census is deliberately not a gate**, and is absent from `scripts/check_doctrines.sh`. Its
1,500-character default was read off a distribution — a reporting convenience, not a derived limit.
Whether anything should *watch* paragraph size is `BOOK-PARAGRAPH-BLOBS.2`'s question, and decision
[`0047`](docs/decisions/0047-negative-control-carrier-is-the-mutation.md) prefers removing the need
over watching harder.

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
  mandatory `CHANGES.md` entry is amended in the same commit (`COMMIT.md`). `MEMORY.md` is
  **not** asserted here: it records where the *work* stopped, not what the *diff* did, so a
  commit that changes no resumable state has nothing true to write in it. Its own doctrine
  (`MEMORY-ARCH`) owns its size, shape and required fields — decision
  [`0051`](docs/decisions/0051-the-resume-pointer-is-updated-when-resumable-state-changes.md).
- [ ] **BOOK SYNC** — if the change touched a documented concept (algorithm, IR, knobs,
  synthesizability, non-triviality, sequential motifs, hierarchy, structured emission),
  the relevant `book/src/*.md` chapter is updated (the book must not drift).

The first two structural boxes are mechanically gated at pre-commit (E3) + CI (E4) via
the driver; the oracle boxes are earned at `cargo test` / the local `tool_matrix` run and
in CI. See `DOCTRINE_ENFORCEMENT.md` §10 for the live registry.
