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

impl<V: 'static> Storage<bool, V> for BooleanStorage<V> {
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
    fn iter<'a>(&'a self, mut f: impl FnMut((bool, &'a V))) {
        if let Some(v) = self.t.as_ref() {
            f((true, v));
        }

        if let Some(v) = self.f.as_ref() {
            f((false, v));
        }
    }

    #[inline]
    fn iter_mut<'a>(&'a mut self, mut f: impl FnMut((bool, &'a mut V))) {
        if let Some(v) = self.t.as_mut() {
            f((true, v));
        }

        if let Some(v) = self.f.as_mut() {
            f((false, v));
        }
    }
}
