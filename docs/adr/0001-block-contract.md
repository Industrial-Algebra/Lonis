# ADR-0001: The Block contract and the tool output type

- Status: accepted
- Date: 2026-08-10
- Branch: `feature/block-contract`
- References: Anima Ecosystem Doctrine §2.7 (block taxonomy, resolved 2026-08-10);
  `docs/handoff/2026-08-10-block-contract-and-harness.md`;
  `amari-discovery` (`src/protocol.rs`, `schemas/`, `src/probes/supervisor.rs`)

## Context

The doctrine resolved what a structured domain object *is*: a **block**
carrying six cross-cutting properties (envelope, attribution, bounded,
versioned, replayable, render-parity) and a 14-kind seed corpus in three
categories. Lonis is the horizontal generalization of the machinery
amari-discovery validates vertically — and, per the stated architectural
direction, **Lonis replaces, generalizes, and extends amari-discovery's
machinery; when Lonis matures, amari-discovery will be refactored to depend on
it.** Reuse from amari-discovery is extraction, not redundancy.

## Decisions

### 1. `Block` is the canonical tool output; `Tool::invoke` returns `Vec<Block>`

`Tool::invoke` will return `Result<Vec<Block>, ToolError>` (implemented in the
follow-up `feature/tool-block-output` PR). Rationale:

- 0/1/N outputs with one type: a tool may emit a single `result`, or a
  `message` + `evidence` + `result` triple, or nothing.
- `--ndjson` maps to one line per block — direct parity with amari-discovery's
  streaming mode.
- The doctrine's "the transcript decomposes into blocks" requires multiplicity.

Alternatives rejected: a single `Block` (forces a wrapper kind for multi-block
output); a streaming iterator (right long-term for long-running tools, but
premature — can be layered over `Vec<Block>` later); keeping
`Envelope<serde_json::Value>` as the output with blocks orthogonal (weakens
the thesis that the block is what any tool emits through).

### 2. The block shape is flat; the envelope's `data` *is* the payload

```json
{
  "schema_version": "lonis.block/v1",
  "provenance": { "replay": {...}, "input_hash": "...", "seed": 7 },
  "warnings": [],
  "attribution": { "identity": "...", "viewpoint": "...",
                   "provenance": { "when": "...", "where": "...", "producer": "..." } },
  "bounds": { "max_items": 64 },
  "payload": { "kind": "message", "data": { ... } }
}
```

The doctrine lists the envelope as `{schema_version, provenance, warnings,
data}`; its `data` is the block payload, so the wire form is flat rather than
nesting `payload` inside an `envelope` object. `ReplayProvenance` is a
deliberate superset of amari-discovery's `Provenance` (adds `plan_hash`,
`result_hash` slots) so the vertical can delete its `protocol.rs` into
`lonis-schema`.

Attribution keeps the full doctrine slot (`identity`, `viewpoint`,
`{when, where, producer}`) — richer than amari-discovery's `capability_id`
collapse, because Lonis transcripts are multi-participant and stream-scoped.
The Rust field is `location`, renamed to `where` on the wire.

### 3. Extensibility: unknown kinds degrade to `Extension` losslessly

`BlockKind` is manually serialized as `{"kind", "data"}`. Unknown kinds
deserialize to `BlockKind::Extension { kind, data }` and re-serialize
identically, so older consumers tolerate newer domain kinds. Internally-tagged
serde derives cannot express the catch-all, hence the manual impls.

`PlanStep` is deliberately open (`kind` + `detail`): the horizontal contract
cannot enumerate every domain's step vocabulary. Verticals keep typed step
unions (e.g. amari-discovery's six-variant `PlanStep`) inside `detail`.

Payload field sets are informed by the ecosystem: `Definition` generalizes
karpal-index's `ApiItem` (semantic overlay slot), `Outcome` generalizes
amari's `DiscoveryOutcome` + schubert's `AccessDecision` (quantitative,
exhaustive, non-boolean) and lifts `ToolError`'s `kind/message/details/
exit_code` into the stream, `ResultPayload` mirrors amari's scored +
evidenced + assumption-bookkeeping result shape, and `Capability` reuses
`ToolContract` directly.

### 4. `SchemaVersion` is a namespaced string protocol marker

`SchemaVersion` changes from `SchemaVersion(u32)` to a validated
`<name>/v<N>` string (`lonis.envelope/v1` default, `lonis.block/v1` for
blocks). This is required by the extraction direction: a bare `u32` could
never carry `amari.discovery/v1`, so amari-discovery could not later adopt
`lonis-schema`. Breaking change; acceptable pre-release.

Each kind has a stable `$id` (`lonis.block/<kind>/v1`), mirroring
amari-discovery's per-kind schema ids. Emitting draft-2020-12 JSON Schemas per
kind (like amari's `schemas/` with `additionalProperties: false` and
`maxItems` bounds) is deferred to a CLI `lonis schema <kind>` command.

### 5. Replay: canonical-JSON content hash + seed

`Block::content_hash()` = SHA-256 over the canonical payload JSON (kind tag +
recursively key-sorted data), so semantically equal payloads hash identically
regardless of map insertion order. Combined with `seed` and the typed hash
slots in `ReplayProvenance`, this is the deterministic-replay pin, mirroring
amari-discovery's `replay {replayable, required_hashes, reasons}` discipline.

### 6. The multivector is the formal composition model, not a wire type

`amari-core`'s `Multivector<P,Q,R>` is a dense `f64` Clifford-algebra compute
type and cannot carry typed block payloads; it is **not** the output
container. But the block taxonomy is algebra-shaped: the three categories are
grades, the 14 kinds are basis vectors, and the geometric product models
composition (`message ∧ evidence` = substantiated claim, `plan ∧ result` =
execution record). `Vec<Block>` *is* the graded sum structurally. A future
additive `geometric` feature will aggregate block streams into real
`Multivector` embeddings for amari-holographic recall — an analysis index
over blocks, never a replacement for them.

## Forward guidance (for the follow-up PRs)

- **Subprocess tools** (`feature/subprocess-tool`): copy amari-discovery's
  supervisor/worker probe isolation — re-exec with a hidden worker subcommand,
  `env_clear()`, neutral cwd, piped stdio with byte caps, hard timeout, crash
  isolation — generalized from "probes" to "external CLI tools". Declare
  `isolation`/`hard_timeout`/`crash_isolation` in tool contracts, and report
  availability as the tri-state `known`/`available`/`executable` + reason.
- **Exit codes**: lonis's vocabulary now includes amari-aligned
  `TOOL_FAILED=6`, `LIMIT_EXCEEDED=7`, `IO=8`, `SERIALIZATION=9`,
  `NOT_IMPLEMENTED=69`, `INTERNAL=70` (lonis keeps `RATE_LIMITED=5` where
  amari uses 5 for `probe_unavailable`; the lonis codes are the baseline map
  tools extend).
- **CLI input**: `lonis call` should accept inline JSON *and* `@file`
  explicitly, and document both (amari's `--input` silently stats the value
  as a path — a DX bug we observed live).
- **Policy**: when `lonis-policy` un-defers, delegate to schubert
  (ecosystem-first): tool authorization = access check; intersection-number
  rate multipliers > flat limits. `crypto.rs` Ed25519 tokens are the future
  verifiable `Attribution.identity`. A `BlockSink` trait (schubert `AuditSink`
  pattern: append-only, async, failing sink never blocks) is the block-stream
  audit trail.

## Consequences

- `lonis-schema` gains `sha2` as its only new dependency (content hashing).
- Pre-release breaking change to `SchemaVersion` (u32 → namespaced string);
  `Envelope`'s wire `schema_version` is now `"lonis.envelope/v1"`.
- 27 new tests; all public items documented; clippy/fmt/doc clean.
