//! SystemVerilog emitter. IR → text.

pub mod sv;

pub use sv::{to_sv, to_sv_design, to_sv_in_design};
