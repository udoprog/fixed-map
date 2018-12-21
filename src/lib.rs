pub use fixed_map_derive::Key;

/// The trait for a key that can be used to store values in the maps.
pub trait Key<K: 'static, V: 'static>: Copy {
    type Storage: Storage<K, V>;
}

/// The trait defining how storage works.
///
/// # Type Arguments
///
/// - `K` is the key being stored.
/// - `V` is the value being stored.
pub trait Storage<K: 'static, V: 'static>: Default {
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
    fn iter<'a, F>(&'a self, f: F)
    where
        F: FnMut((K, &'a V));

    /// This is the storage abstraction for [`Map::iter_mut`](struct.Map.html#method.iter_mut).
    fn iter_mut<'a, F>(&'a mut self, f: F)
    where
        F: FnMut((K, &'a mut V));
}

/// A map with a fixed, pre-determined size.
pub struct Map<K: 'static, V: 'static>
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
/// #[derive(Clone, Copy, fixed_map::Key)]
/// enum Key {
///     One,
///     Two,
/// }
///
/// let mut m = Map::new();
/// m.insert(Key::One, 1);
///
/// assert_eq!(m.get(Key::One), Some(&1));
/// assert_eq!(m.get(Key::Two), None);
/// ```
///
/// ```rust
/// use fixed_map::Map;
///
/// #[derive(Clone, Copy, fixed_map::Key)]
/// enum Part {
///     A,
///     B,
/// }
///
/// #[derive(Clone, Copy, fixed_map::Key)]
/// enum Key {
///     Simple,
///     Composite(Part),
/// }
///
/// let mut m = Map::new();
/// m.insert(Key::Simple, 1);
/// m.insert(Key::Composite(Part::A), 2);
///
/// assert_eq!(m.get(Key::Simple), Some(&1));
/// assert_eq!(m.get(Key::Composite(Part::A)), Some(&2));
/// assert_eq!(m.get(Key::Composite(Part::B)), None);
/// ```
impl<K: 'static, V: 'static> Map<K, V>
where
    K: Key<K, V>,
{
    pub fn new() -> Map<K, V> {
        Map {
            storage: K::Storage::default(),
        }
    }

    /// An iterator visiting all keys in arbitrary order.
    /// The iterator element type is `&'a K`.
    ///
    /// # Examples
    ///
    /// ```
    /// use fixed_map::Map;
    ///
    /// #[derive(Debug, Clone, Copy, PartialEq, Eq, fixed_map::Key)]
    /// pub enum Key {
    ///     One,
    ///     Two,
    ///     Three,
    /// }
    ///
    /// let mut map = Map::new();
    /// map.insert(Key::One, 1);
    /// map.insert(Key::Two, 2);
    ///
    /// let mut out = Vec::new();
    /// map.keys(|key| out.push(key));
    /// assert_eq!(out, vec![Key::One, Key::Two]);
    /// ```
    pub fn keys<'a, F>(&'a self, mut f: F)
    where
        F: FnMut(K),
    {
        self.iter(|(k, _)| f(k));
    }

    /// An iterator visiting all values in arbitrary order.
    /// The iterator element type is `&'a V`.
    ///
    /// # Examples
    ///
    /// ```
    /// use fixed_map::Map;
    ///
    /// #[derive(Debug, Clone, Copy, PartialEq, Eq, fixed_map::Key)]
    /// pub enum Key {
    ///     One,
    ///     Two,
    ///     Three,
    /// }
    ///
    /// let mut map = Map::new();
    /// map.insert(Key::One, 1);
    /// map.insert(Key::Two, 2);
    ///
    /// let mut out = Vec::new();
    /// map.values(|val| out.push(*val));
    /// assert_eq!(out, vec![1, 2]);
    /// ```
    pub fn values<'a, F>(&'a self, mut f: F)
    where
        F: FnMut(&'a V),
    {
        self.iter(|(_, v)| f(v));
    }

    /// An iterator visiting all values mutably in arbitrary order.
    /// The iterator element type is `&'a mut V`.
    ///
    /// # Examples
    ///
    /// ```
    /// use fixed_map::Map;
    ///
    /// #[derive(Debug, Clone, Copy, PartialEq, Eq, fixed_map::Key)]
    /// pub enum Key {
    ///     One,
    ///     Two,
    ///     Three,
    /// }
    ///
    /// let mut map = Map::new();
    /// map.insert(Key::One, 1);
    /// map.insert(Key::Two, 2);
    ///
    /// map.values_mut(|val| *val = *val + 10);
    ///
    /// let mut out = Vec::new();
    /// map.values(|val| out.push(*val));
    /// assert_eq!(out, vec![11, 12]);
    /// ```
    pub fn values_mut<'a, F>(&'a mut self, mut f: F)
    where
        F: FnMut(&'a mut V),
    {
        self.iter_mut(|(_, v)| f(v));
    }

    /// An iterator visiting all key-value pairs in arbitrary order.
    /// The iterator element type is `(&'a K, &'a V)`.
    ///
    /// # Examples
    ///
    /// ```
    /// use fixed_map::Map;
    ///
    /// #[derive(Debug, Clone, Copy, PartialEq, Eq, fixed_map::Key)]
    /// enum Key {
    ///     One,
    ///     Two,
    ///     Three,
    /// }
    ///
    /// let mut map = Map::new();
    /// map.insert(Key::One, 1);
    /// map.insert(Key::Two, 2);
    ///
    /// let mut out = Vec::new();
    /// map.iter(|e| out.push(e));
    /// assert_eq!(out, vec![(Key::One, &1), (Key::Two, &2)]);
    /// ```
    pub fn iter<'a, F>(&'a self, f: F)
    where
        F: FnMut((K, &'a V)),
    {
        self.storage.iter(f)
    }

    /// An iterator visiting all key-value pairs in arbitrary order,
    /// with mutable references to the values.
    /// The iterator element type is `(&'a K, &'a mut V)`.
    ///
    /// # Examples
    ///
    /// ```
    /// use fixed_map::Map;
    ///
    /// #[derive(Debug, Clone, Copy, PartialEq, Eq, fixed_map::Key)]
    /// enum Key {
    ///     One,
    ///     Two,
    ///     Three,
    /// }
    ///
    /// let mut map = Map::new();
    /// map.insert(Key::One, 1);
    /// map.insert(Key::Two, 2);
    ///
    /// // Update all values
    /// map.iter_mut(|(_, val)| {
    ///     *val *= 2;
    /// });
    ///
    /// let mut out = Vec::new();
    /// map.iter(|e| out.push(e));
    /// assert_eq!(out, vec![(Key::One, &2), (Key::Two, &4)]);
    /// ```
    pub fn iter_mut<'a, F>(&'a mut self, f: F)
    where
        F: FnMut((K, &'a mut V)),
    {
        self.storage.iter_mut(f)
    }

    /// Returns a reference to the value corresponding to the key.
    ///
    /// # Examples
    ///
    /// ```
    /// use fixed_map::Map;
    ///
    /// #[derive(Clone, Copy, fixed_map::Key)]
    /// enum Key {
    ///     One,
    ///     Two,
    /// }
    ///
    /// let mut map = Map::new();
    /// map.insert(Key::One, "a");
    /// assert_eq!(map.get(Key::One), Some(&"a"));
    /// assert_eq!(map.get(Key::Two), None);
    /// ```
    pub fn get(&self, key: K) -> Option<&V> {
        self.storage.get(key)
    }

    /// Returns a mutable reference to the value corresponding to the key.
    ///
    /// # Examples
    ///
    /// ```
    /// use fixed_map::Map;
    ///
    /// #[derive(Clone, Copy, fixed_map::Key)]
    /// enum Key {
    ///     One,
    ///     Two,
    /// }
    ///
    /// let mut map = Map::new();
    /// map.insert(Key::One, "a");
    /// if let Some(x) = map.get_mut(Key::One) {
    ///     *x = "b";
    /// }
    /// assert_eq!(map.get(Key::One), Some(&"b"));
    /// ```
    pub fn get_mut(&mut self, key: K) -> Option<&mut V> {
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
    /// #[derive(Clone, Copy, fixed_map::Key)]
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
    /// assert_eq!(map.get(Key::Two), Some(&"c"));
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
    /// #[derive(Clone, Copy, fixed_map::Key)]
    /// enum Key {
    ///     One,
    ///     Two,
    /// }
    ///
    /// let mut map = Map::new();
    /// map.insert(Key::One, "a");
    /// assert_eq!(map.remove(Key::One), Some("a"));
    /// assert_eq!(map.remove(Key::One), None);
    /// ```
    pub fn remove(&mut self, key: K) -> Option<V> {
        self.storage.remove(key)
    }

    /// Clears the map, removing all key-value pairs. Keeps the allocated memory
    /// for reuse.
    ///
    /// # Examples
    ///
    /// ```
    /// use fixed_map::Map;
    ///
    /// #[derive(Clone, Copy, fixed_map::Key)]
    /// enum Key {
    ///     One,
    ///     Two,
    /// }
    ///
    /// let mut map = Map::new();
    /// map.insert(Key::One, "a");
    /// map.clear();
    /// assert!(map.is_empty());
    /// ```
    pub fn clear(&mut self) {
        self.storage.clear()
    }

    /// Returns true if the map contains no elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use fixed_map::Map;
    ///
    /// #[derive(Clone, Copy, fixed_map::Key)]
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

        self.storage.iter(|_| {
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
    /// #[derive(Clone, Copy, fixed_map::Key)]
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

        self.storage.iter(|_| {
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
