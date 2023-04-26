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
    Singleton(()),
}

fn main() {
    let mut map = Map::new();
    map.insert(MyKey::Composite(Part::One), 42);
    assert_eq!(map.get(MyKey::Composite(Part::One)), Some(&42));
    assert_eq!(map.get(MyKey::Simple), None);
}
