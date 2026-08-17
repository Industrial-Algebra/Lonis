# Getting Started

## Install

```sh
cargo install lonis-cli   # the `lonis` binary
```

Or from source:

```sh
git clone https://github.com/Industrial-Algebra/Lonis
cd Lonis && cargo install --path crates/lonis-cli
```

## Five minutes with the harness

```sh
# What tools are registered?
lonis tools list

# Describe one (its ToolContract)
lonis tools describe lonis:builtin:echo

# Invoke it — inline JSON, a @file, or stdin
lonis call lonis:builtin:echo '{"hello": "world"}' --mode json
```

Every invocation follows the same split: **blocks on stdout**, structured
errors on stderr with a stable exit code.

```json
[
  {
    "schema_version": "lonis.block/v1",
    "provenance": { "tool_version": "0.1.0", "input_hash": "…" },
    "attribution": {
      "identity": "lonis:builtin:echo",
      "provenance": { "when": "2026-08-10T12:00:00Z", "producer": "lonis:builtin:echo" }
    },
    "payload": { "kind": "result", "data": { "output": { "hello": "world" } } }
  }
]
```

## The normative contract

```sh
lonis schema              # list all 16 schema families
lonis schema block        # the envelope (all kinds + the extension seam)
lonis schema message      # one kind's payload schema
```

Validate anything you emit against these — they're the same documents the
golden fixtures validate against in CI.

## As a library

```toml
[dependencies]
lonis = "0.1"   # the facade: schema + derive + core
```

```rust
use lonis::{Block, Capabilities, Tool, ToolRegistry};
```

See [The Block Contract](./concepts/block-contract.md) next.
