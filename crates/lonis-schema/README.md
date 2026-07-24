# lonis-schema

The shared contract layer for
[Lonis](https://github.com/Industrial-Algebra/Lonis)-compatible tools: the
canonical success envelope, output modes, structured tool errors, the
capabilities self-description trait, and tool contracts.

See [`docs/plans/lonis-schema-design.md`](../../docs/plans/lonis-schema-design.md)
for the design decisions.

## Types

- `Envelope<T>` — success result + `Meta` on stdout (errors go to stderr).
- `OutputMode` — `Human` (default) / `Json` / `Ndjson` (streaming).
- `ToolError` — `{ kind, message, details, exit_code }`.
- `Capabilities` — self-description trait; tools extend with domain states.
- `ToolContract` — namespaced id, schemas, determinism, side-effects, cost.
- `ToolId` — universal `<tool>:<namespace>:<item>` naming.

## License

Apache-2.0.
