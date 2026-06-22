# CAPABILITY-BREADTH-EXPANSION: more SV-2017/2023 up-opts + Mealy FSM outputs

## Metadata

- Tree ID: `CAPABILITY-BREADTH-EXPANSION`
- Status: `active`
- Roadmap lane: `Capability / breadth — high value-per-effort RTL surface additions (north star)`
- Created: `2026-06-17`
- Last updated: `2026-06-22`
- Owner: repo-local workflow

## Goal

Add the highest "user-visible value per effort" capability breadth, in two
strands:

1. **More SV-2017/2023 up-opts** — today only the IEEE 1800-2023 `union soft`
   overlay ships (`SV-VERSION-TARGETING` / decision `0010`). Add more
   version-distinctive, default-off, **proven** up-opts continuing that pattern:
   `enum` / `typedef`, packed multidimensional arrays, and other 2017/2023
   constructs — each gated on `sv_version`, down-gating below its standard, and
   proven downstream-clean in the matching tool mode.
2. **Mealy FSM outputs** — the Phase-6 FSM motif emits **Moore** outputs only;
   add **Mealy** outputs (outputs that also depend on the current input), as a
   default-off extension of the existing `Fsm` block + emitter.

Each new construct is API-selectable (its knob/`sv_version` gate steerable via
the MCP/config API) and introspectable (its emission counted/queryable).

## Non-Goals

- No non-synthesizable constructs; every up-opt stays inside the synthesizable
  subset and is proven accepted in the matching tool standard mode (the
  `union soft` precedent — Verilator-matching-mode acceptance; Yosys/Icarus a
  recorded no-op where they don't support the syntax).
- No retirement of the Moore FSM path; Mealy is additive (its own knob).
- Default DUT output stays byte-identical (every addition is default-off /
  down-gated).

## Acceptance Criteria

- At least one new SV up-opt **and** the Mealy FSM output extension land, each
  default-off, each proven downstream-clean (matching-mode acceptance; LRM-cited
  legality grounded against the local SV LRM cache, `reference_sv_lrm_local_cache`).
- **API-completeness gate (decision `0017`):** each new construct's knob /
  `sv_version` gate is settable via the MCP/config API, and its emission is
  queryable via `--introspect` (a metric, like `num_emitted_*`, schema-bumped per
  the additive-MINOR policy). The CLI is a shim over the same surface.
- Rules-first / valid-by-construction; a repo-owned `tool_matrix` gate per
  construct (the `--sv-version-gate` / motif-gate precedent); `tests/snapshots.rs`
  untouched by default; no retirement.
- Documented in `book/src/knobs.md` + the relevant book chapter
  (`sequential.md` for Mealy, `knobs.md`/`structured-emission.md` for up-opts) +
  USER_GUIDE + README; committed through `COMMIT.md`.

## Task Tree

- ID: `CAPABILITY-BREADTH-EXPANSION`
  Status: `active`
  Goal: `Two breadth strands — more SV-2017/2023 up-opts (continuing the union soft / SV-VERSION-TARGETING pattern) and Mealy FSM outputs (extending the Phase-6 Moore-only Fsm) — each default-off, proven, API-selectable + introspectable.`
  Children: `CAPABILITY-BREADTH-EXPANSION.1`, `CAPABILITY-BREADTH-EXPANSION.2`

- ID: `CAPABILITY-BREADTH-EXPANSION.1`
  Status: `pending`
  Goal: `SV up-opt breadth — design/decision leaf (ADR, no code): pick the NEXT version-distinctive up-opt after union soft (candidates: enum/typedef, packed multidimensional arrays, other 2017/2023 constructs), grounded in a fresh empirical probe (Verilator matching --language mode + Yosys both modes + Icarus + iverilog sim-equiv where applicable) and the local SV LRM cache for legality; pin its own default-off knob + sv_version gate + down-gate fallback + the num_emitted_* metric + the --sv-version-gate (or dedicated) coverage fact + the MCP selectability/queryability (decision 0017). Reuses src/ir/soft_union.rs + the SvVersion::permits gate as the template. Record as the next decision record + pre-split impl.`
  Acceptance: `A decision record + a tree entry pinning the chosen up-opt, the probe evidence, the knob/gate/metric, and the MCP surface; docs-only; INDEX + this tree + docs/TASK_TREE.md updated.`
  Verification: `pending`
  Commit: `pending`

- ID: `CAPABILITY-BREADTH-EXPANSION.2`
  Status: `done` (container — `.2a` design + `.2b` impl both done; the Mealy strand is complete)
  Goal: `Mealy FSM outputs — design/decision leaf (ADR, no code): ground the Mealy extension in the real Phase-6 Fsm block + emitter (src/ir Fsm + Node::FsmOut + the encoding-derived emitter; Moore-only today), pin the Mealy output model (an output that also depends on the current input, default-off behind its own knob, valid-by-construction + synthesizable), the num_emitted_* metric + a tool_matrix coverage fact, and the MCP selectability/queryability (decision 0017). Record as the next decision record + pre-split impl.`
  Children: `CAPABILITY-BREADTH-EXPANSION.2a`, `CAPABILITY-BREADTH-EXPANSION.2b`

- ID: `CAPABILITY-BREADTH-EXPANSION.2a`
  Status: `done`
  Goal: `Mealy FSM output design ADR — pin the model (a default-off combinational output decode over (state_q, sel): a per-(state, sel_value) table mirroring transitions; FsmOut stays opaque, only its decode reads the input-dependent sel cone), the fsm_mealy_prob knob, the num_mealy_fsm_modules metric + schema 1.13, the saw_mealy_fsm_design tool_matrix gate, and the MCP selectability/queryability (decision 0017), grounded in a fresh all-tool empirical probe + the SV LRM.`
  Acceptance: `A decision record + a tree entry pinning the Mealy output model, the knob, the metric/gate, and the MCP surface; docs-only; INDEX + this tree + docs/TASK_TREE.md updated.`
  Verification: `done — decision 0024 written (KM answers: front-matter); empirical probe banked in the ADR (verilator -Wall 1800-2012/2017/2023 + yosys both modes + iverilog -g2012 all ACCEPT warning-clean on the (state_q, sel) Mealy decode; enum/typedef + packed multidim arrays probed NOT version-distinctive, substantiating advancing .2 ahead of .1); INDEX + this tree + docs/TASK_TREE.md updated.`
  Commit: `CAPABILITY-BREADTH-EXPANSION.2a`

- ID: `CAPABILITY-BREADTH-EXPANSION.2b`
  Status: `done` (container — `.2b.1` mechanism + `.2b.2` metric/gate + `.2b.3` docs all done)
  Goal: `Mealy FSM output impl — default-off / DUT byte-identical, snapshots untouched.`
  Children: `CAPABILITY-BREADTH-EXPANSION.2b.1`, `CAPABILITY-BREADTH-EXPANSION.2b.2`, `CAPABILITY-BREADTH-EXPANSION.2b.3`

- ID: `CAPABILITY-BREADTH-EXPANSION.2b.1`
  Status: `done`
  Goal: `Mealy core mechanism — Fsm.mealy_outputs: Option<Vec<Vec<u128>>> 2-D table (None=Moore, byte-identical default); fsm_mealy_prob knob (config + --fsm-mealy-prob CLI + dump-config + config_category "fsm"); the per-(state, sel_value) table built + rolled inside build_fsm_block; the emitter nested case(state_q)→case(sel) Mealy output decode (Moore else-branch kept byte-identical); validate.rs Mealy-table shape/mask check; Mealy FSMs conservatively excluded from merge_equivalent_fsms (sound, nothing retired). FsmOut stays opaque (no DepSet change — sel kept reachable via fsm.sel; non-triviality/validation already satisfied; the analyze sel-fold is a deferred fidelity refinement). Lib unit tests.`
  Acceptance: `cargo check/test/clippy/fmt green; snapshots 6/6 (Moore byte-identical); fsm_mealy_prob=1.0 emits the nested case(sel) Mealy decode, all-tool-clean (Verilator -Wall 2012/2017/2023 + both Yosys + Icarus); fsm_mealy_prob=0.0 builds Moore (None).`
  Verification: `done — cargo test green (full suite); snapshots 6/6; clippy -D warnings + fmt --check clean; downstream probe (seed 7, --fsm-prob 1.0 --fsm-mealy-prob 1.0) emits 6 nested case(sel) decodes, ACCEPT warning-clean across Verilator -Wall 1800-2012/2017/2023 + Yosys both modes + Icarus -g2012; 2 new lib tests (build_fsm_block_is_moore_by_default / _is_mealy_when_knob_on).`
  Commit: `CAPABILITY-BREADTH-EXPANSION.2b.1`

- ID: `CAPABILITY-BREADTH-EXPANSION.2b.2`
  Status: `done` (container — `.2b.2a` metric/schema + `.2b.2b` gate both done)
  Goal: `Mealy introspection + gate. Default-off / DUT byte-identical.`
  Children: `CAPABILITY-BREADTH-EXPANSION.2b.2a`, `CAPABILITY-BREADTH-EXPANSION.2b.2b`

- ID: `CAPABILITY-BREADTH-EXPANSION.2b.2a`
  Status: `done`
  Goal: `Mealy metric + introspection schema bump — the num_mealy_fsm_modules DesignMetrics field (mirroring num_fsm_modules; a filter over Module::fsms for is_mealy()) surfaced in --introspect via the exact serde projection; the additive introspection schema MINOR bump 1.12 → 1.13 (const + doc comment + all schema_version test assertions in introspect/mod.rs + mcp/mod.rs + the docs/AGENT_INTROSPECTION_SCHEMA.md §6.3/§7 contract). Queryable per decision 0017. Default-off ⇒ 0 / DUT byte-identical.`
  Acceptance: `--introspect (design) shows num_mealy_fsm_modules at schema 1.13; full cargo test green; snapshots 6/6 (byte-identical); the metric is SCHEMA-DERIVED (zero new computed truth).`
  Verification: `done — cargo test green (full); snapshots 6/6; clippy -D warnings + fmt clean; live --introspect on a hierarchy with fsm_mealy_prob=1.0 reports num_mealy_fsm_modules: 2 (= num_fsm_modules) at schema_version 1.13; single-module run omits design_metrics (mirrors num_fsm_modules exactly).`
  Commit: `CAPABILITY-BREADTH-EXPANSION.2b.2a`

- ID: `CAPABILITY-BREADTH-EXPANSION.2b.2b`
  Status: `done`
  Goal: `Mealy tool_matrix gate — a repo-owned saw_mealy_fsm_design coverage fact + a focused fsm_mealy_prob=1.0 scenario (full multi-tool plan: Verilator + both Yosys + Icarus; Mealy is universally synthesizable) + ModuleReport/DesignReport detection + gap enforcement, mirroring the FSM/memory motif gates. Banked downstream-clean.`
  Acceptance: `a tool_matrix gate lights saw_mealy_fsm_design downstream-clean (Verilator + both Yosys + Icarus); default-off byte-identical.`
  Verification: `done — src/bin/tool_matrix.rs gains CoverageSummary::saw_mealy_fsm_design + phase6_mealy_fsm_focus_config (= phase6_fsm_focus_config + fsm_mealy_prob=1.0) registered as phase6_mealy_fsm in the Phase4Hierarchy set + coverage detection/merge/gap (all beside saw_fsm_design) + a phase6_mealy_fsm_scenario_is_non_vacuous test; scenario count 222→225 (one tuple ×3 strategies), gate design total 888→900. cargo test green (tool_matrix bin 79/0 incl. the new non-vacuous test, lib 589/0, snapshots 6/6 byte-identical); clippy --all-targets -D warnings + fmt --check clean. PRIMARY downstream proof — focused harness-faithful run (seed 7, the exact phase6_mealy_fsm shape) → 5 modules (1 wrapper + 4 Mealy FSM leaves with case(sel) decode) ACCEPT warning-clean across verilator --lint-only --top-module + Yosys BOTH modes (synth -noabc / abc -fast; opt -fast; check) + iverilog -g2012 -s top. The --phase4-hierarchy-gate run enforces coverage_gaps=[] incl. saw_mealy_fsm_design (gap-enforcement + non-vacuity unit-proven); the 224 non-Mealy scenarios are unchanged from the r87 / phase6-fsm-p1 banks (the focused Mealy scenario is the only new design surface), and the full 225-scenario/900-design bank is a long-running regression run separately like the prior phase banks.`
  Commit: `CAPABILITY-BREADTH-EXPANSION.2b.2b`

- ID: `CAPABILITY-BREADTH-EXPANSION.2b.3`
  Status: `done`
  Goal: `Mealy user-facing docs — book/src/sequential.md (Moore vs Mealy, a byte-verified example), book/src/knobs.md (fsm_mealy_prob), USER_GUIDE.md (the --fsm-mealy-prob row), README "Current CLI truth", and a KM how-to card. mdbook build clean; book back in sync with the codebase.`
  Acceptance: `mdbook build clean; the Mealy knob + behavior documented with an example; KM regenerated.`
  Verification: `done — book/src/sequential.md gains a "FSM outputs: Moore vs Mealy" section with a byte-verified runnable example (cargo run --release -- --seed 3 --fsm-prob 1.0 --fsm-mealy-prob 1.0 …; exercised + passing in tests/book_examples every_runnable_book_bash_block_succeeds; the emitted module is Verilator -Wall 1800-2012/2017/2023 + Yosys both modes + Icarus clean); book/src/knobs.md documents fsm_mealy_prob (knob entry + the knob→metric table row, and corrects the stale "Mealy not emitted today" line); USER_GUIDE.md adds the --fsm-mealy-prob row; README adds the --fsm-mealy-prob "Current CLI truth" bullet; a new KM how-to card docs/knowledge/fsm-mealy-outputs.md (cross-linked to decision 0024) with a working reverify; the deferred introspection schema 1.12→1.13 book-example refresh applied across api-tools.md / agent-mcp.md / api-introspection.md / api-reference.md (current-version statements + JSON examples bumped to 1.13; coverage_readout provenance left at 1.12). mdbook build clean; KM regenerated + check green (60 facts).`
  Commit: `CAPABILITY-BREADTH-EXPANSION.2b.3`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `CAPABILITY-BREADTH-EXPANSION.1` | `pending` | SV up-opt breadth — design-first ADR + fresh probe + LRM grounding before code. The **entire Mealy strand (`.2`) is now done** (`.2a` ADR + `.2b.1` mechanism + `.2b.2a` metric/schema + `.2b.2b` gate + `.2b.3` docs); `.1` is the only remaining child. **Deferred (not retired):** the `.2a` probe re-confirmed (per decision `0010`) that the named candidates (enum/typedef, packed multidim arrays) are accepted at every Verilator `--language` mode + Yosys + Icarus ⇒ not version-distinctive, no down-gating teeth; the genuinely-2023 clean space with the installed tools is thin (essentially `union soft`, shipped). A future `.1` either finds a genuinely-2023 construct or rescopes to `union soft` breadth. |

## Decisions

- `2026-06-17`: Registered as an owner-directed capability-breadth lane. Binds
  decision [`0017`](../decisions/0017-api-first-everything-mcp-accessible.md)
  (each construct API-selectable + introspectable). Two parallel design-first
  strands (`.1` SV up-opts, `.2` Mealy FSM); each is its own decision record when
  picked. Reuses `SV-VERSION-TARGETING` (decisions `0009`/`0010`) and the
  Phase-6 FSM motif; nothing retired.
- `2026-06-22` (`.2a`): **Mealy FSM output design ADR** — decision
  [`0024`](../decisions/0024-mealy-fsm-outputs.md). A Mealy FSM output is a
  default-off combinational decode of `(state_q, sel)` — a per-`(state, sel_value)`
  constant table mirroring `transitions`, rendered as the proven nested
  `case (state_q)` → `case (sel)` form that drives the **opaque** `FsmOut` leaf
  (only its decode reads the input-dependent `sel` cone; the state register stays
  Moore-clocked). New default-off `fsm_mealy_prob` knob, `num_mealy_fsm_modules`
  metric (introspection schema `1.12 → 1.13`), `saw_mealy_fsm_design` tool_matrix
  gate, `--fsm-mealy-prob` CLI flag + MCP/config settability. **`.2` advanced
  ahead of frontier-ordered `.1`** on fresh evidence: a probe of the named `.1`
  candidates (enum/typedef, packed multidim arrays) found them accepted at every
  Verilator `--language` mode + Yosys + Icarus ⇒ not version-distinctive (no
  down-gating teeth), re-confirming decision `0010`; Mealy is genuinely-new,
  all-tool-clean, high-certainty breadth. `.1` stays `pending`, nothing retired.

## Open Questions

- Which up-opt first (`enum`/`typedef` vs packed multidim arrays vs another 2023
  construct) — the `.2a` probe showed the named candidates are **not**
  version-distinctive with the installed tools (accepted at every `--language`
  mode + Yosys + Icarus); the genuinely-2023 clean space is thin (essentially
  `union soft`, shipped). A future `.1` either finds a genuinely-2023 construct or
  rescopes to `union soft` breadth. *(Resolves at `.1`.)*
- ~~Mealy output shape: per-FSM-output vs whole-FSM mode~~ — **resolved at `.2a`
  (decision `0024`):** whole-FSM mode first cut (an `Fsm` has exactly one output
  today; per-output choice is moot until multi-output FSMs exist). The output
  reuses the existing `sel` cone (one cone notion, not two), and the decode mirrors
  the proven next-state nested case.
- Exact `Fsm` IR field layout for the Mealy table + the `FsmOut` virtual-deps
  construction folding `sel`'s support + the Mealy FSM identity/dedup keying.
  *(Resolves at `.2b` / `.2b.1`.)*

## Blockers

- None. (Independent of the six usability lanes; reuses closed
  `SV-VERSION-TARGETING` + Phase-6 surfaces.)

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-06-17` | `CAPABILITY-BREADTH-EXPANSION` | `tree registered (docs-only); no code` | `registered` |
| `2026-06-22` | `CAPABILITY-BREADTH-EXPANSION.2a` | `decision 0024 written; empirical probe — verilator -Wall 1800-2012/2017/2023 + yosys both modes + iverilog -g2012 all ACCEPT warning-clean on the (state_q, sel) Mealy decode; enum/typedef + packed multidim arrays probed NOT version-distinctive (accepted at every mode); INDEX + tree + docs/TASK_TREE.md updated; mem-arch + KM self-checks` | `done` (docs-only; no code; DUT byte-identical) |
| `2026-06-22` | `CAPABILITY-BREADTH-EXPANSION.2b.1` | `cargo test green (full suite); snapshots 6/6 (Moore byte-identical); clippy -D warnings + fmt --check clean; downstream probe (seed 7, --fsm-prob 1.0 --fsm-mealy-prob 1.0) → 6 nested case(sel) decodes, ACCEPT warning-clean across Verilator -Wall 1800-2012/2017/2023 + Yosys both modes + Icarus -g2012; 2 new lib tests` | `done` (Mealy mechanism; default-off DUT byte-identical) |
| `2026-06-22` | `CAPABILITY-BREADTH-EXPANSION.2b.2a` | `num_mealy_fsm_modules DesignMetrics field + schema 1.12→1.13 (const + comment + all schema_version assertions in introspect/mod.rs + mcp/mod.rs + the schema-doc §6.3/§7); cargo test green (lib 589/0 + full); snapshots 6/6; clippy -D warnings + fmt clean; live --introspect (hierarchy, fsm_mealy_prob=1.0) → num_mealy_fsm_modules: 2 at schema 1.13` | `done` (metric queryable; default-off byte-identical) |
| `2026-06-22` | `CAPABILITY-BREADTH-EXPANSION.2b.2b` | `saw_mealy_fsm_design coverage fact + phase6_mealy_fsm scenario (= phase6_fsm + fsm_mealy_prob=1.0) + detection/merge/Phase4Hierarchy gap + phase6_mealy_fsm_scenario_is_non_vacuous test (count 222→225, gate designs 888→900) in src/bin/tool_matrix.rs; cargo test green (tool_matrix bin 79/0, lib 589/0, snapshots 6/6 byte-identical); clippy --all-targets -D warnings + fmt clean; PRIMARY proof — focused harness-faithful downstream run (seed 7, the exact phase6_mealy_fsm shape) ACCEPT warning-clean across verilator --top-module + Yosys both modes + iverilog -g2012 -s top; the --phase4-hierarchy-gate run enforces coverage_gaps=[] incl. saw_mealy_fsm_design (gap-enforcement + non-vacuity unit-proven; 224 non-Mealy scenarios unchanged from the r87/phase6-fsm-p1 banks)` | `done` (Mealy gate; default-off DUT byte-identical) |
| `2026-06-22` | `CAPABILITY-BREADTH-EXPANSION.2b.3` | `book sequential.md "FSM outputs: Moore vs Mealy" + byte-verified runnable example (passing in tests/book_examples; emitted module Verilator -Wall 2012/2017/2023 + Yosys both + Icarus clean); knobs.md fsm_mealy_prob entry + metric-table row + corrected stale line; USER_GUIDE --fsm-mealy-prob row; README --fsm-mealy-prob bullet; new KM card docs/knowledge/fsm-mealy-outputs.md (working reverify, cross-linked to 0024); deferred schema 1.12→1.13 book-example refresh (api-tools/agent-mcp/api-introspection/api-reference; provenance kept at 1.12); mdbook build clean; KM regen+check green (60 facts)` | `done` (docs-only; book back in sync; DUT byte-identical) |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `CAPABILITY-BREADTH-EXPANSION` | `USABILITY-LANE-OWNERSHIP.1 — register 7 owner-directed usability/capability lanes + API-first decision 0017` | Tree registered (not yet started); frontier `.1` (SV up-opt design ADR) + `.2` (Mealy FSM design ADR) pending. |
| `CAPABILITY-BREADTH-EXPANSION.2a` | `CAPABILITY-BREADTH-EXPANSION.2a — Mealy FSM output design ADR (decision 0024)` | Design ADR (docs-only). Pins the Mealy `(state_q, sel)` output model, `fsm_mealy_prob` knob, `num_mealy_fsm_modules` metric (schema `1.13`), `saw_mealy_fsm_design` gate, MCP surface. `.2` split into `.2a` (done) + `.2b` (proposed). |
| `CAPABILITY-BREADTH-EXPANSION.2b.1` | `CAPABILITY-BREADTH-EXPANSION.2b.1 — Mealy FSM output mechanism (knob + IR + emitter + validate)` | First **code** slice of the lane. `Fsm.mealy_outputs` + `fsm_mealy_prob`/`--fsm-mealy-prob` + the emitter nested `case(state_q)→case(sel)` Mealy decode + validate + dedup-exclusion + 2 lib tests. Default-off DUT byte-identical (snapshots 6/6); all-tool-clean. `.2b` split into `.2b.1` (done) + `.2b.2` (metric/gate) + `.2b.3` (docs). |
| `CAPABILITY-BREADTH-EXPANSION.2b.2a` | `CAPABILITY-BREADTH-EXPANSION.2b.2a — Mealy metric num_mealy_fsm_modules + introspection schema 1.13` | The `num_mealy_fsm_modules` `DesignMetrics` field (serde-projected into `--introspect`) + the additive schema bump `1.12 → 1.13` (const + comment + all `schema_version` assertions + the schema-doc contract). Queryable (decision `0017`); default-off byte-identical. `.2b.2` split into `.2b.2a` (done) + `.2b.2b` (gate). |
| `CAPABILITY-BREADTH-EXPANSION.2b.2b` | `CAPABILITY-BREADTH-EXPANSION.2b.2b — Mealy FSM tool_matrix gate (saw_mealy_fsm_design + phase6_mealy_fsm scenario)` | The repo-owned Mealy gate in `src/bin/tool_matrix.rs`: `saw_mealy_fsm_design` coverage fact + `phase6_mealy_fsm` scenario (`fsm_mealy_prob=1.0`) + detection/merge/`Phase4Hierarchy` gap + a non-vacuity test (count 222→225, gate designs 888→900), mirroring the `phase6_fsm`/`phase6_inferrable_memory` motif gates. Banked downstream-clean; default-off DUT byte-identical. `.2b.2` done; `.2b` frontier → `.2b.3` (docs). |
| `CAPABILITY-BREADTH-EXPANSION.2b.3` | `CAPABILITY-BREADTH-EXPANSION.2b.3 — Mealy user-facing docs (book + USER_GUIDE + README + KM card + schema 1.13 refresh)` | Docs-only: `book/src/sequential.md` "FSM outputs: Moore vs Mealy" + a byte-verified runnable example; `knobs.md` `fsm_mealy_prob` entry + table row + corrected stale line; `USER_GUIDE`/`README` `--fsm-mealy-prob`; new KM card `docs/knowledge/fsm-mealy-outputs.md`; the deferred introspection schema `1.12→1.13` book-example refresh. Closes `.2b` and the whole Mealy strand `.2`. Book back in sync; DUT byte-identical. Tree frontier → `.1` (SV up-opt, deferred). |

## Changelog

- `2026-06-17`: Created task tree (registration via `USABILITY-LANE-OWNERSHIP.1`).
- `2026-06-22`: `.2a` done — Mealy FSM output design ADR (decision `0024`); `.2`
  split into `.2a` (design, done) + `.2b` (impl, active); frontier advanced to
  `.2b`. `.1` deferred (not retired) on the `.2a` not-version-distinctive probe.
- `2026-06-22`: `.2b.1` done — Mealy FSM output **mechanism** (the `fsm_mealy_prob`
  knob + `Fsm.mealy_outputs` 2-D table + the emitter nested-case Mealy decode +
  validate + the `merge_equivalent_fsms` Mealy exclusion + lib tests);
  default-off DUT byte-identical, all-tool-clean. `.2b` split into `.2b.1` (done)
  + `.2b.2` (metric/gate, proposed) + `.2b.3` (docs, proposed); frontier `.2b.2`.
- `2026-06-22`: `.2b.2a` done — the `num_mealy_fsm_modules` `DesignMetrics` metric
  + introspection schema bump `1.12 → 1.13` (queryable per decision `0017`);
  default-off byte-identical. `.2b.2` split into `.2b.2a` (done) + `.2b.2b` (the
  `tool_matrix` gate, proposed); frontier `.2b.2b`.
- `2026-06-22`: `.2b.2b` done — the Mealy `tool_matrix` gate (`saw_mealy_fsm_design`
  coverage fact + the focused `phase6_mealy_fsm` scenario + detection/merge/gap +
  a non-vacuity test) in `src/bin/tool_matrix.rs`, mirroring the
  `phase6_fsm`/`phase6_inferrable_memory` motif gates; proven downstream-clean by a
  focused harness-faithful run of the exact gate scenario (Verilator `--top-module`
  + Yosys both modes + Icarus), with the `--phase4-hierarchy-gate` run enforcing
  `coverage_gaps=[]` incl. `saw_mealy_fsm_design` (gap-enforcement + non-vacuity
  unit-proven; 224 non-Mealy scenarios unchanged from the r87 / `phase6-fsm-p1`
  banks); default-off DUT byte-identical. `.2b.2`
  done; `.2b` frontier → `.2b.3` (the user-facing docs); `.1` still deferred (not
  retired).
- `2026-06-22`: `.2b.3` done — the Mealy user-facing docs: `book/src/sequential.md`
  "FSM outputs: Moore vs Mealy" with a byte-verified runnable example (passing in
  `tests/book_examples`), `knobs.md` `fsm_mealy_prob` (entry + metric-table row +
  corrected stale line), `USER_GUIDE`/`README` `--fsm-mealy-prob`, the new KM card
  `docs/knowledge/fsm-mealy-outputs.md`, and the deferred introspection schema
  `1.12 → 1.13` book-example refresh. **`.2b` done; the whole Mealy strand `.2` is
  done.** Book back in sync; DUT byte-identical. The tree stays `active` with the
  only remaining child `.1` (SV up-opt) deferred-not-retired.
