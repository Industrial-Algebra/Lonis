// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! # lonis-core
//!
//! Minimal harness runtime for Lonis tools, building on [`lonis_schema`]:
//!
//! - the callable [`Tool`] trait ([`Capabilities`] + a uniform JSON-typed
//!   [`Tool::invoke`] boundary),
//! - an in-process [`ToolRegistry`],
//! - envelope/error rendering per [`OutputMode`] ([`render`] / [`render_error`]),
//! - and [`run_tool`], which enforces the amari split: a success [`Envelope`]
//!   goes to **stdout**, a structured [`ToolError`] goes to **stderr** with its
//!   exit code.
//!
//! See `docs/plans/lonis-schema-design.md` (decision #1) for the stdout/stderr
//! split this runtime enforces.

#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::io::Write;

use lonis_schema::{exit_code, Capabilities, Envelope, OutputMode, ToolContract, ToolError};

// ===========================================================================
// Tool trait
// ===========================================================================

/// A callable Lonis tool: [`Capabilities`] (self-description) plus a uniform
/// JSON-typed [`Tool::invoke`] boundary.
///
/// Tools accept [`serde_json::Value`] input and return a success [`Envelope`]
/// (rendered on stdout) or a [`ToolError`] (rendered on stderr). A typed tool
/// deserializes the input into its own request type and serializes its result
/// into the envelope payload.
pub trait Tool: Capabilities {
    /// Invoke the tool.
    ///
    /// # Errors
    /// Returns a [`ToolError`] (serialized to stderr) on failure.
    fn invoke(&self, input: serde_json::Value) -> Result<Envelope<serde_json::Value>, ToolError>;

    /// The tool's declared contract, if any (used by `lonis tools describe`).
    ///
    /// Owned so tools can build it without static/const gymnastics; called
    /// rarely (description only).
    fn contract(&self) -> Option<ToolContract> {
        None
    }
}

// ===========================================================================
// Registry
// ===========================================================================

/// Registry of in-process tools, keyed by tool id.
///
/// Deterministic iteration order (sorted by id) so `lonis tools list` and tests
/// are stable.
#[derive(Default)]
pub struct ToolRegistry {
    tools: BTreeMap<String, Box<dyn Tool>>,
}

impl ToolRegistry {
    /// Construct an empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a tool.
    ///
    /// # Errors
    /// Returns a [`ToolError`] with kind `already_registered` if a tool with the
    /// same id is already present.
    pub fn register(&mut self, tool: Box<dyn Tool>) -> Result<(), ToolError> {
        let id = tool.tool_id();
        if self.tools.contains_key(id.as_str()) {
            return Err(ToolError::new(
                "already_registered",
                format!("a tool with id `{}` is already registered", id.as_str()),
                exit_code::GENERIC,
            ));
        }
        self.tools.insert(id.as_str().to_owned(), tool);
        Ok(())
    }

    /// Look up a tool by id.
    #[must_use]
    pub fn get(&self, id: &str) -> Option<&dyn Tool> {
        self.tools.get(id).map(std::ops::Deref::deref)
    }

    /// Iterate registered tools (sorted by id).
    pub fn iter(&self) -> impl Iterator<Item = (&str, &dyn Tool)> {
        self.tools.iter().map(|(k, v)| (k.as_str(), v.as_ref()))
    }

    /// Number of registered tools.
    #[must_use]
    pub fn len(&self) -> usize {
        self.tools.len()
    }

    /// Whether the registry is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.tools.is_empty()
    }

    /// Invoke a tool by id with the given input.
    ///
    /// # Errors
    /// [`ToolError`] with kind `not_found` (exit code 3) if the id is unknown;
    /// otherwise the tool's own error.
    pub fn invoke(
        &self,
        id: &str,
        input: serde_json::Value,
    ) -> Result<Envelope<serde_json::Value>, ToolError> {
        match self.tools.get(id) {
            Some(tool) => tool.invoke(input),
            None => Err(ToolError::new(
                "not_found",
                format!("no tool with id `{id}`"),
                exit_code::NOT_FOUND,
            )),
        }
    }
}

// ===========================================================================
// Rendering (amari split)
// ===========================================================================

/// Render a success [`Envelope`] to `w` according to `mode`.
///
/// # Errors
/// Propagates serialization/IO errors.
pub fn render<W: Write>(
    env: &Envelope<serde_json::Value>,
    mode: OutputMode,
    w: &mut W,
) -> std::io::Result<()> {
    match mode {
        OutputMode::Json | OutputMode::Ndjson => {
            writeln!(w, "{}", serde_json::to_string(env).map_err(io_err)?)
        }
        OutputMode::Human => writeln!(
            w,
            "[{}] {}",
            env.tool.as_str(),
            serde_json::to_string_pretty(&env.result).map_err(io_err)?
        ),
    }
}

/// Render a [`ToolError`] to `w` according to `mode`. JSON/NDJSON emit the
/// structured error object; Human emits the short `kind: message` form.
///
/// # Errors
/// Propagates serialization/IO errors.
pub fn render_error<W: Write>(err: &ToolError, mode: OutputMode, w: &mut W) -> std::io::Result<()> {
    match mode {
        OutputMode::Json | OutputMode::Ndjson => {
            writeln!(w, "{}", serde_json::to_string(err).map_err(io_err)?)
        }
        OutputMode::Human => writeln!(w, "{err}"),
    }
}

/// Run a tool from a registry, rendering success to stdout and the structured
/// error to stderr with the tool's exit code. Returns the process exit code.
///
/// This is the amari split (decision #1): a parseable [`Envelope`] always on
/// stdout, diagnostics always on stderr.
#[must_use]
pub fn run_tool(
    registry: &ToolRegistry,
    id: &str,
    input: serde_json::Value,
    mode: OutputMode,
) -> u8 {
    match registry.invoke(id, input) {
        Ok(env) => match render(&env, mode, &mut std::io::stdout()) {
            Ok(()) => exit_code::SUCCESS,
            Err(_) => exit_code::GENERIC,
        },
        Err(err) => {
            let code = err.exit_code;
            let _ = render_error(&err, mode, &mut std::io::stderr());
            code
        }
    }
}

fn io_err(e: serde_json::Error) -> std::io::Error {
    std::io::Error::other(e)
}

#[cfg(test)]
mod tests {
    use super::*;
    use lonis_schema::{exit_code, Capabilities, Envelope, OutputMode, SchemaVersion, ToolId};

    // --- a test echo tool ---

    struct Echo;
    const FMTS: [OutputMode; 3] = [OutputMode::Human, OutputMode::Json, OutputMode::Ndjson];
    const MAP: &[(&str, u8)] = &[
        ("ok", exit_code::SUCCESS),
        ("bad_input", exit_code::INVALID_INPUT),
    ];

    impl Capabilities for Echo {
        fn schema_version(&self) -> SchemaVersion {
            SchemaVersion::default()
        }
        fn tool_version(&self) -> &str {
            "0.0.1"
        }
        fn output_formats(&self) -> &'static [OutputMode] {
            &FMTS
        }
        fn exit_code_map(&self) -> &'static [(&'static str, u8)] {
            MAP
        }
        fn tool_id(&self) -> ToolId {
            ToolId::new("lonis:test:echo").unwrap()
        }
    }

    impl Tool for Echo {
        fn invoke(
            &self,
            input: serde_json::Value,
        ) -> Result<Envelope<serde_json::Value>, ToolError> {
            if input.is_null() {
                return Err(ToolError::new(
                    "bad_input",
                    "echo requires non-null input",
                    exit_code::INVALID_INPUT,
                ));
            }
            Ok(Envelope::new(self.tool_id(), input))
        }
    }

    fn build_reg() -> ToolRegistry {
        let mut reg = ToolRegistry::new();
        reg.register(Box::new(Echo)).unwrap();
        reg
    }

    // --- registry ---

    #[test]
    fn register_and_get() {
        let mut reg = ToolRegistry::new();
        assert!(reg.is_empty());
        reg.register(Box::new(Echo)).unwrap();
        assert_eq!(reg.len(), 1);
        assert!(reg.get("lonis:test:echo").is_some());
        assert!(reg.get("lonis:nope:x").is_none());
    }

    #[test]
    fn rejects_duplicate() {
        let mut reg = ToolRegistry::new();
        reg.register(Box::new(Echo)).unwrap();
        let err = reg.register(Box::new(Echo)).unwrap_err();
        assert_eq!(err.kind, "already_registered");
        assert_eq!(err.exit_code, exit_code::GENERIC);
    }

    #[test]
    fn iter_sorted() {
        let reg = build_reg();
        let ids: Vec<_> = reg.iter().map(|(id, _)| id).collect();
        assert_eq!(ids, vec!["lonis:test:echo"]);
    }

    // --- invoke ---

    #[test]
    fn invoke_returns_envelope() {
        let reg = build_reg();
        let env = reg
            .invoke("lonis:test:echo", serde_json::json!({"hi": 1}))
            .unwrap();
        assert_eq!(env.tool.as_str(), "lonis:test:echo");
        assert_eq!(env.result, serde_json::json!({"hi": 1}));
    }

    #[test]
    fn invoke_unknown_is_not_found() {
        let reg = build_reg();
        let err = reg
            .invoke("lonis:nope:x", serde_json::Value::Null)
            .unwrap_err();
        assert_eq!(err.kind, "not_found");
        assert_eq!(err.exit_code, exit_code::NOT_FOUND);
    }

    #[test]
    fn invoke_propagates_tool_error() {
        let reg = build_reg();
        let err = reg
            .invoke("lonis:test:echo", serde_json::Value::Null)
            .unwrap_err();
        assert_eq!(err.kind, "bad_input");
        assert_eq!(err.exit_code, exit_code::INVALID_INPUT);
    }

    // --- render ---

    #[test]
    fn render_json_round_trips() {
        let env = Envelope::new(
            ToolId::new("lonis:test:echo").unwrap(),
            serde_json::json!({"a": 1}),
        );
        let mut buf = Vec::new();
        render(&env, OutputMode::Json, &mut buf).unwrap();
        let parsed: Envelope<serde_json::Value> =
            serde_json::from_str(std::str::from_utf8(&buf).unwrap().trim()).unwrap();
        assert_eq!(parsed.result, serde_json::json!({"a": 1}));
        assert_eq!(parsed.tool.as_str(), "lonis:test:echo");
    }

    #[test]
    fn render_human_is_pretty() {
        let env = Envelope::new(
            ToolId::new("lonis:test:echo").unwrap(),
            serde_json::json!({"a": 1}),
        );
        let mut buf = Vec::new();
        render(&env, OutputMode::Human, &mut buf).unwrap();
        let s = String::from_utf8(buf).unwrap();
        assert!(s.contains("[lonis:test:echo]"));
        assert!(s.contains("\"a\": 1"));
    }

    #[test]
    fn render_error_json_structured() {
        let err = ToolError::new("bad_input", "nope", exit_code::INVALID_INPUT);
        let mut buf = Vec::new();
        render_error(&err, OutputMode::Json, &mut buf).unwrap();
        let s = String::from_utf8(buf).unwrap();
        assert!(s.contains("\"kind\":\"bad_input\""));
        assert!(s.contains("\"exit_code\":2"));
    }

    #[test]
    fn render_error_human_display() {
        let err = ToolError::new("bad_input", "nope", exit_code::INVALID_INPUT);
        let mut buf = Vec::new();
        render_error(&err, OutputMode::Human, &mut buf).unwrap();
        assert_eq!(String::from_utf8_lossy(&buf).trim(), "bad_input: nope");
    }

    // --- run_tool (amari split) ---

    #[test]
    fn run_tool_success_exit_zero() {
        let reg = build_reg();
        let code = run_tool(
            &reg,
            "lonis:test:echo",
            serde_json::json!(42),
            OutputMode::Json,
        );
        assert_eq!(code, exit_code::SUCCESS);
    }

    #[test]
    fn run_tool_not_found_exit_three() {
        let reg = build_reg();
        let code = run_tool(
            &reg,
            "lonis:nope:x",
            serde_json::Value::Null,
            OutputMode::Json,
        );
        assert_eq!(code, exit_code::NOT_FOUND);
    }

    #[test]
    fn run_tool_bad_input_exit_two() {
        let reg = build_reg();
        let code = run_tool(
            &reg,
            "lonis:test:echo",
            serde_json::Value::Null,
            OutputMode::Json,
        );
        assert_eq!(code, exit_code::INVALID_INPUT);
    }

    #[test]
    fn contract_default_is_none() {
        let echo = Echo;
        assert!(echo.contract().is_none());
    }
}
