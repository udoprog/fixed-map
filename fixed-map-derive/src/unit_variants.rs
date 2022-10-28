use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::{DataEnum, Ident};

use crate::context::Ctxt;

/// Every variant is a unit variant.
pub(crate) fn implement(cx: &Ctxt<'_>, en: &DataEnum) -> Result<TokenStream, ()> {
    let vis = &cx.ast.vis;
    let ident = &cx.ast.ident;
    let lt = cx.lt;

    let array_into_iter = cx.toks.array_into_iter();
    let clone_t = cx.toks.clone_t();
    let copy_t = cx.toks.copy_t();
    let default = cx.toks.default_t();
    let entry_enum = cx.toks.entry_enum();
    let eq = cx.toks.eq_t();
    let hash = cx.toks.hash_t();
    let hasher = cx.toks.hasher_t();
    let into_iterator_t = cx.toks.into_iterator_t();
    let iterator_cmp = cx.toks.iterator_cmp();
    let iterator_flat_map = cx.toks.iterator_flat_map();
    let iterator_flatten = cx.toks.iterator_flatten();
    let iterator_partial_cmp = cx.toks.iterator_partial_cmp();
    let iterator_t = cx.toks.iterator_t();
    let key_t = cx.toks.key_t();
    let mem = cx.toks.mem();
    let occupied_entry_t = cx.toks.occupied_entry_t();
    let option = cx.toks.option();
    let option_bucket_none = cx.toks.option_bucket_none();
    let option_bucket_option = cx.toks.option_bucket_option();
    let option_bucket_some = cx.toks.option_bucket_some();
    let ord_t = cx.toks.ord_t();
    let ordering = cx.toks.ordering();
    let partial_eq_t = cx.toks.partial_eq_t();
    let partial_ord_t = cx.toks.partial_ord_t();
    let slice_iter = cx.toks.slice_iter();
    let slice_iter_mut = cx.toks.slice_iter_mut();
    let storage_t = cx.toks.storage_t();
    let vacant_entry_t = cx.toks.vacant_entry_t();

    let const_wrapper = Ident::new(
        &format!("__IMPL_KEY_FOR_{}", cx.ast.ident),
        Span::call_site(),
    );

    let count = en.variants.len();
    let mut variants = Vec::with_capacity(count);
    let mut names = Vec::with_capacity(count);
    let mut field_inits = Vec::with_capacity(count);

    for (index, variant) in en.variants.iter().enumerate() {
        field_inits.push(quote!(#option::None));
        variants.push(&variant.ident);
        names.push(format_ident!("_{}", index));
    }

    let entry_impl = quote! {
        #vis struct VacantEntry<#lt, V> {
            key: #ident,
            inner: #option_bucket_none<#lt, V>,
        }

        #[automatically_derived]
        impl<#lt, V> #vacant_entry_t<#lt, #ident, V> for VacantEntry<#lt, V> {
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
        impl<#lt, V> #occupied_entry_t<#lt, #ident, V> for OccupiedEntry<#lt, V> {
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
    };

    Ok(quote! {
        const #const_wrapper: () = {
            #[repr(transparent)]
            #vis struct Storage<V> {
                data: [#option<V>; #count],
            }

            #[automatically_derived]
            impl<V> #clone_t for Storage<V> where V: #clone_t {
                #[inline]
                fn clone(&self) -> Storage<V> {
                    Storage {
                        data: #clone_t::clone(&self.data),
                    }
                }
            }

            #[automatically_derived]
            impl<V> #copy_t for Storage<V> where V: #copy_t {
            }

            #[automatically_derived]
            impl<V> #partial_eq_t for Storage<V> where V: #partial_eq_t {
                #[inline]
                fn eq(&self, other: &Storage<V>) -> bool {
                    #partial_eq_t::eq(&self.data, &other.data)
                }

                #[inline]
                fn ne(&self, other: &Storage<V>) -> bool {
                    #partial_eq_t::ne(&self.data, &other.data)
                }
            }

            #[automatically_derived]
            impl<V> #eq for Storage<V> where V: #eq {}

            #[automatically_derived]
            impl<V> #hash for Storage<V> where V: #hash {
                #[inline]
                fn hash<H>(&self, state: &mut H)
                where
                    H: #hasher,
                {
                    #hash::hash(&self.data, state);
                }
            }

            #[automatically_derived]
            impl<V> #partial_ord_t for Storage<V> where V: #partial_ord_t {
                #[inline]
                fn partial_cmp(&self, other: &Self) -> Option<#ordering> {
                    #iterator_partial_cmp(&self.data, &other.data)
                }
            }

            #[automatically_derived]
            impl<V> #ord_t for Storage<V> where V: #ord_t {
                #[inline]
                fn cmp(&self, other: &Self) -> #ordering {
                    #iterator_cmp(&self.data, &other.data)
                }
            }

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
            impl<V> #storage_t<#ident, V> for Storage<V> {
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
                type Occupied<#lt> = OccupiedEntry<#lt, V> where V: #lt;
                type Vacant<#lt> = VacantEntry<#lt, V> where V: #lt;

                #[inline]
                fn len(&self) -> usize {
                    let [#(#names),*] = &self.data;
                    0 #(+ usize::from(#option::is_some(#names)))*
                }

                #[inline]
                fn is_empty(&self) -> bool {
                    let [#(#names),*] = &self.data;
                    true #(&& #option::is_none(#names))*
                }

                #[inline]
                fn insert(&mut self, key: #ident, value: V) -> #option<V> {
                    let [#(#names),*] = &mut self.data;

                    match key {
                        #(#ident::#variants => #mem::replace(#names, #option::Some(value)),)*
                    }
                }

                #[inline]
                fn contains_key(&self, value: #ident) -> bool {
                    let [#(#names),*] = &self.data;

                    match value {
                        #(#ident::#variants => #option::is_some(#names),)*
                    }
                }

                #[inline]
                fn get(&self, value: #ident) -> #option<&V> {
                    let [#(#names),*] = &self.data;

                    match value {
                        #(#ident::#variants => #option::as_ref(#names),)*
                    }
                }

                #[inline]
                fn get_mut(&mut self, value: #ident) -> #option<&mut V> {
                    let [#(#names),*] = &mut self.data;

                    match value {
                        #(#ident::#variants => #option::as_mut(#names),)*
                    }
                }

                #[inline]
                fn remove(&mut self, value: #ident) -> #option<V> {
                    let [#(#names),*] = &mut self.data;

                    match value {
                        #(#ident::#variants => #mem::take(#names),)*
                    }
                }

                #[inline]
                fn retain<F>(&mut self, mut func: F)
                where
                    F: FnMut(#ident, &mut V) -> bool
                {
                    let [#(#names),*] = &mut self.data;

                    #(if let #option::Some(val) = #option::as_mut(#names) {
                        if !func(#ident::#variants, val) {
                            *#names = None;
                        }
                    })*
                }

                #[inline]
                fn clear(&mut self) {
                    self.data = [#(#field_inits),*];
                }

                #[inline]
                fn iter(&self) -> Self::Iter<'_> {
                    let [#(#names),*] = &self.data;
                    #iterator_t::flat_map(#into_iterator_t::into_iter([#((#ident::#variants, #names)),*]), |(k, v)| #option::Some((k, #option::as_ref(v)?)))
                }

                #[inline]
                fn keys(&self) -> Self::Keys<'_> {
                    let [#(#names),*] = &self.data;
                    #iterator_t::flatten(#into_iterator_t::into_iter([#(if #names.is_some() { Some(#ident::#variants) } else { None }),*]))
                }

                #[inline]
                fn values(&self) -> Self::Values<'_> {
                    #iterator_t::flatten(#into_iterator_t::into_iter(&self.data))
                }

                #[inline]
                fn iter_mut(&mut self) -> Self::IterMut<'_> {
                    let [#(#names),*] = &mut self.data;
                    #iterator_t::flat_map(#into_iterator_t::into_iter([#((#ident::#variants, #names)),*]), |(k, v)| #option::Some((k, #option::as_mut(v)?)))
                }

                #[inline]
                fn values_mut(&mut self) -> Self::ValuesMut<'_> {
                    #iterator_t::flatten(#into_iterator_t::into_iter(&mut self.data))
                }

                #[inline]
                fn into_iter(self) -> Self::IntoIter {
                    let [#(#names),*] = self.data;
                    #iterator_t::flat_map(#into_iterator_t::into_iter([#((#ident::#variants, #names)),*]), |(k, v)| #option::Some((k, v?)))
                }

                #[inline]
                fn entry(&mut self, key: #ident) -> #entry_enum<'_, Self, #ident, V> {
                    let [#(#names),*] = &mut self.data;

                    match key {
                        #(#ident::#variants => option_to_entry(#names, key),)*
                    }
                }
            }

            #[automatically_derived]
            impl #key_t for #ident {
                type Storage<V> = Storage<V>;
            }

            #entry_impl
        };
    })
}
