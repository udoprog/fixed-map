use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{DataEnum, Ident};

use crate::context::Ctxt;

/// Every variant is a unit variant.
pub(crate) fn implement(cx: &Ctxt<'_>, en: &DataEnum) -> Result<TokenStream, ()> {
    let vis = &cx.ast.vis;
    let ident = &cx.ast.ident;

    let lt = cx.lt;
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
    let storage_entry_trait = &cx.toks.storage_entry_trait;
    let occupied_entry_trait = &cx.toks.occupied_entry_trait;
    let vacant_entry_trait = &cx.toks.vacant_entry_trait;
    let entry_enum = &cx.toks.entry_enum;
    let option_bucket_option = &cx.toks.option_bucket_option;
    let option_bucket_some = &cx.toks.option_bucket_some;
    let option_bucket_none = &cx.toks.option_bucket_none;

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
    let mut entry = Vec::new();

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
        entry.push(quote!(option_to_entry(#name, key)));
    }

    let count = en.variants.len();

    let entry_impl = if cfg!(feature = "entry") {
        quote! {
            #vis struct VacantEntry<#lt, V> {
                key: #ident,
                inner: #option_bucket_none<#lt, V>,
            }

            #[automatically_derived]
            impl<#lt, V> #vacant_entry_trait<#lt, #ident, V> for VacantEntry<#lt, V> {
                #[inline]
                fn key(&self) -> #ident {
                    self.key
                }

                #[inline]
                fn insert(self, value: V) -> &#lt mut V {
                    #option_bucket_none::insert(self.inner, value)
                }
            }

            #vis struct OccupiedEntry<#lt, V> {
                key: #ident,
                inner: #option_bucket_some<#lt, V>,
            }

            #[automatically_derived]
            impl<#lt, V> #occupied_entry_trait<#lt, #ident, V> for OccupiedEntry<#lt, V> {
                #[inline]
                fn key(&self) -> #ident {
                    self.key
                }

                #[inline]
                fn get(&self) -> &V {
                    #option_bucket_some::as_ref(&self.inner)
                }

                #[inline]
                fn get_mut(&mut self) -> &mut V {
                    #option_bucket_some::as_mut(&mut self.inner)
                }

                #[inline]
                fn into_mut(self) -> &#lt mut V {
                    #option_bucket_some::into_mut(self.inner)
                }

                #[inline]
                fn insert(&mut self, value: V) -> V {
                    #option_bucket_some::replace(&mut self.inner, value)
                }

                #[inline]
                fn remove(self) -> V {
                    #option_bucket_some::take(self.inner)
                }
            }

            #[inline]
            fn option_to_entry<V>(opt: &mut #option<V>, key: #ident) -> #entry_enum<'_, Storage<V>, #ident, V> {
                match #option_bucket_option::new(opt) {
                    #option_bucket_option::Some(inner) => #entry_enum::Occupied(OccupiedEntry { key, inner }),
                    #option_bucket_option::None(inner) => #entry_enum::Vacant(VacantEntry { key, inner }),
                }
            }

            #[automatically_derived]
            impl<V> #storage_entry_trait<#ident, V> for Storage<V> {
                type Occupied<#lt> = OccupiedEntry<#lt, V> where V: #lt;
                type Vacant<#lt> = VacantEntry<#lt, V> where V: #lt;

                #[inline]
                fn entry(&mut self, key: #ident) -> #entry_enum<'_, Self, #ident, V> {
                    let [#(#names),*] = &mut self.data;

                    match key {
                        #(#pattern => #entry,)*
                    }
                }
            }
        }
    } else {
        quote!()
    };

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
                type Iter<#lt> = #iterator_flat_map<
                    #array_into_iter<(#ident, &#lt #option<V>), #count>,
                    #option<(#ident, &#lt V)>,
                    fn((#ident, &#lt #option<V>)) -> #option<(#ident, &#lt V)>
                > where V: #lt;
                type Keys<#lt> = #iterator_flatten<#array_into_iter<#option<#ident>, #count>> where V: #lt;
                type Values<#lt> = #iterator_flatten<#slice_iter<#lt, #option<V>>> where V: #lt;
                type IterMut<#lt> = #iterator_flat_map<
                    #array_into_iter<(#ident, &#lt mut #option<V>), #count>,
                    #option<(#ident, &#lt mut V)>,
                    fn((#ident, &#lt mut #option<V>)) -> #option<(#ident, &#lt mut V)>
                > where V: #lt;
                type ValuesMut<#lt> = #iterator_flatten<#slice_iter_mut<#lt, #option<V>>> where V: #lt;
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

            #entry_impl
        };
    })
}
