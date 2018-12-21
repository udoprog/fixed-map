use crate::Key;
use std::hash;
use std::marker;
use std::mem;

/// The trait defining how storage works.
///
/// # Type Arguments
///
/// - `K` is the key being stored.
/// - `V` is the value being stored.
pub trait Storage<K: 'static, V: 'static>: Default {
    /// This is the storage abstraction for [`Map::insert`](struct.Map.html#method.insert).
    fn insert(&mut self, key: K, value: V) -> Option<V>;

    /// This is the storage abstraction for [`Map::get`](struct.Map.html#method.get).
    fn get(&self, key: K) -> Option<&V>;

    /// This is the storage abstraction for [`Map::get_mut`](struct.Map.html#method.get_mut).
    fn get_mut(&mut self, key: K) -> Option<&mut V>;

    /// This is the storage abstraction for [`Map::remove`](struct.Map.html#method.remove).
    fn remove(&mut self, key: K) -> Option<V>;

    /// This is the storage abstraction for [`Map::clear`](struct.Map.html#method.clear).
    fn clear(&mut self);

    /// This is the storage abstraction for [`Map::iter`](struct.Map.html#method.iter).
    fn iter<'a, F>(&'a self, f: F)
    where
        F: FnMut((K, &'a V));

    /// This is the storage abstraction for [`Map::iter_mut`](struct.Map.html#method.iter_mut).
    fn iter_mut<'a, F>(&'a mut self, f: F)
    where
        F: FnMut((K, &'a mut V));
}

/// Storage for a type that only has a single value (like `()`).
pub struct SingletonStorage<K: 'static, V: 'static> {
    inner: Option<V>,
    key: marker::PhantomData<K>,
}

impl<K: 'static, V: 'static> Clone for SingletonStorage<K, V>
where
    V: Clone,
{
    fn clone(&self) -> Self {
        SingletonStorage {
            inner: self.inner.clone(),
            key: marker::PhantomData,
        }
    }
}

impl<K: 'static, V: 'static> Default for SingletonStorage<K, V> {
    fn default() -> Self {
        Self {
            inner: Default::default(),
            key: marker::PhantomData,
        }
    }
}

impl<K: 'static, V: 'static> Storage<K, V> for SingletonStorage<K, V>
where
    K: Default,
{
    #[inline]
    fn insert(&mut self, _: K, value: V) -> Option<V> {
        mem::replace(&mut self.inner, Some(value))
    }

    #[inline]
    fn get(&self, _: K) -> Option<&V> {
        self.inner.as_ref()
    }

    #[inline]
    fn get_mut(&mut self, _: K) -> Option<&mut V> {
        self.inner.as_mut()
    }

    #[inline]
    fn remove(&mut self, _: K) -> Option<V> {
        mem::replace(&mut self.inner, None)
    }

    #[inline]
    fn clear(&mut self) {
        self.inner = None;
    }

    #[inline]
    fn iter<'a, F>(&'a self, mut f: F)
    where
        F: FnMut((K, &'a V)),
    {
        if let Some(value) = self.inner.as_ref() {
            f((K::default(), value));
        }
    }

    #[inline]
    fn iter_mut<'a, F>(&'a mut self, mut f: F)
    where
        F: FnMut((K, &'a mut V)),
    {
        if let Some(value) = self.inner.as_mut() {
            f((K::default(), value));
        }
    }
}

/// Storage static types that must be stored in a map.
pub struct MapStorage<K: 'static, V: 'static> {
    inner: hashbrown::HashMap<K, V>,
}

impl<K: 'static, V: 'static> Clone for MapStorage<K, V>
where
    K: Clone + Eq + hash::Hash,
    V: Clone,
{
    fn clone(&self) -> Self {
        MapStorage {
            inner: self.inner.clone(),
        }
    }
}

impl<K: 'static, V: 'static> Default for MapStorage<K, V>
where
    K: Eq + hash::Hash,
{
    fn default() -> Self {
        Self {
            inner: Default::default(),
        }
    }
}

impl<K, V> Storage<K, V> for MapStorage<K, V>
where
    K: Copy + Eq + hash::Hash,
{
    #[inline]
    fn insert(&mut self, key: K, value: V) -> Option<V> {
        self.inner.insert(key, value)
    }

    #[inline]
    fn get(&self, key: K) -> Option<&V> {
        self.inner.get(&key)
    }

    #[inline]
    fn get_mut(&mut self, key: K) -> Option<&mut V> {
        self.inner.get_mut(&key)
    }

    #[inline]
    fn remove(&mut self, key: K) -> Option<V> {
        self.inner.remove(&key)
    }

    #[inline]
    fn clear(&mut self) {
        self.inner.clear();
    }

    #[inline]
    fn iter<'a, F>(&'a self, mut f: F)
    where
        F: FnMut((K, &'a V)),
    {
        for (key, value) in &self.inner {
            f((*key, value));
        }
    }

    #[inline]
    fn iter_mut<'a, F>(&'a mut self, mut f: F)
    where
        F: FnMut((K, &'a mut V)),
    {
        for (key, value) in &mut self.inner {
            f((*key, value));
        }
    }
}

impl<V: 'static> Key<&'static str, V> for &'static str {
    type Storage = MapStorage<Self, V>;
}

macro_rules! impl_map_storage {
    ($ty:ty) => {
        impl<V: 'static> Key<$ty, V> for $ty {
            type Storage = MapStorage<$ty, V>;
        }
    };
}

macro_rules! impl_singleton_storage {
    ($ty:ty) => {
        impl<V: 'static> Key<$ty, V> for $ty {
            type Storage = SingletonStorage<$ty, V>;
        }
    };
}

impl_map_storage!(u8);
impl_map_storage!(u32);
impl_map_storage!(u64);
impl_map_storage!(u128);
impl_map_storage!(usize);
impl_map_storage!(i8);
impl_map_storage!(i32);
impl_map_storage!(i64);
impl_map_storage!(i128);
impl_map_storage!(isize);
impl_singleton_storage!(());
