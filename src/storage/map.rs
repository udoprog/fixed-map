use core::hash;
use core::iter;

use crate::storage::Storage;

/// Storage for static types that must be stored in a map.
#[repr(transparent)]
pub struct MapStorage<K, V> {
    inner: ::hashbrown::HashMap<K, V>,
}

impl<K, V> Clone for MapStorage<K, V>
where
    K: Clone,
    V: Clone,
{
    #[inline]
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
    #[inline]
    fn default() -> Self {
        Self {
            inner: ::hashbrown::HashMap::default(),
        }
    }
}

impl<K, V> PartialEq for MapStorage<K, V>
where
    K: Eq + hash::Hash,
    V: PartialEq,
{
    #[inline]
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

pub struct Iter<'a, K, V> {
    iter: hashbrown::hash_map::Iter<'a, K, V>,
}

impl<'a, K, V> Clone for Iter<'a, K, V>
where
    K: 'a + Copy,
{
    #[inline]
    fn clone(&self) -> Self {
        Iter {
            iter: self.iter.clone(),
        }
    }
}

impl<'a, K, V> Iterator for Iter<'a, K, V>
where
    K: Copy,
{
    type Item = (K, &'a V);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|(k, v)| (*k, v))
    }
}

pub struct IterMut<'a, K, V> {
    iter: hashbrown::hash_map::IterMut<'a, K, V>,
}

impl<'a, K, V> Iterator for IterMut<'a, K, V>
where
    K: Copy,
{
    type Item = (K, &'a mut V);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|(k, v)| (*k, v))
    }
}

impl<K, V> Storage<K, V> for MapStorage<K, V>
where
    K: Copy + Eq + hash::Hash,
{
    type Iter<'this> = Iter<'this, K, V> where Self: 'this, V: 'this;
    type Keys<'this> = iter::Copied<::hashbrown::hash_map::Keys<'this, K, V>> where Self: 'this;
    type Values<'this> = ::hashbrown::hash_map::Values<'this, K, V> where Self: 'this;
    type IterMut<'this> = IterMut<'this, K, V> where Self: 'this, V: 'this;
    type ValuesMut<'this> = ::hashbrown::hash_map::ValuesMut<'this, K, V> where Self: 'this;
    type IntoIter = hashbrown::hash_map::IntoIter<K, V>;

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
    fn iter(&self) -> Self::Iter<'_> {
        Iter {
            iter: self.inner.iter(),
        }
    }

    #[inline]
    fn keys(&self) -> Self::Keys<'_> {
        self.inner.keys().copied()
    }

    #[inline]
    fn values(&self) -> Self::Values<'_> {
        self.inner.values()
    }

    #[inline]
    fn iter_mut(&mut self) -> Self::IterMut<'_> {
        IterMut {
            iter: self.inner.iter_mut(),
        }
    }

    #[inline]
    fn values_mut(&mut self) -> Self::ValuesMut<'_> {
        self.inner.values_mut()
    }

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}
