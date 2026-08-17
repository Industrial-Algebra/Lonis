# Lonis

An **AI-native tool harness** for the
[Anima](https://github.com/Industrial-Algebra) ecosystem — a local-first,
machine-readable alternative to MCP for exposing sharply bounded tool surfaces
to agents. *"MCP exposes servers to models; Lonis exposes tools to agents."*

**Documentation**: [lonis-tooling.netlify.app](https://lonis-tooling.netlify.app)
(mdBook, deployed on release tags).

> **Status:** the harness runtime has landed. The original Python
> bitmap-vision analyzer has been spun out to
> [Perceptron](https://github.com/Industrial-Algebra/Perceptron)
> (tagged here as `lonis-python-legacy-v0.1.0`). Primary consumer is
> agents/LLMs, not humans.

## Workspace

- **`crates/lonis`** — the umbrella facade: one crate for consumers,
  re-exporting schema + derive + core (serde-style).
- **`crates/lonis-schema`** — the contract layer: the `Block<P>` structured
  domain object (doctrine §2.7) with the 14-kind seed corpus, attribution,
  bounds, replay provenance, content hashing; `Capabilities` /
  `ToolContract`; structured `ToolError`.
- **`crates/lonis-derive`** — `#[derive(LonisCapabilities)]` from
  `#[lonis(tool_id = "...")]`.
- **`crates/lonis-core`** — the harness runtime: `Tool<P>` /
  `ToolRegistry<P>`, per-mode rendering, `run_tool` (stdout blocks /
  stderr errors), and `SubprocessTool` (bounded external CLI adapter).
- **`crates/lonis-cli`** — the `lonis` binary: `tools list`/`describe`,
  `call <id> [input | @file]`, `--mode human|json|ndjson`.

## ADRs

- [`docs/adr/0001-block-contract.md`](docs/adr/0001-block-contract.md) — the `Block` contract; `invoke → Vec<Block>`; extraction direction (amari-discovery deletes into lonis-schema).
- [`docs/adr/0002-typed-block-payloads.md`](docs/adr/0002-typed-block-payloads.md) — `Block<P: BlockPayload>`; verticals get fully-typed registries, erasure only at the subprocess seam.
- [`docs/adr/0003-subprocess-tool-protocol.md`](docs/adr/0003-subprocess-tool-protocol.md) — the external CLI wire protocol (stdin JSON in / blocks out / structured errors on stderr / bounded + isolated).

## Design docs

- [`docs/plans/lonis-harness-design-draft.md`](docs/plans/lonis-harness-design-draft.md) — the overarching harness design.
- [`docs/plans/lonis-schema-design.md`](docs/plans/lonis-schema-design.md) — the contract layer.
- [`docs/plans/lonis-provider-interface-spec-v0.md`](docs/plans/lonis-provider-interface-spec-v0.md),
  [`lonis-external-provider-protocol-spec-v0.md`](docs/plans/lonis-external-provider-protocol-spec-v0.md) — provider model.
- [`docs/lonis-legacy-and-future-context.md`](docs/lonis-legacy-and-future-context.md) — the Lonis/Perceptron split rationale.

## License

Apache-2.0.
