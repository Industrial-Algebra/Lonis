# Lonis Provider and Interface Specification v0

## Status

This is a first-pass specification draft for how **Lonis** should discover, describe, invoke, and format external tool surfaces.

It assumes:

- Lonis is strictly the harness
- Perceptron is a separate project/repo
- Figma integration is another external tool surface/provider
- Lonis is optimized primarily for AI consumers rather than human CLI ergonomics

This document is intended to define the minimum contract needed to make provider integration real.

---

## 1. Design Goals

The provider model should make it possible for Lonis to:

- consume tool surfaces from independent projects
- expose those tools through one stable AI-facing interface
- normalize discovery and invocation across heterogeneous providers
- preserve strong metadata around capabilities, side effects, and verification status
- avoid MCP-style ambient protocol complexity
- remain local-first and explicit

The interface model should make it possible for an AI to:

- list available tools
- inspect schemas and metadata
- invoke tools in one canonical way
- receive predictable machine-readable results
- distinguish success, failure, warnings, and policy restrictions cleanly

---

## 2. Terminology

### Lonis
The harness runtime and CLI.

### Provider
A source of one or more tools that Lonis can discover and invoke.

Examples:
- Perceptron provider
- Figma bridge provider
- future filesystem/search/code providers

### Tool
A single invokable capability exposed through Lonis.

Examples:
- `perceptron.analyze`
- `perceptron.attend`
- `figma.get_document`
- `figma.create_frame`

### Contract
The machine-readable declaration of a tool's identity, inputs, outputs, behavior, and constraints.

### Envelope
The standard Lonis wrapper around tool results and tool errors.

### Artifact
A persisted or referenceable result object produced by a tool, optionally reused by later calls.

---

## 3. Provider Model Overview

Lonis should support providers as first-class entities.

A provider contributes:

- provider metadata
- a set of tool definitions
- an invocation mechanism
- optional provider diagnostics
- optional provider-specific artifact/session handling

### Provider responsibilities
A provider must be able to:

1. identify itself
2. enumerate its tools
3. describe each tool in a machine-readable format
4. accept normalized invocation requests
5. return normalized invocation results or errors

### Lonis responsibilities
Lonis must:

1. discover providers
2. merge provider tools into a unified registry
3. expose a consistent command surface
4. apply policy/visibility filtering
5. normalize responses into Lonis envelopes
6. surface diagnostics and metadata clearly

---

## 4. Provider Types

v0 should support at least two provider classes.

## 4.1 Native providers
Providers compiled into the Lonis workspace as Rust crates.

Pros:
- best performance
- strongest type integration
- easiest internal testing
- direct schema generation possible

Use cases:
- core diagnostic tools
- built-in utility surfaces
- future first-party Lonis-native tools

## 4.2 External executable providers
Providers implemented in separate repositories/programs that Lonis invokes through a standard contract.

Pros:
- supports separation of projects
- ideal for Perceptron
- ideal for Figma bridge rewrite if kept independently deployable
- clean boundary between harness and provider

Use cases:
- Perceptron
- Figma integration
- domain-specific external systems

## 4.3 Deferred provider types
Out of scope for v0 unless clearly needed:

- dynamic library loading
- remote network providers
- always-on daemon providers as the default model

These can be added later if justified.

---

## 5. Canonical Lonis Command Surface

Lonis should present one primary machine-oriented invocation model.

## 5.1 Discovery commands

```bash
lonis providers list --format json
lonis providers describe <provider> --format json
lonis tools list --format json
lonis tools describe <tool> --format json
lonis tools schema <tool> --format json
```

## 5.2 Invocation command

```bash
lonis call <tool-name>
```

Structured input should be accepted via:
- `--input @file.json`
- stdin
- possibly inline data for small requests

The important thing is that AI consumers only need to learn **one canonical call shape**.

## 5.3 Diagnostics commands

```bash
lonis status --format json
lonis doctor --format json
lonis provider status <provider> --format json
lonis provider doctor <provider> --format json
```

The Kai Jyozu reference strongly suggests diagnostics need to be first-class.

---

## 6. Tool Naming Rules

Tools should use stable namespaced identifiers.

Pattern:

```text
<provider>.<tool>
```

Examples:
- `perceptron.analyze`
- `perceptron.attend`
- `figma.get_document`
- `figma.create_frame`

Rules:
- lowercase ASCII
- dot-separated namespaces
- stable once published
- no ambiguous aliases in machine mode

Human-friendly aliases may exist later, but should not replace the canonical tool name.

---

## 7. Provider Manifest Requirements

Every provider should expose machine-readable metadata.

Suggested minimal provider manifest:

```json
{
  "name": "figma",
  "version": "0.1.0",
  "display_name": "Figma Tool Surface",
  "description": "Lonis provider for interacting with a live Figma session",
  "provider_type": "external-executable",
  "protocol_version": "0",
  "tools": [
    "figma.get_document",
    "figma.create_frame"
  ],
  "capabilities": [
    "external_application",
    "mutable_operations",
    "diagnostics"
  ]
}
```

Additional useful metadata:
- homepage/repo
- schema version
- supported Lonis versions
- experimental/stable status
- verification summary

---

## 8. Tool Contract Requirements

Every tool must expose a machine-readable contract.

Suggested minimum fields:

```json
{
  "name": "figma.create_frame",
  "provider": "figma",
  "schema_version": "1",
  "description": "Create a Figma frame on the current page or under a specified parent",
  "input_schema": {},
  "output_schema": {},
  "examples": [],
  "capabilities": ["mutates_external_application"],
  "side_effects": ["writes_figma_document"],
  "determinism": "deterministic",
  "cost": "low",
  "verification": {
    "status": "verified",
    "last_verified_at": "2026-03-28T00:00:00Z"
  },
  "safety": {
    "destructive": false,
    "requires_confirmation": false
  }
}
```

## Required contract fields

### Identity
- `name`
- `provider`
- `schema_version`
- `description`

### Schemas
- `input_schema`
- `output_schema`

### Behavior metadata
- `determinism`
- `cost`
- `capabilities`
- `side_effects`

### Safety metadata
- `destructive`
- `requires_confirmation`
- optional `policy_tags`

### Maturity metadata
- `verification.status`

## Recommended verification statuses
- `experimental`
- `implemented`
- `verified`
- `deprecated`
- `disabled`

This follows a useful lesson from Kai Jyozu: AI-facing tools benefit from explicit verified-vs-merely-implemented distinctions.

---

## 9. Canonical Invocation Request

Lonis should normalize all tool invocations into a standard request object.

Suggested request shape:

```json
{
  "tool": "figma.create_frame",
  "schema_version": "1",
  "input": {
    "name": "Hero",
    "x": 100,
    "y": 100,
    "width": 1440,
    "height": 900
  },
  "context": {
    "profile": "default",
    "request_id": "req_123",
    "cwd": "/workspace/project"
  }
}
```

## Notes
- `context` should remain optional and explicit
- providers should not assume hidden ambient state
- request IDs are useful for diagnostics and auditability

---

## 10. Canonical Invocation Response Envelope

Lonis should always return a standard envelope.

## 10.1 Success envelope

```json
{
  "ok": true,
  "tool": "figma.create_frame",
  "provider": "figma",
  "schema_version": "1",
  "result": {
    "node_id": "123:456",
    "name": "Hero"
  },
  "meta": {
    "duration_ms": 41,
    "warnings": [],
    "artifacts": []
  }
}
```

## 10.2 Error envelope

```json
{
  "ok": false,
  "tool": "figma.delete_node",
  "provider": "figma",
  "schema_version": "1",
  "error": {
    "code": "confirmation_required",
    "message": "Destructive tool requires explicit confirmation",
    "details": {
      "field": "confirm"
    }
  },
  "meta": {
    "duration_ms": 2,
    "warnings": []
  }
}
```

## 10.3 Envelope rules
- stdout should contain only the envelope in machine mode
- stderr should contain diagnostics/logging only
- exit code should indicate coarse process-level success/failure
- `ok=false` does not necessarily require a nonzero exit if the harness completed normally and returned a valid tool error envelope

This point should be decided explicitly during implementation.

---

## 11. Error Taxonomy

Lonis should define a core error vocabulary.

Suggested common codes:
- `invalid_input`
- `unknown_tool`
- `unknown_provider`
- `provider_unavailable`
- `tool_disabled`
- `confirmation_required`
- `permission_denied`
- `timeout`
- `not_found`
- `conflict`
- `unsupported`
- `internal_error`

Providers may emit provider-specific subcodes, but the top-level Lonis vocabulary should remain stable.

---

## 12. Input and Output Schemas

v0 does not require a specific schema language beyond something Lonis can inspect and emit reliably.

Practical options:
- JSON Schema
- a Lonis-native schema IR with JSON Schema export

### Recommendation for v0
Use a schema system that can be emitted as JSON cleanly, since AI consumers and integration layers benefit from JSON-friendly contracts.

EDN or p-expressions may still appear in payload content, but tool contract discovery should remain maximally compatible.

---

## 13. Capability, Safety, and Policy Model

Because tools are intended for AI use, Lonis should make capability and safety metadata explicit.

## 13.1 Capability examples
- `read_file`
- `write_file`
- `read_external_application`
- `mutates_external_application`
- `network_access`
- `model_inference`
- `artifact_generation`

## 13.2 Safety examples
- `destructive`
- `requires_confirmation`
- `sandbox_only`
- `shared_resource_risk`

## 13.3 Policy behavior
Lonis should be able to:
- hide tools that violate a profile/policy
- refuse invocations that violate policy
- explain policy blocks in structured error form

This is strongly supported by the Kai Jyozu experience.

---

## 14. Provider Lifecycle

External providers should not require MCP-style always-on presence by default.

### Preferred v0 model
Lonis invokes an external provider explicitly when needed.

That provider may internally:
- execute directly and exit
- contact an existing local bridge/service
- manage its own session state

But from Lonis's perspective, the invocation remains explicit and local.

### Why
This keeps Lonis aligned with its core philosophy:
- no ambient server presence by default
- no unnecessary configuration burden
- explicit execution boundaries

---

## 15. Provider Interface Modes

For external providers, Lonis likely needs a minimal executable contract.

A plausible v0 executable-provider interface:

### Discovery
```bash
provider-binary manifest
provider-binary tools list
provider-binary tools describe <tool>
```

### Invocation
```bash
provider-binary call <tool>
```

with JSON request/response bodies.

Lonis can then wrap and normalize the provider output.

This mirrors the Lonis model and makes provider implementation conceptually simple.

---

## 16. Artifacts and Explicit State

Some tools will need reusable outputs.

Examples:
- Perceptron analysis artifacts
- Figma session/channel handles
- exported assets
- generated reports

Lonis should support explicit artifact references in envelopes.

Example:

```json
{
  "ok": true,
  "tool": "perceptron.analyze",
  "result": {
    "content_type": "application/edn",
    "value": "(...)"
  },
  "meta": {
    "artifacts": [
      {
        "id": "artifact_abc",
        "type": "perceptron.analysis",
        "reusable": true
      }
    ]
  }
}
```

Any state reuse should be explicit.

Avoid hidden session magic.

---

## 17. Diagnostics Model

Diagnostics should exist at both harness and provider levels.

## 17.1 Harness diagnostics
- installed/discovered providers
- tool registry summary
- profile/policy state
- environment/config issues

## 17.2 Provider diagnostics
- provider availability
- bridge/service connectivity
- version compatibility
- health checks
- recent errors or warnings

Kai Jyozu's `status` and `doctor` strongly suggest these are necessary, not optional.

---

## 18. Figma Provider as Reference Integration

The Figma surface is likely the best first provider to ground the Lonis design because it has:

- a large tool surface
- clear read vs write distinctions
- meaningful safety policy
- mutation against an external live application
- a need for diagnostics
- a need for verification state

The Kai Jyozu work suggests Lonis should preserve and formalize patterns like:
- tool verification states
- destructive gating metadata
- doctor/status commands
- compatibility/operational reporting

This makes Figma an excellent testbed for the general provider contract.

---

## 19. Perceptron Provider as Contrast Case

Perceptron is an equally useful reference case because it differs from Figma in important ways.

Likely characteristics:
- model inference
- artifact-heavy outputs
- EDN/p-expression payloads
- iterative follow-up calls
- lower mutation risk but higher compute/latency concerns

Together, Figma and Perceptron define a useful design envelope:
- external mutable application provider
- external perceptual/model provider

If Lonis can serve both cleanly, the provider model is probably sound.

---

## 20. Proposed v0 Scope

v0 should define and implement only what is necessary to prove the concept.

### In scope
- provider discovery
- tool discovery
- canonical `lonis call <tool>` flow
- standard result/error envelopes
- tool metadata and schema exposure
- capability/safety metadata
- verification status fields
- harness/provider diagnostics basics
- one or two real providers

### Out of scope
- remote provider registry
- package manager
- dynamic plugin marketplace
- daemon-first architecture
- distributed transport layers
- hidden session orchestration

---

## 21. Open Questions

These should be settled in the next pass.

### 21.1 Envelope format
Should Lonis envelopes always be JSON, even if tool payloads are EDN or binary references?

### 21.2 Exit semantics
Should valid tool-level failures return exit code 0 with `ok=false`, or nonzero exit codes?

### 21.3 Provider executable contract
Should external providers implement a standardized subcommand surface, or a stdin/stdout RPC mode, or both?

### 21.4 Schema representation
Should Lonis adopt JSON Schema directly, or define an internal schema IR that exports to JSON Schema?

### 21.5 Tool visibility
How should profiles/policies hide or expose tools by default?

### 21.6 Artifacts
Should artifact storage be owned by providers, Lonis, or be left implementation-defined in v0?

---

## 22. Recommended Immediate Next Step

The next productive move is to convert this spec into a concrete v0 design package containing:

1. example provider manifest
2. example tool contract JSON
3. example `lonis call` request/response examples
4. one Figma provider walkthrough
5. one Perceptron provider walkthrough

That would make the design concrete enough to validate before implementation starts.
