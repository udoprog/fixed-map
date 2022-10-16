use core::{marker::PhantomData, mem};

use super::bucket::{NoneBucket, OptionBucket, SomeBucket};
use crate::{key::Key, storage::OptionStorage};

struct VacantEntryNone<'a, K: Key, V> {
    _key: PhantomData<K>,
    none: NoneBucket<'a, V>,
}

struct OccupiedEntryNone<'a, K: Key, V> {
    _key: PhantomData<K>,
    none: SomeBucket<'a, V>,
}

enum VacantEntryEither<'a, K: Key, V>
where
    K::Storage<V>: 'a,
{
    None(VacantEntryNone<'a, K, V>),
    Some(<K::Storage<V> as super::StorageEntry<K, V>>::Vacant<'a>),
}

pub struct VacantEntry<'a, K: Key, V> {
    either: VacantEntryEither<'a, K, V>,
}

enum OccupiedEntryEither<'a, K: Key, V>
where
    K::Storage<V>: 'a,
{
    None(OccupiedEntryNone<'a, K, V>),
    Some(<K::Storage<V> as super::StorageEntry<K, V>>::Occupied<'a>),
}

pub struct OccupiedEntry<'a, K: Key, V> {
    either: OccupiedEntryEither<'a, K, V>,
}

impl<'a, K: Key, V> VacantEntryNone<'a, K, V> {
    pub fn key(&self) -> Option<K> {
        None
    }

    pub fn insert(self, value: V) -> &'a mut V {
        self.none.insert(value)
    }
}

impl<'a, K: Key, V> super::VacantEntry<'a, Option<K>, V> for VacantEntry<'a, K, V> {
    fn key(&self) -> Option<K> {
        match &self.either {
            VacantEntryEither::None(entry) => entry.key(),
            VacantEntryEither::Some(entry) => Some(entry.key()),
        }
    }

    fn insert(self, value: V) -> &'a mut V {
        match self.either {
            VacantEntryEither::None(entry) => entry.insert(value),
            VacantEntryEither::Some(entry) => entry.insert(value),
        }
    }
}

impl<'a, K: Key, V> OccupiedEntryNone<'a, K, V> {
    pub fn key(&self) -> Option<K> {
        None
    }

    pub fn get(&self) -> &V {
        self.none.as_ref()
    }

    pub fn get_mut(&mut self) -> &mut V {
        self.none.as_mut()
    }

    pub fn into_mut(self) -> &'a mut V {
        self.none.into_mut()
    }

    pub fn insert(&mut self, value: V) -> V {
        mem::replace(self.none.as_mut(), value)
    }

    pub fn remove(self) -> V {
        self.none.take()
    }
}

impl<'a, K: Key, V> super::OccupiedEntry<'a, Option<K>, V> for OccupiedEntry<'a, K, V> {
    fn key(&self) -> Option<K> {
        match &self.either {
            OccupiedEntryEither::None(entry) => entry.key(),
            OccupiedEntryEither::Some(entry) => Some(entry.key()),
        }
    }

    fn get(&self) -> &V {
        match &self.either {
            OccupiedEntryEither::None(entry) => entry.get(),
            OccupiedEntryEither::Some(entry) => entry.get(),
        }
    }

    fn get_mut(&mut self) -> &mut V {
        match &mut self.either {
            OccupiedEntryEither::None(entry) => entry.get_mut(),
            OccupiedEntryEither::Some(entry) => entry.get_mut(),
        }
    }

    fn into_mut(self) -> &'a mut V {
        match self.either {
            OccupiedEntryEither::None(entry) => entry.into_mut(),
            OccupiedEntryEither::Some(entry) => entry.into_mut(),
        }
    }

    fn insert(&mut self, value: V) -> V {
        match &mut self.either {
            OccupiedEntryEither::None(entry) => entry.insert(value),
            OccupiedEntryEither::Some(entry) => entry.insert(value),
        }
    }

    fn remove(self) -> V {
        match self.either {
            OccupiedEntryEither::None(entry) => entry.remove(),
            OccupiedEntryEither::Some(entry) => entry.remove(),
        }
    }
}

pub type Entry<'a, K, V> = super::Entry<OccupiedEntry<'a, K, V>, VacantEntry<'a, K, V>>;

use crate::storage::entry::StorageEntry;

impl<K: Key, V> OptionStorage<K, V> {
    pub fn entry<'this>(
        self: &'this mut OptionStorage<K, V>,
        key: Option<K>,
    ) -> Entry<'this, K, V> {
        match key {
            Some(key) => match self.some.entry(key) {
                super::Entry::Occupied(entry) => Entry::Occupied(OccupiedEntry {
                    either: OccupiedEntryEither::Some(entry),
                }),
                super::Entry::Vacant(entry) => Entry::Vacant(VacantEntry {
                    either: VacantEntryEither::Some(entry),
                }),
            },
            None => match OptionBucket::new(&mut self.none) {
                OptionBucket::Some(some) => Entry::Occupied(OccupiedEntry {
                    either: OccupiedEntryEither::None(OccupiedEntryNone {
                        _key: PhantomData,
                        none: some,
                    }),
                }),
                OptionBucket::None(none) => Entry::Vacant(VacantEntry {
                    either: VacantEntryEither::None(VacantEntryNone {
                        _key: PhantomData,
                        none,
                    }),
                }),
            },
        }
    }
}
