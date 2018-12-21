//! A fixed-size Map implementation.
//!
//! This crate provides a map implementation that can make use of a fixed-size backing storage.
//!
//! ## The `Key` derive
//!
//! The `Key` derive is provided to construct optimized storage for a given Key.
//!
//! For example:
//!
//! ```rust
//! use fixed_map::Map;
//!
//! #[derive(Clone, Copy, fixed_map::Key)]
//! enum Part {
//!     One,
//!     Two,
//! }
//!
//! #[derive(Clone, Copy, fixed_map::Key)]
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

#![deny(missing_docs)]

mod key;
mod map;
mod set;
mod storage;

pub use self::key::Key;
pub use self::map::Map;
pub use self::set::Set;
pub use self::storage::{MapStorage, SingletonStorage, Storage};
pub use fixed_map_derive::Key;
