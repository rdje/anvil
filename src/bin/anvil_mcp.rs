//! `anvil-mcp` — the read-only MCP server binary (`AGENT-INTROSPECTION-MCP.4`).
//!
//! A thin transport shell over [`anvil::mcp::McpServer`]. By default it speaks
//! the MCP **stdio** transport: it reads newline-delimited JSON-RPC 2.0
//! messages from stdin, dispatches each through the pure server, and writes
//! each response as one JSON line to stdout (flushing per message, as the MCP
//! stdio transport requires).
//!
//! With `--http <addr>` (`AGENT-MCP-EXPANSION.4b`) it instead serves the same
//! dispatcher over a hand-rolled HTTP/1.1 POST transport (loopback by default).
//! All protocol logic — tools, resources, prompts, the content-addressed cache,
//! the HTTP framing — lives in `anvil::mcp` and is unit-tested there; this
//! binary is just the transport selection.
//!
//! It runs no external tools of its own and writes no files; it is a separate
//! target, so the default `anvil` build and the `--artifact dut` byte-identical
//! contract are unaffected.

use anvil::mcp::{self, McpServer};
use std::io::{BufRead, Write};

fn main() -> std::io::Result<()> {
    // Hand-parse a single optional `--http <addr>` — the transport bin stays
    // dependency-light (no clap surface). Anything else is a clean usage error.
    let mut args = std::env::args().skip(1);
    let mut http_addr: Option<String> = None;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--http" => {
                http_addr = Some(args.next().ok_or_else(|| {
                    std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "--http requires an address (a bare PORT or IP:PORT)",
                    )
                })?);
            }
            "-h" | "--help" => {
                print_usage();
                return Ok(());
            }
            other => {
                eprintln!("anvil-mcp: unknown argument: {other}");
                print_usage();
                std::process::exit(2);
            }
        }
    }

    match http_addr {
        Some(arg) => {
            let (addr, non_loopback) = mcp::resolve_http_addr(&arg)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?;
            if non_loopback {
                eprintln!(
                    "anvil-mcp: WARNING: binding the non-loopback address {addr} exposes the \
                     controlled validate/minimize tools over the network. Prefer a loopback bind \
                     (a bare PORT binds 127.0.0.1) unless you trust everyone who can reach it."
                );
            }
            mcp::serve_http(addr)
        }
        None => serve_stdio(),
    }
}

/// The default stdio transport: one JSON-RPC message per line in, one response
/// line out, flushed per message.
fn serve_stdio() -> std::io::Result<()> {
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

fn print_usage() {
    eprintln!(
        "anvil-mcp — read-only MCP server for ANVIL\n\
         \n\
         USAGE:\n    \
         anvil-mcp                 speak JSON-RPC 2.0 over stdio (default)\n    \
         anvil-mcp --http <ADDR>   speak JSON-RPC 2.0 over HTTP POST; <ADDR> is a\n                              \
         bare PORT (binds 127.0.0.1:PORT, loopback) or IP:PORT\n    \
         anvil-mcp --help          show this help"
    );
}
