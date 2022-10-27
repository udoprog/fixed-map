use core::cell::RefCell;
use core::fmt;

use proc_macro2::{Span, TokenStream};
use syn::{DeriveInput, Path};

// Builder function to use when constructing token.
type Builder = fn(&Toks<'_>) -> Path;

// Helper macro to define re-usable token paths.
macro_rules! toks {
    ($vis:vis struct $ident:ident <$lt:lifetime> { $($field:ident = [$($path:tt)*]),* $(,)? }) => {
        $vis struct $ident<$lt> {
            core: Path,
            crate_prefix: &$lt Path,
            $($field: Builder,)*
        }

        impl<$lt> $ident<$lt> {
            /// Construct path tokens with the given prefix.
            pub(crate) fn new(crate_prefix: &$lt Path) -> Self {
                let core = leading_path(["core"]);

                Self {
                    core,
                    crate_prefix,
                    $($field: toks!(@path $($path)*),)*
                }
            }

            $(
                #[inline]
                $vis fn $field(&self) -> Path {
                    let f = self.$field;
                    f(self)
                }
            )*
        }
    };

    (@path ::core $(:: $rest:tt)*) => {
        |s| suffixed(&s.core, [$(stringify!($rest)),*])
    };

    (@path crate $(:: $rest:tt)*) => {
        |s| suffixed(&s.crate_prefix, [$(stringify!($rest)),*])
    };
}

toks! {
    pub(crate) struct Toks<'a> {
        array_into_iter = [::core::array::IntoIter],
        bool_type = [::core::primitive::bool],
        clone_t = [::core::clone::Clone],
        copy_t = [::core::marker::Copy],
        default_t = [::core::default::Default],
        double_ended_iterator_t = [::core::iter::DoubleEndedIterator],
        entry_enum = [crate::storage::entry::Entry],
        eq_t = [::core::cmp::Eq],
        hash_t = [::core::hash::Hash],
        hasher_t = [::core::hash::Hasher],
        into_iterator_t = [::core::iter::IntoIterator],
        iterator_flat_map = [::core::iter::FlatMap],
        iterator_flatten = [::core::iter::Flatten],
        iterator_t = [::core::iter::Iterator],
        key_t = [crate::key::Key],
        mem = [::core::mem],
        occupied_entry_t = [crate::storage::entry::OccupiedEntry],
        option = [::core::option::Option],
        option_bucket_none = [crate::option_bucket::NoneBucket],
        option_bucket_option = [crate::option_bucket::OptionBucket],
        option_bucket_some = [crate::option_bucket::SomeBucket],
        ord_t = [::core::cmp::Ord],
        ordering = [::core::cmp::Ordering],
        partial_eq_t = [::core::cmp::PartialEq],
        partial_ord_t = [::core::cmp::PartialOrd],
        slice_iter = [::core::slice::Iter],
        slice_iter_mut = [::core::slice::IterMut],
        storage_entry_t = [crate::storage::entry::StorageEntry],
        storage_t = [crate::storage::Storage],
        vacant_entry_t = [crate::storage::entry::VacantEntry],
    }
}

/// Construct a leading path.
pub(crate) fn leading_path<const N: usize>(parts: [&'static str; N]) -> Path {
    let mut path = Path {
        leading_colon: Some(<syn::Token![::]>::default()),
        segments: syn::punctuated::Punctuated::default(),
    };

    for part in parts {
        let segment = syn::PathSegment::from(syn::Ident::new(part, Span::call_site()));
        path.segments.push(segment);
    }

    path
}

/// Add the given parts as suffix to the specified prefix path.
fn suffixed<const N: usize>(prefix: &Path, parts: [&'static str; N]) -> Path {
    let mut path = prefix.clone();

    for part in parts {
        let segment = syn::PathSegment::from(syn::Ident::new(part, Span::call_site()));
        path.segments.push(segment);
    }

    path
}

pub(crate) struct Ctxt<'a> {
    /// Errors collected in the context.
    errors: RefCell<Vec<syn::Error>>,
    /// Generated tokens.
    pub(crate) toks: &'a Toks<'a>,
    /// Input ast.
    pub(crate) ast: &'a DeriveInput,
    /// Usable lifetime parameter.
    pub(crate) lt: &'a syn::Lifetime,
}

impl<'a> Ctxt<'a> {
    pub(crate) fn new(tokens: &'a Toks<'a>, ast: &'a DeriveInput, lt: &'a syn::Lifetime) -> Self {
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
