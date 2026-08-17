# SubprocessTool and SubprocessProvider

From `lonis-core`.

## `SubprocessTool`

Hosts one external CLI as one `Tool<BlockKind>`.

```rust
SubprocessTool::new(tool_id, command)
    .with_args(args)
    .with_timeout_millis(5_000)
    .with_max_stdout_bytes(1_048_576)
    .with_max_stderr_bytes(262_144)
    .with_cwd(path)
    .with_env(vars)
    .with_mapping(StdoutMapping::Blocks /* or Text */)
    .with_description(..).with_version(..)
```

- `availability() -> Availability` — `Ready` / `Missing` / `NotExecutable`
- `invoke` — stdin JSON in; blocks or `ToolError` out; bounded and isolated
  (see [Subprocess Tools](../concepts/subprocess.md))
- `invoke_stream` — ndjson lines become blocks as they arrive (see
  [Stream Mode](../concepts/stream-mode.md))
- `contract()` — a `ToolContract` (determinism `Nondeterministic`, side
  effects `MutatesExternal` by default)

## `SubprocessProvider`

Discovers and hosts a whole surface from one executable.

```rust
let provider = SubprocessProvider::new("mytool");
let manifest = provider.manifest()?;      // ProviderManifest
let tools = provider.tools()?;            // Vec<ProviderToolSummary>
let contract = provider.describe("op")?;  // ToolContract
let tool = provider.tool("op");           // SubprocessTool (argv: call op)
```

Discovery args default to the v0 surface with `--mode json` and are
overridable (`with_manifest_args`, `with_tools_list_args`); bounds set on
the provider are inherited by constructed tools. Dotted names mangle to
namespaced `ToolId`s.
