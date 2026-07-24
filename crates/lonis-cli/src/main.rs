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
    /// Inspect registered tools.
    Tools {
        #[command(subcommand)]
        action: ToolsAction,
    },
    /// Invoke a tool by id. INPUT is a JSON value; if omitted, JSON is read from stdin.
    Call {
        /// Tool id (`<tool>:<namespace>:<item>`).
        id: String,
        /// JSON input value.
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
        Command::Tools { action } => match action {
            ToolsAction::List => {
                for (id, tool) in registry.iter() {
                    println!("{id}\t{}", tool.tool_version());
                }
                ExitCode::SUCCESS
            }
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

/// Parse JSON input from an explicit argument or stdin (empty stdin → null).
fn parse_input(input: Option<&str>) -> Result<serde_json::Value, String> {
    match input {
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
