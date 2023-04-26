#![cfg(feature = "hashbrown")]

use fixed_map::{Key, Map};

#[derive(Clone, Copy, Key)]
enum Part {
    One,
    Two,
}

#[derive(Clone, Copy, Key)]
enum MyKey {
    Simple,
    Composite(Part),
    String(&'static str),
    Number(u32),
    Singleton(()),
}

#[test]
fn map_feature() {
    let mut map = Map::new();

    map.insert(MyKey::Simple, 1);
    map.insert(MyKey::Composite(Part::One), 2);
    map.insert(MyKey::String("foo"), 3);
    map.insert(MyKey::Number(1), 4);
    map.insert(MyKey::Singleton(()), 5);

    assert_eq!(map.get(MyKey::Simple), Some(&1));
    assert_eq!(map.get(MyKey::Composite(Part::One)), Some(&2));
    assert_eq!(map.get(MyKey::Composite(Part::Two)), None);
    assert_eq!(map.get(MyKey::String("foo")), Some(&3));
    assert_eq!(map.get(MyKey::String("bar")), None);
    assert_eq!(map.get(MyKey::Number(1)), Some(&4));
    assert_eq!(map.get(MyKey::Number(2)), None);
    assert_eq!(map.get(MyKey::Singleton(())), Some(&5));
}
