use crate::{key::Key, storage::Storage};
use std::mem;

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
    fn clone(&self) -> Self {
        OptionStorage {
            some: self.some.clone(),
            none: self.none.clone(),
        }
    }
}

impl<K, V> Default for OptionStorage<K, V>
where
    K: Key<K, V>,
{
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

pub struct Iter<K, V>
where
    K: Key<K, V>,
{
    some: <K::Storage as Storage<K, V>>::Iter,
    none: Option<*const V>,
}

impl<K, V> Clone for Iter<K, V>
where
    K: Key<K, V>,
{
    fn clone(&self) -> Iter<K, V> {
        Iter {
            some: self.some.clone(),
            none: self.none.clone(),
        }
    }
}

impl<K, V> Iterator for Iter<K, V>
where
    K: Key<K, V>,
{
    type Item = (Option<K>, *const V);

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

pub struct IterMut<K, V>
where
    K: Key<K, V>,
{
    some: <K::Storage as Storage<K, V>>::IterMut,
    none: Option<*mut V>,
}

impl<K, V> Iterator for IterMut<K, V>
where
    K: Key<K, V>,
{
    type Item = (Option<K>, *mut V);

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
    type Iter = Iter<K, V>;
    type IterMut = IterMut<K, V>;

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
    fn iter(&self) -> Self::Iter {
        Iter {
            some: self.some.iter(),
            none: self.none.as_ref().map(|v| v as *const V),
        }
    }

    #[inline]
    fn iter_mut(&mut self) -> Self::IterMut {
        IterMut {
            some: self.some.iter_mut(),
            none: self.none.as_mut().map(|v| v as *mut V),
        }
    }
}
