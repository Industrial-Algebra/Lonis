// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! Facade composition tests: a consumer depending only on the `lonis`
//! umbrella crate can define, register, invoke, and render a tool.

use lonis::block::kinds::{BlockKind, ResultPayload};
use lonis::{
    exit_code, Attribution, Block, Capabilities, OutputMode, SchemaVersion, Tool, ToolError,
    ToolId, ToolRegistry,
};

struct Echo;

const FMTS: [OutputMode; 3] = [OutputMode::Human, OutputMode::Json, OutputMode::Ndjson];
const MAP: &[(&str, u8)] = &[("ok", exit_code::SUCCESS)];

impl Capabilities for Echo {
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
        ToolId::new("lonis:facade:echo").unwrap()
    }
}

impl Tool<BlockKind> for Echo {
    fn invoke(&self, input: serde_json::Value) -> Result<Vec<Block<BlockKind>>, ToolError> {
        let id = self.tool_id();
        Ok(vec![Block::new(
            Attribution::new(id.as_str(), id.as_str()),
            BlockKind::Result(ResultPayload {
                output: input,
                score: None,
                evidence: Vec::new(),
                validated_assumptions: Vec::new(),
                refuted_assumptions: Vec::new(),
                resources: None,
                duration_micros: None,
            }),
        )])
    }
}

#[test]
fn facade_composes_schema_and_core() {
    let mut registry = ToolRegistry::new();
    registry.register(Box::new(Echo)).unwrap();
    let blocks = registry
        .invoke("lonis:facade:echo", serde_json::json!({"via": "facade"}))
        .unwrap();
    assert_eq!(blocks.len(), 1);
    assert_eq!(blocks[0].payload().kind_name(), "result");

    let mut buf = Vec::new();
    lonis::render(&blocks, OutputMode::Json, &mut buf).unwrap();
    let parsed: Vec<lonis::SeedBlock> = serde_json::from_slice(&buf).unwrap();
    assert_eq!(parsed, blocks);
}

#[test]
fn facade_exposes_subprocess_adapter() {
    let tool = lonis::SubprocessTool::new(
        ToolId::new("lonis:facade:cat").unwrap(),
        "/definitely/missing",
    );
    assert!(!tool.availability().is_ready());
}

#[cfg(feature = "derive")]
#[test]
fn facade_exposes_the_derive() {
    #[derive(lonis::LonisCapabilities)]
    #[lonis(tool_id = "lonis:facade:derived")]
    struct Derived;

    let caps = Derived;
    assert_eq!(caps.tool_id().as_str(), "lonis:facade:derived");
    assert_eq!(caps.schema_version(), SchemaVersion::default());
}
