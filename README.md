# fixed-map
[![Build Status](https://travis-ci.org/udoprog/fixed-map.svg?branch=master)](https://travis-ci.org/udoprog/fixed-map)

**Note:** this crate is still in heavy development. Please be careful!

This crate provides a map implementation that relies on fixed-size backing storage.

It accomplishes this by deriving an optimal storage strategy from the _key_ to be used in the map
using the `Key` derive:

```rust
#[derive(fixed_map::Key)]
pub enum Key {
    One,
    Two,
}

let mut map = fixed_map::Map::new();
assert_eq!(map.get(&Key::One), None);
map.insert(Key::One, 42);
assert_eq!(map.get(&Key::One), Some(&42));
assert_eq!(map.get(&Key::Two), None);
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

## LICENSE

This project is distributed under the terms of both the MIT license and the Apache License (Version
2.0).

This project contains code derived from [HashMap] (Rust stdlib).

[HashMap]: https://github.com/rust-lang/rust/blob/2c1a715cbda1d6eba39625aca08f1f2ac7c0dcc8/src/libstd/collections/hash/map.rs
