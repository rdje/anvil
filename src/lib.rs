//! anvil — random by-construction generator of synthesizable SystemVerilog RTL.
//!
//! See the `book/` directory for design rationale. The crate is organized
//! around a typed circuit IR (`ir`), a generator that builds it by
//! fanin-cone recursion (`gen`), and an emitter that pretty-prints it as
//! SystemVerilog (`emit`).

pub mod config;
pub mod emit;
pub mod gen;
pub mod ir;
pub mod metrics;

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
