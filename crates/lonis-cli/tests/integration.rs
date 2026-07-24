// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! End-to-end CLI tests: exercise the amari split (stdout envelope / stderr
//! error) through the real `lonis` binary.

use assert_cmd::Command;
use lonis_schema::Envelope;

#[test]
fn call_echo_emits_envelope_on_stdout() {
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
    let env: Envelope<serde_json::Value> = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(env.tool.as_str(), "lonis:builtin:echo");
    assert_eq!(env.result, serde_json::json!({"hello": "world"}));
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
