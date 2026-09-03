// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! Value-oracle tests for `lonis-identity` (plan contract §5, tests 1–5).

use lonis_identity::{IdentitySource, ParticipantId, ParticipantIdError};

#[test]
fn participant_id_parses_and_splits() {
    let persona = ParticipantId::new("persona:sara").unwrap();
    assert_eq!(persona.source(), "persona");
    assert_eq!(persona.rest(), "sara");

    let golden = ParticipantId::new("lonis:test:golden").unwrap();
    assert_eq!(golden.source(), "lonis");
    assert_eq!(golden.rest(), "test:golden");
}

#[test]
fn participant_id_rejects_invalid() {
    assert_eq!(
        ParticipantId::new("sara").unwrap_err(),
        ParticipantIdError::TooFewSegments
    );
    assert_eq!(
        ParticipantId::new(":sara").unwrap_err(),
        ParticipantIdError::EmptySegment
    );
    assert_eq!(
        ParticipantId::new("persona:").unwrap_err(),
        ParticipantIdError::EmptySegment
    );
    assert_eq!(
        ParticipantId::new("a::b").unwrap_err(),
        ParticipantIdError::EmptySegment
    );
}

#[test]
fn participant_id_serde_transparent() {
    let id = ParticipantId::new("persona:sara").unwrap();
    // Transparent serde: the id serializes as the bare string.
    assert_eq!(serde_json::to_string(&id).unwrap(), "\"persona:sara\"");
    // Round-trip from the bare string form.
    let back: ParticipantId = serde_json::from_str("\"persona:sara\"").unwrap();
    assert_eq!(back, id);
}

#[test]
fn display_and_from() {
    let id = ParticipantId::new("persona:sara").unwrap();
    // Display prints the full id.
    assert_eq!(format!("{id}"), "persona:sara");
    // From<&str> — no validation — round-trips through Display.
    let from_str: ParticipantId = ParticipantId::from("x:y");
    assert_eq!(format!("{from_str}"), "x:y");
    // From<String> — no validation — same rule.
    let from_string: ParticipantId = ParticipantId::from(String::from("x:y"));
    assert_eq!(from_string.to_string(), from_str.to_string());
    // From<ParticipantId> for String recovers the id.
    let into_string: String = ParticipantId::new("persona:sara").unwrap().into();
    assert_eq!(into_string, "persona:sara");
}

/// A test registry owning the `persona` source.
struct VecRegistry {
    ids: Vec<String>,
}

impl VecRegistry {
    fn new(ids: &[&str]) -> Self {
        Self {
            ids: ids.iter().map(|id| (*id).to_owned()).collect(),
        }
    }
}

impl IdentitySource for VecRegistry {
    fn owns(&self, source: &str) -> bool {
        source == "persona"
    }

    fn validate(&self, id: &ParticipantId) -> Result<(), String> {
        if self.ids.iter().any(|known| known == id.as_str()) {
            Ok(())
        } else {
            Err(format!("unknown participant: {}", id.as_str()))
        }
    }
}

#[test]
fn identity_source_trait() {
    let registry = VecRegistry::new(&["persona:sara"]);
    assert!(registry.owns("persona"));
    assert!(!registry.owns("agent"));

    let known = ParticipantId::new("persona:sara").unwrap();
    assert!(registry.validate(&known).is_ok());

    let unknown = ParticipantId::new("persona:nobody").unwrap();
    let err = registry.validate(&unknown).unwrap_err();
    assert!(err.starts_with("unknown participant:"));
    assert!(err.contains("persona:nobody"));
}
