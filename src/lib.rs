//! anvil — constrained-random generator of synthesizable SystemVerilog RTL.
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
pub use gen::Generator;
pub use ir::Module;
