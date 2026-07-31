# README Stability Policy

> **ANVIL adoption note.** This is the project-owned copy required by the
> policy's own "Storage location" clause below. **It is authoritative for
> ANVIL** (owner ruling `2026-07-31`): the file it was copied from is the
> **template of origin, not an upstream** — nothing syncs from it, no mechanism
> watches it, and a later edit there has no standing here. Stated explicitly
> because the *absence* of a sync is a deliberate choice that otherwise looks
> exactly like an oversight. Verified at the ruling: **0 of 227** origin-template
> tokens are absent from this copy, so nothing was lost in adoption and there is
> nothing to re-sync. Adopted `2026-07-30` by
> [`README-POLICY-ADOPTION`](docs/tasks/README-POLICY-ADOPTION.md) under an owner
> directive of that date; the reasoning, the per-section audit, and the
> rejected alternatives are in decision
> [`0036`](docs/decisions/0036-readme-landing-page-restoration.md).
>
> **ANVIL's reviewed caps** were derived at decision `0036` §(c) from the
> measured survivors of the audit, deliberately below this policy's
> illustrative numbers. They are stated in that record and are enforced by the
> `README-GROWTH` doctrine — `scripts/check_readme_growth.sh`, registered in
> `scripts/check_doctrines.sh` so the git hook (E3) and CI (E4) both run it
> (see [`DOCTRINE_ENFORCEMENT.md`](DOCTRINE_ENFORCEMENT.md) §10), landing at
> [`README-POLICY-ADOPTION.3`](docs/tasks/README-POLICY-ADOPTION.md). **The
> check is authoritative**, so the numbers are not restated here: decision
> [`0033`](docs/decisions/0033-shadow-enumeration-classification.md) records
> that a number written beside the thing that defines it is one more copy of
> it, and copies fall silently out of date.
>
> **Scope: `README.md` alone.** [`CHANGES.md`](CHANGES.md) and
> [`DEVELOPMENT_NOTES.md`](DEVELOPMENT_NOTES.md) are append-only *by doctrine*
> (decision [`0031`](docs/decisions/0031-ssd-volume-exclusivity.md)) and are
> exempt; [`USER_GUIDE.md`](USER_GUIDE.md)'s length is its purpose. The policy
> is written for a landing page, and ANVIL adopts it for one.
>
> Everything below this note is the project-neutral policy, verbatim.

---

This project-neutral policy keeps a repository README useful as a stable
landing page instead of letting it grow into a changelog, roadmap, or
documentation catalog.

## Storage location

Store the adopting project's canonical copy as the git-tracked
`<repository-root>/README_POLICY.md`, alongside `README.md`. Keeping the policy
with the file it governs gives contributors, local hooks, and CI one
discoverable, versioned source of truth. A user-home, machine-global, or other
external copy may serve as a reusable template, but it must not replace the
project-owned repository copy.

## Content contract

Keep only information a first-time visitor needs:

- purpose, audience, and top-level scope;
- prerequisites and one minimal verified quick start;
- stable architecture at a glance;
- links to canonical documentation, support, and contribution guidance;
- license and other essential repository-level notices.

Route changing detail elsewhere:

| Content | Canonical home |
| --- | --- |
| User-facing feature detail and examples | User guide or product manual |
| Current work, priorities, and roadmap status | Roadmap, issue tracker, or task system |
| Release history | Releases, changelog, or git history |
| Design rationale | Decision records or architecture docs |
| Exhaustive file/API/sample inventories | Generated indexes or dedicated references |
| Diagnostics and operational procedures | Troubleshooting or contributor docs |

Change the README only when its purpose, first-use path, top-level architecture,
or canonical navigation changes. Ordinary feature work should update the
canonical destination, not the README.

## Mechanical growth guard

Enforce both a line cap and a byte cap. Choose them after a deliberate review
and trim, leaving only modest headroom. Never raise a cap merely to land new
content; move the detail to its canonical home. A cap increase requires an
explicit reviewed decision that the landing-page contract itself expanded.

A minimal deterministic check is:

```sh
line_cap=300
byte_cap=16384
lines=$(wc -l < README.md | tr -d ' ')
bytes=$(wc -c < README.md | tr -d ' ')
test "$lines" -le "$line_cap"
test "$bytes" -le "$byte_cap"
```

Keep the check non-mutating, return nonzero with a routing hint on failure, and
run it in both the local pre-commit hook and CI. Line and byte checks complement
each other: neither wrapped prose nor very long lines can bypass the budget.

## Adoption checklist

1. Add and commit `<repository-root>/README_POLICY.md` beside `README.md`.
2. Remove duplicated status, history, inventories, and deep reference prose.
3. Verify the retained quick start and links.
4. Record where each excluded content class belongs.
5. Set reviewed line and byte caps with modest headroom.
6. Commit the deterministic check and wire it into pre-commit and CI.
7. Require an explicit decision before either cap can increase.
