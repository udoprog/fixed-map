//! [<img alt="github" src="https://img.shields.io/badge/github-udoprog/fixed--map-8da0cb?style=for-the-badge&logo=github" height="20">](https://github.com/udoprog/fixed-map)
//! [<img alt="crates.io" src="https://img.shields.io/crates/v/fixed-map-derive.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/fixed-map-derive)
//! [<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-fixed--map--derive-66c2a5?style=for-the-badge&logoColor=white&logo=data:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K" height="20">](https://docs.rs/fixed-map-derive)
//! [<img alt="build status" src="https://img.shields.io/github/workflow/status/udoprog/fixed-map/CI/main?style=for-the-badge" height="20">](https://github.com/udoprog/fixed-map/actions?query=branch%3Amain)
//!
//! This crate contains the procedural macros used in [fixed-map].
//!
//! [fixed-map]: https://github.com/udoprog/fixed-map

#![recursion_limit = "256"]
#![forbid(unsafe_code)]
// Enable pedantic lints as warnings so we don't break builds when
// lints are modified or new lints are added to clippy.
#![warn(
    // Enable more useful rustc lints
    absolute_paths_not_starting_with_crate,
    elided_lifetimes_in_paths,
    explicit_outlives_requirements,
    keyword_idents,
    macro_use_extern_crate,
    meta_variable_misuse,
    missing_copy_implementations,
    missing_docs,
    non_ascii_idents,
    noop_method_call,
    pointer_structural_match,
    single_use_lifetimes,
    trivial_casts,
    trivial_numeric_casts,
    unreachable_pub,
    unused_extern_crates,
    unused_import_braces,
    unused_lifetimes,
    unused_macro_rules,
    unused_qualifications,
    unused_tuple_struct_fields,
    variant_size_differences,
    // Enable pedantic clippy lints
    clippy::pedantic,
    // Useful clippy lints for no_std support
    clippy::std_instead_of_core,
    clippy::std_instead_of_alloc,
    clippy::alloc_instead_of_core
)]
#![allow(
    // rustc exceptions
    missing_docs,
    // `clippy::pedantic` exceptions
    // style choice
    clippy::module_name_repetitions,
    // style choice
    clippy::too_many_lines,
    // false positive
    clippy::type_repetition_in_bounds,
    // false positive
    clippy::expl_impl_clone_on_copy,
    // conscious choice
    clippy::unnecessary_wraps
)]

use proc_macro2::TokenStream;
use quote::quote;
use syn::spanned::Spanned;
use syn::{Data, DataEnum, DeriveInput, Fields};

mod any_variants;
mod context;
mod unit_variants;

/// Derive to implement the `Key` trait.
///
/// Requires that `fixed_map` is in scope.
///
/// This derive implements the `Key` trait for a given type.
///
/// The `Key` trait is what allows `fixed_map` to set up storage for a type that will be the key in
/// a fixed map.
///
/// Given the following enum:
///
/// ```rust
/// use fixed_map::Key;
///
/// #[derive(Clone, Copy, Key)]
/// pub enum Key {
///     First,
///     Second,
///     Third,
/// }
/// ```
///
/// It performs the following simplified expansion:
///
/// ```rust,no_compile,no_run
/// use fixed_map::Key;
///
/// #[derive(Clone, Copy)]
/// pub enum Key {
///     First,
///     Second,
///     Third,
/// }
///
/// /// Build a storage struct containing an item for each key:
/// pub struct KeyStorage<V> {
///     /// Storage for `Key::First`.
///     f1: Option<V>,
///     /// Storage for `Key::Second`.
///     f2: Option<V>,
///     /// Storage for `Key::Third`.
///     f3: Option<V>,
/// }
///
/// /// Implement storage for `KeyStorage`.
/// impl<V> fixed_map::storage::Storage<Key, V> for KeyStorage<V> {
///     fn get(&self, key: Key) -> Option<&V> {
///         match key {
///             Key::First => self.f1.as_ref(),
///             Key::Second => self.f2.as_ref(),
///             Key::Third => self.f3.as_ref(),
///         }
///     }
///
///     /* skipped */
/// }
///
/// impl<V> Default for KeyStorage<V> {
///     fn default() -> Self {
///         Self {
///             f1: None,
///             f2: None,
///             f3: None,
///         }
///     }
/// }
///
/// /// Implement the `Key` trait to point out storage.
/// impl<V> fixed_map::key::Key<Key, V> for Key {
///     type Storage = KeyStorage<V>;
/// }
/// ```
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

/// Derive to implement the `Key` trait.
fn impl_storage(cx: &context::Ctxt<'_>) -> Result<TokenStream, ()> {
    if let Data::Enum(en) = &cx.ast.data {
        if is_all_unit_variants(en) {
            unit_variants::implement(cx, en)
        } else {
            any_variants::implement(cx, en)
        }
    } else {
        cx.error(cx.ast.span(), "named fields are not supported");
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
