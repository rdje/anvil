# CAPABILITY-BREADTH-EXPANSION: more SV-2017/2023 up-opts + Mealy FSM outputs

## Metadata

- Tree ID: `CAPABILITY-BREADTH-EXPANSION`
- Status: `active`
- Roadmap lane: `Capability / breadth — high value-per-effort RTL surface additions (north star)`
- Created: `2026-06-17`
- Last updated: `2026-08-01`
- Owner: repo-local workflow

## Goal

Add the highest "user-visible value per effort" capability breadth, in three
strands (a **third** was added `2026-08-01` at `.3`, decision
[`0044`](../decisions/0044-capability-breadth-unique-priority-case-qualifiers.md);
strands 1 and 2 below are unchanged):

1. **More SV-2017/2023 up-opts** — today only the IEEE 1800-2023 `union soft`
   overlay ships (`SV-VERSION-TARGETING` / decision `0010`). Add more
   version-distinctive, default-off, **proven** up-opts continuing that pattern:
   `enum` / `typedef`, packed multidimensional arrays, and other 2017/2023
   constructs — each gated on `sv_version`, down-gating below its standard, and
   proven downstream-clean in the matching tool mode.
2. **Mealy FSM outputs** — the Phase-6 FSM motif emits **Moore** outputs only;
   add **Mealy** outputs (outputs that also depend on the current input), as a
   default-off extension of the existing `Fsm` block + emitter.
3. **Synthesizable breadth constructs** (added `2026-08-01`, decision `0044`) —
   2012-legal synthesizable constructs the emitter does not produce at all. This
   is a **different question** from strand 1, not a replacement for it: strand 1
   asks *which post-2012 construct is distinctive*, strand 3 asks *which
   2012-legal construct is missing*. First pick: the `unique` / `priority`
   **case qualifiers** on `CaseMux` / `CasezMux`.

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
  Goal: `Three breadth strands — more SV-2017/2023 up-opts (continuing the union soft / SV-VERSION-TARGETING pattern), Mealy FSM outputs (extending the Phase-6 Moore-only Fsm), and 2012-legal synthesizable breadth constructs the emitter does not produce at all (added 2026-08-01, decision 0044) — each default-off, proven, API-selectable + introspectable.`
  Children: `CAPABILITY-BREADTH-EXPANSION.1`, `CAPABILITY-BREADTH-EXPANSION.2`, `CAPABILITY-BREADTH-EXPANSION.3`, `CAPABILITY-BREADTH-EXPANSION.4` (itself `.4a` design-detail + `.4b` impl)

- ID: `CAPABILITY-BREADTH-EXPANSION.1`
  Status: `pending`
  Goal: `SV up-opt breadth — design/decision leaf (ADR, no code): pick the NEXT version-distinctive up-opt after union soft (candidates: enum/typedef, packed multidimensional arrays, other 2017/2023 constructs), grounded in a fresh empirical probe (Verilator matching --language mode + Yosys both modes + Icarus + iverilog sim-equiv where applicable) and the local SV LRM cache for legality; pin its own default-off knob + sv_version gate + down-gate fallback + the num_emitted_* metric + the --sv-version-gate (or dedicated) coverage fact + the MCP selectability/queryability (decision 0017). Reuses src/ir/soft_union.rs + the SvVersion::permits gate as the template. Record as the next decision record + pre-split impl.`
  Acceptance: `A decision record + a tree entry pinning the chosen up-opt, the probe evidence, the knob/gate/metric, and the MCP surface; docs-only; INDEX + this tree + docs/TASK_TREE.md updated.`
  Verification: `pending` — **deferred-not-retired**, twice on measurement: the `.2a` probe (`2026-06-22`) and the pre-`.1` probe (`2026-08-01`) both found the version-distinctive seam close to exhausted after `union soft`. `.3` (decision `0044`) opened the *synthesizable-breadth* question as a THIRD strand rather than rewriting this leaf's goal, so the question this leaf asks — *which post-2012 construct is both synthesizable and distinctive?* — is still asked, and is still the only place in the repo that asks it.
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

- ID: `CAPABILITY-BREADTH-EXPANSION.3`
  Status: `done`
  Goal: `Synthesizable-breadth strand — design/decision leaf (ADR, no code): open a THIRD strand for 2012-legal synthesizable constructs the emitter does not produce, and pick the first one. Ground it in a fresh empirical probe (Verilator matching --language modes + Yosys both modes + Icarus + the --assert runtime violation checker) and in the local SV LRM cache for legality; pin the construct, the candidate set, the default-off knobs, the metrics, the coverage facts, the per-qualifier downstream tool plans, and the MCP selectability/queryability (decision 0017). Record the scope call — a new strand rather than a rewrite of .1 — explicitly, since it is the one judgement that could have gone wrong silently.`
  Acceptance: `A decision record + a tree entry pinning the chosen construct, the probe evidence, the knobs/gate/metrics, and the MCP surface; docs-only; INDEX + this tree + docs/TASK_TREE.md + ROADMAP updated.`
  Verification: `done — decision 0044 written (KM answers: front-matter). Empirical probe banked in the ADR, Verilator 5.046 / Yosys 0.64 / Icarus 13.0: (a) by-construction argument read off src/emit/sv.rs:800-806 (default: always emitted ⇒ FULL), src/emit/sv.rs:712-721 (CaseMux labels SW'd{i} ⇒ PARALLEL) and src/gen/cone/motifs.rs:832-841 (build_casez_patterns is the SOLE casez pattern source; arm i carries care-value i with one don't-care LSB ⇒ PARALLEL); (b) corpus measurement — 50,761 case/casez blocks over 120 modules × 3 construction strategies, 100 % FULL + PARALLEL, 0 nested, 0 unparsed, 0 violations; plus 130 FSM blocks (106 nested, symbolic localparam labels) 100 % FULL + PARALLEL; (c) the checker proven non-vacuous on a hand-written overlapping casez + default-less case (exits 1); (d) downstream ON-vs-OFF over 24 gate-shaped modules / 169 qualified blocks — verilator --lint-only (repo-owned argv) 0/0, verilator -Wall × 1800-2012/2017/2023 68 vs 68 (Δ=0), Yosys both modes 0 fail / 0 msg, synthesized cell counts identical 24/24, iverilog -g2012 exit 0; (e) runtime — verilator --binary --assert reports ZERO unique/priority violations on a real CaseMux module (exhaustive sweep) and a real CasezMux module with unique casez on all 5 blocks (20,000 vectors), output-identical, while the negative control DOES report "unique case, but multiple matches found". CORRECTION LANDED: the pre-.1 CasezMux-can-overlap claim is withdrawn on (a)+(b). Icarus emits "vvp.tgt sorry: Case unique/unique0 qualities are ignored." for unique/unique0 (exit 0; priority silent) ⇒ the ADR pins per-qualifier tool plans rather than letting the gate pass on first_tool_warning's `warning:` lexical accident. INDEX + this tree + docs/TASK_TREE.md + ROADMAP.md updated.`
  Commit: `CAPABILITY-BREADTH-EXPANSION.3`

- ID: `CAPABILITY-BREADTH-EXPANSION.4`
  Status: `active` (container — `.4a` design-detail **done**; `.4b` impl pending)
  Goal: `Case-qualifier impl — the default-off unique_case_prob / priority_case_prob knobs (config + CLI + dump-config + KnobId rows in their OWN `qualifiers` steering category, not `emission` — corrected at .4b.1) + a src/ir/case_qualifier.rs gen-time annotation pass writing Module.case_qualifiers: BTreeMap<NodeId, CaseQualifier>, excluding gates already claimed by case_mux_if_gates / casez_mux_if_gates and constant-selector gates + the emitter keyword prefix on the case/casez line + num_emitted_unique_cases / num_emitted_priority_cases at introspection schema 1.27 → 1.28 + the repo-owned tool_matrix --case-qualifier-gate (unique: Verilator + both Yosys, Icarus a recorded accepting no-op; priority: the full three-tool plan) + book/USER_GUIDE/KM. Default-off / DUT byte-identical, snapshots untouched.`
  Acceptance: `cargo check/test/clippy/fmt green; snapshots 6/6 byte-identical with both knobs at 0.0; each knob at 1.0 emits the qualifier on every eligible gate and is downstream-clean on its pinned tool plan; the gate lights saw_unique_case_qualifier / saw_priority_case_qualifier with coverage_gaps = []. PLUS an in-crate #[test] asserting the FULL + PARALLEL property over emitted CaseMux/CasezMux blocks — this is the DURABLE home for the .3 corpus measurement, which was taken with a scratch checker under .cache/ (untracked by design). Rust-side invariants belong in a #[test] already gated by cargo test + CI, not in a tracked shell script (the decision-0033 reasoning ENUMERATION-PARITY applies to its own Rust-side pairs). The test must fail on a hand-built overlapping-arm or default-less fixture, per DOCTRINE_ENFORCEMENT.md section 9.`
  Children: `CAPABILITY-BREADTH-EXPANSION.4a`, `CAPABILITY-BREADTH-EXPANSION.4b`

- ID: `CAPABILITY-BREADTH-EXPANSION.4a`
  Status: `done`
  Goal: `Case-qualifier impl design-detail (docs-only, no code): ground decision 0044 in a fresh read of the real code and resolve its five Open Questions — the Module carrier + enum and where the enum lives, the two knobs' KnobId category, the metric names + schema bump, the per-gate roll precedence, and whether decision 0032's emit-surface interaction gate gains the two knobs as a tenth/eleventh axis. Pin every .4b impl point (predicate, carrier, emitter integration, pass ordering, knobs, metrics, gate scenarios + per-qualifier tool plans, byte-identical argument) and the shape of the FULL + PARALLEL property test that is .4's acceptance requirement.`
  Acceptance: `A DEVELOPMENT_NOTES.md design-detail entry resolving all five Open Questions with grounding line references to current HEAD; the tree + docs/TASK_TREE.md updated; docs-only ⇒ DUT byte-identical.`
  Verification: `done — DEVELOPMENT_NOTES.md "2026-08-01 — unique / priority case qualifiers — impl design-detail — CAPABILITY-BREADTH-EXPANSION.4a". All five Open Questions resolved: (1) carrier = Module.case_qualifiers: BTreeMap<NodeId, CaseQualifier> (a MAP, so "at most one qualifier per gate" is unrepresentable-otherwise rather than asserted), enum in the new src/ir/case_qualifier.rs beside its pass (the direction IR-TYPES-DECOMPOSITION is moving types.rs, and Module derives only Debug/Clone/Default so no serde attribute is needed); (2) KnobId category = emission, on both of decision 0035's axes (post-construction pipeline position + per-candidate-gate resolution); (3) metrics num_emitted_unique_cases / num_emitted_priority_cases at schema 1.27 → 1.28; (4) precedence = roll unique first and SKIP the priority roll on a hit — the skip is load-bearing, because rolling both and letting unique win would record a priority fire that emits nothing, the exact telemetry-vs-reality divergence decision 0034 removed; with it, fires(knob) == the knob's metric exactly and the two metrics sum to case_qualifiers.len(); (5) decision 0032's interaction gate does NOT gain the knobs — a qualifier replaces no rendering, so it is not a distinct_emit_surfaces member and applies no exclusion pressure, and folding unique in would cost the universal scenarios their Icarus column (the trade 0032 §(c) already rejected for soft_union) — the co-occurrence instead lands INSIDE --case-qualifier-gate as a dedicated scenario, so the pass's one non-vacuous exclusion is not left unexercised end-to-end. TWO non-obvious findings recorded: (a) unique_case_prob / priority_case_prob do not end in _emit_prob, so config.rs knob_group falls through to "other" which knob_catalog_classifies_every_field forbids ⇒ .4b.1 adds an explicit "case_qualifier" arm — and the alternative (naming them *_emit_prob to ride the existing arm) would have CONSCRIPTED them into --profile structured-emission-max via the decision-0032 preset drift test, which derives its expected set from that catalog group, silently costing the preset its Icarus column; (b) the unique tool plan needs a SECOND pure-function-of-config predicate beside scenario_emits_soft_union_overlay, not a widening of verilator_only — that flag drops Yosys AND Icarus, this reduction drops Icarus only. FULL + PARALLEL test shape pinned as two separate #[cfg(test)] predicates (PARALLEL from the IR by pairwise ((p_i^p_j) & c_i & c_j) == 0; FULL from the emitted text by a nesting-aware default:-per-block scan) each with its own firing negative control, per DOCTRINE_ENFORCEMENT.md §9. Docs-only; no src/ change.`
  Commit: `CAPABILITY-BREADTH-EXPANSION.4a`

- ID: `CAPABILITY-BREADTH-EXPANSION.4b`
  Status: `active` (container — `.4b.1` **done**; `.4b.2` metric+gate, itself split into `.4b.2a`/`.4b.2b`, and `.4b.3` docs pending)
  Goal: `Case-qualifier impl — default-off / DUT byte-identical, snapshots untouched. Pre-split into .4b.1 (live: the pass + carrier + enum + two knobs + knob_group arm + two gen/mod.rs rolls + the emitter prefix on the two unprojected branches + the lib proofs incl. the FULL/PARALLEL property tests and their negative controls), .4b.2 (metrics num_emitted_unique_cases / num_emitted_priority_cases @ introspection schema 1.27 → 1.28 + the repo-owned tool_matrix --case-qualifier-gate: 13 scenarios, metric-keyed detection, three coverage facts, per-qualifier tool plans), and .4b.3 (book/src/structured-emission.md + knobs.md + USER_GUIDE + a KM card).`
  Acceptance: `As .4's acceptance, per leaf.`
  Children: `CAPABILITY-BREADTH-EXPANSION.4b.1`, `CAPABILITY-BREADTH-EXPANSION.4b.2`, `CAPABILITY-BREADTH-EXPANSION.4b.3`
  Notes: `Every impl point is pinned in the .4a DEVELOPMENT_NOTES entry; .4b implements it rather than re-deriving it.`

- ID: `CAPABILITY-BREADTH-EXPANSION.4b.1`
  Status: `done`
  Goal: `Case-qualifier LIVE SOURCE — the new src/ir/case_qualifier.rs (CaseQualifier enum + gate_qualifies + annotate_case_qualifiers) + Module.case_qualifiers: BTreeMap<NodeId, CaseQualifier> + ir/mod.rs registration + the two default-off unique_case_prob / priority_case_prob knobs (config field + default fn + Default + validate-probs list + apply + Overrides + the explicit knob_group "case_qualifier" arm) + the --unique-case-prob / --priority-case-prob CLI flags + two KnobId rows in a NEW `qualifiers` steering category (NOT `emission` — corrected mid-leaf, see Verification) + the tenth guarded roll in BOTH gen/mod.rs paths (after casez_mux_if, so the non-vacuous exclusion sees both chain sets) + the emitter keyword prefix on the two UNPROJECTED case/casez branches. Plus the lib proofs, including the FULL + PARALLEL property test over real generated output with its negative controls. No metric/schema bump (that is .4b.2).`
  Acceptance: `cargo check/test/clippy/fmt green; snapshots 6/6 byte-identical with both knobs at 0.0; each knob at 1.0 qualifies every eligible gate; downstream-clean on the pinned per-qualifier tool plans; the FULL/PARALLEL property test must FAIL when its subject is deleted from the real source (DOCTRINE_ENFORCEMENT.md §9).`
  Verification: `done — ONE CORRECTION TO .4a LANDED MID-LEAF: the KnobId category is `qualifiers`, NOT `emission`. The first full-suite run failed every_emission_surface_is_individually_measurable_and_steerable (tests/pipeline.rs), which pins emission.len() == 9 — "the 9 emit-projection surfaces decision 0035 names". The guard is right: a qualifier is not a projection (.4a's own load-bearing argument), and it claims only the gates the projections DECLINED, so the two are anti-correlated and one --steer emission=2.0 would average a self-cancelling mixture — the same reason 0035 split motifs off. .4a argued the category from where/how often the roll happens (mechanism); a steering category is a property of the USER'S INTENT. The guard was heeded, not edited. Own `qualifiers` category ⇒ --steer emission keeps its pre-existing meaning (0035's promise) and the qualifiers are independently steerable: measured at unique_case_prob=0.4, --steer qualifiers=2.0 moves the achieved rate 0.385 → 0.769 and 0.25 moves it to 0.154, ATTEMPTS FIXED at 26 throughout (rules-first: the prior scales the probability, it adds no rejection path); pinned by a new case_qualifiers_are_their_own_steerable_category guard. ENUMERATION-PARITY then priced the taxonomy change immediately — 7 fenced doc sites had to name `qualifiers` (book/src/{algorithm,knobs,agent-mcp,api-introspection}.md + USER_GUIDE.md + docs/AGENT_INTROSPECTION_SCHEMA.md + CODEBASE_ANALYSIS.md), and book/src/algorithm.md's ungated "eight coarse categories" count beside the fence was repaired by DELETION per decision 0033 R1. Then: cargo check --all-targets, clippy --all-targets -D warnings, fmt --all --check all clean; full cargo test green; lib 749 → 764 (+15 new proofs); tests/snapshots.rs 6/6 BYTE-IDENTICAL (both knobs default 0.0 ⇒ the generator guard skips the pass ⇒ zero draws). FIRING PROOF (seed 7, 8 comb-only case/casez-biased modules, 30 case/casez statements): knobs off ⇒ 30 bare / 0 qualified; --unique-case-prob 1.0 ⇒ 30 unique / 0 bare; --priority-case-prob 1.0 ⇒ 30 priority / 0 bare — every eligible gate, exactly. BEHAVIOUR PRESERVATION proven STRUCTURALLY, stronger than the sibling surfaces' -Wall delta-of-0: stripping the qualifier token from each ON corpus yields output BYTE-IDENTICAL to the OFF corpus, so nothing but the token moved. DOWNSTREAM (Verilator 5.046 / Yosys 0.64 / Icarus 13.0) — verilator --lint-only (the repo bar): 0 nonzero exits, 0 %Warning, 0 %Error across all three lanes; yosys BOTH repo modes (synth -noabc / +abc -fast; opt -fast; check): 0 failures, 0 warnings, all three lanes, and synthesized cell counts IDENTICAL 8/8 (off = uniq = prio = 0,222,26,44,180,26,230,123) — the synthesis-side confirmation that an assertion which holds gives the synthesizer no new freedom; iverilog -g2012: exit 0 all three lanes, priority SILENT, unique printing exactly 30 "vvp.tgt sorry: Case unique/unique0 qualities are ignored." lines and 0 lines matching "warning:" — reproducing decision 0044's measurement and confirming first_tool_warning's iverilog arm would have passed it on a lexical accident, which is why .4b.2 splits the tool plan rather than widening that shared classifier. §9 VACUITY PROBE run against the property test on the REAL SOURCE, not fixtures: suppressing the emitter's unconditional default: arm makes full_violations report every generated block (8+ on seed 1); widening build_casez_patterns' wildcard mask by one bit makes parallel_violations report real overlaps (e.g. "node 8: casez arms 0 and 1 overlap"); both files restored to a zero diff. This is the check .3's scratch checker failed.`
  Commit: `CAPABILITY-BREADTH-EXPANSION.4b.1`

- ID: `CAPABILITY-BREADTH-EXPANSION.4b.2`
  Status: `active` (container — split into `.4b.2a` metric+schema and `.4b.2b` gate)
  Goal: `Case-qualifier metrics + gate — Metrics::num_emitted_unique_cases / num_emitted_priority_cases (a filter over Module.case_qualifiers; exact because the pass excludes statically-collapsed and chain-projected gates) surfaced in --introspect via the serde projection, at the additive introspection schema MINOR bump 1.27 → 1.28 (const + doc comment + every schema_version assertion in introspect/mod.rs + mcp/mod.rs + the docs/AGENT_INTROSPECTION_SCHEMA.md contract). Plus the repo-owned tool_matrix --case-qualifier-gate + ScenarioSet::CaseQualifierSweep: 13 scenarios = {unique, priority} x {case_mux-biased, casez_mux-biased} x 3 strategies, plus ONE co-occurrence scenario (case_mux_if_emit_prob = casez_mux_if_emit_prob = 0.25 with priority_case_prob = 1.0) so the pass's non-vacuous exclusion fires end-to-end. Metric-keyed detection (decision 0028), three coverage facts (saw_unique_case_qualifier / saw_priority_case_qualifier / saw_case_qualifier_beside_if_chain), and the per-qualifier tool plans via a NEW scenario_emits_unique_case_qualifier predicate beside scenario_emits_soft_union_overlay — a separate boolean, not a widening of verilator_only, which drops a different pair of columns.`
  Acceptance: `--introspect reports both metrics at schema 1.28; the gate lights all three coverage facts with coverage_gaps = []; unique scenarios run Verilator + both Yosys with Icarus a recorded accepting no-op; priority + co-occurrence run the full three-tool plan; default-off DUT byte-identical.`
  Children: `CAPABILITY-BREADTH-EXPANSION.4b.2a`, `CAPABILITY-BREADTH-EXPANSION.4b.2b`
  Notes: `Split for the same reason both sibling surfaces were (STRUCTURED-EMISSION-EXPANSION.17b.2a/.17b.2b, .19b.2a/.19b.2b) and this tree's own Mealy strand (.2b.2a/.2b.2b): the metric is a schema bump that touches ~40 version assertions across four crates' worth of surface and must land with nothing else moving, while the gate is a 13-scenario addition to a 14.6k-line binary with its own non-vacuity proof. One commit each keeps both reviewable and either revertible alone.`

- ID: `CAPABILITY-BREADTH-EXPANSION.4b.2a`
  Status: `pending`
  Goal: `Case-qualifier METRICS + the additive schema bump. Metrics::num_emitted_unique_cases / num_emitted_priority_cases — each a filter over Module.case_qualifiers.values(), #[serde(default)] so the introspection projection stays additive — set in metrics::compute beside num_emitted_casez_mux_if_chains, with lib proofs mirroring metrics_count_emitted_casez_mux_if_chains and one proof of the .4a telemetry identity (the two metrics sum to case_qualifiers.len()). Plus introspection SCHEMA_VERSION 1.27 → 1.28 across every CURRENT-value reference: the const + its doc-comment history in introspect/mod.rs, the 3 introspect assertions, the mcp/mod.rs assertions, docs/AGENT_INTROSPECTION_SCHEMA.md (current-version statements + a 1.27 → 1.28 changelog entry + the two new metric field entries), and the TOOLBOX/USER_GUIDE/CODEBASE_ANALYSIS envelope references. Per-metric "landed at schema X" attributions are HISTORY and stay untouched.`
  Acceptance: `cargo check/test/clippy/fmt green; snapshots 6/6 byte-identical (a post-hoc Metrics field changes no RTL); no stray CURRENT-value "1.27" left in src/ or the live docs; live --introspect reports schema 1.28 with both metrics 0 by default and non-zero under --unique-case-prob / --priority-case-prob.`
  Verification: `done — cargo check --all-targets + clippy --all-targets -D warnings + fmt --all --check all clean; full cargo test green (0 failures); lib 764 → 765 (+1 proof); tests/snapshots.rs 6/6 BYTE-IDENTICAL. LIVE PROOF at schema 1.28 (seed 7, comb-only case/casez-biased): default 0 unique / 0 priority; --unique-case-prob 1.0 → 9 / 0; --priority-case-prob 1.0 → 0 / 9. EXACTNESS CROSS-CHECKED AGAINST THE EMITTED TEXT rather than asserted: grep -cE "^\s+(unique|priority) case[z]? \(" over the same corpora reports 9 and 9 — metric == emitted statements, both qualifiers. TELEMETRY IDENTITY CONFIRMED END-TO-END: fires(unique_case_prob) == 9 == num_emitted_unique_cases with attempts 9; and with BOTH knobs at 1.0 the run reports 9 unique / 0 priority and priority_case_prob records NO attempts and NO fires at all — the .4a hit-skip working on real output, so there is no phantom fire to interpret. THE BUMP WAS A CLASSIFICATION PASS, not a sed: current-value refs moved (the const + its doc-comment history, 3 introspect + 18 mcp assertions, the schema doc's three current-version statements + a new 1.27 → 1.28 changelog entry, TOOLBOX + USER_GUIDE envelopes, 19 book example-JSON strings + 3 "currently 1.27" book statements), while HISTORY did not (per-feature attributions "reach_path landed at 1.27", prior changelog entries, the CODEBASE_ANALYSIS timeline rows, and the docs/knowledge/api-reference.md anecdote whose whole point is a card that rotted to 1.11 against a live 1.27). One knowledge-card EVIDENCE POINTER was reworded rather than bumped — docs/knowledge/semantic-introspection-reach-path.md cited "src/introspect/mod.rs (SCHEMA_VERSION = 1.27)" as if the number were the fact; it now names the const, which is the lesson api-reference.md had already recorded three files away and four leaves earlier. No current-value 1.27 remains in src/ (the five survivors are all prose about 1.27-the-predecessor inside the new doc comment). Two metrics rather than one, and the reason is downstream not aesthetic: .4b.1 measured Icarus printing "sorry:" per unique block and staying silent for priority, so .4b.2b picks a per-qualifier tool plan — a combined count would force the gate to re-derive the split from the config, the exact decision-0033 shadow the metric removes.`
  Commit: `CAPABILITY-BREADTH-EXPANSION.4b.2a`

- ID: `CAPABILITY-BREADTH-EXPANSION.4b.2b`
  Status: `pending`
  Goal: `The repo-owned tool_matrix --case-qualifier-gate + ScenarioSet::CaseQualifierSweep: 13 scenarios = {unique, priority} x {case_mux-biased, casez_mux-biased} x 3 strategies, plus ONE co-occurrence scenario (Interleaved, both selector kinds live, case_mux_if_emit_prob = casez_mux_if_emit_prob = 0.25, priority_case_prob = 1.0) so the pass's non-vacuous exclusion fires end-to-end. Detection is METRIC-KEYED (decision 0028), never a text scan. Three coverage facts: saw_unique_case_qualifier / saw_priority_case_qualifier / saw_case_qualifier_beside_if_chain. Per-qualifier tool plans via a NEW scenario_emits_unique_case_qualifier predicate threaded as a SEPARATE boolean beside scenario_emits_soft_union_overlay's verilator_only — gating ONLY the Icarus column, because verilator_only drops Yosys and Icarus where this drops Icarus alone. first_tool_warning is NOT widened to catch "sorry:" (a shared classifier serving one scenario is the wrong mechanism).`
  Acceptance: `The gate lights all three coverage facts with coverage_gaps = []; a non-vacuity test pins the scenario-count and gate-design deltas; unique scenarios run Verilator + both Yosys with Icarus a recorded accepting no-op; priority + co-occurrence run the full three-tool plan; banked downstream-clean; default-off DUT byte-identical.`
  Verification: `done — cargo check --all-targets + clippy --all-targets -D warnings + fmt --all --check clean; full cargo test green; tool_matrix bin 115 → 125 proofs; snapshots 6/6 byte-identical. BANKED CLEAN: docs/evidence/anvil-case-qualifier-gate-r1.md (Verilator 5.046 / Yosys 0.64 / Icarus 13.0) — 13 scenarios / 52 modules / 359 qualified statements (173 unique, 186 priority) / coverage_gaps = [] with all three facts lit / Verilator 52/0 / Yosys 52/0 in BOTH modes / Icarus 28/0, where 28 is exactly the seven non-unique scenarios x 4, so the per-qualifier tool plan is VISIBLE IN THE BANK rather than merely asserted. The co-occurrence scenario emitted 3 chains beside 23 qualifiers. THE FINDING: the non-vacuity test first generated ONE module per scenario while the gate generates FOUR, and it FAILED on the co-occurrence scenario — the test's fault, not the config's. Measured over the real four: the 12 single-qualifier scenarios fire in 4/4 modules, the co-occurrence one in 2/4. So a one-module probe reports a false failure here — and had module 0 happened to carry both, it would have passed while hiding a real dependency, since the co-occurrence fact is PER MODULE and is satisfied by half of them, making the unit floor load-bearing for this gate rather than a generic minimum. Pinned by a new case_qualifier_gate_unit_floor_cannot_be_lowered_from_the_cli proof (derive_run_plan takes the max of the CLI value and the floor, so --modules-per-scenario 1 cannot silently drop below it). A CLAIM RETRACTED BEFORE IT SHIPPED: the detection comment first justified metric-keying by asserting unique/priority would collide with ordinary emitted vocabulary; probed and FALSE (neither word appears anywhere else in generated SV), so it was replaced with the reason that does not depend on that measurement — the metric already carries the fact exactly, so a text scan would be a second source of truth (0033) and would couple the gate to the emitter's spelling, turning it green-and-blind rather than red. TWO DELIBERATE DEFERRALS: the book is untouched because .4b.3 owns surface + knobs + gate together and a gate-only section would be immediately restructured; and the digest's commit field records c1cc6a1 (the commit the run executed AGAINST) — evidence_digest.sh warns that a gate banked by its own introducing commit needs it backfilled, which .4b.3 will do.`
  Commit: `CAPABILITY-BREADTH-EXPANSION.4b.2b`

- ID: `CAPABILITY-BREADTH-EXPANSION.4b.3`
  Status: `pending`
  Goal: `Case-qualifier user-facing docs — a book/src/structured-emission.md section that is deliberately NOT titled "the tenth surface" (a qualifier decorates a rendering rather than replacing one, and the chapter's mutual-exclusion section would otherwise mis-describe it), with a byte-verified runnable example; book/src/knobs.md entries + the knob->metric table rows; USER_GUIDE.md flag rows; and a KM how-to card with a working reverify. mdbook build clean.`
  Acceptance: `mdbook build + tests/book_examples clean; both knobs documented with an example; KM regenerated and in sync.`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `CAPABILITY-BREADTH-EXPANSION.4b.3` | `pending` | **Next.** The user docs, and it now carries three things beyond its original scope. (a) The book section must **not** be called "the tenth surface" — the chapter's "Combining the surfaces" section is about the mutual-exclusion chain over *projections*, and a qualifier is not one of them. (b) It owns the **gate** documentation too: `.4b.2b` deliberately left the book untouched, because a gate-only section in a chapter with no surface section would have to be immediately restructured — so surface + knobs + gate land together. (c) It must **backfill the evidence digest's `commit` field**: `docs/evidence/anvil-case-qualifier-gate-r1.md` records `c1cc6a1`, the commit the run executed *against*, and `scripts/evidence_digest.sh` warns that a gate banked by its own introducing commit needs the real hash written in afterwards. |
| 2 | `CAPABILITY-BREADTH-EXPANSION.4b.2a` | `done` | The metric + the additive schema bump. |
| 3 | `CAPABILITY-BREADTH-EXPANSION.4b.2b` | `done` | The gate — banked clean at `anvil-case-qualifier-gate-r1`. |
| 4 | `CAPABILITY-BREADTH-EXPANSION.1` | `pending` | SV up-opt breadth — **deferred-not-retired**, and deferred on evidence twice (the `.2a` and pre-`.1` probes): the named candidates (enum/typedef, packed multidim arrays) are accepted at every Verilator `--language` mode + Yosys + Icarus ⇒ not version-distinctive, no down-gating teeth, and post-2012 SV additions are overwhelmingly verification features a synthesizable generator cannot emit. The genuinely-2023 clean space with the installed tools is essentially `union soft`, shipped. Unblocks when a tool gains real 2023 synthesizable coverage. Its question was **not** folded into `.3` — see decision `0044` Context. |

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

## Pre-`.1` findings (`2026-08-01`, measured — SUPERSEDED at `.3`, kept verbatim)

> **Superseded `2026-08-01` by `.3` / decision [`0044`](../decisions/0044-capability-breadth-unique-priority-case-qualifiers.md).**
> Kept verbatim rather than rewritten (`MEMORY_ARCHITECTURE.md` §10: *supersede, don't mutate*).
> Two of its four points were resolved and **one was measured wrong** — see the inline markers.
> Read `0044` for the outcome; this block is the record of what was believed at the time.

**1. `.1`'s premise is contested by measurement, and the owner has not ruled.** The leaf asks for
the *"next version-distinctive up-opt"*, but its own candidates (`enum`/`typedef`, packed
multi-dimensional arrays) are **SV-2012-legal** — breadth, not up-opts. Post-2012 SystemVerilog
additions are overwhelmingly **verification** features (assertions, classes, coverage,
randomization) that a *synthesizable* generator cannot emit, so the version-distinctive seam is
close to exhausted after `union soft`. Measured `2026-08-01`: **zero** emitter string literals for
`enum`, `unique`, `priority`, `always_latch`, `casex`, `interface`, `modport`, `inside` — a large
**2012-legal** breadth gap, all eight synthesizable and well-supported by Verilator/Yosys/Icarus.
Recommended reframing: *next version-distinctive up-opt* → **next synthesizable breadth construct**.

**2. Recommended first pick: `unique` / `priority` case qualifiers — and it is provably
valid-by-construction, which is the part that had to be checked rather than assumed.** Generated a
real module (`--seed 3`, `case_mux_prob = 1.0`, 3 arms, 2-bit selector) and read the RTL:

```systemverilog
always_comb begin
    case (slice_0)
        2'd0: case_mux_0 = i_2;
        2'd1: case_mux_0 = i_2;
        2'd2: case_mux_0 = 4'hc;
        default: case_mux_0 = 4'h0;
    endcase
end
```

Two properties fall out **by construction**, not by analysis:

- **FULL** — the emitter always writes a `default:` arm (`src/emit/sv.rs:805`), so every selector
  value is covered even when `arms < 2**sel_width` (here 3 arms over a 2-bit selector).
- **PARALLEL** — arm labels are the **sequential integers** `0..N-1` (`sel_width'd{arm_idx}`), so
  they are distinct by construction and no two arms can match the same selector value.

`priority` asserts full; `unique` asserts full **and** parallel. `CaseMux` satisfies **both**, so
either qualifier is legal on every emitted `CaseMux` with **no** analysis pass and no
generate-then-filter — the generator already knows the answer.

**3. `CasezMux` is NOT in scope for `unique`, and this is the trap to avoid.**
**⛔ WITHDRAWN `2026-08-01` at `.3` — this point is FALSE.** `build_casez_patterns`
(`src/gen/cone/motifs.rs:832-841`) is the **sole** casez pattern source in the generator and gives
arm `i` the care-value `i` with exactly one don't-care LSB, so `CasezMux` arms are **disjoint by
construction** — confirmed over 14,415 emitted `casez` blocks and by a 20,000-vector
`verilator --assert` run with `unique casez` reporting zero violations. The error was reasoning
from the *IR shape* (the triples **can** express overlap) instead of from the **constructor**
(which never builds one). *What the IR can represent is not what the generator constructs.* The
original text follows unchanged:
Its arms carry
`(pattern, wildcard_mask, data)` triples, so two arms **can** overlap for a given selector value.
`unique` on an overlapping `casez` is a false assertion — the exact class of bug that makes tools
disagree, which is worth *generating* deliberately but never worth emitting *accidentally*.
`priority` (full only, first-match wins — `casez` semantics already) is the safe qualifier there.
So the design must gate the two qualifiers **per gate kind**, not with one shared knob.

**4. Why this is high value-per-effort.** `CaseMux`/`CasezMux` already exist in `GateOp`, so the
change is **emission-level only** — no IR change, no new node kind. And the qualifiers are precisely
the construct that makes lint/synthesis/simulation disagree on full/parallel-case inference, which
is the project's stated purpose (*"stress such tools and expose real bugs"*).

**Not yet done:** the downstream probe (Verilator `--lint-only`, Yosys both modes, Icarus) on a
qualifier-bearing module, and the ADR itself. That is `.1`'s remaining work.
**✅ BOTH DONE `2026-08-01` at `.3`** — under a new third strand, not under `.1`. Point 1's
recommended reframing was **declined**: `.1`'s question was kept and `.3` opened beside it
(decision `0044` Context). Point 2 was confirmed and strengthened from one read module to a
50,761-block corpus measurement plus a runtime violation-checker proof with a firing negative
control. Point 3 was withdrawn, above.

- `2026-08-01` (`.3`): **A third strand — `unique` / `priority` case qualifiers** — decision
  [`0044`](../decisions/0044-capability-breadth-unique-priority-case-qualifiers.md). A
  default-off, valid-by-construction qualifier prefix on the `case`/`casez` statement a
  dynamic-selector `CaseMux`/`CasezMux` already emits. Both asserted properties come free from
  the emitter: the always-present `default:` arm gives **FULL**, and the sequential arm indices
  (plain labels for `CaseMux`; the single-don't-care-LSB care-values of `build_casez_patterns`
  for `CasezMux`) give **PARALLEL** — no analysis pass, no generate-then-filter. New default-off
  `unique_case_prob` / `priority_case_prob` knobs, `num_emitted_unique_cases` /
  `num_emitted_priority_cases` metrics (introspection schema `1.27 → 1.28`), a
  `tool_matrix --case-qualifier-gate` with **per-qualifier tool plans**, CLI + MCP/config
  settability. **The scope call is part of the decision:** `.1` was **not** reframed — its
  question (*which post-2012 construct is distinctive?*) is distinct from `.3`'s (*which
  2012-legal construct is missing?*), and it is the only place the repo asks it. Two prior
  beliefs were corrected on measurement: `CasezMux` **is** parallel by construction (the
  pre-`.1` overlap claim is withdrawn), and `priority` — not `unique` — is the semantically
  inert one here, so picking it first to dodge Icarus's `sorry:` note would have shipped the
  qualifier with no new tool code path exercised. `.3` split the strand into `.3` (design, done)
  + `.4` (impl, pending).

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
- ~~The `Module` carrier + enum for the qualifier marks, the two knobs' `KnobId`
  **category** (`emission` vs `selectors`), the per-gate roll precedence, and
  whether decision `0032`'s emit-surface **interaction** gate gains the two knobs
  as a tenth/eleventh axis~~ — **resolved at `.4a`** (`DEVELOPMENT_NOTES.md`,
  `2026-08-01`): carrier `Module.case_qualifiers: BTreeMap<NodeId, CaseQualifier>`
  with the enum in the new `src/ir/case_qualifier.rs`; category ~~`emission`~~ → **`qualifiers`**
  (⛔ `.4a`'s `emission` answer was **corrected at `.4b.1`**: a `tests/pipeline.rs` guard pins
  `emission` to the nine *projections*, and a qualifier claims only the gates those projections
  declined, so pooling them would average a self-cancelling mixture — `.4a` argued from where the
  roll happens, but a steering category is a property of the **user's intent**);
  precedence *roll `unique` first and **skip** the `priority` roll on a hit*, so
  each knob's recorded fires equal its metric exactly; and **no** — a qualifier
  replaces no rendering, so it is not a `distinct_emit_surfaces` member, and
  folding `unique` into `0032`'s universal scenarios would cost them their Icarus
  column. The lane's first **non-vacuous** exclusion (against `case_mux_if_gates`
  / `casez_mux_if_gates`, which leave no `case` keyword to prefix) is therefore
  exercised end-to-end by a dedicated co-occurrence scenario **inside**
  `--case-qualifier-gate`, rather than being left written-but-never-fired.
- Whether the two qualifier knobs should ever join a `--profile`. **Not at `.4`**:
  `.4a` found that naming them `*_emit_prob` would have conscripted them into
  `structured-emission-max` through decision `0032`'s preset drift test (which
  derives its expected set from the `structured_emission` *catalog group*),
  costing that preset its Icarus column for a construct that adds no surface
  diversity. They take their own `case_qualifier` catalog group and no preset. A
  dedicated qualifier preset is a separate decision, recorded not retired.
  *(Resolves at `.5`+, if ever.)*
- Whether `unique0`, the `unique if` / `priority if` variants on the eighth/ninth-surface
  chains, and the FSM `case (state_q)` / `case (sel)` blocks each become their own
  `.5`+ leaf. All three are recorded, none retired; the FSM blocks are already
  **measured** full+parallel (130/130) so their safety question is closed in advance.
  *(Resolves at `.5`+.)*

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
| `2026-08-01` | `CAPABILITY-BREADTH-EXPANSION.3` | `decision 0044 written (KM answers: front-matter); by-construction argument read off src/emit/sv.rs:800-806 + :712-721 + src/gen/cone/motifs.rs:832-841; corpus measurement 50,761 case/casez blocks (120 modules × 3 strategies) 100 % FULL + PARALLEL with 0 nested / 0 unparsed / 0 violations, plus 130 FSM blocks (106 nested) 100 % FULL + PARALLEL; checker proven non-vacuous on a hand-written overlapping casez + default-less case (exit 1) AFTER v1 of the checker failed that same test and was rewritten; downstream ON-vs-OFF over 24 modules / 169 qualified blocks — verilator --lint-only 0/0, verilator -Wall × 2012/2017/2023 Δ=0 (68 vs 68), Yosys both modes 0 fail / 0 msg, cell counts identical 24/24, iverilog -g2012 exit 0; runtime verilator --binary --assert ZERO violations on a real CaseMux (exhaustive) and a real CasezMux with unique casez (20,000 vectors), negative control DOES fire; LRM §12.5.3 grounded against the local 2017 cache; INDEX + tree + docs/TASK_TREE.md + ROADMAP.md updated; mem-arch + KM + doctrine self-checks` | `done` (docs-only; no code; DUT byte-identical) — **one prior finding corrected**: `CasezMux` is parallel by construction; the pre-`.1` overlap claim is withdrawn |
| `2026-08-01` | `CAPABILITY-BREADTH-EXPANSION.4b.1` | `cargo check --all-targets + clippy --all-targets -D warnings + fmt --all --check clean; full cargo test green; lib 749 → 764 (+15 proofs); snapshots 6/6 BYTE-IDENTICAL. Firing proof (seed 7, 8 comb-only modules, 30 case/casez statements): off 30 bare/0 qualified, --unique-case-prob 1.0 30 unique/0 bare, --priority-case-prob 1.0 30 priority/0 bare. Behaviour preservation proven STRUCTURALLY: stripping the qualifier token from each ON corpus gives output byte-identical to OFF. Downstream — verilator --lint-only 0 exits/0 %Warning/0 %Error all three lanes; yosys both repo modes 0 fail/0 warn all three lanes with cell counts IDENTICAL 8/8; iverilog -g2012 exit 0 all three, priority silent, unique exactly 30 "vvp.tgt sorry:" and 0 "warning:". §9 vacuity probe on the REAL SOURCE: suppressing the emitter default: arm reddens full_violations; widening build_casez_patterns' mask reddens parallel_violations; both restored to zero diff` | `done` (live surface; default-off DUT byte-identical) — the `.3` corpus measurement now has a durable in-crate home |
| `2026-08-01` | `CAPABILITY-BREADTH-EXPANSION.4b.2a` | `cargo check --all-targets + clippy --all-targets -D warnings + fmt --all --check clean; full cargo test green; lib 764 → 765 (+1 proof, metrics_count_emitted_case_qualifiers_per_qualifier); snapshots 6/6 BYTE-IDENTICAL. Live --introspect at schema 1.28 (seed 7, comb-only case/casez-biased): default 0/0, --unique-case-prob 1.0 → 9 unique, --priority-case-prob 1.0 → 9 priority; each cross-checked against grep -cE over the emitted SV (9 and 9 — metric == emitted statements). Telemetry identity live: fires == metric == 9 with attempts 9, and with BOTH knobs at 1.0 priority_case_prob records no attempts and no fires (the .4a hit-skip, end-to-end). Schema propagated across the const + doc-comment history, 3 introspect + 18 mcp assertions, the schema-doc contract (3 current-version statements + a 1.27 → 1.28 changelog entry), TOOLBOX + USER_GUIDE envelopes, and 19 book example-JSON strings + 3 book "currently" statements; history refs (attributions, prior changelog entries, timeline rows, the api-reference anecdote) deliberately untouched; one KM evidence pointer reworded to name the const instead of its value` | `done` (metric queryable per decision `0017`; default-off DUT byte-identical) |
| `2026-08-01` | `CAPABILITY-BREADTH-EXPANSION.4b.2b` | `cargo check --all-targets + clippy --all-targets -D warnings + fmt --all --check clean; full cargo test green; tool_matrix bin 115 → 125 proofs; snapshots 6/6 byte-identical. BANKED: docs/evidence/anvil-case-qualifier-gate-r1.md — 13 scenarios / 52 modules / 359 qualified statements (173 unique, 186 priority) / coverage_gaps = [] with saw_unique_case_qualifier + saw_priority_case_qualifier + saw_case_qualifier_beside_if_chain all lit / Verilator 52/0 / Yosys 52/0 BOTH modes / Icarus 28/0 = exactly the seven non-unique scenarios x 4. Co-occurrence scenario: 3 chains beside 23 qualifiers. Non-vacuity proven over the REAL run plan (4 modules/scenario from one Generator), which is how the 2-of-4 co-occurrence rate — and hence the load-bearing unit floor — was found; a claim about vocabulary collision was probed FALSE and retracted before the commit` | `done` (gate; default-off DUT byte-identical) — the lane's only **non-vacuous** exclusion is now exercised end-to-end |
| `2026-08-01` | `CAPABILITY-BREADTH-EXPANSION.4a` | `impl design-detail written to DEVELOPMENT_NOTES.md, grounded in a fresh read of current HEAD (src/ir/case_mux_if_emit.rs, src/emit/sv.rs:667-809, src/gen/cone/motifs.rs:832-841, src/gen/mod.rs:257/274/564/580, src/ir/types.rs:166/576, src/ir/knob_id.rs:220-253, src/config.rs:126/1084/1335/1720/1955/2082/2197-2204/2303-2364/2401/2806/2901, src/main.rs:555-557/1157, src/metrics.rs:398-411/1031, src/introspect/mod.rs:70, src/bin/tool_matrix.rs:1083/1677/4288/4326/5498/5550/6141/6432/6469/6475/7235-7244/8794); all five decision-0044 Open Questions resolved; the .4b impl points and the FULL/PARALLEL property-test shape pinned; tree + docs/TASK_TREE.md updated; mem-arch + KM + doctrine self-checks` | `done` (docs-only; no code; DUT byte-identical) — **two non-obvious findings**: the knob names force an explicit `knob_group` arm, and the `*_emit_prob` alternative would have silently conscripted the qualifiers into `--profile structured-emission-max`; and the `unique` tool plan needs a second pure-function-of-config predicate, not a widening of `verilator_only` |
| `2026-06-22` | `CAPABILITY-BREADTH-EXPANSION.2b.3` | `book sequential.md "FSM outputs: Moore vs Mealy" + byte-verified runnable example (passing in tests/book_examples; emitted module Verilator -Wall 2012/2017/2023 + Yosys both + Icarus clean); knobs.md fsm_mealy_prob entry + metric-table row + corrected stale line; USER_GUIDE --fsm-mealy-prob row; README --fsm-mealy-prob bullet; new KM card docs/knowledge/fsm-mealy-outputs.md (working reverify, cross-linked to 0024); deferred schema 1.12→1.13 book-example refresh (api-tools/agent-mcp/api-introspection/api-reference; provenance kept at 1.12); mdbook build clean; KM regen+check green (60 facts)` | `done` (docs-only; book back in sync; DUT byte-identical) |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `CAPABILITY-BREADTH-EXPANSION` | `USABILITY-LANE-OWNERSHIP.1 — register 7 owner-directed usability/capability lanes + API-first decision 0017` | Tree registered (not yet started); frontier `.1` (SV up-opt design ADR) + `.2` (Mealy FSM design ADR) pending. |
| `CAPABILITY-BREADTH-EXPANSION.2a` | `CAPABILITY-BREADTH-EXPANSION.2a — Mealy FSM output design ADR (decision 0024)` | Design ADR (docs-only). Pins the Mealy `(state_q, sel)` output model, `fsm_mealy_prob` knob, `num_mealy_fsm_modules` metric (schema `1.13`), `saw_mealy_fsm_design` gate, MCP surface. `.2` split into `.2a` (done) + `.2b` (proposed). |
| `CAPABILITY-BREADTH-EXPANSION.2b.1` | `CAPABILITY-BREADTH-EXPANSION.2b.1 — Mealy FSM output mechanism (knob + IR + emitter + validate)` | First **code** slice of the lane. `Fsm.mealy_outputs` + `fsm_mealy_prob`/`--fsm-mealy-prob` + the emitter nested `case(state_q)→case(sel)` Mealy decode + validate + dedup-exclusion + 2 lib tests. Default-off DUT byte-identical (snapshots 6/6); all-tool-clean. `.2b` split into `.2b.1` (done) + `.2b.2` (metric/gate) + `.2b.3` (docs). |
| `CAPABILITY-BREADTH-EXPANSION.2b.2a` | `CAPABILITY-BREADTH-EXPANSION.2b.2a — Mealy metric num_mealy_fsm_modules + introspection schema 1.13` | The `num_mealy_fsm_modules` `DesignMetrics` field (serde-projected into `--introspect`) + the additive schema bump `1.12 → 1.13` (const + comment + all `schema_version` assertions + the schema-doc contract). Queryable (decision `0017`); default-off byte-identical. `.2b.2` split into `.2b.2a` (done) + `.2b.2b` (gate). |
| `CAPABILITY-BREADTH-EXPANSION.2b.2b` | `CAPABILITY-BREADTH-EXPANSION.2b.2b — Mealy FSM tool_matrix gate (saw_mealy_fsm_design + phase6_mealy_fsm scenario)` | The repo-owned Mealy gate in `src/bin/tool_matrix.rs`: `saw_mealy_fsm_design` coverage fact + `phase6_mealy_fsm` scenario (`fsm_mealy_prob=1.0`) + detection/merge/`Phase4Hierarchy` gap + a non-vacuity test (count 222→225, gate designs 888→900), mirroring the `phase6_fsm`/`phase6_inferrable_memory` motif gates. Banked downstream-clean; default-off DUT byte-identical. `.2b.2` done; `.2b` frontier → `.2b.3` (docs). |
| `CAPABILITY-BREADTH-EXPANSION.3` | `CAPABILITY-BREADTH-EXPANSION.3 — unique/priority case qualifiers: a third breadth strand (decision 0044)` | Design ADR (docs-only). Opens a **third** strand rather than reframing `.1`. Pins the `unique`/`priority` case-qualifier projection of `CaseMux`/`CasezMux`, the two default-off knobs, the metrics (schema `1.27 → 1.28`), the `--case-qualifier-gate` with **per-qualifier** tool plans, and the MCP surface. Banks a 50,761-block corpus measurement + a runtime `--assert` proof with a firing negative control. **Withdraws** the pre-`.1` `CasezMux`-can-overlap claim. `.3` split the strand into `.3` (done) + `.4` (impl). |
| `CAPABILITY-BREADTH-EXPANSION.4b.1` | `CAPABILITY-BREADTH-EXPANSION.4b.1 — unique/priority case qualifiers: the live surface` | First **code** slice of the strand. `src/ir/case_qualifier.rs` (pass + `CaseQualifier` + 15 proofs incl. the FULL/PARALLEL property test) + `Module.case_qualifiers` + the two default-off knobs/flags + two `KnobId` rows + the tenth guarded roll in both `gen/mod.rs` paths + the emitter prefix on the two *unprojected* branches. Default-off DUT byte-identical (snapshots 6/6); 30/30 eligible gates qualified when on; Verilator/Yosys/Icarus clean with identical synthesized cell counts. `.4b` split into `.4b.1` (done) + `.4b.2` (metric/gate) + `.4b.3` (docs). |
| `CAPABILITY-BREADTH-EXPANSION.4b.2b` | `CAPABILITY-BREADTH-EXPANSION.4b.2b — the repo-owned --case-qualifier-gate` | `ScenarioSet::CaseQualifierSweep` + `--case-qualifier-gate`: 13 scenarios (`{unique, priority}` × `{case_mux, casez_mux}` × 3 strategies, plus the co-occurrence one), three coverage facts, metric-keyed detection, and per-qualifier tool plans via a new `scenario_emits_unique_case_qualifier` predicate gating the Icarus column alone. Banked clean at `anvil-case-qualifier-gate-r1` (52 modules, 359 qualified statements, `coverage_gaps = []`, Icarus 28/0 = the non-`unique` modules exactly). `.4b.2` **done**; frontier → `.4b.3`. |
| `CAPABILITY-BREADTH-EXPANSION.4b.2a` | `CAPABILITY-BREADTH-EXPANSION.4b.2a — case-qualifier metrics + introspection schema 1.28` | The two `Metrics::num_emitted_{unique,priority}_cases` counts (one filter each over `Module::case_qualifiers`, `#[serde(default)]`, exact because the pass excludes statically-collapsed *and* chain-projected gates) + the additive schema bump `1.27 → 1.28` across every current-value reference in code, contract doc, live docs and book. **Two** metrics because `.4b.2b` picks a per-qualifier tool plan. Queryable per decision `0017`; default-off DUT byte-identical (snapshots 6/6, lib 764 → 765). `.4b.2` split into `.4b.2a` (done) + `.4b.2b` (the gate); frontier → `.4b.2b`. |
| `CAPABILITY-BREADTH-EXPANSION.4a` | `CAPABILITY-BREADTH-EXPANSION.4a — case-qualifier impl design-detail (decision 0044's five Open Questions resolved)` | Docs-only. Resolves the carrier (`Module.case_qualifiers: BTreeMap<NodeId, CaseQualifier>`, the enum in a new `src/ir/case_qualifier.rs`), the `KnobId` category (`emission`), the metric names + schema `1.27 → 1.28`, the roll precedence (`unique` first, `priority` roll **skipped** on a hit so recorded fires equal the metric), and decision `0032`'s interaction gate (**no** — a qualifier replaces no rendering; the co-occurrence lands inside `--case-qualifier-gate` instead, so the one non-vacuous exclusion is not left unexercised). Pins every `.4b` impl point and the FULL/PARALLEL property-test shape. `.4` split into `.4a` (done) + `.4b` (impl); frontier → `.4b.1`. |
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
- `2026-08-01`: `.3` done — **a third strand opened**, not a reframing of `.1`
  (decision [`0044`](../decisions/0044-capability-breadth-unique-priority-case-qualifiers.md)):
  the `unique` / `priority` **case qualifiers** on `CaseMux` / `CasezMux`, default-off,
  valid-by-construction from the emitter's always-present `default:` arm (FULL) and the
  generator's sequential arm indices (PARALLEL). Banked evidence: 50,761 emitted `case`/`casez`
  blocks measured 100 % FULL + PARALLEL (plus 130 nested FSM blocks), a downstream ON-vs-OFF
  sweep clean at the repo bar with `-Wall` Δ=0 and identical synthesized cell counts, and a
  `verilator --assert` runtime proof of zero violations with a **firing** negative control.
  Two corrections landed: the pre-`.1` `CasezMux`-can-overlap claim is **withdrawn**, and the
  first version of the measurement checker **failed** `DOCTRINE_ENFORCEMENT.md` §9's
  delete-the-subject test and was rewritten before any number from it was trusted. `.1` stays
  `pending`/deferred with its question intact. Frontier → `.4` (impl). Docs-only; DUT
  byte-identical.
- `2026-08-01`: `.4a` done — the case-qualifier **impl design-detail**
  (`DEVELOPMENT_NOTES.md`), resolving all five of decision `0044`'s Open Questions
  against a fresh read of current HEAD. `.4` split into `.4a` (design-detail, done)
  + `.4b` (impl, pre-split `.4b.1` live / `.4b.2` metric+gate / `.4b.3` docs);
  frontier → `.4b.1`. The load-bearing resolutions: the carrier is a **map**
  (`case_qualifiers: BTreeMap<NodeId, CaseQualifier>`), so "at most one qualifier
  per gate" is unrepresentable-otherwise rather than asserted; the precedence
  **skips** the second roll on a hit, so each knob's recorded fires equal its
  metric exactly instead of counting a `priority` fire that emits nothing; and
  decision `0032`'s interaction gate does **not** absorb the two knobs — a
  qualifier replaces no rendering, so the co-occurrence proof lands inside
  `--case-qualifier-gate` as a dedicated scenario, which is also the only thing
  that stops this pass's one **non-vacuous** exclusion from being written,
  unit-tested and never fired end-to-end (the exact hole `0032` itself was opened
  to close). Two non-obvious findings recorded: the knob names force an explicit
  `knob_group` arm — and the `*_emit_prob` alternative would have conscripted the
  qualifiers into `--profile structured-emission-max` through `0032`'s preset
  drift test, costing that preset its Icarus column; and the `unique` tool plan
  needs a **second** pure-function-of-config predicate rather than a widening of
  `verilator_only`, which drops a different pair of columns. Docs-only; no `src/`
  change; DUT byte-identical.
- `2026-08-01`: `.4b.1` done — **the case-qualifier surface is live**. `src/ir/case_qualifier.rs`
  (the pass + `CaseQualifier` + 15 proofs) + `Module.case_qualifiers` + the default-off
  `unique_case_prob` / `priority_case_prob` knobs and their CLI flags + two `KnobId` rows in a new
  **`qualifiers`** category (this line read `emission` when first written, from the superseded `.4a`
  resolution; the correction landed *inside* `.4b.1` and is recorded in its Verification field —
  fixed here rather than left standing, because a wrong statement about shipped code misinforms a
  recovering session, which is not what layer-B's no-retro-edit rule protects)
  + the tenth guarded roll in both `gen/mod.rs` paths + the emitter keyword prefix on the two
  **unprojected** `case`/`casez` branches. Default-off DUT byte-identical (snapshots 6/6, lib
  749 → 764). Each knob at `1.0` qualifies **30 of 30** eligible statements; the ON corpora are
  byte-identical to OFF once the token is stripped, so behaviour preservation is *structural*
  rather than a warning-count delta. Downstream-clean: Verilator `--lint-only` 0/0/0, Yosys both
  repo modes 0 fail / 0 warn with **identical** synthesized cell counts 8/8, Icarus exit 0 —
  `priority` silent, `unique` printing exactly 30 `sorry:` lines and **0** `warning:` lines, which
  reproduces decision `0044`'s measurement and confirms the shared `first_tool_warning` classifier
  would have passed it by accident. The FULL/PARALLEL property test was put through
  `DOCTRINE_ENFORCEMENT.md` §9 **on the real source** — deleting the emitter's `default:` arm and
  widening `build_casez_patterns`' mask each redden it — so the `.3` corpus measurement now has a
  durable home that provably looks. `.4b` split into `.4b.1` (done) + `.4b.2` (metric+gate) +
  `.4b.3` (docs); frontier → `.4b.2`.
- `2026-08-01`: `.4b.2a` done — **the surface is queryable**. `Metrics::num_emitted_unique_cases`
  / `num_emitted_priority_cases` (one filter each over `Module::case_qualifiers`,
  `#[serde(default)]`, set beside the two chain metrics) at introspection schema
  **`1.27 → 1.28`**. `.4b.2` was split into `.4b.2a` (metric+schema, done) + `.4b.2b` (the
  gate) for the reason both sibling surfaces and this tree's own Mealy strand were split: the
  bump touches ~40 version assertions and must land with nothing else moving. **Two** metrics
  rather than one is a downstream fact, not a taste: `.4b.1` measured Icarus printing
  `sorry:` for every `unique` block and staying silent for `priority`, so `.4b.2b` must pick a
  per-qualifier tool plan, and a combined count would force it to re-derive the split from the
  config — the exact shadow (decision `0033`) the metric exists to remove. Exactness was
  **cross-checked against the emitted text** rather than asserted (metric `9` == `grep -c` `9`,
  both qualifiers), and the `.4a` telemetry identity confirmed end-to-end: `fires == metric`,
  and with both knobs at `1.0` `priority_case_prob` records **no attempts at all**, so there is
  no phantom fire to interpret. The bump itself was treated as a **classification** pass —
  current values moved, history (per-feature attributions, prior changelog entries, timeline
  rows, and one anecdote whose whole point is a rotted copied number) did not — and one
  knowledge-card evidence pointer was reworded from `SCHEMA_VERSION = 1.27` to name the const,
  a lesson a sibling card had already recorded. Default-off DUT byte-identical (snapshots 6/6,
  lib 764 → 765). Frontier → `.4b.2b`.
- `2026-08-01`: `.4b.2b` done — **the gate**. `tool_matrix --case-qualifier-gate` +
  `ScenarioSet::CaseQualifierSweep`: 13 scenarios, three independently-required coverage
  facts, metric-keyed detection off the `.4b.2a` metrics, and per-qualifier tool plans via a
  new `scenario_emits_unique_case_qualifier` predicate threaded as a **second** boolean
  beside `verilator_only`. Banked clean —
  [`anvil-case-qualifier-gate-r1`](../evidence/anvil-case-qualifier-gate-r1.md): 52 modules,
  **359** qualified statements, `coverage_gaps = []`, Verilator 52/0, Yosys 52/0 in *both*
  modes, Icarus **28/0** — and 28 is exactly the seven non-`unique` scenarios × 4, so the
  per-qualifier plan is visible in the bank rather than asserted. The thirteenth scenario
  earns its place: it is the only one in which the pass's **non-vacuous** exclusion (a
  chain-projected gate has no `case` keyword to prefix) fires end-to-end, and it runs the
  chain knobs at `0.25` rather than `1.0` precisely because at `1.0` the qualifier would
  claim *nothing* — proving the exclusion but losing the co-occurrence. Two things were
  found by building it: the non-vacuity probe was reading **one** module where the gate runs
  **four**, which is how the co-occurrence fact's 2-of-4 rate (and hence the load-bearing
  unit floor) surfaced; and a comment justifying metric-keying by a vocabulary collision was
  **probed false and retracted** before the commit. `.4b.2` done; frontier → `.4b.3`, which
  now also owns the book's gate section and the digest's `commit` backfill.
