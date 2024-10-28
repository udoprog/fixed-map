use core::cell::RefCell;
use core::fmt;

use proc_macro2::Span;
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
                #[allow(clippy::wrong_self_convention)]
                $vis fn $field(&self) -> Path {
                    let f = self.$field;
                    f(self)
                }
            )*
        }
    };

    (@path core $(:: $rest:tt)*) => {
        |s| suffixed(&s.core, [$(stringify!($rest)),*])
    };

    (@path crate $(:: $rest:tt)*) => {
        |s| suffixed(&s.crate_prefix, [$(stringify!($rest)),*])
    };
}

toks! {
    pub(crate) struct Toks<'a> {
        array_into_iter = [core::array::IntoIter],
        bool_type = [core::primitive::bool],
        clone_t = [core::clone::Clone],
        copy_t = [core::marker::Copy],
        double_ended_iterator_t = [core::iter::DoubleEndedIterator],
        entry_enum = [crate::map::Entry],
        eq_t = [core::cmp::Eq],
        hash_t = [core::hash::Hash],
        hasher_t = [core::hash::Hasher],
        into_iterator_t = [core::iter::IntoIterator],
        iterator_cmp = [crate::macro_support::__storage_iterator_cmp],
        iterator_cmp_bool = [crate::macro_support::__storage_iterator_cmp_bool],
        iterator_flat_map = [core::iter::FlatMap],
        iterator_flatten = [core::iter::Flatten],
        iterator_partial_cmp = [crate::macro_support::__storage_iterator_partial_cmp],
        iterator_partial_cmp_bool = [crate::macro_support::__storage_iterator_partial_cmp_bool],
        iterator_t = [core::iter::Iterator],
        key_t = [crate::Key],
        mem = [core::mem],
        occupied_entry_t = [crate::map::OccupiedEntry],
        option = [core::option::Option],
        option_bucket_none = [crate::option_bucket::NoneBucket],
        option_bucket_option = [crate::option_bucket::OptionBucket],
        option_bucket_some = [crate::option_bucket::SomeBucket],
        ord_t = [core::cmp::Ord],
        ordering = [core::cmp::Ordering],
        partial_eq_t = [core::cmp::PartialEq],
        partial_ord_t = [core::cmp::PartialOrd],
        slice_iter = [core::slice::Iter],
        slice_iter_mut = [core::slice::IterMut],
        map_storage_t = [crate::map::MapStorage],
        set_storage_t = [crate::set::SetStorage],
        raw_storage_t = [crate::raw::RawStorage],
        vacant_entry_t = [crate::map::VacantEntry],
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

/// Options for derive.
#[derive(Default)]
pub(crate) struct Opts {
    /// Implements sets as bitsets when possible.
    pub(crate) bitset: Option<Span>,
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
    pub(crate) fn error(&self, error: syn::Error) {
        self.errors.borrow_mut().push(error);
    }

    /// Emit an error.
    pub(crate) fn span_error(&self, span: Span, message: impl fmt::Display) {
        self.error(syn::Error::new(span, message));
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
