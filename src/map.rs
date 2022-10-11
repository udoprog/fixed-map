//! Contains the fixed `Map` implementation.
use crate::{key::Key, storage::Storage};

/// A fixed map with a predetermined size.
///
/// # Examples
///
/// ```rust
/// use fixed_map::{Key, Map};
///
/// #[derive(Clone, Copy, Key)]
/// enum Part {
///     One,
///     Two,
/// }
///
/// #[derive(Clone, Copy, Key)]
/// enum Key {
///     Simple,
///     Composite(Part),
///     String(&'static str),
///     Number(u32),
///     Singleton(()),
///     Option(Option<Part>),
///     Boolean(bool),
/// }
///
/// let mut map = Map::new();
///
/// map.insert(Key::Simple, 1);
/// map.insert(Key::Composite(Part::One), 2);
/// map.insert(Key::String("foo"), 3);
/// map.insert(Key::Number(1), 4);
/// map.insert(Key::Singleton(()), 5);
/// map.insert(Key::Option(None), 6);
/// map.insert(Key::Option(Some(Part::One)), 7);
/// map.insert(Key::Boolean(true), 8);
///
/// assert_eq!(map.get(Key::Simple), Some(&1));
/// assert_eq!(map.get(Key::Composite(Part::One)), Some(&2));
/// assert_eq!(map.get(Key::Composite(Part::Two)), None);
/// assert_eq!(map.get(Key::String("foo")), Some(&3));
/// assert_eq!(map.get(Key::String("bar")), None);
/// assert_eq!(map.get(Key::Number(1)), Some(&4));
/// assert_eq!(map.get(Key::Number(2)), None);
/// assert_eq!(map.get(Key::Singleton(())), Some(&5));
/// assert_eq!(map.get(Key::Option(None)), Some(&6));
/// assert_eq!(map.get(Key::Option(Some(Part::One))), Some(&7));
/// assert_eq!(map.get(Key::Option(Some(Part::Two))), None);
/// assert_eq!(map.get(Key::Boolean(true)), Some(&8));
/// assert_eq!(map.get(Key::Boolean(false)), None);
/// ```
///
/// Storing references:
///
/// ```rust
/// use fixed_map::{Key, Map};
///
/// #[derive(Debug, Clone, Copy, Key)]
/// enum Key {
///     First,
///     Second,
/// }
///
/// let mut map = Map::new();
/// let a = 42u32;
///
/// map.insert(Key::First, &a);
///
/// assert_eq!(map.values().cloned().collect::<Vec<_>>(), vec![&42u32]);
/// ```
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
/// use fixed_map::{Key, Map};
///
/// #[derive(Clone, Copy, Key)]
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
/// use fixed_map::{Key, Map};
///
/// #[derive(Clone, Copy, Key)]
/// enum Part {
///     A,
///     B,
/// }
///
/// #[derive(Clone, Copy, Key)]
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
impl<K, V> Map<K, V>
where
    K: Key<K, V>,
{
    /// Creates an empty `Map`.
    ///
    /// # Examples
    ///
    /// ```
    /// use fixed_map::{Key, Map};
    ///
    /// #[derive(Clone, Copy, Key)]
    /// enum Key {
    ///     One,
    ///     Two,
    /// }
    ///
    /// let mut map: Map<Key, i32> = Map::new();
    /// ```
    #[inline]
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
    /// use fixed_map::{Key, Map};
    ///
    /// #[derive(Debug, Clone, Copy, PartialEq, Eq, Key)]
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
    #[inline]
    pub fn keys(&self) -> Keys<'_, K, V> {
        Keys { inner: self.iter() }
    }

    /// An iterator visiting all values in arbitrary order.
    /// The iterator element type is `&'a V`.
    ///
    /// # Examples
    ///
    /// ```
    /// use fixed_map::{Key, Map};
    ///
    /// #[derive(Debug, Clone, Copy, PartialEq, Eq, Key)]
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
    #[inline]
    pub fn values(&self) -> Values<'_, K, V> {
        Values { inner: self.iter() }
    }

    /// An iterator visiting all values mutably in arbitrary order.
    /// The iterator element type is `&'a mut V`.
    ///
    /// # Examples
    ///
    /// ```
    /// use fixed_map::{Key, Map};
    ///
    /// #[derive(Debug, Clone, Copy, PartialEq, Eq, Key)]
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
    #[inline]
    pub fn values_mut(&mut self) -> ValuesMut<'_, K, V> {
        ValuesMut {
            inner: self.iter_mut(),
        }
    }

    /// An iterator visiting all key-value pairs in arbitrary order.
    /// The iterator element type is `(K, &'a V)`.
    ///
    /// # Examples
    ///
    /// ```
    /// use fixed_map::{Key, Map};
    ///
    /// #[derive(Debug, Clone, Copy, PartialEq, Eq, Key)]
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
    #[inline]
    pub fn iter(&self) -> Iter<'_, K, V> {
        Iter {
            iter: self.storage.iter(),
        }
    }

    /// An iterator visiting all key-value pairs in arbitrary order,
    /// with mutable references to the values.
    /// The iterator element type is `(K, &'a mut V)`.
    ///
    /// # Examples
    ///
    /// ```
    /// use fixed_map::{Key, Map};
    ///
    /// #[derive(Debug, Clone, Copy, PartialEq, Eq, Key)]
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
    #[inline]
    pub fn iter_mut(&mut self) -> IterMut<'_, K, V> {
        IterMut {
            iter: self.storage.iter_mut(),
        }
    }

    /// Returns a reference to the value corresponding to the key.
    ///
    /// # Examples
    ///
    /// ```
    /// use fixed_map::{Key, Map};
    ///
    /// #[derive(Clone, Copy, Key)]
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
    #[inline]
    pub fn get(&self, key: K) -> Option<&V> {
        self.storage.get(key)
    }

    /// Returns a mutable reference to the value corresponding to the key.
    ///
    /// # Examples
    ///
    /// ```
    /// use fixed_map::{Key, Map};
    ///
    /// #[derive(Clone, Copy, Key)]
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
    #[inline]
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
    /// use fixed_map::{Key, Map};
    ///
    /// #[derive(Clone, Copy, Key)]
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
    #[inline]
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        self.storage.insert(key, value)
    }

    /// Removes a key from the map, returning the value at the key if the key
    /// was previously in the map.
    ///
    /// # Examples
    ///
    /// ```
    /// use fixed_map::{Key, Map};
    ///
    /// #[derive(Clone, Copy, Key)]
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
    #[inline]
    pub fn remove(&mut self, key: K) -> Option<V> {
        self.storage.remove(key)
    }

    /// Clears the map, removing all key-value pairs. Keeps the allocated memory
    /// for reuse.
    ///
    /// # Examples
    ///
    /// ```
    /// use fixed_map::{Key, Map};
    ///
    /// #[derive(Clone, Copy, Key)]
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
    #[inline]
    pub fn clear(&mut self) {
        self.storage.clear()
    }

    /// Returns true if the map contains no elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use fixed_map::{Key, Map};
    ///
    /// #[derive(Clone, Copy, Key)]
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
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.iter().next().is_none()
    }

    /// Returns the number of elements in the map.
    ///
    /// # Examples
    ///
    /// ```
    /// use fixed_map::{Key, Map};
    ///
    /// #[derive(Clone, Copy, Key)]
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
    #[inline]
    pub fn len(&self) -> usize {
        self.iter().count()
    }
}

impl<K, V> Clone for Map<K, V>
where
    K: Key<K, V>,
    K::Storage: Clone,
{
    #[inline]
    fn clone(&self) -> Map<K, V> {
        Map {
            storage: self.storage.clone(),
        }
    }
}

impl<K, V> Default for Map<K, V>
where
    K: Key<K, V>,
{
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<K, V> std::fmt::Debug for Map<K, V>
where
    K: Key<K, V> + std::fmt::Debug,
    V: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut debug_map = f.debug_map();
        for (k, v) in self.iter() {
            debug_map.entry(&k, v);
        }
        debug_map.finish()
    }
}

impl<K, V> PartialEq for Map<K, V>
where
    K: Key<K, V>,
    K::Storage: PartialEq,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.storage == other.storage
    }
}

impl<K, V> Eq for Map<K, V>
where
    K: Key<K, V>,
    K::Storage: Eq,
{
}

/// An iterator over the entries of a `Map`.
///
/// This `struct` is created by the [`iter`] method on [`Map`]. See its
/// documentation for more.
///
/// [`iter`]: struct.Map.html#method.iter
/// [`Map`]: struct.Map.html
pub struct Iter<'a, K, V: 'a>
where
    K: 'a + Key<K, V>,
{
    iter: <K::Storage as Storage<K, V>>::Iter<'a>,
}

impl<'a, K, V: 'a> Clone for Iter<'a, K, V>
where
    K: Key<K, V>,
{
    #[inline]
    fn clone(&self) -> Iter<'a, K, V> {
        Iter {
            iter: self.iter.clone(),
        }
    }
}

impl<'a, K: 'a, V: 'a> Iterator for Iter<'a, K, V>
where
    K: Key<K, V>,
{
    type Item = (K, &'a V);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|(k, v)| (k, v))
    }
}

impl<'a, K, V> IntoIterator for &'a Map<K, V>
where
    K: Key<K, V>,
{
    type Item = (K, &'a V);
    type IntoIter = Iter<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// A mutable iterator over the entries of a `Map`.
///
/// This `struct` is created by the [`iter_mut`] method on [`Map`]. See its
/// documentation for more.
///
/// [`iter_mut`]: struct.Map.html#method.iter_mut
/// [`Map`]: struct.Map.html
pub struct IterMut<'a, K, V: 'a>
where
    K: 'a + Key<K, V>,
{
    iter: <K::Storage as Storage<K, V>>::IterMut<'a>,
}

impl<'a, K, V> Iterator for IterMut<'a, K, V>
where
    K: Key<K, V>,
{
    type Item = (K, &'a mut V);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|(k, v)| (k, v))
    }
}

impl<'a, K, V> IntoIterator for &'a mut Map<K, V>
where
    K: Key<K, V>,
{
    type Item = (K, &'a mut V);
    type IntoIter = IterMut<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

/// An owning iterator over the entries of a `Map`.
///
/// This `struct` is created by the [`into_iter`] method on [`Map`]. See its
/// documentation for more.
///
/// [`into_iter`]: struct.Map.html#method.into_iter
/// [`Map`]: struct.Map.html
pub struct IntoIter<K, V>
where
    K: Key<K, V>,
{
    iter: <K::Storage as Storage<K, V>>::IntoIter,
}

impl<K, V> Iterator for IntoIter<K, V>
where
    K: Key<K, V>,
{
    type Item = (K, V);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

impl<K, V> IntoIterator for Map<K, V>
where
    K: Key<K, V>,
{
    type Item = (K, V);
    type IntoIter = IntoIter<K, V>;

    /// An owning iterator visiting all key-value pairs in arbitrary order.
    /// The iterator element type is `(K, V)`.
    ///
    /// # Examples
    ///
    /// ```
    /// use fixed_map::{Key, Map};
    ///
    /// #[derive(Debug, Clone, Copy, PartialEq, Eq, Key)]
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
    /// // Convert to a Vec
    /// let v: Vec<_> = map.into_iter().collect();
    ///
    /// assert_eq!(v, vec![(Key::One, 1), (Key::Two, 2)]);
    /// ```
    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            iter: self.storage.into_iter(),
        }
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
pub struct Keys<'a, K, V: 'a>
where
    K: Key<K, V>,
{
    inner: Iter<'a, K, V>,
}

impl<'a, K: 'a, V: 'a> Iterator for Keys<'a, K, V>
where
    K: Key<K, V>,
{
    type Item = K;

    #[inline]
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
pub struct Values<'a, K, V: 'a>
where
    K: Key<K, V>,
{
    inner: Iter<'a, K, V>,
}

impl<'a, K: 'a, V: 'a> Iterator for Values<'a, K, V>
where
    K: Key<K, V>,
{
    type Item = &'a V;

    #[inline]
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
pub struct ValuesMut<'a, K, V: 'a>
where
    K: Key<K, V>,
{
    inner: IterMut<'a, K, V>,
}

impl<'a, K: 'a, V: 'a> Iterator for ValuesMut<'a, K, V>
where
    K: Key<K, V>,
{
    type Item = &'a mut V;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|(_, v)| v)
    }
}

impl<K, V> std::iter::FromIterator<(K, V)> for Map<K, V>
where
    K: Key<K, V>,
{
    #[inline]
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        let mut map = Self::new();
        for (k, v) in iter {
            map.insert(k, v);
        }
        map
    }
}

#[cfg(feature = "serde")]
impl<K, V> serde::Serialize for Map<K, V>
where
    K: Key<K, V> + serde::Serialize,
    V: serde::Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeMap as _;

        let mut map = serializer.serialize_map(Some(self.len()))?;

        for (k, v) in self.iter() {
            map.serialize_entry(&k, v)?;
        }

        map.end()
    }
}

#[cfg(feature = "serde")]
impl<'de, K, V> serde::de::Deserialize<'de> for Map<K, V>
where
    K: Key<K, V> + serde::de::Deserialize<'de>,
    V: serde::Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        return deserializer.deserialize_map(MapVisitor(std::marker::PhantomData));

        struct MapVisitor<K, V>(std::marker::PhantomData<(K, V)>);

        impl<'de, K, V> serde::de::Visitor<'de> for MapVisitor<K, V>
        where
            K: Key<K, V> + serde::de::Deserialize<'de>,
            V: serde::Deserialize<'de>,
        {
            type Value = Map<K, V>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("a map")
            }

            #[inline]
            fn visit_map<T>(self, mut visitor: T) -> Result<Self::Value, T::Error>
            where
                T: serde::de::MapAccess<'de>,
            {
                let mut map = Map::new();

                while let Some((key, value)) = visitor.next_entry()? {
                    map.insert(key, value);
                }

                Ok(map)
            }
        }
    }
}
