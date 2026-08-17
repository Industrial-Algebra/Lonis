# Introduction

**Lonis is an AI-native tool harness** for the
[Anima](https://github.com/Industrial-Algebra) ecosystem — a local-first,
machine-readable alternative to ambient server protocols for exposing
sharply bounded tool surfaces to agents.

> *"MCP exposes servers to models; Lonis exposes tools to agents."*

Lonis is built on one conviction, inherited from its namesake's first life as
a vision analyzer: **never let the model guess**. Where the original Lonis
measured pixels so a vision model couldn't hallucinate colors, this Lonis
binds tools to contracts so an agent can't misread what it can do. In both
incarnations, the LLM is a pure reasoner suspended between an unreliable
sensorium and an unreliable effector system — and Lonis is the well-formed
I/O membrane around it.

## What it is

- **A contract** (`lonis-schema`): the `Block` — a structured domain object
  every tool emits through, uniformly versioned, attributed, bounded,
  replayable, and render-parity (human and machine render from the same
  typed value).
- **A runtime** (`lonis-core`): the `Tool` trait, registries, per-mode
  rendering, and bounded adapters that host *any* composable CLI as a tool —
  with hard timeouts, byte caps, cleared environments, and kill-and-reap
  discipline.
- **A CLI** (`lonis`): the harness binary — discover tools, invoke them,
  emit their schemas. `lonis` is itself a conforming provider: it can host
  itself.
- **Macros** (`lonis-derive`): derives that make the contract's sharp edges
  compile-time guarantees instead of runtime failures.
- **A facade** (`lonis`): one crate re-exporting all of it, serde-style.

## Where it sits

In the Anima doctrine's planes, Lonis is the **Tools plane** — the piece
that turns the other planes' domain objects into invocable verbs with
bounded surfaces. `amari-discovery` is the reference vertical whose
machinery Lonis generalizes (and which will eventually delete its own
protocol layer into `lonis-schema`). `karpal-discovery` is the first
external consumer.

## Why not MCP?

MCP is the diplomacy layer — how Anima talks to *external* harnesses. Lonis
is domestic tooling: Unix, not RPC. Fixed argv, JSON on stdin, blocks on
stdout, structured errors on stderr, exit codes, cleared environments, byte
caps, kill-and-reap on timeout. For internal tools, a bounded subprocess
with a sharp contract beats a long-lived server: less context pollution, no
connection lifecycle, crash isolation by construction.
