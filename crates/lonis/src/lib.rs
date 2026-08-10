// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! # lonis
//!
//! The Lonis umbrella facade: one crate for consumers, re-exporting
//! [`lonis_schema`] (the contract), `lonis_derive` (the
//! `LonisCapabilities` derive, behind `derive`), and [`lonis_core`] (the
//! harness runtime, behind `core`, on by default) — serde-style.
//!
//! ```rust
//! use lonis::block::kinds::{BlockKind, Message};
//! use lonis::{Attribution, SeedBlock};
//!
//! let block = SeedBlock::new(
//!     Attribution::new("dominic", "lonis:builtin:echo"),
//!     BlockKind::Message(Message {
//!         role: Some("agent".into()),
//!         content: "hello from the facade".into(),
//!         reply_to: None,
//!     }),
//! );
//! assert_eq!(block.schema_id(), "lonis.block/message/v1");
//! ```
//!
//! ## Features
//!
//! - `core` (default): the harness runtime ([`Tool`], [`ToolRegistry`],
//!   [`SubprocessTool`], [`render`], [`run_tool`]).
//! - `derive`: the `LonisCapabilities` derive macro (re-exported from
//!   `lonis-schema`, serde-style).

#![forbid(unsafe_code)]

pub use lonis_schema::block;
pub use lonis_schema::block::kinds::{BlockCategory, BlockKind};
pub use lonis_schema::{
    exit_code, json_content_hash, now_rfc3339, Attribution, AttributionSource, Block, BlockBounds,
    BlockPayload, Capabilities, Compatibility, Cost, Determinism, OutputMode, ReplayMetadata,
    ReplayProvenance, SchemaRef, SchemaVersion, SchemaVersionError, SeedBlock, SideEffects,
    ToolContract, ToolError, ToolId, ToolIdError, BLOCK_SCHEMA_V1, TOOL_PROTOCOL_V1,
};

#[cfg(feature = "derive")]
pub use lonis_schema::LonisCapabilities;

#[cfg(feature = "core")]
pub use lonis_core::{
    render, render_error, run_tool, Availability, StdoutMapping, SubprocessTool, Tool, ToolRegistry,
};
