// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! Built-in tools shipped with the `lonis` CLI: `lonis:builtin:echo` and
//! `lonis:builtin:version`.

use lonis_core::{Tool, ToolRegistry};
use lonis_schema::{
    exit_code, Capabilities, Cost, Determinism, Envelope, OutputMode, SchemaRef, SchemaVersion,
    SideEffects, ToolContract, ToolError, ToolId,
};

const FMTS: [OutputMode; 3] = [OutputMode::Human, OutputMode::Json, OutputMode::Ndjson];
const MAP: &[(&str, u8)] = &[
    ("ok", exit_code::SUCCESS),
    ("bad_input", exit_code::INVALID_INPUT),
];

/// Build a registry preloaded with the built-in tools.
pub fn registry() -> ToolRegistry {
    let mut registry = ToolRegistry::new();
    registry.register(Box::new(Echo)).expect("register echo");
    registry
        .register(Box::new(Version))
        .expect("register version");
    registry
}

struct Echo;

impl Capabilities for Echo {
    fn schema_version(&self) -> SchemaVersion {
        SchemaVersion::default()
    }
    fn tool_version(&self) -> &str {
        env!("CARGO_PKG_VERSION")
    }
    fn output_formats(&self) -> &'static [OutputMode] {
        &FMTS
    }
    fn exit_code_map(&self) -> &'static [(&'static str, u8)] {
        MAP
    }
    fn tool_id(&self) -> ToolId {
        ToolId::new("lonis:builtin:echo").unwrap()
    }
}

impl Tool for Echo {
    fn invoke(&self, input: serde_json::Value) -> Result<Envelope<serde_json::Value>, ToolError> {
        if input.is_null() {
            return Err(ToolError::new(
                "bad_input",
                "echo requires non-null input",
                exit_code::INVALID_INPUT,
            ));
        }
        Ok(Envelope::new(self.tool_id(), input))
    }

    fn contract(&self) -> Option<ToolContract> {
        Some(ToolContract {
            name: self.tool_id(),
            description: "Echo the input JSON value back as the result.".into(),
            input_schema: SchemaRef("lonis.builtin/v1#EchoInput".into()),
            output_schema: SchemaRef("lonis.builtin/v1#EchoOutput".into()),
            determinism: Determinism::Deterministic,
            side_effects: SideEffects::None,
            cost: Cost::Low,
            capabilities: Vec::new(),
        })
    }
}

struct Version;

impl Capabilities for Version {
    fn schema_version(&self) -> SchemaVersion {
        SchemaVersion::default()
    }
    fn tool_version(&self) -> &str {
        env!("CARGO_PKG_VERSION")
    }
    fn output_formats(&self) -> &'static [OutputMode] {
        &FMTS
    }
    fn exit_code_map(&self) -> &'static [(&'static str, u8)] {
        MAP
    }
    fn tool_id(&self) -> ToolId {
        ToolId::new("lonis:builtin:version").unwrap()
    }
}

impl Tool for Version {
    fn invoke(&self, _input: serde_json::Value) -> Result<Envelope<serde_json::Value>, ToolError> {
        Ok(Envelope::new(
            self.tool_id(),
            serde_json::json!({
                "lonis": env!("CARGO_PKG_VERSION"),
                "schema": SchemaVersion::default().as_str(),
            }),
        ))
    }

    fn contract(&self) -> Option<ToolContract> {
        Some(ToolContract {
            name: self.tool_id(),
            description: "Report the lonis and schema versions.".into(),
            input_schema: SchemaRef("lonis.builtin/v1#VersionInput".into()),
            output_schema: SchemaRef("lonis.builtin/v1#VersionOutput".into()),
            determinism: Determinism::Deterministic,
            side_effects: SideEffects::ReadOnly,
            cost: Cost::Low,
            capabilities: Vec::new(),
        })
    }
}
