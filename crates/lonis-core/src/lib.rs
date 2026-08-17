// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! # lonis-core
//!
//! Minimal harness runtime for Lonis tools, building on [`lonis_schema`]:
//!
//! - the callable [`Tool`] trait ([`Capabilities`] + a uniform JSON-typed
//!   [`Tool::invoke`] boundary returning [`Block`]s — ADR-0001),
//! - an in-process [`ToolRegistry`],
//! - block/error rendering per [`OutputMode`] ([`render`] / [`render_error`]),
//! - and [`run_tool`], which enforces the amari split: success [`Block`]s go
//!   to **stdout**, a structured [`ToolError`] goes to **stderr** with its
//!   exit code.
//!
//! See `docs/adr/0001-block-contract.md` for the output contract.

#![forbid(unsafe_code)]

pub mod provider;
pub mod stream;
pub mod subprocess;

pub use provider::{ProviderManifest, ProviderToolList, ProviderToolSummary, SubprocessProvider};
pub use stream::BlockStream;

pub use subprocess::{Availability, StdoutMapping, SubprocessTool};

use std::collections::BTreeMap;
use std::io::Write;

use lonis_schema::{
    exit_code, Block, BlockPayload, Capabilities, OutputMode, ToolContract, ToolError,
};

// ===========================================================================
// Tool trait
// ===========================================================================

/// A callable Lonis tool: [`Capabilities`] (self-description) plus a uniform
/// JSON-typed [`Tool::invoke`] boundary, generic over its typed payload `P`
/// (ADR-0002).
///
/// Tools accept [`serde_json::Value`] input and return the [`Block`]s they
/// emit (rendered on stdout) or a [`ToolError`] (rendered on stderr). A tool
/// may emit zero, one, or many blocks (ADR-0001). A vertical implements
/// `Tool<MyPayload>` for each of its tools and gets a fully-typed,
/// homogeneous registry; the umbrella host instantiates `P = BlockKind` and
/// erases only across subprocess JSON channels.
pub trait Tool<P: BlockPayload>: Capabilities {
    /// Invoke the tool.
    ///
    /// # Errors
    /// Returns a [`ToolError`] (serialized to stderr) on failure.
    fn invoke(&self, input: serde_json::Value) -> Result<Vec<Block<P>>, ToolError>;

    /// Invoke the tool, streaming blocks as they are produced (ADR-0009).
    ///
    /// The default collects [`Tool::invoke`] and replays it as a stream;
    /// tools with genuine incremental output (e.g. [`crate::SubprocessTool`]
    /// over ndjson) override this. A failed invocation yields its
    /// [`ToolError`] as the stream's final item.
    ///
    /// # Errors
    /// Returns immediately on startup errors (unknown tool, unavailable
    /// binary); mid-stream failures surface as stream items.
    fn invoke_stream(&self, input: serde_json::Value) -> Result<BlockStream<P>, ToolError> {
        Ok(BlockStream::from_blocks(self.invoke(input)?))
    }

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

/// Registry of in-process tools, keyed by tool id, homogeneous in the
/// payload type `P` (ADR-0002: a vertical's registry is fully typed).
///
/// Deterministic iteration order (sorted by id) so `lonis tools list` and tests
/// are stable.
pub struct ToolRegistry<P: BlockPayload> {
    tools: BTreeMap<String, Box<dyn Tool<P>>>,
}

impl<P: BlockPayload> Default for ToolRegistry<P> {
    fn default() -> Self {
        Self {
            tools: BTreeMap::new(),
        }
    }
}

impl<P: BlockPayload> ToolRegistry<P> {
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
    pub fn register(&mut self, tool: Box<dyn Tool<P>>) -> Result<(), ToolError> {
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
    pub fn get(&self, id: &str) -> Option<&dyn Tool<P>> {
        self.tools.get(id).map(std::ops::Deref::deref)
    }

    /// Iterate registered tools (sorted by id).
    pub fn iter(&self) -> impl Iterator<Item = (&str, &dyn Tool<P>)> {
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

    /// Invoke a tool by id, streaming blocks as produced (ADR-0009).
    ///
    /// # Errors
    /// [`ToolError`] with kind `not_found` (exit code 3) if the id is unknown;
    /// otherwise the tool's startup error.
    pub fn invoke_stream(
        &self,
        id: &str,
        input: serde_json::Value,
    ) -> Result<BlockStream<P>, ToolError> {
        match self.tools.get(id) {
            Some(tool) => tool.invoke_stream(input),
            None => Err(ToolError::new(
                "not_found",
                format!("no tool with id `{id}`"),
                exit_code::NOT_FOUND,
            )),
        }
    }

    /// Invoke a tool by id with the given input.
    ///
    /// # Errors
    /// [`ToolError`] with kind `not_found` (exit code 3) if the id is unknown;
    /// otherwise the tool's own error.
    pub fn invoke(&self, id: &str, input: serde_json::Value) -> Result<Vec<Block<P>>, ToolError> {
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

/// Render success [`Block`]s to `w` according to `mode`.
///
/// - `Json` emits one JSON array of blocks (stable machine shape, even for a
///   single block),
/// - `Ndjson` emits one block per line (parity with amari-discovery's
///   streaming mode),
/// - `Human` emits each block's `render_human` line.
///
/// # Errors
/// Propagates serialization/IO errors.
pub fn render<P: BlockPayload, W: Write>(
    blocks: &[Block<P>],
    mode: OutputMode,
    w: &mut W,
) -> std::io::Result<()> {
    match mode {
        OutputMode::Json => writeln!(w, "{}", serde_json::to_string(blocks).map_err(io_err)?),
        OutputMode::Ndjson => {
            for block in blocks {
                writeln!(w, "{}", serde_json::to_string(block).map_err(io_err)?)?;
            }
            Ok(())
        }
        OutputMode::Human => {
            for block in blocks {
                writeln!(w, "{}", block.render_human())?;
            }
            Ok(())
        }
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

/// Run a tool from a registry, rendering success blocks to stdout and the
/// structured error to stderr with the tool's exit code. Returns the process
/// exit code.
///
/// This is the amari split (decision #1): parseable [`Block`]s always on
/// stdout, diagnostics always on stderr.
#[must_use]
pub fn run_tool<P: BlockPayload>(
    registry: &ToolRegistry<P>,
    id: &str,
    input: serde_json::Value,
    mode: OutputMode,
) -> u8 {
    match registry.invoke(id, input) {
        Ok(blocks) => match render(&blocks, mode, &mut std::io::stdout()) {
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

/// Run a tool streaming its blocks (ADR-0009), rendering incrementally to
/// stdout and a terminal error to stderr with its exit code.
///
/// - `Ndjson`/`Human` render each block as it arrives,
/// - `Json` buffers and emits one array at the end (a valid JSON document
///   cannot be emitted incrementally),
/// - a terminal [`ToolError`] item renders to stderr and ends the run with
///   its exit code; blocks delivered before it stay on stdout.
#[must_use]
pub fn run_stream<P: BlockPayload>(
    registry: &ToolRegistry<P>,
    id: &str,
    input: serde_json::Value,
    mode: OutputMode,
) -> u8 {
    let stream = match registry.invoke_stream(id, input) {
        Ok(stream) => stream,
        Err(err) => {
            let code = err.exit_code;
            let _ = render_error(&err, mode, &mut std::io::stderr());
            return code;
        }
    };
    let mut buffered = Vec::new();
    for item in stream {
        match item {
            Ok(block) => match mode {
                OutputMode::Json => buffered.push(block),
                OutputMode::Ndjson => {
                    let _ = writeln!(
                        std::io::stdout(),
                        "{}",
                        serde_json::to_string(&block).unwrap_or_else(|_| "null".into())
                    );
                }
                OutputMode::Human => {
                    let _ = writeln!(std::io::stdout(), "{}", block.render_human());
                }
            },
            Err(err) => {
                let code = err.exit_code;
                let _ = render_error(&err, mode, &mut std::io::stderr());
                return code;
            }
        }
    }
    if mode == OutputMode::Json {
        let _ = writeln!(
            std::io::stdout(),
            "{}",
            serde_json::to_string(&buffered).unwrap_or_else(|_| "[]".into())
        );
    }
    exit_code::SUCCESS
}

#[cfg(test)]
mod tests {
    use super::*;
    use lonis_schema::block::kinds::{BlockKind, Message, ResultPayload};
    use lonis_schema::{
        exit_code, Attribution, Capabilities, OutputMode, SchemaVersion, SeedBlock, ToolId,
    };

    // --- a test echo tool ---

    struct Echo;
    const FMTS: [OutputMode; 3] = [OutputMode::Human, OutputMode::Json, OutputMode::Ndjson];
    const MAP: &[(&str, u8)] = &[
        ("ok", exit_code::SUCCESS),
        ("bad_input", exit_code::INVALID_INPUT),
    ];

    fn attribution(id: &ToolId) -> Attribution {
        Attribution::new(id.as_str(), id.as_str())
    }

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

    impl Tool<BlockKind> for Echo {
        fn invoke(&self, input: serde_json::Value) -> Result<Vec<SeedBlock>, ToolError> {
            if input.is_null() {
                return Err(ToolError::new(
                    "bad_input",
                    "echo requires non-null input",
                    exit_code::INVALID_INPUT,
                ));
            }
            let id = self.tool_id();
            Ok(vec![
                Block::new(
                    attribution(&id),
                    BlockKind::Message(Message {
                        role: Some("tool".into()),
                        content: "echoing".into(),
                        reply_to: None,
                    }),
                ),
                Block::new(
                    attribution(&id),
                    BlockKind::Result(ResultPayload {
                        output: input,
                        score: None,
                        evidence: Vec::new(),
                        validated_assumptions: Vec::new(),
                        refuted_assumptions: Vec::new(),
                        resources: None,
                        duration_micros: None,
                    }),
                ),
            ])
        }
    }

    fn build_reg() -> ToolRegistry<BlockKind> {
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
    fn invoke_returns_blocks() {
        let reg = build_reg();
        let blocks = reg
            .invoke("lonis:test:echo", serde_json::json!({"hi": 1}))
            .unwrap();
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0].payload().kind_name(), "message");
        assert_eq!(blocks[1].payload().kind_name(), "result");
        assert_eq!(blocks[1].attribution.provenance.producer, "lonis:test:echo");
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

    fn echo_blocks() -> Vec<SeedBlock> {
        Echo.invoke(serde_json::json!({"a": 1})).unwrap()
    }

    #[test]
    fn render_json_emits_array_of_blocks() {
        let blocks = echo_blocks();
        let mut buf = Vec::new();
        render(&blocks, OutputMode::Json, &mut buf).unwrap();
        let parsed: Vec<SeedBlock> =
            serde_json::from_str(std::str::from_utf8(&buf).unwrap().trim()).unwrap();
        assert_eq!(parsed, blocks);
    }

    #[test]
    fn render_ndjson_emits_one_block_per_line() {
        let blocks = echo_blocks();
        let mut buf = Vec::new();
        render(&blocks, OutputMode::Ndjson, &mut buf).unwrap();
        let text = String::from_utf8(buf).unwrap();
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 2);
        for (line, block) in lines.iter().zip(&blocks) {
            let parsed: SeedBlock = serde_json::from_str(line).unwrap();
            assert_eq!(&parsed, block);
        }
    }

    #[test]
    fn render_human_uses_render_human_per_block() {
        let blocks = echo_blocks();
        let mut buf = Vec::new();
        render(&blocks, OutputMode::Human, &mut buf).unwrap();
        let text = String::from_utf8(buf).unwrap();
        assert!(text.contains("message: echoing"));
        assert!(text.contains("result:"));
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
