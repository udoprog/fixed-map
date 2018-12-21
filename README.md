# fixed-map
[![Build Status](https://travis-ci.org/udoprog/fixed-map.svg?branch=master)](https://travis-ci.org/udoprog/fixed-map)

**Note:** this crate is still in heavy development. Please be careful!

This crate provides a map implementation that relies on fixed-size backing storage.

It accomplishes this by deriving an optimal storage strategy from the _key_ to be used in the map
using the `Key` derive:

```rust
#[derive(Clone, Copy, fixed_map::Key)]
pub enum Part {
    First,
    Second,
}

#[derive(Clone, Copy, fixed_map::Key)]
pub enum Key {
    Simple,
    Composite(Part),
}

let mut map = fixed_map::Map::new();
assert_eq!(map.get(Key::Simple), None);

map.insert(Key::Simple, 1);
map.insert(Key::Composite(Part::One), 2);

assert_eq!(map.get(Key::Simple), Some(&1));
assert_eq!(map.get(Key::Composite(Part::One)), Some(&2));
assert_eq!(map.get(Key::Composite(Part::Two)), None);
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
fixed/get4              time:   [240.19 ps 241.25 ps 242.46 ps]
fixed/get8              time:   [236.54 ps 237.33 ps 238.19 ps]
fixed/get16             time:   [255.66 ps 256.90 ps 258.11 ps]
fixed/get32             time:   [1.8133 ns 1.8212 ns 1.8284 ns]

array/get4              time:   [516.23 ps 519.27 ps 521.86 ps]
array/get8              time:   [519.22 ps 521.55 ps 523.52 ps]
array/get16             time:   [473.06 ps 474.45 ps 475.91 ps]
array/get32             time:   [516.33 ps 518.58 ps 520.51 ps]

hashbrown/get4          time:   [3.1502 ns 3.1804 ns 3.2172 ns]
hashbrown/get8          time:   [3.1468 ns 3.1666 ns 3.1921 ns]
hashbrown/get16         time:   [3.0559 ns 3.0616 ns 3.0693 ns]
hashbrown/get32         time:   [3.4007 ns 3.4204 ns 3.4364 ns]

fixed/insert4           time:   [255.35 ps 256.28 ps 257.33 ps]
fixed/insert8           time:   [254.84 ps 256.28 ps 257.89 ps]
fixed/insert16          time:   [12.732 ns 12.789 ns 12.850 ns]
fixed/insert32          time:   [19.602 ns 19.720 ns 19.868 ns]

array/insert4           time:   [258.83 ps 260.36 ps 262.32 ps]
array/insert8           time:   [258.48 ps 259.14 ps 259.86 ps]
array/insert16          time:   [268.11 ps 271.70 ps 275.91 ps]
array/insert32          time:   [256.29 ps 257.54 ps 258.83 ps]

hashbrown/insert4       time:   [32.985 ns 33.108 ns 33.238 ns]
hashbrown/insert8       time:   [32.479 ns 32.566 ns 32.673 ns]
hashbrown/insert16      time:   [35.995 ns 36.180 ns 36.334 ns]
hashbrown/insert32      time:   [37.159 ns 37.383 ns 37.570 ns]

fixed/iter4             time:   [253.92 ps 258.03 ps 262.92 ps]
fixed/iter8             time:   [238.28 ps 238.74 ps 239.26 ps]
fixed/iter16            time:   [5.7057 ns 5.7416 ns 5.7852 ns]
fixed/iter32            time:   [7.7160 ns 7.7793 ns 7.8619 ns]

array/iter4             time:   [248.58 ps 252.06 ps 256.00 ps]
array/iter8             time:   [256.90 ps 258.58 ps 260.22 ps]
array/iter16            time:   [259.71 ps 261.15 ps 262.78 ps]
array/iter32            time:   [257.05 ps 258.52 ps 260.07 ps]

hashbrown/iter4         time:   [1.4265 ns 1.4359 ns 1.4485 ns]
hashbrown/iter8         time:   [1.9639 ns 1.9830 ns 2.0062 ns]
hashbrown/iter16        time:   [3.6334 ns 3.6486 ns 3.6630 ns]
hashbrown/iter32        time:   [7.4109 ns 7.4662 ns 7.5130 ns]
```

[`hashbrown`]: https://github.com/Amanieu/hashbrown

## LICENSE

This project is distributed under the terms of both the MIT license and the Apache License (Version
2.0).

This project contains code derived from [HashMap] (Rust stdlib).

[HashMap]: https://github.com/rust-lang/rust/blob/2c1a715cbda1d6eba39625aca08f1f2ac7c0dcc8/src/libstd/collections/hash/map.rs
