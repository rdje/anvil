//! DIFFERENTIAL-SIMULATION — `#[ignore]`-gated focused tests +
//! cargo-portable helper proofs.
//!
//! Per `.3a`'s design (`DEVELOPMENT_NOTES.md` "Tool-matrix
//! `--diff-sim` wiring + representative-subset selector + coverage
//! fact design") the harness helpers themselves were extracted to
//! `src/diff_sim/mod.rs` in `.3b.1` so `src/bin/tool_matrix.rs` can
//! share them — full-factorization doctrine,
//! `feedback_full_factorization.md`. This integration-test file
//! retains the load-bearing `#[ignore]`-gated focused proofs (one
//! combinational + one sequential) plus a smoke proof of
//! `emit_testbench` against a real generated `Module` (the unit
//! tests in `src/diff_sim/mod.rs::tests` cover the pure-input
//! helpers without pulling in the generator).

use anvil::diff_sim::{
    baked_input_vectors, emit_testbench, is_sequential, normalize_trace, run_iverilog,
    run_verilator, tools_present,
};
use anvil::ir::Module;
use anvil::{Config, Generator};

fn build_one_module(seed: u64, sequential: bool) -> Module {
    // Override only what the diff-sim shape needs; leave the rest at
    // `Config::default()` (struct-update keeps clippy happy).
    let cfg = Config {
        seed,
        flop_prob: if sequential { 1.0 } else { 0.0 },
        ..Config::default()
    };
    let mut gen = Generator::new(cfg);
    gen.generate_module()
}

// ===================================================================
// `.2b`'s tool-equipped `#[ignore]` proofs.
// ===================================================================

/// Combinational differential proof: drive both sims, byte-compare
/// post-reset traces. `#[ignore]` so the portable `cargo test`
/// stays green tool-less (Phase-1 doctrine).
#[test]
#[ignore]
fn differential_simulation_combinational() {
    if !tools_present() {
        eprintln!(
            "differential_simulation_combinational: iverilog and/or verilator not on $PATH \
             (skipping; rerun with both installed for the differential gate)"
        );
        return;
    }
    let seed = 7u64;
    let top = build_one_module(seed, false);
    let n_inputs = top
        .inputs
        .iter()
        .filter(|p| p.name != "clk" && p.name != "rst_n")
        .count();
    let vectors = baked_input_vectors(seed, n_inputs, 8);
    let tb_sv = emit_testbench(&top, &vectors);

    let dir = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("diff-sim-comb");
    std::fs::create_dir_all(&dir).expect("can create harness dir");
    std::fs::write(dir.join("dut.sv"), anvil::emit::to_sv(&top)).expect("write dut.sv");
    std::fs::write(dir.join("tb.sv"), &tb_sv).expect("write tb.sv");

    let trace_iv = run_iverilog(&dir).expect("iverilog must run");
    let trace_vl = run_verilator(&dir).expect("verilator must run");
    let norm_iv = normalize_trace(&trace_iv);
    let norm_vl = normalize_trace(&trace_vl);

    assert!(
        !norm_iv.is_empty(),
        "iverilog produced no hex trace lines; raw=\n{trace_iv}"
    );
    assert_eq!(
        norm_iv, norm_vl,
        "combinational differential mismatch (seed={seed}, dir={dir:?}); \
         iverilog=\n{trace_iv}\nverilator=\n{trace_vl}"
    );
    eprintln!(
        "differential_simulation_combinational clean across {} samples (seed={seed})",
        norm_iv.len()
    );
}

/// Sequential differential proof: same shape with reset + warmup +
/// per-cycle sampling. The post-reset canonical sample point
/// neutralises the pre-reset 4-state gap from `.1`.
#[test]
#[ignore]
fn differential_simulation_sequential() {
    if !tools_present() {
        eprintln!(
            "differential_simulation_sequential: iverilog and/or verilator not on $PATH \
             (skipping; rerun with both installed for the differential gate)"
        );
        return;
    }
    let seed = 42u64;
    let top = build_one_module(seed, true);
    let n_inputs = top
        .inputs
        .iter()
        .filter(|p| p.name != "clk" && p.name != "rst_n")
        .count();
    let vectors = baked_input_vectors(seed, n_inputs, 8);
    let tb_sv = emit_testbench(&top, &vectors);

    let dir = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("diff-sim-seq");
    std::fs::create_dir_all(&dir).expect("can create harness dir");
    std::fs::write(dir.join("dut.sv"), anvil::emit::to_sv(&top)).expect("write dut.sv");
    std::fs::write(dir.join("tb.sv"), &tb_sv).expect("write tb.sv");

    let trace_iv = run_iverilog(&dir).expect("iverilog must run");
    let trace_vl = run_verilator(&dir).expect("verilator must run");
    let norm_iv = normalize_trace(&trace_iv);
    let norm_vl = normalize_trace(&trace_vl);

    assert!(
        !norm_iv.is_empty(),
        "iverilog produced no hex trace lines; raw=\n{trace_iv}"
    );
    assert_eq!(
        norm_iv, norm_vl,
        "sequential differential mismatch (seed={seed}, dir={dir:?}); \
         iverilog=\n{trace_iv}\nverilator=\n{trace_vl}"
    );
    eprintln!(
        "differential_simulation_sequential clean across {} post-reset samples (seed={seed})",
        norm_iv.len()
    );
}

// ===================================================================
// Cargo-portable proofs that round-trip through a real generated
// `Module` — exercises `is_sequential` + `emit_testbench` against
// the live generator (the unit tests in `src/diff_sim/mod.rs::tests`
// cover the pure-input helpers without pulling in the generator,
// which keeps the lib unit suite cheap).
// ===================================================================

/// `is_sequential` returns `true` iff the module declares a clock
/// port — drives the testbench shape selection. Smoke-shape against
/// a `flop_prob=0.0` build (combinational) and a `flop_prob=1.0`
/// build (sequential).
#[test]
fn is_sequential_matches_clock_presence() {
    let comb = build_one_module(1, false);
    assert!(
        !is_sequential(&comb),
        "flop_prob=0.0 module should have no clock"
    );
    let seq = build_one_module(1, true);
    assert!(
        is_sequential(&seq),
        "flop_prob=1.0 module should have a clock"
    );
}

/// `emit_testbench` renders the documented shape: a `tb` module
/// containing the DUT instance, the input drivers, and at least one
/// `$display` per sample line.
#[test]
fn emit_testbench_has_the_documented_shape() {
    let top = build_one_module(7, false);
    let vectors = baked_input_vectors(7, top.inputs.len(), 4);
    let sv = emit_testbench(&top, &vectors);
    assert!(sv.contains("module tb;"));
    assert!(sv.contains(&format!("{} dut (", top.name)));
    assert!(sv.contains("$display("));
    assert!(sv.contains("$finish;"));
    assert!(sv.contains("endmodule"));
}
