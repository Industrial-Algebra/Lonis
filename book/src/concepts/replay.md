# Replay and Content Hashing

Decided in
[ADR-0001 §5](https://github.com/Industrial-Algebra/Lonis/blob/develop/docs/adr/0001-block-contract.md)
(content hashing), [ADR-0007](https://github.com/Industrial-Algebra/Lonis/blob/develop/docs/adr/0007-canonicalization-policy.md)
(canonicalization policy), and [ADR-0008](https://github.com/Industrial-Algebra/Lonis/blob/develop/docs/adr/0008-replay-verification.md)
(verification).

## Pins

Blocks carry replay provenance: typed hash slots (`project_hash`,
`input_hash`, `plan_hash`, `result_hash`), an optional `seed`, and a
`replay { replayable, required_hashes, reasons }` declaration.

`Block::content_hash()` is SHA-256 over the **canonical** payload JSON:
object keys recursively sorted, and numbers normalized — integral floats
collapse to integers (`100.0`, `1e2` → `100`), negative zero to zero.
Semantically equal payloads hash identically across producers and
languages.

> Residual limitation: exotic float spellings (`0.30000000000000004` vs
> `0.3`) still hash differently. Hash-critical values should be integers or
> strings when cross-producer equality matters.

## Verification

`verify_replay(block, observed) -> ReplayStatus` adjudicates a block's pins
against the hashes observed now:

- `NotReplayable { reasons }` — the producer declared it (e.g.
  environment-specific state);
- `Replayable` — every required hash is present and equal;
- `Failed { missing, mismatches }` — one combined report; unknown required
  field names **fail closed** (a v2 hash field can't silently pass an older
  consumer).

Recomputing the observed hashes (re-inspecting a project, re-canonicalizing
an input with `json_content_hash`) is the vertical's business; the
horizontal contract adjudicates equality.

## Golden pins

The wire shape and the canonicalization are pinned by checked-in golden
instances and hashes (`tests/golden/blocks/`) — a drift breaks CI, not a
consumer. See [Schema Reference and Validation](../guide/schemas.md).
