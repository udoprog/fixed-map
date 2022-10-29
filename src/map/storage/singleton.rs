use core::mem;

use crate::map::{Entry, Storage};
use crate::option_bucket::{NoneBucket, OptionBucket, SomeBucket};

/// [`Storage`] type that can only inhabit a single value (like `()`).
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct SingletonStorage<V> {
    inner: Option<V>,
}

impl<V> PartialEq for SingletonStorage<V>
where
    V: PartialEq,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl<V> Eq for SingletonStorage<V> where V: Eq {}

impl<K, V> Storage<K, V> for SingletonStorage<V>
where
    K: Default,
{
    type Iter<'this> = ::core::option::IntoIter<(K, &'this V)> where V: 'this;
    type Keys<'this> = ::core::option::IntoIter<K> where V: 'this;
    type Values<'this> = ::core::option::Iter<'this, V> where V: 'this;
    type IterMut<'this> = ::core::option::IntoIter<(K, &'this mut V)> where V: 'this;
    type ValuesMut<'this> = ::core::option::IterMut<'this, V> where V: 'this;
    type IntoIter = ::core::option::IntoIter<(K, V)>;
    type Occupied<'this> = SomeBucket<'this, V> where V: 'this;
    type Vacant<'this> = NoneBucket<'this, V> where V: 'this;

    #[inline]
    fn empty() -> Self {
        Self {
            inner: Option::default(),
        }
    }

    #[inline]
    fn len(&self) -> usize {
        usize::from(self.inner.is_some())
    }

    #[inline]
    fn is_empty(&self) -> bool {
        self.inner.is_none()
    }

    #[inline]
    fn insert(&mut self, _: K, value: V) -> Option<V> {
        mem::replace(&mut self.inner, Some(value))
    }

    #[inline]
    fn contains_key(&self, _: K) -> bool {
        self.inner.is_some()
    }

    #[inline]
    fn get(&self, _: K) -> Option<&V> {
        self.inner.as_ref()
    }

    #[inline]
    fn get_mut(&mut self, _: K) -> Option<&mut V> {
        self.inner.as_mut()
    }

    #[inline]
    fn remove(&mut self, _: K) -> Option<V> {
        mem::replace(&mut self.inner, None)
    }

    #[inline]
    fn retain<F>(&mut self, mut func: F)
    where
        F: FnMut(K, &mut V) -> bool,
    {
        if let Some(val) = self.inner.as_mut() {
            if !func(K::default(), val) {
                self.inner = None;
            }
        }
    }

    #[inline]
    fn clear(&mut self) {
        self.inner = None;
    }

    #[inline]
    fn iter(&self) -> Self::Iter<'_> {
        self.inner.as_ref().map(|v| (K::default(), v)).into_iter()
    }

    #[inline]
    fn keys(&self) -> Self::Keys<'_> {
        Some(K::default()).into_iter()
    }

    #[inline]
    fn values(&self) -> Self::Values<'_> {
        self.inner.iter()
    }

    #[inline]
    fn iter_mut(&mut self) -> Self::IterMut<'_> {
        self.inner.as_mut().map(|v| (K::default(), v)).into_iter()
    }

    #[inline]
    fn values_mut(&mut self) -> Self::ValuesMut<'_> {
        self.inner.iter_mut()
    }

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.inner.map(|v| (K::default(), v)).into_iter()
    }

    #[inline]
    fn entry(&mut self, _key: K) -> Entry<'_, Self, K, V> {
        match OptionBucket::new(&mut self.inner) {
            OptionBucket::Some(some) => Entry::Occupied(some),
            OptionBucket::None(none) => Entry::Vacant(none),
        }
    }
}
