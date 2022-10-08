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
//! * `map` - Causes [Storage] to be implemented by dynamic types such as
//!   `&'static str` or `u32`. These are backed by a `hashbrown` HashMap
//!   (default).
//! * `serde` - Causes [Map] and [Set] to implement [Serialize] and
//!   [Deserialize] if it's implemented by the key and value.
//!
//! ## Deriving `Key`
//!
//! The [Key derive] is provided to instruct the `fixed-map` containers on how
//! to build optimized storage for a given Key. We also require the key to
//! implement [Copy] for it to implement `Key`.
//!
//! ```rust
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
//!     String(&'static str),
//!     Number(u32),
//!     Singleton(()),
//! }
//!
//! let mut map = Map::new();
//!
//! map.insert(Key::Simple, 1);
//! map.insert(Key::Composite(Part::One), 2);
//! map.insert(Key::String("foo"), 3);
//! map.insert(Key::Number(1), 4);
//! map.insert(Key::Singleton(()), 5);
//!
//! assert_eq!(map.get(Key::Simple), Some(&1));
//! assert_eq!(map.get(Key::Composite(Part::One)), Some(&2));
//! assert_eq!(map.get(Key::Composite(Part::Two)), None);
//! assert_eq!(map.get(Key::String("foo")), Some(&3));
//! assert_eq!(map.get(Key::String("bar")), None);
//! assert_eq!(map.get(Key::Number(1)), Some(&4));
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
//! ```rust
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
//! ```rust
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
//! get/fixed/4             time:   [211.20 ps 211.61 ps 212.06 ps]
//! get/fixed/8             time:   [210.09 ps 211.65 ps 213.61 ps]
//! get/fixed/16            time:   [210.94 ps 212.20 ps 213.97 ps]
//! get/fixed/32            time:   [209.48 ps 210.00 ps 210.55 ps]
//!
//! get/hashbrown/4         time:   [2.9004 ns 2.9068 ns 2.9137 ns]
//! get/hashbrown/8         time:   [2.9575 ns 2.9649 ns 2.9738 ns]
//! get/hashbrown/16        time:   [2.9513 ns 2.9580 ns 2.9655 ns]
//! get/hashbrown/32        time:   [2.9391 ns 2.9462 ns 2.9533 ns]
//!
//! get/array/4             time:   [217.27 ps 219.37 ps 222.77 ps]
//! get/array/8             time:   [220.93 ps 223.42 ps 226.52 ps]
//! get/array/16            time:   [217.84 ps 218.26 ps 218.73 ps]
//! get/array/32            time:   [217.30 ps 218.28 ps 219.88 ps]
//!
//! insert/fixed/4          time:   [427.55 ps 429.60 ps 431.87 ps]
//! insert/fixed/8          time:   [638.18 ps 641.55 ps 646.20 ps]
//! insert/fixed/16         time:   [1.0662 ns 1.0705 ns 1.0755 ns]
//! insert/fixed/32         time:   [1.7118 ns 1.7194 ns 1.7282 ns]
//!
//! insert/hashbrown/4      time:   [57.832 ns 58.001 ns 58.190 ns]
//! insert/hashbrown/8      time:   [70.735 ns 71.018 ns 71.379 ns]
//! insert/hashbrown/16     time:   [91.086 ns 94.213 ns 97.604 ns]
//! insert/hashbrown/32     time:   [119.31 ns 120.15 ns 121.22 ns]
//!
//! insert/array/4          time:   [424.26 ps 428.23 ps 432.54 ps]
//! insert/array/8          time:   [641.01 ps 642.00 ps 643.69 ps]
//! insert/array/16         time:   [1.0672 ns 1.0725 ns 1.0806 ns]
//! insert/array/32         time:   [1.6412 ns 1.6482 ns 1.6555 ns]
//!
//! iter/fixed/4            time:   [10.800 ns 10.865 ns 10.929 ns]
//! iter/fixed/8            time:   [23.932 ns 24.194 ns 24.460 ns]
//! iter/fixed/16           time:   [63.797 ns 64.442 ns 65.156 ns]
//! iter/fixed/32           time:   [419.05 ns 422.97 ns 427.21 ns]
//!
//! iter/hashbrown/4        time:   [2.6698 ns 2.6759 ns 2.6836 ns]
//! iter/hashbrown/8        time:   [3.3816 ns 3.3906 ns 3.4004 ns]
//! iter/hashbrown/16       time:   [5.3848 ns 5.4060 ns 5.4270 ns]
//! iter/hashbrown/32       time:   [8.2614 ns 8.3419 ns 8.4313 ns]
//!
//! iter/array/4            time:   [213.45 ps 213.93 ps 214.74 ps]
//! iter/array/8            time:   [213.42 ps 214.08 ps 214.92 ps]
//! iter/array/16           time:   [214.07 ps 215.12 ps 216.59 ps]
//! iter/array/32           time:   [213.99 ps 214.95 ps 216.22 ps]
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

#![deny(missing_docs)]

pub mod key;

pub mod map;
pub use self::map::Map;

pub mod set;
pub use self::set::Set;

pub mod storage;

#[doc(inline)]
pub use fixed_map_derive::Key;
