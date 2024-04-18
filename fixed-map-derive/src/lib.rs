//! [<img alt="github" src="https://img.shields.io/badge/github-udoprog/fixed--map-8da0cb?style=for-the-badge&logo=github" height="20">](https://github.com/udoprog/fixed-map)
//! [<img alt="crates.io" src="https://img.shields.io/crates/v/fixed-map-derive.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/fixed-map-derive)
//! [<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-fixed--map--derive-66c2a5?style=for-the-badge&logoColor=white&logo=data:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K" height="20">](https://docs.rs/fixed-map-derive)
//!
//! This crate contains the procedural macros used in [fixed-map].
//!
//! [fixed-map]: https://github.com/udoprog/fixed-map

#![recursion_limit = "256"]
#![forbid(unsafe_code)]
#![warn(absolute_paths_not_starting_with_crate)]
#![warn(clippy::alloc_instead_of_core)]
#![warn(clippy::pedantic)]
#![warn(clippy::std_instead_of_alloc)]
#![warn(clippy::std_instead_of_core)]
#![warn(dead_code)]
#![warn(elided_lifetimes_in_paths)]
#![warn(explicit_outlives_requirements)]
#![warn(keyword_idents)]
#![warn(macro_use_extern_crate)]
#![warn(meta_variable_misuse)]
#![warn(missing_copy_implementations)]
#![warn(missing_docs)]
#![warn(non_ascii_idents)]
#![warn(noop_method_call)]
#![warn(pointer_structural_match)]
#![warn(single_use_lifetimes)]
#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]
#![warn(unreachable_pub)]
#![warn(unused_extern_crates)]
#![warn(unused_import_braces)]
#![warn(unused_lifetimes)]
#![warn(unused_macro_rules)]
#![warn(unused_qualifications)]
#![warn(variant_size_differences)]
#![allow(clippy::expl_impl_clone_on_copy)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::type_repetition_in_bounds)]
#![allow(clippy::unnecessary_wraps)]
#![allow(missing_docs)]

use proc_macro2::TokenStream;
use quote::quote;
use syn::spanned::Spanned;
use syn::{Data, DataEnum, DeriveInput, Fields};

mod any_variants;
mod attrs;
mod context;
mod symbol;
mod unit_variants;

/// See <https://docs.rs/fixed-map>.
#[proc_macro_derive(Key, attributes(key))]
pub fn storage_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse_macro_input!(input as DeriveInput);

    let lt = syn::Lifetime::new("'a", ast.span());
    let crate_prefix = context::leading_path(["fixed_map"]);
    let tokens = context::Toks::new(&crate_prefix);
    let cx = context::Ctxt::new(&tokens, &ast, &lt);

    let result = impl_storage(&cx);

    if let Ok(gen) = result {
        return gen.into();
    }

    let errors = cx.into_errors();
    let compile_errors = errors.iter().map(syn::Error::to_compile_error);
    quote!(#(#compile_errors)*).into()
}

fn impl_storage(cx: &context::Ctxt<'_>) -> Result<TokenStream, ()> {
    let opts = attrs::parse(cx)?;

    if let Data::Enum(en) = &cx.ast.data {
        if is_all_unit_variants(en) {
            unit_variants::implement(cx, &opts, en)
        } else {
            any_variants::implement(cx, en)
        }
    } else {
        cx.span_error(cx.ast.span(), "named fields are not supported");
        Err(())
    }
}

fn is_all_unit_variants(en: &DataEnum) -> bool {
    for v in &en.variants {
        if !matches!(&v.fields, Fields::Unit) {
            return false;
        }
    }

    true
}
