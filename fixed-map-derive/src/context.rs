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
    pub(crate) double_ended_iterator_t: TokenStream,
    pub(crate) eq: TokenStream,
    pub(crate) into_iter: TokenStream,
    pub(crate) iterator_t: TokenStream,
    pub(crate) iterator_flatten: TokenStream,
    pub(crate) key_trait: TokenStream,
    pub(crate) mem: TokenStream,
    pub(crate) option: TokenStream,
    pub(crate) partial_eq: TokenStream,
    pub(crate) slice_iter_mut: TokenStream,
    pub(crate) slice_iter: TokenStream,
    pub(crate) storage_trait: TokenStream,
    pub(crate) iterator_flat_map: TokenStream,
    pub(crate) storage_entry_trait: TokenStream,
    pub(crate) occupied_entry_trait: TokenStream,
    pub(crate) vacant_entry_trait: TokenStream,
    pub(crate) entry_enum: TokenStream,
    pub(crate) option_bucket_option: TokenStream,
    pub(crate) option_bucket_some: TokenStream,
    pub(crate) option_bucket_none: TokenStream,
}

impl Toks {
    pub(crate) fn new(krate: &TokenStream) -> Self {
        Self {
            array_into_iter: quote!(::core::array::IntoIter),
            bool_type: quote!(::core::primitive::bool),
            clone: quote!(::core::clone::Clone),
            copy: quote!(::core::marker::Copy),
            default: quote!(::core::default::Default),
            double_ended_iterator_t: quote!(::core::iter::DoubleEndedIterator),
            eq: quote!(::core::cmp::Eq),
            into_iter: quote!(::core::iter::IntoIterator::into_iter),
            iterator_t: quote!(::core::iter::Iterator),
            iterator_flatten: quote!(::core::iter::Flatten),
            key_trait: quote!(#krate::key::Key),
            mem: quote!(::core::mem),
            option: quote!(::core::option::Option),
            partial_eq: quote!(::core::cmp::PartialEq),
            slice_iter_mut: quote!(::core::slice::IterMut),
            slice_iter: quote!(::core::slice::Iter),
            storage_trait: quote!(#krate::storage::Storage),
            iterator_flat_map: quote!(::core::iter::FlatMap),
            storage_entry_trait: quote!(#krate::storage::entry::StorageEntry),
            occupied_entry_trait: quote!(#krate::storage::entry::OccupiedEntry),
            vacant_entry_trait: quote!(#krate::storage::entry::VacantEntry),
            entry_enum: quote!(#krate::storage::entry::Entry),
            option_bucket_option: quote!(#krate::option_bucket::OptionBucket),
            option_bucket_some: quote!(#krate::option_bucket::SomeBucket),
            option_bucket_none: quote!(#krate::option_bucket::NoneBucket),
        }
    }
}

pub(crate) struct Ctxt<'a> {
    /// Errors collected in the context.
    errors: RefCell<Vec<syn::Error>>,
    /// Generated tokens.
    pub(crate) toks: &'a Toks,
    /// Input ast.
    pub(crate) ast: &'a DeriveInput,
    /// Usable lifetime parameter.
    pub(crate) lt: &'a syn::Lifetime,
}

impl<'a> Ctxt<'a> {
    pub(crate) fn new(tokens: &'a Toks, ast: &'a DeriveInput, lt: &'a syn::Lifetime) -> Self {
        Self {
            errors: RefCell::new(Vec::new()),
            toks: tokens,
            ast,
            lt,
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

    /// Perform a fallible operation and capture an error (if any).
    pub(crate) fn fallible<T, O>(&self, op: T) -> Result<O, ()>
    where
        T: FnOnce() -> Result<O, syn::Error>,
    {
        match op() {
            Ok(output) => Ok(output),
            Err(error) => {
                self.errors.borrow_mut().push(error);
                Err(())
            }
        }
    }
}

/// A field specification.
pub(crate) struct FieldSpec<'a> {
    pub(crate) span: Span,
    pub(crate) index: usize,
    /// Index-based name (`f1`, `f2`)
    pub(crate) name: syn::Ident,
    /// Variant name
    pub(crate) var: &'a syn::Ident,
    pub(crate) kind: FieldKind,
}

/// The kind of a field.
pub(crate) enum FieldKind {
    Simple,
    Complex {
        /// Type of variant field
        element: TokenStream,
        /// <E as Key>::Storage::<V> (E = type of variant field)
        storage: TokenStream,
        /// <<E as Key>::Storage::<V> as Storage<E, V>> (E = type of variant field)
        as_storage: TokenStream,
    },
}
