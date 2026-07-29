//! `VOLUME-DATA-LOCALITY.2` — the single resolver for where ANVIL writes
//! its own working data.
//!
//! **The rule this module enforces:** no ANVIL-owned data — sandboxes,
//! reproducer bundles, scratch corpora, debug dumps — may land on a
//! different filesystem volume from the project it belongs to. The OS
//! temp directory (`std::env::temp_dir()`, i.e. `/tmp` on Unix) fails
//! that rule whenever the project lives on a mounted volume rather than
//! the boot disk, which is the normal case for a large checkout on an
//! external SSD. Data written there is also silently purged, so a
//! "banked" artifact can evaporate between sessions (the defect
//! `EVIDENCE-BANK-DURABILITY` was opened for).
//!
//! Every call site therefore resolves through [`sandbox_root`] rather
//! than reaching for `temp_dir()` itself — one resolver, not a policy
//! re-implemented per module (`feedback_full_factorization`).
//!
//! ## Why not simply "the ANVIL repo root"?
//!
//! ANVIL is **distributed**: tagged release binaries and a composite
//! GitHub Action run `anvil hunt` inside *other people's* projects,
//! where no ANVIL checkout exists. A hardcoded repo path would be both
//! wrong and unportable there. The honest generalization is *the
//! current project's own volume, derived at runtime* — which is what
//! the resolution order below computes, and why nothing here is ever
//! persisted as an absolute path.

use std::path::{Path, PathBuf};

/// Environment override for the sandbox root. Set it to place ANVIL's
/// working data anywhere the caller chooses (a RAM disk, a scratch
/// volume, a CI workspace). It wins over every derived location.
pub const SANDBOX_ROOT_ENV: &str = "ANVIL_SANDBOX_ROOT";

/// Directory, relative to the resolved project root, that holds ANVIL's
/// working data. Repo-root-*relative* by construction: the absolute
/// path is only ever derived at runtime, never stored.
const SANDBOX_SUBDIR: &str = ".cache/anvil-sandbox";

/// Markers that identify a project root when walking up from the
/// current directory. `.git` first: it is the outermost boundary of a
/// checkout, whereas `Cargo.toml` also appears in nested crates.
const ROOT_MARKERS: [&str; 2] = [".git", "Cargo.toml"];

/// The project root that ANVIL-owned data should live under, found by
/// walking up from `start` for a [`ROOT_MARKERS`] entry.
///
/// Returns `None` when no marker is found — the caller then falls back
/// to `start` itself, which is still on the project's volume (the
/// property that actually matters) even though it is not a "root".
pub fn find_project_root(start: &Path) -> Option<PathBuf> {
    for dir in start.ancestors() {
        if ROOT_MARKERS.iter().any(|m| dir.join(m).exists()) {
            return Some(dir.to_path_buf());
        }
    }
    None
}

/// Where ANVIL writes its sandboxes and other working data.
///
/// Resolution order, first match wins:
///
/// 1. `$ANVIL_SANDBOX_ROOT`, if set to a non-empty value — used
///    verbatim, so a caller can always place the data explicitly.
/// 2. `<project root>/.cache/anvil-sandbox`, where the root is the
///    nearest ancestor of the current directory holding a `.git` or
///    `Cargo.toml` marker. This keeps the data on the project's volume.
/// 3. `<current directory>/.cache/anvil-sandbox`, when no marker is
///    found — still the caller's own volume.
///
/// It never returns the OS temp directory. If the current directory
/// cannot be read at all (a deleted CWD), it resolves relative to `.`,
/// which remains volume-correct.
pub fn sandbox_root() -> PathBuf {
    if let Some(explicit) = std::env::var_os(SANDBOX_ROOT_ENV) {
        if !explicit.is_empty() {
            return PathBuf::from(explicit);
        }
    }
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let root = find_project_root(&cwd).unwrap_or(cwd);
    root.join(SANDBOX_SUBDIR)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A unique on-volume scratch directory for a test, derived through
    /// the resolver itself — so the test suite obeys the same rule it
    /// checks.
    fn scratch(tag: &str) -> PathBuf {
        let d = sandbox_root().join(format!("paths-test-{tag}"));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).expect("create scratch");
        d
    }

    #[test]
    fn sandbox_root_is_never_the_os_temp_dir() {
        // The defect this module exists to prevent: the OS temp dir is
        // a different volume from a checkout on an external disk.
        let root = sandbox_root();
        assert!(
            !root.starts_with(std::env::temp_dir()),
            "sandbox root {} must not live under the OS temp dir {}",
            root.display(),
            std::env::temp_dir().display()
        );
    }

    #[test]
    fn sandbox_root_lands_under_a_project_root_when_one_exists() {
        // Running under `cargo test`, the CWD is inside this crate, so
        // the marker walk must find a root and use it.
        let cwd = std::env::current_dir().expect("cwd");
        let root = find_project_root(&cwd).expect("crate has a Cargo.toml/.git marker");
        assert!(
            sandbox_root().starts_with(&root),
            "sandbox root must sit under the resolved project root"
        );
        assert!(sandbox_root().ends_with(SANDBOX_SUBDIR));
    }

    #[test]
    fn find_project_root_walks_up_to_the_marker() {
        let base = scratch("walk-up");
        let nested = base.join("a/b/c");
        std::fs::create_dir_all(&nested).expect("create nested");
        // No marker anywhere under `base`, but `base` itself is inside
        // this crate, so the walk finds the crate root — never `None`
        // for a path inside a checkout.
        let found = find_project_root(&nested).expect("a root exists above the crate scratch dir");
        assert!(nested.starts_with(&found));
        assert!(
            ROOT_MARKERS.iter().any(|m| found.join(m).exists()),
            "the returned root must actually carry a marker"
        );
        std::fs::remove_dir_all(&base).ok();
    }

    #[test]
    fn find_project_root_prefers_the_nearest_marker() {
        let base = scratch("nearest");
        let inner = base.join("inner");
        std::fs::create_dir_all(inner.join("deeper")).expect("create inner");
        std::fs::write(inner.join("Cargo.toml"), "[package]\n").expect("write marker");
        assert_eq!(
            find_project_root(&inner.join("deeper")).expect("marker found"),
            inner,
            "the nearest ancestor carrying a marker wins over the crate root above it"
        );
        std::fs::remove_dir_all(&base).ok();
    }

    /// The env override is read per call (no caching), so a caller can
    /// place the data explicitly. Asserted without mutating the process
    /// environment, which would race other tests.
    #[test]
    fn env_override_is_honoured_when_set() {
        match std::env::var_os(SANDBOX_ROOT_ENV) {
            Some(v) if !v.is_empty() => {
                assert_eq!(sandbox_root(), PathBuf::from(v));
            }
            _ => {
                // Unset in this run: the derived path must still be
                // marker-based, which the tests above already pin.
                assert!(sandbox_root().ends_with(SANDBOX_SUBDIR));
            }
        }
    }
}
