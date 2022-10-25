use crate::option_bucket::{NoneBucket, OptionBucket, SomeBucket};
use crate::storage::entry;
use crate::{key::Key, storage::OptionStorage};

struct VacantEntryNone<'a, V> {
    none: NoneBucket<'a, V>,
}

struct OccupiedEntryNone<'a, V> {
    none: SomeBucket<'a, V>,
}

enum VacantEntryEither<'a, K: 'a, V>
where
    K: Key,
    K::Storage<V>: entry::StorageEntry<K, V>,
{
    None(VacantEntryNone<'a, V>),
    Some(<K::Storage<V> as entry::StorageEntry<K, V>>::Vacant<'a>),
}

pub struct VacantEntry<'a, K, V>
where
    K: Key,
    K::Storage<V>: entry::StorageEntry<K, V>,
{
    either: VacantEntryEither<'a, K, V>,
}

enum OccupiedEntryEither<'a, K: 'a, V>
where
    K: Key,
    K::Storage<V>: entry::StorageEntry<K, V>,
{
    None(OccupiedEntryNone<'a, V>),
    Some(<K::Storage<V> as entry::StorageEntry<K, V>>::Occupied<'a>),
}

pub struct OccupiedEntry<'a, K, V>
where
    K: Key,
    K::Storage<V>: entry::StorageEntry<K, V>,
{
    either: OccupiedEntryEither<'a, K, V>,
}

impl<'a, V> VacantEntryNone<'a, V> {
    #[inline]
    fn insert(self, value: V) -> &'a mut V {
        self.none.insert(value)
    }
}

impl<'a, K, V> entry::VacantEntry<'a, Option<K>, V> for VacantEntry<'a, K, V>
where
    K: Key,
    K::Storage<V>: entry::StorageEntry<K, V>,
{
    #[inline]
    fn key(&self) -> Option<K> {
        match &self.either {
            VacantEntryEither::None(_) => None,
            VacantEntryEither::Some(entry) => Some(entry.key()),
        }
    }

    #[inline]
    fn insert(self, value: V) -> &'a mut V {
        match self.either {
            VacantEntryEither::None(entry) => entry.insert(value),
            VacantEntryEither::Some(entry) => entry.insert(value),
        }
    }
}

impl<'a, V> OccupiedEntryNone<'a, V> {
    #[inline]
    fn get(&self) -> &V {
        self.none.as_ref()
    }

    #[inline]
    fn get_mut(&mut self) -> &mut V {
        self.none.as_mut()
    }

    #[inline]
    fn into_mut(self) -> &'a mut V {
        self.none.into_mut()
    }

    #[inline]
    fn insert(&mut self, value: V) -> V {
        self.none.replace(value)
    }

    #[inline]
    fn remove(self) -> V {
        self.none.take()
    }
}

impl<'a, K, V> entry::OccupiedEntry<'a, Option<K>, V> for OccupiedEntry<'a, K, V>
where
    K: Key,
    K::Storage<V>: entry::StorageEntry<K, V>,
{
    #[inline]
    fn key(&self) -> Option<K> {
        match &self.either {
            OccupiedEntryEither::None(_) => None,
            OccupiedEntryEither::Some(entry) => Some(entry.key()),
        }
    }

    #[inline]
    fn get(&self) -> &V {
        match &self.either {
            OccupiedEntryEither::None(entry) => entry.get(),
            OccupiedEntryEither::Some(entry) => entry.get(),
        }
    }

    #[inline]
    fn get_mut(&mut self) -> &mut V {
        match &mut self.either {
            OccupiedEntryEither::None(entry) => entry.get_mut(),
            OccupiedEntryEither::Some(entry) => entry.get_mut(),
        }
    }

    #[inline]
    fn into_mut(self) -> &'a mut V {
        match self.either {
            OccupiedEntryEither::None(entry) => entry.into_mut(),
            OccupiedEntryEither::Some(entry) => entry.into_mut(),
        }
    }

    #[inline]
    fn insert(&mut self, value: V) -> V {
        match &mut self.either {
            OccupiedEntryEither::None(entry) => entry.insert(value),
            OccupiedEntryEither::Some(entry) => entry.insert(value),
        }
    }

    #[inline]
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
{
    type Occupied<'this> = OccupiedEntry<'this, K, V> where K: 'this, V: 'this;
    type Vacant<'this> = VacantEntry<'this, K, V> where K: 'this, V: 'this;

    #[inline]
    fn entry(&mut self, key: Option<K>) -> entry::Entry<'_, Self, Option<K>, V> {
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
                    either: OccupiedEntryEither::None(OccupiedEntryNone { none: some }),
                }),
                OptionBucket::None(none) => entry::Entry::Vacant(VacantEntry {
                    either: VacantEntryEither::None(VacantEntryNone { none }),
                }),
            },
        }
    }
}
