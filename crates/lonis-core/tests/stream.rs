// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! Integration tests for stream mode (ADR-0009): incremental block delivery
//! through the subprocess seam, with backpressure, bounds, and error tails.

use std::path::PathBuf;
use std::time::{Duration, Instant};

use lonis_core::{SubprocessTool, Tool};
use lonis_schema::block::kinds::BlockKind;

fn mock_tool() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_mock_tool"))
}

#[test]
fn stream_yields_blocks_incrementally() {
    let tool = SubprocessTool::new(
        lonis_schema::ToolId::new("lonis:test:mock").unwrap(),
        mock_tool(),
    )
    .with_args(vec!["--stream".into(), "3".into()]);
    let start = Instant::now();
    let stream = tool
        .invoke_stream(serde_json::Value::Null)
        .expect("stream starts");
    // The first block arrives well before the child finishes (the mock
    // sleeps between lines).
    let mut blocks = Vec::new();
    for item in stream {
        blocks.push(item.expect("block"));
        if blocks.len() == 1 {
            assert!(
                start.elapsed() < Duration::from_secs(2),
                "first block should arrive incrementally, not at child exit"
            );
        }
    }
    assert_eq!(blocks.len(), 3);
    assert!(blocks.iter().all(|b| b.payload().kind_name() == "result"));
}

#[test]
fn stream_error_tail_after_partial_output() {
    let tool = SubprocessTool::new(
        lonis_schema::ToolId::new("lonis:test:mock").unwrap(),
        mock_tool(),
    )
    .with_args(vec!["--stream-fail".into()]);
    let items: Vec<_> = tool
        .invoke_stream(serde_json::Value::Null)
        .expect("stream starts")
        .collect();
    assert_eq!(items.len(), 2);
    assert!(items[0].is_ok());
    let err = items[1].as_ref().unwrap_err();
    assert_eq!(err.kind, "mock_failure");
    assert_eq!(err.exit_code, 3);
}

#[test]
fn stream_enforces_timeout() {
    let tool = SubprocessTool::new(
        lonis_schema::ToolId::new("lonis:test:mock").unwrap(),
        mock_tool(),
    )
    .with_args(vec!["--stream-sleep".into(), "30000".into()])
    .with_timeout_millis(200);
    let items: Vec<_> = tool
        .invoke_stream(serde_json::Value::Null)
        .expect("stream starts")
        .collect();
    let err = items.last().expect("an error item").as_ref().unwrap_err();
    assert_eq!(err.kind, "timeout");
}

#[test]
fn dropping_the_stream_kills_the_child() {
    let tool = SubprocessTool::new(
        lonis_schema::ToolId::new("lonis:test:mock").unwrap(),
        mock_tool(),
    )
    .with_args(vec!["--stream-sleep".into(), "30000".into()]);
    let stream = tool
        .invoke_stream(serde_json::Value::Null)
        .expect("stream starts");
    let start = Instant::now();
    drop(stream);
    // Drop must not wait for the child to finish sleeping.
    assert!(start.elapsed() < Duration::from_secs(5));
}

#[test]
fn collect_then_stream_default_via_custom_tool() {
    // A Tool without an invoke_stream override streams its Vec<Block>.
    struct OneShot;
    impl lonis_schema::Capabilities for OneShot {
        fn schema_version(&self) -> lonis_schema::SchemaVersion {
            lonis_schema::SchemaVersion::default()
        }
        fn tool_version(&self) -> &str {
            "0.0.1"
        }
        fn output_formats(&self) -> &'static [lonis_schema::OutputMode] {
            &[lonis_schema::OutputMode::Ndjson]
        }
        fn exit_code_map(&self) -> &'static [(&'static str, u8)] {
            &[("ok", 0)]
        }
        fn tool_id(&self) -> lonis_schema::ToolId {
            lonis_schema::ToolId::new("lonis:test:oneshot").unwrap()
        }
    }
    impl Tool<BlockKind> for OneShot {
        fn invoke(
            &self,
            _input: serde_json::Value,
        ) -> Result<Vec<lonis_schema::Block<BlockKind>>, lonis_schema::ToolError> {
            Ok(vec![
                lonis_schema::Block::new(
                    lonis_schema::Attribution::new("lonis:test:oneshot", "lonis:test:oneshot"),
                    BlockKind::Extension {
                        kind: "test.one".into(),
                        data: serde_json::json!(1),
                    },
                ),
                lonis_schema::Block::new(
                    lonis_schema::Attribution::new("lonis:test:oneshot", "lonis:test:oneshot"),
                    BlockKind::Extension {
                        kind: "test.two".into(),
                        data: serde_json::json!(2),
                    },
                ),
            ])
        }
    }
    let stream = OneShot
        .invoke_stream(serde_json::Value::Null)
        .expect("stream");
    let items: Vec<_> = stream.collect::<Result<Vec<_>, _>>().expect("all ok");
    assert_eq!(items.len(), 2);
    assert_eq!(items[0].payload().kind_name(), "test.one");
    assert_eq!(items[1].payload().kind_name(), "test.two");
}

#[cfg(feature = "futures")]
#[test]
fn futures_adapter_collects_the_same_blocks() {
    use futures::StreamExt as _;
    let tool = SubprocessTool::new(
        lonis_schema::ToolId::new("lonis:test:mock").unwrap(),
        mock_tool(),
    )
    .with_args(vec!["--stream".into(), "2".into()]);
    let stream = tool.invoke_stream(serde_json::Value::Null).unwrap();
    let items: Vec<_> = futures::executor::block_on(stream.into_async().collect::<Vec<_>>());
    assert_eq!(items.len(), 2);
    assert!(items.iter().all(Result::is_ok));
}
