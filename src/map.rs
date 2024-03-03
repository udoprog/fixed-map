//! Contains the fixed [`Map`] implementation.

mod entry;
pub use self::entry::Entry;

pub(crate) mod storage;
pub use self::storage::{MapStorage, OccupiedEntry, VacantEntry};

use core::cmp::{Ord, Ordering, PartialOrd};
use core::fmt;
use core::hash::{Hash, Hasher};

use crate::Key;

/// The iterator produced by [`Map::iter`].
pub type Iter<'a, K, V> = <<K as Key>::MapStorage<V> as MapStorage<K, V>>::Iter<'a>;

/// The iterator produced by [`Map::keys`].
pub type Keys<'a, K, V> = <<K as Key>::MapStorage<V> as MapStorage<K, V>>::Keys<'a>;

/// The iterator produced by [`Map::values`].
pub type Values<'a, K, V> = <<K as Key>::MapStorage<V> as MapStorage<K, V>>::Values<'a>;

/// The iterator produced by [`Map::iter`].
pub type IterMut<'a, K, V> = <<K as Key>::MapStorage<V> as MapStorage<K, V>>::IterMut<'a>;

/// The iterator produced by [`Map::values_mut`].
pub type ValuesMut<'a, K, V> = <<K as Key>::MapStorage<V> as MapStorage<K, V>>::ValuesMut<'a>;

/// The iterator produced by [`Map::into_iter`].
pub type IntoIter<K, V> = <<K as Key>::MapStorage<V> as MapStorage<K, V>>::IntoIter;

/// A fixed map with storage specialized through the [`Key`] trait.
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
/// enum MyKey {
///     Simple,
///     Composite(Part),
///     # #[cfg(feature = "hashbrown")]
///     String(&'static str),
///     # #[cfg(feature = "hashbrown")]
///     Number(u32),
///     Singleton(()),
///     Option(Option<Part>),
///     Boolean(bool),
/// }
///
/// let mut map = Map::new();
///
/// map.insert(MyKey::Simple, 1);
/// map.insert(MyKey::Composite(Part::One), 2);
/// # #[cfg(feature = "hashbrown")]
/// map.insert(MyKey::String("foo"), 3);
/// # #[cfg(feature = "hashbrown")]
/// map.insert(MyKey::Number(1), 4);
/// map.insert(MyKey::Singleton(()), 5);
/// map.insert(MyKey::Option(None), 6);
/// map.insert(MyKey::Option(Some(Part::One)), 7);
/// map.insert(MyKey::Boolean(true), 8);
///
/// assert_eq!(map.get(MyKey::Simple), Some(&1));
/// assert_eq!(map.get(MyKey::Composite(Part::One)), Some(&2));
/// assert_eq!(map.get(MyKey::Composite(Part::Two)), None);
/// # #[cfg(feature = "hashbrown")]
/// assert_eq!(map.get(MyKey::String("foo")), Some(&3));
/// # #[cfg(feature = "hashbrown")]
/// assert_eq!(map.get(MyKey::String("bar")), None);
/// # #[cfg(feature = "hashbrown")]
/// assert_eq!(map.get(MyKey::Number(1)), Some(&4));
/// # #[cfg(feature = "hashbrown")]
/// assert_eq!(map.get(MyKey::Number(2)), None);
/// assert_eq!(map.get(MyKey::Singleton(())), Some(&5));
/// assert_eq!(map.get(MyKey::Option(None)), Some(&6));
/// assert_eq!(map.get(MyKey::Option(Some(Part::One))), Some(&7));
/// assert_eq!(map.get(MyKey::Option(Some(Part::Two))), None);
/// assert_eq!(map.get(MyKey::Boolean(true)), Some(&8));
/// assert_eq!(map.get(MyKey::Boolean(false)), None);
/// ```
///
/// Storing references:
///
/// ```rust
/// use fixed_map::{Key, Map};
///
/// #[derive(Debug, Clone, Copy, Key)]
/// enum MyKey {
///     First,
///     Second,
/// }
///
/// let mut map = Map::new();
/// let a = 42u32;
///
/// map.insert(MyKey::First, &a);
///
/// assert_eq!(map.values().cloned().collect::<Vec<_>>(), vec![&42u32]);
/// ```
#[repr(transparent)]
pub struct Map<K, V>
where
    K: Key,
{
    storage: K::MapStorage<V>,
}

/// A map implementation that uses fixed storage.
///
/// # Examples
///
/// ```rust
/// use fixed_map::{Key, Map};
///
/// #[derive(Clone, Copy, Key)]
/// enum MyKey {
///     One,
///     Two,
/// }
///
/// let mut m = Map::new();
/// m.insert(MyKey::One, 1);
///
/// assert_eq!(m.get(MyKey::One), Some(&1));
/// assert_eq!(m.get(MyKey::Two), None);
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
/// enum MyKey {
///     Simple,
///     Composite(Part),
/// }
///
/// let mut m = Map::new();
/// m.insert(MyKey::Simple, 1);
/// m.insert(MyKey::Composite(Part::A), 2);
///
/// assert_eq!(m.get(MyKey::Simple), Some(&1));
/// assert_eq!(m.get(MyKey::Composite(Part::A)), Some(&2));
/// assert_eq!(m.get(MyKey::Composite(Part::B)), None);
/// ```
impl<K, V> Map<K, V>
where
    K: Key,
{
    /// Creates an empty [`Map`].
    ///
    /// # Examples
    ///
    /// ```
    /// use fixed_map::{Key, Map};
    ///
    /// #[derive(Clone, Copy, Key)]
    /// enum MyKey {
    ///     One,
    ///     Two,
    /// }
    ///
    /// let mut map: Map<MyKey, i32> = Map::new();
    /// ```
    #[inline]
    #[must_use]
    pub fn new() -> Map<K, V> {
        Map {
            storage: K::MapStorage::empty(),
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
    /// enum MyKey {
    ///     One,
    ///     Two,
    ///     Three,
    /// }
    ///
    /// let mut map = Map::new();
    /// map.insert(MyKey::One, 1);
    /// map.insert(MyKey::Two, 2);
    ///
    /// assert_eq!(map.iter().collect::<Vec<_>>(), vec![(MyKey::One, &1), (MyKey::Two, &2)]);
    /// ```
    #[inline]
    pub fn iter(&self) -> Iter<'_, K, V> {
        self.storage.iter()
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
    /// pub enum MyKey {
    ///     First,
    ///     Second,
    ///     Third,
    /// }
    ///
    /// let mut map = Map::new();
    /// map.insert(MyKey::First, 1);
    /// map.insert(MyKey::Second, 2);
    ///
    /// assert!(map.keys().eq([MyKey::First, MyKey::Second]));
    /// assert!(map.keys().rev().eq([MyKey::Second, MyKey::First]));
    /// ```
    ///
    /// Using a composite key:
    ///
    /// ```
    /// use fixed_map::{Key, Map};
    ///
    /// #[derive(Debug, Clone, Copy, PartialEq, Eq, Key)]
    /// pub enum MyKey {
    ///     First,
    ///     Second(bool),
    ///     Third,
    /// }
    ///
    /// let mut map = Map::new();
    /// map.insert(MyKey::First, 1);
    /// map.insert(MyKey::Second(false), 2);
    ///
    /// dbg!(map.keys().collect::<Vec<_>>());
    ///
    /// assert!(map.keys().eq([MyKey::First, MyKey::Second(false)]));
    /// assert!(map.keys().rev().eq([MyKey::Second(false), MyKey::First]));
    /// ```
    #[inline]
    pub fn keys(&self) -> Keys<'_, K, V> {
        self.storage.keys()
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
    /// pub enum MyKey {
    ///     First,
    ///     Second,
    ///     Third,
    /// }
    ///
    /// let mut map = Map::new();
    /// map.insert(MyKey::First, 1);
    /// map.insert(MyKey::Second, 2);
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
    /// pub enum MyKey {
    ///     First(bool),
    ///     Second,
    ///     Third,
    /// }
    ///
    /// let mut map = Map::new();
    /// map.insert(MyKey::First(false), 1);
    /// map.insert(MyKey::Second, 2);
    ///
    /// assert!(map.values().copied().eq([1, 2]));
    /// assert!(map.values().rev().copied().eq([2, 1]));
    /// ```
    #[inline]
    pub fn values(&self) -> Values<'_, K, V> {
        self.storage.values()
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
    /// enum MyKey {
    ///     First,
    ///     Second,
    /// }
    ///
    /// let mut map = Map::new();
    /// map.insert(MyKey::First, 1);
    /// map.insert(MyKey::Second, 2);
    ///
    /// // Update all values
    /// for (_, val) in map.iter_mut() {
    ///     *val *= 2;
    /// }
    ///
    /// assert!(map.iter().eq([(MyKey::First, &2), (MyKey::Second, &4)]));
    /// ```
    ///
    /// Using a composite key:
    ///
    /// ```
    /// use fixed_map::{Key, Map};
    ///
    /// #[derive(Debug, Clone, Copy, PartialEq, Eq, Key)]
    /// enum MyKey {
    ///     First(bool),
    ///     Second,
    /// }
    ///
    /// let mut map = Map::new();
    /// map.insert(MyKey::First(true), 1);
    /// map.insert(MyKey::Second, 2);
    ///
    /// // Update all values
    /// for (_, val) in map.iter_mut() {
    ///     *val *= 2;
    /// }
    ///
    /// assert!(map.iter().eq([(MyKey::First(true), &2), (MyKey::Second, &4)]));
    /// ```
    #[inline]
    pub fn iter_mut(&mut self) -> IterMut<'_, K, V> {
        self.storage.iter_mut()
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
    /// pub enum MyKey {
    ///     First,
    ///     Second,
    /// }
    ///
    /// let mut map = Map::new();
    /// map.insert(MyKey::First, 2);
    /// map.insert(MyKey::Second, 5);
    ///
    /// for (index, val) in map.values_mut().enumerate() {
    ///     *val *= index + 1;
    /// }
    ///
    /// assert!(map.values().copied().eq([2, 10]));
    ///
    /// let mut map = Map::new();
    /// map.insert(MyKey::First, 2);
    /// map.insert(MyKey::Second, 5);
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
    /// pub enum MyKey {
    ///     First(bool),
    ///     Second,
    /// }
    ///
    /// let mut map = Map::new();
    /// map.insert(MyKey::First(false), 2);
    /// map.insert(MyKey::Second, 5);
    ///
    /// for (index, val) in map.values_mut().enumerate() {
    ///     *val *= index + 1;
    /// }
    ///
    /// assert!(map.values().copied().eq([2, 10]));
    ///
    /// let mut map = Map::new();
    /// map.insert(MyKey::First(false), 2);
    /// map.insert(MyKey::Second, 5);
    ///
    /// for (index, val) in map.values_mut().rev().enumerate() {
    ///     *val *= index + 1;
    /// }
    ///
    /// assert!(map.values().copied().eq([4, 5]));
    /// ```
    #[inline]
    pub fn values_mut(&mut self) -> ValuesMut<'_, K, V> {
        self.storage.values_mut()
    }

    /// Returns `true` if the map currently contains the given key.
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
    /// let mut map = Map::new();
    /// map.insert(MyKey::First, "a");
    /// assert_eq!(map.contains_key(MyKey::First), true);
    /// assert_eq!(map.contains_key(MyKey::Second), false);
    /// ```
    #[inline]
    pub fn contains_key(&self, key: K) -> bool {
        self.storage.contains_key(key)
    }

    /// Returns a reference to the value corresponding to the key.
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
    /// let mut map = Map::new();
    /// map.insert(MyKey::First, "a");
    /// assert_eq!(map.get(MyKey::First).copied(), Some("a"));
    /// assert_eq!(map.get(MyKey::Second), None);
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
    /// let mut map = Map::new();
    /// map.insert(MyKey::First(true), "a");
    /// assert_eq!(map.get(MyKey::First(true)).copied(), Some("a"));
    /// assert_eq!(map.get(MyKey::Second), None);
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
    /// enum MyKey {
    ///     First,
    ///     Second,
    /// }
    ///
    /// let mut map = Map::new();
    /// map.insert(MyKey::First, "a");
    ///
    /// if let Some(x) = map.get_mut(MyKey::First) {
    ///     *x = "b";
    /// }
    ///
    /// assert_eq!(map.get(MyKey::First).copied(), Some("b"));
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
    ///     Second(()),
    ///     Third,
    /// }
    ///
    /// let mut map = Map::new();
    /// map.insert(MyKey::First(true), "a");
    ///
    /// if let Some(x) = map.get_mut(MyKey::First(true)) {
    ///     *x = "b";
    /// }
    ///
    /// assert_eq!(map.get(MyKey::First(true)).copied(), Some("b"));
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
    /// enum MyKey {
    ///     One,
    ///     Two,
    /// }
    ///
    /// let mut map = Map::new();
    /// assert_eq!(map.insert(MyKey::One, "a"), None);
    /// assert_eq!(map.is_empty(), false);
    ///
    /// map.insert(MyKey::Two, "b");
    /// assert_eq!(map.insert(MyKey::Two, "c"), Some("b"));
    /// assert_eq!(map.get(MyKey::Two), Some(&"c"));
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
    /// enum MyKey {
    ///     One,
    ///     Two,
    /// }
    ///
    /// let mut map = Map::new();
    /// map.insert(MyKey::One, "a");
    /// assert_eq!(map.remove(MyKey::One), Some("a"));
    /// assert_eq!(map.remove(MyKey::One), None);
    /// ```
    #[inline]
    pub fn remove(&mut self, key: K) -> Option<V> {
        self.storage.remove(key)
    }

    /// Retains only the elements specified by the predicate.
    ///
    /// In other words, remove all pairs (k, v) for which f(k, &mut v) returns false.
    /// The elements are visited in unsorted (and unspecified) order.
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
    /// map.insert(MyKey::First, 42);
    /// map.insert(MyKey::Second, -10);
    ///
    /// map.retain(|k, v| *v > 0);
    ///
    /// assert_eq!(map.len(), 1);
    /// assert_eq!(map.get(MyKey::First), Some(&42));
    /// assert_eq!(map.get(MyKey::Second), None);
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
    /// map.insert(MyKey::First(true), 42);
    /// map.insert(MyKey::First(false), -31);
    /// map.insert(MyKey::Second, 100);
    ///
    /// let mut other = map.clone();
    /// assert_eq!(map.len(), 3);
    ///
    /// map.retain(|k, v| *v > 0);
    ///
    /// assert_eq!(map.len(), 2);
    /// assert_eq!(map.get(MyKey::First(true)), Some(&42));
    /// assert_eq!(map.get(MyKey::First(false)), None);
    /// assert_eq!(map.get(MyKey::Second), Some(&100));
    ///
    /// other.retain(|k, v| matches!(k, MyKey::First(_)));
    ///
    /// assert_eq!(other.len(), 2);
    /// assert_eq!(other.get(MyKey::First(true)), Some(&42));
    /// assert_eq!(other.get(MyKey::First(false)), Some(&-31));
    /// assert_eq!(other.get(MyKey::Second), None);
    /// ```
    #[inline]
    pub fn retain<F>(&mut self, f: F)
    where
        F: FnMut(K, &mut V) -> bool,
    {
        self.storage.retain(f);
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
    /// enum MyKey {
    ///     One,
    ///     Two,
    /// }
    ///
    /// let mut map = Map::new();
    /// map.insert(MyKey::One, "a");
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
    /// enum MyKey {
    ///     First,
    ///     Second,
    /// }
    ///
    /// let mut map = Map::new();
    /// assert!(map.is_empty());
    /// map.insert(MyKey::First, 1);
    /// assert!(!map.is_empty());
    /// ```
    ///
    /// An empty key:
    ///
    /// ```
    /// use fixed_map::{Key, Map};
    ///
    /// #[derive(Clone, Copy, Key)]
    /// enum MyKey {}
    ///
    /// let map = Map::<MyKey, u32>::new();
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
    /// enum MyKey {
    ///     First,
    ///     Second,
    /// }
    ///
    /// let mut map: Map<MyKey, i32> = Map::new();
    /// assert_eq!(map.len(), 0);
    ///
    /// map.insert(MyKey::First, 42);
    /// assert_eq!(map.len(), 1);
    ///
    /// map.insert(MyKey::First, 42);
    /// assert_eq!(map.len(), 1);
    ///
    /// map.remove(MyKey::First);
    /// assert_eq!(map.len(), 0);
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
    /// assert_eq!(map.len(), 0);
    ///
    /// map.insert(MyKey::First(true), 42);
    /// assert_eq!(map.len(), 1);
    ///
    /// map.insert(MyKey::First(false), 42);
    /// assert_eq!(map.len(), 2);
    ///
    /// map.remove(MyKey::First(true));
    /// assert_eq!(map.len(), 1);
    /// ```
    #[inline]
    pub fn len(&self) -> usize {
        self.storage.len()
    }

    /// Gets the given key’s corresponding [`Entry`] in the [`Map`] for in-place manipulation.
    ///
    /// # Examples
    ///
    /// ```
    /// use fixed_map::{Key, Map};
    ///
    /// #[derive(Clone, Copy, Key)]
    /// enum MyKey {
    ///     Even,
    ///     Odd,
    /// }
    ///
    /// let mut map: Map<MyKey, u32> = Map::new();
    ///
    /// for n in [3, 45, 3, 23, 2, 10, 59, 11, 51, 70] {
    ///     map
    ///         .entry(if n % 2 == 0 { MyKey::Even } else { MyKey::Odd })
    ///         .and_modify(|x| *x += 1)
    ///         .or_insert(1);
    /// }
    ///
    /// assert_eq!(map.get(MyKey::Even), Some(&3));
    /// assert_eq!(map.get(MyKey::Odd), Some(&7));
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
    /// let mut map: Map<MyKey, Vec<i32>> = Map::new();
    ///
    /// map.entry(MyKey::First(true)).or_default().push(1);
    /// map.entry(MyKey::Second).or_insert_with(|| vec![2; 8]).truncate(4);
    ///
    /// assert_eq!(map.get(MyKey::First(true)), Some(&vec![1]));
    /// assert_eq!(map.get(MyKey::Second), Some(&vec![2; 4]));
    /// ```
    #[inline]
    pub fn entry(&mut self, key: K) -> Entry<'_, K::MapStorage<V>, K, V> {
        K::MapStorage::entry(&mut self.storage, key)
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
/// enum MyKey {
///     First(bool),
///     Second,
/// }
///
/// let mut a = Map::new();
/// a.insert(MyKey::First(true), 1);
/// let mut b = a.clone();
/// b.insert(MyKey::Second, 2);
///
/// assert_ne!(a, b);
///
/// assert_eq!(a.get(MyKey::First(true)), Some(&1));
/// assert_eq!(a.get(MyKey::Second), None);
///
/// assert_eq!(b.get(MyKey::First(true)), Some(&1));
/// assert_eq!(b.get(MyKey::Second), Some(&2));
/// ```
impl<K, V> Clone for Map<K, V>
where
    K: Key,
    K::MapStorage<V>: Clone,
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
/// enum MyKey {
///     First,
///     Second,
/// }
///
/// let mut a = Map::new();
/// a.insert(MyKey::First, 1);
/// let mut b = a;
/// b.insert(MyKey::Second, 2);
///
/// assert_ne!(a, b);
///
/// assert_eq!(a.get(MyKey::First), Some(&1));
/// assert_eq!(a.get(MyKey::Second), None);
///
/// assert_eq!(b.get(MyKey::First), Some(&1));
/// assert_eq!(b.get(MyKey::Second), Some(&2));
/// ```
impl<K, V> Copy for Map<K, V>
where
    K: Key,
    K::MapStorage<V>: Copy,
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
/// enum MyKey {
///     First,
///     Second,
/// }
///
/// let a = Map::<MyKey, u32>::default();
/// let b = Map::<MyKey, u32>::new();
///
/// assert_eq!(a, b);
/// ```
impl<K, V> Default for Map<K, V>
where
    K: Key,
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
/// enum MyKey {
///     First,
///     Second,
/// }
///
/// let mut a = Map::new();
/// a.insert(MyKey::First, 42);
///
/// assert_eq!("{First: 42}", format!("{:?}", a));
/// ```
impl<K, V> fmt::Debug for Map<K, V>
where
    K: Key + fmt::Debug,
    V: fmt::Debug,
{
    #[inline]
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
/// enum MyKey {
///     First,
///     Second,
/// }
///
/// let mut a = Map::new();
/// a.insert(MyKey::First, 42);
/// // Note: `a` is Copy since it's using a simple key.
/// let mut b = a;
///
/// assert_eq!(a, b);
///
/// b.insert(MyKey::Second, 42);
/// assert_ne!(a, b);
/// ```
///
/// Using a composite key:
///
/// ```
/// use fixed_map::{Key, Map};
///
/// #[derive(Debug, Clone, Copy, Key)]
/// enum MyKey {
///     First(bool),
///     Second,
/// }
///
/// let mut a = Map::new();
/// a.insert(MyKey::First(true), 42);
/// let mut b = a.clone();
///
/// assert_eq!(a, b);
///
/// b.insert(MyKey::Second, 42);
/// assert_ne!(a, b);
/// ```
impl<K, V> PartialEq for Map<K, V>
where
    K: Key,
    K::MapStorage<V>: PartialEq,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.storage == other.storage
    }
}

impl<K, V> Eq for Map<K, V>
where
    K: Key,
    K::MapStorage<V>: Eq,
{
}

/// [`Hash`] implementation for a [`Set`].
///
/// [`Set`]: crate::Set
///
/// # Examples
///
/// ```
/// use std::collections::HashSet;
///
/// use fixed_map::{Key, Map};
///
/// #[derive(Debug, Clone, Copy, Hash, Key)]
/// enum MyKey {
///     First,
///     Second,
/// }
///
/// let mut a = Map::new();
/// a.insert(MyKey::First, 1);
///
/// let mut set = HashSet::new();
/// set.insert(a);
/// ```
///
/// Using a composite key:
///
/// ```
/// use std::collections::HashSet;
///
/// use fixed_map::{Key, Map};
///
/// #[derive(Debug, Clone, Copy, Hash, Key)]
/// enum MyKey {
///     First(bool),
///     Second,
/// }
///
/// let mut a = Map::new();
/// a.insert(MyKey::First(true), 1);
///
/// // TODO: support this
/// // let mut set = HashSet::new();
/// // set.insert(a);
/// ```
impl<K, V> Hash for Map<K, V>
where
    K: Key,
    K::MapStorage<V>: Hash,
{
    #[inline]
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.storage.hash(state);
    }
}

/// [`PartialOrd`] implementation for a [`Map`].
///
/// For more details on ordering, see the [`Key`] documentation.
///
/// # Examples
///
/// ```
/// use fixed_map::{Key, Map};
///
/// #[derive(Debug, Clone, Copy, Hash, Key)]
/// enum MyKey {
///     First,
///     Second,
///     Third,
/// }
///
/// let mut a = Map::new();
/// a.insert(MyKey::First, 1);
///
/// let mut b = Map::new();
/// b.insert(MyKey::Third, 1);
///
/// assert!(a < b);
///
/// let mut empty = Map::new();
/// assert!(empty < a);
/// assert!(empty < b);
/// ```
///
/// Using a composite key:
///
/// ```
/// use fixed_map::{Key, Map};
///
/// #[derive(Debug, Clone, Copy, Hash, Key)]
/// enum MyKey {
///     First(bool),
///     Second,
/// }
///
/// let mut a = Map::new();
/// a.insert(MyKey::First(true), 1);
///
/// let mut b = Map::new();
/// b.insert(MyKey::Second, 1);
///
/// // TODO: support this
/// // assert!(a < b);
/// ```
impl<K, V> PartialOrd for Map<K, V>
where
    K: Key,
    K::MapStorage<V>: PartialOrd,
{
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.storage.partial_cmp(&other.storage)
    }

    #[inline]
    fn lt(&self, other: &Self) -> bool {
        self.storage.lt(&other.storage)
    }

    #[inline]
    fn le(&self, other: &Self) -> bool {
        self.storage.le(&other.storage)
    }

    #[inline]
    fn gt(&self, other: &Self) -> bool {
        self.storage.gt(&other.storage)
    }

    #[inline]
    fn ge(&self, other: &Self) -> bool {
        self.storage.ge(&other.storage)
    }
}

/// [`Ord`] implementation for a [`Map`].
///
/// For more details on ordering, see the [`Key`] documentation.
///
/// # Examples
///
/// ```
/// use fixed_map::{Key, Map};
///
/// #[derive(Debug, Clone, Copy, Hash, Key)]
/// enum MyKey {
///     First,
///     Second,
/// }
///
/// let mut a = Map::new();
/// a.insert(MyKey::First, 1);
///
/// let mut b = Map::new();
/// b.insert(MyKey::Second, 1);
///
/// let mut list = vec![b, a];
/// list.sort();
///
/// assert_eq!(list, [a, b]);
/// ```
///
/// Using a composite key:
///
/// ```
/// use fixed_map::{Key, Map};
///
/// #[derive(Debug, Clone, Copy, Hash, Key)]
/// enum MyKey {
///     First(bool),
///     Second,
/// }
///
/// let mut a = Map::new();
/// a.insert(MyKey::First(true), 1);
///
/// let mut b = Map::new();
/// b.insert(MyKey::Second, 1);
///
/// // TODO: support this
/// // let mut list = vec![a, b];
/// // list.sort();
/// ```
impl<K, V> Ord for Map<K, V>
where
    K: Key,
    K::MapStorage<V>: Ord,
{
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        self.storage.cmp(&other.storage)
    }

    #[inline]
    fn max(self, other: Self) -> Self {
        Self {
            storage: self.storage.max(other.storage),
        }
    }

    #[inline]
    fn min(self, other: Self) -> Self {
        Self {
            storage: self.storage.min(other.storage),
        }
    }

    #[inline]
    fn clamp(self, min: Self, max: Self) -> Self {
        Self {
            storage: self.storage.clamp(min.storage, max.storage),
        }
    }
}

impl<'a, K, V> IntoIterator for &'a Map<K, V>
where
    K: Key,
{
    type Item = (K, &'a V);
    type IntoIter = Iter<'a, K, V>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// [`IntoIterator`] implementation which uses [`Map::iter_mut`]. See its
/// documentation for more.
impl<'a, K, V> IntoIterator for &'a mut Map<K, V>
where
    K: Key,
{
    type Item = (K, &'a mut V);
    type IntoIter = IterMut<'a, K, V>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

/// Produce an owning iterator visiting all key-value pairs of the [`Map`] in an
/// arbitrary order. The iterator element type is `(K, V)`.
///
/// # Examples
///
/// ```
/// use fixed_map::{Key, Map};
///
/// #[derive(Debug, Clone, Copy, PartialEq, Eq, Key)]
/// enum MyKey {
///     First,
///     Second,
///     Third,
/// }
///
/// let mut map = Map::new();
/// map.insert(MyKey::First, 1);
/// map.insert(MyKey::Third, 3);
///
/// let mut it = map.into_iter();
/// assert_eq!(it.next(), Some((MyKey::First, 1)));
/// assert_eq!(it.next(), Some((MyKey::Third, 3)));
/// assert_eq!(it.next(), None);
///
/// let mut it = map.into_iter().rev();
/// assert_eq!(it.next(), Some((MyKey::Third, 3)));
/// assert_eq!(it.next(), Some((MyKey::First, 1)));
/// assert_eq!(it.next(), None);
/// ```
///
/// Into iterator with a composite key:
///
/// ```
/// use fixed_map::{Key, Map};
///
/// #[derive(Debug, Clone, Copy, PartialEq, Eq, Key)]
/// enum MyKey {
///     First(bool),
///     Second,
///     Third,
/// }
///
/// let mut map = Map::<_, u32>::new();
/// map.insert(MyKey::First(false), 1);
/// map.insert(MyKey::Third, 3);
///
/// let mut it = map.into_iter();
/// assert_eq!(it.next(), Some((MyKey::First(false), 1)));
/// assert_eq!(it.next(), Some((MyKey::Third, 3)));
/// assert_eq!(it.next(), None);
///
/// let mut it = map.into_iter().rev();
/// assert_eq!(it.next(), Some((MyKey::Third, 3)));
/// assert_eq!(it.next(), Some((MyKey::First(false), 1)));
/// assert_eq!(it.next(), None);
/// ```
impl<K, V> IntoIterator for Map<K, V>
where
    K: Key,
{
    type Item = (K, V);
    type IntoIter = IntoIter<K, V>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.storage.into_iter()
    }
}

/// A simple [`FromIterator`] implementation for [`Map`].
///
/// # Example
///
/// ```
/// use fixed_map::{Key, Map};
///
/// #[derive(Debug, Clone, Copy, Key)]
/// enum MyKey {
///     First,
///     Second,
/// }
///
/// let v = vec![(MyKey::First, 1), (MyKey::Second, 2), (MyKey::First, 3)];
/// let m: Map<_, u8> = v.into_iter().collect();
///
/// let mut n = Map::new();
/// n.insert(MyKey::Second, 2);
/// n.insert(MyKey::First, 3);
///
/// assert_eq!(m, n);
/// ```
impl<K, V> FromIterator<(K, V)> for Map<K, V>
where
    K: Key,
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
    K: Key + serde::Serialize,
    V: serde::Serialize,
{
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeMap as _;

        let mut map = serializer.serialize_map(Some(self.len()))?;

        for (k, v) in self {
            map.serialize_entry(&k, v)?;
        }

        map.end()
    }
}

#[cfg(feature = "serde")]
impl<'de, K, V> serde::de::Deserialize<'de> for Map<K, V>
where
    K: Key + serde::de::Deserialize<'de>,
    V: serde::Deserialize<'de>,
{
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct MapVisitor<K, V>(core::marker::PhantomData<(K, V)>);

        impl<'de, K, V> serde::de::Visitor<'de> for MapVisitor<K, V>
        where
            K: Key + serde::de::Deserialize<'de>,
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

        deserializer.deserialize_map(MapVisitor(core::marker::PhantomData))
    }
}
