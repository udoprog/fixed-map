use fixed_map::Key;

#[derive(Debug, Clone, Copy, Key)]
enum Key {
    First,
    Second,
}

#[test]
fn test_clone() {
    use fixed_map::Map;

    let mut a = Map::new();
    a.insert(Key::First, 42);

    let b = a.clone();
    assert_eq!(b.get(Key::First).cloned(), Some(42));
    assert_eq!(b.get(Key::Second).cloned(), None);
}

#[test]
fn test_eq() {
    use fixed_map::Map;

    let mut a = Map::new();
    a.insert(Key::First, 42);

    let mut b = a.clone();
    assert_eq!(a, b);

    b.insert(Key::Second, 42);
    assert_ne!(a, b);
}

#[test]
fn test_debug() {
    use fixed_map::Map;

    let mut a = Map::new();
    a.insert(Key::First, 42);

    assert_eq!("{First: 42}", format!("{:?}", a))
}
