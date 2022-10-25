#![allow(clippy::match_bool)]

use crate::option_bucket::{NoneBucket, OptionBucket, SomeBucket};
use crate::storage::entry;
use crate::storage::BooleanStorage;

pub struct VacantEntry<'a, V> {
    key: bool,
    inner: NoneBucket<'a, V>,
}

pub struct OccupiedEntry<'a, V> {
    key: bool,
    inner: SomeBucket<'a, V>,
}

impl<'a, V> entry::VacantEntry<'a, bool, V> for VacantEntry<'a, V> {
    #[inline]
    fn key(&self) -> bool {
        self.key
    }

    #[inline]
    fn insert(self, value: V) -> &'a mut V {
        self.inner.insert(value)
    }
}

impl<'a, V> entry::OccupiedEntry<'a, bool, V> for OccupiedEntry<'a, V> {
    #[inline]
    fn key(&self) -> bool {
        self.key
    }

    #[inline]
    fn get(&self) -> &V {
        self.inner.as_ref()
    }

    #[inline]
    fn get_mut(&mut self) -> &mut V {
        self.inner.as_mut()
    }

    #[inline]
    fn into_mut(self) -> &'a mut V {
        self.inner.into_mut()
    }

    #[inline]
    fn insert(&mut self, value: V) -> V {
        self.inner.replace(value)
    }

    #[inline]
    fn remove(self) -> V {
        self.inner.take()
    }
}

impl<V> entry::StorageEntry<bool, V> for BooleanStorage<V> {
    type Occupied<'this> = OccupiedEntry<'this, V> where V: 'this;
    type Vacant<'this> = VacantEntry<'this, V> where V: 'this;

    #[inline]
    fn entry(&mut self, key: bool) -> entry::Entry<'_, Self, bool, V> {
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
