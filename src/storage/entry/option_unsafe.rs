#![allow(unsafe_code)]

use core::{marker::PhantomData, mem};

use crate::{key::Key, storage::Storage};

/// Abstraction for an `Option` that's known to be `Some`
struct SomeBucket<'a, V> {
    opt: *mut Option<V>,
    inner: *mut V,
    _life: PhantomData<&'a mut Option<V>>,
}
impl<'a, V> SomeBucket<'a, V> {
    fn new(opt: &'a mut Option<V>) -> Option<Self> {
        let opt_ptr: *mut Option<V> = opt;

        opt.as_mut().map(|inner| SomeBucket {
            opt: opt_ptr,
            inner,
            _life: PhantomData,
        })
    }

    fn as_ref(&self) -> &V {
        unsafe { &(*self.inner) }
    }

    fn as_mut(&mut self) -> &mut V {
        unsafe { &mut (*self.inner) }
    }

    fn into_mut(self) -> &'a mut V {
        unsafe { &mut (*self.inner) }
    }

    fn take(self) -> V {
        unsafe {
            let value = self.inner.read();
            self.opt.write(None);
            value
        }
    }
}

struct VacantEntryNone<'a, K: Key, V> {
    _key: PhantomData<K>,
    none: &'a mut Option<V>,
}

struct OccupiedEntryNone<'a, K: Key, V> {
    _key: PhantomData<K>,
    none: SomeBucket<'a, V>,
}

struct VacantEntrySome<'a, K: Key, V> {
    key: K,
    some: &'a mut K::Storage<V>,
}

struct OccupiedEntrySome<'a, K: Key, V> {
    key: K,
    some: &'a mut K::Storage<V>,
}

enum VacantEntryEither<'a, K: Key, V> {
    None(VacantEntryNone<'a, K, V>),
    Some(VacantEntrySome<'a, K, V>)
}

struct VacantEntry<'a, K: Key, V> {
    either: VacantEntryEither<'a, K, V>
}

enum OccupiedEntryEither<'a, K: Key, V> {
    None(OccupiedEntryNone<'a, K, V>),
    Some(OccupiedEntrySome<'a, K, V>)
}

struct OccupiedEntry<'a, K: Key, V> {
    either: OccupiedEntryEither<'a, K, V>
}

impl<'a, K: Key, V> VacantEntryNone<'a, K, V> {
    pub fn key(&self) -> Option<K> {
        None
    }

    pub fn insert(self, value: V) -> &'a mut V {
        *self.none = Some(value);
        unsafe { self.none.as_mut().unwrap_unchecked() }
    }
}

impl<'a, K: Key, V> VacantEntrySome<'a, K, V> {
    pub fn key(&self) -> Option<K> {
        Some(self.key)
    }

    pub fn insert(self, value: V) -> &'a mut V {
        self.some.insert(self.key, value);
        unsafe { self.some.get_mut(self.key).unwrap_unchecked() }
    }
}

impl<'a, K: Key, V> super::VacantEntry<'a, Option<K>, V> for VacantEntry<'a, K, V> {
    fn key(&self) -> Option<K> {
        match &self.either {
            VacantEntryEither::None(entry) => entry.key(),
            VacantEntryEither::Some(entry) => entry.key(),
        }
    }

    fn insert(self, value: V) -> &'a mut V {
        match self.either {
            VacantEntryEither::None(entry) => entry.insert(value),
            VacantEntryEither::Some(entry) => entry.insert(value),
        }
    }
}

impl<'a, K: Key, V> OccupiedEntryNone<'a, K, V> {
    pub fn key(&self) -> Option<K> {
        None
    }

    pub fn get(&self) -> &V {
        self.none.as_ref()
    }

    pub fn get_mut(&mut self) -> &mut V {
        self.none.as_mut()
    }

    pub fn into_mut(self) -> &'a mut V {
        self.none.into_mut()
    }

    pub fn insert(&mut self, value: V) -> V {
        mem::replace(self.none.as_mut(), value)
    }

    pub fn remove(self) -> V {
        self.none.take()
    }
}

impl<'a, K: Key, V> OccupiedEntrySome<'a, K, V> {
    pub fn key(&self) -> Option<K> {
        Some(self.key)
    }

    pub fn get(&self) -> &V {
        self.some.get(self.key).unwrap()
    }

    pub fn get_mut(&mut self) -> &mut V {
        self.some.get_mut(self.key).unwrap()
    }

    pub fn into_mut(self) -> &'a mut V {
        self.some.get_mut(self.key).unwrap()
    }

    pub fn insert(&mut self, value: V) -> V {
        self.some.insert(self.key, value).unwrap()
    }

    pub fn remove(self) -> V {
        self.some.remove(self.key).unwrap()
    }
}

impl<'a, K: Key, V> super::OccupiedEntry<'a, Option<K>, V> for OccupiedEntry<'a, K, V> {
    fn key(&self) -> Option<K> {
        match &self.either {
            OccupiedEntryEither::None(entry) => entry.key(),
            OccupiedEntryEither::Some(entry) => entry.key(),
        }
    }

    fn get(&self) -> &V {
        match &self.either {
            OccupiedEntryEither::None(entry) => entry.get(),
            OccupiedEntryEither::Some(entry) => entry.get(),
        }
    }

    fn get_mut(&mut self) -> &mut V {
        match &mut self.either {
            OccupiedEntryEither::None(entry) => entry.get_mut(),
            OccupiedEntryEither::Some(entry) => entry.get_mut(),
        }
    }

    fn into_mut(self) -> &'a mut V {
        match self.either {
            OccupiedEntryEither::None(entry) => entry.into_mut(),
            OccupiedEntryEither::Some(entry) => entry.into_mut(),
        }
    }

    fn insert(&mut self, value: V) -> V {
        match &mut self.either {
            OccupiedEntryEither::None(entry) => entry.insert(value),
            OccupiedEntryEither::Some(entry) => entry.insert(value),
        }
    }

    fn remove(self) -> V {
        match self.either {
            OccupiedEntryEither::None(entry) => entry.remove(),
            OccupiedEntryEither::Some(entry) => entry.remove(),
        }
    }
}

type Entry<'a, K, V> = super::Entry<OccupiedEntry<'a, K, V>, VacantEntry<'a, K, V>>;
