# Typed Payloads and the Erased Seam

Decided in
[ADR-0002](https://github.com/Industrial-Algebra/Lonis/blob/develop/docs/adr/0002-typed-block-payloads.md),
from the karpal-discovery session's feedback.

## The principle

**Typing matters in-process; across a process/JSON boundary it's always
JSON anyway.** So maximize in-process typing, and erase only where erasure
is unavoidable — a subprocess.

## What it means concretely

`Block` is generic over its payload:

```rust
pub trait BlockPayload: Serialize + DeserializeOwned + Send + Sync + 'static {
    fn kind_name(&self) -> &str;
    fn schema_id(&self) -> String;
    fn render_human(&self) -> String;
}
```

A vertical defines its own payload enum — karpal-discovery's
`KarpalPayload` — and gets a **fully-typed** `Block<KarpalPayload>`,
`Tool<KarpalPayload>`, and `ToolRegistry<KarpalPayload>`. Zero erasure
anywhere the vertical reaches.

Erasure reappears exactly once: at the umbrella host, where blocks arrive
across a subprocess JSON channel and are parsed as `SeedBlock`
(`Block<BlockKind>`). Unknown kinds land in `BlockKind::Extension`
losslessly — the **erased seam**. Erasure is topological: on the boundary
of the system, nowhere in the interior.

## The derive

`#[derive(BlockPayload)]` makes the seam's two easy-to-get-wrong rules
compile-time guarantees ([ADR-0004](https://github.com/Industrial-Algebra/Lonis/blob/develop/docs/adr/0004-vertical-payload-authoring.md)):

```rust
#[derive(Debug, Clone, PartialEq, lonis_schema::BlockPayload)]
#[lonis_payload(namespace = "karpal", render_fn = "render_search")]
enum KarpalPayload {
    Search { query: String, results: Vec<ItemSummary> },
    Ready,
}
// serde tag == kind_name() == "karpal.search" — from one declaration.
```

- Payloads serialize **adjacently tagged** (`{"kind", "data"}`) — an
  internally-tagged enum hard-fails at the seam, and the derive makes that
  shape automatic.
- Kind tags are **namespaced** (`<vertical>.<kind>`) so kinds never collide
  across verticals.
- `render_fn` keeps a custom human render without giving up the derived
  wire safety.
