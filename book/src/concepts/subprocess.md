# Subprocess Tools: Bounded and Isolated

`SubprocessTool` hosts an arbitrary external CLI as a Lonis `Tool` —
the thesis made concrete. Decided in
[ADR-0003](https://github.com/Industrial-Algebra/Lonis/blob/develop/docs/adr/0003-subprocess-tool-protocol.md).

## The wire protocol

| Direction | Channel | Shape |
|---|---|---|
| In | stdin | One JSON value, then EOF |
| Success | stdout | Blocks: JSON array, ndjson lines, or a single block object |
| Failure | stderr + nonzero exit | Structured `ToolError` JSON, propagated verbatim |

Anything else on stderr maps to kind `tool_failed` with the child's exit
code. Non-block stdout maps to `invalid_output` (exit 9).

## Bounds are first-class

Every invocation runs under:

- a **hard wall-clock timeout** (default 5 s),
- **byte caps** on stdout (default 1 MiB) and stderr (default 256 KiB).

Exceeding either **kills and reaps the child** and reports
`LIMIT_EXCEEDED` (exit 7).

## Isolation

Following the amari-discovery probe blueprint: direct exec (no shell), a
**cleared environment** (only `PATH` inherited, plus explicit additions),
and a **neutral working directory** (the system temp dir, configurable).

Consequence for tool authors: targets arrive via **stdin input or argv —
never env or cwd**.

## Availability

A tri-state probe (`Ready` / `Missing` / `NotExecutable` + reason) — a tool
the harness knows about may still be absent on this host. `invoke` on an
unavailable tool fails without spawning.

## Legacy CLIs

`StdoutMapping::Text` wraps raw stdout in an attributed `result` block with
a pinned `input_hash` — grep/jq-style tools work today; prefer real blocks
for anything you control.

## Legacy text wrap

See [Authoring Tools for the Subprocess Seam](../guide/authoring-tools.md)
for the full protocol guide, including streaming (ndjson + flush) and the
provider surface.
