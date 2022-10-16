//! Module for the trait to define [`Storage`].

mod boolean;
#[cfg(feature = "entry")]
pub(crate) mod entry;
#[cfg(feature = "map")]
mod map;
mod option;
mod singleton;

#[cfg(feature = "entry")]
pub use self::entry::StorageEntry;

pub use self::boolean::BooleanStorage;
#[cfg(feature = "map")]
pub use self::map::MapStorage;
pub use self::option::OptionStorage;
pub use self::singleton::SingletonStorage;

/// The trait defining how storage works.
///
/// # Type Arguments
///
/// - `K` is the key being stored.
/// - `V` is the value being stored.
pub trait Storage<K, V>: Default {
    /// Immutable iterator over storage.
    type Iter<'this>: Iterator<Item = (K, &'this V)>
    where
        Self: 'this,
        V: 'this;

    /// Immutable iterator over keys in storage.
    type Keys<'this>: Iterator<Item = K>
    where
        Self: 'this;

    /// Immutable iterator over values in storage.
    type Values<'this>: Iterator<Item = &'this V>
    where
        Self: 'this,
        V: 'this;

    /// Mutable iterator over storage.
    type IterMut<'this>: Iterator<Item = (K, &'this mut V)>
    where
        Self: 'this,
        V: 'this;

    /// Mutable iterator over values in storage.
    type ValuesMut<'this>: Iterator<Item = &'this mut V>
    where
        Self: 'this,
        V: 'this;

    /// Consuming iterator.
    type IntoIter: Iterator<Item = (K, V)>;

    /// Get the length of storage.
    fn len(&self) -> usize;

    /// Check if storage is empty.
    fn is_empty(&self) -> bool;

    /// This is the storage abstraction for [`Map::insert`][crate::Map::insert].
    fn insert(&mut self, key: K, value: V) -> Option<V>;

    /// This is the storage abstraction for [`Map::contains_key`][crate::Map::contains_key].
    fn contains_key(&self, key: K) -> bool;

    /// This is the storage abstraction for [`Map::get`][crate::Map::get].
    fn get(&self, key: K) -> Option<&V>;

    /// This is the storage abstraction for [`Map::get_mut`][crate::Map::get_mut].
    fn get_mut(&mut self, key: K) -> Option<&mut V>;

    /// This is the storage abstraction for [`Map::remove`][crate::Map::remove].
    fn remove(&mut self, key: K) -> Option<V>;

    /// This is the storage abstraction for [`Map::retain`][crate::Map::retain].
    fn retain<F>(&mut self, f: F)
    where
        F: FnMut(K, &mut V) -> bool;

    /// This is the storage abstraction for [`Map::clear`][crate::Map::clear].
    fn clear(&mut self);

    /// This is the storage abstraction for [`Map::iter`][crate::Map::iter].
    fn iter(&self) -> Self::Iter<'_>;

    /// This is the storage abstraction for [`Map::keys`][crate::Map::keys].
    fn keys(&self) -> Self::Keys<'_>;

    /// This is the storage abstraction for [`Map::values`][crate::Map::values].
    fn values(&self) -> Self::Values<'_>;

    /// This is the storage abstraction for [`Map::iter_mut`][crate::Map::iter_mut].
    fn iter_mut(&mut self) -> Self::IterMut<'_>;

    /// This is the storage abstraction for [`Map::values_mut`][crate::Map::values_mut].
    fn values_mut(&mut self) -> Self::ValuesMut<'_>;

    /// This is the storage abstraction for [`Map::into_iter`][crate::Map::into_iter].
    fn into_iter(self) -> Self::IntoIter;
}
