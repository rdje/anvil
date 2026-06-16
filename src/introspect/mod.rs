//! Agent-introspection emission surface (`AGENT-INTROSPECTION-MCP.3`).
//!
//! Builds the stable, versioned introspection document specified in
//! [`docs/AGENT_INTROSPECTION_SCHEMA.md`](../../docs/AGENT_INTROSPECTION_SCHEMA.md)
//! from facts ANVIL *already records*. There is exactly one design rule
//! here, inherited from decision `0004`:
//!
//! > **Invariant SCHEMA-DERIVED.** Every payload field is a `serde`
//! > projection of an existing struct — `Config`, [`Metrics`],
//! > [`DesignMetrics`]. The adapter computes **zero** new truth; it only
//! > chooses which existing fact lands under which envelope key. The only
//! > genuinely new fields are the *envelope* metadata (version strings, the
//! > request echo, the content-addressed `run_id`, resource pointers),
//! > which this surface owns.
//!
//! The surface is read-mostly, additive, and default-off: it is reached only
//! through the `--introspect` CLI flag on a single-artifact stdout run, so the
//! default `anvil` build and the `--artifact dut` byte-identical contract are
//! unaffected (`introspect` never calls the generator differently — it
//! consumes an already-generated `Module`/`Design`).
//!
//! Scope: `AGENT-INTROSPECTION-MCP.3` landed the **DUT** lane (`module` and
//! `design` artifacts, typed [`IntrospectionDocument`]); `AGENT-MCP-EXPANSION.3b`
//! added the non-DUT `microdesign`/`frontend` lanes via [`manifest_lane_document`]
//! (built as a JSON `Value` so the DUT typed path stays byte-identical),
//! inlining each lane's expected-facts manifest under the schema's
//! `microdesign_manifest`/`frontend_manifest` payload key (§5/§6.5). The
//! `coverage` section is a `tool_matrix`-run property (a lone artifact cannot
//! prove `saw_recursive_hierarchy_*`), so a single-artifact introspect omits
//! it and records a `warnings[]` note, exactly as the schema (§5/§6.4)
//! requires.

/// Derived-relation analysis (`SEMANTIC-INTROSPECTION-EXPANSION.2b.1`): the
/// pure output-support-cone query over the already-emitted IR. Kept in its own
/// submodule so the default `IntrospectionPayload` stays lean (decision `0011`
/// Q2); the cone is reached through the MCP `analyze` tool (`.2b.2`), not the
/// default `--introspect` document.
pub mod analyze;

use crate::config::Config;
use crate::emit;
use crate::ir::{Design, Module};
use crate::metrics::{compute, compute_design, DesignMetrics, Metrics};
use serde::{Deserialize, Serialize};

/// The schema version this surface emits. Bumped per the policy in
/// `docs/AGENT_INTROSPECTION_SCHEMA.md` §7 (`MAJOR.MINOR`). `1.7` is the
/// additive (backward-compatible) MINOR bump that adds the fourth derived-query
/// kind `module_reachability` — the
/// [`DerivedAnalysis::module_reachability`](analyze::DerivedAnalysis) field
/// carrying [`ModuleReachability`](analyze::ModuleReachability)s
/// (`SEMANTIC-INTROSPECTION-EXPANSION.5b.2`). The field is
/// `#[serde(default, skip_serializing_if)]`, so `output_support` / `input_reach`
/// / `flop_reset_provenance` documents are **byte-identical** to `1.6` (the key
/// is omitted); only a `module_reachability` document carries it, and only the
/// `schema_version` string advances. Old consumers ignore the new field. See the
/// schema-doc §7 changelog for the full `1.0 → … → 1.6 → 1.7` history.
pub const SCHEMA_VERSION: &str = "1.7";

/// The lane string for the DUT artifact lane.
pub const LANE_DUT: &str = "dut";

/// The lane string for the oracle-backed micro-design lane (Phase 7).
pub const LANE_MICRODESIGN: &str = "microdesign";

/// The lane string for the source-level frontend/elaboration accept lane
/// (Phase 8).
pub const LANE_FRONTEND: &str = "frontend";

/// Recorded in `warnings[]` when a single-artifact introspect cannot carry
/// the matrix-only `coverage` section (schema §6.4).
pub const COVERAGE_ABSENT_NOTE: &str =
    "coverage section absent: single-artifact generate, not a tool_matrix run";

/// The crate version, surfaced as `anvil_version` (schema §4).
pub fn anvil_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// A deliberate, fetch-on-demand pointer to a bulk resource (the emitted
/// `.sv`, a manifest). Bulk artifacts are *not* inlined by default — the
/// agent fetches them as resources (schema §6.6, "structured queries, not
/// bulk dumps").
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRef {
    pub uri: String,
    pub bytes: Option<usize>,
}

/// Echo of the determinism tuple `(seed, knobs, lane)` plus the
/// content-addressed `run_id`. `knobs` is the effective [`Config`] — the
/// `config` section of the schema lives here (schema §4 / §6.1).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestEcho {
    pub seed: u64,
    pub lane: String,
    pub knobs: Config,
    pub run_id: String,
}

/// Descriptor of the produced artifact. `sv` is a [`ResourceRef`] (not
/// inlined); `sv_sha256`/`manifest` are optional and `None` for the
/// single-shot stdout surface in `.3`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactDescriptor {
    pub kind: String,
    pub top: Option<String>,
    pub sv: ResourceRef,
    pub sv_sha256: Option<String>,
    pub manifest: Option<ResourceRef>,
}

/// One per-module metrics entry inside a `design` document's payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleMetricsEntry {
    pub name: String,
    pub metrics: Metrics,
}

/// The structured-facts payload. Each present field is the exact `serde`
/// projection of an existing struct (invariant SCHEMA-DERIVED). Absent
/// sections are omitted rather than null so the document stays compact.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IntrospectionPayload {
    /// `module` artifact: `metrics::compute(&Module)`.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub module_metrics: Option<Metrics>,
    /// `design` artifact: `metrics::compute_design(&Design)`.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub design_metrics: Option<DesignMetrics>,
    /// `design` artifact: per-child `metrics::compute(&Module)`.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub modules: Vec<ModuleMetricsEntry>,
}

/// The top-level introspection document (schema §4). The envelope fields are
/// owned by this surface; the payload is derived (§2).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntrospectionDocument {
    pub schema_version: String,
    pub anvil_version: String,
    pub lane: String,
    pub request: RequestEcho,
    pub artifact: ArtifactDescriptor,
    pub introspection: IntrospectionPayload,
    pub warnings: Vec<String>,
}

impl IntrospectionDocument {
    /// Serialize to the canonical pretty JSON the CLI prints. Deterministic:
    /// struct field order is declaration order and every nested map is a
    /// `BTreeMap`, so the bytes are a pure function of the inputs.
    pub fn to_json_pretty(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

/// A standalone derived-relation analysis document
/// (`SEMANTIC-INTROSPECTION-EXPANSION.2b`). It reuses the introspection
/// **envelope** ([`RequestEcho`] + the content-addressed `run_id`, the
/// [`ArtifactDescriptor`]) but carries a single `analysis` payload instead of
/// the structural `introspection` payload — so the default `--introspect`
/// document stays lean (decision `0011` Q2) while the derived relation is a
/// first-class, versioned, self-identifying document. Served by the pure MCP
/// `analyze` tool and as the `anvil://artifact/<run_id>/analysis/<query>`
/// resource. Schema-derived (invariant SCHEMA-DERIVED): `analysis` is a pure
/// projection of the IR graph the generator already produced.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DerivedAnalysisDocument {
    pub schema_version: String,
    pub anvil_version: String,
    pub lane: String,
    pub request: RequestEcho,
    pub artifact: ArtifactDescriptor,
    /// The derived relation for this artifact (e.g. the output support cone).
    pub analysis: analyze::DerivedAnalysis,
    pub warnings: Vec<String>,
}

impl DerivedAnalysisDocument {
    pub fn to_json_pretty(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

/// Wrap a [`DerivedAnalysis`](analyze::DerivedAnalysis) in the introspection
/// envelope, reusing `base`'s request/artifact metadata — the same `run_id`,
/// seed, knobs, and artifact pointers as the artifact's `generate` /
/// `introspect` document, so an agent can correlate the analysis with the
/// artifact it describes. Pure: byte-identical for the same `(base, analysis)`.
pub fn derived_analysis_document(
    base: &IntrospectionDocument,
    analysis: analyze::DerivedAnalysis,
) -> DerivedAnalysisDocument {
    DerivedAnalysisDocument {
        schema_version: base.schema_version.clone(),
        anvil_version: base.anvil_version.clone(),
        lane: base.lane.clone(),
        request: base.request.clone(),
        artifact: base.artifact.clone(),
        analysis,
        warnings: base.warnings.clone(),
    }
}

/// Content address for a request: a pure FNV-1a 64-bit hash over the
/// canonical encoding of `(schema_version, anvil_version, lane, seed,
/// knobs)`. It is **not** a random nonce — identical inputs yield an
/// identical `run_id`, which is exactly the content-addressed cache key
/// decision `0004` relies on. The hash function is an implementation detail
/// (the schema only requires purity + a hex string), free to change in a
/// future leaf without altering the contract.
///
/// Exposed (`AGENT-INTROSPECTION-MCP.5.2`) so the controlled `validate` tool
/// stamps each run with the **same** content address `generate`/`introspect`
/// use — one deterministic `run_id` per `(seed, knobs)`, not a second scheme.
pub fn content_run_id(lane: &str, seed: u64, knobs: &Config) -> String {
    // `serde_json::to_string(Config)` is deterministic (declaration field
    // order; BTreeMap-sorted nested maps), so the canonical string is stable.
    let knobs_json = serde_json::to_string(knobs).unwrap_or_default();
    content_run_id_for_knobs(lane, seed, &knobs_json)
}

/// The generalized content address over an already-canonical knobs string.
/// `content_run_id` is the DUT specialization (`knobs_json =
/// serde_json::to_string(&Config)`); the non-DUT lanes
/// (`AGENT-MCP-EXPANSION.3b`) pass their own deterministic scoped-knob
/// encoding (e.g. `{"n_params":5}`), so a `microdesign` run and a `dut` run
/// with the same `seed` get distinct content addresses (the `lane` field
/// already separates them, and differing scoped knobs now differ too). The
/// DUT call path is byte-identical: it produces exactly the same canonical
/// string this function always built.
pub fn content_run_id_for_knobs(lane: &str, seed: u64, knobs_json: &str) -> String {
    let canonical = format!(
        "{SCHEMA_VERSION}\u{1f}{}\u{1f}{lane}\u{1f}{seed}\u{1f}{knobs_json}",
        anvil_version(),
    );
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325; // FNV-1a 64-bit offset basis
    for byte in canonical.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3); // FNV-1a 64-bit prime
    }
    format!("{hash:016x}")
}

/// Build the introspection document for a single DUT `module` artifact.
///
/// `seed` and `cfg` are the request that produced `m`; `m` is the
/// already-generated module (this function does not generate — it
/// re-projects). Pure: byte-identical for the same `(seed, cfg, m)`.
pub fn module_document(seed: u64, cfg: &Config, m: &Module) -> IntrospectionDocument {
    let metrics = compute(m);
    let sv_len = emit::to_sv_versioned(m, cfg.sv_version).len();
    let run_id = content_run_id(LANE_DUT, seed, cfg);
    IntrospectionDocument {
        schema_version: SCHEMA_VERSION.to_string(),
        anvil_version: anvil_version().to_string(),
        lane: LANE_DUT.to_string(),
        request: RequestEcho {
            seed,
            lane: LANE_DUT.to_string(),
            knobs: cfg.clone(),
            run_id: run_id.clone(),
        },
        artifact: ArtifactDescriptor {
            kind: "module".to_string(),
            top: Some(m.name.clone()),
            sv: ResourceRef {
                uri: format!("anvil://artifact/{run_id}/{}.sv", m.name),
                bytes: Some(sv_len),
            },
            sv_sha256: None,
            manifest: None,
        },
        introspection: IntrospectionPayload {
            module_metrics: Some(metrics),
            ..Default::default()
        },
        warnings: vec![COVERAGE_ABSENT_NOTE.to_string()],
    }
}

/// Build the introspection document for a single DUT `design` artifact.
///
/// `seed` and `cfg` are the request that produced `design`. Pure.
pub fn design_document(seed: u64, cfg: &Config, design: &Design) -> IntrospectionDocument {
    let design_metrics = compute_design(design);
    let sv_len = emit::to_sv_design_versioned(design, cfg.sv_version).len();
    let run_id = content_run_id(LANE_DUT, seed, cfg);
    let modules = design
        .modules
        .iter()
        .map(|m| ModuleMetricsEntry {
            name: m.name.clone(),
            metrics: compute(m),
        })
        .collect();
    IntrospectionDocument {
        schema_version: SCHEMA_VERSION.to_string(),
        anvil_version: anvil_version().to_string(),
        lane: LANE_DUT.to_string(),
        request: RequestEcho {
            seed,
            lane: LANE_DUT.to_string(),
            knobs: cfg.clone(),
            run_id: run_id.clone(),
        },
        artifact: ArtifactDescriptor {
            kind: "design".to_string(),
            top: Some(design.top.clone()),
            sv: ResourceRef {
                uri: format!("anvil://artifact/{run_id}/{}.sv", design.top),
                bytes: Some(sv_len),
            },
            sv_sha256: None,
            manifest: None,
        },
        introspection: IntrospectionPayload {
            design_metrics: Some(design_metrics),
            modules,
            ..Default::default()
        },
        warnings: vec![COVERAGE_ABSENT_NOTE.to_string()],
    }
}

/// The introspection payload key carrying a lane's inlined expected-facts
/// manifest (schema §5 / §6.5). `None` for the DUT lane (it has no manifest).
pub fn manifest_payload_key(lane: &str) -> Option<&'static str> {
    match lane {
        LANE_MICRODESIGN => Some("microdesign_manifest"),
        LANE_FRONTEND => Some("frontend_manifest"),
        _ => None,
    }
}

/// Build the introspection document for a non-DUT, manifest-carrying lane
/// (`microdesign` / `frontend`) as a JSON [`Value`](serde_json::Value)
/// (`AGENT-MCP-EXPANSION.3b`, design `.3a`).
///
/// Per the schema contract (`docs/AGENT_INTROSPECTION_SCHEMA.md` §5 / §6.5,
/// defined at v1.0), the lane's expected-facts **manifest** is **inlined** in
/// the `introspection` payload under `microdesign_manifest` /
/// `frontend_manifest` — these are "small and stable" and an exact serde
/// projection of `microdesign::Manifest` / `frontend::Manifest`, so this adds
/// zero new truth. (§6.6's "resource, not inlined" rule applies only to the
/// bulk `.sv`.) The same manifest is *also* exposed as a fetch-on-demand
/// `artifact.manifest` resource (the §4 slot), so an agent can read the
/// inlined facts directly or fetch the raw bytes deliberately; both derive
/// from the one `emit_manifest` output and cannot drift.
///
/// It is built as a `Value` rather than the typed [`IntrospectionDocument`]
/// on purpose: the typed [`RequestEcho::knobs`] is a [`Config`], which the
/// non-DUT lanes do not have (their knobs are `n_params`/`n_children`).
/// Keeping the typed DUT path untouched preserves its byte-stability (a
/// `Config` serializes in declaration order; a `serde_json::Value` object
/// would re-sort the keys). The DUT-only payload sections
/// (`module_metrics`/`design_metrics`/`modules`) are absent for non-DUT
/// lanes. Envelope keys match [`IntrospectionDocument`] exactly.
#[allow(clippy::too_many_arguments)]
pub fn manifest_lane_document(
    lane: &str,
    kind: &str,
    seed: u64,
    knobs: &serde_json::Value,
    top: Option<&str>,
    run_id: &str,
    sv_len: usize,
    manifest_facts: &serde_json::Value,
    manifest_len: usize,
) -> serde_json::Value {
    // Inline the manifest under the schema-defined payload key (§6.5).
    let mut payload = serde_json::Map::new();
    if let Some(key) = manifest_payload_key(lane) {
        payload.insert(key.to_string(), manifest_facts.clone());
    }
    serde_json::json!({
        "schema_version": SCHEMA_VERSION,
        "anvil_version": anvil_version(),
        "lane": lane,
        "request": {
            "seed": seed,
            "lane": lane,
            "knobs": knobs,
            "run_id": run_id,
        },
        "artifact": {
            "kind": kind,
            "top": top,
            "sv": {
                "uri": format!("anvil://artifact/{run_id}/sv"),
                "bytes": sv_len,
            },
            "sv_sha256": serde_json::Value::Null,
            "manifest": {
                "uri": format!("anvil://artifact/{run_id}/manifest"),
                "bytes": manifest_len,
            },
        },
        "introspection": serde_json::Value::Object(payload),
        "warnings": [COVERAGE_ABSENT_NOTE],
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Generator;
    use serde_json::Value;

    fn comb_cfg(seed: u64) -> Config {
        Config {
            seed,
            ..Config::default()
        }
    }

    fn wrapper_cfg(seed: u64) -> Config {
        Config {
            seed,
            hierarchy_depth: 1,
            num_leaf_modules: 2,
            num_child_instances: 2,
            ..Config::default()
        }
    }

    #[test]
    fn module_document_is_schema_v1_and_dut() {
        let cfg = comb_cfg(7);
        let mut gen = Generator::new(cfg.clone());
        let m = gen.generate_module();
        let doc = module_document(7, &cfg, &m);

        assert_eq!(doc.schema_version, "1.7");
        assert_eq!(doc.anvil_version, env!("CARGO_PKG_VERSION"));
        assert_eq!(doc.lane, "dut");
        assert_eq!(doc.request.seed, 7);
        assert_eq!(doc.artifact.kind, "module");
        assert_eq!(doc.artifact.top.as_deref(), Some(m.name.as_str()));
        assert!(doc.artifact.sv.bytes.is_some());
        assert!(doc.artifact.sv_sha256.is_none());
        assert!(doc.warnings.iter().any(|w| w == COVERAGE_ABSENT_NOTE));
    }

    #[test]
    fn payload_is_derived_zero_new_truth() {
        // Every payload field must equal the exact serde projection of the
        // existing struct — the SCHEMA-DERIVED invariant, asserted directly.
        let cfg = comb_cfg(3);
        let mut gen = Generator::new(cfg.clone());
        let m = gen.generate_module();
        let doc = module_document(3, &cfg, &m);

        // request.knobs IS the effective config (the `config` section).
        assert_eq!(
            serde_json::to_value(&doc.request.knobs).unwrap(),
            serde_json::to_value(&cfg).unwrap()
        );
        // module_metrics IS metrics::compute(&m), byte-for-byte.
        assert_eq!(
            serde_json::to_value(doc.introspection.module_metrics.as_ref().unwrap()).unwrap(),
            serde_json::to_value(compute(&m)).unwrap()
        );
        // No design payload on a module artifact.
        assert!(doc.introspection.design_metrics.is_none());
        assert!(doc.introspection.modules.is_empty());
    }

    #[test]
    fn run_id_is_deterministic_and_request_sensitive() {
        let cfg_a = comb_cfg(11);
        let mut gen_a = Generator::new(cfg_a.clone());
        let m_a = gen_a.generate_module();

        // Same (seed, cfg) => identical run_id (content address).
        let id1 = module_document(11, &cfg_a, &m_a).request.run_id;
        let id2 = module_document(11, &cfg_a, &m_a).request.run_id;
        assert_eq!(id1, id2);
        assert_eq!(id1.len(), 16); // 64-bit hex

        // Different seed => different request => different run_id.
        let cfg_b = comb_cfg(12);
        let mut gen_b = Generator::new(cfg_b.clone());
        let m_b = gen_b.generate_module();
        let id3 = module_document(12, &cfg_b, &m_b).request.run_id;
        assert_ne!(id1, id3);
    }

    #[test]
    fn design_document_has_design_metrics_and_modules() {
        let cfg = wrapper_cfg(42);
        let mut gen = Generator::new(cfg.clone());
        let design = gen.generate_design();
        let doc = design_document(42, &cfg, &design);

        assert_eq!(doc.artifact.kind, "design");
        assert_eq!(doc.artifact.top.as_deref(), Some(design.top.as_str()));
        assert!(doc.introspection.module_metrics.is_none());
        assert!(doc.introspection.design_metrics.is_some());
        assert_eq!(doc.introspection.modules.len(), design.modules.len());
        // Each per-module entry is the exact compute(&module) projection.
        for (entry, module) in doc.introspection.modules.iter().zip(&design.modules) {
            assert_eq!(entry.name, module.name);
            assert_eq!(
                serde_json::to_value(&entry.metrics).unwrap(),
                serde_json::to_value(compute(module)).unwrap()
            );
        }
        assert_eq!(
            serde_json::to_value(doc.introspection.design_metrics.as_ref().unwrap()).unwrap(),
            serde_json::to_value(compute_design(&design)).unwrap()
        );
    }

    #[test]
    fn document_round_trips_through_json() {
        let cfg = comb_cfg(5);
        let mut gen = Generator::new(cfg.clone());
        let m = gen.generate_module();
        let doc = module_document(5, &cfg, &m);

        let s = doc.to_json_pretty().unwrap();
        let reparsed: IntrospectionDocument = serde_json::from_str(&s).unwrap();
        assert_eq!(
            serde_json::to_value(&doc).unwrap(),
            serde_json::to_value(&reparsed).unwrap()
        );
        // Pretty JSON is deterministic for the same inputs.
        assert_eq!(s, module_document(5, &cfg, &m).to_json_pretty().unwrap());
    }

    #[test]
    fn document_is_valid_json_object_with_envelope_keys() {
        let cfg = comb_cfg(1);
        let mut gen = Generator::new(cfg.clone());
        let m = gen.generate_module();
        let v: Value = serde_json::to_value(module_document(1, &cfg, &m)).unwrap();
        let obj = v.as_object().unwrap();
        for key in [
            "schema_version",
            "anvil_version",
            "lane",
            "request",
            "artifact",
            "introspection",
            "warnings",
        ] {
            assert!(obj.contains_key(key), "missing envelope key `{key}`");
        }
    }

    #[test]
    fn derived_analysis_document_reuses_envelope_and_carries_analysis() {
        // The analysis document reuses the artifact's envelope (same content
        // address) and carries the support-cone projection as its payload.
        let cfg = comb_cfg(9);
        let mut gen = Generator::new(cfg.clone());
        let m = gen.generate_module();
        let base = module_document(9, &cfg, &m);
        let analysis = analyze::module_support_cones(&m, None);
        let doc = derived_analysis_document(&base, analysis.clone());

        assert_eq!(doc.schema_version, "1.7");
        assert_eq!(doc.lane, base.lane);
        assert_eq!(doc.request.run_id, base.request.run_id); // same content address
        assert_eq!(doc.analysis.query, "output_support");
        // The analysis payload IS the support-cone projection, byte-for-byte.
        assert_eq!(
            serde_json::to_value(&doc.analysis).unwrap(),
            serde_json::to_value(&analysis).unwrap()
        );
        // Round-trips through JSON and is deterministic.
        let s = doc.to_json_pretty().unwrap();
        let reparsed: DerivedAnalysisDocument = serde_json::from_str(&s).unwrap();
        assert_eq!(
            serde_json::to_value(&doc).unwrap(),
            serde_json::to_value(&reparsed).unwrap()
        );
        let v: Value = serde_json::to_value(&doc).unwrap();
        let obj = v.as_object().unwrap();
        for key in [
            "schema_version",
            "anvil_version",
            "lane",
            "request",
            "artifact",
            "analysis",
            "warnings",
        ] {
            assert!(obj.contains_key(key), "missing envelope key `{key}`");
        }
    }
}
