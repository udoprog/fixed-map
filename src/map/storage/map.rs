use core::hash::Hash;
use core::iter;

use crate::map::{Entry, OccupiedEntry, Storage, VacantEntry};

type S = ::hashbrown::hash_map::DefaultHashBuilder;
type Occupied<'a, K, V> = ::hashbrown::hash_map::OccupiedEntry<'a, K, V, S>;
type Vacant<'a, K, V> = ::hashbrown::hash_map::VacantEntry<'a, K, V, S>;
type HashMapEntry<'a, K, V> = ::hashbrown::hash_map::Entry<'a, K, V, S>;

/// [`Storage`] for dynamic types, using [`hashbrown::HashMap`].
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

impl<K, V> PartialEq for MapStorage<K, V>
where
    K: Eq + Hash,
    V: PartialEq,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.inner.eq(&other.inner)
    }
}

impl<K, V> Eq for MapStorage<K, V>
where
    K: Eq + Hash,
    V: Eq,
{
}

impl<'a, K, V> OccupiedEntry<'a, K, V> for Occupied<'a, K, V>
where
    K: Copy,
{
    #[inline]
    fn key(&self) -> K {
        *self.key()
    }

    #[inline]
    fn get(&self) -> &V {
        self.get()
    }

    #[inline]
    fn get_mut(&mut self) -> &mut V {
        self.get_mut()
    }

    #[inline]
    fn into_mut(self) -> &'a mut V {
        self.into_mut()
    }

    #[inline]
    fn insert(&mut self, value: V) -> V {
        self.insert(value)
    }

    #[inline]
    fn remove(self) -> V {
        self.remove()
    }
}

impl<'a, K, V> VacantEntry<'a, K, V> for Vacant<'a, K, V>
where
    K: Copy + Hash,
{
    #[inline]
    fn key(&self) -> K {
        *self.key()
    }

    #[inline]
    fn insert(self, value: V) -> &'a mut V {
        self.insert(value)
    }
}

impl<K, V> Storage<K, V> for MapStorage<K, V>
where
    K: Copy + Eq + Hash,
{
    type Iter<'this> = iter::Map<::hashbrown::hash_map::Iter<'this, K, V>, fn((&'this K, &'this V)) -> (K, &'this V)> where K: 'this, V: 'this;
    type Keys<'this> = iter::Copied<::hashbrown::hash_map::Keys<'this, K, V>> where K: 'this, V: 'this;
    type Values<'this> = ::hashbrown::hash_map::Values<'this, K, V> where K: 'this, V: 'this;
    type IterMut<'this> = iter::Map<::hashbrown::hash_map::IterMut<'this, K, V>, fn((&'this K, &'this mut V)) -> (K, &'this mut V)> where K: 'this, V: 'this;
    type ValuesMut<'this> = ::hashbrown::hash_map::ValuesMut<'this, K, V> where K: 'this, V: 'this;
    type IntoIter = ::hashbrown::hash_map::IntoIter<K, V>;
    type Occupied<'this> = Occupied<'this, K, V> where K: 'this, V: 'this;
    type Vacant<'this> = Vacant<'this, K, V> where K: 'this, V: 'this;

    #[inline]
    fn empty() -> Self {
        Self {
            inner: ::hashbrown::HashMap::new(),
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

    #[inline]
    fn entry(&mut self, key: K) -> Entry<'_, Self, K, V> {
        match self.inner.entry(key) {
            HashMapEntry::Occupied(entry) => Entry::Occupied(entry),
            HashMapEntry::Vacant(entry) => Entry::Vacant(entry),
        }
    }
}
