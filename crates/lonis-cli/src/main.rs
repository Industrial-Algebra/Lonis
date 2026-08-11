// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! The `lonis` command-line harness.

#![forbid(unsafe_code)]

use std::io::Read;
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use lonis_core::run_tool;
use lonis_schema::{exit_code, OutputMode};

mod builtins;

#[derive(Parser)]
#[command(
    name = "lonis",
    version,
    about = "Lonis — an AI-native tool harness for the Anima ecosystem"
)]
struct Cli {
    /// Output mode for results.
    #[arg(long, value_enum, global = true, default_value_t = ModeArg::Human)]
    mode: ModeArg,
    #[command(subcommand)]
    command: Command,
}

/// CLI mirror of [`OutputMode`]; lives here so `lonis-schema` stays clap-free.
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
#[value(rename_all = "kebab-case")]
enum ModeArg {
    Human,
    Json,
    Ndjson,
}

impl From<ModeArg> for OutputMode {
    fn from(mode: ModeArg) -> Self {
        match mode {
            ModeArg::Human => OutputMode::Human,
            ModeArg::Json => OutputMode::Json,
            ModeArg::Ndjson => OutputMode::Ndjson,
        }
    }
}

#[derive(Subcommand)]
enum Command {
    /// Emit the provider manifest (ADR-0006: lonis is a conforming provider).
    Manifest,
    /// Inspect registered tools.
    Tools {
        #[command(subcommand)]
        action: ToolsAction,
    },
    /// Emit a versioned block-contract JSON Schema (`lonis schema` lists all).
    Schema {
        /// Schema family: `block` (the envelope) or one of the 14 seed kinds.
        kind: Option<String>,
    },
    /// Invoke a tool by id. INPUT is a JSON value, or `@<path>` to read JSON
    /// from a file; if omitted, JSON is read from stdin.
    Call {
        /// Tool id (`<tool>:<namespace>:<item>`).
        id: String,
        /// JSON input value, or `@<path>` to a JSON file.
        input: Option<String>,
    },
}

#[derive(Subcommand)]
enum ToolsAction {
    /// List registered tool ids and versions.
    List,
    /// Describe a tool's contract and capabilities.
    Describe {
        /// Tool id.
        id: String,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let mode: OutputMode = cli.mode.into();
    let registry = builtins::registry();
    match cli.command {
        Command::Manifest => {
            let tools: Vec<String> = registry.iter().map(|(id, _)| id.to_owned()).collect();
            let manifest = serde_json::json!({
                "name": "lonis",
                "version": env!("CARGO_PKG_VERSION"),
                "description": "Lonis — an AI-native tool harness for the Anima ecosystem",
                "provider_type": "external-executable",
                "protocol_version": "lonis.provider/v1",
                "tools": tools,
                "capabilities": ["diagnostics"]
            });
            println!("{manifest}");
            ExitCode::SUCCESS
        }
        Command::Tools { action } => match action {
            ToolsAction::List => match mode {
                OutputMode::Human => {
                    for (id, tool) in registry.iter() {
                        println!("{id}\t{}", tool.tool_version());
                    }
                    ExitCode::SUCCESS
                }
                OutputMode::Json | OutputMode::Ndjson => {
                    let tools: Vec<serde_json::Value> = registry
                        .iter()
                        .map(|(id, tool)| {
                            serde_json::json!({
                                "name": id,
                                "version": tool.tool_version(),
                                "description": tool.contract().map(|c| c.description),
                            })
                        })
                        .collect();
                    println!(
                        "{}",
                        serde_json::json!({"provider": "lonis", "tools": tools})
                    );
                    ExitCode::SUCCESS
                }
            },
            ToolsAction::Describe { id } => match registry.get(&id) {
                Some(tool) => match tool.contract() {
                    Some(contract) => match serde_json::to_string_pretty(&contract) {
                        Ok(json) => {
                            println!("{json}");
                            ExitCode::SUCCESS
                        }
                        Err(err) => {
                            eprintln!("generic: {err}");
                            ExitCode::from(exit_code::GENERIC)
                        }
                    },
                    None => {
                        println!("tool `{id}` declares no contract");
                        ExitCode::SUCCESS
                    }
                },
                None => {
                    eprintln!("not_found: no tool with id `{id}`");
                    ExitCode::from(exit_code::NOT_FOUND)
                }
            },
        },
        Command::Schema { kind } => match kind {
            None => match lonis_schema::block::schemas::block_schema_catalog() {
                Ok(catalog) => {
                    for summary in &catalog.schemas {
                        println!("{}\t{}", summary.kind.as_str(), summary.id);
                    }
                    ExitCode::SUCCESS
                }
                Err(err) => {
                    eprintln!("generic: {err}");
                    ExitCode::from(exit_code::GENERIC)
                }
            },
            Some(kind) => match kind.parse() {
                Ok(kind) => {
                    match lonis_schema::block::schemas::block_schema(kind).and_then(|s| {
                        s.canonical_json()
                            .map_err(|e| lonis_schema::block::schemas::SchemaError(e.to_string()))
                    }) {
                        Ok(bytes) => {
                            print!("{}", String::from_utf8_lossy(&bytes));
                            ExitCode::SUCCESS
                        }
                        Err(err) => {
                            eprintln!("generic: {err}");
                            ExitCode::from(exit_code::GENERIC)
                        }
                    }
                }
                Err(_) => {
                    eprintln!("not_found: no block schema kind `{kind}`");
                    ExitCode::from(exit_code::NOT_FOUND)
                }
            },
        },
        Command::Call { id, input } => {
            let value = match parse_input(input.as_deref()) {
                Ok(value) => value,
                Err(err) => {
                    eprintln!("invalid_input: {err}");
                    return ExitCode::from(exit_code::INVALID_INPUT);
                }
            };
            ExitCode::from(run_tool(&registry, &id, value, mode))
        }
    }
}

/// Parse JSON input from an explicit argument, an `@<path>` file reference,
/// or stdin (empty stdin → null).
fn parse_input(input: Option<&str>) -> Result<serde_json::Value, String> {
    match input {
        Some(text) if text.starts_with('@') => {
            let path = &text[1..];
            let contents = std::fs::read_to_string(path)
                .map_err(|err| format!("cannot read input file `{path}`: {err}"))?;
            serde_json::from_str(contents.trim()).map_err(|err| err.to_string())
        }
        Some(text) => serde_json::from_str(text).map_err(|err| err.to_string()),
        None => {
            let mut buf = String::new();
            let _ = std::io::stdin().read_to_string(&mut buf);
            let trimmed = buf.trim();
            if trimmed.is_empty() {
                Ok(serde_json::Value::Null)
            } else {
                serde_json::from_str(trimmed).map_err(|err| err.to_string())
            }
        }
    }
}
