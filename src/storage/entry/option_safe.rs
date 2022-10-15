#![allow(clippy::single_match_else, clippy::or_fun_call)]

use crate::{key::Key, storage::Storage};

struct Entry<'a, K: Key, V> {
    key: Option<K>,
    some: &'a mut K::Storage<V>,
    none: &'a mut Option<V>,
}

impl<'a, K: Key, V> Entry<'a, K, V> {
    fn or_insert(self, default: V) -> &'a mut V {
        match self.key {
            Some(key) => {
                if !self.some.contains_key(key) {
                    self.some.insert(key, default);
                }
                self.some.get_mut(key).unwrap()
            },
            None => {
                self.none.get_or_insert(default)
            },
        }
    }

    fn or_insert_with<F: FnOnce() -> V>(self, default: F) -> &'a mut V {
        match self.key {
            Some(key) => {
                if !self.some.contains_key(key) {
                    self.some.insert(key, default());
                }
                self.some.get_mut(key).unwrap()
            },
            None => {
                self.none.get_or_insert_with(default)
            },
        }
    }

    fn or_insert_with_key<F: FnOnce(Option<K>) -> V>(self, default: F) -> &'a mut V {
        match self.key {
            Some(key) => {
                if !self.some.contains_key(key) {
                    self.some.insert(key, default(self.key));
                }
                self.some.get_mut(key).unwrap()
            },
            None => {
                self.none.get_or_insert_with(|| default(self.key))
            },
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
            },
        }
    }

    fn or_default(self) -> &'a mut V
    where
        V: Default
    {
        self.or_insert(V::default())
    }
}

// struct VacantEntry<'a, K: Key, V> {
//     key: K,
//     some: &'a mut K::Storage<V>,
//     none: &'a mut Option<V>,
// }

// struct OccupiedEntryNone<'a, K: Key, V> {
//     key: K,
//     some: &'a mut K::Storage<V>,
//     none: &'a mut Option<V>,
// }

// struct OccupiedEntrySome<'a, K: Key, V> {
//     key: K,
//     some: &'a mut K::Storage<V>,
//     none: &'a mut Option<V>,
// }

// pub enum Entry<'a, K: Key, V> {
//     VacantEntry(VacantEntry<'a, K, V>),
//     OccupiedEntryNone(OccupiedEntryNone<'a, K, V>),
//     OccupiedEntrySome(OccupiedEntrySome<'a, K, V>),
// }

// impl<'a, K: Key, V> OccupiedEntryNone<'a, K, V> {
//     pub fn key(&self) -> K {
//         self.key
//     }

//     pub fn get(&self) -> &V {
//         self.none.as_ref().unwrap()
//     }

//     pub fn get_mut(&mut self) -> &mut V {
//         self.none.as_mut().unwrap()
//     }

//     pub fn into_mut(self) -> &'a mut V {
//         self.none.as_mut().unwrap()
//     }

//     pub fn insert(&mut self, value: V) -> V {
//         self.none.replace(value).unwrap()
//     }

//     pub fn remove(self) -> V {
//         self.none.take().unwrap()
//     }
// }

// impl<'a, K: Key, V> VacantEntry<'a, K, V> {
//     pub fn key(&self) -> K {
//         self.key
//     }
// }

// use core::{fmt, mem};

// use crate::key::Key;
// use crate::storage::OptionStorage;
// use crate::storage::entry::bucket::Bucket;

// /// A view into a single entry in a map, which may either be vacant or occupied.
// ///
// /// This `enum` is constructed from the [`entry`] method on [`HashMap`].
// ///
// /// [`HashMap`]: struct.HashMap.html
// /// [`entry`]: struct.HashMap.html#method.entry
// ///
// /// # Examples
// ///
// /// ```
// /// use hashbrown::hash_map::{Entry, HashMap, OccupiedEntry};
// ///
// /// let mut map = HashMap::new();
// /// map.extend([("a", 10), ("b", 20), ("c", 30)]);
// /// assert_eq!(map.len(), 3);
// ///
// /// // Existing key (insert)
// /// let entry: Entry<_, _, _> = map.entry("a");
// /// let _raw_o: OccupiedEntry<_, _, _> = entry.insert(1);
// /// assert_eq!(map.len(), 3);
// /// // Nonexistent key (insert)
// /// map.entry("d").insert(4);
// ///
// /// // Existing key (or_insert)
// /// let v = map.entry("b").or_insert(2);
// /// assert_eq!(std::mem::replace(v, 2), 20);
// /// // Nonexistent key (or_insert)
// /// map.entry("e").or_insert(5);
// ///
// /// // Existing key (or_insert_with)
// /// let v = map.entry("c").or_insert_with(|| 3);
// /// assert_eq!(std::mem::replace(v, 3), 30);
// /// // Nonexistent key (or_insert_with)
// /// map.entry("f").or_insert_with(|| 6);
// ///
// /// println!("Our HashMap: {:?}", map);
// ///
// /// let mut vec: Vec<_> = map.iter().map(|(&k, &v)| (k, v)).collect();
// /// // The `Iter` iterator produces items in arbitrary order, so the
// /// // items must be sorted to test them against a sorted array.
// /// vec.sort_unstable();
// /// assert_eq!(vec, [("a", 1), ("b", 2), ("c", 3), ("d", 4), ("e", 5), ("f", 6)]);
// /// ```
// pub enum Entry<'a, K: Key, V>
// {
//     /// An occupied entry.
//     ///
//     /// # Examples
//     ///
//     /// ```
//     /// use hashbrown::hash_map::{Entry, HashMap};
//     /// let mut map: HashMap<_, _> = [("a", 100), ("b", 200)].into();
//     ///
//     /// match map.entry("a") {
//     ///     Entry::Vacant(_) => unreachable!(),
//     ///     Entry::Occupied(_) => { }
//     /// }
//     /// ```
//     Occupied(OccupiedEntry<'a, K, V>),

//     /// A vacant entry.
//     ///
//     /// # Examples
//     ///
//     /// ```
//     /// use hashbrown::hash_map::{Entry, HashMap};
//     /// let mut map: HashMap<&str, i32> = HashMap::new();
//     ///
//     /// match map.entry("a") {
//     ///     Entry::Occupied(_) => unreachable!(),
//     ///     Entry::Vacant(_) => { }
//     /// }
//     /// ```
//     Vacant(VacantEntry<'a, K, V>),
// }

// /// A view into an occupied entry in a `HashMap`.
// /// It is part of the [`Entry`] enum.
// ///
// /// [`Entry`]: enum.Entry.html
// ///
// /// # Examples
// ///
// /// ```
// /// use hashbrown::hash_map::{Entry, HashMap, OccupiedEntry};
// ///
// /// let mut map = HashMap::new();
// /// map.extend([("a", 10), ("b", 20), ("c", 30)]);
// ///
// /// let _entry_o: OccupiedEntry<_, _, _> = map.entry("a").insert(100);
// /// assert_eq!(map.len(), 3);
// ///
// /// // Existing key (insert and update)
// /// match map.entry("a") {
// ///     Entry::Vacant(_) => unreachable!(),
// ///     Entry::Occupied(mut view) => {
// ///         assert_eq!(view.get(), &100);
// ///         let v = view.get_mut();
// ///         *v *= 10;
// ///         assert_eq!(view.insert(1111), 1000);
// ///     }
// /// }
// ///
// /// assert_eq!(map[&"a"], 1111);
// /// assert_eq!(map.len(), 3);
// ///
// /// // Existing key (take)
// /// match map.entry("c") {
// ///     Entry::Vacant(_) => unreachable!(),
// ///     Entry::Occupied(view) => {
// ///         assert_eq!(view.remove_entry(), ("c", 30));
// ///     }
// /// }
// /// assert_eq!(map.get(&"c"), None);
// /// assert_eq!(map.len(), 2);
// /// ```
// pub struct OccupiedEntry<'a, K: Key, V> {
//     hash: u64,
//     key: Option<K>,
//     elem: Bucket<(K, V)>,
//     table: &'a mut OptionStorage<K, V>,
// }

// // unsafe impl<K, V> Send for OccupiedEntry<'_, K, V>
// // where
// //     K: Send,
// //     V: Send,
// // {
// // }
// // unsafe impl<K, V> Sync for OccupiedEntry<'_, K, V>
// // where
// //     K: Sync,
// //     V: Sync,
// // {
// // }

// impl<K: fmt::Debug + Key, V: fmt::Debug> fmt::Debug for OccupiedEntry<'_, K, V> {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         f.debug_struct("OccupiedEntry")
//             .field("key", self.key())
//             .field("value", self.get())
//             .finish()
//     }
// }

// /// A view into a vacant entry in a `HashMap`.
// /// It is part of the [`Entry`] enum.
// ///
// /// [`Entry`]: enum.Entry.html
// ///
// /// # Examples
// ///
// /// ```
// /// use hashbrown::hash_map::{Entry, HashMap, VacantEntry};
// ///
// /// let mut map = HashMap::<&str, i32>::new();
// ///
// /// let entry_v: VacantEntry<_, _, _> = match map.entry("a") {
// ///     Entry::Vacant(view) => view,
// ///     Entry::Occupied(_) => unreachable!(),
// /// };
// /// entry_v.insert(10);
// /// assert!(map[&"a"] == 10 && map.len() == 1);
// ///
// /// // Nonexistent key (insert and update)
// /// match map.entry("b") {
// ///     Entry::Occupied(_) => unreachable!(),
// ///     Entry::Vacant(view) => {
// ///         let value = view.insert(2);
// ///         assert_eq!(*value, 2);
// ///         *value = 20;
// ///     }
// /// }
// /// assert!(map[&"b"] == 20 && map.len() == 2);
// /// ```
// pub struct VacantEntry<'a, K: Key, V> {
//     hash: u64,
//     key: K,
//     table: &'a mut OptionStorage<K, V>,
// }

// impl<K: fmt::Debug + Key, V> fmt::Debug for VacantEntry<'_, K, V> {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         f.debug_tuple("VacantEntry").field(self.key()).finish()
//     }
// }

// impl<'a, K: Key, V> Entry<'a, K, V> {
//     /// Sets the value of the entry, and returns an OccupiedEntry.
//     ///
//     /// # Examples
//     ///
//     /// ```
//     /// use hashbrown::HashMap;
//     ///
//     /// let mut map: HashMap<&str, u32> = HashMap::new();
//     /// let entry = map.entry("horseyland").insert(37);
//     ///
//     /// assert_eq!(entry.key(), &"horseyland");
//     /// ```
//     #[inline]
//     pub fn insert(self, value: V) -> OccupiedEntry<'a, K, V>
//     {
//         match self {
//             Entry::Occupied(mut entry) => {
//                 entry.insert(value);
//                 entry
//             }
//             Entry::Vacant(entry) => entry.insert_entry(value),
//         }
//     }

//     /// Ensures a value is in the entry by inserting the default if empty, and returns
//     /// a mutable reference to the value in the entry.
//     ///
//     /// # Examples
//     ///
//     /// ```
//     /// use hashbrown::HashMap;
//     ///
//     /// let mut map: HashMap<&str, u32> = HashMap::new();
//     ///
//     /// // nonexistent key
//     /// map.entry("poneyland").or_insert(3);
//     /// assert_eq!(map["poneyland"], 3);
//     ///
//     /// // existing key
//     /// *map.entry("poneyland").or_insert(10) *= 2;
//     /// assert_eq!(map["poneyland"], 6);
//     /// ```
//     #[inline]
//     pub fn or_insert(self, default: V) -> &'a mut V
//     {
//         match self {
//             Entry::Occupied(entry) => entry.into_mut(),
//             Entry::Vacant(entry) => entry.insert(default),
//         }
//     }

//     /// Ensures a value is in the entry by inserting the result of the default function if empty,
//     /// and returns a mutable reference to the value in the entry.
//     ///
//     /// # Examples
//     ///
//     /// ```
//     /// use hashbrown::HashMap;
//     ///
//     /// let mut map: HashMap<&str, u32> = HashMap::new();
//     ///
//     /// // nonexistent key
//     /// map.entry("poneyland").or_insert_with(|| 3);
//     /// assert_eq!(map["poneyland"], 3);
//     ///
//     /// // existing key
//     /// *map.entry("poneyland").or_insert_with(|| 10) *= 2;
//     /// assert_eq!(map["poneyland"], 6);
//     /// ```
//     #[inline]
//     pub fn or_insert_with<F: FnOnce() -> V>(self, default: F) -> &'a mut V
//     {
//         match self {
//             Entry::Occupied(entry) => entry.into_mut(),
//             Entry::Vacant(entry) => entry.insert(default()),
//         }
//     }

//     /// Ensures a value is in the entry by inserting, if empty, the result of the default function.
//     /// This method allows for generating key-derived values for insertion by providing the default
//     /// function a reference to the key that was moved during the `.entry(key)` method call.
//     ///
//     /// The reference to the moved key is provided so that cloning or copying the key is
//     /// unnecessary, unlike with `.or_insert_with(|| ... )`.
//     ///
//     /// # Examples
//     ///
//     /// ```
//     /// use hashbrown::HashMap;
//     ///
//     /// let mut map: HashMap<&str, usize> = HashMap::new();
//     ///
//     /// // nonexistent key
//     /// map.entry("poneyland").or_insert_with_key(|key| key.chars().count());
//     /// assert_eq!(map["poneyland"], 9);
//     ///
//     /// // existing key
//     /// *map.entry("poneyland").or_insert_with_key(|key| key.chars().count() * 10) *= 2;
//     /// assert_eq!(map["poneyland"], 18);
//     /// ```
//     #[inline]
//     pub fn or_insert_with_key<F: FnOnce(&K) -> V>(self, default: F) -> &'a mut V
//     {
//         match self {
//             Entry::Occupied(entry) => entry.into_mut(),
//             Entry::Vacant(entry) => {
//                 let value = default(entry.key());
//                 entry.insert(value)
//             }
//         }
//     }

//     /// Returns a reference to this entry's key.
//     ///
//     /// # Examples
//     ///
//     /// ```
//     /// use hashbrown::HashMap;
//     ///
//     /// let mut map: HashMap<&str, u32> = HashMap::new();
//     /// map.entry("poneyland").or_insert(3);
//     /// // existing key
//     /// assert_eq!(map.entry("poneyland").key(), &"poneyland");
//     /// // nonexistent key
//     /// assert_eq!(map.entry("horseland").key(), &"horseland");
//     /// ```
//     #[inline]
//     pub fn key(&self) -> &K {
//         match *self {
//             Entry::Occupied(ref entry) => entry.key(),
//             Entry::Vacant(ref entry) => entry.key(),
//         }
//     }

//     /// Provides in-place mutable access to an occupied entry before any
//     /// potential inserts into the map.
//     ///
//     /// # Examples
//     ///
//     /// ```
//     /// use hashbrown::HashMap;
//     ///
//     /// let mut map: HashMap<&str, u32> = HashMap::new();
//     ///
//     /// map.entry("poneyland")
//     ///    .and_modify(|e| { *e += 1 })
//     ///    .or_insert(42);
//     /// assert_eq!(map["poneyland"], 42);
//     ///
//     /// map.entry("poneyland")
//     ///    .and_modify(|e| { *e += 1 })
//     ///    .or_insert(42);
//     /// assert_eq!(map["poneyland"], 43);
//     /// ```
//     #[inline]
//     pub fn and_modify<F>(self, f: F) -> Self
//     where
//         F: FnOnce(&mut V),
//     {
//         match self {
//             Entry::Occupied(mut entry) => {
//                 f(entry.get_mut());
//                 Entry::Occupied(entry)
//             }
//             Entry::Vacant(entry) => Entry::Vacant(entry),
//         }
//     }

//     /// Provides shared access to the key and owned access to the value of
//     /// an occupied entry and allows to replace or remove it based on the
//     /// value of the returned option.
//     ///
//     /// # Examples
//     ///
//     /// ```
//     /// use hashbrown::HashMap;
//     /// use hashbrown::hash_map::Entry;
//     ///
//     /// let mut map: HashMap<&str, u32> = HashMap::new();
//     ///
//     /// let entry = map
//     ///     .entry("poneyland")
//     ///     .and_replace_entry_with(|_k, _v| panic!());
//     ///
//     /// match entry {
//     ///     Entry::Vacant(e) => {
//     ///         assert_eq!(e.key(), &"poneyland");
//     ///     }
//     ///     Entry::Occupied(_) => panic!(),
//     /// }
//     ///
//     /// map.insert("poneyland", 42);
//     ///
//     /// let entry = map
//     ///     .entry("poneyland")
//     ///     .and_replace_entry_with(|k, v| {
//     ///         assert_eq!(k, &"poneyland");
//     ///         assert_eq!(v, 42);
//     ///         Some(v + 1)
//     ///     });
//     ///
//     /// match entry {
//     ///     Entry::Occupied(e) => {
//     ///         assert_eq!(e.key(), &"poneyland");
//     ///         assert_eq!(e.get(), &43);
//     ///     }
//     ///     Entry::Vacant(_) => panic!(),
//     /// }
//     ///
//     /// assert_eq!(map["poneyland"], 43);
//     ///
//     /// let entry = map
//     ///     .entry("poneyland")
//     ///     .and_replace_entry_with(|_k, _v| None);
//     ///
//     /// match entry {
//     ///     Entry::Vacant(e) => assert_eq!(e.key(), &"poneyland"),
//     ///     Entry::Occupied(_) => panic!(),
//     /// }
//     ///
//     /// assert!(!map.contains_key("poneyland"));
//     /// ```
//     #[inline]
//     pub fn and_replace_entry_with<F>(self, f: F) -> Self
//     where
//         F: FnOnce(&K, V) -> Option<V>,
//     {
//         match self {
//             Entry::Occupied(entry) => entry.replace_entry_with(f),
//             Entry::Vacant(_) => self,
//         }
//     }
// }

// impl<'a, K: Key, V: Default> Entry<'a, K, V> {
//     /// Ensures a value is in the entry by inserting the default value if empty,
//     /// and returns a mutable reference to the value in the entry.
//     ///
//     /// # Examples
//     ///
//     /// ```
//     /// use hashbrown::HashMap;
//     ///
//     /// let mut map: HashMap<&str, Option<u32>> = HashMap::new();
//     ///
//     /// // nonexistent key
//     /// map.entry("poneyland").or_default();
//     /// assert_eq!(map["poneyland"], None);
//     ///
//     /// map.insert("horseland", Some(3));
//     ///
//     /// // existing key
//     /// assert_eq!(map.entry("horseland").or_default(), &mut Some(3));
//     /// ```
//     #[inline]
//     pub fn or_default(self) -> &'a mut V
//     {
//         match self {
//             Entry::Occupied(entry) => entry.into_mut(),
//             Entry::Vacant(entry) => entry.insert(Default::default()),
//         }
//     }
// }

// impl<'a, K: Key, V> OccupiedEntry<'a, K, V> {
//     /// Gets a reference to the key in the entry.
//     ///
//     /// # Examples
//     ///
//     /// ```
//     /// use hashbrown::hash_map::{Entry, HashMap};
//     ///
//     /// let mut map: HashMap<&str, u32> = HashMap::new();
//     /// map.entry("poneyland").or_insert(12);
//     ///
//     /// match map.entry("poneyland") {
//     ///     Entry::Vacant(_) => panic!(),
//     ///     Entry::Occupied(entry) => assert_eq!(entry.key(), &"poneyland"),
//     /// }
//     /// ```
//     #[inline]
//     pub fn key(&self) -> &K {
//         unsafe { &self.elem.as_ref().0 }
//     }

//     /// Take the ownership of the key and value from the map.
//     /// Keeps the allocated memory for reuse.
//     ///
//     /// # Examples
//     ///
//     /// ```
//     /// use hashbrown::HashMap;
//     /// use hashbrown::hash_map::Entry;
//     ///
//     /// let mut map: HashMap<&str, u32> = HashMap::new();
//     /// // The map is empty
//     /// assert!(map.is_empty() && map.capacity() == 0);
//     ///
//     /// map.entry("poneyland").or_insert(12);
//     /// let capacity_before_remove = map.capacity();
//     ///
//     /// if let Entry::Occupied(o) = map.entry("poneyland") {
//     ///     // We delete the entry from the map.
//     ///     assert_eq!(o.remove_entry(), ("poneyland", 12));
//     /// }
//     ///
//     /// assert_eq!(map.contains_key("poneyland"), false);
//     /// // Now map hold none elements but capacity is equal to the old one
//     /// assert!(map.len() == 0 && map.capacity() == capacity_before_remove);
//     /// ```
//     #[inline]
//     pub fn remove_entry(self) -> (K, V) {
//         unsafe { self.table.table.remove(self.elem) }
//     }

//     /// Gets a reference to the value in the entry.
//     ///
//     /// # Examples
//     ///
//     /// ```
//     /// use hashbrown::HashMap;
//     /// use hashbrown::hash_map::Entry;
//     ///
//     /// let mut map: HashMap<&str, u32> = HashMap::new();
//     /// map.entry("poneyland").or_insert(12);
//     ///
//     /// match map.entry("poneyland") {
//     ///     Entry::Vacant(_) => panic!(),
//     ///     Entry::Occupied(entry) => assert_eq!(entry.get(), &12),
//     /// }
//     /// ```
//     #[inline]
//     pub fn get(&self) -> &V {
//         unsafe { &self.elem.as_ref().1 }
//     }

//     /// Gets a mutable reference to the value in the entry.
//     ///
//     /// If you need a reference to the `OccupiedEntry` which may outlive the
//     /// destruction of the `Entry` value, see [`into_mut`].
//     ///
//     /// [`into_mut`]: #method.into_mut
//     ///
//     /// # Examples
//     ///
//     /// ```
//     /// use hashbrown::HashMap;
//     /// use hashbrown::hash_map::Entry;
//     ///
//     /// let mut map: HashMap<&str, u32> = HashMap::new();
//     /// map.entry("poneyland").or_insert(12);
//     ///
//     /// assert_eq!(map["poneyland"], 12);
//     /// if let Entry::Occupied(mut o) = map.entry("poneyland") {
//     ///     *o.get_mut() += 10;
//     ///     assert_eq!(*o.get(), 22);
//     ///
//     ///     // We can use the same Entry multiple times.
//     ///     *o.get_mut() += 2;
//     /// }
//     ///
//     /// assert_eq!(map["poneyland"], 24);
//     /// ```
//     #[inline]
//     pub fn get_mut(&mut self) -> &mut V {
//         unsafe { &mut self.elem.as_mut().1 }
//     }

//     /// Converts the OccupiedEntry into a mutable reference to the value in the entry
//     /// with a lifetime bound to the map itself.
//     ///
//     /// If you need multiple references to the `OccupiedEntry`, see [`get_mut`].
//     ///
//     /// [`get_mut`]: #method.get_mut
//     ///
//     /// # Examples
//     ///
//     /// ```
//     /// use hashbrown::hash_map::{Entry, HashMap};
//     ///
//     /// let mut map: HashMap<&str, u32> = HashMap::new();
//     /// map.entry("poneyland").or_insert(12);
//     ///
//     /// assert_eq!(map["poneyland"], 12);
//     ///
//     /// let value: &mut u32;
//     /// match map.entry("poneyland") {
//     ///     Entry::Occupied(entry) => value = entry.into_mut(),
//     ///     Entry::Vacant(_) => panic!(),
//     /// }
//     /// *value += 10;
//     ///
//     /// assert_eq!(map["poneyland"], 22);
//     /// ```
//     #[inline]
//     pub fn into_mut(self) -> &'a mut V {
//         unsafe { &mut self.elem.as_mut().1 }
//     }

//     /// Sets the value of the entry, and returns the entry's old value.
//     ///
//     /// # Examples
//     ///
//     /// ```
//     /// use hashbrown::HashMap;
//     /// use hashbrown::hash_map::Entry;
//     ///
//     /// let mut map: HashMap<&str, u32> = HashMap::new();
//     /// map.entry("poneyland").or_insert(12);
//     ///
//     /// if let Entry::Occupied(mut o) = map.entry("poneyland") {
//     ///     assert_eq!(o.insert(15), 12);
//     /// }
//     ///
//     /// assert_eq!(map["poneyland"], 15);
//     /// ```
//     #[inline]
//     pub fn insert(&mut self, value: V) -> V {
//         mem::replace(self.get_mut(), value)
//     }

//     /// Takes the value out of the entry, and returns it.
//     /// Keeps the allocated memory for reuse.
//     ///
//     /// # Examples
//     ///
//     /// ```
//     /// use hashbrown::HashMap;
//     /// use hashbrown::hash_map::Entry;
//     ///
//     /// let mut map: HashMap<&str, u32> = HashMap::new();
//     /// // The map is empty
//     /// assert!(map.is_empty() && map.capacity() == 0);
//     ///
//     /// map.entry("poneyland").or_insert(12);
//     /// let capacity_before_remove = map.capacity();
//     ///
//     /// if let Entry::Occupied(o) = map.entry("poneyland") {
//     ///     assert_eq!(o.remove(), 12);
//     /// }
//     ///
//     /// assert_eq!(map.contains_key("poneyland"), false);
//     /// // Now map hold none elements but capacity is equal to the old one
//     /// assert!(map.len() == 0 && map.capacity() == capacity_before_remove);
//     /// ```
//     #[inline]
//     pub fn remove(self) -> V {
//         self.remove_entry().1
//     }

//     /// Replaces the entry, returning the old key and value. The new key in the hash map will be
//     /// the key used to create this entry.
//     ///
//     /// # Panics
//     ///
//     /// Will panic if this OccupiedEntry was created through [`Entry::insert`].
//     ///
//     /// # Examples
//     ///
//     /// ```
//     ///  use hashbrown::hash_map::{Entry, HashMap};
//     ///  use std::rc::Rc;
//     ///
//     ///  let mut map: HashMap<Rc<String>, u32> = HashMap::new();
//     ///  let key_one = Rc::new("Stringthing".to_string());
//     ///  let key_two = Rc::new("Stringthing".to_string());
//     ///
//     ///  map.insert(key_one.clone(), 15);
//     ///  assert!(Rc::strong_count(&key_one) == 2 && Rc::strong_count(&key_two) == 1);
//     ///
//     ///  match map.entry(key_two.clone()) {
//     ///      Entry::Occupied(entry) => {
//     ///          let (old_key, old_value): (Rc<String>, u32) = entry.replace_entry(16);
//     ///          assert!(Rc::ptr_eq(&key_one, &old_key) && old_value == 15);
//     ///      }
//     ///      Entry::Vacant(_) => panic!(),
//     ///  }
//     ///
//     ///  assert!(Rc::strong_count(&key_one) == 1 && Rc::strong_count(&key_two) == 2);
//     ///  assert_eq!(map[&"Stringthing".to_owned()], 16);
//     /// ```
//     #[inline]
//     pub fn replace_entry(self, value: V) -> (K, V) {
//         let entry = unsafe { self.elem.as_mut() };

//         let old_key = mem::replace(&mut entry.0, self.key.unwrap());
//         let old_value = mem::replace(&mut entry.1, value);

//         (old_key, old_value)
//     }

//     /// Replaces the key in the hash map with the key used to create this entry.
//     ///
//     /// # Panics
//     ///
//     /// Will panic if this OccupiedEntry was created through [`Entry::insert`].
//     ///
//     /// # Examples
//     ///
//     /// ```
//     /// use hashbrown::hash_map::{Entry, HashMap};
//     /// use std::rc::Rc;
//     ///
//     /// let mut map: HashMap<Rc<String>, usize> = HashMap::with_capacity(6);
//     /// let mut keys_one: Vec<Rc<String>> = Vec::with_capacity(6);
//     /// let mut keys_two: Vec<Rc<String>> = Vec::with_capacity(6);
//     ///
//     /// for (value, key) in ["a", "b", "c", "d", "e", "f"].into_iter().enumerate() {
//     ///     let rc_key = Rc::new(key.to_owned());
//     ///     keys_one.push(rc_key.clone());
//     ///     map.insert(rc_key.clone(), value);
//     ///     keys_two.push(Rc::new(key.to_owned()));
//     /// }
//     ///
//     /// assert!(
//     ///     keys_one.iter().all(|key| Rc::strong_count(key) == 2)
//     ///         && keys_two.iter().all(|key| Rc::strong_count(key) == 1)
//     /// );
//     ///
//     /// reclaim_memory(&mut map, &keys_two);
//     ///
//     /// assert!(
//     ///     keys_one.iter().all(|key| Rc::strong_count(key) == 1)
//     ///         && keys_two.iter().all(|key| Rc::strong_count(key) == 2)
//     /// );
//     ///
//     /// fn reclaim_memory(map: &mut HashMap<Rc<String>, usize>, keys: &[Rc<String>]) {
//     ///     for key in keys {
//     ///         if let Entry::Occupied(entry) = map.entry(key.clone()) {
//     ///         // Replaces the entry's key with our version of it in `keys`.
//     ///             entry.replace_key();
//     ///         }
//     ///     }
//     /// }
//     /// ```
//     #[inline]
//     pub fn replace_key(self) -> K {
//         let entry = unsafe { self.elem.as_mut() };
//         mem::replace(&mut entry.0, self.key.unwrap())
//     }

//     /// Provides shared access to the key and owned access to the value of
//     /// the entry and allows to replace or remove it based on the
//     /// value of the returned option.
//     ///
//     /// # Examples
//     ///
//     /// ```
//     /// use hashbrown::HashMap;
//     /// use hashbrown::hash_map::Entry;
//     ///
//     /// let mut map: HashMap<&str, u32> = HashMap::new();
//     /// map.insert("poneyland", 42);
//     ///
//     /// let entry = match map.entry("poneyland") {
//     ///     Entry::Occupied(e) => {
//     ///         e.replace_entry_with(|k, v| {
//     ///             assert_eq!(k, &"poneyland");
//     ///             assert_eq!(v, 42);
//     ///             Some(v + 1)
//     ///         })
//     ///     }
//     ///     Entry::Vacant(_) => panic!(),
//     /// };
//     ///
//     /// match entry {
//     ///     Entry::Occupied(e) => {
//     ///         assert_eq!(e.key(), &"poneyland");
//     ///         assert_eq!(e.get(), &43);
//     ///     }
//     ///     Entry::Vacant(_) => panic!(),
//     /// }
//     ///
//     /// assert_eq!(map["poneyland"], 43);
//     ///
//     /// let entry = match map.entry("poneyland") {
//     ///     Entry::Occupied(e) => e.replace_entry_with(|_k, _v| None),
//     ///     Entry::Vacant(_) => panic!(),
//     /// };
//     ///
//     /// match entry {
//     ///     Entry::Vacant(e) => {
//     ///         assert_eq!(e.key(), &"poneyland");
//     ///     }
//     ///     Entry::Occupied(_) => panic!(),
//     /// }
//     ///
//     /// assert!(!map.contains_key("poneyland"));
//     /// ```
//     #[inline]
//     pub fn replace_entry_with<F>(self, f: F) -> Entry<'a, K, V>
//     where
//         F: FnOnce(&K, V) -> Option<V>,
//     {
//         unsafe {
//             let mut spare_key = None;

//             self.table
//                 .table
//                 .replace_bucket_with(self.elem.clone(), |(key, value)| {
//                     if let Some(new_value) = f(&key, value) {
//                         Some((key, new_value))
//                     } else {
//                         spare_key = Some(key);
//                         None
//                     }
//                 });

//             if let Some(key) = spare_key {
//                 Entry::Vacant(VacantEntry {
//                     hash: self.hash,
//                     key,
//                     table: self.table,
//                 })
//             } else {
//                 Entry::Occupied(self)
//             }
//         }
//     }
// }

// impl<'a, K: Key, V> VacantEntry<'a, K, V> {
//     /// Gets a reference to the key that would be used when inserting a value
//     /// through the `VacantEntry`.
//     ///
//     /// # Examples
//     ///
//     /// ```
//     /// use hashbrown::HashMap;
//     ///
//     /// let mut map: HashMap<&str, u32> = HashMap::new();
//     /// assert_eq!(map.entry("poneyland").key(), &"poneyland");
//     /// ```
//     #[inline]
//     pub fn key(&self) -> &K {
//         &self.key
//     }

//     /// Take ownership of the key.
//     ///
//     /// # Examples
//     ///
//     /// ```
//     /// use hashbrown::hash_map::{Entry, HashMap};
//     ///
//     /// let mut map: HashMap<&str, u32> = HashMap::new();
//     ///
//     /// match map.entry("poneyland") {
//     ///     Entry::Occupied(_) => panic!(),
//     ///     Entry::Vacant(v) => assert_eq!(v.into_key(), "poneyland"),
//     /// }
//     /// ```
//     #[inline]
//     pub fn into_key(self) -> K {
//         self.key
//     }

//     /// Sets the value of the entry with the VacantEntry's key,
//     /// and returns a mutable reference to it.
//     ///
//     /// # Examples
//     ///
//     /// ```
//     /// use hashbrown::HashMap;
//     /// use hashbrown::hash_map::Entry;
//     ///
//     /// let mut map: HashMap<&str, u32> = HashMap::new();
//     ///
//     /// if let Entry::Vacant(o) = map.entry("poneyland") {
//     ///     o.insert(37);
//     /// }
//     /// assert_eq!(map["poneyland"], 37);
//     /// ```
//     #[inline]
//     pub fn insert(self, value: V) -> &'a mut V
//     {
//         let table = &mut self.table.table;
//         let entry = table.insert_entry(
//             self.hash,
//             (self.key, value),
//             make_hasher::<K, _, V, S>(&self.table.hash_builder),
//         );
//         &mut entry.1
//     }

//     #[inline]
//     pub(crate) fn insert_entry(self, value: V) -> OccupiedEntry<'a, K, V>
//     {
//         let elem = self.table.table.insert(
//             self.hash,
//             (self.key, value),
//             make_hasher::<K, _, V, S>(&self.table.hash_builder),
//         );
//         OccupiedEntry {
//             hash: self.hash,
//             key: None,
//             elem,
//             table: self.table,
//         }
//     }
// }
