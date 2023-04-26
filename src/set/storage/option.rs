use core::iter;
use core::mem;
use core::option;

use crate::set::SetStorage;
use crate::Key;

type Iter<'a, T> = iter::Chain<
    iter::Map<<<T as Key>::SetStorage as SetStorage<T>>::Iter<'a>, fn(T) -> Option<T>>,
    option::IntoIter<Option<T>>,
>;
type IntoIter<T> = iter::Chain<
    iter::Map<<<T as Key>::SetStorage as SetStorage<T>>::IntoIter, fn(T) -> Option<T>>,
    option::IntoIter<Option<T>>,
>;

/// [`SetStorage`] for [`Option`] types.
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
/// enum MyKey {
///     First(Option<Part>),
///     Second,
/// }
///
/// let mut a = Map::new();
/// a.insert(MyKey::First(None), 1);
/// a.insert(MyKey::First(Some(Part::A)), 2);
///
/// assert_eq!(a.get(MyKey::First(Some(Part::A))), Some(&2));
/// assert_eq!(a.get(MyKey::First(Some(Part::B))), None);
/// assert_eq!(a.get(MyKey::First(None)), Some(&1));
/// assert_eq!(a.get(MyKey::Second), None);
///
/// assert!(a.iter().eq([(MyKey::First(Some(Part::A)), &2), (MyKey::First(None), &1)]));
/// assert!(a.values().copied().eq([2, 1]));
/// assert!(a.keys().eq([MyKey::First(Some(Part::A)), MyKey::First(None)]));
/// ```
pub struct OptionSetStorage<T>
where
    T: Key,
{
    some: T::SetStorage,
    none: bool,
}

impl<T> Clone for OptionSetStorage<T>
where
    T: Key,
    T::SetStorage: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        Self {
            some: self.some.clone(),
            none: self.none,
        }
    }
}

impl<T> Copy for OptionSetStorage<T>
where
    T: Key,
    T::SetStorage: Copy,
{
}

impl<T> PartialEq for OptionSetStorage<T>
where
    T: Key,
    T::SetStorage: PartialEq,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.none == other.none && self.some == other.some
    }
}

impl<T> Eq for OptionSetStorage<T>
where
    T: Key,
    T::SetStorage: Eq,
{
}

impl<T> SetStorage<Option<T>> for OptionSetStorage<T>
where
    T: Key,
{
    type Iter<'this> = Iter<'this, T> where T: 'this;
    type IntoIter = IntoIter<T>;

    #[inline]
    fn empty() -> Self {
        Self {
            some: T::SetStorage::empty(),
            none: false,
        }
    }

    #[inline]
    fn len(&self) -> usize {
        self.some.len() + usize::from(self.none)
    }

    #[inline]
    fn is_empty(&self) -> bool {
        self.some.is_empty() && self.none
    }

    #[inline]
    fn insert(&mut self, value: Option<T>) -> bool {
        match value {
            Some(value) => self.some.insert(value),
            None => mem::replace(&mut self.none, true),
        }
    }

    #[inline]
    fn contains(&self, value: Option<T>) -> bool {
        match value {
            Some(key) => self.some.contains(key),
            None => self.none,
        }
    }

    #[inline]
    fn remove(&mut self, key: Option<T>) -> bool {
        match key {
            Some(key) => self.some.remove(key),
            None => mem::replace(&mut self.none, false),
        }
    }

    #[inline]
    fn retain<F>(&mut self, mut func: F)
    where
        F: FnMut(Option<T>) -> bool,
    {
        self.some.retain(|value| func(Some(value)));

        if self.none {
            self.none = func(None);
        }
    }

    #[inline]
    fn clear(&mut self) {
        self.some.clear();
        self.none = false;
    }

    #[inline]
    fn iter(&self) -> Self::Iter<'_> {
        let map: fn(_) -> _ = Some;
        self.some
            .iter()
            .map(map)
            .chain(self.none.then_some(None::<T>))
    }

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let map: fn(_) -> _ = Some;
        self.some
            .into_iter()
            .map(map)
            .chain(self.none.then_some(None::<T>))
    }
}
