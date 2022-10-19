#![cfg(feature = "entry")]

use fixed_map::{Key, Map};

#[derive(Clone, Copy, Key)]
enum Part {
    One,
    Two,
}

#[test]
fn simple() {
    let mut map: Map<Part, i32> = Map::new();

    map.insert(Part::One, 1);
    assert_eq!(map.entry(Part::Two).or_default(), &0);
    assert_eq!(
        map.entry(Part::One).and_modify(|x| *x += 1).or_insert(12),
        &2
    );
}

#[cfg(feature = "entry")]
const _: () = {
    #[derive(Clone, Copy, Key)]
    enum Key {
        Simple,
        Composite(Part),
        String(&'static str),
        Number(u32),
        Singleton(()),
    }
};
