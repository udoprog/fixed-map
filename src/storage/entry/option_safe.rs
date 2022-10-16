#![allow(clippy::single_match_else, clippy::or_fun_call)]

use crate::{key::Key, storage::entry::StorageEntry, storage::Storage};

struct Entry<'a, K: Key, V> {
    key: Option<K>,
    some: &'a mut K::Storage<V>,
    none: &'a mut Option<V>,
}

impl<'a, K: Key, V> Entry<'a, K, V> {
    fn or_insert(self, default: V) -> &'a mut V {
        match self.key {
            Some(key) => self.some.entry(key).or_insert(default),
            None => self.none.get_or_insert(default),
        }
    }

    fn or_insert_with<F: FnOnce() -> V>(self, default: F) -> &'a mut V {
        match self.key {
            Some(key) => self.some.entry(key).or_insert_with(default),
            None => self.none.get_or_insert_with(default),
        }
    }

    fn or_insert_with_key<F: FnOnce(Option<K>) -> V>(self, default: F) -> &'a mut V {
        match self.key {
            Some(key) => self
                .some
                .entry(key)
                .or_insert_with_key(|k| default(Some(k))),
            None => self.none.get_or_insert_with(|| default(self.key)),
        }
    }

    fn key(&self) -> Option<K> {
        self.key
    }

    fn and_modify<F: FnOnce(&mut V)>(self, f: F) -> Self {
        match self.key {
            Some(key) => {
                if let Some(val) = self.some.get_mut(key) {
                    f(val);
                }
                self
            }
            None => {
                if let Some(val) = self.none.as_mut() {
                    f(val);
                }
                self
            }
        }
    }

    fn or_default(self) -> &'a mut V
    where
        V: Default,
    {
        self.or_insert(V::default())
    }
}
