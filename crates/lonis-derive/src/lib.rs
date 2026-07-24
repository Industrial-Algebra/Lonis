// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! Procedural macros for Lonis tools.
//!
//! Currently provides [`LonisCapabilities`], which derives the
//! [`lonis_schema::Capabilities`] self-description trait for a tool type from a
//! `#[lonis(tool_id = "...")]` attribute — removing the five-method boilerplate
//! every tool otherwise repeats (design decision #3: "traits +
//! `#[lonis::tool]` derive").
//!
//! Re-exported from `lonis-schema` behind its `derive` feature, so tool authors
//! write `use lonis_schema::LonisCapabilities;`.

#![forbid(unsafe_code)]

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, LitStr};

/// Derive [`lonis_schema::Capabilities`] for a tool type.
///
/// Requires a `#[lonis(tool_id = "<tool>:<namespace>:<item>")]` attribute. The
/// derived impl uses sensible defaults: schema version 1, all three output
/// modes, the baseline exit-code map, and the tool crate's `CARGO_PKG_VERSION`.
///
/// ```ignore
/// use lonis_schema::{Capabilities, LonisCapabilities};
///
/// #[derive(LonisCapabilities)]
/// #[lonis(tool_id = "amari:discovery:search")]
/// struct SearchTool;
///
/// assert_eq!(SearchTool.tool_id().as_str(), "amari:discovery:search");
/// ```
#[proc_macro_derive(LonisCapabilities, attributes(lonis))]
pub fn derive_lonis_capabilities(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let mut tool_id: Option<String> = None;
    for attr in &input.attrs {
        if attr.path().is_ident("lonis") {
            if let Err(err) = attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("tool_id") {
                    let lit: LitStr = meta.value()?.parse()?;
                    tool_id = Some(lit.value());
                }
                Ok(())
            }) {
                return err.to_compile_error().into();
            }
        }
    }

    let Some(tool_id) = tool_id else {
        return syn::Error::new_spanned(
            &input,
            "`#[derive(LonisCapabilities)]` requires `#[lonis(tool_id = \"...\")]`",
        )
        .to_compile_error()
        .into();
    };

    let expanded = quote! {
        impl ::lonis_schema::Capabilities for #name {
            fn schema_version(&self) -> ::lonis_schema::SchemaVersion {
                ::lonis_schema::SchemaVersion::default()
            }

            fn tool_version(&self) -> &str {
                env!("CARGO_PKG_VERSION")
            }

            fn output_formats(&self) -> &'static [::lonis_schema::OutputMode] {
                &[
                    ::lonis_schema::OutputMode::Human,
                    ::lonis_schema::OutputMode::Json,
                    ::lonis_schema::OutputMode::Ndjson,
                ]
            }

            fn exit_code_map(&self) -> &'static [(&'static str, u8)] {
                &[
                    ("ok", ::lonis_schema::exit_code::SUCCESS),
                    ("generic", ::lonis_schema::exit_code::GENERIC),
                    ("invalid_input", ::lonis_schema::exit_code::INVALID_INPUT),
                    ("not_found", ::lonis_schema::exit_code::NOT_FOUND),
                    ("confirmation_required", ::lonis_schema::exit_code::CONFIRMATION_REQUIRED),
                    ("rate_limited", ::lonis_schema::exit_code::RATE_LIMITED),
                ]
            }

            fn tool_id(&self) -> ::lonis_schema::ToolId {
                ::lonis_schema::ToolId::new(#tool_id)
                    .expect("lonis tool_id must be a valid namespaced id")
            }
        }
    };
    expanded.into()
}
