---
id: probability-is-priority-under-mutual-exclusion
title: Under mutual exclusion with a fixed pass order, a probability knob is **priority**, not intensity — setting nine emit probabilities to `1.0` makes the second pass claim every gate and five later surfaces emit **zero**
answers:
  - "why does structured-emission-max emit only one surface"
  - "why did setting every emit probability to 1.0 reduce diversity"
  - "how should mutually exclusive emit probabilities be calibrated"
  - "what value maximises the least represented emission surface"
  - "what probability should a new structured-emission surface get"
  - "are max coverage and max diversity the same thing"
date: 2026-07-30
status: current
tags: [knobs, calibration, structured-emission, probability, preset, gotcha]
reverify: "cargo run -- --seed 42 --profile structured-emission-max --metrics"
evidence: docs/decisions/0032-emit-surface-interaction-gate.md (the full measurement and the max-min calibration); src/config.rs (`presets()`; the preset-covers-every-surface drift test)
---

The nine `*_emit_prob` knobs select **mutually exclusive** projections of the same gate, evaluated
in a **fixed pass order**. So a probability is not an intensity dial — it is a **priority**.

Measured: with all nine at `1.0`, `function_emit` (second in the order) claims every admissible
gate and **five later surfaces emit zero**. `--profile structured-emission-max` sets four surfaces
and emits **one**.

**Max *coverage* and max *diversity* are opposed here, and ANVIL wants diversity.** Calibrate by
**max-min** — the value that maximises the *least-represented* surface, measured at `0.25` — not
by pushing every knob up.

**A tenth surface joins at the shared intermediate value, never at `1.0`.** Full detail, the
measurement, and the drift test that keeps the preset honest:
[`0032`](../decisions/0032-emit-surface-interaction-gate.md).
