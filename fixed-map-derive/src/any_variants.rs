use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned};
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

    let mut copy_bounds = Vec::new();

    let mut field_specs = Vec::new();

    for (index, variant) in en.variants.iter().enumerate() {
        let var = &variant.ident;
        let field = Ident::new(&format!("f{}", index), Span::call_site());

        field_inits.push(quote!(#field: #default::default()));
        field_clones.push(quote!(#field: #clone::clone(&self.#field)));
        field_partial_eqs.push(quote! {
            if self.#field != other.#field {
                return false;
            }
        });

        let kind = match &variant.fields {
            Fields::Unit => {
                fields.push(quote!(#field: #option<V>));
                pattern.push(quote!(#ident::#var));
                clear.push(quote!(self.#field = #option::None));

                get.push(quote!(#option::as_ref(&self.#field)));
                get_mut.push(quote!(#option::as_mut(&mut self.#field)));
                insert.push(quote!(#mem::replace(&mut self.#field, #option::Some(value))));
                remove.push(quote!(#mem::replace(&mut self.#field, #option::None)));

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
                let storage = quote!(<#element as #key_trait<#element, V>>::Storage);
                let as_storage = quote!(<#storage as #storage_trait<#element, V>>);

                fields.push(quote!(#field: #storage));
                pattern.push(quote!(#ident::#var(v)));
                clear.push(quote!(#as_storage::clear(&mut self.#field)));

                get.push(quote!(#as_storage::get(&self.#field, v)));
                get_mut.push(quote!(#as_storage::get_mut(&mut self.#field, v)));
                insert.push(quote!(#as_storage::insert(&mut self.#field, v, value)));
                remove.push(quote!(#as_storage::remove(&mut self.#field, v)));

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
            name: field,
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
                type Keys<'this> = Keys<'this, V> where Self: 'this;
                type Values<'this> = Values<'this, V> where Self: 'this;
                type IterMut<'this> = IterMut<'this, V> where Self: 'this;
                type ValuesMut<'this> = ValuesMut<'this, V> where Self: 'this;
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

            impl<V> #key_trait<#ident, V> for #ident {
                type Storage = Storage<V>;
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

#[derive(Default)]
struct IteratorSpec {
    next: Vec<TokenStream>,
    where_clause: Option<syn::WhereClause>,
}

impl IteratorSpec {
    /// Initializes an empty `where`-clause if there is not one present already.
    pub fn make_where_clause(&mut self) -> &mut syn::WhereClause {
        self.where_clause.get_or_insert_with(|| syn::WhereClause {
            where_token: <syn::Token![where]>::default(),
            predicates: syn::punctuated::Punctuated::new(),
        })
    }
}

/// Build iterator next.
fn build_iter_next(
    cx: &Ctxt,
    field_specs: &[FieldSpec<'_>],
    assoc_type: &syn::Ident,
    lt: Option<&syn::Lifetime>,
) -> Result<(IteratorSpec, IteratorSpec), ()> {
    let option = &cx.toks.option;
    let iterator_t = &cx.toks.iterator_t;
    let double_ended_iterator_t = &cx.toks.double_ended_iterator_t;
    let ident = &cx.ast.ident;

    let mut iterator = IteratorSpec::default();
    let mut double_ended = IteratorSpec::default();

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
                iterator.next.push(quote! {
                    #index => {
                        if let #option::Some(value) = self.#name.take() {
                            return #option::Some((#ident::#var, value));
                        }
                    }
                });

                double_ended.next.push(quote! {
                    #index => {
                        if let #option::Some(value) = self.#name.take() {
                            return #option::Some((#ident::#var, value));
                        }
                    }
                });
            }
            FieldKind::Complex { as_storage, .. } => {
                iterator.next.push(quote! {
                    #index => {
                        if let #option::Some((key, value)) = #iterator_t::next(&mut self.#name) {
                            return #option::Some((#ident::#var(key), value));
                        }
                    }
                });

                double_ended.next.push(quote! {
                    #index => {
                        if let #option::Some((key, value)) = #double_ended_iterator_t::next_back(&mut self.#name) {
                            return #option::Some((#ident::#var(key), value));
                        }
                    }
                });

                // NB: The `Item = ..` component of the bound is technically
                // superflous but currently necessary to satisfy rustc.
                let where_clause = double_ended.make_where_clause();

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

    Ok((iterator, double_ended))
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
    let (step_forward, step_backward, next_backward) = build_steps();

    let mut iter_clone = Vec::new();
    let mut field_decls = Vec::new();
    let mut init = Vec::new();

    let (iterator, double_ended) = build_iter_next(cx, field_specs, &type_name, Some(cx.lt))?;

    let IteratorSpec {
        next: iter_next, ..
    } = iterator;

    let IteratorSpec {
        next: double_ended_next,
        where_clause: double_ended_where,
    } = double_ended;

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

    let iter_impl = quote! {
        #vis struct #type_name<#lt, V> {
            start: usize,
            end: usize,
            #(#field_decls,)*
        }

        impl<#lt, V> #clone for #type_name<#lt, V> {
            #[inline]
            fn clone(&self) -> Self {
                Self {
                    start: self.start,
                    end: self.end,
                    #(#iter_clone,)*
                }
            }
        }

        impl<#lt, V> #iterator_t for #type_name<#lt, V> {
            type Item = (#ident, &#lt V);

            #[inline]
            fn next(&mut self) -> #option<Self::Item> {
                while self.start < self.end {
                    match self.start {
                        #(#iter_next,)*
                        _ => break,
                    }

                    #step_forward
                }

                #option::None
            }
        }

        impl<#lt, V> #double_ended_iterator_t for #type_name<#lt, V> #double_ended_where {
            #[inline]
            fn next_back(&mut self) -> #option<Self::Item> {
                while self.start < self.end {
                    let next = #next_backward;

                    match next {
                        #(#double_ended_next,)*
                        _ => break,
                    }

                    #step_backward
                }

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
    let (step_forward, step_backward, next_backward) = build_steps();

    let mut iter_clone = Vec::new();
    let mut field_decls = Vec::new();
    let mut init = Vec::new();

    let mut iter_next = Vec::new();

    let mut double_ended = IteratorSpec::default();

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

                iter_next.push(quote! {
                    #index => {
                        if #mem::take(&mut self.#name) {
                            return #option::Some(#ident::#var);
                        }

                        self.start = self.start.wrapping_add(1);
                    }
                });

                double_ended.next.push(quote! {
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

                iter_next.push(quote! {
                    #index => {
                        if let #option::Some(key) = #iterator_t::next(&mut self.#name) {
                            return #option::Some(#ident::#var(key));
                        }
                    }
                });

                double_ended.next.push(quote! {
                    #index => {
                        if let #option::Some(key) = #double_ended_iterator_t::next_back(&mut self.#name) {
                            return #option::Some(#ident::#var(key));
                        }
                    }
                });

                let where_clause = double_ended.make_where_clause();

                let assoc_type = quote!(#as_storage::#type_name<#lt>);

                where_clause.predicates.push(cx.fallible(|| syn::parse2(quote_spanned! {
                    *span => #assoc_type: #double_ended_iterator_t<Item = <#assoc_type as #iterator_t>::Item>
                }))?);
            }
        }
    }

    let IteratorSpec {
        next: double_ended_next_back,
        where_clause: double_ended_where_clause,
    } = double_ended;

    let iter_impl = quote! {
        #vis struct #type_name<#lt, V> where V: #lt {
            start: usize,
            end: usize,
            #(#field_decls,)*
        }

        impl<V> #clone for #type_name<'_, V> {
            #[inline]
            fn clone(&self) -> Self {
                Self {
                    start: self.start,
                    end: self.end,
                    #(#iter_clone,)*
                }
            }
        }

        impl<V> #iterator_t for #type_name<'_, V> {
            type Item = #ident;

            #[inline]
            fn next(&mut self) -> #option<Self::Item> {
                while self.start < self.end {
                    match self.start {
                        #(#iter_next,)*
                        _ => break,
                    }

                    #step_forward
                }

                #option::None
            }
        }

        impl<#lt, V> #double_ended_iterator_t for #type_name<#lt, V> #double_ended_where_clause {
            #[inline]
            fn next_back(&mut self) -> #option<Self::Item> {
                while self.start < self.end {
                    let next = #next_backward;

                    match next {
                        #(#double_ended_next_back,)*
                        _ => break,
                    }

                    #step_backward
                }

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
    let (step_forward, step_backward, next_backward) = build_steps();

    let mut iter_clone = Vec::new();
    let mut field_decls = Vec::new();
    let mut init = Vec::new();

    let mut iter_next = Vec::new();

    let mut double_ended = IteratorSpec::default();

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

                iter_next.push(quote! {
                    #index => {
                        if let #option::Some(value) = self.#name.take() {
                            return #option::Some(value);
                        }
                    }
                });

                double_ended.next.push(quote! {
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

                iter_next.push(quote! {
                    #index => {
                        if let #option::Some(value) = #iterator_t::next(&mut self.#name) {
                            return #option::Some(value);
                        }
                    }
                });

                double_ended.next.push(quote! {
                    #index => {
                        if let #option::Some(value) = #double_ended_iterator_t::next_back(&mut self.#name) {
                            return #option::Some(value);
                        }
                    }
                });

                let where_clause = double_ended.make_where_clause();

                let assoc_type = quote!(#as_storage::#type_name<#lt>);

                where_clause.predicates.push(cx.fallible(|| syn::parse2(quote_spanned! {
                    *span => #assoc_type: #double_ended_iterator_t<Item = <#assoc_type as #iterator_t>::Item>
                }))?);
            }
        }
    }

    let IteratorSpec {
        next: double_ended_next_back,
        where_clause: double_ended_where_clause,
    } = double_ended;

    let iter_impl = quote! {
        #vis struct #type_name<#lt, V> {
            start: usize,
            end: usize,
            #(#field_decls,)*
        }

        impl<#lt, V> #clone for #type_name<#lt, V> {
            #[inline]
            fn clone(&self) -> Self {
                Self {
                    start: self.start,
                    end: self.end,
                    #(#iter_clone,)*
                }
            }
        }

        impl<#lt, V> #iterator_t for #type_name<#lt, V> {
            type Item = &#lt V;

            #[inline]
            fn next(&mut self) -> #option<Self::Item> {
                while self.start < self.end {
                    match self.start {
                        #(#iter_next,)*
                        _ => break,
                    }

                    #step_forward
                }

                #option::None
            }
        }

        impl<#lt, V> #double_ended_iterator_t for #type_name<#lt, V> #double_ended_where_clause {
            #[inline]
            fn next_back(&mut self) -> #option<Self::Item> {
                while self.start < self.end {
                    let next = #next_backward;

                    match next {
                        #(#double_ended_next_back,)*
                        _ => break,
                    }

                    #step_backward
                }

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
    let (step_forward, step_backward, next_backward) = build_steps();

    let mut field_decls = Vec::new();
    let mut init = Vec::new();

    let (iterator, double_ended) = build_iter_next(cx, field_specs, &type_name, Some(cx.lt))?;

    let IteratorSpec {
        next: iter_next, ..
    } = iterator;

    let IteratorSpec {
        next: double_ended_next,
        where_clause: double_ended_where,
    } = double_ended;

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

    let iter_impl = quote! {
        #vis struct #type_name<#lt, V> {
            start: usize,
            end: usize,
            #(#field_decls,)*
        }

        impl<#lt, V> #iterator_t for #type_name<#lt, V> {
            type Item = (#ident, &#lt mut V);

            #[inline]
            fn next(&mut self) -> #option<Self::Item> {
                while self.start < self.end {
                    match self.start {
                        #(#iter_next,)*
                        _ => break,
                    }

                    #step_forward
                }

                #option::None
            }
        }

        impl<#lt, V> #double_ended_iterator_t for #type_name<#lt, V> #double_ended_where {
            #[inline]
            fn next_back(&mut self) -> #option<Self::Item> {
                while self.start < self.end {
                    let next = #next_backward;

                    match next {
                        #(#double_ended_next,)*
                        _ => break,
                    }

                    #step_backward
                }

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

    let mut iter_clone = Vec::new();
    let mut field_decls = Vec::new();
    let mut init = Vec::new();

    let mut iter_next = Vec::new();

    for FieldSpec {
        index, name, kind, ..
    } in field_specs
    {
        iter_clone.push(quote!(#name: #clone::clone(&self.#name)));

        match kind {
            FieldKind::Simple => {
                field_decls.push(quote!(#name: #option<&#lt mut V>));
                init.push(quote!(#name: #option::as_mut(&mut self.#name)));

                iter_next.push(quote! {
                    #index => {
                        if let #option::Some(v) = self.#name.take() {
                            return #option::Some(v);
                        }

                        self.start += 1;
                    }
                });
            }
            FieldKind::Complex { as_storage, .. } => {
                field_decls.push(quote!(#name: #as_storage::ValuesMut<#lt>));
                init.push(quote!(#name: #as_storage::values_mut(&mut self.#name)));

                iter_next.push(quote! {
                    #index => {
                        if let #option::Some(value) = #iterator_t::next(&mut self.#name) {
                            return #option::Some(value);
                        }

                        self.start += 1;
                    }
                });
            }
        }
    }

    let iter_impl = quote! {
        #vis struct #type_name<#lt, V> {
            start: usize,
            end: usize,
            #(#field_decls,)*
        }

        impl<#lt, V> #iterator_t for #type_name<#lt, V> {
            type Item = &#lt mut V;

            #[inline]
            fn next(&mut self) -> #option<Self::Item> {
                loop {
                    match self.start {
                        #(#iter_next,)*
                        _ => return #option::None,
                    }
                }
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
    let (step_forward, step_backward, next_backward) = build_steps();

    let mut field_decls = Vec::new();
    let mut init = Vec::new();
    let mut field_clone = Vec::new();
    let mut clone_bounds = Vec::new();

    let (iterator, double_ended) = build_iter_next(cx, field_specs, &type_name, None)?;

    let IteratorSpec {
        next: iter_next, ..
    } = iterator;

    let IteratorSpec {
        next: double_ended_next,
        where_clause: double_ended_where,
    } = double_ended;

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

    let iter_impl = quote! {
        #vis struct #type_name<V> {
            start: usize,
            end: usize,
            #(#field_decls,)*
        }

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

        impl<V> #iterator_t for #type_name<V> {
            type Item = (#ident, V);

            #[inline]
            fn next(&mut self) -> Option<Self::Item> {
                while self.start < self.end {
                    match self.start {
                        #(#iter_next,)*
                        _ => break,
                    }

                    #step_forward
                }

                #option::None
            }
        }

        impl<V> #double_ended_iterator_t for #type_name<V> #double_ended_where {
            #[inline]
            fn next_back(&mut self) -> #option<Self::Item> {
                while self.start < self.end {
                    let next = #next_backward;

                    match next {
                        #(#double_ended_next,)*
                        _ => break,
                    }

                    #step_backward
                }

                #option::None
            }
        }
    };

    Ok((iter_impl, init))
}

/// Build a forward step operation.
fn build_steps() -> (TokenStream, TokenStream, TokenStream) {
    let step_forward = quote!(self.start = usize::min(self.start.wrapping_add(1), self.end));
    let step_backward = quote!(self.end = usize::max(next, self.start));
    let next_backward = quote!(self.end.wrapping_sub(1));
    (step_forward, step_backward, next_backward)
}
