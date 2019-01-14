use crate::storage::Storage;
use std::mem;

/// Storage for `bool`s.
pub struct BooleanStorage<V> {
    t: Option<V>,
    f: Option<V>,
}

impl<V> Clone for BooleanStorage<V>
where
    V: Clone,
{
    fn clone(&self) -> Self {
        BooleanStorage {
            t: self.t.clone(),
            f: self.f.clone(),
        }
    }
}

impl<V> Default for BooleanStorage<V> {
    fn default() -> Self {
        Self {
            t: Default::default(),
            f: Default::default(),
        }
    }
}

impl<V> PartialEq for BooleanStorage<V>
where
    V: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.t == other.t && self.f == other.f
    }
}

impl<V> Eq for BooleanStorage<V> where V: Eq {}

pub struct Iter<V> {
    t: Option<*const V>,
    f: Option<*const V>,
}

impl<V> Clone for Iter<V> {
    fn clone(&self) -> Iter<V> {
        Iter {
            t: self.t.clone(),
            f: self.f.clone(),
        }
    }
}

impl<V> Iterator for Iter<V> {
    type Item = (bool, *const V);

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

pub struct IterMut<V> {
    t: Option<*mut V>,
    f: Option<*mut V>,
}

impl<V> Iterator for IterMut<V> {
    type Item = (bool, *mut V);

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

impl<V> Storage<bool, V> for BooleanStorage<V> {
    type Iter = Iter<V>;
    type IterMut = IterMut<V>;

    #[inline]
    fn insert(&mut self, key: bool, value: V) -> Option<V> {
        match key {
            true => mem::replace(&mut self.t, Some(value)),
            false => mem::replace(&mut self.f, Some(value)),
        }
    }

    #[inline]
    fn get(&self, key: bool) -> Option<&V> {
        match key {
            true => self.t.as_ref(),
            false => self.f.as_ref(),
        }
    }

    #[inline]
    fn get_mut(&mut self, key: bool) -> Option<&mut V> {
        match key {
            true => self.t.as_mut(),
            false => self.f.as_mut(),
        }
    }

    #[inline]
    fn remove(&mut self, key: bool) -> Option<V> {
        match key {
            true => mem::replace(&mut self.t, None),
            false => mem::replace(&mut self.f, None),
        }
    }

    #[inline]
    fn clear(&mut self) {
        self.t = None;
        self.f = None;
    }

    #[inline]
    fn iter(&self) -> Self::Iter {
        Iter {
            t: self.t.as_ref().map(|v| v as *const V),
            f: self.f.as_ref().map(|v| v as *const V),
        }
    }

    #[inline]
    fn iter_mut(&mut self) -> Self::IterMut {
        IterMut {
            t: self.t.as_mut().map(|v| v as *mut V),
            f: self.f.as_mut().map(|v| v as *mut V),
        }
    }
}
