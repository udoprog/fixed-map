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

* [`hashbrown`] - a high performance hash map created with `HashMap::with_capacity`.
* `array`, which is a simple `[Option<Key>; N]` array.

Note: for the `insert` benchmark the underlying map is cloned in each iteration.

```
get_bench4/fixed_map    time:   [233.38 ps 233.72 ps 234.15 ps]
get_bench8/fixed_map    time:   [241.19 ps 245.81 ps 251.53 ps]
get_bench16/fixed_map   time:   [1.3941 ns 1.3957 ns 1.3981 ns]

get_bench4/array        time:   [464.69 ps 465.17 ps 465.85 ps]
get_bench8/array        time:   [478.36 ps 484.59 ps 492.95 ps]
get_bench16/array       time:   [464.86 ps 465.65 ps 466.71 ps]

get_bench4/hashbrown    time:   [3.0677 ns 3.0753 ns 3.0829 ns]
get_bench8/hashbrown    time:   [3.1362 ns 3.1653 ns 3.1957 ns]
get_bench16/hashbrown   time:   [3.0293 ns 3.0328 ns 3.0373 ns]

insert_bench4/fixed_map time:   [251.08 ps 252.22 ps 253.44 ps]
insert_bench8/fixed_map time:   [5.9293 ns 5.9604 ns 5.9946 ns]
insert_bench16/fixed_map
                        time:   [12.549 ns 12.561 ns 12.577 ns]

insert_bench4/array     time:   [251.16 ps 251.45 ps 251.82 ps]
insert_bench8/array     time:   [263.79 ps 267.37 ps 270.81 ps]
insert_bench16/array    time:   [250.07 ps 250.42 ps 250.83 ps]

insert_bench4/hashbrown time:   [32.153 ns 32.373 ns 32.734 ns]
insert_bench8/hashbrown time:   [33.509 ns 33.825 ns 34.176 ns]
insert_bench16/hashbrown
                        time:   [32.113 ns 32.160 ns 32.220 ns]

iter_bench4/fixed_map   time:   [233.42 ps 234.21 ps 235.07 ps]
iter_bench8/fixed_map   time:   [232.64 ps 233.39 ps 234.38 ps]
iter_bench16/fixed_map  time:   [5.0975 ns 5.1016 ns 5.1055 ns]

iter_bench4/array       time:   [245.52 ps 248.31 ps 251.31 ps]
iter_bench8/array       time:   [232.42 ps 232.71 ps 233.08 ps]
iter_bench16/array      time:   [232.45 ps 232.83 ps 233.37 ps]

iter_bench4/hashbrown   time:   [1.4189 ns 1.4311 ns 1.4446 ns]
iter_bench8/hashbrown   time:   [1.8590 ns 1.8609 ns 1.8635 ns]
iter_bench16/hashbrown  time:   [3.2537 ns 3.2579 ns 3.2635 ns]
```

[`hashbrown`]: https://github.com/Amanieu/hashbrown

## LICENSE

This project is distributed under the terms of both the MIT license and the Apache License (Version
2.0).

This project contains code derived from [HashMap] (Rust stdlib).

[HashMap]: https://github.com/rust-lang/rust/blob/2c1a715cbda1d6eba39625aca08f1f2ac7c0dcc8/src/libstd/collections/hash/map.rs
