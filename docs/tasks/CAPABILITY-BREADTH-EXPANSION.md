# CAPABILITY-BREADTH-EXPANSION: more SV-2017/2023 up-opts + Mealy FSM outputs

## Metadata

- Tree ID: `CAPABILITY-BREADTH-EXPANSION`
- Status: `active`
- Roadmap lane: `Capability / breadth — high value-per-effort RTL surface additions (north star)`
- Created: `2026-06-17`
- Last updated: `2026-06-17`
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
  Status: `pending`
  Goal: `Mealy FSM outputs — design/decision leaf (ADR, no code): ground the Mealy extension in the real Phase-6 Fsm block + emitter (src/ir Fsm + Node::FsmOut + the encoding-derived emitter; Moore-only today), pin the Mealy output model (an output that also depends on the current input, default-off behind its own knob, valid-by-construction + synthesizable), the num_emitted_* metric + a tool_matrix coverage fact, and the MCP selectability/queryability (decision 0017). Record as the next decision record + pre-split impl.`
  Acceptance: `A decision record + a tree entry pinning the Mealy output model, the knob, the metric/gate, and the MCP surface; docs-only; INDEX + this tree + docs/TASK_TREE.md updated.`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `CAPABILITY-BREADTH-EXPANSION.1` | `pending` | SV up-opt breadth — directly continues the closed `SV-VERSION-TARGETING` pattern (`union soft` precedent), highest value-per-effort; design-first ADR + fresh probe + LRM grounding before code. |
| 2 | `CAPABILITY-BREADTH-EXPANSION.2` | `pending` | Mealy FSM outputs — extends the Phase-6 `Fsm`; also eligible; design-first ADR before code. |

## Decisions

- `2026-06-17`: Registered as an owner-directed capability-breadth lane. Binds
  decision [`0017`](../decisions/0017-api-first-everything-mcp-accessible.md)
  (each construct API-selectable + introspectable). Two parallel design-first
  strands (`.1` SV up-opts, `.2` Mealy FSM); each is its own decision record when
  picked. Reuses `SV-VERSION-TARGETING` (decisions `0009`/`0010`) and the
  Phase-6 FSM motif; nothing retired.

## Open Questions

- Which up-opt first (`enum`/`typedef` vs packed multidim arrays vs another 2023
  construct) — decided by the `.1` probe against the live toolchain + the SV LRM
  cache. *(Resolves at `.1`.)*
- Mealy output shape: a per-FSM-output Mealy/Moore choice vs. a whole-FSM mode,
  and how it interacts with the existing encoding-derived emitter. *(Resolves at
  `.2`.)*

## Blockers

- None. (Independent of the six usability lanes; reuses closed
  `SV-VERSION-TARGETING` + Phase-6 surfaces.)

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-06-17` | `CAPABILITY-BREADTH-EXPANSION` | `tree registered (docs-only); no code` | `registered` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `CAPABILITY-BREADTH-EXPANSION` | `USABILITY-LANE-OWNERSHIP.1 — register 7 owner-directed usability/capability lanes + API-first decision 0017` | Tree registered (not yet started); frontier `.1` (SV up-opt design ADR) + `.2` (Mealy FSM design ADR) pending. |

## Changelog

- `2026-06-17`: Created task tree (registration via `USABILITY-LANE-OWNERSHIP.1`).
