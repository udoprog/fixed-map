#![allow(clippy::match_bool)]

use crate::storage::entry;
use crate::storage::BooleanStorage;
use entry::bucket::{NoneBucket, OptionBucket, SomeBucket};

pub struct VacantEntry<'this, V> {
    key: bool,
    inner: NoneBucket<'this, V>,
}

pub struct OccupiedEntry<'this, V> {
    key: bool,
    inner: SomeBucket<'this, V>,
}

impl<'this, V> entry::VacantEntry<'this, bool, V> for VacantEntry<'this, V> {
    fn key(&self) -> bool {
        self.key
    }

    fn insert(self, value: V) -> &'this mut V {
        self.inner.insert(value)
    }
}

impl<'this, V> entry::OccupiedEntry<'this, bool, V> for OccupiedEntry<'this, V> {
    fn key(&self) -> bool {
        self.key
    }

    fn get(&self) -> &V {
        self.inner.as_ref()
    }

    fn get_mut(&mut self) -> &mut V {
        self.inner.as_mut()
    }

    fn into_mut(self) -> &'this mut V {
        self.inner.into_mut()
    }

    fn insert(&mut self, value: V) -> V {
        self.inner.replace(value)
    }

    fn remove(self) -> V {
        self.inner.take()
    }
}

impl<V> entry::StorageEntry<bool, V> for BooleanStorage<V> {
    type Occupied<'this> = OccupiedEntry<'this, V> where Self: 'this;
    type Vacant<'this> = VacantEntry<'this, V> where Self: 'this;

    #[inline]
    fn entry<'this>(
        &'this mut self,
        key: bool,
    ) -> entry::Entry<Self::Occupied<'this>, Self::Vacant<'this>>
    where
        Self::Occupied<'this>: entry::OccupiedEntry<'this, bool, V>,
        Self::Vacant<'this>: entry::VacantEntry<'this, bool, V>,
    {
        match key {
            true => match OptionBucket::new(&mut self.t) {
                OptionBucket::Some(inner) => entry::Entry::Occupied(OccupiedEntry { key, inner }),
                OptionBucket::None(inner) => entry::Entry::Vacant(VacantEntry { key, inner }),
            },
            false => match OptionBucket::new(&mut self.f) {
                OptionBucket::Some(inner) => entry::Entry::Occupied(OccupiedEntry { key, inner }),
                OptionBucket::None(inner) => entry::Entry::Vacant(VacantEntry { key, inner }),
            },
        }
    }
}
