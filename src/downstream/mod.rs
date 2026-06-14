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

use crate::config::Config;
use crate::introspect::content_run_id;
use crate::mem_guard::{AbortReason, MemGuard, MemLimits};
use crate::{emit, Generator};
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

// ---------------------------------------------------------------------------
// AGENT-INTROSPECTION-MCP.5.2 — the controlled `validate` orchestration.
//
// Generate the DUT artifact for `(seed, knobs)` deterministically into a fresh
// per-run *sandbox* directory, run the selected vetted acceptance tools (the
// `run_*` primitives above), optionally decline before a spawn when a memory
// ceiling is crossed, and return a structured report whose every tool entry
// carries its exact reproducible argv (which the MCP layer audit-logs).
// Guardrails per decision `0004`: no arbitrary shell (a fixed tool allow-list,
// fixed binary names), no arbitrary filesystem write (the sandbox root is fixed
// by the caller, never the agent), and a ram-guard decline path.
// ---------------------------------------------------------------------------

/// One acceptance tool the agent may ask `validate` to run. A fixed
/// allow-list: there is no arbitrary-command tool and no agent-supplied binary
/// path — the binaries are the standard names (decision `0004`: "only fixed,
/// vetted tool invocations").
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AcceptanceTool {
    Verilator,
    Yosys,
    Iverilog,
}

impl AcceptanceTool {
    /// Parse the agent-facing tool name. Returns `None` for anything off the
    /// allow-list — the caller turns that into a clean error, never a spawn.
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "verilator" => Some(Self::Verilator),
            "yosys" => Some(Self::Yosys),
            "iverilog" => Some(Self::Iverilog),
            _ => None,
        }
    }

    /// The fixed binary name. Not agent-overridable.
    pub fn binary(self) -> &'static str {
        match self {
            Self::Verilator => "verilator",
            Self::Yosys => "yosys",
            Self::Iverilog => "iverilog",
        }
    }
}

/// How `validate` should run. The agent controls *which vetted tools* run and
/// the Yosys mode; it controls neither the sandbox location (fixed to a tmp
/// scope by the caller) nor the tool binaries.
#[derive(Debug, Clone)]
pub struct ValidateOptions {
    /// The vetted tools to run, in order. Empty = generate + sandbox only
    /// (a no-tool smoke; useful on hosts without the tools installed).
    pub tools: Vec<AcceptanceTool>,
    /// Yosys synthesis mode when [`AcceptanceTool::Yosys`] is selected.
    pub yosys_mode: YosysMode,
    /// Abort ceilings checked *before each tool spawn* (decline-to-start-more).
    /// Default off (sentinel `0`), mirroring `mem_guard`. The host-%-used axis
    /// is the meaningful one here — it declines a new heavy tool when the host
    /// is already in the danger zone; a child tool's own RSS balloon is the job
    /// of the external `scripts/ram_guard.sh` wrapper.
    pub mem_limits: MemLimits,
    /// Sandbox root under which a fresh per-run subdirectory is created. The
    /// MCP adapter fixes this to the OS temp dir; tests pass a controlled path.
    pub sandbox_root: PathBuf,
    /// Keep the sandbox directory after the run (default: remove it).
    pub keep_sandbox: bool,
}

impl Default for ValidateOptions {
    fn default() -> Self {
        Self {
            tools: vec![AcceptanceTool::Verilator, AcceptanceTool::Yosys],
            yosys_mode: YosysMode::WithoutAbc,
            mem_limits: MemLimits {
                max_rss_mb: 0,
                ram_abort_pct: 0,
            },
            sandbox_root: std::env::temp_dir(),
            keep_sandbox: false,
        }
    }
}

/// The structured result of a `validate` run: the deterministic `run_id`, the
/// artifact descriptor, every tool invocation (each carrying its exact
/// reproducible argv), the overall verdict, and any decline reason.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidateReport {
    pub run_id: String,
    pub lane: String,
    pub kind: String,
    pub top: String,
    /// The sandbox directory the artifact was written to (and tools ran in).
    pub sandbox: String,
    /// One entry per tool invocation (Yosys `both` yields two).
    pub tools: Vec<ToolInvocation>,
    /// `true` iff no tool was declined and every run tool succeeded.
    pub ok: bool,
    /// Set when the run stopped before completing all tools (e.g. the memory
    /// guard tripped before a spawn). `None` on a complete run.
    pub declined: Option<String>,
}

/// Validate the DUT artifact for `(seed, cfg)` against the selected downstream
/// tools, sandboxed and (optionally) ram-guarded.
///
/// The artifact is generated deterministically into a fresh per-run
/// subdirectory of `opts.sandbox_root` — never an agent-supplied path — so
/// there is no arbitrary filesystem write; only the fixed `run_*` invocations
/// run, with fixed binary names (no arbitrary shell); and the memory guard can
/// decline-to-spawn before the host enters the danger zone. The returned
/// [`ValidateReport`] carries the reproducible `(run_id, argv)` for every call.
pub fn validate(seed: u64, cfg: &Config, opts: &ValidateOptions) -> Result<ValidateReport> {
    let run_id = content_run_id("dut", seed, cfg);

    // Generate deterministically. Mirrors the CLI / MCP single-artifact
    // dispatch: a hierarchy range ⇒ a design, else a leaf module.
    let mut generator = Generator::new(cfg.clone());
    let (kind, top, sv) = if cfg.effective_hierarchy_depth_range().is_some() {
        let design = generator.generate_design();
        (
            "design".to_string(),
            design.top.clone(),
            emit::to_sv_design(&design),
        )
    } else {
        let module = generator.generate_module();
        (
            "module".to_string(),
            module.name.clone(),
            emit::to_sv(&module),
        )
    };

    // Fresh per-run sandbox directory under the caller-fixed root.
    let sandbox = opts.sandbox_root.join(format!("anvil-validate-{run_id}"));
    std::fs::create_dir_all(&sandbox)
        .with_context(|| format!("create sandbox {}", sandbox.display()))?;
    let sv_path = sandbox.join(format!("{top}.sv"));
    std::fs::write(&sv_path, &sv).with_context(|| format!("write {}", sv_path.display()))?;

    let guard = MemGuard::from_limits(opts.mem_limits);
    let is_design = kind == "design";
    let mut tools = Vec::new();
    let mut declined = None;

    for tool in &opts.tools {
        if let Some(reason) = guard.check() {
            declined = Some(decline_message(&reason));
            break;
        }
        match tool {
            AcceptanceTool::Verilator => {
                tools.push(if is_design {
                    run_verilator_design(
                        tool.binary(),
                        &sandbox,
                        std::slice::from_ref(&sv_path),
                        &top,
                    )?
                } else {
                    run_verilator(tool.binary(), &sandbox, &sv_path, &top)?
                });
            }
            AcceptanceTool::Yosys => {
                let invs = if is_design {
                    run_yosys_design(
                        opts.yosys_mode,
                        tool.binary(),
                        &sandbox,
                        std::slice::from_ref(&sv_path),
                        &top,
                    )?
                } else {
                    run_yosys(opts.yosys_mode, tool.binary(), &sandbox, &sv_path, &top)?
                };
                tools.extend(invs);
            }
            AcceptanceTool::Iverilog => {
                tools.push(if is_design {
                    run_iverilog_compile_design(
                        tool.binary(),
                        &sandbox,
                        std::slice::from_ref(&sv_path),
                        &top,
                    )?
                } else {
                    run_iverilog_compile(tool.binary(), &sandbox, &sv_path, &top)?
                });
            }
        }
    }

    let ok = declined.is_none() && tools.iter().all(|t| t.success);
    let sandbox_str = sandbox.display().to_string();
    if !opts.keep_sandbox {
        let _ = std::fs::remove_dir_all(&sandbox);
    }

    Ok(ValidateReport {
        run_id,
        lane: "dut".to_string(),
        kind,
        top,
        sandbox: sandbox_str,
        tools,
        ok,
        declined,
    })
}

fn decline_message(reason: &AbortReason) -> String {
    format!("memory guard declined to spawn the next tool: {reason}")
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

    // ----- AGENT-INTROSPECTION-MCP.5.2: `validate` -----

    fn test_root(tag: &str) -> PathBuf {
        std::env::temp_dir().join(format!("anvil-validate-test-{tag}"))
    }

    fn no_tools(tag: &str) -> ValidateOptions {
        ValidateOptions {
            tools: vec![],
            sandbox_root: test_root(tag),
            ..Default::default()
        }
    }

    #[test]
    fn acceptance_tool_allow_list_rejects_unknown_names() {
        assert_eq!(
            AcceptanceTool::from_name("verilator"),
            Some(AcceptanceTool::Verilator)
        );
        assert_eq!(
            AcceptanceTool::from_name("yosys"),
            Some(AcceptanceTool::Yosys)
        );
        assert_eq!(
            AcceptanceTool::from_name("iverilog"),
            Some(AcceptanceTool::Iverilog)
        );
        // Anything off the allow-list is rejected — never a spawn.
        assert_eq!(AcceptanceTool::from_name("rm -rf /"), None);
        assert_eq!(AcceptanceTool::from_name("bash"), None);
        assert_eq!(AcceptanceTool::from_name(""), None);
        // Binary names are the fixed standard ones.
        assert_eq!(AcceptanceTool::Verilator.binary(), "verilator");
        assert_eq!(AcceptanceTool::Yosys.binary(), "yosys");
        assert_eq!(AcceptanceTool::Iverilog.binary(), "iverilog");
    }

    #[test]
    fn validate_no_tools_generates_into_sandbox_and_cleans_up() {
        let cfg = Config {
            seed: 7,
            ..Config::default()
        };
        let report = validate(7, &cfg, &no_tools("notools")).unwrap();

        assert_eq!(report.lane, "dut");
        assert_eq!(report.kind, "module");
        assert!(report.tools.is_empty());
        assert!(report.declined.is_none());
        // No tool ran and none declined ⇒ the run is vacuously ok.
        assert!(report.ok);
        // The run carries the SAME content address generate/introspect use.
        assert_eq!(report.run_id, content_run_id("dut", 7, &cfg));
        // Default keep_sandbox = false ⇒ the sandbox is removed.
        assert!(!Path::new(&report.sandbox).exists());
    }

    #[test]
    fn validate_keeps_sandbox_and_writes_sv_when_requested() {
        let cfg = Config {
            seed: 11,
            ..Config::default()
        };
        let opts = ValidateOptions {
            keep_sandbox: true,
            ..no_tools("keep")
        };
        let report = validate(11, &cfg, &opts).unwrap();
        let dir = Path::new(&report.sandbox);
        assert!(dir.exists(), "kept sandbox must remain");
        let sv = dir.join(format!("{}.sv", report.top));
        assert!(sv.exists(), "the emitted .sv must be in the sandbox");
        let text = std::fs::read_to_string(&sv).unwrap();
        assert!(text.contains("module "));
        // Clean up the kept sandbox so the test leaves no residue.
        std::fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn validate_design_artifact_is_recognised() {
        let cfg = Config {
            seed: 3,
            hierarchy_depth: 1,
            num_leaf_modules: 2,
            num_child_instances: 2,
            ..Config::default()
        };
        let opts = ValidateOptions {
            keep_sandbox: true,
            ..no_tools("design")
        };
        let report = validate(3, &cfg, &opts).unwrap();
        assert_eq!(report.kind, "design");
        let dir = Path::new(&report.sandbox);
        assert!(dir.join(format!("{}.sv", report.top)).exists());
        std::fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn validate_memory_guard_declines_before_spawning_any_tool() {
        // A 1 MiB RSS ceiling trips immediately (this process is far larger),
        // so validate declines before spawning the first tool — proving the
        // decline-to-start-more guard runs *before* any spawn, with no tool
        // dependency. Best-effort: on a host where RSS is unreadable the guard
        // never trips (mem_guard's documented policy), so skip the assertion.
        if crate::mem_guard::read_process_rss_mb().is_none() {
            return;
        }
        let cfg = Config {
            seed: 5,
            ..Config::default()
        };
        let opts = ValidateOptions {
            tools: vec![AcceptanceTool::Verilator, AcceptanceTool::Yosys],
            mem_limits: MemLimits {
                max_rss_mb: 1,
                ram_abort_pct: 0,
            },
            sandbox_root: test_root("decline"),
            keep_sandbox: false,
            yosys_mode: YosysMode::WithoutAbc,
        };
        let report = validate(5, &cfg, &opts).unwrap();
        assert!(report.declined.is_some(), "guard must decline");
        assert!(report.tools.is_empty(), "no tool may spawn after a decline");
        assert!(!report.ok);
    }

    #[test]
    #[ignore = "requires verilator + yosys on PATH"]
    fn validate_dut_seed_is_downstream_clean_end_to_end() {
        let cfg = Config {
            seed: 42,
            ..Config::default()
        };
        let opts = ValidateOptions {
            tools: vec![AcceptanceTool::Verilator, AcceptanceTool::Yosys],
            sandbox_root: test_root("e2e"),
            ..Default::default()
        };
        let report = validate(42, &cfg, &opts).unwrap();
        assert!(report.declined.is_none());
        assert!(
            report.ok,
            "ANVIL DUT output must be downstream-clean by construction: {report:?}"
        );
        assert!(report.tools.iter().any(|t| t.tool == "verilator"));
        assert!(report.tools.iter().any(|t| t.tool.starts_with("yosys")));
    }
}
