# ADR-0007: Canonicalization policy for content hashes

- Status: accepted
- Date: 2026-08-10
- Branch: `feature/canonical-hash-hardening`
- References: ADR-0001 §5 (content hashing), ADR-0005 (golden pins); the
  PR 6+ hardening backlog (canonicalization edges)

## Context

`Block::content_hash()` / `json_content_hash()` hash SHA-256 over canonical
JSON. As shipped, canonicalization was key-sorting only — so `100`, `100.0`,
and `1e2` hashed differently. Within one producer that's harmless (the same
code serializes both write and verify), but replay pins
(`input_hash`/`plan_hash`/…) are meant to be *checked by other parties*,
potentially non-Rust producers (the authoring guide explicitly targets
them). JSON's number syntax makes "same value, different bytes" routine
across languages: Python's `json.dumps(100.0)` vs Rust's `100`.

## Decision

Canonicalization is **key-sorting + number normalization**:

1. Object keys recursively sorted (as before).
2. Integral floats collapse to integers: `100.0`, `1e2` → `100`, within the
   exactly-representable range (|n| ≤ 2^53).
3. Negative zero collapses to zero (JCS behavior).
4. Non-integral floats keep serde_json's shortest-round-trip (ryu)
   rendering; integers are exact at any width (no f64 round-trip).

**Residual limitation (documented on `json_content_hash`):** producers
emitting exotic float spellings (`0.30000000000000004` vs `0.3`) still hash
differently — full RFC 8785 (JCS) number formatting is out of scope until a
cross-language consumer needs it. Guidance: hash-critical values should be
integers or strings when cross-producer equality matters.

## Consequences

- The `evidence` golden's hash pin changed (`weight: 1.0` now hashes as `1`)
  — regenerated via `dump_golden_blocks`, an auditable one-line diff; the
  pin system working as intended on a deliberate change.
- Five new tests pin the policy: number-form equivalence, negative zero,
  non-integral stability, >2^53 integer exactness, nested normalization.
- No API change; `content_hash` values from before this change are
  superseded (pre-release, no consumers with stored hashes).
