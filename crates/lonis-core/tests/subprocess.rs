// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! Integration tests for `SubprocessTool`: real process-spawn boundary,
//! exercising the ADR-0003 wire protocol against the `mock_tool` bin.

use std::path::PathBuf;

use lonis_core::{StdoutMapping, SubprocessTool, Tool};
use lonis_schema::{exit_code, SeedBlock, ToolId};

/// The mock binary: a `[[bin]]` target, so Cargo resolves it robustly in
/// both workspace and scoped (`-p lonis-core`) test runs.
fn mock_tool() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_mock_tool"))
}

fn mock(args: &[&str]) -> SubprocessTool {
    SubprocessTool::new(ToolId::new("lonis:test:mock").unwrap(), mock_tool())
        .with_args(args.iter().map(|a| (*a).to_owned()).collect())
}

#[test]
fn blocks_mode_returns_typed_blocks() {
    let tool = mock(&[]);
    let blocks = tool.invoke(serde_json::json!({"hi": 1})).unwrap();
    assert_eq!(blocks.len(), 1);
    assert_eq!(blocks[0].payload().kind_name(), "result");
    let wire = serde_json::to_value(&blocks[0]).unwrap();
    assert_eq!(wire["payload"]["data"]["output"]["echo"]["hi"], 1);
    assert_eq!(wire["schema_version"], "lonis.block/v1");
}

#[test]
fn ndjson_mode_returns_one_block_per_line() {
    let tool = mock(&["--ndjson"]);
    let blocks = tool.invoke(serde_json::json!(42)).unwrap();
    assert_eq!(blocks.len(), 2);
    assert!(blocks.iter().all(|b| b.payload().kind_name() == "result"));
}

#[test]
fn text_mode_wraps_stdout_in_a_result_block() {
    let tool = mock(&["--text"]).with_mapping(StdoutMapping::Text);
    let blocks = tool.invoke(serde_json::Value::Null).unwrap();
    assert_eq!(blocks.len(), 1);
    let wire = serde_json::to_value(&blocks[0]).unwrap();
    assert_eq!(
        wire["payload"]["data"]["output"]["stdout"],
        "plain text output"
    );
    // The harness attributes the wrapped block to the tool itself.
    assert_eq!(
        wire["attribution"]["provenance"]["producer"],
        "lonis:test:mock"
    );
    // And pins the input hash for replay.
    assert!(wire["provenance"]["input_hash"].is_string());
}

#[test]
fn unknown_kind_from_subprocess_lands_in_extension() {
    let tool = mock(&["--unknown-kind"]);
    let blocks: Vec<SeedBlock> = tool.invoke(serde_json::Value::Null).unwrap();
    assert_eq!(blocks.len(), 1);
    assert_eq!(blocks[0].payload().kind_name(), "widget");
    let wire = serde_json::to_value(&blocks[0]).unwrap();
    assert_eq!(wire["payload"]["data"]["gadget"], 1);
}

#[test]
fn structured_stderr_propagates_tool_error() {
    let tool = mock(&["--fail"]);
    let err = tool.invoke(serde_json::Value::Null).unwrap_err();
    assert_eq!(err.kind, "mock_failure");
    assert_eq!(err.exit_code, 3);
    assert_eq!(err.details.unwrap()["deliberate"], true);
}

#[test]
fn unstructured_stderr_maps_to_tool_failed() {
    let tool = mock(&["--fail-plain"]);
    let err = tool.invoke(serde_json::Value::Null).unwrap_err();
    assert_eq!(err.kind, "tool_failed");
    assert_eq!(err.message, "boom");
    assert_eq!(err.exit_code, 2);
}

#[test]
fn timeout_kills_the_child_and_reports_limit_exceeded() {
    let tool = mock(&["--sleep", "30000"]).with_timeout_millis(200);
    let start = std::time::Instant::now();
    let err = tool.invoke(serde_json::Value::Null).unwrap_err();
    assert_eq!(err.kind, "timeout");
    assert_eq!(err.exit_code, exit_code::LIMIT_EXCEEDED);
    assert!(start.elapsed() < std::time::Duration::from_secs(10));
}

#[test]
fn output_cap_kills_the_child_and_reports_limit_exceeded() {
    let tool = mock(&["--big", "1048576"]).with_max_stdout_bytes(1024);
    let err = tool.invoke(serde_json::Value::Null).unwrap_err();
    assert_eq!(err.kind, "output_limit_exceeded");
    assert_eq!(err.exit_code, exit_code::LIMIT_EXCEEDED);
}

#[test]
fn missing_binary_reports_unavailable() {
    let tool = SubprocessTool::new(
        ToolId::new("lonis:test:missing").unwrap(),
        "/definitely/not/a/real/binary",
    );
    assert!(!tool.availability().is_ready());
    let err = tool.invoke(serde_json::Value::Null).unwrap_err();
    assert_eq!(err.kind, "unavailable");
}

#[test]
fn invalid_stdout_maps_to_serialization_error() {
    let tool = mock(&["--big", "64"]);
    let err = tool.invoke(serde_json::Value::Null).unwrap_err();
    assert_eq!(err.kind, "invalid_output");
    assert_eq!(err.exit_code, exit_code::SERIALIZATION);
}
