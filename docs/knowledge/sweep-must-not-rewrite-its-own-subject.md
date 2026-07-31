---
id: sweep-must-not-rewrite-its-own-subject
title: A repo-wide string sweep must allow-list the documents whose **subject** is that string and must anchor a path rewrite to a path start — an unanchored sweep once rewrote a decision record's own `reverify` command into nonsense and minted ten paths to a directory that never existed
answers:
  - "how do I safely run a repo-wide string replacement"
  - "why did a sweep break a decision record's reverify command"
  - "which documents must be excluded from a mass rewrite"
  - "why did a path rewrite fire inside target/tmp"
  - "how do I write a gate for a path rule without inheriting the sweep's blind spot"
  - "is it safe to mass-rewrite /tmp references"
date: 2026-07-30
status: current
tags: [sweep, refactor, docs, paths, gotcha, history, doctrine]
reverify: "bash scripts/check_no_boot_volume_refs.sh"
evidence: docs/decisions/0031-ssd-volume-exclusivity.md (the damaged `reverify`; the load-bearing allow-list); docs/tasks/CARGO-TMPDIR-SWEEP-REGRESSION.md (the unanchored-prefix regression); scripts/check_no_boot_volume_refs.sh
---

Two failures of the same sweep, both worth knowing before writing another one.

**1. Never mass-rewrite a string across documents whose *subject* is that string.** A blanket
`/tmp` sweep rewrote decision [`0030`](../decisions/0030-durable-closure-evidence-citations.md)'s
own `reverify` command into the meaningless `ls -d anvil-*`. A policy document must be allowed to
name what it forbids, and append-only history must stay raw
([`0031`](../decisions/0031-ssd-volume-exclusivity.md)) — so **allow-list the policy and history
documents first**. This is why `check_no_boot_volume_refs.sh`'s allow-list is load-bearing rather
than an exception: the gap between it and "every tracked file" *is* the rule, which is also why it
is not a shadow enumeration under [[shadow-enumeration-classification]]'s test (2).

**2. Anchor a path-prefix rewrite to a path start.** Applied unanchored, the same sweep fired
*inside* `target/tmp/…` — Cargo's `CARGO_TARGET_TMPDIR`, which is on-volume and correct — and
minted **ten** paths to a directory that never existed. **A wrong path that looks plausible
survives review.**

**And the corollary that matters most:** a gate built from the sweep's own search string inherits
the sweep's blind spot. Write the check from the **property** (*"a boot-volume path is
absolute"*), never from the string the sweep happened to grep for.

See also [[coverage-check-vacuity]] for the acceptance test that catches a check which passes
while proving nothing.
