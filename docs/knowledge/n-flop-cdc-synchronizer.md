---
id: n-flop-cdc-synchronizer
title: N-flop CDC synchronizer is config-selectable
answers:
  - "can ANVIL generate N-flop CDC synchronizers"
  - "what does cdc_synchronizer_stages do"
  - "how can ANVIL prove a 3-flop CDC synchronizer was generated"
  - "does ANVIL generate async FIFOs or pulse synchronizers"
date: 2026-06-05
status: current
tags: [cdc, multi-clock, synchronizer, signoff]
evidence: src/gen/multi_clock.rs; src/config.rs; src/metrics.rs; src/bin/tool_matrix.rs; book/src/sequential.md; book/src/knobs.md
---

ANVIL's opt-in multi-clock path uses `multi_clock_prob` and
`cdc_synchronizer_stages`. The default `cdc_synchronizer_stages = 2`
preserves the existing 2-flop chain; values `>= 3` generate an N-flop
1-bit synchronizer when promotion finds an eligible 1-bit flop-driven
output. Metrics `num_cdc_synchronizer_chains` and
`max_cdc_synchronizer_stages` expose the result;
`num_cdc_2_flop_synchronizers` counts exact 2-stage chains. The
`tool_matrix` default set includes `int_multi_clock_3flop_sync` and the
coverage fact `saw_cdc_nflop_synchronizer`. General CDC fabrics such as
async FIFO, gray-code pointer transfer, req/ack handshakes, pulse
synchronizers, and reset synchronizers remain deferred.
