# LOCAL-REFERENCE-CACHE: untracked per-machine grounding material

## Metadata

- Tree ID: `LOCAL-REFERENCE-CACHE`
- Status: `active`
- Roadmap lane: `Workflow / tooling — local LRM grounding material`
- Created: `2026-06-16`
- Last updated: `2026-06-16` (**`.1` landed** — gitignore `.cache/` for the
  owner-provided IEEE 1800-2017/2023 SystemVerilog LRM Markdown.)
- Owner: repo-local workflow

## Goal

Give agents a per-machine, **untracked** local copy of authoritative reference
material (starting with the SystemVerilog LRM) so they can `grep` the spec
before making legality/synthesizability/version claims — instead of relying on
model memory — while never letting that bulky material enter the repo's signoff
surface.

## Non-Goals

- Not committing the reference material itself (it is bulky, third-party, and
  per-machine; it stays under the gitignored `.cache/`).
- Not a substitute for the live docs / book — this is raw grounding material,
  not project documentation.

## Acceptance Criteria

- `.cache/` is gitignored so the material can never be accidentally tracked.
- A provenance note records what the material is and where it came from.
- The cache is recorded as a durable `reference` so future sessions consult it.

## Task Tree

- ID: `LOCAL-REFERENCE-CACHE`
  Status: `active`
  Goal: `Per-machine untracked reference cache for agent grounding (SV LRM first).`
  Children: `LOCAL-REFERENCE-CACHE.1`

- ID: `LOCAL-REFERENCE-CACHE.1`
  Status: `done`
  Goal: `Gitignore .cache/ and land the owner-provided IEEE 1800-2017/2023 SystemVerilog LRM Markdown under .cache/local-references/sv/ with a provenance README; record it as a reference auto-memory.`
  Acceptance: `.cache/ is gitignored (git check-ignore confirms); the LRM Markdown is present and greppable under .cache/local-references/sv/{2017,2023}/; a provenance README exists; self-checks clean; committed through COMMIT.md.`
  Result: `Done. .gitignore ignores .cache/. The owner provided the IEEE 1800-2017 (59 section .md files) and 1800-2023 (58 section .md files) LRM as per-section Markdown, copied from /Users/richarddje/Documents/github/pgen/docs/systemverilog/{2017,2023}/md to .cache/local-references/sv/{2017,2023}/ with a provenance README.md. git check-ignore confirms the tree is ignored (absent from git status). Recorded as the reference_sv_lrm_local_cache auto-memory + MEMORY.md index line. Markdown preferred over the also-available PDF (greppable). Per-machine — a fresh clone re-copies from source.`
  Verification: `git check-ignore .cache/local-references/sv/2017/section-1-overview.md => ignored; git status shows no .cache entries; grep confirms 2023 §7 union-soft + §13 function-automatic material present; bash scripts/check_memory_architecture.sh + bash knowledge-map/scripts/check_knowledge_map.sh clean. No source touched.`
  Commit: `done`

## Current Frontier

No active frontier. `.1` is done. Future reference caches (additional specs,
tool manuals) would land as new `.N` leaves; none are queued.

| Order | Leaf | Status | Why |
| --- | --- | --- | --- |
| — | `LOCAL-REFERENCE-CACHE.1` | `done` | Gitignore `.cache/` + land the SV LRM (1800-2017/2023) Markdown under `.cache/local-references/sv/` + provenance README + `reference` auto-memory. |

## Decisions

- `2026-06-16` (`.1`): the reference material stays **untracked** under
  `.cache/` (bulky, third-party, per-machine), never part of the repo. Markdown
  is preferred over the also-available PDF because it is greppable and directly
  readable. Provenance is recorded both in-cache (`README.md`) and as a durable
  `reference` auto-memory so a fresh session knows the cache exists and how to
  refresh it.

## Open Questions

- None.

## Blockers

- None.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-06-16` | `LOCAL-REFERENCE-CACHE.1` | `.gitignore` `.cache/` entry; `git check-ignore` confirms the LRM tree is ignored and absent from `git status`; LRM Markdown present + greppable (`2023` §7 `union soft`, §13 `function automatic`); provenance `README.md`; `reference_sv_lrm_local_cache` auto-memory + index line; `bash scripts/check_memory_architecture.sh` + `bash knowledge-map/scripts/check_knowledge_map.sh` clean. No source touched. | `done` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `LOCAL-REFERENCE-CACHE.1` | `LOCAL-REFERENCE-CACHE.1 — gitignore .cache/ for local SV LRM` | Gitignore `.cache/`; owner-provided IEEE 1800-2017/2023 LRM Markdown lands untracked under `.cache/local-references/sv/` with a provenance README; recorded as a `reference` auto-memory. No code / no RTL change. |

## Changelog

- `2026-06-16`: Created and landed `.1` — the untracked per-machine SV LRM
  reference cache (`.cache/local-references/sv/{2017,2023}/`) + `.gitignore`
  entry + provenance README + `reference` auto-memory. Workflow/tooling lane;
  no code change. Future reference caches land as new `.N` leaves.
