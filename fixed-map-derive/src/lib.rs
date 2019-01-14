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
/// impl<V> fixed_map::storage::Storage<Key, V> for KeyStorage<V> {
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
/// impl<V> fixed_map::key::Key<Key, V> for Key {
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
    let ident = &ast.ident;

    let const_wrapper = Ident::new(&format!("__IMPL_KEY_FOR_{}", ast.ident), Span::call_site());

    let mut fields = Vec::new();
    let mut field_inits = Vec::new();
    let mut field_clones = Vec::new();
    let mut field_partial_eqs = Vec::new();

    let mut get = Vec::new();
    let mut get_mut = Vec::new();
    let mut insert = Vec::new();
    let mut remove = Vec::new();
    let mut clear = Vec::new();

    let mut iter_as_ref = Vec::new();
    let mut iter_as_mut = Vec::new();

    for (i, variant) in en.variants.iter().enumerate() {
        let field = Ident::new(&format!("f{}", i), Span::call_site());

        field_inits.push(quote!(#field: Default::default()));
        field_clones.push(quote!(#field: self.#field.clone()));
        field_partial_eqs.push(quote! {
            if self.#field != other.#field {
                return false;
            }
        });

        match variant.fields {
            Fields::Unit => {
                let var = &variant.ident;
                let m = quote!(#ident::#var);

                fields.push(quote!(#field: Option<V>));

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
                let m = quote!(#ident::#var(v));

                fields.push(
                    quote!(#field: <#element as fixed_map::key::Key<#element, V>>::Storage),
                );

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
                        f((#ident::#var(k), v));
                    });
                });

                iter_as_mut.push(quote!{
                    self.#field.iter_mut(|(k, v)| {
                        f((#ident::#var(k), v));
                    });
                });
            },
            _ => panic!("Only unit fields are supported in fixed enums"),
        }
    }

    quote! {
        const #const_wrapper: () = {
            #vis struct Storage<V: 'static> {
                #(#fields,)*
            }

            impl<V> Clone for Storage<V> where V: Clone {
                fn clone(&self) -> Storage<V> {
                    Storage {
                        #(#field_clones,)*
                    }
                }
            }

            impl<V> std::cmp::PartialEq for Storage<V> where V: std::cmp::PartialEq {
                fn eq(&self, other: &Storage<V>) -> bool {
                    #(#field_partial_eqs;)*
                    true
                }
            }

            impl<V> std::cmp::Eq for Storage<V> where V: std::cmp::Eq {
            }

            impl<V> Default for Storage<V> {
                fn default() -> Storage<V> {
                    Storage {
                        #(#field_inits,)*
                    }
                }
            }

            impl<V> fixed_map::storage::Storage<#ident, V> for Storage<V> {
                #[inline]
                fn insert(&mut self, key: #ident, value: V) -> Option<V> {
                    match key {
                        #(#insert,)*
                    }
                }

                #[inline]
                fn get(&self, value: #ident) -> Option<&V> {
                    match value {
                        #(#get,)*
                    }
                }

                #[inline]
                fn get_mut(&mut self, value: #ident) -> Option<&mut V> {
                    match value {
                        #(#get_mut,)*
                    }
                }

                #[inline]
                fn remove(&mut self, value: #ident) -> Option<V> {
                    match value {
                        #(#remove,)*
                    }
                }

                #[inline]
                fn clear(&mut self) {
                    #(#clear;)*
                }

                #[inline]
                fn iter<'a>(&'a self, mut f: impl FnMut((#ident, &'a V))) {
                    #(#iter_as_ref)*
                }

                #[inline]
                fn iter_mut<'a>(&'a mut self, mut f: impl FnMut((#ident, &'a mut V))) {
                    #(#iter_as_mut)*
                }
            }

            impl<V: 'static> fixed_map::key::Key<#ident, V> for #ident {
                type Storage = Storage<V>;
            }
        };
    }
}
