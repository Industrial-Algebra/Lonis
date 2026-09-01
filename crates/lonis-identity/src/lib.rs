// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! # lonis-identity
//!
//! Namespaced participant identities and the validation seam that owns them.
//!
//! Lonis is an AI-native tool harness: blocks are attributed to *participants*
//! (the personas, agents, tools, and humans speaking into a stream), and a
//! participant identity must be speakable — a registry-owned, validated
//! `source:slug` string, not an arbitrary blob (doctrine §2.7 attribution).
//!
//! This crate provides the [`ParticipantId`] newtype (wire-transparent: it
//! serializes as the identical bare string, so the 0.1 wire form is
//! untouched) and the [`IdentitySource`] trait — the validation seam that
//! registry owners (Dominic's persona registry, Tsume's worn-persona set,
//! Wallace's participant registry) implement for the sources they own. See
//! `docs/plans/2026-08-29-v0.2-typed-seam.md` and ADR-0010 for the design.

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

/// Namespaced participant identity (`source:slug`, e.g. `persona:sara`,
/// `agent:pi`, `lonis:test:golden`). Mirrors `ToolId`'s validation rules.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ParticipantId(String);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParticipantIdError {
    TooFewSegments,
    EmptySegment,
}

impl ParticipantId {
    /// Parse and validate: at least two non-empty `:`-separated segments
    /// (identical rule to `ToolId::new`).
    // `#[must_use]` is contract-mandated (verbatim signature §4.1); clippy
    // flags it as redundant on `Result`, so silence the lint locally.
    #[allow(clippy::double_must_use)]
    #[must_use]
    pub fn new(id: impl Into<String>) -> Result<Self, ParticipantIdError> {
        let id = id.into();
        let parts: Vec<&str> = id.split(':').collect();
        if parts.len() < 2 {
            return Err(ParticipantIdError::TooFewSegments);
        }
        if parts.iter().any(|p| p.is_empty()) {
            return Err(ParticipantIdError::EmptySegment);
        }
        Ok(Self(id))
    }

    /// The full id as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// The registry/source that owns this id — the first `:`-separated segment.
    /// `"lonis:test:golden"` → `"lonis"`, `"persona:sara"` → `"persona"`.
    #[must_use]
    pub fn source(&self) -> &str {
        self.0.split(':').next().unwrap_or("")
    }

    /// Everything after the first colon, free-form:
    /// `"lonis:test:golden"` → `"test:golden"`.
    #[must_use]
    pub fn rest(&self) -> &str {
        self.0.split_once(':').map(|(_, rest)| rest).unwrap_or("")
    }
}

impl std::fmt::Display for ParticipantId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<ParticipantId> for String {
    fn from(id: ParticipantId) -> String {
        id.0
    }
}

impl From<String> for ParticipantId {
    /// NO validation — parsing trust boundary is `new()`.
    fn from(id: String) -> Self {
        Self(id)
    }
}

impl From<&str> for ParticipantId {
    /// NO validation, same rule.
    fn from(id: &str) -> Self {
        Self(id.to_owned())
    }
}

/// A registry that can validate participant ids for the source(s) it owns.
/// Implemented by the Dominic persona registry, Tsume's worn-persona set,
/// and Wallace's participant registry — each for the sources it owns.
pub trait IdentitySource {
    /// Whether this source is authoritative for the given namespace prefix.
    #[must_use]
    fn owns(&self, source: &str) -> bool;
    /// Validate that `id` exists and is speakable under this source.
    /// Returns `Ok(())` or a human-readable reason.
    fn validate(&self, id: &ParticipantId) -> Result<(), String>;
}

/// Source prefixes reserved by the Lonis ecosystem (documented convention;
/// see ADR-0010 §"reserved sources").
pub const RESERVED_SOURCES: &[&str] = &["persona", "agent", "tool", "human", "lonis"];
