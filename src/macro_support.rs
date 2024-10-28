//! ## PRIVATE API
//!
//! This API is private, for use only in the `derive(Key)` macro. Usage for
//! other purposes is not supported, and this API will not abide by semver
//! stability guarantees.

#![allow(clippy::missing_inline_in_public_items)]

use core::cmp::Ordering;

#[inline]
fn flatten<T>(value: (usize, &Option<T>)) -> Option<(usize, &T)> {
    match value {
        (index, Some(value)) => Some((index, value)),
        _ => None,
    }
}

/// `partial_cmp` implementation over iterators which ensures that storage
/// ordering between `None` and `Some` is handled in a reasonable manner.
pub fn __storage_iterator_partial_cmp<'a, A, B, T>(a: A, b: B) -> Option<Ordering>
where
    A: IntoIterator<Item = &'a Option<T>>,
    B: IntoIterator<Item = &'a Option<T>>,
    T: 'a + PartialOrd<T>,
{
    let a = a.into_iter().enumerate().filter_map(flatten);
    let b = b.into_iter().enumerate().filter_map(flatten);
    a.partial_cmp(b)
}

/// `cmp` implementation over iterators which ensures that storage ordering
/// between `None` and `Some` is handled in a reasonable manner.
pub fn __storage_iterator_cmp<'a, A, B, T>(a: A, b: B) -> Ordering
where
    A: IntoIterator<Item = &'a Option<T>>,
    B: IntoIterator<Item = &'a Option<T>>,
    T: 'a + Ord,
{
    let a = a.into_iter().enumerate().filter_map(flatten);
    let b = b.into_iter().enumerate().filter_map(flatten);
    a.cmp(b)
}

#[inline]
fn filter_bool(&(_, value): &(usize, &bool)) -> bool {
    *value
}

/// `partial_cmp` implementation over iterators which ensures that storage
/// ordering between `false` and `true` is handled in a reasonable manner.
pub fn __storage_iterator_partial_cmp_bool<'a, A, B>(a: A, b: B) -> Option<Ordering>
where
    A: IntoIterator<Item = &'a bool>,
    B: IntoIterator<Item = &'a bool>,
{
    let a = a.into_iter().enumerate().filter(filter_bool);
    let b = b.into_iter().enumerate().filter(filter_bool);
    a.partial_cmp(b)
}

/// `cmp` implementation over iterators which ensures that storage ordering
/// between `false` and `true` is handled in a reasonable manner.
pub fn __storage_iterator_cmp_bool<'a, A, B>(a: A, b: B) -> Ordering
where
    A: IntoIterator<Item = &'a bool>,
    B: IntoIterator<Item = &'a bool>,
{
    let a = a.into_iter().enumerate().filter(filter_bool);
    let b = b.into_iter().enumerate().filter(filter_bool);
    a.cmp(b)
}
