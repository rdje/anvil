# Development Notes
Engineering rationale behind design decisions. The "why" that does not belong in code comments and is too detailed for `MEMORY.md`.

For the canonical statement of the algorithm and load-bearing decisions, see `book/src/`. This file is the contributor-facing scratchpad: rejected alternatives, calibration notes, gotchas, and the reasoning behind small choices the book does not cover.

---

## 2026-06-22 — Doctrine-enforcement adoption (portable architecture #4) — `DOCTRINE-ENFORCEMENT-ADOPTION`

Decision `0026`. ANVIL already ran three portable architectures (task-trees,
`MEMORY_ARCHITECTURE.md`, the Knowledge Map) with their checks wired *directly*
into `.githooks/pre-commit` + CI. This tree adds the fourth — doctrine
enforcement — and unifies them behind one registry+driver
(`scripts/check_doctrines.sh`). Rationale + design choices that don't belong in
the decision record:

- **Why a driver over the existing checks, not a rewrite.** The two existing
  checks are registered verbatim; the driver only adds the meta-check
  (each registered check exists+executable — a dangling registry entry was
  previously possible) and a collect-all-results report. Editing the hook per
  new check is gone; adding a doctrine is one registry line.
- **Why the new checks are *structural co-staging proxies*, not oracles.** At
  pre-commit the subject line is not yet available (the `commit-msg` hook sees
  it, `pre-commit` does not), and re-running the cargo/`tool_matrix` oracle on
  every commit is too slow for the local gate (`DOCTRINE_ENFORCEMENT.md` §4(7)).
  So `CODE-CHANGE-EVIDENCE` / `TASK-TREE-OWNERSHIP` prove the *mandatory files
  are co-staged* (`CHANGES.md`+`MEMORY.md`; an owning `docs/tasks/*.md`); the
  un-fakeable leg is the `cargo test` + `tool_matrix` re-run at `COMMIT.md`/CI
  and the `commit-msg` leaf-id gate. §9 states this honest limit openly — the
  goal is expensive-and-visible non-compliance, not literal impossibility.
- **Why scope-aware.** A code-only doctrine must exempt pure docs/workflow
  commits (§4(5)) — otherwise it would block its *own* adoption (every leaf here
  is workflow/docs) and all future doc commits. The code globs are
  `src/|tests/|examples/|build.rs|Cargo.toml|Cargo.lock`; `scripts/`, `.githooks/`,
  `docs/`, `*.md` are explicitly **not** code.
- **bash 3.2 compatibility.** macOS ships bash 3.2, so the new checks avoid
  `mapfile`/`readarray` and use `printf … | grep` over the staged list. A
  `DOCTRINE_STAGED_OVERRIDE` env seam exists for the self-test only (documented
  as such; not a security boundary — `--no-verify` already exists).
- **`TOOLBOX.md` is ANVIL-specific (owner steer).** It catalogs ANVIL's *own*
  diagnostic instruments (trace/metrics/introspect/`analyze`/`coverage`/`validate`/
  `minimize`/`hunt`/`divergence`/`--diff-sim`/`tool_matrix` gates/snapshots/
  `ram_guard`), grouped by what you are diagnosing, not a generic debug toolbox —
  the first stop when a generated artifact misbehaves.
- **Cargo.lock counted as code.** A lockfile bump alters build behaviour, so a
  `Cargo.lock`-only commit is treated as a code change (must carry the evidence +
  an owning leaf). Conservative and consistent with the doctrine's scope wording.

## 2026-06-22 — Multi-output task surface — impl-time notes — `STRUCTURED-EMISSION-EXPANSION.12b.1`

The live surface implemented decision `0025` / the `.12a` design with **no
deviations** — the seven pinned choices held verbatim. Two notes worth keeping:

- **The cone primitives composed for free.** `render_cone_gate_expr` /
  `cone_operand_ref` were written for the cone-function surface (root + interior
  with an `interior_set`); passing an **empty `interior_set`** turns them into a
  pure "operand → folded literal | `a{position in params}`" renderer, which is
  exactly the deduplicated shared-formal body the multi-output task needs. No new
  body renderer was required — the `feedback_full_factorization` reuse the ADR
  predicted. Members **keep** their module wires (the per-gate assign loop emits
  the passthrough `assign <m> = <m>__mtv;`), so — unlike a cone-function interior
  — there is no use-count rule and DAG-shared members are fine.

- **Gate-shape calibration (carried to `.12b.2`).** The surface fires only on a
  co-supported *independent* pair (a fanout signal feeding two sibling gates that
  don't feed each other). A comb-only config with `terminal_reuse_prob = 0.6`,
  `max_depth = 2`, `min_outputs = 2` fires it readily: a forced
  `multi_output_task_emit_prob = 1.0` sweep (`/tmp/anvil-mo-sweep/`) emitted
  `__mt(` on 4/5 seeds (2–6 tasks each), all Verilator `-Wall` Δ=0 vs OFF across
  1800-2012/2017/2023 + Yosys both modes + Icarus, and seed 7's module was proven
  **exhaustively** sim-equivalent to the inline reference over all 128 input
  values. Seed 100's lone `-Wall` warning is a pre-existing `UNUSEDSIGNAL` on an
  unrelated `concat_0` wire (identical ON and OFF) — the established "residual is
  pre-existing, Δ=0" situation, not a projection bug.

## 2026-06-22 — Multi-output task surface — impl design-detail — `STRUCTURED-EMISSION-EXPANSION.12a`

The design-detail leaf for the sixth structured surface (decision `0025`, the
multi-output combinational `task automatic`). No source change; this entry grounds
the ADR in the real code and pins every `.12b` impl choice so the implementation
slice is mechanical. Read against `src/ir/task_emit.rs` (`gate_qualifies`),
`src/ir/cone_function_emit.rs` (`admissible` / `sibling_marked`), `src/emit/sv.rs`
(`render_gate_task_decl` / `render_gate_task_call` + the cone primitives
`cone_function_params` / `cone_operand_ref` / `render_cone_gate_expr` + the per-gate
assign loop ~`468`), `src/gen/mod.rs` (the `soft_union → function_emit →
generate_loop → task_emit → cone_function` roll chain, both the single + design
call sites), `src/config.rs` (the `*_emit_prob` knob list + the `Option<f64>` CLI
overlay), `src/metrics.rs`, and `src/ir/types.rs` (the `*_gates` Module fields).

**(1) The candidate predicate — replicate, don't share (the codebase convention).**
Each emit-projection pass (`function_emit`, `generate_loop`, `task_emit`,
`cone_function`) carries its **own** 4-line admissibility `matches!` (non-structured
— not `CaseMux`/`CasezMux`/`ForFold` — non-`Slice`, `>= 1` operand) plus
pass-specific sibling exclusions; none share one predicate. So
`multi_output_task_emit` gets its own local `admissible(node)` mirroring
`task_emit::gate_qualifies`, plus exclusion of every prior sibling mark
(`function_emit_gates` / `generate_loop_gates` / `task_emit_gates` /
`soft_union_slice_gates`) and of gates already consumed into an earlier
multi-output group this pass. This is **not** a `feedback_full_factorization`
violation: the "one mechanism" reuse is the **body rendering** (the cone
primitives, point 5) and the candidate *concept*, not the trivial `matches!`
(decision `0016`'s `cone_function_emit::admissible` likewise replicates it).

**(2) The Module carrier: `multi_output_task_groups: BTreeMap<NodeId, Vec<NodeId>>`,**
keyed by the group **leader** (lowest-`NodeId` member) → the **partner** members
(for the first-cut pair, a single-element `[partner]`); the full group is `key ++
value`. This mirrors `cone_function_gates: BTreeMap<NodeId, Vec<NodeId>>` (root →
interior) exactly: a `BTreeMap` so the emitter iterates in leader-`NodeId` order
(deterministic), and an emitter-surface annotation only — flat IR / validators /
CSE keys / `canonical_module_signature` untouched (a default-empty field added to
`Module`, like the four existing `*_gates`). A `member_set()` helper
(`groups.iter().flat_map(|(k,v)| once(k).chain(v))`) gives the "is this gate a
multi-output member" lookup the emitter + `cone_function` need.

**(3) Selection — `src/ir/multi_output_task_emit.rs`,
`annotate_multi_output_task_groups(m, rng, prob) -> usize`.** Skip `param_env`
modules (the `task_emit` scoping). Collect admissible, non-sibling-marked
candidates in ascending `NodeId`. Walk them in order with a `used: BTreeSet<NodeId>`;
for the lowest ungrouped candidate `ga`, **roll `gen_bool(prob)` once** (the leader
roll — reproducible; the call site guards on `prob > 0.0` so default `0.0` draws
nothing ⇒ byte-identical); if it fires, scan forward for the **next** ungrouped
candidate `gb` (ascending `NodeId`) such that (a) `operands(ga) ∩ operands(gb)` has
a **non-constant** member and (b) `ga`/`gb` are mutually fan-in-independent
(point 4); on the first such `gb`, insert `ga → [gb]` and add both to `used`. If the
roll fires but no partner qualifies, `ga` is left ungrouped (no group, no retry) —
it still emits inline. **One roll per leader** (not per pair) keeps the RNG draw
count a clean function of the candidate list, so a future widening to `k > 2`
won't perturb the pair-era stream for unaffected seeds. Returns the group count.

**(4) The soundness check — `in_fanin(m, target, root)` bounded backward DFS.**
The shared task reads its members' deduplicated direct operands and writes the
member output vars (whose passthrough `assign`s drive the member nets). If `gb`
were in `ga`'s transitive fan-in (or vice-versa), a member net would feed — through
gates outside the task — a direct operand the task reads, closing a combinational
cycle through the single `always_comb` task call (a Verilator `UNOPTFLAT`, even
though it converges functionally). So a pair is admissible only when **neither
member is in the other's operand cone**. Implementation: a DFS from `root`'s
operands over `Node::Gate` operands with a `visited: BTreeSet` (each node expanded
once ⇒ bounded by the cone size), returning `true` if `target` is reached. The IR's
**operand-topological `NodeId` invariant** (`Module::intern_gate` appends after its
operands ⇒ a gate's operands always have strictly smaller `NodeId`) means for
`ga < gb`, `gb ∉ fanin(ga)` is automatic, so only `in_fanin(m, ga, gb)` must be
checked — but the impl checks both directions for robustness (cheap; survives any
future invariant change). The DFS only needs to descend through `Node::Gate`
operands; primary inputs / flop `Q`s / instance outputs / constants are leaves
(never the `target` of interest since `target` is itself a gate).

**(5) Rendering — reuse the cone primitives for a deduplicated body.** The decl is

```systemverilog
task automatic <leader>__mt(output logic [W0-1:0] o0, output logic [W1-1:0] o1,
                            input logic [..] a0, input logic [..] a1, ...);
    o0 = <render_cone_gate_expr(op0, operands0, m, {}, params, names)>;
    o1 = <render_cone_gate_expr(op1, operands1, m, {}, params, names)>;
endtask
```

where `params = multi_output_task_params(m, members)` — the **deduplicated union**
of the members' **non-constant** direct operands, ascending `NodeId`
(`cone_function_params` adapted: union over members instead of root+interior, no
interior concept). Passing an **empty `interior_set`** to the existing
`render_cone_gate_expr` / `cone_operand_ref` makes every member operand resolve to
either a folded `Constant` literal or its boundary parameter `a{position in
params}` — exactly the shared-formal semantics (a shared non-constant operand →
one `a{i}` feeding multiple outputs). Output formals are `o{j}` (one per member, in
ascending-member order), input formals `a{i}` (one per dedup param); `o*` vs `a*`
never collide. The call site (one `always_comb` **per group** — mirrors the
single-gate task's one-call-per-gate; simpler than sharing a block):

```systemverilog
logic [W0-1:0] <m0name>__mtv;
logic [W1-1:0] <m1name>__mtv;
always_comb <leader>__mt(<m0name>__mtv, <m1name>__mtv, <node_ref(param0)>, <node_ref(param1)>, ...);
```

and in the per-gate assign loop each member's inline `assign` becomes the
passthrough `assign <mjname> = <mjname>__mtv;`. **Members keep their module wires**
(unlike `cone_function` interiors — multi-output members are co-equal roots, not
absorbed, so no use-count rule and DAG-shared members are fine). Names: task
`<leader>__mt`, per-member var `<mjname>__mtv` (suffixes unique vs `__t` / `__tv` /
`__f` / `__cf`; the gate detector greps `__mt(`, which does **not** substring-match
`__t(` because the char before `t(` is `m`).

**(6) Ordering + mutual exclusion.** The new pass runs **after `task_emit` and
before `cone_function`** in both `gen/mod.rs` call sites (the established "later
pass excludes earlier marks" chain). `cone_function_emit::sibling_marked` is
extended to also return `true` for any gate in the `multi_output_task_groups`
member-set, so a multi-output member is never a cone root or absorbed interior.
All knobs default `0.0` ⇒ no pass runs ⇒ byte-identical; order only matters when
several are on, and is documented.

**(7) Knob + metric + gate (the `.12b` split).** `Config::multi_output_task_emit_prob`
(default `0.0`, `0.0..=1.0` validation, dump-config) + a first-class
`--multi-output-task-emit-prob` `Option<f64>` CLI flag (the
`KNOB-ERGONOMICS-AND-PRESETS` convention, beside `--task-emit-prob` /
`--cone-function-emit-prob`). `Metrics::num_emitted_multi_output_tasks` (=
`m.multi_output_task_groups.len()`, `#[serde(default)]`) surfaced in introspection
`module_metrics` ⇒ `SCHEMA_VERSION` `1.13 → 1.14` (the metric bumps; the knob rides
the version — the `.10b` precedent). `tool_matrix --multi-output-task-gate` +
`ScenarioSet::MultiOutputTaskSweep` + `ModuleReport.emitted_multi_output_task`
(`__mt(` detection) + `saw_multi_output_task_emit` + the early-return gap arm,
templated on `--cone-function-gate`. **Gate-shape calibration (the one risk):** the
surface fires only when co-supported *independent* pairs exist — a fanout signal
feeding two sibling gates that don't feed each other. So the focus config sets a
high `terminal_reuse_prob` (≈`0.6`) to make gates reuse shared input terminals
(co-support) while keeping the cones shallow (`max_depth` small) so siblings stay
fan-in-independent; flagged for verification at `.12b.2`. Split `.12b` →
`.12b.1` (live surface: knob + Module field + `src/ir/multi_output_task_emit.rs` +
two rolls + emitter + `cone_function` exclusion + lib proofs + forced-sweep) +
`.12b.2` (metric + schema `1.13→1.14` + the `tool_matrix` gate) + `.12b.3` (book /
USER_GUIDE / README / KM). Default-off / DUT byte-identical throughout.

## 2026-06-22 — Mealy FSM output mechanism — impl-time refinements — `CAPABILITY-BREADTH-EXPANSION.2b.1`

The first **code** slice of the Mealy lane. Decision `0024` pinned the model; the
impl surfaced a few choices worth keeping:

- **`Fsm.mealy_outputs: Option<Vec<Vec<u128>>>` (not a `bool` + a reshaped table).**
  `None` ⇒ Moore (the default), `Some(table)` ⇒ Mealy. An `Option` field is the
  lowest-blast-radius shape: every existing Moore path sees `None` and behaves
  exactly as before, and `outputs` is retained untouched. Adding the field does
  **not** perturb any hash, because the FSM identity surfaces hash explicit field
  lists, not a derive: `FsmSignature` (the `merge_equivalent_fsms` key) and
  `canonical_module_signature` (which never even iterates `module.fsms` — it keys
  `FsmOut` by `(fsm, width)` only) are both unchanged ⇒ Moore stays byte-identical
  (snapshots 6/6).

- **`FsmOut` stays a fully-opaque leaf — the ADR's "fold `sel` into the FsmOut
  deps" was found unnecessary for soundness.** On inspection, `Node::FsmOut` has
  **no** `deps` field; its dep set is derived virtually (`from_fsm_virtual(fsm)` =
  one `FsmVirtual` atom). Non-triviality is already satisfied by that atom;
  `sel`'s cone stays reachable through compaction via the `fsm.sel` reference
  (the `fsmout_keeps_sel_cone_through_compaction` invariant), Moore or Mealy. So
  folding `sel` into the FsmOut deps would only refine **analyze** support-cone
  *fidelity* (a Mealy output's combinational input-dependence), not correctness —
  and would ripple into CSE keys / dedup / metrics for no soundness gain. Kept
  `FsmOut` opaque (consistent with `FlopQ`/`MemRead`); the analyze sel-fold is a
  deferred, clearly-scoped fidelity refinement, not a blocker.

- **Mealy FSMs are conservatively excluded from `merge_equivalent_fsms`.** A
  Mealy FSM's identity would have to key on the full 2-D `mealy_outputs` table;
  until that keying lands, a one-line `if fsm.is_mealy() { continue; }` leaves each
  Mealy FSM its own canonical block — sound (never an incorrect merge), nothing
  retired (the memories-stay-state-by-instance precedent). Moore merge is
  untouched. Whole-module dedup already excludes all FSM modules
  (`sequential_leaf_eligible` ⇒ `!has_local_fsms()`), so no module-level work was
  needed.

- **The Mealy table is a pure `(state, sel_value)` formula, not an RNG draw.** The
  only RNG consumed for Mealy is the single gating `gen_bool(fsm_mealy_prob)`; the
  table values come from a deterministic `(s, j)` hash so the output genuinely
  varies with the input `sel` without perturbing the RNG stream beyond that one
  roll. The emitter's Moore `else`-branch reproduces the prior bytes exactly, so
  `fsm_mealy_prob == 0.0` is byte-identical.

## 2026-06-22 — Mealy FSM output model + the `.1`-vs-`.2` ordering call — `CAPABILITY-BREADTH-EXPANSION.2a`

The design ADR (decision `0024`) for the Mealy strand of
`CAPABILITY-BREADTH-EXPANSION`. The deeper rationale behind two non-obvious
choices, kept here:

- **Reuse `sel`, do not invent a second input cone.** A Mealy output is, by
  definition, a function of the current state *and the current input*. The Phase-6
  `Fsm` already carries exactly one input-dependent cone: `sel` (the
  transition-select source, a real `NodeId`, dependency-tracked + validated). The
  clean model makes the Mealy output a per-`(state, sel_value)` constant table —
  the **structural twin of `transitions`** — so the output decode is the *same*
  nested `case (state_q)` → `case (sel)` shape the emitter already produces for the
  next-state logic (and which is already downstream-proven). Inventing a fresh
  output-input cone would create a second input notion beside `sel`, complicate the
  `FsmOut` virtual-deps, and break the 1:1 parallel with `transitions` — a
  `feedback_full_factorization` smell (two cone notions where one suffices).
  Behaviour stays defined **by construction** from the generated table, not derived.

- **The one real subtlety: `FsmOut` deps under Mealy.** Today `FsmOut` is an opaque
  leaf with virtual deps `DepSet::from_fsm_virtual(fsm)` (Moore: output = f(state),
  no comb input path). Mealy gives `FsmOut` a genuine combinational path from
  inputs through `sel`, so the virtual dep set must fold in `sel`'s support or
  non-triviality / IR validation / dedup would under-count the support. Flagged for
  `.2b` (touches `metrics.rs` + `compact.rs` parallel sites). `FsmOut` stays opaque
  to CSE either way — only its *decode* changes.

- **Why `.2` (Mealy) jumped ahead of frontier-ordered `.1` (next SV up-opt).** The
  `.1` leaf is a *probe-and-decide* leaf; I ran its probe first. The two named `.1`
  candidates — `enum`/`typedef` and packed multidimensional arrays — are accepted
  at **every** Verilator `--language 1800-2012/2017/2023` mode **and** by Yosys and
  Icarus, i.e. they are legal at the 1800-2012 floor ⇒ **not version-distinctive,
  no down-gating teeth** (the exact bar decision `0010` set for an up-opt). That
  re-confirms `0010`'s finding that the genuinely-2023 *synthesizable* space with
  the installed tools is essentially just `union soft` (shipped). So `.1`'s "next
  up-opt" yield is thin/uncertain, whereas Mealy is genuinely-new, **all-tool-clean**
  (a stronger downstream story than the Verilator-only `union soft`), and a small
  bounded extension of a proven motif — higher value-per-effort *with confidence*.
  `.1` stays `pending` with the probe evidence recorded (`feedback_never_retire_strategies`);
  self-selecting `.2` follows `feedback_pick_and_roll_at_no_frontier`.

## 2026-06-21 — Coverage-steered generation outer loop (derive + `--steer`) — `COVERAGE-STEERED-GENERATION.2c.1`

The third **code** slice of the steering lane: the *derive* step of the outer
measure→derive→re-steer loop + the `--steer` CLI shim. Notes worth keeping:

- **The feedback loop is OUTER, not in-generator — so `derive` is a pure
  `CoverageReadout → SteeringConfig` function.** This is the reconciliation with
  `feedback_rules_first_generation` (decision `0023` §4): each generation pass
  stays a pure rules-first function of `(seed, knobs, steering-config)`; the
  "feedback" is three separate, deterministic steps the orchestration runs —
  *measure* (`.2b`'s `coverage` readout) → *derive* (`.2c.1`'s helper) → *re-steer*
  (regenerate with the new steering-config). `derive_steering_from_coverage` never
  touches the generator and never filters — it just maps under-hit categories to
  up-weights via `weight = clamp(target_share / max(observed, eps), 0, max_weight)`.
  Per-category (not per-knob) by default because that is the granularity a
  `SteeringConfig` targets and the coarsest useful rebalancing lever.

- **Derived weights are milli-quantized — the `.2b` determinism lesson applied
  forward.** A steering weight is an *input* to a future generation run, so if
  `derive` produced a raw `f64`-division weight it could differ by 1 ULP between
  machines and silently break `(seed, knobs, steering-config)` reproducibility (the
  exact hazard `.2b` hit on `fire_rate`). Each weight is quantized to milli
  (`(w*1000).round()/1000`) — far finer than any steering decision needs, and the
  `.round()` collapses sub-milli divergence. Milli (not ppm) here because weights
  range up to `max_weight` (e.g. 8), not `[0,1]`.

- **One classifier for `--steer`, reusing the existing taxonomy.**
  `SteeringConfig::set_weight` is the single place that decides whether a `--steer`
  key is a knob name (→ `per_knob`) or a category (→ `per_category`), via the
  `KnobId::category_of_name` / `KnobId::all` inversion added in `.2b` — no second
  name/category table (`feedback_full_factorization`). Knob names and category
  names are disjoint, so the classification is unambiguous; an unknown key errors
  naming the categories (the cold-path list is built from `KnobId::all`, not a
  hand-kept constant).

- **Why the fallible `--steer` parse lives in `resolve_config`, not
  `apply_cli_overrides`.** `apply_cli_overrides(&mut self)` is infallible (it just
  copies `Option` fields), but `--steer` can fail (unknown key / bad weight). Rather
  than make the whole override-application fallible (it is called from several
  places), the steer pairs are applied in `resolve_config` (which already returns
  `Result` and already calls `validate`). MCP leaves `Overrides.steer` empty (it
  sets steering via the `config` JSON `steering` block), so this adds no MCP
  behaviour and the shared resolver stays one path. Precedence: preset-steer then
  explicit-steer then `validate`, so explicit `--steer` beats a preset on the same
  key and the merged weights are range-checked once.

## 2026-06-21 — Coverage-steered generation readout — `COVERAGE-STEERED-GENERATION.2b`

The second **code** slice of the steering lane: the achieved-coverage *readout*
(the **read** half), surfaced both embedded in `--introspect` and as a standalone
MCP `coverage` query. Implements decision
[`0023`](docs/decisions/0023-coverage-steered-generation.md) §3/§5. Notes worth
keeping:

- **A genuine FP-determinism gotcha — integer fixed-point fire rate.** The
  readout exposes the empirical fire rate (`fires / attempts`). The first cut
  computed it as `fires as f64 / attempts as f64`. The pre-existing exact-equality
  test `mcp::introspect_tool_round_trips_to_the_schema_document` (which compares
  the MCP-serialized document against a fresh `module_document` recompute) caught a
  **1-ULP divergence**: `comb_mux_prob` came out `0.11188325225851284` in the MCP
  build path and `…285` in the recompute path — same integer inputs (161/1439),
  different last bit. The two divisions are correctly-rounded IEEE in isolation,
  but the compiler can fold one call site at compile time (LLVM APFloat) and run
  the other at runtime, and the rounding of those two paths is not *guaranteed*
  bit-identical for every operand. For a project whose whole contract is
  byte-identical output, a float that varies by evaluation context is unacceptable
  in the document. Fix: compute the rate as a round-half-up **integer
  parts-per-million** quotient (`(fires*1e6 + attempts/2)/attempts`, all `u64`)
  then one exact `u64 → f64 / 1e6`. The integer quotient is identical across call
  sites by construction; the single final division of a specific integer by `1e6`
  is correctly rounded and unique. `attempts`/`fires` remain the exact integers, so
  no precision is lost — the float is just the convenience projection (6 dp is far
  finer than any steering decision needs). Lesson: **never serialize a raw
  `f64`-division result into a determinism-contracted document; reduce it to
  integer arithmetic + one final exact conversion.** (`avg_fanout` etc. survive
  because both compared documents call the *same* `compute()` site once — they
  never recompute the float at a *second* site; the fire rate did, exposing the
  hazard.)

- **Embedded *and* standalone — but one projection.** Unlike the `analyze`
  derived-relation surface (kept out of the default document, decision `0011` Q2,
  because a support cone is `O(nodes)`), decision `0023` explicitly puts the
  coverage readout *in* `--introspect`: it is small and bounded (≤ ~22 knobs + a
  6-name category roll-up + three small histograms) and is intrinsically a property
  of *that run*. The MCP `coverage` tool returns the **same** `CoverageReadout`
  embedded in the document (it reuses `doc.introspection.coverage_readout`, never
  recomputes), so the two surfaces cannot drift — `feedback_full_factorization`
  (one classifier, not two).

- **Why a `category` roll-up is in the readout.** The outer steering loop
  (decision `0023` §4) derives a `SteeringConfig` from under-hit constructs, and a
  `SteeringConfig` targets per-knob **and** per-category weights. So the readout
  ships both granularities (`knob_fire_rates` + `category_fire_rates`) directly,
  saving the agent the roll-up (`feedback_api_for_agents_not_humans`). The
  per-category pool is **attempt-weighted** (`sum(fires)/sum(attempts)`), not an
  average of per-knob rates — the honest pooled rate. `KnobId::all()` +
  `category_of_name` invert the existing `name`→`category` table so there is no
  second copy of the mapping.

- **Why the `saw_*` facts are NOT in the readout.** Decision `0023` §3 lists the
  matrix coverage facts as part of the conceptual readout, but a single artifact
  cannot prove `saw_recursive_hierarchy_*` (those need a `tool_matrix` corpus). The
  existing `coverage` section (schema §6.4) is matrix-only for exactly this reason;
  the per-artifact `coverage_readout` is the orthogonal single-run projection. Kept
  them as two distinct sections (no conflation).

## 2026-06-21 — Coverage-steered generation core — `COVERAGE-STEERED-GENERATION.2a`

The first **code** slice of the steering lane: the `SteeringConfig` type + the
`weight()` lookup + the one-line prior multiplier at `roll_knob`. Implements
decision [`0023`](docs/decisions/0023-coverage-steered-generation.md) exactly as
the tree's "Implementation Notes (for `.2a`)" pre-pinned it. Engineering notes
worth keeping:

- **Byte-identity tactic, two layers of safety.** The unsteered path must stay
  byte-identical. `roll_knob` was `gen_bool(prob.min(1.0))`; it is now
  `gen_bool(steering.effective_prob(knob, prob))`. `effective_prob` **short-circuits
  to `prob.min(1.0)` when the config is empty** (the default), so the truly-default
  path is byte-identical by construction — independent of any float reasoning. The
  multiplier path `(prob * weight).clamp(0.0, 1.0)` is *also* bit-exact at
  `weight == 1.0` for `prob ∈ [0,1]` (multiplication by 1.0 is exact; clamp is a
  no-op in range; `clamp(0,1) == min(1.0)` there), proven by
  `neutral_steering_weight_is_byte_identical_to_unsteered` (explicit `1.0` weights
  produce identical SV across 16 seeds). The `is_empty()` short-circuit is the
  belt; the exact multiplier is the suspenders.
- **`skip_serializing_if` was a deliberate first.** `config.rs` had **zero**
  `skip_serializing_if` — every knob always serialized. The `steering` field is the
  first, gated on `SteeringConfig::is_empty`, so an unset block is *omitted* from
  `--dump-config`/`--introspect` JSON ⇒ those stay byte-identical when unset, and
  the introspection schema bump is correctly deferred to `.2b` (the readout). Old
  configs without a `steering` key still deserialize (the field is `#[serde(default)]`).
- **Category taxonomy is exhaustive at compile time.** `KnobId::category()` is a
  wildcard-free match over all 21 variants into a fixed 6-name taxonomy
  (`state`/`selectors`/`datapath`/`terminals`/`sharing`/`hierarchy`), so a future
  knob *must* declare a category — drift is a compile error, not a silent gap.
- **Weight resolution order is most-specific-first:** `per_knob` (by
  `KnobId::name()`) → `per_category` (by `KnobId::category()`) → neutral `1.0`.
  Validation rejects negative or non-finite weights (`ConfigError::SteeringWeight`),
  mirroring the existing probability-range checks; `0.0` is legal (fully suppress a
  construct).
- **No-filter is architectural, not a runtime check.** There is exactly one
  `gen_bool` per `roll_knob` and no rejection branch — the multiplier biases the
  *prior*, it never builds-then-discards. The proof is the unchanged draw structure
  plus the byte-stability tests; `feedback_rules_first_generation` is satisfied by
  construction.
- **Scope held to the ADR pre-split.** `.2a` is the core only. The SCHEMA-DERIVED
  achieved-coverage readout (`--introspect` schema bump + MCP coverage query) is
  `.2b`; the `--steer` CLI shim, the outer `derive_steering_from_coverage` helper,
  and the book/USER_GUIDE/KM-card are `.2c`. The `steering` block is already
  `--config`-JSON-settable today, but its *effect is not yet observable* without the
  `.2b` readout, which is why the user-facing docs intentionally wait for `.2c`.

## 2026-06-21 — Coverage-steered generation design — `COVERAGE-STEERED-GENERATION.1` (decision 0023)

`.1` is the design ADR for biasing generation toward under-exercised constructs
**without** generate-then-filter. Full rationale: decision
[`0023`](docs/decisions/0023-coverage-steered-generation.md). The load-bearing
choices worth keeping:

- **Steering is a prior multiplier at the one instrumented decision site, not a
  filter.** Every steerable construction choice already flows through
  `roll_knob(g, m, knob, prob)` (`src/gen/cone.rs`), which does exactly one
  `rng.gen_bool(prob)` and records `knob_roll_attempts`/`fires`. Steering inserts
  `effective_prob = clamp01(prob * weight(knob))` before that single draw. This is
  the crux of the `feedback_rules_first_generation` reconciliation: it biases the
  *prior* of a decision; there is no rejection branch, no second artifact, no
  build-then-discard. The forbidden mode (generate-then-filter) is rejected
  outright in the ADR.
- **One draw per roll ⇒ byte-stable; identity when unset ⇒ byte-identical
  default.** The RNG draw count is unchanged, so output is byte-stable per
  `(seed, knobs, steering-config)`; with no steering-config every `weight` is
  `1.0` ⇒ `effective_prob == prob` exactly ⇒ `tests/snapshots.rs` untouched.
- **The "feedback" is an OUTER loop, not an in-generator one.** measure (read the
  achieved-coverage readout) → derive (a pure function maps under-hit categories
  to up-weights) → re-steer (re-run with that steering-config). Each generation
  pass stays a pure, rules-first function of its inputs — mirroring how
  `coverage_gaps` already works. An in-`--count` adaptive schedule is more
  powerful but couples units within a run (unit N depends on count/order), so it
  is deferred to a follow-up `.N`.
- **Readout reuses existing telemetry (zero new truth).** The achieved-coverage
  query is a SCHEMA-DERIVED projection of `knob_roll_attempts`/`fires` + the
  gate/operand/depth histograms + `CoverageSummary saw_*` (the decision `0011`
  precedent) — not a new computed metric.
- **First cut steers only the `roll_knob`/`KnobId` surface.** Raw `gen_bool` sites
  (`src/gen/mod.rs`) and weighted-choice sites (`gate_struct_weight`) lack
  telemetry, so steering them blind is unprovable; routing them through `roll_knob`
  (telemetry + steerability together) is deferred. Keeps the rules-first proof
  bounded.
- `.2` pre-split `.2a` (steering core + byte-identical-off / distribution-shift /
  no-filter proofs — **code**, task-tree-owned) / `.2b` (the SCHEMA-DERIVED readout
  + MCP coverage query) / `.2c` (the outer measure→derive→re-steer helper + docs +
  KM; close). Docs-only so far / DUT byte-identical.

## 2026-06-21 — CI packaging `.2c`: "Use ANVIL in your CI" docs + KM card (closes `.2`) — `CI-PACKAGING-DISTRIBUTION.2c` (decision 0022)

`.2c` is the docs-and-close slice. Two choices worth recording:

- **A separate KM usage card, not an edit to decision `0022`'s answers.** The
  `0022` ADR already answers the *design* questions ("is there an ANVIL GitHub
  Action", "what release mechanism…"). The shipped Action raises new *usage*
  questions ("what inputs does the Action take", "how do I pin it to a release",
  "does it fail my CI on a finding"). I wrote a new `docs/knowledge/ci-github-action.md`
  card with answers **disjoint** from `0022`'s, rather than mutating the ADR's
  front-matter — so there are no duplicate question keys (the KM derive-and-diff
  gate stays clean) and the design vs usage facts each have one canonical home
  (the KM "one canonical card per fact" rule). `0022`'s *body* got a one-line
  "implemented" status note (front-matter untouched ⇒ KM map unchanged).
- **The book recipe leads with a `yaml` fence.** The `book_examples` harness only
  runs ```bash fences; the `uses:` GitHub-Action snippet is ```yaml, so it is
  documented without being executed (a CI runner is not the book-test
  environment). The one `anvil hunt` bash line in the recipe invokes external
  tools, so it carries the `<!-- book-test: skip — … -->` sentinel on the line
  immediately before the fence (the harness requires that exact placement) — the
  byte-identical book-runnable contract (54 runnable blocks) is preserved.

Docs-only; default DUT output byte-identical. `.2c` closes `.2` (the
implementation); the tree stays `active` with no current frontier — more targets,
a Marketplace listing, or an MCP-driven Action variant are optional `.N` picks.

## 2026-06-21 — CI packaging impl `.2b`: the drop-in composite Action over `anvil hunt` — `CI-PACKAGING-DISTRIBUTION.2b` (decision 0022)

`.2b` implements the composite GitHub Action the `.1` ADR pinned: a root
`action.yml` + `scripts/anvil_hunt_action.sh` entrypoint + a presence-gated
self-test workflow. The non-obvious choices worth keeping:

- **Root `action.yml`, entrypoint under `scripts/`.** Placing `action.yml` at the
  repo root gives the simplest public surface (`uses: <owner>/anvil@<tag>`); the
  entrypoint lives with the other repo helpers in `scripts/` and is located at
  runtime via `${GITHUB_ACTION_PATH}/scripts/anvil_hunt_action.sh`
  (`GITHUB_ACTION_PATH` = the action's checkout root = the repo root here).
- **Pin-by-ref, with no hardcoded repo.** The entrypoint downloads the release
  from `https://github.com/${GITHUB_ACTION_REPOSITORY}/releases/download/
  ${version}/anvil-${version}-${target}.{tar.gz|zip}`, where `version` defaults
  to `${GITHUB_ACTION_REF}` — so pinning the Action to `@v0.1.0` automatically
  pins the *binary* to the `v0.1.0` release, with nothing repo-specific baked in.
  The target is mapped from `RUNNER_OS`/`RUNNER_ARCH` to the `.2a` triples.
- **`anvil hunt` always exits 0 → the Action decides red/green.** `run_hunt_command`
  prints the `HuntReport` JSON and returns `Ok(())` regardless of findings
  (`src/main.rs`), so the entrypoint parses `summary.n_failures` (python3,
  preinstalled on every runner — no `jq` dependency) into a `findings` step
  output, and `action.yml`'s `fail-on-finding` step does `exit 1` when it is
  nonzero. The CLI is a pure shim (decision 0017); the policy lives in the Action.
- **`anvil-bin` + `artifact-name` are Action-level plumbing beyond the ADR's 1:1
  hunt-flag table.** The ADR table already pairs the hunt flags with two
  Action-level inputs (`anvil-version`, `fail-on-finding`). `.2b` adds two more in
  that same category: `anvil-bin` (use a prebuilt binary instead of downloading —
  required for the self-test before any release exists, and useful for users who
  build from source or consume the `.2a` artifact directly) and `artifact-name`
  (override the upload name to avoid `actions/upload-artifact@v4`'s unique-name
  collision in a matrix). Neither forks the engine — both still run `anvil hunt`.
- **An absent downstream tool is a *finding*, not a no-op.** `downstream::run_tool`
  reports a spawn failure as `success:false` (`src/downstream/mod.rs`), which
  `validate`/`hunt` count as a failure. So the self-test cannot just run the hunt
  blindly on a tool-less runner — it probes `command -v verilator/yosys`, runs the
  Action only when ≥1 is present, and otherwise **skips clean** (the ADR's
  "skips clean when absent"). It uses `anvil-bin: target/release/anvil` so no
  published release is needed.
- **Validated by a real local smoke**, not just a lint: the entrypoint was run
  against the local release `anvil` + verilator + yosys over a 3-seed sweep —
  exit 0, `findings=0`, valid-JSON report (`n_clean:3, n_failures:0`), outputs
  wired. Default DUT output stays downstream-clean, as the project thesis
  predicts.

This slice touches no Rust; default DUT output stays byte-identical. The
user-facing "Use ANVIL in your CI" docs + the KM card are `.2c` (which closes `.2`).

## 2026-06-21 — CI packaging impl `.2a`: the hand-rolled `v*`-tag release workflow — `CI-PACKAGING-DISTRIBUTION.2a` (decision 0022)

`.2a` implements the release path the `.1` ADR pinned: `.github/workflows/release.yml`,
a tag-triggered 5-target build matrix that publishes `anvil`+`anvil-mcp` archives +
`SHA256SUMS` to the GitHub Release. The non-obvious choices worth keeping:

- **Toolchain pin via env, not a `rust-toolchain.toml`.** The pin lives in the
  workflow (`RUST_TOOLCHAIN: '1.95'`, tracking `Cargo.toml`'s `rust-version` MSRV)
  rather than a repo-root `rust-toolchain.toml`, because the latter would silently
  re-pin *every* build (local + the existing `ci.yml`, which uses `@stable`) — a
  blast radius wider than this slice owns. Bump the env and `Cargo.toml` together.
  The load-bearing reproducibility guarantee is the release tag + committed
  `Cargo.lock` (`--locked`) + ANVIL's platform-independent ChaCha8 generation, **not**
  binary bit-for-bit identity — so the toolchain pin is belt-and-suspenders, not the
  contract.
- **aarch64-Linux is the only cross target.** Its GNU cross linker
  (`gcc-aarch64-linux-gnu`) is installed on the x86_64 ubuntu runner and selected
  with the per-target `CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER` env override
  (appended to `$GITHUB_ENV`) instead of writing a `.cargo/config.toml`, so no other
  build (or a developer's local tree) is affected. The four other targets build
  natively (two macOS runners x86_64/arm64; windows-msvc; x86_64-linux).
- **`gh` CLI to publish, not a third-party release action.** `gh` is preinstalled on
  GitHub runners, so `gh release view || gh release create … && gh release upload
  --clobber` (idempotent on re-run) keeps the no-new-dependency ethos decision `0022`
  invokes — the same reasoning that rejected `cargo-dist`. `actions/checkout`,
  `actions/upload-artifact`, `actions/download-artifact`, `dtolnay/rust-toolchain`,
  and `Swatinem/rust-cache` are the same first-party / already-in-`ci.yml` building
  blocks, not new deps.
- **Flat archives; both binaries + `README.md`.** Unix `.tar.gz` and Windows `.zip`
  both place `anvil`(`.exe`)/`anvil-mcp`(`.exe`)/`README.md` at the archive root
  (no top-level dir) for a uniform extract experience. No `LICENSE*` file exists in
  the repo (the license text lives in `README.md`'s License section), so the README
  is what carries it into the bundle.
- **Least privilege.** Top-level `permissions: contents: read`; only the `publish`
  job escalates to `contents: write` (needed for `gh release`). The build matrix
  jobs cannot mutate the repo.
- **Tag-only trigger (no `workflow_dispatch`).** Faithful to the ADR ("triggered on
  `v*` tags"); a half-working manual path (dispatch with no tag context) would be a
  worse signoff outcome than precision. A Marketplace listing / more targets / an
  MCP-driven variant remain optional post-`.2c` `.N` work, per decision `0022`.

This slice touches no Rust; default DUT output stays byte-identical. Validated by a
pure-Python structural lint (offline: no `pyyaml`/`actionlint`/`yq`) + the mem-arch
and KM gates.

## 2026-06-18 — CI packaging design: hand-rolled release + composite Action over `anvil hunt` — `CI-PACKAGING-DISTRIBUTION.1` (decision 0022)

`.1` is the design ADR for the drop-in CI lane. Full rationale: decision
[`0022`](docs/decisions/0022-ci-packaging-prebuilt-binaries-and-github-action.md).
The load-bearing choices worth keeping:

- **Hand-rolled release workflow over `cargo-dist`.** The project has a clear,
  stated dependency-averse / transparent-infra ethos (the README's "hand-rolled
  loopback-default transport … no new dependency" for the MCP HTTP transport). A
  small in-repo `release.yml` matrix is auditable and adds no release-tool
  dependency; `cargo-dist` is more opinionated + generates config to maintain. Same
  reasoning that picked a hand-rolled transport picks a hand-rolled release.
- **Composite Action, not a container.** A container action tempts vendoring the
  downstream tools (Verilator/Yosys) into the image — directly against the
  no-vendoring non-goal (licensing/size/staleness; the user cares about *their*
  tool versions' bugs). A composite action runs on the user's runner with the
  user's installed tools, and is transparent (just steps).
- **The Action is a pure shim over `anvil hunt` — no Action-only path** (decision
  `0017`). Every Action input maps 1:1 onto an `anvil hunt` flag / `hunt`-tool
  control, so CI usage and agent usage share one engine. This is why the lane could
  be designed now: the engine (`anvil hunt`, decision `0018`; `divergence`,
  decision `0019`) **already ships**, so `.2` wraps a real binary, and the
  `--profile` presets (decision `0021`) drop straight in as an Action input.
- **Reproducibility is inherited, not rebuilt.** A CI finding reproduces locally
  because each hunt bundle already carries `repro.sh` + `knobs.json` (byte-identical
  regen) and the hunt is seeded-deterministic; the Action only adds the
  `anvil-version` pin. No new reproducibility machinery.
- `.2` pre-split `.2a` (release.yml — task-tree-owned CI infra) / `.2b` (action.yml +
  entrypoint + self-test) / `.2c` (README/USER_GUIDE "Use ANVIL in your CI" + KM).

---

## 2026-06-18 — Knob ergonomics impl: 16 promoted CLI flags + presets + the shared resolver — `KNOB-ERGONOMICS-AND-PRESETS.2b.1`

`.2b.1` lands the `.2a` design. Two implementation points worth keeping:

- **The four dedup/identity bools are `SetTrue` flags mapped to `Some(true)` only
  when present** (`cli.hierarchy_module_dedup.then_some(true)` in `cli_overrides`).
  A `SetTrue` clap flag is a plain `bool` (false when absent); mapping the absent
  case to `None` (not `Some(false)`) is what preserves "an unset flag never
  clobbers a preset/config value." These knobs are off-by-default opt-ins, so an
  on-only CLI flag is the right ergonomic — you never need `--…-dedup false`, you
  just omit it. The 12 prob/u32 knobs are already `Option<T>` clap flags, so they
  map 1:1.
- **`resolve_config` replaced the inline `apply_cli_overrides; seed; validate` in
  `main.rs`'s DUT path** and is the single resolver the MCP `profile` input will
  reuse in `.2b.2`. With `profile = None` and only-absent promoted flags it is
  bit-for-bit the old path (proven: `tests/snapshots.rs` 6/6 unchanged + the
  `resolve_config_default_path_is_default_plus_seed` unit test). `Config` is not
  `PartialEq`, so the default-path test compares `serde_json::to_value` (a clean
  byte-identical proxy). `Config` field defaults are untouched, so `--dump-config`
  of a default run is unchanged too.
- **User-facing docs (USER_GUIDE / README / `book/src/knobs.md` + the
  `--profile` walk-through / KM card) are deferred to the `.2b.3` docs-closeout
  leaf**, following the structured-emission precedent (`.2b.1` impl → `.2b.3`
  docs). Tracked, not silent — landing in this same session.

---

## 2026-06-18 — Knob ergonomics impl design-detail: reuse `Overrides` as the preset carrier; one shared resolver — `KNOB-ERGONOMICS-AND-PRESETS.2a`

`.2a` pins the exact Rust shapes for the decision-`0021` design so `.2b` is a clean,
well-owned code slice. Grounded in the current code:
`Config::apply_cli_overrides(&Overrides)` (the Option-by-field applier),
`main.rs` resolution (`base = --config full | Config::default(); apply_cli_overrides;
cfg.seed = cli.seed; validate`), and `mcp::config_from_args` (full `config` arg →
`Config` → validate). The pinned design-detail:

- **The preset carrier IS `Overrides` — no second partial-config type.** A preset is
  a named bundle of partial overrides, which is *exactly* what `Overrides` already
  is (all-`Option` fields, applied by `apply_cli_overrides`). So a preset reuses
  `Overrides`, and applying a preset is just a second `cfg.apply_cli_overrides(&preset.overrides)`
  call — one applier, not two (full-factorization, `feedback_full_factorization`).
  Consequence: a preset can set any **CLI-overridable** knob (the 66 existing + the
  16 `.2b.1` promotes). The 3 kept-config-only knobs (`library_prob`,
  `use_async_reset`, `max_nodes_per_module`) are not preset-settable in the first cut
  — none of the 4 curated presets need them; promoting one later (additive) is the
  path if a future preset does. `Overrides` gains `#[derive(Serialize)]` with
  per-field `skip_serializing_if = "Option::is_none"` so a preset's override set is
  enumerable for `anvil://catalog/presets` (decision 0017 queryability).

- **`Overrides` needs `Default` (all-`None`).** Presets are built with struct-update
  syntax: `Overrides { function_emit_prob: Some(1.0), cone_function_emit_prob: Some(1.0), ..Default::default() }`.
  The registry is one function `pub fn presets() -> Vec<Preset>` (the single source
  of truth) returning `Preset { name: &'static str, description: &'static str, overrides: Overrides }`
  for the 4 curated presets — deterministic, no randomness. `pub fn lookup_preset(name) -> Option<&Preset>`
  (or returns owned) for resolution; unknown name is an error.

- **One shared resolver, used by both the CLI and MCP (CLI is a shim).** Signature
  ~ `fn resolve_config(base: Config, profile: Option<&str>, overrides: &Overrides, seed: u64) -> Result<Config>`:
  apply the looked-up preset's `Overrides` to `base`, then apply the explicit
  `overrides`, then stamp `seed`, then `validate()`. Order =
  `default|--config(base) → profile → explicit → seed` (decision 0021). On the CLI,
  `overrides = cli_overrides(&cli)` (only user-passed flags ⇒ explicit beats preset).
  On MCP, the first cut has **no** per-knob `overrides` above the profile (the
  `config` arg is the full base, `profile` layers on top); a partial MCP `overrides`
  input is a recorded additive `.N`. Unknown profile ⇒ a CLI error listing valid
  names / MCP `-32602` (the existing unknown-query convention).

- **`config_from_args` gains a `profile` arg.** `let cfg = base-from-config-or-default;
  if let Some(p)=args["profile"] { apply preset }; seed; validate`. The MCP
  `generate`/`introspect`/`analyze` input schemas add `profile: string` (optional),
  documented as "a curated knob bundle; see anvil://catalog/presets".

- **The rich knob catalog mirrors `downstream::adapter_catalog()`.** A new
  `fn knob_catalog() -> Vec<KnobInfo>` where
  `KnobInfo { name, group, ty, default: Value, validation: ValidationKind, cli_flag: Option<String>, config_only: bool }`.
  Names + defaults are projected from `serde_json::to_value(Config::default())`
  (derived); `group`/`validation`/`cli_flag` come from one metadata table keyed by
  field name. **Completeness test:** assert the set of catalog `name`s equals the set
  of keys in `to_value(Config::default())` (no missing, no orphan) — a new `Config`
  field then *must* get a catalog entry or the test fails (the KM derive-and-diff
  anti-drift pattern). Surfaced as a new `anvil://catalog/knob-schema` resource
  (+ optionally a pure `knob_catalog` MCP tool); the raw `anvil://catalog/knobs`
  (`Config::default()` dump) is kept untouched (no retirement).

- **`.2b` re-split for signoff-sized slices:** `.2b.1` = `Overrides`
  `Default`/`Serialize` + the 16 promoted `Option` CLI flags + the preset registry
  + the shared resolver + `--profile` CLI flag + byte-stability / explicit-beats-preset
  proofs (DUT byte-identical: no `--profile` ⇒ snapshots untouched). `.2b.2` = the
  SCHEMA-DERIVED `knob_catalog` + completeness test + `anvil://catalog/knob-schema`
  + `anvil://catalog/presets` + the MCP `profile` input. `.2b.3` = docs
  (`book/src/knobs.md` + `book/src/agent-mcp.md` + USER_GUIDE + README + KM card).

---

## 2026-06-18 — Knob ergonomics design: promotion cut, declarative presets, resolution order — `KNOB-ERGONOMICS-AND-PRESETS.1` (decision 0021)

`.1` is the design ADR for the knob-ergonomics lane. The full rationale is decision
[`0021`](docs/decisions/0021-knob-ergonomics-presets-and-queryable-catalog.md); the
load-bearing engineering choices worth keeping out of the commit message:

- **The audit was verified programmatically, not eyeballed.** Diffing the `Config`
  struct fields against the `Overrides` struct fields (Python over `src/config.rs`)
  gave 86 `Config` fields, 66 in `Overrides`, plus `seed` special-cased
  (`--seed`, stamped directly in `main.rs`, not via `Overrides`) ⇒ 67 CLI-reachable
  and **exactly 19 genuinely config-file-only**. An earlier sub-agent summary
  miscounted (it conflated `validate()`/other `self.x =` writes with overrides and
  reported inconsistent 45/13 vs 61/21 figures), which is why the diff was redone
  field-by-field — the number gets quoted in the ADR, so it had to be exact.

- **Why promote only 16 of 19.** Kept config-only: `library_prob` (internal
  hierarchy-reuse tuning), `use_async_reset` (niche structural emit toggle),
  `max_nodes_per_module` (a guard-rail that pairs with the RAM governor, not a
  generation *feature*). Promoting them is additive future work — none retired,
  all three stay MCP-settable via the full-`Config` `config` arg.

- **Promoted flags MUST be `Option<T>`, never clap-defaulted.** This is the
  subtle correctness point for presets: the resolver order is
  `default → --config → --profile → explicit knobs → --seed`, and "explicit beats
  preset" only holds because the `Overrides` carry `Option`s that are `None` when
  the user did not pass the flag. A concrete clap default (e.g. `function_emit_prob`
  defaulting to `0.0`) would silently clobber `--profile structured-emission-max`
  for every un-passed knob. So the existing Option-based `Overrides` discipline is
  mandatory for the new flags too.

- **Presets are declarative data, not `fn(&mut Config)` closures.** A closure's
  overrides cannot be enumerated, so the registry would not be API-queryable
  (decision `0017`). Each preset carries an enumerable `(field, value)` set so
  `anvil://catalog/presets` can show *what* a preset changes, not just its name.

- **The rich knob catalog is SCHEMA-DERIVED + completeness-gated.** Names/defaults
  derive from `Config::default()` serde; group/validation-range/cli-flag come from a
  metadata table keyed by field name, with a test asserting one entry per `Config`
  field (and no orphans). Validation ranges live in `validate()` as imperative
  checks (not data), so a hand table is honest — the completeness test is what keeps
  it from drifting as knobs are added (the KM derive-and-diff pattern).

- **Existing `anvil://catalog/knobs` raw-default resource is kept; the rich catalog
  is a new surface.** Upgrading the existing resource in place could break agents
  parsing the raw-default form — so the rich catalog and `anvil://catalog/presets`
  are additive (no retirement). `.2` pre-split into `.2a` (carrier/resolver/test
  contract design-detail) + `.2b` (impl + proofs + book/USER_GUIDE/README/KM).

---

## 2026-06-21 — Live `slang` facts in the matrix report; why no coverage fact — `DOWNSTREAM-ADAPTER-EXPANSION.2c.2b`

`.2c.2b` surfaces the `extract_facts` projection in the `tool_matrix` report
(`ModuleReport`/`DesignReport.slang_facts: Option<AdapterFacts>`, off the wire when `None`).
Two decisions worth keeping:

- **The slang column builds its `AdapterRunCx` explicitly, not via the `run_column` closure.**
  The other columns only need the `ToolInvocation` `run_column` returns; slang additionally
  needs the *same* cx to call `extract_facts(&cx, &inv)` afterward (the hook reconstructs the
  `<stem>.slang.json` path from `cx.out_dir` + `cx.target.stem()`). Reusing one cx for both
  `run` and `extract_facts` keeps the side-file path consistent and avoids re-deriving it.

- **Rejected: an opportunistic `saw_slang_facts` `CoverageSummary` fact.** It was in the
  `.2c.2b` plan, implemented, then removed. Every other `saw_*` fact is a plain always-
  serialized bool, so adding one changes *every* `tool_matrix` report's `CoverageSummary`
  JSON — verified directly: a no-op `--slang` smoke (slang absent) showed `saw_slang_facts`
  appearing 18× (once per scenario + global). That is a report-shape change for **all** runs,
  not just `--slang` runs — the opposite of the off-the-wire `slang_facts` field. The direct
  precedent, the `sv2v` column (`.2b.2`), added **no** coverage fact, and the decision-`0020`
  requirement ("surface the facts in the report") is fully met by the `slang_facts` field. So
  the coverage fact was dropped for byte-identical cleanliness + precedent consistency; its
  proof became `slang_facts_serialize_only_when_present` (the serde-skip guarantee — a more
  direct proof of the byte-identical claim than a coverage-fact assertion). Lesson:
  `skip_serializing_if` Option fields are byte-identical-safe to add; always-serialized
  scalars are not — prefer the former for opt-in report data.

---

## 2026-06-21 — The `tool_matrix --slang` column — a faithful `--sv2v` mirror — `DOWNSTREAM-ADAPTER-EXPANSION.2c.2a`

`.2c.2a` adds the `tool_matrix --slang` elaboration-acceptance column by mirroring the
`--sv2v` touchpoints (`.2b.2`) one-for-one: CLI flag/bin, `ModuleReport`/`DesignReport`
field, both checkpoints + the `--resume` guard, the `ToolSummary` tally + `any_failed` +
console line, `MatrixReport.slang_enabled`, and `unit_divergence` inclusion, all gated on a
`tool_version` presence probe (the decision-`0020` friendly no-op). No new rationale beyond
the `.2b.2` column and the `.2c.1` adapter — two mechanical notes only:

- **`ModuleToolColumns` grew 4-tuple → 5-tuple.** `run_module_tools` already returned a
  named tuple alias (introduced at `.2b.2` to satisfy clippy `type_complexity`); slang is
  the 5th element. The design-level path returns a full `DesignReport`, so it just gains a
  field. Keeping the alias means the column count can grow without re-tripping
  `type_complexity`.
- **slang's column is *not* skipped for `union soft` up-opt modules** the way `sv2v` is.
  `sv2v` targets Verilog-2005 and rejects the SV-2023 `union soft` syntax, so it is skipped
  alongside Yosys/Icarus; `slang` is a full SV-2023-aware elaborator, so it *could* accept
  it — but the column is still gated `!verilator_only` to stay aligned with the other added
  adapters in the up-opt scenario (which runs Verilator-only by design). The live
  `extract_facts` fact-surfacing is deliberately deferred to `.2c.2b`.

---

## 2026-06-21 — The trait's first `extract_facts` hook + the `slang` adapter — verifying a JSON-AST schema for an absent tool — `DOWNSTREAM-ADAPTER-EXPANSION.2c.1`

`.2c.1` lands the `slang` adapter and, with it, the `Adapter` trait's **first**
`extract_facts` fact hook (deferred from the `.2a.1` registry). The notes worth keeping:

- **The hook is defaulted to `None`, so it is zero-cost for every existing adapter.**
  `extract_facts(&self, &AdapterRunCx, &ToolInvocation) -> Option<AdapterFacts>` has a
  default body returning `None`; the three built-ins + `sv2v` don't override it ⇒ their
  reports stay byte-identical and the trait stays object-safe (`&dyn Adapter`). Only
  `SlangAdapter` overrides it (and `supports_facts() => true`). This is the same
  "capability-defaulted-off, one adapter opts in" shape as `supports_facts` (`.2a.2`).

- **The fact source is a side *file*, not stdout.** slang writes its AST to the path given
  to `--ast-json <file>` (the runner captures only stdout/stderr). So `run_slang` writes
  `<stem>.slang.json` into `cx.out_dir`, and `extract_facts` reads it back from the same
  path — which is why `AdapterTarget` grew a `stem()` accessor (the hook gets `cx`, not the
  argv). A missing/unparseable file ⇒ `None`, never an error: that *is* the slang-absent
  friendly no-op on this host (a hard reject can also produce no AST).

- **Verifying a schema for a tool you can't run (no corners).** `slang` is absent on every
  current dev host (the `.1` toolchain probe), so the parser was written against slang's
  **published** `--ast-json` schema (sv-lang.com user manual + command-line reference),
  not a guess: root `{ "design": { "kind":"Root", "members":[…] } }`; a top `Instance`
  with a `body` (`InstanceBody`); `Port` nodes `{name, direction:"In"/"Out", type:"logic[3:0]"}`;
  child `Instance` nodes whose `body.definition` is a `"<addr> <name>"` pair (the name token
  is the module name). The portable proof runs the **pure** `parse_slang_ast_facts` against
  a faithful synthetic fixture of that schema; the `#[ignore]` real-tool gate (`.2c.2`)
  upgrades it to a banked proof once slang is installed. This is the decision-`0020`
  absent-tool cadence (structural + `#[ignore]` gate), applied to the *fact* path.

- **`AdapterFacts` is SCHEMA-DERIVED, not an oracle.** It projects *slang's own*
  elaboration output (top/ports/instances) verbatim — directions and types are kept as
  slang spells them (`"In"`, `"logic[3:0]"`), not renormalized — so it never becomes an
  ANVIL behavioural oracle (the decision-`0004` ceiling). The port `type` serde key is
  slang's own (`ty` field, `#[serde(rename = "type")]`), so the projection reads back in
  the tool's vocabulary.

- **Why no `tool_matrix` column / live report surfacing here.** `.2c.1` is the additive
  half (mirroring `.2b.1`): selectable + discoverable + the parser + the hook, all
  byte-identical. Attaching the extracted facts to a live report and the `--slang` matrix
  column are byte-identical-sensitive, so they are `.2c.2` — keeping each sub-slice
  independently committable and provably byte-identical.

---

## 2026-06-18 — The `tool_matrix --sv2v` column — presence-gated friendly no-op, and where it differs from `--iverilog-compile` — `DOWNSTREAM-ADAPTER-EXPANSION.2b.2`

`.2b.2` adds the `tool_matrix --sv2v` column by mirroring the 19 `--iverilog-compile`
touchpoints (CLI flag/bin, `ModuleReport`/`DesignReport` field, checkpoint + `--resume`
guard, `ToolSummary` tally + `any_failed` + console line, `MatrixReport.sv2v_enabled`,
`unit_divergence` inclusion). Three deliberate *departures* from a pure mirror, all forced
by `sv2v` being **absent on most hosts** (unlike iverilog, which `--iverilog-compile`
assumes present):

- **The column is gated on a `tool_version` presence probe.** `--iverilog-compile` runs
  unconditionally and a spawn failure (absent tool) would count as a failure and bail via
  `any_failed`. For `sv2v` that would make `--sv2v` *fail* on a host without `sv2v` —
  violating decision `0020`'s "a missing binary is a friendly no-op, never a hard failure".
  So `run_module_tools`/`run_design_tools` only run the `sv2v` column when
  `downstream::tool_version(&cli.sv2v_bin).is_some()`; absent ⇒ the column is `None` (no
  row, not counted, no bail). This is the `diff_sim::tools_present()` precedent. The cost is
  one `sv2v --version` probe per artifact when `--sv2v` is set — instant (`ENOENT`) when
  absent, and only on the deliberate opt-in path when present; acceptable for a first cut.
  `sv2v` **is** in `any_failed` (a real reject bails, like iverilog) — the presence gate is
  what keeps "absent" out of that count.
- **`run_module_tools` now returns a 4-tuple → a named alias.** The added `sv2v` column
  pushed the return type past clippy's `type_complexity` threshold, so the tuple is factored
  into `type ModuleToolColumns = (…)`. (The clippy-recommended "factor into a type" fix; a
  struct was overkill for four positional columns the one caller immediately destructures.)
- **`unit_divergence` reached 8 args → a documented `#[allow(too_many_arguments)]`.** It
  takes one parameter per acceptance column on top of cli + location, *because* it checks the
  off/out-of-subset cases before assembling, so the default path clones nothing. Bundling the
  columns into a struct would either lose that early-return or add a lifetime-bearing struct
  for no real gain; the suppression is the honest, behaviour-preserving choice for a collector.

Verified the no-op end-to-end with a real smoke: `tool_matrix --skip-verilator --skip-yosys
--sv2v` over 17 modules exits 0 with `sv2v pass/fail = 0/0` and zero `sv2v` invocations in
the report — `--sv2v` requested, `sv2v` absent, run clean.

## 2026-06-18 — The `sv2v` adapter — argv + warning rule chosen against an absent tool — `DOWNSTREAM-ADAPTER-EXPANSION.2b.1`

`.2b.1` lands the first new downstream adapter, `sv2v`. `sv2v` is **absent** on this
host (and most), so the argv + warning rule are chosen from sv2v's documented CLI, not
verified against a live binary — the decision-`0020` "structural + `#[ignore]` real-tool
gate, upgraded to a banked proof once installed" cadence. Three choices worth recording:

- **argv shape.** Module = `sv2v <file>` (sv2v reads the one emitted leaf module and
  transpiles it to stdout; exit code is the accept/reject signal). Design =
  `sv2v --top=<top> <files…>` — `--top=<top>` pins the elaboration root, mirroring the
  `-top` / `--top-module` / `-s <top>` pins the other design adapters use. The transpiled
  Verilog goes to sv2v's stdout and is **discarded** (`run_tool` keeps it only in a log on
  failure): this is an acceptance gate, **not** a behavioural oracle (decision `0004`) —
  ANVIL never treats the transpiled output as golden semantics (that is the orthogonal
  `--diff-sim` axis's job).
- **warning rule.** sv2v's exact warning prefix is unverified locally, so `first_tool_warning`
  matches `warning:` case-insensitively across stdout+stderr (the iverilog rule). This is
  safe against false positives: the only sv2v stdout is transpiled Verilog over ANVIL's
  generated identifiers (`add_0`, `mux_0`, …), which never contain the `warning:` token.
  If a real sv2v emits a different prefix, the `#[ignore]` gate (`.2b.2`) surfaces it.
- **accept/reject only — no fact hook.** `sv2v` is the *minimal* adapter shape on purpose
  (`supports_facts == false`): it exercises the whole trait end-to-end without the richer
  `extract_facts` JSON-AST path, which lands with `slang` (`.2c`). Keeping the first new
  adapter minimal is the decision-`0019` "reject/warning first, fold the richer axis in
  later" cadence.

`.2b.1` deliberately stops at the downstream adapter + the MCP `tools`/catalog surface
(selectable + discoverable immediately, additive / byte-identical). The byte-identical-
sensitive `tool_matrix` column (a new `--sv2v` flag + `ModuleReport`/`DesignReport.sv2v`
field + checkpoint/resume guard + tally) and the real-tool gate are `.2b.2`, mirroring the
`.2a` split that isolated byte-identical-sensitive work.

## 2026-06-18 — Routing the matrix columns through the adapter registry — fixed columns, per-column dispatch — `DOWNSTREAM-ADAPTER-EXPANSION.2a.3`

`.2a.3` routed the last two downstream callers (`validate_tool_specs`/`run_tool_spec`
and the `tool_matrix` `run_module_tools`/`run_design_tools` columns) through the closed
`Adapter` registry. Two choices worth recording for the `.2b` (`sv2v`) follow-up:

- **The matrix keeps *fixed, named* columns — it does NOT iterate the registry to build a
  dynamic column list.** `ModuleReport`/`DesignReport` have hard-coded `verilator: Option<…>`,
  `yosys: Vec<…>`, `iverilog_compile: Option<…>` fields, and those exact field names +
  the `"verilator"`/`"yosys-<mode>"`/`"iverilog-compile"` labels are a byte-identical
  constraint for banked reports + `--resume` (decision `0020`). So "route through the
  registry" means *per-column dispatch* — each column looks up its adapter via
  `AcceptanceTool::{Verilator,Yosys,Iverilog}.adapter()` and calls `.run(&cx)` — not a loop
  over `adapters()`. Adding `sv2v` (`.2b`) is therefore a registry entry **plus** a new
  named column field **plus** one `run_column` line; the registry makes the *invocation*
  one line, but the report still grows a typed column (that is intentional — typed columns
  are what keep the serialized shape a stable wire contract).
- **The Yosys version-axis `Both`→single collapse must happen BEFORE the adapter, not
  after.** `YosysAdapter::run` faithfully emits *two* `ToolInvocation`s for
  `YosysMode::Both` (that is correct for the matrix, where both modes are columns). But the
  tool-version-vs-version axis (`run_tool_spec`) wants exactly *one* invocation per spec —
  comparing the two Yosys modes is the cross-tool axis, not the version axis. So
  `run_tool_spec` collapses `Both`→`WithoutAbc` into `cx.yosys_mode` *before* calling the
  adapter, then takes `.into_iter().next()`. The new proof
  `validate_tool_specs_routes_each_kind_through_its_adapter_single_row` pins this (one row
  per spec under `Both`), because it is the one place the registry routing could silently
  change row counts.

## 2026-06-17 — Adapter catalog — a live `present` probe inside an otherwise-static catalog read — `DOWNSTREAM-ADAPTER-EXPANSION.2a.2`

The `anvil://catalog/adapters` resource projects the registry as `{id, binary,
present, supports_facts}`. Two choices worth recording:

- **`present` is a *live* probe, so this catalog read is mildly side-effecting —
  deliberately.** `anvil://catalog/knobs` and `anvil://catalog/lanes` are pure/static
  (they serialize a default `Config` / a constant). `present` cannot be — "is this
  tool installed *right now*" is a property of the host, so `adapter_catalog()` runs
  a best-effort `<binary> --version` (`downstream::tool_version`) per adapter at read
  time. That is the *point* of the decision-`0017` discoverability surface ("which
  tools exist AND which are installed"), and it matches the existing `tools_present()`
  precedent (which already spawns `--version`). It stays SCHEMA-DERIVED in spirit: it
  projects the registry + the live environment, not a recomputed or stored truth. The
  cost is three short-lived `--version` spawns per catalog read — acceptable for an
  explicit agent discovery call, and the probe is best-effort (a missing tool ⇒
  `present: false`, never an error).

- **`supports_facts` is a defaulted trait flag added *before* the fact hook exists.**
  The actual `extract_facts` hook lands at `.2c` (slang). But the catalog's schema
  should be stable from the start, so `Adapter::supports_facts() -> bool` ships now
  with a `false` default (all three built-ins inherit it); `slang` overrides it to
  `true` when it lands the hook. A defaulted trait method consumed by the catalog is
  not dead code, and it keeps the catalog's wire shape from changing when `.2c` adds
  the first fact-bearing adapter.

## 2026-06-17 — Downstream adapter registry — the byte-identical "delegate verbatim" trick + the `Sync` static — `DOWNSTREAM-ADAPTER-EXPANSION.2a.1`

The first impl leaf of decision `0020`. Two implementation choices worth recording:

- **How the refactor is provably byte-identical: each built-in adapter's `run`
  *delegates verbatim* to the existing `run_*` primitive.** The temptation in a
  "generalize the three tools behind a trait" refactor is to *re-implement* the argv
  construction inside each adapter. That would be a second source of truth for the
  command lines (the thing `src/downstream` exists to prevent) and a byte-identical
  risk. Instead, `VerilatorAdapter::run` literally calls `run_verilator(...)`,
  `YosysAdapter::run` calls `run_yosys(...)` (already returns a `Vec`), etc. — so the
  argv, the warning detection, the log capture, and the `ToolInvocation` shape are
  *the same code*, and byte-identical is guaranteed by construction, not by testing.
  The trait's `run` returns `Vec<ToolInvocation>` precisely because Yosys already
  does (1–2 rows per `YosysMode`); Verilator/Icarus wrap their single result in a
  one-element `Vec`. `validate`'s loop becomes `tools.extend(adapter.run(&cx)?)`,
  which reproduces the old match arms exactly (push-one vs extend-many) with the
  mem-guard still checked once per selected tool.

- **`AdapterTarget` is `Copy`; the registry is a `static` needing `Adapter: Sync`.**
  `AdapterTarget` holds only references (`&Path`, `&str`, `&[PathBuf]`), so it derives
  `Copy` and is built once before the `validate` loop and copied into each
  `AdapterRunCx` — no per-iteration re-derivation, and NLL lets `sb.top` move into the
  `ValidateReport` after the loop's last use of the borrow. The registry tripped one
  compiler subtlety: `fn adapters() -> &'static [&'static dyn Adapter]` returning
  `&[&A, &B, &C]` fails (`E0515`: the array is a temporary). The fix is a named
  `static ADAPTER_REGISTRY: [&dyn Adapter; 3]` returned by reference — which requires
  the array to be `Sync` (statics must be), i.e. `&dyn Adapter: Sync`, which holds
  *because* `trait Adapter: Sync` (a supertrait `Sync` makes the trait object `Sync`).
  That is the whole reason the trait carries the `Sync` supertrait.

- **Why `.2a` was split (`.2a.1`/`.2a.2`/`.2a.3`).** The original `.2a` bundled the
  registry refactor + the catalog query + routing the `tool_matrix` *fixed columns*
  through the registry. The last part is the byte-identical-sensitive one (the
  `ModuleReport`/`DesignReport` `verilator`/`yosys`/`iverilog_compile` fields are a
  serde wire contract for banked reports + `--resume`). Splitting keeps `.2a.1` a
  single-file, provably-byte-identical foundation (touching only `src/downstream`),
  with the catalog (`.2a.2`) and the riskier orchestrator routing (`.2a.3`) as their
  own reviewable slices — the repo's standard "split a big impl leaf" discipline.

## 2026-06-17 — Downstream adapter interface — a closed registry, not a plugin system — `DOWNSTREAM-ADAPTER-EXPANSION.1` (decision `0020`)

The design ADR for making ANVIL's downstream reach pluggable. Rationales worth
recording (the full argument is decision `0020`):

- **"Pluggable" deliberately means a *closed, compile-time* registry, NOT a
  runtime plugin.** The tempting reading of "let new tools plug in" is a dynamic /
  loadable / agent-supplied-command adapter. That is rejected outright: it destroys
  the decision-`0004` fixed-allow-list (the agent picks *which vetted kind* runs,
  never *which binary* — `AcceptanceTool::binary()`) and breaks reproducibility /
  auditability. The registry keeps the runnable set fixed at compile time and
  reviewed; it only collapses "add a tool" from a sprawling cross-cutting edit
  (a new `AcceptanceTool` variant + a `run_*` pair + a `first_tool_warning` arm + a
  `match` arm in `validate` *and* `validate_tool_specs` *and* the `tool_matrix`
  columns) into **one self-contained descriptor**. This is the same boundary
  `0019.2f` drew for the version axis — adapters are emphatically not a back door to
  caller-supplied binaries.

- **The trait generalizes only what genuinely differs per tool: argv + the warning
  predicate + an optional `extract_facts` hook.** Everything else is already shared
  and must NOT be re-implemented per tool — `run_tool` is the one sandboxed runner,
  `tool_verdict` is the one accept/warn/reject classifier (extracted at `0019.2a`),
  `prepare_dut_sandbox`/`MemGuard` are the one sandbox/decline lifecycle. One runner,
  one classifier (`feedback_full_factorization`). The payoff is structural: because
  the verdict is unchanged, every new adapter becomes a new comparable verdict in
  `divergence::run` and a new selectable tool in `hunt`/`validate` **for free** —
  the adapter expansion multiplies the bug-surface across all three detector
  surfaces without touching any of them. That compounding is the lane's reason to
  exist.

- **Byte-identical built-ins are a hard constraint, not an aspiration.** Banked
  `tool_matrix` reports and `--resume` checkpoints key off the literal
  `ToolInvocation.tool` labels (`"verilator"`, the `yosys-<mode>` rows,
  `"iverilog-compile"`) and the exact argv. So re-expressing the three built-ins as
  the first registry entries must reproduce those byte-for-byte; new adapters add
  *new* rows/columns only, opt-in. `AcceptanceTool` is not retired
  (`feedback_never_retire_strategies`) — it stays the canonical built-in identity.

- **`sv2v` first, `slang` second — minimal-surface-first.** Live-toolchain probe
  this session: `slang`/`sv2v`/`surelog`/`svlint`/`verible`/`moore` all **absent**
  (only verilator 5.046 / yosys 0.64 / iverilog 13.0 present). With nothing
  installed, the choice is on shape: `sv2v` is the *minimal* adapter (a pure
  accept/reject transpile column — no fact hook), so it proves the whole trait
  end-to-end in the smallest leaf; `slang` is the *richer* one (strict elaborator +
  `--ast-json`), so it lands the optional `extract_facts` hook (the
  `tests/frontend_parity.rs` Verilator-JSON-AST precedent). Both absent ⇒ ship
  structural + a friendly `tools_present()`-style no-op + an `#[ignore]` tool-gated
  real-tool gate (the `sv_version_downstream` / `hunt_e2e` / `divergence_e2e`
  precedent), upgraded to a banked proof once `brew install sv2v` / `slang` is run.
  This is the `0019` "reject/warning first, fold the richer axis in later" cadence.

- **API-completeness (`0017`) without a new MCP tool.** Adapters ride the existing
  surfaces: selectable through the existing `tools: [...]` arg (the same
  allow-listed `from_name` path — unknown ⇒ a clean `-32602`, never a spawn) and
  queryable through the existing `ValidateReport`/`DivergenceReport`/matrix columns.
  The one genuinely-new API piece is an **adapter-catalog** projection
  (`{ id, binary, present, supports_facts }`) so an agent can *discover which tools
  exist and which are installed* over the API alone — the decision-`0017` "drive it
  with only the MCP API" test. An adapter's richer facts are a SCHEMA-DERIVED
  projection of *its own tool's* output (e.g. slang's AST), never an ANVIL
  behavioural oracle — the `0004` ceiling holds.

## 2026-06-17 — Acceptance-divergence hunting — the version-axis trust boundary + the e2e gate shape — `ACCEPTANCE-DIVERGENCE-HUNTING.2f`

The closing leaf. Two durable rationales worth recording:

- **Why the tool-version-vs-version axis stays library-only (NOT exposed over
  MCP/CLI).** The whole controlled-tool security model (decision `0004`) is: the
  agent chooses *which allow-listed kind* runs (`verilator`/`yosys`/`iverilog`),
  but **never which binary** — the binary is pinned by `AcceptanceTool::binary()`.
  The `.2e` version axis (`DivergenceOptions.tool_specs` →
  `downstream::validate_tool_specs`) deliberately pairs an allow-listed *kind* with
  a **caller-supplied binary path** so two versions can be compared. That is
  exactly right for an *in-process* caller who already controls the host, but
  exposing it over the agent interface would let a caller point the `verilator`
  kind at *any* executable — a strictly larger trust surface than every other
  controlled tool. So `.2f` decided: keep it a **library surface**, do not wire it
  into the MCP `divergence` tool or the CLI. Safe exposure would need its own
  design — an **operator-configured** version-binary registry the server consults
  (the operator, not the agent, enumerates the allowed binaries/labels), never an
  agent-supplied path. Recorded as future breadth in decision `0019`; nothing
  retired (the library surface + its cargo-portable proofs stay). This is the
  conservative reading of decision `0017` (API-first): the *capability* (acceptance
  divergence) is fully MCP-invocable + queryable + CLI-shimmed; only the *unsafe
  parameterization* (an arbitrary binary path) is gated behind a future
  trust-boundary decision.

- **Why the e2e gate has no "manufactured ANVIL failure" (the `hunt_e2e`
  precedent).** ANVIL output is valid by construction, so there is no honest way to
  make a real downstream tool *reject* it — a genuine rejection would be the real
  downstream-tool bug the lane exists to *surface*, not a fixture (fabricating one
  means emitting illegal RTL, which the project forbids). So `tests/divergence_e2e.rs`
  proves the two things that *are* provable end-to-end: (1) the **steady state**
  (an all-agree real-tool run records the full per-tool verdict matrix with
  `diverged=false`, and the report is queryable), and (2) the **classifier** (a
  synthetic accept/reject `ValidateReport`, injected through the **public** API,
  classifies `accept_reject`). The synthetic injection is portable (no tool) and
  runs on every `cargo test`; the all-agree run is `#[ignore]` + tool-gated. Same
  honesty shape as `BUG-HUNT-ORCHESTRATION.2e`.

## 2026-06-17 — Acceptance-divergence hunting — the tool-version-vs-version axis — `ACCEPTANCE-DIVERGENCE-HUNTING.2e`

The version axis (one tool **kind**, two **versions/binaries**). Design choices
worth recording:

- **The version axis reuses the run_* primitives, NOT a forked invocation set.**
  The `run_verilator`/`run_yosys`/`run_iverilog_compile` primitives already take a
  caller-supplied `bin: &str` — the fixed-binary allow-list is enforced one layer
  up, in `ValidateOptions` via `AcceptanceTool::binary()`. So the version axis runs
  the *same* vetted command lines (the part that must never drift —
  `feedback_full_factorization`) with a caller-supplied binary for an allow-listed
  *kind*. `ToolSpec` carries `(kind: AcceptanceTool, binary, label)`: the kind keeps
  the argv + warning-detection allow-list honest; the binary is the version shim;
  the label distinguishes the two versions' rows. ANVIL never manages installs.
- **Extracted `prepare_dut_sandbox` so `validate` and the version axis share one
  sandbox lifecycle.** The divergence module's docstring disclaims a "second sandbox
  loop". The version axis genuinely cannot go through `validate` (which runs each
  `AcceptanceTool` once with its *fixed* binary — injecting caller-supplied binaries
  there would weaken the contract `minimize` + the MCP `validate` tool depend on).
  So the generate→mkdir→write lifecycle was factored into
  `prepare_dut_sandbox(seed, cfg, root, prefix)` + `DutSandbox`, and BOTH `validate`
  (`prefix="anvil-validate"`) and `validate_tool_specs` (`prefix="anvil-divergence"`)
  call it. `validate` is byte-identical after the refactor (same `run_id`, same dir
  name, same `ValidateReport`) — proven by snapshots 6/6 + the real-tool
  `anvil hunt --divergence` regression (`n_clean=4 n_failures=0`).
- **`classify_version_mismatch` is a distinct RELATION, not a second classifier.**
  The accept/warn/reject verdict still comes only from the one
  `downstream::tool_verdict`. `classify_divergences` (cross-tool) asks "do *different
  tools* disagree on acceptance?"; `classify_version_mismatch` (version axis) asks
  "do *versions of one kind* disagree?". Same verdict, different grouping — so it is
  not a forked classifier. Factored `assemble_report(report, classify)` so both
  report-builders share one projection and differ only by the classifier passed.
- **One invocation per spec; Yosys uses a single mode for the axis.** The version
  axis compares N specs as N verdicts, so each spec must contribute exactly one
  invocation. Yosys `Both` would yield two per binary (without-abc + with-abc),
  conflating an intra-binary mode difference with a cross-version difference — so the
  axis collapses `Both`→`WithoutAbc` (the stable baseline). Comparing the two Yosys
  modes is already the *cross-tool* axis (`--yosys-mode both` on `validate`); it is
  not the version axis's job.
- **`ToolInvocation.version` is `#[serde(default, skip_serializing_if)]` and
  captured only on the axis.** Adding a field to `ToolInvocation` (a stable wire
  contract embedded in banked `tool_matrix` reports + `--resume` checkpoints) is
  safe only if it is absent when unset: skip-serializing-`None` keeps every existing
  report byte-identical and `#[serde(default)]` lets pre-`.2e` checkpoints
  deserialize. Capturing `--version` only on the opt-in axis (never in `run_tool`)
  means the default paths spawn no extra process — byte-identical behaviour, not
  just byte-identical output.
- **The axis is library-only at `.2e` (rejected: wiring it through MCP/CLI now).**
  Decision `0017` (API-first) wants every capability MCP-invocable — but exposing a
  *caller-supplied binary path* for an allow-listed kind is a materially different
  trust surface from the fixed-binary `validate`/`divergence` tools (an agent could
  name any path on disk). The `.2e` acceptance scopes to the library core +
  classifier + cargo proofs; the MCP/CLI exposure (with whatever allow-list/guard it
  needs) is deferred to `.2f`'s closeout decision. Recorded so it is not lost.

## 2026-06-17 — Acceptance-divergence hunting — the MCP tool + CLI shim — `ACCEPTANCE-DIVERGENCE-HUNTING.2d`

The third (and last non-version) surface. Design choices worth recording:

- **The MCP `divergence` tool is single-`(seed, config)` (mirrors `validate`),
  NOT a sweep (like `hunt`).** The ADR sketched a `seeds` arg, but `divergence::run`
  is single-seed and the `.2d` acceptance says "shimming `divergence::run`" /
  "`DivergenceReport` returned" (singular). So the standalone tool runs one artifact
  and returns one report; the multi-seed sweep is the `hunt` tool's `divergence`
  axis. This avoids duplicating the sweep loop and keeps a clean split: `divergence`
  = "did these tools disagree on *this* artifact?", `hunt --divergence` = "sweep N
  seeds for a disagreement". Documented in the tool's schema comment.
- **Cache the run_id only when it diverged.** `validate`/`hunt` cache findings;
  the all-agree steady state is not a finding, so `run_divergence` caches the
  artifact (`introspect_dut_artifact(seed, cfg)` → the content address) only when
  `report.diverged`, mirroring `cache_hunt_failures`'s find-then-cache shape. The
  agent then reads `anvil://artifact/<run_id>/{sv,introspection}` for the divergent
  reproducer — the MCP-native mechanism (no filesystem path from the agent,
  decision `0004`).
- **Two `.2c.1` placeholders flipped, not rewritten.** `.2c.1` deliberately wired
  `divergence: false` in both `mcp::run_hunt` and `main::build_hunt_request` with a
  "`.2d` will wire the arg" comment. `.2d` flips them to
  `parse_bool_arg(args, "divergence", false)?` and `args.divergence` respectively —
  default-off, so any caller that omits the arg/flag stays byte-identical. This is
  the decision-`0017` completeness step: the action was reachable internally; now
  it is MCP-invocable (`divergence` tool + `hunt` axis) and CLI-invocable
  (`anvil hunt --divergence`), with the CLI a thin shim.
- **No introspection schema bump.** The tree allowed a bump only "if a
  `DivergenceReport` projection is served as a resource". It is not — the divergent
  `run_id` reuses the existing `sv`/`introspection` artifact resources — so the
  schema stays `1.11`.

## 2026-06-17 — Acceptance-divergence hunting — the `tool_matrix` column — `ACCEPTANCE-DIVERGENCE-HUNTING.2c.2`

The second of the one-detector-two-surfaces pair. Two design choices worth recording:

- **The matrix column is a pure projection, so it has NO tool-clean precondition —
  the opposite of `--diff-sim`.** `--diff-sim` spawns two simulators *after*
  Verilator + Yosys both accept the SV (no point asking simulators to agree on
  output a parser already rejected), so its per-unit gate is
  `cli.diff_sim && verilator_ok && all_yosys_ok && in_subset`. The divergence column
  spawns **nothing** — it classifies the `ToolInvocation`s the matrix *already ran* —
  so requiring the tools to be clean first would be exactly backwards: a divergence
  is most interesting precisely when one tool rejects what another accepts. Hence
  `unit_divergence`'s gate is only `cli.divergence && in_subset` (then "≥1 tool ran").
  This asymmetry is deliberate and is the reason the two columns do not share a gate
  helper even though they share the subset selector.
- **One classifier, reached two ways.** Rather than reimplement verdict
  classification in the binary, `unit_divergence` assembles the already-run
  invocations into a `ValidateReport` and calls the *same*
  `divergence::classify_report` the hunt loop uses (`feedback_full_factorization` —
  no second classifier). The `ValidateReport.run_id`/`sandbox` are the unit's name /
  scenario dir (the matrix retains the actual `.sv` on disk per the reproducer
  policy, so it does not content-address here); `ValidateReport.ok` is computed but
  unread by `classify_report` (it only reads `tools` + the metadata it carries
  through). The shared subset-membership check was likewise factored into one
  `scenario_in_named_subset(scenario_dir, sentinel)` used by both columns.
- **`saw_acceptance_divergence` is opportunistic and `compute_coverage_gaps` is
  untouched.** A coverage *gate* requiring a divergence would fail on
  valid-by-construction RTL, whose steady state is all-agree (decision `0019`'s
  honesty boundary). The fact is recorded when seen and merged like any other
  `saw_*`, but it is never added to the gap list — the real-tool smoke confirms the
  steady state (`diverged=false`, fact `false`) on a clean 17/0 sweep.

## 2026-06-17 — Acceptance-divergence hunting — the `hunt::run` fold — `ACCEPTANCE-DIVERGENCE-HUNTING.2c.1`

Folding the detector into the hunt loop. The non-obvious calls:

- **Acceptance divergence lives on the *finding* path, not the clean path.** The
  ADR sketched "run the detector on each swept artifact", but think it through:
  the hunt's `report.ok == true` means *every* selected tool accepted — so there
  is no disagreement, and a divergence is impossible. A divergence requires some
  tool to accept while another rejects/warns — which is exactly `report.ok ==
  false` (the finding path) *with* a mix of verdicts. So the fold belongs where
  the finding is built, and it *refines* a reject/warning finding into an
  `acceptance_divergence` when the disagreement is cross-tool. (Contrast the
  `diff_sim` axis, which is the opposite — it needs all tools to *accept* first,
  so it lives on the clean path.)
- **Classify the tools `validate` already ran — don't re-validate.** On the
  finding path the loop already holds `report: ValidateReport` with every tool's
  invocation. Calling `divergence::run` there would re-generate + re-run every
  tool. So `.2c.1` extracts `divergence::classify_report(&ValidateReport)` (the
  pure projection half of `run`; `run` now = `validate` + `classify_report`,
  byte-identical) and the hunt calls *that*. One orchestration, run once.
- **A divergence is not minimized — same reason as `cross_sim_mismatch`.** The
  `validate` minimize oracle only knows "some tool fails"; shrinking can land on a
  config where the *accepting* tool also starts rejecting, destroying the
  divergence. So reporting a minimized `acceptance_divergence` would be misleading;
  `minimize` is skipped (`failure.divergence.is_none()` guards the minimize block).
- **Adding a required field to `HuntRequest` rippled to four constructors.** The
  field is required (not `#[serde(default)]` on the struct — `HuntRequest` isn't
  deserialized), so every literal had to set it: the hunt tests, the MCP
  `run_hunt`, and the CLI `build_hunt_request`. The two surface constructors set
  `divergence: false` with a comment that the actual arg-wiring is `.2d` — so this
  leaf is byte-identical for the MCP tool and the CLI (no schema/flag change yet).

## 2026-06-17 — Acceptance-divergence hunting — the `src/divergence/` library core — `ACCEPTANCE-DIVERGENCE-HUNTING.2b`

The detector core. The decisions worth recording:

- **`divergence::run` reuses `downstream::validate`; it does not re-run the
  tools.** The ADR sketched "compose `generate_dut_artifact` + the `run_*`
  primitives", but `validate` is *already* that composition — and crucially it
  already runs **every** enabled tool/mode to completion (it never short-circuits
  on a reject; only `MemGuard` declines before a spawn). So the highest-factored
  realisation is to call `validate` and project `report.tools` into verdicts,
  rather than fork a second sandbox-and-run loop. This is the same
  "one orchestration, no second source of truth" move that `MinimizeOptions` made
  by wrapping `ValidateOptions` — which is exactly why `DivergenceOptions` wraps
  `ValidateOptions` too. A forked loop would have been a second place to keep the
  sandbox/allow-list/RAM-guard discipline in sync; rejected.
- **`ToolDecision`, not `ToolVerdict`, for the per-tool record.** The `.1` ADR's
  JSON sketch named the per-tool record `ToolVerdict { tool, verdict, … }`, but
  `.2a` already shipped a `ToolVerdict` *enum* (`Accept`/`Warn`/`Reject`) that
  `hunt` depends on. Renaming the enum would churn `.2a`; so the record is
  `ToolDecision { tool, verdict: ToolVerdict, exit_code, first_message }`. A
  deliberate, minor naming refinement of the ADR, recorded here so the ADR↔code
  delta is explicit (no silent drift).
- **`classify_divergences` emits one `Divergence` per present pair-class, and up
  to all three co-occur.** With verdicts drawn from {accept, warn, reject}, each
  *pair of distinct values both present* is a divergence: `accept_reject`,
  `accept_warn`, `warn_reject`. If all three values are present, all three fire —
  that's correct (each is a real, separately-actionable disagreement), and the
  output order is fixed + the tool lists sorted/deduped so the result is
  deterministic (the reproducibility contract — no hash-map iteration in an
  output path).
- **Yosys `both` makes a tool diverge with itself.** The comparison unit is the
  *labelled* tool (`ToolInvocation.tool`), and Yosys `both` yields
  `yosys-without-abc` + `yosys-with-abc` as two labels. So a without-abc-vs-with-abc
  disagreement (a real shape the repo has seen — ABC-flow warnings on valid
  designs) is a first-class divergence with no extra plumbing. Proven directly.
- **Honesty / no oracle.** `divergence::run` only *classifies* the tools' own
  verdicts; it never decides which tool is "right" (decision `0004`, gap 4). On
  valid-by-construction RTL the steady state is all-agree, so the no-tools proof
  and the all-agree proof both assert `diverged == false` — a found divergence is
  the (opportunistic) downstream-bug signal, surfaced at `.2c`+.

## 2026-06-17 — Acceptance-divergence hunting — shared accept/warn/reject classifier extract — `ACCEPTANCE-DIVERGENCE-HUNTING.2a`

The first code leaf of lane 2, a **pure byte-identical refactor** (the
`BUG-HUNT-ORCHESTRATION.2a` extract-then-reuse precedent that lifted
`diff_sim::run_agreement`). Notes:

- **One classifier, lifted to the primitive layer.** `hunt::run` already
  classified a failing `ToolInvocation` into warning-vs-reject inline. The
  divergence detector (`.2b`) needs the *same* trinary, so the logic moves to
  `downstream::tool_verdict(&ToolInvocation) -> ToolVerdict{Accept,Warn,Reject}`
  beside `ToolInvocation` (the type it projects) and `first_tool_warning` (the
  warning detector that already folded warning into `success=false`). Both
  consumers now derive from one definition — the `feedback_full_factorization`
  "no second classifier" rule, and the same reason `validate`/`run_*` live in
  `downstream` rather than being re-implemented per caller.
- **The trinary adds `Accept`; the old inline code only saw failures.**
  `hunt::classify_detection` is only ever called on `first_failing_tool`'s output
  (a `!success` invocation), so it only needed warn-vs-reject. `tool_verdict`
  generalises to the full trinary (`success ⇒ Accept`) because divergence compares
  *every* tool including the accepting ones. To keep `classify_detection`
  byte-identical I map `Warn ⇒ "warning"` and `Accept | Reject ⇒ "reject"`: the
  `Accept` arm is **unreachable** there (the input is a failing tool), folded in
  with `Reject` defensively rather than `unreachable!()` so a future caller can't
  panic it. The live arms (`Warn`/`Reject`) are exactly the old
  `exit_code == Some(0) ? "warning" : "reject"`.
- **Why `tool_verdict` keys on `success`, not on `exit_code` alone.** `success`
  already folds warning-as-failure (`first_tool_warning`), so `success ⇒ Accept`
  is the honest "clean accept". A non-zero exit that somehow reported `success`
  still classifies `Accept` (keys on the folded verdict, not the raw code) — proven
  in the unit test so the precedence is explicit.
- **Wire shape.** `ToolVerdict` is `#[serde(rename_all = "snake_case")]` ⇒
  `"accept"`/`"warn"`/`"reject"`, the stable form the `.2b` `DivergenceReport` and
  the eventual MCP/`tool_matrix` surfaces serialise. `ToolInvocation`'s shape is
  untouched ⇒ banked matrix reports + `--resume` checkpoints stay valid; snapshots
  6/6 byte-identical.

## 2026-06-17 — Acceptance-divergence hunting — design ADR — `ACCEPTANCE-DIVERGENCE-HUNTING.1`

The `.1` design leaf (decision `0019`), autonomously PNT-picked as usability lane 2
right after the `BUG-HUNT-ORCHESTRATION` tree closed — decision `0018` itself names
this lane as "the natural next detector that plugs into the just-completed hunt
engine". Docs-only; no `src/` touched ⇒ DUT byte-identical. The non-obvious calls:

- **One detector, three surfaces — not three detectors.** The tree's load-bearing
  open question was "does divergence detection *ride the hunt loop* or is it an
  *independent `tool_matrix` column*?". The answer is **both, via one shared
  `divergence::run`** — a `hunt::run` axis, a `tool_matrix` column, and a controlled
  MCP `divergence` tool all shim the same library entry. Forking the detector per
  surface would have re-created the very drift `BUG-HUNT-ORCHESTRATION` avoided by
  composing `downstream::validate` rather than re-parsing tool output. (decision
  `0017` also *requires* the MCP surface + the CLI-as-shim, so "matrix-column-only"
  was never admissible anyway.)
- **The verdict is a trinary projection, and the classifier is *extracted*, not
  rewritten.** `hunt::run` already classifies a `ToolInvocation` into reject
  (non-zero exit) vs warning (clean exit + `!success`, from `first_tool_warning`).
  Acceptance divergence needs exactly that trinary (accept/warn/reject), so `.2a`
  *extracts* the inline logic into a shared `downstream::tool_verdict` and both
  consumers derive from it — the `feedback_full_factorization` "no second classifier"
  rule, mirroring how `.2a` of the hunt tree extracted `diff_sim::run_agreement`. A
  fresh divergence-specific parser was rejected for the same drift reason.
- **Why a trinary and not `validate`'s binary `ok`.** `validate` folds warning into
  `ok = false` — right for "is this a finding?", wrong here: an *accept-vs-warn*
  lint-severity divergence would collapse into the accept-vs-reject bucket and lose
  signal. Divergence must keep the three states distinct. (And it runs **every**
  tool to completion — no fold, no short-circuit on first reject — because it needs
  every verdict, the key behavioural difference from `validate`.)
- **Labelled tools, so `--yosys-mode both` can diverge with itself.** The unit of
  comparison is a *labelled* tool (`verilator` / `yosys-without-abc` /
  `yosys-with-abc` / `iverilog`), not a tool *kind*. That makes a without-abc vs
  with-abc disagreement a first-class divergence — a real signal the repo has
  already seen (ABC-flow warnings on valid designs, README "Current CLI truth").
- **The honesty boundary (the same one `.2e` of the hunt tree hit).** On
  valid-by-construction RTL the steady state is *all tools accept* — a real
  divergence would be an actual downstream-tool bug (the thing the lane *surfaces*),
  not a fixture. So `saw_acceptance_divergence` is **opportunistic, never a required
  coverage gate** (a required-divergence gate would fail on clean output, i.e.
  always). The gates instead prove the matrix is *produced, classified, and
  queryable* — via a synthetic injected accept/reject `ToolInvocation` set in a
  cargo-portable unit test + an all-agree real-tool run recording `diverged=false`.
- **Version-vs-version is deferred to `.2e`, deliberately.** Multi-tool same-version
  divergence is portable (no extra install) and higher-leverage, so it ships first;
  version pinning needs the caller to supply two binaries (environment-specific), so
  it folds in later — the hunt's "reject/warning first, cross-sim later" cadence.
  The allow-list stays by *kind* (`AcceptanceTool::from_name`); only the *binary*
  for that kind may be caller-supplied. ANVIL never manages tool installs.
- **No new reproducer format.** A divergence finding reuses `write_bundle`
  (hunt path) or the `tool_matrix` `.sv`+log retention (matrix path); only `repro.sh`
  changes to record *each* labelled tool's `argv` so the disagreement reproduces,
  not just one side. Nothing retired (`feedback_never_retire_strategies`).

## 2026-06-17 — Bug-hunt orchestration — real-tool e2e gate + tree closeout — `BUG-HUNT-ORCHESTRATION.2e`

The tree's **final leaf**. The crux was the honest design of the e2e gate:

- **There is no manufacturable ANVIL failure — and that's the point.** ANVIL
  output is valid by construction, so no `(seed, knobs)` makes a real tool reject
  it; a genuine rejection would be an actual downstream-tool bug — the thing the
  loop exists to *surface* — not a test fixture (fabricating one would mean
  emitting illegal RTL, which the project forbids). So `tests/hunt_e2e.rs` does
  **not** pretend to find a synthetic bug. It proves the two things that *are*
  provable end-to-end with the real toolchain: (1) the whole `anvil hunt` loop
  runs **clean** against real Verilator (`n_failures == 0`, the steady state),
  and (2) the reproducer **recipe** is byte-identical-faithful (`anvil --config
  <dumped knobs>` reproduces `anvil --seed` byte-for-byte + the tool accepts the
  regenerated `.sv` — exactly `repro.sh`'s two steps). The bundle **directory
  format** is the cargo-portable job of `.2b.2b`'s `write_bundle…` unit test.
  This split keeps every proof honest and every layer covered.
- **The e2e test drives the real binary, not a library re-entry.** It uses
  `env!("CARGO_BIN_EXE_anvil")` + `std::process::Command`, so it exercises the
  actual `anvil hunt` argv path (clap parse → `run_hunt_command` → `hunt::run`),
  the same thing a user runs — the strongest end-to-end signal.
- **`serde_json` had to move into `[dev-dependencies]`.** Integration tests link
  the crate as an *external* crate and cannot name its regular `[dependencies]`,
  so parsing the `HuntReport` JSON needed `serde_json` as a dev-dep (same major ⇒
  Cargo unifies it; the shipped binary is unchanged). Substring assertions on the
  pretty JSON would have avoided the dep but are brittle; the typed parse into
  `anvil::hunt::HuntReport` is the signoff-quality choice.
- **Book over a runnable example.** The book's "bug-hunting loop end to end" now
  features the turnkey `hunt` (CLI + MCP), but its `anvil hunt …` block carries
  the `book-test: skip` sentinel — the loop is tool-gated, and `book_examples`
  must stay green on a tool-less CI. USER_GUIDE (not scanned by `book_examples`)
  carries the runnable examples.
- **Closure bookkeeping.** `.2e` closes `.2`, the tree, and the root node; ROADMAP
  owner-directed lane 1 flips to DONE. No `src/` generator/emitter change ⇒ DUT
  byte-identical; the only code is `tests/` + a dev-dep.

## 2026-06-17 — Bug-hunt orchestration — the `anvil hunt` CLI subcommand — `BUG-HUNT-ORCHESTRATION.2d`

ANVIL's **first subcommand**. The whole risk here is *not perturbing the
flat-flag default path* — `anvil --seed N …` and every book bash block must stay
byte-identical. Choices:

- **Optional subcommand, flat flags stay top-level.** `Cli` gains one field —
  `#[command(subcommand)] command: Option<Commands>` — and keeps all ~60 flat
  flags. clap then parses `anvil --seed 42` with `command == None` (the existing
  flow runs untouched) and `anvil hunt …` with `command == Some(Hunt(..))`. No
  `args_conflicts_with_subcommands`, no required flat flag — both would break one
  of the two cases. The byte-identical guard is asserted at the parse level
  (`flat_default_invocation_has_no_subcommand`) on top of `snapshots`/`book_examples`.
- **Dispatch before the lane/DUT path, return early.** `main` matches
  `&cli.command` right after `init_tracing` and returns `run_hunt_command(hunt)`
  for the `Some(Hunt)` case; the entire historical body sits in the fall-through
  (`None`) path, edited only by being preceded by the early return.
- **`build_hunt_request` split out for a tool-free proof.** A finding needs a
  real downstream failure, so the *execution* of `run_hunt_command` isn't
  cargo-portable. The arg → `HuntRequest` mapping is the new logic, so I factored
  it into `build_hunt_request(&HuntCommand) -> Result<HuntRequest>` and unit-test
  *that* (seed stamped into the knob profile, empty `--tools` → the
  verilator+yosys default, `--no-minimize`/`--budget`/`--diff-sim`/`--out` map
  through). The end-to-end run is covered by a manual real-tool smoke (a 3-seed
  verilator sweep returned `n_failures = 0` with distinct per-seed run_ids) and
  the dedicated `.2e` gate.
- **`AcceptanceTool` gained `clap::ValueEnum`.** `--tools verilator,yosys` parses
  straight to `Vec<AcceptanceTool>` (kebab-case of the variants already matches
  the tool names), mirroring how `YosysMode` is already a `ValueEnum`. A pure
  derive addition — no behaviour change.
- **`--out` is the CLI's bundle switch; the MCP path stays cache-only.** The
  human CLI directs reproducer bundles to `--out <dir>` (→ `HuntRequest.bundle_root`),
  exactly the human convenience decision 0018 reserves for the CLI; the MCP `hunt`
  tool keeps `bundle_root = None` and serves artifacts from its cache. The
  validate sandbox is always the OS-temp default in both (decision 0004).

## 2026-06-17 — Bug-hunt orchestration — the MCP `hunt` controlled tool — `BUG-HUNT-ORCHESTRATION.2c`

`hunt` becomes the MCP surface's first **orchestration** tool (vs. the existing
single-step tools), a thin shim over `anvil::hunt::run` (decision `0017`: the
action is MCP-invocable and its results queryable). Choices worth keeping:

- **MCP path: `bundle_root = None`; artifacts come from the cache, not disk.**
  The decision-0018 reproducer bundle is a *directory*, which is exactly the
  `anvil hunt --out` CLI convenience (`.2d`). For the MCP tool, writing
  directories from a server call is messy (cleanup, the "agent never supplies a
  path" rule) and redundant: `generate`/`introspect` already make artifacts
  queryable by **caching** them and serving `anvil://artifact/<run_id>/…`
  resources. So `run_hunt` sets `bundle_root = None` and instead calls
  `cache_hunt_failures`, which populates `self.cache` for each finding's
  `run_id` (original + minimized) via the `downstream::introspect_dut_artifact`
  added in `.2b.2b`. The agent reads the reproducer through the same resource
  scheme it already uses — no new mechanism, no filesystem side effects beyond
  the auto-removed validate sandboxes.
- **The cache key falls out for free.** `cache_artifact` keys on the
  introspection document's `run_id`, and `introspect_dut_artifact(seed, cfg)`
  produces a document whose `run_id == content_run_id("dut", seed, cfg)` ==
  exactly the finding's `run_id` (the sweep stamps `cfg.seed = seed`; minimize
  holds the seed fixed). So caching the original `(base_cfg with seed)` and the
  `minimized_config` lands them under the addresses the report already advertises.
- **One top-level `hunt` audit record, not per-seed.** The library `hunt::run`
  composes the **library** `downstream::validate`/`minimize`, which do *not*
  touch `self.audit` (only the MCP `run_validate`/`run_minimize` wrappers do).
  That is by design — the orchestrator composes the library, not the MCP tools.
  So a hunt emits one summarizing audit record (sweep params + summary + each
  finding's seed/run_id/failing_tool/detection), not N per-seed records.
- **Shared budget parser (full-factorization).** The `max_oracle_calls` parse
  block was inline in `run_minimize`; I lifted it to a free
  `parse_max_oracle_calls(args)` and have both `run_minimize` and `run_hunt`
  call it (byte-identical for minimize), beside new `parse_hunt_seeds` /
  `parse_bool_arg` helpers — one parser per knob, mirroring
  `parse_validate_tools`/`parse_yosys_mode_arg`.
- **No introspection schema bump.** The `HuntReport` is a *tool result*, not a
  section of the `IntrospectionDocument`; the cache serves the existing
  schema-`1.11` introspection documents unchanged. The schema constant is
  untouched.
- **Cache-population is unit-tested without a real tool.** A finding needs a
  real downstream failure, which a cargo-portable test can't produce. So
  `cache_hunt_failures` is exercised directly with a *synthetic* `HuntReport`
  (one failure), then `resources/read anvil://artifact/<run_id>/sv` is asserted
  to resolve to the real regenerated SV — the same path the tool runs, minus the
  unreproducible-without-tools failure.

## 2026-06-17 — Bug-hunt orchestration — reproducer-bundle emitter — `BUG-HUNT-ORCHESTRATION.2b.2b`

The second-half slice of `.2b.2`: on each finding, `hunt::run` (when
`HuntRequest.bundle_root` is set) writes a self-contained, one-command-
reproducible **directory** `<bundle_root>/<run_id>/`. Design choices and
gotchas worth keeping:

- **`introspect_dut_artifact` as a sibling of `generate_dut_artifact`, not a
  fourth copy of the dispatch.** The bundle needs both the emitted SV
  (`repro.sv`) *and* the construction-truth `IntrospectionDocument`
  (`introspection.json`). `generate_dut_artifact` already owns the
  module-vs-design dispatch for the SV (extracted at `.2b.2a` precisely so the
  branch is not copied per caller). Rather than re-copy that `if
  effective_hierarchy_depth_range().is_some()` branch a fourth time inside
  `hunt`, I added its introspection analogue beside it in `downstream` —
  `introspect_dut_artifact(seed, cfg) -> IntrospectionDocument` — projecting
  through the **pure** `introspect::module_document`/`design_document` (which
  only re-project an already-generated `Module`/`Design`, never generate). The
  hunt bundle emitter therefore copies no dispatch; both projections live in
  one home, available to a future `mcp`/`main` convergence.
- **`repro.sh` substitutes the dead sandbox SV path, it does not replay it
  verbatim.** `validate` runs each tool in an ephemeral per-run sandbox it then
  removes, so the captured `ToolInvocation.argv` references a path that no
  longer exists. `repro.sh` step 1 regenerates the artifact as `repro.sv` in the
  bundle dir; step 2 replays the failing tool's `argv` with every occurrence of
  the sandbox SV path (`<sandbox>/<top>.sv`) plain-substring-replaced by
  `repro.sv`. That single replace also rewrites the path embedded inside a Yosys
  `-p` script, because a temp path needs no double-quote escaping and so appears
  verbatim in the script string. Each token is POSIX single-quoted
  (`shell_quote`) so a `;`/space/`"` in the Yosys script survives.
- **Tool logs are not copied — a NOTE explains why.** The sandbox (and its
  `<stem>.<tool>.<stream>.log` sidecars) is gone by the time the bundle is
  written, so `tool-logs/` carries a `NOTE.txt`: the first failing line is in
  `hunt-verdict.json` (`first_error` / `diff_sim.mismatch_excerpt`), and
  `./repro.sh` regenerates the full output. This is the leaf goal's explicit
  "or a note that repro.sh regenerates them" escape hatch — honest, not a
  silent omission.
- **The bundle prefers the minimized reproducer.** When `minimize` confirmed a
  smaller still-failing config (`reproduced_initial && final_validation`), the
  bundle reproduces *that* (its `minimized_config` → `repro.sv`/`knobs.json`,
  its report → `repro.sh`), and the directory is named by the minimized
  `run_id`. Otherwise it bundles the originally-detected `(cfg, report)`. A
  `cross_sim_mismatch` finding has no rejecting tool, so `repro.sh` step 2
  points the filer at the recorded `diff_sim` excerpt instead of a command.
- **`hunt-verdict.json` omits the self-referential `bundle` ref.** The on-disk
  verdict is serialized while `HuntFailure.bundle` is still `None` (the ref is
  attached to the in-memory failure only after the directory is written), so
  the file does not point back at the directory that contains it.
- **Default-off / DUT byte-identical.** `bundle_root` defaults to `None`; with
  it unset, `hunt::run` writes nothing new and the generator/emitter are
  untouched (`tests/snapshots.rs` 6/6). The whole emitter is pure composition
  over `generate_dut_artifact` + `introspect_dut_artifact`; it runs no tool, so
  the four new proofs are cargo-portable (the bundle emitter is unit-tested
  directly with a synthetic failing `ValidateReport`, since a real finding
  needs a real tool).

## 2026-06-17 — Bug-hunt orchestration — cross-sim fold + shared generate helper — `BUG-HUNT-ORCHESTRATION.2b.2a`

The second-half-first slice of `.2b.2` (pre-split into `.2b.2a` cross-sim fold +
`.2b.2b` bundle emitter). Two pieces:

1. **`downstream::generate_dut_artifact(cfg) -> (kind, top, sv)` extracted.**
   `validate` had the DUT design-vs-module dispatch inline; the hunt's cross-sim
   detector needs the same `(top, sv)` to drive the two simulators on exactly the
   artifact `validate` accepted. Rather than copy the branch a third time
   (`validate` had it; `tool_matrix` has its own), I lifted it into a shared
   `downstream` helper that `validate` now calls — byte-identical (the
   `downstream` lib tests stay 20/0). Deliberately takes `cfg` only (not `seed`):
   generation seeds from `cfg.seed`, and the `seed` arg is only a run_id input,
   so a `seed` parameter would be dead weight.

2. **Cross-sim mismatch folded into `hunt::run`.** `HuntRequest.diff_sim: bool`;
   when set, each *validate-clean* artifact is re-checked by
   `cross_sim_mismatch(req, cfg, run_id)` → `diff_sim::run_agreement` in a
   per-run sandbox under the caller-set `sandbox_root` (removed unless
   `keep_sandbox`). A real mismatch (`ran && !success`) is a finding with
   `detection == "cross_sim_mismatch"`, carrying the `DiffSimReport` in the new
   `HuntFailure.diff_sim`. Design choices worth recording:
   - **Cross-sim runs only on parse/synth-clean artifacts.** A tool that already
     *rejected* the SV is the finding; there's no point asking simulators to
     agree on output a tool refused (the book's framing; the `tool_matrix`
     `--diff-sim` ordering).
   - **Cross-sim findings are NOT minimized.** The `minimize` oracle is
     `validate` (parse/synth acceptance); it cannot reproduce a *trace*
     disagreement (validate says clean), so `reproduced_initial` would be false
     and the shrink would be meaningless. `minimized` is `None` for these; a
     diff-sim-oracle minimize is a possible future extension.
   - **Cargo-portable proof.** `diff_sim_on_clean_artifact_no_ops_without_simulators`
     guards on `tools_present()` and proves the fold is a no-op when a simulator
     is absent (so a `--diff-sim` hunt on a tool-less host never invents a
     finding). The present-tools path is owned by `tests/diff_sim.rs` + the
     `tool_matrix` `#[ignore]` e2e gate.

## 2026-06-17 — Bug-hunt orchestration — seed-threading gotcha — `BUG-HUNT-ORCHESTRATION.2b.1` (fix)

A correctness fix to `.2b.1`'s loop, found while grounding `.2b.2` against the
real `validate` body. **Gotcha worth remembering for any future
`validate`/`minimize` caller:** the `seed` *argument* to `validate(seed, cfg,
…)` feeds only the `run_id` (the content address) and the audit log — the
**generator seeds from `cfg.seed`** (`Generator::new(cfg)` →
`ChaCha8Rng::seed_from_u64(cfg.seed)`). The two must agree, and the established
convention is the caller stamps it (`config_from_args` does `cfg.seed = seed`
before every `validate`/`minimize`). A sweep that passes a fixed profile config
and only varies the `seed` arg gets **distinct run_ids over an identical
artifact** — a silent no-op fuzz. `hunt::run` now stamps `seed` into a
per-iteration `seed_config(req, seed)` clone. The lesson: when threading a seed
sweep through `validate`/`minimize`, vary `Config::seed`, not just the `seed`
argument.

## 2026-06-17 — Bug-hunt orchestration — `src/hunt/` library core — `BUG-HUNT-ORCHESTRATION.2b.1`

The engine of decision `0018`, as the first half of `.2b` (pre-split into
`.2b.1` loop-core + `.2b.2` cross-sim+bundle). `src/hunt/mod.rs` +
`pub mod hunt`. Design points worth recording:

1. **Thin orchestrator, proven by what it imports.** `hunt::run` imports
   `downstream::{validate, minimize}` and *nothing that detects or generates*.
   Detection is `!ValidateReport.ok` — and because `validate`'s
   `first_tool_warning` already folds a warning into `ok == false`, reject and
   warning are one signal with **zero** new parsing in `hunt`. The only
   `hunt`-local logic is (a) the seed sweep, (b) classifying a found failure as
   `reject` vs `warning` (`exit_code == Some(0)` ⇒ warning, else reject), and
   (c) projecting `ValidateReport`/`MinimizeReport` into the report shape.

2. **`HuntRequest` embeds `ValidateOptions`, not a parallel knob set.** The
   per-seed downstream run is configured by the existing `ValidateOptions`
   (tools / yosys_mode / mem_limits / **caller-set sandbox_root** / keep_sandbox),
   reused verbatim as the `MinimizeOptions.validate` oracle so a minimized
   reproducer is gated by the *same* guardrails. This keeps the sandbox path
   caller-set (never agent-supplied — decision `0004`) and avoids a second copy
   of the tool/sandbox policy.

3. **Every `HuntReport` field is SCHEMA-DERIVED.** `HuntVerdict` / `HuntFailure`
   / `HuntMinimized` / `HuntSummary` are pure projections of
   `ValidateReport.{run_id,ok,declined,tools}`, `ToolInvocation.{tool,argv,error,
   exit_code,success}`, and `MinimizeReport.{reproduced_initial,reductions,
   oracle_calls,budget_exhausted,final_validation}` — no new computed truth, no
   behavioural oracle (decisions `0017` / `0004`). `minimized_run_id` reads
   `final_validation.run_id` when the search reproduced, else falls back to the
   original `run_id` (the minimized config then echoes the input, so the
   addresses coincide) — avoiding a dependency on `content_run_id` visibility.

4. **Cargo-portable proofs without real tools.** The key proof
   (`run_no_tool_smoke_is_all_clean`) sets `ValidateOptions.tools = vec![]` — a
   no-tool smoke `validate` (generate + sandbox only) returns `ok == true`
   vacuously — so the loop's sweep + aggregation are proven against a real
   `validate` without iverilog/verilator/yosys present. Plus: reproducible
   run_ids (content addressing), `classify_detection` warning-vs-reject,
   `first_failing_tool`, and a `HuntReport` serde round-trip (confirming the
   `skip_serializing_if` fields stay absent in the wire form the MCP tool will
   serve). lib 505→510; `tests/snapshots.rs` 6/6 byte-identical — `hunt` is
   wired into no generate/emit path, so DUT output is untouched.

5. **Scope held to the library core.** No cross-sim detection, no on-disk
   bundle, no CLI/MCP — those are `.2b.2` / `.2c` / `.2d`. `HuntFailure.detection`
   already reserves `"cross_sim_mismatch"` for `.2b.2`; the report's `lane` is
   `"dut"` (validate/minimize are DUT-only; non-DUT is a future extension noted
   in the type docs).

## 2026-06-17 — Bug-hunt orchestration — extract diff-sim run+compare — `BUG-HUNT-ORCHESTRATION.2a`

First implementation leaf of decision `0018`'s pre-split `.2`. A pure,
byte-identical move: the per-module diff-sim run+compare pipeline left the
`tool_matrix` binary for the `anvil::diff_sim` library so the bug-hunt loop
(`.2b`) — and `ACCEPTANCE-DIVERGENCE-HUNTING` — detect a cross-simulator
mismatch through one hardened surface instead of duplicating the harness. The
mechanism mirrors `AGENT-INTROSPECTION-MCP.5.1` (which earlier lifted the
acceptance-tool invocations into `anvil::downstream`) and
`DIFFERENTIAL-SIMULATION.3b.1` (the diff-sim *primitives* extract). Notes:

1. **Moved as-is; not merged.** `src/diff_sim/mod.rs` now owns `DiffSimReport`,
   `DutPort`, `parse_dut_ports`, `emit_testbench_for_ports`,
   `push_display_for_ports`, and a new
   `run_agreement(work_dir, top_name, sv_text, n_vectors) -> DiffSimReport`. The
   bodies are verbatim, so the emitted `tb.sv` and the serialized `DiffSimReport`
   are byte-identical and `tool_matrix_report.json` is unchanged. The library
   still carries **two** testbench emitters — the IR-driven `emit_testbench`
   (canonical, used by `tests/diff_sim.rs`) and the SV-text-driven
   `emit_testbench_for_ports` (the matrix/hunt path, no live `Module` in scope).
   Unifying them is a deferred cleanup; `.2a`'s contract is a byte-identical
   move, not a behaviour change, so a merge that could perturb either path's
   output was explicitly out of scope.

2. **`run_agreement` takes the work dir, not `(scenario_dir, stem)`.** The old
   `run_diff_sim_for_module` computed `dir = scenario_dir.join("<stem>-diff-sim")`
   internally. The reusable entry takes the already-joined `work_dir` so any
   caller (the matrix wrapper, the future hunt loop) controls sandbox placement —
   the sandbox path stays *caller-set, never agent-supplied* (decision `0004`).
   The `tool_matrix` wrapper keeps the exact old dir name, so existing output
   trees are unchanged. The hardcoded `8` baked vectors became the `n_vectors`
   parameter (the wrapper passes `8`).

3. **Tests follow the code.** The two pure-unit tests
   (`parse_dut_ports_recognises_anvil_emitter_shape`,
   `emit_testbench_for_ports_renders_combinational_and_sequential_shapes`) moved
   into the `diff_sim` test module with their functions; a new
   `run_agreement_is_a_friendly_no_op_without_tools` covers the tools-absent
   path. The `tool_matrix` `#[ignore]` e2e gate stays (it now exercises the
   wrapper → `run_agreement`), as does the coverage-fact test (it constructs the
   now-`pub`-fielded `DiffSimReport`). Net: lib 502→505, tool_matrix 73→71
   passed + the e2e ignored; `tests/snapshots.rs` 6/6 byte-identical proves DUT
   output untouched.

## 2026-06-17 — Bug-hunt orchestration — design ADR — `BUG-HUNT-ORCHESTRATION.1` (decision `0018`)

Picked the owner-recommended highest-leverage usability lane (idea 1) at the PNT
boundary. This is the `.1` design/decision leaf — docs-only, no `src/` — so the
code state stays the green `.10b.3` baseline and the DUT contract is untouched.
Decision `0018` pins the loop; a few choices worth recording so `.2` does not
re-derive them:

1. **Thin orchestrator, not a new engine.** The whole loop is *composition* over
   surfaces that already exist as library functions. `src/downstream/mod.rs`
   already exports `validate(seed, cfg, &ValidateOptions) -> ValidateReport` and
   `minimize(seed, cfg, &MinimizeOptions) -> MinimizeReport`; crucially,
   `first_tool_warning` already folds a *warning* into `ToolInvocation.success ==
   false`, so **reject and warning are already one unified failure signal** —
   the hunt needs no warning parser of its own. So `src/hunt/` adds only
   `hunt::run(&HuntRequest) -> HuntReport` (the seed-sweep + bundle emitter); it
   adds **no** second detector and **no** second minimizer. This is the
   full-factorization reflex applied to orchestration: one source of truth for
   "is this a finding?".

2. **diff-sim must be extracted before the loop can reuse it (`.2a`).** The
   `src/diff_sim/` module today holds only the deterministic *primitives*
   (`baked_input_vectors`, `emit_testbench`, `is_sequential`); the actual
   *run-both-sims-and-compare* lives inside `src/bin/tool_matrix.rs` (the
   `DiffSimReport` producer). So the cross-sim mismatch detector cannot be
   composed without first promoting that run+compare into a reusable
   `diff_sim::run_agreement(...)` library entry — the same extract-then-reuse
   the `DIFFERENTIAL-SIMULATION.3b.1` refactor did. Recorded as the first impl
   sub-leaf (`.2a`), orderable first; a first hunt cut may ship reject/warning
   only and fold cross-sim in next.

3. **Bundle = a directory, not an archive.** Matches the existing
   `--out`/`tool_matrix` directory-tree convention, stays inspectable/diffable/
   git-attachable for filing, and lets an agent fetch parts as `anvil://…`
   resources without unpacking. An archive view is a trivial later add-on.

4. **`anvil hunt` is ANVIL's first subcommand — guard the default path.** The CLI
   is flat flags + `--artifact` today (no subcommands). Adding a `hunt`
   subcommand (clap) must not perturb the default `anvil --seed N …` invocation,
   which is load-bearing for `tests/snapshots.rs` (6/6) and
   `tests/book_examples.rs::every_runnable_book_bash_block_succeeds` (3/3). `.2d`
   owns proving both unchanged. A `hunt` subcommand was chosen over an
   `anvil --hunt` flag because the hunt's option set (`--seeds`, `--budget`,
   `--no-minimize`, …) is mutually-exclusive with the generate flags and would be
   awkward to overlay; nothing is retired.

5. **Sandbox path is caller-set, never agent-supplied (decision `0004`).** The
   MCP `hunt` tool writes bundles to a fixed sandboxed per-run dir and returns
   its path + resource URIs; only the *human* CLI may pass `--out <dir>`. This
   mirrors `ValidateOptions.sandbox_root` being caller-set — the agent never
   hands a filesystem path to a controlled tool. So `hunt::run` takes a
   `bundle_root: PathBuf` from its caller; the MCP shim fixes it, the CLI shim
   exposes it.

6. **Every `HuntReport` field is SCHEMA-DERIVED.** The report is a projection of
   `ValidateReport`/`MinimizeReport`/`DiffSimReport`/`ToolInvocation` — no new
   computed truth, satisfying decision `0017`'s queryable gate without breaching
   the no-shadow-simulator ceiling (decision `0004`). The hunt *classifies*
   findings (reject | warning | cross_sim_mismatch); it never adjudicates them.

## 2026-06-17 — Structured emission — cone-function metric + repo-owned gate — `STRUCTURED-EMISSION-EXPANSION.10b.2`

The fifth surface's metric (`Metrics::num_emitted_cone_functions`,
`= m.cone_function_gates.len()`) bumps the introspection schema `1.10 → 1.11`
(the `.6b.2a` precedent: a new derived `Metrics` field bumps; the `.10b.1` knob
rode the version via `#[serde(default)]`). The repo-owned
`tool_matrix --cone-function-gate` is templated on `--task-emit-gate` /
`--function-emit-gate`. Two non-obvious choices worth recording:

1. **Detection token is `__cf(`, not `function automatic`.** The cone surface
   renders `function automatic logic [..] <root>__cf(...)`, so its emitted SV
   *also* contains `"function automatic"` — the same substring the single-gate
   `emitted_combinational_function` probe matches. Reusing that probe would
   blur the two surfaces (and, in the cone sweep, light
   `saw_combinational_function_emit` too). `ModuleReport.emitted_cone_function`
   therefore probes the cone-specific `"__cf("` token (the call + decl name
   suffix), which is disjoint from the single-gate `"__f("` token. The
   `ConeFunctionSweep` gap arm checks only `saw_cone_function_emit`, so any
   incidental `saw_combinational_function_emit` lighting is harmless.

2. **`cone_function_focus_config` uses `terminal_reuse_prob = 0.3`, not `0.9`.**
   The `function_emit` / `task_emit` focus configs use `terminal_reuse_prob =
   0.9` — fine for those single-gate surfaces, which absorb a gate regardless of
   its fanout. The cone surface absorbs an interior gate **only when it is
   single-use** (`use_count == 1`; the `.10b.1` soundness rule). Heavier
   terminal reuse drives more CSE-induced sharing under node-id + e-graph, which
   turns interior gates multi-use → boundary params → smaller (or empty) cones.
   Dropping the focus config's reuse to the default `0.3` keeps single-use
   interior gates plentiful so the gate reliably fires. Empirically the gate
   emits a cone function in **12/12** modules (148 cone functions total) at
   `0.3`. Banked clean `/tmp/anvil-cone-function-gate-r1` (3 scenarios / 12
   modules / `coverage_gaps = []` / `12/0` Verilator + both Yosys + Icarus).
   Default-off / DUT byte-identical (snapshots 6/6).

## 2026-06-17 — Structured emission — multi-gate-cone `function automatic` live surface — `STRUCTURED-EMISSION-EXPANSION.10b.1`

The fifth surface (decision `0016`, designed at `.10a`) goes live. Two
implementation wrinkles surfaced during the build that the `.10a` design-detail
under-specified — recorded here so the next session does not re-derive them:

1. **`node_ref` resolves intrinsic-name nodes by *kind*, ignoring the `names`
   array.** For `Node::PrimaryInput` / `Constant` / `FlopQ` / `MemRead` /
   `FsmOut`, `node_ref` returns the port name / literal / flop name etc. *without*
   consulting `names[id]` (only `Gate` / `InstanceOutput` use `names[id]`). So the
   tempting shortcut — reuse `render_gate` with a per-function `names` override
   mapping boundary inputs to `a{i}` — does **not** work (an input boundary param
   would still render as its port name inside the function). The fix is a
   dedicated `render_cone_gate_expr` that maps each operand `NodeId` → its
   in-function name via an explicit resolver (`cone_operand_ref`): interior gate →
   its module wire name (the function-local), `Constant` → its literal (folded
   inline), boundary leaf → `a{i}`. It mirrors `render_gate`'s operator match (a
   third sibling of `render_gate` / `render_gate_function_body`; a future cleanup
   could DRY the three behind one resolver, but keeping them separate preserved
   the byte-identical contract on the hot `render_gate` path).

2. **Absorbing an interior gate must suppress its module-level WIRE declaration,
   not just its inline `assign`.** Unlike the sibling projections (whose marked
   gate keeps a driven module wire), an absorbed cone interior gate lives *only*
   inside the function. Leaving its `wire <name>;` declaration at module level
   would make it undriven ⇒ `-Wall` UNDRIVEN/UNUSED. So the net-declaration loop
   *and* the gate-assign loop both skip the `cone_interior` set (built once from
   `m.cone_function_gates.values()`).

**Soundness of suppression — the single-use rule, conservatively realized.** An
interior gate is absorbed only when its **global use-count == 1** (counted across
*all* value-consumer references: gate operands + output `drives` + flop `d` /
`mux` (`OneHot` arms + `Encoded` sel/data) + instance `inputs` — the complete set
of `NodeId` reference sites in a non-param module). A use-count of 1 means the
gate's sole consumer is the cone edge that reached it, so suppressing both its
wire and its assign is provably safe — nothing else reads it. This is the
conservative subset of the `.10a` "single-use-within-cone" rule (a gate shared
even *within* one cone has count ≥ 2 and stays a boundary param); broadening to
true within-cone sharing is a recorded follow-up. Constants fold inline (a leaf
that is not a support source); structural leaves and multi-use / sibling-marked /
`Slice` / structured gates are boundary params.

**Live shape.** `Config::cone_function_emit_prob` (default `0.0`, config-file-only,
separate from `function_emit_prob`) + `Module.cone_function_gates: BTreeMap<NodeId,
Vec<NodeId>>` + new `src/ir/cone_function_emit.rs` (`annotate_cone_function_gates`
— the use-count, the post-order single-use cone-walk, greedy node-order selection
with interior reservation, run last) + two guarded `gen/mod.rs` rolls (single +
design, after `task_emit`) + the `src/emit/sv.rs` cone-decl section +
interior-suppression in the net-decl + gate-assign loops + the four render helpers
+ 8 lib proofs. No metric / no schema bump (the metric `num_emitted_cone_functions`
+ schema `1.10 → 1.11` land at `.10b.2`, the `.6b.1`/`.4b.1` precedent).
Default-off / DUT byte-identical (snapshots 6/6; lib 493 → 501). Forced
`cone_function_emit_prob=1.0` sweep (`/tmp/anvil-cf-sweep/`, 8 seeds, 18 cone
functions): Verilator `--lint-only` `-Wall` Δ=0 vs OFF + Yosys both modes + Icarus
all clean. `.10b` pre-split → `.10b.1` (this) / `.10b.2` (metric + gate) / `.10b.3`
(user docs).

---

## 2026-06-17 — Structured emission — multi-gate-cone `function automatic` impl design-detail — `STRUCTURED-EMISSION-EXPANSION.10a`

Grounds decision `0016` in the real emitter + cone-walk and resolves the five
open questions it deferred to `.10a`. Read this session: `src/ir/function_emit.rs`
(`gate_qualifies` + `annotate_function_emit_gates` — the single-gate predicate to
fork), `src/introspect/analyze.rs` (`build_cone` + `visit` at ~`729`/`769` — the
post-order fan-in walk with the support-leaf boundary rules), `src/emit/sv.rs`
(`render_gate_function_decl`/`render_gate_function_body`/`render_gate_function_call`
at ~`1293`–`1399`, the function-decl section at ~`371`, the gate-assign-loop
inline-suppression at ~`440`–`500`), `src/gen/mod.rs` (the
`soft_union → function_emit → generate_loop → task_emit` call-site roll chain at
~`91`–`125` and the design-level mirror at ~`293`–`327`), `src/config.rs` (the
`*_emit_prob` knob defaults + validation + `dump-config` list). **No source change
in this leaf.**

The fifth surface deepens the first (decision `0012`): the single-gate function
takes one gate's **direct operands** as positional params `a0..a{n-1}` and a
one-line body; the cone function takes the cone's **support leaves** as params and
a **multi-statement body** (one function-local per absorbed interior gate, topo
order) returning the root. The cone-walk's boundary rules are exactly
`analyze::visit`'s (PrimaryInput / Constant / FlopQ / MemRead / FsmOut /
InstanceOutput stop), extended with three emitter-surface boundaries (below).

**Resolved open questions.**

1. **Knob + selection shape.** New `Config::cone_function_emit_prob` (default
   `0.0`, config-file-only, `#[serde(default)]`, `0.0..=1.0` validation, in
   `dump-config`), **separate** from `function_emit_prob` (decision `0016`: the
   single-gate surface stays byte-identical). Selection = **walk candidate roots,
   roll per qualifying cone**: a new `src/ir/cone_function_emit.rs`
   `annotate_cone_function_gates(m, rng, prob)` scans gates in node order; a gate
   is a **root candidate** iff it is an admissible op (the `function_emit`
   `gate_qualifies` set — non-structured, non-`Slice`, `>= 1` operand), is **not**
   already marked by any sibling projection (`function_emit_gates` /
   `generate_loop_gates` / `task_emit_gates` / `soft_union_slice_gates`), and its
   cone (below) has **`>= 1` absorbed interior gate** (so the body is genuinely
   multi-statement; a zero-interior cone is just the single-gate surface and is
   left to `function_emit`). For each candidate in order, `rng.gen_bool(p)` decides
   selection; on success the cone is recorded and its absorbed interior gates are
   reserved so a later root cannot also absorb them (first-come, deterministic by
   node order). Mirrors `annotate_function_emit_gates`' collect-then-mark shape;
   `param_env.is_some()` modules are skipped (the Phase-5 parameterized scope-out,
   as in the sibling passes).

2. **Interior-node admissibility + fanout handling.** A purpose-built post-order
   walk from the root (mirroring `analyze::visit`'s structure, **not** calling it —
   that fn is private and tailored to the `SupportCone` doc shape; coupling the
   emitter annotation to the introspection doc is undesirable). A reached
   `Node::Gate g` is **absorbed as an interior local** iff *all* hold: (a) `g` is
   an admissible op (non-structured, non-`Slice`); (b) `g` is **not** sibling-marked
   or already reserved by another cone; (c) `g` is **single-use within the cone** —
   every consumer of `g` in the module is inside this cone (not a module-output
   drive, not a flop `D`, not a node outside the cone). Otherwise `g` is a **cone
   boundary** and becomes a **parameter** (its module wire is passed in), keeping
   its own inline `assign` outside the function. Structural leaves
   (PrimaryInput / FlopQ / InstanceOutput) are boundary **params**; `Constant` is
   **folded inline as a literal in the body** (not a param — it is a leaf-but-not-a-
   support-source per `visit`, and folding avoids an unused-width param). The root
   itself is always absorbed (it is the return). This guarantees every function-body
   node renders as a blocking assign over params/earlier-locals/literals and is
   robustly `-Wall` clean (no `UNUSEDSIGNAL`: every param is used; multi-use gates
   stay external).

3. **Topo-order + local naming.** The post-order DFS yields absorbed interior
   gates in dependency order (children before parents) — emit one
   `logic <width> <wire>;` decl per absorbed interior gate (reusing the gate's
   existing **module wire name**, e.g. `and_3`, as the function-local name: unique
   within the module ⇒ unique within the function; params are `a0..a{k-1}` so no
   clash), then the assignments in that order, then `<root>__cf = <root-expr>;`.
   Body rendering = a cone variant of `render_gate_function_body` that maps each
   operand `NodeId` → its **in-function name**: a boundary leaf → its param `aI`
   (deterministic param order = ascending `NodeId` of the boundary set; a distinct
   boundary `NodeId` ⇒ exactly one param, unlike the single-gate positional
   duplication), an absorbed interior gate → its local wire name, a `Constant` →
   its literal. Return type + param widths via `param_width_decl_w` (as in
   `render_gate_function_decl`).

4. **Pass ordering + mutual exclusion.** Run `annotate_cone_function_gates`
   **last** in both call-site chains (after `task_emit`), guarded on
   `cfg.cone_function_emit_prob > 0.0` so the default draws nothing ⇒ byte-identical
   stream. Running last means the four sibling marks are visible: a sibling-marked
   gate is never a cone root and is never absorbed (it stays a boundary param). The
   reserved-interior set prevents two cones from claiming the same gate. IR change:
   `Module.cone_function_gates: BTreeMap<NodeId, Vec<NodeId>>` (root → topo-ordered
   absorbed interior gate ids; the params + literals are derived at emit time from
   the interior gates' operands), `BTreeMap` for deterministic emit order. The
   emitter (`to_sv_with_modules`): in the function-decl section, for each
   `(root, interior)` render `render_cone_function_decl` (the multi-statement body)
   + at the gate-assign loop, **suppress the inline assign of every absorbed
   interior gate** (they live only inside the function) and **replace the root's
   assign RHS with the `<root>__cf(<param refs>)` call** — the existing
   `function_emit_gate`/`generate_loop_gate`/`task_emit_gate` suppression pattern,
   extended to the cone's interior set.

5. **Metric + gate.** `Metrics::num_emitted_cone_functions = m.cone_function_gates.len()`
   (`#[serde(default)]`), surfaced in introspection `module_metrics` ⇒
   `SCHEMA_VERSION` `1.10 → 1.11` (the metric bumps; the `.10b.1` knob rides the
   version, the `function_emit`/`generate_loop`/`task_emit` precedent). New
   `tool_matrix --cone-function-gate` + `ScenarioSet::ConeFunctionSweep` +
   `build_cone_function_sweep_scenarios`/`cone_function_focus_config` (one comb-only
   `cone_function_emit_prob=1.0` DUT × the three construction strategies) +
   `ModuleReport.emitted_cone_function` (SV-text detection of `__cf(`) +
   `saw_cone_function_emit` + `MatrixReport.cone_function_gate` + early-return gap
   arm + proofs, templated on `--function-emit-gate`. Full Verilator + both Yosys +
   Icarus plan (a synthesizable function is accepted by every tool).

**`.10b` impl shape.** New `src/ir/cone_function_emit.rs` (+ `src/ir/mod.rs`
registration) with the walk + `annotate_cone_function_gates`; `Config`
+`cone_function_emit_prob`; `Module.cone_function_gates`; the `src/emit/sv.rs`
`render_cone_function_decl` + call + interior-suppression; lib proofs (cone mark
with `>= 1` interior; multi-statement emit shape with locals; single-use-only
absorption / multi-use stays a param; sibling-marked excluded; the single-gate
`function_emit` surface unchanged; identity/node-count untouched; an emit/sim
faithfulness check); the metric + schema `1.10 → 1.11`; the `--cone-function-gate`
proof; book/USER_GUIDE/KM. Default-off / DUT byte-identical (snapshots untouched).
Pre-split `.10b.1` (live surface) / `.10b.2` (metric + gate) / `.10b.3` (user docs)
if warranted, the `.6b`/`.4b` precedent.

---

## 2026-06-17 — Structured emission — picking the fifth surface (the multi-gate-cone `function automatic`) — `STRUCTURED-EMISSION-EXPANSION.9` (decision `0016`)

Design/decision leaf, **no source change**. At a no-active-frontier boundary
(the fourth surface closed at `.8b`), autonomously selected the fifth structured
surface per `feedback_pick_and_roll_at_no_frontier`. The reasoning the decision
record (`docs/decisions/0016-…`) condenses:

**The decisive axis is by-construction source, not downstream cleanliness.** A
fresh probe (`/tmp/anvil-se9-probe/`) ran the three recorded candidates through
Verilator 5.046 `-Wall` + both Yosys 0.64 modes + Icarus 13.0, and *all three*
came back with **zero `%Warning`**:
- a **multi-gate-cone `function automatic`** (function-local `logic` temps + a
  topo-ordered statement body) — sim-proven bit-equal to the inline cone
  `((a&b)|c)^a` over 4000 random vectors;
- a **multi-output combinational `task automatic`** — also clean + sim-equiv over
  4000 vectors (this *clears* the decision-`0012` "multi-output task" caution with
  fresh evidence; the caution was really about side-effecting tasks);
- **nested/multi-level `generate`** — clean across all tools.

Since cleanliness doesn't discriminate, the tiebreaker is the axis decision
`0015` already flagged: **does the surface have a routine by-construction
source?** The cone-function wins decisively — *any* combinational cone with `>= 2`
interior gates qualifies, which is pervasive in real generation — whereas the
multi-output task needs *groups* of sink gates sharing a support set (a
policy-laden grouping heuristic) and nested generate needs a 2-D replication
`{N{ {M{x}} }}` that **full factorization collapses** to `{N*M{x}}`, so there is
no node to project. So: cone-function fifth; multi-output task is the leading
deferred runner-up; nested generate deferred again (now with fresh evidence);
`interface`/`modport` stays disqualified (`0015`: Icarus syntax-fail + both-Yosys
implicit-decl warnings).

**It deepens the first surface, so it needs its own knob.** Decision `0012`
deliberately narrowed the first cut to a *single gate over its direct operands*
and recorded the *cone* (a Gate + its fan-in to the support-leaf boundary) as the
follow-up. The fifth surface is exactly that recorded follow-up. Critically, it
must **not** reuse `function_emit_prob`: that knob marks individual gates, and
folding cone selection into it would change its existing emitted output (no longer
byte-identical for that knob's users) and blur two distinct surfaces. So the fifth
surface gets its **own** default-off `cone_function_emit_prob` — the shipped
single-gate surface is untouched, nothing retired (`feedback_never_retire_strategies`).

**Why it's a genuinely new shape (not cosmetic).** The single-gate function body
is one line (`f = a OP b;`, no locals). The cone-function body is a sequence of
**function-local `logic` declarations** + topo-ordered blocking assignments for
the interior gates + the return — a real new elaboration construct (function-local
nets + an internal statement sequence) a downstream tool must handle.

**Reuse / blast radius.** The cone-walk to the support-leaf boundary is exactly
the existing `src/introspect/analyze.rs` `output_support` traversal; the rendering
extends the existing `<wire>__f` function decl/call path in `src/emit/sv.rs` to a
multi-statement body; the annotation mirrors `src/ir/function_emit.rs`. No new IR
node, no new whole-module behaviour — the `function_emit`/`generate_loop`/
`task_emit`/`soft_union` emit-projection precedent.

The `.10a` design-detail leaf will pin: the knob name + selection shape; the
interior-node admissibility set + fanout-node handling (boundary-stop vs
duplicate-into-function); the topo-order/local-naming scheme; the pass ordering +
mutual-exclusion bookkeeping vs the four sibling projections; and the
`num_emitted_cone_functions` metric (schema `1.10 → 1.11`) + the
`--cone-function-gate` / `saw_cone_function_emit` scenario.

---

## 2026-06-17 — Structured emission — wider-lane `generate for` part-select impl design-detail — `STRUCTURED-EMISSION-EXPANSION.8a`

Grounds decision `0015` in the real emitter and resolves the three open questions
it deferred to `.8a`. Read this session: `src/ir/generate_loop.rs` (the
`gate_qualifies` 1-bit-lane predicate) and `src/emit/sv.rs`
(`generate_loop_gate` at ~`1512`, `render_generate_loop_block` at ~`1548`). No
source change in this leaf.

**Corpus-liveness evidence (the surface is real, not hand-built-only).** A
300-module comb-only sweep (`/tmp/anvil-widelane-probe/`, seed 1,
`terminal_reuse_prob=0.95`, `gate_struct_weight=12`, widths 4–16) emits **448
`{N{x}}` replications, of which 20 have a multi-bit lane** (`LW > 1`) — e.g.
`{2{i_4}}` (7b→14b), `{3{case_mux_0}}` (12b→36b), `{6{i_1}}` (8b→48b),
`{4{concat_7}}` (20b→80b). So the broadened predicate fires on real generation
(~4.5% of replications), and the existing `--generate-loop-gate` corpus will
exercise the new branch once the predicate is relaxed — it is not a
hand-built-only surface.

**Resolved open questions.**
1. **`generate_loop_gate` return shape — keep `(lane, N)`; recompute `LW` in the
   renderer.** `render_generate_loop_block(name, lane, n, m, names)` already takes
   `m`, so it computes `lw = m.nodes[lane as usize].width()` itself. No signature
   change to `generate_loop_gate`; it stays `Option<(NodeId, usize)>`. (The gate
   helper still *validates* `width == N*LW` defensively, computing `LW` the same
   way.)
2. **Render branch — keep `LW == 1` byte-identical; part-select only for `LW > 1`.**
   `LW == 1` keeps the exact existing line `assign <name>[<gi>] = <x>;`; `LW > 1`
   emits `assign <name>[<gi>*LW +: LW] = <x>;`. Do **not** collapse the 1-bit case
   into `[<gi>*1 +: 1]` — that would change the shipped 1-bit surface's bytes and
   break its proofs + the `.4b` gate. The branch is a single `if lw == 1 { … }
   else { … }` around the one `writeln!` body line.
3. **Predicate relaxation in `gate_qualifies`** (`src/ir/generate_loop.rs`):
   replace `if lane.width() != 1 || *width as usize != operands.len()` with
   `let lw = lane.width(); if lw == 0 || *width as usize != operands.len() * lw as usize`
   (i.e. any `LW >= 1` with the result width matching `N*LW`). The
   `function_emit` / `soft_union` exclusions and the all-same-operand / `N >= 2`
   checks are unchanged. The same relaxation is mirrored in the emitter-side
   defensive re-check (`generate_loop_gate`).

**Downstream proof shape for `.8b`.** Primary = a **deterministic lib emit-test**
that hand-builds a wider-lane replication (e.g. `{3{x}}` with a 4-bit lane,
width 12) and asserts the rendered body is `assign <wire>[<gi>*4 +: 4] = <x>;`
(authoritative, seed-free) **plus** a test that the existing 1-bit-lane module
still renders `[<gi>]` (byte-identity guard). Bonus = the existing
`tool_matrix --generate-loop-gate` will now also project wider-lane corpus
replications; `.8b` confirms the bank stays clean and (corpus-liveness above)
that wider lanes are exercised. A behaviour/sim faithfulness check is covered by
the empirical probe (`/tmp/anvil-probe-se4/` proved `assign y[gi*8 +: 8] = b;`
≡ `{4{b}}` under iverilog); `.8b` may add a lib assertion that the unrolled
ranges tile `[0, N*LW)` exactly.

**Byte-identity contract.** Default-off (`generate_loop_emit_prob == 0.0`) is
untouched. With the knob on, only *wider-lane* replications change rendering
(they previously emitted inline `{N{x}}`); the 1-bit-lane rendering is verbatim,
so `tests/snapshots.rs` (default-off) and every shipped 1-bit `generate_loop`
proof + the `.4b` `--generate-loop-gate` bank stay green unchanged. Reuses
`generate_loop_emit_prob` + `num_emitted_generate_loops` ⇒ **no new knob, no new
metric, no introspection schema bump.**

Split `.8b` does the two edits + the lib proofs + the gate confirmation +
book/USER_GUIDE closeout (replace the "wider lane stays inline — a recorded
follow-up" caveat in `book/src/structured-emission.md` with the shipped
wider-lane surface). Pre-split `.8b` further (`.8b.1` live + `.8b.2` gate/docs)
only if it grows beyond a clean single slice.

## 2026-06-17 — Structured emission — pick the FOURTH surface (wider-lane `generate for` part-select) — `STRUCTURED-EMISSION-EXPANSION.7` (decision `0015`)

A design/decision leaf (no source), autonomously selected at the
no-active-frontier boundary (`feedback_pick_and_roll_at_no_frontier`) after the
third structured surface (the combinational `task automatic`, decision `0014`)
closed. It picks the **fourth** structured-emission surface.

**The pick: the wider-lane `generate for` part-select.** This is a
behaviour-preserving *broadening* of the second surface (decision `0013`'s
`generate for` loop) from the 1-bit lane to a lane of any width `LW >= 1`. For a
marked `{N{x}}` replication whose lane `x` is `LW` bits (so the result is `N*LW`
bits) the loop body becomes the indexed part-select
`assign <wire>[<wire>__gi*LW +: LW] = <x>;`. Bit-group `g` of `{N{x}}` — bits
`[g*LW +: LW]` — is exactly the lane, so the unrolled loop stays
byte-equivalent to the inline replication. It closes the wider-lane follow-up
that decision `0013` and `book/src/structured-emission.md` both recorded ("a
wider lane would need a part-select body and stays inline — a recorded
follow-up").

**Why this over the previously-recorded leading candidates (nested `generate`,
`interface`/`modport`).** A fresh empirical tool-acceptance probe this session
(`/tmp/anvil-probe-se4/`; Verilator 5.046 `-Wall --lint-only`, Yosys 0.64
`synth -noabc` and `abc -fast; opt -fast; check`, Icarus `iverilog -g2012`):
- **Wider-lane part-select**: universally warning-clean (the lone Verilator
  `DECLFILENAME` complaint was a filename≠module-name probe artifact, gone with
  `-Wno-DECLFILENAME` / a matching filename) **and** iverilog simulation proved
  the unrolled loop **bit-equal to `{4{b}}`** across sampled inputs. Minimal
  blast radius. **Picked.**
- **`interface`/`modport`**: **empirically disqualified.** Icarus syntax-fails
  the `modport`-typed port (`syntax error … Errors in port declarations`); *both*
  Yosys modes warn `Identifier '\p.data' / '\intf.data' is implicitly declared`.
  This confirms with current tools the weak/version-inconsistent support the
  prior decisions only cited — it would fail the clean-across-every-tool bar.
- **Nested/multi-level `generate`**: clean across all three tools, but a bigger
  emitter change (nested genvar scoping) **and** it lacks a routine
  by-construction source — ANVIL's replications are 1-dimensional; `{N{ {M{x}} }}`
  is not a normal construction. Recorded as a later `generate`-deepening surface.
- **Constant-predicate `generate if`**: clean, but introduces a dead untaken
  branch (unused logic) and the source-level frontend lane already exercises
  `generate if`. Deferred.

**Discipline / why it's cheap (decision `0015`).** Rules-first — broaden the
existing `annotate_generate_loop_gates` predicate; never generate-then-filter.
It **reuses** the existing `generate_loop_emit_prob` knob (default `0.0` ⇒
byte-identical, snapshots untouched) and the `num_emitted_generate_loops` metric
— so **no new knob, no new metric, and no introspection schema bump** (the
fourth surface is the first that needs none of those). No new IR node / no new
computed truth (the emit-projection precedent).

**Planned `.8` implementation shape (for `.8a` to pin against real code).** Two
surgical edits, both confirmed small by the bootstrap codebase walk:
- `src/ir/generate_loop.rs::gate_qualifies` — relax the `lane.width() != 1`
  restriction to `LW >= 1` and the `*width == operands.len()` check to
  `*width == operands.len() * LW`; keep the `function_emit` / `soft_union`
  exclusions.
- `src/emit/sv.rs::generate_loop_gate` + `render_generate_loop_block` — the gate
  helper currently returns `(lane, N)`; the wider lane needs `LW` too (return it,
  or recompute `m.nodes[lane].width()` in the renderer). The renderer branches:
  `LW == 1` keeps the existing `assign <wire>[gi] = <x>;` **byte-identical** (so
  the shipped 1-bit surface, its proofs, and the `.4b` gate stay green
  unchanged); `LW > 1` emits `assign <wire>[gi*LW +: LW] = <x>;`. Do **not**
  collapse `LW==1` into `[gi*1 +: 1]` — that would change the shipped surface's
  bytes.
- Downstream proof: the existing `tool_matrix --generate-loop-gate` covers wider
  lanes the moment the predicate is relaxed; `.8` adds a focused assertion (and,
  if warranted, a wider-lane coverage signal) so the wider lane is *proven
  exercised*, not merely possible.

Split `.7` (this design leaf, done) + `.8` (impl, pre-split `.8a` design-detail +
`.8b` impl) + future `.9+` (nested/multi-level `generate`, `interface`/`modport`,
richer multi-output tasks). Default-off / DUT byte-identical throughout. Nothing
retired.

## 2026-06-16 — Structured emission — combinational `task automatic` live surface — `STRUCTURED-EMISSION-EXPANSION.6b.1`

The third richer-structured emit surface goes live, implementing the `.6a`
design (decision `0014`). It is the decision `0012` single-gate
`function automatic` parallel, but expressed as a **procedural `task automatic`
called from `always_comb`** — a genuinely distinct elaboration surface (a task
writes through an `output` arg; a function returns a value).

**What landed.**
- `Config::task_emit_prob: f64` (default `0.0`, `default_task_emit_prob()` serde
  default; added to the `Default` impl + the `0.0..=1.0` validation list),
  config-file-only (no CLI flag — the `function_emit_prob` /
  `generate_loop_emit_prob` precedent).
- `Module.task_emit_gates: BTreeSet<NodeId>` (Default-empty; emitter-surface
  annotation only — flat IR / validators / CSE / `canonical_module_signature`
  untouched; disjoint from `function_emit_gates` / `generate_loop_gates` /
  `soft_union_slice_gates`).
- New `src/ir/task_emit.rs` `annotate_task_emit_gates(m, rng, prob)` (the
  `function_emit.rs` precedent): the candidate is the **function-emit candidate
  set** (a non-structured, non-`Slice` `Node::Gate` with ≥1 operand) **plus**
  exclusion of any gate already marked for the three sibling projections; rolls
  `gen_bool(prob)` per candidate; `param_env` modules skipped.
- Two guarded gen-time call-site rolls (`generate_module` + `generate_design`)
  run **after** the generate_loop roll, so an already-marked gate is excluded
  (the established "later pass excludes earlier marks" ordering — soft_union →
  function_emit → generate_loop → task_emit).
- `src/emit/sv.rs`: a `task_emit_gate` accessor (defensively re-checks the
  candidate contract, mirrors `function_emit_gate`), a task section (after the
  generate-loop section) emitting per marked gate
  `task automatic <wire>__t(output logic [W-1:0] o, input ...); o = <body>;
  endtask` (`render_gate_task_decl`) + `logic [W-1:0] <wire>__tv; always_comb
  <wire>__t(<wire>__tv, <refs>);` (`render_gate_task_call`), and the gate-assign
  loop rewrites a marked gate's assign to the passthrough `assign <wire> =
  <wire>__tv;`.

**Integration form (resolved at `.6a`).** The **output-var + passthrough-assign**
form: `<wire>` stays a continuous-assign net (minimal downstream change, the
`function_emit` "only the gate's own drive changes" parallel), the task writes a
local `logic <wire>__tv` var, and a continuous `assign` drives `<wire>` from it.
The `<wire>`-as-procedural-var alternative was rejected as the first cut.

**Body reuse.** `render_gate_task_decl` reuses `render_gate_function_body`
verbatim for the body expression — the only difference from `function_emit` is
the procedural `output` arg (`o = <body>;`) vs the function return value
(`<fname> = <body>;`). One body renderer, two projection shapes.

**Why no schema bump here.** The `task_emit_prob` knob rides `request.knobs` via
`#[serde(default)]` (the `function_emit_prob` / `generate_loop_emit_prob`
precedent); the `num_emitted_combinational_tasks` metric — which *will* bump the
introspection schema `1.9 → 1.10` — is the `.6b.2` slice, not this one.

**Verification.** `cargo check --all-targets` clean; `cargo clippy --all-targets
-- -D warnings` clean; `cargo fmt --all --check` clean; `cargo test --lib` 489
passed / 2 ignored (incl. 11 new `task_emit` proofs); `cargo test --test
snapshots` 6/6 byte-identical (default-off). Forced `task_emit_prob=1.0` sweep
(5 seeds 1/7/42/100/2024, 4–39 tasks each, banked `/tmp/anvil-te-r1/`):
Verilator `--lint-only` clean + `-Wall` ON-vs-OFF delta = 0, Yosys without-abc +
with-abc clean, Icarus `iverilog -g2012` clean. (The transient `DECLFILENAME`
during the sweep was a test-harness filename≠module-name artifact, not a task
warning — gone once files are named after their module.)

**Width-1 spacing note.** For a width-1 gate `param_width_decl_w` yields `""`, so
the decl reads `output logic  o` (double space) — identical to the existing
`function automatic logic  <name>__f` width-1 behaviour; accepted by all three
tools. Mirrored deliberately, nothing special-cased.

Default-off / DUT byte-identical. Nothing retired (excluded gates still emit
inline). Frontier → `.6b.2` (the metric + schema `1.9 → 1.10` + the repo-owned
`tool_matrix --task-emit-gate` + `saw_combinational_task_emit`).

---

## 2026-06-16 — Structured emission — combinational `task automatic` impl design-detail — `STRUCTURED-EMISSION-EXPANSION.6a`

Design-detail leaf for `.6` — the third richer-structured surface (decision
`0014`): a default-off, valid-by-construction combinational `task automatic`
emit-projection. Docs-only (no source). The exact parallel of the `.2a`
combinational-`function` design, re-expressed as a procedural `task`. Grounded
in the real emitter + the two sibling gen-time emit-projection passes:

- **`src/emit/sv.rs`** — `to_sv_with_modules` already has the structural
  template: a per-node decl section before the gate `assign`s (the
  `function automatic` block, then the `generate for` block), and the per-gate
  assign loop that `continue`s past a marked gate to suppress its inline assign.
  The task surface adds a third such section + a third `continue` guard. Crucially,
  **`render_gate_function_body(op, operand_widths)`** — the positional body
  renderer the function surface already uses (`o = a0 <op> a1 …` over positional
  params) — is **reused verbatim** as the task body, so no new body renderer is
  needed.
- **`src/ir/function_emit.rs` / `src/ir/generate_loop.rs`** — the gen-time
  annotation precedent (`annotate_*` collects qualifying candidates, rolls
  `gen_bool(prob)`, marks a `Module` `BTreeSet<NodeId>`; the call site in both
  `generate_module` + `generate_design` guards on `prob > 0.0`; `param_env`
  modules skipped; the later pass excludes earlier marks). The task pass is the
  third in this chain.

### Q1 — net-vs-var integration: the output-var + passthrough-`assign` form (minimal blast radius)

A `task` cannot drive a continuous `assign` (it writes through an `output`/`ref`
argument from a procedural context). So a task-emitted gate must be produced in
an `always_comb`. Two forms were both probed clean at `.5`:

- **(leading) output-var + passthrough.** Keep the gate's wire a *net* exactly as
  today (`wire [W-1:0] <wire>;`), add a companion var `logic [W-1:0] <wire>__tv;`,
  emit `always_comb <wire>__t(<wire>__tv, <operand refs>);`, and change the gate's
  own assign to `assign <wire> = <wire>__tv;`. **Only the gate's own drive
  changes** (its RHS becomes `<wire>__tv`), the wire-decl section stays uniform
  (every gate is still a `wire` — zero risk of a net-vs-var mismatch elsewhere),
  and downstream refs to `<wire>` are unchanged. This is the exact
  `function_emit` "only the gate's assign RHS changes" parallel, and is the
  **leading first cut**.
- **(rejected for the first cut) `<wire>`-as-var.** Make `<wire>` itself a `logic`
  var driven by `always_comb <wire>__t(<wire>, …)` and drop the assign. Cleaner
  output (no `__tv`) but it touches the wire-decl section (one node becomes a var,
  not a net), a larger blast radius for no first-cut benefit. Recorded follow-up;
  nothing precludes it.

One `always_comb` **per** task-emitted gate (simplest, deterministic ordering;
a shared `always_comb` is a later optional consolidation). The task decl lives
in module scope alongside the `function automatic` decls.

### Q2 — gen-time annotation (`Module.task_emit_gates`), the third pass in the chain

A new **`src/ir/task_emit.rs`** (registered `pub mod task_emit;` in `src/ir/mod.rs`)
carries `annotate_task_emit_gates(m, rng, prob)`, mirroring
`annotate_function_emit_gates` exactly: skip `param_env` modules, collect
candidates, roll `gen_bool(prob)`, insert into a new
**`Module.task_emit_gates: BTreeSet<NodeId>`** (emitter-surface annotation only).
The candidate predicate is the **same** as `function_emit`'s — an ordinary
combinational `Gate` (not `CaseMux`/`CasezMux`/`ForFold`, not `Slice`, ≥1
operand) — **plus** exclusion of gates already in `function_emit_gates`,
`generate_loop_gates`, and `soft_union_slice_gates` (the three sibling
projections; each gate is projected by at most one). The generator call site
lands in both `generate_module` + `generate_design` **after** the
`generate_loop` roll (the established "later pass excludes earlier marks"
ordering), guarded on `task_emit_prob > 0.0`.

### Q3 — the `task automatic` decl + `always_comb` call rendering

For a marked gate `g = op(o0,o1,…)` of width `W` (id `idx`, wire `names[idx]`):

- **Accessor** (defensive, mirrors `function_emit_gate`): `task_emit_gate(m, idx)
  -> Option<(GateOp, &[NodeId], u32)>` returns `None` unless `idx ∈
  m.task_emit_gates` and the node still satisfies the candidate contract.
- **Declaration** (a new section after the `function automatic` / generate-block
  sections): `task automatic <wire>__t(output logic [W-1:0] o, input logic
  [W0-1:0] a0, …); o = <render_gate_function_body(op, &operand_widths)>; endtask`
  — the body renderer is reused verbatim, with `o` (the output param) as the LHS
  instead of `<wire>__f`.
- **Var decl + call** (a section, or folded into the wire-decl + assign loop):
  `logic [W-1:0] <wire>__tv;` + `always_comb <wire>__t(<wire>__tv, <node_ref(o0)>,
  …);`.
- **Call-site suppression**: in the per-gate assign loop, a marked task gate's
  `assign <wire> = render_gate(...)` becomes `assign <wire> = <wire>__tv;` (or a
  `continue` if the var is the gate wire — but the leading output-var form keeps
  the assign, just changes the RHS). Positional args handle duplicate operands
  (the same `render_gate_function_call`-style `node_ref` per position).
- Naming: `<wire>__t` (task) + `<wire>__tv` (output var) mirror the `__f` / `__u`
  / `__gi` / `__gen` suffix conventions; `build_names` uniqueness carries over.

### Q4 — the `task_emit_prob` knob

A new **`Config::task_emit_prob: f64`** (default `0.0`) beside
`function_emit_prob` / `generate_loop_emit_prob`, with a
`default_task_emit_prob()` serde default, added to the `Default` impl and the
`0.0..=1.0` validation list. **Config-file-only** (no CLI flag — the
`function_emit_prob` / `generate_loop_emit_prob` precedent). Default `0.0` ⇒
`annotate_task_emit_gates` not called (call-site guard) ⇒ no RNG draw, nothing
marked ⇒ **byte-identical** (`tests/snapshots.rs` untouched). Rides
`request.knobs` via `#[serde(default)]` ⇒ **no schema bump for the knob**.

### Q5 — the metric + the downstream gate

- A **`Metrics::num_emitted_combinational_tasks`** (`= m.task_emit_gates.len()`,
  `#[serde(default)]`) surfaced in introspection `module_metrics` ⇒ schema MINOR
  bump `1.9 → 1.10` (the `1.7→1.8` / `1.8→1.9` metric-bump precedent; bump
  `SCHEMA_VERSION` + the schema_version assertions + schema doc +
  README/USER_GUIDE/book current-output refs + the CODEBASE_ANALYSIS envelope).
- A repo-owned **`tool_matrix --task-emit-gate`** + `ScenarioSet::TaskEmitSweep` +
  `build_task_emit_sweep_scenarios` (a comb-only DUT forcing `task_emit_prob = 1.0`
  × three construction strategies, shaped like `function_emit_focus_config`) +
  `ModuleReport.emitted_combinational_task` (from
  `prepared.sv_text.contains("task automatic")`) + `CoverageSummary.saw_combinational_task_emit`
  (lit on Verilator success AND clean Yosys — a combinational task is universally
  synthesizable, like a function, so the full tool plan runs) + merge +
  early-return gap arm + 5 proofs + the ModuleReport fixture updates. Bank clean
  `/tmp/anvil-task-emit-gate-r1`.

### `.6b` impl shape (the implementation slice — likely pre-split)

`.6b` mirrors `.4b` and should pre-split: `.6b.1` (live surface: the knob +
`Module.task_emit_gates` + `src/ir/task_emit.rs` + the two call-site rolls + the
`to_sv_with_modules` task decl/var/always_comb/assign rendering + lib proofs +
a forced-knob Verilator/Yosys/Icarus spot-check) + `.6b.2` (the metric + schema
`1.9→1.10` and the `tool_matrix --task-emit-gate` gate; may further split
`.6b.2a`/`.6b.2b`) + `.6b.3` (book/knobs/USER_GUIDE/README/KM closeout).
Default-off / DUT byte-identical throughout. **Rejected** (carried from `.5` /
decision `0014`): the `<wire>`-as-var integration in the first cut, a
multi-output/side-effecting task, a multi-gate-cone task body, a new IR `Task`
node, and changing the default.

## 2026-06-16 — Structured emission — `generate for` loop live surface — `STRUCTURED-EMISSION-EXPANSION.4b.1`

The second richer-structured emit surface (decision `0013`) goes live, exactly
per the `.4a` design-detail below. Implemented as a single live-surface slice;
`.4b` pre-split into `.4b.1` (this — knob + annotation + Module field + emitter
rendering + lib proofs + forced sweep), `.4b.2` (the repo-owned `tool_matrix
--generate-loop-gate` + the `num_emitted_generate_loops` metric + the
`saw_generate_loop_emit` coverage fact + schema `1.8→1.9`), `.4b.3`
(book/USER_GUIDE/README/KM closeout).

### What landed (the live surface)

- `Config::generate_loop_emit_prob` (default `0.0`, `#[serde(default =
  "default_generate_loop_emit_prob")]`) + `Module.generate_loop_gates:
  BTreeSet<NodeId>` (default empty) — both beside the `function_emit` /
  `soft_union` / `aggregate` analogues.
- `src/ir/generate_loop.rs::annotate_generate_loop_gates(m, rng, prob)` — the
  gen-time mark, rolled at the call site in **both** `generate_module` and
  `generate_design`, guarded on `prob > 0.0`, **after** the function-emit pass so
  an already function-emit-marked replication is excluded (the two
  emit-projections are mutually exclusive on a gate). Candidate = a
  `GateOp::Concat` of the `{N{x}}` form (`≥ 2` operands all the same `NodeId`)
  with a **1-bit lane**. `param_env` modules skipped.
- `src/emit/sv.rs` — a `generate for` block section (after the function-decl
  section, before the gate assigns) emitting per marked gate `genvar <wire>__gi;
  generate for (…) begin : <wire>__gen assign <wire>[<gi>] = <x>; end
  endgenerate`, and the per-gate assign loop `continue`s past a marked gate so
  the inline `assign <wire> = {N{x}};` is suppressed. Helpers: `generate_loop_gate`
  (marked + defensively-revalidated lookup, returns `(lane, N)`) +
  `render_generate_loop_block`.

### The `gi = gi + 1` increment (vs the decision record's `gi++`)

Decision `0013`'s rendered example used `gi++`; `.4a` recorded `gi = gi + 1` as
the "maximally-portable fallback." I implemented the **`gi = gi + 1`** form: it
is the most conservative increment (no dependence on the `++` operator, valid in
every Verilog/SV standard), and the forced sweep verifies it clean across all
four tools — so there is no reason to take on `gi++`'s (smaller) portability
surface for a first cut. Not a behaviour difference; the unrolled loop is
identical. `gi++` stays available if ever wanted; nothing is retired.

### Why the 1-bit-lane restriction (and why it is not narrow in practice)

A 1-bit lane makes `assign <wire>[gi] = <x>;` byte-faithful to `{N{x}}` (result
width `== N`, bit `g` == the lane). A wider lane (`{N{byte}}`) would need a
part-select body (`<wire>[gi*LW +: LW]`) — verified clean too, but more emitter
surgery; recorded as a follow-up (the wider replication still emits inline,
nothing retired). It is *not* a rare case: a `{W{sel}}` 1-bit broadcast is the
common ANVIL one-hot mux-mask idiom (`render_gate`'s own `Concat` comment calls
it out), so the surface fires readily — a forced `generate_loop_emit_prob=1.0`
default-config probe lit a `generate for` on **27/30** seeds.

### Validation

`cargo test --lib` 477 (468 + 9 new `generate_loop` proofs) / 2 ignored; `cargo
test --test snapshots` 6/6 byte-identical (default-off; the umbrella
DUT-byte-identical proofs still green); clippy `-D warnings` + fmt clean. Forced
`generate_loop_emit_prob=1.0` sweep (5 seeds, `/tmp/anvil-gl-r1/`, 62–168 loops
each): Verilator `--lint-only` **5/5 rc=0 / 0 warnings**, **`-Wall` ON-vs-OFF
delta = 0** (the change adds no new warnings; the residual `-Wall UNUSEDSIGNAL`
is pre-existing, identical ON and OFF), Yosys without-abc **5/5**, with-abc
**5/5**, Icarus `iverilog -g2012` **5/5**. No introspection schema bump (the
default-off prob-knob rides `request.knobs` via `#[serde(default)]`, the
`.2b.1` precedent; the `.4b.2` metric will bump `1.8→1.9`).

## 2026-06-16 — Structured emission — `generate for` loop impl design-detail — `STRUCTURED-EMISSION-EXPANSION.4a`

Design-detail leaf for `.4` — the second richer-structured surface (decision
`0013`): a default-off, valid-by-construction **`generate for` loop**
emit-projection of an existing `{N{x}}` replication. Docs-only (no source).
Grounded in a fresh read of the real emitter + the two default-off
emit-projection precedents (the same two `.2a` grounded the function surface
in):

- **`src/emit/sv.rs`** — `to_sv_with_modules`. The `{N{x}}` replication is
  *already* recognised today inside `render_gate` (the `Concat` arm,
  `src/emit/sv.rs:1159`): when a `GateOp::Concat` gate has `operands.len() >= 2`
  **and** every operand is the **same `NodeId`** (`operands.iter().all(|id| *id ==
  operands[0])`), it renders the canonical replication form
  `format!("{{{}{{{}}}}}", operands.len(), a(0))` ⇒ `{N{<x>}}` (e.g. the
  `{11{or_0}}` one-hot-mask broadcast its own comment calls out). That exact
  predicate **is** the index-regular source the loop projects. Helpers reused:
  `build_names(m)` (node id → wire name), `node_ref(id, m, &names)` (operand
  reference), `param_width_decl_w(m, w)`; the function-emit section (a per-node
  decl block before the gate `assign`s) is the structural template for a new
  generate-block section.
- **`src/ir/function_emit.rs`** + **`src/ir/soft_union.rs`** — the
  gen-time-annotation precedent (`.2b.1` / `SV-VERSION-TARGETING.3b.2`):
  `annotate_*` collects qualifying candidates, rolls `rng.gen_bool(prob)` per
  candidate (seeded; never `thread_rng`), inserts marks into a
  `Module` `BTreeSet<NodeId>` that is an **emitter-surface annotation only** (flat
  IR body / validators / CSE / `canonical_module_signature` untouched); the
  call site in **both** `generate_module` and `generate_design`
  (`src/gen/mod.rs`) guards on `prob > 0.0`, so the default draws nothing ⇒
  byte-identical stream + output; the emitter consults the mark per node and
  re-checks the qualifying contract defensively (`function_emit_gate` /
  `soft_union_slice_overlay`). This is the exact mechanism the generate-loop
  surface mirrors.

### Q1 — selection rule: a `{N{x}}` replication `Concat` with **1-bit lanes** (the minimal faithful loop)

The candidate is the *same node the emitter already replication-renders*, narrowed
to the cleanest first cut:

- **Candidate rule (rules-first, mirrors `render_gate`'s replication predicate):**
  a `Node::Gate` with `op == GateOp::Concat`, `operands.len() >= 2`, and
  `operands.iter().all(|id| *id == operands[0])` (an N-fold replication of one
  operand) — **and**, for the first cut, the replicated operand (the *lane*) is
  **1 bit wide** (`m.nodes[operands[0]].width() == 1`). With a 1-bit lane the
  result width `W == N == operands.len()`, and bit `g` of the result is exactly
  the lane `x`, so the loop body `assign <wire>[gi] = <x>;` is **byte-faithful**
  to `{N{x}}`. This is the common ANVIL idiom (`{W{sel_i}}` one-hot broadcasts),
  so the surface is not rare.
- **Why 1-bit lanes first.** A *wider* lane (`LW > 1`) is still index-regular but
  needs a part-select body (`assign <wire>[gi*LW +: LW] = <x>;`) and genvar
  arithmetic — verified clean too, but more emitter surgery for the same first-cut
  value. Restricting to `LW == 1` keeps the first cut the *minimal faithful loop*
  (the single-gate-function parallel from `.2a`). **Wider-lane part-select is a
  recorded follow-up; nothing is retired** — a wider replication still emits the
  inline `{N{x}}`.
- **Mutual exclusion.** A replication `Concat` is *also* a function-emit candidate
  (`function_emit` excludes only `CaseMux`/`CasezMux`/`ForFold`/`Slice`, not
  `Concat`). The two emit-projections must be disjoint on a gate. Resolution
  (the established "later pass excludes earlier marks" ordering — `function_emit`
  runs after `soft_union` and excludes its marks): run the generate-loop
  annotation **after** `function_emit` and have its candidate predicate **exclude
  gates already in `m.function_emit_gates`** (and, defensively, `soft_union_slice_gates`
  — moot since those are only `Slice`s). One-directional exclusion suffices because
  `function_emit` has already run. The downstream gate sets only
  `generate_loop_emit_prob = 1.0` (with `function_emit_prob = 0.0`), so the loops
  fire uncontested there.

### Q2 — gen-time annotation (`Module.generate_loop_gates`), not an emit-time pass

A new **`src/ir/generate_loop.rs`** (registered `pub mod generate_loop;` in
`src/ir/mod.rs` beside `function_emit` / `soft_union`) carries

```rust
pub fn annotate_generate_loop_gates(m: &mut Module, rng: &mut impl Rng, prob: f64) -> usize
```

mirroring `annotate_function_emit_gates`: skip `m.param_env.is_some()` modules
(symbolic widths out of scope, like the other passes), collect candidate ids
(the Q1 predicate), roll `gen_bool(prob)` per candidate, insert into a new
**`Module.generate_loop_gates: BTreeSet<NodeId>`** (default empty, `#[serde]`
default; an emitter-surface annotation only). The generator call site lands in
**both** `generate_module` and `generate_design` (beside the `soft_union` /
`function_emit` rolls), **after** the function-emit roll, guarded on
`self.cfg.generate_loop_emit_prob > 0.0` so the default path draws nothing from
the RNG ⇒ byte-identical. Rationale for gen-time (not emit-time): matches the
established precedent exactly, keeps the roll seeded/reproducible at the
generation boundary, and leaves the emitter a pure projection of the mark.

### Q3 — the `genvar` / `generate for` rendering + inline-assign suppression

For a marked replication gate `g = {N{x}}` (id `idx`, wire `names[idx]`, lane
operand `o0 = operands[0]`, `N = operands.len()`):

- **Accessor** (defensive, mirrors `function_emit_gate`):
  `generate_loop_gate(m, idx) -> Option<(NodeId /*lane operand*/, usize /*N*/)>`
  returns `None` unless `idx ∈ m.generate_loop_gates` **and** the node still
  satisfies the Q1 replication-with-1-bit-lane contract — a stale marker can
  never produce an invalid loop.
- **Generate-block section** (a new section emitted right after the function-decl
  section, before the gate-`assign` loop — module items are order-independent, so
  placement is cosmetic; the driven `<wire>` is already declared in the wire-decl
  section above):

  ```systemverilog
  genvar <wire>__gi;
  generate
      for (<wire>__gi = 0; <wire>__gi < N; <wire>__gi++) begin : <wire>__gen
          assign <wire>[<wire>__gi] = <x>;
      end
  endgenerate
  ```

  where `<wire> = names[idx]`, `<x> = node_ref(o0, m, &names)`, and `N` is a
  literal. `build_names` guarantees unique gate wire names ⇒ the `<wire>__gi`
  genvar and `<wire>__gen` loop label are unique too (no genvar redeclaration
  across multiple loops; the `__gi`/`__gen` suffixes mirror the `__f` / `__u`
  conventions). The `gi++` increment is the form the `.3` empirical probe verified
  clean across all four tools; `gi = gi + 1` is the maximally-portable fallback if
  any future tool objects (pin at `.4b`).
- **Inline-assign suppression.** In the per-`Node::Gate` assign loop, add
  `if generate_loop_gate(m, idx).is_some() { continue; }` **first** (defensive
  precedence over the `soft_union` / `function_emit` arms, though all three are
  disjoint by the Q1 annotation rule). Without it the gate would fall to the
  `render_gate` `Concat` arm and emit the inline `assign <wire> = {N{x}};`; the
  `continue` hands the drive of `<wire>` to the generate block. **Behaviour-
  preserving by construction**: the unrolled loop is exactly `{N{x}}`.

### Q4 — the `generate_loop_emit_prob` knob

A new **`Config::generate_loop_emit_prob: f64`** (default `0.0`) beside
`function_emit_prob` / `soft_union_slice_prob` / `aggregate_prob`, with a
`default_generate_loop_emit_prob()` serde default, added to the `Default` impl and
the `0.0..=1.0` validation list (`src/config.rs:~1363`). It is a
**config-file-only knob — no `--generate-loop-emit-prob` CLI flag** (the
`function_emit_prob` / `soft_union_slice_prob` precedent; set it via `--config`
JSON). Default `0.0` ⇒ `annotate_generate_loop_gates` is not called (call-site
guard) ⇒ no RNG draw, nothing marked, `generate_loop_gates` empty ⇒
**byte-identical** (`tests/snapshots.rs` untouched). It surfaces in
`--dump-config` / `--introspect` automatically (a `Config` field rides
`request.knobs` via `#[serde(default)]`) ⇒ **no introspection schema bump for the
knob** (the `.2b.1` `function_emit_prob` precedent). A `num_emitted_generate_loops`
metric (`= m.generate_loop_gates.len()`, in the `.4b` gate/closeout) *would* bump
the schema MINOR `1.8 → 1.9` (the `.2b.2a` `num_emitted_combinational_functions`
precedent).

### Q5 — the downstream gate (`saw_generate_loop_emit`)

A repo-owned `tool_matrix --generate-loop-gate`, templated on
`--function-emit-gate` (`.2b.2b`): a `ScenarioSet::GenerateLoopSweep` +
`build_generate_loop_sweep_scenarios` forcing `generate_loop_emit_prob = 1.0` over
a **comb-only single-module DUT** across all three construction strategies, a
`ModuleReport.emitted_generate_loop` SV-text detection (`#[serde(default)]`, from
`prepared.sv_text.contains("generate")` / `"genvar"` — the
`emitted_combinational_function` / `emitted_soft_union_overlay` precedent), a
`CoverageSummary.saw_generate_loop_emit` fact lit when an emitted-loop module is
accepted by Verilator success **and** a non-empty clean Yosys vec (a `generate for`
is universally synthesizable — like the function, *unlike* the Verilator-only
`union soft` up-opt — so the gate runs the full Verilator + both Yosys (+ Icarus)
plan), merged in `merge_coverage`, and an early-return arm in
`compute_coverage_gaps` so no broad-motif richness leaks in. Bank clean at
`/tmp/anvil-generate-loop-gate-r1` (the `.2b.2b` `/tmp/anvil-function-emit-gate-r1`
parallel).

- **Load-bearing gate-shape risk (flag for `.4b`):** the DUT corpus must actually
  *produce* `{N{x}}` 1-bit-lane replications for the loop to fire — these come from
  the one-hot mux-mask broadcast idiom (`{W{sel_i}} & data_i`, per the `render_gate`
  comment). The forced sweep config must therefore select a construction shape that
  emits those broadcasts (the share-heavy comb config / the mux-encoding path).
  `.4b` must confirm the banked report shows `saw_generate_loop_emit = true` /
  `emitted_generate_loop` on the modules and broaden the config (or add a
  dedicated replication-rich scenario) if a chosen seed/strategy yields no
  replications — the forced-sweep banked evidence is the proof, exactly as for
  `.2b.1`'s `/tmp/anvil-fe-r2/`.

### `.4b` impl shape (the single implementation slice)

`Config::generate_loop_emit_prob` + `default_generate_loop_emit_prob()` + Default +
validation; `Module.generate_loop_gates: BTreeSet<NodeId>`; new
`src/ir/generate_loop.rs` (`annotate_generate_loop_gates` + the candidate predicate +
lib proofs: marks a 1-bit-lane replication / skips a wider-lane replication / skips a
non-replication Concat / skips a function-emit-marked gate / `param_env` skip /
identity-and-node-count-untouched / end-to-end emit + default-off byte-identical);
the two `src/gen/mod.rs` call-site rolls (after function-emit); the `src/emit/sv.rs`
`generate_loop_gate` accessor + `render_generate_loop_block` + the generate-block
section + the assign-loop `continue`; a forced `generate_loop_emit_prob=1.0`
Verilator `--lint-only` + both-Yosys + Icarus spot-check; then the `.4b` gate (or a
pre-split `.4b.1` live surface + `.4b.2` gate/metric + `.4b.3` book/USER_GUIDE/KM
closeout if the slice is too broad for one signoff-quality commit — decide at pick
time, the `.2b` precedent). Default-off / DUT byte-identical throughout
(`tests/snapshots.rs` untouched). **Rejected** (carried from `.2a` / decision
`0013`): wider-lane part-select in the first cut, a pure emit-time pass, a new IR
`Generate` node, and changing the default.

---

## 2026-06-16 — Structured emission — picking the second surface (`generate for`) — `STRUCTURED-EMISSION-EXPANSION.3` (decision 0013)

The owner steered the lane to its next surface (*"structured emission: next
surface"* → `generate`). Two design points are worth recording beyond the
decision record.

**Why `generate for` and not `generate if` for the DUT lane.** ANVIL resolves
*every* structural choice at construction time (seed-deterministic). So a
`generate if` in the DUT lane would always have a **constant** predicate, and
the elaborator would discard the untaken branch — the construct degenerates to
"emit the taken branch with extra dead syntax around it." That is low
DUT-stress value (the frontend lane already exercises `generate if` precisely
because *there* the predicate is a real elaboration-time parameter expression).
A `generate for` over a replication produces genuine repeated structure the
elaborator must unroll — a real new elaboration surface. So the first generate
cut is a loop, not a conditional.

**The valid-by-construction source is the subtle part.** A faithful
`generate for` needs N items that are **index-regular** — bit/lane `g` is a
function of the genvar. ANVIL's emitted structure is mostly *not* index-regular:
replicated child instances (hierarchy) have per-instance irregular connections;
wide ops emit as a single `assign`. The one clean, already-present
index-regular source is a **replication** `{N{x}}` (`GateOp::Concat` of one
operand) — bit `g` is exactly `x` (or lane `g` of `x`). That makes it the
function-emit analog: wrap an *existing, already-valid* node, behaviour-preserving
by construction, default-off byte-identical. Pinning whether to also cover a
`Concat` of N identical lanes, or to *construct* index-regular replicated
instances rules-first, is the `.4a` open question — but the leading first cut is
the pure `{N{x}}` projection, to stay strictly inside the emit-projection family
(no new whole-module behaviour).

**Evidence correction for `task`.** Decision `0012` deferred `task` citing weak
Yosys synth support. An empirical probe this session (Verilator 5.046 `-Wall` +
Yosys 0.64 both modes + Icarus `iverilog -g2012`) shows a **simple combinational
void `task`** — an `always_comb`-called `task automatic` with a single
`output`/`ref` — is accepted **clean** by all four tools. So the real caution is
narrower: *multi-output / side-effecting / multi-statement* tasks are the risky
ones, not combinational void tasks. `task` is therefore recorded as the
**leading future** surface (`.5+`), not a weak also-ran — nothing is retired.

## 2026-06-16 — Structured emission — user-docs closeout + a config-overlay gotcha — `STRUCTURED-EMISSION-EXPANSION.2b.2c`

The `.2b.2c` closeout is docs-only, but three choices are worth recording.

**Placement.** The combinational `function automatic` surface got a *dedicated*
"How It Works" chapter (`book/src/structured-emission.md`) rather than only a
knob entry, because it is the **first** of a family of structured-emission
surfaces (`task` / nested `generate` / `interface`/`modport` are `.3+`): the
chapter gives that family a permanent home, and future surfaces extend it
instead of scattering. The `function_emit_prob` *knob* still also lands in the
canonical knob reference (`book/src/knobs.md`), `USER_GUIDE.md`, and the README
"Current CLI truth" — next to its `soft_union_slice_prob`/`aggregate_prob`
emit-projection siblings — so a reader looking up knobs finds it where the
others live.

**Accuracy correction.** The resume pointer and earlier tree notes referred
loosely to an "`anvil --function-emit-prob`" knob entry. There is **no such CLI
flag**: `function_emit_prob` is a config-file-only knob (serde `#[serde(default
= "default_function_emit_prob")]`, no `--function-emit-prob` in `src/main.rs`),
exactly like `soft_union_slice_prob` and `aggregate_prob`. The docs were written
to the verified reality (set it via `--config` JSON), not the loose phrasing —
signoff docs must match code, not memory.

**Gotcha (non-obvious; pre-existing behaviour, surfaced while writing the repro
block).** A *minimal* `--config` JSON does **not** behave like a knob overlay on
the effective defaults. `--config '{"seed":42}'` (everything else left to serde
struct defaults) emits **nothing** — the serde `Default` for structural fields
like the width/input/output bounds is `0`, which is a different, degenerate
config than the one `--dump-config` prints (the latter is the *effective* config
after the builder fills real defaults). So the reliable way to flip one knob is
`anvil --seed N --dump-config > base.json` → edit the field → `anvil --seed N
--config base.json`. This is why the book's repro block uses that round-trip (and
why it is skip-sentinelled — it edits a config file, so it is not a one-line
generator example the `book_examples` harness can run unattended). A future
"partial-config overlay" mode (merge onto effective defaults) would remove this
sharp edge, but is out of scope here.

## 2026-06-16 — Structured emission — repo-owned `function automatic` emit gate — `STRUCTURED-EMISSION-EXPANSION.2b.2b`

The repo-owned `tool_matrix --function-emit-gate` proves the combinational
`function automatic` surface (decision `0012`) is downstream-accepted, not just
that the knob can be set. It is templated on `--signoff-knob-sweep-gate` for the
gate scaffolding (`ScenarioSet::FunctionEmitSweep` + `build_function_emit_sweep_scenarios`
+ run-plan + early-return `compute_coverage_gaps` arm) and on the `union soft`
up-opt for emitted-construct detection (`ModuleReport.emitted_combinational_function`).

Two design points worth recording for the next session:

- **The fact requires Verilator AND Yosys clean — not Verilator-only.** The
  `union soft` up-opt fact (`saw_sv_version_2023_soft_union_upopt`) is
  Verilator-only because Yosys/Icarus *reject* the `union soft` syntax (a
  recorded no-op, decision `0010`). A `function automatic` is the opposite: a
  synthesizable function is accepted by *every* tool. So
  `saw_combinational_function_emit` is the stronger fact — it requires the
  emitted-function module to be accepted by Verilator success **and** a
  non-empty, clean Yosys vec. The gate therefore runs the *full* tool plan
  (`verilator_only = false`), and Icarus acceptance (when `--iverilog-compile`
  is set) rides the existing `ToolSummary::any_failed` bail rather than a
  dedicated coverage fact — keeping the gate honest whether or not Icarus is
  enabled.

- **Detection is from the emitted SV text, not the metric.** Both were
  available — `num_emitted_combinational_functions > 0` (the `.2b.2a` metric) or
  an SV-text scan. The SV-text `contains("function automatic")` mirrors the
  `union soft` precedent exactly and proves the construct is in the *file the
  tools actually checked*, which is the honest signal for an acceptance gate
  (the metric counts marked gates; the text proves emission). They agree by
  construction here, but the text is the load-bearing evidence.

Calibration: the focus config is `share_heavy_comb_only_config`-shaped
(node-id + e-graph, rich combinational cone, `flop_prob = 0.0`) with
`function_emit_prob = 1.0`. With `SIGNOFF_KNOB_SWEEP`-style 4 units/scenario ×
3 strategies = 12 modules, the live bank emitted **608** functions —
abundantly non-vacuous (the `.2b.1` forced sweep saw 830–1299/module on the
fatter default shape). Banked clean `/tmp/anvil-function-emit-gate-r1`
(`coverage_gaps = []`, `12/0` Verilator + both Yosys + Icarus compile). No
schema bump (a harness-only change); default `function_emit_prob = 0.0`
emission byte-identical (snapshots 6/6).

## 2026-06-16 — Structured emission — the emit metric + schema `1.7 → 1.8` — `STRUCTURED-EMISSION-EXPANSION.2b.2a`

`Metrics::num_emitted_combinational_functions` (`= m.function_emit_gates.len()`,
computed in `metrics::compute()`) makes the function-emit surface
introspection-queryable. `Metrics` is the exact serde projection surfaced in
introspection `module_metrics`, so adding the field bumps the introspection
schema MINOR `1.7 → 1.8`.

This resolves the schema question the `.2b.1` entry below flagged ("`.2b.2`
reconfirms"). The split is principled and policy-honest (§7):

- The **knob** `function_emit_prob` (added at `.2b.1`) rode the existing schema
  via `#[serde(default)]` in `request.knobs` — the default-off probability-knob
  precedent (`soft_union_slice_prob` / `aggregate_prob` / `memory_prob` /
  `fsm_prob` / `multi_clock_prob` all did the same; only the `sv_version` *enum*
  took a dedicated `1.1 → 1.2` bump).
- The **metric** `num_emitted_combinational_functions` (added here) is a new
  derived `Metrics` field surfaced in `module_metrics`, which **is** a MINOR
  bump — the `1.0 → 1.1` `bisimulation_flops_merged` and `1.3 → 1.4`
  `DesignMetrics` precedents.

So a single feature legitimately split its introspection surfacing across two
slices: the knob rode the version, the metric bumped it. Gotcha for the next
session: a schema bump touches more than the const — the 9 `schema_version`
assertions in `src/introspect/mod.rs` + `src/mcp/mod.rs`, the schema doc
changelog/§7 lines, and every **current-output** doc reference (README /
USER_GUIDE / the 5 `book/src/agent-mcp.md` example JSONs) — but **not** the
historical "landed at schema X" attributions (the ROADMAP semantic-introspection
lane line, the `1.6 → 1.7` book prose). `CODEBASE_ANALYSIS.md`'s envelope line
had drifted (frozen at `"1.4"`); corrected to `"1.8"` with the full chain here.

## 2026-06-16 — Structured emission — combinational `function automatic` live surface — `STRUCTURED-EMISSION-EXPANSION.2b.1`

The `.2a` design (above) implemented as a real, default-off emitter change. The
mechanism is exactly the `soft_union.rs` precedent; two findings during
implementation are worth recording.

### What landed (the live surface)

- `Config::function_emit_prob` (default `0.0`, `#[serde(default)]`) +
  `Module.function_emit_gates: BTreeSet<NodeId>` (default empty) — both beside
  their `soft_union` siblings.
- `src/ir/function_emit.rs::annotate_function_emit_gates(m, rng, prob)` — the
  gen-time mark, rolled at the call site in **both** `generate_module` and
  `generate_design`, guarded on `prob > 0.0`, **after** the soft_union pass so the
  `union soft` marks are visible and excluded.
- `src/emit/sv.rs` — a `function automatic` decl section (after the wire decls,
  before the gate assigns) + a call-site substitution in the gate-assign loop.
  `render_gate_function_body` is the **positional** counterpart of `render_gate`
  (param `a{i}` in place of `node_ref(operands[i])`), so duplicate operands (e.g.
  `xor(n, n)`) get distinct params with no aliasing, and the function returns
  exactly the gate's value — behaviour-preserving by construction.

### Gotcha — `Slice` is excluded from the first-cut candidate set (`-Wall UNUSEDSIGNAL`)

The `.2a` candidate rule said "any non-structured `Gate` with ≥1 operand". A forced
`function_emit_prob=1.0` sweep then showed `verilator -Wall` flagging
`UNUSEDSIGNAL` on every `slice_*__f` function — and **only** slice functions.
Root cause: `Slice {hi, lo}` reads only bits `[hi:lo]` of its operand, so a
function taking the **full-width** source as `a0` and returning `a0[hi:lo]` leaves
the remaining bits of `a0` unread → an unused-variable warning. Every other op
(`And`/`Or`/`Xor`/`Add`/`Sub`/`Mul`/`Not`/comparisons/`Mux`/`Concat`/reductions/
shifts) consumes its operands **in full** (ANVIL's IR is width-consistent: operand
width == result width for the arithmetic/logic ops; `Concat` sums; `Mux` sel is
1-bit; shift amounts are used whole), so they are warning-clean. Decision:
**exclude `Slice` from function-emit candidacy** in the first cut (in both the
annotate predicate and the emitter's defensive `function_emit_gate` re-check).
`Slice` still emits inline — **nothing is retired** (`feedback_never_retire_strategies`);
a slice-aware projection that passes only the used sub-range `src[hi:lo]` (so the
function param is exactly the slice width) is a recorded follow-up. The
`render_gate_function_body` `Slice` arm is kept (correct) for that future cut even
though it is currently unreachable via candidacy.

Note the repo's downstream bar is `verilator --lint-only` (not `-Wall`):
`UNUSEDSIGNAL` is a `-Wall`-only warning and is **pre-existing** in ANVIL's random
RTL (the function-emit-OFF baseline carries ~20 of them — gates with naturally
dead output bits). The slice fix is about not letting the function projection
*add* such warnings; at the repo bar the forced sweep is fully clean across
Verilator + both Yosys modes + Icarus.

### Why no introspection `schema_version` bump

`function_emit_prob` surfaces in `request.knobs` automatically (the introspection
doc embeds `cfg.clone()`), so it also changes `run_id` (a content hash over
`serde_json::to_string(&Config)`). That is **not** a schema bump here: it follows
the established default-off probability-knob precedent —
`soft_union_slice_prob` / `aggregate_prob` / `aggregate_array_prob` /
`memory_prob` / `fsm_prob` / `multi_clock_prob` were all added under the existing
`schema_version` via `#[serde(default)]` (the schema doc §6.1 "exact serde
projection of Config; new knobs carry `#[serde(default)]`, which keeps the schema
additive"). Only `Config::sv_version` (a new emission-capability *enum*) took a
dedicated `1.1 → 1.2` bump. A `1.7` consumer ignores the new key; an absent key
reads back as `0.0`. The introspect `schema_version` lib tests stay green at
`1.7`. `.2b.2` reconfirms at closeout.

## 2026-06-16 — Structured emission — combinational `function automatic` impl design-detail — `STRUCTURED-EMISSION-EXPANSION.2a`

Design-detail leaf for `.2` — the first richer-structured surface (decision
`0012`): a default-off, valid-by-construction combinational `function automatic`
emit-projection. Docs-only (no source). Grounded in a fresh read of the real
emitter + the two default-off emit-projection precedents:

- **`src/emit/sv.rs`** — `to_sv_with_modules` emits, per `Node::Gate`, `assign
  <names[idx]> = <render_gate(op, operands, m, &names)>;` (the gate-emission loop),
  with the structured-gate (`CaseMux`/`CasezMux`/`ForFold`) and `soft_union`
  overlay special-cases consulted first. Helpers: `build_names(m) ->
  Vec<Option<String>>` (node id → wire name), `node_ref(id, m, &names)` (an operand
  reference — a wire name, or a literal for a `Constant`), `render_gate(op,
  operands, m, &names)` (the RHS expression), `param_width_decl_w(m, w)` (a
  `[W-1:0]` width decl).
- **`src/ir/soft_union.rs`** + `Module.soft_union_slice_gates: BTreeSet<NodeId>`
  — the gen-time-annotation precedent: `annotate_soft_union_slices(m, rng, prob)`
  collects qualifying candidates, rolls `rng.gen_bool(prob)` per candidate
  (seeded; never `thread_rng`), inserts marks into a `Module` `BTreeSet<NodeId>`
  that is "an emitter-surface annotation only — the flat IR body, validators, CSE
  keys and `canonical_module_signature` are all untouched"; the call site guards on
  `prob > 0.0` so the default (`0.0`) draws nothing ⇒ byte-identical stream +
  output. The emitter consults the mark per node. This is the exact mechanism the
  function-emit surface mirrors.

### Q1 — first-cut cone selection: a single-gate "operand function" (the minimal cone)

Decision `0012` describes the general shape (wrap a combinational cone, params =
its support leaves). The **first concrete cut** is the *minimal* cone: wrap **one
selected `Node::Gate`** as a `function automatic` of its **direct operands**. The
operands are already module-scope wires (or literals) — every `NodeId` operand has
a `build_names` wire (or `node_ref` literal) — so the call passes existing values
and **nothing else in the module moves**. This sidesteps the sharing/scoping
hazard of a multi-level cone (a cone-internal gate that is *also* referenced
outside the cone cannot be moved into a function body without breaking the external
reference); the single-gate form has **zero** such hazard because only the root's
own `assign` RHS changes. A single-op function is still a genuine new structural
surface (the tool must parse / elaborate / inline a `function automatic` decl +
call). The richer **multi-gate-cone body** (private-internal gates as function
locals, support leaves as params, dropping the private internals' module-scope
assigns) is a recorded **follow-up leaf** once the basic surface is proven
downstream-clean — it is the harder, sharing-aware version and stays out of the
first cut.

- **Candidate rule (rules-first):** a `Node::Gate` whose `op` is **not** a
  structured gate (`CaseMux`/`CasezMux`/`ForFold` — those already have their own
  procedural rendering) and is **not** already marked for the `soft_union` overlay
  (disjoint from the existing per-node special-cases), and whose operand count is
  `>= 1`. (Optionally restrict to `>= 2` operands so the function is non-trivial —
  a `.2b` calibration knob; start permissive.) Selection is at construction time
  (a gen-time annotation pass), never generate-then-filter.

### Q2 — gen-time annotation (the `soft_union.rs` precedent), not an emit-time pass

A new `src/ir/function_emit.rs` (or `src/emit/function_emit.rs`) carries
`annotate_function_emit_gates(m: &mut Module, rng: &mut impl Rng, prob: f64) ->
usize`, mirroring `annotate_soft_union_slices`: collect candidate gate ids, roll
`gen_bool(prob)` per candidate, insert into a new `Module.function_emit_gates:
BTreeSet<NodeId>` (an **emitter-surface annotation only** — flat IR body /
validators / CSE / `canonical_module_signature` untouched, default empty). The
generator call site (beside the `soft_union` / `aggregate` rolls) guards on
`function_emit_prob > 0.0`, so the default draws nothing from the RNG ⇒
byte-identical stream + output. Rationale for gen-time (not pure emit-time): it
matches the established precedent exactly, keeps the roll seeded/reproducible at
the generation boundary, and leaves the emitter a pure projection of the mark.

### Q3 — the `function automatic` signature + body rendering

For a marked gate `g = op(o0, o1, …)` of width `W` (id `idx`, wire `names[idx]`):

- **Declaration** (emitted near the top of the module body, before the gate
  `assign`s — a new "function declarations" section, like the aggregate typedef
  block): `function automatic logic [W-1:0] <names[idx]>__f(input logic [W0-1:0]
  a0, input logic [W1-1:0] a1, …); <names[idx]>__f = <body>; endfunction`, where
  `Wi = m.nodes[oi].width()` and the params are **positional** (`a0, a1, …`), one
  per operand **position** (so a duplicated operand, e.g. `xor(n, n)`, gets two
  distinct params bound to the same wire at the call — no aliasing ambiguity).
- **Body**: `op` applied to the **positional param names**, produced by a
  `render_gate`-parallel routine that renders the same RHS but substitutes `ai` for
  each operand position instead of `node_ref(oi)`. (Implementation note for `.2b`:
  either a small `render_gate_with_operand_names(op, &["a0","a1",…], …)` variant, or
  reuse `render_gate` against a temporary name table mapping each operand id → its
  positional param — but the temp-table approach breaks on duplicate operands, so
  the positional-render variant is preferred.)
- **Call site**: the gate's `assign <names[idx]> = <render_gate(...)>;` becomes
  `assign <names[idx]> = <names[idx]>__f(<node_ref(o0)>, <node_ref(o1)>, …);`. The
  operand refs are exactly today's `node_ref` outputs (wires / literals), so the
  function is **behaviour-preserving by construction**.
- Naming: `<wire>__f` mirrors the `soft_union` `<gate>__u` suffix convention;
  `build_names` already guarantees unique gate wire names, so `__f` suffixes are
  unique too. The function name is module-local (functions live in module scope).

### Q4 — the `function_emit_prob` knob

A new `Config::function_emit_prob: f64` (default `0.0`) beside `aggregate_prob` /
`soft_union_slice_prob`, with a `default_function_emit_prob()` serde default
(`0.0`). Default `0.0` ⇒ `annotate_function_emit_gates` is not called (call-site
guard) ⇒ no RNG draw, nothing marked, `function_emit_gates` empty ⇒
**byte-identical** (`tests/snapshots.rs` untouched). Surfaced in `--dump-config` /
`--introspect` automatically (it is a `Config` field; the introspection schema is a
serde projection of `Config`, so this is a `Config`-field MINOR bump — to be
confirmed against the schema-version policy in `.2b`, like `sv_version`'s `1.1 →
1.2`).

### Q5 — the downstream gate

A focused repo-owned gate proves the emitted functions are accepted
**warning-clean** by Verilator + **both** Yosys modes + Icarus, gated on a new
`saw_combinational_function_emit` coverage fact (the "prove the surface is
accepted, not just produced" bar the prior breadth lanes hold — the
`saw_sv_version_2023_soft_union_upopt` / `saw_packed_aggregate_design`
precedent). Shape (a `tool_matrix` scenario or a dedicated bank) resolved in
`.2b.2`; a `Metrics`/`DesignMetrics` count
(`num_emitted_combinational_functions`, a structural scan of
`function_emit_gates`) feeds the fact.

### `.2b` pre-split

Per the `soft_union` `.3b.2a`/`.3b.2b` + the SEMANTIC-INTROSPECTION `.Xb.1`/`.Xb.2`
precedent, `.2b` pre-splits into:
- **`.2b.1`** (the live surface, **new frontier**): `Config::function_emit_prob`
  + `Module.function_emit_gates` + `src/ir/function_emit.rs`
  (`annotate_function_emit_gates`) + the generator call-site roll + the emitter
  `function automatic` decl/call rendering in `to_sv_with_modules` + lib proofs
  (a marked gate emits a behaviour-preserving function + call; default-off
  byte-identical; the mark leaves CSE / `canonical_module_signature` untouched).
  Banked Verilator `--lint-only` clean on a forced-knob sample.
- **`.2b.2`** (the repo-owned gate + closeout): the `saw_combinational_function_emit`
  coverage fact + the `tool_matrix` scenario (Verilator + both Yosys + Icarus) +
  the metric + book/USER_GUIDE/KM.

### Rejected (at this design altitude)

- **Multi-level cone body in the first cut** — rejected for the first cut (the
  sharing/scoping hazard); a follow-up leaf, not a blocker.
- **A pure emit-time selection pass** — rejected in favour of the gen-time
  annotation (the `soft_union.rs` precedent: seeded, reproducible, emitter stays a
  pure projection).
- **Node-id operand→param name mapping** — rejected (breaks on duplicate
  operands); positional params are used.

## 2026-06-16 — Semantic introspection — `module_reachability` impl design-detail — `SEMANTIC-INTROSPECTION-EXPANSION.5a`

Design-detail leaf for `.5` — the **fourth** derived query, `module_reachability`:
*which modules in a design are reachable from the top via the instance graph, and
how does each module sit in that graph?* Docs-only (no source). This is the last
named query kind in decision `0011`; with it the lane's named set is delivered.
Grounded in the real IR (`src/ir/types.rs`):

```rust
pub struct Design { pub top: String, pub modules: Vec<Module> }
pub struct Module { /* … */ pub instances: Vec<Instance>, /* … */ }
pub struct Instance { pub id: InstanceId, pub name: String, pub module: String,
                      pub role: InstanceRole, /* … */ }   // `module` = child module NAME
```

The instance graph is already in the IR: each `Module` lists its child `Instance`s,
and each `Instance.module` names the child module definition (resolved against
`Design.modules` by name). `module_reachability` is the pure graph-reachability
relation over those edges, rooted at `Design.top`. Same SCHEMA-DERIVED /
structure-first ceiling (decisions `0004`/`0011`): reachability is a **relation**
over the construction graph, never behaviour. It is the first query whose natural
home is the **whole design** rather than one module's node graph (`output_support`
/ `input_reach` walk the gate graph; `flop_reset_provenance` projects `Module.flops`
— all three operate on the top module; this one traverses the module table).

### Q1 — result shape: a FOURTH parallel result vec

`DerivedAnalysis` gains a fourth `#[serde(default, skip_serializing_if =
"Vec::is_empty")]` vec, continuing the parallel-vec pattern (`.3a` chose this over
a tagged `results` enum so prior documents stay byte-identical; each new kind = one
more skip-if vec, the `query` field discriminates — now scaled to four):

```rust
pub module_reachability: Vec<ModuleReachability>,   // populated only by module_reachability

pub struct ModuleReachability {
    pub module: String,              // the module name (the entity this entry is about)
    pub reachable: bool,             // reachable from design.top via the instance graph
    #[serde(skip_serializing_if = "Option::is_none")]
    pub depth: Option<usize>,        // min instance-graph distance from top (0 = top);
                                     // present iff reachable, omitted otherwise
    pub instantiates: Vec<String>,   // distinct child module NAMES directly instantiated
                                     // (sorted, deduped) — this module's local out-edges
    pub instance_count: usize,       // total direct child instances (m.instances.len())
}
```

Field-name note: the primary key is `module`, not `target` — like `FlopProvenance`
used `flop` (the prior two used `target`), the entity here is a module, named
descriptively. `depth` is `Option<usize>` with `skip_serializing_if =
"Option::is_none"`: a number (0 for the top) when reachable, omitted when not — the
schema's "omit rather than null" convention, and an agent reads `reachable` first
anyway. `instantiates` (the distinct direct child module names) is present for
**every** module, reachable or not — it is a local structural fact, so the flat Vec
carries the full out-edge structure and a consumer can reconstruct the entire
instance graph + recompute reachability itself (the API-audience completeness rule:
ship the complete structured relation). `instance_count ≥ instantiates.len()` when a
module instantiates the same child more than once. No design-level summary struct
(total/reachable/dead counts) — those are derivable from the Vec, so adding one
would be a second source of truth; the flat Vec is complete. Additive MINOR schema
bump `1.6 → 1.7`.

### Q2 — derivation: a BFS over the design's instance edges

`design_module_reachability(&Design, target)`:
1. Build a `name → &Module` index over `design.modules`.
2. BFS from `design.top`: min-depth, a `visited` depth map, a `VecDeque` frontier;
   for each module follow its **distinct** child names (`instances[].module`) to
   children present in the index, recording first-visit (= min) depth. A child name
   with no matching `Module` def is a recorded out-edge that cannot be traversed
   (defensive — a well-formed design has all defs; we never panic, mirroring the
   `visit()` "missing node ⇒ 0" guard in the support walker).
3. Emit one `ModuleReachability` per module in `design.modules`, **sorted by module
   name** (deterministic, independent of the `modules`-vec / instance order — the
   same determinism contract `flop_provenance` holds by sorting on flop id):
   `reachable` = visited; `depth` = the BFS depth if reachable else `None`;
   `instantiates` = distinct `instances[].module` (sorted, via `BTreeSet`);
   `instance_count` = `instances.len()`.

Pure: no IR field, no generator change (the `coverage_gaps` / `output_support`
project-don't-recompute precedent). Cost is `O(modules + edges)` — a single linear
BFS — well within a read-only analysis. Min-depth BFS is order-independent, and the
output sort + `BTreeSet` aggregation make the document a byte-stable function of the
design.

### Q3 — target addressing: a module NAME

`target` is a **module name** (not a port name or `"flop:<id>"`): the natural
identifier for a module-level query.
- `target = None` ⇒ one entry per module in `design.modules`, sorted by name.
- `target = Some("<module name>")` ⇒ that one module's entry (it must exist in
  `design.modules`).
- an unknown module name ⇒ no entry ⇒ `-32602` at the MCP layer — the established
  "unknown target vs known-but-empty" contract (every resolvable module yields
  exactly one entry, even an unreachable one). The `run_analyze` empty-result guard
  checks `analysis.module_reachability.is_empty()` for this kind.

This is deliberately a *different* target vocabulary from the prior three queries,
documented as such: `output_support` takes an output port name / `"flop:<id>"`;
`input_reach` takes a source (input / `"flop:<id>"` Q / `"<inst>.<port>"`);
`flop_reset_provenance` takes `"flop:<id>"`; `module_reachability` takes a module
name.

### Q4 — module-vs-design semantics

`run_analyze` already routes design-vs-module on
`cfg.effective_hierarchy_depth_range().is_some()` (a hierarchy config ⇒ a `Design`;
otherwise a single `Module`). `design_module_reachability` is the real query.
`module_module_reachability(&Module, target)` is the **degenerate one-node case**: a
bare `Module` carries no child definitions to traverse (the same "no child defs"
boundary the module variant of every other query hits — cf. the
`"<instance>.port<id>"` fallback in `format_instance_leaf_module`). It emits exactly
one entry — the module itself: `reachable = true`, `depth = Some(0)`, `instantiates`
= its own distinct `instances[].module` names, `instance_count = instances.len()`.
It does **not** fabricate entries for the named-but-undefined children (no `Module`
to honestly report on). A non-hierarchical DUT leaf has no instances, so the runtime
module-path answer is the honest `{module, reachable: true, depth: 0, instantiates:
[], instance_count: 0}`. (`module_module_reachability` still handles a hand-built
module-with-instances correctly, for test coverage and completeness.) `target` for
the module variant: `None` or `Some(m.name)` ⇒ the one entry; anything else ⇒ none ⇒
`-32602`.

### `.5b` pre-split

Per the `.3b`/`.4b` precedent:
- **`.5b.1`** (pure core, **new frontier**): `QUERY_MODULE_REACHABILITY` +
  `ModuleReachability` + the fourth `module_reachability` vec +
  `design_module_reachability` / `module_module_reachability` (+ a shared
  `module_reachability_with` helper if it factors cleanly). The 6 existing
  `DerivedAnalysis` literals gain `module_reachability: Vec::new()`. **Not** added to
  `supported_query_kinds()` yet — it joins the registry together with the
  `run_analyze` dispatch in `.5b.2`, so no intermediate commit mislabels the
  supported set. Lib-tested only; not wired to any emit path ⇒ DUT byte-identical
  (snapshots 6/6).
- **`.5b.2`** (surface): add the kind to `supported_query_kinds()` AND branch
  `run_analyze` by kind (same commit) + the vec-aware `-32602` guard; schema
  `1.6 → 1.7` (+ the `"1.6"` test-assertion bumps); the `analyze_schema` enum + the
  tool/`instructions` descriptions; schema-doc §6.7 + a `1.6 → 1.7` changelog + the
  row; book(`agent-mcp`) + USER_GUIDE + README; a KM card. Default-off / DUT
  byte-identical.

### Rejected

- **A tagged `results` enum** — rejected at `.3a` and stays rejected: it would break
  the byte shape of the three delivered documents. A fourth parallel skip-if vec is
  additive and keeps the bump a clean MINOR.
- **A separate reachability *summary* struct** (counts of total / reachable / dead
  modules) — derivable from the flat Vec, so it would be a second source of truth.
  The Vec is complete.
- **Recursing the support/reach cone *through* child instances** (so `output_support`
  crosses the instance boundary) — that is a different, larger feature (the cone
  queries deliberately stop at the instance boundary, decision `0011` Q3);
  `module_reachability` answers the orthogonal module-graph question and leaves the
  per-module cone boundaries unchanged.
- **Computing reachability at generation time / storing it in the IR** — rejected
  (byte-identical contract); it is pure read-only post-hoc analysis.

## 2026-06-16 — Semantic introspection — `flop_reset_provenance` surface + schema 1.6 — `SEMANTIC-INTROSPECTION-EXPANSION.4b.2`

Wires the `.4b.1` core to the MCP surface, closing the third derived query
(`.4b` + `.4`) end-to-end. Same shape as the `.3b.2` (`input_reach`) closeout —
registry entry + `run_analyze` dispatch in one commit, schema `1.5 → 1.6`,
`output_support`/`input_reach` documents byte-identical (the `flop_provenance`
key is `skip_serializing_if`-omitted). One architectural observation worth
recording for cold recovery: **the parallel-vec pattern has now scaled cleanly to
three query kinds** (`results` / `reach_results` / `flop_provenance`), each a
`skip_serializing_if` vec the `query` field discriminates. This validates the
`.3a` decision to reject a tagged-enum `results` — every new kind is one more
optional vec, prior documents stay byte-identical, and the schema bump is always
a clean additive MINOR. The lane's named-query set is now down to one open kind,
`module_reachability` (`.5+`, open-ended). E2e `anvil-mcp` smoke: seed 3 → schema
`1.6`, 31 flops (flop 0 async/hold/encoded); unknown `flop:<id>` → `-32602`.

## 2026-06-16 — Semantic introspection — `flop_reset_provenance` impl design-detail — `SEMANTIC-INTROSPECTION-EXPANSION.4a`

Design-detail leaf for `.4` — the **third** derived query, `flop_reset_provenance`:
*which flops are reset-defined vs data-driven, and how is each one's next state
built?* Docs-only (no source). Grounded in the real `Flop` type
(`src/ir/types.rs`): `Flop { id, width, d: Option<NodeId>, q, reset_val: u128,
reset_kind: ResetKind {None|Sync|Async}, kind: FlopKind {ZeroDefault|QFeedback},
mux: FlopMux {None|OneHot(Vec<MuxArm>)|Encoded{sel,data}} }`. Every field this
query reports already exists on the flop — so it is the **purest** derived query
yet: a direct projection of `Module.flops`, not even a graph walk (unlike
`output_support` / `input_reach`, which traverse the node graph). Same
SCHEMA-DERIVED / structure-first ceiling (decision `0004`/`0011`); reset
*provenance* is a relation (how the register's next state is constructed), never
behaviour.

### Q1 — result shape: a THIRD parallel result vec

`DerivedAnalysis` gains a third `#[serde(default, skip_serializing_if =
"Vec::is_empty")]` vec, continuing the established parallel-vec pattern (`.3a`
chose this over a tagged enum to keep prior documents byte-identical; each new
kind = one more skip-if vec, the `query` field discriminates):

```rust
pub flop_provenance: Vec<FlopProvenance>,   // populated only by flop_reset_provenance

pub struct FlopProvenance {
    pub flop: u32,                 // flop id (addressed "flop:<id>")
    pub width: u32,
    pub has_reset: bool,           // reset_kind != None
    pub reset_kind: String,        // "none" | "sync" | "async"
    pub reset_value: String,       // reset_val as a DECIMAL string (see below)
    pub default_behavior: String,  // "zero" (ZeroDefault) | "hold" (QFeedback)
    pub mux_kind: String,          // "none" | "one_hot" | "encoded"
    pub mux_arms: usize,           // # mux arms (0 for None; OneHot arms / Encoded data len)
    pub has_d: bool,               // d.is_some() — a dead/undriven flop has no D cone
}
```

`reset_value` is a **decimal string**, not a number: `reset_val` is `u128`, and
serializing 128-bit ints as JSON numbers is fragile (not all consumers/serde
configs round-trip `u128`), whereas a string is exact, deterministic, and
machine-parseable. `mux_arms` counts arms for `OneHot` (`arms.len()`) and data
slots for `Encoded` (`data.len()`); `default_behavior` exposes the `FlopKind`
semantics (what `D` becomes when no select is asserted). The enum→string mappings
(`reset_kind`, `mux_kind`, `default_behavior`) keep the wire stable even if the
Rust enum gains variants. Additive MINOR schema bump `1.5 → 1.6`.

### Q2 — derivation: a direct projection of `Module.flops`

No traversal: iterate `Module.flops`, read each flop's fields, map the enums to
strings. Pure, O(flops), no IR field, no generator change (the `coverage_gaps` /
`output_support` project-don't-recompute precedent). Determinism: emit flops in
**ascending id** order (the `source_universe` flop ordering precedent).

### Q3 — target addressing

- `target = None` ⇒ a `FlopProvenance` for **every** flop (ascending id) — the
  agent-audience completeness rule.
- `target = Some("flop:<id>")` ⇒ that one flop (same `"flop:<id>"` spelling the
  other two queries use; here it addresses the flop itself, not a D/Q direction).
- An unknown `"flop:<id>"` (or any unrecognised string) ⇒ no result ⇒ `-32602` at
  the MCP layer (the established contract). A module with **no flops** + `None` ⇒
  an empty `flop_provenance` (a known-empty result, distinct from an unknown
  target — but with `None` there is no target to be unknown, so it is simply the
  honest "no flops" answer; `Some("flop:0")` on a flopless module ⇒ `-32602`).

### Q4 — schema: additive MINOR `1.5 → 1.6`

A new `#[serde(default, skip_serializing_if)]` field + the `FlopProvenance`
struct + the `"flop_reset_provenance"` query kind. `output_support` /
`input_reach` documents stay byte-identical (the new key is omitted on them); the
`DerivedAnalysisDocument` envelope is reused unchanged. DUT `.sv` byte-identical
(introspection is not in `tests/snapshots.rs`).

### `.4b` impl shape (and the recommended pre-split)

Pre-split `.4b` → **`.4b.1`** (pure core) + **`.4b.2`** (surface), per the `.3b`
precedent:

- **`.4b.1`:** in `src/introspect/analyze.rs` add `QUERY_FLOP_RESET_PROVENANCE =
  "flop_reset_provenance"`, the `FlopProvenance` struct, the `flop_provenance`
  field on `DerivedAnalysis`, and the pure builders `module_flop_provenance(&Module,
  Option<&str>)` / `design_flop_provenance(&Design, Option<&str>)`. **Do NOT** add
  the kind to `supported_query_kinds()` yet (registry + `run_analyze` dispatch land
  together in `.4b.2`). Lib proofs: each `ResetKind`/`FlopKind`/`FlopMux` variant
  maps correctly; `reset_value` string; `None` ⇒ all flops ascending; flopless
  module ⇒ empty; `"flop:<id>"` target + unknown-target ⇒ none; determinism.
  Snapshots 6/6 byte-identical.
- **`.4b.2`:** add the kind to `supported_query_kinds()` + branch `run_analyze`
  (the empty-result `-32602` guard checks `flop_provenance` for this kind);
  `SCHEMA_VERSION` `1.5 → 1.6` (+ the `"1.5"` test-assertion updates); the
  `analyze_schema` `enum` + tool/instructions text; schema-doc §6.7 + a `1.5 →
  1.6` changelog + the row; book `agent-mcp` (row + worked example) + USER_GUIDE +
  README + a KM card. Default-off / DUT byte-identical.

## 2026-06-16 — Semantic introspection — `input_reach` surface + schema 1.5 — `SEMANTIC-INTROSPECTION-EXPANSION.3b.2`

Wires the `.3b.1` core to the agent-facing surface, closing the `input_reach`
query (`.3b` + `.3`) end-to-end. Three things worth recording:

- **Registry + dispatch in one commit (the `.3b.1` promise kept).** This commit
  both adds `QUERY_INPUT_REACH` to `analyze::supported_query_kinds()` *and*
  branches `run_analyze` by query kind (`module_input_reach`/`design_input_reach`
  vs the support builders). They land together so the registry (the MCP `-32602`
  gate) and the dispatch never disagree — there is no commit in which
  `input_reach` is "accepted but mislabelled". The unknown-target → `-32602`
  guard now checks the result vec the *query* populates (`reach_results` for
  `input_reach`, `results` for `output_support`).
- **Schema `1.4 → 1.5`, additive MINOR, `output_support` byte-identical.** The
  only wire change is the `reach_results` field (already added in `.3b.1` with
  `skip_serializing_if`), so an `output_support` document is byte-identical to
  `1.4`. The bump is the honest, consistent call (every prior field addition
  bumped MINOR; the `.2b` analysis surface bumped even though its default
  document was unchanged). All `"1.4"` schema-version test assertions in
  `introspect`/`mcp` moved to `"1.5"`.
- **Doc-hygiene caught in passing:** the MCP `introspect` tool description still
  claimed "schema 1.0"; rather than chase the version forever I made it
  version-agnostic ("the `schema_version` field carries the version"), so it
  can't re-stale on the next bump. DUT byte-identical throughout (snapshots 6/6).

## 2026-06-16 — Semantic introspection — pure `input_reach` core — `SEMANTIC-INTROSPECTION-EXPANSION.3b.1`

Implements the `.3a` design: the pure `input_reach` core in
`src/introspect/analyze.rs` (the dual fan-out of `output_support`). Lib-tested
only; not wired to the MCP surface (that is `.3b.2`). Two impl-time decisions
worth recording beyond the `.3a` design:

- **The registry stays at `output_support` until `.3b.2`.** `QUERY_INPUT_REACH`
  + the `ReachResult` struct + the `reach_results` field + the
  `module_input_reach`/`design_input_reach` builders all land here, but
  `supported_query_kinds()` is **deliberately not** extended yet. That set is the
  MCP gate `run_analyze` checks; adding `input_reach` to it before the
  `run_analyze` dispatch branch exists (a `.3b.2` change) would let an
  `input_reach` request fall through to the support-cone branch and silently
  mislabel in the intermediate commit. Registry entry + dispatch land together in
  `.3b.2` so the two never disagree. This is the "keep every commit coherent" rule
  applied to a two-commit feature split.
- **Reach = the transpose of support, computed by inversion — not a second
  walker.** `input_reach_with` builds every target's `SupportCone` with the
  *existing* `build_cone` machinery (outputs + each `"flop:<id>"` D-cone), then
  buckets each target under the sources its cone lists. The flop/instance/mem-fsm
  boundary rules therefore live in exactly one place; a forward consumers-BFS
  would have had to re-implement all of them against a reverse adjacency. A flop
  in a cone's `support_flops` is the flop's **Q**, so as a reach *source* it is
  keyed `"flop:<id>"` — the same register-boundary spelling the `output_support`
  D-cone *target* uses, with the direction set by the query kind. Gotcha for the
  source universe: declared control ports (`clk`/`rst_n`) are enumerated too and
  show **empty** combinational reach — the honest dual of `output_support`'s
  "one cone per declared output, even undriven", and intentional under the
  API-audience completeness steering.

## 2026-06-16 — Semantic introspection — `input_reach` impl design-detail — `SEMANTIC-INTROSPECTION-EXPANSION.3a`

Design-detail leaf for `.3` — the **second** derived query, `input_reach`: the
dual fan-OUT of the delivered `output_support` cone. Docs-only (no source).
Grounded in a fresh read of `src/introspect/analyze.rs` (the `DerivedAnalysis` /
`SupportCone` types + the `module_support_cones` / `design_support_cones`
builders + the `visit` fan-in DFS + the `resolve_target` resolver),
`src/introspect/mod.rs` (the `DerivedAnalysisDocument` envelope +
`derived_analysis_document` wrapper + `SCHEMA_VERSION`), and `src/mcp/mod.rs`
(`run_analyze` dispatch + the `analyze_schema` enum + the unknown-target →
`-32602` guard). API-audience steering (owner, `2026-06-16`,
`feedback_api_for_agents_not_humans`): optimize for machine-friendly
**completeness** (full reach sets, all-sources, explicit ids) over human-terse
digests, within the unchanged SCHEMA-DERIVED / no-shadow-simulator ceiling.

`input_reach` answers the symmetric question to `output_support`: given a
**source** — a primary-input port, a flop `Q`, or a child-instance output —
*which outputs and which flop `D`-cones does it structurally reach?* It is the
exact transpose of the support relation, and the design pins it that way so the
two queries cannot drift.

### Q1 — result shape + the `DerivedAnalysis.results` vs second-vec decision

A new `ReachResult` struct (the dual of `SupportCone`), serde + `Default`, with
`BTreeSet → sorted Vec` for byte-stability, in the same `analyze.rs`:

```rust
pub struct ReachResult {
    pub target: String,             // the reach SOURCE this result is about:
                                    //   an input port name, "flop:<id>" (a Q), or "<inst>.<port>"
    pub reaches_outputs: Vec<String>,// output port names the source reaches (sorted)
    pub reaches_flops: Vec<u32>,     // flop ids whose D-cone the source reaches (sorted)
    pub fanout_targets: usize,       // = reaches_outputs.len() + reaches_flops.len() (the dual of cone_nodes-ish summary)
}
```

The per-result addressing field stays named `target` (not `source`) so **every**
derived-query result shares one uniform "the entity this result is about" key
regardless of direction — the module docs state that for `input_reach`, `target`
is the reach *source*. (Rejected `source`: it would fork the result-document
shape per query kind for no machine-parsing gain; a `query`-keyed field is the
clean discriminator.)

**The `DerivedAnalysis` shape choice (the leaf's named fork):** keep
`results: Vec<SupportCone>` exactly as-is and add a **second parallel vec**

```rust
pub struct DerivedAnalysis {
    pub query: String,
    pub results: Vec<SupportCone>,                 // populated by output_support
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reach_results: Vec<ReachResult>,           // populated by input_reach
}
```

A given query populates exactly one vec; the `query` field tells the agent which
to read. Chosen over the two rejected alternatives:

- **Generalize `results` to a tagged enum `Vec<AnalysisResult>`** — rejected: it
  retags the *existing* `output_support` wire shape, breaking every `1.3`/`1.4`
  consumer of that document (a MAJOR break for a purely additive feature).
- **Shoehorn reach into `SupportCone`** — rejected: `support_inputs` / `cone_depth`
  are semantically wrong for a fan-out relation; it would mislead an agent.

The second-vec design keeps `output_support` documents **byte-identical**
(`reach_results` is empty there ⇒ `skip_serializing_if` omits the key entirely),
so no existing consumer is touched, while `input_reach` documents carry
`results: []` + a populated `reach_results`. This is a clean additive change.

### Q2 — the derivation: invert the support relation (reuse `module_support_cones`)

`input_reach` is computed by **inverting the existing support cones**, not by a
new forward traversal:

1. Enumerate **all targets** of the artifact = every output port name **and**
   every `"flop:<id>"` for `f in m.flops` (the D-cone targets).
2. Build each target's `SupportCone` with the **existing** `analyze.rs`
   machinery (`build_cone` / `visit` / the `fmt` instance-leaf closure) — no new
   walker.
3. Invert: a source `X` *reaches* target `T` **iff** `X ∈ support(T)`. Scan each
   cone's `support_inputs` / `support_flops` / `support_instance_outputs` and
   bucket the target under each support element.

This is the leaf's "invert per-output/per-flop-D support" option, chosen over a
"forward consumers BFS" because:

- **Dual-consistency is free and provable.** By construction `X reaches Y ⇔ Y's
  support ∋ X`, so the `.3b` lib proofs are literally the transpose of the
  landed `output_support` proofs — there is no second place for the
  flop-boundary / instance-boundary / mem-fsm-termination stopping rules to
  drift. A forward BFS would have to *re-implement* every one of those boundary
  rules against a reverse-adjacency it would also have to build.
- **No IR field, no generator change** (the `coverage_gaps` /
  `output_support` project-don't-recompute precedent) ⇒ DUT byte-identical.
- **Cost is bounded by module size** and this is a read-only analysis tool, not
  an output path; per the API-audience steering, completeness wins over
  micro-optimizing the O(targets × cone) inversion. (A future shared
  reverse-index is a noted optimization, not first-cut.)

### Q3 — `target` (source) addressing + the `"flop:<id>"` duality

- `target = None` ⇒ **all sources**, for completeness (API-audience steering):
  one `ReachResult` per declared **input port** (declaration order), then one per
  flop `Q` as `"flop:<id>"` (ascending id), then one per child-instance-output
  node present in the IR (sorted resolved `"<inst>.<port>"` name). A source that
  reaches nothing yields an **empty** `ReachResult` (dual of `output_support`'s
  "known-but-undriven output ⇒ one empty cone").
- `target = Some("<input port name>")` ⇒ that input's reach.
- `target = Some("flop:<id>")` ⇒ that flop's **Q** reach (fan-out). **Addressing
  duality, documented explicitly:** `"flop:<id>"` denotes the flop's *register
  boundary*; the **query kind** sets the direction — `output_support` asks what
  feeds its `D` (fan-in), `input_reach` asks what its `Q` reaches (fan-out). Same
  spelling, opposite arrow.
- `target = Some("<inst>.<port>")` ⇒ that child-instance output's reach (design
  lane; resolved via the same `format_instance_leaf_design` naming).
- An **unresolvable** source ⇒ no `ReachResult` ⇒ `-32602` at the MCP layer
  (exactly the `output_support` precedent). A *resolvable* source always yields
  exactly one result, even when empty, so empty `reach_results` for an explicit
  target means "unknown source", never "known but reaches nothing".

### Q4 — schema-version decision: additive MINOR `1.4 → 1.5`

Adding the `reach_results` field (a new `#[serde(default,
skip_serializing_if)]` wire field) + the `ReachResult` struct + the
`"input_reach"` query kind is, per the §7 policy ("surfacing struct fields added
with `#[serde(default)]`" / "a new embedded section"), an **additive MINOR bump
`1.4 → 1.5`** — the same call the `.2b` analysis surface made even though its
default document was unchanged. The `output_support` document stays
byte-identical (`reach_results` omitted); only the `schema_version` string
advances and the new field becomes advertised. DUT `.sv` stays byte-identical
(introspection output is not in `tests/snapshots.rs`). The `DerivedAnalysisDocument`
envelope and the `derived_analysis_document` wrapper are **reused unchanged** —
the payload is still a single `analysis: DerivedAnalysis`.

### `.3b` impl shape (and the recommended pre-split)

Pre-split `.3b` → **`.3b.1`** (pure core) + **`.3b.2`** (surface), per the `.2b`
precedent, because it spans two reviewable ownership areas:

- **`.3b.1` — pure core (`src/introspect/analyze.rs`, lib-tested):** add
  `QUERY_INPUT_REACH = "input_reach"`, the `ReachResult` struct, the
  `reach_results` field on `DerivedAnalysis`, and the pure builders
  `module_input_reach(&Module, Option<&str>)` / `design_input_reach(&Design,
  Option<&str>)` (enumerate-targets → build-cones → invert → resolve-source).
  **Do NOT yet add `input_reach` to `supported_query_kinds()`** — that registry
  entry is the MCP gate, and adding it before `run_analyze` dispatches the kind
  (a `.3b.2` change) would let an `input_reach` call fall through to the
  support-cone branch and silently mislabel in the intermediate commit. Keep
  each commit coherent: the registry entry + the dispatch land together in
  `.3b.2`. Lib proofs: exact reach (the transpose of the support proofs);
  flop-`Q`-as-source reach; instance-output-as-source reach (design);
  `None` ⇒ all-sources incl. an empty `ReachResult`; determinism + sorted;
  unknown-source ⇒ no result. Snapshots 6/6 byte-identical (not wired to any
  emit path).
- **`.3b.2` — surface:** add `input_reach` to `supported_query_kinds()`; branch
  `run_analyze` by query kind (support builders vs reach builders) and update the
  empty-result → `-32602` guard to check the vec the query populates; bump
  `SCHEMA_VERSION` `1.4 → 1.5` (+ the `"1.4"` test-assertion updates); add
  `"input_reach"` to the `analyze_schema` `enum` + refresh the tool description;
  schema-doc §6.7 + a `1.4 → 1.5` changelog entry + the `input_reach` row; book
  `agent-mcp` (an `input_reach` row + a worked example) + USER_GUIDE (the tool
  enum + `1.4 → 1.5`) + a KM fact. Default-off / DUT byte-identical.

## 2026-06-16 — Sequential proof metric + schema 1.4 + downstream bank — `IDENTITY-DEEPENING.3b.2b.2a`

The observability + evidence closeout half of the cross-module sequential merge.

**One grouping, two consumers — by construction, not by hope.** The metric's
"duplicate pairs" must equal exactly what the pass would collapse, or the gate
"metric reads N, pass reduces to 0" is a lie. So I factored the pre-filter + greedy
grouping into a single non-mutating `group_sequentially_equivalent_modules(&Design)
-> Vec<Vec<usize>>` (`src/ir/dedup.rs`) that BOTH `dedup_sequential_modules_once`
(filters to len≥2, picks lex-survivor) and `compute_design`'s metric (counts pairs,
hashes class ids) call. The dedup pass behaviour is unchanged (the gate tests from
`.3b.2b.1` still pass byte-for-byte).

**Why the signature is a class-id hash, not a content hash.** The combinational
`semantic_module_signatures` are content hashes (truth tables) — a per-module
canonical proof, so equality is transitive *and* independently computable. Sequential
equivalence has no per-module canonical form: it is decided pairwise (a bisimulation
between two machines). So `sequential_module_proof_signatures[i]` is the deterministic
**class id** = FNV-1a of the class's lex-smallest member name. Two modules share a
value iff they were proven equivalent *within this design's grouping*. This is honest
(documented on the field) and sufficient for the metric's job (observability + the
"reducible to 0" gate). The combinational metric counts pairs by grouping equal
proofs in a `BTreeMap`; the sequential metric counts pairs from the shared grouping —
a real difference forced by the pairwise nature of the proof.

**The schema bump was mandatory, not optional.** `DesignMetrics` is a `serde`
projection inside the `--introspect` payload, and the SCHEMA-DERIVED invariant says
any change to the projected structs is a schema change. The schema doc §7 even names
"DesignMetrics fields" as the canonical additive-MINOR case. So adding the pair is a
`1.3 → 1.4` MINOR bump — both fields `#[serde(default)]` (a `1.3` reader ignores them;
absent reads back empty/0), the envelope shape unchanged. I kept every schema-version
statement in sync in one slice (const + schema doc §4/§7 + checklist + the
`introspect`/`mcp` test assertions + the README/USER_GUIDE/book example numbers) so
the contract never drifts. Contrast `SV-VERSION-TARGETING.3b.2b`, which *avoided* a
bump by keeping its evidence flag matrix-local (`ModuleReport`) — that was the right
call there (no design-level meaning); here the metric is genuinely a `DesignMetrics`
fact, so the bump is correct.

**Cost is bounded even on the default path.** `compute_design` runs on every manifest
/ coverage computation, knob on or off. The metric pre-filters eligible stateful-leaf
modules by `(interface, flop multiset)`, so the `O(n²)` cross-module proof only fires
inside a same-shape bucket of ≥2 — and a default design rarely has even one eligible
stateful leaf, let alone two of one shape. So the added cost is ~zero on the default
path, and the metric is RTL-invisible regardless (snapshots 6/6 byte-identical).

**Bank faithfulness: one `.sv` per module.** The merged multi-module design tripped
Verilator `DECLFILENAME` only because the smoke dump bundled both modules in one file;
ANVIL `--out` writes one `.sv` per module definition (no such warning). So the bank
splits the dump per module before linting — the faithful representation. Clean across
Verilator `-Wall`, Yosys both modes (non-empty `$_DFF_` netlist), and Icarus
`-g2012`.

## 2026-06-16 — Cross-module sequential-equivalence merge mechanism — `IDENTITY-DEEPENING.3b.2b.1`

Implemented the bounded whole-leaf-module sequential-equivalence merge designed in
decision `0008` and grounded in `.3b.1`: the mechanism + proof, fully wired and
default-off. (The metric pair, downstream-clean bank, and book/USER_GUIDE/ROADMAP/KM
closeout are deferred to `.3b.2b.2`.)

**The proof reuses the flop primitive on a *combined* module — no new engine.**
`modules_sequentially_equivalent(a, b)` (`src/ir/compact.rs`) materializes a throwaway
combined module `a.nodes ++ b.nodes` / `a.flops ++ b.flops` with B's `NodeId`/`FlopId`
references offset (`build_combined_module`), then calls the existing
`bisimulation_partition` (`.3b.2a`) on the union state and checks per-output-port drive
cone equality under the final quotient. The whole trick that makes a single bisimulation
class span flops from *both* modules is that B's `Node::PrimaryInput { port, width }`
nodes keep their `port`, and `LeafEndpoint::PrimaryInput` already keys by `(port, width)`
— so A's and B's primary inputs unify *for free* in the shared endpoint vocabulary. This
is why the central `.3a` "cross-module cone-proof signature" challenge needed no new
machinery (the `.3b.1` resolution).

**Soundness does not need the partition to refine by output, nor a flop bijection.**
`bisimulation_partition` only refines by D-cone (transition) agreement under the quotient
+ reset bucketing — it does *not* label-refine by which output a flop feeds. That is
sound here because output equality is proven *separately* (step 4) under the *final*
quotient: the partition guarantees "same class ⇒ equal value for all time" (reset base
case + stable quotient step = coinduction), and step 4 proves A's and B's output cones
are equal functions of those equal-valued classes + unified inputs. Together ⇒ equal
outputs every cycle. No flop bijection is required: if A's and B's output cones reference
flops in *different* classes the structural/semantic proof simply differs and the pair is
rejected (conservative, never unsound).

**Two `cone_proof` gotchas, both load-bearing.** (1) The output-equality proofs for all
ports must share ONE `StructuralSignatureCtx` interner — `ConeProof::Structural` is an
interner-relative id, so proofs computed against different contexts are incomparable.
(2) The four memos are valid across all ports only because the quotient is *fixed* (the
final `rep_map`); they are `NodeId`-keyed and assume fixed endpoint identity (the same
invariant the `.2b` refinement loop rebuilds-per-iteration for).

**First-cut eligibility (`sequential_leaf_eligible`) and excluded boundaries.** Stateful
flops-only leaf modules with every flop settled + reset-defined, and no memories / FSMs /
instances / params / aggregates / multiple clock domains. Resetless flops are excluded
(no `t=0` base case — carries the `0007`/`.2b` boundary forward). Multi-clock is excluded
in the first cut because clock-domain indices are module-local, so a naive cross-module
union would be unsound without a domain correspondence; `clock_domains.is_empty()` sidesteps
it cleanly. Each exclusion is a separately-recorded boundary / named future leaf, none
retired.

**The dedup pass groups greedily by representative — sound because the relation is
transitive.** `dedup_sequential_modules` (`src/ir/dedup.rs`) buckets eligible leaves by a
cheap `SequentialPrefilterKey` (sorted `(PortId,width)` interface + sorted flop multiset),
then within a bucket compares each module against existing group representatives. Real
sequential equivalence is a true equivalence relation, so matching any representative is
sound even though the (incomplete-but-sound) prover only checked against `rep`: `X≡rep`
and `Y≡rep` ⇒ `X≡Y`, so rewriting both to the group's lex-smallest survivor is sound. It
reuses the shared `rewrite_instance_module_names` / `prune_modules_made_unreachable` tail;
leaf modules never instantiate anything, so the combinational pass's ancestor/descendant
rewrite-cycle guard is unnecessary here.

**Budget.** Per-cone checks reuse the 12-bit / 128-node / 131072-work
`MERGE_SEMANTIC_LIMITS` verbatim; the combined flop count is capped at
`N_BISIM_MODULE_FLOPS = 64` so every combined `(width,reset,domain)` bucket also stays
within the per-bucket `N_BISIM_FLOPS` refinement cap. Over-budget pairs fail to merge
(never a guess).

**Wire-in + default-off contract.** Gated in `generate_design` (`src/gen/mod.rs`)
identically to the two sibling module-dedup knobs (`hierarchy_sequential_module_dedup`
+ node-id / e-graph), running after structural and combinational dedup. Default `false`
⇒ every existing design byte-identical (`tests/snapshots.rs` 6/6 untouched). Rules-first
gate: `delay2_leaf` fixtures (a two-cycle delay line, one built with a redundant `~~in`
D-cone so it is structurally distinct but sequentially equivalent) merge to one
definition with the knob on while `dedup_modules` (structural) and `dedup_semantic_modules`
(stateful-skip) both leave them as two.

## 2026-06-16 — `union soft` up-opt matrix gate — `SV-VERSION-TARGETING.3b.2b`

Industrialized the `.3b.2a` `union soft` up-opt into `tool_matrix --sv-version-gate`
(a tenth scenario), closing the `SV-VERSION-TARGETING` tree. The non-obvious design
choices:

- **The Verilator-only tool plan is a pure function of the scenario config, not a
  `Scenario` flag.** A scenario that emits a `union soft` overlay *inherently* forces
  Yosys/Icarus to a no-op (they reject the syntax — decision `0010`), so the decision
  belongs to the config, not to an independent field that could drift from it. The
  predicate `scenario_emits_soft_union_overlay = soft_union_slice_prob > 0.0 &&
  sv_version.permits(Sv2023)` is computed once in `run_module_scenario` and threaded
  as `verilator_only` exactly where `verilator_language` already flows (resume →
  materialize → `run_module_tools`). Below 2023 the overlay down-gates to a plain
  slice every tool accepts, so the `permits(Sv2023)` half of the predicate is
  load-bearing — a 2012/2017 soft-union scenario would (correctly) NOT be Verilator-only.

- **Coverage evidence is *actual emission*, not the knob.** The up-opt fact must not
  be lit by a scenario that merely *requested* the overlay — a seed could produce no
  qualifying low-bits slice, Verilator would still pass (plain SV), and the fact would
  be a false positive. So `ModuleReport.emitted_soft_union_overlay` is set from
  `prepared.sv_text.contains("union soft")` (the emitted text), and the fact requires
  it. Bonus: the banked report then visibly shows which modules carried the up-opt.

- **Rejected: a `Metrics` overlay counter.** The repo's established pattern for "light
  a coverage fact from a by-construction signal" is a `Metrics` field (e.g.
  `num_operator_gates_with_duplicate_operands` for the signoff sweep). But `Metrics`
  is projected verbatim into the `--introspect` document, so adding a field forces an
  introspection schema MINOR bump (`1.3 → 1.4`) touching the schema doc + the
  MCP/introspect assertions + DUT-facing output — a wide, user-facing surface change
  for a matrix-scoped concern. The matrix-local `ModuleReport` bool is the
  minimum-blast-radius choice and is *stronger* evidence (text emission vs a marker
  count that's populated regardless of `sv_version`).

- **`all_yosys_invocations_ok(&[])` is vacuously `true`** (`.all()` over an empty
  iterator), so a Yosys-no-op union module would have falsely lit the Yosys-requiring
  `saw_sv_version_2023_targeted_acceptance` fact. Added a `!module.yosys.is_empty()`
  guard to the general per-version lighting. This only affects empty-Yosys modules
  (which did not exist in the sweep before this leaf), so the existing 9-scenario
  behavior is unchanged — but the fact is now honest about "Yosys actually ran clean".

- **Gate-scoped, not a new `ScenarioSet`.** The task asked for "a dedicated
  `--sv-version-gate` up-opt scenario", so the overlay scenario lives inside the
  existing `SvVersionSweep` and `compute_coverage_gaps`' early-return arm gained one
  more required fact. The sweep is now heterogeneous (9 Yosys-running common-floor
  scenarios + 1 Verilator-only up-opt scenario), which is exactly why the tool plan
  had to become per-scenario.

## 2026-06-16 — Semantic introspection — the MCP `analyze` surface — `SEMANTIC-INTROSPECTION-EXPANSION.2b.2`

Wired the `.2b.1` analysis core to the agent surface (schema `1.3` + the pure MCP
`analyze` tool). Surface-wiring decisions worth carrying forward:

- **`-32602` is a JSON-RPC protocol error, not a tool-level `isError`.** Unknown
  `query`/`target` returns `err(id, INVALID_PARAMS, …)` (a top-level `error`
  object) — matching `prompts/get`, the existing precedent for invalid arguments
  — *not* `tool_error` (which is a successful tool call carrying
  `isError: true`). So `run_analyze` owns `id` and returns the *full* JSON-RPC
  response, unlike the `ok(id, tool_text/tool_error)` arms. Rule of thumb: a
  malformed *request* (bad query/target) is `-32602`; a *tool-domain* problem
  (e.g. a non-DUT lane, which is a valid request the tool can't serve) is
  `tool_error`.
- **Unknown-target detection rides the pure layer's totality.** `run_analyze`
  doesn't re-resolve the target; it builds the cone and treats "explicit target +
  empty `results`" as unknown (→ `-32602`). This works precisely because the
  `.2b.1` builders are total and only an *unknown* target yields zero cones (a
  known-but-empty cone still yields one) — so no error type crosses the pure
  boundary.
- **The analysis is cached on the artifact, not in a side map.** Added
  `CachedArtifact.analyses: BTreeMap<query, Value>` (default-empty ⇒ the two
  existing constructions just gained one field). `run_analyze` caches the base
  artifact (so its `sv`/`introspection` resources exist) via the existing
  `cache_artifact`, then `get_mut`s the entry to insert the analysis — so
  `anvil://artifact/<run_id>/analysis/<query>` is a sibling of the artifact's
  other resources and `parse_artifact_uri` needs **no** change (it splits on the
  first `/`, yielding `part = "analysis/<query>"`, matched by a
  `part.starts_with("analysis/")` arm).
- **DUT-only by construction.** The cone walks the DUT IR `Module`/`Design`
  graph; the microdesign/frontend lanes are source-level/oracle artifacts with no
  gate graph, so a non-DUT `lane` is rejected before the DUT knob parse. A
  derived query *kind* for those lanes (if ever wanted) is a separate future
  surface, not a lane arg on `analyze`.
- **Double-generation is acceptable and consistent.** `run_analyze` generates the
  `Module`/`Design` once for the cone, then `cache_artifact` regenerates the SV
  from the doc's request echo — exactly the pattern `generate`/`introspect`
  already use (deterministic ⇒ identical artifact). Not optimised; correctness +
  consistency over a micro-saving on a read-mostly tool.
- **Schema bump is a string, not a shape change.** Only `SCHEMA_VERSION` and the
  ~6 `"1.2"` test assertions move to `"1.3"`; the default `IntrospectionDocument`
  is byte-identical otherwise, so snapshots stay 6/6 and the default
  `--introspect` document is unchanged for a `1.2` consumer (the cone is a
  *separate* `DerivedAnalysisDocument`, reached only via `analyze`).

## 2026-06-16 — Semantic introspection — pure support-cone analysis core — `SEMANTIC-INTROSPECTION-EXPANSION.2b.1`

Implemented the `.2a` design as the pure module `src/introspect/analyze.rs`.
Engineering decisions worth carrying forward (the `.2a` design fixed the struct
shape; `.2b.1` fixed the exact traversal semantics):

- **"+ flop D-cones" resolved into addressable targets, not transitive
  recursion.** The `.2a`/`MEMORY.md` phrase "DFS over … + flop D-cones" was
  ambiguous between (a) crossing flops to find transitive sequential support and
  (b) making each flop's `D` a separately *queryable* combinational cone. `.2b.1`
  chose **(b)**: the support cone is purely **combinational** — `FlopQ` is a
  register-boundary leaf (recorded in `support_flops`, the walk stops there), and
  the cone feeding a flop's `D` is reached as the distinct target `"flop:<id>"`.
  This gives one consistent stopping rule (everything stops at a register/leaf
  boundary), a clean `cone_depth` = combinational depth, and matches the standard
  meaning of "support cone". Transitive-through-flops reachability is left to a
  future `input_reach`/sequential kind, not folded into the first query.
- **`MemRead`/`FsmOut` are opaque registered leaves with no support slot.** The
  `.2a` struct has exactly three support lists (inputs / flops / instance
  outputs). Memory reads and FSM Moore outputs are opaque sequential leaves
  (like `FlopQ`) but have no list, so `.2b.1` **terminates** the cone at them
  (counted in `cone_nodes`, listed nowhere) and documents it as a boundary +
  a recorded future query kind — rather than silently mis-bucketing them into
  `support_flops`. They are default-off (`memory_prob`/`fsm_prob` = 0.0), so the
  common DUT cone is fully covered by the three lists.
- **Unknown-target signalling stays in the pure layer's shape, not an error
  type.** A resolvable target always yields exactly one `SupportCone` (even when
  its support sets are empty — an undriven output or a `d = None` flop); only a
  genuinely unknown target yields **zero** cones. So `results.is_empty()` (for an
  explicit `Some(target)`) unambiguously means "unknown target", which the
  `.2b.2` MCP layer maps to `-32602` — no `Result`/error enum needed across the
  pure boundary, keeping the builders total and easy to test.
- **Instance-leaf naming is variant-specific by necessity.** A bare `Module`
  carries no child definitions, so `module_support_cones` names instance-output
  leaves `"<instance>.port<id>"`; `design_support_cones` resolves the child
  module from `Design.modules` and emits `"<instance>.<child-output-port-name>"`.
  Leaf DUT modules have no instances, so the module-variant fallback is rarely
  exercised; the design variant is the useful path. One shared core walker takes
  an instance-leaf formatter closure so the DFS itself is not duplicated.
- **Hand-built in-crate tests, not generated modules.** Because `analyze.rs` is
  in-crate, its `#[cfg(test)]` module can build `Module`s with `..Module::default()`
  (the CSE bookkeeping fields are `pub(crate)`), so cone correctness is asserted
  against modules with a *known* graph (exact `support_inputs`/`cone_nodes`/
  `cone_depth`) — far stronger than poking at an opaque generated module. Mirrors
  the `src/ir/types.rs` test helpers.

## 2026-06-16 — Semantic introspection — support-cone impl design-detail — `SEMANTIC-INTROSPECTION-EXPANSION.2a`

Design-detail leaf for `.2`. Resolves decision `0011`'s three open questions
before code and pre-splits `.2` → `.2a` (this design) + `.2b` (impl). Grounded in
a fresh read of `src/introspect/mod.rs` (the `IntrospectionPayload` /
`IntrospectionDocument` / `RequestEcho` / `content_run_id_for_knobs` surface) and
`src/mcp/mod.rs` (the pure-tool dispatch + `CachedArtifact` cache). Docs-only.

- **The derived-relation types (a new pure `src/introspect/analyze.rs`,
  serde + `Default`, BTreeSet→sorted-Vec so the JSON bytes are a pure function):**
  ```rust
  pub struct DerivedAnalysis { pub query: String, pub results: Vec<SupportCone> }
  pub struct SupportCone {
      pub target: String,                       // output port name (or "flop:<id>")
      pub support_inputs: Vec<String>,          // primary-input port names (sorted)
      pub support_flops: Vec<u32>,              // flop ids (sorted)
      pub support_instance_outputs: Vec<String>,// "<inst>.<port>" (sorted, design only)
      pub cone_nodes: usize,                     // # IR nodes in the transitive fan-in
      pub cone_depth: usize,                     // max combinational depth to a leaf
  }
  ```
  Pure builders `module_support_cones(m: &Module, target: Option<&str>) ->
  DerivedAnalysis` (+ a `design_*` variant) do a memoized DFS over the existing
  `Module.nodes` operands + `drives` (+ flop D-cones; child-instance outputs are
  **leaves** — see cone semantics). **No IR field, no generator change** — the
  analysis is a pure function of the already-emitted IR (the `coverage_gaps`
  project-don't-recompute precedent; `metrics::compute` already walks this graph).
- **The `query`-kind enum (Q1 of `0011`):** `output_support` is the first kind.
  Future kinds slot into the same registry: `input_reach` (symmetric fanout),
  `flop_reset_provenance`, `module_reachability`. The MCP `analyze` tool rejects
  an unknown `query` with `-32602` (the `prompts/get` validation precedent).
- **`target` addressing (Q1):** an output **port name** string; absent/`null` ⇒
  **all outputs**. Flop D-cones address as `"flop:<id>"`. Keep it stringly +
  simple; an unknown target is `-32602`.
- **Default `introspect` stays lean (Q2):** the existing `IntrospectionPayload`
  is **untouched** — no `analysis` field is added to the default document, so
  `--introspect` / the `introspect` tool keep their current shape (only the
  `schema_version` string bumps). The derived analysis is reached **only** via
  the new MCP `analyze` tool, which returns a standalone `DerivedAnalysisDocument`
  reusing the same envelope (`schema_version` `1.3`, `RequestEcho` + content
  `run_id`, the artifact `ResourceRef`) with an `analysis: DerivedAnalysis`
  payload. Big cones: inline for the first cut (a cone is bounded by module size);
  a `ResourceRef` spill-over above a node-count threshold is a noted `.2b` option,
  not first-cut.
- **Cross-instance cone semantics (Q3):** the cone **stops at the instance
  boundary** — a child-instance output is reported as a support *leaf*
  (`support_instance_outputs`), not recursed into. Recursing through the child
  (a `depth`/`recurse` arg) is a future kind. This keeps the first query a clean,
  single-module-graph walk that also works per-module inside a design.
- **Schema `1.2 → 1.3` (MINOR, additive):** bump `SCHEMA_VERSION` + add the
  `DerivedAnalysis`/`SupportCone`/`DerivedAnalysisDocument` types + a schema-doc
  `1.3` changelog/section. **DUT `.sv` stays byte-identical** (introspect output
  is not in `tests/snapshots.rs`); the bump only changes the `--introspect`
  document's `schema_version` field + the ~5 `"1.2"` test assertions (the exact
  `.2b.1` `1.1→1.2` procedure). The pure tools (`generate`/`introspect`/…) stay
  pure; `analyze` is a new **pure** tool (no FS, no spawn, cached like the rest).
- **`.2b` impl scope:** `src/introspect/analyze.rs` + the types + the schema bump
  + the MCP `analyze` tool + dispatch + `tools/list` entry + a `DerivedAnalysis`
  resource (`anvil://artifact/<run_id>/analysis/<query>`) + lib proofs (cone
  correctness on a hand-built module; determinism; unknown-target/-query errors)
  + book(`agent-mcp.md`)/USER_GUIDE/schema-doc + a KM fact. Default-off / DUT
  byte-identical (snapshots 6/6). Pre-split `.2b` → `.2b.1` (analyze module +
  types, lib-tested) + `.2b.2` (MCP tool + schema + docs) if broad.

## 2026-06-16 — SV-version targeting — live `union soft` up-opt — `SV-VERSION-TARGETING.3b.2a`

Implemented the `.3b.1` mechanism exactly. The first ANVIL emission that diverges
across `--sv-version` targets. Engineering notes + gotchas worth carrying forward:

- **The decision lives in the IR, the rendering in the emitter** (mirrors
  `aggregate_layout`). A pure emitter cannot roll an RNG, so the per-gate choice
  is a seeded `gen_bool` in `src/ir/soft_union.rs`, rolled at the `generate_module`
  / `generate_design` call sites (guarded by `> 0.0` so the default draws nothing
  ⇒ byte-identical stream), and recorded in `Module.soft_union_slice_gates`. The
  emitter reads the marker + `SvVersion::permits(Sv2023)`.
- **`Module` has no `serde` derive** (`Debug, Clone, Default` only), so adding a
  field is free of snapshot/serde risk — snapshots compare the *emitted SV string*,
  not a `Module` serialization. The marker is also kept out of
  `canonical_module_signature` (identity-invariant, like `aggregate_layout`).
- **`union soft` is not a concatenation** ⇒ it is *not* an `AggregateKind` (that
  machinery is sound only for bit-equivalent regroupings). The up-opt is instead a
  faithful *alternative rendering of a low-bits `Slice`*: `u.w = src; gate = u.n`
  ≡ `src[hi:0]` because packed-union members are LSB-aligned. The qualifying gate
  is `Slice { hi, lo: 0 }` over a non-constant source strictly wider than the
  slice (so the two members differ — the 2023 *soft* requirement; an equal-width
  member set is a plain `union packed`).
- **Emission shape (probe-verified before writing the emitter):** an *anonymous*
  `union soft { logic [W-1:0] w; logic [SW-1:0] n; } <gate>__u;` variable +
  *continuous* `assign`s is Verilator `--lint-only --language 1800-2023`
  warning-clean (no `-Wall` needed) and `--binary`-correct (`y=5` for `a=0xA5`).
  No typedef needed; the `<gate>__u` name is unique because gate names are.
- **Gotcha — the divergence proof can't be an integration test.** Hand-building a
  `Module` needs the crate-private CSE bookkeeping fields (`gate_instances` /
  `const_instances`), so `..Module::default()` is not constructible across the
  crate boundary. The divergence/down-gate proof therefore lives in-crate
  (`src/ir/soft_union.rs` tests); the integration file carries a pointer comment.
- **Gotcha — `--config` JSON needs every non-`#[serde(default)]` field.** `Config`
  requires `seed` and `max_nodes_per_module` (among others). The reliable recipe
  to drive the knob from the CLI is `--dump-config > c.json`, patch the few keys
  (`soft_union_slice_prob`, `sv_version`, `gate_struct_weight`, widths), then
  `--config c.json`. Recorded in the KM fact's `reverify`.
- **Where overlays come from:** the structured-gate surface
  (`gen/cone/terminals.rs::pick_slice_gate`) emits `Slice { lo: 0..=3 }`, so only
  ~25% are low-bits; cranking `gate_struct_weight` + wide widths makes them
  plentiful (47/48 seeds produced overlays in the bank).

## 2026-06-16 — SV-version targeting — soft-union up-opt mechanism (impl design-detail) — `SV-VERSION-TARGETING.3b.1`

Design-detail leaf for `.3b`. Resolves the mechanism open question decision
`0010` left for `.3b` ("projection shape: port-boundary union fold vs lower-risk
internal-only `union soft` overlay") and pre-splits `.3b` → `.3b.1` (this design)
+ `.3b.2` (impl). Grounded in a fresh read of `src/ir/aggregate.rs`,
`src/emit/sv.rs` (`render_gate` `Slice` arm at `sv.rs:1040`, gate decl/assign
region), and `src/config.rs`. Docs-only.

- **The aggregate-projection mechanism `0010` floated is rejected — a union is
  not a concatenation.** The Phase 5b boundary-aggregate machinery
  (`src/ir/aggregate.rs`) is sound *only* because a packed `struct`/array is
  LRM-bit-equivalent to the **concatenation** of its members: the projection is a
  bijective, semantically-empty, `canonical_module_signature`-invariant
  regrouping (proven by `canonical_signature_is_invariant_under_projection`). A
  packed `union` **overlays** members (width = `max`, all members alias the same
  bits), so it is *not* concatenation-equivalent and a `union` `AggregateKind`
  sibling would break that invariant. Rejected.
- **The port-boundary union fold is also rejected for the first up-opt.** Folding
  N input ports into one `union soft` boundary port changes the module
  *interface* (N independent inputs → one aliased input) — a genuinely
  behaviour-altering construction that would have to happen at generation time
  (IR/interface + width-adaptation), a large blast radius. Deferred as possible
  later breadth (nothing retired).
- **Chosen mechanism: an emitter-level, `sv_version >= Sv2023`-gated, default-off
  alternative rendering of a *proper low-bits* `Slice` gate as an internal
  `union soft` overlay.** For a `GateOp::Slice { hi, lo: 0 }` over a non-constant,
  multi-bit source of width `W` (with `hi < W-1`), instead of `src[hi:0]` emit:
  ```systemverilog
  typedef union soft { logic [W-1:0] w; logic [hi:0] n; } <u>_t;
  <u>_t <u>;
  assign <u>.w = <src>;
  // the slice wire is then driven by:
  assign <slice_wire> = <u>.n;
  ```
  This is **behaviour-preserving**: packed-union members are LSB-aligned, so
  `<u>.n` (width `hi+1`) equals `<src>[hi:0]` — verified by the `.3a` probe
  (`u.m1`, a 1-bit member of `u = 8'hA5`, read as `1` = bit 0; `verilator
  --binary` → `y=a5`). It is genuinely 2023 (heterogeneous-width members are
  legal only as `union soft`, IEEE 1800-2023 §7.3.1), Verilator-accepted, and
  surgical (one extra decl + drive in the gate decl/assign region; `render_gate`
  stays a pure expression and the member ref flows through the existing per-gate
  name machinery, mirroring how `MemRead`/`FsmOut` emit a decl/block + an opaque
  read name).
- **Why a `Slice` low-bits overlay is the right first cut:** it is the *faithful*
  use of a heterogeneous soft union (write the wide member, read the narrow one =
  low-bits extraction), so it needs no new IR node and no interface change —
  lowest blast radius while still emitting a real 2023 construct. `lo != 0`
  slices do not qualify (union members are LSB-aligned, not arbitrary ranges); a
  proper low-bits slice (`lo == 0`, `hi < W-1`) is required.
- **`.3b.2` impl scope:** a default-off `Config` knob (working name
  `soft_union_slice_prob`, default `0.0`) rolled rules-first at the slice-emission
  decision; the `sv_version.permits(Sv2023)` gate AND the knob both required; the
  emitter overlay path; `tests/sv_version.rs` extended to show **divergence** at
  `Sv2023` when the knob fires (the byte-identity corpus stays byte-identical at
  the default and across versions when the knob is off);
  `tests/sv_version_downstream.rs` extended to prove `verilator --language
  1800-2023` accepts/builds the overlay; a dedicated matrix up-opt scenario +
  `saw_sv_version_2023_soft_union_upopt` fact that requires Verilator
  matching-mode acceptance and records Yosys/Icarus as a no-op (not a failure);
  book(knobs + sv-version)/USER_GUIDE/README/ROADMAP + a KM fact. Snapshots 6/6
  must stay byte-identical (knob default-off).
- **Open verification risk for `.3b.2`:** confirm the overlay is **Verilator
  `--lint-only` warning-clean** (ANVIL folds warning→failure), since the `.3a`
  probe's `-Wall` warnings were toy-unused-signal artifacts; the real emitted
  overlay drives `w` and reads `n` (both used), so it should be clean — to be
  banked, not assumed.

## 2026-06-16 — SV-version targeting — first up-opt design (soft packed union) — `SV-VERSION-TARGETING.3a`

Design leaf for `.3`, the first version-distinctive *up-opted* construct. Splits
`.3` into `.3a` (this design) + `.3b` (impl). Docs-only; full rationale +
rejected alternatives in decision [`0010`](docs/decisions/0010-sv-version-first-upopt-soft-packed-union.md).
Grounded in a direct probe of the installed Verilator 5.046 / Yosys 0.64 /
Icarus 13.0.

- **The empirical finding that shaped everything: the installed tools do not
  enforce 1800-version *acceptance*.** Verilator 5.046 accepts every supported
  construct — and reserves keywords — identically across `--language 1800-2012`
  / `1800-2017` / `1800-2023` (probed: `soft` and `implements` as identifiers
  fail at *all three* modes). So **no construct exists for which 2012 rejects and
  2023 accepts**. Yosys/Icarus expose no 1800 selector and parse a fixed
  conservative subset. The up-opt's teeth are therefore (a) LRM correctness,
  (b) ANVIL's construction-time down-gating guarantee, and (c) matching-mode
  acceptance (`verilator --language 1800-2023`), **not** tool-side version
  rejection. The design says this out loud rather than over-claim.
- **First up-opt = heterogeneous-width packed `union soft` (IEEE 1800-2023
  §7.3.1)**, a new default-off aggregate projection gated on `sv_version >=
  Sv2023`, sibling of `AggregateKind::StructPacked`/`ArrayPacked`. Chosen because
  it has **real down-gating teeth**: a *non-soft* packed union with
  heterogeneous-width members is illegal pre-2023 and **all three tools reject
  it** — Verilator's own diagnostic quotes the standard:
  `Hard packed union members must have equal size (IEEE 1800-2023 7.3.1)`. The
  soft form genuinely elaborates (`verilator --binary` → `y=a5`). The down-gate
  fallback at `< 2023` is the existing packed `struct` projection ⇒ default
  byte-identical.
- **Downstream proof handling.** Verilator `--language 1800-2023` (accept +
  `--binary`) is the primary proof; Yosys/Icarus reject the `union soft` syntax,
  so for the up-opt scenario they are a **recorded no-op, not a failure** (the
  Icarus-beyond-`-g2012` path `0009` already authorized, extended to Yosys). The
  existing `--sv-version-gate` `saw_sv_version_2023_targeted_acceptance` fact
  requires *Yosys-clean*, so the union scenario gets a **dedicated** up-opt fact
  (working name `saw_sv_version_2023_soft_union_upopt`) requiring only Verilator
  matching-mode acceptance — `.3b` work.
- **`.3b` open questions** (resolved at `.3b`/`.3b.1`): port-boundary union fold
  (union width = max member width, changes the input bit-budget) vs lower-risk
  internal-only `union soft` overlay over an existing wide signal; the exact
  `AggregateKind` variant + `render_aggregate_typedef` emit site + the
  `permits(Sv2023)` gate; the new union-projection knob + default-off; the matrix
  up-opt scenario/fact + Yosys/Icarus no-op recording; and the
  `tests/sv_version.rs` update (byte-identity must now show **divergence** at 2023
  when the knob fires).

## 2026-06-16 — SV-version targeting — repo-owned per-version acceptance gate — `SV-VERSION-TARGETING.2b.2b`

Second half of `.2b.2`: industrializes the `.2b.2a` focused proof into a
coverage-gated `tool_matrix` lane, closing `.2` (plumbing + down-gating +
per-version acceptance axis). All code is in `src/bin/tool_matrix.rs`.

- **Gate shape mirrors `--signoff-knob-sweep-gate`.** New `--sv-version-gate`
  → `ScenarioSet::SvVersionSweep`, mutually exclusive with the other gates,
  auto-`fail_on_coverage_gap`, units/scenario floor
  `SV_VERSION_SWEEP_MIN_UNITS_PER_SCENARIO = 2`. Chose the established gate
  precedent over a `--sv-version`-parameterized run of an existing set (decision
  `0009`'s open question) because the per-version *coverage facts* need a
  dedicated `compute_coverage_gaps` contract, exactly as the knob-sweep gate.
- **Scenario set: per-version × {comb leaf, seq leaf, hierarchy design}.** Nine
  `Interleaved` scenarios reusing the well-trodden downstream-clean recipes
  (`share_heavy_comb_only_config`, `motif_heavy_sequential_config`,
  `phase4_recursive_canonical_module_signature_focus_config`) with
  `cfg.sv_version` set per target. The hierarchy design is load-bearing: it is
  the only scenario that exercises the design-path emit
  (`to_sv_in_design_versioned`) and the design-path Verilator-language threading
  (`run_verilator_design` selector). Strategy breadth is **out of scope** (the
  other gates own it), so `compute_coverage_gaps` returns for this set *before*
  the construction-strategy loop — which is why an Interleaved-only sweep is
  valid and the gap test asserts an empty strategy set yields no strategy gap.
- **Threading.** A single `version_targeted: bool` (= `scenario_set ==
  SvVersionSweep`) flows from `main` through `run_scenario` into the two
  scenario runners; each derives `sv_version = scenario.config.sv_version` and
  `verilator_language = verilator_language_for(scenario, version_targeted)`
  (`Some(ieee_standard())` under the gate, else `None`). `sv_version` threads to
  the emit sites; `verilator_language` threads to `run_{module,design}_tools`
  and the resume paths. **The emit switch is byte-neutral today** —
  `to_sv_versioned(m, Sv2012) == to_sv(m)` (`.2b.1`) and every non-gate scenario
  is `Sv2012` — so the default/phase/signoff matrix runs stay byte-identical;
  the gate's only observable effect is the Verilator `--language` argv (and the
  emitted SV across the three targets is identical until the up-opting leaf
  `.3`).
- **Acceptance fact is honest-by-gating.** `light_sv_version_acceptance` lights
  `saw_sv_version_<year>_targeted_acceptance` + the umbrella only when
  `version_targeted` AND Verilator actually ran-and-succeeded AND Yosys is clean
  — so the fact means "accepted in the matching `--language` mode," never "ran
  at the tool's default language." Outside the gate the facts stay false (no
  false positives leak into other sets). Verified: the banked report's per-
  scenario Verilator argv carries the matching `--language 1800-20xx` and all
  four facts are lit.
- **Banked clean** at `/tmp/anvil-sv-version-gate-r1` (9 scenarios, 18 units,
  `coverage_gaps = []`, Verilator 18/0, Yosys without-abc 18/0, with-abc 18/0).

## 2026-06-15 — SV-version targeting — per-version downstream acceptance proof — `SV-VERSION-TARGETING.2b.2a`

First half of `.2b.2` (split here into `.2b.2a` downstream selector + focused
proof, and `.2b.2b` the repo-owned `tool_matrix` gate). Proves decision
`0009`'s "per-version acceptance" half: the version-targeted corpus is
**accepted by a downstream tool in its matching standard mode**.

- **Verilator `--language` selector probed against the real binary first.**
  Verilator 5.046 `--help` lists both `--default-language <lang>` and
  `--language <lang>` ("Default language standard to parse"); both accept
  `1800-2012`/`2017`/`2023` and lint a generated module clean (exit 0, no
  warning). Chose **`--language <std>`** (the documented standard selector).
  No aspirational flag — the spelling is verified, per the `.2a` open question.
- **`run_verilator` / `run_verilator_design` gain `language: Option<&str>`**
  (`src/downstream/mod.rs`). `Some("1800-2017")` prepends `--language
  1800-2017`; **`None` reproduces today's exact argv byte-for-byte**. The four
  existing callers (`validate()` ×2, `tool_matrix` ×2) pass `None`, so every
  banked report + the agent `validate` tool are unchanged. `SvVersion::
  ieee_standard()` (added `.2b.1`) supplies the `"1800-20xx"` string.
- **Focused real-tool gate `tests/sv_version_downstream.rs` (`#[ignore]`).**
  Mirrors the diff-sim / parity-gate precedent (tool-dependent ⇒ default
  `cargo test` doesn't need the tools). Over a leaf corpus (comb / seq /
  structured / memory / fsm) and a hierarchy design, emits at each `SvVersion`
  and asserts Verilator `--language 1800-{2012,2017,2023}` is warning-clean
  (`ToolInvocation.success` already folds warning-as-failure), and that Icarus
  `-g2012` accepts the subset for every target. **Banked clean** against
  Verilator 5.046 + Icarus 13.0: `2 passed` in 6.18s.
- **Why a focused `#[ignore]` gate, not the matrix yet.** It delivers the
  per-version *acceptance proof* in a small, bankable slice; the heavier
  repo-owned industrialization (a `--sv-version-gate` + `ScenarioSet::
  SvVersionSweep` + a `saw_sv_version_targeted_acceptance` coverage fact under
  `coverage_gaps` enforcement, threading the language into the matrix's
  per-scenario Verilator run) is the follow-on `.2b.2b`. Splitting isolates the
  byte-identical downstream API change from the matrix surgery.

## 2026-06-15 — SV-version targeting — knob + emitter capability bound — `SV-VERSION-TARGETING.2b.1`

First code slice of `.2b` (implement decision `0009` per the `.2a` design). Adds
the `--sv-version` knob and threads it into the emitter as a down-gating
capability bound. **Default-off / byte-identical**: snapshots 6/6 untouched, and
the new `tests/sv_version.rs` proves every target is byte-identical over the
current subset.

- **`SvVersion` enum (`src/config.rs`).** `Sv2012 < Sv2017 < Sv2023` (`Ord`),
  `#[derive(... Default ...)]` with `#[default] Sv2012`. Bare-year CLI/serde
  spelling via per-variant `#[value(name = "2012")]` + `#[serde(rename =
  "2012")]` (a decimal-leading token isn't a kebab identifier). `permits(self,
  introduced) = self >= introduced` is the down-gating bound; `ieee_standard()
  → "1800-20xx"` is staged for the `.2b.2` Verilator language axis.
  `Config::sv_version` is `#[serde(default)]` (old config JSON without the key
  reads back as the floor) + `Overrides`/`apply_cli_overrides`. No new
  `validate` rule (an enum is always valid).
- **Emitter (`src/emit/sv.rs`).** New `to_sv_versioned` /
  `to_sv_in_design_versioned` / `to_sv_design_versioned`; the historical
  `to_sv*` delegate with `SvVersion::default()`, so every existing caller
  (snapshots, book examples, tests, MCP, umbrella) is byte-identical.
  `to_sv_with_modules` gains the `sv_version` param (used in the `info!` trace —
  no byte impact). The bound currently gates **nothing**: the whole subset is
  1800-2012-valid, so down-gating to any target removes nothing — the explicit,
  testable down-gating guarantee over the current subset. `.3`'s first up-opt
  is the first site to call `sv_version.permits(...)` before emitting.
- **Threaded the bound at every DUT emit site:** `src/main.rs` (stdout +
  `--out` design/module paths), `src/introspect/mod.rs` (`sv_len`),
  `src/mcp/mod.rs` (`generate`), `src/umbrella/mod.rs` (DutLane — captures
  `sv_version` before `cfg` moves into the generator). All byte-identical at
  the default. (`tool_matrix` deliberately deferred to `.2b.2`, which pairs the
  version with the per-version downstream `--language` axis.)
- **Introspection schema MINOR bump `1.1 → 1.2`.** `sv_version` surfaces for
  free (serde projection of `Config` in `request.knobs`); bumped
  `SCHEMA_VERSION` + the schema-doc changelog/version lines + the five `"1.1"`
  test assertions (1 in `introspect`, 3 in `mcp`, plus the schema-doc self-check
  prose). `--dump-config` shows `"sv_version": "2012"` automatically.
- **Why threading is a *parameter*, not an IR field.** A standard target is
  global emission policy, not a per-module structural fact — so it stays off
  `Module`/`Design`, leaving CSE keys, `canonical_module_signature`, validators,
  and every Module-serialization surface untouched (contrast the per-module
  `aggregate_layout` *structural* annotation).
- **Validation.** `cargo check --all-targets` / `cargo clippy --all-targets -D
  warnings` / `cargo fmt --check` clean; `cargo test --lib` 405/0; `tests/
  snapshots` 6/6; `tests/sv_version` 2/2; CLI smoke: default == `--sv-version
  2012` == `2023` byte-for-byte, `--dump-config`/`--introspect` carry the field
  + schema `1.2`, bad value rejected with the possible-values list.

## 2026-06-15 — SV-version targeting — implementation design detail — `SV-VERSION-TARGETING.2a`

Design-detail leaf for `.2` (implement decision `0009`). **No source change.**
Grounds the `.2b` implementation in the *real* `src/config.rs` / `src/emit/sv.rs`
/ `src/introspect/mod.rs` / `src/downstream/mod.rs` / `src/bin/tool_matrix.rs`
code and resolves decision `0009`'s five open questions before any code lands.
`.2` is split here into `.2a` (this design detail) + `.2b` (impl), and `.2b` is
pre-split into `.2b.1` (knob plumbing + emitter capability bound) + `.2b.2`
(per-version downstream acceptance axis), per the split-before-implement
discipline (the `.3b.1`/`.3b.2` precedent).

- **1 — enum + the byte-identical floor default.** A `SvVersion` enum in
  `src/config.rs` with variants `Sv2012`, `Sv2017`, `Sv2023`, deriving
  `Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize,
  Deserialize, clap::ValueEnum`. **`PartialOrd`/`Ord` in declaration order**
  (`Sv2012 < Sv2017 < Sv2023`) is load-bearing: capability checks read
  `target >= SvVersion::Sv2017`. CLI + serde value spellings are the **bare
  years** via per-variant `#[value(name = "2012")]` (clap) + `#[serde(rename =
  "2012")]` — a decimal-leading token is not a valid kebab identifier, so a
  per-variant name is the clean spelling (the `IdentityMode` ValueEnum-alias
  precedent). So `--sv-version 2017` parses and `--dump-config` prints
  `"sv_version": "2012"`. **Default = `Sv2012`** (`#[serde(default =
  "default_sv_version")]` so old config files still deserialize). Rationale:
  ANVIL's entire current emitted subset (`logic`/`always_ff`/`always_comb`/
  packed `struct`/packed arrays/`typedef`/`localparam`) is valid in IEEE
  1800-2012, so (a) the default reproduces today's emission byte-for-byte, and
  (b) down-gating *to the floor* is a provable **no-op** — the strongest
  statement of the guarantee. This finalizes decision `0009`'s "working name
  `Sv2017`": 2012 is the honest floor, and because no version-distinctive
  construct exists yet all three targets are byte-equal today regardless, so the
  label choice is free and 2012 is the most defensible.

- **2 — where the bound lives + how it threads to the emitter.** The target is a
  global **emission capability**, so it threads to the emitter as a parameter,
  **not** onto the IR. Keeping `Module`/`Design` free of an emission-policy field
  preserves CSE keys / `canonical_module_signature` / validators / every
  Module-serialization surface untouched (a *standard target* is global emission
  policy, unlike the per-module `aggregate_layout` *structural* annotation, so a
  threaded parameter is the right tool and avoids any Module-serde ripple).
  Add version-aware entry points; the existing ones delegate with the floor
  default so **every current caller (main, tests, umbrella, mcp, examples) stays
  byte-identical**:
  - `pub fn to_sv(m) -> String` ⟶ `to_sv_versioned(m, SvVersion::default())`;
    new `pub fn to_sv_versioned(m, v: SvVersion) -> String`.
  - same for `to_sv_in_design` / `to_sv_design`.
  - `to_sv_with_modules` gains an `sv_version: SvVersion` param threaded to the
    construct sites; the bound is a `SvVersion::permits(introduced: SvVersion)
    -> bool` predicate (`self >= introduced`). **In `.2b.1` it gates nothing**
    (every emitted construct's introducing standard ≤ 2012), so output is
    byte-identical for all three targets — the down-gating guarantee over the
    existing subset, made explicit and testable. The mechanism is what `.3`
    consults for the first up-opted construct.
  - `src/main.rs`'s DUT stdout path + the umbrella DUT lane call the versioned
    entry with `cfg.sv_version`.

- **3 — the down-gating byte-identity proof (`.2b.1`).** A focused test asserts
  that over a representative corpus `to_sv_versioned(m, Sv2012) ==
  to_sv_versioned(m, Sv2017) == to_sv_versioned(m, Sv2023)` byte-for-byte —
  proving the current subset is a genuine common floor and the bound removes
  nothing today. `tests/snapshots.rs` is untouched (default path unchanged,
  6/6 byte-identical).

- **4 — introspection + dump-config.** Both are direct `serde` projections of
  `Config` (`--dump-config` = `serde_json::to_string_pretty(&cfg)`;
  `--introspect` carries `RequestEcho.knobs: Config`), so `sv_version` surfaces
  for free. Schema **MINOR bump 1.1 → 1.2**: `SCHEMA_VERSION` in
  `src/introspect/mod.rs:43`; the `schema_version` line + a `1.1 → 1.2` changelog
  entry in `docs/AGENT_INTROSPECTION_SCHEMA.md`; and the five hardcoded `"1.1"`
  test assertions (2 in `src/introspect/mod.rs`, 3 in `src/mcp/mod.rs`).
  `.2b.1` also audits that no test pins the exact full `Config` JSON field set
  (additive field is covered by the existing `#[serde(default)]` MINOR policy).

- **5 — per-version downstream acceptance axis (`.2b.2`).**
  `SvVersion::verilator_language_arg(self) -> &'static str` →
  `"1800-2012"|"1800-2017"|"1800-2023"`. `run_verilator*` gains an **optional**
  language selector: `None` = today's exact argv (byte-identical tool
  invocation); `Some(std)` prepends `--language <std>`. The accepted spelling
  (`--language` vs `--default-language`) is **probed against the installed
  Verilator at `.2b.2`** before wiring — no aspirational flag. Yosys stays
  `read_verilog -sv` (no finer selector → validates synthesizable-subset
  acceptance). Icarus `-g2012` is its newest generation: for the *current
  subset* (g2012-valid) the column runs for all three targets; a genuinely
  beyond-g2012 construct (only at `.3`) gates the iverilog column to a recorded
  no-op, never a failure. Harness shape: a focused `--sv-version-gate` +
  `ScenarioSet::SvVersionSweep` mirroring `--signoff-knob-sweep-gate` — sweep the
  three targets over a small scenario set, run Verilator in the matching
  `--language` mode, light a coverage fact (`saw_sv_version_targeted_acceptance`
  + per-version sub-facts) under `coverage_gaps` enforcement, bank clean.

- **Why split.** `.2` touches `config` + `main` CLI + `emit` + `introspect` +
  schema doc + (then) `downstream` + `tool_matrix` + six live docs + the book —
  more than one signoff-quality slice. `.2b.1` (knob + emitter bound, default
  byte-identical, snapshot-locked) isolates the surface every downstream consumer
  sees from the heavier tool-harness work in `.2b.2`, exactly as `.3b` isolated
  the byte-identical `bisimulation_partition` refactor from the cross-module
  feature.

## 2026-06-15 — SV-version targeting — design — `SV-VERSION-TARGETING.1`

Owner roadmap steering opened three new capability lanes; the recommended,
highest-leverage one (`SV-VERSION-TARGETING`) is activated and designed here, the
other two (`STRUCTURED-EMISSION-EXPANSION`, `SEMANTIC-INTROSPECTION-EXPANSION`)
registered `proposed`. Design/decision leaf, no source change. Full rationale +
rejected alternatives in decision
[`0009`](docs/decisions/0009-sv-version-targeting.md); engineering grounding here.

- **The gap (grounded).** No existing `sv_version` knob. The emitter
  (`src/emit/sv.rs`) produces a conservative synthesizable subset
  (`module`/`logic`/`always_ff`/`always_comb`, packed arrays, packed `struct`) —
  a 2012/2017 common floor valid across 1800-2012/2017/2023. The downstream gates
  (`src/downstream/mod.rs`) run at fixed implicit standards
  (`verilator --lint-only` tool-default, `yosys read_verilog -sv`,
  `iverilog -g2012`) with no version axis. So ANVIL can neither *guarantee*
  avoidance of newer constructs nor *deliberately exercise* a newer standard.
- **Two construction-time effects (both rules-first).** `--sv-version
  <2012|2017|2023>` (`Config::sv_version`): **down-gating** = never emit a
  construct newer than the target (a standard-validity guarantee for tools/flows
  pinned to that standard); **up-opting** = deliberately emit a higher standard's
  distinctive synthesizable constructs, each gated at construction time on
  `sv_version >= that_standard`. The version is a construction-time capability
  bound, never a post-hoc filter (`feedback_rules_first_generation`,
  core principle 2).
- **Byte-identical default.** The default value reproduces today's emission
  byte-for-byte (`tests/snapshots.rs` untouched). Selecting a different version
  is the only way to change output — opt-in like every ANVIL capability knob
  (`feedback_never_retire_strategies`).
- **No aspirational up-opts.** An up-opted construct lands only once proven
  accepted in the matching downstream tool standard mode (Verilator
  `--language 1800-20xx`; Yosys `-sv`; Icarus `-g2012` is its newest generation,
  so beyond-`g2012` corpora gate the iverilog column to a recorded no-op rather
  than a failure). Counterexamples retain seed + `sv_version` + knobs.
- **First increment (`.2`).** Plumbing + down-gating + the per-version acceptance
  axis over the *existing* subset (default byte-identical; introspection field +
  schema MINOR bump). The first version-distinctive up-opted construct is `.3`,
  design-first.
- **North-star fit.** Adds an explicit `sv_version` adversarial axis (ROADMAP
  steering gap 3) and version-targeted breadth (gap 1) — legal, standard-valid,
  unusual RTL that stresses version-specific downstream parser/elaborator paths.

## 2026-06-15 — Factor `bisimulation_partition` — refactor — `IDENTITY-DEEPENING.3b.2a`

First code slice of `.3b.2` (implement decision `0008`). Pure refactor of
`src/ir/compact.rs`: the "bucket → refinable partition → greatest-fixpoint
refinement" core of `merge_bisimilar_flops` is extracted into a **non-mutating**
`bisimulation_partition(&Module) -> Option<Vec<Vec<FlopId>>>`. The cross-module
whole-leaf-module equivalence check (`.3b.2b`) runs this *identical* refinement
on a throwaway combined module, so it must not be duplicated.

- **Byte-identical contract preserved exactly.** `bisimulation_partition` returns
  `None` precisely when the original hit its `!has_refinable` early-`return 0`,
  and `Some(classes)` with the identical final partition otherwise.
  `merge_bisimilar_flops` keeps its guards (knob / `< 2` flops / node-id+e-graph /
  settled-D) and its collapse + `finalize_flop_merge` tail unchanged, so when the
  partition is `None` it returns `0` without touching the module and otherwise
  collapses exactly as before. Verified: `tests/snapshots` 6/6 byte-identical and
  all 6 `merge_bisimilar_flops_*` gate tests green.
- **Why a separate slice.** Landing the refactor on its own — byte-identical,
  snapshot-locked — isolates the risky cross-module logic in `.3b.2b` from any
  chance of perturbing the proven flop-merge path (the `.2b` `finalize_flop_merge`
  precedent: refactor first, feature second). The helper is the foundation the
  combined-module `modules_bisimilar` check calls.

## 2026-06-15 — Cross-module sequential merge — design detail — `IDENTITY-DEEPENING.3b.1`

Design-detail leaf for `.3b` (implement decision `0008`). No source change. This
grounds the `.3b.2` implementation in the **real** `src/ir/dedup.rs` /
`src/metrics.rs` / `src/ir/compact.rs` code and resolves decision `0008`'s
central open question (the "cross-module cone-proof signature"). The headline:
**the cross-module proof needs no new engine, and no flop bijection.**

- **NO new proof engine — materialize a temporary combined `Module`.**
  `cone_proof` (`src/ir/compact.rs`) and `merge_bisimilar_flops` are
  module-local (they walk one `Module`'s `nodes`/`flops`). But
  `LeafEndpoint::PrimaryInput { port, width }` keys endpoints by `(port, width)`,
  **not** by `NodeId`. So to compare two candidate modules `A`, `B`, build a
  throwaway `combined: Module` = `A.nodes ++ B.nodes` (B's `NodeId`s offset by
  `A.nodes.len()`, B's internal operand / `FlopQ` / drive references remapped by
  the offset) and `A.flops ⊎ B.flops` (B's `FlopId`s offset, B's `flop.d`/`flop.q`
  NodeIds + `flop_domains` remapped). Because the interface base case (below)
  requires A and B to share input `PortId`s, B's `PrimaryInput{port,width}`
  nodes keep their `port`, and A's and B's primary-input endpoints **unify for
  free** inside `cone_proof` — no cross-module endpoint vocabulary, no
  `(ModuleTag, FlopId)` map needed (the `.3a` decision allowed for one; reading
  the code showed it is unnecessary). This is strictly less machinery than `0008`
  anticipated.
- **Reuse the `merge_bisimilar_flops` refinement via a factored
  `bisimulation_partition`.** `merge_bisimilar_flops` (`src/ir/compact.rs`
  1286–1431) already computes the coarsest stable partition (bucket by
  `(width, reset_kind, reset_val, domain)`; resetless pinned singleton; refine to
  fixpoint with the quotient `rep_map` threaded into
  `cone_proof(.., Some(&rep_map))`; fresh memos per iteration) and *then*
  collapses + `finalize_flop_merge`. `.3b.2` extracts the "bucket → refinable
  partition → greatest-fixpoint refine → `rep_map`" core into a **non-mutating**
  `bisimulation_partition(m: &Module) -> HashMap<FlopId, FlopId>` (or
  `Vec<Vec<FlopId>>`). `merge_bisimilar_flops` then = `bisimulation_partition` +
  its existing collapse/`finalize_flop_merge` tail, so it stays **byte-identical**
  (snapshots 6/6). The module-equivalence check calls `bisimulation_partition`
  on the `combined` module and uses the `rep_map` *without* collapsing.
- **NO flop bijection required — the sound, sufficient condition.** Run
  `bisimulation_partition(combined)` to a stable `rep_map`, then the verdict is:
  - **(i) interface base case:** A and B have identical input-port and
    output-port sets keyed by `(PortId, width)` (the same match
    `semantic_module_proof_body` enforces; reuse `emitted_data_input_ports_in` +
    `module.outputs`); AND
  - **(ii) output equality under the quotient:** for every output port `p`,
    `cone_proof(combined, driveA(p), Some(rep_map)) ==
    cone_proof(combined, driveB(p)+offset, Some(rep_map))` (`ConeProof:
    PartialEq`, already used at compact.rs:1396).
  - **Soundness (coinduction), worked out and recorded.** Define
    `R((sA, sB))` ≡ "every union class in `rep_map` holds equal values across
    both modules' members." At `t = 0`, refinement only *sub-divides* the
    reset-value buckets, so every class lies within one `(width, reset_kind,
    reset_val, domain)` bucket ⇒ all members share `reset_val` ⇒ `R` holds. If
    `R(t)` holds: for `f, g` in one class their quotient-substituted D-cones are
    equal (partition stability) and their `FlopQ`/`PrimaryInput` operands are
    class-equal/shared ⇒ `Q(t+1)` equal ⇒ `R(t+1)`. And under `R(t)`,
    `driveA(p)` and `driveB(p)` — equal under `rep_map` by (ii) — evaluate
    equally ⇒ outputs agree at every `t`. Hence for **every** input sequence A
    and B emit identical output sequences ⇒ observably equivalent ⇒ merging the
    two definitions is sound. A class containing only A-flops or only B-flops is
    fine: equivalence is decided at the *observable* boundary (outputs), not by a
    1:1 state map.
- **Grouping that fits `dedup_semantic_modules_once`.** The combinational pass
  groups by a derived-`Ord` `SemanticModuleProof` (its truth table) and merges
  each group to the lex-smallest survivor. Bisimilarity is an equivalence
  relation but pairwise (no cheap total canonical signature for the sequential
  case in the first cut), so `.3b.2`'s `dedup_sequential_modules_once`:
  1. selects eligible candidates (`has_local_flops()` **true**, but
     `has_local_memories()`/`has_local_fsms()`/`param_env`/`aggregate_layout`
     **false** and `instances.is_empty()` — flops-only leaf modules; skip the
     top by name; skip any module with a resetless flop);
  2. **pre-filters** into buckets keyed by `(sorted input (PortId,width), sorted
     output (PortId,width), flop multiset {(width,reset_kind,reset_val,domain)},
     output count)` — cheap, no cone proof — so the `O(modules²)` comparison only
     runs on plausibly-equivalent groups;
  3. within a bucket, runs pairwise `modules_bisimilar(A, B)` (the combined-module
     check above) and **union-finds** the results into equivalence classes;
  4. reuses `dedup_semantic_modules_once`'s exact tail: lex-smallest survivor,
     `rewrite_instance_module_names`, iterate-to-fixpoint, then
     `prune_modules_made_unreachable`. (First cut excludes instance-bearing
     candidates, so `semantic_group_has_ancestor_relation` is vacuous among
     candidates; keep an equivalent guard for parity/safety.)
  New pass `dedup_sequential_modules(design)` lives in `src/ir/dedup.rs` beside
  `dedup_semantic_modules`, invoked from the same finalization site in
  `src/gen/mod.rs` (after structural + combinational dedup), gated on the new
  knob + `identity_mode = node-id` + effective `e-graph`.
- **Names finalized.** Knob `hierarchy_sequential_module_dedup` on
  `Config`/`Module`/`Design` (serde default `false`, no CLI flag — mirrors
  `hierarchy_semantic_module_dedup`); design-level
  `DesignMetrics::sequential_module_proof_signatures` (a per-eligible-module
  signature, parallel to `semantic_module_signatures`) +
  `num_sequentially_duplicate_module_pairs` (parallel to
  `num_semantically_duplicate_module_pairs`), reducible to zero by the pass.
  Union flop cap `N_bisim_module_flops` (mirrors `N_BISIM_FLOPS = 64`) bounds the
  combined-module refinement; over-cap pairs are skipped (no merge).
- **Gate (rules-first, lowest cost — the `.2b` precedent).** Two stateful
  flops-only leaf modules that are sequentially equivalent up to a non-identity
  state correspondence (e.g. permuted / mutually cross-wired registers, same
  reset) yet structurally distinct enough that both `dedup_modules` (signatures
  differ) and `dedup_semantic_modules` (skips stateful) leave **2** modules;
  with the knob on, the design collapses to **1** and the merged multi-module SV
  is clean across Verilator + both Yosys modes. Plus knob-off snapshots 6/6
  byte-identical, and the existing structural / combinational / flop / FSM merges
  unchanged. The random generator rarely emits a distinct-but-equivalent stateful
  pair the exact + bisim flop passes have not already collapsed intra-module, so
  a dedicated `tool_matrix` scenario set is **not** the lowest-cost proof (same
  reasoning as `.2b`) — a rules-first hand fixture is.

## 2026-06-15 — Whole-module sequential equivalence — design — `IDENTITY-DEEPENING.3a`

Design/decision leaf for `.3` (whole stateful-leaf-module bounded sequential
equivalence). No source change. Full rationale + soundness proof + rejected
alternatives in decision
[`0008`](docs/decisions/0008-identity-deepening-whole-module-sequential-equivalence.md);
this entry records the contributor-facing engineering grounding that pins `.3b`.

- **The approach: cross-module bisimulation, NOT reachable-product / BMC.** The
  `.2` primitive (`merge_bisimilar_flops`) proves flops equivalent *within one
  module* by greatest-fixpoint partition refinement over a quotient D-signature.
  `.3` lifts that to **two modules**: form the disjoint union `M_A.flops ⊎
  M_B.flops`, bucket by `(width, reset_kind, reset_val, domain)` (same key),
  unify each module's `PrimaryInput{port, width}` endpoints across the two
  modules by `(PortId, width)`, run the refinement to a fixpoint, then prove
  every output-port cone equal under the resulting quotient. Bisimulation from a
  reset base case is sound *for all time* (coinduction); reachable-product /
  bounded model checking proves agreement only to depth `k` and is **unsound as a
  merge proof** — rejected here exactly as decision `0007` rejected it at the
  flop level.
- **Generalizes the combinational module proof; added beside it.** A
  pure-combinational module has zero flops ⇒ the union is empty, refinement is
  trivial, and the verdict reduces to "every output cone equal over the input
  endpoints" — exactly what `dedup_semantic_modules` proves today. So `.3` is a
  strict superset. But to keep `dedup_semantic_modules` (and its whole-module
  input-truth-table enumeration via `evaluate_semantic_module_node`)
  **byte-identical**, `.3b` lands as a *separate* default-off pass that runs only
  on flop-bearing leaf modules — the `.2b` precedent of placing
  `merge_bisimilar_flops` next to `merge_equivalent_flops` rather than rewriting
  it. Unifying the two proof engines is a possible later cleanup, never a first
  step.
- **Eligibility = flops-only leaf modules (first cut).** Today
  `semantic_module_proof_inner` (`src/metrics.rs`) returns `None` on
  `has_local_flops() || has_local_memories() || has_local_fsms() ||
  param_env.is_some() || aggregate_layout.is_some()`. `.3b`'s new pass ACCEPTS
  `has_local_flops()` (the whole point) but keeps every other skip — plus
  `!instances.is_empty()` for the first cut. Memories (no reset base case,
  `memory-identity-boundary`), FSM blocks (larger correspondence problem;
  intra-module duplicates already merge via `merge_equivalent_fsms`), wrappers
  (sequential analogue of the bounded-instance combinational wrapper case),
  params, and aggregate projections all stay excluded as named boundaries —
  nothing retired.
- **Resetless flops excluded (carries the `.2b` fix forward).** A module with any
  `reset_kind = None` flop has no provable equal initial state for that flop ⇒ no
  cross-module bisimulation base case ⇒ the module is conservatively skipped.
  This preserves `reset-defined-self-hold-flop-identity` at the module level too.
- **CENTRAL `.3b` IMPL CHALLENGE: a cross-module cone-proof signature.**
  `cone_proof` (`src/ir/compact.rs`) is **module-local** — it keys `LeafEndpoint`s
  by the module's own `FlopId` / `PortId`. The new proof compares cones *across
  two modules*, so `.3b`'s core work is a normalized cone proof whose endpoints
  live in a **shared vocabulary**: `PrimaryInput` keyed by `(PortId, width)` and
  `FlopQ` keyed by a **global union class id** (spanning both modules) rather than
  a module-local `FlopId`. The `.2b` quotient param (`Option<&HashMap<FlopId,
  FlopId>>` threaded via `canonical_flop_endpoint`) is the template; `.3b`
  generalizes it to a union-class map keyed across two modules (e.g.
  `(ModuleTag, FlopId) -> ClassId`). This is why `.3` is split: the soundness +
  budget are settled here (`.3a`), the cross-module proof representation is the
  dedicated impl (`.3b`).
- **Budget reuse + caps.** Per-cone checks reuse `MERGE_SEMANTIC_LIMITS` (12-bit
  support / 128 nodes / 131072 work) verbatim; the cross-module refinement is
  `O(k² · iterations)`, `iterations <= k`, `k` = union flop count, capped by a
  calibration cap `N_bisim_module_flops` (mirrors `N_bisim_flops = 64`); a
  candidate-pair pre-filter (matching `(PortId, width)` interface + flop multiset
  key + output count) keeps the `O(modules²)` comparison tight before any cone
  proof. Over-budget ⇒ fail-closed (no merge), never a guess.
- **Control + gate (working names, finalized at `.3b`).** Default-off
  `Config::hierarchy_sequential_module_dedup` (node-id / e-graph), parallel to
  `hierarchy_module_dedup` (structural) and `hierarchy_semantic_module_dedup`
  (combinational); design-level metric pair `sequential_module_proof` signature +
  `num_sequentially_duplicate_module_pairs` (parallel to
  `semantic_module_signatures` / `num_semantically_duplicate_module_pairs`);
  rules-first gate = two stateful leaf modules sequentially equivalent up to a
  non-identity state correspondence (permuted / cross-wired registers, same reset)
  that both `dedup_modules` and `dedup_semantic_modules` leave as 2, collapsing to
  1 with the knob on, downstream-clean across Verilator + both Yosys modes; plus
  knob-off snapshots 6/6 byte-identical.

## 2026-06-15 — Bisimulation flop merge — impl — `IDENTITY-DEEPENING.2b`

Implemented the `.2a` design in `src/ir/compact.rs`: the new
`merge_bisimilar_flops` pass, the shared `finalize_flop_merge` refactor, the
quotient-aware proof threading, the default-off `Config::bisimulation_flop_merge`
knob (threaded onto `Module`), and the `Metrics::bisimulation_flops_merged`
counter. Findings and decisions the design entry could not fully pin until the
code existed:

- **SOUNDNESS FIX — resetless flops must be excluded from refinement.** The
  `.2a` bucketing keyed on `(width, reset_kind, reset_val, domain)` but did not
  call out that `reset_kind = None` flops have **no base case**: without a reset
  there is no provable equal initial state, so a bisimulation correspondence
  cannot be established. Two resetless self-hold flops would otherwise quotient
  to the same `FlopQ{rep}` signature and *wrongly merge* — violating the
  recorded `reset-defined-self-hold-flop-identity` boundary (the exact pass keeps
  them apart precisely via concrete `FlopQ` endpoint identity, which quotienting
  erases). Fix: only `reset_kind != None` buckets are refinable; resetless flops
  are pinned as singletons. Regression-locked by
  `merge_bisimilar_flops_keeps_resetless_mutual_swap_distinct`.
- **Quotient threading, not duplication.** Rather than copy the proof engine, a
  single `Option<&HashMap<FlopId, FlopId>>` quotient param is threaded directly
  through `collect_leaf_endpoints` / `structural_node_sig_id` /
  `evaluate_node_under_assignment` / `semantic_proof_eligibility` /
  `semantic_cone_proof_with_limits` / `cone_proof`. `None` (every exact / cleanup
  / FSM / gate caller) returns the concrete id ⇒ **byte-identical**; only the
  bisimulation pass passes `Some(class_rep_map)`. `canonical_flop_endpoint` is the
  one substitution point. `semantic_cone_proof` (no-limits MERGE wrapper) became
  test-only (`#[cfg(test)]`) since `cone_proof` now inlines the limits to thread
  the quotient.
- **Global-bucket refinement, deterministic grouping.** The partition is over all
  flops (buckets in `BTreeMap` key order, classes in ascending `FlopId`,
  representative = min id). Grouping members by quotient signature uses a
  **stable linear scan** (first-equal-signature group), never a `HashMap` over
  signatures — a `HashMap` iteration would leak nondeterminism into emitted RTL.
  `ResetKind` is `Hash`/`Eq` but not `Ord`, so `reset_kind_discriminant` maps it
  to a `u8` for the bucket key.
- **Memo-clear gotcha honored.** Fresh `structural_memo` / `structural_ctx` /
  `endpoint_memo` / `semantic_memo` are allocated **per refinement iteration**
  (they are `NodeId`-keyed and assume fixed endpoint identity; the class map
  changes between iterations, so reuse would be unsound). Shared across all
  classes *within* one iteration (the quotient is fixed there), which keeps
  structural sig ids mutually comparable.
- **Pass ordering + finalize reuse.** `merge_bisimilar_flops` runs AFTER
  `merge_equivalent_flops` (exact classes already collapsed) and BEFORE the FSM
  merge + compaction in `generate_leaf_module`. Both flop passes now build an
  `old_to_canonical_old` map and call the extracted `finalize_flop_merge`
  (renumber → `q_node_remap` → node/drive/instance/flop rewire → domain remap →
  rebuild tables). The exact pass stays byte-identical (verified: snapshots 6/6).
- **Downstream-clean bank (decision: focused test + manual smoke, lowest cost).**
  The mutual swap of two equal-reset registers correctly collapses to a single
  self-holding register (holding the reset value forever). Emitted SV is clean
  across **Verilator `--lint-only -Wall`** (0 warnings), **Yosys without-abc**,
  **Yosys with-abc**, and **Icarus `iverilog -g2012`**. Re-bank via
  `ANVIL_DUMP_BISIM_SV=1 cargo test --lib merge_bisimilar_flops_merges_mutual_swap_registers`
  then lint `/tmp/anvil-bisim-merged.sv`. The random generator rarely produces an
  exact bisimilar pair the exact pass has not already merged, so a dedicated
  `tool_matrix` scenario set was *not* the lowest-cost proof (mirrors the
  signoff-knob-sweep cost reasoning) — the rules-first hand fixture is.
- **Schema MINOR bump.** Adding `Metrics::bisimulation_flops_merged` surfaces a
  new key in the `--introspect` `module_metrics` projection, which the schema's
  own §7 policy classifies as a backward-compatible MINOR bump:
  `SCHEMA_VERSION` 1.0 → 1.1 (introspect + MCP tests + schema doc updated).

## 2026-06-15 — Bisimulation flop merge — grounded design detail — `IDENTITY-DEEPENING.2a`

Design-detail leaf, no source change. Grounded decision `0007` in the real merge
machinery (`src/ir/compact.rs`) so `.2b` is a clean code-only slice. Key
findings from reading `merge_equivalent_flops` / `flop_d_signature` /
`cone_proof` / `semantic_cone_proof`:

- **Why a NEW pass, not a tweak to `merge_equivalent_flops`.** The exact pass
  builds a `FlopSignature { width, clock_domain, d: FlopDSignature, reset_val,
  reset_kind }` and groups flops by it. The `d` field keys `FlopQ` endpoints
  **concretely** (`cone_proof` → structural/semantic signature over the actual
  `LeafEndpoint::FlopQ { flop }` set), which is exactly why `D = Q_g` and `D =
  Q_f` get different signatures and never merge. Bisimulation needs a *quotient*
  D-signature (FlopQ → class representative) under iterative refinement, which
  is a different control structure (fixpoint loop), so `.2b` adds
  `merge_bisimilar_flops` beside the exact pass and leaves the latter
  byte-identical.
- **Shared-finalize refactor.** Everything in `merge_equivalent_flops` after
  `old_to_canonical_old` is built (the renumber → `q_node_remap` → node/drive/
  instance rewire → `remap_explicit_flop_domains_after_merge` →
  `rebuild_instance_tables` tail, lines ~1040–1103) is generic given a
  canonical-map + removed-count. `.2b` extracts it into
  `finalize_flop_merge(m, old_to_canonical_old, removed)` and calls it from both
  passes. Verified the tail is partition-agnostic, so this keeps the exact pass
  output identical.
- **Quotient D-signature.** Within a `(width, reset_kind, reset_val,
  clock_domain)` bucket, refine a partition: each iteration recomputes every
  flop's D-cone signature via the existing `cone_proof`, but with
  `LeafEndpoint::FlopQ { flop }` canonicalized to `class_rep(flop)` (a
  `flop -> class` map threaded into `collect_leaf_endpoints` / the structural
  signature / the semantic endpoint offsets). Flops whose quotient D-signatures
  differ split out; repeat until no class splits (coarsest stable partition).
  `D == Q` (self-hold) maps to "own class id" and falls out as the trivial
  fixed point; same-endpoint cones fall out when the identity correspondence is
  stable — both existing classes are special cases, not retired.
- **GOTCHA: memos must be rebuilt per refinement iteration.** `structural_memo`
  / `semantic_memo` / `endpoint_memo` are `NodeId`-keyed and assume a *fixed*
  endpoint identity. The class map changes between refinement iterations, so the
  same `NodeId` can have a different quotient signature across iterations —
  reusing a stale memo would be unsound. `.2b` allocates fresh memos each
  refinement pass (or keys them by a partition-generation counter).
- **Budget.** Per D-cone check reuses `MERGE_SEMANTIC_LIMITS` (12-bit support /
  128 nodes / 131072 work); over-budget cones take the structural fallback
  (which must also be quotient-aware). A bucket-size cap `N_bisim_flops`
  (default `64`) bounds the `O(k² · iterations)` refinement; larger buckets are
  left to the exact pass only and `log`/metric-noted, never silently dropped.
- **Ordering.** Run `merge_bisimilar_flops` AFTER `merge_equivalent_flops`
  (exact classes already collapsed) and BEFORE `merge_equivalent_fsms` +
  `compact_node_ids` in `generate_leaf_module`.
- **Gate (rules-first).** A hand-built `compact.rs` test: flops `f`, `g` with
  `D_f = Q_g`, `D_g = Q_f` (mutual swap), identical width/reset/domain, each
  observed by an output. Assert `merge_equivalent_flops` removes `0` (the exact
  pass provably cannot), then with the knob on `merge_bisimilar_flops` removes
  `1`. Plus the knob-off `tests/snapshots.rs` 6/6 byte-identical regression.
  `.2b` decides dedicated `tool_matrix` scenario vs focused test + manual
  Verilator/Yosys smoke for the downstream-clean bank.
- **Names pinned:** `Config::bisimulation_flop_merge: bool` (default `false`,
  config.rs ~562/821 pattern), threaded onto `Module` beside `identity_mode`;
  `Module::bisimulation_flops_merged: u32` → `Metrics::bisimulation_flops_merged`
  (metrics.rs ~236/856 `flops_merged` plumbing).

## 2026-06-15 — IDENTITY-DEEPENING first extension chosen — `IDENTITY-DEEPENING.1` (decision 0007)

Design/decision leaf, no source change. Picked the first sound identity
extension for Lane 1 and split the tree. Full rationale, soundness argument,
budget, downstream gate, and rejected alternatives live in
`docs/decisions/0007-identity-deepening-first-extension.md`; the contributor-side
"why" worth keeping here:

- **Why sequential, not module-level, first.** The two candidates the tree named
  were (a) bounded semantic *module* equivalence beyond structural signatures and
  (b) a broader bounded *sequential* class. Reading `src/ir/dedup.rs` showed (a)
  is **already partly built**: `dedup_semantic_modules` proves bounded
  whole-module truth-table equivalence for pure-combinational leaves and bounded
  combinational wrappers (`bounded-semantic-module-identity`). So the genuinely
  open, high-value, soundly-bounded frontier is sequential — and the recorded
  no-merge boundary (`reset-defined-self-hold-flop-identity`) names exactly the
  class to attack: *mutually-recursive registers and non-exact feedback*.
- **Why bisimulation (greatest fixpoint), not BMC.** A merge must be a *proof for
  all time*, not "agrees up to depth k". Bisimulation from a reset base case is
  the textbook sound proof of sequential equivalence, and partition refinement
  (Kanellakis–Smolka / Hopcroft) gives it for free with a clean termination bound
  (≤ k iterations, k = bucket size). BMC is unsound as a *merge* proof and was
  rejected outright.
- **Why it reuses, not replaces, the combinational engine.** The bisimulation
  step compares two D-cones *up to the current state correspondence*: rewrite
  each `FlopQ` endpoint to its current class representative, then run the existing
  bounded endpoint-preserving combinational proof over the quotient endpoint set.
  Same 12-bit / 128-node / 131072-work budget; budget-exceeded ⇒ conservative
  class split ⇒ no merge. The exact self-hold (`D==Q`) and same-endpoint classes
  fall out as the identity-correspondence special cases — they are generalized,
  not retired.
- **Why a new default-off knob, not on-by-default at e-graph.** The existing flop
  merge is on at `node-id`/`cse` and changes default output vs `relaxed`; adding
  the *broader* class to it would change default-config snapshots. To keep
  `tests/snapshots.rs` byte-identical by default and match the established opt-in
  precedent (`hierarchy_module_dedup` / `hierarchy_semantic_module_dedup`), the
  additional merges sit behind a new default-off `Config` knob (working name
  `bisimulation_flop_merge`) that also requires `node-id`/`e-graph`. `.2`
  finalizes the knob/metric names and the bucket-size cap empirically.

## 2026-06-15 — First signoff knob-sweep batch impl — `SIGNOFF-AUTOMATION-EXPANSION.2b`

Implemented the `.2a` design. Landed in `src/metrics.rs` (the new
`num_operator_gates_with_duplicate_operands` post-hoc metric) and
`src/bin/tool_matrix.rs` (the `ScenarioSet::SignoffKnobSweep` set, four
focus configs + `build_signoff_knob_sweep_scenarios`, the
`--signoff-knob-sweep-gate` flag, four `saw_*` facts, the early-return
gap arm, and the new constant `SIGNOFF_KNOB_SWEEP_MIN_UNITS_PER_SCENARIO
= 4`). Banked downstream-clean at `/tmp/anvil-signoff-knob-sweep-r1` (12
scenarios, 48 modules, four facts `true`, `coverage_gaps = []`, `48/0`
Verilator + both Yosys). Empirical findings the design entry could not
have known without probing the real generator:

- **`flop_prob` is load-bearing for the degenerate mux.** The
  `num_muxes_degenerate` fact (`mux_arm_duplication_rate`) needs the
  chained-ternary comb-mux assembly to collapse an arm and its running
  tail to the same `NodeId` in a tiny pool. Forcing `flop_prob = 0.0`
  (a pure-comb DUT) made `num_muxes_degenerate` collapse to ~0 across
  seeds; leaving `flop_prob` at its default `0.15` produced 10–37
  degenerate muxes per seed reliably. So the mux-dup scenario does **not**
  override `flop_prob` — the richer (partly sequential) cone is what
  exercises the duplication path. (Operand-dup is independent of
  `flop_prob`: `pick_signals_with_dup_rate` builds the operand list
  directly, so a tiny pool + arith-only weights lights it regardless.)
- **Duplication facts only light in single-module DUTs, not wrapper
  leaves.** Probing a depth-1 wrapper whose leaves carried the mux-dup
  knobs produced **zero** degenerate muxes (the wrapper-lane leaf
  builder does not hit the chained-ternary comb-mux path the same way),
  while a single-module DUT lit it. So the two duplication scenarios are
  single-module DUTs and the two aggregate/memory scenarios are depth-1
  wrapper designs — the set is mixed (artifact_kind `"module"`, the
  Default-set convention), and the design path's per-leaf
  `accumulate_module_coverage` is irrelevant to them.
- **Array-packed needs uniform widths + a calibrated seed.** With
  `aggregate_array_prob = 1.0` and `min_width == max_width = 8`,
  `num_array_packed_aggregate_modules > 0` on most seeds (some still
  pick `StructPacked`); the gate's ≥4 units/scenario × 3 strategies
  covers the tail.
- **memory×fsm interplay confirmed at `memory_prob = 0.5`, `fsm_prob =
  1.0`, 6 leaves:** 7/8 probed seeds realized both a memory module and
  an FSM module (only an all-memory seed missed) — confirming the
  `.2a` mutual-exclusivity analysis (memory rolled first, returns early).
- **Gate isolation via early return.** `compute_coverage_gaps` has
  unconditional broad-motif checks (priority encoder, comb/flop muxes,
  case/casez, for-fold) that every other set satisfies. Rather than
  guard ~10 checks, the `SignoffKnobSweep` arm checks its four facts and
  `return`s before them; the two post-return `match scenario_set`
  blocks (axis + `required_categories`/`required_knobs`) carry
  unreachable `SignoffKnobSweep => {}` / `&[]` arms purely for
  exhaustiveness.
- **`num_muxes_degenerate` does materialize through generation**, despite
  the chained-ternary muxing each data arm against the running tail
  rather than against another arm: in a 1–2-signal pool the tail
  collapses (via CSE) to the same `NodeId` as the next arm, so
  `make_mux(a, a)` is reached and kept at rate 1.0. The metric is not a
  hand-construction-only artifact.

## 2026-06-15 — First signoff knob-sweep batch design — `SIGNOFF-AUTOMATION-EXPANSION.2a`

Design leaf (docs-only, no source change) concretizing the open question that
decision [`0006`](docs/decisions/0006-signoff-automation-first-increment.md)
deliberately left to `.2`: the exact knob batch, the scenario shapes, the
`saw_*` fact names, the focused gate, and the gap wiring for the first
richer-knob-sweep increment (implemented in `.2b`). Split from the original `.2`
leaf per the `.3a`/`.3b` precedent because the batch crosses real policy choices
(combined-stress vs per-knob scenarios; which facts/metrics; a new metric or a
deferral; gate membership) and touches ~6 regions of the 9.6k-line
`src/bin/tool_matrix.rs` plus `src/metrics.rs` — independently reviewable as
design then implementation.

### Which knobs are *genuinely* unswept (inventory refinement)

`.1`/`0006` listed candidates loosely. The matrix study refines this: the
single-knob axes for `width_parameterization_prob`, `aggregate_prob`,
`memory_prob`, and `fsm_prob` **already exist** in the default scenario set
(`phase5_width_parameterized`, `phase5b_packed_aggregate`,
`phase6_inferrable_memory`, `phase6_fsm`) with gated `saw_*` facts
(`saw_width_parameterized_design` / `saw_packed_aggregate_design` /
`saw_inferrable_memory_design` / `saw_fsm_design`). The genuinely **unswept**
knobs — fired only by chance inside motif-heavy profiles, never as explicit axes
— are exactly four:

1. `mux_arm_duplication_rate` — no scenario, no fact.
2. `operand_duplication_rate` (Add/Mul) — no scenario, no fact, **and no metric**.
3. `aggregate_array_prob` — no scenario, no fact (this is the deferred
   `AGGREGATE-ARRAY-PACKING.4b` "optional matrix CI instrumentation"; `.2b`
   closes that deferral).
4. memory×fsm **interplay** — single-knob memory and FSM scenarios exist, but
   nothing proves a memory leaf and an FSM leaf in the **same** design.

`.2b` promotes exactly these four into explicit first-class axes + facts.

### Scenario shapes — one focused scenario per knob

Per-knob focused scenarios, not one combined-stress scenario: a day-one
downstream failure must point at exactly one knob, and each fact must be
provable from a single realized metric (no entangled attribution).

- **`int_operand_duplication`** — single-module DUT, interleaved, comb-only,
  arithmetic-favoring gate weights (Add/Mul present), `operand_duplication_rate
  = 1.0`. (`operand_duplication_rate` covers Add/Mul only, per `Config` doc.)
- **`int_mux_arm_duplication`** — single-module DUT, interleaved, comb-only,
  comb-mux-favoring with `min_mux_arms = max_mux_arms = 2` (force 2-to-1 muxes),
  `mux_arm_duplication_rate = 1.0`.
- **`phase5b_array_packed_aggregate`** — depth-1 wrapper design shaped like the
  `phase5b_packed_aggregate` anchor but with `aggregate_prob = 1.0` **and**
  `aggregate_array_prob = 1.0`, and **uniform** data-port widths
  (`min_width == max_width`). Uniformity is load-bearing: `ArrayPacked` is a
  faithful projection only over a uniform-width group, so a non-uniform group
  falls back to `StructPacked` (`src/ir/aggregate.rs`).
- **`memory_fsm_interplay`** — depth-1 wrapper design with `memory_prob` strictly
  in `(0,1)`, `fsm_prob = 1.0`, enough library leaves, and a calibrated seed.
  **Gotcha (load-bearing):** per-leaf memory-vs-FSM selection in
  `src/gen/module.rs:368-386` is *mutually exclusive* — `memory_prob` is rolled
  first and on a hit immediately `return`s a memory leaf, so `memory_prob = 1.0`
  produces **no** FSM leaf, ever. Interplay therefore needs a probabilistic
  memory split so some leaves fall through to the FSM roll (which then fires at
  `fsm_prob = 1.0`); the fixed seed + leaf count are calibrated so the realized
  design provably carries ≥1 memory leaf **and** ≥1 FSM leaf. The gate enforces
  the fact, so a miscalibrated seed fails loudly and is recalibrated (the
  standard matrix calibration loop, cf. ROADMAP's "(2,2 calibrated)" notes).

### Coverage facts — each provable from one realized metric

- `saw_operand_duplication` ← `metrics.num_operator_gates_with_duplicate_operands
  > 0` (module-level). Requires a **new** metric in `src/metrics.rs`: count of
  `Add`/`Mul` `Node::Gate`s whose operand list repeats a `NodeId`. RTL
  byte-identical — metrics are post-hoc over the finalized IR and never emitted.
- `saw_mux_arm_duplication` ← `metrics.num_muxes_degenerate > 0` (module-level,
  existing metric — the `(s)?(x):(x)` same-arm form, documented as "should be 0
  at `mux_arm_duplication_rate = 0.0`").
- `saw_array_packed_aggregate_design` ← `scenario.config.aggregate_array_prob >
  0.0 && design.metrics.num_array_packed_aggregate_modules > 0` (design-level,
  existing metric).
- `saw_memory_fsm_interplay_design` ← `memory_prob > 0.0 && fsm_prob > 0.0 &&
  num_memory_modules > 0 && num_fsm_modules > 0` (design-level, existing metrics).

### The focused gate

New opt-in `tool_matrix --signoff-knob-sweep-gate` + `ScenarioSet::SignoffKnobSweep`
+ `build_signoff_knob_sweep_scenarios` (the four focused scenarios). Modeled on
`--phase2-share-gate` / `--phase3-structured-gate`: it auto-enables
`--fail-on-coverage-gap`, is mutually exclusive with the other gate flags, and
`compute_coverage_gaps` for this set requires the four new facts (and the
shared comb-only/sequential basics those scenarios light). Kept as a
**dedicated** scenario set rather than folded into the default set, to keep the
blast radius minimal, leave the matrix's existing leaf/child/range shape-coverage
sets unperturbed, and keep the banked report self-contained — mirroring the
phase2/3/4 dedicated gates. No existing gate, scenario, fact, or metric is
retired (`feedback_never_retire_strategies`).

### Invariants preserved

- **Rules-first / no generate-then-filter:** all four knobs are construction-time
  decisions inside the generator; the matrix only *observes* the realized IR.
- **Default-off / byte-identical:** each knob changes RTL only when set `> 0`;
  the default sweep, the `insta` snapshot guard, and every existing gate are
  untouched. The new operand-duplication metric is post-hoc ⇒ RTL byte-identical.
- **Single downstream source of truth** stays `tool_matrix` + `downstream`:
  `.2b` adds scenarios + facts + one gate, never a second runtime path.

---

## 2026-06-15 — Hand-rolled HTTP transport impl — `AGENT-MCP-EXPANSION.4b`

Landed the transport exactly as `.4a` pinned it: a new `src/mcp/http.rs`
(`read_http_request` / `write_http_response` / `handle_http_connection` /
`serve_http` / `resolve_http_addr`) re-exported from `src/mcp/mod.rs`, plus the
opt-in `--http <addr>` flag on `src/bin/anvil_mcp.rs`. Engineering notes beyond
the `.4a` design entry below:

- **`read_exact` resolves through `BufRead`'s supertrait — drop the `Read`
  import.** The body read is `reader.read_exact(&mut body)` where `reader: &mut
  impl BufRead`. `read_exact` is a `Read` method, but a `BufRead` bound already
  carries the `Read` impl, so the call resolves without `std::io::Read` in
  scope; importing it tripped an `unused_imports` warning (which clippy
  `-D warnings` would reject). `Read` is imported **only** in the test module,
  where `TcpStream::read_to_string` needs it.
- **`try_clone()` to read and write the same socket.** `handle_http_connection`
  wraps the stream in a `BufReader` for line-oriented header parsing, but the
  response needs the raw `Write` half. `stream.try_clone()` yields a second
  handle to the same connection (both close when dropped), the standard pattern
  for a read-buffered + separately-written socket without threading.
- **204 No Content carries no `Content-Length` (RFC 7230).** A `204` must not
  send a body or a `Content-Length`; emitting one trips strict clients. So the
  `None` arm special-cases `204` (status line + `Connection: close` only) and
  uses `Content-Length: 0` for the `4xx` framing errors. JSON-RPC-level errors
  never reach this path — they ride inside a `200` body.
- **One shared `McpServer`, no lock.** `serve_http` constructs a single
  `McpServer` and threads `&mut server` through the sequential accept loop, so
  the content-addressed cache + audit log persist across calls exactly as over
  stdio — and because the loop is single-threaded, no `Mutex` is needed despite
  the `&mut self` dispatcher.
- **Bare-port loopback default via an all-digits check.** `resolve_http_addr`
  treats an all-ASCII-digits arg as a port and binds `127.0.0.1:port`; anything
  else is parsed as a full `SocketAddr` (an out-of-range bare port like `99999`
  fails the `u16` parse and errors cleanly). `!addr.ip().is_loopback()` drives
  the bin's stderr exposure warning.
- **Validation.** `cargo fmt/check/clippy -D warnings` clean; `cargo test --lib
  mcp::` 50 pass (35 prior + 15 new framing/resolve/round-trip tests, the last
  two binding a real `TcpListener` on `127.0.0.1:0`); `cargo test --test
  snapshots` 6/6 byte-identical (the stdio default path is untouched). A
  real-binary `curl` smoke confirmed `initialize`→`200`, `tools/list`→`200`,
  `GET`→`405`, notification→`204`. No new Cargo dependency.

## 2026-06-15 — Hand-rolled HTTP transport design — `AGENT-MCP-EXPANSION.4a`

`.4a` is the design leaf that pins the framing for the optional HTTP
transport before `.4b` writes the network code. The owner already fixed the
high level in `bc70aee` (hand-rolled HTTP/1.1 over `std::net`, no new crate
dependency, `--http <addr>` on the existing bin, loopback default, same
dispatcher); `.4a` resolves the remaining framing/connection/concurrency
choices so the impl is mechanical. `.4` was split into `.4a` (design) +
`.4b` (impl), mirroring `.3a`/`.3b`.

**Decision — reuse `McpServer::handle_line`, the transport-agnostic seam.**
`handle_line(&mut self, &str) -> Option<String>` already does exactly what a
transport needs: parse one JSON-RPC message, dispatch, serialize the
response (or `None` for a notification). The HTTP transport calls the *same*
method the stdio loop calls — there is no second protocol path, so tools,
resources, prompts, error codes, and the content-addressed cache behave
identically over both transports and stay covered by the existing in-process
tests.

**Decision — single-threaded, one shared `McpServer`, sequential accept
loop.** `McpServer` is `&mut self` (it mutates the artifact `cache` and the
`audit` log) and uses no sync primitives. Rather than wrap it in
`Arc<Mutex<…>>` and spawn a thread per connection, the accept loop is
single-threaded and reuses **one** `McpServer` across connections, serving
requests sequentially. This (a) keeps the cache + audit log continuous across
calls within a process — an agent `generate`s then `resources/read`s the
artifact, exactly as over stdio — and (b) needs no locking. A `Mutex` around
`handle` would serialize requests anyway, so threading would add lock churn
and contention for no throughput gain in the local single-agent bug-hunting
loop this serves. Robustness without threads: a per-connection **read
timeout** keeps a stalled client from wedging the loop, and per-connection
I/O errors are logged-and-swallowed so one bad client never terminates
`serve_http`.

**Decision — one request per connection, `Connection: close`.** No
keep-alive, no pipelining. Each accepted connection carries exactly one
JSON-RPC POST and is closed after the response. This removes the trickiest
hand-rolled-HTTP failure modes (persistent-connection framing, request
boundary tracking) at no real cost for an RPC workload, and it composes
cleanly with the sequential accept loop.

**Decision — minimal, liberal request parsing; strict, small status set.**
Read the request line + headers (CRLF-delimited; header names matched
case-insensitively) up to the blank line, then read exactly `Content-Length`
body bytes as one JSON-RPC message (`handle_line` trims, and `serde_json`
accepts arbitrary whitespace, so a pretty-printed body works too). The body
size is capped at `MAX_BODY_BYTES = 16 MiB` as defense-in-depth against a
giant-allocation request even though the default bind is loopback. The
JSON-RPC ⇄ HTTP mapping is deliberately tiny:

| Condition | HTTP status |
| --- | --- |
| `handle_line` → `Some(json)` | `200 OK` + `application/json` body |
| `handle_line` → `None` (notification / blank) | `204 No Content`, no body |
| method not `POST` | `405 Method Not Allowed` |
| `POST` without `Content-Length` | `411 Length Required` |
| malformed request line / unparseable `Content-Length` | `400 Bad Request` |
| `Content-Length` > `MAX_BODY_BYTES` | `413 Payload Too Large` |

JSON-RPC-level errors (unknown method, bad params) stay **inside** the
`200 OK` body as JSON-RPC error objects — same as over stdio. HTTP status is
only about transport framing, never about RPC semantics.

**Decision — loopback-default address semantics on a hand-parsed flag.** The
bin hand-parses a single optional `--http <addr>` from `std::env::args()`
(no clap surface — the bin is a thin transport, and one optional flag does
not justify pulling clap's derive into it; `--help`/unknown-arg still get a
clear stderr message + nonzero exit). `<addr>` is interpreted so the *default
is loopback*: a bare port (all digits) binds `127.0.0.1:<port>`; a full
`IP:port` is parsed as a `SocketAddr` and honored as given, but binding a
**non-loopback** IP prints a prominent stderr warning that the controlled
`validate`/`minimize` tools are now reachable over the network (decision
`0005`'s security note). Without `--http`, the bin runs the unchanged stdio
loop — byte-identical, default-off.

**Decision — framing helpers in the lib, loop callable from the bin.** Pure
helpers `read_http_request(&mut impl BufRead) -> io::Result<HttpRequest|…>`
and `write_http_response(&mut impl Write, status, body)` plus
`handle_http_connection(stream, &mut McpServer)` and
`serve_http(SocketAddr) -> io::Result<()>` live in a new `src/mcp/http.rs`
re-exported from `src/mcp/mod.rs`, so the framing is unit-testable in-process
over `Cursor`/byte buffers (the same lib-not-bin discipline as the rest of
the MCP surface). `.4b`'s real-socket round-trip test binds a `TcpListener`
on `127.0.0.1:0`, serves one connection in a thread, and asserts a real POST
of `initialize` yields `200 OK` + the JSON-RPC `result`.

**Rejected alternatives.** (1) *clap in the bin* — heavier than a one-flag
hand-parse for a transport shell. (2) *HTTP keep-alive / pipelining* —
needless framing complexity; one-shot connections are robust. (3)
*thread-per-connection + `Arc<Mutex<McpServer>>`* — the workload is
sequential and a whole-`handle` Mutex would serialize anyway, so threading is
pure overhead here. (4) *a separate `anvil-mcp-http` bin* — a flag on the
existing bin keeps the default build + stdio path untouched (decision
`0005`). (5) *an async/`tokio` HTTP stack or an MCP SDK* — already rejected
in `0004` for the stdio server; the same dependency-light doctrine applies.

**No new Cargo dependency.** `std::net` (`TcpListener`/`TcpStream`/
`SocketAddr`), `std::io` (`BufRead`/`Write`/`BufReader`), and `std::time`
(`Duration` for the read timeout) cover everything. `Cargo.toml` is
untouched, so the conservative-dependency posture holds.

## 2026-06-15 — non-DUT lanes over MCP — `AGENT-MCP-EXPANSION.3b`

Implements the non-DUT (`microdesign`/`frontend`) generate/introspect path.
Engineering notes beyond the `.3a` design entry below:

- **Schema-conformance correction.** The `.3a` plan (ResourceRef-only, no
  inline) was **wrong against the contract**: `AGENT_INTROSPECTION_SCHEMA.md`
  §5/§6.5 already define inlined `microdesign_manifest`/`frontend_manifest`
  payload sections at v1.0; §6.6's "resource, not inlined" applies only to
  the bulk `.sv`. `.3b` conforms — it inlines the manifest in the payload
  **and** sets the `artifact.manifest` ResourceRef (both from one
  `emit_manifest` output). No schema-version bump. Lesson: a design leaf
  must read the schema *spec*, not only the code.
- **Why a `Value`, not the typed doc (byte-stability trap).** The non-DUT
  document is a `serde_json::Value`, not `IntrospectionDocument`, because the
  typed `RequestEcho.knobs` is a `Config` (non-DUT lanes have
  `n_params`/`n_children`) and, critically, a `serde_json::Value::Object`
  is a sorted `Map` — round-tripping the DUT `Config` through `to_value`
  would **re-sort the keys** and change the `--introspect` DUT bytes. So the
  typed DUT path is left 100% untouched; only the new non-DUT path is a
  Value.
- **`content_run_id` generalization.** Split into
  `content_run_id_for_knobs(lane, seed, knobs_json)` with `content_run_id`
  (the DUT `Config` specialization) delegating to it — the DUT canonical
  string is unchanged, so DUT run_ids are byte-identical. Non-DUT lanes pass
  a deterministic scoped-knob JSON so their content address is collision-free
  across differing `n_params`/`n_children`.
- **One manifest source.** `build_and_cache_lane` parses the lane's
  `emit_manifest` string once: the parsed `Value` is inlined in the payload
  and the raw string is served as the `…/manifest` resource. A test asserts
  the inlined facts equal the served resource, so the two cannot drift.

## 2026-06-15 — non-DUT introspection projection — `AGENT-MCP-EXPANSION.3a`

Design leaf deciding how the `microdesign`/`frontend` lanes introspect
over MCP. Key engineering insight:

- **The schema already reserved the slot.** `ArtifactDescriptor.manifest:
  Option<ResourceRef>` (`src/introspect/mod.rs:82`) has been `None` since
  `.3` (DUT lanes have no manifest). The non-DUT lanes already emit their
  expected-facts manifest (`emit_manifest`, a serde projection of each
  lane's `Manifest`), carried on `umbrella::LaneArtifact.manifest:
  Option<String>`. So non-DUT introspection just *populates that slot* with
  a `ResourceRef` to `anvil://artifact/<run_id>/manifest` and serves the
  manifest as a resource — no new field, no schema bump, no computed truth.
- **Why a resource, not inlined.** Decision `0004`/schema §6.6 mandates
  "structured queries, not bulk dumps; full manifests are resources the
  agent fetches deliberately." Inlining the manifest into the payload (a
  `lane_manifest` field) was rejected: it bumps the schema and contradicts
  §6.6.
- **`.3b` gotcha — content address.** `content_run_id` keys on
  `(schema_version, anvil_version, lane, seed, knobs_json)` where
  `knobs_json` is `serde_json::to_string(Config)`. Non-DUT lanes have no
  `Config` — their knobs are `n_params`/`n_children`. `.3b` must feed a
  deterministic canonical encoding of those scoped knobs into the content
  address (the `lane` field already separates lanes), or non-DUT run_ids
  would collide across differing scoped knobs.

## 2026-06-15 — coverage_gaps pure-projection tool — `AGENT-MCP-EXPANSION.2`

`.2` implements the `.1`/`0005` decision: the `coverage_gaps` MCP tool
relays the recorded gap list from a `tool_matrix_report.json` rather than
recomputing. Implementation notes beyond the design entry below:

- **Early dispatch.** `coverage_gaps` is matched in `tools_call` *before*
  the shared `config_from_args` parse, because it takes neither `seed` nor
  `config` — the other five tools do. Keeping it ahead of that parse avoids
  threading an irrelevant `(seed, cfg)` through a pure file/inline read.
- **`dark_coverage_facts` is a filter, not a computation.** The projection
  also surfaces the recorded `saw_*` booleans that are still `false` — the
  directly actionable "what's dark?" set the `close_coverage_gap` prompt
  references. It is a filter over recorded values (sorted for deterministic
  output regardless of the `serde_json` map backing), so it adds no new
  truth.
- **Why a `Value` key projection, not a typed struct.** Mirroring
  `MatrixReport`/`CoverageSummary` into `src/mcp/` would couple the adapter
  to a bin-private struct that grows on nearly every hierarchy slice (~150
  fields today). Reading known keys off `serde_json::Value` keeps the
  adapter robust to that churn and missing/renamed fields degrade to
  `null`, not a hard parse failure (except the load-bearing
  `coverage_gaps` array, whose absence is a clean "not a tool_matrix
  report" error).

## 2026-06-15 — Agent/MCP expansion design — `AGENT-MCP-EXPANSION.1` (decision `0005`)

`.1` is the design/decision leaf scoping the read-mostly agent/MCP breadth
expansion. Full rationale is in
`docs/decisions/0005-agent-mcp-expansion-surface.md`; the load-bearing
engineering points:

**Decision — coverage gaps are PROJECTED from a recorded report, not
recomputed.** The coverage-gap computation (`CoverageSummary` +
`compute_coverage_gaps`) is **private to the `tool_matrix` binary**
(`src/bin/tool_matrix.rs:286,6552`) — neither `src/mcp/` nor `src/lib.rs`
can call it. But the serialized `MatrixReport` already carries `coverage`
and the already-computed `coverage_gaps: Vec<String>` (`:488-489`). So the
`.2` MCP tool relays that recorded list rather than re-deriving it. This
keeps the single gap computation in `tool_matrix` (no second source of
truth), keeps the tool read-only (a file/inline read, no generation, no
tool spawn, no recompute), and keeps SCHEMA-DERIVED intact.

**Gotcha — do NOT mirror `CoverageSummary` into `src/mcp/`.** That struct
already has ~150 fields and grows on nearly every hierarchy slice. The
`.2` tool must project the recorded JSON via `serde_json::Value` key reads,
so the MCP side is decoupled from the bin-private struct's churn.

**Rejected alternative — a controlled `coverage_gaps` tool that runs a
matrix subset on demand.** It would compute coverage state on demand (a
second runtime path that can drift from `compute_coverage_gaps`), turn a
read-only query into a heavy tool-spawning controlled action, and pull
matrix logic into `src/mcp/`. It loses on every invariant the
pure-projection path keeps.

**Decision — `.3` split into `.3a`/`.3b`.** Routing the non-DUT lanes
(`microdesign`, `frontend`) through the umbrella `ArtifactLane` dispatch is
straightforward, but the non-DUT introspection document must stay a serde
projection of each lane's existing manifest, and whether that projection
already exists is unresolved — so design (`.3a`) precedes impl (`.3b`),
mirroring the original lane's `.5.1/.5.2/.5.3` split.

**Decision — `.4` HTTP transport reuses the pure dispatcher.**
`McpServer::handle` is already transport-agnostic, so HTTP drives the same
dispatcher behind an opt-in flag. Because HTTP would expose the controlled
`validate`/`minimize` tools over a socket, the transport binds
**loopback-only by default**; the per-call `downstream` guardrails are
unchanged.

## 2026-06-15 — Agent-workflow prompts as MCP prompts — `AGENT-INTROSPECTION-MCP.6`

`.6` ships the five agent-workflow prompts (`find_downstream_bug`,
`close_coverage_gap`, `minimize_reproducer`, `triage_tool_failures`,
`explain_artifact`) as first-class **MCP prompts** in `src/mcp/mod.rs`. Notes:

**Decision — prompts are a real MCP primitive, not static doc text.** The `.1`
phasing hint called `.6` "docs/config", but MCP defines `prompts/list` /
`prompts/get` precisely for packaging named, parameterized workflows a client
can fetch and run. Implementing the workflows there (rather than as prose in the
book) is the agent-drivable realization decision `0004` envisioned when it
mapped "Prompts (workflows)" onto ANVIL, and it lets the leaf acceptance ("each
prompt drives its tool chain end-to-end on a sample") be *proven by execution*
instead of asserted. The book/USER_GUIDE prose still lands — but in `.7`, as the
user-facing closeout; `.6` is the mechanism.

**Decision — prompts add no capability and no new truth.** A `PromptSpec`'s
renderer is a pure function `args -> ordered (role, text) messages`; it only
instantiates a chain over the *existing* tools/resources. This keeps the
read-mostly, no-second-source-of-truth doctrine intact: a prompt cannot generate,
validate, or compute anything the tools don't already own. The single `PROMPTS`
registry owns the set so it can't drift from the dispatch (the same
one-owner pattern as `parse_validate_tools`/`parse_yosys_mode_arg` for the
controlled tools).

**Gotcha — MCP prompt arguments are strings.** Per the MCP prompt contract,
`prompts/get` arguments arrive as a `{string: string}` map. The server rejects a
non-string value with a clean `-32602` (see the
`prompts_get_enforces_required_args_and_unknown_name` test) rather than coercing
— so the rendered `"seed": 42` substitution comes from a string `"42"`, and a
client passing a JSON number gets a contract error, not a silent stringify.

**Test shape — portability via `tools: []`.** ANVIL output is
valid-by-construction, so no real tool can manufacture a failing case, and the
external-tool legs of the chains can't be exercised portably with a *failure*.
The end-to-end test therefore drives every chain with `tools: []` (the validate
oracle is vacuously `ok`, minimize reports `reproduced_initial=false`), which
still exercises the full generate/sandbox/oracle/audit path through the server —
proving each prompt names a real, runnable sequence — without requiring
verilator/yosys to be installed. Same pattern the `.5.2`/`.5.3` tool tests use.

---

## 2026-06-15 — Controlled `minimize` delta-debugger — `AGENT-INTROSPECTION-MCP.5.3`

`.5.3` adds `downstream::minimize(seed, cfg, opts)` + the MCP `minimize`
adapter, closing the agent bug-hunting loop (generate → validate → shrink to a
minimal reproducer). Notes worth keeping:

**Decision — `validate` is a pure failure *oracle*; minimize searches inputs
only.** A candidate "reproduces" iff its `.5.2` `validate` run *completes* (the
memory guard did not decline) and the verdict is **not** `ok`. The search never
touches emitted RTL — it re-runs the existing rules-first generator + the vetted
oracle on each candidate `(seed, knobs)`. So it is squarely *not*
generate-then-filter / repair (the doctrine line `0004` draws): the agent drives
a deterministic experiment over the *input* space; ANVIL stays the source of
truth. A decline is treated as **inconclusive → does not reproduce**, so a
half-run under memory pressure can never be mistaken for a passing shrink.

**Decision — hold the seed fixed; shrink only knobs.** Changing the seed yields
a *different* artifact, not a smaller version of the same one — it would change
the reproducer's identity, not minimize it. The seed pins the reproducer; the
knobs are the reducible surface. (The leaf title "delta-debug of `(seed,
knobs)`" refers to the reproducer *tuple* that is shipped, not to mutating the
seed.)

**Decision — two reduction registries, deterministic order.** Integer **size
bounds** (`max_depth`, `max_width`, `max_inputs`, `max_outputs`,
`max_flops_per_module`, `max_mux_arms`, `max_gate_arity`, `max_coefficient`,
`max_shift_amount`, `max_comparand`) are bisected toward each knob's floor; each
floor tracks the companion `min_*` so the candidate range stays valid.
**Optional-motif probabilities** (flop + the const/encoder/case/casez/for-fold/
comb-mux/flop-mux motifs, the Phase 5/5b/6 param/aggregate/memory/fsm/multi-clock
lanes, the duplication rates, the hierarchy routing probs) are driven to `0.0`
("feature absent"). The two registries are swept to a fixpoint.

**Rejected — zeroing sharing/reuse/library/constant knobs.** `share_prob`,
`terminal_reuse_prob`, `library_prob`, `constant_prob` are *excluded* from the
prob registry: their `0.0` is not unambiguously simpler (e.g. `share_prob=0`
*increases* node count). Minimize only makes moves that are monotone
simplifications — smaller bounds, or one fewer optional motif.

**Decision — bisection is a bounded heuristic, not a proven minimum.** Downstream
failures are not monotone in the knobs, so bisecting toward a floor can step over
a reproducer below a non-reproducing midpoint. That is the accepted delta-debug
trade-off: the acceptance bar is *a* smaller reproducer, deterministically and
under a hard budget — not the global minimum. Every candidate is re-checked with
`Config::validate` *before* the generator sees it (an invalid midpoint just
raises the search floor — never a spawn), and the whole search is capped by
`max_oracle_calls` (default 200) + a `MINIMIZE_MAX_PASSES` fixpoint bound.

**Decision — capture `final_validation` from the last failing oracle call (no
extra tool run).** Every accepted reduction came from a `validate` run that
returned a failing `ValidateReport`; later candidates can only `Pass`/`Decline`
(rejected). So the *last* failing report always lands on the minimized config —
the production oracle closure stashes it via an `&mut Option<ValidateReport>`
capture, and `minimize` attaches it as `final_validation`. No confirming re-run
is spent, and the report shows exactly which tool still rejects the minimized
artifact.

**Test strategy — synthetic predicate oracle for the shrink logic.** ANVIL output
is valid by construction, so no real tool can manufacture a failing case to
delta-debug; a real reproduction would itself be the headline finding (a
generator bug). The search core (`search_minimal`) is therefore generic over the
oracle and unit-tested with a pure predicate (bisection finds the exact monotone
boundary; unconstrained bounds collapse to floors; a depended-on knob is
preserved; the budget and a guard-decline each stop the search). The
*real-oracle wiring* is proven by the `tools: []` no-repro path and the
tool-gated e2e, where seed 42 honestly returns `reproduced_initial = false`.

**Refactor — shared `tools`/`yosys_mode` parsing.** `run_validate` and
`run_minimize` both parse the same agent-facing `tools` allow-list and
`yosys_mode`; the logic was lifted into `parse_validate_tools` /
`parse_yosys_mode_arg` (one owner) so the two controlled tools cannot drift —
the same full-factorization move `.5.1` made for the invocations themselves.

## 2026-06-14 — Controlled `validate` tool — `AGENT-INTROSPECTION-MCP.5.2`

`.5.2` adds the first agent tool that runs external tools:
`downstream::validate(seed, cfg, opts)` + the MCP `validate` adapter. The
security model is the load-bearing part; notes worth keeping:

**Decision — generate into a fresh per-run sandbox, never an agent path.** The
artifact is regenerated deterministically (the run is a pure function of
`(seed, knobs)`) and written to
`<sandbox_root>/anvil-validate-<run_id>/<top>.sv`. The MCP adapter fixes
`sandbox_root` to `std::env::temp_dir()` and does **not** expose it as a tool
argument, so the agent cannot direct a write anywhere. The directory is removed
after the run unless `keep_sandbox` (tests set it to inspect the `.sv`).

**Decision — one combined `.sv` file even for designs.** `tool_matrix` writes
one file per module for hierarchy realism, but for an acceptance check
`emit::to_sv_design` (all modules in one string) + `--top-module` / `-top`
is equivalent and simpler, and reuses the `.5.1` `*_design` runners unchanged
(they take a `&[PathBuf]` — a one-element slice via `std::slice::from_ref`).

**Decision — fixed tool allow-list, no arbitrary shell, no binary override.**
`AcceptanceTool` is a closed `verilator`/`yosys`/`iverilog` enum with fixed
`binary()` names. The MCP tool's `tools` argument selects *which* of these run;
anything off the list is a clean tool error, never a spawn (`from_name`
returns `None`). There is deliberately no agent-facing way to pass a binary
path or a raw command — decision `0004`'s "only fixed, vetted tool
invocations."

**Decision — ram-guard is decline-to-start-more, and honest about its reach.**
`validate` checks `MemGuard` (built from explicit `MemLimits` via the new
`MemGuard::from_limits`) *before each spawn*. The honest scope: the in-process
guard samples ANVIL's own RSS + the host used-%; the host-% axis is what
meaningfully protects against starting a heavy `yosys` when the machine is
already near the edge. A child tool's *own* RSS balloon is outside this
process, so `scripts/ram_guard.sh` remains the right wrapper for that — the
guard is documented as complementary, not a replacement. The decline test
arms a 1 MiB RSS ceiling (this process is far larger) so the guard trips
before the first spawn deterministically, with no tool dependency; it
no-ops where the OS read is unavailable (mem_guard's best-effort policy).

**Decision — audit log on the server, reproducible argv per call.** Each
`validate` MCP call appends a record (run_id, seed, kind, top, the
`argv.join(" ")` of every spawned tool, verdict, decline reason) to an
append-only `McpServer.audit`, exposed read-only as `anvil://audit/log`. The
`ToolInvocation.argv` already carries the binary + flags, so the audit record
is a faithful, replayable command line. A rejected call (bad tool name) is
*not* logged — it never ran.

**Decision — reuse `introspect::content_run_id` (made `pub`).** `validate`
stamps the same content-addressed `run_id` that `generate`/`introspect` use,
so an agent can correlate a validation with the artifact it introspected. One
hash, one scheme — not a second run-id source.

## 2026-06-14 — Shared downstream-tool invocation surface — `AGENT-INTROSPECTION-MCP.5.1`

`.5` (controlled `validate` + `minimize`) was split into `.5.1`/`.5.2`/`.5.3`.
`.5.1` lands the lower-level dependency the other two rest on. Notes worth
keeping:

**Decision — extract the invocations into the library; do not duplicate, do
not shell `tool_matrix`.** Decision `0004` says `validate` must run external
tools "only through the existing hardened `tool_matrix` invocations." Three
options were on the table:

1. *Duplicate* the `verilator --lint-only` / `yosys synth` / `iverilog -g2012`
   command lines (and the warning-as-failure logic) inside the MCP layer.
   **Rejected** — that is exactly the "second source of truth that can drift"
   the project's full-factorization doctrine (`feedback_full_factorization.md`)
   and `0004` forbid; the two copies would diverge the first time one tool's
   flags change.
2. Have the MCP `validate` tool *shell the `tool_matrix` binary* and parse its
   JSON report. **Rejected for `validate`** — `tool_matrix` is a scenario-sweep
   harness that builds its corpus from built-in scenarios; it has no
   "validate this one `(seed, knobs)` artifact" mode, so reusing it would
   itself require a new binary mode, and shelling a sibling binary is a heavier,
   less testable dependency than a library call.
3. *Move* the invocation primitives into the library so both the binary and the
   agent tools call one implementation. **Chosen** — this is the same move
   `DIFFERENTIAL-SIMULATION.3a` made for `src/diff_sim/` ("the helpers live in
   the library so the binary can `use anvil::diff_sim::{…}` — the
   full-factorization-doctrine choice over duplicating them in the binary").

So `src/downstream/mod.rs` now owns `ToolInvocation`, `run_tool`,
`first_tool_warning`, the per-tool runners + argv/script builders, the
double-quote escapers, `YosysMode`, and `yosys_mode_slug`; `tool_matrix`
`use`s them.

**Why it is safe (behavior-preserving).** The move changes *where* the code
lives, not *what it does*. `ToolInvocation` keeps the identical
`#[derive(Serialize, Deserialize)]` field set, so `tool_matrix_report.json`
and the `--resume` checkpoint wire shapes are byte-for-byte unchanged and the
banked Phase-1..9 reports stay valid. `YosysMode` keeps `clap::ValueEnum`
(`clap` is already a library dependency), so `--yosys-mode` parses unchanged.
The matrix's own tool tests (`summarize_tools_counts_yosys_modes_separately`,
the resume tests, …) pass unchanged, and `tests/snapshots.rs` stays 6/6
byte-identical (the DUT contract). `anyhow` is a library dependency, so the
`Result<…>` signatures moved verbatim — no error-type translation.

**Gotcha — `use std::process::Command;` became dead in the binary.** `run_tool`
was the binary's only `Command::new` caller (the diff-sim simulator runners
already live in `src/diff_sim/`), so after the move the import is unused and
`clippy -D warnings` would reject it. Dropped it in the same edit.

**Forward plan this sets up (`.5.2`/`.5.3`).** `validate(seed, knobs, tools)`
(`.5.2`) will: regenerate the artifact deterministically into a *sandboxed*
temp dir under a project-root/tmp scope (never an agent-supplied path — no
arbitrary FS write), call the `downstream::run_*` functions there, wrap the
run in the `mem_guard` envelope (and document the external `scripts/ram_guard.sh`
guard), return the structured `ToolInvocation` rows + an overall verdict, and
append an audit-log line recording the reproducible `(seed, knobs)` + the exact
argv of every spawned tool. There is **no arbitrary-shell tool** — only the
fixed, vetted `downstream` invocations. `minimize` (`.5.3`) will delta-debug
`(seed, knobs)` toward a smaller failing reproducer using `.5.2`'s `validate`
as a pure failure oracle, bounded and deterministic.

## 2026-06-14 — Read-only MCP server — `AGENT-INTROSPECTION-MCP.4`

`.4` lands the MCP bridge: `src/mcp/mod.rs` (pure dispatch + cache) + the
`anvil-mcp` stdio bin. Notes worth keeping:

**Decision — hand-rolled JSON-RPC over stdio; reject `rmcp` + `tokio`.** The
Rust MCP ecosystem has an async SDK (`rmcp`), but pulling it in drags `tokio`
and a large async surface into an otherwise sync, conservative-dependency
crate. Decision `0004` only asks for "a simple in-process MCP server (stdio
first) … no gRPC service is required." MCP's stdio transport is newline-
delimited JSON-RPC 2.0 — trivial to implement over `serde_json`. So the server
is hand-rolled: zero new dependencies, and the "thin adapter beside the core"
doctrine is honoured literally. If multi-client / HTTP demand ever appears
(`0004` open question), revisiting an SDK is a separate, owned decision.

**Decision — pure dispatch in lib, transport in bin.** `McpServer::handle` is
a pure `&Value -> Option<Value>` (notifications → `None`); `handle_line` wraps
it for the stdio framing. So the entire protocol surface — initialize
handshake, tools, resources, error codes — is unit-tested in-process (12
tests) with no child process, and `src/bin/anvil_mcp.rs` is a ~20-line
stdin→handle_line→stdout loop. This mirrors how the rest of ANVIL keeps logic
in the lib and binaries thin (`tool_matrix`, `diff_sim`).

**Decision — determinism collapses the session into a content-addressed
cache.** `generate` builds the artifact, then caches it keyed by the
introspection document's `run_id` (the FNV-1a content address from `.3`).
`resources/read anvil://artifact/<run_id>/{sv,introspection}` serves the cached
bytes back. Because `(seed, knobs) → artifact` is a pure function, the cache is
trivially sound and no nonce / stateful session is needed — exactly the `0004`
simplification over the stateful-simulator reference case.

**Decision — pure/safe tools only in `.4`; external exec is `.5`.** `generate`,
`introspect`, `dump_config` touch no filesystem and run no external tools.
`coverage_gaps` / `validate` / `minimize` need Verilator/Yosys/iverilog and so
belong with `.5`, where they run **only** through the hardened `tool_matrix`
path, sandboxed + ram-guarded. Tool-level failures (bad config) return MCP
`isError: true` content; protocol failures (unknown method/uri) return JSON-RPC
error codes — the two failure planes are kept distinct.

**Decision — explicit `[[bin]] name = "anvil-mcp"`.** A `src/bin/anvil_mcp.rs`
auto-target would be named `anvil_mcp` (file stem); an explicit `[[bin]]` gives
the hyphenated `anvil-mcp` matching `0004` and suppresses the duplicate auto
target. Separate target ⇒ the default `anvil` build and `--artifact dut`
contract are untouched (snapshots 6/6).

**Deferred to `.7`.** User-facing docs (book chapter + USER_GUIDE + README CLI
surface) are intentionally the `.7` closeout, not done per-leaf: the lane is a
stable user feature only once `.5` (validate/minimize) and `.6` (prompts)
complete it. Documenting a half-built capability in the book would violate book
doctrine more than the short deferral does.

---

## 2026-06-14 — Agent-introspection emission surface — `AGENT-INTROSPECTION-MCP.3`

`.3` implements the read-only emission surface over the `.2` schema:
`src/introspect/mod.rs` + a default-off `--introspect` CLI flag. Notes worth
keeping:

**Decision — `request.knobs` *is* the `config` section (one home).** The
schema's `config` section and the envelope's `request.knobs` are the same
effective `Config`; the emitter carries it once, in `request.knobs`. Avoiding a
duplicate top-level `config` section is the more faithful reading of "no second
source of truth" — there is exactly one home for the knobs.

**Decision — `run_id` is a content address, not a nonce.** `content_run_id` is
FNV-1a 64-bit over the canonical string `(schema_version ⏐ anvil_version ⏐ lane
⏐ seed ⏐ serde_json(knobs))`. `serde_json::to_string(&Config)` is deterministic
(declaration field order; BTreeMap-sorted nested maps), so identical inputs
yield an identical `run_id` — exactly the content-addressed cache key `0004`
relies on. The hash function is an implementation detail (the schema only
requires purity + a hex string); it can change in a later leaf without touching
the contract.

**Decision — single-artifact stdout only; reject `--out` / `--count > 1`.**
Introspection is a single-artifact view. Restricting `--introspect` to the
`(None, 1)` stdout path keeps the streamed `--out` manifest path (and its
governor checkpoints) completely untouched, so the default `--out` flow stays
byte-identical and the surface's contract stays unambiguous. A guard bails with
a clear message otherwise.

**Decision — derive `Serialize` + `Deserialize` on the whole envelope.** The
document round-trips through JSON (tested), which both proves the shape is
well-formed and gives the future MCP server (`.4`) a typed consumer for free.
`PartialEq` is *not* derived (`Config` does not implement it); tests compare via
`serde_json::Value` instead, which also directly asserts the SCHEMA-DERIVED
invariant (`request.knobs` == input `Config`; `module_metrics` == `compute`).

**Scope held for later leaves.** DUT lane only; `coverage` (a `tool_matrix`-run
property) and the `microdesign`/`frontend` lane-manifest sections are deferred
to `.4`+ and flagged at runtime with a `warnings[]` note. Byte-identical DUT
contract verified by snapshots 6/6.

---

## 2026-06-14 — Agent-introspection schema contract — `AGENT-INTROSPECTION-MCP.2`

The `.2` leaf pins the introspection **schema** — the contract the `.3`
emission surface and the `.4` MCP server must conform to. Full spec:
`docs/AGENT_INTROSPECTION_SCHEMA.md`. Docs-only; no code. The load-bearing
reasoning:

**Decision — a thin versioned *envelope* around existing facts.** The
top-level document carries `schema_version` (`"1.0"`), `anvil_version`
(`env!("CARGO_PKG_VERSION")` = `0.1.0`), `lane`, a `request` echo of the
`(seed, knobs, lane)` determinism tuple plus a content-addressed `run_id`, an
`artifact` descriptor (`.sv`/manifest as fetch-on-demand `ResourceRef`s, not
inlined), the `introspection` payload, and `warnings`. Only the envelope is
new — and this doc is its single source of truth.

**Decision — invariant SCHEMA-DERIVED: zero new computed truth.** Every
payload section is the *exact `serde` projection* of a struct that already
exists and already runs: `config` ← `Config` (`src/config.rs`),
`module_metrics` ← `Metrics`, `design_metrics` ← `DesignMetrics`
(`src/metrics.rs`, via `compute`/`compute_design`), `coverage` ←
`CoverageSummary` incl. `coverage_gaps` (`src/bin/tool_matrix.rs`),
`microdesign_manifest`/`frontend_manifest` ← the lane `Manifest` structs. A
conforming emitter MUST `serde`-serialize the live value, never re-derive
fields — so the schema can never become a second source of truth that drifts
(the `0004` anti-drift principle, made mechanical).

**Why not re-list every metric field in the spec.** Re-typing the ~60
`Metrics` + ~140 `DesignMetrics` fields into the doc would *be* the forbidden
second source of truth. So "lists every field + provenance" is satisfied at
the correct granularity: every **envelope** field is listed explicitly (the
doc owns them); every **embedded section** is mapped to the struct/file/
producer that owns its fields, and the struct's *category groups* are
enumerated so coverage is visible without mirroring. Leaf field lists stay in
code, enumerated by `serde` at emit time.

**Decision — `coverage` is a matrix-run property, not a single-artifact
one.** A lone module cannot prove `saw_recursive_hierarchy_*`; the section is
present only when the producing call ran `tool_matrix`, otherwise absent with
a `warnings[]` note. Keeps provenance honest.

**Decision — versioning policy.** `MAJOR.MINOR`. Additive growth via
`#[serde(default)]` fields (already pervasive on `Config`/`Metrics`) is
MINOR/compatible; rename, retype, unit change, semantic change, or section
removal is MAJOR. `anvil_version` travels alongside so an agent separates
"newer generator, same shape" from "newer shape". Emitters advertise the
version(s) they produce and refuse an unsupported request explicitly; a bump
never introduces wall-clock/host/random data, preserving determinism.

**Status.** With `.1` (architecture) + `.2` (schema) landed, the lane design
is complete; the first code leaf `.3` is parked on owner acceptance.

---

## 2026-06-14 — Agent introspection + MCP lane design — `AGENT-INTROSPECTION-MCP.1`

Owner-directed new capability lane: make ANVIL agent-drivable (an LLM can
generate, introspect, validate, and triage via MCP). Full architecture and
the transferred-vs-dropped analysis of the RTL-simulator MCP reference
advice live in `docs/decisions/0004-agent-introspection-mcp-lane.md`; the
tree is `docs/tasks/AGENT-INTROSPECTION-MCP.md`. The load-bearing points:

**Decision — MCP beside the core, never in the kernel.** The deterministic
generator stays untouched; MCP is a thin read-mostly adapter over the
existing library API (`Generator`/`Config`/`metrics`/`manifest`). Separate
default-off target ⇒ the default `anvil` build and the `--artifact dut`
byte-identical contract are unaffected. "Machine-controllable first,
MCP-exposed second."

**Decision — the introspection schema is *derived*, not a second source of
truth.** It is assembled from the existing `metrics`/`DesignMetrics`,
expected-facts manifests, and config echo — same anti-drift principle as
the Knowledge Map. ANVIL *is* the oracle, so this is construction-truth
(DepSet, motif/rule provenance, coverage facts), not inference from parsed
SV.

**Decision — determinism collapses the "service session" into a
content-addressed cache.** Unlike a stateful RTL simulator, ANVIL artifacts
are pure functions of `(seed, knobs, lane, version)`. A simple in-process
stdio MCP server with a `(seed,knobs,lane,version)`-keyed cache suffices;
no gRPC/service layer is needed for correctness or performance.

**Rejected (simulator-specific, deliberately not copied).**
- *Stateful session API* (`run_until`/`force_signal`/waveform DB/
  signal-over-time/`explain_x`/sensitivity trees/stepping) — ANVIL has no
  temporal session; this would invent state it does not have.
- *MCP in the kernel* — couples the deterministic core to integration.
- *LLM as signoff oracle / any output-mutating API path* — violates
  rules-first / valid-by-construction; the agent drives experiments and
  explains, ANVIL stays the source of truth.
- *Raw-shell tool* — only fixed, vetted invocations; the `validate` tool
  reuses the hardened `tool_matrix` invocations, sandboxed + ram-guarded,
  with deterministic run ids + audit log.

**Phasing.** `.1` design (this) → `.2` schema spec (docs) → `.3` emission
surface (code) → `.4` read-only MCP server → `.5` controlled
validate/minimize → `.6` prompts → `.7` book/closeout. Design-first: no
code until `.1`/`.2` are accepted.

---

## 2026-06-14 — Internal RAM/RSS self-governor — `WORKLOAD-MEMORY-SAFETY.4`

Landed `.4`, the process-level backstop the `.1` design recorded: an
opt-in governor that lets `anvil` sample its own RSS and/or host
used-RAM% and abort a bulk `--out` run cleanly before the host danger
zone. New module `src/mem_guard.rs`; two knobs `max_rss_mb` /
`ram_abort_pct` (sentinel `0` = off); CLI flags `--max-rss-mb` /
`--ram-abort-pct`.

**Decision — separate the pure decision from the I/O.** `evaluate(&MemLimits,
&MemSample)` is a pure function (no syscalls), so the trip logic —
boundaries, axis precedence, never-abort-on-`None` — is exhaustively
unit-tested without touching the OS. The OS reads (`read_process_rss_mb`,
`read_host_used_pct`) are thin best-effort wrappers. This is why the
acceptance "focused tests for the decision logic" is satisfiable cleanly:
the decision *is* a function.

**Decision — sample between units, never mid-cone.** Rules-first +
valid-by-construction (`feedback_rules_first_generation`): the check runs
at the *start of each iteration* of the `--out` streaming closures, so the
governor declines to start the next module/design rather than truncating a
half-built cone (which would emit invalid RTL). This is the
"decline-to-start-more" mechanism the `.1` design mandated. The per-module
node budget (`.3`) is the orthogonal *single-module* construction-time
bound; `.4` is the *process* bound. Sampling deep inside the hot cone
worklist (the `.1` "stretch") is deliberately deferred — it would touch
the hot path for marginal benefit now that `.3` already caps a single
module — and is recorded as a `.5` deferred boundary.

**Decision — RSS checked before host-%.** A single fast-growing module can
balloon this process's RSS faster than the host %-used signal moves, and
the external `ram_guard.sh` 3 s poll can miss it entirely; the per-process
RSS bound is the more urgent guard, so `evaluate` tests it first.

**Decision — best-effort reads never abort a healthy run.** Mirrors
`scripts/ram_guard.sh` ("a probe hiccup never kills a healthy job"): an
unreadable `/proc` file or a failed `ps`/`memory_pressure` yields `None`,
and `None` never trips. The reads are also dep-free (no `sysinfo` crate):
Linux uses `/proc`; macOS shells the same tools the shell guard uses.

**Decision — exit code 99, distinct from validation/other errors.** A
governor trip exits `99` (matching `ram_guard.sh`), so a wrapping script
can tell "governor stopped me" from a config error (exit 1) or a normal
failure. Implemented without fighting the streaming writer: the closure
returns an `io::Error` of kind `OutOfMemory` carrying the message; `main`
matches that kind, prints the message, and `process::exit(99)`.

**Decision — process-safety knob is exempt from the output-metric
doctrine.** `book/src/knobs.md`'s "every knob needs a metric for its effect
on *generated output*" does not apply: the governor has no output effect
(off ⇒ byte-identical; fired ⇒ no further output). Forcing a per-module
`Metrics` field would be dishonest. The decision-logic tests + the clean
abort are the observability. This categorization is recorded in the book
and is a deliberate, defensible reading of the doctrine, not an evasion.

**Rejected / deferred.**
- *A per-module `Metrics` RSS field* — rejected: the governor is run-level,
  not per-module-shape; a per-module metric would misrepresent it (see
  above).
- *Pulling in the `sysinfo` crate* — rejected: a new dependency for two
  small reads the shell guard already does dep-free; `/proc` + `ps` /
  `memory_pressure` match the established approach exactly.
- *Sampling inside the cone worklist drain* — deferred to a possible future
  leaf: `.3`'s node budget already bounds a single module construction-time;
  intra-cone RSS sampling would touch the hot path for marginal gain.
- *Guarding the `count == 1` stdout path and the non-DUT lanes* — out of
  scope: those emit one small artifact, not a bulk loop; documented as a
  `.5` deferred boundary.

Validation: `cargo test --lib mem_guard` 11/11 + config 2/2 + bin 2/2;
`cargo test --test snapshots` 6/6 (default SV byte-identical); clippy
`-D warnings` + fmt clean; live exit-99 / byte-identical smokes;
`mdbook build` clean. Heavy builds under `scripts/ram_guard.sh`.

---

## 2026-06-14 — cone.rs decomposition design — `CONE-DECOMPOSITION.1`

Owner asked (2026-06-14) to "carefully and meticulously" break the
5551-line `src/gen/cone.rs` into interconnected parts. This is a **pure
structural refactor** — zero behaviour change, byte-identical generated
RTL — tracked by `docs/tasks/CONE-DECOMPOSITION.md`.

**Seam map (target `src/gen/cone/`):** `snapshot.rs` (rollback machinery),
`semantic.rs` (value-set / unsigned-bounds / exact-value proofs — the
largest, purely `&Module` chunk, ~1360 lines), `primitives.rs` (IR-building
gate makers), `terminals.rs` (terminal/pool selection + gate-shape policy),
`flops.rs` (flop drains + D assemblers), `motifs.rs` (block/motif builders).
The **strategy orchestration** stays in the root `cone.rs`:
`build_cone_with_retry`, `build_graph_first`, `grow_pool_one_unit`,
`build_outputs_interleaved`, `process_signal_frame`, `deliver`,
`build_cone`, `roll_knob`, `node_budget_reached`, the `Dest`/`SignalFrame`/
`GateFrame` types, the `FlopWorklist` alias, and `#[cfg(test)] mod tests`.

**Rust mechanic.** Rust 2018 lets `src/gen/cone.rs` coexist with a sibling
`src/gen/cone/` directory; the root declares `mod snapshot;` etc. No rename.

**Visibility — flat namespace via root glob re-export.** The original file
is one all-see-all namespace. To preserve that with minimal churn, each
moved fn becomes `pub(crate)`, and the root does
`mod <name>; pub(crate) use <name>::*;` per submodule. That (a) keeps every
external path stable — `crate::gen::cone::<symbol>` still resolves for the
callers in `src/gen/module.rs`, `src/gen/hierarchy.rs`, and `src/ir/compact.rs`
(which uses `cone::obvious_unsigned_compare_result` and
`cone::prove_node_exact_value_from_bounds`) — and (b) lets each submodule's
`use super::*;` see all sibling items. The existing test module already
uses `use super::*;`, so wildcard imports are accepted by the lint config.

**Validation protocol (byte-identical-or-bust).** A pure code move that
compiles + passes the 307 lib tests (incl. the 42 cone tests + the
`node_budget` test) + the 6 SV snapshots is behaviour-preserving. Full
`cargo test` runs at the first extraction (`.2`, validating the mechanic
end-to-end) and at closeout (`.7`); intermediate leaves use
`cargo check --all-targets` + `cargo test --lib` + `cargo test --test
snapshots` + clippy + fmt, all under `scripts/ram_guard.sh`. A snapshot
byte-diff means the move is wrong — fix the move, never accept the
snapshot.

**Order — most self-contained first.** `snapshot` (tiny, proves the
mechanic) → `semantic` (biggest readability win, pure fns) → `primitives`
→ `terminals` → `flops` → `motifs`.

**`.2` execution note (reusable gotcha for later leaves).** When a struct
moves to a submodule but the *tests stay in the root*, any test that reads
the struct's fields breaks (`E0616 private field`). Fix: bump exactly the
fields the tests touch to `pub(crate)` (done for `ConstructionSnapshot`).
The same will apply wherever a moved type's internals are asserted by a
root-resident test. Mechanic confirmed end-to-end: `mod snapshot;
pub(crate) use snapshot::*;` keeps the symbols reachable from the root and
re-exported, full suite byte-identical.

**Proven extraction recipe (used for `.2` snapshot + `.3` semantic; reuse
for `.4`–`.6`).** For a large contiguous block `[A,B]` in `cone.rs`:
1. `sed -n 'A,Bp' cone.rs > cone/<name>.rs` (exact byte-for-byte copy).
2. `perl -i -pe 's/^(fn |struct |enum )/pub(crate) $1/' cone/<name>.rs`
   (bump column-0 items; already-`pub(crate)` lines aren't matched;
   `impl` blocks are left alone). *Note:* BSD `sed -i ''` choked on the
   `-E` script here — use `perl` for the in-place edits.
3. Prepend the module header: doc comment + `use crate::ir::{…}` (the IR
   types the block uses) + `use super::{…}` for any root/sibling symbols
   the block calls (the compiler's `E0425` lists them — e.g. `semantic.rs`
   needed only `use super::node_deps;`).
4. `perl -i -ne 'print unless $. >= A && $. <= B' cone.rs` (delete the
   moved block).
5. Add `mod <name>; pub(crate) use <name>::*;` to the root.
6. `cargo fmt` (collapses the double blank line left at the seam),
   `cargo check --all-targets`, then fix the imports the compiler names.
7. Gate: `cargo test --lib` + `cargo test --test snapshots` (byte-identical)
   + clippy `-D warnings` + fmt. Full suite at the milestones (`.2`, `.7`).
GOTCHA beyond the field-visibility one above: an import used *only* by the
moved code becomes unused in the root — move it (e.g. `HashMap` migrated
into the test module for `.3`, since the tests still reach it via
`use super::*`).

---

## 2026-06-14 — Per-module construction-time node budget — `WORKLOAD-MEMORY-SAFETY.3`

Turned `max_nodes_per_module` from a **ghost knob** (declared + defaulted
to `1000`, enforced nowhere) into a real, rules-first construction-time
budget that bounds one module's `Vec<Node>` arena.

**Where the budget lives — the `force_leaf` funnel.** The recursion has
exactly two "terminate vs. grow" decision points, both structured
identically as `force_leaf`:
`src/gen/cone.rs::process_signal_frame` (the interleaved/default strategy)
and `src/gen/cone.rs::build_cone` (the recursive builder used by the
sequential/shuffled strategies, flop-D cones, and every motif sub-cone).
A new `node_budget_reached(g, m)` helper is OR-ed into both `force_leaf`
expressions as the **first** term, plus a `break` in
`build_graph_first`'s pool-growth loop (the only strategy that doesn't go
through a `force_leaf`). Once the arena reaches the budget, every further
recursion point forces a terminal pick — steering to existing signals
instead of opening new sub-cones.

**Why steering, not truncation.** Rules-first + valid-by-construction
(`feedback_rules_first_generation`): forcing `force_leaf` reuses the
*exact* path the depth limit already takes (`pick_terminal`), which always
returns a legal dep-bearing terminal, so every cone still closes and every
output keeps dep-set ≥ 1. We never cut a half-built cone (that would emit
invalid RTL). Hence a **soft** ceiling: a bounded number of
terminal/adapter nodes may still be appended to close frames already
in-flight when the budget is crossed — the test allows generous slack
rather than asserting exact equality.

**Why the default had to change `1000` → `0`.** The knob was inert, so
*enforcing* it at the old default would have silently changed output for
any module exceeding 1000 nodes — a reproducibility break. Sentinel
`0 = unlimited` keeps the default path byte-identical: `node_budget_reached`
returns `false`, so the `force_leaf` expression reduces to the original
`depth >= max_depth || gen_bool(...)` with identical RNG consumption (the
`||` short-circuit means the budget term, being `false`, changes nothing).
The only observable default-path change is the `--dump-config` /
`manifest.json` config echo (`1000` → `0`) — config JSON, **not** SV, so
`tests/snapshots.rs` stays green without acceptance. Effect measured by
the existing `Metrics::num_nodes` (knob-effectiveness map updated).

**RNG note.** When the budget *does* fire (non-default), it short-circuits
before `gen_bool`, so the RNG stream differs from the unbounded run — but
that is expected and deterministic: the budget is a knob, and
reproducibility is per-`(seed, knobs)`.

Validation: `cargo test --lib node_budget` (caps + shrinks + stays valid),
`cargo test --test snapshots` 6/6 (default-path SV byte-identical), clippy
`-D warnings` + fmt clean, full `cargo test` under
`scripts/ram_guard.sh --threshold 88`. Book `knobs.md` updated (the knob
was previously mis-described as an enforced "hard cap" — that drift is now
true).

---

## 2026-06-14 — Streaming manifest writer (byte-identical) — `WORKLOAD-MEMORY-SAFETY.2`

Landed `.2`: `src/manifest.rs` + rewired both `--out DIR` lanes in
`src/main.rs` to stream the manifest array element-by-element instead of
accumulating a `Vec<serde_json::Value>` and `to_string_pretty`-ing it at
the end. Peak metadata memory drops from O(`--count`) to O(1).

**Gotcha — reproducing serde_json's pretty bytes by hand is fragile;
don't.** The byte-identical contract is enforced *without* hand-rolling
the JSON framing:

1. **Framing comes from serde, not from me.** I serialise the same
   top-level object with the array key bound to a unique placeholder
   *string* (`"__ANVIL_STREAM_ARRAY_PLACEHOLDER__"`), then split the
   serde output around that quoted token. This captures serde's exact
   key ordering (serde_json sorts object keys — the crate does **not**
   enable `preserve_order`, so top-level keys come out `config` <
   `modules`/`designs` < `seed`), the `seed`/`config` rendering, commas,
   and the trailing brace — none of it guessed.
2. **Elements come from serde, re-indented.** Each element is
   `to_string_pretty`'d standalone, then every interior line is prefixed
   by the constant base indent (`\n` → `\n    `). Pretty-print
   indentation is purely a function of nesting depth, so a constant
   prefix reproduces exactly the nested bytes. The array's element indent
   (4 spaces) and closing-bracket indent (2 spaces) are serde invariants
   for an array that is a direct child of the root — which the manifest
   array always is.

`streamed_matches_reference[_for_designs]` proves (1)+(2) against serde
itself for counts 0/1/2/5/17 and nested designs. Belt-and-suspenders:
an actual old-vs-new `diff -r` of `--seed 42 --count 5` (flat) and a
depth-1 wrapper design came back **byte-identical** across `manifest.json`
and every `.sv`.

**Lint gotcha.** rustc 1.95's clippy promotes
`io::Error::new(io::ErrorKind::Other, e)` to a warning — use
`io::Error::other(e)` (applies to the new `manifest::io_err` helper and
the `main.rs` validate-error mapping).

Validation: `cargo check --all-targets`, `cargo clippy --all-targets -D
warnings`, `cargo fmt --check`, `cargo test --lib manifest`,
`cargo test --test snapshots` (6/6 SV byte-identity), plus the old-vs-new
`diff -r`. Heavy builds wrapped in `scripts/ram_guard.sh --threshold 88`.

---

## 2026-06-14 — Workload memory-safety design (bounded-memory generation) — `WORKLOAD-MEMORY-SAFETY.1`

**Problem.** ANVIL has no internal defence against driving a RAM-limited
host to the danger/reboot zone on a huge workload. Two distinct
unbounded-growth vectors exist today, plus one missing per-module bound:

1. **Cross-run metadata accumulation (unbounded in `--count`).** In the
   directory-output path (`src/main.rs:507-575`) the per-artifact JSON
   metadata is accumulated in a `Vec` before a single final
   `manifest.json` write. Flat lane (`main.rs:551-575`): `let mut manifest
   = Vec::new()` grows one full-metrics JSON object per module across all
   `n` iterations. Hierarchical lane (`main.rs:509-550`): `let mut designs
   = Vec::new()` grows per design, and each design additionally holds **all
   its modules** in `design.modules` while emitting. The emitted `.sv`
   itself *is* already streamed (generate → `emit::to_sv` to a `String` →
   `std::fs::write` → `String` dropped; the previous module is dropped
   before the next is generated — the `Generator` retains no module
   history, only `rng` + `cfg` + `next_module_index`). So the leak is the
   metadata `Vec`, not the modules. `--count 1_000_000` accumulates a
   million metrics objects in RAM before writing anything.

2. **No per-module construction-time node bound.** `max_nodes_per_module`
   is a **ghost knob**: declared at `src/config.rs:337`, defaulted to
   `1000` at `src/config.rs:729`, and **read/enforced nowhere** (`grep -rn
   max_nodes_per_module src/` returns only those two lines). A pathological
   `(seed, knobs)` — high `--max-depth`, high arity, low sharing — can grow
   a single module's `Vec<Node>` arena (the dominant per-module cost; each
   `Node::Gate` also carries an `operands: Vec<NodeId>` and a `DepSet`
   wrapping a `BTreeSet`) without any internal ceiling.

3. **No internal RAM/RSS governor.** `scripts/ram_guard.sh`
   (`RESOURCE-SAFE-TOOLING`) guards *external* heavy jobs from the outside;
   nothing makes the `anvil` process itself notice it is ballooning and
   stop cleanly. A single fast-growing module can outrun a 3 s external
   poll.

**Design — three mechanisms, all default-off / byte-identical.** This is
the load-bearing constraint: every prior capability knob (`multi_clock_prob`,
`aggregate_prob`, `memory_prob`, `fsm_prob`) shipped defaulting to the
no-op value, and the reproducibility contract (`book/src/knobs.md`) is
non-negotiable. So none of the below may change default SV output;
`tests/snapshots.rs` + `tests/book_examples.rs` must stay green without
snapshot acceptance.

- **`.2` Stream the manifest (bounded in `--count`).** Replace the
  accumulate-then-write `Vec<serde_json::Value>` with an incremental writer
  that emits the *same* JSON array bytes (`[`, comma-separated pretty
  elements, `]`) as the current `serde_json::to_string_pretty` produces,
  so `manifest.json` is byte-identical while peak metadata RAM drops from
  O(`--count`) to O(1). The hierarchical lane additionally must not be made
  worse; a `Design`'s own module set is intrinsic to that design and is
  emitted then dropped per design (already O(one design), not O(`--count`)).

- **`.3` Real per-module node budget (rules-first).** Wire the budget into
  the cone-construction recursion so that, as the budget is approached,
  construction *prefers terminal reuse / stops opening new sub-cones*
  (rules-first per `feedback_rules_first_generation`) — it never truncates
  a finished cone (that would emit invalid RTL and break
  valid-by-construction). Default must preserve byte-identical RTL:
  treat the budget as a sentinel `0 = unlimited` and change the default
  from `1000` → `0`. Only `--dump-config` / the `manifest.json` config echo
  shift (config JSON, **not** SV output, **not** the SV snapshots which key
  on emitted RTL). A `Metrics` field must measure realized node count vs
  budget (knob-effectiveness doctrine, `book/src/knobs.md`).

- **`.4` Internal RAM/RSS self-governor (opt-in).** An opt-in knob makes
  `anvil` sample its own RSS (and optionally host %-used, reusing
  `ram_guard.sh`'s macOS `memory_pressure` / Linux `/proc/meminfo`
  approach) at safe checkpoints (between modules, and — stretch — at cone
  worklist-drain boundaries) and abort with a deterministic non-zero exit
  and a message naming the seed + effective knobs, *before* the host danger
  zone. Default unset ⇒ no sampling ⇒ byte-identical. This catches the
  single-pathological-module case that `.2`/`.3` and the external watchdog
  can each miss.

**Rejected / deferred.**
- *Generate-then-filter a too-big module away* — rejected: violates
  rules-first + valid-by-construction; the whole point is construction-time
  steering.
- *Sub-seeding the RNG per module to parallelise/bound* — rejected here:
  would break the serial-RNG reproducibility contract
  (`book/src/knobs.md` "The RNG is not sub-seeded per module").
- *Changing `manifest.json` to JSON-lines by default* — deferred to a
  possible `.2` opt-in sidecar; the default stays the byte-identical
  pretty-printed array.
- *Enforcing `max_nodes_per_module` at its current `1000` default* —
  rejected: it is currently inert, so enforcing at `1000` would silently
  change output for any module exceeding 1000 nodes (a reproducibility
  break). Hence the sentinel-`0`-unlimited default.

No code changed in this leaf (design only). Validation: docs-only;
memory-architecture + knowledge-map self-checks; `git diff --check`. Full
`cargo test` intentionally skipped (no code change; full-suite RAM risk per
`docs/decisions/0003-resource-safe-validation.md`). Tracked by
`docs/tasks/WORKLOAD-MEMORY-SAFETY.md`.

---

## 2026-06-14 — mdBook drift correction (delivered motifs were labelled "future") — `LIVE-DOC-BOOK-ALIGNMENT.1`

A live-doc/mdBook audit found the user-facing book still described
several **delivered** capabilities as future work, violating the
no-drift mandate (`book/src/synthesizability.md` "Memories (future…)";
`book/src/ir.md` "Future extensions … not yet implemented" whose body
already carried "Delivered" tags; the `### Parameters and generics
(Phase 5)` subsection written entirely as unbuilt; `book/src/faq.md`
"future memories / FSM"). All were corrected to present-tense
delivered framing sourced from `ROADMAP.md` / the code's own docs.

**Protected-file justification (`book/src/core-idea.md`).** COMMIT.md
forbids casual edits to `core-idea.md` / `non-goals.md` /
`why-not-grammar.md`. The single edit here changed "Future motifs
(FSMs, memories, parameterized sub-designs) **should be** added by
extending the recursion's choice set" → "Advanced motifs (…) **are**
added by …". This is a tense-only factual correction (those motifs are
delivered); the load-bearing design decision — *extend the recursion,
never wrap it in iterative scaffolding* — is preserved verbatim. No
design decision was altered, added, or removed.

No code, IR, knob, or generated-output change. Validation: `mdbook
build book` clean; memory-architecture + knowledge-map self-checks
pass; `git diff --check` clean. Full `cargo test` intentionally skipped
(no code changed; full-suite RAM risk per the resource-safe-validation
policy in `docs/decisions/0003-resource-safe-validation.md`).

---

## Core design decisions (recap)

These are documented in detail in the mdBook. They are restated here only as anchors:

- **Recursion is the core principle.** Every non-trivial generation step is a recursive descent over the typed circuit graph. Iteration is the exception, used only where termination or ordering genuinely require it (e.g., the flop worklist drainer, the per-output driver loop). When in doubt, recurse. See `book/src/core-idea.md` "The single guiding principle".
- **Synchronous-design discipline.** Every stateful module is fully
  synchronous to one or more declared clock/reset domains. The K=1
  default is one `clk` (posedge) and one `rst_n` (async, active-low);
  the K=N path declares `ClockDomain` entries and tags flops through
  `Module.flop_domains`. Enforced by construction — there is no IR
  field for arbitrary per-flop clock expressions or per-flop reset
  polarity. See `book/src/sequential.md` "Synchronous-design
  discipline".
- **Flop-D mux motifs.** Every flop's D input is constructed from one of: M=0 (direct cone), M≥2 OneHot (OR-of-masked arms), M≥2 Encoded (chained ternary over `Eq(sel, k)`). M=1 is excluded by design; it collapses to a wire. The style (OneHot vs Encoded) and kind (ZeroDefault vs QFeedback) are chosen per-flop and orthogonal — four motif variants plus the M=0 plain register. See `book/src/sequential.md` "Flop motifs".
- **Q-feedback freedom (revised).** A flop's own Q may appear freely — any number of times — as a leaf in any of its data, select, or direct-D sub-cones. The clock edge breaks the Q→D loop temporally; this is the standard synchronous feedback pattern (counters, accumulators, state machines). Independently, `FlopKind::QFeedback` adds an explicit Q fall-through term in the mux when no select fires. Both are legal; both can be active at the same flop. Combinational self-reference (Rule 1) is still forbidden. See `book/src/structural-rules.md` Rules 2 and 3.
- **Structural rules catalog.** Every load-bearing generator invariant is documented in `book/src/structural-rules.md`. That chapter is the durable source of truth — new rules land there as they become invariants. Inline design-decision recaps in this file should *point* to the catalog, not duplicate rule text.
- **Operators vs blocks.** Load-bearing conceptual distinction. An operator is an associative primitive function; its generalization is **arity** (N same-width operands). A block is a functional unit with internal structure; its generalization is **ports / port counts / arms**, encoding choices, feedback topology. Arity is operator vocabulary only — blocks have ports, not arity. `And / Or / Xor / Add / Mul` are operators and got N-arity in `2026-04-15-0015`. `Sub` is not associative and stays 2-arity. `Mux` and `Flop` are blocks and are governed by block rules, not arity knobs. See `book/src/structural-rules.md` "Operators vs blocks" preamble and Rule 14.
- **Roles of constants in RTL.** Integer literals appear as operands with three *distinct* semantic roles: **coefficient** (multiplicative weight in arithmetic linear combinations; per-op constraints: Add `ci ≠ 0`, Sub `ci > 0` strictly positive, Mul TBD), **shift amount** (structural parameter of `Shl/Shr` — `a << 2`; constant-amount vs variable-amount are both legal, with real designs biased heavily toward constant), and **comparand** (threshold / sentinel on the RHS of a comparison — `a == 7`; additive to signal-vs-signal comparisons, not a replacement). These three are *not interchangeable*: each has its own motif family, its own constraints, and its own knob(s). Do not unify them under a single `constant_prob` knob — doing so loses the semantic distinctions. See `book/src/structural-rules.md` "Roles of constants in RTL".
- **Construction strategies.** Three live strategies construct a
  module's internal logic: `sequential` (per-output cone recursion in
  declaration order), `shuffled` (same, randomised output order), and
  `interleaved` (frames interleaved via random-pop work queue — cones
  grow in lockstep). `graph-first` remains as a deprecated CLI/config
  alias for `interleaved`; the original speculative pool-growth
  implementation is retired. The strategy is a property of **how** the
  generator builds; the emitted SV is a DAG regardless. Different
  strategies produce different output *distributions*
  (declaration-order bias, within-module sharing symmetry). See
  `book/src/construction-strategies.md`.
- **Circuit IR over annotated EBNF.** The generator builds a typed circuit graph and emits SV from it. See `book/src/why-not-grammar.md`.
- **Generation by construction, not generate-then-filter.** Validity is structural; the validator is a safety net, not a gate. See `book/src/by-construction.md`.
- **Synthesizability is a subset constraint.** The gate set, flop
  pattern, and emitter cover only the synthesizable subset. Broader
  artifact families must keep that contract too; the project is
  broadening to more kinds of valid-by-construction synthesizable
  artifacts, not abandoning synthesizability. See
  `book/src/synthesizability.md`.
- **Non-triviality via dep-set tracking + structural anti-collapse
  rules.** No bundled oracle. Expected-facts manifests for specific
  artifact families are acceptable; a shadow simulator used as a global
  filter is not. See `book/src/non-triviality.md`.
- **Random by-construction synthesizable RTL is the product goal.**
  `anvil` is not trying to be merely "valid enough". The target is a
  signoff-level quality random synthesizable RTL generator whose outputs
  are accepted by mainstream downstream HDL consumers by default and
  remain rich enough to expose real bugs in parsers, elaborators, RTL
  compilers, linters, simulators, synthesizers, and similar tools.
  Feature growth and downstream-acceptance robustness are both
  first-class; neither is optional garnish for the other.
- **No oracle, no reference simulator.** `anvil` is still a generator,
  not a bundled shadow simulator. It can stress downstream tools by
  emitting high-quality legal RTL and explicit expected-facts contracts
  where appropriate, not by embedding a second implementation of RTL
  semantics. See `book/src/non-goals.md`.

If you need to revise any of these, that is a deliberate task with its own commit and a `DEVELOPMENT_NOTES.md` entry.

---

## Design notes
### Icarus compile axis and static structured-gate lowering (2026-06-05, SIGNOFF-SURFACE-EXPANSION.3)

`tool_matrix --iverilog-compile` is an acceptance column, not a
semantic agreement column. It shells `iverilog -g2012` over each
emitted module/design, records the result under `iverilog_compile`,
and treats warnings as failures. It deliberately does not run `vvp` or
compare traces; that remains the job of `--diff-sim`, whose harness has
a testbench and normalizes Icarus/Verilator traces.

The first Icarus sweep exposed a real warning class rather than a
syntax failure: constant-controlled structured blocks can leave
Icarus with an `always_comb` block that has no effective sensitivity.
The fix is emitter-local and semantics-preserving: dynamic case/casez
selectors and dynamic for-fold sources still emit the intended
procedural surfaces, while constant selectors/sources lower to a
continuous `assign` of the selected arm, default zero, or folded
literal. This keeps the user-visible structured surfaces for dynamic
cases and removes empty-sensitivity warnings in strict frontends.

Rejected alternatives:

- Reclassify Icarus warnings as acceptable. The matrix convention is
  "warning-clean means no warnings", and weakening only one column
  would hide useful signal.
- Switch all structured blocks from `always_comb` to `always @*`.
  Icarus still warns when `@*` has no sensitivity, so the change would
  not solve the issue.
- Run full diff-sim for every matrix artifact. That remains too
  expensive; the existing per-axis `--diff-sim` subset is the semantic
  agreement gate.

### Verilator JSON frontend parity extractor (2026-06-05, SIGNOFF-SURFACE-EXPANSION.2)

The richer Phase-8 AST/source parity follow-up landed through
Verilator JSON, not Verilator XML. Local evidence: Verilator 5.046
rejects `--xml-only` but supports `--json-only`,
`--json-only-output`, and `--json-only-meta-output`; `slang` was not on
`PATH`.

The extractor deliberately stays in `tests/frontend_parity.rs` because
it is a signoff harness, not production DUT generation. It parses
Verilator's netlist JSON for the Phase-8 source-level frontend lane:
top `MODULE` GPARAM/LPARAM `VAR.valuep[CONST]` facts become top
params/localparams; `PACKAGE` LPARAMs become `pkg::name` constants;
top `CELL.modp` links resolve each instance to Verilator's specialized
child module, whose `origName` is the source child and whose GPARAMs are
the resolved instance bindings; surviving `GENBLOCK` names recover the
generate branch. That makes the Verilator gate stricter than the Yosys
gate for this lane: Yosys still covers 5 of 7 categories because it
folds package constants and top localparams, while Verilator JSON
enforces all 7 categories through `ParityScope::all()`.

Rejected alternatives:

- Keep waiting for `slang --ast-json`. It is not present locally, and
  this slice can add a real available richer gate without making slang
  mandatory.
- Keep the old `verilator --xml-only` wording. It is false for the
  local Verilator build and would turn a concrete available gate into a
  stale aspirational one.
- Move the extractor into `src/frontend/`. The production frontend lane
  already emits its own manifest and comparator types; downstream-tool
  JSON parsing is harness wiring and belongs in the integration test
  where the optional real-tool gate lives.

### Bounded semantic module identity (2026-06-05, HIERARCHY-SEMANTIC-IDENTITY.1/.2)

`Config::hierarchy_semantic_module_dedup` is a separate default-off
module identity pass, not a broadening of the existing structural
`hierarchy_module_dedup` knob. The structural pass remains
canonical-signature-only; the semantic pass groups modules by a bounded
whole-module truth-table proof and runs only when the config asks for it
under `identity_mode = node-id` with effective `factorization_level =
e-graph`.

The proof boundary is deliberately narrow: non-top, pure
combinational, state-free, concrete modules only; same emitted data
input/output interface by `(PortId, width)`; <= 12 emitted input-support
bits; <= 128 reachable output-cone nodes within the work budget; and
<= 128-bit outputs. The supported classes are instance-free modules and
bounded pure-combinational wrappers with <= 8 child instances, where
every child is itself inside the proof boundary and every instance has
concrete, non-parameterized bindings. The full proof object, not its
compact hash, is the merge key. The hash exposed in
`DesignMetrics.semantic_module_signatures` is only observability.

The `(PortId, width)` interface key is non-negotiable. Module dedup
rewrites `Instance.module` names but intentionally leaves parent-side
`(port_id, node)` bindings alone. Merging modules whose public port IDs
differ would make a previously valid instance bind a signal to the
wrong child port. Width-only or name-only interface matching was
therefore rejected.

Rejected alternatives:

- Reuse `hierarchy_module_dedup` and silently make it semantic. That
  would change an established config meaning and break the documented
  structural-only boundary.
- Key the merge by a 64-bit semantic hash. That is acceptable for
  metrics but not for signoff-level merge decisions; the pass compares
  the full proof value.
- Treat instance outputs as opaque endpoints. `.2` instead admits only
  wrappers whose child semantics can be recursively proven and whose
  instance bindings can be substituted into the child proof.
- Merge leaves and wrappers in the same proof class. That would permit
  semantic dedup to flatten hierarchy as a side effect, and it can
  create a cycle if the lexicographic survivor is an ancestor. The pass
  keeps leaf and wrapper proof classes separate and skips any semantic
  merge group containing an ancestor/descendant pair.
- Admit flops, memories, or FSMs. Those need transition/state proof
  inputs beyond this pure-combinational whole-module truth-table class.

### Reset-defined memory identity blocker (2026-06-05, MEMORY-STATE-IDENTITY.1)

The current `Memory` motif is intentionally the reset-less synchronous
write/read template that Yosys infers as `$mem_v2`. That makes memory
state opaque: identical write/read cones do not prove identical stored
contents, so `MemRead` remains identity-by-instance.

Probe evidence for the obvious reset-defined alternative is not
signoff-clean. A 16x8 reset-all unpacked-array template passed
`verilator --lint-only /tmp/anvil-reset-mem-probe.sv`, but
`yosys -p "read_verilog -sv /tmp/anvil-reset-mem-probe.sv; synth -noabc; stat"`
warned that it was replacing the memory with a list of registers and
reported flip-flop/register logic rather than a preserved memory cell.
That is synthesizable, but it is not the warning-clean memory-inference
lane ANVIL currently documents and gates.

Rejected alternative: silently add a reset-all branch to the existing
memory motif and then merge memories with equal source cones. That would
change the artifact family from inferred memory toward register-file
logic and would violate the current downstream-warning contract. A
future reset-defined register-file motif can be introduced only as an
explicit, separately documented lane with its own knob, metrics, and
tool-matrix evidence.

### Exact reset-defined self-hold identity (2026-06-05, SEQUENTIAL-COINDUCTIVE-IDENTITY.2.2)

The first coinductive state merge is intentionally tiny: two flops may
share state when they have the same width, reset kind, reset value,
clock/reset domain, and each D input is exactly its own Q. Reset
establishes equality, and `D == Q` preserves that equality on every
subsequent clock. The proof deliberately requires an actual reset;
reset-less self-hold flops can start with different values and must stay
distinct.

Rejected alternatives: alpha-renaming arbitrary state variables inside
D cones, proving mutually-recursive update functions, or treating
semantically equivalent feedback such as a reduced `Not(Not(Q))` as a
coinductive class. Those need a bounded transition-relation proof and a
clear reset-domain model. `SEQUENTIAL-COINDUCTIVE-IDENTITY.2.2` keeps
the class exact and local to avoid turning a safe state-sharing pass
into an implicit sequential-equivalence engine.

### Domain-aware flop identity (2026-06-05, SEQUENTIAL-COINDUCTIVE-IDENTITY.2.1)

`merge_equivalent_flops` now treats `Module::flop_domain(flop.id)` as
part of the reset-defined state signature. This is a hard safety
precondition for any broader sequential identity work: two registers
with the same D proof and reset value are still distinct if they sample
different `(clk, rst_n)` domain pairs. The K=1 default remains domain
0 through `Module::flop_domain`, so existing single-clock fixtures keep
their previous merge behavior.

The implementation also remaps explicit `Module.flop_domains` entries
when state is merged or dead flops are compacted. Without that metadata
remap, a later pass or library caller could observe stale domain tags
after dense `FlopId` renumbering.

### Sequential identity proof envelope inventory (2026-06-05, SEQUENTIAL-COINDUCTIVE-IDENTITY.1)

Before broadening state sharing, the proof inputs have to be explicit.
For flops, the minimum safe signature is width, reset kind, reset value,
clock/reset domain, and a bounded D-cone proof over canonical endpoints.
The current generated multi-clock promotion pass runs after leaf
finalization, so promotion-added synchronizer flops are not re-merged by
the existing generated flow. Still, the IR already has
`Module::flop_domain`, and any helper that can run after domain tags
exist must include it in the signature before merging state.

The first broader class worth implementing is exact self-hold
coinduction: two same-domain, same-width, same-reset registers whose D
input is exactly their own Q are equal after reset and preserve equality
on every clock. That is intentionally much narrower than arbitrary
sequential equivalence. Mutually-recursive registers, update functions
that are equivalent only after renaming state variables, retimed state,
cross-domain state, and convergence-after-N-cycles candidates remain
blocked until ANVIL has a bounded transition-relation proof and the
necessary IR domain/reset facts.

### Bounded semantic proof budget audit (2026-06-05, COMBINATIONAL-SEMANTIC-IDENTITY.2)

The semantic proof limit moved from a flat 10 endpoint-support-bit cap
to a two-part budget: support may reach 12 bits, but only when
`assignment_count * cone_node_count` stays inside the previous
10-bit worst-case envelope. Merge proofs therefore admit tiny 12-bit
cones while still skipping larger 12-bit cones before truth-table
evaluation. Cleanup exact proofs use the same 12-bit support ceiling
but a stricter 64-node / 65536-work-unit budget and still require no
more than three canonical endpoints.

Rejected alternative: simply raising the support cap. That would double
or quadruple per-proof cost with no bound on how many finalization
candidates could become eligible. The combined work budget gives the
useful shallow-cone win without making the semantic pass a second
whole-graph evaluator.

Focused resource evidence: `/usr/bin/time -l cargo test -q
semantic_merge_proof` passed 3 tests in 0.32s with 45252608-byte max
RSS; `/usr/bin/time -l cargo test -q cleanup_exact_proof` passed 4
tests in 0.05s with 45678592-byte max RSS. The full suite was not
needed for this budget audit.

### Gate-to-endpoint semantic fold (2026-06-05, COMBINATIONAL-SEMANTIC-IDENTITY.1)

The bounded `EGraph` fragment now indexes earlier non-gate canonical
nodes as valid semantic targets. After enumerating a small-support cone,
the proof drops any endpoint whose bits do not affect the output. That
lets a cone such as `a & (b | !b)` reduce its proof from syntactic
endpoints `{a,b}` to the functional endpoint `{a}`, then rewire the gate
to the existing `a` node.

This is not a general unbounded e-graph. The current support, cone-node,
and combined work budgets still apply before enumeration, and the proof
still keys on canonical endpoints after minimization. The important
boundary is now: semantically-dead helper endpoints may disappear, but
live canonical roots remain part of identity.

### Endpoint-preserving semantic gate merge (2026-06-05, ENDPOINT-IDENTITY-BOUNDARY.1)

The bounded `EGraph` fragment now has a paired no-merge regression for
the endpoint part of the proof. The existing tests already prove
same-endpoint semantic equivalents can merge; the new proof constructs
`a & (b | !b)` and `c & (d | !d)`. These have the same local truth-table
shape. After `COMBINATIONAL-SEMANTIC-IDENTITY.1`, each cone may fold to
its own live endpoint (`a` and `c` respectively), but they still must
not collapse to the same canonical node.

This protects the doctrine that NodeId identity is not "looks like the
same Boolean shape somewhere". It is equality of functionality over the
same canonical endpoints. Any future proof-budget expansion must keep
that endpoint key.

### Hierarchy module dedup remains structural-only (2026-06-05, HIERARCHY-IDENTITY-BOUNDARY.1)

The hierarchy module-dedup pass now has an explicit boundary regression:
two one-bit modules that compute the same function (`input` versus
`Not(Not(input))`) but have different IR structure must not merge under
`dedup_modules`. Their canonical module signatures differ, the pass
removes zero modules, and the top-level instances keep their original
module names.

This protects the current proof contract. `hierarchy_module_dedup`
deduplicates canonical structural module templates; it is not a
whole-module semantic-equivalence engine. A future deeper hierarchy
identity task would need a module-level proof, not just a stronger
claim attached to the current signature hash.

### Memory identity remains instance-local (2026-06-05, MEMORY-IDENTITY-BOUNDARY.1)

The roadmap's memory-state identity gap is now protected by a focused
regression instead of relying on prose alone. A two-memory module with
identical write/read source cones is driven through the node-id /
e-graph state-sharing boundary (`merge_equivalent_flops`,
`merge_equivalent_fsms`, and compaction), and both independent
`Memory` blocks plus both `MemRead` leaves must remain.

This is the correct signoff boundary for the current inferrable-memory
template. Unlike generated FSMs, memories have no reset-defined array
contents, so equal address/write cones do not prove equal stored state.
A future memory-state merge would first need a stronger reset/init or
equivalence proof; until then, memory identity is by instance.

### Post-dedup unreachable-module pruning (2026-06-05, HIERARCHY-DEDUP-PRUNE.1)

`dedup_modules` now snapshots the module definitions reachable from
`Design::top` before structural module merging, then prunes only those
definitions that become unreachable after a real merge and instance
rewrite. This is a hierarchy-identity cleanup: if a parent module is
merged away and its private child is no longer reachable, that child
should not remain in the emitted design.

The guard is deliberately "after at least one merge", not "whenever the
function is called", and the prune set is deliberately "reachable before,
unreachable after", not "all top-unreachable definitions". That preserves
the existing under-instantiated library surface when
`hierarchy_module_dedup` finds no duplicate canonical signatures and
also keeps modules that were intentionally unreferenced before dedup out
of the reachability-prune set. Such modules may still merge if they have
duplicate structural signatures. Dedup-off behavior is unchanged, and no
broader module equivalence is introduced.

### Deterministic FSM identity merge (2026-06-05, SEQUENTIAL-IDENTITY.1)

The roadmap's full-factorization doctrine now has a finite sequential
extension beyond flop D-cones: `merge_equivalent_fsms` deduplicates
generated FSM blocks under `identity_mode = node-id` when their selector
proof, selector width, encoding, state count, transition table,
Moore-output table, and output width match.

The proof boundary is deliberate. FSMs reset to state 0 and have
explicit transition/output tables, so duplicate blocks are one proven
state machine. Memories remain opaque because the current inferrable
memory template does not reset array contents; identical write/read
cones alone are not enough to prove identical stored state.

### Knowledge Map enforcement (2026-06-05, KNOWLEDGE-MAP-DOC.2)

`KNOWLEDGE_MAP.md` is a derived retrieval index, not an authored live
doc. Its source of truth is YAML front-matter in fact-bearing markdown
files under the configured scan directories. The local pre-commit hook
therefore regenerates and stages the map before checking it, while CI
runs the same check as the server-side backstop.

Facts are added lazily when a durable conclusion is established or
archaeology is caught. Existing task trees, live docs, and mdBook prose
are not converted into cards as a migration project.

### Live-doc path portability (2026-06-04, LIVE-DOC-PATH-HYGIENE.1)

Live docs and the mdBook must describe project files with paths relative to the
repository root. Local checkout prefixes are not portable and must not be
recorded in user-facing docs, contributor live docs, or task-tree files.

External validation evidence is different: banked artifacts under `/tmp` keep
their absolute paths because those paths identify evidence outside the repo, not
project files inside the checkout.

### Multi-clock + CDC primitives design (2026-05-24, MULTI-CLOCK-CDC.1)

Research-only slice (no code; `.2`+ implement). `MULTI-CLOCK-CDC.1`
opens the only remaining named follow-up tree on the repo after
`DIFFERENTIAL-SIMULATION` closed `2026-05-24`. Per the proven
Phase-7/8/9 + `DIFFERENTIAL-SIMULATION.2a`/`.3a` design-first
discipline: the IR extension (per-flop clock + per-flop reset),
the CDC primitive catalogue, the by-construction rule, and the
downstream-tool gate are all load-bearing structural decisions to
settle before code. Multi-clock CDC touches the most load-bearing
ANVIL invariant (`book/src/sequential.md` "Synchronous-design
discipline": "Every module is fully synchronous to a single clock
domain"), so the design-first slice is mandatory.

**Goal.** Generate modules with N≥2 declared clock domains whose
inter-domain signals are wrapped by-construction in a CDC
primitive (2-flop synchronizer at minimum); every emitted
multi-clock module passes the chosen downstream-tool CDC check
(Verilator `--cdc=metastable` is the first-cut candidate) and
shows cross-simulator agreement under
`tool_matrix --diff-sim` on a synchronised stimulus.

**CDC primitive catalogue — first-cut scope.** The IEEE CDC
literature names ~7 patterns; ANVIL adopts them in priority order:

| Tier | Primitive | First cut? | Notes |
| --- | --- | --- | --- |
| 1 | **2-flop synchronizer** (1-bit) | **Yes** | The minimum-viable CDC building block. Every 1-bit signal crossing domain A → domain B is two flops registered in B's domain; the metastability is captured + resolved by the second flop. Covers ~80% of real CDC paths. |
| 2 | N-flop synchronizer (1-bit) | **Yes (2026-06-05, `SIGNOFF-SURFACE-EXPANSION.1`)** | Same as tier-1 with N≥3 flops; needed for very-high-speed paths where 2 flops is insufficient. Implemented as `cdc_synchronizer_stages`, default 2 for byte-identical compatibility. |
| 3 | Async FIFO (multi-bit) | Deferred (own tree) | Major structural change: depth, gray-code pointers, empty/full handshake, separate read/write domains. Phase-sized. |
| 4 | Gray-code pointer transfer | Deferred (own tree) | Foundation for async FIFO; gray code's single-bit transition prevents pointer corruption mid-flight. |
| 5 | Req/ack handshake (multi-bit) | Deferred (`.6` or follow-up) | 4-phase or 2-phase handshake for word transfer; smaller than FIFO but still structural. |
| 6 | Pulse synchronizer | Deferred (`.7` or follow-up) | Toggle + 2-flop sync + XOR; transfers an event across domains. |
| 7 | Reset synchronizer | Deferred (`.4` or follow-up) | Async-assert + sync-deassert; each domain gets its own. |

**Tier 1 (2-flop synchronizer)** is the minimum viable cut. Tier 2
reuses tier 1 mechanically by adding stage count. The remaining
deferred tiers are large enough to warrant their own task tree (FIFO,
handshake, gray code). Per `feedback_full_factorization.md`
and `feedback_rules_first_generation.md`: when the generator
makes a domain-crossing decision, the synchronizer wrap is
issued by-construction — there is never a "generate the path
then check for synchronizer" filter pass.

**Minimum-viable IR extension.** The single-clock invariant lives
in `Module.clock: Option<Port>` and `Module.reset: Option<Port>`
(single reserved slots) plus the `always_ff @(posedge clk or
negedge rst_n)` template in `src/emit/sv.rs`. Two surface IR
changes:

- **Multi-domain Module shape.** `Module.clock_domains:
  Vec<ClockDomain>` where each `ClockDomain` carries
  `{ clk_port, rst_n_port, name }`. The existing single-domain
  Module continues to exist as the K=1 special case
  (`clock_domains.len() == 1` with `name = "default"`); this
  keeps the by-construction default behavior byte-identical
  unless a multi-clock knob fires. The existing `Module.clock`
  / `Module.reset` accessors stay (delegate to
  `clock_domains[0]`) so callers that don't care about
  multi-clock see no change.
- **Per-flop domain tag.** `Flop.domain: usize` (index into
  `Module.clock_domains`) — every flop knows which domain it
  belongs to. The emitter groups flops by domain and produces
  one `always_ff` block per (domain, polarity) tuple. The
  Phase-1 doctrine "one `always_ff` per module" is preserved
  for K=1; for K=N it generalises to "one `always_ff` per
  domain", which is the standard SV idiom.

The IR extension is backward-compatible. Existing modules with
no multi-clock knob fire stay K=1 with `domain = 0` for every
flop, and the emit is byte-identical.

**By-construction rule** (`book/src/structural-rules.md`, new
Rule for multi-clock). When the generator emits a flop in
domain B whose D-cone references a flop output in domain A,
the cone is rewritten to dereference a synchronizer chain in
domain B instead — that is, the flop sees `Synchronizer{
src_flop_q, dst_domain, stages }` as its operand, never the bare
cross-domain flop output. The default chain is the original
2-flop synchronizer; `SIGNOFF-SURFACE-EXPANSION.1` adds the
`cdc_synchronizer_stages` count for N-flop chains. All stages
are newly-minted flops in dst_domain. The rule fires at
*construction time*; there is no post-pass filter. The
bookkeeping that discovers domain-crossing operands is
`Flop.domain` + the cone-recursion that ANVIL already does.

This is exactly the rules-first generation pattern
(`feedback_rules_first_generation.md`): we never generate an
unsynchronised cross-domain path then filter it out; the rule
**constructs** the synchronizer in place.

**Downstream-tool gate.** Two candidates, evaluated:

- (a) **`verilator --cdc=metastable`** — a Verilator linter
  flag that flags cross-clock-domain paths without registered
  synchronizers. Pros: already integrated with the
  `tool_matrix` Verilator column; one flag toggle. Cons:
  experimental Verilator feature, may have false positives /
  miss real bugs. First-cut choice.
- (b) **`yosys read_verilog -cdc`** — explored: Yosys doesn't
  have a built-in CDC check in stable 0.64; the `-cdc` flag
  is project-folklore that doesn't exist. Rejected.
- (c) **Custom oracle.** ANVIL is a generator; we can record
  every constructed synchronizer in a manifest and emit a
  matching `cdc_manifest.json`, then assert the manifest
  matches what Verilator's linter reports. Defers to `.4`
  once `.3` lands. This mirrors the Phase-7 parity oracle
  pattern.

**Cross-simulator agreement (`tool_matrix --diff-sim`).** The
just-landed `.3b.2` `--diff-sim` column trivially extends to
multi-clock: the testbench drives multiple clocks (independent
periods) and stimulates inputs in each source domain;
outputs are sampled in their declared domain. For the *first
real-tool gate* on multi-clock, we sample only domain-B
outputs at domain-B sample points (a "synchronised stimulus"
flow) — this avoids the metastability-glass-jaw problem
where a transition mid-sync-flop produces different
trace-line values in iverilog (4-state) vs verilator
(2-state). Sequential domain-A→B paths with proper
synchronizers will produce byte-equal traces in both sims by
the cycle-accurate `@(negedge clk_B)` sample.

**Rejected alternatives.**

- (A) **Single-flop synchronizer.** Rejected — even 1-bit
  cross-domain paths need ≥2 flops to resolve metastability
  per standard CDC literature. A single flop is not a
  synchronizer.
- (B) **Clock-gating-instead-of-multi-clock.** Rejected — ICG
  is a power-optimisation concern, orthogonal to CDC. ANVIL's
  stance is "emit always-on flops; let downstream insert ICG".
- (C) **Latches for level-sensitive crossing.** Rejected —
  ANVIL's synchronous-design discipline forbids latches
  (`book/src/sequential.md`).
- (D) **Async-FIFO as the minimum viable cut.** Rejected —
  too large for the first multi-clock slice. FIFO requires
  gray-code pointer + handshake + depth; pushes outside the
  by-construction `.2`/`.3` envelope. Lands in its own
  follow-up tree.
- (E) **Generate-then-filter** (synchronizer-or-bust
  post-pass). Rejected — violates
  `feedback_rules_first_generation.md`. The synchronizer
  must be constructed in place.
- (F) **Dynamic frequency / dynamic clock ratios.** Rejected
  — the IR records a fixed declared frequency per port (or
  just a domain-name tag); runtime-dynamic frequency is a
  testbench concern, not a generator concern.

**Leaf shape.** `.2` implements the IR extension (multi-domain
`Module`, per-flop `domain`, synchronizer construction rule,
emitter); `.3` adds the downstream-tool gate (Verilator
`--cdc=metastable`) and the matrix wiring (`--multi-clock-prob`
knob, `saw_multi_clock_design` + `saw_cdc_2_flop_synchronizer`
coverage facts; `SIGNOFF-SURFACE-EXPANSION.1` adds
`saw_cdc_nflop_synchronizer`); `.4` documents the contract (README +
USER_GUIDE + `book/src/sequential.md` updates removing the
"Multi-clock deferred" caveat).

**Knob shape.** Single `--multi-clock-prob: f64` per-module
roll (defaults to `0.0` for byte-identical backward
compatibility). When fired, the generator picks `N` from
`--num-clock-domains-min`/`--num-clock-domains-max` range
(defaults `2..=2` — start simple). Per-module roll because
hierarchy is orthogonal: a multi-clock parent may have
single-clock children or vice versa; the generator handles
this generically via `Flop.domain`.

This entry is design-only and is itself task-tree owned
(`MULTI-CLOCK-CDC.1`); it makes no code change, consistent
with the task-tree-ownership doctrine's code/not-code
boundary.

### Tool-matrix `--diff-sim` wiring + representative-subset selector + coverage fact design (2026-05-24, DIFFERENTIAL-SIMULATION.3a)

Design-only slice (no code; `.3b` implements). `.3` split mirrors
the proven Phase 7/8/9 design-first discipline + the
`PHASE-7-ORACLE-MICRODESIGN.2c.2a`/`.2c.2b` precedent: the
module-extraction decision, the CLI shape, the subset selector,
and the coverage-fact wiring are load-bearing choices to settle
before code; the design itself is docs-only.

**Goal.** Wire the `tests/diff_sim.rs::emit_testbench` +
`run_iverilog` + `run_verilator` + `normalize_trace` machinery
landed in `.2b.2` into `src/bin/tool_matrix.rs` as an opt-in
`--diff-sim` mode, so the matrix records cross-simulator semantic
agreement per scenario alongside its existing parse/synth/lint
columns. A new `saw_design_with_cross_simulator_agreement`
coverage fact fires when at least one DUT in the run achieves
byte-equal post-reset traces.

**Module-extraction decision (the structural choice that justified
splitting `.3`).** The harness helpers currently live in
`tests/diff_sim.rs` and are NOT exported from the `anvil` library
crate — `src/bin/tool_matrix.rs` cannot reach them today. Two
options:

- (A) **Extract to `src/diff_sim/mod.rs`** (library module).
  `tests/diff_sim.rs` switches to `use anvil::diff_sim::{…}`;
  `src/bin/tool_matrix.rs` does likewise. Full-factorization
  doctrine satisfied (one home for the testbench emitter +
  orchestration). Cost: one module move + two `use` updates.

- (B) **Duplicate the helpers in `tool_matrix.rs`** (or copy
  paste). Violates the full-factorization doctrine
  (`feedback_full_factorization.md`) — two homes for the
  testbench-emitter code, divergence inevitable. Rejected.

`.3b` takes (A). The new `src/diff_sim/mod.rs` exports
`baked_input_vectors`, `mask_to_width`, `fmt_sv_hex`,
`is_sequential`, `emit_testbench`, `run_iverilog`, `run_verilator`,
`normalize_trace`, `tools_present`, plus a thin façade
`run_differential(top: &Module, vectors: &[Vec<u128>], work_dir:
&Path) -> Result<DiffOutcome, DiffError>` that orchestrates the
whole flow (emit testbench → emit DUT SV → invoke iverilog →
invoke verilator → normalize + compare → return aligned
traces/diff). The façade is what `tool_matrix.rs` calls per
scenario.

**CLI flag shape.** New `--diff-sim` opt-in flag on `Cli` (mirrors
the existing `--skip-verilator`/`--skip-yosys` opt-out flags and
the `--phase4-hierarchy-gate` opt-in elevation flag). Default:
`false`. When set: every scenario in the selected scenario set
runs the differential harness AFTER the existing parse/synth/lint
columns succeed (gated on Verilator AND Yosys both clean — no
point asking simulators to agree on output that one tool already
rejected). The flag is orthogonal to `--phase4-hierarchy-gate` /
the other gate-elevation flags; it adds a new column, it does not
change which scenarios run.

**Representative-subset selector.** The full 204-scenario matrix
is computationally infeasible for the differential harness
(per-design wall-clock cost: ~5-10 s for iverilog +
~10-20 s for verilator compile+run = ~20 s/scenario × 204 ≈
68 min just for diff-sim). Three options for subset selection:

- (1) **`--diff-sim-subset <integer>`** — randomly sample N
  scenarios (seeded). Simple; reproducible; representative of
  the distribution. Default `N=5`. Rejected: random sampling
  loses the curated coverage structure (e.g., always picking 5
  combinational misses sequential coverage).

- (2) **Hand-curated subset** — a fixed list of scenario names
  (e.g., `["minimal-comb", "minimal-seq", "phase4-hier-comb",
  "phase4-hier-seq", "phase6-fsm-leaf"]`). Coverage-aware
  (one per major axis). Rejected: brittle — every new
  scenario-set requires updating the list; doesn't scale with
  `Phase4Hierarchy`/`Phase3Structured` etc.

- (3) **Per-axis sampling** — for the selected scenario set,
  pick the first scenario that satisfies each major coverage
  axis (combinational, sequential-flop, hierarchy, memory,
  fsm), capped at K=5. Coverage-aware AND self-maintaining.
  **Chosen for `.3b`.** Selection is deterministic (first match
  per axis in scenario-set declaration order), reproducible,
  and naturally adapts as new scenarios land.

The selected subset is recorded in the matrix report under
`diff_sim_subset: Vec<String>` (scenario names) so the report
itself is self-describing.

**Coverage-fact wiring.** New `saw_design_with_cross_simulator_
agreement: bool` field on `CoverageSummary` (alongside the
existing `saw_inferrable_memory_design`/`saw_fsm_design` from
Phase 6). Fires when at least one DUT in the subset achieves
byte-equal post-reset traces. Merged into the aggregate `dst |=
src` per the existing pattern at `tool_matrix.rs:5847`.
`--diff-sim` is NOT a gate-elevation flag by default (the matrix
will not exit non-zero if the fact is false unless
`--fail-on-coverage-gap` is set AND `--diff-sim` is set — the
existing opt-in coverage-gap semantics, no new flag needed).

**Per-scenario report shape.** New optional field on
`ModuleReport`: `diff_sim: Option<DiffSimReport>`. `DiffSimReport`
records: `ran: bool` (was this scenario in the subset?),
`success: bool` (byte-equal post-reset traces?), `n_samples:
u32` (sample count), `iverilog: Option<ToolInvocation>`,
`verilator: Option<ToolInvocation>`, `mismatch_excerpt:
Option<String>` (first 10 lines of the diff, retained per the
Phase-7 counterexample doctrine — never a silent pass).
`tools_present()` guard makes the column a friendly no-op when
either simulator is absent (the column reports `ran: false`
with a clear reason; matrix exits clean).

**Wiring point in `tool_matrix.rs`.** Inserted as a new
per-module step in the existing per-module pipeline, AFTER
Verilator + Yosys (the existing tools) and BEFORE checkpoint
write (so a `--resume` re-run replays the diff-sim column from
checkpoint without re-invoking the simulators). Gated by
`cli.diff_sim` AND scenario presence in the subset AND
Verilator+Yosys both clean — the existing "downstream tools
already accepted the SV" precondition.

**Rejected alternatives.**

- (i) **`--diff-sim` as a gate-elevation flag** (always
  required to pass) — rejected: the simulator runtime is too
  large to gate-mandatorily in CI; the existing `--phase4-
  hierarchy-gate` already takes ~75 min, and a mandatory
  diff-sim column on top would push the gate over 2 h. Opt-in
  with explicit `--fail-on-coverage-gap` is the right
  trade-off.

- (ii) **Duplicate the helpers in `src/bin/tool_matrix.rs`** —
  rejected per the module-extraction discussion above
  (full-factorization doctrine).

- (iii) **Random subset sampler** — rejected per the
  per-axis-sampling discussion above (loses curated coverage
  structure).

- (iv) **Hand-curated subset** — rejected per the
  per-axis-sampling discussion above (brittle, doesn't scale).

- (v) **Move `tests/diff_sim.rs` entirely** (delete the file,
  put the gated tests inside `src/diff_sim/mod.rs` as
  `#[cfg(test)]`) — rejected: separation of library API surface
  from the gated integration tests is the established convention
  (cf. `tests/microdesign_parity.rs` consumes
  `src::microdesign::*`, `tests/frontend_parity.rs` consumes
  `src::frontend::*`). Library exports the API; the integration
  test owns the `#[ignore]` gates.

**Proof shape (`.3b`).** `cargo fmt`/clippy(-D warnings)/check/
test all clean. New `src/diff_sim/mod.rs` carries the extracted
helpers + the `run_differential` façade. `tests/diff_sim.rs`
updated to `use anvil::diff_sim::{…}` (no logic change). New
`src/bin/tool_matrix.rs` `--diff-sim` flag + per-module wiring
+ subset selector + `saw_design_with_cross_simulator_agreement`
coverage fact + `DiffSimReport` per-module field + merge into
aggregate. Cargo-portable proofs: subset selector picks one per
axis deterministically; coverage fact merges correctly; CLI
parse smoke. Tool-gated `#[ignore]` proof: end-to-end
`tool_matrix --diff-sim --base-seed 0 --modules-per-scenario 1
--out /tmp/anvil-diff-sim-p1` exits 0 with
`saw_design_with_cross_simulator_agreement=true` on a machine
with both simulators installed. `.4` documents the contract.

This entry is design-only and is itself task-tree owned
(`DIFFERENTIAL-SIMULATION.3a`); it makes no code change,
consistent with the task-tree-ownership doctrine's code/not-code
boundary.

### Book-examples-runnable design (2026-05-18, BOOK-EXAMPLES-RUNNABLE.1)

Design-only slice. No code. The repo is now public with the mdBook
live at `https://rdje.github.io/anvil/`; every example is a
copy-paste contract with users. This entry inventories the
fenced-block reality and designs the convention migration + the
CI-gated drift-proof harness, so `.2` has an unambiguous target.

**Fenced-block inventory (audited `book/src/*.md`).** 62 ```bash`` ,
8 ```rust`` , 9 ```systemverilog`` , 4 ```text`` .

- **`bash` (62) — the runnable copy-paste surface.** `recipes.md` 41,
  `tutorial.md` 10, `getting-started.md` 6, `knobs.md` 2,
  `factorization.md`/`faq.md`/`introduction.md` 1 each. Leading
  tokens: ~44 lines start with bare `anvil`, ~24 with `cargo`, plus
  `\`-continued multi-line commands and `| …` pipes. ~58 bare-`anvil`
  occurrences total. **Defect:** bare `anvil …` is not runnable from
  a fresh clone (no binary on PATH) — `getting-started.md` already
  uses `cargo run --release --`, the rest don't. This is the core
  break the owner flagged.
- **`rust` (8) — illustrative IR/struct sketches**, not programs:
  `ir.md` 3, `hierarchy.md` 3, `architecture.md` 1, `knobs.md` 1.
  Partial (reference internal types, no imports/`fn main`) → would
  fail `mdbook test` if treated as doctests.
- **`systemverilog` (9) + `text` (4) — emitted-output samples**, not
  commands; never executed, but some directly follow a command as
  its shown output.

**Owner decisions (2026-05-18), recorded in the tree:** (1) runnable
blocks standardize on **`cargo run --release --`** (+ one optional
`cargo install --path .` → `anvil` shorthand note); (2) correctness
is **CI-gated** via an extraction harness + `mdbook test`, not a
one-time audit.

**Architecture — chosen: a `cargo test` integration harness +
`mdbook test`, both in CI.**

1. **Convention migration.** In every runnable `bash` block, the
   command head `anvil ` → `cargo run --release -- ` (preserving
   `\`-continuations, `| …` pipes, and redirections). One shorthand
   note (getting-started + knobs reference): "`cargo install --path
   .` once, then use `anvil` instead of `cargo run --release --`".
2. **Skip marker for genuinely illustrative bash.** Default = run.
   A block opted out with an HTML-comment sentinel on the line
   immediately before the fence: `<!-- book-test: skip — <reason>
   -->`. HTML comments don't render in mdBook output and aren't in
   the copy-paste body, so users never see noise; the harness keys
   off it. Reason string is mandatory (no silent skips).
3. **Harness = `tests/book_examples.rs` (cargo integration test).**
   Not a CI-only shell script (that would drift from the `cargo
   test` gate — the project convention is *everything is
   `cargo test`-gated*; CI already runs `cargo test`). It: walks
   `book/src/*.md`; parses ```bash`` fences; honours the skip
   sentinel; builds the binary once (`cargo build --release`, then
   invokes `target/release/anvil` so per-example cost excludes
   rebuild); runs each block in a fresh temp CWD with a per-command
   timeout and `CARGO_NET_OFFLINE=true` (anvil examples are fully
   local — no network); asserts exit 0. Where a ```text`` /
   ```systemverilog`` / ```console`` block immediately follows a
   command block and is tagged asserted, compare: seed-stable
   commands (anvil is reproducible by `--seed`) → exact match;
   tool-version-sensitive → shape/prefix match. Untagged output
   blocks are documentation only (not asserted) — recorded so a
   future contributor doesn't assume all output is checked.
4. **`mdbook test` + rust sketches.** Annotate the 8 ```rust`` blocks
   `rust,ignore` (still rendered in the book; not compiled), so
   `mdbook test book` is green and *meaningful*: any future real
   ```rust`` example is compiled, sketches are explicitly exempt.
5. **CI.** `tests/book_examples.rs` runs under the existing `cargo
   test` step in `.github/workflows/ci.yml`; add an `mdbook test
   book` step. Both gate `main`.

**Rejected alternatives.** (A) Rust `doctest`/`mdbook test` only —
covers just the 8 ```rust`` sketches, **not** the 62 ```bash``
blocks that are the actual copy-paste surface; leaves the real
defect unenforced. (B) A standalone CI-only `.sh` extractor — works
on GitHub but is invisible to local `cargo test`, so it drifts from
the COMMIT.md gate and a contributor can't reproduce a failure
locally with one command; violates the "everything is
`cargo test`-gated" project convention. (C) Generate the book
examples *from* tests (golden-doc) — strongest anti-drift but a
large restructuring of authored prose, fights the book-doctrine's
hand-written friendly voice, and is disproportionate; the
extraction harness gets ~all the safety at a fraction of the churn.

**Proof shape (`.2`, expected to split).** Harness enumerates ≥ the
runnable-block count, builds once, runs each against the fresh
binary, all exit 0; tagged sample outputs match; `mdbook test book`
green; CI gates on `cargo test` (incl. `book_examples`) + `mdbook
test`; a deliberate broken example fails the harness (negative
control); book meaning unchanged (only invocation normalised + the
one shorthand note). Split candidates: harness impl / the ~62-block
migration / CI wiring (independently reviewable).

This entry is design-only and is itself task-tree owned
(`BOOK-EXAMPLES-RUNNABLE.1`); it makes no code change, consistent
with the task-tree-ownership doctrine's code/not-code boundary.

**As-built resolution (2026-05-18, `.2.2` — tree CLOSED).** The
harness landed as `tests/book_examples.rs` essentially per design.
One implementation deviation worth recording: blocks are run with the
**shell-script model** (`bash -eu -o pipefail`, one child per block,
`cargo run --release --` text-substituted to `"$ANVIL"` = the
once-built release binary) rather than a parsed-command model — this
is what makes `$()`/for-loop/`# comment` blocks runnable verbatim,
and a classification guard *panics* on any unclassified residual so a
silent gap is impossible.

**Non-obvious gotcha (cost a full debugging cycle — record
permanently).** The first full runs reported 12 blocks "TIMED OUT
after 600 s". This was **not** a book defect: a default `--seed 42`
module is ≈86 KB on stdout (a 5-level `factorization` sweep ≈525 KB),
but the OS pipe buffer is ≈64 KB. The original `run_script` used
`Stdio::piped()` and a `try_wait()` poll loop that **never drained
the pipe until after the child exited** — so `anvil` blocked forever
in `write()`, the child never exited, and the loop spun to the
timeout (12 × 600 s ≈ the observed 7273 s total). Directly invoking
each "timed-out" command proved the examples are correct (0.03–0.15 s
each). **Rule for any future child-process harness here: never pair
`Stdio::piped()` with a non-draining wait loop for a child whose
output can exceed ~64 KB.** Fix chosen: redirect child stdout/stderr
to temp **files** (no buffer limit, std-only, no reader-thread
plumbing) and reap the child after a timeout kill. Post-fix:
`cargo test --test book_examples` = 3/3, 54 runnable blocks exit-0,
9 skip-sentineled, 76.4 s (down from 7273 s). The `PER_BLOCK_TIMEOUT`
is now purely a defensive backstop, not a hot path. CI gates this via
`cargo test` + the added `.github/workflows/ci.yml` `mdbook test
book` step.

### Phase 6 inferrable-memory motif design (2026-05-18, PHASE-6-ADVANCED-MOTIFS.1)

Design-only slice. No code. Lifts `book/src/ir.md` "Synthesizable
aggregates → Unpacked arrays (the memory-inference pattern)" /
ROADMAP Phase 6 into a concrete, codebase-grounded plan with an
empirical Yosys-inference probe, a chosen architecture, rejected
alternatives, and a proof shape, so the implementation leaf
(`PHASE-6-ADVANCED-MOTIFS.2`) has an unambiguous target.

**Goal (from ROADMAP Phase 6 / book).** Emit inferrable memory
motifs (single-port and simple dual-port; inferrable patterns only),
valid by construction, downstream-clean (Verilator + both Yosys
modes), and **recognised as memory by Yosys** (`$mem_v2` after the
memory pass). Default-off / byte-identical; never retire existing
behaviour; single-clock discipline preserved.

**Empirical Yosys-inference probe (resolves the tree's Open
Question).** Two LRM-synthesizable templates were probed in `/tmp`
through Verilator and **both** repo Yosys modes:

- **Single-port** — `logic [DW-1:0] mem [0:2**AW-1]`;
  `always_ff @(posedge clk) begin if (we) mem[addr] <= wdata;
  rdata <= mem[addr]; end`.
- **Simple dual-port** — same array, one write port (`waddr`/`we`)
  and one independent read port (`raddr`), synchronous read.

Result: `read_verilog -sv; proc; opt; memory_collect; stat` yields
**exactly `1 $mem_v2`** for *both* templates; `verilator
--lint-only` exits 0; `synth -noabc; check -assert` and `synth;
abc -fast; check -assert` both exit 0 with no `ERROR`. Conclusion:
both shapes are reliably memory-inferred and downstream-clean across
the repo's exact gate toolchain. (Plain `synth` then maps tiny mems
down to FFs for the final netlist — expected with no BRAM target;
the *inference* is what Phase 6 asserts, captured at the
`memory_collect` stage, not the post-`memory_map` cell mix.)

**Code reality that constrains the design** (audited; key anchors).
The IR has **no array/memory concept**: `Port` (`src/ir/types.rs`),
every `Node::*` and `Flop` carry a scalar `u32` width; the only
stateful element is `Flop` (one shared `always_ff` per module,
`src/emit/sv.rs`); `Node` leaves are `PrimaryInput`, `Constant`,
`FlopQ`, `InstanceOutput`. Per the **operators-vs-blocks doctrine**
(`DEVELOPMENT_NOTES.md` "Core design decisions"), a memory is a
*block* (a functional unit with internal structure and its own
state/ports), not an operator and not a datatype — exactly like
`Flop`/`Mux`. The emitter is a dumb serialiser; blocks are
first-class motifs it renders verbatim.

**Architectural decision — chosen: (M) a first-class `Memory` block,
sibling to `Flop`, kept out of the NodeId expression graph.** A
memory is *state with identity by instance*, not an expression
(re-evaluating `mem[a]` is not pure), so — exactly as `Flop` is a
module-level element with a `FlopQ` leaf, not a `Gate` — Phase 6
adds:

1. A `Memory` element on `Module` (additive `Vec<Memory>`, `Default`
   empty ⇒ zero churn to `..Module::default()` sites, the proven
   Phase 5/5b additive pattern): `{ id, addr_width, data_width,
   kind: SinglePort | SimpleDualPort, write port (we/addr/data
   source `NodeId`s), read port(s) }`.
2. A new gate-graph **leaf** `Node::MemRead { mem, ... }` (sibling to
   `FlopQ`) so a memory read result can feed cones without the array
   itself ever entering combinational factorization (a `MemRead`
   leaf is opaque to CSE, like `FlopQ` — it is identity-by-instance,
   never merged with anything).
3. Emitter: render the **empirically-validated inferrable template**
   verbatim (`logic [DW-1:0] mem_k [0:2**AW-1]` + the synchronous
   write/read `always_ff`), wired to the existing `clk`. Validator:
   address/data widths consistent; read leaves resolve to a declared
   memory.
4. Opt-in `Config::memory_prob` (`f64`, serde-default `0.0`,
   probability-range validated — the Phase 5/5b knob pattern).
   Default-off ⇒ no `Memory`, byte-identical for fixed seeds.

This keeps **valid-by-construction** (a rules-first generator block,
no post-hoc filter), preserves single-clock discipline (the memory
shares the module `clk`), and keeps the full-factorization doctrine
intact (the array is never a NodeId; `MemRead` is an opaque leaf).
It mirrors how `Flop` was integrated, which is the lowest-risk
precedent in the codebase.

**Rejected alternatives.**

- **(A) Model memory as a register file of `Flop`s + address mux.**
  Rejected: Yosys does **not** infer a flop-array + mux as `$mem`
  (the probe's whole point) — it would defeat Phase 6's purpose
  (memory-inference stress) entirely, and explodes node/flop counts
  (2^AW flops + a 2^AW-way mux per read). It is not "a memory" to
  any downstream tool.
- **(B) Emitter-only string template with no IR representation.**
  Rejected: not valid-by-construction — the memory's write/read
  data sources must be real generated cones with dependency
  tracking and validation; a free-floating text template cannot be
  driven, validated, or factored, and is the post-hoc-template
  anti-pattern the project forbids (rules-first construction).
- **(C) A generic unpacked-array *datatype* threaded through
  `Port`/`Node` width arithmetic.** Rejected: memory is a *block*,
  not a datatype (operators-vs-blocks doctrine); threading an
  array type through every width check / CSE key / validator /
  emitter is a massive invasive change for zero gain over (M),
  and conflates two orthogonal concepts. (M) confines memory to a
  new block + one opaque leaf, exactly as `Flop` is confined.

**Proof shape (for `.2`).** (1) Default-off byte-identical for fixed
seeds across all `ConstructionStrategy` values (no `Memory` ⇒
identical `to_sv`). (2) Forced-on: a focused proof that a generated
memory module emits the inferrable template, `validate_design`
passes, and **Yosys `memory_collect` reports ≥1 `$mem_v2`** in both
repo modes (the inference assertion — the Phase 6 contract), plus
`verilator --lint-only` clean. (3) A `tool_matrix`
`phase6_inferrable_memory` scenario shaped like the dedup/phase5/5b
anchor (shape-coverage sets unperturbed) + `DesignMetrics
.num_memory_modules` + a `saw_inferrable_memory_design` coverage
fact/gap + non-vacuity test; **no ROADMAP promotion** until the real
repo-owned gate is run and verified clean (r87 no-aspirational-claims
— same `.2.x` decomposition as Phase 5/5b). (4) Full `cargo` hygiene
gate; `mdbook` reconciled (`ir.md` memory delivered note +
`knobs.md` `memory_prob`).

This entry is design-only and is itself task-tree owned
(`PHASE-6-ADVANCED-MOTIFS.1`); it makes no code change, consistent
with the task-tree-ownership doctrine's code/not-code boundary.

### Phase 6 generated-encoding FSM motif design (2026-05-18, PHASE-6-ADVANCED-MOTIFS.3.1)

Design-only slice. No code. Lifts ROADMAP Phase 6 "FSMs with
explicitly generated state encodings" into a concrete,
codebase-grounded plan with an empirical downstream-tool probe, a
chosen architecture, rejected alternatives, the `.3` proof shape and
split, so `.3.2`+ have an unambiguous target. Mirrors the proven
Phase 5 / 5b / `.1`-memory design-first method.

**Codebase grounding.** The IR has no state-machine concept. `Flop`
is the only stateful element; `Node` is a scalar `u32`-typed
expression graph; the operators-vs-blocks doctrine (established by
the `.1`/`.2.1` memory work) says a stateful, non-CSE-able motif is a
**block**, not an operator and not a datatype. A "generated-encoding
FSM" decomposes exactly into primitives the emitter already proves
synthesizable: (1) a **state register** — a flop holding the
encoded-state bits; (2) **combinational next-state decode** — a
`case (state_q)` over the generated state constants; (3)
**combinational output decode** — a Moore `case (state_q)`; (4)
**generated state constants** — `localparam` values whose width and
bit-pattern are fixed *by the chosen encoding*. The encoding choice
(binary / one-hot / gray) is the entire novel surface; everything
else is flop + comb logic ANVIL already emits valid-by-construction.

**Empirical downstream probe (resolves the open design question:
"is a generated-encoding FSM downstream-clean in both repo Yosys
modes + Verilator, for every encoding?").** Hand-wrote the exact SV
template ANVIL would emit (localparam state constants + `state_q`
flop with async-low reset + `always_comb` next-state `case` +
`always_comb` Moore output `case`) for a 4-state FSM in all three
encodings:

- **binary** (`state_q` width `ceil(log2 N)` = 2 bits; constants
  `2'd0..2'd3`),
- **one-hot** (`state_q` width `N` = 4 bits; constants `4'b0001`,
  `4'b0010`, `4'b0100`, `4'b1000`),
- **gray** (`state_q` width 2 bits; constants `00,01,11,10`).

Result — **all three are downstream-clean**: `verilator --lint-only
-Wall` exit 0; `yosys read_verilog -sv; synth -noabc; check -assert`
clean; `yosys synth; abc -fast; check -assert` clean — i.e. clean in
**both** repo-owned Yosys modes and Verilator. The state-register
width and constants differ by encoding (`[1:0]` for binary/gray vs
`[3:0]` for one-hot), so **"encoding selectable" is a structural
fact, not cosmetic** — exactly the ROADMAP Phase 6 requirement. (The
case-decode shape Yosys also recognises via its `fsm` pass, but that
is a bonus; the inference contract here is plain clean synthesis,
unlike memory whose contract was the `$mem_v2` template — an FSM is
"just" flop + comb logic, so the risk is encoding correctness, not
inferability.)

**Chosen architecture — (F): first-class `Fsm` block + opaque
`Node::FsmOut` leaf + generated-encoding emitter + opt-in knob.**
Mirrors the landed memory motif ((M)) so it reuses the proven
opaque-stateful-leaf pipeline integration:

1. **IR.** Additive `Vec<Fsm>` on `Module` (Default-empty → trees
   without FSMs are byte-identical). An `Fsm` carries: state count
   `N`, the chosen `FsmEncoding { Binary, OneHot, Gray }`, the
   per-state next-state transition table (indices into states,
   selected by a bounded input/condition cone), and the per-state
   Moore output value. State constants are *derived* from
   `(encoding, N)` at emit — never stored redundantly (full
   factorization: the encoding is the identity of the constants).
2. **Opaque leaf.** `Node::FsmOut { fsm: FsmId }` — a sibling to
   `FlopQ`/`MemRead`, **never CSE'd / never factorized** (the FSM is
   a block; its output is an opaque source like a flop's Q). Same
   `compact.rs` reachability obligation discovered in `.2.1a`: a
   reachable `FsmOut` must transitively keep the FSM's
   transition/condition source cones alive (sibling rule to
   `FlopQ`/`MemRead` keeping their D/we/addr cones).
3. **Emitter.** Renders the probed-clean template: generated
   `localparam` state constants per the encoding, the `state_q`
   flop on the shared `clk` with async-low reset to `S0`, the
   next-state `always_comb` `case`, the Moore output `always_comb`
   `case`. Single-clock invariant preserved (no new clock).
4. **Knob.** Opt-in `Config::fsm_prob` serde-default `0.0`
   (default-off ⇒ byte-identical), one roll in the same
   mutually-exclusive opt-in lane as `memory_prob` /
   `width_parameterization_prob` (rules-first `build_fsm_block`,
   never generate-then-filter).

**Rejected alternatives.** (A) **Build the FSM from existing
primitives** (a flop + a hand-rolled mux/`Eq` tree) with *no* block —
rejected: the state encoding is then implicit and *not selectable*,
the motif is unrecognisable as an FSM to a reader or to Yosys's
`fsm` pass, and it defeats the ROADMAP's *"explicitly generated
state encodings"* requirement. (B) **Emitter-only string template**
(no IR `Fsm`) — rejected: not valid-by-construction, can't be
validated, breaks the operators-vs-blocks doctrine the memory work
established. (C) **A generic `enum`/typedef datatype threaded
through the width/IR machinery** — rejected for the same reason
memory's (C) was: a massive invasive change to scalar IR arithmetic;
an FSM is a *block*, not a datatype. (D) **Mealy outputs (outputs a
function of state *and* input)** — deferred, not rejected: Moore-only
keeps the output decode a pure `case (state_q)` (matches the probed-
clean template and the deterministic-output contract); Mealy is a
recorded post-`.3` extension, not a `.3` blocker.

**Proof shape (`.3`, split mirrors `.2`).** `.3` becomes a container
mirroring the proven memory `.2.1`–`.2.4`:

- **`.3.1`** (this slice) — design; design-only, no code.
- **`.3.2`** — IR + opaque `FsmOut` leaf + `compact.rs` reachability
  + emitter + validator scaffold + `fsm_prob` knob + rules-first
  `build_fsm_block` (default-off byte-identical; forced-on focused
  proof). May sub-split `.3.2a`/`.3.2b` (IR-core+reachability /
  knob+generator) **if** implementing it surfaces a lower-level
  dependency, exactly as `.2.1` split on the compaction-reachability
  discovery — decided when reached, not pre-emptively.
- **`.3.3`** — cargo-portable proof (`tests/pipeline.rs`): across
  `ConstructionStrategy × FactorizationLevel × seeds`, the emitted SV
  is *exactly* the probed-clean per-encoding template, exactly one
  `FsmOut` survives every factorization level (CSE/EGraph-opaque),
  all three encodings are reachable and structurally distinct;
  `validate_design` clean; default-off byte-identical reaffirmed.
- **`.3.4`** — `phase6_fsm` matrix scenario + `num_fsm_modules`
  metric + `saw_fsm_design` fact/`Phase4Hierarchy` gap (no ROADMAP
  advance), then the **real repo-owned gate** verified downstream-
  clean (`coverage_gaps=[]`, Verilator + both Yosys all-pass,
  `saw_fsm_design=true`, P4/P5/P5b/P6-memory regressions clean)
  *before* any promotion (r87 no-aspirational-claims). FSM is the
  **last** Phase 6 motif: when `.3.4` verifies clean it both records
  FSM delivered **and** — memory already delivered at `.2.4` — closes
  ROADMAP Phase 6 and the `PHASE-6-ADVANCED-MOTIFS` tree (multi-clock
  CDC stays the explicitly-optional, separately-prioritised deferral
  per the 2026-05-16 Decision; not a Phase 6 blocker).

The cargo gate **cannot** shell Yosys/Verilator (project convention
since Phase 1); downstream cleanliness is proved by `.1`-style probe
(done, above) + the `.3.4` repo-owned `tool_matrix` gate, never in
`cargo test` — identical to how memory and Phase 5/5b were proved.

This entry is design-only and is itself task-tree owned
(`PHASE-6-ADVANCED-MOTIFS.3.1`); it makes no code change, consistent
with the task-tree-ownership doctrine's code/not-code boundary.

**As-built IR shape (2026-05-18, `.3.2a`).** `.3.2` was split up
front into `.3.2a` (IR core + opaque-leaf pipeline integration) and
`.3.2b` (knob + rules-first generator) — the opaque-stateful-leaf
compaction-reachability is correctness-critical pipeline code, known
concretely from the landed memory `.2.1a` (it is *not* mechanical
`FlopQ`-mirroring). `.3.2a` landed and fixes the concrete IR shape
the architecture-(F) sketch left open:

- `FsmEncoding{Binary,OneHot,Gray}` owns the encoding maths:
  `state_width(N)` = `ceil(log2 N)` for Binary/Gray, `N` for OneHot;
  `state_const(s)` = `s` / `1<<s` / `s ^ (s>>1)`. The state constants
  are **derived**, never stored (full factorization: the encoding is
  the identity of the constants).
- `Fsm { num_states, encoding, sel:NodeId, sel_width,
  transitions:[N][1<<sel_width], outputs:[N], out_width }`. A single
  generated `sel` cone drives the next-state decode
  (`next = transitions[state][sel]`); Moore outputs are a per-state
  value table. Reset state is index 0. This is the minimal shape
  that is valid-by-construction and downstream-clean per the `.3.1`
  probe; richer transition conditions are a post-`.3` extension, not
  a `.3` blocker (recorded, like Mealy).
- Emitter detail worth recording: state `localparam`s are emitted
  **per-FSM-prefixed** (`FSM<id>_S<k>`), not the probe's bare `Sk`,
  so multiple FSMs in one module never collide; they are emitted in
  module body just before the FSM `always` blocks (LRM-legal;
  Verilator/Yosys-accepted; the authoritative tool re-verification
  is the `.3.4` repo gate, exactly as for memory).
- Default-off is **trivially** byte-identical: the emitter blocks are
  gated on `!m.fsms.is_empty()`, the predicates only OR when `fsms`
  is non-empty, and the `FsmOut` match arms only fire when a `FsmOut`
  node exists — none of which occur without the (`.3.2b`) generator.

`.3.2b` landed the generator/knob: new calibration knob
`Config::fsm_prob` (`f64`, serde-default `0.0`, probability-range
validated — the same shape as `memory_prob`/`aggregate_prob`/
`width_parameterization_prob`). Rules-first `build_fsm_block`
constructs the FSM leaf *by rule* (it is never a generate-then-filter
— `num_states`/`encoding`/`sel_width`/`out_width` are rolled via
`g.rng` for reproducibility; transitions and distinct masked Moore
outputs are filled deterministically). The opt-in roll is a single
`g.rng.gen_bool` in `generate_leaf_module_with_interface_profile`,
placed **after** the Phase 5 width-parameterization lane and the
Phase 6 memory lane and therefore **mutually exclusive** with both —
the established Phase-5/5b/6-memory opt-in-lane discipline (one
exclusive motif per free-standing single-module design;
`interface_profile.is_none()` only; default-off never enters, so
emission is byte-identical). This keeps the four opt-in motif lanes
(param / aggregate-via-annotation / memory / FSM) from interacting.

### Phase 7 oracle-backed micro-design artifact family design (2026-05-18, PHASE-7-ORACLE-MICRODESIGN.1)

Design-only slice. No code. Lifts ROADMAP Phase 7 ("oracle-backed
micro-design artifacts — `rtl_const_expr`-style corpora") into a
concrete, codebase-grounded plan: the expected-facts schema, the
oracle-by-construction generation strategy, the reproducibility
contract, the parity-check harness shape, the boundary with the
existing DUT lane and Phases 8/9, rejected alternatives, and the
`.2` proof shape + split. Mirrors the proven Phase 5/5b/6
design-first method.

**The conceptual shift (why this is a new family, not a knob).**
Phases 1–6 generate *structurally valid random RTL* whose function
is deliberately meaningless — the contract is "lints/elaborates/
synthesizes clean", there is **no semantic oracle** ("structural,
not meaningful" — `book/src/non-triviality.md`). Phase 7 is the
**opposite**: tiny `.sv` files whose *elaboration facts are exactly
known by construction*, shipped with a machine-checkable manifest,
so a downstream tool can be checked against an **oracle** (does the
tool resolve this parameter / width / generate-branch to the value
ANVIL already knows?), not merely "did it not error". Pressure point
= front-end constant-expression / parameter / elaboration
correctness, not cone-synthesis robustness.

**Codebase grounding.** The existing IR (`src/ir/types.rs`) is a
scalar-`u32` gate-level circuit graph: `Port`/`Node`/`Flop`/
`Memory`/`Fsm`/`Instance`, no notion of `parameter`/`localparam`,
elaboration-time expressions, `generate`, packages, or typed
constants. `WidthExpr{Lit,Param}`/`ParamEnv` (Phase 5) is the
*closest* existing concept but is a narrow width-only annotation on
the circuit IR, not a general constant-expression/elaboration model.
Therefore Phase 7 needs its **own small source-level
constant/parameter IR** — a parameter+localparam dependency DAG of
typed constant expressions with their *evaluated* values — distinct
from and not threaded through the circuit IR (same operators-vs-
blocks / category-boundary discipline that kept memory and FSM as
blocks rather than datatypes). It reuses ANVIL's seeding (ChaCha8,
no `thread_rng`), CLI/knob plumbing, and reproducibility doctrine,
but is a **separate generator path** — it does not go through
`build_cone`.

**Artifact family — `rtl_const_expr` (per ROADMAP).** One module (or
a tiny package+module cluster) exercising, by construction:
parameter/localparam dependency chains
(`localparam B = A*2; localparam C = B + W;`); expression-derived
widths/ranges (`logic [DEPTH-1:0]`, `[$clog2(N)-1:0]`); `generate
if`/`for` whose conditions and bounds are expression-driven;
package-qualified constants (`pkg::WIDTH`); and precedence-sensitive
arithmetic / shift / comparison / equality / bitwise / logical /
ternary expressions. Typical size: one module, or a small cluster
when the pressure point needs local hierarchy.

**Expected-facts manifest (schema sketch).** One JSON manifest per
emitted `.sv`, capturing only *obviously-checkable elaboration
facts* the generator already knows:

```json
{ "seed": <u64>, "top": "<module>",
  "params":   { "<name>": { "value": <int>, "expr": "<src>" } },
  "localparams": { "<name>": { "value": <int>, "expr": "<src>" } },
  "widths":   { "<signal>": { "msb": <int>, "lsb": <int>, "bits": <int> } },
  "generate": { "<label>": { "taken": <bool> | "iterations": <int> } },
  "package_constants": { "pkg::<name>": <int> },
  "const_exprs": [ { "expr": "<src>", "value": <int>, "width": <int> } ] }
```

**Generation strategy — oracle by construction (the key idea).** The
generator builds the parameter/localparam/const-expression DAG and
**evaluates every node as it constructs it** (it chose the literals
and operators, so it computes the resolved integer/width *the same
way SV elaboration must*). The `.sv` text is emitted *from* that
evaluated DAG; the manifest is emitted *from the same resolved
values*. The generator **is** the oracle — there is no separate
analysis pass and no re-parsing of generated text. This is the exact
valid-by-construction / rules-first doctrine that governs the rest
of ANVIL (compute the fact at construction time; never
generate-then-analyze). Evaluation uses wide integer semantics
matching SV's constant-expression rules (2-state, sign/width per
LRM) for the integer subset Phase 7 emits — deliberately bounded so
the oracle is trivially correct.

**Reproducibility contract.** Identical to the DUT lane: `(seed,
knobs)` → byte-identical `.sv` **and** byte-identical `.json`
manifest, on any platform, forever. The manifest is part of the
reproducible artifact, not a side report.

**Parity-check harness.** Separate from the `tool_matrix`
lint/synth DUT gate (that proves *acceptance*; Phase 7 proves *fact
agreement*). The harness elaborates each emitted `.sv` with a
downstream consumer that can report resolved facts — candidate:
Yosys `read_verilog -sv; ... ; write_json` (parameter/width facts)
and/or Verilator/`slang` parameter introspection — and compares the
reported facts to the manifest: exact agreement, or a **retained
counterexample** (the `.sv` + manifest + tool output kept for
triage), never a silent pass. As with memory/FSM, a cargo-portable
formalization is available — the emitted declarations' widths/param
values equal the manifest *by construction* (structural-equivalence,
`cargo test`-able) — while the genuine downstream parity runs in the
repo-owned gate (cargo cannot shell yosys/verilator; project
convention since Phase 1).

**Boundaries.** Phase 7 = *constant/elaboration facts on tiny
modules*. Phase 8 (frontend/elaboration accept corpora) = *compact
elaboratable hierarchies* with a richer source-level hierarchy/
package IR — Phase 7's const-expr IR is the seed of, but smaller
than, Phase 8's. Phase 9 (umbrella) = the artifact-family selector
unifying DUT / Phase-7 / Phase-8 lanes; Phase 7 lands behind an
explicit family flag now and is rehomed under the Phase 9 selector
later. Phase 7 does **not** build the selector (that is Phase 9's
`.1`-blocked-until-≥2-lanes leaf).

**Rejected alternatives.** (A) **Reuse the gate-level circuit IR**
for const-expr artifacts — rejected: it has no parameter/localparam/
generate/package/typed-constant concept; forcing them through scalar
`u32` node graphs is the same category error as memory's rejected
datatype option (C). (B) **Generate random SV then parse it back to
derive the manifest** (generate-then-analyze) — rejected: violates
the oracle-by-construction doctrine and re-implements elaboration in
the oracle, so the oracle can be as wrong as the tool under test;
the generator already holds every resolved value. (C) **Bundle a
reference elaborator** to compute expected facts — rejected: project
non-goal (no bundled reference simulator); the construction-time
oracle is exact and free. (D) **Emit facts as SV comments instead of
a separate manifest** — rejected: not machine-checkable without
re-parsing, and couples the oracle to comment-formatting; a typed
JSON manifest is the durable contract.

**Proof shape (`.2`, expected to split).** Reproducible corpus
(byte-stable `.sv` + `.json` across re-runs and the existing
cross-platform reproducibility harness); a manifest-schema
validator; the parity harness over ≥1 downstream consumer green (or
counterexamples retained); behind an explicit artifact-family flag;
no regression to the DUT lane. Split candidates (independently
reviewable): const-expr/parameter IR + construction-time evaluator /
SV emitter + manifest emitter / parity harness + repo-owned gate.

This entry is design-only and is itself task-tree owned
(`PHASE-7-ORACLE-MICRODESIGN.1`); it makes no code change,
consistent with the task-tree-ownership doctrine's code/not-code
boundary.

**As-built — `.2a` IR + evaluator (2026-05-19).** `.2` split into
`.2a` (IR + evaluator/oracle) / `.2b` (SV + manifest emitters) /
`.2c` (parity harness + gate). `.2a` landed as a **new separate
top-level module `src/microdesign/`** (`pub mod microdesign` in
`src/lib.rs`) — *not* under `src/ir/`, exactly as the design's
rejected-alternative (A) requires (the gate-level circuit IR has no
parameter/localparam/expression concept; it must stay a separate
generator path). Concrete shape decisions worth recording: the
const-expr value type is **`i128`** (the rules-first builder keeps
every intermediate well inside it, so the oracle is *trivially
exact*; width-sized truncation against declared port/param widths is
deferred to `.2b` where widths exist — `.2a` is purely the value
DAG). `eval()` is total except two **defensive** `EvalError`s
(`DivByZero`, `UndefinedParam`) that the rules-first builder never
triggers but a hand-malformed unit must classify rather than panic;
shift amounts are clamped `[0,127]` so a (builder-impossible) huge
amount cannot panic Rust's shift. `resolve()` *is* the oracle: it
runs once at construction time and fills every `ParamDecl.value`;
the load-bearing `.2a` invariant (unit-proven) is that this stored
value never drifts from a fresh re-evaluation of its expression over
the resolved prefix — that equality is *why* `.2b` can emit both the
SV and the JSON manifest from `value` without a second analysis pass
or a re-parse. `build_constexpr_unit(seed,n)` uses the project
ChaCha8 convention verbatim (`ChaCha8Rng::seed_from_u64`, no
`thread_rng`).

**As-built — `.2b` emitters (2026-05-19).** SV + JSON manifest
emitters in the same module, both reading `ParamDecl.value` (the
`.2a` oracle). Decisions worth recording: (1) `expr_to_sv` is
**fully parenthesized** — the evaluator already fixed semantics; a
minimal-parens printer would risk the *downstream* front-end
parsing a different precedence than the oracle computed, so the
printer must not be clever. The precedence-sensitive-expression
axis is still exercised because the `.2a` builder emits genuinely
nested `a + b*c` / ternary shapes that round-trip *as written*.
(2) **Default-off DUT-byte-identical is structural, not a flag
check**: `microdesign` is a separate top-level module that the DUT
generate path never calls, so "the artifact-family flag is off" is
the *absence of a call site* — there is nothing to gate and nothing
that could perturb DUT output. The actual `--artifact` selector is
Phase 9's; `.2b` deliberately does not wire a CLI flag (that would
be premature and is Phase 9's lane-migration concern). (3) The
manifest uses `BTreeMap` for every object so `serde_json`
pretty-output key order is deterministic ⇒ the `.json` is a
byte-stable part of the reproducible artifact, exactly like the
`.sv`. (4) `widths`/`generate`/`package_constants` are derived by
small fixed rules (`(last % 8)+1`; `P0 >= pkg_const(seed)`;
`seed % 64 + 1`) whose *resolved* values come from the oracle —
the SV carries the symbolic form, the manifest the resolved form,
and `manifest_mirrors_the_oracle` pins their equality.

### Phase 8 frontend/elaboration accept-corpus source-IR design (2026-05-18, PHASE-8-FRONTEND-ACCEPT.1)

Design-only slice. No code. Lifts ROADMAP Phase 8 ("frontend/
elaboration accept corpora — compact elaboratable hierarchies")
into a concrete, codebase-grounded plan: why a dedicated
source-level IR, the surfaces it must express, the
expected-elaboration-facts manifest schema, the parity harness, the
relationship to Phase 7 / Phase 9, rejected alternatives, the `.2`
proof shape + split. Mirrors the proven design-first method.

**The shift (and the boundary with Phase 7).** Phases 1–6 emit
*already-elaborated, parameter-resolved* gate-level RTL (the
"structural, not meaningful" DUT lane). Phase 7 is a tiny
*single-module* const-expr oracle (one module, constant facts).
Phase 8 is the **frontend/elaboration** lane: *compact elaboratable
hierarchies* (1–3 modules + packages) emitted with **parameters
unresolved in the SV text**, shipped with a manifest of what
*elaboration must resolve them to*. The pressure point is the
downstream tool's **front-end / elaboration** (parameter override
resolution, instance binding, generate selection, package/type
resolution) — a surface the gate-level circuit IR cannot represent
*at all*.

**Codebase grounding.** The circuit IR (`Port`/`Node`/`Flop`/
`Memory`/`Fsm`/`Instance` in `src/ir/types.rs`) is *post-
elaboration*: scalar `u32` nets, resolved widths, flattened/
instantiated modules. It has no module-declaration, parameter-port,
`localparam`, package, `typedef`/struct/union/enum, procedural-block
or `generate` concept. Phase 5's `ParamEnv` and Phase 7's const-expr
DAG are *sub-models* (resolved-width annotation; single-module
constant facts) — neither expresses a hierarchy of un-elaborated
module declarations. Phase 8 therefore needs a first-class
**source-level AST IR** that emits *un-elaborated* SV, distinct
from and not threaded through the circuit IR (the roadmap decree +
the same category-boundary discipline that kept memory/FSM as
blocks). It **reuses Phase 7's construction-time integer/const-expr
evaluator and JSON-manifest core** (do not reimplement) and ANVIL's
seeding/CLI/reproducibility; it is a **separate generator path**.

**Surfaces the source IR must express (per ROADMAP).** ANSI port
lists + parameter ports; parameter/localparam flows across
instances; instantiation variants — named/ordered parameter
overrides, named/ordered/wildcard (`.*`) port connections, instance
arrays; package imports + package-qualified constants/types;
typedef-backed types — packed/unpacked structs, unions, enums,
builtin integral atoms (`int`/`byte`/`logic`/…); the full
`assign` / `always_comb` / `always @(*)` / `always_ff` /
`always_latch` set; `generate if` / `for`.

**Source-IR sketch.**

```
SourceUnit   = { packages: Vec<Package>, modules: Vec<Module> }   // ordered, top last
Package      = { name, items: Vec<PkgItem /* Localparam | Typedef */> }
Module       = { name, params: Vec<ParamDecl>,            // #(parameter ...)
                 ports:  Vec<PortDecl /* ANSI, typed, dir */>,
                 items:  Vec<ModuleItem> }
ModuleItem   = Localparam(name, Expr)
             | VarDecl(name, Type)
             | Typedef(name, Type)
             | ContinuousAssign(lhs, Expr)
             | Always(kind: Comb|FfPosedge|Latch|StarAt, body)
             | Instance{ target, params: Named|Ordered(Vec<Expr>),
                         ports: Named|Ordered|Wildcard, array: Option<RangeExpr> }
             | Generate(If{cond: Expr} | For{genvar, bound: Expr})
Type         = Logic{packed_dims} | Atom(int|byte|…) | Enum{base,members}
             | Struct{packed,fields} | Union{fields} | Named(typedef)
             | PkgQual(pkg,name)
Expr         = the Phase 7 const-expr node set (reused), over
               parameters/localparams/genvars/package constants.
```

Every `ParamDecl`/`Localparam`/generate condition carries its
**construction-time-evaluated** value (Phase 7's evaluator), so the
manifest is exact and the SV text can stay un-elaborated.

**Expected-elaboration-facts manifest (extends Phase 7's schema).**
Per emitted top, JSON, byte-stable: resolved top parameter values;
the **instance tree** (instance path → target module → resolved
child parameter values → child port bindings); selected `generate`
branches / unrolled `for` iteration counts; package constant/type
resolutions; typedef-resolved widths. The Phase 7
`params`/`localparams`/`widths`/`const_exprs` blocks are reused
verbatim; Phase 8 adds `instances`, `generate`, `packages`,
`typedefs`.

**Generation strategy — oracle by construction (reuse Phase 7's
evaluator).** Identical doctrine: the generator chooses the
hierarchy + parameter values and *performs the elaboration itself*
at construction time (it knows the instance tree, override
resolution, and generate selection because it built them); it emits
*un-elaborated* SV **and** the elaborated-facts manifest from the
same resolved knowledge — no analysis pass, no re-parse, no bundled
elaborator. The novelty vs Phase 7: the SV text deliberately keeps
parameters symbolic (`foo #(.W(W*2)) u();`) and the manifest asserts
the elaboration result (`u.W == 16`) — that gap is exactly the
front-end behaviour under test.

**Open-Question resolution (reuse of Phase 7 manifest machinery).**
**Resolved**: Phase 8 *reuses* Phase 7's construction-time
evaluator + JSON-manifest emitter core and *extends* the schema
with hierarchy/instance/generate/package facts. Dependency
direction: `PHASE-8-FRONTEND-ACCEPT.2` depends on
`PHASE-7-ORACLE-MICRODESIGN.2`'s evaluator/manifest core landing
first (recorded so `.2` sequences correctly). Phase 9 unifies the
artifact-family selector; Phase 8 lands behind an explicit family
flag, not the selector.

**Parity harness.** Same shape as Phase 7 but hierarchy-aware: a
downstream elaborator (Yosys `read_verilog -sv; hierarchy -top …;
write_json`, and/or `slang`/Verilator hierarchy+param
introspection) reports the elaborated hierarchy facts; the harness
compares to the manifest — exact agreement or a **retained
counterexample**. Repo-owned gate (cargo cannot shell
yosys/verilator — the Phase-1 convention); a cargo-portable
structural-consistency slice (emitted declarations vs the
generator's own resolved values) complements it.

**Rejected alternatives.** (A) **Reuse the gate-level circuit IR** —
rejected by roadmap decree *and* structurally: it is
post-elaboration and cannot express modules/parameters/packages/
generate. (B) **Emit already-elaborated SV** (parameters
pre-resolved in text) — rejected: that is the Phases 1–6 DUT lane;
it exercises synthesis, not the front-end/elaboration path Phase 8
exists to stress; un-resolved-text-plus-manifest *is* the contract.
(C) **A full SV parser/elaborator inside ANVIL to derive facts** —
rejected: oracle-by-construction makes it unnecessary and it would
re-introduce the very elaboration bugs under test (same as Phase
7's (B)/(C)). (D) **Extend Phase 7's single-module const-expr IR
in place** instead of a dedicated hierarchy/package IR — rejected:
hierarchy/instantiation/packages are a categorically larger
surface; cramming them into the const-expr DAG repeats the
circuit-IR category error. Phase 8 *reuses Phase 7's evaluator* but
is its own structural source IR.

**Proof shape (`.2`, expected to split).** Reproducible 1–3 module
accept corpora (byte-stable SV + manifest, cross-platform); the
source IR emits valid un-elaborated SV (the downstream tool
elaborates it clean); manifest-schema validation; parity harness
green or retained counterexamples; behind the artifact-family flag;
no DUT-lane regression. Split candidates (independently reviewable):
source IR + construction-time elaboration-evaluator (reusing the
Phase 7 core) / SV emitter + manifest emitter / parity harness +
repo-owned gate.

This entry is design-only and is itself task-tree owned
(`PHASE-8-FRONTEND-ACCEPT.1`); it makes no code change, consistent
with the task-tree-ownership doctrine's code/not-code boundary.

### Phase 9 multi-artifact umbrella selector design (2026-05-18, PHASE-9-MULTI-ARTIFACT-UMBRELLA.1)

Design-only slice. No code. Lifts ROADMAP Phase 9 ("multi-artifact
ANVIL umbrella — an artifact-family selector with shared plumbing")
into a concrete plan: the lane interface, the shared
reproducibility/manifest/seed/output/check contract, the
CLI/selector surface, the lane-migration plan, rejected
alternatives, the `.2` proof shape + split. Designed *now* (per the
tree's 2026-05-16 Decision) so Phases 7/8 are built
selector-compatible rather than retrofitted.

**The point (and the explicit anti-goal).** Phase 9 makes one tool
drive every valid-by-construction lane *with the lanes kept
separate*. The anti-goal it exists to prevent: collapsing into "one
generator that emits random SV files" with contradictory promises.
The lane *interface* unifies **plumbing** (seed, knobs,
reproducibility, manifest, output layout, downstream dispatch); it
does **not** merge the generators.

**The lanes.**

- **L1 — DUT RTL** (Phases 1–6): structurally-valid random
  synthesizable RTL; oracle = lint/elaborate/synth-clean (the
  `tool_matrix` gate). Generator = `build_cone`/hierarchy; circuit
  IR. **No semantic manifest** (deliberate — "structural, not
  meaningful").
- **L2 — oracle-backed micro-design** (Phase 7): tiny const-expr
  `.sv` + expected-facts manifest; oracle = fact agreement (parity).
  Const/param IR.
- **L3 — frontend/elaboration accept** (Phase 8): compact
  un-elaborated hierarchies + elaborated-facts manifest; oracle =
  elaboration-fact agreement (hierarchy parity). Source AST IR.
- Future valid synthesizable lanes plug in via the same contract.

**Lane interface (the abstraction).**

```
trait ArtifactLane {
    fn name(&self) -> &str;                 // "dut" | "oracle-microdesign" | "frontend-accept"
    fn validate_knobs(&self, &Config) -> Result<(), ConfigError>; // lane-scoped only
    fn generate(&self, seed, &Config) -> Corpus;   // (seed,knobs) -> byte-stable artifacts
    fn manifest(&self, &Corpus) -> Option<Manifest>;// None for L1 (first-class, not a hack)
    fn check_plan(&self, &Corpus) -> CheckPlan;     // SynthAccept (L1) | ParityVsManifest (L2/L3)
}
```

Shared plumbing the umbrella owns (never duplicated per lane):
ChaCha8 seed→artifact derivation + byte-stable cross-platform output
(today's doctrine, centralized); the JSON manifest emitter + schema
versioning (Phase 7 core; `Option` so L1's absence is typed, not a
sentinel); a lane-scoped knob namespace (each lane validates only
its knobs; cross-lane knob bleed is rejected); a uniform on-disk
layout (`<out>/<lane>/<scenario>/… [+ manifest.json]`); a uniform
`CheckPlan` the repo-owned gate dispatches (synth-accept for L1,
parity-vs-manifest for L2/L3).

**CLI/selector surface — Open-Question resolution.** **Resolved**:
a top-level **`--artifact <lane>` flag on the existing `anvil`
binary, default `dut`**. Default-`dut` ⇒ every current invocation,
the entire book, and CI keep working **byte-identically** (this is
load-bearing — `BOOK-EXAMPLES-RUNNABLE` made hundreds of
`cargo run --release -- …` examples a CI-gated contract; a
subcommand-only redesign would regress all of them). `--artifact
oracle-microdesign` / `--artifact frontend-accept` opt into L2/L3.
`tool_matrix` stays the L1 gate harness; the umbrella adds lane
dispatch, not a rewrite. Rejected forms recorded below.

**Lane-migration plan.** L1 is wrapped as the **default** lane with
**zero behaviour change**: `DutLane::generate` *is* today's
`generate_design`; the default selector reproduces every existing
seed byte-identically (a hard regression gate in `.2`). L2/L3 are
built against this `ArtifactLane` contract from the start (Phases
7.2 / 8.2 implement to it — the reason `.1` is designed early), so
there is **no retrofit**. The shared
`(lane, seed, lane_knobs) → byte-identical corpus (+ manifest)`
contract is a strict superset of today's `(seed, knobs)` DUT
contract with `lane` prepended and `dut` defaulted.

**Rejected alternatives.** (A) **Separate binaries per lane** —
rejected: duplicates seed/knob/reproducibility plumbing, fragments
the "one go-to tool" goal, multiplies the CI/book surface. (B)
**One generator path emitting all families via mode flags inside
`build_cone`** — rejected: the explicit anti-goal; synth-clean vs
oracle-exact vs elaboration-accept are contradictory promises that
cannot share one generator without category errors — unify the
*interface*, not the generators. (C) **Subcommand-only CLI**
(`anvil gen-dut …`) — rejected: breaks the existing flat CLI and the
entire CI-gated book example surface for no plumbing benefit a
default-`dut` `--artifact` flag does not already provide. (D)
**Defer the abstraction until ≥2 lanes exist** — rejected by the
tree's standing Decision: designing it now is exactly what keeps
Phases 7/8 lane-compatible instead of retrofitted.

**Proof shape (`.2`, blocked until ≥2 delivered lanes).** The
`ArtifactLane` contract + shared plumbing implemented; the DUT lane
wrapped default-`dut` **byte-identical** (every existing seed
reproduces — hard regression gate, incl. the book/CI examples); ≥1
of L2/L3 selectable via `--artifact`; uniform output layout +
manifest plumbing; lane-scoped knob validation; no
DUT-lane/book/CI regression. Unblock condition (recorded in the
tree): the DUT lane plus ≥1 of Phase 7/8 lanes exist. Split
candidates (independently reviewable): lane trait + shared plumbing
/ DUT-lane wrap (byte-identical regression-gated) / first non-DUT
lane wired to the selector.

This entry is design-only and is itself task-tree owned
(`PHASE-9-MULTI-ARTIFACT-UMBRELLA.1`); it makes no code change,
consistent with the task-tree-ownership doctrine's code/not-code
boundary.

### Second-simulator (iverilog) compatibility note (2026-05-18, DIFFERENTIAL-SIMULATION.1)

Research-only slice (no code). Establishes which second simulator
can ingest ANVIL's existing Verilator-clean SV and where it would
diverge, so `DIFFERENTIAL-SIMULATION.2`'s harness has a concrete
target.

**Empirical ingest probe.** Installed Icarus Verilog **13.0
(stable)** and ran `iverilog -g2012 -o /dev/null <files>`
(SV-2012, full parse + elaborate) against freshly-generated release
output for every ANVIL output category, with `verilator
--lint-only` on the same files as the contrast:

| Category | sample | `iverilog -g2012` | `verilator --lint-only` |
| --- | --- | --- | --- |
| combinational leaf | `--seed 7 --flop-prob 0` | **exit 0, silent** | exit 0, clean |
| sequential leaf (flops) | `--seed 5 --flop-prob 1.0` | **exit 0, silent** | exit 0, clean |
| bounded recursive hierarchy (4 modules) | `--min/max-hierarchy-depth 2`, 2 inst | **exit 0, silent** | exit 0, clean |
| helper-instance / sibling routes (3 modules) | `--hierarchy-sibling-route-prob 1.0` | **exit 0, silent** | exit 0, clean |

**Verdict: iverilog is a zero-configuration second simulator for
every ANVIL output category.** No source edits, no compat shims,
no per-category flags — only the standard `-g2012` SV-2012 select
(ANVIL emits SystemVerilog: `always_ff`/`always_comb`, packed
part-selects, `{N{x}}` replication, async-reset flops, ANSI ports,
multi-module hierarchies). Both engines accept all four categories,
so the **chosen differential pair is Verilator ↔ iverilog** — and
it is a *strong* pair precisely because the engines are
semantically independent: **Verilator** is a compiled,
2-state-by-default, cycle-driven simulator; **iverilog** is an
interpreted, 4-state (`0/1/x/z`), event-driven simulator. Agreement
across that gap is meaningful corroboration, not two views of the
same engine.

**Where they will diverge (the `.2`/`.3` harness must design around
this — not an ingest blocker).** ANVIL output is combinational +
synchronous-reset flops with no `X`/`Z` injection, so the only
material Verilator/iverilog semantic gap is **pre-reset 4-state
behaviour**: iverilog drives flops `x` until the async reset
deasserts; Verilator (2-state default) starts them `0`. Therefore
the differential harness must (a) drive a deterministic reset
sequence first, (b) sample outputs **only at a single canonical
post-reset point**, and (c) compare defined bits only. Combinational
cones are pure functions of inputs ⇒ no timing gap once inputs are
held. These are exactly the Open Questions the tree already routes
to `.2` (input-vector scheme; canonical sample point; timing) —
this note confirms they are *design* problems, not *feasibility*
blockers.

**Rejected alternatives.** (A) `verilator --binary` self-vs-self —
rejected: same engine, zero independent corroboration (the whole
point is engine independence). (B) Yosys as the sim peer — rejected
(already in tree Decisions): Yosys is a *synthesizer*, not an
event-driven simulator; it cannot be a semantic-equivalence peer.
(C) Commercial simulators (VCS/Xcelium/Questa) — deferred (tree
Decision): unavailable in-environment; the open-source pair already
gives independent corroboration. (D) Single-simulator (Verilator
only) — rejected: cannot prove *cross-simulator* agreement, which
is the signoff-quality bar this tree exists to raise.

This entry is research-only and is itself task-tree owned
(`DIFFERENTIAL-SIMULATION.1`); it makes no code change, consistent
with the task-tree-ownership doctrine's code/not-code boundary
(`.2`+ build the harness).

### Single-design differential harness design (2026-05-18, DIFFERENTIAL-SIMULATION.2a)

Design-only slice (no code; `.2b` implements). `.2` was split —
the harness's testbench-generation strategy, reset/sample
alignment, stimulus determinism, dual-simulator orchestration, and
the tool-gated-test convention are load-bearing decisions that
should be settled and reviewed before code (and the design itself
is docs-only, ~zero contention on the near-complete Phase 6 gate,
mirroring the Phase 7/8/9 design-first discipline).

**Goal.** A single-design utility: given a canonical
`(seed, config)`, drive the generated module through **both**
Verilator and iverilog and return aligned output traces, so `.2b`'s
focused test can assert they agree byte-for-byte. Builds directly
on `.1` (iverilog is zero-config-compatible; the only divergence is
pre-reset 4-state).

**Testbench generation — from the IR, not by parsing SV.** The
harness generates the design *in-process* via the library (exactly
like `tests/snapshots.rs`), so it already holds the typed
`Design`/`Module`: port names (`i_*`/`o_*`/`clk`/`rst_n`), widths,
directions, and whether the module carries sequential state
(`has_local_flops()/has_local_memories()/has_local_fsms()`). The
generic SystemVerilog testbench is emitted **from that IR** — never
by re-parsing emitted SV (brittle, a re-implementation of the
front-end). The testbench: instantiates the DUT, drives each input
from a baked deterministic vector sequence, and `$display`s each
output as fixed-width hex at the canonical sample point(s) into a
trace file. One identical testbench file feeds both simulators.

**Reset + canonical sample point (neutralises `.1`'s divergence).**
Per `.1`, the only Verilator/iverilog semantic gap on ANVIL output
is pre-reset 4-state (`iverilog` flops `x` until async reset
deasserts; Verilator-2-state starts `0`). The testbench therefore:
combinational module → hold each input vector, sample the outputs
after a settle delay (no clock); sequential module → assert
`rst_n = 0` for a fixed K cycles, deassert, then for each of N
cycles apply the next input vector and sample outputs **at a single
fixed post-reset cycle offset** (a deterministic warmup then
per-cycle sampling). Only post-reset, fully-defined samples are
compared — the pre-reset `x`/`0` gap is never observed.

**Deterministic stimulus — baked, not per-sim `$random`.** Input
vectors are computed in Rust from the seed (a reproducible
sequence: zero, all-ones, walking-1, then seeded pseudo-random) and
**baked into the testbench as constants**. `$random` is *not* used:
iverilog and Verilator have different `$random` streams, which
would inject false mismatches. Baked identical stimulus guarantees
both simulators see exactly the same inputs.

**Dual-simulator orchestration.** (a) iverilog:
`iverilog -g2012 -o sim.vvp dut.sv tb.sv` then `vvp sim.vvp`
→ trace A. (b) Verilator:
`verilator --binary -j0 -sv --top-module tb dut.sv tb.sv` (5.x
`--binary` builds a runnable directly from the *same* testbench)
then run the produced binary → trace B. Both `$display` the
identical fixed-width-hex trace format; the harness byte-compares A
vs B and returns the aligned traces (+ a structured diff on
mismatch — never a silent pass; a mismatch is a *retained
counterexample* with the SV + stimulus, mirroring the Phase 7
parity-harness discipline).

**Tool-gated test convention (load-bearing — Phase-1 doctrine).**
`cargo test` must pass on machines without verilator/iverilog (the
convention since Phase 1; reaffirmed for memory/FSM `.2.2` and the
tool_matrix gate). So `.2b`'s focused differential test is
`#[ignore]` by default — run explicitly (`cargo test -- --ignored
diff_sim`) or from a repo-owned context where both simulators are
present. The harness itself is a plain utility fn; the *gated* test
is the only tool-requiring surface, so the portable `cargo test`
stays green tool-less and `.2b` adds ~zero mandatory-gate runtime.

**Rejected alternatives.** (A) Parse emitted SV text to discover
ports — rejected: brittle front-end re-implementation; the IR has
exact port info already. (B) Per-simulator `$random` stimulus —
rejected: divergent streams ⇒ false mismatches; bake identical
vectors. (C) Make the differential test a normal (non-`#[ignore]`)
`cargo test` — rejected: breaks the tool-less-portability doctrine.
(D) Verilator `--cc` + a hand-written C++ main — rejected vs
`--binary`: more moving parts and a second harness language;
`--binary` runs the *same* SV testbench iverilog uses, keeping one
testbench for both. (E) Compare full cycle-by-cycle traces incl.
pre-reset — rejected: re-introduces exactly the `.1` 4-state gap;
post-reset canonical sampling is the correct contract.

**Proof shape (`.2b`).** A `#[ignore]` focused test builds a
hand-picked combinational and a sequential `(seed, config)` leaf,
runs the harness (both simulators, post-reset aligned traces), and
asserts byte-equality; `cargo fmt/clippy/check/test` green with the
diff-sim test ignored by default. `.3` wires it into `tool_matrix
--diff-sim` over a representative subset + the
`saw_design_with_cross_simulator_agreement` fact; `.4` documents
the contract (README/USER_GUIDE/book).

This entry is design-only and is itself task-tree owned
(`DIFFERENTIAL-SIMULATION.2a`); it makes no code change, consistent
with the task-tree-ownership doctrine's code/not-code boundary.

### Coverage baseline triage — top-5 under-covered files (2026-05-18, COVERAGE-INSTRUMENTATION.2)

Triage-only slice (no code; `.3` acts on these findings). Classifies
every top-5 under-covered file from `docs/coverage-baseline.md`
(85.26% lines overall) into: **(a) dead code → remove**,
**(b) rarely-fired real path → add a focused proof**, **(c)
intentionally unreachable / integration-only → leave + document**.
Method: reasoned code inspection (orphan-symbol audit, panic/
rollback-site enumeration, `Err`-return vs inline-test count), not a
coverage re-measure.

| # | File | Uncov / % | Disposition | `.3` action |
| --- | --- | --- | --- | --- |
| 1 | `bin/tool_matrix.rs` | 1951 / 72.07% | **(c)** gate-exclusive. Every `*_focus_config` / scenario-builder is referenced from `build_scenarios` — **no orphan/retired builders, zero dead code**. The miss is the `Phase4Hierarchy` scenario + per-scenario config helpers, which fire only under the matrix gate the baseline *deliberately* excludes (75-min runtime). Already exercised by the repo-owned gate. | None. Optionally a "deep" `cargo llvm-cov` incl. the gate for an occasional refresh — not every-slice discipline. |
| 2 | `gen/cone.rs` | 454 / 88.65% | **(b) + (c)** — the **only real proof-gap in the top-5**. 45 panic/`expect`/`unreachable` sites: most are (c) by-construction-invariant guards. But `build_cone_with_retry`'s **retry-budget-exhaustion** path (`⚠️ cone retry budget exhausted`), `rollback_construction_snapshot`, the **anti-collapse reject / skipped-emission** branches, and `pick_terminal`'s adapter fallback are (b) genuinely reachable under specific knob/seed pressure. | `.3`: add focused proofs forcing (i) empty-dep-root retry→rollback→exhaustion, (ii) an anti-collapse reject, (iii) the `pick_terminal` adapter fallback. Leave the invariant-guard `expect`s. |
| 3 | `ir/validate.rs` | 254 / 75.07% | **(c)** intentional defensive validation. 62 `return Err(ValidateError::…)` arms; 26 inline tests already drive the malformed-input-reachable subset (hand-crafted broken modules). The residual arms guard "cannot happen from any generator path" invariants — the safety net the valid-by-construction doctrine relies on; **not dead, not a meaningful proof gap**. | Leave + documented here. Optional low-priority `.3`: a few more hand-broken-IR unit tests for the highest-value invariants. |
| 4 | `config.rs` | 250 / 67.87% | **(c) + audit** integration-only. Unit tests build `Config` via `..Config::default()`, bypassing the clap/serde-default + probability-range validation arms (only a real binary invocation drives them). 137 `pub` fields / 37 validate sites; the orphan-builder-style check found no retired symbols, but a per-field *wiring* audit was out of scope for triage. | `.3`: spot-audit for orphan knobs no longer wired (baseline-flagged); otherwise integration-style binary invocations, lower leverage. |
| 5 | `main.rs` | 142 / 60.56% | **(c)** clap-derive + flag→`Config` overlay boilerplate, exercised only by real binary runs (no test spawns the binary). **Lowest leverage of the five**; not dead, not a real proof gap. | None / optional `.3` binary-smoke with a few flag combos. |

**Headline finding (right-sizes `.3`).** There is **no confirmed
dead code** in the top-5 — the 3314 headline uncovered lines are
*gate-exclusive* (`tool_matrix`), *intentional defensive*
(`validate.rs`), or *integration-only* (`config.rs`/`main.rs`) **by
design**, not test debt. The single high-value `.3` target is a
**handful of `gen/cone.rs` focused proofs** (retry-exhaustion /
anti-collapse-reject / adapter-fallback). `.3` should therefore be
scoped to those cone proofs + an optional `config.rs` orphan-knob
spot-audit — *not* a broad coverage-chasing exercise. This is the
honest disposition the baseline's "(a)/(b)/(c) per file" promise
asked for.

This entry is triage-only and is itself task-tree owned
(`COVERAGE-INSTRUMENTATION.2`); it makes no code change, consistent
with the task-tree-ownership doctrine's code/not-code boundary
(`.3` performs the code actions).

**`.3` outcome (2026-05-18, COVERAGE-INSTRUMENTATION.3 — tree
CLOSED).** Acted on the triage exactly as scoped, nothing more:

- **(b) cone.rs #2 — the one real proof-gap — closed.** Added
  `tests/pipeline.rs::constant_pressure_exhausts_cone_retry_and_stays_valid_and_reproducible`
  (4 `ConstructionStrategy` × 4 seeds, `constant_prob = 1.0`,
  `max_depth = 1`). `constant_prob = 1.0` makes `pick_terminal`
  always take its "emit fresh constant" branch ⇒ every cone root is
  empty-dep ⇒ `build_cone_with_retry` runs the empty-dep retry +
  `rollback_construction_snapshot` loop across all `MAX_RETRIES`
  then the "⚠️ retry budget exhausted, accepting last attempt"
  fallback. The proof pins the invariant those branches exist to
  guarantee: *maximum constant pressure cannot break the pipeline*
  — `generate_design` stays `validate_design`-clean and
  byte-reproducible (no panic / infinite-loop / invalid IR;
  trivially-constant outputs are accepted, not fatal). Soundness +
  reproducibility are asserted, *not* non-triviality (the fallback
  is documented to allow trivially-constant outputs).
- **(a) config.rs #4 — orphan-knob spot-audit — no dead code.** Of
  74 `pub Config` fields, exactly 3 have zero external field-access:
  `library_prob`, `max_nodes_per_module`, `use_async_reset`. All
  three are **intentionally-reserved** and *already documented as
  such* in `book/src/knobs.md` — a future Phase-4+ probabilistic
  dial, a safety ceiling "not typically tuned", and "currently
  unused; flops are always async-reset by discipline",
  respectively. They are serde/CLI-stable knobs whose removal would
  break config compatibility and contradict the book. **Disposition:
  leave as-is** — confirms `.2`'s "no confirmed dead code"
  headline, with the orphan-knob question now positively resolved.
- **(c)** the gate-exclusive (`tool_matrix.rs`), intentional-
  defensive (`validate.rs`), and integration-only
  (`config.rs`/`main.rs`) regions are left exactly as `.2`
  documented — not test debt.
- **Baseline refreshed** via `cargo llvm-cov --release` (the
  instrumented full suite, which also served as this slice's
  COMMIT.md `cargo test` gate); `docs/coverage-baseline.md` carries
  the refreshed numbers + a `.3` addendum. Net: the
  `COVERAGE-INSTRUMENTATION` tree is **closed** with the single real
  proof-gap closed and every other "gap" positively confirmed
  intentional — no broad coverage-chasing, exactly the honest
  outcome `.2` argued for.

### Phase 5b packed-aggregate emitter projection design (2026-05-17, PHASE-5B-AGGREGATES.1)

Design-only slice. No code. Lifts `book/src/ir.md` "Synthesizable
aggregates" (the **packed** sub-question only) into a concrete,
codebase-grounded implementation plan with a rejected-alternatives
trail and a proof shape, so the implementation leaf
(`PHASE-5B-AGGREGATES.2`) has an unambiguous target.

**Goal (from ROADMAP Phase 5b / book "Synthesizable aggregates").**
Emit packed `struct` / `union` / `array` as an **opt-in projection
over the existing flat IR**, valid by construction, downstream-clean
(Verilator + both Yosys modes). Purpose is **parser/elaboration
coverage** in downstream tools, not new synthesis behaviour: a packed
aggregate is semantically a flat bit vector (synthesis treats it as
concatenation with named field-access sugar). Default-off /
byte-identical; no IR restructuring; no Phase-4/Phase-5 dependency;
never retire existing behaviour.

**Code reality that constrains the design** (audited; key anchors):

- The emitter is an explicit **dumb serialiser**
  (`src/emit/sv.rs:49-56` `to_sv_with_modules`): it walks `m.nodes`
  in order, assumes every IR invariant was enforced upstream, does no
  filtering or reachability. The module surface is built from flat
  scalar vectors only: header `module {name} (` / `#( parameter int
  {W} = {D} )` (sv.rs:79-118), ports `input|output logic {wd} {name}`
  via `param_width_decl` (sv.rs:91-116), internal `wire|logic {wd}
  {name};` per `Node::Gate`/`InstanceOutput` (sv.rs:140-173), flop
  `logic {wd} {name};` (sv.rs:122-130), then combinational `assign`s,
  child instance port connections (sv.rs:~315) and output-port
  `assign`s (sv.rs:376-380).
- `Port { id, name, width: u32, dir }` (`src/ir/types.rs:24-29`) and
  every `Node::*`/`Flop` width is a bare `u32`. There is **no**
  aggregate/struct concept anywhere in the IR, validators
  (`src/ir/validate.rs`), CSE keys (`intern_gate`/`intern_constant`),
  or the dedup signature (`canonical_module_signature`,
  `src/metrics.rs`).
- **Phase 5 set the exact precedent to follow.** `param_env:
  Option<ParamEnv>` + `WidthExpr` (`src/ir/types.rs:31-69`) is a
  per-module annotation the IR body never reads; only the emitter
  consults it at the `param_width_decl` width chokepoint, and the
  identity rule consults it in `canonical_module_signature`. The flat
  `width: u32` fields were intentionally untouched. Default-off
  (`param_env == None`) ⇒ byte-identical emission. Phase 5b is the
  same shape one layer out: an emitter-consulted annotation that
  regroups *which* ports render as a packed aggregate, with the IR
  body still flat.

**Architectural decision — chosen: (P) emitter-only packed-aggregate
projection driven by a per-module annotation.** Mirror Phase 5's
annotation-consulted-only-by-emitter architecture (C):

1. Construction is **unchanged**. Modules are built exactly as today
   over flat `u32`-width ports/nodes; all fold/validate/CSE/dedup
   machinery runs untouched.
2. A post-construction, opt-in pass records a lightweight per-module
   annotation (working name `AggregateLayout`): a small additive,
   `Default`-able `Module` field (zero churn to `..Module::default()`
   sites, exactly as `param_env`/`parameterized_*_ports` were added)
   describing **how a contiguous, same-direction subset of ports
   maps onto one packed type**: kind (`StructPacked` |
   `UnionPacked` | `ArrayPacked`), the chosen type name
   (`{module}_{in|out}_t`), and the ordered `(field_name, PortId)`
   list. The bit layout is the existing port concatenation order — a
   **bijective, bit-layout-preserving regrouping**, semantically a
   no-op (the synthesised netlist is identical to the flat form).
3. Emitter learns the projection at the same chokepoints Phase 5
   touched: emit `typedef struct packed { logic [w-1:0] f0; … }
   {module}_in_t;` (and/or union/array) before the module, replace
   the grouped port list with one aggregate port, and rewrite
   references to a grouped port from `name` to `agg.fieldN` (a pure
   rename at the SV surface — the internal flat wires/assigns are
   unchanged; only the port-boundary read/drive uses `.fieldN`). For
   `union`, all members share the same total width (legal because the
   group's total width is fixed); for `array`, the fields are
   same-width slots. No annotation (default-off) ⇒ byte-identical.
4. Knob surface: opt-in `aggregate_*_prob` (`f64`, serde-default
   `0.0`, probability-range validated) — same pattern as
   `width_parameterization_prob`. Default 0.0 ⇒ no annotation ⇒
   byte-identical for fixed seeds. (Single `aggregate_prob` + a
   kind-choice sub-roll vs three separate probs is a `.2`
   calibration sub-decision; the design only fixes "opt-in,
   default-off, serde-default".)

**Soundness rule.** A packed `struct`/`union`/`array` is *defined* by
the SV LRM to be bit-equivalent to the concatenation of its members;
the projection only chooses a syntactic surface for a fixed bit
layout the flat form already had. Therefore the projection is **valid
by construction** for *every* generated module whose grouped ports are
contiguous and same-direction, with **no** validator participation and
**no** generate-then-filter: it is a construction-time emitter rule,
not a post-hoc text rewrite. Downstream-cleanliness follows from the
equivalence and is *proven*, not assumed, by the matrix gate.

**Identity interaction (resolves the tree's Open Question).**
`canonical_module_signature` is computed from the **flat IR**, which
the projection never mutates. The aggregate annotation is *not* hashed
into the signature (unlike Phase 5's `param_env`, which had to be,
because parameterization changes the legal width set — aggregates
change *nothing* semantic). Consequence: a module and its
aggregate-projected twin share one signature and **dedup-collapse**,
which is correct (they are the identical circuit). `dedup_modules`
unchanged. This is the opposite of the Phase 5 identity rule and is
deliberate.

**Rejected alternatives.**

- **(A) First-class aggregate IR nodes** (`struct`/`union`/`array`
  variants in `Port`/`Node`, width-aware). Rejected: a massive
  invasive change rippling through `validate.rs`, the
  `intern_gate`/`intern_constant` CSE keys, `canonical_module_signature`
  + `dedup.rs`, and all per-op width arithmetic — for **zero new
  synthesis behaviour** (packed aggregates are semantically flat).
  Directly violates the book's "keep the IR flat" and this tree's
  Non-Goal "any IR restructuring; aggregates are an emitter projection
  over the existing flat IR". It is the strict superset only if/when a
  *semantically distinct* aggregate (unpacked memory) is pursued —
  that is Phase 6, not here.
- **(B) Post-hoc textual/AST rewrite of the emitted SV string.**
  Rejected: fragile, can desync from the IR, and is exactly the
  post-hoc-rewrite / generate-then-filter anti-pattern the project
  doctrine forbids (rules-first, construction-time). The projection
  must be a deterministic emitter rule reading a recorded annotation,
  not a regex pass over `to_sv` output.
- **(C) Unpacked aggregates / enums in this phase.** Rejected /
  deferred (restated so the deferral is not silently revisited, per
  the existing 2026-05-16 tree Decision): unpacked array is the Phase
  6 memory-inference motif; unpacked datapath `struct`/`union` is
  mostly non-synthesizable; enums are thin (typed constant sets with
  no stress value beyond constants). Phase 5b is **packed-only**.

**Proof shape (for `.2`).** (1) Default-off byte-identical for fixed
seeds across all `ConstructionStrategy` values (no annotation ⇒
identical `to_sv`). (2) Forced-on: a focused proof that a projected
module's emitted SV declares a `typedef … packed` and a single
aggregate port, and that field references resolve. (3) A
`tool_matrix` aggregate scenario downstream-clean: Verilator
`--lint-only` + both Yosys modes all-pass, `coverage_gaps=[]`, a new
`saw_packed_aggregate_design` coverage fact. (4) Identity-invariance:
a unit test that a module and its aggregate-projected twin produce the
**same** `canonical_module_signature` (annotation not hashed) and
dedup-collapse. (5) Full `cargo` hygiene gate; `mdbook` clean with
`book/src/ir.md` "Synthesizable aggregates" reconciled to what landed
and `book/src/knobs.md` documenting `aggregate_*_prob`.

This entry is design-only and is itself task-tree owned
(`PHASE-5B-AGGREGATES.1`); it makes no code change, consistent with
the task-tree-ownership doctrine's code/not-code boundary.

### Phase 5 rules-first pivot (2026-05-16, PHASE-5-PARAMETERIZATION.2.2.1)

Implementation finding that corrects the `.1` design's instantiation
assumption. The `.1`/`.2.1` plan was: build modules normally, then a
post-construction pass marks the width-homogeneous ones parameterized.
A 64-seed forced-on sweep (`width_parameterization_prob = 1.0`,
single width, `constant_prob = 0`, `max_depth = 1`) produced **zero**
width-homogeneous modules: the unconstrained cone generator almost
always introduces a constant, a comparison, a mux, a slice/concat, or
mixed operand widths. Two consequences:

1. **Inert.** A parameterization that only fires when the RNG happens
   to emit a homogeneous module would essentially never fire on real
   output — a feature that cannot trigger is not a capability.
2. **Doctrine violation.** "Generate, then keep the ones that happen to
   qualify" is precisely the generate-then-filter anti-pattern ANVIL
   forbids (valid/structured *by construction*, not by post-hoc
   selection).

**Decision:** keep the `is_width_generic` gate (it is correct and
cheap) but demote it to a post-construction *assertion*, and add a
**rules-first parameterizable-leaf constructor** (`.2.2.2`): when the
knob fires for a module, *construct* it width-homogeneously by rule
(one design width; only width-preserving same-width gates; no
`Constant`/`Slice`/`Concat`/`ForFold`/`Mux`/compare), valid by
construction. The gate then always accepts it. Rejected alternative:
"loosen the gate to parameterize partially-homogeneous modules" —
rejected because a module mixing `[W-1:0]` and `[7:0]` logic that must
agree in width is unsound when `W ≠ 8`; partial parameterization
re-introduces exactly the multi-width unsoundness `.1` set out to
avoid. This does not change architecture (C) (still post-construction
annotation + monomorphic body); it changes *how the body is built* so
the sound subset is reached by rule instead of by luck.

### Phase 5 parameterization design (2026-05-16, PHASE-5-PARAMETERIZATION.1)

Design-only slice. No code. Lifts `book/src/ir.md` "Parameters and
generics (Phase 5)" into a concrete, codebase-grounded implementation +
parameter-aware-identity plan, with rejected alternatives and a proof
shape, so the implementation leaf (`.2`) has an unambiguous target.

**Goal (from ROADMAP Phase 5).** Emitted modules carry `parameter`
declarations for widths; instances pick parameter values from allowed
ranges and override via `#(.W(value))`; parameter-dependent widths
propagate correctly; parameter-aware identity stays sound (distinct
parameter values must not alias to one `NodeId` or one module template
unless genuinely equivalent). Default-off; never retire existing
behaviour.

**Code reality that constrains the design** (audited; key anchors):
width is a bare `u32` everywhere — `Port.width`, `Node::*` width fields,
`Flop.width`, the `intern_gate`/`intern_constant` CSE keys
(`src/ir/types.rs`), the per-op width arithmetic in
`input_widths_for` / `make_width_adapter` (`src/gen/cone.rs`), the
gate-shape + design child-width equality rules (`src/ir/validate.rs`),
the single `width_decl` rendering chokepoint and the parameterless
module header / instance emission (`src/emit/sv.rs`), and the
width-hashing in `canonical_module_signature` (`src/metrics.rs:2187`)
that `src/ir/dedup.rs` groups on. Constant folding/peephole in
`intern_gate`, `make_width_adapter`, `input_widths_for`, `ForFold`
(`trip_count*chunk_width`) and `Slice` (`hi`/`lo` are themselves bare
indices) do **genuine integer arithmetic** and cannot run on opaque
symbolic widths. `shrink_primary_inputs_to_live_width`
(`src/gen/module.rs`) actively rewrites port widths post-construction.

**Architectural decision — chosen: (C) post-construction
parameterization pass + monomorphic instantiation.** Phase 5 lands as a
*post-finalisation pass* (sibling in spirit to the module-dedup pass),
not as a symbolic type threaded through construction:

1. The cone/module is constructed exactly as today, at a concrete
   "design" width `W0` drawn (reproducibly, via `g.rng`) from the new
   parameter's allowed range. All existing fold/validate/cse machinery
   runs unchanged on concrete `u32` — **valid-by-construction is
   preserved with zero changes to the invasive width-arithmetic code**.
2. A post-construction pass marks a *sound parameterizable subset* of
   widths as symbolic in `W`: the interface port widths chosen to carry
   the parameter, plus exactly those internal node widths that the
   construction-time width relations make **affine in `W0`** and that
   stay legal for the whole declared `W` range. Widths that enter
   structurally-constrained integer math (`ForFold trip_count*chunk`,
   `Slice hi/lo`, replicate counts in `make_width_adapter`,
   constant-fold masks) are **excluded from the parameterized set in the
   first slice** — they keep concrete `u32`. The pass records a
   per-module `ParamEnv { name: "W", range: CountRange, design_value:
   W0 }` and a lightweight `WidthExpr` (small enum
   `{ Lit(u32), Param }`, deliberately *not* the full
   `Add/Mul/Clog2/...` algebra yet — see rejected (B)) only on the
   parameterized width sites, each also retaining its resolved `u32`.
3. Instantiation (`src/gen/hierarchy.rs`, between child selection and
   the input-binding loop) picks a value from the param range via
   `g.rng`, records it in a new `Instance.param_bindings`, and binds
   child ports at the **resolved** width so the existing exact-equality
   child-width validation still holds.
4. Emitter: `width_decl` and the module header learn the symbolic form
   (`logic [W-1:0]`, `#( parameter int W = W0 )`); instance emission
   gains `#(.W(value))`. Everywhere a width is *not* in the
   parameterized set, emission is byte-identical to today.

Soundness rule: a module is only emitted parameterized when its chosen
parameterized widths remain legal (validator-clean, downstream-clean)
for **every** value in the declared range — guaranteed by restricting
the parameterized set to affine-in-`W` interface/derived widths and by
the matrix gate sweeping ≥2 values per parameterized scenario. This is
construction-time soundness (a generator rule), not generate-then-filter.

**Parameter-aware identity rule.** The single place width enters module
identity is the per-port/per-node `fnv1a_64_u32(h, width)` calls in
`canonical_module_signature` (`src/metrics.rs`). The rule:
parameterized width sites hash their **normalized symbolic form**
(`WidthExpr::Param` → a fixed sentinel, not `W0`); non-parameterized
sites hash their concrete `u32` as today. Consequence: two
instantiations / monomorphic emissions of the *same template* at W=8
and W=16 produce the **same** signature (legitimately one template — the
existing `dedup_modules` then collapses them with no change to
`dedup.rs`); a genuinely concrete width-7 module still hashes distinctly
and never aliases a parameterized one. `Instance.param_bindings` is
*not* hashed into the parent signature (consistent with the existing
exclusion of `Instance.module`/`name`), so a parent that instantiates
one template at several values keeps one child template — which is the
entire point of parameterization. This extends the doctrine "NodeId =
identity of an expression" / "ModuleId = identity of a hierarchical
module template" to "a parameterized template is one identity across its
legal parameter range".

**Rejected alternatives.**
- **(A) Monomorphize only, emit a symbolic header over a fixed body.**
  Pick `W0`, build the body at `W0`, emit `parameter W=W0` + `[W-1:0]`
  but never make the body width-generic. Rejected: the emitted module is
  a *lie* — overriding `#(.W(16))` on a body built for `W0=8` is not
  valid-by-construction (it would only be correct at `W==W0`). It would
  also force generate-then-filter to avoid bad overrides. Violates the
  by-construction and no-post-hoc-repair doctrines.
- **(B) Full symbolic `WidthExpr{Add,Sub,Mul,Div,Clog2,Max,Min}`
  threaded through the IR from construction.** The book's eventual
  target. Rejected *as the first slice*: it propagates through every
  invasive site in §6 of the audit (all constant folding/peephole, the
  width adapter, `input_widths_for`, `ForFold`, symbolic `Slice`
  indices) — constant folding cannot operate on symbolic widths at all,
  so the e-graph/factorization doctrine would have to be suspended for
  parameterized cones. Too large for one signoff-quality slice and
  high-risk to the existing proven surface. Recorded as the **Phase 5
  follow-on** once (C) is downstream-clean: (C)'s `WidthExpr{Lit,Param}`
  is deliberately the minimal seed of (B)'s algebra, so (B) is a strict
  extension, not a rework.
- **(C') Symbolic widths but disable factorization for parameterized
  modules.** Rejected: silently weakening `identity_mode = node-id`
  for a whole class of modules is exactly the kind of silent
  mode-retirement the project forbids.

**Proof shape for `.2`.** (1) Focused proof: a parameterized module is
emitted with `parameter W` and instantiated at ≥2 distinct in-range
values via `#(.W(v))`; `ir::validate::validate_design` passes; the
emitted SV elaborates/synthesizes clean at each value. (2) Identity
proof: same template at W=8 and W=16 → one `canonical_module_signature`
(and `dedup_modules` collapses them); a concrete non-parameterized
module of width 8 keeps a distinct signature (extends the existing
`dedup_is_a_no_op_when_modules_are_structurally_distinct` test). (3)
Matrix gate: new opt-in knob `width_parameterization_prob` (f64, default
`0.0`, serde-default pattern like `hierarchy_module_dedup`), a
`phase5_*` focus config sweeping the param range, a new
`saw_width_parameterized_design` coverage fact gated under a new
`ScenarioSet::Phase5` (or folded into the Phase 4 design set initially),
proven downstream-clean (Verilator + both Yosys modes) with
`coverage_gaps=[]`. Default-off keeps every existing scenario
byte-identical.

**Open questions (do not block `.2`; recorded for it).**
- Whether Phase 5 gets its own `ScenarioSet::Phase5` gate or rides the
  Phase 4 design harness for the first slice. Lean: ride Phase 4
  harness first (cheaper), split when the parameterized matrix grows.
- Whether `.2` should be split (IR+emit scaffold → instantiation
  substitution → identity rule → matrix gate) — likely yes; `.2` will
  be re-decomposed in the tree when reached.
- Multi-parameter modules and parameter-dependent *depth/count* (not
  just width) are explicitly out of the first slice (ROADMAP notes
  parameter-aware child selection / parameter-driven parent generation
  remain later Phase 5 work).

### Module-dedup pass implemented (2026-05-15, r87, HIERARCHY-AWARE-IDENTITY.4 + .5)
The dedup pass design sketched in `HIERARCHY-AWARE-IDENTITY.3` is now
live as `src/ir/dedup.rs`. Implementation matches the sketch
exactly: pipeline placement (post-finalisation, called from
`Generator::generate_design`), instance-rewrite policy (fixed-point
iteration with lexicographic-smallest-name survivor, top always
preserved by name), toggle/API choice (new `Config::hierarchy_module_dedup:
bool`, default `false`, orthogonal to `IdentityMode`). The
canonical-signature hash is reused from `src/metrics.rs` (exposed as
`pub(crate)` for that purpose — single source of truth).

**Validation evidence:** r87 gate downstream-clean at 210 scenarios /
840 designs / `coverage_gaps = []`. The new
`phase4_hier1_module_dedup_active` matrix scenario per construction
strategy proves dedup runs cleanly through Verilator and both Yosys
modes; the earlier `phase4_hier1_structurally_duplicate_modules`
scenario remains in the bank with dedup off, providing the
side-by-side before/after comparison.

**`HIERARCHY-AWARE-IDENTITY` tree status:** complete. All five leaves
(`.1` canonical signatures, `.2` existence proof, `.3` design sketch,
`.4` implementation, `.5` matrix gate proof) are `done`. The doctrine
extension — "ModuleId = identity of a hierarchical module template"
— is now live under the opt-in `Config::hierarchy_module_dedup`
knob.

### Module-dedup pass design sketch (2026-05-15, HIERARCHY-AWARE-IDENTITY.3)
This is the pre-implementation design sketch for the eventual
`H-A-I.4` dedup pass. No code lands in this slice.

**Pre-conditions established by earlier slices.**

- `H-A-I.1` (r85) gives every `Module` a deterministic 64-bit FNV-1a
  canonical signature exposed as
  `DesignMetrics.canonical_module_signatures`. The signature covers
  port shape, node sequence, drive structure, flop structure, and
  instance interfaces but intentionally excludes `instance.module`
  and `instance.name`. Two structurally-identical Modules with
  distinctly-named children therefore share a signature.
- `H-A-I.2` (r86) proves the planner can emit structurally-duplicate
  Modules under tight 1-in/1-out/width-1 / `max_depth=1` /
  `terminal_reuse_prob=1.0` leaf constraints. The dedup pass has a
  live exercise.

**Pass goal.** Given a finished `Design`, collapse every group of
Modules in `design.modules` that share a canonical signature to a
single surviving entry, and rewrite every `Instance.module` reference
in the remaining Modules so they point at the surviving canonical
peer. Default behaviour stays identical to today; the pass is opt-in.

**Pipeline placement.**

- **Chosen placement:** post-finalisation, after the existing
  per-module `compact_node_ids` pass and right before `Design` is
  returned from `generate_design`. The post-finalisation point is
  the only point where every Module's canonical structure is settled
  (every gate has been compacted, every flop merge has run, every
  `intern_*` retry has completed) — running before then would dedup
  Modules that are not yet in their canonical form.
- **Module location:** new `src/ir/dedup.rs`, alongside
  `src/ir/compact.rs`. Separate file because the operation is
  Design-level (cross-Module), not Module-level (per-Module compaction).
- **Rejected alternative — incremental dedup during construction:**
  i.e., dedup each Module against the existing pool as soon as it's
  emitted. Rejected because (a) ANVIL's planner emits parents
  bottom-up, so dedup at emission time would dedup leaves before
  their children's instances are wired, breaking the
  instance-rewrite contract; and (b) it couples the dedup pass to
  the generator's emission ordering, making future planner changes
  hostile to dedup.
- **Rejected alternative — dedup as an emitter pass in
  `src/emit/sv.rs`:** rejected because emitter doctrine is
  "dumb serialiser, no transformation". Rule 21 / dumb-emitter
  doctrine forbids semantic transformations during emit.

**Instance-rewrite policy.**

1. Compute signatures and group Modules by signature.
2. Within each group, pick the **canonical survivor** as the one
   with the lexicographically-smallest `Module.name`. Deterministic
   tiebreaker for stable output.
3. Build a `name_remap: HashMap<String, String>` from
   merged-away → survivor.
4. Walk every surviving Module's `instances` list; for each
   `Instance.module`, replace with `name_remap.get(...).unwrap_or(self)`.
5. Drop the merged-away Modules from `design.modules`.
6. **Iterate to fixed point.** After one pass, second-level parents
   may now have IDENTICAL instance-graph shapes (because their
   leaves were deduped to a common name). Re-run the pass; new
   duplicates may emerge. Repeat until a pass produces no merges.
   Bottom-up dedup order is the result of fixed-point iteration,
   not an explicit traversal — simpler and provably correct.

**Edge cases.**

- **Top module.** The Design's top must NEVER be merged away. The
  canonical-survivor pick must skip the top, or equivalently always
  pick the top when it appears in a group. Practical implementation:
  exclude the top from the grouping step.
- **Empty design / single-Module design.** No work to do; pass
  returns the Design unchanged. No special-case needed; the grouping
  produces no groups with `count > 1`.
- **Library-mode duplicates.** When `hierarchy_child_source_mode =
  library`, the planner already reuses one Module definition across
  multiple instance slots — so the signature-collision rate at
  library mode is already 0 by construction (the duplicates that
  *would* exist are folded into the library's single definition
  before dedup runs). Dedup is a no-op for library mode unless the
  planner's library construction itself emits structural twins,
  which `H-A-I.2` shows is rare. Most dedup benefit will come from
  on-demand mode.
- **Cycles in instance graph.** Cannot happen — `Design::modules`
  forms a strict DAG (top depends on children depend on
  grandchildren). The fixed-point iteration terminates because
  each iteration reduces `design.modules.len()` strictly, bounded
  below by 1 (the top).
- **Mismatched instance counts after a merged-away module had
  different child references than the survivor.** Cannot happen if
  the signature excludes child-module names but INCLUDES the
  instance interface structure (`role`, `inputs` shape). My current
  signature does both — see `canonical_module_signature` in
  `src/metrics.rs`. So two Modules sharing a signature have the
  same number of instances with the same input wiring; only the
  child names differ, and the rewrite handles that.

**Toggle and API.**

- **Chosen toggle:** a new `Config` knob
  `hierarchy_module_dedup: bool`, default `false`. Plain bool rather
  than an enum variant because the operation is binary (do dedup /
  don't). Future extensions (e.g., dedup-with-aggressive-merging
  beyond canonical signature) would warrant an enum.
- **Rejected alternative — extend `IdentityMode` with a new
  `HierarchicalNodeId` variant.** Rejected because `IdentityMode`
  governs *gate-level* expression identity; extending it to also
  control module-level identity overloads the enum's meaning. The
  existing `IdentityMode::NodeId` doctrine ("NodeId = identity of an
  expression") stands unchanged; the module-level analogue is a
  separate concern and gets its own knob. (`feedback_never_retire_strategies`
  applies: don't retire `IdentityMode::NodeId`, don't silently
  redefine it.)
- **Rejected alternative — extend `FactorizationLevel` ladder with
  a `module-dedup` rung.** Rejected for the same reason: the ladder
  is about gate-level factorization strength, not hierarchy-level
  identity. Dedup at the Module level is orthogonal.

**Proof shape for `H-A-I.4`.**

- **Focused proof:** build a 4-leaf design under the
  `H-A-I.2` tight-leaf config. Compute metrics without dedup:
  `num_modules = 5`, `num_distinct_module_signatures = 2`,
  `num_structurally_duplicate_module_pairs = 6`. Run dedup. Re-compute
  metrics: `num_modules = 2` (top + the surviving leaf),
  `num_distinct = 2`, `num_pairs = 0`. Validate the resulting Design
  via `validate_design` to ensure no broken instance references.
- **Matrix scenario:** mirror `H-A-I.2`'s tight-leaf scenario but
  with the dedup toggle on. New saw fact
  `saw_design_with_module_dedup_active` requires `num_pairs == 0`
  AND `num_modules` strictly less than what `H-A-I.2`'s peer
  scenario emits. Both scenarios stay in the bank so the
  before/after comparison is visible.
- **Default-off preservation:** the existing `H-A-I.2` scenario
  (`phase4_hier1_structurally_duplicate_modules`) must continue to
  produce `num_structurally_duplicate_module_pairs > 0` after
  `H-A-I.4` lands, proving the toggle defaults off.

**Open questions for `H-A-I.4` implementation.**

- Should dedup also remove unused Modules (modules that no Instance
  in the surviving Module set references)? The existing
  `num_unused_module_definitions` metric flags this — the dedup
  pass could opportunistically clean up, OR a separate
  `prune_unused_modules` pass could be a sibling slice. Likely the
  latter (single responsibility).
- Should the survivor's name be re-emitted (e.g., `mod_42_merged`)
  to make the dedup visible in the SV output? Or keep the
  lexicographically-smallest original name? Default-keep is
  cheaper; explicit re-emit is more debuggable.
- Should we emit a manifest entry recording which Modules were
  deduped onto which survivors? Useful for downstream tools that
  want to back-trace; trivial to add via a new
  `DesignMetrics.dedup_remap: BTreeMap<String, String>`.

**Slice budget for `H-A-I.4`.** Implementation should fit in one
slice: ~50 lines in `src/ir/dedup.rs`, ~20 lines wiring the toggle
in `Config`, ~30 lines of focused proof, ~15 lines of matrix
scenario + saw fact. No new dependency on external crates.

## Workflow notes
### Task-tree ownership is mandatory for all code changes (2026-05-17, owner directive)

**Doctrine, non-negotiable, no compromise.** It is strictly forbidden
to make any code change without it being task-tree tracked or
task-tree owned **first**. This supersedes the earlier "task trees are
opt-in per top-level task" / "stay on `rN` for linear coverage" scope:
that softer framing no longer governs code.

**Why:** the owner observed that task-tree ownership improved code
review and code quality *tremendously* over the ad-hoc / linear-`rN`
cadence — the recursive breakdown, explicit frontier, recorded
decisions/blockers, and the 1:1 leaf↔commit mapping force each change
to be scoped, justified, and reviewable before it lands, and make
pause/resume recovery lossless. The empirical improvement, not a
process preference, is the rationale.

**Boundary.** "Code" = anything that changes program/generator
behaviour or generated RTL (`src/`, `tests/`, `examples/`,
build/codegen logic, behaviour-altering `Cargo` manifests). Pure-docs
/ live-doc / mdBook / workflow-config edits and recording doctrine
itself are *not* code changes and need no tree (this very entry is an
example). `rN` is **not** retired — it survives only as the optional
within-leaf slice cadence *inside* a tree; a bare unowned `rN` code
slice is no longer legal.

**Mechanics.** Before editing code, confirm/create the owning leaf
(`docs/tasks/<TREE>.md` + `docs/TASK_TREE.md` row); leaf ID in the
commit subject; one completed leaf per commit; the frontier names the
next eligible leaf. Recorded across `COMMIT.md`, `docs/TASK_TREE.md`
("ANVIL Adoption Scope"), `SESSION_BOOTSTRAP.md`, this file,
`README.md`, and the mdBook (`architecture.md`); session memory
`feedback_task_tree_available.md`. Keep all in sync if the policy
ever changes.

### Coverage baseline established (2026-05-14, COVERAGE-INSTRUMENTATION.1)
cargo-llvm-cov 0.8.7 + llvm-tools-aarch64-apple-darwin already
installed locally. Baseline run via `cargo llvm-cov --release`
(intentionally excludes the 75-min Phase 4 hierarchy matrix gate so
the baseline stays reproducible in minutes). Result: **85.26% lines,
91.95% functions, 87.61% regions** across 14 crate files. Full
per-file breakdown lives in `docs/coverage-baseline.md`.

**Key signal:** the planner core (`gen/hierarchy.rs`, `gen/module.rs`,
`gen/cone.rs`, `ir/compact.rs`, `emit/sv.rs`) sits at 88-99% lines
*without* the matrix gate's 204 scenarios contributing. That confirms
the focused-proof + unit-test combination already exercises the
construction discipline comprehensively, not just at the macro
(matrix-gate) level. `metrics.rs` is at 99.66% — meaning the
detection helpers (`binding_uses_*`, canonical-signature hash,
ratio computations) are very densely tested by the focused proofs
the recent rN slices added.

**Top-5 under-covered files (for `.2` triage):**

1. `bin/tool_matrix.rs` — 1951 lines, 72.07%. Matrix-gate-only paths.
2. `gen/cone.rs` — 454 lines, 88.65%. The only planner-core file
   outside the 95%+ band; likely anti-collapse rollback paths.
3. `ir/validate.rs` — 254 lines, 75.07%. Mostly defensive panics
   ("this case cannot happen" invariants); expected.
4. `config.rs` — 250 lines, 67.87%. CLI overlay variants.
5. `main.rs` — 142 lines, 60.56%. Clap derives + flag plumbing.

`.2` produces a disposition matrix per file: (a) dead code -> remove,
(b) rarely-fired path -> add focused proof, (c) defensive
unreachable -> leave and document.

### Registered three quality-improvement task trees (2026-05-14)
Added active task trees for the three quality dials discussed in
the session that prompted task-tree adoption itself:

- `INSTA-SNAPSHOTS` — `insta`-backed snapshot tests of generator
  output, enforcing the "byte-identical forever" reproducibility
  contract directly. Currently provable only by intent.
- `DIFFERENTIAL-SIMULATION` — cross-simulator semantic equivalence
  (Verilator + iverilog at minimum). Raises the downstream contract
  from "parses and synthesises" to "all observers agree on semantics".
- `COVERAGE-INSTRUMENTATION` — `cargo-llvm-cov`-backed coverage
  reports converting matrix-comprehensiveness from intent to
  measurement.

**Rationale.** ANVIL already does the rarest hard thing right:
validity by construction. The remaining quality dial is *consistency
across observers* — different simulators, different runs, different
platforms, different code paths. Each tree owns one orthogonal axis
of that dial; together they cover the "signoff-level random RTL"
ambition stated in `README.md` along its three reachable directions.

**Sequencing intent.** No leaf is `in_progress`. When the user opens
a quality slice, the natural order is INSTA-SNAPSHOTS.1 (cheapest,
nothing else depends on it), then COVERAGE-INSTRUMENTATION.1 (medium
cost, exposes planner test gaps), then DIFFERENTIAL-SIMULATION.1
(highest cost but highest signoff payoff). The user picks; the trees
just make the scope durable.

**Rejected alternative.** Folding all three into a single
`SIGNOFF-QUALITY` umbrella tree. Rejected: the three are
operationally independent (one can ship without the others), and
collapsing them would hide which axis is being worked on at any
given moment.

### Adopted FSMGen task-tree workflow on ANVIL (2026-05-14)
Added a repo-local task-tree tracking workflow at `docs/TASK_TREE.md`
plus the portable setup guide at `docs/TASK_TREE_README.md` (lifted from
FSMGen's `docs/TASK_TREE_README.md`). One initial active tree:
`docs/tasks/HIERARCHY-AWARE-IDENTITY.md`, covering the hierarchy-aware
identity work that r85 opened.

**Scope decision:** task trees are opt-in per top-level task on ANVIL,
not mandatory. Linear `rN` coverage slices (r73-r82 depth sweeps, r83
three-stage chain, r84 helper budget 5) already had clean handoff under
the `rN` + `CHANGES.md` + `MEMORY.md` combination — adding leaf-IDs and
per-leaf task files there would mostly add overhead without solving a
real problem. The value of task-tree is highest where the work has:
more than ~3 planned sub-slices, real blockers or design decisions to
record, parallel sub-axes that do not fit a single linear `rN` ladder,
or is likely to span multiple sessions with pause/resume cycles. The
upcoming hierarchy-aware-identity dedup work fits all four; the closed
depth sweeps fit none.

**Rejected alternative:** full FSMGen-style mandate ("all work is
task-tree-managed by default"). FSMGen's ISF lane has that policy
because every ISF objective has multiple independent dimensions; ANVIL's
linear `rN` shape does not.

**Commit-workflow tie-in:** `COMMIT.md` gained a "Task-tree-managed
commits" section requiring the leaf ID in commit subjects when work is
task-tree-managed, and same-commit updates to the owning
`docs/tasks/<TREE>.md` file. Non-task-tree commits (linear `rN`,
isolated doc edits) follow the standard checklist without the
leaf-ID rule.

## Calibration notes
### Phase 4 r86 proves the planner can emit structurally-duplicate Modules downstream-clean (HIERARCHY-AWARE-IDENTITY.2)
The latest full downstream-clean Phase 4 hierarchy evidence anchor is
now `/tmp/anvil-tool-matrix-phase4-hierarchy-r86/tool_matrix_report.json`:
207 scenarios / 828 designs, `coverage_gaps = []`, Verilator/Yosys
all 828/0. Closes leaf `HIERARCHY-AWARE-IDENTITY.2`.

**Calibration discovery.** Initial 500-config sweep (varying
num_leaf_modules, num_child_instances, seed, strategy with default
leaf-input/output ranges) produced **zero** structurally-duplicate
Module pairs. The leaf generator's RNG advances between calls, so two
leaves with the same interface profile but different RNG states
produce different gate structures by default.

**Calibration choice.** Tight 1-input / 1-output / width-1 leaves with
`max_depth = 1` and `terminal_reuse_prob = 1.0` collapse the leaf
generator's degrees of freedom: there's essentially one legal
"drive output from the lone input" structure. Under these
constraints, every library leaf hashes to the same canonical
signature, so a depth-1 wrapper with 4 leaves produces a
4*(4-1)/2 = 6 duplicate-pair design.

**Implication for `H-A-I.4` (dedup pass).** Dedup is therefore real
and applicable to ANVIL's planner. The dedup pass will need to:
(a) merge Module definitions sharing a canonical signature, and
(b) remap every `Instance.module` string in the rest of the design
to point at the surviving merged definition. Both passes are
straightforward over `Design::modules`. The opt-in toggle is left
to `H-A-I.4` for the design sketch.

### Phase 4 r85 lands canonical module signatures as the first slice of hierarchy-aware identity downstream-clean
The latest full downstream-clean Phase 4 hierarchy evidence anchor is now
`/tmp/anvil-tool-matrix-phase4-hierarchy-r85/tool_matrix_report.json`:
204 scenarios / 816 designs, `coverage_gaps = []`, Verilator/Yosys all
816/0. PNT-3 of the autonomous-PNT chain. Each module gets a
dependency-free FNV-1a 64-bit signature covering port shape, node
sequence, drive structure, flop structure, and instance interfaces. The
hash deliberately omits `instance.module` and `instance.name` so two
parents that instantiate distinctly-named-but-identically-shaped
children share a signature — that isomorphism awareness is what makes
the signature useful for future `Design::modules` deduplication.
Calibration: depth 2, 4,4 child instances,
`hierarchy_child_input_cone_prob = 1.0`, no helpers, no flops, no
sibling routing — a vanilla recursive hierarchy that produces multiple
distinct module shapes so the diversity fact (`num_distinct >= 2`)
fires reliably.

### Phase 4 r84 proves a recursive non-top internal parent can saturate a parent-cone helper budget of 5 helpers downstream-clean
The latest full downstream-clean Phase 4 hierarchy evidence anchor is now
`/tmp/anvil-tool-matrix-phase4-hierarchy-r84/tool_matrix_report.json`:
201 scenarios / 804 designs, `coverage_gaps = []`, Verilator/Yosys all
804/0. Second slice of the broader-Phase-4 work (PNT-2 of the
autonomous-PNT chain). Extends the helper-budget axis from 3 (previous
saturating proof) to 5. Calibration: depth 2, 4,4 child instances,
`max_parent_cone_instances_per_module = 5`,
`hierarchy_child_input_cone_prob = 1.0`, and
`hierarchy_parent_cone_instance_prob = 1.0`. Each non-top internal
parent has ~4 children x ~2 inputs = 8 child-input decision sites,
giving the planner enough demand to fully saturate the budget-5
allocation per parent.

### Phase 4 r83 proves recursive non-top registered parent-composed three-stage chain downstream-clean
The latest full downstream-clean Phase 4 hierarchy evidence anchor is now
`/tmp/anvil-tool-matrix-phase4-hierarchy-r83/tool_matrix_report.json`:
198 scenarios / 792 designs, `coverage_gaps = []`, Verilator/Yosys all
792/0. First slice of the broader-Phase-4 work after the depth-7 sweep
closed in r82. Promotes a new chain-depth axis on top of the closed
depth-3..7 sweeps: registered parent-composed child-input bindings can
chain through three parent-local flop stages without helper instances
below the top parent. Calibration: depth 3, 4,4 child instances,
`max_flops_per_module = 128`, `max_depth = 8`. These limits give the
planner enough flop budget and cone depth to naturally produce
chain-length-3 structures below the top across all four construction
strategies; the planner has no explicit chain-length knob, so the new
detection just walks the existing FlopQ -> D chain three deep and
counts bindings whose Q's D is a non-slice/non-concat gate over both
instance outputs and another Q.

### Phase 4 r82 closes the depth-7 sweep with recursive non-top stateful unregistered parent-composed mixed-support child inputs without helpers downstream-clean (2,2 calibrated)
The latest full downstream-clean Phase 4 hierarchy evidence anchor is now
`/tmp/anvil-tool-matrix-phase4-hierarchy-r82/tool_matrix_report.json`:
195 scenarios / 780 designs, `coverage_gaps = []`, Verilator/Yosys all
780/0. Fifth and final slice of the depth-7 sweep mirroring
r77/r72/r67/r62 — closes the depth-7 axis. Calibration: depth-7 stateful
mixed-support cells use the same 2,2 child-instance bounds as r77 at
depth 6 and r79 at depth 7 (mixed-support cells at depths ≥ 6 use 2,2
because the 4,4 tree at depth 7 would yield ~5461 internal occurrences,
far beyond a safe-slice budget). The depth-7 axis is now fully closed:
all five cells (parent-flops r78, mixed-support child inputs r79,
parent-port-composed outputs r80, stateful parent-port-composed outputs
r81, stateful mixed-support child inputs r82) are first-class
downstream-clean coverage facts.

### Phase 4 r81 extended the depth-7 axis with recursive non-top stateful parent-port-composed parent outputs without helpers downstream-clean
The latest full downstream-clean Phase 4 hierarchy evidence anchor is now
`/tmp/anvil-tool-matrix-phase4-hierarchy-r81/tool_matrix_report.json`:
192 scenarios / 768 designs, `coverage_gaps = []`, Verilator/Yosys all
768/0. Fourth slice of the depth-7 sweep mirroring r76/r71/r66/r61.
Only one cell remained to close depth-7: stateful mixed-support child
inputs (r82, with the same 2,2 calibration as r74/r77/r79).

### Phase 4 r80 extended the depth-7 axis with recursive non-top parent-port-composed parent outputs without helpers or parent-local state downstream-clean
The latest full downstream-clean Phase 4 hierarchy evidence anchor is now
`/tmp/anvil-tool-matrix-phase4-hierarchy-r80/tool_matrix_report.json`:
189 scenarios / 756 designs, `coverage_gaps = []`, Verilator/Yosys all
756/0. Third slice of the depth-7 sweep mirroring r75/r70/r65/r60.
Parent-port-composed cells already use 2,2 children at all depths so no
calibration drift here.

### Phase 4 r79 extended the depth-7 axis with recursive non-top mixed-support child inputs without helpers downstream-clean (2,2 calibrated)
The latest full downstream-clean Phase 4 hierarchy evidence anchor is now
`/tmp/anvil-tool-matrix-phase4-hierarchy-r79/tool_matrix_report.json`. It
keeps the live hierarchy policy at four designs per scenario and expands
it to 186 scenarios / 744 designs, with `coverage_gaps = []`,
`artifact_kind = "design"`, Verilator `744/0`, Yosys without-ABC
`744/0`, and Yosys with-ABC `744/0`.

This bank extends the depth-7 axis (opened by r78 with parent flops) to
the unregistered parent-composed mixed-support child-input surface,
mirroring r74 (depth 6), r69 (depth 5), r64 (depth 4), and r59
(depth 3). Smoke at depth 7 with 2,2 child instances confirmed 127
internal module occurrences with `child_input_bindings_from_parent_composed_logic = 219`
versus 1 top-only and `child_input_bindings_from_mixed_support = 173`
versus 1 top-only.

**Calibration:** depth-7 mixed-support cells continue the 2,2
child-instance calibration introduced at depth 6. The 4,4 tree at
depth 7 would yield ~5461 internal occurrences, far beyond a safe-slice
budget for downstream-clean tools. 2,2 at depth 7 still proves the
mixed-support surface cleanly.

The slice does not change the generator: it tightens the gate around an
already-supported capability.

Current-code validation includes the focused recursive pipeline
regression, `cargo test --bin tool_matrix`, and the full r79 Phase 4
hierarchy gate through Verilator plus both repo-owned Yosys modes.

### Phase 4 r78 opened the depth-7 axis with recursive non-top parent-local flops downstream-clean
The latest full downstream-clean Phase 4 hierarchy evidence anchor is now
`/tmp/anvil-tool-matrix-phase4-hierarchy-r78/tool_matrix_report.json`. It
keeps the live hierarchy policy at four designs per scenario and expands
it to 183 scenarios / 732 designs, with `coverage_gaps = []`,
`artifact_kind = "design"`, Verilator `732/0`, Yosys without-ABC
`732/0`, and Yosys with-ABC `732/0`.

This bank opens the depth-7 axis, mirroring how r73 opened depth-6
above the closed depth-5 sweep, r68 opened depth-5 above the closed
depth-4 sweep, and r63 opened depth-4 above the closed depth-3 sweep.
Smoke at depth 7 with 2,2 child instances confirmed 127 non-top
internal-parent occurrences with `hierarchy_parent_local_flops = 8122`
versus `top_local_flops = 64` and 127 internal occurrences carrying
parent-local flops.

The depth-6 sweep closed in r77 with all five mixed-support cells gated
as first-class facts; r78 now starts the depth-7 sweep with the
simplest surface — parent flops at depth 7 — as a foothold. Future
r79..r82 will close the depth-7 sweep mirroring r58..r62 (depth 3),
r63..r67 (depth 4), r68..r72 (depth 5), and r73..r77 (depth 6).
Mixed-support cells at depth 7 will adopt the 2,2 child-instance
calibration introduced at depth 6.

The slice does not change the generator: it tightens the gate around an
already-supported capability.

Current-code validation includes the focused recursive pipeline
regression, `cargo test --bin tool_matrix`, and the full r78 Phase 4
hierarchy gate through Verilator plus both repo-owned Yosys modes.

### Phase 4 r77 closed the depth-6 sweep with recursive non-top stateful unregistered parent-composed mixed-support child inputs without helpers downstream-clean (2,2 calibrated)
The latest full downstream-clean Phase 4 hierarchy evidence anchor is now
`/tmp/anvil-tool-matrix-phase4-hierarchy-r77/tool_matrix_report.json`. It
keeps the live hierarchy policy at four designs per scenario and expands
it to 180 scenarios / 720 designs, with `coverage_gaps = []`,
`artifact_kind = "design"`, Verilator `720/0`, Yosys without-ABC
`720/0`, and Yosys with-ABC `720/0`.

This bank closes the depth-6 sweep. r73 opened the depth-6 axis with
parent flops, r74 extended with mixed-support child inputs (2,2
calibrated), r75 with parent-port-composed parent outputs, r76 with
stateful parent-port-composed parent outputs. r77 closes the sweep
with stateful unregistered parent-composed mixed-support child inputs,
mirroring r72 (depth 5), r67 (depth 4), and r62 (depth 3).

**Calibration follow-on:** depth-6 stateful mixed-support cells use the
same 2,2 child-instance calibration adopted by r74. Smoke confirmed 63
internal module occurrences with `hierarchy_parent_local_flops = 4032`
versus `top_local_flops = 64`,
`child_input_bindings_from_stateful_parent_composed_mixed_support = 74`
versus 1 top-only, and
`stateful_parent_composed_mixed_support_child_input_binding_fraction
= 0.454`.

The depth-6 axis now has all five mixed-support cells gated as
first-class coverage facts, mirroring closed depth-3 (r58..r62),
depth-4 (r63..r67), and depth-5 (r68..r72) sweeps. The Phase 4 depth
sweep template is now consistent across depths 3-6.

The slice does not change the generator: it tightens the gate around an
already-supported capability.

Current-code validation includes the focused recursive pipeline
regression, `cargo test --bin tool_matrix`, and the full r77 Phase 4
hierarchy gate through Verilator plus both repo-owned Yosys modes.

### Phase 4 r76 extended the depth-6 axis with recursive non-top stateful parent-port-composed parent outputs without helpers downstream-clean
The latest full downstream-clean Phase 4 hierarchy evidence anchor is now
`/tmp/anvil-tool-matrix-phase4-hierarchy-r76/tool_matrix_report.json`. It
keeps the live hierarchy policy at four designs per scenario and expands
it to 177 scenarios / 708 designs, with `coverage_gaps = []`,
`artifact_kind = "design"`, Verilator `708/0`, Yosys without-ABC
`708/0`, and Yosys with-ABC `708/0`.

This bank extends the depth-6 axis (opened by r73 with parent flops,
extended by r74 with mixed-support child inputs and r75 with
parent-port-composed parent outputs) to the stateful
parent-port-composed parent-output surface, mirroring r71 (depth 5),
r66 (depth 4), and r61 (depth 3). Smoke at depth 6 with 2,2 child
instances confirmed 63 internal module occurrences with
`hierarchy_parent_local_flops = 4028` versus `top_local_flops = 64`,
`hierarchy_parent_port_composed_outputs = 960` versus 160 top-only,
`hierarchy_parent_port_composed_outputs_through_parent_flops = 890`
versus 109 top-only, and
`hierarchy_parent_port_composed_parent_flop_output_fraction = 0.927`.

Only one cell remains to close the depth-6 sweep: stateful unregistered
parent-composed mixed-support child inputs (r77).

The slice does not change the generator: it tightens the gate around an
already-supported capability.

Current-code validation includes the focused recursive pipeline
regression, `cargo test --bin tool_matrix`, and the full r76 Phase 4
hierarchy gate through Verilator plus both repo-owned Yosys modes.

### Phase 4 r75 extended the depth-6 axis with recursive non-top parent-port-composed parent outputs without helpers or parent-local state downstream-clean
The latest full downstream-clean Phase 4 hierarchy evidence anchor is now
`/tmp/anvil-tool-matrix-phase4-hierarchy-r75/tool_matrix_report.json`. It
keeps the live hierarchy policy at four designs per scenario and expands
it to 174 scenarios / 696 designs, with `coverage_gaps = []`,
`artifact_kind = "design"`, Verilator `696/0`, Yosys without-ABC
`696/0`, and Yosys with-ABC `696/0`.

This bank extends the depth-6 axis (opened by r73 with parent flops and
extended by r74 with mixed-support child inputs) to the unregistered
parent-port-composed parent-output surface, mirroring r70 (depth 5),
r65 (depth 4), and r60 (depth 3). Smoke confirmed 63 internal module
occurrences with `hierarchy_parent_port_composed_outputs = 1008` versus
`top_parent_port_composed_outputs = 168` and a
`hierarchy_parent_port_composed_output_fraction = 1.0` at depth 6 with
2,2 child-instance bounds. No calibration drift — parent-port-composed
cells use 2,2 at all depths.

The slice does not change the generator: it tightens the gate around an
already-supported capability.

Current-code validation includes the focused recursive pipeline
regression, `cargo test --bin tool_matrix`, and the full r75 Phase 4
hierarchy gate through Verilator plus both repo-owned Yosys modes.

### Phase 4 r74 extended the depth-6 axis with recursive non-top mixed-support child inputs without helpers downstream-clean
The latest full downstream-clean Phase 4 hierarchy evidence anchor is now
`/tmp/anvil-tool-matrix-phase4-hierarchy-r74/tool_matrix_report.json`. It
keeps the live hierarchy policy at four designs per scenario and expands
it to 171 scenarios / 684 designs, with `coverage_gaps = []`,
`artifact_kind = "design"`, Verilator `684/0`, Yosys without-ABC
`684/0`, and Yosys with-ABC `684/0`.

This bank extends the depth-6 axis (opened by r73 with parent flops) to
the unregistered parent-composed mixed-support child-input surface,
mirroring how r69 followed r68 at depth 5, r64 followed r63 at depth 4,
and r59 followed r58 at depth 3.

**Calibration: depth-6 mixed-support cells use 2,2 child-instance
bounds, not the 4,4 used at depths 3-5.** Smoke at depth 6 with 4,4
showed 1365 internal module occurrences (4× the d5 count of 341);
yosys-with-abc spent 22+ minutes on a single design, projecting to ~10h
per gate. That exceeds a safe-slice budget for a 10-step batch. The 2,2
calibration at depth 6 yields 63 occurrences (matching r73's
parent-flop scenario) and proves the same surface cleanly: focused
proof passes in 0.42s release. This is a slice-time calibration choice,
not a strategy retirement — the 4,4 mixed-support cells at d3-d5 remain
unchanged. r77 (stateful mixed-support at d6) will adopt the same 2,2
calibration. If a future workstream wants a downstream-clean d6 4,4
mixed-support proof, that can land as a separate slice with a longer
budget.

The slice does not change the generator: it tightens the gate around an
already-supported capability.

Current-code validation includes the focused recursive pipeline
regression, `cargo test --bin tool_matrix`, and the full r74 Phase 4
hierarchy gate through Verilator plus both repo-owned Yosys modes.

### Phase 4 r73 opened the depth-6 axis with recursive non-top parent-local flops downstream-clean
The latest full downstream-clean Phase 4 hierarchy evidence anchor is now
`/tmp/anvil-tool-matrix-phase4-hierarchy-r73/tool_matrix_report.json`. It
keeps the live hierarchy policy at four designs per scenario and expands
it to 168 scenarios / 672 designs, with `coverage_gaps = []`,
`artifact_kind = "design"`, Verilator `672/0`, Yosys without-ABC
`672/0`, and Yosys with-ABC `672/0`.

This bank opens the depth-6 axis, mirroring how r68 opened depth-5
above the closed depth-4 sweep, and r63 opened depth-4 above the closed
depth-3 sweep. Smoke at depth 6 with 2,2 child instances confirmed 63
non-top internal-parent occurrences with `hierarchy_parent_local_flops
= 4028` versus `top_local_flops = 64` and 63 internal occurrences
carrying parent-local flops.

The depth-5 sweep closed in r72 with all five mixed-support cells gated
as first-class facts; r73 now starts the depth-6 sweep with the
simplest surface — parent flops at depth 6 — as a foothold. Future
r74..r77 slices will close the depth-6 sweep mirroring r58..r62 (depth
3), r63..r67 (depth 4), and r68..r72 (depth 5).

The slice does not change the generator: it tightens the gate around an
already-supported capability.

Current-code validation includes the focused recursive pipeline
regression, `cargo test --bin tool_matrix`, and the full r73 Phase 4
hierarchy gate through Verilator plus both repo-owned Yosys modes.

### Phase 4 r72 closed the depth-5 sweep with recursive non-top stateful unregistered parent-composed mixed-support child inputs without helpers downstream-clean
The latest full downstream-clean Phase 4 hierarchy evidence anchor is now
`/tmp/anvil-tool-matrix-phase4-hierarchy-r72/tool_matrix_report.json`. It
keeps the live hierarchy policy at four designs per scenario and expands
it to 165 scenarios / 660 designs, with `coverage_gaps = []`,
`artifact_kind = "design"`, Verilator `660/0`, Yosys without-ABC
`660/0`, and Yosys with-ABC `660/0`.

This bank closes the depth-5 sweep. r68 opened the depth-5 axis with
parent flops, r69 extended it with mixed-support child inputs, r70 with
parent-port-composed parent outputs, r71 with stateful parent-port-composed
parent outputs. r72 closes the sweep with stateful unregistered
parent-composed mixed-support child inputs, mirroring how r67 closed
depth 4 and r62 closed depth 3. Smoke confirmed 341 internal module
occurrences with `hierarchy_parent_local_flops = 21820` versus
`top_local_flops = 64`, `child_input_bindings_from_parent_composed_logic
= 1777` versus 3 top-only, `child_input_bindings_from_stateful_parent_composed_mixed_support
= 1460` versus 2 top-only, and
`stateful_parent_composed_mixed_support_child_input_binding_fraction
= 0.642` at depth 5 with `4,4` child-instance bounds.

The depth-5 axis now has all five mixed-support cells gated as
first-class coverage facts, mirroring the closed depth-3 (r58..r62) and
depth-4 (r63..r67) sweeps. Future Phase 4 work can pursue depth 6 or
broaden the registered-helper / multi-helper surface.

The slice does not change the generator: it tightens the gate around an
already-supported capability.

Current-code validation includes the focused recursive pipeline
regression, `cargo test --bin tool_matrix`, and the full r72 Phase 4
hierarchy gate through Verilator plus both repo-owned Yosys modes.

### Phase 4 r71 extended the depth-5 axis with recursive non-top stateful parent-port-composed parent outputs without helpers downstream-clean
The previous full downstream-clean Phase 4 hierarchy evidence anchor is
`/tmp/anvil-tool-matrix-phase4-hierarchy-r71/tool_matrix_report.json`. It
keeps the live hierarchy policy at four designs per scenario and expands
it to 162 scenarios / 648 designs, with `coverage_gaps = []`,
`artifact_kind = "design"`, Verilator `648/0`, Yosys without-ABC
`648/0`, and Yosys with-ABC `648/0`.

This bank extends the depth-5 axis (opened by r68 with parent flops,
extended by r69 with mixed-support child inputs, and extended by r70
with parent-port-composed parent outputs) to the stateful
parent-port-composed parent-output surface, mirroring how r66 followed
r65 at depth 4 and r61 followed r60 at depth 3. Smoke confirmed 31
internal module occurrences with `hierarchy_parent_local_flops = 1980`
versus `top_local_flops = 64`, `hierarchy_parent_port_composed_outputs
= 340` versus 68 top-only, `hierarchy_parent_port_composed_outputs_through_parent_flops
= 336` versus 64 top-only, and `hierarchy_parent_port_composed_parent_flop_output_fraction
= 0.988` at depth 5 with `2,2` child-instance bounds.

Only one cell remains to close the depth-5 sweep: stateful unregistered
parent-composed mixed-support child inputs (depth-3 territory r62 /
depth-4 territory r67).

The slice does not change the generator: it tightens the gate around an
already-supported capability.

Current-code validation includes the focused recursive pipeline
regression, `cargo test --bin tool_matrix`, and the full r71 Phase 4
hierarchy gate through Verilator plus both repo-owned Yosys modes.

### Phase 4 r70 extended the depth-5 axis with recursive non-top parent-port-composed parent outputs without helpers or parent-local state downstream-clean
The previous full downstream-clean Phase 4 hierarchy evidence anchor is
`/tmp/anvil-tool-matrix-phase4-hierarchy-r70/tool_matrix_report.json`. It
keeps the live hierarchy policy at four designs per scenario and expands
it to 159 scenarios / 636 designs, with `coverage_gaps = []`,
`artifact_kind = "design"`, Verilator `636/0`, Yosys without-ABC
`636/0`, and Yosys with-ABC `636/0`.

This bank extends the depth-5 axis (opened by r68 with parent flops and
extended by r69 with mixed-support child inputs) to the unregistered
parent-port-composed parent-output surface, mirroring how r65 followed
r64 at depth 4 and r60 followed r59 at depth 3. Smoke confirmed 31
internal module occurrences with `hierarchy_parent_port_composed_outputs
= 390` versus `top_parent_port_composed_outputs = 78` and a
`hierarchy_parent_port_composed_output_fraction = 1.0` at depth 5 with
`2,2` child-instance bounds.

The slice does not change the generator: it tightens the gate around an
already-supported capability.

Current-code validation includes the focused recursive pipeline
regression, `cargo test --bin tool_matrix`, and the full r70 Phase 4
hierarchy gate through Verilator plus both repo-owned Yosys modes.

### Phase 4 r69 extended the depth-5 axis with recursive non-top mixed-support child inputs without helpers downstream-clean
The previous full downstream-clean Phase 4 hierarchy evidence anchor is
`/tmp/anvil-tool-matrix-phase4-hierarchy-r69/tool_matrix_report.json`. It
keeps the live hierarchy policy at four designs per scenario and expands
it to 156 scenarios / 624 designs, with `coverage_gaps = []`,
`artifact_kind = "design"`, Verilator `624/0`, Yosys without-ABC
`624/0`, and Yosys with-ABC `624/0`.

This bank extends the depth-5 axis (opened by r68) to the unregistered
parent-composed mixed-support child-input surface, mirroring how r64
followed r63 at depth 4 and r59 followed r58 at depth 3. Smoke confirmed
341 internal module occurrences with 1457 hierarchy-wide vs 3 top-only
mixed-support bindings and 1599 vs 3 parent-composed bindings at
depth 5.

The slice does not change the generator: it tightens the gate around an
already-supported capability.

Current-code validation includes the focused recursive pipeline
regression, `cargo test --bin tool_matrix`, and the full r69 Phase 4
hierarchy gate through Verilator plus both repo-owned Yosys modes.

### Phase 4 r68 opened the depth-5 axis with recursive non-top parent-local flops downstream-clean
The previous full downstream-clean Phase 4 hierarchy evidence anchor is
`/tmp/anvil-tool-matrix-phase4-hierarchy-r68/tool_matrix_report.json`. It
keeps the live hierarchy policy at four designs per scenario and expands
it to 153 scenarios / 612 designs, with `coverage_gaps = []`,
`artifact_kind = "design"`, Verilator `612/0`, Yosys without-ABC
`612/0`, and Yosys with-ABC `612/0`.

This bank opens the depth-5 axis. The depth-4 sweep was structurally
complete in r67 (all five mixed-support cells covered: parent-flops,
no-state and stateful child-input mixed-support, no-state and stateful
parent-output mixed-support). r68 starts the depth-5 axis with the
simplest surface — parent flops at depth 5 — by adding
`saw_recursive_hierarchy_depth_5_parent_local_flops` (coverage gap when
missing) plus the focused proof
`recursive_hierarchy_parents_can_emit_local_flops_at_depth_5` and the
matrix scenario `phase4_recur_d5_parent_state` per construction strategy
(`2,2` child-instance bounds, four intermediate parent layers below the
top). Smoke at depth 5 confirmed 31 internal module occurrences with
1984 hierarchy-wide parent-local flops versus 64 top-only, so the
recursive generator handles depth-5 nesting cleanly.

The slice does not change the generator: it tightens the gate around an
already-supported capability.

Current-code validation includes the focused recursive pipeline
regression, `cargo test --bin tool_matrix`, and the full r68 Phase 4
hierarchy gate through Verilator plus both repo-owned Yosys modes.

### Phase 4 r67 closed the depth-4 sweep with recursive non-top stateful parent-composed mixed-support child inputs without helpers downstream-clean
The previous full downstream-clean Phase 4 hierarchy evidence anchor is
`/tmp/anvil-tool-matrix-phase4-hierarchy-r67/tool_matrix_report.json`. It
keeps the live hierarchy policy at four designs per scenario and expands
it to 150 scenarios / 600 designs, with `coverage_gaps = []`,
`artifact_kind = "design"`, Verilator `600/0`, Yosys without-ABC
`600/0`, and Yosys with-ABC `600/0`.

This bank closes the depth-4 sweep, mirroring how r62 closed the depth-3
sweep. The depth-4 axis now covers parent-flops (r63), no-state
mixed-support child inputs (r64), no-state parent-port-composed outputs
(r65), stateful parent-port-composed outputs (r66), and stateful
unregistered parent-composed mixed-support child inputs (r67). The new
`saw_recursive_hierarchy_depth_4_stateful_parent_composed_mixed_support_child_inputs`
fact (coverage gap when missing) plus the focused proof
`recursive_hierarchy_stateful_parent_composed_routes_mix_parent_ports_at_depth_4_without_helpers`
and the matrix scenario `phase4_recur_d4_stateful_parent_composed_mixed_support_child_input`
per construction strategy isolate the surface across three intermediate
parent layers below the top. Smoke confirmed 85 internal module
occurrences with 471 hierarchy-wide vs 3 top-only
stateful-parent-composed-mixed-support bindings and 5438 vs 64
parent-local flops at depth 4.

The slice does not change the generator: it tightens the gate around an
already-supported capability.

Current-code validation includes the focused recursive pipeline
regression, `cargo test --bin tool_matrix`, and the full r67 Phase 4
hierarchy gate through Verilator plus both repo-owned Yosys modes.

### Phase 4 r66 extended the depth-4 axis with recursive non-top stateful parent-port-composed parent outputs without helpers downstream-clean
The previous full downstream-clean Phase 4 hierarchy evidence anchor is
`/tmp/anvil-tool-matrix-phase4-hierarchy-r66/tool_matrix_report.json`. It
keeps the live hierarchy policy at four designs per scenario and expands
it to 147 scenarios / 588 designs, with `coverage_gaps = []`,
`artifact_kind = "design"`, Verilator `588/0`, Yosys without-ABC
`588/0`, and Yosys with-ABC `588/0`.

This bank extends the depth-4 axis (r63 parent-flops, r64 mixed-support
child inputs, r65 no-state parent-port-composed outputs) to the
stateful parent-port-composed parent-output surface, mirroring how r61
followed r60 at depth 3. The new
`saw_recursive_hierarchy_depth_4_stateful_parent_port_composed_outputs`
fact (coverage gap when missing) plus the focused proof
`recursive_hierarchy_stateful_parent_outputs_mix_parent_ports_at_depth_4_without_helpers`
and the matrix scenario `phase4_recur_d4_stateful_parent_port_composed_output`
per construction strategy isolate the surface across three intermediate
parent layers below the top. Smoke confirmed 15 internal module
occurrences with 128 hierarchy-wide vs 32 top-only
parent-port-composed-through-flops outputs and 960 vs 64 parent-local
flops at depth 4.

The slice does not change the generator: it tightens the gate around an
already-supported capability.

Current-code validation includes the focused recursive pipeline
regression, `cargo test --bin tool_matrix`, and the full r66 Phase 4
hierarchy gate through Verilator plus both repo-owned Yosys modes.

### Phase 4 r65 extended the depth-4 axis with recursive non-top parent-port-composed parent outputs without helpers or state downstream-clean
The previous full downstream-clean Phase 4 hierarchy evidence anchor is
`/tmp/anvil-tool-matrix-phase4-hierarchy-r65/tool_matrix_report.json`. It
keeps the live hierarchy policy at four designs per scenario and expands
it to 144 scenarios / 576 designs, with `coverage_gaps = []`,
`artifact_kind = "design"`, Verilator `576/0`, Yosys without-ABC
`576/0`, and Yosys with-ABC `576/0`.

This bank extends the depth-4 axis (r63 parent-flops, r64 mixed-support
child inputs) to the parent-port-composed parent-output surface,
mirroring how r60 followed r59 at depth 3. The new
`saw_recursive_hierarchy_depth_4_parent_port_composed_outputs` fact
(coverage gap when missing) plus the focused proof
`recursive_hierarchy_parent_outputs_mix_parent_ports_at_depth_4_without_helpers`
and the matrix scenario `phase4_recur_d4_parent_port_composed_output`
per construction strategy isolate the surface across three intermediate
parent layers below the top. Smoke confirmed 15 internal module
occurrences with 176 hierarchy-wide vs 44 top-only parent-port-composed
outputs at depth 4.

The slice does not change the generator: it tightens the gate around an
already-supported capability.

Current-code validation includes the focused recursive pipeline
regression, `cargo test --bin tool_matrix`, and the full r65 Phase 4
hierarchy gate through Verilator plus both repo-owned Yosys modes.

### Phase 4 r64 extended the depth-4 axis with recursive non-top mixed-support child inputs without helpers downstream-clean
The previous full downstream-clean Phase 4 hierarchy evidence anchor is
`/tmp/anvil-tool-matrix-phase4-hierarchy-r64/tool_matrix_report.json`. It
keeps the live hierarchy policy at four designs per scenario and expands
it to 141 scenarios / 564 designs, with `coverage_gaps = []`,
`artifact_kind = "design"`, Verilator `564/0`, Yosys without-ABC
`564/0`, and Yosys with-ABC `564/0`.

This bank extends the depth-4 axis (opened by r63) to the unregistered
parent-composed mixed-support child-input surface, mirroring how r59
followed r58 at depth 3. The new
`saw_recursive_hierarchy_depth_4_mixed_support_child_inputs` fact
(coverage gap when missing) plus the focused proof
`recursive_hierarchy_parent_composed_routes_mix_parent_ports_at_depth_4_without_helpers`
and the matrix scenario `phase4_recur_d4_parent_composed_mixed_support_child_input`
per construction strategy isolate the surface across three intermediate
parent layers below the top. Smoke confirmed 85 internal module
occurrences with 315 hierarchy-wide vs 3 top-only mixed-support
bindings and 355 vs 3 parent-composed bindings at depth 4.

The slice does not change the generator: it tightens the gate around an
already-supported capability.

Current-code validation includes the focused recursive pipeline
regression, `cargo test --bin tool_matrix`, and the full r64 Phase 4
hierarchy gate through Verilator plus both repo-owned Yosys modes.

### Phase 4 r63 opened the depth-4 axis with recursive non-top parent-local flops downstream-clean
The previous full downstream-clean Phase 4 hierarchy evidence anchor is
`/tmp/anvil-tool-matrix-phase4-hierarchy-r63/tool_matrix_report.json`. It
keeps the live hierarchy policy at four designs per scenario and expands
it to 138 scenarios / 552 designs, with `coverage_gaps = []`,
`artifact_kind = "design"`, Verilator `552/0`, Yosys without-ABC
`552/0`, and Yosys with-ABC `552/0`.

This bank opens the depth-4 axis. The depth-3 push was structurally
complete in r62 (all four mixed-support cells covered at depth 3:
parent-flops, no-state child-input, no-state parent-output, stateful
parent-output, stateful child-input). r63 starts the depth-4 axis with
the simplest surface — parent flops at depth 4 — by adding
`saw_recursive_hierarchy_depth_4_parent_local_flops` (coverage gap when
missing) plus the focused proof
`recursive_hierarchy_parents_can_emit_local_flops_at_depth_4` and the
matrix scenario `phase4_recur_d4_parent_state` per construction strategy
(`2,2` child-instance bounds, three intermediate parent layers below
the top). The smoke run at depth 4 confirmed 15 internal module
occurrences with 960 hierarchy-wide parent-local flops versus 64
top-only, so the recursive generator handles depth-4 nesting cleanly.

The slice does not change the generator: it tightens the gate around an
already-supported capability. No new metric is needed.

Current-code validation includes the focused recursive pipeline
regression, `cargo test --bin tool_matrix`, and the full r63 Phase 4
hierarchy gate through Verilator plus both repo-owned Yosys modes.

### Phase 4 r62 closed the depth-3 push by gating recursive non-top stateful parent-composed mixed-support child inputs without helpers downstream-clean
The previous full downstream-clean Phase 4 hierarchy evidence anchor is
`/tmp/anvil-tool-matrix-phase4-hierarchy-r62/tool_matrix_report.json`. It
keeps the live hierarchy policy at four designs per scenario and expands
it to 135 scenarios / 540 designs, with `coverage_gaps = []`,
`artifact_kind = "design"`, Verilator `540/0`, Yosys without-ABC
`540/0`, and Yosys with-ABC `540/0`.

This bank closes the final symmetric cell of the depth-3 push. The
sweep has now covered parent-flops (r58), no-state mixed-support child
inputs (r59), no-state parent-port-composed outputs (r60), stateful
parent-port-composed outputs (r61), and now stateful unregistered
parent-composed mixed-support child inputs (r62). r62 adds
`saw_recursive_hierarchy_depth_3_stateful_parent_composed_mixed_support_child_inputs`
(coverage gap when missing) plus the focused proof
`recursive_hierarchy_stateful_parent_composed_routes_mix_parent_ports_at_depth_3_without_helpers`
and the matrix scenario `phase4_recur_d3_stateful_parent_composed_mixed_support_child_input`
per construction strategy. The smoke run at depth 3 confirmed 21
internal module occurrences with 129 hierarchy-wide
stateful-parent-composed-mixed-support bindings versus 3 top-only and
1344 vs 64 parent-local flops, so the recursive generator handles
depth-3 stateful child-input mixed-support cleanly.

The slice does not change the generator: it tightens the gate around an
already-supported capability. No new metric is needed because the
`child_input_bindings_from_stateful_parent_composed_mixed_support`
counter added in r56 already populates correctly at depth 3.

Current-code validation includes the focused recursive pipeline
regression, `cargo test --bin tool_matrix`, and the full r62 Phase 4
hierarchy gate through Verilator plus both repo-owned Yosys modes.

### Phase 4 r61 pushed recursive non-top stateful parent-port-composed parent outputs to exact hierarchy depth 3 without helpers downstream-clean
The previous full downstream-clean Phase 4 hierarchy evidence anchor is
`/tmp/anvil-tool-matrix-phase4-hierarchy-r61/tool_matrix_report.json`. It
keeps the live hierarchy policy at four designs per scenario and expands
it to 132 scenarios / 528 designs, with `coverage_gaps = []`,
`artifact_kind = "design"`, Verilator `528/0`, Yosys without-ABC
`528/0`, and Yosys with-ABC `528/0`.

This bank closes the last symmetric gap in the depth-3 push. r58/r59/r60
covered parent-flops, mixed-support child inputs, and no-state
parent-port-composed outputs at depth 3. r61 adds the stateful version
of the parent-output surface (r55's depth-2 territory) at depth 3 by
adding `saw_recursive_hierarchy_depth_3_stateful_parent_port_composed_outputs`
(coverage gap when missing) plus the focused proof
`recursive_hierarchy_stateful_parent_outputs_mix_parent_ports_at_depth_3_without_helpers`
and the matrix scenario `phase4_recur_d3_stateful_parent_port_composed_output`
per construction strategy. The smoke run at depth 3 confirmed 7 internal
module occurrences with 36 hierarchy-wide parent-port-composed outputs
through parent-local Qs versus 12 top-only and 448 vs 64 parent-local
flops, so the recursive generator handles depth-3 stateful parent-output
composition cleanly.

The slice does not change the generator: it tightens the gate around an
already-supported capability. No new metric is needed because the
through-parent-flop output counters added in r55 already populate
correctly at depth 3.

Current-code validation includes the focused recursive pipeline
regression, `cargo test --bin tool_matrix`, and the full r61 Phase 4
hierarchy gate through Verilator plus both repo-owned Yosys modes.

### Phase 4 r60 pushed recursive non-top parent-port-composed parent outputs to exact hierarchy depth 3 without helpers or state downstream-clean
The previous full downstream-clean Phase 4 hierarchy evidence anchor is
`/tmp/anvil-tool-matrix-phase4-hierarchy-r60/tool_matrix_report.json`. It
keeps the live hierarchy policy at four designs per scenario and expands
it to 129 scenarios / 516 designs, with `coverage_gaps = []`,
`artifact_kind = "design"`, Verilator `516/0`, Yosys without-ABC
`516/0`, and Yosys with-ABC `516/0`.

This bank closes the remaining symmetry gap in the depth-3 push.
r58 took parent-flops to depth 3 and r59 took unregistered
parent-composed mixed-support child inputs to depth 3, but the
parent-output cone surface (r54's depth-2 territory) had no
exact-depth-3 focused proof. r60 closes that gap by adding
`saw_recursive_hierarchy_depth_3_parent_port_composed_outputs`
(coverage gap when missing) plus the focused proof
`recursive_hierarchy_parent_outputs_mix_parent_ports_at_depth_3_without_helpers`
and the matrix scenario `phase4_recur_d3_parent_port_composed_output`
per construction strategy. The scenario uses `2,2` child-instance bounds
(matching r58's depth-3 parent-state shape but with parent flops off and
the parent-output cone surface as the only active route). The smoke run
at depth 3 confirmed 7 internal module occurrences with 72 hierarchy-wide
parent-port-composed outputs versus 24 top-only, so the recursive
generator handles depth-3 parent-output composition cleanly.

The slice does not change the generator: it tightens the gate around an
already-supported capability. No new metric is needed because
`hierarchy_parent_composed_outputs`, `top_parent_composed_outputs`,
`hierarchy_parent_port_composed_outputs`, `top_parent_port_composed_outputs`,
and `realized_max_leaf_depth` are already populated.

Current-code validation includes the focused recursive pipeline
regression, `cargo test --bin tool_matrix`, and the full r60 Phase 4
hierarchy gate through Verilator plus both repo-owned Yosys modes.

### Phase 4 r59 pushed recursive non-top mixed-support child inputs to exact hierarchy depth 3 without helpers downstream-clean
The previous full downstream-clean Phase 4 hierarchy evidence anchor is
`/tmp/anvil-tool-matrix-phase4-hierarchy-r59/tool_matrix_report.json`. It
keeps the live hierarchy policy at four designs per scenario and expands
it to 126 scenarios / 504 designs, with `coverage_gaps = []`,
`artifact_kind = "design"`, Verilator `504/0`, Yosys without-ABC
`504/0`, and Yosys with-ABC `504/0`.

This bank pushes the unregistered parent-composed mixed-support
child-input surface from exact depth 2 (r53) to exact depth 3. r58
already pushed the parent-flop surface to depth 3 but left the
mixed-support child-input surface depth-bound at 2. r59 closes that
asymmetry by adding `saw_recursive_hierarchy_depth_3_mixed_support_child_inputs`
(coverage gap when missing) plus the focused proof
`recursive_hierarchy_parent_composed_routes_mix_parent_ports_at_depth_3_without_helpers`
and the matrix scenario `phase4_recur_d3_parent_composed_mixed_support_child_input`
per construction strategy. The scenario uses `4,4` child-instance bounds
(distinct from r58's depth-3 / `2,2` parent-state shape) to broaden the
depth-3 evidence across different design shapes. The smoke run at
depth 3 confirmed 21 internal module occurrences with 115 hierarchy-wide
mixed-support bindings versus 3 top-only, so the recursive generator
handles depth-3 mixed-support routing cleanly.

The slice does not change the generator: it tightens the gate around an
already-supported capability. No new metric is needed because
`child_input_bindings_from_parent_composed_logic`,
`child_input_bindings_from_mixed_support`, the corresponding top
counters, and `realized_max_leaf_depth` are already populated.

Current-code validation includes the focused recursive pipeline
regression, `cargo test --bin tool_matrix`, and the full r59 Phase 4
hierarchy gate through Verilator plus both repo-owned Yosys modes.

### Phase 4 r58 pushed recursive parent-local flops to exact hierarchy depth 3 downstream-clean
The previous full downstream-clean Phase 4 hierarchy evidence anchor is
`/tmp/anvil-tool-matrix-phase4-hierarchy-r58/tool_matrix_report.json`. It
keeps the live hierarchy policy at four designs per scenario and expands
it to 123 scenarios / 492 designs, with `coverage_gaps = []`,
`artifact_kind = "design"`, Verilator `492/0`, Yosys without-ABC
`492/0`, and Yosys with-ABC `492/0`.

This bank pushes the parent-state surface from exact depth 2 to exact
depth 3. All r51-r57 focused proofs use depth 2 (one layer of internal
parents below the top). The mixed-range `2:3` scenario already produces
depth-3 designs sometimes, but no focused proof asserts the parent-state
surface fires AT depth 3 specifically. r58 closes that asymmetry by
adding `saw_recursive_hierarchy_depth_3_parent_local_flops` (coverage
gap when missing) plus the focused proof
`recursive_hierarchy_parents_can_emit_local_flops_at_depth_3` and the
matrix scenario `phase4_recur_d3_parent_state` per construction strategy
(2,2 child-instance bounds, distinct from r57's depth-2 / 4,4 shape).
The smoke run at depth 3 confirmed 7 internal module occurrences and
448 parent-local flops with `top_local_flops = 64`, so the recursive
generator handles depth-3 nesting cleanly.

The slice does not change the generator: it tightens the gate around an
already-supported capability. No new metric is needed because
`realized_max_leaf_depth`, `hierarchy_parent_local_flops`,
`top_local_flops`, and `internal_module_occurrences_with_local_flops`
are already populated.

Current-code validation includes the focused recursive pipeline
regression, `cargo test --bin tool_matrix`, and the full r58 Phase 4
hierarchy gate through Verilator plus both repo-owned Yosys modes.

### Phase 4 r57 gated recursive non-top parent-local flops as first-class coverage downstream-clean
The previous full downstream-clean Phase 4 hierarchy evidence anchor is
`/tmp/anvil-tool-matrix-phase4-hierarchy-r57/tool_matrix_report.json`. It
keeps the live hierarchy policy at four designs per scenario and expands
it to 120 scenarios / 480 designs, with `coverage_gaps = []`,
`artifact_kind = "design"`, Verilator `480/0`, Yosys without-ABC
`480/0`, and Yosys with-ABC `480/0`.

This bank promotes recursive non-top parent-local flops to first-class
gated coverage. r55 and r56 already evidenced non-top parent-local flops
as a side-channel of their richer mixed-support assertions, but the gate
did not enforce the parent-flop surface below the top parent on its own.
A regression that broke parent-flop emission specifically for non-top
parents could therefore have slipped past the existing matrix. r57
closes that gap by adding `saw_recursive_hierarchy_parent_local_flops`
(coverage gap when missing) plus a dedicated focused proof
`recursive_hierarchy_parents_can_emit_local_flops_below_top` that
isolates the parent-flop surface by disabling helpers, sibling routing,
registered routing, and parent-composed child-input cones. The new
matrix scenario `phase4_recur_d2_parent_state` uses `4,4` child-instance
bounds (distinct from r55's `2,2`) so the parent-state surface has its
own labeled focus point in the matrix rather than relying on
side-channel evidence from richer scenarios.

The slice does not change the generator: it tightens the gate around an
already-supported capability. No new metric is needed because
`hierarchy_parent_local_flops`, `top_local_flops`,
`internal_module_occurrences_with_local_flops`, and
`realized_max_leaf_depth` are already populated.

Current-code validation includes the focused recursive pipeline
regression, `cargo test --bin tool_matrix`, and the full r57 Phase 4
hierarchy gate through Verilator plus both repo-owned Yosys modes.

### Phase 4 r56 proved recursive stateful no-helper parent-composed mixed-support child inputs downstream-clean
The previous full downstream-clean Phase 4 hierarchy evidence anchor is
`/tmp/anvil-tool-matrix-phase4-hierarchy-r56/tool_matrix_report.json`. It
keeps the live hierarchy policy at four designs per scenario and expands
it to 117 scenarios / 468 designs, with `coverage_gaps = []`,
`artifact_kind = "design"`, Verilator `468/0`, Yosys without-ABC
`468/0`, and Yosys with-ABC `468/0`.

This bank adds the child-input sibling of the r55 parent-output proof. r53
proved recursive non-top unregistered parent-composed mixed-support child
inputs in a stateless setup; r56 keeps the same no-helper, no-registered
shape but turns on parent-local flops and requires the new
`child_input_bindings_from_stateful_parent_composed_mixed_support` counter
to exceed top-only below the top parent. That proves a non-top parent's
unregistered parent-composed child-input cone can simultaneously source
parent ports, child outputs, and parent-local Qs without using helper
instances or registered routing.

The new metric is computed at the existing parent-composed child-input
binding site by intersecting the binding's dep set across `has_ports`,
`has_instance_outputs`, and `has_flop_virtuals`. No new IR construct
appears in the generator path: the slice exposes a stricter cell of the
existing parent-composed mixed-support surface.

Current-code validation includes the focused recursive pipeline
regression, `cargo test --bin tool_matrix`, and the full r56 Phase 4
hierarchy gate through Verilator plus both repo-owned Yosys modes.

### Phase 4 r55 proved recursive stateful no-helper parent-port-composed outputs downstream-clean
The previous full downstream-clean Phase 4 hierarchy evidence anchor is
`/tmp/anvil-tool-matrix-phase4-hierarchy-r55/tool_matrix_report.json`. It
keeps the live hierarchy policy at four designs per scenario and expands
it to 114 scenarios / 456 designs, with `coverage_gaps = []`,
`artifact_kind = "design"`, Verilator `456/0`, Yosys without-ABC
`456/0`, and Yosys with-ABC `456/0`.

This bank adds the stateful sibling of the r54 parent-output proof. The
focused exact-depth-2 lane disables helper instances, direct sibling
routing, registered sibling routing, and child-input parent-cone routes,
then enables parent-local flops and requires hierarchy-wide
parent-port-composed parent-output counters through parent-local Qs to
exceed their top-only counterparts. That proves recursive non-top parent
outputs can mix parent data ports, child outputs, and parent-local Qs
without using helper instances.

Current-code validation includes the focused recursive pipeline
regression, `cargo test --bin tool_matrix`, and the full r55 Phase 4
hierarchy gate through Verilator plus both repo-owned Yosys modes.

### Phase 4 r54 proved recursive no-helper parent-port-composed outputs downstream-clean
The previous full downstream-clean Phase 4 hierarchy evidence anchor is
`/tmp/anvil-tool-matrix-phase4-hierarchy-r54/tool_matrix_report.json`. It
keeps the live hierarchy policy at four designs per scenario and expands
it to 111 scenarios / 444 designs, with `coverage_gaps = []`,
`artifact_kind = "design"`, Verilator `444/0`, Yosys without-ABC
`444/0`, and Yosys with-ABC `444/0`.

This bank adds a focused exact-depth-2 recursive parent-output proof below
the top parent. The focused lane disables helper instances, parent-local
flops, direct sibling routing, registered sibling routing, and child-input
parent-cone routes, then requires hierarchy-wide parent-port-composed
parent-output counters to exceed their top-only counterparts. That makes
recursive non-top parent outputs that mix parent data ports with child
outputs a first-class coverage fact instead of inferring the case from the
older top-parent parent-output evidence.

Current-code validation includes the focused recursive pipeline
regression, `cargo test --bin tool_matrix`, and the full r54 Phase 4
hierarchy gate through Verilator plus both repo-owned Yosys modes.

### Phase 4 r53 proved recursive no-helper parent-composed mixed support downstream-clean
The previous full downstream-clean Phase 4 hierarchy evidence anchor is
`/tmp/anvil-tool-matrix-phase4-hierarchy-r53/tool_matrix_report.json`. It
kept the live hierarchy policy at four designs per scenario and expanded
it to 108 scenarios / 432 designs, with `coverage_gaps = []`,
`artifact_kind = "design"`, Verilator `432/0`, Yosys without-ABC
`432/0`, and Yosys with-ABC `432/0`.

This bank adds an ordinary unregistered parent-composed child-input
mixed-support proof below the top parent. No-helper child-input cones now
promote their root when needed so the same parent-composed binding can
carry both parent data-port support and sibling child-output support.
The focused lane disables direct sibling routes, registered child-input
routes, helper instances, and parent-local flops, so the proof stays in
the unregistered parent-composed bucket instead of being classified as a
helper-backed or registered route.

Current-code validation includes the focused recursive pipeline
regression, `cargo test --bin tool_matrix`, and the full r53 Phase 4
hierarchy gate through Verilator plus both repo-owned Yosys modes.

### Phase 4 r52 proves recursive direct registered sibling mixed support downstream-clean
The latest full downstream-clean Phase 4 hierarchy evidence anchor is now
`/tmp/anvil-tool-matrix-phase4-hierarchy-r52/tool_matrix_report.json`. It
keeps the live hierarchy policy at four designs per scenario and expands
it to 105 scenarios / 420 designs, with `coverage_gaps = []`,
`artifact_kind = "design"`, Verilator `420/0`, Yosys without-ABC
`420/0`, and Yosys with-ABC `420/0`.

This bank does not need a new generator path. The r51 direct registered
sibling mixed-support route is generated by the same parent-generation
logic below the top parent, so r52 adds a focused exact-depth-2 recursive
scenario and a stricter coverage fact that requires hierarchy-wide
registered sibling mixed-support counters to exceed the top-only
counters. The focused lane disables registered parent-composed and
helper-instance sources, so the recursive proof stays classified as
non-top direct registered sibling routing rather than registered
parent-composed or helper-backed D-cone routing.

Current-code validation includes the focused recursive pipeline
regression, `cargo test --bin tool_matrix`, and the full r52 Phase 4
hierarchy gate through Verilator plus both repo-owned Yosys modes.

### Phase 4 r51 adds direct registered sibling mixed support downstream-clean
The previous direct registered sibling mixed-support Phase 4 hierarchy evidence anchor is
`/tmp/anvil-tool-matrix-phase4-hierarchy-r51/tool_matrix_report.json`. It
keeps the live hierarchy policy at four designs per scenario and expands
it to 102 scenarios / 408 designs, with `coverage_gaps = []`,
`artifact_kind = "design"`, Verilator `408/0`, Yosys without-ABC
`408/0`, and Yosys with-ABC `408/0`.

This bank adds the default-off
`hierarchy_registered_sibling_mixed_support_prob` route. When a direct
registered sibling D source has instance-output support but lacks parent
ports, the route may mix in one compatible parent data-port companion
before the parent-local flop. The mixed D expression is wrapped before
registration so the binding still proves direct registered sibling
routing and does not satisfy the registered parent-composed classifier.

The new metric is intentionally narrow:
`binding_uses_registered_sibling_mixed_support` requires a final
child-input binding sourced by a `FlopQ`, port support in that flop's D
cone, virtual instance-output support in the same D cone, and no
registered parent-composed D-cone classification. The focused pipeline
regression disables parent-composed routes and proves positive direct
registered sibling mixed-support while keeping registered
parent-composed and registered mixed-support parent-composed counters at
zero.

Current-code validation includes the focused metrics regression, the
focused pipeline regression, `cargo test --bin tool_matrix`, and the
full r51 Phase 4 hierarchy gate through Verilator plus both repo-owned
Yosys modes.

### Phase 4 r50 banks accumulated mixed-support hierarchy coverage downstream-clean
The previous accumulated mixed-support Phase 4 hierarchy evidence anchor is
`/tmp/anvil-tool-matrix-phase4-hierarchy-r50/tool_matrix_report.json`. It
kept the live policy at 99 scenarios / 396 designs with four designs per
scenario and recorded `coverage_gaps = []`, `artifact_kind = "design"`,
Verilator `396/0`, Yosys without-ABC `396/0`, and Yosys with-ABC
`396/0`.

This bank promotes the three current mixed-support coverage-only slices
into the full downstream-clean surface: stateful helper-backed parent
outputs with parent-port support, unregistered parent-composed helper
child-input mixed support, and stateful helper-through-parent-flop
unregistered child-input mixed support. The prior coverage-only report
trees remain useful focused breadcrumbs, and `r50` remains the previous
full downstream-clean evidence for those policy facts before `r51` carried
them forward.

### Phase 4 stateful parent-composed helper child-input mixed support
The hierarchy gate now distinguishes the stateful parent-composed helper
child-input route from the stricter overlap where the same unregistered
final child-input binding both consumes a helper-sourced parent-local Q
and also carries parent data-port support. This is separate from the
plain helper-through-parent-flop child-input counter and from the plain
unregistered helper mixed-support counter.

The metric intentionally requires both halves on the same binding:
`binding_uses_parent_cone_instance_flop_mixed_support` first reuses the
helper-through-parent-flop classifier, then requires parent-port support
on the final child-input binding's dependency set. Because the
helper-through-parent-flop classifier rejects final `FlopQ` registered
bindings, the new metric stays focused on unregistered parent-composed
child-input logic that reads helper-sourced parent state.

The Phase 4 coverage facts are narrow. The nonrecursive fact requires
child-input cone routing, parent-cone helper instances, parent-local
flops, no direct sibling or registered helper routes in the focused
lane, positive
`child_input_bindings_from_parent_cone_instance_flop_mixed_support`, and
zero registered helper counters. The recursive fact additionally
requires the hierarchy-wide stateful helper and mixed-support counters
to exceed their top-only counterparts.

Current-code validation includes the focused metrics regression,
`cargo test --bin tool_matrix`, and a coverage-only 99-scenario /
396-design Phase 4 dry run at
`/tmp/anvil-tool-matrix-phase4-stateful-helper-child-input-mixed-check`
with `coverage_gaps = []`,
`saw_hierarchy_parent_composed_parent_cone_instance_flop_mixed_support_routing = true`,
and
`saw_recursive_hierarchy_parent_composed_parent_cone_instance_flop_mixed_support_routing = true`.
The previous full downstream-clean `r50` bank carried these facts through
Verilator and both repo-owned Yosys modes; `r51` carries them forward, and
the coverage-only dry run remains a focused breadcrumb.
### Phase 4 unregistered parent-composed helper child-input mixed support
The hierarchy gate now distinguishes parent-composed child-input
bindings that merely reach parent-cone helper outputs from the stricter
overlap where the same unregistered binding also carries parent data-port
support. The generator now repairs required helper-backed child-input
cones by adding a parent-port companion when the helper route would
otherwise lack ports.

The metric is intentionally separate from the registered helper
mixed-support route: `binding_uses_parent_cone_instance_mixed_support`
rejects final `FlopQ` child-input bindings and requires the final
binding to be parent-composed logic. The focused regression reuses the
budgeted helper case because `max_parent_cone_instances_per_module = 3`
already forces helper-backed child-input bindings without needing a new
scenario.

The Phase 4 coverage facts are also narrow. The nonrecursive fact
requires unregistered child-input cones, helper instances, no
parent-flop route, positive
`child_input_bindings_from_parent_cone_instance_mixed_support`, and zero
registered helper child-input bindings. The recursive fact additionally
requires the non-top hierarchy counters to exceed the top counters while
helper-through-flop and registered-helper counters remain zero.

Current-code validation includes the focused metrics regression,
`cargo test --bin tool_matrix`, and a coverage-only 99-scenario /
396-design Phase 4 dry run at
`/tmp/anvil-tool-matrix-phase4-parent-helper-child-input-mixed-check`
with `coverage_gaps = []`,
`saw_hierarchy_parent_cone_instance_mixed_support_routing = true`, and
`saw_recursive_hierarchy_parent_cone_instance_mixed_support_routing =
true`. The full downstream-clean `r50` bank now carries these facts
through Verilator and both repo-owned Yosys modes; the coverage-only dry
run remains a focused breadcrumb.

### Phase 4 stateful parent-output helper mixed-support metrics
The hierarchy gate now distinguishes parent outputs that reach
parent-cone helper instance outputs through parent-local flops from the
stricter overlap where that same output cone also carries parent-port
support. The implementation adds hierarchy/top counters and fractions in
`DesignMetrics`, plus nonrecursive and recursive coverage facts in
`src/bin/tool_matrix.rs`.

The recursive fact stays intentionally narrow: it requires the
hierarchy-wide mixed-through-flop counter to exceed the top-only counter
while child-input helper and registered-helper binding counters stay
zero, so the proof remains a parent-output route instead of drifting
into child-input helper evidence. The Phase 4 required-knob list also
now includes the plain `hierarchy_sibling_route_prob` attempt, closing
the last missing decision-site requirement for the direct sibling route
axis.

Validation included the focused metrics regression,
`cargo test --bin tool_matrix`, a coverage-only 99-scenario / 396-design
Phase 4 dry run at
`/tmp/anvil-tool-matrix-phase4-mixed-helper-check`,
`cargo check --all-targets`, and the full `cargo test` suite with 302 passing
tests. The full downstream-clean `r50` bank now carries these facts
through Verilator and both repo-owned Yosys modes; the coverage-only dry
run remains a focused breadcrumb.

### Phase 4 r49 banks recursive parent-output helper mixed-support downstream-clean
The live Phase 4 hierarchy policy now requires recursive non-top parent
outputs to prove the same output cone can carry both parent-port support
and parent-cone helper output support. The focused regression is
`cargo test recursive_hierarchy_parent_outputs_mix_helper_instances_with_parent_ports_below_top`.

This needed a dedicated output mixed-support metric instead of inferring
the fact from `hierarchy_parent_port_composed_outputs` and
`hierarchy_outputs_reaching_parent_cone_instances`. Those counters can
both be true in a design while describing different parent outputs. The
new `*_outputs_reaching_parent_cone_instance_mixed_support` counters
make the overlap explicit.

The full downstream-clean evidence anchor for that slice was
`/tmp/anvil-tool-matrix-phase4-hierarchy-r49/tool_matrix_report.json`:
`99` scenarios, `4` designs/scenario, `396` total designs,
`coverage_gaps = []`,
`saw_recursive_hierarchy_parent_cone_instance_mixed_support_outputs = true`,
`saw_recursive_hierarchy_parent_cone_instance_outputs = true`,
`saw_hierarchy_parent_port_composed_outputs = true`, and
`saw_recursive_hierarchy_registered_parent_cone_instance_mixed_support_routing = true`
with `396/0` pass-fail in Verilator plus both repo-owned Yosys modes.
The previous recursive non-top registered parent-composed helper
mixed-support full bank is `r48`; `r50` is the previous accumulated
mixed-support hierarchy bank, and `r51` is the current full downstream-clean
Phase 4 hierarchy bank.

### Phase 4 r48 banks recursive registered helper mixed-support routing downstream-clean
The live Phase 4 hierarchy policy now requires the recursive
registered parent-composed helper route to carry parent-port support in
the same D cone below the top parent. In an exact-depth-2 recursive
hierarchy, a parent-cone helper instance can feed registered
parent-composed child-input logic, that logic can also consume parent
data ports, and the resulting parent-local Q can bind a later child
input. The focused regression is
`cargo test recursive_hierarchy_registered_helper_routes_mix_parent_ports_below_top`.

This needed a dedicated helper-mixed metric instead of inferring the
fact from the older registered helper and registered mixed-support
counters. Those counters can both be true in a design without proving
that parent-port support and the parent-cone helper output occur in the
same registered D cone. The new
`registered_parent_cone_instance_mixed_support_*` counters make that
overlap explicit.

The full downstream-clean evidence anchor for that slice was
`/tmp/anvil-tool-matrix-phase4-hierarchy-r48/tool_matrix_report.json`:
`99` scenarios, `4` designs/scenario, `396` total designs,
`coverage_gaps = []`,
`saw_recursive_hierarchy_registered_parent_cone_instance_mixed_support_routing = true`,
`saw_recursive_hierarchy_registered_parent_composed_parent_cone_instance_routing = true`,
`saw_recursive_hierarchy_registered_multistage_mixed_support_routing = true`,
`saw_recursive_hierarchy_registered_multistage_sibling_routing = true`,
`saw_recursive_hierarchy_registered_multistage_routing = true`,
`saw_recursive_hierarchy_registered_mixed_support_routing = true`,
`saw_hierarchy_registered_multistage_routing = true`,
`saw_hierarchy_registered_multistage_sibling_routing = true`,
`saw_recursive_hierarchy_registered_multistage_parent_cone_instance_routing = true`,
`saw_recursive_hierarchy_registered_multistage_parent_composed_parent_cone_instance_routing = true`,
and
`saw_recursive_hierarchy_registered_parent_composed_parent_cone_instance_routing = true`
with `396/0` pass-fail in Verilator plus both repo-owned Yosys modes.
The previous recursive non-top registered multistage mixed-support
no-helper full bank is `r47`.

### Phase 4 r47 banks recursive registered multistage mixed-support routing downstream-clean
The live Phase 4 hierarchy policy now requires the recursive no-helper
overlap between registered mixed support and multi-stage registered
parent-composed routing. Below the top parent, an exact-depth-2
recursive hierarchy can build a registered D cone that simultaneously
uses parent data ports, child instance outputs, and an earlier
parent-local Q, then bind a later child input through the resulting
parent-local state without relying on parent-cone helper instances. The
focused regression is
`cargo test recursive_hierarchy_registered_multistage_mixed_support_routes_below_top`.

This needed a dedicated metric instead of inferring the fact from the
existing mixed-support and multistage counters. Those older counters can
be true in the same design while describing different bindings; the new
`registered_multistage_mixed_support_*` counters only fire when one
registered route contains both kinds of support in the same D cone and
then participates in later Q reuse.

The `r47` full downstream-clean evidence anchor is
`/tmp/anvil-tool-matrix-phase4-hierarchy-r47/tool_matrix_report.json`:
`99` scenarios, `4` designs/scenario, `396` total designs,
`coverage_gaps = []`,
`saw_recursive_hierarchy_registered_multistage_mixed_support_routing = true`,
`saw_recursive_hierarchy_registered_multistage_sibling_routing = true`,
`saw_recursive_hierarchy_registered_multistage_routing = true`,
`saw_recursive_hierarchy_registered_mixed_support_routing = true`,
`saw_hierarchy_registered_multistage_routing = true`,
`saw_hierarchy_registered_multistage_sibling_routing = true`,
`saw_recursive_hierarchy_registered_multistage_parent_cone_instance_routing = true`,
`saw_recursive_hierarchy_registered_multistage_parent_composed_parent_cone_instance_routing = true`,
and
`saw_recursive_hierarchy_registered_parent_composed_parent_cone_instance_routing = true`
with `396/0` pass-fail in Verilator plus both repo-owned Yosys modes.
The previous recursive non-top registered sibling multistage no-helper
full bank is `r46`.

### Cargo default-run is part of the README contract
The repository has two binaries: the generator (`anvil`) and the
auxiliary `tool_matrix` harness. Cargo cannot infer which one plain
`cargo run -- ...` should execute unless `Cargo.toml` keeps
`default-run = "anvil"` in the `[package]` section.

This is a user-facing contract, not cosmetic metadata: README and the
mdBook intentionally teach `cargo run -- ...` for generator examples,
while `tool_matrix` is always selected explicitly with
`cargo run --bin tool_matrix -- ...`. Future auxiliary binaries must
preserve that default-run setting or update every source-tree command
surface in the live docs at the same time.

### Phase 4 r46 banks recursive registered sibling multistage routing downstream-clean
The live Phase 4 hierarchy policy now requires the recursive no-helper
multi-stage registered sibling-routed child-input cross product. Below
the top parent, an exact-depth-2 recursive hierarchy can bind one child
input from an earlier child output through parent-local state, then
reuse that earlier parent-local Q as the D source for a later direct
registered sibling route, without relying on parent-composed D logic or
parent-cone helper instances. The focused regression is
`cargo test recursive_hierarchy_registered_sibling_routes_can_chain_without_helpers_below_top`.
It uses four child instances per recursive parent so the sibling-output
route has enough earlier sources to force both the first registered
binding and the later Q-reuse binding across every construction
strategy.

The full downstream-clean evidence anchor for that slice was
`/tmp/anvil-tool-matrix-phase4-hierarchy-r46/tool_matrix_report.json`:
`99` scenarios, `4` designs/scenario, `396` total designs,
`coverage_gaps = []`,
`saw_recursive_hierarchy_registered_multistage_sibling_routing = true`,
`saw_recursive_hierarchy_registered_multistage_routing = true`,
`saw_recursive_hierarchy_registered_mixed_support_routing = true`,
`saw_hierarchy_registered_multistage_routing = true`,
`saw_hierarchy_registered_multistage_sibling_routing = true`,
`saw_recursive_hierarchy_registered_multistage_parent_cone_instance_routing = true`,
`saw_recursive_hierarchy_registered_multistage_parent_composed_parent_cone_instance_routing = true`,
and
`saw_recursive_hierarchy_registered_parent_composed_parent_cone_instance_routing = true`
with `396/0` pass-fail in Verilator plus both repo-owned Yosys modes.
The previous recursive non-top registered parent-composed multistage
no-helper full bank is `r45`.

### Phase 4 r45 banks recursive registered multistage routing downstream-clean
The live Phase 4 hierarchy policy now requires the recursive no-helper
multi-stage registered parent-composed cross product. Below the top
parent, an exact-depth-2 recursive hierarchy can first bind a child input
through parent-local state, then reuse that earlier parent-local Q in a
later registered parent-composed child-input D cone, without relying on
parent-cone helper instances. The focused regression is
`cargo test recursive_hierarchy_registered_parent_composed_routes_can_chain_without_helpers_below_top`.
It uses four child instances per recursive parent because the two-child
registered mixed-support calibration is too sparse to force this
multi-stage subcase across every construction strategy.

The full downstream-clean evidence anchor for that slice was
`/tmp/anvil-tool-matrix-phase4-hierarchy-r45/tool_matrix_report.json`:
`96` scenarios, `4` designs/scenario, `384` total designs,
`coverage_gaps = []`,
`saw_recursive_hierarchy_registered_multistage_routing = true`,
`saw_recursive_hierarchy_registered_mixed_support_routing = true`,
`saw_hierarchy_registered_multistage_routing = true`,
`saw_recursive_hierarchy_registered_multistage_parent_cone_instance_routing = true`,
`saw_recursive_hierarchy_registered_multistage_parent_composed_parent_cone_instance_routing = true`,
and
`saw_recursive_hierarchy_registered_parent_composed_parent_cone_instance_routing = true`
with `384/0` pass-fail in Verilator plus both repo-owned Yosys modes.
The previous recursive non-top registered mixed-support full bank is
`r44`.

### Phase 4 r44 banks recursive registered mixed-support routing downstream-clean
The live Phase 4 hierarchy policy now requires the recursive no-helper
registered mixed-support cross product. Below the top parent, an
exact-depth-2 recursive hierarchy can build registered parent-composed
child-input D logic from both parent data ports and child outputs, then
drive later child inputs through parent-local state without relying on
parent-cone helper instances. The focused regression is
`cargo test recursive_hierarchy_registered_mixed_support_routes_below_top`.
It requires the recursive tree shape, non-top parent-local flops,
non-top registered parent-composed child-input bindings, non-top
registered child-output support, non-top registered mixed-support
bindings, and zero registered helper-sourced D-cone bindings.

The previous full downstream-clean evidence anchor is
`/tmp/anvil-tool-matrix-phase4-hierarchy-r44/tool_matrix_report.json`:
`93` scenarios, `4` designs/scenario, `372` total designs,
`coverage_gaps = []`,
`saw_recursive_hierarchy_registered_mixed_support_routing = true`,
`saw_hierarchy_registered_mixed_support_routing = true`,
`saw_recursive_multiple_parent_cone_instances_per_parent_child_inputs = true`,
`saw_recursive_multiple_parent_cone_instances_per_parent_through_flops = true`,
`saw_recursive_multiple_parent_cone_instances_per_parent = true`,
`saw_recursive_hierarchy_registered_multistage_parent_cone_instance_routing = true`,
`saw_recursive_hierarchy_registered_multistage_parent_composed_parent_cone_instance_routing = true`,
and
`saw_recursive_hierarchy_registered_parent_composed_parent_cone_instance_routing = true`
with `372/0` pass-fail in Verilator plus both repo-owned Yosys modes.
The previous recursive non-top child-input multi-helper budget full bank
is `r43`.

### Phase 4 r43 banks recursive non-top child-input helper budgets downstream-clean
The live Phase 4 hierarchy policy now closes the child-input local-budget
cross product for recursive parent-cone helpers. Below the top parent,
an exact-depth-2 recursive hierarchy can spend a multi-helper
`max_parent_cone_instances_per_module = 3` budget while driving
parent-composed child-input bindings directly from helper outputs. The
focused regression is
`cargo test recursive_hierarchy_parent_cone_helper_budget_allows_multiple_helpers_below_top`.
It requires the recursive tree shape, the configured helper budget in
`max_parent_cone_instances_per_internal_module`, helper instances beyond
the top parent, non-top parent-composed child-input bindings, non-top
child-input bindings sourced from helper outputs, and zero
helper-through-flop or registered helper child-input bindings.

The current full downstream-clean evidence anchor is
`/tmp/anvil-tool-matrix-phase4-hierarchy-r43/tool_matrix_report.json`:
`90` scenarios, `4` designs/scenario, `360` total designs,
`coverage_gaps = []`,
`saw_recursive_multiple_parent_cone_instances_per_parent_child_inputs = true`,
`saw_recursive_multiple_parent_cone_instances_per_parent_through_flops = true`,
`saw_recursive_multiple_parent_cone_instances_per_parent = true`,
`saw_multiple_parent_cone_instances_per_parent = true`,
`saw_recursive_hierarchy_parent_cone_instance_flop_outputs = true`,
`saw_recursive_hierarchy_parent_cone_instance_outputs = true`,
`saw_recursive_hierarchy_direct_sibling_parent_cone_instance_routing = true`,
`saw_recursive_hierarchy_direct_registered_sibling_parent_cone_instance_routing = true`,
`saw_recursive_hierarchy_registered_multistage_parent_cone_instance_routing = true`,
`saw_recursive_hierarchy_registered_multistage_parent_composed_parent_cone_instance_routing = true`,
`saw_recursive_hierarchy_registered_parent_composed_parent_cone_instance_routing = true`,
and
`saw_recursive_hierarchy_parent_composed_parent_cone_instance_flop_routing = true`
with `360/0` pass-fail in Verilator plus both repo-owned Yosys modes.
The previous recursive non-top stateful multi-helper budget full bank is
`r42`.

### Phase 4 r42 banks recursive non-top stateful helper budgets downstream-clean
Phase 4 r42 closed the stateful local-budget
cross product for recursive parent-output helpers. Below the top parent,
an exact-depth-2 recursive hierarchy can spend a multi-helper
`max_parent_cone_instances_per_module = 3` budget, register the helper
outputs into parent-local flops, and drive parent outputs from those
helper-sourced Qs. The focused regression is
`cargo test recursive_hierarchy_parent_outputs_can_spend_stateful_helper_budget_below_top`.
It requires the recursive tree shape, the configured helper budget in
`max_parent_cone_instances_per_internal_module`, helper instances beyond
the top parent, parent-local flops below the top parent, parent outputs
that depend on helper outputs through those flops, and zero child-input
helper bindings through either direct, stateful, or registered helper
routes.

The full downstream-clean evidence anchor for that slice was
`/tmp/anvil-tool-matrix-phase4-hierarchy-r42/tool_matrix_report.json`:
`87` scenarios, `4` designs/scenario, `348` total designs,
`coverage_gaps = []`,
`saw_recursive_multiple_parent_cone_instances_per_parent_through_flops = true`,
`saw_recursive_multiple_parent_cone_instances_per_parent = true`,
`saw_multiple_parent_cone_instances_per_parent = true`,
`saw_recursive_hierarchy_parent_cone_instance_flop_outputs = true`,
`saw_recursive_hierarchy_parent_cone_instance_outputs = true`,
`saw_recursive_hierarchy_direct_sibling_parent_cone_instance_routing = true`,
`saw_recursive_hierarchy_direct_registered_sibling_parent_cone_instance_routing = true`,
`saw_recursive_hierarchy_registered_multistage_parent_cone_instance_routing = true`,
`saw_recursive_hierarchy_registered_multistage_parent_composed_parent_cone_instance_routing = true`,
`saw_recursive_hierarchy_registered_parent_composed_parent_cone_instance_routing = true`,
and
`saw_recursive_hierarchy_parent_composed_parent_cone_instance_flop_routing = true`
with `348/0` pass-fail in Verilator plus both repo-owned Yosys modes.
The previous recursive non-top multi-helper budget full bank is `r41`.

### Phase 4 r41 banks recursive non-top helper budgets downstream-clean
The live Phase 4 hierarchy policy now names the local-budget half of the
recursive parent-output helper surface. Below the top parent, an
exact-depth-2 recursive hierarchy can spend a multi-helper
`max_parent_cone_instances_per_module = 3` budget for parent-output
composition, not just accumulate one helper per parent across multiple
parents. The focused regression is
`cargo test recursive_hierarchy_parent_outputs_can_spend_helper_budget_below_top`.
It requires the recursive tree shape, the configured helper budget in
`max_parent_cone_instances_per_internal_module`, helper instances beyond
the top parent, parent outputs that depend on those helper outputs, no
child-input helper bindings, and no registered child-input helper D
cones.

The full downstream-clean evidence anchor for that slice was
`/tmp/anvil-tool-matrix-phase4-hierarchy-r41/tool_matrix_report.json`:
`87` scenarios, `4` designs/scenario, `348` total designs,
`coverage_gaps = []`,
`saw_recursive_multiple_parent_cone_instances_per_parent = true`,
`saw_multiple_parent_cone_instances_per_parent = true`,
`saw_recursive_hierarchy_parent_cone_instance_flop_outputs = true`,
`saw_recursive_hierarchy_parent_cone_instance_outputs = true`,
`saw_recursive_hierarchy_direct_sibling_parent_cone_instance_routing = true`,
`saw_recursive_hierarchy_direct_registered_sibling_parent_cone_instance_routing = true`,
`saw_recursive_hierarchy_registered_multistage_parent_cone_instance_routing = true`,
`saw_recursive_hierarchy_registered_multistage_parent_composed_parent_cone_instance_routing = true`,
`saw_recursive_hierarchy_registered_parent_composed_parent_cone_instance_routing = true`,
and
`saw_recursive_hierarchy_parent_composed_parent_cone_instance_flop_routing = true`
with `348/0` pass-fail in Verilator plus both repo-owned Yosys modes.
The previous recursive non-top stateful parent-output helper full bank
is `r40`.

### Phase 4 r40 banks recursive non-top stateful parent-output helper routing downstream-clean
The live Phase 4 hierarchy policy now includes the stateful
parent-output version of the recursive exact-depth-2 helper axis: below
the top parent, a non-top parent can instantiate helper children as
internal parent-cone sources, register those helper outputs into
parent-local flops, and drive parent outputs from the helper-sourced
state. The focused regression is
`cargo test recursive_hierarchy_parent_outputs_can_route_helper_instances_through_parent_flops_below_top`.
It requires the recursive tree shape, more helper instances below top
than at top, more parent-local flops below top than at top, more
helper-through-flop parent-output support across the hierarchy than at
top, no child-input helper bindings, and no registered child-input
helper D cones so the route stays distinct from both child-input helper
routing and direct recursive parent-output helper routing.

The full downstream-clean evidence anchor for that slice was
`/tmp/anvil-tool-matrix-phase4-hierarchy-r40/tool_matrix_report.json`:
`87` scenarios, `4` designs/scenario, `348` total designs,
`coverage_gaps = []`,
`saw_recursive_hierarchy_parent_cone_instance_flop_outputs = true`,
`saw_recursive_hierarchy_parent_cone_instance_outputs = true`,
`saw_recursive_hierarchy_direct_sibling_parent_cone_instance_routing = true`,
`saw_recursive_hierarchy_direct_registered_sibling_parent_cone_instance_routing = true`,
`saw_recursive_hierarchy_registered_multistage_parent_cone_instance_routing = true`,
`saw_recursive_hierarchy_registered_multistage_parent_composed_parent_cone_instance_routing = true`,
`saw_recursive_hierarchy_registered_parent_composed_parent_cone_instance_routing = true`,
and
`saw_recursive_hierarchy_parent_composed_parent_cone_instance_flop_routing = true`
with `348/0` pass-fail in Verilator plus both repo-owned Yosys modes.
The previous recursive non-top parent-output helper full bank is `r39`.

### Phase 4 r39 banks recursive non-top parent-output helper routing downstream-clean
The live Phase 4 hierarchy policy now includes the parent-output version
of the recursive exact-depth-2 helper axis: below the top parent, a
non-top parent can instantiate helper children as internal parent-cone
sources and drive its own parent outputs from those helper outputs. The
focused regression is
`cargo test recursive_hierarchy_parent_outputs_can_depend_on_helper_instances_below_top`.
It requires the recursive tree shape, more helper instances below top
than at top, more parent outputs reaching helper instances across the
hierarchy than at top, no child-input helper bindings, and no
helper-through-parent-flop output counts so the route stays distinct
from child-input helper routing and stateful parent-output helper
routing.

The full downstream-clean evidence anchor for that slice was
`/tmp/anvil-tool-matrix-phase4-hierarchy-r39/tool_matrix_report.json`:
`84` scenarios, `4` designs/scenario, `336` total designs,
`coverage_gaps = []`,
`saw_recursive_hierarchy_parent_cone_instance_outputs = true`,
`saw_recursive_hierarchy_direct_sibling_parent_cone_instance_routing = true`,
`saw_recursive_hierarchy_direct_registered_sibling_parent_cone_instance_routing = true`,
`saw_recursive_hierarchy_registered_multistage_parent_cone_instance_routing = true`,
`saw_recursive_hierarchy_registered_multistage_parent_composed_parent_cone_instance_routing = true`,
`saw_recursive_hierarchy_registered_parent_composed_parent_cone_instance_routing = true`,
and
`saw_recursive_hierarchy_parent_composed_parent_cone_instance_flop_routing = true`
with `336/0` pass-fail in Verilator plus both repo-owned Yosys modes.
The previous recursive multi-stage registered parent-composed helper
full bank is `r38`.

### Phase 4 r38 banks recursive non-top multi-stage registered parent-composed helper routing downstream-clean
The live Phase 4 hierarchy policy now includes the multi-stage
registered parent-composed version of the recursive exact-depth-2 helper
axis: below the top parent, a parent-cone helper output can seed a
parent-local Q, and later registered parent-composed D logic can reuse
that helper-sourced Q before driving a later child input. The focused
regression is
`cargo test recursive_hierarchy_registered_parent_composed_routes_can_chain_helper_instances_below_top`.
It requires the recursive tree shape, more helper instances below top
than at top, more multi-stage registered parent-composed bindings below
top than at top, more multi-stage helper-sourced parent-composed
bindings below top than at top, local parent flops below top, and zero
direct multi-stage registered helper counters so the route stays
distinct from the direct registered sibling helper-chain axis.

The full downstream-clean evidence anchor for that slice was
`/tmp/anvil-tool-matrix-phase4-hierarchy-r38/tool_matrix_report.json`:
`81` scenarios, `4` designs/scenario, `324` total designs,
`coverage_gaps = []`,
`saw_recursive_hierarchy_direct_sibling_parent_cone_instance_routing = true`,
`saw_recursive_hierarchy_direct_registered_sibling_parent_cone_instance_routing = true`,
`saw_recursive_hierarchy_registered_multistage_parent_cone_instance_routing = true`,
`saw_recursive_hierarchy_registered_multistage_parent_composed_parent_cone_instance_routing = true`,
`saw_recursive_hierarchy_registered_parent_composed_parent_cone_instance_routing = true`,
and
`saw_recursive_hierarchy_parent_composed_parent_cone_instance_flop_routing = true`
with `324/0` pass-fail in Verilator plus both repo-owned Yosys modes.
The previous recursive multi-stage direct registered-helper full bank is
`r37`.

### Phase 4 r37 banks recursive non-top multi-stage direct registered helper routing downstream-clean
The live Phase 4 hierarchy policy now includes the multi-stage version
of the recursive exact-depth-2 direct registered helper axis: below the
top parent, a direct registered sibling route can seed a parent-local Q
from a parent-cone helper instance, and a later direct registered sibling
route can reuse that helper-sourced Q as the next parent-flop D source.
The focused regression is
`cargo test recursive_hierarchy_registered_sibling_routes_can_chain_helper_instances_below_top`.
It requires the recursive tree shape, more helper instances below top
than at top, more multi-stage registered sibling bindings below top than
at top, more multi-stage helper-sourced registered sibling bindings below
top than at top, local parent flops below top, and zero registered
parent-composed counters so the route stays distinct from the
parent-composed helper-chain axis.

The full downstream-clean evidence anchor for that slice was
`/tmp/anvil-tool-matrix-phase4-hierarchy-r37/tool_matrix_report.json`:
`78` scenarios, `4` designs/scenario, `312` total designs,
`coverage_gaps = []`,
`saw_recursive_hierarchy_direct_sibling_parent_cone_instance_routing = true`,
`saw_recursive_hierarchy_direct_registered_sibling_parent_cone_instance_routing = true`,
`saw_recursive_hierarchy_registered_multistage_parent_cone_instance_routing = true`,
`saw_recursive_hierarchy_registered_parent_composed_parent_cone_instance_routing = true`,
and
`saw_recursive_hierarchy_parent_composed_parent_cone_instance_flop_routing = true`
with `312/0` pass-fail in Verilator plus both repo-owned Yosys modes.
The previous recursive registered parent-composed helper full bank is
`r36`.

### Phase 4 r36 banks recursive non-top registered parent-composed helper routing downstream-clean
The live Phase 4 hierarchy policy now includes the registered
parent-composed version of the recursive exact-depth-2 helper axis:
non-top registered parent-composed child-input D cones can source from
parent-cone helper instances below the top parent. The focused
regression is
`cargo test recursive_hierarchy_registered_child_input_cones_can_use_helper_instances_below_top`.
It requires the recursive tree shape, more helper instances below top
than at top, more registered parent-composed bindings below top than at
top, more registered helper bindings below top than at top, and local
parent flops below top.

The coverage-only policy anchor is
`/tmp/anvil-tool-matrix-phase4-recursive-registered-parent-helper-r36/tool_matrix_report.json`:
`75` scenarios, `4` designs/scenario, `300` total designs,
`coverage_gaps = []`, and
`saw_recursive_hierarchy_registered_parent_composed_parent_cone_instance_routing = true`.

The full downstream-clean evidence anchor for that slice was
`/tmp/anvil-tool-matrix-phase4-hierarchy-r36/tool_matrix_report.json`:
`75` scenarios, `4` designs/scenario, `300` total designs,
`coverage_gaps = []`,
`saw_recursive_hierarchy_direct_sibling_parent_cone_instance_routing = true`,
`saw_recursive_hierarchy_direct_registered_sibling_parent_cone_instance_routing = true`,
`saw_recursive_hierarchy_registered_parent_composed_parent_cone_instance_routing = true`,
and
`saw_recursive_hierarchy_parent_composed_parent_cone_instance_flop_routing = true`
with `300/0` pass-fail in Verilator plus both repo-owned Yosys modes.
The previous recursive direct registered-helper full bank is `r35`.

### Phase 4 r35 banks recursive non-top direct registered helper routing downstream-clean
The live Phase 4 hierarchy policy now includes the registered sibling
version of the recursive exact-depth-2 helper axis: non-top direct
registered sibling-routed child-input D paths can source from parent-cone
helper instances below the top parent. The focused regression is
`cargo test recursive_hierarchy_registered_sibling_routes_can_use_helper_instances_below_top`.
It requires the recursive tree shape, more helper instances below top
than at top, more registered sibling helper bindings below top than at
top, local parent flops below top, and zero registered parent-composed
D-cone counters so the route stays distinct from registered
parent-composed helper routing.

The coverage-only policy anchor is
`/tmp/anvil-tool-matrix-phase4-recursive-direct-registered-helper-r35/tool_matrix_report.json`:
`72` scenarios, `4` designs/scenario, `288` total designs,
`coverage_gaps = []`, and
`saw_recursive_hierarchy_direct_registered_sibling_parent_cone_instance_routing = true`.

The current full downstream-clean evidence anchor is
`/tmp/anvil-tool-matrix-phase4-hierarchy-r35/tool_matrix_report.json`:
`72` scenarios, `4` designs/scenario, `288` total designs,
`coverage_gaps = []`,
`saw_recursive_hierarchy_direct_sibling_parent_cone_instance_routing = true`,
`saw_recursive_hierarchy_direct_registered_sibling_parent_cone_instance_routing = true`,
and
`saw_recursive_hierarchy_parent_composed_parent_cone_instance_flop_routing = true`
with `288/0` pass-fail in Verilator plus both repo-owned Yosys modes.
The previous recursive direct-helper full bank is `r34`.

### Phase 4 r34 banks recursive non-top direct helper routing downstream-clean
The live Phase 4 hierarchy policy now includes a recursive exact-depth-2
axis that proves direct sibling-routed child inputs below the top parent
can source from parent-cone helper instances. The focused regression is
`cargo test recursive_hierarchy_sibling_routes_can_use_helper_instances_below_top`.
It requires the recursive tree shape, more helper instances and helper
bindings below top than at top, and zero registered helper counters so the
route stays distinct from registered sibling/helper D routing.

The first coverage-only policy anchor was
`/tmp/anvil-tool-matrix-phase4-recursive-direct-helper-r32/tool_matrix_report.json`;
the first full downstream attempt at
`/tmp/anvil-tool-matrix-phase4-hierarchy-r32/tool_matrix_report.json`
correctly failed because Yosys found one warning in both modes. The
repro was `int_nodeid_egraph_phase4_recur_profile_d2_top4_mid2_seq`,
`design_0002`, top `mod_50_0019`: a procedural `case` with an exact
selector chose an arm whose bounds made a later shift provably constant.
That exposed a real cleanup gap rather than a hierarchy bug: the cheap
bounds revisit handled shifts and ternary muxes, but not exact-selector
`CaseMux` / `CasezMux` arms. `src/gen/cone.rs` now teaches
`node_unsigned_bounds` and `exact_gate_value` to follow those procedural
mux arms conservatively, with regressions for the CaseMux overshift shape
and exact matching Casez patterns.

The current full downstream-clean evidence anchor is
`/tmp/anvil-tool-matrix-phase4-hierarchy-r34/tool_matrix_report.json`:
`69` scenarios, `4` designs/scenario, `276` total designs,
`coverage_gaps = []`,
`saw_recursive_hierarchy_direct_sibling_parent_cone_instance_routing = true`,
and
`saw_recursive_hierarchy_parent_composed_parent_cone_instance_flop_routing = true`
with `276/0` pass-fail in Verilator plus both repo-owned Yosys modes.
The previous recursive helper-state full bank is `r31` for the
66-scenario policy. The first clean direct-helper full bank was `r33`;
`r34` refreshes it after the post-remap idempotent duplicate cleanup.

### Phase 4 r31 banks recursive non-top helper state downstream-clean
The live Phase 4 hierarchy policy now includes a recursive exact-depth-2
axis that proves stateful parent-composed helper child-input routing
below the top parent. The focused regression is
`cargo test recursive_hierarchy_parent_composed_helper_routes_can_use_parent_flops_below_top`.
It is deliberately stronger than the depth-1 stateful helper proof:
it requires `hierarchy_parent_cone_instances > top_parent_cone_instances`,
`hierarchy_parent_local_flops > top_local_flops`, and
`child_input_bindings_from_parent_cone_instances_through_parent_flops >
top_child_input_bindings_from_parent_cone_instances_through_parent_flops`.

The first coverage-only policy anchor was
`/tmp/anvil-tool-matrix-phase4-recursive-helper-state-r31/tool_matrix_report.json`;
the current full downstream-clean evidence anchor is
`/tmp/anvil-tool-matrix-phase4-hierarchy-r31/tool_matrix_report.json`:
`66` scenarios, `4` designs/scenario, `264` total designs,
`coverage_gaps = []`, and
`saw_recursive_hierarchy_parent_composed_parent_cone_instance_flop_routing = true`
with `264/0` pass-fail in Verilator plus both repo-owned Yosys modes.
The previous full downstream-clean bank is `r30` for the 63-scenario
stateful parent-composed helper policy.

### Phase 4 r30 superseded the r29 registered parent-composed helper bank
Stateful parent-composed helper child-input routing now has its own
proof, distinct from registered child-input helper D cones. In the new
shape, a parent-cone helper output seeds a parent-local Q, and
unregistered parent-composed child-input logic consumes that helper Q
before binding the later child input. The focused proof should assert
`child_input_bindings_from_parent_cone_instances_through_parent_flops > 0`
and
`parent_cone_instance_flop_child_input_binding_fraction > 0.0` while
keeping `child_input_bindings_from_registered_parent_cone_instances = 0`.

The current evidence anchor is
`/tmp/anvil-tool-matrix-phase4-hierarchy-r30/tool_matrix_report.json`:
`63` scenarios, `4` designs/scenario, `252` total designs,
`coverage_gaps = []`, and `252/0` pass-fail in Verilator plus both
repo-owned Yosys modes. It fully banks the direct sibling helper,
direct registered sibling helper, multi-stage registered sibling,
stateful parent-output helper, multi-stage direct registered sibling
helper, multi-stage registered parent-composed helper, and stateful
parent-composed helper child-input routes. Keep `r23` as the
pre-direct-helper full-bank breadcrumb, `r24` as the coverage-only
direct-helper proof, `r25` as the direct-helper full bank, `r26` as the
previous multi-stage sibling full bank, `r27` as the previous stateful
parent-output helper bank, `r28` as the previous multi-stage direct
registered sibling helper bank, and `r29` as the previous
multi-stage registered parent-composed helper bank.

### Phase 4 r29 supersedes the r28 direct registered helper-chain bank
Registered parent-composed helper routing now has its own multi-stage
proof, distinct from the direct registered sibling helper proof. In the
new shape, a parent-cone helper output seeds an earlier parent-local Q,
and later registered parent-composed D logic reuses that Q before
driving a later child input. The focused proof should keep
`child_input_bindings_from_registered_multistage_parent_cone_instances = 0`
while asserting
`child_input_bindings_from_registered_multistage_parent_composed_parent_cone_instances > 0`,
so the direct sibling helper chain and parent-composed helper chain
stay observably separate.

The current evidence anchor is
`/tmp/anvil-tool-matrix-phase4-hierarchy-r29/tool_matrix_report.json`:
`60` scenarios, `4` designs/scenario, `240` total designs,
`coverage_gaps = []`, and `240/0` pass-fail in Verilator plus both
repo-owned Yosys modes. It fully banks the direct sibling helper,
direct registered sibling helper, multi-stage registered sibling,
stateful parent-output helper, multi-stage direct registered sibling
helper, and multi-stage registered parent-composed helper routes. Keep
`r23` as the pre-direct-helper full-bank breadcrumb, `r24` as the
coverage-only direct-helper proof, `r25` as the direct-helper full bank,
`r26` as the previous multi-stage sibling full bank, `r27` as the
previous stateful parent-output helper bank, and `r28` as the previous
multi-stage direct registered sibling helper bank.

### Phase 4 r28 superseded the r27 stateful parent-output helper bank
Direct registered sibling helper routing now has two distinct
child-input-proven forms: a helper output can feed the immediate
parent-local D path, and a helper output can first seed a parent-local Q
that a later registered sibling route reuses as the next flop's D
source. The second form is not registered parent-composed logic: the
focused proof should keep both
`child_input_bindings_from_registered_parent_composed_logic = 0` and
`child_input_bindings_from_registered_multistage_parent_composed_logic = 0`
while asserting
`child_input_bindings_from_registered_multistage_parent_cone_instances > 0`.

The evidence anchor was
`/tmp/anvil-tool-matrix-phase4-hierarchy-r28/tool_matrix_report.json`:
`57` scenarios, `4` designs/scenario, `228` total designs,
`coverage_gaps = []`, and `228/0` pass-fail in Verilator plus both
repo-owned Yosys modes. It fully banks the direct sibling helper,
direct registered sibling helper, multi-stage registered sibling,
stateful parent-output helper, and multi-stage direct registered
sibling helper routes. Keep `r23` as the pre-direct-helper full-bank
breadcrumb, `r24` as the coverage-only direct-helper proof, `r25` as
the direct-helper full bank, `r26` as the previous multi-stage sibling
full bank, and `r27` as the previous stateful parent-output helper
bank.

### Phase 4 r27 superseded the r26 multi-stage sibling bank
Parent-output helper routing now has two distinct output-proven forms:
direct helper-to-parent-output composition, and helper-to-parent-output
composition through parent-local state. The second form is not the same
as registered child-input routing: no child input needs to bind from a
helper output, and the proof should keep
`child_input_bindings_from_parent_cone_instances = 0` while asserting
that parent outputs reach helper instances through flop Qs.

That evidence anchor was
`/tmp/anvil-tool-matrix-phase4-hierarchy-r27/tool_matrix_report.json`:
`54` scenarios, `4` designs/scenario, `216` total designs,
`coverage_gaps = []`, and `216/0` pass-fail in Verilator plus both
repo-owned Yosys modes. It fully banks the direct sibling helper, direct
registered sibling helper, multi-stage registered sibling, and stateful
parent-output helper routes. Keep `r23` as the pre-direct-helper
full-bank breadcrumb, `r24` as the coverage-only direct-helper proof,
`r25` as the direct-helper full bank, and `r26` as the previous
multi-stage sibling full bank.

### Parent-output helper-through-flop metrics should stay dependency-based
The tempting implementation of the stateful parent-output helper metric
is to recursively walk every output cone looking for `FlopQ` nodes, then
recursively walk every flop's D cone looking for parent-cone helper
instance outputs. That is correct on small examples but too expensive
on parent-state-heavy scenarios that have many local flops and no
parent-cone helpers, and it re-walks the same D cones repeatedly.

The metric now first checks whether the module even has
`InstanceRole::ParentCone` instances. When it does, it uses the output
root's `DepSet` to find flop virtuals and the existing
`collect_instance_output_support` memo to ask whether each flop D side
reaches a parent-cone helper output. This preserves the structural fact
while keeping the Phase 4 matrix from turning a coverage metric into a
generation-time hotspot.

### Phase 4 r26 supersedes the r25 direct-helper bank
Direct sibling helper routes originally landed after the `r23`
full downstream-clean Phase 4 hierarchy bank, so `r24` was deliberately
coverage-only evidence for the expanded 48-scenario policy. `r25`
banked that direct-helper policy through the downstream tools. `r26`
adds the multi-stage registered sibling route and is now the current
historical 51-scenario evidence anchor.

That evidence anchor is
`/tmp/anvil-tool-matrix-phase4-hierarchy-r26/tool_matrix_report.json`:
`51` scenarios, `4` designs/scenario, `204` total designs,
`coverage_gaps = []`, and `204/0` pass-fail in Verilator plus both
repo-owned Yosys modes. It fully banks the direct sibling helper and
direct registered sibling helper routes, plus the registered sibling
route that chains through earlier parent-local Qs. Keep `r23` only as
the pre-direct-helper full-bank breadcrumb, `r24` only as the
coverage-only direct-helper proof, and `r25` only as the previous
direct-helper full bank.

### Direct sibling helper routes must stay unregistered in the metrics
Direct sibling routing can now request a parent-cone helper instance
source when `hierarchy_parent_cone_instance_prob` fires. That route
still binds the later child input directly from a dep-bearing parent
source; it does not allocate a parent-local flop and it must not be
counted as registered routing just because the source is a helper
instance output.

The metric contract is therefore split deliberately:
`child_input_bindings_from_parent_cone_instances` and the matching
plain helper fractions prove that a helper output reached a child input,
while `child_input_bindings_from_registered_parent_cone_instances`
stays zero unless the final binding goes through a registered route.
The focused direct sibling helper regression forces the registered
sibling and registered parent-composed axes off to preserve that
separation.

### Registered helper metrics must include direct registered sibling routes
The registered helper-instance metric originally grew out of the
registered parent-composed child-input route, where the child binding is
proved by a parent-composed D cone followed by one local parent flop.
That shape is still important, but it is not the only registered route
that can legitimately use a parent-cone helper instance.

Direct registered sibling routing can now request a helper instance
source when `hierarchy_parent_cone_instance_prob` fires. In that case
the route remains a registered child-input binding, but the flop D side
may be the helper `InstanceOutput` itself or a width-adapter gate over
that output, not a registered parent-composed D-cone root.

Therefore the metric contract is dependency-based: inspect the final
registered flop D dependencies and ask whether they include a
parent-cone helper instance output. Requiring the D node to also look
like registered parent-composed logic would undercount the direct
registered sibling helper route and would make the test prove the wrong
shape.

### Parent-output helper budgeting must be output-proven, not helper-to-helper-proven
The parent-output helper route originally proved only one fact: a parent
output could depend on a parent-cone helper instance output. The
separate helper-budget route proved a different fact through
child-input bindings: a parent could allocate multiple helper children
when `max_parent_cone_instances_per_module` was raised. Those two facts
did not prove that parent-output composition itself could spend the
budget.

The current implementation therefore collects parent-output helper
sources before parent-output root construction and lets promotion select
a required helper source per output. That makes the budget visible at
the output-composition seam instead of relying on later child-input
routes.

One gotcha is important: helper instances created specifically for
parent-output composition should not bind their own child inputs from
earlier helper outputs. If that helper-to-helper chaining is allowed,
`child_input_bindings_from_parent_cone_instances` becomes non-zero and
the test no longer proves an output-only path. The parent-output helper
collector therefore uses non-helper parent sources for helper child
inputs while still publishing the helper outputs to the real parent
source pool for output composition.

### Phase 4 starts as wrapper hierarchy on purpose
The first hierarchy slice is deliberately **not** "instances can appear
anywhere in any parent cone". That broader story is the destination,
but it is not the cheapest truthful first landing.

What landed instead is:

- generate a library of leaf modules with the already-proven leaf
  kernel,
- choose the wrapper's instantiated-child count separately from the
  library size,
- keep `num_child_instances = 0` as the legacy compatibility mode
  meaning "instantiate every generated leaf definition exactly once",
- if fewer instances than library entries are requested, instantiate a
  shuffled subset without replacement,
- if more instances than library entries are requested, cover every
  library entry once and then fill the remaining slots by reuse with
  replacement,
- build a real top wrapper module,
- treat child instance outputs as real parent-side leaf variables for
  top-output construction, and
- make emission / validation / manifest handling design-aware.

That buys several real things immediately:

- ANVIL now emits genuine multi-module SV, not just disconnected leaf
  files;
- downstream tools now see elaboration and inter-module port binding;
- the IR and validator now carry explicit instance structure; and
- the hierarchy layer stays above `generate_leaf_module` instead of
  smearing inter-module behavior into the leaf kernel.

Just as importantly, it keeps the open work honest. The first-landed
top layer was **combinational only**; since then, bounded recursive
sub-hierarchy growth, local parent flops, child-input routing, mixed
parent-port / child-output parent outputs, parent-cone helper instances
for parent-composed child-input cones, direct sibling child-input
routes, direct registered sibling D sources, registered child-input D
cones, parent-output cones, and explicit helper budgeting have landed
as separate slices. Broader helper-instance placement
beyond those seams, broader registered hierarchy-local routing, and
hierarchical identity remain future work.

Two narrow implementation choices are load-bearing in this slice:

- the wrapper top marks shared `clk` / `rst_n` as
  `Module.clock` / `Module.reset`, and control-port visibility is now
  design-aware instead of leaf-local: pure comb-only modules omit those
  ports, while hierarchy parents keep them visible iff they carry local
  state or sequential descendants;
- `Node::InstanceOutput` now carries a real dep-bearing leaf identity,
  so parent cones can use child outputs without being mistaken for
  empty-dep constants by later cleanup/finalisation passes.

That second point shook loose three old wrapper-era assumptions that had
to be fixed at the root instead of papered over:

- `compact_node_ids` was only treating output drives and flop holders as
  liveness roots, so instance input bindings could survive with stale
  `NodeId`s after compaction. The real fix was to mark instance-input
  bindings as holders and remap them just like drives and flops.
- `validate_design` was still enforcing "every child output is exposed
  exactly once", which was only true for the pass-through wrapper era.
  The right rule is narrower: any *referenced* child output node must
  name a real child port at the right width, but unreferenced child
  outputs are legal.

### Depth ranges must stay ranges in recursive hierarchy
The original bounded recursive planner was honest enough as a first
landing, but it was still throwing away too much information: it took
`min_hierarchy_depth..=max_hierarchy_depth` and collapsed that interval
to one exact realized depth for the whole design.

That was acceptable as a foothold. It is not the right long-term
algorithm, because the point of a bounded depth interval is to describe
allowed leaf-depth variation, not to sample one global scalar and ignore
the rest.

The strengthened planner now carries a remaining `[min,max]` depth
interval per subtree:

- `max == 0` still means a mandatory leaf;
- if a flexible subtree has only one child, it may sample one exact
  depth inside the still-allowed interval for that chain; and
- when a subtree is both depth-flexible and branching (`instances >= 2`)
  it now deliberately generates child definitions that realize both the
  shallowest and deepest still-legal descendants, instead of hoping RNG
  stumbles into both.

That last point is the load-bearing part. Mixed-depth recursion should
not be a rare accident of repeated runs; when the structure can support
it, the planner should intentionally exercise it.

The metrics contract grew with the planner: `DesignMetrics` now expose
`leaf_module_occurrences_by_depth`, so "did we really get both shallow
and deep leaves?" is answerable numerically from the manifest rather
than by reading emitted SV.

The focused artifact at `/tmp/anvil-hier-mixed-depth-smoke-r1/manifest.json`
was the first clean proof of that new mixed-depth recursive axis. The
current repo-owned Phase 4 gate at
`/tmp/anvil-tool-matrix-phase4-hierarchy-r27/tool_matrix_report.json`
also proves it, so the mixed-depth story is no longer "focused-only"
evidence.
- the emitter was still assuming every child output had a corresponding
  `Node::InstanceOutput`. That is no longer true once the parent may use
  only a subset of child outputs, so unused outputs are now rendered as
  explicit unconnected instance ports (`.port()`).

The separation between **library size** and **instance count** matters
enough to say plainly here: they are different planning decisions and
should not be welded together. Repeated instantiation of the same child
definition stresses elaboration and sharing pressure in a different way
from simply generating more unique children. Under-instantiating the
library is also useful because it exercises real unused-module cleanup
in downstream tools. The wrapper slice still is not the final hierarchy
algorithm, but this split is a real step toward a budget-driven one.

Module names are also a hierarchy resource now, not an incidental string
format at each construction site. Leaf modules, recursive parent
modules, and later designs in the same generator run all reserve names
from the same `Generator` sequence. That keeps `--count N --out DIR`
safe for hierarchy output: one module definition still maps to one
`.sv` file, and no later design can overwrite an earlier definition by
reusing the same `mod_<seed>_<index>` name.

### Bounded recursive hierarchy keeps the old wrapper lane, then adds a real tree planner
The next honest Phase 4 step was **not** to quietly overload the old
depth-1 wrapper knobs until they accidentally meant recursion. That
would have blurred the already-banked wrapper evidence and made the
meaning of the config surface harder to recover later.

So the deliberate rule now is:

- keep the legacy exact wrapper lane alive:
  `hierarchy_depth = 1`, `num_leaf_modules`, `num_child_instances`;
- add a separate bounded recursive lane:
  `min_hierarchy_depth..=max_hierarchy_depth` and
  `min_child_instances_per_module..=max_child_instances_per_module`; and
- make the two planning surfaces mutually exclusive.

That gives us a clean story:

- old repo-owned wrapper closure artifacts stay truthful;
- new recursive hierarchy is explicit in configs and manifests; and
- future recursive work can evolve without pretending the exact wrapper
  lane already solved arbitrary tree planning.

The current recursive planner is now intentionally exact about the
interval contract rather than about one sampled scalar:

- every realized leaf depth stays inside the requested `[min:max]`
  interval;
- when a subtree is both depth-flexible and branching, the planner
  deliberately exercises both the shallowest and deepest still-legal
  descendants instead of relying on luck; and
- each non-leaf module still picks its child-instance count uniformly
  inside the requested child-instance interval.

So the current guarantees are:

- realized leaf depths stay inside the requested interval;
- mixed shallow/deep trees are now intentional when the structure can
  support them;
- realized branching always stays inside the requested interval; and
- the metrics can prove all of that numerically.

One more planning layer is now live on top of that baseline:

- `min_child_instances_per_module..=max_child_instances_per_module`
  remains the global fallback range for recursive branching; and
- repeated `child_instances_per_depth` overrides can tighten or replace
  that range at specific parent depths (`0` = top, `1` = its direct
  children, ...).

That keeps the control surface honest: users can ask for "top is wide,
lower levels are narrower" without inventing a separate planner mode or
forcing the manifest reader to reverse-engineer the realized tree by
hand.

What it does **not** do yet is make every parent-side cone free to
instantiate arbitrary helper modules. The narrow helper-instance seams
are now live for parent-composed child-input cones, direct sibling
routes, direct registered sibling D sources, registered child-input D
cones, parent-output cones, multi-stage direct registered sibling
helper chains, and multi-stage registered parent-composed helper
chains, with explicit per-parent budgeting. Broader helper placement
beyond those seams remains future hierarchy work.

One more gate-level rule turned out to matter here: when a repo-owned
matrix grows new representative scenarios, its per-scenario evidence
budget must not shrink by accident. The Phase 4 gate moved from 15 to
18 scenarios once the mixed-depth recursive axis was added, so its
minimum total design budget was raised from 48 to 60 to preserve the
old 4 designs/scenario sampling depth instead of silently falling to 3.

That lesson is now encoded directly for Phase 4. After the
parent-output helper, budgeted-helper, and registered helper-sourced
child-input axes raised the scenario set to 42, the old
`PHASE4_HIERARCHY_MIN_TOTAL_DESIGNS = 120` rule silently produced only
3 designs/scenario (`126` total) in the clean pre-fix `r22` run. The
live gate now uses a per-scenario floor
`PHASE4_HIERARCHY_MIN_DESIGNS_PER_SCENARIO = 4`. After the direct
sibling helper and direct registered sibling helper axes raised the
scenario set to `48`, the live regression expected `192` total designs.
After the multi-stage registered sibling route raised the scenario set
to `51`, the live regression expects `204` total designs. The
stateful parent-output helper route raised the scenario set to `54`,
and the live regression expected `216` total designs. The multi-stage
direct registered sibling helper route raised it to `57` scenarios /
`228` total designs in `r28`, and the multi-stage registered
parent-composed helper route raises it to `60` scenarios / `240` total
designs in `r29`.

One more planner rule is load-bearing here: in recursive range mode,
child libraries are generated **on demand per parent**, and every
generated direct child definition is instantiated at least once. That
keeps reuse live without manufacturing dead unreachable subtrees just to
inflate counts. The legacy exact wrapper lane remains the place where
top-level under-instantiation of a pre-generated library is exercised.

### Explicit child sourcing must be a real axis, not a vague future promise
Once the wrapper/reuse/under-instantiation story and the recursive
mixed-depth story were both banked, the next honest Phase 4 question
was no longer "can hierarchy exist?" but "how do parents obtain child
definitions?"

That decision is too load-bearing to hide behind ad hoc planner
behavior. It needs to be a user-visible, measurable axis:

- `library` means pre-generate a reusable child-definition pool and let
  instance slots pick from it;
- `on-demand` means synthesize child definitions against exact
  parent-planned data-interface profiles per planned instance slot.

The current landed `on-demand` slice is now the stronger honest one:
each planned child slot carries an exact parent-planned data-interface
profile, and the realized child definition is validated against that
exact emitted data-input/output shape. Control ports stay structural,
which is also the right rule: `clk` / `rst_n` are propagated by
sequential-state presence, not by the data-profile planner.

That is also why the metrics contract grew again. The hierarchy reports
now need to distinguish:

- reused child definitions,
- single-use instantiated definitions, and
- the average instance count per unique instantiated module,
- exact profiled instance-slot coverage, and
- whether child data-input bindings stay dep-bearing instead of
  collapsing to constants.

Without those numbers, `library` vs `on-demand` would still force a
human to open the emitted `.sv`, which is exactly the trust failure we
want to avoid.

The repo-owned Phase 4 gate has now caught up here too. The current
artifact is `/tmp/anvil-tool-matrix-phase4-hierarchy-r27/tool_matrix_report.json`,
and it explicitly proves both child-sourcing modes (`library` and
`on-demand`) together with structural proof that the on-demand
scenarios really emitted fresh child definitions per planned instance
slot and exact profiled child-interface synthesis.

### Combinational sibling routing was the right next layer before local parent state
Once parent-composed outputs and exact profiled child sourcing were
real, the next honest hierarchy question for that slice was not
"should parents have local flops yet?" It was "can one child feed
another through the parent without us faking it as a top-level wrapper
input?"

The current answer is now yes, but intentionally only on the simpler
surface:

- later child data inputs may bind from earlier sibling instance
  outputs;
- the routing stays acyclic by construction because only already-built
  sibling outputs are eligible;
- the routing stayed purely combinational in that slice; and
- local parent flops deliberately remained future work instead of being
  smuggled into the same step.

That last point matters. Child-output -> child-input through local flop
layers is a valid future hierarchy surface, but it is a different
question from the one we needed to close here. This slice was about
making the parent behave more like the leaf generator's cone builder,
except with child-module outputs as additional dep-bearing leaves, while
keeping the phase boundary honest.

The metrics contract had to grow again for the same reason. A sibling
routing feature that can only be confirmed by opening `.sv` is not a
trustworthy feature. The design reports now distinguish:

- child inputs bound from parent ports,
- child inputs bound from sibling instance outputs,
- mixed-support child inputs, and
- the hierarchy-wide and top-level fractions of child inputs that come
  from sibling instance outputs.

The focused proof artifact is now
`/tmp/anvil-hier-sibling-routing-smoke-r1/manifest.json`, and the
repo-owned Phase 4 gate at
`/tmp/anvil-tool-matrix-phase4-hierarchy-r27/tool_matrix_report.json`
now requires `saw_hierarchy_sibling_routing = true`.

### Parent-composed child-input bindings are the cone-builder analogue of sibling routing
The next hierarchy routing step keeps the same phase boundary but
removes one more artificial flat-wrapper shape. Direct sibling routing
answers "can a later child consume an earlier child output?" Parent-
composed child-input binding answers "can the parent build a small
combinational cone for a child input, using the same generator machinery
as leaf cones?"

For that slice, the rule was deliberately narrow and structural:

- `hierarchy_child_input_cone_prob` controls the probability of this
  route;
- the cone's source pool contains only already-available parent sources:
  parent data inputs, earlier sibling instance outputs, and earlier
  parent-side route gates;
- local parent flops stayed disabled, so the parent-composed
  child-input route was a purely combinational composition surface; and
- the rule applies to both the legacy wrapper lane and the bounded
  recursive lane.

This is the shape the user suggested: at the composition level, replace
"gate" by "child module" where it makes sense, but keep that first
slice combinational. Local parent flops are now landed under
`hierarchy_parent_flop_prob`; the first one-flop registered sibling
route is now landed under `hierarchy_registered_sibling_route_prob`,
and the first registered parent-composed child-input route is now
landed under `hierarchy_registered_child_input_cone_prob`. The first
multi-stage registered parent-composed subcase is also live now, while
broader registered hierarchy routing remains a later, separate
hierarchy surface.

The metrics contract grew again with
`child_input_bindings_from_parent_composed_logic`,
`parent_composed_child_input_binding_fraction`, and
`top_parent_composed_child_input_binding_fraction`. The repo-owned
Phase 4 gate treats this as a required coverage fact via
`saw_hierarchy_parent_composed_child_inputs`; the current banked gate at
`/tmp/anvil-tool-matrix-phase4-hierarchy-r27/tool_matrix_report.json`
proves it together with local parent state, `coverage_gaps = []`, and
216/0 clean pass-fail in Verilator plus both repo-owned Yosys modes.
The focused targeted proof is
`/tmp/anvil-hier-child-input-cone-smoke-r1/manifest.json`.

### Parent-cone helper instances make module instantiation a parent source choice
The first helper-instantiation slice is intentionally small: when
`hierarchy_parent_cone_instance_prob` fires during a
parent-composed child-input route, the parent may instantiate one helper
child as an internal parent-cone source. That helper is not one of the
planned child slots. It is tagged with `InstanceRole::ParentCone`, bound
from the parent source pool, and its outputs can feed later child inputs
through ordinary parent combinational logic.

That gives the hierarchy planner a first real "module instance as cone
source" behavior without making every parent-side cone recursive all at
once. The metrics contract is explicit:
`top_parent_cone_instances`, `hierarchy_parent_cone_instances`,
`child_input_bindings_from_parent_cone_instances`,
`top_child_input_bindings_from_parent_cone_instances`,
`parent_cone_instance_child_input_binding_fraction`, and
`top_parent_cone_instance_child_input_binding_fraction`.

The focused proof is
`/tmp/anvil-parent-cone-instance-smoke-r1/manifest.json`
(`top_parent_cone_instances = 1`, `hierarchy_parent_cone_instances = 1`,
`child_input_bindings_from_parent_cone_instances = 4`, and
`top_child_input_bindings_from_parent_cone_instances = 4`), clean in
Verilator, Yosys `synth -noabc`, and the repo-owned Yosys with-ABC
path. The repo-owned Phase 4 gate now banks this as a required coverage
fact at `/tmp/anvil-tool-matrix-phase4-hierarchy-r27/tool_matrix_report.json`
with `coverage_gaps = []` and 216/0 pass-fail in Verilator plus both
repo-owned Yosys modes.

Current HEAD has broadened that helper source beyond the original
parent-composed child-input seam. Helper outputs can now feed direct
unregistered sibling routes, direct registered sibling-route D inputs,
registered parent-composed child-input D cones, and parent-output
composition. The direct sibling helper proof is
`cargo test hierarchy_sibling_routes_can_use_helper_instances`; it
requires registered helper counters to stay zero while
`child_input_bindings_from_parent_cone_instances > 0`,
`parent_cone_instance_child_input_binding_fraction > 0.0`,
`top_parent_cone_instance_child_input_binding_fraction > 0.0`, and
helper instances are present beyond the planned child slots.

### Local parent flops are a separate hierarchy state axis
The next Phase 4 step deliberately does not overload leaf `flop_prob`.
Hierarchy parent state is controlled by its own knob,
`hierarchy_parent_flop_prob`, because the parent layer is a different
structural axis from leaf-module sequential richness. The default is
`0.0`, which preserves the previously banked combinational hierarchy
surface; setting it non-zero lets parent output cones and
parent-composed child-input cones emit local parent flops.

The important invariant is the same one used for sequential
descendants: `clk` and `rst_n` are structural, not decorative. A parent
module reserves those control ports when local parent state is possible,
but the emitter only exposes them when the module actually carries
local flops or sequential descendants. Pure comb-only modules remain
free of control ports.

The implementation reuses the normal cone/flop worklist machinery.
While building a hierarchy parent cone, the generator temporarily maps
flop rolls to `KnobId::HierarchyParentFlopProb`, then drains the parent
flop worklist before finalization. That keeps telemetry honest: leaf
flop attempts still count as `flop_prob`, while parent-state attempts
count as `hierarchy_parent_flop_prob`.

Metrics now expose this state surface directly:
`hierarchy_parent_local_flops`,
`internal_module_occurrences_with_local_flops`, `top_local_flops`,
`child_input_bindings_from_parent_flops`,
`parent_flop_child_input_binding_fraction`, and
`top_parent_flop_child_input_binding_fraction`.
The focused proof is
`/tmp/anvil-hier-parent-state-smoke-r1/manifest.json`
(`hierarchy_parent_local_flops = 8`, `top_local_flops = 8`,
`top_clock_inputs = 1`, `top_reset_inputs = 1`,
`child_input_bindings_from_parent_flops = 1`), clean in Verilator,
Yosys `synth -noabc`, and the repo-owned Yosys with-ABC path. The
repo-owned Phase 4 gate now also banks this as a required coverage fact
at `/tmp/anvil-tool-matrix-phase4-hierarchy-r27/tool_matrix_report.json`
with `coverage_gaps = []` and 216/0 pass-fail in Verilator plus both
repo-owned Yosys modes.

### Registered sibling routing is a distinct hierarchy route axis
Direct sibling routing and registered sibling routing are deliberately
separate knobs. `hierarchy_sibling_route_prob` keeps the acyclic
combinational route live: earlier child output directly feeds a later
child input. `hierarchy_registered_sibling_route_prob` adds a
parent-local flop between those endpoints. A later registered sibling
route can now also choose an earlier parent-local Q as its D source,
creating a multi-stage registered sibling chain without parent-composed
logic. That route is still
acyclic at the module-instance level, but it introduces real state in
the parent, so it must be measured as both child-input provenance and
parent-local state.

The initial implementation intentionally used one flop and no extra mux
or cone around it. That is not "good enough"; it is the smallest
signoff-clean primitive for this axis. Richer registered
child-to-child patterns build from the same invariant:
earlier child output -> parent state -> later child input, with metrics
proving the route instead of requiring SV inspection. The multi-stage
direct sibling subcase is now reported separately through
`child_input_bindings_from_registered_multistage_instance_outputs`,
`top_child_input_bindings_from_registered_multistage_instance_outputs`,
`registered_multistage_instance_output_child_input_binding_fraction`,
and
`top_registered_multistage_instance_output_child_input_binding_fraction`.

This slice exposed a real finalization gotcha: post-construction remap
passes already rewrote output drives and flop fields, but instance
input bindings were also live NodeId consumers. Once a child input
could bind to a parent-local Q node, flop merging could leave an
instance input pointing at a stale duplicate FlopQ. The fix belongs in
`ir::compact`: every partial NodeId remap now rewrites instance input
bindings too. The focused unit test covers that root cause, not only
the hierarchy symptom.

### Registered parent-composed routing is not the same as registered sibling routing
The registered sibling route proves a minimal stateful handoff:
earlier child output -> parent flop -> later child input. The next
route axis deliberately adds parent logic before that flop:
earlier child output or earlier parent route gate -> parent-local
combinational logic -> parent flop -> later child input.

This is controlled by
`hierarchy_registered_child_input_cone_prob`, not by overloading
`hierarchy_registered_sibling_route_prob` or
`hierarchy_parent_flop_prob`. The distinction matters because the
metric has to prove a different structure: the binding must pass
through a parent-local flop whose D input is itself a parent-local gate
with instance-output support.

The implementation now builds the D cone from the full available parent
source pool, with spontaneous nested flop generation disabled for this
route. It then repairs the root before allocating the final flop: the D
path must keep sibling-output support, when parent data inputs are live
it can add parent-port support, and when earlier parent flops are live
it can add a prior-Q companion to create a multi-stage registered
chain. If the repaired root is not already a substantive parent gate,
the generator wraps it in a non-collapsing XOR-with-all-ones parent
gate. That keeps the route signoff-clean and construction-time
deterministic while preserving the structural proof obligation.

Metrics now expose the route directly through
`child_input_bindings_from_registered_parent_composed_logic`,
`top_child_input_bindings_from_registered_parent_composed_logic`,
`registered_parent_composed_child_input_binding_fraction`, and
`top_registered_parent_composed_child_input_binding_fraction`. Current
HEAD also exposes the mixed registered-support subcase through
`child_input_bindings_from_registered_mixed_support`,
`top_child_input_bindings_from_registered_mixed_support`,
`registered_mixed_support_child_input_binding_fraction`, and
`top_registered_mixed_support_child_input_binding_fraction`. Current
HEAD also exposes the first multi-stage registered subcase through
`child_input_bindings_from_registered_multistage_parent_composed_logic`,
`top_child_input_bindings_from_registered_multistage_parent_composed_logic`,
`registered_multistage_parent_composed_child_input_binding_fraction`,
and
`top_registered_multistage_parent_composed_child_input_binding_fraction`.
The original focused proof is
`/tmp/anvil-hier-registered-child-input-cone-smoke-r2/manifest.json`
(`child_input_bindings_from_registered_parent_composed_logic = 3`,
`top_child_input_bindings_from_registered_parent_composed_logic = 3`,
`registered_parent_composed_child_input_binding_fraction = 0.75`,
`top_registered_parent_composed_child_input_binding_fraction = 0.75`,
`hierarchy_parent_local_flops = 3`), clean in Verilator, Yosys
`synth -noabc`, and the repo-owned Yosys with-ABC path. The repo-owned
Phase 4 gate now banks this as a required coverage fact at
`/tmp/anvil-tool-matrix-phase4-hierarchy-r27/tool_matrix_report.json`
with `coverage_gaps = []` and 216/0 pass-fail in Verilator plus both
repo-owned Yosys modes.

The focused mixed-support proof is
`/tmp/anvil-hier-registered-mixed-child-input-smoke-r1/manifest.json`
(`child_input_bindings_from_registered_mixed_support = 3`,
`top_child_input_bindings_from_registered_mixed_support = 3`,
`registered_mixed_support_child_input_binding_fraction = 0.75`), clean
in Verilator, Yosys `synth -noabc`, and the repo-owned Yosys with-ABC
path. The current-code coverage-only Phase 4 matrix probe at
`/tmp/anvil-tool-matrix-phase4-registered-mixed-r1/tool_matrix_report.json`
first banked `saw_hierarchy_registered_mixed_support_routing = true`
with `coverage_gaps = []`; the full downstream-clean `r27` bank now
carries the same fact with Verilator and both repo-owned Yosys modes.

The focused multi-stage registered proof is
`/tmp/anvil-hier-registered-multistage-child-input-smoke-r1/manifest.json`
(`child_input_bindings_from_registered_multistage_parent_composed_logic = 2`,
`top_child_input_bindings_from_registered_multistage_parent_composed_logic = 2`,
`registered_multistage_parent_composed_child_input_binding_fraction = 0.5`),
clean in Verilator, Yosys `synth -noabc`, and the repo-owned Yosys
with-ABC path. The current-code coverage-only Phase 4 matrix probe at
`/tmp/anvil-tool-matrix-phase4-registered-multistage-r1/tool_matrix_report.json`
first banked `saw_hierarchy_registered_multistage_routing = true` with
`coverage_gaps = []`; the full downstream-clean `r27` bank now carries
the same fact with Verilator and both repo-owned Yosys modes.

The focused multi-stage registered sibling proof is
`cargo test hierarchy_registered_sibling_routes_can_chain_through_parent_flops`.
It proves the direct registered sibling route can chain through earlier
parent-local Qs while keeping registered parent-composed counters at
zero. The `r27` Phase 4 matrix banks that as
`saw_hierarchy_registered_multistage_sibling_routing = true` through
the dedicated
`phase4_hier2_inst4_registered_sibling_multistage_state` scenario.

### Parent outputs can mix parent ports with child outputs
The first parent-output composition slice built output cones from child
`InstanceOutput` leaves only. That proved real parent-side logic above
children, but it left parent data inputs out of the parent output
surface.

The current parent-output builder now starts from the full parent
source pool. After module finalization, it rebuilds live pools from the
settled parent module and repairs every parent output that lost
structural child-output support. When live parent data inputs exist,
the same repair path also adds parent-port support to outputs that
otherwise only reached child outputs.

The post-final repair point matters. Cleanup can fold or replace drive
roots, so the invariant has to be checked after compaction, input
pruning, and profile enforcement have settled. The repair adds ordinary
parent gates over live nodes; it does not patch emitted SV.

Metrics expose the result as
`top_parent_port_composed_outputs`,
`hierarchy_parent_port_composed_outputs`,
`top_parent_port_composed_output_fraction`, and
`hierarchy_parent_port_composed_output_fraction`. The focused
regression is
`cargo test --test pipeline hierarchy_parent_outputs_can_mix_parent_ports_with_child_outputs`.
The repo-owned Phase 4 coverage gate now tracks this as
`saw_hierarchy_parent_port_composed_outputs`; the current-code
coverage-only matrix probe at
`/tmp/anvil-tool-matrix-phase4-parent-port-coverage-r1/tool_matrix_report.json`
first recorded `coverage_gaps = []` with that fact true. It skipped
Verilator/Yosys; the full downstream-clean Phase 4 `r27` bank now
carries the same fact with real tool validation.

### Hierarchy quality has to be visible in the numbers
The user requirement here is the right one: for hierarchy, ANVIL should
not depend on someone opening the emitted `.sv` and eyeballing whether
the composition looks plausible. The reports and manifests need to
carry enough exact facts that the result can be trusted numerically.

That is why the current hierarchy slice now has a dedicated
`DesignMetrics` layer instead of only per-module metrics and a few
coarse booleans. The current trustworthy design facts are:

- library size vs instantiated child count,
- unique-instantiated-module count and unused-library count,
- reuse / coverage ratios,
- top interface shape, including `top_clock_inputs` and
  `top_reset_inputs`,
- direct-vs-composed outputs and parent-port-composed output counts,
- control fanout to child instances,
- weighted child interface / node / flop load, and
- per-definition instantiation histograms.

The smoke at `/tmp/anvil-hier-metrics-smoke-r1` mattered because it did
more than prove the metrics serializer. It exposed two real root-cause
bugs that would have made those numbers lie:

- wrapper tops were creating shared `clk` / `rst_n` ports without
  tagging them as `Module.clock` / `Module.reset`; and
- control-port emission was using a too-local rule, so wrappers with no
  local flops could hide `clk` / `rst_n` even when those ports were
  still required by sequential descendants.

The durable rule now is exact and inductive:

- pure comb-only modules do not emit `clk` / `rst_n`;
- sequential leaves do emit `clk` / `rst_n`; and
- hierarchy parents keep `clk` / `rst_n` visible iff they carry local
  state or sequential descendants, all the way up the instantiated
  chain.

That rule is now pinned in IR helpers, validation, metrics, and the SV
emitter, plus direct regression tests for both the comb-only and
grandparent-wrapper cases.

The recursive planner widened the metrics contract too. Wrapper-only
facts were no longer enough; the numbers now have to describe the
**tree**. So `DesignMetrics` now also carries:

- `realized_min_leaf_depth`, `realized_max_leaf_depth`,
  `avg_leaf_depth`, `max_module_depth`;
- `module_defs_by_depth`, `module_occurrences_by_depth`,
  `instance_slots_by_parent_depth`;
- `avg_child_instances_by_parent_depth`,
  `min_child_instances_by_parent_depth`,
  `max_child_instances_by_parent_depth`;
- `child_instances_per_internal_module_histogram`,
  `min/avg/max_child_instances_per_internal_module`; and
- hierarchy-wide composition counters in addition to the top-only ones.

That is the current trust surface for recursive hierarchy quality: the
user should not have to inspect the `.sv` to tell whether ANVIL built
the requested tree shape.

### Literal-backed for-fold sources must be materialized before procedural part-selects
The repo-owned Phase 4 hierarchy gate exposed a real emitter defect in
the bounded procedural `for` surface.

The bad shape was not subtle:

- direct literal indexing such as `24'h86899[(i * 12) +: 12]`, and then
- an attempted blanket fix that emitted `(signal)[(i * 12) +: 12]`.

Neither is a robust answer for the downstream tools we care about.
Verilator and Yosys both rejected those forms during the hierarchy
matrix.

The correct fix is narrower and more truthful:

- keep ordinary named packed sources as `src[(i * K) +: K]`;
- but when the fold source is a constant, materialize it through a
  packed procedural temporary inside the surrounding `always_comb`;
- then index that temporary.

That preserves the intended structured surface, keeps the emitted SV
legal, and fixes the root cause instead of weakening the gate or hiding
the fold behind different syntax.

### Constant-backed slices must fold to literals, not literal indexing
The new under-instantiation hierarchy smoke exposed another emitter bug
in a different surface:

- `assign slice_26 = 20'h0[18:1];`

That is just as wrong as the old procedural literal-indexing bug, but
the right fix is even narrower here. When a `Slice` operand is a
constant, there is no need to emit a slice at all. We already know the
answer exactly.

So the deliberate rule now is:

- if the slice source is non-constant, emit the normal `src[hi:lo]` or
  `src[bit]` form;
- if the slice source is a constant, compute the sliced value in the
  emitter and print the narrower constant literal directly.

That keeps the output legal and simple, and it fixes the real cause
instead of wrapping an invalid shape in more syntax.

### The broadened Phase 4 matrix found the next runtime cost shape
After landing `num_child_instances`, the Phase 4 `tool_matrix` planning
was widened from the old "leaf count x comb/seq" wrapper sweep to four
more truthful representative profiles:

- `phase4_hier2_inst2_comb`  — exact library/instance cardinality
- `phase4_hier2_inst4_seq`   — repeated child-definition reuse
- `phase4_hier4_inst2_comb`  — under-instantiated library
- `phase4_hier4_inst4_seq`   — exact cardinality at the heavier end

That is the right coverage model for the current wrapper slice, and the
full refreshed rerun now closes cleanly at
`/tmp/anvil-tool-matrix-phase4-hierarchy-r7/tool_matrix_report.json`.
But the reruns also made the runtime story obvious: the heavy
sequential `hier4_inst4_seq` cases spend real time inside Yosys because
they elaborate/synthesize tiny wrapper tops over very large sequential
child libraries.

So the durable lesson is:

- this is a downstream cost shape, not a malformed-output bug;
- the refreshed exact / reuse / under-instantiation matrix is now
  actually banked cleanly at `r7`; and
- future Phase 4 work should keep watching those heavy sequential
  corners, because they are the place where hierarchy cost surfaces
  first even when the emitted RTL is valid.

### The recursive hierarchy gate must prove hierarchy, not quietly re-run the fattest leaf stress lane
When the Phase 4 gate was widened again to cover the newer recursive
and per-depth-branching surfaces, the first full rerun (`r8`) exposed a
different version of the same problem. The new coverage logic itself
was fine, but the recursive sequential scenarios were still borrowing
the heaviest Phase 1 motif-heavy sequential leaf profile.

That made the hierarchy gate pay for a huge amount of downstream Yosys
work that belonged to leaf stress, not to hierarchy proof. The proof was
therefore answering the right structural question with the wrong leaf
payload.

The right fix was not to drop the recursive scenarios and not to weaken
the coverage facts. The right fix was to decouple concerns:

- keep the recursive depth-2 and per-depth override profiles in the
  repo-owned Phase 4 matrix;
- keep the clean-tool requirement exactly the same; but
- switch the Phase 4 sequential hierarchy scenarios to a
  hierarchy-focused sequential leaf profile sized for hierarchy proof
  rather than Phase-1-scale leaf stress.

That is why the banked `r9` report closes quickly and honestly:

- the gate still proves wrapper exact / reuse / under-instantiation;
- it still proves recursive depth `2`;
- it still proves the per-depth override profile `0=4:4,1=2:2`;
- it still proves parent-side composition above instance outputs; and
- it no longer burns runtime re-proving the fattest leaf-stress shape
  just to answer a hierarchy question.

### Wrapped-add bounds must preserve a shifted single interval when it stays linear
The `e-graph` warning in
`/tmp/anvil-tool-matrix-phase1-real-r20/int_nodeid_e-graph_default/mod_8_0053.sv`
turned out not to be a generic "Yosys got grumpy" case. It exposed a
specific gap in ANVIL's unsigned-bounds reasoning for `GateOp::Add`.

The old logic did this:

- collect operand bounds;
- if the sum might wrap the target width, fall back to full-range.

That is safe, but it was too blunt for the real rhs shape:

- one non-exact interval (`or_22` bounded to `[0xe7, 0xff]`);
- plus exact constants (`0x0c` and `0xc4`).

In that case, the exact constants are not adding uncertainty. They are
just translating one interval around the unsigned ring. If the
translated interval still lands as one linear interval in unsigned
space, we should keep it. For the real failing case, `[0xe7, 0xff] +
0xd0 (mod 256)` becomes `[183, 207]`, which is still linear and is more
than enough to prove a 3-bit shift is always an overshift.

So the deliberate rule now is:

- if an `Add` node has exactly one non-exact interval operand and the
  rest are exact constants, combine the exact constants first;
- translate the one live interval by that exact wrapped addend; and
- keep the translated interval only when it stays linear (`start <= end`
  after modular translation), otherwise fall back to full-range.

That rule is intentionally narrow. It improves downstream cleanliness on
the real `shift >> wrapped_add` warning shape without reopening the
broader exact-set proof surface that earlier slices had to cap for
runtime reasons.

### `tool_matrix` frontier runs now use per-module checkpoints
`tool_matrix` now writes `<stem>.module-report.json` after each fully
processed module and supports `--resume`.

The resume contract is intentionally narrow:

- checkpoint reuse is allowed only when the current tool surface matches
  the checkpoint (`skip_verilator`, `skip_yosys`, `yosys_mode`);
- same-binary fast resume is allowed only when the checkpoint also
  carries a matching runtime fingerprint, a matching saved-`sv` hash,
  and a saved generator checkpoint;
- otherwise the regenerated module must still match the saved `.sv`
  text and module identity; and
- metrics are refreshed locally on resume instead of being treated as
  the reuse key.

This means resume is intentionally **byte-stable**, not "best effort".
If generator semantics change and a regenerated module no longer matches
the saved `.sv`, that old tree is evidence only; use a fresh `--out`
tree for the new semantics instead of trying to cross that boundary in
place.

That last point is important. In the real smoke proof, the saved `.sv`
matched exactly while the checkpointed metrics did not, which means
metrics are too strict a resume key even when the emitted artifact is
unchanged. The load-bearing truth for reuse is therefore the emitted
module, not the old metric blob.

The newer fast path exists to avoid replaying hundreds of already-proven
modules on the **same binary** just to reconstruct RNG state. Each
fresh checkpoint now records:

- a generator checkpoint (ChaCha stream position + next module index),
- a hash of the emitted `.sv`, and
- a fingerprint of the current `tool_matrix` binary.

When all three match, resume can restore the generator directly and
reuse the saved report without regenerating that module. If any of them
do not match, the old strict replay path stays in force. That keeps the
same byte-stable correctness bar while removing the most painful
same-build resume cost.

Older output trees without sidecars are still resumable: `--resume`
will validate the saved `.sv`, rerun the current tool surface once for
that module, and then write the new checkpoint sidecar.

Likewise, older sidecars that predate the generator-checkpoint metadata
still resume correctly; they simply pay the strict replay cost once and
are upgraded in place to the newer, faster format.

One more operational detail now matters in practice: once a proof or
cleanup change alters emitted `.sv`, an older frontier tree becomes
historical evidence only even if it was the latest live checkpoint at
the time. That happened to `/tmp/anvil-tool-matrix-phase1-real-r18`
after the rollback / compare-cleanup repairs: the tree still records a
real 372-checkpoint both-mode frontier, but current code must continue
from a fresh output tree instead of trying to "upgrade" it in place.

### Cleanup exact proofs must stay compare-aware without becoming broad again
The post-construction `fold_proven_gates` pass now follows a deliberate
split:

- the **general** cleanup exact prover stays tiny-only (small width,
  small support, small endpoint count) so it cannot reintroduce the old
  large-cone runtime blowups; but
- compare gates still get the bounded unsigned-compare proof even when
  the cone is too large for the general cleanup exact gate; and
- shift gates (`Shl` / `Shr`) may still use the **bounds-only** exact
  result even when the cone is too large for the general cleanup exact
  gate.

That split exists because "large cone" and "cheap compare tautology"
are not the same thing. A dead-selector rhs can make `x >= 0` or
`1 < dead_rhs` obviously constant even when the whole cone's endpoint
set is wider than the general cleanup exact gate allows. Likewise, a
large-endpoint rhs range can still make `2'h1 >> rhs` or `x << rhs`
obviously zero. Those compare/shift revisit paths are therefore
downstream-cleanliness exceptions worth keeping separate from the
broader exact-value cleanup budget.

### `constant_prob = 0.1`
Default chosen to prevent constants from dominating cone leaves. Real synthesis-stress workloads may want lower (≤ 0.05); aggressive pattern coverage may want higher. Revisit after first seed sweep with metrics on what fraction of generated cones survive non-triviality on the first attempt.

### `terminal_reuse_prob = 0.3`
Probability that, when a cone reaches a leaf decision and the signal pool has matching-width entries, it picks an existing pool entry rather than emitting a constant or recursing further. Higher = more sharing-like behavior even before Phase 3 explicitly turns on `share_prob`. Default is a guess; tune after Phase 1.

### `share_prob = 0.3` default
The non-leaf DAG-sharing fork is enabled by default at a modest rate. Every operand has a 30% chance of terminating at an existing pool entry rather than recursing. This is the Phase 2 guiding mode: cones are a mix of tree and DAG shapes, chosen per recursion point. Raise (0.5–0.9) for fanout-stress generation; lower (0.0–0.1) for wide-sprawling tree-ish cones. `share_prob = 0.0` does not produce *pure* trees — `pick_terminal` still reuses matching-width pool entries at forced leaves. The distinction is: `share_prob` controls *non-leaf* sharing; leaf-level reuse is always on.

### Phase 2 share-gate metric: normalize by total nodes
The first repo-owned Phase 2 gate attempt tried to prove "controlled
sharing factor" with raw `total_shared_nodes`. The real run showed that
proxy was backwards: when `share_prob` rises, ANVIL often reuses enough
existing structure that the entire graph collapses, so the *absolute*
count of shared nodes can fall even while the graph becomes more
shared. The repo-owned `tool_matrix --phase2-share-gate` therefore uses
`shared_node_fraction = total_shared_nodes / total_nodes` as the
monotonic proof metric and records node-count collapse alongside it.
Current closure proof on `/tmp/anvil-tool-matrix-phase2-share-r1`:
`0.4122 @ share_prob=0.0`, `0.4232 @ 0.3`, `0.4386 @ 0.9`, while
`avg_nodes/module` drops from `4727.56` to `3525.01` to `2117.76`.

### Phase 3 should have its own structured-surface gate
Once the `case`, `casez`, bounded `for`-fold, selectable
`Slice` / `Concat`, and variable-shift surfaces were all landed, the
remaining honest Phase 3 blocker was no longer feature breadth. It was
evidence breadth.

That shape now lives in the harness itself as `tool_matrix
--phase3-structured-gate`. The dedicated matrix covers all three live
construction strategies under `identity_mode = node-id` +
`factorization_level = e-graph`, and the report is allowed to go green
only if it proves the landed Phase 3 surfaces directly:

- priority encoder
- one-hot and encoded comb mux
- procedural `case`
- procedural `casez`
- bounded procedural `for`-fold
- one-hot and encoded flop mux
- selectable `Slice`
- selectable `Concat`
- variable shifts

The closure proof now lives at
`/tmp/anvil-tool-matrix-phase3-structured-r4/tool_matrix_report.json`
with `21` scenarios, `210` total modules, `coverage_gaps = []`, and
`210/0` pass-fail in Verilator plus both repo-owned Yosys modes.

### Semantic merge proofs also need a cone-size budget
The first real Phase 3 gate run did not fail in Yosys or Verilator. It
stalled inside `merge_equivalent_gates`, specifically
`semantic_cone_proof -> evaluate_node_under_assignment`.

The root cause was subtle but real: *small endpoint support is not a
sufficient runtime guard by itself*. A settled cone can depend on only
2 or 3 canonical leaf endpoints and still contain a very large internal
graph. Brute-forcing every assignment through that whole graph turns
compaction into a whole-cone evaluator.

The durable fix is now explicit in `src/ir/compact.rs`:

- cleanup-time exact proofs stay on their already-strict tiny-cone path
- semantic merge proofs have their own reachable-cone budget
- once that budget is exceeded, compaction falls back to the
  structural proof path instead of chasing semantic equivalence at any
  cost

That keeps the semantic merge fragment live where it is valuable while
stopping large settled cones from becoming a runtime trap.

### `gate_*_weight` defaults
3:2:1:1:1 (bitwise:arith:struct:compare:reduce). Bitwise dominates because bitwise gates are the most type-flexible and produce the widest cones. Comparisons are weighted lower because they collapse the width to 1, which limits downstream cone depth. These are gut-feel; replace with measurements when phase-1 sweeps land.

### `flop_mux_encoding_prob = 0.5`
Default chosen to give equal motif exposure to OneHot and Encoded styles across a random seed sweep. If post-synthesis metrics show that one style dominates as a bug-finding target, bias the default. The knob also allows users to run workloads stressing only one style for targeted testing.

### `flop_qfeedback_prob = 0.5`
Default 50/50. No empirical data yet. Real designs probably lean heavier on QFeedback (hold-on-no-write is far more common than zero-on-no-write), but generating the less-common pattern is precisely where random generation earns its keep. Revisit with data.

### QFeedback-in-Encoded: replace `data_0` with Q
Alternative considered: add Q as an extra (M+1)th entry encoded with the largest select value. **Rejected** because:
- It would require the sel bus to be one bit wider than `ceil(log2(M))` whenever M is a power of 2, breaking the clean "M mux entries ⇔ `ceil(log2(M))`-bit sel" invariant.
- The "slot 0 is Q" convention mirrors common RTL idioms where the zero-index / reset state is treated specially.
- It keeps M as the single knob for mux entry count across both styles.

---

## Rejected alternatives

### Annotated-EBNF runtime engine
Considered: a generic attribute-grammar interpreter that reads an annotated SV grammar at runtime and produces output. **Rejected** because:
- SV's grammar is enormous; encoding all of it is months of work for productions we will never emit.
- Threading mutable scope/driven-set/flop-worklist state through pure inherited/synthesized attributes is awkward; it really wants `&mut Context`.
- Extending the grammar engine for a new motif is comparable in effort to adding a Rust enum variant + emitter arm, with much worse error messages.

The grammar view is preserved as a *correctness argument* (every constructor preserves invariants ⇔ every production is valid under its attributes). Not as a runtime artifact.

### Oracle / reference simulator
Considered: a Rust evaluator that walks the IR with concrete input vectors and produces expected output values, used both for non-triviality filtering and for downstream tool testing. **Rejected** because:
- Doubles implementation effort.
- Introduces a second correctness question (is our interpreter LRM-correct?).
- The user's stated goal is *generation*, not building a full shadow
  simulator or tool-oracle inside `anvil`.
- Non-triviality is cheaper to enforce by dep-set tracking + structural rules; multi-vector evaluation is overkill for that use case.

That does **not** lower the output-quality bar. The generator is still
expected to emit modules that run cleanly in downstream tools.
Verilator / Yosys are external validators, not the place where
`anvil` gets to finish the job.

### `always_comb` + `case` for encoded-mux flop D

Considered for the Encoded-style flop D: emit an `always_comb` block with a `case (sel)` statement driving D. **Rejected** in favor of a chained ternary over `Eq(sel, k)` because:

- The emitter already handles `Mux` and `Eq` as ordinary `GateOp` variants; nothing new is required.
- `case` would require introducing procedural block emission (`always_comb`) and name-binding for the case target, which is a bigger scope than a uniform expression-level SV emitter.
- Synthesis tools produce the same netlist from both forms for well-formed one-cycle muxes; the readability difference only matters to a human reader.

If a future motif (e.g., FSM state encoding) genuinely requires `case`, revisit then.

This remains the right decision for **flop D** muxes even after the
Phase 3 case-mux slice landed. The new case surface is a separate
combinational block motif with its own knob (`case_mux_prob`) and its
own structured gate kind; the flop path stays expression-based and
keeps its existing chained-ternary semantics.

### Casez muxes are a separate structured surface, not a decorated case-mux

The right shape for the `casez` slice was **not** to smuggle wildcard
syntax into the existing `CaseMux` gate or to make the emitter infer
question-mark patterns from ordinary indexed arms. `case` and `casez`
exercise different frontend/elaboration paths, so the IR should say so
explicitly.

That is why the slice introduced a distinct `GateOp::CasezMux` plus its
own knob (`casez_mux_prob`). Each arm stores a constant pattern, a
constant wildcard mask, and a data node. The emitter renders those as a
procedural `always_comb casez (sel)` block; the validator enforces the
constant-pattern contract; and the exact evaluator in `ir::compact`
understands the same first-match semantics.

Generation deliberately keeps the wildcard patterns **non-overlapping**
by construction. That preserves the intended "wildcarded mux" surface
without accidentally turning the new motif into a priority-case stressor
on top of the syntax stress we actually wanted.

### Bounded unrolled logic belongs in the IR as a block, not as emitter sugar

The right shape for the statically bounded `for` slice was to model it
as its own structured combinational block, not to hope that repeated
operator trees would "look enough like a loop" in emitted SV.

That is why the slice introduced a distinct
`GateOp::ForFold { kind, trip_count, chunk_width }` plus its own knob
(`for_fold_prob`). The IR carries the fold kind (`xor` / `or` / `and` /
`add`), the exact static trip count, and the chunk width. The single
operand is a packed source bus of width `trip_count * chunk_width`.

The emitter then has one honest job: declare the target as `logic`,
emit an `always_comb begin`, initialize the accumulator, and render a
bounded `for (int i = 0; i < N; i++)` loop over
`src[(i * chunk_width) +: chunk_width]`. The validator enforces that
shape directly, and the exact evaluator in `ir::compact` evaluates the
same chunk-fold semantics.

This keeps the syntax surface real. Downstream tools see an actual
procedural bounded loop, not just an expression tree that happens to
resemble one semantically.

### Selectable Slice/Concat must be non-degenerate by construction

Making generic `Slice` / `Concat` first-class selectable shapes was not
just a matter of adding them to `pick_gate`. The naive version would
have "landed" them and then immediately lost them again:

- selectable `Slice` would often degenerate to the full-width identity
  and disappear under the peephole layer
- selectable `Concat` would sometimes degenerate to the single-operand
  identity and disappear the same way

So the right design is to make the selectable forms intentionally
non-degenerate:

- selectable `Slice` always uses a source wider than its high bit
- selectable `Concat` always partitions the output width across at
  least 2 operands

That keeps the new surface honest. We are exercising real frontend
surface area, not just incrementing counters on gates that the settled
graph will erase as trivial identities.

### Late mixed-constant cleanup after remaps

Intern-time constant folding is not enough by itself once the
post-construction cleanup passes start remapping settled graphs. A gate
that was clean when originally interned can later become something like
`1 + x + inner`, where `inner` is subsequently proven/remapped to `1`.

The right place to address that is **not** to overcomplicate
associative flattening or to relax the strict duplicate doctrine; it is
to run a small late cleanup pass on the settled graph. That is now
`fold_mixed_associative_constants` in `src/ir/compact.rs`, wired after
the posthoc associative-normalisation points. It re-aggregates
associative constants (`1 + x + 1 -> x` at width 1, `1 + x + 1 -> 2 +
x` at width 8, `3 * x * 5 -> 15 * x`, etc.) after remaps expose those
opportunities.

### M = 1 mux arm

Excluded from `pick_mux_arm_count` by design. A 1-arm mux is algebraically `sel ? data_0 : 0` (ZeroDefault) or `sel ? data_0 : Q` (QFeedback) — in either case a trivially-simplified shape that adds no motif diversity over what a simple 2-arm mux or an M=0 direct cone already covers. Allowing M=1 would bloat the generator's decision space without expanding the generated-SV distribution meaningfully.

### `#![allow(clippy::too_many_arguments)]` in `src/gen/cone.rs`

The cone-recursion helpers legitimately thread 5–8 context references (`Generator`, `Module`, `SignalPool`, `FlopWorklist`, `width`, `depth`, `exclude`, sometimes more). Packaging them into a `Ctx` struct would help readability but also forces mutable-borrow juggling that fragments the code with no semantic benefit. The lint is silenced at the module level rather than per-function to avoid the ceremony of annotating every helper. Not recommended for modules outside `gen/cone.rs`.

### Generate-then-validate (filter loop)
Considered: emit random IR with looser invariants, then run the validator and discard rejected outputs. **Rejected** because:
- Untestable bound on generation time.
- Tempts contributors to weaken constructors and rely on the validator, leading to silent correctness drift.
- Complex invariants (dep-set non-emptiness) are far more expensive to check post-hoc than to maintain incrementally.

The bounded retry in `cone::build_cone_with_retry` is the *only* exception — it exists because dep-set non-emptiness depends on terminal selection in a way that cannot always be predicted at the gate level (e.g., when all available pool entries happen to be constants). Retry budget is small (4) and falls back to accepting the last attempt.

---

## Implementation gotchas

### Reproducibility hazards
- `HashMap` iteration order is *not* stable across builds. If iteration order ever affects output, switch to `BTreeMap` or sort the keys explicitly. The current code avoids this; new contributions must too.
- `f64` non-associativity is fine for probability comparisons but never use `f64` arithmetic to compute IR fields — only RNG-driven discrete choices.
- `rand::thread_rng()` is forbidden everywhere. All randomness flows from the seeded `ChaCha8Rng` in the `Generator`.

### IR arena indexing
`NodeId` is `u32`. We use `Vec<Node>` indexed by `u32`. This is fine for the foreseeable size range (modules of ≤ 10⁶ nodes). If we ever need more, the change is local to `ir/types.rs`.

Indices are stable for the lifetime of a `Module` because we only ever push, never remove. The bounded retry in `cone::build_cone_with_retry` rewinds by `Vec::truncate`, which is safe because no other code holds `NodeId`s referring to the rewound region.

### Width 0 is illegal
`Config::validate` requires `min_width >= 1`. Width-0 signals are not synthesizable and SV does not allow them. Do not relax this.

### 128-bit constant cap
Constants fit in `u128`. Modules with `max_width > 128` are technically allowed, but the constant generator emits `0` for any width ≥ 128. This is a deliberate simplification; widening the constant representation is straightforward when needed.

---

## Testing strategy notes

- **Unit tests** live in each module under `#[cfg(test)] mod tests`. Test IR constructors enforce invariants; test gate width rules; test dep-set propagation; test the emitter on hand-built IRs.
- **Integration tests** in `tests/`: cross-seed generation + IR validation + reproducibility.
- **External smoke tests** (Verilator lint, Yosys synth) are gated by env vars so they are skippable for developers without those tools. CI must enable them.

A failed external smoke test is always a generator bug. Do not "fix" by tweaking generator output — find the root invariant violation and fix it.

Same principle for the IR validator (`src/ir/validate.rs`): if it rejects real generator output, that's a generator bug. The validator is an active safety net, not a gate to be worked around. The per-gate arity + width checker added in slice `2026-04-15-0008` is specifically designed to catch width bugs in the new flop-mux assembly code, where gates are constructed by hand rather than by recursion — the most likely place for a width-arithmetic slip.

### Canonical state backreferences are validator-owned (2026-04-20)

Once `merge_equivalent_flops` started rewriting state after drain,
`Flop.id`, `Flop.q`, and `Node::FlopQ { flop, .. }` stopped being
"born correct and forgotten" fields. They are now recovery-critical
identity links that a bad renumbering pass can corrupt.

`ir::validate::validate` now owns that contract:

- every output drive root exists before root inspection;
- `m.flops[idx].id == idx`;
- `Flop.d`, `Flop.q`, and every `NodeId` stored inside `FlopMux`
  exist;
- `Flop.q` points at a `Node::FlopQ` whose backref and width match
  the owning flop; and
- every `Node::FlopQ` points at a real flop and is that flop's
  canonical `q` node.

Keep the emitter dumb. If any of these invariants fail, fix the
producer or rewrite pass; do not add emitter-side repair logic.

### Compaction now legitimises dynamic absorbing folds (2026-04-20)

Before `compact_node_ids`, the cautious rule for absorbing constants
was "only fold if the other operand is not a gate", because
`x & 0 -> 0`, `x | all_ones -> all_ones`, and `x * 0 -> 0` would
otherwise orphan a dynamic subgraph immediately.

That restriction is now obsolete. Finalisation already performs a
reachability compaction from real roots and rebuilds the dedup tables,
so these local identities are safe to fire regardless of whether the
other operand is a gate. In other words: once compaction exists, the
correctness risk is no longer "did we orphan something?" but "did we
miss an identity we should have collapsed?"

The practical consequence showed up in tool smoke:

- the remaining seed-42 Verilator `UNSIGNED` / `CMPCONST` warnings
  were not tool quirks;
- they were missed IR-local tautologies; and
- the right fix was to strengthen the rewrite ladder
  (absorbing folds, unsigned boundary comparisons, const-selector
  muxes), not to suppress or special-case Verilator.

This is the pattern to keep following for the NodeId-identity roadmap:
when equivalent local forms are discovered in emitted SV, first ask
whether they should have already become the same node in the IR.

### Signoff-quality and downstream-tool exercise are not competing goals (2026-04-20, refined 2026-04-26)

The user clarified the product direction explicitly, and later refined
the terminology around it:

- `anvil` should become a signoff-level quality random
  by-construction synthesizable RTL generator;
- generated HDL artifacts should be accepted by downstream HDL
  consumers by default; and
- `anvil` corpora should still be rich enough to exercise parsers,
  elaborators, RTL compilers, linters, simulators, synthesizers, and
  similar consumers.

Those statements are compatible. The project is **not** trying to expose
tool bugs by emitting junk, malformed syntax, or semantically dubious
RTL. The downstream-tool exercise value comes from breadth, interaction
richness, factorization pressure, stateful motifs, hierarchy, memories,
and other legal-but-hard combinations that downstream HDL consumers
should accept. Verilator and Yosys are repository validation tools for
that acceptance promise, not the only product targets.

When choosing between slices, prefer work that strengthens one of these
two axes without regressing the other:

1. broader / harder legal design space; or
2. stronger confidence that generated output is clean and robust in
   downstream HDL consumers.

### Purpose terminology clarification (2026-04-26)

The user clarified the wording around ANVIL's purpose:

- avoid calling ANVIL "constrained-random" unless that term is
  explicitly redefined away from SystemVerilog/UVM-style user-authored
  constraints or solver-driven randomization;
- the preferred short description is **random by-construction
  synthesizable SystemVerilog RTL generator**;
- ANVIL targets generated HDL artifacts that downstream consumers can
  accept: parsers, elaborators, RTL compilers, linters, simulators,
  synthesizers, and related tools;
- Verilator and Yosys are repository validation tools for syntax,
  elaboration/lint, and synthesis acceptability, not the only product
  targets; and
- ANVIL-generated corpora can still be used to stress downstream tools,
  but that is a use of the legal generated artifacts, not a license to
  describe ANVIL as primarily a malformed-input fuzzer or generic
  toolchain stress tester.

Follow-up gotcha (2026-04-27): package metadata is part of that same
terminology surface. `Cargo.toml` must not keep stale
`constrained-random` wording after README, Rustdoc, and mdBook text have
been corrected; Cargo metadata is visible to tooling and cold-start
readers before they open the longer docs.

### Verbatim user doctrine: structure over intended functionality (2026-04-20)

The following user guidance is intentionally logged **verbatim** because
it is doctrinal and should steer future implementation choices:

> Let's be clear. Generating module by recursively generating fanin cones of its outputs, mechanically means that the resulting functionality will be gibberish but that's not the point. Having functioning behavior makes no sense here. For some modules, we might get some usable functionality but that's not the goal. The ultimate goal is to be able to generate synthesable legit RTL code that downstream tools (parser, synthesizer, linter, ...) can ingest.
>
> My construction we are not aiming at functionality but at structure, capiche.
>
> ANVIL will be able to create complex to very complex synthesizable RTL code.
>
> Any functionally correct synthesizable RTL code is undistinguishable from an functionally incorrect or even gibberish code at first sight, to ensure function correctioness one need functonal verification which needs to match a specification against a RTL module.
>
> So no one can tell at first glance whether a RTL is gibberish or functionally correct with a specification, meaning for most of what will be generated, function correctness is not the goal and can't be by construction.
>
> But they are features that will create functionally correct blocks.

Operational consequence: optimize ANVIL primarily for structural
legitimacy, synthesizability, complexity, and downstream-tool
ingestibility. Treat whole-module function correctness as out of scope
unless a feature introduces a local block motif whose own behavior is
well-defined by construction.

### Broader artifact-family mandate (2026-04-20)

The user then broadened the scope again and explicitly corrected one
important boundary:

> It might sound contradictory but in addition to what's already
> described in the roadmap, book and live docs, I think it would good
> to include support for such things in the roadmap, book and live docs
> in order for ANVIL to be able address a lot more types of SV files
> formats as output. Being able to generate various types of pseudo
> random files for various types on downstream consumers would be a
> great plus, I think.
>
> In fine, I want ANVIL to be able accurately and precisely address the
> initial request, in full.
>
> ANVIL shall be the go to tool for everything (pseudo random) HDL
> generation related thing.
>
> I don't think this contradict the current roadmap of AMVIL that much,
> it is just that we are broadening the type HDL outputs we can target.
>
> As you wrote it clearly above, right now ANVIL is still a "leaf-module
> typed circuit generator", I agree.
>
> We need to start somewhere, but that is not the end goal.
>
> So we need to be able to embrass more output artifact types.
>
> This "valid-by-construction synthesizable lane” is still valid, and
> it will stay that way!
>
> We are just generating more types of valid-by-construction
> synthesizable artifacts.

Operational consequence:

- the current leaf-module typed circuit generator is now explicitly the
  **first artifact family**, not the whole product;
- future broadening still stays inside the
  **valid-by-construction synthesizable** contract;
- the first requested additions are oracle-backed micro-design corpora,
  source-level parameter / hierarchy / package IR, and explicit
  expected-facts manifests; and
- an earlier idea of broadening via invalid/reject corpora is **not**
  the adopted direction for ANVIL after the user's correction above.

This is a real scope change for planning, not a soft aspiration. The
roadmap now needs explicit phases for these broader synthesizable
artifact families.

### Repo-owned tool matrix harness (2026-04-20)

The "no hidden bias" / "exercise all axes" doctrine now has an
executable first form in the repo: `src/bin/tool_matrix.rs`.

The design choices for this harness are deliberate:

- it is a Rust binary in-repo, not an external shell script, so it can
  reuse `Config`, `Generator`, metrics, and manifest formats directly;
- it uses a **curated matrix**, not one giant Cartesian product, so the
  sweep stays fast enough to run routinely while still covering the
  load-bearing axes:
  - interleaved ladder sweep across `relaxed` plus every
    `factorization_level` rung,
  - strategy sweep across `sequential` / `shuffled` / `interleaved`,
  - a share-heavy comb-only profile,
  - a motif-heavy sequential profile;
- it reuses structural metrics as the coverage surface instead of
  inventing a second observability stack; gate kinds, block counters,
  and knob roll attempts/fires already tell us whether a scenario
  actually exercised what it claimed to stress; and
- it exits non-zero on downstream-tool failures because the point is to
  surface generator bugs, not to produce a pretty report while quietly
  accepting red runs.

The first smoke run after landing the harness was immediately useful:
it found one real emitter bug (`logic[0:0]`-style scalar slice
emission) and, after that fix, reduced the remaining failures to the
warning-cleanliness bucket (`CMPCONST` / `UNSIGNED` under Verilator).
That is exactly the intended feedback loop for the tool-clean
industrialization lane.

### Comparison warning-cleanliness is partly a generator concern, not only a factorization concern

The follow-up `tool_matrix` slice made an important distinction
explicit in code: obviously-constant unsigned comparisons are not just
"optional peephole opportunities". They are also by-construction
tool-cleanliness hazards.

That means ANVIL now has an **always-on generator-side proof path** for
comparisons in `src/gen/cone.rs`, independent of
`identity_mode` / `factorization_level`. If the generator can already
prove that a comparison is constant, it emits the constant directly
instead of relying on the factorization ladder to clean the shape up
later.

Current proof layers:

- conservative unsigned bounds for easy local identities (`x & 0 = 0`,
  `x | all_ones = all_ones`, `x * 0 = 0`, overshift-to-zero,
  select-known muxes, etc.);
- exact finite-set reasoning for comparison operands up to 8 bits
  wide; and
- replicated-concat correlation handling for shapes like `{N{bit}}`,
  so repeated copies of the same leaf are not treated as independent
  free variables during the proof.

This is intentionally narrower than full semantic factorization: it is
there to keep emitted RTL cleaner across *all* identity/factorization
modes, including `relaxed` and low rungs like `none` / `cse`.

Two implementation refinements became load-bearing once the real
`--phase1-gate` run started surfacing concrete warning files:

- **Exact proof must short-circuit once the result is already forced.**
  A small-width node can depend on a wider cone through `Slice`, so
  "walk every operand recursively until all are exact" is too blunt. If
  an exact prefix has already forced the result, the helper must stop:
  `6'h16 | 6'h39 | tail` at width 6 is already `6'h3f`; `2'h1 * 2'h2 *
  2'h2 * tail` at width 2 is already `0`; `x ^ x` is already `0`; and
  `x <= x` is already `1`. Letting the proof recurse into an irrelevant
  non-exact tail just turns an exact fact into an unnecessary `None`.
- **The small finite-set engine and the settled-graph exact-value
  engine need the same short-circuit doctrine.** The first catches
  narrow local cones directly; the second matters because
  `node_unsigned_bounds` asks "is this gate already exact?" before it
  falls back to interval reasoning. If only one engine gets the
  shortcut, the other can still miss exactly the same downstream
  warning.

Another refinement became necessary once the real `int_nodeid_cse`
frontier hit a correlation-heavy one-hot-mux cone: **exact finite-set
reasoning must also be budgeted.** The helper now carries a shared work
budget and memoizes both exact results and "unknown" results, so it can
still prove small exact facts on narrow cones without turning itself
into an exponential runtime trap on shared cartesian searches. The
durable contract is "prove what is cheap and crisp; otherwise return
`None` and fall back to the cheaper proof layers."

The next fresh-current-code `operand-unique` frontier made one more
refinement necessary: **budget alone is not a good enough admission
rule.** Even a budgeted proof can still waste generator time if ANVIL
keeps entering exact finite-set reasoning on larger shared cones whose
endpoint support is already beyond the intended proof domain.

So the contract is now sharper:

- exact finite-set reasoning is for **small width and small endpoint
  support**, not just small width; and
- the current support cap is **3 canonical leaf endpoints**.

That support cap applies both to `prove_node_exact_value` on one cone
and to the combined endpoint set used by comparison folding. This keeps
the proof useful where it is strongest, while making larger shared
cones stay on the cheaper proof layers instead of burning CPU proving
finite-set facts that are not load-bearing for cleanliness.

The first fresh-current-code both-mode rerun exposed a second, more
basic compare-cleanliness gap: **the cheap proof layer must know a few
arithmetic reflexive identities too, not only comparison tautologies.**

The concrete failing shape was:

- `sub_16 = mul_17 - mul_17`
- `and_49 = mul_18 & mul_18 & sub_16`
- `lt_0 = add_13 < and_49`

Verilator quite reasonably warned that the unsigned comparison was
constant. The missing fact was just `x - x = 0`.

The exact finite-set engine was not the right place to rely on for this
because it may legitimately decline a cone. The **cheap** layer has to
know it too. So `exact_gate_value` and `node_unsigned_bounds` now both
encode reflexive subtraction directly. Durable rule:

- local exact/bounds proofs should carry the cheapest algebraic facts
  that directly prevent mainstream tool warnings, even when those facts
  do not require the heavier finite-set prover at all.

### Downstream warnings are a generator bug, and the final graph gets a last proof pass

The follow-up slice closed the remaining `tool_matrix` warning bucket by
making two policy changes explicit in code.

First, ANVIL now runs a post-construction proof-cleanup pass in
`src/ir/compact.rs` (`fold_proven_gates`) after cone construction and
again after the sharing/remap passes settle. The key distinction is
timing: some exact proofs are not visible when a gate is first
constructed, but become visible later once remaps, merges, or other
local simplifications have changed the graph that the gate actually
sees. That pass:

- rewrites any gate whose current cone is provably exact into a
  constant in place; and
- rewires muxes whose selector is now provably constant.

One more settled-graph wrinkle showed up immediately afterwards:
remap-producing post-construction passes can reintroduce legal
associative nestings **after** the intern-time Associative layer has
already done its work. The live example was a width-1 `Add` whose
operand was later remapped to another width-1 `Add`, leaving
`nested_associative_operand_count = 1` at default knobs even though
flattening was still legal under the strict duplicate policy.

The durable rule is: **any pass that can change which already-built
node an operand points at may need to restore associative normal form
afterwards.** ANVIL now does that with
`flatten_posthoc_associative_gates(&mut Module)` in `src/ir/compact.rs`
after `fold_proven_gates` and after `merge_equivalent_gates`. The pass
uses the same duplicate policy as the intern-time Associative layer:
`And`/`Or` dedup, `Xor` pair-cancels, `Add`/`Mul` flatten only when the
flat list would still be legal at the current
`operand_duplication_rate`.

The proof stack now has three complementary layers:

- construction-time local proofs in `src/gen/cone.rs`,
- post-construction exact-value cleanup on the settled graph, and
- bounded semantic identity / sharing for the `e-graph` fragment.

One more durable constraint became explicit when the fresh current-code
`nodeid-cse` frontier stalled during resume: sampling the live process
showed the hotspot in `ir::compact::fold_proven_gates` /
`semantic_exact_value`, not in Yosys or Verilator. The settled-graph
cleanup prover is therefore intentionally **stricter** than the
generator-side semantic-sharing passes. Today it only brute-forces cones
that are all of:

- at most 8 bits wide;
- at most 10 total support bits; and
- at most 3 canonical leaf endpoints.

If a cone falls outside that tiny cleanup surface, the pass memoizes
`None` immediately and moves on. Durable rule: late proof-cleanup exists
to scrub obvious constants for downstream-tool cleanliness, not to widen
the main identity/factorization contract at arbitrary runtime cost.

### Narrow slices of wide cones are still narrow proof domains (2026-04-20)

The next live warning bucket made a subtle point painfully concrete:
the small finite-set engine is allowed to be width-bounded, but it is
not allowed to treat a narrow `Slice` result as "unprovable" just
because the source cone is wider than 8 bits. A 14-bit or 25-bit source
feeding an 8-bit slice still yields an 8-bit proof problem.

The durable implementation rule is:

- if a narrow slice's source is already exact, use that exact value;
- otherwise, if the source is too wide for direct enumeration, fall
  back to the full narrow output domain instead of returning `None`.

That fallback is conservative but still useful: it keeps later local
operations (`Or` with forcing constants, exact shifts, subtract-small,
dynamic overshift) in the proof path, which is enough to recover exact
facts like "this `Shr` is forced to zero". Returning `None` too early
throws away that whole proof chain.

One more shift-specific wrinkle showed up later in the fresh
`associative` frontier: some rhs cones are too large for the general
small-support exact enumerator **as whole cones**, but still have a
tiny value domain because they are really just boolean-mask arithmetic
(`{8{bit}} + constant`, similar patterns). The durable rule is:

- shift overshift proofs may use a tiny-domain rhs fallback for narrow
  boolean-mask arithmetic, even when the whole cone is too large for
  the main exact small-set engine.

That fallback stays intentionally narrow: width <= 8, tiny result-set
cap, and only a few structural forms. It exists to suppress pointless
dynamic shifts whose rhs is semantically always oversized, not to
replace the main semantic-sharing machinery.

### Finalisation liveness must be output-rooted, not flop-table-rooted (2026-04-20)

The dead-register Verilator warning exposed a mismatch between Rule-18
gate liveness and sequential liveness. The old compaction pass rooted
every `flop.q` unconditionally because the flop existed in `m.flops`.
That preserved dead state even when no output cone, live flop D-cone,
or other retained logic ever consumed that Q.

The durable rule is:

- start final liveness from output drive-roots;
- when the walk reaches a live `Node::FlopQ`, mark the owning flop
  live and pull in its `d` / mux-held nodes;
- drop any flop whose `Q` is never reached by the live graph.

That is the sequential analogue of Rule 18: state is live because it is
observed by retained logic, not because it once got allocated.

### Post-remap identity cannot violate strict Add/Mul duplicate policy (2026-04-20)

Late proof / sharing passes operate after construction, so they can
collapse two previously-distinct child cones to one canonical node.
That is fine in general, but under strict `operand_duplication_rate`
the final emitted IR is still not allowed to contain duplicate
`NodeId`s inside an `Add` or `Mul` operand list.

The durable rule is therefore stronger than "the remap is semantically
valid":

- a candidate remap is only acceptable if every strict `Add` / `Mul`
  consumer remains duplicate-free after the rewrite.

ANVIL now enforces that by pruning duplicate-introducing remaps before
they are applied in `fold_proven_gates` and `merge_equivalent_gates`.
This preserves the default "zero duplicate operands" doctrine without
backing away from late exact-value cleanup or bounded semantic sharing.

### Evidence slices are legitimate when the real gate frontier moves materially (2026-04-20)

Not every important slice changes code. Once the user set the quality
bar as "no warnings or errors from Verilator and Yosys", the real
`tool_matrix --phase1-gate` run became part of the implementation loop,
not just a nice-to-have afterthought.

That means there is a legitimate kind of slice whose output is:

- a materially advanced real downstream-clean frontier,
- recorded precisely (scenario names, module counts, command line), and
- committed into the live docs so the next session does not restart the
  same evidence climb from memory or vibes.

The key is that the checkpoint must be **material**, not cosmetic. In
this session, moving from the earlier 76-module clean frontier to 246
clean modules across multiple identity/factorization lanes cleared that
bar easily. It changed what we know about the repaired generator.

So the durable rule is:

- if a long real gate run advances the proven clean frontier
  substantially, it is acceptable to checkpoint that evidence as its own
  slice, even if no code changed in that commit.

That rule matters for crash recovery too, which is exactly why the
commit workflow is strict in the first place.

Second, the repo-owned downstream harness now treats warnings as
failures rather than as "successful but noisy" runs. `tool_matrix`
scans tool output for warning markers and marks the invocation failed
even if the process exit status is zero. The Yosys script was also
tightened from `synth` to `synth -noabc` so the matrix does not accept a
self-inflicted ABC combinational-network warning and then pretend the
run was clean.

This is a durable project rule now: for repo-owned Verilator/Yosys
evidence, "green" means no errors and no warnings.

### The 1000-module Phase 1 gate should be a first-class harness mode

Once the smoke matrix was green, the next missing piece was not more
doctrine. It was executable ergonomics. The Phase 1 exit criterion had
become "run the same harness, but remember to multiply the scenario
count, pick a large enough `--modules-per-scenario`, and also remember
that coverage gaps must fail."

That shape now lives in the harness itself as `tool_matrix
--phase1-gate`:

- it auto-enables coverage-gap failure; and
- it raises `modules_per_scenario` high enough to generate at least
  1000 modules total across the built-in scenario set.

The deliberate choice here is to encode the gate in the repo-owned tool
rather than leaving the phase-exit arithmetic in roadmap prose. When a
quality gate matters, the project should be able to invoke it directly.

### Codebase suitability assessment: four steering gaps (2026-04-20)

The short answer to "is the existing codebase suited to the goal?" is:
**yes, as a foundation; no, not yet as a finished system**.

Why "yes": the architecture already matches the problem. `gen` builds a
typed IR instead of text, `Module::intern_gate` is a single
construction-time chokepoint for combinational identity,
`ir::compact` owns post-drain cleanup and state-finalisation work,
`validate` owns the invariant contract, `config` keeps the control
surface explicit, and the SV emitter stays deliberately dumb. That is
the right shape for a signoff-grade legal-RTL generator.

What still needs to stay explicit:

1. **Feature breadth grows above the leaf kernel, not by muddying it.**
   `src/gen/module.rs` is the leaf-module kernel. Hierarchy should land
   as a higher layer (planned `src/gen/hierarchy.rs`), not as ad hoc
   special cases in the leaf path. Likewise, memories/FSMs/aggregates
   should become first-class motifs or module-level generators, not
   emitter tricks.
2. **`NodeId`-as-identity must keep expanding through the IR, not via
   emitter magic.** Today's live coverage is normalized combinational
   identity plus a conservative endpoint-preserving state merge.
   Future work is stronger state identity across richer state graphs and
   later hierarchical/block identity, but it must stay faithful to the
   doctrine: same identity requires proven same functionality with
   respect to the same canonical leaf variables. Keep
   `--identity-mode` as the coarse on/off switch and
   `--factorization-level` as the finer dial; construction strategy
   must stay orthogonal.
3. **Tool cleanliness must be industrialized.** Seed 42 being clean is
   good news, not a stopping point. Each new motif/category/knob needs
   matrixed Verilator/Yosys evidence, retained seed+config
   counterexamples, and root-cause fixes at the IR/generator layer
   rather than warning suppressions.
4. **Structure-first doctrine remains load-bearing.** Absent a
   specification, whole-module functional intent is not the optimization
   target. Invest in legal interaction surfaces, factorization
   pressure, hierarchy, and stateful richness. Functionally correct
   local blocks are welcome; a bundled whole-module oracle is not the
   direction.

### Endpoint-preserving functional doctrine for state identity (2026-04-20)

The user clarified the intended meaning of state equality sharply:

- two fanin cones may **not** share one `NodeId` if they do not have the
  same leaf endpoints as variables;
- the relevant variables are the canonical leaf endpoints: primary
  inputs and/or flop `Q` outputs; and
- the goal is equality by proven same functionality with respect to
  those same endpoints, not equality by visual resemblance or by
  matching graph skeleton alone.

Operational consequence:

- `merge_equivalent_flops` now uses a conservative leaf-aware proof form
  over the already-normalized IR rather than exact `d: NodeId`;
- that proof form now includes a bounded semantic check for
  small-support cones, so some different-shape cones can merge when
  they evaluate identically over the same canonical endpoint set; and
- any future strengthening of sequential identity must preserve the
  canonical leaf namespace. "Rename each owning `q` to SELF" is **not**
  acceptable in strict `NodeId as identity` mode, and neither is
  equating cones solely because they happen to look structurally alike.

---

## Generation-time defects observed in sample output (pending fixes)

Cataloguing real defects observed in sample module `mod_1_0000`
(3 outputs, 10-level fanin, default knobs, graph-first strategy).
These are generator bugs — not SV-emitter or validator bugs.
Enumerated here so the next session can fix them at the root.

- **Constant-select muxes.** Every `wN = (2'h2 == 2'hK) ? ... : ...`
  in the sample is a mux whose select is a *literal* comparison of
  two literals. The select folds at elaboration. Root cause: the
  encoded-mux assembler feeds the select-side recursion through
  the same `pick_terminal` path that can terminate on a constant
  leaf, and the one-hot-mux assembler similarly accepts a constant
  for the per-arm select bit. Fix: in mux-select position, forbid
  constant termination — require a non-constant signal source.
- **N-arity self-cancellation.** `w_21 = i_2 ^ i_2 ^ i_2 ^ i_2 = 0`.
  The N-arity operator expansion re-picks the same pool entry for
  every operand, and `Xor` of even repetitions is zero. Fix: the
  anti-collapse check must look at operand *multiset equality* for
  idempotent / self-inverse operators, not just dep-set
  non-emptiness. (And for `And`/`Or` the same issue produces
  `x & x & x = x` which is a structural collapse, not a zero, but
  still a motif violation.)
- **Coefficient width overflow.** `1'h6` appears — a 6 encoded in a
  1-bit literal, which truncates to 0. Root cause: the linear-
  combination coefficient generator picks the coefficient value
  independently of the operand width. Fix: clamp the coefficient to
  `bits ≤ operand_width`, or widen the literal to the operand width
  and let the top bits be real.
- **Dead wires.** `w_17`, `w_26`, `w_27`, `w_29` are declared and
  assigned but never read. Graph-first speculative pool growth is
  the source; Rule 18 (proposed) addresses this.
- **Stranded flop.** `r_3 <= r_3` — a flop whose D is its own Q and
  whose Q is never read. A no-op. Rule 18 covers this too, as long
  as "consumer" is defined to exclude the flop's own Q feedback.
- **Structurally-identical one-hot arms.** `w_8`, `w_10`, `w_12`,
  `w_14` are all `{w_6,...} & w_5`, meaning four arms of the one-
  hot mux have the same per-arm product. OR-reducing identical
  arms collapses to just the arm value. Fix: in one-hot assembly,
  require per-arm *data* distinctness (or require the per-arm
  select to differ; the current issue is that all arms share the
  same broadcast select bit `w_6`).

All six share a theme the user articulated: signals are being
created without a *reason to exist*. The fixes are three-category:
(1) tighten anti-collapse (operand-multiset check); (2) position-
dependent leaf rules (no const in mux select); (3) width-aware
constant generation. Rule 18 addresses the orthogonal
"unconsumed output" axis.

---

## File-level conventions

- Every Rust source file starts with a doc comment explaining its scope.
- Public types in `ir/types.rs` and `config.rs` get full doc comments. Internal helpers do not need them.
- No multi-paragraph docstrings. One short line; if more is needed, link to `book/`.
- No comments explaining *what* the code does; only *why* when non-obvious.

---

## Construction-time CSE via `Module::intern_gate` (2026-04-15 → 2026-04-16)

Design decision: *all* `Node::Gate` and `Node::Constant` creation is routed through two inherent methods on `Module`:

```rust
pub fn intern_gate(&mut self, op, operands, width, deps) -> (NodeId, bool);
pub fn intern_constant(&mut self, width, value) -> (NodeId, bool);
```

The boolean return is `is_new`: callers that also maintain a `SignalPool` must call `pool.add` only when `is_new` is true, otherwise the pool accumulates duplicate entries for deduped nodes.

Rationale: we need CSE at *construction* time, not as a post-pass. Rule 21 ("AST-instance cap") uses the dedup tables on `Module` as the single source of truth for "which NodeIds represent which expressions."

Rejected alternative: decouple the dedup table from `Module`, keep it in the generator. Rejected because the dedup is an IR-level invariant — the emitter and validator may also want to reason about it, and the tables must survive a `Module::clone()`.

### Snapshot contract with `build_cone_with_retry`

`build_cone_with_retry` rewinds state on empty-dep retries. Before the snapshot fix, it rolled back `m.nodes.truncate(snap_len)` but *not* `gate_instances` / `const_instances`. Stale entries then pointed at truncated `NodeId`s; subsequent intern calls would return a different node than the key promised (witnessed by `const_comparand_across_all_strategies_is_valid` failing at seed 2 Interleaved during the migration).

Fix: snapshot and restore `gate_instances` and `const_instances` alongside `m.nodes`, `m.flops`, pool, and worklist. The `HashMap::clone` cost is bounded by module size — measured negligible on the default knob range.

## Rule 18 "No orphan gates": α construction-time (2026-04-16)

Two enforcement paths were considered:

- **(α) Construction-time:** only create a gate when a specific consumer is already waiting for it. `build_cone` snapshots state before operand construction; on anti-collapse rejection, the snapshot is restored — operand sub-trees vanish from the IR. `process_signal_frame` (interleaved) can't snapshot per-gate because sibling frames have committed, so it delivers one of the existing operand NodeIds as the fallback instead of calling `pick_terminal` (which would create a fresh orphan-prone node).
- **(β) Emission-time tree-shake:** post-generation, compute the live set from drive-roots + flop D/Q transitive fanin, emit only that set.

Rejected β: it's a generate-then-filter step, violating the "by construction" doctrine. User-memory feedback: *"Rule-based generation, not post-hoc filtering."* α is adopted.

Corollary: GraphFirst retired. Its phase-1 speculative pool growth produced 13–27 % orphan gates per module. The variant is kept as a silent CLI alias for Interleaved for backward compat; the dedicated code path (`build_graph_first`, `grow_pool_one_unit`, `*_pool_only` helpers) is unreachable at runtime and may be removed in a future cleanup slice.

## Full factorization doctrine (2026-04-16)

User framing: **`NodeId` is the identity of an expression**; two expressions that are the same mathematically must share one NodeId, different expressions must have different NodeIds.

Implementation ladder (see `book/src/structural-rules.md` Rule 21c):

1. Syntactic CSE (Rule 21) — `(op, operands, width)` key. **Implemented.**
2. Operand-uniqueness (Rule 8 extended) — no NodeId twice in one operand list. **Implemented.**
3. Commutative normalization (Rule 21b) — sort commutative operands before interning. **Implemented.**
4. Associative flattening — flatten `(a+b)+c` to `Add(a,b,c)` when semantically safe. **Implemented.**
5. Constant folding — `x+0 → x`, all-constant evaluation, etc. **Implemented.**
6. Peephole — local algebraic / structural rewrites. **Implemented.**
7. E-graph — full semantic equivalence. **Partially implemented.**
   Default user-requested level. Today's live fragment is still bounded:
   small-support combinational cones can merge post-construction when
   they are proven equivalent over the same canonical leaf endpoints.

`FactorizationLevel::effective()` clamps user requests down to the highest implemented layer so aspirational levels don't error. Today `e-graph` remains the strongest implemented rung, but only as a bounded fragment rather than the full semantic-equivalence aspiration. Construction strategy is orthogonal: `sequential` / `shuffled` / `interleaved` decide build order, while the factorization ladder records how much of the `node-id` identity contract the current build can currently enforce/prove.

## Identity mode is orthogonal to construction strategy (2026-04-20)

User clarification that should remain durable:
**"NodeId as identity" is a mode of operation, not a cone-builder.**

That means:
- `construction_strategy` answers *how fanin cones are walked/built*
  (`sequential`, `shuffled`, `interleaved`, graph-first alias);
- factorization / identity mode answers *when two built objects are
  considered the same thing* and therefore must share one NodeId.

Implementation consequence: expose the peak-sharing / no-sharing
switch as a separate CLI axis (`--full-factorization`,
`--no-full-factorization`) rather than pretending it is another
construction strategy value. Future work on the true NodeId-as-
identity engine must preserve this separation.

## Identity mode is now a first-class typed axis (2026-04-20)

The separation above now lives in the code, not just in the docs:

- `Config` owns a new `IdentityMode` enum with `node-id`
  (default) and `relaxed`.
- `Module` mirrors both `identity_mode` and the requested
  `factorization_level`.
- The actual gating sites consult
  `effective_factorization_level()` instead of reading the raw
  ladder directly.

Design consequence:
- `identity_mode = relaxed` is the coarse hard-off switch. It
  forces the effective level to `none`, so `intern_gate` and
  `intern_constant` always allocate fresh NodeIds.
- `identity_mode = node-id` selects the full-factorization doctrine:
  `NodeId` is the identity of an expression.
- `factorization_level` is then the fine-grained implementation /
  proof-depth selector inside that doctrine. Lower rungs are useful
  diagnostic and stress modes, but they are not alternate semantics for
  `node-id`.

This is the minimum architectural move that makes the future
"NodeId as identity" engine honest: the repo can now talk about
identity mode without smuggling it through the ladder alone.

## Adversarial generation must be modeled as orthogonal axes (2026-04-20)

User clarification that should remain durable:
ANVIL must model all axes of adversarial generation explicitly and use
them efficiently during actual generation; there should be no hidden
bias toward whichever path the current implementation happens to favor.

Practically that means:
- construction strategy (`sequential`, `shuffled`, `interleaved`,
  graph-first alias) is one axis;
- identity mode (`node-id` vs `relaxed`) is another;
- factorization level is a third;
- motif/category weights, sequential density, width/depth ranges, and
  the probability knobs are additional orthogonal axes.

Implementation consequence: whenever a new generator feature lands, the
question is not only "does it work?" but also "which axis did it add,
how is that axis surfaced, how is it measured, and how do we avoid
silently under-sampling it during real workloads?"

## Stateful identity must be decided post-drain (2026-04-20)

For gates and constants, identity is knowable at intern time: the
full key exists when `intern_gate` / `intern_constant` runs.

Flops are different. `build_flop_leaf` allocates a Q leaf
immediately, but the flop's semantics are not complete until the
worklist later constructs its D-cone. So the first honest stateful
extension of "NodeId as identity" cannot be an allocation-time guess;
it has to run after drain.

Current rule: after `summarize_flop_mux_metadata`, flops are merged
iff they have the same emitted-state signature over the same canonical
leaf variables: same `width`, `reset_kind`, `reset_val`, and the same
leaf-aware D-cone proof form. Today that proof form has two rungs:

1. normalized structural proof over the already-canonicalized IR; and
2. bounded semantic proof for small-support cones (enumerate every
   endpoint assignment, key by the resulting truth table).

Construction provenance (`FlopKind`, cleared mux operand metadata) is
deliberately ignored once D exists, because emitted hardware semantics
are carried by width/reset/D-cone meaning, not by how the generator
happened to assemble them.

This is intentionally narrower than full sequential equivalence. Two
cones that happen to compute the same function but are not reduced to
the same proof form by the current ladder, or whose endpoint support is
too large for the bounded semantic check, are not merged yet. That
deeper coinductive story remains a
future slice.

## Bounded E-graph fragment for combinational identity (2026-04-20)

`merge_equivalent_gates(&mut Module)` is now the first live
post-construction combinational extension of the `e-graph` rung.

Current rule:
- gated by `identity_mode = node-id`;
- gated by effective factorization level `>= e-graph`;
- same canonical leaf endpoints are mandatory; and
- functionality may be proven either by the already-normalized
  structural proof form or by a bounded semantic truth table for
  small-support cones.

This is deliberately not the whole e-graph story. It is a bounded proof
fragment that makes the strongest mode honest today while preserving the
user's doctrine that `relaxed` remains a real no-sharing mode and that
construction strategy stays a separate axis.

## Emitter is a dumb serialiser (2026-04-16)

User-memory feedback: *"All thinking, checks, rules' enforcement ought to be done solely at the IR level. By the time you reach emission it is too late to roll back."*

Consequence: `emit::to_sv` iterates `m.nodes` in order and writes. No filtering, no reachability check, no live-set computation. Any invariant worth enforcing must be enforced at IR construction or at a `generate_leaf_module` finalization step — never at the emitter.

The safety-net audit in `generate_leaf_module` (`count_orphan_gates`) is *at the IR level* and warns on Rule 18 violations; it does not modify the IR. The emitter trusts what it is given.

## Rejected: without-replacement operand picking as the default

For And/Or/Xor/Add/Mul operand lists, operand duplicates are caught by `violates_anti_collapse` after operands are picked. A natural alternative is to pick operands *without replacement* at the source — maintain a `HashSet<NodeId>` during the per-operand loop and exclude already-picked NodeIds.

Considered and not adopted as the default because:
1. Pool sizes at default knobs are often ≤ N (the requested arity). Without-replacement falls back to "partial arity" + distribution shift.
2. Anti-collapse + rollback already gives 0 duplicates at default. The without-replacement change would save RNG cycles at the cost of a distribution shift that has no empirically measured benefit.
3. `operand_duplication_rate` is the documented knob for users who want the alternative behaviour.

Retained for reference in case a future motif benefits from it.

## Finalisation trims metadata-only and unused-bit surface (2026-04-19)

This slice locked in a small but important finalisation doctrine:
**emit what the live hardware uses, not the generator's provisional
scratch structure.**

- **Width adapters now expand to the exact target width.** The old
  non-multiple up-width adapter built an oversized replicated `Concat`
  and then sliced it back down. Functionally fine, but it manufactured
  dead high bits that lint tools quite rightly flagged. The adapter now
  builds the exact-width shape directly (`{src[rem-1:0], src, ...}`).
- **`Flop.mux` operand NodeIds are construction-time metadata, not
  emitted hardware roots.** Once `flop.d` is assembled, keeping the
  original select/data operand references around lets metadata-only
  cones survive liveness/compaction even though the emitter never reads
  them. Finalisation now keeps only the variant shape and discards
  those operand references before compaction.
- **Primary inputs are shrunk/pruned to the live bit surface.** After
  compaction, each surviving primary input is reduced to the highest bit
  any live consumer touches, and entirely unused data inputs are
  dropped from the emitted interface. This keeps Verilator from
  reporting unused input bits or dead ports.
- **Residual associative-opportunity metrics now respect duplicate
  policy.** Nested `Add`/`Mul` slots that would introduce duplicates if
  flattened are intentionally preserved at strict
  `operand_duplication_rate`; the metric now matches that semantic
  policy instead of counting those slots as "missed" flattening.

Rejected alternative: paper over the issue in the emitter with
tool-specific lint pragmas. That would hide the symptom without fixing
the IR/finalisation mismatch.

## Yosys ABC/no-ABC is now an explicit harness axis (2026-04-21)

Historically, the repo-owned Yosys smoke path settled on
`synth -noabc` because some runs with the default ABC-enabled `synth`
were reported to blow up or time out. That was useful operationally,
but it left the distinction implicit: future sessions could see one
hardcoded `-noabc` script and have no way to tell whether it was a
deliberate stability baseline, a temporary workaround, or stale cargo
cult.

`tool_matrix` now makes that choice explicit with a Yosys mode axis:

- `without-abc` — current stable baseline, still the default;
- `with-abc` — the repo-owned ABC-enabled harness path; and
- `both` — run both sub-modes per generated file and report them
  separately.

The default remains `without-abc` because that is the last known-good
repo-owned baseline. The point of adding `with-abc` and `both` is not
to silently relax warnings; it is to make the instability visible and
reproducible.

On the first small repo-owned probe, `without-abc` passed 15/15 while
the original `with-abc` path failed 14/15, not from a crash but from
ABC's `Warning: The network is combinational` line. Yosys's own `help abc`
text explains why that can happen even on sequential modules: ABC is
run on logic snippets extracted from the design, not necessarily on the
whole module as one sequential network.

The repo-owned harness now treats `with-abc` as the explicit
warning-clean script:

`synth -noabc; abc -fast; opt -fast; stat; check`

That keeps ABC in the loop while avoiding the default `scorr`-based ABC
script that was producing the non-actionable warning bucket. The
follow-up small `--yosys-mode both` probe is now clean in both
sub-modes: `without-abc = 15/15 pass`, `with-abc = 15/15 pass`.
