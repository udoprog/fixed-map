//! Contains the fixed [`Map`] implementation.

use core::fmt;

use crate::{key::Key, storage::Storage};

/// A fixed map.
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
#[repr(transparent)]
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
    /// Creates an empty [`Map`].
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
    #[must_use]
    pub fn new() -> Map<K, V> {
        Map {
            storage: K::Storage::default(),
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
    ///     First,
    ///     Second,
    ///     Third,
    /// }
    ///
    /// let mut map = Map::new();
    /// map.insert(Key::First, 1);
    /// map.insert(Key::Second, 2);
    ///
    /// assert!(map.keys().eq([Key::First, Key::Second]));
    /// assert!(map.keys().rev().eq([Key::Second, Key::First]));
    /// ```
    ///
    /// Using a composite key:
    ///
    /// ```
    /// use fixed_map::{Key, Map};
    ///
    /// #[derive(Debug, Clone, Copy, PartialEq, Eq, Key)]
    /// pub enum Key {
    ///     First,
    ///     Second(bool),
    ///     Third,
    /// }
    ///
    /// let mut map = Map::new();
    /// map.insert(Key::First, 1);
    /// map.insert(Key::Second(false), 2);
    ///
    /// dbg!(map.keys().collect::<Vec<_>>());
    ///
    /// assert!(map.keys().eq([Key::First, Key::Second(false)]));
    /// assert!(map.keys().rev().eq([Key::Second(false), Key::First]));
    /// ```
    #[inline]
    pub fn keys(&self) -> Keys<'_, K, V> {
        Keys {
            iter: self.storage.keys(),
        }
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
    ///     First,
    ///     Second,
    ///     Third,
    /// }
    ///
    /// let mut map = Map::new();
    /// map.insert(Key::First, 1);
    /// map.insert(Key::Second, 2);
    ///
    /// assert!(map.values().copied().eq([1, 2]));
    /// assert!(map.values().rev().copied().eq([2, 1]));
    /// ```
    ///
    /// Using a composite key:
    ///
    /// ```
    /// use fixed_map::{Key, Map};
    ///
    /// #[derive(Debug, Clone, Copy, PartialEq, Eq, Key)]
    /// pub enum Key {
    ///     First(bool),
    ///     Second,
    ///     Third,
    /// }
    ///
    /// let mut map = Map::new();
    /// map.insert(Key::First(false), 1);
    /// map.insert(Key::Second, 2);
    ///
    /// assert!(map.values().copied().eq([1, 2]));
    /// assert!(map.values().rev().copied().eq([2, 1]));
    /// ```
    #[inline]
    pub fn values(&self) -> Values<'_, K, V> {
        Values {
            iter: self.storage.values(),
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
    ///     First,
    ///     Second,
    /// }
    ///
    /// let mut map = Map::new();
    /// map.insert(Key::First, 1);
    /// map.insert(Key::Second, 2);
    ///
    /// // Update all values
    /// for (_, val) in map.iter_mut() {
    ///     *val *= 2;
    /// }
    ///
    /// assert!(map.iter().eq([(Key::First, &2), (Key::Second, &4)]));
    /// ```
    ///
    /// Using a composite key:
    ///
    /// ```
    /// use fixed_map::{Key, Map};
    ///
    /// #[derive(Debug, Clone, Copy, PartialEq, Eq, Key)]
    /// enum Key {
    ///     First(&'static str),
    ///     Second,
    /// }
    ///
    /// let mut map = Map::new();
    /// map.insert(Key::First("Hello"), 1);
    /// map.insert(Key::Second, 2);
    ///
    /// // Update all values
    /// for (_, val) in map.iter_mut() {
    ///     *val *= 2;
    /// }
    ///
    /// assert!(map.iter().eq([(Key::First("Hello"), &2), (Key::Second, &4)]));
    /// ```
    #[inline]
    pub fn iter_mut(&mut self) -> IterMut<'_, K, V> {
        IterMut {
            iter: self.storage.iter_mut(),
        }
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
    ///     First,
    ///     Second,
    /// }
    ///
    /// let mut map = Map::new();
    /// map.insert(Key::First, 2);
    /// map.insert(Key::Second, 5);
    ///
    /// for (index, val) in map.values_mut().enumerate() {
    ///     *val *= index + 1;
    /// }
    ///
    /// assert!(map.values().copied().eq([2, 10]));
    ///
    /// let mut map = Map::new();
    /// map.insert(Key::First, 2);
    /// map.insert(Key::Second, 5);
    ///
    /// for (index, val) in map.values_mut().rev().enumerate() {
    ///     *val *= index + 1;
    /// }
    ///
    /// assert!(map.values().copied().eq([4, 5]));
    /// ```
    ///
    /// Using a composite key:
    ///
    /// ```
    /// use fixed_map::{Key, Map};
    ///
    /// #[derive(Debug, Clone, Copy, PartialEq, Eq, Key)]
    /// pub enum Key {
    ///     First(bool),
    ///     Second,
    /// }
    ///
    /// let mut map = Map::new();
    /// map.insert(Key::First(false), 2);
    /// map.insert(Key::Second, 5);
    ///
    /// for (index, val) in map.values_mut().enumerate() {
    ///     *val *= index + 1;
    /// }
    ///
    /// assert!(map.values().copied().eq([2, 10]));
    ///
    /// let mut map = Map::new();
    /// map.insert(Key::First(false), 2);
    /// map.insert(Key::Second, 5);
    ///
    /// for (index, val) in map.values_mut().rev().enumerate() {
    ///     *val *= index + 1;
    /// }
    ///
    /// assert!(map.values().copied().eq([4, 5]));
    /// ```
    #[inline]
    pub fn values_mut(&mut self) -> ValuesMut<'_, K, V> {
        ValuesMut {
            iter: self.storage.values_mut(),
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
    ///     First,
    ///     Second,
    /// }
    ///
    /// let mut map = Map::new();
    /// map.insert(Key::First, "a");
    /// assert_eq!(map.get(Key::First).copied(), Some("a"));
    /// assert_eq!(map.get(Key::Second), None);
    /// ```
    ///
    /// Using a composite key:
    ///
    /// ```
    /// use fixed_map::{Key, Map};
    ///
    /// #[derive(Clone, Copy, Key)]
    /// enum Key {
    ///     First(&'static str),
    ///     Second,
    /// }
    ///
    /// let mut map = Map::new();
    /// map.insert(Key::First("Hello"), "a");
    /// assert_eq!(map.get(Key::First("Hello")).copied(), Some("a"));
    /// assert_eq!(map.get(Key::Second), None);
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
    ///     First,
    ///     Second,
    /// }
    ///
    /// let mut map = Map::new();
    /// map.insert(Key::First, "a");
    ///
    /// if let Some(x) = map.get_mut(Key::First) {
    ///     *x = "b";
    /// }
    ///
    /// assert_eq!(map.get(Key::First).copied(), Some("b"));
    /// ```
    ///
    /// Using a composite key:
    ///
    /// ```
    /// use fixed_map::{Key, Map};
    ///
    /// #[derive(Clone, Copy, Key)]
    /// enum Key {
    ///     First(&'static str),
    ///     Second(u32),
    ///     Third,
    /// }
    ///
    /// let mut map = Map::new();
    /// map.insert(Key::First("Hello"), "a");
    ///
    /// if let Some(x) = map.get_mut(Key::First("Hello")) {
    ///     *x = "b";
    /// }
    ///
    /// assert_eq!(map.get(Key::First("Hello")).copied(), Some("b"));
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
        self.storage.clear();
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
    ///     First,
    ///     Second,
    /// }
    ///
    /// let mut map = Map::new();
    /// assert!(map.is_empty());
    /// map.insert(Key::First, 1);
    /// assert!(!map.is_empty());
    /// ```
    ///
    /// An empty key:
    ///
    /// ```
    /// use fixed_map::{Key, Map};
    ///
    /// #[derive(Clone, Copy, Key)]
    /// enum Key {}
    ///
    /// let map = Map::<Key, u32>::new();
    /// assert!(map.is_empty());
    /// ```
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.storage.is_empty()
    }

    /// Gets the current length of a [`Map`].
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
    /// assert_eq!(map.len(), 0);
    ///
    /// map.insert(Key::First, 42);
    /// assert_eq!(map.len(), 1);
    ///
    /// map.insert(Key::First, 42);
    /// assert_eq!(map.len(), 1);
    ///
    /// map.remove(Key::First);
    /// assert_eq!(map.len(), 0);
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
    /// assert_eq!(map.len(), 0);
    ///
    /// map.insert(Key::First(true), 42);
    /// assert_eq!(map.len(), 1);
    ///
    /// map.insert(Key::First(false), 42);
    /// assert_eq!(map.len(), 2);
    ///
    /// map.remove(Key::First(true));
    /// assert_eq!(map.len(), 1);
    /// ```
    pub fn len(&self) -> usize {
        self.storage.len()
    }
}

/// [`Clone`] implementation for a [`Map`].
///
/// # Examples
///
/// ```
/// use fixed_map::{Key, Map};
///
/// #[derive(Debug, Clone, Copy, Key)]
/// enum Key {
///     First(&'static str),
///     Second,
/// }
///
/// let mut a = Map::new();
/// a.insert(Key::First("Hello"), 1);
/// let mut b = a.clone();
/// b.insert(Key::Second, 2);
///
/// assert_ne!(a, b);
///
/// assert_eq!(a.get(Key::First("Hello")), Some(&1));
/// assert_eq!(a.get(Key::Second), None);
///
/// assert_eq!(b.get(Key::First("Hello")), Some(&1));
/// assert_eq!(b.get(Key::Second), Some(&2));
/// ```
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

/// The [`Copy`] implementation for a [`Map`] depends on its [`Key`]. If the
/// derived key only consists of unit variants the corresponding [`Map`] will be
/// [`Copy`] as well.
///
/// # Examples
///
/// ```
/// use fixed_map::{Key, Map};
///
/// #[derive(Debug, Clone, Copy, Key)]
/// enum Key {
///     First,
///     Second,
/// }
///
/// let mut a = Map::new();
/// a.insert(Key::First, 1);
/// let mut b = a;
/// b.insert(Key::Second, 2);
///
/// assert_ne!(a, b);
///
/// assert_eq!(a.get(Key::First), Some(&1));
/// assert_eq!(a.get(Key::Second), None);
///
/// assert_eq!(b.get(Key::First), Some(&1));
/// assert_eq!(b.get(Key::Second), Some(&2));
/// ```
impl<K, V> Copy for Map<K, V>
where
    K: Key<K, V>,
    K::Storage: Copy,
{
}

/// The [`Default`] implementation for a [`Map`] produces an empty map.
///
/// # Examples
///
/// ```
/// use fixed_map::{Key, Map};
///
/// #[derive(Debug, Clone, Copy, Key)]
/// enum Key {
///     First,
///     Second,
/// }
///
/// let a = Map::<Key, u32>::default();
/// let b = Map::<Key, u32>::new();
///
/// assert_eq!(a, b);
/// ```
impl<K, V> Default for Map<K, V>
where
    K: Key<K, V>,
{
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

/// The [`Debug`][fmt::Debug] implementation for a [`Map`].
///
/// # Examples
///
/// ```
/// use fixed_map::{Key, Map};
///
/// #[derive(Debug, Clone, Copy, Key)]
/// enum Key {
///     First,
///     Second,
/// }
///
/// let mut a = Map::new();
/// a.insert(Key::First, 42);
///
/// assert_eq!("{First: 42}", format!("{:?}", a));
/// ```
impl<K, V> fmt::Debug for Map<K, V>
where
    K: Key<K, V> + fmt::Debug,
    V: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_map().entries(self.iter()).finish()
    }
}

/// [`PartialEq`] implementation for a [`Map`].
///
/// # Examples
///
/// ```
/// use fixed_map::{Key, Map};
///
/// #[derive(Debug, Clone, Copy, Key)]
/// enum Key {
///     First,
///     Second,
/// }
///
/// let mut a = Map::new();
/// a.insert(Key::First, 42);
/// // Note: `a` is Copy since it's using a simple key.
/// let mut b = a;
///
/// assert_eq!(a, b);
///
/// b.insert(Key::Second, 42);
/// assert_ne!(a, b);
/// ```
///
/// Using a composite key:
///
/// ```
/// use fixed_map::{Key, Map};
///
/// #[derive(Debug, Clone, Copy, Key)]
/// enum Key {
///     First(&'static str),
///     Second,
/// }
///
/// let mut a = Map::new();
/// a.insert(Key::First("Hello"), 42);
/// let mut b = a.clone();
///
/// assert_eq!(a, b);
///
/// b.insert(Key::Second, 42);
/// assert_ne!(a, b);
/// ```
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

/// An iterator over the entries of a [`Map`].
///
/// This type is created by the [`iter`][Map::iter] method. See its
/// documentation for more.
#[repr(transparent)]
pub struct Iter<'a, K, V>
where
    K: 'a + Key<K, V>,
    V: 'a,
{
    iter: <K::Storage as Storage<K, V>>::Iter<'a>,
}

impl<'a, K, V> Clone for Iter<'a, K, V>
where
    K: Key<K, V>,
    <K::Storage as Storage<K, V>>::Iter<'a>: Clone,
{
    #[inline]
    fn clone(&self) -> Iter<'a, K, V> {
        Iter {
            iter: self.iter.clone(),
        }
    }
}

iterator!(@identity, {Iter, Iter}, {'a}, [K, V], K, V => (K, &'a V));

impl<'a, K, V> IntoIterator for &'a Map<K, V>
where
    K: Key<K, V>,
{
    type Item = (K, &'a V);
    type IntoIter = Iter<'a, K, V>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// A mutable iterator over the entries of a [`Map`].
///
/// This type is created by the [`iter_mut`][Map::iter_mut] method.
/// See its documentation for more.
pub struct IterMut<'a, K, V: 'a>
where
    K: 'a + Key<K, V>,
{
    iter: <K::Storage as Storage<K, V>>::IterMut<'a>,
}

iterator!(@identity, {IterMut, IterMut}, {'a}, [K, V], K, V => (K, &'a mut V));

/// [`IntoIterator`] implementation which uses [`Map::iter_mut`]. See its
/// documentation for more.
impl<'a, K, V> IntoIterator for &'a mut Map<K, V>
where
    K: Key<K, V>,
{
    type Item = (K, &'a mut V);
    type IntoIter = IterMut<'a, K, V>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

/// An owning iterator over the entries of a [`Map`].
///
/// This type is created by the [`into_iter`][Map::into_iter] method. See its
/// documentation for more.
pub struct IntoIter<K, V>
where
    K: Key<K, V>,
{
    iter: <K::Storage as Storage<K, V>>::IntoIter,
}

iterator!(@identity, {IntoIter, IntoIter}, {}, [K, V], K, V => (K, V));

impl<K, V> IntoIterator for Map<K, V>
where
    K: Key<K, V>,
{
    type Item = (K, V);
    type IntoIter = IntoIter<K, V>;

    /// An owning iterator visiting all key-value pairs in arbitrary order. The
    /// iterator element type is `(K, V)`.
    ///
    /// # Examples
    ///
    /// ```
    /// use fixed_map::{Key, Map};
    ///
    /// #[derive(Debug, Clone, Copy, PartialEq, Eq, Key)]
    /// enum Key {
    ///     First,
    ///     Second,
    ///     Third,
    /// }
    ///
    /// let mut map = Map::new();
    /// map.insert(Key::First, 1);
    /// map.insert(Key::Third, 3);
    ///
    /// let mut it = map.into_iter();
    /// assert_eq!(it.next(), Some((Key::First, 1)));
    /// assert_eq!(it.next(), Some((Key::Third, 3)));
    /// assert_eq!(it.next(), None);
    ///
    /// let mut it = map.into_iter().rev();
    /// assert_eq!(it.next(), Some((Key::Third, 3)));
    /// assert_eq!(it.next(), Some((Key::First, 1)));
    /// assert_eq!(it.next(), None);
    /// ```
    ///
    /// Into iterator with a composite key:
    ///
    /// ```
    /// use fixed_map::{Key, Map};
    ///
    /// #[derive(Debug, Clone, Copy, PartialEq, Eq, Key)]
    /// enum Key {
    ///     First(bool),
    ///     Second,
    ///     Third,
    /// }
    ///
    /// let mut map = Map::<_, u32>::new();
    /// map.insert(Key::First(false), 1);
    /// map.insert(Key::Third, 3);
    ///
    /// let mut it = map.into_iter();
    /// assert_eq!(it.next(), Some((Key::First(false), 1)));
    /// assert_eq!(it.next(), Some((Key::Third, 3)));
    /// assert_eq!(it.next(), None);
    ///
    /// let mut it = map.into_iter().rev();
    /// assert_eq!(it.next(), Some((Key::Third, 3)));
    /// assert_eq!(it.next(), Some((Key::First(false), 1)));
    /// assert_eq!(it.next(), None);
    /// ```
    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            iter: self.storage.into_iter(),
        }
    }
}

/// An iterator over the keys of a [`Map`].
///
/// This type is created by the [`keys`][Map::keys] method. See its
/// documentation for more.
pub struct Keys<'a, K, V>
where
    K: 'a + Key<K, V>,
    V: 'a,
{
    iter: <K::Storage as Storage<K, V>>::Keys<'a>,
}

iterator!(@identity, {Keys, Keys}, {'a}, [K, V], K, V => K);

impl<'a, K, V> Clone for Keys<'a, K, V>
where
    K: Key<K, V>,
    <K::Storage as Storage<K, V>>::Keys<'a>: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        Self {
            iter: self.iter.clone(),
        }
    }
}

/// An iterator over the values of a [`Map`].
///
/// This type is created by the [`values`][Map::values] method. See its
/// documentation for more.
#[repr(transparent)]
pub struct Values<'a, K, V>
where
    K: 'a + Key<K, V>,
    V: 'a,
{
    iter: <K::Storage as Storage<K, V>>::Values<'a>,
}

iterator!(@identity, {Values, Values}, {'a}, [K, V], K, V => &'a V);

/// A mutable iterator over the values of a [`Map`].
///
/// This type is created by the [`values_mut`][Map::values_mut] method. See its
/// documentation for more.
pub struct ValuesMut<'a, K, V>
where
    K: 'a + Key<K, V>,
    V: 'a,
{
    iter: <K::Storage as Storage<K, V>>::ValuesMut<'a>,
}

iterator!(@identity, {ValuesMut, ValuesMut}, {'a}, [K, V], K, V => &'a mut V);

/// A simple [`FromIterator`] implementation for [`Map`].
///
/// # Example
///
/// ```
/// use fixed_map::{Key, Map};
///
/// #[derive(Debug, Clone, Copy, Key)]
/// enum Key {
///     First,
///     Second,
/// }
///
/// let v = vec![(Key::First, 1), (Key::Second, 2), (Key::First, 3)];
/// let m: Map<_, u8> = v.into_iter().collect();
///
/// let mut n = Map::new();
/// n.insert(Key::Second, 2);
/// n.insert(Key::First, 3);
///
/// assert_eq!(m, n);
/// ```
impl<K, V> FromIterator<(K, V)> for Map<K, V>
where
    K: Key<K, V>,
{
    #[inline]
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = (K, V)>,
    {
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
        return deserializer.deserialize_map(MapVisitor(core::marker::PhantomData));

        struct MapVisitor<K, V>(core::marker::PhantomData<(K, V)>);

        impl<'de, K, V> serde::de::Visitor<'de> for MapVisitor<K, V>
        where
            K: Key<K, V> + serde::de::Deserialize<'de>,
            V: serde::Deserialize<'de>,
        {
            type Value = Map<K, V>;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
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
