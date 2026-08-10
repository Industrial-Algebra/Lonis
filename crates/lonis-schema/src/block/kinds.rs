// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! The 14 seed block kinds (doctrine §2.7) and their typed payloads.
//!
//! Three categories, extensible by domains: unknown kinds deserialize into
//! [`BlockKind::Extension`] and re-serialize identically, so a consumer on an
//! older contract tolerates newer kinds.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::ToolContract;

// ===========================================================================
// Category 1 — participant-stream primitives
// ===========================================================================

/// A participant message in the stream.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Message {
    /// The speaker's role, when applicable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    /// The message content.
    pub content: String,
    /// Content hash or id of the block this replies to, when applicable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_to: Option<String>,
}

/// A question posed to a participant.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Question {
    /// The question text.
    pub text: String,
    /// Structured context for the question, when applicable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context: Option<Value>,
    /// Offered answer options (empty = free-form).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub options: Vec<String>,
}

/// An answer to a question.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Answer {
    /// The question text or reference being answered, when known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub question: Option<String>,
    /// The answer text.
    pub text: String,
    /// Confidence in `[0, 1]`, when assessed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confidence: Option<f64>,
}

/// A decision reached in the stream.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Decision {
    /// What was decided.
    pub statement: String,
    /// Why, when recorded.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rationale: Option<String>,
    /// Alternatives considered and not taken.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub alternatives: Vec<String>,
}

/// The lifecycle state of an [`Action`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionStatus {
    /// Declared but not started.
    Pending,
    /// Currently executing.
    Running,
    /// Finished successfully.
    Completed,
    /// Finished unsuccessfully.
    Failed,
}

/// An action taken or proposed.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Action {
    /// What is done (e.g. `invoke`, `write`, `deploy`).
    pub verb: String,
    /// The target of the verb, when applicable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    /// Structured parameters, when applicable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parameters: Option<Value>,
    /// Lifecycle state.
    pub status: ActionStatus,
}

/// The epistemic state of an [`Assumption`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssumptionStatus {
    /// Not yet checked.
    Open,
    /// Supported by evidence or a probe.
    Validated,
    /// Contradicted by evidence or a probe.
    Refuted,
}

/// An assumption the stream depends on.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Assumption {
    /// The assumption statement.
    pub statement: String,
    /// Epistemic state.
    pub status: AssumptionStatus,
}

/// A compression of prior blocks.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Summary {
    /// Content hashes or ids of the blocks summarized.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub of: Vec<String>,
    /// The summary text.
    pub text: String,
}

// ===========================================================================
// Category 2 — knowledge / definition primitives
// ===========================================================================

/// An observed fact with its source and hash.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Evidence {
    /// Stable evidence category.
    pub kind: String,
    /// Concise human-readable description.
    pub summary: String,
    /// Source path, command, or record, when applicable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    /// Content hash of the observed artifact, when applicable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hash: Option<String>,
    /// Relative weight used by ranking, when applicable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub weight: Option<f64>,
}

/// A vocabulary or API record: stable id plus a semantic overlay.
///
/// The field set generalizes karpal-index's `ApiItem`; the `overlay` slot
/// carries domain-specific relations (e.g. supertraits, implementors).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    /// Stable identifier for the defined item.
    pub id: String,
    /// The item's name.
    pub name: String,
    /// What sort of item it is (e.g. `struct`, `trait`, `function`).
    pub kind: String,
    /// Source location, when known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    /// Signature, when applicable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
    /// One-line summary, when available.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    /// Full documentation, when available.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub docs: Option<String>,
    /// Domain semantic overlay (relations, rankings, embeddings).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub overlay: Option<Value>,
}

/// A typed want: a statement of intent plus explicit constraints.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Intent {
    /// What is wanted.
    pub statement: String,
    /// Explicit constraints on satisfying the intent.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub constraints: Vec<String>,
}

// ===========================================================================
// Category 3 — process primitives
// ===========================================================================

/// One ordered step in a [`Plan`].
///
/// Deliberately open (`kind` + `detail`): the horizontal contract cannot
/// enumerate every domain's step vocabulary. Verticals keep their own typed
/// step unions (e.g. amari-discovery's six-variant `PlanStep`) inside
/// `detail`; `kind` is the stable discriminant.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlanStep {
    /// Stable step kind (e.g. `dependency`, `probe`, `test`).
    pub kind: String,
    /// Step-specific structured detail.
    pub detail: Value,
}

/// One deterministic rewrite applied during plan normalization.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NormalizationTrace {
    /// Steps immediately before the rewrite.
    pub before: Vec<PlanStep>,
    /// Steps immediately after the rewrite.
    pub after: Vec<PlanStep>,
}

/// Completion and trace metadata for plan normalization.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Normalization {
    /// Whether the plan reached a rewrite fixed point within its limits.
    pub normalized: bool,
    /// Maximum rewrites allowed for this normalization attempt.
    pub max_rewrites: usize,
    /// Applied rewrites in deterministic order.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub trace: Vec<NormalizationTrace>,
}

/// An ordered, replayable plan.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Plan {
    /// The goal the plan serves, when stated.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub goal: Option<String>,
    /// Ordered steps.
    pub steps: Vec<PlanStep>,
    /// Canonical prerequisite-first order of referenced capability ids.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub prerequisite_order: Vec<String>,
    /// Bounded rewrite-normalization metadata, when normalized.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub normalization: Option<Normalization>,
    /// Hash of canonical plan content excluding trace metadata.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plan_hash: Option<String>,
}

/// Typed resource usage observed while producing a result.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResourceUse {
    /// Domain operations performed.
    pub operations: u64,
    /// Graph or term nodes visited.
    pub nodes: u64,
    /// Iterative steps performed.
    pub iterations: u64,
    /// Input and output bytes accounted.
    pub bytes: u64,
}

/// What happened when work ran: output, score, evidence, assumption
/// bookkeeping, and resource usage.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResultPayload {
    /// The structured output.
    pub output: Value,
    /// Score, when the result is ranked.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub score: Option<f64>,
    /// Evidence supporting the result.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence: Vec<Evidence>,
    /// Assumptions validated by this result.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub validated_assumptions: Vec<String>,
    /// Assumptions refuted by this result.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub refuted_assumptions: Vec<String>,
    /// Observed resource usage, when accounted.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resources: Option<ResourceUse>,
    /// Elapsed execution time in microseconds, when measured.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration_micros: Option<u64>,
}

/// The class of an [`Outcome`] — structured, non-exceptional domain results
/// and structured errors share one kind (doctrine §2.7 `outcome`/`error`).
/// Generalizes amari-discovery's `DiscoveryOutcome` and schubert's
/// `AccessDecision`: quantitative, exhaustive, never a bare boolean.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutcomeStatus {
    /// The operation succeeded fully.
    Success,
    /// The operation succeeded with caveats (see `details`).
    Partial,
    /// Available evidence rules out all candidates.
    NoMatch,
    /// More evidence is required.
    InsufficientEvidence,
    /// Preconditions explicitly prevent completion.
    Blocked,
    /// A structured domain error (with `kind` / `message` / `exit_code`).
    Error,
}

/// A structured domain outcome or error: the block-level generalization of
/// [`crate::ToolError`] into the stream.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Outcome {
    /// The outcome class.
    pub status: OutcomeStatus,
    /// Stable, machine-readable subkind.
    pub kind: String,
    /// Short human-readable message.
    pub message: String,
    /// Structured details, when applicable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub details: Option<Value>,
    /// Stable exit code, when the outcome terminates an invocation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<u8>,
}

// ===========================================================================
// BlockKind — the extensible 14-kind payload enum
// ===========================================================================

/// The three doctrine categories, plus the extension bucket.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BlockCategory {
    /// Participant-stream primitives (7 seed kinds).
    ParticipantStream,
    /// Knowledge / definition primitives (4 seed kinds).
    Knowledge,
    /// Process primitives (3 seed kinds).
    Process,
    /// A domain-registered kind outside the seed corpus.
    Extension,
}

/// A block payload: one of the 14 seed kinds, or a domain extension.
///
/// The wire form is `{"kind": <name>, "data": <payload>}`. Unknown `kind`
/// values deserialize into [`BlockKind::Extension`] and re-serialize
/// identically — domains register new kinds without breaking older
/// consumers.
#[derive(Debug, Clone, PartialEq)]
pub enum BlockKind {
    /// A participant message.
    Message(Message),
    /// A question.
    Question(Question),
    /// An answer.
    Answer(Answer),
    /// A decision.
    Decision(Decision),
    /// An action.
    Action(Action),
    /// An assumption.
    Assumption(Assumption),
    /// A summary.
    Summary(Summary),
    /// An observed fact.
    Evidence(Evidence),
    /// A vocabulary / API record.
    Definition(Definition),
    /// A tool self-description (reuses [`ToolContract`]).
    Capability(ToolContract),
    /// A typed want.
    Intent(Intent),
    /// An ordered replayable plan.
    Plan(Plan),
    /// What happened when work ran.
    Result(ResultPayload),
    /// A structured domain outcome or error.
    Outcome(Outcome),
    /// A domain-registered kind outside the seed corpus.
    Extension {
        /// The registered kind name.
        kind: String,
        /// The domain payload.
        data: Value,
    },
}

impl BlockKind {
    /// The wire discriminant (e.g. `message`).
    #[must_use]
    pub fn kind_name(&self) -> &str {
        match self {
            Self::Message(_) => "message",
            Self::Question(_) => "question",
            Self::Answer(_) => "answer",
            Self::Decision(_) => "decision",
            Self::Action(_) => "action",
            Self::Assumption(_) => "assumption",
            Self::Summary(_) => "summary",
            Self::Evidence(_) => "evidence",
            Self::Definition(_) => "definition",
            Self::Capability(_) => "capability",
            Self::Intent(_) => "intent",
            Self::Plan(_) => "plan",
            Self::Result(_) => "result",
            Self::Outcome(_) => "outcome",
            Self::Extension { kind, .. } => kind,
        }
    }

    /// The doctrine category of this kind.
    #[must_use]
    pub fn category(&self) -> BlockCategory {
        match self {
            Self::Message(_)
            | Self::Question(_)
            | Self::Answer(_)
            | Self::Decision(_)
            | Self::Action(_)
            | Self::Assumption(_)
            | Self::Summary(_) => BlockCategory::ParticipantStream,
            Self::Evidence(_) | Self::Definition(_) | Self::Capability(_) | Self::Intent(_) => {
                BlockCategory::Knowledge
            }
            Self::Plan(_) | Self::Result(_) | Self::Outcome(_) => BlockCategory::Process,
            Self::Extension { .. } => BlockCategory::Extension,
        }
    }

    /// The stable `$id` of this kind (e.g. `lonis.block/message/v1`).
    #[must_use]
    pub fn schema_id(&self) -> String {
        format!("lonis.block/{}/v1", self.kind_name())
    }

    /// Render the human-facing form (render-parity: same typed value the
    /// machine form serializes from).
    #[must_use]
    pub fn render_human(&self) -> String {
        match self {
            Self::Message(m) => format!("message: {}", m.content),
            Self::Question(q) => format!("question: {}", q.text),
            Self::Answer(a) => format!("answer: {}", a.text),
            Self::Decision(d) => format!("decision: {}", d.statement),
            Self::Action(a) => format!("action: {} {:?}", a.verb, a.status),
            Self::Assumption(a) => format!("assumption: {} ({:?})", a.statement, a.status),
            Self::Summary(s) => format!("summary: {}", s.text),
            Self::Evidence(e) => format!("evidence[{}]: {}", e.kind, e.summary),
            Self::Definition(d) => format!("definition: {} ({})", d.name, d.kind),
            Self::Capability(c) => format!("capability: {} — {}", c.name.as_str(), c.description),
            Self::Intent(i) => format!("intent: {}", i.statement),
            Self::Plan(p) => format!("plan: {} step(s)", p.steps.len()),
            Self::Result(r) => format!(
                "result: {} assumption(s) validated",
                r.validated_assumptions.len()
            ),
            Self::Outcome(o) => format!("outcome[{:?}]: {}", o.status, o.message),
            Self::Extension { kind, data } => format!("extension[{kind}]: {data}"),
        }
    }
}

impl Serialize for BlockKind {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeMap as _;
        let mut map = serializer.serialize_map(Some(2))?;
        map.serialize_entry("kind", self.kind_name())?;
        match self {
            Self::Message(p) => map.serialize_entry("data", p)?,
            Self::Question(p) => map.serialize_entry("data", p)?,
            Self::Answer(p) => map.serialize_entry("data", p)?,
            Self::Decision(p) => map.serialize_entry("data", p)?,
            Self::Action(p) => map.serialize_entry("data", p)?,
            Self::Assumption(p) => map.serialize_entry("data", p)?,
            Self::Summary(p) => map.serialize_entry("data", p)?,
            Self::Evidence(p) => map.serialize_entry("data", p)?,
            Self::Definition(p) => map.serialize_entry("data", p)?,
            Self::Capability(p) => map.serialize_entry("data", p)?,
            Self::Intent(p) => map.serialize_entry("data", p)?,
            Self::Plan(p) => map.serialize_entry("data", p)?,
            Self::Result(p) => map.serialize_entry("data", p)?,
            Self::Outcome(p) => map.serialize_entry("data", p)?,
            Self::Extension { data, .. } => map.serialize_entry("data", data)?,
        }
        map.end()
    }
}

impl<'de> Deserialize<'de> for BlockKind {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error as _;
        let mut wire = serde_json::Map::<String, Value>::deserialize(deserializer)?;
        let kind = wire
            .remove("kind")
            .and_then(|v| v.as_str().map(str::to_owned))
            .ok_or_else(|| D::Error::custom("block payload requires a string `kind`"))?;
        let data = wire
            .remove("data")
            .ok_or_else(|| D::Error::custom("block payload requires `data`"))?;
        if !wire.is_empty() {
            return Err(D::Error::custom("block payload has unknown fields"));
        }
        fn parse<T: serde::de::DeserializeOwned, E: serde::de::Error>(data: Value) -> Result<T, E> {
            serde_json::from_value(data).map_err(E::custom)
        }
        match kind.as_str() {
            "message" => Ok(Self::Message(parse(data)?)),
            "question" => Ok(Self::Question(parse(data)?)),
            "answer" => Ok(Self::Answer(parse(data)?)),
            "decision" => Ok(Self::Decision(parse(data)?)),
            "action" => Ok(Self::Action(parse(data)?)),
            "assumption" => Ok(Self::Assumption(parse(data)?)),
            "summary" => Ok(Self::Summary(parse(data)?)),
            "evidence" => Ok(Self::Evidence(parse(data)?)),
            "definition" => Ok(Self::Definition(parse(data)?)),
            "capability" => Ok(Self::Capability(parse(data)?)),
            "intent" => Ok(Self::Intent(parse(data)?)),
            "plan" => Ok(Self::Plan(parse(data)?)),
            "result" => Ok(Self::Result(parse(data)?)),
            "outcome" => Ok(Self::Outcome(parse(data)?)),
            _ => Ok(Self::Extension { kind, data }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn message() -> Message {
        Message {
            role: Some("agent".into()),
            content: "hello".into(),
            reply_to: None,
        }
    }

    fn every_kind() -> Vec<BlockKind> {
        vec![
            BlockKind::Message(message()),
            BlockKind::Question(Question {
                text: "proceed?".into(),
                context: None,
                options: vec!["yes".into(), "no".into()],
            }),
            BlockKind::Answer(Answer {
                question: Some("proceed?".into()),
                text: "yes".into(),
                confidence: Some(0.9),
            }),
            BlockKind::Decision(Decision {
                statement: "use Vec<Block>".into(),
                rationale: Some("generalizes 0/1/N".into()),
                alternatives: vec!["single Block".into()],
            }),
            BlockKind::Action(Action {
                verb: "invoke".into(),
                target: Some("lonis:builtin:echo".into()),
                parameters: Some(json!({"input": {}})),
                status: ActionStatus::Pending,
            }),
            BlockKind::Assumption(Assumption {
                statement: "the catalog is current".into(),
                status: AssumptionStatus::Open,
            }),
            BlockKind::Summary(Summary {
                of: vec!["abc123".into()],
                text: "we decided things".into(),
            }),
            BlockKind::Evidence(Evidence {
                kind: "observation".into(),
                summary: "tests pass".into(),
                source: Some("cargo test".into()),
                hash: Some("deadbeef".into()),
                weight: Some(1.0),
            }),
            BlockKind::Definition(Definition {
                id: "karpal:karpal-proof:Proven".into(),
                name: "Proven".into(),
                kind: "struct".into(),
                path: Some("karpal-proof/src/proof.rs:12".into()),
                signature: Some("pub struct Proven<T>".into()),
                summary: Some("A value with a proof witness.".into()),
                docs: None,
                overlay: Some(json!({"implementors": ["Goal"]})),
            }),
            BlockKind::Capability(crate::ToolContract {
                name: crate::ToolId::new("lonis:builtin:echo").unwrap(),
                description: "echo input".into(),
                input_schema: crate::SchemaRef("lonis.echo/input/v1".into()),
                output_schema: crate::SchemaRef("lonis.echo/output/v1".into()),
                determinism: crate::Determinism::Deterministic,
                side_effects: crate::SideEffects::None,
                cost: crate::Cost::Low,
                capabilities: Vec::new(),
            }),
            BlockKind::Intent(Intent {
                statement: "recall similar decisions".into(),
                constraints: vec!["bounded to 64 items".into()],
            }),
            BlockKind::Plan(Plan {
                goal: Some("integrate holographic recall".into()),
                steps: vec![PlanStep {
                    kind: "dependency".into(),
                    detail: json!({"package": "amari-holographic", "version": "0.24.1"}),
                }],
                prerequisite_order: vec!["amari:amari-holographic:memory:retrieval".into()],
                normalization: Some(Normalization {
                    normalized: true,
                    max_rewrites: 4096,
                    trace: vec![NormalizationTrace {
                        before: vec![PlanStep {
                            kind: "dependency".into(),
                            detail: json!({"package": "a"}),
                        }],
                        after: vec![PlanStep {
                            kind: "dependency".into(),
                            detail: json!({"package": "a"}),
                        }],
                    }],
                }),
                plan_hash: Some("cafe".into()),
            }),
            BlockKind::Result(ResultPayload {
                output: json!({"nim_sum": 1}),
                score: Some(0.95),
                evidence: vec![Evidence {
                    kind: "probe".into(),
                    summary: "nim-sum probe".into(),
                    source: None,
                    hash: None,
                    weight: None,
                }],
                validated_assumptions: vec!["catalog current".into()],
                refuted_assumptions: Vec::new(),
                resources: Some(ResourceUse {
                    operations: 118,
                    nodes: 8,
                    iterations: 11,
                    bytes: 54,
                }),
                duration_micros: Some(87205),
            }),
            BlockKind::Outcome(Outcome {
                status: OutcomeStatus::Blocked,
                kind: "precondition_failed".into(),
                message: "policy forbids".into(),
                details: Some(json!({"policy": "no-network"})),
                exit_code: Some(4),
            }),
        ]
    }

    #[test]
    fn seed_corpus_has_fourteen_kinds() {
        assert_eq!(every_kind().len(), 14);
    }

    #[test]
    fn all_fourteen_kinds_round_trip() {
        for kind in every_kind() {
            let json = serde_json::to_string(&kind).unwrap();
            let back: BlockKind = serde_json::from_str(&json).unwrap();
            assert_eq!(back, kind, "kind {:?} did not round-trip", kind.kind_name());
        }
    }

    #[test]
    fn schema_id_per_kind() {
        let expected = [
            ("message", "lonis.block/message/v1"),
            ("question", "lonis.block/question/v1"),
            ("answer", "lonis.block/answer/v1"),
            ("decision", "lonis.block/decision/v1"),
            ("action", "lonis.block/action/v1"),
            ("assumption", "lonis.block/assumption/v1"),
            ("summary", "lonis.block/summary/v1"),
            ("evidence", "lonis.block/evidence/v1"),
            ("definition", "lonis.block/definition/v1"),
            ("capability", "lonis.block/capability/v1"),
            ("intent", "lonis.block/intent/v1"),
            ("plan", "lonis.block/plan/v1"),
            ("result", "lonis.block/result/v1"),
            ("outcome", "lonis.block/outcome/v1"),
        ];
        for (kind, (name, id)) in every_kind().iter().zip(expected) {
            assert_eq!(kind.kind_name(), name);
            assert_eq!(kind.schema_id(), id);
        }
    }

    #[test]
    fn categories_partition_the_seed_corpus() {
        let kinds = every_kind();
        let stream = kinds
            .iter()
            .filter(|k| k.category() == BlockCategory::ParticipantStream)
            .count();
        let knowledge = kinds
            .iter()
            .filter(|k| k.category() == BlockCategory::Knowledge)
            .count();
        let process = kinds
            .iter()
            .filter(|k| k.category() == BlockCategory::Process)
            .count();
        assert_eq!((stream, knowledge, process), (7, 4, 3));
    }

    #[test]
    fn unknown_kind_deserializes_to_extension() {
        let wire = json!({"kind": "widget", "data": {"gadget": 1}});
        let kind: BlockKind = serde_json::from_value(wire.clone()).unwrap();
        assert_eq!(
            kind,
            BlockKind::Extension {
                kind: "widget".into(),
                data: json!({"gadget": 1}),
            }
        );
        assert_eq!(kind.kind_name(), "widget");
        assert_eq!(kind.schema_id(), "lonis.block/widget/v1");
        assert_eq!(kind.category(), BlockCategory::Extension);
        // Re-serializes to the identical wire shape.
        assert_eq!(serde_json::to_value(&kind).unwrap(), wire);
    }

    #[test]
    fn wire_shape_is_tagged_kind_plus_data() {
        let wire = serde_json::to_value(BlockKind::Message(message())).unwrap();
        assert_eq!(
            wire,
            json!({"kind": "message", "data": {"role": "agent", "content": "hello"}})
        );
    }

    #[test]
    fn outcome_status_serde_snake_case() {
        let cases = [
            (OutcomeStatus::Success, "success"),
            (OutcomeStatus::Partial, "partial"),
            (OutcomeStatus::NoMatch, "no_match"),
            (OutcomeStatus::InsufficientEvidence, "insufficient_evidence"),
            (OutcomeStatus::Blocked, "blocked"),
            (OutcomeStatus::Error, "error"),
        ];
        for (status, name) in cases {
            assert_eq!(
                serde_json::to_string(&status).unwrap(),
                format!("\"{name}\"")
            );
            let back: OutcomeStatus = serde_json::from_str(&format!("\"{name}\"")).unwrap();
            assert_eq!(back, status);
        }
    }

    #[test]
    fn action_and_assumption_status_serde() {
        assert_eq!(
            serde_json::to_string(&ActionStatus::Running).unwrap(),
            "\"running\""
        );
        assert_eq!(
            serde_json::to_string(&AssumptionStatus::Validated).unwrap(),
            "\"validated\""
        );
        assert_eq!(
            serde_json::to_string(&AssumptionStatus::Refuted).unwrap(),
            "\"refuted\""
        );
    }

    #[test]
    fn payloads_deny_unknown_fields() {
        let bad = json!({"kind": "message", "data": {"content": "hi", "bogus": 1}});
        assert!(serde_json::from_value::<BlockKind>(bad).is_err());
    }

    #[test]
    fn render_human_is_nonempty_and_kind_aware() {
        for kind in every_kind() {
            let rendered = kind.render_human();
            assert!(!rendered.is_empty());
            assert!(
                rendered.contains(kind.kind_name()),
                "render for {:?} should mention its kind",
                kind.kind_name()
            );
        }
    }
}
