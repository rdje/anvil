---
id: doctrine-check-must-classify-not-guess
title: A doctrine check must **classify** and fail closed on an unknown, never guess — a heuristic either misses a real breach or cries wolf on prose, and **a gate that cries wolf gets deleted, taking its real coverage with it**
answers:
  - "should a doctrine check use a heuristic or an explicit classification"
  - "why does EVIDENCE-CITATIONS fail on an unclassified token"
  - "why is a grandfathered list pinned by count and SHA"
  - "what happens to a gate that produces false positives"
  - "how do I stop a grandfather list from becoming an escape hatch"
date: 2026-07-30
status: current
tags: [doctrine, enforcement, gate-quality, classification, evidence, gotcha]
reverify: "bash scripts/check_evidence_citations.sh"
evidence: DOCTRINE_ENFORCEMENT.md §10 (`EVIDENCE-CITATIONS` — three buckets, fail-closed); docs/decisions/0030-durable-closure-evidence-citations.md; scripts/check_evidence_citations.sh
---

`EVIDENCE-CITATIONS` requires every `anvil-<name>` token to sit in one of **three** declared
buckets and **fails closed on an unknown one**. That is deliberate: after the `/tmp` prefix was
removed, **no lexical rule separates a bank name from a binary name**, so a guessing gate can only
fail in one of two ways — ignore a real uncited bank, or cry wolf on ordinary prose.

**A gate that cries wolf gets deleted, and its deletion takes its real coverage with it.** That is
the asymmetry that makes classification worth the maintenance.

**Its grandfathered list is pinned by entry count *and* membership SHA-256, because its membership
is a historical fact** — the banks that existed before decision
[`0030`](../decisions/0030-durable-closure-evidence-citations.md). Unpinned, *"just grandfather
it"* bypasses the doctrine in one line and the check becomes decorative. Pinned, the list is
incapable of growing, which is why it is authoritative rather than a shadow under
[[shadow-enumeration-classification]]'s test (2).

See also [[doctrine-enforcement]] for the registry and driver, and [[coverage-check-vacuity]] for
the probe that proves a check is not passing vacuously.
