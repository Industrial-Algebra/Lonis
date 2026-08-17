# Feature Flags

Features are **additive only** — enabling a feature never removes API.

## lonis-schema

| Feature | Effect |
|---|---|
| `derive` | Re-exports the `LonisCapabilities` and `BlockPayload` derives from `lonis-derive` (serde-style) |

## lonis-core

| Feature | Effect |
|---|---|
| `futures` | `BlockStream::into_async()` — a runtime-agnostic `futures_core::Stream` bridge (adds `futures-core` + `futures-channel` only; never a runtime) |

## lonis (facade)

| Feature | Default | Effect |
|---|---|---|
| `core` | ✓ | Re-export the harness runtime (`Tool`, `ToolRegistry`, `SubprocessTool`, …) |
| `derive` | | Re-export the derives |
| `futures` | | Propagates `lonis-core/futures` |
