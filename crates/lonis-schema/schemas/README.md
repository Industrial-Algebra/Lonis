# Curated block-contract JSON Schemas

Draft 2020-12 JSON Schemas for the `lonis.block/v1` contract (ADR-0005):
`block-v1.json` is the envelope (payload = `oneOf` all 14 seed kinds);
`<kind>-v1.json` describes one kind's `{"kind", "data"}` payload.

- **Per-kind files are the source of truth** for payload shapes.
- **`block-v1.json` is composed**: `python3 compose_block_schema.py` rebuilds
  it from the per-kind files (hoisting nested `$defs` like `plan_step` /
  `evidence_data`, with collision detection).
- `$id`s mirror `BlockKind::schema_id()`:
  `https://industrialalgebra.com/schemas/lonis.block/<kind>/v1`.
- Bounds are first-class in the wire contract: `additionalProperties: false`
  throughout (mirrors serde `deny_unknown_fields`), `maxItems`/`maxLength`
  on every collection/string.
- Golden instances under `../tests/golden/blocks/` validate against these
  schemas (and vice versa) in `../tests/schemas.rs`; regenerate goldens with
  `cargo run -p lonis-schema --example dump_golden_blocks` after an
  *intentional* wire change.
