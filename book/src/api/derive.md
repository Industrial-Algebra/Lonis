# Derive Macros

From `lonis-derive`, re-exported by `lonis-schema` behind its `derive`
feature (and by the facade behind `derive`).

## `#[derive(LonisCapabilities)]`

Generates the five-method `Capabilities` impl from one attribute:

```rust
#[derive(LonisCapabilities)]
#[lonis(tool_id = "amari:discovery:search")]
struct SearchTool;
```

Defaults: current protocol `SchemaVersion`, all three output modes, the
baseline exit-code map, `CARGO_PKG_VERSION`.

## `#[derive(BlockPayload)]`

Generates, from one enum declaration ([ADR-0004](https://github.com/Industrial-Algebra/Lonis/blob/develop/docs/adr/0004-vertical-payload-authoring.md)):

- adjacently-tagged `{"kind", "data"}` serde impls,
- `kind_name()` — snake_case variant names, dot-namespaced via
  `#[lonis_payload(namespace = "karpal")]` → `karpal.search`,
- `schema_id()` — `lonis.block/<kind>/v1`,
- `render_human()` — a `<kind>: <Debug>` default, or a hook:
  `render_fn = "path::to::render"` (`fn(&Self) -> String`).

Supports struct variants (named fields) and unit variants (`data: null`).
Tuple variants and generics are compile errors. Unknown kinds are rejected
on the vertical's own enum (closed universe); cross-vertical tolerance
lives host-side in `BlockKind::Extension`.

Consumers need `serde` and `serde_json` as dependencies; the enum must
implement `Debug` when using the default render.
