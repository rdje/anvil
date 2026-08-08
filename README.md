# anvil
Single entry point for the project.

## Project objective
`anvil` is a random by-construction generator of **synthesizable
SystemVerilog RTL artifacts**. Its default DUT lane produces
syntactically valid, semantically correct, synthesizable, and
structurally non-trivial modules by building a typed circuit graph via
fanin-cone recursion and emitting SV from it.

The intended destination is stronger than "valid enough": `anvil`
should become a **signoff-level-quality random RTL generator** whose
outputs are boringly clean for mainstream downstream HDL consumers. The
product goal is **legal, reproducible, unusual RTL** that parsers,
elaborators, RTL compilers, linters, simulators, and synthesis tools
should accept. Those artifacts can be used to stress such tools and
expose real bugs precisely because they stay inside the accepted
synthesizable envelope.

Whole-module intended functionality is not the target. By construction,
the recursive fanin-cone process mainly aims at legal structure and
tool-ingestible complexity; absent a specification, most generated
modules are expected to be functionally arbitrary or outright
gibberish, and that is acceptable.

The scope is broader than one leaf-module format. ANVIL ships three
artifact lanes through the same `anvil` binary: the default DUT RTL
lane, an oracle-backed micro-design lane, and a source-level
frontend/elaboration accept lane with explicit expected-facts
manifests. The default remains `--artifact dut`; the other lanes are
opt-in and keep their generators decoupled from the DUT path.

**Three load-bearing principles:**
1. **Recursion is the core algorithm.** The generator answers one question — *"what drives this signal?"* — and recurses. Every level of abstraction (gate, cone, module, hierarchy) is the same recursion with a richer choice set. Iteration is the exception; recursion is the default. Anything that can be expressed as a recursive descent over a typed circuit graph should be.
2. **Every emitted module is valid by construction.** No generate-then-filter. No post-hoc repair. If a generator output fails semantic validation or synthesis, that is a generator bug, not expected behavior.
3. **Every output is reproducible.** Byte-identical output for the same `(seed, knobs)` pair, across platforms, forever. Seeded ChaCha8; no `thread_rng`; no wall-clock entropy; no hash-map iteration order in output paths.

## Prerequisites and quick start
A stable Rust toolchain (`cargo`) is the only build dependency. The
optional downstream checks below need Verilator, Yosys, or Icarus
Verilog installed; ANVIL vendors none of them.

```bash
cargo build                                            # build
cargo test                                             # IR validation + byte-identical reproducibility
cargo run -- --seed 42                                 # one module to stdout
cargo run -- --seed 42 --count 100 --out ./generated   # a corpus + manifest.json
verilator --lint-only generated/mod_42_0000.sv         # optional: elaboration check
```

Cargo's default run target is `anvil`, so plain `cargo run -- …`
invokes the generator even though the repository also has the auxiliary
`tool_matrix` harness; select that one explicitly with `cargo run --bin
tool_matrix -- …`.

A downstream tool rejecting a generated file is a **generator bug**:
file it with the seed and the effective knobs from `manifest.json`.

Everything past this first run — every flag, knob, preset, steering
category, artifact lane, `tool_matrix` gate invocation, and
downstream-verification recipe — lives in [`USER_GUIDE.md`](USER_GUIDE.md)
and the [book](book/src/SUMMARY.md). This file deliberately does not
mirror them; see [Where content goes](#where-content-goes).

## Architecture at a glance
### Crate layout
- `src/main.rs` — CLI entry point; `src/lib.rs` — library root
- `src/config.rs` — knobs, CLI overlay, validation
- `src/ir/types.rs` — `Module`, `Node`, `GateOp`, `Flop`, `DepSet`
- `src/ir/validate.rs` — IR invariant checker (safety net)
- `src/ir/knob_id.rs` — `KnobId`: the steering taxonomy (what is steerable, and
  in which coverage family)
- `src/ir/knob_roll.rs` — the crate's single steering-aware knob-roll primitive
- `src/ir/soft_union.rs`, `src/ir/function_emit.rs`, … — emit-projection
  annotation passes (one per structured-emission surface)
- `src/gen/mod.rs` — `Generator` entry points; `src/gen/cone.rs` — fanin-cone recursion
- `src/gen/module.rs` — leaf-module generator; `src/gen/pool.rs` — `SignalPool` terminal selection
- `src/gen/hierarchy.rs` — hierarchy planner (legacy depth-1 wrapper lane +
  bounded recursive lane, child sourcing, parent-side composition, child-input
  routing, parent-cone helper instances, optional parent-local state)
- `src/emit/sv.rs` — IR → SystemVerilog pretty-printer
- `src/introspect/mod.rs` — versioned introspection document builder (`--introspect`)
- `src/downstream/mod.rs` — hardened downstream-tool surface
  (verilator/yosys/iverilog/sv2v/slang) + `validate` / `minimize`
- `src/microdesign/`, `src/frontend/`, `src/umbrella/` — the non-DUT artifact
  lanes and the `--artifact` selector plumbing
- `src/mcp/mod.rs` — read-mostly MCP server (tools / resources / prompts)

### Binaries, tests, and examples
- `src/bin/tool_matrix.rs` — curated downstream scenario-matrix harness and gates
- `src/bin/anvil_mcp.rs` — `anvil-mcp`, the stdio/HTTP transport over `src/mcp`
- `tests/pipeline.rs` — end-to-end: generate → validate → emit
- `tests/snapshots.rs` — the `insta` byte-identical-reproducibility guard
- `examples/generate_one.rs` — minimal library usage

## Canonical documentation
Only the documents below are status authority. The mdBook is explicitly
part of that set — not reference material adjacent to it. Recovery
requires reading it.

| Document | What it is |
| --- | --- |
| [`MEMORY.md`](MEMORY.md) | the resume pointer — read this first when resuming work |
| [`MEMORY_ARCHITECTURE.md`](MEMORY_ARCHITECTURE.md) | durable agent-memory standard: resume-pointer, task-tree, decision-record, and git-history layers |
| [`SESSION_BOOTSTRAP.md`](SESSION_BOOTSTRAP.md) | what a fresh session reads first to regain full context |
| [`USER_GUIDE.md`](USER_GUIDE.md) | **the live CLI reference**: every flag, knob, preset, steering category, `tool_matrix` gate, and downstream-verification workflow |
| [`book/`](book/src/SUMMARY.md) | the mdBook: *Using anvil* (Getting Started / Tutorial / Recipes), *How It Works* (Core Idea / Algorithm / IR), *Correctness Guarantees*, *Motif Catalogue*, *Reference* |
| [`ROADMAP.md`](ROADMAP.md) | phased scope of the DUT lane, the delivered non-DUT lanes, and the post-phase follow-up trees |
| [`docs/TASK_TREE.md`](docs/TASK_TREE.md) · [`docs/tasks/*.md`](docs/tasks) | the task-tree workflow and one file per tree: stable leaf IDs, frontier, blockers, verification log |
| [`docs/decisions/*.md`](docs/decisions) | durable decision records — the "why" behind every capability and boundary |
| [`KNOWLEDGE_MAP.md`](KNOWLEDGE_MAP.md) · [`knowledge-map/`](knowledge-map/KNOWLEDGE_MAP_ARCHITECTURE.md) | question-keyed retrieval index over facts already logged in the repo, and the standard behind it |
| [`docs/evidence/`](docs/evidence/README.md) | committed closure-evidence digests — what a banked gate run actually reported (the corpora themselves are bulk and untracked; [`INVENTORY.md`](docs/evidence/INVENTORY.md) classifies every cited bank) |
| [`DOCTRINE_ENFORCEMENT.md`](DOCTRINE_ENFORCEMENT.md) | every load-bearing doctrine paired with a deterministic check, run from one registry+driver |
| [`LIVE_DOCUMENT_SIZE_CONTAINMENT.md`](LIVE_DOCUMENT_SIZE_CONTAINMENT.md) | how every long-lived document stays a bounded working set while its history stays recoverable ([`0052`](docs/decisions/0052-live-document-size-containment-adoption.md)) |
| [`TOOLBOX.md`](TOOLBOX.md) | ANVIL's own diagnostic instruments, plus the acceptance checklist a code change must satisfy |
| [`CODEBASE_ANALYSIS.md`](CODEBASE_ANALYSIS.md) | live Rust-workspace analysis, aligned to the roadmap and current code |
| [`DEVELOPMENT_NOTES.md`](DEVELOPMENT_NOTES.md) | engineering rationale, rejected alternatives, and earned gotchas (append-only) |
| [`CHANGES.md`](CHANGES.md) | fully detailed description of completed changes (append-only) |
| [`COMMIT.md`](COMMIT.md) | the canonical commit workflow |
| [`README_POLICY.md`](README_POLICY.md) | why this file is short, and what may be added to it |

**Two non-negotiable doctrines to know before editing anything:**

- **No code change may be made without a task-tree leaf owning it first**
  (`2026-05-17`). Pure-docs / mdBook / workflow edits are exempt; see
  [`docs/TASK_TREE.md`](docs/TASK_TREE.md) "ANVIL Adoption Scope" for the
  code/not-code boundary and [`docs/TASK_TREE_README.md`](docs/TASK_TREE_README.md)
  for the portable setup guide.
- **Every doctrine is mechanically gated.** `scripts/check_doctrines.sh` is the
  single registry+driver, run by the git hook and by CI. Live registry:
  <!--enum:doctrine-ids-->`MEMORY-ARCH`, `KNOWLEDGE-MAP`, `CODE-CHANGE-EVIDENCE`,
  `TASK-TREE-OWNERSHIP`, `NO-BOOT-VOLUME-REFS`, `EVIDENCE-CITATIONS`,
  `ENUMERATION-PARITY`, `README-GROWTH`, `TABLE-RENDER-FIDELITY`,
  `CHANGES-ENTRY-PLACEMENT`, and
  `BOOK-LINK-TARGETS`<!--/enum:doctrine-ids--> (`README-GROWTH` keeps *this*
  file a landing page) — a list itself gated by `ENUMERATION-PARITY`.

## Where content goes
This README is a landing page under [`README_POLICY.md`](README_POLICY.md),
not a changelog, roadmap, or CLI reference. Ordinary feature work updates the
canonical destination below and leaves this file alone.

| Content | Canonical home |
| --- | --- |
| User-facing flags, knobs, presets, gate invocations, examples | [`USER_GUIDE.md`](USER_GUIDE.md), [`book/src/`](book/src/SUMMARY.md) |
| Current work, priorities, phase status | [`ROADMAP.md`](ROADMAP.md), [`docs/tasks/`](docs/tasks) |
| Cross-tree status, one row per tree | [`docs/TASK_TREE.md`](docs/TASK_TREE.md) — its *row* contract: one frontier, not a per-leaf journal ([`0042`](docs/decisions/0042-task-tree-index-is-a-mixed-surface.md)) |
| Release / change history | [`CHANGES.md`](CHANGES.md), git history |
| Design rationale and rejected alternatives | [`docs/decisions/`](docs/decisions), [`DEVELOPMENT_NOTES.md`](DEVELOPMENT_NOTES.md) |
| Banked gate evidence and coverage-fact tallies | [`docs/evidence/`](docs/evidence/README.md) |
| Diagnostics and operational procedures | [`TOOLBOX.md`](TOOLBOX.md) |

Change this file only when the project's purpose, first-use path, top-level
architecture, or canonical navigation changes.

## License
Licensed under either of:
- Apache License, Version 2.0
- MIT License

at your option.

Read [`SESSION_BOOTSTRAP.md`](SESSION_BOOTSTRAP.md) and start from there.
