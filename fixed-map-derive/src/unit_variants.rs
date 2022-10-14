use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{DataEnum, Ident};

use crate::context::Ctxt;

/// Every variant is a unit variant.
pub(crate) fn implement(cx: &Ctxt, en: &DataEnum) -> Result<TokenStream, ()> {
    let vis = &cx.ast.vis;
    let ident = &cx.ast.ident;

    let clone = &cx.toks.clone;
    let copy = &cx.toks.copy;
    let default = &cx.toks.default;
    let eq = &cx.toks.eq;
    let into_iter = &cx.toks.into_iter;
    let iterator = &cx.toks.iterator_t;
    let iterator_flatten = &cx.toks.iterator_flatten;
    let key_trait = &cx.toks.key_trait;
    let mem = &cx.toks.mem;
    let option = &cx.toks.option;
    let partial_eq = &cx.toks.partial_eq;
    let slice_iter = &cx.toks.slice_iter;
    let slice_iter_mut = &cx.toks.slice_iter_mut;
    let array_into_iter = &cx.toks.array_into_iter;
    let storage_trait = &cx.toks.storage_trait;
    let iterator_flat_map = &cx.toks.iterator_flat_map;

    let const_wrapper = Ident::new(
        &format!("__IMPL_KEY_FOR_{}", cx.ast.ident),
        Span::call_site(),
    );

    let mut len = Vec::new();
    let mut is_empty = Vec::new();
    let mut pattern = Vec::new();
    let mut names = Vec::new();
    let mut fields = Vec::new();
    let mut field_inits = Vec::new();
    let mut contains_key = Vec::new();
    let mut get = Vec::new();
    let mut get_mut = Vec::new();
    let mut insert = Vec::new();
    let mut remove = Vec::new();
    let mut retain = Vec::new();
    let mut keys_iter_init = Vec::new();
    let mut iter_init = Vec::new();

    for (index, variant) in en.variants.iter().enumerate() {
        let var = &variant.ident;
        let name = Ident::new(&format!("f{}", index), Span::call_site());

        len.push(quote!(usize::from(#option::is_some(#name))));
        is_empty.push(quote!(#option::is_none(#name)));
        field_inits.push(quote!(#option::None));
        fields.push(quote!(#option<V>));
        pattern.push(quote!(#ident::#var));
        contains_key.push(quote!(#option::is_some(#name)));
        get.push(quote!(#option::as_ref(#name)));
        get_mut.push(quote!(#option::as_mut(#name)));
        insert.push(quote!(#mem::replace(#name, #option::Some(value))));
        remove.push(quote!(#mem::take(#name)));
        retain.push(quote! {
            if let Some(val) = #option::as_mut(#name) {
                if !func(#ident::#var, val) {
                    *#name = None;
                }
            }
        });
        keys_iter_init.push(quote!(if #name.is_some() { Some(#ident::#var) } else { None }));
        iter_init.push(quote!((#ident::#var, #name)));
        names.push(name.clone());
    }

    let count = en.variants.len();

    Ok(quote! {
        const #const_wrapper: () = {
            #[repr(transparent)]
            #vis struct Storage<V> {
                data: [#option<V>; #count],
            }

            #[automatically_derived]
            impl<V> #clone for Storage<V> where V: #clone {
                #[inline]
                fn clone(&self) -> Storage<V> {
                    Storage {
                        data: #clone::clone(&self.data),
                    }
                }
            }

            #[automatically_derived]
            impl<V> #copy for Storage<V> where V: #copy {
            }

            #[automatically_derived]
            impl<V> #partial_eq for Storage<V> where V: #partial_eq{
                #[inline]
                fn eq(&self, other: &Storage<V>) -> bool {
                    self.data == other.data
                }
            }

            #[automatically_derived]
            impl<V> #eq for Storage<V> where V: #eq {}

            #[automatically_derived]
            impl<V> #default for Storage<V> {
                #[inline]
                fn default() -> Storage<V> {
                    Storage {
                        data: [#(#field_inits),*],
                    }
                }
            }

            #[automatically_derived]
            impl<V> #storage_trait<#ident, V> for Storage<V> {
                type Iter<'this> = #iterator_flat_map<
                    #array_into_iter<(#ident, &'this #option<V>), #count>,
                    #option<(#ident, &'this V)>,
                    fn((#ident, &'this #option<V>)) -> #option<(#ident, &'this V)>
                > where Self: 'this;
                type Keys<'this> = #iterator_flatten<#array_into_iter<#option<#ident>, #count>> where V: 'this;
                type Values<'this> = #iterator_flatten<#slice_iter<'this, #option<V>>> where Self: 'this;
                type IterMut<'this> = #iterator_flat_map<
                    #array_into_iter<(#ident, &'this mut #option<V>), #count>,
                    #option<(#ident, &'this mut V)>,
                    fn((#ident, &'this mut #option<V>)) -> #option<(#ident, &'this mut V)>
                > where Self: 'this;
                type ValuesMut<'this> = #iterator_flatten<#slice_iter_mut<'this, #option<V>>> where Self: 'this;
                type IntoIter = #iterator_flat_map<
                    #array_into_iter<(#ident, #option<V>), #count>,
                    #option<(#ident, V)>,
                    fn((#ident, #option<V>)) -> #option<(#ident, V)>
                >;

                #[inline]
                fn len(&self) -> usize {
                    let [#(#names),*] = &self.data;
                    0 #(+ #len)*
                }

                #[inline]
                fn is_empty(&self) -> bool {
                    let [#(#names),*] = &self.data;
                    true #(&& #is_empty)*
                }

                #[inline]
                fn insert(&mut self, key: #ident, value: V) -> #option<V> {
                    let [#(#names),*] = &mut self.data;

                    match key {
                        #(#pattern => #insert,)*
                    }
                }

                #[inline]
                fn contains_key(&self, value: #ident) -> bool {
                    let [#(#names),*] = &self.data;

                    match value {
                        #(#pattern => #contains_key,)*
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
                fn retain<F>(&mut self, mut func: F)
                where
                    F: FnMut(#ident, &mut V) -> bool
                {
                    let [#(#names),*] = &mut self.data;
                    #(#retain)*
                }

                #[inline]
                fn clear(&mut self) {
                    self.data = [#(#field_inits),*];
                }

                #[inline]
                fn iter(&self) -> Self::Iter<'_> {
                    let [#(#names),*] = &self.data;
                    #iterator::flat_map(#into_iter([#(#iter_init),*]), |(k, v)| #option::Some((k, #option::as_ref(v)?)))
                }

                #[inline]
                fn keys(&self) -> Self::Keys<'_> {
                    let [#(#names),*] = &self.data;
                    #iterator::flatten(#into_iter([#(#keys_iter_init),*]))
                }

                #[inline]
                fn values(&self) -> Self::Values<'_> {
                    #iterator::flatten(#into_iter(&self.data))
                }

                #[inline]
                fn iter_mut(&mut self) -> Self::IterMut<'_> {
                    let [#(#names),*] = &mut self.data;
                    #iterator::flat_map(#into_iter([#(#iter_init),*]), |(k, v)| #option::Some((k, #option::as_mut(v)?)))
                }

                #[inline]
                fn values_mut(&mut self) -> Self::ValuesMut<'_> {
                    #iterator::flatten(#into_iter(&mut self.data))
                }

                #[inline]
                fn into_iter(self) -> Self::IntoIter {
                    let [#(#names),*] = self.data;
                    #iterator::flat_map(#into_iter([#(#iter_init),*]), |(k, v)| #option::Some((k, v?)))
                }
            }

            #[automatically_derived]
            impl #key_trait for #ident {
                type Storage<V> = Storage<V>;
            }
        };
    })
}
