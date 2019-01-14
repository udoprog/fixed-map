use crate::storage::Storage;
use std::marker;
use std::mem;

/// Storage types that can only inhabit a single value (like `()`).
pub struct SingletonStorage<K, V> {
    inner: Option<V>,
    key: marker::PhantomData<K>,
}

impl<K, V> Clone for SingletonStorage<K, V>
where
    V: Clone,
{
    fn clone(&self) -> Self {
        SingletonStorage {
            inner: self.inner.clone(),
            key: marker::PhantomData,
        }
    }
}

impl<K, V> Default for SingletonStorage<K, V> {
    fn default() -> Self {
        Self {
            inner: Default::default(),
            key: marker::PhantomData,
        }
    }
}

impl<K, V> PartialEq for SingletonStorage<K, V>
where
    V: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl<K, V> Eq for SingletonStorage<K, V> where V: Eq {}

pub struct Iter<K, V> {
    value: Option<(K, *const V)>,
}

impl<K, V> Clone for Iter<K, V>
where
    K: Copy,
{
    fn clone(&self) -> Self {
        Iter {
            value: self.value.clone(),
        }
    }
}

impl<K, V> Iterator for Iter<K, V> {
    type Item = (K, *const V);

    fn next(&mut self) -> Option<Self::Item> {
        self.value.take()
    }
}

pub struct IterMut<K, V> {
    value: Option<(K, *mut V)>,
}

impl<K, V> Iterator for IterMut<K, V> {
    type Item = (K, *mut V);

    fn next(&mut self) -> Option<Self::Item> {
        self.value.take()
    }
}

impl<K, V> Storage<K, V> for SingletonStorage<K, V>
where
    K: Copy + Default,
{
    type Iter = Iter<K, V>;
    type IterMut = IterMut<K, V>;

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
    fn iter(&self) -> Self::Iter {
        Iter {
            value: self.inner.as_ref().map(|v| (K::default(), v as *const V)),
        }
    }

    #[inline]
    fn iter_mut(&mut self) -> Self::IterMut {
        IterMut {
            value: self.inner.as_mut().map(|v| (K::default(), v as *mut V)),
        }
    }
}
