//! Module for the trait to define a `Key`.

use crate::storage::{MapStorage, OptionStorage, SingletonStorage, Storage};

/// The trait for a key that can be used to store values in the maps.
pub trait Key<K: 'static, V: 'static>: Copy {
    /// The `Storage` implementation to use for the key implementing this trait.
    type Storage: Storage<K, V>;
}

impl<V: 'static> Key<&'static str, V> for &'static str {
    type Storage = MapStorage<Self, V>;
}

impl<K: 'static, V: 'static> Key<Option<K>, V> for Option<K>
where
    K: Key<K, V>,
{
    type Storage = OptionStorage<K, V>;
}

macro_rules! impl_map_storage {
    ($ty:ty) => {
        impl<V: 'static> Key<$ty, V> for $ty {
            type Storage = MapStorage<$ty, V>;
        }
    };
}

macro_rules! impl_singleton_storage {
    ($ty:ty) => {
        impl<V: 'static> Key<$ty, V> for $ty {
            type Storage = SingletonStorage<$ty, V>;
        }
    };
}

impl_map_storage!(u8);
impl_map_storage!(u32);
impl_map_storage!(u64);
impl_map_storage!(u128);
impl_map_storage!(usize);
impl_map_storage!(i8);
impl_map_storage!(i32);
impl_map_storage!(i64);
impl_map_storage!(i128);
impl_map_storage!(isize);
impl_singleton_storage!(());
