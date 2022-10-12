use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{DataEnum, Ident};

use crate::context::Ctxt;

/// Every variant is a unit variant.
pub(crate) fn implement(cx: &Ctxt, en: &DataEnum) -> Result<TokenStream, ()> {
    let vis = &cx.ast.vis;
    let ident = &cx.ast.ident;

    let array_into_iter = &cx.tokens.array_into_iter;
    let clone = &cx.tokens.clone;
    let copy = &cx.tokens.copy;
    let default = &cx.tokens.default;
    let eq = &cx.tokens.eq;
    let into_iter = &cx.tokens.into_iter;
    let iterator = &cx.tokens.iterator;
    let key_trait = &cx.tokens.key_trait;
    let mem = &cx.tokens.mem;
    let option = &cx.tokens.option;
    let partial_eq = &cx.tokens.partial_eq;
    let slice_iter = &cx.tokens.slice_iter;
    let slice_iter_mut = &cx.tokens.slice_iter_mut;
    let storage_trait = &cx.tokens.storage_trait;

    let const_wrapper = Ident::new(
        &format!("__IMPL_KEY_FOR_{}", cx.ast.ident),
        Span::call_site(),
    );

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
        let field = Ident::new(&format!("f{}", index), Span::call_site());

        names.push(field.clone());

        field_inits.push(quote!(#option::None));

        fields.push(quote!(#option<V>));
        pattern.push(quote!(#ident::#var));

        get.push(quote!(#option::as_ref(#field)));
        get_mut.push(quote!(#option::as_mut(#field)));
        insert.push(quote!(#mem::replace(#field, #option::Some(value))));
        remove.push(quote!(#mem::take(#field)));

        keys_iter_init.push(quote!(if #field.is_some() { Some(#ident::#var) } else { None }));
        iter_init.push(quote!((#ident::#var, #field)));
    }

    let count = en.variants.len();

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
            #vis struct Keys {
                iter: #array_into_iter<#option<#ident>, #count>,
            }

            impl #clone for Keys {
                #[inline]
                fn clone(&self) -> Self {
                    Keys {
                        iter: #clone::clone(&self.iter),
                    }
                }
            }

            impl #iterator for Keys {
                type Item = #ident;

                #[inline]
                fn next(&mut self) -> #option<Self::Item> {
                    loop {
                        if let #option::Some(key) = #iterator::next(&mut self.iter)? {
                            return #option::Some(key);
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
            #vis struct ValuesMut<'a, V> {
                iter: #slice_iter_mut<'a, #option<V>>,
            }

            impl<'a, V> #iterator for ValuesMut<'a, V> {
                type Item = &'a mut V;

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
    })
}
