//! Module that defines the [`Storage`] trait.

mod boolean;
pub(crate) use self::boolean::BooleanStorage;

#[cfg(feature = "map")]
mod map;
#[cfg(feature = "map")]
pub(crate) use self::map::MapStorage;

mod option;
pub(crate) use self::option::OptionStorage;

mod singleton;
pub(crate) use self::singleton::SingletonStorage;

use crate::map::Entry;

/// The trait defining how storage works.
///
/// # Type Arguments
///
/// - `K` is the key being stored.
/// - `V` is the value being stored.
pub trait Storage<K, V>: Sized {
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

    /// An occupied entry.
    type Occupied<'this>: OccupiedEntry<'this, K, V>
    where
        Self: 'this;

    /// A vacant entry.
    type Vacant<'this>: VacantEntry<'this, K, V>
    where
        Self: 'this;

    /// Construct empty storage.
    fn empty() -> Self;

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

    /// This is the storage abstraction for [`Map::entry`][crate::Map::entry].
    fn entry(&mut self, key: K) -> Entry<'_, Self, K, V>;
}

/// A view into an occupied entry in a [`Map`][crate::Map]. It is part of the
/// [`Entry`] enum.
pub trait OccupiedEntry<'a, K, V> {
    /// Gets a copy of the key in the entry.
    ///
    /// # Examples
    ///
    /// ```
    /// use fixed_map::{Key, Map};
    /// use fixed_map::map::{Entry, OccupiedEntry};
    ///
    /// #[derive(Clone, Copy, Key, Debug, PartialEq)]
    /// enum Key {
    ///     First,
    ///     Second,
    /// }
    ///
    /// let mut map: Map<Key, i32> = Map::new();
    /// map.insert(Key::First, 12);
    ///
    /// let occupied = match map.entry(Key::First) {
    ///     Entry::Occupied(entry) => entry,
    ///     _ => unreachable!(),
    /// };
    ///
    /// assert_eq!(occupied.key(), Key::First);
    /// ```
    ///
    /// Using a composite key:
    ///
    /// ```
    /// use fixed_map::{Key, Map};
    /// use fixed_map::map::{Entry, OccupiedEntry};
    ///
    /// #[derive(Clone, Copy, Key, Debug, PartialEq)]
    /// enum Key {
    ///     First(bool),
    ///     Second,
    /// }
    ///
    /// let mut map: Map<Key, i32> = Map::new();
    /// map.insert(Key::First(false), 12);
    ///
    /// let occupied = match map.entry(Key::First(false)) {
    ///     Entry::Occupied(entry) => entry,
    ///     _ => unreachable!(),
    /// };
    ///
    /// assert_eq!(occupied.key(), Key::First(false));
    /// ```
    fn key(&self) -> K;

    /// Gets a reference to the value in the entry.
    ///
    /// # Examples
    ///
    /// ```
    /// use fixed_map::{Key, Map};
    /// use fixed_map::map::{Entry, OccupiedEntry};
    ///
    /// #[derive(Clone, Copy, Key)]
    /// enum Key {
    ///     First,
    ///     Second,
    /// }
    ///
    /// let mut map: Map<Key, i32> = Map::new();
    /// map.insert(Key::First, 12);
    ///
    /// let occupied = match map.entry(Key::First) {
    ///     Entry::Occupied(entry) => entry,
    ///     _ => unreachable!(),
    /// };
    ///
    /// assert_eq!(occupied.get(), &12);
    /// ```
    ///
    /// Using a composite key:
    ///
    /// ```
    /// use fixed_map::{Key, Map};
    /// use fixed_map::map::{Entry, OccupiedEntry};
    ///
    /// #[derive(Clone, Copy, Key)]
    /// enum Key {
    ///     First(bool),
    ///     Second,
    /// }
    ///
    /// let mut map: Map<Key, i32> = Map::new();
    /// map.insert(Key::First(false), 12);
    ///
    /// let occupied = match map.entry(Key::First(false)) {
    ///     Entry::Occupied(entry) => entry,
    ///     _ => unreachable!(),
    /// };
    ///
    /// assert_eq!(occupied.get(), &12);
    /// ```
    fn get(&self) -> &V;

    /// Gets a mutable reference to the value in the entry.
    ///
    /// If you need a reference to the `OccupiedEntry` which may
    /// outlive the destruction of the `Entry` value, see [`into_mut`][Self::into_mut].
    ///
    /// # Examples
    ///
    /// ```
    /// use fixed_map::{Key, Map};
    /// use fixed_map::map::{Entry, OccupiedEntry};
    ///
    /// #[derive(Clone, Copy, Key)]
    /// enum Key {
    ///     First,
    ///     Second,
    /// }
    ///
    /// let mut map: Map<Key, i32> = Map::new();
    /// map.insert(Key::First, 12);
    ///
    /// let mut occupied = match map.entry(Key::First) {
    ///     Entry::Occupied(entry) => entry,
    ///     _ => unreachable!(),
    /// };
    ///
    /// assert_eq!(occupied.get_mut(), &12);
    /// ```
    ///
    /// Using a composite key:
    ///
    /// ```
    /// use fixed_map::{Key, Map};
    /// use fixed_map::map::{Entry, OccupiedEntry};
    ///
    /// #[derive(Clone, Copy, Key)]
    /// enum Key {
    ///     First(bool),
    ///     Second,
    /// }
    ///
    /// let mut map: Map<Key, i32> = Map::new();
    /// map.insert(Key::First(false), 12);
    ///
    /// let mut occupied = match map.entry(Key::First(false)) {
    ///     Entry::Occupied(entry) => entry,
    ///     _ => unreachable!(),
    /// };
    ///
    /// *occupied.get_mut() *= 2;
    /// assert_eq!(occupied.get(), &24);
    /// // We can use the same Entry multiple times.
    /// *occupied.get_mut() -= 10;
    /// assert_eq!(occupied.get(), &14);
    /// ```
    fn get_mut(&mut self) -> &mut V;

    /// Converts the `OccupiedEntry` into a mutable reference to the value in the entry
    /// with a lifetime bound to the map itself.
    ///
    /// If you need multiple references to the `OccupiedEntry`, see [`get_mut`][Self::get_mut].
    ///
    /// # Examples
    ///
    /// ```
    /// use fixed_map::{Key, Map};
    /// use fixed_map::map::{Entry, OccupiedEntry};
    ///
    /// #[derive(Clone, Copy, Key)]
    /// enum Key {
    ///     First,
    ///     Second,
    /// }
    ///
    /// let mut map: Map<Key, i32> = Map::new();
    /// map.insert(Key::First, 12);
    ///
    /// if let Entry::Occupied(occupied) = map.entry(Key::First) {
    ///     *occupied.into_mut() += 10;
    /// };
    ///
    /// assert_eq!(map.get(Key::First), Some(&22));
    /// ```
    ///
    /// Using a composite key:
    ///
    /// ```
    /// use fixed_map::{Key, Map};
    /// use fixed_map::map::{Entry, OccupiedEntry};
    ///
    /// #[derive(Clone, Copy, Key)]
    /// enum Key {
    ///     First(bool),
    ///     Second,
    /// }
    ///
    /// let mut map: Map<Key, i32> = Map::new();
    /// map.insert(Key::First(false), 12);
    ///
    /// if let Entry::Occupied(occupied) = map.entry(Key::First(false)) {
    ///     *occupied.into_mut() += 10;
    /// };
    ///
    /// assert_eq!(map.get(Key::First(false)), Some(&22));
    /// ```
    fn into_mut(self) -> &'a mut V;

    /// Sets the value of the entry, and returns the entry's old value.
    ///
    /// # Examples
    ///
    /// ```
    /// use fixed_map::{Key, Map};
    /// use fixed_map::map::{Entry, OccupiedEntry};
    ///
    /// #[derive(Clone, Copy, Key)]
    /// enum Key {
    ///     First,
    ///     Second,
    /// }
    ///
    /// let mut map: Map<Key, i32> = Map::new();
    /// map.insert(Key::First, 12);
    ///
    /// if let Entry::Occupied(mut occupied) = map.entry(Key::First) {
    ///     assert_eq!(occupied.insert(10), 12);
    /// };
    ///
    /// assert_eq!(map.get(Key::First), Some(&10));
    /// ```
    ///
    /// Using a composite key:
    ///
    /// ```
    /// use fixed_map::{Key, Map};
    /// use fixed_map::map::{Entry, OccupiedEntry};
    ///
    /// #[derive(Clone, Copy, Key)]
    /// enum Key {
    ///     First(bool),
    ///     Second,
    /// }
    ///
    /// let mut map: Map<Key, i32> = Map::new();
    /// map.insert(Key::First(false), 12);
    ///
    /// if let Entry::Occupied(mut occupied) = map.entry(Key::First(false)) {
    ///     assert_eq!(occupied.insert(10), 12);
    /// };
    ///
    /// assert_eq!(map.get(Key::First(false)), Some(&10));
    /// ```
    fn insert(&mut self, value: V) -> V;

    /// Takes the value out of the entry, and returns it.
    ///
    /// # Examples
    ///
    /// ```
    /// use fixed_map::{Key, Map};
    /// use fixed_map::map::{Entry, OccupiedEntry};
    ///
    /// #[derive(Clone, Copy, Key)]
    /// enum Key {
    ///     First,
    ///     Second,
    /// }
    ///
    /// let mut map: Map<Key, i32> = Map::new();
    /// map.insert(Key::First, 12);
    ///
    /// if let Entry::Occupied(occupied) = map.entry(Key::First) {
    ///     assert_eq!(occupied.remove(), 12);
    /// };
    ///
    /// assert_eq!(map.contains_key(Key::First), false);
    /// ```
    ///
    /// Using a composite key:
    ///
    /// ```
    /// use fixed_map::{Key, Map};
    /// use fixed_map::map::{Entry, OccupiedEntry};
    ///
    /// #[derive(Clone, Copy, Key)]
    /// enum Key {
    ///     First(bool),
    ///     Second,
    /// }
    ///
    /// let mut map: Map<Key, i32> = Map::new();
    /// map.insert(Key::First(true), 12);
    ///
    /// if let Entry::Occupied(occupied) = map.entry(Key::First(true)) {
    ///     assert_eq!(occupied.remove(), 12);
    /// };
    ///
    /// assert_eq!(map.contains_key(Key::First(true)), false);
    /// ```
    fn remove(self) -> V;
}

/// A view into a vacant entry in a [`Map`][crate::Map].
/// It is part of the [`Entry`] enum.
pub trait VacantEntry<'a, K, V> {
    /// Gets a copy of the key that would be used
    /// when inserting a value through the `VacantEntry`.
    ///
    /// # Examples
    ///
    /// ```
    /// use fixed_map::{Key, Map};
    /// use fixed_map::map::{Entry, VacantEntry};
    ///
    /// #[derive(Clone, Copy, Key, Debug, PartialEq)]
    /// enum Key {
    ///     First,
    ///     Second,
    /// }
    ///
    /// let mut map: Map<Key, i32> = Map::new();
    /// let vacant = match map.entry(Key::First) {
    ///     Entry::Vacant(entry) => entry,
    ///     _ => unreachable!(),
    /// };
    ///
    /// assert_eq!(vacant.key(), Key::First);
    /// ```
    ///
    /// Using a composite key:
    ///
    /// ```
    /// use fixed_map::{Key, Map};
    /// use fixed_map::map::{Entry, VacantEntry};
    ///
    /// #[derive(Clone, Copy, Key, Debug, PartialEq)]
    /// enum Key {
    ///     First(bool),
    ///     Second,
    /// }
    ///
    /// let mut map: Map<Key, i32> = Map::new();
    /// let vacant = match map.entry(Key::First(false)) {
    ///     Entry::Vacant(entry) => entry,
    ///     _ => unreachable!(),
    /// };
    ///
    /// assert_eq!(vacant.key(), Key::First(false));
    /// ```
    fn key(&self) -> K;

    /// Sets the value of the entry with the `VacantEntry`’s key,
    /// and returns a mutable reference to it.
    ///
    /// # Examples
    ///
    /// ```
    /// use fixed_map::{Key, Map};
    /// use fixed_map::map::{Entry, VacantEntry};
    ///
    /// #[derive(Clone, Copy, Key)]
    /// enum Key {
    ///     First,
    ///     Second,
    /// }
    ///
    /// let mut map: Map<Key, i32> = Map::new();
    ///
    /// if let Entry::Vacant(vacant) = map.entry(Key::First) {
    ///     assert_eq!(vacant.insert(37), &37);
    /// }
    ///
    /// assert_eq!(map.get(Key::First), Some(&37));
    /// ```
    ///
    /// Using a composite key:
    ///
    /// ```
    /// use fixed_map::{Key, Map};
    /// use fixed_map::map::{Entry, VacantEntry};
    ///
    /// #[derive(Clone, Copy, Key)]
    /// enum Key {
    ///     First(bool),
    ///     Second,
    /// }
    ///
    /// let mut map: Map<Key, i32> = Map::new();
    ///
    /// if let Entry::Vacant(vacant) = map.entry(Key::First(false)) {
    ///     assert_eq!(vacant.insert(37), &37);
    /// }
    ///
    /// assert_eq!(map.get(Key::First(false)), Some(&37));
    /// ```
    fn insert(self, value: V) -> &'a mut V;
}
