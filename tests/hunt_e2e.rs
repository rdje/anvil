//! BUG-HUNT-ORCHESTRATION.2e — the real-tool end-to-end gate for the turnkey
//! `anvil hunt` bug-hunt loop, plus the reproducer-recipe fidelity proof.
//!
//! `#[ignore]`-gated and tool-gated, exactly like `tests/diff_sim.rs`: the
//! portable `cargo test` stays green on a tool-less host, while a run with the
//! real downstream tools installed (`cargo test --test hunt_e2e -- --ignored`)
//! exercises the whole `anvil hunt` CLI → `hunt::run` → `downstream::validate`
//! → real-Verilator path.
//!
//! ## Why there is no "manufactured ANVIL failure" here
//!
//! ANVIL output is **valid by construction**, so there is no by-construction way
//! to make a real downstream tool *reject* it: a genuine rejection would be an
//! actual downstream-tool bug — the very thing this loop exists to *surface* —
//! not a fixture we can fabricate (fabricating one would mean emitting illegal
//! RTL, which the project forbids). So this gate proves the two things that
//! *are* provable end-to-end with the real toolchain:
//!
//! 1. **the whole loop runs clean** against real Verilator — `n_failures == 0`,
//!    the expected steady state — end to end through the real `anvil hunt`
//!    binary, with the sweep really sweeping (distinct content-addressed
//!    `run_id`s per seed); and
//! 2. **the reproducer recipe is byte-identical-faithful**: `anvil --seed S
//!    --config <dumped knobs>` regenerates exactly the artifact `anvil --seed S`
//!    produced, and the real tool accepts it — i.e. a finding's `repro.sh`
//!    (regenerate from `(seed, knobs.json)` → re-run the tool) reproduces the
//!    artifact one-command.
//!
//! The reproducer **bundle directory format** itself (every file present, the
//! `repro.sh` sandbox-path substitution, the minimized-vs-original choice) is
//! proven cargo-portably by `write_bundle_emits_a_self_contained_reproducer_directory`
//! in `src/hunt` (`.2b.2b`) — it needs no real tool. Together they close the
//! `BUG-HUNT-ORCHESTRATION` tree: the engine, both invocation surfaces, the
//! bundle format, and the real-tool loop + reproducer recipe are all proven.

use std::path::PathBuf;
use std::process::Command;

/// Cheap presence probe: the binary spawns at all (`--version` may exit
/// non-zero, but a spawn failure means the tool is absent). Mirrors the spirit
/// of `diff_sim::tools_present`.
fn tool_present(bin: &str) -> bool {
    Command::new(bin).arg("--version").output().is_ok()
}

/// The freshly-built `anvil` binary under test (Cargo sets this for integration
/// tests), so the e2e gate drives the real CLI, not a library re-entry.
fn anvil() -> &'static str {
    env!("CARGO_BIN_EXE_anvil")
}

fn work_dir(tag: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(tag);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("create e2e work dir");
    dir
}

/// The real `anvil hunt` CLI runs the loop end-to-end against real Verilator and
/// returns a clean sweep (`n_failures == 0`) — valid-by-construction RTL is
/// downstream-clean, so a finding here would be a candidate **downstream-tool**
/// bug, not an ANVIL bug. Also proves the sweep really swept (one distinct
/// content-addressed `run_id` per seed).
#[test]
#[ignore]
fn hunt_cli_clean_sweep_against_real_verilator() {
    if !tool_present("verilator") {
        eprintln!(
            "hunt_cli_clean_sweep_against_real_verilator: verilator not on $PATH \
             (skipping; rerun with it installed for the real-tool hunt gate)"
        );
        return;
    }
    let out = Command::new(anvil())
        .args([
            "hunt",
            "--seed",
            "1",
            "--seeds",
            "3",
            "--tools",
            "verilator",
        ])
        .output()
        .expect("run anvil hunt");
    assert!(
        out.status.success(),
        "anvil hunt exited non-zero:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let report: anvil::hunt::HuntReport =
        serde_json::from_slice(&out.stdout).expect("anvil hunt must print a HuntReport JSON");

    assert_eq!(report.lane, "dut");
    assert_eq!(report.base_seed, 1);
    assert_eq!(report.seeds, 3);
    assert_eq!(report.verdicts.len(), 3);
    // Valid-by-construction ⇒ a clean sweep is the expected steady state.
    assert_eq!(
        report.summary.n_failures, 0,
        "ANVIL output should be downstream-clean; a failure here is a candidate \
         downstream-tool bug. failures={:?}",
        report.failures
    );
    assert_eq!(report.summary.n_clean, 3);
    // The sweep really swept: a distinct content address per seed.
    let ids: std::collections::BTreeSet<&str> =
        report.verdicts.iter().map(|v| v.run_id.as_str()).collect();
    assert_eq!(ids.len(), 3, "each seed must content-address distinctly");
    eprintln!("hunt_cli_clean_sweep_against_real_verilator: clean 3-seed sweep, run_ids={ids:?}");
}

/// The reproducer **recipe** a finding's `repro.sh` runs is byte-identical-
/// faithful against the real binary + tool: `anvil --seed S --config <dumped
/// knobs>` regenerates exactly what `anvil --seed S` produced (step 1), and the
/// real tool accepts that regenerated `.sv` (step 2). This is what makes a
/// `--out` bundle genuinely one-command-reproducible.
#[test]
#[ignore]
fn hunt_reproducer_recipe_is_byte_identical_and_accepted() {
    if !tool_present("verilator") {
        eprintln!(
            "hunt_reproducer_recipe_is_byte_identical_and_accepted: verilator not on $PATH \
             (skipping)"
        );
        return;
    }
    let dir = work_dir("hunt-repro-recipe");
    let knobs = dir.join("knobs.json");

    // `repro.sh` step 1a — dump the effective knobs (what the bundle persists).
    let dumped = Command::new(anvil())
        .args(["--seed", "5", "--dump-config"])
        .output()
        .expect("anvil --dump-config");
    assert!(dumped.status.success());
    std::fs::write(&knobs, &dumped.stdout).expect("write knobs.json");

    // `repro.sh` step 1b — regenerate the RTL from (seed, knobs.json).
    let regen = Command::new(anvil())
        .args(["--seed", "5", "--config"])
        .arg(&knobs)
        .output()
        .expect("anvil --config <knobs>");
    assert!(regen.status.success());

    // It must equal the direct generation for the same seed, byte-for-byte —
    // the reproducibility contract `repro.sh` relies on.
    let direct = Command::new(anvil())
        .args(["--seed", "5"])
        .output()
        .expect("anvil --seed 5");
    assert!(direct.status.success());
    assert_eq!(
        regen.stdout, direct.stdout,
        "anvil --config <dumped knobs> must reproduce anvil --seed byte-for-byte"
    );

    // `repro.sh` step 2 — the real tool accepts the regenerated artifact.
    let sv = dir.join("repro.sv");
    std::fs::write(&sv, &regen.stdout).expect("write repro.sv");
    let lint = Command::new("verilator")
        .arg("--lint-only")
        .arg(&sv)
        .output()
        .expect("run verilator");
    assert!(
        lint.status.success(),
        "verilator must accept the regenerated repro.sv:\n{}",
        String::from_utf8_lossy(&lint.stderr)
    );
    eprintln!("hunt_reproducer_recipe_is_byte_identical_and_accepted: regen byte-identical + verilator clean");
}
