// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! # lonis-schema
//!
//! The shared contract layer for Lonis-compatible tools.
//!
//! Lonis is an AI-native tool harness: it exposes sharply bounded,
//! discoverable, machine-readable tool surfaces to agents. `lonis-schema`
//! defines the types every Lonis tool (and composable CLI) depends on so the
//! harness and its consumers can discover, invoke, and parse them uniformly:
//!
//! - the [`Block`] contract (doctrine §2.7) — the canonical structured
//!   domain object every tool emits through, with the 14-kind seed corpus
//!   ([`block::BlockKind`]), attribution, bounds, replay provenance, content
//!   hashing, and render-parity,
//! - the structured [`ToolError`] (on stderr),
//! - [`OutputMode`] (human / json / ndjson),
//! - the [`Capabilities`] self-description trait,
//! - and the [`ToolContract`] a tool/probe declares.
//!
//! See `docs/plans/lonis-schema-design.md` and `docs/adr/0001-block-contract.md`
//! for the design decisions.

#![forbid(unsafe_code)]

pub mod block;

pub use block::kinds::{BlockCategory, BlockKind};
pub use block::{
    json_content_hash, now_rfc3339, Attribution, AttributionSource, Block, BlockBounds,
    BlockPayload, Compatibility, ReplayMetadata, ReplayProvenance, SeedBlock, BLOCK_SCHEMA_V1,
};

/// Re-export the derives (behind the `derive` feature).
#[cfg(feature = "derive")]
pub use lonis_derive::{BlockPayload, LonisCapabilities};

use serde::{Deserialize, Serialize};

// ===========================================================================
// Output mode
// ===========================================================================

/// How a tool renders its output. `Human` is the default; `Json` emits one
/// typed envelope on stdout; `Ndjson` streams independently-parseable envelopes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OutputMode {
    /// Pretty, human-readable (default).
    #[default]
    Human,
    /// One JSON array of [`Block`]s on stdout.
    Json,
    /// Streaming newline-delimited [`Block`]s, one per line.
    Ndjson,
}

// ===========================================================================
// Schema version (namespaced string protocol marker: <name>/v<N>)
// ===========================================================================

/// The protocol marker for the v1 tool protocol (invoke contract).
pub const TOOL_PROTOCOL_V1: &str = "lonis.tool/v1";

/// A versioned protocol marker of the form `<name>/v<N>` (e.g.
/// `lonis.envelope/v1`, `lonis.block/v1`, `amari.discovery/v1`), pinned on
/// every envelope and block so consumers can branch on shape.
///
/// The namespaced string form generalizes amari-discovery's
/// `amari.discovery/v1`: any vertical's protocol marker can be carried
/// through the same slot, which is what allows amari-discovery's
/// `protocol.rs` to eventually delete into `lonis-schema` (ADR-0001).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
#[serde(transparent)]
pub struct SchemaVersion(String);

impl SchemaVersion {
    /// Parse a protocol marker, validating the canonical `<name>/v<N>` form.
    ///
    /// The name must be non-empty lowercase ASCII (digits, `.`, `-`, `_`
    /// allowed), and the version suffix must be `v` followed by a positive
    /// integer without leading zeros.
    ///
    /// # Errors
    /// Returns [`SchemaVersionError`] if the marker is not canonical.
    pub fn new(marker: impl Into<String>) -> Result<Self, SchemaVersionError> {
        let marker = marker.into();
        let (name, version) = marker
            .rsplit_once('/')
            .ok_or_else(|| SchemaVersionError::new(&marker))?;
        let name_canonical = !name.is_empty()
            && name.bytes().all(|b| {
                b.is_ascii_lowercase() || b.is_ascii_digit() || matches!(b, b'.' | b'-' | b'_')
            });
        let version_canonical = version.strip_prefix('v').is_some_and(|n| {
            !n.is_empty() && !n.starts_with('0') && n.bytes().all(|b| b.is_ascii_digit())
        });
        if !name_canonical || !version_canonical {
            return Err(SchemaVersionError::new(&marker));
        }
        Ok(Self(marker))
    }

    /// The marker string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for SchemaVersion {
    fn default() -> Self {
        Self(TOOL_PROTOCOL_V1.to_owned())
    }
}

impl<'de> Deserialize<'de> for SchemaVersion {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error as _;
        SchemaVersion::new(String::deserialize(deserializer)?).map_err(D::Error::custom)
    }
}

/// Errors from [`SchemaVersion::new`].
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("schema version `{0}` must be a canonical `<name>/v<N>` protocol marker")]
pub struct SchemaVersionError(String);

impl SchemaVersionError {
    fn new(marker: &str) -> Self {
        Self(marker.to_owned())
    }
}

// ===========================================================================
// Tool id (namespaced: <tool>:<namespace>:<item>)
// ===========================================================================

/// A namespaced tool identifier of the form `<tool>:<namespace>:<item>`,
/// generalized from amari's `amari:<crate>:<module>:<capability>`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ToolId(String);

impl ToolId {
    /// Parse a tool id, validating the namespaced form.
    ///
    /// # Errors
    /// Returns [`ToolIdError`] if the id has fewer than two non-empty
    /// `:`-separated segments or contains an empty segment.
    pub fn new(id: impl Into<String>) -> Result<Self, ToolIdError> {
        let id = id.into();
        let parts: Vec<&str> = id.split(':').collect();
        if parts.len() < 2 {
            return Err(ToolIdError::TooFewSegments);
        }
        if parts.iter().any(|p| p.is_empty()) {
            return Err(ToolIdError::EmptySegment);
        }
        Ok(Self(id))
    }

    /// The `:`-separated segments of the id.
    #[must_use]
    pub fn parts(&self) -> Vec<&str> {
        self.0.split(':').collect()
    }

    /// The raw id string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Errors from [`ToolId::new`].
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ToolIdError {
    /// Fewer than two segments.
    #[error("tool id must have at least two ':'-separated segments")]
    TooFewSegments,
    /// An empty segment.
    #[error("tool id segments must be non-empty")]
    EmptySegment,
}

// ===========================================================================
// Tool error (structured, on stderr)
// ===========================================================================

/// A structured tool error, serialized to **stderr** in machine output modes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolError {
    /// Stable, machine-readable error kind.
    pub kind: String,
    /// Short human-readable message.
    pub message: String,
    /// Structured, machine-readable details (optional).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
    /// Stable exit code for this error.
    pub exit_code: u8,
}

impl ToolError {
    /// Construct a tool error.
    #[must_use]
    pub fn new(kind: impl Into<String>, message: impl Into<String>, exit_code: u8) -> Self {
        Self {
            kind: kind.into(),
            message: message.into(),
            details: None,
            exit_code,
        }
    }

    /// Attach structured details.
    #[must_use]
    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }
}

impl std::fmt::Display for ToolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self { kind, message, .. } = self;
        write!(f, "{kind}: {message}")
    }
}

impl std::error::Error for ToolError {}

/// Shared baseline exit-code vocabulary. Tools define their own map; these are
/// the common baselines self-described via [`Capabilities::exit_code_map`].
pub mod exit_code {
    /// Success.
    pub const SUCCESS: u8 = 0;
    /// Generic failure.
    pub const GENERIC: u8 = 1;
    /// Invalid input.
    pub const INVALID_INPUT: u8 = 2;
    /// Not found.
    pub const NOT_FOUND: u8 = 3;
    /// Confirmation required for a destructive / unsafe action.
    pub const CONFIRMATION_REQUIRED: u8 = 4;
    /// Rate limited.
    pub const RATE_LIMITED: u8 = 5;
    /// The tool failed to complete its operation.
    pub const TOOL_FAILED: u8 = 6;
    /// A declared resource limit was exceeded.
    pub const LIMIT_EXCEEDED: u8 = 7;
    /// An I/O failure.
    pub const IO: u8 = 8;
    /// A serialization / deserialization failure.
    pub const SERIALIZATION: u8 = 9;
    /// The operation is not implemented.
    pub const NOT_IMPLEMENTED: u8 = 69;
    /// An internal error.
    pub const INTERNAL: u8 = 70;
}

// ===========================================================================
// Capabilities (trait — tools extend with domain states)
// ===========================================================================

/// Self-description trait. Every Lonis tool implements this so the harness and
/// agents can discover what it does, what it emits, and how it signals failure
/// (decision #2: trait-based).
pub trait Capabilities {
    /// The schema version of the tool's protocol.
    fn schema_version(&self) -> SchemaVersion;
    /// The tool's version string.
    fn tool_version(&self) -> &str;
    /// Output formats the tool supports.
    fn output_formats(&self) -> &'static [OutputMode];
    /// The stable exit-code map: `(kind, exit_code)` pairs.
    fn exit_code_map(&self) -> &'static [(&'static str, u8)];
    /// The tool's id.
    fn tool_id(&self) -> ToolId;
}

// ===========================================================================
// Tool contract
// ===========================================================================

/// How deterministic a tool's output is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Determinism {
    /// Fully deterministic: same input always yields identical output.
    Deterministic,
    /// Deterministic given a seed; otherwise bounded nondeterminism.
    BoundedNondeterministic,
    /// Nondeterministic.
    Nondeterministic,
}

/// The side-effect class of a tool.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SideEffects {
    /// No side effects.
    None,
    /// Reads only.
    ReadOnly,
    /// Writes the filesystem.
    WritesFilesystem,
    /// Mutates an external application / system.
    MutatesExternal,
    /// Network access.
    Network,
    /// Destructive.
    Destructive,
}

/// Cost hint.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Cost {
    /// Low cost.
    Low,
    /// Medium cost.
    Medium,
    /// High cost.
    High,
}

/// Reference to an input or output schema (e.g. a JSON-schema ref or a
/// versioned type identifier).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SchemaRef(pub String);

/// A declared tool / probe contract.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolContract {
    /// Namespaced tool id.
    pub name: ToolId,
    /// Short purpose description.
    pub description: String,
    /// Input schema reference.
    pub input_schema: SchemaRef,
    /// Output schema reference.
    pub output_schema: SchemaRef,
    /// Determinism level.
    pub determinism: Determinism,
    /// Side-effect class.
    pub side_effects: SideEffects,
    /// Cost hint.
    pub cost: Cost,
    /// Required capabilities.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub capabilities: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn output_mode_default_is_human() {
        assert_eq!(OutputMode::default(), OutputMode::Human);
    }

    #[test]
    fn output_mode_serde() {
        assert_eq!(
            serde_json::to_string(&OutputMode::Json).unwrap(),
            "\"json\""
        );
        assert_eq!(
            serde_json::to_string(&OutputMode::Ndjson).unwrap(),
            "\"ndjson\""
        );
        let mode: OutputMode = serde_json::from_str("\"human\"").unwrap();
        assert_eq!(mode, OutputMode::Human);
    }

    #[test]
    fn tool_id_accepts_namespaced() {
        let id = ToolId::new("amari:discovery:search").unwrap();
        assert_eq!(id.as_str(), "amari:discovery:search");
        assert_eq!(id.parts(), vec!["amari", "discovery", "search"]);
    }

    #[test]
    fn tool_id_rejects_single_segment() {
        assert_eq!(
            ToolId::new("lonis").unwrap_err(),
            ToolIdError::TooFewSegments
        );
    }

    #[test]
    fn tool_id_rejects_empty_segment() {
        assert_eq!(
            ToolId::new("amari::search").unwrap_err(),
            ToolIdError::EmptySegment
        );
    }

    #[test]
    fn tool_error_serializes() {
        let err = ToolError::new("not_found", "no such probe", exit_code::NOT_FOUND)
            .with_details(serde_json::json!({"id": "p1"}));
        let json = serde_json::to_string(&err).unwrap();
        assert!(json.contains("\"kind\":\"not_found\""));
        assert!(json.contains("\"exit_code\":3"));
        assert!(json.contains("\"id\":\"p1\""));
    }

    #[test]
    fn tool_error_display() {
        let err = ToolError::new("x", "boom", exit_code::GENERIC);
        assert_eq!(format!("{err}"), "x: boom");
    }

    #[test]
    fn determinism_serde_kebab() {
        assert_eq!(
            serde_json::to_string(&Determinism::BoundedNondeterministic).unwrap(),
            "\"bounded-nondeterministic\""
        );
    }

    #[test]
    fn tool_contract_round_trips() {
        let contract = ToolContract {
            name: ToolId::new("amari:probe:run").unwrap(),
            description: "run a probe".into(),
            input_schema: SchemaRef("amari.discovery/v1#ProbeRequest".into()),
            output_schema: SchemaRef("amari.discovery/v1#ProbeResult".into()),
            determinism: Determinism::BoundedNondeterministic,
            side_effects: SideEffects::None,
            cost: Cost::Medium,
            capabilities: vec!["model_inference".into()],
        };
        let json = serde_json::to_string(&contract).unwrap();
        let back: ToolContract = serde_json::from_str(&json).unwrap();
        assert_eq!(back, contract);
    }

    // --- Capabilities trait usage ---

    struct DummyCapabilities;
    const FMTS: [OutputMode; 3] = [OutputMode::Human, OutputMode::Json, OutputMode::Ndjson];
    const MAP: &[(&str, u8)] = &[
        ("ok", exit_code::SUCCESS),
        ("not_found", exit_code::NOT_FOUND),
    ];

    impl Capabilities for DummyCapabilities {
        fn schema_version(&self) -> SchemaVersion {
            SchemaVersion::default()
        }
        fn tool_version(&self) -> &str {
            "0.0.1"
        }
        fn output_formats(&self) -> &'static [OutputMode] {
            &FMTS
        }
        fn exit_code_map(&self) -> &'static [(&'static str, u8)] {
            MAP
        }
        fn tool_id(&self) -> ToolId {
            ToolId::new("lonis:test:dummy").unwrap()
        }
    }

    #[test]
    fn capabilities_impl_works() {
        let caps = DummyCapabilities;
        assert_eq!(caps.tool_id().as_str(), "lonis:test:dummy");
        assert_eq!(caps.output_formats().len(), 3);
        assert_eq!(caps.exit_code_map().len(), 2);
        assert_eq!(caps.tool_version(), "0.0.1");
    }
}
