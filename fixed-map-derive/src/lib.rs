#![recursion_limit = "256"]
#![deny(unsafe_code)]

extern crate proc_macro;

use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Data, DataEnum, DeriveInput, Fields, Ident};

struct Tokens {
    array_into_iter: TokenStream,
    clone: TokenStream,
    copy: TokenStream,
    default: TokenStream,
    eq: TokenStream,
    into_iter: TokenStream,
    iterator: TokenStream,
    key_trait: TokenStream,
    mem: TokenStream,
    option_as_mut: TokenStream,
    option_as_ref: TokenStream,
    option: TokenStream,
    partial_eq: TokenStream,
    slice_iter: TokenStream,
    storage_trait: TokenStream,
}

impl Tokens {
    fn new(krate: &TokenStream) -> Self {
        Self {
            array_into_iter: quote!(::core::array::IntoIter),
            clone: quote!(::core::clone::Clone),
            copy: quote!(::core::marker::Copy),
            default: quote!(::core::default::Default),
            eq: quote!(::core::cmp::Eq),
            into_iter: quote!(::core::iter::IntoIterator::into_iter),
            iterator: quote!(::core::iter::Iterator),
            key_trait: quote!(#krate::key::Key),
            mem: quote!(::core::mem),
            option_as_mut: quote!(::core::option::Option::as_mut),
            option_as_ref: quote!(::core::option::Option::as_ref),
            option: quote!(::core::option::Option),
            partial_eq: quote!(::core::cmp::PartialEq),
            slice_iter: quote!(::core::slice::Iter),
            storage_trait: quote!(#krate::storage::Storage),
        }
    }
}

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
    let gen = impl_storage(&ast);
    gen.into()
}

/// Derive to implement the `Key` trait.
fn impl_storage(ast: &DeriveInput) -> TokenStream {
    match ast.data {
        Data::Enum(ref en) => impl_storage_enum(ast, en),
        _ => panic!("`Key` attribute is only supported on enums"),
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

/// Implement `Key` for enums.
fn impl_storage_enum(ast: &DeriveInput, en: &DataEnum) -> TokenStream {
    let tokens = Tokens::new(&quote!(fixed_map));

    if is_all_unit_variants(en) {
        handle_units(&tokens, ast, en)
    } else {
        handle_mixed(&tokens, ast, en)
    }
}

/// Every variant is a unit variant.
fn handle_units(tokens: &Tokens, ast: &DeriveInput, en: &DataEnum) -> TokenStream {
    let vis = &ast.vis;
    let ident = &ast.ident;

    let array_into_iter = &tokens.array_into_iter;
    let clone = &tokens.clone;
    let copy = &tokens.copy;
    let default = &tokens.default;
    let eq = &tokens.eq;
    let into_iter = &tokens.into_iter;
    let iterator = &tokens.iterator;
    let key_trait = &tokens.key_trait;
    let mem = &tokens.mem;
    let option = &tokens.option;
    let option_as_mut = &tokens.option_as_mut;
    let option_as_ref = &tokens.option_as_ref;
    let partial_eq = &tokens.partial_eq;
    let slice_iter = &tokens.slice_iter;
    let storage_trait = &tokens.storage_trait;

    let const_wrapper = Ident::new(&format!("__IMPL_KEY_FOR_{}", ast.ident), Span::call_site());

    let mut pattern = Vec::new();

    let mut names = Vec::new();

    let mut fields = Vec::new();
    let mut field_inits = Vec::new();

    let mut get = Vec::new();
    let mut get_mut = Vec::new();
    let mut insert = Vec::new();
    let mut remove = Vec::new();

    let mut iter_init = Vec::new();

    for (index, variant) in en.variants.iter().enumerate() {
        let var = &variant.ident;
        let field = Ident::new(&format!("f{}", index), Span::call_site());

        names.push(field.clone());

        field_inits.push(quote!(#option::None));

        fields.push(quote!(#option<V>));
        pattern.push(quote!(#ident::#var));

        get.push(quote!(#option_as_ref(#field)));
        get_mut.push(quote!(#option_as_mut(#field)));
        insert.push(quote!(#mem::replace(#field, #option::Some(value))));
        remove.push(quote!(#mem::replace(#field, #option::None)));

        iter_init.push(quote!((#ident::#var, #field)));
    }

    let count = en.variants.len();

    quote! {
        const #const_wrapper: () = {
            #[repr(transparent)]
            #vis struct Storage<V> {
                data: [#option<V>; #count],
            }

            impl<V> #clone for Storage<V> where V: #clone {
                #[inline]
                fn clone(&self) -> Storage<V> {
                    Storage {
                        data: #clone::clone(&self.data),
                    }
                }
            }

            impl<V> #copy for Storage<V> where V: #copy {
            }

            impl<V> #partial_eq for Storage<V> where V: #partial_eq {
                #[inline]
                fn eq(&self, other: &Storage<V>) -> bool {
                    self.data == self.data
                }
            }

            impl<V> #eq for Storage<V> where V: #eq {}

            impl<V> #default for Storage<V> {
                #[inline]
                fn default() -> Storage<V> {
                    Storage {
                        data: [#(#field_inits),*],
                    }
                }
            }

            impl<V> #storage_trait<#ident, V> for Storage<V> {
                type Iter<'this> = Iter<'this, V> where Self: 'this;
                type Values<'this> = Values<'this, V> where Self: 'this;
                type IterMut<'this> = IterMut<'this, V> where Self: 'this;
                type IntoIter = IntoIter<V>;

                #[inline]
                fn insert(&mut self, key: #ident, value: V) -> #option<V> {
                    let [#(#names),*] = &mut self.data;

                    match key {
                        #(#pattern => #insert,)*
                    }
                }

                #[inline]
                fn get(&self, value: #ident) -> #option<&V> {
                    let [#(#names),*] = &self.data;

                    match value {
                        #(#pattern => #get,)*
                    }
                }

                #[inline]
                fn get_mut(&mut self, value: #ident) -> #option<&mut V> {
                    let [#(#names),*] = &mut self.data;

                    match value {
                        #(#pattern => #get_mut,)*
                    }
                }

                #[inline]
                fn remove(&mut self, value: #ident) -> #option<V> {
                    let [#(#names),*] = &mut self.data;

                    match value {
                        #(#pattern => #remove,)*
                    }
                }

                #[inline]
                fn clear(&mut self) {
                    self.data = [#(#field_inits),*];
                }

                #[inline]
                fn iter(&self) -> Self::Iter<'_> {
                    let [#(#names),*] = &self.data;

                    Iter {
                        iter: #into_iter([#(#iter_init),*]),
                    }
                }

                #[inline]
                fn values(&self) -> Self::Values<'_> {
                    Values {
                        iter: #into_iter(&self.data),
                    }
                }

                #[inline]
                fn iter_mut(&mut self) -> Self::IterMut<'_> {
                    let [#(#names),*] = &mut self.data;

                    IterMut {
                        iter: #into_iter([#(#iter_init),*]),
                    }
                }

                #[inline]
                fn into_iter(self) -> Self::IntoIter {
                    let [#(#names),*] = self.data;

                    IntoIter {
                        iter: #into_iter([#(#iter_init),*]),
                    }
                }
            }

            impl<V> #key_trait<#ident, V> for #ident {
                type Storage = Storage<V>;
            }

            #[repr(transparent)]
            #vis struct Iter<'a, V> {
                iter: #array_into_iter<(#ident, &'a #option<V>), #count>,
            }

            impl<'a, V> #clone for Iter<'a, V> {
                #[inline]
                fn clone(&self) -> Iter<'a, V> {
                    Iter {
                        iter: #clone::clone(&self.iter),
                    }
                }
            }

            impl<'a, V> #iterator for Iter<'a, V> {
                type Item = (#ident, &'a V);

                #[inline]
                fn next(&mut self) -> #option<Self::Item> {
                    loop {
                        if let (key, #option::Some(value)) = #iterator::next(&mut self.iter)? {
                            return #option::Some((key, value));
                        }
                    }
                }
            }

            #[repr(transparent)]
            #vis struct Values<'a, V> {
                iter: #slice_iter<'a, #option<V>>,
            }

            impl<'a, V> #clone for Values<'a, V> {
                #[inline]
                fn clone(&self) -> Self {
                    Values {
                        iter: #clone::clone(&self.iter),
                    }
                }
            }

            impl<'a, V> #iterator for Values<'a, V> {
                type Item = &'a V;

                #[inline]
                fn next(&mut self) -> #option<Self::Item> {
                    loop {
                        if let #option::Some(value) = #iterator::next(&mut self.iter)? {
                            return #option::Some(value);
                        }
                    }
                }
            }

            #[repr(transparent)]
            #vis struct IterMut<'a, V> {
                iter: #array_into_iter<(#ident, &'a mut #option<V>), #count>,
            }

            impl<'a, V> #iterator for IterMut<'a, V> {
                type Item = (#ident, &'a mut V);

                #[inline]
                fn next(&mut self) -> #option<Self::Item> {
                    loop {
                        if let (key, #option::Some(value)) = #iterator::next(&mut self.iter)? {
                            return #option::Some((key, value));
                        }
                    }
                }
            }

            #[repr(transparent)]
            #vis struct IntoIter<V> {
                iter: #array_into_iter<(#ident, #option<V>), #count>,
            }

            impl<V> #iterator for IntoIter<V> {
                type Item = (#ident, V);

                #[inline]
                fn next(&mut self) -> #option<Self::Item> {
                    loop {
                        if let (key, #option::Some(value)) = #iterator::next(&mut self.iter)? {
                            return #option::Some((key, value));
                        }
                    }
                }
            }
        };
    }
}

fn handle_mixed(tokens: &Tokens, ast: &DeriveInput, en: &DataEnum) -> TokenStream {
    let vis = &ast.vis;
    let ident = &ast.ident;

    let clone = &tokens.clone;
    let copy = &tokens.copy;
    let default = &tokens.default;
    let eq = &tokens.eq;
    let iterator = &tokens.iterator;
    let key_trait = &tokens.key_trait;
    let mem = &tokens.mem;
    let option = &tokens.option;
    let option_as_mut = &tokens.option_as_mut;
    let option_as_ref = &tokens.option_as_ref;
    let partial_eq = &tokens.partial_eq;
    let storage_trait = &tokens.storage_trait;

    let const_wrapper = Ident::new(&format!("__IMPL_KEY_FOR_{}", ast.ident), Span::call_site());

    let mut pattern = Vec::new();

    let mut fields = Vec::new();
    let mut field_inits = Vec::new();
    let mut field_clones = Vec::new();
    let mut field_partial_eqs = Vec::new();

    let mut get = Vec::new();
    let mut get_mut = Vec::new();
    let mut insert = Vec::new();
    let mut remove = Vec::new();
    let mut clear = Vec::new();

    let mut iter_clone = Vec::new();

    let mut iter_init = Vec::new();
    let mut values_iter_init = Vec::new();
    let mut iter_fields = Vec::new();
    let mut values_iter_fields = Vec::new();

    let mut iter_mut_init = Vec::new();
    let mut iter_mut_fields = Vec::new();

    let mut into_iter_init = Vec::new();
    let mut into_iter_fields = Vec::new();

    let mut iter_next = Vec::new();
    let mut values_iter_next = Vec::new();
    let mut copy_bounds = Vec::new();

    for (index, variant) in en.variants.iter().enumerate() {
        let var = &variant.ident;
        let field = Ident::new(&format!("f{}", index), Span::call_site());

        iter_clone.push(quote!(#field: #clone::clone(&self.#field)));

        field_inits.push(quote!(#field: #default::default()));
        field_clones.push(quote!(#field: #clone::clone(&self.#field)));
        field_partial_eqs.push(quote! {
            if self.#field != other.#field {
                return false;
            }
        });

        match variant.fields {
            Fields::Unit => {
                fields.push(quote!(#field: #option<V>));
                pattern.push(quote!(#ident::#var));
                clear.push(quote!(self.#field = #option::None));

                get.push(quote!(#option_as_ref(&self.#field)));
                get_mut.push(quote!(#option_as_mut(&mut self.#field)));
                insert.push(quote!(#mem::replace(&mut self.#field, #option::Some(value))));
                remove.push(quote!(#mem::replace(&mut self.#field, #option::None)));

                iter_fields.push(quote!(#field: #option<&'a V>));
                values_iter_fields.push(quote!(#field: #option<&'a V>));
                iter_init.push(quote!(#field: #option_as_ref(&self.#field)));
                values_iter_init.push(quote!(#field: #option_as_ref(&self.#field)));
                iter_mut_fields.push(quote!(#field: #option<&'a mut V>));
                iter_mut_init.push(quote!(#field: #option_as_mut(&mut self.#field)));

                into_iter_fields.push(quote!(#field: #option<V>));
                into_iter_init.push(quote!(#field: self.#field));

                iter_next.push(quote! {
                    #index => {
                        if let #option::Some(v) = self.#field.take() {
                            return #option::Some((#ident::#var, v));
                        }

                        self.step += 1;
                    }
                });

                values_iter_next.push(quote! {
                    #index => {
                        if let #option::Some(v) = self.#field.take() {
                            return #option::Some(v);
                        }

                        self.step += 1;
                    }
                });
            }
            Fields::Unnamed(ref unnamed) => {
                if unnamed.unnamed.len() > 1 {
                    panic!("Unnamed variants must have exactly one element");
                }

                let element = unnamed.unnamed.first().expect("Expected one element");
                let storage = quote!(<#element as #key_trait<#element, V>>::Storage);
                let as_storage = quote!(<#storage as #storage_trait<#element, V>>);

                fields.push(quote!(#field: #storage));
                pattern.push(quote!(#ident::#var(v)));
                clear.push(quote!(#as_storage::clear(&mut self.#field)));

                get.push(quote!(#as_storage::get(&self.#field, v)));
                get_mut.push(quote!(#as_storage::get_mut(&mut self.#field, v)));
                insert.push(quote!(#as_storage::insert(&mut self.#field, v, value)));
                remove.push(quote!(#as_storage::remove(&mut self.#field, v)));

                iter_fields.push(quote!(#field: #as_storage::Iter<'a>));
                values_iter_fields.push(quote!(#field: #as_storage::Values<'a>));
                iter_init.push(quote!(#field: #as_storage::iter(&self.#field)));
                values_iter_init.push(quote!(#field: #as_storage::values(&self.#field)));
                iter_mut_fields.push(quote!(#field: #as_storage::IterMut<'a>));
                iter_mut_init.push(quote!(#field: #as_storage::iter_mut(&mut self.#field)));

                into_iter_fields.push(quote!(#field: #as_storage::IntoIter));
                into_iter_init.push(quote!(#field: #storage::into_iter(self.#field)));

                iter_next.push(quote! {
                    #index => {
                        if let #option::Some((k, v)) = #iterator::next(&mut self.#field) {
                            return #option::Some((#ident::#var(k), v));
                        }

                        self.step += 1;
                    }
                });

                values_iter_next.push(quote! {
                    #index => {
                        if let #option::Some(v) = #iterator::next(&mut self.#field) {
                            return #option::Some(v);
                        }

                        self.step += 1;
                    }
                });

                copy_bounds.push(quote!(#storage: #copy));
            }
            _ => panic!("Only unit fields are supported in fixed enums"),
        }
    }

    let pattern = &pattern;
    let iter_next = &iter_next;
    let iter_mut_next = iter_next;
    let into_iter_next = iter_next;

    quote! {
        const #const_wrapper: () = {
            #vis struct Storage<V> {
                #(#fields,)*
            }

            impl<V> #clone for Storage<V> where V: #clone {
                #[inline]
                fn clone(&self) -> Storage<V> {
                    Storage {
                        #(#field_clones,)*
                    }
                }
            }

            impl<V> #copy for Storage<V> where V: #copy, #(#copy_bounds,)* {}

            impl<V> #partial_eq for Storage<V> where V: #partial_eq {
                #[inline]
                fn eq(&self, other: &Storage<V>) -> bool {
                    #(#field_partial_eqs;)*
                    true
                }
            }

            impl<V> #eq for Storage<V> where V: #eq {}

            impl<V> #default for Storage<V> {
                #[inline]
                fn default() -> Storage<V> {
                    Storage {
                        #(#field_inits,)*
                    }
                }
            }

            impl<V> #storage_trait<#ident, V> for Storage<V> {
                type Iter<'this> = Iter<'this, V> where Self: 'this;
                type Values<'this> = Values<'this, V> where Self: 'this;
                type IterMut<'this> = IterMut<'this, V> where Self: 'this;
                type IntoIter = IntoIter<V>;

                #[inline]
                fn insert(&mut self, key: #ident, value: V) -> #option<V> {
                    match key {
                        #(#pattern => #insert,)*
                    }
                }

                #[inline]
                fn get(&self, value: #ident) -> #option<&V> {
                    match value {
                        #(#pattern => #get,)*
                    }
                }

                #[inline]
                fn get_mut(&mut self, value: #ident) -> #option<&mut V> {
                    match value {
                        #(#pattern => #get_mut,)*
                    }
                }

                #[inline]
                fn remove(&mut self, value: #ident) -> #option<V> {
                    match value {
                        #(#pattern => #remove,)*
                    }
                }

                #[inline]
                fn clear(&mut self) {
                    #(#clear;)*
                }

                #[inline]
                fn iter(&self) -> Self::Iter<'_> {
                    Iter {
                        step: 0,
                        #(#iter_init,)*
                    }
                }

                #[inline]
                fn values(&self) -> Self::Values<'_> {
                    Values {
                        step: 0,
                        #(#values_iter_init,)*
                    }
                }

                #[inline]
                fn iter_mut(&mut self) -> Self::IterMut<'_> {
                    IterMut {
                        step: 0,
                        #(#iter_mut_init,)*
                    }
                }

                #[inline]
                fn into_iter(self) -> Self::IntoIter {
                    IntoIter {
                        step: 0,
                        #(#into_iter_init,)*
                    }
                }
            }

            impl<V> #key_trait<#ident, V> for #ident {
                type Storage = Storage<V>;
            }

            #vis struct Iter<'a, V> {
                step: usize,
                #(#iter_fields,)*
            }

            impl<'a, V> #clone for Iter<'a, V> {
                #[inline]
                fn clone(&self) -> Iter<'a, V> {
                    Iter {
                        step: self.step,
                        #(#iter_clone,)*
                    }
                }
            }

            impl<'a, V> #iterator for Iter<'a, V> {
                type Item = (#ident, &'a V);

                #[inline]
                fn next(&mut self) -> #option<Self::Item> {
                    loop {
                        match self.step {
                            #(#iter_next,)*
                            _ => return #option::None,
                        }
                    }
                }
            }

            #vis struct Values<'a, V> {
                step: usize,
                #(#values_iter_fields,)*
            }

            impl<'a, V> #clone for Values<'a, V> {
                #[inline]
                fn clone(&self) -> Values<'a, V> {
                    Values {
                        step: self.step,
                        #(#iter_clone,)*
                    }
                }
            }

            impl<'a, V> #iterator for Values<'a, V> {
                type Item = &'a V;

                #[inline]
                fn next(&mut self) -> #option<Self::Item> {
                    loop {
                        match self.step {
                            #(#values_iter_next,)*
                            _ => return #option::None,
                        }
                    }
                }
            }

            #vis struct IterMut<'a, V> {
                step: usize,
                #(#iter_mut_fields,)*
            }

            impl<'a, V> #iterator for IterMut<'a, V> {
                type Item = (#ident, &'a mut V);

                #[inline]
                fn next(&mut self) -> #option<Self::Item> {
                    loop {
                        match self.step {
                            #(#iter_mut_next,)*
                            _ => return #option::None,
                        }
                    }
                }
            }

            #vis struct IntoIter<V> {
                step: usize,
                #(#into_iter_fields,)*
            }

            impl<V> Iterator for IntoIter<V> {
                type Item = (#ident, V);

                #[inline]
                fn next(&mut self) -> Option<Self::Item> {
                    loop {
                        match self.step {
                            #(#into_iter_next,)*
                            _ => return None,
                        }
                    }
                }
            }
        };
    }
}
