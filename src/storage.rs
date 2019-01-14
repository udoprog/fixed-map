//! Module for the trait to define `Storage`.

mod boolean;
mod map;
mod option;
mod singleton;

pub use self::boolean::BooleanStorage;
pub use self::map::MapStorage;
pub use self::option::OptionStorage;
pub use self::singleton::SingletonStorage;

/// The trait defining how storage works.
///
/// # Type Arguments
///
/// - `K` is the key being stored.
/// - `V` is the value being stored.
pub trait Storage<K, V: 'static>: Default {
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
    fn iter<'a>(&'a self, f: impl FnMut((K, &'a V)));

    /// This is the storage abstraction for [`Map::iter_mut`](struct.Map.html#method.iter_mut).
    fn iter_mut<'a>(&'a mut self, f: impl FnMut((K, &'a mut V)));
}
