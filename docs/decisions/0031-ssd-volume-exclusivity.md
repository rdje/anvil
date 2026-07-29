---
id: ssd-volume-exclusivity
title: ANVIL stores and references nothing on the boot volume — the SSD is the only project volume
answers:
  - "can ANVIL write to /tmp"
  - "where does ANVIL store its working data"
  - "is the boot volume allowed"
  - "can the project reference Macintosh HD"
  - "where is CARGO_HOME for this project"
  - "why is ANVIL_SANDBOX_ROOT set"
  - "what volume must project data live on"
  - "is cross-volume access ever allowed"
  - "how do I run ANVIL builds on the right volume"
date: 2026-07-29
status: current
tags: [doctrine, data-locality, volume, paths, portability, owner-directive]
evidence: docs/tasks/VOLUME-DATA-LOCALITY.md; src/paths.rs; CHANGES.md
reverify: "grep -rn 'temp_dir()' src/ tests/  → matches only src/paths.rs;  echo $CARGO_HOME  → a path under the repo volume;  df -h . | awk 'NR==2{print $1}'  → the repo's device"
---

# 0031 - The SSD is the only project volume

- Date: 2026-07-29
- Status: accepted
- Tags: doctrine, data-locality, volume, owner-directive
- Owning tree: `VOLUME-DATA-LOCALITY`
- Supersedes nothing; **amends** [`0002`](0002-live-doc-path-portability.md)
  and [`0030`](0030-durable-closure-evidence-citations.md) (see below)

## Context

The owner moved every project onto a 4TB SSD and issued a **standing,
absolute directive** (`2026-07-29`, stated twice for emphasis):

> "All your files, gates, everything shall point and be located on this
> SSD volume. Nothing should reference or point into the Macintosh HD,
> the boot volume ever again."
>
> "Do not ever reference or store on Macintosh HD ever again."

Measured on this machine: the repository is `/dev/disk5s1`
(`/Volumes/SSD`); `/tmp`, `/private/tmp`, `$HOME`, and the per-user
`TMPDIR` (`/var/folders/…`) are all `/dev/disk3s1s1`, the boot volume.

This is **stronger than CLAUDE.md §13**, which permitted documented
read-only cross-volume access. §13 governed where project data is
*written*; this directive governs **references too** — a doc that points
at `/tmp/anvil-…` violates it even though nothing executes that string.

## Decision

**The SSD is the only volume this project stores on or points at.** Two
obligations, both absolute:

1. **Storage.** Every byte the project owns — outputs, build artifacts,
   caches, dependency/package stores, logs, test fixtures, temporary
   workspaces, agent scratch — is written under the repository volume,
   at a path derived at runtime from the current root
   ([`src/paths.rs`](../../src/paths.rs)). Never `/tmp`, `/private/tmp`,
   `$TMPDIR`, `$HOME`, or any other boot-volume location.
2. **Reference.** No tracked file may *point at* a boot-volume path as
   though it were live project data — including prose. Historical
   citations must be rewritten or explicitly labelled as dead
   breadcrumbs (this is what makes `EVIDENCE-BANK-DURABILITY.4`
   mandatory rather than tidy-up).

### The only admitted exception: the installed toolchain

Compilers and downstream tools are installed by the OS package manager
and cannot be relocated by this repository: `rustc`/`cargo` under
`~/.rustup`, and `verilator` / `yosys` / `iverilog` under
`/opt/homebrew`. ANVIL **executes** these; it does not store data in
them. This is explicitly identified, execute/read-only, and documented
here as required by §13's toolchain carve-out.

Everything the project can control is moved: `CARGO_HOME` and
`RUSTUP_HOME` are repointed onto the repository volume so the crate
registry, git checkouts of dependencies, and installed toolchains are
project-local rather than shared boot-volume stores.

Honest limit, stated rather than glossed: the **agent harness** writes
its own runtime files (task output, tool results) under
`/private/tmp/claude-501/…` and `~/.claude/…`. Those paths are the
harness's, not the repository's, and no in-repo setting can move them.
The mitigation is that no ANVIL data depends on them: every command that
matters redirects its real output to `.cache/scratch/` on the SSD, so
losing the harness's files loses nothing.

### Shared global caches are populated, never deleted

`~/.cargo` (845M) and `~/.rustup` (2.1G) are shared with the owner's
other projects. Per §13 they are **not deleted**: a project-local store
is populated, the shared copy stops being accessed, and only
provably-project-owned records are ever removed. The owner separately
confirmed (`2026-07-29`) that `~/Documents/github` — the pre-move
checkouts — is deliberate owner-owned data to be removed by the owner
themselves, never by an agent, and excluded from every audit.

## Consequences

- `ANVIL_SANDBOX_ROOT`, `CARGO_HOME`, and `RUSTUP_HOME` are part of the
  project's environment contract; the repo documents them and the agent
  harness sets them.
- A doctrine check (`scripts/check_no_boot_volume_refs.sh`, registered in
  `scripts/check_doctrines.sh`) fails on a new boot-volume reference in
  tracked files, so compliance is mechanical rather than remembered.
- Decision [`0002`](0002-live-doc-path-portability.md) is **amended**:
  its allowance that "banked artifacts outside the repository, such as
  `/tmp/anvil-…`, may remain absolute" is **withdrawn**. That allowance
  is what let the boot-volume citations accumulate.
- Decision [`0030`](0030-durable-closure-evidence-citations.md) is
  **amended**: its rejected option (iii) — relocating banks out of
  `/tmp` — is now **partly mandated**. A future bank must be
  repo-derived and on-volume; the committed digest remains the citation
  form, so both mechanisms apply rather than one replacing the other.
- Builds run against a cold project-local registry the first time,
  costing one re-download; thereafter they are volume-correct and
  unaffected by anything on the boot disk.

## Links

- Task tree: `docs/tasks/VOLUME-DATA-LOCALITY.md`
- Code: `src/paths.rs` (the runtime resolver)
- Standards: `CLAUDE.md` §12 (repo-root-relative paths), §13 (data
  locality), `MEMORY_ARCHITECTURE.md` §2 (durability properties)
