//! Module for the trait to define a `Key`.

#[cfg(feature = "map")]
use crate::storage::MapStorage;
use crate::storage::{BooleanStorage, OptionStorage, SingletonStorage, Storage};

/// The trait for a key that can be used to store values in the maps.
pub trait Key<K, V>: Copy {
    /// The `Storage` implementation to use for the key implementing this trait.
    type Storage: Storage<K, V>;
}

impl<V> Key<bool, V> for bool {
    type Storage = BooleanStorage<V>;
}

impl<K, V> Key<Option<K>, V> for Option<K>
where
    K: Key<K, V>,
{
    type Storage = OptionStorage<K, V>;
}

macro_rules! map_key {
    ($ty:ty) => {
        #[cfg(feature = "map")]
        impl<V> Key<$ty, V> for $ty {
            type Storage = MapStorage<$ty, V>;
        }
    };
}

macro_rules! singleton_key {
    ($ty:ty) => {
        impl<V> Key<$ty, V> for $ty {
            type Storage = SingletonStorage<V>;
        }
    };
}

map_key!(char);
map_key!(u8);
map_key!(u32);
map_key!(u64);
map_key!(u128);
map_key!(usize);
map_key!(i8);
map_key!(i32);
map_key!(i64);
map_key!(i128);
map_key!(isize);
map_key!(&'static str);
map_key!(&'static [u8]);
singleton_key!(());
