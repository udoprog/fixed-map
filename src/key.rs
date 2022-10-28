//! Module for the trait to define a `Key`.

#[cfg(feature = "map")]
use crate::storage::MapStorage;
use crate::storage::{BooleanStorage, OptionStorage, SingletonStorage, Storage};

/// The trait for a key that can be used to store values in a
/// [`Map`][crate::Set] or [`Set`][crate::Set].
///
/// This can be derived automatically from enums. The following is a *simple*
/// key which has no nested keys:
///
/// ```
/// use fixed_map::Key;
///
/// #[derive(Clone, Copy, Key)]
/// enum MyKey {
///     First,
///     Second,
/// }
/// ```
///
/// *Composite keys* are when keys structurally includes other keys. They have
/// slightly worse performance than simple keys because they can't be simply
/// arranged as arrays internally. `bool` below here implements [`Key`] and
/// using it in one key constructs a composite key. It's a simple form of key
/// since it can only inhabit two values - `true` or `false`. `Option<K>` can
/// also be used as a composite key:
///
/// ```
/// use fixed_map::Key;
///
/// #[derive(Clone, Copy, Key)]
/// enum MyKey {
///     First(bool),
///     Second(Option<bool>),
/// }
/// ```
///
/// Some composite keys require dynamic storage since they can inhabit a large
/// number of values, and preferrably should be avoided in favor of using a
/// `HashMap` directly. But if you absolutely have to you can enable the `map`
/// feature:
///
/// ```
/// use fixed_map::Key;
///
/// #[derive(Clone, Copy, Key)]
/// enum MyKey {
///     # #[cfg(feature = "map")]
///     First(u32),
///     Second,
/// }
/// ```
///
/// ## Ordering
///
/// Keys provide their own ordering semantics instead of relying on the
/// [`PartialOrd`] and [`Ord`] traits.
///
/// Therefore keys when stored in a collection such as [`Map`] and [`Set`] are
/// always ordered in *declaration order*. This allows those containers
/// themselves to be ordered if the underlying key supports, it similarly to how
/// [`BTreeMap`] and [`BTreeSet`] works.
///
/// ```
/// use fixed_map::{Key, Set};
///
/// #[derive(Clone, Copy, Key)]
/// enum Key {
///     First,
///     Second,
///     Third,
/// }
///
/// let mut a = Set::new();
/// a.insert(Key::First);
///
/// let mut b = Set::new();
/// b.insert(Key::Third);
///
/// let mut c = Set::new();
/// c.insert(Key::First);
/// c.insert(Key::Third);
///
/// assert!(a < b);
/// assert!(c < b);
/// assert!(a < c);
/// ```
///
/// The same example with [`BTreeSet`]:
///
/// ```
/// use std::collections::BTreeSet;
///
/// #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
/// enum Key {
///     First,
///     Second,
///     Third,
/// }
///
/// let mut a = BTreeSet::new();
/// a.insert(Key::First);
///
/// let mut b = BTreeSet::new();
/// b.insert(Key::Third);
///
/// let mut c = BTreeSet::new();
/// c.insert(Key::First);
/// c.insert(Key::Third);
///
/// assert!(a < b);
/// assert!(c < b);
/// assert!(a < c);
/// ```
///
/// [`BTreeMap`]: std::collections::BTreeMap
/// [`BTreeSet`]: std::collections::BTreeSet
/// [`Map`]: crate::Map
/// [`Set`]: crate::Set
pub trait Key: Copy {
    /// The `Storage` implementation to use for the key implementing this trait.
    type Storage<V>: Storage<Self, V>;
}

impl Key for bool {
    type Storage<V> = BooleanStorage<V>;
}

impl<K> Key for Option<K>
where
    K: Key,
{
    type Storage<V> = OptionStorage<K, V>;
}

macro_rules! map_key {
    ($ty:ty) => {
        #[cfg(feature = "map")]
        impl Key for $ty {
            type Storage<V> = MapStorage<$ty, V>;
        }
    };
}

macro_rules! singleton_key {
    ($ty:ty) => {
        impl Key for $ty {
            type Storage<V> = SingletonStorage<V>;
        }
    };
}

map_key!(char);
map_key!(u8);
map_key!(u32);
map_key!(u64);
map_key!(u128);
map_key!(usize);
map_key!(i8);
map_key!(i32);
map_key!(i64);
map_key!(i128);
map_key!(isize);
map_key!(&'static str);
map_key!(&'static [u8]);
singleton_key!(());
