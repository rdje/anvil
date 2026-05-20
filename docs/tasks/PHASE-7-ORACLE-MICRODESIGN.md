# PHASE-7-ORACLE-MICRODESIGN: Oracle-backed micro-design artifacts

## Metadata

- Tree ID: `PHASE-7-ORACLE-MICRODESIGN`
- Status: `active`
- Roadmap lane: Phase 7 — Oracle-backed micro-design artifacts
- Created: `2026-05-16`
- Last updated: `2026-05-20` (**`.2c` split** into `.2c.1` parity-harness build + `.2c.2` real-gate verify → ROADMAP Phase 7, mirroring the proven memory `.2.3`/`.2.4` and FSM `.3.4a`/`.3.4b` decomposition; tree-planning only, no code; frontier → `.2c.1`)
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
  Status: `active`
  Goal: `Reproducible oracle-backed micro-design corpus with expected-facts contract and downstream parity checks.`
  Children: `PHASE-7-ORACLE-MICRODESIGN.1`, `PHASE-7-ORACLE-MICRODESIGN.2`

- ID: `PHASE-7-ORACLE-MICRODESIGN.1`
  Status: `done`
  Goal: `Design the micro-design artifact family in DEVELOPMENT_NOTES.md / book: expected-facts schema, generation strategy (param/expr chains), reproducibility contract, parity-check harness shape, relationship to the existing DUT lane, rejected alternatives. Design-only.`
  Acceptance: `Design entry with expected-facts schema sketch and >=1 rejected alternative; mdbook clean; no code change.`
  Verification: `DEVELOPMENT_NOTES.md "Phase 7 oracle-backed micro-design artifact family design (2026-05-18, PHASE-7-ORACLE-MICRODESIGN.1)" entry landed. Records: the conceptual shift (Phases 1-6 = random RTL, no semantic oracle; Phase 7 = tiny .sv whose elaboration facts are known by construction + a machine-checkable manifest — pressure point is front-end constant-expr/param/elaboration correctness). Codebase grounding (the scalar-u32 gate-level circuit IR has no parameter/localparam/generate/package/typed-constant concept; WidthExpr/ParamEnv is width-only; Phase 7 needs its own small source-level constant/parameter IR, a separate generator path, reusing seeding/CLI/reproducibility). rtl_const_expr artifact family per ROADMAP (param/localparam dependency chains; expr-derived widths/ranges; generate if/for; package-qualified constants; precedence-sensitive expressions). Expected-facts JSON manifest schema sketch (params/localparams/widths/generate/package_constants/const_exprs). Oracle-by-construction generation strategy (the generator evaluates every const-expr/param node as it builds it and emits both the .sv and the manifest from the same resolved values — no analysis pass, no re-parse; the generator IS the oracle; valid-by-construction/rules-first). Reproducibility contract (seed,knobs → byte-identical .sv + .json). Parity-check harness (separate from the tool_matrix lint/synth DUT gate; downstream consumer reports resolved facts → compared to manifest; exact agreement or retained counterexample; cargo-portable structural-equivalence formalization + repo-owned gate for the genuine tool parity, mirroring memory/FSM). Boundaries (Phase 8 = richer source-level hierarchy/package IR; Phase 9 = the family selector; Phase 7 lands behind an explicit family flag, no selector). 4 rejected alternatives (reuse circuit IR / generate-then-parse / bundle reference elaborator / facts-as-comments). .2 proof shape + split candidates. Design-only; no code; mdbook build book clean; cargo fmt --all --check clean; full cargo test green at base 5db4ac9 (no src/tests touched).`
  Commit: `Docs: PHASE-7-ORACLE-MICRODESIGN.1 oracle-backed micro-design artifact-family design`

- ID: `PHASE-7-ORACLE-MICRODESIGN.2`
  Status: `active`
  Goal: `Implement the micro-design generator + manifest + parity harness per .1, behind an explicit artifact-family selector flag, with a matrix/parity gate. Split per the Splitting Rules along the exact independently-reviewable boundaries .1's design named (const-expr/parameter IR + construction-time evaluator / SV emitter + manifest emitter / parity harness + repo-owned gate).`
  Children: `PHASE-7-ORACLE-MICRODESIGN.2a` (done), `PHASE-7-ORACLE-MICRODESIGN.2b` (done), `PHASE-7-ORACLE-MICRODESIGN.2c` (active container: `.2c.1`, `.2c.2`)

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
  Status: `active`
  Goal: `The parity harness + repo-owned gate: a downstream consumer (Yosys write_json / slang|Verilator param introspection) reports resolved facts; compare to the manifest — exact agreement or a retained counterexample (SV+manifest+tool output). Tool-gated (cargo test stays green tool-less — Phase-1 doctrine, like memory/FSM .2.2 + DIFFERENTIAL .2b). Then verify a clean run and record ROADMAP Phase 7 -> done (r87 no-aspirational-claims: verified artifact precedes the promotion). Split (Splitting Rules + the proven memory .2.3/.2.4 and FSM .3.4a/.3.4b decomposition: the harness machinery is code that lands before any advance; the real-tool gate run + ROADMAP promotion is a separate gated step) into .2c.1 (parity harness build: cargo-portable comparator proof + tool-gated #[ignore] real-tool harness scaffold; no real run, no ROADMAP advance, cargo stays green tool-less) and .2c.2 (run the real tool-equipped #[ignore] gate, verify clean, record ROADMAP Phase 7 -> done — gate-blocked).`
  Children: `PHASE-7-ORACLE-MICRODESIGN.2c.1`, `PHASE-7-ORACLE-MICRODESIGN.2c.2`

- ID: `PHASE-7-ORACLE-MICRODESIGN.2c.1`
  Status: `pending`
  Goal: `Build the parity harness — cargo-portable + tool-gated. A new top-level test file (e.g. tests/microdesign_parity.rs, mirroring tests/pipeline.rs) carrying: (a) a pure-Rust fact-extraction-and-comparison core that operates on already-collected tool output (an in-test synthetic representation, NOT a tool invocation) — testable cargo-portably without yosys/verilator/slang; (b) a tool-equipped #[ignore]-gated test that, when invoked with the tools available (cargo test -- --ignored), drives a fixed deterministic corpus through emit_sv + emit_manifest, shells the chosen downstream consumer on each .sv, parses the resolved-facts report, and feeds it to the cargo-portable comparator core to assert exact agreement (or retain a counterexample tuple of {sv, manifest, tool_output, divergence}). Tool-gated ⇒ portable cargo test stays green tool-less (Phase-1 doctrine; identical to differential-sim .2b and the convention recorded in this tree's Decisions). Cargo-portable comparator proof: deterministic seeds {0,1,7,42,12345} (matching .2a's reproducibility set) × build → manifest, then feed a hand-constructed-to-agree synthetic tool report to the comparator and prove exact-equality; AND feed a deliberately-perturbed synthetic report and prove the comparator surfaces the right divergence kind (param-mismatch / width-mismatch / generate-branch-mismatch / package-constant-mismatch). No ROADMAP advance (that is .2c.2 on verified evidence).`
  Acceptance: `cargo fmt/clippy(-D warnings)/check --all-targets/test green; new tests/microdesign_parity.rs landed with the cargo-portable comparator + the #[ignore]-gated tool harness; cargo-portable comparator proof exact-agrees on synthetic-agree fixtures and surfaces the right divergence kind on each perturbed fixture; the #[ignore] test compiles + is invocable but is NOT run in the portable suite; ROADMAP unchanged (advance is .2c.2 on a verified gate); no book/ change (book reconciliation is .2c.2).`
  Verification: `pending`
  Commit: `pending`

- ID: `PHASE-7-ORACLE-MICRODESIGN.2c.2`
  Status: `pending`
  Goal: `Real tool-equipped run of the .2c.1 #[ignore]-gated parity harness against a fixed deterministic corpus; VERIFY exact-agreement (or zero retained counterexamples) BEFORE any promotion (r87 no-aspirational-claims, mirroring memory .2.4 and FSM .3.4b). Then record ROADMAP Phase 7 -> done (with the explicit artifact-family lane note and the boundary to Phase 8/Phase 9 preserved); reconcile book (the Phase-7 micro-design lane in book/src/ir.md and/or a new "Micro-design lane" page in the book), README phase narrative, CODEBASE_ANALYSIS phase-coverage-map Phase-7 row, MEMORY recent commits. Closes PHASE-7-ORACLE-MICRODESIGN.2c + the .2 container + the PHASE-7-ORACLE-MICRODESIGN tree.`
  Acceptance: `A repo-owned banked artifact captures the harness's exact-agreement on the full corpus (zero retained counterexamples) or — if any divergence — the precise counterexample tuple is committed alongside; ROADMAP Phase 7 -> done only after the verified run; the .2c container + .2 container + tree all -> done. No aspirational claims (verified artifact precedes the ROADMAP promotion).`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `PHASE-7-ORACLE-MICRODESIGN.2c.1` | `pending` | `.2a`+`.2b` **done** — `src/microdesign/` has the const-expr/parameter IR + evaluator/oracle (`.2a`) and the un-resolved SV emitter + JSON expected-facts manifest emitter from the same oracle (`.2b`), 7 unit proofs green, byte-reproducible. `.2c` **split** today per the proven memory `.2.3`/`.2.4` and FSM `.3.4a`/`.3.4b` decomposition into `.2c.1` (build the parity harness — cargo-portable comparator proof + tool-gated `#[ignore]` real-tool harness; no real run, no advance, cargo stays green tool-less) and `.2c.2` (real-tool gate run + ROADMAP Phase 7 promotion; gate-blocked). `.2c.1` is unblocked and is the next slice. |

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

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `PHASE-7-ORACLE-MICRODESIGN.1` | `Docs: PHASE-7-ORACLE-MICRODESIGN.1 oracle-backed micro-design artifact-family design` | Design-only; expected-facts JSON schema + oracle-by-construction strategy + new parity harness + 4 rejected alternatives. No code. |
| `PHASE-7-ORACLE-MICRODESIGN.2` (split) | `Docs: split PHASE-7-ORACLE-MICRODESIGN.2 into .2a (IR+evaluator) / .2b (emitters) / .2c (parity gate)` | Tree-planning, no code. Boundaries per `.1`'s named split candidates. |
| `PHASE-7-ORACLE-MICRODESIGN.2a` | `Phase 7: PHASE-7-ORACLE-MICRODESIGN.2a const-expr/parameter IR + construction-time evaluator (oracle)` | New `src/microdesign/` IR + evaluator/oracle + reproducible rules-first builder; 4 unit proofs; no emit/harness. |
| `PHASE-7-ORACLE-MICRODESIGN.2b` | `Phase 7: PHASE-7-ORACLE-MICRODESIGN.2b SV + JSON expected-facts manifest emitters (from the .2a oracle)` | Un-resolved SV emitter + JSON manifest emitter, both from the `.2a` oracle; 3 new proofs (7 total); byte-reproducible; no harness. |
| `PHASE-7-ORACLE-MICRODESIGN.2c` (split) | `Docs: split PHASE-7-ORACLE-MICRODESIGN.2c into .2c.1 (build harness) + .2c.2 (real-tool gate + ROADMAP Phase 7)` | Tree-planning, no code. Mirrors memory `.2.3`/`.2.4` and FSM `.3.4a`/`.3.4b` decomposition. |

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
