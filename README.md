# Lonis

An **AI-native tool harness** for the
[Anima](https://github.com/Industrial-Algebra) ecosystem — a local-first,
machine-readable alternative to MCP for exposing sharply bounded tool surfaces
to agents. *"MCP exposes servers to models; Lonis exposes tools to agents."*

> **Status:** pivoting. The original Python bitmap-vision analyzer has been
> spun out to [Perceptron](https://github.com/Industrial-Algebra/Perceptron)
> (tagged here as `lonis-python-legacy-v0.1.0`). This repository is becoming a
> Rust workspace hosting the harness and its shared contract layer. Primary
> consumer is agents/LLMs, not humans.

## Workspace

- **`crates/lonis-schema`** — the shared contract layer (envelope,
  capabilities, tool errors, tool contracts) every Lonis-compatible tool
  depends on. See
  [`docs/plans/lonis-schema-design.md`](docs/plans/lonis-schema-design.md).

## Design docs

- [`docs/plans/lonis-harness-design-draft.md`](docs/plans/lonis-harness-design-draft.md) — the overarching harness design.
- [`docs/plans/lonis-schema-design.md`](docs/plans/lonis-schema-design.md) — the contract layer.
- [`docs/plans/lonis-provider-interface-spec-v0.md`](docs/plans/lonis-provider-interface-spec-v0.md),
  [`lonis-external-provider-protocol-spec-v0.md`](docs/plans/lonis-external-provider-protocol-spec-v0.md) — provider model.
- [`docs/lonis-legacy-and-future-context.md`](docs/lonis-legacy-and-future-context.md) — the Lonis/Perceptron split rationale.

## License

Apache-2.0.
