//! Contains the fixed `Set` implementation.

use core::fmt;
use core::iter;

use crate::key::Key;
use crate::storage::Storage;

/// A fixed set implemented as a `Map` where the value is `()`.
///
/// # Examples
///
/// ```rust
/// use fixed_map::{Key, Set};
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
/// let mut set = Set::new();
///
/// set.insert(Key::Simple);
/// set.insert(Key::Composite(Part::One));
/// set.insert(Key::String("foo"));
/// set.insert(Key::Number(1));
/// set.insert(Key::Singleton(()));
/// set.insert(Key::Option(None));
/// set.insert(Key::Option(Some(Part::One)));
/// set.insert(Key::Boolean(true));
///
/// assert!(set.contains(Key::Simple));
/// assert!(set.contains(Key::Composite(Part::One)));
/// assert!(!set.contains(Key::Composite(Part::Two)));
/// assert!(set.contains(Key::String("foo")));
/// assert!(!set.contains(Key::String("bar")));
/// assert!(set.contains(Key::Number(1)));
/// assert!(!set.contains(Key::Number(2)));
/// assert!(set.contains(Key::Singleton(())));
/// assert!(set.contains(Key::Option(None)));
/// assert!(set.contains(Key::Option(Some(Part::One))));
/// assert!(!set.contains(Key::Option(Some(Part::Two))));
/// assert!(set.contains(Key::Boolean(true)));
/// assert!(!set.contains(Key::Boolean(false)));
/// ```
pub struct Set<K>
where
    K: Key<K, ()>,
{
    storage: K::Storage,
}

/// A map implementation that uses fixed storage.
///
/// # Examples
///
/// ```rust
/// use fixed_map::{Key, Set};
///
/// #[derive(Clone, Copy, Key)]
/// enum Key {
///     One,
///     Two,
/// }
///
/// let mut m = Set::new();
/// m.insert(Key::One);
///
/// assert_eq!(m.contains(Key::One), true);
/// assert_eq!(m.contains(Key::Two), false);
/// ```
///
/// ```rust
/// use fixed_map::{Key, Set};
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
/// let mut m = Set::new();
/// m.insert(Key::Simple);
/// m.insert(Key::Composite(Part::A));
///
/// assert_eq!(m.contains(Key::Simple), true);
/// assert_eq!(m.contains(Key::Composite(Part::A)), true);
/// assert_eq!(m.contains(Key::Composite(Part::B)), false);
/// ```
impl<K> Set<K>
where
    K: Key<K, ()>,
{
    /// Creates an empty `Set`.
    ///
    /// # Examples
    ///
    /// ```
    /// use fixed_map::{Key, Set};
    ///
    /// #[derive(Clone, Copy, Key)]
    /// enum Key {
    ///     One,
    ///     Two,
    /// }
    ///
    /// let set: Set<Key> = Set::new();
    /// ```
    #[inline]
    pub fn new() -> Set<K> {
        Set {
            storage: K::Storage::default(),
        }
    }

    /// An iterator visiting all values in arbitrary order.
    /// The iterator element type is `K`.
    ///
    /// # Examples
    ///
    /// ```
    /// use fixed_map::{Key, Set};
    ///
    /// #[derive(Debug, Clone, Copy, PartialEq, Eq, Key)]
    /// enum Key {
    ///     One,
    ///     Two,
    ///     Three,
    /// }
    ///
    /// let mut map = Set::new();
    /// map.insert(Key::One);
    /// map.insert(Key::Two);
    ///
    /// assert_eq!(map.iter().collect::<Vec<_>>(), vec![Key::One, Key::Two]);
    /// ```
    #[inline]
    pub fn iter(&self) -> Iter<K> {
        Iter {
            iter: self.storage.iter(),
        }
    }

    /// Returns `true` if the set contains a value.
    /// Returns a reference to the value corresponding to the key.
    ///
    /// # Examples
    ///
    /// ```
    /// use fixed_map::{Key, Set};
    ///
    /// #[derive(Clone, Copy, Key)]
    /// enum Key {
    ///     One,
    ///     Two,
    /// }
    ///
    /// let mut map = Set::new();
    /// map.insert(Key::One);
    /// assert_eq!(map.contains(Key::One), true);
    /// assert_eq!(map.contains(Key::Two), false);
    /// ```
    #[inline]
    pub fn contains(&self, key: K) -> bool {
        self.storage.get(key).is_some()
    }

    /// Adds a value to the set.
    ///
    /// If the set did not have this value present, `true` is returned.
    ///
    /// If the set did have this value present, `false` is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use fixed_map::{Key, Set};
    ///
    /// #[derive(Clone, Copy, Key)]
    /// enum Key {
    ///     One,
    ///     Two,
    /// }
    ///
    /// let mut set = Set::new();
    /// assert_eq!(set.insert(Key::One), true);
    /// assert_eq!(set.is_empty(), false);
    ///
    /// set.insert(Key::Two);
    /// assert_eq!(set.insert(Key::Two), false);
    /// assert_eq!(set.contains(Key::Two), true);
    /// ```
    #[inline]
    pub fn insert(&mut self, value: K) -> bool {
        self.storage.insert(value, ()).is_none()
    }

    /// Removes a value from the set. Returns `true` if the value was
    /// present in the set.
    ///
    /// # Examples
    ///
    /// ```
    /// use fixed_map::{Key, Set};
    ///
    /// #[derive(Clone, Copy, Key)]
    /// enum Key {
    ///     One,
    ///     Two,
    /// }
    ///
    /// let mut set = Set::new();
    /// set.insert(Key::One);
    /// assert_eq!(set.remove(Key::One), true);
    /// assert_eq!(set.remove(Key::One), false);
    /// ```
    #[inline]
    pub fn remove(&mut self, key: K) -> bool {
        self.storage.remove(key).is_some()
    }

    /// Clears the set, removing all values.
    ///
    /// # Examples
    ///
    /// ```
    /// use fixed_map::{Key, Set};
    ///
    /// #[derive(Clone, Copy, Key)]
    /// enum Key {
    ///     One,
    ///     Two,
    /// }
    ///
    /// let mut set = Set::new();
    /// set.insert(Key::One);
    /// set.clear();
    /// assert!(set.is_empty());
    /// ```
    #[inline]
    pub fn clear(&mut self) {
        self.storage.clear()
    }

    /// Returns true if the set contains no elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use fixed_map::{Key, Set};
    ///
    /// #[derive(Clone, Copy, Key)]
    /// enum Key {
    ///     One,
    ///     Two,
    /// }
    ///
    /// let mut set = Set::new();
    /// assert!(set.is_empty());
    /// set.insert(Key::One);
    /// assert!(!set.is_empty());
    /// ```
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.storage.iter().next().is_none()
    }

    /// Returns the number of elements in the set.
    ///
    /// # Examples
    ///
    /// ```
    /// use fixed_map::{Key, Set};
    ///
    /// #[derive(Clone, Copy, Key)]
    /// enum Key {
    ///     One,
    ///     Two,
    /// }
    ///
    /// let mut set = Set::new();
    /// assert_eq!(set.len(), 0);
    /// set.insert(Key::One);
    /// assert_eq!(set.len(), 1);
    /// ```
    #[inline]
    pub fn len(&self) -> usize {
        self.storage.iter().count()
    }
}

impl<K> Clone for Set<K>
where
    K: Key<K, ()>,
    K::Storage: Clone,
{
    #[inline]
    fn clone(&self) -> Set<K> {
        Set {
            storage: self.storage.clone(),
        }
    }
}

impl<K> Default for Set<K>
where
    K: Key<K, ()>,
{
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<K> fmt::Debug for Set<K>
where
    K: Key<K, ()> + fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut debug_set = f.debug_set();
        for k in self.iter() {
            debug_set.entry(&k);
        }
        debug_set.finish()
    }
}

impl<K> PartialEq for Set<K>
where
    K: Key<K, ()>,
    K::Storage: PartialEq,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.storage == other.storage
    }
}

impl<K> Eq for Set<K>
where
    K: Key<K, ()>,
    K::Storage: Eq,
{
}

/// An iterator over the items of a `Set`.
///
/// This `struct` is created by the [`iter`] method on [`Set`].
/// See its documentation for more.
///
/// [`iter`]: struct.Set.html#method.iter
/// [`Set`]: struct.Set.html
pub struct Iter<'a, K>
where
    K: 'a + Key<K, ()>,
{
    iter: <<K as Key<K, ()>>::Storage as Storage<K, ()>>::Iter<'a>,
}

impl<'a, K> Iterator for Iter<'a, K>
where
    K: Key<K, ()>,
{
    type Item = K;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|(k, _)| k)
    }
}

impl<'a, K> IntoIterator for &'a Set<K>
where
    K: Key<K, ()>,
{
    type Item = K;
    type IntoIter = Iter<'a, K>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// An owning iterator over the items of a `Set`.
///
/// This `struct` is created by the [`into_iter`] method on [`Set`].
/// See its documentation for more.
///
/// [`into_iter`]: struct.Set.html#method.iter
/// [`Set`]: struct.Set.html
pub struct IntoIter<K>
where
    K: Key<K, ()>,
{
    iter: <<K as Key<K, ()>>::Storage as Storage<K, ()>>::IntoIter,
}

impl<K> Iterator for IntoIter<K>
where
    K: Key<K, ()>,
{
    type Item = K;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|(k, _)| k)
    }
}

impl<K> IntoIterator for Set<K>
where
    K: Key<K, ()>,
{
    type Item = K;
    type IntoIter = IntoIter<K>;

    /// An iterator visiting all values in arbitrary order.
    /// The iterator element type is `K`.
    ///
    /// # Examples
    ///
    /// ```
    /// use fixed_map::{Key, Set};
    ///
    /// #[derive(Debug, Clone, Copy, PartialEq, Eq, Key)]
    /// enum Key {
    ///     One,
    ///     Two,
    ///     Three,
    /// }
    ///
    /// let mut map = Set::new();
    /// map.insert(Key::One);
    /// map.insert(Key::Two);
    ///
    /// assert_eq!(map.into_iter().collect::<Vec<_>>(), vec![Key::One, Key::Two]);
    /// ```
    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            iter: self.storage.into_iter(),
        }
    }
}

impl<K> iter::FromIterator<K> for Set<K>
where
    K: Key<K, ()>,
{
    #[inline]
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = K>,
    {
        let mut set = Self::new();
        for k in iter {
            set.insert(k);
        }
        set
    }
}

#[cfg(feature = "serde")]
impl<K> serde::Serialize for Set<K>
where
    K: Key<K, ()> + serde::Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeSeq as _;

        let mut seq = serializer.serialize_seq(Some(self.len()))?;

        for v in self.iter() {
            seq.serialize_element(&v)?;
        }

        seq.end()
    }
}

#[cfg(feature = "serde")]
impl<'de, K> serde::de::Deserialize<'de> for Set<K>
where
    K: Key<K, ()> + serde::de::Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        return deserializer.deserialize_seq(SeqVisitor(core::marker::PhantomData));

        struct SeqVisitor<K>(core::marker::PhantomData<K>);

        impl<'de, K> serde::de::Visitor<'de> for SeqVisitor<K>
        where
            K: Key<K, ()> + serde::de::Deserialize<'de>,
        {
            type Value = Set<K>;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a sequence")
            }

            #[inline]
            fn visit_seq<V>(self, mut visitor: V) -> Result<Self::Value, V::Error>
            where
                V: serde::de::SeqAccess<'de>,
            {
                let mut set = Set::new();

                while let Some(elem) = visitor.next_element()? {
                    set.insert(elem);
                }

                Ok(set)
            }
        }
    }
}
