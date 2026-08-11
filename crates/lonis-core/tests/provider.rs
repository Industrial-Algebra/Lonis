// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! Integration tests for `SubprocessProvider`: discovering and invoking
//! tools hosted by one external executable (ADR-0006), through a real
//! process boundary against the `mock_tool` bin's provider subcommands.

use std::path::PathBuf;

use lonis_core::{SubprocessProvider, Tool};
use lonis_schema::Capabilities;

/// The mock binary: a `[[bin]]` target, so Cargo resolves it robustly in
/// both workspace and scoped (`-p lonis-core`) test runs.
fn mock_tool() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_mock_tool"))
}

fn provider() -> SubprocessProvider {
    SubprocessProvider::new(mock_tool())
}

#[test]
fn manifest_parses_provider_metadata() {
    let manifest = provider().manifest().unwrap();
    assert_eq!(manifest.name, "mock");
    assert_eq!(manifest.provider_type, "external-executable");
    assert!(manifest.tools.contains(&"mock.echo".to_owned()));
    assert!(manifest.tools.contains(&"mock.fail".to_owned()));
}

#[test]
fn tools_list_parses_summaries() {
    let tools = provider().tools().unwrap();
    assert_eq!(tools.len(), 2);
    let echo = tools.iter().find(|t| t.name == "mock.echo").unwrap();
    assert_eq!(echo.description.as_deref(), Some("echo the input"));
}

#[test]
fn describe_returns_a_tool_contract() {
    let contract = provider().describe("mock.echo").unwrap();
    assert_eq!(contract.name.as_str(), "mock:echo");
    assert_eq!(contract.description, "echo the input");
}

#[test]
fn provider_tool_invokes_through_call_prefix() {
    let tool = provider().tool("mock.echo");
    let blocks = tool.invoke(serde_json::json!({"via": "provider"})).unwrap();
    assert_eq!(blocks.len(), 1);
    let wire = serde_json::to_value(&blocks[0]).unwrap();
    assert_eq!(wire["payload"]["data"]["output"]["echo"]["via"], "provider");
    // The hosted tool is registered under a mangled namespaced id.
    assert_eq!(tool.tool_id().as_str(), "mock:echo");
}

#[test]
fn provider_tool_propagates_structured_errors() {
    let tool = provider().tool("mock.fail");
    let err = tool.invoke(serde_json::Value::Null).unwrap_err();
    assert_eq!(err.kind, "mock_failure");
    assert_eq!(err.exit_code, 3);
}

#[test]
fn missing_provider_reports_unavailable() {
    let provider = SubprocessProvider::new("/definitely/not/a/provider");
    assert!(!provider.availability().is_ready());
    let err = provider.manifest().unwrap_err();
    assert_eq!(err.kind, "unavailable");
}

#[test]
fn non_conforming_binary_maps_to_invalid_output() {
    // `--blocks` mode emits a block array, not a manifest.
    let provider = SubprocessProvider::new(mock_tool()).with_manifest_args(vec!["--text".into()]);
    let err = provider.manifest().unwrap_err();
    assert_eq!(err.kind, "invalid_output");
}
