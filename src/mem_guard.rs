//! Opt-in internal RAM/RSS self-governor (`WORKLOAD-MEMORY-SAFETY.4`).
//!
//! Lets the `anvil` process sample **its own** resident set (RSS) and/or
//! the **host's** used-RAM percentage at safe checkpoints (between
//! modules/designs in the `--out` loop) and abort cleanly — deterministic
//! non-zero exit, a stderr message naming the seed + effective knobs —
//! *before* the host crosses a danger threshold. It complements
//! `scripts/ram_guard.sh` (`RESOURCE-SAFE-TOOLING`), which guards
//! *external* heavy jobs from the outside; this guards `anvil`'s own
//! process from the inside, catching the single-pathological-module /
//! huge-`--count` balloon that a 3-second external poll can outrun.
//!
//! ## Default-off / byte-identical
//!
//! Both limits default to the sentinel `0` (off). When both are off the
//! guard is *never sampled* — [`MemGuard::check`] short-circuits to
//! `None` before touching the OS — so default `anvil` (and
//! `--artifact dut`) output stays byte-identical and consumes RNG
//! identically. The governor never changes emitted RTL: it is a
//! process-safety mechanism, not a generation knob. It therefore aborts
//! *between* finished units (decline-to-start-more), never by truncating
//! a partially-built cone — that would emit invalid RTL and break
//! valid-by-construction (`feedback_rules_first_generation`).
//!
//! ## OS reads (mirrors `scripts/ram_guard.sh`)
//!
//! Host used-RAM%: macOS `memory_pressure` ("free percentage", so
//! `used = 100 - free`); Linux `/proc/meminfo` `MemTotal`/`MemAvailable`.
//! Process RSS: Linux `/proc/self/status` `VmRSS`; macOS
//! `ps -o rss= -p <pid>`. Every read is **best-effort**: an unreadable
//! sample yields `None` and never aborts a healthy run — the same policy
//! the shell guard uses ("a probe hiccup never kills a healthy job").

use crate::config::Config;
use std::fmt;

/// The two opt-in abort ceilings. Sentinel `0` = off on each axis.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct MemLimits {
    /// Abort once this process's RSS reaches this many MiB. `0` = off.
    pub max_rss_mb: u64,
    /// Abort once host used RAM reaches this percentage (`1..=100`).
    /// `0` = off.
    pub ram_abort_pct: u32,
}

impl MemLimits {
    /// True iff at least one axis is armed. When false the guard never
    /// samples the OS and never aborts (byte-identical default path).
    pub fn enabled(&self) -> bool {
        self.max_rss_mb > 0 || self.ram_abort_pct > 0
    }
}

/// One point-in-time reading. Either field is `None` when its axis is
/// disabled or the OS read failed (best-effort).
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct MemSample {
    pub rss_mb: Option<u64>,
    pub host_used_pct: Option<u32>,
}

/// Why the governor decided to abort. Carries both the observed value
/// and the configured ceiling so the message is self-explanatory.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum AbortReason {
    /// Process RSS crossed the `--max-rss-mb` ceiling.
    Rss { rss_mb: u64, limit_mb: u64 },
    /// Host used-RAM% crossed the `--ram-abort-pct` ceiling.
    Host { used_pct: u32, limit_pct: u32 },
}

impl fmt::Display for AbortReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AbortReason::Rss { rss_mb, limit_mb } => write!(
                f,
                "process RSS {rss_mb} MiB reached the --max-rss-mb {limit_mb} MiB ceiling"
            ),
            AbortReason::Host {
                used_pct,
                limit_pct,
            } => write!(
                f,
                "host used RAM {used_pct}% reached the --ram-abort-pct {limit_pct}% ceiling"
            ),
        }
    }
}

/// Pure decision logic: given the configured limits and one sample,
/// decide whether to abort. RSS (single-process balloon) is checked
/// first because it can outrun the host %-used signal. A disabled axis
/// or an unreadable (`None`) reading never trips. This function does no
/// I/O, so it is exhaustively unit-testable.
pub fn evaluate(limits: &MemLimits, sample: &MemSample) -> Option<AbortReason> {
    if limits.max_rss_mb > 0 {
        if let Some(rss_mb) = sample.rss_mb {
            if rss_mb >= limits.max_rss_mb {
                return Some(AbortReason::Rss {
                    rss_mb,
                    limit_mb: limits.max_rss_mb,
                });
            }
        }
    }
    if limits.ram_abort_pct > 0 {
        if let Some(used_pct) = sample.host_used_pct {
            if used_pct >= limits.ram_abort_pct {
                return Some(AbortReason::Host {
                    used_pct,
                    limit_pct: limits.ram_abort_pct,
                });
            }
        }
    }
    None
}

/// The configured guard. Holds the limits; samples the OS lazily and
/// only for the armed axes.
#[derive(Copy, Clone, Debug)]
pub struct MemGuard {
    limits: MemLimits,
}

impl MemGuard {
    /// Build the guard from the effective config.
    pub fn from_config(cfg: &Config) -> Self {
        Self {
            limits: MemLimits {
                max_rss_mb: cfg.max_rss_mb,
                ram_abort_pct: cfg.ram_abort_pct,
            },
        }
    }

    /// True iff at least one axis is armed.
    pub fn enabled(&self) -> bool {
        self.limits.enabled()
    }

    /// Read the OS for the armed axes only. A disarmed axis is left
    /// `None` so no `ps` / `memory_pressure` subprocess is spawned and
    /// no `/proc` file is read when it would not matter.
    pub fn sample(&self) -> MemSample {
        MemSample {
            rss_mb: (self.limits.max_rss_mb > 0)
                .then(read_process_rss_mb)
                .flatten(),
            host_used_pct: (self.limits.ram_abort_pct > 0)
                .then(read_host_used_pct)
                .flatten(),
        }
    }

    /// The checkpoint call: `None` (continue) when disabled or inside the
    /// safe envelope; `Some(reason)` when the run should abort. Disabled
    /// is the common (default) case and short-circuits before any OS read.
    pub fn check(&self) -> Option<AbortReason> {
        if !self.limits.enabled() {
            return None;
        }
        evaluate(&self.limits, &self.sample())
    }
}

/// The clean-abort message: names the reason, the seed, and the full
/// effective knobs so the aborted run is reproducible/inspectable.
pub fn abort_message(reason: &AbortReason, seed: u64, cfg: &Config) -> String {
    let knobs =
        serde_json::to_string(cfg).unwrap_or_else(|_| "<config serialization failed>".to_string());
    format!(
        "anvil: memory governor abort — {reason}.\n\
         Run stopped cleanly before the host danger zone; output is partial.\n\
         seed={seed}\n\
         effective knobs: {knobs}"
    )
}

/// Best-effort process-RSS read in MiB. Linux: `/proc/self/status`
/// `VmRSS` (kB). macOS: `ps -o rss= -p <pid>` (kB). `None` on any other
/// platform or on read/parse failure — the caller treats `None` as
/// "no abort".
pub fn read_process_rss_mb() -> Option<u64> {
    #[cfg(target_os = "linux")]
    {
        let status = std::fs::read_to_string("/proc/self/status").ok()?;
        for line in status.lines() {
            if let Some(rest) = line.strip_prefix("VmRSS:") {
                // `VmRSS:\t  123456 kB`
                let kb: u64 = rest.split_whitespace().next()?.parse().ok()?;
                return Some(kb / 1024);
            }
        }
        None
    }
    #[cfg(target_os = "macos")]
    {
        let pid = std::process::id();
        let out = std::process::Command::new("ps")
            .args(["-o", "rss=", "-p", &pid.to_string()])
            .output()
            .ok()?;
        if !out.status.success() {
            return None;
        }
        let kb: u64 = String::from_utf8_lossy(&out.stdout).trim().parse().ok()?;
        Some(kb / 1024)
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        None
    }
}

/// Best-effort host used-RAM percentage, mirroring `scripts/ram_guard.sh`.
/// Linux: `/proc/meminfo` `(MemTotal - MemAvailable) / MemTotal`. macOS:
/// `memory_pressure` "free percentage", so `used = 100 - free`. `None`
/// on any other platform or on read/parse failure.
pub fn read_host_used_pct() -> Option<u32> {
    #[cfg(target_os = "linux")]
    {
        let info = std::fs::read_to_string("/proc/meminfo").ok()?;
        let mut total: Option<u64> = None;
        let mut avail: Option<u64> = None;
        for line in info.lines() {
            if let Some(rest) = line.strip_prefix("MemTotal:") {
                total = rest.split_whitespace().next().and_then(|v| v.parse().ok());
            } else if let Some(rest) = line.strip_prefix("MemAvailable:") {
                avail = rest.split_whitespace().next().and_then(|v| v.parse().ok());
            }
        }
        let (total, avail) = (total?, avail?);
        if total == 0 {
            return None;
        }
        Some((((total - avail.min(total)) * 100) / total) as u32)
    }
    #[cfg(target_os = "macos")]
    {
        let out = std::process::Command::new("memory_pressure")
            .output()
            .ok()?;
        if !out.status.success() {
            return None;
        }
        let text = String::from_utf8_lossy(&out.stdout);
        for line in text.lines() {
            if let Some((_, rhs)) = line.split_once("free percentage") {
                // e.g. `System-wide memory free percentage: 42%`
                let free: u32 = rhs
                    .trim_start_matches([':', ' '])
                    .trim_end_matches(['%', ' '])
                    .trim()
                    .parse()
                    .ok()?;
                return Some(100u32.saturating_sub(free));
            }
        }
        None
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn limits(rss: u64, pct: u32) -> MemLimits {
        MemLimits {
            max_rss_mb: rss,
            ram_abort_pct: pct,
        }
    }

    #[test]
    fn disabled_limits_never_abort_even_on_huge_sample() {
        let l = limits(0, 0);
        assert!(!l.enabled());
        let s = MemSample {
            rss_mb: Some(u64::MAX),
            host_used_pct: Some(100),
        };
        assert_eq!(evaluate(&l, &s), None);
    }

    #[test]
    fn rss_trips_at_or_above_limit_only() {
        let l = limits(100, 0);
        assert_eq!(
            evaluate(
                &l,
                &MemSample {
                    rss_mb: Some(99),
                    host_used_pct: None
                }
            ),
            None
        );
        assert_eq!(
            evaluate(
                &l,
                &MemSample {
                    rss_mb: Some(100),
                    host_used_pct: None
                }
            ),
            Some(AbortReason::Rss {
                rss_mb: 100,
                limit_mb: 100
            })
        );
        assert_eq!(
            evaluate(
                &l,
                &MemSample {
                    rss_mb: Some(250),
                    host_used_pct: None
                }
            ),
            Some(AbortReason::Rss {
                rss_mb: 250,
                limit_mb: 100
            })
        );
    }

    #[test]
    fn host_pct_trips_at_or_above_limit_only() {
        let l = limits(0, 90);
        assert_eq!(
            evaluate(
                &l,
                &MemSample {
                    rss_mb: None,
                    host_used_pct: Some(89)
                }
            ),
            None
        );
        assert_eq!(
            evaluate(
                &l,
                &MemSample {
                    rss_mb: None,
                    host_used_pct: Some(90)
                }
            ),
            Some(AbortReason::Host {
                used_pct: 90,
                limit_pct: 90
            })
        );
    }

    #[test]
    fn unreadable_sample_never_aborts_even_when_armed() {
        // Both axes armed, but the OS reads failed → None → no abort.
        let l = limits(100, 90);
        assert_eq!(evaluate(&l, &MemSample::default()), None);
    }

    #[test]
    fn rss_is_checked_before_host_when_both_trip() {
        let l = limits(100, 90);
        let s = MemSample {
            rss_mb: Some(200),
            host_used_pct: Some(95),
        };
        assert_eq!(
            evaluate(&l, &s),
            Some(AbortReason::Rss {
                rss_mb: 200,
                limit_mb: 100
            })
        );
    }

    #[test]
    fn host_trips_when_only_host_over() {
        let l = limits(100, 90);
        let s = MemSample {
            rss_mb: Some(10),
            host_used_pct: Some(95),
        };
        assert_eq!(
            evaluate(&l, &s),
            Some(AbortReason::Host {
                used_pct: 95,
                limit_pct: 90
            })
        );
    }

    #[test]
    fn guard_from_default_config_is_disabled_and_byte_identical_path() {
        let guard = MemGuard::from_config(&Config::default());
        assert!(!guard.enabled());
        // Disabled guard short-circuits to None without sampling the OS.
        assert_eq!(guard.check(), None);
    }

    #[test]
    fn guard_from_armed_config_reports_enabled() {
        let cfg = Config {
            max_rss_mb: 4096,
            ..Config::default()
        };
        assert!(MemGuard::from_config(&cfg).enabled());
        let cfg = Config {
            ram_abort_pct: 92,
            ..Config::default()
        };
        assert!(MemGuard::from_config(&cfg).enabled());
    }

    #[test]
    fn abort_message_names_seed_reason_and_knobs() {
        let reason = AbortReason::Rss {
            rss_mb: 200,
            limit_mb: 100,
        };
        let msg = abort_message(&reason, 4242, &Config::default());
        assert!(msg.contains("memory governor abort"));
        assert!(msg.contains("--max-rss-mb 100"));
        assert!(msg.contains("seed=4242"));
        assert!(msg.contains("effective knobs:"));
        // The effective knobs are the real serialized config.
        assert!(msg.contains("\"max_depth\""));
    }

    #[test]
    fn display_messages_are_human_readable() {
        assert_eq!(
            AbortReason::Rss {
                rss_mb: 512,
                limit_mb: 256
            }
            .to_string(),
            "process RSS 512 MiB reached the --max-rss-mb 256 MiB ceiling"
        );
        assert_eq!(
            AbortReason::Host {
                used_pct: 91,
                limit_pct: 88
            }
            .to_string(),
            "host used RAM 91% reached the --ram-abort-pct 88% ceiling"
        );
    }

    #[test]
    fn live_os_reads_are_sane_or_none() {
        // Best-effort: on this dev host they should read SOME plausible
        // value; in a locked-down sandbox they may be None. Never panic;
        // never return an absurd value.
        if let Some(rss) = read_process_rss_mb() {
            assert!(rss < 1_000_000, "RSS {rss} MiB implausibly large");
        }
        if let Some(pct) = read_host_used_pct() {
            assert!(pct <= 100, "used pct {pct} out of range");
        }
    }
}
