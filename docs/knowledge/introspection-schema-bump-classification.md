---
id: introspection-schema-bump-classification
title: Bumping the introspection schema is a classification pass, not a find-and-replace — and an evidence pointer names a symbol, never its value
answers:
  - "how do I bump the introspection schema version"
  - "how do I add a metric to the introspection document"
  - "which files do I update for a schema_version bump"
  - "can I sed the old schema version to the new one"
  - "why is there still an old schema version in the repo after a bump"
  - "should I update 1.26 to 1.27 in the schema changelog"
  - "what is a MINOR vs MAJOR introspection schema bump"
  - "why did a knowledge card go stale after a schema bump"
  - "how should a fact card cite a constant"
date: 2026-08-01
status: current
tags: [introspection, schema, versioning, mcp, live-docs, gotcha, knowledge-map, drift]
evidence: src/introspect/mod.rs (SCHEMA_VERSION + its doc-comment history); docs/AGENT_INTROSPECTION_SCHEMA.md (section 7 policy + the per-bump changelog); src/mcp/mod.rs (the schema_version assertions); docs/knowledge/api-reference.md (the card that rotted to 1.11 against a live 1.27); docs/tasks/CAPABILITY-BREADTH-EXPANSION.md (the .4b.2a verification entry, which classifies every touched site)
reverify: 'grep -rn "SCHEMA_VERSION" src/introspect/mod.rs && grep -c "schema_version\"\], \"" src/mcp/mod.rs'
---

# A version string is three different kinds of fact, and only one of them moves

An additive introspection bump (`MINOR`, per `docs/AGENT_INTROSPECTION_SCHEMA.md` §7) means
raising `introspect::SCHEMA_VERSION` and propagating it. The version literal occurs dozens of
times across the repo, and **a bulk replace is wrong** — the occurrences fall into three classes:

| class | example | action |
| --- | --- | --- |
| **current value** | the `SCHEMA_VERSION` const; the `introspect` + `mcp` `schema_version` assertions; *"This document defines `X`"*; the book's `"schema_version": "X"` example outputs; the `TOOLBOX` / `USER_GUIDE` envelope rows | **bump** |
| **history** | a changelog entry `X → Y`; a per-feature attribution (*"`reach_path` landed at `1.27`"*); a `CODEBASE_ANALYSIS.md` timeline row | **never touch** — rewriting it falsifies *when* a key appeared, which is the only question the changelog exists to answer |
| **evidence for an anecdote** | *"this line came to say `1.11` against a live `1.27`"* | **never touch** — the number *is* the evidence |

Find every site with `grep -rn "<old>" --include=*.rs --include=*.md`, then classify each hit
before editing it. The canonical worked example is the `1.27 → 1.28` bump at
`CAPABILITY-BREADTH-EXPANSION.4b.2a`, whose tree verification entry lists what moved and what
deliberately did not.

## The corollary that keeps cards from rotting

**An `evidence:` pointer names a symbol; it never records that symbol's current value.**

- ✅ `src/introspect/mod.rs (SCHEMA_VERSION)`
- ❌ `src/introspect/mod.rs (SCHEMA_VERSION = 1.27)`

The second form goes stale on **every** future bump, including bumps that have nothing to do
with the card's subject — so a card about one feature quietly starts misinforming because an
unrelated feature shipped. This repo has paid for the lesson twice: `docs/knowledge/api-reference.md`
copied a version into its body and rotted from `1.11` to a live `1.27`, wrote the lesson down —
and a sibling card three files away was still citing `SCHEMA_VERSION = 1.27` four leaves later.
**When one card pays for a lesson, sweep for the siblings that have not.**

Related: [[api-reference]] · [[doctrine-enforcement]]
