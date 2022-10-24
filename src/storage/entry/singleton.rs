use crate::storage::entry;
use crate::storage::SingletonStorage;
use option_bucket::{NoneBucket, OptionBucket, SomeBucket};

pub struct VacantEntry<'this, V> {
    inner: NoneBucket<'this, V>,
}

pub struct OccupiedEntry<'this, V> {
    inner: SomeBucket<'this, V>,
}

impl<'this, K: Copy + Default, V> entry::VacantEntry<'this, K, V> for VacantEntry<'this, V> {
    fn key(&self) -> K {
        K::default()
    }

    fn insert(self, value: V) -> &'this mut V {
        self.inner.insert(value)
    }
}

impl<'this, K: Copy + Default, V> entry::OccupiedEntry<'this, K, V> for OccupiedEntry<'this, V> {
    fn key(&self) -> K {
        K::default()
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

impl<K: Copy + Default, V> entry::StorageEntry<K, V> for SingletonStorage<V> {
    type Occupied<'this> = OccupiedEntry<'this, V> where V: 'this;
    type Vacant<'this> = VacantEntry<'this, V> where V: 'this;

    #[inline]
    fn entry(&mut self, _key: K) -> entry::Entry<Self::Occupied<'_>, Self::Vacant<'_>> {
        match OptionBucket::new(&mut self.inner) {
            OptionBucket::Some(inner) => entry::Entry::Occupied(OccupiedEntry { inner }),
            OptionBucket::None(inner) => entry::Entry::Vacant(VacantEntry { inner }),
        }
    }
}
