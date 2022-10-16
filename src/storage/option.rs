use core::iter;
use core::mem;
use core::option;

use crate::key::Key;
use crate::storage::Storage;

type Iter<'a, K, V> = iter::Chain<
    iter::Map<
        <<K as Key>::Storage<V> as Storage<K, V>>::Iter<'a>,
        fn((K, &'a V)) -> (Option<K>, &'a V),
    >,
    iter::Map<option::Iter<'a, V>, fn(&'a V) -> (Option<K>, &'a V)>,
>;
type Keys<'a, K, V> = iter::Chain<
    iter::Map<<<K as Key>::Storage<V> as Storage<K, V>>::Keys<'a>, fn(K) -> Option<K>>,
    option::IntoIter<Option<K>>,
>;
type Values<'a, K, V> =
    iter::Chain<<<K as Key>::Storage<V> as Storage<K, V>>::Values<'a>, option::Iter<'a, V>>;
type IterMut<'a, K, V> = iter::Chain<
    iter::Map<
        <<K as Key>::Storage<V> as Storage<K, V>>::IterMut<'a>,
        fn((K, &'a mut V)) -> (Option<K>, &'a mut V),
    >,
    iter::Map<option::IterMut<'a, V>, fn(&'a mut V) -> (Option<K>, &'a mut V)>,
>;
type ValuesMut<'a, K, V> =
    iter::Chain<<<K as Key>::Storage<V> as Storage<K, V>>::ValuesMut<'a>, option::IterMut<'a, V>>;
type IntoIter<K, V> = iter::Chain<
    iter::Map<<<K as Key>::Storage<V> as Storage<K, V>>::IntoIter, fn((K, V)) -> (Option<K>, V)>,
    iter::Map<option::IntoIter<V>, fn(V) -> (Option<K>, V)>,
>;

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
    pub(in crate::storage) some: K::Storage<V>,
    pub(in crate::storage) none: Option<V>,
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
    fn retain<F>(&mut self, mut func: F)
    where
        F: FnMut(Option<K>, &mut V) -> bool,
    {
        self.some.retain(|k, v| func(Some(k), v));
        if let Some(none) = self.none.as_mut() {
            if !func(None, none) {
                self.none = None;
            }
        }
    }

    #[inline]
    fn clear(&mut self) {
        self.some.clear();
        self.none = None;
    }

    #[inline]
    fn iter(&self) -> Self::Iter<'_> {
        let map: fn(_) -> _ = |(k, b)| (Some(k), b);
        let a = self.some.iter().map(map);
        let map: fn(_) -> _ = |v| (None, v);
        let b = self.none.iter().map(map);
        a.chain(b)
    }

    #[inline]
    fn keys(&self) -> Self::Keys<'_> {
        let map: fn(_) -> _ = |k| Some(k);
        self.some
            .keys()
            .map(map)
            .chain(self.none.is_some().then_some(None::<K>))
    }

    #[inline]
    fn values(&self) -> Self::Values<'_> {
        self.some.values().chain(self.none.iter())
    }

    #[inline]
    fn iter_mut(&mut self) -> Self::IterMut<'_> {
        let map: fn(_) -> _ = |(k, b)| (Some(k), b);
        let a = self.some.iter_mut().map(map);
        let map: fn(_) -> _ = |v| (None, v);
        let b = self.none.iter_mut().map(map);
        a.chain(b)
    }

    #[inline]
    fn values_mut(&mut self) -> Self::ValuesMut<'_> {
        self.some.values_mut().chain(self.none.iter_mut())
    }

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let map: fn(_) -> _ = |(k, b)| (Some(k), b);
        let a = self.some.into_iter().map(map);
        let map: fn(_) -> _ = |v| (None, v);
        let b = self.none.into_iter().map(map);
        a.chain(b)
    }
}
