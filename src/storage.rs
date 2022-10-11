//! Module for the trait to define `Storage`.

mod boolean;
#[cfg(feature = "map")]
mod map;
mod option;
mod singleton;

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
    type Iter<'this>: Clone + Iterator<Item = (K, &'this V)>
    where
        Self: 'this,
        V: 'this;

    /// Immutable iterator over storage.
    type Values<'this>: Clone + Iterator<Item = &'this V>
    where
        Self: 'this,
        V: 'this;

    /// Mutable iterator over storage.
    type IterMut<'this>: Iterator<Item = (K, &'this mut V)>
    where
        Self: 'this,
        V: 'this;

    /// Consuming iterator.
    type IntoIter: Iterator<Item = (K, V)>;

    /// This is the storage abstraction for [`Map::insert`](struct.Map.html#method.insert).
    fn insert(&mut self, key: K, value: V) -> Option<V>;

    /// This is the storage abstraction for [`Map::get`](struct.Map.html#method.get).
    fn get(&self, key: K) -> Option<&V>;

    /// This is the storage abstraction for [`Map::get_mut`](struct.Map.html#method.get_mut).
    fn get_mut(&mut self, key: K) -> Option<&mut V>;

    /// This is the storage abstraction for [`Map::remove`](struct.Map.html#method.remove).
    fn remove(&mut self, key: K) -> Option<V>;

    /// This is the storage abstraction for [`Map::clear`](struct.Map.html#method.clear).
    fn clear(&mut self);

    /// This is the storage abstraction for [`Map::iter`](struct.Map.html#method.iter).
    fn iter(&self) -> Self::Iter<'_>;

    /// This is the storage abstraction for [`Map::values`](struct.Map.html#method.values).
    fn values(&self) -> Self::Values<'_>;

    /// This is the storage abstraction for [`Map::iter_mut`](struct.Map.html#method.iter_mut).
    fn iter_mut(&mut self) -> Self::IterMut<'_>;

    /// This is the storage abstraction for [`Map::into_iter`](struct.Map.html#method.into_iter)
    fn into_iter(self) -> Self::IntoIter;
}
