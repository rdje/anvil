//! The circuit IR. See `book/src/ir.md` for rationale.

pub mod aggregate;
pub mod compact;
pub mod dedup;
pub mod param;
pub mod soft_union;
pub mod types;
pub mod validate;

pub use compact::*;
pub use types::*;
