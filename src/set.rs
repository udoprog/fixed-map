//! Contains the fixed [`Set`] implementation.

use core::cmp::Ordering;
use core::fmt;
use core::hash::{Hash, Hasher};
use core::iter;

use crate::key::Key;
use crate::storage::Storage;

/// The iterator produced by [`Set::into_iter`].
pub type IntoIter<K> =
    iter::Map<<<K as Key>::Storage<()> as Storage<K, ()>>::IntoIter, fn((K, ())) -> K>;

/// The iterator produced by [`Set::iter`].
pub type Iter<'a, K> = <<K as Key>::Storage<()> as Storage<K, ()>>::Keys<'a>;

/// A fixed set with storage specialized through the [`Key`] trait.
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
///     # #[cfg(feature = "map")]
///     String(&'static str),
///     # #[cfg(feature = "map")]
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
/// # #[cfg(feature = "map")]
/// set.insert(Key::String("foo"));
/// # #[cfg(feature = "map")]
/// set.insert(Key::Number(1));
/// set.insert(Key::Singleton(()));
/// set.insert(Key::Option(None));
/// set.insert(Key::Option(Some(Part::One)));
/// set.insert(Key::Boolean(true));
///
/// assert!(set.contains(Key::Simple));
/// assert!(set.contains(Key::Composite(Part::One)));
/// assert!(!set.contains(Key::Composite(Part::Two)));
/// # #[cfg(feature = "map")]
/// assert!(set.contains(Key::String("foo")));
/// # #[cfg(feature = "map")]
/// assert!(!set.contains(Key::String("bar")));
/// # #[cfg(feature = "map")]
/// assert!(set.contains(Key::Number(1)));
/// # #[cfg(feature = "map")]
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
    K: Key,
{
    storage: K::Storage<()>,
}

/// A set implementation that uses fixed storage.
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
    K: Key,
{
    /// Creates an empty [`Set`].
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
    #[must_use]
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
    /// let mut set = Set::new();
    /// set.insert(Key::One);
    /// set.insert(Key::Two);
    ///
    /// assert_eq!(set.iter().collect::<Vec<_>>(), vec![Key::One, Key::Two]);
    /// ```
    #[inline]
    pub fn iter(&self) -> Iter<'_, K> {
        self.storage.keys()
    }

    /// Returns `true` if the set currently contains the given value.
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
    /// assert_eq!(set.contains(Key::One), true);
    /// assert_eq!(set.contains(Key::Two), false);
    /// ```
    #[inline]
    pub fn contains(&self, value: K) -> bool {
        self.storage.contains_key(value)
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

    /// Retains only the elements specified by the predicate.
    ///
    /// In other words, remove all elements e for which f(e) returns false.
    /// The elements are visited in unsorted (and unspecified) order.
    ///
    /// # Examples
    ///
    /// ```
    /// use fixed_map::{Key, Set};
    ///
    /// #[derive(Clone, Copy, Key)]
    /// enum Key {
    ///     First,
    ///     Second,
    /// }
    ///
    /// let mut set = Set::new();
    ///
    /// set.insert(Key::First);
    /// set.insert(Key::Second);
    ///
    /// set.retain(|k| matches!(k, Key::First));
    ///
    /// assert_eq!(set.len(), 1);
    /// assert_eq!(set.contains(Key::First), true);
    /// assert_eq!(set.contains(Key::Second), false);
    /// ```
    ///
    /// Using a composite key:
    ///
    /// ```
    /// use fixed_map::{Key, Set};
    ///
    /// #[derive(Clone, Copy, Key)]
    /// enum Key {
    ///     First(bool),
    ///     Second(bool),
    /// }
    ///
    /// let mut set = Set::new();
    ///
    /// set.insert(Key::First(true));
    /// set.insert(Key::First(false));
    /// set.insert(Key::Second(true));
    /// set.insert(Key::Second(false));
    ///
    /// let mut other = set.clone();
    /// assert_eq!(set.len(), 4);
    ///
    /// set.retain(|k| matches!(k, Key::First(true) | Key::Second(true)));
    ///
    /// assert_eq!(set.len(), 2);
    /// assert_eq!(set.contains(Key::First(true)), true);
    /// assert_eq!(set.contains(Key::First(false)), false);
    /// assert_eq!(set.contains(Key::Second(true)), true);
    /// assert_eq!(set.contains(Key::Second(false)), false);
    ///
    /// other.retain(|k| matches!(k, Key::First(_)));
    ///
    /// assert_eq!(other.len(), 2);
    /// assert_eq!(other.contains(Key::First(true)), true);
    /// assert_eq!(other.contains(Key::First(false)), true);
    /// assert_eq!(other.contains(Key::Second(true)), false);
    /// assert_eq!(other.contains(Key::Second(false)), false);
    /// ```
    #[inline]
    pub fn retain<F>(&mut self, mut f: F)
    where
        F: FnMut(K) -> bool,
    {
        self.storage.retain(|k, _| f(k));
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
        self.storage.clear();
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
        self.storage.is_empty()
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
    ///     First,
    ///     Second,
    /// }
    ///
    /// let mut set = Set::new();
    /// assert_eq!(set.len(), 0);
    /// set.insert(Key::First);
    /// assert_eq!(set.len(), 1);
    /// ```
    #[inline]
    pub fn len(&self) -> usize {
        self.storage.len()
    }
}

/// [`Clone`] implementation for a [`Set`].
///
/// # Examples
///
/// ```
/// use fixed_map::{Key, Set};
///
/// #[derive(Debug, Clone, Copy, Key)]
/// enum Key {
///     First(bool),
///     Second,
/// }
///
/// let mut a = Set::new();
/// a.insert(Key::First(true));
/// let mut b = a.clone();
/// b.insert(Key::Second);
///
/// assert_ne!(a, b);
///
/// assert!(a.contains(Key::First(true)));
/// assert!(!a.contains(Key::Second));
///
/// assert!(b.contains(Key::First(true)));
/// assert!(b.contains(Key::Second));
/// ```
impl<K> Clone for Set<K>
where
    K: Key,
    K::Storage<()>: Clone,
{
    #[inline]
    fn clone(&self) -> Set<K> {
        Set {
            storage: self.storage.clone(),
        }
    }
}

/// The [`Copy`] implementation for a [`Set`] depends on its [`Key`]. If the
/// derived key only consists of unit variants the corresponding [`Set`] will be
/// [`Copy`] as well.
///
/// # Examples
///
/// ```
/// use fixed_map::{Key, Set};
///
/// #[derive(Debug, Clone, Copy, Key)]
/// enum Key {
///     First,
///     Second,
/// }
///
/// let mut a = Set::new();
/// a.insert(Key::First);
/// let mut b = a;
/// b.insert(Key::Second);
///
/// assert_ne!(a, b);
///
/// assert!(a.contains(Key::First));
/// assert!(!a.contains(Key::Second));
///
/// assert!(b.contains(Key::First));
/// assert!(b.contains(Key::Second));
/// ```
impl<K> Copy for Set<K>
where
    K: Key,
    K::Storage<()>: Copy,
{
}

/// The [`Default`] implementation for a [`Set`] produces an empty set.
///
/// # Examples
///
/// ```
/// use fixed_map::{Key, Set};
///
/// #[derive(Debug, Clone, Copy, Key)]
/// enum Key {
///     First,
///     Second,
/// }
///
/// let a = Set::<Key>::default();
/// let b = Set::<Key>::new();
///
/// assert_eq!(a, b);
/// ```
impl<K> Default for Set<K>
where
    K: Key,
{
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

/// The [`Debug`][fmt::Debug] implementation for a [`Set`].
///
/// # Examples
///
/// ```
/// use fixed_map::{Key, Set};
///
/// #[derive(Debug, Clone, Copy, Key)]
/// enum Key {
///     First,
///     Second,
/// }
///
/// let mut a = Set::new();
/// a.insert(Key::First);
///
/// assert_eq!("{First}", format!("{:?}", a));
/// ```
impl<K> fmt::Debug for Set<K>
where
    K: Key + fmt::Debug,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_set().entries(self.iter()).finish()
    }
}

/// [`PartialEq`] implementation for a [`Set`].
///
/// # Examples
///
/// ```
/// use fixed_map::{Key, Set};
///
/// #[derive(Debug, Clone, Copy, Key)]
/// enum Key {
///     First,
///     Second,
/// }
///
/// let mut a = Set::new();
/// a.insert(Key::First);
/// // Note: `a` is Copy since it's using a simple key.
/// let mut b = a;
///
/// assert_eq!(a, b);
///
/// b.insert(Key::Second);
/// assert_ne!(a, b);
/// ```
///
/// Using a composite key:
///
/// ```
/// use fixed_map::{Key, Set};
///
/// #[derive(Debug, Clone, Copy, Key)]
/// enum Key {
///     First(bool),
///     Second,
/// }
///
/// let mut a = Set::new();
/// a.insert(Key::First(true));
/// let mut b = a.clone();
///
/// assert_eq!(a, b);
///
/// b.insert(Key::Second);
/// assert_ne!(a, b);
/// ```
impl<K> PartialEq for Set<K>
where
    K: Key,
    K::Storage<()>: PartialEq,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.storage == other.storage
    }
}

impl<K> Eq for Set<K>
where
    K: Key,
    K::Storage<()>: Eq,
{
}

/// [`Hash`] implementation for a [`Set`].
///
/// # Examples
///
/// ```
/// use std::collections::HashSet;
///
/// use fixed_map::{Key, Set};
///
/// #[derive(Debug, Clone, Copy, Hash, Key)]
/// enum Key {
///     First,
///     Second,
/// }
///
/// let mut a = Set::new();
/// a.insert(Key::First);
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
/// use fixed_map::{Key, Set};
///
/// #[derive(Debug, Clone, Copy, Hash, Key)]
/// enum Key {
///     First(bool),
///     Second,
/// }
///
/// let mut a = Set::new();
/// a.insert(Key::First(true));
///
/// // TODO: support this
/// // let mut set = HashSet::new();
/// // set.insert(a);
/// ```
impl<K> Hash for Set<K>
where
    K: Key,
    K::Storage<()>: Hash,
{
    #[inline]
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.storage.hash(state);
    }
}

/// [`PartialOrd`] implementation for a [`Set`].
///
/// # Examples
///
/// ```
/// use fixed_map::{Key, Set};
///
/// #[derive(Debug, Clone, Copy, Hash, Key)]
/// enum Key {
///     First,
///     Second,
/// }
///
/// let mut a = Set::new();
/// a.insert(Key::First);
///
/// let mut b = Set::new();
/// b.insert(Key::Second);
///
/// assert!(a < b);
/// assert!(a <= b);
/// assert!(!(a > b));
/// assert!(!(a >= b));
/// ```
///
/// Using a composite key:
///
/// ```
/// use fixed_map::{Key, Set};
///
/// #[derive(Debug, Clone, Copy, Hash, Key)]
/// enum Key {
///     First(bool),
///     Second,
/// }
///
/// let mut a = Set::new();
/// a.insert(Key::First(true));
///
/// let mut b = Set::new();
/// b.insert(Key::Second);
///
/// // TODO: support this
/// // assert!(a > b);
/// ```
impl<K> PartialOrd for Set<K>
where
    K: Key,
    K::Storage<()>: PartialOrd,
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

/// [`Ord`] implementation for a [`Set`].
///
/// # Examples
///
/// ```
/// use fixed_map::{Key, Set};
///
/// #[derive(Debug, Clone, Copy, Hash, Key)]
/// enum Key {
///     First,
///     Second,
/// }
///
/// let mut a = Set::new();
/// a.insert(Key::First);
///
/// let mut b = Set::new();
/// b.insert(Key::Second);
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
/// use fixed_map::{Key, Set};
///
/// #[derive(Debug, Clone, Copy, Hash, Key)]
/// enum Key {
///     First(bool),
///     Second,
/// }
///
/// let mut a = Set::new();
/// a.insert(Key::First(true));
///
/// let mut b = Set::new();
/// b.insert(Key::Second);
///
/// // TODO: support this
/// // let mut list = vec![a, b];
/// // list.sort();
/// ```
impl<K> Ord for Set<K>
where
    K: Key,
    K::Storage<()>: Ord,
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

impl<'a, K> IntoIterator for &'a Set<K>
where
    K: Key,
{
    type Item = K;
    type IntoIter = Iter<'a, K>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// Produce an owning iterator which iterates over all elements in the set in
/// order.
///
/// # Examples
///
/// ```
/// use fixed_map::{Key, Set};
///
/// #[derive(Debug, Clone, Copy, PartialEq, Eq, Key)]
/// enum Key {
///     First,
///     Second,
///     Third,
/// }
///
/// let mut set = Set::new();
/// set.insert(Key::First);
/// set.insert(Key::Second);
///
/// assert_eq!(set.into_iter().collect::<Vec<_>>(), vec![Key::First, Key::Second]);
/// ```
impl<K> IntoIterator for Set<K>
where
    K: Key,
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
    /// let mut set = Set::new();
    /// set.insert(Key::One);
    /// set.insert(Key::Two);
    ///
    /// assert_eq!(set.into_iter().collect::<Vec<_>>(), vec![Key::One, Key::Two]);
    /// ```
    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.storage.into_iter().map(|(key, ())| key)
    }
}

impl<K> FromIterator<K> for Set<K>
where
    K: Key,
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
    K: Key + serde::Serialize,
{
    #[inline]
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
    K: Key + serde::de::Deserialize<'de>,
{
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct SeqVisitor<K>(core::marker::PhantomData<K>);

        impl<'de, K> serde::de::Visitor<'de> for SeqVisitor<K>
        where
            K: Key + serde::de::Deserialize<'de>,
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

        deserializer.deserialize_seq(SeqVisitor(core::marker::PhantomData))
    }
}
