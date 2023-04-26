use fixed_map::{Key, Map};

#[derive(Debug, Clone, Copy, Key)]
enum MyKey {
    First,
    Second,
}

fn main() {
    let mut map = Map::new();
    map.insert(MyKey::First, 42);
    assert_eq!(map.get(MyKey::First), Some(&42));
    assert_eq!(map.get(MyKey::Second), None);
}
