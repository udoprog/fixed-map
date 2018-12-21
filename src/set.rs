//! Contains the fixed `Set` implementation.
use std::vec;

use crate::{Key, Storage};

/// A map with a fixed, pre-determined size.
pub struct Set<K: 'static>
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
/// use fixed_map::Set;
///
/// #[derive(Clone, Copy, fixed_map::Key)]
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
/// use fixed_map::Set;
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
/// let mut m = Set::new();
/// m.insert(Key::Simple);
/// m.insert(Key::Composite(Part::A));
///
/// assert_eq!(m.contains(Key::Simple), true);
/// assert_eq!(m.contains(Key::Composite(Part::A)), true);
/// assert_eq!(m.contains(Key::Composite(Part::B)), false);
/// ```
impl<K: 'static> Set<K>
where
    K: Key<K, ()>,
{
    pub fn new() -> Set<K> {
        Set {
            storage: K::Storage::default(),
        }
    }

    /// An iterator visiting all values in arbitrary order.
    /// The iterator element type is `K`.
    ///
    /// Because of limitations in how Rust can express lifetimes through traits, this method will
    /// first pre-allocate a vector to store all references.
    ///
    /// For a zero-cost version of this function, see [`Set::iter_fn`].
    ///
    /// [`Set::iter_fn`]: struct.Set.html#method.iter_fn
    ///
    /// # Examples
    ///
    /// ```
    /// use fixed_map::Set;
    ///
    /// #[derive(Debug, Clone, Copy, PartialEq, Eq, fixed_map::Key)]
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
    pub fn iter(&self) -> Iter<K> {
        let mut out = vec![];
        self.storage.iter(|(k, _)| out.push(k));

        Iter {
            inner: out.into_iter(),
        }
    }

    /// An closure visiting all values in arbitrary order.
    /// The closure argument type is `K`.
    ///
    /// This is a zero-cost version of [`Set::iter`].
    ///
    /// [`Set::iter`]: struct.Set.html#method.iter
    ///
    /// # Examples
    ///
    /// ```
    /// use fixed_map::Set;
    ///
    /// #[derive(Debug, Clone, Copy, PartialEq, Eq, fixed_map::Key)]
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
    /// let mut out = Vec::new();
    /// map.iter_fn(|e| out.push(e));
    /// assert_eq!(out, vec![Key::One, Key::Two]);
    /// ```
    #[inline]
    pub fn iter_fn<'a, F>(&'a self, mut f: F)
    where
        F: FnMut(K),
    {
        self.storage.iter(|(k, _)| f(k))
    }

    /// Returns `true` if the set contains a value.
    /// Returns a reference to the value corresponding to the key.
    ///
    /// # Examples
    ///
    /// ```
    /// use fixed_map::Set;
    ///
    /// #[derive(Clone, Copy, fixed_map::Key)]
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
    /// use fixed_map::Set;
    ///
    /// #[derive(Clone, Copy, fixed_map::Key)]
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
    /// use fixed_map::Set;
    ///
    /// #[derive(Clone, Copy, fixed_map::Key)]
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
    /// use fixed_map::Set;
    ///
    /// #[derive(Clone, Copy, fixed_map::Key)]
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
    /// use fixed_map::Set;
    ///
    /// #[derive(Clone, Copy, fixed_map::Key)]
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
    pub fn is_empty(&self) -> bool {
        let mut empty = true;

        self.storage.iter(|_| {
            empty = false;
        });

        empty
    }

    /// Returns the number of elements in the set.
    ///
    /// # Examples
    ///
    /// ```
    /// use fixed_map::Set;
    ///
    /// #[derive(Clone, Copy, fixed_map::Key)]
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
    pub fn len(&self) -> usize {
        let mut len = 0;

        self.storage.iter(|_| {
            len += 1;
        });

        len
    }
}

impl<K> Clone for Set<K>
where
    K: Key<K, ()>,
    K::Storage: Clone,
{
    fn clone(&self) -> Set<K> {
        Set {
            storage: self.storage.clone(),
        }
    }
}

/// An iterator over the items of a `Set`.
///
/// This `struct` is created by the [`iter`] method on [`Set`].
/// See its documentation for more.
///
/// [`iter`]: struct.Set.html#method.iter
/// [`Set`]: struct.Set.html
#[derive(Clone)]
pub struct Iter<K> {
    inner: vec::IntoIter<K>,
}

impl<K> Iterator for Iter<K> {
    type Item = K;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}
