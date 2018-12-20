pub use fixed_map_derive::Key;

/// The trait for a key that can be used to store values in the maps.
pub trait Key<K, V> {
    type Storage: Storage<K, V>;
}

/// The trait defining how storage works.
///
/// # Type Arguments
///
/// - `K` is the key being stored.
/// - `V` is the value being stored.
pub trait Storage<K, V>: Default {
    /// This is the storage abstraction for [`Map::insert`](struct.Map.html#method.insert).
    fn insert(&mut self, key: K, value: V) -> Option<V>;

    /// This is the storage abstraction for [`Map::get`](struct.Map.html#method.get).
    fn get(&self, key: &K) -> Option<&V>;

    /// This is the storage abstraction for [`Map::get_mut`](struct.Map.html#method.get_mut).
    fn get_mut(&mut self, key: &K) -> Option<&mut V>;

    /// This is the storage abstraction for [`Map::remove`](struct.Map.html#method.remove).
    fn remove(&mut self, key: &K) -> Option<V>;

    /// Call the given closure for each key and value combination that is present in storage.
    fn for_each_value<F>(&self, f: F)
    where
        F: FnMut(&V);
}

/// A map with a fixed, pre-determined size.
pub struct Map<K, V>
where
    K: Key<K, V>,
{
    storage: K::Storage,
}

/// A map implementation that uses fixed storage.
///
/// # Examples
///
/// ```rust
/// use fixed_map::Map;
///
/// #[derive(fixed_map::Key)]
/// enum MyKey {
///     Foo,
///     Bar,
/// }
///
/// let mut m = Map::new();
/// m.insert(MyKey::Foo, 42);
///
/// assert_eq!(m.get(&MyKey::Foo), Some(&42));
/// assert_eq!(m.get(&MyKey::Bar), None);
/// ```
impl<K, V> Map<K, V>
where
    K: Key<K, V>,
{
    pub fn new() -> Map<K, V> {
        Map {
            storage: K::Storage::default(),
        }
    }

    /// Returns a reference to the value corresponding to the key.
    ///
    /// # Examples
    ///
    /// ```
    /// use fixed_map::Map;
    ///
    /// #[derive(fixed_map::Key)]
    /// enum Key {
    ///     One,
    ///     Two,
    /// }
    ///
    /// let mut map = Map::new();
    /// map.insert(Key::One, "a");
    /// assert_eq!(map.get(&Key::One), Some(&"a"));
    /// assert_eq!(map.get(&Key::Two), None);
    /// ```
    pub fn get(&self, key: &K) -> Option<&V> {
        self.storage.get(key)
    }

    /// Returns a mutable reference to the value corresponding to the key.
    ///
    /// # Examples
    ///
    /// ```
    /// use fixed_map::Map;
    ///
    /// #[derive(fixed_map::Key)]
    /// enum Key {
    ///     One,
    ///     Two,
    /// }
    ///
    /// let mut map = Map::new();
    /// map.insert(Key::One, "a");
    /// if let Some(x) = map.get_mut(&Key::One) {
    ///     *x = "b";
    /// }
    /// assert_eq!(map.get(&Key::One), Some(&"b"));
    /// ```
    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        self.storage.get_mut(key)
    }

    /// Inserts a key-value pair into the map.
    ///
    /// If the map did not have this key present, [`None`] is returned.
    ///
    /// If the map did have this key present, the value is updated, and the old
    /// value is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use fixed_map::Map;
    ///
    /// #[derive(fixed_map::Key)]
    /// enum Key {
    ///     One,
    ///     Two,
    /// }
    ///
    /// let mut map = Map::new();
    /// assert_eq!(map.insert(Key::One, "a"), None);
    /// assert_eq!(map.is_empty(), false);
    ///
    /// map.insert(Key::Two, "b");
    /// assert_eq!(map.insert(Key::Two, "c"), Some("b"));
    /// assert_eq!(map.get(&Key::Two), Some(&"c"));
    /// ```
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        self.storage.insert(key, value)
    }

    /// Removes a key from the map, returning the value at the key if the key
    /// was previously in the map.
    ///
    /// # Examples
    ///
    /// ```
    /// use fixed_map::Map;
    ///
    /// #[derive(fixed_map::Key)]
    /// enum Key {
    ///     One,
    ///     Two,
    /// }
    ///
    /// let mut map = Map::new();
    /// map.insert(Key::One, "a");
    /// assert_eq!(map.remove(&Key::One), Some("a"));
    /// assert_eq!(map.remove(&Key::One), None);
    /// ```
    pub fn remove(&mut self, key: &K) -> Option<V> {
        self.storage.remove(key)
    }

    /// Returns true if the map contains no elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use fixed_map::Map;
    ///
    /// #[derive(fixed_map::Key)]
    /// enum Key {
    ///     One,
    ///     Two,
    /// }
    ///
    /// let mut map = Map::new();
    /// assert!(map.is_empty());
    /// map.insert(Key::One, "a");
    /// assert!(!map.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        let mut empty = true;

        self.storage.for_each_value(|_| {
            empty = false;
        });

        empty
    }

    /// Returns the number of elements in the map.
    ///
    /// # Examples
    ///
    /// ```
    /// use fixed_map::Map;
    ///
    /// #[derive(fixed_map::Key)]
    /// enum Key {
    ///     One,
    ///     Two,
    /// }
    ///
    /// let mut map = Map::new();
    /// assert_eq!(map.len(), 0);
    /// map.insert(Key::One, "a");
    /// assert_eq!(map.len(), 1);
    /// ```
    pub fn len(&self) -> usize {
        let mut len = 0;

        self.storage.for_each_value(|_| {
            len += 1;
        });

        len
    }
}

impl<K, V> Clone for Map<K, V>
where
    K: Key<K, V>,
    K::Storage: Clone,
{
    fn clone(&self) -> Map<K, V> {
        Map {
            storage: self.storage.clone(),
        }
    }
}
