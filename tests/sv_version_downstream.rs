//! `SV-VERSION-TARGETING.2b.2a` — per-version downstream acceptance proof.
//!
//! The `.2b.1` slice landed the `--sv-version` capability bound and proved the
//! current subset is a 2012/2017/2023 *common floor* (cross-version
//! byte-identity). This gate proves the other half of decision `0009`'s
//! contract: the version-targeted corpus is **accepted by a downstream tool in
//! its matching standard mode**. It runs Verilator with `--language 1800-2012`
//! / `--language 1800-2017` / `--language 1800-2023` on a representative ANVIL
//! corpus and asserts each is warning-clean, and confirms Icarus `-g2012`
//! (its newest generation) accepts the subset too.
//!
//! It is `#[ignore]`-gated (tool-dependent — the diff-sim / parity-gate
//! precedent): the default `cargo test` does not require Verilator/Icarus.
//! Run it explicitly to bank evidence:
//!
//! ```text
//! cargo test --test sv_version_downstream -- --ignored --nocapture
//! ```
//!
//! The full repo-owned matrix industrialization (a `--sv-version-gate` +
//! `ScenarioSet::SvVersionSweep` + a `saw_sv_version_targeted_acceptance`
//! coverage fact) is the follow-on leaf `SV-VERSION-TARGETING.2b.2b`.

use anvil::config::SvVersion;
use anvil::downstream::{run_iverilog_compile, run_verilator, run_verilator_design};
use anvil::{Config, Generator};
use std::path::PathBuf;
use std::process::Command;

const VERSIONS: [SvVersion; 3] = [SvVersion::Sv2012, SvVersion::Sv2017, SvVersion::Sv2023];

fn tool_present(bin: &str, version_flag: &str) -> bool {
    Command::new(bin)
        .arg(version_flag)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// A unique, clean scratch dir under `target/tmp` for one corpus member.
fn scratch_dir(tag: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("tmp")
        .join("sv-version-downstream")
        .join(tag);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("create scratch dir");
    dir
}

fn leaf_corpus() -> Vec<(&'static str, Config)> {
    vec![
        (
            "comb",
            Config {
                seed: 1,
                ..Config::default()
            },
        ),
        (
            "seq",
            Config {
                seed: 2,
                flop_prob: 0.6,
                max_flops_per_module: 16,
                ..Config::default()
            },
        ),
        (
            "structured",
            Config {
                seed: 3,
                case_mux_prob: 0.5,
                casez_mux_prob: 0.5,
                for_fold_prob: 0.5,
                priority_encoder_prob: 0.3,
                ..Config::default()
            },
        ),
        (
            "memory",
            Config {
                seed: 5,
                memory_prob: 1.0,
                ..Config::default()
            },
        ),
        (
            "fsm",
            Config {
                seed: 6,
                fsm_prob: 1.0,
                ..Config::default()
            },
        ),
    ]
}

#[test]
#[ignore = "requires Verilator; run with --ignored to bank per-version acceptance evidence"]
fn verilator_accepts_each_sv_version_in_matching_language_mode() {
    if !tool_present("verilator", "--version") {
        eprintln!("verilator not present — skipping per-version acceptance gate");
        return;
    }

    // Leaf modules across the construct surface.
    for (tag, cfg) in leaf_corpus() {
        cfg.validate().expect("config valid");
        let m = Generator::new(cfg.clone()).generate_module();
        for v in VERSIONS {
            let dir = scratch_dir(&format!("leaf-{tag}-{}", v.ieee_standard()));
            let path = dir.join(format!("{}.sv", m.name));
            std::fs::write(&path, anvil::emit::to_sv_versioned(&m, v)).expect("write sv");
            let inv = run_verilator("verilator", &dir, &path, &m.name, Some(v.ieee_standard()))
                .expect("verilator invocation");
            assert!(
                inv.success,
                "verilator --language {} rejected leaf '{tag}' (seed {}): {:?}\nargv: {:?}",
                v.ieee_standard(),
                cfg.seed,
                inv.error,
                inv.argv
            );
        }
    }

    // A multi-file hierarchy design.
    let design_cfg = Config {
        seed: 11,
        min_hierarchy_depth: 2,
        max_hierarchy_depth: 2,
        min_child_instances_per_module: 2,
        max_child_instances_per_module: 2,
        ..Config::default()
    };
    design_cfg.validate().expect("design config valid");
    let design = Generator::new(design_cfg).generate_design();
    anvil::ir::validate::validate_design(&design).expect("design validates");
    for v in VERSIONS {
        let dir = scratch_dir(&format!("design-{}", v.ieee_standard()));
        let mut sv_paths = Vec::new();
        for module in &design.modules {
            let p = dir.join(format!("{}.sv", module.name));
            std::fs::write(
                &p,
                anvil::emit::to_sv_in_design_versioned(module, &design, v),
            )
            .expect("write sv");
            sv_paths.push(p);
        }
        let inv = run_verilator_design(
            "verilator",
            &dir,
            &sv_paths,
            &design.top,
            Some(v.ieee_standard()),
        )
        .expect("verilator design invocation");
        assert!(
            inv.success,
            "verilator --language {} rejected the hierarchy design: {:?}\nargv: {:?}",
            v.ieee_standard(),
            inv.error,
            inv.argv
        );
    }
}

#[test]
#[ignore = "requires Icarus Verilog; run with --ignored to confirm the subset compiles at -g2012"]
fn iverilog_g2012_accepts_the_subset_for_every_target() {
    if !tool_present("iverilog", "-V") {
        eprintln!("iverilog not present — skipping g2012 acceptance gate");
        return;
    }

    // Icarus' newest generation is -g2012; the whole current subset is
    // g2012-valid, so each version target (which down-gates to the same
    // subset today) compiles. A genuinely beyond-g2012 construct (only at
    // SV-VERSION-TARGETING.3) would gate this column to a recorded no-op.
    for (tag, cfg) in leaf_corpus() {
        cfg.validate().expect("config valid");
        let m = Generator::new(cfg.clone()).generate_module();
        for v in VERSIONS {
            let dir = scratch_dir(&format!("iverilog-{tag}-{}", v.ieee_standard()));
            let path = dir.join(format!("{}.sv", m.name));
            std::fs::write(&path, anvil::emit::to_sv_versioned(&m, v)).expect("write sv");
            let inv = run_iverilog_compile("iverilog", &dir, &path, &m.name)
                .expect("iverilog invocation");
            assert!(
                inv.success,
                "iverilog -g2012 rejected leaf '{tag}' targeting {} (seed {}): {:?}",
                v.ieee_standard(),
                cfg.seed,
                inv.error
            );
        }
    }
}
