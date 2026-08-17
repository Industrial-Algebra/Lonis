# Lonis Harness Design Draft

## Status

This document replaces the older assumption that Lonis itself would evolve into the Perceptron perception engine.

That is no longer the plan.

## Revised Direction

### Lonis
Lonis becomes a **strict AI-oriented tool harness**:

- local-first
- machine-consumable
- pluggable
- optimized for LLM/agent use rather than human CLI ergonomics
- implemented as a Rust monorepo

### Perceptron
Perceptron moves to a **separate repository/project** and exposes a tool surface that Lonis can consume and format for AI use.

### Other tool surfaces
Other existing systems can also be exposed through Lonis, including:

- a rewritten Figma bridge tool surface derived from the existing Kai Jyozu work
- future local or external tools with explicit contracts

So the new model is not:

```text
Lonis contains Perceptron
```

but:

```text
Lonis hosts and routes tool surfaces, including Perceptron
```

## Core Thesis

Lonis should be:

> A local-first, AI-native command harness that exposes sharply bounded, discoverable, machine-readable tool surfaces with stable contracts and explicit capability boundaries.

Humans are not the primary audience.

Humans will:
- install Lonis
- inspect Lonis
- debug Lonis
- develop tools for Lonis

But the primary consumer is:
- an LLM
- an agent runtime
- a coding/planning system
- other automated tool-using software

That priority should shape every major design decision.

## Why Lonis Exists

The motivation is not simply to create another CLI framework.

Lonis exists to provide a better alternative to MCP-style integration in cases where MCP introduces too much friction.

### Problems with MCP-style integration

- ambient tool exposure pollutes model context
- configuration is often annoying and fragile
- tools are exposed through persistent server surfaces whether needed or not
- server lifecycle and transport behavior add avoidable complexity
- trust boundaries are often too coarse
- the practical tool surface can become messy or difficult to inspect

### Lonis advantages

- explicit local invocation
- bounded tool exposure
- machine-readable discovery
- structured outputs optimized for AI consumption
- less ambient protocol overhead
- cleaner capability boundaries
- easier composition and scoping

A useful formulation:

> MCP exposes servers to models; Lonis exposes tools to agents.

## Primary Design Principle: Optimize for AI Consumption

Because Lonis is intended primarily for AI consumers, it should prefer:

- stable schemas over human-friendly prose
- predictable invocation over shorthand convenience
- explicit contracts over convention
- typed outputs over terminal decoration
- machine-readable discovery over documentation-only discoverability
- explicit errors over freeform diagnostics
- bounded tool surfaces over ambient capability

### Design consequences

This implies:

- structured output by default
- structured input by default
- canonical invocation patterns
- a discoverable tool registry
- versioned contracts
- explicit side-effect metadata
- strict separation of stdout vs stderr responsibilities

## What Lonis Is Not

Lonis should not be designed as:

- a human-first shell utility
- a shell replacement
- an MCP clone
- a generic daemon framework
- a wrapper around arbitrary shell commands with minimal structure
- a monolithic AI assistant product

It is a harness for tool surfaces.

## Conceptual Model

Lonis should be understood as four things at once.

### 1. Harness runtime
Responsible for:
- discovering tools
- validating inputs
- invoking tools
- formatting outputs
- surfacing errors
- applying policy and capability constraints

### 2. Contract layer
Defines:
- tool names
- tool descriptions
- input schema
- output schema
- side-effect metadata
- versioning

### 3. Local tool bus
A local invocation substrate for agents and runtimes that want deterministic, explicit access to tools without persistent protocol overhead.

### 4. Monorepo platform
A Rust workspace containing:
- the core harness
- shared schema/runtime crates
- adapters and tool integration crates
- first-party utility surfaces

## Separation of Concerns

This separation should remain sharp.

### Lonis
- tool harness
- registry
- runtime
- formatting
- policy
- discovery

### Perceptron
- structured perception engine
- separate repo
- tool provider consumed by Lonis

### Figma bridge surface
- application-specific integration surface
- likely adapted from Kai Jyozu concepts
- consumed through Lonis as another tool family

This avoids collapsing Lonis into any one domain.

## AI-First UX Principles

## 1. Discovery must be machine-readable
An AI should be able to determine:
- what tools exist
- what each tool does
- what inputs it accepts
- what outputs it emits
- what risks/side effects/costs it carries

without reading prose documentation.

Possible commands:

```bash
lonis tools list --format json
lonis tools describe perceptron.analyze --format json
lonis tools schema figma.get_document --format json
```

## 2. Output must be structurally reliable
AI systems are harmed by:
- mixed prose and data
- inconsistent shapes
- warnings interleaved with stdout payloads
- decorative formatting in default mode

Preferred rule:
- stdout = data
- stderr = diagnostics/logging
- exit code = coarse execution status

## 3. Invocation must be canonical
Rather than many one-off invocation styles, Lonis should have one primary machine-oriented call shape.

For example:

```bash
lonis call perceptron.analyze --input @request.json
lonis call figma.get_document --input @request.json
```

or stdin-based variants.

The important design point is consistency.

## 4. Tool scope must be explicit
Agents should not inherit a giant, ambient pool of capabilities without clear boundaries.

Tools should be discoverable and invokable under explicit profiles, policies, or registries.

## 5. Errors must be machine-actionable
Errors should have:
- stable codes
- short messages
- structured details
- actionable meaning

## 6. Human niceties are secondary overlays
Pretty renderers, colorized views, summaries, or debugging helpers are useful, but they should sit on top of a strong machine contract rather than define the primary interface.

## Canonical Invocation Model

A strong candidate is:

```bash
lonis call <tool-name>
```

with structured input passed by:
- file
- stdin
- inline JSON/EDN where appropriate

This gives the AI consumer a single stable execution pattern.

### Example response envelope

```json
{
  "ok": true,
  "tool": "perceptron.analyze",
  "schema_version": "1",
  "result": {},
  "meta": {
    "duration_ms": 182,
    "warnings": []
  }
}
```

### Example error envelope

```json
{
  "ok": false,
  "tool": "figma.delete_node",
  "schema_version": "1",
  "error": {
    "code": "confirmation_required",
    "message": "Destructive action requires explicit confirmation",
    "details": {
      "field": "confirm"
    }
  }
}
```

## Tool Contract Requirements

Every Lonis tool should declare at minimum:

- stable tool name
- short purpose description
- input schema
- output schema
- schema version
- determinism level
- side-effect class
- capability requirements
- cost hint
- human-readable usage examples
- machine-readable examples where possible

Possible metadata shape:

```json
{
  "name": "perceptron.analyze",
  "description": "Generate structured perceptual description for an input image",
  "schema_version": "1",
  "capabilities": ["read_file", "image_decode", "model_inference"],
  "side_effects": [],
  "determinism": "bounded-nondeterministic",
  "cost": "medium"
}
```

## Output Formats

Lonis should distinguish between:

### Internal/canonical exchange format
Likely JSON envelopes for broad compatibility and robust parsing.

### Domain payload formats
Tool results may include or emit:
- JSON
- EDN
- p-expressions
- plain text renderings
- binary artifact references

This is especially important because Perceptron may want to emit EDN/p-expressions, while Lonis should still provide a stable machine envelope around that output.

Possible model:
- Lonis envelope in JSON
- payload can declare `content_type`
- renderers can convert for display or secondary consumption

## Capabilities and Trust Model

Because Lonis is for AI use, tool declarations should include explicit capability and risk metadata.

Examples:
- read-only
- writes filesystem
- mutates external application
- network access
- model inference
- high-cost
- destructive
- nondeterministic

This enables:
- policy filtering
- safe-by-default tool profiles
- runtime warnings
- host-side decision-making

## State Model

Lonis should begin stateless by default.

If tools need reusable artifacts or sessions, those should be explicit:
- `session_id`
- `artifact_id`
- `handle`
- `cache_key`

Avoid hidden ambient memory.

This is especially important for AI systems, which benefit from explicit references more than implicit session magic.

## Monorepo Shape (Provisional)

A likely Rust workspace shape:

```text
lonis/
├── Cargo.toml
├── crates/
│   ├── lonis-cli/          # binary entrypoint
│   ├── lonis-core/         # harness runtime
│   ├── lonis-schema/       # shared input/output/type definitions
│   ├── lonis-registry/     # tool registration and discovery
│   ├── lonis-runtime/      # execution engine
│   ├── lonis-output/       # envelope/render/format support
│   ├── lonis-policy/       # capability and safety policy
│   ├── lonis-adapters/     # external tool/provider integration patterns
│   ├── lonis-figma/        # Figma tool surface adapter/bindings
│   └── ...
└── docs/
```

Perceptron should remain out-of-repo and integrate through a tool surface or adapter contract.

## Tool Naming

Namespaced tool names are strongly preferred.

Examples:
- `perceptron.analyze`
- `perceptron.attend`
- `figma.get_document`
- `figma.create_frame`
- `figma.set_fill`

Benefits:
- scales better
- reduces ambiguity
- communicates grouping to agents
- supports multiple providers/families cleanly

## External Tool Surface Model

Because Perceptron and the Figma surface may live outside the core harness, Lonis should support a clear provider model.

Possible provider types:

### 1. Native crate providers
Compiled directly into the workspace.

### 2. External executable providers
Tools exposed by separate programs under a Lonis-compatible contract.

### 3. Adapter providers
Wrappers around existing systems that normalize them to Lonis contracts.

This is likely important for Perceptron and for the Figma bridge rewrite.

## Lessons from Kai Jyozu

The existing Kai Jyozu reference suggests several design lessons that Lonis should absorb.

### 1. Verified tool surfaces matter
Kai Jyozu tracks a concrete verified tool surface and compatibility matrix.

Lonis should likely support:
- declared tools
- implemented tools
- verified tools
- unsupported tools

That distinction is useful for AI consumers and operators.

### 2. Diagnostics matter
Kai Jyozu includes:
- `status`
- `doctor`
- health checks
- process locks
- audit logs

Lonis should probably include analogous diagnostics for the harness and tool providers, even if the core mission is AI-first.

### 3. Safety metadata must be explicit
Kai Jyozu surfaces destructive-tool requirements directly in tool metadata.

Lonis should carry that forward as a general principle:
- side effects are part of the contract
- confirmations should be visible in schemas/metadata
- dangerous tools should be easy to filter or gate

### 4. Cross-consumer compatibility matters
Kai Jyozu explicitly considers multiple agent/client environments.

Lonis should similarly aim for output and invocation discipline that works across:
- direct shell invocation
- editor integrations
- coding agents
- scripts
- local orchestrators

### 5. Operational clarity matters even for AI-facing systems
An AI-first harness still benefits from human inspection tools when debugging deployment or integration issues.

## Figma as an Early Design Informant

The Figma bridge rewrite is likely an excellent early proving ground for Lonis because:

- the domain has many tools
- safety and mutation boundaries matter
- structured inputs/outputs matter
- external application state matters
- verification and diagnostics matter

This makes it a strong reference implementation for:
- tool contracts
- side-effect declarations
- policy gating
- diagnostics commands
- capability grouping

## Non-Goals for v1

To preserve focus, initial Lonis should avoid becoming:

- a generic shell replacement
- a persistent distributed runtime
- an embedded agent framework
- a package registry from day one
- a daemon-first architecture
- a catch-all execution wrapper for arbitrary shell tools

## Likely v1 Command Families

### Discovery
- `lonis tools list`
- `lonis tools describe <tool>`
- `lonis tools schema <tool>`

### Invocation
- `lonis call <tool>`

### Diagnostics
- `lonis status`
- `lonis doctor`

### Rendering/inspection
- `lonis render <artifact-or-result>`
- `lonis inspect <artifact-or-result>`

### Policy/profile
- `lonis profile list`
- `lonis profile show <name>`

## Strong Design Statement

A concise statement of intent:

> Lonis is an AI-native tool harness, not a human-first CLI and not a perception engine. Its job is to expose sharply bounded, discoverable, machine-readable tool surfaces with stable contracts and explicit capability boundaries. Perceptron, Figma integration, and other systems should plug into Lonis through those contracts rather than defining Lonis itself.

## Open Questions

The biggest unresolved questions appear to be:

1. What is the canonical Lonis envelope format?
   - JSON only?
   - JSON envelope with EDN-capable payloads?

2. What is the provider model?
   - compiled-in crates?
   - external executables?
   - both?

3. How are tool schemas declared?
   - Rust traits?
   - manifests?
   - macros + generated schema?

4. How much policy is part of core Lonis?
   - minimal capability metadata only?
   - runtime enforcement?
   - profile-based filtering?

5. What diagnostics are core versus provider-specific?

6. How should verification state be represented?
   - implemented vs verified vs experimental?

## Recommended Immediate Next Step

Before deeper implementation planning, define:

1. the canonical `lonis call` contract
2. the tool metadata/schema model
3. the provider model for external tool surfaces
4. one concrete reference integration, likely the Figma surface

That will make the rest of the harness design substantially easier to ground.
