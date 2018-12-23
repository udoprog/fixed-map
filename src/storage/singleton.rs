use crate::storage::Storage;
use std::marker;
use std::mem;

/// Storage types that only has a single value (like `()`).
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
