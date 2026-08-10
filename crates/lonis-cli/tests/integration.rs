// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! End-to-end CLI tests: exercise the amari split (stdout blocks / stderr
//! error) through the real `lonis` binary.

use assert_cmd::Command;
use lonis_schema::SeedBlock;

#[test]
fn call_echo_emits_block_array_on_stdout() {
    let output = Command::cargo_bin("lonis")
        .unwrap()
        .args([
            "call",
            "lonis:builtin:echo",
            "{\"hello\":\"world\"}",
            "--mode",
            "json",
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let blocks: Vec<SeedBlock> = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(blocks.len(), 1);
    assert_eq!(blocks[0].payload().kind_name(), "result");
    assert_eq!(
        blocks[0].attribution.provenance.producer,
        "lonis:builtin:echo"
    );
    let wire = serde_json::to_value(&blocks[0]).unwrap();
    assert_eq!(
        wire["payload"]["data"]["output"],
        serde_json::json!({"hello": "world"})
    );
    // Replay provenance: the echo tool hashes its input.
    assert!(wire["provenance"]["input_hash"].is_string());
    assert_eq!(wire["schema_version"], "lonis.block/v1");
}

#[test]
fn call_echo_ndjson_emits_one_block_per_line() {
    let output = Command::cargo_bin("lonis")
        .unwrap()
        .args(["call", "lonis:builtin:echo", "[1,2]", "--mode", "ndjson"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines.len(), 1);
    let block: SeedBlock = serde_json::from_str(lines[0]).unwrap();
    assert_eq!(block.payload().kind_name(), "result");
}

#[test]
fn call_echo_accepts_at_file_input() {
    let mut tmp = std::env::temp_dir();
    tmp.push("lonis-cli-test-input.json");
    std::fs::write(&tmp, "{\"from\":\"file\"}").unwrap();
    let arg = format!("@{}", tmp.display());
    let output = Command::cargo_bin("lonis")
        .unwrap()
        .args(["call", "lonis:builtin:echo", &arg, "--mode", "json"])
        .output()
        .unwrap();
    std::fs::remove_file(&tmp).ok();
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let blocks: Vec<SeedBlock> = serde_json::from_slice(&output.stdout).unwrap();
    let wire = serde_json::to_value(&blocks[0]).unwrap();
    assert_eq!(
        wire["payload"]["data"]["output"],
        serde_json::json!({"from": "file"})
    );
}

#[test]
fn unknown_tool_exits_three_with_structured_stderr() {
    let output = Command::cargo_bin("lonis")
        .unwrap()
        .args(["call", "lonis:nope:x", "--mode", "json"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(3));
    // stdout stays clean; the structured error is on stderr (amari split).
    assert!(output.stdout.is_empty());
    let err: serde_json::Value = serde_json::from_slice(&output.stderr).unwrap();
    assert_eq!(err["kind"], "not_found");
}

#[test]
fn bad_input_exits_two() {
    let output = Command::cargo_bin("lonis")
        .unwrap()
        .args(["call", "lonis:builtin:echo", "--mode", "json"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
}

#[test]
fn tools_list_shows_builtins() {
    let output = Command::cargo_bin("lonis")
        .unwrap()
        .args(["tools", "list"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("lonis:builtin:echo"));
    assert!(stdout.contains("lonis:builtin:version"));
}

#[test]
fn describe_echo_emits_contract() {
    let output = Command::cargo_bin("lonis")
        .unwrap()
        .args(["tools", "describe", "lonis:builtin:echo"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let contract: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(contract["name"], "lonis:builtin:echo");
    assert_eq!(contract["determinism"], "deterministic");
}
