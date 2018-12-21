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

The goal is for the performance of fixed-map to be identical to storing the data linearly in memory.

In the following benchmarks, fixed-map is compared to:

* [`hashbrown`] - a high performance hash map.
* `array`, which is a simple `[Option<Key>; 4]` array.

```
get/fixed_map           time:   [255.71 ps 257.67 ps 259.98 ps]
get/array               time:   [512.06 ps 514.43 ps 516.83 ps]
get/hashbrown           time:   [3.2981 ns 3.3287 ns 3.3607 ns]

insert/fixed_map        time:   [254.23 ps 255.44 ps 257.31 ps]
insert/array            time:   [274.18 ps 275.29 ps 276.47 ps]
insert/hashbrown        time:   [35.409 ns 35.635 ns 35.861 ns]

iter/fixed_map          time:   [249.76 ps 251.58 ps 253.69 ps]
iter/array              time:   [256.04 ps 259.76 ps 264.41 ps]
iter/hashbrown          time:   [1.5128 ns 1.5322 ns 1.5534 ns]
```

[`hashbrown`]: https://github.com/Amanieu/hashbrown

## LICENSE

This project is distributed under the terms of both the MIT license and the Apache License (Version
2.0).

This project contains code derived from [HashMap] (Rust stdlib).

[HashMap]: https://github.com/rust-lang/rust/blob/2c1a715cbda1d6eba39625aca08f1f2ac7c0dcc8/src/libstd/collections/hash/map.rs
