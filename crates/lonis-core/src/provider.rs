// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! `SubprocessProvider` — one external executable hosting many tools
//! (ADR-0006).
//!
//! Where [`SubprocessTool`] binds one argv prefix to one tool, a provider
//! discovers a whole tool surface through the v0 provider commands:
//!
//! - `<cmd> manifest` → [`ProviderManifest`] (provider metadata + tool names),
//! - `<cmd> tools list` → [`ProviderToolSummary`] summaries,
//! - `<cmd> tools describe <name>` → a [`ToolContract`],
//! - `<cmd> call <name>` → invocation, per the ADR-0003 wire protocol.
//!
//! Discovery and invocation run through the same bounded, isolated execution
//! core as [`SubprocessTool`] (timeouts, byte caps, cleared environment,
//! neutral cwd). `lonis` itself is a conforming provider — the self-host
//! dogfood.

use std::path::PathBuf;

use serde::Deserialize;

use lonis_schema::{exit_code, ToolContract, ToolError, ToolId};

use crate::subprocess::{Availability, SubprocessTool};

/// Provider metadata emitted by `<cmd> manifest` (v0 spec §6).
///
/// Unknown fields are tolerated: manifests evolve across provider versions,
/// and a host should not reject a newer provider for carrying new metadata.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct ProviderManifest {
    /// Provider name (e.g. `lonis`, `figma`).
    pub name: String,
    /// Provider version string.
    pub version: String,
    /// What the provider does.
    pub description: String,
    /// Provider kind (`external-executable` for this adapter).
    pub provider_type: String,
    /// The provider protocol version the executable speaks.
    pub protocol_version: String,
    /// Tool names the provider hosts.
    pub tools: Vec<String>,
    /// Human display name, when provided.
    #[serde(default)]
    pub display_name: Option<String>,
    /// Declared provider capabilities, when provided.
    #[serde(default)]
    pub capabilities: Vec<String>,
}

/// The `<cmd> tools list` response (v0 spec §7).
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct ProviderToolList {
    /// The provider's name, when echoed.
    #[serde(default)]
    pub provider: Option<String>,
    /// Compact tool summaries.
    pub tools: Vec<ProviderToolSummary>,
}

/// A compact tool summary from `tools list` (full details live in
/// `tools describe`).
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct ProviderToolSummary {
    /// The tool's name within the provider (e.g. `mock.echo`,
    /// `lonis:builtin:echo`).
    pub name: String,
    /// Short purpose description, when provided.
    #[serde(default)]
    pub description: Option<String>,
}

/// A provider: one external executable hosting many tools behind the v0
/// subcommand surface.
///
/// Discovery commands default to the v0 surface with `--mode json` (the
/// shape `lonis` itself speaks); every arg prefix is overridable for
/// providers with different conventions.
#[derive(Debug)]
pub struct SubprocessProvider {
    command: PathBuf,
    manifest_args: Vec<String>,
    tools_list_args: Vec<String>,
    /// Bounded execution configuration shared with constructed tools.
    configure: SubprocessTool,
}

impl SubprocessProvider {
    /// Create a provider over an executable, with default v0 discovery args
    /// and default bounds (the amari probe bounds).
    #[must_use]
    pub fn new(command: impl Into<PathBuf>) -> Self {
        let command = command.into();
        Self {
            configure: SubprocessTool::new(
                ToolId::new("lonis:provider:discovery").expect("valid id"),
                command.clone(),
            ),
            command,
            manifest_args: vec!["--mode".into(), "json".into(), "manifest".into()],
            tools_list_args: vec![
                "--mode".into(),
                "json".into(),
                "tools".into(),
                "list".into(),
            ],
        }
    }

    /// Override the manifest discovery args (for non-conforming CLIs or
    /// testing the error path).
    #[must_use]
    pub fn with_manifest_args(mut self, args: Vec<String>) -> Self {
        self.manifest_args = args;
        self
    }

    /// Override the tools-list discovery args.
    #[must_use]
    pub fn with_tools_list_args(mut self, args: Vec<String>) -> Self {
        self.tools_list_args = args;
        self
    }

    /// Set the hard wall-clock timeout for discovery and (unless overridden
    /// on the tool) invocation, in milliseconds.
    #[must_use]
    pub fn with_timeout_millis(mut self, millis: u64) -> Self {
        self.configure = self.configure.with_timeout_millis(millis);
        self
    }

    /// Set the stdout byte cap for discovery and (unless overridden on the
    /// tool) invocation.
    #[must_use]
    pub fn with_max_stdout_bytes(mut self, bytes: u64) -> Self {
        self.configure = self.configure.with_max_stdout_bytes(bytes);
        self
    }

    /// Add environment variables on top of the cleared environment for
    /// discovery and (unless overridden on the tool) invocation.
    #[must_use]
    pub fn with_env(mut self, vars: Vec<(String, String)>) -> Self {
        self.configure = self.configure.with_env(vars);
        self
    }

    /// Set the working directory for discovery and invocation.
    #[must_use]
    pub fn with_cwd(mut self, cwd: impl Into<PathBuf>) -> Self {
        self.configure = self.configure.with_cwd(cwd);
        self
    }

    /// Probe whether the provider executable is invocable on this host.
    #[must_use]
    pub fn availability(&self) -> Availability {
        self.configure.availability()
    }

    fn discovery(&self, args: &[String]) -> Result<serde_json::Value, ToolError> {
        let outcome = self.configure.exec_bounded(args, None)?;
        if !outcome.status.success() {
            return Err(crate::subprocess::map_failure(
                &outcome.status,
                &outcome.stderr,
            ));
        }
        let text = String::from_utf8_lossy(&outcome.stdout).into_owned();
        serde_json::from_str(text.trim()).map_err(|err| {
            ToolError::new(
                "invalid_output",
                format!(
                    "provider `{}` did not emit the expected JSON: {err}",
                    self.command.display()
                ),
                exit_code::SERIALIZATION,
            )
        })
    }

    /// Fetch and parse the provider's manifest.
    ///
    /// # Errors
    /// [`ToolError`] kind `unavailable` (missing binary), `tool_failed`
    /// (nonzero exit), or `invalid_output` (non-conforming JSON).
    pub fn manifest(&self) -> Result<ProviderManifest, ToolError> {
        let args = self.manifest_args.clone();
        let value = self.discovery(&args)?;
        serde_json::from_value(value).map_err(|err| {
            ToolError::new(
                "invalid_output",
                format!("provider manifest does not match the v0 shape: {err}"),
                exit_code::SERIALIZATION,
            )
        })
    }

    /// Fetch the provider's tool summaries.
    ///
    /// # Errors
    /// As [`SubprocessProvider::manifest`].
    pub fn tools(&self) -> Result<Vec<ProviderToolSummary>, ToolError> {
        let args = self.tools_list_args.clone();
        let value = self.discovery(&args)?;
        let list: ProviderToolList = serde_json::from_value(value).map_err(|err| {
            ToolError::new(
                "invalid_output",
                format!("provider tools list does not match the v0 shape: {err}"),
                exit_code::SERIALIZATION,
            )
        })?;
        Ok(list.tools)
    }

    /// Fetch one tool's full contract via `tools describe <name>`.
    ///
    /// # Errors
    /// As [`SubprocessProvider::manifest`], plus the provider's own error
    /// for an unknown tool.
    pub fn describe(&self, name: &str) -> Result<ToolContract, ToolError> {
        let args = vec![
            "--mode".to_owned(),
            "json".to_owned(),
            "tools".to_owned(),
            "describe".to_owned(),
            name.to_owned(),
        ];
        let value = self.discovery(&args)?;
        serde_json::from_value(value).map_err(|err| {
            ToolError::new(
                "invalid_output",
                format!("provider tools describe is not a ToolContract: {err}"),
                exit_code::SERIALIZATION,
            )
        })
    }

    /// Construct a [`SubprocessTool`] invoking `name` through the provider's
    /// `call` subcommand, inheriting the provider's bounds.
    ///
    /// Provider tool names may be dotted (`mock.echo`) or already namespaced
    /// (`lonis:builtin:echo`); dots are mangled to colons for the [`ToolId`].
    #[must_use]
    pub fn tool(&self, name: &str) -> SubprocessTool {
        let id = mangle_tool_id(name);
        self.configure.clone_for(
            id,
            vec![
                "--mode".to_owned(),
                "json".to_owned(),
                "call".to_owned(),
                name.to_owned(),
            ],
        )
    }
}

/// Mangle a provider tool name into a valid [`ToolId`]: dots become colons
/// (`mock.echo` → `mock:echo`); names with fewer than two segments get a
/// `provider` namespace prefix.
fn mangle_tool_id(name: &str) -> ToolId {
    let mangled = name.replace('.', ":");
    ToolId::new(&mangled).unwrap_or_else(|_| {
        ToolId::new(format!("provider:{mangled}")).expect("prefixed id is valid")
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_tolerates_unknown_fields() {
        let value = serde_json::json!({
            "name": "figma",
            "version": "0.1.0",
            "description": "Figma surface",
            "provider_type": "external-executable",
            "protocol_version": "0",
            "tools": ["figma.get_document"],
            "verification_summary": {"status": "verified"},
            "runtime": {"language": "typescript"}
        });
        let manifest: ProviderManifest = serde_json::from_value(value).unwrap();
        assert_eq!(manifest.name, "figma");
        assert_eq!(manifest.tools, vec!["figma.get_document"]);
    }

    #[test]
    fn mangle_dotted_names() {
        assert_eq!(mangle_tool_id("mock.echo").as_str(), "mock:echo");
        assert_eq!(
            mangle_tool_id("figma.get_document").as_str(),
            "figma:get_document"
        );
        assert_eq!(
            mangle_tool_id("lonis:builtin:echo").as_str(),
            "lonis:builtin:echo"
        );
        assert_eq!(mangle_tool_id("bare").as_str(), "provider:bare");
    }
}
