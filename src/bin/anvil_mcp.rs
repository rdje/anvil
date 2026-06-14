//! `anvil-mcp` — the read-only MCP server binary (`AGENT-INTROSPECTION-MCP.4`).
//!
//! A thin stdio loop over [`anvil::mcp::McpServer`]: it reads newline-delimited
//! JSON-RPC 2.0 messages from stdin, dispatches each through the pure server,
//! and writes each response as one JSON line to stdout (flushing per message,
//! as the MCP stdio transport requires). All protocol logic — tools, resources,
//! the content-addressed cache — lives in `anvil::mcp` and is unit-tested
//! there; this binary is just the transport.
//!
//! It runs no external tools and writes no files; it is a separate target, so
//! the default `anvil` build and the `--artifact dut` byte-identical contract
//! are unaffected.

use anvil::mcp::McpServer;
use std::io::{BufRead, Write};

fn main() -> std::io::Result<()> {
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout().lock();
    let mut server = McpServer::new();

    for line in stdin.lock().lines() {
        let line = line?;
        if let Some(response) = server.handle_line(&line) {
            stdout.write_all(response.as_bytes())?;
            stdout.write_all(b"\n")?;
            stdout.flush()?;
        }
    }
    Ok(())
}
