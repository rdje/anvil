---
id: verilator-declfilename-on-fixed-filename-dumps
title: `verilator -Wall` always fails `DECLFILENAME` on a fixed-filename dump — pass `-Wno-DECLFILENAME`; it is a filename artifact, not a generator defect
answers:
  - "why does verilator -Wall report DECLFILENAME on generated RTL"
  - "is a DECLFILENAME warning a generator bug"
  - "what verilator flags should a fixed-filename dump use"
  - "how do I lint a generated module written to a fixed path"
date: 2026-07-30
status: current
tags: [verilator, downstream, lint, gotcha, false-positive]
reverify: "verilator --lint-only -Wall -Wno-DECLFILENAME generated/mod_42_0000.sv"
evidence: docs/tasks/VOLUME-DATA-LOCALITY.md; docs/tasks/STRUCTURED-EMISSION-EXPANSION.md (repeated probe invocations)
---

`DECLFILENAME` fires when a module's name does not match its containing file's name. Any probe
that writes generated RTL to a **fixed** filename (`dump.sv`, `probe.sv`, a sandbox scratch path)
therefore trips it on every run, regardless of what the generator emitted.

**Pass `-Wno-DECLFILENAME` for fixed-filename probes.** It is an artifact of where the file was
written, not a property of the RTL.

This matters because of ANVIL's standing rule that **a downstream tool rejecting a generated file
is a generator bug**. That rule is only useful if the warning set is free of filename artifacts —
otherwise the one warning that *is* a real defect arrives in a bucket everybody has learned to
ignore. Corpus runs written under `--out` use per-module filenames and do not need the flag.
