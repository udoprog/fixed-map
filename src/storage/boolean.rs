use core::mem;

use crate::storage::Storage;

/// Storage for `bool`s.
pub struct BooleanStorage<V> {
    t: Option<V>,
    f: Option<V>,
}

impl<V> Clone for BooleanStorage<V>
where
    V: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        BooleanStorage {
            t: self.t.clone(),
            f: self.f.clone(),
        }
    }
}

impl<V> Default for BooleanStorage<V> {
    #[inline]
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
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.t == other.t && self.f == other.f
    }
}

impl<V> Eq for BooleanStorage<V> where V: Eq {}

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
        if let Some(t) = self.t.take() {
            return Some((true, t));
        }

        if let Some(f) = self.f.take() {
            return Some((false, f));
        }

        None
    }
}

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
        self.t.take().or_else(|| self.f.take())
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

impl<V> Storage<bool, V> for BooleanStorage<V> {
    type Iter<'this> = Iter<'this, V> where Self: 'this;
    type Values<'this> = Values<'this, V> where Self: 'this;
    type IterMut<'this> = IterMut<'this, V> where Self: 'this;
    type IntoIter = IntoIter<V>;

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
    fn iter(&self) -> Self::Iter<'_> {
        Iter {
            t: self.t.as_ref(),
            f: self.f.as_ref(),
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
    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            t: self.t,
            f: self.f,
        }
    }
}
