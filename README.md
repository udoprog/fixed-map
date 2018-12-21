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
use fixed_map::Map;

#[derive(Clone, Copy, fixed_map::Key)]
enum Part {
    One,
    Two,
}

#[derive(Clone, Copy, fixed_map::Key)]
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
#[derive(fixed_map::Key)]
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
fixed/get4              time:   [244.72 ps 246.69 ps 248.79 ps]
fixed/get8              time:   [237.30 ps 237.71 ps 238.24 ps]
fixed/get16             time:   [256.93 ps 258.53 ps 259.80 ps]
fixed/get32             time:   [235.62 ps 235.99 ps 236.38 ps]

array/get4              time:   [517.19 ps 518.88 ps 520.36 ps]
array/get8              time:   [511.14 ps 512.83 ps 514.46 ps]
array/get16             time:   [506.10 ps 509.83 ps 513.22 ps]
array/get32             time:   [478.93 ps 480.89 ps 482.95 ps]

hashbrown/get4          time:   [3.0849 ns 3.0924 ns 3.1010 ns]
hashbrown/get8          time:   [3.0917 ns 3.1057 ns 3.1213 ns]
hashbrown/get16         time:   [3.0759 ns 3.0813 ns 3.0873 ns]
hashbrown/get32         time:   [3.0871 ns 3.0985 ns 3.1116 ns]

fixed/insert4           time:   [473.71 ps 474.98 ps 476.45 ps]
fixed/insert8           time:   [716.10 ps 718.24 ps 720.88 ps]
fixed/insert16          time:   [1.2674 ns 1.2730 ns 1.2795 ns]
fixed/insert32          time:   [2.1538 ns 2.1698 ns 2.1848 ns]

array/insert4           time:   [509.23 ps 511.78 ps 514.19 ps]
array/insert8           time:   [759.61 ps 765.17 ps 770.49 ps]
array/insert16          time:   [1.3557 ns 1.3639 ns 1.3710 ns]
array/insert32          time:   [2.1022 ns 2.1154 ns 2.1344 ns]

hashbrown/insert4       time:   [48.671 ns 49.152 ns 49.610 ns]
hashbrown/insert8       time:   [53.619 ns 53.703 ns 53.806 ns]
hashbrown/insert16      time:   [72.105 ns 72.210 ns 72.340 ns]
hashbrown/insert32      time:   [107.33 ns 107.52 ns 107.76 ns]

fixed/iter4             time:   [237.63 ps 237.94 ps 238.30 ps]
fixed/iter8             time:   [236.91 ps 237.21 ps 237.57 ps]
fixed/iter16            time:   [237.61 ps 238.06 ps 238.57 ps]
fixed/iter32            time:   [238.33 ps 238.87 ps 239.42 ps]

array/iter4             time:   [239.11 ps 240.03 ps 241.25 ps]
array/iter8             time:   [238.40 ps 238.94 ps 239.49 ps]
array/iter16            time:   [253.86 ps 254.84 ps 255.72 ps]
array/iter32            time:   [254.73 ps 255.67 ps 256.50 ps]

hashbrown/iter4         time:   [1.5091 ns 1.5182 ns 1.5272 ns]
hashbrown/iter8         time:   [2.0387 ns 2.0610 ns 2.0827 ns]
hashbrown/iter16        time:   [3.3735 ns 3.3957 ns 3.4190 ns]
hashbrown/iter32        time:   [5.9786 ns 6.0241 ns 6.0869 ns]
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
