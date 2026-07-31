//! The circuit IR. See `book/src/ir.md` for rationale.

pub mod aggregate;
pub mod case_mux_if_emit;
pub mod casez_mux_if_emit;
pub mod compact;
pub mod cone_function_emit;
pub mod dedup;
pub mod function_emit;
pub mod generate_loop;
/// The canonicalization engine (`IR-TYPES-DECOMPOSITION.3`). It adds an inherent
/// `impl Module`, so there is nothing to re-export: every `intern_gate` /
/// `intern_constant` call site resolves through `Module` exactly as before.
pub mod intern;
pub mod knob_id;
pub mod knob_roll;
pub mod multi_output_task_emit;
pub mod mux_if_emit;
pub mod param;
pub mod soft_union;
pub mod task_emit;
pub mod types;
pub mod validate;

pub use compact::*;
pub use knob_id::*;
pub use types::*;
