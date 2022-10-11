use core::mem;

use crate::{key::Key, storage::Storage};

/// Storage for `Option<T>`s.
pub struct OptionStorage<K, V>
where
    K: Key<K, V>,
{
    some: K::Storage,
    none: Option<V>,
}

impl<K, V> Clone for OptionStorage<K, V>
where
    K: Key<K, V>,
    K::Storage: Clone,
    V: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        OptionStorage {
            some: self.some.clone(),
            none: self.none.clone(),
        }
    }
}

impl<K, V> Copy for OptionStorage<K, V>
where
    K: Key<K, V>,
    K::Storage: Copy,
    V: Copy,
{
}

impl<K, V> Default for OptionStorage<K, V>
where
    K: Key<K, V>,
{
    #[inline]
    fn default() -> Self {
        Self {
            some: Default::default(),
            none: Default::default(),
        }
    }
}

impl<K, V> PartialEq for OptionStorage<K, V>
where
    K: Key<K, V>,
    K::Storage: PartialEq,
    V: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.none == other.none && self.some == other.some
    }
}

impl<K, V> Eq for OptionStorage<K, V>
where
    K: Key<K, V>,
    K::Storage: Eq,
    V: Eq,
{
}

pub struct Iter<'a, K, V>
where
    K: 'a + Key<K, V>,
    V: 'a,
{
    some: <K::Storage as Storage<K, V>>::Iter<'a>,
    none: Option<&'a V>,
}

impl<'a, K, V> Clone for Iter<'a, K, V>
where
    K: Key<K, V>,
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
    K: Key<K, V>,
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

pub struct Values<'a, K, V>
where
    K: 'a + Key<K, V>,
    V: 'a,
{
    some: <K::Storage as Storage<K, V>>::Values<'a>,
    none: Option<&'a V>,
}

impl<'a, K, V> Clone for Values<'a, K, V>
where
    K: Key<K, V>,
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
    K: Key<K, V>,
{
    type Item = &'a V;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.some.next().or(self.none.take())
    }
}

pub struct IterMut<'a, K, V>
where
    K: 'a + Key<K, V>,
    V: 'a,
{
    some: <K::Storage as Storage<K, V>>::IterMut<'a>,
    none: Option<&'a mut V>,
}

impl<'a, K, V> Iterator for IterMut<'a, K, V>
where
    K: Key<K, V>,
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

pub struct IntoIter<K, V>
where
    K: Key<K, V>,
{
    some: <K::Storage as Storage<K, V>>::IntoIter,
    none: Option<V>,
}

impl<K, V> Iterator for IntoIter<K, V>
where
    K: Key<K, V>,
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
    K: Key<K, V>,
{
    type Iter<'this> = Iter<'this, K, V> where Self: 'this;
    type Values<'this> = Values<'this, K, V> where Self: 'this;
    type IterMut<'this> = IterMut<'this, K, V> where Self: 'this;
    type IntoIter = IntoIter<K, V>;

    #[inline]
    fn insert(&mut self, key: Option<K>, value: V) -> Option<V> {
        match key {
            Some(key) => self.some.insert(key, value),
            None => mem::replace(&mut self.none, Some(value)),
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
    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            some: self.some.into_iter(),
            none: self.none,
        }
    }
}
