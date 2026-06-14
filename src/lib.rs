//! anvil — random by-construction generator of synthesizable SystemVerilog RTL.
//!
//! See the `book/` directory for design rationale. The crate is organized
//! around a typed circuit IR (`ir`), a generator that builds it by
//! fanin-cone recursion (`gen`), and an emitter that pretty-prints it as
//! SystemVerilog (`emit`).

pub mod config;
/// `DIFFERENTIAL-SIMULATION` — iverilog↔verilator differential
/// harness. Per `.3a`'s design, the helpers live in this library
/// module so `src/bin/tool_matrix.rs` can `use anvil::diff_sim::{…}`
/// (full-factorization doctrine, `feedback_full_factorization.md`)
/// — the alternative of duplicating them in the binary is forbidden.
/// `tests/diff_sim.rs` consumes the same surface and owns the
/// `#[ignore]`-gated focused tests (`differential_simulation_combinational`
/// + `differential_simulation_sequential`). The upcoming `.3b.2`
/// adds a `tool_matrix --diff-sim` opt-in column.
pub mod diff_sim;
/// Hardened downstream-tool invocation surface
/// (`AGENT-INTROSPECTION-MCP.5.1`). The single source of truth for the
/// `verilator --lint-only` / `yosys synth` / `iverilog -g2012` acceptance
/// command lines (plus warning-as-failure detection) that `tool_matrix` has
/// used since Phase 1. Extracted from the binary so the controlled `validate`
/// / `minimize` MCP tools (`.5.2` / `.5.3`, decision `0004`) can reuse the
/// *existing hardened invocations* instead of forking a second, drift-prone
/// set — the same full-factorization move that put [`diff_sim`] in the lib.
pub mod downstream;
pub mod emit;
/// Phase 8 frontend / elaboration accept-corpus lane
/// (`PHASE-8-FRONTEND-ACCEPT`). A **separate generator path** from
/// the DUT lane: a source-level **AST IR** (`SourceUnit` → `Package` →
/// `Module` → `ModuleItem`) with a **construction-time elaboration-
/// evaluator** that resolves every parameter, generate predicate, and
/// instance binding as the IR is built — the *oracle*. Extends Phase
/// 7's `ConstExpr` / `eval` core with hierarchy: packages, modules,
/// instances, generate-if blocks. Deliberately not threaded through
/// the gate-level circuit IR (`ir`) — the circuit IR cannot express
/// modules/params/packages/generate, the category error `.1`
/// rejected.
pub mod frontend;
pub mod gen;
/// Agent-introspection emission surface (`AGENT-INTROSPECTION-MCP.3`).
/// Builds the versioned introspection document specified in
/// `docs/AGENT_INTROSPECTION_SCHEMA.md` from facts ANVIL already records
/// (`Config` / `Metrics` / `DesignMetrics`). Invariant SCHEMA-DERIVED:
/// every payload field is a `serde` projection of an existing struct —
/// the adapter computes zero new truth. Read-mostly, additive, and
/// default-off (the `--introspect` CLI flag); the default `anvil` build
/// stays byte-identical.
pub mod introspect;
pub mod ir;
/// Streaming `manifest.json` writer (`WORKLOAD-MEMORY-SAFETY.2`).
/// Writes the directory-output manifest array element-by-element so
/// peak metadata memory is O(1) in `--count` instead of O(`--count`),
/// byte-identical to the previous accumulate-then-`to_string_pretty`
/// path. See `src/manifest.rs`.
pub mod manifest;
/// Read-only in-process MCP server (`AGENT-INTROSPECTION-MCP.4`).
/// A thin, dependency-light JSON-RPC 2.0 dispatcher (newline-delimited
/// stdio, driven by the `anvil-mcp` bin) that exposes the deterministic
/// generator as MCP **resources** + pure/safe **tools**
/// (`generate`/`introspect`/`dump_config`) over a content-addressed
/// cache. No external-tool execution (that is `.5`); the default `anvil`
/// build and the `--artifact dut` contract are unaffected.
pub mod mcp;
/// Opt-in internal RAM/RSS self-governor (`WORKLOAD-MEMORY-SAFETY.4`).
/// Default-off / byte-identical; aborts an `--out` run cleanly before
/// the host danger zone, naming the seed + effective knobs. See
/// `src/mem_guard.rs`.
pub mod mem_guard;
pub mod metrics;
/// Phase 7 oracle-backed micro-design lane (`PHASE-7-ORACLE-MICRODESIGN`).
/// A **separate generator path** from the DUT lane: a source-level
/// const-expr / parameter IR + construction-time evaluator (the
/// oracle). Deliberately not threaded through the gate-level circuit
/// IR (`ir`).
pub mod microdesign;
/// Phase 9 multi-artifact umbrella (`PHASE-9-MULTI-ARTIFACT-UMBRELLA`).
/// Unifies the **plumbing** across the three delivered artifact lanes
/// — DUT RTL (Phases 1–6), oracle-backed micro-design (Phase 7,
/// `microdesign`), and frontend / elaboration accept (Phase 8,
/// `frontend`) — via the `ArtifactLane` trait. Explicit anti-goal:
/// never collapse the three lanes' rules-first generators into one
/// "random SV generator"; only their plumbing (seed→artifact,
/// byte-stable output, optional manifest, downstream check plan)
/// unifies here.
pub mod umbrella;

pub use config::Config;
pub use gen::{Generator, GeneratorCheckpoint};
pub use ir::{Design, Module};

// ---------------------------------------------------------------
// Trace verbosity toggle for `--trace debug` (highest verbosity).
// ---------------------------------------------------------------
//
// `tracing`'s level enum tops out at `TRACE`, so `high` and `debug`
// both route to that level at the subscriber. To give `debug` strictly
// more coverage than `high`, extra verbose events guard themselves
// with this atomic flag — set to `true` only when the CLI level is
// `debug`.
//
// Use the `trace_verbose!` macro rather than loading the flag
// directly.
use std::sync::atomic::{AtomicBool, Ordering};

static TRACE_DEBUG: AtomicBool = AtomicBool::new(false);

/// Enable (`true`) or disable (`false`) the `--trace debug`
/// super-verbose events. Called by the binary from `init_tracing`.
pub fn set_trace_debug(enabled: bool) {
    TRACE_DEBUG.store(enabled, Ordering::Relaxed);
}

/// True iff the current process was started with `--trace debug`.
#[inline]
pub fn trace_debug_enabled() -> bool {
    TRACE_DEBUG.load(Ordering::Relaxed)
}

/// `tracing::trace!` guarded by the `--trace debug` flag. Used for
/// super-verbose per-branch / per-intern events that would flood
/// the output at `--trace high` and are only desired when the user
/// is debugging the generator itself.
#[macro_export]
macro_rules! trace_verbose {
    ($($arg:tt)*) => {
        if $crate::trace_debug_enabled() {
            ::tracing::trace!($($arg)*);
        }
    };
}
