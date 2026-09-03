// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! v0.2 wire tests (plan contract §5, tests 6–13): the typed identity seam,
//! the verification tier (R2), and the no-match contract (R7).
//!
//! Test 6 is the wire-preservation proof: every golden fixture parses as a
//! `SeedBlock` and re-serializes byte-identically (mod trailing newline), so
//! the 0.1 wire generation is untouched by the 0.2 type changes.

use std::path::PathBuf;

use lonis_schema::{
    Attribution, Cost, Determinism, IdentitySource, NoMatchDiagnostic, ParticipantId, SchemaRef,
    ScoredNeighbor, SeedBlock, SideEffects, ToolContract, ToolId, Verification,
};

fn golden_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/golden/blocks")
}

/// The 15 golden block fixtures (one per seed kind). `hashes.json` in the
/// same directory is a content-hash pin, not a block, so it is not parsed.
const GOLDEN_BLOCK_KINDS: [&str; 15] = [
    "action",
    "answer",
    "assumption",
    "capability",
    "decision",
    "definition",
    "evidence",
    "extension",
    "intent",
    "message",
    "outcome",
    "plan",
    "question",
    "result",
    "summary",
];

#[test]
fn golden_fixtures_unchanged_and_round_trip() {
    for kind in GOLDEN_BLOCK_KINDS {
        let path = golden_dir().join(format!("{kind}.json"));
        let bytes = std::fs::read(&path)
            .unwrap_or_else(|err| panic!("missing golden fixture {}: {err}", path.display()));
        let block: SeedBlock = serde_json::from_slice(&bytes).unwrap_or_else(|err| {
            panic!("golden {kind}.json does not parse as a SeedBlock: {err}")
        });
        // Re-serialize the way the fixtures are written (pretty + trailing
        // newline) and assert byte identity — the wire-preservation proof.
        let mut re_serialized = serde_json::to_vec_pretty(&block).unwrap();
        re_serialized.push(b'\n');
        assert_eq!(
            re_serialized, bytes,
            "wire drift in golden fixture {kind}.json — v0.2 must not change the 0.1 wire form"
        );
    }
}

#[test]
fn attribution_identity_wire_identical() {
    let attr = Attribution::new("persona:sara", "tsume:discord");
    let json = serde_json::to_string(&attr).unwrap();
    // The typed ParticipantId renders as the bare JSON string — no object.
    assert!(json.contains("\"identity\":\"persona:sara\""));
    assert!(!json.contains("\"identity\":{"));
}

#[test]
fn attribution_new_accepts_str_and_string() {
    // Compile-time oracle: both &str and String identity/producer forms work.
    let from_str = Attribution::new("a:b", "p");
    let from_string = Attribution::new(String::from("a:b"), String::from("p"));
    assert_eq!(from_str.identity.as_str(), "a:b");
    assert_eq!(from_string.identity.as_str(), "a:b");
    assert_eq!(from_str.provenance.producer, "p");
    assert_eq!(from_string.provenance.producer, "p");
}

fn base_contract(verification: Option<Verification>) -> ToolContract {
    ToolContract {
        name: ToolId::new("lonis:builtin:echo").unwrap(),
        description: "echo input".into(),
        input_schema: SchemaRef("lonis.echo/input/v1".into()),
        output_schema: SchemaRef("lonis.echo/output/v1".into()),
        determinism: Determinism::Deterministic,
        side_effects: SideEffects::None,
        cost: Cost::Low,
        capabilities: Vec::new(),
        verification,
    }
}

#[test]
fn tool_contract_without_verification_matches_v01() {
    let contract = base_contract(None);
    let json = serde_json::to_string(&contract).unwrap();
    // Unset ⇒ the serialized form is identical to 0.1: no `verification` key.
    assert!(!json.contains("verification"));
    // The 0.1 field set round-trips unchanged.
    let back: ToolContract = serde_json::from_str(&json).unwrap();
    assert_eq!(back, contract);
    assert!(back.verification.is_none());
}

#[test]
fn verification_serde_all_tiers() {
    // Curated.
    assert_eq!(
        serde_json::to_string(&Verification::Curated).unwrap(),
        "{\"tier\":\"curated\"}"
    );
    let back: Verification = serde_json::from_str("{\"tier\":\"curated\"}").unwrap();
    assert_eq!(back, Verification::Curated);

    // AutoExtracted.
    let auto = Verification::AutoExtracted {
        source: "dominic:registry".into(),
        extracted_at: "2026-08-29T00:00:00Z".into(),
    };
    assert_eq!(
        serde_json::to_string(&auto).unwrap(),
        "{\"tier\":\"auto_extracted\",\"source\":\"dominic:registry\",\
         \"extracted_at\":\"2026-08-29T00:00:00Z\"}"
    );
    let back: Verification = serde_json::from_str(
        "{\"tier\":\"auto_extracted\",\"source\":\"dominic:registry\",\
         \"extracted_at\":\"2026-08-29T00:00:00Z\"}",
    )
    .unwrap();
    assert_eq!(back, auto);

    // Probed.
    let probed = Verification::Probed {
        evidence_hash: "cafe".into(),
    };
    assert_eq!(
        serde_json::to_string(&probed).unwrap(),
        "{\"tier\":\"probed\",\"evidence_hash\":\"cafe\"}"
    );
    let back: Verification =
        serde_json::from_str("{\"tier\":\"probed\",\"evidence_hash\":\"cafe\"}").unwrap();
    assert_eq!(back, probed);
}

#[test]
fn tool_contract_with_verification_round_trips() {
    let tiers = [
        Verification::Curated,
        Verification::AutoExtracted {
            source: "dominic:registry".into(),
            extracted_at: "2026-08-29T00:00:00Z".into(),
        },
        Verification::Probed {
            evidence_hash: "deadbeef".into(),
        },
    ];
    for tier in tiers {
        let contract = base_contract(Some(tier));
        let json = serde_json::to_string(&contract).unwrap();
        assert!(json.contains("verification"), "verification key missing");
        let back: ToolContract = serde_json::from_str(&json).unwrap();
        assert_eq!(back.verification, contract.verification);
        assert_eq!(back, contract);
    }
}

#[test]
fn no_match_diagnostic_serde() {
    // Populated `nearest` round-trips.
    let with_nearest = NoMatchDiagnostic {
        query: "Functor".into(),
        matched: 0,
        nearest: vec![ScoredNeighbor {
            identity: "schubert:cell_of:Functor".into(),
            score: 0.42,
            basis: "token_overlap".into(),
        }],
        diagnostic: "matched 0 concepts; closest by token overlap: schubert_cell_of".into(),
    };
    let json = serde_json::to_string(&with_nearest).unwrap();
    assert!(json.contains("\"nearest\""));
    assert!(json.contains("\"score\":0.42"));
    let back: NoMatchDiagnostic = serde_json::from_str(&json).unwrap();
    assert_eq!(back, with_nearest);

    // Empty `nearest` is omitted in the serialized form and round-trips.
    let empty = NoMatchDiagnostic {
        query: "Functoor".into(),
        matched: 0,
        nearest: Vec::new(),
        diagnostic: "matched 0 concepts".into(),
    };
    let json = serde_json::to_string(&empty).unwrap();
    assert!(!json.contains("nearest"));
    let back: NoMatchDiagnostic = serde_json::from_str(&json).unwrap();
    assert_eq!(back, empty);
    assert!(back.nearest.is_empty());
}

/// The `IdentitySource` trait resolves through the `lonis-schema` re-export.
struct TestRegistry;

impl IdentitySource for TestRegistry {
    fn owns(&self, source: &str) -> bool {
        source == "persona"
    }

    fn validate(&self, id: &ParticipantId) -> Result<(), String> {
        if self.owns(id.source()) {
            Ok(())
        } else {
            Err(format!("unknown participant: {}", id.as_str()))
        }
    }
}

#[test]
fn identity_source_reexported() {
    let registry = TestRegistry;
    assert!(registry.owns("persona"));
    assert!(!registry.owns("agent"));
    assert!(registry
        .validate(&ParticipantId::new("persona:sara").unwrap())
        .is_ok());
    assert!(registry
        .validate(&ParticipantId::new("agent:pi").unwrap())
        .is_err());
}
