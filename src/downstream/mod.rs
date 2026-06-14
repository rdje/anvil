//! Hardened downstream-tool invocation surface (`AGENT-INTROSPECTION-MCP.5.1`).
//!
//! This is the **single source of truth** for the acceptance-tool command
//! lines ANVIL runs against its emitted SystemVerilog: `verilator
//! --lint-only`, `yosys synth`, and `iverilog -g2012` compile/elaborate,
//! together with the warning-as-failure detection that makes a noisy-but-
//! exit-0 tool run count as a failure. These invocations were proven across
//! every phase gate (Phases 1–9, the banked `tool_matrix` reports), so they
//! are the *vetted* command lines decision `0004` requires the agent lane to
//! reuse — never a second, drift-prone set.
//!
//! ## Why this lives in the library
//!
//! Until `.5.1` these functions lived inside the `tool_matrix` **binary**, so
//! nothing else could call them. The controlled `validate` / `minimize` MCP
//! tools (`.5.2` / `.5.3`, decision `0004`) must run external tools **only via
//! the existing hardened tool_matrix invocations** — and the library cannot
//! import from a binary. Duplicating the invocations in the lib would create a
//! second source of truth that can drift, which the project's
//! full-factorization doctrine (`feedback_full_factorization.md`) and `0004`
//! ("no second source of truth") forbid. So `.5.1` *moves* them here, exactly
//! as `DIFFERENTIAL-SIMULATION.3a` moved the differential-harness helpers into
//! [`crate::diff_sim`] so the binary could `use anvil::diff_sim::{…}`.
//!
//! This is a pure, behavior-preserving extraction: `src/bin/tool_matrix.rs`
//! now `use`s these symbols instead of defining them, the serialized
//! [`ToolInvocation`] JSON shape is byte-for-byte unchanged (so banked matrix
//! reports and `--resume` checkpoints stay valid), and the matrix's own unit
//! tests plus the snapshot guard prove no drift. The default `anvil` build and
//! the `--artifact dut` byte-identical contract are untouched.
//!
//! ## Scope of `.5.1` vs `.5.2` / `.5.3`
//!
//! `.5.1` (this leaf) lands the *invocation primitives* only. The higher-level
//! `validate(seed, knobs, tools)` orchestration — generate into a sandboxed
//! temp dir, run these tools, ram-guard, audit-log the reproducible command
//! line — is `.5.2`, and the `minimize` delta-debugger is `.5.3`. Both build
//! on this surface; neither is present yet.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Yosys synthesis mode: keep the current no-ABC path, run the warning-clean
/// ABC-enabled harness path, or run both. Carries `clap::ValueEnum` so the
/// `tool_matrix` CLI can still parse `--yosys-mode` directly against this
/// (now library-owned) type.
#[derive(clap::ValueEnum, Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum YosysMode {
    WithoutAbc,
    WithAbc,
    Both,
}

/// The stable slug recorded in matrix reports and `--resume` checkpoints for a
/// given [`YosysMode`]. Kept here beside the enum so the wire form has one
/// owner.
pub fn yosys_mode_slug(mode: YosysMode) -> &'static str {
    match mode {
        YosysMode::WithoutAbc => "without-abc",
        YosysMode::WithAbc => "with-abc",
        YosysMode::Both => "both",
    }
}

/// One external-tool run: the tool label, the exact argv (including the
/// binary, so the command line is reproducible), the pass/fail verdict
/// (`success` folds in warning-as-failure), the exit code, optional captured
/// log file names, and the first warning/error string when not clean. This is
/// the structured per-tool report row both `tool_matrix` and the agent
/// `validate` tool return; its serde shape is a stable wire contract.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInvocation {
    pub tool: String,
    pub argv: Vec<String>,
    pub success: bool,
    pub exit_code: Option<i32>,
    pub stdout_log: Option<String>,
    pub stderr_log: Option<String>,
    pub error: Option<String>,
}

/// Run `verilator --lint-only` on a single emitted module.
pub fn run_verilator(
    bin: &str,
    out_dir: &Path,
    sv_path: &Path,
    stem: &str,
) -> Result<ToolInvocation> {
    run_tool(
        "verilator",
        bin,
        vec!["--lint-only".to_string(), sv_path.display().to_string()],
        out_dir,
        stem,
    )
}

/// Run `verilator --lint-only --top-module <top>` on a multi-file design.
pub fn run_verilator_design(
    bin: &str,
    out_dir: &Path,
    sv_paths: &[PathBuf],
    top: &str,
) -> Result<ToolInvocation> {
    let mut argv = vec![
        "--lint-only".to_string(),
        "--top-module".to_string(),
        top.to_string(),
    ];
    argv.extend(sv_paths.iter().map(|path| path.display().to_string()));
    run_tool("verilator", bin, argv, out_dir, top)
}

/// Run `iverilog -g2012` compile/elaborate on a single emitted module,
/// removing the produced `.vvp` on success (acceptance check only — no
/// testbench is run).
pub fn run_iverilog_compile(
    bin: &str,
    out_dir: &Path,
    sv_path: &Path,
    stem: &str,
) -> Result<ToolInvocation> {
    let (argv, output) = iverilog_module_argv(out_dir, sv_path, stem);
    let invocation = run_tool("iverilog-compile", bin, argv, out_dir, stem)?;
    if invocation.success {
        let _ = std::fs::remove_file(output);
    }
    Ok(invocation)
}

/// Run `iverilog -g2012 -s <top>` compile/elaborate on a multi-file design,
/// removing the produced `.vvp` on success.
pub fn run_iverilog_compile_design(
    bin: &str,
    out_dir: &Path,
    sv_paths: &[PathBuf],
    top: &str,
) -> Result<ToolInvocation> {
    let (argv, output) = iverilog_design_argv(out_dir, sv_paths, top);
    let invocation = run_tool("iverilog-compile", bin, argv, out_dir, top)?;
    if invocation.success {
        let _ = std::fs::remove_file(output);
    }
    Ok(invocation)
}

/// Build the `iverilog -g2012 -o <stem>.iverilog.vvp <sv>` argv for a single
/// module and return it with the output `.vvp` path.
pub fn iverilog_module_argv(out_dir: &Path, sv_path: &Path, stem: &str) -> (Vec<String>, PathBuf) {
    let output = out_dir.join(format!("{stem}.iverilog.vvp"));
    (
        vec![
            "-g2012".to_string(),
            "-o".to_string(),
            output.display().to_string(),
            sv_path.display().to_string(),
        ],
        output,
    )
}

/// Build the `iverilog -g2012 -s <top> -o <top>.iverilog.vvp <files...>` argv
/// for a multi-file design and return it with the output `.vvp` path.
pub fn iverilog_design_argv(
    out_dir: &Path,
    sv_paths: &[PathBuf],
    top: &str,
) -> (Vec<String>, PathBuf) {
    let output = out_dir.join(format!("{top}.iverilog.vvp"));
    let mut argv = vec![
        "-g2012".to_string(),
        "-s".to_string(),
        top.to_string(),
        "-o".to_string(),
        output.display().to_string(),
    ];
    argv.extend(sv_paths.iter().map(|path| path.display().to_string()));
    (argv, output)
}

/// Run the Yosys synthesis acceptance script(s) for `mode` on a single module.
pub fn run_yosys(
    mode: YosysMode,
    bin: &str,
    out_dir: &Path,
    sv_path: &Path,
    stem: &str,
) -> Result<Vec<ToolInvocation>> {
    let mut invocations = Vec::new();
    for (tool_label, script) in yosys_invocations(mode, sv_path) {
        invocations.push(run_tool(
            tool_label,
            bin,
            vec!["-p".to_string(), script],
            out_dir,
            stem,
        )?);
    }
    Ok(invocations)
}

/// Run the Yosys synthesis acceptance script(s) for `mode` on a multi-file
/// design.
pub fn run_yosys_design(
    mode: YosysMode,
    bin: &str,
    out_dir: &Path,
    sv_paths: &[PathBuf],
    top: &str,
) -> Result<Vec<ToolInvocation>> {
    let mut invocations = Vec::new();
    for (tool_label, script) in yosys_design_invocations(mode, sv_paths, top) {
        invocations.push(run_tool(
            tool_label,
            bin,
            vec!["-p".to_string(), script],
            out_dir,
            top,
        )?);
    }
    Ok(invocations)
}

/// The Yosys `-p` script(s) for a single module under `mode`. `without-abc`
/// is the stable baseline (`synth -noabc; stat`); `with-abc` is the
/// warning-clean ABC path (`synth -noabc; abc -fast; opt -fast; stat; check`).
pub fn yosys_invocations(mode: YosysMode, sv_path: &Path) -> Vec<(&'static str, String)> {
    let escaped = escape_for_double_quotes(sv_path);
    match mode {
        YosysMode::WithoutAbc => vec![(
            "yosys-without-abc",
            format!("read_verilog -sv \"{escaped}\"; synth -noabc; stat"),
        )],
        YosysMode::WithAbc => vec![(
            "yosys-with-abc",
            format!(
                "read_verilog -sv \"{escaped}\"; synth -noabc; abc -fast; opt -fast; stat; check"
            ),
        )],
        YosysMode::Both => vec![
            (
                "yosys-without-abc",
                format!("read_verilog -sv \"{escaped}\"; synth -noabc; stat"),
            ),
            (
                "yosys-with-abc",
                format!(
                    "read_verilog -sv \"{escaped}\"; synth -noabc; abc -fast; opt -fast; stat; check"
                ),
            ),
        ],
    }
}

/// The Yosys `-p` script(s) for a multi-file design under `mode`, pinned to
/// `-top <top>` and closed with `check`.
pub fn yosys_design_invocations(
    mode: YosysMode,
    sv_paths: &[PathBuf],
    top: &str,
) -> Vec<(&'static str, String)> {
    let escaped_files = escape_paths_for_double_quotes(sv_paths);
    match mode {
        YosysMode::WithoutAbc => vec![(
            "yosys-without-abc",
            format!(
                "read_verilog -sv {escaped_files}; synth -top {top} -noabc; stat; check"
            ),
        )],
        YosysMode::WithAbc => vec![(
            "yosys-with-abc",
            format!(
                "read_verilog -sv {escaped_files}; synth -top {top} -noabc; abc -fast; opt -fast; stat; check"
            ),
        )],
        YosysMode::Both => vec![
            (
                "yosys-without-abc",
                format!(
                    "read_verilog -sv {escaped_files}; synth -top {top} -noabc; stat; check"
                ),
            ),
            (
                "yosys-with-abc",
                format!(
                    "read_verilog -sv {escaped_files}; synth -top {top} -noabc; abc -fast; opt -fast; stat; check"
                ),
            ),
        ],
    }
}

/// Spawn one external tool, capture stdout/stderr, fold warning-as-failure
/// into `success`, and persist the streams as `.log` sidecars in `out_dir`
/// when the run is not clean. A spawn failure (tool absent) is reported as a
/// non-`success` [`ToolInvocation`] with the OS error, never a panic — so a
/// missing tool degrades gracefully.
pub fn run_tool(
    tool_name: &str,
    binary: &str,
    argv: Vec<String>,
    out_dir: &Path,
    stem: &str,
) -> Result<ToolInvocation> {
    let output = Command::new(binary).args(&argv).output();
    match output {
        Ok(output) => {
            let warning = first_tool_warning(
                tool_name,
                String::from_utf8_lossy(&output.stdout).as_ref(),
                String::from_utf8_lossy(&output.stderr).as_ref(),
            );
            let success = output.status.success() && warning.is_none();
            let stdout_log = write_tool_log_if_needed(
                out_dir,
                stem,
                tool_name,
                "stdout",
                &output.stdout,
                !success,
            )?;
            let stderr_log = write_tool_log_if_needed(
                out_dir,
                stem,
                tool_name,
                "stderr",
                &output.stderr,
                !success,
            )?;
            Ok(ToolInvocation {
                tool: tool_name.to_string(),
                argv: std::iter::once(binary.to_string()).chain(argv).collect(),
                success,
                exit_code: output.status.code(),
                stdout_log,
                stderr_log,
                error: warning,
            })
        }
        Err(err) => Ok(ToolInvocation {
            tool: tool_name.to_string(),
            argv: std::iter::once(binary.to_string()).chain(argv).collect(),
            success: false,
            exit_code: None,
            stdout_log: None,
            stderr_log: None,
            error: Some(err.to_string()),
        }),
    }
}

/// The first warning line a tool emitted, per-tool. A warning is a *failure*
/// for ANVIL's signoff bar (the generated RTL must be boringly clean), so
/// `run_tool` folds a non-`None` result into `success = false`. Verilator uses
/// `%Warning-…`; Yosys uses `Warning:` / `: Warning:`; iverilog is matched
/// case-insensitively on `warning:`.
pub fn first_tool_warning(tool_name: &str, stdout: &str, stderr: &str) -> Option<String> {
    match tool_name {
        "verilator" => stdout
            .lines()
            .chain(stderr.lines())
            .map(str::trim_start)
            .find(|line| line.starts_with("%Warning-"))
            .map(ToOwned::to_owned),
        tool_name if tool_name.starts_with("yosys") => stdout
            .lines()
            .chain(stderr.lines())
            .map(str::trim_start)
            .find(|line| line.starts_with("Warning:") || line.contains(": Warning:"))
            .map(ToOwned::to_owned),
        "iverilog-compile" => stdout
            .lines()
            .chain(stderr.lines())
            .map(str::trim_start)
            .find(|line| line.to_ascii_lowercase().contains("warning:"))
            .map(ToOwned::to_owned),
        _ => None,
    }
}

/// Persist a captured stream as `<stem>.<tool>.<stream>.log` in `out_dir`,
/// returning the file name. Skipped (returns `None`) when the stream is empty
/// and the run did not fail; on failure a clean run still writes the (possibly
/// empty) log so the failure is inspectable.
fn write_tool_log_if_needed(
    out_dir: &Path,
    stem: &str,
    tool_name: &str,
    stream: &str,
    bytes: &[u8],
    always_write_on_failure: bool,
) -> Result<Option<String>> {
    if bytes.is_empty() && !always_write_on_failure {
        return Ok(None);
    }
    let file_name = format!("{stem}.{tool_name}.{stream}.log");
    let path = out_dir.join(&file_name);
    std::fs::write(&path, bytes).with_context(|| format!("write {}", path.display()))?;
    Ok(Some(file_name))
}

/// Escape a path for embedding inside a double-quoted Yosys `-p` script
/// argument (backslash and double-quote).
pub fn escape_for_double_quotes(path: &Path) -> String {
    path.display()
        .to_string()
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
}

/// Join several paths as space-separated double-quoted, escaped tokens for a
/// Yosys `read_verilog` script.
pub fn escape_paths_for_double_quotes(paths: &[PathBuf]) -> String {
    paths
        .iter()
        .map(|path| format!("\"{}\"", escape_for_double_quotes(path)))
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn yosys_mode_expands_to_expected_invocations() {
        let path = Path::new("/tmp/example.sv");

        let without = yosys_invocations(YosysMode::WithoutAbc, path);
        assert_eq!(without.len(), 1);
        assert_eq!(without[0].0, "yosys-without-abc");
        assert!(without[0].1.contains("synth -noabc; stat"));

        let with = yosys_invocations(YosysMode::WithAbc, path);
        assert_eq!(with.len(), 1);
        assert_eq!(with[0].0, "yosys-with-abc");
        assert!(with[0]
            .1
            .contains("synth -noabc; abc -fast; opt -fast; stat; check"));
        assert!(with[0].1.contains("abc -fast"));

        let both = yosys_invocations(YosysMode::Both, path);
        assert_eq!(both.len(), 2);
        assert_eq!(both[0].0, "yosys-without-abc");
        assert_eq!(both[1].0, "yosys-with-abc");
    }

    #[test]
    fn hierarchy_yosys_mode_expands_to_expected_invocations() {
        let paths = vec![PathBuf::from("/tmp/a.sv"), PathBuf::from("/tmp/b.sv")];

        let without = yosys_design_invocations(YosysMode::WithoutAbc, &paths, "top_mod");
        assert_eq!(without.len(), 1);
        assert!(without[0].1.contains("read_verilog -sv"));
        assert!(without[0].1.contains("\"/tmp/a.sv\" \"/tmp/b.sv\""));
        assert!(without[0]
            .1
            .contains("synth -top top_mod -noabc; stat; check"));

        let with = yosys_design_invocations(YosysMode::WithAbc, &paths, "top_mod");
        assert_eq!(with.len(), 1);
        assert!(with[0]
            .1
            .contains("synth -top top_mod -noabc; abc -fast; opt -fast; stat; check"));
    }

    #[test]
    fn iverilog_compile_invocations_use_sv2012_and_design_top() {
        let out_dir = PathBuf::from("/tmp/anvil-iverilog-argv");
        let sv_path = PathBuf::from("/tmp/anvil-iverilog-argv/mod.sv");

        let (module_argv, module_output) = iverilog_module_argv(&out_dir, &sv_path, "mod");
        assert_eq!(
            module_argv,
            vec![
                "-g2012",
                "-o",
                "/tmp/anvil-iverilog-argv/mod.iverilog.vvp",
                "/tmp/anvil-iverilog-argv/mod.sv"
            ]
        );
        assert_eq!(
            module_output,
            PathBuf::from("/tmp/anvil-iverilog-argv/mod.iverilog.vvp")
        );

        let paths = vec![
            PathBuf::from("/tmp/anvil-iverilog-argv/leaf.sv"),
            PathBuf::from("/tmp/anvil-iverilog-argv/top.sv"),
        ];
        let (design_argv, design_output) = iverilog_design_argv(&out_dir, &paths, "top_mod");
        assert_eq!(
            design_argv,
            vec![
                "-g2012",
                "-s",
                "top_mod",
                "-o",
                "/tmp/anvil-iverilog-argv/top_mod.iverilog.vvp",
                "/tmp/anvil-iverilog-argv/leaf.sv",
                "/tmp/anvil-iverilog-argv/top.sv"
            ]
        );
        assert_eq!(
            design_output,
            PathBuf::from("/tmp/anvil-iverilog-argv/top_mod.iverilog.vvp")
        );
    }

    #[test]
    fn iverilog_compile_warning_detection_is_case_insensitive() {
        assert_eq!(
            first_tool_warning("iverilog-compile", "", "/tmp/m.sv:2: warning: example").as_deref(),
            Some("/tmp/m.sv:2: warning: example")
        );
        assert_eq!(
            first_tool_warning("iverilog-compile", "WARNING: noisy stdout", "").as_deref(),
            Some("WARNING: noisy stdout")
        );
        assert!(first_tool_warning("iverilog-compile", "clean", "clean").is_none());
    }

    #[test]
    fn yosys_mode_slug_round_trips_each_variant() {
        assert_eq!(yosys_mode_slug(YosysMode::WithoutAbc), "without-abc");
        assert_eq!(yosys_mode_slug(YosysMode::WithAbc), "with-abc");
        assert_eq!(yosys_mode_slug(YosysMode::Both), "both");
    }

    #[test]
    fn verilator_and_yosys_warnings_fail_the_clean_check() {
        // A `%Warning-` line is a Verilator failure even with exit 0.
        assert!(first_tool_warning("verilator", "%Warning-WIDTH: trunc", "").is_some());
        assert!(first_tool_warning("verilator", "all clean", "").is_none());
        // Yosys matches `Warning:` and `: Warning:`.
        assert!(first_tool_warning("yosys-without-abc", "Warning: foo", "").is_some());
        assert!(first_tool_warning("yosys-with-abc", "x.v:1: Warning: bar", "").is_some());
        assert!(first_tool_warning("yosys-without-abc", "Number of cells: 3", "").is_none());
    }

    #[test]
    fn escape_paths_quotes_and_joins() {
        let paths = vec![PathBuf::from("/a b/x.sv"), PathBuf::from("/c/y.sv")];
        assert_eq!(
            escape_paths_for_double_quotes(&paths),
            "\"/a b/x.sv\" \"/c/y.sv\""
        );
    }
}
