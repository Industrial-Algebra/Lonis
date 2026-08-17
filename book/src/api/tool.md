# Tool and ToolRegistry

From `lonis-core`.

## `Capabilities`

Self-description — every tool implements it:

```rust
fn schema_version(&self) -> SchemaVersion;
fn tool_version(&self) -> &str;
fn output_formats(&self) -> &'static [OutputMode];
fn exit_code_map(&self) -> &'static [(&'static str, u8)];
fn tool_id(&self) -> ToolId;
```

Usually derived: `#[derive(LonisCapabilities)]` with
`#[lonis(tool_id = "...")]`.

## `Tool<P: BlockPayload>`

```rust
fn invoke(&self, input: Value) -> Result<Vec<Block<P>>, ToolError>;
fn invoke_stream(&self, input: Value) -> Result<BlockStream<P>, ToolError> { /* default */ }
fn contract(&self) -> Option<ToolContract> { None }
```

`invoke_stream` defaults to collect-then-stream; tools with genuine
incremental output override it.

## `ToolRegistry<P>`

In-process registry, homogeneous in the payload type (a vertical's registry
is fully typed). Deterministic order (sorted by id).

- `register(Box<dyn Tool<P>>)` — `already_registered` on duplicates
- `get(id)` / `iter()` / `len()` / `is_empty()`
- `invoke(id, input)` / `invoke_stream(id, input)` — `not_found` (exit 3)
  on unknown ids

## Rendering

- `render(&[Block<P>], mode, writer)` — json array / ndjson lines / human
- `render_error(&ToolError, mode, writer)`
- `run_tool(registry, id, input, mode) -> u8` — the amari split: blocks to
  stdout, structured error to stderr with its exit code
- `run_stream(registry, id, input, mode) -> u8` — the streaming variant
