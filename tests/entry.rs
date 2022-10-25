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

    assert_eq!(map.entry(Part::Two).or_default(), &0);
    assert_eq!(
        map.entry(Part::One).and_modify(|x| *x += 1).or_insert(12),
        &12
    );
    assert_eq!(
        map.entry(Part::One).and_modify(|x| *x += 1).or_insert(12),
        &13
    );
}

#[test]
fn other() {
    use fixed_map::{Key, Map};

    #[derive(Clone, Copy, Key)]
    enum Key {
        Even,
        Odd,
    }

    let mut map: Map<Key, u32> = Map::new();

    for n in [3, 45, 3, 23, 2, 10, 59, 11, 51, 70] {
        map.entry(if n % 2 == 0 { Key::Even } else { Key::Odd })
            .and_modify(|x| *x += 1)
            .or_insert(1);
    }

    assert_eq!(map.get(Key::Even), Some(&3));
    assert_eq!(map.get(Key::Odd), Some(&7));
}

#[test]
fn composite() {
    use fixed_map::{Key, Map};

    #[derive(Clone, Copy, Key)]
    enum Key {
        First(bool),
        Second,
    }

    let mut map: Map<Key, Vec<i32>> = Map::new();

    map.entry(Key::First(true)).or_default().push(1);
    map.entry(Key::Second)
        .or_insert_with(|| vec![2; 8])
        .truncate(4);

    assert_eq!(map.get(Key::First(true)), Some(&vec![1]));
    assert_eq!(map.get(Key::Second), Some(&vec![2; 4]));
}

#[cfg(feature = "map")]
#[test]
fn compound() {
    #[derive(Clone, Copy, Key)]
    enum Key {
        Simple,
        Composite(Part),
        String(&'static str),
        Number(u32),
        Singleton(()),
    }

    let mut map: Map<Key, i32> = Map::new();

    map.insert(Key::Composite(Part::One), 1);
    assert_eq!(map.entry(Key::Composite(Part::Two)).or_default(), &0);
    assert_eq!(
        map.entry(Key::Composite(Part::One))
            .and_modify(|x| *x += 1)
            .or_insert(12),
        &2
    );
}
