# lonis-cli

The `lonis` command-line harness — the user-facing surface over
[`lonis-core`](../lonis-core).

## Usage

```text
lonis [options] <command>

Commands:
  tools list                       List registered tool ids + versions
  tools describe <id>              Show a tool's contract
  call <id> [input]                Invoke a tool (INPUT is JSON; stdin if omitted)

Options:
  --mode <human|json|ndjson>       Output mode (default: human)
```

## Built-in tools

- `lonis:builtin:echo` — echoes the input JSON back as the result.
- `lonis:builtin:version` — reports the `lonis` and schema versions.

## The amari split

`lonis call` enforces decision #1: a parseable [`Envelope`] always lands on
**stdout**, structured errors always land on **stderr** with a stable exit code.
This keeps stdout cleanly typed for piping and streaming:

```sh
lonis call lonis:builtin:echo '{"a":1}' --mode json   # envelope on stdout
lonis call lonis:nope:x --mode json; echo $?          # exit 3; error JSON on stderr
```
