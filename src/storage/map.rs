use core::hash;
use core::iter;

use crate::storage::Storage;

/// Storage for dynamic types, using [`hashbrown::HashMap`].
///
/// This allows for dynamic types such as `&'static str` or `u32` to be used as
/// a [`Key`][crate::Key].
///
/// # Examples
///
/// ```
/// use fixed_map::{Key, Map};
///
/// #[derive(Clone, Copy, Key)]
/// enum Key {
///     First(u32),
///     Second,
/// }
///
/// let mut map = Map::new();
/// map.insert(Key::First(1), 10);
/// assert_eq!(map.get(Key::First(1)).copied(), Some(10));
/// assert_eq!(map.get(Key::First(2)), None);
/// assert_eq!(map.get(Key::Second), None);
/// ```
#[repr(transparent)]
pub struct MapStorage<K, V> {
    pub(in crate::storage) inner: ::hashbrown::HashMap<K, V>,
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

impl<K, V> Storage<K, V> for MapStorage<K, V>
where
    K: Copy + Eq + hash::Hash,
{
    type Iter<'this> = iter::Map<::hashbrown::hash_map::Iter<'this, K, V>, fn((&'this K, &'this V)) -> (K, &'this V)> where Self: 'this, V: 'this;
    type Keys<'this> = iter::Copied<::hashbrown::hash_map::Keys<'this, K, V>> where Self: 'this;
    type Values<'this> = ::hashbrown::hash_map::Values<'this, K, V> where Self: 'this;
    type IterMut<'this> = iter::Map<::hashbrown::hash_map::IterMut<'this, K, V>, fn((&'this K, &'this mut V)) -> (K, &'this mut V)> where Self: 'this, V: 'this;
    type ValuesMut<'this> = ::hashbrown::hash_map::ValuesMut<'this, K, V> where Self: 'this;
    type IntoIter = ::hashbrown::hash_map::IntoIter<K, V>;

    #[inline]
    fn len(&self) -> usize {
        self.inner.len()
    }

    #[inline]
    fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    #[inline]
    fn insert(&mut self, key: K, value: V) -> Option<V> {
        self.inner.insert(key, value)
    }

    #[inline]
    fn contains_key(&self, key: K) -> bool {
        self.inner.contains_key(&key)
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
    fn retain<F>(&mut self, mut func: F)
    where
        F: FnMut(K, &mut V) -> bool,
    {
        self.inner.retain(|&k, v| func(k, v));
    }

    #[inline]
    fn clear(&mut self) {
        self.inner.clear();
    }

    #[inline]
    fn iter(&self) -> Self::Iter<'_> {
        let map: fn(_) -> _ = |(k, v): (&K, &V)| (*k, v);
        self.inner.iter().map(map)
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
        let map: fn(_) -> _ = |(k, v): (&K, &mut V)| (*k, v);
        self.inner.iter_mut().map(map)
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
