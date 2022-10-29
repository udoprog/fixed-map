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
    Number(u32),
    Singleton(()),
    String(&'static str),
}

fn main() {
    let mut map = Map::new();
    map.insert(Key::Number(42), 42);
    assert_eq!(map.get(Key::Number(42)), Some(&42));
    assert_eq!(map.get(Key::Simple), None);
}
