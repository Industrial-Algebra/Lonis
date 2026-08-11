# Authoring tools for the Lonis subprocess seam

A guide for writing external tools that `SubprocessTool` hosts (ADR-0003),
in any language. Distills the karpal-discovery spike findings
(2026-08-10) into rules.

## The wire protocol (ADR-0003)

| Direction | Channel | Shape |
|---|---|---|
| Input | stdin | One JSON value, then EOF |
| Success | stdout | Blocks: JSON array, or one block per line (ndjson), or a single block object |
| Failure | stderr + nonzero exit | A structured `ToolError` JSON (`{"kind", "message", "details?", "exit_code"}`), propagated verbatim |

Anything else on stderr with a nonzero exit maps to kind `tool_failed` with
your exit code. Stdout that is not blocks maps to `invalid_output` (exit 9).

## Rules every tool author must know

### 1. Payloads serialize adjacently tagged: `{"kind": …, "data": …}`

The host parses `payload` as a `kind` tag plus a `data` payload. **Internal
tagging** (fields flattened next to `kind`) hard-fails at the seam with
`block payload requires 'data'`. In Rust, use `#[derive(BlockPayload)]` and
this is generated for you; in other languages, emit the two-key object.

### 2. Kind tags are namespaced: `<vertical>.<kind>`

The host sees only the serde `kind` tag — it becomes `Extension.kind`. Tag
your kinds `<vertical>.<snake_case_kind>` (`karpal.search`, not `search`) so
kinds never collide across verticals. In Rust, the derive generates the serde
tag, `kind_name()`, and `schema_id()` (`lonis.block/<kind>/v1`) from one
declaration, so they cannot diverge:

```rust
#[derive(Debug, Clone, PartialEq, lonis_schema::BlockPayload)]
#[lonis_payload(namespace = "karpal")]
enum KarpalPayload {
    Search { results: Vec<String> },
    Ready,
}
// kind_name() == serde tag == "karpal.search" / "karpal.ready"
```

Unknown kinds are *tolerated* host-side (they land in `Extension`
losslessly) but *rejected* when deserializing into your own enum — your
vertical's universe is closed.

### 3. The minimal block JSON shape (non-Rust producers)

Every block requires `schema_version`, `attribution` (with
`provenance.when` RFC 3339 and `provenance.producer`), and `payload`:

```json
{
  "schema_version": "lonis.block/v1",
  "attribution": {
    "identity": "mytool:ns:op",
    "provenance": { "when": "2026-08-10T12:00:00Z", "producer": "mytool:ns:op" }
  },
  "payload": { "kind": "mytool.thing", "data": { "…": "…" } }
}
```

Optional: `provenance` (replay: `input_hash`, `seed`, …), `warnings`,
`bounds`. Unknown top-level fields are **rejected** (`deny_unknown_fields`).

The normative, machine-checkable form is the curated JSON Schema:
`lonis schema block` (envelope) or `lonis schema <kind>` (per kind) emits
the draft 2020-12 document; `lonis schema` lists all 16 families. Validate
your output against them before shipping a tool — the envelope's `payload`
`oneOf` includes an `extension` branch, so vertical kinds (like the
`mytool.thing` example above) validate: the envelope checks structure, and
your own kind schema checks your `data` shape.

### 4. Targets arrive via stdin input or argv — never env or cwd

`SubprocessTool` deliberately isolates: the environment is cleared (only
`PATH` is inherited, plus variables you explicitly configure) and the working
directory is neutral (the system temp dir, configurable). A tool that
operates on an external target must receive it in the input JSON
(`{"workspace": "…", "query": "…"}`) or its argv — anything read from the
ambient environment or cwd will not be there.

### 5. You are bounded — design for it

Every invocation runs under a hard wall-clock timeout and byte caps on both
output streams (defaults 5 s / 1 MiB stdout / 256 KiB stderr, configurable by
the host). Exceeding either **kills your process** and reports
`LIMIT_EXCEEDED` (exit 7). Stream incrementally and keep payloads small;
large results belong in files referenced by blocks, not in blocks.

### 6. Plain-text legacy CLIs still work

With `StdoutMapping::Text`, the host wraps raw stdout in an attributed
`result` block with a pinned `input_hash`. Useful for grep/jq-style tools;
prefer real blocks for anything you control.

## Depending on Lonis pre-publication (Rust consumers)

Until `lonis-schema` reaches crates.io, depend via **git deps pinned to a
rev**, never path deps — path deps resolve relative to the *worktree* and
break across worktree/machine locations:

```toml
[dependencies]
lonis-schema = { git = "https://github.com/Industrial-Algebra/Lonis", rev = "3f3ffdc", optional = true }
lonis-core   = { git = "https://github.com/Industrial-Algebra/Lonis", rev = "3f3ffdc", optional = true }

[features]
lonis = ["dep:lonis-schema", "dep:lonis-core"]
```

Optional deps behind a feature keep the default build publishable and
lonis-free (validated by karpal-discovery: 19 default tests, no lonis fetch).

## Reference implementation

`crates/lonis-core/examples/mock_tool.rs` is a minimal Rust tool speaking
this protocol (all modes: blocks, ndjson, text, structured failure, bounded
output). Copy it.
