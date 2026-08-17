# The lonis CLI

```sh
lonis [--mode human|json|ndjson] <command>
```

## Commands

| Command | Purpose |
|---|---|
| `lonis tools list` | Registered tool ids and versions (`--mode json` → provider-list shape) |
| `lonis tools describe <id>` | A tool's `ToolContract` (JSON) |
| `lonis call <id> [input] [--stream]` | Invoke a tool |
| `lonis schema [kind]` | Emit the curated JSON Schemas (`lonis schema` lists all 16) |
| `lonis manifest` | The provider manifest — `lonis` is a conforming provider |

## Input

`call` accepts three input forms, all explicit:

- inline JSON: `lonis call lonis:builtin:echo '{"a": 1}'`
- a file: `lonis call lonis:builtin:echo @input.json`
- stdin (when the argument is omitted)

## The amari split

- **stdout**: blocks (a JSON array, one block per ndjson line, or human
  render) — always parseable.
- **stderr**: a structured `ToolError` (`{"kind", "message", "details?",
  "exit_code"}`) — with the process exit code set from it.

Exit-code baseline: `0` ok, `1` generic, `2` invalid input, `3` not found,
`4` confirmation required, `5` rate limited, `6` tool failed, `7` limit
exceeded, `8` io, `9` serialization, `69` not implemented, `70` internal.
Tools extend this map via `Capabilities::exit_code_map`.

## Streaming

`lonis call <id> <input> --stream --mode ndjson` renders blocks as the tool
produces them (see [Stream Mode](../concepts/stream-mode.md)).
