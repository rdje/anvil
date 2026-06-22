//! The circuit IR. See `book/src/ir.md` for rationale.

pub mod aggregate;
pub mod case_mux_if_emit;
pub mod casez_mux_if_emit;
pub mod compact;
pub mod cone_function_emit;
pub mod dedup;
pub mod function_emit;
pub mod generate_loop;
pub mod multi_output_task_emit;
pub mod mux_if_emit;
pub mod param;
pub mod soft_union;
pub mod task_emit;
pub mod types;
pub mod validate;

pub use compact::*;
pub use types::*;
