#![recursion_limit = "256"]
#![forbid(unsafe_code)]

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
/// The `Key` trait is what allows fixed_map to set up storage for a type that will be the key in
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
    let tokens = context::Toks::new(&quote!(fixed_map));
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
    match &cx.ast.data {
        Data::Enum(en) => {
            if is_all_unit_variants(en) {
                unit_variants::implement(cx, en)
            } else {
                any_variants::implement(cx, en)
            }
        }
        _ => {
            cx.error(cx.ast.span(), "named fields are not supported");
            Err(())
        }
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
