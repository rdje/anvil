---
id: reference-cache-stays-untracked-public-repo
title: The local LRM cache is **never** tracked — `rdje/anvil` is a **public** repo and the material is `Copyright © 2024 IEEE. All rights reserved.`; under [`0031`](0031-ssd-volume-exclusivity.md)'s no-history-rewrite rule a mistaken commit would be **permanent**
answers:
  - "may I commit the SystemVerilog LRM into the repo"
  - "why is .cache/local-references not tracked"
  - "is the anvil repository public or private"
  - "can I git add -f the reference cache"
  - "how do I make the LRM durable without committing it"
  - "what may I cite from the LRM cache in a tracked file"
  - "why can a mistaken commit not simply be removed from this repo"
date: 2026-08-01
status: accepted
tags: [reference-cache, licensing, copyright, git, public-repo, irreversibility, owner-directive]
reverify: "gh repo view rdje/anvil --json visibility  ->  PUBLIC ; git check-ignore -v .cache/local-references/sv/README.md  ->  matched by .gitignore:19"
evidence: "measured 2026-08-01 — gh reported {\"isPrivate\":false,\"visibility\":\"PUBLIC\"}; the cache is 6,721,089 bytes across 118 Markdown files; section-1-overview.md of the 2023 edition carries 'Copyright (c) 2024 IEEE' and 'All rights reserved'; .gitignore:19 already excludes .cache/"
---

# Context

The owner asked for the cached IEEE 1800-2017/2023 LRM Markdown under
`.cache/local-references/sv/` to be copied into the repository and git-tracked,
clarifying the intent as *"just for reference."* Measured before acting:

| fact | value |
| --- | --- |
| `rdje/anvil` visibility | **PUBLIC** |
| material | **6,721,089 B** across **118** files |
| notice in the text | *"Copyright © 2024 IEEE"*, *"All rights reserved"* |
| already excluded? | yes — `.gitignore:19` (`.cache/`) |

# Decision

**The reference cache is never tracked, and never force-added.** Reference-only
intent does not change the analysis: committing to a repo with a public GitHub
remote *publishes* the material, and that is redistribution of a copyrighted
standard.

**The owner confirmed this reasoning on `2026-08-01`** and withdrew the request,
so this record states an agreed boundary rather than an agent's refusal.

# Why this is recorded rather than left to judgement

Three reasons, and the third is the load-bearing one:

1. **`.gitignore` stops the accident, not the intention.** `git add -f` bypasses
   it, and an agent asked directly to track the files may reach for exactly that.
   The *reason* is what refuses; the ignore rule only refuses by default.
2. **Repo visibility is not self-evident from inside the checkout.** It takes a
   `gh repo view` to learn it, and an agent that does not think to ask will
   assume whatever is convenient. The `reverify:` line above makes it one command.
3. **A mistake here cannot be undone in this project.** Decision
   [`0031`](0031-ssd-volume-exclusivity.md) forbids rewriting history — no
   `filter-branch`, no force-push. The standard remedy for "copyrighted material
   committed by mistake" is therefore **prohibited by the project's own standing
   directive**, which makes this one of the few genuinely irreversible actions
   available in this repo. That asymmetry is the whole argument for writing it
   down: the cost of the guardrail is one file; the cost of the mistake is
   permanent.

# No new doctrine

`.gitignore:19` already mechanises exclusion, and `feedback_full_factorization`
forbids a **second** registered mechanism for a job that already has one — the
same test applied at
[`OVERFLOW-DESTINATION-INSTRUMENTATION.7`](../tasks/OVERFLOW-DESTINATION-INSTRUMENTATION.md)
before registering `TABLE-RENDER-FIDELITY`, and at `.3` before *declining* to
register one for `MEMORY.md`'s byte cap. The registry stays at **9**. What was
missing was never a check — it was the **reason**, and a reason belongs in
layer C.

# What IS allowed, and how to make the reference durable

The copyrighted expression is the **prose**. These are not it, and may be tracked
freely:

- **Clause and section citations** — e.g. *"the `union soft` heterogeneous-width
  packed union, IEEE 1800-2023 §7.3.1"*. Cite the clause; never paste the text.
- **A manifest** — the expected directory layout, the 118 section filenames, a
  `sha256` per file, and the refresh procedure. That makes a refreshed copy
  **verifiable** rather than assumed, and it survives the cache being wiped.
- **Derived facts about ANVIL's own emission** — e.g. which constructs ANVIL
  emits and which clause governs each.

The manifest was offered and not taken up on `2026-08-01`; it remains the
recommended way to close the durability gap if the cache's provenance path
(a sibling local checkout) ever goes away. **Recorded here so the option is not
re-invented from scratch.**

# Consequences

1. `.cache/local-references/` stays untracked, permanently. Any future request to
   track it is answered by this record, not re-litigated from first principles.
2. A tracked file may cite the LRM **by clause number**, and must never quote its
   prose at length.
3. The cache's own `README.md` lives inside the untracked cache, so it disappears
   with it — which is precisely why this record, not that README, is the durable
   home for the rule.
4. Generalisation for any project adopting these standards: **check the remote's
   visibility before tracking any third-party material, and check whether your
   own doctrine forbids the remedy before assuming a bad commit is fixable.**
