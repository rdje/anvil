# PHASE-7-ORACLE-MICRODESIGN: Oracle-backed micro-design artifacts

## Metadata

- Tree ID: `PHASE-7-ORACLE-MICRODESIGN`
- Status: `done`
- Roadmap lane: Phase 7 — Oracle-backed micro-design artifacts
- Created: `2026-05-16`
- Last updated: `2026-05-20` (**`.2c.2b.2` real-tool parity gate clean — `PHASE-7-ORACLE-MICRODESIGN` tree CLOSED; ROADMAP Phase 7 → done**; verified-clean banked artifact `/tmp/anvil-microdesign-parity-phase7-yosys-p1/` — `cargo test -- --ignored parity_against_real_yosys_write_json` against yosys 0.64 reports "parity gate clean across 5 seeds" with `test result: ok. 1 passed; 0 failed`; per-seed fact agreement verified across the corpus including the previously-divergent seed 7 (P4=-1 → bits=8 on both sides post-`.2c.2b.1` fix) and both generate branches exercised (seed 12345 takes `g_else`); r87 no-aspirational-claims observed; the explicit yosys-supported-categories scope caveat recorded — richer-AST coverage via slang/verilator-with-debug is the recorded post-Phase-7 follow-up that does NOT block closure)
- Owner: repo-local workflow

## Goal

Add a new artifact family: small, self-contained `.sv` files with a
**known expected-facts manifest** (e.g. `rtl_const_expr`-style) — param
/ localparam dependency chains, expression-derived widths and ranges,
generate conditions and loop bounds driven by expressions,
package-qualified constants, precedence-sensitive expressions — each
with a machine-checkable expected-facts contract and downstream parity
checks.

## Non-Goals

- Broad cone complexity / DUT RTL stress — that is the existing Phase
  1–4 lane; Phase 7 is the opposite (tiny, oracle-backed).
- A bundled reference simulator — facts are obviously-checkable
  elaboration facts, not full RTL semantics (project non-goal).
- The artifact-family selector that unifies lanes — that is Phase 9.

## Acceptance Criteria

- Reproducible micro-design corpus generator (seeded, byte-stable).
- Explicit expected-facts manifest per emitted file.
- Parity checks: downstream consumers either agree with the manifest or
  a counterexample is retained.

## Task Tree

- ID: `PHASE-7-ORACLE-MICRODESIGN`
  Status: `done`
  Goal: `Reproducible oracle-backed micro-design corpus with expected-facts contract and downstream parity checks.`
  Children: `PHASE-7-ORACLE-MICRODESIGN.1` (done), `PHASE-7-ORACLE-MICRODESIGN.2` (done — 2026-05-20)

- ID: `PHASE-7-ORACLE-MICRODESIGN.1`
  Status: `done`
  Goal: `Design the micro-design artifact family in DEVELOPMENT_NOTES.md / book: expected-facts schema, generation strategy (param/expr chains), reproducibility contract, parity-check harness shape, relationship to the existing DUT lane, rejected alternatives. Design-only.`
  Acceptance: `Design entry with expected-facts schema sketch and >=1 rejected alternative; mdbook clean; no code change.`
  Verification: `DEVELOPMENT_NOTES.md "Phase 7 oracle-backed micro-design artifact family design (2026-05-18, PHASE-7-ORACLE-MICRODESIGN.1)" entry landed. Records: the conceptual shift (Phases 1-6 = random RTL, no semantic oracle; Phase 7 = tiny .sv whose elaboration facts are known by construction + a machine-checkable manifest — pressure point is front-end constant-expr/param/elaboration correctness). Codebase grounding (the scalar-u32 gate-level circuit IR has no parameter/localparam/generate/package/typed-constant concept; WidthExpr/ParamEnv is width-only; Phase 7 needs its own small source-level constant/parameter IR, a separate generator path, reusing seeding/CLI/reproducibility). rtl_const_expr artifact family per ROADMAP (param/localparam dependency chains; expr-derived widths/ranges; generate if/for; package-qualified constants; precedence-sensitive expressions). Expected-facts JSON manifest schema sketch (params/localparams/widths/generate/package_constants/const_exprs). Oracle-by-construction generation strategy (the generator evaluates every const-expr/param node as it builds it and emits both the .sv and the manifest from the same resolved values — no analysis pass, no re-parse; the generator IS the oracle; valid-by-construction/rules-first). Reproducibility contract (seed,knobs → byte-identical .sv + .json). Parity-check harness (separate from the tool_matrix lint/synth DUT gate; downstream consumer reports resolved facts → compared to manifest; exact agreement or retained counterexample; cargo-portable structural-equivalence formalization + repo-owned gate for the genuine tool parity, mirroring memory/FSM). Boundaries (Phase 8 = richer source-level hierarchy/package IR; Phase 9 = the family selector; Phase 7 lands behind an explicit family flag, no selector). 4 rejected alternatives (reuse circuit IR / generate-then-parse / bundle reference elaborator / facts-as-comments). .2 proof shape + split candidates. Design-only; no code; mdbook build book clean; cargo fmt --all --check clean; full cargo test green at base 5db4ac9 (no src/tests touched).`
  Commit: `Docs: PHASE-7-ORACLE-MICRODESIGN.1 oracle-backed micro-design artifact-family design`

- ID: `PHASE-7-ORACLE-MICRODESIGN.2`
  Status: `done`
  Goal: `Implement the micro-design generator + manifest + parity harness per .1, behind an explicit artifact-family selector flag, with a matrix/parity gate. Split per the Splitting Rules along the exact independently-reviewable boundaries .1's design named (const-expr/parameter IR + construction-time evaluator / SV emitter + manifest emitter / parity harness + repo-owned gate).`
  Children: `PHASE-7-ORACLE-MICRODESIGN.2a` (done), `PHASE-7-ORACLE-MICRODESIGN.2b` (done), `PHASE-7-ORACLE-MICRODESIGN.2c` (done — 2026-05-20)

- ID: `PHASE-7-ORACLE-MICRODESIGN.2a`
  Status: `done`
  Goal: `The source-level const-expr/parameter IR + the construction-time evaluator (the oracle). A small typed parameter+localparam dependency DAG of integer constant expressions with their evaluated values (wide-int semantics matching SV constant-expression rules for the bounded integer subset), reproducible from (seed, knobs) via the existing ChaCha8 stream. Separate generator path; NOT threaded through the gate-level circuit IR. Unit-proven: the evaluator's resolved values match by construction; reproducible byte-stable IR for a fixed seed. No SV/manifest emit yet, no harness.`
  Acceptance: `cargo fmt/clippy(-D warnings)/check --all-targets/test green; new const-expr IR + evaluator with unit proofs (evaluation correctness on a curated expr set incl. precedence/width/localparam-chain cases; reproducibility); no emit/harness; no ROADMAP advance; no book/ change.`
  Verification: `New separate top-level module src/microdesign/mod.rs (registered pub mod microdesign in src/lib.rs; deliberately NOT in src/ir/ — the circuit IR has no param/localparam/expr concept; the category error .1 rejected). IR: ConstExpr{Lit(i128),Param(name),Unary(UnOp{Neg,BitNot,LogNot}),Bin(BinOp{Add,Sub,Mul,Div,Mod,Shl,Shr,BitAnd,BitOr,BitXor,Eq,Ne,Lt,Gt,Le,Ge,LogAnd,LogOr}),Ternary}; ParamDecl{name,kind:Parameter|Localparam,expr,value:i128 (the construction-time-resolved oracle)}; ConstExprUnit{params:Vec<ParamDecl>} = an ordered forward-ref-free dependency DAG. Construction-time evaluator: eval() (SV-constant-expr-style — truncating div/mod toward zero, clamped shift, comparisons/logicals→1/0; defensive EvalError{UndefinedParam,DivByZero}); resolve() fills every ParamDecl.value in declaration order = THE ORACLE (run once at construction; .2b's SV+manifest will read these, never re-derive). build_constexpr_unit(seed,n) = rules-first reproducible builder (ChaCha8::seed_from_u64, project convention, no thread_rng): decl 0 a literal root, each later decl an expr over earlier decls + small literals (parameter/localparam chains, precedence-sensitive a+b*c, ternary-over-comparison), resolved in place (builder IS the oracle — no analysis pass/re-parse). 4 unit proofs green: eval_matches_known_values (precedence 2+3*4=14, (5<<2)|1=21, cmp/logical→1/0, trunc div/mod -7/2=-3 rem -1, ternary+unary, localparam chain A=5;B=A*2;C=B+A→5,10,15), eval_reports_div_by_zero_and_undefined_param (defensive paths), build_is_reproducible_and_seed_sensitive (byte-identical per seed across {0,1,7,42,12345}; distinct seeds differ), stored_values_are_consistent_with_a_fresh_reeval (the load-bearing invariant: stored oracle value == fresh re-eval of each decl's expr over the resolved prefix, seeds 0..16; decl 0 is always Parameter). cargo fmt --all --check / clippy --all-targets -- -D warnings / check --all-targets clean; full cargo test green (COMMIT.md gate). No SV/manifest emit, no harness (.2b/.2c). No ROADMAP/book change.`
  Commit: `Phase 7: PHASE-7-ORACLE-MICRODESIGN.2a const-expr/parameter IR + construction-time evaluator (oracle)`

- ID: `PHASE-7-ORACLE-MICRODESIGN.2b`
  Status: `done`
  Goal: `Emitters: the un-resolved-where-appropriate SV emitter for the const-expr/parameter IR (rtl_const_expr family — param/localparam chains, expr-derived widths/ranges, generate if/for, package-qualified constants, precedence-sensitive expressions) + the JSON expected-facts manifest emitter (params/localparams/widths/generate/package_constants/const_exprs per .1's schema), both emitted from the same evaluated IR (.2a). Behind an explicit artifact-family flag (default off ⇒ DUT lane byte-identical). Cargo-portable structural proof: emitted declarations/manifest are consistent with the evaluator by construction; reproducible.`
  Acceptance: `cargo fmt/clippy/check/test green; forced-on emits valid SV + a schema-valid manifest, byte-reproducible; default-off byte-identical to the DUT lane; structural-consistency proof; no ROADMAP advance.`
  Verification: `src/microdesign/mod.rs extended (same separate module; serde::Serialize added). expr_to_sv() — fully-parenthesized SV pretty-printer (precedence-unambiguous; the .2a builder's nested a+b*c / ternary shapes round-trip as written → the precedence-sensitive-expression axis). emit_sv(unit,seed) emits the rtl_const_expr family as UN-RESOLVED SV: a package mc_<seed>_pkg with localparam int K; a module mc_<seed> with #(parameter int P..=<symbolic expr>) headers (NOT resolved ints), localparam chains in body, localparam int PKG_REF = mc_<seed>_pkg::K (package-qualified constant), localparam int W_SIG = ((<last> % 8)+1) + logic [W_SIG-1:0] sig (expr-derived width), and a generate if (<P0 >= K>) / else (generate if). Manifest structs (pub, serde) + build_manifest()/emit_manifest() produce the .1 schema (seed/top/params/localparams/widths/generate/package_constants/const_exprs) entirely from the .2a resolved value oracle (BTreeMap ⇒ deterministic key order ⇒ byte-stable serde_json pretty). Default-off DUT-byte-identical is trivial+structural: microdesign is a separate module never invoked by the DUT generate path (the Phase-9 selector wires invocation later). 3 new unit proofs (7 total in the module): emit_sv_is_valid_unresolved_shape (package/module/parameter-symbolic/PKG_REF/W_SIG/generate-if-else/endmodule; chained decls render their symbolic expr, not a bare int), manifest_mirrors_the_oracle (valid JSON; every params/localparams value == ParamDecl.value; expr == expr_to_sv; widths.bits/msb, generate.taken, package_constants, const_exprs len all == the oracle), sv_and_manifest_are_byte_reproducible (same seed → identical .sv & .json across rebuilds; distinct seeds differ). cargo fmt --all --check / clippy --all-targets -- -D warnings / check --all-targets clean (fixed a useless format! + a literal-modulo clippy hit by using the real pkg_const helper in the test); full cargo test green (COMMIT.md gate). No parity harness (.2c); no ROADMAP/book change.`
  Commit: `Phase 7: PHASE-7-ORACLE-MICRODESIGN.2b SV + JSON expected-facts manifest emitters (from the .2a oracle)`

- ID: `PHASE-7-ORACLE-MICRODESIGN.2c`
  Status: `done`
  Goal: `The parity harness + repo-owned gate: a downstream consumer (Yosys write_json / slang|Verilator param introspection) reports resolved facts; compare to the manifest — exact agreement or a retained counterexample (SV+manifest+tool output). Tool-gated (cargo test stays green tool-less — Phase-1 doctrine, like memory/FSM .2.2 + DIFFERENTIAL .2b). Then verify a clean run and record ROADMAP Phase 7 -> done (r87 no-aspirational-claims: verified artifact precedes the promotion). Split (Splitting Rules + the proven memory .2.3/.2.4 and FSM .3.4a/.3.4b decomposition: the harness machinery is code that lands before any advance; the real-tool gate run + ROADMAP promotion is a separate gated step) into .2c.1 (parity harness build: cargo-portable comparator proof + tool-gated #[ignore] real-tool harness scaffold; no real run, no ROADMAP advance, cargo stays green tool-less) and .2c.2 (run the real tool-equipped #[ignore] gate, verify clean, record ROADMAP Phase 7 -> done — gate-blocked).`
  Children: `PHASE-7-ORACLE-MICRODESIGN.2c.1` (done), `PHASE-7-ORACLE-MICRODESIGN.2c.2` (done — 2026-05-20)

- ID: `PHASE-7-ORACLE-MICRODESIGN.2c.1`
  Status: `done`
  Goal: `Build the parity harness — cargo-portable + tool-gated. A new top-level test file (e.g. tests/microdesign_parity.rs, mirroring tests/pipeline.rs) carrying: (a) a pure-Rust fact-extraction-and-comparison core that operates on already-collected tool output (an in-test synthetic representation, NOT a tool invocation) — testable cargo-portably without yosys/verilator/slang; (b) a tool-equipped #[ignore]-gated test that, when invoked with the tools available (cargo test -- --ignored), drives a fixed deterministic corpus through emit_sv + emit_manifest, shells the chosen downstream consumer on each .sv, parses the resolved-facts report, and feeds it to the cargo-portable comparator core to assert exact agreement (or retain a counterexample tuple of {sv, manifest, tool_output, divergence}). Tool-gated ⇒ portable cargo test stays green tool-less (Phase-1 doctrine; identical to differential-sim .2b and the convention recorded in this tree's Decisions). Cargo-portable comparator proof: deterministic seeds {0,1,7,42,12345} (matching .2a's reproducibility set) × build → manifest, then feed a hand-constructed-to-agree synthetic tool report to the comparator and prove exact-equality; AND feed a deliberately-perturbed synthetic report and prove the comparator surfaces the right divergence kind (param-mismatch / width-mismatch / generate-branch-mismatch / package-constant-mismatch). No ROADMAP advance (that is .2c.2 on verified evidence).`
  Acceptance: `cargo fmt/clippy(-D warnings)/check --all-targets/test green; new tests/microdesign_parity.rs landed with the cargo-portable comparator + the #[ignore]-gated tool harness; cargo-portable comparator proof exact-agrees on synthetic-agree fixtures and surfaces the right divergence kind on each perturbed fixture; the #[ignore] test compiles + is invocable but is NOT run in the portable suite; ROADMAP unchanged (advance is .2c.2 on a verified gate); no book/ change (book reconciliation is .2c.2).`
  Verification: `src/microdesign/mod.rs extended with the parity comparator core: ToolReport (normalized resolved-facts view a downstream consumer is expected to produce — seed/top + params/localparams as name→i128 + widths as name→WidthFact + generate as name→bool + package_constants as name→i128; BTreeMap throughout for deterministic iteration; serde Serialize+Deserialize for JSON round-trip diagnostics), Divergence enum with 17 variants (SeedMismatch/TopMismatch + {ParamMissingInTool, ParamMissingInManifest, ParamMismatch} × {param, localparam, width, generate, package-constant} categories — independently per axis + per direction so .1's rejected-alternative "single facts-disagree bit" gap is closed), compare_manifest_to_tool_report (cargo-portable walker; no tool invocation; accumulates the full divergence set rather than fail-fast so the gate can either Ok(()) or retain the full counterexample profile in one pass; symbolic manifest expr strings deliberately NOT compared — they are un-resolved-SV documentation, not a fact the tool re-emits), synthetic_tool_report_from_manifest (always-agreeing reference). Promoted FactEntry/WidthFact/GenFact/ConstExprFact/Manifest fields to pub and derived Clone+PartialEq+Eq+Deserialize on each so the comparator and the test harness can construct/compare them by value. New tests/microdesign_parity.rs (mirrors tests/pipeline.rs/snapshots.rs as a top-level integration test) carries 9 cargo-portable comparator proofs (all green): comparator_agrees_on_synthetic_tool_report_built_from_the_oracle (load-bearing baseline — synthetic ToolReport from manifest must agree exactly across the reproducibility-set seeds {0,1,7,42,12345}), comparator_surfaces_param_mismatch_when_a_param_is_perturbed, comparator_surfaces_localparam_mismatch_when_perturbed (exercised >=1 seed; reproducibility-set sanity guard), comparator_surfaces_width_mismatch_when_perturbed (on widths["sig"] which .2b always emits), comparator_surfaces_generate_branch_mismatch_when_taken_is_flipped (on generate["g_taken"]), comparator_surfaces_package_constant_mismatch_when_perturbed, comparator_surfaces_param_missing_in_tool_when_dropped, comparator_surfaces_param_missing_in_manifest_when_extra (spurious-extra-report defensive coverage), comparator_surfaces_seed_and_top_mismatch_when_perturbed (the stale-or-mis-routed-report check) + 1 tool-gated #[ignore] real-tool harness scaffold parity_against_real_yosys_write_json (yosys-presence guard at the head so the scaffold is safely invocable on machines without the tool — the iverilog-not-installed convention from DIFFERENTIAL-SIMULATION.1; instantiates the full corpus driver loop with placeholder for the .2c.2-owned emit→shell→extract→compare end-to-end wiring). cargo fmt --all --check / cargo clippy --all-targets -- -D warnings / cargo check --all-targets clean. Full cargo test green: tests/microdesign_parity 9 passed + 1 ignored; tests/pipeline 121 passed; tests/snapshots 6 passed; doc-tests 0 (unchanged); lib microdesign tests still 7/7 green (.2a + .2b unchanged). The portable cargo test stays green tool-less; the tool-gated harness is invocable only via cargo test -- --ignored AND when yosys is on $PATH. No ROADMAP advance (that is .2c.2 on a verified clean banked artifact, r87 no-aspirational-claims). No book/ change (book reconciliation is .2c.2).`
  Commit: `Phase 7: PHASE-7-ORACLE-MICRODESIGN.2c.1 parity harness — comparator core + cargo-portable proofs + tool-gated #[ignore] scaffold`

- ID: `PHASE-7-ORACLE-MICRODESIGN.2c.2`
  Status: `done`
  Goal: `Real tool-equipped run of the .2c.1 #[ignore]-gated parity harness against a fixed deterministic corpus; VERIFY exact-agreement (or zero retained counterexamples) BEFORE any promotion (r87 no-aspirational-claims, mirroring memory .2.4 and FSM .3.4b). Then record ROADMAP Phase 7 -> done (with the explicit artifact-family lane note and the boundary to Phase 8/Phase 9 preserved); reconcile book (the Phase-7 micro-design lane in book/src/ir.md and/or a new "Micro-design lane" page in the book), README phase narrative, CODEBASE_ANALYSIS phase-coverage-map Phase-7 row, MEMORY recent commits. Closes PHASE-7-ORACLE-MICRODESIGN.2c + the .2 container + the PHASE-7-ORACLE-MICRODESIGN tree. Split (Splitting Rules + the proven memory .2.1->.2.1a/.2.1b discovered-dependency-split precedent: implementing the yosys-specific extractor + scoped comparator is itself signoff-sized code that lands BEFORE any verified-clean banked artifact can exist, exactly as memory's compaction-reachability was a load-bearing lower-level dependency that justified the .2.1 split) into .2c.2a (FactCategory + ParityScope + scoped comparator + yosys-specific write_json extractor + end-to-end-runnable #[ignore] test; cargo stays green tool-less; no real run, no ROADMAP advance) and .2c.2b (run the #[ignore] gate against real yosys; verify exact-agreement on yosys-supported categories; bank the artifact; record ROADMAP Phase 7 -> done; book/README/CODEBASE reconcile; gate-blocked).`
  Children: `PHASE-7-ORACLE-MICRODESIGN.2c.2a` (done), `PHASE-7-ORACLE-MICRODESIGN.2c.2b` (done — 2026-05-20)

- ID: `PHASE-7-ORACLE-MICRODESIGN.2c.2a`
  Status: `done`
  Goal: `Land the extractor + scoped comparator + end-to-end-runnable #[ignore] harness. Empirical probe (recorded in Decisions 2026-05-20 below) confirmed yosys 0.64 write_json exposes resolved parameter values via .parameter_default_values (binary-string form interpreted as SV "int" → signed 32-bit → i128), the elaborated-generate-branch via the netname prefix (g_taken.gflag vs g_else.gflag — both branches reachable across the reproducibility-set seeds), and the top wire "sig" width via .netnames["sig"].bits. Localparams and package_constants are folded by yosys and not name-introspectable, so the parity scope yosys covers is the 4-axis (Seed, Top, Params, Generate; with optional Widths["sig"]). src/microdesign/mod.rs: add pub enum FactCategory (Seed/Top/Params/Localparams/Widths/Generate/PackageConstants — one per axis the comparator already enumerates), pub struct ParityScope { categories: BTreeSet<FactCategory> } with all()/none()/only(...) constructors, pub fn compare_manifest_to_tool_report_in_scope(m, t, scope) — the existing compare_manifest_to_tool_report becomes the all-categories case. tests/microdesign_parity.rs: yosys_write_json_to_tool_report extractor (parses .parameter_default_values, scans .netnames for the g_taken./g_else. prefix to populate generate["g_taken"], pulls .netnames["sig"].bits into a WidthFact for widths["sig"]), the yosys ParityScope used by the harness (Seed/Top/Params/Generate/Widths), and the parity_against_real_yosys_write_json #[ignore] test rewritten to be end-to-end-runnable: for each seed in SEEDS, write emit_sv to a tmp file, shell yosys with the documented script (read_verilog -sv <sv>; hierarchy -top mc_<seed>; proc; opt; write_json <out.json>), parse the output, build a ToolReport, call compare_manifest_to_tool_report_in_scope with the yosys scope, assert Ok(()) or retain the counterexample tuple. Plus 1+ cargo-portable proofs that exercise the scoped comparator (e.g. compare_in_scope_ignores_categories_not_in_scope: an out-of-scope category divergence must NOT surface; an in-scope category divergence DOES). No real cargo-test run of the #[ignore] (that is .2c.2b); cargo stays green tool-less; ROADMAP unchanged.`
  Acceptance: `cargo fmt/clippy(-D warnings)/check --all-targets/test green; FactCategory + ParityScope + scoped comparator land + scoped-comparator proof green; yosys_write_json_to_tool_report extractor lands; #[ignore] test is end-to-end runnable (compiles; invocable with `cargo test -- --ignored` AND yosys on $PATH; no longer a no-op-with-placeholder); portable cargo test stays green tool-less (the #[ignore] is NOT run in the portable suite); ROADMAP unchanged (advance is .2c.2b on a verified run); no book/ change (book reconciliation is .2c.2b).`
  Verification: `src/microdesign/mod.rs extended: pub enum FactCategory{Seed,Top,Params,Localparams,Widths,Generate,PackageConstants} (7 variants — one per fact axis the comparator enumerates); pub struct ParityScope{categories: BTreeSet<FactCategory>} with all()/none()/only(&[...]) constructors + .contains(category); pub fn compare_manifest_to_tool_report_in_scope(manifest, report, scope) — the scoped walker that skips out-of-scope axes entirely (no MissingIn*/Mismatch variants surface for skipped categories). The existing strict compare_manifest_to_tool_report now delegates to compare_manifest_to_tool_report_in_scope(m, r, &ParityScope::all()) — backwards-compatible by construction (every previously-passing call still passes; the 9 .2c.1 proofs unchanged-green). tests/microdesign_parity.rs extended with 6 new tests (3 scoped-comparator + 3 yosys-extractor): scoped_comparator_only_enforces_scoped_categories (params-only scope ignores width perturbation; in-scope param perturbation surfaces ParamMismatch — the load-bearing scoping proof), yosys_scope_ignores_localparams_and_package_constants (yosys_write_json_scope = only(&[Seed,Top,Params,Widths,Generate]); empty localparams + empty package_constants in the report — matching what yosys actually reports because it folds them — compare Ok(()) under the yosys scope; the SAME empty maps must surface PackageConstantMissingInTool under ParityScope::all() — the strict-vs-scoped delta), empty_scope_ignores_every_disagreement (ParityScope::none() must Ok(()) even on a maximally-disagreeing report — self-check on the scoping implementation). Yosys extractor: pub-helpers (test-local) parse_yosys_binary_param(s) — parses yosys's binary-string parameter values as SV int (signed 32-bit → i128 via u32 → i32 cast for sign-extension; defensive on empty/non-binary/>32-bit inputs); yosys_write_json_to_tool_report(json, seed) — populates ToolReport.params from .modules.mc_<seed>.parameter_default_values, .generate["g_taken"] from a netnames-key-prefix scan (g_taken. vs g_else. — surviving prefix tells us which branch was kept; the .2b corpus exercises BOTH per .3.4b precedent — seed 12345 takes g_else, others take g_taken), .widths["sig"] from .netnames.sig.bits.len(). Folded axes (localparams, package_constants) deliberately empty. Extractor proofs: yosys_extractor_reads_a_synthetic_write_json_correctly (hand-built JSON for seed 0 matches: P0=46, g_taken=true, widths.sig.bits=6; folded axes empty), yosys_extractor_reports_g_else_when_else_branch_survives (the g_else-survives case for seed-12345-shape input → generate.g_taken=false), parse_yosys_binary_param_sign_extends (1...1 → -1; 0...01 → 1; 0..101110 → 46; empty/'z'/33-bit inputs → None — the -1 cell is load-bearing because .2a's builder can produce negative resolved values, e.g. seed 7 P4 = -1). parity_against_real_yosys_write_json rewritten end-to-end: for each seed in SEEDS, emit_sv → CARGO_TARGET_TMPDIR/microdesign-parity-phase7-yosys/mc_<seed>.sv, emit_manifest → mc_<seed>.json, shell yosys -q -p "read_verilog -sv ...; hierarchy -top mc_<seed>; proc; opt; write_json ...", parse → ToolReport, call scoped comparator with yosys_write_json_scope, accumulate counterexamples, panic with full diagnostic on non-empty counterexample list (or eprintln "parity gate clean across N seeds"); yosys-presence guard at the head keeps the harness invocable on machines without the tool. NO real cargo-test run of the #[ignore] (that is .2c.2b's deliverable); the test is INVOCABLE end-to-end but the portable cargo test stays green tool-less (15 passed + 1 ignored). cargo fmt --all --check / cargo clippy --all-targets -- -D warnings / cargo check --all-targets clean. Full cargo test green: tests/microdesign_parity 15 passed + 1 ignored; tests/pipeline 121 passed (661s); tests/snapshots 6 passed; lib 228 passed (microdesign 7/7 + the rest unchanged); doc-tests 0; bin tests 5+29+3 passed. No ROADMAP advance (that is .2c.2b on a verified clean banked artifact, r87). No book/ change.`
  Commit: `Phase 7: PHASE-7-ORACLE-MICRODESIGN.2c.2a scoped comparator + yosys write_json extractor + end-to-end-runnable #[ignore] harness`

- ID: `PHASE-7-ORACLE-MICRODESIGN.2c.2b`
  Status: `done`
  Goal: `Run the #[ignore]-gated parity gate against real yosys, verify exact-agreement on the yosys-supported categories (Seed/Top/Params/Widths/Generate) across the full corpus, bank a verified-clean banked artifact (under /tmp/anvil-microdesign-parity-phase7-yosys-p1/ or similar repo-local convention), then promote ROADMAP Phase 7 → done with the explicit "yosys-supported categories" caveat (localparams and package_constants are folded by yosys and remain visible only to richer-AST tools like slang/verilator-with-debug — additional categories enter Phase 7 via follow-up extractors); reconcile book (book/src/ir.md "Phase 7 micro-design lane" entry or new page), README phase narrative, CODEBASE_ANALYSIS phase-coverage-map Phase-7 row, MEMORY recent commits. Closes PHASE-7-ORACLE-MICRODESIGN.2c + .2 + the tree. Split (Splitting Rules + the proven memory .2.1→.2.1a/.2.1b discovered-dependency precedent — repeated here at one level deeper): the very first real-tool run of the .2c.2a end-to-end harness on seed 7 surfaced a single WidthMismatch{name:"sig", expected:bits=8, actual:bits=2}. Root cause (recorded in Decisions): width_expr emits the SV text "((P4 % 8) + 1)" but computes its oracle as last.value.rem_euclid(8) + 1; for seed 7 last.value = P4 = -1, so the oracle reports 7+1=8 (mathematical non-negative modulo) but SV evaluates (-1 % 8) + 1 = 0 (truncated-toward-zero modulo on signed values), and yosys interprets logic [W_SIG-1 : 0] = logic [-1:0] as 2 bits. The oracle and the SV disagree for negative last values — an ANVIL-self-consistency bug, not a yosys bug. ANVIL's "valid-by-construction + downstream-acceptance-quality" north-star (the framing user-confirmed 2026-05-18) requires fixing this before ROADMAP Phase 7 can be promoted. Split into .2c.2b.1 (fix the semantic alignment: change BOTH the SV text and the oracle to the standard SV non-negative-modulo idiom "(((x % 8) + 8) % 8) + 1" so the width is always in [1, 8] and oracle ≡ SV; add a regression proof exercising negative-value seeds; cargo gates green; no real run, no ROADMAP advance) and .2c.2b.2 (re-run the #[ignore] gate against real yosys, verify clean across the corpus, bank artifact, record ROADMAP Phase 7 → done with the explicit "yosys-supported categories" caveat, reconcile book/README/CODEBASE; gate-blocked, r87).`
  Children: `PHASE-7-ORACLE-MICRODESIGN.2c.2b.1` (done), `PHASE-7-ORACLE-MICRODESIGN.2c.2b.2` (done — 2026-05-20)

- ID: `PHASE-7-ORACLE-MICRODESIGN.2c.2b.1`
  Status: `done`
  Goal: `Fix the oracle/SV semantic-alignment bug in width_expr surfaced by .2c.2a's first real-tool run. The .2b emitter currently has SV text "((<last> % 8) + 1)" but the oracle uses last.value.rem_euclid(8) + 1; these diverge whenever last.value is negative (Rust's rem_euclid is mathematical non-negative modulo; SV's "%" on signed values is truncated toward zero — they only agree for non-negative dividends). Change BOTH the SV text and the oracle to the standard SV non-negative-modulo idiom: SV "((((<last> %% 8) + 8) %% 8) + 1)" and Rust "((last.value %% 8 + 8) %% 8 + 1) as u32". The result is always in [1, 8] for any last.value (positive or negative; well-defined; matches yosys's literal SV evaluation). Add a regression proof in src/microdesign/mod.rs::tests that exercises the negative-value branch: build a unit whose last decl resolves to a negative value (e.g. force one of the .2a builder's seeds where this happens — seed 7 P4 = -1 from today's probe — and assert build_manifest's widths["sig"].bits matches what the SV literally evaluates to under the new idiom). No ROADMAP advance.`
  Acceptance: `cargo fmt/clippy(-D warnings)/check --all-targets/test green; width_expr's SV text and oracle BOTH use the non-negative-modulo idiom; regression proof landed and green; .2a/.2b/.2c.1/.2c.2a portable proofs still green (the change is to the width FORMULA — manifest_mirrors_the_oracle continues to hold by construction since both sides moved together; sv_and_manifest_are_byte_reproducible re-baselines under the new idiom); ROADMAP unchanged (advance is .2c.2b.2 on a verified real-tool run); no book/ change (book reconciliation is .2c.2b.2).`
  Verification: `src/microdesign/mod.rs::width_expr: SV text changed from "(({} % 8) + 1)" to "((({} % 8) + 8) % 8 + 1)" — the standard SV non-negative-modulo idiom (a SV elaborator evaluates this to the same value Rust's rem_euclid produces for the oracle; ((-1 % 8) + 8) % 8 + 1 = (-1 + 8) % 8 + 1 = 7 + 1 = 8 in BOTH Rust and SV). The oracle's bits formula (last.value.rem_euclid(8) + 1) was left UNCHANGED because it was already correct — only the SV text needed to catch up to it. Comment block at the call site records the root cause + the fix rationale + the SV-vs-Rust modulo-semantic delta. New lib regression test width_expr_uses_sv_non_negative_modulo_idiom_and_agrees_for_negative_last_values (3 axes): (a) the .2c.2a counterexample fixture — seed 7's reproducible P4=-1 must produce widths["sig"].bits = 8 (the bug case, before the fix the SV would evaluate to W_SIG=0); (b) a non-negative-last fixture — seed 0's P4=365 stays at the same bits the old formula produced (the new idiom collapses to the old form for non-negative dividends); (c) cross-seed structural pin — every reproducibility-set seed's W_SIG line uses the new idiom textually (no regression to the old form). Updated the existing emit_sv_is_valid_unresolved_shape's substring pin from "localparam int W_SIG = ((P" to "localparam int W_SIG = (((P" to match the new emit (three open parens before the param name). cargo fmt --all --check / cargo clippy --all-targets -- -D warnings / cargo check --all-targets clean. Full cargo test green: lib 229 passed (was 228 + 1 new regression proof); microdesign 8/8 (was 7/7 + regression); tests/microdesign_parity 15 passed + 1 ignored (every .2c.1 + .2c.2a portable proof still green — the change is structural-only to the formula, and manifest_mirrors_the_oracle continues to hold because BOTH sides of the equality moved in lockstep; sv_and_manifest_are_byte_reproducible re-baselines without code change because the BUILD process re-runs the new formula deterministically); tests/pipeline 121 passed (658s); tests/snapshots 6 passed; bin tests 5+29+3 passed; doc-tests 0 (unchanged). Portable cargo test stays green tool-less. The #[ignore] real-tool gate is now unblocked: .2c.2b.2's task is to run it, verify clean (the seed-7 counterexample is now structurally impossible to surface), bank the artifact, and promote ROADMAP Phase 7. No ROADMAP advance in this slice. No book/ change.`
  Commit: `Phase 7: PHASE-7-ORACLE-MICRODESIGN.2c.2b.1 width_expr oracle/SV semantic alignment — non-negative-modulo idiom + regression proof`

- ID: `PHASE-7-ORACLE-MICRODESIGN.2c.2b.2`
  Status: `done`
  Goal: `Real tool-equipped re-run of the .2c.2a #[ignore] gate (now with .2c.2b.1's alignment fix in place); VERIFY exact-agreement on the yosys-supported categories (Seed/Top/Params/Widths["sig"]/Generate) across the full corpus, with the previously-divergent seed 7 now clean. Bank the verified-clean artifact (CARGO_TARGET_TMPDIR/microdesign-parity-phase7-yosys/{*.sv, *.json, *.yosys.json} + a recorded harness output snippet in the Verification Log). Record ROADMAP Phase 7 → done with the explicit "yosys-supported categories" scope caveat (localparams and package-constants remain visible only to richer-AST tools — slang/verilator-with-debug — and are recorded as a post-Phase-7 follow-up that does NOT block closure; ANVIL's by-construction oracle already covers all 7 categories). Reconcile book (book/src/ir.md or a new "Phase 7 micro-design lane" page), README phase narrative, CODEBASE_ANALYSIS phase-coverage-map Phase-7 row, MEMORY recent commits. Closes PHASE-7-ORACLE-MICRODESIGN.2c.2 + .2c + .2 container + PHASE-7-ORACLE-MICRODESIGN tree.`
  Acceptance: `Banked artifact captures the gate's exact-agreement on the corpus (zero retained counterexamples after .2c.2b.1's fix); ROADMAP Phase 7 → done only after the verified clean run; .2c.2b + .2c.2 + .2c + .2 container + tree all → done. No aspirational claims (verified artifact precedes the ROADMAP promotion).`
  Verification: `Real tool-equipped re-run of the .2c.2a #[ignore] parity gate against yosys 0.64 (with .2c.2b.1's width_expr non-negative-modulo idiom in place): cargo test --test microdesign_parity -- --ignored parity_against_real_yosys_write_json --nocapture exited 0 with stdout "parity gate clean across 5 seeds; artifacts in /Users/richarddje/Documents/github/anvil/target/tmp/microdesign-parity-phase7-yosys" and "test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 15 filtered out; finished in 0.04s". Per-seed fact agreement verified before promotion (r87 no-aspirational-claims): seed 0 — manifest {P0=46, widths.sig.bits=6, g_taken=true} matches yosys {P0='00...101110'=46, sig 6-bit, netname g_taken.gflag}; seed 1 — {P0=26, P2=27, bits=8, taken=true} matches {P0=26, P2=27 binary, sig 8-bit, g_taken}; seed 7 (the previously-divergent case) — {P0=9, P3=3, P4=-1, bits=8, taken=true} matches {P0=9, P3=3, P4='111...1'=-1 (sign-extended correctly), sig 8-bit, g_taken} — the .2c.2b.1 fix worked; seed 42 — {P0=44, bits=3, taken=true} matches {P0=44, sig 3-bit, g_taken}; seed 12345 — {P0=43, P3=47, P4=186, bits=3, taken=false} matches {P0=43, P3=47, P4=186, sig 3-bit, netname g_else.gflag} — the g_else branch is also covered. **Both generate branches are exercised by the corpus** (4 seeds take g_taken, 1 seed takes g_else); **negative parameter values are correctly sign-extended** (seed 7 P4=-1 from yosys's "111...1" binary string); **the previously-divergent seed 7 widths["sig"] now agrees** (oracle and yosys both report bits=8). Banked artifact copied to /tmp/anvil-microdesign-parity-phase7-yosys-p1/ per established convention (15 files — 5 × {mc_<seed>.sv, mc_<seed>.json, mc_<seed>.yosys.json}). Promotion strictly followed the verified artifact: ROADMAP Phase 7 (not started)→(done) with the verified-clean artifact citation + the explicit yosys-supported-categories scope caveat (Seed/Top/Params/Widths/Generate; localparams + package_constants are folded by yosys and remain visible only to richer-AST tools — slang/verilator-with-debug — recorded as a post-Phase-7 follow-up that does NOT block closure since ANVIL's by-construction oracle already covers all 7 categories); book/src/ir.md gains a Phase 7 micro-design lane delivered note citing the artifact; README phase narrative Phase 7 → done; CODEBASE_ANALYSIS phase-coverage-map Phase-7 row → done (2026-05-20). Multi-clock CDC remains the explicitly-optional, separately-prioritised deferral (per Phase 6 closure note). No src/ or tests/ change in this slice (the code landed in .2a/.2b/.2c.1/.2c.2a/.2c.2b.1); mdbook build book clean; cargo unchanged-green. **Closes the .2c.2b container; closes the .2c.2 container; closes the .2c container; closes the .2 container; closes the PHASE-7-ORACLE-MICRODESIGN tree; closes ROADMAP Phase 7.**`
  Commit: `Phase 7: PHASE-7-ORACLE-MICRODESIGN.2c.2b.2 parity gate clean against yosys — closes Phase 7 + tree`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| — | (closed) | (tree done) | **`PHASE-7-ORACLE-MICRODESIGN` tree CLOSED (2026-05-20).** `.2c.2b.2`'s real-tool parity gate against yosys 0.64 came back clean across all 5 reproducibility-set seeds (`test result: ok. 1 passed; 0 failed`); the verified-clean artifact was banked at `/tmp/anvil-microdesign-parity-phase7-yosys-p1/` BEFORE promotion (r87 no-aspirational-claims); per-seed fact agreement verified including the previously-divergent seed 7 (P4=-1 → bits=8 on both sides post-`.2c.2b.1` fix) and both generate branches exercised (seed 12345 takes `g_else`, others `g_taken`); negative parameter values correctly sign-extended through the yosys binary-string format. **ROADMAP Phase 7 closed** with the explicit yosys-supported-categories scope caveat — richer-AST coverage via slang or verilator-with-debug remains a recorded post-Phase-7 follow-up that does NOT block closure (ANVIL's by-construction oracle already covers all 7 categories; the parity gate exercises what the tool reports). Continuous-PNT continues on the still-open trees (`PHASE-8-FRONTEND-ACCEPT.2`, `PHASE-9-MULTI-ARTIFACT-UMBRELLA.2`, `DIFFERENTIAL-SIMULATION.2b`). |

## Decisions

- `2026-05-16`: Phase 7 introduces a *second* artifact lane; it must not
  overload the existing DUT generator path (the doctrinal lane
  separation is preserved here and unified later in Phase 9).
- `2026-05-18`: **`.2` split** into `.2a` (const-expr/parameter IR +
  construction-time evaluator/oracle), `.2b` (SV emitter + JSON
  manifest emitter, behind an artifact-family flag), `.2c` (parity
  harness + repo-owned gate → ROADMAP Phase 7). Splitting Rules
  along the exact independently-reviewable boundaries `.1`'s design
  named ("const-expr/parameter IR + construction-time evaluator /
  SV emitter + manifest emitter / parity harness + repo-owned
  gate"); each is separately reviewable and `.2a`'s evaluator +
  `.2b`'s manifest core are the dependency `PHASE-8-FRONTEND-ACCEPT.2`
  and the Phase-9 manifest plumbing reuse. Tree-planning, docs-only
  (~zero contention on the near-complete Phase 6 priority gate —
  the same contention-aware discipline applied all session). `.2`
  is now a container; no renumbering. Frontier → `.2a`.
- `2026-05-20`: **`.2c` split** into `.2c.1` (build the parity
  harness — cargo-portable comparator proof + tool-gated `#[ignore]`
  real-tool harness scaffold; no real run, no ROADMAP advance,
  cargo stays green tool-less per the Phase-1 doctrine recorded in
  this tree's Decisions) and `.2c.2` (real tool-equipped run of
  the harness + verify exact-agreement + record **ROADMAP Phase 7 →
  done**; gate-blocked, r87 no-aspirational-claims). Splitting Rules
  + the proven memory `.2.3`/`.2.4` and FSM `.3.4a`/`.3.4b`
  decomposition: the harness code that lands BEFORE any advance is
  one signoff-sized leaf; the gated real run + ROADMAP promotion +
  book reconcile is a separate gated step. `.2c` is now a
  container; no renumbering. Frontier → `.2c.1`.
- `2026-05-20` (**`.2c.2b` split — first real-tool run surfaced an
  ANVIL-self-consistency bug**): the inaugural execution of the
  `.2c.2a` `#[ignore]` parity gate (`cargo test -- --ignored
  parity_against_real_yosys_write_json` against locally-installed
  yosys 0.64, immediately after `.2c.2a` landed at `900061c`)
  retained exactly one counterexample: seed 7,
  `WidthMismatch { name: "sig", expected: bits=8, actual: bits=2 }`.
  Root cause: `width_expr` (in `src/microdesign/mod.rs`) emits SV
  text `((<last> % 8) + 1)` but uses Rust's
  `last.value.rem_euclid(8) + 1` for the oracle. Rust's
  `rem_euclid` is the *mathematical non-negative modulo*
  (`(-1).rem_euclid(8) = 7`) while SV's `%` on signed integers is
  *truncated toward zero* (`-1 % 8 = -1`, identical to Rust's
  `%`). For seed 7 `P4 = -1`: the oracle says `bits = 7 + 1 = 8`;
  the SV evaluates `(-1 % 8) + 1 = 0` and yosys interprets
  `logic [W_SIG-1:0]` with `W_SIG = 0` as `[-1:0]` ⇒ 2 bits.
  Oracle ≠ SV ⇒ ANVIL-self-consistency bug (NOT a yosys bug).
  ANVIL's "valid-by-construction + downstream-acceptance-quality"
  north-star (the framing the user confirmed 2026-05-18) requires
  fixing this BEFORE ROADMAP Phase 7 can be promoted (r87
  no-aspirational-claims: a known counterexample is not a clean
  banked artifact). Per Splitting Rules + the proven memory
  `.2.1`→`.2.1a`/`.2.1b` discovered-dependency-split precedent
  (the same precedent applied two levels up at the `.2c` and
  `.2c.2` splits — repeated here one level deeper), `.2c.2b` was
  split into `.2c.2b.1` (semantic-alignment fix: change BOTH the
  SV text and the oracle to the standard SV non-negative-modulo
  idiom `((x % 8) + 8) % 8 + 1` so the width is always in
  `[1, 8]` and oracle ≡ SV; a regression proof exercising a
  negative-value seed; cargo gates green; no real run, no ROADMAP
  advance) and `.2c.2b.2` (re-run the `#[ignore]` gate against
  real yosys + verify clean across the corpus + bank the artifact
  + record **ROADMAP Phase 7 → done** with the explicit
  yosys-supported-categories scope caveat; gate-blocked, r87).
  `.2c.2b` is now a container; no renumbering. This is the parity
  gate doing exactly what `.1` designed it to do — surface
  semantic disagreement between oracle and downstream — and the
  fix lands as the next slice. Frontier → `.2c.2b.1`.
- `2026-05-20` (**`.2c.2` split — discovered lower-level
  dependency**): an empirical probe of yosys 0.64's `write_json`
  output for the `.2b` `mc_<seed>` corpus (seeds `{0,1,7,42,12345}`,
  `N_PARAMS=5`) showed yosys exposes resolved parameter values in
  `.modules.<top>.parameter_default_values` (binary-string form
  interpreted as SV `int` → signed 32-bit → `i128`), the
  elaborated-generate-branch via `.modules.<top>.netnames` keys
  prefixed by `g_taken.` or `g_else.` (the corpus exercises both —
  seed 12345 takes `g_else`, the others take `g_taken`), and the
  top wire `sig` width via `.modules.<top>.netnames["sig"].bits`.
  Localparams and package-constants (`mc_<seed>_pkg::K`) are
  **folded by yosys** and not name-introspectable from `write_json`
  alone — they require richer-AST tools (`slang --ast-json`,
  `verilator --xml-only`). The parity scope yosys actually covers
  is therefore the 4-axis (`Seed`, `Top`, `Params`, `Generate`)
  plus the partial `Widths["sig"]` cross-check. This is a
  **discovered lower-level dependency** in the spirit of the
  memory `.2.1` split (compaction-reachability for an opaque
  stateful leaf was load-bearing pipeline code that justified
  splitting `.2.1` into `.2.1a`/`.2.1b`). Per Splitting Rules,
  `.2c.2` is split into `.2c.2a` (`FactCategory` + `ParityScope`
  + scoped comparator in `src/microdesign/` + yosys-specific
  extractor + end-to-end-runnable `#[ignore]` test, with one+
  cargo-portable proof of the scoped comparator; cargo stays
  green tool-less; no real run, no ROADMAP advance) and `.2c.2b`
  (run the `#[ignore]` gate against real yosys + verify
  exact-agreement on yosys-supported categories + bank the
  artifact + record **ROADMAP Phase 7 → done** with the explicit
  scope caveat; book/README/CODEBASE reconcile; gate-blocked,
  r87). `.2c.2` is now a container; no renumbering. The richer
  fact-category coverage via slang/verilator-with-debug is a
  recorded follow-up that does not block `.2c.2b` — ANVIL's
  by-construction oracle already covers all 7 categories; the
  gate exercises whatever the tool reports. Frontier → `.2c.2a`.

## Open Questions

- Manifest format (JSON schema vs sidecar comments) — **resolved by
  `.1`**: a typed **JSON manifest** per `.sv` (params/localparams/
  widths/generate/package_constants/const_exprs). Sidecar comments
  rejected (not machine-checkable without re-parsing; couples the
  oracle to comment formatting).
- Whether the parity harness reuses `tool_matrix` or is new —
  **resolved by `.1`**: a **new, separate** parity harness (the
  `tool_matrix` gate proves lint/synth *acceptance*; Phase 7 proves
  *fact agreement* — a different contract). Cargo-portable
  structural-equivalence formalization + a repo-owned gate for the
  genuine downstream parity (cargo cannot shell yosys/verilator —
  the Phase-1 convention), mirroring memory/FSM.

## Blockers

- None for `.1`. `.2` benefits from but is not hard-blocked by Phase 5
  parameterization; `.1` will record whether `.2` should wait.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-05-18` | `PHASE-7-ORACLE-MICRODESIGN.1` | `DEVELOPMENT_NOTES.md` Phase 7 design entry landed (conceptual shift; codebase grounding — own source-level const/param IR, separate generator path; `rtl_const_expr` family; expected-facts JSON schema; oracle-by-construction generation; reproducibility; new parity harness; Phase-8/9 boundaries; 4 rejected alternatives; `.2` split). Design-only, no code; `mdbook build book` clean; `cargo fmt --all --check` clean; full `cargo test` green at base `5db4ac9` (no `src/`/`tests/` touched). | Done. |
| `2026-05-18` | `PHASE-7-ORACLE-MICRODESIGN.2` (split) | `.2` made a container with children `.2a` (const-expr/parameter IR + construction-time evaluator/oracle), `.2b` (SV + JSON-manifest emitters behind an artifact-family flag), `.2c` (parity harness + repo-owned gate → ROADMAP Phase 7) — the exact independently-reviewable boundaries `.1`'s design named. Tree-planning, docs-only; no `src/`/`tests/` (cargo unchanged-green vs base `e550db1`). | Done. Frontier → `.2a`. |
| `2026-05-19` | `PHASE-7-ORACLE-MICRODESIGN.2a` | New separate top-level `src/microdesign/mod.rs` (`pub mod microdesign`; not in `src/ir/`): `ConstExpr`/`UnOp`/`BinOp`/`ParamKind`/`ParamDecl`(+`value` oracle)/`ConstExprUnit` IR; `eval()` (SV-constant-expr semantics — trunc div/mod, clamped shift, cmp/logical→1/0, defensive `EvalError`); `resolve()` = the construction-time oracle (fills every value in decl order); `build_constexpr_unit(seed,n)` rules-first reproducible builder (`ChaCha8::seed_from_u64`, no `thread_rng`; literal root + earlier-decl chains/precedence/ternary; resolved in place). 4 unit proofs green: `eval_matches_known_values`, `eval_reports_div_by_zero_and_undefined_param`, `build_is_reproducible_and_seed_sensitive`, `stored_values_are_consistent_with_a_fresh_reeval` (the oracle-no-drift invariant). `cargo fmt --all --check`/`clippy --all-targets -- -D warnings`/`check --all-targets` clean; full `cargo test` green (COMMIT.md gate). No SV/manifest emit, no harness; no ROADMAP/book change. | Done. Frontier → `.2b`. |
| `2026-05-19` | `PHASE-7-ORACLE-MICRODESIGN.2b` | `src/microdesign/` extended: `expr_to_sv` (fully-parenthesized precedence-unambiguous printer), `emit_sv(unit,seed)` (un-resolved `rtl_const_expr` SV — `package mc_<seed>_pkg`/`K`, module with symbolic `parameter`/`localparam` chains, `PKG_REF = mc_<seed>_pkg::K`, expr-derived `W_SIG`+`logic[W_SIG-1:0] sig`, `generate if/else`), `Manifest`+`build_manifest`/`emit_manifest` (the `.1` JSON schema, all facts from the `.2a` resolved oracle, `BTreeMap` ⇒ byte-stable `serde_json`). Default-off DUT-byte-identical is structural (separate module, never invoked by the DUT path). 3 new proofs (7 total): `emit_sv_is_valid_unresolved_shape`, `manifest_mirrors_the_oracle` (valid JSON; every fact == oracle), `sv_and_manifest_are_byte_reproducible`. `cargo fmt --all --check`/`clippy --all-targets -- -D warnings`/`check --all-targets` clean; full `cargo test` green (COMMIT.md gate). No parity harness (`.2c`); no ROADMAP/book change. | Done. Frontier → `.2c`. |
| `2026-05-20` | `PHASE-7-ORACLE-MICRODESIGN.2c` (split) | `.2c` made a container with children `.2c.1` (build the parity harness — cargo-portable comparator proof + tool-gated `#[ignore]` real-tool harness scaffold; no real run, no ROADMAP advance, cargo stays green tool-less) and `.2c.2` (real tool-equipped run + verify exact-agreement + ROADMAP Phase 7 → done; gate-blocked). Mirrors the proven memory `.2.3`/`.2.4` and FSM `.3.4a`/`.3.4b` decomposition (the harness machinery is code that lands first; the real-tool gate + promotion is a separate gated step; r87 no-aspirational-claims). Tree-planning, docs-only; no `src/`/`tests/` change (`cargo` unchanged-green vs `13faa77`). `mdbook build book` clean (no `book/` change). | Done. Frontier → `.2c.1`. |
| `2026-05-20` | `PHASE-7-ORACLE-MICRODESIGN.2c.2` (split) | `.2c.2` made a container with children `.2c.2a` (`FactCategory` + `ParityScope` + scoped comparator + yosys-specific `write_json` extractor + end-to-end-runnable `#[ignore]` harness; cargo stays green tool-less) and `.2c.2b` (real `--ignored` run + verify + bank + ROADMAP Phase 7 → done + book reconcile; gate-blocked). Discovered lower-level dependency: yosys 0.64 `write_json` exposes 4 of 7 manifest fact categories (seed/top/params/generate + partial widths["sig"]); localparams + package_constants are folded — needs the scoped comparator extension before any real run. Mirrors the proven memory `.2.1`→`.2.1a`/`.2.1b` precedent. Empirical probe across seeds `{0,1,7,42,12345}`: corpus exercises BOTH generate branches (seed 12345 takes `g_else`, others take `g_taken`). Tree-planning, docs-only; no `src/`/`tests/` change (`cargo` unchanged-green vs `c91d35e`). `mdbook build book` clean (no `book/` change). | Done. Frontier → `.2c.2a`. |
| `2026-05-20` | `PHASE-7-ORACLE-MICRODESIGN.2c.2b.2` | Real tool-equipped re-run of the `.2c.2a` `#[ignore]` parity gate against yosys 0.64 (with `.2c.2b.1`'s `width_expr` non-negative-modulo idiom in place): `cargo test --test microdesign_parity -- --ignored parity_against_real_yosys_write_json --nocapture` exited 0 with stdout `"parity gate clean across 5 seeds; artifacts in /Users/richarddje/.../target/tmp/microdesign-parity-phase7-yosys"` and `"test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 15 filtered out"`. Per-seed fact agreement verified BEFORE promotion (r87): seed 0 — manifest `{P0=46, bits=6, taken=true}` ≡ yosys `{P0=46, sig 6-bit, g_taken.gflag}`; seed 1 — `{P0=26, P2=27, bits=8, taken=true}` ≡ matching; seed 7 (the previously-divergent case) — `{P0=9, P3=3, P4=-1, bits=8, taken=true}` ≡ `{P4=-1 sign-extended from "11..1", sig 8-bit, g_taken}` — **the `.2c.2b.1` fix worked**; seed 42 — `{P0=44, bits=3, taken=true}` ≡ matching; seed 12345 — `{P0=43, P3=47, P4=186, bits=3, taken=false}` ≡ `{matching, netname g_else.gflag}` — the `g_else` branch is also exercised. **Both generate branches** (4 seeds `g_taken`, 1 seed `g_else`); **negative parameter sign-extension** correct; **previously-divergent seed 7 widths["sig"] now agrees**. Banked artifact copied to `/tmp/anvil-microdesign-parity-phase7-yosys-p1/` per established convention (15 files: 5 × {`.sv`, `.json`, `.yosys.json`}). Promotion strictly followed the artifact: ROADMAP Phase 7 `(not started)`→`(done)` + yosys-supported-categories scope caveat + richer-AST coverage as recorded post-closure follow-up; `book/src/ir.md` gains a Phase 7 micro-design lane delivered note; README + CODEBASE_ANALYSIS Phase-7 row done. No `src/`/`tests/` change in this slice (code in `.2a`/`.2b`/`.2c.1`/`.2c.2a`/`.2c.2b.1`); `mdbook build book` clean; `cargo` unchanged-green. **Closes `.2c.2b` + `.2c.2` + `.2c` + `.2` containers + `PHASE-7-ORACLE-MICRODESIGN` tree + ROADMAP Phase 7.** | Done — Phase 7 closed; tree closed. |
| `2026-05-20` | `PHASE-7-ORACLE-MICRODESIGN.2c.2b.1` | `src/microdesign/mod.rs::width_expr`: SV text changed from `"(({} % 8) + 1)"` to `"((({} % 8) + 8) % 8 + 1)"` — the standard SV non-negative-modulo idiom. The oracle (`last.value.rem_euclid(8) + 1`) was left **unchanged** because it was already correct; only the SV text needed to catch up. Comment block at the call site records the root cause + the fix rationale + the SV-vs-Rust modulo-semantic delta. New lib regression test `width_expr_uses_sv_non_negative_modulo_idiom_and_agrees_for_negative_last_values` (3 axes): (a) the `.2c.2a` counterexample fixture — seed 7's reproducible `P4 = -1` → `widths.sig.bits = 8`; (b) the non-negative collapse case — seed 0 `P4 = 365` → matches the old formula `365 % 8 + 1 = 6`; (c) cross-seed structural pin — every reproducibility-set seed's W_SIG line uses the new idiom textually. Existing `emit_sv_is_valid_unresolved_shape` substring pin updated from `"localparam int W_SIG = ((P"` to `"localparam int W_SIG = (((P"` to match the new emit. `cargo fmt --all --check`/`clippy --all-targets -- -D warnings`/`check --all-targets` clean. Full `cargo test` green: lib 229 (was 228 + 1 regression proof); `microdesign` 8/8 (was 7/7 + regression); `tests/microdesign_parity` 15 passed + 1 ignored (every `.2c.1` + `.2c.2a` portable proof still green — `manifest_mirrors_the_oracle` continues to hold because both sides of its equality moved in lockstep; `sv_and_manifest_are_byte_reproducible` re-baselines without code change because the formula is rebuilt deterministically); `tests/pipeline` 121 passed (658s); `tests/snapshots` 6 passed; bin tests 5+29+3 passed; doc-tests 0 (unchanged). Portable `cargo test` stays green tool-less. The `#[ignore]` real-tool gate is now unblocked. No ROADMAP advance (that is `.2c.2b.2` on a verified clean banked artifact, r87 no-aspirational-claims). No `book/` change. | Done. Frontier → `.2c.2b.2` (real-tool re-run + verify + bank + ROADMAP Phase 7 → done). |
| `2026-05-20` | `PHASE-7-ORACLE-MICRODESIGN.2c.2b` (split) | `.2c.2b` made a container with children `.2c.2b.1` (semantic-alignment fix: width_expr's oracle uses Rust's `rem_euclid` while the SV uses `%`; they diverge for negative `last.value` — change BOTH to the SV non-negative-modulo idiom `((x % 8) + 8) % 8 + 1`; regression proof on a negative-value seed; cargo gates green; no real run, no ROADMAP advance) and `.2c.2b.2` (real-tool re-run + verify clean + bank artifact + ROADMAP Phase 7 → done + book/README/CODEBASE reconcile; gate-blocked). Triggered by the inaugural `.2c.2a` `#[ignore]`-gate run on locally-installed yosys 0.64 retaining exactly one counterexample (seed 7, `WidthMismatch { sig, expected bits=8, actual bits=2 }`); root-caused to an ANVIL-self-consistency bug in `width_expr` (NOT a yosys bug). r87 no-aspirational-claims: the fix must precede the promoting commit's verified-clean banked artifact. Mirrors the proven memory `.2.1`→`.2.1a`/`.2.1b` discovered-dependency-split precedent (applied two levels up at `.2c` and `.2c.2`; repeated here one level deeper). Tree-planning, docs-only; no `src/`/`tests/` change (`cargo` unchanged-green vs `900061c`). `mdbook build book` clean (no `book/` change). | Done. Frontier → `.2c.2b.1`. |
| `2026-05-20` | `PHASE-7-ORACLE-MICRODESIGN.2c.2a` | `src/microdesign/mod.rs`: `pub enum FactCategory` (7 variants — one per fact axis); `pub struct ParityScope { categories: BTreeSet<FactCategory> }` with `all()`/`none()`/`only(&[...])` constructors + `.contains(category)`; `pub fn compare_manifest_to_tool_report_in_scope(manifest, report, scope)` — the scoped walker that skips out-of-scope axes entirely. The existing strict `compare_manifest_to_tool_report` now delegates with `ParityScope::all()` (backwards-compatible by construction; the 9 `.2c.1` proofs unchanged-green). `tests/microdesign_parity.rs`: 3 scoped-comparator proofs (`scoped_comparator_only_enforces_scoped_categories` — the load-bearing scoping proof: params-only scope ignores width perturbation BUT surfaces param perturbation; `yosys_scope_ignores_localparams_and_package_constants` — `yosys_write_json_scope` = `only(&[Seed,Top,Params,Widths,Generate])`; empty folded axes Ok under yosys scope, surface `PackageConstantMissingInTool` under strict-all; `empty_scope_ignores_every_disagreement` — `ParityScope::none()` Ok even on maximally-disagreeing report). Yosys extractor (helper-functions in the test file): `parse_yosys_binary_param(s)` — parses yosys's binary-string parameter values as SV `int` (signed 32-bit → `i128` via `u32 → i32` cast for sign-extension; defensive on empty/non-binary/`>32-bit`); `yosys_write_json_to_tool_report(json, seed)` — populates `params` from `.parameter_default_values`, `generate["g_taken"]` from netnames-key prefix scan, `widths["sig"]` from `.netnames.sig.bits` length; folded axes (`localparams`, `package_constants`) deliberately empty. 3 extractor proofs: `yosys_extractor_reads_a_synthetic_write_json_correctly` (hand-built JSON for seed 0 matches: P0=46, g_taken=true, widths.sig.bits=6; folded empty); `yosys_extractor_reports_g_else_when_else_branch_survives` (the g_else-survives case → `g_taken=false`); `parse_yosys_binary_param_sign_extends` (1...1 → -1; 0...01 → 1; 0..101110 → 46; empty/'z'/33-bit inputs → None — load-bearing because `.2a`'s builder can produce negative resolved values e.g. seed 7 P4 = -1). `parity_against_real_yosys_write_json` rewritten end-to-end: per-seed `emit_sv`→`CARGO_TARGET_TMPDIR/microdesign-parity-phase7-yosys/mc_<seed>.sv`, `emit_manifest`→`.json`, shell `yosys -q -p "read_verilog -sv ...; hierarchy -top mc_<seed>; proc; opt; write_json ..."`, parse → `ToolReport`, call scoped comparator with `yosys_write_json_scope`, accumulate counterexamples, panic with full diagnostic on any non-empty counterexample list (or `eprintln "parity gate clean across N seeds"`); yosys-presence guard at the head keeps the harness invocable on machines without the tool. NO real cargo-test run of the `#[ignore]` (that is `.2c.2b`'s deliverable). `cargo fmt --all --check`/`clippy --all-targets -- -D warnings`/`check --all-targets` clean. Full `cargo test` green: `tests/microdesign_parity` 15 passed + 1 ignored; `tests/pipeline` 121 passed (661s); `tests/snapshots` 6 passed; lib 228 passed (microdesign 7/7); doc-tests 0; bin tests 5+29+3 passed. Portable `cargo test` stays green tool-less. No ROADMAP advance (that is `.2c.2b` on a verified clean banked artifact, r87). No `book/` change. | Done. Frontier → `.2c.2b` (real `--ignored` run + ROADMAP Phase 7 → done). |
| `2026-05-20` | `PHASE-7-ORACLE-MICRODESIGN.2c.1` | `src/microdesign/mod.rs`: parity comparator core appended — `ToolReport` (normalized resolved-facts view from a downstream consumer; `BTreeMap` throughout for determinism; serde Serialize+Deserialize for JSON round-trip diagnostics); `Divergence` enum (17 variants — `SeedMismatch`/`TopMismatch` + {missing-in-tool, missing-in-manifest, mismatch} × {param, localparam, width, generate, package-constant} so `.1`'s rejected-alternative "single facts-disagree bit" gap is closed); `compare_manifest_to_tool_report` (cargo-portable walker; accumulates the full divergence set rather than fail-fast; symbolic `expr` strings deliberately not compared); `synthetic_tool_report_from_manifest` (always-agreeing reference, used by the proofs and as the fallback by `.2c.2`'s real-tool path). `FactEntry`/`WidthFact`/`GenFact`/`ConstExprFact`/`Manifest` fields promoted to `pub`; derives extended to `Clone`+`PartialEq`+`Eq`+`Deserialize`. New `tests/microdesign_parity.rs`: 9 cargo-portable comparator proofs (`comparator_agrees_on_synthetic_tool_report_built_from_the_oracle` baseline + per-axis divergence proofs covering param/localparam/width/generate-branch/package-constant/param-missing-in-tool/param-missing-in-manifest/seed+top — every axis surfaces the right `Divergence` variant) + 1 tool-gated `#[ignore]` `parity_against_real_yosys_write_json` scaffold (yosys-presence guard at the head; corpus-driver loop wired against the same `SEEDS={0,1,7,42,12345}`/`N_PARAMS=5` constants the portable proofs use, with placeholder for the `.2c.2`-owned `emit_sv`→shell→extract→compare end-to-end wiring). `cargo fmt --all --check`/`clippy --all-targets -- -D warnings`/`check --all-targets` clean. Full `cargo test` green: tests/microdesign_parity 9 passed + 1 ignored; tests/pipeline 121 passed (657s); tests/snapshots 6 passed; doc-tests unchanged; lib `microdesign` tests still 7/7 green (`.2a`+`.2b` unchanged). Portable `cargo test` stays green tool-less; the tool-gated harness is invocable only via `cargo test -- --ignored` AND when `yosys` is on `$PATH`. No ROADMAP advance (that is `.2c.2` on a verified clean banked artifact). No `book/` change. | Done. Frontier → `.2c.2` (real-tool run + ROADMAP Phase 7 → done). |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `PHASE-7-ORACLE-MICRODESIGN.1` | `Docs: PHASE-7-ORACLE-MICRODESIGN.1 oracle-backed micro-design artifact-family design` | Design-only; expected-facts JSON schema + oracle-by-construction strategy + new parity harness + 4 rejected alternatives. No code. |
| `PHASE-7-ORACLE-MICRODESIGN.2` (split) | `Docs: split PHASE-7-ORACLE-MICRODESIGN.2 into .2a (IR+evaluator) / .2b (emitters) / .2c (parity gate)` | Tree-planning, no code. Boundaries per `.1`'s named split candidates. |
| `PHASE-7-ORACLE-MICRODESIGN.2a` | `Phase 7: PHASE-7-ORACLE-MICRODESIGN.2a const-expr/parameter IR + construction-time evaluator (oracle)` | New `src/microdesign/` IR + evaluator/oracle + reproducible rules-first builder; 4 unit proofs; no emit/harness. |
| `PHASE-7-ORACLE-MICRODESIGN.2b` | `Phase 7: PHASE-7-ORACLE-MICRODESIGN.2b SV + JSON expected-facts manifest emitters (from the .2a oracle)` | Un-resolved SV emitter + JSON manifest emitter, both from the `.2a` oracle; 3 new proofs (7 total); byte-reproducible; no harness. |
| `PHASE-7-ORACLE-MICRODESIGN.2c` (split) | `Docs: split PHASE-7-ORACLE-MICRODESIGN.2c into .2c.1 (build harness) + .2c.2 (real-tool gate + ROADMAP Phase 7)` | Tree-planning, no code. Mirrors memory `.2.3`/`.2.4` and FSM `.3.4a`/`.3.4b` decomposition. |
| `PHASE-7-ORACLE-MICRODESIGN.2c.1` | `Phase 7: PHASE-7-ORACLE-MICRODESIGN.2c.1 parity harness — comparator core + cargo-portable proofs + tool-gated #[ignore] scaffold` | Parity comparator core in `src/microdesign/` (`ToolReport`/`Divergence` × 17 variants/`compare_manifest_to_tool_report`/`synthetic_tool_report_from_manifest`) + `pub`/`Clone`/`PartialEq`/`Eq`/`Deserialize` promotions on the fact records + new `tests/microdesign_parity.rs` (9 cargo-portable proofs + 1 tool-gated `#[ignore]` scaffold); portable `cargo test` stays green tool-less. No ROADMAP advance (that is `.2c.2`). |
| `PHASE-7-ORACLE-MICRODESIGN.2c.2` (split) | `Docs: split PHASE-7-ORACLE-MICRODESIGN.2c.2 into .2c.2a (extractor + scoped comparator) + .2c.2b (real-tool gate + ROADMAP Phase 7)` | Tree-planning, no code. Discovered lower-level dependency: yosys `write_json` exposes 4 of 7 manifest fact categories — needs `FactCategory`+`ParityScope` extension before any real run. Mirrors memory `.2.1`→`.2.1a`/`.2.1b`. |
| `PHASE-7-ORACLE-MICRODESIGN.2c.2a` | `Phase 7: PHASE-7-ORACLE-MICRODESIGN.2c.2a scoped comparator + yosys write_json extractor + end-to-end-runnable #[ignore] harness` | `FactCategory`+`ParityScope`+scoped comparator in `src/microdesign/` (strict comparator delegates to `ParityScope::all()`); yosys-specific extractor + 3 sanity proofs (synthetic JSON / g_else survives / sign-extension) + 3 scoped-comparator proofs (scoping itself / yosys-scope vs strict-all / empty-scope) in `tests/microdesign_parity.rs`; `#[ignore]` test now end-to-end-runnable (no real run from cargo); portable test stays green tool-less. No ROADMAP advance (that is `.2c.2b`). |
| `PHASE-7-ORACLE-MICRODESIGN.2c.2b` (split) | `Docs: split PHASE-7-ORACLE-MICRODESIGN.2c.2b on a real-tool-surfaced ANVIL-self-consistency bug — split into .2c.2b.1 (semantic-alignment fix) + .2c.2b.2 (re-run + ROADMAP Phase 7)` | Tree-planning, no code. First real-tool run of the `.2c.2a` `#[ignore]` gate retained one counterexample (seed 7, `WidthMismatch sig 8→2`) — root cause: `width_expr` oracle uses Rust's `rem_euclid` but SV uses `%`; diverges for negative `last.value`. Per ANVIL's "valid-by-construction" north-star + r87, fix must land before ROADMAP Phase 7 promotion. |
| `PHASE-7-ORACLE-MICRODESIGN.2c.2b.1` | `Phase 7: PHASE-7-ORACLE-MICRODESIGN.2c.2b.1 width_expr oracle/SV semantic alignment — non-negative-modulo idiom + regression proof` | `width_expr` SV text flipped to the standard SV non-negative-modulo idiom `((x % 8) + 8) % 8 + 1` so oracle ≡ SV for any `last.value`. New lib regression proof + cross-seed structural pin. The seed-7 `WidthMismatch` counterexample is now structurally impossible. Portable `cargo test` stays green tool-less. No ROADMAP advance (that is `.2c.2b.2`). |
| `PHASE-7-ORACLE-MICRODESIGN.2c.2b.2` | `Phase 7: PHASE-7-ORACLE-MICRODESIGN.2c.2b.2 parity gate clean against yosys — closes Phase 7 + tree` | Real-tool `--ignored` parity gate against yosys 0.64 came back clean across all 5 seeds; verified-clean artifact banked at `/tmp/anvil-microdesign-parity-phase7-yosys-p1/`; per-seed fact agreement verified incl. seed 7 (P4=-1) and seed 12345 (g_else branch). **Closes the `.2c.2b` container + `.2c.2` container + `.2c` container + `.2` container + `PHASE-7-ORACLE-MICRODESIGN` tree; closes ROADMAP Phase 7** with the explicit yosys-supported-categories scope caveat. No code change in this slice (docs/ROADMAP/book only). |

## Changelog

- `2026-05-16`: Created task tree as part of the directive to task-tree
  every remaining roadmap phase.
- `2026-05-18`: **`.1` design landed** (design-only, no code) —
  continuous-PNT while both remaining Phase 6 leaves (`.2.4`/`.3.4b`)
  are gate-blocked. `DEVELOPMENT_NOTES.md` "Phase 7 oracle-backed
  micro-design artifact family design": the oracle-by-construction
  shift (the generator evaluates every const-expr/param node as it
  builds it and emits the `.sv` + JSON manifest from the same
  resolved values — no analysis pass, no re-parse), its own
  source-level const/parameter IR (separate generator path; the
  circuit IR has no param/generate/package concept), the
  expected-facts JSON schema, the reproducibility contract, a new
  parity harness (distinct from the `tool_matrix` DUT gate;
  cargo-portable structural-equivalence + repo-owned gate), the
  Phase-8/9 boundaries, 4 rejected alternatives, and the `.2` proof
  shape + split candidates. Both Open Questions resolved (typed JSON
  manifest; new separate parity harness). `mdbook` clean. Frontier →
  `.2` (implement; expected to split IR+evaluator / emitters /
  harness+gate).
- `2026-05-18`: **`.2` split** (tree-planning, docs-only, no code)
  — continuous-PNT while Phase 6 `.2.4`/`.3.4b` are gate-blocked
  and all design/research/triage leaves are exhausted; formalising
  the split is the remaining ~zero-contention advance (the heavy
  `.2a`/`.2b`/`.2c` implementation waits for the near-complete
  priority gate to free the machine — same contention-aware
  sequencing applied all session). `.2` → container `.2a`
  (const-expr/parameter IR + construction-time evaluator/oracle;
  unit-proven, no emit/harness), `.2b` (SV emitter + JSON manifest
  emitter from the same evaluated IR, behind an artifact-family
  flag, default-off DUT-byte-identical), `.2c` (parity harness +
  repo-owned gate, tool-gated so `cargo test` stays tool-less →
  ROADMAP Phase 7 only after a verified clean run, r87). Exactly
  the independently-reviewable boundaries `.1` named; `.2a`+`.2b`'s
  evaluator/manifest core is the reuse `PHASE-8-FRONTEND-ACCEPT.2`
  and the Phase-9 plumbing depend on. `cargo` unchanged-green vs
  `e550db1`. Frontier → `.2a`.
- `2026-05-19`: **`.2a` landed** — the foundational Phase 7 IR +
  oracle. New separate top-level module `src/microdesign/mod.rs`
  (`pub mod microdesign` in `src/lib.rs`; deliberately *not* in
  `src/ir/` — the circuit IR has no parameter/localparam/expression
  concept, the category error `.1` rejected). `ConstExpr` AST
  (`Lit`/`Param`/`Unary`/`Bin`/`Ternary`), `ParamDecl` with the
  construction-time-resolved `value` (the oracle),
  `ConstExprUnit` (an ordered forward-ref-free parameter/localparam
  dependency DAG). `eval()` implements the bounded SV
  constant-expression integer semantics (truncating div/mod,
  clamped shift, comparisons/logicals → 1/0; defensive
  `EvalError`). `resolve()` = the **oracle**: fills every
  `ParamDecl.value` in declaration order, run once at construction
  time (`.2b`'s SV + manifest will read these, never re-derive).
  `build_constexpr_unit(seed, n)` = a rules-first reproducible
  builder (`ChaCha8::seed_from_u64`, project convention, no
  `thread_rng`): literal root + earlier-decl chains / precedence /
  ternary, resolved in place (the builder *is* the oracle — no
  analysis pass, no re-parse). 4 unit proofs green incl. the
  load-bearing `stored_values_are_consistent_with_a_fresh_reeval`
  invariant (the stored oracle value never drifts from its expr)
  and `build_is_reproducible_and_seed_sensitive`. `cargo fmt
  --all --check` / `clippy --all-targets -- -D warnings` /
  `check --all-targets` clean; full `cargo test` green incl. the
  new module (COMMIT.md gate). No SV/manifest emit, no harness
  (`.2b`/`.2c`); no ROADMAP/book change. Frontier → `.2b`
  (SV + JSON-manifest emitters from this evaluated IR).
- `2026-05-19`: **`.2b` landed** — `src/microdesign/` extended with
  the un-resolved SV emitter + the JSON expected-facts manifest
  emitter, **both from the `.2a` resolved oracle**. `expr_to_sv` is
  a fully-parenthesized printer (precedence-unambiguous; the `.2a`
  builder's nested `a+b*c`/ternary shapes carry the
  precedence-sensitive-expression axis). `emit_sv` produces the
  `rtl_const_expr` family as deliberately *un-resolved*
  SystemVerilog (`package mc_<seed>_pkg`/`K`; a module with
  *symbolic* `parameter`/`localparam` chains, `PKG_REF =
  mc_<seed>_pkg::K`, an expr-derived `W_SIG` + `logic[W_SIG-1:0]
  sig`, and a `generate if/else` over a param predicate) — the gap
  between symbolic text and the manifest's resolved facts is
  exactly the front-end behaviour Phase 7 stresses. `Manifest` +
  `build_manifest`/`emit_manifest` serialize the `.1` schema
  (seed/top/params/localparams/widths/generate/package_constants/
  const_exprs) entirely from the oracle; `BTreeMap` ordering ⇒
  byte-stable `serde_json` pretty output. Default-off
  DUT-byte-identical is *structural* (microdesign is a separate
  module never invoked by the DUT generate path; the Phase-9
  selector wires invocation later). 3 new unit proofs (7 total):
  `emit_sv_is_valid_unresolved_shape`, `manifest_mirrors_the_oracle`
  (valid JSON; every fact equals the oracle),
  `sv_and_manifest_are_byte_reproducible`. `cargo fmt --all
  --check` / `clippy --all-targets -- -D warnings` /
  `check --all-targets` clean; full `cargo test` green (COMMIT.md
  gate). No parity harness (`.2c`); no ROADMAP/book change.
  Frontier → `.2c` (parity harness + repo-owned gate → ROADMAP
  Phase 7, r87).
- `2026-05-20`: **`.2c` split** into `.2c.1` (build the parity
  harness — cargo-portable comparator proof + tool-gated `#[ignore]`
  real-tool harness scaffold; no real run, no ROADMAP advance,
  cargo stays green tool-less per Phase-1 doctrine) and `.2c.2`
  (real tool-equipped run + verify exact-agreement + record
  **ROADMAP Phase 7 → done**; gate-blocked, r87
  no-aspirational-claims). Splitting Rules + the proven memory
  `.2.3`/`.2.4` and FSM `.3.4a`/`.3.4b` decomposition — the harness
  machinery is code that lands first as one signoff-sized leaf;
  the gated real run + ROADMAP promotion + book reconcile is a
  separate gated step. `.2c` is now a container; no renumbering.
  Continuous-PNT immediately after closing Phase 6 + the
  `PHASE-6-ADVANCED-MOTIFS` tree at `13faa77` (the 30-commit batch
  pushed `8076e25..13faa77`). Tree-planning, docs-only; no
  `src/`/`tests/` change (`cargo` unchanged-green vs `13faa77`);
  `mdbook build book` clean (no `book/` change). Frontier →
  `.2c.1` (the parity-harness-build leaf; unblocked).
- `2026-05-20`: **`.2c.1` landed — parity harness scaffold.**
  `src/microdesign/mod.rs` extended with the parity comparator
  core: `ToolReport` (normalized resolved-facts view from a
  downstream consumer; `BTreeMap` throughout for determinism;
  serde `Serialize`+`Deserialize` for JSON round-trip
  diagnostics), `Divergence` enum (17 variants —
  `SeedMismatch`/`TopMismatch` + {missing-in-tool,
  missing-in-manifest, mismatch} × {param, localparam, width,
  generate, package-constant}; `.1`'s rejected-alternative
  "single facts-disagree bit" gap is closed),
  `compare_manifest_to_tool_report` (cargo-portable walker;
  accumulates the full divergence set rather than fail-fast — so
  `.2c.2`'s gate either reports `Ok(())` or retains the full
  counterexample profile in one pass; symbolic `expr` strings
  deliberately NOT compared as they are un-resolved-SV
  documentation, not facts the tool re-emits),
  `synthetic_tool_report_from_manifest` (always-agreeing reference
  used by the proofs and as the fallback by `.2c.2`'s real-tool
  path). `FactEntry`/`WidthFact`/`GenFact`/`ConstExprFact`/`Manifest`
  fields promoted to `pub`; derives extended to
  `Clone`+`PartialEq`+`Eq`+`Deserialize`. New
  `tests/microdesign_parity.rs` (mirrors `tests/pipeline.rs` /
  `tests/snapshots.rs` as a top-level integration test) carries 9
  cargo-portable comparator proofs (all green) covering the
  baseline agreement + each divergence axis: param /
  localparam / width / generate-branch / package-constant /
  param-missing-in-tool / param-missing-in-manifest / seed+top.
  Plus 1 tool-gated `#[ignore]` `parity_against_real_yosys_write_json`
  scaffold with a yosys-presence guard (matches the
  `iverilog`-not-installed convention from
  `DIFFERENTIAL-SIMULATION.1`) and the corpus-driver loop wired
  against the same `SEEDS={0,1,7,42,12345}`/`N_PARAMS=5` constants
  the portable proofs use, with a placeholder for the
  `.2c.2`-owned `emit_sv`→shell→extract→compare end-to-end
  wiring. `cargo fmt --all --check` / `clippy --all-targets -- -D
  warnings` / `check --all-targets` clean. Full `cargo test`
  green: `tests/microdesign_parity` 9 passed + 1 ignored;
  `tests/pipeline` 121 passed (657s); `tests/snapshots` 6 passed;
  doc-tests 0 (unchanged); lib `microdesign` tests still 7/7 green
  (`.2a`+`.2b` unchanged). Portable `cargo test` stays green
  tool-less. No ROADMAP advance (that is `.2c.2` on a verified
  clean banked artifact, r87). No `book/` change. Frontier →
  `.2c.2` (run the real `cargo test -- --ignored
  parity_against_real_yosys_write_json`; verify exact-agreement
  across the corpus; record ROADMAP Phase 7 → done).
- `2026-05-20`: **`.2c.2` split** on a discovered lower-level
  dependency. An empirical probe of yosys 0.64's `write_json`
  output for the `.2b` `mc_<seed>` corpus (seeds
  `{0,1,7,42,12345}`, `N_PARAMS=5`) confirmed yosys exposes
  `.parameter_default_values` (binary-string → SV `int` → `i128`),
  the elaborated-generate-branch via `.netnames` keys prefixed by
  `g_taken.`/`g_else.` (the corpus exercises **both** — seed 12345
  takes `g_else`, the others take `g_taken`), and the top wire
  `sig` width via `.netnames["sig"].bits` — i.e. **4 of the 7
  manifest fact categories**. Localparams and package-constants
  (`mc_<seed>_pkg::K`) are **folded by yosys** and not
  name-introspectable from `write_json` alone — they require
  richer-AST tools (`slang --ast-json`, `verilator --xml-only`).
  The yosys parity scope is therefore `Seed`/`Top`/`Params`/
  `Generate`/`Widths["sig"]`. This is a discovered lower-level
  dependency mirroring the memory `.2.1` split (compaction-
  reachability for an opaque stateful leaf was load-bearing
  pipeline code that justified `.2.1`→`.2.1a`/`.2.1b`): the
  `FactCategory`+`ParityScope` extension to the comparator must
  land BEFORE any real-tool run can be honest about what was
  checked. Per Splitting Rules, `.2c.2` was split into
  `.2c.2a` (extractor + scoped comparator + end-to-end-runnable
  `#[ignore]` harness + cargo-portable scoped-comparator proof;
  cargo stays green tool-less; no real run, no ROADMAP advance)
  and `.2c.2b` (run the `#[ignore]` gate against real yosys +
  verify exact-agreement on yosys-supported categories + bank
  the artifact + record **ROADMAP Phase 7 → done** with the
  explicit scope caveat; book/README/CODEBASE reconcile;
  gate-blocked, r87). `.2c.2` is now a container; no renumbering.
  Richer fact-category coverage via `slang`/`verilator-with-debug`
  is a recorded follow-up that does not block `.2c.2b` — ANVIL's
  by-construction oracle already covers all 7 categories; the
  gate exercises whatever the tool reports. Tree-planning, docs-
  only; no `src/`/`tests/` change (`cargo` unchanged-green vs
  `c91d35e`); `mdbook build book` clean. Frontier → `.2c.2a`.
- `2026-05-20`: **`.2c.2a` landed — scoped comparator + yosys
  extractor + end-to-end-runnable `#[ignore]` harness.**
  `src/microdesign/mod.rs` extended with `pub enum FactCategory`
  (`Seed`/`Top`/`Params`/`Localparams`/`Widths`/`Generate`/
  `PackageConstants` — one per fact axis), `pub struct ParityScope
  { categories: BTreeSet<FactCategory> }` with `all()`/`none()`/
  `only(&[...])` constructors + `.contains(category)`, and
  `pub fn compare_manifest_to_tool_report_in_scope(manifest,
  report, scope)` — the scoped walker. The existing strict
  `compare_manifest_to_tool_report` now delegates to this with
  `ParityScope::all()` so every previously-passing call still
  passes (the 9 `.2c.1` proofs unchanged-green by construction).
  `tests/microdesign_parity.rs` extended with 6 new
  cargo-portable proofs: 3 scoped-comparator
  (`scoped_comparator_only_enforces_scoped_categories` — the
  load-bearing scoping proof: a params-only scope ignores width
  perturbation BUT surfaces param perturbation;
  `yosys_scope_ignores_localparams_and_package_constants` —
  `yosys_write_json_scope` = `only(&[Seed, Top, Params, Widths,
  Generate])`; empty folded axes compare `Ok(())` under the
  yosys scope, surface `PackageConstantMissingInTool` under
  `ParityScope::all()`; `empty_scope_ignores_every_disagreement`
  — `ParityScope::none()` `Ok(())` even on a maximally-
  disagreeing report — self-check on the scoping itself) + 3
  yosys-extractor (`yosys_extractor_reads_a_synthetic_write_json_correctly`
  — hand-built JSON for seed 0 matches: `P0=46`, `g_taken=true`,
  `widths.sig.bits=6`; folded axes empty;
  `yosys_extractor_reports_g_else_when_else_branch_survives` —
  the `g_else`-survives case → `g_taken=false`;
  `parse_yosys_binary_param_sign_extends` — `1...1` → `-1`;
  `0...01` → `1`; `0..101110` → `46`; empty/`z`/`33-bit` inputs
  → `None`; load-bearing because `.2a`'s builder can produce
  negative resolved values e.g. seed 7 `P4 = -1`). The
  yosys-specific `yosys_write_json_to_tool_report(json, seed)`
  extractor populates `params` from
  `.parameter_default_values`, `generate["g_taken"]` from a
  netnames-key prefix scan (`g_taken.`/`g_else.`), and
  `widths["sig"]` from `.netnames.sig.bits` length; the folded
  axes (`localparams`, `package_constants`) are deliberately
  empty. `parity_against_real_yosys_write_json` rewritten
  end-to-end: per-seed `emit_sv`→`CARGO_TARGET_TMPDIR/
  microdesign-parity-phase7-yosys/mc_<seed>.sv`,
  `emit_manifest`→`.json`, shell `yosys -q -p "read_verilog -sv
  ...; hierarchy -top mc_<seed>; proc; opt; write_json ..."`,
  parse → `ToolReport`, call scoped comparator with
  `yosys_write_json_scope`, accumulate counterexamples, panic
  with full diagnostic on any non-empty counterexample list (or
  `eprintln "parity gate clean across N seeds"`); the
  yosys-presence guard at the head keeps the harness invocable
  on machines without the tool. **NO real cargo-test run of the
  `#[ignore]` test** (that is `.2c.2b`'s deliverable). `cargo
  fmt --all --check`/`clippy --all-targets -- -D warnings`/
  `check --all-targets` clean. Full `cargo test` green:
  `tests/microdesign_parity` 15 passed + 1 ignored;
  `tests/pipeline` 121 passed (661s); `tests/snapshots` 6
  passed; lib 228 passed (microdesign 7/7 + the rest unchanged);
  doc-tests 0 (unchanged); bin tests 5+29+3 passed. Portable
  `cargo test` stays green tool-less. No ROADMAP advance (that
  is `.2c.2b` on a verified clean banked artifact, r87
  no-aspirational-claims). No `book/` change. Frontier →
  `.2c.2b` (run `cargo test -- --ignored
  parity_against_real_yosys_write_json`; verify clean banked
  artifact; record ROADMAP Phase 7 → done with the explicit
  scope caveat; reconcile book/README/CODEBASE).
- `2026-05-20`: **`.2c.2b` split** — the very first real-tool run
  of the `.2c.2a` `#[ignore]` parity gate (`cargo test --
  --ignored parity_against_real_yosys_write_json` against
  locally-installed yosys 0.64, immediately after `.2c.2a`
  landed at `900061c`) retained exactly one counterexample:
  **seed 7, `WidthMismatch { name: "sig", expected: bits=8,
  actual: bits=2 }`**. Root cause is an ANVIL-self-consistency
  bug in `width_expr` (`src/microdesign/mod.rs`): the SV text
  is `((<last> % 8) + 1)` but the oracle is
  `last.value.rem_euclid(8) + 1`. Rust's `rem_euclid` is the
  *mathematical non-negative modulo* (`(-1).rem_euclid(8) = 7`);
  SV's `%` on signed integers is *truncated toward zero* (`-1 %
  8 = -1`, identical to Rust's `%`). For seed 7's
  `P4 = -1`: the oracle reports `bits = 8` while the SV
  evaluates `(-1 % 8) + 1 = 0` and yosys interprets
  `logic [-1:0] sig` as 2 bits. Oracle ≠ SV ⇒ NOT a yosys bug
  but an ANVIL self-consistency defect surfaced by the parity
  gate. ANVIL's "valid-by-construction +
  downstream-acceptance-quality" north-star (user-confirmed
  2026-05-18) requires fixing this BEFORE ROADMAP Phase 7 can
  be promoted (r87 no-aspirational-claims: a counterexample is
  not a clean banked artifact). Per the proven memory `.2.1`
  →`.2.1a`/`.2.1b` discovered-dependency-split precedent (the
  same precedent applied at the `.2c` and `.2c.2` splits;
  repeated here one level deeper), `.2c.2b` was split into
  `.2c.2b.1` (semantic-alignment fix: change BOTH the SV text
  and the oracle to the standard SV non-negative-modulo idiom
  `((x % 8) + 8) % 8 + 1` so the width is always in `[1, 8]`
  and oracle ≡ SV; add a regression proof exercising a
  negative-value seed; cargo gates green; no real run, no
  ROADMAP advance) and `.2c.2b.2` (re-run the `#[ignore]`
  gate against real yosys + verify clean + bank + record
  **ROADMAP Phase 7 → done** with the explicit
  yosys-supported-categories scope caveat;
  book/README/CODEBASE reconcile; gate-blocked, r87).
  `.2c.2b` is now a container; no renumbering. The parity
  gate is doing exactly what `.1` designed it to do —
  surface semantic disagreement between the oracle and the
  downstream — and the next slice closes the loop. Tree-
  planning, docs-only; no `src/`/`tests/` change (`cargo`
  unchanged-green vs `900061c`); `mdbook build book` clean.
  Frontier → `.2c.2b.1`.
- `2026-05-20`: **`.2c.2b.1` landed — width_expr oracle/SV
  semantic-alignment fix.** `src/microdesign/mod.rs::width_expr`
  SV text changed from `"(({} % 8) + 1)"` to
  `"((({} % 8) + 8) % 8 + 1)"` — the standard SV
  non-negative-modulo idiom. A SV elaborator evaluates the new
  text to the same value Rust's `rem_euclid` already produced
  for the oracle (`((-1 % 8) + 8) % 8 + 1 = 8` in both
  languages), so the oracle (left unchanged because it was
  already correct) now agrees with the SV literal evaluation
  for every `last.value` — positive *or* negative. A comment
  block at the call site records the root cause + the fix
  rationale + the SV-vs-Rust modulo-semantic delta. New lib
  regression test `width_expr_uses_sv_non_negative_modulo_idiom_and_agrees_for_negative_last_values`
  (3 axes): (a) the `.2c.2a` counterexample fixture — seed 7's
  reproducible `P4 = -1` → `widths.sig.bits = 8` (the bug
  case; before the fix the SV would evaluate `W_SIG = 0` and
  yosys would interpret `logic [-1:0] sig` as 2 bits); (b) a
  non-negative-collapse case — seed 0 `P4 = 365` → matches
  the old formula `365 % 8 + 1 = 6` (the new idiom is
  identity-on-the-non-negative-dividend domain); (c) a
  cross-seed structural pin — every reproducibility-set seed's
  W_SIG line uses the new idiom textually. The existing
  `emit_sv_is_valid_unresolved_shape` substring pin updated
  from `"localparam int W_SIG = ((P"` to
  `"localparam int W_SIG = (((P"` to match the new emit. `cargo
  fmt --all --check`/`clippy --all-targets -- -D warnings`/
  `check --all-targets` clean. Full `cargo test` green: lib 229
  (was 228 + 1 regression); `microdesign` 8/8 (was 7/7 +
  regression); `tests/microdesign_parity` 15 passed + 1 ignored
  (every `.2c.1` + `.2c.2a` portable proof still green —
  `manifest_mirrors_the_oracle` continues to hold because both
  sides of its equality moved in lockstep);
  `sv_and_manifest_are_byte_reproducible` re-baselines without
  code change because the formula is rebuilt deterministically;
  `tests/pipeline` 121 passed (658s); `tests/snapshots` 6 passed;
  bin tests 5+29+3 passed; doc-tests 0 (unchanged). Portable
  `cargo test` stays green tool-less. **The `#[ignore]` real-
  tool gate is now unblocked**: the seed-7 `WidthMismatch`
  counterexample is structurally impossible to surface after
  this fix. No ROADMAP advance (that is `.2c.2b.2` on a
  verified clean banked artifact, r87 no-aspirational-claims).
  No `book/` change. Frontier → `.2c.2b.2` (run the
  `--ignored` gate; bank the verified-clean artifact; record
  ROADMAP Phase 7 → done with the explicit yosys-supported-
  categories scope caveat; reconcile book/README/CODEBASE).
- `2026-05-20`: **`.2c.2b.2` landed — `PHASE-7-ORACLE-MICRODESIGN`
  tree CLOSED; ROADMAP Phase 7 → done.** Real tool-equipped
  re-run of the `.2c.2a` `#[ignore]` parity gate against yosys
  0.64 (with `.2c.2b.1`'s width_expr non-negative-modulo idiom
  in place): `cargo test --test microdesign_parity -- --ignored
  parity_against_real_yosys_write_json --nocapture` exited 0
  with stdout `"parity gate clean across 5 seeds; artifacts in
  /Users/.../target/tmp/microdesign-parity-phase7-yosys"` and
  `"test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured;
  15 filtered out"`. Per-seed fact agreement verified BEFORE
  any promotion (r87 no-aspirational-claims): seed 0 — manifest
  `{P0=46, bits=6, taken=true}` ≡ yosys
  `{P0='00..101110'=46, sig 6-bit, g_taken.gflag}`; seed 1 —
  `{P0=26, P2=27, bits=8, taken=true}` ≡ matching; **seed 7
  (the previously-divergent case)** — `{P0=9, P3=3, P4=-1,
  bits=8, taken=true}` ≡ `{P4='11..1' sign-extended to -1, sig
  8-bit, g_taken}` — the `.2c.2b.1` fix worked perfectly;
  seed 42 — `{P0=44, bits=3, taken=true}` ≡ matching; seed
  12345 — `{P0=43, P3=47, P4=186, bits=3, taken=false}` ≡
  `{matching, netname g_else.gflag}`. **Both generate
  branches** are exercised (4 seeds `g_taken`, 1 seed
  `g_else`); **negative parameter values** are correctly
  sign-extended through the yosys binary-string format;
  **the previously-divergent seed 7 `widths["sig"]` now
  agrees** (oracle and yosys both report `bits = 8`). Banked
  artifact copied to `/tmp/anvil-microdesign-parity-phase7-
  yosys-p1/` per established convention (15 files: 5 ×
  {`mc_<seed>.sv`, `mc_<seed>.json`, `mc_<seed>.yosys.json`}).
  Promotion strictly followed the artifact: ROADMAP Phase 7
  `(not started)`→`(done)` + the verified-clean artifact
  citation + the explicit **yosys-supported-categories scope
  caveat** (Seed/Top/Params/Widths/Generate; localparams +
  package-constants remain visible only to richer-AST tools —
  slang/verilator-with-debug — recorded as a post-Phase-7
  follow-up that does NOT block closure since ANVIL's
  by-construction oracle already covers all 7 categories);
  `book/src/ir.md` gains a Phase 7 micro-design lane delivered
  note citing the artifact; README phase narrative Phase 7 →
  done; CODEBASE_ANALYSIS phase-coverage-map Phase-7 row →
  done. Multi-clock CDC stays the explicitly-optional,
  separately-prioritised deferral (per the Phase 6 closure
  note carried forward). No `src/`/`tests/` change in this
  slice (the code landed in `.2a`/`.2b`/`.2c.1`/`.2c.2a`/
  `.2c.2b.1`); `mdbook build book` clean; `cargo`
  unchanged-green. **Closes the `.2c.2b` container; closes the
  `.2c.2` container; closes the `.2c` container; closes the
  `.2` container; closes the `PHASE-7-ORACLE-MICRODESIGN`
  tree; closes ROADMAP Phase 7.** Frontier → (closed). The
  parity gate doing exactly what `.1` designed it to do —
  surfacing an ANVIL-self-consistency bug on the first run +
  validating the fix on the second — is the strongest possible
  affirmation of Phase 7's by-construction oracle approach.
