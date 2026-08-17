# Stream Mode

Decided in
[ADR-0009](https://github.com/Industrial-Algebra/Lonis/blob/develop/docs/adr/0009-stream-mode.md).

## The principle

**Async is a property of the session orchestrator, not of the tool
boundary.** A subprocess is bytes on a pipe — synchronous by physics.
Wallace-class hosts are async by nature, but that's the host layer. So
Lonis streams synchronously and bridges at the seam.

## The shape

- `BlockStream<P>` — a pull iterator of `Result<Block<P>, ToolError>`.
  `Tool::invoke_stream` defaults to collect-then-stream (object-safe: no
  `async fn` in the trait). A terminal failure is the stream's final item;
  blocks delivered before it are kept.
- `SubprocessTool` streams for real: ndjson stdout lines become blocks **as
  they arrive**; the supervisor enforces timeout and byte caps *concurrently
  with delivery*; stderr drains bounded for the terminal error mapping.
- **Backpressure is real**: a bounded channel means a slow consumer backs
  up the pipe, which throttles the child.
- **Dropping the stream kills the child** — an abandoned stream never
  leaves a running process.

## Async hosts

`BlockStream::into_async()` (feature `futures`) yields a runtime-agnostic
`futures_core::Stream`. The library never names a runtime; tokio is the
host's choice.

## For tool authors

Emit **one block per line (ndjson)** and **flush after each line** —
language runtimes block-buffer stdout on pipes (Rust's `println!`
included), so without an explicit flush your blocks arrive in one burst at
exit.

## CLI

```sh
lonis call <tool> '<input>' --stream --mode ndjson
```

ndjson/human render incrementally; `--mode json` buffers to one array (a
valid JSON document can't be emitted incrementally).
