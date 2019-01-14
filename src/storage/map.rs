use crate::storage::Storage;
use std::hash;

/// Storage for static types that must be stored in a map.
pub struct MapStorage<K: 'static, V: 'static> {
    inner: hashbrown::HashMap<K, V>,
}

impl<K: 'static, V: 'static> Clone for MapStorage<K, V>
where
    K: Clone,
    V: Clone,
{
    fn clone(&self) -> Self {
        MapStorage {
            inner: self.inner.clone(),
        }
    }
}

impl<K: 'static, V: 'static> Default for MapStorage<K, V>
where
    K: Eq + hash::Hash,
{
    fn default() -> Self {
        Self {
            inner: Default::default(),
        }
    }
}

impl<K: 'static, V: 'static> PartialEq for MapStorage<K, V>
where
    K: Eq + hash::Hash,
    V: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl<K: 'static, V: 'static> Eq for MapStorage<K, V>
where
    K: Eq + hash::Hash,
    V: Eq,
{
}

impl<K, V> Storage<K, V> for MapStorage<K, V>
where
    K: Copy + Eq + hash::Hash,
{
    #[inline]
    fn insert(&mut self, key: K, value: V) -> Option<V> {
        self.inner.insert(key, value)
    }

    #[inline]
    fn get(&self, key: K) -> Option<&V> {
        self.inner.get(&key)
    }

    #[inline]
    fn get_mut(&mut self, key: K) -> Option<&mut V> {
        self.inner.get_mut(&key)
    }

    #[inline]
    fn remove(&mut self, key: K) -> Option<V> {
        self.inner.remove(&key)
    }

    #[inline]
    fn clear(&mut self) {
        self.inner.clear();
    }

    #[inline]
    fn iter<'a, F>(&'a self, mut f: F)
    where
        F: FnMut((K, &'a V)),
    {
        for (key, value) in &self.inner {
            f((*key, value));
        }
    }

    #[inline]
    fn iter_mut<'a, F>(&'a mut self, mut f: F)
    where
        F: FnMut((K, &'a mut V)),
    {
        for (key, value) in &mut self.inner {
            f((*key, value));
        }
    }
}
