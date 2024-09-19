// Iterators are confusing if they impl `Copy`.
#![allow(missing_copy_implementations)]

use ::core::mem;

use crate::set::SetStorage;

const TRUE_BIT: u8 = 0b10;
const FALSE_BIT: u8 = 0b01;

/// [`SetStorage`] for [`bool`] types.
///
/// # Examples
///
/// ```
/// use fixed_map::{Key, Set};
///
/// #[derive(Debug, Clone, Copy, PartialEq, Key)]
/// enum MyKey {
///     First(bool),
///     Second,
/// }
///
/// let mut a = Set::new();
/// a.insert(MyKey::First(false));
///
/// assert!(!a.contains(MyKey::First(true)));
/// assert!(a.contains(MyKey::First(false)));
/// assert!(!a.contains(MyKey::Second));
///
/// assert!(a.iter().eq([MyKey::First(false)]));
/// ```
///
/// Iterator over boolean set:
///
/// ```
/// use fixed_map::{Key, Set};
///
/// #[derive(Debug, Clone, Copy, PartialEq, Key)]
/// enum MyKey {
///     Bool(bool),
///     Other,
/// }
///
/// let mut a = Set::new();
/// a.insert(MyKey::Bool(true));
/// a.insert(MyKey::Bool(false));
///
/// assert!(a.iter().eq([MyKey::Bool(true), MyKey::Bool(false)]));
/// assert_eq!(a.iter().rev().collect::<Vec<_>>(), vec![MyKey::Bool(false), MyKey::Bool(true)]);
/// ```
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct BooleanSetStorage {
    bits: u8,
}

/// See [`BooleanSetStorage::iter`].
pub struct Iter {
    bits: u8,
}

impl Clone for Iter {
    #[inline]
    fn clone(&self) -> Iter {
        Iter { bits: self.bits }
    }
}

impl Iterator for Iter {
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

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.bits.count_ones() as usize;
        (len, Some(len))
    }
}

impl DoubleEndedIterator for Iter {
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

impl ExactSizeIterator for Iter {
    #[inline]
    fn len(&self) -> usize {
        self.bits.count_ones() as usize
    }
}

impl SetStorage<bool> for BooleanSetStorage {
    type Iter<'this> = Iter;
    type IntoIter = Iter;

    #[inline]
    fn empty() -> Self {
        Self { bits: 0 }
    }

    #[inline]
    fn len(&self) -> usize {
        usize::from(self.bits & TRUE_BIT != 0)
            .saturating_add(usize::from(self.bits & FALSE_BIT != 0))
    }

    #[inline]
    fn is_empty(&self) -> bool {
        self.bits == 0
    }

    #[inline]
    fn insert(&mut self, value: bool) -> bool {
        let update = self.bits | to_bits(value);
        test(mem::replace(&mut self.bits, update), value)
    }

    #[inline]
    fn contains(&self, value: bool) -> bool {
        test(self.bits, value)
    }

    #[inline]
    fn remove(&mut self, value: bool) -> bool {
        let value = to_bits(value);
        let update = self.bits & !value;
        mem::replace(&mut self.bits, update) != 0
    }

    #[inline]
    fn retain<F>(&mut self, mut f: F)
    where
        F: FnMut(bool) -> bool,
    {
        if test(self.bits, true) && !f(true) {
            self.bits &= !TRUE_BIT;
        }

        if test(self.bits, false) && !f(false) {
            self.bits &= !FALSE_BIT;
        }
    }

    #[inline]
    fn clear(&mut self) {
        self.bits = 0;
    }

    #[inline]
    fn iter(&self) -> Self::Iter<'_> {
        Iter { bits: self.bits }
    }

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        Iter { bits: self.bits }
    }
}

#[inline]
const fn test(bits: u8, value: bool) -> bool {
    bits & to_bits(value) != 0
}

#[inline]
const fn to_bits(value: bool) -> u8 {
    if value {
        TRUE_BIT
    } else {
        FALSE_BIT
    }
}
