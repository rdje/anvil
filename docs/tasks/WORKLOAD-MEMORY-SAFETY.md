# WORKLOAD-MEMORY-SAFETY: ANVIL runs never exhaust host RAM

## Metadata

- Tree ID: `WORKLOAD-MEMORY-SAFETY`
- Status: `active`
- Roadmap lane: `Quality / signoff — resource-safe generation (ANVIL's own runtime)`
- Created: `2026-06-14`
- Last updated: `2026-06-14`
- Owner: repo-local workflow

## Goal

Make ANVIL's own generation runs incapable of driving a RAM-limited host
toward the danger/reboot zone, even on huge workloads — a very large
`--count`, or a single pathologically deep/wide module/design. Deliver
**bounded-memory generation**, **streamed/chunked output**, and an
**internal node/RAM governor** so the `anvil` process itself stays inside
a safe envelope and fails cleanly (with the seed + effective knobs)
rather than letting the OS OOM-kill it or reboot the machine.

This complements `RESOURCE-SAFE-TOOLING` (`scripts/ram_guard.sh`), which
guards *external* heavy jobs (cargo builds/tests, `tool_matrix` sweeps).
This tree guards `anvil`'s *own* process from the inside.

## Non-Goals

- **No change to generated RTL by default.** Every mechanism here is
  default-off / byte-identical-preserving, exactly like every prior
  capability knob (`multi_clock_prob`, `aggregate_prob`, `memory_prob`,
  `fsm_prob` all defaulted to the no-op value). A snapshot/`book_examples`
  change is only acceptable as a deliberate, separately-reviewed act per
  the `INSTA-SNAPSHOTS` protocol — and is explicitly out of scope for the
  default path.
- **Not a correctness mechanism.** Valid-by-construction is untouched: no
  generate-then-filter, no post-hoc truncation of a partially-built cone
  (that would emit invalid RTL). Bounding happens at *construction-choice*
  time (rules-first) or by *declining to start* more work — never by
  mutilating a finished structure.
- **Not a replacement for `scripts/ram_guard.sh`.** The external watchdog
  stays the right tool for cargo/tool_matrix. This tree does not touch it.
- **Not a distributed/streaming-to-network feature.** Output streaming
  here means "do not accumulate the whole run's metadata/modules in RAM
  before writing to the `--out` directory", nothing more.

## Acceptance Criteria

- ANVIL can run an arbitrarily large `--count … --out DIR` workload with
  **bounded** peak process memory w.r.t. `--count` (the per-module/peak
  cost no longer grows linearly in the number of artifacts produced).
- A per-module **construction-time node budget** exists and is actually
  enforced (the current `max_nodes_per_module` ghost knob is wired up or
  replaced), rules-first, with a default that preserves byte-identical
  output.
- An **internal RAM/RSS self-governor** can abort a run cleanly (clear
  message naming the seed + effective knobs, deterministic exit code)
  before the host crosses a configurable danger threshold — default off,
  byte-identical when unset.
- Default `anvil` invocations remain byte-identical (snapshots +
  `tests/book_examples.rs` unchanged); each landed leaf proves this.
- Live docs (`USER_GUIDE.md`, the relevant `book/src/*.md`,
  `CODEBASE_ANALYSIS.md`) and this tree are kept in sync per `COMMIT.md`.

## Task Tree

- ID: `WORKLOAD-MEMORY-SAFETY`
  Status: `active`
  Goal: `ANVIL's own runs stay inside a safe RAM envelope on huge workloads.`
  Children: `WORKLOAD-MEMORY-SAFETY.1`, `WORKLOAD-MEMORY-SAFETY.2`, `WORKLOAD-MEMORY-SAFETY.3`, `WORKLOAD-MEMORY-SAFETY.4`, `WORKLOAD-MEMORY-SAFETY.5`

- ID: `WORKLOAD-MEMORY-SAFETY.1`
  Status: `done`
  Goal: `Audit ANVIL's runtime memory drivers and record the bounded-memory design (mechanisms, defaults, byte-identical strategy, leaf shape).`
  Acceptance: `DEVELOPMENT_NOTES.md carries the memory-safety design rationale; this tree's Decisions/Open-Questions capture the policy; the data-flow + every existing size-bounding knob + the ghost-knob finding are documented with file:line evidence. Docs-only; no code change.`
  Verification: `done — see Verification Log`
  Commit: `WORKLOAD-MEMORY-SAFETY.1 - audit + bounded-memory design`

- ID: `WORKLOAD-MEMORY-SAFETY.2`
  Status: `pending`
  Goal: `Stream the directory-output manifest so a huge --count does not accumulate all per-artifact metadata in RAM.`
  Acceptance: `For --out DIR runs the in-memory manifest/designs accumulation is bounded (incremental write), manifest.json stays byte-identical to today's output, default behaviour unchanged; focused test + a large-count smoke prove bounded growth; snapshots + book_examples byte-identical.`
  Verification: `pending`
  Commit: `pending`

- ID: `WORKLOAD-MEMORY-SAFETY.3`
  Status: `pending`
  Goal: `Turn the per-module node budget into a real, rules-first construction-time cap (wire up / replace the max_nodes_per_module ghost knob).`
  Acceptance: `A construction-time node budget is enforced rules-first (prefer terminal reuse / stop opening new sub-cones near budget, never truncate a finished cone); default preserves byte-identical generated RTL (default = unlimited or a value provably ≥ all current outputs); a metric measures it (knob-effectiveness doctrine); validation + focused tests; snapshots + book_examples byte-identical at default.`
  Verification: `pending`
  Commit: `pending`

- ID: `WORKLOAD-MEMORY-SAFETY.4`
  Status: `pending`
  Goal: `Add an opt-in internal RAM/RSS self-governor that aborts a run cleanly before the host danger zone.`
  Acceptance: `An opt-in knob (e.g. --max-rss-mb / --ram-abort-pct) makes anvil stop with a deterministic non-zero exit code and a message naming the seed + effective knobs before crossing the threshold; default off ⇒ byte-identical; cross-platform RAM/RSS read (macOS + Linux) consistent with ram_guard.sh's approach; focused tests for the decision logic.`
  Verification: `pending`
  Commit: `pending`

- ID: `WORKLOAD-MEMORY-SAFETY.5`
  Status: `pending`
  Goal: `Closeout: sync USER_GUIDE + book + CODEBASE_ANALYSIS + roadmap status, record deferred boundaries, close the tree.`
  Acceptance: `USER_GUIDE "Resource-safe runs" extended with the internal governor/bounded-output knobs; the relevant book chapter(s) describe the safe-envelope contract; CODEBASE_ANALYSIS reflects the new modules/knobs; tree closed with explicit deferred-boundary record.`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `WORKLOAD-MEMORY-SAFETY.2` | `pending` | Lowest-risk, clearest win: the manifest/designs Vec is the unambiguous unbounded-O(`--count`) growth point; can be made byte-identical. |
| 2 | `WORKLOAD-MEMORY-SAFETY.3` | `pending` | Per-module construction-time node budget; needs the `.1` default-preserving policy settled first. |
| 3 | `WORKLOAD-MEMORY-SAFETY.4` | `pending` | Internal RAM/RSS self-governor; opt-in, independent of `.2`/`.3`. |
| 4 | `WORKLOAD-MEMORY-SAFETY.5` | `pending` | Closeout once the mechanisms land. |

## Decisions

- `2026-06-14` (`.1`): **Every mechanism is default-off / byte-identical.**
  This mirrors the established pattern for every capability knob in ANVIL
  (multi_clock / aggregate / memory / fsm all default to the no-op value)
  and the non-negotiable reproducibility contract (`book/src/knobs.md`
  "Reproducibility"). The default `anvil` (and `--artifact dut`) output
  must stay byte-identical so `tests/snapshots.rs` and
  `tests/book_examples.rs` remain green without snapshot acceptance.
- `2026-06-14` (`.1`): **Bounding is construction-time or decline-to-start,
  never truncation.** Rules-first doctrine (`feedback_rules_first_generation`):
  ANVIL may steer cone construction toward terminals as a node budget is
  approached, or decline to begin another module/design, but it must never
  emit a partially-built / mutilated cone — that would produce invalid RTL
  and break valid-by-construction.
- `2026-06-14` (`.1`): **`max_nodes_per_module` is a ghost knob** — it is
  declared (`src/config.rs:337`) and defaulted to `1000`
  (`src/config.rs:729`) but never read or enforced anywhere in generation
  (grep proves only those two occurrences). `.3` either wires it up with a
  sentinel meaning "unlimited" (preferred, default-preserving) or replaces
  it with a dedicated opt-in budget knob. Because it is currently inert,
  enforcing it at its present default of `1000` WOULD change output for any
  module that exceeds 1000 nodes — so the default must become "unlimited"
  (sentinel `0`) to preserve byte-identical RTL; only `--dump-config` /
  `manifest.json` config echo would shift (not SV output, not SV
  snapshots).
- `2026-06-14` (`.1`): **`.2` keeps manifest.json byte-identical** by
  streaming the *same* JSON array structure incrementally (open
  bracket → comma-separated elements → close bracket) rather than building
  a `Vec<serde_json::Value>` and pretty-printing it at the end. The bytes
  on disk must match today's `serde_json::to_string_pretty` output exactly.
- `2026-06-14` (`.1`): **`.4` reuses `ram_guard.sh`'s OS-read approach**
  (macOS `memory_pressure` "free percentage"; Linux `/proc/meminfo`
  `MemAvailable`) for the host-pressure read, and additionally exposes a
  process-RSS bound, so a single pathological module is also covered (host
  %-used can lag a fast single-process balloon).

## Open Questions

- `.3`: should the node budget be a hard *cap* (decline to grow further,
  steer to terminals) or also a *soft* signal feeding share/terminal-reuse
  probabilities? Default-unlimited makes this non-blocking; resolve at `.3`
  implementation with a focused metric. Owner: repo-local.
- `.4`: knob spelling/semantics — `--max-rss-mb` (absolute, per-process,
  portable) vs `--ram-abort-pct` (host %-used, matches ram_guard). Leaning
  toward supporting both, RSS as the primary single-process guard. Does not
  block the frontier (`.2` is next). Owner: repo-local.
- `.2`: should the streamed manifest also gain an opt-in JSON-lines
  sidecar for truly huge runs, or is incremental-array streaming enough?
  Resolve at `.2`; incremental-array is the byte-identical default.

## Blockers

- None.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-06-14` | `WORKLOAD-MEMORY-SAFETY.1` | Codebase memory-behaviour audit (Explore survey + direct verification: `grep -rn max_nodes_per_module src/` → only `config.rs:337` decl + `config.rs:729` default; `src/main.rs:507-575` output paths read directly). Docs-only; design recorded in `DEVELOPMENT_NOTES.md` + this tree. memory-architecture + knowledge-map self-checks (pre-commit). `git diff --check`. Full `cargo test` intentionally skipped (no code change; full-suite RAM risk per `docs/decisions/0003-resource-safe-validation.md`). | passed (docs-only) |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `WORKLOAD-MEMORY-SAFETY.1` | `WORKLOAD-MEMORY-SAFETY.1 - audit + bounded-memory design` | Tree genesis + design leaf. Pending hash. |

## Changelog

- `2026-06-14`: Created tree; landed `.1` (audit + bounded-memory design, docs-only). Frontier now `.2` (streaming manifest).
