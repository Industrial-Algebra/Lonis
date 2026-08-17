// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! Composes `block-v1.json` (the envelope schema) from the per-kind curated
//! schema files — the Rust replacement for the retired Python composer.
//! Per-kind files are the source of truth; run this after editing them:
//!
//! ```sh
//! cargo run -p lonis-schema --example compose_block_schema
//! ```
//!
//! The composition: each per-kind document (minus `$schema`/`$id`/marker/
//! `title`) becomes a `$defs` entry; nested per-kind `$defs` (e.g.
//! `plan_step`, `evidence_data`) are hoisted into the envelope's `$defs`
//! with collision detection; `payload` becomes a `oneOf` over all kinds.

use std::path::PathBuf;

use serde_json::{json, Map, Value};

fn schemas_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("schemas")
}

fn payload_kinds() -> Vec<String> {
    // Discover per-kind files from the directory (`<kind>-v1.json`, excluding
    // the composed `block-v1.json`) — robust across contract versions.
    let mut kinds: Vec<String> = std::fs::read_dir(schemas_dir())
        .unwrap()
        .filter_map(|entry| {
            let name = entry.ok()?.file_name().into_string().ok()?;
            name.strip_suffix("-v1.json")
                .filter(|kind| *kind != "block")
                .map(str::to_owned)
        })
        .collect();
    // Present kinds in doctrine order (stream / knowledge / process, then
    // the extension seam); unknown future kinds sort after, alphabetically.
    const DOCTRINE_ORDER: [&str; 15] = [
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
    ];
    let rank = |kind: &String| {
        DOCTRINE_ORDER
            .iter()
            .position(|k| k == kind)
            .unwrap_or(DOCTRINE_ORDER.len())
    };
    kinds.sort_by(|a, b| rank(a).cmp(&rank(b)).then_with(|| a.cmp(b)));
    kinds
}

fn read_payload_docs(kinds: &[String]) -> Map<String, Value> {
    let mut defs = Map::new();
    for kind in kinds {
        let text = std::fs::read_to_string(schemas_dir().join(format!("{kind}-v1.json")))
            .unwrap_or_else(|err| panic!("missing {kind}-v1.json: {err}"));
        let mut doc: Value = serde_json::from_str(&text).unwrap();
        let obj = doc.as_object_mut().unwrap();
        for meta in ["$schema", "$id", "x-lonis-protocol-version", "title"] {
            obj.remove(meta);
        }
        defs.insert(kind.clone(), doc);
    }
    defs
}

/// Hoist nested per-kind `$defs` into the envelope's `$defs` so their
/// `#/$defs/...` refs resolve against this document; conflicting redefinitions
/// are a hard error.
fn hoist_nested_defs(kinds: &[String], defs: &mut Map<String, Value>) {
    let mut hoisted: Vec<(String, Value)> = Vec::new();
    for kind in kinds {
        if let Some(Value::Object(nested)) =
            defs[kind.as_str()].as_object().and_then(|d| d.get("$defs"))
        {
            for (name, sub) in nested {
                if let Some(existing) = defs.get(name) {
                    assert_eq!(existing, sub, "$defs collision on `{name}`");
                } else if let Some((_, existing)) = hoisted.iter().find(|(n, _)| n == name) {
                    assert_eq!(existing, sub, "$defs collision on `{name}`");
                } else {
                    hoisted.push((name.clone(), sub.clone()));
                }
            }
        }
    }
    for kind in kinds {
        if let Some(obj) = defs.get_mut(kind.as_str()).and_then(Value::as_object_mut) {
            obj.remove("$defs");
        }
    }
    for (name, sub) in hoisted {
        defs.insert(name, sub);
    }
}

fn str_len(max: u64) -> Value {
    json!({"type": "string", "maxLength": max})
}

fn arr(items: Value, max: u64) -> Value {
    json!({"type": "array", "maxItems": max, "items": items})
}

fn envelope(kinds: &[String], defs: Map<String, Value>) -> Value {
    let one_of: Vec<Value> = kinds
        .iter()
        .map(|kind| json!({"$ref": format!("#/$defs/{kind}")}))
        .collect();
    json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "$id": "https://industrialalgebra.com/schemas/lonis.block/v1",
        "x-lonis-protocol-version": "lonis.block/v1",
        "title": "lonis.block envelope",
        "type": "object",
        "additionalProperties": false,
        "required": ["schema_version", "attribution", "payload"],
        "properties": {
            "schema_version": {"const": "lonis.block/v1"},
            "provenance": {
                "type": "object",
                "additionalProperties": false,
                "properties": {
                    "tool_version": str_len(256),
                    "compatibility": {
                        "type": "object",
                        "additionalProperties": false,
                        "required": ["status"],
                        "properties": {
                            "status": str_len(256),
                            "reasons": arr(str_len(4096), 256),
                        },
                    },
                    "replay": {
                        "type": "object",
                        "additionalProperties": false,
                        "required": ["replayable"],
                        "properties": {
                            "replayable": {"type": "boolean"},
                            "required_hashes": arr(str_len(256), 64),
                            "reasons": arr(str_len(4096), 256),
                        },
                    },
                    "project_hash": str_len(128),
                    "input_hash": str_len(128),
                    "plan_hash": str_len(128),
                    "result_hash": str_len(128),
                    "seed": {"type": "integer", "minimum": 0},
                },
            },
            "warnings": arr(str_len(4096), 256),
            "attribution": {
                "type": "object",
                "additionalProperties": false,
                "required": ["identity", "provenance"],
                "properties": {
                    "identity": str_len(1024),
                    "viewpoint": str_len(1024),
                    "provenance": {
                        "type": "object",
                        "additionalProperties": false,
                        "required": ["when", "producer"],
                        "properties": {
                            "when": str_len(64),
                            "where": str_len(4096),
                            "producer": str_len(1024),
                        },
                    },
                },
            },
            "bounds": {
                "type": "object",
                "additionalProperties": false,
                "properties": {
                    "max_items": {"type": "integer", "minimum": 0},
                    "max_bytes": {"type": "integer", "minimum": 0},
                    "max_length": {"type": "integer", "minimum": 0},
                    "timeout_millis": {"type": "integer", "minimum": 0},
                },
            },
            "payload": {"oneOf": one_of},
        },
        "$defs": defs,
    })
}

fn main() {
    let kinds = payload_kinds();
    let mut defs = read_payload_docs(&kinds);
    hoist_nested_defs(&kinds, &mut defs);
    let document = envelope(&kinds, defs);
    let path = schemas_dir().join("block-v1.json");
    let mut bytes = serde_json::to_vec_pretty(&document).unwrap();
    bytes.push(b'\n');
    std::fs::write(&path, bytes).unwrap();
    println!("wrote {} ({} kinds)", path.display(), kinds.len());
}
