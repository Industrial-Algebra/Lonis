# Changelog

All notable changes to the Lonis workspace are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] — Unreleased

First public release of the Lonis workspace: an AI-native tool harness for
the Anima ecosystem. *"MCP exposes servers to models; Lonis exposes tools to
agents."*

### The contract (lonis-schema)

- `Block<P: BlockPayload>` — the doctrine §2.7 structured domain object:
  flat wire shape (`schema_version`, `provenance`, `warnings`, `attribution`,
  `bounds`, `payload`), `deny_unknown_fields` throughout (ADR-0001/0002)
- The 14-kind seed corpus (`BlockKind`) in three categories with a lossless
  `Extension` seam; verticals implement `BlockPayload` for fully-typed
  in-process contracts
- Full `Attribution` (identity / viewpoint / when / where / producer),
  first-class `BlockBounds`, `ReplayProvenance` (a superset of
  amari-discovery's provenance, for the future extraction), and
  `verify_replay` (ADR-0008)
- Canonical content hashing (SHA-256, key-sorted + number-normalized per
  ADR-0007) and render-parity (`render_human` from the same typed value)
- 16 curated draft-2020-12 JSON Schemas (14 kinds + envelope + extension
  seam) with golden wire fixtures and pinned hashes (ADR-0005)
- Namespaced-string `SchemaVersion` (`<name>/v<N>`) and the amari-aligned
  exit-code vocabulary

### The runtime (lonis-core)

- `Tool<P>` / `ToolRegistry<P>` — generic over the typed payload; a
  vertical's registry is homogeneous and fully typed (ADR-0002)
- `SubprocessTool` — bounded, isolated external CLI adapter: stdin JSON in,
  blocks out, structured `ToolError` on stderr; hard timeout + byte caps
  kill and reap; availability tri-state (ADR-0003)
- `SubprocessProvider` — one executable hosting many tools via
  `manifest` / `tools list` / `tools describe` / `call`; `lonis` itself is a
  conforming provider (ADR-0006)
- Stream mode: `BlockStream<P>` pull iterator with real backpressure
  (bounded channel), drop-kills-child, and a runtime-agnostic
  `futures_core::Stream` bridge behind the `futures` feature (ADR-0009)

### The macros (lonis-derive)

- `#[derive(LonisCapabilities)]` from `#[lonis(tool_id = "...")]`
- `#[derive(BlockPayload)]` — adjacently-tagged wire serde, namespaced kind
  tags, `schema_id`, and a `render_fn` hook, from one declaration (ADR-0004)

### The CLI (lonis-cli)

- `lonis tools list|describe`, `lonis call <id> [input | @file] [--stream]`,
  `lonis schema [kind]`, `lonis manifest`; `--mode human|json|ndjson`

### The facade (lonis)

- One crate re-exporting schema + derive + core, serde-style (`core`
  default; `derive`, `futures` opt-in)
