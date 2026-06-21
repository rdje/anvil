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
  Status: `active` (container — split into `.2a` design + `.2b` impl)
  Goal: `Mealy FSM outputs — design/decision leaf (ADR, no code): ground the Mealy extension in the real Phase-6 Fsm block + emitter (src/ir Fsm + Node::FsmOut + the encoding-derived emitter; Moore-only today), pin the Mealy output model (an output that also depends on the current input, default-off behind its own knob, valid-by-construction + synthesizable), the num_emitted_* metric + a tool_matrix coverage fact, and the MCP selectability/queryability (decision 0017). Record as the next decision record + pre-split impl.`
  Children: `CAPABILITY-BREADTH-EXPANSION.2a`, `CAPABILITY-BREADTH-EXPANSION.2b`

- ID: `CAPABILITY-BREADTH-EXPANSION.2a`
  Status: `done`
  Goal: `Mealy FSM output design ADR — pin the model (a default-off combinational output decode over (state_q, sel): a per-(state, sel_value) table mirroring transitions; FsmOut stays opaque, only its decode reads the input-dependent sel cone), the fsm_mealy_prob knob, the num_mealy_fsm_modules metric + schema 1.13, the saw_mealy_fsm_design tool_matrix gate, and the MCP selectability/queryability (decision 0017), grounded in a fresh all-tool empirical probe + the SV LRM.`
  Acceptance: `A decision record + a tree entry pinning the Mealy output model, the knob, the metric/gate, and the MCP surface; docs-only; INDEX + this tree + docs/TASK_TREE.md updated.`
  Verification: `done — decision 0024 written (KM answers: front-matter); empirical probe banked in the ADR (verilator -Wall 1800-2012/2017/2023 + yosys both modes + iverilog -g2012 all ACCEPT warning-clean on the (state_q, sel) Mealy decode; enum/typedef + packed multidim arrays probed NOT version-distinctive, substantiating advancing .2 ahead of .1); INDEX + this tree + docs/TASK_TREE.md updated.`
  Commit: `CAPABILITY-BREADTH-EXPANSION.2a`

- ID: `CAPABILITY-BREADTH-EXPANSION.2b`
  Status: `proposed`
  Goal: `Mealy FSM output impl — the Fsm Mealy discriminator + 2-D mealy_outputs[state][sel_value] table, the emitter nested case(state_q)→case(sel) output decode, the fsm_mealy_prob roll inside build_fsm_block, the FsmOut virtual-deps fix folding sel's support, the num_mealy_fsm_modules metric (schema 1.13), the saw_mealy_fsm_design tool_matrix gate, and book/USER_GUIDE/README/KM docs; default-off / DUT byte-identical, snapshots untouched. Pre-split into .2b.1 (design-detail) + .2b.2 (impl) + .2b.3 (docs) when picked.`
  Acceptance: `Mealy outputs emit by construction under fsm_mealy_prob; default-off byte-identical (snapshots 6/6); a downstream-clean tool_matrix gate (Verilator + both Yosys + Icarus) lights saw_mealy_fsm_design; the knob is MCP/config-settable + CLI-flagged and the metric queryable; book + short-form docs synced.`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `CAPABILITY-BREADTH-EXPANSION.2b` | `proposed` | Mealy FSM output impl — `.2a` design ADR (decision `0024`) is done; implement the default-off `fsm_mealy_prob` Mealy output decode over `(state_q, sel)`, the metric (schema `1.13`), the downstream-clean gate, and docs. All-tool-clean by the `.2a` probe; reuses the proven next-state nested-case emitter shape. |
| 2 | `CAPABILITY-BREADTH-EXPANSION.1` | `pending` | SV up-opt breadth — design-first ADR + fresh probe + LRM grounding before code. **Deferred (not retired):** the `.2a` probe re-confirmed (per decision `0010`) that the named candidates (enum/typedef, packed multidim arrays) are accepted at every Verilator `--language` mode + Yosys + Icarus ⇒ not version-distinctive, no down-gating teeth; the genuinely-2023 clean space with the installed tools is thin (essentially `union soft`, shipped). A future `.1` either finds a genuinely-2023 construct or rescopes to `union soft` breadth. |

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

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `CAPABILITY-BREADTH-EXPANSION` | `USABILITY-LANE-OWNERSHIP.1 — register 7 owner-directed usability/capability lanes + API-first decision 0017` | Tree registered (not yet started); frontier `.1` (SV up-opt design ADR) + `.2` (Mealy FSM design ADR) pending. |
| `CAPABILITY-BREADTH-EXPANSION.2a` | `CAPABILITY-BREADTH-EXPANSION.2a — Mealy FSM output design ADR (decision 0024)` | Design ADR (docs-only). Pins the Mealy `(state_q, sel)` output model, `fsm_mealy_prob` knob, `num_mealy_fsm_modules` metric (schema `1.13`), `saw_mealy_fsm_design` gate, MCP surface. `.2` split into `.2a` (done) + `.2b` (proposed). |

## Changelog

- `2026-06-17`: Created task tree (registration via `USABILITY-LANE-OWNERSHIP.1`).
- `2026-06-22`: `.2a` done — Mealy FSM output design ADR (decision `0024`); `.2`
  split into `.2a` (design, done) + `.2b` (impl, proposed); frontier advances to
  `.2b`. `.1` deferred (not retired) on the `.2a` not-version-distinctive probe.
