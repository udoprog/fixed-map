//! Contains the fixed [`Set`] implementation.

use ::core::cmp::Ordering;
use ::core::fmt;
use ::core::hash::{Hash, Hasher};

pub mod intersection;
pub mod storage;

pub use self::intersection::Intersection;
pub use self::storage::SetStorage;

use crate::raw::RawStorage;
use crate::Key;

/// The iterator produced by [`Set::iter`].
pub type Iter<'a, T> = <<T as Key>::SetStorage as SetStorage<T>>::Iter<'a>;

/// The iterator produced by [`Set::into_iter`].
pub type IntoIter<T> = <<T as Key>::SetStorage as SetStorage<T>>::IntoIter;

/// A fixed set with storage specialized through the [`Key`] trait.
///
/// # Examples
///
/// ```
/// use fixed_map::{Key, Set};
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
/// let mut set = Set::new();
///
/// set.insert(MyKey::Simple);
/// set.insert(MyKey::Composite(Part::One));
/// # #[cfg(feature = "hashbrown")]
/// set.insert(MyKey::String("foo"));
/// # #[cfg(feature = "hashbrown")]
/// set.insert(MyKey::Number(1));
/// set.insert(MyKey::Singleton(()));
/// set.insert(MyKey::Option(None));
/// set.insert(MyKey::Option(Some(Part::One)));
/// set.insert(MyKey::Boolean(true));
///
/// assert!(set.contains(MyKey::Simple));
/// assert!(set.contains(MyKey::Composite(Part::One)));
/// assert!(!set.contains(MyKey::Composite(Part::Two)));
/// # #[cfg(feature = "hashbrown")]
/// assert!(set.contains(MyKey::String("foo")));
/// # #[cfg(feature = "hashbrown")]
/// assert!(!set.contains(MyKey::String("bar")));
/// # #[cfg(feature = "hashbrown")]
/// assert!(set.contains(MyKey::Number(1)));
/// # #[cfg(feature = "hashbrown")]
/// assert!(!set.contains(MyKey::Number(2)));
/// assert!(set.contains(MyKey::Singleton(())));
/// assert!(set.contains(MyKey::Option(None)));
/// assert!(set.contains(MyKey::Option(Some(Part::One))));
/// assert!(!set.contains(MyKey::Option(Some(Part::Two))));
/// assert!(set.contains(MyKey::Boolean(true)));
/// assert!(!set.contains(MyKey::Boolean(false)));
/// ```
#[repr(transparent)]
pub struct Set<T>
where
    T: Key,
{
    storage: T::SetStorage,
}

/// A set implementation that uses fixed storage.
///
/// # Examples
///
/// ```
/// use fixed_map::{Key, Set};
///
/// #[derive(Clone, Copy, Key)]
/// enum MyKey {
///     First,
///     Second,
/// }
///
/// let mut m = Set::new();
/// m.insert(MyKey::First);
///
/// assert_eq!(m.contains(MyKey::First), true);
/// assert_eq!(m.contains(MyKey::Second), false);
/// ```
///
/// Using a composite key:
///
/// ```
/// use fixed_map::{Key, Set};
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
/// let mut m = Set::new();
/// m.insert(MyKey::Simple);
/// m.insert(MyKey::Composite(Part::A));
///
/// assert_eq!(m.contains(MyKey::Simple), true);
/// assert_eq!(m.contains(MyKey::Composite(Part::A)), true);
/// assert_eq!(m.contains(MyKey::Composite(Part::B)), false);
/// ```
impl<T> Set<T>
where
    T: Key,
{
    /// Creates an empty [`Set`].
    ///
    /// # Examples
    ///
    /// ```
    /// use fixed_map::{Key, Set};
    ///
    /// #[derive(Clone, Copy, Key)]
    /// enum MyKey {
    ///     One,
    ///     Two,
    /// }
    ///
    /// let set: Set<MyKey> = Set::new();
    /// ```
    #[inline]
    #[must_use]
    pub fn new() -> Set<T> {
        Set {
            storage: T::SetStorage::empty(),
        }
    }

    /// An iterator visiting all values in arbitrary order.
    /// The iterator element type is `T`.
    ///
    /// # Examples
    ///
    /// ```
    /// use fixed_map::{Key, Set};
    ///
    /// #[derive(Debug, Clone, Copy, Key, PartialEq, Eq)]
    /// enum MyKey {
    ///     One,
    ///     Two,
    ///     Three,
    /// }
    ///
    /// let mut set = Set::new();
    /// set.insert(MyKey::One);
    /// set.insert(MyKey::Two);
    ///
    /// assert_eq!(set.iter().collect::<Vec<_>>(), vec![MyKey::One, MyKey::Two]);
    /// ```
    #[inline]
    pub fn iter(&self) -> Iter<'_, T> {
        self.storage.iter()
    }

    /// Returns `true` if the set currently contains the given value.
    ///
    /// # Examples
    ///
    /// ```
    /// use fixed_map::{Key, Set};
    ///
    /// #[derive(Clone, Copy, Key)]
    /// enum MyKey {
    ///     One,
    ///     Two,
    /// }
    ///
    /// let mut set = Set::new();
    /// set.insert(MyKey::One);
    /// assert_eq!(set.contains(MyKey::One), true);
    /// assert_eq!(set.contains(MyKey::Two), false);
    /// ```
    #[inline]
    pub fn contains(&self, value: T) -> bool {
        self.storage.contains(value)
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
    /// enum MyKey {
    ///     One,
    ///     Two,
    /// }
    ///
    /// let mut set = Set::new();
    /// assert!(set.insert(MyKey::One));
    /// assert!(!set.is_empty());
    ///
    /// set.insert(MyKey::Two);
    /// assert!(!set.insert(MyKey::Two));
    /// assert!(set.contains(MyKey::Two));
    /// ```
    #[inline]
    pub fn insert(&mut self, value: T) -> bool {
        self.storage.insert(value)
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
    /// enum MyKey {
    ///     One,
    ///     Two,
    /// }
    ///
    /// let mut set = Set::new();
    /// set.insert(MyKey::One);
    /// assert_eq!(set.remove(MyKey::One), true);
    /// assert_eq!(set.remove(MyKey::One), false);
    /// ```
    #[inline]
    pub fn remove(&mut self, value: T) -> bool {
        self.storage.remove(value)
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
    /// enum MyKey {
    ///     First,
    ///     Second,
    /// }
    ///
    /// let mut set = Set::new();
    ///
    /// set.insert(MyKey::First);
    /// set.insert(MyKey::Second);
    ///
    /// set.retain(|k| matches!(k, MyKey::First));
    ///
    /// assert_eq!(set.len(), 1);
    /// assert_eq!(set.contains(MyKey::First), true);
    /// assert_eq!(set.contains(MyKey::Second), false);
    /// ```
    ///
    /// Using a composite key:
    ///
    /// ```
    /// use fixed_map::{Key, Set};
    ///
    /// #[derive(Clone, Copy, Key)]
    /// enum MyKey {
    ///     First(bool),
    ///     Second(bool),
    /// }
    ///
    /// let mut set = Set::new();
    ///
    /// set.insert(MyKey::First(true));
    /// set.insert(MyKey::First(false));
    /// set.insert(MyKey::Second(true));
    /// set.insert(MyKey::Second(false));
    ///
    /// let mut other = set.clone();
    /// assert_eq!(set.len(), 4);
    ///
    /// set.retain(|k| matches!(k, MyKey::First(true) | MyKey::Second(true)));
    ///
    /// assert_eq!(set.len(), 2);
    /// assert_eq!(set.contains(MyKey::First(true)), true);
    /// assert_eq!(set.contains(MyKey::First(false)), false);
    /// assert_eq!(set.contains(MyKey::Second(true)), true);
    /// assert_eq!(set.contains(MyKey::Second(false)), false);
    ///
    /// other.retain(|k| matches!(k, MyKey::First(_)));
    ///
    /// assert_eq!(other.len(), 2);
    /// assert_eq!(other.contains(MyKey::First(true)), true);
    /// assert_eq!(other.contains(MyKey::First(false)), true);
    /// assert_eq!(other.contains(MyKey::Second(true)), false);
    /// assert_eq!(other.contains(MyKey::Second(false)), false);
    /// ```
    #[inline]
    pub fn retain<F>(&mut self, f: F)
    where
        F: FnMut(T) -> bool,
    {
        self.storage.retain(f);
    }

    /// Clears the set, removing all values.
    ///
    /// # Examples
    ///
    /// ```
    /// use fixed_map::{Key, Set};
    ///
    /// #[derive(Clone, Copy, Key)]
    /// enum MyKey {
    ///     One,
    ///     Two,
    /// }
    ///
    /// let mut set = Set::new();
    /// set.insert(MyKey::One);
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
    /// enum MyKey {
    ///     One,
    ///     Two,
    /// }
    ///
    /// let mut set = Set::new();
    /// assert!(set.is_empty());
    /// set.insert(MyKey::One);
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
    /// enum MyKey {
    ///     First,
    ///     Second,
    /// }
    ///
    /// let mut set = Set::new();
    /// assert_eq!(set.len(), 0);
    /// set.insert(MyKey::First);
    /// assert_eq!(set.len(), 1);
    /// ```
    #[inline]
    pub fn len(&self) -> usize {
        self.storage.len()
    }

    /// Visits the values representing the intersection,
    /// i.e., the values that are both in `self` and `other`.
    ///
    /// When an equal element is present in `self` and `other`
    /// then the resulting `Intersection` may yield references to
    /// one or the other. This can be relevant if `T` contains fields which
    /// are not compared by its `Eq` implementation, and may hold different
    /// value between the two equal copies of `T` in the two sets.
    ///
    /// # Examples
    ///
    /// ```
    /// use fixed_map::{Key, Set};
    ///
    /// #[derive(Clone, Copy, Key, Debug)]
    /// enum MyKey {
    ///     First,
    ///     Second,
    /// }
    ///
    /// let mut a = Set::new();
    /// a.insert(MyKey::First);
    /// let mut b = Set::new();
    /// b.insert(MyKey::First);
    /// b.insert(MyKey::Second);
    ///
    /// let intersection: Set<_> = a.intersection(&b).collect();
    /// assert_eq!(intersection, [MyKey::First].into_iter().collect());
    /// ```
    #[inline]
    pub fn intersection<'a>(&'a self, other: &'a Set<T>) -> Intersection<'a, T> {
        if self.len() <= other.len() {
            Intersection {
                iter: self.iter(),
                other,
            }
        } else {
            Intersection {
                iter: other.iter(),
                other: self,
            }
        }
    }
}

impl<T> Set<T>
where
    T: Key,
    T::SetStorage: RawStorage,
{
    /// Get the raw value of the set.
    ///
    /// # Examples
    ///
    /// ```
    /// use fixed_map::{Key, Set};
    ///
    /// #[derive(Debug, Clone, Copy, Key)]
    /// #[key(bitset)]
    /// enum MyKey {
    ///     First,
    ///     Second,
    /// }
    ///
    /// let mut set = Set::new();
    /// assert!(set.as_raw() == 0);
    /// set.insert(MyKey::First);
    /// assert!(set.as_raw() != 0);
    ///
    /// let set2 = Set::from_raw(set.as_raw());
    /// assert_eq!(set, set2);
    /// ```
    #[inline]
    pub fn as_raw(&self) -> <T::SetStorage as RawStorage>::Value {
        self.storage.as_raw()
    }

    /// Construct the set from a raw value.
    ///
    /// # Examples
    ///
    /// ```
    /// use fixed_map::{Key, Set};
    ///
    /// #[derive(Debug, Clone, Copy, Key)]
    /// #[key(bitset)]
    /// enum MyKey {
    ///     First,
    ///     Second,
    /// }
    ///
    /// let mut set = Set::new();
    /// assert!(set.as_raw() == 0);
    /// set.insert(MyKey::First);
    /// assert!(set.as_raw() != 0);
    ///
    /// let set2 = Set::from_raw(set.as_raw());
    /// assert_eq!(set, set2);
    /// ```
    #[inline]
    pub fn from_raw(raw: <T::SetStorage as RawStorage>::Value) -> Self {
        Self {
            storage: <T::SetStorage as RawStorage>::from_raw(raw),
        }
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
/// enum MyKey {
///     First(bool),
///     Second,
/// }
///
/// let mut a = Set::new();
/// a.insert(MyKey::First(true));
/// let mut b = a.clone();
/// b.insert(MyKey::Second);
///
/// assert_ne!(a, b);
///
/// assert!(a.contains(MyKey::First(true)));
/// assert!(!a.contains(MyKey::Second));
///
/// assert!(b.contains(MyKey::First(true)));
/// assert!(b.contains(MyKey::Second));
/// ```
impl<T> Clone for Set<T>
where
    T: Key,
    T::SetStorage: Clone,
{
    #[inline]
    fn clone(&self) -> Set<T> {
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
/// enum MyKey {
///     First,
///     Second,
/// }
///
/// let mut a = Set::new();
/// a.insert(MyKey::First);
/// let mut b = a;
/// b.insert(MyKey::Second);
///
/// assert_ne!(a, b);
///
/// assert!(a.contains(MyKey::First));
/// assert!(!a.contains(MyKey::Second));
///
/// assert!(b.contains(MyKey::First));
/// assert!(b.contains(MyKey::Second));
/// ```
impl<T> Copy for Set<T>
where
    T: Key,
    T::SetStorage: Copy,
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
/// enum MyKey {
///     First,
///     Second,
/// }
///
/// let a = Set::<MyKey>::default();
/// let b = Set::<MyKey>::new();
///
/// assert_eq!(a, b);
/// ```
impl<T> Default for Set<T>
where
    T: Key,
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
/// enum MyKey {
///     First,
///     Second,
/// }
///
/// let mut a = Set::new();
/// a.insert(MyKey::First);
///
/// assert_eq!("{First}", format!("{:?}", a));
/// ```
impl<T> fmt::Debug for Set<T>
where
    T: Key + fmt::Debug,
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
/// enum MyKey {
///     First,
///     Second,
/// }
///
/// let mut a = Set::new();
/// a.insert(MyKey::First);
/// // Note: `a` is Copy since it's using a simple key.
/// let mut b = a;
///
/// assert_eq!(a, b);
///
/// b.insert(MyKey::Second);
/// assert_ne!(a, b);
/// ```
///
/// Using a composite key:
///
/// ```
/// use fixed_map::{Key, Set};
///
/// #[derive(Debug, Clone, Copy, Key)]
/// enum MyKey {
///     First(bool),
///     Second,
/// }
///
/// let mut a = Set::new();
/// a.insert(MyKey::First(true));
/// let mut b = a.clone();
///
/// assert_eq!(a, b);
///
/// b.insert(MyKey::Second);
/// assert_ne!(a, b);
/// ```
impl<T> PartialEq for Set<T>
where
    T: Key,
    T::SetStorage: PartialEq,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.storage == other.storage
    }
}

impl<T> Eq for Set<T>
where
    T: Key,
    T::SetStorage: Eq,
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
/// #[derive(Debug, Clone, Copy, Key, Hash)]
/// enum MyKey {
///     First,
///     Second,
/// }
///
/// let mut a = Set::new();
/// a.insert(MyKey::First);
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
/// #[derive(Debug, Clone, Copy, Key, Hash)]
/// enum MyKey {
///     First(bool),
///     Second,
/// }
///
/// let mut a = Set::new();
/// a.insert(MyKey::First(true));
///
/// // TODO: support this
/// // let mut set = HashSet::new();
/// // set.insert(a);
/// ```
impl<T> Hash for Set<T>
where
    T: Key,
    T::SetStorage: Hash,
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
/// For more details on ordering, see the [`Key`] documentation.
///
/// # Examples
///
/// ```
/// use fixed_map::{Key, Set};
///
/// #[derive(Debug, Clone, Copy, Key, Hash)]
/// enum MyKey {
///     First,
///     Second,
///     Third,
/// }
///
/// let mut a = Set::new();
/// a.insert(MyKey::First);
///
/// let mut b = Set::new();
/// b.insert(MyKey::Third);
///
/// assert!(a < b);
///
/// let mut empty = Set::new();
/// assert!(empty < a);
/// assert!(empty < b);
/// ```
///
/// Using a composite key:
///
/// ```
/// use fixed_map::{Key, Set};
///
/// #[derive(Debug, Clone, Copy, Key, Hash)]
/// enum MyKey {
///     First(bool),
///     Second,
/// }
///
/// let mut a = Set::new();
/// a.insert(MyKey::First(true));
///
/// let mut b = Set::new();
/// b.insert(MyKey::Second);
///
/// // TODO: support this
/// // assert!(a < b);
/// ```
impl<T> PartialOrd for Set<T>
where
    T: Key,
    T::SetStorage: PartialOrd,
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
/// For more details on ordering, see the [`Key`] documentation.
///
/// # Examples
///
/// ```
/// use fixed_map::{Key, Set};
///
/// #[derive(Debug, Clone, Copy, Key, Hash)]
/// enum MyKey {
///     First,
///     Second,
/// }
///
/// let mut a = Set::new();
/// a.insert(MyKey::First);
///
/// let mut b = Set::new();
/// b.insert(MyKey::Second);
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
/// #[derive(Debug, Clone, Copy, Key, Hash)]
/// enum MyKey {
///     First(bool),
///     Second,
/// }
///
/// let mut a = Set::new();
/// a.insert(MyKey::First(true));
///
/// let mut b = Set::new();
/// b.insert(MyKey::Second);
///
/// // TODO: support this
/// // let mut list = vec![a, b];
/// // list.sort();
/// ```
impl<T> Ord for Set<T>
where
    T: Key,
    T::SetStorage: Ord,
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

impl<'a, T> IntoIterator for &'a Set<T>
where
    T: Key,
{
    type Item = T;
    type IntoIter = Iter<'a, T>;

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
/// #[derive(Debug, Clone, Copy, Key, PartialEq, Eq)]
/// enum MyKey {
///     First,
///     Second,
///     Third,
/// }
///
/// let mut set = Set::new();
/// set.insert(MyKey::First);
/// set.insert(MyKey::Second);
///
/// assert_eq!(set.into_iter().collect::<Vec<_>>(), vec![MyKey::First, MyKey::Second]);
/// ```
impl<T> IntoIterator for Set<T>
where
    T: Key,
{
    type Item = T;
    type IntoIter = IntoIter<T>;

    /// An iterator visiting all values in arbitrary order.
    /// The iterator element type is `T`.
    ///
    /// # Examples
    ///
    /// ```
    /// use fixed_map::{Key, Set};
    ///
    /// #[derive(Debug, Clone, Copy, Key, PartialEq, Eq)]
    /// enum MyKey {
    ///     One,
    ///     Two,
    ///     Three,
    /// }
    ///
    /// let mut set = Set::new();
    /// set.insert(MyKey::One);
    /// set.insert(MyKey::Two);
    ///
    /// assert_eq!(set.into_iter().collect::<Vec<_>>(), vec![MyKey::One, MyKey::Two]);
    /// ```
    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.storage.into_iter()
    }
}

impl<T> FromIterator<T> for Set<T>
where
    T: Key,
{
    #[inline]
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        let mut set = Self::new();

        for value in iter {
            set.insert(value);
        }

        set
    }
}

#[cfg(feature = "serde")]
impl<T> serde::Serialize for Set<T>
where
    T: Key + serde::Serialize,
{
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeSeq as _;

        let mut seq = serializer.serialize_seq(Some(self.len()))?;

        for v in self {
            seq.serialize_element(&v)?;
        }

        seq.end()
    }
}

#[cfg(feature = "serde")]
impl<'de, T> serde::de::Deserialize<'de> for Set<T>
where
    T: Key + serde::de::Deserialize<'de>,
{
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct SeqVisitor<T>(core::marker::PhantomData<T>);

        impl<'de, T> serde::de::Visitor<'de> for SeqVisitor<T>
        where
            T: Key + serde::de::Deserialize<'de>,
        {
            type Value = Set<T>;

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
