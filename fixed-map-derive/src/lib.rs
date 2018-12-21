#![recursion_limit = "256"]

extern crate proc_macro;

use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Data, DataEnum, DeriveInput, Fields, Ident};

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
/// ```rust,no_run
/// #[derive(Clone, Copy, fixed_map::Key)]
/// pub enum Key {
///     First,
///     Second,
///     Third,
/// }
/// ```
///
/// It performs the following simplified expansion:
///
/// ```rust,no_run
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
/// impl<V> fixed_map::Storage<Key, V> for KeyStorage<V> {
///     fn get(&self, key: Key) -> Option<&V> {
///         match *self {
///             Key::First => self.f1.as_ref(),
///             Key::Second => self.f2.as_ref(),
///             Key::Third => self.f3.as_ref(),
///         }
///     }
///
///     /* other methods skipped */
/// }
///
/// /// Implement the `Key` trait to point out storage.
/// impl<V> fixed_map::Key<Key, V> for Key {
///     type Storage = KeyStorage<V>;
/// }
/// ```
#[proc_macro_derive(Key, attributes(key))]
pub fn storage_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse_macro_input!(input as DeriveInput);
    let gen = impl_storage(&ast);
    gen.into()
}

/// Derive to implement the `Key` trait.
fn impl_storage(ast: &DeriveInput) -> TokenStream {
    match ast.data {
        Data::Enum(ref en) => return impl_storage_enum(ast, en),
        _ => panic!("`Key` attribute is only supported on enums"),
    }
}

/// Implement `Key` for enums.
fn impl_storage_enum(ast: &DeriveInput, en: &DataEnum) -> TokenStream {
    let vis = &ast.vis;
    let base = &ast.ident;

    let storage = Ident::new(&format!("{}Storage", base), Span::call_site());

    let mut field_inits = Vec::new();
    let mut fields = Vec::new();

    let mut get = Vec::new();
    let mut get_mut = Vec::new();
    let mut insert = Vec::new();
    let mut remove = Vec::new();
    let mut clear = Vec::new();

    let mut iter_as_ref = Vec::new();
    let mut iter_as_mut = Vec::new();

    let first = en
        .variants
        .iter()
        .next()
        .expect("enum must have at least one variant");

    let default_fn = match first.fields {
        Fields::Unit => {
            let ident = &first.ident;
            quote!(#base::#ident)
        }
        _ => {
            panic!("Only unit fields are supported in fixed enums");
        }
    };

    for (i, variant) in en.variants.iter().enumerate() {
        let field = Ident::new(&format!("f{}", i), Span::call_site());

        match variant.fields {
            Fields::Unit => {
                let var = &variant.ident;
                let m = quote!(#base::#var);

                fields.push(quote!(#field: Option<V>));
                field_inits.push(quote!(#field: Default::default()));

                get.push(quote!(#m => return self.#field.as_ref()));

                get_mut.push(quote!(#m => return self.#field.as_mut()));

                insert.push(
                    quote!{#m => {
                        return ::std::mem::replace(&mut self.#field, Some(value));
                    }}
                );

                remove
                    .push(quote!(#m => return ::std::mem::replace(&mut self.#field, None)));

                clear.push(quote!(self.#field = None));

                iter_as_ref.push(quote!{
                    if let Some(value) = self.#field.as_ref() {
                        f((#m, value));
                    }
                });

                iter_as_mut.push(quote!{
                    if let Some(value) = self.#field.as_mut() {
                        f((#m, value));
                    }
                });
            },
            Fields::Unnamed(ref unnamed) => {
                if unnamed.unnamed.len() > 1 {
                    panic!("Unnamed variants must have exactly one element");
                }

                let element = unnamed.unnamed.first().expect("Expected one element");

                let var = &variant.ident;
                let m = quote!(#base::#var(v));

                fields.push(
                    quote!(#field: <#element as fixed_map::Key<#element, V>>::Storage),
                );

                field_inits.push(quote!(#field: Default::default()));

                get.push(quote!(#m => return self.#field.get(v)));
                get_mut.push(quote!(#m => return self.#field.get_mut(v)));

                insert.push(
                    quote!{#m => {
                        return self.#field.insert(v, value);
                    }}
                );

                remove.push(quote!{#m => {
                    return self.#field.remove(v);
                }});

                clear.push(quote!(self.#field.clear()));

                iter_as_ref.push(quote!{
                    self.#field.iter(|(k, v)| {
                        f((#base::#var(k), v));
                    });
                });

                iter_as_mut.push(quote!{
                    self.#field.iter_mut(|(k, v)| {
                        f((#base::#var(k), v));
                    });
                });
            },
            _ => panic!("Only unit fields are supported in fixed enums"),
        }
    }

    quote! {
        impl Default for #base {
            fn default() -> #base {
                #default_fn
            }
        }

        #[derive(Clone)]
        #vis struct #storage<V: 'static> {
            #(#fields,)*
        }

        impl<V: 'static> Default for #storage<V> {
            fn default() -> #storage<V> {
                #storage { #(#field_inits,)* }
            }
        }

        impl<V: 'static> fixed_map::Storage<#base, V> for #storage<V> {
            #[inline]
            fn insert(
                &mut self,
                key: #base,
                value: V,
            ) -> Option<V> {
                match key {
                    #(#insert,)*
                }
            }

            #[inline]
            fn get(&self, value: #base) -> Option<&V> {
                match value {
                    #(#get,)*
                }
            }

            #[inline]
            fn get_mut(&mut self, value: #base) -> Option<&mut V> {
                match value {
                    #(#get_mut,)*
                }
            }

            #[inline]
            fn remove(&mut self, value: #base) -> Option<V> {
                match value {
                    #(#remove,)*
                }
            }

            #[inline]
            fn clear(&mut self) {
                #(#clear;)*
            }

            #[inline]
            fn iter<'a, F>(&'a self, mut f: F) where F: FnMut((#base, &'a V)) {
                #(#iter_as_ref)*
            }

            #[inline]
            fn iter_mut<'a, F>(&'a mut self, mut f: F) where F: FnMut((#base, &'a mut V)) {
                #(#iter_as_mut)*
            }
        }

        impl<V: 'static> fixed_map::Key<#base, V> for #base {
            type Storage = #storage<V>;
        }
    }
}
