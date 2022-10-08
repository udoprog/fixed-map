[<img alt="github" src="https://img.shields.io/badge/github-udoprog/fixed-map-8da0cb?style=for-the-badge&logo=github" height="20">](https://github.com/udoprog/fixed-map)
[<img alt="crates.io" src="https://img.shields.io/crates/v/fixed-map.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/fixed-map)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-fixed-map-66c2a5?style=for-the-badge&logoColor=white&logo=data:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K" height="20">](https://docs.rs/fixed-map)
[<img alt="build status" src="https://img.shields.io/github/workflow/status/udoprog/fixed-map/CI/main?style=for-the-badge" height="20">](https://github.com/udoprog/fixed-map/actions?query=branch%3Amain)

# fixed-map

This crate provides a map implementation that can make use of a fixed-size
backing storage. It enables the compiler to heavily optimize map lookups by
translating them into pattern matching over strictly defined enums.
Potentially allowing for interesting performance characteristics.

For more information on how to use, see the [documentation].

### Features

The following features are available:

* `map` - Causes [Storage] to be implemented by dynamic types such as
  `&'static str` or `u32`. These are backed by a `hashbrown` HashMap
  (default).
* `serde` - Causes [Map] and [Set] to implement [Serialize] and
  [Deserialize] if it's implemented by the key and value.

### Deriving `Key`

The [Key derive] is provided to instruct the `fixed-map` containers on how
to build optimized storage for a given Key. We also require the key to
implement [Copy] for it to implement `Key`.

```rust
use fixed_map::{Key, Map};

#[derive(Clone, Copy, Key)]
enum Part {
    One,
    Two,
}

#[derive(Clone, Copy, Key)]
enum Key {
    Simple,
    Composite(Part),
    String(&'static str),
    Number(u32),
    Singleton(()),
}

let mut map = Map::new();

map.insert(Key::Simple, 1);
map.insert(Key::Composite(Part::One), 2);
map.insert(Key::String("foo"), 3);
map.insert(Key::Number(1), 4);
map.insert(Key::Singleton(()), 5);

assert_eq!(map.get(Key::Simple), Some(&1));
assert_eq!(map.get(Key::Composite(Part::One)), Some(&2));
assert_eq!(map.get(Key::Composite(Part::Two)), None);
assert_eq!(map.get(Key::String("foo")), Some(&3));
assert_eq!(map.get(Key::String("bar")), None);
assert_eq!(map.get(Key::Number(1)), Some(&4));
assert_eq!(map.get(Key::Number(2)), None);
assert_eq!(map.get(Key::Singleton(())), Some(&5));
```

### Why does this crate exist?

There are many cases where you want associate a value with a small, fixed
number of elements identified by an enum.

For example, let's say you have a game where each room has something in four
directions. We can model this relationship between the direction and the
item using two enums.

```rust
pub enum Dir {
    North,
    East,
    South,
    West,
}

pub enum Item {
    Bow,
    Sword,
    Axe,
}
```

The goal is for the performance of fixed-map to be identical to storing the
data linearly in memory like you could by storing the data as an array like
`[Option<Item>; N]` where each index correspondings to each variant in
`Dir`.

Doing this yourself could look like this:

```rust
#[repr(usize)]
pub enum Dir {
    North,
    East,
    South,
    West,
}

#[derive(Debug)]
pub enum Item {
    Bow,
    Sword,
    Axe,
}

let mut map: [Option<Item>; 4] = [None, None, None, None];
map[Dir::North as usize] = Some(Item::Bow);

if let Some(item) = &map[Dir::North as usize] {
    println!("found item: {:?}", item);
}
```

But with `fixed-map` you can do it like this without (hopefully) incurring
any drop in performance:

```rust
use fixed_map::{Key, Map};

#[derive(Clone, Copy, Key)]
pub enum Dir {
    North,
    East,
    South,
    West,
}

#[derive(Debug)]
pub enum Item {
    Bow,
    Sword,
    Axe,
}

let mut map = Map::new();
map.insert(Dir::North, Item::Bow);

if let Some(item) = map.get(Dir::North) {
    println!("found item: {:?}", item);
}
```

### Unsafe use

This crate uses unsafe for its iterators. This is needed because there is no
proper way to associate generic lifetimes to associated types.

Instead, we associate the lifetime to the container (`Map` or `Set`) which
wraps a set of unsafe derefs over raw pointers.

### Benchmarks

In the following benchmarks, fixed-map is compared to:

* `fixed` - A `fixed_map::Map` with a derived `Key` with `N` variants.
* [`hashbrown`] - A high performance hash map. This is only included for
  reference.
  - Note: Maps are created with `HashMap::with_capacity(N)`.
* `array` - A simple `[Option<Key>; N]` array.

Note: for all `insert` benchmarks the underlying map is cloned in each
iteration.

```
fixed/get4              time:   [253.97 ps 255.78 ps 257.75 ps]
fixed/get8              time:   [257.56 ps 259.70 ps 261.82 ps]
fixed/get16             time:   [253.47 ps 255.78 ps 258.44 ps]
fixed/get32             time:   [245.90 ps 246.58 ps 247.35 ps]

array/get4              time:   [490.51 ps 491.72 ps 493.00 ps]
array/get8              time:   [514.83 ps 519.88 ps 525.49 ps]
array/get16             time:   [520.87 ps 528.57 ps 538.60 ps]
array/get32             time:   [515.18 ps 525.02 ps 537.46 ps]

hashbrown/get4          time:   [3.4101 ns 3.4419 ns 3.4792 ns]
hashbrown/get8          time:   [3.3051 ns 3.3220 ns 3.3437 ns]
hashbrown/get16         time:   [3.2872 ns 3.2961 ns 3.3058 ns]
hashbrown/get32         time:   [3.2700 ns 3.2783 ns 3.2869 ns]

fixed/insert4           time:   [494.04 ps 495.74 ps 497.61 ps]
fixed/insert8           time:   [752.24 ps 755.71 ps 760.33 ps]
fixed/insert16          time:   [1.3142 ns 1.3196 ns 1.3252 ns]
fixed/insert32          time:   [2.1143 ns 2.1203 ns 2.1259 ns]

array/insert4           time:   [500.43 ps 504.79 ps 509.23 ps]
array/insert8           time:   [767.07 ps 769.24 ps 771.50 ps]
array/insert16          time:   [1.3343 ns 1.3386 ns 1.3431 ns]
array/insert32          time:   [2.1609 ns 2.1668 ns 2.1730 ns]

hashbrown/insert4       time:   [85.362 ns 86.288 ns 87.712 ns]
hashbrown/insert8       time:   [96.778 ns 97.102 ns 97.482 ns]
hashbrown/insert16      time:   [122.12 ns 122.56 ns 123.05 ns]
hashbrown/insert32      time:   [166.31 ns 167.78 ns 169.47 ns]

fixed/iter4             time:   [2.0091 ns 2.0141 ns 2.0191 ns]
fixed/iter8             time:   [6.4643 ns 6.4953 ns 6.5312 ns]
fixed/iter16            time:   [46.684 ns 46.839 ns 47.006 ns]
fixed/iter32            time:   [99.789 ns 100.03 ns 100.28 ns]

array/iter4             time:   [6.5876 ns 6.6116 ns 6.6361 ns]
array/iter8             time:   [6.4052 ns 6.4235 ns 6.4438 ns]
array/iter16            time:   [7.5715 ns 7.6424 ns 7.7706 ns]
array/iter32            time:   [10.104 ns 10.174 ns 10.281 ns]

hashbrown/iter4         time:   [3.7405 ns 3.7538 ns 3.7672 ns]
hashbrown/iter8         time:   [4.5252 ns 4.5433 ns 4.5692 ns]
hashbrown/iter16        time:   [8.5868 ns 8.6118 ns 8.6375 ns]
hashbrown/iter32        time:   [12.868 ns 12.898 ns 12.928 ns]
```

[`hashbrown`]: https://github.com/Amanieu/hashbrown

### Examples

Most examples are in place to test what kind of assembler they compile to.

To do this, run:

```sh
RUSTFLAGS="--emit asm" cargo build --release --example <example>
```

You should be able to find the assembler generated in the target folder:

```sh
ls target/release/examples/
```

[Copy]: https://doc.rust-lang.org/std/marker/trait.Copy.html
[Deserialize]: https://docs.rs/serde/1/serde/trait.Deserialize.html
[documentation]: https://docs.rs/fixed-map
[Key derive]: https://docs.rs/fixed-map/*/fixed_map/derive.Key.html
[Map]: https://docs.rs/fixed-map/*/fixed_map/map/struct.Map.html
[Serialize]: https://docs.rs/serde/1/serde/trait.Serialize.html
[Set]: https://docs.rs/fixed-map/*/fixed_map/map/struct.Set.html
