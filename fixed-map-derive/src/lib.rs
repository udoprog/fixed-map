#![recursion_limit = "256"]

extern crate proc_macro;

use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Data, DataEnum, DeriveInput, Fields, Ident};

/// Derive to implement the `Key` trait.
///
/// Requires that `fixed_map` is in scope.
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
                field_inits.push(quote!(#field: None));

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
                        f((&#m, value));
                    }
                });

                iter_as_mut.push(quote!{
                    if let Some(value) = self.#field.as_mut() {
                        f((&#m, value));
                    }
                });
            }
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
        #vis struct #storage<V> {
            #(#fields,)*
        }

        impl<V> Default for #storage<V> {
            fn default() -> #storage<V> {
                #storage { #(#field_inits,)* }
            }
        }

        impl<V: 'static> fixed_map::Storage<#base, V> for #storage<V> {
            fn insert(
                &mut self,
                key: #base,
                value: V,
            ) -> Option<V> {
                match key {
                    #(#insert,)*
                }
            }

            fn get(&self, value: &#base) -> Option<&V> {
                match *value {
                    #(#get,)*
                }
            }

            fn get_mut(&mut self, value: &#base) -> Option<&mut V> {
                match *value {
                    #(#get_mut,)*
                }
            }

            fn remove(&mut self, value: &#base) -> Option<V> {
                match *value {
                    #(#remove,)*
                }
            }

            fn clear(&mut self) {
                #(#clear;)*
            }

            fn iter<'a, F>(&'a self, mut f: F) where F: FnMut((&'a #base, &'a V)) {
                #(#iter_as_ref)*
            }

            fn iter_mut<'a, F>(&'a mut self, mut f: F) where F: FnMut((&'a #base, &'a mut V)) {
                #(#iter_as_mut)*
            }
        }

        impl<V: 'static> fixed_map::Key<#base, V> for #base {
            type Storage = #storage<V>;
        }
    }
}
