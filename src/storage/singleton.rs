use core::mem;

use crate::storage::Storage;

/// Storage types that can only inhabit a single value (like `()`).
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct SingletonStorage<V> {
    inner: Option<V>,
}

impl<V> Default for SingletonStorage<V> {
    #[inline]
    fn default() -> Self {
        Self {
            inner: Option::default(),
        }
    }
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

pub struct Iter<'a, K, V> {
    value: Option<(K, &'a V)>,
}

impl<K, V> Clone for Iter<'_, K, V>
where
    K: Copy,
{
    #[inline]
    fn clone(&self) -> Self {
        Iter { value: self.value }
    }
}

impl<'a, K, V> Iterator for Iter<'a, K, V> {
    type Item = (K, &'a V);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.value.take()
    }
}

pub struct IterMut<'a, K, V> {
    value: Option<(K, &'a mut V)>,
}

impl<'a, K, V> Iterator for IterMut<'a, K, V> {
    type Item = (K, &'a mut V);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.value.take()
    }
}

impl<K, V> DoubleEndedIterator for IterMut<'_, K, V> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.value.take()
    }
}

pub struct IntoIter<K, V> {
    value: Option<(K, V)>,
}

impl<K, V> Iterator for IntoIter<K, V> {
    type Item = (K, V);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.value.take()
    }
}

impl<K, V> DoubleEndedIterator for IntoIter<K, V> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.value.take()
    }
}

impl<K, V> Storage<K, V> for SingletonStorage<V>
where
    K: Copy + Default,
{
    type Iter<'this> = Iter<'this, K, V> where Self: 'this, V: 'this;
    type Keys<'this> = ::core::option::IntoIter<K> where Self: 'this;
    type Values<'this> = ::core::option::Iter<'this, V> where Self: 'this, V: 'this;
    type IterMut<'this> = IterMut<'this, K, V> where Self: 'this, V: 'this;
    type ValuesMut<'this> = ::core::option::IterMut<'this, V> where Self: 'this, V: 'this;
    type IntoIter = IntoIter<K, V>;

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
    fn clear(&mut self) {
        self.inner = None;
    }

    #[inline]
    fn iter(&self) -> Self::Iter<'_> {
        Iter {
            value: self.inner.as_ref().map(|v| (K::default(), v)),
        }
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
        IterMut {
            value: self.inner.as_mut().map(|v| (K::default(), v)),
        }
    }

    #[inline]
    fn values_mut(&mut self) -> Self::ValuesMut<'_> {
        self.inner.iter_mut()
    }

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            value: self.inner.map(|v| (K::default(), v)),
        }
    }
}
