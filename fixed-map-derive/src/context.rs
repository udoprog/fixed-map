use core::cell::RefCell;
use core::fmt;

use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::DeriveInput;

pub(crate) struct Toks {
    pub(crate) array_into_iter: TokenStream,
    pub(crate) bool_type: TokenStream,
    pub(crate) clone: TokenStream,
    pub(crate) copy: TokenStream,
    pub(crate) default: TokenStream,
    pub(crate) eq: TokenStream,
    pub(crate) into_iter: TokenStream,
    pub(crate) iterator: TokenStream,
    pub(crate) key_trait: TokenStream,
    pub(crate) mem: TokenStream,
    pub(crate) option: TokenStream,
    pub(crate) partial_eq: TokenStream,
    pub(crate) slice_iter: TokenStream,
    pub(crate) slice_iter_mut: TokenStream,
    pub(crate) storage_trait: TokenStream,
}

impl Toks {
    pub(crate) fn new(krate: &TokenStream) -> Self {
        Self {
            array_into_iter: quote!(::core::array::IntoIter),
            bool_type: quote!(::core::primitive::bool),
            clone: quote!(::core::clone::Clone),
            copy: quote!(::core::marker::Copy),
            default: quote!(::core::default::Default),
            eq: quote!(::core::cmp::Eq),
            into_iter: quote!(::core::iter::IntoIterator::into_iter),
            iterator: quote!(::core::iter::Iterator),
            key_trait: quote!(#krate::key::Key),
            mem: quote!(::core::mem),
            option: quote!(::core::option::Option),
            partial_eq: quote!(::core::cmp::PartialEq),
            slice_iter: quote!(::core::slice::Iter),
            slice_iter_mut: quote!(::core::slice::IterMut),
            storage_trait: quote!(#krate::storage::Storage),
        }
    }
}

pub(crate) struct Ctxt<'a> {
    /// Errors collected in the context.
    errors: RefCell<Vec<syn::Error>>,
    /// Generated tokens.
    pub(crate) tokens: &'a Toks,
    /// Input ast.
    pub(crate) ast: &'a DeriveInput,
}

impl<'a> Ctxt<'a> {
    pub(crate) fn new(tokens: &'a Toks, ast: &'a DeriveInput) -> Self {
        Self {
            errors: RefCell::new(Vec::new()),
            tokens,
            ast,
        }
    }

    /// Emit an error.
    pub(crate) fn error(&self, span: Span, message: impl fmt::Display) {
        self.errors
            .borrow_mut()
            .push(syn::Error::new(span, message));
    }

    /// Convert into interior errors.
    pub(crate) fn into_errors(self) -> Vec<syn::Error> {
        self.errors.into_inner()
    }
}

/// A field specification.
pub(crate) struct FieldSpec<'a> {
    pub(crate) index: usize,
    pub(crate) name: syn::Ident,
    pub(crate) var: &'a syn::Ident,
    pub(crate) kind: FieldKind,
}

/// The kind of a field.
pub(crate) enum FieldKind {
    Simple,
    Complex {
        as_storage: TokenStream,
        storage: TokenStream,
    },
}
