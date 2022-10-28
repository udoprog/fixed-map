use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote, quote_spanned, ToTokens};
use syn::spanned::Spanned;
use syn::{DataEnum, Fields, Ident};

use crate::context::{Ctxt, FieldKind, FieldSpec};

pub(crate) fn implement(cx: &Ctxt<'_>, en: &DataEnum) -> Result<TokenStream, ()> {
    let vis = &cx.ast.vis;
    let ident = &cx.ast.ident;
    let lt = cx.lt;

    let clone_t = cx.toks.clone_t();
    let copy_t = cx.toks.copy_t();
    let eq_t = cx.toks.eq_t();
    let key_t = cx.toks.key_t();
    let mem = cx.toks.mem();
    let option = cx.toks.option();
    let partial_eq_t = cx.toks.partial_eq_t();
    let storage_t = cx.toks.storage_t();

    let const_wrapper = Ident::new(
        &format!("__IMPL_KEY_FOR_{}", cx.ast.ident),
        Span::call_site(),
    );

    let mut len = Vec::new();
    let mut is_empty = Vec::new();
    let mut pattern = Vec::new();
    let mut fields = Vec::new();
    let mut field_inits = Vec::new();
    let mut contains_key = Vec::new();
    let mut get = Vec::new();
    let mut get_mut = Vec::new();
    let mut insert = Vec::new();
    let mut remove = Vec::new();
    let mut retain = Vec::new();
    let mut clear = Vec::new();
    let mut copy_bounds = Vec::new();
    let mut field_specs = Vec::new();
    let mut names = Vec::new();

    for (index, variant) in en.variants.iter().enumerate() {
        let var = &variant.ident;
        let name = format_ident!("_{}", index);
        names.push(name.clone());

        let kind = match &variant.fields {
            Fields::Unit => {
                field_inits.push(quote!(#option::None));
                len.push(quote!(usize::from(#option::is_some(&self.#name))));
                is_empty.push(quote!(#option::is_none(&self.#name)));
                fields.push(quote!(#option<V>));
                pattern.push(quote!(#ident::#var));
                clear.push(quote!(self.#name = #option::None));
                contains_key.push(quote!(#option::is_some(&self.#name)));
                get.push(quote!(#option::as_ref(&self.#name)));
                get_mut.push(quote!(#option::as_mut(&mut self.#name)));
                insert.push(quote!(#mem::replace(&mut self.#name, #option::Some(value))));
                remove.push(quote!(#mem::replace(&mut self.#name, #option::None)));
                retain.push(quote! {
                    if let #option::Some(val) = #option::as_mut(&mut self.#name) {
                        if !func(#ident::#var, val) {
                            self.#name = None;
                        }
                    }
                });

                FieldKind::Simple
            }
            Fields::Unnamed(unnamed) => {
                if unnamed.unnamed.len() > 1 {
                    cx.error(
                        variant.fields.span(),
                        "unnamed variants must have a single field",
                    );
                    continue;
                }

                let element = unnamed.unnamed.first().expect("Expected one element");
                let storage = quote!(<#element as #key_t>::Storage::<V>);
                let as_storage = quote!(<#storage as #storage_t<#element, V>>);

                field_inits.push(quote!(#as_storage::empty()));
                len.push(quote!(#as_storage::len(&self.#name)));
                is_empty.push(quote!(#as_storage::is_empty(&self.#name)));

                fields.push(quote!(#storage));
                pattern.push(quote!(#ident::#var(v)));
                clear.push(quote!(#as_storage::clear(&mut self.#name)));
                contains_key.push(quote!(#as_storage::contains_key(&self.#name, v)));
                get.push(quote!(#as_storage::get(&self.#name, v)));
                get_mut.push(quote!(#as_storage::get_mut(&mut self.#name, v)));
                insert.push(quote!(#as_storage::insert(&mut self.#name, v, value)));
                remove.push(quote!(#as_storage::remove(&mut self.#name, v)));
                retain.push(quote! {
                    #as_storage::retain(&mut self.#name, |k, v| func(#ident::#var(k), v));
                });

                copy_bounds.push(storage.clone());

                FieldKind::Complex {
                    element: quote!(#element),
                    storage,
                    as_storage,
                }
            }
            Fields::Named(_) => {
                cx.error(variant.fields.span(), "named fields are not supported");
                continue;
            }
        };

        field_specs.push(FieldSpec {
            span: variant.span(),
            index,
            name,
            var,
            kind,
        });
    }

    let mut iter_clone = Vec::new();

    for FieldSpec { name, .. } in &field_specs {
        iter_clone.push(quote!(#name: #clone_t::clone(&self.#name)));
    }

    let pattern = &pattern;

    let (iter_impl, iter_init) = build_iter_impl(cx, "Iter", &field_specs)?;
    let (keys_impl, keys_iter_init) = build_keys_impl(cx, "Keys", &field_specs)?;
    let (values_impl, values_iter_init) = build_values_impl(cx, "Values", &field_specs)?;
    let (iter_mut_impl, iter_mut_init) = build_iter_mut_impl(cx, "IterMut", &field_specs)?;
    let (values_mut_impl, values_mut_init) = build_values_mut_impl(cx, "ValuesMut", &field_specs)?;
    let (into_iter_impl, into_iter_init) = build_into_iter_impl(cx, "IntoIter", &field_specs)?;
    let (entry_impl, entry_items_impl) = build_entry_impl(cx, &field_specs)?;

    let end = field_specs.len();

    Ok(quote! {
        const #const_wrapper: () = {
            #vis struct Storage<V> {
                #(#names: #fields,)*
            }

            #[automatically_derived]
            impl<V> #clone_t for Storage<V> where V: #clone_t {
                #[inline]
                fn clone(&self) -> Storage<V> {
                    Storage {
                        #(#names: #clone_t::clone(&self.#names),)*
                    }
                }
            }

            #[automatically_derived]
            impl<V> #copy_t for Storage<V> where V: #copy_t, #(#copy_bounds: #copy_t,)* {}

            #[automatically_derived]
            impl<V> #partial_eq_t for Storage<V> where V: #partial_eq_t {
                #[inline]
                fn eq(&self, other: &Storage<V>) -> bool {
                    #(if #partial_eq_t::ne(&self.#names, &other.#names) {
                        return false;
                    })*
                    true
                }
            }

            #[automatically_derived]
            impl<V> #eq_t for Storage<V> where V: #eq_t {}

            #[automatically_derived]
            impl<V> #storage_t<#ident, V> for Storage<V> {
                type Iter<#lt> = Iter<#lt, V> where V: #lt;
                type Keys<#lt> = Keys<#lt, V> where V: #lt;
                type Values<#lt> = Values<#lt, V> where V: #lt;
                type IterMut<#lt> = IterMut<#lt, V> where V: #lt;
                type ValuesMut<#lt> = ValuesMut<#lt, V> where V: #lt;
                type IntoIter = IntoIter<V>;

                #[inline]
                fn empty() -> Self {
                    Self {
                        #(#names: #field_inits,)*
                    }
                }

                #[inline]
                fn len(&self) -> usize {
                    #(#len)+*
                }

                #[inline]
                fn is_empty(&self) -> bool {
                    #(#is_empty)&&*
                }

                #[inline]
                fn insert(&mut self, key: #ident, value: V) -> #option<V> {
                    match key {
                        #(#pattern => #insert,)*
                    }
                }

                #[inline]
                fn contains_key(&self, value: #ident) -> bool {
                    match value {
                        #(#pattern => #contains_key,)*
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
                fn retain<F>(&mut self, mut func: F)
                where
                    F: FnMut(#ident, &mut V) -> bool
                {
                    #(#retain)*
                }

                #[inline]
                fn clear(&mut self) {
                    #(#clear;)*
                }

                #[inline]
                fn iter(&self) -> Self::Iter<'_> {
                    Iter {
                        start: 0,
                        end: #end,
                        #(#iter_init,)*
                    }
                }

                #[inline]
                fn keys(&self) -> Self::Keys<'_> {
                    Keys {
                        start: 0,
                        end: #end,
                        #(#keys_iter_init,)*
                    }
                }

                #[inline]
                fn values(&self) -> Self::Values<'_> {
                    Values {
                        start: 0,
                        end: #end,
                        #(#values_iter_init,)*
                    }
                }

                #[inline]
                fn iter_mut(&mut self) -> Self::IterMut<'_> {
                    IterMut {
                        start: 0,
                        end: #end,
                        #(#iter_mut_init,)*
                    }
                }

                #[inline]
                fn values_mut(&mut self) -> Self::ValuesMut<'_> {
                    ValuesMut {
                        start: 0,
                        end: #end,
                        #(#values_mut_init,)*
                    }
                }

                #[inline]
                fn into_iter(self) -> Self::IntoIter {
                    IntoIter {
                        start: 0,
                        end: #end,
                        #(#into_iter_init,)*
                    }
                }

                #entry_items_impl
            }

            #[automatically_derived]
            impl #key_t for #ident {
                type Storage<V> = Storage<V>;
            }

            #iter_impl
            #keys_impl
            #values_impl
            #iter_mut_impl
            #values_mut_impl
            #into_iter_impl

            #entry_impl
        };
    })
}

/// Build iterator next.
fn build_iter_next(
    cx: &Ctxt<'_>,
    step_forward: &mut IteratorNext,
    step_backward: &mut IteratorNextBack,
    field_specs: &[FieldSpec<'_>],
    assoc_type: &Ident,
    lt: Option<&syn::Lifetime>,
) -> Result<(), ()> {
    let option = cx.toks.option();
    let iterator_t = cx.toks.iterator_t();
    let double_ended_iterator_t = cx.toks.double_ended_iterator_t();
    let ident = &cx.ast.ident;

    for FieldSpec {
        span,
        index,
        name,
        var,
        kind,
        ..
    } in field_specs
    {
        match kind {
            FieldKind::Simple => {
                step_forward.next.push(quote! {
                    #index => {
                        if let #option::Some(value) = #option::take(&mut self.#name) {
                            return #option::Some((#ident::#var, value));
                        }
                    }
                });

                step_backward.next.push(quote! {
                    #index => {
                        if let #option::Some(value) = #option::take(&mut self.#name) {
                            return #option::Some((#ident::#var, value));
                        }
                    }
                });
            }
            FieldKind::Complex { as_storage, .. } => {
                step_forward.next.push(quote! {
                    #index => {
                        if let #option::Some((key, value)) = #iterator_t::next(&mut self.#name) {
                            return #option::Some((#ident::#var(key), value));
                        }
                    }
                });

                step_backward.next.push(quote! {
                    #index => {
                        if let #option::Some((key, value)) = #double_ended_iterator_t::next_back(&mut self.#name) {
                            return #option::Some((#ident::#var(key), value));
                        }
                    }
                });

                // NB: The `Item = ..` component of the bound is technically
                // superflous but currently necessary to satisfy rustc.
                let where_clause = step_backward.make_where_clause();

                let assoc_type = if let Some(lt) = lt {
                    quote!(#as_storage::#assoc_type<#lt>)
                } else {
                    quote!(#as_storage::#assoc_type)
                };

                where_clause.predicates.push(cx.fallible(|| syn::parse2(quote_spanned! {
                    *span => #assoc_type: #double_ended_iterator_t<Item = <#assoc_type as #iterator_t>::Item>
                }))?);
            }
        }
    }

    Ok(())
}

/// Construct an iterator implementation.
fn build_iter_impl(
    cx: &Ctxt<'_>,
    id: &str,
    field_specs: &[FieldSpec<'_>],
) -> Result<(TokenStream, Vec<TokenStream>), ()> {
    let type_name = syn::Ident::new(id, Span::call_site());

    let lt = cx.lt;
    let ident = &cx.ast.ident;
    let vis = &cx.ast.vis;

    let option = cx.toks.option();
    let iterator_t = cx.toks.iterator_t();
    let double_ended_iterator_t = cx.toks.double_ended_iterator_t();
    let clone_t = cx.toks.clone_t();

    let mut step_forward = IteratorNext::default();
    let mut step_backward = IteratorNextBack::default();

    let mut iter_clone = Vec::new();
    let mut field_decls = Vec::new();
    let mut init = Vec::new();

    build_iter_next(
        cx,
        &mut step_forward,
        &mut step_backward,
        field_specs,
        &type_name,
        Some(cx.lt),
    )?;

    for FieldSpec { name, kind, .. } in field_specs {
        iter_clone.push(quote!(#name: #clone_t::clone(&self.#name)));

        match kind {
            FieldKind::Simple => {
                field_decls.push(quote!(#name: #option<&#lt V>));
                init.push(quote!(#name: #option::as_ref(&self.#name)));
            }
            FieldKind::Complex { as_storage, .. } => {
                field_decls.push(quote!(#name: #as_storage::Iter<#lt>));
                init.push(quote!(#name: #as_storage::iter(&self.#name)));
            }
        }
    }

    step_backward
        .make_where_clause()
        .predicates
        .push(cx.fallible(|| syn::parse2(quote!(V: #lt)))?);

    let double_ended_where_clause = &step_backward.where_clause;

    let iter_impl = quote! {
        #vis struct #type_name<#lt, V> where V: #lt {
            start: usize,
            end: usize,
            #(#field_decls,)*
        }

        #[automatically_derived]
        impl<#lt, V> #clone_t for #type_name<#lt, V> where V: #lt {
            #[inline]
            fn clone(&self) -> Self {
                Self {
                    start: self.start,
                    end: self.end,
                    #(#iter_clone,)*
                }
            }
        }

        #[automatically_derived]
        impl<#lt, V> #iterator_t for #type_name<#lt, V> where V: #lt {
            type Item = (#ident, &#lt V);

            #[inline]
            fn next(&mut self) -> #option<Self::Item> {
                #step_forward
                #option::None
            }
        }

        #[automatically_derived]
        impl<#lt, V> #double_ended_iterator_t for #type_name<#lt, V> #double_ended_where_clause {
            #[inline]
            fn next_back(&mut self) -> #option<Self::Item> {
                #step_backward
                #option::None
            }
        }
    };

    Ok((iter_impl, init))
}

/// Constructs a key's `iterator_t` implementation.
fn build_keys_impl(
    cx: &Ctxt<'_>,
    id: &str,
    field_specs: &[FieldSpec<'_>],
) -> Result<(TokenStream, Vec<TokenStream>), ()> {
    let type_name = syn::Ident::new(id, Span::call_site());

    let lt = cx.lt;
    let ident = &cx.ast.ident;
    let vis = &cx.ast.vis;

    let bool_type = cx.toks.bool_type();
    let clone_t = cx.toks.clone_t();
    let double_ended_iterator_t = cx.toks.double_ended_iterator_t();
    let iterator_t = cx.toks.iterator_t();
    let mem = cx.toks.mem();
    let option = cx.toks.option();

    let mut step_forward = IteratorNext::default();
    let mut step_backward = IteratorNextBack::default();

    let mut iter_clone = Vec::new();
    let mut field_decls = Vec::new();
    let mut init = Vec::new();

    for FieldSpec {
        span,
        index,
        name,
        var,
        kind,
        ..
    } in field_specs
    {
        iter_clone.push(quote!(#name: #clone_t::clone(&self.#name)));

        match kind {
            FieldKind::Simple => {
                field_decls.push(quote!(#name: #bool_type));
                init.push(quote!(#name: #option::is_some(&self.#name)));

                step_forward.next.push(quote! {
                    #index => {
                        if #mem::take(&mut self.#name) {
                            return #option::Some(#ident::#var);
                        }
                    }
                });

                step_backward.next.push(quote! {
                    #index => {
                        if #mem::take(&mut self.#name) {
                            return #option::Some(#ident::#var);
                        }
                    }
                });
            }
            FieldKind::Complex { as_storage, .. } => {
                field_decls.push(quote!(#name: #as_storage::Keys<#lt>));
                init.push(quote!(#name: #as_storage::keys(&self.#name)));

                step_forward.next.push(quote! {
                    #index => {
                        if let #option::Some(key) = #iterator_t::next(&mut self.#name) {
                            return #option::Some(#ident::#var(key));
                        }
                    }
                });

                step_backward.next.push(quote! {
                    #index => {
                        if let #option::Some(key) = #double_ended_iterator_t::next_back(&mut self.#name) {
                            return #option::Some(#ident::#var(key));
                        }
                    }
                });

                let where_clause = step_backward.make_where_clause();

                let assoc_type = quote!(#as_storage::#type_name<#lt>);

                where_clause.predicates.push(cx.fallible(|| syn::parse2(quote_spanned! {
                    *span => #assoc_type: #double_ended_iterator_t<Item = <#assoc_type as #iterator_t>::Item>
                }))?);
            }
        }
    }

    step_backward
        .make_where_clause()
        .predicates
        .push(cx.fallible(|| syn::parse2(quote!(V: #lt)))?);

    let double_ended_where_clause = &step_backward.where_clause;

    let iter_impl = quote! {
        #vis struct #type_name<#lt, V> where V: #lt {
            start: usize,
            end: usize,
            #(#field_decls,)*
        }

        #[automatically_derived]
        impl<#lt, V> #clone_t for #type_name<#lt, V> where V: #lt {
            #[inline]
            fn clone(&self) -> Self {
                Self {
                    start: self.start,
                    end: self.end,
                    #(#iter_clone,)*
                }
            }
        }

        #[automatically_derived]
        impl<#lt, V> #iterator_t for #type_name<#lt, V> where V: #lt {
            type Item = #ident;

            #[inline]
            fn next(&mut self) -> #option<Self::Item> {
                #step_forward
                #option::None
            }
        }

        #[automatically_derived]
        impl<#lt, V> #double_ended_iterator_t for #type_name<#lt, V> #double_ended_where_clause {
            #[inline]
            fn next_back(&mut self) -> #option<Self::Item> {
                #step_backward
                #option::None
            }
        }
    };

    Ok((iter_impl, init))
}

/// Construct a values `iterator_t` implementation.
fn build_values_impl(
    cx: &Ctxt<'_>,
    id: &str,
    field_specs: &[FieldSpec<'_>],
) -> Result<(TokenStream, Vec<TokenStream>), ()> {
    let type_name = syn::Ident::new(id, Span::call_site());

    let lt = cx.lt;
    let vis = &cx.ast.vis;

    let clone_t = cx.toks.clone_t();
    let double_ended_iterator_t = cx.toks.double_ended_iterator_t();
    let iterator_t = cx.toks.iterator_t();
    let option = cx.toks.option();

    let mut step_forward = IteratorNext::default();
    let mut step_backward = IteratorNextBack::default();

    let mut iter_clone = Vec::new();
    let mut field_decls = Vec::new();
    let mut init = Vec::new();

    for FieldSpec {
        span,
        index,
        name,
        kind,
        ..
    } in field_specs
    {
        iter_clone.push(quote!(#name: #clone_t::clone(&self.#name)));

        match kind {
            FieldKind::Simple => {
                field_decls.push(quote!(#name: #option<&#lt V>));
                init.push(quote!(#name: #option::as_ref(&self.#name)));

                step_forward.next.push(quote! {
                    #index => {
                        if let #option::Some(value) = #option::take(&mut self.#name) {
                            return #option::Some(value);
                        }
                    }
                });

                step_backward.next.push(quote! {
                    #index => {
                        if let #option::Some(value) = #option::take(&mut self.#name) {
                            return #option::Some(value);
                        }
                    }
                });
            }
            FieldKind::Complex { as_storage, .. } => {
                field_decls.push(quote!(#name: #as_storage::Values<#lt>));
                init.push(quote!(#name: #as_storage::values(&self.#name)));

                step_forward.next.push(quote! {
                    #index => {
                        if let #option::Some(value) = #iterator_t::next(&mut self.#name) {
                            return #option::Some(value);
                        }
                    }
                });

                step_backward.next.push(quote! {
                    #index => {
                        if let #option::Some(value) = #double_ended_iterator_t::next_back(&mut self.#name) {
                            return #option::Some(value);
                        }
                    }
                });

                let where_clause = step_backward.make_where_clause();

                let assoc_type = quote!(#as_storage::#type_name<#lt>);

                where_clause.predicates.push(cx.fallible(|| syn::parse2(quote_spanned! {
                    *span => #assoc_type: #double_ended_iterator_t<Item = <#assoc_type as #iterator_t>::Item>
                }))?);
            }
        }
    }

    step_backward
        .make_where_clause()
        .predicates
        .push(cx.fallible(|| syn::parse2(quote!(V: #lt)))?);

    let double_ended_where_clause = &step_backward.where_clause;

    let iter_impl = quote! {
        #vis struct #type_name<#lt, V> where V: #lt {
            start: usize,
            end: usize,
            #(#field_decls,)*
        }

        #[automatically_derived]
        impl<#lt, V> #clone_t for #type_name<#lt, V> where V: #lt {
            #[inline]
            fn clone(&self) -> Self {
                Self {
                    start: self.start,
                    end: self.end,
                    #(#iter_clone,)*
                }
            }
        }

        #[automatically_derived]
        impl<#lt, V> #iterator_t for #type_name<#lt, V> where V: #lt {
            type Item = &#lt V;

            #[inline]
            fn next(&mut self) -> #option<Self::Item> {
                #step_forward
                #option::None
            }
        }

        #[automatically_derived]
        impl<#lt, V> #double_ended_iterator_t for #type_name<#lt, V> #double_ended_where_clause {
            #[inline]
            fn next_back(&mut self) -> #option<Self::Item> {
                #step_backward
                #option::None
            }
        }
    };

    Ok((iter_impl, init))
}

/// Construct an iterator implementation.
fn build_iter_mut_impl(
    cx: &Ctxt<'_>,
    id: &str,
    field_specs: &[FieldSpec<'_>],
) -> Result<(TokenStream, Vec<TokenStream>), ()> {
    let type_name = syn::Ident::new(id, Span::call_site());

    let ident = &cx.ast.ident;
    let lt = cx.lt;
    let vis = &cx.ast.vis;

    let double_ended_iterator_t = cx.toks.double_ended_iterator_t();
    let iterator_t = cx.toks.iterator_t();
    let option = cx.toks.option();

    let mut step_forward = IteratorNext::default();
    let mut step_backward = IteratorNextBack::default();

    let mut field_decls = Vec::new();
    let mut init = Vec::new();

    build_iter_next(
        cx,
        &mut step_forward,
        &mut step_backward,
        field_specs,
        &type_name,
        Some(cx.lt),
    )?;

    for FieldSpec { name, kind, .. } in field_specs {
        match kind {
            FieldKind::Simple => {
                field_decls.push(quote!(#name: #option<&#lt mut V>));
                init.push(quote!(#name: #option::as_mut(&mut self.#name)));
            }
            FieldKind::Complex {
                as_storage,
                storage,
                ..
            } => {
                field_decls.push(quote!(#name: #as_storage::IterMut<#lt>));
                init.push(quote!(#name: #storage::iter_mut(&mut self.#name)));
            }
        }
    }

    step_backward
        .make_where_clause()
        .predicates
        .push(cx.fallible(|| syn::parse2(quote!(V: #lt)))?);

    let double_ended_where = &step_backward.where_clause;

    let iter_impl = quote! {
        #vis struct #type_name<#lt, V> where V: #lt {
            start: usize,
            end: usize,
            #(#field_decls,)*
        }

        #[automatically_derived]
        impl<#lt, V> #iterator_t for #type_name<#lt, V> where V: #lt {
            type Item = (#ident, &#lt mut V);

            #[inline]
            fn next(&mut self) -> #option<Self::Item> {
                #step_forward
                #option::None
            }
        }

        #[automatically_derived]
        impl<#lt, V> #double_ended_iterator_t for #type_name<#lt, V> #double_ended_where {
            #[inline]
            fn next_back(&mut self) -> #option<Self::Item> {
                #step_backward
                #option::None
            }
        }
    };

    Ok((iter_impl, init))
}

/// Construct a values mutable `iterator_t` implementation.
fn build_values_mut_impl(
    cx: &Ctxt<'_>,
    id: &str,
    field_specs: &[FieldSpec<'_>],
) -> Result<(TokenStream, Vec<TokenStream>), ()> {
    let type_name = syn::Ident::new(id, Span::call_site());

    let lt = cx.lt;
    let vis = &cx.ast.vis;

    let option = cx.toks.option();
    let iterator_t = cx.toks.iterator_t();
    let clone_t = cx.toks.clone_t();
    let double_ended_iterator_t = cx.toks.double_ended_iterator_t();

    let mut step_forward = IteratorNext::default();
    let mut step_backward = IteratorNextBack::default();

    let mut iter_clone = Vec::new();
    let mut field_decls = Vec::new();
    let mut init = Vec::new();

    for FieldSpec {
        span,
        index,
        name,
        kind,
        ..
    } in field_specs
    {
        iter_clone.push(quote!(#name: #clone_t::clone(&self.#name)));

        match kind {
            FieldKind::Simple => {
                field_decls.push(quote!(#name: #option<&#lt mut V>));
                init.push(quote!(#name: #option::as_mut(&mut self.#name)));

                step_forward.next.push(quote! {
                    #index => {
                        if let #option::Some(value) = #option::take(&mut self.#name) {
                            return #option::Some(value);
                        }
                    }
                });

                step_backward.next.push(quote! {
                    #index => {
                        if let #option::Some(value) = #option::take(&mut self.#name) {
                            return #option::Some(value);
                        }
                    }
                });
            }
            FieldKind::Complex { as_storage, .. } => {
                field_decls.push(quote!(#name: #as_storage::ValuesMut<#lt>));
                init.push(quote!(#name: #as_storage::values_mut(&mut self.#name)));

                step_forward.next.push(quote! {
                    #index => {
                        if let #option::Some(value) = #iterator_t::next(&mut self.#name) {
                            return #option::Some(value);
                        }
                    }
                });

                step_backward.next.push(quote! {
                    #index => {
                        if let #option::Some(value) = #double_ended_iterator_t::next_back(&mut self.#name) {
                            return #option::Some(value);
                        }
                    }
                });

                let where_clause = step_backward.make_where_clause();

                let assoc_type = quote!(#as_storage::#type_name<#lt>);

                where_clause.predicates.push(cx.fallible(|| syn::parse2(quote_spanned! {
                    *span => #assoc_type: #double_ended_iterator_t<Item = <#assoc_type as #iterator_t>::Item>
                }))?);
            }
        }
    }

    step_backward
        .make_where_clause()
        .predicates
        .push(cx.fallible(|| syn::parse2(quote!(V: #lt)))?);

    let double_ended_where_clause = &step_backward.where_clause;

    let iter_impl = quote! {
        #vis struct #type_name<#lt, V> where V: #lt {
            start: usize,
            end: usize,
            #(#field_decls,)*
        }

        #[automatically_derived]
        impl<#lt, V> #iterator_t for #type_name<#lt, V> where V: #lt {
            type Item = &#lt mut V;

            #[inline]
            fn next(&mut self) -> #option<Self::Item> {
                #step_forward
                #option::None
            }
        }

        #[automatically_derived]
        impl<#lt, V> #double_ended_iterator_t for #type_name<#lt, V> #double_ended_where_clause {
            #[inline]
            fn next_back(&mut self) -> #option<Self::Item> {
                #step_backward
                #option::None
            }
        }
    };

    Ok((iter_impl, init))
}

/// Construct `IntoIter` implementation.
fn build_into_iter_impl(
    cx: &Ctxt<'_>,
    id: &str,
    field_specs: &[FieldSpec<'_>],
) -> Result<(TokenStream, Vec<TokenStream>), ()> {
    let type_name = syn::Ident::new(id, Span::call_site());

    let ident = &cx.ast.ident;
    let vis = &cx.ast.vis;

    let option = cx.toks.option();
    let clone_t = cx.toks.clone_t();
    let iterator_t = cx.toks.iterator_t();
    let double_ended_iterator_t = cx.toks.double_ended_iterator_t();

    let mut step_forward = IteratorNext::default();
    let mut step_backward = IteratorNextBack::default();

    let mut field_decls = Vec::new();
    let mut init = Vec::new();
    let mut field_clone = Vec::new();
    let mut clone_bounds = Vec::new();

    build_iter_next(
        cx,
        &mut step_forward,
        &mut step_backward,
        field_specs,
        &type_name,
        None,
    )?;

    for FieldSpec { name, kind, .. } in field_specs {
        field_clone.push(quote!(#name: #clone_t::clone(&self.#name)));

        match kind {
            FieldKind::Simple => {
                field_decls.push(quote!(#name: #option<V>));
                init.push(quote!(#name: self.#name));
            }
            FieldKind::Complex {
                as_storage,
                storage,
                ..
            } => {
                field_decls.push(quote!(#name: #as_storage::IntoIter));
                init.push(quote!(#name: #storage::into_iter(self.#name)));
                clone_bounds.push(quote!(#as_storage::IntoIter: #clone_t));
            }
        }
    }

    let double_ended_where = &step_backward.where_clause;

    let iter_impl = quote! {
        #vis struct #type_name<V> {
            start: usize,
            end: usize,
            #(#field_decls,)*
        }

        #[automatically_derived]
        impl<V> #clone_t for #type_name<V> where V: Clone, #(#clone_bounds,)* {
            #[inline]
            fn clone(&self) -> Self {
                Self {
                    start: self.start,
                    end: self.end,
                    #(#field_clone,)*
                }
            }
        }

        #[automatically_derived]
        impl<V> #iterator_t for #type_name<V> {
            type Item = (#ident, V);

            #[inline]
            fn next(&mut self) -> Option<Self::Item> {
                #step_forward
                #option::None
            }
        }

        #[automatically_derived]
        impl<V> #double_ended_iterator_t for #type_name<V> #double_ended_where {
            #[inline]
            fn next_back(&mut self) -> #option<Self::Item> {
                #step_backward
                #option::None
            }
        }
    };

    Ok((iter_impl, init))
}

#[derive(Default)]
struct IteratorNext {
    next: Vec<TokenStream>,
}

impl ToTokens for IteratorNext {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let iter_next = &self.next;

        tokens.extend(quote! {
            while self.start < self.end {
                match self.start {
                    #(#iter_next,)*
                    _ => break,
                }

                self.start = usize::min(self.start.wrapping_add(1), self.end);
            }
        });
    }
}

#[derive(Default)]
struct IteratorNextBack {
    next: Vec<TokenStream>,
    where_clause: Option<syn::WhereClause>,
}

impl IteratorNextBack {
    /// Initializes an empty `where`-clause if there is not one present already.
    pub(crate) fn make_where_clause(&mut self) -> &mut syn::WhereClause {
        self.where_clause.get_or_insert_with(|| syn::WhereClause {
            where_token: <syn::Token![where]>::default(),
            predicates: syn::punctuated::Punctuated::new(),
        })
    }
}

impl ToTokens for IteratorNextBack {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let iter_next = &self.next;

        tokens.extend(quote! {
            while self.start < self.end {
                let next = self.end.wrapping_sub(1);

                match next {
                    #(#iter_next,)*
                    _ => break,
                }

                self.end = usize::max(next, self.start);
            }
        });
    }
}

/// Construct `StorageEntry` implementation.
fn build_entry_impl(
    cx: &Ctxt<'_>,
    field_specs: &[FieldSpec<'_>],
) -> Result<(TokenStream, TokenStream), ()> {
    let ident = &cx.ast.ident;
    let vis = &cx.ast.vis;
    let lt = cx.lt;

    let entry_enum = cx.toks.entry_enum();
    let occupied_entry_t = cx.toks.occupied_entry_t();
    let option = cx.toks.option();
    let option_bucket_none = cx.toks.option_bucket_none();
    let option_bucket_option = cx.toks.option_bucket_option();
    let option_bucket_some = cx.toks.option_bucket_some();
    let storage_t = cx.toks.storage_t();
    let vacant_entry_t = cx.toks.vacant_entry_t();

    let mut init = Vec::new();
    let mut occupied_variant = Vec::new();
    let mut vacant_variant = Vec::new();

    let mut vacant_key = Vec::new();
    let mut vacant_insert = Vec::new();

    let mut occupied_key = Vec::new();
    let mut occupied_get = Vec::new();
    let mut occupied_get_mut = Vec::new();
    let mut occupied_into_mut = Vec::new();
    let mut occupied_insert = Vec::new();
    let mut occupied_remove = Vec::new();

    for FieldSpec {
        name, kind, var, ..
    } in field_specs
    {
        let pattern = quote!(#ident::#var);

        match kind {
            FieldKind::Simple => {
                init.push(quote!( #pattern => option_to_entry(&mut self.#name, key) ));
            }
            FieldKind::Complex {
                element, storage, ..
            } => {
                let as_storage = quote!(<#storage as #storage_t<#element, V>>);

                occupied_variant.push(quote!( #name(#as_storage::Occupied<#lt>) ));
                vacant_variant.push(quote!( #name(#as_storage::Vacant<#lt>) ));

                init.push(quote! {
                    #pattern(key) => match #storage_t::entry(&mut self.#name, key) {
                        #entry_enum::Occupied(entry) => #entry_enum::Occupied(OccupiedEntry::#name(entry)),
                        #entry_enum::Vacant(entry) => #entry_enum::Vacant(VacantEntry::#name(entry)),
                    }
                });

                let as_vacant_entry =
                    quote!(<#as_storage::Vacant<#lt> as #vacant_entry_t<#lt, #element, V>>);

                vacant_key.push(
                    quote!( VacantEntry::#name(entry) => #pattern(#as_vacant_entry::key(entry)) ),
                );
                vacant_insert.push(
                    quote!( VacantEntry::#name(entry) => #as_vacant_entry::insert(entry, value) ),
                );

                let as_occupied_entry =
                    quote!(<#as_storage::Occupied<#lt> as #occupied_entry_t<#lt, #element, V>>);

                occupied_key.push(quote!( OccupiedEntry::#name(entry) => #pattern(#as_occupied_entry::key(entry)) ));
                occupied_get
                    .push(quote!( OccupiedEntry::#name(entry) => #as_occupied_entry::get(entry) ));
                occupied_get_mut.push(
                    quote!( OccupiedEntry::#name(entry) => #as_occupied_entry::get_mut(entry) ),
                );
                occupied_into_mut.push(
                    quote!( OccupiedEntry::#name(entry) => #as_occupied_entry::into_mut(entry) ),
                );
                occupied_insert.push(quote!( OccupiedEntry::#name(entry) => #as_occupied_entry::insert(entry, value) ));
                occupied_remove.push(
                    quote!( OccupiedEntry::#name(entry) => #as_occupied_entry::remove(entry) ),
                );
            }
        }
    }

    let entry_impl = quote! {
        #vis struct SimpleVacantEntry<#lt, V> {
            key: #ident,
            inner: #option_bucket_none<#lt, V>,
        }

        impl<#lt, V> SimpleVacantEntry<#lt, V> {
            #[inline]
            fn insert(self, value: V) -> &#lt mut V {
                #option_bucket_none::insert(self.inner, value)
            }
        }

        #vis struct SimpleOccupiedEntry<#lt, V> {
            key: #ident,
            inner: #option_bucket_some<#lt, V>,
        }

        impl<#lt, V> SimpleOccupiedEntry<#lt, V> {
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

        #vis enum VacantEntry<#lt, V> {
            Simple(SimpleVacantEntry<#lt, V>),
            #(#vacant_variant,)*
        }

        #vis enum OccupiedEntry<#lt, V> {
            Simple(SimpleOccupiedEntry<#lt, V>),
            #(#occupied_variant,)*
        }

        #[automatically_derived]
        impl<#lt, V> #vacant_entry_t<#lt, #ident, V> for VacantEntry<#lt, V> {
            #[inline]
            fn key(&self) -> #ident {
                match self {
                    VacantEntry::Simple(entry) => entry.key,
                    #(#vacant_key,)*
                }
            }

            #[inline]
            fn insert(self, value: V) -> &#lt mut V {
                match self {
                    VacantEntry::Simple(entry) => entry.insert(value),
                    #(#vacant_insert,)*
                }
            }
        }

        #[automatically_derived]
        impl<#lt, V> #occupied_entry_t<#lt, #ident, V> for OccupiedEntry<#lt, V> {
            #[inline]
            fn key(&self) -> #ident {
                match self {
                    OccupiedEntry::Simple(entry) => entry.key,
                    #(#occupied_key,)*
                }
            }

            #[inline]
            fn get(&self) -> &V {
                match self {
                    OccupiedEntry::Simple(entry) => entry.get(),
                    #(#occupied_get,)*
                }
            }

            #[inline]
            fn get_mut(&mut self) -> &mut V {
                match self {
                    OccupiedEntry::Simple(entry) => entry.get_mut(),
                    #(#occupied_get_mut,)*
                }
            }

            #[inline]
            fn into_mut(self) -> &#lt mut V {
                match self {
                    OccupiedEntry::Simple(entry) => entry.into_mut(),
                    #(#occupied_into_mut,)*
                }
            }

            #[inline]
            fn insert(&mut self, value: V) -> V {
                match self {
                    OccupiedEntry::Simple(entry) => entry.insert(value),
                    #(#occupied_insert,)*
                }
            }

            #[inline]
            fn remove(self) -> V {
                match self {
                    OccupiedEntry::Simple(entry) => entry.remove(),
                    #(#occupied_remove,)*
                }
            }
        }

        #[inline]
        fn option_to_entry<V>(opt: &mut #option<V>, key: #ident) -> #entry_enum<'_, Storage<V>, #ident, V> {
            match #option_bucket_option::new(opt) {
                #option_bucket_option::Some(inner) => #entry_enum::Occupied(OccupiedEntry::Simple(SimpleOccupiedEntry { key, inner })),
                #option_bucket_option::None(inner) => #entry_enum::Vacant(VacantEntry::Simple(SimpleVacantEntry { key, inner })),
            }
        }
    };

    let entry_storage_impl = quote! {
        type Occupied<#lt> = OccupiedEntry<#lt, V> where V: #lt;
        type Vacant<#lt> = VacantEntry<#lt, V> where V: #lt;

        #[inline]
        fn entry(&mut self, key: #ident) -> #entry_enum<'_, Self, #ident, V> {
            match key {
                #(#init,)*
            }
        }
    };

    Ok((entry_impl, entry_storage_impl))
}
