//! DIFFERENTIAL-SIMULATION — iverilog↔verilator differential harness.
//!
//! Implements the design from `DEVELOPMENT_NOTES.md` "Single-design
//! differential harness design (2026-05-18, DIFFERENTIAL-SIMULATION.2a)"
//! and the `.2b.2` cycle-accurate-timing + clk/rst_n-inclusion fixes.
//! Per `.3a` (`DEVELOPMENT_NOTES.md` "Tool-matrix `--diff-sim` wiring +
//! representative-subset selector + coverage fact design"), the
//! helpers live here (library module) so `src/bin/tool_matrix.rs` can
//! `use anvil::diff_sim::{…}` — the full-factorization-doctrine choice
//! over duplicating them in the binary. `tests/diff_sim.rs` consumes
//! the same surface and owns the `#[ignore]`-gated focused tests.
//!
//! Every load-bearing decision named in `.2a` is observed here:
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
//!   `PHASE-7-ORACLE-MICRODESIGN`'s Decisions and applied throughout
//!   the parity gates). The `#[ignore]` gates live in
//!   `tests/diff_sim.rs`; this module is the library API surface.

use crate::ir::{Module, Port};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Command;

/// A deterministic input-vector sequence baked into the testbench
/// from the seed. Per `.2a`'s design — never `$random`. The first
/// few vectors are canonical edge cases (all-zeros, all-ones,
/// walking-1); the remainder are seeded ChaCha8 pseudo-random.
pub fn baked_input_vectors(seed: u64, n_inputs: usize, n_vectors: usize) -> Vec<Vec<u128>> {
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
pub fn mask_to_width(v: u128, width: u32) -> u128 {
    if width >= 128 {
        v
    } else {
        v & ((1u128 << width) - 1)
    }
}

/// Format `v` as a fixed-width SV hex literal (`<width>'h<nibbles>`).
pub fn fmt_sv_hex(v: u128, width: u32) -> String {
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
pub fn is_sequential(top: &Module) -> bool {
    top.has_local_flops() || top.has_local_memories() || top.has_local_fsms()
}

/// Emit a generic SystemVerilog testbench for `top`. Returns the SV
/// text. The testbench is parameter-less and language-portable
/// between iverilog -g2012 and verilator --binary.
///
/// `.2b.2` fixes from `.2b.1`'s first real-tool run:
///
/// 1. **Clock/reset port inclusion bug** — `Module.clock` /
///    `Module.reset` are reserved-slot IR fields that may be
///    `Some` even for pure-combinational modules, but `emit::to_sv`
///    only emits the `clk`/`rst_n` ports when the module has
///    sequential state. The testbench port-map must match the
///    SV-emit's port set, not the IR's reserved-slot set — so we
///    filter `clk`/`rst_n` out of the testbench when
///    `is_sequential(top)` is false.
///
/// 2. **Off-by-one trace-alignment** — `.2b.1`'s `#N`-based
///    sequential timing raced with the posedge event ordering
///    across iverilog vs verilator (iverilog emitted one extra
///    leading sample). Fixed by switching to the standard
///    cycle-accurate idiom: drive inputs at `@(negedge clk)`
///    (a quiet point — no flops fire), let the next `@(posedge
///    clk)` latch them, then sample at the FOLLOWING `@(negedge
///    clk)` when outputs have settled. Both sims agree on edge
///    ordering with this idiom.
pub fn emit_testbench(top: &Module, vectors: &[Vec<u128>]) -> String {
    let seq = is_sequential(top);
    let mut s = String::new();
    s.push_str("// DIFFERENTIAL-SIMULATION.2b — generic Verilator↔iverilog testbench\n");
    s.push_str("module tb;\n");

    // `.2b.2` fix #1: declare `clk`/`rst_n` only when the DUT
    // actually has them. The IR may carry reserved-slot
    // `Module.clock`/`Module.reset` even for combinational
    // modules, but `emit::to_sv` only renders them with sequential
    // state — the testbench port map MUST match the SV-emit's
    // port set.
    let testbench_inputs: Vec<&Port> = top
        .inputs
        .iter()
        .filter(|p| seq || (p.name != "clk" && p.name != "rst_n"))
        .collect();

    // Declare reg/wire for every port the testbench connects.
    for p in &testbench_inputs {
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

    // Instantiate the DUT by named port map — matches the testbench's
    // filtered port set.
    s.push_str(&format!("    {} dut (\n", top.name));
    let all_ports: Vec<&Port> = testbench_inputs
        .iter()
        .copied()
        .chain(top.outputs.iter())
        .collect();
    for (i, p) in all_ports.iter().enumerate() {
        let comma = if i + 1 < all_ports.len() { "," } else { "" };
        s.push_str(&format!("        .{}({}){}\n", p.name, p.name, comma));
    }
    s.push_str("    );\n");

    // Data inputs are the non-clock/non-reset input ports.
    let data_inputs: Vec<&Port> = testbench_inputs
        .iter()
        .copied()
        .filter(|p| p.name != "clk" && p.name != "rst_n")
        .collect();

    if seq {
        // Clock generator: clk toggles every #5, so a full period
        // is 10 time units (posedge at t=5, 15, 25, ...).
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
        // Hold rst_n=0 for 4 full clock cycles, then deassert at
        // a known negedge so subsequent timing is sim-agnostic.
        for _ in 0..4 {
            s.push_str("        @(posedge clk);\n");
        }
        s.push_str("        @(negedge clk);\n");
        s.push_str("        rst_n = 1'b1;\n");
        // 2-cycle warmup with rst_n deasserted.
        for _ in 0..2 {
            s.push_str("        @(posedge clk);\n");
        }
        // Cycle-accurate per-vector loop: drive at negedge (quiet
        // point — no flops fire), let the next posedge latch, then
        // sample at the FOLLOWING negedge when outputs have
        // settled. Both sims agree on edge ordering with this
        // idiom (`.2b.1`'s `#10` raced with the posedge across
        // iverilog vs verilator).
        for v in vectors {
            s.push_str("        @(negedge clk);\n");
            for (i, p) in data_inputs.iter().enumerate() {
                let val = v.get(i).copied().unwrap_or(0);
                s.push_str(&format!(
                    "        {} = {};\n",
                    p.name,
                    fmt_sv_hex(val, p.width)
                ));
            }
            s.push_str("        @(posedge clk);\n");
            s.push_str("        @(negedge clk);\n");
            emit_display_outputs(&mut s, &top.outputs);
        }
        s.push_str("        $finish;\n");
        s.push_str("    end\n");
    } else {
        // Pure combinational: hold + settle + sample.
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

/// Run iverilog: compile + run + capture stdout. Returns `None`
/// when either step fails (caller logs the diagnostic via
/// `eprintln!` already attached to stderr).
pub fn run_iverilog(dir: &Path) -> Option<String> {
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

/// Run verilator --binary: build + run + capture stdout. Returns
/// `None` when either step fails (caller logs via `eprintln!`).
pub fn run_verilator(dir: &Path) -> Option<String> {
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
pub fn normalize_trace(s: &str) -> Vec<String> {
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

/// Probe whether both iverilog and verilator are on `$PATH`. Used
/// by gated tests (`tests/diff_sim.rs`) and the upcoming
/// `tool_matrix --diff-sim` mode (`.3b.2`) to no-op gracefully when
/// either is absent.
pub fn tools_present() -> bool {
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

/// A single cross-simulator agreement outcome — the result of driving
/// one DUT through iverilog **and** verilator and byte-comparing the
/// normalized post-reset traces.
///
/// `BUG-HUNT-ORCHESTRATION.2a` lifted this struct + the run+compare
/// pipeline below out of `src/bin/tool_matrix.rs` (where it lived as
/// `DIFFERENTIAL-SIMULATION.3b.2`) into this library module, so the
/// turnkey bug-hunt loop (`BUG-HUNT-ORCHESTRATION`, decision `0018`)
/// and the acceptance-divergence lane can detect a cross-simulator
/// mismatch through the same hardened surface instead of duplicating
/// the harness. `tool_matrix`'s per-module `diff_sim` column is now a
/// thin wrapper over [`run_agreement`]; the serde shape is unchanged,
/// so `tool_matrix_report.json` stays byte-identical.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffSimReport {
    /// Was the diff-sim gate actually invoked for this module?
    /// `false` when simulators were absent (`tools_present()`
    /// returned false) — the caller still exits clean; the column
    /// just records the reason it didn't run.
    pub ran: bool,
    /// `true` iff `normalize_trace(iverilog) == normalize_trace(verilator)`
    /// (byte-equal post-reset traces) — drives the
    /// `saw_design_with_cross_simulator_agreement` coverage fact.
    pub success: bool,
    /// Number of post-reset sample lines compared (the length of
    /// the normalized trace). Zero when `ran=false`.
    pub n_samples: usize,
    /// Free-form skip reason when `ran=false` (e.g., "iverilog or
    /// verilator absent from $PATH", "verilator pre-step failed",
    /// "yosys pre-step failed"). Empty when `ran=true`.
    pub skip_reason: String,
    /// First-mismatch counterexample excerpt (up to 10 lines from
    /// each side, side-by-side) when `success=false` and `ran=true`.
    /// Per the Phase-7 doctrine — every mismatch is a retained
    /// counterexample, never a silent pass.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mismatch_excerpt: Option<String>,
}

/// A port declaration parsed from an ANVIL-emitted SV header.
#[derive(Debug, Clone)]
pub struct DutPort {
    pub name: String,
    pub width: u32,
    pub is_input: bool,
}

/// Parse the port section of an ANVIL-emitted DUT module
/// (`BUG-HUNT-ORCHESTRATION.2a`, lifted from `tool_matrix`'s
/// `DIFFERENTIAL-SIMULATION.3b.2` path). ANVIL's emitter writes ports
/// as `input  logic [W-1:0] name,` or 1-bit `input  logic  name,`
/// (with *two* spaces between `input` and `logic` — see
/// `src/emit/sv.rs::write!("    input  logic {} {}")`). The parser
/// whitespace-normalises each line via `split_whitespace` rather than
/// fixed-prefix matching, so it's robust to any internal-whitespace
/// variation. Aggregate ports (`input <type> <name>`, no `logic`
/// keyword — Phase 5b) are treated as unrecognised and the function
/// returns `None`; the caller treats that as "skip diff-sim for this
/// module" (the generic testbench cannot type-correctly drive an
/// aggregate without the struct definition in scope).
pub fn parse_dut_ports(sv: &str, top_name: &str) -> Option<Vec<DutPort>> {
    let mut in_module = false;
    let mut in_port_list = false;
    let mut ports: Vec<DutPort> = Vec::new();
    for raw in sv.lines() {
        let line = raw.trim();
        if !in_module {
            if (line.starts_with("module ") || line.starts_with(&format!("module {top_name}")))
                && line.contains(top_name)
            {
                in_module = true;
                if line.contains('(') {
                    in_port_list = true;
                }
            }
            continue;
        }
        if !in_port_list {
            if line.contains('(') {
                in_port_list = true;
            }
            continue;
        }
        if line.starts_with(");") || line == ")" {
            return Some(ports);
        }
        let stripped = line.trim_start_matches('(').trim();
        if stripped.is_empty() {
            continue;
        }
        let trimmed_comma = stripped.trim_end_matches(',').trim();
        let tokens: Vec<&str> = trimmed_comma.split_whitespace().collect();
        // Expected shapes (after split_whitespace):
        //   `input logic <name>`           → 3 tokens, width=1
        //   `input logic [W-1:0] <name>`    → 4 tokens, width from bracket
        //   `output logic <name>`          → 3 tokens, width=1
        //   `output logic [W-1:0] <name>`   → 4 tokens, width from bracket
        // Anything else (Phase-5b aggregate `input <type> <name>`
        // without `logic`, etc.) → bail to caller.
        let (is_input, rest_tokens) = match tokens.first().copied() {
            Some("input") => (true, &tokens[1..]),
            Some("output") => (false, &tokens[1..]),
            _ => return None,
        };
        let after_logic = match rest_tokens.first().copied() {
            Some("logic") => &rest_tokens[1..],
            _ => return None,
        };
        let (width, name) = match after_logic.len() {
            1 => (1u32, after_logic[0].to_string()),
            2 => {
                let bracket = after_logic[0];
                let inner = bracket.strip_prefix('[')?.strip_suffix(']')?;
                let (msb, lsb) = inner.split_once(':')?;
                let msb_val: i64 = msb.trim().parse().ok()?;
                let lsb_val: i64 = lsb.trim().parse().ok()?;
                let width = (msb_val - lsb_val + 1).max(1) as u32;
                (width, after_logic[1].to_string())
            }
            _ => return None,
        };
        if name.is_empty() {
            return None;
        }
        ports.push(DutPort {
            name,
            width,
            is_input,
        });
    }
    None
}

/// Emit a parameter-less SV testbench from a `Vec<DutPort>` (the
/// strict-subset parser's output). The shape mirrors
/// [`emit_testbench`]'s IR-driven version, but is driven from parsed
/// ports — this is the path the SV-text consumer (`tool_matrix`'s
/// `--diff-sim` column, and now the bug-hunt loop) uses when it has
/// no live `Module` in scope. The two emitters share the same
/// testbench shape so behavior is identical; the IR-driven version
/// remains canonical and is what the `#[ignore]`-gated proofs in
/// `tests/diff_sim.rs` exercise. (Unifying the two paths is a deferred
/// cleanup; `.2a` is a byte-identical move, not a merge.)
pub fn emit_testbench_for_ports(top_name: &str, ports: &[DutPort], n_vectors: usize) -> String {
    let has_clk = ports
        .iter()
        .any(|p| p.is_input && p.name == "clk" && p.width == 1);
    let has_rst_n = ports
        .iter()
        .any(|p| p.is_input && p.name == "rst_n" && p.width == 1);
    let seq = has_clk && has_rst_n;
    let inputs: Vec<&DutPort> = ports.iter().filter(|p| p.is_input).collect();
    let outputs: Vec<&DutPort> = ports.iter().filter(|p| !p.is_input).collect();
    let data_inputs: Vec<&DutPort> = inputs
        .iter()
        .copied()
        .filter(|p| p.name != "clk" && p.name != "rst_n")
        .collect();
    let n_data = data_inputs.len();
    let vectors = baked_input_vectors(0, n_data, n_vectors);
    let mut s = String::new();
    s.push_str("// DIFFERENTIAL-SIMULATION.3b.2 — tool_matrix --diff-sim testbench\n");
    s.push_str("module tb;\n");
    for p in &inputs {
        if p.width == 1 {
            s.push_str(&format!("    reg {};\n", p.name));
        } else {
            s.push_str(&format!("    reg [{}:0] {};\n", p.width - 1, p.name));
        }
    }
    for p in &outputs {
        if p.width == 1 {
            s.push_str(&format!("    wire {};\n", p.name));
        } else {
            s.push_str(&format!("    wire [{}:0] {};\n", p.width - 1, p.name));
        }
    }
    s.push_str(&format!("    {top_name} dut (\n"));
    let all_ports: Vec<&DutPort> = inputs
        .iter()
        .copied()
        .chain(outputs.iter().copied())
        .collect();
    for (i, p) in all_ports.iter().enumerate() {
        let comma = if i + 1 < all_ports.len() { "," } else { "" };
        s.push_str(&format!("        .{}({}){}\n", p.name, p.name, comma));
    }
    s.push_str("    );\n");

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
        for _ in 0..4 {
            s.push_str("        @(posedge clk);\n");
        }
        s.push_str("        @(negedge clk);\n");
        s.push_str("        rst_n = 1'b1;\n");
        for _ in 0..2 {
            s.push_str("        @(posedge clk);\n");
        }
        for v in &vectors {
            s.push_str("        @(negedge clk);\n");
            for (i, p) in data_inputs.iter().enumerate() {
                let val = v.get(i).copied().unwrap_or(0);
                s.push_str(&format!(
                    "        {} = {};\n",
                    p.name,
                    fmt_sv_hex(val, p.width)
                ));
            }
            s.push_str("        @(posedge clk);\n");
            s.push_str("        @(negedge clk);\n");
            push_display_for_ports(&mut s, &outputs);
        }
        s.push_str("        $finish;\n");
        s.push_str("    end\n");
    } else {
        s.push_str("    initial begin\n");
        for v in &vectors {
            for (i, p) in data_inputs.iter().enumerate() {
                let val = v.get(i).copied().unwrap_or(0);
                s.push_str(&format!(
                    "        {} = {};\n",
                    p.name,
                    fmt_sv_hex(val, p.width)
                ));
            }
            s.push_str("        #1;\n");
            push_display_for_ports(&mut s, &outputs);
        }
        s.push_str("        $finish;\n");
        s.push_str("    end\n");
    }
    s.push_str("endmodule\n");
    s
}

fn push_display_for_ports(s: &mut String, outputs: &[&DutPort]) {
    if outputs.is_empty() {
        s.push_str("        $display(\"NO_OUT\");\n");
        return;
    }
    let fmt = (0..outputs.len())
        .map(|_| "%h")
        .collect::<Vec<_>>()
        .join(" ");
    s.push_str(&format!("        $display(\"{fmt}\",\n"));
    for (i, p) in outputs.iter().enumerate() {
        let comma = if i + 1 < outputs.len() { "," } else { "" };
        s.push_str(&format!("            {}{}\n", p.name, comma));
    }
    s.push_str("        );\n");
}

/// Drive one DUT through both simulators and byte-compare the
/// normalized post-reset traces, returning a [`DiffSimReport`]
/// (`BUG-HUNT-ORCHESTRATION.2a`, lifted verbatim from `tool_matrix`'s
/// `run_diff_sim_for_module`). `work_dir` is the (caller-chosen)
/// sandbox directory for the run's `dut.sv` / `tb.sv` / sim artifacts
/// — never agent-supplied (decision `0004`); the caller computes it.
/// `sv_text` is the already-emitted DUT SystemVerilog (held verbatim
/// for byte-stability); `top_name` is its top module; `n_vectors` is
/// the baked input-vector count. The harness is friendly when tools
/// are absent: it returns a `ran: false` report with a skip reason
/// rather than failing. It only parses the DUT's port section (a
/// stable strict subset of SV) to build the testbench; the full
/// testbench-from-IR path stays in [`emit_testbench`].
pub fn run_agreement(
    work_dir: &Path,
    top_name: &str,
    sv_text: &str,
    n_vectors: usize,
) -> DiffSimReport {
    if !tools_present() {
        return DiffSimReport {
            ran: false,
            success: false,
            n_samples: 0,
            skip_reason: "iverilog and/or verilator absent from $PATH".to_string(),
            mismatch_excerpt: None,
        };
    }
    let Some(ports) = parse_dut_ports(sv_text, top_name) else {
        return DiffSimReport {
            ran: false,
            success: false,
            n_samples: 0,
            skip_reason: format!("could not parse DUT port section for top `{top_name}`"),
            mismatch_excerpt: None,
        };
    };

    if let Err(e) = std::fs::create_dir_all(work_dir) {
        return DiffSimReport {
            ran: false,
            success: false,
            n_samples: 0,
            skip_reason: format!("create diff-sim work dir failed: {e}"),
            mismatch_excerpt: None,
        };
    }
    let dut_path = work_dir.join("dut.sv");
    let tb_path = work_dir.join("tb.sv");
    let tb_text = emit_testbench_for_ports(top_name, &ports, n_vectors);
    if std::fs::write(&dut_path, sv_text).is_err() || std::fs::write(&tb_path, &tb_text).is_err() {
        return DiffSimReport {
            ran: false,
            success: false,
            n_samples: 0,
            skip_reason: "write dut.sv / tb.sv failed".to_string(),
            mismatch_excerpt: None,
        };
    }
    let Some(iv) = run_iverilog(work_dir) else {
        return DiffSimReport {
            ran: false,
            success: false,
            n_samples: 0,
            skip_reason: "iverilog compile/run failed (see stderr)".to_string(),
            mismatch_excerpt: None,
        };
    };
    let Some(vl) = run_verilator(work_dir) else {
        return DiffSimReport {
            ran: false,
            success: false,
            n_samples: 0,
            skip_reason: "verilator compile/run failed (see stderr)".to_string(),
            mismatch_excerpt: None,
        };
    };
    let norm_iv = normalize_trace(&iv);
    let norm_vl = normalize_trace(&vl);
    if norm_iv.is_empty() {
        return DiffSimReport {
            ran: false,
            success: false,
            n_samples: 0,
            skip_reason: "iverilog produced no hex trace lines".to_string(),
            mismatch_excerpt: None,
        };
    }
    if norm_iv == norm_vl {
        DiffSimReport {
            ran: true,
            success: true,
            n_samples: norm_iv.len(),
            skip_reason: String::new(),
            mismatch_excerpt: None,
        }
    } else {
        // Retained counterexample per the Phase-7 doctrine. First
        // 10 sample lines from each side, side-by-side.
        let mut excerpt = String::new();
        excerpt.push_str("iverilog | verilator\n");
        let n = norm_iv.len().min(norm_vl.len()).min(10);
        for i in 0..n {
            excerpt.push_str(&format!("{} | {}\n", norm_iv[i], norm_vl[i]));
        }
        if norm_iv.len() != norm_vl.len() {
            excerpt.push_str(&format!(
                "(traces differ in length: iverilog={} vs verilator={})\n",
                norm_iv.len(),
                norm_vl.len()
            ));
        }
        DiffSimReport {
            ran: true,
            success: false,
            n_samples: norm_iv.len(),
            skip_reason: String::new(),
            mismatch_excerpt: Some(excerpt),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `parse_dut_ports` recognises ANVIL's emitter port shape
    /// (moved here with the function in `BUG-HUNT-ORCHESTRATION.2a`).
    #[test]
    fn parse_dut_ports_recognises_anvil_emitter_shape() {
        // Synthetic ANVIL-shape SV header. The strict-subset
        // parser only needs the port declarations between `(` and
        // `);` after `module <top> (`.
        let sv = "\
module dummy_top (\n\
    input logic clk,\n\
    input logic rst_n,\n\
    input logic [7:0] i_a,\n\
    input logic [0:0] i_b,\n\
    output logic [15:0] o_x,\n\
    output logic o_y\n\
);\n\
endmodule\n";
        let ports = parse_dut_ports(sv, "dummy_top").expect("parse should succeed");
        assert_eq!(ports.len(), 6);
        let names: Vec<_> = ports.iter().map(|p| p.name.as_str()).collect();
        assert_eq!(names, vec!["clk", "rst_n", "i_a", "i_b", "o_x", "o_y"]);
        assert_eq!(ports[0].width, 1);
        assert_eq!(ports[2].width, 8);
        assert_eq!(ports[4].width, 16);
        assert!(ports[0].is_input);
        assert!(!ports[4].is_input);
    }

    /// `emit_testbench_for_ports` renders the combinational and
    /// sequential testbench shapes (moved here with the function in
    /// `BUG-HUNT-ORCHESTRATION.2a`).
    #[test]
    fn emit_testbench_for_ports_renders_combinational_and_sequential_shapes() {
        let comb_ports = vec![
            DutPort {
                name: "i_a".to_string(),
                width: 4,
                is_input: true,
            },
            DutPort {
                name: "o_y".to_string(),
                width: 4,
                is_input: false,
            },
        ];
        let comb_tb = emit_testbench_for_ports("comb_top", &comb_ports, 4);
        assert!(comb_tb.contains("module tb;"));
        assert!(comb_tb.contains("comb_top dut ("));
        assert!(comb_tb.contains("$display("));
        assert!(comb_tb.contains("#1;"));
        // Combinational: no clock generator.
        assert!(!comb_tb.contains("always #5 clk = ~clk;"));

        let seq_ports = vec![
            DutPort {
                name: "clk".to_string(),
                width: 1,
                is_input: true,
            },
            DutPort {
                name: "rst_n".to_string(),
                width: 1,
                is_input: true,
            },
            DutPort {
                name: "i_a".to_string(),
                width: 4,
                is_input: true,
            },
            DutPort {
                name: "o_y".to_string(),
                width: 4,
                is_input: false,
            },
        ];
        let seq_tb = emit_testbench_for_ports("seq_top", &seq_ports, 4);
        assert!(seq_tb.contains("module tb;"));
        assert!(seq_tb.contains("seq_top dut ("));
        assert!(seq_tb.contains("always #5 clk = ~clk;"));
        // Sequential: cycle-accurate negedge/posedge idiom.
        assert!(seq_tb.contains("@(posedge clk);"));
        assert!(seq_tb.contains("@(negedge clk);"));
        // Reset is asserted in the prologue.
        assert!(seq_tb.contains("rst_n = 1'b0;"));
    }

    /// `run_agreement` is a friendly no-op when the simulators are
    /// absent — it returns `ran: false` with a clear skip reason
    /// rather than failing (`BUG-HUNT-ORCHESTRATION.2a`). Tool-gated
    /// agreement proofs live in `tests/diff_sim.rs` / the
    /// `tool_matrix` `#[ignore]` e2e gate.
    #[test]
    fn run_agreement_is_a_friendly_no_op_without_tools() {
        if tools_present() {
            // On a machine with both simulators this path can't be
            // exercised without spawning them; the tool-gated e2e
            // gate covers the present-tools case. Skip cleanly.
            return;
        }
        let dir = std::env::temp_dir().join("anvil-diff-sim-noop-probe");
        let report = run_agreement(&dir, "m", "module m (\n);\nendmodule\n", 4);
        assert!(!report.ran);
        assert!(!report.success);
        assert_eq!(report.n_samples, 0);
        assert!(report.skip_reason.contains("absent from $PATH"));
    }

    /// `baked_input_vectors` is reproducible for fixed `(seed,
    /// n_inputs, n_vectors)` and starts with the documented
    /// canonical-edge-case triple.
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

    /// `fmt_sv_hex` produces fixed-width SV hex literals matching
    /// the declared port width, masked.
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
}
