# ADR-0003: The subprocess tool wire protocol

- Status: accepted
- Date: 2026-08-10
- Branch: `feature/subprocess-tool`
- References: ADR-0001 (forward guidance), ADR-0002 (the erased seam);
  amari-discovery `src/probes/supervisor.rs`;
  `docs/plans/2026-03-28-lonis-external-provider-protocol-spec-v0.md`

## Context

The handoff's third work item: a `SubprocessTool` implementing `Tool` by
spawning composable external CLIs — the Lonis thesis (*"MCP exposes servers to
models; Lonis exposes tools to agents"*) and, per ADR-0002, the erased seam
where typing gives way to JSON. The March-era external-provider spec (v0)
sketched a richer *provider* model (one executable hosting many tools behind
`manifest` / `tools list` / `tools describe` / `call` subcommands); this ADR
settles the foundational invocation protocol that both models share.

## Decision

`SubprocessTool` (in `lonis-core`) hosts one external CLI as one
`Tool<BlockKind>` (the seed payload — the umbrella host's type). Per
invocation:

- **Input**: the input JSON value on the child's stdin, then stdin closes
  (EOF). No shell, no arg interpolation — fixed argv from configuration.
- **Success output**: blocks on stdout in any of three shapes — a JSON array,
  one block per line (ndjson), or a single block object. Unknown kinds land
  in `BlockKind::Extension` losslessly (ADR-0002's erased seam doing its
  job). A `StdoutMapping::Text` mode wraps legacy plain-text CLI output in a
  `result` block attributed to the tool with a pinned `input_hash`.
- **Errors**: a structured `ToolError` JSON on stderr plus a nonzero exit
  code (propagated verbatim — the subprocess speaks the same error protocol);
  unstructured stderr maps to kind `tool_failed` carrying the child's exit
  code. Stdout that is not blocks maps to `invalid_output` (exit code 9,
  `SERIALIZATION`).
- **Bounds** (doctrine §2.7 `BlockBounds`, first-class): a hard wall-clock
  timeout and byte caps on both streams. Exceeding either **kills and reaps
  the child** and reports `timeout` / `output_limit_exceeded` (exit code 7,
  `LIMIT_EXCEEDED`). Defaults are the amari probe bounds: 5 s, 1 MiB stdout,
  256 KiB stderr.
- **Isolation** (the amari probe blueprint): direct exec (no shell), a
  cleared environment (only `PATH` inherited, plus explicit additions), and a
  configurable working directory defaulting to the system temp dir (neutral
  cwd).
- **Availability**: a tri-state probe (`Ready` / `Missing` / `NotExecutable`
  + reason), the amari `capabilities` pattern; `invoke` on an unavailable
  tool reports kind `unavailable` without spawning.

## Relationship to the v0 provider spec

The v0 spec's *provider* model (one executable → many tools via `manifest` /
`tools list` / `tools describe` / `call` subcommands) layers on top of this
protocol as a future `SubprocessProvider`: discovery commands construct
`SubprocessTool`s whose argv prefix is `call <tool>`. Notably, `lonis` itself
(`tools list` / `tools describe` / `call`, blocks out / `ToolError` on
stderr) is already close to a conforming provider — the harness hosting
itself through the seam is the dogfood. The v0 spec's `manifest`/`status`/
`doctor` diagnostics and artifact-handling sections remain future work.

## Consequences

- `lonis-core` gains `subprocess` (public: `SubprocessTool`, `StdoutMapping`,
  `Availability`) and a `mock_tool` example binary that speaks the protocol
  for tests (and serves as the reference implementation for tool authors).
- Invocation is synchronous with a 10 ms `try_wait` poll — adequate for CLI
  tools; an async/streaming variant (`stream mode` in the v0 spec) is a
  named future extension over `Vec<Block>`, per ADR-0001.
- 20 new tests: unit (parse shapes, availability, failure mapping) and
  integration through a real process boundary (blocks/ndjson/text modes,
  `Extension` seam, structured stderr propagation, timeout kill, output-cap
  kill, missing binary, invalid output). 90 workspace tests green; matrix
  clean.
