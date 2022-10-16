use crate::key::Key;
use crate::storage::Storage;

mod bucket;
mod option_safe;
mod option_unsafe;

pub trait OccupiedEntry<'this, K: Key, V> {
    fn key(&self) -> K;
    fn get(&self) -> &V;
    fn get_mut(&mut self) -> &mut V;
    fn into_mut(self) -> &'this mut V;
    fn insert(&mut self, value: V) -> V;
    fn remove(self) -> V;
}

pub trait VacantEntry<'this, K: Key, V> {
    fn key(&self) -> K;
    fn insert(self, value: V) -> &'this mut V;
}

pub enum Entry<Occupied, Vacant> {
    Occupied(Occupied),
    Vacant(Vacant),
}

impl<Occupied, Vacant> Entry<Occupied, Vacant> {
    pub fn or_insert<'this, K: Key, V>(self, default: V) -> &'this mut V
    where
        Occupied: OccupiedEntry<'this, K, V>,
        Vacant: VacantEntry<'this, K, V>,
    {
        match self {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => entry.insert(default),
        }
    }

    pub fn or_insert_with<'this, K: Key, V, F: FnOnce() -> V>(self, default: F) -> &'this mut V
    where
        Occupied: OccupiedEntry<'this, K, V>,
        Vacant: VacantEntry<'this, K, V>,
    {
        match self {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => entry.insert(default()),
        }
    }

    pub fn or_insert_with_key<'this, K: Key, V, F: FnOnce(K) -> V>(self, default: F) -> &'this mut V
    where
        Occupied: OccupiedEntry<'this, K, V>,
        Vacant: VacantEntry<'this, K, V>,
    {
        match self {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => {
                let value = default(entry.key());
                entry.insert(value)
            }
        }
    }

    pub fn key<'this, K: Key, V>(&self) -> K
    where
        Occupied: OccupiedEntry<'this, K, V>,
        Vacant: VacantEntry<'this, K, V>,
    {
        match self {
            Entry::Occupied(entry) => entry.key(),
            Entry::Vacant(entry) => entry.key(),
        }
    }

    pub fn and_modify<'this, K: Key, V, F: FnOnce(&mut V)>(self, f: F) -> Self
    where
        Occupied: OccupiedEntry<'this, K, V>,
        Vacant: VacantEntry<'this, K, V>,
    {
        match self {
            Entry::Occupied(mut entry) => {
                f(entry.get_mut());
                Entry::Occupied(entry)
            }
            Entry::Vacant(entry) => Entry::Vacant(entry),
        }
    }

    pub fn or_default<'this, K: Key, V>(self) -> &'this mut V
    where
        V: Default,
        Occupied: OccupiedEntry<'this, K, V>,
        Vacant: VacantEntry<'this, K, V>,
    {
        match self {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => entry.insert(Default::default()),
        }
    }
}

trait StorageEntry<K: Key, V>: Storage<K, V> {
    type Occupied<'this>
    where
        Self: 'this;
    type Vacant<'this>
    where
        Self: 'this;

    fn entry<'this>(&'this mut self, key: K) -> Entry<Self::Occupied<'this>, Self::Vacant<'this>>
    where
        Self::Occupied<'this>: OccupiedEntry<'this, K, V>,
        Self::Vacant<'this>: VacantEntry<'this, K, V>;
}

// For experimentation, implement a fake entry API for storage
mod fake {
    use crate::{key::Key, storage::Storage};

    pub struct OccupiedEntry<'this, K: Key, V> {
        key: K,
        table: &'this mut K::Storage<V>,
    }

    impl<'this, K: Key, V> super::OccupiedEntry<'this, K, V> for OccupiedEntry<'this, K, V> {
        fn key(&self) -> K {
            self.key
        }
        fn get(&self) -> &V {
            self.table.get(self.key).unwrap()
        }
        fn get_mut(&mut self) -> &mut V {
            self.table.get_mut(self.key).unwrap()
        }
        fn into_mut(self) -> &'this mut V {
            self.table.get_mut(self.key).unwrap()
        }
        fn insert(&mut self, value: V) -> V {
            self.table.insert(self.key, value).unwrap()
        }
        fn remove(self) -> V {
            self.table.remove(self.key).unwrap()
        }
    }

    pub struct VacantEntry<'this, K: Key, V> {
        key: K,
        table: &'this mut K::Storage<V>,
    }

    impl<'this, K: Key, V> super::VacantEntry<'this, K, V> for VacantEntry<'this, K, V> {
        fn key(&self) -> K {
            self.key
        }
        fn insert(self, value: V) -> &'this mut V {
            self.table.insert(self.key, value);
            self.table.get_mut(self.key).unwrap()
        }
    }

    impl<K: Key, V> super::StorageEntry<K, V> for K::Storage<V> {
        type Occupied<'this> = OccupiedEntry<'this, K, V> where Self: 'this;
        type Vacant<'this> = VacantEntry<'this, K, V> where Self: 'this;

        fn entry<'this>(
            &'this mut self,
            key: K,
        ) -> super::Entry<Self::Occupied<'this>, Self::Vacant<'this>>
        where
            Self::Occupied<'this>: super::OccupiedEntry<'this, K, V>,
            Self::Vacant<'this>: super::VacantEntry<'this, K, V>,
        {
            if self.contains_key(key) {
                super::Entry::Occupied(OccupiedEntry { key, table: self })
            } else {
                super::Entry::Vacant(VacantEntry { key, table: self })
            }
        }
    }
}
