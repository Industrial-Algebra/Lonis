# The Block Contract

A **block** is the canonical structured domain object — what every Lonis
tool emits through. Defined by the Anima doctrine §2.7; decided in
[ADR-0001](https://github.com/Industrial-Algebra/Lonis/blob/develop/docs/adr/0001-block-contract.md).

## Six cross-cutting properties

Every block carries all six:

| Property | Where | Meaning |
|---|---|---|
| **Envelope** | `schema_version`, `provenance`, `warnings` | Versioned, self-describing frame |
| **Attribution** | `attribution` | Who (`identity`), under what lens (`viewpoint`), and `when`/`where`/`producer` |
| **Bounded** | `bounds` | Resource limits (`max_items`/`max_bytes`/`max_length`/`timeout_millis`) are first-class |
| **Versioned** | `schema_version` + per-kind `$id` | `lonis.block/v1` + `lonis.block/<kind>/v1` |
| **Replayable** | `provenance` hashes + `seed` | Canonical content hashes → deterministic replay |
| **Render-parity** | `render_human()` | Human and machine render from the *same typed value* — drift is structurally impossible |

## The wire shape

Flat — the envelope's `data` *is* the payload, not a wrapper:

```json
{
  "schema_version": "lonis.block/v1",
  "provenance": { "replay": { "replayable": true }, "input_hash": "…", "seed": 7 },
  "warnings": [],
  "attribution": {
    "identity": "dominic",
    "viewpoint": "reviewer",
    "provenance": { "when": "…", "where": "session:abc", "producer": "lonis:test" }
  },
  "bounds": { "max_items": 64 },
  "payload": { "kind": "message", "data": { "content": "hello" } }
}
```

Unknown top-level fields are **rejected** (`deny_unknown_fields` everywhere).

## The 14 seed kinds

Three categories, plus the open seam:

- **Participant-stream** (7): `message`, `question`, `answer`, `decision`,
  `action`, `assumption`, `summary` — what a transcript decomposes into.
- **Knowledge/definition** (4): `evidence`, `definition`, `capability`,
  `intent` — the "what is" set.
- **Process** (3): `plan`, `result`, `outcome` — the "how / what-happened"
  set. `outcome` covers both structured domain results and structured
  errors (`kind`/`message`/`details`/`exit_code`).
- **Extension** — any other kind tag, carried losslessly
  (`Extension { kind, data }`). This is how verticals and future kinds
  cross the wire without breaking older consumers.

## The normative form

The contract's machine-checkable form is 16 curated draft-2020-12 JSON
Schemas (`lonis schema`), validated both directions against golden fixtures
in CI — see [Schema Reference and Validation](../guide/schemas.md).
