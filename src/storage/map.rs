use crate::storage::Storage;
use std::hash;

/// Storage for static types that must be stored in a map.
pub struct MapStorage<K, V> {
    inner: hashbrown::HashMap<K, V>,
}

impl<K, V> Clone for MapStorage<K, V>
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

impl<K, V> Default for MapStorage<K, V>
where
    K: Eq + hash::Hash,
{
    fn default() -> Self {
        Self {
            inner: Default::default(),
        }
    }
}

impl<K, V> PartialEq for MapStorage<K, V>
where
    K: Eq + hash::Hash,
    V: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl<K, V> Eq for MapStorage<K, V>
where
    K: Eq + hash::Hash,
    V: Eq,
{
}

pub struct Iter<K, V> {
    iter: std::vec::IntoIter<(K, *const V)>,
}

impl<K, V> Clone for Iter<K, V>
where
    K: Copy,
{
    fn clone(&self) -> Iter<K, V> {
        Iter {
            iter: self.iter.clone(),
        }
    }
}

impl<K, V> Iterator for Iter<K, V> {
    type Item = (K, *const V);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

pub struct IterMut<K, V> {
    iter: std::vec::IntoIter<(K, *mut V)>,
}

impl<K, V> Iterator for IterMut<K, V> {
    type Item = (K, *mut V);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

impl<K, V> Storage<K, V> for MapStorage<K, V>
where
    K: Copy + Eq + hash::Hash,
{
    type Iter = Iter<K, V>;
    type IterMut = IterMut<K, V>;

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
    fn iter(&self) -> Self::Iter {
        Iter {
            iter: self
                .inner
                .iter()
                .map(|(k, v)| (*k, v as *const V))
                .collect::<Vec<_>>()
                .into_iter(),
        }
    }

    #[inline]
    fn iter_mut(&mut self) -> Self::IterMut {
        IterMut {
            iter: self
                .inner
                .iter_mut()
                .map(|(k, v)| (*k, v as *mut V))
                .collect::<Vec<_>>()
                .into_iter(),
        }
    }
}
