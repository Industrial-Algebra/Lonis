# Authoring Tools for the Subprocess Seam

The full guide lives in the repository at
[`docs/guides/subprocess-tool-authoring.md`](https://github.com/Industrial-Algebra/Lonis/blob/develop/docs/guides/subprocess-tool-authoring.md)
— this page is the summary.

## The wire protocol

- **in**: one JSON value on stdin, then EOF
- **out (success)**: blocks on stdout (array / ndjson / single object)
- **out (failure)**: structured `ToolError` on stderr + nonzero exit

## The rules

1. **Payloads serialize adjacently tagged** (`{"kind", "data"}`). Internal
   tagging hard-fails at the seam. In Rust, use
   `#[derive(BlockPayload)]`; in other languages, emit the two-key object.
2. **Kind tags are namespaced** (`<vertical>.<kind>`) and equal to
   `kind_name()` — one declaration, no divergence.
3. **The minimal block** requires `schema_version`, `attribution` (with
   `provenance.when` RFC 3339 + `provenance.producer`), and `payload`.
   Unknown top-level fields are rejected. Validate against
   `lonis schema block`.
4. **Targets arrive via stdin or argv** — never env or cwd. The
   environment is cleared and the cwd neutral by design.
5. **You are bounded** — hard timeout, byte caps; exceeding either kills
   your process. Keep payloads small; stream incrementally.
6. **Streaming**: one block per line (ndjson), and **flush after each
   line** — runtimes block-buffer stdout on pipes.

## Hosting many tools

Ship one executable with the provider surface (`manifest` / `tools list` /
`tools describe` / `call`) — the host discovers your whole operation set.
See [The Provider Model](../concepts/provider.md).

## Depending on Lonis pre-1.0-era

Since v0.1.0 the crates are on crates.io: `lonis-schema = "0.1"`. Before
publication, the discipline was git deps pinned to a rev, optional behind a
feature — the pattern is preserved in the full guide for pre-release
branches.
