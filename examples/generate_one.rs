//! Minimal library-usage example.
//!
//! Run with:  cargo run --example generate_one

use anvil::{Config, Generator};

fn main() {
    let cfg = Config {
        seed: 42,
        ..Config::default()
    };
    let mut g = Generator::new(cfg);
    let m = g.generate_module();
    println!("{}", anvil::emit::to_sv(&m));
}
