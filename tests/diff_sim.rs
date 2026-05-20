//! DIFFERENTIAL-SIMULATION.2b — single-design Verilator↔iverilog
//! differential harness.
//!
//! Per `.2a`'s design (DEVELOPMENT_NOTES.md): given a canonical
//! `(seed, config)`, the harness drives the generated DUT through
//! **both** Verilator and iverilog and asserts the post-reset
//! output traces are byte-identical. Every load-bearing decision
//! `.2a` named is observed here:
//!
//! - **Testbench from IR, not by re-parsing SV.** The harness asks
//!   the in-process `Module` for `inputs`/`outputs`/`clock`/`reset`;
//!   the generic SV testbench is emitted from that typed IR.
//! - **Reset + canonical post-reset sample point** neutralises
//!   `.1`'s pre-reset 4-state divergence. Combinational: hold +
//!   settle + sample (no clock). Sequential: `rst_n=0` K cycles →
//!   deassert → fixed warmup → per-cycle sampling.
//! - **Deterministic stimulus baked into the testbench from the
//!   seed.** No `$random` — iverilog and Verilator have different
//!   streams; identical baked vectors guarantee both sims see the
//!   same inputs.
//! - **Dual-simulator orchestration** via `iverilog -g2012 -o
//!   sim.vvp; vvp` and `verilator --binary -j0 -sv --top-module tb`.
//!   Both display the identical fixed-width-hex trace format; the
//!   harness byte-compares.
//! - **Tool-gated `#[ignore]` test** keeps the portable `cargo test`
//!   green tool-less (Phase-1 doctrine, reaffirmed in
//!   `PHASE-7-ORACLE-MICRODESIGN`'s Decisions and applied
//!   throughout the parity gates).

use anvil::ir::{Module, Port};
use anvil::{Config, Generator};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use std::path::Path;
use std::process::Command;

/// A deterministic input-vector sequence baked into the testbench
/// from the seed. Per `.2a`'s design — never `$random`. The first
/// few vectors are canonical edge cases (all-zeros, all-ones,
/// walking-1); the remainder are seeded ChaCha8 pseudo-random.
fn baked_input_vectors(seed: u64, n_inputs: usize, n_vectors: usize) -> Vec<Vec<u128>> {
    let mut out: Vec<Vec<u128>> = Vec::with_capacity(n_vectors);
    if n_inputs == 0 {
        return out;
    }
    out.push(vec![0u128; n_inputs]);
    out.push(vec![u128::MAX; n_inputs]);
    let mut walked = vec![0u128; n_inputs];
    walked[0] = 1;
    out.push(walked);
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    while out.len() < n_vectors {
        let v: Vec<u128> = (0..n_inputs).map(|_| rng.gen::<u128>()).collect();
        out.push(v);
    }
    out
}

/// Mask `v` to `width` bits.
fn mask_to_width(v: u128, width: u32) -> u128 {
    if width >= 128 {
        v
    } else {
        v & ((1u128 << width) - 1)
    }
}

/// Format `v` as a fixed-width SV hex literal (`<width>'h<nibbles>`).
fn fmt_sv_hex(v: u128, width: u32) -> String {
    let nibbles = width.div_ceil(4) as usize;
    format!(
        "{}'h{:0width$x}",
        width,
        mask_to_width(v, width),
        width = nibbles
    )
}

/// Whether the IR module has any sequential state — drives the
/// "clock + reset + warmup + per-cycle" testbench shape vs the pure
/// "hold + settle + sample" combinational shape.
///
/// Per ANVIL's synchronous-design discipline (DEVELOPMENT_NOTES.md
/// "Synchronous-design discipline"), every module declares
/// `clk`/`rst_n` ports unconditionally, so `Module.clock.is_some()`
/// does NOT discriminate combinational vs sequential modules.
/// Instead we ask the IR directly via `has_local_flops()` /
/// `has_local_memories()` / `has_local_fsms()` — exactly the
/// sequential-state predicate `.2a`'s design entry named.
fn is_sequential(top: &Module) -> bool {
    top.has_local_flops() || top.has_local_memories() || top.has_local_fsms()
}

/// Emit a generic SystemVerilog testbench for `top`. Returns the SV
/// text. The testbench is parameter-less and language-portable
/// between iverilog -g2012 and verilator --binary.
fn emit_testbench(top: &Module, vectors: &[Vec<u128>]) -> String {
    let seq = is_sequential(top);
    let mut s = String::new();
    s.push_str("// DIFFERENTIAL-SIMULATION.2b — generic Verilator↔iverilog testbench\n");
    s.push_str("module tb;\n");

    // Declare reg/wire for every port (inputs + outputs).
    for p in &top.inputs {
        if p.width == 1 {
            s.push_str(&format!("    reg {};\n", p.name));
        } else {
            s.push_str(&format!("    reg [{}:0] {};\n", p.width - 1, p.name));
        }
    }
    for p in &top.outputs {
        if p.width == 1 {
            s.push_str(&format!("    wire {};\n", p.name));
        } else {
            s.push_str(&format!("    wire [{}:0] {};\n", p.width - 1, p.name));
        }
    }

    // Instantiate the DUT by named port map.
    s.push_str(&format!("    {} dut (\n", top.name));
    let all_ports: Vec<&Port> = top.inputs.iter().chain(top.outputs.iter()).collect();
    for (i, p) in all_ports.iter().enumerate() {
        let comma = if i + 1 < all_ports.len() { "," } else { "" };
        s.push_str(&format!("        .{}({}){}\n", p.name, p.name, comma));
    }
    s.push_str("    );\n");

    // Data inputs are the non-clock/non-reset input ports.
    let data_inputs: Vec<&Port> = top
        .inputs
        .iter()
        .filter(|p| p.name != "clk" && p.name != "rst_n")
        .collect();

    if seq {
        s.push_str("    initial clk = 1'b0;\n");
        s.push_str("    always #5 clk = ~clk;\n");
        s.push_str("    initial begin\n");
        s.push_str("        rst_n = 1'b0;\n");
        for p in &data_inputs {
            s.push_str(&format!(
                "        {} = {};\n",
                p.name,
                fmt_sv_hex(0, p.width)
            ));
        }
        // Hold rst_n=0 across several cycles, then deassert + warmup.
        s.push_str("        #45;\n");
        s.push_str("        rst_n = 1'b1;\n");
        s.push_str("        #20;\n");
        for v in vectors {
            for (i, p) in data_inputs.iter().enumerate() {
                let val = v.get(i).copied().unwrap_or(0);
                s.push_str(&format!(
                    "        {} = {};\n",
                    p.name,
                    fmt_sv_hex(val, p.width)
                ));
            }
            s.push_str("        #10;\n");
            emit_display_outputs(&mut s, &top.outputs);
        }
        s.push_str("        $finish;\n");
        s.push_str("    end\n");
    } else {
        s.push_str("    initial begin\n");
        for v in vectors {
            for (i, p) in data_inputs.iter().enumerate() {
                let val = v.get(i).copied().unwrap_or(0);
                s.push_str(&format!(
                    "        {} = {};\n",
                    p.name,
                    fmt_sv_hex(val, p.width)
                ));
            }
            s.push_str("        #1;\n");
            emit_display_outputs(&mut s, &top.outputs);
        }
        s.push_str("        $finish;\n");
        s.push_str("    end\n");
    }

    s.push_str("endmodule\n");
    s
}

/// `$display` each output as `%h` joined by spaces (one per port
/// per sample). Stable across iverilog -g2012 and verilator
/// --binary; the harness's `normalize_trace` filters to the
/// hex-only lines.
fn emit_display_outputs(s: &mut String, outputs: &[Port]) {
    if outputs.is_empty() {
        // No outputs — emit a marker line so the trace still has
        // one line per sample. (`%d 0` is benign and stable.)
        s.push_str("        $display(\"NO_OUT\");\n");
        return;
    }
    let fmt = (0..outputs.len())
        .map(|_| "%h")
        .collect::<Vec<_>>()
        .join(" ");
    s.push_str(&format!("        $display(\"{}\",\n", fmt));
    for (i, p) in outputs.iter().enumerate() {
        let comma = if i + 1 < outputs.len() { "," } else { "" };
        s.push_str(&format!("            {}{}\n", p.name, comma));
    }
    s.push_str("        );\n");
}

/// Run iverilog: compile + run + capture stdout.
fn run_iverilog(dir: &Path) -> Option<String> {
    let dut = dir.join("dut.sv");
    let tb = dir.join("tb.sv");
    let vvp_out = dir.join("sim.vvp");
    let compile = Command::new("iverilog")
        .arg("-g2012")
        .arg("-o")
        .arg(&vvp_out)
        .arg(&dut)
        .arg(&tb)
        .output()
        .ok()?;
    if !compile.status.success() {
        eprintln!(
            "iverilog compile failed: stderr=\n{}",
            String::from_utf8_lossy(&compile.stderr)
        );
        return None;
    }
    let run = Command::new("vvp").arg(&vvp_out).output().ok()?;
    if !run.status.success() {
        eprintln!(
            "vvp run failed: stderr=\n{}",
            String::from_utf8_lossy(&run.stderr)
        );
        return None;
    }
    Some(String::from_utf8_lossy(&run.stdout).into_owned())
}

/// Run verilator --binary: build + run + capture stdout.
fn run_verilator(dir: &Path) -> Option<String> {
    let dut = dir.join("dut.sv");
    let tb = dir.join("tb.sv");
    let build = Command::new("verilator")
        .args(["--binary", "-j", "0", "-sv", "--top-module", "tb", "--Mdir"])
        .arg(dir.join("obj_dir"))
        .arg(&dut)
        .arg(&tb)
        .current_dir(dir)
        .output()
        .ok()?;
    if !build.status.success() {
        eprintln!(
            "verilator build failed: stderr=\n{}",
            String::from_utf8_lossy(&build.stderr)
        );
        return None;
    }
    let bin = dir.join("obj_dir").join("Vtb");
    let run = Command::new(&bin).output().ok()?;
    if !run.status.success() {
        eprintln!(
            "verilator run failed: stderr=\n{}",
            String::from_utf8_lossy(&run.stderr)
        );
        return None;
    }
    Some(String::from_utf8_lossy(&run.stdout).into_owned())
}

/// Normalize a trace to its hex-only lines (the `$display` output).
/// Both simulators may emit timing / version / config preamble
/// lines that aren't part of the trace.
fn normalize_trace(s: &str) -> Vec<String> {
    s.lines()
        .map(|l| l.trim())
        .filter(|l| {
            !l.is_empty()
                && l.split_whitespace()
                    .all(|tok| tok.chars().all(|c| c.is_ascii_hexdigit()))
        })
        .map(|l| l.to_string())
        .collect()
}

fn tools_present() -> bool {
    let iv = Command::new("iverilog")
        .arg("-V")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    let vl = Command::new("verilator")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    iv && vl
}

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
// Cargo-portable proofs of the harness helpers — exercise the
// stimulus generator, hex formatter, sequential-detection, and trace
// normalizer without any tool dependency, so the portable suite
// gains coverage of the load-bearing emit + parse logic.
// ===================================================================

/// `baked_input_vectors` is reproducible for fixed `(seed, n_inputs,
/// n_vectors)` and starts with the documented canonical-edge-case
/// triple.
#[test]
fn baked_input_vectors_are_reproducible_with_canonical_edge_cases() {
    let a = baked_input_vectors(7, 3, 8);
    let b = baked_input_vectors(7, 3, 8);
    assert_eq!(a, b);
    assert_eq!(a.len(), 8);
    assert_eq!(a[0], vec![0u128, 0, 0]);
    assert_eq!(a[1], vec![u128::MAX, u128::MAX, u128::MAX]);
    assert_eq!(a[2], vec![1, 0, 0]);
    let c = baked_input_vectors(42, 3, 8);
    assert_ne!(a, c, "distinct seeds must differ in the pseudo-random tail");
}

/// `fmt_sv_hex` produces fixed-width SV hex literals matching the
/// declared port width, masked.
#[test]
fn fmt_sv_hex_produces_fixed_width_masked_literals() {
    assert_eq!(fmt_sv_hex(0xa, 4), "4'ha");
    assert_eq!(fmt_sv_hex(0, 1), "1'h0");
    assert_eq!(fmt_sv_hex(1, 1), "1'h1");
    assert_eq!(fmt_sv_hex(0xabc, 8), "8'hbc");
    assert_eq!(fmt_sv_hex(0x1ff, 9), "9'h1ff");
    assert_eq!(
        fmt_sv_hex(u128::MAX, 128),
        format!("128'h{:032x}", u128::MAX)
    );
}

/// `normalize_trace` filters to hex-only lines.
#[test]
fn normalize_trace_filters_to_hex_only_lines() {
    let raw = "\
        // banner\n\
        VERILATOR_VERSION 5.046\n\
        deadbeef\n\
        \n\
        ca fe ba be\n\
        Finished at time 100\n\
        beef\n";
    let n = normalize_trace(raw);
    // "ca fe ba be" passes the per-token hex test (every
    // whitespace-separated token is hex), so it's accepted.
    assert_eq!(n, vec!["deadbeef", "ca fe ba be", "beef"]);
}

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
