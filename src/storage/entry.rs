//! Module for the trait to define [`StorageEntry`].

use crate::storage::Storage;

mod boolean;
#[cfg(feature = "map")]
mod map;
mod option;
mod singleton;

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

/// A view into a single entry in a map, which may either be vacant or occupied.
///
/// This enum is constructed from the [`entry`][crate::Map::entry] method on [`Map`][crate::Map].
pub enum Entry<'a, S: 'a, K, V>
where
    S: StorageEntry<K, V>,
{
    /// An occupied entry.
    Occupied(S::Occupied<'a>),
    /// A vacant entry.
    Vacant(S::Vacant<'a>),
}

impl<'a, S: 'a, K, V> Entry<'a, S, K, V>
where
    S: StorageEntry<K, V>,
{
    /// Ensures a value is in the entry by inserting the default if empty,
    /// and returns a mutable reference to the value in the entry.
    ///
    /// # Examples
    ///
    /// ```
    /// use fixed_map::{Key, Map};
    ///
    /// #[derive(Clone, Copy, Key)]
    /// enum Key {
    ///     First,
    ///     Second,
    /// }
    ///
    /// let mut map: Map<Key, i32> = Map::new();
    ///
    /// map.entry(Key::First).or_insert(3);
    /// assert_eq!(map.get(Key::First), Some(&3));
    ///
    /// *map.entry(Key::First).or_insert(10) *= 2;
    /// assert_eq!(map.get(Key::First), Some(&6));
    /// ```
    ///
    /// Using a composite key:
    ///
    /// ```
    /// use fixed_map::{Key, Map};
    ///
    /// #[derive(Clone, Copy, Key)]
    /// enum Key {
    ///     First(bool),
    ///     Second,
    /// }
    ///
    /// let mut map: Map<Key, i32> = Map::new();
    ///
    /// map.entry(Key::First(false)).or_insert(3);
    /// assert_eq!(map.get(Key::First(false)), Some(&3));
    ///
    /// *map.entry(Key::First(false)).or_insert(10) *= 2;
    /// assert_eq!(map.get(Key::First(false)), Some(&6));
    /// ```
    #[inline]
    pub fn or_insert(self, default: V) -> &'a mut V {
        match self {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => entry.insert(default),
        }
    }

    /// Ensures a value is in the entry by inserting the result of the default function if empty,
    /// and returns a mutable reference to the value in the entry.
    ///
    /// # Examples
    ///
    /// ```
    /// use fixed_map::{Key, Map};
    ///
    /// #[derive(Clone, Copy, Key)]
    /// enum Key {
    ///     First,
    ///     Second,
    /// }
    ///
    /// let mut map: Map<Key, String> = Map::new();
    ///
    /// map.entry(Key::First).or_insert_with(|| format!("{}", 3));
    /// assert_eq!(map.get(Key::First), Some(&"3".to_string()));
    /// ```
    ///
    /// Using a composite key:
    ///
    /// ```
    /// use fixed_map::{Key, Map};
    ///
    /// #[derive(Clone, Copy, Key)]
    /// enum Key {
    ///     First(bool),
    ///     Second,
    /// }
    ///
    /// let mut map: Map<Key, String> = Map::new();
    ///
    /// map.entry(Key::First(false)).or_insert_with(|| format!("{}", 3));
    /// assert_eq!(map.get(Key::First(false)), Some(&"3".to_string()));
    /// ```
    #[inline]
    pub fn or_insert_with<F>(self, default: F) -> &'a mut V
    where
        F: FnOnce() -> V,
    {
        match self {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => entry.insert(default()),
        }
    }

    /// Ensures a value is in the entry by inserting, if empty, the result of the default function.
    /// This method allows for generating key-derived values for insertion by providing the default
    /// function a copy of the key that was passed to the `.entry(key)` method call.
    ///
    /// # Examples
    ///
    /// ```
    /// use fixed_map::{Key, Map};
    ///
    /// #[derive(Clone, Copy, Key, Debug)]
    /// enum Key {
    ///     First,
    ///     Second,
    /// }
    ///
    /// let mut map: Map<Key, String> = Map::new();
    ///
    /// map.entry(Key::First).or_insert_with_key(|k| format!("{:?} = {}", k, 3));
    /// assert_eq!(map.get(Key::First), Some(&"First = 3".to_string()));
    /// ```
    ///
    /// Using a composite key:
    ///
    /// ```
    /// use fixed_map::{Key, Map};
    ///
    /// #[derive(Clone, Copy, Key, Debug)]
    /// enum Key {
    ///     First(bool),
    ///     Second,
    /// }
    ///
    /// let mut map: Map<Key, String> = Map::new();
    ///
    /// map.entry(Key::First(false)).or_insert_with_key(|k| format!("{:?} = {}", k, 3));
    /// assert_eq!(map.get(Key::First(false)), Some(&"First(false) = 3".to_string()));
    /// ```
    #[inline]
    pub fn or_insert_with_key<F>(self, default: F) -> &'a mut V
    where
        F: FnOnce(K) -> V,
    {
        match self {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => {
                let value = default(entry.key());
                entry.insert(value)
            }
        }
    }

    /// Returns a copy of this entry's key.
    ///
    /// # Examples
    ///
    /// ```
    /// use fixed_map::{Key, Map};
    ///
    /// #[derive(Clone, Copy, Key, Debug, PartialEq)]
    /// enum Key {
    ///     First,
    ///     Second,
    /// }
    ///
    /// let mut map: Map<Key, i32> = Map::new();
    /// assert_eq!(map.entry(Key::First).key(), Key::First);
    /// ```
    ///
    /// Using a composite key:
    ///
    /// ```
    /// use fixed_map::{Key, Map};
    ///
    /// #[derive(Clone, Copy, Key, Debug, PartialEq)]
    /// enum Key {
    ///     First(bool),
    ///     Second,
    /// }
    ///
    /// let mut map: Map<Key, i32> = Map::new();
    /// assert_eq!(map.entry(Key::First(false)).key(), Key::First(false));
    /// ```
    #[inline]
    pub fn key(&self) -> K {
        match self {
            Entry::Occupied(entry) => entry.key(),
            Entry::Vacant(entry) => entry.key(),
        }
    }

    /// Provides in-place mutable access to an occupied entry before any
    /// potential inserts into the map.
    ///
    /// # Examples
    ///
    /// ```
    /// use fixed_map::{Key, Map};
    ///
    /// #[derive(Clone, Copy, Key)]
    /// enum Key {
    ///     First,
    ///     Second,
    /// }
    ///
    /// let mut map: Map<Key, i32> = Map::new();
    ///
    /// map.entry(Key::First)
    ///    .and_modify(|e| { *e += 1 })
    ///    .or_insert(42);
    /// assert_eq!(map.get(Key::First), Some(&42));
    ///
    /// map.entry(Key::First)
    ///    .and_modify(|e| { *e += 1 })
    ///    .or_insert(42);
    /// assert_eq!(map.get(Key::First), Some(&43));
    /// ```
    ///
    /// Using a composite key:
    ///
    /// ```
    /// use fixed_map::{Key, Map};
    ///
    /// #[derive(Clone, Copy, Key)]
    /// enum Key {
    ///     First(bool),
    ///     Second,
    /// }
    ///
    /// let mut map: Map<Key, i32> = Map::new();
    ///
    /// map.entry(Key::First(true))
    ///    .and_modify(|e| { *e += 1 })
    ///    .or_insert(42);
    /// assert_eq!(map.get(Key::First(true)), Some(&42));
    ///
    /// map.entry(Key::First(true))
    ///    .and_modify(|e| { *e += 1 })
    ///    .or_insert(42);
    /// assert_eq!(map.get(Key::First(true)), Some(&43));
    /// ```
    #[inline]
    #[must_use]
    pub fn and_modify<F>(self, f: F) -> Self
    where
        F: FnOnce(&mut V),
    {
        match self {
            Entry::Occupied(mut entry) => {
                f(entry.get_mut());
                Entry::Occupied(entry)
            }
            Entry::Vacant(entry) => Entry::Vacant(entry),
        }
    }

    /// Ensures a value is in the entry by inserting the default value if empty,
    /// and returns a mutable reference to the value in the entry.
    ///
    /// # Examples
    ///
    /// ```
    /// use fixed_map::{Key, Map};
    ///
    /// #[derive(Clone, Copy, Key)]
    /// enum Key {
    ///     First,
    ///     Second,
    /// }
    ///
    /// let mut map: Map<Key, i32> = Map::new();
    ///
    /// map.entry(Key::First).or_default();
    /// assert_eq!(map.get(Key::First), Some(&0));
    /// ```
    ///
    /// Using a composite key:
    ///
    /// ```
    /// use fixed_map::{Key, Map};
    ///
    /// #[derive(Clone, Copy, Key)]
    /// enum Key {
    ///     First(bool),
    ///     Second,
    /// }
    ///
    /// let mut map: Map<Key, i32> = Map::new();
    ///
    /// map.entry(Key::First(false)).or_default();
    /// assert_eq!(map.get(Key::First(false)), Some(&0));
    /// ```
    #[inline]
    pub fn or_default(self) -> &'a mut V
    where
        V: Default,
    {
        match self {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => entry.insert(Default::default()),
        }
    }
}

/// The trait defining how the entry API for storage works.
///
/// # Type Arguments
///
/// - `K` is the key being stored.
/// - `V` is the value being stored.
pub trait StorageEntry<K, V>: Storage<K, V> {
    /// An occupied entry.
    type Occupied<'this>: OccupiedEntry<'this, K, V>
    where
        Self: 'this;

    /// A vacant entry.
    type Vacant<'this>: VacantEntry<'this, K, V>
    where
        Self: 'this;

    /// This is the storage abstraction for [`Map::entry`][crate::Map::entry].
    fn entry(&mut self, key: K) -> Entry<'_, Self, K, V>;
}
