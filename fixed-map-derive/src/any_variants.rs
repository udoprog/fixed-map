use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::spanned::Spanned;
use syn::{DataEnum, Fields, Ident};

use crate::context::{Ctxt, FieldKind, FieldSpec};

pub(crate) fn implement(cx: &Ctxt, en: &DataEnum) -> Result<TokenStream, ()> {
    let vis = &cx.ast.vis;
    let ident = &cx.ast.ident;

    let clone = &cx.tokens.clone;
    let copy = &cx.tokens.copy;
    let default = &cx.tokens.default;
    let eq = &cx.tokens.eq;
    let key_trait = &cx.tokens.key_trait;
    let mem = &cx.tokens.mem;
    let option = &cx.tokens.option;
    let partial_eq = &cx.tokens.partial_eq;
    let storage_trait = &cx.tokens.storage_trait;

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
                        step: 0,
                        #(#iter_init,)*
                    }
                }

                #[inline]
                fn keys(&self) -> Self::Keys<'_> {
                    Keys {
                        step: 0,
                        #(#keys_iter_init,)*
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
                fn values_mut(&mut self) -> Self::ValuesMut<'_> {
                    ValuesMut {
                        step: 0,
                        #(#values_mut_init,)*
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
fn build_iter_next(cx: &Ctxt, field_specs: &[FieldSpec<'_>]) -> Result<Vec<TokenStream>, ()> {
    let option = &cx.tokens.option;
    let iterator = &cx.tokens.iterator;
    let ident = &cx.ast.ident;

    let mut output = Vec::new();

    for FieldSpec {
        index,
        name,
        var,
        kind,
    } in field_specs
    {
        match kind {
            FieldKind::Simple => {
                output.push(quote! {
                    #index => {
                        if let #option::Some(value) = self.#name.take() {
                            return #option::Some((#ident::#var, value));
                        }

                        self.step += 1;
                    }
                });
            }
            FieldKind::Complex { .. } => {
                output.push(quote! {
                    #index => {
                        if let #option::Some((key, value)) = #iterator::next(&mut self.#name) {
                            return #option::Some((#ident::#var(key), value));
                        }

                        self.step += 1;
                    }
                });
            }
        }
    }

    Ok(output)
}

/// Construct an iterator implementation.
fn build_iter_impl(
    cx: &Ctxt,
    id: &str,
    field_specs: &[FieldSpec<'_>],
) -> Result<(TokenStream, Vec<TokenStream>), ()> {
    let type_name = syn::Ident::new(id, Span::call_site());

    let option = &cx.tokens.option;
    let iterator = &cx.tokens.iterator;
    let clone = &cx.tokens.clone;
    let ident = &cx.ast.ident;
    let vis = &cx.ast.vis;

    let mut iter_clone = Vec::new();
    let mut iter_fields = Vec::new();
    let mut init = Vec::new();

    let iter_next = build_iter_next(cx, field_specs)?;

    for FieldSpec { name, kind, .. } in field_specs {
        iter_clone.push(quote!(#name: #clone::clone(&self.#name)));

        match kind {
            FieldKind::Simple => {
                iter_fields.push(quote!(#name: #option<&'a V>));
                init.push(quote!(#name: #option::as_ref(&self.#name)));
            }
            FieldKind::Complex { as_storage, .. } => {
                iter_fields.push(quote!(#name: #as_storage::Iter<'a>));
                init.push(quote!(#name: #as_storage::iter(&self.#name)));
            }
        }
    }

    let iter_impl = quote! {
        #vis struct #type_name<'a, V> {
            step: usize,
            #(#iter_fields,)*
        }

        impl<'a, V> #clone for #type_name<'a, V> {
            #[inline]
            fn clone(&self) -> Self {
                Self {
                    step: self.step,
                    #(#iter_clone,)*
                }
            }
        }

        impl<'a, V> #iterator for #type_name<'a, V> {
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
    };

    Ok((iter_impl, init))
}

/// Construct an keys iterator implementation.
fn build_keys_impl(
    cx: &Ctxt,
    id: &str,
    field_specs: &[FieldSpec<'_>],
) -> Result<(TokenStream, Vec<TokenStream>), ()> {
    let type_name = syn::Ident::new(id, Span::call_site());

    let option = &cx.tokens.option;
    let iterator = &cx.tokens.iterator;
    let clone = &cx.tokens.clone;
    let bool_type = &cx.tokens.bool_type;
    let mem = &cx.tokens.mem;
    let ident = &cx.ast.ident;
    let vis = &cx.ast.vis;

    let mut iter_clone = Vec::new();
    let mut iter_fields = Vec::new();
    let mut init = Vec::new();

    let mut iter_next = Vec::new();

    for FieldSpec {
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
                iter_fields.push(quote!(#name: #bool_type));
                init.push(quote!(#name: #option::is_some(&self.#name)));

                iter_next.push(quote! {
                    #index => {
                        if #mem::take(&mut self.#name) {
                            return #option::Some(#ident::#var);
                        }

                        self.step += 1;
                    }
                });
            }
            FieldKind::Complex { as_storage, .. } => {
                iter_fields.push(quote!(#name: #as_storage::Keys<'a>));
                init.push(quote!(#name: #as_storage::keys(&self.#name)));

                iter_next.push(quote! {
                    #index => {
                        if let #option::Some(key) = #iterator::next(&mut self.#name) {
                            return #option::Some(#ident::#var(key));
                        }

                        self.step += 1;
                    }
                });
            }
        }
    }

    let iter_impl = quote! {
        #vis struct #type_name<'a, V> where V: 'a {
            step: usize,
            #(#iter_fields,)*
        }

        impl<V> #clone for #type_name<'_, V> {
            #[inline]
            fn clone(&self) -> Self {
                Self {
                    step: self.step,
                    #(#iter_clone,)*
                }
            }
        }

        impl<V> #iterator for #type_name<'_, V> {
            type Item = #ident;

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
    };

    Ok((iter_impl, init))
}

/// Construct a values iterator implementation.
fn build_values_impl(
    cx: &Ctxt,
    id: &str,
    field_specs: &[FieldSpec<'_>],
) -> Result<(TokenStream, Vec<TokenStream>), ()> {
    let type_name = syn::Ident::new(id, Span::call_site());

    let option = &cx.tokens.option;
    let iterator = &cx.tokens.iterator;
    let clone = &cx.tokens.clone;
    let vis = &cx.ast.vis;

    let mut iter_clone = Vec::new();
    let mut iter_fields = Vec::new();
    let mut init = Vec::new();

    let mut iter_next = Vec::new();

    for FieldSpec {
        index, name, kind, ..
    } in field_specs
    {
        iter_clone.push(quote!(#name: #clone::clone(&self.#name)));

        match kind {
            FieldKind::Simple => {
                iter_fields.push(quote!(#name: #option<&'a V>));
                init.push(quote!(#name: #option::as_ref(&self.#name)));

                iter_next.push(quote! {
                    #index => {
                        if let #option::Some(v) = self.#name.take() {
                            return #option::Some(v);
                        }

                        self.step += 1;
                    }
                });
            }
            FieldKind::Complex { as_storage, .. } => {
                iter_fields.push(quote!(#name: #as_storage::Values<'a>));
                init.push(quote!(#name: #as_storage::values(&self.#name)));

                iter_next.push(quote! {
                    #index => {
                        if let #option::Some(value) = #iterator::next(&mut self.#name) {
                            return #option::Some(value);
                        }

                        self.step += 1;
                    }
                });
            }
        }
    }

    let iter_impl = quote! {
        #vis struct #type_name<'a, V> {
            step: usize,
            #(#iter_fields,)*
        }

        impl<'a, V> #clone for #type_name<'a, V> {
            #[inline]
            fn clone(&self) -> Self {
                Self {
                    step: self.step,
                    #(#iter_clone,)*
                }
            }
        }

        impl<'a, V> #iterator for #type_name<'a, V> {
            type Item = &'a V;

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
    let vis = &cx.ast.vis;
    let iterator = &cx.tokens.iterator;
    let option = &cx.tokens.option;

    let mut field_decls = Vec::new();
    let mut init = Vec::new();

    let iter_next = build_iter_next(cx, field_specs)?;

    for FieldSpec { name, kind, .. } in field_specs {
        match kind {
            FieldKind::Simple => {
                field_decls.push(quote!(#name: #option<&'a mut V>));
                init.push(quote!(#name: #option::as_mut(&mut self.#name)));
            }
            FieldKind::Complex {
                as_storage,
                storage,
            } => {
                field_decls.push(quote!(#name: #as_storage::IterMut<'a>));
                init.push(quote!(#name: #storage::iter_mut(&mut self.#name)));
            }
        }
    }

    let iter_impl = quote! {
        #vis struct #type_name<'a, V> {
            step: usize,
            #(#field_decls,)*
        }

        impl<'a, V> #iterator for #type_name<'a, V> {
            type Item = (#ident, &'a mut V);

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
    };

    Ok((iter_impl, init))
}

/// Construct a values mutable iterator implementation.
fn build_values_mut_impl(
    cx: &Ctxt,
    id: &str,
    field_specs: &[FieldSpec<'_>],
) -> Result<(TokenStream, Vec<TokenStream>), ()> {
    let type_name = syn::Ident::new(id, Span::call_site());

    let option = &cx.tokens.option;
    let iterator = &cx.tokens.iterator;
    let clone = &cx.tokens.clone;
    let vis = &cx.ast.vis;

    let mut iter_clone = Vec::new();
    let mut iter_fields = Vec::new();
    let mut init = Vec::new();

    let mut iter_next = Vec::new();

    for FieldSpec {
        index, name, kind, ..
    } in field_specs
    {
        iter_clone.push(quote!(#name: #clone::clone(&self.#name)));

        match kind {
            FieldKind::Simple => {
                iter_fields.push(quote!(#name: #option<&'a mut V>));
                init.push(quote!(#name: #option::as_mut(&mut self.#name)));

                iter_next.push(quote! {
                    #index => {
                        if let #option::Some(v) = self.#name.take() {
                            return #option::Some(v);
                        }

                        self.step += 1;
                    }
                });
            }
            FieldKind::Complex { as_storage, .. } => {
                iter_fields.push(quote!(#name: #as_storage::ValuesMut<'a>));
                init.push(quote!(#name: #as_storage::values_mut(&mut self.#name)));

                iter_next.push(quote! {
                    #index => {
                        if let #option::Some(value) = #iterator::next(&mut self.#name) {
                            return #option::Some(value);
                        }

                        self.step += 1;
                    }
                });
            }
        }
    }

    let iter_impl = quote! {
        #vis struct #type_name<'a, V> {
            step: usize,
            #(#iter_fields,)*
        }

        impl<'a, V> #iterator for #type_name<'a, V> {
            type Item = &'a mut V;

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
    let option = &cx.tokens.option;
    let clone = &cx.tokens.clone;

    let mut field_decls = Vec::new();
    let mut init = Vec::new();
    let mut field_clone = Vec::new();
    let mut clone_bounds = Vec::new();

    let iter_next = build_iter_next(cx, field_specs)?;

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
            step: usize,
            #(#field_decls,)*
        }

        impl<V> #clone for #type_name<V> where V: Clone, #(#clone_bounds,)* {
            #[inline]
            fn clone(&self) -> Self {
                Self {
                    step: self.step,
                    #(#field_clone,)*
                }
            }
        }

        impl<V> Iterator for #type_name<V> {
            type Item = (#ident, V);

            #[inline]
            fn next(&mut self) -> Option<Self::Item> {
                loop {
                    match self.step {
                        #(#iter_next,)*
                        _ => return None,
                    }
                }
            }
        }
    };

    Ok((iter_impl, init))
}
