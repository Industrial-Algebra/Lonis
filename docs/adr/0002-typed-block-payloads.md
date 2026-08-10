# ADR-0002: Typed block payloads — `Block<P: BlockPayload>`

- Status: accepted
- Date: 2026-08-10
- Branch: `feature/generic-block-payload`
- References: ADR-0001; feedback from the concurrent karpal-discovery session
  (the first external consumer of the Lonis contract)

## Context

ADR-0001 shipped a non-generic `Block` whose payload is the 14-kind
`BlockKind` enum, with `Extension { kind, data: serde_json::Value }` as the
extensibility seam. The karpal-discovery session — building the first vertical
against the contract — identified that this erases types *in-process*, where
typing matters most:

> Typing matters in-process — the vertical's own code, its tests, and any
> library consumer that depends on it. Across a process/JSON boundary it's
> always JSON; there's no type to preserve. So the goal is: maximize
> in-process typing, and only erase at a boundary where erasure is unavoidable
> anyway (a subprocess).

The generic-contagion worry (if `Block<P>` is generic, what is `P` for a
heterogeneous registry?) dissolves on inspection: **a vertical's registry is
homogeneous** — all of karpal-discovery's tools emit the same payload enum, so
the vertical gets a fully-typed `ToolRegistry<KarpalPayload>` with zero
erasure. Heterogeneity only exists at Lonis's umbrella layer, which hosts
tools from different verticals — and there the boundary is a subprocess JSON
channel, where erasure is natural, not a type-system problem.

## Decision

`Block` is generic over a typed payload:

```rust
pub trait BlockPayload: Serialize + DeserializeOwned + Send + Sync + 'static {
    fn kind_name(&self) -> &str;
    fn schema_id(&self) -> String;
    fn render_human(&self) -> String;
}

pub struct Block<P: BlockPayload> { /* … */ pub payload: P }
```

- **`Tool<P>`, `ToolRegistry<P>`, `render<P>`, `run_tool<P>` are generic over
  the same `P`.** A generic *parameter* (not an associated type): it matches
  the consumer's expected shape (`impl Tool<KarpalPayload> for DiscoverSearch`),
  keeps `Box<dyn Tool<P>>` object safety trivial, and makes a registry's
  monomorphism explicit in its type.
- **The 14-kind `BlockKind` implements `BlockPayload`** and is the seed
  payload. It keeps its name (it *is* the kind registry); `SeedBlock =
  Block<BlockKind>` is the umbrella host's type alias.
- **`Extension` stays on `BlockKind`, but its role narrows**: it exists for
  the erased seam (unknown kinds crossing a JSON boundary, the umbrella host
  parsing vertical output) — no longer as the way domains extend the contract.
  Domains now extend by implementing `BlockPayload` on their own enums.
- The trait is minimal (`kind_name`, `schema_id`, `render_human`). The
  doctrine `category()` stays off it: the three categories classify the seed
  corpus, not vertical payloads. The `'static` bound rules out borrowed-data
  payloads, which could never cross a JSON boundary anyway.
- The wire form is unchanged: seed blocks serialize exactly as ADR-0001;
  vertical payloads carry their own tagged form under `payload`, with the
  block envelope (`schema_version: lonis.block/v1`, attribution, bounds,
  provenance) uniform across all `P`.

Refines ADR-0001 decisions 1 and 3: `Vec<Block>` is now `Vec<Block<P>>`, and
extensibility is by trait impl rather than by `Extension` alone.

## Consequences

- karpal-discovery (and any vertical) defines `KarpalPayload: BlockPayload`
  once and every tool is end-to-end typed — no `to_value`/`from_value`
  round-trip, no type loss, catalog structs flow straight into payload
  variants.
- Breaking change to `Block`/`Tool`/`ToolRegistry` (all gains a `P`);
  acceptable pre-release, and cheaper now than after the subprocess and
  facade PRs land.
- `lonis-cli` instantiates `ToolRegistry<BlockKind>` — its behavior and wire
  output are unchanged (verified by the same integration tests).
- New tests: a `VerticalPayload` fixture (à la `KarpalPayload`) round-trips,
  renders, hashes, and pattern-matches with zero `Value` contact; the seed
  alias is exercised; all 70 workspace tests green with the full matrix clean.
