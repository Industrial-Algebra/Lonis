# lonis-schema — Design

> **Status:** Decided (2026-07). The contract layer for Lonis-compatible tools,
> extracted/generalized from amari-discovery and Proserpina (which already
> implement ~90% of it). amari-discovery ships independently; Lonis becomes its
> dependency once stable. See
> [`lonis-harness-design-draft.md`](lonis-harness-design-draft.md) for the
> overarching harness design.

## 1. Role

`lonis-schema` is the shareable foundation every Lonis tool / composable CLI
depends on: the canonical success envelope, output modes, structured tool
errors, the capabilities self-description trait, and tool contracts. Tools emit
these so the harness (`lonis-core` / `lonis-cli`) and agents can discover,
invoke, and parse them uniformly. The composable tools — amari-discovery,
karpal-discovery, schubert-discovery, Hotaru, Proserpina, Sakamoto — depend on
`lonis-schema` (or conform to it) and become Lonis-compatible providers.

## 2. Extraction source

amari-discovery (design doc in the Amari repo) already implements: `Envelope<T>`,
`Capabilities::envelope()`, `OutputMode` (human / json / ndjson), structured
errors (`kind` / `message` / `details` + `exit_code`), a versioned protocol
(`amari.discovery/v1`), namespaced ids (`amari:<crate>:<module>:<capability>`),
probe/tool contracts, and the `stdout = data / stderr = diagnostics`
discipline. Proserpina contributes the `capabilities` / auth / exit-code
conventions. `lonis-schema` generalizes these.

## 3. Decisions (locked)

1. **Envelope error model — amari split.** Success `Envelope<T>` on **stdout**
   (always parseable as the result type); structured `ToolError`
   (`kind` / `message` / `details` + `exit_code`) on **stderr**. Not the
   unified `{ok, result|error}` ok-velope — keeps stdout cleanly typed for
   piping / streaming.
2. **Capabilities — trait-based.** `lonis-schema` provides a base
   `Capabilities` trait; tools extend with domain states (amari's
   known / available / executable).
3. **Schema declaration — traits + `#[lonis::tool]` derive.** Typed Rust
   contracts plus a derive that emits a machine-readable manifest (planned;
   amari's typed-Rust + schema-export validates the approach).
4. **Tool ids — universal `<tool>:<namespace>:<item>`.** Generalized from
   amari's `amari:<crate>:<module>:<capability>`.
5. **NDJSON streaming — first-class.** `OutputMode::Ndjson` streams
   independently-parseable envelopes.

## 4. Types (v0.0.1)

| Type | Role |
|---|---|
| `Envelope<T>` | `{ schema_version, tool: ToolId, result: T, meta: Meta }` on stdout |
| `Meta` | `{ duration_ms?, warnings, seed?, hashes }` |
| `OutputMode` | `Human` (default) / `Json` / `Ndjson` |
| `ToolError` | `{ kind, message, details?, exit_code }` on stderr; impl `Display` + `Error` |
| `exit_code` | shared vocabulary (`SUCCESS` / `GENERIC` / `INVALID_INPUT` / `NOT_FOUND` / `CONFIRMATION_REQUIRED` / `RATE_LIMITED`); tools extend |
| `Capabilities` | trait: `schema_version`, `tool_version`, `output_formats`, `exit_code_map`, `tool_id` |
| `ToolContract` | `{ name, description, input_schema, output_schema, determinism, side_effects, cost, capabilities }` |
| `Determinism` / `SideEffects` / `Cost` | contract enums |
| `SchemaVersion` / `ToolId` / `SchemaRef` | newtypes |

## 5. Migration

1. Lift amari-discovery's `Envelope` / `Capabilities` / error types into
   `lonis-schema`; amari-discovery depends on it and extends with its domain
   layer.
2. karpal-discovery / schubert-discovery adopt the same crate.
3. Hotaru / Proserpina / Sakamoto conform (emit the envelope + tool contracts).
4. Add the `#[lonis::tool]` derive (a `lonis-derive` crate) once the types
   stabilize.

## 6. Out of scope (v0.0.1)

The harness runtime (`lonis-core` / `lonis-cli` / `lonis-registry` /
`lonis-runtime`), policy, adapters, the Figma surface, provider transports, and
the derive macro. This crate is the contract types only.
