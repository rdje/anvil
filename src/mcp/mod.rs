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
//! Scope of `.4`: the DUT lane, the three pure tools, resources over the
//! cache, and two static catalogs (`knobs`, `lanes`). The controlled
//! `validate` / `minimize` tools (which *do* run external tools, sandboxed)
//! are `.5`.

use crate::config::Config;
use crate::downstream::{self, AcceptanceTool, ValidateOptions, YosysMode};
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
                "capabilities": { "tools": {}, "resources": {} },
                "serverInfo": {
                    "name": "anvil-mcp",
                    "version": introspect::anvil_version(),
                },
                "instructions":
                    "ANVIL agent-introspection. Pure tools: generate, introspect, \
                     dump_config (construction-truth derived from existing \
                     metrics/config; no side effects). Controlled tool: validate \
                     runs the vetted downstream tools (verilator / yosys / \
                     iverilog) on a (seed, knobs) artifact inside a sandboxed temp \
                     dir — a fixed allow-list, no arbitrary shell — and audit-logs \
                     each call (see the anvil://audit/log resource). Artifacts and \
                     static catalogs are exposed as resources."
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
        let tools = match args.get("tools") {
            None | Some(Value::Null) => {
                vec![AcceptanceTool::Verilator, AcceptanceTool::Yosys]
            }
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
                selected
            }
            Some(_) => return Err("`tools` must be an array of tool names".to_string()),
        };
        let yosys_mode = match args.get("yosys_mode").and_then(Value::as_str) {
            None => YosysMode::WithoutAbc,
            Some(s) => parse_yosys_mode(s).ok_or_else(|| {
                format!("unknown yosys_mode '{s}': allowed = without-abc, with-abc, both")
            })?,
        };

        let opts = ValidateOptions {
            tools,
            yosys_mode,
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
    fn tools_list_has_the_pure_tools_and_validate() {
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
            vec!["generate", "introspect", "dump_config", "validate"]
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
}
