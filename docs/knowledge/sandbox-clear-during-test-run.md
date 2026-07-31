---
id: sandbox-clear-during-test-run
title: Never clear `.cache/anvil-sandbox` while a test run is in flight — it deletes the running tests' working dirs AND their stdout/stderr capture files, so the failure arrives with **empty** output
answers:
  - "why did book_examples fail with empty output"
  - "a test failed but captured no stdout or stderr at all"
  - "is it safe to clear the anvil sandbox during a test run"
  - "where does ANVIL scratch actually live"
  - "what should I suspect when a failure has no output"
date: 2026-07-30
status: current
tags: [testing, sandbox, harness, gotcha, scratch, paths]
reverify: "grep -n 'sandbox_root' src/paths.rs"
evidence: src/paths.rs (`sandbox_root()` resolves all ANVIL scratch to `.cache/anvil-sandbox/`); docs/decisions/0031-ssd-volume-exclusivity.md (why scratch moved onto the repo volume)
---

`.cache/anvil-sandbox/` is the live sandbox root — every ANVIL scratch path resolves through
`crate::paths::sandbox_root()` (decision [`0031`](../decisions/0031-ssd-volume-exclusivity.md)).
Clearing it mid-run deletes the working directories of tests that are currently executing **and**
the files their output is being captured into.

The symptom is distinctive and misleading: a `book_examples` failure with **empty** captured
output. There is no stack, no stderr, no partial log — because the file the runner was writing to
was removed underneath it.

**Clear before a run or after it, never during.**

**The generalisable diagnostic:** *a failure with no captured output at all is evidence about the
harness's scratch, not about the code.* Suspect the working directory before reading `src/`. A
real code failure produces output; a vanished capture file produces silence.
