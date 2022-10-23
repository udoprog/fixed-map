use core::hash::Hash;

use crate::storage::entry;
use crate::storage::MapStorage;

use ::hashbrown::hash_map::DefaultHashBuilder as S;
type OccupiedEntry<'this, K, V> = ::hashbrown::hash_map::OccupiedEntry<'this, K, V, S>;
type VacantEntry<'this, K, V> = ::hashbrown::hash_map::VacantEntry<'this, K, V, S>;
type HEntry<'this, K, V> = ::hashbrown::hash_map::Entry<'this, K, V, S>;

impl<'this, K: Copy + Eq + Hash, V> entry::OccupiedEntry<'this, K, V>
    for OccupiedEntry<'this, K, V>
{
    fn key(&self) -> K {
        *self.key()
    }

    fn get(&self) -> &V {
        self.get()
    }

    fn get_mut(&mut self) -> &mut V {
        self.get_mut()
    }

    fn into_mut(self) -> &'this mut V {
        self.into_mut()
    }

    fn insert(&mut self, value: V) -> V {
        self.insert(value)
    }

    fn remove(self) -> V {
        self.remove()
    }
}

impl<'this, K: Copy + Eq + Hash, V> entry::VacantEntry<'this, K, V> for VacantEntry<'this, K, V> {
    fn key(&self) -> K {
        *self.key()
    }

    fn insert(self, value: V) -> &'this mut V {
        self.insert(value)
    }
}

impl<'this, K: Copy + Eq + Hash, V> entry::StorageEntry<'this, K, V> for MapStorage<K, V>
where
    Self: 'this,
{
    type Occupied = OccupiedEntry<'this, K, V>;
    type Vacant = VacantEntry<'this, K, V>;

    #[inline]
    fn entry(&'this mut self, key: K) -> entry::Entry<Self::Occupied, Self::Vacant> {
        match self.inner.entry(key) {
            HEntry::Occupied(entry) => entry::Entry::Occupied(entry),
            HEntry::Vacant(entry) => entry::Entry::Vacant(entry),
        }
    }
}