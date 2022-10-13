//! [<img alt="github" src="https://img.shields.io/badge/github-udoprog/fixed--map-8da0cb?style=for-the-badge&logo=github" height="20">](https://github.com/udoprog/fixed-map)
//! [<img alt="crates.io" src="https://img.shields.io/crates/v/fixed-map.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/fixed-map)
//! [<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-fixed--map-66c2a5?style=for-the-badge&logoColor=white&logo=data:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K" height="20">](https://docs.rs/fixed-map)
//! [<img alt="build status" src="https://img.shields.io/github/workflow/status/udoprog/fixed-map/CI/main?style=for-the-badge" height="20">](https://github.com/udoprog/fixed-map/actions?query=branch%3Amain)
//!
//! This crate provides a map implementation that can make use of a fixed-size
//! backing storage. It enables the compiler to heavily optimize map lookups by
//! translating them into pattern matching over strictly defined enums.
//! Potentially allowing for interesting performance characteristics.
//!
//! For more information on how to use, see the [documentation].
//!
//! ## Features
//!
//! The following features are available:
//!
//! * `std` - Disabling this feature enables causes this crate to be no-std.
//!   This means that dynamic types cannot be used in keys, like ones enabled by
//!   the `map` feature (default).
//! * `map` - Causes [Storage] to be implemented by dynamic types such as
//!   `&'static str` or `u32`. These are backed by a `hashbrown` (default).
//! * `serde` - Causes [Map] and [Set] to implement [Serialize] and
//!   [Deserialize] if it's implemented by the key and value.
//!
//! ## Deriving `Key`
//!
//! The [Key derive] is provided to instruct the `fixed-map` containers on how
//! to build optimized storage for a given Key. We also require the key to
//! implement [Copy] for it to implement `Key`.
//!
//! ```
//! use fixed_map::{Key, Map};
//!
//! #[derive(Clone, Copy, Key)]
//! enum Part {
//!     One,
//!     Two,
//! }
//!
//! #[derive(Clone, Copy, Key)]
//! enum Key {
//!     Simple,
//!     Composite(Part),
//! # #[cfg(feature = "map")]
//!     String(&'static str),
//! # #[cfg(feature = "map")]
//!     Number(u32),
//!     Singleton(()),
//! }
//!
//! let mut map = Map::new();
//!
//! map.insert(Key::Simple, 1);
//! map.insert(Key::Composite(Part::One), 2);
//! # #[cfg(feature = "map")]
//! map.insert(Key::String("foo"), 3);
//! # #[cfg(feature = "map")]
//! map.insert(Key::Number(1), 4);
//! map.insert(Key::Singleton(()), 5);
//!
//! assert_eq!(map.get(Key::Simple), Some(&1));
//! assert_eq!(map.get(Key::Composite(Part::One)), Some(&2));
//! assert_eq!(map.get(Key::Composite(Part::Two)), None);
//! # #[cfg(feature = "map")]
//! assert_eq!(map.get(Key::String("foo")), Some(&3));
//! # #[cfg(feature = "map")]
//! assert_eq!(map.get(Key::String("bar")), None);
//! # #[cfg(feature = "map")]
//! assert_eq!(map.get(Key::Number(1)), Some(&4));
//! # #[cfg(feature = "map")]
//! assert_eq!(map.get(Key::Number(2)), None);
//! assert_eq!(map.get(Key::Singleton(())), Some(&5));
//! ```
//!
//! ## Why does this crate exist?
//!
//! There are many cases where you want associate a value with a small, fixed
//! number of elements identified by an enum.
//!
//! For example, let's say you have a game where each room has something in four
//! directions. We can model this relationship between the direction and the
//! item using two enums.
//!
//! ```
//! pub enum Dir {
//!     North,
//!     East,
//!     South,
//!     West,
//! }
//!
//! pub enum Item {
//!     Bow,
//!     Sword,
//!     Axe,
//! }
//! ```
//!
//! The goal is for the performance of fixed-map to be identical to storing the
//! data linearly in memory like you could by storing the data as an array like
//! `[Option<Item>; N]` where each index correspondings to each variant in
//! `Dir`.
//!
//! Doing this yourself could look like this:
//!
//! ```
//! #[repr(usize)]
//! pub enum Dir {
//!     North,
//!     East,
//!     South,
//!     West,
//! }
//!
//! #[derive(Debug)]
//! pub enum Item {
//!     Bow,
//!     Sword,
//!     Axe,
//! }
//!
//! let mut map: [Option<Item>; 4] = [None, None, None, None];
//! map[Dir::North as usize] = Some(Item::Bow);
//!
//! if let Some(item) = &map[Dir::North as usize] {
//!     println!("found item: {:?}", item);
//! }
//! ```
//!
//! But with `fixed-map` you can do it like this without (hopefully) incurring
//! any drop in performance:
//!
//! ```
//! use fixed_map::{Key, Map};
//!
//! #[derive(Clone, Copy, Key)]
//! pub enum Dir {
//!     North,
//!     East,
//!     South,
//!     West,
//! }
//!
//! #[derive(Debug)]
//! pub enum Item {
//!     Bow,
//!     Sword,
//!     Axe,
//! }
//!
//! let mut map = Map::new();
//! map.insert(Dir::North, Item::Bow);
//!
//! if let Some(item) = map.get(Dir::North) {
//!     println!("found item: {:?}", item);
//! }
//! ```
//!
//! ## Unsafe use
//!
//! This crate uses unsafe for its iterators. This is needed because there is no
//! proper way to associate generic lifetimes to associated types.
//!
//! Instead, we associate the lifetime to the container (`Map` or `Set`) which
//! wraps a set of unsafe derefs over raw pointers.
//!
//! ## Benchmarks
//!
//! In the following benchmarks, fixed-map is compared to:
//!
//! * `fixed` - A `fixed_map::Map` with a derived `Key` with `N` variants.
//! * [`hashbrown`] - A high performance hash map. This is only included for
//!   reference.
//!   - Note: Maps are created with `HashMap::with_capacity(N)`.
//! * `array` - A simple `[Option<Key>; N]` array.
//!
//! Note: for all `insert` benchmarks the underlying map is cloned in each
//! iteration.
//!
//! ```text
//! get/fixed/4             time:   [208.96 ps 209.57 ps 210.17 ps]
//! get/fixed/8             time:   [211.12 ps 211.86 ps 212.55 ps]
//! get/fixed/16            time:   [211.50 ps 211.84 ps 212.23 ps]
//! get/fixed/32            time:   [211.02 ps 211.40 ps 211.79 ps]
//! get/array/4             time:   [215.76 ps 216.56 ps 217.68 ps]
//! get/array/8             time:   [216.80 ps 217.28 ps 217.83 ps]
//! get/array/16            time:   [215.88 ps 216.21 ps 216.58 ps]
//! get/array/32            time:   [216.39 ps 216.82 ps 217.33 ps]
//! get/hashbrown/4         time:   [2.9134 ns 2.9168 ns 2.9210 ns]
//! get/hashbrown/8         time:   [2.9143 ns 2.9175 ns 2.9212 ns]
//! get/hashbrown/16        time:   [2.9258 ns 2.9293 ns 2.9328 ns]
//! get/hashbrown/32        time:   [2.9387 ns 2.9428 ns 2.9466 ns]
//!
//! insert/fixed/4          time:   [421.82 ps 422.47 ps 423.22 ps]
//! insert/fixed/8          time:   [635.46 ps 636.91 ps 638.55 ps]
//! insert/fixed/16         time:   [1.0579 ns 1.0599 ns 1.0621 ns]
//! insert/fixed/32         time:   [1.6991 ns 1.7016 ns 1.7043 ns]
//! insert/array/4          time:   [419.26 ps 419.76 ps 420.30 ps]
//! insert/array/8          time:   [624.30 ps 626.27 ps 628.33 ps]
//! insert/array/16         time:   [1.0444 ns 1.0467 ns 1.0490 ns]
//! insert/array/32         time:   [1.6828 ns 1.6904 ns 1.6990 ns]
//! insert/hashbrown/4      time:   [87.002 ns 87.233 ns 87.475 ns]
//! insert/hashbrown/8      time:   [96.995 ns 97.287 ns 97.589 ns]
//! insert/hashbrown/16     time:   [517.89 ns 518.66 ns 519.57 ns]
//! insert/hashbrown/32     time:   [156.10 ns 156.67 ns 157.30 ns]
//!
//! values/fixed/4          time:   [209.09 ps 209.51 ps 209.91 ps]
//! values/fixed/8          time:   [213.99 ps 215.34 ps 217.08 ps]
//! values/fixed/16         time:   [213.24 ps 213.94 ps 214.72 ps]
//! values/fixed/32         time:   [212.71 ps 213.82 ps 215.15 ps]
//! values/array/4          time:   [211.07 ps 211.78 ps 212.59 ps]
//! values/array/8          time:   [211.48 ps 212.03 ps 212.65 ps]
//! values/array/16         time:   [213.04 ps 213.49 ps 213.99 ps]
//! values/array/32         time:   [213.18 ps 213.78 ps 214.60 ps]
//! values/hashbrown/4      time:   [3.3965 ns 3.4007 ns 3.4056 ns]
//! values/hashbrown/8      time:   [3.8443 ns 3.8627 ns 3.8895 ns]
//! values/hashbrown/16     time:   [5.6312 ns 5.6666 ns 5.7029 ns]
//! values/hashbrown/32     time:   [8.7221 ns 8.7674 ns 8.8117 ns]
//!
//! array/sum_values        time:   [3.0394 ns 3.0463 ns 3.0534 ns]
//! fixed/sum_values        time:   [3.0503 ns 3.0559 ns 3.0619 ns]
//! ```
//!
//! [`hashbrown`]: https://github.com/Amanieu/hashbrown
//!
//! ## Examples
//!
//! Most examples are in place to test what kind of assembler they compile to.
//!
//! To do this, run:
//!
//! ```sh
//! RUSTFLAGS="--emit asm" cargo build --release --example <example>
//! ```
//!
//! You should be able to find the assembler generated in the target folder:
//!
//! ```sh
//! ls target/release/examples/
//! ```
//!
//! [Copy]: https://doc.rust-lang.org/std/marker/trait.Copy.html
//! [Deserialize]: https://docs.rs/serde/1/serde/trait.Deserialize.html
//! [documentation]: https://docs.rs/fixed-map
//! [Key derive]: https://docs.rs/fixed-map/*/fixed_map/derive.Key.html
//! [Map]: https://docs.rs/fixed-map/*/fixed_map/map/struct.Map.html
//! [Serialize]: https://docs.rs/serde/1/serde/trait.Serialize.html
//! [Set]: https://docs.rs/fixed-map/*/fixed_map/map/struct.Set.html
//! [Storage]: https://docs.rs/fixed-map/*/fixed_map/storage/trait.Storage.html

#![no_std]
#![forbid(unsafe_code)]
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
    clippy::alloc_instead_of_core
)]
// `clippy::pedantic` exceptions
#![allow(
    // style choice
    clippy::module_name_repetitions,
    // false positive
    clippy::type_repetition_in_bounds,
    // false positive
    clippy::expl_impl_clone_on_copy
)]

pub mod key;

pub mod map;
pub use self::map::Map;

pub mod set;
pub use self::set::Set;

pub mod storage;

#[doc(inline)]
pub use fixed_map_derive::Key;
