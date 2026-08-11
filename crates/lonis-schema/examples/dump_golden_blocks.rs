// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! Regenerates the checked-in golden wire fixtures under
//! `tests/golden/blocks/` (one block per seed kind, plus `hashes.json`
//! pinning each block's content hash). Run after any intentional wire-shape
//! change; the schema tests treat these as immutable pins otherwise.

use std::path::PathBuf;

use lonis_schema::block::kinds::*;
use lonis_schema::{
    Attribution, AttributionSource, Block, Cost, Determinism, SchemaRef, SeedBlock, SideEffects,
    ToolContract, ToolId,
};

fn attribution() -> Attribution {
    Attribution {
        identity: "lonis:test:golden".into(),
        viewpoint: None,
        provenance: AttributionSource {
            when: "2026-08-10T00:00:00Z".into(),
            location: None,
            producer: "lonis:test:golden".into(),
        },
    }
}

fn block(payload: BlockKind) -> SeedBlock {
    Block::new(attribution(), payload)
}

fn goldens() -> Vec<(&'static str, SeedBlock)> {
    vec![
        (
            "message",
            block(BlockKind::Message(Message {
                role: Some("agent".into()),
                content: "hello".into(),
                reply_to: None,
            })),
        ),
        (
            "question",
            block(BlockKind::Question(Question {
                text: "proceed?".into(),
                context: None,
                options: vec!["yes".into(), "no".into()],
            })),
        ),
        (
            "answer",
            block(BlockKind::Answer(Answer {
                question: Some("proceed?".into()),
                text: "yes".into(),
                confidence: Some(0.9),
            })),
        ),
        (
            "decision",
            block(BlockKind::Decision(Decision {
                statement: "use Vec<Block>".into(),
                rationale: Some("generalizes 0/1/N".into()),
                alternatives: vec!["single Block".into()],
            })),
        ),
        (
            "action",
            block(BlockKind::Action(Action {
                verb: "invoke".into(),
                target: Some("lonis:builtin:echo".into()),
                parameters: Some(serde_json::json!({"input": {}})),
                status: ActionStatus::Pending,
            })),
        ),
        (
            "assumption",
            block(BlockKind::Assumption(Assumption {
                statement: "the catalog is current".into(),
                status: AssumptionStatus::Open,
            })),
        ),
        (
            "summary",
            block(BlockKind::Summary(Summary {
                of: vec!["abc123".into()],
                text: "we decided things".into(),
            })),
        ),
        (
            "evidence",
            block(BlockKind::Evidence(Evidence {
                kind: "observation".into(),
                summary: "tests pass".into(),
                source: Some("cargo test".into()),
                hash: Some("deadbeef".into()),
                weight: Some(1.0),
            })),
        ),
        (
            "definition",
            block(BlockKind::Definition(Definition {
                id: "karpal:karpal-proof:Proven".into(),
                name: "Proven".into(),
                kind: "struct".into(),
                path: Some("karpal-proof/src/proof.rs:12".into()),
                signature: Some("pub struct Proven<T>".into()),
                summary: Some("A value with a proof witness.".into()),
                docs: None,
                overlay: Some(serde_json::json!({"implementors": ["Goal"]})),
            })),
        ),
        (
            "capability",
            block(BlockKind::Capability(ToolContract {
                name: ToolId::new("lonis:builtin:echo").unwrap(),
                description: "echo input".into(),
                input_schema: SchemaRef("lonis.echo/input/v1".into()),
                output_schema: SchemaRef("lonis.echo/output/v1".into()),
                determinism: Determinism::Deterministic,
                side_effects: SideEffects::None,
                cost: Cost::Low,
                capabilities: Vec::new(),
            })),
        ),
        (
            "intent",
            block(BlockKind::Intent(Intent {
                statement: "recall similar decisions".into(),
                constraints: vec!["bounded to 64 items".into()],
            })),
        ),
        (
            "plan",
            block(BlockKind::Plan(Plan {
                goal: Some("integrate holographic recall".into()),
                steps: vec![PlanStep {
                    kind: "dependency".into(),
                    detail: serde_json::json!({"package": "amari-holographic", "version": "0.24.1"}),
                }],
                prerequisite_order: vec!["amari:amari-holographic:memory:retrieval".into()],
                normalization: Some(Normalization {
                    normalized: true,
                    max_rewrites: 4096,
                    trace: vec![NormalizationTrace {
                        before: vec![PlanStep {
                            kind: "dependency".into(),
                            detail: serde_json::json!({"package": "a"}),
                        }],
                        after: vec![PlanStep {
                            kind: "dependency".into(),
                            detail: serde_json::json!({"package": "a"}),
                        }],
                    }],
                }),
                plan_hash: Some("cafe".into()),
            })),
        ),
        (
            "result",
            block(BlockKind::Result(ResultPayload {
                output: serde_json::json!({"nim_sum": 1}),
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
            })),
        ),
        (
            "outcome",
            block(BlockKind::Outcome(Outcome {
                status: OutcomeStatus::Blocked,
                kind: "precondition_failed".into(),
                message: "policy forbids".into(),
                details: Some(serde_json::json!({"policy": "no-network"})),
                exit_code: Some(4),
            })),
        ),
    ]
}

fn main() {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/golden/blocks");
    std::fs::create_dir_all(&dir).unwrap();
    let mut hashes = serde_json::Map::new();
    for (kind, block) in goldens() {
        let mut bytes = serde_json::to_vec_pretty(&block).unwrap();
        bytes.push(b'\n');
        std::fs::write(dir.join(format!("{kind}.json")), bytes).unwrap();
        hashes.insert(kind.to_owned(), block.content_hash().into());
    }
    let mut bytes = serde_json::to_vec_pretty(&serde_json::Value::Object(hashes)).unwrap();
    bytes.push(b'\n');
    std::fs::write(dir.join("hashes.json"), bytes).unwrap();
    println!("wrote 14 golden blocks + hashes.json to {}", dir.display());
}
