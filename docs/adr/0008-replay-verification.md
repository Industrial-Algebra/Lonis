# ADR-0008: Replay verification helper

- Status: accepted
- Date: 2026-08-10
- Branch: `feature/replay-verification`
- References: ADR-0001 §5 (replay: content hash + seed); amari-discovery's
  `replay {replayable, required_hashes, reasons}` discipline; the PR 6+
  hardening backlog

## Context

ADR-0001 made blocks *replayable by declaration*: `ReplayProvenance` pins
hashes and `ReplayMetadata` declares `replayable` + `required_hashes`. But
nothing *checked* a replay — the consumer side of the contract existed only
as a promise. amari-discovery's `plan` command shows what the check looks
like in a vertical (`--recommendation … --project …` re-validates
`project_hash`/`input_hash`); Lonis, as the horizontal layer, should supply
the check itself.

## Decision

`lonis_schema::verify_replay(block, observed) -> ReplayStatus`:

- `NotReplayable { reasons }` — the producer declared it non-replayable
  (reasons pass through verbatim; e.g. environment-specific state).
- `Replayable` — `replayable` is set and every `required_hashes` field is
  present on both sides and equal (including the trivial case of no
  requirements).
- `Failed { missing, mismatches }` — one combined failure report: `missing`
  for required fields absent on the block, unobserved, or unknown
  (forward-tolerant: a v2 hash field on an older consumer fails closed, not
  silently open), and `mismatches` carrying `(field, expected, observed)`
  triples for unequal values.

`ObservedHashes` mirrors the four typed hash slots (`project`, `input`,
`plan`, `result`) — the same superset ADR-0001 chose for
`ReplayProvenance`, keeping the amari-discovery extraction mapping 1:1.

Deliberately a pure function over data: *how* the observed hashes are
recomputed (re-inspecting a project, re-canonicalizing input) is the
vertical's business; the horizontal contract only adjudicates equality.

## Consequences

- The replay property is now enforced *somewhere* in the horizontal layer,
  not merely declared: verticals can gate plan replay on one call.
- 11 unit tests pin the semantics (match, no-requirements, not-replayable,
  mismatch detail, missing on either side, unknown field, accumulation).
- Future work named, not built: an `ObservedHashes` computed from a
  re-serialized input (`json_content_hash`) is a one-liner convenience that
  can land when a consumer needs it.
