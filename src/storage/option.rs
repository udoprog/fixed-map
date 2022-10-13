use core::mem;

use crate::{key::Key, storage::Storage};

/// Storage for [`Option`] types.
///
/// # Examples
///
/// ```
/// use fixed_map::{Key, Map};
///
/// #[derive(Debug, Clone, Copy, PartialEq, Key)]
/// enum Part {
///     A,
///     B,
/// }
///
/// #[derive(Debug, Clone, Copy, PartialEq, Key)]
/// enum Key {
///     First(Option<Part>),
///     Second,
/// }
///
/// let mut a = Map::new();
/// a.insert(Key::First(None), 1);
/// a.insert(Key::First(Some(Part::A)), 2);
///
/// assert_eq!(a.get(Key::First(Some(Part::A))), Some(&2));
/// assert_eq!(a.get(Key::First(Some(Part::B))), None);
/// assert_eq!(a.get(Key::First(None)), Some(&1));
/// assert_eq!(a.get(Key::Second), None);
///
/// assert!(a.iter().eq([(Key::First(Some(Part::A)), &2), (Key::First(None), &1)]));
/// assert!(a.values().copied().eq([2, 1]));
/// assert!(a.keys().eq([Key::First(Some(Part::A)), Key::First(None)]));
/// ```
pub struct OptionStorage<K, V>
where
    K: Key,
{
    some: K::Storage<V>,
    none: Option<V>,
}

impl<K, V> Clone for OptionStorage<K, V>
where
    K: Key,
    K::Storage<V>: Clone,
    V: Clone,
{
    fn clone(&self) -> Self {
        Self {
            some: self.some.clone(),
            none: self.none.clone(),
        }
    }
}

impl<K, V> Copy for OptionStorage<K, V>
where
    K: Key,
    K::Storage<V>: Copy,
    V: Copy,
{
}

impl<K, V> Default for OptionStorage<K, V>
where
    K: Key,
{
    #[inline]
    fn default() -> Self {
        Self {
            some: K::Storage::default(),
            none: Option::default(),
        }
    }
}

impl<K, V> PartialEq for OptionStorage<K, V>
where
    K: Key,
    K::Storage<V>: PartialEq,
    V: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.none == other.none && self.some == other.some
    }
}

impl<K, V> Eq for OptionStorage<K, V>
where
    K: Key,
    K::Storage<V>: Eq,
    V: Eq,
{
}

pub struct Iter<'a, K, V>
where
    K: 'a + Key,
{
    some: <K::Storage<V> as Storage<K, V>>::Iter<'a>,
    none: Option<&'a V>,
}

impl<'a, K, V> Clone for Iter<'a, K, V>
where
    K: Key,
    <K::Storage<V> as Storage<K, V>>::Iter<'a>: Clone,
{
    #[inline]
    fn clone(&self) -> Iter<'a, K, V> {
        Iter {
            some: self.some.clone(),
            none: self.none,
        }
    }
}

impl<'a, K, V> Iterator for Iter<'a, K, V>
where
    K: Key,
{
    type Item = (Option<K>, &'a V);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if let Some((k, v)) = self.some.next() {
            return Some((Some(k), v));
        }

        if let Some(v) = self.none.take() {
            return Some((None, v));
        }

        None
    }
}

/// See [`OptionStorage::keys`].
pub struct Keys<'a, K, V>
where
    K: Key,
    K::Storage<V>: 'a,
{
    some: <K::Storage<V> as Storage<K, V>>::Keys<'a>,
    none: bool,
}

impl<'a, K, V> Clone for Keys<'a, K, V>
where
    K: Key,
    <K::Storage<V> as Storage<K, V>>::Keys<'a>: Clone,
{
    #[inline]
    fn clone(&self) -> Keys<'a, K, V> {
        Keys {
            some: self.some.clone(),
            none: self.none,
        }
    }
}

impl<K, V> Iterator for Keys<'_, K, V>
where
    K: Key,
{
    type Item = Option<K>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(key) = self.some.next() {
            return Some(Some(key));
        }

        if ::core::mem::take(&mut self.none) {
            return Some(None);
        }

        None
    }
}

/// See [`OptionStorage::values`].
pub struct Values<'a, K, V>
where
    K: 'a + Key,
{
    some: <K::Storage<V> as Storage<K, V>>::Values<'a>,
    none: Option<&'a V>,
}

impl<'a, K, V> Clone for Values<'a, K, V>
where
    K: Key,
    <K::Storage<V> as Storage<K, V>>::Values<'a>: Clone,
{
    #[inline]
    fn clone(&self) -> Values<'a, K, V> {
        Values {
            some: self.some.clone(),
            none: self.none,
        }
    }
}

impl<'a, K, V> Iterator for Values<'a, K, V>
where
    K: Key,
{
    type Item = &'a V;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.some.next().or_else(|| self.none.take())
    }
}

pub struct IterMut<'a, K, V>
where
    K: 'a + Key,
{
    some: <K::Storage<V> as Storage<K, V>>::IterMut<'a>,
    none: Option<&'a mut V>,
}

impl<'a, K, V> Iterator for IterMut<'a, K, V>
where
    K: Key,
{
    type Item = (Option<K>, &'a mut V);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if let Some((k, v)) = self.some.next() {
            return Some((Some(k), v));
        }

        if let Some(v) = self.none.take() {
            return Some((None, v));
        }

        None
    }
}

/// See [`OptionStorage::values`].
pub struct ValuesMut<'a, K, V>
where
    K: 'a + Key,
{
    some: <K::Storage<V> as Storage<K, V>>::ValuesMut<'a>,
    none: Option<&'a mut V>,
}

impl<'a, K, V> Iterator for ValuesMut<'a, K, V>
where
    K: Key,
{
    type Item = &'a mut V;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.some.next().or_else(|| self.none.take())
    }
}

pub struct IntoIter<K, V>
where
    K: Key,
{
    some: <K::Storage<V> as Storage<K, V>>::IntoIter,
    none: Option<V>,
}

impl<K, V> Iterator for IntoIter<K, V>
where
    K: Key,
{
    type Item = (Option<K>, V);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if let Some((k, v)) = self.some.next() {
            return Some((Some(k), v));
        }

        if let Some(v) = self.none.take() {
            return Some((None, v));
        }

        None
    }
}

impl<K, V> Storage<Option<K>, V> for OptionStorage<K, V>
where
    K: Key,
{
    type Iter<'this> = Iter<'this, K, V> where Self: 'this;
    type Keys<'this> = Keys<'this, K, V> where Self: 'this;
    type Values<'this> = Values<'this, K, V> where Self: 'this;
    type IterMut<'this> = IterMut<'this, K, V> where Self: 'this;
    type ValuesMut<'this> = ValuesMut<'this, K, V> where Self: 'this;
    type IntoIter = IntoIter<K, V>;

    #[inline]
    fn len(&self) -> usize {
        self.some.len() + usize::from(self.none.is_some())
    }

    #[inline]
    fn is_empty(&self) -> bool {
        self.some.is_empty() && self.none.is_none()
    }

    #[inline]
    fn insert(&mut self, key: Option<K>, value: V) -> Option<V> {
        match key {
            Some(key) => self.some.insert(key, value),
            None => mem::replace(&mut self.none, Some(value)),
        }
    }

    #[inline]
    fn contains_key(&self, key: Option<K>) -> bool {
        match key {
            Some(key) => self.some.contains_key(key),
            None => self.none.is_some(),
        }
    }

    #[inline]
    fn get(&self, key: Option<K>) -> Option<&V> {
        match key {
            Some(key) => self.some.get(key),
            None => self.none.as_ref(),
        }
    }

    #[inline]
    fn get_mut(&mut self, key: Option<K>) -> Option<&mut V> {
        match key {
            Some(key) => self.some.get_mut(key),
            None => self.none.as_mut(),
        }
    }

    #[inline]
    fn remove(&mut self, key: Option<K>) -> Option<V> {
        match key {
            Some(key) => self.some.remove(key),
            None => mem::replace(&mut self.none, None),
        }
    }

    #[inline]
    fn clear(&mut self) {
        self.some.clear();
        self.none = None;
    }

    #[inline]
    fn iter(&self) -> Self::Iter<'_> {
        Iter {
            some: self.some.iter(),
            none: self.none.as_ref(),
        }
    }

    #[inline]
    fn keys(&self) -> Self::Keys<'_> {
        Keys {
            some: self.some.keys(),
            none: true,
        }
    }

    #[inline]
    fn values(&self) -> Self::Values<'_> {
        Values {
            some: self.some.values(),
            none: self.none.as_ref(),
        }
    }

    #[inline]
    fn iter_mut(&mut self) -> Self::IterMut<'_> {
        IterMut {
            some: self.some.iter_mut(),
            none: self.none.as_mut(),
        }
    }

    #[inline]
    fn values_mut(&mut self) -> Self::ValuesMut<'_> {
        ValuesMut {
            some: self.some.values_mut(),
            none: self.none.as_mut(),
        }
    }

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            some: self.some.into_iter(),
            none: self.none,
        }
    }
}
