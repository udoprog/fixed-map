use crate::Storage;

/// The trait for a key that can be used to store values in the maps.
pub trait Key<K: 'static, V: 'static>: Copy {
    /// The `Storage` implementation to use for the key implementing this trait.
    type Storage: Storage<K, V>;
}
