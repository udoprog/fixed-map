//! Module that defines the [`Intersection`] for [`Set`].

use core::fmt;

use super::{Iter, Key, Set};

/// A lazy iterator producing elements in the intersection of `Set`s.
///
/// This `struct` is created by the [`intersection`] method on [`Set`]. See its
/// documentation for more.
///
/// [`intersection`]: Set::intersection
///
/// # Examples
///
/// ```
/// use fixed_map::{Key, Set};
///
/// #[derive(Clone, Copy, Key, Debug)]
/// enum K {
///     One,
///     Two,
///     Three,
/// }
///
/// let a = Set::from([K::One]);
/// let b = Set::from([K::One, K::Two, K::Two]);
///
/// let intersection = a.intersection(&b).collect::<Set<_>>();
/// assert_eq!(intersection, Set::from([K::One]));
/// ```
#[must_use = "this returns the intersection as an iterator, \
              without modifying either input set"]
pub struct Intersection<'a, T: 'a + Key> {
    // iterator of the first set
    pub(super) iter: Iter<'a, T>,
    // the second set
    pub(super) other: &'a Set<T>,
}

impl<T: Key> Clone for Intersection<'_, T> {
    #[inline]
    fn clone(&self) -> Self {
        Intersection {
            iter: self.iter.clone(),
            ..*self
        }
    }
}

impl<'a, T> Iterator for Intersection<'a, T>
where
    T: Key,
{
    type Item = T;

    #[inline]
    fn next(&mut self) -> Option<T> {
        loop {
            let elt = self.iter.next()?;

            if self.other.contains(elt) {
                return Some(elt);
            }
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let (_, upper) = self.iter.size_hint();
        (0, upper)
    }

    #[inline]
    fn fold<B, F>(self, init: B, mut f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
    {
        self.iter.fold(init, |acc, elt| {
            if self.other.contains(elt) {
                f(acc, elt)
            } else {
                acc
            }
        })
    }
}

impl<T> fmt::Debug for Intersection<'_, T>
where
    T: fmt::Debug + Key,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.clone()).finish()
    }
}
