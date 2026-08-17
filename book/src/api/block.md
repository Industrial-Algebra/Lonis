# Blocks and Payloads

From `lonis-schema` (re-exported by the `lonis` facade).

## `Block<P: BlockPayload>`

The canonical structured domain object. Flat wire shape:
`schema_version`, `provenance`, `warnings`, `attribution`, `bounds`,
`payload`. Key methods:

- `Block::new(attribution, payload)` — at the v1 contract
- `with_provenance` / `with_warnings` / `with_bounds` — builders
- `schema_id()` — the payload kind's stable `$id`
- `content_hash()` — SHA-256 over canonical payload JSON
- `render_human()` — render-parity human form

## `BlockPayload`

The trait verticals implement (usually via the derive):

```rust
fn kind_name(&self) -> &str;
fn schema_id(&self) -> String;
fn render_human(&self) -> String;
```

## `BlockKind` / `SeedBlock`

The 14-kind seed payload enum (implementing `BlockPayload`) plus the
`Extension { kind, data }` catch-all. `SeedBlock = Block<BlockKind>` is the
umbrella host's type.

## Supporting types

- `Attribution { identity, viewpoint, provenance: AttributionSource { when, where, producer } }`
  — `Attribution::new(identity, producer)` stamps RFC 3339 UTC.
- `BlockBounds { max_items, max_bytes, max_length, timeout_millis }` — all
  optional; a default (unbounded) set is omitted from the wire.
- `ReplayProvenance` — tool version, compatibility, replay metadata, the
  four typed hash slots, seed.
- `verify_replay(block, observed) -> ReplayStatus` — replay verification
  (see [Replay and Content Hashing](../concepts/replay.md)).
- `json_content_hash(&Value)` — canonical hashing for arbitrary JSON
  (e.g. pinning `input_hash`).

## Errors and identifiers

`ToolError { kind, message, details?, exit_code }` ·
`ToolId(<tool>:<namespace>:<item>)` · `SchemaVersion(<name>/v<N>)` ·
`exit_code` baseline constants.
