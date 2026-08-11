# ADR-0005: Curated JSON Schema emission + golden wire fixtures

- Status: accepted
- Date: 2026-08-10
- Branch: `feature/schema-emission`
- References: ADR-0001 (deferred schema emission, "if useful" — it is);
  amari-discovery `src/schema.rs` + `schemas/`; the PR 6+ hardening backlog
  (golden fixtures, canonicalization pins)

## Context

ADR-0001 deferred emitting draft-2020-12 JSON Schemas per block kind. Two
forces made it timely: the gap review found the serde round-trip tests are
*self-consistent* (a wire-shape drift would stay green), and external tool
authors (per the subprocess guide) need a machine-checkable contract to emit
blocks in any language.

## Decision

- **Curated, checked-in JSON Schema files** (`crates/lonis-schema/schemas/`),
  embedded with `include_str!` and validated on load — the amari-discovery
  `schema.rs` registry pattern, generalized: one `<kind>-v1.json` per seed
  kind (14) plus `block-v1.json` (the envelope, whose `payload` is a `oneOf`
  over all kinds via `$defs`).
- **Per-kind files are the source of truth**; `block-v1.json` is *composed*
  from them by a checked-in script (`schemas/compose_block_schema.py`),
  hoisting nested `$defs` with collision detection. No dual authoring.
- `$id`s mirror the `schema_id()` convention:
  `https://industrialalgebra.com/schemas/lonis.block/<kind>/v1`; each
  document carries `x-lonis-protocol-version: lonis.block/v1`.
- **Bounds are first-class in the wire contract**: `additionalProperties:
  false` everywhere (mirroring serde's `deny_unknown_fields`),
  `maxItems`/`maxLength` on every collection and string — the doctrine's
  "bounded" property expressed in the schemas themselves, as amari does.
- **Golden fixtures pin the wire**: one checked-in block instance per kind
  (`tests/golden/blocks/<kind>.json`, produced by
  `examples/dump_golden_blocks.rs` from real Rust values) plus
  `hashes.json` pinning each block's `content_hash`. Tests assert:
  goldens parse as `SeedBlock` and re-serialize semantically identical
  (wire pin), validate against the envelope *and* their kind schema
  (schema↔serde consistency, via the `jsonschema` dev-dependency), and keep
  their pinned content hashes (canonicalization stability).
- **`lonis schema [kind]`** CLI: no kind lists the catalog (`kind` + `$id`
  lines); a kind emits the canonical pretty document; unknown kind exits 3.

## Consequences

- The wire shape can no longer drift silently: changing any field, kind tag,
  or canonicalization breaks a checked-in pin that requires deliberate
  regeneration.
- Vertical/external tool authors get a validator-usable contract per kind
  (the authoring guide's "minimal block JSON" now has a normative form).
- `lonis-schema` gains `jsonschema` as a dev-dependency only; the schemas
  are data, not code — zero runtime cost beyond the embedded text.
- Regeneration discipline: schemas after intentional wire changes
  (`compose_block_schema.py`), goldens after intentional shape changes
  (`dump_golden_blocks`); both leave an auditable diff.
