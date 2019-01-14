# fixed-map
[![Build Status](https://travis-ci.org/udoprog/fixed-map.svg?branch=master)](https://travis-ci.org/udoprog/fixed-map)

**Note:** this crate is still in heavy development. Please be careful!

This crate provides a map implementation that can make use of a fixed-size backing storage.

For more information on how to use `fixed-map`, see the [documentation].

[documentation]: https://docs.rs/fixed-map

## The `Key` derive

The `Key` derive is provided to construct optimized storage for a given Key.

For example:

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

## Missing APIs

The API of this project is incomplete since it is experimental.
You can help by adding more!

## Why does this crate exist?

There are many cases where you want associate a value with a small, fixed number of elements
identified by an enum.

For example, let's say you have a game where each room has something in for directions:

```rust
#[derive(Key)]
pub enum Dir {
    North,
    East,
    South,
    West,
}
```

Now we can use this map to associate an item with each direction.

```rust
let mut map = fixed_map::Map::new();
map.insert(Dir::North, Item::Bow);
```

## Performance

The goal is for the performance of fixed-map to be identical to storing the data linearly in memory
like when using `[Option<Key>; N]`.

In the following benchmarks, fixed-map is compared to:

* `fixed` - A `fixed_map::Map` with a derived `Key` with `N` variants.
* [`hashbrown`] - A high performance hash map. This is only included for reference.
  - Note: Maps are created with `HashMap::with_capacity(N)`.
* `array` - A simple `[Option<Key>; N]` array.

Note: for all `insert` benchmarks the underlying map is cloned in each iteration.

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

## Examples

Most examples are in place to test what kind of assembler they compile to.

To do this, run:

```
RUSTFLAGS="--emit asm" cargo build --release --example <example>
```

You should be able to find the assembler generated in the target folder:

```
ls target/release/examples/
```

## LICENSE

This project is distributed under the terms of both the MIT license and the Apache License (Version
2.0).

This project contains code derived from [HashMap] (Rust stdlib).

[HashMap]: https://github.com/rust-lang/rust/blob/2c1a715cbda1d6eba39625aca08f1f2ac7c0dcc8/src/libstd/collections/hash/map.rs
