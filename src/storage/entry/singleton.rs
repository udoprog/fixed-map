use crate::option_bucket::{NoneBucket, OptionBucket, SomeBucket};
use crate::storage::entry;
use crate::storage::SingletonStorage;

pub struct VacantEntry<'a, V> {
    inner: NoneBucket<'a, V>,
}

pub struct OccupiedEntry<'a, V> {
    inner: SomeBucket<'a, V>,
}

impl<'a, K, V> entry::VacantEntry<'a, K, V> for VacantEntry<'a, V>
where
    K: Copy + Default,
{
    #[inline]
    fn key(&self) -> K {
        K::default()
    }

    #[inline]
    fn insert(self, value: V) -> &'a mut V {
        self.inner.insert(value)
    }
}

impl<'a, K: Copy + Default, V> entry::OccupiedEntry<'a, K, V> for OccupiedEntry<'a, V> {
    #[inline]
    fn key(&self) -> K {
        K::default()
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

impl<K, V> entry::StorageEntry<K, V> for SingletonStorage<V>
where
    K: Copy + Default,
{
    type Occupied<'this> = OccupiedEntry<'this, V> where V: 'this;
    type Vacant<'this> = VacantEntry<'this, V> where V: 'this;

    #[inline]
    fn entry(&mut self, _key: K) -> entry::Entry<'_, Self, K, V> {
        match OptionBucket::new(&mut self.inner) {
            OptionBucket::Some(inner) => entry::Entry::Occupied(OccupiedEntry { inner }),
            OptionBucket::None(inner) => entry::Entry::Vacant(VacantEntry { inner }),
        }
    }
}
