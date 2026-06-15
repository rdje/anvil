//! Optional hand-rolled HTTP/1.1 transport for `anvil-mcp`
//! (`AGENT-MCP-EXPANSION.4b`).
//!
//! A minimal, dependency-free HTTP/1.1 POST transport that drives the **same**
//! [`McpServer::handle_line`] dispatcher the stdio loop uses, so tools,
//! resources, prompts, error codes, and the content-addressed cache behave
//! identically over both transports. It is **opt-in** (`anvil-mcp --http
//! <addr>`); without the flag the bin runs the unchanged stdio loop, so the
//! default build and the `--artifact dut` byte-identical contract are
//! unaffected.
//!
//! Design (pinned in `AGENT-MCP-EXPANSION.4a`; rationale in
//! `DEVELOPMENT_NOTES.md` "Hand-rolled HTTP transport design"):
//!
//! - **Same dispatcher.** Each connection carries one JSON-RPC POST whose body
//!   is handed to [`McpServer::handle_line`]; there is no second protocol path.
//! - **One request per connection, `Connection: close`.** No keep-alive, no
//!   pipelining — the simplest robust framing for an RPC workload.
//! - **Single-threaded, one shared server.** [`serve_http`] reuses one
//!   [`McpServer`] across sequentially-served connections, so the cache and
//!   audit log persist across calls (as over stdio) with no locking. A
//!   per-connection read timeout keeps a stalled client from wedging the loop.
//! - **Tiny, framing-only status set.** `200` (response), `204` (notification),
//!   `400`/`405`/`411`/`413` (malformed framing). JSON-RPC-level errors stay
//!   inside the `200` body as JSON-RPC error objects, exactly as over stdio.
//! - **Loopback default.** [`resolve_http_addr`] binds a bare port on
//!   `127.0.0.1`; a non-loopback bind is honored but the bin warns that the
//!   controlled `validate`/`minimize` tools become network-reachable.
//! - **No new dependency.** `std::net` / `std::io` / `std::time` only.

use super::McpServer;
use std::io::{self, BufRead, BufReader, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::time::Duration;

/// Maximum request body we will read — defense-in-depth against a giant
/// allocation, even though the default bind is loopback.
const MAX_BODY_BYTES: usize = 16 * 1024 * 1024;

/// Per-connection read timeout: keeps a stalled client from wedging the
/// single-threaded accept loop.
const READ_TIMEOUT: Duration = Duration::from_secs(30);

/// The outcome of parsing one HTTP request off a connection.
#[derive(Debug)]
enum Request {
    /// A well-formed `POST` carrying this JSON-RPC body.
    Post(String),
    /// A framing error: respond with this status code + reason and close.
    Error(u16, &'static str),
    /// The client opened then closed the connection without sending a request.
    Eof,
}

/// The canonical reason phrase for each status this transport emits.
fn reason_phrase(status: u16) -> &'static str {
    match status {
        200 => "OK",
        204 => "No Content",
        400 => "Bad Request",
        405 => "Method Not Allowed",
        411 => "Length Required",
        413 => "Payload Too Large",
        _ => "Error",
    }
}

/// Resolve a `--http <addr>` argument to a bind address, applying the
/// loopback-default rule: a bare port `N` (all ASCII digits) ⇒ `127.0.0.1:N`;
/// a full `IP:PORT` is parsed and honored as given. Returns the address plus
/// whether the bind is **non-loopback**, so the caller can warn that the
/// controlled tools become network-reachable.
pub fn resolve_http_addr(arg: &str) -> Result<(SocketAddr, bool), String> {
    let addr: SocketAddr = if !arg.is_empty() && arg.bytes().all(|b| b.is_ascii_digit()) {
        let port: u16 = arg
            .parse()
            .map_err(|_| format!("invalid --http port (0..=65535): {arg}"))?;
        SocketAddr::from(([127, 0, 0, 1], port))
    } else {
        arg.parse()
            .map_err(|_| format!("invalid --http address (want PORT or IP:PORT): {arg}"))?
    };
    let non_loopback = !addr.ip().is_loopback();
    Ok((addr, non_loopback))
}

/// Parse one HTTP/1.1 request: the request line, headers (CRLF-delimited,
/// names matched case-insensitively) up to the blank line, then exactly
/// `Content-Length` body bytes. Only `POST` is accepted, on any path.
fn read_http_request<R: BufRead>(reader: &mut R) -> io::Result<Request> {
    // Request line: METHOD SP target SP HTTP/x.y
    let mut request_line = String::new();
    if reader.read_line(&mut request_line)? == 0 {
        return Ok(Request::Eof);
    }
    let method = request_line.split_whitespace().next().unwrap_or("");
    if !method.eq_ignore_ascii_case("POST") {
        return Ok(Request::Error(405, "only POST is accepted"));
    }

    // Headers until a blank line. We only need Content-Length.
    let mut content_length: Option<usize> = None;
    loop {
        let mut header = String::new();
        if reader.read_line(&mut header)? == 0 {
            return Ok(Request::Error(400, "unexpected eof in headers"));
        }
        let header = header.trim_end_matches(['\r', '\n']);
        if header.is_empty() {
            break; // end of headers
        }
        if let Some((name, value)) = header.split_once(':') {
            if name.trim().eq_ignore_ascii_case("content-length") {
                match value.trim().parse::<usize>() {
                    Ok(len) => content_length = Some(len),
                    Err(_) => return Ok(Request::Error(400, "malformed Content-Length")),
                }
            }
        }
        // Headers without a colon are ignored (lenient).
    }

    let len = match content_length {
        Some(len) => len,
        None => return Ok(Request::Error(411, "POST requires Content-Length")),
    };
    if len > MAX_BODY_BYTES {
        return Ok(Request::Error(413, "request body exceeds the 16 MiB cap"));
    }

    let mut body = vec![0u8; len];
    reader.read_exact(&mut body)?;
    match String::from_utf8(body) {
        Ok(s) => Ok(Request::Post(s)),
        Err(_) => Ok(Request::Error(400, "request body is not valid UTF-8")),
    }
}

/// Write one HTTP/1.1 response. `Some(json)` is a `200 OK` JSON body; `None`
/// is a bodyless status (`204` notification, or a `4xx` framing error). All
/// responses carry `Connection: close` since this transport is one-shot per
/// connection.
fn write_http_response<W: Write>(w: &mut W, status: u16, body: Option<&str>) -> io::Result<()> {
    write!(w, "HTTP/1.1 {status} {}\r\n", reason_phrase(status))?;
    match body {
        Some(b) => {
            let bytes = b.as_bytes();
            write!(w, "Content-Type: application/json\r\n")?;
            write!(w, "Content-Length: {}\r\n", bytes.len())?;
            write!(w, "Connection: close\r\n\r\n")?;
            w.write_all(bytes)?;
        }
        // RFC 7230 forbids a Content-Length on a 204 No Content.
        None if status == 204 => {
            write!(w, "Connection: close\r\n\r\n")?;
        }
        None => {
            write!(w, "Content-Length: 0\r\n")?;
            write!(w, "Connection: close\r\n\r\n")?;
        }
    }
    w.flush()
}

/// Serve exactly one HTTP request on an accepted connection, dispatching the
/// JSON-RPC body through the shared [`McpServer`]. One request per connection.
fn handle_http_connection(stream: TcpStream, server: &mut McpServer) -> io::Result<()> {
    stream.set_read_timeout(Some(READ_TIMEOUT))?;
    let mut writer = stream.try_clone()?;
    let mut reader = BufReader::new(stream);
    match read_http_request(&mut reader)? {
        Request::Post(body) => match server.handle_line(&body) {
            Some(resp) => write_http_response(&mut writer, 200, Some(&resp)),
            None => write_http_response(&mut writer, 204, None),
        },
        Request::Error(status, reason) => {
            eprintln!("anvil-mcp: HTTP {status} — {reason}");
            write_http_response(&mut writer, status, None)
        }
        Request::Eof => Ok(()),
    }
}

/// Bind `addr` and serve the MCP protocol over the hand-rolled HTTP/1.1 POST
/// transport, driving the same dispatcher as the stdio loop. Single-threaded:
/// one shared [`McpServer`] serves connections sequentially, so the
/// content-addressed cache and audit log persist across calls (exactly as over
/// stdio) with no locking. Per-connection errors are logged and swallowed; the
/// loop runs until the process is terminated.
///
/// Binding a non-loopback address exposes the controlled `validate` /
/// `minimize` tools over the network; the `anvil-mcp` bin warns when it does.
pub fn serve_http(addr: SocketAddr) -> io::Result<()> {
    let listener = TcpListener::bind(addr)?;
    let bound = listener.local_addr().unwrap_or(addr);
    eprintln!("anvil-mcp: HTTP transport listening on http://{bound} (POST JSON-RPC)");
    let mut server = McpServer::new();
    for incoming in listener.incoming() {
        match incoming {
            Ok(stream) => {
                if let Err(e) = handle_http_connection(stream, &mut server) {
                    eprintln!("anvil-mcp: connection error: {e}");
                }
            }
            Err(e) => eprintln!("anvil-mcp: accept error: {e}"),
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;
    use std::io::{Cursor, Read};

    fn post(body: &str) -> String {
        format!(
            "POST /mcp HTTP/1.1\r\nHost: localhost\r\nContent-Length: {}\r\n\r\n{}",
            body.len(),
            body
        )
    }

    #[test]
    fn parses_well_formed_post_body() {
        let raw = post(r#"{"jsonrpc":"2.0","id":1,"method":"tools/list"}"#);
        let mut r = Cursor::new(raw.into_bytes());
        match read_http_request(&mut r).unwrap() {
            Request::Post(b) => assert_eq!(b, r#"{"jsonrpc":"2.0","id":1,"method":"tools/list"}"#),
            other => panic!("expected Post, got {other:?}"),
        }
    }

    #[test]
    fn header_name_is_case_insensitive() {
        let body = r#"{"jsonrpc":"2.0","id":1,"method":"ping"}"#;
        let raw = format!(
            "POST / HTTP/1.1\r\ncOnTeNt-LeNgTh: {}\r\n\r\n{}",
            body.len(),
            body
        );
        let mut r = Cursor::new(raw.into_bytes());
        assert!(matches!(
            read_http_request(&mut r).unwrap(),
            Request::Post(_)
        ));
    }

    #[test]
    fn non_post_method_is_405() {
        let mut r = Cursor::new(b"GET / HTTP/1.1\r\n\r\n".to_vec());
        assert!(matches!(
            read_http_request(&mut r).unwrap(),
            Request::Error(405, _)
        ));
    }

    #[test]
    fn missing_content_length_on_post_is_411() {
        let mut r = Cursor::new(b"POST / HTTP/1.1\r\nHost: x\r\n\r\n".to_vec());
        assert!(matches!(
            read_http_request(&mut r).unwrap(),
            Request::Error(411, _)
        ));
    }

    #[test]
    fn malformed_content_length_is_400() {
        let mut r = Cursor::new(b"POST / HTTP/1.1\r\nContent-Length: abc\r\n\r\n".to_vec());
        assert!(matches!(
            read_http_request(&mut r).unwrap(),
            Request::Error(400, _)
        ));
    }

    #[test]
    fn oversized_body_is_413_without_reading_it() {
        // The cap is checked before the body is read, so a huge declared length
        // returns 413 even with no body present.
        let raw = format!(
            "POST / HTTP/1.1\r\nContent-Length: {}\r\n\r\n",
            MAX_BODY_BYTES + 1
        );
        let mut r = Cursor::new(raw.into_bytes());
        assert!(matches!(
            read_http_request(&mut r).unwrap(),
            Request::Error(413, _)
        ));
    }

    #[test]
    fn empty_stream_is_eof() {
        let mut r = Cursor::new(Vec::new());
        assert!(matches!(read_http_request(&mut r).unwrap(), Request::Eof));
    }

    #[test]
    fn writes_200_with_json_body() {
        let mut buf: Vec<u8> = Vec::new();
        write_http_response(&mut buf, 200, Some(r#"{"ok":true}"#)).unwrap();
        let out = String::from_utf8(buf).unwrap();
        assert!(out.starts_with("HTTP/1.1 200 OK\r\n"));
        assert!(out.contains("Content-Type: application/json\r\n"));
        assert!(out.contains("Content-Length: 11\r\n"));
        assert!(out.contains("Connection: close\r\n"));
        assert!(out.ends_with("\r\n\r\n{\"ok\":true}"));
    }

    #[test]
    fn writes_204_without_content_length() {
        let mut buf: Vec<u8> = Vec::new();
        write_http_response(&mut buf, 204, None).unwrap();
        let out = String::from_utf8(buf).unwrap();
        assert!(out.starts_with("HTTP/1.1 204 No Content\r\n"));
        assert!(!out.contains("Content-Length"));
        assert!(out.ends_with("Connection: close\r\n\r\n"));
    }

    #[test]
    fn resolve_bare_port_binds_loopback() {
        let (addr, non_loopback) = resolve_http_addr("8765").unwrap();
        assert_eq!(addr, SocketAddr::from(([127, 0, 0, 1], 8765)));
        assert!(!non_loopback);
    }

    #[test]
    fn resolve_explicit_loopback_is_not_flagged() {
        let (addr, non_loopback) = resolve_http_addr("127.0.0.1:9000").unwrap();
        assert_eq!(addr, SocketAddr::from(([127, 0, 0, 1], 9000)));
        assert!(!non_loopback);
    }

    #[test]
    fn resolve_non_loopback_is_flagged() {
        let (_addr, non_loopback) = resolve_http_addr("0.0.0.0:8765").unwrap();
        assert!(non_loopback);
    }

    #[test]
    fn resolve_rejects_garbage_and_out_of_range_port() {
        assert!(resolve_http_addr("not-an-addr").is_err());
        assert!(resolve_http_addr("99999").is_err()); // > u16::MAX
    }

    /// Real-socket round-trip: bind `127.0.0.1:0`, serve one connection in a
    /// thread, and POST a real `initialize` request — proving the HTTP framing
    /// drives the same dispatcher end-to-end.
    #[test]
    fn round_trips_initialize_over_a_real_socket() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let server_thread = std::thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            let mut server = McpServer::new();
            handle_http_connection(stream, &mut server).unwrap();
        });

        let mut client = TcpStream::connect(addr).unwrap();
        client
            .write_all(
                post(r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#).as_bytes(),
            )
            .unwrap();
        client.flush().unwrap();
        let mut resp = String::new();
        client.read_to_string(&mut resp).unwrap(); // reads until the server closes
        server_thread.join().unwrap();

        assert!(resp.starts_with("HTTP/1.1 200 OK\r\n"), "resp: {resp}");
        assert!(resp.contains("Content-Type: application/json"));
        // The JSON-RPC result carries the protocol handshake.
        let (_headers, json_body) = resp.split_once("\r\n\r\n").unwrap();
        let v: Value = serde_json::from_str(json_body).unwrap();
        assert_eq!(v["id"], 1);
        assert_eq!(v["result"]["protocolVersion"], crate::mcp::PROTOCOL_VERSION);
    }

    /// A notification (no `id`) yields a `204 No Content` over HTTP, since the
    /// dispatcher returns `None`.
    #[test]
    fn notification_round_trips_to_204() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let server_thread = std::thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            let mut server = McpServer::new();
            handle_http_connection(stream, &mut server).unwrap();
        });

        let mut client = TcpStream::connect(addr).unwrap();
        client
            .write_all(post(r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#).as_bytes())
            .unwrap();
        client.flush().unwrap();
        let mut resp = String::new();
        client.read_to_string(&mut resp).unwrap();
        server_thread.join().unwrap();

        assert!(
            resp.starts_with("HTTP/1.1 204 No Content\r\n"),
            "resp: {resp}"
        );
        assert!(!resp.contains("Content-Length"));
    }
}
