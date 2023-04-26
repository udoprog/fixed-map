use crate::map::{MapStorage, OccupiedEntry, VacantEntry};

/// A view into a single entry in a map, which may either be vacant or occupied.
///
/// This enum is constructed from the [`entry`][crate::Map::entry] method on [`Map`][crate::Map].
pub enum Entry<'a, S: 'a, K, V>
where
    S: MapStorage<K, V>,
{
    /// An occupied entry.
    Occupied(S::Occupied<'a>),
    /// A vacant entry.
    Vacant(S::Vacant<'a>),
}

impl<'a, S: 'a, K, V> Entry<'a, S, K, V>
where
    S: MapStorage<K, V>,
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
    /// enum MyKey {
    ///     First,
    ///     Second,
    /// }
    ///
    /// let mut map: Map<MyKey, i32> = Map::new();
    ///
    /// map.entry(MyKey::First).or_insert(3);
    /// assert_eq!(map.get(MyKey::First), Some(&3));
    ///
    /// *map.entry(MyKey::First).or_insert(10) *= 2;
    /// assert_eq!(map.get(MyKey::First), Some(&6));
    /// ```
    ///
    /// Using a composite key:
    ///
    /// ```
    /// use fixed_map::{Key, Map};
    ///
    /// #[derive(Clone, Copy, Key)]
    /// enum MyKey {
    ///     First(bool),
    ///     Second,
    /// }
    ///
    /// let mut map: Map<MyKey, i32> = Map::new();
    ///
    /// map.entry(MyKey::First(false)).or_insert(3);
    /// assert_eq!(map.get(MyKey::First(false)), Some(&3));
    ///
    /// *map.entry(MyKey::First(false)).or_insert(10) *= 2;
    /// assert_eq!(map.get(MyKey::First(false)), Some(&6));
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
    /// enum MyKey {
    ///     First,
    ///     Second,
    /// }
    ///
    /// let mut map: Map<MyKey, String> = Map::new();
    ///
    /// map.entry(MyKey::First).or_insert_with(|| format!("{}", 3));
    /// assert_eq!(map.get(MyKey::First), Some(&"3".to_string()));
    /// ```
    ///
    /// Using a composite key:
    ///
    /// ```
    /// use fixed_map::{Key, Map};
    ///
    /// #[derive(Clone, Copy, Key)]
    /// enum MyKey {
    ///     First(bool),
    ///     Second,
    /// }
    ///
    /// let mut map: Map<MyKey, String> = Map::new();
    ///
    /// map.entry(MyKey::First(false)).or_insert_with(|| format!("{}", 3));
    /// assert_eq!(map.get(MyKey::First(false)), Some(&"3".to_string()));
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
    /// enum MyKey {
    ///     First,
    ///     Second,
    /// }
    ///
    /// let mut map: Map<MyKey, String> = Map::new();
    ///
    /// map.entry(MyKey::First).or_insert_with_key(|k| format!("{:?} = {}", k, 3));
    /// assert_eq!(map.get(MyKey::First), Some(&"First = 3".to_string()));
    /// ```
    ///
    /// Using a composite key:
    ///
    /// ```
    /// use fixed_map::{Key, Map};
    ///
    /// #[derive(Clone, Copy, Key, Debug)]
    /// enum MyKey {
    ///     First(bool),
    ///     Second,
    /// }
    ///
    /// let mut map: Map<MyKey, String> = Map::new();
    ///
    /// map.entry(MyKey::First(false)).or_insert_with_key(|k| format!("{:?} = {}", k, 3));
    /// assert_eq!(map.get(MyKey::First(false)), Some(&"First(false) = 3".to_string()));
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
    /// enum MyKey {
    ///     First,
    ///     Second,
    /// }
    ///
    /// let mut map: Map<MyKey, i32> = Map::new();
    /// assert_eq!(map.entry(MyKey::First).key(), MyKey::First);
    /// ```
    ///
    /// Using a composite key:
    ///
    /// ```
    /// use fixed_map::{Key, Map};
    ///
    /// #[derive(Clone, Copy, Key, Debug, PartialEq)]
    /// enum MyKey {
    ///     First(bool),
    ///     Second,
    /// }
    ///
    /// let mut map: Map<MyKey, i32> = Map::new();
    /// assert_eq!(map.entry(MyKey::First(false)).key(), MyKey::First(false));
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
    /// enum MyKey {
    ///     First,
    ///     Second,
    /// }
    ///
    /// let mut map: Map<MyKey, i32> = Map::new();
    ///
    /// map.entry(MyKey::First)
    ///    .and_modify(|e| { *e += 1 })
    ///    .or_insert(42);
    /// assert_eq!(map.get(MyKey::First), Some(&42));
    ///
    /// map.entry(MyKey::First)
    ///    .and_modify(|e| { *e += 1 })
    ///    .or_insert(42);
    /// assert_eq!(map.get(MyKey::First), Some(&43));
    /// ```
    ///
    /// Using a composite key:
    ///
    /// ```
    /// use fixed_map::{Key, Map};
    ///
    /// #[derive(Clone, Copy, Key)]
    /// enum MyKey {
    ///     First(bool),
    ///     Second,
    /// }
    ///
    /// let mut map: Map<MyKey, i32> = Map::new();
    ///
    /// map.entry(MyKey::First(true))
    ///    .and_modify(|e| { *e += 1 })
    ///    .or_insert(42);
    /// assert_eq!(map.get(MyKey::First(true)), Some(&42));
    ///
    /// map.entry(MyKey::First(true))
    ///    .and_modify(|e| { *e += 1 })
    ///    .or_insert(42);
    /// assert_eq!(map.get(MyKey::First(true)), Some(&43));
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
    /// enum MyKey {
    ///     First,
    ///     Second,
    /// }
    ///
    /// let mut map: Map<MyKey, i32> = Map::new();
    ///
    /// map.entry(MyKey::First).or_default();
    /// assert_eq!(map.get(MyKey::First), Some(&0));
    /// ```
    ///
    /// Using a composite key:
    ///
    /// ```
    /// use fixed_map::{Key, Map};
    ///
    /// #[derive(Clone, Copy, Key)]
    /// enum MyKey {
    ///     First(bool),
    ///     Second,
    /// }
    ///
    /// let mut map: Map<MyKey, i32> = Map::new();
    ///
    /// map.entry(MyKey::First(false)).or_default();
    /// assert_eq!(map.get(MyKey::First(false)), Some(&0));
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
