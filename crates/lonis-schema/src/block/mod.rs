// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! The `Block` contract — the canonical structured domain object (doctrine §2.7).
//!
//! A [`Block`] is what any Lonis tool emits: uniformly versioned
//! ([`SchemaVersion`] + a stable `$id` per kind), attributed
//! ([`Attribution`]), bounded ([`BlockBounds`]), replayable (content hash +
//! seed via [`ReplayProvenance`]), and render-parity (human + machine render
//! from the same typed value). The payload is a [`BlockKind`] — the 14-kind
//! seed corpus, extensible by domains.

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;

use crate::SchemaVersion;

pub mod kinds;
pub mod schemas;

pub use kinds::BlockKind;

/// The protocol marker for the v1 block contract.
pub const BLOCK_SCHEMA_V1: &str = "lonis.block/v1";

// ===========================================================================
// BlockPayload — the typed-payload contract (ADR-0002)
// ===========================================================================

/// A typed block payload.
///
/// Typing matters in-process — a vertical's own code, its tests, and any
/// library consumer; across a process/JSON boundary it is always JSON anyway.
/// So a vertical defines its own payload enum (e.g. karpal-discovery's
/// `KarpalPayload`) implementing this trait, and gets a fully-typed
/// `Block<MyPayload>` / `ToolRegistry<MyPayload>` with zero erasure. Erasure
/// only reappears at the umbrella host, where the boundary is a subprocess
/// JSON channel and erasure is natural (ADR-0002).
///
/// The 14 seed kinds ([`BlockKind`]) implement this trait and serve as the
/// seed payload; [`BlockKind::Extension`] covers the erased seam.
pub trait BlockPayload: Serialize + DeserializeOwned + Send + Sync + 'static {
    /// The wire discriminant (e.g. `message`, `karpal.capability`).
    fn kind_name(&self) -> &str;
    /// The stable `$id` of this payload kind (e.g. `lonis.block/message/v1`).
    fn schema_id(&self) -> String;
    /// Render the human-facing form (render-parity: from the same typed
    /// value the machine form serializes from).
    fn render_human(&self) -> String;
}

// ===========================================================================
// Replay provenance (envelope-level; superset of amari-discovery's Provenance)
// ===========================================================================

/// Compatibility status and the evidence supporting it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Compatibility {
    /// A stable compatibility status such as `compatible` or `unknown_version`.
    pub status: String,
    /// Human-readable reasons for the status.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reasons: Vec<String>,
}

/// Requirements for replaying a block.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReplayMetadata {
    /// Whether the block carries sufficient provenance for replay.
    pub replayable: bool,
    /// Hash fields that must match before replay.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub required_hashes: Vec<String>,
    /// Reasons replay is unavailable or constrained.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reasons: Vec<String>,
}

/// Replay provenance carried on every block: content hashes, seed, tool
/// version, compatibility, and replay requirements.
///
/// A deliberate superset of amari-discovery's `Provenance` (which carries
/// `project_hash` / `input_hash`; amari adds `plan_hash` / `result_hash` at
/// the plan/result level) so the vertical's `protocol.rs` can delete into
/// this type (ADR-0001).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReplayProvenance {
    /// The producing tool's version.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_version: Option<String>,
    /// Compatibility between inputs and the producer's reference data.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compatibility: Option<Compatibility>,
    /// Replay requirements for the block.
    #[serde(default, skip_serializing_if = "ReplayMetadata::is_default")]
    pub replay: ReplayMetadata,
    /// Hash of the inspected project, when project context exists.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_hash: Option<String>,
    /// Hash of explicit input, when applicable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_hash: Option<String>,
    /// Hash of the plan this block belongs to, when applicable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plan_hash: Option<String>,
    /// Hash of the result this block summarizes, when applicable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result_hash: Option<String>,
    /// Deterministic seed, when the operation uses one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seed: Option<u64>,
}

impl ReplayMetadata {
    fn is_default(&self) -> bool {
        self == &Self::default()
    }
}

impl ReplayProvenance {
    /// Whether no provenance slots are populated.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self == &Self::default()
    }
}

// ===========================================================================
// Attribution (who / under what lens / when / where / which producer)
// ===========================================================================

/// When / where / which-producer produced an attributed block.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AttributionSource {
    /// RFC 3339 timestamp of emission.
    pub when: String,
    /// Stream or session context (serialized as `where`, per doctrine §2.7).
    #[serde(rename = "where", default, skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
    /// The agent or tool id that emitted the block.
    pub producer: String,
}

/// Who a block is attributed to, under what lens, and its emission
/// provenance.
///
/// Richer than amari-discovery's `capability_id` (which collapses attribution
/// because amari-discovery is single-agent, capability-scoped): Lonis keeps
/// the full slot for multi-participant, stream-scoped transcripts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Attribution {
    /// Participant identity (registry-owned, e.g. Dominic's registry).
    pub identity: String,
    /// The lens or role the participant spoke under, when applicable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub viewpoint: Option<String>,
    /// Emission provenance: when, where, and which producer.
    pub provenance: AttributionSource,
}

/// The current UTC time as an RFC 3339 string, for [`AttributionSource::when`].
#[must_use]
pub fn now_rfc3339() -> String {
    time::OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .expect("RFC 3339 formatting is infallible")
}

impl Attribution {
    /// Construct attribution for a participant/producer pair, stamped now
    /// (UTC). `viewpoint` and `location` start unset.
    #[must_use]
    pub fn new(identity: impl Into<String>, producer: impl Into<String>) -> Self {
        Self {
            identity: identity.into(),
            viewpoint: None,
            provenance: AttributionSource {
                when: now_rfc3339(),
                location: None,
                producer: producer.into(),
            },
        }
    }
}

// ===========================================================================
// Bounds (resource limits are first-class)
// ===========================================================================

/// Resource bounds declared on a block. Every unset slot means *unbounded*;
/// a fully-default [`BlockBounds`] is omitted from the wire form.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BlockBounds {
    /// Maximum number of items the payload may carry.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_items: Option<u64>,
    /// Maximum serialized size in bytes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_bytes: Option<u64>,
    /// Maximum length of any single string field.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_length: Option<u64>,
    /// Wall-clock bound on producing the payload, in milliseconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout_millis: Option<u64>,
}

impl BlockBounds {
    /// Whether every bound is unset.
    #[must_use]
    pub fn is_unbounded(&self) -> bool {
        self == &Self::default()
    }
}

// ===========================================================================
// Block
// ===========================================================================

/// The canonical structured domain object (doctrine §2.7), generic over its
/// typed payload `P` (ADR-0002).
///
/// The wire form is flat — the envelope properties (`schema_version`,
/// `provenance`, `warnings`) sit alongside `attribution`, `bounds`, and the
/// tagged `payload` — because the doctrine envelope's `data` *is* the
/// payload, not a wrapper around it.
///
/// Verticals instantiate with their own payload enum for a fully-typed
/// in-process contract; the umbrella host uses [`SeedBlock`] (or the
/// [`BlockKind::Extension`] seam) where tools are reached across subprocess
/// JSON channels.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, bound = "P: BlockPayload")]
pub struct Block<P: BlockPayload> {
    /// The block contract protocol marker (`lonis.block/v1`).
    pub schema_version: SchemaVersion,
    /// Replay provenance: hashes, seed, compatibility, replay requirements.
    #[serde(default, skip_serializing_if = "ReplayProvenance::is_empty")]
    pub provenance: ReplayProvenance,
    /// Non-fatal warnings accumulated while producing the block.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<String>,
    /// Who / under what lens / when / where / which producer.
    pub attribution: Attribution,
    /// First-class resource bounds.
    #[serde(default, skip_serializing_if = "BlockBounds::is_unbounded")]
    pub bounds: BlockBounds,
    /// The typed payload: one of the 14 seed kinds, or a vertical's own
    /// [`BlockPayload`] type.
    pub payload: P,
}

/// A block over the 14-kind seed payload — the umbrella host's type.
pub type SeedBlock = Block<BlockKind>;

impl<P: BlockPayload> Block<P> {
    /// Construct a block at the v1 contract with empty provenance, warnings,
    /// and bounds.
    #[must_use]
    pub fn new(attribution: Attribution, payload: P) -> Self {
        Self {
            schema_version: SchemaVersion::new(BLOCK_SCHEMA_V1)
                .expect("BLOCK_SCHEMA_V1 is canonical"),
            provenance: ReplayProvenance::default(),
            warnings: Vec::new(),
            attribution,
            bounds: BlockBounds::default(),
            payload,
        }
    }

    /// Attach replay provenance.
    #[must_use]
    pub fn with_provenance(mut self, provenance: ReplayProvenance) -> Self {
        self.provenance = provenance;
        self
    }

    /// Attach non-fatal warnings.
    #[must_use]
    pub fn with_warnings(mut self, warnings: Vec<String>) -> Self {
        self.warnings = warnings;
        self
    }

    /// Attach resource bounds.
    #[must_use]
    pub fn with_bounds(mut self, bounds: BlockBounds) -> Self {
        self.bounds = bounds;
        self
    }

    /// The payload.
    #[must_use]
    pub const fn payload(&self) -> &P {
        &self.payload
    }

    /// The stable `$id` of the payload's kind (e.g. `lonis.block/message/v1`).
    #[must_use]
    pub fn schema_id(&self) -> String {
        self.payload.schema_id()
    }

    /// SHA-256 of the canonical payload JSON (kind tag + recursively
    /// key-sorted data) — the content hash replay pins on.
    #[must_use]
    pub fn content_hash(&self) -> String {
        let value =
            serde_json::to_value(&self.payload).expect("BlockKind serialization is infallible");
        json_content_hash(&value)
    }

    /// Render the human-facing form from the same typed value the machine
    /// form serializes from (render-parity).
    #[must_use]
    pub fn render_human(&self) -> String {
        self.payload.render_human()
    }
}

// ===========================================================================
// Canonical JSON + SHA-256 (replay hashing)
// ===========================================================================

/// SHA-256 (hex) of the canonical form of any JSON value — object keys
/// recursively sorted, so semantically equal values hash identically
/// regardless of map insertion order. Tools use this for `input_hash` and
/// other replay pins.
#[must_use]
pub fn json_content_hash(value: &Value) -> String {
    sha256_hex(&canonical_json_string(value))
}

/// Serialize a value with object keys recursively sorted and numbers
/// normalized (ADR-0007), so semantically equal payloads hash identically
/// across producers.
///
/// Number normalization: integral floats collapse to integers (`100.0`,
/// `1e2` → `100`), and negative zero collapses to `0`. Non-integral floats
/// keep serde_json's shortest-round-trip (ryu) rendering; integers are
/// exact at any width (no f64 round-trip). Residual limitation: producers
/// emitting exotic float spellings (e.g. `0.30000000000000004` vs `0.3`)
/// still hash differently — hash-critical values should be integers or
/// strings when cross-producer equality matters.
fn canonical_json_string(value: &Value) -> String {
    fn sorted(value: &Value) -> Value {
        match value {
            Value::Object(map) => map
                .iter()
                .map(|(k, v)| (k.clone(), sorted(v)))
                .collect::<serde_json::Map<String, Value>>()
                .into(),
            Value::Array(items) => items.iter().map(sorted).collect(),
            Value::Number(number) => normalize_number(number),
            scalar => scalar.clone(),
        }
    }
    // serde_json::Map without the `preserve_order` feature is already
    // BTree-ordered; the explicit sort keeps the hash stable if that feature
    // is ever enabled downstream.
    serde_json::to_string(&sorted(value)).expect("canonical JSON serialization is infallible")
}

/// Normalize a JSON number for hashing: integral floats → integers,
/// negative zero → zero. Everything else keeps its exact form.
fn normalize_number(number: &serde_json::Number) -> Value {
    if let Some(float) = number.as_f64() {
        if number.is_f64() {
            if float == 0.0 {
                return Value::from(0_u64);
            }
            if float.fract() == 0.0 && float.abs() <= 9_007_199_254_740_992.0 {
                // Integral float within the exactly-representable range.
                #[allow(clippy::cast_possible_truncation)]
                {
                    if float < 0.0 {
                        return Value::from(float as i64);
                    }
                    return Value::from(float as u64);
                }
            }
        }
    }
    Value::Number(number.clone())
}

fn sha256_hex(input: &str) -> String {
    use sha2::Digest as _;
    let digest = sha2::Sha256::digest(input.as_bytes());
    let mut hex = String::with_capacity(digest.len() * 2);
    for byte in digest {
        use std::fmt::Write as _;
        let _ = write!(hex, "{byte:02x}");
    }
    hex
}

#[cfg(test)]
mod tests {
    use super::kinds::*;
    use super::*;
    use serde_json::json;

    fn attribution() -> Attribution {
        Attribution {
            identity: "dominic".into(),
            viewpoint: Some("reviewer".into()),
            provenance: AttributionSource {
                when: "2026-08-10T12:00:00Z".into(),
                location: Some("session:abc".into()),
                producer: "lonis:test:fixture".into(),
            },
        }
    }

    fn message_block() -> Block<BlockKind> {
        Block::new(
            attribution(),
            BlockKind::Message(Message {
                role: None,
                content: "hello".into(),
                reply_to: None,
            }),
        )
    }

    // -- BlockPayload generics (ADR-0002) --

    /// A vertical's own payload enum, à la karpal-discovery's `KarpalPayload`:
    /// fully typed, no `Value` anywhere the vertical reaches.
    #[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
    #[serde(tag = "kind", rename_all = "snake_case")]
    enum VerticalPayload {
        Capability { record: String },
        Search { results: Vec<String> },
    }

    impl BlockPayload for VerticalPayload {
        fn kind_name(&self) -> &str {
            match self {
                Self::Capability { .. } => "capability",
                Self::Search { .. } => "search",
            }
        }
        fn schema_id(&self) -> String {
            format!("lonis.block/vertical.{}/v1", self.kind_name())
        }
        fn render_human(&self) -> String {
            match self {
                Self::Capability { record } => format!("capability: {record}"),
                Self::Search { results } => format!("search: {} result(s)", results.len()),
            }
        }
    }

    #[test]
    fn block_kind_implements_block_payload() {
        fn assert_impl<T: BlockPayload>() {}
        assert_impl::<BlockKind>();
    }

    #[test]
    fn vertical_payload_block_is_fully_typed_end_to_end() {
        let block = Block::new(
            attribution(),
            VerticalPayload::Search {
                results: vec!["karpal-proof".into()],
            },
        );
        // Typed access — no to_value/from_value round-trip.
        let Block {
            payload: VerticalPayload::Search { results },
            ..
        } = &block
        else {
            panic!("expected a search payload")
        };
        assert_eq!(results, &vec!["karpal-proof".to_string()]);

        let wire = serde_json::to_string(&block).unwrap();
        let back: Block<VerticalPayload> = serde_json::from_str(&wire).unwrap();
        assert_eq!(back, block);
        assert_eq!(block.payload().kind_name(), "search");
        assert_eq!(block.schema_id(), "lonis.block/vertical.search/v1");
        assert!(block.render_human().contains("search"));
        assert_eq!(block.content_hash().len(), 64);
    }

    #[test]
    fn vertical_payload_wire_carries_its_own_tagged_form() {
        let block = Block::new(
            attribution(),
            VerticalPayload::Capability {
                record: "Proven".into(),
            },
        );
        let wire = serde_json::to_value(&block).unwrap();
        assert_eq!(
            wire["payload"],
            json!({"kind": "capability", "record": "Proven"})
        );
        assert_eq!(wire["schema_version"], json!("lonis.block/v1"));
    }

    #[test]
    fn seed_block_alias_is_block_over_block_kind() {
        let block: SeedBlock = message_block();
        assert_eq!(block.payload().kind_name(), "message");
    }

    // -- SchemaVersion (namespaced string protocol marker) --

    #[test]
    fn schema_version_accepts_canonical_markers() {
        for marker in ["lonis.block/v1", "lonis.envelope/v1", "amari.discovery/v1"] {
            let version = SchemaVersion::new(marker).unwrap();
            assert_eq!(version.as_str(), marker);
        }
    }

    #[test]
    fn schema_version_rejects_non_canonical_markers() {
        for bad in [
            "",
            "v1",
            "lonis.block/1",
            "lonis.block/v0",
            "lonis.block/v01",
            "a/b/v1",
            "Lonis.block/v1",
            "lonis block/v1",
            "lonis.block/v1/extra",
        ] {
            assert!(SchemaVersion::new(bad).is_err(), "should reject `{bad}`");
        }
    }

    #[test]
    fn schema_version_serde_transparent_and_validated() {
        let version = SchemaVersion::new("lonis.block/v1").unwrap();
        assert_eq!(
            serde_json::to_string(&version).unwrap(),
            "\"lonis.block/v1\""
        );
        let back: SchemaVersion = serde_json::from_str("\"amari.discovery/v1\"").unwrap();
        assert_eq!(back.as_str(), "amari.discovery/v1");
        assert!(serde_json::from_str::<SchemaVersion>("\"bogus\"").is_err());
    }

    #[test]
    fn schema_version_default_is_tool_protocol_v1() {
        assert_eq!(SchemaVersion::default().as_str(), "lonis.tool/v1");
    }

    // -- Attribution --

    #[test]
    fn attribution_wire_uses_doctrine_field_names() {
        let wire = serde_json::to_value(attribution()).unwrap();
        assert_eq!(
            wire,
            json!({
                "identity": "dominic",
                "viewpoint": "reviewer",
                "provenance": {
                    "when": "2026-08-10T12:00:00Z",
                    "where": "session:abc",
                    "producer": "lonis:test:fixture",
                }
            })
        );
        let back: Attribution = serde_json::from_value(wire).unwrap();
        assert_eq!(back, attribution());
    }

    #[test]
    fn attribution_omits_optional_slots() {
        let mut attr = attribution();
        attr.viewpoint = None;
        attr.provenance.location = None;
        let wire = serde_json::to_value(&attr).unwrap();
        assert!(!wire.to_string().contains("viewpoint"));
        assert!(!wire.to_string().contains("where"));
    }

    // -- Bounds --

    #[test]
    fn bounds_default_is_unbounded_and_omitted() {
        let bounds = BlockBounds::default();
        assert!(bounds.is_unbounded());
        assert_eq!(serde_json::to_value(&bounds).unwrap(), json!({}));
    }

    #[test]
    fn bounds_serialize_set_limits_only() {
        let bounds = BlockBounds {
            max_items: Some(64),
            ..BlockBounds::default()
        };
        assert!(!bounds.is_unbounded());
        assert_eq!(
            serde_json::to_value(&bounds).unwrap(),
            json!({"max_items": 64})
        );
    }

    // -- Replay provenance --

    #[test]
    fn replay_provenance_round_trips() {
        let provenance = ReplayProvenance {
            tool_version: Some("0.0.1".into()),
            compatibility: Some(Compatibility {
                status: "compatible".into(),
                reasons: Vec::new(),
            }),
            replay: ReplayMetadata {
                replayable: true,
                required_hashes: vec!["input_hash".into()],
                reasons: Vec::new(),
            },
            project_hash: Some("aa".into()),
            input_hash: Some("bb".into()),
            plan_hash: None,
            result_hash: None,
            seed: Some(42),
        };
        let wire = serde_json::to_value(&provenance).unwrap();
        assert_eq!(wire["replay"]["replayable"], json!(true));
        assert!(!wire.to_string().contains("plan_hash"));
        let back: ReplayProvenance = serde_json::from_value(wire).unwrap();
        assert_eq!(back, provenance);
    }

    #[test]
    fn empty_replay_provenance_is_empty() {
        assert!(ReplayProvenance::default().is_empty());
    }

    // -- Block --

    #[test]
    fn attribution_new_stamps_rfc3339_now() {
        let attr = Attribution::new("dominic", "lonis:test:fixture");
        assert_eq!(attr.identity, "dominic");
        assert_eq!(attr.provenance.producer, "lonis:test:fixture");
        assert!(attr.viewpoint.is_none());
        assert!(attr.provenance.location.is_none());
        // RFC 3339 shape: yyyy-mm-ddThh:mm:ss…
        let when = &attr.provenance.when;
        assert!(when.len() >= 20 && when.contains('T') && when.contains(':'));
    }

    #[test]
    fn json_content_hash_is_public_and_canonical() {
        let a = json_content_hash(&json!({"x": 1, "y": {"p": 1, "q": 2}}));
        let b = json_content_hash(&json!({"y": {"q": 2, "p": 1}, "x": 1}));
        assert_eq!(a, b);
        assert_eq!(a.len(), 64);
    }

    // -- Canonicalization hardening (ADR-0007): cross-producer number forms --

    #[test]
    fn hash_normalizes_number_forms() {
        // 100 (int), 100.0 (float), 1e2 (exponent) are the same value; a
        // Python producer emitting 100.0 and a Rust producer emitting 100
        // must agree on the hash.
        let int = json_content_hash(&json!({"n": 100}));
        let float = json_content_hash(&json!({"n": 100.0}));
        let exp =
            json_content_hash(&serde_json::from_str::<serde_json::Value>("{\"n\": 1e2}").unwrap());
        assert_eq!(int, float);
        assert_eq!(int, exp);
    }

    #[test]
    fn hash_normalizes_negative_zero() {
        let pos = json_content_hash(&json!({"n": 0}));
        let neg = json_content_hash(&json!({"n": -0.0}));
        assert_eq!(pos, neg);
    }

    #[test]
    fn hash_preserves_non_integral_floats() {
        let a = json_content_hash(&json!({"n": 0.9}));
        let b = json_content_hash(&json!({"n": 0.95}));
        assert_ne!(a, b);
        // Stability: same float, same hash.
        assert_eq!(a, json_content_hash(&json!({"n": 0.9})));
    }

    #[test]
    fn hash_handles_large_integers_exactly() {
        // u64 beyond 2^53 must not be routed through f64.
        let big: u64 = 9_007_199_254_740_993; // 2^53 + 1
        let a = json_content_hash(&json!({"n": big}));
        let b = json_content_hash(&json!({"n": big}));
        assert_eq!(a, b);
        let other = json_content_hash(&json!({"n": big - 1}));
        assert_ne!(a, other);
    }

    #[test]
    fn hash_normalizes_nested_numbers() {
        let a = json_content_hash(&json!({"outer": [{"n": 7.0}, 1e3]}));
        let b = json_content_hash(&json!({"outer": [{"n": 7}, 1000]}));
        assert_eq!(a, b);
    }

    #[test]
    fn block_wire_shape_is_flat_with_doctrine_keys() {
        let wire = serde_json::to_value(message_block()).unwrap();
        assert_eq!(
            wire,
            json!({
                "schema_version": "lonis.block/v1",
                "attribution": {
                    "identity": "dominic",
                    "viewpoint": "reviewer",
                    "provenance": {
                        "when": "2026-08-10T12:00:00Z",
                        "where": "session:abc",
                        "producer": "lonis:test:fixture",
                    }
                },
                "payload": {"kind": "message", "data": {"content": "hello"}}
            })
        );
    }

    #[test]
    fn block_round_trips() {
        let block = message_block()
            .with_bounds(BlockBounds {
                max_bytes: Some(1024),
                ..BlockBounds::default()
            })
            .with_provenance(ReplayProvenance {
                seed: Some(7),
                ..ReplayProvenance::default()
            })
            .with_warnings(vec!["truncated".into()]);
        let wire = serde_json::to_string(&block).unwrap();
        let back: Block<BlockKind> = serde_json::from_str(&wire).unwrap();
        assert_eq!(back, block);
    }

    #[test]
    fn block_denies_unknown_top_level_fields() {
        let mut wire = serde_json::to_value(message_block()).unwrap();
        wire.as_object_mut()
            .unwrap()
            .insert("bogus".into(), json!(1));
        assert!(serde_json::from_value::<Block<BlockKind>>(wire).is_err());
    }

    #[test]
    fn content_hash_is_deterministic_and_key_order_invariant() {
        let make = |data: serde_json::Value| {
            Block::new(
                attribution(),
                BlockKind::Extension {
                    kind: "widget".into(),
                    data,
                },
            )
        };
        let a = make(json!({"x": 1, "y": {"p": 1, "q": 2}}));
        let b = make(json!({"y": {"q": 2, "p": 1}, "x": 1}));
        let hash = a.content_hash();
        assert_eq!(hash, b.content_hash());
        assert_eq!(hash.len(), 64);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn content_hash_is_sensitive_to_payload() {
        let a = Block::new(
            attribution(),
            BlockKind::Message(Message {
                role: None,
                content: "a".into(),
                reply_to: None,
            }),
        );
        let b = Block::new(
            attribution(),
            BlockKind::Message(Message {
                role: None,
                content: "b".into(),
                reply_to: None,
            }),
        );
        assert_ne!(a.content_hash(), b.content_hash());
    }

    #[test]
    fn render_human_parity_renders_from_same_typed_value() {
        let block = message_block();
        let human = block.render_human();
        assert!(human.contains("message"));
        assert!(human.contains("hello"));
        // Same typed value renders the machine form too (render-parity).
        let machine = serde_json::to_value(&block).unwrap();
        assert_eq!(machine["payload"]["data"]["content"], json!("hello"));
    }

    #[test]
    fn block_kind_accessor_matches_payload() {
        let block = message_block();
        assert_eq!(block.payload().kind_name(), "message");
        assert_eq!(block.schema_id(), "lonis.block/message/v1");
    }
}
