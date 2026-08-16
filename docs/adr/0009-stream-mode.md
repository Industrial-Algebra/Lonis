# ADR-0009: Stream mode — sync pull core, async at the host

- Status: accepted
- Date: 2026-08-10
- Branch: `feature/stream-mode`
- References: ADR-0001 (deferred streaming), ADR-0002 (erasure lives at the
  boundary), ADR-0003 (the subprocess seam); operator decision (sync/async
  fork, resolved to the layered design)

## Context

ADR-0001's `Vec<Block>` answers single-shot invocation. Long-running tools —
and mid-term, Wallace's multi-participant sessions — need blocks as they're
produced. The open question was sync vs async. The operator's lean was async
(sessions are async by nature); the resolution came from applying ADR-0002's
seam principle to concurrency.

## Decision

**Async is a property of the session orchestrator, not of the tool
boundary.** A subprocess is bytes on a pipe — synchronous by physics. So:

1. **lonis-core: a synchronous pull primitive.** `BlockStream<P>` is an
   iterator of `Result<Block<P>, ToolError>`. `Tool::invoke_stream` has a
   default collect-then-stream impl (object-safe — no `async fn` in the
   trait, which would break `Box<dyn Tool<P>>`). A terminal failure is the
   stream's final item; blocks delivered before it are kept.
2. **`SubprocessTool` streams for real** (Blocks mapping): a parser thread
   turns ndjson stdout lines into blocks as they arrive; the supervisor
   thread enforces the same bounds as `exec_bounded` (timeout, byte caps,
   kill-and-reap) *concurrently with delivery*; stderr drains bounded for
   the terminal error mapping. Streaming subprocesses **emit ndjson** (one
   block per line, flushed — Rust stdout is block-buffered on pipes).
   `Text` mapping falls back to collect-then-stream.
3. **Backpressure is real**: the stream flows through a bounded
   `sync_channel(64)`; a slow consumer backs up the pipe, which throttles
   the child. Dropping the stream **kills the child** (`ChildGuard`) — an
   abandoned stream never leaves a running process.
4. **Async hosts bridge above the seam**: `BlockStream::into_async()`
   (feature `futures`, propagated through the facade) yields a
   runtime-agnostic `futures_channel` receiver implementing
   `futures_core::Stream`. The library never names a runtime; tokio is the
   host's (Wallace's) choice.
5. **CLI**: `lonis call --stream`. `run_stream` renders ndjson/human
   incrementally; json mode buffers to one array (a valid JSON document
   cannot be emitted incrementally).

## Consequences

- No runtime dependency for sync consumers; the default path (collect) costs
  nothing.
- The wire is unchanged: streaming is `Ndjson` mode with timing.
- The trait stays dyn-safe; the registry gains `invoke_stream`.
- 8 new tests (incremental arrival before child exit, error tail after
  partial output, timeout mid-stream, drop-kills-child, default
  collect-then-stream, the `futures` bridge, CLI `--stream`). 139 workspace
  tests green; matrix clean.
