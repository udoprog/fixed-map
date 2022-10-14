use fixed_map::{Key, Map};

#[derive(Debug, Clone, Copy, Key)]
enum Key {
    First,
    Second,
}

fn main() {
    let mut map = Map::new();
    map.insert(Key::First, 42);
    assert_eq!(map.get(Key::First), Some(&42));
    assert_eq!(map.get(Key::Second), None);
}

// Execute this during testing as well.
#[cfg(test)]
#[test]
fn test_main() {
    main();
}
