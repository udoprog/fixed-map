use crate::storage::Storage;
use std::mem;

/// Storage types that can only inhabit a single value (like `()`).
pub struct SingletonStorage<V> {
    inner: Option<V>,
}

impl<V> Clone for SingletonStorage<V>
where
    V: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        SingletonStorage {
            inner: self.inner.clone(),
        }
    }
}

impl<V> Default for SingletonStorage<V> {
    #[inline]
    fn default() -> Self {
        Self {
            inner: Default::default(),
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

impl<'a, K, V> Clone for Iter<'a, K, V>
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

impl<K, V> Storage<K, V> for SingletonStorage<V>
where
    K: Copy + Default,
{
    type Iter<'this> = Iter<'this, K, V> where Self: 'this, V: 'this;
    type IterMut<'this> = IterMut<'this, K, V> where Self: 'this, V: 'this;
    type IntoIter = IntoIter<K, V>;

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
    fn iter_mut(&mut self) -> Self::IterMut<'_> {
        IterMut {
            value: self.inner.as_mut().map(|v| (K::default(), v)),
        }
    }

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            value: self.inner.map(|v| (K::default(), v)),
        }
    }
}
