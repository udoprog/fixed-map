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
    Singleton(()),
}

fn main() {
    let mut map = Map::new();
    map.insert(Key::Composite(Part::One), 42);
    assert_eq!(map.get(Key::Composite(Part::One)), Some(&42));
    assert_eq!(map.get(Key::Simple), None);
}
