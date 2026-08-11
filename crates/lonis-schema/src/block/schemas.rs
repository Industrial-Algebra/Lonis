// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! Curated JSON Schemas (draft 2020-12) for the block contract (ADR-0005).
//!
//! One checked-in schema per seed kind plus the envelope, embedded with
//! `include_str!` and validated on load — the amari-discovery `schema.rs`
//! registry pattern, generalized to the 14-kind corpus. The `$id`s mirror
//! the [`super::kinds::BlockKind::schema_id`] convention:
//! `https://industrialalgebra.com/schemas/lonis.block/<kind>/v1`.

use serde::Serialize;
use serde_json::Value;

use super::BLOCK_SCHEMA_V1;

const BLOCK_V1: &str = include_str!("../../schemas/block-v1.json");
const MESSAGE_V1: &str = include_str!("../../schemas/message-v1.json");
const QUESTION_V1: &str = include_str!("../../schemas/question-v1.json");
const ANSWER_V1: &str = include_str!("../../schemas/answer-v1.json");
const DECISION_V1: &str = include_str!("../../schemas/decision-v1.json");
const ACTION_V1: &str = include_str!("../../schemas/action-v1.json");
const ASSUMPTION_V1: &str = include_str!("../../schemas/assumption-v1.json");
const SUMMARY_V1: &str = include_str!("../../schemas/summary-v1.json");
const EVIDENCE_V1: &str = include_str!("../../schemas/evidence-v1.json");
const DEFINITION_V1: &str = include_str!("../../schemas/definition-v1.json");
const CAPABILITY_V1: &str = include_str!("../../schemas/capability-v1.json");
const INTENT_V1: &str = include_str!("../../schemas/intent-v1.json");
const PLAN_V1: &str = include_str!("../../schemas/plan-v1.json");
const RESULT_V1: &str = include_str!("../../schemas/result-v1.json");
const OUTCOME_V1: &str = include_str!("../../schemas/outcome-v1.json");

/// Errors from schema loading — embedded curated schemas should never be
/// malformed, so any failure here is a build/packaging defect.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("embedded schema corruption: {0}")]
pub struct SchemaError(pub String);

/// One of the curated schema families: the envelope plus the 14 seed kinds.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BlockSchemaKind {
    /// The full block envelope (payload = `oneOf` all seed kinds).
    Block,
    /// `message` payload.
    Message,
    /// `question` payload.
    Question,
    /// `answer` payload.
    Answer,
    /// `decision` payload.
    Decision,
    /// `action` payload.
    Action,
    /// `assumption` payload.
    Assumption,
    /// `summary` payload.
    Summary,
    /// `evidence` payload.
    Evidence,
    /// `definition` payload.
    Definition,
    /// `capability` payload.
    Capability,
    /// `intent` payload.
    Intent,
    /// `plan` payload.
    Plan,
    /// `result` payload.
    Result,
    /// `outcome` payload.
    Outcome,
}

impl BlockSchemaKind {
    /// Every schema family in deterministic order (envelope first).
    pub const ALL: [Self; 15] = [
        Self::Block,
        Self::Message,
        Self::Question,
        Self::Answer,
        Self::Decision,
        Self::Action,
        Self::Assumption,
        Self::Summary,
        Self::Evidence,
        Self::Definition,
        Self::Capability,
        Self::Intent,
        Self::Plan,
        Self::Result,
        Self::Outcome,
    ];

    /// The stable lowercase family name.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Block => "block",
            Self::Message => "message",
            Self::Question => "question",
            Self::Answer => "answer",
            Self::Decision => "decision",
            Self::Action => "action",
            Self::Assumption => "assumption",
            Self::Summary => "summary",
            Self::Evidence => "evidence",
            Self::Definition => "definition",
            Self::Capability => "capability",
            Self::Intent => "intent",
            Self::Plan => "plan",
            Self::Result => "result",
            Self::Outcome => "outcome",
        }
    }

    const fn source(self) -> &'static str {
        match self {
            Self::Block => BLOCK_V1,
            Self::Message => MESSAGE_V1,
            Self::Question => QUESTION_V1,
            Self::Answer => ANSWER_V1,
            Self::Decision => DECISION_V1,
            Self::Action => ACTION_V1,
            Self::Assumption => ASSUMPTION_V1,
            Self::Summary => SUMMARY_V1,
            Self::Evidence => EVIDENCE_V1,
            Self::Definition => DEFINITION_V1,
            Self::Capability => CAPABILITY_V1,
            Self::Intent => INTENT_V1,
            Self::Plan => PLAN_V1,
            Self::Result => RESULT_V1,
            Self::Outcome => OUTCOME_V1,
        }
    }
}

impl std::fmt::Display for BlockSchemaKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for BlockSchemaKind {
    type Err = SchemaError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::ALL
            .into_iter()
            .find(|kind| kind.as_str() == value)
            .ok_or_else(|| SchemaError(format!("unknown block schema kind `{value}`")))
    }
}

/// Compact identity for one available schema.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SchemaSummary {
    /// Schema family.
    pub kind: BlockSchemaKind,
    /// Stable JSON Schema `$id`.
    pub id: String,
    /// The block contract protocol version described by the schema.
    pub protocol_version: String,
}

/// Deterministically ordered catalog of available block schemas.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SchemaCatalog {
    /// All schema identities (envelope first, then the 14 seed kinds).
    pub schemas: Vec<SchemaSummary>,
}

/// One parsed curated schema and its stable identity.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct BlockSchema {
    /// Schema family.
    pub kind: BlockSchemaKind,
    /// Stable JSON Schema `$id`.
    pub id: String,
    /// The block contract protocol version described by this schema.
    pub protocol_version: String,
    /// The complete JSON Schema document.
    pub document: Value,
}

impl BlockSchema {
    /// Serializes the document as canonical pretty JSON with a trailing
    /// newline (the `lonis schema <kind>` output).
    ///
    /// # Errors
    /// Returns a serialization error when the document cannot be encoded.
    pub fn canonical_json(&self) -> Result<Vec<u8>, serde_json::Error> {
        let mut bytes = serde_json::to_vec_pretty(&self.document)?;
        bytes.push(b'\n');
        Ok(bytes)
    }
}

/// Returns all available schema identities in deterministic order.
///
/// # Errors
/// Returns [`SchemaError`] if any embedded schema fails to load.
pub fn block_schema_catalog() -> Result<SchemaCatalog, SchemaError> {
    let schemas = BlockSchemaKind::ALL
        .into_iter()
        .map(|kind| {
            let schema = block_schema(kind)?;
            Ok(SchemaSummary {
                kind,
                id: schema.id,
                protocol_version: schema.protocol_version,
            })
        })
        .collect::<Result<Vec<_>, SchemaError>>()?;
    Ok(SchemaCatalog { schemas })
}

/// Loads and validates one embedded curated schema.
///
/// # Errors
/// Returns [`SchemaError`] when the embedded document is malformed, lacks a
/// `$id`, or carries an unsupported protocol marker.
pub fn block_schema(kind: BlockSchemaKind) -> Result<BlockSchema, SchemaError> {
    let document: Value = serde_json::from_str(kind.source()).map_err(|err| {
        SchemaError(format!(
            "embedded {} schema is malformed: {err}",
            kind.as_str()
        ))
    })?;
    let id = document
        .get("$id")
        .and_then(Value::as_str)
        .ok_or_else(|| SchemaError(format!("embedded {} schema has no `$id`", kind.as_str())))?
        .to_owned();
    let protocol_version = document
        .get("x-lonis-protocol-version")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            SchemaError(format!(
                "embedded {} schema has no protocol marker",
                kind.as_str()
            ))
        })?
        .to_owned();
    if protocol_version != BLOCK_SCHEMA_V1 {
        return Err(SchemaError(format!(
            "embedded {} schema protocol `{protocol_version}` is unsupported",
            kind.as_str()
        )));
    }
    Ok(BlockSchema {
        kind,
        id,
        protocol_version,
        document,
    })
}
