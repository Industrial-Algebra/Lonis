// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! A mock Lonis-native composable CLI, used by `tests/subprocess.rs` to
//! exercise `SubprocessTool` through a real process boundary.
//!
//! Protocol (ADR-0003): JSON input on stdin, blocks on stdout, structured
//! `ToolError` on stderr, narrow exit codes.

use std::io::Read as _;

use lonis_schema::block::kinds::{BlockKind, ResultPayload};
use lonis_schema::{Attribution, Block, SeedBlock, ToolError};

fn result_block(output: serde_json::Value) -> SeedBlock {
    Block::new(
        Attribution::new("lonis:test:mock", "lonis:test:mock"),
        BlockKind::Result(ResultPayload {
            output,
            score: None,
            evidence: Vec::new(),
            validated_assumptions: Vec::new(),
            refuted_assumptions: Vec::new(),
            resources: None,
            duration_micros: None,
        }),
    )
}

fn main() {
    let raw: Vec<String> = std::env::args().skip(1).collect();
    // Strip the global `--mode <x>` pair (providers pass `--mode json`).
    let mut args: Vec<String> = Vec::new();
    let mut skip = false;
    for (i, arg) in raw.iter().enumerate() {
        if skip {
            skip = false;
            continue;
        }
        if arg == "--mode" {
            skip = true;
            continue;
        }
        let _ = i;
        args.push(arg.clone());
    }

    // Provider subcommand surface (ADR-0006): manifest / tools / call.
    if let Some(first) = args.first().map(String::as_str) {
        match first {
            "manifest" => {
                println!(
                    "{}",
                    serde_json::json!({
                        "name": "mock",
                        "version": "0.0.1",
                        "description": "mock Lonis provider for tests",
                        "provider_type": "external-executable",
                        "protocol_version": "lonis.provider/v1",
                        "tools": ["mock.echo", "mock.fail"]
                    })
                );
                return;
            }
            "tools" => match args.get(1).map(String::as_str) {
                Some("list") => {
                    println!(
                        "{}",
                        serde_json::json!({
                            "provider": "mock",
                            "tools": [
                                {"name": "mock.echo", "description": "echo the input"},
                                {"name": "mock.fail", "description": "fail deliberately"}
                            ]
                        })
                    );
                    return;
                }
                Some("describe") => {
                    let name = args.get(2).map(String::as_str).unwrap_or("");
                    match name {
                        "mock.echo" => {
                            println!(
                                "{}",
                                serde_json::json!({
                                    "name": "mock:echo",
                                    "description": "echo the input",
                                    "input_schema": "mock.echo/input/v1",
                                    "output_schema": "mock.echo/output/v1",
                                    "determinism": "deterministic",
                                    "side_effects": "none",
                                    "cost": "low"
                                })
                            );
                            return;
                        }
                        other => {
                            let err = ToolError::new("not_found", format!("no tool `{other}`"), 3);
                            eprintln!("{}", serde_json::to_string(&err).unwrap());
                            std::process::exit(3);
                        }
                    }
                }
                _ => {}
            },
            "call" => {
                let name = args.get(1).map(String::as_str).unwrap_or("");
                match name {
                    "mock.echo" => return run("--blocks", &args),
                    "mock.fail" => return run("--fail", &args),
                    other => {
                        let err = ToolError::new("not_found", format!("no tool `{other}`"), 3);
                        eprintln!("{}", serde_json::to_string(&err).unwrap());
                        std::process::exit(3);
                    }
                }
            }
            _ => {}
        }
    }

    run(
        args.first().map(String::as_str).unwrap_or("--blocks"),
        &args,
    );
}

fn run(mode: &str, args: &[String]) {
    let mut input = String::new();
    let _ = std::io::stdin().read_to_string(&mut input);
    let input: serde_json::Value =
        serde_json::from_str(input.trim()).unwrap_or(serde_json::Value::Null);

    match mode {
        "--blocks" => {
            let blocks = vec![result_block(serde_json::json!({"echo": input}))];
            println!("{}", serde_json::to_string(&blocks).unwrap());
        }
        "--ndjson" => {
            let first = result_block(serde_json::json!({"echo": input}));
            let second = result_block(serde_json::json!({"again": true}));
            println!("{}", serde_json::to_string(&first).unwrap());
            println!("{}", serde_json::to_string(&second).unwrap());
        }
        "--text" => {
            println!("plain text output");
        }
        "--unknown-kind" => {
            println!(
                "{}",
                serde_json::json!([{
                    "schema_version": "lonis.block/v1",
                    "attribution": {
                        "identity": "lonis:test:mock",
                        "provenance": {
                            "when": "2026-08-10T00:00:00Z",
                            "producer": "lonis:test:mock"
                        }
                    },
                    "payload": {"kind": "widget", "data": {"gadget": 1}}
                }])
            );
        }
        "--sleep" => {
            let millis: u64 = args.get(1).map_or(5_000, |v| v.parse().unwrap());
            std::thread::sleep(std::time::Duration::from_millis(millis));
            let blocks = vec![result_block(serde_json::json!({"slept": millis}))];
            println!("{}", serde_json::to_string(&blocks).unwrap());
        }
        "--big" => {
            let bytes: usize = args.get(1).map_or(1_048_576, |v| v.parse().unwrap());
            let chunk = "x".repeat(bytes);
            println!("{chunk}");
        }
        "--fail" => {
            let err = ToolError::new("mock_failure", "the mock failed deliberately", 3)
                .with_details(serde_json::json!({"deliberate": true}));
            eprintln!("{}", serde_json::to_string(&err).unwrap());
            std::process::exit(3);
        }
        "--fail-plain" => {
            eprintln!("boom");
            std::process::exit(2);
        }
        other => {
            eprintln!("unknown mock mode `{other}`");
            std::process::exit(1);
        }
    }
}
