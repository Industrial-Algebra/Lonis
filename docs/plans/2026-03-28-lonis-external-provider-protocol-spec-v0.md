# Lonis External Provider Protocol Specification v0

## Status

This document defines a first-pass protocol for **external executable providers** that integrate with the Lonis harness.

It builds on:
- `2026-03-28-lonis-harness-design-draft.md`
- `2026-03-28-lonis-provider-interface-spec-v0.md`

The goal is to make separate projects such as **Perceptron** and a future **Figma tool surface** consumable by Lonis through a stable, explicit, local-first contract.

---

## 1. Purpose

An external provider protocol is needed so that Lonis can:

- consume tools from separate repositories/projects
- normalize discovery and invocation
- avoid MCP-style ambient server complexity
- preserve explicit process boundaries
- keep Lonis itself strictly focused on being the harness

The protocol should be:
- local-first
- machine-readable
- explicit
- easy to implement in any language
- robust under AI-driven invocation

---

## 2. Design Principles

## 2.1 Explicit over ambient
Providers should be invoked deliberately by Lonis rather than exposed as always-on ambient servers by default.

## 2.2 Machine-first
The protocol should optimize for structured parsing, not human ergonomics.

## 2.3 Minimal surface area
The smallest viable provider protocol is preferable to a broad RPC system.

## 2.4 Process boundary as feature
External execution is not a compromise. It is a useful architectural boundary that:
- keeps projects separate
- clarifies ownership
- limits failure scope
- simplifies integration

## 2.5 Deterministic I/O discipline
- stdout should carry the protocol payload
- stderr should carry diagnostics/logging only
- exit codes should have narrow, stable meaning

---

## 3. Provider Modes

v0 should support **one required mode** and **one optional mode**.

## 3.1 Required mode: subcommand mode
Providers are executables exposing standardized subcommands.

Example:

```bash
perceptron-provider manifest
perceptron-provider tools list
perceptron-provider tools describe perceptron.analyze
perceptron-provider call perceptron.analyze
```

This should be the canonical v0 provider interface.

## 3.2 Optional mode: stream mode
A future optimization may allow persistent stdin/stdout request handling for expensive startup providers.

This is **out of scope for required v0 behavior**, but the spec should not preclude it later.

---

## 4. Required Provider Commands

Every external provider must implement the following command surface.

## 4.1 `manifest`
Returns provider-level metadata.

Example:

```bash
figma-provider manifest
```

Response: JSON object on stdout.

## 4.2 `tools list`
Returns the provider's available tools.

Example:

```bash
figma-provider tools list
```

Response: JSON array or object on stdout.

## 4.3 `tools describe <tool>`
Returns the full contract for a tool.

Example:

```bash
figma-provider tools describe figma.create_frame
```

Response: JSON object on stdout.

## 4.4 `call <tool>`
Invokes a tool.

Example:

```bash
figma-provider call figma.create_frame
```

Request should be passed via stdin in canonical machine mode.

Response: JSON success or error payload on stdout.

## 4.5 Optional diagnostics
Strongly recommended for serious providers:

```bash
figma-provider status
figma-provider doctor
```

These are especially relevant for providers with bridges, sessions, local services, or external application dependencies.

---

## 5. Canonical Wire Format

For v0, the required wire format should be **JSON**.

Reasoning:
- widely available
- easy to parse in all languages
- easy for AI systems and wrappers to inspect
- works well for envelopes and schemas

Domain-specific payloads may still contain:
- EDN strings
- p-expressions
- artifact references
- base64 data

But the provider protocol itself should use JSON.

---

## 6. Provider Manifest Shape

Required output of `manifest`:

```json
{
  "name": "figma",
  "version": "0.1.0",
  "display_name": "Figma Tool Surface",
  "description": "External Lonis provider for interacting with a live Figma session",
  "provider_type": "external-executable",
  "protocol_version": "0",
  "runtime": {
    "language": "typescript",
    "entrypoint": "figma-provider"
  },
  "tools": [
    "figma.get_document",
    "figma.create_frame"
  ],
  "capabilities": [
    "diagnostics",
    "external_application",
    "mutable_operations"
  ]
}
```

## Required manifest fields
- `name`
- `version`
- `description`
- `provider_type`
- `protocol_version`
- `tools`

## Recommended manifest fields
- `display_name`
- `runtime`
- `capabilities`
- `supported_lonis_versions`
- `verification_summary`
- `homepage`
- `repository`

---

## 7. `tools list` Response Shape

Recommended output:

```json
{
  "provider": "figma",
  "tools": [
    {
      "name": "figma.get_document",
      "description": "Get metadata and page information for the current Figma document",
      "verification_status": "verified"
    },
    {
      "name": "figma.create_frame",
      "description": "Create a frame in the current Figma document",
      "verification_status": "verified"
    }
  ]
}
```

This should be compact enough for discovery while leaving full schema details to `tools describe`.

---

## 8. `tools describe` Response Shape

Recommended output:

```json
{
  "name": "figma.create_frame",
  "provider": "figma",
  "schema_version": "1",
  "description": "Create a Figma frame on the current page or under a specified parent",
  "input_schema": {
    "type": "object",
    "required": ["name", "x", "y", "width", "height"],
    "properties": {
      "name": { "type": "string" },
      "x": { "type": "number" },
      "y": { "type": "number" },
      "width": { "type": "number" },
      "height": { "type": "number" }
    }
  },
  "output_schema": {
    "type": "object",
    "required": ["node_id"],
    "properties": {
      "node_id": { "type": "string" },
      "name": { "type": "string" }
    }
  },
  "capabilities": ["mutates_external_application"],
  "side_effects": ["writes_figma_document"],
  "determinism": "deterministic",
  "cost": "low",
  "verification": {
    "status": "verified"
  },
  "safety": {
    "destructive": false,
    "requires_confirmation": false
  }
}
```

---

## 9. `call <tool>` Request Shape

Lonis should send a normalized request object to provider stdin.

Recommended request shape:

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
    "request_id": "req_123",
    "cwd": "/workspace/project",
    "profile": "default"
  }
}
```

## Request rules
- top-level `tool` required
- top-level `input` required, may be `{}`
- `context` optional
- unknown fields should be ignored unless provider chooses to validate strictly

---

## 10. `call <tool>` Response Shape

Providers should return a structured response on stdout.

## 10.1 Success

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

## 10.2 Tool-level error

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
    "duration_ms": 3,
    "warnings": []
  }
}
```

Providers should prefer returning structured tool-level errors over crashing.

---

## 11. Exit Code Semantics

This should be kept narrow and predictable.

## 11.1 Recommended rule
- `0`: command executed and returned a valid JSON payload on stdout
- nonzero: provider process/protocol failure

This means a tool-level failure may still exit `0` if it returned a valid envelope with `ok: false`.

This is recommended because it is much easier for Lonis and AI consumers to distinguish:
- protocol/process failure
- valid tool-level failure

## 11.2 Suggested nonzero meanings
- `1`: general provider failure
- `2`: invalid command usage
- `3`: protocol violation / invalid JSON output
- `4`: unsupported protocol version

Lonis should treat nonzero exits as provider-level execution failure unless explicitly documented otherwise.

---

## 12. stdout / stderr Rules

## stdout
- only JSON payloads
- no banners
- no human log lines
- no progress prose in machine mode

## stderr
- diagnostics
- debug logs
- warnings for human operators

If a provider wants human-oriented rendering, it should expose that through explicit non-machine modes, not the canonical machine path.

---

## 13. Error Taxonomy

Providers should align with Lonis core error codes where possible.

Recommended top-level codes:
- `invalid_input`
- `unknown_tool`
- `provider_unavailable`
- `timeout`
- `permission_denied`
- `confirmation_required`
- `not_found`
- `unsupported`
- `conflict`
- `internal_error`

Provider-specific subcodes may be included in `details`.

---

## 14. Diagnostics Commands

Providers with real runtime complexity should implement diagnostics.

## 14.1 `status`
Should answer questions like:
- is the provider installed and runnable?
- is required local infrastructure reachable?
- are sessions/connections active?
- are locks or cached artifacts present?

## 14.2 `doctor`
Should run deeper checks and report structured failures, especially for:
- missing dependencies
- version mismatches
- connectivity issues
- bridge/plugin availability
- malformed environment/config

Kai Jyozu strongly suggests that diagnostics are essential for real-world tool surfaces.

---

## 15. Verification Metadata

Providers should expose verification state at the tool level.

Recommended statuses:
- `experimental`
- `implemented`
- `verified`
- `deprecated`
- `disabled`

This metadata should appear in:
- `tools list`
- `tools describe`
- optional provider-level summaries

This allows Lonis to expose more trustworthy tool registries to AI consumers.

---

## 16. Artifact Handling

Some providers will emit reusable artifacts.

Examples:
- Perceptron analyses
- exports
- session handles
- snapshots

Providers may include artifact metadata in the `meta.artifacts` field.

Example:

```json
{
  "ok": true,
  "tool": "perceptron.analyze",
  "provider": "perceptron",
  "schema_version": "1",
  "result": {
    "content_type": "application/edn",
    "value": "(...)"
  },
  "meta": {
    "duration_ms": 182,
    "warnings": [],
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

Artifact persistence ownership may remain implementation-defined in v0.

---

## 17. Version Negotiation

v0 should remain simple.

Each provider manifest should declare:
- `protocol_version`

Lonis should reject incompatible providers explicitly.

Possible future enhancement:
- `supported_protocol_versions`

But v0 can remain single-version if needed.

---

## 18. Security and Trust Posture

Lonis should not assume providers are benign or side-effect free.

Providers should declare:
- side effects
- capabilities
- destructive status
- confirmation requirements

Lonis may later enforce policy based on those declarations.

v0 should at least preserve and surface this metadata clearly.

---

## 19. Example: Perceptron Provider Shape

Example command surface:

```bash
perceptron-provider manifest
perceptron-provider tools list
perceptron-provider tools describe perceptron.analyze
perceptron-provider call perceptron.analyze
perceptron-provider call perceptron.attend
```

Characteristics:
- model inference
- EDN/p-expression payloads
- artifact reuse
- lower mutation risk
- higher compute cost

---

## 20. Example: Figma Provider Shape

Example command surface:

```bash
figma-provider manifest
figma-provider tools list
figma-provider tools describe figma.get_document
figma-provider call figma.get_document
figma-provider status
figma-provider doctor
```

Characteristics:
- live external application integration
- mutable operations
- safety/confirmation rules
- diagnostics required
- verification metadata highly valuable

---

## 21. Non-Goals for v0

Not required initially:
- network-transparent remote providers
- dynamic plugin loading
- rich bidirectional streaming events
- embedded daemon/session manager in the protocol itself
- generalized transport negotiation

Keep the protocol simple first.

---

## 22. Recommended Next Step

The next implementation-oriented step should be to draft:

1. a sample provider manifest JSON file
2. a sample provider executable behavior contract test suite
3. one Perceptron provider example
4. one Figma provider example

That would turn this protocol draft into something executable and testable.
