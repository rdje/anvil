//! Read-only in-process MCP server (`AGENT-INTROSPECTION-MCP.4`).
//!
//! A thin, dependency-light **JSON-RPC 2.0** dispatcher that exposes ANVIL's
//! deterministic generator to an AI agent over the MCP stdio transport
//! (newline-delimited JSON, one message per line). It is the read-only half
//! of the agent lane: it serves construction-truth, it does not run external
//! tools (that is `.5`) and it never mutates generator state.
//!
//! Design (decision `0004`, schema `docs/AGENT_INTROSPECTION_SCHEMA.md`):
//!
//! - **Beside the core.** This lives in its own module + `anvil-mcp` bin; the
//!   default `anvil` build and the `--artifact dut` byte-identical contract
//!   are unaffected. The generator kernel learns nothing about MCP.
//! - **Pure dispatch, testable without a process.** [`McpServer::handle`]
//!   maps a JSON-RPC request `Value` to an optional response `Value`
//!   (notifications return `None`). The `anvil-mcp` bin is a thin stdio loop
//!   over [`McpServer::handle_line`]. So the whole protocol surface is unit
//!   tested in-process.
//! - **Determinism → content-addressed cache.** Artifacts are pure functions
//!   of `(seed, knobs, lane, version)`, so `generate` caches by the
//!   introspection document's content-addressed `run_id` and `resources/read`
//!   serves the cached `.sv` / introspection document back. No nonces.
//! - **Pure/safe tools only.** `generate`, `introspect`, `dump_config` — all
//!   side-effect-free. No filesystem writes, no shell, no external tools.
//!
//! Scope: `.4` landed the DUT lane, the three pure tools, resources over the
//! cache, and two static catalogs (`knobs`, `lanes`). `.5` adds the controlled
//! tools that *do* run external tools, sandboxed: `validate` (`.5.2`) and the
//! `minimize` delta-debugger (`.5.3`), both over [`crate::downstream`] and
//! audit-logged to the `anvil://audit/log` resource.

use crate::config::Config;
use crate::downstream::{self, AcceptanceTool, MinimizeOptions, ValidateOptions, YosysMode};
use crate::introspect;
use crate::{emit, Generator};
use serde_json::{json, Value};
use std::collections::BTreeMap;

/// MCP protocol version this server speaks (the stable stdio revision used by
/// Claude Code / Cursor at the time of writing).
pub const PROTOCOL_VERSION: &str = "2024-11-05";

/// JSON-RPC: method not found.
const METHOD_NOT_FOUND: i64 = -32601;
/// JSON-RPC: invalid params.
const INVALID_PARAMS: i64 = -32602;

/// One cached artifact, keyed by its content-addressed `run_id`.
#[derive(Debug, Clone)]
struct CachedArtifact {
    kind: String,
    top: String,
    sv: String,
    /// The full introspection document (schema-conformant), as a JSON value.
    document: Value,
}

/// The read-only MCP server: a JSON-RPC dispatcher plus a content-addressed
/// artifact cache. Hold one per connection (the `anvil-mcp` bin owns one).
#[derive(Debug, Default)]
pub struct McpServer {
    cache: BTreeMap<String, CachedArtifact>,
    initialized: bool,
    /// Append-only audit trail of controlled `validate` calls
    /// (`AGENT-INTROSPECTION-MCP.5.2`): one record per call with the
    /// reproducible `(run_id, seed)` and the exact command line of every tool
    /// spawned. Exposed read-only as the `anvil://audit/log` resource.
    audit: Vec<Value>,
}

impl McpServer {
    pub fn new() -> Self {
        Self::default()
    }

    /// Drive one line of the stdio transport: parse a JSON-RPC message,
    /// dispatch it, and return the serialized response line (or `None` for a
    /// notification / blank line). A parse error yields a JSON-RPC parse-error
    /// response with a null id, per JSON-RPC 2.0.
    pub fn handle_line(&mut self, line: &str) -> Option<String> {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            return None;
        }
        let req: Value = match serde_json::from_str(trimmed) {
            Ok(v) => v,
            Err(e) => {
                return Some(
                    json!({
                        "jsonrpc": "2.0",
                        "id": Value::Null,
                        "error": { "code": -32700, "message": format!("parse error: {e}") }
                    })
                    .to_string(),
                )
            }
        };
        self.handle(&req).map(|resp| resp.to_string())
    }

    /// Pure JSON-RPC dispatch. Returns `Some(response)` for a request (a
    /// message carrying an `id`), `None` for a notification.
    pub fn handle(&mut self, req: &Value) -> Option<Value> {
        let method = req.get("method").and_then(Value::as_str).unwrap_or("");
        let id = req.get("id").cloned();
        let params = req.get("params").cloned().unwrap_or(Value::Null);

        // Notifications (no id) get no response; we still process the one we
        // care about (`notifications/initialized`).
        if id.is_none() {
            if method == "notifications/initialized" {
                self.initialized = true;
            }
            return None;
        }
        let id = id.unwrap();

        match method {
            "initialize" => Some(self.on_initialize(id)),
            "ping" => Some(ok(id, json!({}))),
            "tools/list" => Some(ok(id, self.tools_list())),
            "tools/call" => Some(self.tools_call(id, &params)),
            "resources/list" => Some(ok(id, self.resources_list())),
            "resources/read" => Some(self.resources_read(id, &params)),
            "prompts/list" => Some(ok(id, prompts_list())),
            "prompts/get" => Some(prompts_get(id, &params)),
            other => Some(err(
                id,
                METHOD_NOT_FOUND,
                &format!("method not found: {other}"),
            )),
        }
    }

    fn on_initialize(&mut self, id: Value) -> Value {
        ok(
            id,
            json!({
                "protocolVersion": PROTOCOL_VERSION,
                "capabilities": { "tools": {}, "resources": {}, "prompts": {} },
                "serverInfo": {
                    "name": "anvil-mcp",
                    "version": introspect::anvil_version(),
                },
                "instructions":
                    "ANVIL agent-introspection. Pure tools: generate, introspect, \
                     dump_config (construction-truth derived from existing \
                     metrics/config; no side effects). Controlled tools: validate \
                     runs the vetted downstream tools (verilator / yosys / \
                     iverilog) on a (seed, knobs) artifact inside a sandboxed temp \
                     dir — a fixed allow-list, no arbitrary shell; minimize \
                     delta-debugs a failing (seed, knobs) to a smaller reproducer \
                     using validate as the oracle (deterministic, budget-bounded, \
                     seed held fixed). Both audit-log each call (see the \
                     anvil://audit/log resource). Artifacts and static catalogs \
                     are exposed as resources. Workflow prompts (prompts/list, \
                     prompts/get) package the agent loops: find_downstream_bug, \
                     close_coverage_gap, minimize_reproducer, triage_tool_failures, \
                     explain_artifact — each renders an ordered chain over the tools \
                     above."
            }),
        )
    }

    fn tools_list(&self) -> Value {
        let knob_schema = json!({
            "type": "object",
            "properties": {
                "seed": { "type": "integer", "minimum": 0,
                          "description": "RNG seed (deterministic output)." },
                "config": { "type": "object",
                            "description": "Full effective Config (as emitted by dump_config). Omit for defaults." }
            },
            "additionalProperties": false
        });
        let validate_schema = json!({
            "type": "object",
            "properties": {
                "seed": { "type": "integer", "minimum": 0,
                          "description": "RNG seed (deterministic output)." },
                "config": { "type": "object",
                            "description": "Full effective Config (as emitted by dump_config). Omit for defaults." },
                "tools": {
                    "type": "array",
                    "items": { "type": "string", "enum": ["verilator", "yosys", "iverilog"] },
                    "description": "Vetted downstream tools to run (default: verilator + yosys). \
                                    A fixed allow-list — no arbitrary commands or binary paths."
                },
                "yosys_mode": {
                    "type": "string",
                    "enum": ["without-abc", "with-abc", "both"],
                    "description": "Yosys synthesis mode when yosys is selected (default without-abc)."
                }
            },
            "additionalProperties": false
        });
        let minimize_schema = json!({
            "type": "object",
            "properties": {
                "seed": { "type": "integer", "minimum": 0,
                          "description": "RNG seed (held fixed; it pins the reproducer)." },
                "config": { "type": "object",
                            "description": "Full effective Config (as emitted by dump_config). Omit for defaults." },
                "tools": {
                    "type": "array",
                    "items": { "type": "string", "enum": ["verilator", "yosys", "iverilog"] },
                    "description": "Vetted downstream tools used as the failure oracle \
                                    (default: verilator + yosys). A fixed allow-list."
                },
                "yosys_mode": {
                    "type": "string",
                    "enum": ["without-abc", "with-abc", "both"],
                    "description": "Yosys synthesis mode when yosys is selected (default without-abc)."
                },
                "max_oracle_calls": {
                    "type": "integer", "minimum": 1,
                    "description": "Hard ceiling on validate evaluations (default 200). Bounds the search."
                }
            },
            "additionalProperties": false
        });
        json!({
            "tools": [
                {
                    "name": "generate",
                    "description": "Generate a DUT artifact for (seed, config) and cache it; \
                                    returns its content-addressed run_id and resource URIs.",
                    "inputSchema": knob_schema,
                },
                {
                    "name": "introspect",
                    "description": "Return the versioned introspection document (schema 1.0) for \
                                    (seed, config): config echo + metrics, derived from existing facts.",
                    "inputSchema": knob_schema,
                },
                {
                    "name": "dump_config",
                    "description": "Return the effective Config for (seed, config) after validation.",
                    "inputSchema": knob_schema,
                },
                {
                    "name": "validate",
                    "description": "Generate the (seed, config) DUT artifact into a sandboxed temp dir \
                                    and run the selected vetted downstream tools (verilator/yosys/iverilog) \
                                    on it; returns structured per-tool reports + an overall verdict. \
                                    Audit-logged; no arbitrary shell.",
                    "inputSchema": validate_schema,
                },
                {
                    "name": "minimize",
                    "description": "Delta-debug a failing (seed, config) to a smaller failing reproducer \
                                    using validate as the failure oracle: shrink size bounds and disable \
                                    optional motifs while a downstream tool still rejects the artifact. \
                                    Deterministic, seed held fixed, budget-bounded; audit-logged. \
                                    Reports reproduced_initial=false (and shrinks nothing) when the \
                                    output is downstream-clean.",
                    "inputSchema": minimize_schema,
                },
            ]
        })
    }

    fn tools_call(&mut self, id: Value, params: &Value) -> Value {
        let name = params.get("name").and_then(Value::as_str).unwrap_or("");
        let args = params.get("arguments").cloned().unwrap_or(json!({}));

        let (seed, cfg) = match config_from_args(&args) {
            Ok(pair) => pair,
            Err(e) => return ok(id, tool_error(&e)),
        };

        match name {
            "dump_config" => match serde_json::to_string_pretty(&cfg) {
                Ok(text) => ok(id, tool_text(&text)),
                Err(e) => ok(id, tool_error(&format!("serialize config: {e}"))),
            },
            "introspect" => {
                let (_sv, _kind, _top, doc) = build_artifact(seed, &cfg);
                self.cache_artifact(&doc);
                match serde_json::to_string_pretty(&doc) {
                    Ok(text) => ok(id, tool_text(&text)),
                    Err(e) => ok(id, tool_error(&format!("serialize document: {e}"))),
                }
            }
            "generate" => {
                let (_sv, kind, top, doc) = build_artifact(seed, &cfg);
                let run_id = self.cache_artifact(&doc);
                let summary = json!({
                    "run_id": run_id,
                    "lane": "dut",
                    "kind": kind,
                    "top": top,
                    "resources": {
                        "sv": format!("anvil://artifact/{run_id}/sv"),
                        "introspection": format!("anvil://artifact/{run_id}/introspection"),
                    }
                });
                match serde_json::to_string_pretty(&summary) {
                    Ok(text) => ok(id, tool_text(&text)),
                    Err(e) => ok(id, tool_error(&format!("serialize summary: {e}"))),
                }
            }
            "validate" => match self.run_validate(seed, &cfg, &args) {
                Ok(text) => ok(id, tool_text(&text)),
                Err(e) => ok(id, tool_error(&e)),
            },
            "minimize" => match self.run_minimize(seed, &cfg, &args) {
                Ok(text) => ok(id, tool_text(&text)),
                Err(e) => ok(id, tool_error(&e)),
            },
            other => ok(id, tool_error(&format!("unknown tool: {other}"))),
        }
    }

    /// The controlled `validate` tool (`AGENT-INTROSPECTION-MCP.5.2`): run the
    /// selected vetted downstream tools on the `(seed, cfg)` artifact in a
    /// sandboxed temp dir, audit-log the call, and return the structured
    /// report. The sandbox root and tool binaries are fixed here (the OS temp
    /// dir / the standard names) — the agent controls only *which* vetted tools
    /// run and the Yosys mode; there is no arbitrary-path or arbitrary-command
    /// surface.
    fn run_validate(&mut self, seed: u64, cfg: &Config, args: &Value) -> Result<String, String> {
        let opts = ValidateOptions {
            tools: parse_validate_tools(args)?,
            yosys_mode: parse_yosys_mode_arg(args)?,
            ..ValidateOptions::default()
        };
        let report = downstream::validate(seed, cfg, &opts).map_err(|e| e.to_string())?;

        // Audit-log the reproducible call: the run_id, the seed, and the exact
        // command line of every tool spawned (the verdict too, for triage).
        self.audit.push(json!({
            "tool": "validate",
            "run_id": report.run_id,
            "seed": seed,
            "lane": report.lane,
            "kind": report.kind,
            "top": report.top,
            "commands": report
                .tools
                .iter()
                .map(|t| t.argv.join(" "))
                .collect::<Vec<_>>(),
            "ok": report.ok,
            "declined": report.declined,
        }));

        serde_json::to_string_pretty(&report).map_err(|e| format!("serialize report: {e}"))
    }

    /// The controlled `minimize` tool (`AGENT-INTROSPECTION-MCP.5.3`):
    /// delta-debug the failing `(seed, cfg)` to a smaller failing reproducer,
    /// using the sandboxed `validate` oracle on every candidate. Same
    /// guardrails as `validate` (fixed tool allow-list, fixed OS-temp sandbox,
    /// no arbitrary shell or path) plus a hard oracle-call budget so the search
    /// is bounded. The reproducible call — the minimized config's content
    /// address, the knob reductions, the spent budget, and the surviving tool
    /// command lines — is audit-logged.
    fn run_minimize(&mut self, seed: u64, cfg: &Config, args: &Value) -> Result<String, String> {
        let max_oracle_calls = match args.get("max_oracle_calls") {
            None | Some(Value::Null) => MinimizeOptions::default().max_oracle_calls,
            Some(v) => {
                let n = v
                    .as_u64()
                    .ok_or_else(|| "`max_oracle_calls` must be a positive integer".to_string())?;
                if n == 0 {
                    return Err("`max_oracle_calls` must be >= 1".to_string());
                }
                u32::try_from(n).map_err(|_| "`max_oracle_calls` is too large".to_string())?
            }
        };
        let opts = MinimizeOptions {
            validate: ValidateOptions {
                tools: parse_validate_tools(args)?,
                yosys_mode: parse_yosys_mode_arg(args)?,
                ..ValidateOptions::default()
            },
            max_oracle_calls,
        };
        let report = downstream::minimize(seed, cfg, &opts).map_err(|e| e.to_string())?;

        // Audit-log the reproducible call: the minimized config's content
        // address, the seed, what was reduced, the budget spent, and — when the
        // failure survived — the exact command line of every tool that still
        // rejects the minimized artifact.
        let minimized_run_id = introspect::content_run_id("dut", seed, &report.minimized_config);
        let commands: Vec<String> = report
            .final_validation
            .as_ref()
            .map(|r| r.tools.iter().map(|t| t.argv.join(" ")).collect())
            .unwrap_or_default();
        self.audit.push(json!({
            "tool": "minimize",
            "seed": seed,
            "lane": "dut",
            "reproduced_initial": report.reproduced_initial,
            "minimized_run_id": minimized_run_id,
            "reductions": report
                .reductions
                .iter()
                .map(|r| r.knob.clone())
                .collect::<Vec<_>>(),
            "oracle_calls": report.oracle_calls,
            "budget_exhausted": report.budget_exhausted,
            "declined": report.declined,
            "commands": commands,
        }));

        serde_json::to_string_pretty(&report).map_err(|e| format!("serialize report: {e}"))
    }

    /// Build the artifact for `(seed, cfg)`, store it in the content-addressed
    /// cache keyed by the document's `run_id`, and return that `run_id`.
    fn cache_artifact(&mut self, doc: &introspect::IntrospectionDocument) -> String {
        let run_id = doc.request.run_id.clone();
        let document = serde_json::to_value(doc).unwrap_or(Value::Null);
        let (sv, kind, top) = (
            rebuild_sv_for(doc),
            doc.artifact.kind.clone(),
            doc.artifact.top.clone().unwrap_or_default(),
        );
        self.cache.entry(run_id.clone()).or_insert(CachedArtifact {
            kind,
            top,
            sv,
            document,
        });
        run_id
    }

    fn resources_list(&self) -> Value {
        let mut resources = vec![
            json!({
                "uri": "anvil://catalog/knobs",
                "name": "knob catalog (default Config)",
                "mimeType": "application/json",
            }),
            json!({
                "uri": "anvil://catalog/lanes",
                "name": "artifact lane catalog",
                "mimeType": "application/json",
            }),
            json!({
                "uri": "anvil://audit/log",
                "name": "validate audit log",
                "mimeType": "application/json",
            }),
        ];
        for (run_id, art) in &self.cache {
            resources.push(json!({
                "uri": format!("anvil://artifact/{run_id}/sv"),
                "name": format!("{} {} SystemVerilog", art.kind, art.top),
                "mimeType": "text/x-systemverilog",
            }));
            resources.push(json!({
                "uri": format!("anvil://artifact/{run_id}/introspection"),
                "name": format!("{} {} introspection", art.kind, art.top),
                "mimeType": "application/json",
            }));
        }
        json!({ "resources": resources })
    }

    fn resources_read(&self, id: Value, params: &Value) -> Value {
        let uri = match params.get("uri").and_then(Value::as_str) {
            Some(u) => u,
            None => return err(id, INVALID_PARAMS, "resources/read requires a `uri`"),
        };

        let (mime, text) = match uri {
            "anvil://catalog/knobs" => (
                "application/json",
                serde_json::to_string_pretty(&Config::default()).unwrap_or_default(),
            ),
            "anvil://catalog/lanes" => (
                "application/json",
                serde_json::to_string_pretty(&json!({
                    "default": "dut",
                    "lanes": [
                        { "name": "dut", "description": "DUT synthesizable RTL (Phases 1-6)." },
                        { "name": "microdesign", "description": "Oracle-backed micro-design (Phase 7)." },
                        { "name": "frontend", "description": "Source-level frontend/elaboration accept (Phase 8)." },
                    ]
                }))
                .unwrap_or_default(),
            ),
            "anvil://audit/log" => (
                "application/json",
                serde_json::to_string_pretty(&self.audit).unwrap_or_default(),
            ),
            other => match parse_artifact_uri(other) {
                Some((run_id, part)) => match self.cache.get(run_id) {
                    Some(art) if part == "sv" => ("text/x-systemverilog", art.sv.clone()),
                    Some(art) if part == "introspection" => (
                        "application/json",
                        serde_json::to_string_pretty(&art.document).unwrap_or_default(),
                    ),
                    Some(_) => {
                        return err(id, INVALID_PARAMS, &format!("unknown artifact part in `{other}`"))
                    }
                    None => {
                        return err(
                            id,
                            INVALID_PARAMS,
                            &format!("no cached artifact for `{other}` (call generate first)"),
                        )
                    }
                },
                None => return err(id, INVALID_PARAMS, &format!("unknown resource uri `{other}`")),
            },
        };

        ok(
            id,
            json!({ "contents": [ { "uri": uri, "mimeType": mime, "text": text } ] }),
        )
    }
}

/// Decode tool arguments into `(seed, validated Config)`. `config` (when
/// present) must be a **full** effective Config (as emitted by `dump_config`),
/// because `Config` has no partial-deserialize defaults; omit it for the
/// defaults. `seed` overrides `config.seed`.
fn config_from_args(args: &Value) -> Result<(u64, Config), String> {
    let mut cfg = match args.get("config") {
        Some(c) if !c.is_null() => serde_json::from_value::<Config>(c.clone())
            .map_err(|e| format!("invalid config: {e}"))?,
        _ => Config::default(),
    };
    let seed = args.get("seed").and_then(Value::as_u64).unwrap_or(cfg.seed);
    cfg.seed = seed;
    cfg.validate().map_err(|e| e.to_string())?;
    Ok((seed, cfg))
}

/// Build the DUT artifact and its introspection document for `(seed, cfg)`.
/// Mirrors the CLI single-artifact dispatch (`hierarchical` ⇒ design).
fn build_artifact(
    seed: u64,
    cfg: &Config,
) -> (String, String, String, introspect::IntrospectionDocument) {
    let mut gen = Generator::new(cfg.clone());
    if cfg.effective_hierarchy_depth_range().is_some() {
        let design = gen.generate_design();
        let sv = emit::to_sv_design(&design);
        let doc = introspect::design_document(seed, cfg, &design);
        (sv, "design".to_string(), design.top.clone(), doc)
    } else {
        let m = gen.generate_module();
        let sv = emit::to_sv(&m);
        let doc = introspect::module_document(seed, cfg, &m);
        (sv, "module".to_string(), m.name.clone(), doc)
    }
}

/// Re-derive the emitted SV for a cached document by regenerating from its
/// request echo. Deterministic: `(seed, knobs)` ⇒ byte-identical SV, so this
/// is the same artifact the document describes.
fn rebuild_sv_for(doc: &introspect::IntrospectionDocument) -> String {
    let (sv, _kind, _top, _doc) = build_artifact(doc.request.seed, &doc.request.knobs);
    sv
}

/// Parse `anvil://artifact/<run_id>/<part>` into `(run_id, part)`.
fn parse_artifact_uri(uri: &str) -> Option<(&str, &str)> {
    let rest = uri.strip_prefix("anvil://artifact/")?;
    rest.split_once('/')
}

/// Parse the agent-facing `yosys_mode` string into a [`YosysMode`]. Returns
/// `None` for anything off the fixed set so the tool reports a clean error.
fn parse_yosys_mode(s: &str) -> Option<YosysMode> {
    match s {
        "without-abc" => Some(YosysMode::WithoutAbc),
        "with-abc" => Some(YosysMode::WithAbc),
        "both" => Some(YosysMode::Both),
        _ => None,
    }
}

/// Parse the shared `tools` argument used by both `validate` and `minimize`
/// into the fixed [`AcceptanceTool`] allow-list. Absent ⇒ the default
/// `verilator + yosys`. Any off-allow-list name is a clean error, never a
/// spawn. One owner so the two controlled tools cannot drift apart.
fn parse_validate_tools(args: &Value) -> Result<Vec<AcceptanceTool>, String> {
    match args.get("tools") {
        None | Some(Value::Null) => Ok(vec![AcceptanceTool::Verilator, AcceptanceTool::Yosys]),
        Some(Value::Array(items)) => {
            let mut selected = Vec::with_capacity(items.len());
            for item in items {
                let name = item
                    .as_str()
                    .ok_or_else(|| "`tools` entries must be strings".to_string())?;
                let tool = AcceptanceTool::from_name(name).ok_or_else(|| {
                    format!("unknown tool '{name}': allowed = verilator, yosys, iverilog")
                })?;
                selected.push(tool);
            }
            Ok(selected)
        }
        Some(_) => Err("`tools` must be an array of tool names".to_string()),
    }
}

/// Parse the shared `yosys_mode` argument used by both `validate` and
/// `minimize`. Absent ⇒ `without-abc`; off-set ⇒ a clean error.
fn parse_yosys_mode_arg(args: &Value) -> Result<YosysMode, String> {
    match args.get("yosys_mode").and_then(Value::as_str) {
        None => Ok(YosysMode::WithoutAbc),
        Some(s) => parse_yosys_mode(s).ok_or_else(|| {
            format!("unknown yosys_mode '{s}': allowed = without-abc, with-abc, both")
        }),
    }
}

// --- Agent-workflow prompts (`AGENT-INTROSPECTION-MCP.6`) --------------------
//
// MCP *prompts* are the third protocol primitive (beside tools + resources):
// named, parameterized workflow templates the agent fetches with `prompts/get`
// and then executes by calling this server's tools in the order the rendered
// message lays out. Each prompt here packages one bug-hunting loop end-to-end
// over the *existing* tools/resources — it adds no new capability and computes
// no new truth; it is pure guidance text, instantiated with the caller's sample
// arguments. The five workflows are exactly those named in decision `0004`.

/// One declared argument of a workflow prompt (MCP `PromptArgument`).
struct PromptArg {
    name: &'static str,
    description: &'static str,
    required: bool,
}

/// A pure prompt renderer: argument map -> ordered `(role, text)` messages.
type PromptRender = fn(&BTreeMap<String, String>) -> Vec<(&'static str, String)>;

/// A workflow prompt: its name, one-line description, declared arguments, and a
/// pure renderer that instantiates the workflow messages from those arguments.
struct PromptSpec {
    name: &'static str,
    description: &'static str,
    args: &'static [PromptArg],
    render: PromptRender,
}

impl PromptSpec {
    /// The MCP `Prompt` descriptor returned by `prompts/list`.
    fn descriptor(&self) -> Value {
        json!({
            "name": self.name,
            "description": self.description,
            "arguments": self
                .args
                .iter()
                .map(|a| json!({
                    "name": a.name,
                    "description": a.description,
                    "required": a.required,
                }))
                .collect::<Vec<_>>(),
        })
    }
}

/// The fixed registry of agent-workflow prompts (order is the `prompts/list`
/// order). One owner so the prompt set cannot drift from the dispatch.
static PROMPTS: &[PromptSpec] = &[
    PromptSpec {
        name: "find_downstream_bug",
        description: "Autonomous loop: generate valid-by-construction RTL, validate it against the vetted downstream tools, and on a rejection minimize it to a reproducer.",
        args: &[
            PromptArg { name: "seed", description: "RNG seed to start from (default 42).", required: false },
            PromptArg { name: "tools", description: "Comma-separated downstream tools (default verilator,yosys).", required: false },
            PromptArg { name: "yosys_mode", description: "Yosys mode: without-abc | with-abc | both (default without-abc).", required: false },
        ],
        render: render_find_downstream_bug,
    },
    PromptSpec {
        name: "close_coverage_gap",
        description: "Raise the generation knob(s) that light a currently-dark coverage surface, then confirm the metric is non-zero and still downstream-clean.",
        args: &[
            PromptArg { name: "target", description: "The coverage surface / metric to exercise (e.g. saw_fsm_design).", required: true },
            PromptArg { name: "seed", description: "RNG seed (default 42).", required: false },
        ],
        render: render_close_coverage_gap,
    },
    PromptSpec {
        name: "minimize_reproducer",
        description: "Shrink a failing (seed, knobs) to a minimal downstream reproducer (seed held fixed; deterministic, budget-bounded).",
        args: &[
            PromptArg { name: "seed", description: "The failing seed (held fixed — it pins the reproducer).", required: true },
            PromptArg { name: "tools", description: "Comma-separated oracle tools (default verilator,yosys).", required: false },
            PromptArg { name: "yosys_mode", description: "Yosys mode: without-abc | with-abc | both (default without-abc).", required: false },
        ],
        render: render_minimize_reproducer,
    },
    PromptSpec {
        name: "triage_tool_failures",
        description: "Validate a (seed, knobs) artifact, then classify which downstream tool/mode rejected it and extract the actionable diagnostic.",
        args: &[
            PromptArg { name: "seed", description: "RNG seed (default 42).", required: false },
            PromptArg { name: "tools", description: "Comma-separated downstream tools (default verilator,yosys).", required: false },
            PromptArg { name: "yosys_mode", description: "Yosys mode: without-abc | with-abc | both (default without-abc).", required: false },
        ],
        render: render_triage_tool_failures,
    },
    PromptSpec {
        name: "explain_artifact",
        description: "Explain a generated artifact from construction-truth (recorded metrics/provenance), not by parsing the emitted SV.",
        args: &[
            PromptArg { name: "seed", description: "RNG seed (default 42).", required: false },
        ],
        render: render_explain_artifact,
    },
];

/// `prompts/list`: the static registry of agent-workflow prompts.
fn prompts_list() -> Value {
    json!({ "prompts": PROMPTS.iter().map(PromptSpec::descriptor).collect::<Vec<_>>() })
}

/// `prompts/get`: instantiate one workflow's messages from its arguments.
/// Validates the prompt name, the argument value types (MCP prompt arguments
/// are strings), and that every declared-required argument is present, before
/// rendering — so a malformed request is a clean JSON-RPC error, never a panic.
fn prompts_get(id: Value, params: &Value) -> Value {
    let name = params.get("name").and_then(Value::as_str).unwrap_or("");
    let spec = match PROMPTS.iter().find(|p| p.name == name) {
        Some(s) => s,
        None => return err(id, INVALID_PARAMS, &format!("unknown prompt: {name}")),
    };

    // Collect the (string-valued) arguments, per the MCP prompt contract.
    let mut argmap = BTreeMap::new();
    if let Some(obj) = params.get("arguments").and_then(Value::as_object) {
        for (k, v) in obj {
            match v.as_str() {
                Some(s) => {
                    argmap.insert(k.clone(), s.to_string());
                }
                None => {
                    return err(
                        id,
                        INVALID_PARAMS,
                        &format!("prompt argument `{k}` must be a string"),
                    )
                }
            }
        }
    }

    // Every declared-required argument must be present.
    for a in spec.args {
        if a.required && !argmap.contains_key(a.name) {
            return err(
                id,
                INVALID_PARAMS,
                &format!("prompt `{name}` requires argument `{}`", a.name),
            );
        }
    }

    let messages: Vec<Value> = (spec.render)(&argmap)
        .into_iter()
        .map(|(role, text)| json!({ "role": role, "content": { "type": "text", "text": text } }))
        .collect();
    ok(
        id,
        json!({ "description": spec.description, "messages": messages }),
    )
}

/// Fetch a prompt argument or its default.
fn prompt_arg(args: &BTreeMap<String, String>, key: &str, default: &str) -> String {
    args.get(key)
        .cloned()
        .unwrap_or_else(|| default.to_string())
}

/// Render a comma-separated `tools` argument as a JSON array literal for the
/// workflow text, e.g. `verilator, iverilog` -> `["verilator", "iverilog"]`.
fn prompt_tools_array(args: &BTreeMap<String, String>, default: &str) -> String {
    let raw = prompt_arg(args, "tools", default);
    let items: Vec<String> = raw
        .split(',')
        .map(str::trim)
        .filter(|t| !t.is_empty())
        .map(|t| format!("\"{t}\""))
        .collect();
    format!("[{}]", items.join(", "))
}

fn render_find_downstream_bug(args: &BTreeMap<String, String>) -> Vec<(&'static str, String)> {
    let seed = prompt_arg(args, "seed", "42");
    let tools = prompt_tools_array(args, "verilator,yosys");
    let mode = prompt_arg(args, "yosys_mode", "without-abc");
    let text = format!(
        "Hunt for a downstream-tool bug with ANVIL's valid-by-construction RTL. \
ANVIL is the oracle: any rejection of its output is a candidate downstream-tool bug, never an ANVIL bug — and you must never mutate or repair the RTL.\n\
\n\
Run this tool chain in order:\n\
1. `generate` {{ \"seed\": {seed} }} -> note `run_id`; read `anvil://artifact/<run_id>/sv` if you want the source.\n\
2. `validate` {{ \"seed\": {seed}, \"tools\": {tools}, \"yosys_mode\": \"{mode}\" }} -> inspect `ok` and the per-tool reports.\n\
3. If `ok` is false (a vetted tool rejected valid-by-construction RTL): call `minimize` {{ \"seed\": {seed}, \"tools\": {tools}, \"yosys_mode\": \"{mode}\" }} to shrink (seed, knobs) to a minimal reproducer, then read `anvil://audit/log` for the exact reproducible command lines.\n\
4. If `ok` is true: the artifact is downstream-clean — pick another seed and repeat."
    );
    vec![("user", text)]
}

fn render_close_coverage_gap(args: &BTreeMap<String, String>) -> Vec<(&'static str, String)> {
    let seed = prompt_arg(args, "seed", "42");
    let target = prompt_arg(args, "target", "<coverage target>");
    let text = format!(
        "Drive a generation knob so a currently-dark coverage surface ({target}) is exercised — rules-first: light it by construction, never by post-hoc filtering.\n\
\n\
Run this tool chain in order:\n\
1. Read the `anvil://catalog/knobs` resource for the default Config and the knob taxonomy.\n\
2. `dump_config` {{ \"seed\": {seed} }} -> the effective Config baseline.\n\
3. Raise the knob(s) that gate {target} (e.g. set the owning motif probability to 1.0) and call `introspect` {{ \"seed\": {seed}, \"config\": <edited config> }} -> confirm the matching metric under `introspection.module_metrics` / `introspection.design_metrics` is now non-zero.\n\
4. `validate` {{ \"seed\": {seed}, \"config\": <edited config> }} -> confirm the newly-exercised surface is still downstream-clean."
    );
    vec![("user", text)]
}

fn render_minimize_reproducer(args: &BTreeMap<String, String>) -> Vec<(&'static str, String)> {
    let seed = prompt_arg(args, "seed", "<failing seed>");
    let tools = prompt_tools_array(args, "verilator,yosys");
    let mode = prompt_arg(args, "yosys_mode", "without-abc");
    let text = format!(
        "Shrink a failing (seed, knobs) to a minimal downstream reproducer. The seed is held fixed (it pins the reproducer); only knobs shrink. The search is deterministic and budget-bounded.\n\
\n\
Run this tool chain in order:\n\
1. `minimize` {{ \"seed\": {seed}, \"config\": <the failing Config from dump_config>, \"tools\": {tools}, \"yosys_mode\": \"{mode}\" }} (optionally cap the search with \"max_oracle_calls\").\n\
2. Inspect `reproduced_initial` (false => the case is downstream-clean, nothing to minimize), `reductions` (which knobs shrank), and `final_validation` (the surviving failing-tool reports).\n\
3. Read `anvil://audit/log` for the minimized `run_id` and the reproducible command lines."
    );
    vec![("user", text)]
}

fn render_triage_tool_failures(args: &BTreeMap<String, String>) -> Vec<(&'static str, String)> {
    let seed = prompt_arg(args, "seed", "42");
    let tools = prompt_tools_array(args, "verilator,yosys");
    let mode = prompt_arg(args, "yosys_mode", "without-abc");
    let text = format!(
        "Classify which downstream tool/mode rejected an artifact and extract the actionable diagnostic.\n\
\n\
Run this tool chain in order:\n\
1. `validate` {{ \"seed\": {seed}, \"tools\": {tools}, \"yosys_mode\": \"{mode}\" }}.\n\
2. For each entry in `tools[]`, read `ok`, `tool`, `argv` (the exact command line), and the captured output; identify the first failing tool and its message. A top-level `declined` verdict means the RAM guard stopped the run, not a tool failure.\n\
3. Read `anvil://audit/log` to recover the reproducible (run_id, seed, command lines).\n\
4. Summarize: tool, mode, failure class, and the next step (usually hand off to the `minimize_reproducer` workflow)."
    );
    vec![("user", text)]
}

fn render_explain_artifact(args: &BTreeMap<String, String>) -> Vec<(&'static str, String)> {
    let seed = prompt_arg(args, "seed", "42");
    let text = format!(
        "Explain a generated artifact from construction-truth — ANVIL records structure/provenance by construction, so read those facts instead of parsing the SV.\n\
\n\
Run this tool chain in order:\n\
1. `generate` {{ \"seed\": {seed} }} -> `run_id`, `kind`, `top`.\n\
2. `introspect` {{ \"seed\": {seed} }} -> read `artifact`, `config`, and `introspection.module_metrics` / `introspection.design_metrics`; these are ground truth.\n\
3. `resources/read` `anvil://artifact/<run_id>/sv` -> the emitted SystemVerilog, if you need the source.\n\
4. Summarize: lane, top module, width/depth/flop/motif structure, and which knobs shaped it. Do not claim whole-module intended behavior — ANVIL generates legal structure, not a spec."
    );
    vec![("user", text)]
}

fn ok(id: Value, result: Value) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "result": result })
}

fn err(id: Value, code: i64, message: &str) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "error": { "code": code, "message": message } })
}

/// MCP tool success: a single text-content block.
fn tool_text(text: &str) -> Value {
    json!({ "content": [ { "type": "text", "text": text } ], "isError": false })
}

/// MCP tool failure: text content with `isError: true` (a tool-level error,
/// not a JSON-RPC protocol error).
fn tool_error(message: &str) -> Value {
    json!({ "content": [ { "type": "text", "text": message } ], "isError": true })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn req(id: i64, method: &str, params: Value) -> Value {
        json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params })
    }

    fn call(server: &mut McpServer, id: i64, tool: &str, args: Value) -> Value {
        server
            .handle(&req(
                id,
                "tools/call",
                json!({ "name": tool, "arguments": args }),
            ))
            .unwrap()
    }

    fn tool_text_of(resp: &Value) -> String {
        resp["result"]["content"][0]["text"]
            .as_str()
            .unwrap()
            .to_string()
    }

    #[test]
    fn initialize_reports_server_info_and_protocol() {
        let mut s = McpServer::new();
        let resp = s.handle(&req(0, "initialize", json!({}))).unwrap();
        assert_eq!(resp["result"]["protocolVersion"], PROTOCOL_VERSION);
        assert_eq!(resp["result"]["serverInfo"]["name"], "anvil-mcp");
        assert!(resp["result"]["capabilities"]["tools"].is_object());
        assert!(resp["result"]["capabilities"]["resources"].is_object());
    }

    #[test]
    fn initialized_notification_has_no_response() {
        let mut s = McpServer::new();
        let n = json!({ "jsonrpc": "2.0", "method": "notifications/initialized" });
        assert!(s.handle(&n).is_none());
        assert!(s.initialized);
    }

    #[test]
    fn tools_list_has_the_pure_tools_and_controlled_tools() {
        let mut s = McpServer::new();
        let resp = s.handle(&req(1, "tools/list", json!({}))).unwrap();
        let names: Vec<&str> = resp["result"]["tools"]
            .as_array()
            .unwrap()
            .iter()
            .map(|t| t["name"].as_str().unwrap())
            .collect();
        assert_eq!(
            names,
            vec![
                "generate",
                "introspect",
                "dump_config",
                "validate",
                "minimize"
            ]
        );
    }

    #[test]
    fn introspect_tool_round_trips_to_the_schema_document() {
        let mut s = McpServer::new();
        let resp = call(&mut s, 2, "introspect", json!({ "seed": 42 }));
        assert_eq!(resp["result"]["isError"], false);
        let doc: Value = serde_json::from_str(&tool_text_of(&resp)).unwrap();
        assert_eq!(doc["schema_version"], "1.0");
        assert_eq!(doc["lane"], "dut");
        assert_eq!(doc["request"]["seed"], 42);
        // Matches the introspect surface exactly (same construction-truth).
        let cfg = Config {
            seed: 42,
            ..Config::default()
        };
        let mut gen = Generator::new(cfg.clone());
        let m = gen.generate_module();
        let direct = serde_json::to_value(introspect::module_document(42, &cfg, &m)).unwrap();
        assert_eq!(doc, direct);
    }

    #[test]
    fn generate_then_read_resources_round_trips() {
        let mut s = McpServer::new();
        let gen_resp = call(&mut s, 3, "generate", json!({ "seed": 7 }));
        let summary: Value = serde_json::from_str(&tool_text_of(&gen_resp)).unwrap();
        let run_id = summary["run_id"].as_str().unwrap().to_string();
        assert_eq!(summary["kind"], "module");

        // resources/list now includes this artifact + the static catalogs.
        let list = s.handle(&req(4, "resources/list", json!({}))).unwrap();
        let uris: Vec<String> = list["result"]["resources"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| r["uri"].as_str().unwrap().to_string())
            .collect();
        assert!(uris.contains(&"anvil://catalog/knobs".to_string()));
        assert!(uris.contains(&format!("anvil://artifact/{run_id}/sv")));

        // resources/read the SV: non-empty SystemVerilog.
        let sv_resp = s
            .handle(&req(
                5,
                "resources/read",
                json!({ "uri": format!("anvil://artifact/{run_id}/sv") }),
            ))
            .unwrap();
        let sv = sv_resp["result"]["contents"][0]["text"].as_str().unwrap();
        assert!(sv.contains("module "));

        // resources/read the introspection document: schema 1.0.
        let doc_resp = s
            .handle(&req(
                6,
                "resources/read",
                json!({ "uri": format!("anvil://artifact/{run_id}/introspection") }),
            ))
            .unwrap();
        let doc: Value =
            serde_json::from_str(doc_resp["result"]["contents"][0]["text"].as_str().unwrap())
                .unwrap();
        assert_eq!(doc["schema_version"], "1.0");
    }

    #[test]
    fn dump_config_returns_effective_config() {
        let mut s = McpServer::new();
        let resp = call(&mut s, 7, "dump_config", json!({ "seed": 9 }));
        let cfg: Config = serde_json::from_str(&tool_text_of(&resp)).unwrap();
        assert_eq!(cfg.seed, 9);
    }

    #[test]
    fn catalog_resources_are_readable() {
        let mut s = McpServer::new();
        let resp = s
            .handle(&req(
                8,
                "resources/read",
                json!({ "uri": "anvil://catalog/knobs" }),
            ))
            .unwrap();
        let cfg: Config =
            serde_json::from_str(resp["result"]["contents"][0]["text"].as_str().unwrap()).unwrap();
        assert_eq!(cfg.seed, Config::default().seed);
    }

    #[test]
    fn unknown_method_is_a_jsonrpc_error() {
        let mut s = McpServer::new();
        let resp = s.handle(&req(9, "no/such/method", json!({}))).unwrap();
        assert_eq!(resp["error"]["code"], METHOD_NOT_FOUND);
    }

    #[test]
    fn unknown_resource_uri_is_an_error() {
        let mut s = McpServer::new();
        let resp = s
            .handle(&req(10, "resources/read", json!({ "uri": "anvil://nope" })))
            .unwrap();
        assert_eq!(resp["error"]["code"], INVALID_PARAMS);
    }

    #[test]
    fn invalid_config_is_a_tool_error_not_a_panic() {
        let mut s = McpServer::new();
        // min_width > max_width fails Config::validate().
        let bad = json!({ "seed": 0, "config": { "min_width": 99, "max_width": 1 } });
        let resp = call(&mut s, 11, "introspect", bad);
        // Full-config deserialize fails (missing fields) OR validate fails;
        // either way it is a clean tool error, never a panic.
        assert_eq!(resp["result"]["isError"], true);
    }

    #[test]
    fn handle_line_round_trips_json_text() {
        let mut s = McpServer::new();
        let out = s
            .handle_line(r#"{"jsonrpc":"2.0","id":1,"method":"tools/list"}"#)
            .unwrap();
        assert!(out.contains("\"generate\""));
        // A blank line is ignored.
        assert!(s.handle_line("   ").is_none());
        // A malformed line yields a parse-error response, not a panic.
        let perr = s.handle_line("{not json").unwrap();
        assert!(perr.contains("-32700"));
    }

    #[test]
    fn generate_run_id_is_deterministic() {
        let mut s = McpServer::new();
        let a = tool_text_of(&call(&mut s, 1, "generate", json!({ "seed": 5 })));
        let b = tool_text_of(&call(&mut s, 2, "generate", json!({ "seed": 5 })));
        let ra: Value = serde_json::from_str(&a).unwrap();
        let rb: Value = serde_json::from_str(&b).unwrap();
        assert_eq!(ra["run_id"], rb["run_id"]);
    }

    #[test]
    fn validate_tool_no_tools_round_trips_and_audits() {
        let mut s = McpServer::new();
        // Audit log starts empty.
        let empty = s
            .handle(&req(
                20,
                "resources/read",
                json!({ "uri": "anvil://audit/log" }),
            ))
            .unwrap();
        let log: Value =
            serde_json::from_str(empty["result"]["contents"][0]["text"].as_str().unwrap()).unwrap();
        assert_eq!(log.as_array().unwrap().len(), 0);

        // `tools: []` exercises the generate+sandbox path without needing any
        // external tool present — portable.
        let resp = call(&mut s, 21, "validate", json!({ "seed": 7, "tools": [] }));
        assert_eq!(resp["result"]["isError"], false);
        let report: Value = serde_json::from_str(&tool_text_of(&resp)).unwrap();
        assert_eq!(report["lane"], "dut");
        assert_eq!(report["kind"], "module");
        assert_eq!(report["ok"], true);
        assert!(report["declined"].is_null());
        assert_eq!(report["tools"].as_array().unwrap().len(), 0);

        // The call was audit-logged with its reproducible run_id.
        let after = s
            .handle(&req(
                22,
                "resources/read",
                json!({ "uri": "anvil://audit/log" }),
            ))
            .unwrap();
        let log: Value =
            serde_json::from_str(after["result"]["contents"][0]["text"].as_str().unwrap()).unwrap();
        let entries = log.as_array().unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0]["tool"], "validate");
        assert_eq!(entries[0]["run_id"], report["run_id"]);
        assert_eq!(entries[0]["seed"], 7);
    }

    #[test]
    fn validate_tool_rejects_unknown_tool_name() {
        let mut s = McpServer::new();
        let resp = call(
            &mut s,
            23,
            "validate",
            json!({ "seed": 0, "tools": ["bash"] }),
        );
        assert_eq!(resp["result"]["isError"], true);
        assert!(tool_text_of(&resp).contains("unknown tool"));
        // A rejected call must not be audit-logged (it never ran).
        let log = s
            .handle(&req(
                24,
                "resources/read",
                json!({ "uri": "anvil://audit/log" }),
            ))
            .unwrap();
        let entries: Value =
            serde_json::from_str(log["result"]["contents"][0]["text"].as_str().unwrap()).unwrap();
        assert_eq!(entries.as_array().unwrap().len(), 0);
    }

    #[test]
    fn validate_tool_rejects_unknown_yosys_mode() {
        let mut s = McpServer::new();
        let resp = call(
            &mut s,
            25,
            "validate",
            json!({ "seed": 0, "tools": [], "yosys_mode": "turbo" }),
        );
        assert_eq!(resp["result"]["isError"], true);
        assert!(tool_text_of(&resp).contains("unknown yosys_mode"));
    }

    #[test]
    fn minimize_tool_no_repro_round_trips_and_audits() {
        let mut s = McpServer::new();
        // `tools: []` ⇒ the validate oracle is vacuously ok ⇒ nothing
        // reproduces. Portable: needs no external tool present.
        let resp = call(&mut s, 30, "minimize", json!({ "seed": 7, "tools": [] }));
        assert_eq!(resp["result"]["isError"], false);
        let report: Value = serde_json::from_str(&tool_text_of(&resp)).unwrap();
        assert_eq!(report["seed"], 7);
        assert_eq!(report["reproduced_initial"], false);
        assert_eq!(report["reductions"].as_array().unwrap().len(), 0);
        assert_eq!(report["oracle_calls"], 1);
        assert!(report["final_validation"].is_null());

        // The call was audit-logged as a minimize entry.
        let log = s
            .handle(&req(
                31,
                "resources/read",
                json!({ "uri": "anvil://audit/log" }),
            ))
            .unwrap();
        let entries: Value =
            serde_json::from_str(log["result"]["contents"][0]["text"].as_str().unwrap()).unwrap();
        let entries = entries.as_array().unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0]["tool"], "minimize");
        assert_eq!(entries[0]["seed"], 7);
        assert_eq!(entries[0]["reproduced_initial"], false);
    }

    #[test]
    fn minimize_tool_rejects_unknown_tool_name() {
        let mut s = McpServer::new();
        let resp = call(
            &mut s,
            32,
            "minimize",
            json!({ "seed": 0, "tools": ["rm -rf /"] }),
        );
        assert_eq!(resp["result"]["isError"], true);
        assert!(tool_text_of(&resp).contains("unknown tool"));
    }

    #[test]
    fn minimize_tool_rejects_zero_budget() {
        let mut s = McpServer::new();
        let resp = call(
            &mut s,
            33,
            "minimize",
            json!({ "seed": 0, "tools": [], "max_oracle_calls": 0 }),
        );
        assert_eq!(resp["result"]["isError"], true);
        assert!(tool_text_of(&resp).contains("max_oracle_calls"));
    }

    // --- Agent-workflow prompts (`AGENT-INTROSPECTION-MCP.6`) ----------------

    fn prompt_get(server: &mut McpServer, id: i64, name: &str, args: Value) -> Value {
        server
            .handle(&req(
                id,
                "prompts/get",
                json!({ "name": name, "arguments": args }),
            ))
            .unwrap()
    }

    fn prompt_text(resp: &Value) -> String {
        resp["result"]["messages"][0]["content"]["text"]
            .as_str()
            .unwrap()
            .to_string()
    }

    #[test]
    fn initialize_advertises_prompts_capability() {
        let mut s = McpServer::new();
        let resp = s.handle(&req(0, "initialize", json!({}))).unwrap();
        assert!(resp["result"]["capabilities"]["prompts"].is_object());
    }

    #[test]
    fn prompts_list_lists_the_five_workflows() {
        let mut s = McpServer::new();
        let resp = s.handle(&req(1, "prompts/list", json!({}))).unwrap();
        let prompts = resp["result"]["prompts"].as_array().unwrap();
        let names: Vec<&str> = prompts
            .iter()
            .map(|p| p["name"].as_str().unwrap())
            .collect();
        assert_eq!(
            names,
            vec![
                "find_downstream_bug",
                "close_coverage_gap",
                "minimize_reproducer",
                "triage_tool_failures",
                "explain_artifact",
            ]
        );
        // Each declares a description and an arguments list.
        for p in prompts {
            assert!(p["description"].as_str().unwrap().len() > 10);
            assert!(p["arguments"].is_array());
        }
    }

    #[test]
    fn prompts_get_renders_each_workflow_tool_chain() {
        let mut s = McpServer::new();

        // find_downstream_bug names generate -> validate -> minimize.
        let text = prompt_text(&prompt_get(
            &mut s,
            2,
            "find_downstream_bug",
            json!({ "seed": "42" }),
        ));
        assert!(text.contains("`generate`"));
        assert!(text.contains("`validate`"));
        assert!(text.contains("`minimize`"));
        assert!(text.contains("\"seed\": 42"));

        // explain_artifact names generate -> introspect -> the sv resource.
        let text = prompt_text(&prompt_get(
            &mut s,
            3,
            "explain_artifact",
            json!({ "seed": "7" }),
        ));
        assert!(text.contains("`generate`"));
        assert!(text.contains("`introspect`"));
        assert!(text.contains("anvil://artifact/<run_id>/sv"));
        assert!(text.contains("\"seed\": 7"));

        // triage_tool_failures names validate + the audit log.
        let text = prompt_text(&prompt_get(&mut s, 4, "triage_tool_failures", json!({})));
        assert!(text.contains("`validate`"));
        assert!(text.contains("anvil://audit/log"));

        // minimize_reproducer (seed required) names minimize + audit log.
        let text = prompt_text(&prompt_get(
            &mut s,
            5,
            "minimize_reproducer",
            json!({ "seed": "9" }),
        ));
        assert!(text.contains("`minimize`"));
        assert!(text.contains("\"seed\": 9"));
        assert!(text.contains("anvil://audit/log"));

        // close_coverage_gap (target required) names the knobs catalog + introspect.
        let text = prompt_text(&prompt_get(
            &mut s,
            6,
            "close_coverage_gap",
            json!({ "target": "saw_fsm_design" }),
        ));
        assert!(text.contains("anvil://catalog/knobs"));
        assert!(text.contains("`introspect`"));
        assert!(text.contains("saw_fsm_design"));
    }

    #[test]
    fn prompts_get_substitutes_the_tools_array() {
        let mut s = McpServer::new();
        let text = prompt_text(&prompt_get(
            &mut s,
            2,
            "find_downstream_bug",
            json!({ "tools": "verilator, iverilog" }),
        ));
        assert!(text.contains("[\"verilator\", \"iverilog\"]"));
    }

    #[test]
    fn prompts_get_enforces_required_args_and_unknown_name() {
        let mut s = McpServer::new();
        // close_coverage_gap requires `target`.
        let r = prompt_get(&mut s, 2, "close_coverage_gap", json!({ "seed": "1" }));
        assert_eq!(r["error"]["code"], INVALID_PARAMS);
        // minimize_reproducer requires `seed`.
        let r = prompt_get(&mut s, 3, "minimize_reproducer", json!({}));
        assert_eq!(r["error"]["code"], INVALID_PARAMS);
        // Unknown prompt name.
        let r = prompt_get(&mut s, 4, "no_such_prompt", json!({}));
        assert_eq!(r["error"]["code"], INVALID_PARAMS);
        // Non-string argument value is rejected (MCP prompt args are strings).
        let r = s
            .handle(&req(
                5,
                "prompts/get",
                json!({ "name": "explain_artifact", "arguments": { "seed": 42 } }),
            ))
            .unwrap();
        assert_eq!(r["error"]["code"], INVALID_PARAMS);
    }

    #[test]
    fn each_workflow_tool_chain_runs_end_to_end_on_a_sample() {
        // The external-tool legs are exercised with `tools: []` so every chain
        // runs portably (no verilator/yosys needed); the validate/minimize
        // sandbox + oracle path still executes. This proves each prompt's named
        // chain is a real, runnable sequence against this very server.
        let mut s = McpServer::new();

        // explain_artifact: generate -> introspect -> resources/read sv.
        let gen = call(&mut s, 1, "generate", json!({ "seed": 42 }));
        let summary: Value = serde_json::from_str(&tool_text_of(&gen)).unwrap();
        let run_id = summary["run_id"].as_str().unwrap().to_string();
        let intro = call(&mut s, 2, "introspect", json!({ "seed": 42 }));
        assert_eq!(intro["result"]["isError"], false);
        let sv = s
            .handle(&req(
                3,
                "resources/read",
                json!({ "uri": format!("anvil://artifact/{run_id}/sv") }),
            ))
            .unwrap();
        assert!(sv["result"]["contents"][0]["text"]
            .as_str()
            .unwrap()
            .contains("module "));

        // find_downstream_bug / triage_tool_failures: generate -> validate (ok).
        let val = call(&mut s, 4, "validate", json!({ "seed": 42, "tools": [] }));
        let report: Value = serde_json::from_str(&tool_text_of(&val)).unwrap();
        assert_eq!(report["ok"], true);

        // minimize_reproducer: minimize (no repro on downstream-clean output).
        let min = call(&mut s, 5, "minimize", json!({ "seed": 42, "tools": [] }));
        let mreport: Value = serde_json::from_str(&tool_text_of(&min)).unwrap();
        assert_eq!(mreport["reproduced_initial"], false);

        // close_coverage_gap: dump_config -> introspect surfaces the metrics block.
        let cfg = call(&mut s, 6, "dump_config", json!({ "seed": 42 }));
        assert_eq!(cfg["result"]["isError"], false);
        let doc: Value = serde_json::from_str(&tool_text_of(&intro)).unwrap();
        assert!(doc["introspection"]["module_metrics"].is_object());

        // The validate + minimize legs were audit-logged (chain observability).
        let log = s
            .handle(&req(
                7,
                "resources/read",
                json!({ "uri": "anvil://audit/log" }),
            ))
            .unwrap();
        let entries: Value =
            serde_json::from_str(log["result"]["contents"][0]["text"].as_str().unwrap()).unwrap();
        assert_eq!(entries.as_array().unwrap().len(), 2);
    }
}
