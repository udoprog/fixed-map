use core::hash::Hash;
use core::iter;

use crate::set::SetStorage;

/// [`SetStorage`] for dynamically stored types, using [`hashbrown::HashSet`].
///
/// This allows for dynamic types such as `&'static str` or `u32` to be used as
/// a [`Key`][crate::Key].
///
/// # Examples
///
/// ```
/// use fixed_map::{Key, Set};
///
/// #[derive(Clone, Copy, Key)]
/// enum MyKey {
///     First(u32),
///     Second,
/// }
///
/// let mut map = Set::new();
/// map.insert(MyKey::First(1));
/// assert_eq!(map.contains(MyKey::First(1)), true);
/// assert_eq!(map.contains(MyKey::First(2)), false);
/// assert_eq!(map.contains(MyKey::Second), false);
/// ```
#[repr(transparent)]
pub struct HashbrownSetStorage<T> {
    inner: ::hashbrown::HashSet<T>,
}

impl<T> Clone for HashbrownSetStorage<T>
where
    T: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        HashbrownSetStorage {
            inner: self.inner.clone(),
        }
    }
}

impl<T> PartialEq for HashbrownSetStorage<T>
where
    T: Eq + Hash,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.inner.eq(&other.inner)
    }
}

impl<T> Eq for HashbrownSetStorage<T> where T: Eq + Hash {}

impl<T> SetStorage<T> for HashbrownSetStorage<T>
where
    T: Copy + Eq + Hash,
{
    type Iter<'this>
        = iter::Copied<::hashbrown::hash_set::Iter<'this, T>>
    where
        T: 'this;
    type IntoIter = ::hashbrown::hash_set::IntoIter<T>;

    #[inline]
    fn empty() -> Self {
        Self {
            inner: ::hashbrown::HashSet::new(),
        }
    }

    #[inline]
    fn len(&self) -> usize {
        self.inner.len()
    }

    #[inline]
    fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    #[inline]
    fn insert(&mut self, value: T) -> bool {
        self.inner.insert(value)
    }

    #[inline]
    fn contains(&self, value: T) -> bool {
        self.inner.contains(&value)
    }

    #[inline]
    fn remove(&mut self, value: T) -> bool {
        self.inner.remove(&value)
    }

    #[inline]
    fn retain<F>(&mut self, mut func: F)
    where
        F: FnMut(T) -> bool,
    {
        self.inner.retain(|&value| func(value));
    }

    #[inline]
    fn clear(&mut self) {
        self.inner.clear();
    }

    #[inline]
    fn iter(&self) -> Self::Iter<'_> {
        self.inner.iter().copied()
    }

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}
