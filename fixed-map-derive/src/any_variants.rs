use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned, ToTokens};
use syn::spanned::Spanned;
use syn::{DataEnum, Fields, Ident};

use crate::context::{Ctxt, FieldKind, FieldSpec};

pub(crate) fn implement(cx: &Ctxt, en: &DataEnum) -> Result<TokenStream, ()> {
    let vis = &cx.ast.vis;
    let ident = &cx.ast.ident;

    let clone = &cx.toks.clone;
    let copy = &cx.toks.copy;
    let default = &cx.toks.default;
    let eq = &cx.toks.eq;
    let key_trait = &cx.toks.key_trait;
    let mem = &cx.toks.mem;
    let option = &cx.toks.option;
    let partial_eq = &cx.toks.partial_eq;
    let storage_trait = &cx.toks.storage_trait;

    let const_wrapper = Ident::new(
        &format!("__IMPL_KEY_FOR_{}", cx.ast.ident),
        Span::call_site(),
    );

    let mut len = Vec::new();
    let mut is_empty = Vec::new();
    let mut pattern = Vec::new();
    let mut fields = Vec::new();
    let mut field_inits = Vec::new();
    let mut field_clones = Vec::new();
    let mut field_partial_eqs = Vec::new();
    let mut contains_key = Vec::new();
    let mut get = Vec::new();
    let mut get_mut = Vec::new();
    let mut insert = Vec::new();
    let mut remove = Vec::new();
    let mut retain = Vec::new();
    let mut clear = Vec::new();
    let mut copy_bounds = Vec::new();
    let mut field_specs = Vec::new();

    for (index, variant) in en.variants.iter().enumerate() {
        let var = &variant.ident;
        let name = Ident::new(&format!("f{}", index), Span::call_site());

        field_inits.push(quote!(#name: #default::default()));
        field_clones.push(quote!(#name: #clone::clone(&self.#name)));
        field_partial_eqs.push(quote! {
            if self.#name != other.#name {
                return false;
            }
        });

        let kind = match &variant.fields {
            Fields::Unit => {
                len.push(quote!(usize::from(#option::is_some(&self.#name))));
                is_empty.push(quote!(#option::is_none(&self.#name)));
                fields.push(quote!(#name: #option<V>));
                pattern.push(quote!(#ident::#var));
                clear.push(quote!(self.#name = #option::None));
                contains_key.push(quote!(#option::is_some(&self.#name)));
                get.push(quote!(#option::as_ref(&self.#name)));
                get_mut.push(quote!(#option::as_mut(&mut self.#name)));
                insert.push(quote!(#mem::replace(&mut self.#name, #option::Some(value))));
                remove.push(quote!(#mem::replace(&mut self.#name, #option::None)));
                retain.push(quote! {
                    if let Some(val) = #option::as_mut(&mut self.#name) {
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
                let storage = quote!(<#element as #key_trait>::Storage::<V>);
                let as_storage = quote!(<#storage as #storage_trait<#element, V>>);

                len.push(quote!(#as_storage::len(&self.#name)));
                is_empty.push(quote!(#as_storage::is_empty(&self.#name)));

                fields.push(quote!(#name: #storage));
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

                copy_bounds.push(quote!(#storage: #copy));

                FieldKind::Complex {
                    as_storage,
                    storage,
                }
            }
            _ => {
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
        iter_clone.push(quote!(#name: #clone::clone(&self.#name)));
    }

    let pattern = &pattern;

    let (iter_impl, iter_init) = build_iter_impl(cx, "Iter", &field_specs)?;
    let (keys_impl, keys_iter_init) = build_keys_impl(cx, "Keys", &field_specs)?;
    let (values_impl, values_iter_init) = build_values_impl(cx, "Values", &field_specs)?;
    let (iter_mut_impl, iter_mut_init) = build_iter_mut_impl(cx, "IterMut", &field_specs)?;
    let (values_mut_impl, values_mut_init) = build_values_mut_impl(cx, "ValuesMut", &field_specs)?;
    let (into_iter_impl, into_iter_init) = build_into_iter_impl(cx, "IntoIter", &field_specs)?;

    let end = field_specs.len();

    Ok(quote! {
        const #const_wrapper: () = {
            #vis struct Storage<V> {
                #(#fields,)*
            }

            #[automatically_derived]
            impl<V> #clone for Storage<V> where V: #clone {
                #[inline]
                fn clone(&self) -> Storage<V> {
                    Storage {
                        #(#field_clones,)*
                    }
                }
            }

            #[automatically_derived]
            impl<V> #copy for Storage<V> where V: #copy, #(#copy_bounds,)* {}

            #[automatically_derived]
            impl<V> #partial_eq for Storage<V> where V: #partial_eq {
                #[inline]
                fn eq(&self, other: &Storage<V>) -> bool {
                    #(#field_partial_eqs;)*
                    true
                }
            }

            #[automatically_derived]
            impl<V> #eq for Storage<V> where V: #eq {}

            #[automatically_derived]
            impl<V> #default for Storage<V> {
                #[inline]
                fn default() -> Self {
                    Self {
                        #(#field_inits,)*
                    }
                }
            }

            #[automatically_derived]
            impl<V> #storage_trait<#ident, V> for Storage<V> {
                type Iter<'this> = Iter<'this, V> where Self: 'this;
                type Keys<'this> = Keys<'this, V> where Self: 'this;
                type Values<'this> = Values<'this, V> where Self: 'this;
                type IterMut<'this> = IterMut<'this, V> where Self: 'this;
                type ValuesMut<'this> = ValuesMut<'this, V> where Self: 'this;
                type IntoIter = IntoIter<V>;

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
            }

            #[automatically_derived]
            impl #key_trait for #ident {
                type Storage<V> = Storage<V>;
            }

            #iter_impl
            #keys_impl
            #values_impl
            #iter_mut_impl
            #values_mut_impl
            #into_iter_impl
        };
    })
}

/// Build iterator next.
fn build_iter_next(
    cx: &Ctxt,
    step_forward: &mut IteratorNext,
    step_backward: &mut IteratorNextBack,
    field_specs: &[FieldSpec<'_>],
    assoc_type: &syn::Ident,
    lt: Option<&syn::Lifetime>,
) -> Result<(), ()> {
    let option = &cx.toks.option;
    let iterator_t = &cx.toks.iterator_t;
    let double_ended_iterator_t = &cx.toks.double_ended_iterator_t;
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
                        if let #option::Some(value) = self.#name.take() {
                            return #option::Some((#ident::#var, value));
                        }
                    }
                });

                step_backward.next.push(quote! {
                    #index => {
                        if let #option::Some(value) = self.#name.take() {
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
    cx: &Ctxt,
    id: &str,
    field_specs: &[FieldSpec<'_>],
) -> Result<(TokenStream, Vec<TokenStream>), ()> {
    let type_name = syn::Ident::new(id, Span::call_site());

    let lt = cx.lt;
    let option = &cx.toks.option;
    let iterator_t = &cx.toks.iterator_t;
    let double_ended_iterator_t = &cx.toks.double_ended_iterator_t;
    let clone = &cx.toks.clone;
    let ident = &cx.ast.ident;
    let vis = &cx.ast.vis;

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
        iter_clone.push(quote!(#name: #clone::clone(&self.#name)));

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
        impl<#lt, V> #clone for #type_name<#lt, V> where V: #lt {
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

/// Construct an keys iterator_t implementation.
fn build_keys_impl(
    cx: &Ctxt,
    id: &str,
    field_specs: &[FieldSpec<'_>],
) -> Result<(TokenStream, Vec<TokenStream>), ()> {
    let type_name = syn::Ident::new(id, Span::call_site());

    let lt = cx.lt;
    let option = &cx.toks.option;
    let iterator_t = &cx.toks.iterator_t;
    let double_ended_iterator_t = &cx.toks.double_ended_iterator_t;
    let clone = &cx.toks.clone;
    let bool_type = &cx.toks.bool_type;
    let mem = &cx.toks.mem;
    let ident = &cx.ast.ident;
    let vis = &cx.ast.vis;

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
        iter_clone.push(quote!(#name: #clone::clone(&self.#name)));

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
        impl<#lt, V> #clone for #type_name<#lt, V> where V: #lt {
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

/// Construct a values iterator_t implementation.
fn build_values_impl(
    cx: &Ctxt,
    id: &str,
    field_specs: &[FieldSpec<'_>],
) -> Result<(TokenStream, Vec<TokenStream>), ()> {
    let type_name = syn::Ident::new(id, Span::call_site());

    let lt = cx.lt;
    let option = &cx.toks.option;
    let iterator_t = &cx.toks.iterator_t;
    let double_ended_iterator_t = &cx.toks.double_ended_iterator_t;
    let clone = &cx.toks.clone;
    let vis = &cx.ast.vis;

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
        iter_clone.push(quote!(#name: #clone::clone(&self.#name)));

        match kind {
            FieldKind::Simple => {
                field_decls.push(quote!(#name: #option<&#lt V>));
                init.push(quote!(#name: #option::as_ref(&self.#name)));

                step_forward.next.push(quote! {
                    #index => {
                        if let #option::Some(value) = self.#name.take() {
                            return #option::Some(value);
                        }
                    }
                });

                step_backward.next.push(quote! {
                    #index => {
                        if let #option::Some(value) = self.#name.take() {
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
        impl<#lt, V> #clone for #type_name<#lt, V> where V: #lt {
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

    let lt = cx.lt;
    let ident = &cx.ast.ident;
    let vis = &cx.ast.vis;
    let iterator_t = &cx.toks.iterator_t;
    let double_ended_iterator_t = &cx.toks.double_ended_iterator_t;
    let option = &cx.toks.option;

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

/// Construct a values mutable iterator_t implementation.
fn build_values_mut_impl(
    cx: &Ctxt,
    id: &str,
    field_specs: &[FieldSpec<'_>],
) -> Result<(TokenStream, Vec<TokenStream>), ()> {
    let type_name = syn::Ident::new(id, Span::call_site());

    let lt = cx.lt;
    let option = &cx.toks.option;
    let iterator_t = &cx.toks.iterator_t;
    let clone = &cx.toks.clone;
    let vis = &cx.ast.vis;
    let double_ended_iterator_t = &cx.toks.double_ended_iterator_t;

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
        iter_clone.push(quote!(#name: #clone::clone(&self.#name)));

        match kind {
            FieldKind::Simple => {
                field_decls.push(quote!(#name: #option<&#lt mut V>));
                init.push(quote!(#name: #option::as_mut(&mut self.#name)));

                step_forward.next.push(quote! {
                    #index => {
                        if let #option::Some(v) = self.#name.take() {
                            return #option::Some(v);
                        }
                    }
                });

                step_backward.next.push(quote! {
                    #index => {
                        if let #option::Some(v) = self.#name.take() {
                            return #option::Some(v);
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

/// Construct IntoIter implementation.
fn build_into_iter_impl(
    cx: &Ctxt<'_>,
    id: &str,
    field_specs: &[FieldSpec<'_>],
) -> Result<(TokenStream, Vec<TokenStream>), ()> {
    let type_name = syn::Ident::new(id, Span::call_site());

    let ident = &cx.ast.ident;
    let vis = &cx.ast.vis;
    let option = &cx.toks.option;
    let clone = &cx.toks.clone;
    let iterator_t = &cx.toks.iterator_t;
    let double_ended_iterator_t = &cx.toks.double_ended_iterator_t;

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
        field_clone.push(quote!(#name: #clone::clone(&self.#name)));

        match kind {
            FieldKind::Simple => {
                field_decls.push(quote!(#name: #option<V>));
                init.push(quote!(#name: self.#name));
            }
            FieldKind::Complex {
                as_storage,
                storage,
            } => {
                field_decls.push(quote!(#name: #as_storage::IntoIter));
                init.push(quote!(#name: #storage::into_iter(self.#name)));
                clone_bounds.push(quote!(#as_storage::IntoIter: #clone));
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
        impl<V> #clone for #type_name<V> where V: Clone, #(#clone_bounds,)* {
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
    pub fn make_where_clause(&mut self) -> &mut syn::WhereClause {
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
