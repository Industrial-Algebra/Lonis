# ADR-0006: The provider surface — `SubprocessProvider` and lonis-as-provider

- Status: accepted
- Date: 2026-08-10
- Branch: `feature/subprocess-provider`
- References: ADR-0003 (named this layer as future work);
  `docs/plans/2026-03-28-lonis-external-provider-protocol-spec-v0.md` §3–§8;
  karpal-discovery spike ("many operations → provider model")

## Context

`SubprocessTool` (ADR-0003) binds one argv prefix to one tool. The
karpal-discovery spike named the consequence: registering each of a
vertical's operations individually is verbose, and the v0 provider spec
already sketched the cleaner model — **one executable hosting many tools**
behind `manifest` / `tools list` / `tools describe` / `call` subcommands.

## Decision

**`SubprocessProvider`** (in `lonis-core`) discovers a provider's surface and
constructs `SubprocessTool`s from it:

- Discovery runs the v0 commands (`--mode json manifest`, `--mode json tools
  list`, `--mode json tools describe <name>`) through the **same bounded,
  isolated execution core** as `SubprocessTool` (extracted as
  `exec_bounded`: spawn with `env_clear` + `PATH`, optional stdin, capped
  drain threads, hard timeout, kill-and-reap).
- `ProviderManifest` / `ProviderToolList` tolerate unknown fields (providers
  evolve; a host must not reject newer metadata).
- `tools describe` parses directly into `ToolContract` — the shapes were
  already isomorphic (v0 §8 was written against the same instinct).
- `provider.tool(name)` returns a `SubprocessTool` with argv prefix
  `--mode json call <name>`, inheriting the provider's bounds.
- **Name mangling**: v0 tool names may be dotted (`mock.echo`,
  `figma.get_document`); dots become colons for the `ToolId`
  (`mock.echo` → `mock:echo`), with a `provider:` prefix for bare names.

**`lonis` is a conforming provider.** The CLI gains a `manifest` subcommand
and a JSON mode for `tools list` (v0 §7 shape: `{"provider", "tools":
[{"name", "version", "description"}]}`); `tools describe` already emitted
`ToolContract` JSON; `call` already speaks ADR-0003. The self-host dogfood —
`SubprocessProvider` discovering and invoking `lonis`'s own builtins through
a real process boundary — is an integration test.

**Divergence from the v0 spec** (recorded, deliberate): the v0 spec's `call`
request/response shapes (§9–§10, a wrapped `{tool, input}` request) are
superseded by ADR-0003's simpler stdin-JSON/blocks-out protocol; the v0
discovery surface (§6–§8) stands. The spec's `status`/`doctor` diagnostics
and artifact handling remain future work.

## Consequences

- A vertical ships *one* executable; the host gets its whole operation set
  from `manifest`/`tools list` — karpal-discovery's verbosity complaint is
  answered structurally.
- `lonis-core` gains a `serde` dependency (provider DTOs) and exports
  `SubprocessProvider`, `ProviderManifest`, `ProviderToolList`,
  `ProviderToolSummary`.
- A real bug was caught by the new tests: `exec_bounded` previously left
  stdin open when no input was provided — any discovery call (or
  stdin-reading tool invoked without input) would hang to the timeout.
  stdin now always closes (EOF) whether or not input was written.
- 12 new tests: mock-provider end-to-end (manifest/list/describe/call,
  error propagation, unavailable, non-conforming), CLI conformance
  (`manifest`, `tools list --mode json`), the self-host dogfood, and unit
  tests (manifest tolerance, name mangling). 129 workspace tests green;
  matrix clean.
