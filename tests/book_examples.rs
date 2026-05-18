//! BOOK-EXAMPLES-RUNNABLE.2.2 — every runnable mdBook `bash` example
//! actually runs.
//!
//! The mdBook (`book/`) is the public, copy-paste user surface
//! (`https://rdje.github.io/anvil/`). This harness extracts every
//! ```bash fenced block from `book/src/*.md`, skips those carrying an
//! HTML-comment sentinel `<!-- book-test: skip — <reason> -->` on the
//! line immediately before the fence, and runs the rest as shell
//! scripts in a fresh temp CWD with `cargo run --release --` resolved
//! to the test-built `anvil` binary (functionally equivalent to what a
//! user runs — exit-code contract, not release-perf). A non-skipped
//! block that still references `cargo`, a bare `anvil`, or an external
//! tool (`jq`/`verilator`/`yosys`/`git`) is a *classification bug*:
//! the harness panics so the gap can never be silent. `cargo test`
//! (hence CI) gates on this.
//!
//! std-only by design (no new dependency for a test harness).

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

// The book says `cargo run --release --`; the harness honours that —
// it builds and runs the *release* binary (fidelity to what a user
// runs). Each runnable block completes in well under a second; large
// stdout is captured to a file (see `run_script`) so there is no
// pipe-buffer deadlock. This timeout is therefore purely a defensive
// backstop against a genuine hang — generous so a CPU-starved shared
// runner never flakes on it.
const PER_BLOCK_TIMEOUT: Duration = Duration::from_secs(600);

use std::sync::OnceLock;

/// Build the release `anvil` once and return its path. Falls back to
/// the Cargo-provided test binary if the release build is unavailable.
fn anvil_bin() -> &'static str {
    static BIN: OnceLock<String> = OnceLock::new();
    BIN.get_or_init(|| {
        let status = Command::new(env!("CARGO"))
            .args(["build", "--release", "--bin", "anvil"])
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .status();
        let rel = Path::new(env!("CARGO_MANIFEST_DIR")).join("target/release/anvil");
        match status {
            Ok(s) if s.success() && rel.is_file() => rel.to_string_lossy().into_owned(),
            _ => env!("CARGO_BIN_EXE_anvil").to_string(),
        }
    })
}

fn book_src() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("book/src")
}

fn md_files() -> Vec<PathBuf> {
    let mut v: Vec<PathBuf> = fs::read_dir(book_src())
        .expect("book/src readable")
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().map(|x| x == "md").unwrap_or(false))
        .collect();
    v.sort();
    v
}

struct Block {
    file: String,
    fence_line: usize, // 1-based line of the ```bash
    skip_reason: Option<String>,
    body: String,
}

fn parse_blocks(md: &Path) -> Vec<Block> {
    let text = fs::read_to_string(md).unwrap();
    let lines: Vec<&str> = text.split('\n').collect();
    let name = md.file_name().unwrap().to_string_lossy().to_string();
    let mut out = Vec::new();
    let mut i = 0usize;
    while i < lines.len() {
        if lines[i].trim() == "```bash" {
            // Sentinel must be the line IMMEDIATELY before the fence.
            let skip_reason = if i > 0 {
                let prev = lines[i - 1].trim();
                if let Some(rest) = prev.strip_prefix("<!-- book-test: skip") {
                    let reason = rest.trim_start_matches([' ', '—', '-', ':']).trim();
                    let reason = reason.trim_end_matches("-->").trim();
                    assert!(
                        !reason.is_empty(),
                        "{name}:{}: book-test skip sentinel must carry a reason",
                        i
                    );
                    Some(reason.to_string())
                } else {
                    None
                }
            } else {
                None
            };
            let mut body = String::new();
            let mut j = i + 1;
            while j < lines.len() && lines[j].trim() != "```" {
                body.push_str(lines[j]);
                body.push('\n');
                j += 1;
            }
            out.push(Block {
                file: name.clone(),
                fence_line: i + 1,
                skip_reason,
                body,
            });
            i = j + 1;
        } else {
            i += 1;
        }
    }
    out
}

/// Run `script` as bash in a fresh temp CWD with a timeout. Returns
/// the combined output on failure for diagnostics.
fn run_script(script: &str, tag: &str) -> Result<(), String> {
    let anvil = anvil_bin();
    let dir = std::env::temp_dir().join(format!(
        "anvil-booktest-{}-{}",
        std::process::id(),
        tag.replace([':', '/', '.', ' '], "_")
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();

    // Capture child stdout/stderr to FILES, not pipes. A book example
    // routinely prints a whole module to stdout (a default `--seed 42`
    // module is ~86 KB; a 5-level `factorization` sweep ~525 KB) which
    // overflows the ~64 KB OS pipe buffer. With an undrained pipe the
    // child blocks forever on write(), never exits, and the wait loop
    // below would spin to the timeout — a false "TIMED OUT". Files have
    // no such limit and need no concurrent reader thread (std-only).
    // The capture files live OUTSIDE the script CWD so a block that
    // enumerates its working dir is unaffected.
    let out_path = std::env::temp_dir().join(format!(
        "anvil-booktest-cap-{}-{}.out",
        std::process::id(),
        tag.replace([':', '/', '.', ' '], "_")
    ));
    let err_path = out_path.with_extension("err");
    let out_f = fs::File::create(&out_path).map_err(|e| e.to_string())?;
    let err_f = fs::File::create(&err_path).map_err(|e| e.to_string())?;

    let full = format!("set -euo pipefail\nexport ANVIL={anvil}\n{script}\n");
    let mut child = Command::new("bash")
        .arg("-c")
        .arg(&full)
        .current_dir(&dir)
        .env("CARGO_NET_OFFLINE", "true")
        .env("ANVIL", anvil)
        .stdin(Stdio::null())
        .stdout(Stdio::from(out_f))
        .stderr(Stdio::from(err_f))
        .spawn()
        .map_err(|e| format!("spawn bash failed: {e}"))?;

    let cleanup = |dir: &Path, o: &Path, e: &Path| {
        let _ = fs::remove_dir_all(dir);
        let _ = fs::remove_file(o);
        let _ = fs::remove_file(e);
    };

    let start = Instant::now();
    let status = loop {
        if let Some(s) = child.try_wait().map_err(|e| e.to_string())? {
            break s;
        }
        if start.elapsed() > PER_BLOCK_TIMEOUT {
            let _ = child.kill();
            let _ = child.wait(); // reap so no zombie outlives the test
            cleanup(&dir, &out_path, &err_path);
            return Err(format!("{tag}: TIMED OUT after {PER_BLOCK_TIMEOUT:?}"));
        }
        std::thread::sleep(Duration::from_millis(50));
    };
    let mut so = fs::read_to_string(&out_path).unwrap_or_default();
    so.push_str(&fs::read_to_string(&err_path).unwrap_or_default());
    cleanup(&dir, &out_path, &err_path);
    if status.success() {
        Ok(())
    } else {
        Err(format!(
            "{tag}: exit {:?}\n--- output (tail) ---\n{}",
            status.code(),
            so.lines()
                .rev()
                .take(15)
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect::<Vec<_>>()
                .join("\n")
        ))
    }
}

/// `cargo run --release --` / `cargo run --bin tool_matrix --` resolve
/// to the test-built binaries; a bare `anvil ` (defensive) likewise.
fn substitute(body: &str) -> String {
    body.replace("cargo run --release --", "\"$ANVIL\"")
}

#[test]
fn every_runnable_book_bash_block_succeeds() {
    let mut ran = 0usize;
    let mut skipped = 0usize;
    let mut failures = Vec::new();

    for md in md_files() {
        for b in parse_blocks(&md) {
            let tag = format!("{}:{}", b.file, b.fence_line);
            if b.skip_reason.is_some() {
                skipped += 1;
                continue;
            }
            let script = substitute(&b.body);
            // Classification guard: a non-skipped block must be fully
            // resolved to the binary and contain no external tool.
            for bad in [
                "cargo run",
                "cargo install",
                "\nanvil ",
                " anvil ",
                "verilator",
                "yosys",
                "jq ",
                "git clone",
            ] {
                if script.contains(bad) {
                    panic!(
                        "{tag}: non-skipped bash block still references `{}` after substitution \
                         — migrate it or add a `<!-- book-test: skip — <reason> -->` sentinel \
                         on the line before the fence.\n--- block ---\n{}",
                        bad.trim(),
                        b.body
                    );
                }
            }
            ran += 1;
            if let Err(e) = run_script(&script, &tag) {
                failures.push(e);
            }
        }
    }

    eprintln!("book examples: ran {ran} runnable bash block(s), {skipped} skip-sentineled");
    assert!(
        ran >= 40,
        "expected the bulk of the ~56 runnable blocks to be exercised, only {ran}"
    );
    assert!(
        failures.is_empty(),
        "{} book example(s) failed:\n\n{}",
        failures.len(),
        failures.join("\n\n")
    );
}

/// Negative control: prove the harness actually fails a broken
/// example (so a green result is meaningful, not vacuous).
#[test]
fn harness_detects_a_broken_command() {
    let broken = substitute("cargo run --release -- --totally-bogus-flag-xyz\n");
    let r = run_script(&broken, "negative-control");
    assert!(
        r.is_err(),
        "harness MUST fail on a broken command — a green run would be vacuous"
    );
}

/// Every skip sentinel carries a non-empty reason (no silent skips).
#[test]
fn skip_sentinels_have_reasons() {
    let mut n = 0;
    for md in md_files() {
        for b in parse_blocks(&md) {
            if let Some(r) = &b.skip_reason {
                assert!(!r.is_empty());
                n += 1;
            }
        }
    }
    assert!(n >= 1, "expected at least one skip-sentineled block");
    eprintln!("skip-sentineled blocks with reasons: {n}");
}

// Touch `Write` so an unused-import lint never masks a real issue if
// the harness evolves to write script files.
#[allow(dead_code)]
fn _assert_write_in_scope() {
    let _ = |mut v: Vec<u8>| v.write_all(b"x");
}
