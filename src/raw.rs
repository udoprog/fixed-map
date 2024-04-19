//! Raw access to underlying storage.
//!
//! This can be useful to implement more efficient serialization, since it might
//! provide access to smaller primitive values.

/// Trait implemented for storage which can be easily converted to and from a
/// raw value.
///
/// This is implemented for [`SetStorage`] when the `#[key(bitset)]` attribute
/// is present.
///
/// [`SetStorage`]: crate::set::SetStorage
pub trait RawStorage: Sized {
    /// The backing raw value.
    type Value;

    /// Get the raw value of the storage.
    fn as_raw(&self) -> Self::Value;

    /// Build storage from raw storage.
    fn from_raw(raw: Self::Value) -> Self;
}
