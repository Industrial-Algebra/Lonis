// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! Functional tests for the `BlockPayload` derive (ADR-0004), exercised
//! through the `lonis_schema::BlockPayload` re-export — replicating the
//! karpal-discovery spike's `KarpalPayload` end-to-end, including the seam.

use lonis_schema::block::kinds::BlockKind;
use lonis_schema::{Attribution, Block, BlockPayload, SeedBlock};

#[derive(Debug, Clone, PartialEq, lonis_schema::BlockPayload)]
#[lonis_payload(namespace = "karpal")]
enum KarpalPayload {
    Search { results: Vec<String> },
    Capability { record: String },
    Ready,
}

#[test]
fn namespaced_kind_names() {
    let search = KarpalPayload::Search { results: vec![] };
    assert_eq!(search.kind_name(), "karpal.search");
    assert_eq!(
        KarpalPayload::Capability {
            record: String::new()
        }
        .kind_name(),
        "karpal.capability"
    );
    assert_eq!(KarpalPayload::Ready.kind_name(), "karpal.ready");
}

#[test]
fn schema_ids_carry_the_namespaced_kind() {
    let search = KarpalPayload::Search { results: vec![] };
    assert_eq!(search.schema_id(), "lonis.block/karpal.search/v1");
}

#[test]
fn wire_is_adjacently_tagged_kind_data() {
    let payload = KarpalPayload::Search {
        results: vec!["Functor".into()],
    };
    assert_eq!(
        serde_json::to_value(&payload).unwrap(),
        serde_json::json!({"kind": "karpal.search", "data": {"results": ["Functor"]}})
    );
    // Unit variants carry null data.
    assert_eq!(
        serde_json::to_value(&KarpalPayload::Ready).unwrap(),
        serde_json::json!({"kind": "karpal.ready", "data": null})
    );
}

#[test]
fn round_trip_every_variant() {
    for payload in [
        KarpalPayload::Search {
            results: vec!["Functor".into()],
        },
        KarpalPayload::Capability {
            record: "Proven".into(),
        },
        KarpalPayload::Ready,
    ] {
        let wire = serde_json::to_string(&payload).unwrap();
        let back: KarpalPayload = serde_json::from_str(&wire).unwrap();
        assert_eq!(back, payload);
    }
}

#[test]
fn unknown_kind_errors_on_the_verticals_own_enum() {
    let wire = serde_json::json!({"kind": "karpal.nope", "data": {}});
    assert!(serde_json::from_value::<KarpalPayload>(wire).is_err());
}

#[test]
fn serde_tag_and_kind_name_agree_through_the_seam() {
    // The spike's core assertion: a block carrying the vertical payload
    // parses host-side as a SeedBlock with a lossless Extension whose kind
    // IS the same namespaced tag the vertical reports in-process.
    let block = Block::new(
        Attribution::new("karpal:discovery:search", "karpal:discovery:search"),
        KarpalPayload::Search {
            results: vec!["Functor".into()],
        },
    );
    let wire = serde_json::to_string(&vec![block]).unwrap();
    let hosted: Vec<SeedBlock> = serde_json::from_str(&wire).unwrap();
    assert_eq!(hosted.len(), 1);
    let BlockKind::Extension { kind, data } = hosted[0].payload() else {
        panic!("vertical payload must land in Extension")
    };
    assert_eq!(kind, "karpal.search");
    assert_eq!(data["results"][0], "Functor");
}

#[test]
fn render_human_defaults_to_kind_plus_debug() {
    let payload = KarpalPayload::Ready;
    let rendered = payload.render_human();
    assert!(rendered.contains("karpal.ready"));
    assert!(rendered.contains("Ready"));
}

// -- no namespace: bare snake_case kind --

#[derive(Debug, Clone, PartialEq, lonis_schema::BlockPayload)]
enum BarePayload {
    Fetch { url: String },
}

#[test]
fn no_namespace_uses_bare_variant_name() {
    let payload = BarePayload::Fetch { url: String::new() };
    assert_eq!(payload.kind_name(), "fetch");
    assert_eq!(payload.schema_id(), "lonis.block/fetch/v1");
    let wire = serde_json::to_value(&payload).unwrap();
    assert_eq!(wire["kind"], "fetch");
    let back: BarePayload = serde_json::from_value(wire).unwrap();
    assert_eq!(back, payload);
}
