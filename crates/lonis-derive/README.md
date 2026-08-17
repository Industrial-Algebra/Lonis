# lonis-derive

Procedural macros for [Lonis](https://github.com/Industrial-Algebra/Lonis) tools.

## `LonisCapabilities`

Derives the [`Capabilities`](../lonis-schema) self-description trait for a tool
type from a `#[lonis(tool_id = "...")]` attribute — design decision #3
("traits + `#[lonis::tool]` derive"). It removes the five-method `Capabilities`
boilerplate every tool otherwise repeats.

```rust
use lonis_schema::{Capabilities, LonisCapabilities};

#[derive(LonisCapabilities)]
#[lonis(tool_id = "amari:discovery:search")]
struct SearchTool;

// SearchTool now implements Capabilities:
assert_eq!(SearchTool.tool_id().as_str(), "amari:discovery:search");
assert_eq!(SearchTool.schema_version().get(), 1);
```

The derived impl uses sensible defaults: schema version 1, all three output
modes (`human`/`json`/`ndjson`), the baseline exit-code map, and the tool
crate's `CARGO_PKG_VERSION`. The tool still writes its own `impl Tool` (the
`invoke` body can't be derived).

Enable it on `lonis-schema`:

```toml
lonis-schema = { version = "0.0.1", features = ["derive"] }
```
