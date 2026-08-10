# ADR-0004: Vertical payload authoring — `#[derive(BlockPayload)]` + protocol guide

- Status: accepted
- Date: 2026-08-10
- Branch: `feature/block-payload-derive`
- References: ADR-0002 (typed payloads, the erased seam), ADR-0003 (subprocess
  protocol); karpal-discovery subprocess-hosting spike
  (`karpal/docs/plans/2026-08-10-karpal-subprocess-spike.md`)

## Context

The karpal-discovery spike hosted a `karpal` binary through `SubprocessTool`
and validated the ADR-0002/0003 architecture end-to-end (lossless `Extension`
round-trip, verbatim `ToolError` propagation, optional git deps leaving the
default build publishable). It surfaced four requirements:

1. Vertical payload enums must serialize **adjacently tagged**
   (`{"kind", "data"}`) — internally-tagged enums hard-fail at the seam
   (`block payload requires 'data'`; verified independently).
2. The serde `kind` tag must equal `BlockPayload::kind_name()` and be
   namespaced — the host sees only the serde tag (it becomes
   `Extension.kind`), so hand-maintained divergence breaks the contract.
3. Cross-repo pre-publication deps must be git deps pinned to a rev, not
   path deps (worktree-relative breakage).
4. Tool targets must arrive via stdin input or argv — the subprocess
   environment is cleared and the cwd neutral *by design*.

## Decision

**1+2 → a derive; 3+4 → a guide.**

`#[derive(BlockPayload)]` (in `lonis-derive`, re-exported from `lonis-schema`
behind `derive`) generates, from one enum declaration:

- adjacently-tagged `{"kind", "data"}` `Serialize`/`Deserialize` impls
  (struct variants with named fields; unit variants carry `data: null`;
  tuple variants and generics are compile errors),
- `kind_name()` — snake_case variant names, dot-namespaced:
  `#[lonis_payload(namespace = "karpal")]` + `Search` → **`karpal.search`**
  (dots, matching the ecosystem's dotted protocol markers),
- `schema_id()` — `lonis.block/<kind>/v1`,
- `render_human()` — a `<kind>: <Debug>` default (the enum must be `Debug`;
  verticals wanting richer output hand-implement `BlockPayload`).

Because the serde tag and `kind_name()` come from one source, they cannot
diverge. Deserializing into the vertical's own enum **rejects unknown kinds**
— a vertical's universe is closed; cross-vertical tolerance lives host-side
in `BlockKind::Extension`, exactly where ADR-0002 put it.

Requirements 3+4 (plus the minimal block JSON shape for non-Rust producers,
bounds discipline, and text mapping) are documented in
`docs/guides/subprocess-tool-authoring.md`, with `mock_tool.rs` as the
reference implementation.

## Consequences

- Vertical authors get the spike's hard-won lessons as a compile-time
  guarantee instead of a runtime failure.
- `lonis-derive` gains `proc-macro2` (direct) for the derive implementation.
- The spike's underscore kind (`karpal_search`) migrates to dots
  (`karpal.search`) — trivial pre-release with one consumer.
- 8 new derive tests replicate the spike's `KarpalPayload` end-to-end,
  including the seam round-trip through `SeedBlock`.
