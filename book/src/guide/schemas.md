# Schema Reference and Validation

The block contract's normative, machine-checkable form is **16 curated
draft-2020-12 JSON Schemas**, checked into
[`crates/lonis-schema/schemas/`](https://github.com/Industrial-Algebra/Lonis/tree/develop/crates/lonis-schema/schemas)
and emitted by the CLI.

```sh
lonis schema            # catalog: 16 families with their $ids
lonis schema block      # the envelope (payload = oneOf 14 kinds + extension)
lonis schema message    # one kind's payload schema
lonis schema extension  # the erased seam itself
```

## Conventions

- `$id`s mirror `BlockKind::schema_id()`:
  `https://industrialalgebra.com/schemas/lonis.block/<kind>/v1`
- `additionalProperties: false` throughout — mirrors serde's
  `deny_unknown_fields`
- `maxItems`/`maxLength` bounds on every collection and string — the
  doctrine's "bounded" property expressed *in the wire contract itself*
- The `extension` branch admits any kind tag **not** in the seed enum, so
  vertical payloads validate against the envelope (structure here; data
  shape against the vertical's own schema)

## The two-way pin

Schemas and golden fixtures validate each other in CI
(`crates/lonis-schema/tests/schemas.rs`):

- golden block instances (one per kind, produced from real Rust values)
  parse as `SeedBlock` and re-serialize identically — the **wire-shape
  pin**;
- the same instances validate against the envelope *and* their kind
  schema — **serde↔schema consistency**;
- each block's `content_hash` is pinned in `hashes.json` — the
  **canonicalization pin**.

## Regenerating

```sh
# After an intentional per-kind schema change:
cargo run -p lonis-schema --example compose_block_schema

# After an intentional wire-shape change:
cargo run -p lonis-schema --example dump_golden_blocks
```

Both leave an auditable diff that a reviewer can judge.
