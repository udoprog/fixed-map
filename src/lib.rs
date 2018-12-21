pub use fixed_map_derive::Key;
use std::vec;

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
    /// The iterator element type is `K`.
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
    /// assert_eq!(map.keys().collect::<Vec<_>>(), vec![Key::One, Key::Two]);
    /// ```
    pub fn keys<'a>(&'a self) -> Keys<'a, K, V> {
        Keys { inner: self.iter() }
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
    /// assert_eq!(map.values().map(|v| *v).collect::<Vec<_>>(), vec![1, 2]);
    /// ```
    pub fn values<'a>(&'a self) -> Values<'a, K, V> {
        Values { inner: self.iter() }
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
    /// for val in map.values_mut() {
    ///     *val += 10;
    /// }
    ///
    /// assert_eq!(map.values().map(|v| *v).collect::<Vec<_>>(), vec![11, 12]);
    /// ```
    pub fn values_mut<'a>(&'a mut self) -> ValuesMut<'a, K, V> {
        ValuesMut {
            inner: self.iter_mut(),
        }
    }

    /// An iterator visiting all key-value pairs in arbitrary order.
    /// The iterator element type is `(K, &'a V)`.
    ///
    /// Because of limitations in how Rust can express lifetimes through traits, this method will
    /// first pre-allocate a vector to store all references.
    ///
    /// For a zero-cost version of this function, see [`Map::iter_fn`].
    ///
    /// [`Map::iter_fn`]: struct.Map.html#method.iter_fn
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
    /// assert_eq!(map.iter().collect::<Vec<_>>(), vec![(Key::One, &1), (Key::Two, &2)]);
    /// ```
    pub fn iter<'a>(&'a self) -> Iter<'a, K, V> {
        let mut out = vec![];
        self.storage.iter(|e| out.push(e));

        Iter {
            inner: out.into_iter(),
        }
    }

    /// An closure visiting all key-value pairs in arbitrary order.
    /// The closure argument type is `(K, &'a V)`.
    ///
    /// This is a zero-cost version of [`Map::iter`].
    ///
    /// [`Map::iter`]: struct.Map.html#method.iter
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
    /// map.iter_fn(|e| out.push(e));
    /// assert_eq!(out, vec![(Key::One, &1), (Key::Two, &2)]);
    /// ```
    pub fn iter_fn<'a, F>(&'a self, f: F)
    where
        F: FnMut((K, &'a V)),
    {
        self.storage.iter(f)
    }

    /// An iterator visiting all key-value pairs in arbitrary order,
    /// with mutable references to the values.
    /// The iterator element type is `(K, &'a mut V)`.
    ///
    /// Because of limitations in how Rust can express lifetimes through traits, this method will
    /// first pre-allocate a vector to store all references.
    ///
    /// For a zero-cost version of this function, see [`Map::iter_mut_fn`].
    ///
    /// [`Map::iter_mut_fn`]: struct.Map.html#method.iter_mut_fn
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
    /// for (_, val) in map.iter_mut() {
    ///     *val *= 2;
    /// }
    ///
    /// assert_eq!(map.iter().collect::<Vec<_>>(), vec![(Key::One, &2), (Key::Two, &4)]);
    /// ```
    pub fn iter_mut<'a>(&'a mut self) -> IterMut<'a, K, V> {
        let mut out = vec![];
        self.storage.iter_mut(|e| out.push(e));

        IterMut {
            inner: out.into_iter(),
        }
    }

    /// An closure visiting all key-value pairs in arbitrary order,
    /// with mutable references to the values.
    /// The closure argument type is `(K, &'a mut V)`.
    ///
    /// This is a zero-cost version of [`Map::iter_mut`].
    ///
    /// [`Map::iter_mut`]: struct.Map.html#method.iter_mut
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
    /// map.iter_mut_fn(|(_, val)| {
    ///     *val *= 2;
    /// });
    ///
    /// let mut out = Vec::new();
    /// map.iter_fn(|e| out.push(e));
    /// assert_eq!(out, vec![(Key::One, &2), (Key::Two, &4)]);
    /// ```
    pub fn iter_mut_fn<'a, F>(&'a mut self, f: F)
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

/// An iterator over the entries of a `Map`.
///
/// This `struct` is created by the [`iter`] method on [`Map`]. See its
/// documentation for more.
///
/// [`iter`]: struct.Map.html#method.iter
/// [`Map`]: struct.Map.html
#[derive(Clone)]
pub struct Iter<'a, K, V: 'a> {
    inner: vec::IntoIter<(K, &'a V)>,
}

impl<'a, K: 'a, V: 'a> Iterator for Iter<'a, K, V> {
    type Item = (K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

/// A mutable iterator over the entries of a `Map`.
///
/// This `struct` is created by the [`iter_mut`] method on [`Map`]. See its
/// documentation for more.
///
/// [`iter_mut`]: struct.Map.html#method.iter_mut
/// [`Map`]: struct.Map.html
pub struct IterMut<'a, K, V: 'a> {
    inner: vec::IntoIter<(K, &'a mut V)>,
}

impl<'a, K: 'a, V: 'a> Iterator for IterMut<'a, K, V> {
    type Item = (K, &'a mut V);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

/// An iterator over the keys of a `Map`.
///
/// This `struct` is created by the [`keys`] method on [`Map`]. See its
/// documentation for more.
///
/// [`keys`]: struct.Map.html#method.keys
/// [`Map`]: struct.Map.html
#[derive(Clone)]
pub struct Keys<'a, K, V: 'a> {
    inner: Iter<'a, K, V>,
}

impl<'a, K: 'a, V: 'a> Iterator for Keys<'a, K, V> {
    type Item = K;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|(k, _)| k)
    }
}

/// An iterator over the values of a `Map`.
///
/// This `struct` is created by the [`values`] method on [`Map`]. See its
/// documentation for more.
///
/// [`values`]: struct.Map.html#method.values
/// [`Map`]: struct.Map.html
#[derive(Clone)]
pub struct Values<'a, K, V: 'a> {
    inner: Iter<'a, K, V>,
}

impl<'a, K: 'a, V: 'a> Iterator for Values<'a, K, V> {
    type Item = &'a V;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|(_, v)| v)
    }
}

/// A mutable iterator over the values of a `Map`.
///
/// This `struct` is created by the [`values_mut`] method on [`Map`]. See its
/// documentation for more.
///
/// [`values_mut`]: struct.Map.html#method.values_mut
/// [`Map`]: struct.Map.html
pub struct ValuesMut<'a, K, V: 'a> {
    inner: IterMut<'a, K, V>,
}

impl<'a, K: 'a, V: 'a> Iterator for ValuesMut<'a, K, V> {
    type Item = &'a mut V;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|(_, v)| v)
    }
}
