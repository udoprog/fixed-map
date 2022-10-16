use core::marker::PhantomData;

use crate::storage::entry;
use crate::{key::Key, storage::OptionStorage};
use entry::bucket::{NoneBucket, OptionBucket, SomeBucket};

struct VacantEntryNone<'this, K: Key, V> {
    _key: PhantomData<K>,
    none: NoneBucket<'this, V>,
}

struct OccupiedEntryNone<'this, K: Key, V> {
    _key: PhantomData<K>,
    none: SomeBucket<'this, V>,
}

enum VacantEntryEither<'this, K, V>
where
    K: Key,
    K::Storage<V>: 'this + entry::StorageEntry<K, V>,
{
    None(VacantEntryNone<'this, K, V>),
    Some(<K::Storage<V> as entry::StorageEntry<K, V>>::Vacant<'this>),
}

pub struct VacantEntry<'this, K, V>
where
    K: Key,
    K::Storage<V>: 'this + entry::StorageEntry<K, V>,
{
    either: VacantEntryEither<'this, K, V>,
}

enum OccupiedEntryEither<'this, K, V>
where
    K: Key,
    K::Storage<V>: 'this + entry::StorageEntry<K, V>,
{
    None(OccupiedEntryNone<'this, K, V>),
    Some(<K::Storage<V> as entry::StorageEntry<K, V>>::Occupied<'this>),
}

pub struct OccupiedEntry<'this, K, V>
where
    K: Key,
    K::Storage<V>: 'this + entry::StorageEntry<K, V>,
{
    either: OccupiedEntryEither<'this, K, V>,
}

impl<'this, K: Key, V> VacantEntryNone<'this, K, V> {
    #[allow(clippy::unused_self)]
    fn key(&self) -> Option<K> {
        None
    }

    fn insert(self, value: V) -> &'this mut V {
        self.none.insert(value)
    }
}

impl<'this, K, V> entry::VacantEntry<'this, Option<K>, V> for VacantEntry<'this, K, V>
where
    K: Key,
    K::Storage<V>: entry::StorageEntry<K, V>,
    <K::Storage<V> as entry::StorageEntry<K, V>>::Vacant<'this>: entry::VacantEntry<'this, K, V>,
    <K::Storage<V> as entry::StorageEntry<K, V>>::Occupied<'this>:
        entry::OccupiedEntry<'this, K, V>,
{
    fn key(&self) -> Option<K> {
        match &self.either {
            VacantEntryEither::None(entry) => entry.key(),
            VacantEntryEither::Some(entry) => Some(entry.key()),
        }
    }

    fn insert(self, value: V) -> &'this mut V {
        match self.either {
            VacantEntryEither::None(entry) => entry.insert(value),
            VacantEntryEither::Some(entry) => entry.insert(value),
        }
    }
}

impl<'this, K: Key, V> OccupiedEntryNone<'this, K, V> {
    #[allow(clippy::unused_self)]
    fn key(&self) -> Option<K> {
        None
    }

    fn get(&self) -> &V {
        self.none.as_ref()
    }

    fn get_mut(&mut self) -> &mut V {
        self.none.as_mut()
    }

    fn into_mut(self) -> &'this mut V {
        self.none.into_mut()
    }

    fn insert(&mut self, value: V) -> V {
        self.none.replace(value)
    }

    fn remove(self) -> V {
        self.none.take()
    }
}

impl<'this, K, V> entry::OccupiedEntry<'this, Option<K>, V> for OccupiedEntry<'this, K, V>
where
    K: Key,
    K::Storage<V>: entry::StorageEntry<K, V>,
    <K::Storage<V> as entry::StorageEntry<K, V>>::Vacant<'this>: entry::VacantEntry<'this, K, V>,
    <K::Storage<V> as entry::StorageEntry<K, V>>::Occupied<'this>:
        entry::OccupiedEntry<'this, K, V>,
{
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

    fn into_mut(self) -> &'this mut V {
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

impl<K, V> entry::StorageEntry<Option<K>, V> for OptionStorage<K, V>
where
    K: Key,
    K::Storage<V>: entry::StorageEntry<K, V>,
    for<'this> <K::Storage<V> as entry::StorageEntry<K, V>>::Vacant<'this>:
        entry::VacantEntry<'this, K, V>,
    for<'this> <K::Storage<V> as entry::StorageEntry<K, V>>::Occupied<'this>:
        entry::OccupiedEntry<'this, K, V>,
{
    type Occupied<'this> = OccupiedEntry<'this, K, V> where Self: 'this;
    type Vacant<'this> = VacantEntry<'this, K, V> where Self: 'this;

    #[inline]
    fn entry<'this>(
        &'this mut self,
        key: Option<K>,
    ) -> entry::Entry<Self::Occupied<'this>, Self::Vacant<'this>>
    where
        Self::Occupied<'this>: entry::OccupiedEntry<'this, Option<K>, V>,
        Self::Vacant<'this>: entry::VacantEntry<'this, Option<K>, V>,
    {
        match key {
            Some(key) => match self.some.entry(key) {
                entry::Entry::Occupied(entry) => entry::Entry::Occupied(OccupiedEntry {
                    either: OccupiedEntryEither::Some(entry),
                }),
                entry::Entry::Vacant(entry) => entry::Entry::Vacant(VacantEntry {
                    either: VacantEntryEither::Some(entry),
                }),
            },
            None => match OptionBucket::new(&mut self.none) {
                OptionBucket::Some(some) => entry::Entry::Occupied(OccupiedEntry {
                    either: OccupiedEntryEither::None(OccupiedEntryNone {
                        _key: PhantomData,
                        none: some,
                    }),
                }),
                OptionBucket::None(none) => entry::Entry::Vacant(VacantEntry {
                    either: VacantEntryEither::None(VacantEntryNone {
                        _key: PhantomData,
                        none,
                    }),
                }),
            },
        }
    }
}
