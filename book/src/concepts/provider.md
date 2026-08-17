# The Provider Model

One executable hosting many tools. Decided in
[ADR-0006](https://github.com/Industrial-Algebra/Lonis/blob/develop/docs/adr/0006-subprocess-provider.md).

## The problem it solves

`SubprocessTool` binds one argv prefix to one tool. A vertical with a full
operation set (search / detail / inspect / recommend / …) shouldn't
register each one individually — the executable itself should describe its
surface.

## The four subcommands

```bash
mytool --mode json manifest              # {"name", "version", "tools": [...], ...}
mytool --mode json tools list            # {"provider", "tools": [{"name", "description"}]}
mytool --mode json tools describe <name> # a ToolContract JSON
mytool --mode json call <name>           # ADR-0003 invocation (stdin JSON → blocks)
```

`SubprocessProvider` discovers that surface and constructs a
`SubprocessTool` per operation — with provider-wide bounds (timeout, byte
caps, env, cwd) inherited by every tool.

## Details that matter

- **Forward-compatible**: manifests tolerate unknown fields — a host never
  rejects a newer provider for new metadata.
- **Name mangling**: dotted v0 names (`figma.get_document`) become
  namespaced `ToolId`s (`figma:get_document`).
- **The same bounded, isolated execution core** as `SubprocessTool` runs
  discovery and invocation.

## lonis is a conforming provider

```bash
lonis manifest
lonis --mode json tools list
```

The harness hosting itself through the seam is the dogfood — and the
integration test.
