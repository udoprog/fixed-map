use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote, quote_spanned, ToTokens};
use syn::spanned::Spanned;
use syn::{DataEnum, Ident, Pat};

const MAP_STORAGE: &str = "__MapStorage";
const SET_STORAGE: &str = "__SetStorage";

use crate::context::Ctxt;

///
pub(crate) fn implement(cx: &Ctxt<'_>, en: &DataEnum) -> Result<TokenStream, ()> {
    let ident = &cx.ast.ident;

    let key_t = cx.toks.key_t();
    let storage_t = cx.toks.storage_t();
    let set_storage_t = cx.toks.set_storage_t();

    let const_wrapper = Ident::new(
        &format!("__IMPL_KEY_FOR_{}", cx.ast.ident),
        Span::call_site(),
    );

    let mut fields = Fields::default();

    for (index, variant) in en.variants.iter().enumerate() {
        let var = &variant.ident;
        let name = format_ident!("_{}", index);

        let kind = match &variant.fields {
            syn::Fields::Unit => {
                fields
                    .patterns
                    .push(cx.fallible(|| syn::parse2(quote!(#ident::#var)))?);
                Kind::Simple
            }
            syn::Fields::Unnamed(unnamed) => {
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
                let set_storage = quote!(<#element as #key_t>::SetStorage);
                let as_set_storage = quote!(<#set_storage as #set_storage_t<#element>>);

                fields
                    .patterns
                    .push(cx.fallible(|| syn::parse2(quote!(#ident::#var(v))))?);

                Kind::Complex(Complex {
                    element,
                    storage,
                    as_storage,
                    set_storage,
                    as_set_storage,
                })
            }
            syn::Fields::Named(_) => {
                cx.error(variant.fields.span(), "named fields are not supported");
                continue;
            }
        };

        fields.fields.push(Field {
            span: variant.span(),
            index,
            name,
            var,
            kind,
        });
    }

    let (map_storage_type_name, map_storage_impl) = impl_map_storage(cx, &fields)?;
    let (set_storage_type_name, set_storage_impl) = impl_set_storage(cx, &fields)?;

    Ok(quote! {
        const #const_wrapper: () = {
            #map_storage_impl
            #set_storage_impl

            #[automatically_derived]
            impl #key_t for #ident {
                type Storage<V> = #map_storage_type_name<V>;
                type SetStorage = #set_storage_type_name;
            }
        };
    })
}

/// Implement `Storage` implementation.
fn impl_map_storage(cx: &Ctxt<'_>, fields: &Fields<'_>) -> Result<(Ident, TokenStream), ()> {
    let vis = &cx.ast.vis;
    let ident = &cx.ast.ident;

    let mem = cx.toks.mem();
    let option = cx.toks.option();
    let storage_t = cx.toks.storage_t();

    let type_name = format_ident!("{MAP_STORAGE}");

    let mut output = Output::default();

    map_storage_iter(cx, "Iter", fields, &mut output)?;
    map_storage_keys(cx, "Keys", fields, &mut output)?;
    map_storage_values(cx, "Values", fields, &mut output)?;
    map_storage_iter_mut(cx, "IterMut", fields, &mut output)?;
    map_storage_values_mut(cx, "ValuesMut", fields, &mut output)?;
    map_storage_into_iter(cx, "IntoIter", fields, &mut output)?;
    map_storage_entry(cx, fields, &type_name, &mut output)?;

    {
        let partial_eq_t = cx.toks.partial_eq_t();
        let eq_t = cx.toks.eq_t();
        let names = fields.names();

        output.impls.extend(quote! {
            #[automatically_derived]
            impl<V> #partial_eq_t for #type_name<V> where V: #partial_eq_t {
                #[inline]
                fn eq(&self, other: &Self) -> bool {
                    #(if #partial_eq_t::ne(&self.#names, &other.#names) {
                        return false;
                    })*

                    true
                }
            }

            #[automatically_derived]
            impl<V> #eq_t for #type_name<V> where V: #eq_t {}
        });
    }

    {
        let clone_t = cx.toks.clone_t();
        let copy_t = cx.toks.copy_t();
        let bounds = fields.complex().map(|Complex { storage, .. }| storage);
        let names = fields.names();

        output.impls.extend(quote! {
            #[automatically_derived]
            impl<V> #clone_t for #type_name<V> where V: #clone_t {
                #[inline]
                fn clone(&self) -> Self {
                    Self {
                        #(#names: #clone_t::clone(&self.#names),)*
                    }
                }
            }

            #[automatically_derived]
            impl<V> #copy_t for #type_name<V> where V: #copy_t, #(#bounds: #copy_t,)* {}
        });
    }

    {
        let inits = fields.iter().map(|f| match &f.kind {
            Kind::Complex(Complex { as_storage, .. }) => quote!(#as_storage::empty()),
            Kind::Simple => quote!(#option::None),
        });

        let names = fields.names();

        output.items.extend(quote! {
            #[inline]
            fn empty() -> Self {
                Self {
                    #(#names: #inits,)*
                }
            }
        });
    }

    {
        let patterns = &fields.patterns;

        let insert = fields.iter().map(|Field { name, kind, .. }| match kind {
            Kind::Complex(Complex { as_storage, .. }) => {
                quote!(#as_storage::insert(&mut self.#name, v, value))
            }
            Kind::Simple => quote!(#mem::replace(&mut self.#name, #option::Some(value))),
        });

        output.items.extend(quote! {
            #[inline]
            fn insert(&mut self, key: #ident, value: V) -> #option<V> {
                match key {
                    #(#patterns => #insert,)*
                }
            }
        });
    }

    {
        let len = fields.iter().map(|Field { name, kind, .. }| match kind {
            Kind::Complex(Complex { as_storage, .. }) => {
                quote!(#as_storage::len(&self.#name))
            }
            Kind::Simple => quote!(usize::from(#option::is_some(&self.#name))),
        });

        output.items.extend(quote! {
            #[inline]
            fn len(&self) -> usize {
                0 #(+ #len)*
            }
        });
    }

    {
        let is_empty = fields.iter().map(|Field { name, kind, .. }| match kind {
            Kind::Complex(Complex { as_storage, .. }) => {
                quote!(#as_storage::is_empty(&self.#name))
            }
            Kind::Simple => quote!(#option::is_none(&self.#name)),
        });

        output.items.extend(quote! {
            #[inline]
            fn is_empty(&self) -> bool {
                true #(&& #is_empty)*
            }
        });
    }

    {
        let patterns = &fields.patterns;

        let contains_key = fields.iter().map(|Field { name, kind, .. }| match kind {
            Kind::Complex(Complex { as_storage, .. }) => {
                quote!(#as_storage::contains_key(&self.#name, v))
            }
            Kind::Simple => quote!(#option::is_some(&self.#name)),
        });

        output.items.extend(quote! {
            #[inline]
            fn contains_key(&self, value: #ident) -> bool {
                match value {
                    #(#patterns => #contains_key,)*
                }
            }
        });
    }

    {
        let patterns = &fields.patterns;

        let get = fields.iter().map(|Field { name, kind, .. }| match kind {
            Kind::Complex(Complex { as_storage, .. }) => {
                quote!(#as_storage::get(&self.#name, v))
            }
            Kind::Simple => quote!(#option::as_ref(&self.#name)),
        });

        output.items.extend(quote! {
            #[inline]
            fn get(&self, value: #ident) -> #option<&V> {
                match value {
                    #(#patterns => #get,)*
                }
            }
        });
    }

    {
        let patterns = &fields.patterns;

        let get_mut = fields.iter().map(|Field { name, kind, .. }| match kind {
            Kind::Complex(Complex { as_storage, .. }) => {
                quote!(#as_storage::get_mut(&mut self.#name, v))
            }
            Kind::Simple => quote!(#option::as_mut(&mut self.#name)),
        });

        output.items.extend(quote! {
            #[inline]
            fn get_mut(&mut self, value: #ident) -> #option<&mut V> {
                match value {
                    #(#patterns => #get_mut,)*
                }
            }
        });
    }

    {
        let remove = fields.iter().map(|Field { name, kind, .. }| match kind {
            Kind::Complex(Complex { as_storage, .. }) => {
                quote!(#as_storage::remove(&mut self.#name, v))
            }
            Kind::Simple => quote!(#mem::replace(&mut self.#name, #option::None)),
        });

        let patterns = &fields.patterns;

        output.items.extend(quote! {
            #[inline]
            fn remove(&mut self, value: #ident) -> #option<V> {
                match value {
                    #(#patterns => #remove,)*
                }
            }
        });
    }

    {
        let retain = fields.iter().map(
            |Field {
                 var, name, kind, ..
             }| match kind {
                Kind::Complex(Complex { as_storage, .. }) => quote! {
                    #as_storage::retain(&mut self.#name, |k, v| func(#ident::#var(k), v));
                },
                Kind::Simple => quote! {
                    if let #option::Some(val) = #option::as_mut(&mut self.#name) {
                        if !func(#ident::#var, val) {
                            self.#name = None;
                        }
                    }
                },
            },
        );

        output.items.extend(quote! {
            #[inline]
            fn retain<F>(&mut self, mut func: F)
            where
                F: FnMut(#ident, &mut V) -> bool
            {
                #(#retain;)*
            }
        });
    }

    {
        let clear = fields.iter().map(|Field { name, kind, .. }| match kind {
            Kind::Complex(Complex { as_storage, .. }) => quote! {
                #as_storage::clear(&mut self.#name)
            },
            Kind::Simple => quote! {
                self.#name = #option::None
            },
        });

        output.items.extend(quote! {
            #[inline]
            fn clear(&mut self) {
                #(#clear;)*
            }
        });
    }

    let field_decls = fields.iter().map(|Field { name, kind, .. }| match kind {
        Kind::Complex(Complex { storage, .. }) => quote!(#name: #storage),
        Kind::Simple => quote!(#name: #option<V>),
    });

    let Output { impls, items } = output;

    let map_storage_impl = quote! {
        #vis struct #type_name<V> {
            #(#field_decls,)*
        }

        #[automatically_derived]
        impl<V> #storage_t<#ident, V> for #type_name<V> {
            #items
        }

        #impls
    };

    Ok((type_name, map_storage_impl))
}

/// Implement `SetStorage` implementation.
fn impl_set_storage(cx: &Ctxt<'_>, fields: &Fields<'_>) -> Result<(Ident, TokenStream), ()> {
    let vis = &cx.ast.vis;
    let ident = &cx.ast.ident;

    let mem = cx.toks.mem();
    let set_storage_t = cx.toks.set_storage_t();

    let type_name = format_ident!("{SET_STORAGE}");

    let mut output = Output::default();

    set_storage_iter(cx, "Iter", fields, &mut output)?;
    set_storage_into_iter(cx, "IntoIter", fields, &mut output)?;

    {
        let partial_eq_t = cx.toks.partial_eq_t();
        let eq_t = cx.toks.eq_t();
        let names = fields.names();

        output.impls.extend(quote! {
            #[automatically_derived]
            impl #partial_eq_t for #type_name {
                #[inline]
                fn eq(&self, other: &Self) -> bool {
                    #(if #partial_eq_t::ne(&self.#names, &other.#names) {
                        return false;
                    })*

                    true
                }
            }

            #[automatically_derived]
            impl #eq_t for #type_name  {}
        });
    }

    {
        let clone_t = cx.toks.clone_t();
        let copy_t = cx.toks.copy_t();
        let bounds = fields
            .complex()
            .map(|Complex { set_storage, .. }| set_storage)
            .collect::<Vec<_>>();
        let names = fields.names();

        output.impls.extend(quote! {
            #[automatically_derived]
            impl #clone_t for #type_name where #(for<'trivial_bounds> #bounds: #clone_t,)* {
                #[inline]
                fn clone(&self) -> Self {
                    Self {
                        #(#names: #clone_t::clone(&self.#names),)*
                    }
                }
            }

            #[automatically_derived]
            impl #copy_t for #type_name where #(for<'trivial_bounds> #bounds: #copy_t,)* {}
        });
    }

    {
        let inits = fields.iter().map(|f| match &f.kind {
            Kind::Complex(Complex { as_set_storage, .. }) => quote!(#as_set_storage::empty()),
            Kind::Simple => quote!(false),
        });

        let names = fields.names();

        output.items.extend(quote! {
            #[inline]
            fn empty() -> Self {
                Self {
                    #(#names: #inits,)*
                }
            }
        });
    }

    {
        let patterns = &fields.patterns;

        let insert = fields.iter().map(|Field { name, kind, .. }| match kind {
            Kind::Complex(Complex { as_set_storage, .. }) => {
                quote!(#as_set_storage::insert(&mut self.#name, v))
            }
            Kind::Simple => quote!(!#mem::replace(&mut self.#name, true)),
        });

        output.items.extend(quote! {
            #[inline]
            fn insert(&mut self, key: #ident) -> bool {
                match key {
                    #(#patterns => #insert,)*
                }
            }
        });
    }

    {
        let len = fields.iter().map(|Field { name, kind, .. }| match kind {
            Kind::Complex(Complex { as_set_storage, .. }) => {
                quote!(#as_set_storage::len(&self.#name))
            }
            Kind::Simple => quote!(usize::from(self.#name)),
        });

        output.items.extend(quote! {
            #[inline]
            fn len(&self) -> usize {
                0 #(+ #len)*
            }
        });
    }

    {
        let is_empty = fields.iter().map(|Field { name, kind, .. }| match kind {
            Kind::Complex(Complex { as_set_storage, .. }) => {
                quote!(#as_set_storage::is_empty(&self.#name))
            }
            Kind::Simple => quote!(!self.#name),
        });

        output.items.extend(quote! {
            #[inline]
            fn is_empty(&self) -> bool {
                true #(&& #is_empty)*
            }
        });
    }

    {
        let patterns = &fields.patterns;

        let contains = fields.iter().map(|Field { name, kind, .. }| match kind {
            Kind::Complex(Complex { as_set_storage, .. }) => {
                quote!(#as_set_storage::contains(&self.#name, v))
            }
            Kind::Simple => quote!(self.#name),
        });

        output.items.extend(quote! {
            #[inline]
            fn contains(&self, value: #ident) -> bool {
                match value {
                    #(#patterns => #contains,)*
                }
            }
        });
    }

    {
        let remove = fields.iter().map(|Field { name, kind, .. }| match kind {
            Kind::Complex(Complex { as_set_storage, .. }) => {
                quote!(#as_set_storage::remove(&mut self.#name, v))
            }
            Kind::Simple => quote!(#mem::replace(&mut self.#name, false)),
        });

        let patterns = &fields.patterns;

        output.items.extend(quote! {
            #[inline]
            fn remove(&mut self, value: #ident) -> bool {
                match value {
                    #(#patterns => #remove,)*
                }
            }
        });
    }

    {
        let retain = fields.iter().map(
            |Field {
                 var, name, kind, ..
             }| match kind {
                Kind::Complex(Complex { as_set_storage, .. }) => quote! {
                    #as_set_storage::retain(&mut self.#name, |k| func(#ident::#var(k)));
                },
                Kind::Simple => quote! {
                    if self.#name {
                        self.#name = func(#ident::#var);
                    }
                },
            },
        );

        output.items.extend(quote! {
            #[inline]
            fn retain<F>(&mut self, mut func: F)
            where
                F: FnMut(#ident) -> bool
            {
                #(#retain;)*
            }
        });
    }

    {
        let clear = fields.iter().map(|Field { name, kind, .. }| match kind {
            Kind::Complex(Complex { as_set_storage, .. }) => quote! {
                #as_set_storage::clear(&mut self.#name)
            },
            Kind::Simple => quote! {
                self.#name = false
            },
        });

        output.items.extend(quote! {
            #[inline]
            fn clear(&mut self) {
                #(#clear;)*
            }
        });
    }

    let field_decls = fields.iter().map(|Field { name, kind, .. }| match kind {
        Kind::Complex(Complex { set_storage, .. }) => quote!(#name: #set_storage),
        Kind::Simple => quote!(#name: bool),
    });

    let Output { impls, items } = output;

    let map_storage_impl = quote! {
        #vis struct #type_name {
            #(#field_decls,)*
        }

        #[automatically_derived]
        impl #set_storage_t<#ident> for #type_name {
            #items
        }

        #impls
    };

    Ok((type_name, map_storage_impl))
}

/// Build iterator next.
fn build_iter_next(
    cx: &Ctxt<'_>,
    step_forward: &mut IteratorNext,
    step_backward: &mut IteratorNextBack,
    fields: &Fields<'_>,
    assoc_type: &Ident,
    lt: Option<&syn::Lifetime>,
) -> Result<(), ()> {
    let option = cx.toks.option();
    let iterator_t = cx.toks.iterator_t();
    let double_ended_iterator_t = cx.toks.double_ended_iterator_t();
    let ident = &cx.ast.ident;

    for Field {
        span,
        index,
        name,
        var,
        kind,
        ..
    } in fields
    {
        match kind {
            Kind::Simple => {
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
            Kind::Complex(Complex { as_storage, .. }) => {
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
fn map_storage_iter(
    cx: &Ctxt<'_>,
    assoc_type: &str,
    fields: &Fields<'_>,
    output: &mut Output,
) -> Result<(), ()> {
    let type_name = format_ident!("{MAP_STORAGE}{assoc_type}");
    let assoc_type = Ident::new(assoc_type, Span::call_site());

    let lt = cx.lt;
    let ident = &cx.ast.ident;
    let vis = &cx.ast.vis;

    let option = cx.toks.option();
    let iterator_t = cx.toks.iterator_t();
    let double_ended_iterator_t = cx.toks.double_ended_iterator_t();
    let clone_t = cx.toks.clone_t();

    let mut step_forward = IteratorNext::default();
    let mut step_backward = IteratorNextBack::default();

    let mut field_decls = Vec::new();
    let mut init = Vec::new();

    build_iter_next(
        cx,
        &mut step_forward,
        &mut step_backward,
        fields,
        &assoc_type,
        Some(cx.lt),
    )?;

    for Field { name, kind, .. } in fields {
        match kind {
            Kind::Simple => {
                field_decls.push(quote!(#name: #option<&#lt V>));
                init.push(quote!(#name: #option::as_ref(&self.#name)));
            }
            Kind::Complex(Complex { as_storage, .. }) => {
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
    let names = fields.names();

    output.impls.extend(quote! {
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
                    #(#names: #clone_t::clone(&self.#names),)*
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
    });

    let end = fields.len();

    output.items.extend(quote! {
        type #assoc_type<#lt> = #type_name<#lt, V> where V: #lt;

        #[inline]
        fn iter(&self) -> Self::#assoc_type<'_> {
            #type_name { start: 0, end: #end, #(#init,)* }
        }
    });

    Ok(())
}

/// Constructs a key's `Iterator` implementation.
fn map_storage_keys(
    cx: &Ctxt<'_>,
    assoc_type: &str,
    fields: &Fields<'_>,
    output: &mut Output,
) -> Result<(), ()> {
    let type_name = format_ident!("{MAP_STORAGE}{assoc_type}");
    let assoc_type = Ident::new(assoc_type, Span::call_site());

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

    let mut field_decls = Vec::new();
    let mut init = Vec::new();

    for Field {
        span,
        index,
        name,
        var,
        kind,
        ..
    } in fields
    {
        match kind {
            Kind::Simple => {
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
            Kind::Complex(Complex { as_storage, .. }) => {
                field_decls.push(quote!(#name: #as_storage::#assoc_type<#lt>));
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

                let assoc_type = quote!(#as_storage::#assoc_type<#lt>);

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
    let names = fields.names();

    output.impls.extend(quote! {
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
                    #(#names: #clone_t::clone(&self.#names),)*
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
    });

    let end = fields.len();

    output.items.extend(quote! {
        type #assoc_type<#lt> = #type_name<#lt, V> where V: #lt;

        #[inline]
        fn keys(&self) -> Self::#assoc_type<'_> {
            #type_name { start: 0, end: #end, #(#init,)* }
        }
    });

    Ok(())
}

/// Construct a values `Iterator` implementation.
fn map_storage_values(
    cx: &Ctxt<'_>,
    assoc_type: &str,
    fields: &Fields<'_>,
    output: &mut Output,
) -> Result<(), ()> {
    let type_name = format_ident!("{MAP_STORAGE}{assoc_type}");
    let assoc_type = Ident::new(assoc_type, Span::call_site());

    let lt = cx.lt;
    let vis = &cx.ast.vis;

    let clone_t = cx.toks.clone_t();
    let double_ended_iterator_t = cx.toks.double_ended_iterator_t();
    let iterator_t = cx.toks.iterator_t();
    let option = cx.toks.option();

    let mut step_forward = IteratorNext::default();
    let mut step_backward = IteratorNextBack::default();

    let mut field_decls = Vec::new();
    let mut init = Vec::new();

    for Field {
        span,
        index,
        name,
        kind,
        ..
    } in fields
    {
        match kind {
            Kind::Simple => {
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
            Kind::Complex(Complex { as_storage, .. }) => {
                field_decls.push(quote!(#name: #as_storage::#assoc_type<#lt>));
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

                let assoc_type = quote!(#as_storage::#assoc_type<#lt>);

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
    let names = fields.names();

    output.impls.extend(quote! {
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
                    #(#names: #clone_t::clone(&self.#names),)*
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
    });

    let end = fields.len();

    output.items.extend(quote! {
        type #assoc_type<#lt> = #type_name<#lt, V> where V: #lt;

        #[inline]
        fn values(&self) -> Self::#assoc_type<'_> {
            #type_name { start: 0, end: #end, #(#init,)* }
        }
    });

    Ok(())
}

/// Construct an iterator implementation.
fn map_storage_iter_mut(
    cx: &Ctxt<'_>,
    assoc_type: &str,
    fields: &Fields<'_>,
    output: &mut Output,
) -> Result<(), ()> {
    let type_name = format_ident!("{MAP_STORAGE}{assoc_type}");
    let assoc_type = Ident::new(assoc_type, Span::call_site());

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
        fields,
        &assoc_type,
        Some(cx.lt),
    )?;

    for Field { name, kind, .. } in fields {
        match kind {
            Kind::Simple => {
                field_decls.push(quote!(#name: #option<&#lt mut V>));
                init.push(quote!(#name: #option::as_mut(&mut self.#name)));
            }
            Kind::Complex(Complex {
                as_storage,
                storage,
                ..
            }) => {
                field_decls.push(quote!(#name: #as_storage::#assoc_type<#lt>));
                init.push(quote!(#name: #storage::iter_mut(&mut self.#name)));
            }
        }
    }

    step_backward
        .make_where_clause()
        .predicates
        .push(cx.fallible(|| syn::parse2(quote!(V: #lt)))?);

    let double_ended_where = &step_backward.where_clause;

    output.impls.extend(quote! {
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
    });

    let end = fields.len();

    output.items.extend(quote! {
        type #assoc_type<#lt> = #type_name<#lt, V> where V: #lt;

        #[inline]
        fn iter_mut(&mut self) -> Self::#assoc_type<'_> {
            #type_name { start: 0, end: #end, #(#init,)* }
        }
    });

    Ok(())
}

/// Construct a values mutable `Iterator` implementation.
fn map_storage_values_mut(
    cx: &Ctxt<'_>,
    assoc_type: &str,
    fields: &Fields<'_>,
    output: &mut Output,
) -> Result<(), ()> {
    let type_name = format_ident!("{MAP_STORAGE}{assoc_type}");
    let assoc_type = Ident::new(assoc_type, Span::call_site());

    let lt = cx.lt;
    let vis = &cx.ast.vis;

    let option = cx.toks.option();
    let iterator_t = cx.toks.iterator_t();
    let double_ended_iterator_t = cx.toks.double_ended_iterator_t();

    let mut step_forward = IteratorNext::default();
    let mut step_backward = IteratorNextBack::default();

    let mut field_decls = Vec::new();
    let mut init = Vec::new();

    for Field {
        span,
        index,
        name,
        kind,
        ..
    } in fields
    {
        match kind {
            Kind::Simple => {
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
            Kind::Complex(Complex { as_storage, .. }) => {
                field_decls.push(quote!(#name: #as_storage::#assoc_type<#lt>));
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

                let assoc_type = quote!(#as_storage::#assoc_type<#lt>);

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

    output.impls.extend(quote! {
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
    });

    let end = fields.len();

    output.items.extend(quote! {
        type #assoc_type<#lt> = #type_name<#lt, V> where V: #lt;

        #[inline]
        fn values_mut(&mut self) -> Self::#assoc_type<'_> {
            #type_name { start: 0, end: #end, #(#init,)* }
        }
    });

    Ok(())
}

/// Construct `IntoIter` implementation.
fn map_storage_into_iter(
    cx: &Ctxt<'_>,
    assoc_type: &str,
    fields: &Fields<'_>,
    output: &mut Output,
) -> Result<(), ()> {
    let type_name = format_ident!("{MAP_STORAGE}{assoc_type}");
    let assoc_type = Ident::new(assoc_type, Span::call_site());

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

    build_iter_next(
        cx,
        &mut step_forward,
        &mut step_backward,
        fields,
        &assoc_type,
        None,
    )?;

    for Field { name, kind, .. } in fields {
        match kind {
            Kind::Simple => {
                field_decls.push(quote!(#name: #option<V>));
                init.push(quote!(#name: self.#name));
            }
            Kind::Complex(Complex {
                as_storage,
                storage,
                ..
            }) => {
                field_decls.push(quote!(#name: #as_storage::#assoc_type));
                init.push(quote!(#name: #storage::into_iter(self.#name)));
            }
        }
    }

    let double_ended_where = &step_backward.where_clause;
    let names = fields.names();
    let clone_bounds = fields
        .complex()
        .map(|Complex { as_storage, .. }| quote!(#as_storage::#assoc_type: #clone_t));

    output.impls.extend(quote! {
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
                    #(#names: #clone_t::clone(&self.#names),)*
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
    });

    let end = fields.len();

    output.items.extend(quote! {
        type #assoc_type = #type_name<V>;

        #[inline]
        fn into_iter(self) -> Self::#assoc_type {
            #type_name { start: 0, end: #end, #(#init,)* }
        }
    });

    Ok(())
}

/// Constructs a sets iterator implementation.
fn set_storage_iter(
    cx: &Ctxt<'_>,
    assoc_type: &str,
    fields: &Fields<'_>,
    output: &mut Output,
) -> Result<(), ()> {
    let type_name = format_ident!("{SET_STORAGE}{assoc_type}");
    let assoc_type = Ident::new(assoc_type, Span::call_site());

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

    let mut field_decls = Vec::new();
    let mut init = Vec::new();

    for Field {
        span,
        index,
        name,
        var,
        kind,
        ..
    } in fields
    {
        match kind {
            Kind::Simple => {
                field_decls.push(quote!(#name: #bool_type));
                init.push(quote!(#name: self.#name));

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
            Kind::Complex(Complex { as_set_storage, .. }) => {
                field_decls.push(quote!(#name: #as_set_storage::#assoc_type<#lt>));
                init.push(quote!(#name: #as_set_storage::iter(&self.#name)));

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

                let assoc_type = quote!(#as_set_storage::#assoc_type<#lt>);

                where_clause.predicates.push(cx.fallible(|| syn::parse2(quote_spanned! {
                    *span => #assoc_type: #double_ended_iterator_t<Item = <#assoc_type as #iterator_t>::Item>
                }))?);
            }
        }
    }

    let double_ended_where_clause = &step_backward.where_clause;
    let names = fields.names();

    output.impls.extend(quote! {
        #vis struct #type_name<#lt> {
            start: usize,
            end: usize,
            #(#field_decls,)*
        }

        #[automatically_derived]
        impl<#lt> #clone_t for #type_name<#lt> {
            #[inline]
            fn clone(&self) -> Self {
                Self {
                    start: self.start,
                    end: self.end,
                    #(#names: #clone_t::clone(&self.#names),)*
                }
            }
        }

        #[automatically_derived]
        impl<#lt> #iterator_t for #type_name<#lt> {
            type Item = #ident;

            #[inline]
            fn next(&mut self) -> #option<Self::Item> {
                #step_forward
                #option::None
            }
        }

        #[automatically_derived]
        impl<#lt> #double_ended_iterator_t for #type_name<#lt> #double_ended_where_clause {
            #[inline]
            fn next_back(&mut self) -> #option<Self::Item> {
                #step_backward
                #option::None
            }
        }
    });

    let end = fields.len();

    output.items.extend(quote! {
        type #assoc_type<#lt> = #type_name<#lt>;

        #[inline]
        fn iter(&self) -> Self::#assoc_type<'_> {
            #type_name { start: 0, end: #end, #(#init,)* }
        }
    });

    Ok(())
}

/// Constructs a sets owning iterator implementation.
fn set_storage_into_iter(
    cx: &Ctxt<'_>,
    assoc_type: &str,
    fields: &Fields<'_>,
    output: &mut Output,
) -> Result<(), ()> {
    let type_name = format_ident!("{SET_STORAGE}{assoc_type}");
    let assoc_type = Ident::new(assoc_type, Span::call_site());

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

    let mut field_decls = Vec::new();
    let mut init = Vec::new();

    for Field {
        span,
        index,
        name,
        var,
        kind,
        ..
    } in fields
    {
        match kind {
            Kind::Simple => {
                field_decls.push(quote!(#name: #bool_type));
                init.push(quote!(#name: self.#name));

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
            Kind::Complex(Complex { as_set_storage, .. }) => {
                field_decls.push(quote!(#name: #as_set_storage::#assoc_type));
                init.push(quote!(#name: #as_set_storage::into_iter(self.#name)));

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

                let assoc_type = quote!(#as_set_storage::#assoc_type);

                where_clause.predicates.push(cx.fallible(|| syn::parse2(quote_spanned! {
                    *span => for<'trivial_bounds> #assoc_type: #double_ended_iterator_t<Item = <#assoc_type as #iterator_t>::Item>
                }))?);
            }
        }
    }

    let names = fields.names();

    output.impls.extend(quote! {
        #vis struct #type_name {
            start: usize,
            end: usize,
            #(#field_decls,)*
        }
    });

    {
        let bounds = fields
            .complex()
            .map(|Complex { as_set_storage, .. }| quote!(#as_set_storage::#assoc_type));

        output.impls.extend(quote! {
            #[automatically_derived]
            impl #clone_t for #type_name where #(for<'trivial_bounds> #bounds: #clone_t,)* {
                #[inline]
                fn clone(&self) -> Self {
                    Self {
                        start: self.start,
                        end: self.end,
                        #(#names: #clone_t::clone(&self.#names),)*
                    }
                }
            }
        });
    }

    output.impls.extend(quote! {
        #[automatically_derived]
        impl #iterator_t for #type_name {
            type Item = #ident;

            #[inline]
            fn next(&mut self) -> #option<Self::Item> {
                #step_forward
                #option::None
            }
        }
    });

    let double_ended_where_clause = &step_backward.where_clause;

    output.impls.extend(quote! {
        #[automatically_derived]
        impl #double_ended_iterator_t for #type_name #double_ended_where_clause {
            #[inline]
            fn next_back(&mut self) -> #option<Self::Item> {
                #step_backward
                #option::None
            }
        }
    });

    let end = fields.len();

    output.items.extend(quote! {
        type #assoc_type = #type_name;

        #[inline]
        fn into_iter(self) -> Self::#assoc_type {
            #type_name { start: 0, end: #end, #(#init,)* }
        }
    });

    Ok(())
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
fn map_storage_entry(
    cx: &Ctxt<'_>,
    fields: &Fields<'_>,
    storage: &Ident,
    output: &mut Output,
) -> Result<(), ()> {
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

    for Field {
        name, kind, var, ..
    } in fields
    {
        let pattern = quote!(#ident::#var);

        match kind {
            Kind::Simple => {
                init.push(quote!( #pattern => option_to_entry(&mut self.#name, key) ));
            }
            Kind::Complex(Complex {
                element, storage, ..
            }) => {
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

    output.impls.extend(quote! {
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
        fn option_to_entry<V>(opt: &mut #option<V>, key: #ident) -> #entry_enum<'_, #storage<V>, #ident, V> {
            match #option_bucket_option::new(opt) {
                #option_bucket_option::Some(inner) => #entry_enum::Occupied(OccupiedEntry::Simple(SimpleOccupiedEntry { key, inner })),
                #option_bucket_option::None(inner) => #entry_enum::Vacant(VacantEntry::Simple(SimpleVacantEntry { key, inner })),
            }
        }
    });

    output.items.extend(quote! {
        type Occupied<#lt> = OccupiedEntry<#lt, V> where V: #lt;
        type Vacant<#lt> = VacantEntry<#lt, V> where V: #lt;

        #[inline]
        fn entry(&mut self, key: #ident) -> #entry_enum<'_, Self, #ident, V> {
            match key {
                #(#init,)*
            }
        }
    });

    Ok(())
}

/// Output collector.
#[derive(Default)]
struct Output {
    impls: TokenStream,
    items: TokenStream,
}

/// A field specification.
pub(crate) struct Field<'a> {
    pub(crate) span: Span,
    pub(crate) index: usize,
    /// Index-based name (`f1`, `f2`)
    pub(crate) name: Ident,
    /// Variant name
    pub(crate) var: &'a Ident,
    pub(crate) kind: Kind<'a>,
}

/// The kind of a single storage element.
pub(crate) enum Kind<'a> {
    Simple,
    Complex(Complex<'a>),
}

/// A complex field kind.
pub(crate) struct Complex<'a> {
    /// Type of variant field
    pub(crate) element: &'a syn::Field,
    /// <E as Key>::Storage::<V> (E = type of variant field)
    pub(crate) storage: TokenStream,
    /// <<E as Key>::Storage::<V> as Storage<E, V>> (E = type of variant field)
    pub(crate) as_storage: TokenStream,
    /// <E as Key>::SetStorage (E = type of variant field)
    pub(crate) set_storage: TokenStream,
    /// <<E as Key>::SetStorage as SetStorage<E>> (E = type of variant field)
    pub(crate) as_set_storage: TokenStream,
}

#[derive(Default)]
pub(crate) struct Fields<'a> {
    fields: Vec<Field<'a>>,
    patterns: Vec<Pat>,
}

impl<'a> Fields<'a> {
    /// Get names of all the fields.
    fn names(&self) -> impl Iterator<Item = &'_ Ident> {
        self.fields.iter().map(|f| &f.name)
    }

    /// Get names of all the fields.
    fn complex(&self) -> impl Iterator<Item = &'_ Complex<'a>> {
        self.fields.iter().filter_map(|f| match &f.kind {
            Kind::Complex(c) => Some(c),
            Kind::Simple => None,
        })
    }

    /// Iterate over fields.
    fn iter(&self) -> ::core::slice::Iter<'_, Field<'a>> {
        self.fields.iter()
    }

    /// Length of fields.
    fn len(&self) -> usize {
        self.fields.len()
    }
}

impl<'b, 'a> IntoIterator for &'b Fields<'a> {
    type Item = &'b Field<'a>;
    type IntoIter = ::core::slice::Iter<'b, Field<'a>>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.fields.iter()
    }
}
