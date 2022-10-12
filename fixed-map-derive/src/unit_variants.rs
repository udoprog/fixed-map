use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::spanned::Spanned;
use syn::{DataEnum, Ident};

use crate::context::Ctxt;

/// Every variant is a unit variant.
pub(crate) fn implement(cx: &Ctxt, en: &DataEnum) -> Result<TokenStream, ()> {
    let vis = &cx.ast.vis;
    let ident = &cx.ast.ident;

    let lt = cx.lt;
    let array_into_iter = &cx.toks.array_into_iter;
    let clone = &cx.toks.clone;
    let copy = &cx.toks.copy;
    let default = &cx.toks.default;
    let eq = &cx.toks.eq;
    let into_iter = &cx.toks.into_iter;
    let iterator = &cx.toks.iterator_t;
    let double_ended_iterator_t = &cx.toks.double_ended_iterator_t;
    let key_trait = &cx.toks.key_trait;
    let mem = &cx.toks.mem;
    let option = &cx.toks.option;
    let partial_eq = &cx.toks.partial_eq;
    let slice_iter = &cx.toks.slice_iter;
    let slice_iter_mut = &cx.toks.slice_iter_mut;
    let storage_trait = &cx.toks.storage_trait;

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
    let mut get = Vec::new();
    let mut get_mut = Vec::new();
    let mut insert = Vec::new();
    let mut remove = Vec::new();
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
        get.push(quote!(#option::as_ref(#name)));
        get_mut.push(quote!(#option::as_mut(#name)));
        insert.push(quote!(#mem::replace(#name, #option::Some(value))));
        remove.push(quote!(#mem::take(#name)));
        keys_iter_init.push(quote!(if #name.is_some() { Some(#ident::#var) } else { None }));
        iter_init.push(quote!((#ident::#var, #name)));
        names.push(name.clone());
    }

    let count = en.variants.len();
    let next = syn::Ident::new("next", cx.ast.span());
    let nth = syn::Ident::new("nth", cx.ast.span());
    let next_back = syn::Ident::new("next_back", cx.ast.span());
    let nth_back = syn::Ident::new("nth_back", cx.ast.span());

    let mut iterators = Vec::new();

    {
        iterators.push(quote! {
            #[repr(transparent)]
            #vis struct Iter<#lt, V> {
                iter: #array_into_iter<(#ident, &#lt #option<V>), #count>,
            }

            impl<#lt, V> #clone for Iter<#lt, V> {
                #[inline]
                fn clone(&self) -> Iter<#lt, V> {
                    Iter {
                        iter: #clone::clone(&self.iter),
                    }
                }
            }
        });

        let build = |t, item, next, nth| {
            quote! {
                impl<#lt, V> #t for Iter<#lt, V> {
                    #item

                    #[inline]
                    fn #next(&mut self) -> #option<Self::Item> {
                        loop {
                            if let (key, #option::Some(value)) = #t::#next(&mut self.iter)? {
                                return #option::Some((key, value));
                            }
                        }
                    }

                    #[inline]
                    fn #nth(&mut self, n: usize) -> #option<Self::Item> {
                        loop {
                            if let (key, #option::Some(value)) = #t::#nth(&mut self.iter, n)? {
                                return #option::Some((key, value));
                            }
                        }
                    }
                }
            }
        };

        iterators.push(build(
            iterator,
            Some(quote!(type Item = (#ident, &#lt V);)),
            &next,
            &nth,
        ));
        iterators.push(build(double_ended_iterator_t, None, &next_back, &nth_back));
    }

    {
        iterators.push(quote! {
            #[repr(transparent)]
            #vis struct Keys {
                iter: #array_into_iter<#option<#ident>, #count>,
            }

            impl #clone for Keys {
                #[inline]
                fn clone(&self) -> Self {
                    Self {
                        iter: #clone::clone(&self.iter),
                    }
                }
            }
        });

        let build = |t, item, next, nth| {
            quote! {
                impl #t for Keys {
                    #item

                    #[inline]
                    fn #next(&mut self) -> #option<Self::Item> {
                        loop {
                            if let #option::Some(key) = #t::#next(&mut self.iter)? {
                                return #option::Some(key);
                            }
                        }
                    }

                    #[inline]
                    fn #nth(&mut self, n: usize) -> #option<Self::Item> {
                        loop {
                            if let #option::Some(key) = #t::#nth(&mut self.iter, n)? {
                                return #option::Some(key);
                            }
                        }
                    }
                }
            }
        };

        iterators.push(build(
            iterator,
            Some(quote!(type Item = #ident;)),
            &next,
            &nth,
        ));
        iterators.push(build(double_ended_iterator_t, None, &next_back, &nth_back));
    }

    {
        iterators.push(quote! {
            #[repr(transparent)]
            #vis struct Values<#lt, V> {
                iter: #slice_iter<#lt, #option<V>>,
            }

            impl<#lt, V> #clone for Values<#lt, V> {
                #[inline]
                fn clone(&self) -> Self {
                    Self {
                        iter: #clone::clone(&self.iter),
                    }
                }
            }
        });

        let build = |t, item, next, nth| {
            quote! {
                impl<#lt, V> #t for Values<#lt, V> {
                    #item

                    #[inline]
                    fn #next(&mut self) -> #option<Self::Item> {
                        loop {
                            if let #option::Some(value) = #t::#next(&mut self.iter)? {
                                return #option::Some(value);
                            }
                        }
                    }

                    #[inline]
                    fn #nth(&mut self, n: usize) -> #option<Self::Item> {
                        loop {
                            if let #option::Some(value) = #t::#nth(&mut self.iter, n)? {
                                return #option::Some(value);
                            }
                        }
                    }
                }
            }
        };

        iterators.push(build(
            iterator,
            Some(quote!(type Item = &#lt V;)),
            &next,
            &nth,
        ));
        iterators.push(build(double_ended_iterator_t, None, &next_back, &nth_back));
    }

    {
        iterators.push(quote! {
            #[repr(transparent)]
            #vis struct IterMut<#lt, V> {
                iter: #array_into_iter<(#ident, &#lt mut #option<V>), #count>,
            }
        });

        let build = |t, item, next, nth| {
            quote! {
                impl<#lt, V> #t for IterMut<#lt, V> {
                    #item

                    #[inline]
                    fn #next(&mut self) -> #option<Self::Item> {
                        loop {
                            if let (key, #option::Some(value)) = #t::#next(&mut self.iter)? {
                                return #option::Some((key, value));
                            }
                        }
                    }

                    #[inline]
                    fn #nth(&mut self, n: usize) -> #option<Self::Item> {
                        loop {
                            if let (key, #option::Some(value)) = #t::#nth(&mut self.iter, n)? {
                                return #option::Some((key, value));
                            }
                        }
                    }
                }
            }
        };

        iterators.push(build(
            iterator,
            Some(quote!(type Item = (#ident, &#lt mut V);)),
            &next,
            &nth,
        ));
        iterators.push(build(double_ended_iterator_t, None, &next_back, &nth_back));
    }

    {
        iterators.push(quote! {
            #[repr(transparent)]
            #vis struct ValuesMut<#lt, V> {
                iter: #slice_iter_mut<#lt, #option<V>>,
            }
        });

        let build = |t, item, next, nth| {
            quote! {
                impl<#lt, V> #t for ValuesMut<#lt, V> {
                    #item

                    #[inline]
                    fn #next(&mut self) -> #option<Self::Item> {
                        loop {
                            if let #option::Some(value) = #t::#next(&mut self.iter)? {
                                return #option::Some(value);
                            }
                        }
                    }

                    #[inline]
                    fn #nth(&mut self, n: usize) -> #option<Self::Item> {
                        loop {
                            if let #option::Some(value) = #t::#nth(&mut self.iter, n)? {
                                return #option::Some(value);
                            }
                        }
                    }
                }
            }
        };

        iterators.push(build(
            iterator,
            Some(quote!(type Item = &#lt mut V;)),
            &next,
            &nth,
        ));
        iterators.push(build(double_ended_iterator_t, None, &next_back, &nth_back));
    }

    {
        iterators.push(quote! {
            #[repr(transparent)]
            #vis struct IntoIter<V> {
                iter: #array_into_iter<(#ident, #option<V>), #count>,
            }
        });

        let build = |t, item, next, nth| {
            quote! {
                impl<V> #t for IntoIter<V> {
                    #item

                    #[inline]
                    fn #next(&mut self) -> #option<Self::Item> {
                        loop {
                            if let (key, #option::Some(value)) = #t::#next(&mut self.iter)? {
                                return #option::Some((key, value));
                            }
                        }
                    }

                    #[inline]
                    fn #nth(&mut self, n: usize) -> #option<Self::Item> {
                        loop {
                            if let (key, #option::Some(value)) = #t::#nth(&mut self.iter, n)? {
                                return #option::Some((key, value));
                            }
                        }
                    }
                }
            }
        };

        iterators.push(build(
            iterator,
            Some(quote!(type Item = (#ident, V);)),
            &next,
            &nth,
        ));
        iterators.push(build(double_ended_iterator_t, None, &next_back, &nth_back));
    }

    Ok(quote! {
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

            impl<V> #partial_eq for Storage<V> where V: #partial_eq{
                #[inline]
                fn eq(&self, other: &Storage<V>) -> bool {
                    self.data == other.data
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
                type Keys<'this> = Keys where Self: 'this;
                type Values<'this> = Values<'this, V> where Self: 'this;
                type IterMut<'this> = IterMut<'this, V> where Self: 'this;
                type ValuesMut<'this> = ValuesMut<'this, V> where Self: 'this;
                type IntoIter = IntoIter<V>;

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
                fn keys(&self) -> Self::Keys<'_> {
                    let [#(#names),*] = &self.data;

                    Keys {
                        iter: #into_iter([#(#keys_iter_init),*]),
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
                fn values_mut(&mut self) -> Self::ValuesMut<'_> {
                    ValuesMut {
                        iter: #into_iter(&mut self.data),
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

            #(#iterators)*
        };
    })
}
