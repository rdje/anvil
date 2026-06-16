---
id: sv-version-soft-union-upopt
title: ANVIL's first version-distinctive up-opt is a live default-off `union soft` low-bits-slice overlay gated on `--sv-version 2023`
answers:
  - "what is soft_union_slice_prob"
  - "what does soft_union_slice_prob do"
  - "does ANVIL emit a SystemVerilog 2023 union soft"
  - "how does ANVIL emit a heterogeneous-width packed union"
  - "what is the first ANVIL up-opted construct that ships"
  - "how do I make ANVIL emit a union soft overlay"
  - "is the union soft overlay byte-identical by default"
  - "how is the union soft overlay rendered"
  - "does the union soft overlay change behaviour"
  - "which tools accept the ANVIL union soft overlay"
  - "where is the union soft up-opt implemented"
  - "how does ANVIL down-gate a union soft slice below 2023"
date: 2026-06-16
status: current
tags: [sv-version, up-opt, soft-union, 2023, emission, downstream, knob, slice]
evidence: src/ir/soft_union.rs; src/emit/sv.rs (soft_union_slice_overlay); src/config.rs (soft_union_slice_prob); src/gen/mod.rs (generate_module + generate_design rolls); tests/sv_version_downstream.rs; docs/decisions/0010-sv-version-first-upopt-soft-packed-union.md
reverify: 'cargo run --quiet -- --seed 1 --dump-config > /tmp/c.json && python3 -c "import json;c=json.load(open(\"/tmp/c.json\"));c.update({\"soft_union_slice_prob\":1.0,\"sv_version\":\"2023\",\"gate_struct_weight\":10,\"min_width\":4,\"max_width\":16});json.dump(c,open(\"/tmp/su.json\",\"w\"))" && cargo run --quiet -- --seed 7 --config /tmp/su.json | tee /tmp/su.sv | grep -c "union soft" && verilator --lint-only --language 1800-2023 /tmp/su.sv && echo CLEAN'
---

# `SV-VERSION-TARGETING.3b.2a` — the live `union soft` low-bits-slice up-opt

ANVIL's **first version-distinctive up-opt** ships as of `SV-VERSION-TARGETING.3b.2a`
(decision [`0010`](../decisions/0010-sv-version-first-upopt-soft-packed-union.md)).

- **Knob:** `Config::soft_union_slice_prob` (serde/config-file only — no CLI flag,
  like `aggregate_prob`; default `0.0` ⇒ byte-identical; validated `0.0..=1.0`).
- **What it does:** per *proper low-bits* slice — a `GateOp::Slice { hi, lo: 0 }`
  over a **non-constant**, strictly-wider source — the gen-time pass
  `crate::ir::soft_union::annotate_soft_union_slices` rolls the probability on the
  seeded RNG and marks the gate in `Module.soft_union_slice_gates`
  (`BTreeSet<NodeId>`, an emitter-surface annotation, **not** hashed into
  `canonical_module_signature`).
- **Emission (only under a 2023 target):** the emitter
  (`emit/sv.rs::soft_union_slice_overlay`) renders a marked gate as an internal
  heterogeneous-width IEEE 1800-2023 `union soft` overlay instead of `src[hi:0]`:

  ```systemverilog
  wire [3:0] slc;
  union soft { logic [7:0] w; logic [3:0] n; } slc__u;
  assign slc__u.w = a;
  assign slc = slc__u.n;   // == a[3:0]
  ```

- **Down-gating:** below `sv_version` 2023 the marker is ignored — a marked gate
  emits the plain `src[hi:0]`. So output diverges across targets **only** when the
  knob is on *and* the target permits 2023.
- **Genuinely 2023 + behaviour-preserving:** heterogeneous-width packed-union
  members are legal only as `union soft` (a plain packed union requires equal-width
  members — IEEE 1800-2023 §7.3.1); packed-union members are LSB-aligned, so
  `slc__u.n == a[3:0]`.
- **Downstream:** Verilator accepts/builds it under `--language 1800-2023`
  (`tests/sv_version_downstream.rs` banks generator-produced overlays clean);
  Yosys/Icarus have no IEEE-1800 selector and reject the `union soft` syntax, so
  they are a **recorded no-op** for this construct (decision `0010`).
- **Boundaries:** the pass skips Phase 5 `param_env` modules (param/up-opt
  cross-product out of scope); the repo-owned matrix up-opt gate +
  `saw_sv_version_2023_soft_union_upopt` coverage fact land at `.3b.2b`.

See [[sv-version-targeting]] and [[sv-version-first-upopt-soft-packed-union]].
