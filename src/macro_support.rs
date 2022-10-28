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
#[doc(hidden)]
#[allow(clippy::missing_inline_in_public_items)]
pub fn __storage_iterator_partial_cmp<'a, A, B, T: 'a>(a: A, b: B) -> Option<Ordering>
where
    A: IntoIterator<Item = &'a Option<T>>,
    B: IntoIterator<Item = &'a Option<T>>,
    T: PartialOrd<T>,
{
    let a = a.into_iter().enumerate().filter_map(flatten);
    let b = b.into_iter().enumerate().filter_map(flatten);
    a.partial_cmp(b)
}

/// `cmp` implementation over iterators which ensures that storage ordering
/// between `None` and `Some` is handled in a reasonable manner.
#[doc(hidden)]
#[allow(clippy::missing_inline_in_public_items)]
pub fn __storage_iterator_cmp<'a, A, B, T: 'a>(a: A, b: B) -> Ordering
where
    A: IntoIterator<Item = &'a Option<T>>,
    B: IntoIterator<Item = &'a Option<T>>,
    T: Ord,
{
    let a = a.into_iter().enumerate().filter_map(flatten);
    let b = b.into_iter().enumerate().filter_map(flatten);
    a.cmp(b)
}
