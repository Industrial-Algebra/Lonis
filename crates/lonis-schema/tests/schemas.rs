// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! Schema emission + golden wire fixture tests.
//!
//! Two interlocking pins (ADR-0005):
//! - the curated JSON Schemas (draft 2020-12) are emitted for the envelope
//!   and all 14 seed kinds, are valid, and carry stable `$id`s;
//! - one golden block instance per kind (checked in under `tests/golden/`)
//!   parses as a `SeedBlock`, validates against the envelope schema and its
//!   per-kind schema, and keeps a pinned content hash — so the wire shape
//!   and the canonicalization cannot drift silently.

use std::path::PathBuf;

use lonis_schema::block::schemas::{block_schema, block_schema_catalog, BlockSchemaKind};
use lonis_schema::SeedBlock;

fn golden_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/golden/blocks")
}

fn golden(kind: &str) -> serde_json::Value {
    let path = golden_dir().join(format!("{kind}.json"));
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("missing golden fixture {}: {err}", path.display()));
    serde_json::from_str(&text).unwrap()
}

#[test]
fn catalog_covers_envelope_plus_fourteen_kinds_and_the_extension_seam() {
    let catalog = block_schema_catalog().unwrap();
    assert_eq!(catalog.schemas.len(), 16);
    assert_eq!(catalog.schemas[0].kind, BlockSchemaKind::Block);
    // Deterministic order, all kinds present.
    let names: Vec<_> = catalog.schemas.iter().map(|s| s.kind.as_str()).collect();
    for kind in [
        "block",
        "message",
        "question",
        "answer",
        "decision",
        "action",
        "assumption",
        "summary",
        "evidence",
        "definition",
        "capability",
        "intent",
        "plan",
        "result",
        "outcome",
        "extension",
    ] {
        assert!(names.contains(&kind), "catalog missing `{kind}`");
    }
}

#[test]
fn every_schema_is_valid_and_identified() {
    for kind in BlockSchemaKind::ALL {
        let schema = block_schema(kind).unwrap();
        assert!(
            schema
                .id
                .starts_with("https://industrialalgebra.com/schemas/lonis.block/"),
            "bad $id {}",
            schema.id
        );
        assert!(schema.id.ends_with("/v1"));
        assert_eq!(schema.protocol_version, "lonis.block/v1");
        // The document compiles as a draft 2020-12 schema.
        jsonschema::validator_for(&schema.document)
            .unwrap_or_else(|err| panic!("schema {} does not compile: {err}", kind.as_str()));
    }
}

#[test]
fn envelope_schema_id_is_the_block_contract_root() {
    let schema = block_schema(BlockSchemaKind::Block).unwrap();
    assert_eq!(
        schema.id,
        "https://industrialalgebra.com/schemas/lonis.block/v1"
    );
}

#[test]
fn kind_schema_ids_match_schema_id_convention() {
    let schema = block_schema(BlockSchemaKind::Message).unwrap();
    assert_eq!(
        schema.id,
        "https://industrialalgebra.com/schemas/lonis.block/message/v1"
    );
}

#[test]
fn golden_instances_parse_as_seed_blocks() {
    for kind in BlockSchemaKind::ALL {
        if kind == BlockSchemaKind::Block {
            continue;
        }
        let wire = golden(kind.as_str());
        let block: SeedBlock = serde_json::from_value(wire.clone()).unwrap();
        if kind == BlockSchemaKind::Extension {
            // The extension golden carries a vertical kind tag.
            assert_eq!(
                block.payload().category(),
                lonis_schema::BlockCategory::Extension
            );
        } else {
            assert_eq!(block.payload().kind_name(), kind.as_str());
        }
        // Semantic re-serialization identity (the wire pin).
        assert_eq!(serde_json::to_value(&block).unwrap(), wire);
    }
}

#[test]
fn golden_instances_validate_against_the_envelope_schema() {
    let envelope = block_schema(BlockSchemaKind::Block).unwrap();
    let validator = jsonschema::validator_for(&envelope.document).unwrap();
    for kind in BlockSchemaKind::ALL {
        if kind == BlockSchemaKind::Block {
            continue;
        }
        let wire = golden(kind.as_str());
        validator
            .validate(&wire)
            .unwrap_or_else(|err| panic!("golden {} fails envelope schema: {err}", kind.as_str()));
    }
}

#[test]
fn golden_payloads_validate_against_their_kind_schema() {
    for kind in BlockSchemaKind::ALL {
        if kind == BlockSchemaKind::Block {
            continue;
        }
        let wire = golden(kind.as_str());
        let schema = block_schema(kind).unwrap();
        let validator = jsonschema::validator_for(&schema.document).unwrap();
        validator
            .validate(&wire["payload"])
            .unwrap_or_else(|err| panic!("golden {} fails its kind schema: {err}", kind.as_str()));
    }
}

#[test]
fn golden_content_hashes_are_pinned() {
    // Canonicalization stability pin: if canonical_json_string ever changes
    // behavior, these fail loudly instead of silently altering replay hashes.
    let pins_path = golden_dir().join("hashes.json");
    let text = std::fs::read_to_string(&pins_path).expect("missing golden hashes.json");
    let recorded: serde_json::Map<String, serde_json::Value> = serde_json::from_str(&text).unwrap();
    assert_eq!(recorded.len(), 15);
    for kind in BlockSchemaKind::ALL {
        if kind == BlockSchemaKind::Block {
            continue;
        }
        let wire = golden(kind.as_str());
        let block: SeedBlock = serde_json::from_value(wire).unwrap();
        let expected = recorded[kind.as_str()].as_str().unwrap();
        assert_eq!(
            block.content_hash(),
            expected,
            "content hash drift for {}",
            kind.as_str()
        );
    }
}

#[test]
fn envelope_schema_rejects_unknown_top_level_fields() {
    let envelope = block_schema(BlockSchemaKind::Block).unwrap();
    let validator = jsonschema::validator_for(&envelope.document).unwrap();
    let mut wire = golden("message");
    wire.as_object_mut()
        .unwrap()
        .insert("bogus".into(), serde_json::json!(1));
    assert!(validator.validate(&wire).is_err());
}

#[test]
fn kind_schema_rejects_wrong_kind_const() {
    let schema = block_schema(BlockSchemaKind::Message).unwrap();
    let validator = jsonschema::validator_for(&schema.document).unwrap();
    let mut payload = golden("question")["payload"].clone();
    payload.as_object_mut().unwrap()["kind"] = serde_json::json!("message");
    // question's data under message's kind const: the const alone must fail…
    // (data shape aside, the tag must match the schema's kind)
    assert!(validator.validate(&payload).is_err());
}

// -- Issue #10: the envelope admits Extension payloads (the erased seam) --

#[test]
fn envelope_schema_admits_extension_payloads() {
    // The issue's repro: a vertical block is a valid SeedBlock on the wire
    // and must validate against the envelope schema.
    let wire = serde_json::json!({
        "schema_version": "lonis.block/v1",
        "attribution": {
            "identity": "karpal:discovery:search",
            "provenance": { "when": "2026-08-10T23:00:00Z", "producer": "karpal-discovery" }
        },
        "payload": { "kind": "karpal.search", "data": { "query": "Functor", "results": [] } }
    });
    let block: SeedBlock = serde_json::from_value(wire.clone()).unwrap();
    assert_eq!(block.payload().kind_name(), "karpal.search");
    let envelope = block_schema(BlockSchemaKind::Block).unwrap();
    let validator = jsonschema::validator_for(&envelope.document).unwrap();
    validator.validate(&wire).unwrap();
}

#[test]
fn envelope_schema_rejects_seed_shadowing_with_bad_data() {
    // A payload tagged as a seed kind but carrying invalid seed data must
    // not escape through the extension branch (the branches are disjoint).
    let mut wire = golden("message");
    let payload = wire["payload"].as_object_mut().unwrap();
    payload["data"] = serde_json::json!({"not_content": 1});
    let envelope = block_schema(BlockSchemaKind::Block).unwrap();
    let validator = jsonschema::validator_for(&envelope.document).unwrap();
    assert!(validator.validate(&wire).is_err());
}

#[test]
fn extension_golden_validates_and_hashes() {
    let wire = golden("extension");
    let block: SeedBlock = serde_json::from_value(wire.clone()).unwrap();
    let envelope = block_schema(BlockSchemaKind::Block).unwrap();
    jsonschema::validator_for(&envelope.document)
        .unwrap()
        .validate(&wire)
        .unwrap();
    let schema = block_schema(BlockSchemaKind::Extension).unwrap();
    jsonschema::validator_for(&schema.document)
        .unwrap()
        .validate(&wire["payload"])
        .unwrap();
    let _ = block.content_hash();
}
