---
id: ssd-volume-exclusivity
title: ANVIL stores and references nothing on the boot volume — the SSD is the only project volume
answers:
  - "can ANVIL write to /tmp"
  - "where does ANVIL store its working data"
  - "is the boot volume allowed"
  - "can the project reference Macintosh HD"
  - "do CARGO_HOME and RUSTUP_HOME move to the SSD"
  - "why is ANVIL_SANDBOX_ROOT set"
  - "what volume must project data live on"
  - "is cross-volume access ever allowed"
  - "are shared $HOME stores a policy violation"
date: 2026-07-29
status: current
tags: [doctrine, data-locality, volume, paths, portability, owner-directive]
evidence: docs/tasks/VOLUME-DATA-LOCALITY.md; src/paths.rs; CHANGES.md
reverify: "grep -rn 'temp_dir()' src/ tests/  → matches only src/paths.rs (the OS temp dir is named in exactly one place)"
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
   or `$TMPDIR`. (Shared machine-wide stores under `$HOME` are a
   separate category — see the clarification below.)
2. **Reference.** No tracked file may *point at* a boot-volume path as
   though it were live project data — including prose. Historical
   citations must be rewritten or explicitly labelled as dead
   breadcrumbs (this is what makes `EVIDENCE-BANK-DURABILITY.4`
   mandatory rather than tidy-up).

### The hard limit on obligation 2: HISTORY IS NEVER REWRITTEN

**Owner directive (`2026-07-29`), absolute and permanent:** *"NO, no
history rewrite, no at all. I do not want to mess with the git history.
Keep it raw, keep honest, so that people can follow the whole history if
they want to."*

Obligation 2 applies to documents that describe the project's **current
state**. It stops at the record of what happened. Specifically:

- **`git` history is never rewritten.** No `rebase`, no `--amend` of a
  landed commit, no `reset --hard` over published work, no
  `filter-branch`/`filter-repo`, no force-push. History is append-only.
  A commit that said `/tmp/anvil-…` in 2026-06 keeps saying it.
- **`CHANGES.md` and `DEVELOPMENT_NOTES.md` are never retro-edited.**
  They are the human-readable audit trail (`MEMORY_ARCHITECTURE.md`
  layers C/D: *append once, supersede, never silently rewrite*). Their
  ~566 pre-0031 boot-volume references stay exactly as written.
- **Policy documents keep the strings they forbid** — this record,
  `0002`, `0030`, and the two owning task trees. A rule that cannot name
  what it prohibits is unreadable.

The reasoning, which outranks tidiness: a swept history is a **dishonest**
history. Someone auditing why the evidence banks evaporated must be able
to read the entries that cited `/tmp` and see the mistake as it was
actually made. Rewriting them to look compliant would erase the very
evidence that justifies this decision record existing.

This limit was learned concretely: a first, allow-list-free sweep
rewrote decision `0030`'s own `reverify` command from
`ls -d /tmp/anvil-*` to `ls -d anvil-*` — turning a re-runnable check
into nonsense — and had to be reverted. Mechanically rewriting a
document whose *subject* is the string being rewritten destroys it.

### What the rule does NOT cover: shared `$HOME` stores

**Owner clarification (`2026-07-29`): "everything shared in `$HOME`
stays in `$HOME`."** The directive governs data *this project owns*, not
the machine-wide stores it merely uses. So the following stay exactly
where they are and are **not** violations:

- `~/.cargo` (crate registry, git checkouts, installed binaries) and
  `~/.rustup` (toolchains) — shared across all the owner's projects.
  `CARGO_HOME` / `RUSTUP_HOME` are deliberately **left at their
  defaults**; an earlier plan to repoint them onto the repo volume was
  **withdrawn** on this clarification, and the 3 GB of SSD copies it had
  made were removed. This also aligns with §13's standing rule never to
  fragment an ambiguous shared global cache.
- `$HOME/.cargo/bin` in `.github/workflows/*.yml` — those run on
  GitHub's runners, where `$HOME` is the runner's own disk. Correct and
  portable as written.
- Compilers and downstream tools installed by the OS package manager
  (`verilator` / `yosys` / `iverilog` under `/opt/homebrew`). ANVIL
  **executes** these; it stores nothing in them — §13's toolchain
  carve-out.

The line is therefore: **data ANVIL creates → the repo volume; stores
ANVIL merely reads or executes → wherever the system keeps them.**

And the rule for shared things, in the owner's words (`2026-07-29`,
final): *"If something is shared, then please use the shared
information, no need to duplicate the information that is shared on the
SSD."* **Shared means shared — use it, do not fork it.** Copying a
shared store onto the repo volume would burn the disk twice over *and*
create a second cache that silently drifts from the one every other
project uses; that is the same second-source-of-truth failure the
project's full-factorization discipline forbids in its own code.

Honest limit, stated rather than glossed: the **agent harness** writes
its own runtime files (task output, tool results) under
`/private/tmp/claude-501/…` and `~/.claude/…`. Those paths are the
harness's, not the repository's, and no in-repo setting can move them.
The mitigation is that no ANVIL data depends on them: every command that
matters redirects its real output to `.cache/scratch/` on the SSD, so
losing the harness's files loses nothing.

### Shared global caches are left alone

`~/.cargo` (845M) and `~/.rustup` (2.1G) are shared with the owner's
other projects, so per the `$HOME` clarification they are neither
relocated nor deleted — the project simply keeps using them. The owner separately
confirmed (`2026-07-29`) that `~/Documents/github` — the pre-move
checkouts — is deliberate owner-owned data to be removed by the owner
themselves, never by an agent, and excluded from every audit.

## Consequences

- `ANVIL_SANDBOX_ROOT` is the project's one environment knob for where
  its working data lands; `CARGO_HOME` / `RUSTUP_HOME` stay at their
  defaults per the `$HOME` clarification above.
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
- Builds are unchanged: they keep using the shared `~/.cargo` /
  `~/.rustup`, so there is no cold-registry cost and no duplicated
  toolchain.

## Links

- Task tree: `docs/tasks/VOLUME-DATA-LOCALITY.md`
- Code: `src/paths.rs` (the runtime resolver)
- Standards: `CLAUDE.md` §12 (repo-root-relative paths), §13 (data
  locality), `MEMORY_ARCHITECTURE.md` §2 (durability properties)
