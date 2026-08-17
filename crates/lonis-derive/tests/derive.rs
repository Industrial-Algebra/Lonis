// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! Functional tests for the `LonisCapabilities` derive, exercised through the
//! `lonis_schema::LonisCapabilities` re-export (the consumer path).

use lonis_schema::{Capabilities, LonisCapabilities, OutputMode, SchemaVersion};

#[derive(LonisCapabilities)]
#[lonis(tool_id = "amari:discovery:search")]
struct SearchTool;

#[test]
fn derived_tool_id() {
    assert_eq!(SearchTool.tool_id().as_str(), "amari:discovery:search");
}

#[test]
fn derived_output_formats_default_to_all_three() {
    let formats = SearchTool.output_formats();
    assert_eq!(formats.len(), 3);
    assert!(formats.contains(&OutputMode::Human));
    assert!(formats.contains(&OutputMode::Json));
    assert!(formats.contains(&OutputMode::Ndjson));
}

#[test]
fn derived_schema_version_is_one() {
    assert_eq!(SearchTool.schema_version(), SchemaVersion::default());
}

#[test]
fn derived_exit_code_map_has_baselines() {
    let map = SearchTool.exit_code_map();
    assert!(map.iter().any(|(k, _)| *k == "not_found"));
    assert!(map.iter().any(|(k, _)| *k == "invalid_input"));
}

#[test]
fn derived_tool_version_is_crate_version() {
    // tool_version() defaults to the consumer crate's package version; for
    // these integration tests that is lonis-derive's own version.
    assert_eq!(SearchTool.tool_version(), env!("CARGO_PKG_VERSION"));
}
