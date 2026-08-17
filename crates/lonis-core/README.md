# lonis-core

Minimal harness runtime for [Lonis](https://github.com/Industrial-Algebra/Lonis) tools,
built on [`lonis-schema`](../lonis-schema).

## What it provides

- **`Tool` trait** — [`Capabilities`](../lonis-schema) (self-description) plus a
  uniform JSON-typed `invoke` boundary: tools take `serde_json::Value` and
  return `Envelope<Value>` (stdout) or a structured `ToolError` (stderr).
- **`ToolRegistry`** — an in-process registry: `register` / `get` / `iter` /
  `invoke`, keyed by tool id, deterministic iteration order.
- **`render` / `render_error`** — envelope/error rendering per `OutputMode`.
- **`run_tool`** — enforces the **amari split** (decision #1): success
  `Envelope` to stdout, structured `ToolError` to stderr with the tool's exit
  code.

## Example

```rust
use lonis_core::{run_tool, ToolRegistry};
use lonis_schema::OutputMode;

let mut registry = ToolRegistry::new();
// registry.register(Box::new(my_tool))?;

let exit_code = run_tool(&registry, "amari:discovery:search", input, OutputMode::Json);
std::process::exit(exit_code as i32);
```

See `docs/plans/lonis-schema-design.md` for the contract this runtime enforces.
