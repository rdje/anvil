---
id: resource-safe-validation
title: Full-suite validation is resource-monitored and not mandatory for workflow-doc memory and retrieval leaves
answers:
  - "should I run the full cargo test suite for memory architecture docs"
  - "should I run the full cargo test suite for Knowledge Map docs"
  - "what RAM threshold stops a full suite"
  - "when is focused workflow validation enough"
date: 2026-06-04
status: current
tags: [validation, environment, safety]
evidence: docs/decisions/0003-resource-safe-validation.md; COMMIT.md; MEMORY.md
---

# 0003 - Full-suite validation is resource-monitored and not mandatory for workflow-doc leaves

- Date: 2026-06-04
- Status: accepted
- Tags: validation, environment, mac-mini, safety

## Context

ANVIL's full `cargo test` suite includes deep recursive hierarchy tests
that can saturate CPU for several minutes. On the owner's Mac Mini M4 Pro,
CPU at 100% is expected during the full suite, but RAM approaching or
exceeding 90% is a machine-safety concern because continued memory growth
can reboot the machine.

The memory-architecture adoption is a workflow/docs integration. The owner
explicitly scoped it to functional workflow checks and said the full suite
is not required for these leaves.

The Knowledge Map adoption is the same class of workflow/docs retrieval
integration: it changes repo memory/retrieval process and enforcement, not
Rust generator behavior or generated RTL semantics.

## Decision

For workflow-doc leaves under `MEMORY-ARCHITECTURE-DOC` or
`KNOWLEDGE-MAP-DOC`, do not run the full suite unless specifically needed.
Use focused functional checks such as `scripts/check_memory_architecture.sh`,
`knowledge-map/scripts/check_knowledge_map.sh`, `git diff --check`,
`mdbook build book`, and `cargo check --all-targets` where appropriate.

If the full suite is run in any future task, monitor RAM usage. If RAM
usage exceeds 90%, stop the suite immediately and record the stop as an
environment/resource stop, not a product test failure. From 80% to 90%,
watch the trend closely and report if it keeps increasing.

## Consequences

- Workflow-doc integration can be committed without risking machine
  reboot from unnecessary full-suite pressure.
- Full-suite evidence remains available for code or broad behavioral
  changes, but it must be resource-monitored.
- Validation records must explicitly say when the full suite was skipped
  by owner instruction/resource policy.

## Links

- Task-tree: `MEMORY-ARCHITECTURE-DOC.2`
- Task-tree: `KNOWLEDGE-MAP-DOC.3`
- Standard: `MEMORY_ARCHITECTURE.md`
- Standard: `knowledge-map/KNOWLEDGE_MAP_ARCHITECTURE.md`
