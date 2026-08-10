// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! Procedural macros for Lonis tools.
//!
//! Currently provides [`LonisCapabilities`], which derives the
//! `lonis_schema::Capabilities` self-description trait for a tool type from a
//! `#[lonis(tool_id = "...")]` attribute — removing the five-method boilerplate
//! every tool otherwise repeats (design decision #3: "traits +
//! `#[lonis::tool]` derive").
//!
//! Re-exported from `lonis-schema` behind its `derive` feature, so tool authors
//! write `use lonis_schema::LonisCapabilities;`.

#![forbid(unsafe_code)]

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Data, DeriveInput, Fields, LitStr};

/// Derive `lonis_schema::Capabilities` for a tool type.
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

// ===========================================================================
// BlockPayload derive (ADR-0004)
// ===========================================================================

/// Derive `lonis_schema::BlockPayload` for a vertical's payload enum, along
/// with the adjacently-tagged (`{"kind", "data"}`) serde impls the erased
/// seam requires (ADR-0002/0003; karpal-discovery spike findings 1+2).
///
/// One declaration generates the serde wire tag and `kind_name()`
/// consistently, so the in-process kind and the host-visible
/// `Extension.kind` can never diverge. Kind tags are the variant names in
/// snake_case, prefixed by `namespace` when given:
/// `Search` + `namespace = "karpal"` → `karpal.search`.
///
/// Supports struct variants (named fields) and unit variants (which carry
/// `data: null`). Tuple variants and generics are rejected at compile time.
/// `render_human` defaults to `"<kind>: <Debug>"` (the enum must implement
/// `Debug`), or delegates to a hook: `render_fn = "path::to::render"` — a
/// `fn(&Self) -> String` — so a custom human render never costs the derive's
/// wire safety. The consumer crate must depend on `serde` and `serde_json`.
///
/// ```ignore
/// use lonis_schema::BlockPayload;
///
/// #[derive(Debug, Clone, PartialEq, BlockPayload)]
/// #[lonis_payload(namespace = "karpal")]
/// enum KarpalPayload {
///     Search { results: Vec<String> },
///     Ready,
/// }
///
/// assert_eq!(KarpalPayload::Ready.kind_name(), "karpal.ready");
///
/// // With a custom human render (keeps the derived wire safety):
/// #[derive(Debug, Clone, PartialEq, BlockPayload)]
/// #[lonis_payload(namespace = "karpal", render_fn = "render_karpal")]
/// enum RenderedPayload {
///     Search { query: String, results: Vec<String> },
/// }
/// fn render_karpal(p: &RenderedPayload) -> String { /* … */ }
/// ```
#[proc_macro_derive(BlockPayload, attributes(lonis_payload))]
pub fn derive_block_payload(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match expand_block_payload(&input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn expand_block_payload(input: &DeriveInput) -> Result<proc_macro2::TokenStream, syn::Error> {
    let name = &input.ident;

    if !input.generics.params.is_empty() {
        return Err(syn::Error::new_spanned(
            &input.generics,
            "`#[derive(BlockPayload)]` does not support generics",
        ));
    }

    let Data::Enum(data) = &input.data else {
        return Err(syn::Error::new_spanned(
            input,
            "`#[derive(BlockPayload)]` requires an enum of payload variants",
        ));
    };

    let (namespace, render_fn) = parse_payload_attrs(input)?;

    struct Variant {
        ident: syn::Ident,
        kind: String,
        fields: Vec<(syn::Ident, syn::Type)>,
    }

    let mut variants = Vec::new();
    for variant in &data.variants {
        let snake = to_snake_case(&variant.ident.to_string());
        let kind = match &namespace {
            Some(ns) => format!("{ns}.{snake}"),
            None => snake,
        };
        let fields = match &variant.fields {
            Fields::Named(named) => named
                .named
                .iter()
                .map(|f| (f.ident.clone().expect("named field"), f.ty.clone()))
                .collect(),
            Fields::Unit => Vec::new(),
            Fields::Unnamed(_) => {
                return Err(syn::Error::new_spanned(
                    variant,
                    "`#[derive(BlockPayload)]` does not support tuple variants; use struct variants",
                ));
            }
        };
        variants.push(Variant {
            ident: variant.ident.clone(),
            kind,
            fields,
        });
    }

    // Per-variant shadow structs: a borrowed one for Serialize, an owned one
    // for Deserialize (field types copied verbatim; serde derive does the rest).
    let shadow_defs = variants.iter().filter(|v| !v.fields.is_empty()).map(|v| {
        let ser_ident = format_ident!("{}{}SerShadow", name, v.ident);
        let de_ident = format_ident!("{}{}DeShadow", name, v.ident);
        let field_idents: Vec<_> = v.fields.iter().map(|(i, _)| i).collect();
        let field_tys: Vec<_> = v.fields.iter().map(|(_, t)| t).collect();
        quote! {
            #[derive(::serde::Serialize)]
            struct #ser_ident<'a> { #( #field_idents: &'a #field_tys ),* }
            #[derive(::serde::Deserialize)]
            #[serde(deny_unknown_fields)]
            struct #de_ident { #( #field_idents: #field_tys ),* }
        }
    });

    let kind_arms = variants.iter().map(|v| {
        let ident = &v.ident;
        let kind = &v.kind;
        if v.fields.is_empty() {
            quote! { #name::#ident => #kind }
        } else {
            quote! { #name::#ident { .. } => #kind }
        }
    });

    let ser_arms = variants.iter().map(|v| {
        let ident = &v.ident;
        if v.fields.is_empty() {
            quote! { #name::#ident => map.serialize_entry("data", &())? }
        } else {
            let ser_ident = format_ident!("{}{}SerShadow", name, ident);
            let field_idents: Vec<_> = v.fields.iter().map(|(i, _)| i).collect();
            quote! {
                #name::#ident { #( #field_idents ),* } => {
                    map.serialize_entry("data", &#ser_ident { #( #field_idents ),* })?
                }
            }
        }
    });

    let de_arms = variants.iter().map(|v| {
        let ident = &v.ident;
        let kind = &v.kind;
        if v.fields.is_empty() {
            quote! { #kind => Ok(#name::#ident) }
        } else {
            let de_ident = format_ident!("{}{}DeShadow", name, ident);
            let field_idents: Vec<_> = v.fields.iter().map(|(i, _)| i).collect();
            quote! {
                #kind => {
                    let parsed: #de_ident =
                        ::serde_json::from_value(data).map_err(D::Error::custom)?;
                    Ok(#name::#ident { #( #field_idents: parsed.#field_idents ),* })
                }
            }
        }
    });

    let render_body = match &render_fn {
        Some(path) => quote! { #path(self) },
        None => quote! { format!("{}: {:?}", self.__lonis_kind_name(), self) },
    };

    Ok(quote! {
        const _: () = {
            #( #shadow_defs )*

            impl #name {
                fn __lonis_kind_name(&self) -> &'static str {
                    match self {
                        #( #kind_arms, )*
                    }
                }
            }

            impl ::serde::Serialize for #name {
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: ::serde::Serializer,
                {
                    use ::serde::ser::SerializeMap as _;
                    let mut map = serializer.serialize_map(Some(2))?;
                    map.serialize_entry("kind", self.__lonis_kind_name())?;
                    match self {
                        #( #ser_arms, )*
                    }
                    map.end()
                }
            }

            impl<'de> ::serde::Deserialize<'de> for #name {
                fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where
                    D: ::serde::Deserializer<'de>,
                {
                    use ::serde::de::Error as _;
                    let mut wire = <::serde_json::Map<String, ::serde_json::Value> as ::serde::Deserialize>::deserialize(deserializer)?;
                    let kind = wire
                        .remove("kind")
                        .and_then(|v| v.as_str().map(str::to_owned))
                        .ok_or_else(|| D::Error::custom("block payload requires a string `kind`"))?;
                    let data = wire.remove("data").unwrap_or(::serde_json::Value::Null);
                    if !wire.is_empty() {
                        return Err(D::Error::custom("block payload has unknown fields"));
                    }
                    match kind.as_str() {
                        #( #de_arms, )*
                        other => Err(D::Error::custom(format!(
                            "unknown payload kind `{other}` for this enum"
                        ))),
                    }
                }
            }

            impl ::lonis_schema::BlockPayload for #name {
                fn kind_name(&self) -> &str {
                    self.__lonis_kind_name()
                }
                fn schema_id(&self) -> String {
                    format!("lonis.block/{}/v1", self.__lonis_kind_name())
                }
                fn render_human(&self) -> String {
                    #render_body
                }
            }
        };
    })
}

/// Parse the optional `#[lonis_payload(...)]` attribute: `namespace = "…"`
/// (validated canonical) and/or `render_fn = "path::to::fn"` (a
/// `fn(&#name) -> String` hook for `render_human`).
fn parse_payload_attrs(
    input: &DeriveInput,
) -> Result<(Option<String>, Option<syn::Path>), syn::Error> {
    let mut namespace: Option<String> = None;
    let mut render_fn: Option<syn::Path> = None;
    for attr in &input.attrs {
        if attr.path().is_ident("lonis_payload") {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("namespace") {
                    let lit: LitStr = meta.value()?.parse()?;
                    namespace = Some(lit.value());
                    Ok(())
                } else if meta.path.is_ident("render_fn") {
                    let lit: LitStr = meta.value()?.parse()?;
                    render_fn = Some(lit.parse()?);
                    Ok(())
                } else {
                    Err(meta.error(
                        "unsupported `lonis_payload` key (expected `namespace` or `render_fn`)",
                    ))
                }
            })?;
        }
    }
    if let Some(ns) = &namespace {
        let canonical = !ns.is_empty()
            && ns.bytes().all(|b| {
                b.is_ascii_lowercase() || b.is_ascii_digit() || matches!(b, b'.' | b'-' | b'_')
            });
        if !canonical {
            return Err(syn::Error::new_spanned(
                input,
                "payload namespace must be non-empty lowercase ASCII (digits, `.`, `-`, `_` allowed)",
            ));
        }
    }
    Ok((namespace, render_fn))
}

/// `SearchResults` → `search_results` (sufficient for variant idents).
fn to_snake_case(name: &str) -> String {
    let mut out = String::with_capacity(name.len() + 4);
    for (i, ch) in name.chars().enumerate() {
        if ch.is_uppercase() {
            if i > 0 {
                out.push('_');
            }
            out.extend(ch.to_lowercase());
        } else {
            out.push(ch);
        }
    }
    out
}
