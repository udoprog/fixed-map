use ::core::mem;

use crate::set::SetStorage;

/// [`SetStorage`]  types that can only inhabit a single value (like `()`).
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SingletonSetStorage {
    is_set: bool,
}

impl<T> SetStorage<T> for SingletonSetStorage
where
    T: Default + Clone,
{
    type Iter<'this> = ::core::option::IntoIter<T>;
    type IntoIter = ::core::option::IntoIter<T>;

    #[inline]
    fn empty() -> Self {
        Self { is_set: false }
    }

    #[inline]
    fn len(&self) -> usize {
        usize::from(self.is_set)
    }

    #[inline]
    fn is_empty(&self) -> bool {
        !self.is_set
    }

    #[inline]
    fn insert(&mut self, _: T) -> bool {
        !mem::replace(&mut self.is_set, true)
    }

    #[inline]
    fn contains(&self, _: T) -> bool {
        self.is_set
    }

    #[inline]
    fn remove(&mut self, _: T) -> bool {
        mem::replace(&mut self.is_set, false)
    }

    #[inline]
    fn retain<F>(&mut self, mut func: F)
    where
        F: FnMut(T) -> bool,
    {
        self.is_set = func(T::default());
    }

    #[inline]
    fn clear(&mut self) {
        self.is_set = false;
    }

    #[inline]
    fn iter(&self) -> Self::Iter<'_> {
        self.is_set.then_some(T::default()).into_iter()
    }

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.is_set.then_some(T::default()).into_iter()
    }
}
