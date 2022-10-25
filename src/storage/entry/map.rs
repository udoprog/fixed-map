use core::hash::Hash;

use crate::storage::entry;
use crate::storage::MapStorage;

type S = ::hashbrown::hash_map::DefaultHashBuilder;
type OccupiedEntry<'a, K, V> = ::hashbrown::hash_map::OccupiedEntry<'a, K, V, S>;
type VacantEntry<'a, K, V> = ::hashbrown::hash_map::VacantEntry<'a, K, V, S>;
type HEntry<'a, K, V> = ::hashbrown::hash_map::Entry<'a, K, V, S>;

impl<'a, K, V> entry::OccupiedEntry<'a, K, V> for OccupiedEntry<'a, K, V>
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

impl<'this, K, V> entry::VacantEntry<'this, K, V> for VacantEntry<'this, K, V>
where
    K: Copy + Hash,
{
    #[inline]
    fn key(&self) -> K {
        *self.key()
    }

    #[inline]
    fn insert(self, value: V) -> &'this mut V {
        self.insert(value)
    }
}

impl<K, V> entry::StorageEntry<K, V> for MapStorage<K, V>
where
    K: Copy + Eq + Hash,
{
    type Occupied<'this> = OccupiedEntry<'this, K, V> where K: 'this, V: 'this;
    type Vacant<'this> = VacantEntry<'this, K, V> where K: 'this, V: 'this;

    #[inline]
    fn entry(&mut self, key: K) -> entry::Entry<'_, Self, K, V> {
        match self.inner.entry(key) {
            HEntry::Occupied(entry) => entry::Entry::Occupied(entry),
            HEntry::Vacant(entry) => entry::Entry::Vacant(entry),
        }
    }
}
