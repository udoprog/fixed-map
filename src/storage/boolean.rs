#![allow(missing_copy_implementations)] // Iterators are confusing if they impl `Copy`.

use core::mem;

use crate::storage::Storage;

const TRUE_BIT: u8 = 0b10;
const FALSE_BIT: u8 = 0b01;

/// Storage for [`bool`] types.
///
/// # Examples
///
/// ```
/// use fixed_map::{Key, Map};
///
/// #[derive(Debug, Clone, Copy, PartialEq, Key)]
/// enum Key {
///     First(bool),
///     Second,
/// }
///
/// let mut a = Map::new();
/// a.insert(Key::First(false), 1);
///
/// assert_eq!(a.get(Key::First(true)), None);
/// assert_eq!(a.get(Key::First(false)), Some(&1));
/// assert_eq!(a.get(Key::Second), None);
///
/// assert!(a.iter().eq([(Key::First(false), &1)]));
/// assert!(a.values().copied().eq([1]));
/// assert!(a.keys().eq([Key::First(false)]));
/// ```
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct BooleanStorage<V> {
    t: Option<V>,
    f: Option<V>,
}

impl<V> Default for BooleanStorage<V> {
    #[inline]
    fn default() -> Self {
        Self {
            t: Option::default(),
            f: Option::default(),
        }
    }
}

/// Iterator over boolean storage.
///
/// # Examples
///
/// ```
/// use fixed_map::{Key, Map};
///
/// #[derive(Debug, Clone, Copy, PartialEq, Key)]
/// enum Key {
///     Bool(bool),
///     Other,
/// }
///
/// let mut a = Map::new();
/// a.insert(Key::Bool(true), 1);
/// a.insert(Key::Bool(false), 2);
///
/// assert!(a.iter().eq([(Key::Bool(true), &1), (Key::Bool(false), &2)]));
/// assert_eq!(a.iter().rev().collect::<Vec<_>>(), vec![(Key::Bool(false), &2), (Key::Bool(true), &1)]);
/// ```
pub struct Iter<'a, V> {
    t: Option<&'a V>,
    f: Option<&'a V>,
}

impl<'a, V> Clone for Iter<'a, V> {
    #[inline]
    fn clone(&self) -> Iter<'a, V> {
        Iter {
            t: self.t,
            f: self.f,
        }
    }
}

impl<'a, V> Iterator for Iter<'a, V> {
    type Item = (bool, &'a V);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(value) = self.t.take() {
            return Some((true, value));
        }

        if let Some(value) = self.f.take() {
            return Some((false, value));
        }

        None
    }
}

impl<V> DoubleEndedIterator for Iter<'_, V> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        if let Some(value) = self.f.take() {
            return Some((false, value));
        }

        if let Some(value) = self.t.take() {
            return Some((true, value));
        }

        None
    }
}

impl<V> ExactSizeIterator for Iter<'_, V> {
    #[inline]
    fn len(&self) -> usize {
        usize::from(self.t.is_some()) + usize::from(self.f.is_some())
    }
}

/// See [`BooleanStorage::keys`].
pub struct Keys {
    bits: u8,
}

impl Clone for Keys {
    #[inline]
    fn clone(&self) -> Keys {
        Keys { bits: self.bits }
    }
}

impl Iterator for Keys {
    type Item = bool;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.bits & TRUE_BIT != 0 {
            self.bits &= !TRUE_BIT;
            return Some(true);
        }

        if self.bits & FALSE_BIT != 0 {
            self.bits &= !FALSE_BIT;
            return Some(false);
        }

        None
    }
}

impl DoubleEndedIterator for Keys {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.bits & FALSE_BIT != 0 {
            self.bits &= !FALSE_BIT;
            return Some(false);
        }

        if self.bits & TRUE_BIT != 0 {
            self.bits &= !TRUE_BIT;
            return Some(true);
        }

        None
    }
}

/// See [`BooleanStorage::values`].
pub struct Values<'a, V> {
    t: Option<&'a V>,
    f: Option<&'a V>,
}

impl<'a, V> Clone for Values<'a, V> {
    #[inline]
    fn clone(&self) -> Values<'a, V> {
        Values {
            t: self.t,
            f: self.f,
        }
    }
}

impl<'a, V> Iterator for Values<'a, V> {
    type Item = &'a V;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(value) = self.t.take() {
            return Some(value);
        }

        if let Some(value) = self.f.take() {
            return Some(value);
        }

        None
    }
}

impl<V> DoubleEndedIterator for Values<'_, V> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        if let Some(value) = self.f.take() {
            return Some(value);
        }

        if let Some(value) = self.t.take() {
            return Some(value);
        }

        None
    }
}

pub struct IterMut<'a, V> {
    t: Option<&'a mut V>,
    f: Option<&'a mut V>,
}

impl<'a, V> Iterator for IterMut<'a, V> {
    type Item = (bool, &'a mut V);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(t) = self.t.take() {
            return Some((true, t));
        }

        if let Some(f) = self.f.take() {
            return Some((false, f));
        }

        None
    }
}

/// See [`BooleanStorage::values`].
pub struct ValuesMut<'a, V> {
    t: Option<&'a mut V>,
    f: Option<&'a mut V>,
}

impl<'a, V> Iterator for ValuesMut<'a, V> {
    type Item = &'a mut V;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(value) = self.t.take() {
            return Some(value);
        }

        if let Some(value) = self.f.take() {
            return Some(value);
        }

        None
    }
}

impl<V> DoubleEndedIterator for ValuesMut<'_, V> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        if let Some(value) = self.f.take() {
            return Some(value);
        }

        if let Some(value) = self.t.take() {
            return Some(value);
        }

        None
    }
}

pub struct IntoIter<V> {
    t: Option<V>,
    f: Option<V>,
}

impl<V> Iterator for IntoIter<V> {
    type Item = (bool, V);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(t) = self.t.take() {
            return Some((true, t));
        }

        if let Some(f) = self.f.take() {
            return Some((false, f));
        }

        None
    }
}

impl<V> DoubleEndedIterator for IntoIter<V> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        if let Some(f) = self.f.take() {
            return Some((false, f));
        }

        if let Some(t) = self.t.take() {
            return Some((true, t));
        }

        None
    }
}

impl<V> Storage<bool, V> for BooleanStorage<V> {
    type Iter<'this> = Iter<'this, V> where Self: 'this;
    type Keys<'this> = Keys where Self: 'this;
    type Values<'this> = Values<'this, V> where Self: 'this;
    type IterMut<'this> = IterMut<'this, V> where Self: 'this;
    type ValuesMut<'this> = ValuesMut<'this, V> where Self: 'this;
    type IntoIter = IntoIter<V>;

    #[inline]
    fn len(&self) -> usize {
        usize::from(self.t.is_some()).saturating_add(usize::from(self.f.is_some()))
    }

    #[inline]
    fn is_empty(&self) -> bool {
        self.t.is_none() && self.f.is_none()
    }

    #[inline]
    fn insert(&mut self, key: bool, value: V) -> Option<V> {
        if key {
            mem::replace(&mut self.t, Some(value))
        } else {
            mem::replace(&mut self.f, Some(value))
        }
    }

    #[inline]
    fn contains_key(&self, key: bool) -> bool {
        if key {
            self.t.is_some()
        } else {
            self.f.is_some()
        }
    }

    #[inline]
    fn get(&self, key: bool) -> Option<&V> {
        if key {
            self.t.as_ref()
        } else {
            self.f.as_ref()
        }
    }

    #[inline]
    fn get_mut(&mut self, key: bool) -> Option<&mut V> {
        if key {
            self.t.as_mut()
        } else {
            self.f.as_mut()
        }
    }

    #[inline]
    fn remove(&mut self, key: bool) -> Option<V> {
        if key {
            mem::replace(&mut self.t, None)
        } else {
            mem::replace(&mut self.f, None)
        }
    }

    #[inline]
    fn clear(&mut self) {
        self.t = None;
        self.f = None;
    }

    #[inline]
    fn iter(&self) -> Self::Iter<'_> {
        Iter {
            t: self.t.as_ref(),
            f: self.f.as_ref(),
        }
    }

    #[inline]
    fn keys(&self) -> Self::Keys<'_> {
        Keys {
            bits: if self.t.is_some() { TRUE_BIT } else { 0 }
                | if self.f.is_some() { FALSE_BIT } else { 0 },
        }
    }

    #[inline]
    fn values(&self) -> Self::Values<'_> {
        Values {
            t: self.t.as_ref(),
            f: self.f.as_ref(),
        }
    }

    #[inline]
    fn iter_mut(&mut self) -> Self::IterMut<'_> {
        IterMut {
            t: self.t.as_mut(),
            f: self.f.as_mut(),
        }
    }

    #[inline]
    fn values_mut(&mut self) -> Self::ValuesMut<'_> {
        ValuesMut {
            t: self.t.as_mut(),
            f: self.f.as_mut(),
        }
    }

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            t: self.t,
            f: self.f,
        }
    }
}
