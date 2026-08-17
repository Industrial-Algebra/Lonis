# Lonis Figma Provider Mapping Draft

## Status

This document maps the lessons and structures from the existing **Kai Jyozu** Figma bridge into the emerging **Lonis provider model**.

It is not an implementation plan yet. It is a translation layer between:

- what already works in Kai Jyozu
- what the new Lonis harness expects from providers

The goal is to use the Figma surface as a concrete proving ground for Lonis design decisions.

---

## 1. Why Figma Should Be an Early Lonis Provider

Figma is one of the best early provider candidates because it exercises many of the hardest parts of the harness design:

- many tools
- read vs write distinctions
- destructive vs safe operations
- external application state
- bridge/session complexity
- diagnostics needs
- tool verification maturity
- structured inputs and outputs

If Lonis can represent the Figma surface cleanly, the core provider design is likely on the right track.

---

## 2. What Kai Jyozu Already Proves

The Kai Jyozu reference demonstrates several ideas that should directly inform the Lonis provider model.

## 2.1 Large tool surfaces need structure
Kai Jyozu has a broad verified tool surface spanning:
- reads
- drawing/creation
- styling
- layout
- text
- export/vector
- pages
- prototyping
- comments
- variables/tokens
- libraries

This suggests the Lonis registry needs to support:
- grouping
- namespacing
- maturity metadata
- clear discovery output

## 2.2 Verification state matters
Kai Jyozu explicitly distinguishes verified tool coverage and tracks compatibility.

For Lonis, this suggests every Figma tool should carry metadata such as:
- `experimental`
- `implemented`
- `verified`
- `deprecated`
- `disabled`

This is especially important for AI consumers, which should not assume that every exposed tool is equally reliable.

## 2.3 Safety metadata matters
Kai Jyozu already includes:
- destructive-tool startup gating
- confirmation requirements
- protected-page policies
- sandbox-only policies
- dry-run support
- audit logging

For Lonis, this strongly implies Figma tools should expose contract metadata like:
- `destructive`
- `requires_confirmation`
- `supports_dry_run`
- `shared_resource_risk`
- `policy_tags`

## 2.4 Diagnostics are first-class
Kai Jyozu includes:
- `status`
- `doctor`
- health/session APIs
- process locks
- audit visibility

This argues strongly that the Figma provider should expose provider-level diagnostics in Lonis.

## 2.5 External runtime complexity is normal
Kai Jyozu's architecture includes:
- MCP server
- bridge integration / WebSocket lifecycle
- Figma plugin UI/runtime
- channel/session state

Lonis should not try to erase this complexity. It should instead give it a disciplined provider shape.

---

## 3. High-Level Mapping

### Kai Jyozu today

```text
Agent
  -> MCP server
  -> bridge integration / WebSocket
  -> Figma plugin
  -> Figma canvas
```

### Lonis future

```text
AI consumer
  -> Lonis harness
  -> figma provider
  -> local bridge/session layer
  -> Figma plugin
  -> Figma canvas
```

Key shift:
- MCP server is no longer the primary AI-facing boundary
- the **Lonis Figma provider** becomes the AI-facing tool surface

---

## 4. Proposed Provider Identity

Suggested provider name:

```text
figma
```

Suggested executable identity:

```text
figma-provider
```

Suggested Lonis namespace:
- `figma.get_document`
- `figma.get_page`
- `figma.get_selection`
- `figma.create_frame`
- `figma.set_fill`
- `figma.delete_node`
- etc.

This preserves the application-focused grouping and avoids leaking implementation-specific bridge concepts into the AI-facing tool names.

---

## 5. Proposed Tool Taxonomy

A Lonis Figma provider should likely expose tools grouped along lines similar to Kai Jyozu's practical categories.

## 5.1 Connection/session tools
Examples:
- `figma.join_channel`
- `figma.get_connection_status`
- `figma.list_sessions`

These may or may not be AI-primary tools, but some session/connection visibility will likely be necessary.

## 5.2 Read/inspection tools
Examples:
- `figma.get_document`
- `figma.get_page`
- `figma.get_node`
- `figma.get_selection`
- `figma.find_nodes_by_name`

These are low-risk and should probably be safe-by-default.

## 5.3 Creation tools
Examples:
- `figma.create_frame`
- `figma.create_rectangle`
- `figma.create_ellipse`
- `figma.create_text`
- `figma.create_component`

## 5.4 Mutation tools
Examples:
- `figma.set_fill`
- `figma.resize_node`
- `figma.move_node`
- `figma.set_auto_layout`
- `figma.set_text_content`

## 5.5 Destructive tools
Examples:
- `figma.delete_node`
- `figma.delete_page`
- `figma.detach_instance` (depending on policy classification)

These should carry stronger metadata and policy hooks.

## 5.6 Export tools
Examples:
- `figma.export_node`
- `figma.export_selection`
- `figma.get_export_data`

## 5.7 Collaboration and system tools
Examples:
- `figma.create_comment`
- `figma.get_comments`
- `figma.enable_library`
- `figma.import_component`

---

## 6. Proposed Mapping of Kai Jyozu Concepts to Lonis Concepts

| Kai Jyozu concept | Lonis equivalent |
|---|---|
| MCP tool | Lonis tool contract |
| MCP server | Figma provider executable |
| tool compatibility matrix | verification metadata in registry/contracts |
| safety guardrails | tool safety metadata + policy enforcement |
| bridge status | `figma-provider status` / `lonis provider status figma` |
| doctor command | `figma-provider doctor` / `lonis provider doctor figma` |
| audit log | provider diagnostic/audit facility |
| shared-file protection | policy tags and confirmation semantics |
| OpenCode/Claude/Cursor formatting quirks | handled at Lonis harness/rendering boundary |

This is a crucial advantage of Lonis: it can normalize AI-facing output even if the underlying provider has application-specific complexity.

---

## 7. Tool Contract Recommendations for Figma

Figma tools should likely carry richer metadata than simpler providers because they interact with a live shared design environment.

Suggested required metadata beyond the generic Lonis minimum:
- `destructive`
- `requires_confirmation`
- `supports_dry_run`
- `shared_resource_risk`
- `policy_tags`
- `verification.status`

### Example: `figma.delete_node`

```json
{
  "name": "figma.delete_node",
  "provider": "figma",
  "schema_version": "1",
  "description": "Delete a node from the active Figma document",
  "capabilities": ["mutates_external_application"],
  "side_effects": ["writes_figma_document", "deletes_user_content"],
  "determinism": "deterministic",
  "cost": "low",
  "verification": {
    "status": "verified"
  },
  "safety": {
    "destructive": true,
    "requires_confirmation": true,
    "supports_dry_run": true,
    "policy_tags": ["shared_resource_risk"]
  }
}
```

---

## 8. Diagnostics Mapping

Kai Jyozu's diagnostics should map naturally into provider-level Lonis diagnostics.

## 8.1 `figma-provider status`
Should provide structured answers to questions like:
- is the provider runnable?
- is the local bridge reachable?
- is a plugin connected?
- are there active sessions/channels?
- is there a process lock?
- where is the audit log?

## 8.2 `figma-provider doctor`
Should check:
- bridge availability
- stale listeners
- lock inconsistencies
- config path mistakes
- plugin connection issues
- channel validity
- version compatibility

Lonis should then be able to surface these through:
- `lonis provider status figma`
- `lonis provider doctor figma`

---

## 9. Safety and Policy Mapping

Kai Jyozu already has practical safety mechanisms that should translate into the Lonis model, not be discarded.

## 9.1 Destructive startup gate
This may become either:
- provider-owned policy
- Lonis-owned policy
- or both

At minimum, the provider must surface the requirement explicitly.

## 9.2 Confirmation requirements
These should appear in tool schemas and metadata, not just prose docs.

## 9.3 Protected page policies
These suggest the need for policy tags or risk classes, for example:
- `production_page_risk`
- `library_page_risk`
- `shared_resource_risk`

## 9.4 Sandbox-only mode
This may be represented as:
- provider config
- Lonis profile/policy
- or both

The key point is that the AI-facing tool surface must not hide these constraints.

## 9.5 Audit logging
Even if audit storage remains provider-specific, Lonis should be able to report that audit exists and where to inspect it.

---

## 10. Suggested Envelope Patterns for Figma

Because Figma actions often have side effects, envelopes should be especially informative.

## 10.1 Safe read tool example

```json
{
  "ok": true,
  "tool": "figma.get_document",
  "provider": "figma",
  "schema_version": "1",
  "result": {
    "document_id": "abc",
    "name": "Marketing Site",
    "pages": [
      { "id": "1:1", "name": "Design" }
    ]
  },
  "meta": {
    "duration_ms": 12,
    "warnings": []
  }
}
```

## 10.2 Destructive refusal example

```json
{
  "ok": false,
  "tool": "figma.delete_node",
  "provider": "figma",
  "schema_version": "1",
  "error": {
    "code": "confirmation_required",
    "message": "Destructive action requires confirm=true",
    "details": {
      "policy": "destructive_tool_guard"
    }
  },
  "meta": {
    "duration_ms": 2,
    "warnings": []
  }
}
```

## 10.3 Policy block example

```json
{
  "ok": false,
  "tool": "figma.delete_page",
  "provider": "figma",
  "schema_version": "1",
  "error": {
    "code": "permission_denied",
    "message": "Operation blocked by protected-page policy",
    "details": {
      "page": "Production",
      "policy_tag": "production_page_risk"
    }
  },
  "meta": {
    "duration_ms": 3,
    "warnings": []
  }
}
```

---

## 11. Connection and Session Design Questions

Figma is the provider most likely to force clarity around explicit state.

Questions:
- Should channel/session join be explicit AI-visible tooling?
- Should the provider own session handles entirely?
- Should Lonis know about sessions at all, or just pass inputs through?

My current inclination:
- provider owns detailed session state
- Lonis should only need explicit handles when relevant
- tools that depend on a session should accept or infer a provider-defined session handle explicitly

Example:

```json
{
  "tool": "figma.get_document",
  "input": {
    "session_id": "figma_session_123"
  }
}
```

This avoids hidden ambient connection state while still respecting the reality of live Figma sessions.

---

## 12. Verification Mapping

Kai Jyozu's compatibility matrix implies Lonis may need a stronger registry view than just a flat tool list.

For the Figma provider, Lonis should ideally be able to show:
- total tools exposed
- tools by category
- tools by verification status
- tools blocked by policy/profile

This may be useful for:
- AI planning
- ops/debugging
- release management

---

## 13. Recommended First Figma Provider Slice

To validate the Lonis provider model, the initial Figma integration should probably be narrow but representative.

Suggested first slice:

### Read tools
- `figma.get_document`
- `figma.get_page`
- `figma.get_selection`
- `figma.get_node`

### Safe mutation tools
- `figma.create_frame`
- `figma.create_text`
- `figma.set_fill`
- `figma.move_node`

### One destructive tool
- `figma.delete_node`

### Diagnostics
- `figma-provider status`
- `figma-provider doctor`

This slice covers:
- read path
- write path
- destructive policy path
- diagnostics path

That is enough to pressure-test the Lonis contract meaningfully.

---

## 14. What Lonis Should Normalize vs What Figma Provider Should Own

## Lonis should normalize
- tool discovery shape
- invocation shape
- output/error envelope shape
- capability metadata fields
- verification metadata fields
- profile/policy visibility model

## Figma provider should own
- bridge mechanics
- plugin/runtime communication
- session/channel specifics
- Figma-domain schemas
- application-specific diagnostics
- audit log implementation

This separation keeps Lonis from becoming a domain-specific bridge runtime.

---

## 15. What Not to Port from Kai Jyozu Directly

Not everything should be carried over unchanged.

Lonis should not inherit:
- MCP as the primary AI boundary
- client-specific formatting quirks as core behavior
- protocol assumptions tied to MCP transport
- unnecessary server-centric patterns if explicit local invocation works better

Instead, Lonis should preserve the useful operational lessons while simplifying the AI-facing interface.

---

## 16. Strong Design Takeaway

The Figma provider should be understood as:

> A domain-specific operational surface with strong safety and diagnostics requirements, exposed through Lonis's normalized AI-native tool contracts.

This is exactly the kind of provider that can prove whether Lonis is a real harness rather than just a thin wrapper.

---

## 17. Recommended Next Step

The next useful move would be to draft a concrete Figma provider contract package containing:

1. provider manifest example
2. sample contracts for 5–10 representative tools
3. `status` and `doctor` response schemas
4. safety metadata conventions for destructive tools
5. one end-to-end invocation example using the Lonis envelope

That would make the mapping concrete enough to validate against the real Kai Jyozu behavior.
