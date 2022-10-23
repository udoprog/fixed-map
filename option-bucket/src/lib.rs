//! Utility for working with [`Option`s][Option]
//! in cases where we want mutable access to the value within
//! and the [`Option`] itself (but not at the same time).
//!
//! Provides three types:
//! - [`SomeBucket`] allows accessing the value
//!   inside an [`Option`] known to be [`Some`] without `unwrap`
//! - [`NoneBucket`] allows optimally inserting a value
//!   into an [`Option`] known to be [`None`]
//! - [`OptionBucket`], an enum over the previous two,
//!   easily constructed from any `&mut Option`
//!
//! # Examples
//!
//! Safely implement [`Option::get_or_insert`]
//! ```
//! use option_bucket::OptionBucket;
//!
//! fn get_or_insert<T>(this: &mut Option<T>, value: T) -> &mut T {
//!     match OptionBucket::new(this) {
//!          OptionBucket::Some(some) => some.into_mut(),
//!          OptionBucket::None(none) => none.insert(value),
//!     }
//! }
//!
//! let mut x = None;
//! assert_eq!(get_or_insert(&mut x, 12), &12);
//! ```
//!
//! Safely implement entry API for [`Option`]
//! ```
//! use option_bucket::*;
//!
//! struct OccupiedEntry<'a, T> {
//!     inner: SomeBucket<'a, T>
//! }
//! struct VacantEntry<'a, T> {
//!     inner: NoneBucket<'a, T>,
//! }
//! enum Entry<'a, T> {
//!     Vacant(VacantEntry<'a, T>),
//!     Occupied(OccupiedEntry<'a, T>),
//! }
//!
//! impl<'a, T> VacantEntry<'a, T> {
//!     fn insert(self, value: T) -> &'a mut T {
//!         self.inner.insert(value)
//!     }
//! }
//! impl<'a, T> OccupiedEntry<'a, T> {
//!     fn get(&self) -> &T {
//!         self.inner.as_ref()
//!     }
//!     fn get_mut(&mut self) -> &mut T {
//!         self.inner.as_mut()
//!     }
//!     fn into_mut(self) -> &'a mut T {
//!         self.inner.into_mut()
//!     }
//!     fn insert(&mut self, value: T) -> T {
//!         self.inner.replace(value)
//!     }
//!     fn remove(self) -> T {
//!         self.inner.take()
//!     }
//! }
//!
//! fn option_entry<T>(this: &mut Option<T>) -> Entry<'_, T> {
//!     match OptionBucket::new(this) {
//!         OptionBucket::Some(inner) => Entry::Occupied(OccupiedEntry { inner }),
//!         OptionBucket::None(inner) => Entry::Vacant(VacantEntry { inner }),
//!     }
//! }
//! ```
//!
//! # Unsafe Usage
//!
//! This crate uses `unsafe` to dereference raw pointers and to
//! obtain mutable references to the inner value of `Some`s.

#![no_std]
#![deny(missing_docs)]
// Enable pedantic lints as warnings so we don't break builds when
// lints are modified or new lints are added to clippy.
#![warn(
    // Enable more useful rustc lints
    absolute_paths_not_starting_with_crate,
    elided_lifetimes_in_paths,
    explicit_outlives_requirements,
    keyword_idents,
    macro_use_extern_crate,
    meta_variable_misuse,
    missing_copy_implementations,
    missing_docs,
    non_ascii_idents,
    noop_method_call,
    pointer_structural_match,
    single_use_lifetimes,
    trivial_casts,
    trivial_numeric_casts,
    unreachable_pub,
    unused_extern_crates,
    unused_import_braces,
    unused_lifetimes,
    unused_macro_rules,
    unused_qualifications,
    unused_tuple_struct_fields,
    variant_size_differences,
    // Enable pedantic clippy lints
    clippy::pedantic,
    // Useful clippy lints for no_std support
    clippy::std_instead_of_core,
    clippy::std_instead_of_alloc,
    clippy::alloc_instead_of_core,
    // Useful clippy lint for unsafe code
    clippy::undocumented_unsafe_blocks,
    // Always require `unsafe` block when using `unsafe`
    // operations, even in `unsafe` functions
    unsafe_op_in_unsafe_fn,
)]
// `clippy::pedantic` exceptions
#![allow(clippy::should_implement_trait, clippy::must_use_candidate)]

use core::marker::PhantomData;

/// Abstraction for an [`&mut Option`][Option] that's known to be [`Some`].
///
/// # Size
///
/// `SomeOption` is the size of two pointers, making it
/// twice the size of `&mut Option`. One points to the
/// value inside, and the other points to the `Option` itself.
pub struct SomeBucket<'a, T> {
    /// Pointer to the `Option<T>::Some`
    outer: *mut Option<T>,
    /// Pointer to the value within the `Option<T>::Some`
    inner: *mut T,
    _life: PhantomData<&'a mut Option<T>>,
}
impl<'a, T> SomeBucket<'a, T> {
    /// Creates a new [`SomeBucket`], without checking that
    /// the input [`Option`] is `Some`.
    ///
    /// It's recommended to use [`SomeBucket::new`] or
    /// [`OptionBucket::new`] instead.
    ///
    /// # Safety
    ///
    /// Caller must guarantee that `opt` is NOT `None`.
    pub unsafe fn new_unchecked(opt: &'a mut Option<T>) -> Self {
        debug_assert!(
            opt.is_some(),
            "Undefined Behavior: `None` value passed to `SomeBucket::new_unchecked`."
        );

        let outer: *mut Option<T> = opt;

        // SAFETY: Caller guarantees that `opt` is `Some`
        let inner: *mut T = unsafe { opt.as_mut().unwrap_unchecked() };

        SomeBucket {
            outer,
            inner,
            _life: PhantomData,
        }
    }

    /// Creates a new [`SomeBucket`]. Returns `Some(SomeBucket<T>)`
    /// if `opt` is [`Some`], otherwise returns `None`.
    ///
    /// For an unchecked version, see [`SomeBucket::new_unchecked`].
    pub fn new(opt: &'a mut Option<T>) -> Option<Self> {
        if opt.is_some() {
            // SAFETY: If conditional ensures that `opt` is `Some`
            unsafe { Some(Self::new_unchecked(opt)) }
        } else {
            None
        }
    }

    /// Converts from `&Option<T>::Some` to `&T`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use option_bucket::SomeBucket;
    ///
    /// let mut text: Option<String> = Some("Hello, world!".to_string());
    /// let some = SomeBucket::new(&mut text).unwrap();
    ///
    /// let hello: &str = &some.as_ref()[..5];
    /// let length: usize = some.as_ref().len();
    /// assert_eq!(hello, "Hello");
    /// assert_eq!(length, 13);
    /// ```
    pub fn as_ref(&self) -> &T {
        // SAFETY:
        // `inner` is a valid pointer to `T`, as guaranteed
        // by invariants of `new`. This can not alias a
        // mutable reference, because any way to produce one
        // requires a unique reference to self
        unsafe { &(*self.inner) }
    }

    /// Converts from `&mut Option<T>::Some` to `&mut T`,
    /// with the lifetime of this `SomeBucket`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use option_bucket::SomeBucket;
    ///
    /// let mut text: Option<String> = Some("Hello, world!".to_string());
    /// let mut some = SomeBucket::new(&mut text).unwrap();
    ///
    /// some.as_mut().push_str(" Happy to be here.");
    /// assert_eq!(some.as_ref(), "Hello, world! Happy to be here.");
    /// ```
    pub fn as_mut(&mut self) -> &mut T {
        // SAFETY:
        // `inner` is a valid pointer to `T`, as guaranteed
        // be invariants of `new`. This can not alias another
        // mutable reference, because any way to produce one
        // requires a unique reference to self
        unsafe { &mut (*self.inner) }
    }

    /// Converts from `&mut Option<T>::Some` to `&mut T`,
    /// with the lifetime of the original reference,
    /// consuming this `SomeBucket`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use option_bucket::SomeBucket;
    ///
    /// let mut text: Option<String> = Some("Hello, world!".to_string());
    /// let some = SomeBucket::new(&mut text).unwrap();
    ///
    /// some.into_mut().push_str(" Happy to be here.");
    /// assert_eq!(&text.unwrap(), "Hello, world! Happy to be here.");
    /// ```
    ///
    /// ```compile_fail
    /// # use option_bucket::SomeBucket;
    ///
    /// let mut text: Option<String> = Some("Hello, world!".to_string());
    /// let some = SomeBucket::new(&mut text).unwrap();
    ///
    /// some.into_mut().push_str(" Happy to be here.");
    /// // can not longer use `some`
    /// some.as_ref();
    /// ```
    pub fn into_mut(self) -> &'a mut T {
        // SAFETY:
        // `inner` is a valid pointer to `T`, as guaranteed
        // be invariants of `new`. This can not alias another
        // mutable reference, because any way to produce one
        // requires a unique reference to self
        unsafe { &mut (*self.inner) }
    }

    /// Sets the value in the `Option<T>::Some`, and returns
    /// the old value.
    ///
    /// # Examples
    ///
    /// ```
    /// # use option_bucket::SomeBucket;
    ///
    /// let mut x = Some(2);
    /// let mut some = SomeBucket::new(&mut x).unwrap();
    ///
    /// let old = some.replace(5);
    /// assert_eq!(old, 2);
    /// assert_eq!(x, Some(5));
    /// ```
    pub fn replace(&mut self, value: T) -> T {
        core::mem::replace(self.as_mut(), value)
    }

    /// Takes the value out of the option, leaving a `None` in its place,
    /// and consuming this `SomeBucket`.
    ///
    /// ```
    /// # use option_bucket::SomeBucket;
    ///
    /// let mut x = Some(vec![1, 2]);
    /// let some = SomeBucket::new(&mut x).unwrap();
    ///
    /// let y = some.take();
    /// assert_eq!(x, None);
    /// assert_eq!(y, vec![1, 2]);
    /// ```
    pub fn take(self) -> T {
        // SAFETY:
        // `inner` is a valid pointer to `T`, as guaranteed
        // be invariants of `new`. We set the original
        // `Option` to `None`, ensuring that the original
        // copy of the value will not be double-dropped
        unsafe {
            let value = self.inner.read();
            self.outer.write(None);
            value
        }
    }
}

/// Abstraction for an [`&mut Option`][Option] that's known to be `None`.
///
/// # Size
///
/// `NoneBucket` is the same size as `&mut Option`
pub struct NoneBucket<'a, T> {
    outer: &'a mut Option<T>,
}
impl<'a, T> NoneBucket<'a, T> {
    /// Creates a new [`NoneBucket`], without checking that
    /// the input [`Option`] is `None`.
    ///
    /// It's recommended to use [`NoneBucket::new`] or
    /// [`OptionBucket::new`] instead.
    ///
    /// # Safety
    ///
    /// Caller must guarantee that `opt` is NOT [`Some`].
    pub unsafe fn new_unchecked(opt: &'a mut Option<T>) -> Self {
        debug_assert!(
            opt.is_none(),
            "Undefined Behavior: `Some` value passed to `NoneBucket::new_unchecked`."
        );

        NoneBucket { outer: opt }
    }

    /// Creates a new [`NoneBucket`]. Returns `Some(NoneBucket<T>)`
    /// if `opt` is [`None`], otherwise returns `None`.
    ///
    /// For an unchecked version, see [`NoneBucket::new_unchecked`].
    pub fn new(opt: &'a mut Option<T>) -> Option<Self> {
        if opt.is_none() {
            // SAFETY: if conditional ensures that `opt` is `None`
            unsafe { Some(Self::new_unchecked(opt)) }
        } else {
            None
        }
    }

    /// Inserts value into the option, then returns a mutable reference to it.
    ///
    /// This is practically identical to [`Option::insert`], but avoids
    /// operations handling [`drop`]ping the old value
    /// (since we know there was no old value).
    /// ```
    /// # use option_bucket::NoneBucket;
    ///
    /// let mut opt = None;
    /// let mut none = NoneBucket::new(&mut opt).unwrap();
    /// let val = none.insert(1);
    /// assert_eq!(*val, 1);
    /// *val = 3;
    /// assert_eq!(opt.unwrap(), 3);
    /// ```
    pub fn insert(self, value: T) -> &'a mut T {
        // SAFETY: `outer` is `None`, so there is no old value to leak
        unsafe {
            let outer: *mut Option<T> = self.outer;
            outer.write(Some(value));
        }

        // SAFETY: the code above just filled the option
        unsafe { self.outer.as_mut().unwrap_unchecked() }
    }
}

/// Recommended entry for getting a [`SomeBucket`] or
/// [`NoneBucket`]. Infallibly convertible from any
/// [`&mut Option`][Option].
pub enum OptionBucket<'a, T> {
    /// An option known to be `Some`.
    Some(SomeBucket<'a, T>),
    /// An option known to be `None`.
    None(NoneBucket<'a, T>),
}
impl<'a, T> OptionBucket<'a, T> {
    /// Create an `OptionBucket` from an `&mut Option`.
    ///
    /// # Examples
    ///
    /// ```
    /// use option_bucket::OptionBucket;
    ///
    /// let mut none: Option<i32> = None;
    /// let none_bucket = OptionBucket::new(&mut none);
    /// assert!(matches!(none_bucket, OptionBucket::None(_)));
    ///
    /// let mut some: Option<i32> = Some(12);
    /// let some_bucket = OptionBucket::new(&mut some);
    /// assert!(matches!(some_bucket, OptionBucket::Some(_)));
    /// ```
    pub fn new(opt: &'a mut Option<T>) -> Self {
        if opt.is_some() {
            // SAFETY: if conditional ensures that `opt` is `Some`
            unsafe { OptionBucket::Some(SomeBucket::new_unchecked(opt)) }
        } else {
            // SAFETY: if conditional ensures that `opt` is `None`
            unsafe { OptionBucket::None(NoneBucket::new_unchecked(opt)) }
        }
    }
}
