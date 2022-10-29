//! Module that defines the [`SetStorage`] trait.

mod singleton;
pub use self::singleton::SingletonSetStorage;

mod boolean;
pub use self::boolean::BooleanSetStorage;

#[cfg(feature = "map")]
mod hashbrown;
#[cfg(feature = "map")]
pub use self::hashbrown::HashbrownSetStorage;

mod option;
pub use self::option::OptionSetStorage;

/// The trait defining how storage works for [`Set`][crate::Set].
///
/// # Type Arguments
///
/// - `T` is the key being stored.
pub trait SetStorage<T>: Sized {
    /// Immutable iterator over storage.
    type Iter<'this>: Iterator<Item = T>
    where
        Self: 'this;

    /// Owning iterator over the storage.
    type IntoIter: Iterator<Item = T>;

    /// Construct empty storage.
    fn empty() -> Self;

    /// Get the length of storage.
    fn len(&self) -> usize;

    /// Check if storage is empty.
    fn is_empty(&self) -> bool;

    /// This is the storage abstraction for [`Set::insert`][crate::Set::insert].
    fn insert(&mut self, value: T) -> bool;

    /// This is the storage abstraction for [`Set::contains`][crate::Set::contains].
    fn contains(&self, value: T) -> bool;

    /// This is the storage abstraction for [`Set::remove`][crate::Set::remove].
    fn remove(&mut self, value: T) -> bool;

    /// This is the storage abstraction for [`Set::retain`][crate::Set::retain].
    fn retain<F>(&mut self, f: F)
    where
        F: FnMut(T) -> bool;

    /// This is the storage abstraction for [`Set::clear`][crate::Set::clear].
    fn clear(&mut self);

    /// This is the storage abstraction for [`Set::iter`][crate::Set::iter].
    fn iter(&self) -> Self::Iter<'_>;

    /// This is the storage abstraction for [`Set::into_iter`][crate::Set::into_iter].
    fn into_iter(self) -> Self::IntoIter;
}
