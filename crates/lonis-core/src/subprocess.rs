// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! `SubprocessTool` — host an external composable CLI as a Lonis [`Tool`].
//!
//! This is the Lonis thesis in action: *"MCP exposes servers to models;
//! Lonis exposes tools to agents."* A subprocess adapter is how Lonis hosts
//! arbitrary tools — and the erased seam of ADR-0002: across the process
//! boundary it is always JSON, so the adapter emits [`SeedBlock`]s, with
//! unknown kinds landing in [`BlockKind::Extension`] losslessly.
//!
//! ## Wire protocol (ADR-0003)
//!
//! - **in**: the input JSON value on the child's stdin (stdin then closed),
//! - **out (success)**: blocks on stdout — a JSON array, or one block per
//!   line (ndjson), or a single block object,
//! - **out (error)**: a structured [`ToolError`] JSON on stderr plus a
//!   nonzero exit code; unstructured stderr maps to kind `tool_failed`,
//! - **bounds** (doctrine §2.7 `BlockBounds`, first-class): a hard wall-clock
//!   timeout and byte caps on both output streams; exceeding either kills
//!   the child and reports `LIMIT_EXCEEDED`.
//!
//! Isolation follows the amari-discovery probe blueprint: no shell, a
//! cleared environment (only `PATH` inherited, plus explicit additions), and
//! a configurable working directory (default: the system temp dir).

use std::io::Write as _;
use std::path::PathBuf;
use std::process::{Child, Command, ExitStatus, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use lonis_schema::block::kinds::{BlockKind, ResultPayload};
use lonis_schema::{
    exit_code, json_content_hash, Attribution, Block, Capabilities, Cost, Determinism, OutputMode,
    ReplayProvenance, SchemaRef, SchemaVersion, SeedBlock, SideEffects, ToolContract, ToolError,
    ToolId,
};

use crate::Tool;

const FORMATS: [OutputMode; 3] = [OutputMode::Human, OutputMode::Json, OutputMode::Ndjson];
const EXIT_MAP: &[(&str, u8)] = &[
    ("ok", exit_code::SUCCESS),
    ("unavailable", exit_code::TOOL_FAILED),
    ("tool_failed", exit_code::TOOL_FAILED),
    ("timeout", exit_code::LIMIT_EXCEEDED),
    ("output_limit_exceeded", exit_code::LIMIT_EXCEEDED),
    ("io", exit_code::IO),
    ("invalid_output", exit_code::SERIALIZATION),
];

const DEFAULT_TIMEOUT_MILLIS: u64 = 5_000;
const DEFAULT_MAX_STDOUT_BYTES: u64 = 1_048_576;
const DEFAULT_MAX_STDERR_BYTES: u64 = 262_144;
const POLL_INTERVAL: Duration = Duration::from_millis(10);

/// How a subprocess tool's stdout maps to blocks.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum StdoutMapping {
    /// Lonis-native: stdout carries blocks (JSON array, ndjson, or a single
    /// block object).
    #[default]
    Blocks,
    /// Legacy composable CLI: raw stdout text is wrapped in a `result` block
    /// attributed to the tool.
    Text,
}

/// Availability tri-state (the amari `capabilities` pattern): a tool the
/// harness knows about may still be missing or non-executable on this host.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Availability {
    /// The command exists and is executable (or resolves via `PATH`).
    Ready,
    /// The command path does not exist.
    Missing {
        /// Human-readable reason.
        reason: String,
    },
    /// The command path exists but is not executable.
    NotExecutable {
        /// Human-readable reason.
        reason: String,
    },
}

impl Availability {
    /// Whether the tool can be invoked.
    #[must_use]
    pub fn is_ready(&self) -> bool {
        matches!(self, Self::Ready)
    }

    /// The reason the tool is unavailable, when it is.
    #[must_use]
    pub fn reason(&self) -> Option<&str> {
        match self {
            Self::Ready => None,
            Self::Missing { reason } | Self::NotExecutable { reason } => Some(reason),
        }
    }
}

/// A [`Tool`] that spawns an external composable CLI per invocation.
///
/// Bounded per doctrine §2.7: a hard timeout (kill on exceed) and byte caps
/// on stdout/stderr, all configurable; defaults are the amari probe bounds
/// (5 s, 1 MiB stdout, 256 KiB stderr).
#[derive(Debug)]
pub struct SubprocessTool {
    id: ToolId,
    command: PathBuf,
    args: Vec<String>,
    description: String,
    version: String,
    mapping: StdoutMapping,
    timeout: Duration,
    max_stdout_bytes: u64,
    max_stderr_bytes: u64,
    cwd: Option<PathBuf>,
    env: Vec<(String, String)>,
}

impl SubprocessTool {
    /// Create a subprocess tool invoking `command` (bare names resolve via
    /// the inherited `PATH`).
    #[must_use]
    pub fn new(id: ToolId, command: impl Into<PathBuf>) -> Self {
        let command = command.into();
        Self {
            description: format!("External tool `{}`", command.display()),
            id,
            command,
            args: Vec::new(),
            version: "unknown".to_owned(),
            mapping: StdoutMapping::default(),
            timeout: Duration::from_millis(DEFAULT_TIMEOUT_MILLIS),
            max_stdout_bytes: DEFAULT_MAX_STDOUT_BYTES,
            max_stderr_bytes: DEFAULT_MAX_STDERR_BYTES,
            cwd: None,
            env: Vec::new(),
        }
    }

    /// Set fixed argument prefix for every invocation.
    #[must_use]
    pub fn with_args(mut self, args: Vec<String>) -> Self {
        self.args = args;
        self
    }

    /// Set the human description used in the tool contract.
    #[must_use]
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Set the external tool's version string.
    #[must_use]
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = version.into();
        self
    }

    /// Set the stdout mapping (default: [`StdoutMapping::Blocks`]).
    #[must_use]
    pub fn with_mapping(mut self, mapping: StdoutMapping) -> Self {
        self.mapping = mapping;
        self
    }

    /// Set the hard wall-clock timeout in milliseconds (default 5 000).
    #[must_use]
    pub fn with_timeout_millis(mut self, millis: u64) -> Self {
        self.timeout = Duration::from_millis(millis);
        self
    }

    /// Set the stdout byte cap (default 1 MiB).
    #[must_use]
    pub fn with_max_stdout_bytes(mut self, bytes: u64) -> Self {
        self.max_stdout_bytes = bytes;
        self
    }

    /// Set the stderr byte cap (default 256 KiB).
    #[must_use]
    pub fn with_max_stderr_bytes(mut self, bytes: u64) -> Self {
        self.max_stderr_bytes = bytes;
        self
    }

    /// Set the working directory (default: the system temp dir — a neutral
    /// cwd, per the amari probe isolation pattern).
    #[must_use]
    pub fn with_cwd(mut self, cwd: impl Into<PathBuf>) -> Self {
        self.cwd = Some(cwd.into());
        self
    }

    /// Add environment variables on top of the cleared environment (`PATH`
    /// is always inherited).
    #[must_use]
    pub fn with_env(mut self, vars: Vec<(String, String)>) -> Self {
        self.env = vars;
        self
    }

    /// Probe whether the command is invocable on this host.
    ///
    /// Bare command names (no path separator) defer to `PATH` resolution at
    /// spawn time and report [`Availability::Ready`].
    #[must_use]
    pub fn availability(&self) -> Availability {
        let command = &self.command;
        if command.components().count() < 2 {
            return Availability::Ready;
        }
        match std::fs::metadata(command) {
            Err(_) => Availability::Missing {
                reason: format!("`{}` does not exist", command.display()),
            },
            Ok(metadata) => {
                if is_executable(&metadata) {
                    Availability::Ready
                } else {
                    Availability::NotExecutable {
                        reason: format!("`{}` is not executable", command.display()),
                    }
                }
            }
        }
    }

    fn neutral_cwd(&self) -> PathBuf {
        self.cwd.clone().unwrap_or_else(std::env::temp_dir)
    }

    fn unavailable_error(&self, availability: &Availability) -> ToolError {
        ToolError::new(
            "unavailable",
            format!(
                "tool `{}` is unavailable: {}",
                self.id.as_str(),
                availability.reason().unwrap_or("unknown reason")
            ),
            exit_code::TOOL_FAILED,
        )
    }
}

#[cfg(unix)]
fn is_executable(metadata: &std::fs::Metadata) -> bool {
    use std::os::unix::fs::PermissionsExt as _;
    metadata.is_file() && metadata.permissions().mode() & 0o111 != 0
}

#[cfg(not(unix))]
fn is_executable(metadata: &std::fs::Metadata) -> bool {
    metadata.is_file()
}

impl Capabilities for SubprocessTool {
    fn schema_version(&self) -> SchemaVersion {
        SchemaVersion::default()
    }
    fn tool_version(&self) -> &str {
        &self.version
    }
    fn output_formats(&self) -> &'static [OutputMode] {
        &FORMATS
    }
    fn exit_code_map(&self) -> &'static [(&'static str, u8)] {
        EXIT_MAP
    }
    fn tool_id(&self) -> ToolId {
        self.id.clone()
    }
}

/// Drain a child output stream into a bounded buffer; sets `exceeded` and
/// keeps discarding (so the child never blocks on a full pipe) past the cap.
fn drain_bounded<R: std::io::Read + Send + 'static>(
    stream: R,
    cap: u64,
    exceeded: Arc<AtomicBool>,
) -> std::thread::JoinHandle<Vec<u8>> {
    std::thread::spawn(move || {
        let mut reader = stream;
        let mut buf = Vec::new();
        let mut chunk = [0_u8; 8192];
        loop {
            match reader.read(&mut chunk) {
                Ok(0) | Err(_) => break,
                Ok(n) => {
                    if (buf.len() as u64) < cap {
                        let room = (cap - buf.len() as u64) as usize;
                        buf.extend_from_slice(&chunk[..n.min(room)]);
                    }
                    if (buf.len() as u64) >= cap && n > 0 {
                        exceeded.store(true, Ordering::Relaxed);
                    }
                }
            }
        }
        buf
    })
}

/// Parse success stdout into blocks: JSON array, else one block per line
/// (ndjson), else a single block object.
fn parse_blocks(stdout: &str) -> Result<Vec<SeedBlock>, ToolError> {
    let invalid =
        |message: String| ToolError::new("invalid_output", message, exit_code::SERIALIZATION);
    let trimmed = stdout.trim();
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }
    if trimmed.starts_with('[') {
        return serde_json::from_str(trimmed)
            .map_err(|err| invalid(format!("stdout is not a block array: {err}")));
    }
    let mut blocks = Vec::new();
    for line in trimmed.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let block: SeedBlock = serde_json::from_str(line)
            .map_err(|err| invalid(format!("stdout line is not a block: {err}")))?;
        blocks.push(block);
    }
    Ok(blocks)
}

/// Wrap legacy plain-text stdout in a `result` block attributed to the tool.
fn text_block(tool: &SubprocessTool, stdout: &str, input_hash: String) -> SeedBlock {
    Block::new(
        Attribution::new(tool.id.as_str(), tool.id.as_str()),
        BlockKind::Result(ResultPayload {
            output: serde_json::json!({"stdout": stdout.trim()}),
            score: None,
            evidence: Vec::new(),
            validated_assumptions: Vec::new(),
            refuted_assumptions: Vec::new(),
            resources: None,
            duration_micros: None,
        }),
    )
    .with_provenance(ReplayProvenance {
        tool_version: Some(tool.version.clone()),
        input_hash: Some(input_hash),
        ..ReplayProvenance::default()
    })
}

/// Map a nonzero exit: structured [`ToolError`] on stderr wins; otherwise a
/// generic `tool_failed` carrying the trimmed stderr and the child's code.
fn map_failure(status: &ExitStatus, stderr: &[u8]) -> ToolError {
    let text = String::from_utf8_lossy(stderr).trim().to_owned();
    if let Ok(err) = serde_json::from_str::<ToolError>(&text) {
        return err;
    }
    let code = status
        .code()
        .and_then(|c| u8::try_from(c).ok())
        .filter(|&c| c != 0)
        .unwrap_or(exit_code::GENERIC);
    ToolError::new(
        "tool_failed",
        if text.is_empty() {
            format!("subprocess exited with status {status}")
        } else {
            text
        },
        code,
    )
}

fn kill_and_reap(child: &mut Child) {
    let _ = child.kill();
    let _ = child.wait();
}

impl Tool<BlockKind> for SubprocessTool {
    fn invoke(&self, input: serde_json::Value) -> Result<Vec<SeedBlock>, ToolError> {
        let availability = self.availability();
        if !availability.is_ready() {
            return Err(self.unavailable_error(&availability));
        }

        let mut command = Command::new(&self.command);
        command
            .args(&self.args)
            .env_clear()
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .current_dir(self.neutral_cwd());
        // Keep PATH so bare command names resolve; add explicit vars.
        if let Some(path) = std::env::var_os("PATH") {
            command.env("PATH", path);
        }
        command.envs(self.env.iter().cloned());

        let mut child = command.spawn().map_err(|err| {
            ToolError::new(
                "io",
                format!("failed to spawn `{}`: {err}", self.command.display()),
                exit_code::IO,
            )
        })?;

        // Feed input, then close stdin so the child sees EOF.
        if let Some(mut stdin) = child.stdin.take() {
            let _ = stdin.write_all(input.to_string().as_bytes());
        }

        let stdout_exceeded = Arc::new(AtomicBool::new(false));
        let stderr_exceeded = Arc::new(AtomicBool::new(false));
        let stdout_drain = drain_bounded(
            child.stdout.take().expect("stdout piped"),
            self.max_stdout_bytes,
            Arc::clone(&stdout_exceeded),
        );
        let stderr_drain = drain_bounded(
            child.stderr.take().expect("stderr piped"),
            self.max_stderr_bytes,
            Arc::clone(&stderr_exceeded),
        );

        let start = Instant::now();
        let status = loop {
            if stdout_exceeded.load(Ordering::Relaxed) || stderr_exceeded.load(Ordering::Relaxed) {
                kill_and_reap(&mut child);
                let _ = stdout_drain.join();
                let _ = stderr_drain.join();
                return Err(ToolError::new(
                    "output_limit_exceeded",
                    format!("tool `{}` exceeded its output byte cap", self.id.as_str()),
                    exit_code::LIMIT_EXCEEDED,
                ));
            }
            if start.elapsed() > self.timeout {
                kill_and_reap(&mut child);
                let _ = stdout_drain.join();
                let _ = stderr_drain.join();
                return Err(ToolError::new(
                    "timeout",
                    format!(
                        "tool `{}` exceeded its {}ms timeout",
                        self.id.as_str(),
                        self.timeout.as_millis()
                    ),
                    exit_code::LIMIT_EXCEEDED,
                ));
            }
            match child.try_wait() {
                Ok(Some(status)) => break status,
                Ok(None) => std::thread::sleep(POLL_INTERVAL),
                Err(err) => {
                    kill_and_reap(&mut child);
                    return Err(ToolError::new(
                        "io",
                        format!("failed waiting on `{}`: {err}", self.command.display()),
                        exit_code::IO,
                    ));
                }
            }
        };

        let stdout = stdout_drain.join().unwrap_or_default();
        let stderr = stderr_drain.join().unwrap_or_default();

        if !status.success() {
            return Err(map_failure(&status, &stderr));
        }

        let stdout_text = String::from_utf8_lossy(&stdout).into_owned();
        match self.mapping {
            StdoutMapping::Blocks => parse_blocks(&stdout_text),
            StdoutMapping::Text => Ok(vec![text_block(
                self,
                &stdout_text,
                json_content_hash(&input),
            )]),
        }
    }

    fn contract(&self) -> Option<ToolContract> {
        Some(ToolContract {
            name: self.id.clone(),
            description: self.description.clone(),
            input_schema: SchemaRef("lonis.subprocess/input/v1".into()),
            output_schema: SchemaRef("lonis.subprocess/output/v1".into()),
            determinism: Determinism::Nondeterministic,
            side_effects: SideEffects::MutatesExternal,
            cost: Cost::Medium,
            capabilities: Vec::new(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id() -> ToolId {
        ToolId::new("lonis:test:subprocess").unwrap()
    }

    fn block_json(kind: &str) -> serde_json::Value {
        serde_json::json!({
            "schema_version": "lonis.block/v1",
            "attribution": {
                "identity": "lonis:test:subprocess",
                "provenance": {"when": "2026-08-10T00:00:00Z", "producer": "lonis:test:subprocess"}
            },
            "payload": {"kind": kind, "data": (if kind == "message" {
                serde_json::json!({"content": "hi"})
            } else {
                serde_json::json!({"gadget": 1})
            })}
        })
    }

    #[test]
    fn parse_blocks_accepts_array() {
        let wire = serde_json::to_string(&serde_json::json!([block_json("message")])).unwrap();
        let blocks = parse_blocks(&wire).unwrap();
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].payload().kind_name(), "message");
    }

    #[test]
    fn parse_blocks_accepts_ndjson_and_single_object() {
        let one = serde_json::to_string(&block_json("message")).unwrap();
        let two = serde_json::to_string(&block_json("widget")).unwrap();
        let blocks = parse_blocks(&format!("{one}\n{two}\n")).unwrap();
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[1].payload().kind_name(), "widget");
        let single = parse_blocks(&one).unwrap();
        assert_eq!(single.len(), 1);
    }

    #[test]
    fn parse_blocks_empty_stdout_is_zero_blocks() {
        assert_eq!(parse_blocks("  \n ").unwrap().len(), 0);
    }

    #[test]
    fn parse_blocks_rejects_garbage() {
        let err = parse_blocks("not json at all").unwrap_err();
        assert_eq!(err.kind, "invalid_output");
        assert_eq!(err.exit_code, exit_code::SERIALIZATION);
    }

    #[test]
    fn availability_missing_path() {
        let tool = SubprocessTool::new(id(), "/definitely/not/here");
        assert_eq!(
            tool.availability(),
            Availability::Missing {
                reason: "`/definitely/not/here` does not exist".into()
            }
        );
    }

    #[cfg(unix)]
    #[test]
    fn availability_not_executable() {
        let tool = SubprocessTool::new(id(), "/etc/hostname");
        assert!(!tool.availability().is_ready());
    }

    #[test]
    fn availability_bare_name_defers_to_path() {
        let tool = SubprocessTool::new(id(), "ls");
        assert!(tool.availability().is_ready());
    }

    #[test]
    fn builders_set_bounds() {
        let tool = SubprocessTool::new(id(), "ls")
            .with_timeout_millis(1234)
            .with_max_stdout_bytes(99)
            .with_max_stderr_bytes(42)
            .with_mapping(StdoutMapping::Text);
        assert_eq!(tool.timeout, Duration::from_millis(1234));
        assert_eq!(tool.max_stdout_bytes, 99);
        assert_eq!(tool.max_stderr_bytes, 42);
        assert_eq!(tool.mapping, StdoutMapping::Text);
    }

    #[test]
    fn map_failure_prefers_structured_stderr() {
        let err =
            ToolError::new("structured", "from child", 5).with_details(serde_json::json!({"x": 1}));
        let status = Command::new("false").status().unwrap();
        let mapped = map_failure(&status, serde_json::to_string(&err).unwrap().as_bytes());
        assert_eq!(mapped.kind, "structured");
        assert_eq!(mapped.exit_code, 5);
    }

    #[cfg(unix)]
    #[test]
    fn map_failure_plain_stderr_carries_child_code() {
        let status = Command::new("sh").args(["-c", "exit 2"]).status().unwrap();
        let mapped = map_failure(&status, b"boom");
        assert_eq!(mapped.kind, "tool_failed");
        assert_eq!(mapped.message, "boom");
        assert_eq!(mapped.exit_code, 2);
    }
}
